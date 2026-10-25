//! The generation-parameter catalogue for the language-model assistant
//! (`LLM_ASSISTANT_SCOPE.md` s.5b.1, milestone LLM-6a; owner Ruling DI).
//!
//! WHY a side table and not a new `ParamSpec` field: the registry rows in
//! `params.rs` are the single source of truth for ranges and labels, and editing
//! 120 rows to add prose would bury them. This file follows `JS_PATHS`' precedent
//! (scope s.5b.1): `DESCS` is a side table keyed by the registry key, with a test
//! (`tests/param_catalogue.rs`) that fails when an offered key has no row, so a
//! new parameter cannot reach a model undescribed. [`build`] joins the live
//! registry (`params::PARAMS`), the app defaults (`params::defaults()`), the
//! world's current values (from `WorldGen.get_params()`, passed in), `DESCS`,
//! `INERT_WHEN`, `BLURBS` and `params::invalidates` into a
//! `cartalith_llm::catalogue::Catalogue` on **every request**; nothing about a
//! parameter's type, range or label is written a second time here.
//!
//! Godot-free on purpose, like `params.rs`, so the test target can include both
//! by `#[path]` and check the REAL rows under a plain `cargo test`.
//!
//! **How the descriptions were written.** Each is one sentence pair, at most 160
//! characters: `Raise: <what raising does>. Lower: <what lowering does>.` for a
//! number, `On: ... Off: ...` for a switch, written from the "what it does" column
//! of `GENERATION_PARAMETERS.md` (read 2026-10-26) and reviewed against it. The
//! direction is the point: the LLM-6c stage-1 run saw labels only. The one the
//! scope singles out, `climate.rain_dep` ("Dryness"), says what the document
//! says: raising it makes air rain out sooner, so land past the first ridge is
//! drier and wet/dry edges sharper; it is not a global dryness dial, and "drier"
//! overall lives in `climate.ocean`, `climate.evap` and `climate.ocean_hum` (all
//! in the `weather` group). A test pins the direction words for the keys the
//! stage-1 report scored. Tests cannot prove a description TRUE (scope s.5b.7
//! item 6); a change to a parameter's behaviour must re-read its row here.
//!
//! MUST NEVER: offer an excluded key; invent a value for an absent current
//! (it is omitted); write a parameter (this file only describes them).

use cartalith_engine::WorldParams;
use cartalith_engine::staleness::PipelineStage;
use cartalith_llm::catalogue::{Catalogue, Group, Kind, PValue, Regen, Row};

use crate::params;

/// Keys never offered to a model (scope s.5b.1, a labelled judgement): `use_gpu`
/// is a machine setting, not a property of the world, and the five geology-first
/// switches have no GUI control by their own row comments in `params.rs`.
pub const EXCLUDED: &[&str] =
    &["use_gpu", "geology_model", "geology_processes", "relief_model", "geology_karst", "geology_carve"];

/// One line per registry group: what a user means by it, in their words (scope
/// s.5b.1's table and s.5b.6's prompt, as the stage-1 prototype measured them).
/// A group of `params::groups()` with no row here makes `Catalogue::problems()`
/// report it, which fails the tests and stops the bridge sending the catalogue.
pub const BLURBS: &[(&str, &str)] = &[
    ("world", "sea level, peak altitude, river density, whole-world wrap"),
    ("planet", "gravity, day length, axial tilt"),
    ("world_structure", "continent-scale steering (continentality, fragmentation, ...)"),
    ("tectonics", "plates, drift, mountain-building weights, rock resistance"),
    ("volcanism", "volcano and crater counts, ages, kinds"),
    ("erosion", "geological age, rivers, glaciers, coast, sediment"),
    ("climate", "latitude span, temperatures, lapse rate, ice, ocean currents"),
    ("weather", "RAINFALL: orographic rain, evaporation, dryness, ocean moisture"),
    ("civ", "factions and settlement counts"),
];

/// When a valid change does nothing: `(key, gate key, gate value the key needs,
/// reason shown)`. A change to `key` while the gate key's CURRENT value is not
/// the needed one is still valid (the user may flip the gate next), but the card
/// and the schema say it will have no effect. Each row is from
/// `GENERATION_PARAMETERS.md`'s own "ignored when" / "only read when" text.
pub const INERT_WHEN: &[(&str, &str, bool, &str)] = &[
    ("sea_level", "world_structure.enabled", false, "continental steering is on, and it sets sea level from continentality"),
    ("tect.plates", "world_structure.enabled", false, "continental steering is on, and it derives the plate count"),
    ("tect.vel", "world_structure.enabled", false, "continental steering is on, and it derives the drift"),
    ("volc.count", "world_structure.enabled", false, "continental steering is on, and hotspot density sets the count"),
    ("world_structure.continentality", "world_structure.enabled", true, "continental steering is off"),
    ("world_structure.fragmentation", "world_structure.enabled", true, "continental steering is off"),
    ("world_structure.tectonic_energy", "world_structure.enabled", true, "continental steering is off"),
    ("world_structure.ocean_depth", "world_structure.enabled", true, "continental steering is off"),
    ("world_structure.hotspot_density", "world_structure.enabled", true, "continental steering is off"),
    ("tect.high_ceiling", "tect.orogen_relief", true, "orogen relief is off"),
    ("volc.fill_edifice_pits", "volc.edifice_model", true, "shaped edifices are off"),
    ("volc.morphometry", "volc.edifice_model", true, "shaped edifices are off"),
    ("volc.cinder_grid_law", "volc.morphometry", true, "volcano morphometry is off"),
    ("crater.count", "crater.physical_model", false, "the physical crater model is on and sets the count"),
    ("crater.surface_age_myr", "crater.physical_model", true, "the physical crater model is off"),
    ("geo.age", "geology_processes", true, "landform processes do not read the rock column in this build"),
    ("passes.velo_iters", "passes.velocity", true, "velocity erosion is off"),
    ("passes.velo_strength", "passes.velocity", true, "velocity erosion is off"),
    ("passes.velo_meander", "passes.velocity", true, "velocity erosion is off"),
    ("passes.glacial_snowline", "passes.glacial", true, "glacial erosion is off"),
    ("passes.glacial_kg", "passes.glacial", true, "glacial erosion is off"),
    ("passes.glacial_mg", "passes.glacial", true, "glacial erosion is off"),
    ("passes.glacial_u_factor", "passes.glacial", true, "glacial erosion is off"),
    ("passes.glacial_passes", "passes.glacial", true, "glacial erosion is off"),
    ("passes.wave_str", "passes.coastal", true, "coastal processes are off"),
    ("passes.estuary_depth", "passes.coastal", true, "coastal processes are off"),
    ("passes.marsh_band", "passes.coastal", true, "coastal processes are off"),
    ("passes.coastal_passes", "passes.coastal", true, "coastal processes are off"),
    ("passes.diffuse_passes", "passes.hillslope", true, "hillslope diffuse is off"),
    ("passes.sediment_capacity", "passes.sediment_fill", true, "sediment fill is off"),
    ("passes.tidal_k", "passes.tidal_flats", true, "tidal flats are off"),
    ("climate.lat_n", "world", false, "the map is a whole world, pole to pole"),
    ("climate.lat_s", "world", false, "the map is a whole world, pole to pole"),
    ("climate.current_k", "climate.currents", true, "ocean currents are off"),
    ("climate.wind_dir_deg", "climate.wind_manual", true, "the wind is planetary, not manual"),
    ("civ.seed_thresh", "civ.fixed_counts", false, "fixed settlement counts are on"),
    ("civ.seed_suppress_div", "civ.fixed_counts", false, "fixed settlement counts are on"),
    ("civ.n_capital", "civ.fixed_counts", true, "fixed settlement counts are off"),
    ("civ.n_city", "civ.fixed_counts", true, "fixed settlement counts are off"),
    ("civ.n_town", "civ.fixed_counts", true, "fixed settlement counts are off"),
    ("civ.n_village", "civ.fixed_counts", true, "fixed settlement counts are off"),
    ("civ.n_hamlet", "civ.fixed_counts", true, "fixed settlement counts are off"),
];

/// What each offered key does, in the user's words: `Raise: .. Lower: ..` or
/// `On: .. Off: ..`, at most 160 characters. See the module docs for the source.
/// "(current)" marks the value the shipped app ships for a ruled switch.
pub const DESCS: &[(&str, &str)] = &[
    // ---- world
    ("world", "On: the map is a whole planet that wraps east-west, pole to pole. Off: a framed region between two latitudes."),
    ("sea_level", "Raise: more of the map is sea, less land. Lower: more land; 0 leaves the least sea. A height threshold, not metres."),
    ("peak_m", "Raise: taller mountains and colder, snowier peaks (it scales all heights). Lower: lower relief, lower mountains, milder heights."),
    ("carve_rivers", "On: rivers cut real valleys into the terrain while generating. Off: no river channels are carved at all."),
    ("river_density", "Raise: more, smaller river channels. Lower: fewer, larger rivers."),
    ("integrate_drainage", "On: water reaching a hollow carries on to the sea, so rivers connect (current). Off: rivers end in local pits (older)."),
    // ---- planet
    ("planet.g", "Raise: stronger gravity, lower mountains, smaller craters, colder heights. Lower: taller mountains and bigger craters."),
    ("planet.rotation_hours", "Raise: a longer day, fewer broader wind belts, a milder pole-equator contrast. Lower: a fast spin with many wind belts."),
    ("planet.axial_tilt_deg", "Raise: a flatter temperature contrast between equator and poles. Lower: a sharper one, hot tropics and cold poles."),
    // ---- world_structure
    ("world_structure.enabled", "On: continents follow the five steering knobs and sea level is set from continentality. Off: plain random plates."),
    ("world_structure.continentality", "Raise: more land and less ocean. Lower: more ocean, less land. Works only with continental steering on."),
    ("world_structure.fragmentation", "Raise: many smaller landmasses and island chains. Lower: one big landmass. Works only with continental steering on."),
    ("world_structure.tectonic_energy", "Raise: stronger relief and faster drift, bigger mountains. Lower: calmer, lower relief. Works only with steering on."),
    ("world_structure.ocean_depth", "Raise: deeper ocean basins. Lower: shallower seas and wider shelves. Works only with continental steering on."),
    ("world_structure.hotspot_density", "Raise: more hotspots, so more volcanoes. Lower: fewer volcanoes. Works only with continental steering on."),
    // ---- tectonics
    ("tect.plates", "Raise: more plates, fragmented land, busier coasts. Lower: fewer plates and large continents. Ignored with steering on."),
    ("tect.vel", "Raise: faster drift, more stress at plate edges, taller ranges. Lower: calmer plate edges. Ignored with steering on."),
    ("tect.warp", "Raise: more organic, twisted coasts and ridgelines. Lower: geometric, blobby shapes."),
    ("tect.blur_r", "Raise: broad smooth swells instead of sharp ranges. Lower: sharp, narrow mountain belts."),
    ("tect.alpha", "Raise: plate tectonics dominates heights, stronger mountain belts. Lower: weaker tectonics, lower mountains, noise dominates."),
    ("tect.beta", "Raise: more rough fractal detail near plate boundaries. Lower: smoother terrain."),
    ("tect.age_inf", "Raise: sharp young arcs against flat old plate interiors. Lower: roughness spread more evenly."),
    ("tect.ridged", "On: sharp, crested mountain detail. Off: rolling, rounded hills."),
    ("tect.flexure", "Raise: broader arches around ranges, deeper rift basins, wider continental shelves. Lower: less of each."),
    ("tect.hetero", "Raise: more varied plate interiors, with basins and uplands. Lower: more uniform interiors."),
    ("tect.resist", "Raise: a wider spread of rock hardness; old shields resist river erosion more. Lower: rock erodes more evenly."),
    ("tect.dynamic_lithology", "On: crust stripped by erosion exposes harder rock. Off: rock hardness stays as first built."),
    ("tect.lloyd", "Raise: more evenly sized plates. Lower: more clustered, irregular plate sizes."),
    ("tect.narrow_plate_base_blur", "On: coastlines wander off the plate outlines (current). Off: coasts follow plate polygons more closely (older)."),
    ("tect.orogen_relief", "On: ridged, gullied ranges in the mountain belts (current). Off: the older, smoother mountain belts."),
    ("tect.high_ceiling", "On: tall volcanoes on high ground keep their summits (current). Off: the highest ground is clipped flat (older)."),
    // ---- volcanism
    ("volc.count", "Raise: more volcanoes. Lower: fewer volcanoes; 0 gives none. Ignored with continental steering on."),
    ("volc.age", "Raise: older, lower, softer volcanoes, more of them extinct. Lower: young, tall, sharp, more active volcanoes."),
    ("volc.provinces", "On: volcanoes cluster on arcs, rifts and hotspots with varied sizes. Off: a simple scatter along plate edges."),
    ("volc.exclude_transform", "On: no volcanoes on sliding (transform) plate edges (current). Off: older placement that puts some there."),
    ("volc.edifice_model", "On: shaped shield, strato and cinder volcanoes (current). Off: one plain cone shape at every size (older)."),
    ("volc.fill_edifice_pits", "On: small hollows on volcano flanks drain instead of holding tiny lakes (current). Off: older pitted flanks."),
    ("volc.morphometry", "On: cinder cones and summit craters sized from real volcano shapes (current). Off: the older sizes."),
    ("volc.cinder_grid_law", "On: cinder cones sized to keep their shape on the grid (current). Off: older, flatter cinder cones."),
    ("volc.dry_active_craters", "On: active volcanoes keep dry craters (current). Off: active craters can fill with water."),
    ("volc.lava_bench", "On: young lava flows reaching the sea build small new land (current). Off: no new land."),
    ("volc.kind_dome", "On: lava domes on the upper flanks of young stratovolcanoes (current). Off: no domes."),
    ("volc.kind_maar", "On: maar craters beside cinder cones on wet ground (current). Off: no maars."),
    ("volc.kind_caldera", "On: old arc stratovolcanoes collapse into calderas (current). Off: they keep summit craters."),
    ("volc.kind_supervolcano", "On: at most one supervolcano caldera where a hotspot sits under a continent (current). Off: none."),
    ("volc.kind_flood_basalt", "On: one dark flood-basalt plateau drawn at an old continental hotspot, no height moves (current). Off: none."),
    ("crater.count", "Raise: more impact craters. Lower: fewer; 0 gives none. Ignored while the physical crater model is on."),
    ("crater.age", "Raise: older, shallower, infilled craters. Lower: crisp young craters with central peaks."),
    ("crater.physical_model", "On: crater numbers and sizes follow surface age and map area (current). Off: a fixed crater count (older)."),
    ("crater.surface_age_myr", "Raise: an older surface with more, more worn craters. Lower: a young surface with few. Physical crater model only."),
    // ---- erosion
    ("geo.age", "Raise: more geological time for rock-aware processes. Lower: less. Read only when landform processes read rock."),
    ("stream.uplift", "Raise: ongoing uplift fights river incision, growing active ranges. Lower: rivers only carve; 0 is the default."),
    ("stream.k", "Raise: deeper, denser river valley networks. Lower: shallower, sparser valleys."),
    ("stream.iters", "Raise: more mature, graded river profiles. Lower: younger, less worn river profiles."),
    ("stream.deposit", "Raise: more floodplains and fans in low, flat reaches. Lower: less sediment laid down."),
    ("stream.climate_k", "Raise: wet regions erode much faster than dry ones. Lower: erosion depends less on rainfall."),
    ("passes.velocity", "On: a slow shallow-water erosion pass that makes meanders and oxbows. Off: skipped."),
    ("passes.velo_iters", "Raise: a longer meander simulation, more developed channels. Lower: a shorter one. Velocity erosion only."),
    ("passes.velo_strength", "Raise: the velocity pass erodes and carries more. Lower: gentler. Velocity erosion only."),
    ("passes.velo_meander", "Raise: more strongly meandering channels. Lower: straighter channels; 0 keeps them straight. Velocity erosion only."),
    ("passes.glacial", "On: glaciers carve U-shaped valleys and cirques where it is high and freezing (current). Off: no glacial carving."),
    ("passes.glacial_snowline", "Raise: a higher snowline, so less ground is glaciated. Lower: ice reaches lower ground. Glacial erosion only."),
    ("passes.glacial_kg", "Raise: glaciers carve deeper. Lower: lighter glacial carving. Glacial erosion only."),
    ("passes.glacial_mg", "Raise: big glaciers carve far more than small ones. Lower: carving is more even across sizes. Glacial erosion only."),
    ("passes.glacial_u_factor", "Raise: wider U-shaped glacial valleys. Lower: narrower, more V-shaped valleys. Glacial erosion only."),
    ("passes.glacial_passes", "Raise: more glacial passes, deeper troughs. Lower: fewer passes. Glacial erosion only."),
    ("passes.coastal", "On: sea cliffs retreat, river mouths widen into estuaries, marshes build. Off: coasts stay as generated."),
    ("passes.wave_str", "Raise: stronger surf and faster cliff retreat. Lower: calmer coasts. Coastal processes only."),
    ("passes.estuary_depth", "Raise: river mouths widen into estuaries further inland. Lower: narrower mouths. Coastal processes only."),
    ("passes.marsh_band", "Raise: a wider band of tidal marsh above sea level. Lower: a narrower band. Coastal processes only."),
    ("passes.coastal_passes", "Raise: more wave and estuary passes, more coastal change. Lower: fewer passes. Coastal processes only."),
    ("passes.hillslope", "On: hillslopes soften, rounding ridges and smoothing relief everywhere. Off: skipped."),
    ("passes.diffuse_d", "Raise: softer hillslopes and more worn craters. Lower: crisper relief. Wears craters even with hillslope diffuse off."),
    ("passes.diffuse_passes", "Raise: more smoothing steps, softer relief. Lower: fewer steps. Hillslope diffuse only."),
    ("passes.sediment_fill", "On: eroded sediment is carried downstream and laid down as deltas, shelves and floodplains. Off: skipped."),
    ("passes.sediment_capacity", "Raise: rivers carry more sediment before dropping it. Lower: they drop it sooner. Sediment fill only."),
    ("passes.evolve_cycles", "Raise: more coupled climate and erosion cycles, so rain shapes valleys more. Lower: fewer; 0 is off."),
    ("passes.tidal_flats", "On: shallow sea inside the tidal range builds up into mudflats. Off: skipped."),
    ("passes.tidal_k", "Raise: faster mudflat build-up. Lower: slower build-up. Tidal flats only."),
    // ---- climate
    ("climate.lat_n", "Raise: the map's top edge lies further north. Lower: further south. Ignored for a whole world."),
    ("climate.lat_s", "Raise: the map's bottom edge lies further north. Lower: further south. Ignored for a whole world."),
    ("climate.equator_temp", "Raise: hotter tropics, a warmer world. Lower: colder tropics, a colder world."),
    ("climate.pole_temp", "Raise: milder poles, a warmer world. Lower: colder poles, a colder world with more ice."),
    ("climate.lapse_rate", "Raise: colder peaks and a lower snowline, a colder world at height. Lower: milder high ground."),
    ("climate.albedo_k", "Raise: snow and ice cool their surroundings, so icy regions grow colder and wider. Lower: weaker; 0 is off."),
    ("climate.currents", "On: warm and cold ocean currents shift coastal temperature and rain. Off: no current effect."),
    ("climate.current_k", "Raise: stronger ocean-current warming and cooling of coasts. Lower: weaker. Ocean currents only."),
    ("climate.terrain_wind_deflection", "On: winds bend around mountains and coasts. Off: winds ignore terrain (for comparison only)."),
    // ---- weather (rainfall)
    ("climate.w_iters", "Raise: moisture travels deeper inland, wetter interiors. Lower: wetness stays near coasts, drier interiors."),
    ("climate.rain_k", "Raise: wetter windward slopes and starker rain shadows. Lower: less rain from air rising over mountains."),
    ("climate.evap", "Raise: more moisture picked up over the sea, a wetter world. Lower: less moisture, a drier world."),
    ("climate.rain_dep", "Raise: air rains out sooner; land past the first ridge drier, wet/dry edges sharper. Lower: rain reaches further inland."),
    ("climate.ocean", "Raise: more moisture from the sea, a wetter world (the global rain budget). Lower: less moisture, a drier world."),
    ("climate.wind_manual", "On: one fixed prevailing wind direction, Region maps only. Off: planetary wind belts by latitude."),
    ("climate.wind_dir_deg", "Raise: turns the manual wind direction clockwise, in degrees. Lower: anticlockwise. Manual wind on a Region map only."),
    ("climate.press_k", "Raise: monsoon-like winds bent toward hot land. Lower: plain latitude wind belts; 0 ignores pressure."),
    ("climate.zonal_k", "Raise: a wetter equator and drier subtropical belts. Lower: weaker latitude rain bands."),
    ("climate.ocean_hum", "Raise: moister air over the sea, a wetter world. Lower: drier sea air, a drier world."),
    ("climate.bulk_evap", "On: evaporation follows wind speed and air dryness. Off: one flat evaporation rate everywhere."),
    ("climate.rain_bspline", "On: smooth rain with no square crease pattern (current). Off: the older bilinear rain with faint grid creases."),
    // ---- civ
    ("civ.villages", "On: an extra pass seeds villages between settlements. Off: no seeded villages."),
    ("civ.metropolis", "On: the most central cities become imperial seats. Off: no metropolis tier."),
    ("civ.recovery_phase", "Raise: a later stage of recovery after a collapse (1 survival to 4 mature). Lower: an earlier stage; 0 is a stable world."),
    ("civ.biome_k", "On: biome carrying capacity adjusts where people settle. Off: it does not."),
    ("civ.factions", "Raise: more factions share the land. Lower: fewer factions; at least 1."),
    ("civ.seed_thresh", "Raise: only the best sites are settled, fewer settlements. Lower: more settlements. Ignored with fixed counts on."),
    ("civ.seed_suppress_div", "Raise: settlements pack closer, more of them. Lower: they spread further apart. Ignored with fixed counts on."),
    ("civ.fixed_counts", "On: the five settlement counts set exact numbers. Off: settlements are placed by site suitability."),
    ("civ.n_capital", "Raise: more capitals. Lower: fewer capitals. Read only with fixed settlement counts on."),
    ("civ.n_city", "Raise: more cities. Lower: fewer cities. Read only with fixed settlement counts on."),
    ("civ.n_town", "Raise: more towns. Lower: fewer towns. Read only with fixed settlement counts on."),
    ("civ.n_village", "Raise: more villages. Lower: fewer villages. Read only with fixed settlement counts on."),
    ("civ.n_hamlet", "Raise: more hamlets. Lower: fewer hamlets. Read only with fixed settlement counts on."),
    ("civ.garrison_exposure_scale", "Raise: garrisons grow more on exposed borders. Lower: border exposure matters less; 0 ignores it."),
];

/// The description of `key`, or `None` (a test names every offered key without one).
pub fn desc(key: &str) -> Option<&'static str> {
    DESCS.iter().find(|(k, _)| *k == key).map(|(_, d)| *d)
}

/// The blurb of `group`, or `None`.
pub fn blurb(group: &str) -> Option<&'static str> {
    BLURBS.iter().find(|(g, _)| *g == group).map(|(_, b)| *b)
}

fn to_pvalue(v: params::Value) -> PValue {
    match v {
        params::Value::Bool(b) => PValue::Bool(b),
        params::Value::Num(n) => PValue::Num(n),
    }
}

/// What showing a change to `key` needs, from `params::invalidates` (the
/// staleness map the Recompute button already runs on). The one key whose
/// `None` means "read fresh" rather than "needs a Generate" is named by that
/// function's own carve-out comment.
pub fn regen(key: &str) -> Regen {
    match params::invalidates(key) {
        Some(PipelineStage::Hydrology) => Regen::RefreshClimate,
        Some(PipelineStage::Climate) => Regen::RefreshSettlements,
        Some(_) => Regen::FullGenerate,
        None if key == "civ.garrison_exposure_scale" => Regen::Live,
        None => Regen::FullGenerate,
    }
}

/// A ruled-divergence switch (scope s.5b.1): a bool whose shipped app default
/// (`params::defaults()`) differs from the parity baseline
/// (`WorldParams::defaults`). DERIVED, not listed, so a new ruled flag is tagged
/// without an edit here.
pub fn is_ruled_divergence(key: &str) -> bool {
    let app = params::defaults();
    let parity = WorldParams::defaults(0, 0, 0);
    matches!(
        (params::get(&app, key), params::get(&parity, key)),
        (Some(params::Value::Bool(a)), Some(params::Value::Bool(b))) if a != b
    )
}

/// The catalogue for one request. `current(key)` is the world's current value
/// (the shell's `get_params()` snapshot); a key it does not answer is sent with
/// no `current` (omitted, never 0), and its `INERT_WHEN` gate is not evaluated.
pub fn build(current: &dyn Fn(&str) -> Option<PValue>) -> Catalogue {
    let defaults = params::defaults();
    let rows = params::PARAMS
        .iter()
        .filter(|s| !EXCLUDED.contains(&s.key))
        .map(|s| {
            let inert = INERT_WHEN.iter().find(|(k, _, _, _)| *k == s.key).and_then(|(_, gate, needed, why)| {
                match current(gate) {
                    Some(PValue::Bool(b)) if b != *needed => Some(format!("Will have no effect while {why}.")),
                    _ => None,
                }
            });
            Row {
                key: s.key.to_owned(),
                group: s.group.to_owned(),
                kind: match s.kind {
                    params::Kind::Bool => Kind::Bool,
                    params::Kind::Int => Kind::Int,
                    params::Kind::Float => Kind::Float,
                },
                min: s.min,
                max: s.max,
                step: s.step,
                label: s.label.to_owned(),
                unit: s.unit.to_owned(),
                desc: desc(s.key).unwrap_or("").to_owned(),
                default: params::get(&defaults, s.key).map(to_pvalue),
                current: current(s.key),
                advanced: is_ruled_divergence(s.key),
                inert,
                regen: regen(s.key),
            }
        })
        .collect();
    let groups = params::groups()
        .into_iter()
        .map(|g| Group { name: g.to_owned(), blurb: blurb(g).unwrap_or("").to_owned() })
        .collect();
    Catalogue { rows, excluded: EXCLUDED.iter().map(|k| (*k).to_owned()).collect(), groups }
}
