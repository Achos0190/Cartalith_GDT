//! Grounded place facts (LLM-11, `LLM_ASSISTANT_SCOPE.md` "Grounded place prose").
//!
//! WHY: a model asked for prose about a settlement invents its population, its neighbours, its
//! gods and its trade unless it is handed the world's own figures. The owner's rule (2026-10-26):
//! prose for a place note must ALWAYS be written from the world's generated data for that place,
//! everything within 25 km of it, and its trade, religion and relations -- never from the model's
//! memory. This module turns what the engine's own getters returned (gathered by the shell,
//! `shell/llm_place_facts.gd`) into ONE structured JSON object and a compact text rendering of the
//! same facts, deterministic for the same world and place.
//!
//! ## What this module does and does not do
//!
//! * It is **pure**: it receives a `serde_json::Value` of getter results and returns data. It
//!   reads no engine, opens no file and no socket (the crate has no dependency that could).
//! * It **selects** (the 25 km radius, the neighbour cap, the lattice of terrain samples) and
//!   **renders**; it never computes a world figure of its own beyond distance and bearing from
//!   cell coordinates, and a percentage of a count it was handed.
//! * A figure the engine did not supply is **absent, with a reason** ([`not_available`] list and a
//!   `not available (reason)` line). It is never `0`, `none` or a plausible default
//!   (`MISTAKES.md`: never encode "no value" as a plausible value).
//! * Every cap is a named constant below and every truncation is **listed** ("truncated"), in the
//!   JSON, in the text and in the "this will send" summary.
//!
//! ## The shell contract (`raw`)
//!
//! `raw` is the object `llm_place_facts.gd` builds; keys it may carry (all optional except
//! `world`, `place_index` and `settlements`; an absent key becomes a `not available` line naming
//! the getter that returned nothing, supplied in `raw.unavailable`):
//! `world {map_width_km, gw, gh}`, `place_index`, `settlements [{i, tid, name, x, y, pop, kind,
//! faction, capital, coastal, religion}]`, `place {...}`, `neighbour_extra {index: {...}}`,
//! `factions`, `faction_economy`, `relations`, `military_factions`, `labels`, `place_cell`,
//! `samples`, `river`, `landmarks`, `unavailable`. See `shell/llm_place_facts.gd` for each.
//!
//! MUST NEVER: invent a value; send a map image (Ruling DI P5; this is text only); drop a
//! truncation without listing it; depend on a clock or a random source (determinism is a bar).

use serde_json::{Map, Value, json};

/// The radius around the place that "everything nearby" means. 25 km: the owner's figure
/// (2026-10-26). Inclusive: a settlement at exactly 25.0 km is inside.
pub const RADIUS_KM: f64 = 25.0;
/// Most neighbouring settlements listed. Labelled judgement: twelve one-line entries are about
/// 700 tokens; more crowds the 8 192-token window a small local model is usually run with. The
/// ones kept are the largest by population (ties: nearer, then lower id), shown nearest first.
pub const MAX_NEIGHBOURS: usize = 12;
/// Names of omitted neighbours listed, so the model and the user can see what was left out.
pub const MAX_OMITTED_NAMES: usize = 20;
/// Most landmarks listed. Labelled judgement.
pub const MAX_LANDMARKS: usize = 8;
/// Most trade flows listed for the place. Labelled judgement.
pub const MAX_FLOWS: usize = 8;
/// Most conflicts listed. Labelled judgement.
pub const MAX_CONFLICTS: usize = 5;
/// Most journeys listed. Labelled judgement.
pub const MAX_JOURNEYS: usize = 5;
/// Most faction relations listed (the place's faction against others). Labelled judgement.
pub const MAX_RELATIONS: usize = 8;
/// Most volcano groups listed. Labelled judgement.
pub const MAX_VOLCANOES: usize = 5;
/// Most biomes listed in the survey. Labelled judgement.
pub const MAX_BIOMES: usize = 6;
/// Most characters of any user-written free text (a place's history field, a custom religion's
/// notes) copied into the facts. Labelled judgement: a short paragraph.
pub const MAX_USER_TEXT_CHARS: usize = 600;
/// The most characters of WORLD DATA text. Labelled judgement: about 2 400 tokens. If the sections
/// together exceed it, the lowest-priority sections are replaced by one line naming what was
/// dropped (the order is the order of [`SECTION_ORDER`]).
pub const MAX_WORLD_CHARS: usize = 9_000;
/// Spacing of the terrain-survey lattice, km. Labelled judgement: 2.5 km gives about 350 samples in
/// the disc, which is cheap to read and fine enough to see a ridge, a coast or a river of order 3+;
/// features smaller than the spacing can be missed, and the text says so.
pub const LATTICE_STEP_KM: f64 = 2.5;
/// The most lattice points ever requested (a guard against a pathologically fine grid).
pub const MAX_SAMPLES: usize = 600;

/// The order sections are rendered in, which is also the order they are dropped from the end when
/// the size cap bites. `(id, label)`.
pub const SECTION_ORDER: [(&str, &str); 11] = [
    ("place", "The place"),
    ("terrain", "Its ground and climate"),
    ("faction", "Its faction and government"),
    ("religion", "Religion"),
    ("relations", "Relations of its faction"),
    ("economy", "Economy and trade"),
    ("defence", "Defence"),
    ("history", "History in the world's record"),
    ("neighbours", "Settlements within 25 km"),
    ("surroundings", "The land within 25 km"),
    ("landmarks", "Landmarks within 25 km"),
];

// ----------------------------------------------------------------------------------------
// Small readers over `serde_json::Value`. An absent key, a null and a wrong type all read as
// absent: the shell omits what the engine did not supply, and this never turns absence into 0.
// ----------------------------------------------------------------------------------------

fn st<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v.get(k).and_then(Value::as_str).filter(|s| !s.trim().is_empty())
}
fn fl(v: &Value, k: &str) -> Option<f64> {
    v.get(k).and_then(Value::as_f64).filter(|x| x.is_finite())
}
fn it(v: &Value, k: &str) -> Option<i64> {
    v.get(k).and_then(|x| x.as_i64().or_else(|| x.as_f64().filter(|f| f.is_finite()).map(|f| f.round() as i64)))
}
fn bl(v: &Value, k: &str) -> Option<bool> {
    v.get(k).and_then(Value::as_bool)
}
fn arr<'a>(v: &'a Value, k: &str) -> &'a [Value] {
    v.get(k).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[])
}
fn strs(v: &Value, k: &str) -> Vec<String> {
    arr(v, k).iter().filter_map(|x| x.as_str()).map(str::to_owned).collect()
}

/// `x` to `dp` decimals, no trailing-zero games: the same text every run.
fn f1(x: f64) -> String {
    format!("{x:.1}")
}
fn f0(x: f64) -> String {
    format!("{x:.0}")
}
/// A percentage of a 0..1 share, whole number.
fn pct(share: f64) -> String {
    format!("{}%", (share * 100.0).round() as i64)
}

/// Cut `text` to `max` characters at a character boundary; the flag says it was cut.
pub fn clip_chars(text: &str, max: usize) -> (String, bool) {
    let n = text.chars().count();
    if n <= max {
        (text.to_owned(), false)
    } else {
        (text.chars().take(max).collect::<String>() + "...", true)
    }
}

/// Collapse a user-written free text to one line so it cannot add structure of its own.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ----------------------------------------------------------------------------------------
// Geometry: cell size, distance, bearing, the lattice
// ----------------------------------------------------------------------------------------

/// One cell's side in km (`map_width_km / gw`), or `None` when the world has no declared width
/// (then no distance can be stated honestly and nothing is invented).
pub fn cell_km(world: &Value) -> Option<f64> {
    let w = fl(world, "map_width_km")?;
    let gw = fl(world, "gw")?;
    (w > 0.0 && gw > 0.0).then(|| w / gw)
}

/// Eight-point compass word for the vector `(dx, dy)` in cells, north being UP the map
/// (`-y`, as the map is drawn). `(0,0)` has no bearing and answers `""`.
pub fn bearing(dx: f64, dy: f64) -> &'static str {
    if dx == 0.0 && dy == 0.0 {
        return "";
    }
    // Degrees clockwise from north.
    let deg = dx.atan2(-dy).to_degrees().rem_euclid(360.0);
    const NAMES: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    NAMES[(((deg + 22.5) / 45.0).floor() as usize) % 8]
}

/// Distance in km between two cell positions at `cell_km` per cell.
pub fn distance_km(ax: f64, ay: f64, bx: f64, by: f64, cell_km: f64) -> f64 {
    (bx - ax).hypot(by - ay) * cell_km
}

/// Signed displacement `(dx, dy)` in cells from `a` to `b`. On a map that wraps east-west
/// (`world.wrap_x`, the "world" parameter) the shorter way round the seam is taken, so a town just
/// across the seam is a near neighbour and not 400 km away.
pub fn delta(world: &Value, ax: f64, ay: f64, bx: f64, by: f64) -> (f64, f64) {
    let mut dx = bx - ax;
    if bl(world, "wrap_x") == Some(true)
        && let Some(gw) = fl(world, "gw").filter(|g| *g > 0.0)
    {
        if dx > gw / 2.0 {
            dx -= gw;
        } else if dx < -gw / 2.0 {
            dx += gw;
        }
    }
    (dx, by - ay)
}

/// The lattice of cells the shell reads with `sample_cell`, anchored on the place so the same
/// place gives the same points. Row-major, inside the grid and inside [`RADIUS_KM`].
/// `step = max(1, round(LATTICE_STEP_KM / cell_km))` cells. At most [`MAX_SAMPLES`] points.
pub fn sample_points(cx: i64, cy: i64, cell_km: f64, gw: i64, gh: i64, wrap_x: bool) -> Vec<(i64, i64)> {
    if !(cell_km.is_finite() && cell_km > 0.0) || gw <= 0 || gh <= 0 {
        return Vec::new();
    }
    let step = ((LATTICE_STEP_KM / cell_km).round() as i64).max(1);
    let n = (RADIUS_KM / (step as f64 * cell_km)).ceil() as i64;
    let mut out = Vec::new();
    for j in -n..=n {
        for i in -n..=n {
            let (mut x, y) = (cx + i * step, cy + j * step);
            if wrap_x {
                x = x.rem_euclid(gw);
            }
            if x < 0 || y < 0 || x >= gw || y >= gh {
                continue;
            }
            let d = ((i * step) as f64).hypot((j * step) as f64) * cell_km;
            if d <= RADIUS_KM && out.len() < MAX_SAMPLES {
                out.push((x, y));
            }
        }
    }
    out
}

/// One settlement near the place.
#[derive(Debug, Clone, PartialEq)]
pub struct Neighbour {
    /// Index into the settlement list (what the shell passes to `civ_settlement_details`).
    pub index: usize,
    /// Stable id.
    pub tid: i64,
    /// Display name.
    pub name: String,
    /// Distance from the place, km.
    pub dist_km: f64,
    /// Compass word.
    pub bearing: &'static str,
    /// Population (the ranking key for the cap).
    pub pop: i64,
}

/// The neighbours inside the radius.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NeighbourSelection {
    /// Every settlement within [`RADIUS_KM`] (the place itself excluded).
    pub within: usize,
    /// The ones kept (at most [`MAX_NEIGHBOURS`]), nearest first.
    pub kept: Vec<Neighbour>,
    /// The rest inside the radius, nearest first.
    pub omitted: Vec<Neighbour>,
    /// Why nothing could be computed (no declared map width, place not found); `None` when it ran.
    pub unavailable: Option<String>,
}

/// Select the neighbours: every settlement within [`RADIUS_KM`] of the place (inclusive), keep the
/// [`MAX_NEIGHBOURS`] largest (ties: nearer, then lower id), return them nearest first.
pub fn select_neighbours(raw: &Value) -> NeighbourSelection {
    let world = raw.get("world").cloned().unwrap_or(Value::Null);
    let Some(ck) = cell_km(&world) else {
        return NeighbourSelection { unavailable: Some("the world declares no map width in km, so no distance can be stated".into()), ..Default::default() };
    };
    let list = arr(raw, "settlements");
    let Some(pi) = it(raw, "place_index").filter(|&i| i >= 0 && (i as usize) < list.len()) else {
        return NeighbourSelection { unavailable: Some("the place is not in the settlement list".into()), ..Default::default() };
    };
    let p = &list[pi as usize];
    let (px, py) = (fl(p, "x").unwrap_or(0.0), fl(p, "y").unwrap_or(0.0));
    let mut all: Vec<Neighbour> = list
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != pi as usize)
        .filter_map(|(i, s)| {
            let (x, y) = (fl(s, "x")?, fl(s, "y")?);
            let (dx, dy) = delta(&world, px, py, x, y);
            let d = dx.hypot(dy) * ck;
            (d <= RADIUS_KM).then(|| Neighbour {
                index: i,
                tid: it(s, "tid").unwrap_or(i as i64),
                name: st(s, "name").unwrap_or("(unnamed)").to_owned(),
                dist_km: d,
                bearing: bearing(dx, dy),
                pop: it(s, "pop").unwrap_or(0),
            })
        })
        .collect();
    let within = all.len();
    // Retention order: largest first, then nearer, then lower id. Total order, so deterministic.
    all.sort_by(|a, b| {
        b.pop
            .cmp(&a.pop)
            .then(a.dist_km.partial_cmp(&b.dist_km).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.tid.cmp(&b.tid))
    });
    let omitted_part = if all.len() > MAX_NEIGHBOURS { all.split_off(MAX_NEIGHBOURS) } else { Vec::new() };
    let by_distance = |a: &Neighbour, b: &Neighbour| {
        a.dist_km.partial_cmp(&b.dist_km).unwrap_or(std::cmp::Ordering::Equal).then(a.tid.cmp(&b.tid))
    };
    let mut kept = all;
    kept.sort_by(by_distance);
    let mut omitted = omitted_part;
    omitted.sort_by(by_distance);
    NeighbourSelection { within, kept, omitted, unavailable: None }
}

// ----------------------------------------------------------------------------------------
// The builder
// ----------------------------------------------------------------------------------------

/// One rendered section: its structured JSON and its text lines. `count` is the number of fact
/// lines (what the "this will send" summary reports).
#[derive(Debug, Clone, Default)]
struct Section {
    json: Map<String, Value>,
    lines: Vec<String>,
}

impl Section {
    /// A fact with a structured value and its text. Text line: `label: text`.
    fn fact(&mut self, key: &str, label: &str, value: Value, text: impl Into<String>) {
        self.json.insert(key.to_owned(), value);
        self.lines.push(format!("{label}: {}", text.into()));
    }
    /// A line with no structured twin of its own (a list entry whose structure is stored by the caller).
    fn line(&mut self, text: impl Into<String>) {
        self.lines.push(text.into());
    }
    fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

/// Collected gaps and truncations.
#[derive(Debug, Default)]
struct Notes {
    /// `(item, reason)`.
    not_available: Vec<(String, String)>,
    /// Plain sentences, each saying what was cut and how much.
    truncated: Vec<String>,
}

impl Notes {
    fn na(&mut self, item: &str, reason: impl Into<String>) {
        self.not_available.push((item.to_owned(), reason.into()));
    }
    fn cut(&mut self, what: impl Into<String>) {
        self.truncated.push(what.into());
    }
}

/// What the "this will send" line and the card list: the sections sent with their line counts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FactsSummary {
    /// `(label, lines)` per section actually sent, in order.
    pub sections: Vec<(String, usize)>,
    /// Characters of WORLD DATA text.
    pub chars: usize,
    /// What was truncated or dropped, as sentences.
    pub truncated: Vec<String>,
    /// `(item, reason)` for everything the engine did not expose.
    pub not_available: Vec<(String, String)>,
    /// Neighbours listed / within the radius.
    pub neighbours_listed: usize,
    /// Neighbours within the radius (listed or not).
    pub neighbours_within: usize,
}

/// The finished facts.
#[derive(Debug, Clone)]
pub struct PlaceFacts {
    /// The structured object (`schema`, `place`, `world_data.<section>`, `not_available`, `truncated`).
    pub json: Value,
    /// The compact text rendering of the same facts (the WORLD DATA block).
    pub text: String,
    /// What was sent.
    pub summary: FactsSummary,
    /// Name of the place (for the context attribute).
    pub place_name: String,
}

fn label_of(raw: &Value, group: &str, key: &str) -> String {
    raw.get("labels").and_then(|l| l.get(group)).and_then(|g| g.get(key)).and_then(Value::as_str).unwrap_or(key).to_owned()
}

fn faction_name(raw: &Value, id: i64) -> Option<String> {
    arr(raw, "factions").iter().find(|f| it(f, "id") == Some(id)).and_then(|f| st(f, "name")).map(str::to_owned)
}

/// Build the facts for the place named in `raw`. Never fails: a place that cannot be found yields
/// facts with a single `not available` line saying so.
pub fn build(raw: &Value) -> PlaceFacts {
    let mut notes = Notes::default();
    // The shell reports getters that returned nothing; carry those through verbatim.
    if let Some(u) = raw.get("unavailable").and_then(Value::as_object) {
        let mut keys: Vec<&String> = u.keys().collect();
        keys.sort();
        for k in keys {
            if let Some(r) = u[k].as_str() {
                notes.na(k, r);
            }
        }
    }
    let world = raw.get("world").cloned().unwrap_or(Value::Null);
    let ck = cell_km(&world);
    let list = arr(raw, "settlements");
    let pi = it(raw, "place_index").filter(|&i| i >= 0 && (i as usize) < list.len());
    let Some(pi) = pi else {
        notes.na("place", "the place is not in the settlement list");
        return finish(raw, "(unknown place)".into(), Vec::new(), notes, 0, 0, Value::Null);
    };
    let p = &list[pi as usize];
    let place_name = st(p, "name").unwrap_or("(unnamed)").to_owned();
    let extra = raw.get("place").cloned().unwrap_or(Value::Null);
    let fid = it(p, "faction").unwrap_or(0);
    let mut sections: Vec<(&'static str, Section)> = Vec::new();

    // ---- place ----
    let mut s = Section::default();
    s.fact("name", "name", json!(place_name), place_name.clone());
    if let Some(t) = it(p, "tid") {
        s.fact("id", "id", json!(t), t.to_string());
    }
    if let Some(k) = st(p, "kind") {
        s.fact("size_class", "size class", json!(k), k);
    }
    if let Some(pop) = it(p, "pop") {
        s.fact("population", "population", json!(pop), pop.to_string());
    }
    if let Some(c) = bl(p, "capital") {
        s.fact("capital", "capital", json!(c), if c { "yes" } else { "no" });
    }
    if let Some(c) = bl(p, "coastal") {
        s.fact("coastal", "coastal", json!(c), if c { "yes" } else { "no" });
    }
    if let (Some(x), Some(y)) = (fl(p, "x"), fl(p, "y")) {
        s.fact("cell", "position (cell x, y)", json!([x, y]), format!("{}, {}", f0(x), f0(y)));
        if let Some(ck) = ck {
            let (kx, ky) = (x * ck, y * ck);
            s.fact("km_from_west_north", "position (km from the map's west edge, north edge)", json!([kx, ky]), format!("{}, {}", f1(kx), f1(ky)));
        }
    }
    if let Some(ck) = ck {
        let (w, h) = (fl(&world, "map_width_km").unwrap_or(0.0), fl(&world, "gh").map(|g| g * ck).unwrap_or(0.0));
        s.fact("map_km", "map size", json!([w, h]), format!("{} km wide, {} km high", f0(w), f0(h)));
    } else {
        notes.na("distances", "the world declares no map width in km");
    }
    if let Some(sp) = st(&extra, "specialisation").filter(|v| *v != "none") {
        s.fact("specialisation", "specialisation", json!(sp), label_of(raw, "specialisation", sp));
    } else if st(&extra, "specialisation").is_some() {
        s.fact("specialisation", "specialisation", json!("none"), "none set");
    }
    let traits = strs(&extra, "traits");
    if !traits.is_empty() {
        let labels: Vec<String> = traits.iter().map(|t| label_of(raw, "traits", t)).collect();
        s.fact("traits", "traits", json!(traits), labels.join(", "));
    }
    match (it(&extra, "age"), it(&extra, "age_inferred")) {
        (Some(a), _) if a >= 0 => s.fact("age_years", "age", json!(a), format!("{a} years (set by the user)")),
        (_, Some(a)) => s.fact("age_years", "age", json!(a), format!("about {a} years (inferred from its size)")),
        _ => notes.na("age", "no age was supplied for this place"),
    }
    if let Some(h) = st(&extra, "history") {
        let (t, cut) = clip_chars(&one_line(h), MAX_USER_TEXT_CHARS);
        if cut {
            notes.cut(format!("the place's history text was cut to {MAX_USER_TEXT_CHARS} characters"));
        }
        s.fact("history_note", "history written by the user in the place editor", json!(t), t.clone());
    }
    sections.push(("place", s));

    // ---- terrain (own cell) ----
    let mut s = Section::default();
    if let Some(c) = raw.get("place_cell").filter(|c| c.is_object() && !c.as_object().unwrap().is_empty()) {
        terrain_lines(&mut s, c);
    } else {
        notes.na("terrain at the place", "the engine returned no sample for the place's cell");
    }
    // Rivers/coast flags from the engine's own settlement score decomposition are not used: the
    // lattice survey below states the surroundings.
    sections.push(("terrain", s));

    // ---- faction and government ----
    let mut s = Section::default();
    let faction_row = arr(raw, "factions").iter().find(|f| it(f, "id") == Some(fid));
    if fid <= 0 {
        s.line("faction: none (the settlement belongs to no faction)");
        s.json.insert("faction".into(), Value::Null);
    } else if let Some(f) = faction_row {
        let nm = st(f, "name").unwrap_or("(unnamed)");
        s.fact("name", "faction", json!(nm), nm);
        s.json.insert("id".into(), json!(fid));
        for (k, label) in [("culture", "culture"), ("government", "government"), ("ag_tech", "agricultural technology")] {
            if let Some(v) = st(f, k) {
                let shown = if k == "government" { label_of(raw, "government", v) } else { v.to_owned() };
                s.fact(k, label, json!(v), shown);
            }
        }
        if let Some(v) = st(f, "religious_policy") {
            s.fact("religious_policy", "religious policy", json!(v), label_of(raw, "policy", v));
        }
        if let Some(n) = it(f, "settlement_count") {
            s.fact("settlement_count", "settlements held", json!(n), n.to_string());
        }
        if let Some(n) = it(f, "population") {
            s.fact("population", "faction population", json!(n), n.to_string());
        }
        if let (Some(cells), Some(ck)) = (fl(f, "claimed_cells"), ck) {
            let km2 = cells * ck * ck;
            s.fact("territory_km2", "territory", json!(km2), format!("{} km2", f0(km2)));
        } else {
            notes.na("faction territory", "the claim grid is not known for this world");
        }
    } else {
        notes.na("faction", format!("faction {fid} is not in the roster the engine returned"));
    }
    let own = arr(&extra, "ownership");
    if !own.is_empty() {
        let mut parts = Vec::new();
        for o in own {
            let who = it(o, "faction_id").and_then(|id| faction_name(raw, id)).unwrap_or_else(|| "a faction".into());
            let end = if bl(o, "current") == Some(true) { "present".to_owned() } else { it(o, "end_year").map(|y| y.to_string()).unwrap_or_else(|| "?".into()) };
            parts.push(format!("{} to {end}: {who}", it(o, "start_year").map(|y| y.to_string()).unwrap_or_else(|| "?".into())));
        }
        s.fact("ownership", "held by (year ranges)", json!(own), parts.join("; "));
    }
    sections.push(("faction", s));

    // ---- religion ----
    let mut s = Section::default();
    let mut rel_ok = false;
    if let Some(r) = st(p, "religion") {
        rel_ok = true;
        s.fact("plurality", "main faith here", json!(r), label_of(raw, "religion", r));
        if let Some(sh) = extra.get("religion_shares").and_then(Value::as_object) {
            let mut v: Vec<(&String, f64)> = sh.iter().filter_map(|(k, x)| x.as_f64().map(|x| (k, x))).filter(|(_, x)| *x >= 0.005).collect();
            v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(b.0)));
            v.truncate(5);
            if !v.is_empty() {
                let txt: Vec<String> = v.iter().map(|(k, x)| format!("{} {}", label_of(raw, "religion", k), pct(*x))).collect();
                s.fact("shares", "faiths here (share of people)", json!(v.iter().map(|(k, x)| json!({"faith": k, "share": x})).collect::<Vec<_>>()), txt.join(", "));
            }
        }
    } else {
        notes.na("faith of the place", "the belief model has not run for this world (no religion key on the settlement)");
    }
    if let Some(f) = faction_row {
        if let Some(r) = st(f, "religion") {
            rel_ok = true;
            let name = st(f, "religion_name").map(str::to_owned);
            let shown = name.clone().unwrap_or_else(|| label_of(raw, "religion", r));
            s.fact("ruler_faith", "faith of the ruling faction", json!(r), shown);
            if let Some(n) = name {
                s.json.insert("ruler_faith_custom_name".into(), json!(n));
                s.line(format!("(that is the user's custom religion, which behaves as {})", label_of(raw, "religion", r)));
            }
        }
        // The shell resolves the ruling faction's CUSTOM religion row (with its beliefs already
        // labelled) into `custom_religion`; absent when the faction uses a built-in faith.
        if let Some(cr) = raw.get("custom_religion").filter(|c| c.is_object()) {
            if let Some(b) = st(cr, "base_label") {
                s.line(format!("custom religion base: {b}"));
            }
            let axes = arr(cr, "axes");
            if !axes.is_empty() {
                let parts: Vec<String> = axes
                    .iter()
                    .filter_map(|a| Some(format!("{}: {}", st(a, "label")?, st(a, "value_label")?)))
                    .collect();
                if !parts.is_empty() {
                    s.fact("custom_religion_beliefs", "custom religion's own beliefs", json!(axes), parts.join("; "));
                }
            }
            if let Some(n) = st(cr, "notes") {
                let (t, cut) = clip_chars(&one_line(n), MAX_USER_TEXT_CHARS);
                if cut {
                    notes.cut(format!("the custom religion's notes were cut to {MAX_USER_TEXT_CHARS} characters"));
                }
                s.fact("custom_religion_notes", "notes written by the user on that religion", json!(t), t.clone());
            }
        }
    }
    if !rel_ok {
        // Already noted above; keep the section honest rather than empty-silent.
        s.line("religion: not available (see the not-available list)");
    }
    notes.na("worship sites and local rites", "the engine models faith shares per settlement but has no worship-site or rite entities");
    sections.push(("religion", s));

    // ---- relations ----
    let mut s = Section::default();
    if fid > 0 && !arr(raw, "relations").is_empty() {
        let mut rows: Vec<&Value> = arr(raw, "relations").iter().filter(|r| it(r, "a") == Some(fid) || it(r, "b") == Some(fid)).collect();
        // Most bordered first, then by the other faction's id: a total order.
        let other = |r: &Value| if it(r, "a") == Some(fid) { it(r, "b").unwrap_or(0) } else { it(r, "a").unwrap_or(0) };
        rows.sort_by(|x, y| {
            fl(y, "border_cells").unwrap_or(-1.0).partial_cmp(&fl(x, "border_cells").unwrap_or(-1.0)).unwrap_or(std::cmp::Ordering::Equal).then(other(x).cmp(&other(y)))
        });
        let total = rows.len();
        rows.truncate(MAX_RELATIONS);
        if total > rows.len() {
            notes.cut(format!("relations: {} of {} faction pairs listed (most bordered first)", rows.len(), total));
        }
        let mut list = Vec::new();
        for r in rows {
            let o = other(r);
            let oname = if it(r, "a") == Some(fid) { st(r, "b_name") } else { st(r, "a_name") }.map(str::to_owned).or_else(|| faction_name(raw, o)).unwrap_or_else(|| format!("faction {o}"));
            let mut txt = format!("{oname}: ");
            if let Some(stn) = st(r, "stance") {
                txt += stn;
                if let Some(v) = fl(r, "value") {
                    txt += &format!(" (value {})", format!("{v:.2}"));
                }
                if let Some(b) = fl(r, "border_fraction") {
                    txt += &format!(", {} of its border shared", pct(b));
                }
            } else {
                txt += "stance not available (the claim grid is not known)";
            }
            s.line(format!("- {txt}"));
            list.push(json!({"other": oname, "stance": st(r, "stance"), "value": fl(r, "value"), "border_fraction": fl(r, "border_fraction")}));
        }
        s.json.insert("pairs".into(), Value::Array(list));
    } else {
        notes.na("relations", "the engine returned no faction relations (no world, no faction, or no claim grid)");
    }
    notes.na("alliances, tribute and treaties as records", "relations are derived stances only; the engine stores no treaty or tribute");
    sections.push(("relations", s));

    // ---- economy and trade ----
    let mut s = Section::default();
    if let Some(e) = arr(raw, "faction_economy").iter().find(|e| it(e, "faction") == Some(fid)) {
        if st(e, "absent").is_some() {
            notes.na("faction economy", "the claim grid is not known, so territory-integrated economy cannot be computed");
        } else {
            if let Some(v) = fl(e, "food_surplus") {
                s.fact("food_surplus", "faction food surplus (people fed beyond need)", json!(v), f0(v));
            }
            if let Some(v) = fl(e, "food_capacity") {
                s.fact("food_capacity", "faction food production capacity", json!(v), f0(v));
            }
            let (ex, im, stg) = (strs(e, "exports"), strs(e, "imports"), strs(e, "strategic"));
            if !ex.is_empty() {
                s.fact("faction_exports", "faction exports", json!(ex), ex.join(", "));
            }
            if !im.is_empty() {
                s.fact("faction_imports", "faction imports", json!(im), im.join(", "));
            }
            if !stg.is_empty() {
                s.fact("strategic", "strategic resources", json!(stg), stg.join(", "));
            }
        }
    } else {
        notes.na("faction economy", "the engine returned no economy row for this faction");
    }
    if let Some(t) = extra.get("trade") {
        let (ex, im) = (strs(t, "exports"), strs(t, "imports"));
        s.fact("exports", "this settlement's resource surplus (exports)", json!(ex), if ex.is_empty() { "none".to_owned() } else { ex.join(", ") });
        s.fact("imports", "this settlement's resource deficit (imports)", json!(im), if im.is_empty() { "none".to_owned() } else { im.join(", ") });
    } else {
        notes.na("settlement trade balance", "the engine returned no trade balance for this settlement");
    }
    let flows = arr(&extra, "flows");
    if !flows.is_empty() {
        let total = it(&extra, "flows_total").unwrap_or(flows.len() as i64);
        let mut v: Vec<&Value> = flows.iter().collect();
        v.truncate(MAX_FLOWS);
        if total as usize > v.len() {
            notes.cut(format!("trade flows: {} of {} listed (largest first)", v.len(), total));
        }
        let mut list = Vec::new();
        for f in v {
            let dir = if st(f, "direction") == Some("out") { "sends" } else { "receives" };
            let good = st(f, "good").unwrap_or("goods");
            let other = st(f, "other").unwrap_or("another settlement");
            let mode = st(f, "mode").unwrap_or("unknown route");
            let vol = fl(f, "volume").map(f1).unwrap_or_else(|| "?".into());
            s.line(format!("- {dir} {good} {} {other} by {mode} (volume {vol})", if dir == "sends" { "to" } else { "from" }));
            list.push(f.clone());
        }
        s.json.insert("flows".into(), Value::Array(list));
    } else if raw.get("unavailable").and_then(|u| u.get("trade flows")).is_none() {
        s.line("trade flows: none matched for this settlement in the engine's trade network");
    }
    notes.na("per-settlement production and consumption", "the engine exposes a resource surplus/deficit list and matched flows, not production or consumption figures per settlement");
    sections.push(("economy", s));

    // ---- defence ----
    let mut s = Section::default();
    if let Some(m) = extra.get("military").filter(|m| m.is_object()) {
        if let Some(w) = st(m, "wall_spec") {
            s.fact("wall_spec", "walls", json!(w), w);
        }
        if let Some(d) = fl(m, "defensibility") {
            s.fact("defensibility", "defensibility (0 to 1)", json!(d), format!("{d:.2}"));
        }
        if let Some(g) = fl(m, "garrison") {
            s.fact("garrison", "garrison", json!(g), f0(g));
        }
        if let Some(g) = fl(m, "border_exposure") {
            s.fact("border_exposure", "border exposure", json!(g), format!("{g:.2}"));
        }
    } else {
        notes.na("defences of the place", "the engine returned no military row for this settlement");
    }
    if let Some(m) = raw.get("military_factions").and_then(Value::as_array).and_then(|a| a.iter().find(|f| it(f, "faction") == Some(fid))) {
        if let Some(v) = fl(m, "military") {
            s.fact("faction_military", "faction military strength (0 to 100)", json!(v), f0(v));
        }
        if let Some(mp) = m.get("manpower").filter(|x| x.is_object()) {
            let mut parts = Vec::new();
            for (k, l) in [("standing_army", "standing army"), ("field_army", "field army"), ("emergency_mobilization", "emergency levy")] {
                if let Some(v) = fl(mp, k) {
                    parts.push(format!("{l} {}", f0(v)));
                }
            }
            if !parts.is_empty() {
                s.fact("faction_manpower", "faction forces", json!(mp), parts.join(", "));
            }
        }
    }
    sections.push(("defence", s));

    // ---- history ----
    let mut s = Section::default();
    let conflicts = arr(&extra, "conflicts");
    if !conflicts.is_empty() {
        let total = conflicts.len();
        let mut v: Vec<&Value> = conflicts.iter().collect();
        v.truncate(MAX_CONFLICTS);
        if total > v.len() {
            notes.cut(format!("conflicts: {} of {} listed (earliest first)", v.len(), total));
        }
        for c in v {
            let name = st(c, "name").unwrap_or("(unnamed conflict)");
            let kind = st(c, "kind").unwrap_or("conflict");
            let span = match (it(c, "start_year"), it(c, "end_year")) {
                (Some(a), Some(b)) => format!("{a} to {b}"),
                (Some(a), None) => format!("from {a}"),
                _ => "undated".into(),
            };
            let sides = strs(c, "sides_names");
            let sides_txt = if sides.is_empty() { String::new() } else { format!("; sides: {}", sides.join(" against ")) };
            let outcome = st(c, "outcome").map(|o| format!("; outcome: {o}")).unwrap_or_default();
            s.line(format!("- {name} ({kind}, {span}{sides_txt}{outcome})"));
        }
        s.json.insert("conflicts".into(), Value::Array(conflicts.iter().take(MAX_CONFLICTS).cloned().collect()));
    }
    let journeys = arr(&extra, "journeys");
    if !journeys.is_empty() {
        let total = journeys.len();
        let mut v: Vec<&Value> = journeys.iter().collect();
        v.truncate(MAX_JOURNEYS);
        if total > v.len() {
            notes.cut(format!("journeys: {} of {} listed", v.len(), total));
        }
        for j in v {
            let name = st(j, "name").unwrap_or("(unnamed journey)");
            let when = st(j, "date").map(|d| format!(" on {d}")).unwrap_or_default();
            s.line(format!("- the journey \"{name}\" passes here{when}"));
        }
        s.json.insert("journeys".into(), Value::Array(journeys.iter().take(MAX_JOURNEYS).cloned().collect()));
    }
    if s.is_empty() {
        s.line("no conflicts or saved journeys touch this place in the world's record");
    }
    sections.push(("history", s));

    // ---- neighbours ----
    let sel = select_neighbours(raw);
    let mut s = Section::default();
    if let Some(why) = &sel.unavailable {
        notes.na("neighbouring settlements", why.clone());
    } else {
        let nx = raw.get("neighbour_extra").cloned().unwrap_or(Value::Null);
        s.json.insert("radius_km".into(), json!(RADIUS_KM));
        s.json.insert("within_radius".into(), json!(sel.within));
        s.line(format!("{} settlements lie within {} km; {} listed", sel.within, f0(RADIUS_KM), sel.kept.len()));
        let mut jl = Vec::new();
        for n in &sel.kept {
            let sv = &list[n.index];
            let ex = nx.get(n.index.to_string()).cloned().unwrap_or(Value::Null);
            let mut bits: Vec<String> = Vec::new();
            if let Some(k) = st(sv, "kind") {
                bits.push(k.to_owned());
            }
            bits.push(format!("pop {}", n.pop));
            if let Some(fi) = it(sv, "faction") {
                bits.push(if fi <= 0 { "no faction".into() } else { faction_name(raw, fi).map(|n| format!("faction {n}")).unwrap_or_else(|| format!("faction {fi}")) });
            }
            bits.push(format!("{} km {}", f1(n.dist_km), n.bearing));
            if bl(sv, "capital") == Some(true) {
                bits.push("capital".into());
            }
            if bl(sv, "coastal") == Some(true) {
                bits.push("coastal".into());
            }
            if let Some(r) = st(sv, "religion") {
                bits.push(format!("faith {}", label_of(raw, "religion", r)));
            }
            if let Some(sp) = st(&ex, "specialisation").filter(|v| *v != "none") {
                bits.push(format!("specialisation {}", label_of(raw, "specialisation", sp)));
            }
            if let Some(t) = ex.get("trade") {
                let (e, i) = (strs(t, "exports"), strs(t, "imports"));
                if !e.is_empty() {
                    bits.push(format!("exports {}", e.join("/")));
                }
                if !i.is_empty() {
                    bits.push(format!("imports {}", i.join("/")));
                }
            }
            if let Some(w) = ex.get("military").and_then(|m| st(m, "wall_spec")) {
                bits.push(format!("walls {w}"));
            }
            s.line(format!("- {} ({})", n.name, bits.join("; ")));
            jl.push(json!({"name": n.name, "id": n.tid, "distance_km": n.dist_km, "bearing": n.bearing, "population": n.pop,
                           "kind": st(sv, "kind"), "faction": it(sv, "faction"), "capital": bl(sv, "capital"), "coastal": bl(sv, "coastal"),
                           "religion": st(sv, "religion"), "specialisation": st(&ex, "specialisation"), "trade": ex.get("trade"), "walls": ex.get("military").and_then(|m| st(m, "wall_spec"))}));
        }
        s.json.insert("listed".into(), Value::Array(jl));
        if !sel.omitted.is_empty() {
            let names: Vec<String> = sel.omitted.iter().take(MAX_OMITTED_NAMES).map(|n| format!("{} ({} km {}, pop {})", n.name, f1(n.dist_km), n.bearing, n.pop)).collect();
            let more = sel.omitted.len().saturating_sub(MAX_OMITTED_NAMES);
            let tail = if more > 0 { format!("; and {more} more") } else { String::new() };
            let msg = format!("neighbours: {} of {} within {} km are not listed (the {} largest are): {}{}", sel.omitted.len(), sel.within, f0(RADIUS_KM), MAX_NEIGHBOURS, names.join("; "), tail);
            notes.cut(msg.clone());
            s.line(msg);
        }
    }
    sections.push(("neighbours", s));

    // ---- surroundings (lattice survey) ----
    let mut s = Section::default();
    survey(&mut s, raw, p, ck, &mut notes);
    notes.na("hazards (flood, quake, storm)", "the engine has no hazard model; proximity of volcanoes and plate boundaries is listed under the land if present");
    sections.push(("surroundings", s));

    // ---- landmarks ----
    let mut s = Section::default();
    landmarks(&mut s, raw, p, ck, &mut notes);
    sections.push(("landmarks", s));

    finish(raw, place_name, sections, notes, sel.kept.len(), sel.within, json!({"place_index": pi}))
}

fn terrain_lines(s: &mut Section, c: &Value) {
    if let Some(v) = fl(c, "elevation_m") {
        s.fact("elevation_m", "elevation", json!(v), format!("{} m above sea level", f0(v)));
    }
    if let Some(v) = st(c, "biome") {
        s.fact("biome", "biome", json!(v), v);
    }
    if let Some(v) = st(c, "water") {
        s.fact("water", "ground", json!(v), v);
    }
    if let Some(v) = fl(c, "temperature_c") {
        s.fact("temperature_c", "mean temperature", json!(v), format!("{} C", f1(v)));
    }
    if let Some(v) = fl(c, "precipitation") {
        s.fact("precipitation", "rainfall index (0 to 1)", json!(v), format!("{v:.2}"));
    }
    if let Some(v) = fl(c, "slope_deg") {
        s.fact("slope_deg", "slope", json!(v), format!("{} degrees", f1(v)));
    }
    if let Some(v) = st(c, "rock") {
        s.fact("rock", "surface rock", json!(v), v);
    }
    if let Some(v) = fl(c, "river_order") {
        s.fact("river_order", "river on this cell, Strahler order", json!(v), f0(v));
    }
    if let Some(k) = st(c, "volcano_kind") {
        let act = st(c, "volcano_activity").unwrap_or("of unknown activity");
        s.fact("volcano", "on a volcano", json!(k), format!("{k}, {act}"));
    }
}

/// Everything the lattice survey states. Pure counts over the sample list the shell read.
fn survey(s: &mut Section, raw: &Value, p: &Value, ck: Option<f64>, notes: &mut Notes) {
    let samples = arr(raw, "samples");
    let (Some(ck), Some(px), Some(py)) = (ck, fl(p, "x"), fl(p, "y")) else {
        notes.na("land within 25 km", "the world has no declared map width, or the place has no position");
        return;
    };
    if samples.is_empty() {
        notes.na("land within 25 km", "the engine returned no terrain samples around the place");
        return;
    }
    struct Pt<'a> {
        v: &'a Value,
        d: f64,
        b: &'static str,
    }
    let world = raw.get("world").cloned().unwrap_or(Value::Null);
    let pts: Vec<Pt> = samples
        .iter()
        .filter_map(|v| {
            let (x, y) = (fl(v, "x")?, fl(v, "y")?);
            let (dx, dy) = delta(&world, px, py, x, y);
            let d = dx.hypot(dy) * ck;
            (d <= RADIUS_KM).then(|| Pt { v, d, b: bearing(dx, dy) })
        })
        .collect();
    let n = pts.len();
    if n == 0 {
        notes.na("land within 25 km", "no sample fell inside the radius");
        return;
    }
    let step_km = (LATTICE_STEP_KM / ck).round().max(1.0) * ck;
    s.json.insert("samples".into(), json!(n));
    s.json.insert("sample_spacing_km".into(), json!(step_km));
    s.line(format!("method: {n} ground samples on a {} km lattice inside {} km; anything narrower than the spacing may be missed", f1(step_km), f0(RADIUS_KM)));
    // Water.
    let count_water = |w: &str| pts.iter().filter(|q| st(q.v, "water") == Some(w)).count();
    let (land, ocean, lake) = (count_water("land"), count_water("ocean"), count_water("lake"));
    s.json.insert("water_counts".into(), json!({"land": land, "ocean": ocean, "lake": lake}));
    s.line(format!("surface: {} land, {} sea, {} lake (share of samples)", pct(land as f64 / n as f64), pct(ocean as f64 / n as f64), pct(lake as f64 / n as f64)));
    let nearest = |w: &str| pts.iter().filter(|q| st(q.v, "water") == Some(w)).min_by(|a, b| a.d.partial_cmp(&b.d).unwrap_or(std::cmp::Ordering::Equal));
    if let Some(q) = nearest("ocean") {
        s.line(format!("nearest sea sample: {} km {}", f1(q.d), q.b));
        s.json.insert("nearest_sea".into(), json!({"distance_km": q.d, "bearing": q.b}));
    } else {
        s.line(format!("no sea within {} km (by the samples)", f0(RADIUS_KM)));
    }
    if let Some(q) = nearest("lake") {
        s.line(format!("nearest lake sample: {} km {}", f1(q.d), q.b));
    }
    // Biomes among all samples.
    let mut counts: Vec<(String, usize)> = Vec::new();
    for q in &pts {
        if let Some(b) = st(q.v, "biome") {
            match counts.iter_mut().find(|(k, _)| k == b) {
                Some(e) => e.1 += 1,
                None => counts.push((b.to_owned(), 1)),
            }
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    if !counts.is_empty() {
        let shown: Vec<String> = counts.iter().take(MAX_BIOMES).map(|(k, c)| format!("{k} {}", pct(*c as f64 / n as f64))).collect();
        if counts.len() > MAX_BIOMES {
            notes.cut(format!("biomes: {} of {} listed", MAX_BIOMES, counts.len()));
        }
        s.json.insert("biomes".into(), json!(counts.iter().take(MAX_BIOMES).map(|(k, c)| json!({"biome": k, "share": *c as f64 / n as f64})).collect::<Vec<_>>()));
        s.line(format!("biomes: {}", shown.join(", ")));
    }
    // Elevation over land samples.
    let elev: Vec<(&Pt, f64)> = pts.iter().filter(|q| st(q.v, "water") == Some("land")).filter_map(|q| fl(q.v, "elevation_m").map(|e| (q, e))).collect();
    if !elev.is_empty() {
        let hi = elev.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal).then(b.0.d.partial_cmp(&a.0.d).unwrap_or(std::cmp::Ordering::Equal))).unwrap();
        let lo = elev.iter().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal).then(a.0.d.partial_cmp(&b.0.d).unwrap_or(std::cmp::Ordering::Equal))).unwrap();
        let mean = elev.iter().map(|e| e.1).sum::<f64>() / elev.len() as f64;
        s.json.insert("land_elevation_m".into(), json!({"min": lo.1, "max": hi.1, "mean": mean}));
        s.line(format!("land elevation: lowest {} m, mean {} m, highest {} m ({} km {} of the place); relief {} m", f0(lo.1), f0(mean), f0(hi.1), f1(hi.0.d), hi.0.b, f0(hi.1 - lo.1)));
    }
    // Rivers.
    let rv: Vec<(&Pt, f64)> = pts.iter().filter_map(|q| fl(q.v, "river_order").map(|o| (q, o))).collect();
    if rv.is_empty() {
        s.line("river samples: none (the lattice found no river cell)");
    } else {
        let maxo = rv.iter().map(|r| r.1).fold(0.0, f64::max);
        let near = rv.iter().min_by(|a, b| a.0.d.partial_cmp(&b.0.d).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
        s.json.insert("river_samples".into(), json!({"count": rv.len(), "max_order": maxo, "nearest_km": near.0.d}));
        s.line(format!("river samples: {} cells on rivers, highest Strahler order {}, nearest {} km {}", rv.len(), f0(maxo), f1(near.0.d), near.0.b));
    }
    if let Some(r) = raw.get("river").filter(|r| r.is_object() && !r.as_object().unwrap().is_empty()) {
        let mut bits = Vec::new();
        if let Some(o) = it(r, "order") {
            bits.push(format!("Strahler order {o}"));
        }
        if let Some(k) = fl(r, "km") {
            bits.push(format!("{} km long", f1(k)));
        }
        if let Some(c) = fl(r, "catchment_km2") {
            bits.push(format!("catchment {} km2", f0(c)));
        }
        if let Some(t) = it(r, "tributaries") {
            bits.push(format!("{t} tributaries"));
        }
        if let (Some(a), Some(b)) = (fl(r, "source_m"), fl(r, "mouth_m")) {
            bits.push(format!("falls from {} m to {} m", f0(a), f0(b)));
        }
        if let Some(d) = river_distance_km(r, px, py, ck) {
            bits.push(format!("{} km from the place at its nearest", f1(d)));
        }
        if !bits.is_empty() {
            s.json.insert("nearest_river".into(), r.clone());
            s.line(format!("nearest river (order 2 or more): {} (rivers are not named by the engine)", bits.join(", ")));
        }
    } else {
        s.line(format!("no river of order 2 or more within {} km (by the engine's river lookup)", f0(RADIUS_KM)));
    }
    // Volcanoes.
    let mut vol: Vec<(String, String, i64, f64, &'static str, usize)> = Vec::new();
    for q in &pts {
        let Some(act) = st(q.v, "volcano_activity") else { continue };
        let kind = st(q.v, "volcano_kind").unwrap_or("edifice");
        let age = it(q.v, "volcano_age_years").unwrap_or(-1);
        match vol.iter_mut().find(|e| e.0 == kind && e.1 == act && e.2 == age) {
            Some(e) => {
                e.5 += 1;
                if q.d < e.3 {
                    e.3 = q.d;
                    e.4 = q.b;
                }
            }
            None => vol.push((kind.to_owned(), act.to_owned(), age, q.d, q.b, 1)),
        }
    }
    vol.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(&b.0)));
    if vol.is_empty() {
        s.line(format!("volcanoes: none among the samples within {} km", f0(RADIUS_KM)));
    } else {
        if vol.len() > MAX_VOLCANOES {
            notes.cut(format!("volcanoes: {} of {} groups listed (nearest first)", MAX_VOLCANOES, vol.len()));
        }
        for v in vol.iter().take(MAX_VOLCANOES) {
            let age = if v.2 >= 0 { format!(", about {} years old", v.2) } else { String::new() };
            s.line(format!("volcano: {} ({}{}), nearest sample {} km {}", v.0, v.1, age, f1(v.3), v.4));
        }
        s.json.insert("volcanoes".into(), json!(vol.iter().take(MAX_VOLCANOES).map(|v| json!({"kind": v.0, "activity": v.1, "age_years": if v.2 >= 0 { Some(v.2) } else { None }, "nearest_km": v.3, "bearing": v.4})).collect::<Vec<_>>()));
    }
    // Plate boundary.
    let bd = pts.iter().filter(|q| bl(q.v, "boundary") == Some(true)).min_by(|a, b| a.d.partial_cmp(&b.d).unwrap_or(std::cmp::Ordering::Equal));
    match bd {
        Some(q) => {
            let t = st(q.v, "boundary_type").unwrap_or("boundary");
            s.line(format!("plate boundary: {t}, nearest sample {} km {}", f1(q.d), q.b));
        }
        None => s.line(format!("plate boundary: none among the samples within {} km", f0(RADIUS_KM))),
    }
    // Climate spread.
    let temps: Vec<f64> = pts.iter().filter_map(|q| fl(q.v, "temperature_c")).collect();
    if !temps.is_empty() {
        let (lo, hi) = temps.iter().fold((f64::MAX, f64::MIN), |a, &t| (a.0.min(t), a.1.max(t)));
        s.line(format!("temperature across the samples: {} C to {} C", f1(lo), f1(hi)));
    }
}

/// Distance in km from `(px, py)` (cells) to the polyline in `river.points` (`[x0,y0,x1,y1,...]`
/// cells), or `None` when the points are absent.
fn river_distance_km(river: &Value, px: f64, py: f64, ck: f64) -> Option<f64> {
    let pts: Vec<f64> = river.get("points")?.as_array()?.iter().filter_map(Value::as_f64).collect();
    if pts.len() < 2 {
        return None;
    }
    let xy: Vec<(f64, f64)> = pts.chunks(2).filter(|c| c.len() == 2).map(|c| (c[0], c[1])).collect();
    let mut best = f64::MAX;
    if xy.len() == 1 {
        best = (xy[0].0 - px).hypot(xy[0].1 - py);
    }
    for w in xy.windows(2) {
        let (a, b) = (w[0], w[1]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let len2 = dx * dx + dy * dy;
        let t = if len2 == 0.0 { 0.0 } else { (((px - a.0) * dx + (py - a.1) * dy) / len2).clamp(0.0, 1.0) };
        best = best.min((a.0 + t * dx - px).hypot(a.1 + t * dy - py));
    }
    best.is_finite().then(|| best * ck)
}

fn landmarks(s: &mut Section, raw: &Value, p: &Value, ck: Option<f64>, notes: &mut Notes) {
    let Some(l) = raw.get("landmarks") else {
        notes.na("landmarks", "the landmark pass has not run for this world (Cartograph > Landmarks)");
        return;
    };
    if bl(l, "available") == Some(false) {
        notes.na("landmarks", st(l, "reason").unwrap_or("the landmark pass has not run for this world").to_owned());
        return;
    }
    let (Some(ck), Some(px), Some(py)) = (ck, fl(p, "x"), fl(p, "y")) else {
        notes.na("landmarks", "no map width or no position for the place");
        return;
    };
    let world = raw.get("world").cloned().unwrap_or(Value::Null);
    let mut near: Vec<(f64, &'static str, &Value)> = arr(l, "rows")
        .iter()
        .filter_map(|r| {
            let (x, y) = (fl(r, "x")?, fl(r, "y")?);
            let (dx, dy) = delta(&world, px, py, x, y);
            let d = dx.hypot(dy) * ck;
            (d <= RADIUS_KM).then(|| (d, bearing(dx, dy), r))
        })
        .collect();
    near.sort_by(|a, b| {
        fl(b.2, "importance").unwrap_or(0.0).partial_cmp(&fl(a.2, "importance").unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal).then(a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)).then(it(a.2, "id").cmp(&it(b.2, "id")))
    });
    let total = near.len();
    if total == 0 {
        s.line(format!("landmarks: none within {} km", f0(RADIUS_KM)));
        return;
    }
    if total > MAX_LANDMARKS {
        notes.cut(format!("landmarks: {} of {} within {} km listed (most important first)", MAX_LANDMARKS, total, f0(RADIUS_KM)));
    }
    let mut jl = Vec::new();
    for (d, b, r) in near.into_iter().take(MAX_LANDMARKS) {
        let kind = st(r, "kind").unwrap_or("landmark");
        let causal = strs(r, "causal");
        let why = if causal.is_empty() { String::new() } else { format!("; because {}", causal.join(", ")) };
        s.line(format!("- {kind} ({} km {}{}{})", f1(d), b, st(r, "class").map(|c| format!("; {c} scale")).unwrap_or_default(), why));
        jl.push(json!({"kind": kind, "distance_km": d, "bearing": b, "class": st(r, "class"), "causal": causal}));
    }
    s.json.insert("listed".into(), Value::Array(jl));
    s.json.insert("within_radius".into(), json!(total));
}

/// Assemble sections into the final object and text, applying the size cap.
fn finish(raw: &Value, place_name: String, sections: Vec<(&'static str, Section)>, mut notes: Notes, listed: usize, within: usize, extra_json: Value) -> PlaceFacts {
    let mut world_data = Map::new();
    let mut text = String::new();
    let mut sent: Vec<(String, usize)> = Vec::new();
    let mut dropped: Vec<String> = Vec::new();
    let mut over = false;
    for (id, sec) in &sections {
        let label = SECTION_ORDER.iter().find(|(i, _)| i == id).map(|(_, l)| *l).unwrap_or(id);
        if sec.is_empty() && sec.json.is_empty() {
            continue;
        }
        let block = format!("[{label}]\n{}\n", sec.lines.join("\n"));
        if over || text.chars().count() + block.chars().count() > MAX_WORLD_CHARS {
            over = true;
            dropped.push(label.to_owned());
            continue;
        }
        text.push_str(&block);
        sent.push((label.to_owned(), sec.lines.len()));
        world_data.insert((*id).to_owned(), Value::Object(sec.json.clone()));
    }
    if !dropped.is_empty() {
        let msg = format!("sections dropped for size (limit {MAX_WORLD_CHARS} characters): {}", dropped.join(", "));
        notes.cut(msg.clone());
        text.push_str(&format!("[Omitted for size]\n{msg}\n"));
    }
    if !notes.not_available.is_empty() {
        text.push_str("[Not available from the world data (do not guess these)]\n");
        for (item, reason) in &notes.not_available {
            text.push_str(&format!("- {item}: not available ({reason})\n"));
        }
    }
    if !notes.truncated.is_empty() {
        text.push_str("[Truncated (listed so you know what is missing)]\n");
        for t in &notes.truncated {
            text.push_str(&format!("- {t}\n"));
        }
    }
    let json = json!({
        "schema": 1,
        "place": place_name,
        "radius_km": RADIUS_KM,
        "world_data": Value::Object(world_data),
        "not_available": notes.not_available.iter().map(|(i, r)| json!({"item": i, "reason": r})).collect::<Vec<_>>(),
        "truncated": notes.truncated,
        "meta": extra_json,
    });
    let _ = raw;
    let summary = FactsSummary {
        sections: sent,
        chars: text.chars().count(),
        truncated: notes.truncated,
        not_available: notes.not_available,
        neighbours_listed: listed,
        neighbours_within: within,
    };
    PlaceFacts { json, text, summary, place_name }
}

/// The bundle the shell stores and hands to `prepare` (`prompt::PlaceContext::from_json` reads
/// it): the WORLD DATA text, the VAULT NOTES text, exactly what each contains, the structured
/// facts for display, and the grounding corpus. `own_note_text` is the place's existing note
/// (already read by the shell); it joins the corpus whether or not Vault notes are sent, so the
/// grounding check treats a name or number the user wrote as grounded.
pub fn bundle(facts: &PlaceFacts, vault: &crate::place_vault::VaultNotes, own_note_text: &str, vault_allowed: bool) -> Value {
    let (own, _) = crate::place_vault::strip_model_blocks(own_note_text);
    let mut truncated = facts.summary.truncated.clone();
    truncated.extend(vault.truncated.iter().cloned());
    let corpus = format!("{}
{}
{}", facts.text, vault.corpus, own);
    json!({
        "label": facts.place_name,
        "world_text": facts.text,
        "world_sections": facts.summary.sections.iter().map(|(l, n)| json!([l, n])).collect::<Vec<_>>(),
        "truncated": truncated,
        "not_available": facts.summary.not_available.len(),
        "not_available_items": facts.summary.not_available.iter().map(|(i, r)| json!({"item": i, "reason": r})).collect::<Vec<_>>(),
        "neighbours_listed": facts.summary.neighbours_listed,
        "neighbours_within": facts.summary.neighbours_within,
        "vault_text": vault.text,
        "vault_notes": vault.sent.iter().map(|n| json!({"title": n.title, "rel": n.rel, "why": n.why, "chars": n.chars})).collect::<Vec<_>>(),
        "vault_withheld": vault.withheld,
        "vault_allowed": vault_allowed,
        "model_blocks_removed": vault.model_blocks_removed,
        "facts_json": facts.json,
        "corpus": corpus,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> Value {
        // 800 km over 2000 cells = 0.4 km per cell: 62 cells = 24.8 km, 63 cells = 25.2 km.
        json!({"map_width_km": 800.0, "gw": 2000, "gh": 1500})
    }

    fn st_row(i: usize, name: &str, x: f64, y: f64, pop: i64) -> Value {
        json!({"i": i, "tid": (i as i64) + 1, "name": name, "x": x, "y": y, "pop": pop, "kind": "town", "faction": 1})
    }

    /// Protects: on a wrapping map the shorter way round the seam is the distance; on a flat map it
    /// is not. Place at x=5, a town at x=1995 on a 2000-cell map: 10 cells (4 km) west when wrapped,
    /// 1990 cells (796 km) when not. MUTATION: ignore `wrap_x` and the first assert goes red.
    #[test]
    fn wrap_takes_the_short_way_round() {
        let mut w = world();
        let raw = |w: &Value| json!({"world": w.clone(), "place_index": 0, "settlements": [
            st_row(0, "Home", 5.0, 700.0, 500), st_row(1, "Across", 1995.0, 700.0, 100)]});
        w["wrap_x"] = json!(true);
        let sel = select_neighbours(&raw(&w));
        assert_eq!(sel.within, 1);
        assert!((sel.kept[0].dist_km - 4.0).abs() < 1e-9 && sel.kept[0].bearing == "W", "{:?}", sel.kept);
        w["wrap_x"] = json!(false);
        assert_eq!(select_neighbours(&raw(&w)).within, 0);
    }

    /// Protects: cell size is map width over grid width, and a missing width is `None`, not 0.
    /// MUTATION: divide by gh, or default a missing width to 0, and this goes red.
    #[test]
    fn cell_size_and_missing_width() {
        assert_eq!(cell_km(&world()), Some(0.4));
        assert_eq!(cell_km(&json!({"gw": 100})), None);
        assert_eq!(cell_km(&json!({"map_width_km": 0.0, "gw": 100})), None);
    }

    /// Protects: bearings are compass words with NORTH UP THE MAP (-y), as literals for all eight
    /// points and the zero vector.
    /// MUTATION: use +y for north and N/S swap.
    #[test]
    fn bearings_are_literal() {
        assert_eq!(bearing(0.0, -10.0), "N");
        assert_eq!(bearing(10.0, -10.0), "NE");
        assert_eq!(bearing(10.0, 0.0), "E");
        assert_eq!(bearing(10.0, 10.0), "SE");
        assert_eq!(bearing(0.0, 10.0), "S");
        assert_eq!(bearing(-10.0, 10.0), "SW");
        assert_eq!(bearing(-10.0, 0.0), "W");
        assert_eq!(bearing(-10.0, -10.0), "NW");
        assert_eq!(bearing(0.0, 0.0), "");
    }

    /// Protects the radius, owner's bar: a settlement 24.8 km away is IN and one 25.2 km away is
    /// OUT; exactly 25.0 km is IN (inclusive); the place itself is never its own neighbour.
    /// MUTATION: change RADIUS_KM, use `<` instead of `<=`, or drop the self-exclusion.
    #[test]
    fn radius_is_inclusive_and_exact() {
        // 0.4 km/cell: 62 cells = 24.8 km, 62.5 cells = 25.0 km, 63 cells = 25.2 km.
        let raw = json!({"world": world(), "place_index": 0, "settlements": [
            st_row(0, "Home", 1000.0, 700.0, 500),
            st_row(1, "Inside", 1062.0, 700.0, 100),
            st_row(2, "Edge", 1062.5, 700.0, 100),
            st_row(3, "Outside", 1063.0, 700.0, 100),
            st_row(4, "North", 1000.0, 638.0, 100),
        ]});
        let sel = select_neighbours(&raw);
        let names: Vec<&str> = sel.kept.iter().map(|n| n.name.as_str()).collect();
        assert_eq!(sel.within, 3, "{names:?}");
        assert!(names.contains(&"Inside") && names.contains(&"Edge") && names.contains(&"North"), "{names:?}");
        assert!(!names.contains(&"Outside") && !names.contains(&"Home"), "{names:?}");
        let inside = sel.kept.iter().find(|n| n.name == "Inside").unwrap();
        assert!((inside.dist_km - 24.8).abs() < 1e-9 && inside.bearing == "E");
        assert_eq!(sel.kept.iter().find(|n| n.name == "North").unwrap().bearing, "N");
    }

    /// Protects: the neighbour cap keeps the LARGEST twelve (not the nearest), shows them nearest
    /// first, and counts the rest as omitted with their names.
    /// MUTATION: sort by distance before truncating, or change the cap.
    #[test]
    fn cap_keeps_largest_and_lists_the_rest() {
        let mut v = vec![st_row(0, "Home", 1000.0, 700.0, 500)];
        for k in 1..=15usize {
            // Settlement k sits k cells east; population grows with k, so the largest are the FARTHEST.
            v.push(st_row(k, &format!("N{k:02}"), 1000.0 + k as f64, 700.0, 10 * k as i64));
        }
        let sel = select_neighbours(&json!({"world": world(), "place_index": 0, "settlements": v}));
        assert_eq!(sel.within, 15);
        assert_eq!(sel.kept.len(), 12);
        assert_eq!(sel.omitted.len(), 3);
        let kept: Vec<&str> = sel.kept.iter().map(|n| n.name.as_str()).collect();
        assert_eq!(kept, ["N04", "N05", "N06", "N07", "N08", "N09", "N10", "N11", "N12", "N13", "N14", "N15"], "largest twelve, nearest first");
        let om: Vec<&str> = sel.omitted.iter().map(|n| n.name.as_str()).collect();
        assert_eq!(om, ["N01", "N02", "N03"]);
    }

    /// Protects: the lattice is anchored on the place, row-major, inside the grid and the radius,
    /// and identical on a second call.
    /// MUTATION: drop the bounds test or the radius test and a literal count changes.
    #[test]
    fn lattice_is_bounded_and_deterministic() {
        // 0.4 km cells: step = round(2.5/0.4) = 6 cells (2.4 km); n = ceil(25/2.4) = 11.
        let a = sample_points(1000, 700, 0.4, 2000, 1500, false);
        assert_eq!(a, sample_points(1000, 700, 0.4, 2000, 1500, false));
        assert!(a.contains(&(1000, 700)) && a.contains(&(1006, 700)) && !a.contains(&(1001, 700)));
        for &(x, y) in &a {
            assert!(((x - 1000) as f64).hypot((y - 700) as f64) * 0.4 <= 25.0);
        }
        // 66 cells = 26.4 km is outside; 60 cells = 24 km is inside.
        assert!(a.contains(&(1060, 700)) && !a.contains(&(1066, 700)));
        // Clipped at the grid edge: a place in the corner keeps only in-grid points.
        let corner = sample_points(2, 3, 0.4, 2000, 1500, false);
        assert!(corner.iter().all(|&(x, y)| x >= 0 && y >= 0));
        assert!(corner.len() < a.len());
        assert!(sample_points(5, 5, 0.0, 10, 10, false).is_empty());
        // On a map that wraps east-west the lattice continues across the seam instead of stopping.
        let wrapped = sample_points(2, 700, 0.4, 2000, 1500, true);
        assert!(wrapped.iter().any(|&(x, _)| x > 1900), "points wrap to the east edge");
        assert!(wrapped.iter().all(|&(x, _)| (0..2000).contains(&x)));
        assert!(wrapped.len() > sample_points(2, 700, 0.4, 2000, 1500, false).len());
    }

    fn base_raw() -> Value {
        json!({
            "world": world(), "place_index": 0,
            "settlements": [
                {"i": 0, "tid": 7, "name": "Veldmark", "x": 1000.0, "y": 700.0, "pop": 4213, "kind": "city", "faction": 2, "capital": true, "coastal": true, "religion": "sunfaith"},
                {"i": 1, "tid": 8, "name": "Oakhurst", "x": 1030.0, "y": 700.0, "pop": 612, "kind": "village", "faction": 2, "religion": "sunfaith"},
                {"i": 2, "tid": 9, "name": "Farhaven", "x": 1500.0, "y": 700.0, "pop": 99999, "kind": "city", "faction": 3},
            ],
            "factions": [
                {"id": 2, "name": "Kingdom of Vael", "culture": "norse", "religion": "sunfaith", "government": "monarchy", "ag_tech": "iron", "religious_policy": "plural", "settlement_count": 2, "population": 4825, "claimed_cells": 100000},
                {"id": 3, "name": "Far League", "culture": "lake", "religion": "none", "government": "republic"},
            ],
            "relations": [{"a": 2, "b": 3, "a_name": "Kingdom of Vael", "b_name": "Far League", "stance": "wary", "value": -0.2, "border_fraction": 0.31, "border_cells": 40}],
            "place": {"specialisation": "port", "traits": ["harbour"], "age_inferred": 310, "age": -1, "trade": {"exports": ["timber"], "imports": ["salt"]},
                      "religion_shares": {"sunfaith": 0.8, "river_cult": 0.2}, "military": {"wall_spec": "stone", "defensibility": 0.72, "garrison": 180}},
            "neighbour_extra": {"1": {"specialisation": "farming", "trade": {"exports": ["grain"], "imports": []}}},
            "labels": {"religion": {"sunfaith": "Sunfaith", "river_cult": "River Cult"}, "specialisation": {"port": "Port"}, "traits": {"harbour": "Harbour"}},
            "place_cell": {"x": 1000, "y": 700, "elevation_m": 14.0, "biome": "temperate forest", "water": "land", "temperature_c": 11.2, "precipitation": 0.6},
            "samples": [
                {"x": 1000, "y": 750, "water": "ocean", "biome": "ocean", "elevation_m": -40.0},
                {"x": 1010, "y": 690, "water": "land", "biome": "temperate forest", "elevation_m": 320.0, "river_order": 3},
                {"x": 1020, "y": 700, "water": "land", "biome": "grassland", "elevation_m": 80.0, "volcano_kind": "strato", "volcano_activity": "dormant", "volcano_age_years": 90000}
            ],
        })
    }

    /// Protects: the facts carry the engine's own numbers verbatim (population 4213, distance
    /// 12.0 km east for 30 cells at 0.4 km), the faction, the religion, the relation and the trade.
    /// MUTATION: change a getter key, a unit conversion or a label and a literal below goes red.
    #[test]
    fn facts_carry_the_engine_values() {
        let f = build(&base_raw());
        let t = &f.text;
        for needle in [
            "population: 4213", "size class: city", "capital: yes", "faction: Kingdom of Vael", "government: monarchy",
            "main faith here: Sunfaith", "faiths here (share of people): Sunfaith 80%, River Cult 20%",
            "- Far League: wary (value -0.20), 31% of its border shared",
            "this settlement's resource surplus (exports): timber", "(imports): salt",
            "- Oakhurst (village; pop 612; faction Kingdom of Vael; 12.0 km E;", "specialisation farming", "exports grain",
            "walls: stone", "defensibility (0 to 1): 0.72", "elevation: 14 m above sea level",
            "nearest sea sample: 20.0 km S", "highest Strahler order 3", "volcano: strato (dormant, about 90000 years old)",
        ] {
            assert!(t.contains(needle), "missing {needle:?} in\n{t}");
        }
        // Farhaven is 200 km away: never listed.
        assert!(!t.contains("Farhaven"), "{t}");
        assert_eq!(f.json["world_data"]["place"]["population"], json!(4213));
        assert_eq!(f.summary.neighbours_within, 1);
    }

    /// Protects: absence is stated with a reason, never as a value. A world with no trade balance,
    /// no landmarks and no samples says "not available" for each and writes no `0`.
    /// MUTATION: default an absent field to 0 / "none" in the builder.
    #[test]
    fn absent_data_is_not_available_with_a_reason() {
        let raw = json!({"world": world(), "place_index": 0, "settlements": [{"i": 0, "tid": 1, "name": "Lone", "x": 10.0, "y": 10.0, "pop": 50, "kind": "hamlet", "faction": 0}]});
        let f = build(&raw);
        let t = &f.text;
        assert!(t.contains("faction: none"), "{t}");
        assert!(t.contains("terrain at the place: not available (the engine returned no sample"), "{t}");
        assert!(t.contains("landmarks: not available (the landmark pass has not run"), "{t}");
        assert!(t.contains("land within 25 km: not available (the engine returned no terrain samples"), "{t}");
        assert!(t.contains("settlement trade balance: not available"), "{t}");
        assert!(t.contains("per-settlement production and consumption: not available"), "{t}");
        assert!(t.contains("worship sites and local rites: not available"), "{t}");
        assert!(!t.contains("elevation"), "an absent elevation must not appear at all: {t}");
        assert!(!t.contains("exports:"), "{t}");
    }

    /// Protects: determinism. The same raw input renders byte-identically, JSON and text.
    #[test]
    fn same_input_same_output() {
        let a = build(&base_raw());
        let b = build(&base_raw());
        assert_eq!(a.text, b.text);
        assert_eq!(a.json.to_string(), b.json.to_string());
    }

    /// Protects: a user-written history longer than the cap is cut and the cut is LISTED.
    /// MUTATION: raise MAX_USER_TEXT_CHARS or skip the notes.cut call.
    #[test]
    fn long_user_text_is_cut_and_listed() {
        let mut raw = base_raw();
        raw["place"]["history"] = json!("x".repeat(601));
        let f = build(&raw);
        assert!(f.summary.truncated.iter().any(|t| t.contains("history text was cut to 600")), "{:?}", f.summary.truncated);
        assert!(f.text.contains("[Truncated"));
        raw["place"]["history"] = json!("x".repeat(600));
        assert!(build(&raw).summary.truncated.iter().all(|t| !t.contains("history")));
    }

    /// Protects: omitted neighbours are listed by name inside the text and the truncation list.
    #[test]
    fn omitted_neighbours_are_named() {
        let mut v = vec![st_row(0, "Home", 1000.0, 700.0, 500)];
        for k in 1..=14usize {
            v.push(st_row(k, &format!("Town{k:02}"), 1000.0 + k as f64, 700.0, 10 * k as i64));
        }
        let f = build(&json!({"world": world(), "place_index": 0, "settlements": v}));
        assert!(f.text.contains("2 of 14 within 25 km are not listed"), "{}", f.text);
        assert!(f.text.contains("Town01 (0.4 km E, pop 10)") && f.text.contains("Town02"), "{}", f.text);
        assert_eq!(f.summary.neighbours_listed, 12);
    }

    /// Protects: the size cap. A flood of long lines drops whole trailing sections and says which.
    /// MUTATION: lift MAX_WORLD_CHARS and the omission line disappears.
    #[test]
    fn size_cap_drops_the_tail_and_says_so() {
        let mut raw = base_raw();
        let long = "word ".repeat(120);
        raw["place"]["history"] = json!(long);
        let mut lms = Vec::new();
        for k in 0..60 {
            lms.push(json!({"id": k, "kind": "waterfall with a very long descriptive causal chain", "class": "local", "x": 1000 + k, "y": 700, "importance": 0.5, "causal": ["a long reason ".repeat(300)]}));
        }
        raw["landmarks"] = json!({"available": true, "rows": lms});
        let f = build(&raw);
        // The landmarks block alone is far over the cap, so it is dropped and the drop is named.
        assert!(f.text.contains("[Omitted for size]") && f.text.contains("sections dropped for size (limit 9000 characters): Landmarks within 25 km"), "{}", f.text.chars().take(400).collect::<String>());
        assert!(f.summary.truncated.iter().any(|t| t.contains("sections dropped for size")));
        assert!(!f.text.contains("[Landmarks within 25 km]"), "the dropped section is not also sent");
        assert!(f.summary.sections.len() >= 5);
        // Everything kept fits under the cap (the "not available" and "truncated" tails come after it).
        let kept: usize = f.text.split("[Omitted for size]").next().unwrap().chars().count();
        assert!(kept <= MAX_WORLD_CHARS, "{kept}");
    }

    /// Protects: a river's distance is measured to the polyline, not to a vertex.
    /// MUTATION: measure to the nearest vertex and the literal changes.
    #[test]
    fn river_distance_is_to_the_segment() {
        // River along y = 700 from x = 900 to x = 1100; place at (1000, 750): perpendicular 50 cells = 20 km.
        let r = json!({"points": [900.0, 700.0, 1100.0, 700.0]});
        let d = river_distance_km(&r, 1000.0, 750.0, 0.4).unwrap();
        assert!((d - 20.0).abs() < 1e-9, "{d}");
        assert!(river_distance_km(&json!({}), 0.0, 0.0, 0.4).is_none());
    }
}
