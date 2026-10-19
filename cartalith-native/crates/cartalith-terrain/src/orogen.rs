//! **Orogen relief** -- a ridged, dissected mountain-range height term
//! localised to the orogenic belts (owner Ruling CU item 1, 2026-10-19;
//! `MOUNTAIN_RELIEF_INVESTIGATION_2026-10-17.md` H1 at its §4 defaults).
//!
//! **Why it exists.** The investigation measured the reference height formula
//! (`compute_height`, `0.5 + a·(0.40·base + 0.50·t) + … + b·noise·(…)`) on
//! nine worlds and found that the boundary-stress / orogeny term `t` is a
//! smooth function of distance to the boundary, so every belt is one smooth
//! swell (71077345's: about 1.6 km over a 17 km half-height width, flanks
//! about 5°), and that the only rough term, `b·noise`, is uniform over the
//! map, so it does not concentrate crests in the belt: in the top decile of
//! land the median slope was 0.3-2.8° and crest cells were no denser than in
//! the lowlands. Nothing downstream (shading, detail synthesis, look) can show
//! a range that is not in the height field.
//!
//! **What it does.** Three steps, all inside the belt:
//! 1. **The belt.** `compute_height`'s own uplift term `t` (`stress` on
//!    classic worlds, the orogeny field plus the negative stress on
//!    world-structure worlds) is spread by a kilometre-sized blur
//!    ([`OrogenTuning::belt_km`]) -- the raw term is a 17 km-wide swell, far
//!    narrower than a range -- normalised by its own 99.5th percentile and
//!    passed through a smoothstep ([`belt_weight`]).
//! 2. **Relief** ([`apply_orogen_relief`]): a smooth lift of the belt
//!    ([`OrogenTuning::lift_m`]) plus a zero-mean ridged multifractal
//!    (Musgrave's, *Texturing and Modeling: A Procedural Approach*, 3rd ed.,
//!    ch. 16) over gradient noise, whose wavelengths are keyed to
//!    **kilometres** (a 3 000 km world gets the same-sized ridges as an 800 km
//!    one, in fewer cells), whose finest octave stops at two grid cells, and
//!    whose amplitude is in **metres** through the world's `metersPerUnit`
//!    anchoring (`1 - sea` maps to `peak_m`).
//! 3. **Dissection** (`cartalith_engine`'s `orogen_dissect`, which needs
//!    `cartalith-erosion` and so lives there): stream power inside the belt
//!    with the routing refreshed, which cuts the relief into dendritic valleys
//!    and spurs graded to the belt's own forelands.
//!
//! The light stream-power pass, the channel carve and the glacial pass that
//! run later in `generate_terrain` then work on this relief (on the uniform
//! swell they had almost nothing to incise: investigation §2.1, arm G).
//! [`soft_ceiling`] is H2, the smooth top that replaces the clip at 1.
//!
//! **What it must never do.**
//! - Move a cell across the land/sea split **at this stage**: a land cell is
//!   never lowered below `sea + floor_m` and a sea cell is never touched. (The
//!   end-to-end land mask still moves slightly, because later stages read the
//!   new relief -- the archetype sea-level anchor is re-taken after volcanism,
//!   and the light pass, carve and coastal passes respond; measured in the
//!   investigation's §7.)
//! - Run on the parity path: it is reached only through
//!   `TectonicParams::orogen_relief`, `false` in `WorldParams::defaults` and
//!   every JS-captured golden (`DECISIONS.md` §7p; `MISTAKES.md` "Change
//!   generated output").
//! - Read anything but its inputs, or the host's `libm`: a pure function of
//!   the field, `t`, the seed and the scale, through the vendored `js_exp` and
//!   integer hashing, so a seeded world is the same on every platform.

use cartalith_jsmath::{js_exp, smoothstep};
use cartalith_noise::{gpu_hash, pvnoise, vnoise};
use rayon::prelude::*;

/// The tuning of [`apply_orogen_relief`] and of the engine's dissection pass
/// (`cartalith_engine`'s `orogen_dissect`). None is from the reference, which
/// has no such term. Each is a labelled judgement chosen by measurement on the
/// investigation's worlds (`MOUNTAIN_RELIEF_INVESTIGATION_2026-10-17.md` §7
/// records the arms); the reason for each is beside it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrogenTuning {
    /// Peak-to-trough amplitude of the ridged term at full belt weight, in
    /// metres. 1 500: a labelled judgement between the 1 200 m and 2 200 m
    /// arms measured on 71077345 (top-decile median slope 5.5° and 7.3°, closed
    /// hollows 0.0005 and 0.0007 of the top decile before the late fill; §7):
    /// what the ridged term leaves closed, the late fill turns into flats.
    pub amp_m: f64,
    /// A smooth lift of the whole belt at full weight, in metres, so the range
    /// stands above its forelands rather than being hills on a plain. 400: at
    /// 900 m the belt crests (p99 up to 3 700 m) left no headroom under the
    /// 4 000 m ceiling for the volcanoes standing on them (§7).
    pub lift_m: f64,
    /// Half-width, in kilometres, of the box blur (three passes) that spreads
    /// the narrow uplift term into a belt. 30: the investigation measured
    /// 71077345's swell at 17 km half height, far narrower than a range, and
    /// an unspread weight put ridges on the swell's crest only (§7, first arm).
    pub belt_km: f64,
    /// Belt weight: `smoothstep(lo, hi, spread_t / p995(spread_t))`.
    pub mask_lo: f64,
    pub mask_hi: f64,
    /// The first octave's lattice spacing, in kilometres. 20: at 40 km the
    /// first octave closed wide hollows that the late fill turned into
    /// broad flats; at 20 the crests are denser and the basins smaller (on
    /// 71077345 the top-decile median slope is 7.5° at 20 km with 24
    /// dissection iterations, 5.7° at 40 km with 16; §7 and its arms).
    pub wavelength_km: f64,
    /// Musgrave's `gain` (how strongly a crest gates the next octave; his
    /// suggested 2) and `H` (octave amplitude `2^(-H·o)`; 0.9 keeps the fine
    /// octaves a little stronger than half-per-octave, for crags).
    pub gain: f64,
    pub h: f64,
    /// Domain-warp amplitude in first-octave lattice units, which bends the
    /// noise lattice's axes out of the crests.
    pub warp: f64,
    /// Ramp, in metres above sea level, over which the term fades in from the
    /// coast, and the floor a lowered land cell keeps above sea level.
    pub coast_ramp_m: f64,
    pub floor_m: f64,
    /// The dissection pass: iterations, the stream-power `K`, how often the
    /// routing is rebuilt, and the belt weight below which a cell is pinned.
    /// 24 / 0.03 / 4 (six routings): sharper valleys than 16 iterations
    /// (four routings, arm E) for two more routings, each about half a second
    /// of a release generation at 2048 x 1311 (§7.6 measured the whole term at
    /// +3.4 s), and cheaper than 30 iterations rebuilt every 3 (ten routings,
    /// arm A). 0.02: a cell with a belt weight under 2 % is foreland.
    pub dissect_iters: u32,
    pub dissect_k: f64,
    pub dissect_refresh: usize,
    pub dissect_pin_below: f32,
}

impl Default for OrogenTuning {
    fn default() -> Self {
        Self {
            amp_m: 1500.0,
            lift_m: 400.0,
            belt_km: 30.0,
            mask_lo: 0.10,
            mask_hi: 0.55,
            wavelength_km: 20.0,
            gain: 2.0,
            h: 0.9,
            warp: 0.35,
            coast_ramp_m: 300.0,
            floor_m: 10.0,
            dissect_iters: 24,
            dissect_k: 0.03,
            dissect_refresh: 4,
            dissect_pin_below: 0.02,
        }
    }
}

/// The belt weight in `[0, 1]` per cell from the uplift term `t`
/// (`compute_height`'s own `t`): its positive part spread by three box-blur
/// passes of radius `r` cells (`r = 0`: no spread; `wrap` wraps east-west),
/// then `smoothstep(lo, hi, spread / p995)`, where `p995` is the 99.5th
/// percentile of the spread field's positive cells (a percentile, not the
/// maximum, so one spike does not dim every belt). All zeros when `t` has no
/// positive cell -- a world with no compressive boundary gets no ranges.
pub fn belt_weight(t: &[f32], gw: usize, gh: usize, r: i64, wrap: bool, tuning: &OrogenTuning) -> Vec<f32> {
    let mut b: Vec<f32> = t.iter().map(|&v| v.max(0.0)).collect();
    if r > 0 {
        let mut tmp = vec![0f32; b.len()];
        for _ in 0..3 {
            crate::box_h(&b, &mut tmp, gw, gh, r, wrap);
            crate::box_v(&tmp, &mut b, gw, gh, r);
        }
    }
    let mut pos: Vec<f32> = b.iter().copied().filter(|&v| v > 0.0).collect();
    if pos.is_empty() {
        return vec![0.0; b.len()];
    }
    let k = ((pos.len() - 1) as f64 * 0.995).round() as usize;
    let (_, p, _) = pos.select_nth_unstable_by(k, |a, b| a.total_cmp(b));
    let s = (*p as f64).max(1e-9);
    b.par_iter().map(|&v| smoothstep(tuning.mask_lo, tuning.mask_hi, (v.max(0.0) as f64) / s) as f32).collect()
}

/// One value-noise sample in `[0, 1]`, periodic in x with period `px`
/// lattice cells when `px` is `Some` (world wrap), plain otherwise.
fn noise(x: f64, y: f64, s: i32, px: Option<i32>) -> f64 {
    match px {
        Some(p) => pvnoise(x, y, s, p),
        None => vnoise(x, y, s),
    }
}

/// Sixteen unit gradient directions, `(cos 22.5°·k, sin 22.5°·k)` written
/// as literals so the noise never calls the host's `libm` (`f64::cos` is not
/// guaranteed identical across platforms, and a seeded world must be).
const GRADS: [(f64, f64); 16] = [
    (1.0, 0.0),
    (0.923_879_532_511_286_7, 0.382_683_432_365_089_8),
    (0.707_106_781_186_547_6, 0.707_106_781_186_547_6),
    (0.382_683_432_365_089_8, 0.923_879_532_511_286_7),
    (0.0, 1.0),
    (-0.382_683_432_365_089_8, 0.923_879_532_511_286_7),
    (-0.707_106_781_186_547_6, 0.707_106_781_186_547_6),
    (-0.923_879_532_511_286_7, 0.382_683_432_365_089_8),
    (-1.0, 0.0),
    (-0.923_879_532_511_286_7, -0.382_683_432_365_089_8),
    (-0.707_106_781_186_547_6, -0.707_106_781_186_547_6),
    (-0.382_683_432_365_089_8, -0.923_879_532_511_286_7),
    (0.0, -1.0),
    (0.382_683_432_365_089_8, -0.923_879_532_511_286_7),
    (0.707_106_781_186_547_6, -0.707_106_781_186_547_6),
    (0.923_879_532_511_286_7, -0.382_683_432_365_089_8),
];

/// 2-D gradient (Perlin) noise in about `[-1, 1]`, periodic in x with period
/// `px` lattice cells when `px` is `Some`. Why gradient noise and not this
/// crate's value noise: a ridged term creases along the noise's zero set, and
/// value noise's bilinear lattice puts those creases on the grid axes (the
/// first build of this term drew rectilinear crests); gradient noise's zero set
/// curves. Corners hash through `gpu_hash` (wrapping `u32` arithmetic, exact on
/// every platform) into [`GRADS`], with Perlin's quintic fade (Perlin 2002,
/// "Improving noise").
fn gradient_noise(x: f64, y: f64, s: i32, px: Option<i32>) -> f64 {
    let (xf0, yf0) = (x.floor(), y.floor());
    let (fx, fy) = (x - xf0, y - yf0);
    let (xi, yi) = (xf0 as i64, yf0 as i64);
    let fade = |t: f64| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let corner = |dx: i64, dy: i64| -> f64 {
        let cx = match px {
            Some(p) => (xi + dx).rem_euclid(p as i64),
            None => xi + dx,
        };
        let g = GRADS[(gpu_hash(cx as i32, (yi + dy) as i32, s) >> 28) as usize];
        g.0 * (fx - dx as f64) + g.1 * (fy - dy as f64)
    };
    let (u, v) = (fade(fx), fade(fy));
    let a = corner(0, 0) + u * (corner(1, 0) - corner(0, 0));
    let b = corner(0, 1) + u * (corner(1, 1) - corner(0, 1));
    (a + v * (b - a)) * std::f64::consts::SQRT_2
}

/// Musgrave's ridged multifractal at lattice coordinate `(x, y)` (first
/// octave), `octaves` octaves, normalised so its maximum possible value is 1.
/// `px` is the first octave's x period (world wrap), doubled per octave.
fn ridged_multifractal(x: f64, y: f64, seed: i32, octaves: u32, px: Option<i32>, tn: &OrogenTuning) -> f64 {
    let mut freq = 1.0f64;
    let mut weight = 1.0f64;
    let mut sum = 0.0f64;
    let mut norm = 0.0f64;
    let mut p = px;
    for o in 0..octaves as i32 {
        let n = gradient_noise(x * freq, y * freq, seed.wrapping_add(o.wrapping_mul(7919)), p).clamp(-1.0, 1.0);
        let mut signal = 1.0 - n.abs();
        signal *= signal;
        signal *= weight;
        weight = (signal * tn.gain).clamp(0.0, 1.0);
        // `2^(-H·o)` through the vendored `js_exp`, not `powf`: the host's
        // libm is not guaranteed identical across platforms.
        let a = js_exp(-tn.h * o as f64 * std::f64::consts::LN_2);
        sum += signal * a;
        norm += a;
        freq *= 2.0;
        p = p.map(|v| v * 2);
    }
    sum / norm
}

/// Everything [`apply_orogen_relief`] reads besides the field.
pub struct OrogenInputs<'a> {
    pub gw: usize,
    pub gh: usize,
    pub world: bool,
    pub seed: i32,
    pub map_width_km: f64,
    pub peak_m: f64,
    /// The sea level the land/sea split is judged at (the archetype's
    /// re-anchored level on world-structure worlds).
    pub sea: f64,
    /// `compute_height`'s uplift term `t`, per cell.
    pub t: &'a [f32],
}

/// Adds the belt-localised ridged relief to `field` (see the module doc).
/// Returns the belt weight it used (all zeros when the world has no belt),
/// so the caller's dissection pass works on exactly the same belt.
///
/// # Panics
/// If `t` or `field` is not `gw * gh` long.
pub fn apply_orogen_relief(field: &mut [f32], inp: &OrogenInputs, tn: &OrogenTuning) -> Vec<f32> {
    let (gw, gh) = (inp.gw, inp.gh);
    let n = gw * gh;
    assert!(field.len() == n && inp.t.len() == n, "orogen relief: field or t is not {gw}x{gh}");
    let cell_km = inp.map_width_km / gw as f64;
    let r = (tn.belt_km / cell_km).round().max(0.0) as i64;
    let m = belt_weight(inp.t, gw, gh, r, inp.world, tn);
    if m.iter().all(|&v| v == 0.0) {
        return m;
    }
    // Metres to field units (`metersPerUnit`'s anchoring).
    let unit = |metres: f64| crate::geology::m_to_norm(metres, inp.sea, inp.peak_m);
    let amp = unit(tn.amp_m);
    let lift = unit(tn.lift_m);
    let ramp = unit(tn.coast_ramp_m).max(1e-9);
    let floor = inp.sea + unit(tn.floor_m);
    // Kilometre-keyed lattice: `f0` first-octave lattice cells across the map
    // width (a whole number, so the world-wrap period is exact), and as many
    // octaves as keep a lattice cell at least two grid cells wide.
    let f0 = (inp.map_width_km / tn.wavelength_km).round().max(1.0);
    let cells_per_lattice = gw as f64 / f0;
    // Counted, not `log2`'d, for the same cross-platform reason as `js_exp`.
    let mut octaves = 1u32;
    while octaves < 8 && cells_per_lattice / f64::from(1u32 << octaves) >= 2.0 {
        octaves += 1;
    }
    let px = inp.world.then_some(f0 as i32);
    let seed = inp.seed.wrapping_add(0x6f72); // "or": this term's own stream
    let sample = |x: usize, y: usize| -> f64 {
        let lx = x as f64 / gw as f64 * f0;
        let ly = y as f64 / gw as f64 * f0;
        // A domain warp at the first octave's own lattice (period `f0` under
        // world wrap, so the offset -- and therefore the sum -- stays periodic).
        let wx = tn.warp * (2.0 * noise(lx, ly, seed.wrapping_add(11), px) - 1.0);
        let wy = tn.warp * (2.0 * noise(lx, ly, seed.wrapping_add(23), px) - 1.0);
        ridged_multifractal(lx + wx, ly + wy, seed, octaves, px, tn)
    };
    // The term's own mean over the belt (weighted by the belt), so the term
    // raises crests and lowers valleys about the swell rather than lifting
    // the whole belt.
    let r: Vec<f32> = (0..n).into_par_iter().map(|i| if m[i] > 0.0 { sample(i % gw, i / gw) as f32 } else { 0.0 }).collect();
    let (mut sw, mut swr) = (0.0f64, 0.0f64);
    for i in 0..n {
        let w = m[i] as f64;
        sw += w;
        swr += w * r[i] as f64;
    }
    let mean = if sw > 0.0 { swr / sw } else { 0.0 };
    // Per cell: sea untouched (the `h0 < sea` early-out is also what the coast
    // ramp gives, `g = 0` below sea -- a mutation check found the two
    // equivalent, so the early-out is a statement of intent, not a second
    // guard); on land the term fades in over the coast ramp,
    // and a lowered cell keeps `floor` (or its own height, if it was already
    // below the floor), so no land cell becomes sea.
    field.par_iter_mut().enumerate().for_each(|(i, h)| {
        let h0 = *h as f64;
        if m[i] == 0.0 || h0 < inp.sea {
            return;
        }
        let g = smoothstep(0.0, 1.0, (h0 - inp.sea) / ramp);
        let d = m[i] as f64 * g * (lift + amp * (r[i] as f64 - mean));
        *h = (h0 + d).max(floor.min(h0)) as f32;
    });
    m
}

/// The knee of [`soft_ceiling`], in field units: heights at or below it are
/// untouched. 0.9 is a labelled judgement: at sea level 0.42 and `peak_m`
/// 4 000 m it is about 3 300 m, so only the highest ground is compressed (the
/// investigation's nine worlds had their 99th land percentile at 1 909-3 828 m),
/// while a summit that reached 1 keeps about a quarter of the knee band's
/// height range above the knee (`soft_ceiling(1.0)` is 0.976).
pub const SOFT_CEILING_KNEE: f64 = 0.9;

/// H2 (`MOUNTAIN_RELIEF_INVESTIGATION_2026-10-17.md` §3, §4 Q5): a smooth
/// saturation toward 1 in place of the hard clip at 1, so the highest ground
/// keeps its shape instead of being cut flat (314159's highest box was a
/// volcano summit clipped flat at 4 000 m, slope 0.0°).
///
/// `v <= k` is returned unchanged; above it, `k + (1 - k)·tanh((v - k)/(1 - k))`,
/// which is continuous with slope 1 at the knee, non-decreasing, and never
/// above 1 -- so a field that went above 1 (stacked volcano stamps, ridged
/// crests) keeps its relief order up there instead of a flat cut, until the
/// difference falls below `f32` resolution far above the knee. Must never be
/// applied on the parity path, whose reference clips hard.
pub fn soft_ceiling(v: f64) -> f64 {
    let k = SOFT_CEILING_KNEE;
    if v <= k {
        v
    } else {
        // `tanh(x) = 1 - 2 / (e^{2x} + 1)` through the vendored `js_exp`
        // (the host's `tanh` is not guaranteed identical across platforms).
        let x = (v - k) / (1.0 - k);
        k + (1.0 - k) * (1.0 - 2.0 / (js_exp(2.0 * x) + 1.0))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /// `ridged_multifractal` at three fixed points (seed 9, four octaves,
    /// default tuning), recorded from this build on 2026-10-19.
    const RIDGED_PIN: [f64; 3] = [0.653_833_276_796_069_5, 0.364_048_393_959_155_17, 0.357_546_008_054_079_1];

    /// Protects: H2's saturation curve -- identity at and below the knee,
    /// `0.9 + 0.1·tanh(1)` at 1 (a literal, not the formula re-derived), never
    /// above 1, and non-decreasing across the knee band and far above it.
    #[test]
    fn soft_ceiling_is_identity_below_the_knee_and_saturates_above() {
        assert_eq!(soft_ceiling(0.5), 0.5);
        assert_eq!(soft_ceiling(0.9), 0.9);
        assert!((soft_ceiling(1.0) - 0.976_159_415_595_576_5).abs() < 1e-12, "{}", soft_ceiling(1.0));
        let mut prev = 0.0;
        for k in 0..4000 {
            let v = 0.85 + k as f64 * 0.0005;
            let s = soft_ceiling(v);
            assert!(s >= prev && s <= 1.0, "v {v}: {s} after {prev}");
            prev = s;
        }
        assert!(soft_ceiling(50.0) <= 1.0);
    }

    /// Protects: the gradient noise tiles on a cylinder when a period is given
    /// (world wrap), to the rounding of the shifted coordinate, and stays
    /// within its stated range.
    #[test]
    fn gradient_noise_is_periodic_under_world_wrap() {
        for &(x, y) in &[(0.3, 0.7), (5.25, 2.5), (11.9, 0.01), (-3.4, 8.8)] {
            let a = gradient_noise(x, y, 17, Some(7));
            let b = gradient_noise(x + 7.0, y, 17, Some(7));
            // Equal up to the rounding of `x + 7` itself (the fractional part
            // of 7.3 is not exactly that of 0.3).
            assert!((a - b).abs() < 1e-12, "({x}, {y}): {a} vs {b}");
            assert!(a.abs() <= 1.5, "{a}");
        }
        // And not periodic without one (a different lattice cell's gradient).
        assert_ne!(gradient_noise(0.3, 0.7, 17, None), gradient_noise(7.3, 0.7, 17, None));
    }

    /// Protects: the ridged multifractal is normalised to `[0, 1]`, and its
    /// shape: three samples pinned as literals (a regression pin of the
    /// squared, weight-gated signal -- changing the octave arithmetic moves them).
    #[test]
    fn ridged_multifractal_stays_in_unit_range() {
        let tn0 = OrogenTuning::default();
        let got: Vec<f64> = [(0.37, 0.61), (2.25, 1.5), (5.1, 3.3)].iter().map(|&(x, y)| ridged_multifractal(x, y, 9, 4, None, &tn0)).collect();
        for (g, want) in got.iter().zip(RIDGED_PIN) {
            assert!((g - want).abs() < 1e-12, "{g} vs {want}");
        }
        let tn = OrogenTuning::default();
        for k in 0..500 {
            let v = ridged_multifractal(k as f64 * 0.137, k as f64 * 0.071, 9, 6, None, &tn);
            assert!((0.0..=1.0).contains(&v), "{v}");
        }
    }

    /// A `gw x gh` fixture: sea (0.3) in the left quarter, low land (0.45,
    /// about 207 m, so a lowered valley would reach the sea floor without the
    /// guard) elsewhere, sea level 0.42, and an uplift line `t = 1` down
    /// column 40.
    fn fixture(gw: usize, gh: usize) -> (Vec<f32>, Vec<f32>) {
        let field: Vec<f32> = (0..gw * gh).map(|i| if i % gw < gw / 4 { 0.3 } else { 0.45 }).collect();
        let t: Vec<f32> = (0..gw * gh).map(|i| if i % gw == 40 { 1.0 } else { 0.0 }).collect();
        (field, t)
    }

    fn inputs(gw: usize, gh: usize, t: &[f32]) -> OrogenInputs<'_> {
        // 720 km over 96 cells: 7.5 km cells, so the 30 km belt blur is a
        // radius of 4 cells and the belt ends well inside the land.
        OrogenInputs { gw, gh, world: false, seed: 5, map_width_km: 720.0, peak_m: 4000.0, sea: 0.42, t }
    }

    /// Protects: the belt is where `t` is, spread by the blur radius and no
    /// further, and a world with no positive `t` gets no belt at all.
    #[test]
    fn belt_weight_follows_the_uplift_line_and_its_spread() {
        let (gw, gh) = (96usize, 32usize);
        let (_, t) = fixture(gw, gh);
        let tn = OrogenTuning::default();
        let narrow = belt_weight(&t, gw, gh, 0, false, &tn);
        assert_eq!(narrow[16 * gw + 40], 1.0);
        assert_eq!(narrow[16 * gw + 43], 0.0);
        let wide = belt_weight(&t, gw, gh, 4, false, &tn);
        assert!(wide[16 * gw + 43] > 0.0, "the spread reaches 3 cells");
        assert_eq!(wide[16 * gw + 70], 0.0, "and not 30");
        let none = belt_weight(&vec![-1.0; gw * gh], gw, gh, 4, false, &tn);
        assert!(none.iter().all(|&v| v == 0.0));
    }

    /// Protects: the relief term's "must never" list -- sea cells untouched
    /// bit for bit, no land cell lowered below sea level, ground outside the
    /// belt untouched, the belt itself changed, the same output twice, and a
    /// world without a belt returned unchanged.
    #[test]
    fn relief_moves_only_belt_land_and_never_floods_it() {
        let (gw, gh) = (96usize, 32usize);
        let (field0, t) = fixture(gw, gh);
        let tn = OrogenTuning::default();
        let mut f = field0.clone();
        let m = apply_orogen_relief(&mut f, &inputs(gw, gh, &t), &tn);
        let mut g = field0.clone();
        apply_orogen_relief(&mut g, &inputs(gw, gh, &t), &tn);
        assert_eq!(f, g, "deterministic");
        let mut moved = 0;
        for i in 0..gw * gh {
            if field0[i] < 0.42 || m[i] == 0.0 {
                assert_eq!(f[i].to_bits(), field0[i].to_bits(), "cell {i} outside belt land moved");
            } else {
                assert!(f[i] as f64 >= 0.42, "cell {i} flooded: {}", f[i]);
                moved += (f[i] != field0[i]) as usize;
            }
        }
        assert!(moved > 100, "the belt barely moved: {moved}");
        let outside = (0..gw * gh).filter(|&i| field0[i] >= 0.42 && m[i] == 0.0).count();
        assert!(outside > 100, "the fixture must have land outside the belt: {outside}");
        let mut h = field0.clone();
        let flat = vec![0f32; gw * gh];
        let m0 = apply_orogen_relief(&mut h, &inputs(gw, gh, &flat), &tn);
        assert!(m0.iter().all(|&v| v == 0.0));
        assert_eq!(h, field0);
    }
}
