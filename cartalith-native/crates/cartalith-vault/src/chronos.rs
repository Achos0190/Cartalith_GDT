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
//! Read-only. Nothing in this crate writes a `chronos` block.

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
}
