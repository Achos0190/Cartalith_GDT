//! **Orogen relief** -- a dissected mountain-range height term localised to
//! the orogenic belts (owner Ruling CU item 1, 2026-10-19;
//! `MOUNTAIN_RELIEF_INVESTIGATION_2026-10-17.md` H1 at its §4 defaults, and
//! that document's §7.7 for the follow-up that replaced the first build's
//! ridged term).
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
//!    ([`OrogenTuning::lift_m`]) plus **slope-aligned gullies**
//!    ([`gully_noise`]): ridges and valleys that run down the range's own
//!    regional slope, finer octaves running down the walls of coarser ones, so
//!    the range reads as a spine with branching spurs. The octaves are keyed
//!    to **kilometres** (a 3 000 km world gets the same-sized valleys as an
//!    800 km one, in fewer cells), the finest stops at two grid cells, and the
//!    amplitude is in **metres** through the world's `metersPerUnit`
//!    anchoring (`1 - sea` maps to `peak_m`).
//! 3. **Dissection** (`cartalith_engine`'s `orogen_dissect`, which needs
//!    `cartalith-erosion` and so lives there): stream power inside the belt
//!    with the routing refreshed, which deepens the valleys toward the belt's
//!    own forelands.
//!
//! The light stream-power pass, the channel carve and the glacial pass that
//! run later in `generate_terrain` then work on this relief, and the closed
//! hollows all of them leave inside the belt are drained at the very end by
//! [`breach_belt_pits`]. [`soft_ceiling`] is H2, the smooth top that replaces
//! the clip at 1.
//!
//! **What the first build did instead, and why it went** (2026-10-19, the
//! follow-up; measured in the investigation's §7.7): the relief term was a
//! ridged multifractal (Musgrave), whose crests follow the closed zero
//! contours of its noise. Its 20 km first octave drew ring-shaped crests --
//! meandering, worm-like at 3 000 km x16 -- and every ring enclosed a hollow,
//! which the late Priority-Flood fill turned into a dead-flat basin (flat
//! tan basins at x4-x16). Switching the ridged term off left 2-4x fewer flat
//! patches on its own; the gullies have no closed contour by construction.
//! The late fill became a breach for the same defect.
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
//!   the field, `t`, the seed and the scale, through the vendored `js_exp`,
//!   integer hashing and `+ - * /`, so a seeded world is the same on every
//!   platform.

use cartalith_jsmath::{js_exp, smoothstep};
use cartalith_noise::gpu_hash;
use rayon::prelude::*;

/// The tuning of [`apply_orogen_relief`], of the engine's dissection pass
/// (`cartalith_engine`'s `orogen_dissect`) and of the late breach
/// ([`breach_belt_pits`]). None is from the reference, which has no such term.
/// Each is a labelled judgement chosen by measurement on the investigation's
/// worlds (`MOUNTAIN_RELIEF_INVESTIGATION_2026-10-17.md` §7 and §7.7 record
/// the arms); the reason for each is beside it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrogenTuning {
    /// A smooth lift of the whole belt at full weight, in metres, so the range
    /// stands above its forelands rather than being hills on a plain. 400: at
    /// 900 m the belt crests (p99 up to 3 700 m) left no headroom under the
    /// 4 000 m ceiling for the volcanoes standing on them (§7).
    pub lift_m: f64,
    /// Half-width, in kilometres, of the box blur (three passes) that spreads
    /// the narrow uplift term into a belt. 30: the investigation measured
    /// 71077345's swell at 17 km half height, far narrower than a range, and
    /// an unspread weight put relief on the swell's crest only (§7, first arm).
    pub belt_km: f64,
    /// Belt weight: `smoothstep(lo, hi, spread_t / p995(spread_t))`.
    pub mask_lo: f64,
    pub mask_hi: f64,
    /// The first gully octave's lattice spacing, in kilometres: the spacing
    /// of the main valleys. 20: kept from the first build (§7), where 40 km
    /// read as too few, too wide valleys; real ranges' main valleys sit some
    /// 10-20 km apart.
    pub wavelength_km: f64,
    /// Peak-to-trough amplitude, in metres, of the gully term at full belt
    /// weight. 3 000: at 1 500 (the ridged term's amplitude) the gullies were
    /// faint beside the dissection's own texture; at 4 000 the deepest valley
    /// heads closed into more hollows (flat patches 0.0029 -> 0.0087 of land
    /// on 71077345 before the breach, §7.7).
    pub gully_amp_m: f64,
    /// The gully octaves' decay: octave `o` weighs `2^(-gully_h·o)`. 1.5: at
    /// 1.0 every octave added the same slope (each halves in height and
    /// doubles in frequency) and the fine ones drew a scribble over the spurs;
    /// at 1.5 the main valleys lead and the fine ones branch off them (§7.7).
    pub gully_h: f64,
    /// The slope -- as a fraction of the gully amplitude per first-octave
    /// lattice unit -- at which a gully octave reaches half its amplitude, so
    /// flat ground (no slope, no direction to run in) gets almost none. 0.2:
    /// at 0.5 the flanks away from the spine stayed nearly smooth.
    pub gully_slope_ref: f64,
    /// The blur of the regional slope the gullies follow, as a fraction of
    /// the first octave's lattice, so `compute_height`'s uniform `b·noise`
    /// roughness does not steer them. 0.25: a labelled judgement.
    pub gully_base_blur: f64,
    /// A dome of the belt's weight, in metres, added to the field for the
    /// gullies' **direction only** (never to the height): the belt's 400 m lift
    /// has a flat top, where the regional slope -- and so the gullies'
    /// direction -- was noise. 1 500: spurs run off the spine across the whole
    /// belt instead of only near the original swell (§7.7, arms g7-g8).
    pub gully_dir_lift_m: f64,
    /// Ramp, in metres above sea level, over which the term fades in from the
    /// coast, and the floor a lowered land cell keeps above sea level.
    pub coast_ramp_m: f64,
    pub floor_m: f64,
    /// The dissection pass: iterations, the stream-power `K`, how often the
    /// routing is rebuilt, and the belt weight below which a cell is pinned.
    /// 24 / 0.03 / 4 (six routings): sharper valleys than 16 iterations
    /// (four routings, arm E) for two more routings, each about half a second
    /// of a release generation at 2048 x 1311 (§7.6 measured the whole term at
    /// +3.4 s; owner Ruling CV accepted the 24 iterations), and cheaper than
    /// 30 iterations rebuilt every 3 (ten routings, arm A). 0.02: a cell with
    /// a belt weight under 2 % is foreland.
    pub dissect_iters: u32,
    pub dissect_k: f64,
    pub dissect_refresh: usize,
    pub dissect_pin_below: f32,
    /// [`breach_belt_pits`]' deepest notch, in metres below the sill. 50: with
    /// no cap a notch reached 850 m and drew a straight trench across a range
    /// at x64 (314159 earth, §7.7); at 50 m no trench reads, and only a
    /// hollow deeper than that keeps a filled (flat) floor, 50 m below its
    /// sill (flat-patch share 0.0001-0.0063 of land against 0.0004-0.0302
    /// with the fill).
    pub breach_max_depth_m: f64,
}

impl Default for OrogenTuning {
    fn default() -> Self {
        Self {
            lift_m: 400.0,
            belt_km: 30.0,
            mask_lo: 0.10,
            mask_hi: 0.55,
            wavelength_km: 20.0,
            gully_amp_m: 3000.0,
            gully_h: 1.5,
            gully_slope_ref: 0.2,
            gully_base_blur: 0.25,
            gully_dir_lift_m: 1500.0,
            coast_ramp_m: 300.0,
            floor_m: 10.0,
            dissect_iters: 24,
            dissect_k: 0.03,
            dissect_refresh: 4,
            dissect_pin_below: 0.02,
            breach_max_depth_m: 50.0,
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

/// `(cos 2πx, sin 2πx)` from `+ - * /` and two `round`s only, so a seeded
/// world is the same on every platform (the host's `f64::cos` is not
/// guaranteed identical across platforms; the vendored `js_cos` is, but costs
/// more than this term can afford at 25 kernel points per cell and octave).
/// The argument is reduced to `[-1/2, 1/2]` turns, then about the nearest
/// quarter turn to `[-1/8, 1/8]` turns, where the degree-10 / 11 Taylor
/// polynomials below are accurate to about `1e-10` -- far below what an `f32`
/// height can hold.
fn cos_sin_turns(x: f64) -> (f64, f64) {
    let r = x - x.round();
    let q = (r * 4.0).round();
    let a = (r - q * 0.25) * std::f64::consts::TAU;
    let a2 = a * a;
    let c = 1.0 - a2 / 2.0 * (1.0 - a2 / 12.0 * (1.0 - a2 / 30.0 * (1.0 - a2 / 56.0 * (1.0 - a2 / 90.0))));
    let s = a * (1.0 - a2 / 6.0 * (1.0 - a2 / 20.0 * (1.0 - a2 / 42.0 * (1.0 - a2 / 72.0 * (1.0 - a2 / 110.0)))));
    // Rotate back by the quarter turns taken out (`q` is -2..=2; ±2 are the
    // same half turn).
    match q as i64 {
        0 => (c, s),
        1 => (-s, c),
        -1 => (s, -c),
        _ => (-c, -s),
    }
}

/// The squared radius, in lattice units, of [`gully_noise`]'s kernel across
/// the slope (1.25²). A labelled judgement: wide enough that two or three
/// stripes overlap at every point (so the blend reads as branching gullies,
/// not one stripe per cell), with the reach along the slope set by
/// [`GULLY_STRETCH`].
const GULLY_R2: f64 = 1.5625;

/// How much farther [`gully_noise`]'s kernel reaches along the slope than
/// across it, so one point's stripe draws a gully about that much longer than
/// it is wide. 1.6: a labelled judgement; with it the kernel's reach
/// (1.25 · 1.6 = 2 lattice units) is covered exactly by the 5 x 5 block of
/// jittered points [`gully_noise`] visits (a point two cells out is at least
/// 2.25 units away). Raising it past 1.6 needs a wider block.
const GULLY_STRETCH: f64 = 1.6;

/// The largest turn, in turns either way, by which a point's stripe is set off
/// the local slope, and the spread of the points' strengths (`1 - VARY..1`).
/// Without them a uniform regional slope drew a comb of equally spaced,
/// parallel, equal gullies (314159's belt at x4, §7.7). 0.06 turns (22°) and
/// 0.5: labelled judgements, read in the frames.
const GULLY_TURN: f64 = 0.06;
const GULLY_VARY: f64 = 0.5;

/// One octave of **gully noise** at lattice coordinate `(x, y)`: ridges and
/// valleys whose crest lines run along the unit direction `along` (the local
/// downslope direction), spaced about one lattice unit apart across it.
/// Returns the value in `[-1, 1]` and its gradient in lattice units.
///
/// The construction is the "erosion noise" of the procedural-terrain
/// literature (Íñigo Quílez's slope-aligned gullies, popularised as Clay
/// John's *Eroded Terrain Noise*, Shadertoy, 2020; Rune Skovbo Johansen's
/// *Advanced Terrain Erosion Filter*, 2025 -- cited from memory, not opened in
/// this pass): each jittered lattice point carries a cosine stripe across
/// `along`, and the stripes of the nearby points are blended with a compact
/// smooth weight, elongated along the slope. A stripe aligned with the slope
/// has no closed contour, so the term cannot enclose a hollow the way the
/// ridged term it replaced did (see the module doc).
///
/// `px` is the lattice's x period under world wrap.
fn gully_noise(x: f64, y: f64, along: (f64, f64), seed: i32, px: Option<i32>) -> (f64, f64, f64) {
    let (xi, yi) = (x.floor() as i64, y.floor() as i64);
    let (mut v, mut gx, mut gy, mut wt) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for dy in -2..=2i64 {
        for dx in -2..=2i64 {
            let (cx, cy) = (xi + dx, yi + dy);
            let hx = match px {
                Some(p) => cx.rem_euclid(p as i64),
                None => cx,
            };
            // The feature point of lattice cell (cx, cy), jittered within the
            // cell's middle half, `[0.25, 0.75)` on each axis (two streams of
            // one hash), so the own cell's point is never more than 1.07
            // units away and the weight sum is never zero.
            let jx = 0.25 + 0.5 * (gpu_hash(hx as i32, cy as i32, seed) as f64 / 4_294_967_296.0);
            let jy = 0.25 + 0.5 * (gpu_hash(hx as i32, cy as i32, seed.wrapping_add(101)) as f64 / 4_294_967_296.0);
            let (ddx, ddy) = (x - (cx as f64 + jx), y - (cy as f64 + jy));
            // The point's own turn off the slope and its own strength (see
            // `GULLY_TURN`), from a third stream of the hash.
            let hr = gpu_hash(hx as i32, cy as i32, seed.wrapping_add(211));
            let turn = GULLY_TURN * (2.0 * ((hr & 0xffff) as f64 / 65_536.0) - 1.0);
            let strength = 1.0 - GULLY_VARY * ((hr >> 16) as f64 / 65_536.0);
            let (ct, st) = cos_sin_turns(turn);
            let al = (along.0 * ct - along.1 * st, along.0 * st + along.1 * ct);
            let across = (-al.1, al.0);
            let ua = ddx * al.0 + ddy * al.1;
            let ux = ddx * across.0 + ddy * across.1;
            let d2 = ux * ux + ua * ua / (GULLY_STRETCH * GULLY_STRETCH);
            // A compact Gaussian stand-in, `(1 - d²/R²)³` for `d < R`: smooth,
            // zero outside, and only `+ *` (no `exp`).
            if d2 >= GULLY_R2 {
                continue;
            }
            let k = 1.0 - d2 / GULLY_R2;
            let w = k * k * k;
            let (c, s) = cos_sin_turns(ux);
            v += w * strength * c;
            // d/dp cos(2π·ux) = -2π·sin(2π·ux)·across (the weight's own
            // derivative left out, as the literature does: it only rounds the
            // stripes' ends).
            gx -= w * strength * s * std::f64::consts::TAU * across.0;
            gy -= w * strength * s * std::f64::consts::TAU * across.1;
            wt += w;
        }
    }
    if wt <= 0.0 {
        return (0.0, 0.0, 0.0);
    }
    (v / wt, gx / wt, gy / wt)
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

/// Adds the belt-localised gully relief to `field` (see the module doc).
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
    let lift = unit(tn.lift_m);
    let ramp = unit(tn.coast_ramp_m).max(1e-9);
    let floor = inp.sea + unit(tn.floor_m);
    // Kilometre-keyed lattice: `f0` first-octave lattice cells across the map
    // width (a whole number, so the world-wrap period is exact), and as many
    // octaves as keep a lattice cell at least two grid cells wide.
    let f0 = (inp.map_width_km / tn.wavelength_km).round().max(1.0);
    let cells_per_lattice = gw as f64 / f0;
    // Counted, not `log2`'d, for the cross-platform reason in the module doc.
    let mut octaves = 1u32;
    while octaves < 8 && cells_per_lattice / f64::from(1u32 << octaves) >= 2.0 {
        octaves += 1;
    }
    let px = inp.world.then_some(f0 as i32);
    let seed = inp.seed.wrapping_add(0x6f72); // "or": this term's own stream
    // Octave `o` weighs `2^(-gully_h·o)` (through the vendored `js_exp`, not
    // `powf`), normalised so the octaves' peak-to-trough sum is `gully_amp_m`.
    let g_amp = unit(tn.gully_amp_m);
    let g_w: Vec<f64> = (0..octaves).map(|o| js_exp(-tn.gully_h * o as f64 * std::f64::consts::LN_2)).collect();
    let g_norm: f64 = 2.0 * g_w.iter().sum::<f64>();
    let g_w: Vec<f64> = g_w.iter().map(|w| g_amp * w / g_norm).collect();
    let slope_ref = (g_amp * tn.gully_slope_ref).max(1e-12);
    // The regional slope the gullies follow: the field plus a dome of the belt
    // weight (direction only, see `gully_dir_lift_m`), blurred so the uniform
    // `b·noise` roughness does not steer them, differenced per cell and
    // expressed per first-octave lattice unit (the unit the octave loop
    // accumulates in).
    let (bgx, bgy) = {
        let dl = unit(tn.gully_dir_lift_m);
        let mut b: Vec<f32> = (0..n).map(|i| (field[i] as f64 + m[i] as f64 * dl) as f32).collect();
        let rb = (cells_per_lattice * tn.gully_base_blur).round().max(1.0) as i64;
        let mut tmp = vec![0f32; n];
        for _ in 0..3 {
            crate::box_h(&b, &mut tmp, gw, gh, rb, inp.world);
            crate::box_v(&tmp, &mut b, gw, gh, rb);
        }
        let k = cells_per_lattice / 2.0;
        let at = |x: i64, y: i64| -> f64 {
            let x = if inp.world { x.rem_euclid(gw as i64) } else { x.clamp(0, gw as i64 - 1) };
            b[y.clamp(0, gh as i64 - 1) as usize * gw + x as usize] as f64
        };
        let gx: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|i| {
                let (x, y) = ((i % gw) as i64, (i / gw) as i64);
                ((at(x + 1, y) - at(x - 1, y)) * k) as f32
            })
            .collect();
        let gy: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|i| {
                let (x, y) = ((i % gw) as i64, (i / gw) as i64);
                ((at(x, y + 1) - at(x, y - 1)) * k) as f32
            })
            .collect();
        (gx, gy)
    };
    let sample = |x: usize, y: usize| -> f64 {
        let lx = x as f64 / gw as f64 * f0;
        let ly = y as f64 / gw as f64 * f0;
        let i = y * gw + x;
        let (mut gx, mut gy) = (bgx[i] as f64, bgy[i] as f64);
        let mut total = 0.0;
        let mut f = 1.0f64;
        let mut pf = px;
        for (o, &w) in g_w.iter().enumerate() {
            // Each octave follows the slope so far -- the regional slope plus
            // every coarser octave's own -- so a fine octave runs down the
            // walls of the coarse gullies: branching, not parallel combs.
            // Flat ground (`s` far under `slope_ref`) gets almost none.
            let s = (gx * gx + gy * gy).sqrt();
            let along = if s > 0.0 { (gx / s, gy / s) } else { (1.0, 0.0) };
            let (v, dvx, dvy) = gully_noise(lx * f, ly * f, along, seed.wrapping_add(1000 + o as i32 * 7919), pf);
            let a = w * s / (s + slope_ref);
            total += a * v;
            gx += a * dvx * f;
            gy += a * dvy * f;
            f *= 2.0;
            pf = pf.map(|p| p * 2);
        }
        total
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
        let d = m[i] as f64 * g * (lift + (r[i] as f64 - mean));
        *h = (h0 + d).max(floor.min(h0)) as f32;
    });
    m
}

/// Drains every closed hollow of the belt by **breaching** -- lowering the
/// cells on the hollow's outlet path until it descends to fixed ground --
/// instead of raising the hollow to its spill level (the 2026-10-19
/// follow-up to Ruling CU item 1, defect (a): a filled hollow is a dead-flat
/// surface, and drew as a flat tan basin at x4-x16).
///
/// The method is Lindsay's hybrid breaching over Priority-Flood (Lindsay,
/// *Efficient hybrid breaching-filling sink removal methods for flow path
/// enforcement in digital elevation models*, Hydrological Processes 30, 2016;
/// Barnes, Lehman & Mulla, *Priority-flood*, Computers & Geosciences 62, 2014
/// -- both cited from memory, not opened in this pass). The flood is seeded
/// from the fixed ground around the region, as
/// [`crate::edifice_pits::fill_edifice_flank_pits`] seeds it, and each cell
/// remembers the cell it was reached from (so its path runs through the
/// lowest sill, by the flood's order). A cell reached at or below that cell's
/// height is a hollow floor: the path back from it is lowered one `f32` step
/// per cell below it until it meets ground already lower. The hollow keeps
/// its own floor and walls; a one-cell notch is cut through the sill.
///
/// Three cases are **filled** instead, each for a measured reason:
/// - a floor more than `max_depth` (field units) below the highest ground on
///   its path is first raised to `sill - max_depth`, so no notch is deeper
///   than that (uncapped, notches reached 850 m and drew straight trenches
///   across a range at x64);
/// - a floor at or just above the sea (one step per path cell) is raised one
///   step above the cell it was reached from, the fill this replaces: a notch
///   from it would dip under the sea and open the hollow to it as an inlet;
/// - a floor below the fixed ground its path ends on is raised to one step per
///   path cell above that ground, so the notch never has to dig the fixed
///   ground.
///
/// **Never** touches a cell whose `mask` is not
/// [`crate::edifice_pits::FLANK_FLANK`] (so the sea, the ground outside the
/// belt and every summit caldera keep their bytes), and never lowers a cell
/// below the sea. `mask` is the late belt mask the engine builds.
/// Connectivity is the water-body classifier's (4-connected), as in the fill.
/// Returns `(cells lowered, cells raised)`.
pub fn breach_belt_pits(gw: usize, gh: usize, field: &mut [f32], mask: &[u8], sea: f64, max_depth: f64) -> (usize, usize) {
    use crate::edifice_pits::FLANK_FLANK;
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    let n = gw * gh;
    assert!(field.len() >= n && mask.len() >= n, "breach_belt_pits: short field");
    const NONE: u32 = u32::MAX;
    let mut closed = vec![false; n];
    let mut parent = vec![NONE; n];
    // The fixed ground each cell's path ends on (its height's bits), the
    // path's length to it, and the highest ground (original heights) on the
    // path: the sill a hollow reached through it is notched through.
    let mut root_bits = vec![0u32; n];
    let mut depth = vec![0u32; n];
    let mut pass = vec![0f32; n];
    let mut heap: BinaryHeap<Reverse<(u32, usize)>> = BinaryHeap::new();
    let nb = |i: usize| -> [Option<usize>; 4] {
        let (x, y) = (i % gw, i / gw);
        [(x > 0).then(|| i - 1), (x + 1 < gw).then(|| i + 1), (y > 0).then(|| i - gw), (y + 1 < gh).then(|| i + gw)]
    };
    // Heights are non-negative after generation's clamp, so `max(0)`'s bit
    // pattern orders them as `f32` does (one step is one bit).
    let bits = |v: f32| v.max(0.0).to_bits();
    let sea_bits = bits(sea as f32);
    for i in 0..n {
        let ns = nb(i);
        let seed = if mask[i] == FLANK_FLANK { ns.iter().any(|j| j.is_none()) } else { ns.iter().flatten().any(|&j| mask[j] == FLANK_FLANK) };
        if seed {
            closed[i] = true;
            root_bits[i] = bits(field[i]);
            pass[i] = field[i];
            heap.push(Reverse((bits(field[i]), i)));
        }
    }
    let (mut lowered, mut raised) = (0usize, 0usize);
    while let Some(Reverse((_, c))) = heap.pop() {
        for j in nb(c).into_iter().flatten() {
            if closed[j] || mask[j] != FLANK_FLANK {
                continue;
            }
            closed[j] = true;
            parent[j] = c as u32;
            root_bits[j] = root_bits[c];
            depth[j] = depth[c] + 1;
            pass[j] = pass[c].max(field[j]);
            // The depth cap: a floor more than `max_depth` under the sill is
            // raised to `sill - max_depth` (land only; a below-sea floor is
            // the sea case below).
            let floor_min = (pass[c] as f64 - max_depth) as f32;
            if field[j] < floor_min && field[j] as f64 >= sea {
                field[j] = floor_min;
                raised += 1;
            }
            let (bj, bc) = (bits(field[j]), bits(field[c]));
            if bj <= bc && bj < sea_bits.saturating_add(depth[j]).saturating_add(1) {
                // The sea case: filled one step above the cell it was reached
                // from -- the Priority-Flood+ε fill this replaces.
                field[j] = f32::from_bits(bc + 1);
                raised += 1;
            } else if bj <= bc {
                // A hollow floor. The notch descends one step per cell from
                // `j`, so `j` must sit at least `depth[j]` steps above the
                // path's fixed ground: raise it there if it does not.
                let need = root_bits[j].saturating_add(depth[j]);
                let mut t = bj;
                if t < need {
                    t = need;
                    field[j] = f32::from_bits(t);
                    raised += 1;
                }
                // Lower the path below `j`, one step per cell, until a cell
                // is already lower or the fixed ground is reached (which
                // `need` makes lower). `<=`, not `<`: a cell already AT `t`
                // needs nothing, and its parent is lower still (every popped
                // cell's path descends), so `<` would only take one more,
                // empty step -- a mutation check (2026-10-19) found the two
                // equivalent; `<=` states the intent.
                let mut m = c;
                loop {
                    t -= 1;
                    if mask[m] != FLANK_FLANK || bits(field[m]) <= t {
                        break;
                    }
                    field[m] = f32::from_bits(t);
                    lowered += 1;
                    let p = parent[m];
                    if p == NONE {
                        break;
                    }
                    m = p as usize;
                }
            }
            heap.push(Reverse((bits(field[j]), j)));
        }
    }
    (lowered, raised)
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
/// above 1 -- so a field that went above 1 (stacked volcano stamps, high
/// crests) keeps its relief order up there instead of a flat cut, until the
/// difference falls below `f32` resolution far above the knee. Must never be
/// applied on the parity path, whose reference clips hard.
pub fn soft_ceiling(v: f64) -> f64 {
    soft_ceiling_to(v, 1.0)
}

/// The headroom of the **higher world ceiling** (owner Ruling DB item 3,
/// 2026-10-25; `TectonicParams::high_ceiling`), as a multiple of the world's
/// land range above the sea: the top is [`high_ceiling_top`], `1 + (1 -
/// sea)·HEADROOM` in field units, so in metres (`(h - sea)/(1 - sea)·peak_m`)
/// it is `(1 + HEADROOM)·peak_m` on every world whatever its sea level.
///
/// With it on, [`soft_ceiling_to`] saturates toward that top instead of 1,
/// so a tall cone stamped on ground already near 1 (71077345's stratocone
/// #15: about 0.78 units on a plateau at about 1) keeps its shape instead of
/// being squeezed into the last thousandths below 1, where `f32` rounds it
/// to a flat cap edged by a contour of the ground beneath.
///
/// 1.0 (a top at twice `peak_m`: 8 000 m at the default 4 000 m, against
/// Everest's 8 849 m) is a labelled judgement, not a measurement. A fixed
/// top in field units was tried first, 2.0 (2026-10-25,
/// `summit_ceiling_census`): on the Hawaii-like seed 8, whose World
/// Structure sea level is 0.858, the same 0.85-unit shield stamps then
/// reached 1.80, about 26 600 m, because one field unit there is 28 000 m;
/// relative to the sea the top stays a mountain-sized number everywhere.
/// Only heights above the knee (0.9) change; the knee itself is unchanged.
pub const HIGH_CEILING_HEADROOM: f64 = 1.0;

/// The higher world ceiling's top, in field units, for a world whose sea
/// level is `sea`: `1 + (1 - sea)·`[`HIGH_CEILING_HEADROOM`]. Never below 1
/// (a sea at or above 1 leaves the old top), so never at or below the knee.
pub fn high_ceiling_top(sea: f64) -> f64 {
    1.0 + (1.0 - sea).max(0.0) * HIGH_CEILING_HEADROOM
}

/// The top a kernel or the tile synthesis clamps the height field to: 1 for every field that is at
/// or under 1 on entry (every field the reference produces, which clips at 1,
/// and every app field without Ruling DB's higher ceiling), else the field's
/// own maximum on entry.
///
/// Why: the reference's kernels clamp to `0..1` because the field never
/// went above 1. Under the higher world ceiling (owner Ruling DB item 3,
/// `TectonicParams::high_ceiling`, `orogen::high_ceiling_top`) summits sit
/// above 1, and a clamp at 1 would cut each one back to the flat cap the
/// ruling removes. Clamping at the entry maximum instead lets no pass push
/// the world higher than it found it, and cuts no summit it did not erode.
///
/// Must never return less than 1: with the field at or under 1 the result is
/// exactly 1.0, so every reference-path clamp keeps its bits by control flow.
pub fn field_top(fld: &[f32]) -> f32 {
    fld.iter().fold(1.0f32, |m, &v| if v > m { v } else { m })
}

/// [`soft_ceiling`] saturating toward `top` instead of 1:
/// `v <= k` unchanged, above it `k + (top - k)·tanh((v - k)/(top - k))`,
/// continuous with slope 1 at the knee, non-decreasing, never above `top`.
/// `top` 1.0 is [`soft_ceiling`] exactly (the same expression, so the same
/// bits); [`high_ceiling_top`] is Ruling DB's higher ceiling. Must never be
/// called with `top <= SOFT_CEILING_KNEE` (the band would be empty or
/// inverted); the engine passes only 1.0 or [`high_ceiling_top`].
pub fn soft_ceiling_to(v: f64, top: f64) -> f64 {
    let k = SOFT_CEILING_KNEE;
    if v <= k {
        v
    } else {
        // `tanh(x) = 1 - 2 / (e^{2x} + 1)` through the vendored `js_exp`
        // (the host's `tanh` is not guaranteed identical across platforms).
        let x = (v - k) / (top - k);
        k + (top - k) * (1.0 - 2.0 / (js_exp(2.0 * x) + 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edifice_pits::{FLANK_FLANK, FLANK_NONE};

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

    /// Protects: Ruling DB item 3's higher ceiling -- the headroom is the
    /// literal 1.0 and the top `1 + (1 - sea)` (literals at two sea levels,
    /// not the formula re-derived), heights at or below the knee are
    /// untouched, `top` 1.0 is `soft_ceiling` bit for bit, and under the
    /// higher top the curve is non-decreasing, never above the top, and keeps
    /// a cone's relief: a summit stamped at 1.73 stands well above ground at
    /// 1.0 instead of both rounding to the old cap.
    #[test]
    fn the_higher_ceiling_keeps_summits_apart_and_leaves_the_knee() {
        assert_eq!(HIGH_CEILING_HEADROOM, 1.0);
        assert!((high_ceiling_top(0.42) - 1.58).abs() < 1e-12);
        assert!((high_ceiling_top(0.858) - 1.142).abs() < 1e-12);
        assert_eq!(high_ceiling_top(1.2), 1.0);
        let top = high_ceiling_top(0.42);
        assert_eq!(soft_ceiling_to(0.9, top), 0.9);
        assert_eq!(soft_ceiling_to(0.42, top), 0.42);
        for k in 0..400 {
            let v = 0.85 + k as f64 * 0.01;
            assert_eq!(soft_ceiling_to(v, 1.0).to_bits(), soft_ceiling(v).to_bits(), "v {v}");
        }
        let mut prev = 0.0;
        for k in 0..4000 {
            let v = 0.85 + k as f64 * 0.001;
            let s = soft_ceiling_to(v, top);
            assert!(s >= prev && s <= top, "v {v}: {s} after {prev}");
            prev = s;
        }
        let (ground, summit) = (soft_ceiling_to(1.0, top), soft_ceiling_to(1.73, top));
        assert!(summit - ground > 0.3, "ground {ground}, summit {summit}");
        // Under the old top both sit within 0.025 of 1.
        assert!(soft_ceiling(1.73) - soft_ceiling(1.0) < 0.025);
    }

    /// Protects: the libm-free cosine and sine agree with the host's to
    /// `1e-9` over several turns either side of zero, including the quarter
    /// turns the reduction pivots on (an error in a polynomial coefficient or
    /// in the quadrant table fails here).
    #[test]
    fn cos_sin_turns_matches_the_trig_functions() {
        let mut worst = 0.0f64;
        for k in -4000..=4000 {
            let x = k as f64 * 0.000_731_3 * 7.0;
            let (c, s) = cos_sin_turns(x);
            let a = x * std::f64::consts::TAU;
            worst = worst.max((c - a.cos()).abs()).max((s - a.sin()).abs());
        }
        for &x in &[0.0, 0.125, 0.25, 0.375, 0.5, -0.25, 0.75, 1.0, 2.5] {
            let (c, s) = cos_sin_turns(x);
            let a = x * std::f64::consts::TAU;
            worst = worst.max((c - a.cos()).abs()).max((s - a.sin()).abs());
        }
        assert!(worst < 1e-9, "worst error {worst}");
    }

    /// Protects: the gully noise tiles on a cylinder when a period is given
    /// (world wrap) and stays within its stated range, and its stripes run
    /// along `along`: stepping along the slope changes the value far less
    /// than stepping across it (the property that makes valleys run
    /// downhill).
    #[test]
    fn gully_noise_wraps_and_runs_along_the_slope() {
        for &(x, y) in &[(0.3, 0.7), (5.25, 2.5), (11.9, 0.01), (-3.4, 8.8)] {
            let a = gully_noise(x, y, (0.6, 0.8), 17, Some(7));
            let b = gully_noise(x + 7.0, y, (0.6, 0.8), 17, Some(7));
            assert!((a.0 - b.0).abs() < 1e-9, "({x}, {y}): {} vs {}", a.0, b.0);
            assert!(a.0.abs() <= 1.0, "{}", a.0);
        }
        // Mean squared change over a short step along vs across the slope,
        // over many points: the ratio is far below 1.
        let along = (1.0, 0.0);
        let (mut d_along, mut d_across) = (0.0, 0.0);
        for k in 0..2000 {
            let (x, y) = (k as f64 * 0.137, (k * 7 % 101) as f64 * 0.093);
            let v = gully_noise(x, y, along, 5, None).0;
            d_along += (gully_noise(x + 0.05, y, along, 5, None).0 - v).powi(2);
            d_across += (gully_noise(x, y + 0.05, along, 5, None).0 - v).powi(2);
        }
        // The ratio's band, measured 2026-10-19 (0.041): a round kernel
        // (no `GULLY_STRETCH`) reads 0.063 -- shorter gullies -- and stripes
        // never turned off the slope (`GULLY_TURN` 0) read 0.008 -- the comb
        // of parallel gullies the turn exists to break.
        let ratio = d_along / d_across;
        assert!((0.02..0.052).contains(&ratio), "along/across {ratio}");
        // Few samples near the stripes' full height: the points' own
        // strengths (`GULLY_VARY`) and the blend keep most of the field off
        // its extremes. Measured: 41 of 4 000 above |0.7|; equal strengths
        // read 502, a round kernel 238, unturned stripes 117.
        let mut hi = 0;
        for k in 0..4000 {
            let (x, y) = (k as f64 * 0.0937, (k * 13 % 211) as f64 * 0.0711);
            if gully_noise(x, y, along, 5, None).0.abs() > 0.7 {
                hi += 1;
            }
        }
        assert!(hi < 80, "{hi} of 4 000 samples near full height");
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

    /// Protects: the gullies follow the regional slope -- on a belt along a
    /// horizontal uplift line (regional slope north-south), the gully term
    /// varies far more along a row (across north-south gullies) than down a
    /// column (along them). A term that ignored the slope direction (or took
    /// the gradient's perpendicular) fails it.
    #[test]
    fn gullies_run_down_the_belt_flanks() {
        let (gw, gh) = (96usize, 96usize);
        let field0 = vec![0.45f32; gw * gh];
        // A horizontal uplift line: the regional slope runs north-south.
        let t: Vec<f32> = (0..gw * gh).map(|i| if i / gw == 48 { 1.0 } else { 0.0 }).collect();
        // The first octave alone (`gully_h` 8 makes the finer ones vanish):
        // the finer octaves run down the walls of the coarser gullies by
        // design, across the regional slope, so only the first is aligned
        // with it.
        let tn = OrogenTuning { gully_h: 8.0, ..OrogenTuning::default() };
        let inp = OrogenInputs { gw, gh, world: false, seed: 9, map_width_km: 192.0, peak_m: 4000.0, sea: 0.42, t: &t };
        let mut on = field0.clone();
        apply_orogen_relief(&mut on, &inp, &tn);
        // The control: the same belt and lift, no gullies; the difference is
        // the gully term alone (and its mean offset, which differencing drops).
        let mut off = field0.clone();
        apply_orogen_relief(&mut off, &inp, &OrogenTuning { gully_amp_m: 0.0, ..tn });
        let f: Vec<f64> = on.iter().zip(&off).map(|(&a, &b)| a as f64 - b as f64).collect();
        // On the flank rows (12-28 cells off the line, where the belt weight
        // and so the regional slope ramps; measured, the rows nearer the line
        // are the belt's flat-topped plateau, where there is no slope to
        // follow), squared steps of the gully term along x (across
        // north-south gullies) against along y.
        let (mut dx, mut dy) = (0.0f64, 0.0f64);
        for y in (20..31).chain(64..77) {
            for x in 10..86 {
                let i = y * gw + x;
                dx += (f[i + 1] - f[i]).powi(2);
                dy += (f[i + gw] - f[i]).powi(2);
            }
        }
        assert!(dx > 0.0, "the flanks carry no relief");
        assert!(dy < 0.3 * dx, "stripes must run down the slope: along {dy} across {dx}");
        // And flat ground gets almost none: the belt's flat-topped plateau
        // (rows 40-56 around the line) has no regional slope to run down.
        let mut plateau = 0.0f64;
        for y in 40..56 {
            for x in 10..86 {
                let i = y * gw + x;
                plateau += (f[i + 1] - f[i]).powi(2);
            }
        }
        // Per row, against the flank rows' (24 of them) across-slope steps.
        assert!(plateau / 16.0 < 0.05 * dx / 24.0, "the plateau carries gullies: {plateau} vs flanks {dx}");
    }

    /// A 3-row strip around a profile: high fixed walls (0.95, `FLANK_NONE`)
    /// above and below, the profile in the middle row with fixed ground at
    /// both of its ends and a breachable region between. Returns
    /// `(field, mask, width)`; the profile's cells are `field[w + x]`. (One
    /// row alone would not do: every cell of it is on the map edge, which the
    /// flood seeds as an outlet.)
    fn strip(profile: &[f32]) -> (Vec<f32>, Vec<u8>, usize) {
        let w = profile.len();
        let mut f = vec![0.95f32; 3 * w];
        let mut m = vec![FLANK_NONE; 3 * w];
        for (x, &v) in profile.iter().enumerate() {
            f[w + x] = v;
            if x > 0 && x + 1 < w {
                m[w + x] = FLANK_FLANK;
            }
        }
        (f, m, w)
    }

    /// Protects: a hollow is drained by a notch, not filled -- its floor keeps
    /// its height, the sill cells between it and the outlet are lowered below
    /// it, every cell then has a strictly lower 4-neighbour on a path to the
    /// fixed ground (no closed hollow is left), and the fixed ground keeps its
    /// bytes.
    #[test]
    fn breach_notches_a_hollow_instead_of_filling_it() {
        // Outlet 0.50 | 0.60 sill 0.62 | floor 0.55 | 0.60 | wall 0.70.
        let (mut ff, m, w) = strip(&[0.50, 0.60, 0.62, 0.55, 0.60, 0.70]);
        let before = ff.clone();
        let (lowered, raised) = breach_belt_pits(w, 3, &mut ff, &m, 0.42, 1.0);
        let f = &ff[w..2 * w];
        assert_eq!(f[3], before[w + 3], "the floor keeps its height");
        assert_eq!((f[0], f[5]), (before[w], before[w + 5]), "fixed ground untouched");
        assert_eq!(ff[..w], before[..w], "the walls untouched");
        assert!(f[2] < f[3] && f[1] < f[2] && f[0] < f[1], "the notch descends to the outlet: {f:?}");
        assert!(lowered >= 2 && raised == 0, "lowered {lowered} raised {raised}");
        assert!(f[4] > f[3], "the far wall drains into the notched floor");
    }

    /// Protects: the depth cap -- a floor deeper than `max_depth` below its
    /// sill is raised to `sill - max_depth` and the notch is no deeper than
    /// that; and the sea case -- a floor below the sea is filled above the
    /// cell it was reached from rather than notched through to the sea.
    #[test]
    fn breach_caps_the_notch_and_fills_below_the_sea() {
        let (mut ff, m, w) = strip(&[0.50, 0.70, 0.55, 0.70, 0.80]);
        breach_belt_pits(w, 3, &mut ff, &m, 0.42, 0.05);
        let f = &ff[w..2 * w];
        // Sill 0.70: the floor is raised to 0.65 and the sill notched just below it.
        assert!((f[2] - 0.65).abs() < 1e-6, "floor raised to the cap: {}", f[2]);
        assert!(f[1] < f[2] && f[1] > 0.64, "notch at most the cap deep: {}", f[1]);
        let (mut ss, m, w) = strip(&[0.50, 0.60, 0.40, 0.60, 0.70]);
        breach_belt_pits(w, 3, &mut ss, &m, 0.42, 1.0);
        let s = &ss[w..2 * w];
        // The cap reads the HIGHEST ground on the path, not the last cell:
        // outlet 0.50 | sill 0.80 | 0.70 | floor 0.55 | wall 0.90 -- the floor
        // is raised to 0.80 - 0.05, not 0.70 - 0.05.
        let (mut gg, m, w) = strip(&[0.50, 0.80, 0.70, 0.55, 0.90]);
        breach_belt_pits(w, 3, &mut gg, &m, 0.42, 0.05);
        assert!((gg[w + 3] - 0.75).abs() < 1e-6, "cap from the path's sill: {}", gg[w + 3]);
        // A hollow deeper than its outlet: outlet (fixed) 0.60 | sill 0.70 |
        // floor 0.55 | wall 0.80. The notch cannot go under the outlet, so the
        // floor is raised to just above it and the path still descends to it.
        let (mut hh, m, w) = strip(&[0.60, 0.70, 0.55, 0.80]);
        breach_belt_pits(w, 3, &mut hh, &m, 0.42, 1.0);
        let r = &hh[w..2 * w];
        assert!(r[2] > r[1] && r[1] > r[0] && r[0] == 0.60, "descends to the outlet: {r:?}");
        // Filled to its spill (0.60, give or take the few `f32` steps the
        // drain then takes off the now-level floor), not notched to the sea.
        assert!((s[2] - 0.60).abs() < 1e-6, "a below-sea floor is filled to its spill: {}", s[2]);
        assert!((s[1] - 0.60).abs() < 1e-6, "and its sill is not notched: {}", s[1]);
    }
}
