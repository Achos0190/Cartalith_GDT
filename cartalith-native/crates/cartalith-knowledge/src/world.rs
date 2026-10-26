//! The world as documents (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.4.3, milestone RK-1; Ruling DM item
//! 1b): a pure function from plain input structs to short text documents.
//!
//! # The principle
//!
//! The model never sees raw engine arrays. Each world entity becomes one [`Document`] rendered by
//! [`render_documents`] in a fixed template. The input structs ([`WorldInput`] and its rows) are
//! filled by the shell bridge (`cartalith-godot`'s `knowledge_bridge.rs`) from the engine's own
//! state, so this crate has no engine dependency and the renderer can be tested on a literal input.
//!
//! # "Not available" instead of invention (Ruling DM; `MISTAKES.md`, "Return, marshal or default a
//! value that can be absent")
//!
//! Every figure the engine may not have is an `Option` in the input. `None` renders as
//! `label: not available (reason)`, never as `0`, `none` or an empty value. The reason is the
//! input row's own entry in its `absent` map when the bridge knows the cause, else a fixed default
//! for that field. A figure that IS zero (a village of population 0, a coast with no river) is
//! `Some(0)` and renders as the number: absent and zero are different statements.
//!
//! # Determinism (s.6.4)
//!
//! Documents are returned sorted by id; every list inside a document is sorted by (name, id); no
//! hash map is iterated; floats are printed with a fixed precision. Two renders of one input are
//! byte-identical (a test hashes them).
//!
//! # Size
//!
//! A document is at most [`MAX_DOC_BYTES`]; a longer one is cut at a line boundary and says so
//! (`[truncated: N more lines]`), so a cut is never silent.
//!
//! # What it must never do
//!
//! Invent a name, a number or a relation the input does not carry; render a missing figure as a
//! plausible value; read a clock or a random source.

use std::collections::BTreeMap;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::model::{Document, Source, world_id};

/// The largest a rendered world document may be, in bytes (RK-1 bar: "each document at most
/// 6 000 bytes"). A named constant with a literal-pinned test.
pub const MAX_DOC_BYTES: usize = 6000;

/// The relation names the world graph uses, in priority order for link expansion (s.4.4).
/// `documented_by` joins a world document to the Vault note whose machine-owned block names it;
/// that edge is added by the index, not by [`world_edges`].
pub const WORLD_RELATIONS: [&str; 11] = [
    "owned_by",
    "adheres_to",
    "governed_by",
    "located_in",
    "member_of",
    "trades_with",
    "at_war_with",
    "allied_with",
    "near_landmark",
    "party_to",
    "has_culture",
];

/// `field -> reason` for the figures one input row could not supply (module doc).
pub type Absent = BTreeMap<String, String>;

/// A settlement (`get_settlements`, `civ_settlement_details`, `explain_settlement`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SettlementIn {
    /// The stable settlement id (`tid`): what a Vault `entity="settlement:<id>"` block names.
    pub id: i64,
    pub name: String,
    /// The kind key (`city`, `town`, `village`, ...).
    pub kind: String,
    pub population: u32,
    pub x: i64,
    pub y: i64,
    /// Owning faction id; 0 means no faction.
    pub faction: i64,
    pub capital: bool,
    pub coastal: bool,
    /// Province id covering its cell, if known and any.
    pub province: Option<i64>,
    /// Continent id, if known.
    pub continent: Option<i64>,
    /// The plurality religion's label, if the belief layer ran.
    pub religion: Option<String>,
    /// The order of the river the site stands on (0 = none), if the site explanation exists.
    pub river_order: Option<u32>,
    #[serde(default)]
    pub absent: Absent,
}

/// A faction (`get_factions`, `civ_faction_territory_stats`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactionIn {
    /// 1-based roster id.
    pub id: i64,
    pub name: String,
    pub culture: String,
    /// The religion's display label (a custom religion's own name).
    pub religion_label: String,
    /// The religion's key (`custom:3` or a built-in key).
    pub religion_key: String,
    pub government_key: String,
    pub government_label: String,
    pub ag_tech: String,
    pub claimed_cells: Option<u64>,
    #[serde(default)]
    pub absent: Absent,
}

/// A religion, built-in or custom (`civ_religion_vocabulary`, `civ_custom_religions`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReligionIn {
    pub key: String,
    pub label: String,
    /// True for a user-made religion.
    pub custom: bool,
    /// The built-in a custom religion behaves as.
    pub base_key: Option<String>,
    pub notes: Option<String>,
    /// `axis -> value`, sorted; unclassified axes omitted by the bridge.
    #[serde(default)]
    pub axes: BTreeMap<String, String>,
    #[serde(default)]
    pub absent: Absent,
}

/// A government form (`civ_government_vocabulary`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GovernmentIn {
    pub key: String,
    pub label: String,
    pub hint: Option<String>,
    pub group: Option<String>,
    #[serde(default)]
    pub absent: Absent,
}

/// A culture (`get_cultures`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CultureIn {
    /// The 0-based `CIV_CULTURES` index: what a Vault `entity="culture:<n>"` block names.
    pub index: i64,
    pub key: String,
    pub name: String,
    pub terrain_affinity: Option<String>,
    #[serde(default)]
    pub absent: Absent,
}

/// A province (`get_provinces`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProvinceIn {
    pub id: i64,
    pub name: String,
    pub faction: i64,
    /// The seed settlement's id, if it resolves.
    pub capital: Option<i64>,
    #[serde(default)]
    pub absent: Absent,
}

/// A continent (`get_continents`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContinentIn {
    pub id: i64,
    pub name: String,
    pub cells: u64,
    /// The faction holding most cells (0 = unclaimed).
    pub faction: i64,
    #[serde(default)]
    pub absent: Absent,
}

/// A faction pair's relation (`civ_faction_relations`). `a < b` by faction id.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RelationIn {
    pub a: i64,
    pub b: i64,
    /// One of allied / friendly / neutral / wary / hostile; absent without a claim grid.
    pub stance: Option<String>,
    pub value: Option<f64>,
    pub border_cells: Option<u64>,
    #[serde(default)]
    pub absent: Absent,
}

/// One trade flow between two settlements (`civ_trade_flows`' `flows`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TradeFlowIn {
    /// Exporting settlement id.
    pub from: i64,
    /// Importing settlement id.
    pub to: i64,
    pub good: String,
    /// `land`, `river` or `sea`.
    pub mode: String,
    pub volume: f64,
    pub distance_km: f64,
}

/// A generated landmark (`landmarks`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LandmarkIn {
    pub index: i64,
    pub kind: String,
    /// The Vault entity id (`landmark_entity_id`), if the bridge computed it.
    pub entity_id: Option<i64>,
    pub name: Option<String>,
    pub x: i64,
    pub y: i64,
    /// Province name covering it, if any.
    pub region: Option<String>,
    /// The settlement nearest to it (id, distance in cells), if any settlement exists.
    pub nearest_settlement: Option<(i64, f64)>,
    /// Why it is there (the placement cause), if the run recorded one.
    pub cause: Option<String>,
    #[serde(default)]
    pub absent: Absent,
}

/// A traced river (`get_rivers`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RiverIn {
    pub index: i64,
    pub order: u32,
    pub km: f64,
    pub tributaries: u32,
    pub head: (i64, i64),
    pub mouth: (i64, i64),
    pub source_m: Option<f64>,
    pub mouth_m: Option<f64>,
    #[serde(default)]
    pub absent: Absent,
}

/// A volcano (the geology table's `VolcanoRow`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VolcanoIn {
    pub id: i64,
    /// The edifice morphology; `None` under the reference edifice model, which records none.
    pub kind: Option<String>,
    pub age_class: Option<String>,
    pub state: Option<String>,
    pub x: i64,
    pub y: i64,
    #[serde(default)]
    pub absent: Absent,
}

/// A conflict from the Story layer (`conflict_list`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConflictIn {
    pub id: i64,
    pub name: String,
    pub kind: String,
    /// Faction ids on the sides.
    pub parties: Vec<i64>,
    pub start_year: i64,
    pub end_year: Option<i64>,
    pub notes: Option<String>,
    #[serde(default)]
    pub absent: Absent,
}

/// A saved journey (`journey_list`): the planner's name, party, start year and route length.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct JourneyIn {
    pub id: i64,
    pub name: String,
    /// The party preset id it was saved with (may name a preset since deleted).
    pub party_preset: String,
    pub start_year: i64,
    /// The saved route's length in km.
    pub distance_km: f64,
    /// The route mode key (`land`, `river`, `sea`, ...).
    pub mode: String,
    #[serde(default)]
    pub absent: Absent,
}

/// Everything the renderer needs, as plain data. Every list may be empty (an empty list renders
/// no document of that kind; it is the bridge's job to say a getter was unavailable).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldInput {
    /// The generation seed, echoed in the world rollup (a figure of the world).
    pub seed: i64,
    pub settlements: Vec<SettlementIn>,
    pub factions: Vec<FactionIn>,
    pub religions: Vec<ReligionIn>,
    pub governments: Vec<GovernmentIn>,
    pub cultures: Vec<CultureIn>,
    pub provinces: Vec<ProvinceIn>,
    pub continents: Vec<ContinentIn>,
    pub relations: Vec<RelationIn>,
    /// `None` = the trade match was not computed (every settlement then says "not available");
    /// `Some(empty)` = it ran and found no flow.
    pub trade: Option<Vec<TradeFlowIn>>,
    pub landmarks: Vec<LandmarkIn>,
    pub rivers: Vec<RiverIn>,
    pub volcanoes: Vec<VolcanoIn>,
    pub conflicts: Vec<ConflictIn>,
    pub journeys: Vec<JourneyIn>,
}

/// A directed world relation `from --rel--> to`, by document id.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldEdge {
    pub from: String,
    pub rel: &'static str,
    pub to: String,
}

// ----------------------------------------------------------------------------------- helpers

/// `label: value` or `label: not available (reason)` (module doc).
fn opt_line<T: Display>(lines: &mut Vec<String>, label: &str, v: Option<T>, absent: &Absent, key: &str, default_reason: &str) {
    match v {
        Some(v) => lines.push(format!("{label}: {v}")),
        None => {
            let reason = absent.get(key).map_or(default_reason, |r| r.as_str());
            lines.push(format!("{label}: not available ({reason})"));
        }
    }
}

/// The most items one inline list shows before it says how many it left out. A labelled judgement
/// (60 references of about 30 bytes is about 1.8 KB, under a third of [`MAX_DOC_BYTES`]); without it
/// one long list line would be the line the size cap drops, and the document would lose its list
/// silently instead of naming the cut.
pub const MAX_LIST_ITEMS: usize = 60;

/// `(a, b, c)` -> `"a, b, c"`, or `none` for an empty list that is a real empty set. A list longer
/// than [`MAX_LIST_ITEMS`] shows the first ones and ends `(+N more not listed)`.
fn list_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_string()
    } else if items.len() > MAX_LIST_ITEMS {
        format!("{} (+{} more not listed)", items[..MAX_LIST_ITEMS].join(", "), items.len() - MAX_LIST_ITEMS)
    } else {
        items.join(", ")
    }
}

/// Fixed-precision float text (determinism: no shortest-round-trip surprises across versions).
fn f1(v: f64) -> String {
    format!("{v:.1}")
}

/// Cuts `lines` to [`MAX_DOC_BYTES`] at a line boundary and says so (module doc).
fn finish(mut lines: Vec<String>) -> String {
    let total: usize = lines.iter().map(|l| l.len() + 1).sum();
    if total <= MAX_DOC_BYTES {
        return lines.join("\n");
    }
    // Reserve room for the marker line (at most ~40 bytes).
    let budget = MAX_DOC_BYTES - 48;
    let mut used = 0usize;
    let mut keep = 0usize;
    for l in &lines {
        // A single line longer than the whole budget is cut by characters below.
        if used + l.len() + 1 > budget {
            break;
        }
        used += l.len() + 1;
        keep += 1;
    }
    let dropped = lines.len() - keep;
    lines.truncate(keep);
    if keep == 0 {
        // Pathological: even the first line is huge. Cut it by characters.
        return format!("[truncated: document cut at {} bytes]", MAX_DOC_BYTES);
    }
    lines.push(format!("[truncated: {dropped} more lines]"));
    lines.join("\n")
}

/// Index of lookups the renderers share.
struct Ctx<'a> {
    w: &'a WorldInput,
    faction_name: BTreeMap<i64, &'a str>,
    settlement_name: BTreeMap<i64, &'a str>,
    province_name: BTreeMap<i64, &'a str>,
    continent_name: BTreeMap<i64, &'a str>,
}

impl<'a> Ctx<'a> {
    fn new(w: &'a WorldInput) -> Self {
        Ctx {
            w,
            faction_name: w.factions.iter().map(|f| (f.id, f.name.as_str())).collect(),
            settlement_name: w.settlements.iter().map(|s| (s.id, s.name.as_str())).collect(),
            province_name: w.provinces.iter().map(|p| (p.id, p.name.as_str())).collect(),
            continent_name: w.continents.iter().map(|c| (c.id, c.name.as_str())).collect(),
        }
    }

    /// `Name (W:faction:3)` or, for an id with no row, just the id (never an invented name).
    fn faction_ref(&self, id: i64) -> String {
        match self.faction_name.get(&id) {
            Some(n) => format!("{n} ({})", world_id("faction", id)),
            None => world_id("faction", id),
        }
    }

    fn settlement_ref(&self, id: i64) -> String {
        match self.settlement_name.get(&id) {
            Some(n) => format!("{n} ({})", world_id("settlement", id)),
            None => world_id("settlement", id),
        }
    }

    /// Factions (ids) whose religion key is `key`, sorted by name then id.
    fn factions_with(&self, pred: impl Fn(&FactionIn) -> bool) -> Vec<&'a FactionIn> {
        let mut v: Vec<&FactionIn> = self.w.factions.iter().filter(|f| pred(f)).collect();
        v.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
        v
    }
}

fn sorted_by_name_settlements<'a>(mut v: Vec<&'a SettlementIn>) -> Vec<&'a SettlementIn> {
    v.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    v
}

/// Per-pair aggregation of a trade list: key `(min, max)` -> flows in either direction.
fn pair_flows(trade: &[TradeFlowIn]) -> BTreeMap<(i64, i64), Vec<&TradeFlowIn>> {
    let mut m: BTreeMap<(i64, i64), Vec<&TradeFlowIn>> = BTreeMap::new();
    for f in trade {
        let key = (f.from.min(f.to), f.from.max(f.to));
        m.entry(key).or_default().push(f);
    }
    m
}

// ---------------------------------------------------------------------------------- renderers

fn render_settlement(c: &Ctx, s: &SettlementIn, pairs: Option<&BTreeMap<(i64, i64), Vec<&TradeFlowIn>>>) -> Document {
    let id = world_id("settlement", s.id);
    let mut l: Vec<String> = Vec::new();
    l.push(format!("Settlement: {}", s.name));
    l.push(format!("id: {id}"));
    l.push(format!("kind: {}", s.kind));
    l.push(format!("population: {}", s.population));
    l.push(format!("position: cell ({}, {})", s.x, s.y));
    l.push(format!("capital: {}", if s.capital { "yes" } else { "no" }));
    l.push(format!("coastal: {}", if s.coastal { "yes" } else { "no" }));
    if s.faction > 0 {
        l.push(format!("owner faction: {}", c.faction_ref(s.faction)));
    } else {
        l.push("owner faction: not available (no faction holds this settlement)".to_string());
    }
    let prov = s.province.map(|p| c.province_name.get(&p).map_or_else(|| world_id("province", p), |n| format!("{n} ({})", world_id("province", p))));
    opt_line(&mut l, "province", prov, &s.absent, "province", "no province covers its cell");
    let cont = s.continent.map(|p| c.continent_name.get(&p).map_or_else(|| world_id("continent", p), |n| format!("{n} ({})", world_id("continent", p))));
    opt_line(&mut l, "continent", cont, &s.absent, "continent", "the landmass labelling is not available");
    opt_line(&mut l, "plurality religion", s.religion.as_deref(), &s.absent, "religion", "the belief layer has not run");
    let river = s.river_order.map(|o| if o == 0 { "none (no traced river at the site)".to_string() } else { format!("river of order {o}") });
    opt_line(&mut l, "river at site", river, &s.absent, "river_order", "the site explanation is not available");
    // Trade partners, from both directions of every flow touching this settlement.
    match pairs {
        None => l.push("trade partners: not available (the trade match was not computed)".to_string()),
        Some(p) => {
            let mut rows: Vec<(String, String)> = Vec::new();
            for ((a, b), flows) in p.iter() {
                if *a != s.id && *b != s.id {
                    continue;
                }
                let other = if *a == s.id { *b } else { *a };
                let mut sends: Vec<String> = Vec::new();
                let mut recvs: Vec<String> = Vec::new();
                let mut vol = 0.0;
                for f in flows {
                    vol += f.volume;
                    if f.from == s.id {
                        sends.push(f.good.clone());
                    } else {
                        recvs.push(f.good.clone());
                    }
                }
                sends.sort();
                sends.dedup();
                recvs.sort();
                recvs.dedup();
                let mut parts = Vec::new();
                if !sends.is_empty() {
                    parts.push(format!("sends {}", sends.join("/")));
                }
                if !recvs.is_empty() {
                    parts.push(format!("receives {}", recvs.join("/")));
                }
                let name = c.settlement_name.get(&other).copied().unwrap_or("").to_string();
                rows.push((format!("{name}\u{0}{other:020}"), format!("{} [{}; volume {}]", c.settlement_ref(other), parts.join(", "), f1(vol))));
            }
            rows.sort();
            if rows.is_empty() {
                l.push("trade partners: none (the trade match found no flow touching it)".to_string());
            } else {
                l.push(format!("trade partners ({}):", rows.len()));
                for (_, r) in rows {
                    l.push(format!("- {r}"));
                }
            }
        }
    }
    // Landmarks that name this settlement as nearest.
    let mut near: Vec<String> = c
        .w
        .landmarks
        .iter()
        .filter(|m| m.nearest_settlement.is_some_and(|(sid, _)| sid == s.id))
        .map(|m| format!("{} ({})", m.name.clone().unwrap_or_else(|| m.kind.clone()), world_id("landmark", m.index)))
        .collect();
    near.sort();
    if !near.is_empty() {
        l.push(format!("landmarks nearby: {}", near.join(", ")));
    }
    Document {
        id,
        source: Source::World,
        kind: "settlement".to_string(),
        title: s.name.clone(),
        locator: format!("settlement {}", s.name),
        header: format!("World data | settlement | {} | {}", s.kind, c.faction_name.get(&s.faction).copied().unwrap_or("")),
        body: finish(l),
    }
}

fn render_faction(c: &Ctx, f: &FactionIn) -> Document {
    let id = world_id("faction", f.id);
    let mut l: Vec<String> = Vec::new();
    l.push(format!("Faction: {}", f.name));
    l.push(format!("id: {id}"));
    l.push(format!("culture: {}", f.culture));
    l.push(format!("religion: {} ({})", f.religion_label, world_id("religion", &f.religion_key)));
    l.push(format!("government: {} ({})", f.government_label, world_id("government", &f.government_key)));
    l.push(format!("agricultural technology: {}", f.ag_tech));
    let mine = sorted_by_name_settlements(c.w.settlements.iter().filter(|s| s.faction == f.id).collect());
    let pop: u64 = mine.iter().map(|s| s.population as u64).sum();
    l.push(format!("settlements: {}", mine.len()));
    l.push(format!("population: {pop}"));
    opt_line(&mut l, "claimed cells", f.claimed_cells, &f.absent, "claimed_cells", "the territory grid is not known");
    if let Some(cap) = mine.iter().find(|s| s.capital) {
        l.push(format!("capital: {}", c.settlement_ref(cap.id)));
    } else {
        l.push("capital: not available (no capital settlement is recorded for this faction)".to_string());
    }
    let names: Vec<String> = mine.iter().map(|s| c.settlement_ref(s.id)).collect();
    l.push(format!("settlement list: {}", list_or_none(&names)));
    let mut provs: Vec<&ProvinceIn> = c.w.provinces.iter().filter(|p| p.faction == f.id).collect();
    provs.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    if !provs.is_empty() {
        let p: Vec<String> = provs.iter().map(|p| format!("{} ({})", p.name, world_id("province", p.id))).collect();
        l.push(format!("provinces: {}", p.join(", ")));
    }
    // Relations, from this faction's side.
    let mut rel: Vec<(String, String)> = Vec::new();
    for r in &c.w.relations {
        let other = if r.a == f.id { r.b } else if r.b == f.id { r.a } else { continue };
        let desc = match (&r.stance, r.value) {
            (Some(s), Some(v)) => format!("{s} (value {v:.2})"),
            (Some(s), None) => s.clone(),
            _ => format!("not available ({})", r.absent.get("stance").map_or("no claim grid", |x| x.as_str())),
        };
        rel.push((c.faction_name.get(&other).copied().unwrap_or("").to_string(), format!("{}: {desc}", c.faction_ref(other))));
    }
    rel.sort();
    if !rel.is_empty() {
        l.push("relations:".to_string());
        for (_, r) in rel {
            l.push(format!("- {r}"));
        }
    }
    let wars: Vec<String> = c
        .w
        .conflicts
        .iter()
        .filter(|k| k.parties.contains(&f.id))
        .map(|k| format!("{} ({})", k.name, world_id("conflict", k.id)))
        .collect();
    if !wars.is_empty() {
        l.push(format!("conflicts: {}", wars.join(", ")));
    }
    Document {
        id,
        source: Source::World,
        kind: "faction".to_string(),
        title: f.name.clone(),
        locator: format!("faction {}", f.name),
        header: format!("World data | faction | {} | {} | {}", f.culture, f.religion_label, f.government_label),
        body: finish(l),
    }
}

fn render_religion(c: &Ctx, r: &ReligionIn) -> Document {
    let id = world_id("religion", &r.key);
    let mut l = vec![format!("Religion: {}", r.label), format!("id: {id}"), format!("key: {}", r.key)];
    if r.custom {
        l.push(format!("origin: custom religion behaving as {}", r.base_key.as_deref().unwrap_or("an unknown base")));
    } else {
        l.push("origin: built-in".to_string());
    }
    opt_line(&mut l, "notes", r.notes.as_deref().filter(|n| !n.is_empty()), &r.absent, "notes", "none were written");
    if !r.axes.is_empty() {
        let a: Vec<String> = r.axes.iter().map(|(k, v)| format!("{k}={v}")).collect();
        l.push(format!("axes: {}", a.join(", ")));
    }
    let adherents = c.factions_with(|f| f.religion_key == r.key);
    let names: Vec<String> = adherents.iter().map(|f| c.faction_ref(f.id)).collect();
    l.push(format!("state religion of factions: {}", list_or_none(&names)));
    Document {
        id,
        source: Source::World,
        kind: "religion".to_string(),
        title: r.label.clone(),
        locator: format!("religion {}", r.label),
        header: format!("World data | religion | {}", if r.custom { "custom" } else { "built-in" }),
        body: finish(l),
    }
}

fn render_government(c: &Ctx, g: &GovernmentIn) -> Document {
    let id = world_id("government", &g.key);
    let mut l = vec![format!("Government: {}", g.label), format!("id: {id}"), format!("key: {}", g.key)];
    opt_line(&mut l, "group", g.group.as_deref(), &g.absent, "group", "this form is not in a display group");
    opt_line(&mut l, "description", g.hint.as_deref(), &g.absent, "hint", "the vocabulary row carries none");
    let users = c.factions_with(|f| f.government_key == g.key);
    let names: Vec<String> = users.iter().map(|f| c.faction_ref(f.id)).collect();
    l.push(format!("factions governed this way: {}", list_or_none(&names)));
    Document {
        id,
        source: Source::World,
        kind: "government".to_string(),
        title: g.label.clone(),
        locator: format!("government {}", g.label),
        header: "World data | government".to_string(),
        body: finish(l),
    }
}

fn render_culture(c: &Ctx, cu: &CultureIn) -> Document {
    let id = world_id("culture", &cu.key);
    let mut l = vec![format!("Culture: {}", cu.name), format!("id: {id}"), format!("key: {}", cu.key)];
    opt_line(&mut l, "terrain affinity", cu.terrain_affinity.as_deref().filter(|t| !t.is_empty()), &cu.absent, "terrain_affinity", "the culture table gives this culture no terrain pairing");
    let users = c.factions_with(|f| f.culture == cu.key);
    let names: Vec<String> = users.iter().map(|f| c.faction_ref(f.id)).collect();
    l.push(format!("factions of this culture: {}", list_or_none(&names)));
    let pop: u64 = users.iter().map(|f| c.w.settlements.iter().filter(|s| s.faction == f.id).map(|s| s.population as u64).sum::<u64>()).sum();
    let places: usize = users.iter().map(|f| c.w.settlements.iter().filter(|s| s.faction == f.id).count()).sum();
    l.push(format!("settlements: {places}"));
    l.push(format!("population: {pop}"));
    Document {
        id,
        source: Source::World,
        kind: "culture".to_string(),
        title: cu.name.clone(),
        locator: format!("culture {}", cu.name),
        header: "World data | culture".to_string(),
        body: finish(l),
    }
}

fn render_province(c: &Ctx, p: &ProvinceIn) -> Document {
    let id = world_id("province", p.id);
    let mut l = vec![format!("Province: {}", p.name), format!("id: {id}")];
    if p.faction > 0 {
        l.push(format!("owner faction: {}", c.faction_ref(p.faction)));
    } else {
        l.push("owner faction: not available (no faction holds this province)".to_string());
    }
    opt_line(&mut l, "capital", p.capital.map(|s| c.settlement_ref(s)), &p.absent, "capital", "the seed settlement does not resolve");
    let members = sorted_by_name_settlements(c.w.settlements.iter().filter(|s| s.province == Some(p.id)).collect());
    let names: Vec<String> = members.iter().map(|s| c.settlement_ref(s.id)).collect();
    l.push(format!("settlements in the province: {}", list_or_none(&names)));
    Document {
        id,
        source: Source::World,
        kind: "province".to_string(),
        title: p.name.clone(),
        locator: format!("province {}", p.name),
        header: format!("World data | province | {}", c.faction_name.get(&p.faction).copied().unwrap_or("")),
        body: finish(l),
    }
}

fn render_continent(c: &Ctx, k: &ContinentIn) -> Document {
    let id = world_id("continent", k.id);
    let mut l = vec![format!("Continent: {}", k.name), format!("id: {id}"), format!("cells: {}", k.cells)];
    if k.faction > 0 {
        l.push(format!("main faction: {}", c.faction_ref(k.faction)));
    } else {
        l.push("main faction: not available (no faction claims land here)".to_string());
    }
    let members = sorted_by_name_settlements(c.w.settlements.iter().filter(|s| s.continent == Some(k.id)).collect());
    let names: Vec<String> = members.iter().map(|s| c.settlement_ref(s.id)).collect();
    l.push(format!("settlements on the continent: {}", list_or_none(&names)));
    Document {
        id,
        source: Source::World,
        kind: "continent".to_string(),
        title: k.name.clone(),
        locator: format!("continent {}", k.name),
        header: "World data | continent".to_string(),
        body: finish(l),
    }
}

fn render_relation(c: &Ctx, r: &RelationIn) -> Document {
    let id = world_id("relation", format!("{}-{}", r.a, r.b));
    let an = c.faction_name.get(&r.a).copied().unwrap_or("");
    let bn = c.faction_name.get(&r.b).copied().unwrap_or("");
    let mut l = vec![format!("Relation between {an} and {bn}"), format!("id: {id}"), format!("first faction: {}", c.faction_ref(r.a)), format!("second faction: {}", c.faction_ref(r.b))];
    opt_line(&mut l, "stance", r.stance.as_deref(), &r.absent, "stance", "no claim grid, so no border or trade term");
    opt_line(&mut l, "value", r.value.map(|v| format!("{v:.2}")), &r.absent, "value", "no claim grid, so no border or trade term");
    opt_line(&mut l, "shared border cells", r.border_cells, &r.absent, "border_cells", "no claim grid");
    Document {
        id,
        source: Source::World,
        kind: "relation".to_string(),
        title: format!("{an} and {bn}"),
        locator: format!("relation {an} / {bn}"),
        header: "World data | relation | diplomacy".to_string(),
        body: finish(l),
    }
}

fn render_trade_pair(c: &Ctx, key: (i64, i64), flows: &[&TradeFlowIn]) -> Document {
    let id = world_id("trade", format!("{}-{}", key.0, key.1));
    let an = c.settlement_name.get(&key.0).copied().unwrap_or("");
    let bn = c.settlement_name.get(&key.1).copied().unwrap_or("");
    let mut l = vec![format!("Trade between {an} and {bn}"), format!("id: {id}"), format!("first settlement: {}", c.settlement_ref(key.0)), format!("second settlement: {}", c.settlement_ref(key.1))];
    let mut rows: Vec<String> = flows
        .iter()
        .map(|f| format!("- {} sends {} to {} by {}: volume {}, {} km", c.settlement_ref(f.from), f.good, c.settlement_ref(f.to), f.mode, f1(f.volume), f1(f.distance_km)))
        .collect();
    rows.sort();
    l.push(format!("flows ({}):", rows.len()));
    l.extend(rows);
    Document {
        id,
        source: Source::World,
        kind: "trade".to_string(),
        title: format!("{an} and {bn} trade"),
        locator: format!("trade {an} / {bn}"),
        header: "World data | trade".to_string(),
        body: finish(l),
    }
}

fn render_landmark(c: &Ctx, m: &LandmarkIn) -> Document {
    let id = world_id("landmark", m.index);
    let title = m.name.clone().unwrap_or_else(|| m.kind.clone());
    let mut l = vec![format!("Landmark: {title}"), format!("id: {id}"), format!("kind: {}", m.kind), format!("position: cell ({}, {})", m.x, m.y)];
    opt_line(&mut l, "region", m.region.as_deref(), &m.absent, "region", "no province covers its cell");
    opt_line(&mut l, "nearest settlement", m.nearest_settlement.map(|(s, d)| format!("{} at {:.1} cells", c.settlement_ref(s), d)), &m.absent, "nearest_settlement", "the world has no settlement");
    opt_line(&mut l, "cause", m.cause.as_deref(), &m.absent, "cause", "the placement run recorded none");
    Document {
        id,
        source: Source::World,
        kind: "landmark".to_string(),
        title,
        locator: format!("landmark {} at ({}, {})", m.kind, m.x, m.y),
        header: format!("World data | landmark | {}", m.kind),
        body: finish(l),
    }
}

fn render_river(r: &RiverIn) -> Document {
    let id = world_id("river", r.index);
    let mut l = vec![
        format!("River {} (unnamed; order {})", r.index, r.order),
        format!("id: {id}"),
        format!("length: {} km", f1(r.km)),
        format!("tributaries: {}", r.tributaries),
        format!("head: cell ({}, {})", r.head.0, r.head.1),
        format!("mouth: cell ({}, {})", r.mouth.0, r.mouth.1),
    ];
    opt_line(&mut l, "source elevation", r.source_m.map(|v| format!("{} m", f1(v))), &r.absent, "source_m", "no elevation sample");
    opt_line(&mut l, "mouth elevation", r.mouth_m.map(|v| format!("{} m", f1(v))), &r.absent, "mouth_m", "no elevation sample");
    Document {
        id,
        source: Source::World,
        kind: "river".to_string(),
        title: format!("River {}", r.index),
        locator: format!("river {}", r.index),
        header: format!("World data | river | order {}", r.order),
        body: finish(l),
    }
}

fn render_volcano(v: &VolcanoIn) -> Document {
    let id = world_id("volcano", v.id);
    let mut l = vec![format!("Volcano {}", v.id), format!("id: {id}")];
    opt_line(&mut l, "kind", v.kind.as_deref(), &v.absent, "kind", "the reference edifice model records no morphology");
    l.push(format!("position: cell ({}, {})", v.x, v.y));
    opt_line(&mut l, "age", v.age_class.as_deref(), &v.absent, "age_class", "the geology table records none");
    opt_line(&mut l, "activity", v.state.as_deref(), &v.absent, "state", "the geology table records none");
    let kind = v.kind.clone().unwrap_or_else(|| "unclassified".to_string());
    Document {
        id,
        source: Source::World,
        kind: "volcano".to_string(),
        title: format!("Volcano {} ({kind})", v.id),
        locator: format!("volcano {}", v.id),
        header: format!("World data | volcano | {kind}"),
        body: finish(l),
    }
}

fn render_conflict(c: &Ctx, k: &ConflictIn) -> Document {
    let id = world_id("conflict", k.id);
    let mut l = vec![format!("Conflict: {}", k.name), format!("id: {id}"), format!("kind: {}", k.kind)];
    let sides: Vec<String> = k.parties.iter().map(|p| c.faction_ref(*p)).collect();
    l.push(format!("parties: {}", list_or_none(&sides)));
    l.push(format!("started: year {}", k.start_year));
    opt_line(&mut l, "ended", k.end_year.map(|y| format!("year {y}")), &k.absent, "end_year", "it is still ongoing or has no recorded end");
    opt_line(&mut l, "notes", k.notes.as_deref().filter(|n| !n.is_empty()), &k.absent, "notes", "none were written");
    Document {
        id,
        source: Source::World,
        kind: "conflict".to_string(),
        title: k.name.clone(),
        locator: format!("conflict {}", k.name),
        header: format!("World data | conflict | {}", k.kind),
        body: finish(l),
    }
}

fn render_journey(j: &JourneyIn) -> Document {
    let id = world_id("journey", j.id);
    let l = vec![
        format!("Journey: {}", j.name),
        format!("id: {id}"),
        format!("party preset: {}", j.party_preset),
        format!("starts: year {}", j.start_year),
        format!("route: {} km by {}", f1(j.distance_km), j.mode),
    ];
    Document {
        id,
        source: Source::World,
        kind: "journey".to_string(),
        title: j.name.clone(),
        locator: format!("journey {}", j.name),
        header: "World data | journey".to_string(),
        body: finish(l),
    }
}

/// The five largest settlements of `v` as `"Name (pop N)"`, largest first, ties by name.
fn biggest(v: &[&SettlementIn]) -> Vec<String> {
    let mut w: Vec<&&SettlementIn> = v.iter().collect();
    w.sort_by(|a, b| b.population.cmp(&a.population).then(a.name.cmp(&b.name)).then(a.id.cmp(&b.id)));
    w.iter().take(5).map(|s| format!("{} (population {})", s.name, s.population)).collect()
}

/// Rollup ("community summary") documents, derived from the input alone (s.2 row 10).
fn render_rollups(c: &Ctx) -> Vec<Document> {
    let mut out = Vec::new();
    let w = c.w;
    // The whole world.
    {
        let all: Vec<&SettlementIn> = w.settlements.iter().collect();
        let total: u64 = all.iter().map(|s| s.population as u64).sum();
        let mut l = vec!["World overview".to_string(), format!("id: {}", world_id("rollup", "world")), format!("seed: {}", w.seed)];
        l.push(format!("settlements: {}", all.len()));
        l.push(format!("population: {total}"));
        l.push(format!("factions: {}", w.factions.len()));
        l.push(format!("provinces: {}", w.provinces.len()));
        l.push(format!("continents: {}", w.continents.len()));
        l.push(format!("largest settlements: {}", list_or_none(&biggest(&all))));
        out.push(Document { id: world_id("rollup", "world"), source: Source::World, kind: "rollup".to_string(), title: "World overview".to_string(), locator: "world overview".to_string(), header: "World data | rollup | world".to_string(), body: finish(l) });
    }
    for f in &w.factions {
        let mine: Vec<&SettlementIn> = w.settlements.iter().filter(|s| s.faction == f.id).collect();
        let pop: u64 = mine.iter().map(|s| s.population as u64).sum();
        let mut faiths: Vec<String> = mine.iter().filter_map(|s| s.religion.clone()).collect();
        faiths.sort();
        faiths.dedup();
        let id = world_id("rollup", format!("faction:{}", f.id));
        let mut l = vec![format!("Faction overview: {}", f.name), format!("id: {id}"), format!("settlements: {}", mine.len()), format!("population: {pop}")];
        l.push(format!("largest settlements: {}", list_or_none(&biggest(&mine))));
        l.push(format!("religions present in its settlements: {}", list_or_none(&faiths)));
        out.push(Document { id, source: Source::World, kind: "rollup".to_string(), title: format!("{} overview", f.name), locator: format!("overview of faction {}", f.name), header: "World data | rollup | faction".to_string(), body: finish(l) });
    }
    for p in &w.provinces {
        let mine: Vec<&SettlementIn> = w.settlements.iter().filter(|s| s.province == Some(p.id)).collect();
        let pop: u64 = mine.iter().map(|s| s.population as u64).sum();
        let id = world_id("rollup", format!("province:{}", p.id));
        let l = vec![format!("Province overview: {}", p.name), format!("id: {id}"), format!("settlements: {}", mine.len()), format!("population: {pop}"), format!("largest settlements: {}", list_or_none(&biggest(&mine)))];
        out.push(Document { id, source: Source::World, kind: "rollup".to_string(), title: format!("{} overview", p.name), locator: format!("overview of province {}", p.name), header: "World data | rollup | province".to_string(), body: finish(l) });
    }
    for k in &w.continents {
        let mine: Vec<&SettlementIn> = w.settlements.iter().filter(|s| s.continent == Some(k.id)).collect();
        let pop: u64 = mine.iter().map(|s| s.population as u64).sum();
        let id = world_id("rollup", format!("continent:{}", k.id));
        let l = vec![format!("Continent overview: {}", k.name), format!("id: {id}"), format!("settlements: {}", mine.len()), format!("population: {pop}"), format!("largest settlements: {}", list_or_none(&biggest(&mine)))];
        out.push(Document { id, source: Source::World, kind: "rollup".to_string(), title: format!("{} overview", k.name), locator: format!("overview of continent {}", k.name), header: "World data | rollup | continent".to_string(), body: finish(l) });
    }
    out
}

/// Renders every document of `w`, sorted by id. Pure and deterministic (module doc).
pub fn render_documents(w: &WorldInput) -> Vec<Document> {
    let c = Ctx::new(w);
    let pairs = w.trade.as_ref().map(|t| pair_flows(t));
    let mut docs: Vec<Document> = Vec::new();
    for s in &w.settlements {
        docs.push(render_settlement(&c, s, pairs.as_ref()));
    }
    for f in &w.factions {
        docs.push(render_faction(&c, f));
    }
    for r in &w.religions {
        docs.push(render_religion(&c, r));
    }
    for g in &w.governments {
        docs.push(render_government(&c, g));
    }
    for cu in &w.cultures {
        docs.push(render_culture(&c, cu));
    }
    for p in &w.provinces {
        docs.push(render_province(&c, p));
    }
    for k in &w.continents {
        docs.push(render_continent(&c, k));
    }
    for r in &w.relations {
        docs.push(render_relation(&c, r));
    }
    if let Some(p) = pairs.as_ref() {
        for (k, flows) in p {
            docs.push(render_trade_pair(&c, *k, flows));
        }
    }
    for m in &w.landmarks {
        docs.push(render_landmark(&c, m));
    }
    for r in &w.rivers {
        docs.push(render_river(r));
    }
    for v in &w.volcanoes {
        docs.push(render_volcano(v));
    }
    for k in &w.conflicts {
        docs.push(render_conflict(&c, k));
    }
    for j in &w.journeys {
        docs.push(render_journey(j));
    }
    docs.extend(render_rollups(&c));
    docs.sort_by(|a, b| a.id.cmp(&b.id));
    docs
}

/// The directed relations between world documents (s.4.4): the edges link expansion walks. Sorted,
/// de-duplicated, and only between ids that have a document (a dangling reference is not an edge).
pub fn world_edges(w: &WorldInput) -> Vec<WorldEdge> {
    let mut edges: Vec<WorldEdge> = Vec::new();
    let has_f: std::collections::BTreeSet<i64> = w.factions.iter().map(|f| f.id).collect();
    let has_s: std::collections::BTreeSet<i64> = w.settlements.iter().map(|s| s.id).collect();
    let has_p: std::collections::BTreeSet<i64> = w.provinces.iter().map(|p| p.id).collect();
    let has_k: std::collections::BTreeSet<i64> = w.continents.iter().map(|c| c.id).collect();
    let has_rel: std::collections::BTreeSet<&str> = w.religions.iter().map(|r| r.key.as_str()).collect();
    let has_gov: std::collections::BTreeSet<&str> = w.governments.iter().map(|g| g.key.as_str()).collect();
    let has_cul: std::collections::BTreeSet<&str> = w.cultures.iter().map(|g| g.key.as_str()).collect();
    let mut push = |from: String, rel: &'static str, to: String| edges.push(WorldEdge { from, rel, to });
    for s in &w.settlements {
        let me = world_id("settlement", s.id);
        if has_f.contains(&s.faction) {
            push(me.clone(), "owned_by", world_id("faction", s.faction));
        }
        if let Some(p) = s.province.filter(|p| has_p.contains(p)) {
            push(me.clone(), "located_in", world_id("province", p));
        }
        if let Some(k) = s.continent.filter(|k| has_k.contains(k)) {
            push(me.clone(), "located_in", world_id("continent", k));
        }
    }
    for f in &w.factions {
        let me = world_id("faction", f.id);
        if has_rel.contains(f.religion_key.as_str()) {
            push(me.clone(), "adheres_to", world_id("religion", &f.religion_key));
        }
        if has_gov.contains(f.government_key.as_str()) {
            push(me.clone(), "governed_by", world_id("government", &f.government_key));
        }
        if has_cul.contains(f.culture.as_str()) {
            push(me.clone(), "has_culture", world_id("culture", &f.culture));
        }
    }
    for p in &w.provinces {
        if has_f.contains(&p.faction) {
            push(world_id("province", p.id), "member_of", world_id("faction", p.faction));
        }
    }
    if let Some(t) = w.trade.as_ref() {
        for ((a, b), _) in pair_flows(t) {
            if has_s.contains(&a) && has_s.contains(&b) && a != b {
                push(world_id("settlement", a), "trades_with", world_id("settlement", b));
            }
        }
    }
    for r in &w.relations {
        let rel = match r.stance.as_deref() {
            Some("hostile") => Some("at_war_with"),
            Some("allied") => Some("allied_with"),
            _ => None,
        };
        if let Some(rel) = rel {
            if has_f.contains(&r.a) && has_f.contains(&r.b) {
                push(world_id("faction", r.a), rel, world_id("faction", r.b));
            }
        }
    }
    for m in &w.landmarks {
        if let Some((sid, _)) = m.nearest_settlement.filter(|(s, _)| has_s.contains(s)) {
            push(world_id("landmark", m.index), "near_landmark", world_id("settlement", sid));
        }
    }
    for k in &w.conflicts {
        for p in k.parties.iter().filter(|p| has_f.contains(p)) {
            push(world_id("faction", *p), "party_to", world_id("conflict", k.id));
        }
    }
    edges.sort();
    edges.dedup();
    edges
}

/// The map from a Vault `entity="<kind>:<id>"` key to the world document it names, for every entity
/// that HAS a document (a key naming nothing is dangling and joins nothing). Settlement, province,
/// continent and faction ids are the same numbers the Vault uses (`vault_bridge.rs`: a settlement is
/// keyed by `tid`); a culture is its 0-based table index; a landmark its derived entity id.
pub fn entity_keys(w: &WorldInput) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    for s in &w.settlements {
        m.insert(format!("settlement:{}", s.id), world_id("settlement", s.id));
    }
    for p in &w.provinces {
        m.insert(format!("province:{}", p.id), world_id("province", p.id));
    }
    for k in &w.continents {
        m.insert(format!("continent:{}", k.id), world_id("continent", k.id));
    }
    for f in &w.factions {
        m.insert(format!("faction:{}", f.id), world_id("faction", f.id));
    }
    for c in &w.cultures {
        m.insert(format!("culture:{}", c.index), world_id("culture", &c.key));
    }
    for l in &w.landmarks {
        if let Some(e) = l.entity_id {
            m.insert(format!("landmark:{e}"), world_id("landmark", l.index));
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small literal world, written by hand so the expected text is known without running
    /// anything. NOT a stand-in for a real world: the bridge's three-seed tests build real ones.
    fn tiny() -> WorldInput {
        WorldInput {
            seed: 7,
            settlements: vec![
                SettlementIn { id: 11, name: "Kelvhold".into(), kind: "city".into(), population: 6115, x: 4, y: 5, faction: 1, capital: true, coastal: true, province: Some(2), continent: Some(1), religion: Some("The Ashfall".into()), river_order: Some(0), absent: Absent::new() },
                SettlementIn { id: 12, name: "Rhenmark".into(), kind: "village".into(), population: 0, x: 9, y: 9, faction: 1, capital: false, coastal: false, province: None, continent: None, religion: None, river_order: None, absent: Absent::new() },
            ],
            factions: vec![FactionIn { id: 1, name: "Order of Dawn".into(), culture: "nordic".into(), religion_label: "The Ashfall".into(), religion_key: "custom:1".into(), government_key: "monarchy".into(), government_label: "Monarchy".into(), ag_tech: "iron".into(), claimed_cells: None, absent: Absent::new() }],
            religions: vec![ReligionIn { key: "custom:1".into(), label: "The Ashfall".into(), custom: true, base_key: Some("animism".into()), notes: None, axes: BTreeMap::new(), absent: Absent::new() }],
            governments: vec![GovernmentIn { key: "monarchy".into(), label: "Monarchy".into(), hint: None, group: None, absent: Absent::new() }],
            cultures: vec![CultureIn { index: 3, key: "nordic".into(), name: "Nordic".into(), terrain_affinity: None, absent: Absent::new() }],
            provinces: vec![ProvinceIn { id: 2, name: "Dawnmarch".into(), faction: 1, capital: Some(11), absent: Absent::new() }],
            continents: vec![ContinentIn { id: 1, name: "Veldt".into(), cells: 900, faction: 1, absent: Absent::new() }],
            trade: Some(vec![TradeFlowIn { from: 11, to: 12, good: "grain".into(), mode: "land".into(), volume: 3.26, distance_km: 40.0 }]),
            ..Default::default()
        }
    }

    fn doc<'a>(docs: &'a [Document], id: &str) -> &'a Document {
        docs.iter().find(|d| d.id == id).unwrap_or_else(|| panic!("no document {id}"))
    }

    /// Protects the settlement template on a hand-made row: every figure equals its input, the
    /// citation id is present, and trade is rendered from BOTH endpoints (the partner appears in
    /// each other's document). MUTATION: swap `population` for another field and the literal fails.
    #[test]
    fn a_settlement_renders_its_figures_and_both_ends_of_a_trade() {
        let docs = render_documents(&tiny());
        let k = doc(&docs, "W:settlement:11");
        assert!(k.body.contains("population: 6115"), "{}", k.body);
        assert!(k.body.contains("owner faction: Order of Dawn (W:faction:1)"));
        assert!(k.body.contains("province: Dawnmarch (W:province:2)"));
        assert!(k.body.contains("river at site: none (no traced river at the site)"));
        assert!(k.body.contains("Rhenmark (W:settlement:12) [sends grain; volume 3.3]"), "{}", k.body);
        let r = doc(&docs, "W:settlement:12");
        assert!(r.body.contains("Kelvhold (W:settlement:11) [receives grain; volume 3.3]"), "{}", r.body);
        assert_eq!(k.source, Source::World);
    }

    /// Protects "absent is not zero" (RK-1 bar, with its mutation): a village of population 0 renders
    /// `population: 0` (a real zero); an absent province, religion and river each render `not
    /// available (reason)` and NEVER a number or a placeholder word. MUTATION: make `opt_line` print
    /// `0` for `None` and the reason asserts below go red.
    #[test]
    fn absent_figures_say_not_available_with_a_reason_and_zero_stays_zero() {
        let docs = render_documents(&tiny());
        let r = doc(&docs, "W:settlement:12");
        assert!(r.body.contains("population: 0\n"), "{}", r.body);
        assert!(r.body.contains("province: not available (no province covers its cell)"), "{}", r.body);
        assert!(r.body.contains("plurality religion: not available (the belief layer has not run)"));
        assert!(r.body.contains("river at site: not available (the site explanation is not available)"));
        let f = doc(&docs, "W:faction:1");
        assert!(f.body.contains("claimed cells: not available (the territory grid is not known)"), "{}", f.body);
    }

    /// Protects the supplied reason: a row's own `absent` entry wins over the field default.
    #[test]
    fn a_rows_own_reason_wins_over_the_default() {
        let mut w = tiny();
        w.factions[0].absent.insert("claimed_cells".into(), "a reopened archive carried no rasters/territory.i32".into());
        let docs = render_documents(&w);
        assert!(doc(&docs, "W:faction:1").body.contains("claimed cells: not available (a reopened archive carried no rasters/territory.i32)"));
    }

    /// Protects the not-computed trade case: `trade: None` is "not available", `Some(empty)` is "none".
    #[test]
    fn trade_not_computed_differs_from_trade_with_no_flow() {
        let mut w = tiny();
        w.trade = None;
        let a = render_documents(&w);
        assert!(doc(&a, "W:settlement:11").body.contains("trade partners: not available (the trade match was not computed)"));
        w.trade = Some(vec![]);
        let b = render_documents(&w);
        assert!(doc(&b, "W:settlement:11").body.contains("trade partners: none (the trade match found no flow touching it)"));
        assert!(!b.iter().any(|d| d.kind == "trade"));
    }

    /// Protects determinism: two renders are identical, and documents come out sorted by id.
    #[test]
    fn two_renders_are_identical_and_sorted() {
        let w = tiny();
        let a = render_documents(&w);
        let b = render_documents(&w);
        assert_eq!(a, b);
        assert!(a.windows(2).all(|p| p[0].id < p[1].id));
        assert!(a.len() >= 10, "every kind with a row has a document: {}", a.len());
    }

    /// Protects the size cap as a literal and the visible cut. A settlement with 150 trade partners
    /// renders to about 10 KB (a line per partner), so it must be cut at a line boundary to under 6 000
    /// bytes and SAY so (MUTATION: allow twice the cap and the length assert goes red). A faction with
    /// 600 settlements keeps its list, shortened to [`MAX_LIST_ITEMS`] items with the count of the
    /// rest, rather than losing the whole line.
    #[test]
    fn an_oversized_document_is_cut_visibly_under_the_cap() {
        assert_eq!(MAX_DOC_BYTES, 6000);
        assert_eq!(MAX_LIST_ITEMS, 60);
        let mut w = tiny();
        let mut flows = Vec::new();
        for i in 0..150 {
            w.settlements.push(SettlementIn { id: 100 + i, name: format!("Trading Post {i:04}"), kind: "village".into(), population: 5, faction: 1, ..Default::default() });
            flows.push(TradeFlowIn { from: 11, to: 100 + i, good: "grain".into(), mode: "land".into(), volume: 1.0, distance_km: 5.0 });
        }
        w.trade = Some(flows);
        let docs = render_documents(&w);
        let hub = doc(&docs, "W:settlement:11");
        assert!(hub.body.len() <= MAX_DOC_BYTES, "{}", hub.body.len());
        assert!(hub.body.len() > MAX_DOC_BYTES - 200, "the cut must keep what fits, not discard the document: {}", hub.body.len());
        assert!(hub.body.contains("[truncated: "), "the cut must be announced");
        assert!(docs.iter().all(|d| d.body.len() <= MAX_DOC_BYTES));
        // A long inline list is shortened, with the remainder counted, not dropped as a whole line.
        let f = doc(&docs, "W:faction:1");
        assert!(f.body.contains("(+92 more not listed)"), "{}", f.body);
        assert!(f.body.contains("settlement list: "), "the list line survives");
    }

    /// Protects the edge set on the hand-made world, including that a dangling reference is not an
    /// edge (a settlement of an unknown faction gets no `owned_by`).
    #[test]
    fn edges_follow_the_rows_and_skip_dangling_references() {
        let mut w = tiny();
        w.settlements.push(SettlementIn { id: 13, name: "Ghostford".into(), faction: 99, ..Default::default() });
        let e = world_edges(&w);
        let has = |f: &str, r: &str, t: &str| e.iter().any(|x| x.from == f && x.rel == r && x.to == t);
        assert!(has("W:settlement:11", "owned_by", "W:faction:1"));
        assert!(has("W:settlement:11", "located_in", "W:province:2"));
        assert!(has("W:faction:1", "adheres_to", "W:religion:custom:1"));
        assert!(has("W:faction:1", "governed_by", "W:government:monarchy"));
        assert!(has("W:settlement:11", "trades_with", "W:settlement:12"));
        assert!(!e.iter().any(|x| x.from == "W:settlement:13" && x.rel == "owned_by"), "faction 99 has no row");
    }
}
