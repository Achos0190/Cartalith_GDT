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
//! ## World-Structure archetypes — ported, including graph-driven orogeny
//!
//! `state.world_structure.enabled` (default `false`, so this whole section
//! is a no-op path at the JS engine's own defaults) now runs
//! `generate_continentality_field`/`apply_world_structure_sea_level`
//! (`cartalith-terrain`) and derives `tect.plates`/`tect.vel`/`volc.count`
//! from the archetype's own params (`deriveFromWorldStructure()`, reference
//! HTML lines 2528-2538) exactly as JS does. JS's `deriveFromWorldStructure()`
//! also always sets `state.tect.tectonicGraph=true` alongside that
//! derivation (the only trigger this port models — nothing here exposes an
//! independent toggle, matching JS's own only caller); when enabled, this
//! traces `stress.boundary_mask` into typed polylines
//! (`trace_boundaries`/`tag_boundary_types`) and stamps `build_orogeny_field`
//! and `smooth_orogeny` (T2+T3, reference HTML lines 2981-3080) into `oro`,
//! which `compute_height` folds in exactly as `fillHeightRows`'s own
//! `T=oro?oro[i]+Math.min(sf,0):sf` does — the kept negative (divergent)
//! stress layered under the structured margin features, not replaced by
//! them. `foldIntensity`/`trenchDepth`/`faultBlock` (JS's own orogeny-only
//! T5 tuning knobs) aren't exposed as configurable params yet, so
//! `foldK`/`trenchK`/`faultBlockK` are hardcoded to the exact values JS's
//! own null-coalescing defaults produce when nothing overrides them
//! (`0.16`, `1.0`, `0`) — not a separate approximation, the same reasoning
//! `build_orogeny_field`'s own doc comment gives for hardcoding `block_w`/
//! `jitter`.
//!
//! ## What else this deliberately does NOT reproduce, and why
//! - **Ocean-current SST folding** (`state.climate.currents`, JS default
//!   `true`): ported (`cartalith_climate::ocean_sst_anomaly`/
//!   `apply_ocean_currents`, both built on `compute_ocean_current`/
//!   `deflect_flow`) and reachable via `p.climate.currents`, now fully
//!   verified (`compute_ocean_current` golden-tested bit-exact including
//!   its western-intensification heuristic; the two orchestration
//!   functions checked line-for-line against JS) — see
//!   `WeatherParams::currents`'s own doc comment. Still `false` here,
//!   same fixture-cascading reasoning as the other two items on this list.
//! - **Terrain wind deflection** (`buildWind`'s `deflectFlow` block, JS
//!   unconditional since v1.78): ported (`cartalith_climate::deflect_flow`,
//!   now golden-verified — `golden_parity_deflect_flow.rs`, bit-exact) and
//!   reachable via `p.climate.terrain_wind_deflection`. `build_wind`'s own
//!   wiring around it (the `block` field's `land`/`mtn` terms, the
//!   `DeflectFlowParams` constants, the elevation-band damping combine)
//!   checked line-for-line against reference HTML lines 5521-5535 — matches
//!   exactly. Still off by default, same reasoning as `stampVolcanoesProvinces`
//!   (`generate_terrain`'s own doc comment): flipping it changes the wind
//!   field every downstream climate/erosion stage reads, which would
//!   invalidate existing fixtures without also re-extracting them.
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

/// Cartalith's generation-stage dependency chain as a deferred staleness
/// graph (`UNIFIED_TOOL_PLAN.md` milestone A). Unwired: the pipeline below
/// does not consult it yet.
pub mod staleness;

/// The process-global, ten-stage progress counter the Android spec's staged
/// generation readout reads (`ANDROID_BUILD_SCOPE.md`). Wired: every
/// `advance()` call in `generate_terrain_inner` below is real, not a stub.
pub mod progress;

/// `sculptCommit`'s River/Lake water hooks (`UNIFIED_TOOL_PLAN.md`
/// milestone C). Unwired: the pipeline below does not call it yet.
pub mod sculpt_commit;

/// `exportRegionTiles`' assembly of the region-export archive, complete with
/// its per-tile PNG, gzip and `.zip` steps (`UNIFIED_TOOL_PLAN.md` milestones
/// E and E2). Unwired: nothing calls it yet.
pub mod region_export;

/// The LOD tile-pyramid bake, the persistent atlas it writes into, the
/// portable `World/` archive, and the finalize lock that keeps a baked world
/// from being regenerated out from under its own atlas
/// (`GUI_GAP_REGISTER.md` WW-01/PR-10/S4/S5).
pub mod bake;

/// `exportGeoJSON` and its two feature builders (`UNIFIED_TOOL_PLAN.md`
/// milestone E2). Unwired: nothing calls it yet.
pub mod geojson;

/// Heightmap import + the tectonic-inversion pass that makes an imported
/// elevation field behave like a generated world (`GENERATION_PARAMETERS.md`).
pub mod import;

/// `centerLandmasses()` — the X-rotation that moves the emptiest meridian
/// to the map edge, over every retained raster at once
/// (`GUI_GAP_REGISTER.md` MS-01).
pub mod center;

/// `erode()` — the reference's droplet-erosion button (`PARITY_AUDIT.md` §23
/// F11). An OP, not a generation stage: `generate_terrain` below does not call
/// it, and it takes its own `ErodeOpts` rather than `WorldParams` fields.
pub mod erode_op;

/// The channel atlas — the world's affordance fields packed three to an RGB8
/// PNG, plus its decode manifest (`chanAtlasChk`, `PARITY_AUDIT.md` §5 item
/// 14). Data, not a picture: the export raster it ships alongside is
/// `cartalith-godot`'s `render::bake_rect`.
pub mod channel_atlas;

use cartalith_climate::{
    apply_climate_moisture_correctors, apply_ocean_currents, compute_temperature, simulate_weather, ClimateParams,
    WeatherParams,
};
use cartalith_climate::tides::{compute_tide_field, TideParams};
use cartalith_erosion::{
    apply_tidal_sedimentation, coastal_process, glacial_kernel, hillslope_diffuse, isostatic_rebound,
    recompute_resistance_after_erosion, route_sediment, stream_power_kernel, velocity_erode_kernel, CoastalParams,
    GlacialParams, StreamPowerParams, VelocityParams,
};
use cartalith_hydrology::{
    build_channels, compute_flow, enforce_channel_descent, river_width_scale_k, strahler_from_receivers,
    trace_river_polylines, ChannelResult,
};
use cartalith_terrain::{
    apply_world_structure_sea_level, assign_plates, build_age_field, build_orogeny_field, build_plates,
    compute_flexure, compute_height, compute_heterogeneity, compute_resistance, compute_stress, compute_warp,
    gauss_blur, generate_continentality_field, normalize_field, smooth_orogeny, stamp_craters,
    stamp_volcanoes_provinces, stamp_volcanoes_simple, tag_boundary_types, trace_boundaries, HeightParams,
    OrogenyParams, WorldStructure,
};

// `Math.round` (ties toward `+Infinity`), from `cartalith-jsmath`.
use cartalith_jsmath::js_round;

/// `state.tect` (reference HTML line 2264-2265) — the formula's real tuning
/// knobs, plus `resist` (`streamParams()`'s erodibility-resistance weight,
/// now read by `carveRiverValleys`'s light stream-power pass) and
/// `dynamic_lithology` (`eroFinish`'s L4 exhumation-hardening gate — see
/// `recompute_resistance_after_erosion`'s call site below). The remaining
/// World-Structure-gated fields (`tectonicGraph`/`foldIntensity`/
/// `trenchDepth`/`faultBlock`) stay omitted — WS stays off in this pipeline
/// (see the module doc comment), so nothing here reads them.
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
pub struct VolcanismParams {
    pub count: i32,
    pub age: f64,
    pub provinces: bool,
}

/// `state.crater` (reference HTML line 2267).
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
pub struct PlanetParams {
    pub g: f64,
    pub rotation_hours: f64,
    pub axial_tilt_deg: f64,
}

/// `state.climate` (reference HTML line 2280) fields this pipeline's
/// temperature/weather/moisture-corrector stages actually read.
#[derive(Clone, Debug, PartialEq)]
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
    /// `cartalith_climate::WeatherParams::terrain_wind_deflection` passed
    /// straight through — see that field's own doc comment for why this
    /// port defaults it `false` where JS has no equivalent flag (always on
    /// since v1.78).
    pub terrain_wind_deflection: bool,
    /// `cartalith_climate::WeatherParams::currents`/`apply_ocean_currents`'s
    /// own gate, passed straight through — see that field's own doc
    /// comment for why this port defaults it `false` where JS defaults it
    /// `true`.
    pub currents: bool,
    pub current_k: f64,
}

/// `state.stream` (reference HTML line 2269) fields `carveRiverValleys`'s
/// light stream-power pass reads via `streamParams()`. `cycles` is omitted
/// — only read by `evolveCoupled()`, the manual "Stream evolve" tool, not
/// `carveRiverValleys`.
#[derive(Clone, Debug, PartialEq)]
pub struct StreamParams {
    pub uplift: f64,
    pub k: f64,
    pub iters: i32,
    pub deposit: f64,
    pub climate_k: f64,
}

/// The reference's **manual erosion buttons**, exposed here as
/// generation-time passes instead (`GUI_GAP_REGISTER.md` §19, WW-02/MS-04/
/// MS-05; permitted by `DECISIONS.md` §7d).
///
/// **Every toggle is off and every cycle count is zero by default**, so a
/// default `generate_terrain` is bit-identical to before this struct existed
/// — the same guarantee the reference gives for its own ops (*"A new op
/// (never auto-runs) → generate() bit-identical at defaults"*).
///
/// ## Why parameters and not buttons
///
/// The reference runs none of these inside `generate()`; each is a button
/// over the finished field followed by `computeFlow(true); refreshClimate()`.
/// This port takes the §7d route: the *same* kernels, run at the *end* of
/// generation (after `carve_rivers`, which is where the reference's finished
/// field is), followed by the same flow+climate refresh — [`refresh_climate`].
/// Growing the reference's opt-in buttons on top of these is still open and
/// costs nothing extra, since the run path now exists.
///
/// ## Order
///
/// Fixed and not user-orderable: `velocity → glacial → coastal →
/// hillslope → evolve → sediment_fill → tidal_flats`. It is the reference's own panel
/// order, which is the only ordering evidence there is — the reference never
/// composes two of these in one op, so there is no reference answer and
/// therefore **no golden fixture for the composed result**. Each *kernel* is
/// golden-parity bit-exact on its own (`cartalith-erosion::passes`); the
/// sequence is this port's choice, disclosed rather than implied.
///
/// Field names follow the reference's `state.velo`/`state.glacial`/
/// `state.coastal`/`state.erosion` keys; the derived knobs each kernel needs
/// beyond these (`dt`, `rain_rate`, `evap`, …) come from the reference's own
/// `veloParams()`/`glacialParams()` mappings, not from new judgement.
#[derive(Clone, Debug, PartialEq)]
pub struct ErosionPassParams {
    /// `velocityErodeKernel` — Mei virtual-pipes hydraulic erosion.
    pub velocity: bool,
    /// `state.velo.iters`, clamped 10..160 by `veloParams()`. Default `60`.
    pub velo_iters: i32,
    /// `state.velo.strength`. Drives `capacity` and `erodeK`. Default `0.5`.
    pub velo_strength: f64,
    /// `state.velo.meander`. Drives `centrifugalK`. Default `0.6`.
    pub velo_meander: f64,

    /// `glacialKernel` — ice abrasion carving U-shaped valleys.
    pub glacial: bool,
    /// `state.glacial.kg`, erodibility. Default `0.15`.
    pub glacial_kg: f64,
    /// `state.glacial.mg`, discharge exponent. Default `0.4`.
    pub glacial_mg: f64,
    /// `state.glacial.snowline`, fraction of the above-sea range. Default `0.65`.
    pub glacial_snowline: f64,
    /// `state.glacial.uFactor`, trough-wall share. Default `0.6`.
    pub glacial_u_factor: f64,
    /// `state.glacial.passes`. Default `8`.
    pub glacial_passes: i32,

    /// `coastalProcess` — cliff retreat, estuaries, tidal marsh.
    pub coastal: bool,
    /// `state.coastal.waveStr`. Default `0.5`.
    pub wave_str: f64,
    /// `state.coastal.estuaryDepth`. Default `0.08`.
    pub estuary_depth: f64,
    /// `state.coastal.marshBand`. Default `0.03`.
    pub marsh_band: f64,
    /// `state.coastal.passes`. Default `4`.
    pub coastal_passes: i32,

    /// `hillslopeDiffuseCPU` — `∂z/∂t = D∇²z`.
    pub hillslope: bool,
    /// `state.erosion.diffuseD`. Default `0.15`.
    pub diffuse_d: f64,
    /// `state.erosion.diffusePasses`. Default `6`.
    pub diffuse_passes: i32,

    /// `depositSediment()` — stream-power carve, then route the eroded mass
    /// downstream and redeposit it (mass-conserving), building deltas and
    /// floodplains instead of the broad isostatic rebound.
    pub sediment_fill: bool,
    /// `routeSediment`'s `opts.capacity`. The reference's own default, and
    /// its only caller's, is `6.0`.
    pub sediment_capacity: f64,

    /// `evolveCoupled(cycles)` — coupled climate ↔ terrain evolution, one
    /// stream-power carve + full climate refresh per cycle, so the rain
    /// driving the next cycle's incision reflects the orography it just
    /// helped build. `0` (the default) is off; `state.stream.cycles`.
    pub evolve_cycles: i32,

    /// `applyTidalSedimentation()` — the *Tidal flats* button's kernel:
    /// submerged cells inside the spring tidal range accrete toward sea
    /// level, hardest where the water is shallowest.
    ///
    /// The reference gates its own op on `tideField`, which only exists
    /// while `state.planet.tides.enabled` is on. This port has no separate
    /// enable: **this toggle is it**, and turning it on computes the tide
    /// field (`cartalith_climate::tides::compute_tide_field`) from the
    /// finished surface right before the kernel reads it — which is exactly
    /// what `refreshTides()` does in the reference before the button is
    /// reachable. `PlanetParams` carries no moon roster, so the field is
    /// built with `TideParams::default()`'s single Earth–Moon-equivalent
    /// companion at this world's own `planet.g` — the same substitution
    /// `sample_bridge`'s Tides debug view already documents.
    pub tidal_flats: bool,
    /// `applyTidalSedimentation`'s accretion rate. The reference's own
    /// default, and its only caller's, is `0.45`.
    pub tidal_k: f64,
}

impl ErosionPassParams {
    /// Every pass off, every knob at the reference's own `state` literal
    /// (reference HTML lines 2268-2275). Off means `generate_terrain` is
    /// unchanged, so the knobs are documentation until a toggle is flipped.
    pub fn off() -> Self {
        ErosionPassParams {
            velocity: false,
            velo_iters: 60,
            velo_strength: 0.5,
            velo_meander: 0.6,
            glacial: false,
            glacial_kg: 0.15,
            glacial_mg: 0.4,
            glacial_snowline: 0.65,
            glacial_u_factor: 0.6,
            glacial_passes: 8,
            coastal: false,
            wave_str: 0.5,
            estuary_depth: 0.08,
            marsh_band: 0.03,
            coastal_passes: 4,
            hillslope: false,
            diffuse_d: 0.15,
            diffuse_passes: 6,
            sediment_fill: false,
            sediment_capacity: 6.0,
            evolve_cycles: 0,
            tidal_flats: false,
            tidal_k: 0.45,
        }
    }

    /// Whether any pass would actually run — the guard that keeps a default
    /// generation from paying for the flow+climate refresh these need.
    pub fn any(&self) -> bool {
        self.velocity
            || self.glacial
            || self.coastal
            || self.hillslope
            || self.sediment_fill
            || self.evolve_cycles > 0
            || self.tidal_flats
    }
}

/// `state.world_structure` (reference HTML line 2263) — the five
/// archetype knobs `ARCHETYPES`'s presets (earth/supercontinent/
/// archipelago/volcanic/rift, reference HTML lines 2521-2526) set
/// together. This port takes the five raw values directly rather than
/// modeling named archetypes — a caller wanting "Archipelago" passes
/// `ARCHETYPES.archipelago`'s own numbers. See the module doc comment for
/// `tectonicGraph`/graph-driven orogeny, the other thing `enabled` turns on.
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
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
    /// The reference's manual erosion ops, run at the end of generation.
    /// Entirely off by default — see [`ErosionPassParams`].
    pub passes: ErosionPassParams,
    pub world_structure: WorldStructureParams,
    /// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 6: run plate assignment,
    /// domain warp, crustal heterogeneity, and the flexure/base-field blur
    /// on GPU instead of CPU. Default `false` -- with this flag at its
    /// default, `generate_terrain`'s behaviour and output are byte-for-byte
    /// identical to before this milestone (verified: every existing
    /// golden-parity test passes unmodified). **Not a performance-only
    /// switch**: per `DECISIONS.md` §7c, the GPU noise primitive is a
    /// genuinely different hash function from the CPU/JS-matching one, so
    /// `use_gpu: true` produces a different (still valid, still
    /// deterministic-per-seed) world for the same seed, not just a faster
    /// path to the same one. On any GPU init/dispatch failure, each stage
    /// falls back to CPU individually rather than crashing
    /// (`HARDWARE_ACCELERATION.md` §27) -- which path each stage actually
    /// took is recorded on `WorldState.gpu_stages_used`, not hidden.
    pub use_gpu: bool,
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
            // `provinces: true`, matching JS's own literal default.
            // stamp_volcanoes_provinces is golden-verified
            // (cartalith-terrain/tests/golden_parity_volc_provinces.rs --
            // captured by driving the real reference `generate()` under
            // Node with a small grid, bit-exact), and golden_parity_pipeline.rs
            // has been re-extracted against this default (2026-08-15,
            // cartalith-native/docs/CHANGELOG.md). golden_parity_carve.rs
            // has NOT been re-extracted yet -- it also covers
            // terrain_wind_deflection/currents, both still `false`, so
            // re-extracting it belongs with flipping those too, not here.
            volc: VolcanismParams { count: 20, age: 0.40, provinces: true },
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
                // Matching JS's real defaults now (terrain wind deflection
                // is unconditional since v1.78; state.climate.currents
                // defaults true) -- both golden-verified
                // (golden_parity_deflect_flow.rs, golden_parity_ocean_current.rs,
                // golden_parity_weather.rs's own currents_case) as of
                // cartalith-native/docs/CHANGELOG.md, 2026-08-15.
                // golden_parity_carve.rs still assumes both off -- pinned
                // there explicitly rather than left to silently break.
                terrain_wind_deflection: true,
                currents: true,
                current_k: 1.0,
            },
            stream: StreamParams { uplift: 0.0, k: 0.012, iters: 15, deposit: 0.3, climate_k: 0.5 },
            passes: ErosionPassParams::off(),
            world_structure: WorldStructureParams {
                enabled: false,
                continentality: 0.30,
                fragmentation: 0.50,
                tectonic_energy: 0.60,
                ocean_depth: 0.60,
                hotspot_density: 0.20,
            },
            use_gpu: false,
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
///
/// # Four grids that used to be here and are not
///
/// `flexure_field`, `heterogeneity_field`, `flow_area` and
/// `ChannelResult::slope` were retained on this struct and read by nothing
/// outside `generate_terrain` — 40.96 MiB a world at 2048 × 1311, resident
/// for the whole session and carried through every civilisation stage on
/// top of it. Each is still **computed**, still feeds the stage that needs
/// it (`compute_height` for the first two, the moisture correctors for
/// `flow_area`, the channel threshold for `slope`), and is dropped where
/// its last reader finishes. See `MEMORY_OPTIMIZATION_SCOPE.md` R2 for the
/// evidence each was dead, including which golden assertions went with
/// them and why that was judged safe.
pub struct WorldState {
    /// The sea level actually used for this generation — equal to
    /// `p.sea_level` unless `world_structure.enabled` re-anchored it
    /// (`apply_world_structure_sea_level`). Callers that classify land vs.
    /// ocean (a renderer, a land-fraction check) must use this, not
    /// `p.sea_level` directly.
    pub sea_level: f64,
    pub field: Vec<f32>,
    /// `u16`, not `usize`: `tect.plates` is clamped to `4..=40` at every
    /// entry point (`params.rs`'s `ParamSpec`, and the World-Structure
    /// override's own `.clamp(4, 40)`), and the import path's
    /// `pick_plate_seeds` caps at 40 too — so 8 B/cell held a number below
    /// 41. 2 B/cell is 15.36 MiB off both peak and resident at
    /// 2 048 × 1 311 (`MEMORY_OPTIMIZATION_SCOPE.md` R4). Index `plates[]`
    /// with `as usize` at the read.
    pub plate_id: Vec<u16>,
    pub boundary_mask: Vec<u8>,
    pub stress_field: Vec<f32>,
    pub age_field: Vec<f32>,
    pub resistance_field: Vec<f32>,
    /// `plateCrust()` (reference HTML line 3083): raw, unblurred per-cell
    /// plate base (`<0` = oceanic crust). Already computed internally as
    /// `base_raw` for orogeny/height, but not previously retained past
    /// `generate_terrain` -- added for `cartalith-civ`'s `buildLithology`
    /// port, which reads this exact same value (`currentLithology()`'s
    /// `crust` argument in the reference).
    pub crust_field: Vec<f32>,
    /// `StressResult::boundary_type`/`shear_field` (`cartalith-terrain`):
    /// per-cell plate-boundary classification and shear magnitude. Already
    /// computed for T2+T3 orogeny (`tag_boundary_types`/`OrogenyParams::
    /// shear`) but not previously retained past `generate_terrain` --
    /// added for `cartalith-civ`'s `buildResourcePotentials` port (Phase 2
    /// milestone 5, `PHASE2_SCOPE.md`), the same `boundaryType`/
    /// `shearField` arguments the reference passes it.
    pub boundary_type: Vec<u8>,
    pub shear_field: Vec<f32>,
    pub volcanic_field: Vec<f32>,
    pub impact_field: Vec<f32>,
    pub temperature: Vec<f32>,
    pub rainfall: Vec<f32>,
    pub flow_discharge: Vec<f32>,
    /// **`ChannelResult::slope` is released before this is stored** and is
    /// an empty `Vec` here — see `generate_terrain`'s own note at the point
    /// it drops it (`MEMORY_OPTIMIZATION_SCOPE.md` R2). `recv` and `chan`
    /// are the two arrays every consumer in this workspace actually reads.
    pub channels: Option<ChannelResult>,
    pub stream_order: Option<Vec<i16>>,
    pub river_mask: Option<Vec<u8>>,
    pub river_floor: Option<Vec<f32>>,
    /// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 6: which of the four
    /// GPU-eligible substrate stages (`"warp"`, `"heterogeneity"`,
    /// `"plate_assignment"`, `"base_field_blur"`) actually ran on GPU this
    /// generation. Empty when `p.use_gpu` was `false`, or when every stage
    /// fell back to CPU (`HARDWARE_ACCELERATION.md` §27 -- GPU failure
    /// falls back silently in terms of *correctness*, but the caller can
    /// always tell which path actually ran by reading this).
    pub gpu_stages_used: Vec<String>,
}

/// Runs the full ported pipeline once, from a seed to (when
/// `p.carve_rivers`, the JS default) carved river valleys. See the module
/// doc comment for the exact JS functions this mirrors and what's
/// deliberately not reproduced yet.
pub fn generate_terrain(p: &WorldParams) -> WorldState {
    generate_terrain_inner(p, false)
}

/// `generate_terrain`'s body, with one test-only escape hatch:
/// `force_precarve_flow` restores the reference's own literal call order
/// (the pre-carve `computeFlow(true)` that the carve path never reads --
/// see its call site below and `DECISIONS.md` §7f). Nothing but
/// `precarve_flow_skip_leaves_generation_bit_identical` passes `true`; it
/// exists so that "the skip changes nothing" is a proof this crate can run
/// rather than an argument in a comment.
fn generate_terrain_inner(p: &WorldParams, force_precarve_flow: bool) -> WorldState {
    // `progress.rs`'s own doc comment carries the full banner->stage mapping
    // every `progress::advance` call below encodes. `begin_run` resets the
    // counter to stage 0 (Planet); Planet and Extent & scale both tick
    // through immediately since neither has real computation of its own in
    // this function (see the module doc for why).
    crate::progress::begin_run();
    crate::progress::advance(crate::progress::EXTENT_SCALE);

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
    // World structure (stage 2): the continentality field is this stage's
    // own product, and it precedes the real tectonics work below despite
    // sharing this banner with it (`progress.rs`'s own doc comment).
    crate::progress::advance(crate::progress::WORLD_STRUCTURE);
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

    // Tectonics (stage 3): plates, stress, flexure, resistance and the
    // finished height field, through `compute_height`/`normalize_field`
    // below -- the rest of the "buildTectonicSubstrate" banner once World
    // structure's own slice (just above) is subtracted from its front.
    crate::progress::advance(crate::progress::TECTONICS);

    let mut gpu_stages_used: Vec<String> = Vec::new();

    // ---- GPU_LAYER_INTEGRATION_SCOPE.md milestone 6: opt-in partial-GPU
    // substrate path. `p.use_gpu=false` (the default) takes the exact same
    // code path as before this milestone -- every `if p.use_gpu` branch
    // below is additive, never altering the `else` arm's behaviour.
    //
    // Milestone 8 (context reuse): one `GpuDevice` is requested here, once,
    // and threaded through every stage below via the `_with` wrappers --
    // milestone 6 found each of the five GPU dispatches paying its own
    // ~1.3-1.4s adapter/device handshake independently, the dominant cost
    // at every size this port ships at by default below 2048x2048. A
    // `None` here (no adapter, or device-creation failure) makes every
    // stage below fall through to its existing CPU fallback exactly as it
    // did before this milestone -- one failure point instead of five
    // independent (and independently wasteful) retries of a failing
    // handshake.
    //
    // Multi-GPU (`GUI_GAP_REGISTER.md` PR-01/PR-02/PR-04/PR-05): the single
    // `init_gpu_shared_device()` call became `init_gpu_device_set()`, which
    // honours the process-wide device selection and multi-GPU mode. With no
    // preference set -- the default, and what every existing test and every
    // untouched install has -- the set holds exactly one device, obtained by
    // the exact same `PowerPreference::HighPerformance` request as before,
    // so nothing below changes. `gpu_allowed_for_grid` is the VRAM budget
    // gate and is unconditionally `true` while no budget is set.
    //
    // `supports_grid` is the second gate, and it is a *hard device* limit
    // rather than the user-set budget `gpu_allowed_for_grid` applies: a
    // whole-grid dispatch binds a `gw*gh*f32` storage buffer, and a device
    // opened with limits below that does not fail softly -- `create_bind_group`
    // raises a wgpu validation error, which panics, which takes the Godot
    // process down (`cartalith-rust-conventions`). Found by measurement, not by
    // reading: `use_gpu = true` at 8192² (a `RESOLUTION_PRESETS` entry, with the
    // shell's GPU default of on) died on exactly that during
    // `PERFORMANCE_BENCHMARKS.md`'s run. `cartalith-gpu` now opens devices at
    // the adapter's own ceilings, which covers every size this port offers on
    // real hardware; this check is what makes an adapter that still cannot
    // reach a size fall back to CPU (`HARDWARE_ACCELERATION.md` §27) instead.
    let gpu_set = if p.use_gpu && cartalith_gpu::gpu_allowed_for_grid(gw, gh) {
        cartalith_gpu::init_gpu_device_set().ok().filter(|s| s.supports_grid(gw, gh))
    } else {
        None
    };
    let gpu_device: Option<&cartalith_gpu::GpuDevice> = gpu_set.as_ref().map(|s| s.primary());

    // What this generation actually opened, for the Performance window to
    // report instead of inferring. Recorded HERE and not beside the
    // `record_usage` call at the tail of this function: that one is inside an
    // `if let Some(set)`, so a CPU-only run would leave the last GPU run's
    // reading standing. Every path through this line has just decided, so
    // both answers -- a backend, or "no device at all" -- are written on every
    // call. See `multi::record_opened_backend`.
    cartalith_gpu::record_opened_backend(gpu_set.as_ref());

    // `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 9: flow accumulation is
    // called up to FOUR times below (structural drainage, discharge-weighted
    // drainage, the river-network pass, and the post-carve recompute), so
    // its pipeline is built once here rather than per call -- milestone 8's
    // shared-device lesson applied to shader compilation as well as to the
    // adapter/device handshake. `None` whenever `use_gpu` is off or the
    // device handshake failed; every call site below falls back to the real
    // `compute_flow` in that case (`HARDWARE_ACCELERATION.md` §27).
    let gpu_flow = gpu_device.map(cartalith_gpu::init_gpu_flow_with);
    let flow_on_gpu = |field: &[f32], rain: Option<&[f32]>, use_rain: bool| -> Option<Vec<f32>> {
        gpu_flow.as_ref().and_then(|c| cartalith_gpu::dispatch_gpu_flow(c, gw, gh, field, rain, use_rain, world)).map(|r| r.acc)
    };

    // World-wrap isn't supported by the GPU warp kernel yet (milestone 2's
    // own deferral) -- `use_gpu` under `world=true` falls back to CPU for
    // warp specifically, same as any other GPU-unavailable case.
    let warp = if p.use_gpu && !world {
        let amp = (p.tect.warp * 0.18 * gw as f64) as f32;
        if amp < 0.5 {
            None
        } else {
            let wf = (2.5 / gw as f64) as f32; // non-world branch only, matching compute_warp's own `wf`
            // PR-02 `split tiles`: warp is the one stage in this pipeline
            // whose kernel reads nothing outside its own cell, so its row
            // bands can genuinely run on different devices at once (see
            // `warp_grid_gpu_split`'s own doc comment for the audit of why
            // every other GPU stage here cannot). `is_split()` is false
            // unless the mode is `split_tiles` AND at least two devices
            // actually opened, so the single-device path below is what runs
            // by default.
            // The `warp_split` marker is recorded on SUCCESS, not on
            // attempt: both entry points now return `None` when the device
            // cannot complete the dispatch, and a stage that fell back to
            // CPU must not appear in `gpu_stages_used`.
            let split = gpu_set.as_ref().is_some_and(cartalith_gpu::GpuDeviceSet::is_split);
            match gpu_set.as_ref().and_then(|set| {
                if split {
                    cartalith_gpu::warp_grid_gpu_split(set, gw as u32, gh as u32, p.tect.seed, wf, amp)
                } else {
                    cartalith_gpu::warp_grid_gpu_with(set.primary(), gw as u32, gh as u32, p.tect.seed, wf, amp)
                }
            }) {
                Some(wxy) => {
                    if split {
                        gpu_stages_used.push("warp_split".to_string());
                    }
                    gpu_stages_used.push("warp".to_string());
                    Some(wxy)
                }
                None => compute_warp(gw, gh, p.tect.seed, p.tect.warp, world),
            }
        }
    } else {
        compute_warp(gw, gh, p.tect.seed, p.tect.warp, world)
    };
    let (warp_x, warp_y) = match &warp {
        Some((wx, wy)) => (Some(wx.as_slice()), Some(wy.as_slice())),
        None => (None, None),
    };

    let plates = build_plates(gw, gh, p.tect.seed as u32, tect_plates, p.tect.lloyd, world, world_structure_arg);

    let plate_id = if p.use_gpu {
        let plate_x: Vec<f32> = plates.iter().map(|pl| pl.x as f32).collect();
        let plate_y: Vec<f32> = plates.iter().map(|pl| pl.y as f32).collect();
        gpu_device
            .as_ref()
            .and_then(|gpu| cartalith_gpu::assign_plates_grid_gpu_with(gpu, gw as u32, gh as u32, &plate_x, &plate_y, warp_x, warp_y))
            .filter(|ids| ids.iter().all(|&id| id >= 0)) // any unassigned cell => treat as a failed dispatch, fall back
            .map(|ids| {
                gpu_stages_used.push("plate_assignment".to_string());
                ids.into_iter().map(|id| id as u16).collect::<Vec<u16>>()
            })
            .unwrap_or_else(|| assign_plates(gw, gh, world, &plates, warp_x, warp_y))
    } else {
        assign_plates(gw, gh, world, &plates, warp_x, warp_y)
    };

    let stress = compute_stress(gw, gh, world, &plate_id, &plates, tect_vel, p.tect.blur_r);

    // compute_flexure's own body, inlined: mask by boundary, blur (GPU or
    // CPU), max-normalize (CPU either way -- a cheap reduction, not worth
    // its own kernel). `compute_flexure` itself is left completely
    // untouched for the `p.use_gpu=false` path (called directly, no
    // inlining needed there -- see the `else` arm).
    let flexure_field = if p.use_gpu {
        let mut raw = vec![0f32; gw * gh];
        for (r, (&mask, &sv)) in raw.iter_mut().zip(stress.boundary_mask.iter().zip(stress.stress_field.iter())) {
            if mask != 0 {
                *r = sv;
            }
        }
        match gpu_device
            .as_ref()
            .and_then(|gpu| cartalith_gpu::gauss_blur_grid_gpu_with(gpu, &raw, p.tect.blur_r * 3.0, gw as u32, gh as u32, world))
        {
            Some(broad) => {
                gpu_stages_used.push("base_field_blur".to_string()); // shared GPU kernel with base_field below
                let mut mx = 1e-6f64;
                for &v in &broad {
                    let v = (v as f64).abs();
                    if v > mx {
                        mx = v;
                    }
                }
                broad.iter().map(|&v| (v as f64 / mx) as f32).collect()
            }
            None => compute_flexure(gw, gh, &stress.boundary_mask, &stress.stress_field, p.tect.blur_r, world),
        }
    } else {
        compute_flexure(gw, gh, &stress.boundary_mask, &stress.stress_field, p.tect.blur_r, world)
    };

    let base_raw: Vec<f32> = plate_id.iter().map(|&pid| plates[pid as usize].base as f32).collect();
    let base_field = if p.use_gpu {
        match gpu_device.and_then(|gpu| {
            cartalith_gpu::gauss_blur_grid_gpu_with(gpu, &base_raw, (p.tect.blur_r * 0.35).max(2.0), gw as u32, gh as u32, world)
        }) {
            Some(v) => {
                if !gpu_stages_used.iter().any(|s| s == "base_field_blur") {
                    gpu_stages_used.push("base_field_blur".to_string());
                }
                v
            }
            None => gauss_blur(&base_raw, (p.tect.blur_r * 0.35).max(2.0), gw, gh, world),
        }
    } else {
        gauss_blur(&base_raw, (p.tect.blur_r * 0.35).max(2.0), gw, gh, world)
    };

    let age_field = build_age_field(gw, gh, &stress.boundary_mask);

    let heterogeneity_field = if p.use_gpu && !world {
        let hetero_seed = p.tect.seed ^ 0x44bb; // matches compute_heterogeneity's own seed derivation
        let hf = (1.5 * cartalith_terrain::terrain_detail_k(gw, p.map_width_km)) as f32;
        let wx = warp_x.unwrap_or(&[]);
        let wy = warp_y.unwrap_or(&[]);
        let zero_wx;
        let zero_wy;
        let (wx, wy) = if wx.len() == gw * gh && wy.len() == gw * gh {
            (wx, wy)
        } else {
            zero_wx = vec![0f32; gw * gh];
            zero_wy = vec![0f32; gw * gh];
            (zero_wx.as_slice(), zero_wy.as_slice())
        };
        match gpu_device.and_then(|gpu| {
            cartalith_gpu::heterogeneity_grid_gpu_with(gpu, gw as u32, gh as u32, hetero_seed, hf / gw as f32, &age_field, wx, wy)
        }) {
            Some(mut out) => {
                gpu_stages_used.push("heterogeneity".to_string());
                let mut mx = 1e-6f64;
                for &v in &out {
                    let v = (v as f64).abs();
                    if v > mx {
                        mx = v;
                    }
                }
                for v in &mut out {
                    *v = (*v as f64 / mx) as f32;
                }
                out
            }
            None => compute_heterogeneity(gw, gh, p.tect.seed, p.map_width_km, world, &age_field, warp_x, warp_y),
        }
    } else {
        compute_heterogeneity(gw, gh, p.tect.seed, p.map_width_km, world, &age_field, warp_x, warp_y)
    };
    let mut resistance_field = compute_resistance(gw, gh, &plate_id, &plates, &age_field);

    // resistanceToOrogeny() (reference HTML lines 3433-3444): T2+T3,
    // gated on `state.tect.tectonicGraph`, which JS's own
    // deriveFromWorldStructure() sets true exactly when World-Structure is
    // enabled (see this module's doc comment) -- the only trigger this
    // port models, matching the doc comment's own "not modeled at all"
    // note on foldIntensity/trenchDepth/faultBlock: nothing here exposes
    // those T5 knobs yet, so foldK/trenchK/faultBlockK are the exact
    // values JS's own null-coalescing defaults produce when nothing
    // overrides them (`0.16*1`, `1.0`, `0`), not a separate approximation.
    let oro = if p.world_structure.enabled {
        let mut graph = trace_boundaries(&stress.boundary_mask, gw, gh);
        tag_boundary_types(&mut graph, &stress.boundary_type, gw);
        let oro_params = OrogenyParams {
            blur_r: p.tect.blur_r,
            seed: p.tect.seed,
            shear: Some(&stress.shear_field),
            fold_k: 0.16,
            trench_k: 1.0,
            fault_block_k: 0.0,
        };
        let raw = build_orogeny_field(&graph.polylines, &stress.stress_field, &base_raw, gw, gh, &oro_params);
        Some(smooth_orogeny(&raw, gw, gh, p.tect.blur_r, world))
    } else {
        None
    };

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
        oro.as_deref(),
        &height_params,
    );
    let mut field = normalize_field(&raw_height);

    // ---- volcanism + craters (reference HTML lines 3365-3369) ----
    crate::progress::advance(crate::progress::VOLCANISM);
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
    // Milestone 9: the GPU path here is a genuinely different ALGORITHM
    // (per-cell D8 direction + pointer-doubling subtree sums, in u32 fixed
    // point) rather than a translation of the CPU function's global
    // descending-height sort and walk -- see `gpu_flow.wgsl`. Verified
    // against the real `compute_flow` at 512x512: bit-exact for the
    // `use_rain=false` seeding this very call uses, and within 2.3e-4
    // relative for discharge seeding, with no measured change to the river
    // network or to settlement placement (see the scope doc's milestone 9
    // entry). Falls back to the CPU function whenever GPU is unavailable.
    let flow_area = match flow_on_gpu(&field, None, false) {
        Some(v) => {
            gpu_stages_used.push("flow".to_string());
            v
        }
        None => compute_flow(gw, gh, &field, None, false, world),
    };

    let climate_params = climate_params_for(p, sea_level);
    let mut temperature = compute_temperature(gw, gh, &field, None, &climate_params);

    let weather_params = weather_params_for(p, sea_level);
    // decl=0: refreshClimate()'s own simulateWeather(state.climate.wIters)
    // call passes no declination argument, which defaults to 0 (annual
    // mean, no seasonal tilt) -- reference HTML line 5154.
    // GPU_LAYER_INTEGRATION_SCOPE.md milestone 7: simulate_weather's inner
    // loop. Unlike every other GPU-wired stage above, this one's own
    // working set (the coarse `min(gw,240)` grid `build_weather_grid`
    // builds) doesn't grow with `gw`/`gh` once past 240 -- real measurement
    // found GPU losing to CPU even with the shared `gpu_device` (0.93x at
    // the real 240x240/70-iters working size), so `p.use_gpu=true` still
    // takes this path (consistent `gpu_stages_used` reporting, real
    // per-stage fallback), but don't expect it to ever win the way the
    // stages above eventually did -- see the scope doc's own milestone 7
    // section for the honest numbers.
    let mut rainfall = if p.use_gpu {
        let grid = cartalith_climate::build_weather_grid(gw, gh, &field, 0.0, &weather_params);
        match gpu_device.and_then(|gpu| {
            cartalith_gpu::simulate_weather_loop_gpu_with(
                gpu,
                &grid.eh,
                &grid.tc,
                &grid.sst_evap,
                &grid.wx,
                &grid.wy,
                &grid.w_init,
                grid.ww as u32,
                grid.wh as u32,
                p.climate.w_iters,
                grid.sea as f32,
                grid.ocean_hum as f32,
                grid.evap as f32,
                grid.ocean as f32,
                grid.rain_k as f32,
                grid.dry as f32,
                grid.step as f32,
                grid.bulk_evap,
                grid.wrap_x,
            )
        }) {
            Some((_w, rain)) => {
                gpu_stages_used.push("weather".to_string());
                cartalith_climate::finish_weather_grid(&grid.eh, rain, grid.ww, grid.wh, grid.wrap_x, grid.sea, gw, gh)
            }
            None => simulate_weather(gw, gh, &field, p.climate.w_iters, 0.0, &weather_params),
        }
    } else {
        simulate_weather(gw, gh, &field, p.climate.w_iters, 0.0, &weather_params)
    };

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
    // applyOceanCurrents() (reference HTML lines 5270-5288): gated on
    // p.climate.currents, off by default in this port -- see
    // WeatherParams::currents's own doc comment. computeSeasons() stays
    // deferred (module doc comment).
    if p.climate.currents {
        apply_ocean_currents(
            gw,
            gh,
            &field,
            &mut temperature,
            &mut rainfall,
            sea_level,
            world,
            p.climate.lat_n,
            p.climate.lat_s,
            p.climate.equator_temp,
            p.climate.pole_temp,
            p.planet.axial_tilt_deg,
            p.planet.rotation_hours,
            p.climate.wind_manual,
            p.climate.wind_dir_deg,
            p.climate.press_k,
            p.climate.current_k,
        );
    }

    // `computeFlow(true)` before `carveRiverValleys()` (reference HTML line
    // 8760). **Deliberate deviation from the reference's own call order,
    // disclosed here and in `DECISIONS.md` §7f rather than taken silently**
    // (`CLAUDE.md`): in JS `flowField` is a module global the renderer and
    // every overlay may read at any moment, so the reference has to keep it
    // current between the two ops. Here it is a *local*, and when
    // `p.carve_rivers` is on (the default) every statement in the carve
    // block below reads `field`, `pre`, `stress`, `resistance_field`,
    // `rainfall` and its own `flow_for_network` -- never `flow_discharge` --
    // before step (3) overwrites it wholesale. So on the default path the
    // call's result is discarded unread: 402 ms of a measured 4.83 s
    // generation at 2048^2 (~8 %), for a skip rather than an algorithm.
    //
    // When `carve_rivers` is off this call **is** the output, so the skip is
    // conditional, never unconditional.
    //
    // `gpu_stages_used` is unaffected: `flow_on_gpu` returns `Some` iff
    // `gpu_flow` is `Some`, which is fixed for the whole function, and the
    // two flow calls inside the carve block push the same `"flow"` string
    // under the same condition -- so the vector's contents cannot differ.
    // `precarve_flow_skip_leaves_generation_bit_identical` holds all of this
    // to `assert_eq!` identity against the unskipped call order.
    let mut flow_discharge = if p.carve_rivers && !force_precarve_flow {
        Vec::new()
    } else {
        match flow_on_gpu(&field, Some(&rainfall), true) {
            Some(v) => {
                if !gpu_stages_used.iter().any(|s| s == "flow") {
                    gpu_stages_used.push("flow".to_string());
                }
                v
            }
            None => compute_flow(gw, gh, &field, Some(&rainfall), true, world),
        }
    };
    if !p.carve_rivers {
        // No carve pass this run: the direct `flow_discharge` just computed
        // above IS Hydrology's whole output, and the climate priming pass
        // already run above (the "natural order" block) is never refreshed
        // again below -- so Erosion, Hydrology and Climate are all as done
        // as they are going to get. Erosion itself never actually computed
        // anything here (no light carve without `carve_rivers`), but the
        // standalone `passes.*` toggles below (velocity/glacial/coastal/
        // hillslope/evolveCoupled/sediment/tidal) are that same stage's own
        // work too and `advance` is monotonic, so ticking through here does
        // not hide any of it if one fires.
        crate::progress::advance(crate::progress::EROSION);
        crate::progress::advance(crate::progress::HYDROLOGY);
        crate::progress::advance(crate::progress::CLIMATE);
    }

    let mut channels = None;
    let mut stream_order = None;
    let mut river_mask = None;
    let mut river_floor = None;

    if p.carve_rivers {
        // ---- carveRiverValleys (reference HTML lines 8761-8789) ----
        // Erosion (stage 5): the light physical erosion pass below is this
        // stage's real, generation-time work (`progress.rs`'s own doc
        // comment on why the earlier climate-priming pass above gets no
        // bump of its own).
        crate::progress::advance(crate::progress::EROSION);
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
        //
        // Hydrology (stage 6): `build_channels`/`strahler_from_receivers`/
        // `trace_river_polylines`/the carve loop below produce the
        // `channels`/`stream_order`/`river_mask`/`river_floor` fields
        // `WorldState` actually stores -- this stage's real product.
        crate::progress::advance(crate::progress::HYDROLOGY);
        let flow_for_network = match flow_on_gpu(&field, Some(&rainfall), true) {
            Some(v) => {
                if !gpu_stages_used.iter().any(|s| s == "flow") {
                    gpu_stages_used.push("flow".to_string());
                }
                v
            }
            None => compute_flow(gw, gh, &field, Some(&rainfall), true, world),
        };

        // (2) vector network -> distance-field channel carve + lock
        let mut ch = build_channels(&field, &flow_for_network, gw, gh, sea_level, world, p.river_density, p.map_width_km);
        // `MEMORY_OPTIMIZATION_SCOPE.md` R2: `ChannelResult::slope` has no
        // reader anywhere in this workspace -- `strahler_from_receivers` and
        // `trace_river_polylines` below take `recv`/`chan` only, and the
        // slope-area test that produced it already consumed it inside
        // `build_channels`. Released here rather than deleted from
        // `build_channels` itself, because `golden_parity_river.rs` asserts
        // it cell for cell against the JS reference and that check is worth
        // more than the transient it costs for the length of one call.
        // 10.24 MiB a world at 2048x1311, off the resident set for good.
        ch.slope = Vec::new();
        let order = strahler_from_receivers(&ch.recv, &flow_for_network, &ch.chan);
        let polys = trace_river_polylines(&order, &ch.recv, gw, gh, 1);

        let width_k = river_width_scale_k(p.map_width_km);

        // The drawn river's width, which until 2026-08-30 did not exist: the
        // renderer tested `chan[i] != 0`, so every river was one grid cell
        // wide whatever its order and whatever the world's real extent. This
        // is the reference's own disc stamp (HTML 4528-4543), and it is
        // computed here rather than in `build_channels` because it needs
        // Strahler order, which is `strahler_from_receivers`' output above.
        //
        // Costs one `gw*gh` f32 grid, and is skipped entirely when the stamp
        // would be uniform anyway -- see `stamp_river_intensity`'s own note on
        // the 0.5 half-width floor, which binds at world scale.
        ch.intensity = cartalith_hydrology::stamp_river_intensity(
            &field,
            &flow_for_network,
            &ch.chan,
            &order,
            gw,
            gh,
            world,
            cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km),
            width_k,
        );

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
        //
        // Climate (stage 7): this refresh -- not the priming pass computed
        // before Erosion above -- is climate's real, final, stored state on
        // this (default) path. `progress.rs`'s own doc comment explains why
        // the earlier pass gets no bump: showing it would walk the counter
        // backward from Climate to Erosion/Hydrology, which `advance`'s
        // monotonic contract forbids.
        crate::progress::advance(crate::progress::CLIMATE);
        flow_discharge = match flow_on_gpu(&field, Some(&rainfall), true) {
            Some(v) => {
                if !gpu_stages_used.iter().any(|s| s == "flow") {
                    gpu_stages_used.push("flow".to_string());
                }
                v
            }
            None => compute_flow(gw, gh, &field, Some(&rainfall), true, world),
        };
        temperature = compute_temperature(gw, gh, &field, None, &climate_params);
        rainfall = if p.use_gpu {
            let grid = cartalith_climate::build_weather_grid(gw, gh, &field, 0.0, &weather_params);
            match gpu_device.and_then(|gpu| {
                cartalith_gpu::simulate_weather_loop_gpu_with(
                    gpu,
                    &grid.eh,
                    &grid.tc,
                    &grid.sst_evap,
                    &grid.wx,
                    &grid.wy,
                    &grid.w_init,
                    grid.ww as u32,
                    grid.wh as u32,
                    p.climate.w_iters,
                    grid.sea as f32,
                    grid.ocean_hum as f32,
                    grid.evap as f32,
                    grid.ocean as f32,
                    grid.rain_k as f32,
                    grid.dry as f32,
                    grid.step as f32,
                    grid.bulk_evap,
                    grid.wrap_x,
                )
            }) {
                Some((_w, rain)) => {
                    if !gpu_stages_used.iter().any(|s| s == "weather") {
                        gpu_stages_used.push("weather".to_string());
                    }
                    cartalith_climate::finish_weather_grid(&grid.eh, rain, grid.ww, grid.wh, grid.wrap_x, grid.sea, gw, gh)
                }
                None => simulate_weather(gw, gh, &field, p.climate.w_iters, 0.0, &weather_params),
            }
        } else {
            simulate_weather(gw, gh, &field, p.climate.w_iters, 0.0, &weather_params)
        };
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
        // applyOceanCurrents() -- refreshClimate()'s own next step
        // (reference HTML line 8783: `computeFlow(true); refreshClimate();`).
        if p.climate.currents {
            apply_ocean_currents(
                gw,
                gh,
                &field,
                &mut temperature,
                &mut rainfall,
                sea_level,
                world,
                p.climate.lat_n,
                p.climate.lat_s,
                p.climate.equator_temp,
                p.climate.pole_temp,
                p.planet.axial_tilt_deg,
                p.planet.rotation_hours,
                p.climate.wind_manual,
                p.climate.wind_dir_deg,
                p.climate.press_k,
                p.climate.current_k,
            );
        }

        channels = Some(ch);
        stream_order = Some(order);
        river_mask = Some(rmask);
        river_floor = Some(rfloor);
    }

    // ---- the reference's manual erosion ops, as generation-time passes ----
    // `ErosionPassParams`; `GUI_GAP_REGISTER.md` §19, WW-02/MS-04/MS-05.
    // Runs here, at the very end, because "the finished field" is what every
    // one of these buttons operates on in the reference. Entirely skipped
    // when every toggle is off -- which is the default, so a default world is
    // bit-identical to one generated before this block existed.
    if p.passes.any() {
        let q = &p.passes;
        if q.velocity {
            // veloParams() (reference HTML line 3995), verbatim -- including
            // its own `Math.max(10,Math.min(160,...))` on the iteration count.
            let vp = VelocityParams {
                iters: q.velo_iters.clamp(10, 160),
                dt: 0.02,
                gravity: 9.8 * p.planet.g,
                rain_rate: 0.012,
                evap: 0.05,
                capacity: 0.5 + 1.5 * q.velo_strength,
                erode_k: 0.05 + 0.5 * q.velo_strength,
                deposit_k: 0.25,
                min_slope: 0.001,
                centrifugal_k: 1.4 * q.velo_meander,
                sea: sea_level,
            };
            // The returned water/vx/vy are the reference's Velocity debug
            // view and Pillar-3 flow-map, neither of which this port has a
            // consumer for -- dropped rather than carried on `WorldState`.
            // `veloFinish()` is deliberately *not* an isostatic rebound: the
            // reference's own comment says so ("it's a full hydraulic sim").
            let _ = velocity_erode_kernel(&mut field, Some(&rainfall), gw, gh, &vp);
        }
        if q.glacial {
            let pre = field.clone();
            glacial_kernel(
                &mut field,
                &temperature,
                gw,
                gh,
                &GlacialParams {
                    kg: q.glacial_kg,
                    mg: q.glacial_mg,
                    snowline: q.glacial_snowline,
                    u_factor: q.glacial_u_factor,
                    passes: q.glacial_passes,
                    g: p.planet.g,
                    sea: sea_level,
                    world,
                },
            );
            // glacialErode()'s own `eroFinish(pre)` tail -- glacial is the one
            // of these the reference follows with an isostatic rebound.
            isostatic_rebound(&mut field, &pre, gw, gh, p.tect.blur_r, world);
            if p.tect.dynamic_lithology {
                recompute_resistance_after_erosion(&mut resistance_field, &pre, &field, 6.0);
            }
        }
        if q.coastal {
            coastal_process(
                &mut field,
                &flow_discharge,
                gw,
                gh,
                sea_level,
                world,
                p.planet.g,
                &CoastalParams {
                    wave_str: q.wave_str,
                    estuary_depth: q.estuary_depth,
                    marsh_band: q.marsh_band,
                    passes: q.coastal_passes,
                },
            );
        }
        if q.hillslope {
            hillslope_diffuse(&mut field, gw, gh, q.diffuse_passes, q.diffuse_d, world);
        }
        // ---- evolveCoupled(cycles) (reference HTML lines 4270-4279) ----
        // Pure orchestration in the reference too; the per-cycle
        // `refreshClimate()` is the whole point -- the rain driving the next
        // cycle's incision reflects the orography the last one built.
        for _ in 0..q.evolve_cycles.max(0) {
            let pre = field.clone();
            let sp = StreamPowerParams {
                k: p.stream.k,
                uplift: p.stream.uplift,
                deposit: p.stream.deposit,
                climate_k: p.stream.climate_k,
                // evolveCoupled's own `Math.max(4,Math.round(iters*0.6))`.
                iters: (js_round(p.stream.iters as f64 * 0.6) as i32).max(4),
                resist: p.tect.resist,
                g: p.planet.g,
                world,
                sea: sea_level,
            };
            stream_power_kernel(&mut field, &stress.stress_field, &resistance_field, &rainfall, gw, gh, &sp);
            isostatic_rebound(&mut field, &pre, gw, gh, p.tect.blur_r, world);
            if p.tect.dynamic_lithology {
                recompute_resistance_after_erosion(&mut resistance_field, &pre, &field, 6.0);
            }
            refresh_climate(
                p,
                sea_level,
                &field,
                &climate_params,
                &weather_params,
                &mut temperature,
                &mut rainfall,
                &mut flow_discharge,
            );
        }
        // ---- depositSediment() (reference HTML lines 4310-4320) ----
        if q.sediment_fill {
            let pre = field.clone();
            let sp = StreamPowerParams {
                k: p.stream.k,
                uplift: p.stream.uplift,
                deposit: p.stream.deposit,
                climate_k: p.stream.climate_k,
                iters: p.stream.iters,
                resist: p.tect.resist,
                g: p.planet.g,
                world,
                sea: sea_level,
            };
            stream_power_kernel(&mut field, &stress.stress_field, &resistance_field, &rainfall, gw, gh, &sp);
            // the eroded column *is* the sediment supply
            let mut supply = vec![0f32; gw * gh];
            for i in 0..gw * gh {
                let d = (pre[i] as f64 - field[i] as f64) as f32;
                if d > 0.0 {
                    supply[i] = d;
                }
            }
            // discharge on the carved surface, before routing
            flow_discharge = compute_flow(gw, gh, &field, Some(&rainfall), true, world);
            route_sediment(&mut field, &flow_discharge, &supply, gw, gh, sea_level, q.sediment_capacity, world);
        }
        // ---- applyTidalSedimentation() (reference HTML lines 4324-4334) ----
        // Last, as it is in the reference's own source order (immediately
        // after `depositSediment`), and because mudflats accrete onto the
        // coastline the passes above finished shaping.
        //
        // The reference's own `if(!tideField) return;` gate is satisfied here
        // by *building* the field: `refreshTides()` recomputes it from the
        // live surface, so reading it off the just-finished `field` is the
        // reference's own ordering, not a shortcut. No geoid (`None`) --
        // `PlanetParams` carries none, the same reasoning `compute_temperature`
        // and `simulate_weather` already give for their own `None`.
        if q.tidal_flats {
            let tide = compute_tide_field(
                gw,
                gh,
                &field,
                None,
                sea_level,
                &TideParams { g: p.planet.g, ..TideParams::default() },
            );
            apply_tidal_sedimentation(&mut field, &tide, sea_level, gw, gh, q.tidal_k);
        }
        // `erodeFinish`'s own clamp statement (reference HTML line 3894),
        // borrowed from the droplet op's tail. **A deliberate deviation,
        // disclosed** (`CLAUDE.md`'s no-silent-deviation rule): the reference
        // runs this clamp only after the droplet pass, and `veloFinish` /
        // `routeSediment` genuinely can leave a cell outside 0..1 -- velocity
        // erosion carries only a +-1e9 finite guard, and sediment routing adds
        // without an upper bound. In the reference that is a transient a user
        // sees and re-runs past; here it would be baked into a `WorldState`
        // whose 0..1 field range every downstream stage and the renderer both
        // assume (and `generate_terrain`'s own end-to-end test asserts).
        // Applied once for the sequence, after the last pass, so no pass reads
        // a clamped value the reference would have left unclamped.
        // Not `f32::clamp`: this is the reference's own two-statement
        // `if(f<0)f=0; else if(f>1)f=1;`, transcribed -- the same reason
        // `cartalith_erosion::passes` gives for its own copy of this shape.
        #[allow(clippy::manual_clamp)]
        for v in field.iter_mut() {
            if *v < 0.0 {
                *v = 0.0;
            } else if *v > 1.0 {
                *v = 1.0;
            }
        }

        // `computeFlow(true); refreshClimate();` -- the tail every one of
        // these ops has in the reference, paid once for the whole sequence
        // rather than once per pass. Composing several of them in one run is
        // this port's own addition (the reference never does), so there is no
        // reference behaviour to differ from here -- see `ErosionPassParams`.
        refresh_climate(
            p,
            sea_level,
            &field,
            &climate_params,
            &weather_params,
            &mut temperature,
            &mut rainfall,
            &mut flow_discharge,
        );
    }

    // PR-04: record each active device's real allocation total *before* the
    // set is dropped, so Preferences ▸ Devices can show a measured number
    // without paying its own ~1.3 s adapter/device handshake to ask.
    if let Some(set) = gpu_set.as_ref() {
        cartalith_gpu::record_usage(set);
    }

    // Ecology & biomes (8) / Resources & soils (9): no code in this function
    // at all -- see `progress.rs`'s own doc comment for what actually
    // computes biome/soil/resource fields (`cartalith-godot::
    // compute_civilisation`, outside this function and outside the WORLD
    // domain's ten-stage pipeline this counter represents). Both tick
    // through together, honestly reporting "no engine work here" rather
    // than lingering as if real computation were happening.
    crate::progress::advance(crate::progress::ECOLOGY_BIOMES);
    crate::progress::advance(crate::progress::RESOURCES_SOILS);
    crate::progress::finish();

    WorldState {
        sea_level,
        field,
        plate_id,
        boundary_mask: stress.boundary_mask,
        stress_field: stress.stress_field,
        age_field,
        resistance_field,
        crust_field: base_raw,
        boundary_type: stress.boundary_type,
        shear_field: stress.shear_field,
        volcanic_field,
        impact_field,
        temperature,
        rainfall,
        flow_discharge,
        channels,
        stream_order,
        river_mask,
        river_floor,
        gpu_stages_used,
    }
}

/// `computeFlow(true); refreshClimate();` (reference HTML line 5154) — the
/// tail every terrain-changing op in the reference runs: re-derive discharge
/// on the new surface, then temperature, rainfall, the moisture correctors
/// and (when enabled) ocean currents over it.
///
/// `GUI_GAP_REGISTER.md` MS-04 named this as the one genuinely missing engine
/// function: `generate_terrain` used to sequence
/// `compute_temperature`/`simulate_weather` inline, so nothing could re-derive
/// climate over a surface that changed afterwards. It is `pub` because that
/// is the point — any future post-generation op needs exactly this.
///
/// **Order matters and is the reference's**: discharge is computed from the
/// *old* rainfall (that is what `computeFlow(true)` reads), and the moisture
/// correctors then read the *new* discharge. `computeSeasons()` stays
/// deferred, as it is in `generate_terrain` itself.
///
/// CPU only. `p.use_gpu` selects GPU paths inside `generate_terrain` where a
/// device handle is in scope; per `HARDWARE_ACCELERATION.md` §27 a CPU path
/// is always a valid outcome for any stage, and `WorldState.gpu_stages_used`
/// reports what actually ran rather than what was asked for.
#[allow(clippy::too_many_arguments)]
/// The [`ClimateParams`] `generate_terrain` builds, as a function so any
/// other caller of [`refresh_climate`] gets *the same* struct rather than a
/// hand-copied second literal that can drift field-by-field. A pure
/// re-projection of `p` — no arithmetic, so nothing here can move a golden
/// value.
///
/// `sea_level` is separate because it is not `p.sea_level`: a World-Structure
/// archetype re-anchors it during generation, and `WorldState::sea_level`
/// carries the value actually used.
pub fn climate_params_for(p: &WorldParams, sea_level: f64) -> ClimateParams {
    ClimateParams {
        world: p.world,
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
    }
}

/// [`climate_params_for`]'s sibling for [`WeatherParams`], and for the same
/// reason.
pub fn weather_params_for(p: &WorldParams, sea_level: f64) -> WeatherParams {
    WeatherParams {
        world: p.world,
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
        terrain_wind_deflection: p.climate.terrain_wind_deflection,
        currents: p.climate.currents,
        current_k: p.climate.current_k,
    }
}

pub fn refresh_climate(
    p: &WorldParams,
    sea_level: f64,
    field: &[f32],
    climate_params: &ClimateParams,
    weather_params: &WeatherParams,
    temperature: &mut Vec<f32>,
    rainfall: &mut Vec<f32>,
    flow_discharge: &mut Vec<f32>,
) {
    let (gw, gh, world) = (p.gw, p.gh, p.world);
    *flow_discharge = compute_flow(gw, gh, field, Some(rainfall), true, world);
    *temperature = compute_temperature(gw, gh, field, None, climate_params);
    *rainfall = simulate_weather(gw, gh, field, p.climate.w_iters, 0.0, weather_params);
    apply_climate_moisture_correctors(
        gw,
        gh,
        field,
        flow_discharge,
        rainfall,
        sea_level,
        world,
        p.climate.lat_n,
        p.climate.lat_s,
        p.climate.zonal_k,
    );
    if p.climate.currents {
        apply_ocean_currents(
            gw,
            gh,
            field,
            temperature,
            rainfall,
            sea_level,
            world,
            p.climate.lat_n,
            p.climate.lat_s,
            p.climate.equator_temp,
            p.climate.pole_temp,
            p.planet.axial_tilt_deg,
            p.planet.rotation_hours,
            p.climate.wind_manual,
            p.climate.wind_dir_deg,
            p.climate.press_k,
            p.climate.current_k,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `ErosionPassParams`' whole contract: **off is bit-identical**. Not a
    /// tolerance — `assert_eq!` on the raw `f32`s, plus temperature, rainfall
    /// and discharge, because `refresh_climate` must not run either.
    #[test]
    fn erosion_passes_off_leave_generation_bit_identical() {
        let mut p = WorldParams::defaults(24, 18, 4242);
        let base = generate_terrain(&p);
        // Every knob moved, every toggle still off: a knob alone must do
        // nothing at all, or "default-off" is only half true.
        p.passes.velo_strength = 0.9;
        p.passes.glacial_kg = 0.8;
        p.passes.wave_str = 1.0;
        p.passes.diffuse_d = 0.2;
        p.passes.sediment_capacity = 1.0;
        p.passes.tidal_k = 1.0;
        assert!(!p.passes.any());
        let same = generate_terrain(&p);
        assert_eq!(base.field, same.field);
        assert_eq!(base.temperature, same.temperature);
        assert_eq!(base.rainfall, same.rainfall);
        assert_eq!(base.flow_discharge, same.flow_discharge);
    }

    /// `DECISIONS.md` §7f's proof obligation: skipping the pre-carve
    /// `compute_flow` must be a **pure performance change**. Not a
    /// tolerance — `assert_eq!` on the raw `f32`s of every field
    /// `generate_terrain` returns, plus `gpu_stages_used`, against the same
    /// generation run with the reference's own literal call order restored.
    ///
    /// Several seeds and both `world` modes, because the carve block's
    /// reads are what the claim rests on and a wrapped world takes
    /// different branches through `compute_flow`, `build_channels` and
    /// `enforce_channel_descent`. The `carve_rivers = false` case is here
    /// too: there the call is **not** dead, so `force_precarve_flow` must
    /// make no difference for the opposite reason, and a skip that leaked
    /// into that path would show up as an empty `flow_discharge`.
    #[test]
    fn precarve_flow_skip_leaves_generation_bit_identical() {
        for &(gw, gh, seed, world, carve) in &[
            (24usize, 18usize, 4242i32, false, true),
            (24, 18, 4242, true, true),
            (31, 17, 991, false, true),
            (20, 20, 7, true, true),
            (24, 18, 4242, false, false),
            (20, 20, 7, true, false),
        ] {
            let mut p = WorldParams::defaults(gw, gh, seed);
            p.world = world;
            p.carve_rivers = carve;
            let skipped = generate_terrain_inner(&p, false);
            let faithful = generate_terrain_inner(&p, true);
            let label = format!("{gw}x{gh} seed={seed} world={world} carve={carve}");
            assert_eq!(skipped.field, faithful.field, "field ({label})");
            assert_eq!(skipped.temperature, faithful.temperature, "temperature ({label})");
            assert_eq!(skipped.rainfall, faithful.rainfall, "rainfall ({label})");
            assert_eq!(skipped.flow_discharge, faithful.flow_discharge, "flow_discharge ({label})");
            assert_eq!(skipped.river_mask, faithful.river_mask, "river_mask ({label})");
            assert_eq!(skipped.river_floor, faithful.river_floor, "river_floor ({label})");
            assert_eq!(skipped.stream_order, faithful.stream_order, "stream_order ({label})");
            assert_eq!(skipped.resistance_field, faithful.resistance_field, "resistance_field ({label})");
            assert_eq!(skipped.gpu_stages_used, faithful.gpu_stages_used, "gpu_stages_used ({label})");
            assert_eq!(
                skipped.flow_discharge.len(),
                gw * gh,
                "the skip must never leave flow_discharge empty ({label})"
            );
        }
    }

    /// Each pass, alone, must actually move the surface — the check that the
    /// wiring reaches the kernel rather than merely compiling next to it.
    /// Fixtures are shaped to *reach* each one: glacial needs cells above the
    /// snowline **and** below freezing, so its case drops the snowline and
    /// runs a cold world.
    #[test]
    fn each_erosion_pass_changes_the_field_on_its_own() {
        let base_p = WorldParams::defaults(48, 36, 991);
        let base = generate_terrain(&base_p);

        /// One row of the table below: the pass's name, and what turning it
        /// on (plus whatever the fixture needs to *reach* it) looks like.
        type Case = (&'static str, fn(&mut WorldParams));

        let cases: [Case; 7] = [
            ("velocity", |p| {
                p.passes.velocity = true;
                p.passes.velo_iters = 20;
            }),
            ("glacial", |p| {
                p.passes.glacial = true;
                // reach it: an ice-age world with a low snowline
                p.passes.glacial_snowline = 0.05;
                p.climate.equator_temp = -20.0;
                p.climate.pole_temp = -60.0;
            }),
            ("coastal", |p| p.passes.coastal = true),
            ("hillslope", |p| p.passes.hillslope = true),
            ("sediment_fill", |p| p.passes.sediment_fill = true),
            ("evolve", |p| p.passes.evolve_cycles = 2),
            ("tidal_flats", |p| p.passes.tidal_flats = true),
        ];

        for (name, apply) in cases {
            let mut p = base_p.clone();
            apply(&mut p);
            let ws = generate_terrain(&p);
            // The glacial case also changes the climate inputs, so compare it
            // against its own climate-only twin rather than the plain base.
            let reference = if name == "glacial" {
                let mut q = p.clone();
                q.passes.glacial = false;
                generate_terrain(&q).field
            } else {
                base.field.clone()
            };
            let moved = ws.field.iter().zip(&reference).filter(|(a, b)| a != b).count();
            assert!(moved > 0, "{name}: the pass ran but nothing moved");
            assert!(ws.field.iter().all(|v| v.is_finite()), "{name}: produced a non-finite height");
            assert!(
                ws.field.iter().all(|&v| (0.0..=1.0).contains(&v)),
                "{name}: left the field outside 0..1"
            );
        }
    }

    /// The tidal-flats pass's own shape, which "something moved" cannot see:
    /// it is *accretion only* — every changed cell was submerged, every change
    /// is upward, and none of them is pushed past sea level (the kernel's own
    /// `sea - 1e-4` ceiling). A sign error or a swapped `sea`/`depth` would
    /// still move cells and still pass the table above.
    #[test]
    fn the_tidal_flats_pass_only_raises_submerged_cells_toward_sea_level() {
        let base_p = WorldParams::defaults(48, 36, 991);
        let base = generate_terrain(&base_p);
        let mut p = base_p.clone();
        p.passes.tidal_flats = true;
        let ws = generate_terrain(&p);

        let sea = base.sea_level;
        assert_eq!(ws.sea_level, sea, "the pass must not move sea level");
        let mut moved = 0usize;
        for (i, (&after, &before)) in ws.field.iter().zip(&base.field).enumerate() {
            if after == before {
                continue;
            }
            moved += 1;
            assert!(after > before, "cell {i}: tidal sedimentation deposits, never erodes");
            assert!(
                (before as f64) < sea,
                "cell {i}: a cell already at or above sea level must be untouched"
            );
            assert!(
                (after as f64) <= sea,
                "cell {i}: accretion must stop at sea level, not build land"
            );
        }
        assert!(moved > 0, "no mudflat accreted — the tide field never reached the kernel");
    }

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

    /// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 6: the GPU path (when
    /// available on this machine) is internally deterministic and produces
    /// statistically sane terrain -- NOT checked against the CPU/JS
    /// reference (`DECISIONS.md` §7c: different noise, different world, by
    /// design). Environment-tolerant: if this machine has no usable GPU,
    /// every stage falls back to CPU and `gpu_stages_used` is empty --
    /// still a valid, asserted-on outcome, not a test failure.
    #[test]
    fn generate_terrain_gpu_path_is_deterministic_and_valid() {
        let mut p = WorldParams::defaults(24, 18, 777);
        p.use_gpu = true;
        let a = generate_terrain(&p);
        let b = generate_terrain(&p);

        // Determinism: same seed, same use_gpu path, same result -- twice.
        //
        // Held to `DECISIONS.md` §7a's bar for GPU paths -- *principled
        // equivalence*, not bit-identity -- on the owner's decision of
        // 2026-08-25. This was an `assert_eq!` over the whole field, and it
        // failed **intermittently**: roughly 2 of 6 full-workspace runs and 0
        // of 6 in isolation, two runs of one seed differing by about one ulp
        // of `f32`. That is the GPU scheduling its own reductions in a
        // different order between dispatches, which §7a already says this
        // project does not chase; the assertion simply predated the rule.
        //
        // 1e-6 on a field normalised to [0,1] is about eight ulps -- tight
        // enough that genuine non-determinism (a different seed reaching the
        // noise, a stage silently falling back on one run and not the other)
        // still fails, and the worst deviation is reported so a regression
        // says how far it drifted rather than merely that it did.
        const GPU_DETERMINISM_TOL: f32 = 1e-6;
        assert_eq!(
            a.field.len(),
            b.field.len(),
            "GPU-path generation must produce the same field length for a fixed seed"
        );
        let worst = a
            .field
            .iter()
            .zip(b.field.iter())
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max);
        assert!(
            worst <= GPU_DETERMINISM_TOL,
            "GPU-path generation must be deterministic for a fixed seed: \
             worst element deviation {worst:e} exceeds {GPU_DETERMINISM_TOL:e}"
        );
        assert_eq!(a.gpu_stages_used, b.gpu_stages_used, "which stages ran on GPU must itself be deterministic");

        // Statistical sanity: real terrain, not garbage, whichever path
        // (GPU or CPU-fallback) actually produced it.
        let n = 24 * 18;
        assert_eq!(a.field.len(), n);
        assert!(a.field.iter().all(|&v| v.is_finite()), "no NaN/Inf in a GPU-path height field");
        assert!(a.field.iter().all(|&v| (0.0..=1.0).contains(&v)), "GPU-path height field still normalized to [0,1]");
        let mn = a.field.iter().cloned().fold(f32::INFINITY, f32::min);
        let mx = a.field.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        assert!(mx - mn > 0.05, "GPU-path terrain shouldn't be a degenerate flat field");

        // Every reported stage name is one this milestone actually wired.
        // Grows by one name per newly-wired stage (milestone 7 added
        // "weather", milestone 9 adds "flow") -- an allow-list that must
        // track reality, not a weakened assertion.
        let known =
            ["warp", "warp_split", "heterogeneity", "plate_assignment", "base_field_blur", "weather", "flow"];
        for s in &a.gpu_stages_used {
            assert!(known.contains(&s.as_str()), "unexpected gpu_stages_used entry: {s}");
        }
    }

    /// The structural requirement: `use_gpu=true`/`false` must never change
    /// which fields exist or their shapes -- only, potentially, the actual
    /// substrate values (per §7c). A crash or a length mismatch here would
    /// mean the GPU path broke `WorldState`'s own contract with every
    /// downstream consumer (climate, erosion, hydrology, every Phase 2
    /// field).
    #[test]
    fn generate_terrain_gpu_and_cpu_paths_share_worldstate_shape() {
        let mut p_gpu = WorldParams::defaults(20, 16, 42);
        p_gpu.use_gpu = true;
        let p_cpu = WorldParams::defaults(20, 16, 42);

        let a = generate_terrain(&p_gpu);
        let b = generate_terrain(&p_cpu);

        assert_eq!(a.field.len(), b.field.len());
        // `heterogeneity_field`/`flexure_field` were checked here until R2
        // stopped retaining them; `field` is downstream of both (they are
        // two of `compute_height`'s inputs), so a GPU path that produced a
        // wrongly-shaped one still cannot pass this.
        assert_eq!(a.plate_id.len(), b.plate_id.len());
        assert!(b.gpu_stages_used.is_empty(), "CPU path (use_gpu=false) must never report GPU stages used");
    }

    /// GPU_LAYER_INTEGRATION_SCOPE.md milestone 6's own required
    /// measurement: end-to-end `use_gpu=true` vs `use_gpu=false`
    /// `generate_terrain` at the four established sizes -- not isolated
    /// kernel dispatch time (already measured per-kernel in milestones
    /// 2/4/5), but the real cost including a *fresh `GpuContext` per stage,
    /// per call* (see the doc comment on `warp_grid_gpu` et al. in
    /// `cartalith-gpu` for why that's an accepted tradeoff for one-shot
    /// batch generation). `--nocapture` to see the numbers; `#[ignore]`d
    /// since it's a timing report, not a correctness check, and full
    /// 2048x2048 CPU pipeline runs are slow enough to not want in the
    /// default `cargo test` loop.
    #[test]
    #[ignore]
    fn measured_generate_terrain_gpu_vs_cpu_timing() {
        for &sz in &[128usize, 512, 1024, 2048] {
            let mut p_gpu = WorldParams::defaults(sz, sz, 24601);
            p_gpu.use_gpu = true;
            let p_cpu = WorldParams::defaults(sz, sz, 24601);

            let t0 = std::time::Instant::now();
            let ws_gpu = generate_terrain(&p_gpu);
            let gpu_time = t0.elapsed();

            let t1 = std::time::Instant::now();
            let _ws_cpu = generate_terrain(&p_cpu);
            let cpu_time = t1.elapsed();

            eprintln!(
                "generate_terrain {sz}x{sz}: use_gpu=true = {:?} (stages actually on GPU: {:?}), use_gpu=false = {:?}, ratio (CPU/GPU) = {:.2}x",
                gpu_time,
                ws_gpu.gpu_stages_used,
                cpu_time,
                cpu_time.as_secs_f64() / gpu_time.as_secs_f64().max(1e-9)
            );
        }
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
