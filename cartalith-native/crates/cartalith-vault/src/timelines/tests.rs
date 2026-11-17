//! Tests for [`super`]: each says what it protects. The CT-0 labelled sample
//! (`ct0_*`) is **synthetic** -- see the module doc -- and its precision and
//! recall are asserted as literals so loosening a rule moves a number.

use super::*;

fn of(text: &str, form: Form) -> Vec<Candidate> {
    detect_forms(text, &[form])
}

/// Protects F1 (the parser itself): two blocks give two candidates with ordinals 0 and 1, the
/// events, the skipped line listed, and a different hash per block.
#[test]
fn f1_chronos_blocks_are_separate_candidates() {
    let t = "# Kelv\n```chronos\n- [1204-03-14] Coronation | grand\n* [-340] Founded\nnot a line\n```\ntext\n```chronos\n@ [100~200] Reign\n```\n";
    let c = of(t, Form::Chronos);
    assert_eq!(c.len(), 2);
    assert_eq!((c[0].ordinal, c[1].ordinal), (0, 1));
    assert_eq!(c[0].events.len(), 2);
    assert_eq!(c[0].events[0].start, 1204);
    assert_eq!(c[0].events[0].start_md.unwrap().month, 3);
    assert_eq!(c[0].events[1].start, -340);
    assert_eq!(c[0].skipped, vec!["not a line".to_string()]);
    assert_eq!(c[1].events[0].end, Some(200));
    assert_ne!(c[0].block_hash, c[1].block_hash);
    assert_eq!(c[0].heading.as_deref(), Some("Kelv"));
}

/// Protects "an empty or code block is not a timeline": a non-chronos fence is skipped whole, and a
/// year list inside it is not read.
#[test]
fn code_fences_are_not_scanned() {
    let t = "## Timeline\n```text\n- 1200: a\n- 1201: b\n```\n";
    assert!(detect(t).is_empty());
}

/// Protects the closed key list: `founded` and `year` are read (negative and `YYYY-MM-DD` too);
/// `date`, `created`, `modified`, `updated` never are.
#[test]
fn f2_reads_only_the_closed_key_list() {
    let t = "---\nfounded: -340\nborn: 1204-03-14\ndate: 2024-05-01\ncreated: 2024-05-01\nmodified: 2024\nupdated: 2025\ntitle: Kelv\n---\nbody\n";
    let c = of(t, Form::Frontmatter);
    assert_eq!(c.len(), 1);
    let names: Vec<&str> = c[0].events.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["founded", "born"]);
    assert_eq!(c[0].founded_year(), Some(-340));
    assert_eq!(c[0].events[1].start_md.unwrap().day, Some(14));
    // a note with only real-world date keys has no timeline at all
    assert!(of("---\ndate: 2024-05-01\ncreated: 2024-01-01\n---\n", Form::Frontmatter).is_empty());
}

/// Protects F2 against a non-date value: `founded: long ago` and `year: 12x` read nothing.
#[test]
fn f2_ignores_values_that_are_not_years() {
    assert!(of("---\nfounded: long ago\nyear: 12x\nborn: [1, 2]\n---\n", Form::Frontmatter).is_empty());
}

/// Protects F3's two admission rules and its heading match: a headed list of two counts, an
/// unheaded ascending run of four counts, an unheaded run of three does not, `Historical` is not a
/// timeline heading.
#[test]
fn f3_admission_rules() {
    assert_eq!(of("## History\n- 1204: Coronation\n- 1210: War\n", Form::YearList).len(), 1);
    assert_eq!(of("## Notes\n- 1: a\n- 2: b\n- 3: c\n- 4: d\n", Form::YearList).len(), 1);
    assert!(of("## Notes\n- 1204: a\n- 1210: b\n- 1220: c\n", Form::YearList).is_empty());
    assert!(of("## Historical\n- 1204: a\n- 1210: b\n", Form::YearList).is_empty());
    assert!(of("## Eventsourcing
- 1204: a
- 1210: b
", Form::YearList).is_empty(), "whole words only");
    assert!(of("## History\n- 1204: only one\n", Form::YearList).is_empty());
}

/// Protects F3's item grammar: all the forms the design names parse, `- 3 cats` does not.
#[test]
fn f3_item_forms() {
    let t = "## Timeline\n- 1204: Coronation\n- **1210** - War\n- 1215-1220 Siege\n- -340: Founded\n- 3 cats\n";
    let c = of(t, Form::YearList);
    // `- 3 cats` ends the run, so the four real items are one candidate
    assert_eq!(c.len(), 1);
    let ev = &c[0].events;
    assert_eq!(ev.len(), 4);
    assert_eq!((ev[0].start, ev[0].name.as_str()), (1204, "Coronation"));
    assert_eq!((ev[1].start, ev[1].name.as_str()), (1210, "War"));
    assert_eq!((ev[2].start, ev[2].end), (1215, Some(1220)));
    assert_eq!(ev[3].start, -340);
}

/// Protects F4's guards: needs a Year/Date header, a separator row and three year rows.
#[test]
fn f4_table_rules() {
    let ok = "| Year | Event | Note |\n|---|---|---|\n| 1204 | Coronation | grand |\n| 1210 | War | |\n| 1220 | Peace | |\n";
    let c = of(ok, Form::Table);
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].events.len(), 3);
    assert_eq!(c[0].events[0].description.as_deref(), Some("grand"));
    let two = "| Year | Event |\n|---|---|\n| 1204 | a |\n| 1210 | b |\n";
    assert!(of(two, Form::Table).is_empty());
    let nohead = "| Name | Event |\n|---|---|\n| 1204 | a |\n| 1210 | b |\n| 1220 | c |\n";
    assert!(of(nohead, Form::Table).is_empty());
}

fn count(t: &str, f: Form) -> usize {
    of(t, f).len()
}

/// CT-0, F3: a hand-labelled synthetic sample. `POS` are timelines a person would call one; `NEG`
/// are not. The assertion is on the literal tally so a looser heading rule or admission rule moves
/// the number (the mutant is named in the build note).
#[test]
fn ct0_year_list_precision_meets_the_gate() {
    const POS: &[&str] = &[
        "## Timeline\n- 1204: Coronation\n- 1210: War\n",
        "## History\n- **1204** - Coronation\n- **1210** - War\n- **1230** - Peace\n",
        "## Chronology\n- 1204-1210 Siege\n- 1211: Peace\n",
        "## Annals\n- 310: a\n- 312: b\n",
        "## Events\n- -340: Founded\n- -300: Walls\n",
        "## Dates\n- 1204: a\n- 1100: b\n",
        "## Kelv Timeline\n- 100: a\n- 120: b\n",
        "## Notes\n- 100: a\n- 120: b\n- 150: c\n- 190: d\n",
        "## Misc\n- 1: a\n- 40: b\n- 90: c\n- 300: d\n- 700: e\n",
        "# Orsa\n\n## History of Orsa\n\n- 900: Settled\n\n- 950: Burned\n",
        "## Timeline\n- 1204 - Coronation\n- 1210 - War\n",
        "## Timelines\n* 1204: Coronation\n* 1210: War\n",
    ];
    const NEG: &[&str] = &[
        "## Shopping\n- 3 cats\n- 2 dogs\n- 4 eggs\n- 6 pears\n",
        "## Recipe\n- 200 g flour\n- 100 g sugar\n- 3 eggs\n",
        "## Cast\n- 12 Angry Men\n- 7 Samurai\n",
        "## Notes\n- 1204: a\n- 1210: b\n- 1220: c\n",
        "## Historical\n- 1204: a\n- 1210: b\n",
        "## Stats\n- 1204 soldiers\n- 1210 horses\n",
        "## Plan\n- 5: three\n- 4: two\n- 3: one\n- 2: z\n",
        "## Mixed\n- 1204: a\n- note\n- 1210: b\n- 1220: c\n",
        "## Prose\nIn 1204 the king died. In 1210 the war began.\n",
        "## History\n- 1204: only one\n",
        "## Ranks\n- 1: gold\n- 2: silver\n- 3: bronze\n- 4: wood\n", // a realistic false positive
    ];
    let tp = POS.iter().filter(|t| count(t, Form::YearList) == 1).count();
    let fp = NEG.iter().filter(|t| count(t, Form::YearList) >= 1).count();
    let precision = tp as f64 / (tp + fp) as f64;
    assert_eq!((tp, fp, POS.len(), NEG.len()), (12, 1, 12, 11), "recall 12/12, one acknowledged false positive");
    assert!(precision >= 0.90, "F3 precision {precision} below the CT-0 gate");
}

/// CT-0, F4, same arrangement.
#[test]
fn ct0_table_precision_meets_the_gate() {
    const POS: &[&str] = &[
        "| Year | Event |\n|---|---|\n| 1204 | a |\n| 1210 | b |\n| 1220 | c |\n",
        "| Name | Year |\n|:--|--:|\n| Kelv | -340 |\n| Orsa | -210 |\n| Dun | -90 |\n",
        "| Date | Event | Who |\n|---|---|---|\n| 1204-03-14 | Coronation | Kelv |\n| 1210 | War | Orsa |\n| 1215-1220 | Siege | Dun |\n",
        "| **Year** | What |\n|---|---|\n| **100** | a |\n| 200 | b |\n| 300 | c |\n| 400 | d |\n",
        "| year | event |\n| --- | --- |\n| 9 | a |\n| 99 | b |\n| 999 | c |\n",
        "## Annals\n\n| Date | Event |\n|---|---|\n| 310 | a |\n| 320 | b |\n| 330 | c |\n",
    ];
    const NEG: &[&str] = &[
        "| Name | Score |\n|---|---|\n| a | 1204 |\n| b | 1210 |\n| c | 1220 |\n",
        "| Year | Event |\n|---|---|\n| 1204 | a |\n| 1210 | b |\n",
        "| Year | Event |\n|---|---|\n| soon | a |\n| later | b |\n| never | c |\n",
        "| Item | Price |\n|---|---|\n| bread | 2 |\n| milk | 3 |\n| eggs | 4 |\n",
        "| Date | Meeting |\n|---|---|\n| 2024-05-01 | standup |\n| 2024-05-08 | standup |\n| 2024-05-15 | standup |\n", // real-world dates: refused by the real-world guard
    ];
    let tp = POS.iter().filter(|t| count(t, Form::Table) == 1).count();
    let fp = NEG.iter().filter(|t| count(t, Form::Table) >= 1).count();
    let precision = tp as f64 / (tp + fp) as f64;
    assert_eq!((tp, fp, POS.len(), NEG.len()), (6, 0, 6, 5));
    // the guard is what removes the one false positive: a worldbuilding date in the same shape still reads
    assert_eq!(count("| Date | E |
|---|---|
| 1204-03-14 | a |
| 1210-01-02 | b |
| 1220-05-06 | c |
", Form::Table), 1);
    assert!(precision >= 0.90, "F4 precision {precision} below the CT-0 gate");
}

/// Protects the standing rules (design §4.4): undecided counts; decided does not; a changed block
/// resurfaces with the previous decision; an ignore resurfaces only when the hash changes.
#[test]
fn standing_and_badge() {
    let c = of("```chronos\n* [100] a\n```\n", Form::Chronos).remove(0);
    let mut d: Vec<TimelineDecision> = Vec::new();
    assert_eq!(standing(&d, "v", "K.md", &c), Standing::Undecided);
    let mk = |dec, hash: &str| TimelineDecision {
        vault_id: "v".into(), relative_path: "K.md".into(), form: Form::Chronos, ordinal: 0,
        block_hash: hash.into(), decision: dec, entity_kind: None, entity_id: None, epoch_offset: None, decided: 5,
    };
    record(&mut d, mk(Decision::Ignore, &c.block_hash));
    assert_eq!(standing(&d, "v", "K.md", &c), Standing::Decided(Decision::Ignore));
    assert_eq!(badge_count(&d, "v", &[("K.md".into(), c.clone())]), 0);
    let c2 = of("```chronos\n* [100] a\n* [101] b\n```\n", Form::Chronos).remove(0);
    assert_eq!(standing(&d, "v", "K.md", &c2), Standing::Changed(Decision::Ignore));
    assert_eq!(badge_count(&d, "v", &[("K.md".into(), c2)]), 1);
    record(&mut d, mk(Decision::Keep, &c.block_hash));
    assert_eq!(d.len(), 1, "a re-decision replaces, never duplicates");
    assert_eq!(standing(&d, "other", "K.md", &c), Standing::Undecided, "another vault is another key");
}

/// Protects the decision record's JSON shape (a `vault.json` row) with an absent optional omitted.
#[test]
fn decision_json_omits_absent_members() {
    let d = TimelineDecision {
        vault_id: "v".into(), relative_path: "K.md".into(), form: Form::YearList, ordinal: 2,
        block_hash: "h".into(), decision: Decision::Keep, entity_kind: None, entity_id: None, epoch_offset: None, decided: 9,
    };
    let s = serde_json::to_string(&d).unwrap();
    assert_eq!(s, r#"{"vault_id":"v","relative_path":"K.md","form":"year_list","ordinal":2,"block_hash":"h","decision":"keep","decided":9}"#);
    assert_eq!(serde_json::from_str::<TimelineDecision>(&s).unwrap(), d);
}

/// Protects import: epoch offset applied to engine years, dedup on the second call, private
/// inherited, and the day kept; remove_ids is the undo.
#[test]
fn import_offsets_dedups_and_undoes() {
    let c = of("```chronos\n- [1204-03-14] Coronation\n@ [100~200] Reign\n```\n", Form::Chronos).remove(0);
    let mut doc = EventsDoc::default();
    let before = doc.to_json();
    let r = doc.import(&c, "v", "K.md", "settlement", 42, -1000, true);
    assert_eq!(r.added, vec![1, 2]);
    assert_eq!(doc.events[0].start, 204);
    assert_eq!(doc.events[0].start_month, Some(3));
    assert_eq!(doc.events[0].start_day, Some(14));
    assert_eq!(doc.events[1].end, Some(-800));
    assert!(doc.events.iter().all(|e| e.private && e.entity_id == 42));
    let again = doc.import(&c, "v", "K.md", "settlement", 42, -1000, true);
    assert_eq!((again.added.len(), again.duplicates), (0, 2), "a duplicate import adds nothing");
    let other = doc.import(&c, "v", "K.md", "settlement", 43, -1000, false);
    assert_eq!(other.added.len(), 2, "the same events on another entity are not duplicates");
    doc.remove_ids(&r.added);
    doc.remove_ids(&other.added);
    assert_eq!(doc.events.len(), 0);
    assert_eq!(doc.next_id, 5, "ids are never reissued");
    assert_ne!(doc.to_json(), before, "next_id moved: the id counter is part of the saved state");
}

/// Protects the saved slot: round trip equal, and a malformed row costs exactly that row.
#[test]
fn events_doc_round_trip_and_tolerance() {
    let c = of("```chronos\n* [10] a\n* [20] b\n```\n", Form::Chronos).remove(0);
    let mut doc = EventsDoc::default();
    doc.import(&c, "v", "K.md", "faction", 2, 0, false);
    let (back, dropped) = EventsDoc::from_json(&doc.to_json()).unwrap();
    assert_eq!((back, dropped), (doc.clone(), 0));
    let mut v: serde_json::Value = serde_json::from_str(&doc.to_json()).unwrap();
    v["events"][0]["start"] = serde_json::json!("not a number");
    let (got, dropped) = EventsDoc::from_json(&v.to_string()).unwrap();
    assert_eq!((got.events.len(), dropped), (1, 1));
    assert_eq!(got.next_id, 3, "the dropped row's id is still never reissued");
    assert_eq!(EventsDoc::from_json("").unwrap().0, EventsDoc::default());
}

/// Protects re-sync (design §4.5): a new line is `added`, a vanished one `removed`, a line with the
/// same name and new dates `changed`; applying marks `source_removed` and never deletes.
#[test]
fn resync_added_removed_changed() {
    let c1 = of("```chronos\n* [10] Founded\n* [20] Fire\n* [30] Peace\n```\n", Form::Chronos).remove(0);
    let mut doc = EventsDoc::default();
    doc.import(&c1, "v", "K.md", "settlement", 1, 5, false);
    let same = doc.resync_plan(&c1, "v", "K.md", "settlement", 1);
    assert_eq!(same, ResyncPlan::default(), "an unchanged note plans nothing");
    let c2 = of("```chronos\n* [10] Founded\n* [25] Fire\n* [40] Flood\n```\n", Form::Chronos).remove(0);
    let plan = doc.resync_plan(&c2, "v", "K.md", "settlement", 1);
    assert_eq!((plan.added.len(), plan.removed.len(), plan.changed.len()), (1, 1, 1));
    assert_eq!(plan.added[0].start, 45, "added events carry the import's offset");
    assert_eq!(plan.changed[0].1.start, 30);
    doc.apply_resync(&plan, true, true, true, "v", "K.md", "settlement", 1, false);
    assert_eq!(doc.events.len(), 4, "nothing is deleted: 3 + 1 added");
    assert_eq!(doc.events.iter().filter(|e| e.source_removed).count(), 1);
    assert_eq!(doc.events.iter().find(|e| e.title == "Fire").unwrap().start, 30);
    // after applying, the plan against the same note is only the already-flagged removal, which is excluded
    let after = doc.resync_plan(&c2, "v", "K.md", "settlement", 1);
    assert_eq!(after, ResyncPlan::default());
}

/// Protects the founding comparison: equal agrees, different is a conflict carrying both, a
/// non-founding event or a `None` founded is not applicable (never year 0).
#[test]
fn founding_check_cases() {
    let ev = |n: &str, y: i64| Event { kind: Kind::Point, start: y, end: None, start_md: None, end_md: None, color: None, group: None, name: n.into(), description: None };
    assert_eq!(founding_check(Some(1204), &ev("Founding of Kelv", 1204)), FoundingCheck::Agrees);
    assert_eq!(founding_check(Some(1210), &ev("Kelv founded", 1204)), FoundingCheck::Conflict { note: 1204, founded: 1210 });
    assert_eq!(founding_check(Some(1210), &ev("Coronation", 1204)), FoundingCheck::NotApplicable);
    assert_eq!(founding_check(None, &ev("Founded", 0)), FoundingCheck::NotApplicable);
}

/// Protects line identity: it ignores the description and colour, and moves with kind, date or name.
#[test]
fn line_key_identity() {
    let a = chronos::parse_line("* [100] Siege | one").unwrap();
    let b = chronos::parse_line("* [100] #red Siege | two").unwrap();
    assert_eq!(line_key(&a), line_key(&b));
    assert_ne!(line_key(&a), line_key(&chronos::parse_line("* [101] Siege").unwrap()));
    assert_ne!(line_key(&a), line_key(&chronos::parse_line("= [100] Siege").unwrap()));
    assert_ne!(line_key(&a), line_key(&chronos::parse_line("* [100] Siege of").unwrap()));
}

/// CT-0's real-corpus pass. Run by the owner (or a later session) with the vault's folder in
/// `CARTALITH_CT0_VAULT`; it reads Markdown only, writes nothing, opens no socket, and prints
/// per-form counts for the CT-0 report. `#[ignore]`d: it needs a vault.
#[test]
#[ignore]
fn ct0_corpus_pass_over_a_real_vault() {
    let root = std::env::var("CARTALITH_CT0_VAULT").expect("set CARTALITH_CT0_VAULT");
    let v = crate::provider::FsVault::new(root);
    let mut n = [0usize; 4];
    let mut notes = 0;
    for rel in v.list_markdown(100_000).unwrap() {
        let Ok(t) = v.read(&rel) else { continue };
        notes += 1;
        for c in detect(&t) {
            n[match c.form { Form::Chronos => 0, Form::Frontmatter => 1, Form::YearList => 2, Form::Table => 3 }] += 1;
            println!("{rel}\t{}\t{}\t{:?}", c.form.as_str(), c.events.len(), c.year_range());
        }
    }
    println!("notes {notes}: F1 {} F2 {} F3 {} F4 {}", n[0], n[1], n[2], n[3]);
}

/// Protects the read budget: with a budget of 2 over 3 notes only 2 are decided and the third is in
/// neither list; an unreadable note is undecided, never "no timeline"; a note in a private folder
/// carries the marker; a later call with the leftovers continues.
#[test]
fn scan_is_budgeted_and_marks_private() {
    let t = "```chronos
* [1] a
```
";
    let paths: Vec<String> = ["a.md", "Private/b.md", "c.md"].iter().map(|s| s.to_string()).collect();
    let r = scan(&paths, 2, |_| Some(t.to_string()));
    assert_eq!(r.decided, vec!["a.md".to_string(), "Private/b.md".to_string()]);
    assert_eq!(r.found.len(), 2);
    assert!(r.found[0].private.is_none());
    assert!(r.found[1].private.is_some());
    let rest = scan(&paths[2..], 2, |_| Some(t.to_string()));
    assert_eq!(rest.decided, vec!["c.md".to_string()]);
    let unreadable = scan(&paths, 3, |p| if p == "a.md" { None } else { Some(t.to_string()) });
    assert!(!unreadable.decided.contains(&"a.md".to_string()));
}

/// Protects the save: a store holding ONLY a decision is not empty (the `links.is_empty()` gate
/// defect, `MISTAKES.md`), the decision survives the document, and an older document -- one with
/// no `timelines` member, as every prior writer produced -- opens and re-serialises byte-identically
/// with no member added.
#[test]
fn link_store_carries_decisions_and_old_documents_are_unchanged() {
    use crate::links::LinkStore;
    let mut st = LinkStore::default();
    st.add_vault("Elaris");
    let old = st.to_json();
    assert!(!old.contains("timelines"), "no decision, no member");
    assert_eq!(LinkStore::from_json(&old).unwrap().to_json(), old, "an old document round-trips byte-identically");
    let mut only = LinkStore::default();
    assert!(only.is_empty());
    record(&mut only.timelines, TimelineDecision {
        vault_id: "v".into(), relative_path: "K.md".into(), form: Form::Table, ordinal: 0,
        block_hash: "h".into(), decision: Decision::Ignore, entity_kind: None, entity_id: None, epoch_offset: None, decided: 1,
    });
    assert!(!only.is_empty(), "a store holding only a decision must be written");
    let back = LinkStore::from_json(&only.to_json()).unwrap();
    assert_eq!(back.timelines, only.timelines);
}
