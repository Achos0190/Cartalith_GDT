//! **Volcano flank-pit fill** (owner Ruling CO, 2026-10-18; measured as
//! variant (c-late) in `EDIFICE_AND_STREAKS_2026-10-18.md` at the repository
//! root): after every generation stage, fill the small closed hollows left on
//! the flanks of the morphological volcano edifices, so a gully end drains to
//! the flank base instead of holding a 1-3-cell pond.
//!
//! # Why
//!
//! The morphological edifice (`DECISIONS.md` §7l-ii, `crate::EdificeModel`)
//! cuts radial gullies whose depth envelope `4u(1-u)` shallows faster than a
//! concave cone falls on the lower flank, so a gully floor rises again toward
//! the toe and leaves a closed hollow. Measured on five 800 km app worlds:
//! about 600 such hollows per world, median depth about 3 m, of which RV-1's
//! water-body classifier pooled a median 41 (5-170) into 1-3-cell lakes. With
//! this fill on: 10 (0-11), and outlets agree within five cells on 99.93 % of
//! land. Erosion and the carve re-cut hollows a stamp-time fill removes, which
//! is why this runs on the **final** field.
//!
//! The root cause (a gully depth that is not capped by the cone's own fall) is
//! a design row in `OUTSTANDING_WORK.md` ("type-dependent gully spacing and
//! gully depth capped by the cone"); this fill is the measured fix until then.
//!
//! # What it reads, and why only that
//!
//! Only `volcanic_field`, which the stamps write as
//! `max over edifices of (1 - t)(1 - age)`, a cone per edifice. The stamps'
//! own per-edifice records are gone by the end of generation, so
//! [`EdificeFrame::from_volcanic_field`] recovers each edifice's centre and
//! radius from that cone, and [`flank_mask_from_volcanic_field`] marks its
//! flank and its summit. The deep-zoom tile furrows
//! (`crate::edifice_furrows`) read the same frame.
//!
//! # What it must never do
//!
//! - Lower a cell, or touch a cell that is not on an edifice flank.
//! - Fill a caldera or summit depression: those are real closed basins.
//! - Run on the parity path: `cartalith_engine` calls it only under
//!   `VolcanismParams::fill_edifice_pits`, which is `false` in
//!   `WorldParams::defaults` (and the morphological edifice it fills is off
//!   there too).

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

/// One edifice as recovered from `volcanic_field`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecoveredEdifice {
    /// Centre, in coarse cells (the cone's peak cell).
    pub cx: f64,
    pub cy: f64,
    /// Radius in cells: the median of `d / (1 - v / v_peak)` over the
    /// edifice's own cells at `0.2 <= t <= 0.9`.
    pub r: f64,
    /// Noise salt, from the centre cell (deterministic, not an RNG draw);
    /// read only by `crate::edifice_furrows`.
    pub salt: i32,
}

/// Per coarse cell, which recovered edifice owns it.
#[derive(Clone, Debug)]
pub struct EdificeFrame {
    pub gw: usize,
    pub gh: usize,
    pub edifices: Vec<RecoveredEdifice>,
    /// Index into `edifices`, or [`EdificeFrame::NONE`].
    pub owner: Vec<u32>,
}

impl EdificeFrame {
    /// No edifice owns this cell. Not an index.
    pub const NONE: u32 = u32::MAX;

    /// Recover the edifices from the stamp's `volcanic_field` (see the module
    /// doc). Each cell with `v > 0` climbs its steepest 8-neighbour ascent to
    /// a local maximum, which is its edifice's centre: inside one edifice's
    /// winning region the field is that edifice's cone, strictly increasing
    /// toward its centre. An edifice whose radius cannot be recovered (fewer
    /// than three cells in the radius band) is dropped and its cells are
    /// owned by nothing -- no stand-in radius.
    pub fn from_volcanic_field(v: &[f32], gw: usize, gh: usize) -> Self {
        let n = gw * gh;
        assert!(v.len() >= n, "EdificeFrame: short volcanic_field");
        const UNSET: u32 = u32::MAX - 1;
        let mut peak_of = vec![UNSET; n];
        let mut path = Vec::new();
        for s in 0..n {
            if v[s] <= 0.0 {
                peak_of[s] = EdificeFrame::NONE;
                continue;
            }
            if peak_of[s] != UNSET {
                continue;
            }
            path.clear();
            let mut c = s;
            let top = loop {
                if peak_of[c] != UNSET {
                    break peak_of[c];
                }
                path.push(c);
                let (x, y) = ((c % gw) as i64, (c / gw) as i64);
                let mut best = (v[c], c);
                for dy in -1i64..=1 {
                    for dx in -1i64..=1 {
                        let (xx, yy) = (x + dx, y + dy);
                        if (dx, dy) == (0, 0) || xx < 0 || yy < 0 || xx as usize >= gw || yy as usize >= gh {
                            continue;
                        }
                        let j = yy as usize * gw + xx as usize;
                        if v[j] > best.0 {
                            best = (v[j], j);
                        }
                    }
                }
                if best.1 == c {
                    break c as u32;
                }
                c = best.1;
            };
            for &q in &path {
                peak_of[q] = top;
            }
        }
        // Radius per peak: median of the per-cell estimates in the band.
        let mut est: BTreeMap<u32, Vec<f64>> = BTreeMap::new();
        for i in 0..n {
            let p = peak_of[i];
            if p == EdificeFrame::NONE {
                continue;
            }
            let vp = v[p as usize] as f64;
            let t = 1.0 - v[i] as f64 / vp;
            if (0.2..=0.9).contains(&t) {
                let (px, py) = ((p as usize % gw) as f64, (p as usize / gw) as f64);
                let d = ((i % gw) as f64 - px).hypot((i / gw) as f64 - py);
                est.entry(p).or_default().push(d / t);
            }
        }
        let mut index: BTreeMap<u32, u32> = BTreeMap::new();
        let mut edifices = Vec::new();
        for (p, mut e) in est {
            if e.len() < 3 {
                continue;
            }
            e.sort_by(|a, b| a.total_cmp(b));
            let r = e[e.len() / 2];
            let (cx, cy) = ((p as usize % gw) as f64, (p as usize / gw) as f64);
            index.insert(p, edifices.len() as u32);
            edifices.push(RecoveredEdifice { cx, cy, r, salt: (p as i32).wrapping_mul(2_654_435_761u32 as i32) ^ 0x5eed });
        }
        let owner = peak_of.iter().map(|p| index.get(p).copied().unwrap_or(EdificeFrame::NONE)).collect();
        EdificeFrame { gw, gh, edifices, owner }
    }

    /// The edifice owning the coarse cell nearest `(cx, cy)`, if any.
    pub fn owner_near(&self, cx: f64, cy: f64) -> Option<&RecoveredEdifice> {
        if self.gw == 0 || self.gh == 0 || !(cx.is_finite() && cy.is_finite()) {
            return None;
        }
        let x = (cx.round().max(0.0) as usize).min(self.gw - 1);
        let y = (cy.round().max(0.0) as usize).min(self.gh - 1);
        match self.owner[y * self.gw + x] {
            EdificeFrame::NONE => None,
            k => self.edifices.get(k as usize),
        }
    }
}

/// What [`fill_edifice_flank_pits`] reads per cell: no edifice, an edifice's
/// flank, or a summit/caldera (which is never filled).
pub const FLANK_NONE: u8 = 0;
pub const FLANK_FLANK: u8 = 1;
pub const FLANK_CALDERA: u8 = 2;

/// The summit radius, as a fraction of the recovered radius, that
/// [`flank_mask_from_volcanic_field`] protects: past the widest summit
/// depression of the three shipped shapes (a scoria cone's rim at 0.38,
/// `crate::edifice_shape`), so no caldera is filled. **A labelled judgement**.
pub const SUMMIT_T: f64 = 0.4;

/// The flank mask [`fill_edifice_flank_pits`] reads, rebuilt from
/// `volcanic_field` alone: [`FLANK_CALDERA`] within [`SUMMIT_T`] of a
/// recovered edifice's radius from its centre (the summit depression, never
/// filled), [`FLANK_FLANK`] on the rest of a recovered edifice, [`FLANK_NONE`]
/// elsewhere.
pub fn flank_mask_from_volcanic_field(v: &[f32], gw: usize, gh: usize) -> Vec<u8> {
    let f = EdificeFrame::from_volcanic_field(v, gw, gh);
    (0..gw * gh)
        .map(|i| match f.owner[i] {
            EdificeFrame::NONE => FLANK_NONE,
            k => {
                let e = &f.edifices[k as usize];
                if ((i % gw) as f64 - e.cx).hypot((i / gw) as f64 - e.cy) < SUMMIT_T * e.r {
                    FLANK_CALDERA
                } else {
                    FLANK_FLANK
                }
            }
        })
        .collect()
}

/// Fill every closed depression on edifice **flanks** to its spill level, so
/// a gully end drains to the flank base. Priority-Flood+ε (Barnes, Lehman &
/// Mulla, *Priority-flood: An optimal depression-filling and watershed-labeling
/// algorithm for digital elevation models*, Computers & Geosciences 62, 2014
/// -- cited from memory, not opened in this pass): seeded from the fixed
/// ground around the flank region, it raises a flank cell no higher than one
/// `f32` step above the cell it was reached from, so every filled cell has a
/// strictly lower **orthogonal** neighbour and water routes out.
///
/// `flank[i]` is [`FLANK_FLANK`] on a flank, [`FLANK_CALDERA`] inside a summit
/// depression (never filled) and [`FLANK_NONE`] elsewhere. Returns
/// `(cells raised, largest raise)`.
///
/// What it must never do: lower a cell, or touch a cell that is not a flank.
pub fn fill_edifice_flank_pits(gw: usize, gh: usize, field: &mut [f32], flank: &[u8]) -> (usize, f32) {
    let n = gw * gh;
    assert!(field.len() >= n && flank.len() >= n, "fill_edifice_flank_pits: short field");
    let mut closed = vec![false; n];
    // Heights are non-negative after generation's clamp, so the bit pattern of
    // `max(0)` orders them as `f32` does; the index breaks ties
    // deterministically.
    let mut heap: BinaryHeap<Reverse<(u32, usize)>> = BinaryHeap::new();
    // **4-connected**, the water-body classifier's own connectivity
    // (`cartalith_civ::build_water_bodies`' priority flood visits the four
    // orthogonal neighbours). Measured (`EDIFICE_AND_STREAKS_2026-10-18.md`):
    // an 8-connected fill left a cell whose only lower neighbour was diagonal,
    // which the classifier still pooled -- flank pits on 24601 fell 372 -> 39
    // but its 1-3-cell edifice lakes only 170 -> 103. A 4-connected drain also
    // drains 8-connected.
    let nb = |i: usize| -> [Option<usize>; 4] {
        let (x, y) = (i % gw, i / gw);
        [(x > 0).then(|| i - 1), (x + 1 < gw).then(|| i + 1), (y > 0).then(|| i - gw), (y + 1 < gh).then(|| i + gw)]
    };
    // Seeds: the fixed ground around the region -- every non-flank cell that
    // touches a flank (the plain at the toe, a caldera inside the rim) -- and
    // flank cells on the map edge, which drain off the map. Seeding the
    // region's own border cells instead would leave a toe cell that sits
    // below the plain as a pit (measured on a flat-plane stratocone: 18).
    for i in 0..n {
        let ns = nb(i);
        let seed = if flank[i] == FLANK_FLANK {
            ns.iter().any(|j| j.is_none())
        } else {
            ns.iter().flatten().any(|&j| flank[j] == FLANK_FLANK)
        };
        if seed {
            closed[i] = true;
            heap.push(Reverse((field[i].max(0.0).to_bits(), i)));
        }
    }
    let (mut raised, mut most) = (0usize, 0f32);
    while let Some(Reverse((hb, c))) = heap.pop() {
        let hc = f32::from_bits(hb);
        for j in nb(c).into_iter().flatten() {
            if closed[j] || flank[j] != FLANK_FLANK {
                continue;
            }
            closed[j] = true;
            // `<=`, not `<`: a cell level with its spill is part of a flat
            // and gets the ε gradient too (`fill_drains_a_flat_toward_its_outlet`).
            if field[j] <= hc {
                // The next representable `f32` above `hc` (positive, finite).
                let up = f32::from_bits(hc.to_bits() + 1);
                most = most.max(up - field[j]);
                field[j] = up;
                raised += 1;
            }
            heap.push(Reverse((field[j].max(0.0).to_bits(), j)));
        }
    }
    (raised, most)
}

/// The whole production step, as `cartalith_engine::generate_terrain` runs it
/// under `VolcanismParams::fill_edifice_pits`: rebuild the flank mask from
/// `volcanic_field`, then fill. Returns [`fill_edifice_flank_pits`]'s counts.
/// A world with no edifice (an all-zero `volcanic_field`) is left untouched.
pub fn fill_edifice_pits(gw: usize, gh: usize, field: &mut [f32], volcanic_field: &[f32]) -> (usize, f32) {
    let m = flank_mask_from_volcanic_field(volcanic_field, gw, gh);
    fill_edifice_flank_pits(gw, gh, field, &m)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic `volcanic_field`: two cones, `(1 - t)(1 - age)`, the
    /// stamp's own mark.
    fn two_cones(gw: usize, gh: usize) -> Vec<f32> {
        let mut v = vec![0f32; gw * gh];
        for (cx, cy, r, age) in [(40.0, 40.0, 30.0, 0.2), (110.0, 60.0, 20.0, 0.0)] {
            for y in 0..gh {
                for x in 0..gw {
                    let d = (x as f64 - cx).hypot(y as f64 - cy);
                    if d <= r {
                        let c = ((1.0 - d / r) * (1.0 - age)) as f32;
                        let i = y * gw + x;
                        v[i] = v[i].max(c);
                    }
                }
            }
        }
        v
    }

    /// Protects: the frame recovers both cones' centres to the cell and their
    /// radii within a cell, and owns nothing off them.
    #[test]
    fn frame_recovers_centres_and_radii() {
        let (gw, gh) = (160usize, 110usize);
        let f = EdificeFrame::from_volcanic_field(&two_cones(gw, gh), gw, gh);
        assert_eq!(f.edifices.len(), 2, "{:?}", f.edifices);
        let a = f.edifices.iter().find(|e| (e.cx - 40.0).abs() < 1.0).expect("cone at 40,40");
        let b = f.edifices.iter().find(|e| (e.cx - 110.0).abs() < 1.0).expect("cone at 110,60");
        assert!((a.r - 30.0).abs() < 1.0 && (b.r - 20.0).abs() < 1.0, "radii {} {}", a.r, b.r);
        assert_eq!(f.owner[5 * gw + 150], EdificeFrame::NONE);
    }

    /// Protects: the rebuilt mask protects the summit (inside 0.4 of the
    /// radius), marks the rest of the cone flank, and nothing off it.
    #[test]
    fn rebuilt_flank_mask_protects_the_summit() {
        let (gw, gh) = (160usize, 110usize);
        let m = flank_mask_from_volcanic_field(&two_cones(gw, gh), gw, gh);
        assert_eq!(m[40 * gw + 40], FLANK_CALDERA, "centre");
        assert_eq!(m[40 * gw + 51], FLANK_CALDERA, "t = 0.37");
        assert_eq!(m[40 * gw + 58], FLANK_FLANK, "t = 0.6");
        assert_eq!(m[5 * gw + 150], FLANK_NONE, "off");
    }

    /// Protects: the fill on a hand-made field: a one-cell pit on a ramp is
    /// raised to just above its lowest rim neighbour, and the same pit marked
    /// as caldera is left alone.
    #[test]
    fn fill_raises_a_pit_to_its_spill_level_only_on_flanks() {
        let (gw, gh) = (7usize, 7usize);
        let f: Vec<f32> = (0..gw * gh).map(|i| 0.5 + 0.01 * (i % gw) as f32).collect();
        let mut f = f;
        let pit = 3 * gw + 3;
        f[pit] = 0.4;
        let mut mask = vec![FLANK_FLANK; gw * gh];
        let mut g = f.clone();
        let (raised, most) = fill_edifice_flank_pits(gw, gh, &mut g, &mask);
        assert_eq!(raised, 1, "only the pit is below its spill");
        // Its lowest neighbour is at x = 2 (0.52, as the f32 sum above makes it).
        assert_eq!(g[pit], f32::from_bits(f[3 * gw + 2].to_bits() + 1));
        assert!((most - (g[pit] - 0.4)).abs() < 1e-7);
        mask[pit] = FLANK_CALDERA;
        let mut h = f.clone();
        fill_edifice_flank_pits(gw, gh, &mut h, &mask);
        assert_eq!(h[pit], 0.4, "a caldera is never filled");
    }

    /// Protects: a flat is drained, not left level. A 3 x 3 plateau at the
    /// height of its only outlet (a gap in a higher ring) is given a gradient,
    /// so every plateau cell ends with a strictly lower orthogonal neighbour
    /// -- the "+ε" in Priority-Flood+ε. Filling only cells strictly below the
    /// spill would leave the flat as a closed pond to the lake classifier.
    #[test]
    fn fill_drains_a_flat_toward_its_outlet() {
        let (gw, gh) = (7usize, 7usize);
        let mut f = vec![0.6f32; gw * gh];
        for y in 2..5 {
            for x in 2..5 {
                f[y * gw + x] = 0.5;
            }
        }
        // The outlet: a 0.5 channel from the plateau to the map edge.
        f[3 * gw + 1] = 0.5;
        f[3 * gw] = 0.5;
        let mask = vec![FLANK_FLANK; gw * gh];
        fill_edifice_flank_pits(gw, gh, &mut f, &mask);
        for y in 2..5 {
            for x in 2..5 {
                let i = y * gw + x;
                let lower = [i - 1, i + 1, i - gw, i + gw].iter().any(|&j| f[j] < f[i]);
                assert!(lower, "plateau cell ({x},{y}) at {} has no lower neighbour", f[i]);
            }
        }
    }

    /// Protects: the production step end to end on a cone with a flank pit
    /// and a summit pit: the flank pit is filled, the summit pit (inside
    /// [`SUMMIT_T`]) is kept, nothing is lowered, nothing off the cone moves,
    /// and an all-zero `volcanic_field` moves nothing at all.
    #[test]
    fn production_step_fills_flank_pits_and_keeps_the_summit() {
        let (gw, gh) = (160usize, 110usize);
        let v = two_cones(gw, gh);
        // A cone-shaped height on the first cone and a plain at 0.45 elsewhere.
        let mut field: Vec<f32> = (0..gw * gh).map(|i| 0.45 + 0.3 * v[i]).collect();
        let flank_pit = 40 * gw + 60; // t = 0.67 on the first cone
        let summit_pit = 40 * gw + 41; // t = 0.03
        field[flank_pit] -= 0.05;
        field[summit_pit] -= 0.05;
        let before = field.clone();
        let (raised, _) = fill_edifice_pits(gw, gh, &mut field, &v);
        assert!(raised >= 1, "nothing was filled");
        let nb = [flank_pit - 1, flank_pit + 1, flank_pit - gw, flank_pit + gw];
        assert!(nb.iter().any(|&j| field[j] < field[flank_pit]), "the flank pit still has no lower neighbour");
        assert_eq!(field[summit_pit], before[summit_pit], "the summit pit must be kept");
        for i in 0..gw * gh {
            assert!(field[i] >= before[i], "cell {i} was lowered");
            if v[i] <= 0.0 {
                assert_eq!(field[i].to_bits(), before[i].to_bits(), "off-cone cell {i} moved");
            }
        }
        let mut g = before.clone();
        assert_eq!(fill_edifice_pits(gw, gh, &mut g, &vec![0f32; gw * gh]), (0, 0.0));
        assert_eq!(g, before);
    }
}
