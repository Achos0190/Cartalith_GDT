//! orchestrator: owns WorldState, runs the pipeline stages in order
//!
//! `generate_terrain()` (reference HTML `generate()`, lines 3339-3391, and its
//! `buildTectonicSubstrate` prefix, lines 3396-3462) — the sync, no-worker-pool
//! path specifically, since this port has no browser worker pool
//! (`ARCHITECTURE.md`, threading: Rust's equivalent is `rayon`, not ported yet
//! for this stage). Runs every already-ported subsystem in the JS engine's own
//! order, from a seed through to river-network *topology* (channelization +
//! Strahler ordering) — stopping short of `carveRiverValleys()`'s tail, which
//! needs river polyline tracing and channel width/depth stamping this port
//! hasn't ported yet (`cartalith-hydrology`'s own doc comment on `build_channels`).
//!
//! ## What this deliberately does NOT reproduce, and why
//!
//! - **World-Structure archetypes** (`state.world_structure.enabled`, default
//!   `false`): `generateContinentalityField()` and `applyWorldStructureSeaLevel()`
//!   are both unconditional no-ops at that default (`if(!ws.enabled) return`) —
//!   so at the JS engine's own default settings, omitting them entirely is
//!   bit-identical, not an approximation. `MVP_SCOPE.md` point 5 (world-structure
//!   archetypes) remains open separately.
//! - **Graph-driven orogeny** (`state.tect.tectonicGraph`, default `false`):
//!   only ever turns on when a World-Structure archetype is active
//!   (`deriveFromWorldStructure()`), so it's already off in the default path
//!   this function reproduces — `oro` is passed as `None` throughout, matching
//!   `orogenyField`'s own default-`null` state.
//! - **`stampVolcanoesProvinces`** (`state.volc.provinces`, default `true` —
//!   already a known, previously-logged deviation from JS's literal default,
//!   not new to this pass): only `stampVolcanoesSimple` is ported, so this
//!   function always uses it regardless of the `provinces` default.
//! - **Ocean-current SST folding** (`state.climate.currents`, default `true`):
//!   `MVP_SCOPE.md` explicitly names ocean-current terrain coupling a stretch
//!   goal and grants permission to defer it if documented — taken here despite
//!   the JS default being *on*, consistent with `simulate_weather`'s own
//!   already-documented deferral of the same mechanism.
//! - **`carveRiverValleys()`** (default `state.carveRivers = true`): needs
//!   `buildRiverNetwork`'s width/polyline half and `enforceChannelDescent`,
//!   neither ported yet. This function's output ends at the same point
//!   `computeFlow(true)` reaches in `generate()`, one step before it.

use cartalith_climate::{apply_climate_moisture_correctors, compute_temperature, simulate_weather, ClimateParams, WeatherParams};
use cartalith_hydrology::{build_channels, compute_flow, strahler_from_receivers, ChannelResult};
use cartalith_terrain::{
    assign_plates, build_age_field, build_plates, compute_flexure, compute_height, compute_heterogeneity,
    compute_resistance, compute_stress, compute_warp, gauss_blur, normalize_field, stamp_craters,
    stamp_volcanoes_simple, HeightParams,
};

/// `state.tect` (reference HTML line 2264-2265) — the formula's real tuning
/// knobs. `resist` (used only by `streamParams()`'s stream-power tail, not
/// yet wired) and `tectonic_graph`/`fold_intensity`/`trench_depth`/
/// `fault_block`/`dynamic_lithology` (all World-Structure-gated, and WS
/// stays off in this pipeline — see the module doc comment) are omitted:
/// they're real `state.tect` fields, but nothing this function calls reads
/// them yet.
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
}

/// `state.volc` (reference HTML line 2266) minus `provinces` — see the
/// module doc comment on why `stampVolcanoesSimple` always runs here
/// regardless of that flag's default.
pub struct VolcanismParams {
    pub count: i32,
    pub age: f64,
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

/// Everything `generate_terrain` needs from `state` — one struct per
/// `state` sub-object (`tect`/`volc`/`crater`/`planet`/`climate`), plus the
/// handful of top-level fields (`world`/`seaLevel`/`peakM`/`mapWidthKm`)
/// every stage reads directly.
pub struct WorldParams {
    pub gw: usize,
    pub gh: usize,
    pub world: bool,
    pub sea_level: f64,
    pub peak_m: f64,
    pub map_width_km: f64,
    pub tect: TectonicParams,
    pub volc: VolcanismParams,
    pub crater: CraterParams,
    pub planet: PlanetParams,
    pub climate: ClimateInputParams,
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
            },
            volc: VolcanismParams { count: 20, age: 0.40 },
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
        }
    }
}

/// Everything `generate_terrain` produces — the Rust equivalent of the JS
/// module globals `field`/`plateId`/`boundaryMask`/.../`flowField`/
/// `tempField`/`rainField` a fresh `generate()` call leaves behind, as they
/// stand right after `computeFlow(true)` (reference HTML line 3387) —
/// `carveRiverValleys()`'s tail runs after that point and isn't included
/// (see the module doc comment).
pub struct WorldState {
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
    pub channels: ChannelResult,
    pub stream_order: Vec<i16>,
}

/// Runs the full ported pipeline once, from a seed to river-network
/// topology. See the module doc comment for the exact JS function this
/// mirrors and what's deliberately not reproduced yet.
pub fn generate_terrain(p: &WorldParams) -> WorldState {
    let gw = p.gw;
    let gh = p.gh;
    let world = p.world;

    // ---- buildTectonicSubstrate (reference HTML lines 3396-3462) ----
    let warp = compute_warp(gw, gh, p.tect.seed, p.tect.warp, world);
    let (warp_x, warp_y) = match &warp {
        Some((wx, wy)) => (Some(wx.as_slice()), Some(wy.as_slice())),
        None => (None, None),
    };

    let plates = build_plates(gw, gh, p.tect.seed as u32, p.tect.plates, p.tect.lloyd, world, None);
    let plate_id = assign_plates(gw, gh, world, &plates, warp_x, warp_y);
    let stress = compute_stress(gw, gh, world, &plate_id, &plates, p.tect.vel, p.tect.blur_r);
    let flexure_field = compute_flexure(gw, gh, &stress.boundary_mask, &stress.stress_field, p.tect.blur_r, world);

    let base_raw: Vec<f32> = plate_id.iter().map(|&pid| plates[pid].base as f32).collect();
    let base_field = gauss_blur(&base_raw, (p.tect.blur_r * 0.35).max(2.0), gw, gh, world);

    let age_field = build_age_field(gw, gh, &stress.boundary_mask);

    let heterogeneity_field =
        compute_heterogeneity(gw, gh, p.tect.seed, p.map_width_km, world, &age_field, warp_x, warp_y);
    let resistance_field = compute_resistance(gw, gh, &plate_id, &plates, &age_field);
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
    if p.volc.count > 0 {
        // stampVolcanoesSimple, not stampVolcanoesProvinces -- see the
        // module doc comment.
        stamp_volcanoes_simple(
            gw,
            gh,
            p.tect.seed as u32,
            p.map_width_km,
            p.peak_m,
            &stress.boundary_mask,
            p.volc.count,
            p.volc.age,
            &mut field,
            &mut volcanic_field,
        );
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

    // applyWorldStructureSeaLevel(): a no-op at World-Structure's default
    // `enabled:false` -- see the module doc comment. `p.sea_level` is used
    // as-is below.

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
        sea_level: p.sea_level,
        peak_m: p.peak_m,
        albedo_k: p.climate.albedo_k,
    };
    let temperature = compute_temperature(gw, gh, &field, None, &climate_params);

    let weather_params = WeatherParams {
        world,
        lat_n: p.climate.lat_n,
        lat_s: p.climate.lat_s,
        pole_temp: p.climate.pole_temp,
        equator_temp: p.climate.equator_temp,
        tilt_deg: p.planet.axial_tilt_deg,
        rotation_hours: p.planet.rotation_hours,
        lapse_rate: p.climate.lapse_rate,
        sea_level: p.sea_level,
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
        p.sea_level,
        world,
        p.climate.lat_n,
        p.climate.lat_s,
        p.climate.zonal_k,
    );
    // applyOceanCurrents()/computeSeasons(): deferred -- see the module
    // doc comment ("Ocean-current SST folding").

    let flow_discharge = compute_flow(gw, gh, &field, Some(&rainfall), true, world);

    // ---- river network topology (cartalith-hydrology's own scope note on
    // build_channels: not the whole of buildRiverNetwork) ----
    let channels = build_channels(&field, &flow_discharge, gw, gh, p.sea_level, world, 1.0, p.map_width_km);
    let stream_order = strahler_from_receivers(&channels.recv, &flow_discharge, &channels.chan);

    WorldState {
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
    }
}
