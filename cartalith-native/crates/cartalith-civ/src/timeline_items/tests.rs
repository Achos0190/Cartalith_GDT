//! Tests for [`super`]: one literal row per source (CT-1's bar), the filters, the private rule and
//! the Chronos export's round trip (CT-8's bar).

use super::*;
use crate::conflict::ConflictKind;
use cartalith_vault::timelines::{detect_forms, EventsDoc, Form};

fn conflict(id: u64, start: i64, end: Option<i64>, anchor: Option<ConflictAnchor>) -> Conflict {
    Conflict {
        id, name: format!("War {id}"), kind: ConflictKind::Siege, start_year: start, end_year: end,
        sides: vec![1, 2], outcome: String::new(), points: vec![(0.0, 0.0), (1.0, 1.0)], anchor, anchor_at: (0.0, 0.0),
    }
}

fn find<'a>(v: &'a [TimelineItem], key: &str) -> &'a TimelineItem {
    v.iter().find(|i| i.key == key).unwrap_or_else(|| panic!("no item {key}: {:?}", v.iter().map(|i| &i.key).collect::<Vec<_>>()))
}

/// Protects each source's literal row: founding is a year-precision point on the settlement with
/// its owner; `founded: None` emits NOTHING (not year 0); ownership spans carry open_end; a conflict
/// without an end is open, not a point; a journey with a day gets a dated range and one without a
/// plan is a point; snapshots are year ticks.
#[test]
fn one_literal_row_per_source() {
    let src = ItemSources {
        founding: vec![
            FoundingInput { id: 42, name: "Kelv".into(), founded: Some(-340), faction: 2 },
            FoundingInput { id: 43, name: "Orsa".into(), founded: None, faction: 1 },
        ],
        snapshot_years: vec![0, 100],
        ownership: vec![OwnershipInput {
            tid: 42, name: "Kelv".into(),
            spans: vec![
                OwnershipSpan { start_year: 0, end_year: Some(99), faction_id: 2 },
                OwnershipSpan { start_year: 100, end_year: None, faction_id: 3 },
            ],
        }],
        conflicts: vec![conflict(7, 1198, Some(1204), Some(ConflictAnchor::Settlement(42))), conflict(8, 640, None, None)],
        journeys: vec![
            JourneyInput { id: 3, name: "Envoy".into(), start_year: 1203, start_day: Some(60), calendar_days: Some(70) },
            JourneyInput { id: 4, name: "Walk".into(), start_year: 1203, start_day: None, calendar_days: None },
        ],
        ..Default::default()
    };
    let v = build(&src);
    let f = find(&v, "founded:settlement:42");
    assert_eq!((f.start, f.end, f.open_end, f.precision, f.entity, f.factions.clone()), ((-340, None), None, false, Precision::Year, Some((EntityKind::Settlement, 42)), vec![2]));
    assert_eq!(f.title, "Kelv founded");
    assert!(v.iter().all(|i| i.key != "founded:settlement:43"), "founded None emits no item");
    assert_eq!(find(&v, "snapshot:100").source, ItemSource::Snapshot);
    let open = find(&v, "span:settlement:42:100");
    assert_eq!((open.end, open.open_end, open.precision), (None, true, Precision::Range));
    let closed = find(&v, "span:settlement:42:0");
    assert_eq!((closed.end, closed.open_end), (Some((99, None)), false));
    let c = find(&v, "conflict:7");
    assert_eq!((c.start.0, c.end.map(|e| e.0), c.entity, c.factions.clone()), (1198, Some(1204), Some((EntityKind::Settlement, 42)), vec![1, 2]));
    assert!(find(&v, "conflict:8").open_end && find(&v, "conflict:8").end.is_none());
    // 1 March (day 59 is 28 Feb + ... ): day 60 is 2 March; +70 days is 11 May
    let j = find(&v, "journey:3");
    assert_eq!(chronos::format_date(j.start.0, j.start.1), "1203-03-02");
    let e = j.end.unwrap();
    assert_eq!(chronos::format_date(e.0, e.1), "1203-05-11");
    assert_eq!(j.precision, Precision::Range);
    let w = find(&v, "journey:4");
    assert_eq!((w.end, w.open_end, w.precision), (None, false, Precision::Year), "no plan: a point, dated by year");
}

/// Protects "a point and an open range are different things": the two have the same `end: None`
/// and only `open_end` tells them apart.
#[test]
fn a_point_and_an_open_range_are_distinguished() {
    let src = ItemSources { conflicts: vec![conflict(1, 10, None, None)], founding: vec![FoundingInput { id: 1, name: "A".into(), founded: Some(10), faction: 1 }], ..Default::default() };
    let v = build(&src);
    let (a, b) = (find(&v, "conflict:1"), find(&v, "founded:settlement:1"));
    assert_eq!((a.end, b.end), (None, None));
    assert_ne!((a.open_end, a.precision), (b.open_end, b.precision));
}

/// Protects note and imported rows: note events carry their origin, private and entity; an imported
/// row keeps its day and its "source removed" mark in the title; the list is sorted by date then key.
#[test]
fn note_and_imported_rows_and_sort_order() {
    let c = detect_forms("```chronos\n* [1204-03-14] Coronation\n- [1190~1195] Flood\n```\n", &[Form::Chronos]).remove(0);
    let mut doc = EventsDoc::default();
    doc.import(&c, "v", "K.md", "settlement", 42, 0, true);
    doc.events[0].source_removed = true;
    let src = ItemSources {
        note_events: vec![NoteEventsInput {
            link_id: "L1".into(), vault_id: "v".into(), relative_path: "K.md".into(),
            entity: Some((EntityKind::Settlement, 42)), factions: vec![2], private: false, events: c.events.clone(),
        }],
        imported: doc.events.clone(),
        ..Default::default()
    };
    // imported onto settlement 42 AND linked to it: one row per line, not two (CT view issue 3)
    let v = build(&src);
    assert_eq!(v.len(), 2);
    let years: Vec<i64> = v.iter().map(|i| i.start.0).collect();
    assert_eq!(years, vec![1190, 1204], "sorted by start");
    assert!(v.iter().all(|i| i.source == ItemSource::Imported && i.also_linked));
    // the same note linked to a DIFFERENT entity than the import is not the same event: all four show
    let mut other = src.clone();
    other.note_events[0].entity = Some((EntityKind::Faction, 2));
    let w = build(&other);
    assert_eq!(w.len(), 4);
    let n = w.iter().find(|i| i.source == ItemSource::NoteEvent && i.title == "Coronation").unwrap();
    assert_eq!(n.origin.as_ref().unwrap().relative_path, "K.md");
    assert!(!n.private && n.entity == Some((EntityKind::Faction, 2)) && !n.also_linked);
    let i = w.iter().find(|i| i.source == ItemSource::Imported && i.title.starts_with("Coronation")).unwrap();
    assert_eq!(i.title, "Coronation (source removed)");
    assert!(i.private);
    assert_eq!(chronos::format_date(i.start.0, i.start.1), "1204-03-14");
    assert_eq!(build(&src), v, "a rebuild is reproducible");
}

/// Protects the filters: each axis alone, AND across groups, OR inside one, and the hidden count.
#[test]
fn filters_and_hidden_count() {
    let src = ItemSources {
        founding: vec![FoundingInput { id: 42, name: "Kelv".into(), founded: Some(1), faction: 2 }],
        conflicts: vec![conflict(7, 5, Some(6), Some(ConflictAnchor::Settlement(42))), conflict(8, 7, None, None)],
        snapshot_years: vec![3],
        ..Default::default()
    };
    let v = build(&src);
    assert_eq!(v.len(), 4);
    let all = apply(&v, &Filter::default());
    assert_eq!((all.visible.len(), all.hidden), (4, 0));
    let kind = apply(&v, &Filter { kinds: vec![ItemSource::Conflict], ..Default::default() });
    assert_eq!((kind.visible.len(), kind.hidden), (2, 2));
    let ent = apply(&v, &Filter { entities: vec![(EntityKind::Settlement, 42)], ..Default::default() });
    assert_eq!(ent.visible.len(), 2, "the founding and the anchored conflict");
    let fac = apply(&v, &Filter { factions: vec![2], ..Default::default() });
    assert_eq!(fac.visible.len(), 3, "founding (owner 2) and both conflicts (sides 1,2)");
    let and = apply(&v, &Filter { kinds: vec![ItemSource::Conflict], entities: vec![(EntityKind::Settlement, 42)], factions: vec![] });
    assert_eq!((and.visible.len(), and.hidden), (1, 3));
    let two_kinds = apply(&v, &Filter { kinds: vec![ItemSource::Conflict, ItemSource::Snapshot], ..Default::default() });
    assert_eq!(two_kinds.visible.len(), 3, "OR inside a group");
}

/// Protects the track widening: the extent covers a founding at -340 and a range end, and is None
/// for nothing (never (0, 0)).
#[test]
fn year_extent_reaches_foundings() {
    assert_eq!(year_extent(&[]), None);
    let src = ItemSources { founding: vec![FoundingInput { id: 1, name: "A".into(), founded: Some(-340), faction: 1 }], conflicts: vec![conflict(1, 10, Some(900), None)], ..Default::default() };
    assert_eq!(year_extent(&build(&src)), Some((-340, 900)));
}

/// Protects the private rule for the assistant and exports: a private note's items are dropped by
/// `without_private` AND never written by `export_chronos` even when handed to it.
#[test]
fn private_items_are_hidden_and_never_exported() {
    let ev = cartalith_vault::chronos::parse_line("* [1204] Secret coup").unwrap();
    let src = ItemSources {
        note_events: vec![NoteEventsInput { link_id: "L".into(), vault_id: "v".into(), relative_path: "P.md".into(), entity: None, factions: vec![], private: true, events: vec![ev] }],
        conflicts: vec![conflict(1, 10, Some(20), None)],
        ..Default::default()
    };
    let v = build(&src);
    assert_eq!(v.len(), 2);
    assert_eq!(without_private(&v).len(), 1);
    let all: Vec<&TimelineItem> = v.iter().collect();
    let x = export_chronos(&all);
    assert!(!x.text.contains("Secret"), "a private item is never written: {}", x.text);
    assert_eq!(x.written, 1);
}

/// CT-8's bar: the exported text parses with `chronos::parse` with `skipped` EMPTY, each event
/// round-trips through `to_line`/`parse_line`, an open range is written as a point, a title that
/// cannot be written (empty after sanitising) is reported not dropped.
#[test]
fn chronos_export_round_trips() {
    let src = ItemSources {
        founding: vec![FoundingInput { id: 1, name: "Kelv".into(), founded: Some(-340), faction: 1 }],
        conflicts: vec![conflict(1, 10, Some(20), None), conflict(2, 640, None, None)],
        journeys: vec![JourneyInput { id: 3, name: "Envoy | to Orsa".into(), start_year: 1203, start_day: Some(60), calendar_days: Some(70) }],
        ..Default::default()
    };
    let v = build(&src);
    let all: Vec<&TimelineItem> = v.iter().collect();
    let x = export_chronos(&all);
    assert_eq!(x.skipped, Vec::<String>::new());
    assert_eq!(x.written, 4);
    let p = chronos::parse(&x.text);
    assert_eq!((p.blocks, p.skipped.len(), p.events.len()), (1, 0, 4));
    assert!(x.text.starts_with("```chronos\n# generated by Cartalith; edit freely, it is not read back\n"));
    for i in &all {
        let e = to_event(i);
        assert_eq!(chronos::parse_line(&chronos::to_line(&e).unwrap()), Some(e));
    }
    let open = p.events.iter().find(|e| e.name == "War 2").unwrap();
    assert_eq!((open.kind, open.end), (Kind::Point, None), "an open range has no Chronos form");
    let j = p.events.iter().find(|e| e.name.starts_with("Envoy")).unwrap();
    assert_eq!(j.name, "Envoy   to Orsa", "a `|` in a title becomes a space, not a description split");
    assert_eq!(chronos::format_date(j.start, j.start_md), "1203-03-02");
    // an unwritable title is reported
    let mut odd = build(&ItemSources { conflicts: vec![conflict(9, 1, Some(2), None)], ..Default::default() });
    odd[0].title = "||".into();
    let y = export_chronos(&odd.iter().collect::<Vec<_>>());
    assert_eq!((y.written, y.skipped.clone()), (0, vec!["||".to_string()]));
}

/// Protects the Kind filter's wire words: each source parses back from its label, case-insensitively.
#[test]
fn item_source_labels_round_trip() {
    for k in ItemSource::ALL {
        assert_eq!(ItemSource::parse(k.label()), Some(k));
        assert_eq!(ItemSource::parse(&k.label().to_lowercase()), Some(k));
    }
    assert_eq!(ItemSource::parse("nope"), None);
}

/// Protects the owner's case (CT view issue 3): a 21-event note linked AND imported counts 21, not 43,
/// including a shifted import (`epoch_offset` moves the years, never the line identity), a point and a
/// range keep their real forms once each, and an imported-only or linked-only note is untouched.
#[test]
fn linked_and_imported_note_counts_once() {
    let mut text = String::from("```chronos
");
    for i in 0..19 {
        text.push_str(&format!("* [{}] Point {i}
", 3000 + i * 90));
    }
    text.push_str("- [3000~4000] Long reign
@ [4100~4769] Late age
```
");
    let c = detect_forms(&text, &[Form::Chronos]).remove(0);
    assert_eq!(c.events.len(), 21);
    let mut doc = EventsDoc::default();
    doc.import(&c, "v", "K.md", "settlement", 42, 100, false);
    let linked = NoteEventsInput {
        link_id: "L1".into(), vault_id: "v".into(), relative_path: "K.md".into(),
        entity: Some((EntityKind::Settlement, 42)), factions: vec![], private: false, events: c.events.clone(),
    };
    let both = build(&ItemSources { note_events: vec![linked.clone()], imported: doc.events.clone(), ..Default::default() });
    assert_eq!(both.len(), 21, "was 42 before the cross-source dedup");
    let ranges = both.iter().filter(|i| i.end.is_some()).count();
    assert_eq!(ranges, 2, "a range stays one range row, a point one point row");
    assert!(both.iter().all(|i| i.also_linked));
    let only_linked = build(&ItemSources { note_events: vec![linked], ..Default::default() });
    let only_imported = build(&ItemSources { imported: doc.events.clone(), ..Default::default() });
    assert_eq!((only_linked.len(), only_imported.len()), (21, 21));
    assert!(only_imported.iter().all(|i| !i.also_linked));
}

/// Protects CT view issue 1: the strip widens its track to the Note and Imported lanes only, so the
/// extent is taken over the rows a filter keeps -- a founding at -635 must not drag the track back.
#[test]
fn year_extent_over_a_filtered_view_ignores_other_lanes() {
    let c = detect_forms("```chronos
* [3000] A
- [4100~4769] B
```
", &[Form::Chronos]).remove(0);
    let src = ItemSources {
        founding: vec![FoundingInput { id: 1, name: "Kelv".into(), founded: Some(-635), faction: 1 }],
        note_events: vec![NoteEventsInput {
            link_id: "L".into(), vault_id: "v".into(), relative_path: "K.md".into(),
            entity: Some((EntityKind::Settlement, 1)), factions: vec![], private: false, events: c.events,
        }],
        ..Default::default()
    };
    let v = build(&src);
    assert_eq!(year_extent(&v), Some((-635, 4769)));
    let notes = apply(&v, &Filter { kinds: vec![ItemSource::NoteEvent], ..Default::default() });
    assert_eq!(year_extent(notes.visible.iter().copied()), Some((3000, 4769)));
}
