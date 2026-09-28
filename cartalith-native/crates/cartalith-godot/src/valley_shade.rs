//! **RV-3: the valley the map shades is cut along the drawn river line**
//! (`LARGE_ITEM_RULINGS.md` Ruling BD, 2026-09-29; `OUTSTANDING_WORK.md` row
//! "RV-3: a stepped channel is shaded off the smooth line").
//!
//! Generation carves every traced river run into the height field
//! (`cartalith_hydrology::carve_channel_network`). That carve is hydrology:
//! the water bodies, the flow and the channel tree are all computed from the
//! carved field, and Ruling BD keeps it. But the carve follows the traced D8
//! cells, which are a stair of whole cells, and the map draws each river as a
//! smooth vector line (RV-2) that drifts from those cells (measured with
//! `_riverzoom_probe`: median 0.14-0.16 cells, p90 0.57-1.19, per world). The
//! carve also digs every traced run, and about a third more runs are traced
//! than are drawn (1637 / 1219, 1670 / 1245, 1201 / 908 on the probe's three
//! worlds). The renderer shaded the carve as it stood, so the map showed
//! dark, stepped grooves beside every drawn river and along rivers that are
//! not drawn at all.
//!
//! This module builds the height field the **renderer** shades instead
//! ([`valley_shade_field`]): the carve filled back in, then a smooth valley cut
//! along each drawn line, as deep as the carve was there. It is a rendering
//! input only. It must never be written back to the world, never reach
//! hydrology, water classification, the carve, the lock masks, a save, or
//! anything a simulation reads; `WorldGen::valley_shade_field` hands it to the
//! three render paths (the screen texture, the deep-zoom tiles and every
//! export) and to nothing else, so all three shade the same surface.
//!
//! **Water is never touched, and neither is land beside it.** Every cell that
//! is water in the drawn classification or below sea level, and every cell
//! 8-adjacent to one, keeps the true height. So every per-cell water test the
//! renderer makes (`h < sea_level`) and every bilinear one (a bilinear cell
//! with a water corner has all its other corners in that frozen ring) answers
//! exactly as on the true field: no coast, lake shore or sea colour moves.
//! The cost is that the last cell of a channel beside the water keeps its
//! carve; that is disclosed in `STATUS.md`.
//!
//! **No land is taken below sea level.** A re-cut cell is floored at
//! `sea_level + cartalith_hydrology::CARVE_LAND_MARGIN`, the carve's own land
//! floor, so no land cell becomes water to the renderer.

use crate::river_stroke::DrawnRun;

/// How many Gauss-Seidel sweeps the fill runs at most. A labelled judgement:
/// the carve is at most `2·4·width_k + 1` cells across (the disc's half-width
/// cap, `generate_terrain`'s `half_w_cap`), and a harmonic fill across a
/// trench of width `n` settles in the order of `n²` sweeps; 200 covers a
/// 9-cell trench with margin. The loop also stops early once no cell moves by
/// more than [`FILL_TOL`].
const FILL_SWEEPS: usize = 200;

/// Convergence tolerance for the fill, in height-field units (the field is
/// `0..1`; sea level is typically 0.42). A labelled judgement: `1e-7` is below
/// f32's resolution at these heights (~6e-8 at 0.5), so the fill has stopped
/// moving the stored value.
const FILL_TOL: f64 = 1e-7;

/// The valley's shoulder beyond the drawn half-width, in cells, is
/// `max(SHOULDER_MIN_CELLS, half_width)`. A labelled judgement: one cell is the
/// narrowest shoulder a one-cell-per-texel raster can show as a slope rather
/// than a step, and scaling with the half-width keeps a wide river's valley in
/// proportion to it (the carve's own disc has no shoulder at all, which is the
/// stepped edge this module exists to replace).
const SHOULDER_MIN_CELLS: f64 = 1.0;

/// How far from a drawn point, beyond its half-width, the carve's depth is
/// looked for, in cells. The drift of a traced cell from the drawn line is
/// p90 0.57-1.19 cells on the probe's worlds, so the carve under a drawn
/// point is found within this reach; a labelled judgement.
const DEPTH_REACH_CELLS: f64 = 1.5;

/// The half-length, in cells of arc length, of the window the depth is
/// averaged over along each line. A labelled judgement: the carve's depth
/// steps where the traced stair steps (one cell), so a window of a few cells
/// removes the stair without losing a real change of depth along the valley.
const DEPTH_SMOOTH_CELLS: f64 = 1.5;

/// The height field the renderer shades: `field` with the generation carve
/// filled back in, and a smooth valley cut along each drawn river line.
///
/// - `field`: the world's true height (carved), `gw * gh`.
/// - `water`: the drawn water classification (`0` land, anything else water:
///   `WorldGen::drawn_water_classification`), `gw * gh`.
/// - `carved`: the cells the carve lowered (`WorldState::river_mask`, non-zero
///   = lowered), `gw * gh`.
/// - `runs`: the drawn river network (`WorldGen::river_geometry_any`), whose
///   render points are in grid-cell space with a cell's centre at `x + 0.5`.
///
/// Returns `None` when any grid is the wrong length -- the caller then shades
/// the true field, never a guessed one.
///
/// Deterministic: a pure function of its inputs, visited in a fixed order.
#[allow(clippy::too_many_arguments)]
pub fn valley_shade_field(field: &[f32], water: &[u8], carved: &[u8], runs: &[DrawnRun], gw: usize, gh: usize, sea_level: f64, world: bool) -> Option<Vec<f32>> {
    let n = gw.checked_mul(gh)?;
    if n == 0 || field.len() != n || water.len() != n || carved.len() != n {
        return None;
    }
    let frozen = frozen_mask(field, water, gw, gh, sea_level, world);
    let (mut out, depth) = uncarve(field, carved, &frozen, gw, gh, world);
    let base = out.clone();
    let land_floor = sea_level + cartalith_hydrology::CARVE_LAND_MARGIN;
    for run in runs {
        recut_run(&mut out, &base, &depth, &frozen, run, gw, gh, world, land_floor);
    }
    Some(out)
}

/// Cells the valley may never change: water (drawn classification, or below
/// sea level) and every land cell 8-adjacent to water. See the module doc for
/// why the ring makes every bilinear water test exact.
fn frozen_mask(field: &[f32], water: &[u8], gw: usize, gh: usize, sea_level: f64, world: bool) -> Vec<bool> {
    let wet = |i: usize| water[i] != 0 || (field[i] as f64) < sea_level;
    let mut out = vec![false; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            if !wet(y * gw + x) {
                continue;
            }
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    if let Some(j) = index(x as i64 + dx, y as i64 + dy, gw, gh, world) {
                        out[j] = true;
                    }
                }
            }
        }
    }
    out
}

/// The grid index of `(x, y)`, wrapping x in world mode; `None` off the grid.
fn index(x: i64, y: i64, gw: usize, gh: usize, world: bool) -> Option<usize> {
    if y < 0 || y >= gh as i64 {
        return None;
    }
    let x = if world { x.rem_euclid(gw as i64) } else { x };
    if x < 0 || x >= gw as i64 {
        return None;
    }
    Some(y as usize * gw + x as usize)
}

/// Fills the carve back in: every carved, unfrozen cell is relaxed to the
/// harmonic mean of its four neighbours (a membrane spanning the trench from
/// its uncarved banks), never below its carved height -- the carve only ever
/// lowered, so the fill only ever raises. Returns the filled field and, per
/// cell, how deep the carve was there (`0` wherever it did not cut).
fn uncarve(field: &[f32], carved: &[u8], frozen: &[bool], gw: usize, gh: usize, world: bool) -> (Vec<f32>, Vec<f32>) {
    let mut u: Vec<f64> = field.iter().map(|&v| v as f64).collect();
    let cells: Vec<usize> = (0..gw * gh).filter(|&i| carved[i] != 0 && !frozen[i]).collect();
    let is_cell = {
        let mut m = vec![false; gw * gh];
        for &i in &cells {
            m[i] = true;
        }
        m
    };
    // Start each carved cell at the mean of its uncarved 8-neighbours, so the
    // relaxation starts near the bank rather than at the trench floor.
    // A speed-up only: without it the sweeps still converge (a mutant that
    // drops it survives the tests, 2026-09-28, and is equivalent while the
    // loop reaches FILL_TOL).
    for &i in &cells {
        let (x, y) = ((i % gw) as i64, (i / gw) as i64);
        let (mut s, mut k) = (0.0, 0usize);
        for dy in -1i64..=1 {
            for dx in -1i64..=1 {
                if let Some(j) = index(x + dx, y + dy, gw, gh, world).filter(|&j| !is_cell[j]) {
                    s += u[j];
                    k += 1;
                }
            }
        }
        if k > 0 {
            u[i] = (s / k as f64).max(field[i] as f64);
        }
    }
    for sweep in 0..FILL_SWEEPS {
        let mut moved = 0.0f64;
        // Alternate the visiting order, so the fill does not drift the way
        // one-directional Gauss-Seidel does.
        let order: Box<dyn Iterator<Item = &usize>> = if sweep % 2 == 0 { Box::new(cells.iter()) } else { Box::new(cells.iter().rev()) };
        for &i in order {
            let (x, y) = ((i % gw) as i64, (i / gw) as i64);
            let (mut s, mut k) = (0.0, 0usize);
            for (dx, dy) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
                if let Some(j) = index(x + dx, y + dy, gw, gh, world) {
                    s += u[j];
                    k += 1;
                }
            }
            if k == 0 {
                continue;
            }
            // The floor is the carve itself: the fill only raises. Where every
            // bank is higher than the trench (the carve's own shape) the mean
            // is already above it, so a mutant dropping this survives the
            // tests; it guards a carved cell beside a frozen, lower one.
            let v = (s / k as f64).max(field[i] as f64);
            moved = moved.max((v - u[i]).abs());
            u[i] = v;
        }
        if moved < FILL_TOL {
            break;
        }
    }
    let depth: Vec<f32> = (0..gw * gh).map(|i| if is_cell[i] { (u[i] - field[i] as f64).max(0.0) as f32 } else { 0.0 }).collect();
    (u.into_iter().map(|v| v as f32).collect(), depth)
}

/// The valley's cross-section at `d` cells from the drawn centreline, for a
/// drawn half-width `hw`: `1` across the bed (`d <= hw - 0.5`, which exists
/// only for a river wider than one cell), then `(1 - q²)²` down to `0` at
/// `hw + shoulder`. Smooth (its slope is continuous and zero at the rim), so a
/// line moving by a fraction of a cell moves the shading by a fraction, never
/// by a whole cell -- the step this replaces.
fn profile(d: f64, hw: f64) -> f64 {
    let bed = (hw - 0.5).max(0.0);
    let rim = hw + SHOULDER_MIN_CELLS.max(hw);
    if d <= bed {
        return 1.0;
    }
    if d >= rim {
        return 0.0;
    }
    let q = (d - bed) / (rim - bed);
    let a = 1.0 - q * q;
    a * a
}

/// Cuts one drawn run's valley into `out`: at each cell near a drawn piece,
/// `base - depth · profile(distance)`, keeping the deeper of that and what is
/// there already (so confluences take the deeper valley), floored at
/// `land_floor`, and never on a frozen cell.
///
/// The depth at each drawn point is the deepest carve within its half-width
/// plus [`DEPTH_REACH_CELLS`], averaged along the line over
/// [`DEPTH_SMOOTH_CELLS`] of arc length each way. Where the carve cut nothing
/// (a run crossing a kept lake, or a sculpted world with no carve) the depth
/// is `0` and nothing is cut.
#[allow(clippy::too_many_arguments)]
fn recut_run(out: &mut [f32], base: &[f32], depth: &[f32], frozen: &[bool], run: &DrawnRun, gw: usize, gh: usize, world: bool, land_floor: f64) {
    let n = run.pts.len();
    if n < 2 || run.widths.len() != n {
        return;
    }
    let hw: Vec<f64> = run.widths.iter().map(|&w| (w as f64 * 0.5).max(0.0)).collect();
    // The carve's depth under each drawn point.
    let raw: Vec<f64> = (0..n)
        .map(|k| {
            let (px, py) = (run.pts[k].0 as f64, run.pts[k].1 as f64);
            let reach = hw[k] + DEPTH_REACH_CELLS;
            let r = reach.ceil() as i64;
            let (cx, cy) = (px.floor() as i64, py.floor() as i64);
            let mut best = 0.0f64;
            for y in cy - r..=cy + r {
                for x in cx - r..=cx + r {
                    let d = ((x as f64 + 0.5 - px).powi(2) + (y as f64 + 0.5 - py).powi(2)).sqrt();
                    if d > reach {
                        continue;
                    }
                    if let Some(j) = index(x, y, gw, gh, world) {
                        best = best.max(depth[j] as f64);
                    }
                }
            }
            best
        })
        .collect();
    // Smoothed along the arc.
    let mut arc = vec![0.0f64; n];
    for k in 1..n {
        let (a, b) = (run.pts[k - 1], run.pts[k]);
        arc[k] = arc[k - 1] + ((b.0 - a.0) as f64).hypot((b.1 - a.1) as f64);
    }
    let dep: Vec<f64> = (0..n)
        .map(|k| {
            let (mut s, mut c) = (0.0, 0usize);
            let mut j = k;
            while j > 0 && arc[k] - arc[j - 1] <= DEPTH_SMOOTH_CELLS {
                j -= 1;
            }
            while j < n && arc[j] - arc[k] <= DEPTH_SMOOTH_CELLS {
                s += raw[j];
                c += 1;
                j += 1;
            }
            s / c.max(1) as f64
        })
        .collect();
    for &(p0, p1) in &run.pieces {
        let p1 = p1.min(n);
        for k in p0..p1.saturating_sub(1) {
            let (a, b) = (run.pts[k], run.pts[k + 1]);
            let (ax, ay, bx, by) = (a.0 as f64, a.1 as f64, b.0 as f64, b.1 as f64);
            // A segment jumping more than half the map is the world seam, not
            // a river reach (`split_river_polylines`' own rule).
            if (bx - ax).abs() > gw as f64 * 0.5 {
                continue;
            }
            if dep[k] <= 0.0 && dep[k + 1] <= 0.0 {
                continue;
            }
            let rim = |h: f64| h + SHOULDER_MIN_CELLS.max(h);
            let r = rim(hw[k]).max(rim(hw[k + 1]));
            let x0 = (ax.min(bx) - r).floor() as i64;
            let x1 = (ax.max(bx) + r).ceil() as i64;
            let y0 = (ay.min(by) - r).floor() as i64;
            let y1 = (ay.max(by) + r).ceil() as i64;
            let (ex, ey) = (bx - ax, by - ay);
            let l2 = ex * ex + ey * ey;
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let Some(j) = index(x, y, gw, gh, world) else { continue };
                    if frozen[j] {
                        continue;
                    }
                    let (qx, qy) = (x as f64 + 0.5, y as f64 + 0.5);
                    let t = if l2 > 0.0 { (((qx - ax) * ex + (qy - ay) * ey) / l2).clamp(0.0, 1.0) } else { 0.0 };
                    let d = (qx - (ax + ex * t)).hypot(qy - (ay + ey * t));
                    let h = hw[k] + (hw[k + 1] - hw[k]) * t;
                    let w = profile(d, h);
                    if w <= 0.0 {
                        continue;
                    }
                    let dd = dep[k] + (dep[k + 1] - dep[k]) * t;
                    let v = (base[j] as f64 - dd * w).max(land_floor);
                    if v < out[j] as f64 {
                        out[j] = v as f32;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flat plain at 0.6 with a straight carved trench along row `ty`,
    /// `depth` deep, over `x0..x1`.
    fn plain_with_trench(gw: usize, gh: usize, ty: usize, x0: usize, x1: usize, depth: f32) -> (Vec<f32>, Vec<u8>) {
        let mut f = vec![0.6f32; gw * gh];
        let mut c = vec![0u8; gw * gh];
        for x in x0..x1 {
            f[ty * gw + x] -= depth;
            c[ty * gw + x] = 1;
        }
        (f, c)
    }

    fn run(pts: Vec<(f32, f32)>, width: f32) -> DrawnRun {
        let n = pts.len();
        DrawnRun {
            pts,
            widths: vec![width; n],
            colors: vec![[0.0, 0.0, 1.0, 1.0]; n],
            orders: vec![1; n],
            discharge: vec![f32::NAN; n],
            pieces: vec![(0, n)],
            reach: Vec::new(),
            own_order: 1,
        }
    }

    // Protects: an undrawn carve leaves no groove -- the carve under no drawn
    // line is filled back to its banks (Ruling BD: "no carve shading appears
    // where no river is drawn").
    #[test]
    fn a_carve_with_no_drawn_river_is_filled_to_its_banks() {
        let (gw, gh) = (40, 30);
        let (f, c) = plain_with_trench(gw, gh, 15, 5, 35, 0.01);
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, &[], gw, gh, 0.42, false).unwrap();
        for x in 5..35 {
            let v = s[15 * gw + x];
            assert!((v - 0.6).abs() < 1e-5, "cell {x}: {v} should be back at the bank's 0.6");
        }
        // The true field is untouched: the caller's copy is still carved.
        assert!((f[15 * gw + 20] - 0.59).abs() < 1e-6);
    }

    // Protects: the fill relaxes, it does not just copy the bank -- a trench
    // seven cells wide, whose middle row has no uncarved neighbour to start
    // from, still ends within 1e-4 of its banks.
    #[test]
    fn a_wide_carve_is_filled_all_the_way_across() {
        let (gw, gh) = (40, 30);
        let mut f = vec![0.6f32; gw * gh];
        let mut c = vec![0u8; gw * gh];
        for y in 12..19 {
            for x in 5..35 {
                f[y * gw + x] = 0.55;
                c[y * gw + x] = 1;
            }
        }
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, &[], gw, gh, 0.42, false).unwrap();
        let mid = s[15 * gw + 20];
        assert!((mid - 0.6).abs() < 1e-4, "the middle of a 7-cell trench is filled to the banks: {mid}");
    }

    // Protects: the depth is smoothed along the line -- where the carve's
    // depth steps from 0.01 to 0.002 at x = 20, the valley floor there is in
    // between, not either step.
    #[test]
    fn a_step_in_the_carve_depth_is_smoothed_along_the_line() {
        let (gw, gh) = (40, 30);
        let mut f = vec![0.6f32; gw * gh];
        let mut c = vec![0u8; gw * gh];
        for x in 5..35 {
            f[15 * gw + x] = if x < 20 { 0.59 } else { 0.598 };
            c[15 * gw + x] = 1;
        }
        // Render points every quarter cell, as the drawn line is dense.
        let r = run((20..140).map(|k| (k as f32 * 0.25 + 0.5, 15.5)).collect(), 1.0);
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, &[r], gw, gh, 0.42, false).unwrap();
        let v = s[15 * gw + 20] as f64;
        assert!(v > 0.5901 && v < 0.5979, "the floor at the step is between the two depths: {v}");
        // Well away from the step each side keeps its own depth.
        assert!((s[15 * gw + 10] as f64 - 0.59).abs() < 1e-4 && (s[15 * gw + 30] as f64 - 0.598).abs() < 1e-4);
    }

    // Protects: the valley follows the DRAWN line, not the carve. A trench on
    // row 15 with the drawn line on row 18 ends with its lowest cells on row
    // 18, and row 15 back near the bank.
    #[test]
    fn the_valley_is_cut_along_the_drawn_line_not_the_carve() {
        let (gw, gh) = (40, 30);
        let (f, c) = plain_with_trench(gw, gh, 15, 5, 35, 0.01);
        // Drawn 1.5 cells off the carve's centres: row 15's centre is 15.5,
        // the line runs at 17.0, inside DEPTH_REACH_CELLS of the carve.
        let r = run((5..35).map(|x| (x as f32 + 0.5, 17.0)).collect(), 1.0);
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, &[r], gw, gh, 0.42, false).unwrap();
        let at = |x: usize, y: usize| s[y * gw + x];
        // Rows 16 and 17 straddle the line at 0.5 cells: the valley floor.
        assert!(at(20, 16) < 0.595 && at(20, 17) < 0.595, "the valley floor is on the line: {} {}", at(20, 16), at(20, 17));
        // Row 15 (the carve) is now on the valley's shoulder, shallower than
        // the floor, and row 13 is outside the valley altogether.
        assert!(at(20, 15) > at(20, 16), "the carve row is shallower than the line's rows");
        assert!((at(20, 13) - 0.6).abs() < 1e-6, "outside the rim the ground is untouched: {}", at(20, 13));
    }

    // Protects: the valley is as deep as the carve was -- the re-cut moves the
    // valley, it does not deepen or flatten it.
    #[test]
    fn the_valley_is_as_deep_as_the_carve() {
        let (gw, gh) = (40, 30);
        let (f, c) = plain_with_trench(gw, gh, 15, 5, 35, 0.01);
        let r = run((5..35).map(|x| (x as f32 + 0.5, 15.5)).collect(), 1.0);
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, &[r], gw, gh, 0.42, false).unwrap();
        let floor = s[15 * gw + 20];
        assert!((floor - 0.59).abs() < 1e-4, "the floor on the line is the carve's depth, 0.59: {floor}");
    }

    // Protects: shading changes by a fraction of a cell when the line moves by
    // a fraction of a cell -- no whole-cell step. The cross-section at
    // successive sub-cell offsets of the line changes monotonically and by
    // less than the full depth per quarter cell.
    #[test]
    fn a_sub_cell_shift_of_the_line_moves_the_valley_by_a_fraction() {
        let (gw, gh) = (40, 30);
        let (f, c) = plain_with_trench(gw, gh, 15, 5, 35, 0.01);
        let mut prev: Option<f32> = None;
        for q in 0..5 {
            let yl = 15.5 + q as f32 * 0.25;
            let r = run((5..35).map(|x| (x as f32 + 0.5, yl)).collect(), 1.0);
            let s = valley_shade_field(&f, &vec![0; gw * gh], &c, &[r], gw, gh, 0.42, false).unwrap();
            let v = s[17 * gw + 20];
            if let Some(p) = prev {
                assert!(v <= p + 1e-7, "row 17 deepens as the line approaches it");
                assert!(p - v < 0.006, "a quarter-cell move changes row 17 by less than the full 0.01: {}", p - v);
            }
            prev = Some(v);
        }
    }

    // Protects: water, and land beside water, keep their true height, so no
    // shoreline or water colour can move; and no land is cut below sea level.
    // The carve runs from the coast (x = 2, beside the sea at x < 2) past a
    // lake cell (30, 14), and a low patch of land (row 17, 0.45) lies on the
    // valley's shoulder, where the cut would take it under the sea.
    #[test]
    fn water_and_its_banks_are_never_changed_and_land_stays_land() {
        let (gw, gh) = (40, 30);
        let sea = 0.42;
        let (mut f, c) = plain_with_trench(gw, gh, 15, 2, 35, 0.15);
        let mut water = vec![0u8; gw * gh];
        water[14 * gw + 30] = 2;
        for y in 0..gh {
            for x in 0..2 {
                f[y * gw + x] = 0.3;
                water[y * gw + x] = 1;
            }
        }
        for x in 20..24 {
            f[17 * gw + x] = 0.45;
        }
        let r = run((2..35).map(|x| (x as f32 + 0.5, 15.5)).collect(), 3.0);
        let s = valley_shade_field(&f, &water, &c, &[r], gw, gh, sea, false).unwrap();
        for y in 0..gh {
            for x in 0..gw {
                let i = y * gw + x;
                let near_water = (-1i64..=1).any(|dy| {
                    (-1i64..=1).any(|dx| index(x as i64 + dx, y as i64 + dy, gw, gh, false).is_some_and(|j| water[j] != 0 || (f[j] as f64) < sea))
                });
                if near_water {
                    assert_eq!(s[i].to_bits(), f[i].to_bits(), "({x},{y}) is water or beside it and must keep its height");
                } else {
                    assert!((s[i] as f64) >= sea, "({x},{y}) is land and must stay above sea level: {}", s[i]);
                }
            }
        }
        // Positive controls: the ring cells the assertions guard lie on the
        // carve and the valley (so a mutant that ignored the ring would move
        // them), the valley was cut beside the carve, and the low patch was
        // reached and floored rather than left alone.
        assert!(c[15 * gw + 2] != 0 && c[15 * gw + 30] != 0, "premise: the ring cells are carved");
        assert!(s[14 * gw + 10] < 0.5, "the valley exists on row 14: {}", s[14 * gw + 10]);
        let low = s[17 * gw + 21] as f64;
        assert!(low < 0.44 && low >= sea, "the low patch is cut down to the land floor, not below: {low}");
    }

    // Protects: "no value" is not a plausible value -- a grid of the wrong
    // length is refused, and the caller shades the true field.
    #[test]
    fn a_mismatched_grid_is_refused() {
        let f = vec![0.6f32; 12];
        assert!(valley_shade_field(&f, &[0; 12], &[0; 11], &[], 4, 3, 0.42, false).is_none());
        assert!(valley_shade_field(&f, &[0; 11], &[0; 12], &[], 4, 3, 0.42, false).is_none());
        assert!(valley_shade_field(&f, &[0; 12], &[0; 12], &[], 4, 4, 0.42, false).is_none());
    }

    // Protects: the profile's shape -- full depth on the bed, zero at the rim,
    // and a shoulder of at least SHOULDER_MIN_CELLS (literals, not the
    // constants themselves).
    #[test]
    fn the_profile_is_full_on_the_bed_and_zero_at_the_rim() {
        assert_eq!(profile(0.0, 0.5), 1.0);
        assert_eq!(profile(1.5, 0.5), 0.0, "half-width 0.5 + a 1-cell shoulder");
        assert!(profile(1.49, 0.5) > 0.0);
        assert_eq!(profile(1.5, 2.0), 1.0, "a 4-cell river's bed runs to 1.5 cells");
        assert_eq!(profile(4.0, 2.0), 0.0, "half-width 2 + a 2-cell shoulder");
        assert!((profile(0.75, 0.5) - (1.0 - 0.25f64).powi(2)).abs() < 1e-12, "q = 0.5 at mid-shoulder");
    }
}
