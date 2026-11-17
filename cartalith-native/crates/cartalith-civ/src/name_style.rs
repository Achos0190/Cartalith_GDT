//! Ruling FZ (owner, 2026-11-17): **name styles from CSV.**
//!
//! A *name style* is the user's own replacement for one of the seven hard-coded cultures
//! ([`crate::CIV_CULTURES`]: a syllable list and a suffix list each, no weights). It is imported from
//! a CSV file in columns a person can edit in a spreadsheet, and the same format is what
//! [`export_culture`] writes for any built-in culture as a worked example.
//!
//! # The format
//!
//! ```text
//! # A comment row: the importer skips any row whose first cell starts with '#'.
//! Name style,Highland
//! Syllables,2,4
//! Max length,16                      <- optional
//! [Settlements]
//! Start,Start weight,Middle,Middle weight,End,End weight,Suffix,Suffix weight
//! ar,3,bel,,cor,,heim,2
//! [Rivers]
//! Template,Weight
//! River {name},60
//! ```
//!
//! * A weight sits in the cell **next to** its fragment; blank means 1. A weight is the fragment's
//!   share of the draws: weight 3 against two weight-1 neighbours is chosen 3 times in 5. Weights
//!   must be positive numbers (a fragment you never want is a row you delete, not a weight of 0).
//! * `[Section]` rows separate the kinds of name. A section holds fragment columns (its own
//!   syllables, with the same four parts) and/or a `Template,Weight` block whose `{name}` stands
//!   for the stem the fragments built. **A section with no fragments borrows `[Settlements]`'s;
//!   a section with no templates uses the built-in ones** ([`default_templates`]).
//! * The suffix cell `(none)` is the empty suffix (a built-in culture has a lot of those).
//!
//! # How a stem is built (the user path)
//!
//! `n` pieces are drawn, `n` uniform in `Syllables min..=max`: piece 1 from Start, the last from
//! End, the pieces between from Middle (`n = 1` is Start alone), then one Suffix; the first letter
//! is capitalised. A list left empty falls back (no Middle: `n` is capped at 2; no End: the last
//! piece comes from Middle, else Start). The RNG draw order mirrors [`crate::civ_settle_name`]
//! (`1 + n + 1` draws) and a weighted pick is `r * total_weight` walked down the list **in file
//! order**, so a list whose every weight is 1 picks exactly what `(r * len) as usize` picks. That
//! is why [`export_culture`] followed by an import reproduces a built-in culture's names for the
//! same seed (a test pins it for all seven).
//!
//! # What this must never do
//!
//! * Change a built-in culture's output. Nothing here is reached unless a style is chosen; the
//!   built-in [`crate::civ_settle_name`] and [`crate::naming::decorate`] are untouched, which is
//!   what keeps `golden_parity_settlement_naming` green (`DECISIONS.md` section 7p).
//! * **Repair a row.** A bad row is reported with its line number and skipped
//!   ([`StyleIssue`]); it is never silently corrected, truncated or guessed at.
//!
//! # Process-wide registry
//!
//! The roster's culture column is a string key read by about ten naming call sites that take
//! `&[&str]`. Rather than thread a second argument through all of them, a style is *registered* under
//! the key `style:<id>` ([`register_style`]) and [`settle_name_for_key`] looks the key up. Entries
//! are keyed by the style's content hash in the id, so a stale entry is harmless; the registry
//! never feeds a golden because a built-in key never starts with `style:`.

use crate::naming::{FeatureKind, NAME_MAX_TRIES};
use crate::{Culture, CIV_CULTURES};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock, RwLock};

/// Prefix of a culture key that names a user style.
pub const STYLE_KEY_PREFIX: &str = "style:";

/// The suffix cell that means "the empty suffix".
pub const NO_SUFFIX: &str = "(none)";

/// The longest fragment kept, in characters. A labelled judgement: the longest built-in syllable is
/// 5 characters; 24 keeps a whole word usable as a Middle while refusing a pasted sentence.
pub const FRAGMENT_MAX_CHARS: usize = 24;

/// The heaviest weight accepted. A labelled judgement: far beyond any hand-tuned ratio, low enough
/// that a sum of weights stays exact in an `f64`.
pub const WEIGHT_MAX: f64 = 1_000_000.0;

/// The most syllables a style may ask for. A labelled judgement: eight pieces is already a sentence.
pub const SYLLABLES_MAX: usize = 8;

/// One section of a style: its stable key (the text between the brackets), a display label, and the
/// built-in templates it falls back to.
#[derive(Debug, Clone, Copy)]
pub struct SectionDef {
    pub key: &'static str,
    /// What the window shows.
    pub label: &'static str,
    /// The built-in templates `(text, weight)` used when the style has none. Empty means the bare stem.
    pub templates: &'static [(&'static str, f64)],
    /// The built-in feature kind whose [`crate::naming::decorate`] these templates reproduce, when
    /// there is one (so the export of a built-in culture is exact for it).
    pub decorate_kind: Option<FeatureKind>,
}

const BARE: &[(&str, f64)] = &[("{name}", 1.0)];

/// Every section a style may carry, in file order. Settlements first (the fragment fallback), then
/// the kinds the engine names today, then the kinds it does not name today (Ruling FZ item 2: *"every
/// other labelled or pinnable kind"*; the pinnable ones are `crate::poi::KIND_NAMES`, grouped in
/// [`section_for_kind`]).
///
/// The templates for kinds the engine already names are `decorate`'s own, as percentages, so a
/// user style that keeps them names exactly as a built-in culture does. The templates for the
/// rest are this port's own labelled judgement (nothing in the reference names them).
pub const SECTIONS: &[SectionDef] = &[
    SectionDef { key: "Settlements", label: "Settlements", templates: BARE, decorate_kind: None },
    SectionDef { key: "Provinces", label: "Provinces", templates: &[("{name} Province", 70.0), ("The {name} March", 15.0), ("{name} County", 15.0)], decorate_kind: Some(FeatureKind::Province) },
    SectionDef { key: "Continents", label: "Continents", templates: &[("The {name} Reach", 18.0), ("Greater {name}", 14.0), ("The {name} Expanse", 10.0), ("{name}", 58.0)], decorate_kind: Some(FeatureKind::Continent) },
    SectionDef { key: "World", label: "The world", templates: &[("The {name} Reach", 18.0), ("Greater {name}", 14.0), ("The {name} Expanse", 10.0), ("{name}", 58.0)], decorate_kind: Some(FeatureKind::Continent) },
    SectionDef { key: "Bays", label: "Bays", templates: &[("{name} Bay", 50.0), ("Gulf of {name}", 25.0), ("{name} Sound", 25.0)], decorate_kind: Some(FeatureKind::Bay) },
    SectionDef { key: "Lakes", label: "Lakes", templates: &[("Lake {name}", 62.0), ("{name} Mere", 23.0), ("{name} Water", 15.0)], decorate_kind: Some(FeatureKind::Lake) },
    SectionDef { key: "Mountains", label: "Mountains", templates: &[("The {name} Range", 55.0), ("The {name} Mountains", 25.0), ("{name} Heights", 20.0)], decorate_kind: Some(FeatureKind::MountainRange) },
    SectionDef { key: "Mounts", label: "Mounts (single peaks, volcanoes)", templates: &[("Mount {name}", 60.0), ("{name} Peak", 25.0), ("Mount {name}", 15.0)], decorate_kind: None },
    SectionDef { key: "Rivers", label: "Rivers", templates: &[("River {name}", 40.0), ("The {name}", 35.0), ("{name} River", 25.0)], decorate_kind: None },
    SectionDef { key: "Cliffs", label: "Cliffs", templates: &[("{name} Cliffs", 50.0), ("The {name} Scarp", 30.0), ("{name} Bluff", 20.0)], decorate_kind: None },
    SectionDef { key: "Holy sites", label: "Holy sites", templates: &[("The Shrine of {name}", 30.0), ("{name} Temple", 30.0), ("{name} Sanctuary", 20.0), ("The {name} Grove", 20.0)], decorate_kind: None },
    SectionDef { key: "Seas", label: "Seas and oceans", templates: &[("The {name} Sea", 60.0), ("{name} Ocean", 25.0), ("The {name} Deep", 15.0)], decorate_kind: None },
    SectionDef { key: "Islands", label: "Islands", templates: &[("{name} Isle", 40.0), ("{name} Island", 40.0), ("The {name} Skerries", 20.0)], decorate_kind: None },
    SectionDef { key: "Forests", label: "Forests and groves", templates: &[("The {name} Wood", 40.0), ("{name} Forest", 40.0), ("{name} Weald", 20.0)], decorate_kind: None },
    SectionDef { key: "Deserts", label: "Deserts", templates: &[("The {name} Waste", 40.0), ("{name} Desert", 40.0), ("The {name} Sands", 20.0)], decorate_kind: None },
    SectionDef { key: "Glaciers", label: "Glaciers and ice", templates: &[("{name} Glacier", 60.0), ("The {name} Ice", 40.0)], decorate_kind: None },
    SectionDef { key: "Gorges", label: "Gorges and canyons", templates: &[("{name} Gorge", 50.0), ("The {name} Cleft", 30.0), ("{name} Canyon", 20.0)], decorate_kind: None },
    SectionDef { key: "Waterfalls", label: "Waterfalls and springs", templates: &[("{name} Falls", 50.0), ("{name} Spring", 30.0), ("The {name} Cascade", 20.0)], decorate_kind: None },
    SectionDef { key: "Caves", label: "Caves", templates: &[("{name} Cave", 60.0), ("The {name} Hollow", 40.0)], decorate_kind: None },
    SectionDef { key: "Passes", label: "Passes and ridges", templates: &[("{name} Pass", 60.0), ("The {name} Gap", 20.0), ("{name} Ridge", 20.0)], decorate_kind: None },
    SectionDef { key: "Crossings", label: "Bridges, fords and crossings", templates: &[("{name} Bridge", 40.0), ("{name} Ford", 40.0), ("{name} Crossing", 20.0)], decorate_kind: None },
    SectionDef { key: "Castles", label: "Castles, forts and watchtowers", templates: &[("{name} Castle", 40.0), ("Fort {name}", 30.0), ("{name} Keep", 30.0)], decorate_kind: None },
    SectionDef { key: "Ruins", label: "Ruins and tombs", templates: &[("The Ruins of {name}", 50.0), ("{name} Barrow", 30.0), ("Old {name}", 20.0)], decorate_kind: None },
    SectionDef { key: "Monuments", label: "Monuments and rock formations", templates: &[("The {name} Stone", 40.0), ("{name} Monument", 30.0), ("The {name} Spire", 30.0)], decorate_kind: None },
    SectionDef { key: "Harbours", label: "Harbours and portages", templates: &[("{name} Harbour", 60.0), ("{name} Landing", 40.0)], decorate_kind: None },
    SectionDef { key: "Inns", label: "Inns, taverns and camps", templates: &[("The {name} Arms", 40.0), ("The {name} Inn", 40.0), ("The Sleeping {name}", 20.0)], decorate_kind: None },
    SectionDef { key: "Markets", label: "Markets, guilds and depots", templates: &[("{name} Market", 50.0), ("{name} Exchange", 30.0), ("{name} Depot", 20.0)], decorate_kind: None },
    SectionDef { key: "Mines", label: "Mines, quarries and salt works", templates: &[("{name} Mine", 50.0), ("{name} Quarry", 30.0), ("The {name} Works", 20.0)], decorate_kind: None },
    SectionDef { key: "Fields", label: "Farmland and wells", templates: &[("{name} Fields", 50.0), ("{name} Farm", 30.0), ("{name} Well", 20.0)], decorate_kind: None },
    SectionDef { key: "Battlefields", label: "Battlefields", templates: &[("The Field of {name}", 50.0), ("{name} Field", 50.0)], decorate_kind: None },
    SectionDef { key: "Hazards", label: "Hazards (rockfalls, wildlife)", templates: &[("{name} Rockfall", 50.0), ("The {name} Hazard", 50.0)], decorate_kind: None },
    SectionDef { key: "Landmarks", label: "Any other landmark or pin", templates: BARE, decorate_kind: None },
];

/// The section a pin or landmark kind name belongs to (`crate::poi::KIND_NAMES` and the legacy
/// [`crate::poi::PoiKind`] keys). Every kind name maps somewhere; an unknown or custom kind maps to
/// `Landmarks`, so a pin always has a section to take its name from.
pub fn section_for_kind(kind: &str) -> &'static str {
    match kind {
        "capital" | "city" | "town" | "village" | "hamlet" => "Settlements",
        "mountain" => "Mountains",
        "volcano" | "sacred_mountain" | "peak" | "volcanic_feature" => "Mounts",
        "cliff" => "Cliffs",
        "temple" | "shrine" | "sacred_grove" | "ceremonial_site" | "pilgrimage_site" => "Holy sites",
        "lake" => "Lakes",
        "delta" | "river_confluence" => "Rivers",
        "forest" | "ancient_forest" => "Forests",
        "desert" => "Deserts",
        "glacier" | "glacial_feature" => "Glaciers",
        "gorge" => "Gorges",
        "waterfall" | "spring" => "Waterfalls",
        "cave" => "Caves",
        "mountain_pass" | "fortified_pass" | "ridge" | "saddle" | "ancient_road" | "road_junction" => "Passes",
        "bridge" | "bridge_site" | "river_crossing" | "ford" | "fortified_crossing" | "historic_crossing" | "gate" | "border_marker" => "Crossings",
        "castle" | "fort" | "watchtower" | "destroyed_fortress" => "Castles",
        "ruin" | "tomb" | "abandoned_settlement" => "Ruins",
        "monument" | "rock_formation" => "Monuments",
        "harbour" | "portage" | "caravan_station" => "Harbours",
        "inn" | "tavern" | "camp" => "Inns",
        "market" | "market_site" | "shop" | "guild" | "trade_depot" | "smithy" => "Markets",
        "mine" | "quarry" | "salt_works" | "resource_extraction_site" => "Mines",
        "farmland" | "well" => "Fields",
        "battlefield" | "battlefield_historic" => "Battlefields",
        "hazard" | "rockfall" | "wildlife" => "Hazards",
        _ => "Landmarks",
    }
}

/// The definition of a section by key, or `None` for a key this build does not know.
pub fn section_def(key: &str) -> Option<&'static SectionDef> {
    let norm = |t: &str| t.chars().filter(|c| !c.is_whitespace() && *c != '_').collect::<String>().to_ascii_lowercase();
    let want = norm(key);
    SECTIONS.iter().find(|s| norm(s.key) == want)
}

/// The built-in templates a section falls back to (`BARE` for an unknown key).
pub fn default_templates(key: &str) -> &'static [(&'static str, f64)] {
    section_def(key).map(|s| s.templates).unwrap_or(BARE)
}

/// A weighted pick list. Order is file order and is part of the contract (see the module doc).
pub type Weighted = Vec<(String, f64)>;

/// One section's own fragments (each list may be empty).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Fragments {
    pub start: Weighted,
    pub middle: Weighted,
    pub end: Weighted,
    pub suffix: Weighted,
}

impl Fragments {
    fn is_empty(&self) -> bool {
        self.start.is_empty() && self.middle.is_empty() && self.end.is_empty() && self.suffix.is_empty()
    }
}

/// What one section holds.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SectionStyle {
    pub fragments: Fragments,
    pub templates: Weighted,
}

/// A name style.
#[derive(Debug, Clone, PartialEq)]
pub struct NameStyle {
    pub name: String,
    pub min_syl: usize,
    pub max_syl: usize,
    pub max_len: Option<usize>,
    /// Keyed by [`SectionDef::key`].
    pub sections: BTreeMap<String, SectionStyle>,
}

/// One problem found while importing. `line` is 1-based; `0` means the file as a whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleIssue {
    pub line: usize,
    /// `true` for an error (the row was skipped, or the style refused); `false` for a note.
    pub error: bool,
    pub message: String,
}

/// The result of [`parse_style_csv`]: a style (if the file held a usable one) and every issue.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseReport {
    pub style: Option<NameStyle>,
    pub issues: Vec<StyleIssue>,
}

/// Split one CSV line into cells (RFC 4180 quoting: `"a,b"`, `""` for a quote). `Err` for an
/// unterminated quote. A tab-free, single-line reader: the format has no multi-line cell.
fn split_csv_line(line: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_q = false;
    let mut chars = line.chars().peekable();
    let mut was_quoted = false;
    while let Some(c) = chars.next() {
        if in_q {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    cur.push('"');
                } else {
                    in_q = false;
                }
            } else {
                cur.push(c);
            }
        } else if c == '"' && cur.is_empty() && !was_quoted {
            in_q = true;
            was_quoted = true;
        } else if c == ',' {
            out.push(std::mem::take(&mut cur));
            was_quoted = false;
        } else {
            cur.push(c);
        }
    }
    if in_q {
        return Err("a quote is opened and never closed".into());
    }
    out.push(cur);
    Ok(out)
}

/// A CSV cell for writing: quoted when it holds a comma, quote or leading `#`/`[`.
fn csv_cell(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.starts_with('#') || s.starts_with('[') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn parse_weight(cell: &str) -> Result<f64, String> {
    let t = cell.trim();
    if t.is_empty() {
        return Ok(1.0);
    }
    let w: f64 = t.parse().map_err(|_| format!("weight \"{t}\" is not a number"))?;
    if !w.is_finite() || w <= 0.0 {
        return Err(format!("weight \"{t}\" must be a positive number (delete the row to remove a fragment)"));
    }
    if w > WEIGHT_MAX {
        return Err(format!("weight \"{t}\" is above {WEIGHT_MAX}"));
    }
    Ok(w)
}

fn check_fragment(text: &str, is_suffix: bool) -> Result<String, String> {
    let t = text.trim();
    if is_suffix && t == NO_SUFFIX {
        return Ok(String::new());
    }
    if t.is_empty() {
        return Err("empty fragment".into());
    }
    if t.chars().count() > FRAGMENT_MAX_CHARS {
        return Err(format!("fragment \"{t}\" is longer than {FRAGMENT_MAX_CHARS} characters"));
    }
    if t.chars().any(|c| c.is_control() || c == '{' || c == '}') {
        return Err(format!("fragment \"{t}\" holds a control character or a brace"));
    }
    Ok(t.to_string())
}

fn check_template(text: &str) -> Result<String, String> {
    let t = text.trim();
    if t.matches("{name}").count() != 1 {
        return Err(format!("template \"{t}\" must hold {{name}} exactly once"));
    }
    if t.replace("{name}", "").contains(['{', '}']) {
        return Err(format!("template \"{t}\" holds a brace other than {{name}}"));
    }
    if t.chars().any(|c| c.is_control()) || t.chars().count() > 80 {
        return Err(format!("template \"{t}\" is too long or holds a control character"));
    }
    Ok(t.to_string())
}

#[derive(Clone, Copy, PartialEq)]
enum Block {
    None,
    Fragments([Option<usize>; 8]),
    Templates,
}

/// Parse a name-style CSV. Every row is judged on its own: a bad row is reported by line number and
/// skipped, never repaired; the rows around it still import. A style is returned when the file names
/// itself and holds at least one `[Settlements]` Start fragment; otherwise `style` is `None` and the
/// issues say why. `fallback_name` is used only to *say* in an issue what is missing, never to fill
/// in a missing `Name style` row (that is refused).
pub fn parse_style_csv(text: &str) -> ParseReport {
    let mut issues: Vec<StyleIssue> = Vec::new();
    let mut err = |line: usize, m: String, issues: &mut Vec<StyleIssue>| issues.push(StyleIssue { line, error: true, message: m });
    let mut name: Option<String> = None;
    let (mut min_syl, mut max_syl): (Option<usize>, Option<usize>) = (None, None);
    let mut max_len: Option<usize> = None;
    let mut sections: BTreeMap<String, SectionStyle> = BTreeMap::new();
    let mut cur: Option<String> = None;
    let mut block = Block::None;
    let mut seen_header_rows: BTreeSet<(String, bool)> = BTreeSet::new();
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    for (i, raw) in text.lines().enumerate() {
        let ln = i + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.chars().all(|c| c == ',') {
            continue;
        }
        let cells = match split_csv_line(raw) {
            Ok(c) => c,
            Err(e) => {
                err(ln, e, &mut issues);
                continue;
            }
        };
        let first = cells[0].trim();
        if first.starts_with('#') {
            continue;
        }
        let first_l = first.to_ascii_lowercase();
        let rest_blank = |from: usize| cells.iter().skip(from).all(|c| c.trim().is_empty());
        // Separator row: [Section].
        if first.starts_with('[') {
            if !first.ends_with(']') || !rest_blank(1) {
                err(ln, format!("section row \"{first}\" must be just [Name]"), &mut issues);
                cur = None;
                block = Block::None;
                continue;
            }
            let key = first[1..first.len() - 1].trim();
            match section_def(key) {
                Some(d) => {
                    cur = Some(d.key.to_string());
                    sections.entry(d.key.to_string()).or_default();
                }
                None => {
                    err(ln, format!("unknown section [{key}]; its rows are skipped"), &mut issues);
                    cur = None;
                }
            }
            block = Block::None;
            continue;
        }
        if first_l == "name style" {
            let n = cells.get(1).map(|s| s.trim()).unwrap_or("");
            if n.is_empty() || n.chars().count() > 60 || n.chars().any(|c| c.is_control()) {
                err(ln, "Name style needs a name of 1 to 60 characters".into(), &mut issues);
            } else if name.is_some() {
                err(ln, "a second Name style row; the first is kept".into(), &mut issues);
            } else {
                name = Some(n.to_string());
            }
            continue;
        }
        if first_l == "syllables" {
            let a = cells.get(1).map(|s| s.trim().parse::<usize>());
            let b = cells.get(2).map(|s| s.trim().parse::<usize>());
            match (a, b) {
                (Some(Ok(a)), Some(Ok(b))) if a >= 1 && a <= b && b <= SYLLABLES_MAX => {
                    min_syl = Some(a);
                    max_syl = Some(b);
                }
                _ => err(ln, format!("Syllables needs two whole numbers, 1 <= min <= max <= {SYLLABLES_MAX}"), &mut issues),
            }
            continue;
        }
        if first_l == "max length" {
            match cells.get(1).map(|s| s.trim().parse::<usize>()) {
                Some(Ok(n)) if (3..=60).contains(&n) => max_len = Some(n),
                _ => err(ln, "Max length needs a whole number from 3 to 60".into(), &mut issues),
            }
            continue;
        }
        // Column header rows.
        if first_l == "template" {
            let Some(sec) = cur.clone() else {
                err(ln, "a Template block before any [Section] row".into(), &mut issues);
                continue;
            };
            let ok = cells.get(1).map(|s| s.trim().eq_ignore_ascii_case("weight")).unwrap_or(false);
            if !ok {
                err(ln, "the template header must be Template,Weight".into(), &mut issues);
                block = Block::None;
                continue;
            }
            block = Block::Templates;
            seen_header_rows.insert((sec, true));
            continue;
        }
        if first_l == "start" || first_l == "middle" || first_l == "end" || first_l == "suffix" {
            let Some(sec) = cur.clone() else {
                err(ln, "a fragment header before any [Section] row".into(), &mut issues);
                continue;
            };
            // Columns by name: Start, Start weight, Middle, ...
            let mut idx = [None; 8];
            let mut bad = None;
            for (ci, c) in cells.iter().enumerate() {
                let c = c.trim().to_ascii_lowercase();
                if c.is_empty() {
                    continue;
                }
                let slot = match c.as_str() {
                    "start" => 0,
                    "start weight" => 1,
                    "middle" => 2,
                    "middle weight" => 3,
                    "end" => 4,
                    "end weight" => 5,
                    "suffix" => 6,
                    "suffix weight" => 7,
                    _ => {
                        bad = Some(c);
                        break;
                    }
                };
                if idx[slot].is_some() {
                    bad = Some(format!("{c} (twice)"));
                    break;
                }
                idx[slot] = Some(ci);
            }
            if let Some(b) = bad {
                err(ln, format!("unknown or repeated column \"{b}\" in the fragment header"), &mut issues);
                block = Block::None;
                continue;
            }
            block = Block::Fragments(idx);
            seen_header_rows.insert((sec, false));
            continue;
        }
        // Data rows.
        let Some(sec) = cur.clone() else {
            err(ln, "a row before any [Section]; skipped".into(), &mut issues);
            continue;
        };
        match block {
            Block::None => err(ln, "a data row with no Start/Template header above it; skipped".into(), &mut issues),
            Block::Templates => {
                let t = check_template(&cells[0]);
                let w = parse_weight(cells.get(1).map(|s| s.as_str()).unwrap_or(""));
                match (t, w) {
                    (Ok(t), Ok(w)) => sections.entry(sec).or_default().templates.push((t, w)),
                    (Err(e), _) | (_, Err(e)) => err(ln, e, &mut issues),
                }
            }
            Block::Fragments(idx) => {
                // Judge every cell first; any error skips the whole row (never half of it).
                let cell = |slot: usize| idx[slot].and_then(|c| cells.get(c)).map(|s| s.trim().to_string()).unwrap_or_default();
                let mut row: Vec<(usize, String, f64)> = Vec::new();
                let mut row_err: Option<String> = None;
                for part in 0..4 {
                    let frag = cell(part * 2);
                    let wcell = cell(part * 2 + 1);
                    if frag.is_empty() {
                        if !wcell.is_empty() {
                            row_err = Some(format!("a weight with no fragment next to it in column {}", ["Start", "Middle", "End", "Suffix"][part]));
                            break;
                        }
                        continue;
                    }
                    match (check_fragment(&frag, part == 3), parse_weight(&wcell)) {
                        (Ok(f), Ok(w)) => row.push((part, f, w)),
                        (Err(e), _) | (_, Err(e)) => {
                            row_err = Some(e);
                            break;
                        }
                    }
                }
                if row_err.is_none() && row.is_empty() {
                    row_err = Some("a row with no fragment in any column".into());
                }
                match row_err {
                    Some(e) => err(ln, e, &mut issues),
                    None => {
                        let s = sections.entry(sec).or_default();
                        for (part, f, w) in row {
                            let list = match part {
                                0 => &mut s.fragments.start,
                                1 => &mut s.fragments.middle,
                                2 => &mut s.fragments.end,
                                _ => &mut s.fragments.suffix,
                            };
                            list.push((f, w));
                        }
                    }
                }
            }
        }
    }
    // Drop sections that ended up with nothing (a header and no valid rows) so they fall back.
    sections.retain(|_, s| !s.fragments.is_empty() || !s.templates.is_empty());
    let Some(name) = name else {
        issues.push(StyleIssue { line: 0, error: true, message: "no \"Name style,<name>\" row; the file is refused".into() });
        return ParseReport { style: None, issues };
    };
    let have_settle = sections.get("Settlements").map(|s| !s.fragments.start.is_empty()).unwrap_or(false);
    if !have_settle {
        issues.push(StyleIssue { line: 0, error: true, message: "no [Settlements] Start fragment; every other section borrows from it, so the file is refused".into() });
        return ParseReport { style: None, issues };
    }
    let (min_syl, max_syl) = match (min_syl, max_syl) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            issues.push(StyleIssue { line: 0, error: false, message: "no Syllables row: using 2 to 4, the built-in range".into() });
            (2, 4)
        }
    };
    ParseReport { style: Some(NameStyle { name, min_syl, max_syl, max_len, sections }), issues }
}

/// The pick by cumulative weight, in file order. `r` is in `[0, 1)`.
fn pick<'a>(list: &'a [(String, f64)], r: f64) -> &'a str {
    let total: f64 = list.iter().map(|(_, w)| *w).sum();
    let x = r * total;
    let mut acc = 0.0;
    for (t, w) in list {
        acc += *w;
        if x < acc {
            return t;
        }
    }
    &list[list.len() - 1].0
}

fn weighted_mean_len(list: &[(String, f64)]) -> f64 {
    let total: f64 = list.iter().map(|(_, w)| *w).sum();
    if total <= 0.0 {
        return 0.0;
    }
    list.iter().map(|(t, w)| t.chars().count() as f64 * *w).sum::<f64>() / total
}

impl NameStyle {
    /// The fragments a section draws from: its own when it has any Start, else `[Settlements]`'s.
    fn fragments_for(&self, section: &str) -> &Fragments {
        if let Some(s) = self.sections.get(section) {
            if !s.fragments.start.is_empty() {
                return &s.fragments;
            }
        }
        &self.sections["Settlements"].fragments
    }

    /// One bare stem for `section`: `1 + n + 1` draws, mirroring [`crate::civ_settle_name`].
    pub fn stem(&self, rng: &mut cartalith_rng::Mulberry32, section: &str) -> String {
        let f = self.fragments_for(section);
        let span = self.max_syl - self.min_syl + 1;
        let mut n = self.min_syl + (rng.next_f64() * span as f64) as usize;
        if f.middle.is_empty() {
            n = n.min(2);
        }
        let mut s = String::new();
        for k in 0..n {
            let last = k + 1 == n;
            let list: &Weighted = if k == 0 {
                &f.start
            } else if last {
                if !f.end.is_empty() { &f.end } else if !f.middle.is_empty() { &f.middle } else { &f.start }
            } else {
                &f.middle
            };
            s.push_str(pick(list, rng.next_f64()));
        }
        let suffix = if f.suffix.is_empty() { "" } else { pick(&f.suffix, rng.next_f64()) };
        let mut chars = s.chars();
        let mut out = String::with_capacity(s.len() + suffix.len());
        if let Some(c) = chars.next() {
            out.extend(c.to_uppercase());
        }
        out.push_str(chars.as_str());
        out.push_str(suffix);
        out
    }

    /// The length above which a bounded name is redrawn: `Max length` when set, else the same
    /// self-tuned rule as [`crate::naming::culture_name_len_limit`] (mean length of a name at the
    /// middle of the syllable range, times 1.4, rounded up).
    pub fn len_limit(&self, section: &str) -> usize {
        if let Some(m) = self.max_len {
            return m;
        }
        let f = self.fragments_for(section);
        let first = weighted_mean_len(&f.start);
        let mid = if f.middle.is_empty() { first } else { weighted_mean_len(&f.middle) };
        let last = if !f.end.is_empty() { weighted_mean_len(&f.end) } else { mid };
        let n = (self.min_syl + self.max_syl) as f64 / 2.0;
        let body = if n <= 1.0 { first } else { first + (n - 2.0).max(0.0) * mid + last };
        ((body + weighted_mean_len(&f.suffix)) * crate::naming::NAME_LEN_FACTOR).ceil() as usize
    }

    /// [`Self::stem`] with the length bound and per-map uniqueness: the same loop as
    /// [`crate::naming::civ_settle_name_bounded_in`] (10 tries, then a numeric discriminator).
    pub fn stem_bounded(&self, rng: &mut cartalith_rng::Mulberry32, section: &str, seen: &mut BTreeSet<String>) -> String {
        let limit = self.len_limit(section);
        let mut last = String::new();
        for _ in 0..NAME_MAX_TRIES {
            let cand = self.stem(rng, section);
            if cand.chars().count() <= limit && !seen.contains(&cand) {
                seen.insert(cand.clone());
                return cand;
            }
            last = cand;
        }
        if seen.contains(&last) {
            for n in 2usize.. {
                let alt = format!("{last} {n}");
                if !seen.contains(&alt) {
                    seen.insert(alt.clone());
                    return alt;
                }
            }
        }
        seen.insert(last.clone());
        last
    }

    /// One template for `section`: the style's own, else the built-in ones. Exactly **one** RNG draw,
    /// like [`crate::naming::decorate`].
    pub fn decorate(&self, stem: &str, section: &str, rng: &mut cartalith_rng::Mulberry32) -> String {
        let r = rng.next_f64();
        let own = self.sections.get(section).map(|s| &s.templates).filter(|t| !t.is_empty());
        let t = match own {
            Some(list) => pick(list, r).to_string(),
            None => {
                let d = default_templates(section);
                let total: f64 = d.iter().map(|(_, w)| *w).sum();
                let x = r * total;
                let mut acc = 0.0;
                let mut chosen = d[d.len() - 1].0;
                for (t, w) in d {
                    acc += *w;
                    if x < acc {
                        chosen = t;
                        break;
                    }
                }
                chosen.to_string()
            }
        };
        t.replace("{name}", stem)
    }

    /// A finished name for a feature of `section`: a bounded stem, then a template.
    pub fn feature_name(&self, rng: &mut cartalith_rng::Mulberry32, section: &str, seen: &mut BTreeSet<String>) -> String {
        let stem = self.stem_bounded(rng, section, seen);
        self.decorate(&stem, section, rng)
    }

    /// A settlement name: the unbounded stem, like `civ_settle_name` (so a settlement's stream keeps
    /// its draw count), unless the style set `Max length`, in which case the stem is redrawn up to
    /// [`NAME_MAX_TRIES`] times to fit (the last candidate is kept after that, as the built-in bound does).
    pub fn settlement_name(&self, rng: &mut cartalith_rng::Mulberry32) -> String {
        match self.max_len {
            None => self.stem(rng, "Settlements"),
            Some(m) => {
                let mut last = String::new();
                for _ in 0..NAME_MAX_TRIES {
                    last = self.stem(rng, "Settlements");
                    if last.chars().count() <= m {
                        break;
                    }
                }
                last
            }
        }
    }

    /// A stable content id for the library and the registry: FNV-1a over the style's canonical CSV.
    pub fn id(&self) -> String {
        let csv = export_style(self);
        let mut h: u64 = 0xcbf29ce484222325;
        for b in csv.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        format!("{h:016x}")
    }

    /// The registry key of this style, `style:<id>`.
    pub fn key(&self) -> String {
        format!("{STYLE_KEY_PREFIX}{}", self.id())
    }
}

const HEADER: &str = "# Cartalith name style. Rows starting with # are comments the importer skips.\n\
# WEIGHTS: the number in a \"... weight\" column sits next to its fragment and says how often it is chosen\n\
# relative to the others in the same column. Blank means 1. A weight of 3 beside two blank (1) neighbours is\n\
# chosen 3 times in 5. Weights must be positive; delete a row to remove a fragment.\n\
# A name is built from 2 to 4 pieces (see Syllables): the first from Start, the last from End, any between from\n\
# Middle, then one Suffix. The Suffix cell (none) means no suffix.\n\
# A [Section] row starts each kind of name. A section with no fragments borrows [Settlements]'s; a section\n\
# with no templates uses the built-in ones. In a Template, {name} stands for the stem the fragments built.\n\
# Max length (optional) is the longest name in characters; leave it out to let the style size itself.\n";

/// The style as CSV text in the documented format (the writer behind both exports and [`NameStyle::id`]).
pub fn export_style(st: &NameStyle) -> String {
    let mut o = String::from(HEADER);
    o.push_str(&format!("Name style,{}\nSyllables,{},{}\n", csv_cell(&st.name), st.min_syl, st.max_syl));
    if let Some(m) = st.max_len {
        o.push_str(&format!("Max length,{m}\n"));
    }
    let wcell = |w: f64| if (w - 1.0).abs() < f64::EPSILON { String::new() } else { format!("{w}") };
    for def in SECTIONS {
        let Some(sec) = st.sections.get(def.key) else { continue };
        o.push_str(&format!("[{}]\n", def.key));
        if !sec.fragments.is_empty() {
            o.push_str("Start,Start weight,Middle,Middle weight,End,End weight,Suffix,Suffix weight\n");
            let f = &sec.fragments;
            let rows = f.start.len().max(f.middle.len()).max(f.end.len()).max(f.suffix.len());
            for i in 0..rows {
                let mut cells: Vec<String> = Vec::new();
                for (k, l) in [&f.start, &f.middle, &f.end, &f.suffix].iter().enumerate() {
                    match l.get(i) {
                        Some((t, w)) => {
                            let shown = if k == 3 && t.is_empty() { NO_SUFFIX.to_string() } else { csv_cell(t) };
                            cells.push(shown);
                            cells.push(wcell(*w));
                        }
                        None => {
                            cells.push(String::new());
                            cells.push(String::new());
                        }
                    }
                }
                o.push_str(&cells.join(","));
                o.push('\n');
            }
        }
        if !sec.templates.is_empty() {
            o.push_str("Template,Weight\n");
            for (t, w) in &sec.templates {
                o.push_str(&format!("{},{}\n", csv_cell(t), wcell(*w)));
            }
        }
    }
    o
}

/// Collapse an in-order list with repeated entries into `(text, count)` by **first occurrence**.
/// Contiguous repeats become one weighted entry that draws identically to the repeated list; a
/// non-contiguous repeat moves later entries (documented in the module header: such a list does
/// not draw identically, and [`export_culture`]'s round-trip test names which cultures have none).
fn collapse(list: &[&str]) -> Weighted {
    let mut out: Weighted = Vec::new();
    for s in list {
        match out.iter_mut().find(|(t, _)| t == s) {
            Some(e) => e.1 += 1.0,
            None => out.push(((*s).to_string(), 1.0)),
        }
    }
    out
}

/// A built-in culture as a [`NameStyle`] (what [`export_culture`] writes): today's one syllable list
/// becomes Start, Middle and End; repeated entries become weights; the suffix list likewise; and every
/// section the engine names today carries `decorate`'s own templates, so the export is a complete
/// worked example (the other sections carry this port's labelled templates).
pub fn culture_as_style(cul: &Culture) -> NameStyle {
    let syl = collapse(cul.syl);
    let mut sections = BTreeMap::new();
    for def in SECTIONS {
        let mut s = SectionStyle::default();
        if def.key == "Settlements" {
            s.fragments = Fragments { start: syl.clone(), middle: syl.clone(), end: syl.clone(), suffix: collapse(cul.sfx) };
        }
        if def.key != "Settlements" {
            s.templates = def.templates.iter().map(|(t, w)| ((*t).to_string(), *w)).collect();
        }
        sections.insert(def.key.to_string(), s);
    }
    NameStyle { name: cul.key.to_string(), min_syl: 2, max_syl: 4, max_len: None, sections }
}

/// Ruling FZ item 3: write a built-in culture in the exact import format.
pub fn export_culture(cul: &Culture) -> String {
    export_style(&culture_as_style(cul))
}

/// The built-in culture with this key, if any (no fallback to `common`).
pub fn builtin_culture(key: &str) -> Option<&'static Culture> {
    CIV_CULTURES.iter().find(|c| c.key == key)
}

fn registry() -> &'static RwLock<BTreeMap<String, Arc<NameStyle>>> {
    static R: OnceLock<RwLock<BTreeMap<String, Arc<NameStyle>>>> = OnceLock::new();
    R.get_or_init(|| RwLock::new(BTreeMap::new()))
}

/// Register `style` and return its culture key (`style:<id>`). Idempotent; replaces nothing a
/// different style could own, because the id is the content hash.
pub fn register_style(style: NameStyle) -> String {
    let key = style.key();
    if let Ok(mut r) = registry().write() {
        r.entry(key.clone()).or_insert_with(|| Arc::new(style));
    }
    key
}

/// Register `style` under a key a saved world already uses (`style:<id>` as written by an older or newer
/// build, whose canonical CSV hashes differently). The first registration of a key wins, so a world's copy
/// can never replace the library's. A key without the `style:` prefix is refused.
pub fn register_style_under(key: &str, style: NameStyle) -> bool {
    if !key.starts_with(STYLE_KEY_PREFIX) {
        return false;
    }
    if let Ok(mut r) = registry().write() {
        r.entry(key.to_string()).or_insert_with(|| Arc::new(style));
    }
    true
}

/// The registered style for a culture key, if the key names one.
pub fn style_for_key(key: &str) -> Option<Arc<NameStyle>> {
    if !key.starts_with(STYLE_KEY_PREFIX) {
        return None;
    }
    registry().read().ok().and_then(|r| r.get(key).cloned())
}

/// What a roster's culture column makes of a faction's settlement name: the user style's when the
/// faction's key is a registered `style:` key, else exactly [`crate::civ_settle_name`] in the
/// built-in culture (so every built-in key behaves byte-for-byte as before).
pub fn settle_name_for_key(rng: &mut cartalith_rng::Mulberry32, cultures: &[&str], faction: i32) -> String {
    if let Some(key) = usize::try_from(faction).ok().and_then(|f| cultures.get(f)) {
        if let Some(st) = style_for_key(key) {
            return st.settlement_name(rng);
        }
    }
    crate::civ_settle_name(rng, crate::civ_faction_culture(cultures, faction))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::naming::{civ_settle_name_bounded_in, decorate};
    use cartalith_rng::Mulberry32;

    fn parse_ok(t: &str) -> NameStyle {
        let r = parse_style_csv(t);
        assert!(r.issues.iter().all(|i| !i.error), "{:?}", r.issues);
        r.style.expect("a style")
    }

    /// Protects: the documented minimal file parses, weights land next to their fragments, a blank
    /// weight is 1 and `(none)` is the empty suffix.
    #[test]
    fn a_minimal_file_parses_with_weights() {
        let st = parse_ok("# c\nName style,T\nSyllables,2,3\n[Settlements]\nStart,Start weight,End,End weight,Suffix,Suffix weight\nar,3,bel,,(none),2\nkor,,,,heim,\n");
        assert_eq!(st.name, "T");
        assert_eq!((st.min_syl, st.max_syl), (2, 3));
        let f = &st.sections["Settlements"].fragments;
        assert_eq!(f.start, vec![("ar".to_string(), 3.0), ("kor".to_string(), 1.0)]);
        assert_eq!(f.end, vec![("bel".to_string(), 1.0)]);
        assert_eq!(f.suffix, vec![(String::new(), 2.0), ("heim".to_string(), 1.0)]);
    }

    /// Protects: each kind of bad row is reported at its own 1-based line, is skipped whole, and the
    /// good rows around it still import (a literal list of expected lines).
    #[test]
    fn bad_rows_are_reported_by_line_and_skipped() {
        let t = "Name style,B\n\
Syllables,2,4\n\
[Settlements]\n\
Start,Start weight,End,End weight\n\
ok,,fine,\n\
bad,abc,fine2,\n\
neg,-2,x,\n\
w0,0,y,\n\
,3,z,\n\
{br},1,z,\n\
fine3,2,,\n\
[Rivers]\n\
Template,Weight\n\
River {name},2\n\
NoName,1\n\
Two {name} {name},1\n\
[Nowhere]\n\
a,b\n";
        let r = parse_style_csv(t);
        let lines: Vec<usize> = r.issues.iter().filter(|i| i.error).map(|i| i.line).collect();
        assert_eq!(lines, vec![6, 7, 8, 9, 10, 15, 16, 17, 18], "{:?}", r.issues);
        let st = r.style.unwrap();
        let f = &st.sections["Settlements"].fragments;
        // Rows 5 and 11 were kept; 6-10 were skipped whole (their End fragments did not leak in).
        assert_eq!(f.start.iter().map(|x| x.0.as_str()).collect::<Vec<_>>(), vec!["ok", "fine3"]);
        assert_eq!(f.end.iter().map(|x| x.0.as_str()).collect::<Vec<_>>(), vec!["fine"]);
        assert_eq!(st.sections["Rivers"].templates, vec![("River {name}".to_string(), 2.0)]);
    }

    /// Protects: a file with no `Name style` row, or no Settlements Start, is refused (never given a
    /// made-up name or pool), and a missing Syllables row is a note, not an error.
    #[test]
    fn a_file_that_cannot_work_is_refused_not_repaired() {
        assert!(parse_style_csv("[Settlements]\nStart\nar\n").style.is_none());
        assert!(parse_style_csv("Name style,X\n[Rivers]\nTemplate,Weight\n{name} R,1\n").style.is_none());
        let r = parse_style_csv("Name style,X\n[Settlements]\nStart\nar\n");
        assert!(r.style.is_some());
        assert!(r.issues.iter().any(|i| !i.error && i.message.contains("Syllables")));
    }

    /// Protects the weighted pick's boundary: with weights 1 : 1 the draw 0.5 is the second entry (`x < acc`, not
    /// `<=`), draws just under it are the first, and the draw nearest 1 is the last: a literal boundary, since a
    /// seeded stream never lands exactly on one.
    #[test]
    fn the_weighted_pick_boundary_is_exclusive() {
        let l: Weighted = vec![("a".into(), 1.0), ("b".into(), 1.0)];
        assert_eq!(pick(&l, 0.0), "a");
        assert_eq!(pick(&l, 0.4999999), "a");
        assert_eq!(pick(&l, 0.5), "b");
        assert_eq!(pick(&l, 0.9999999), "b");
        let w: Weighted = vec![("a".into(), 3.0), ("b".into(), 1.0)];
        assert_eq!(pick(&w, 0.75), "b", "3:1 puts the edge at 0.75");
    }

    /// Protects: quoting (a comma inside quotes, a doubled quote) and a BOM are read, an unclosed quote
    /// is a row error on its own line.
    #[test]
    fn csv_quoting_and_bom() {
        let r = parse_style_csv("\u{feff}Name style,\"A, B\"\n[Settlements]\nStart\n\"x\"\"y\"\n\"open\n");
        assert_eq!(r.style.as_ref().unwrap().name, "A, B");
        assert_eq!(r.style.unwrap().sections["Settlements"].fragments.start[0].0, "x\"y");
        assert_eq!(r.issues.iter().filter(|i| i.error).map(|i| i.line).collect::<Vec<_>>(), vec![5]);
    }

    /// Protects: the weights are a literal statistic. Fragments weighted 6 : 3 : 1 over 20 000 draws
    /// land within 3 percentage points of 60 / 30 / 10 % at a fixed seed.
    #[test]
    fn weights_are_the_draw_shares() {
        let st = parse_ok("Name style,W\nSyllables,1,1\n[Settlements]\nStart,Start weight,Suffix\na,6,(none)\nb,3,\nc,1,\n");
        let mut rng = Mulberry32::new(777);
        let mut n = [0usize; 3];
        for _ in 0..20_000 {
            match st.stem(&mut rng, "Settlements").as_str() {
                "A" => n[0] += 1,
                "B" => n[1] += 1,
                "C" => n[2] += 1,
                o => panic!("{o}"),
            }
        }
        let p: Vec<f64> = n.iter().map(|&c| c as f64 / 20_000.0).collect();
        assert!((p[0] - 0.6).abs() < 0.03 && (p[1] - 0.3).abs() < 0.03 && (p[2] - 0.1).abs() < 0.03, "{p:?}");
    }

    /// Protects: a style is seed-deterministic (same seed, same 50 names) and a different seed differs.
    #[test]
    fn a_style_is_deterministic() {
        let st = parse_ok(&export_culture(&CIV_CULTURES[2]));
        let run = |s| {
            let mut r = Mulberry32::new(s);
            (0..50).map(|_| st.stem(&mut r, "Settlements")).collect::<Vec<_>>()
        };
        assert_eq!(run(5), run(5));
        assert_ne!(run(5), run(6));
    }

    /// Protects Ruling FZ item 3: for every built-in culture, export then import yields a style that
    /// names **exactly** what the built-in culture names at one seed (settlements, bounded stems, and
    /// every decorated feature kind the engine names today), and the second export is byte-identical
    /// to the first (a literal fixed point).
    #[test]
    fn export_then_import_reproduces_every_builtin_culture() {
        for cul in CIV_CULTURES.iter() {
            let csv = export_culture(cul);
            let report = parse_style_csv(&csv);
            assert!(report.issues.is_empty(), "{}: {:?}", cul.key, report.issues);
            let st = report.style.unwrap();
            assert_eq!(export_style(&st), csv, "{}: the second export differs", cul.key);
            // Settlements.
            let (mut a, mut b) = (Mulberry32::new(4242), Mulberry32::new(4242));
            for i in 0..300 {
                assert_eq!(crate::civ_settle_name(&mut a, cul), st.settlement_name(&mut b), "{} settlement {i}", cul.key);
            }
            // Bounded, unique stems then decorate, per kind.
            for (section, kind) in [("Continents", FeatureKind::Continent), ("Provinces", FeatureKind::Province), ("Bays", FeatureKind::Bay), ("Mountains", FeatureKind::MountainRange), ("Lakes", FeatureKind::Lake)] {
                let (mut a, mut b) = (Mulberry32::new(99), Mulberry32::new(99));
                let (mut sa, mut sb) = (BTreeSet::new(), BTreeSet::new());
                for i in 0..120 {
                    let want = {
                        let stem = civ_settle_name_bounded_in(&mut a, cul, &mut sa);
                        decorate(&stem, kind, &mut a)
                    };
                    assert_eq!(want, st.feature_name(&mut b, section, &mut sb), "{} {section} {i}", cul.key);
                }
            }
        }
    }

    /// Protects: the exported file is a worked example (it names every section, carries the weight
    /// explanation, and writes `common`'s three empty suffixes as `(none)` with weight 3).
    #[test]
    fn the_export_is_a_worked_example() {
        let csv = export_culture(&CIV_CULTURES[0]);
        assert!(csv.starts_with("# Cartalith name style."));
        assert!(csv.contains("WEIGHTS:"));
        assert!(csv.contains("Name style,common\nSyllables,2,4\n[Settlements]\n"));
        assert!(csv.contains("(none),3"), "common has three empty suffixes");
        for def in SECTIONS {
            assert!(csv.contains(&format!("[{}]\n", def.key)), "missing [{}]", def.key);
        }
    }

    /// Protects: a section with no fragments borrows [Settlements]'s and one with no templates uses
    /// the built-in ones; an own template list wins.
    #[test]
    fn missing_sections_fall_back() {
        let st = parse_ok("Name style,F\nSyllables,1,1\n[Settlements]\nStart\nzz\n[Rivers]\nTemplate,Weight\nThe {name} Flow,1\n");
        let mut r = Mulberry32::new(1);
        let mut seen = BTreeSet::new();
        assert_eq!(st.feature_name(&mut r, "Rivers", &mut seen), "The Zz Flow");
        let mut r = Mulberry32::new(1);
        let n = st.feature_name(&mut r, "Cliffs", &mut BTreeSet::new());
        assert!(n.contains("Zz") && (n.ends_with("Cliffs") || n.ends_with("Scarp") || n.ends_with("Bluff")), "{n}");
    }

    /// Protects: `Max length` bounds names (and a style without it sizes itself), and uniqueness holds.
    #[test]
    fn max_length_and_uniqueness() {
        let st = parse_ok("Name style,M\nSyllables,1,1\nMax length,4\n[Settlements]\nStart\nab\nabcdefgh\n");
        let mut r = Mulberry32::new(3);
        let mut seen = BTreeSet::new();
        let names: Vec<String> = (0..4).map(|_| st.stem_bounded(&mut r, "Settlements", &mut seen)).collect();
        assert_eq!(seen.len(), 4);
        assert!(names.iter().all(|n| n.starts_with("Ab")), "{names:?}");
    }

    /// Protects: every pin kind maps to a section that exists, and the three examples in the ruling land
    /// in the sections it names.
    #[test]
    fn every_pin_kind_has_a_real_section() {
        for k in crate::poi::KIND_NAMES.iter() {
            assert!(section_def(section_for_kind(k)).is_some(), "{k}");
        }
        assert_eq!(section_for_kind("volcano"), "Mounts");
        assert_eq!(section_for_kind("cliff"), "Cliffs");
        assert_eq!(section_for_kind("shrine"), "Holy sites");
        assert_eq!(section_for_kind("custom-x-0123abcd"), "Landmarks");
        // Every landmark-run kind lands in a named section, never the catch-all.
        for k in crate::landmark::kinds() {
            assert!(section_def(section_for_kind(k.key)).is_some(), "{}", k.key);
            assert_ne!(section_for_kind(k.key), "Landmarks", "landmark kind {} has no section", k.key);
        }
    }

    /// Protects: a registered style is found by key, a built-in key is never intercepted, and
    /// `settle_name_for_key` over built-in keys equals `civ_settle_name` byte for byte.
    #[test]
    fn registry_routes_only_style_keys() {
        let key = register_style(parse_ok("Name style,Reg\nSyllables,1,1\n[Settlements]\nStart\nqq\n"));
        assert!(key.starts_with(STYLE_KEY_PREFIX));
        let mut r = Mulberry32::new(1);
        assert_eq!(settle_name_for_key(&mut r, &["x", key.as_str()], 1), "Qq");
        let (mut a, mut b) = (Mulberry32::new(8), Mulberry32::new(8));
        for f in 0..6 {
            assert_eq!(settle_name_for_key(&mut a, &["common", "highland"], f), crate::civ_settle_name(&mut b, crate::civ_faction_culture(&["common", "highland"], f)));
        }
    }
}
