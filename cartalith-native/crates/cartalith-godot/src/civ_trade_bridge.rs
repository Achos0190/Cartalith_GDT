//! CIVIL ▸ Trade — `GUI_GAP_REGISTER.md` **IN-13**, plus `ECONOMY_SCOPE.md`
//! milestone 2's food shed (2026-09-01).
//!
//! Two entry points, each its own dictionary shape. Its own file with a
//! `#[godot_api(secondary)]` block, the shape `civ_military_bridge.rs` and
//! `geojson_bridge.rs` already use.
//!
//! - [`WorldGen::civ_trade_flows`] runs
//!   [`cartalith_civ::trade::trade_flows`] — the fifteen-good bipartite
//!   match over the whole world. See "Why the whole answer" below.
//! - [`WorldGen::civ_food_shed`] runs
//!   [`cartalith_civ::trade::civ_food_shed`] once per settlement, sharing
//!   one [`cartalith_civ::trade::RoadComponents`] union-find across all of
//!   them the way that function's own doc comment asks a real caller to.
//!   It is `food`'s own route — the one good `trade_flows`'s fifteen-key
//!   match never touches (`crate::trade`'s own module doc says why) — and
//!   reports local/hinterland/import capacity per settlement rather than
//!   routed flows. See "`civ_food_shed` recomputes..." below for the one
//!   thing it does not thread through from `compute_civilisation`.
//!
//! ## Why the whole answer, and not a query API
//!
//! Three surfaces read this: the Trade category's summary, the place
//! editor's per-settlement ledger, and the map's way-load overlay. A
//! per-settlement `#[func]` would either re-run the match on every place
//! editor open (a quarter of a second, for one settlement's four rows) or
//! force a cache onto `WorldGen` — and the design's whole premise, the same
//! one `civ_territory_influence` shipped on, is that **nothing is retained
//! in the engine**.
//!
//! So the engine computes once, hands the shell everything, and forgets.
//! What the shell does with the dictionary — `trade_store.gd` keeps the last
//! one and drops it on `world_changed` — is the shell's business, and a
//! GDScript dictionary it can free is a very different thing from a
//! `CivData` field that survives every generate.
//!
//! ## What is not here
//!
//! No `way_load` raster. Trade load is drawn by thickening the ways
//! `map_overlay.gd` already draws, on their own `WAY_STYLE` colour — see
//! the register's §42 for why that belongs to CARTO ▸ Roads & routes rather
//! than to the Layers popover, which owns *field* rasters and nothing else.
//!
//! ## `civ_food_shed` recomputes `lithology`/`soil` rather than reading a
//! stored field
//!
//! `compute_civilisation` (`lib.rs:818`) computes both as locals feeding
//! `build_carrying_capacity`/`build_settlement_suitability`, then drops
//! them at the end of the function — `CivData` has no field for either,
//! unlike `dens` (`civ_current_agrarian_density`, a different pass that
//! *is* kept past `compute_civilisation`'s return, precisely so callers
//! like this one do not have to redo it). [`WorldGen::civ_food_shed`]'s own
//! doc comment records the tradeoff in full and the thorough fix: a
//! `CivData::soil` field threaded from that existing local, which touches
//! `lib.rs` and was outside this file's ownership when this was written.

use godot::prelude::*;

use cartalith_civ::trade::{
    FoodShed, FoodShedInput, Navigability, RoadComponents, TradeInput, TradeMode, TradeNetwork,
};
use cartalith_civ::urban_adapter::UrbanWorld;

use crate::{WorldGen, WorldSource};

/// Rows past this are summarised rather than listed. The dock shows a dozen;
/// the cap exists so a pathological world cannot hand GDScript a
/// hundred-thousand-element array, not because anything here is expensive.
const MAX_FLOW_ROWS: usize = 4000;

impl WorldGen {
    /// Run the match, or `None` when there is no generated world with a
    /// civilisation layer and at least one settlement.
    fn trade_network(&self) -> Option<(TradeNetwork, u128)> {
        let (Some(WorldSource::Generated(ws)), Some(civ)) =
            (self.source.as_ref(), self.civ.as_ref())
        else {
            return None;
        };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 || civ.settlements.is_empty() {
            return None;
        }
        // `um_site_kind_from_terrain` reads `field`, `flow` and
        // `flow_thresh` and nothing else, so the expensive
        // `trace_river_polylines` pass `urban_bridge.rs` hoists is
        // deliberately not made here — an empty slice is the reference's own
        // "no traced network" case and this code path never consults it.
        let polys: Vec<Vec<(f64, f64)>> = Vec::new();
        let world = UrbanWorld {
            field: &ws.field,
            flow: &ws.flow_discharge,
            water_bodies: &civ.water_bodies,
            order: ws.stream_order.as_deref(),
            river_polys: &polys,
            gw,
            gh,
            sea_level: self.sea_level,
            map_width_km: self.map_width_km,
            // The same call `compute_civilisation` makes, with the same
            // arguments — a settlement's site kind must agree with the river
            // network the rest of the civ layer was built against.
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, self.map_width_km),
            world_seed: self.seed,
        };
        let input = TradeInput {
            settlements: &civ.settlements,
            balances: &civ.trade_balances,
            ways: &civ.ways,
            map_width_km: self.map_width_km,
            gw,
        };
        let t0 = std::time::Instant::now();
        let net = cartalith_civ::trade::trade_flows(&input, &world);
        Some((net, t0.elapsed().as_millis()))
    }

    /// Every settlement's food-shed capacity — `_civFoodShed` run once per
    /// settlement, sharing one [`RoadComponents`] union-find the way
    /// [`cartalith_civ::trade::civ_food_shed`]'s own doc comment asks a
    /// real caller to. `None` under the same conditions
    /// [`Self::trade_network`] returns `None`.
    ///
    /// ## `lithology`/`soil` are recomputed here, not threaded from
    /// `compute_civilisation`
    ///
    /// See this file's own module doc for the full statement. In short:
    /// `CivData` carries no field for either, so this runs the same two
    /// calls `compute_civilisation` (`lib.rs:818`) makes, on this
    /// on-demand call and nothing that runs per-frame — the same tradeoff
    /// `civ_military_bridge.rs::aggregates_with_walls` already makes for
    /// `lithology`/`resources` (see that method's own doc comment: "rebuilt
    /// on demand... the reason both are modal-open calls rather than
    /// per-frame ones").
    fn food_shed_rows(&self) -> Option<(Vec<FoodShed>, u128, usize)> {
        let (Some(WorldSource::Generated(ws)), Some(civ)) =
            (self.source.as_ref(), self.civ.as_ref())
        else {
            return None;
        };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 || civ.settlements.is_empty() {
            return None;
        }
        let t0 = std::time::Instant::now();
        let sea_level = self.sea_level;

        // Same `UrbanWorld` shape `trade_network()` builds above, with the
        // same deliberately-empty `river_polys` — `place_navigability`'s
        // branch (b) reads `field`/`flow`/`flow_thresh` only, never the
        // traced polylines (see `trade_network`'s own comment).
        let polys: Vec<Vec<(f64, f64)>> = Vec::new();
        let world = UrbanWorld {
            field: &ws.field,
            flow: &ws.flow_discharge,
            water_bodies: &civ.water_bodies,
            order: ws.stream_order.as_deref(),
            river_polys: &polys,
            gw,
            gh,
            sea_level,
            map_width_km: self.map_width_km,
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, self.map_width_km),
            world_seed: self.seed,
        };
        let navigability: Vec<Navigability> = civ
            .settlements
            .iter()
            .map(|s| cartalith_civ::trade::place_navigability(&world, s))
            .collect();

        // Per-settlement `farmers_per_urbanite` — the settlement's OWN
        // faction's ag-tech level, via `civ_ag_tech_by_key`: the exact
        // route `civ_military_bridge.rs:396` already uses for the
        // manpower model's agricultural-labour-ratio term. Out-of-range
        // falls back to `FARMERS_PER_URBANITE`, matching
        // `FoodShedInput::farmers_per_urbanite`'s own documented contract.
        let farmers_per_urbanite: Vec<f64> = civ
            .settlements
            .iter()
            .map(|s| {
                civ.faction_roster
                    .0
                    .get(s.placement.faction.max(0) as usize)
                    .map_or(cartalith_civ::timeline::FARMERS_PER_URBANITE, |e| {
                        cartalith_civ::roster::civ_ag_tech_by_key(&e.ag_tech).farmers_per_urbanite
                    })
            })
            .collect();

        // The same two calls `compute_civilisation` makes (`lib.rs:814-818`)
        // to build `soil` — see this file's module doc and this method's
        // own doc comment for why they run again here instead of reading a
        // stored field.
        let soil_slope = cartalith_civ::build_slope_field(&ws.field, gw, gh, self.world);
        let lithology = cartalith_civ::build_lithology(
            &ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field,
            &ws.rainfall, sea_level,
        );
        let soil = cartalith_civ::build_soil_fertility(
            &lithology, &ws.temperature, &ws.rainfall, &soil_slope, &ws.age_field,
        );
        let soil_ref = cartalith_civ::timeline::civ_soil_reference(&soil, &ws.field, sea_level);

        let input = FoodShedInput {
            settlements: &civ.settlements,
            navigability: &navigability,
            farmers_per_urbanite: &farmers_per_urbanite,
            dens: &civ.dens,
            soil: &soil,
            soil_ref,
            field: &ws.field,
            gw,
            gh,
            sea: sea_level,
            world_wrap: self.world,
            map_width_km: self.map_width_km,
        };
        // Built once and reused across every settlement — `civ_food_shed`'s
        // own doc comment on its `rc` parameter asks exactly this of a real
        // caller, the same reuse `civ_military_bridge.rs`'s manpower pass
        // already gives its own `RoadComponents`.
        let mut rc = RoadComponents::build(civ.settlements.len(), &civ.ways);
        let rows: Vec<FoodShed> = (0..civ.settlements.len())
            .map(|i| cartalith_civ::trade::civ_food_shed(&input, &mut rc, i))
            .collect();

        // Itemised like `trade_flows`'s own `transient_bytes`: everything
        // this pass allocated that `compute_civilisation` did not already
        // leave resident — `ws.field`/`temperature`/`rainfall`/`age_field`
        // etc. are pre-existing `WorldState` storage, not counted here.
        let bytes = soil_slope.len() * std::mem::size_of::<f32>()
            + lithology.len() * std::mem::size_of::<u8>()
            + soil.len() * std::mem::size_of::<f32>()
            + navigability.len() * std::mem::size_of::<Navigability>()
            + farmers_per_urbanite.len() * std::mem::size_of::<f64>()
            + rows.len() * std::mem::size_of::<FoodShed>();

        Some((rows, t0.elapsed().as_millis(), bytes))
    }
}

#[godot_api(secondary)]
impl WorldGen {
    /// Match every settlement's surplus to every deficit it can actually
    /// reach, and route what lands on a road.
    ///
    /// **Derived on demand and held nowhere** — the same contract
    /// [`WorldGen::civ_territory_influence`] ships on. Nothing is added to
    /// `CivData`, nothing is saved, and calling this twice on an unchanged
    /// world returns the same answer.
    ///
    /// `{}` before any `generate()`, on a loaded save (which carries no
    /// civilisation layer at all), and on a world with no settlements.
    ///
    /// Returned shape:
    /// - `flow_count`, `goods_moving`, `importing`, `supplied`,
    ///   `unmet_count` — world totals.
    /// - `land_share` / `river_share` / `sea_share` — the fraction of
    ///   matched **volume** each mode carries, not of flow count: one sea
    ///   lane moving a city's demand is not one flow's worth of trade.
    /// - `goods` — one row per resource key that actually moves, with its
    ///   exporter and importer counts, total volume, bulk/luxury class and
    ///   dominant mode.
    /// - `flows` — every matched flow (`from`/`to` are indices into
    ///   `get_settlements()`), capped at [`MAX_FLOW_ROWS`].
    /// - `unmet` — one row per settlement with a need nothing can fill.
    /// - `navigability` — per settlement, in settlement order.
    /// - `way_load` — per way, in `get_roads()` order.
    /// - `ways` — the eight busiest, named.
    /// - `elapsed_ms`, `transient_bytes`, `resident_bytes` (always `0`).
    #[func]
    fn civ_trade_flows(&self) -> VarDictionary {
        let Some((net, ms)) = self.trade_network() else {
            return VarDictionary::new();
        };
        let civ = self.civ.as_ref().expect("trade_network() proved this is Some");
        let name_of = |i: usize| -> String {
            civ.settlements.get(i).map_or_else(String::new, |s| s.name.clone())
        };

        // ---- per-good aggregation, in CIV_RESOURCE_KEYS order ----
        let keys = cartalith_civ::CIV_RESOURCE_KEYS;
        let mut vol = vec![0.0f64; keys.len()];
        let mut by_mode = vec![[0.0f64; 3]; keys.len()];
        let mut exporters: Vec<std::collections::BTreeSet<usize>> =
            vec![Default::default(); keys.len()];
        let mut importers: Vec<std::collections::BTreeSet<usize>> =
            vec![Default::default(); keys.len()];
        let mut supplied: std::collections::BTreeSet<usize> = Default::default();
        let mut mode_vol = [0.0f64; 3];
        let key_index = |g: &str| keys.iter().position(|&k| k == g);

        for f in net.flows.iter() {
            let Some(gi) = key_index(f.good) else { continue };
            vol[gi] += f.volume;
            by_mode[gi][f.mode as usize] += f.volume;
            mode_vol[f.mode as usize] += f.volume;
            exporters[gi].insert(f.from);
            importers[gi].insert(f.to);
            supplied.insert(f.to);
        }
        let total_vol: f64 = mode_vol.iter().sum();
        let share = |v: f64| if total_vol > 0.0 { v / total_vol } else { 0.0 };

        let goods: Array<VarDictionary> = (0..keys.len())
            .filter(|&gi| vol[gi] > 0.0)
            .map(|gi| {
                let m = by_mode[gi];
                let dom = if m[2] >= m[1] && m[2] >= m[0] {
                    TradeMode::Sea
                } else if m[1] >= m[0] {
                    TradeMode::River
                } else {
                    TradeMode::Land
                };
                dict! {
                    "key" => keys[gi],
                    "name" => cartalith_civ::RESOURCE_NAMES
                        .get(gi)
                        .copied()
                        .unwrap_or(keys[gi]),
                    "volume" => vol[gi],
                    "exporters" => exporters[gi].len() as i64,
                    "importers" => importers[gi].len() as i64,
                    "bulk" => cartalith_civ::trade::BULK_GOODS.contains(&keys[gi]),
                    "dominant_mode" => dom.as_str(),
                }
            })
            .collect();

        // ---- flows, biggest first, so a truncated list is the useful end ----
        let mut order: Vec<usize> = (0..net.flows.len()).collect();
        order.sort_by(|&a, &b| {
            net.flows[b]
                .volume
                .partial_cmp(&net.flows[a].volume)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.cmp(&b))
        });
        let flows: Array<VarDictionary> = order
            .iter()
            .take(MAX_FLOW_ROWS)
            .map(|&i| {
                let f = &net.flows[i];
                dict! {
                    "from" => f.from as i64,
                    "to" => f.to as i64,
                    "from_name" => name_of(f.from),
                    "to_name" => name_of(f.to),
                    "good" => f.good,
                    "mode" => f.mode.as_str(),
                    "reach" => f.reach.as_str(),
                    "distance_km" => f.distance_km,
                    "deliverable" => f.deliverable,
                    "volume" => f.volume,
                }
            })
            .collect();

        // ---- unmet needs, grouped per settlement ----
        let mut unmet_by: std::collections::BTreeMap<usize, (Vec<&'static str>, bool)> =
            Default::default();
        for u in net.unmet.iter() {
            let e = unmet_by.entry(u.settlement).or_insert_with(|| (Vec::new(), false));
            e.0.push(u.good);
            e.1 |= u.exporter_exists;
        }
        let unmet: Array<VarDictionary> = unmet_by
            .iter()
            .map(|(&i, (goods, any))| {
                let names: PackedStringArray = goods.iter().map(|&g| GString::from(g)).collect();
                dict! {
                    "index" => i as i64,
                    "name" => name_of(i),
                    "goods" => &names,
                    "exporter_exists" => *any,
                }
            })
            .collect();

        let navigability: Array<VarDictionary> = net
            .navigability
            .iter()
            .enumerate()
            .map(|(i, n)| {
                dict! {
                    "index" => i as i64,
                    "kind" => n.kind.as_str(),
                    "navigable" => n.kind.navigable(),
                    "basis" => n.basis,
                }
            })
            .collect();

        let mut way_order: Vec<usize> = (0..net.way_load.len()).collect();
        way_order.sort_by(|&a, &b| {
            net.way_load[b]
                .partial_cmp(&net.way_load[a])
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.cmp(&b))
        });
        let ways: Array<VarDictionary> = way_order
            .iter()
            .take(8)
            .filter(|&&i| net.way_load[i] > 0.0)
            .map(|&i| {
                dict! {
                    "index" => i as i64,
                    "name" => civ.ways.get(i).map_or_else(String::new, |w| w.name.clone()),
                    "load" => net.way_load[i],
                }
            })
            .collect();
        let idle_ways = net.way_load.iter().filter(|&&v| v <= 0.0).count() as i64;

        let mut d = dict! {
            "flow_count" => net.flows.len() as i64,
            "flow_rows" => flows.len() as i64,
            "goods_moving" => goods.len() as i64,
            "importing" => (supplied.len() + unmet_by.len()) as i64,
            "supplied" => supplied.len() as i64,
            "unmet_count" => net.unmet.len() as i64,
            "settlement_count" => civ.settlements.len() as i64,
            "total_volume" => total_vol,
            "land_share" => share(mode_vol[0]),
            "river_share" => share(mode_vol[1]),
            "sea_share" => share(mode_vol[2]),
            "way_count" => net.way_load.len() as i64,
            "idle_ways" => idle_ways,
            "elapsed_ms" => ms as i64,
            "transient_bytes" => net.transient_bytes as i64,
            // Deliberately reported, and deliberately zero: this is the
            // number the register's own memory objection was about.
            "resident_bytes" => 0i64,
        };
        d.set("goods", &goods);
        d.set("flows", &flows);
        d.set("unmet", &unmet);
        d.set("navigability", &navigability);
        d.set("ways", &ways);
        // **`get_roads()` order, not `civ.ways` order.** The overlay indexes
        // this array by its own row, and `get_roads()` filters hidden ways out
        // and appends the manual ones -- so a straight copy of `net.way_load`
        // is off by however many ways are hidden, silently, from the first
        // hidden way onward. Found by the probe asserting the two lengths
        // match, which is why that assertion is worth making.
        let mut load = PackedFloat32Array::new();
        for (i, w) in civ.ways.iter().enumerate() {
            if !w.hidden {
                load.push(net.way_load.get(i).copied().unwrap_or(0.0) as f32);
            }
        }
        // Manual ways carry no matched trade: they are not in the
        // consolidation the match's union-find reads, so they have no load
        // rather than a load of zero by accident. Padded so the array still
        // lines up with `get_roads()`.
        if let Some(infra) = self.infra.as_ref() {
            for _ in infra.ways.iter().filter(|w| !w.hidden && !w.sea) {
                load.push(0.0);
            }
        }
        d.set("way_load", &load);
        d
    }

    /// CIVIL ▸ Trade's food half — `_civFoodShed` run for every settlement:
    /// its own catchment ceiling, the countryside within land reach, and
    /// genuine spare capacity imported from elsewhere over the cheapest
    /// mode both ends share (`ECONOMY_SCOPE.md` milestone 2).
    ///
    /// **Derived on demand and held nowhere**, the same contract
    /// [`WorldGen::civ_trade_flows`] above ships on — see this file's
    /// module doc for the one place that contract costs more here than
    /// there (the `lithology`/`soil` recompute).
    ///
    /// `{}` before any `generate()`, on a loaded save (which carries no
    /// civilisation layer at all), and on a world with no settlements.
    ///
    /// Returned shape:
    /// - `rows` — one entry per settlement, in `get_settlements()` order:
    ///   `index`, `name`, `pop`, `local_capacity`, `hinterland_capacity`,
    ///   `import_capacity`, `supported`, `suppliers`, `best_mode`
    ///   (`land`/`river`/`sea`), `limited_by` (`local`/`trade`),
    ///   `sustainable`, `over_by`.
    /// - `settlement_count`, `unsustainable_count` — world totals.
    /// - `elapsed_ms`, `transient_bytes`, `resident_bytes` (always `0`) —
    ///   the same three [`WorldGen::civ_trade_flows`] reports, and for the
    ///   same reason.
    #[func]
    fn civ_food_shed(&self) -> VarDictionary {
        let Some((rows, ms, bytes)) = self.food_shed_rows() else {
            return VarDictionary::new();
        };
        let civ = self.civ.as_ref().expect("food_shed_rows() proved this is Some");

        let unsustainable = rows.iter().filter(|r| !r.sustainable).count() as i64;
        let out_rows: Array<VarDictionary> = rows
            .iter()
            .enumerate()
            .map(|(i, r)| {
                dict! {
                    "index" => i as i64,
                    "name" => civ.settlements.get(i).map_or_else(String::new, |s| s.name.clone()),
                    "pop" => civ.settlements.get(i).map_or(0i64, |s| s.pop as i64),
                    "local_capacity" => r.local_capacity,
                    "hinterland_capacity" => r.hinterland_capacity,
                    "import_capacity" => r.import_capacity,
                    "supported" => r.supported,
                    "suppliers" => r.suppliers as i64,
                    "best_mode" => r.best_mode.as_str(),
                    "limited_by" => r.limited_by,
                    "sustainable" => r.sustainable,
                    "over_by" => r.over_by,
                }
            })
            .collect();

        let mut d = dict! {
            "settlement_count" => civ.settlements.len() as i64,
            "unsustainable_count" => unsustainable,
            "elapsed_ms" => ms as i64,
            "transient_bytes" => bytes as i64,
            // Deliberately reported, and deliberately zero — see
            // `civ_trade_flows`'s own comment on the same key.
            "resident_bytes" => 0i64,
        };
        d.set("rows", &out_rows);
        d
    }
}
