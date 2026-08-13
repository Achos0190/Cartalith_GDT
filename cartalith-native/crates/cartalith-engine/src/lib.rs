//! orchestrator: owns WorldState, runs the pipeline stages in order
//!
//! `generate_terrain()` (reference HTML `generate()`, lines 3339-3391, its
//! `buildTectonicSubstrate` prefix at lines 3396-3462, and — when
//! `carve_rivers` is on, the JS default — `carveRiverValleys()` at lines
//! 8761-8789) — the sync, no-worker-pool path specifically, since this port
//! has no browser worker pool (`ARCHITECTURE.md`, threading: Rust's
//! equivalent is `rayon`, not ported yet for this stage). Runs every
//! already-ported subsystem in the JS engine's own order, from a seed all
//! the way through carved river valleys — the same point a fresh default
//! `generate()` call leaves `field`/`tempField`/`rainField`/`flowField` at.
//!
//! ## World-Structure archetypes — ported, with one real deviation
//!
//! `state.world_structure.enabled` (default `false`, so this whole section
//! is a no-op path at the JS engine's own defaults) now runs
//! `generate_continentality_field`/`apply_world_structure_sea_level`
//! (`cartalith-terrain`) and derives `tect.plates`/`tect.vel`/`volc.count`
//! from the archetype's own params (`deriveFromWorldStructure()`, reference
//! HTML lines 2528-2538) exactly as JS does — **except** graph-driven
//! orogeny. JS's `deriveFromWorldStructure()` always sets
//! `state.tect.tectonicGraph=true` alongside the plates/vel/volc.count
//! derivation; this port has not ported `buildOrogenyField` (T2+T3 —
//! boundary-polyline-graph-driven fold/trench/fault-block landforms), so
//! `oro` stays `None` here even when `world_structure.enabled` is `true`.
//! **This is a real, deliberate divergence from JS at that one setting**,
//! not a no-op-at-defaults case like the rest of this list — a
//! World-Structure world generated here will have the right continentality
//! shape and land fraction (both real, verified, load-bearing effects) but
//! the older "blob" convergent-stress uplift instead of JS's structured
//! per-margin orogeny. Flagged here rather than silently approximated;
//! `foldIntensity`/`trenchDepth` (JS's own orogeny-only tuning knobs) are
//! correspondingly not modeled at all.
//!
//! ## What else this deliberately does NOT reproduce, and why
//! - **`stampVolcanoesProvinces`** (`state.volc.provinces`, JS default
//!   `true`): ported (`cartalith_terrain::stamp_volcanoes_provinces`) and
//!   reachable via `p.volc.provinces`, but this port's own default here
//!   is `false` — see `WorldParams::defaults`'s doc comment on why
//!   (no JS runtime in this environment to extract golden fixtures
//!   against a placement algorithm this RNG-order-sensitive).
//! - **Ocean-current SST folding** (`state.climate.currents`, default `true`):
//!   `MVP_SCOPE.md` explicitly names ocean-current terrain coupling a stretch
//!   goal and grants permission to defer it if documented — taken here despite
//!   the JS default being *on*, consistent with `simulate_weather`'s own
//!   already-documented deferral of the same mechanism.
//! - **Dynamic lithology** (`state.tect.dynamicLithology`, default `false`):
//!   ported and wired in (`recompute_resistance_after_erosion`, gated on
//!   `p.tect.dynamic_lithology` exactly as JS gates it on the flag of the
//!   same name in `eroFinish`) — off at the default, so this pipeline is
//!   bit-identical to before unless a caller opts in.
//! - **`enforceRiverChannels()`**: a no-op on any *fresh* `generate()` —
//!   `riverMask` only ever gets cells locked by a PRIOR `carveRiverValleys`
//!   call (or manual river brushing, which this port doesn't have), and
//!   both start empty on a fresh world. `generate_terrain` always runs
//!   fresh, so this call is always a no-op here and is omitted rather than
//!   ported as dead code.
//! - **River-network render/export helpers** (`splitRiverPolylines`,
//!   `riverSinuAmp`/`riverSinuosity`, `buildFeatureRegistry`,
//!   `buildRiverNetwork`'s own width/intensity/depth "cells" stamping
//!   loop): all render- or export-time concerns, not part of the
//!   generate()/carve pipeline — `carveRiverValleys` computes its own,
//!   simpler per-polyline half-width directly rather than reusing that
//!   loop's per-cell output (reference HTML's own comment on
//!   `splitRiverPolylines`: "Applied at the render/export sites ONLY;
//!   traceRiverPolylines itself is untouched so the generate()/carve
//!   pipeline stays bit-identical").

use cartalith_climate::{apply_climate_moisture_correctors, compute_temperature, simulate_weather, ClimateParams, WeatherParams};
use cartalith_erosion::{isostatic_rebound, recompute_resistance_after_erosion, stream_power_kernel, StreamPowerParams};
use cartalith_hydrology::{
    build_channels, compute_flow, enforce_channel_descent, river_width_scale_k, strahler_from_receivers,
    trace_river_polylines, ChannelResult,
};
use cartalith_terrain::{
    apply_world_structure_sea_level, assign_plates, build_age_field, build_plates, compute_flexure, compute_height,
    compute_heterogeneity, compute_resistance, compute_stress, compute_warp, gauss_blur,
    generate_continentality_field, normalize_field, stamp_craters, stamp_volcanoes_provinces, stamp_volcanoes_simple,
    HeightParams, WorldStructure,
};

/// Mirrors JS `Math.round` (ties toward `+Infinity`) — same trap
/// `cartalith-terrain::js_round`/`cartalith-climate::js_round` exist for;
/// duplicated here rather than adding a dependency purely for one line,
/// matching those crates' own precedent.
fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// `state.tect` (reference HTML line 2264-2265) — the formula's real tuning
/// knobs, plus `resist` (`streamParams()`'s erodibility-resistance weight,
/// now read by `carveRiverValleys`'s light stream-power pass) and
/// `dynamic_lithology` (`eroFinish`'s L4 exhumation-hardening gate — see
/// `recompute_resistance_after_erosion`'s call site below). The remaining
/// World-Structure-gated fields (`tectonicGraph`/`foldIntensity`/
/// `trenchDepth`/`faultBlock`) stay omitted — WS stays off in this pipeline
/// (see the module doc comment), so nothing here reads them.
pub struct TectonicParams {
    pub seed: i32,
    pub plates: usize,
    pub vel: f64,
    pub warp: f64,
    pub blur_r: f64,
    pub alpha: f64,
    pub beta: f64,
    pub age_inf: f64,
    pub ridged: bool,
    pub lloyd: usize,
    pub flexure: f64,
    pub hetero: f64,
    pub resist: f64,
    pub dynamic_lithology: bool,
}

/// `state.volc` (reference HTML line 2266). `provinces` selects
/// `stamp_volcanoes_provinces` (JS default, `true`) vs. `stamp_volcanoes_simple`
/// (`false`) — see `generate_terrain`'s own volcanism section and
/// `WorldParams::defaults`'s doc comment on why `false` is this port's own
/// default for now, not JS's.
pub struct VolcanismParams {
    pub count: i32,
    pub age: f64,
    pub provinces: bool,
}

/// `state.crater` (reference HTML line 2267).
pub struct CraterParams {
    pub count: i32,
    pub age: f64,
}

/// `state.planet` (reference HTML lines 2277-2279), minus `radiusRel`
/// (only read by `circulationCells`'s `radius_rel` argument, which
/// `simulate_weather` already accepts a fixed default for via its own
/// `WeatherParams` — not re-exposed here since nothing in this pipeline
/// varies it yet) and the geoid/tides sub-objects (both default `enabled:
/// false`, matching `compute_temperature`'s/`simulate_weather`'s own
/// `None`-geoid reasoning).
pub struct PlanetParams {
    pub g: f64,
    pub rotation_hours: f64,
    pub axial_tilt_deg: f64,
}

/// `state.climate` (reference HTML line 2280) fields this pipeline's
/// temperature/weather/moisture-corrector stages actually read.
pub struct ClimateInputParams {
    pub lat_n: f64,
    pub lat_s: f64,
    pub equator_temp: f64,
    pub pole_temp: f64,
    pub lapse_rate: f64,
    pub albedo_k: f64,
    pub zonal_k: f64,
    pub wind_manual: bool,
    pub wind_dir_deg: f64,
    pub press_k: f64,
    pub ocean_hum: f64,
    pub evap: f64,
    pub ocean: f64,
    pub rain_k: f64,
    pub rain_dep: f64,
    pub bulk_evap: bool,
    pub w_iters: i32,
}

/// `state.stream` (reference HTML line 2269) fields `carveRiverValleys`'s
/// light stream-power pass reads via `streamParams()`. `cycles` is omitted
/// — only read by `evolveCoupled()`, the manual "Stream evolve" tool, not
/// `carveRiverValleys`.
pub struct StreamParams {
    pub uplift: f64,
    pub k: f64,
    pub iters: i32,
    pub deposit: f64,
    pub climate_k: f64,
}

/// `state.world_structure` (reference HTML line 2263) — the five
/// archetype knobs `ARCHETYPES`'s presets (earth/supercontinent/
/// archipelago/volcanic/rift, reference HTML lines 2521-2526) set
/// together. This port takes the five raw values directly rather than
/// modeling named archetypes — a caller wanting "Archipelago" passes
/// `ARCHETYPES.archipelago`'s own numbers. See the module doc comment
/// for the one real deviation (`tectonicGraph`/graph-driven orogeny) this
/// enables that this port doesn't reproduce.
pub struct WorldStructureParams {
    pub enabled: bool,
    pub continentality: f64,
    pub fragmentation: f64,
    pub tectonic_energy: f64,
    pub ocean_depth: f64,
    pub hotspot_density: f64,
}

/// Everything `generate_terrain` needs from `state` — one struct per
/// `state` sub-object (`tect`/`volc`/`crater`/`planet`/`climate`/`stream`/
/// `world_structure`), plus the handful of top-level fields (`world`/
/// `seaLevel`/`peakM`/`mapWidthKm`/`carveRivers`) every stage reads
/// directly. `river_density` is `state.viz.riverDensity` — grouped at the
/// top level since `viz` is otherwise a render-only settings bag this
/// crate has no other reason to model.
pub struct WorldParams {
    pub gw: usize,
    pub gh: usize,
    pub world: bool,
    pub sea_level: f64,
    pub peak_m: f64,
    pub map_width_km: f64,
    pub carve_rivers: bool,
    pub river_density: f64,
    pub tect: TectonicParams,
    pub volc: VolcanismParams,
    pub crater: CraterParams,
    pub planet: PlanetParams,
    pub climate: ClimateInputParams,
    pub stream: StreamParams,
    pub world_structure: WorldStructureParams,
}

impl WorldParams {
    /// `state`'s own literal defaults (reference HTML lines 2258-2310) at
    /// the given grid size and seed — `state.tect.seed` is normally
    /// `(Math.random()*99999)|0` in JS; the caller supplies it here since
    /// this port has no equivalent of reading real time/entropy inside a
    /// deterministic-by-construction crate.
    pub fn defaults(gw: usize, gh: usize, seed: i32) -> Self {
        WorldParams {
            gw,
            gh,
            world: false,
            sea_level: 0.42,
            peak_m: 4000.0,
            map_width_km: 800.0,
            carve_rivers: true,
            river_density: 1.0,
            tect: TectonicParams {
                seed,
                plates: 14,
                vel: 1.0,
                warp: 0.45,
                blur_r: 18.0,
                alpha: 0.85,
                beta: 0.22,
                age_inf: 0.6,
                ridged: true,
                lloyd: 2,
                flexure: 0.20,
                hetero: 0.08,
                resist: 0.50,
                dynamic_lithology: false,
            },
            // `provinces: false`, not JS's own literal default (`true`):
            // stamp_volcanoes_provinces is ported (cartalith-terrain) and
            // reachable via this flag, but this environment has no JS
            // runtime to extract real golden fixtures against it
            // (`PARITY_TESTING.md`'s own extraction procedure needs one),
            // so it isn't golden-verified yet. A hand-derived unit test
            // isn't a substitute here the way it was for
            // recompute_resistance_after_erosion's pure per-cell formula
            // -- this is a multi-branch, RNG-order-sensitive placement
            // algorithm across a whole grid, exactly the kind of thing
            // `cartalith-porting-discipline` says "looks reasonable" isn't
            // sufficient for. `false` keeps this pipeline's default
            // output identical to before this port (and to
            // golden_parity_carve.rs's existing verified fixtures) until
            // someone with a JS runtime extracts real fixtures and flips
            // this default to match JS.
            volc: VolcanismParams { count: 20, age: 0.40, provinces: false },
            crater: CraterParams { count: 100, age: 0.50 },
            planet: PlanetParams { g: 1.0, rotation_hours: 24.0, axial_tilt_deg: 23.4 },
            climate: ClimateInputParams {
                lat_n: 55.0,
                lat_s: 5.0,
                equator_temp: 30.0,
                pole_temp: -25.0,
                lapse_rate: 6.5,
                albedo_k: 0.0,
                zonal_k: 0.5,
                wind_manual: false,
                wind_dir_deg: 0.0,
                press_k: 0.6,
                ocean_hum: 1.0,
                evap: 0.12,
                ocean: 1.0,
                rain_k: 1.0,
                rain_dep: 0.35,
                bulk_evap: true,
                w_iters: 70,
            },
            stream: StreamParams { uplift: 0.0, k: 0.012, iters: 15, deposit: 0.3, climate_k: 0.5 },
            world_structure: WorldStructureParams {
                enabled: false,
                continentality: 0.30,
                fragmentation: 0.50,
                tectonic_energy: 0.60,
                ocean_depth: 0.60,
                hotspot_density: 0.20,
            },
        }
    }
}

/// Everything `generate_terrain` produces — the Rust equivalent of the JS
/// module globals `field`/`plateId`/`boundaryMask`/.../`flowField`/
/// `tempField`/`rainField`/`riverMask`/`riverFloor` a fresh `generate()`
/// call leaves behind. `channels`/`stream_order`/`river_mask`/
/// `river_floor` are `None` when `carve_rivers` is off — matching JS,
/// where `buildRiverNetwork` (and therefore any channel topology at all)
/// is never called anywhere in a default sync `generate()` except from
/// inside `carveRiverValleys`. `field`/`temperature`/`rainfall`/
/// `flow_discharge` reflect the state right after `carveRiverValleys`
/// when it ran, or right after the pre-carve `computeFlow(true)`/
/// `refreshClimate()` when it didn't — either way, the same fields
/// `generate()` itself leaves as current.
pub struct WorldState {
    /// The sea level actually used for this generation — equal to
    /// `p.sea_level` unless `world_structure.enabled` re-anchored it
    /// (`apply_world_structure_sea_level`). Callers that classify land vs.
    /// ocean (a renderer, a land-fraction check) must use this, not
    /// `p.sea_level` directly.
    pub sea_level: f64,
    pub field: Vec<f32>,
    pub plate_id: Vec<usize>,
    pub boundary_mask: Vec<u8>,
    pub stress_field: Vec<f32>,
    pub flexure_field: Vec<f32>,
    pub age_field: Vec<f32>,
    pub heterogeneity_field: Vec<f32>,
    pub resistance_field: Vec<f32>,
    pub volcanic_field: Vec<f32>,
    pub impact_field: Vec<f32>,
    pub temperature: Vec<f32>,
    pub rainfall: Vec<f32>,
    pub flow_area: Vec<f32>,
    pub flow_discharge: Vec<f32>,
    pub channels: Option<ChannelResult>,
    pub stream_order: Option<Vec<i16>>,
    pub river_mask: Option<Vec<u8>>,
    pub river_floor: Option<Vec<f32>>,
}

/// Runs the full ported pipeline once, from a seed to (when
/// `p.carve_rivers`, the JS default) carved river valleys. See the module
/// doc comment for the exact JS functions this mirrors and what's
/// deliberately not reproduced yet.
pub fn generate_terrain(p: &WorldParams) -> WorldState {
    let gw = p.gw;
    let gh = p.gh;
    let world = p.world;

    // `deriveFromWorldStructure()` (reference HTML lines 2528-2538): once a
    // World-Structure archetype is active, plates/velocity/volcano count
    // are ALWAYS the archetype-derived values, not independently
    // configurable -- these three overrides replace `p.tect.plates`/
    // `p.tect.vel`/`p.volc.count` wherever WS is enabled, everywhere below.
    let (tect_plates, tect_vel, volc_count) = if p.world_structure.enabled {
        let ws = &p.world_structure;
        let plates = (js_round(4.0 + ws.fragmentation * 24.0) as usize).clamp(4, 40);
        let vel = ws.tectonic_energy * 2.0;
        let volc_count = js_round(ws.hotspot_density * 60.0) as i32;
        (plates, vel, volc_count)
    } else {
        (p.tect.plates, p.tect.vel, p.volc.count)
    };

    // ---- buildTectonicSubstrate (reference HTML lines 3396-3462) ----
    // generateContinentalityField(): a no-op (`None`) at World-Structure's
    // default `enabled:false` -- bit-identical to omitting it entirely.
    let continental_field = if p.world_structure.enabled {
        Some(generate_continentality_field(
            gw,
            gh,
            world,
            p.tect.seed,
            p.world_structure.continentality,
            p.world_structure.fragmentation,
        ))
    } else {
        None
    };
    let world_structure_arg = continental_field.as_ref().map(|cf| WorldStructure {
        ocean_depth: p.world_structure.ocean_depth,
        continental_field: cf.as_slice(),
    });

    let warp = compute_warp(gw, gh, p.tect.seed, p.tect.warp, world);
    let (warp_x, warp_y) = match &warp {
        Some((wx, wy)) => (Some(wx.as_slice()), Some(wy.as_slice())),
        None => (None, None),
    };

    let plates = build_plates(gw, gh, p.tect.seed as u32, tect_plates, p.tect.lloyd, world, world_structure_arg);
    let plate_id = assign_plates(gw, gh, world, &plates, warp_x, warp_y);
    let stress = compute_stress(gw, gh, world, &plate_id, &plates, tect_vel, p.tect.blur_r);
    let flexure_field = compute_flexure(gw, gh, &stress.boundary_mask, &stress.stress_field, p.tect.blur_r, world);

    let base_raw: Vec<f32> = plate_id.iter().map(|&pid| plates[pid].base as f32).collect();
    let base_field = gauss_blur(&base_raw, (p.tect.blur_r * 0.35).max(2.0), gw, gh, world);

    let age_field = build_age_field(gw, gh, &stress.boundary_mask);

    let heterogeneity_field =
        compute_heterogeneity(gw, gh, p.tect.seed, p.map_width_km, world, &age_field, warp_x, warp_y);
    let mut resistance_field = compute_resistance(gw, gh, &plate_id, &plates, &age_field);
    // orogenyField: always None here -- see the module doc comment
    // ("Graph-driven orogeny").

    // ---- height -> normalize (reference HTML lines 3361-3363) ----
    let height_params = HeightParams {
        nf: 5.0 * cartalith_terrain::terrain_detail_k(gw, p.map_width_km),
        seed: p.tect.seed,
        a: p.tect.alpha,
        b: p.tect.beta,
        age_inf: p.tect.age_inf,
        fwt: p.tect.flexure,
        hwt: p.tect.hetero,
        world,
        ridged: p.tect.ridged,
    };
    let raw_height = compute_height(
        gw,
        gh,
        &base_field,
        &stress.stress_field,
        &flexure_field,
        &heterogeneity_field,
        &age_field,
        warp_x,
        warp_y,
        None,
        &height_params,
    );
    let mut field = normalize_field(&raw_height);

    // ---- volcanism + craters (reference HTML lines 3365-3369) ----
    let mut volcanic_field = vec![0f32; gw * gh];
    let mut impact_field = vec![0f32; gw * gh];
    if volc_count > 0 {
        // stampVolcanoes() (reference HTML lines 3474-3478): dispatches on
        // state.volc.provinces, JS default true.
        if p.volc.provinces {
            stamp_volcanoes_provinces(
                gw,
                gh,
                p.tect.seed as u32,
                p.map_width_km,
                p.peak_m,
                &stress.boundary_mask,
                &stress.stress_field,
                &plate_id,
                &plates,
                volc_count,
                p.volc.age,
                &mut field,
                &mut volcanic_field,
            );
        } else {
            stamp_volcanoes_simple(
                gw,
                gh,
                p.tect.seed as u32,
                p.map_width_km,
                p.peak_m,
                &stress.boundary_mask,
                volc_count,
                p.volc.age,
                &mut field,
                &mut volcanic_field,
            );
        }
    }
    stamp_craters(
        gw,
        gh,
        p.tect.seed as u32,
        p.map_width_km,
        p.planet.g,
        p.crater.count,
        p.crater.age,
        &mut field,
        &mut impact_field,
    );
    for v in &mut field {
        *v = v.clamp(0.0, 1.0);
    }

    // applyWorldStructureSeaLevel() (reference HTML lines 2603-2617): a
    // no-op at World-Structure's default `enabled:false` -- `sea_level`
    // stays `p.sea_level` unchanged. When enabled, re-anchors sea level
    // against the ACTUAL generated field's histogram so the archetype's
    // promised land fraction holds regardless of how tectonicEnergy/
    // oceanDepth reshaped the height distribution -- everything from here
    // down reads `sea_level`, not `p.sea_level`.
    let sea_level = if p.world_structure.enabled {
        apply_world_structure_sea_level(&field, p.world_structure.continentality)
    } else {
        p.sea_level
    };

    // ---- natural order: structural drainage -> climate -> discharge-
    // weighted drainage (reference HTML lines 3382-3386) ----
    let flow_area = compute_flow(gw, gh, &field, None, false, world);

    let climate_params = ClimateParams {
        world,
        lat_n: p.climate.lat_n,
        lat_s: p.climate.lat_s,
        pole_temp: p.climate.pole_temp,
        equator_temp: p.climate.equator_temp,
        tilt_deg: p.planet.axial_tilt_deg,
        rotation_hours: p.planet.rotation_hours,
        lapse_rate: p.climate.lapse_rate,
        g: p.planet.g,
        sea_level,
        peak_m: p.peak_m,
        albedo_k: p.climate.albedo_k,
    };
    let mut temperature = compute_temperature(gw, gh, &field, None, &climate_params);

    let weather_params = WeatherParams {
        world,
        lat_n: p.climate.lat_n,
        lat_s: p.climate.lat_s,
        pole_temp: p.climate.pole_temp,
        equator_temp: p.climate.equator_temp,
        tilt_deg: p.planet.axial_tilt_deg,
        rotation_hours: p.planet.rotation_hours,
        lapse_rate: p.climate.lapse_rate,
        sea_level,
        peak_m: p.peak_m,
        wind_manual: p.climate.wind_manual,
        wind_dir_deg: p.climate.wind_dir_deg,
        press_k: p.climate.press_k,
        ocean_hum: p.climate.ocean_hum,
        evap: p.climate.evap,
        ocean: p.climate.ocean,
        rain_k: p.climate.rain_k,
        rain_dep: p.climate.rain_dep,
        bulk_evap: p.climate.bulk_evap,
    };
    // decl=0: refreshClimate()'s own simulateWeather(state.climate.wIters)
    // call passes no declination argument, which defaults to 0 (annual
    // mean, no seasonal tilt) -- reference HTML line 5154.
    let mut rainfall = simulate_weather(gw, gh, &field, p.climate.w_iters, 0.0, &weather_params);

    // applyClimateMoistureCorrectors() -- unconditional, see
    // cartalith-climate's own doc comment on this function.
    apply_climate_moisture_correctors(
        gw,
        gh,
        &field,
        &flow_area,
        &mut rainfall,
        sea_level,
        world,
        p.climate.lat_n,
        p.climate.lat_s,
        p.climate.zonal_k,
    );
    // applyOceanCurrents()/computeSeasons(): deferred -- see the module
    // doc comment ("Ocean-current SST folding").

    let mut flow_discharge = compute_flow(gw, gh, &field, Some(&rainfall), true, world);

    let mut channels = None;
    let mut stream_order = None;
    let mut river_mask = None;
    let mut river_floor = None;

    if p.carve_rivers {
        // ---- carveRiverValleys (reference HTML lines 8761-8789) ----
        // (1) light physical erosion pass -- natural, discharge-weighted
        // valley networks. `rainfall` here is still the PRE-carve field
        // JS computed above; refreshClimate() doesn't run again until
        // step (3), exactly matching the reference's own read order.
        let pre = field.clone();
        let light_iters = (js_round(p.stream.iters as f64 * 0.6) as i32).max(4);
        let stream_params = StreamPowerParams {
            k: p.stream.k,
            uplift: p.stream.uplift,
            deposit: p.stream.deposit,
            climate_k: p.stream.climate_k,
            iters: light_iters,
            resist: p.tect.resist,
            g: p.planet.g,
            world,
            sea: sea_level,
        };
        stream_power_kernel(&mut field, &stress.stress_field, &resistance_field, &rainfall, gw, gh, &stream_params);
        isostatic_rebound(&mut field, &pre, gw, gh, p.tect.blur_r, world);
        if p.tect.dynamic_lithology {
            // recomputeResistanceAfterErosion(reference HTML line 3144):
            // JS's own call site (`eroFinish`) passes no `opts`, so `k`
            // uses the function's built-in default of 6.0.
            recompute_resistance_after_erosion(&mut resistance_field, &pre, &field, 6.0);
        }
        // enforceRiverChannels(): always a no-op here -- see the module
        // doc comment.
        let flow_for_network = compute_flow(gw, gh, &field, Some(&rainfall), true, world);

        // (2) vector network -> distance-field channel carve + lock
        let ch = build_channels(&field, &flow_for_network, gw, gh, sea_level, world, p.river_density, p.map_width_km);
        let order = strahler_from_receivers(&ch.recv, &flow_for_network, &ch.chan);
        let polys = trace_river_polylines(&order, &ch.recv, gw, gh, 1);

        let width_k = river_width_scale_k(p.map_width_km);
        let half_w_cap = 4.0 * width_k;
        let mut rmask = vec![0u8; gw * gh];
        let mut rfloor = vec![0f32; gw * gh];
        for poly in &polys {
            let &(lx, ly) = poly.last().expect("trace_river_polylines only returns polylines with >=2 points");
            let li = ((ly as i64) * gw as i64 + lx as i64).clamp(0, (gw * gh) as i64 - 1) as usize;
            let o_raw = order[li];
            let o = if o_raw != 0 { o_raw as f64 } else { 1.0 };
            let mut half_w = (0.8 + 0.5 * (o - 1.0)) * width_k;
            if half_w > half_w_cap {
                half_w = half_w_cap;
            }
            let carved = enforce_channel_descent(&mut field, gw, gh, poly, sea_level, half_w, 0.0006);
            for i in carved {
                rmask[i] = 1;
                rfloor[i] = field[i];
            }
        }

        // (3) recompute so overlay + rainfall reflect the carved valleys
        flow_discharge = compute_flow(gw, gh, &field, Some(&rainfall), true, world);
        temperature = compute_temperature(gw, gh, &field, None, &climate_params);
        rainfall = simulate_weather(gw, gh, &field, p.climate.w_iters, 0.0, &weather_params);
        apply_climate_moisture_correctors(
            gw,
            gh,
            &field,
            &flow_discharge,
            &mut rainfall,
            sea_level,
            world,
            p.climate.lat_n,
            p.climate.lat_s,
            p.climate.zonal_k,
        );

        channels = Some(ch);
        stream_order = Some(order);
        river_mask = Some(rmask);
        river_floor = Some(rfloor);
    }

    WorldState {
        sea_level,
        field,
        plate_id,
        boundary_mask: stress.boundary_mask,
        stress_field: stress.stress_field,
        flexure_field,
        age_field,
        heterogeneity_field,
        resistance_field,
        volcanic_field,
        impact_field,
        temperature,
        rainfall,
        flow_area,
        flow_discharge,
        channels,
        stream_order,
        river_mask,
        river_floor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_terrain_runs_end_to_end() {
        let p = WorldParams::defaults(24, 18, 12345);
        let ws = generate_terrain(&p);
        let n = 24 * 18;
        assert_eq!(ws.field.len(), n);
        assert_eq!(ws.temperature.len(), n);
        assert_eq!(ws.rainfall.len(), n);
        assert_eq!(ws.flow_discharge.len(), n);
        assert!(ws.field.iter().all(|&v| (0.0..=1.0).contains(&v)));
        assert!(ws.rainfall.iter().all(|&v| (0.0..=1.0).contains(&v)));
        // carve_rivers defaults true -- channel topology should be present.
        assert!(ws.channels.is_some());
        assert!(ws.stream_order.is_some());
    }

    #[test]
    fn generate_terrain_without_carve_matches_pre_carve_shape() {
        let mut p = WorldParams::defaults(20, 14, 555);
        p.carve_rivers = false;
        let ws = generate_terrain(&p);
        assert!(ws.channels.is_none());
        assert!(ws.stream_order.is_none());
        assert!(ws.river_mask.is_none());
        assert!(ws.river_floor.is_none());
    }

    /// World-Structure is verified numerically against real JS in
    /// `cartalith-terrain`'s own golden tests (`generate_continentality_field`/
    /// `apply_world_structure_sea_level`); this only checks the *wiring* --
    /// that an enabled archetype actually reaches a different, still-valid
    /// `WorldState`, and that a low-continentality archetype (Archipelago)
    /// produces less land than a high-continentality one (Supercontinent),
    /// which is the whole reason `applyWorldStructureSeaLevel` exists
    /// (`cartalith-terrain`'s own doc comment: the v1.25 bug it fixed).
    #[test]
    fn generate_terrain_world_structure_shapes_land_fraction() {
        let land_fraction = |p: &WorldParams| {
            let ws = generate_terrain(p);
            let land = ws.field.iter().filter(|&&h| (h as f64) >= ws.sea_level).count();
            land as f64 / ws.field.len() as f64
        };

        let mut archipelago = WorldParams::defaults(20, 16, 7);
        archipelago.world_structure = WorldStructureParams {
            enabled: true,
            continentality: 0.15,
            fragmentation: 0.90,
            tectonic_energy: 0.80,
            ocean_depth: 0.30,
            hotspot_density: 0.50,
        };

        let mut supercontinent = WorldParams::defaults(20, 16, 7);
        supercontinent.world_structure = WorldStructureParams {
            enabled: true,
            continentality: 0.60,
            fragmentation: 0.10,
            tectonic_energy: 0.50,
            ocean_depth: 0.70,
            hotspot_density: 0.10,
        };

        let archipelago_land = land_fraction(&archipelago);
        let supercontinent_land = land_fraction(&supercontinent);
        assert!(
            archipelago_land < supercontinent_land,
            "archipelago land {archipelago_land} should be less than supercontinent land {supercontinent_land}"
        );
    }
}
