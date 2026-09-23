//! Authored, dated events read out of a note's ` ```chronos ` code blocks
//! (`STORY_PLANNING_SCOPE.md` §3, SP-3; `LARGE_ITEM_RULINGS.md` Ruling AM).
//!
//! **The syntax is the Chronos Timeline Obsidian plugin's, exactly, on
//! purpose.** The owner ruled (2026-09-23) that a settlement's authored
//! history lives in its ordinary vault note as a `chronos` fenced block, so
//! the same note opened in Obsidian with that plugin installed draws a
//! working timeline with no Cartalith involvement. A variant of the syntax
//! would break that for free, so none is accepted or invented here:
//!
//! ```text
//! - [Date~Date] #Color {Group Name} Event Name | Description    event
//! @ [Date~Date] #Color {Group Name} Period Name                 period
//! * [Date] Point Name | Description                             point
//! = [Date] Marker Name                                          marker
//! ```
//!
//! All four share one grammar (only the leading symbol differs), so all four
//! are read: rejecting `* [1200] Siege` would make valid Chronos show nothing
//! here. `> ORDERBY …`/`> DEFAULTVIEW …` flags and `#` comment lines are
//! valid Chronos that describe a *view*, not an event, and are passed over
//! without complaint.
//!
//! **Dates keep their year and nothing finer.** Chronos accepts
//! `YYYY-MM-DDThh:mm:ss` with only `YYYY` required; this port's clock is the
//! Timeline's signed `i64` year (`get_civ_timeline_years`, `civ_goto_year`)
//! and `STORY_PLANNING_SCOPE.md` §5 forbids a finer parallel clock, so
//! `[1879-03-14]` reads as year 1879 and the rest is validated loosely and
//! dropped. Negative years (`[-300~250]`) are valid Chronos and read as-is.
//!
//! **A note is hand-edited free text, so nothing here fails.** A line that
//! does not parse is returned in [`Chronos::skipped`] with its text, rather
//! than aborting the block — one typo must not blank a settlement's history,
//! and it must not vanish silently either.
//!
//! **Writing** (the owner's 2026-09-23 follow-up: *"even when using input
//! fields it should add the syntax accordingly"*): [`to_line`] is the inverse
//! of [`parse_line`] and refuses any value that would not read back
//! identically; [`append_line`] puts a line into a note's block, creating the
//! block if there is none. `VaultSession::add_chronos_event` is the hash-guarded
//! caller. New lines are **appended, never sorted in**: Chronos does not
//! assume source order (that is what its `> ORDERBY` flag is for) and this
//! port's reader sorts by year itself, so an insertion sort would only
//! reorder the author's own lines for nothing.

/// Which of Chronos' four item types a line is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `-`
    Event,
    /// `@` — a background span.
    Period,
    /// `*`
    Point,
    /// `=`
    Marker,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Event => "event",
            Kind::Period => "period",
            Kind::Point => "point",
            Kind::Marker => "marker",
        }
    }
}

/// One parsed line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub kind: Kind,
    pub start: i64,
    /// `Some` only for a `[a~b]` range.
    pub end: Option<i64>,
    /// The `#red` / `#ff8800` token without its `#`, as written.
    pub color: Option<String>,
    pub group: Option<String>,
    pub name: String,
    pub description: Option<String>,
}

/// Everything read from every ` ```chronos ` block in one text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Chronos {
    /// In the order written.
    pub events: Vec<Event>,
    /// Non-blank lines inside a block that are not valid Chronos, trimmed.
    pub skipped: Vec<String>,
    /// How many ` ```chronos ` blocks were found — so "no block" and "an
    /// empty block" are distinguishable.
    pub blocks: usize,
}

/// Every ` ```chronos ` block in `text`. An unclosed fence runs to the end
/// of the document, as CommonMark (and so Obsidian) reads one.
pub fn parse(text: &str) -> Chronos {
    let mut out = Chronos::default();
    let mut inside: Option<usize> = None; // the opening fence's backtick count
    for raw in text.lines() {
        let line = raw.trim();
        match inside {
            None => {
                let ticks = line.len() - line.trim_start_matches('`').len();
                if ticks >= 3 && line[ticks..].trim() == "chronos" {
                    inside = Some(ticks);
                    out.blocks += 1;
                }
            }
            Some(ticks) => {
                if line.len() >= ticks && line.bytes().all(|b| b == b'`') {
                    inside = None;
                } else if line.is_empty() || line.starts_with('>') || line.starts_with('#') {
                    // blank, a view flag, or a comment: valid, and not an event
                } else if let Some(ev) = parse_line(line) {
                    out.events.push(ev);
                } else {
                    out.skipped.push(line.to_string());
                }
            }
        }
    }
    out
}

/// One trimmed item line, or `None` if it is not one.
pub fn parse_line(line: &str) -> Option<Event> {
    let mut chars = line.chars();
    let kind = match chars.next()? {
        '-' => Kind::Event,
        '@' => Kind::Period,
        '*' => Kind::Point,
        '=' => Kind::Marker,
        _ => return None,
    };
    let rest = chars.as_str().trim_start().strip_prefix('[')?;
    let close = rest.find(']')?;
    let (start, end) = match rest[..close].split_once('~') {
        Some((a, b)) => (year(a)?, Some(year(b)?)),
        None => (year(&rest[..close])?, None),
    };
    let mut rest = rest[close + 1..].trim_start();

    let mut color = None;
    if let Some(r) = rest.strip_prefix('#') {
        let n = r.find(char::is_whitespace).unwrap_or(r.len());
        color = Some(r[..n].to_string()).filter(|c| !c.is_empty());
        rest = r[n..].trim_start();
    }
    let mut group = None;
    if let Some(r) = rest.strip_prefix('{') {
        let n = r.find('}')?;
        group = Some(r[..n].trim().to_string()).filter(|g| !g.is_empty());
        rest = r[n + 1..].trim_start();
    }
    let (name, description) = match rest.split_once('|') {
        Some((n, d)) => (n.trim(), Some(d.trim().to_string()).filter(|d| !d.is_empty())),
        None => (rest.trim(), None),
    };
    if name.is_empty() {
        return None;
    }
    Some(Event { kind, start, end, color, group, name: name.to_string(), description })
}

/// The signed year out of a Chronos date. The part after the year must be
/// empty or begin `-MM…` / `T…`, so `[12x]` is refused rather than read as 12.
fn year(s: &str) -> Option<i64> {
    let s = s.trim();
    let (neg, body) = match s.strip_prefix('-') {
        Some(b) => (true, b),
        None => (false, s),
    };
    let digits = body.len() - body.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    if digits == 0 {
        return None;
    }
    let tail = &body[digits..];
    if !(tail.is_empty() || tail.starts_with('-') || tail.starts_with('T')) {
        return None;
    }
    let y: i64 = body[..digits].parse().ok()?;
    Some(if neg { -y } else { y })
}

/// The one Chronos line for `e`, such that `parse_line(&to_line(e)?) ==
/// Some(e.clone())`:
///
/// ```text
/// - [start~end] #color {group} name | description
/// ```
///
/// with each optional part left out when absent. `Err` names the field that
/// cannot be written so it reads back the same — padding or a line break
/// anywhere, an empty optional (write `None`), a name starting `#` or `{` (the
/// reader would take it for a colour or a group; refused even when a colour or
/// group precedes it and it would survive — one rule an author can read beats
/// three) or holding `|` (the reader
/// would split a description off it), a colour holding whitespace, a group
/// holding `}`, or an end year before the start.
pub fn to_line(e: &Event) -> Result<String, String> {
    fn clean<'a>(what: &str, s: &'a str) -> Result<&'a str, String> {
        if s.trim().is_empty() {
            return Err(format!("the {what} is empty"));
        }
        if s.trim() != s || s.contains(['\n', '\r']) {
            return Err(format!("the {what} has leading/trailing spaces or a line break"));
        }
        Ok(s)
    }
    let name = clean("name", &e.name).map_err(|_| "an event needs a name, on one line".to_string())?;
    if name.starts_with(['#', '{']) || name.contains('|') {
        return Err("the name cannot start with # or {, or contain |".into());
    }
    let sym = match e.kind {
        Kind::Event => '-',
        Kind::Period => '@',
        Kind::Point => '*',
        Kind::Marker => '=',
    };
    let mut out = format!("{sym} [{}", e.start);
    if let Some(end) = e.end {
        if end < e.start {
            return Err(format!("the end year {end} is before the start year {}", e.start));
        }
        out += &format!("~{end}");
    }
    out.push(']');
    if let Some(c) = &e.color {
        let c = clean("colour", c)?;
        if c.contains(char::is_whitespace) {
            return Err("the colour must be one word, e.g. red or ff8800".into());
        }
        out += &format!(" #{c}");
    }
    if let Some(g) = &e.group {
        let g = clean("group", g)?;
        if g.contains('}') {
            return Err("the group cannot contain }".into());
        }
        out += &format!(" {{{g}}}");
    }
    out += &format!(" {name}");
    if let Some(d) = &e.description {
        out += &format!(" | {}", clean("description", d)?);
    }
    Ok(out)
}

/// `text` with `line` added as the last line of its last ` ```chronos `
/// block, or — when it has none — with a new block holding `line` appended
/// at the end of the note. Every other byte is kept.
///
/// A block inside Cartalith's machine block (`block.rs`) is **never** the
/// target, because that block is replaced wholesale on the next Cartalith
/// write (`STORY_PLANNING_SCOPE.md` §3) and the event would be lost; a note
/// whose machine block cannot be delimited is refused the same way
/// [`crate::block::upsert`] refuses it. The fence is recognised exactly as
/// [`parse`] recognises it, so the line lands where the reader looks.
pub fn append_line(text: &str, line: &str) -> Result<String, crate::block::BlockError> {
    let machine: Vec<std::ops::Range<usize>> =
        crate::block::blocks(text)?.into_iter().map(|b| b.span).collect();
    let nl = if text.contains("\r\n") { "\r\n" } else { "\n" };
    // (offset of the closing fence line, or None when unclosed) of the last
    // usable block; outer None = no usable block.
    let mut target: Option<Option<usize>> = None;
    let mut open: Option<(usize, bool)> = None; // (ticks, usable)
    let mut at = 0usize;
    for raw in text.split_inclusive('\n') {
        let l = raw.trim();
        match open {
            None => {
                let ticks = l.len() - l.trim_start_matches('`').len();
                if ticks >= 3 && l[ticks..].trim() == "chronos" {
                    let usable = !machine.iter().any(|m| m.contains(&at));
                    open = Some((ticks, usable));
                    if usable {
                        target = Some(None);
                    }
                }
            }
            Some((ticks, usable)) => {
                if l.len() >= ticks && l.bytes().all(|b| b == b'`') {
                    if usable {
                        target = Some(Some(at));
                    }
                    open = None;
                }
            }
        }
        at += raw.len();
    }
    // An unclosed usable block that is the LAST block runs to the end of the
    // note, so appending at the end lands inside it — as the reader sees it.
    let sep = |t: &str| if t.is_empty() || t.ends_with('\n') { "" } else { nl };
    Ok(match target {
        Some(Some(close)) => format!("{}{line}{nl}{}", &text[..close], &text[close..]),
        Some(None) => format!("{text}{}{line}{nl}", sep(text)),
        None => {
            let gap = if text.is_empty() || text.ends_with("\n\n") || text.ends_with("\r\n\r\n") {
                String::new()
            } else {
                format!("{}{nl}", sep(text))
            };
            format!("{text}{gap}```chronos{nl}{line}{nl}```{nl}")
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(body: &str) -> String {
        format!("# Nareth\n\nProse the author wrote.\n\n```chronos\n{body}\n```\n\nMore prose.\n")
    }

    #[test]
    fn the_owner_supplied_examples_read_exactly() {
        let c = parse(&block(
            "- [1879-03-14] Einstein born\n\
             - [1991~2001] Time I believed in Santa\n\
             - [1991~2001] Time I believed in Santa | ended when my brother tried to videotape Santa with a hidden camera\n\
             - [2001~2009] #red Bush\n\
             - [1916] {Marina Tsvetaeva} \"Подруга\"",
        ));
        assert_eq!(c.blocks, 1);
        assert!(c.skipped.is_empty(), "{:?}", c.skipped);
        assert_eq!(c.events.len(), 5);
        let e = &c.events[0];
        assert_eq!((e.kind, e.start, e.end, e.name.as_str()), (Kind::Event, 1879, None, "Einstein born"));
        assert_eq!((c.events[1].start, c.events[1].end), (1991, Some(2001)));
        assert_eq!(c.events[1].description, None);
        assert_eq!(
            c.events[2].description.as_deref(),
            Some("ended when my brother tried to videotape Santa with a hidden camera")
        );
        assert_eq!(c.events[2].name, "Time I believed in Santa");
        assert_eq!(c.events[3].color.as_deref(), Some("red"));
        assert_eq!(c.events[3].name, "Bush");
        assert_eq!(c.events[4].group.as_deref(), Some("Marina Tsvetaeva"));
        assert_eq!(c.events[4].name, "\"Подруга\"");
        assert_eq!(c.events[4].start, 1916);
    }

    #[test]
    fn negative_years_and_a_full_colour_group_description_line() {
        let c = parse(&block("- [-300~250] #ff8800 {Wars} The long siege | walls held"));
        let e = &c.events[0];
        assert_eq!((e.start, e.end), (-300, Some(250)));
        assert_eq!(e.color.as_deref(), Some("ff8800"));
        assert_eq!(e.group.as_deref(), Some("Wars"));
        assert_eq!(e.name, "The long siege");
        assert_eq!(e.description.as_deref(), Some("walls held"));
        assert_eq!(parse_line("- [-1200-06-01T12:00:00] Flood").unwrap().start, -1200);
    }

    #[test]
    fn periods_points_and_markers_share_the_grammar() {
        let c = parse(&block(
            "@ [1100~1250] #blue {Rule} Ashfall regency\n* [1180] Charter granted | by the regent\n= [1200] Census",
        ));
        let kinds: Vec<Kind> = c.events.iter().map(|e| e.kind).collect();
        assert_eq!(kinds, vec![Kind::Period, Kind::Point, Kind::Marker]);
        assert_eq!(c.events[0].end, Some(1250));
        assert_eq!(c.events[1].description.as_deref(), Some("by the regent"));
        assert_eq!(c.events[2].name, "Census");
    }

    #[test]
    fn a_malformed_line_is_skipped_and_reported_not_fatal() {
        let c = parse(&block(
            "- [1200] Good one\n- 1201 no brackets\n- [12x] bad year\n- [1203]\n- [1204 unclosed\nplain prose\n- [1205] Also good",
        ));
        let names: Vec<&str> = c.events.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["Good one", "Also good"]);
        assert_eq!(
            c.skipped,
            vec!["- 1201 no brackets", "- [12x] bad year", "- [1203]", "- [1204 unclosed", "plain prose"]
        );
    }

    #[test]
    fn flags_comments_and_blank_lines_are_neither_events_nor_errors() {
        let c = parse(&block("> ORDERBY start\n> DEFAULTVIEW -100|300\n\n# a comment\n- [7] Seven"));
        assert_eq!(c.events.len(), 1);
        assert!(c.skipped.is_empty(), "{:?}", c.skipped);
    }

    #[test]
    fn no_block_an_empty_block_and_other_fences() {
        assert_eq!(parse("# Nareth\n\n- [1200] Not in a block\n"), Chronos::default());
        assert_eq!(parse("```\n- [1200] plain fence\n```\n```rust\n- [1] x\n```\n").events.len(), 0);
        let empty = parse("```chronos\n```\n");
        assert_eq!((empty.blocks, empty.events.len()), (1, 0));
    }

    #[test]
    fn two_blocks_are_both_read_and_text_after_a_block_is_not() {
        let t = "```chronos\n- [1] A\n```\n- [2] outside\n````chronos\n- [3] B\n````\n";
        let c = parse(t);
        assert_eq!(c.blocks, 2);
        let names: Vec<&str> = c.events.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["A", "B"]);
    }

    #[test]
    fn an_unclosed_fence_runs_to_the_end_of_the_note() {
        let c = parse("```chronos\n- [1] A\n- [2] B");
        assert_eq!(c.events.len(), 2);
    }

    fn ev(start: i64, end: Option<i64>, color: Option<&str>, group: Option<&str>, name: &str, desc: Option<&str>) -> Event {
        Event {
            kind: Kind::Event,
            start,
            end,
            color: color.map(str::to_string),
            group: group.map(str::to_string),
            name: name.to_string(),
            description: desc.map(str::to_string),
        }
    }

    /// The write path's correctness bar: every combination of the four
    /// optional parts (2^4 = 16), each written by `to_line` and read back by
    /// the unmodified `parse_line` — and the literal text for the owner's own
    /// cheatsheet forms, so a serializer that round-tripped through some
    /// private dialect would still fail.
    #[test]
    fn every_optional_combination_round_trips_through_the_real_parser() {
        for mask in 0..16u8 {
            let e = ev(
                -250,
                (mask & 1 != 0).then_some(250),
                (mask & 2 != 0).then_some("ff8800"),
                (mask & 4 != 0).then_some("Marina Tsvetaeva"),
                "\"Подруга\" #2 ~ [x]",
                (mask & 8 != 0).then_some("walls held | twice"),
            );
            let line = to_line(&e).unwrap();
            assert_eq!(parse_line(&line), Some(e), "mask {mask}: {line}");
        }
        assert_eq!(to_line(&ev(1879, None, None, None, "Einstein born", None)).unwrap(), "- [1879] Einstein born");
        assert_eq!(to_line(&ev(2001, Some(2009), Some("red"), None, "Bush", None)).unwrap(), "- [2001~2009] #red Bush");
        assert_eq!(
            to_line(&ev(1991, Some(2001), None, None, "Time I believed in Santa", Some("ended when my brother tried to videotape Santa with a hidden camera"))).unwrap(),
            "- [1991~2001] Time I believed in Santa | ended when my brother tried to videotape Santa with a hidden camera"
        );
        assert_eq!(to_line(&ev(1916, None, None, Some("Marina Tsvetaeva"), "\"Подруга\"", None)).unwrap(), "- [1916] {Marina Tsvetaeva} \"Подруга\"");
        assert_eq!(
            to_line(&ev(-300, Some(250), Some("ff8800"), Some("Wars"), "The long siege", Some("walls held"))).unwrap(),
            "- [-300~250] #ff8800 {Wars} The long siege | walls held"
        );
    }

    #[test]
    fn the_other_three_kinds_write_their_own_symbol() {
        for (kind, sym) in [(Kind::Period, '@'), (Kind::Point, '*'), (Kind::Marker, '=')] {
            let e = Event { kind, ..ev(1200, None, None, None, "Census", None) };
            let line = to_line(&e).unwrap();
            assert_eq!(line, format!("{sym} [1200] Census"));
            assert_eq!(parse_line(&line), Some(e));
        }
    }

    /// Every value `to_line` refuses is one the reader would NOT give back
    /// as written — checked against the reader, not just asserted.
    #[test]
    fn values_that_would_not_read_back_are_refused() {
        let bad = [
            ev(1, None, None, None, "", None),
            ev(1, None, None, None, " padded", None),
            ev(1, None, None, None, "two\nlines", None),
            ev(1, None, None, None, "#hashtag", None),
            ev(1, None, None, None, "{braced}", None),
            ev(1, None, None, None, "a | b", None),
            ev(1, None, Some(""), None, "x", None),
            ev(1, None, Some("dark red"), None, "x", None),
            ev(1, None, None, Some("a}b"), "x", None),
            ev(1, None, None, Some(""), "x", None),
            ev(1, None, None, None, "x", Some("")),
            ev(1, None, None, None, "x", Some("a\nb")),
            ev(5, Some(4), None, None, "x", None),
        ];
        for e in bad {
            assert!(to_line(&e).is_err(), "should refuse {e:?}");
        }
        // Each refused name really does misread without the refusal.
        assert_eq!(parse_line("- [1] #hashtag").map(|e| e.name), None);
        assert_eq!(parse_line("- [1] a | b").unwrap().name, "a");
        // A same-year range is not "before", and is written.
        let same = ev(5, Some(5), None, None, "x", None);
        assert_eq!(parse_line(&to_line(&same).unwrap()), Some(same));
    }

    #[test]
    fn a_note_without_a_block_gets_one_at_the_end() {
        let t = "# Nareth\n\nProse.\n";
        let out = append_line(t, "- [1] A").unwrap();
        assert_eq!(out, "# Nareth\n\nProse.\n\n```chronos\n- [1] A\n```\n");
        let no_nl = append_line("# Nareth", "- [1] A").unwrap();
        assert_eq!(no_nl, "# Nareth\n\n```chronos\n- [1] A\n```\n");
        assert_eq!(append_line("", "- [1] A").unwrap(), "```chronos\n- [1] A\n```\n");
        assert_eq!(parse(&out).events[0].name, "A");
    }

    #[test]
    fn a_line_is_appended_inside_the_last_existing_block() {
        let t = block("- [2] B\n- [1] A");
        let out = append_line(&t, "- [3] C").unwrap();
        assert_eq!(out, t.replace("- [1] A\n```", "- [1] A\n- [3] C\n```"));
        let c = parse(&out);
        assert_eq!((c.blocks, c.events.len()), (1, 3));
        assert_eq!(c.events[2].name, "C", "appended in source order, not sorted");
        // Two blocks: the last one gets it. Longer fences are honoured.
        let two = "```chronos\n- [1] A\n```\n````chronos\n- [2] B\n````\ntail\n";
        assert_eq!(append_line(two, "- [3] C").unwrap(), "```chronos\n- [1] A\n```\n````chronos\n- [2] B\n- [3] C\n````\ntail\n");
        // Unclosed: runs to the end, so the line goes at the end.
        assert_eq!(append_line("```chronos\n- [1] A", "- [2] B").unwrap(), "```chronos\n- [1] A\n- [2] B\n");
        // CRLF notes stay CRLF.
        assert_eq!(append_line("```chronos\r\n- [1] A\r\n```\r\n", "- [2] B").unwrap(), "```chronos\r\n- [1] A\r\n- [2] B\r\n```\r\n");
    }

    #[test]
    fn a_block_inside_the_machine_block_is_never_the_target() {
        use crate::block::{BEGIN_PREFIX, END_MARKER};
        let t = format!("# N\n\n{BEGIN_PREFIX} entity=\"settlement:1\" version=\"1\" -->\n```chronos\n- [1] Machine\n```\n{END_MARKER}\n\nProse.\n");
        let out = append_line(&t, "- [2] Mine").unwrap();
        assert!(out.starts_with(&t), "the machine block is untouched");
        assert!(out.ends_with("Prose.\n\n```chronos\n- [2] Mine\n```\n"), "{out}");
        // An unterminated machine block is refused, not guessed around.
        assert!(append_line(&format!("{BEGIN_PREFIX} -->\n```chronos\n```\n"), "- [2] Mine").is_err());
    }
}
