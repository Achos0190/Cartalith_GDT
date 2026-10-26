//! The world as retrievable documents: this crate's half of `RAG_KNOWLEDGE_BASE_RESEARCH.md`
//! milestone RK-1 (Ruling DM item 1b). `cartalith-knowledge` renders; this file reads the engine's
//! own state into that crate's plain input structs, once, deterministically.
//!
//! # Where each kind comes from (each re-opened at its symbol, 2026-10-26)
//!
//! | Kind | Source (the getter whose data it is) |
//! |---|---|
//! | settlement | `get_settlements` / `civ_settlement_details`: `CivData::settlements`, `explanations` (river order), `belief_current` (plurality religion), `provinces` raster |
//! | faction | `get_factions`: `faction_roster`, `religions`, `faction_claimed_cells` |
//! | religion | `civ_religion_vocabulary` + `civ_custom_religions`: `CIV_RELIGIONS`, `religion_profile`, `ReligionLibrary::defs` |
//! | government | `civ_government_vocabulary`: `CIV_GOVERNMENTS`, `GOVERNMENT_DISPLAY` |
//! | culture | `get_cultures`: `CIV_CULTURES`, `CIV_CULTURE_TERRAIN_KEY` |
//! | province / continent | `get_provinces` / `get_continents`: `province_list`, `continents` |
//! | relation | `civ_faction_relations`: the same `civ_faction_relations` over `MilWorld` aggregates |
//! | trade | `civ_trade_flows`: `trade_network_for`, the SAME match the dock runs |
//! | landmark | `landmarks`: the last `landmark_run()` result, if there is one |
//! | river | `get_rivers`: `river_entities` at order 3 and above (see [`RIVER_MIN_ORDER`]) |
//! | volcano | the geology table (`WorldState::volcanoes`, `volcano_attr`) |
//! | conflict / journey | `conflict_list` / `journey_list`: the Story layer's and the planner's own stores |
//!
//! # Absent means absent
//!
//! A figure the engine cannot give is `None` with a reason in the row's `absent` map (the renderer
//! prints `not available (reason)`), never `0`. The two cases that are easy to get wrong: a world
//! reopened without its rasters has no province or claim grid (`None`, not "no province"); a world
//! with no landmark run has no landmark documents (the list is empty, and a document is not
//! invented to say so).
//!
//! # Why a borrowed struct and not `&WorldGen`
//!
//! A `WorldGen` is a `GodotClass` and cannot be constructed under `cargo test`. [`KnowledgeWorld`]
//! borrows the same parts (`MilWorld`'s precedent), so the tests below run the SAME collection on
//! real generated worlds and recount each document's figures from `CivData` independently.
//!
//! # What it must never do
//!
//! Change the world (it only reads), call anything that mutates, hold a Godot type across a
//! thread, or return a figure it computed with a different rule from the getter it mirrors.

use std::collections::BTreeMap;

use godot::prelude::*;

use cartalith_knowledge::world::{
    Absent, ConflictIn, ContinentIn, CultureIn, FactionIn, GovernmentIn, JourneyIn, LandmarkIn, ProvinceIn, RelationIn, ReligionIn, RiverIn,
    SettlementIn, TradeFlowIn, VolcanoIn, WorldInput,
};

use cartalith_civ::religion_profile::AxisValue;
use crate::civ_military_bridge::{CivView, MilWorld};
use crate::{CivData, WorldGen, WorldSource};

/// The lowest Strahler order whose river is given a document. Rivers are unnamed and a world has
/// hundreds of order-1 runs; order 3 and above are the ones a person can ask about. A labelled
/// judgement, not an engine value; the lower orders are still in the engine (`get_rivers(1)`).
pub(crate) const RIVER_MIN_ORDER: i64 = 3;

/// The parts of a generated world the collection reads, borrowed (module doc).
pub(crate) struct KnowledgeWorld<'a> {
    pub(crate) civ: &'a CivData,
    pub(crate) ws: &'a cartalith_engine::WorldState,
    pub(crate) gw: usize,
    pub(crate) gh: usize,
    pub(crate) sea: f64,
    pub(crate) map_width_km: f64,
    pub(crate) seed: i32,
    pub(crate) world: bool,
    /// `WorldParams::peak_m`, for river elevations in metres.
    pub(crate) peak_m: f64,
}

/// The Story layer's and the planner's stores, borrowed; empty slices when there are none.
pub(crate) struct StoryParts<'a> {
    pub(crate) landmarks: &'a [cartalith_civ::landmark::Landmark],
    pub(crate) conflicts: &'a [cartalith_civ::conflict::Conflict],
    pub(crate) journeys: &'a [cartalith_civ::travel_library::Journey],
}

/// A title-cased culture key, exactly `get_cultures`' rule ("the key title-cased and nothing more").
fn culture_name(key: &str) -> String {
    let mut ch = key.chars();
    match ch.next() {
        Some(f) => f.to_uppercase().collect::<String>() + ch.as_str(),
        None => String::new(),
    }
}

/// The five axes of a built-in religion profile as `axis -> value key`, `base_axes_dict`'s rule.
fn builtin_axes(key: &str) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    let Some(p) = cartalith_civ::religion_profile::religion_profile(key) else { return m };
    if let Some(v) = p.object {
        m.insert("object".to_string(), v.key().to_string());
    }
    m.insert("exclusivity".to_string(), p.exclusivity.key().to_string());
    if let Some(v) = p.membership {
        m.insert("membership".to_string(), v.key().to_string());
    }
    if !p.locus.is_empty() {
        let l: Vec<&str> = p.locus.iter().map(|v| v.key()).collect();
        m.insert("locus".to_string(), l.join("+"));
    }
    if let Some(v) = p.institution {
        m.insert("institution".to_string(), v.key().to_string());
    }
    m
}

/// A custom religion's axis overrides as `axis -> value key`, `axes_dict`'s rule (unset omitted).
fn override_axes(o: &cartalith_civ::religion_profile::AxisOverrides) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    if let Some(v) = o.object {
        m.insert("object".to_string(), v.key().to_string());
    }
    if let Some(v) = o.exclusivity {
        m.insert("exclusivity".to_string(), v.key().to_string());
    }
    if let Some(v) = o.membership {
        m.insert("membership".to_string(), v.key().to_string());
    }
    if let Some(v) = o.locus {
        m.insert("locus".to_string(), v.key().to_string());
    }
    if let Some(v) = o.institution {
        m.insert("institution".to_string(), v.key().to_string());
    }
    m
}

fn absent1(field: &str, reason: &str) -> Absent {
    let mut a = Absent::new();
    a.insert(field.to_string(), reason.to_string());
    a
}

impl KnowledgeWorld<'_> {
    /// Collects every document kind this world can supply (module doc).
    pub(crate) fn input(&self, story: &StoryParts) -> WorldInput {
        let civ = self.civ;
        let grid_known = |len: usize| self.gw > 0 && self.gh > 0 && len == self.gw * self.gh;
        let provinces_known = grid_known(civ.provinces.len());
        let continent_of = self.continent_of_settlements();
        let belief = civ.belief_current();
        let label_of = |key: &str| {
            cartalith_civ::roster::CIV_RELIGIONS.iter().find(|(k, _)| *k == key).map_or_else(|| key.to_string(), |(_, l)| (*l).to_string())
        };

        let settlements: Vec<SettlementIn> = civ
            .settlements
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let cell = s.placement.y * self.gw + s.placement.x;
                let mut absent = Absent::new();
                let province = if provinces_known {
                    civ.provinces.get(cell).copied().filter(|p| *p > 0).map(i64::from)
                } else {
                    absent.insert("province".into(), "the province raster is not known (a reopened project without rasters/provinces.i32)".into());
                    None
                };
                let continent = continent_of.get(i).copied().flatten();
                if continent.is_none() {
                    absent.insert("continent".into(), "the settlement stands on a landmass below the continent size floor".into());
                }
                let religion = belief.and_then(|b| b.get(i)).map(|st| label_of(cartalith_civ::belief::religion_key(st.plurality()).unwrap_or("none")));
                SettlementIn {
                    id: s.tid as i64,
                    name: s.name.clone(),
                    kind: crate::journey_bridge::settlement_kind_key(s.placement.kind).to_string(),
                    population: s.pop,
                    x: s.placement.x as i64,
                    y: s.placement.y as i64,
                    faction: i64::from(s.placement.faction),
                    capital: s.placement.capital,
                    coastal: s.placement.coastal,
                    province,
                    continent,
                    religion,
                    river_order: civ.explanations.get(i).map(|e| e.river_order.max(0) as u32),
                    absent,
                }
            })
            .collect();

        let factions: Vec<FactionIn> = (1..civ.faction_roster.0.len())
            .map(|f| {
                let e = &civ.faction_roster.0[f];
                let (religion_key, religion_label) = match civ.religions.choice(&e.religion) {
                    cartalith_civ::religion_library::ReligionChoice::Builtin(k) => (k.to_string(), label_of(k)),
                    cartalith_civ::religion_library::ReligionChoice::Custom(d) => (cartalith_civ::religion_library::custom_religion_key(d.id), d.name.clone()),
                    cartalith_civ::religion_library::ReligionChoice::Missing(raw) => ("none".to_string(), format!("Missing religion ({raw})")),
                };
                let government_label = cartalith_civ::roster::CIV_GOVERNMENTS.iter().find(|(k, _)| *k == e.government).map_or_else(|| e.government.clone(), |(_, l)| (*l).to_string());
                FactionIn {
                    id: f as i64,
                    name: e.name.clone(),
                    culture: e.culture.clone(),
                    religion_label,
                    religion_key,
                    government_key: e.government.clone(),
                    government_label,
                    ag_tech: e.ag_tech.clone(),
                    claimed_cells: crate::faction_claimed_cells(&civ.territory, self.gw, self.gh, f as i32).map(|n| n as u64),
                    absent: absent1("claimed_cells", "the territory grid is not known (a reopened project without rasters/territory.i32)"),
                }
            })
            .collect();

        let mut religions: Vec<ReligionIn> = cartalith_civ::roster::CIV_RELIGIONS
            .iter()
            .map(|(k, l)| ReligionIn { key: (*k).to_string(), label: (*l).to_string(), custom: false, base_key: None, notes: None, axes: builtin_axes(k), absent: Absent::new() })
            .collect();
        for d in civ.religions.defs() {
            religions.push(ReligionIn {
                key: cartalith_civ::religion_library::custom_religion_key(d.id),
                label: d.name.clone(),
                custom: true,
                base_key: Some(d.base_key.clone()),
                notes: Some(d.notes.clone()),
                axes: override_axes(&d.axes),
                absent: Absent::new(),
            });
        }

        let governments: Vec<GovernmentIn> = cartalith_civ::roster::CIV_GOVERNMENTS
            .iter()
            .zip(cartalith_civ::roster::GOVERNMENT_DISPLAY.iter())
            .map(|((k, l), d)| GovernmentIn { key: (*k).to_string(), label: (*l).to_string(), hint: Some(d.hint.to_string()), group: d.group.map(str::to_string), absent: Absent::new() })
            .collect();

        let cultures: Vec<CultureIn> = cartalith_civ::CIV_CULTURES
            .iter()
            .enumerate()
            .map(|(i, c)| CultureIn {
                index: i as i64,
                key: c.key.to_string(),
                name: culture_name(c.key),
                terrain_affinity: cartalith_civ::CIV_CULTURE_TERRAIN_KEY.iter().find(|(k, _)| *k == c.key).map(|(_, t)| (*t).to_string()),
                absent: Absent::new(),
            })
            .collect();

        let provinces: Vec<ProvinceIn> = civ
            .province_list
            .iter()
            .map(|p| ProvinceIn { id: i64::from(p.id), name: p.name.clone(), faction: i64::from(p.faction), capital: civ.settlements.get(p.capital_settlement_index).map(|s| s.tid as i64), absent: Absent::new() })
            .collect();

        let continents: Vec<ContinentIn> = civ
            .continents
            .iter()
            .map(|c| ContinentIn { id: i64::from(c.id), name: c.name.clone(), cells: c.cells as u64, faction: i64::from(c.faction), absent: Absent::new() })
            .collect();

        let trade = crate::civ_trade_bridge::trade_network_for(self.ws, civ, self.gw, self.gh, self.sea, self.map_width_km, self.seed).map(|net| {
            net.flows
                .iter()
                .filter_map(|f| {
                    let (a, b) = (civ.settlements.get(f.from)?, civ.settlements.get(f.to)?);
                    Some(TradeFlowIn { from: a.tid as i64, to: b.tid as i64, good: f.good.to_string(), mode: f.mode.as_str().to_string(), volume: f.volume, distance_km: f.distance_km })
                })
                .collect::<Vec<_>>()
        });

        let landmarks = self.landmarks(story.landmarks);
        let rivers = self.rivers();
        let volcanoes: Vec<VolcanoIn> = self
            .ws
            .volcanoes
            .as_deref()
            .map(|v| v.as_slice())
            .unwrap_or(&[])
            .iter()
            .enumerate()
            .map(|(i, e)| VolcanoIn {
                id: i as i64,
                kind: e.kind.map(|k| k.key().to_string()),
                age_class: Some(format!("about {:.0} years old", cartalith_terrain::volcano_attr::years(e))),
                state: Some(cartalith_terrain::volcano_attr::activity(e).key().to_string()),
                x: e.cx.round() as i64,
                y: e.cy.round() as i64,
                absent: Absent::new(),
            })
            .collect();
        let conflicts: Vec<ConflictIn> = story
            .conflicts
            .iter()
            .map(|c| ConflictIn {
                id: c.id as i64,
                name: c.name.clone(),
                kind: c.kind.key().to_string(),
                parties: c.sides.iter().map(|s| i64::from(*s)).collect(),
                start_year: c.start_year,
                end_year: c.end_year,
                notes: Some(c.outcome.clone()).filter(|n| !n.is_empty()),
                absent: Absent::new(),
            })
            .collect();
        let journeys: Vec<JourneyIn> = story
            .journeys
            .iter()
            .map(|j| JourneyIn { id: j.id as i64, name: j.name.clone(), party_preset: j.party_preset.clone(), start_year: j.start_year, distance_km: j.route.length_km, mode: crate::project_bridge::route_mode_key(j.route.mode).to_string(), absent: Absent::new() })
            .collect();

        WorldInput {
            seed: i64::from(self.seed),
            settlements,
            factions,
            religions,
            governments,
            cultures,
            provinces,
            continents,
            relations: self.relations(),
            trade,
            landmarks,
            rivers,
            volcanoes,
            conflicts,
            journeys,
        }
    }

    /// Each settlement's continent id, by the landmass its cell lies on: the 8-connected component
    /// `civ_continents` ranked, matched back by its cell count and bounding box (a component that
    /// fell under the floor matches nothing, and the settlement reports an absent continent).
    fn continent_of_settlements(&self) -> Vec<Option<i64>> {
        let civ = self.civ;
        let lq = cartalith_civ::build_landmass_quality(&self.ws.field, None, self.gw, self.gh, self.sea, self.world);
        let mut by_comp: BTreeMap<i32, i64> = BTreeMap::new();
        let mut cells = vec![0usize; lq.count];
        let mut bb = vec![(usize::MAX, usize::MAX, 0usize, 0usize); lq.count];
        for (i, &c) in lq.comp.iter().enumerate() {
            if c < 0 {
                continue;
            }
            let (x, y) = (i % self.gw, i / self.gw);
            let b = &mut bb[c as usize];
            cells[c as usize] += 1;
            *b = (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y));
        }
        for k in &civ.continents {
            if let Some(c) = (0..lq.count).find(|&c| cells[c] == k.cells && bb[c] == (k.min_x, k.min_y, k.max_x, k.max_y)) {
                by_comp.insert(c as i32, i64::from(k.id));
            }
        }
        civ.settlements
            .iter()
            .map(|s| lq.comp.get(s.placement.y * self.gw + s.placement.x).and_then(|c| by_comp.get(c)).copied())
            .collect()
    }

    /// `civ_faction_relations`' rows (the same function over the same aggregates), with the
    /// no-claim-grid case reported as absent figures and not as zeros.
    fn relations(&self) -> Vec<RelationIn> {
        let civ = self.civ;
        let mw = MilWorld { civ, ws: self.ws, gw: self.gw, gh: self.gh, sea: self.sea, map_width_km: self.map_width_km, seed: self.seed };
        let view = CivView::live(civ);
        let defences = mw.defences_of(&civ.settlements, None);
        let agg = mw.aggregates_of(&view, &defences, true);
        let cultures: Vec<&str> = civ.faction_roster.0.iter().map(|e| e.culture.as_str()).collect();
        let religions: Vec<&str> = civ.faction_roster.engine_religions(&civ.religions);
        let claims_known = crate::claim_grid_known(&civ.territory, self.gw, self.gh);
        let input = cartalith_civ::relations::FactionRelationsInput {
            faction_count: civ.faction_roster.0.len(),
            gw: self.gw,
            gh: self.gh,
            territory: Some(&civ.territory),
            cultures: &cultures,
            religions: &religions,
            wrap_x: self.world,
        };
        cartalith_civ::relations::civ_faction_relations(&input, &agg)
            .pairs
            .iter()
            .map(|r| {
                if claims_known {
                    RelationIn { a: r.a as i64, b: r.b as i64, stance: Some(r.stance.to_string()), value: Some(r.value), border_cells: Some(r.border_cells as u64), absent: Absent::new() }
                } else {
                    let why = "the territory grid is not known, so there is no border or trade term";
                    let mut absent = absent1("stance", why);
                    absent.insert("value".into(), why.into());
                    absent.insert("border_cells".into(), why.into());
                    RelationIn { a: r.a as i64, b: r.b as i64, stance: None, value: None, border_cells: None, absent }
                }
            })
            .collect()
    }

    /// The last landmark run's landmarks (none when no run happened).
    fn landmarks(&self, ls: &[cartalith_civ::landmark::Landmark]) -> Vec<LandmarkIn> {
        let civ = self.civ;
        let provinces_known = self.gw > 0 && civ.provinces.len() == self.gw * self.gh;
        ls.iter()
            .map(|m| {
                let nearest = civ
                    .settlements
                    .iter()
                    .map(|s| {
                        let (dx, dy) = (s.placement.x as f64 - m.x as f64, s.placement.y as f64 - m.y as f64);
                        ((dx * dx + dy * dy).sqrt(), s.tid as i64)
                    })
                    .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
                    .map(|(d, id)| (id, d));
                let region = provinces_known
                    .then(|| civ.provinces.get(m.y * self.gw + m.x).copied().filter(|p| *p > 0))
                    .flatten()
                    .and_then(|pid| civ.province_list.iter().find(|p| p.id == pid))
                    .map(|p| p.name.clone());
                let mut absent = Absent::new();
                if !provinces_known {
                    absent.insert("region".into(), "the province raster is not known".into());
                }
                LandmarkIn {
                    index: m.id as i64,
                    kind: m.kind.clone(),
                    entity_id: Some(cartalith_vault::links::landmark_entity_id(&m.key())),
                    name: None,
                    x: m.x as i64,
                    y: m.y as i64,
                    region,
                    nearest_settlement: nearest,
                    cause: Some(m.causal.join(" -> ")).filter(|c| !c.is_empty()),
                    absent,
                }
            })
            .collect()
    }

    /// Rivers of order [`RIVER_MIN_ORDER`] and above (`river_entities`, the call `get_rivers` makes).
    fn rivers(&self) -> Vec<RiverIn> {
        let (Some(order), Some(ch)) = (self.ws.stream_order.as_ref(), self.ws.channels.as_ref()) else { return Vec::new() };
        let rivers = cartalith_hydrology::river_entities(
            order,
            &ch.recv,
            &self.ws.flow_discharge,
            &self.ws.field,
            self.gw,
            self.gh,
            RIVER_MIN_ORDER.clamp(1, i64::from(i32::MAX)) as i32,
            cartalith_hydrology::river_flow_thresh(self.gw, self.gh, self.gw, self.map_width_km),
            cartalith_hydrology::river_width_scale_k(self.map_width_km),
            self.world,
        );
        let ck = cartalith_spatial::cell_km(self.map_width_km, self.gw);
        let metres = |cell: u32| -> Option<f64> {
            let h = *self.ws.field.get(cell as usize)? as f64;
            let denom = if (1.0 - self.sea) == 0.0 { 1e-6 } else { 1.0 - self.sea };
            Some((h - self.sea) / denom * self.peak_m)
        };
        rivers
            .iter()
            .enumerate()
            .map(|(i, r)| RiverIn {
                index: i as i64,
                order: r.order.max(0) as u32,
                km: r.length_cells * ck,
                tributaries: r.tributaries,
                head: ((r.head as usize % self.gw) as i64, (r.head as usize / self.gw) as i64),
                mouth: ((r.mouth as usize % self.gw) as i64, (r.mouth as usize / self.gw) as i64),
                source_m: metres(r.head),
                mouth_m: metres(r.mouth),
                absent: Absent::new(),
            })
            .collect()
    }
}

#[godot_api(secondary)]
impl WorldGen {
    /// RK-1: every world document, rendered from the engine's own state (module doc). Returns
    /// `{ok: true, count, bytes, elapsed_ms, documents: [{id, kind, title, locator, header, body}]}`,
    /// or `{ok: false, reason}` when there is no generated world with a civilisation layer. It reads
    /// only: nothing in the world changes, nothing is stored, and nothing is sent anywhere (the
    /// retrieval feature is off by default and has no socket; the checkbox plumbing is RK-5/RK-6).
    #[func]
    fn knowledge_world_documents(&self) -> VarDictionary {
        let t0 = std::time::Instant::now();
        let Some(input) = self.knowledge_input() else {
            return vdict! { "ok" => false, "reason" => "no generated world with a civilisation layer is open" };
        };
        let docs = cartalith_knowledge::world::render_documents(&input);
        let mut bytes = 0usize;
        let rows: Array<VarDictionary> = docs
            .iter()
            .map(|d| {
                bytes += d.body.len();
                vdict! {
                    "id" => d.id.as_str(),
                    "kind" => d.kind.as_str(),
                    "title" => d.title.as_str(),
                    "locator" => d.locator.as_str(),
                    "header" => d.header.as_str(),
                    "body" => d.body.as_str(),
                }
            })
            .collect();
        vdict! { "ok" => true, "count" => docs.len() as i64, "bytes" => bytes as i64, "elapsed_ms" => t0.elapsed().as_millis() as i64, "documents" => &rows }
    }

    /// RK-5: executes ONE knowledge call for the assistant and returns the RAW result as JSON text
    /// (`cartalith_llm::knowledge_tools::sanitize` rebuilds it before a model sees any of it).
    /// `call_json` is the validated call Rust named in `LlmStream.pending_needs()`; it is validated
    /// AGAIN here (`parse`) against `sources` (`{world: bool, vault: bool}`, from
    /// `LlmStream.knowledge_plan`), so a forged call or a source that is not ticked is refused in
    /// words. Read-only; the shell calls it only while no generation holds the engine. Builds or
    /// refreshes the in-memory index first (module `knowledge_service`); nothing is persisted.
    #[func]
    fn knowledge_call(&mut self, call_json: GString, sources: VarDictionary) -> GString {
        let s = Self::knowledge_sources_of(&sources);
        let out = match serde_json::from_str::<serde_json::Value>(&call_json.to_string()) {
            Ok(serde_json::Value::Object(mut m)) => {
                let tool = m.remove("tool").and_then(|t| t.as_str().map(str::to_owned)).unwrap_or_default();
                match cartalith_llm::knowledge_tools::parse(&tool, &serde_json::Value::Object(m), s) {
                    Ok(call) => self.knowledge_run(&call, s),
                    Err(why) => serde_json::json!({"available": false, "reason": why}).to_string(),
                }
            }
            _ => serde_json::json!({"available": false, "reason": "the knowledge call was not readable"}).to_string(),
        };
        GString::from(out.as_str())
    }

    /// RK-6: the pre-retrieval search. Searches `query` (cut to the 200-character limit) over the
    /// ticked `sources` for the top `knowledge_limits().pack_hits` and returns the raw result JSON
    /// (rebuilt in Rust by `build_pack`). Same rules as [`WorldGen::knowledge_call`].
    #[func]
    fn knowledge_search_json(&mut self, query: GString, sources: VarDictionary) -> GString {
        let s = Self::knowledge_sources_of(&sources);
        if !s.any() {
            return GString::from(r#"{"available": false, "reason": "no knowledge source is turned on"}"#);
        }
        let q: String = query.to_string().chars().map(|c| if c.is_control() { ' ' } else { c }).collect::<String>().trim().chars().take(cartalith_llm::knowledge_tools::MAX_QUERY_CHARS).collect();
        let call = cartalith_llm::knowledge_tools::KnowledgeCall::Search { query: q, scope: s.scopes()[0].to_owned(), limit: cartalith_llm::knowledge_tools::PACK_MAX_HITS };
        if matches!(&call, cartalith_llm::knowledge_tools::KnowledgeCall::Search { query, .. } if query.trim().is_empty()) {
            return GString::from(r#"{"available": false, "reason": "the request has no words to search for"}"#);
        }
        GString::from(self.knowledge_run(&call, s).as_str())
    }

    /// RK-5: forgets the knowledge index and every Vault text it held (the "feature off" state).
    #[func]
    fn knowledge_clear(&mut self) {
        self.knowledge.clear();
    }

    /// The collected [`WorldInput`] as JSON (what the evaluation fixtures commit), or `""` when
    /// there is no generated world. Read-only.
    #[func]
    fn knowledge_world_input_json(&self) -> GString {
        match self.knowledge_input() {
            Some(i) => GString::from(serde_json::to_string(&i).unwrap_or_default().as_str()),
            None => GString::new(),
        }
    }
}

impl WorldGen {
    /// The two source flags of a shell dictionary; anything missing or not a bool reads as OFF.
    fn knowledge_sources_of(d: &VarDictionary) -> cartalith_llm::knowledge_tools::KnowledgeSources {
        cartalith_llm::knowledge_tools::KnowledgeSources { world: crate::llm_bridge::flag(d, "world"), vault: crate::llm_bridge::flag(d, "vault") }
    }

    /// Brings the index up to date for `s` and executes `call`; the raw result as JSON text.
    /// Only a TICKED source is read: the world input is collected only when `s.world`, the Vault is
    /// listed only when `s.vault` and a vault is bound.
    fn knowledge_run(&mut self, call: &cartalith_llm::knowledge_tools::KnowledgeCall, s: cartalith_llm::knowledge_tools::KnowledgeSources) -> String {
        let world = if s.world { self.knowledge_input() } else { None };
        let provider = if s.vault && self.vault.is_bound() { self.vault.vault() } else { None };
        if let Err(why) = self.knowledge.ensure(world.as_ref(), provider, s) {
            return serde_json::json!({"available": false, "reason": why}).to_string();
        }
        match self.knowledge.index() {
            Some(idx) => crate::knowledge_service::execute(idx, call).to_string(),
            None => serde_json::json!({"available": false, "reason": "the knowledge base is not built"}).to_string(),
        }
    }

    /// Collects the world's [`WorldInput`] from this `WorldGen`, or `None` without a generated
    /// world and its civilisation layer.
    fn knowledge_input(&self) -> Option<WorldInput> {
        let (Some(WorldSource::Generated(ws)), Some(civ)) = (self.source.as_ref(), self.civ.as_ref()) else { return None };
        let kw = KnowledgeWorld {
            civ,
            ws,
            gw: self.gw.max(0) as usize,
            gh: self.gh.max(0) as usize,
            sea: self.sea_level,
            map_width_km: self.map_width_km,
            seed: self.seed,
            world: self.world,
            peak_m: self.params.peak_m,
        };
        let landmarks: &[cartalith_civ::landmark::Landmark] = self.landmark_store.last.as_ref().map_or(&[], |r| r.landmarks.as_slice());
        let journeys: &[cartalith_civ::travel_library::Journey] = self.infra.as_ref().map_or(&[], |i| i.journeys.as_slice());
        Some(kw.input(&StoryParts { landmarks, conflicts: &self.conflicts.conflicts, journeys }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_knowledge::world::{render_documents, world_edges};
    use std::collections::BTreeSet;

    /// The three fixture seeds (`world_summary`'s) and the grid every test world uses. 256 wide
    /// keeps a world to seconds and still places dozens of settlements; the count is asserted so a
    /// fixture cannot silently become empty.
    const SEEDS: [i32; 3] = [71077345, 24601, 9137];
    const GW: usize = 512;

    /// One generated world through the app's own parameters, its civilisation layer, and the
    /// collected input.
    fn world(seed: i32) -> (cartalith_engine::WorldState, CivData, usize, cartalith_engine::WorldParams) {
        crate::fh_r2_baseline::generate_world(seed, 800.0, 8, GW)
    }

    fn collect(seed: i32) -> (WorldInput, CivData, cartalith_engine::WorldState, usize) {
        let (ws, civ, gh, p) = world(seed);
        let kw = KnowledgeWorld { civ: &civ, ws: &ws, gw: GW, gh, sea: ws.sea_level, map_width_km: p.map_width_km, seed, world: p.world, peak_m: p.peak_m };
        let input = kw.input(&StoryParts { landmarks: &[], conflicts: &[], journeys: &[] });
        (input, civ, ws, gh)
    }

    fn doc_count(docs: &[cartalith_knowledge::model::Document], kind: &str) -> usize {
        docs.iter().filter(|d| d.kind == kind).count()
    }

    /// Protects RK-1's coverage bar on three real worlds: every settlement, faction, religion,
    /// government, culture, province, continent, volcano and relation has a document, counted
    /// INDEPENDENTLY from `CivData` and the engine tables (not from the input structs the renderer
    /// read), and the world is not empty. MUTATION: skip one kind in `input` and its count goes red.
    #[test]
    fn every_entity_has_a_document_on_three_seeds() {
        for seed in SEEDS {
            let (input, civ, ws, _) = collect(seed);
            let docs = render_documents(&input);
            assert!(civ.settlements.len() >= 10, "seed {seed}: the fixture world must place settlements, got {}", civ.settlements.len());
            assert_eq!(doc_count(&docs, "settlement"), civ.settlements.len(), "seed {seed}");
            assert_eq!(doc_count(&docs, "faction"), civ.faction_roster.0.len() - 1, "seed {seed}");
            assert_eq!(doc_count(&docs, "religion"), cartalith_civ::roster::CIV_RELIGIONS.len() + civ.religions.defs().len(), "seed {seed}");
            assert_eq!(doc_count(&docs, "government"), cartalith_civ::roster::CIV_GOVERNMENTS.len(), "seed {seed}");
            assert_eq!(doc_count(&docs, "culture"), cartalith_civ::CIV_CULTURES.len(), "seed {seed}");
            assert_eq!(doc_count(&docs, "province"), civ.province_list.len(), "seed {seed}");
            assert_eq!(doc_count(&docs, "continent"), civ.continents.len(), "seed {seed}");
            assert_eq!(doc_count(&docs, "volcano"), ws.volcanoes.as_ref().map_or(0, |v| v.len()), "seed {seed}");
            let n = civ.faction_roster.0.len() - 1;
            assert_eq!(doc_count(&docs, "relation"), n * (n - 1) / 2, "seed {seed}: one document per unordered faction pair");
            // Trade: one document per distinct unordered settlement pair with a flow.
            let net = crate::civ_trade_bridge::trade_network_for(&ws, &civ, GW, civ.territory.len() / GW, ws.sea_level, 800.0, seed).expect("trade runs on a world with settlements");
            let pairs: BTreeSet<(usize, usize)> = net.flows.iter().map(|f| (f.from.min(f.to), f.from.max(f.to))).collect();
            assert_eq!(doc_count(&docs, "trade"), pairs.len(), "seed {seed}");
            assert!(!pairs.is_empty(), "seed {seed}: the fixture world must have trade, or the relational category is untested");
            assert!(doc_count(&docs, "rollup") >= 1 + (civ.faction_roster.0.len() - 1), "seed {seed}");
        }
    }

    /// Protects the figures: for every settlement and faction on three real worlds the document's
    /// numbers equal the engine's, recounted from `CivData` with a different loop (a filter and a
    /// sum, not the renderer's input). Also the owner/capital/coastal words, the province name, and
    /// the claimed-cell count against the raw grid. MUTATION: render `pop + 1` and the first assert fails.
    #[test]
    fn document_figures_equal_the_engine_on_three_seeds() {
        for seed in SEEDS {
            let (input, civ, ws, _) = collect(seed);
            let docs = render_documents(&input);
            let by_id: BTreeMap<&str, &str> = docs.iter().map(|d| (d.id.as_str(), d.body.as_str())).collect();
            for s in &civ.settlements {
                let body = by_id[format!("W:settlement:{}", s.tid).as_str()];
                assert!(body.contains(&format!("\npopulation: {}\n", s.pop)), "seed {seed} {}: {body}", s.name);
                assert!(body.starts_with(&format!("Settlement: {}\n", s.name)), "seed {seed}");
                assert!(body.contains(&format!("\ncapital: {}\n", if s.placement.capital { "yes" } else { "no" })));
                assert!(body.contains(&format!("\ncoastal: {}\n", if s.placement.coastal { "yes" } else { "no" })));
                assert!(body.contains(&format!("\nposition: cell ({}, {})\n", s.placement.x, s.placement.y)));
                let f = s.placement.faction as usize;
                assert!(body.contains(&format!("owner faction: {} (W:faction:{f})", civ.faction_roster.0[f].name)), "seed {seed} {}: {body}", s.name);
                // Province: the raster value at the cell, named from the list.
                let pid = civ.provinces[s.placement.y * GW + s.placement.x];
                match civ.province_list.iter().find(|p| p.id == pid) {
                    Some(p) if pid > 0 => assert!(body.contains(&format!("province: {} (W:province:{pid})", p.name)), "{body}"),
                    _ => assert!(body.contains("province: not available (no province covers its cell)"), "{body}"),
                }
            }
            // The river at each site, from the site explanation the engine kept: order > 0 names the
            // order, 0 says there is none (a real zero, not an absent value).
            let mut on_river = 0;
            for (i, s) in civ.settlements.iter().enumerate() {
                let body = by_id[format!("W:settlement:{}", s.tid).as_str()];
                let line = match civ.explanations.get(i) {
                    Some(e) if e.river_order > 0 => {
                        on_river += 1;
                        format!("\nriver at site: river of order {}\n", e.river_order)
                    }
                    Some(_) => "\nriver at site: none (no traced river at the site)\n".to_string(),
                    None => "\nriver at site: not available (the site explanation is not available)\n".to_string(),
                };
                assert!(body.contains(&line), "seed {seed} {}: expected {line:?} in {body}", s.name);
            }
            // The continent of each site, recounted with an independent 8-connected flood fill from
            // the settlement's own cell (the bridge relabels the whole grid and matches by bounding box).
            let (gw, gh) = (GW, civ.territory.len() / GW);
            let mut with_continent = 0;
            for s in &civ.settlements {
                let start = s.placement.y * gw + s.placement.x;
                let (mut seen, mut stack) = (std::collections::HashSet::new(), vec![start]);
                seen.insert(start);
                let (mut minx, mut miny, mut maxx, mut maxy) = (usize::MAX, usize::MAX, 0usize, 0usize);
                while let Some(i) = stack.pop() {
                    let (x, y) = (i % gw, i / gw);
                    (minx, miny, maxx, maxy) = (minx.min(x), miny.min(y), maxx.max(x), maxy.max(y));
                    for dy in -1i64..=1 {
                        for dx in -1i64..=1 {
                            let ny = y as i64 + dy;
                            if (dx, dy) == (0, 0) || ny < 0 || ny >= gh as i64 {
                                continue;
                            }
                            let nx = x as i64 + dx;
                            if nx < 0 || nx >= gw as i64 {
                                continue;
                            }
                            let j = ny as usize * gw + nx as usize;
                            if ws.field[j] as f64 >= ws.sea_level && seen.insert(j) {
                                stack.push(j);
                            }
                        }
                    }
                }
                let want = civ.continents.iter().find(|k| k.cells == seen.len() && (k.min_x, k.min_y, k.max_x, k.max_y) == (minx, miny, maxx, maxy));
                let body = by_id[format!("W:settlement:{}", s.tid).as_str()];
                match want {
                    Some(k) => {
                        with_continent += 1;
                        assert!(body.contains(&format!("continent: {} (W:continent:{})", k.name, k.id)), "seed {seed} {}: {body}", s.name);
                    }
                    None => assert!(body.contains("continent: not available"), "seed {seed} {}: {body}", s.name),
                }
            }
            assert!(on_river >= 1 && with_continent >= civ.settlements.len() / 2, "seed {seed}: on_river {on_river}, with_continent {with_continent}: the checks must see real data");
            for f in 1..civ.faction_roster.0.len() {
                let body = by_id[format!("W:faction:{f}").as_str()];
                let mine: Vec<&cartalith_civ::NamedSettlement> = civ.settlements.iter().filter(|s| s.placement.faction == f as i32).collect();
                let pop: u64 = mine.iter().map(|s| u64::from(s.pop)).sum();
                assert!(body.contains(&format!("\nsettlements: {}\n", mine.len())), "seed {seed} faction {f}: {body}");
                assert!(body.contains(&format!("\npopulation: {pop}\n")), "seed {seed} faction {f}");
                let claimed = civ.territory.iter().filter(|&&t| t == f as i32).count();
                assert!(body.contains(&format!("\nclaimed cells: {claimed}\n")), "seed {seed} faction {f}: {body}");
                assert!(body.contains(&format!("culture: {}\n", civ.faction_roster.0[f].culture)));
            }
        }
    }

    /// Protects the trade bar (the relational ground truth): for every settlement the partner ids in
    /// its document equal the partner set recounted from the raw flows, in BOTH directions, and a
    /// settlement with no flow says so. Run on three worlds. MUTATION: drop the `to` side in
    /// `render_settlement` and the partner sets differ.
    #[test]
    fn trade_partners_equal_the_flows_on_three_seeds() {
        for seed in SEEDS {
            let (input, civ, ws, gh) = collect(seed);
            let docs = render_documents(&input);
            let net = crate::civ_trade_bridge::trade_network_for(&ws, &civ, GW, gh, ws.sea_level, 800.0, seed).unwrap();
            let mut want: BTreeMap<u64, BTreeSet<u64>> = BTreeMap::new();
            for f in &net.flows {
                let (a, b) = (civ.settlements[f.from].tid, civ.settlements[f.to].tid);
                want.entry(a).or_default().insert(b);
                want.entry(b).or_default().insert(a);
            }
            let mut with_partners = 0;
            for s in &civ.settlements {
                let body = &docs.iter().find(|d| d.id == format!("W:settlement:{}", s.tid)).unwrap().body;
                let got: BTreeSet<u64> = body
                    .lines()
                    .filter(|l| l.starts_with("- ") && l.contains("(W:settlement:"))
                    .filter_map(|l| l.split("(W:settlement:").nth(1)?.split(')').next()?.parse().ok())
                    .collect();
                let expect = want.get(&s.tid).cloned().unwrap_or_default();
                assert_eq!(got, expect, "seed {seed} {}", s.name);
                if expect.is_empty() {
                    assert!(body.contains("trade partners: none"), "{body}");
                } else {
                    with_partners += 1;
                }
            }
            assert!(with_partners >= 2, "seed {seed}: the check must see real partners, got {with_partners}");
            // The graph's `trades_with` edges are exactly those pairs too (RK-4's equality, from the same source).
            let edges = world_edges(&input);
            let graph_pairs: BTreeSet<(String, String)> = edges.iter().filter(|e| e.rel == "trades_with").map(|e| (e.from.clone(), e.to.clone())).collect();
            let flow_pairs: BTreeSet<(String, String)> = net
                .flows
                .iter()
                .map(|f| {
                    let (a, b) = (civ.settlements[f.from].tid, civ.settlements[f.to].tid);
                    (format!("W:settlement:{}", a.min(b)), format!("W:settlement:{}", a.max(b)))
                })
                .filter(|(a, b)| a != b)
                .collect();
            assert_eq!(graph_pairs, flow_pairs, "seed {seed}");
        }
    }

    /// Protects ownership/faith/government edges against the roster, recounted: for every settlement
    /// `owned_by` its faction, for every faction `adheres_to` its religion's document and `governed_by`
    /// its government, `located_in` its province. RK-4's getter-equality bar for these relations.
    #[test]
    fn ownership_faith_and_government_edges_equal_the_roster_on_three_seeds() {
        for seed in SEEDS {
            let (input, civ, _ws, _) = collect(seed);
            let edges = world_edges(&input);
            let has = |f: &str, r: &str, t: &str| edges.iter().any(|e| e.from == f && e.rel == r && e.to == t);
            for s in &civ.settlements {
                assert!(has(&format!("W:settlement:{}", s.tid), "owned_by", &format!("W:faction:{}", s.placement.faction)), "seed {seed} {}", s.name);
            }
            for f in 1..civ.faction_roster.0.len() {
                let e = &civ.faction_roster.0[f];
                assert!(has(&format!("W:faction:{f}"), "governed_by", &format!("W:government:{}", e.government)), "seed {seed} faction {f}");
                let rk = match civ.religions.choice(&e.religion) {
                    cartalith_civ::religion_library::ReligionChoice::Builtin(k) => k.to_string(),
                    cartalith_civ::religion_library::ReligionChoice::Custom(d) => cartalith_civ::religion_library::custom_religion_key(d.id),
                    cartalith_civ::religion_library::ReligionChoice::Missing(_) => "none".to_string(),
                };
                assert!(has(&format!("W:faction:{f}"), "adheres_to", &format!("W:religion:{rk}")), "seed {seed} faction {f}");
            }
            let located = edges.iter().filter(|e| e.rel == "located_in" && e.to.starts_with("W:province:")).count();
            let in_province = civ.settlements.iter().filter(|s| civ.provinces[s.placement.y * GW + s.placement.x] > 0).count();
            assert_eq!(located, in_province, "seed {seed}");
        }
    }

    /// Protects determinism and the size cap on real worlds: two renders are byte-identical (a
    /// 64-bit FNV over every id and body) and no document exceeds 6 000 bytes.
    #[test]
    fn two_renders_are_byte_identical_and_under_the_cap_on_three_seeds() {
        let fnv = |docs: &[cartalith_knowledge::model::Document]| {
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for d in docs {
                for b in d.id.bytes().chain(d.body.bytes()) {
                    h ^= u64::from(b);
                    h = h.wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
            h
        };
        for seed in SEEDS {
            let (input, ..) = collect(seed);
            let (a, b) = (render_documents(&input), render_documents(&input));
            assert!(!a.is_empty());
            assert_eq!(fnv(&a), fnv(&b), "seed {seed}");
            assert!(a.iter().all(|d| d.body.len() <= cartalith_knowledge::world::MAX_DOC_BYTES), "seed {seed}");
        }
    }

    /// Protects the relation rows: each faction pair's stance and value equal a direct call of
    /// `civ_faction_relations` over the roster (the engine's own function), and the no-claim-grid
    /// case renders `not available` rather than a number.
    #[test]
    fn relation_documents_equal_civ_faction_relations_and_absent_without_a_claim_grid() {
        let (input, civ, ws, gh) = collect(SEEDS[1]);
        let docs = render_documents(&input);
        let mw = MilWorld { civ: &civ, ws: &ws, gw: GW, gh, sea: ws.sea_level, map_width_km: 800.0, seed: SEEDS[1] };
        let agg = mw.aggregates_of(&CivView::live(&civ), &mw.defences_of(&civ.settlements, None), true);
        let cultures: Vec<&str> = civ.faction_roster.0.iter().map(|e| e.culture.as_str()).collect();
        let religions: Vec<&str> = civ.faction_roster.engine_religions(&civ.religions);
        let inp = cartalith_civ::relations::FactionRelationsInput { faction_count: civ.faction_roster.0.len(), gw: GW, gh, territory: Some(&civ.territory), cultures: &cultures, religions: &religions, wrap_x: false };
        let rel = cartalith_civ::relations::civ_faction_relations(&inp, &agg);
        assert!(!rel.pairs.is_empty());
        for r in &rel.pairs {
            let body = &docs.iter().find(|d| d.id == format!("W:relation:{}-{}", r.a, r.b)).unwrap().body;
            assert!(body.contains(&format!("\nstance: {}\n", r.stance)), "{body}");
            assert!(body.contains(&format!("\nvalue: {:.2}\n", r.value)), "{body}");
            assert!(body.lines().any(|l| l == format!("shared border cells: {}", r.border_cells)), "{body}");
        }
        // Without a claim grid the figures are absent, not zero.
        let mut civ2 = civ;
        civ2.territory = Vec::new();
        let kw = KnowledgeWorld { civ: &civ2, ws: &ws, gw: GW, gh, sea: ws.sea_level, map_width_km: 800.0, seed: SEEDS[1], world: false, peak_m: 1000.0 };
        let docs2 = render_documents(&kw.input(&StoryParts { landmarks: &[], conflicts: &[], journeys: &[] }));
        let r0 = docs2.iter().find(|d| d.kind == "relation").unwrap();
        assert!(r0.body.contains("stance: not available (the territory grid is not known"), "{}", r0.body);
        let f1 = docs2.iter().find(|d| d.id == "W:faction:1").unwrap();
        assert!(f1.body.contains("claimed cells: not available"), "{}", f1.body);
    }

    /// Protects the province rule on a settlement the generated worlds never produce (every site in
    /// the three fixture worlds lies in a province): with its cell's province raster value set to 0
    /// the document says no province covers it (a real "none"), and with the raster removed the
    /// document says the raster is not KNOWN (an absent figure, a different statement). MUTATION:
    /// drop the `> 0` filter and the first assert goes red.
    #[test]
    fn a_site_outside_every_province_and_a_missing_raster_are_different_statements() {
        let (_, mut civ, ws, gh) = collect(SEEDS[0]);
        let s = civ.settlements[0].clone();
        let cell = s.placement.y * GW + s.placement.x;
        civ.provinces[cell] = 0;
        fn input_of(civ: &CivData, ws: &cartalith_engine::WorldState, gh: usize) -> WorldInput {
            KnowledgeWorld { civ, ws, gw: GW, gh, sea: ws.sea_level, map_width_km: 800.0, seed: SEEDS[0], world: false, peak_m: 1000.0 }.input(&StoryParts { landmarks: &[], conflicts: &[], journeys: &[] })
        }
        let docs = render_documents(&input_of(&civ, &ws, gh));
        let body = &docs.iter().find(|d| d.id == format!("W:settlement:{}", s.tid)).unwrap().body;
        assert!(body.contains("province: not available (no province covers its cell)"), "{body}");
        civ.provinces = Vec::new();
        let docs = render_documents(&input_of(&civ, &ws, gh));
        let body = &docs.iter().find(|d| d.id == format!("W:settlement:{}", s.tid)).unwrap().body;
        assert!(body.contains("province: not available (the province raster is not known"), "{body}");
    }

    /// Protects the Story-layer conversions on literal values (no engine run exists for them in a
    /// fresh world): a conflict, a journey and a landmark each keep their figures, an unset conflict
    /// outcome is absent (not an empty note), and a landmark names its nearest settlement.
    #[test]
    fn story_rows_convert_with_absent_notes_and_a_nearest_settlement() {
        let (_, civ, ws, gh) = collect(SEEDS[2]);
        let kw = KnowledgeWorld { civ: &civ, ws: &ws, gw: GW, gh, sea: ws.sea_level, map_width_km: 800.0, seed: SEEDS[2], world: false, peak_m: 1000.0 };
        let s0 = &civ.settlements[0];
        let lm = cartalith_civ::landmark::Landmark {
            id: 7,
            kind: "waterfall".into(),
            class: cartalith_civ::landmark::LandmarkClass::Local,
            x: s0.placement.x,
            y: s0.placement.y,
            elevation: 12.0,
            score: 0.5,
            importance: 0.5,
            causal: vec!["a river drops".into(), "a waterfall".into()],
            seed: 1,
        };
        let conflict = cartalith_civ::conflict::Conflict {
            id: 3,
            name: "The Salt War".into(),
            kind: cartalith_civ::conflict::ConflictKind::Battle,
            start_year: 120,
            end_year: None,
            sides: vec![1, 2],
            outcome: String::new(),
            points: vec![],
            anchor: None,
            anchor_at: (0.0, 0.0),
        };
        let input = kw.input(&StoryParts { landmarks: std::slice::from_ref(&lm), conflicts: std::slice::from_ref(&conflict), journeys: &[] });
        assert_eq!(input.landmarks.len(), 1);
        assert_eq!(input.landmarks[0].nearest_settlement.map(|(id, d)| (id, d)), Some((s0.tid as i64, 0.0)));
        assert_eq!(input.landmarks[0].cause.as_deref(), Some("a river drops -> a waterfall"));
        assert_eq!(input.conflicts[0].notes, None, "an empty outcome is absent, not an empty note");
        assert_eq!(input.conflicts[0].end_year, None);
        let docs = render_documents(&input);
        let c = docs.iter().find(|d| d.id == "W:conflict:3").unwrap();
        assert!(c.body.contains("started: year 120") && c.body.contains("ended: not available (it is still ongoing"), "{}", c.body);
        assert!(docs.iter().any(|d| d.id == "W:landmark:7" && d.body.contains("kind: waterfall")));
    }

    /// RK-1's timing bar, MEASURED (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.8: "render plus index build at
    /// the default grid, target at most 1 s median, min..max reported, run alone"). `#[ignore]`d: a
    /// timing is a measurement, not an assertion. Three worlds at the default 2048-wide grid; per world
    /// five repetitions of (a) collecting the input from the engine state, (b) rendering the
    /// documents, (c) building the BM25 index over them. Prints one line per world and the medians.
    /// Profile: the workspace `dev` profile (`opt-level = 1`); a release build is faster.
    /// `cargo test -p cartalith-godot --lib measure_knowledge_render -j 2 -- --ignored --nocapture`
    #[test]
    #[ignore = "a measurement: run alone"]
    fn measure_knowledge_render_and_index_at_the_default_grid() {
        use std::time::Instant;
        let mut all: Vec<(f64, f64, f64)> = Vec::new();
        for seed in SEEDS {
            let (ws, civ, gh, p) = crate::fh_r2_baseline::generate_world(seed, 800.0, 8, 2048);
            let kw = KnowledgeWorld { civ: &civ, ws: &ws, gw: 2048, gh, sea: ws.sea_level, map_width_km: p.map_width_km, seed, world: p.world, peak_m: p.peak_m };
            let story = StoryParts { landmarks: &[], conflicts: &[], journeys: &[] };
            let (mut tc, mut tr, mut tb) = (Vec::new(), Vec::new(), Vec::new());
            let mut docs = 0;
            for _ in 0..5 {
                let t0 = Instant::now();
                let input = kw.input(&story);
                tc.push(t0.elapsed().as_secs_f64());
                let t1 = Instant::now();
                let side = cartalith_knowledge::search::WorldSide::from_input(&input);
                tr.push(t1.elapsed().as_secs_f64());
                let t2 = Instant::now();
                let idx = cartalith_knowledge::search::KnowledgeIndex::build(Some(&side), None);
                tb.push(t2.elapsed().as_secs_f64());
                docs = idx.doc_count();
            }
            let med = |v: &mut Vec<f64>| {
                v.sort_by(|a, b| a.total_cmp(b));
                v[v.len() / 2]
            };
            let rng = |v: &Vec<f64>| format!("{:.3}..{:.3}", v.iter().cloned().fold(f64::MAX, f64::min), v.iter().cloned().fold(0.0, f64::max));
            let (rc, rr, rb) = (rng(&tc), rng(&tr), rng(&tb));
            let (mc, mr, mb) = (med(&mut tc), med(&mut tr), med(&mut tb));
            println!(
                "seed {seed}: {} settlements, {docs} documents; collect {mc:.3} s ({rc}), render {mr:.3} s ({rr}), index build {mb:.3} s ({rb}); render+build {:.3} s",
                civ.settlements.len(),
                mr + mb
            );
            all.push((mc, mr, mb));
        }
        let m = |f: fn(&(f64, f64, f64)) -> f64| {
            let mut v: Vec<f64> = all.iter().map(f).collect();
            v.sort_by(|a, b| a.total_cmp(b));
            v[1]
        };
        println!("median over the three worlds: collect {:.3} s, render {:.3} s, build {:.3} s, render+build {:.3} s", m(|x| x.0), m(|x| x.1), m(|x| x.2), m(|x| x.1) + m(|x| x.2));
    }

    /// The directory that holds the committed evaluation fixtures (`cartalith-knowledge`'s tests).
    fn fixture_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("cartalith-knowledge").join("tests").join("fixtures")
    }

    /// REGENERATES the three committed fixtures (`knowledge_eval_<seed>.json`) from real generated
    /// worlds: the world input as this bridge collects it today, the synthetic-vault hash and the
    /// questions. `#[ignore]`d on purpose (it writes files); run it, commit the three files, and
    /// `fixtures_are_current_for_all_three_seeds` in `cartalith-knowledge` then holds.
    /// `cargo test -p cartalith-godot --lib regenerate_knowledge_fixtures -- --ignored --nocapture`
    #[test]
    #[ignore = "writes the committed fixtures; run deliberately"]
    fn regenerate_knowledge_fixtures() {
        for seed in SEEDS {
            let (input, ..) = collect(seed);
            let fx = cartalith_knowledge::eval::build_fixture(&input, "2026-10-26");
            let path = fixture_dir().join(format!("knowledge_eval_{seed}.json"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, serde_json::to_string(&fx).unwrap()).unwrap();
            println!("seed {seed}: {} settlements, {} documents, {} questions, {} notes -> {}", input.settlements.len(), render_documents(&input).len(), fx.questions.len(), fx.vault_notes, path.display());
        }
    }

    /// Protects the fixtures' provenance: the world inside each committed fixture EQUALS what this
    /// bridge collects from the engine today, so the fixture is a real producer's output and not a
    /// hand-written sentinel (`MISTAKES.md`: build a fixture from the real producer's output). A
    /// change to world generation, the civilisation layer or this collection that is not followed by a
    /// regeneration fails here, by name.
    #[test]
    fn committed_fixture_worlds_equal_what_the_engine_produces_today() {
        for seed in SEEDS {
            let (input, ..) = collect(seed);
            let path = fixture_dir().join(format!("knowledge_eval_{seed}.json"));
            let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} is missing ({e}); run regenerate_knowledge_fixtures", path.display()));
            let fx: cartalith_knowledge::eval::Fixture = serde_json::from_str(&text).unwrap();
            assert_eq!(fx.seed, i64::from(seed));
            assert!(fx.world == input, "seed {seed}: the committed world differs from the engine's output; regenerate");
        }
    }
}
