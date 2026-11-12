//! **River realism (RR, `RIVER_REALISM_SCOPE.md`; owner Ruling ES, 2026-11-12).**
//! The pure laws behind the drawn river's width, tone, turbidity, faint heads
//! and white water, in one place so the screen shader (`map_shore.gdshader`,
//! which mirrors the zoom-gated parts, held equal by tests that read its text),
//! the deep-zoom tiles and the export (`river_field::PaintSource`) and the
//! stroke (`river_stroke::for_each_span_core`) read one definition.
//!
//! **Render only, every term.** Nothing here reads or writes a world field, a
//! save key or the traced network: the inputs are the drawn runs'
//! discharge, rank and gradient, already read by `WorldGen::river_draws`, and
//! the outputs are a width, a colour and three per-point detail values. No
//! golden can move (`js_reference()` attaches no river layer).
//!
//! What it must never do: hide a river for lack of a reading (an absent
//! discharge gives `None` from the width law and the run is not given a
//! made-up width, exactly as `channel_disc`'s `None` was treated); or draw a
//! zoom-gated term (heads, white water, falls) at a zoom below its gate -- the
//! gates are what keep x1 and x4 pixel-identical where Ruling ES asks it.
//!
//! Every constant names its source: a citation (Leopold & Maddock 1953;
//! Montgomery & Buffington 1997 via Wohl & Merritt 2005), an owner ruling, an
//! RR-0 measurement (`tests/river_realism_measurement.rs`), or a labelled
//! judgement.

// ---------------------------------------------------------------------------
// RR-1: width from discharge
// ---------------------------------------------------------------------------

/// **The full drawn width, in grid cells, of the world's largest river** --
/// the one knob of RR-1 (owner Q1, Ruling ES item 1: *"Show me 1 and 2
/// cella"*). `1.0` is the scope's default: the trunk is as wide as the old
/// `channel_disc` floor drew every river on a whole-world map (half-width 0.5
/// cells), and every lesser river narrows toward the 1 px floor by
/// [`hydraulic_full_width`]. **Pending the owner's pick between 1.0 and 2.0**
/// (`LARGE_ITEM_RULINGS.md` Ruling ES): the comparison frames are in
/// `cartalith-native/docs/evidence/river-realism-2026-11-12/`. The probe sets
/// the other value through `WorldGen::set_river_width_max_cells`; a preset's
/// `river_width` still multiplies (`river_stroke::river_px_width`).
pub const W_MAX_CELLS: f64 = 1.0;

/// The downstream hydraulic-geometry exponent `b` of `w = a·Q^b`: **0.5**,
/// Leopold & Maddock 1953 (USGS PP 252; Lacey's 1930 regime value), CONFIRMED in
/// `RIVER_REALISM_SCOPE.md` §2.1. The delta split `w_a² + w_b² = w_p²`
/// (`river_delta.rs`) is this law with discharge conserved, so the fans and the
/// trunk agree by construction.
pub const HYDRAULIC_B: f64 = 0.5;

/// **RR-1's width law**: the full width in cells of a river carrying `q` when
/// the world's largest drawn river carries `q_max`, `w_max · (q / q_max)^b`
/// ([`HYDRAULIC_B`]). A relative law with one scale, because the true-scale
/// width of even the largest river is sub-pixel below x64 on a whole-world map
/// (§2.1: 0.16 cells), so "width matches volume" can only be shown as a ratio.
///
/// `None` for a reading that is not a positive finite number, or a `q_max`
/// that is not -- never a plausible width standing in for "no reading". `q`
/// above `q_max` (a caller bug) is clamped to `w_max`.
pub fn hydraulic_full_width(q: f64, q_max: f64, w_max: f64) -> Option<f64> {
    if !(q.is_finite() && q > 0.0 && q_max.is_finite() && q_max > 0.0 && w_max.is_finite() && w_max > 0.0) {
        return None;
    }
    Some(w_max * (q / q_max).min(1.0).powf(HYDRAULIC_B))
}

/// The largest finite positive discharge over every drawn point, the `Q_max`
/// of [`hydraulic_full_width`]. `None` for a network with no reading at all.
pub fn network_q_max(q_eff: &[Vec<f32>]) -> Option<f64> {
    let m = q_eff.iter().flatten().copied().filter(|q| q.is_finite() && *q > 0.0).fold(f32::NEG_INFINITY, f32::max);
    (m > 0.0).then_some(m as f64)
}

/// **One run's half-width at every drawn point, in cells (RR-1)**: half of
/// [`hydraulic_full_width`] of `q_eff` (the run's running-maximum discharge,
/// pushed down through junctions: `river_stroke::effective_discharge`), the
/// replacement for `cartalith_hydrology::river_half_width_profile`'s
/// `channel_disc` bracket in the drawing (that law stays for the intensity
/// stamp and the river entities, reference parity).
///
/// Points from `own` on (a tributary's junction cell, which is its trunk's
/// water, or a bridge target) carry the last own value, as the old profile did.
/// A head with no reading takes the first reading; a run with no own reading
/// at all is `None` and is not drawn (the old profile's `None`). Non-decreasing
/// downstream because `q_eff` is a running maximum and the law is monotone.
pub fn hydraulic_half_width_profile(q_eff: &[f32], own: usize, q_max: f64, w_max: f64) -> Option<Vec<f64>> {
    let own = own.min(q_eff.len());
    let raw: Vec<Option<f64>> = q_eff[..own].iter().map(|&q| hydraulic_full_width(q as f64, q_max, w_max).map(|w| 0.5 * w)).collect();
    let mut m = raw.iter().flatten().copied().next()?;
    let mut out: Vec<f64> = raw
        .iter()
        .map(|r| {
            if let Some(v) = *r {
                m = m.max(v);
            }
            m
        })
        .collect();
    out.resize(q_eff.len(), m);
    Some(out)
}

// ---------------------------------------------------------------------------
// RR-2: tone by volume, translucent small water, turbidity, the arid floodplain
// ---------------------------------------------------------------------------

/// The palette position (`river_stroke::palette_at` over `RIVER_ORDER_RGB`,
/// 1 = palest, 8 = darkest) the **largest** river is drawn at (`u = 0`).
/// **Labelled judgement, tuned on the RR-0 frames**: entry 6 is the
/// target render's "dark blue-green" trunk without the near-black of entries
/// 7-8, which read as ink lines over a dark forest.
pub const TONE_ORDER_TRUNK: f64 = 6.0;
/// The palette position the **least** drawn river is drawn at (`u = 1`).
/// **Labelled judgement**: entry 3.5 keeps a headwater blue rather than the
/// old order-1 near-white (156, 208, 228), which read as a pale halo; with
/// [`VOLUME_ALPHA_HEAD`]'s translucency it takes the land's tint instead.
pub const TONE_ORDER_HEAD: f64 = 3.5;
/// How quickly the tone pales with rank: `(1 - u)^γ`. **Labelled judgement**
/// (scope §RR-2: "near 1.5, so that only the top of the network is dark").
pub const TONE_GAMMA: f64 = 1.5;

/// **RR-2 part 1: the palette position of a river point by its discharge rank
/// `u`** (`0` the largest river, `1` the least), replacing the Strahler-order
/// colour: RZ-0 measured that order is not volume on this engine (26 % of the
/// owner world's top-decile discharge length is order 1). Depth `∝ Q^0.4`
/// (Leopold & Maddock) is the physical reason bigger water is darker. A NaN
/// rank reads as the trunk (the least-hidden answer, as an unranked run is
/// drawn at every zoom).
pub fn tone_order(u: f32) -> f64 {
    let u = if u.is_nan() { 0.0 } else { u.clamp(0.0, 1.0) as f64 };
    TONE_ORDER_HEAD + (TONE_ORDER_TRUNK - TONE_ORDER_HEAD) * (1.0 - u).powf(TONE_GAMMA)
}

/// The opacity of the least drawn river relative to the largest (scope §RR-2:
/// "a headwater is 65 % opaque and takes the land's tint; a trunk is
/// opaque"). **Labelled judgement** from swisstopo's 0.7-opacity water lines
/// (§2.2, CONFIRMED read).
pub const VOLUME_ALPHA_HEAD: f32 = 0.65;

/// **RR-2: the alpha multiplier of small water**, `1 - s·(1 - VOLUME_ALPHA_HEAD)·u`
/// with `s` the look's `river_translucency` (0 = every river opaque, the old
/// look; 1 = the full term). Mirrored in `map_shore.gdshader` as
/// `volume_alpha`. A NaN rank reads opaque.
pub fn volume_alpha(u: f32, strength: f32) -> f32 {
    let u = if u.is_nan() { 0.0 } else { u.clamp(0.0, 1.0) };
    1.0 - strength.clamp(0.0, 1.0) * (1.0 - VOLUME_ALPHA_HEAD) * u
}

/// The warm silt tint turbid water is pulled toward, straight RGB `0..=1`:
/// (118, 112, 78). **Labelled judgement** from the Forel-Ule scale's warm end
/// and the USGS remote-sensing page's "sediment-laden water (often brown)"
/// (§2.2).
pub const SILT_RGB: [f32; 3] = [118.0 / 255.0, 112.0 / 255.0, 78.0 / 255.0];
/// The largest share of [`SILT_RGB`] a fully turbid river takes. **Labelled
/// judgement** (scope §RR-2: `TURBID_MAX` 0.35): enough to read warm in a
/// desert, not enough to read as a road.
pub const TURBID_MAX: f32 = 0.35;
/// Which share of a world's river cells, ranked by cell gradient
/// (`cartalith_civ::landmark::landmark_gradient`), reads as clear mountain
/// water: the gradient at this percentile of the world's land channel cells is
/// the "clear" end of [`turbidity`]'s flatness term. **Relative, from RR-0**
/// (`tests/river_realism_measurement.rs`, 2026-11-12): an absolute gradient is
/// resolution-bound -- the 75th percentile at drawn points is 0.0016 on the
/// owner world's 19.6 km cells and 0.024-0.033 on the 1.17 km probe worlds, so
/// one absolute bound (first tried at 0.01) left every owner-world river turbid
/// (median turbidity 0.50, p90 1.00). The steepest quarter of a world's rivers
/// reads clear on every grid. The 0.75 is a **labelled judgement** on that table.
pub const TURBID_CLEAR_PCTL: f64 = 0.75;
/// The mean-normalised rainfall at and below which a river's country counts
/// as desert (full aridity); humid from [`RAIN_HUMID`] on.
/// **Labelled judgement** anchored on `cartalith_civ`'s `LAKE_RAIN` 0.22, the
/// reference's own wet/dry line for standing water.
pub const RAIN_DRY: f64 = 0.15;
/// See [`RAIN_DRY`]. **Labelled judgement**: twice the wet/dry line.
pub const RAIN_HUMID: f64 = 0.45;

fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// **RR-2 part 2: how turbid a river point is**, `0..=1`: the mean of its
/// flatness (`1 - smoothstep(0, clear_slope, gradient)`, `clear_slope` the
/// world's [`TURBID_CLEAR_PCTL`] gradient) and its country's aridity (`1 -
/// smoothstep(RAIN_DRY, RAIN_HUMID, rain)`), so a steep forest river is clear
/// and dark and a lowland desert one warm and pale. `None` when a reading is
/// absent (not finite, or no positive `clear_slope`): such a point is drawn
/// untinted, never at a guessed turbidity.
pub fn turbidity(gradient: f64, rain: f64, clear_slope: f64) -> Option<f32> {
    if !(gradient.is_finite() && rain.is_finite() && clear_slope.is_finite() && clear_slope > 0.0) {
        return None;
    }
    let flat = 1.0 - smooth(0.0, clear_slope, gradient.abs());
    let arid = 1.0 - smooth(RAIN_DRY, RAIN_HUMID, rain);
    Some((0.5 * flat + 0.5 * arid).clamp(0.0, 1.0) as f32)
}

/// A straight RGBA river colour pulled toward [`SILT_RGB`] by
/// `TURBID_MAX · t · strength` (`strength` the look's `river_turbidity`, 0 =
/// off by arithmetic identity). Alpha untouched.
pub fn turbid_rgba(c: [f32; 4], t: f32, strength: f32) -> [f32; 4] {
    let k = TURBID_MAX * t.clamp(0.0, 1.0) * strength.clamp(0.0, 1.0);
    if k <= 0.0 {
        return c;
    }
    [c[0] + (SILT_RGB[0] - c[0]) * k, c[1] + (SILT_RGB[1] - c[1]) * k, c[2] + (SILT_RGB[2] - c[2]) * k, c[3]]
}

// RR-2 part 3, the arid floodplain gate, lives in `render.rs`
// (`render::floodplain_arid_gate`): the post stage that applies it is there, and
// test binaries include `render.rs` on its own.


// ---------------------------------------------------------------------------
// The zoom gates (RR-3 from x4, RR-4 from x16), relative to the opening fit
// ---------------------------------------------------------------------------

/// `z` (pixels per cell over the opening fit density, Ruling EN's zoom) at and
/// below which a head is drawn at full weight -- x1 is pixel-identical with the
/// fade off (scope §RR-3 bar).
pub const HEAD_GATE_LO_Z: f32 = 1.0;
/// `z` at and above which the head fade is complete: x4 (owner Q6 default,
/// "heads fade in from x4", taken by Ruling ES).
pub const HEAD_GATE_HI_Z: f32 = 4.0;
/// `z` at and below which no white water or fall is drawn: x8, so x4 is
/// pixel-identical with the classes off (scope §RR-4 bar: "nothing drawn below
/// x16"; below it a reach is under 2 px long).
pub const REACH_GATE_LO_Z: f32 = 8.0;
/// `z` at and above which white water and falls are drawn in full: x16 (Ruling
/// ES item 3, "drawn from x16").
pub const REACH_GATE_HI_Z: f32 = 16.0;

fn smooth32(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// GLSL-identical gate weight `smoothstep(lo, hi, z)`.
pub fn gate(lo: f32, hi: f32, z: f32) -> f32 {
    smooth32(lo, hi, z)
}

/// `z` for a raster at `ppc` pixels per cell when the opening view fits at
/// `fit` (`river_stroke::scale().0`). `0` for a non-positive fit.
pub fn zoom_of(ppc: f32, fit: f32) -> f32 {
    if fit > 0.0 { ppc.max(0.0) / fit } else { 0.0 }
}

// ---------------------------------------------------------------------------
// RR-3: heads that begin faint
// ---------------------------------------------------------------------------

/// How far down a true headwater its alpha ramps from 0 to 1, in cells of arc.
/// **Labelled judgement** (scope §RR-3: "one meander wavelength",
/// `river_meander::MEANDER_WAVELENGTH_CELLS` 4).
pub const HEAD_FADE_CELLS: f64 = 4.0;

/// The head ramp at `arc` cells from a true head: `0` at the head, `1` from
/// [`HEAD_FADE_CELLS`] on, linear between.
pub fn head_ramp(arc: f64) -> f32 {
    (arc.max(0.0) / HEAD_FADE_CELLS).min(1.0) as f32
}

/// **RR-3's alpha multiplier**: a point whose head ramp is `h` at zoom `z`,
/// `1 - gate(HEAD_GATE_LO_Z, HEAD_GATE_HI_Z, z)·(1 - h)`. Mirrored in
/// `map_shore.gdshader` (`head_alpha`).
pub fn head_alpha(h: f32, z: f32) -> f32 {
    1.0 - gate(HEAD_GATE_LO_Z, HEAD_GATE_HI_Z, z) * (1.0 - h.clamp(0.0, 1.0))
}

// ---------------------------------------------------------------------------
// RR-4: rapids, cascades and falls drawn on the river
// ---------------------------------------------------------------------------

/// The step-pool lower bound: a reach at least this steep (rise over run along
/// the traced course) is rapids. Montgomery & Buffington 1997 reach types,
/// CONFIRMED via Wohl & Merritt 2005 (*WRR* 41, W08419): step-pool 0.03-0.065.
pub const RAPIDS_SLOPE: f64 = 0.03;
/// The cascade lower bound, same source: cascade > 0.065.
pub const CASCADE_SLOPE: f64 = 0.065;
/// The shortest run of steep steps drawn as white water, in traced steps.
/// **Labelled judgement** (scope §RR-4: "so a single noisy step is not white
/// water").
pub const REACH_MIN_STEPS: usize = 2;
/// The white-water detail value at the rapids bound; it rises linearly to 1 at
/// [`CASCADE_SLOPE`] ("strength ∝ gradient"). **Labelled judgement**.
pub const WHITE_AT_RAPIDS: f32 = 0.5;
/// The most a white-water reach lightens the water toward [`FOAM_RGB`] at full
/// strength. **Labelled judgement** (scope §RR-4: `WHITEWATER` 0.25).
pub const WHITEWATER: f32 = 0.25;
/// The colour of broken water and of a fall's lip, straight RGB: (236, 243,
/// 240). **Labelled judgement**: a cool off-white, so it reads as foam on the
/// water and not as the paper.
pub const FOAM_RGB: [f32; 3] = [236.0 / 255.0, 243.0 / 255.0, 240.0 / 255.0];
/// How far a fall's lip goes toward [`FOAM_RGB`] at full strength. **Labelled
/// judgement**: well past [`WHITEWATER`], so a fall reads as a mark, not a
/// rapid.
pub const FALL_LIP: f32 = 0.7;
/// A fall's mark when only the slope test holds (the landmark detector's
/// constraints, `cartalith_civ::landmark::waterfall_constraints_hold`).
/// **Labelled judgement**; raised to 1 by [`FALL_CAUSE_BOOST`].
pub const FALL_MARK_SLOPE: f32 = 0.75;
/// What a cause adds to a fall's mark: a knickpoint (a resistant cap over
/// weaker rock, GF-6) or the hanging-valley signature (scope §2.5). **Labelled
/// judgement** (scope §RR-4: "ranked higher on a knickpoint ... or a
/// tributary's last step into a much deeper trunk valley").
pub const FALL_CAUSE_BOOST: f32 = 0.25;
/// The hanging-valley signature: a tributary's last step is at least this
/// many times its own median step drop. **Labelled judgement** (scope §2.5,
/// "`k` x its own median step", proposed 3).
pub const HANGING_RATIO: f64 = 3.0;

/// **RR-4's white-water values along one traced run**: `grads[k]` the
/// along-course gradient of step `k -> k + 1` (rise over run; NaN where it
/// could not be read) and `wet[k]` whether traced point `k` is water. Returns
/// one value per traced point: `0` calm, `WHITE_AT_RAPIDS..=1` along runs of at
/// least [`REACH_MIN_STEPS`] consecutive steps at or above [`RAPIDS_SLOPE`]
/// (each step's value rising with its gradient to 1 at [`CASCADE_SLOPE`]),
/// marked on both ends of each step. Never on a water point (no rapids on a
/// lake or the sea), never from a NaN gradient.
pub fn reach_white(grads: &[f64], wet: &[bool]) -> Vec<f32> {
    let n = wet.len();
    let mut out = vec![0.0f32; n];
    let steep = |k: usize| grads.get(k).is_some_and(|g| g.is_finite() && *g >= RAPIDS_SLOPE) && !wet[k] && k + 1 < n && !wet[k + 1];
    let mut k = 0;
    while k + 1 < n {
        if !steep(k) {
            k += 1;
            continue;
        }
        let mut e = k;
        while e + 1 < n && steep(e) {
            e += 1;
        }
        if e - k >= REACH_MIN_STEPS {
            for s in k..e {
                let g = grads[s];
                let v = WHITE_AT_RAPIDS + (1.0 - WHITE_AT_RAPIDS) * (((g - RAPIDS_SLOPE) / (CASCADE_SLOPE - RAPIDS_SLOPE)).clamp(0.0, 1.0) as f32);
                out[s] = out[s].max(v);
                out[s + 1] = out[s + 1].max(v);
            }
        }
        k = e.max(k + 1);
    }
    out
}

/// Whether a run's last step is the hanging-valley signature: its drop
/// (`drops`, metres per traced step, the last entry the step onto the trunk)
/// is at least [`HANGING_RATIO`] times the median of its finite positive
/// steps. `false` for a run of fewer than three steps (no median to speak of)
/// or with no finite last step.
pub fn hanging_last_step(drops: &[f64]) -> bool {
    let Some(&last) = drops.last() else { return false };
    if !(last.is_finite() && last > 0.0) || drops.len() < 3 {
        return false;
    }
    let mut v: Vec<f64> = drops[..drops.len() - 1].iter().copied().filter(|d| d.is_finite() && *d > 0.0).collect();
    if v.is_empty() {
        return false;
    }
    v.sort_by(f64::total_cmp);
    let med = v[v.len() / 2];
    last >= HANGING_RATIO * med
}

/// **The colour a drawn river pixel takes from its detail values at zoom `z`**:
/// `rgb` the river's colour (straight `0..=1`), `white` its white-water value,
/// `lip` its fall mark. Below [`REACH_GATE_LO_Z`] this returns `rgb` exactly.
/// Mirrored in `map_shore.gdshader` (`detail_tint`).
pub fn detail_tint(rgb: [f32; 3], white: f32, lip: f32, z: f32) -> [f32; 3] {
    let g = gate(REACH_GATE_LO_Z, REACH_GATE_HI_Z, z);
    if g <= 0.0 {
        return rgb;
    }
    let k = (g * (WHITEWATER * white.clamp(0.0, 1.0)).max(FALL_LIP * lip.clamp(0.0, 1.0))).clamp(0.0, 1.0);
    [rgb[0] + (FOAM_RGB[0] - rgb[0]) * k, rgb[1] + (FOAM_RGB[1] - rgb[1]) * k, rgb[2] + (FOAM_RGB[2] - rgb[2]) * k]
}

/// The detail values a point carries when it has none: calm, no fall, full
/// head weight. What an empty `DrawnRun::detail` reads as, and what the
/// detail texture holds where no river is.
pub const DETAIL_NONE: [f32; 3] = [0.0, 0.0, 1.0];

/// RR-4's per-run counts, for the measurement harness and the probe.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ReachCounts {
    /// Traced points marked as white water (rapids or cascade).
    pub white_points: usize,
    /// Traced points drawn as a fall.
    pub falls: usize,
    /// Of those, falls on a knickpoint.
    pub falls_knick: usize,
    /// Of those, falls at the hanging-valley signature.
    pub falls_hanging: usize,
}

// ---------------------------------------------------------------------------
// The per-network inputs and the per-run evaluation (`WorldGen::river_draws`)
// ---------------------------------------------------------------------------

/// What RR-2 and RR-4 read from a finished world, once per drawn network: the
/// cell gradient as the waterfall detector reads it, the rain, the flow, the
/// height-to-metres scale, and the knickpoints the drawing may read (Ruling ES
/// item 3). Borrowed from the world; derives nothing that is stored.
pub(crate) struct NetworkInputs<'a> {
    gw: usize,
    gh: usize,
    field: &'a [f32],
    rain: &'a [f32],
    flow: &'a [f32],
    /// Per cell, `cartalith_civ::landmark::landmark_gradient` of the terrain
    /// crate's `analysis::slope` -- the waterfall detector's own gradient.
    grad: Vec<f32>,
    /// Metres per field unit (`peak_m / (1 - sea_level)`, the landmark pass's `mpu`).
    mpu: f64,
    /// One cell's side in metres.
    cell_m: f64,
    /// The canonical channel-initiation threshold (`river_flow_thresh`).
    flow_thresh: f64,
    sea: f64,
    /// `RockView::drawing_knickpoints`; `None` when the world has no column.
    knick: Option<Vec<bool>>,
    /// The world's [`TURBID_CLEAR_PCTL`] cell gradient over its land channel
    /// cells; NaN when it has none (turbidity is then not read).
    clear_slope: f64,
    /// The channel tree's receiver per cell (`-1` none), the one the waterfall
    /// detector measures a drop to; empty when the world kept no channel tree.
    recv: Vec<i32>,
}

/// One run's RR values, per point of the run's drawn points (`run_pts`: the
/// traced cells, then any extension or bridge target).
pub(crate) struct RunRealism {
    /// [`turbidity`] per point (NaN: no reading, drawn untinted).
    pub turbidity: Vec<f32>,
    /// [`reach_white`] per point.
    pub white: Vec<f32>,
    /// The fall mark per point (`0` none, else [`FALL_MARK_SLOPE`] plus
    /// [`FALL_CAUSE_BOOST`] for a cause).
    pub lip: Vec<f32>,
    /// Cells of arc from a TRUE head per point, for [`head_ramp`]; a run whose
    /// head meets water or another river reads far past the ramp everywhere.
    pub arc: Vec<f64>,
    /// What was marked, for `river_realism_stats` and the harness.
    pub counts: ReachCounts,
}

/// Far past [`HEAD_FADE_CELLS`]: the arc a point of a run that has no true
/// head reads, so its ramp is 1 everywhere. Finite, so `sample_at`'s
/// interpolation stays finite.
const NO_HEAD_ARC: f64 = 1.0e6;

impl<'a> NetworkInputs<'a> {
    /// The inputs for a generated world `ws` generated from `p`, on a `gw x gh`
    /// grid `map_width_km` wide (the shell passes its `FieldRefs`' slices; the
    /// measurement harness passes the world's own). `None` for a grid the
    /// fields do not fill (the run is then drawn with no RR-2/RR-4 values,
    /// never with guessed ones). Takes plain slices so this module depends on
    /// nothing in the cdylib and `tests/river_realism_measurement.rs` can
    /// include it by path and measure the very law the map draws.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        gw: usize,
        gh: usize,
        sea_level: f64,
        peak_m: f64,
        field: &'a [f32],
        rain: &'a [f32],
        flow: &'a [f32],
        ws: &cartalith_engine::WorldState,
        p: &cartalith_engine::WorldParams,
        map_width_km: f64,
    ) -> Option<Self> {
        let n = gw * gh;
        if n == 0 || field.len() != n || rain.len() != n || flow.len() != n || !(map_width_km > 0.0) {
            return None;
        }
        let slope = cartalith_terrain::analysis::slope(field, gw, gh);
        let grad = slope.iter().map(|&s| cartalith_civ::landmark::landmark_gradient(s, sea_level, peak_m, map_width_km) as f32).collect();
        let d = 1.0 - sea_level;
        let grad: Vec<f32> = grad;
        let clear_slope = {
            let mut g: Vec<f32> = match ws.channels.as_ref().filter(|c| c.chan.len() == n) {
                Some(c) => (0..n).filter(|&i| c.chan[i] != 0 && field[i] as f64 > sea_level).map(|i| grad[i]).filter(|v| v.is_finite()).collect(),
                None => Vec::new(),
            };
            if g.is_empty() {
                f64::NAN
            } else {
                g.sort_by(f32::total_cmp);
                g[((g.len() - 1) as f64 * TURBID_CLEAR_PCTL).round() as usize] as f64
            }
        };
        Some(NetworkInputs {
            gw,
            gh,
            field,
            rain,
            flow,
            grad,
            mpu: peak_m / if d == 0.0 { 1e-6 } else { d },
            cell_m: map_width_km * 1000.0 / gw as f64,
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km),
            sea: sea_level,
            clear_slope,
            knick: cartalith_civ::landmark::RockView::drawing_knickpoints(ws, p),
            recv: ws.channels.as_ref().filter(|c| c.recv.len() == n).map(|c| c.recv.clone()).unwrap_or_default(),
        })
    }

    /// The cell gradient at `i` (rise over run), for the harness's tables.
    #[allow(dead_code)]
    pub(crate) fn grad_at(&self, i: usize) -> f32 {
        self.grad[i]
    }

    fn cell(&self, p: (f64, f64)) -> usize {
        (p.1.max(0.0) as usize).min(self.gh - 1) * self.gw + (p.0.max(0.0) as usize).min(self.gw - 1)
    }

    /// Whether the knickpoints were readable on this world (a rock column).
    pub(crate) fn has_rock(&self) -> bool {
        self.knick.is_some()
    }

    /// RR-2 / RR-3 / RR-4 along one run: `pts` its drawn points, the first
    /// `traced` of them its traced cells; `joins_trunk` whether its last
    /// traced point is a trunk's junction cell (a tributary); `true_head`
    /// whether its head is a channel head (not a lake outlet, not a
    /// continuation); `wet[k]` whether point `k` is drawn water.
    pub(crate) fn run(&self, pts: &[(f64, f64)], traced: usize, joins_trunk: bool, true_head: bool, wet: &[bool]) -> RunRealism {
        let n = pts.len();
        let traced = traced.min(n);
        let cells: Vec<usize> = pts.iter().map(|&p| self.cell(p)).collect();
        // Along-course step gradients and drops over the traced cells.
        let mut grads = vec![f64::NAN; n];
        let mut drops = vec![f64::NAN; n];
        for k in 0..traced.saturating_sub(1) {
            let (a, b) = (pts[k], pts[k + 1]);
            let run_m = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt() * self.cell_m;
            let drop = (self.field[cells[k]] as f64 - self.field[cells[k + 1]] as f64) * self.mpu;
            drops[k] = drop;
            if run_m > 0.0 {
                grads[k] = drop / run_m;
            }
        }
        let traced_wet: Vec<bool> = (0..n).map(|k| k >= traced || wet.get(k).copied().unwrap_or(false)).collect();
        let white = reach_white(&grads, &traced_wet);
        let hanging = joins_trunk && traced >= 2 && hanging_last_step(&drops[..traced - 1]);
        let mut lip = vec![0.0f32; n];
        let mut counts = ReachCounts { white_points: white.iter().filter(|&&w| w > 0.0).count(), ..ReachCounts::default() };
        for k in 0..traced {
            if traced_wet[k] || !(self.field[cells[k]] as f64 > self.sea) {
                continue;
            }
            let c = cells[k];
            // The drop the waterfall detector measures: to the cell's channel
            // receiver (`pool_waterfall`), which is the next traced cell on a
            // traced run, so a placed waterfall is a drawn fall by construction.
            let drop = match self.recv.get(c) {
                Some(&r) if r >= 0 && (r as usize) < self.field.len() => (self.field[c] as f64 - self.field[r as usize] as f64) * self.mpu,
                Some(_) => 0.0,
                None => drops[k],
            };
            let slope_ok = cartalith_civ::landmark::waterfall_constraints_hold(drop, self.grad[c] as f64, self.flow[c] as f64, self.flow_thresh);
            let knick = self.knick.as_ref().is_some_and(|kp| kp[c]);
            let rock_ok = knick && drop >= cartalith_civ::landmark::waterfall_min_drop_m();
            if !(slope_ok || rock_ok) {
                continue;
            }
            let last = k + 2 == traced;
            let hang = slope_ok && last && hanging;
            lip[k] = FALL_MARK_SLOPE + if knick || hang { FALL_CAUSE_BOOST } else { 0.0 };
            counts.falls += 1;
            counts.falls_knick += usize::from(knick);
            counts.falls_hanging += usize::from(hang);
        }
        let last_t = traced.saturating_sub(1);
        let turbidity = (0..n)
            .map(|k| {
                let c = cells[k.min(last_t)];
                turbidity(self.grad[c] as f64, self.rain[c] as f64, self.clear_slope).unwrap_or(f32::NAN)
            })
            .collect();
        let mut arc = vec![NO_HEAD_ARC; n];
        if true_head {
            let mut acc = 0.0;
            for k in 0..n {
                if k > 0 {
                    acc += ((pts[k].0 - pts[k - 1].0).powi(2) + (pts[k].1 - pts[k - 1].1).powi(2)).sqrt();
                }
                arc[k] = acc;
            }
        }
        RunRealism { turbidity, white, lip, arc, counts }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects RR-1's law at literal values: half the discharge is 1/sqrt(2)
    /// of the width, the largest river is exactly `w_max`, and an absent
    /// reading is no width (never a plausible floor).
    #[test]
    fn the_width_is_the_square_root_of_the_discharge_share() {
        assert_eq!(hydraulic_full_width(100.0, 100.0, 1.0), Some(1.0));
        assert!((hydraulic_full_width(25.0, 100.0, 1.0).unwrap() - 0.5).abs() < 1e-12);
        assert!((hydraulic_full_width(1.0, 100.0, 2.0).unwrap() - 0.2).abs() < 1e-12);
        assert_eq!(hydraulic_full_width(f64::NAN, 100.0, 1.0), None);
        assert_eq!(hydraulic_full_width(0.0, 100.0, 1.0), None);
        assert_eq!(hydraulic_full_width(5.0, 0.0, 1.0), None);
        assert_eq!(hydraulic_full_width(500.0, 100.0, 1.0), Some(1.0));
    }

    /// Protects the RR-1 default and exponent at their ruled/cited literals
    /// (Ruling ES item 1's 1.0 pending the owner's pick; Leopold & Maddock's 0.5).
    #[test]
    fn the_width_constants_are_pinned_by_literals() {
        assert_eq!(W_MAX_CELLS, 1.0);
        assert_eq!(HYDRAULIC_B, 0.5);
    }

    /// Protects: the profile is non-decreasing downstream, a head with no
    /// reading takes the first, points past `own` carry the last own value,
    /// and a run with no reading is `None`.
    #[test]
    fn the_profile_is_monotone_and_never_invents_a_width() {
        let q = [f32::NAN, 4.0, 1.0, 16.0, 100.0];
        let p = hydraulic_half_width_profile(&q, 4, 64.0, 1.0).unwrap();
        assert_eq!(p.len(), 5);
        assert!((p[0] - 0.125).abs() < 1e-12 && (p[1] - 0.125).abs() < 1e-12 && (p[2] - 0.125).abs() < 1e-12);
        assert!((p[3] - 0.25).abs() < 1e-12);
        assert!((p[4] - 0.25).abs() < 1e-12, "the trunk's junction cell is not the run's water");
        assert_eq!(hydraulic_half_width_profile(&[f32::NAN, f32::NAN], 2, 64.0, 1.0), None);
        assert_eq!(network_q_max(&[vec![f32::NAN, 3.0], vec![7.0]]), Some(7.0));
        assert_eq!(network_q_max(&[vec![f32::NAN]]), None);
    }

    /// Protects RR-2's tone at its literals: the largest river at palette 6,
    /// the least at 3.5, and between them `(1-u)^1.5`.
    #[test]
    fn the_tone_runs_from_the_trunk_to_the_head() {
        assert_eq!(tone_order(0.0), 6.0);
        assert_eq!(tone_order(1.0), 3.5);
        assert!((tone_order(0.75) - (3.5 + 2.5 * 0.125)).abs() < 1e-9);
        assert_eq!(tone_order(f32::NAN), 6.0);
        assert_eq!(TONE_GAMMA, 1.5);
    }

    /// Protects the translucency law: opaque trunk, 65 % head at full
    /// strength, opaque everywhere at strength 0.
    #[test]
    fn small_water_is_translucent_and_the_switch_turns_it_off() {
        assert_eq!(volume_alpha(0.0, 1.0), 1.0);
        assert!((volume_alpha(1.0, 1.0) - 0.65).abs() < 1e-6);
        assert!((volume_alpha(0.5, 1.0) - 0.825).abs() < 1e-6);
        assert_eq!(volume_alpha(1.0, 0.0), 1.0);
    }

    /// Protects the turbidity law: a steep humid river is clear, a flat desert
    /// one fully turbid, and an absent reading is `None` (untinted).
    #[test]
    fn turbidity_reads_slope_and_aridity() {
        assert_eq!(turbidity(0.02, 0.9, 0.01), Some(0.0));
        assert_eq!(turbidity(0.0, 0.05, 0.01), Some(1.0));
        assert_eq!(turbidity(0.0, 0.9, 0.01), Some(0.5));
        assert_eq!(turbidity(0.005, 0.9, 0.01), Some(0.25), "half the clear slope is half flat (smoothstep midpoint)");
        assert_eq!(turbidity(f64::NAN, 0.3, 0.01), None);
        assert_eq!(turbidity(0.0, 0.3, f64::NAN), None, "no clear slope, no reading");
        let c = turbid_rgba([0.0, 0.0, 0.0, 1.0], 1.0, 1.0);
        assert!((c[0] - 0.35 * 118.0 / 255.0).abs() < 1e-6 && c[3] == 1.0);
        assert_eq!(turbid_rgba([0.2, 0.3, 0.4, 1.0], 1.0, 0.0), [0.2, 0.3, 0.4, 1.0]);
        assert_eq!(TURBID_MAX, 0.35);
        assert_eq!(TURBID_CLEAR_PCTL, 0.75);
        assert_eq!((RAIN_DRY, RAIN_HUMID), (0.15, 0.45));
    }

    /// Protects the zoom gates at their ruled zooms: nothing at x1 for heads,
    /// nothing at x4 or x8 for white water, complete at x4 and x16.
    #[test]
    fn the_gates_open_at_the_ruled_zooms() {
        assert_eq!(head_alpha(0.0, 1.0), 1.0);
        assert_eq!(head_alpha(0.0, 4.0), 0.0);
        assert_eq!(head_alpha(0.5, 4.0), 0.5);
        assert_eq!(detail_tint([0.1, 0.2, 0.3], 1.0, 1.0, 4.0), [0.1, 0.2, 0.3]);
        assert_eq!(detail_tint([0.1, 0.2, 0.3], 1.0, 1.0, 8.0), [0.1, 0.2, 0.3]);
        let t = detail_tint([0.0, 0.0, 0.0], 0.0, 1.0, 16.0);
        assert!((t[0] - 0.7 * 236.0 / 255.0).abs() < 1e-6);
        let w = detail_tint([0.0, 0.0, 0.0], 1.0, 0.0, 64.0);
        assert!((w[1] - 0.25 * 243.0 / 255.0).abs() < 1e-6);
        assert_eq!((HEAD_GATE_LO_Z, HEAD_GATE_HI_Z, REACH_GATE_LO_Z, REACH_GATE_HI_Z), (1.0, 4.0, 8.0, 16.0));
        assert_eq!(HEAD_FADE_CELLS, 4.0);
        assert_eq!(head_ramp(0.0), 0.0);
        assert_eq!(head_ramp(2.0), 0.5);
        assert_eq!(head_ramp(9.0), 1.0);
    }

    /// Protects the reach classifier: a lone steep step is not white water,
    /// two are, a cascade step reads 1, and no mark lands on water or a NaN.
    #[test]
    fn white_water_needs_a_steep_reach_on_land() {
        assert_eq!(RAPIDS_SLOPE, 0.03);
        assert_eq!(CASCADE_SLOPE, 0.065);
        assert_eq!(REACH_MIN_STEPS, 2);
        let dry = [false; 6];
        assert_eq!(reach_white(&[0.0, 0.04, 0.0, 0.0, 0.0], &dry), vec![0.0; 6]);
        let w = reach_white(&[0.0, 0.03, 0.08, 0.0, 0.0], &dry);
        assert_eq!(w[0], 0.0);
        assert!((w[1] - 0.5).abs() < 1e-6 && w[2] == 1.0 && w[3] == 1.0 && w[4] == 0.0);
        let mut wet = dry;
        wet[2] = true;
        assert_eq!(reach_white(&[0.0, 0.08, 0.08, 0.08, 0.0], &wet).iter().filter(|&&v| v > 0.0).count(), 0, "a water point splits the reach below the minimum");
        assert_eq!(reach_white(&[f64::NAN, f64::NAN, 0.0], &[false; 3]), vec![0.0; 3]);
    }

    /// Protects: `map_shore.gdshader`'s copies of every RR constant equal the
    /// Rust (GLSL cannot import Rust), the FOAM colour included, and each RR
    /// term is wired where the Rust applies it: translucency and head into
    /// `am`, the tint into `rcol`, the arid gate into both composites, and the
    /// floodplain keeping the visibility-only alpha (`floodplain_weight`'s).
    #[test]
    fn the_shader_mirrors_the_realism_law() {
        let src = include_str!("../../../godot-project/shell/map_shore.gdshader");
        let konst = |name: &str| -> f32 {
            let key = format!("const float {name} = ");
            let at = src.find(&key).unwrap_or_else(|| panic!("the shader declares no `{name}`")) + key.len();
            src[at..].split(';').next().unwrap().trim().parse().expect("a float literal")
        };
        assert_eq!(konst("VOLUME_ALPHA_HEAD"), VOLUME_ALPHA_HEAD);
        assert_eq!(konst("HEAD_GATE_LO_Z"), HEAD_GATE_LO_Z);
        assert_eq!(konst("HEAD_GATE_HI_Z"), HEAD_GATE_HI_Z);
        assert_eq!(konst("REACH_GATE_LO_Z"), REACH_GATE_LO_Z);
        assert_eq!(konst("REACH_GATE_HI_Z"), REACH_GATE_HI_Z);
        assert_eq!(konst("WHITEWATER"), WHITEWATER);
        assert_eq!(konst("FALL_LIP"), FALL_LIP);
        assert_eq!(konst("FLOODPLAIN_GREEN_EXCESS"), 0.06, "render::FLOODPLAIN_GREEN_EXCESS, held at this literal in river_field's tests");
        assert!(src.contains("const vec3 FOAM_RGB = vec3(236.0 / 255.0, 243.0 / 255.0, 240.0 / 255.0);"), "the foam colour drifted");
        assert_eq!(FOAM_RGB, [236.0 / 255.0, 243.0 / 255.0, 240.0 / 255.0]);
        assert!(src.contains("am = am * volume_alpha(u) * head_alpha(rdet.b, rz);"));
        assert!(src.contains("rcol = detail_tint(rcol, rdet.r, rdet.g, rz);"));
        assert_eq!(src.matches("* floodplain_arid_gate(").count(), 2, "both composites take the arid gate");
        assert!(src.contains("(1.0 - hwk) * am_vis *"), "the floodplain keeps the visibility-only alpha");
        assert!(src.contains(": vec3(0.0, 0.0, 1.0);"), "no detail field reads DETAIL_NONE");
        assert_eq!(DETAIL_NONE, [0.0, 0.0, 1.0]);
    }

    /// Protects the hanging-valley signature at its 3x literal.
    #[test]
    fn a_hanging_valley_is_a_last_step_three_times_the_median() {
        assert!(hanging_last_step(&[10.0, 10.0, 10.0, 30.0]));
        assert!(!hanging_last_step(&[10.0, 10.0, 10.0, 29.0]));
        assert!(!hanging_last_step(&[10.0, 40.0]));
        assert!(!hanging_last_step(&[10.0, 10.0, f64::NAN]));
        assert_eq!(HANGING_RATIO, 3.0);
        assert_eq!((FALL_MARK_SLOPE, FALL_CAUSE_BOOST, FALL_LIP, WHITEWATER, WHITE_AT_RAPIDS), (0.75, 0.25, 0.7, 0.25, 0.5));
    }
}
