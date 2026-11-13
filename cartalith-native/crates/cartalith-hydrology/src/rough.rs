//! **Rough routing: sub-grid roughness in the choice of where water goes**
//! (owner, Ruling ER, 2026-11-11: *"3, and would it change if we had 32
//! instead of 8 directions?"*; `DELTA_GEOMETRY_RESEARCH.md` section 13).
//!
//! **Why.** Single-direction D8 routing sends all the water of a cell to the
//! steepest of its eight neighbours. Over a smooth planar slope that is the
//! same neighbour cell after cell, so a river crossing it is a ruler line on a
//! grid axis or diagonal: section 11 measured 32-58 % of every network's
//! length in such runs, up to 162 cells (3 200 km on the owner's world). Real
//! ground is never planar at the scale of a channel: sub-grid relief, which
//! the height field does not resolve, deflects the flow from cell to cell, so
//! a real river crossing a plain wanders. Section 13 measured the
//! alternatives (16 and 32 directions, a D-infinity direction with
//! least-transverse-deviation, a near-tie break) on the same worlds: 16 and
//! 32 directions leave the drawn curve as straight as D8 (the runs are only
//! at finer angles) and break drainage; the D-infinity path and the tie break
//! help some; this perturbation is the one that breaks the runs.
//!
//! **What.** A seeded value-noise field `n` (amplitude 1, lattice spacing
//! [`ROUGH_WAVELENGTH_CELLS`]) stands for the unresolved relief. A cell picks,
//! among its eight neighbours that are **strictly lower on the surface it is
//! routed over**, the one with the largest
//! `(h_i - h_j + k * s_i * (n_i - n_j)) / d_ij`, where `s_i` is the cell's own
//! steepest drop per cell and `k` is [`ROUGH_STRENGTH`]. The perturbation is
//! scaled by the local slope, so it is the same fraction of the relief on a
//! gentle plain and a steep flank -- **and on the depression-filled flats
//! too**: there `s_i` is the routing surface's few-ULP tilt, but so is every
//! drop it is compared with, so the noise decides the pick just as it does on
//! a slope. (Until 2026-11-13 this said the perturbation *vanishes* on the
//! flats; measured on the owner's combing world it does not -- it is what
//! breaks the comb there, `DELTA_GEOMETRY_RESEARCH.md` section 14.) What it
//! cannot do is choose among one candidate: on the breadth-first fill's
//! 45-degree lines a cell has a single strictly lower neighbour, so the line
//! stays straight whatever the noise; basin routing's tilt
//! (`FillTilt::Floor`, Ruling EV) gives such cells several.
//!
//! **Must never** pick a neighbour that is not strictly lower on the surface
//! (no uphill step, no cycle, and the descending-height accumulation order of
//! `compute_flow` stays valid); leave a cell that has a lower neighbour without
//! a receiver (drainage stays unbroken: the depression-filled surface gives
//! every land cell one); depend on anything but the surface, the grid, the
//! seed and the two constants (deterministic, the same on every machine); or
//! run when the world's `rough_routing` is off -- the reference path and every
//! golden stay plain D8 by control flow.

/// The lattice spacing of the roughness noise, in grid cells: the length over
/// which the unresolved relief turns the flow one way and then the other.
/// **Labelled judgement, chosen by measurement** (section 13,
/// `river_routing_measurement.rs`, at strength 4 on three worlds): 4 cells
/// takes the drawn curve's straight share to 10-17 %; 6 cells to 18-26 %, 8
/// cells to 26-37 % and 16 cells to 39-58 %, close to plain D8's 41-63 %: a
/// lattice much longer than the straight runs leaves them straight.
pub const ROUGH_WAVELENGTH_CELLS: f64 = 4.0;

/// The roughness's strength `k`, as a multiple of the cell's own steepest drop
/// per cell. **Labelled judgement, chosen by measurement** (section 13, at
/// spacing 4 on three worlds): `k` 2 leaves 17-27 % of the drawn curve
/// straight, 3 leaves 12-20 % and 4 leaves 10-17 %, against plain D8's
/// 41-63 %. Why it must be well above 1: on a planar slope the next neighbour
/// round (45 degrees) has a drop per distance of `cos 45` = 0.71 of the
/// steepest, so the perturbation's difference over one cell must exceed 0.29
/// of the drop to turn the flow, and smooth noise of amplitude 1 differs by
/// only a fraction of 1 between neighbours (measured: at `k` 1, spacing 8, not
/// one cell of a 64 x 64 planar ramp turns). It never turns flow uphill
/// whatever `k` is: only strictly lower neighbours are candidates.
pub const ROUGH_STRENGTH: f64 = 4.0;

/// **The roughness of one world**: the noise field and its strength. Built
/// once per generation from the world's seed ([`Roughness::new`]) and handed
/// to every flow routing of that generation, so the accumulation tree and the
/// channel tree are the same tree.
#[derive(Clone, Debug)]
pub struct Roughness {
    /// `gw * gh` noise values in `[-1, 1]`.
    noise: Vec<f32>,
    /// The strength `k` ([`ROUGH_STRENGTH`]).
    k: f64,
}

/// SplitMix64's finaliser (Steele, Lea & Flood 2014), the project's usual
/// integer hash: a well-mixed `u64` of its input.
fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl Roughness {
    /// The roughness of a `gw` x `gh` world with seed `seed`, at the shipped
    /// [`ROUGH_STRENGTH`] and [`ROUGH_WAVELENGTH_CELLS`]. `world` wraps `x`
    /// (a cylindrical world), and the noise then tiles across the seam.
    pub fn new(gw: usize, gh: usize, seed: u64, world: bool) -> Roughness {
        Roughness::with_params(gw, gh, seed, world, ROUGH_STRENGTH, ROUGH_WAVELENGTH_CELLS)
    }

    /// [`Roughness::new`] at a given strength and lattice spacing (cells) --
    /// the measurement harness's sweep (`river_routing_measurement.rs`).
    /// A non-finite or non-positive spacing reads as the shipped one.
    pub fn with_params(gw: usize, gh: usize, seed: u64, world: bool, k: f64, wavelength: f64) -> Roughness {
        let wl = if wavelength.is_finite() && wavelength > 0.0 { wavelength } else { ROUGH_WAVELENGTH_CELLS };
        // Lattice points across: at least one, and on a wrapping world a whole
        // number spanning exactly `gw`, so the noise is continuous over the seam.
        let nx = ((gw as f64 / wl).round() as usize).max(1);
        let sx = gw.max(1) as f64 / nx as f64;
        let salt = mix(seed ^ 0x5275_6768_526F_7574); // "RughRout": this field's own stream
        let lattice = |ix: i64, iy: i64| -> f64 {
            let ix = if world { ix.rem_euclid(nx as i64) } else { ix };
            let h = mix(salt ^ mix((ix as u64).wrapping_mul(0x1_0000_0001) ^ (iy as u64).rotate_left(32)));
            // Top 53 bits to [0, 1), then to [-1, 1].
            (h >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
        };
        let smooth = |t: f64| t * t * (3.0 - 2.0 * t);
        let mut noise = vec![0f32; gw * gh];
        for y in 0..gh {
            let fy = (y as f64 + 0.5) / wl;
            let (iy, ty) = (fy.floor() as i64, smooth(fy - fy.floor()));
            for x in 0..gw {
                let fx = (x as f64 + 0.5) / sx;
                let (ix, tx) = (fx.floor() as i64, smooth(fx - fx.floor()));
                let a = lattice(ix, iy) + (lattice(ix + 1, iy) - lattice(ix, iy)) * tx;
                let b = lattice(ix, iy + 1) + (lattice(ix + 1, iy + 1) - lattice(ix, iy + 1)) * tx;
                noise[y * gw + x] = (a + (b - a) * ty) as f32;
            }
        }
        Roughness { noise, k: if k.is_finite() { k } else { ROUGH_STRENGTH } }
    }

    /// The noise at cell `i` (for tests and the measurement).
    pub fn noise_at(&self, i: usize) -> f32 {
        self.noise[i]
    }
}

/// **The rough receiver of cell `i`** over `field` (the routing surface), or
/// `-1` where no neighbour is strictly lower (a pit, or a seed of the
/// surface). See the module doc for the rule; `d8` is
/// [`crate::d8_table`]. `world` wraps `x` only, as [`crate::d8_receiver`] does.
/// Ties keep the earlier neighbour in the same `dy`-then-`dx` scan order.
#[inline]
pub(crate) fn rough_receiver(field: &[f32], gw: usize, gh: usize, i: usize, world: bool, d8: &[f64; 9], r: &Roughness) -> i64 {
    let x = (i % gw) as i64;
    let y = (i / gw) as i64;
    let h = field[i] as f64;
    // Pass 1: the strictly lower neighbours and the steepest drop per cell.
    let mut cand = [(0usize, 0f64, 0f64); 8];
    let mut m = 0usize;
    let mut s = 0.0f64;
    for dy in -1i64..=1 {
        for dx in -1i64..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let mut nx = x + dx;
            let ny = y + dy;
            if world {
                nx = nx.rem_euclid(gw as i64);
            } else if nx < 0 || nx >= gw as i64 {
                continue;
            }
            if ny < 0 || ny >= gh as i64 {
                continue;
            }
            let j = (ny * gw as i64 + nx) as usize;
            let hj = field[j] as f64;
            if hj < h {
                let d = d8[((dy + 1) * 3 + (dx + 1)) as usize];
                s = s.max((h - hj) / d);
                cand[m] = (j, hj, d);
                m += 1;
            }
        }
    }
    // Pass 2: the largest perturbed drop among them.
    let ni = r.noise[i] as f64;
    let mut best: i64 = -1;
    let mut best_score = f64::NEG_INFINITY;
    for &(j, hj, d) in &cand[..m] {
        let score = (h - hj + r.k * s * (ni - r.noise[j] as f64)) / d;
        if score > best_score {
            best_score = score;
            best = j as i64;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A planar slope falling to the east by `g` per cell, `w` x `h`.
    fn ramp(w: usize, h: usize, g: f32) -> Vec<f32> {
        (0..w * h).map(|i| 10.0 - g * (i % w) as f32).collect()
    }

    /// Protects: the two shipped constants at LITERAL values (the measurement
    /// in section 13 chose them; a silent retune must fail here).
    #[test]
    fn the_shipped_roughness_is_pinned_by_literals() {
        assert_eq!((ROUGH_STRENGTH, ROUGH_WAVELENGTH_CELLS), (4.0, 4.0));
    }

    /// Protects: the noise is a deterministic function of the seed, lies in
    /// `[-1, 1]`, varies (it is not a constant field), differs between seeds,
    /// and on a wrapping world is continuous across the seam (the last column
    /// is as close to the first as any two neighbours are).
    #[test]
    fn the_noise_is_seeded_bounded_and_wraps() {
        let (w, h) = (64usize, 32usize);
        let a = Roughness::new(w, h, 7, true);
        let b = Roughness::new(w, h, 7, true);
        let c = Roughness::new(w, h, 8, true);
        assert_eq!(a.noise, b.noise);
        assert_ne!(a.noise, c.noise);
        assert!(a.noise.iter().all(|v| (-1.0..=1.0).contains(v)));
        let (lo, hi) = a.noise.iter().fold((f32::MAX, f32::MIN), |m, &v| (m.0.min(v), m.1.max(v)));
        assert!(hi - lo > 0.5, "the noise varies: {lo}..{hi}");
        let step = |x0: usize, x1: usize| (0..h).map(|y| (a.noise[y * w + x0] - a.noise[y * w + x1]).abs()).fold(0f32, f32::max);
        let inner = (0..w - 1).map(|x| step(x, x + 1)).fold(0f32, f32::max);
        assert!(step(w - 1, 0) <= inner + 1e-6, "seam {} vs inner {inner}", step(w - 1, 0));
    }

    /// Protects: the must-nevers. On a ramp with noise, every receiver is
    /// strictly lower (no uphill step), every cell with a lower neighbour has
    /// one (no break), and a pit has none. With strength 0 the pick is plain
    /// D8 (`d8_receiver`) cell for cell.
    #[test]
    fn a_rough_receiver_is_always_strictly_lower_and_never_missing() {
        let (w, h) = (40usize, 30usize);
        let f = ramp(w, h, 0.01);
        let d8 = crate::d8_table();
        let r = Roughness::with_params(w, h, 3, false, 2.0, 4.0);
        for i in 0..w * h {
            let j = rough_receiver(&f, w, h, i, false, &d8, &r);
            let has_lower = crate::d8_receiver(&f, w, h, i, false, &d8) >= 0;
            assert_eq!(j >= 0, has_lower, "cell {i}");
            if j >= 0 {
                assert!(f[j as usize] < f[i], "cell {i} -> {j} is not downhill");
            }
        }
        let zero = Roughness::with_params(w, h, 3, false, 0.0, 4.0);
        for i in 0..w * h {
            assert_eq!(rough_receiver(&f, w, h, i, false, &d8, &zero), crate::d8_receiver(&f, w, h, i, false, &d8), "cell {i}");
        }
        let mut pit = vec![1.0f32; 9];
        pit[4] = 0.0;
        assert_eq!(rough_receiver(&pit, 3, 3, 4, false, &d8, &Roughness::new(3, 3, 1, false)), -1);
    }

    /// Protects: the point of the change. On a planar slope plain D8 sends
    /// every cell due east (one repeated direction); the shipped roughness
    /// turns a share of the cells off that axis, so a river crossing the
    /// slope does not run as a ruler line. Pinned loosely (more than a fifth
    /// of the cells turn) as a property, not a value; at the old trial
    /// strength of 1 (spacing 8) none turned, which is why the shipped
    /// strength is 4.
    #[test]
    fn roughness_turns_flow_off_the_grid_axis_on_a_planar_slope() {
        let (w, h) = (64usize, 64usize);
        let f = ramp(w, h, 0.01);
        let d8 = crate::d8_table();
        let r = Roughness::new(w, h, 11, false);
        let mut off = 0usize;
        let mut n = 0usize;
        for y in 1..h - 1 {
            for x in 0..w - 1 {
                let i = y * w + x;
                assert_eq!(crate::d8_receiver(&f, w, h, i, false, &d8), (i + 1) as i64, "plain D8 runs due east");
                n += 1;
                off += (rough_receiver(&f, w, h, i, false, &d8, &r) != (i + 1) as i64) as usize;
            }
        }
        assert!(off * 5 > n, "{off} of {n} cells turned off the axis");
    }
}
