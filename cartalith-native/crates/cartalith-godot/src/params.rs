//! The generation-parameter table: one flat, dotted-key namespace over
//! `cartalith_engine::WorldParams` and its eight sub-structs.
//!
//! Deliberately **free of any `godot` dependency** so it can be unit-tested
//! the same way `render.rs` is (`tests/params_mapping.rs` pulls it in via
//! `#[path = "../src/params.rs"]`) — `lib.rs` owns the thin
//! `Variant` <-> [`Value`] conversion and nothing else.
//!
//! ## Why one flat table rather than ~58 individual `#[func]` setters
//!
//! `GUI_FEATURE_PARITY_SCOPE.md`'s own "honest size statement" counts
//! "~60-80 individual stage sliders" as the largest single piece of the
//! parity effort. Emitting one `#[func]` per field would make the GDScript
//! side hardcode 58 names, 58 ranges, 58 steps and 58 labels a second time —
//! the exact duplication that lets a slider silently drift from the range the
//! reference actually shipped. Instead the table below is the single source of
//! truth for *what* each parameter is, and the GUI builds its dialogs from
//! [`PARAMS`] via `get_param_info()`. Adding a parameter is one row here, and
//! no GDScript change at all.
//!
//! ## Ranges are the reference's own, not invented
//!
//! Every [`ParamSpec::min`]/[`max`](ParamSpec::max)/[`step`](ParamSpec::step)
//! for a parameter with a non-empty [`ParamSpec::reference_control`] is the
//! real reachable range of that control in `reference/Cartalith Gen1
//! v2.10.html`, converted through that control's own `tparam`/`cparam`/
//! `eparam`/`bind` mapping function (e.g. the `alpha` slider is `0..100`
//! raw and maps `v => v/100*1.2`, so this table carries `0.0..1.2` with a
//! step of `0.012`). `GENERATION_PARAMETERS.md` records the raw slider
//! range and the mapping alongside each row. A row with an empty
//! `reference_control` is a parameter the reference never surfaced as a
//! user control — its range is this port's own judgement, flagged as such
//! rather than presented as parity.

use cartalith_engine::staleness::PipelineStage;
use cartalith_engine::WorldParams;

/// What a parameter's value *is*, for the GUI's control-type choice and for
/// this module's own type checking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A checkbox. Only a real boolean is accepted — see [`set`].
    Bool,
    /// A whole number (`SpinBox`/integer slider). Stored as an integer field
    /// on `WorldParams`; a fractional input is rounded, not rejected.
    Int,
    /// A continuous slider.
    Float,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Bool => "bool",
            Kind::Int => "int",
            Kind::Float => "float",
        }
    }
}

/// A single parameter value crossing the GDScript boundary, already reduced
/// to the two shapes `WorldParams` actually stores.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Num(f64),
}

/// What [`set`] did with a value. Reported back to GDScript per key so a
/// dialog can echo the *actual* stored value rather than assuming its own
/// widget won.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Stored exactly as given.
    Applied,
    /// Stored, but adjusted to the parameter's valid range (or rounded to a
    /// whole number for [`Kind::Int`]). Never silently: the caller gets the
    /// key back in `set_params()`'s `"clamped"` list.
    Clamped,
    /// Not stored at all — unknown key, or a value whose type can't be
    /// coerced to this parameter's [`Kind`].
    Rejected,
}

/// One row of the parameter table.
pub struct ParamSpec {
    /// The dotted key GDScript uses, e.g. `"tect.plates"`. Mirrors the
    /// `WorldParams` field path exactly, so a reader of either side can find
    /// the other without a lookup table.
    pub key: &'static str,
    /// Which dialog section this belongs in — matches the reference HTML's
    /// own sidebar panel headings (`"tectonics"` is its **Tectonics** panel,
    /// `"weather"` its **Weather · rainfall sim** panel, and so on).
    pub group: &'static str,
    pub kind: Kind,
    /// Inclusive lower bound. Values below are clamped, not rejected.
    pub min: f64,
    /// Inclusive upper bound.
    pub max: f64,
    /// The reference control's own step, already converted into this
    /// parameter's units. Advisory: [`set`] does not snap to it.
    pub step: f64,
    /// The reference's own label text where it has one, otherwise a plain
    /// description. Wording lives here rather than in GDScript for the same
    /// reason the ranges do.
    pub label: &'static str,
    /// Display suffix (`"°"`, `"h"`, `"m"`, `"×"`), empty when unitless.
    pub unit: &'static str,
    /// The reference HTML element id this parameter's user control has
    /// (`"sea"`, `"plates"`, `"pg"`, ...), or `""` when the reference never
    /// exposed it — an internal tuning constant this port chooses to surface
    /// anyway (`DECISIONS.md` §7d: a superset is not a violation as long as
    /// the default reproduces reference behaviour).
    pub reference_control: &'static str,
    get_fn: fn(&WorldParams) -> Value,
    set_fn: fn(&mut WorldParams, f64),
}

/// The defaults every row's `default` is read from — `WorldParams::defaults`
/// itself, at a zero grid/seed (neither of which is a settable parameter;
/// both are `generate()` arguments).
pub fn defaults() -> WorldParams {
    WorldParams::defaults(0, 0, 0)
}

/// Every exposed generation parameter, in the order the GUI should show them
/// (grouped, and within a group in the reference's own panel order).
#[rustfmt::skip]
pub const PARAMS: &[ParamSpec] = &[
    // ---- world: Source & resolution + Scale & calibration ----------------
    ParamSpec { key: "world", group: "world", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Whole world (wrap in longitude)", unit: "", reference_control: "extentSeg",
        get_fn: |p| Value::Bool(p.world), set_fn: |p, v| p.world = v != 0.0 },
    ParamSpec { key: "sea_level", group: "world", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Sea level", unit: "", reference_control: "sea",
        get_fn: |p| Value::Num(p.sea_level), set_fn: |p, v| p.sea_level = v },
    ParamSpec { key: "peak_m", group: "world", kind: Kind::Float, min: 1.0, max: 30000.0, step: 50.0,
        label: "Peak altitude", unit: "m", reference_control: "peak",
        get_fn: |p| Value::Num(p.peak_m), set_fn: |p, v| p.peak_m = v },
    ParamSpec { key: "carve_rivers", group: "world", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Carve river valleys on generation", unit: "", reference_control: "carveRiversChk",
        get_fn: |p| Value::Bool(p.carve_rivers), set_fn: |p, v| p.carve_rivers = v != 0.0 },
    ParamSpec { key: "river_density", group: "world", kind: Kind::Float, min: 0.30, max: 3.0, step: 0.05,
        label: "River density", unit: "\u{d7}", reference_control: "riverDensR",
        get_fn: |p| Value::Num(p.river_density), set_fn: |p, v| p.river_density = v },
    ParamSpec { key: "use_gpu", group: "world", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "GPU acceleration", unit: "", reference_control: "gpuToggle",
        get_fn: |p| Value::Bool(p.use_gpu), set_fn: |p, v| p.use_gpu = v != 0.0 },

    // ---- planet ----------------------------------------------------------
    ParamSpec { key: "planet.g", group: "planet", kind: Kind::Float, min: 0.30, max: 2.50, step: 0.05,
        label: "Gravity", unit: "g", reference_control: "pg",
        get_fn: |p| Value::Num(p.planet.g), set_fn: |p, v| p.planet.g = v },
    ParamSpec { key: "planet.rotation_hours", group: "planet", kind: Kind::Float, min: 6.0, max: 96.0, step: 1.0,
        label: "Day length", unit: "h", reference_control: "prot",
        get_fn: |p| Value::Num(p.planet.rotation_hours), set_fn: |p, v| p.planet.rotation_hours = v },
    ParamSpec { key: "planet.axial_tilt_deg", group: "planet", kind: Kind::Float, min: 0.0, max: 45.0, step: 0.5,
        label: "Axial tilt", unit: "\u{b0}", reference_control: "ptilt",
        get_fn: |p| Value::Num(p.planet.axial_tilt_deg), set_fn: |p, v| p.planet.axial_tilt_deg = v },

    // ---- world structure -------------------------------------------------
    ParamSpec { key: "world_structure.enabled", group: "world_structure", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Enable continental steering", unit: "", reference_control: "wsEnabled",
        get_fn: |p| Value::Bool(p.world_structure.enabled), set_fn: |p, v| p.world_structure.enabled = v != 0.0 },
    ParamSpec { key: "world_structure.continentality", group: "world_structure", kind: Kind::Float, min: 0.01, max: 0.90, step: 0.01,
        label: "Continentality", unit: "", reference_control: "wsCont",
        get_fn: |p| Value::Num(p.world_structure.continentality), set_fn: |p, v| p.world_structure.continentality = v },
    ParamSpec { key: "world_structure.fragmentation", group: "world_structure", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Fragmentation", unit: "", reference_control: "wsFrag",
        get_fn: |p| Value::Num(p.world_structure.fragmentation), set_fn: |p, v| p.world_structure.fragmentation = v },
    ParamSpec { key: "world_structure.tectonic_energy", group: "world_structure", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Tectonic energy", unit: "", reference_control: "wsTect",
        get_fn: |p| Value::Num(p.world_structure.tectonic_energy), set_fn: |p, v| p.world_structure.tectonic_energy = v },
    ParamSpec { key: "world_structure.ocean_depth", group: "world_structure", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Ocean depth", unit: "", reference_control: "wsOcean",
        get_fn: |p| Value::Num(p.world_structure.ocean_depth), set_fn: |p, v| p.world_structure.ocean_depth = v },
    ParamSpec { key: "world_structure.hotspot_density", group: "world_structure", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Hotspot density", unit: "", reference_control: "wsHot",
        get_fn: |p| Value::Num(p.world_structure.hotspot_density), set_fn: |p, v| p.world_structure.hotspot_density = v },

    // ---- tectonics -------------------------------------------------------
    ParamSpec { key: "tect.plates", group: "tectonics", kind: Kind::Int, min: 4.0, max: 40.0, step: 1.0,
        label: "Plates", unit: "", reference_control: "plates",
        get_fn: |p| Value::Num(p.tect.plates as f64), set_fn: |p, v| p.tect.plates = v as usize },
    ParamSpec { key: "tect.vel", group: "tectonics", kind: Kind::Float, min: 0.0, max: 2.0, step: 0.02,
        label: "Drift", unit: "\u{d7}", reference_control: "vel",
        get_fn: |p| Value::Num(p.tect.vel), set_fn: |p, v| p.tect.vel = v },
    ParamSpec { key: "tect.warp", group: "tectonics", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Warp", unit: "", reference_control: "warp",
        get_fn: |p| Value::Num(p.tect.warp), set_fn: |p, v| p.tect.warp = v },
    ParamSpec { key: "tect.blur_r", group: "tectonics", kind: Kind::Float, min: 2.0, max: 42.0, step: 0.4,
        label: "Uplift spread", unit: "px", reference_control: "sigma",
        get_fn: |p| Value::Num(p.tect.blur_r), set_fn: |p, v| p.tect.blur_r = v },
    ParamSpec { key: "tect.alpha", group: "tectonics", kind: Kind::Float, min: 0.0, max: 1.2, step: 0.012,
        label: "Tectonic \u{3b1}", unit: "", reference_control: "alpha",
        get_fn: |p| Value::Num(p.tect.alpha), set_fn: |p, v| p.tect.alpha = v },
    ParamSpec { key: "tect.beta", group: "tectonics", kind: Kind::Float, min: 0.0, max: 0.6, step: 0.006,
        label: "Noise \u{3b2}", unit: "", reference_control: "beta",
        get_fn: |p| Value::Num(p.tect.beta), set_fn: |p, v| p.tect.beta = v },
    ParamSpec { key: "tect.age_inf", group: "tectonics", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Erosion / age", unit: "", reference_control: "age",
        get_fn: |p| Value::Num(p.tect.age_inf), set_fn: |p, v| p.tect.age_inf = v },
    ParamSpec { key: "tect.ridged", group: "tectonics", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Ridged mountain detail", unit: "", reference_control: "ridged",
        get_fn: |p| Value::Bool(p.tect.ridged), set_fn: |p, v| p.tect.ridged = v != 0.0 },
    ParamSpec { key: "tect.flexure", group: "tectonics", kind: Kind::Float, min: 0.0, max: 0.36, step: 0.006,
        label: "Flexure F", unit: "", reference_control: "flexure",
        get_fn: |p| Value::Num(p.tect.flexure), set_fn: |p, v| p.tect.flexure = v },
    ParamSpec { key: "tect.hetero", group: "tectonics", kind: Kind::Float, min: 0.0, max: 0.16, step: 0.004,
        label: "Heterogeneity C", unit: "", reference_control: "hetero",
        get_fn: |p| Value::Num(p.tect.hetero), set_fn: |p, v| p.tect.hetero = v },
    ParamSpec { key: "tect.resist", group: "tectonics", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Rock resistance", unit: "", reference_control: "resist",
        get_fn: |p| Value::Num(p.tect.resist), set_fn: |p, v| p.tect.resist = v },
    ParamSpec { key: "tect.dynamic_lithology", group: "tectonics", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Dynamic lithology (exhumation hardens rock)", unit: "", reference_control: "dynLithChk",
        get_fn: |p| Value::Bool(p.tect.dynamic_lithology), set_fn: |p, v| p.tect.dynamic_lithology = v != 0.0 },
    ParamSpec { key: "tect.lloyd", group: "tectonics", kind: Kind::Int, min: 0.0, max: 8.0, step: 1.0,
        label: "Lloyd relaxation passes", unit: "", reference_control: "",
        get_fn: |p| Value::Num(p.tect.lloyd as f64), set_fn: |p, v| p.tect.lloyd = v as usize },

    // ---- volcanism & impacts --------------------------------------------
    ParamSpec { key: "volc.count", group: "volcanism", kind: Kind::Int, min: 0.0, max: 100.0, step: 1.0,
        label: "Volcanoes", unit: "", reference_control: "volc",
        get_fn: |p| Value::Num(p.volc.count as f64), set_fn: |p, v| p.volc.count = v as i32 },
    ParamSpec { key: "volc.age", group: "volcanism", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Volcano age", unit: "", reference_control: "volca",
        get_fn: |p| Value::Num(p.volc.age), set_fn: |p, v| p.volc.age = v },
    ParamSpec { key: "volc.provinces", group: "volcanism", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Provinces & arc/rift placement", unit: "", reference_control: "volcProv",
        get_fn: |p| Value::Bool(p.volc.provinces), set_fn: |p, v| p.volc.provinces = v != 0.0 },
    ParamSpec { key: "crater.count", group: "volcanism", kind: Kind::Int, min: 0.0, max: 200.0, step: 2.0,
        label: "Craters", unit: "", reference_control: "crat",
        get_fn: |p| Value::Num(p.crater.count as f64), set_fn: |p, v| p.crater.count = v as i32 },
    ParamSpec { key: "crater.age", group: "volcanism", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Crater age", unit: "", reference_control: "crata",
        get_fn: |p| Value::Num(p.crater.age), set_fn: |p, v| p.crater.age = v },

    // ---- erosion (the stream-power pass carveRiverValleys runs) ----------
    ParamSpec { key: "stream.uplift", group: "erosion", kind: Kind::Float, min: 0.0, max: 0.4, step: 0.004,
        label: "Uplift", unit: "", reference_control: "sUp",
        get_fn: |p| Value::Num(p.stream.uplift), set_fn: |p, v| p.stream.uplift = v },
    ParamSpec { key: "stream.k", group: "erosion", kind: Kind::Float, min: 0.0, max: 0.03, step: 0.0003,
        label: "Channeling", unit: "", reference_control: "sK",
        get_fn: |p| Value::Num(p.stream.k), set_fn: |p, v| p.stream.k = v },
    ParamSpec { key: "stream.iters", group: "erosion", kind: Kind::Int, min: 4.0, max: 40.0, step: 1.0,
        label: "Iterations", unit: "", reference_control: "sIt",
        get_fn: |p| Value::Num(p.stream.iters as f64), set_fn: |p, v| p.stream.iters = v as i32 },
    ParamSpec { key: "stream.deposit", group: "erosion", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Deposition", unit: "", reference_control: "sDep",
        get_fn: |p| Value::Num(p.stream.deposit), set_fn: |p, v| p.stream.deposit = v },
    ParamSpec { key: "stream.climate_k", group: "erosion", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Rain \u{2192} erosion", unit: "", reference_control: "sClim",
        get_fn: |p| Value::Num(p.stream.climate_k), set_fn: |p, v| p.stream.climate_k = v },

    // ---- erosion passes (the reference's manual buttons, as parameters) ---
    // `GUI_GAP_REGISTER.md` §19 / WW-02 / MS-04 / MS-05, and
    // `cartalith_engine::ErosionPassParams`' own doc comment for why these are
    // parameters here and buttons there. The six `.enabled` rows have an empty
    // `reference_control` for exactly that reason -- the reference's control is
    // a *button*, not a checkbox, so this port's toggle is a §7d addition and
    // says so. Every knob row *does* name its reference slider, and carries
    // that slider's real reachable range through its own `eparam` mapping.
    ParamSpec { key: "passes.velocity", group: "erosion", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Velocity (momentum) erosion", unit: "", reference_control: "",
        get_fn: |p| Value::Bool(p.passes.velocity), set_fn: |p, v| p.passes.velocity = v != 0.0 },
    ParamSpec { key: "passes.velo_iters", group: "erosion", kind: Kind::Int, min: 10.0, max: 160.0, step: 1.0,
        label: "Velocity iterations", unit: "", reference_control: "vIt",
        get_fn: |p| Value::Num(p.passes.velo_iters as f64), set_fn: |p, v| p.passes.velo_iters = v as i32 },
    ParamSpec { key: "passes.velo_strength", group: "erosion", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Velocity strength", unit: "", reference_control: "vStr",
        get_fn: |p| Value::Num(p.passes.velo_strength), set_fn: |p, v| p.passes.velo_strength = v },
    ParamSpec { key: "passes.velo_meander", group: "erosion", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Velocity meander", unit: "", reference_control: "vMnd",
        get_fn: |p| Value::Num(p.passes.velo_meander), set_fn: |p, v| p.passes.velo_meander = v },
    ParamSpec { key: "passes.glacial", group: "erosion", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Glacial erosion", unit: "", reference_control: "",
        get_fn: |p| Value::Bool(p.passes.glacial), set_fn: |p, v| p.passes.glacial = v != 0.0 },
    ParamSpec { key: "passes.glacial_snowline", group: "erosion", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Snowline", unit: "", reference_control: "gSnow",
        get_fn: |p| Value::Num(p.passes.glacial_snowline), set_fn: |p, v| p.passes.glacial_snowline = v },
    ParamSpec { key: "passes.glacial_kg", group: "erosion", kind: Kind::Float, min: 0.01, max: 1.0, step: 0.01,
        label: "Glacial intensity", unit: "", reference_control: "gKg",
        get_fn: |p| Value::Num(p.passes.glacial_kg), set_fn: |p, v| p.passes.glacial_kg = v },
    ParamSpec { key: "passes.glacial_mg", group: "erosion", kind: Kind::Float, min: 0.0, max: 2.0, step: 0.05,
        label: "Glacial discharge exponent", unit: "", reference_control: "",
        get_fn: |p| Value::Num(p.passes.glacial_mg), set_fn: |p, v| p.passes.glacial_mg = v },
    ParamSpec { key: "passes.glacial_u_factor", group: "erosion", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "U-width", unit: "", reference_control: "gUF",
        get_fn: |p| Value::Num(p.passes.glacial_u_factor), set_fn: |p, v| p.passes.glacial_u_factor = v },
    ParamSpec { key: "passes.glacial_passes", group: "erosion", kind: Kind::Int, min: 1.0, max: 30.0, step: 1.0,
        label: "Glacial passes", unit: "", reference_control: "gPas",
        get_fn: |p| Value::Num(p.passes.glacial_passes as f64), set_fn: |p, v| p.passes.glacial_passes = v as i32 },
    ParamSpec { key: "passes.coastal", group: "erosion", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Coastal processes", unit: "", reference_control: "",
        get_fn: |p| Value::Bool(p.passes.coastal), set_fn: |p, v| p.passes.coastal = v != 0.0 },
    ParamSpec { key: "passes.wave_str", group: "erosion", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Wave strength", unit: "", reference_control: "cWave",
        get_fn: |p| Value::Num(p.passes.wave_str), set_fn: |p, v| p.passes.wave_str = v },
    ParamSpec { key: "passes.estuary_depth", group: "erosion", kind: Kind::Float, min: 0.0, max: 0.2, step: 0.002,
        label: "Estuary depth", unit: "", reference_control: "cEst",
        get_fn: |p| Value::Num(p.passes.estuary_depth), set_fn: |p, v| p.passes.estuary_depth = v },
    ParamSpec { key: "passes.marsh_band", group: "erosion", kind: Kind::Float, min: 0.0, max: 0.1, step: 0.001,
        label: "Marsh band", unit: "", reference_control: "cMar",
        get_fn: |p| Value::Num(p.passes.marsh_band), set_fn: |p, v| p.passes.marsh_band = v },
    ParamSpec { key: "passes.coastal_passes", group: "erosion", kind: Kind::Int, min: 1.0, max: 15.0, step: 1.0,
        label: "Coastal passes", unit: "", reference_control: "cPas",
        get_fn: |p| Value::Num(p.passes.coastal_passes as f64), set_fn: |p, v| p.passes.coastal_passes = v as i32 },
    ParamSpec { key: "passes.hillslope", group: "erosion", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Hillslope diffuse", unit: "", reference_control: "",
        get_fn: |p| Value::Bool(p.passes.hillslope), set_fn: |p, v| p.passes.hillslope = v != 0.0 },
    ParamSpec { key: "passes.diffuse_d", group: "erosion", kind: Kind::Float, min: 0.002, max: 0.2, step: 0.002,
        label: "Diffusivity D", unit: "", reference_control: "edD",
        get_fn: |p| Value::Num(p.passes.diffuse_d), set_fn: |p, v| p.passes.diffuse_d = v },
    ParamSpec { key: "passes.diffuse_passes", group: "erosion", kind: Kind::Int, min: 1.0, max: 40.0, step: 1.0,
        label: "Diffuse passes", unit: "", reference_control: "edPas",
        get_fn: |p| Value::Num(p.passes.diffuse_passes as f64), set_fn: |p, v| p.passes.diffuse_passes = v as i32 },
    ParamSpec { key: "passes.sediment_fill", group: "erosion", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Sediment fill (route + redeposit)", unit: "", reference_control: "",
        get_fn: |p| Value::Bool(p.passes.sediment_fill), set_fn: |p, v| p.passes.sediment_fill = v != 0.0 },
    ParamSpec { key: "passes.sediment_capacity", group: "erosion", kind: Kind::Float, min: 0.0, max: 20.0, step: 0.5,
        label: "Sediment transport capacity", unit: "", reference_control: "",
        get_fn: |p| Value::Num(p.passes.sediment_capacity), set_fn: |p, v| p.passes.sediment_capacity = v },
    // 0 is off -- the reference's own slider starts at 2 because pressing the
    // button *is* the "on", which a parameter has no equivalent of.
    ParamSpec { key: "passes.evolve_cycles", group: "erosion", kind: Kind::Int, min: 0.0, max: 12.0, step: 1.0,
        label: "Evolve climate \u{2194} terrain cycles", unit: "", reference_control: "evoCyc",
        get_fn: |p| Value::Num(p.passes.evolve_cycles as f64), set_fn: |p, v| p.passes.evolve_cycles = v as i32 },
    // The seventh pass. Unlike the six above it, the reference gates its
    // button on `state.planet.tides.enabled` as well -- this toggle is both,
    // because turning it on is what builds the tide field it reads. See
    // `ErosionPassParams::tidal_flats`.
    ParamSpec { key: "passes.tidal_flats", group: "erosion", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Tidal flats (mudflat accretion)", unit: "", reference_control: "",
        get_fn: |p| Value::Bool(p.passes.tidal_flats), set_fn: |p, v| p.passes.tidal_flats = v != 0.0 },
    ParamSpec { key: "passes.tidal_k", group: "erosion", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Tidal accretion rate", unit: "", reference_control: "",
        get_fn: |p| Value::Num(p.passes.tidal_k), set_fn: |p, v| p.passes.tidal_k = v },

    // ---- climate & biomes ------------------------------------------------
    ParamSpec { key: "climate.lat_n", group: "climate", kind: Kind::Float, min: -90.0, max: 90.0, step: 1.0,
        label: "North edge", unit: "\u{b0}", reference_control: "latN",
        get_fn: |p| Value::Num(p.climate.lat_n), set_fn: |p, v| p.climate.lat_n = v },
    ParamSpec { key: "climate.lat_s", group: "climate", kind: Kind::Float, min: -90.0, max: 90.0, step: 1.0,
        label: "South edge", unit: "\u{b0}", reference_control: "latS",
        get_fn: |p| Value::Num(p.climate.lat_s), set_fn: |p, v| p.climate.lat_s = v },
    ParamSpec { key: "climate.equator_temp", group: "climate", kind: Kind::Float, min: 0.0, max: 45.0, step: 1.0,
        label: "Equator temperature", unit: "\u{b0}C", reference_control: "teq",
        get_fn: |p| Value::Num(p.climate.equator_temp), set_fn: |p, v| p.climate.equator_temp = v },
    ParamSpec { key: "climate.pole_temp", group: "climate", kind: Kind::Float, min: -50.0, max: 10.0, step: 1.0,
        label: "Pole temperature", unit: "\u{b0}C", reference_control: "tpo",
        get_fn: |p| Value::Num(p.climate.pole_temp), set_fn: |p, v| p.climate.pole_temp = v },
    ParamSpec { key: "climate.lapse_rate", group: "climate", kind: Kind::Float, min: 0.0, max: 12.0, step: 0.1,
        label: "Lapse rate", unit: "\u{b0}C/km", reference_control: "lapse",
        get_fn: |p| Value::Num(p.climate.lapse_rate), set_fn: |p, v| p.climate.lapse_rate = v },
    ParamSpec { key: "climate.albedo_k", group: "climate", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Ice albedo", unit: "", reference_control: "albedo",
        get_fn: |p| Value::Num(p.climate.albedo_k), set_fn: |p, v| p.climate.albedo_k = v },
    ParamSpec { key: "climate.currents", group: "climate", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Ocean currents", unit: "", reference_control: "currents",
        get_fn: |p| Value::Bool(p.climate.currents), set_fn: |p, v| p.climate.currents = v != 0.0 },
    ParamSpec { key: "climate.current_k", group: "climate", kind: Kind::Float, min: 0.0, max: 3.0, step: 0.05,
        label: "Ocean-current strength", unit: "\u{d7}", reference_control: "",
        get_fn: |p| Value::Num(p.climate.current_k), set_fn: |p, v| p.climate.current_k = v },
    ParamSpec { key: "climate.terrain_wind_deflection", group: "climate", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Terrain wind deflection", unit: "", reference_control: "",
        get_fn: |p| Value::Bool(p.climate.terrain_wind_deflection), set_fn: |p, v| p.climate.terrain_wind_deflection = v != 0.0 },

    // ---- weather - rainfall sim ------------------------------------------
    ParamSpec { key: "climate.w_iters", group: "weather", kind: Kind::Int, min: 20.0, max: 200.0, step: 5.0,
        label: "Iterations", unit: "", reference_control: "wIters",
        get_fn: |p| Value::Num(p.climate.w_iters as f64), set_fn: |p, v| p.climate.w_iters = v as i32 },
    ParamSpec { key: "climate.rain_k", group: "weather", kind: Kind::Float, min: 0.0, max: 2.0, step: 0.01,
        label: "Orographic", unit: "\u{d7}", reference_control: "rainK",
        get_fn: |p| Value::Num(p.climate.rain_k), set_fn: |p, v| p.climate.rain_k = v },
    ParamSpec { key: "climate.evap", group: "weather", kind: Kind::Float, min: 0.0, max: 0.3, step: 0.003,
        label: "Evaporation", unit: "", reference_control: "evap",
        get_fn: |p| Value::Num(p.climate.evap), set_fn: |p, v| p.climate.evap = v },
    ParamSpec { key: "climate.rain_dep", group: "weather", kind: Kind::Float, min: 0.0, max: 1.0, step: 0.01,
        label: "Dryness", unit: "", reference_control: "rainDep",
        get_fn: |p| Value::Num(p.climate.rain_dep), set_fn: |p, v| p.climate.rain_dep = v },
    ParamSpec { key: "climate.ocean", group: "weather", kind: Kind::Float, min: 0.0, max: 2.0, step: 0.01,
        label: "Ocean supply", unit: "\u{d7}", reference_control: "ocean",
        get_fn: |p| Value::Num(p.climate.ocean), set_fn: |p, v| p.climate.ocean = v },
    ParamSpec { key: "climate.wind_manual", group: "weather", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Manual wind direction", unit: "", reference_control: "windModeSeg",
        get_fn: |p| Value::Bool(p.climate.wind_manual), set_fn: |p, v| p.climate.wind_manual = v != 0.0 },
    ParamSpec { key: "climate.wind_dir_deg", group: "weather", kind: Kind::Float, min: 0.0, max: 360.0, step: 5.0,
        label: "Wind direction", unit: "\u{b0}", reference_control: "windDir",
        get_fn: |p| Value::Num(p.climate.wind_dir_deg), set_fn: |p, v| p.climate.wind_dir_deg = v },
    ParamSpec { key: "climate.press_k", group: "weather", kind: Kind::Float, min: 0.0, max: 1.5, step: 0.05,
        label: "Pressure influence", unit: "", reference_control: "pressK",
        get_fn: |p| Value::Num(p.climate.press_k), set_fn: |p, v| p.climate.press_k = v },
    ParamSpec { key: "climate.zonal_k", group: "weather", kind: Kind::Float, min: 0.0, max: 1.5, step: 0.05,
        label: "Zonal belts", unit: "", reference_control: "zonalK",
        get_fn: |p| Value::Num(p.climate.zonal_k), set_fn: |p, v| p.climate.zonal_k = v },
    ParamSpec { key: "climate.ocean_hum", group: "weather", kind: Kind::Float, min: 0.0, max: 2.0, step: 0.01,
        label: "Sea-surface humidity", unit: "", reference_control: "",
        get_fn: |p| Value::Num(p.climate.ocean_hum), set_fn: |p, v| p.climate.ocean_hum = v },
    ParamSpec { key: "climate.bulk_evap", group: "weather", kind: Kind::Bool, min: 0.0, max: 1.0, step: 1.0,
        label: "Bulk aerodynamic evaporation", unit: "", reference_control: "",
        get_fn: |p| Value::Bool(p.climate.bulk_evap), set_fn: |p, v| p.climate.bulk_evap = v != 0.0 },
];

// ===========================================================================
// Saving and restoring the table (`SAVEFILE_COMPAT.md`, FI-01)
// ===========================================================================
//
// The `.zip` save's `params.json` carries a `state` object, and this module
// is where that object's *parameter* half is built and read back.
//
// ## Two copies of every value, deliberately
//
// A save written here holds each parameter twice:
//
// 1. Under [`NATIVE_PARAMS_KEY`] (`state.cartalith`), keyed by this table's
//    own dotted key. **This is the authoritative copy** and the only one
//    [`apply_saved_state`] reads. It is lossless by construction: every row
//    below round-trips, including the ten this port added that the reference
//    has no equivalent for (`use_gpu`, the six erosion-pass toggles,
//    `passes.sediment_capacity`, `passes.tidal_k`,
//    `climate.terrain_wind_deflection`).
// 2. At its **reference** `state` path (`tect.blurR`, `climate.latN`, ...),
//    where the row has one. This copy exists for one reader only: the
//    original HTML app, whose `loadZip()` can then reopen a file this port
//    wrote.
//
// The duplication is what keeps both honest. Without (1), the ten port-only
// parameters would be silently lost on every save. Without (2), a save this
// port wrote would reopen in the reference app with the reference app's
// *defaults* silently standing in for the world's real settings.
//
// ## Why (2) is not optional even if reference compatibility were dropped
//
// `loadZip()` merges the saved state **shallowly** — `Object.assign(state,
// pk.state)` — so any nested block written here *replaces* the reference's
// whole default block rather than merging into it. `state.tect.seed` is
// mandatory (this port's own reader requires it, `SAVEFILE_COMPAT.md`), so
// `tect` is written no matter what; writing it with only a seed in it would
// leave the reference app with an undefined plate count, drift, warp and
// blur radius. Once `tect` must be complete, every sibling block costs one
// table column.
//
// ## The one block deliberately not written: `state.erosion`
//
// `loadZip()` has no `Object.assign` shim for `erosion` (it does for
// `climate`, `stream`, `velo`, `glacial`, `coastal`, `planet`,
// `world_structure` and `viz`), and this port models 2 of its 16 keys —
// `diffuseD` and `diffusePasses`. Writing those two would replace the
// reference's entire droplet-erosion parameter set with a two-key object.
// Both rows therefore carry an empty reference path and travel in
// `state.cartalith` only; a save this port writes leaves the reference app
// on its own `erosion` defaults, which is a visible, documented limitation
// rather than a silently mangled block.

/// Where the parameter half of a save lives inside `state` — this port's
/// own dotted keys, verbatim, as the lossless copy. A key the reference
/// never wrote, so `loadZip()`'s `Object.assign` carries it through
/// untouched and `serializeState()` writes it back out: a save can make a
/// round trip *through the reference app* without losing this port's
/// parameters.
pub const NATIVE_PARAMS_KEY: &str = "cartalith";

/// Each [`PARAMS`] key's path inside the reference's own `state` object, or
/// `""` where the reference has no equivalent. Kept as its own table rather
/// than a tenth column on [`ParamSpec`] because it is a property of the
/// *reference*, not of this port's parameter — and because a row here that
/// names no key is caught by `every_param_has_a_reference_path_decision`,
/// which is the drift guard a column would otherwise provide.
///
/// Every path was read out of `reference/Cartalith Gen1 v2.10.html`'s own
/// `state` literal (line 2257) and its `loadZip()` shims (line 12624).
#[rustfmt::skip]
const JS_PATHS: &[(&str, &str)] = &[
    ("world", "world"),
    // The *effective* sea level is written over this by
    // `cartalith_io::params_json` -- the reference's own `state.seaLevel` is
    // likewise post-`deriveFromWorldStructure`, so the two agree. The input
    // value stays in the `cartalith` copy.
    ("sea_level", "seaLevel"),
    ("peak_m", "peakM"),
    ("carve_rivers", "carveRivers"),
    ("river_density", "viz.riverDensity"),
    // No reference equivalent: this port's own GPU switch.
    ("use_gpu", ""),

    ("planet.g", "planet.g"),
    ("planet.rotation_hours", "planet.rotationHours"),
    ("planet.axial_tilt_deg", "planet.axialTiltDeg"),

    ("world_structure.enabled", "world_structure.enabled"),
    ("world_structure.continentality", "world_structure.continentality"),
    ("world_structure.fragmentation", "world_structure.fragmentation"),
    ("world_structure.tectonic_energy", "world_structure.tectonicEnergy"),
    ("world_structure.ocean_depth", "world_structure.oceanDepth"),
    ("world_structure.hotspot_density", "world_structure.hotspotDensity"),

    ("tect.plates", "tect.plates"),
    ("tect.vel", "tect.vel"),
    ("tect.warp", "tect.warp"),
    ("tect.blur_r", "tect.blurR"),
    ("tect.alpha", "tect.alpha"),
    ("tect.beta", "tect.beta"),
    ("tect.age_inf", "tect.age"),
    ("tect.ridged", "tect.ridged"),
    ("tect.flexure", "tect.flexure"),
    ("tect.hetero", "tect.hetero"),
    ("tect.resist", "tect.resist"),
    ("tect.dynamic_lithology", "tect.dynamicLithology"),
    ("tect.lloyd", "tect.lloyd"),

    ("volc.count", "volc.count"),
    ("volc.age", "volc.age"),
    ("volc.provinces", "volc.provinces"),
    ("crater.count", "crater.count"),
    ("crater.age", "crater.age"),

    ("stream.uplift", "stream.uplift"),
    ("stream.k", "stream.k"),
    ("stream.iters", "stream.iters"),
    ("stream.deposit", "stream.deposit"),
    ("stream.climate_k", "stream.climateK"),

    // The six pass *toggles* are this port's own (`ParamSpec`'s own comment:
    // the reference's control is a button, not a checkbox), so none of them
    // has a reference path -- only the knobs behind them do.
    ("passes.velocity", ""),
    ("passes.velo_iters", "velo.iters"),
    ("passes.velo_strength", "velo.strength"),
    ("passes.velo_meander", "velo.meander"),
    ("passes.glacial", ""),
    ("passes.glacial_snowline", "glacial.snowline"),
    ("passes.glacial_kg", "glacial.kg"),
    ("passes.glacial_mg", "glacial.mg"),
    ("passes.glacial_u_factor", "glacial.uFactor"),
    ("passes.glacial_passes", "glacial.passes"),
    ("passes.coastal", ""),
    ("passes.wave_str", "coastal.waveStr"),
    ("passes.estuary_depth", "coastal.estuaryDepth"),
    ("passes.marsh_band", "coastal.marshBand"),
    ("passes.coastal_passes", "coastal.passes"),
    ("passes.hillslope", ""),
    // See the module note above: `state.erosion` is unshimmed and only
    // 2/16 modelled, so it is not written at all.
    ("passes.diffuse_d", ""),
    ("passes.diffuse_passes", ""),
    ("passes.sediment_fill", ""),
    ("passes.sediment_capacity", ""),
    ("passes.evolve_cycles", "stream.cycles"),
    // Turning this on is what builds the tide field the pass reads, so the
    // reference's own gate (`state.planet.tides.enabled`) is the closest
    // thing it has. `planet.tides` is `Object.assign`-shimmed, so writing
    // `enabled` alone leaves `k2` and `moons` intact.
    ("passes.tidal_flats", "planet.tides.enabled"),
    ("passes.tidal_k", ""),

    ("climate.lat_n", "climate.latN"),
    ("climate.lat_s", "climate.latS"),
    ("climate.equator_temp", "climate.equatorTemp"),
    ("climate.pole_temp", "climate.poleTemp"),
    ("climate.lapse_rate", "climate.lapseRate"),
    ("climate.albedo_k", "climate.albedo"),
    ("climate.currents", "climate.currents"),
    ("climate.current_k", "climate.currentK"),
    // Reference v1.78 deleted `state.climate.terrainWind` outright
    // (`loadZip` still runs `delete state.climate.terrainWind`), so there is
    // no key to write it to.
    ("climate.terrain_wind_deflection", ""),
    ("climate.w_iters", "climate.wIters"),
    ("climate.rain_k", "climate.rainK"),
    ("climate.evap", "climate.evap"),
    ("climate.rain_dep", "climate.rainDep"),
    ("climate.ocean", "climate.ocean"),
    // A bool here, the string `'manual'`/`'auto'` there -- written by
    // `save_state` rather than through this table, which only carries
    // same-typed values.
    ("climate.wind_manual", ""),
    ("climate.wind_dir_deg", "climate.windDir"),
    ("climate.press_k", "climate.pressK"),
    ("climate.zonal_k", "climate.zonalK"),
    ("climate.ocean_hum", "climate.oceanHum"),
    ("climate.bulk_evap", "climate.bulkEvap"),
];

/// A parameter's path inside the reference's own `state` object; `Some("")`
/// for a row this port added that the reference has no equivalent for, and
/// `None` only for a [`PARAMS`] row nobody has decided about — which
/// `every_param_has_a_reference_path_decision` fails on.
pub fn reference_path(key: &str) -> Option<&'static str> {
    JS_PATHS.iter().find(|(k, _)| *k == key).map(|(_, p)| *p)
}

/// Writes `value` at a dotted path, creating intermediate objects. A
/// non-object sitting where an intermediate object is needed is replaced —
/// this only ever builds a fresh map, so that branch is unreachable in
/// practice and exists so the function has no panic in it.
fn put(root: &mut serde_json::Map<String, serde_json::Value>, path: &str, value: serde_json::Value) {
    match path.split_once('.') {
        None => {
            root.insert(path.to_string(), value);
        }
        Some((head, rest)) => {
            let child = root.entry(head).or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
            if !child.is_object() {
                *child = serde_json::Value::Object(serde_json::Map::new());
            }
            if let Some(map) = child.as_object_mut() {
                put(map, rest, value);
            }
        }
    }
}

/// One parameter as JSON. [`Kind::Int`] rows become JSON integers so a
/// reference-app slider reading `state.tect.plates` gets `14`, not `14.0`.
///
/// A non-finite value becomes `null` rather than panicking — [`set`] rejects
/// non-finite input, so this is unreachable via the public API, but a save
/// writer is the wrong place to discover that assumption was wrong.
fn as_json(spec: &ParamSpec, p: &WorldParams) -> serde_json::Value {
    match (spec.get_fn)(p) {
        Value::Bool(b) => serde_json::Value::Bool(b),
        Value::Num(n) if spec.kind == Kind::Int => serde_json::Value::from(n as i64),
        Value::Num(n) => serde_json::Number::from_f64(n).map_or(serde_json::Value::Null, serde_json::Value::Number),
    }
}

/// The `state` object for a save (`SAVEFILE_COMPAT.md`): every parameter in
/// this table, twice — once under [`NATIVE_PARAMS_KEY`] by this port's own
/// key, once at its reference path where it has one. See the module note
/// above for why both.
///
/// Does **not** carry `GW`, `GH`, `state.tect.seed`, `state.seaLevel`,
/// `state.mapWidthKm` or `state.world` as authoritative values —
/// `cartalith_io::params_json` fills those in from the world's own
/// `SaveParams`, so the file cannot disagree with itself.
pub fn save_state(p: &WorldParams) -> serde_json::Value {
    let mut state = serde_json::Map::new();
    let mut native = serde_json::Map::new();
    for spec in PARAMS {
        let value = as_json(spec, p);
        native.insert(spec.key.to_string(), value.clone());
        match reference_path(spec.key) {
            Some(path) if !path.is_empty() => put(&mut state, path, value),
            _ => {}
        }
    }
    // The one type-changing mapping (see `JS_PATHS`).
    put(
        &mut state,
        "climate.windMode",
        serde_json::Value::from(if p.climate.wind_manual { "manual" } else { "auto" }),
    );
    state.insert(NATIVE_PARAMS_KEY.to_string(), serde_json::Value::Object(native));
    serde_json::Value::Object(state)
}

/// Restores what [`save_state`] wrote, from a save's `state` object.
/// Returns how many parameters were applied.
///
/// Reads **only** [`NATIVE_PARAMS_KEY`], never the reference paths. A
/// genuine HTML-app export carries no such key, so opening one leaves every
/// parameter at its default — exactly the behaviour this port had before a
/// writer existed, rather than a new and differently-wrong reconstruction
/// from a state object whose 200+ keys this port models a fraction of.
///
/// Every value goes through [`set`], so an out-of-range or wrong-typed entry
/// in a hand-edited (or future-version) save is clamped or rejected on the
/// same terms as a GUI write, and never panics.
pub fn apply_saved_state(p: &mut WorldParams, state: &serde_json::Value) -> usize {
    let Some(native) = state.get(NATIVE_PARAMS_KEY).and_then(|v| v.as_object()) else {
        return 0;
    };
    let mut applied = 0;
    for (key, value) in native {
        let parsed = match value {
            serde_json::Value::Bool(b) => Some(Value::Bool(*b)),
            serde_json::Value::Number(n) => n.as_f64().map(Value::Num),
            _ => None,
        };
        let Some(parsed) = parsed else { continue };
        if set(p, key, parsed) != Outcome::Rejected {
            applied += 1;
        }
    }
    applied
}

/// Every distinct `group`, in first-appearance order — the section order a
/// generated dialog should use.
pub fn groups() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for s in PARAMS {
        if !out.contains(&s.group) {
            out.push(s.group);
        }
    }
    out
}

/// The spec for a dotted key, or `None` if no such parameter exists.
pub fn spec(key: &str) -> Option<&'static ParamSpec> {
    PARAMS.iter().find(|s| s.key == key)
}

/// `GUI_GAP_REGISTER.md` **SG-03**: which node of
/// [`cartalith_engine::staleness::pipeline_stage_graph`] a moved dial has to
/// mark changed — or `None` for a parameter with **no live-apply path at
/// all**, which is most of them (56 of the 81 rows).
///
/// ## The rule the table is derived from, not a judgement call
///
/// A parameter belongs here only if some function *other than*
/// `generate_terrain` reads it, because marking a stage stale is a promise
/// that recomputing it will apply the new value. There are exactly two such
/// functions today, and each fixes one row's answer:
///
/// - [`cartalith_engine::refresh_climate`] — the whole of what
///   `recompute_stale` runs. It reads `climate_params_for` +
///   `weather_params_for` (which between them take every `climate.*` field,
///   plus `peak_m` and all three `planet.*` fields) and, directly,
///   `climate.w_iters`, `climate.zonal_k` and `climate.currents`. Those 24
///   keys map to **[`PipelineStage::Hydrology`]**.
/// - `compute_civilisation` via `WorldGen::recompute_civilisation` — reads
///   exactly one `WorldParams` field the user can move, `river_density`
///   (through `fresh_river_order`, so it reaches affordances, roads and
///   territory). That one key maps to **[`PipelineStage::Climate`]**.
///
/// The two remaining `ClimateParams`/`WeatherParams` inputs are deliberately
/// absent: `sea_level`, because `recompute_stale` is handed
/// `WorldState::sea_level` (a World-Structure archetype re-anchors it during
/// generation, so the dial is not what the recompute reads); and `world`,
/// because `WorldGen::recompute_params` pins it to the value `absorb`
/// snapshotted rather than reading the dial — a moved geometry switch must
/// not make a recompute describe a different world.
/// `params_mapping.rs`'s `every_key_that_moves_refresh_climate_is_marked`
/// derives the Hydrology half mechanically, by running `refresh_climate`
/// twice per key, so the list cannot drift from the code.
///
/// ## Why the node marked is one *above* the stage that goes stale
///
/// [`cartalith_spatial::StageGraph`] has no "this stage's own inputs moved"
/// state: `mark_changed(S)` means *S's output changed*, which makes S's
/// **consumers** stale and S itself current (`staleness.rs`'s own
/// `a_downstream_only_edit_recomputes_nothing_upstream_of_it`). So the node
/// to mark is the one immediately upstream of the shallowest stage the dial
/// actually invalidates:
///
/// - a climate dial ⇒ mark `Hydrology` ⇒ climate **and** civ go stale, and
///   `recompute_stale`'s `any_stale(clim)` gate fires, so one
///   `refresh_climate` runs. Not a fiction for the weather half —
///   `refresh_climate`'s first statement recomputes `flow_discharge` from the
///   new rainfall, which *is* hydrology's output. It is one node coarser than
///   the truth for the few temperature-only dials (`lapse_rate`, `albedo_k`),
///   where discharge does not in fact move; representing those exactly would
///   need a fifth, `params` source node, which SG-03 was briefed against the
///   existing four-node set rather than adding.
/// - `river_density` ⇒ mark `Climate` ⇒ **only** civ goes stale, and
///   `recompute_stale` runs nothing at all (neither hydrology nor climate has
///   a changed upstream), leaving `still_stale = ["civ"]` for the
///   Civilization dock's Recompute button. Marking `Civ` itself would mark
///   nothing stale — it is the leaf.
pub fn invalidates(key: &str) -> Option<PipelineStage> {
    match key {
        "river_density" => Some(PipelineStage::Climate),
        // The four non-`climate.` fields `climate_params_for`/
        // `weather_params_for` read. `sea_level` and `world` are the two
        // documented exclusions above.
        "peak_m" | "planet.g" | "planet.rotation_hours" | "planet.axial_tilt_deg" => {
            Some(PipelineStage::Hydrology)
        }
        // Every `climate.*` row — the `climate` and `weather` groups both —
        // is read by `refresh_climate`, without exception. A future row that
        // is not fails the mechanical test rather than silently promising a
        // recompute that would apply nothing.
        _ if key.starts_with("climate.") => Some(PipelineStage::Hydrology),
        _ => None,
    }
}

/// Current value of `key` on `p`, or `None` for an unknown key.
pub fn get(p: &WorldParams, key: &str) -> Option<Value> {
    spec(key).map(|s| (s.get_fn)(p))
}

/// Writes `value` into `p` at `key`.
///
/// Invalid-value policy, decided once here and identical for every parameter
/// (`GENERATION_PARAMETERS.md` states it for the GUI side too):
///
/// - **Unknown key** -> [`Outcome::Rejected`], nothing written. A typo in a
///   GDScript dialog must not silently do nothing *and* look like it worked.
/// - **Wrong type** -> [`Outcome::Rejected`]. A [`Kind::Bool`] parameter
///   takes only a real boolean (no truthy numbers); an [`Kind::Int`] or
///   [`Kind::Float`] parameter takes only a number. GDScript's own `int`
///   literals reach here as numbers, so `{"tect.warp": 1}` is fine.
/// - **Out of range** -> **clamped**, written, and reported as
///   [`Outcome::Clamped`]. Rejecting would be the purer choice, but every one
///   of these values feeds a generation kernel that has no meaningful
///   behaviour outside its range (a negative plate count, a sea level of 4.0)
///   — clamping keeps generation always well-defined, matches the precedent
///   `set_sea_level`/`generate`'s `resolution.max(4)` already set in this
///   file's own history, and is *reported*, so a dialog can read the stored
///   value back rather than assume its widget won.
/// - **NaN / infinity** -> [`Outcome::Rejected`]. Clamping a NaN silently
///   produces a NaN (`f64::clamp` panics on a NaN bound, and `NaN.max(x)` is
///   `x` — either way the result is a lie), and one NaN in the height field
///   propagates through every downstream stage (`cartalith-rust-conventions`:
///   NaN comparison differs between JS and Rust). Rejecting is the only
///   honest option.
/// - A fractional value for a [`Kind::Int`] parameter is **rounded** (and
///   reported as [`Outcome::Clamped`]), not rejected — a GDScript `Slider`
///   with a float `step` will produce `13.999999` for "14".
pub fn set(p: &mut WorldParams, key: &str, value: Value) -> Outcome {
    let Some(s) = spec(key) else { return Outcome::Rejected };
    match (s.kind, value) {
        (Kind::Bool, Value::Bool(b)) => {
            (s.set_fn)(p, if b { 1.0 } else { 0.0 });
            Outcome::Applied
        }
        (Kind::Int, Value::Num(n)) | (Kind::Float, Value::Num(n)) => {
            if !n.is_finite() {
                return Outcome::Rejected;
            }
            let mut v = n;
            if s.kind == Kind::Int {
                v = v.round();
            }
            let clamped = v.clamp(s.min, s.max);
            (s.set_fn)(p, clamped);
            // `!=` on the raw input, so a rounded int and an out-of-range
            // float both report -- the caller is told the stored value is
            // not what it sent, whichever adjustment did it.
            if clamped == n { Outcome::Applied } else { Outcome::Clamped }
        }
        _ => Outcome::Rejected,
    }
}
