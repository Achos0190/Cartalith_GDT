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
//! **Water is never touched, and no water test reads this field.** Every cell
//! that is water in the drawn classification or below sea level keeps the true
//! height ([`is_frozen`]), and since 2026-09-28 every water test in the three
//! render paths reads the world's height instead (`RenderCtx::water_height`:
//! the screen's per-cell test, the export's bilinear one, and the deep-zoom
//! tiles' amplified one through their water tile,
//! `render::render_biome_tile_rgba_water`). So the land beside water may be
//! shaded like any other land -- the last carved cell of a channel at its
//! mouth is filled and re-cut with the rest (`OUTSTANDING_WORK.md` row "The
//! colour-texture repaint ...; and the carved cell beside water keeps its
//! step") -- and no coast, lake shore or sea colour moves. Until that change
//! a ring of land 8-adjacent to water was frozen too, because the bilinear and
//! amplified sea tests read the shaded height and a raised land corner would
//! have moved the shore.
//!
//! **No land is taken below sea level.** A re-cut cell is floored at
//! `sea_level + cartalith_hydrology::CARVE_LAND_MARGIN`, the carve's own land
//! floor, so no land cell becomes water to the renderer.
//!
//! **Only what the generation carve dug is filled; a user's sculpt never is**
//! (`OUTSTANDING_WORK.md` row "RV-3 follow-ups", item 1, 2026-09-28). The
//! lock mask (`WorldState::river_mask`) is written by two hands: the carve at
//! generation ([`LOCK_CARVE`]) and a Sculpt commit's River stamps, which lock
//! the channel the user dug. `WorldGen::sculpt_commit` marks the second kind
//! [`LOCK_SCULPT`] ([`mark_sculpt_locks`]), and only `LOCK_CARVE` cells are
//! filled, so a channel sculpted by hand keeps its shaded groove whether or
//! not a river is drawn along it. And on a carved cell, only the carve's own
//! depth is filled: the fill starts from the carve's floor
//! (`WorldState::river_floor`, the height the carve left), so anything that
//! lowered the cell below that floor afterwards -- a sculpt brush, an erosion
//! pass -- stays in the shading. Before this, a hand-sculpted River channel
//! with no drawn river was filled flat, and so was a sculpt that deepened a
//! generated channel.

use crate::river_stroke::DrawnRun;

/// `WorldState::river_mask` value for a cell the generation carve lowered
/// (`generate_terrain` writes `1` for every cell `carve_channel_network`
/// returns). The only value [`valley_shade_field`] fills. Saves written before
/// 2026-09-28 hold only this value, which is what the carve alone wrote.
pub(crate) const LOCK_CARVE: u8 = 1;

/// `WorldState::river_mask` value for a cell a Sculpt commit locked -- a River
/// stamp's `enforce_channel_descent` cut, or a re-lock of a carved cell with a
/// new floor ([`mark_sculpt_locks`]). Every lock reader tests `!= 0`, so this
/// is as locked as [`LOCK_CARVE`]; it only tells the shading "a user dug
/// this, keep it". It is saved with the mask (`SAVEFILE_COMPAT.md`,
/// `rasters/river_mask.u8`), so the distinction survives a reopen.
pub(crate) const LOCK_SCULPT: u8 = 2;

/// The lock mask to store after a Sculpt commit: `mask`/`floor` are the
/// Sculpt editor's lock arrays after the commit (`WaterState`, which writes
/// only `0`/`1`, or carries forward the values it was seeded with), and
/// `prev_mask`/`prev_floor` are the world's arrays before it.
///
/// A cell locked now and unlocked before, already [`LOCK_SCULPT`], or
/// re-locked with a different floor (a River stamp's descent recorded the
/// user's new bed over a carved cell) becomes `LOCK_SCULPT`; every other
/// locked cell keeps its previous value (a generation cell stays
/// [`LOCK_CARVE`]: the commit's `enforce_river_channels` re-clamp moves the
/// height back to its floor, not the floor). Unlocked is `0`.
///
/// Must never unlock or lock a cell `mask` does not: it relabels, it does not
/// decide what is locked -- that stays the engine's (`sculpt_commit.rs`).
/// With no previous mask (`None`, or the wrong length) every lock is treated
/// as new, so all of it reads as the user's: a world with no generation carve
/// has no carve to fill.
pub(crate) fn mark_sculpt_locks(prev_mask: Option<&[u8]>, prev_floor: Option<&[f32]>, mask: &[u8], floor: &[f32]) -> Vec<u8> {
    let n = mask.len();
    let prev_mask = prev_mask.filter(|p| p.len() == n);
    let prev_floor = prev_floor.filter(|p| p.len() == n && floor.len() == n);
    (0..n)
        .map(|i| {
            if mask[i] == 0 {
                return 0;
            }
            // An earlier sculpt lock (2) falls to the last arm and keeps its
            // value: a lock is never relabelled back to the carve's.
            match prev_mask.map_or(0, |p| p[i]) {
                0 => LOCK_SCULPT,
                p => match prev_floor {
                    Some(pf) if pf[i].to_bits() != floor[i].to_bits() => LOCK_SCULPT,
                    _ => p,
                },
            }
        })
        .collect()
}

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
/// - `carved`: the lock mask (`WorldState::river_mask`): [`LOCK_CARVE`] where
///   the generation carve lowered a cell, [`LOCK_SCULPT`] where a Sculpt
///   commit locked one, `0` elsewhere; `gw * gh`. Only `LOCK_CARVE` is filled.
/// - `floor`: the carve's floor under each lock (`WorldState::river_floor`),
///   `gw * gh`, or `None` when the world has none -- the fill then starts
///   from the cell's current height, which is the floor itself on a world
///   nothing has edited since the carve.
/// - `runs`: the drawn river network (`WorldGen::river_geometry_any`), whose
///   render points are in grid-cell space with a cell's centre at `x + 0.5`.
///
/// Returns `None` when any grid is the wrong length -- the caller then shades
/// the true field, never a guessed one.
///
/// Deterministic: a pure function of its inputs, visited in a fixed order.
///
/// **Cost.** It runs on Godot's main thread, inside the repaint
/// (`WorldGen::valley_shade_field`), once per river-network key -- after a
/// generate and after every edit. So it touches the whole grid only to copy
/// `field` into the result and to list the carved cells; everything else is
/// proportional to the carve and the drawn network, never to the grid: the
/// water ring is tested per visited cell ([`is_frozen`]) instead of built as a
/// grid-sized mask, the fill relaxes a compact array of the carved cells
/// only, and the re-cut gathers its cut per touched cell before writing it
/// once. Measured with `_rv3cost_probe.gd` (`STATUS.md`, "RV-3 follow-ups").
#[allow(clippy::too_many_arguments)]
pub fn valley_shade_field(field: &[f32], water: &[u8], carved: &[u8], floor: Option<&[f32]>, runs: &[DrawnRun], gw: usize, gh: usize, sea_level: f64, world: bool) -> Option<Vec<f32>> {
    let n = gw.checked_mul(gh)?;
    if n == 0 || field.len() != n || water.len() != n || carved.len() != n || floor.is_some_and(|f| f.len() != n) {
        return None;
    }
    let grid = Grid { field, water, gw, gh, sea_level, world };
    let mut out = field.to_vec();
    let depth = uncarve(&grid, &mut out, carved, floor);
    // The re-cut's deepest cut per cell (runs overlap at confluences, and the
    // deeper valley wins), gathered first and written once, so every cut is
    // taken from the filled height and never from another run's cut.
    // `vec![0.0; n]` is a zeroed allocation: only the pages a river touches
    // are ever written.
    let mut cut = vec![0.0f32; n];
    let mut touched: Vec<usize> = Vec::new();
    for run in runs {
        recut_run(&mut cut, &mut touched, &depth, run, gw, gh, world);
    }
    let land_floor = sea_level + cartalith_hydrology::CARVE_LAND_MARGIN;
    for &j in &touched {
        if grid.is_frozen(j) {
            continue;
        }
        let v = (out[j] as f64 - cut[j] as f64).max(land_floor);
        if v < out[j] as f64 {
            out[j] = v as f32;
        }
    }
    Some(out)
}

/// The inputs every stage reads, borrowed once.
struct Grid<'a> {
    field: &'a [f32],
    water: &'a [u8],
    gw: usize,
    gh: usize,
    sea_level: f64,
    world: bool,
}

impl Grid<'_> {
    /// Water in the drawn classification, or below sea level.
    fn wet(&self, i: usize) -> bool {
        self.water[i] != 0 || (self.field[i] as f64) < self.sea_level
    }

    /// See [`is_frozen`].
    fn is_frozen(&self, i: usize) -> bool {
        is_frozen(self, i)
    }
}

/// A cell the valley may never change: water, in the drawn classification or
/// below sea level. Everything else is land, and the fill only raises land and
/// the re-cut never takes it below `sea_level + CARVE_LAND_MARGIN`, so no cell
/// changes side and every per-cell water test is unchanged whatever this
/// field holds.
///
/// **The ring of land beside water is no longer frozen** (2026-09-28, the
/// repaint row's second half, `OUTSTANDING_WORK.md` "...; and the carved cell
/// beside water keeps its step"). It was, because a raised land corner of a
/// bilinear cell whose other corner is water moves the sub-cell sea-level
/// contour, and the export's bilinear test and the tiles' amplified test read
/// this SHADED height -- so filling the last carved cell at a mouth would have
/// moved the drawn shore. Those tests now read the world's height
/// (`RenderCtx::water_height`, and the tiles' water tile), so the ring shades
/// like the rest of the channel and the one-cell step at every mouth and
/// inlet is filled. Measured: `lod_bridge::tests::
/// the_water_mask_is_the_same_whatever_the_shaded_field_holds` (a fixture) and
/// `a_generated_world_draws_the_same_water_with_the_valley_field` (a generated
/// world with this module's field): every tile of levels 0-3 and the export's
/// water, pixel for pixel. The screen's per-cell test cannot move -- no cell
/// changes side.
fn is_frozen(g: &Grid, i: usize) -> bool {
    g.wet(i)
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

/// Fills the carve back in: every [`LOCK_CARVE`], unfrozen cell is relaxed to
/// the harmonic mean of its four neighbours (a membrane spanning the trench
/// from its uncarved banks), never below the carve's floor -- the carve only
/// ever lowered, so the fill only ever raises. Writes the filled height into
/// `out` (which holds `field` on entry) and returns, per cell, how deep the
/// carve was there (`0` wherever it did not cut).
///
/// **Only the carve's own depth.** A cell's surface for the fill is
/// `max(field, floor)`: the carve's floor, or the cell's height where
/// something has since raised it (an erosion deposit). Whatever lowered the
/// cell below its floor after the carve -- a sculpt, an erosion pass -- is
/// subtracted back from the filled height, so it stays in the shading. A
/// [`LOCK_SCULPT`] cell is never filled. With `floor` `None`, or equal to
/// `field` (a world nothing edited since the carve), the surface is `field`
/// and this is the RV-3 fill unchanged.
///
/// Must never write a cell that is not `LOCK_CARVE` and unfrozen.
///
/// The relaxation runs over a compact array of the fill cells alone, each
/// with its four neighbours pre-resolved (a fill cell's slot, or a fixed
/// height), so a sweep costs the carve's size, not the grid's.
fn uncarve(g: &Grid, out: &mut [f32], carved: &[u8], floor: Option<&[f32]>) -> Vec<f32> {
    let (gw, gh, world, field) = (g.gw, g.gh, g.world, g.field);
    // Ascending, so a neighbour's slot is found by binary search.
    let cells: Vec<usize> = (0..gw * gh).filter(|&i| carved[i] == LOCK_CARVE && !g.is_frozen(i)).collect();
    let slot = |j: usize| cells.binary_search(&j).ok();
    // The carve's surface under each fill cell.
    let surf: Vec<f64> = cells.iter().map(|&i| floor.map_or(field[i], |f| field[i].max(f[i])) as f64).collect();
    // Per fill cell: the sum of its fixed (non-fill) 4-neighbours, how many
    // 4-neighbours it has on the grid, and its fill neighbours' slots.
    let mut fixed = vec![0.0f64; cells.len()];
    let mut count = vec![0u8; cells.len()];
    let mut nbrs: Vec<[u32; 4]> = vec![[0; 4]; cells.len()];
    let mut nn = vec![0u8; cells.len()];
    let mut u = vec![0.0f64; cells.len()];
    for (k, &i) in cells.iter().enumerate() {
        let (x, y) = ((i % gw) as i64, (i / gw) as i64);
        for (dx, dy) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
            if let Some(j) = index(x + dx, y + dy, gw, gh, world) {
                count[k] += 1;
                match slot(j) {
                    Some(s) => {
                        nbrs[k][nn[k] as usize] = s as u32;
                        nn[k] += 1;
                    }
                    None => fixed[k] += field[j] as f64,
                }
            }
        }
        // Start each carved cell at the mean of its uncarved 8-neighbours, so
        // the relaxation starts near the bank rather than at the trench
        // floor. A speed-up only: without it the sweeps still converge (a
        // mutant that drops it survives the tests, 2026-09-28, and is
        // equivalent while the loop reaches FILL_TOL).
        let (mut s, mut c) = (0.0, 0usize);
        for dy in -1i64..=1 {
            for dx in -1i64..=1 {
                if let Some(j) = index(x + dx, y + dy, gw, gh, world).filter(|&j| slot(j).is_none()) {
                    s += field[j] as f64;
                    c += 1;
                }
            }
        }
        u[k] = if c > 0 { (s / c as f64).max(surf[k]) } else { surf[k] };
    }
    for sweep in 0..FILL_SWEEPS {
        let mut moved = 0.0f64;
        // Alternate the visiting order, so the fill does not drift the way
        // one-directional Gauss-Seidel does. Both orders reach the same fixed
        // point once the loop meets FILL_TOL, so a mutant that sweeps one way
        // only survives the tests (2026-09-28) and is equivalent on every
        // fill that converges; alternating converges in fewer sweeps.
        let mut visit = |k: usize| {
            if count[k] == 0 {
                return;
            }
            let s = fixed[k] + nbrs[k][..nn[k] as usize].iter().map(|&m| u[m as usize]).sum::<f64>();
            // The floor is the carve itself: the fill only raises. Where every
            // bank is higher than the trench (the carve's own shape) the mean
            // is already above it, so a mutant dropping this survives the
            // tests; it guards a carved cell beside a frozen, lower one.
            let v = (s / count[k] as f64).max(surf[k]);
            moved = moved.max((v - u[k]).abs());
            u[k] = v;
        };
        if sweep % 2 == 0 {
            (0..cells.len()).for_each(&mut visit);
        } else {
            (0..cells.len()).rev().for_each(&mut visit);
        }
        if moved < FILL_TOL {
            break;
        }
    }
    let mut depth = vec![0.0f32; gw * gh];
    for (k, &i) in cells.iter().enumerate() {
        // What lowered the cell below the carve's surface after the carve.
        let later = surf[k] - field[i] as f64;
        out[i] = (u[k] - later) as f32;
        depth[i] = (u[k] - surf[k]).max(0.0) as f32;
    }
    depth
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

/// Gathers one drawn run's valley cut into `cut`: at each cell near a drawn
/// piece, `depth · profile(distance)`, keeping the deeper of that and what is
/// there already (so confluences take the deeper valley). Each cell's first
/// cut is recorded in `touched`. [`valley_shade_field`] then subtracts the cut
/// from the filled height, floored at the land floor, skipping frozen cells;
/// this function must never write the height itself.
///
/// The depth at each drawn point is the deepest carve within its half-width
/// plus [`DEPTH_REACH_CELLS`], averaged along the line over
/// [`DEPTH_SMOOTH_CELLS`] of arc length each way. Where the carve cut nothing
/// (a run crossing a kept lake, or a sculpted world with no carve) the depth
/// is `0` and nothing is cut.
fn recut_run(cut: &mut [f32], touched: &mut Vec<usize>, depth: &[f32], run: &DrawnRun, gw: usize, gh: usize, world: bool) {
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
                    let (qx, qy) = (x as f64 + 0.5, y as f64 + 0.5);
                    let t = if l2 > 0.0 { (((qx - ax) * ex + (qy - ay) * ey) / l2).clamp(0.0, 1.0) } else { 0.0 };
                    let d = (qx - (ax + ex * t)).hypot(qy - (ay + ey * t));
                    let h = hw[k] + (hw[k + 1] - hw[k]) * t;
                    let w = profile(d, h);
                    if w <= 0.0 {
                        continue;
                    }
                    let dd = ((dep[k] + (dep[k + 1] - dep[k]) * t) * w) as f32;
                    if dd > cut[j] {
                        if cut[j] == 0.0 {
                            touched.push(j);
                        }
                        cut[j] = dd;
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
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, None, &[], gw, gh, 0.42, false).unwrap();
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
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, None, &[], gw, gh, 0.42, false).unwrap();
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
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, None, &[r], gw, gh, 0.42, false).unwrap();
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
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, None, &[r], gw, gh, 0.42, false).unwrap();
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
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, None, &[r], gw, gh, 0.42, false).unwrap();
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
            let s = valley_shade_field(&f, &vec![0; gw * gh], &c, None, &[r], gw, gh, 0.42, false).unwrap();
            let v = s[17 * gw + 20];
            if let Some(p) = prev {
                assert!(v <= p + 1e-7, "row 17 deepens as the line approaches it");
                assert!(p - v < 0.006, "a quarter-cell move changes row 17 by less than the full 0.01: {}", p - v);
            }
            prev = Some(v);
        }
    }

    // Protects: water keeps its true height and no land is cut below sea
    // level, so no cell changes side; and the land beside water is shaded
    // like the rest of the channel (the ring is no longer frozen, 2026-09-28).
    // The carve runs from the coast (x = 2, beside the sea at x < 2) past a
    // lake cell (30, 14), and a low patch of land (row 17, 0.45) lies on the
    // valley's shoulder, where the cut would take it under the sea.
    #[test]
    fn water_is_never_changed_and_land_stays_land() {
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
        let s = valley_shade_field(&f, &water, &c, None, &[r], gw, gh, sea, false).unwrap();
        for y in 0..gh {
            for x in 0..gw {
                let i = y * gw + x;
                if water[i] != 0 || (f[i] as f64) < sea {
                    assert_eq!(s[i].to_bits(), f[i].to_bits(), "({x},{y}) is water and must keep its height");
                } else {
                    assert!((s[i] as f64) >= sea, "({x},{y}) is land and must stay above sea level: {}", s[i]);
                }
            }
        }
        // Positive controls: the carved cell beside the sea (2, 15) is shaded
        // now -- a mutant that froze the ring again would leave it at the
        // carve -- the valley was cut beside the carve, and the low patch was
        // reached and floored rather than left alone. (The cell beside the
        // lake, (30, 15), is re-cut to exactly its carve's depth on the drawn
        // line, so it cannot tell frozen from shaded and is not used.)
        assert!(c[15 * gw + 2] != 0, "premise: the cell beside the sea is carved");
        assert_ne!(s[15 * gw + 2].to_bits(), f[15 * gw + 2].to_bits(), "the carved cell beside the sea is shaded with its channel");
        assert!(s[14 * gw + 10] < 0.5, "the valley exists on row 14: {}", s[14 * gw + 10]);
        let low = s[17 * gw + 21] as f64;
        assert!(low < 0.44 && low >= sea, "the low patch is cut down to the land floor, not below: {low}");
    }

    // Protects: "no value" is not a plausible value -- a grid of the wrong
    // length is refused, and the caller shades the true field.
    #[test]
    fn a_mismatched_grid_is_refused() {
        let f = vec![0.6f32; 12];
        assert!(valley_shade_field(&f, &[0; 12], &[0; 11], None, &[], 4, 3, 0.42, false).is_none());
        assert!(valley_shade_field(&f, &[0; 11], &[0; 12], None, &[], 4, 3, 0.42, false).is_none());
        assert!(valley_shade_field(&f, &[0; 12], &[0; 12], None, &[], 4, 4, 0.42, false).is_none());
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

    // Protects: a channel the user sculpted (locked by a Sculpt commit, value
    // 2), with no drawn river on it, keeps its groove in the shading -- the
    // row "RV-3 follow-ups", item 1. The same trench locked by the generation
    // carve (value 1) is the positive control: it is filled.
    #[test]
    fn a_sculpted_channel_with_no_drawn_river_keeps_its_groove() {
        let (gw, gh) = (40, 30);
        let (f, mut c) = plain_with_trench(gw, gh, 15, 5, 35, 0.01);
        for m in c.iter_mut().filter(|m| **m != 0) {
            *m = 2;
        }
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, Some(&f), &[], gw, gh, 0.42, false).unwrap();
        for x in 5..35 {
            assert_eq!(s[15 * gw + x].to_bits(), f[15 * gw + x].to_bits(), "sculpted cell {x} keeps its carved height");
        }
        let (f, c) = plain_with_trench(gw, gh, 15, 5, 35, 0.01);
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, Some(&f), &[], gw, gh, 0.42, false).unwrap();
        assert!((s[15 * gw + 20] - 0.6).abs() < 1e-5, "control: the carve's own trench is filled: {}", s[15 * gw + 20]);
    }

    // Protects: only the carve's own depth is filled. A carved trench (floor
    // 0.59) that a sculpt then lowered by a further 0.02 over x = 15..25 ends
    // 0.02 below the banks there, and at the banks elsewhere.
    #[test]
    fn a_sculpt_that_deepened_a_carved_channel_keeps_its_extra_depth() {
        let (gw, gh) = (40, 30);
        let (mut f, c) = plain_with_trench(gw, gh, 15, 5, 35, 0.01);
        let floor = f.clone();
        for x in 15..25 {
            f[15 * gw + x] -= 0.02;
        }
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, Some(&floor), &[], gw, gh, 0.42, false).unwrap();
        for x in 16..24 {
            let v = s[15 * gw + x];
            assert!((v - 0.58).abs() < 1e-4, "cell {x}: the sculpt's 0.02 stays below the filled 0.6: {v}");
        }
        assert!((s[15 * gw + 8] - 0.6).abs() < 1e-4, "outside the sculpt the carve is filled: {}", s[15 * gw + 8]);
    }

    // Protects: the re-cut is as deep as the CARVE was, not as deep as the
    // cell now is. Where a sculpt lowered a carved cell a further 0.02 below
    // its floor, the valley cut along a drawn line there takes the carve's
    // 0.01 off the sculpted 0.58 (0.57), not 0.03.
    #[test]
    fn the_recut_under_a_sculpt_is_as_deep_as_the_carve_not_the_sculpt() {
        let (gw, gh) = (40, 30);
        let (mut f, c) = plain_with_trench(gw, gh, 15, 5, 35, 0.01);
        let floor = f.clone();
        for x in 15..25 {
            f[15 * gw + x] -= 0.02;
        }
        let r = run((5..35).map(|x| (x as f32 + 0.5, 15.5)).collect(), 1.0);
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, Some(&floor), &[r], gw, gh, 0.42, false).unwrap();
        let v = s[15 * gw + 20];
        assert!((v - 0.57).abs() < 1e-3, "sculpted 0.58 less the carve's 0.01: {v}");
    }

    // Protects: land below sea level that the drawn classification does not
    // call water (a dry depression under the datum) is frozen as water -- the
    // renderer's own `h < sea_level` test calls it sea -- while the carved cell
    // diagonal to it is no longer frozen with it (2026-09-28).
    #[test]
    fn a_cell_below_sea_level_is_frozen_even_when_not_classified_water() {
        let (gw, gh) = (40, 30);
        let (mut f, c) = plain_with_trench(gw, gh, 15, 5, 35, 0.01);
        // Diagonal to the carved cell (20, 15), so the fill (which reads
        // 4-neighbours) is not pulled down by it.
        f[16 * gw + 21] = 0.40;
        let mut c = c;
        c[16 * gw + 21] = 1; // carved too: only its being below sea level keeps it
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, None, &[], gw, gh, 0.42, false).unwrap();
        assert_eq!(s[16 * gw + 21].to_bits(), f[16 * gw + 21].to_bits(), "the below-sea cell keeps its height");
        // (21, 15) is 4-adjacent to the below-sea cell, which holds it at the
        // carve's floor; (20, 15), beside it, is lifted most of the way.
        assert!(s[15 * gw + 20] > 0.595, "the carved cell beside it is filled with the rest: {}", s[15 * gw + 20]);
        assert!(s[15 * gw + 21] >= f[15 * gw + 21], "and none is taken below its carve");
        assert!((s[15 * gw + 10] - 0.6).abs() < 1e-5, "control: away from it the carve is filled");
    }

    // Protects: the item this change exists for -- the last carved cell of a
    // channel at the sea no longer keeps its one-cell step. With no drawn
    // river over it the fill lifts it from the carve's 0.45 toward its banks
    // (the sea cell beside it holds it below them), and it stays land.
    #[test]
    fn the_carved_cell_at_the_mouth_loses_its_step() {
        let (gw, gh) = (40, 30);
        let (mut f, c) = plain_with_trench(gw, gh, 15, 2, 35, 0.15);
        let mut water = vec![0u8; gw * gh];
        for y in 0..gh {
            for x in 0..2 {
                f[y * gw + x] = 0.3;
                water[y * gw + x] = 1;
            }
        }
        let s = valley_shade_field(&f, &water, &c, None, &[], gw, gh, 0.42, false).unwrap();
        let mouth = s[15 * gw + 2] as f64;
        assert!(mouth > 0.45 + 0.05, "the mouth cell is lifted off the carve's 0.45: {mouth}");
        assert!(mouth >= 0.42 && mouth <= 0.6, "and stays land, no higher than its banks: {mouth}");
    }

    // Protects: a carved cell raised above its floor after the carve (an
    // erosion deposit) is filled from its raised height, not its old floor --
    // the fill never lifts ground above its banks.
    #[test]
    fn a_carved_cell_raised_above_its_floor_is_filled_to_the_banks_not_above() {
        let (gw, gh) = (40, 30);
        let (mut f, c) = plain_with_trench(gw, gh, 15, 5, 35, 0.01);
        let floor = f.clone();
        for x in 15..25 {
            f[15 * gw + x] = 0.595;
        }
        let s = valley_shade_field(&f, &vec![0; gw * gh], &c, Some(&floor), &[], gw, gh, 0.42, false).unwrap();
        for x in 5..35 {
            let v = s[15 * gw + x];
            assert!((v - 0.6).abs() < 1e-4, "cell {x}: filled to the banks' 0.6: {v}");
        }
    }

    // Protects: on a world nothing has edited since the carve (every locked
    // cell's height is its floor), passing the floor changes nothing -- the
    // RV-3 look on a fresh world is unchanged by the item-1 fix.
    #[test]
    fn a_floor_equal_to_the_field_changes_nothing() {
        let (gw, gh) = (40, 30);
        let (f, c) = plain_with_trench(gw, gh, 15, 5, 35, 0.01);
        let r = run((5..35).map(|x| (x as f32 + 0.5, 16.2)).collect(), 1.5);
        let a = valley_shade_field(&f, &vec![0; gw * gh], &c, None, std::slice::from_ref(&r), gw, gh, 0.42, false).unwrap();
        let b = valley_shade_field(&f, &vec![0; gw * gh], &c, Some(&f), &[r], gw, gh, 0.42, false).unwrap();
        assert!(a.iter().zip(&b).all(|(x, y)| x.to_bits() == y.to_bits()));
        assert!(a[16 * gw + 20] < 0.595, "premise: the valley was cut");
    }

    // Protects: "no value" is not a plausible value -- a floor of the wrong
    // length is refused rather than read short.
    #[test]
    fn a_floor_of_the_wrong_length_is_refused() {
        let f = vec![0.6f32; 12];
        assert!(valley_shade_field(&f, &[0; 12], &[0; 12], Some(&[0.6; 11]), &[], 4, 3, 0.42, false).is_none());
    }

    // Protects: the relabelling after a Sculpt commit (`mark_sculpt_locks`):
    // a new lock, a re-lock with a new floor, and an earlier sculpt lock read
    // as the user's (2); a generation lock whose floor did not move stays the
    // carve's (1); unlocked stays 0; with no previous mask every lock is new.
    #[test]
    fn a_sculpt_commit_marks_only_the_locks_it_made() {
        let prev = [0u8, 1, 1, 2, 0, 1];
        let prev_floor = [0.0f32, 0.5, 0.5, 0.4, 0.0, 0.5];
        let mask = [1u8, 1, 1, 1, 0, 0];
        let floor = [0.3f32, 0.5, 0.45, 0.4, 0.0, 0.5];
        assert_eq!(mark_sculpt_locks(Some(&prev), Some(&prev_floor), &mask, &floor), vec![2, 1, 2, 2, 0, 0]);
        assert_eq!(mark_sculpt_locks(None, None, &mask, &floor), vec![2, 2, 2, 2, 0, 0]);
        assert_eq!(mark_sculpt_locks(Some(&prev[..5]), Some(&prev_floor), &mask, &floor), vec![2, 2, 2, 2, 0, 0], "a short previous mask is no mask");
    }
}
