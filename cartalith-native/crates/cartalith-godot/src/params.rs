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
