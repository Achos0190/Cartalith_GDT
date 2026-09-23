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
/// generation readout reads (`ANDROID_UI_SPEC.md` §1.3). Wired: every
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
/// The LOD pyramid under XYZ/TMS/WMTS addressing (`FUNCTIONAL_CONTRACT.md`
/// capabilities 6/9, §7d modernize — no reference ancestor).
pub mod slippy_export;

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

/// **EF-0** (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §5) — the elevation
/// field as a queryable, deterministic, multi-resolution primitive over a
/// world: one pyramid tile's refined elevation, or one point of it. Composes
/// `bake`'s own `pyramid_tile` and `cartalith-terrain`'s new
/// `sample_elevation` with the world's seed and sea level. Unwired: no Godot
/// bridge calls it yet, deliberately.
pub mod elevation;

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
    build_channels_routed, compute_flow, compute_flow_routed, enforce_channel_descent, river_width_scale_k,
    routing_view, strahler_from_receivers, trace_river_polylines, ChannelResult,
};
use cartalith_terrain::{
    apply_world_structure_sea_level, assign_plates, build_age_field, build_orogeny_field, build_plates,
    compute_flexure, compute_height, compute_heterogeneity, compute_resistance, compute_stress, compute_warp,
    gauss_blur, generate_continentality_field, normalize_field, smooth_orogeny, stamp_craters,
    stamp_volcanoes_provinces_shaped, stamp_volcanoes_simple_shaped, tag_boundary_types, trace_boundaries,
    HeightParams, OrogenyParams, WorldStructure,
};

// `Math.round` (ties toward `+Infinity`), from `cartalith-jsmath`.
use cartalith_jsmath::js_round;

use std::sync::Arc;

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
/// `stamp_volcanoes_provinces` (JS default, `true`, and this port's default
/// too since 2026-08-15) vs. `stamp_volcanoes_simple` (`false`) — see
/// `generate_terrain`'s own volcanism section.
#[derive(Clone, Debug, PartialEq)]
pub struct VolcanismParams {
    pub count: i32,
    pub age: f64,
    pub provinces: bool,
    /// Drop shear-dominant (`btype::TRANSFORM`) cells from the arc and rift
    /// candidate pools. Transform margins are not a major volcanic
    /// environment — they make earthquakes and fault scarps, not
    /// stratocones — but the reference selects arcs on the *sign of the
    /// blurred normal stress*, which cannot see shear, so a measured 34% of
    /// the arc pool and 32% of the rift pool sit on transform boundaries
    /// (`cartalith-terrain/tests/volcano_transform_boundaries.rs`).
    ///
    /// **`false` here and `true` in `cartalith_godot::params::defaults`**, the
    /// same split `crater.physical_model` uses and for the same reason. Turning
    /// it on moves the height field, and therefore lithology, biomes, carrying
    /// capacity, settlements, roads and sea routes — so it ships at the app
    /// boundary (owner ruling 1, 2026-09-02, `DECISIONS.md` §7l-ii) while this
    /// function stays what ~28 golden suites mean by "the reference's
    /// baseline". With the flag on, the 34.3%/32.3% contamination above is
    /// 0.0% by construction, and both pools stay populated on every one of the
    /// twelve seeds measured.
    pub exclude_transform: bool,

    /// Build volcanoes as **shaped edifices** — a shield, a stratocone or a
    /// scoria cone chosen by the province's own volcanic setting — instead of
    /// the reference's one power-law cone at every scale, and give each one a
    /// summit that has genuinely *collapsed* rather than a crater subtracted
    /// from its peak.
    ///
    /// The reference has no edifice-type distinction at all: `stampOneVolcano`
    /// stamps `H*(1-t)^(1.6-age*0.8)` for a 200 m scoria cone and a 7 000 m
    /// shield alike, and its summit dip (`add -= H*0.5*(1-t/0.16)`) bottoms out
    /// at a single point with no floor and no ring-fault wall. The
    /// arc/rift/hotspot classification that would say *which* edifice to build
    /// already exists one frame up, in `placeProvinceVolcanoes`, and is
    /// discarded there. See `cartalith_terrain::EdificeModel`.
    ///
    /// **`false` here and `true` in `cartalith_godot::params::defaults`.**
    /// Changing an edifice's shape moves the height field, and therefore
    /// lithology, biomes, carrying capacity, settlements, roads and sea routes;
    /// §7l's authorisation was **for craters**, so this needed its own — which
    /// it has, as owner ruling 1 of 2026-09-02 (`DECISIONS.md` §7l-ii). It
    /// ships at the app boundary and this function stays the parity baseline.
    pub edifice_model: bool,
}

/// `state.crater` (reference HTML line 2267).
#[derive(Clone, Debug, PartialEq)]
pub struct CraterParams {
    pub count: i32,
    pub age: f64,
    /// Generate craters from an area density and a size-frequency law instead
    /// of an absolute count.
    ///
    /// **Defaults to `true`. This diverges from the reference deliberately**,
    /// on the owner's ruling of 2026-09-02 — see `DECISIONS.md` §7l. The
    /// reference stamps exactly `count` craters whatever the map represents, so
    /// the same slider is a negligible density on a 40 000 km world and an
    /// unrenderably dense one on a 5 km region; the two differ in area by
    /// 64 000 000x. Setting this `false` restores the reference's own path byte
    /// for byte, and the import/inversion path keeps using it.
    ///
    /// See `cartalith_terrain::crater_lambda`.
    pub physical_model: bool,

    /// **Geological** surface exposure age, in millions of years — how long
    /// this surface has been accumulating impacts.
    ///
    /// Not the civilisation Timeline, which runs in years to millennia and is
    /// six orders of magnitude away; and not [`Self::age`], which is a
    /// morphological 0-1 term for how worn each crater looks. Three different
    /// quantities. See `cartalith_terrain::CRATER_SURFACE_AGE_MYR`.
    pub surface_age_myr: f64,
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
    ///
    /// **Not only an erosion-pass knob.** Since owner ruling 2 of 2026-09-02
    /// (`DECISIONS.md` §7l-ii) this is the world's hillslope diffusivity, and
    /// `cartalith_terrain::crater_degradation_tau` reads it: under
    /// `CraterParams::physical_model` — on at the app boundary — raising it
    /// relaxes craters further at the same surface age, **whether or not
    /// [`Self::hillslope`] is enabled**. So this is the one member of this
    /// struct whose value alone changes generated terrain with every pass off,
    /// and the exception to `erosion_passes_off_leave_generation_bit_identical`
    /// is deliberate. Pinned by
    /// `the_erosion_diffusivity_reaches_craters_only_under_the_physical_model`.
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

/// The civilisation layer's own dials.
///
/// **`generate_terrain` reads none of them.** This struct is carried on
/// [`WorldParams`] rather than consumed here, and its one reader is
/// `cartalith-godot`'s `compute_civilisation` — the civ layer is built above
/// this crate (`ARCHITECTURE.md`: `cartalith-civ` is stateless, the
/// orchestration lives in the Godot bridge), so there is no `cartalith-engine`
/// stage to read it. It lives here anyway because `cartalith-godot`'s
/// `params::PARAMS` table is keyed on `WorldParams` field paths, and the
/// alternative — a second parameter table beside it — is exactly the
/// duplication that table exists to prevent. Adding a field here costs
/// generation nothing: no stage branches on it.
///
/// The first four are the reference's four civ-layer module flags
/// (`_civVillages`, `_civMetropolis`, `_civRecoveryPhase`, `_biomeK`,
/// reference HTML lines 6441-6444), each of which the reference's own comment
/// marks a *transient UI preference* that `serializeState` does not write —
/// hence their empty `JS_PATHS` rows. Every default below reproduces the
/// reference's own, so a default civ layer is bit-identical to before this
/// struct existed.
#[derive(Clone, Debug, PartialEq)]
pub struct CivParams {
    /// `_civVillages` (reference 6444), default OFF — the additive
    /// village-seeding pass.
    pub villages: bool,
    /// `_civMetropolis` (reference 6442), default OFF — "OFF by default =>
    /// auto-populate output bit-identical", the reference's own comment.
    pub metropolis: bool,
    /// `_civRecoveryPhase` (reference 6443), `0` Stable / `1` Survival /
    /// `2` Subsistence / `3` Regional / `4` Mature. `0` is a strict no-op.
    pub recovery_phase: i32,
    /// `_biomeK` (reference 6441), default OFF — the biome carrying-capacity
    /// residual. The reference's own comment on that default: "0 = biome
    /// carrying-capacity residual OFF (bit-identical)".
    pub biome_k: bool,
    /// How many factions settlement placement assigns into — the reference's
    /// `CIV_FACTIONS` length (its literal, reference 14568, is 6). No
    /// reference *control*: the reference edits that array through
    /// `_civAddFaction`/`_civRemoveFaction` rather than a count dial.
    pub factions: i32,
    /// `SETTLE_SEED_THRESH` (reference 6415, `const SETTLE_SEED_THRESH=0.42`)
    /// — the suitability a cell must reach to seed a settlement at all. A
    /// reference constant with no control; raising it thins the world out,
    /// lowering it settles marginal land.
    pub seed_thresh: f64,
    /// The divisor in the seed-suppression radius, `max(6, floor(gw / d))` —
    /// reference 25360's `Math.max(6,(GW/22)|0)`, so the default is `22`.
    /// Larger means a smaller radius, so settlements pack closer together.
    /// The `6`-cell floor is not a dial: it is what stops a small grid from
    /// suppressing nothing at all.
    pub seed_suppress_div: f64,
    /// The reference's Auto-populate count inputs (`civNCap`/`civNCity`/
    /// `civNTown`/`civNVil`/`civNHam`, v2.11 lines 1393-1399), switched on as
    /// a set. The reference tells "blank" from `0` per field, but only one
    /// distinction survives into generation: either every field is blank
    /// (automatic placement) or at least one is set and every blank one
    /// counts as `0` (`c.capital||0`). This flag is that distinction, so no
    /// count ever needs a "no value" sentinel. See [`CivParams::want_counts`].
    pub fixed_counts: bool,
    /// `[capital, city, town, village, hamlet]`, read only while
    /// `fixed_counts` is on.
    pub counts: [i32; 5],
}

impl CivParams {
    /// The reference's `wantCounts`: `Some` only when fixed counts are on
    /// AND they ask for at least one settlement. The reference refuses an
    /// all-zero request with an alert and places nothing; a generation pass
    /// here has nobody to alert, so an all-zero request falls back to
    /// automatic placement instead -- the shell says so beside the fields.
    pub fn want_counts(&self) -> Option<[usize; 5]> {
        let w = self.counts.map(|n| n.max(0) as usize);
        (self.fixed_counts && w.iter().sum::<usize>() >= 1).then_some(w)
    }
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
    /// Integrated drainage (`RC_ENGINE_CHANGES.md` §6g/§6k; the source's
    /// `state.hydro.integrate`): route every flow accumulation and the
    /// channel network's receiver tree over
    /// [`cartalith_hydrology::build_routing_surface`] — the depression-filled,
    /// ε-tilted surface — instead of the raw field, so a local pit no longer
    /// terminates the water that reaches it. The heightmap itself keeps its
    /// pits; only routing sees them filled.
    ///
    /// **`false` here and `true` in the shipped app** (`cartalith-godot`'s
    /// `params::defaults`), the same split as `crater.physical_model`: this
    /// function is the goldens' parity baseline and the v2.10/v2.11 reference
    /// they were captured from has no fill. The source itself shipped the fill
    /// off (v2.41) and then on (v2.59) while its loader kept defaulting a save
    /// without the key to off — a world generated without integration must
    /// reload as the world it was.
    pub integrate_drainage: bool,
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
    /// The civ layer's dials. Read by `cartalith-godot`'s
    /// `compute_civilisation`, by no stage in this crate — see [`CivParams`].
    pub civ: CivParams,
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
            // Off: the reference this baseline reproduces has no fill. On at
            // the app boundary -- see the field's own doc comment.
            integrate_drainage: false,
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
            // `exclude_transform: false` is the parity baseline: the
            // reference puts volcanoes on transform margins and the goldens
            // pin that. See `VolcanismParams::exclude_transform`.
            // `edifice_model: false` for the same reason: the goldens pin the
            // reference's single cone shape. See `VolcanismParams::edifice_model`.
            volc: VolcanismParams {
                count: 20,
                age: 0.40,
                provinces: true,
                exclude_transform: false,
                edifice_model: false,
            },
            // `physical_model: false` HERE and `true` in the shipped app, on
            // purpose. `WorldParams::defaults` is what ~28 golden suites mean
            // by "the reference's baseline" -- 16 of them in `cartalith-civ`
            // alone run `generate_terrain(&defaults())` and compare the civ
            // layer against fixtures captured from the reference under Node.
            // Flipping this here changes the height field, so lithology,
            // biomes, carrying capacity, settlement placement, roads and sea
            // routes all move, and every one of those goldens stops being a
            // parity test. The owner's §7l ruling was to change the *shipped
            // generation*, not to delete the parity baseline; so the divergence
            // lives at the app boundary (`params.rs`'s `crater.physical_model`,
            // default on) and this stays the reference's own behaviour.
            crater: CraterParams {
                count: 100,
                age: 0.50,
                physical_model: false,
                surface_age_myr: cartalith_terrain::CRATER_SURFACE_AGE_MYR,
            },
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
            // Every one of these is the reference's own default (see
            // `CivParams`), so a civ layer built from `defaults()` is
            // bit-identical to the one built before this struct existed.
            civ: CivParams {
                villages: false,
                metropolis: false,
                recovery_phase: 0,
                biome_k: false,
                factions: 6,
                seed_thresh: 0.42,
                seed_suppress_div: 22.0,
                fixed_counts: false,
                counts: [0; 5],
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
    /// `Arc`, not a plain `Vec` — and so are `temperature`, `rainfall` and
    /// `flow_discharge` below. These four are the grids a background LOD
    /// worker needs a whole copy of (`cartalith_godot::lod_worker::
    /// SnapshotInputs`), and LOD-D6 reported the per-snapshot clone of them
    /// at **43.0 MB** at 2 048 × 1 311 — 10.74 MB each — against its own
    /// 60 MiB budget. A worker cannot borrow from the world, so the only way
    /// not to copy the bytes is for the world to hold them behind a refcount.
    /// Making the difference real was measured, at that same 2 684 928 cells,
    /// as a host-polled peak working set: **89 862 144 B** for four deep
    /// clones against **46 886 912 B** for four `Arc` clones, 5 runs each,
    /// spread under 0.01 MB.
    ///
    /// **Reads are unchanged**: `Arc<Vec<f32>>` derefs to `Vec<f32>` derefs
    /// to `[f32]`, so `&ws.field`, `ws.field[i]` and `ws.field.len()` all
    /// compile and mean exactly what they did. The four crates that only read
    /// these grids (`cartalith-civ`, `-hydrology`, `-terrain`, `-spatial`)
    /// needed no change at all.
    ///
    /// **Writes go through `Arc::make_mut`**, which clones only when a second
    /// holder exists — i.e. only when a sculpt, erode, undo or re-centre
    /// lands while an LOD snapshot is still alive. That case costs one copy
    /// of one grid, which is what the old code paid on every snapshot for all
    /// four. The mutating call sites are `erode_op`, `center_landmasses`,
    /// `recompute_stale` and `cartalith-godot`'s sculpt-commit and undo/redo.
    ///
    /// `Arc<[f32]>` was the other candidate and was rejected on two counts:
    /// it has no `make_mut`, so every one of those call sites would need
    /// interior mutability or a rebuild-and-replace; and it cannot be made
    /// from an existing `Vec<f32>` without copying it, so `generate_terrain`
    /// would pay a full copy of each grid to hand it over.
    pub field: Arc<Vec<f32>>,
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
    /// **The second `Arc` group**, with `resistance_field`, `crust_field` and
    /// `volcanic_field` below — the four tectonic-substrate grids
    /// `cartalith_godot::lod_worker::LithoSource` needs, for the one call
    /// `LodSnapshot::build` makes to `cartalith_civ::build_lithology`. They
    /// were plain `Vec`s and were deep-cloned into every LOD snapshot, which
    /// [`WorldState::field`]'s own doc flagged as the next ~43 MB of the same
    /// shape it had just removed: four grids of 2 684 928 cells at 2 048 ×
    /// 1 311 is 10.74 MB each. Same reasoning, same `Arc<Vec<f32>>`, same
    /// rejection of `Arc<[f32]>` (no `make_mut`; `Arc::from(vec)` copies).
    ///
    /// **Transient rather than retained, and that is the only difference**
    /// from the four grids above: `LodSnapshot::build` drops the
    /// `LithoSource` as soon as the lithology is built, so this never showed
    /// up in `LodSnapshot::retained_bytes` — it was a peak, not a residency.
    /// The peak is what a small device runs out of.
    ///
    /// Reads are unchanged (`Deref`), and the one mutating site is
    /// `center_landmasses`, which goes through `Arc::make_mut`.
    pub age_field: Arc<Vec<f32>>,
    pub resistance_field: Arc<Vec<f32>>,
    /// `plateCrust()` (reference HTML line 3083): raw, unblurred per-cell
    /// plate base (`<0` = oceanic crust). Already computed internally as
    /// `base_raw` for orogeny/height, but not previously retained past
    /// `generate_terrain` -- added for `cartalith-civ`'s `buildLithology`
    /// port, which reads this exact same value (`currentLithology()`'s
    /// `crust` argument in the reference).
    ///
    /// `Arc` for the reason [`WorldState::age_field`] gives.
    pub crust_field: Arc<Vec<f32>>,
    /// `StressResult::boundary_type`/`shear_field` (`cartalith-terrain`):
    /// per-cell plate-boundary classification and shear magnitude. Already
    /// computed for T2+T3 orogeny (`tag_boundary_types`/`OrogenyParams::
    /// shear`) but not previously retained past `generate_terrain` --
    /// added for `cartalith-civ`'s `buildResourcePotentials` port (Phase 2
    /// milestone 5, `PHASE2_SCOPE.md`), the same `boundaryType`/
    /// `shearField` arguments the reference passes it.
    pub boundary_type: Vec<u8>,
    pub shear_field: Vec<f32>,
    /// `Arc` for the reason [`WorldState::age_field`] gives. Note this is
    /// also the one of the four that `cartalith_io::SaveFields` carries, so
    /// the save path takes an explicit `.as_ref().clone()` — a save is a
    /// serialisation, and it owns its bytes.
    pub volcanic_field: Arc<Vec<f32>>,
    pub impact_field: Vec<f32>,
    /// See [`WorldState::field`] for why these three are `Arc` too.
    pub temperature: Arc<Vec<f32>>,
    pub rainfall: Arc<Vec<f32>>,
    pub flow_discharge: Arc<Vec<f32>>,
    /// Whether `flow_discharge` and `channels` were routed over the
    /// depression-filled surface ([`WorldParams::integrate_drainage`]) — a
    /// property of *this world's* drainage, recorded so every consumer that
    /// rebuilds a receiver tree from `field` + `flow_discharge`
    /// (`cartalith_civ::fresh_river_network`) builds it over the same surface
    /// the discharge was accumulated on (`RC_ENGINE_CHANGES.md` §6g: "both
    /// trees need the same surface"). Read it from here, not from the live
    /// parameters, which may have moved since the world was made.
    pub integrated_drainage: bool,
    /// **`ChannelResult::slope` is released before this is stored** and is
    /// an empty `Vec` here — see `generate_terrain`'s own note at the point
    /// it drops it (`MEMORY_OPTIMIZATION_SCOPE.md` R2). `recv` and `chan`
    /// are the two arrays every consumer in this workspace actually reads.
    pub channels: Option<ChannelResult>,
    pub stream_order: Option<Vec<i16>>,
    pub river_mask: Option<Vec<u8>>,
    pub river_floor: Option<Vec<f32>>,
    /// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 6: which of the
    /// GPU-eligible substrate stages (`"warp"`, `"heterogeneity"`,
    /// `"plate_assignment"`, `"stress"`, `"base_field_blur"`) actually ran on GPU this
    /// generation. Empty when `p.use_gpu` was `false`, or when every stage
    /// fell back to CPU (`HARDWARE_ACCELERATION.md` §27 -- GPU failure
    /// falls back silently in terms of *correctness*, but the caller can
    /// always tell which path actually ran by reading this).
    pub gpu_stages_used: Vec<String>,
}

// -- CPU worker threads (owner ruling, `LARGE_ITEM_RULINGS.md` "CPU worker
// threads": "Call `ThreadPoolBuilder` at engine init with a count from
// settings; expose a `#[func]` to read and set it") --------------------------
//
// Rayon builds its global pool implicitly, at whatever
// `available_parallelism()` reports, the moment anything first calls
// `par_iter()`/`join()`/etc. `menus.gd`'s own "CPU worker threads" TODO named
// exactly that gap: "this port never calls ThreadPoolBuilder, so there is no
// `#[func]` to set it and no pool init to set it at." `ensure_thread_pool`
// below is that init; `generate_terrain_inner` calls it first, before any
// other work.
//
// **The global pool can be built exactly once per process**
// (`ThreadPoolBuilder::build_global`'s own doc: "this function may only be
// called once"). A rebuildable *scoped* pool installed only around
// `generate_terrain` was the alternative and was rejected: `render.rs`,
// `sample_bridge.rs` and `bake.rs` also run `par_iter()` outside
// `generate_terrain`'s own call graph, in files this lane does not own, and
// would keep reading the *un*configured global pool underneath a scoped one
// -- silently covering only part of the engine's parallel work is the exact
// "setter that silently does nothing" failure this exists to avoid, merely
// spread thinner and harder to notice. The global pool has no such gap:
// every `par_iter()` anywhere in the process reads it, by construction,
// forever after it is built.
//
// So `set_configured_thread_count` **records a preference, and applies it
// immediately only if it is the call that actually builds the pool** --
// otherwise it is inert until the next process launch, and says so. There is
// no live "rebuild" once anything has already used the pool.
// `thread_pool_active_count` is how a caller tells which state it is in,
// rather than being told a change landed when it did not.
//
// **The first version of that return value was wrong, measured false
// 2026-09-03**, and the shape of the mistake is worth keeping: it inferred
// "the pool is still unbuilt" from `ACTIVE_THREADS == 0`, but only
// `ensure_thread_pool` ever writes that counter, while *any* `par_iter()`
// anywhere in the process builds Rayon's global pool implicitly without
// passing through it -- `render.rs`, `sample_bridge.rs` and `bake.rs` all do,
// outside `generate_terrain`'s call graph. In a shell that had drawn
// anything, `set_configured_thread_count(2)` therefore found `ACTIVE_THREADS`
// still `0`, ran `build_global()` (which returned `Err`, discarded),
// changed nothing, and returned `true`. [`POOL_BUILT_FROM`] replaces that
// inference with what the build actually did.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Once;

/// `0` = "let Rayon pick its own default" -- the same number an untouched
/// process always ran at before this existed, and never a value SS2.5's own
/// "1 to the logical core count" range can produce, so it doubles safely as
/// the "nothing configured yet" sentinel the way `DccSettings.
/// gpu_vram_budget_gb`'s `0` = "no cap" does.
static CONFIGURED_THREADS: AtomicUsize = AtomicUsize::new(0);
/// The pool's real, already-built size -- `0` until [`ensure_thread_pool`]
/// has actually run once this process, never `0` after (Rayon guarantees a
/// built pool has at least one worker). Ground truth: what
/// [`configured_thread_count`] asks for and what is actually running can
/// disagree (see this section's own doc comment above), and this is the
/// other half of that pair, not a copy of it.
static ACTIVE_THREADS: AtomicUsize = AtomicUsize::new(0);
/// What [`ensure_thread_pool`]'s one build actually asked Rayon for -- the
/// [`CONFIGURED_THREADS`] value at that instant -- or `usize::MAX` for **"the
/// running pool is not ours"**, which is what `build_global()` returning
/// `Err` means: something else (any implicit `par_iter()`) had already built
/// the global pool, so nothing this crate asked for is in effect.
///
/// `usize::MAX` is safe as that marker because it is not a value any request
/// can carry: [`set_configured_thread_count`] clamps every request to
/// `0..=logical_core_count()` before storing it.
///
/// This is the third distinct number in this section and not a duplicate of
/// either: [`CONFIGURED_THREADS`] is what is wanted *next*,
/// [`ACTIVE_THREADS`] is how many workers are running, and this is what the
/// build was told to do -- the only one of the three that can answer "did my
/// request take effect" without inferring it.
static POOL_BUILT_FROM: AtomicUsize = AtomicUsize::new(usize::MAX);
static POOL_INIT: Once = Once::new();

/// Logical cores this machine reports -- the same `available_parallelism()`
/// call `cartalith_godot::render::recommended_quality_tier` already makes,
/// exposed here too because [`set_configured_thread_count`] needs it to
/// clamp into SS2.5's "1 to the logical core count" range, and a settings UI
/// needs it to lay out a ladder (its own quarter/half/all rungs) without a
/// second source for the same number.
#[must_use]
pub fn logical_core_count() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
}

/// **Engine init.** Builds Rayon's global pool sized from
/// [`CONFIGURED_THREADS`] -- called at the top of `generate_terrain_inner`,
/// the one function every generation reaches, and of `cartalith_godot::
/// WorldGen::load_save` (a real top-level entry point that never calls
/// `generate_terrain` -- loading replaces the world by decoding a save
/// instead of generating one, but can still reach `par_iter()` downstream,
/// e.g. via a texture rebuild, so it needs this too). `pub` so any other
/// entry point this crate does not know about can call it defensively;
/// idempotent by construction (`Once`), so doing so always costs at most one
/// atomic check. A second call, from any thread, does nothing, and a
/// concurrent first call from another thread blocks on this one rather than
/// racing it.
pub fn ensure_thread_pool() {
    POOL_INIT.call_once(|| {
        let want = CONFIGURED_THREADS.load(Ordering::Relaxed);
        let mut b = rayon::ThreadPoolBuilder::new();
        if want > 0 {
            b = b.num_threads(want);
        }
        // `Err` whenever the global pool already exists -- which is the
        // ordinary case in the Godot shell, not an exotic one: an implicit
        // `par_iter()` in `render.rs`/`sample_bridge.rs`/`bake.rs` builds it
        // at Rayon's own default long before any settings row is opened. The
        // `Result` **is** needed, and discarding it is what let
        // `set_configured_thread_count` claim a count it never applied.
        POOL_BUILT_FROM
            .store(if b.build_global().is_ok() { want } else { usize::MAX }, Ordering::Relaxed);
        ACTIVE_THREADS.store(rayon::current_num_threads(), Ordering::Relaxed);
    });
}

/// The preference [`ensure_thread_pool`] will build with, or already built
/// with -- `0` for "follow Rayon's own default". Exactly what was last
/// stored by [`set_configured_thread_count`], which does the clamping; this
/// does not re-clamp, so it always answers what that call actually set.
#[must_use]
pub fn configured_thread_count() -> usize {
    CONFIGURED_THREADS.load(Ordering::Relaxed)
}

/// Record the preferred worker count, clamped to `0..=logical_core_count()`
/// (`0` = auto). The preference is **always** stored, whatever the return
/// value: a caller persisting it for the next launch reads
/// [`configured_thread_count`] back and gets exactly what it asked for.
///
/// Returns whether **the pool this process is actually running is the one
/// this request asked for** -- `true` only when this call built it (or an
/// earlier call built it from the identical request), `false` when anything
/// else got there first, including the implicit build any `par_iter()`
/// performs. `false` therefore means "recorded; takes effect at next start",
/// and it is the honest answer in every already-warm process. See this
/// section's own doc comment above for why a live rebuild is not offered, and
/// for the measured false-`true` this replaces.
pub fn set_configured_thread_count(threads: usize) -> bool {
    let clamped = if threads == 0 { 0 } else { threads.min(logical_core_count()) };
    CONFIGURED_THREADS.store(clamped, Ordering::Relaxed);
    ensure_thread_pool();
    // Not `ACTIVE_THREADS == clamped`, which would answer `true` for a
    // request that merely *coincides* with a pool somebody else built, and
    // could not describe `0` (auto) at all.
    POOL_BUILT_FROM.load(Ordering::Relaxed) == clamped
}

/// What Rayon's global pool is **actually** running with this process --
/// `0` before [`ensure_thread_pool`] has run for the first time (no
/// generation and no [`set_configured_thread_count`] call yet this process),
/// never `0` after.
#[must_use]
pub fn thread_pool_active_count() -> usize {
    ACTIVE_THREADS.load(Ordering::Relaxed)
}

/// `compute_stress` with every per-cell part on `gpu` (`OUTSTANDING_WORK.md`
/// §2.6): the boundary loop as `gpu_stress.wgsl`'s gather over a per-plate-pair
/// table of the CPU's own `stress_edge`, then both blurs on the existing
/// `gauss_blur_grid_gpu_with`, then the CPU's own `normalize_by_abs_max`.
/// Same arguments as `compute_stress`; `None` at any GPU step, and the caller
/// runs the CPU function instead.
///
/// Mask and type are bit-identical to the CPU's; the two fields are
/// principled-equivalent, held to `STRESS_GPU_TOL` in this file's tests.
#[allow(clippy::too_many_arguments)]
pub(crate) fn compute_stress_gpu(
    gpu: &cartalith_gpu::GpuDevice,
    gw: usize,
    gh: usize,
    world: bool,
    plate_id: &[u16],
    plates: &[cartalith_terrain::Plate],
    vel: f64,
    blur_r: f64,
) -> Option<cartalith_terrain::StressResult> {
    use cartalith_terrain::{normalize_by_abs_max, stress_edge};
    let pairs: Vec<_> = plates.iter().flat_map(|a| plates.iter().map(move |b| stress_edge(a, b, vel))).collect();
    let g = cartalith_gpu::stress_gather_grid_gpu_with(gpu, gw as u32, gh as u32, world, plate_id, &pairs)?;
    let blur = |f: &[f32]| cartalith_gpu::gauss_blur_grid_gpu_with(gpu, f, blur_r, gw as u32, gh as u32, world);
    let mut stress_field = blur(&g.raw)?;
    let mut shear_field = blur(&g.raw_s)?;
    normalize_by_abs_max(&mut stress_field);
    normalize_by_abs_max(&mut shear_field);
    Some(cartalith_terrain::StressResult {
        boundary_mask: g.boundary_mask,
        boundary_type: g.boundary_type,
        stress_field,
        shear_field,
    })
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
    // **Engine init** -- see this module's "CPU worker threads" section
    // above. Must run before any `par_iter()` this function (or anything it
    // calls) reaches; a no-op after the first call this process.
    ensure_thread_pool();

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

    // World-wrap support for warp's GPU kernel (`OUTSTANDING_WORK.md` §2.9,
    // closing GPU_LAYER_INTEGRATION_SCOPE.md milestone 2's own deferral):
    // both branches now dispatch to GPU, with `world` threaded through as
    // the periodic-noise flag rather than forcing CPU whenever it's set.
    let warp = if p.use_gpu {
        let amp = (p.tect.warp * 0.18 * gw as f64) as f32;
        if amp < 0.5 {
            None
        } else {
            // Matches `compute_warp`'s own `wf`: `3.0/gw` under world-wrap,
            // `2.5/gw` otherwise.
            let wf = (if world { 3.0 } else { 2.5 } / gw as f64) as f32;
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
                    cartalith_gpu::warp_grid_gpu_split(set, gw as u32, gh as u32, p.tect.seed, wf, amp, world)
                } else {
                    cartalith_gpu::warp_grid_gpu_with(set.primary(), gw as u32, gh as u32, p.tect.seed, wf, amp, world)
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

    // `gpu_device` is already `None` unless `use_gpu` is on, the VRAM budget
    // allows this grid and the device supports it -- the gates every stage
    // here shares. `None` from the dispatch runs the untouched CPU function.
    let stress = match gpu_device
        .and_then(|gpu| compute_stress_gpu(gpu, gw, gh, world, &plate_id, &plates, tect_vel, p.tect.blur_r))
    {
        Some(s) => {
            gpu_stages_used.push("stress".to_string());
            s
        }
        None => compute_stress(gw, gh, world, &plate_id, &plates, tect_vel, p.tect.blur_r),
    };

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

    let heterogeneity_field = if p.use_gpu {
        let hetero_seed = p.tect.seed ^ 0x44bb; // matches compute_heterogeneity's own seed derivation
        let hf_f64 = 1.5 * cartalith_terrain::terrain_detail_k(gw, p.map_width_km);
        let hf = hf_f64 as f32;
        // `compute_heterogeneity`'s own `oct = round(hf).max(2)`, passed as
        // `pfbm`'s period argument under world-wrap -- computed from the
        // f64 `hf` (not the already-f32-narrowed one above) to match that
        // function's own rounding point.
        let p_x = (js_round(hf_f64).max(2.0)) as i32;
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
            cartalith_gpu::heterogeneity_grid_gpu_with(
                gpu,
                gw as u32,
                gh as u32,
                hetero_seed,
                hf / gw as f32,
                world,
                p_x,
                &age_field,
                wx,
                wy,
            )
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
        // `EdificeModel::Reference` is `stampOneVolcano` exactly; the
        // morphological model is opt-in and needs its own owner ruling before
        // it can be the default. See `VolcanismParams::edifice_model`.
        let edifice = if p.volc.edifice_model {
            cartalith_terrain::EdificeModel::Morphological
        } else {
            cartalith_terrain::EdificeModel::Reference
        };
        // stampVolcanoes() (reference HTML lines 3474-3478): dispatches on
        // state.volc.provinces, JS default true.
        if p.volc.provinces {
            stamp_volcanoes_provinces_shaped(
                gw,
                gh,
                p.tect.seed as u32,
                p.map_width_km,
                p.peak_m,
                &stress.boundary_mask,
                &stress.stress_field,
                p.volc.exclude_transform.then_some(&stress.boundary_type[..]),
                &plate_id,
                &plates,
                volc_count,
                p.volc.age,
                &mut field,
                &mut volcanic_field,
                edifice,
            );
        } else {
            stamp_volcanoes_simple_shaped(
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
                edifice,
            );
        }
    }
    // Craters as a density over the map's real area, not an absolute count
    // (`DECISIONS.md` §7l). `physical_model: false` restores the reference's
    // own path exactly.
    let crater_d_min = cartalith_terrain::crater_min_diameter_km(p.map_width_km, gw);
    let crater_count = if p.crater.physical_model {
        let height_km = p.map_width_km * gh as f64 / gw.max(1) as f64;
        cartalith_terrain::auto_crater_count(
            p.tect.seed as u32,
            p.crater.count,
            p.map_width_km * height_km,
            crater_d_min,
            p.crater.surface_age_myr,
        )
    } else {
        p.crater.count
    };
    stamp_craters(
        gw,
        gh,
        p.tect.seed as u32,
        p.map_width_km,
        p.planet.g,
        crater_count,
        p.crater.age,
        p.crater.physical_model,
        crater_d_min,
        // One geological exposure age drives both how many craters accumulated
        // and how far each has since relaxed -- see
        // `cartalith_terrain::crater_degradation_tau`. Inert under
        // `physical_model: false`.
        p.crater.surface_age_myr,
        // One world, one diffusivity: crater relaxation is the same physics as
        // `hillslope_diffuse` (`DECISIONS.md` §7m) at a different scale, so it
        // reads the same coefficient rather than a private anchor -- owner
        // ruling 2, 2026-09-02, §7l-ii. Read RAW, not through
        // `hillslope_extent_scale`: that correction is a discretisation fix for
        // the one-cell Laplacian below, while `crater_degradation_tau` works in
        // km and Myr and wants the physical quantity. Read whether or not
        // `passes.hillslope` is on, because a diffusivity is a property of the
        // landscape rather than of which passes the user enabled. Also inert
        // under `physical_model: false`.
        p.passes.diffuse_d,
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
    //
    // Integrated drainage (`p.integrate_drainage`, `RC_ENGINE_CHANGES.md`
    // §6g): every flow accumulation below routes over the depression-filled
    // surface of the field *as it is at that call* -- the carve moves the
    // field between them, so each takes its own. `routing_view` borrows
    // `field` unchanged when the flag is off, so the off path is the old call.
    // The GPU kernel takes the surface the same way the CPU one does: it only
    // orders and picks receivers by the heights it is handed.
    let integrate = p.integrate_drainage;
    let flow_area = {
        let route = routing_view(&field, gw, gh, sea_level, world, integrate);
        match flow_on_gpu(&route, None, false) {
            Some(v) => {
                gpu_stages_used.push("flow".to_string());
                v
            }
            None => compute_flow(gw, gh, &route, None, false, world),
        }
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
        let route = routing_view(&field, gw, gh, sea_level, world, integrate);
        match flow_on_gpu(&route, Some(&rainfall), true) {
            Some(v) => {
                if !gpu_stages_used.iter().any(|s| s == "flow") {
                    gpu_stages_used.push("flow".to_string());
                }
                v
            }
            None => compute_flow(gw, gh, &route, Some(&rainfall), true, world),
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
        // One routing surface for BOTH trees -- the accumulation and the
        // channel network's own receivers (`RC_ENGINE_CHANGES.md` §6g: filling
        // one and not the other makes the flow and the traced network describe
        // two different objects). Slope stays on the real field inside
        // `build_channels_routed`.
        let route = routing_view(&field, gw, gh, sea_level, world, integrate);
        let flow_for_network = match flow_on_gpu(&route, Some(&rainfall), true) {
            Some(v) => {
                if !gpu_stages_used.iter().any(|s| s == "flow") {
                    gpu_stages_used.push("flow".to_string());
                }
                v
            }
            None => compute_flow(gw, gh, &route, Some(&rainfall), true, world),
        };

        // (2) vector network -> distance-field channel carve + lock
        let mut ch = build_channels_routed(
            &field,
            &route,
            &flow_for_network,
            gw,
            gh,
            sea_level,
            world,
            p.river_density,
            p.map_width_km,
        );
        drop(route);
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
        //
        // `river_render_area_bar` is v2.72's second, independent gate --
        // "a detection ease is not a display threshold" -- on top of
        // `river_flow_thresh`'s own already-eased channelization threshold
        // above. See `stamp_river_intensity`'s own doc comment for why this
        // is a per-cell drainage bar and not a re-use of `river_flow_thresh`.
        ch.intensity = cartalith_hydrology::stamp_river_intensity(
            &field,
            &flow_for_network,
            &ch.chan,
            &ch.recv,
            &order,
            gw,
            gh,
            world,
            cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km),
            width_k,
            cartalith_hydrology::river_render_area_bar(p.map_width_km),
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
        flow_discharge = {
            let route = routing_view(&field, gw, gh, sea_level, world, integrate);
            match flow_on_gpu(&route, Some(&rainfall), true) {
                Some(v) => {
                    if !gpu_stages_used.iter().any(|s| s == "flow") {
                        gpu_stages_used.push("flow".to_string());
                    }
                    v
                }
                None => compute_flow(gw, gh, &route, Some(&rainfall), true, world),
            }
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
            // The one call site that knows the map's real extent, so the one
            // that can correct for it. `diffuse_d` is a coefficient over a
            // one-cell Laplacian, so it must scale as 1/cell_km^2; without
            // this a 5 km region and a 40 000 km world diffused identically,
            // wrong by the same 64 000 000x factor `DECISIONS.md` §7l cites
            // for craters. Exactly 1.0 at the app's own 800 km / 2048 default,
            // so nothing moves there. `DECISIONS.md` §7m.
            let d_scale =
                cartalith_erosion::hillslope_extent_scale(p.map_width_km, gw, q.diffuse_d);
            hillslope_diffuse(&mut field, gw, gh, q.diffuse_passes, q.diffuse_d, d_scale, world);
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
            flow_discharge = compute_flow_routed(gw, gh, &field, Some(&rainfall), true, world, sea_level, integrate);
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
        field: Arc::new(field),
        plate_id,
        boundary_mask: stress.boundary_mask,
        stress_field: stress.stress_field,
        age_field: Arc::new(age_field),
        resistance_field: Arc::new(resistance_field),
        crust_field: Arc::new(base_raw),
        boundary_type: stress.boundary_type,
        shear_field: stress.shear_field,
        volcanic_field: Arc::new(volcanic_field),
        impact_field,
        temperature: Arc::new(temperature),
        rainfall: Arc::new(rainfall),
        flow_discharge: Arc::new(flow_discharge),
        integrated_drainage: integrate,
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
    *flow_discharge =
        compute_flow_routed(gw, gh, field, Some(rainfall), true, world, sea_level, p.integrate_drainage);
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

    /// **Owner ruling 2, 2026-09-02, and the one exception it carves into the
    /// contract above.**
    ///
    /// `crater_degradation_tau` now reads `passes.diffuse_d`, so under
    /// `crater.physical_model` that knob is no longer a pure erosion-pass knob:
    /// it is the world's hillslope diffusivity, and craters relax at it whether
    /// or not the hillslope *pass* runs. Moving it therefore changes generation
    /// with every toggle off — which is the intended coupling, not a leak.
    ///
    /// Pinned in both directions so neither half can rot silently: inert on the
    /// reference path (which is what keeps the test above true, and with it the
    /// sixteen `cartalith-civ` golden suites), live on the shipped path.
    #[test]
    fn the_erosion_diffusivity_reaches_craters_only_under_the_physical_model() {
        // A REGION, deliberately, and an old surface. Under the physical model
        // the crater count is a Poisson draw about an area density, and at the
        // app's own 800 km extent a small test grid resolves nothing below
        // ~33 km, giving lambda ~ 0.12 -- i.e. usually **no craters at all**,
        // and a test that proves nothing while passing. (Written that way
        // first; it passed once and then flipped. The fifth instance of this
        // project's silently-empty-output trap.) At 64 km over 96x72 the floor
        // is 1.33 km and lambda ~ 15, and the assertion below makes the
        // population's existence a precondition rather than a hope.
        let mut p = WorldParams::defaults(96, 72, 913);
        p.map_width_km = 64.0;
        p.crater.count = 200;
        p.crater.surface_age_myr = 2000.0;
        assert!(!p.passes.any(), "the coupling must be visible with every pass off");

        // Reference path: `physical_model` is false in `WorldParams::defaults`,
        // so degradation never runs and the diffusivity is inert.
        assert!(!p.crater.physical_model);
        let a = generate_terrain(&p);
        p.passes.diffuse_d = 0.02;
        let b = generate_terrain(&p);
        assert_eq!(a.field, b.field, "the reference path must ignore diffuse_d entirely");

        // Shipped path: the same move now changes the world.
        p.crater.physical_model = true;
        p.passes.diffuse_d = 0.15;
        let c = generate_terrain(&p);
        p.passes.diffuse_d = 0.02;
        let d = generate_terrain(&p);
        let craters = c.impact_field.iter().filter(|&&v| v > 0.0).count();
        assert!(
            craters > 50,
            "only {craters} shocked cells -- the physical model drew no crater \
             population, so this test would pass on emptiness"
        );
        assert_ne!(c.field, d.field, "diffuse_d did not reach crater degradation");
        assert_ne!(
            c.impact_field, d.impact_field,
            "diffuse_d did not reach the shock record (owner ruling 3)"
        );
    }

    /// The anchor `cartalith_terrain::crater_degradation_tau` was calibrated at
    /// must be the diffusivity this crate actually ships, or the "bit-identical
    /// at the default" claim in its doc comment is quietly false. The two live
    /// in different crates — `cartalith-terrain` cannot import this one — so
    /// this is where they are compared.
    #[test]
    fn the_crater_anchor_matches_the_shipped_diffusivity() {
        assert_eq!(
            cartalith_terrain::CRATER_DEGRADATION_DIFFUSE_D_REF,
            ErosionPassParams::off().diffuse_d,
            "crater degradation was calibrated at a diffusivity this crate no longer \
             ships -- re-anchor it deliberately or restore the default"
        );
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
            let moved = ws.field.iter().zip(reference.iter()).filter(|(a, b)| a != b).count();
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
        for (i, (&after, &before)) in ws.field.iter().zip(base.field.iter()).enumerate() {
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

    /// A thread count must never move a golden -- the CPU worker-thread
    /// lane's own hard constraint. Independent, disposable *scoped* pools
    /// (not the global one `ensure_thread_pool` builds) so two genuinely
    /// different worker counts can be compared within one test process
    /// without fighting Rayon's "global pool builds exactly once" rule.
    /// Every `par_iter()` this pipeline reaches must already be
    /// order-independent of its worker count (`cartalith-civ`'s
    /// indexed-collection comment at its own `par_iter().collect()` sites is
    /// why) -- this checks that claim directly against a live generation
    /// rather than trusting the comment.
    #[test]
    fn thread_count_does_not_change_generated_output() {
        let p = WorldParams::defaults(24, 18, 909);
        // Warm the global pool outside any scoped `install()` first.
        // `ensure_thread_pool` reads `rayon::current_num_threads()`, which
        // reports whichever pool is "current" -- triggering its one-shot
        // `Once` from inside a scoped `install()` below would wrongly record
        // that pool's size as the *global* one for the rest of this test
        // binary. This guarantees the `Once` has already fired against the
        // real global pool before the comparison runs.
        let _ = generate_terrain(&p);

        let one = rayon::ThreadPoolBuilder::new().num_threads(1).build().expect("1-thread pool");
        let many = rayon::ThreadPoolBuilder::new().num_threads(4).build().expect("4-thread pool");
        let a = one.install(|| generate_terrain(&p));
        let b = many.install(|| generate_terrain(&p));
        assert_eq!(a.field, b.field, "generation must be identical regardless of the Rayon pool's worker count");
    }

    /// `ensure_thread_pool` is called from `generate_terrain_inner` before
    /// any other work (see this module's "CPU worker threads" section) --
    /// reverting that call leaves `ACTIVE_THREADS` at its `0` initial value
    /// forever, since nothing else in this crate writes it. Order-independent
    /// against the rest of this test binary: once built, the pool stays
    /// built for the process, so an earlier test having already triggered
    /// this only makes the assertion trivially still true.
    #[test]
    fn generate_terrain_builds_the_thread_pool() {
        let p = WorldParams::defaults(8, 8, 1);
        let _ = generate_terrain(&p);
        assert!(thread_pool_active_count() > 0, "the global pool must be built by the time a generation returns");
    }

    /// SS2.5's own range, "an integer from 1 to the logical core count", plus
    /// the `0` = auto sentinel this port adds. Reads back exactly what was
    /// clamped, independent of pool-build state or of what any other test in
    /// this binary has done to it.
    #[test]
    fn set_configured_thread_count_clamps_into_the_logical_range() {
        set_configured_thread_count(0);
        assert_eq!(configured_thread_count(), 0, "0 must read back as 0 (auto), not resolve to a core count");
        let cores = logical_core_count();
        assert!(cores >= 1);
        set_configured_thread_count(usize::MAX);
        assert_eq!(configured_thread_count(), cores, "an over-range request must clamp to the logical core count");
        set_configured_thread_count(1);
        assert_eq!(configured_thread_count(), 1);
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
            ["warp", "warp_split", "heterogeneity", "plate_assignment", "stress", "base_field_blur", "weather", "flow"];
        for s in &a.gpu_stages_used {
            assert!(known.contains(&s.as_str()), "unexpected gpu_stages_used entry: {s}");
        }
    }

    /// `OUTSTANDING_WORK.md` §2.9 "World-wrap support for the milestone 1-5
    /// kernels": the positive-reachability check -- `world=true` with
    /// `use_gpu=true` must actually DISPATCH warp and heterogeneity on GPU,
    /// not silently fall back to CPU the way both stages did before this
    /// change (`p.use_gpu && !world`, in this function's own prior
    /// history). Same instrumentation every other GPU stage in this file
    /// is proven by: `gpu_stages_used`. Environment-tolerant like its
    /// sibling above -- on a machine with no usable GPU, both fall back and
    /// the positive assertions are skipped (still checked structurally,
    /// see [`generate_terrain_gpu_and_cpu_paths_share_worldstate_shape`]
    /// for the CPU-path contract).
    #[test]
    fn generate_terrain_world_wrap_reaches_gpu_warp_and_heterogeneity() {
        let mut p = WorldParams::defaults(24, 18, 777);
        p.use_gpu = true;
        p.world = true;
        let ws = generate_terrain(&p);

        assert!(ws.field.iter().all(|&v| v.is_finite()), "no NaN/Inf in a world=true GPU-path height field");
        assert!(!ws.field.is_empty(), "a generation that produces nothing measures nothing");

        if cartalith_gpu::last_backend().is_none() {
            eprintln!("no GPU opened on this run (CPU fallback) -- world-wrap GPU reachability has nothing to prove here");
            return;
        }
        assert!(
            ws.gpu_stages_used.iter().any(|s| s == "warp" || s == "warp_split"),
            "world=true, use_gpu=true, real GPU opened -- warp must have dispatched on GPU, not fallen back to CPU. gpu_stages_used = {:?}",
            ws.gpu_stages_used
        );
        assert!(
            ws.gpu_stages_used.iter().any(|s| s == "heterogeneity"),
            "world=true, use_gpu=true, real GPU opened -- heterogeneity must have dispatched on GPU, not fallen back to CPU. gpu_stages_used = {:?}",
            ws.gpu_stages_used
        );
    }

    /// The same reachability check, but proving `use_gpu=true` under
    /// `world=true` produces a genuinely different -- and still valid --
    /// field from `world=false`, the way any other world-wrap toggle does
    /// (`compute_warp`'s own `wf`/noise-function branch on `world`).
    #[test]
    fn generate_terrain_world_wrap_gpu_output_differs_from_non_world() {
        let mut p_world = WorldParams::defaults(24, 18, 777);
        p_world.use_gpu = true;
        p_world.world = true;
        let mut p_flat = WorldParams::defaults(24, 18, 777);
        p_flat.use_gpu = true;
        p_flat.world = false;

        let a = generate_terrain(&p_world);
        let b = generate_terrain(&p_flat);

        assert_eq!(a.field.len(), b.field.len());
        assert!(a.field.iter().all(|&v| v.is_finite()));
        assert!(b.field.iter().all(|&v| v.is_finite()));
        let differs = a.field.iter().zip(b.field.iter()).any(|(x, y)| (x - y).abs() > 1e-6);
        assert!(differs, "world=true must produce a genuinely different field from world=false under use_gpu=true");
    }

    /// Ruling Y (`LARGE_ITEM_RULINGS.md`, 2026-09-21): the actual entry
    /// point the ruling is about, not `cartalith-gpu`'s own lower-level
    /// handshake measurement. Before the ruling, every `use_gpu=true`
    /// `generate_terrain` call opened (and dropped) its own device, so call
    /// two paid the same ~190-235 ms adapter/device handshake as call one
    /// (`cartalith_gpu::GpuDevice`'s own doc comment carries that figure and
    /// its range). After it, `cartalith_gpu::init_gpu_device_set`'s
    /// process-wide cache means only the *first* GPU-path call of the
    /// process pays it.
    ///
    /// `#[ignore]`d for the same reason as `cartalith-gpu`'s own
    /// `measured_device_handshake_and_per_stage_pipeline_build`: it wants an
    /// uncontended device, and this project's own timing rule
    /// (`MISTAKES.md`) is that a figure taken under a parallel `cargo test`
    /// run is a figure about contention, not about the code. Run alone:
    /// `cargo test --release -p cartalith-engine measured_generate_terrain_reuses_the_gpu_device -- --ignored --test-threads=1 --nocapture`.
    ///
    /// Tiny grid (24x18, `generate_terrain_gpu_path_is_deterministic_and_valid`'s
    /// own size) on purpose: at this size the CPU-equivalent work is a
    /// rounding error next to the handshake, so the wall-clock gap between
    /// call one and every call after it is almost entirely the thing this
    /// test is measuring, not noise from the generation itself.
    #[test]
    #[ignore = "wants an uncontended device; run alone with --ignored --test-threads=1 --nocapture"]
    fn measured_generate_terrain_reuses_the_gpu_device_across_calls() {
        let mut p = WorldParams::defaults(24, 18, 777);
        p.use_gpu = true;

        let mut elapsed = Vec::new();
        for _ in 0..3 {
            let t = std::time::Instant::now();
            let ws = generate_terrain(&p);
            elapsed.push(t.elapsed());
            assert!(!ws.field.is_empty(), "a generation that produces nothing measures nothing");
        }
        eprintln!(
            "generate_terrain(use_gpu=true) x3, same process: call 1 = {:?}, call 2 = {:?}, call 3 = {:?}",
            elapsed[0], elapsed[1], elapsed[2]
        );

        if cartalith_gpu::last_backend().is_none() {
            eprintln!("no GPU opened on this run (CPU fallback) -- the device-reuse cost has nothing to measure here");
            return;
        }
        assert!(
            elapsed[1] < elapsed[0],
            "call 2 must be faster than call 1 once the device is cached -- call 1 = {:?}, call 2 = {:?}",
            elapsed[0],
            elapsed[1]
        );
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
    /// per call* (see the milestone-6 header comment above `warp_grid_gpu_with`
    /// et al. in `cartalith-gpu` for why that's an accepted tradeoff for
    /// one-shot batch generation). `--nocapture` to see the numbers; `#[ignore]`d
    /// since it's a timing report, not a correctness check, and full
    /// 2048x2048 CPU pipeline runs are slow enough to not want in the
    /// default `cargo test` loop.
    ///
    /// **Now a median of [`TIMING_ROUNDS`] per size, with the spread printed.**
    /// `OUTSTANDING_WORK.md` §2.6: *"the 2048² ratio moved 1.19x -> 0.98x with
    /// no code change, so single-run variance is currently indistinguishable
    /// from a result."* It was -- and worse than that row knew. While §2.6's
    /// GPU rows were being answered, one `gpu_height` dispatch at 2048²
    /// measured anywhere from 38 ms to 78 ms across runs of the same binary,
    /// and taking medians *reversed the sign* of one conclusion drawn from
    /// single samples. A lone `Instant::now()` pair here reports noise with
    /// the confidence of a result. The min/max are printed beside the median
    /// so a reader can see how much to trust it, rather than being handed one
    /// number with no spread.
    #[test]
    #[ignore]
    fn measured_generate_terrain_gpu_vs_cpu_timing() {
        /// Odd, so the median is a real sample and not an average of two.
        const TIMING_ROUNDS: usize = 3;

        /// Median, min and max of `TIMING_ROUNDS` runs of `f`, plus its last
        /// value (the `WorldState`, for `gpu_stages_used`).
        fn timed<T>(mut f: impl FnMut() -> T) -> (std::time::Duration, std::time::Duration, std::time::Duration, T) {
            let mut times = Vec::with_capacity(TIMING_ROUNDS);
            let mut last = None;
            for _ in 0..TIMING_ROUNDS {
                let t0 = std::time::Instant::now();
                let v = f();
                times.push(t0.elapsed());
                last = Some(v);
            }
            times.sort_unstable();
            (times[TIMING_ROUNDS / 2], times[0], times[TIMING_ROUNDS - 1], last.expect("TIMING_ROUNDS is non-zero"))
        }

        for &sz in &[128usize, 512, 1024, 2048] {
            let mut p_gpu = WorldParams::defaults(sz, sz, 24601);
            p_gpu.use_gpu = true;
            let p_cpu = WorldParams::defaults(sz, sz, 24601);

            let (gpu_time, gpu_min, gpu_max, ws_gpu) = timed(|| generate_terrain(&p_gpu));
            let (cpu_time, cpu_min, cpu_max, _ws_cpu) = timed(|| generate_terrain(&p_cpu));

            eprintln!(
                "generate_terrain {sz}x{sz} (median of {TIMING_ROUNDS}): use_gpu=true = {:?} [{:?}..{:?}] \
                 (stages actually on GPU: {:?}), use_gpu=false = {:?} [{:?}..{:?}], ratio (CPU/GPU) = {:.2}x",
                gpu_time,
                gpu_min,
                gpu_max,
                ws_gpu.gpu_stages_used,
                cpu_time,
                cpu_min,
                cpu_max,
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

    /// CPU-vs-GPU tolerance for `compute_stress_gpu`'s `stress_field` and
    /// `shear_field` (both normalised to a max `|v|` of 1), **derived from
    /// measurement**, not assumed. `measured_stress_gpu_vs_cpu` below,
    /// 2026-09-23, AMD Radeon RX 7800 XT (Vulkan), release, run alone: the
    /// generator's own plate maps at 128², 256², 512², 1024² and 2048², each
    /// with and without world-wrap -- worst element deviation **5.36e-7**
    /// (1024² world shear; 2.98e-7 to 4.77e-7 elsewhere), i.e. a few ulp of
    /// f32 at 1.0. `boundary_mask`/`boundary_type` mismatched on **0** cells in
    /// all ten, and GPU-vs-GPU was **exactly 0** in all ten.
    ///
    /// 2e-6 is ~3.7x the measured worst. A real porting error -- a dropped or
    /// doubled edge, a flipped pair orientation -- moves `raw` by a whole
    /// edge's `c`, which survives the blur at the 1e-2 scale, so it cannot
    /// hide under this (the mutation run confirms it per defect).
    const STRESS_GPU_TOL: f32 = 2e-6;

    /// The generator's own plate map for `p`: `generate_terrain`'s `plate_id`
    /// and the `build_plates` call it makes (world-structure off, the default,
    /// so `plates`/`vel` are `p.tect`'s). Asserts the pair reproduces the
    /// generation's own boundary mask, so the fixture cannot drift from what
    /// `generate_terrain` actually feeds `compute_stress`.
    fn stress_fixture(p: &WorldParams) -> (Vec<u16>, Vec<cartalith_terrain::Plate>) {
        assert!(!p.world_structure.enabled && !p.use_gpu);
        let ws = generate_terrain(p);
        let plates =
            cartalith_terrain::build_plates(p.gw, p.gh, p.tect.seed as u32, p.tect.plates, p.tect.lloyd, p.world, None);
        let cpu = compute_stress(p.gw, p.gh, p.world, &ws.plate_id, &plates, p.tect.vel, p.tect.blur_r);
        assert_eq!(cpu.boundary_mask, ws.boundary_mask, "fixture must be generate_terrain's own plate map");
        (ws.plate_id.clone(), plates)
    }

    fn worst_abs(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len());
        a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0f32, f32::max)
    }

    /// The whole contract, one fixture: mask and type bit-identical, both
    /// fields within `STRESS_GPU_TOL`, and a second dispatch bit-identical.
    fn assert_stress_gpu_matches(
        gpu: &cartalith_gpu::GpuDevice,
        what: &str,
        (gw, gh, world): (usize, usize, bool),
        plate_id: &[u16],
        plates: &[cartalith_terrain::Plate],
        vel: f64,
        blur_r: f64,
    ) -> bool {
        let cpu = compute_stress(gw, gh, world, plate_id, plates, vel, blur_r);
        let Some(g) = compute_stress_gpu(gpu, gw, gh, world, plate_id, plates, vel, blur_r) else {
            eprintln!("{what}: device refused {gw}x{gh} -- skipped");
            return false;
        };
        assert!(cpu.boundary_mask.iter().any(|&m| m != 0), "{what}: the fixture must have boundaries");
        assert_eq!(g.boundary_mask, cpu.boundary_mask, "{what}: boundary_mask must be bit-identical");
        assert_eq!(g.boundary_type, cpu.boundary_type, "{what}: boundary_type must be bit-identical");
        let (ws, wh) = (worst_abs(&g.stress_field, &cpu.stress_field), worst_abs(&g.shear_field, &cpu.shear_field));
        assert!(ws <= STRESS_GPU_TOL, "{what}: stress_field deviates by {ws:e} > {STRESS_GPU_TOL:e}");
        assert!(wh <= STRESS_GPU_TOL, "{what}: shear_field deviates by {wh:e} > {STRESS_GPU_TOL:e}");
        let again = compute_stress_gpu(gpu, gw, gh, world, plate_id, plates, vel, blur_r).expect("same device, same grid");
        assert_eq!(again.stress_field, g.stress_field, "{what}: a gather with no atomics must be bit-deterministic");
        assert_eq!(again.shear_field, g.shear_field, "{what}: a gather with no atomics must be bit-deterministic");
        true
    }

    /// The GPU stress path against the real CPU `compute_stress`, on the
    /// generator's own plate maps -- **non-square** (a swapped axis cannot
    /// pass), with and without world-wrap -- plus hand-built fixtures for what
    /// a real map rarely reaches: one- and two-column world grids (where the
    /// row-wrap edge and the right edge coincide), and a many-plate map built
    /// from EXACT and NEAR magnitude ties with different boundary types, which
    /// is what decides the dominant-type pick's order and its f64-vs-f32 `>=`.
    /// Environment-tolerant: no device is a skip.
    #[test]
    fn gpu_stress_matches_cpu_stress_within_measured_tolerance() {
        let Some(set) = cartalith_gpu::init_gpu_device_set().ok() else {
            eprintln!("no GPU device -- CPU-only machine, nothing to compare");
            return;
        };
        let gpu = set.primary();
        for &(gw, gh, seed, world) in &[(96usize, 64usize, 24601, false), (150, 200, 777, true), (64, 48, 31337, true)] {
            let mut p = WorldParams::defaults(gw, gh, seed);
            p.world = world;
            let (plate_id, plates) = stress_fixture(&p);
            // The pair table is orientation-symmetric: swapping scanner and
            // neighbour negates both the normal and the velocity difference,
            // so `c`, `s`, `mag` and the type are unchanged (equal as f64s;
            // only a zero's sign can differ, and `+0 + -0` is `+0`). That is
            // why reading the table transposed is an EQUIVALENT mutant of the
            // shader, not a missed one -- asserted here rather than claimed.
            for a in &plates {
                for b in &plates {
                    let (ab, ba) = (cartalith_terrain::stress_edge(a, b, p.tect.vel), cartalith_terrain::stress_edge(b, a, p.tect.vel));
                    assert!(ab.0 == ba.0 && ab.1 == ba.1 && ab.2 == ba.2 && ab.3 == ba.3, "stress_edge must be symmetric in its plates");
                }
            }
            if !assert_stress_gpu_matches(gpu, "generated", (gw, gh, world), &plate_id, &plates, p.tect.vel, p.tect.blur_r) {
                return;
            }
        }

        // Exact ties with different types. A (continental, at rest) meets B
        // (oceanic, at (1,0), moving -x) and C (continental, at (0,1), moving
        // -y): both edges are pure convergence of exactly `vel` in f64, one a
        // subduction and one a collision, so the dominant-type pick is decided
        // by the tie rule alone -- by the ORDER edges reach a cell, and by the
        // CPU's f64 `mag >= f32 dom`. At vel 1.1 `f32(mag)` rounds UP, so the
        // CPU keeps the FIRST edge; at 1.3 it rounds down and the LATER edge
        // wins. A real plate map almost never ties two types exactly (the
        // generated fixtures above cannot see an order or `>=` defect).
        let plate = |x: f64, y: f64, vx: f64, vy: f64, base: f64| cartalith_terrain::Plate { x, y, vx, vy, base };
        let plates = [plate(0.0, 0.0, 0.0, 0.0, 0.4), plate(1.0, 0.0, -1.0, 0.0, -0.4), plate(0.0, 1.0, 0.0, -1.0, 0.4)];
        assert_eq!(cartalith_terrain::stress_edge(&plates[0], &plates[1], 1.1).2, 1.1);
        assert_eq!(cartalith_terrain::stress_edge(&plates[0], &plates[2], 1.1).2, 1.1);
        assert!(1.1f64 < f64::from(1.1f32) && 1.3f64 > f64::from(1.3f32), "one vel must round up, one down");
        let mut rng = 0x9e37_79b9u32;
        let mut ids = |n: usize| {
            (0..n)
                .map(|_| {
                    rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    ((rng >> 8) % 3) as u16
                })
                .collect::<Vec<u16>>()
        };
        for &(gw, gh, world) in &[(1usize, 7usize, true), (2, 5, true), (2, 5, false), (37, 23, true), (37, 23, false)] {
            let id = ids(gw * gh);
            for vel in [1.1, 1.3] {
                for blur in [0.0, 2.5] {
                    let what = format!("exact ties {gw}x{gh} world {world} vel {vel} blur {blur}");
                    if !assert_stress_gpu_matches(gpu, &what, (gw, gh, world), &id, &plates, vel, blur) {
                        return;
                    }
                }
            }
        }
    }

    /// `use_gpu: false` -- the engine default, asserted as a literal -- never
    /// reports `"stress"`; `true` reports it exactly when a device takes the
    /// grid, and the stress stage's own outputs land within tolerance of the
    /// CPU's on the same plate map.
    #[test]
    fn generate_terrain_use_gpu_moves_stress_to_the_gpu_and_only_when_asked() {
        let (gw, gh) = (72usize, 48usize);
        let p = WorldParams::defaults(gw, gh, 4242);
        assert!(!p.use_gpu, "engine default must stay CPU");
        let cpu = generate_terrain(&p);
        assert!(!cpu.gpu_stages_used.iter().any(|s| s == "stress"), "use_gpu=false must never reach the GPU");

        let mut pg = p.clone();
        pg.use_gpu = true;
        let g = generate_terrain(&pg);
        let device = cartalith_gpu::init_gpu_device_set().ok().is_some_and(|s| s.supports_grid(gw, gh));
        assert_eq!(g.gpu_stages_used.iter().any(|s| s == "stress"), device, "stress must run on the GPU exactly when a device takes it");
        if device {
            // Other GPU stages (warp, plate assignment) move the plate map, so
            // hold the GPU generation's stress against the CPU function on the
            // GPU generation's OWN plate map.
            let plates = cartalith_terrain::build_plates(gw, gh, p.tect.seed as u32, p.tect.plates, p.tect.lloyd, false, None);
            let want = compute_stress(gw, gh, false, &g.plate_id, &plates, p.tect.vel, p.tect.blur_r);
            assert_eq!(g.boundary_mask, want.boundary_mask);
            assert_eq!(g.boundary_type, want.boundary_type);
            assert!(worst_abs(&g.stress_field, &want.stress_field) <= STRESS_GPU_TOL);
            assert!(worst_abs(&g.shear_field, &want.shear_field) <= STRESS_GPU_TOL);
            // Positive evidence the GPU produced these fields: its f32
            // accumulation is not bit-identical to the CPU's f64-then-round
            // on any real map (measured: 12 050 to 909 687 unequal cells on
            // every fixture), so a wiring that reported "stress" and kept the
            // CPU result would pass every check above and fail this one.
            assert_ne!(g.stress_field, want.stress_field, "the reported GPU stress stage must actually be the GPU's output");
        }
    }

    /// Measurement harness: CPU `compute_stress` vs `compute_stress_gpu` on
    /// the generator's own plate maps, deviation and timing. `#[ignore]`d --
    /// run alone:
    /// `cargo test --release -p cartalith-engine measured_stress_gpu -- --ignored --test-threads=1 --nocapture`
    #[test]
    #[ignore]
    fn measured_stress_gpu_vs_cpu() {
        const ROUNDS: usize = 5;
        fn timed<T>(mut f: impl FnMut() -> T) -> (std::time::Duration, std::time::Duration, std::time::Duration, T) {
            let mut times = Vec::new();
            let mut last = None;
            for _ in 0..ROUNDS {
                let t0 = std::time::Instant::now();
                last = Some(f());
                times.push(t0.elapsed());
            }
            times.sort_unstable();
            (times[ROUNDS / 2], times[0], times[ROUNDS - 1], last.unwrap())
        }
        let Some(set) = cartalith_gpu::init_gpu_device_set().ok() else {
            eprintln!("no GPU device -- nothing to measure");
            return;
        };
        let gpu = set.primary();
        eprintln!("device: {} ({:?})", gpu.adapter_name, gpu.adapter_backend);
        for &(sz, seed) in &[(128usize, 24601), (256, 777), (512, 24601), (1024, 4242), (2048, 24601)] {
            for world in [false, true] {
                let mut p = WorldParams::defaults(sz, sz, seed);
                p.world = world;
                let (plate_id, plates) = stress_fixture(&p);
                let (vel, br) = (p.tect.vel, p.tect.blur_r);
                let (cpu_t, cpu_lo, cpu_hi, cpu) = timed(|| compute_stress(sz, sz, world, &plate_id, &plates, vel, br));
                let (gpu_t, gpu_lo, gpu_hi, g) = timed(|| {
                    compute_stress_gpu(gpu, sz, sz, world, &plate_id, &plates, vel, br).expect("GPU dispatch")
                });
                let g2 = compute_stress_gpu(gpu, sz, sz, world, &plate_id, &plates, vel, br).expect("GPU dispatch");
                let boundary = cpu.boundary_mask.iter().filter(|&&m| m != 0).count();
                let mask_ne = cpu.boundary_mask.iter().zip(&g.boundary_mask).filter(|(a, b)| a != b).count();
                let type_ne = cpu.boundary_type.iter().zip(&g.boundary_type).filter(|(a, b)| a != b).count();
                let unequal = |a: &[f32], b: &[f32]| a.iter().zip(b).filter(|(x, y)| x != y).count();
                eprintln!(
                    "{sz}x{sz} seed {seed} world {world} plates {}: boundary cells {boundary}; mask/type mismatches {mask_ne}/{type_ne}; \
                     stress worst {:e} (unequal {}), shear worst {:e} (unequal {}); GPU-vs-GPU stress {:e} shear {:e}; \
                     CPU {cpu_t:?} [{cpu_lo:?}..{cpu_hi:?}] GPU {gpu_t:?} [{gpu_lo:?}..{gpu_hi:?}] (n={ROUNDS})",
                    plates.len(),
                    worst_abs(&cpu.stress_field, &g.stress_field),
                    unequal(&cpu.stress_field, &g.stress_field),
                    worst_abs(&cpu.shear_field, &g.shear_field),
                    unequal(&cpu.shear_field, &g.shear_field),
                    worst_abs(&g.stress_field, &g2.stress_field),
                    worst_abs(&g.shear_field, &g2.shear_field),
                );
            }
        }
    }
}
