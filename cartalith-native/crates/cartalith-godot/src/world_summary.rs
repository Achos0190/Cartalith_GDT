//! Whole-world summary statistics for the assistant's `get_world_summary` tool
//! (`LLM_ASSISTANT_SCOPE.md` s.5b.5, milestone LLM-8): grid size, sea level, land fraction, land
//! elevation percentiles in metres, and land mean rainfall and temperature.
//!
//! Godot-free on purpose (plain slices in, plain numbers out) so `tests/world_summary.rs` can
//! include this file by path and check it against an independent recount on real generated worlds
//! (the `tests/params_mapping.rs` pattern). `WorldGen::world_summary` (`lib.rs`) is the thin
//! `#[func]` over it.
//!
//! # Definitions (each is a labelled choice, tested)
//!
//! * **Land** is a cell whose height is at or above sea level (`field >= sea_level`), the same
//!   test `WorldGen::ecology_summary` uses (it skips `field < sea`), so the two tools agree about
//!   which cells are land. A non-finite height is no cell at all: it is neither land nor sea and
//!   is not land and is not counted
//!   (the land fraction's denominator is still the whole grid).
//! * **Land fraction** is `land_cells / (gw * gh)`. A real `0.0` for an all-sea world, which is a
//!   measured zero and not an absent value; the figures that need land to exist (percentiles,
//!   means) are `None` then, never `0` (`MISTAKES.md`: never encode "no value" as a plausible one).
//! * **Metres** follow `sample_bridge::FieldRefs::elevation_m` (reference `metersPerUnit`):
//!   `(h - sea) / (1 - sea) * peak_m`, with a `1e-6` guard when `sea == 1`. This is a THIRD copy of
//!   that three-line formula, deliberately: `sample_bridge.rs` cannot be included by a godot-free
//!   test, and the test pins this copy against the formula on literal numbers.
//! * **Percentiles** are nearest-rank over the sorted land heights: for `p` percent of `n` land
//!   cells the value at 1-based rank `ceil(p / 100 * n)` (at least 1). Heights are sorted, then
//!   converted, which is the same order as sorting the metres because the conversion is monotone
//!   when `peak_m > 0` (the registry's minimum for `peak_m` is 1).
//! * **Means** are `f64` sums over land cells of the `f32` temperature and rainfall values,
//!   divided by the land count. Rainfall is the engine's 0-1 index, not millimetres. A temperature
//!   or rainfall raster of the wrong length makes that mean `None` rather than reading a short slice.
//!
//! MUST NEVER: allocate more than one `Vec<f32>` of land heights (about 11 MB at the default
//! 2048 x 1311 grid, measured size by arithmetic, transient), write anything, or return `0` for a
//! figure that does not exist.

/// The percentiles reported, in order. Must equal `cartalith_llm::query::SUMMARY_PERCENTILES`
/// (a test in `tests/world_summary.rs` compares the two, because this file cannot import the crate
/// without making the godot-free include depend on it).
pub const PERCENTILES: [u8; 5] = [5, 25, 50, 75, 95];

/// What [`summarise`] measured.
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    /// Grid width in cells.
    pub grid_width: usize,
    /// Grid height in cells.
    pub grid_height: usize,
    /// Sea level on the 0-1 height scale (echoed).
    pub sea_level: f64,
    /// Land cells.
    pub land_cells: usize,
    /// `land_cells / (gw * gh)`.
    pub land_fraction: f64,
    /// `(percentile, metres)` over land; `None` when there is no land.
    pub elevation_m: Option<Vec<(u8, f64)>>,
    /// The highest land cell in metres; `None` when there is no land.
    pub elevation_max_m: Option<f64>,
    /// Mean rainfall over land (0-1 index); `None` when there is no land or no raster.
    pub land_rainfall_mean: Option<f64>,
    /// Mean temperature over land, degrees Celsius; `None` when there is no land or no raster.
    pub land_temperature_mean_c: Option<f64>,
}

/// Height units to metres above sea level (see the module doc).
fn to_metres(h: f64, sea_level: f64, peak_m: f64) -> f64 {
    let denom = if (1.0 - sea_level) == 0.0 { 1e-6 } else { 1.0 - sea_level };
    (h - sea_level) / denom * peak_m
}

/// Nearest-rank percentile of an ASCENDING non-empty slice: rank `ceil(p/100 * n)`, at least 1.
fn nearest_rank(sorted: &[f32], p: u8) -> f32 {
    let n = sorted.len();
    let rank = ((p as f64 / 100.0) * n as f64).ceil() as usize;
    sorted[rank.clamp(1, n) - 1]
}

/// Summarise a world. `None` when the grid is empty or `field` is not `gw * gh` long (a world that
/// is not there, not a world of zeroes).
pub fn summarise(
    field: &[f32],
    temperature: &[f32],
    rainfall: &[f32],
    sea_level: f64,
    peak_m: f64,
    gw: usize,
    gh: usize,
) -> Option<Summary> {
    let total = gw.checked_mul(gh)?;
    if total == 0 || field.len() != total || !sea_level.is_finite() || !peak_m.is_finite() {
        return None;
    }
    let have_t = temperature.len() == total;
    let have_r = rainfall.len() == total;
    let mut land: Vec<f32> = Vec::new();
    let (mut t_sum, mut r_sum) = (0f64, 0f64);
    for (i, &h) in field.iter().enumerate() {
        if !h.is_finite() || (h as f64) < sea_level {
            continue;
        }
        land.push(h);
        if have_t {
            t_sum += temperature[i] as f64;
        }
        if have_r {
            r_sum += rainfall[i] as f64;
        }
    }
    let n = land.len();
    let mut s = Summary {
        grid_width: gw,
        grid_height: gh,
        sea_level,
        land_cells: n,
        land_fraction: n as f64 / total as f64,
        elevation_m: None,
        elevation_max_m: None,
        land_rainfall_mean: None,
        land_temperature_mean_c: None,
    };
    if n == 0 {
        return Some(s);
    }
    land.sort_unstable_by(f32::total_cmp);
    s.elevation_m = Some(PERCENTILES.iter().map(|&p| (p, to_metres(nearest_rank(&land, p) as f64, sea_level, peak_m))).collect());
    s.elevation_max_m = Some(to_metres(land[n - 1] as f64, sea_level, peak_m));
    s.land_rainfall_mean = have_r.then(|| r_sum / n as f64);
    s.land_temperature_mean_c = have_t.then(|| t_sum / n as f64);
    Some(s)
}
