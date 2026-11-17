//! Recognising timelines in a vault's notes, remembering what the user chose,
//! and the events imported onto the map (`CHRONOS_TIMELINE_DESIGN.md` §4, §5;
//! `LARGE_ITEM_RULINGS.md` Ruling GD, answered by Ruling GE).
//!
//! # What this module is for, and what it must never do
//!
//! A *timeline* is a set of dated events found in one note, in one of four
//! **forms** ([`Form`]). Detection is **read-only over note text**: nothing in
//! here takes a writer, and an import never edits a note (design §4.5). The
//! one note write in this whole feature stays `VaultSession::add_chronos_event`.
//!
//! | Form | Rule | Confidence |
//! |---|---|---|
//! | F1 [`Form::Chronos`] | a fenced `chronos` block, read by [`crate::chronos::parse`] | certain |
//! | F2 [`Form::Frontmatter`] | a **closed key list** ([`FRONTMATTER_KEYS`]) with a year or `YYYY-MM[-DD]` | high |
//! | F3 [`Form::YearList`] | a bullet list of year-led items under a timeline-named heading, or an ascending run | medium |
//! | F4 [`Form::Table`] | a table with a `Year`/`Date` column and at least three year rows | medium |
//!
//! `date`, `created`, `modified` and `updated` are **deliberately not** in the
//! F2 key list: they are real-world note metadata and would flood the strip
//! with the current year. F5 (the Obsidian *Timelines* plugin's HTML spans) is
//! **not built**: its syntax was recalled, not read, and the design defers it
//! (CT-9).
//!
//! # Why F3 and F4 are gated, and what gated them
//!
//! The design (CT-0) ships F3 and F4 only when precision on a hand-labelled
//! sample is at least 0.90. [`SHIP_YEAR_LIST`]/[`SHIP_TABLE`] are those gates;
//! the labelled sample and its literal precision/recall assertions are the
//! tests at the bottom of this file. **The sample is synthetic**: the owner's
//! own vault was out of bounds for the session that built this, so the real
//! corpus pass is the `#[ignore]`d test `ct0_corpus_pass_over_a_real_vault`,
//! run by the owner (or a later session) with `CARTALITH_CT0_VAULT` set.
//!
//! # What is saved
//!
//! Only two things, both additive and both optional (`SAVEFILE_COMPAT.md`
//! §13.3 `timelines[]`, §10.3 `history/events.json`): the decisions
//! ([`TimelineDecision`], in the link store) and the imported events
//! ([`EventsDoc`]). The candidate list itself is recomputed, never saved.

use crate::chronos::{self, Event, Kind, MonthDay};
use crate::provider::content_hash;
use serde::{Deserialize, Serialize};

/// The CT-0 gate for F3 (design §7): shipped because the synthetic labelled
/// sample's precision is asserted at or above 0.90 by
/// `ct0_year_list_precision_meets_the_gate`. Flip to `false` to switch the form
/// off everywhere without touching its code.
pub const SHIP_YEAR_LIST: bool = true;
/// The CT-0 gate for F4, same arrangement as [`SHIP_YEAR_LIST`].
pub const SHIP_TABLE: bool = true;

/// The only frontmatter keys read as dates (design §4.1 F2). A closed list,
/// compared case-insensitively. `founded` is the one with a destination
/// (`PlaceExtras::founded`).
pub const FRONTMATTER_KEYS: &[&str] =
    &["founded", "founding_year", "established", "born", "died", "start_year", "end_year", "year"];

/// Heading words that mark a list as a timeline (design §4.1 F3), matched as
/// whole lowercase words so `History of Kelv` counts and `Historical` does not.
pub const TIMELINE_HEADINGS: &[&str] =
    &["timeline", "timelines", "history", "chronology", "events", "annals", "dates"];

/// A year list with no timeline heading must still have this many items, in
/// strictly ascending year order, to count (design §4.1 F3).
pub const ASCENDING_RUN_MIN: usize = 4;
/// A year list under a timeline heading needs at least this many items.
pub const HEADED_RUN_MIN: usize = 2;
/// A table needs at least this many year rows (design §4.1 F4).
pub const TABLE_ROWS_MIN: usize = 3;
/// Years of full dates in a table that read as real-world metadata, not worldbuilding (F4's guard).
const REAL_WORLD_YEARS: std::ops::RangeInclusive<i64> = 1900..=2100;
/// A year token has at most this many digits: `- 2024` is a year, `- 123456` is not.
const MAX_YEAR_DIGITS: usize = 5;

/// Which of the four recognised shapes a timeline has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    Chronos,
    Frontmatter,
    YearList,
    Table,
}

impl Form {
    /// The stable word written into `vault.json` and shown to the user.
    pub fn as_str(self) -> &'static str {
        match self {
            Form::Chronos => "chronos",
            Form::Frontmatter => "frontmatter",
            Form::YearList => "year_list",
            Form::Table => "table",
        }
    }

    /// Why this was found, in words (design §4.1: "everything recognised is
    /// labelled with its form, so the user sees why it was found").
    pub fn label(self) -> &'static str {
        match self {
            Form::Chronos => "Chronos block",
            Form::Frontmatter => "Dates in properties",
            Form::YearList => "Year list",
            Form::Table => "Dated table",
        }
    }
}

/// One timeline recognised in one note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub form: Form,
    /// Which one of this form in the note (0-based, in document order).
    pub ordinal: usize,
    /// The events, in the order written.
    pub events: Vec<Event>,
    /// F1 only: block lines that were not valid Chronos, listed not dropped.
    pub skipped: Vec<String>,
    /// [`content_hash`] of the block's own text, so a changed block is noticed.
    pub block_hash: String,
    /// The heading the timeline sits under, when it has one.
    pub heading: Option<String>,
}

impl Candidate {
    /// `(min, max)` over every year the events mention; `None` for no events.
    pub fn year_range(&self) -> Option<(i64, i64)> {
        let mut it = self.events.iter().flat_map(|e| std::iter::once(e.start).chain(e.end));
        let first = it.next()?;
        Some(it.fold((first, first), |(lo, hi), y| (lo.min(y), hi.max(y))))
    }

    /// Up to three `"<date> <name>"` preview lines (design §4.3).
    pub fn preview(&self) -> Vec<String> {
        self.events
            .iter()
            .take(3)
            .map(|e| format!("{} {}", chronos::format_date(e.start, e.start_md), e.name))
            .collect()
    }

    /// The `founded` year this candidate states, for F2 only.
    pub fn founded_year(&self) -> Option<i64> {
        if self.form != Form::Frontmatter {
            return None;
        }
        self.events.iter().find(|e| e.name == "founded" || e.name == "founding_year").map(|e| e.start)
    }
}

/// Every timeline in `text`, with the shipped forms only. See [`detect_forms`].
pub fn detect(text: &str) -> Vec<Candidate> {
    let mut forms = vec![Form::Chronos, Form::Frontmatter];
    if SHIP_YEAR_LIST {
        forms.push(Form::YearList);
    }
    if SHIP_TABLE {
        forms.push(Form::Table);
    }
    detect_forms(text, &forms)
}

/// Every timeline of the given `forms` in `text`, F1 then F2 then F3 then F4
/// within document order per form. Pure; reads nothing but `text`.
pub fn detect_forms(text: &str, forms: &[Form]) -> Vec<Candidate> {
    let lines: Vec<&str> = text.lines().collect();
    let fm_end = frontmatter_end(&lines);
    let mut out: Vec<Candidate> = Vec::new();

    if forms.contains(&Form::Frontmatter) {
        if let Some(c) = detect_frontmatter(text) {
            out.push(c);
        }
    }

    // One pass over the body: fences (chronos blocks and code to skip), headings, bullets, tables.
    let mut fence: Option<(usize, bool, usize)> = None; // (ticks, is_chronos, body_start_line)
    let mut heading: Option<String> = None;
    let mut run: Vec<(usize, Event)> = Vec::new();
    let mut run_heading: Option<String> = None;
    let mut table: Vec<&str> = Vec::new();
    let mut table_heading: Option<String> = None;
    let mut ord = [0usize; 4];

    macro_rules! flush_run {
        () => {
            if forms.contains(&Form::YearList) {
                if let Some(c) = year_list_candidate(&run, &run_heading, ord[2]) {
                    ord[2] += 1;
                    out.push(c);
                }
            }
            run.clear();
        };
    }
    macro_rules! flush_table {
        () => {
            if forms.contains(&Form::Table) {
                if let Some(c) = table_candidate(&table, &table_heading, ord[3]) {
                    ord[3] += 1;
                    out.push(c);
                }
            }
            table.clear();
        };
    }

    for (i, raw) in lines.iter().enumerate().skip(fm_end) {
        let line = raw.trim();
        if let Some((ticks, is_chronos, start)) = fence {
            if line.len() >= ticks && line.bytes().all(|b| b == b'`') {
                if is_chronos && forms.contains(&Form::Chronos) {
                    let body = lines[start..i].join("\n");
                    let parsed = chronos::parse(&format!("```chronos\n{body}\n```"));
                    out.push(Candidate {
                        form: Form::Chronos,
                        ordinal: ord[0],
                        events: parsed.events,
                        skipped: parsed.skipped,
                        block_hash: content_hash(&body),
                        heading: heading.clone(),
                    });
                    ord[0] += 1;
                }
                fence = None;
            }
            continue;
        }
        let ticks = line.len() - line.trim_start_matches('`').len();
        if ticks >= 3 {
            flush_run!();
            flush_table!();
            fence = Some((ticks, line[ticks..].trim() == "chronos", i + 1));
            continue;
        }
        if let Some(h) = heading_text(line) {
            flush_run!();
            flush_table!();
            heading = Some(h);
            continue;
        }
        if line.starts_with('|') {
            flush_run!();
            if table.is_empty() {
                table_heading = heading.clone();
            }
            table.push(line);
            continue;
        }
        flush_table!();
        if line.is_empty() {
            continue; // a blank line keeps a loose list open
        }
        match bullet_event(line) {
            Some(ev) => {
                if run.is_empty() {
                    run_heading = heading.clone();
                }
                run.push((i, ev));
            }
            None => {
                flush_run!();
            }
        }
    }
    flush_run!();
    flush_table!();
    out
}

/// The first body line (0-based) after a leading `---` frontmatter block, or 0.
fn frontmatter_end(lines: &[&str]) -> usize {
    if lines.first().map(|l| l.trim_end()) != Some("---") {
        return 0;
    }
    for (i, l) in lines.iter().enumerate().skip(1) {
        let t = l.trim_end();
        if t == "---" || t == "..." {
            return i + 1;
        }
    }
    0
}

/// `# Heading` text, trimmed, for any ATX level.
fn heading_text(line: &str) -> Option<String> {
    let hashes = line.len() - line.trim_start_matches('#').len();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = line[hashes..].strip_prefix(' ')?;
    Some(rest.trim().trim_end_matches('#').trim().to_string())
}

fn heading_is_timeline(h: &Option<String>) -> bool {
    let Some(h) = h else { return false };
    h.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| TIMELINE_HEADINGS.contains(&w))
}

/// F2: the closed key list over the note's properties.
fn detect_frontmatter(text: &str) -> Option<Candidate> {
    let (props, _) = crate::frontmatter::properties(text)?;
    let mut events = Vec::new();
    let mut hash_src = String::new();
    for p in props {
        let key = p.key.to_lowercase();
        if p.list || !FRONTMATTER_KEYS.contains(&key.as_str()) {
            continue;
        }
        let Some(v) = p.values.first() else { continue };
        let Some((y, md, rest)) = year_token(v.trim()) else { continue };
        if !rest.trim().is_empty() {
            continue;
        }
        hash_src.push_str(&format!("{key}={v}\n"));
        events.push(Event { kind: Kind::Point, start: y, end: None, start_md: md, end_md: None, color: None, group: None, name: key, description: None });
    }
    if events.is_empty() {
        return None;
    }
    Some(Candidate { form: Form::Frontmatter, ordinal: 0, events, skipped: Vec::new(), block_hash: content_hash(&hash_src), heading: None })
}

/// A signed year with an optional `-MM[-DD]`, and the unparsed rest of `s`.
/// The year is 1 to [`MAX_YEAR_DIGITS`] digits and may not run straight into a
/// letter (`12x`). A two-digit group after `-` is a month, as Chronos reads it.
fn year_token(s: &str) -> Option<(i64, Option<MonthDay>, &str)> {
    let b = s.as_bytes();
    let neg = b.first() == Some(&b'-');
    let st = usize::from(neg);
    let mut i = st;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let nd = i - st;
    if nd == 0 || nd > MAX_YEAR_DIGITS {
        return None;
    }
    if b.get(i).is_some_and(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let y: i64 = s[st..i].parse().ok()?;
    let year = if neg { -y } else { y };
    // optional -MM or -MM-DD, each group exactly two digits and not followed by a digit
    let two = |from: usize| -> Option<u8> {
        if b.get(from) == Some(&b'-')
            && b.get(from + 1).is_some_and(u8::is_ascii_digit)
            && b.get(from + 2).is_some_and(u8::is_ascii_digit)
            && !b.get(from + 3).is_some_and(u8::is_ascii_digit)
        {
            s[from + 1..from + 3].parse().ok()
        } else {
            None
        }
    };
    if let Some(m) = two(i) {
        if let Some(d) = two(i + 3) {
            if let Some(md) = MonthDay::new(m, Some(d)) {
                return Some((year, Some(md), &s[i + 6..]));
            }
        }
        if let Some(md) = MonthDay::new(m, None) {
            return Some((year, Some(md), &s[i + 3..]));
        }
    }
    Some((year, None, &s[i..]))
}

/// `[a]` or `[a-b]` / `a~b` / `a - b` over `s`, returning the event dates and the rest.
fn when(s: &str) -> Option<(i64, Option<MonthDay>, Option<i64>, Option<MonthDay>, &str)> {
    let (y, md, rest) = year_token(s)?;
    let r = rest.trim_start();
    for sep in ['~', '–', '—', '-'] {
        if let Some(after) = r.strip_prefix(sep) {
            let after = after.trim_start();
            // a range end is a year token; a bare `-` followed by a word is a separator, not a range
            if let Some((y2, md2, rest2)) = year_token(after) {
                if y2 >= y || sep == '~' {
                    return Some((y, md, Some(y2), md2, rest2));
                }
            }
        }
    }
    Some((y, md, None, None, rest))
}

fn make_event(y: i64, md: Option<MonthDay>, e: Option<i64>, emd: Option<MonthDay>, name: &str, desc: Option<String>) -> Event {
    Event {
        kind: if e.is_some() { Kind::Event } else { Kind::Point },
        start: y,
        end: e,
        start_md: md,
        end_md: if e.is_some() { emd } else { None },
        color: None,
        group: None,
        name: name.to_string(),
        description: desc,
    }
}

/// F3: one bullet line as a year-led event, or `None`. The year must be
/// followed by a separator (`:`, a dash) or be wrapped in bold: `- 3 cats` is
/// not an event, `- **1204** Coronation` and `- 1204: Coronation` are.
fn bullet_event(line: &str) -> Option<Event> {
    let rest = line.strip_prefix(['-', '*', '+'])?;
    let rest = rest.strip_prefix([' ', '\t'])?.trim_start();
    let (bold, rest) = match rest.strip_prefix("**") {
        Some(r) => (true, r),
        None => (false, rest),
    };
    let (y, md, e, emd, rest) = when(rest)?;
    let mut rest = rest.trim_start();
    // a year range (`1215-1220 Siege`) is its own separator
    let mut sep = e.is_some();
    if bold {
        rest = rest.strip_prefix("**")?.trim_start();
        sep = true;
    }
    if let Some(r) = rest.strip_prefix([':', '–', '—', '-']) {
        rest = r;
        sep = true;
    }
    if !sep {
        return None;
    }
    let name = rest.trim().trim_start_matches("**").trim();
    if name.is_empty() {
        return None;
    }
    Some(make_event(y, md, e, emd, name, None))
}

fn year_list_candidate(run: &[(usize, Event)], heading: &Option<String>, ordinal: usize) -> Option<Candidate> {
    let n = run.len();
    let headed = heading_is_timeline(heading) && n >= HEADED_RUN_MIN;
    let ascending = n >= ASCENDING_RUN_MIN && run.windows(2).all(|w| w[0].1.start < w[1].1.start);
    if !(headed || ascending) {
        return None;
    }
    let events: Vec<Event> = run.iter().map(|(_, e)| e.clone()).collect();
    let src: String = events.iter().map(|e| format!("{}|{}\n", e.start, e.name)).collect();
    Some(Candidate { form: Form::YearList, ordinal, events, skipped: Vec::new(), block_hash: content_hash(&src), heading: heading.clone() })
}

fn cells(line: &str) -> Vec<&str> {
    let t = line.trim().trim_start_matches('|').trim_end_matches('|');
    t.split('|').map(str::trim).collect()
}

fn table_candidate(lines: &[&str], heading: &Option<String>, ordinal: usize) -> Option<Candidate> {
    if lines.len() < 2 + TABLE_ROWS_MIN {
        return None;
    }
    let header = cells(lines[0]);
    let sep = cells(lines[1]);
    if !sep.iter().all(|c| !c.is_empty() && c.bytes().all(|b| b == b'-' || b == b':' || b == b' ')) {
        return None;
    }
    let col = header.iter().position(|h| {
        let l = h.replace("**", "").trim().to_lowercase();
        l == "year" || l == "date"
    })?;
    let mut events = Vec::new();
    for l in &lines[2..] {
        let cs = cells(l);
        let Some(cell) = cs.get(col) else { continue };
        let cell = cell.replace("**", "");
        let Some((y, md, e, emd, rest)) = when(cell.trim()) else { continue };
        if !rest.trim().is_empty() {
            continue;
        }
        let others: Vec<&str> =
            cs.iter().enumerate().filter(|(i, c)| *i != col && !c.is_empty()).map(|(_, c)| *c).collect();
        let Some(name) = others.first() else { continue };
        let desc = if others.len() > 1 { Some(others[1..].join(" | ")) } else { None };
        events.push(make_event(y, md, e, emd, name, desc));
    }
    if events.len() < TABLE_ROWS_MIN {
        return None;
    }
    // A column of full `YYYY-MM-DD` dates all inside the real-world range is a meeting log, not a
    // story's timeline: the same reason F2 excludes `date`/`created` (design §4.1). Found by the CT-0
    // sample, which first measured F4 at 0.857 precision with that table in it.
    let real_world = events.iter().all(|e| e.start_md.is_some_and(|m| m.day.is_some()) && REAL_WORLD_YEARS.contains(&e.start));
    if real_world {
        return None;
    }
    let src: String = events.iter().map(|e| format!("{}|{}\n", e.start, e.name)).collect();
    Some(Candidate { form: Form::Table, ordinal, events, skipped: Vec::new(), block_hash: content_hash(&src), heading: heading.clone() })
}

/// The identity of one event for re-sync and dedup: a hash of `(kind, start,
/// end, name)`, dates written as Chronos writes them (design §4.5).
pub fn line_key(e: &Event) -> String {
    let end = e.end.map(|y| chronos::format_date(y, e.end_md)).unwrap_or_default();
    content_hash(&format!("{}|{}|{}|{}", e.kind.as_str(), chronos::format_date(e.start, e.start_md), end, e.name))
}

// ---------------------------------------------------------------------------------------------
// The scan (design §4.2)
// ---------------------------------------------------------------------------------------------

/// One note's scan answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteScan {
    pub path: String,
    pub candidates: Vec<Candidate>,
    /// `Some` when the note is private (Ruling DP): its candidates still show, flagged, and the
    /// default for them is always Keep.
    pub private: Option<crate::private::PrivateMarker>,
}

/// The answer for a list of notes: what was found, and which paths were **decided**.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scan {
    /// Only notes with at least one candidate.
    pub found: Vec<NoteScan>,
    /// Every path that was actually read (found or not). A path in neither list was **not decided**
    /// (the read budget ran out, or the note could not be read) and must read as "not looked at",
    /// never as "has no timeline" -- the `private_marks` rule.
    pub decided: Vec<String>,
}

/// Scans `paths` for timelines, reading a note **at most `max_reads` times** so a vault of thousands
/// cannot stall the caller; a later call with the leftovers continues. `read` returns `None` for an
/// unreadable note, which is left undecided. Pure apart from `read`; never writes.
pub fn scan(paths: &[String], max_reads: usize, mut read: impl FnMut(&str) -> Option<String>) -> Scan {
    let mut out = Scan::default();
    for (reads, p) in paths.iter().enumerate() {
        if reads >= max_reads {
            break;
        }
        let Some(text) = read(p) else { continue };
        out.decided.push(p.clone());
        let candidates = detect(&text);
        if !candidates.is_empty() {
            out.found.push(NoteScan { path: p.clone(), candidates, private: crate::private::private_marker(p, &text) });
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------
// Decisions (design §4.3, §4.4) -- vault.json `timelines[]`
// ---------------------------------------------------------------------------------------------

/// What the user chose for one timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Keep,
    Ignore,
    Link,
    Import,
}

/// One remembered choice, keyed `(vault_id, relative_path, form, ordinal)`.
/// `block_hash` is what detects a change after the decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineDecision {
    pub vault_id: String,
    pub relative_path: String,
    pub form: Form,
    pub ordinal: usize,
    pub block_hash: String,
    pub decision: Decision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epoch_offset: Option<i64>,
    /// Unix seconds, supplied by the caller (this crate reads no clock).
    pub decided: i64,
}

/// Where a candidate stands against the saved decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// No decision: counts toward the badge.
    Undecided,
    /// Decided, and the block is unchanged: does not count.
    Decided(Decision),
    /// Decided, but the block changed since ("changed since you decided"): counts toward the badge,
    /// with the previous decision shown. An ignored block that changes is a candidate again.
    Changed(Decision),
}

impl Standing {
    /// Whether the passive badge counts this one.
    pub fn needs_attention(self) -> bool {
        !matches!(self, Standing::Decided(_))
    }
}

/// The standing of `c`, found in `rel` of `vault_id`, against `decisions`.
pub fn standing(decisions: &[TimelineDecision], vault_id: &str, rel: &str, c: &Candidate) -> Standing {
    match decisions.iter().find(|d| d.vault_id == vault_id && d.relative_path == rel && d.form == c.form && d.ordinal == c.ordinal) {
        None => Standing::Undecided,
        Some(d) if d.block_hash == c.block_hash => Standing::Decided(d.decision),
        Some(d) => Standing::Changed(d.decision),
    }
}

/// Inserts or replaces the decision with `d`'s key. Replacing is how "ignore"
/// is revisited and how a changed block is re-decided.
pub fn record(decisions: &mut Vec<TimelineDecision>, d: TimelineDecision) {
    match decisions.iter_mut().find(|x| x.vault_id == d.vault_id && x.relative_path == d.relative_path && x.form == d.form && x.ordinal == d.ordinal) {
        Some(x) => *x = d,
        None => decisions.push(d),
    }
}

/// What the passive badge shows: how many of `(path, candidate)` still need attention.
pub fn badge_count(decisions: &[TimelineDecision], vault_id: &str, found: &[(String, Candidate)]) -> usize {
    found.iter().filter(|(p, c)| standing(decisions, vault_id, p, c).needs_attention()).count()
}

// ---------------------------------------------------------------------------------------------
// Imported events (design §5) -- history/events.json
// ---------------------------------------------------------------------------------------------

/// One event copied onto the map. Years are **engine years** (`epoch_offset`
/// already applied); `epoch_offset` is kept so a re-sync applies it again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportedEvent {
    pub id: u64,
    pub entity_kind: String,
    pub entity_id: i64,
    /// Chronos' kind word: `event`, `period`, `point`, `marker`.
    pub kind: String,
    pub start: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_month: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_day: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_month: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_day: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub vault_id: String,
    pub relative_path: String,
    pub line_key: String,
    pub epoch_offset: i64,
    /// Saved flagged; hidden from the assistant and from exports (Ruling DP's rule, design §4.5).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub private: bool,
    /// The source line vanished from the note at a re-sync; the event is kept, not deleted.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub source_removed: bool,
}

fn kind_of(s: &str) -> Kind {
    match s {
        "period" => Kind::Period,
        "point" => Kind::Point,
        "marker" => Kind::Marker,
        _ => Kind::Event,
    }
}

impl ImportedEvent {
    /// The Chronos event this row stands for, in engine years.
    pub fn as_event(&self) -> Event {
        let md = |m: Option<u8>, d: Option<u8>| m.and_then(|m| MonthDay::new(m, d));
        Event {
            kind: kind_of(&self.kind),
            start: self.start,
            end: self.end,
            start_md: md(self.start_month, self.start_day),
            end_md: if self.end.is_some() { md(self.end_month, self.end_day) } else { None },
            color: self.color.clone(),
            group: self.group.clone(),
            name: self.title.clone(),
            description: self.description.clone(),
        }
    }

    /// `(entity, kind, normalised name, start, end)` -- design §4.5's dedup key.
    pub fn dedup_key(&self) -> String {
        dedup_key(&self.entity_kind, self.entity_id, &self.as_event())
    }
}

fn dedup_key(entity_kind: &str, entity_id: i64, e: &Event) -> String {
    let end = e.end.map(|y| chronos::format_date(y, e.end_md)).unwrap_or_default();
    format!(
        "{entity_kind}:{entity_id}|{}|{}|{}|{end}",
        e.kind.as_str(),
        e.name.trim().to_lowercase(),
        chronos::format_date(e.start, e.start_md)
    )
}

/// `history/events.json`: the imported events and the next id to issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventsDoc {
    pub version: u32,
    /// Ids are issued by the writer and never reissued, even after a delete.
    pub next_id: u64,
    pub events: Vec<ImportedEvent>,
}

impl Default for EventsDoc {
    fn default() -> Self {
        EventsDoc { version: 1, next_id: 1, events: Vec::new() }
    }
}

/// The outcome of one import call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportReport {
    /// Ids of the rows added.
    pub added: Vec<u64>,
    /// Events skipped because the same event was already on the timeline.
    pub duplicates: usize,
}

impl EventsDoc {
    /// Whether the slot has anything to write (a writer omits the member when it has not).
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("an EventsDoc always serializes")
    }

    /// Reads the slot, tolerating a malformed row: **a bad row costs that row**
    /// (`SAVEFILE_COMPAT.md` 10.1's tolerance), and the second value is how many
    /// were dropped so the caller can report them. An unreadable document is an
    /// empty one with every row counted as dropped is NOT claimed -- it returns
    /// `(empty, 0)` only for empty input and `Err` otherwise.
    pub fn from_json(s: &str) -> Result<(Self, usize), serde_json::Error> {
        if s.trim().is_empty() {
            return Ok((EventsDoc::default(), 0));
        }
        let v: serde_json::Value = serde_json::from_str(s)?;
        let mut doc = EventsDoc::default();
        if let Some(n) = v.get("next_id").and_then(|n| n.as_u64()) {
            doc.next_id = n.max(1);
        }
        let mut dropped = 0;
        if let Some(rows) = v.get("events").and_then(|r| r.as_array()) {
            for r in rows {
                match serde_json::from_value::<ImportedEvent>(r.clone()) {
                    Ok(e) => {
                        doc.next_id = doc.next_id.max(e.id + 1);
                        doc.events.push(e);
                    }
                    Err(_) => dropped += 1,
                }
            }
        }
        Ok((doc, dropped))
    }

    /// Imports every event of `c` onto `(entity_kind, entity_id)`, shifting years
    /// by `epoch_offset`. An event whose [`dedup_key`] already exists is counted
    /// in `duplicates` and not added. **Never reads or writes a note.**
    pub fn import(
        &mut self,
        c: &Candidate,
        vault_id: &str,
        rel: &str,
        entity_kind: &str,
        entity_id: i64,
        epoch_offset: i64,
        private: bool,
    ) -> ImportReport {
        let mut rep = ImportReport::default();
        for src in &c.events {
            let mut e = src.clone();
            e.start += epoch_offset;
            e.end = e.end.map(|y| y + epoch_offset);
            let key = dedup_key(entity_kind, entity_id, &e);
            if self.events.iter().any(|x| x.dedup_key() == key) {
                rep.duplicates += 1;
                continue;
            }
            let id = self.next_id;
            self.next_id += 1;
            self.events.push(row_of(id, &e, vault_id, rel, entity_kind, entity_id, epoch_offset, private, line_key(src)));
            rep.added.push(id);
        }
        rep
    }

    /// Removes the rows with these ids (the grouped undo of an import).
    pub fn remove_ids(&mut self, ids: &[u64]) {
        self.events.retain(|e| !ids.contains(&e.id));
    }
}

#[allow(clippy::too_many_arguments)]
fn row_of(id: u64, e: &Event, vault_id: &str, rel: &str, entity_kind: &str, entity_id: i64, epoch_offset: i64, private: bool, line_key: String) -> ImportedEvent {
    ImportedEvent {
        id,
        entity_kind: entity_kind.to_string(),
        entity_id,
        kind: e.kind.as_str().to_string(),
        start: e.start,
        start_month: e.start_md.map(|m| m.month),
        start_day: e.start_md.and_then(|m| m.day),
        end: e.end,
        end_month: e.end_md.map(|m| m.month),
        end_day: e.end_md.and_then(|m| m.day),
        color: e.color.clone(),
        group: e.group.clone(),
        title: e.name.clone(),
        description: e.description.clone(),
        vault_id: vault_id.to_string(),
        relative_path: rel.to_string(),
        line_key,
        epoch_offset,
        private,
        source_removed: false,
    }
}

/// What a re-sync of a changed note would do (design §4.5): shown, never applied silently.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResyncPlan {
    /// Lines now in the note that are not on the map.
    pub added: Vec<Event>,
    /// Imported rows whose source line is gone (kept, marked `source_removed`).
    pub removed: Vec<u64>,
    /// `(row id, the new version of its line)`: same kind and name, new dates.
    pub changed: Vec<(u64, Event)>,
}

impl EventsDoc {
    /// Compares the rows imported from `(vault_id, rel)` onto `entity` with the note's current candidate `c`.
    pub fn resync_plan(&self, c: &Candidate, vault_id: &str, rel: &str, entity_kind: &str, entity_id: i64) -> ResyncPlan {
        let mine: Vec<&ImportedEvent> = self
            .events
            .iter()
            .filter(|e| e.vault_id == vault_id && e.relative_path == rel && e.entity_kind == entity_kind && e.entity_id == entity_id && !e.source_removed)
            .collect();
        let offset = mine.first().map(|e| e.epoch_offset).unwrap_or(0);
        let mut unmatched_new: Vec<&Event> = Vec::new();
        let mut matched = vec![false; mine.len()];
        for ev in &c.events {
            let key = line_key(ev);
            if let Some(i) = mine.iter().position(|m| m.line_key == key) {
                matched[i] = true;
            } else {
                unmatched_new.push(ev);
            }
        }
        let mut plan = ResyncPlan::default();
        for ev in unmatched_new {
            let pair = mine
                .iter()
                .enumerate()
                .find(|(i, m)| !matched[*i] && m.title == ev.name && m.kind == ev.kind.as_str());
            let mut shifted = ev.clone();
            shifted.start += offset;
            shifted.end = shifted.end.map(|y| y + offset);
            match pair {
                Some((i, m)) => {
                    matched[i] = true;
                    plan.changed.push((m.id, shifted));
                }
                None => plan.added.push(shifted),
            }
        }
        for (i, m) in mine.iter().enumerate() {
            if !matched[i] {
                plan.removed.push(m.id);
            }
        }
        plan
    }

    /// Applies the chosen parts of `plan` (design §4.5: the user takes each group).
    #[allow(clippy::too_many_arguments)]
    pub fn apply_resync(&mut self, plan: &ResyncPlan, take_added: bool, take_changed: bool, take_removed: bool, vault_id: &str, rel: &str, entity_kind: &str, entity_id: i64, private: bool) {
        if take_changed {
            for (id, ev) in &plan.changed {
                if let Some(r) = self.events.iter_mut().find(|r| r.id == *id) {
                    let off = r.epoch_offset;
                    // the line key is the note's own (pre-offset) identity
                    let mut raw = ev.clone();
                    raw.start -= off;
                    raw.end = raw.end.map(|y| y - off);
                    let (vid, rl, kind, eid, key) = (r.vault_id.clone(), r.relative_path.clone(), r.entity_kind.clone(), r.entity_id, line_key(&raw));
                    *r = row_of(*id, ev, &vid, &rl, &kind, eid, off, r.private, key);
                }
            }
        }
        if take_added {
            let offset = self.events.iter().find(|e| e.vault_id == vault_id && e.relative_path == rel).map(|e| e.epoch_offset).unwrap_or(0);
            for ev in &plan.added {
                let id = self.next_id;
                self.next_id += 1;
                let mut raw = ev.clone();
                raw.start -= offset;
                raw.end = raw.end.map(|y| y - offset);
                self.events.push(row_of(id, ev, vault_id, rel, entity_kind, entity_id, offset, private, line_key(&raw)));
            }
        }
        if take_removed {
            for r in self.events.iter_mut().filter(|r| plan.removed.contains(&r.id)) {
                r.source_removed = true;
            }
        }
    }
}

/// How a note's founding event relates to the settlement's own `founded` year (design §4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoundingCheck {
    /// The event is not about a founding, or the settlement has no `founded` to compare.
    NotApplicable,
    Agrees,
    /// Both stay visible; nothing is resolved automatically.
    Conflict { note: i64, founded: i64 },
}

/// Compares `ev` with `founded`. An event is a founding when its name contains `found` (case-insensitive).
pub fn founding_check(founded: Option<i64>, ev: &Event) -> FoundingCheck {
    let Some(f) = founded else { return FoundingCheck::NotApplicable };
    if !ev.name.to_lowercase().contains("found") {
        return FoundingCheck::NotApplicable;
    }
    if ev.start == f {
        FoundingCheck::Agrees
    } else {
        FoundingCheck::Conflict { note: ev.start, founded: f }
    }
}

#[cfg(test)]
mod tests;
