//! The consolidated timeline's read model (`CHRONOS_TIMELINE_DESIGN.md` §2,
//! CT-1, CT-8; Ruling GE option (a)).
//!
//! **There is no per-item timeline store.** "Per-item timeline data" is a join
//! over what already exists (founding years, recorded snapshots, ownership
//! spans, conflicts, journeys, note events, imported events), so this is a
//! pure function that builds one sorted list from caller-supplied inputs. It
//! owns no state, writes nothing, never touches the year cursor
//! (`CivData::year` keeps its single writer) and moves no golden or save.
//!
//! The inputs are plain structs rather than the engine's own types where the
//! engine's type would drag a whole subsystem in (a `Journey`'s plan, a
//! settlement's roster row): the bridge fills them from the real sources.
//! This crate depends on `cartalith-vault` only for the Chronos date types
//! (`MonthDay`) and `EntityKind`, which the design's `TimelineItem` names.
//!
//! **Never encode "no value" as a plausible value.** A settlement whose
//! `founded` is `None` emits **no** item (not year 0); an open-ended conflict
//! sets [`TimelineItem::open_end`] rather than a magic end year, and a point
//! and an open range are different (`end: None` alone cannot tell them apart).

use crate::conflict::{Conflict, ConflictAnchor};
use crate::timeline::OwnershipSpan;
use cartalith_vault::chronos::{self, Event, Kind, MonthDay};
use cartalith_vault::links::EntityKind;
use cartalith_vault::timelines::ImportedEvent;

/// Where an item came from. Also the **Kind** filter's axis (design §2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemSource {
    Founding,
    Snapshot,
    Ownership,
    Conflict,
    Journey,
    NoteEvent,
    Imported,
}

impl ItemSource {
    /// Every source, in the order the strip lists its lanes.
    pub const ALL: [ItemSource; 7] = [
        ItemSource::Founding,
        ItemSource::Snapshot,
        ItemSource::Ownership,
        ItemSource::Conflict,
        ItemSource::Journey,
        ItemSource::NoteEvent,
        ItemSource::Imported,
    ];

    /// The lane's word, and the Chronos `{group}` an export writes.
    pub fn label(self) -> &'static str {
        match self {
            ItemSource::Founding => "Founding",
            ItemSource::Snapshot => "Snapshot",
            ItemSource::Ownership => "Ownership",
            ItemSource::Conflict => "Conflict",
            ItemSource::Journey => "Journey",
            ItemSource::NoteEvent => "Note",
            ItemSource::Imported => "Imported",
        }
    }

    /// The wire word the shell passes back for a Kind filter.
    pub fn parse(s: &str) -> Option<ItemSource> {
        ItemSource::ALL.into_iter().find(|k| k.label().eq_ignore_ascii_case(s))
    }
}

/// How finely an item's date is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    Year,
    Date,
    Range,
}

/// A date: the year and, when known, the month/day in the 365-day calendar.
pub type When = (i64, Option<MonthDay>);

/// Where a note-derived item came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    pub vault_id: String,
    pub relative_path: String,
    pub line_key: String,
}

/// One row of the consolidated timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineItem {
    /// Stable: `founded:settlement:42`, `conflict:7`, `journey:3`, `span:settlement:42:120`,
    /// `snapshot:300`, `note:<link_id>:<line_key>`, `imported:<id>`.
    pub key: String,
    pub source: ItemSource,
    pub start: When,
    pub end: Option<When>,
    /// True for a range still open on the right. Distinct from a point (`end: None, open_end: false`).
    pub open_end: bool,
    pub title: String,
    pub entity: Option<(EntityKind, i64)>,
    pub factions: Vec<i32>,
    pub precision: Precision,
    pub origin: Option<Origin>,
    /// From a private note (design §4.5): hidden from the assistant and from exports.
    pub private: bool,
}

/// A settlement as the founding lane needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundingInput {
    pub id: i64,
    pub name: String,
    /// `None` = unknown: emits nothing.
    pub founded: Option<i64>,
    pub faction: i32,
}

/// One settlement's ownership spans (from [`crate::timeline::civ_settlement_ownership_periods`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipInput {
    pub tid: i64,
    pub name: String,
    pub spans: Vec<OwnershipSpan>,
}

/// A journey as the strip needs it; the length is `JourneyTimeline.calendar_days`, derived from the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JourneyInput {
    pub id: u64,
    pub name: String,
    pub start_year: i64,
    /// 0-based day of `start_year`, or `None` when never set (never `Some(0)`).
    pub start_day: Option<u16>,
    /// `None` when no plan gives a length: the journey is then a point.
    pub calendar_days: Option<u32>,
}

/// The events of one linked note, read live by `VaultSession::entity_chronos`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteEventsInput {
    pub link_id: String,
    pub vault_id: String,
    pub relative_path: String,
    pub entity: Option<(EntityKind, i64)>,
    pub factions: Vec<i32>,
    pub private: bool,
    pub events: Vec<Event>,
}

/// Everything the model is built from.
#[derive(Debug, Clone, Default)]
pub struct ItemSources {
    pub founding: Vec<FoundingInput>,
    pub snapshot_years: Vec<i64>,
    pub ownership: Vec<OwnershipInput>,
    pub conflicts: Vec<Conflict>,
    pub journeys: Vec<JourneyInput>,
    pub note_events: Vec<NoteEventsInput>,
    pub imported: Vec<ImportedEvent>,
}

fn precision(start: When, end: Option<When>, open_end: bool) -> Precision {
    if end.is_some() || open_end {
        Precision::Range
    } else if start.1.is_some() {
        Precision::Date
    } else {
        Precision::Year
    }
}

fn item(key: String, source: ItemSource, start: When, end: Option<When>, open_end: bool, title: String) -> TimelineItem {
    TimelineItem {
        key,
        source,
        start,
        end,
        open_end,
        title,
        entity: None,
        factions: Vec::new(),
        precision: precision(start, end, open_end),
        origin: None,
        private: false,
    }
}

fn event_item(key: String, source: ItemSource, e: &Event, title: String) -> TimelineItem {
    item(key, source, (e.start, e.start_md), e.end.map(|y| (y, e.end_md)), false, title)
}

/// Builds the consolidated list, sorted by start date then key (a total order, so a rebuild is
/// reproducible). Pure; see the module doc.
pub fn build(src: &ItemSources) -> Vec<TimelineItem> {
    let mut out: Vec<TimelineItem> = Vec::new();

    for s in &src.founding {
        let Some(y) = s.founded else { continue };
        let mut it = item(format!("founded:settlement:{}", s.id), ItemSource::Founding, (y, None), None, false, format!("{} founded", s.name));
        it.entity = Some((EntityKind::Settlement, s.id));
        it.factions = vec![s.faction];
        out.push(it);
    }
    for y in &src.snapshot_years {
        out.push(item(format!("snapshot:{y}"), ItemSource::Snapshot, (*y, None), None, false, "Recorded year".to_string()));
    }
    for o in &src.ownership {
        for sp in &o.spans {
            let mut it = item(
                format!("span:settlement:{}:{}", o.tid, sp.start_year),
                ItemSource::Ownership,
                (sp.start_year, None),
                sp.end_year.map(|e| (e, None)),
                sp.end_year.is_none(),
                format!("{} under faction {}", o.name, sp.faction_id),
            );
            it.entity = Some((EntityKind::Settlement, o.tid));
            it.factions = vec![sp.faction_id];
            out.push(it);
        }
    }
    for c in &src.conflicts {
        let mut it = item(
            format!("conflict:{}", c.id),
            ItemSource::Conflict,
            (c.start_year, None),
            c.end_year.map(|e| (e, None)),
            c.end_year.is_none(),
            c.name.clone(),
        );
        it.entity = c.anchor.map(|a| match a {
            ConflictAnchor::Settlement(t) => (EntityKind::Settlement, t as i64),
            ConflictAnchor::Province(t) => (EntityKind::Province, t as i64),
        });
        it.factions = c.sides.clone();
        out.push(it);
    }
    for j in &src.journeys {
        let start_md = j.start_day.and_then(|d| MonthDay::from_day_of_year(i64::from(d)));
        let start = (j.start_year, start_md);
        let end = j.calendar_days.map(|n| {
            let (y, md) = chronos::date_of_day_number(chronos::day_number(j.start_year, start_md) + i64::from(n));
            // a journey with no departure day is dated by year; its end is then a year too
            (y, if start_md.is_some() { Some(md) } else { None })
        });
        out.push(item(format!("journey:{}", j.id), ItemSource::Journey, start, end, false, j.name.clone()));
    }
    for n in &src.note_events {
        for e in &n.events {
            let key = cartalith_vault::timelines::line_key(e);
            let mut it = event_item(format!("note:{}:{key}", n.link_id), ItemSource::NoteEvent, e, e.name.clone());
            it.entity = n.entity;
            it.factions = n.factions.clone();
            it.private = n.private;
            it.origin = Some(Origin { vault_id: n.vault_id.clone(), relative_path: n.relative_path.clone(), line_key: key });
            out.push(it);
        }
    }
    for r in &src.imported {
        let e = r.as_event();
        let title = if r.source_removed { format!("{} (source removed)", r.title) } else { r.title.clone() };
        let mut it = event_item(format!("imported:{}", r.id), ItemSource::Imported, &e, title);
        it.entity = EntityKind::parse(&r.entity_kind).map(|k| (k, r.entity_id));
        it.private = r.private;
        it.origin = Some(Origin { vault_id: r.vault_id.clone(), relative_path: r.relative_path.clone(), line_key: r.line_key.clone() });
        out.push(it);
    }
    out.sort_by(|a, b| {
        chronos::day_number(a.start.0, a.start.1).cmp(&chronos::day_number(b.start.0, b.start.1)).then_with(|| a.key.cmp(&b.key))
    });
    out
}

/// The three filters (design §2.2). An empty list means "no restriction on that axis"; groups are
/// ANDed and values inside one group ORed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    pub entities: Vec<(EntityKind, i64)>,
    pub kinds: Vec<ItemSource>,
    pub factions: Vec<i32>,
}

/// What a filter leaves, and how many it hid ("N hidden by filters": an empty strip is never read
/// as "no data").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filtered<'a> {
    pub visible: Vec<&'a TimelineItem>,
    pub hidden: usize,
}

/// Applies `f` to `items`.
pub fn apply<'a>(items: &'a [TimelineItem], f: &Filter) -> Filtered<'a> {
    let keep = |i: &TimelineItem| {
        (f.entities.is_empty() || i.entity.is_some_and(|e| f.entities.contains(&e)))
            && (f.kinds.is_empty() || f.kinds.contains(&i.source))
            && (f.factions.is_empty() || i.factions.iter().any(|x| f.factions.contains(x)))
    };
    let visible: Vec<&TimelineItem> = items.iter().filter(|i| keep(i)).collect();
    let hidden = items.len() - visible.len();
    Filtered { visible, hidden }
}

/// The items an assistant tool or an export may see: private ones are dropped (Ruling DP's rule).
pub fn without_private(items: &[TimelineItem]) -> Vec<&TimelineItem> {
    items.iter().filter(|i| !i.private).collect()
}

/// `(min, max)` over every year any item reaches -- what `tl_range` widens its track to, so a
/// founding at year -340 is reachable. An open end contributes its start only.
pub fn year_extent(items: &[TimelineItem]) -> Option<(i64, i64)> {
    let mut it = items.iter().flat_map(|i| std::iter::once(i.start.0).chain(i.end.map(|e| e.0)));
    let first = it.next()?;
    Some(it.fold((first, first), |(lo, hi), y| (lo.min(y), hi.max(y))))
}

/// The header an export starts with (design §3: a generated, read-only snapshot).
pub const EXPORT_HEADER: &str = "# generated by Cartalith; edit freely, it is not read back";

/// A pasteable Chronos block and what it could not carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChronosExport {
    /// A fenced `chronos` block.
    pub text: String,
    /// How many lines were written.
    pub written: usize,
    /// Titles that could not be written so they read back (never silently dropped).
    pub skipped: Vec<String>,
}

/// An item as the Chronos event an export writes. An open range has no Chronos form (Chronos has no
/// open end), so it is written as a point at its start; the group is the lane's label.
pub fn to_event(i: &TimelineItem) -> Event {
    let name: String = i
        .title
        .replace(['|', '\n', '\r'], " ")
        .trim()
        .trim_start_matches(['#', '{'])
        .trim()
        .to_string();
    Event {
        kind: if i.end.is_some() { Kind::Event } else { Kind::Point },
        start: i.start.0,
        end: i.end.map(|e| e.0),
        start_md: i.start.1,
        end_md: i.end.and_then(|e| e.1),
        color: None,
        group: Some(i.source.label().to_string()),
        name,
        description: None,
    }
}

/// Exports `items` as one Chronos block. **Private items are never written**: an export leaves them
/// out until the user says otherwise (design §4.5); pass an already-filtered view.
pub fn export_chronos(items: &[&TimelineItem]) -> ChronosExport {
    let mut text = format!("```chronos\n{EXPORT_HEADER}\n");
    let (mut written, mut skipped) = (0, Vec::new());
    for i in items.iter().filter(|i| !i.private) {
        match chronos::to_line(&to_event(i)) {
            Ok(l) => {
                text.push_str(&l);
                text.push('\n');
                written += 1;
            }
            Err(_) => skipped.push(i.title.clone()),
        }
    }
    text.push_str("```\n");
    ChronosExport { text, written, skipped }
}

#[cfg(test)]
mod tests;
