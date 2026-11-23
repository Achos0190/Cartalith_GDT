//! Owner Ruling GO (2026-11-22, `LARGE_ITEM_RULINGS.md`; Ruling GN gives the
//! why): faction borders that come to rest on terrain. Three pieces, from
//! `BORDER_FORMATION_RESEARCH.md` §3:
//!
//! - **O1, a territory friction field** ([`territory_friction`]). The territory
//!   sweep used to run over `build_travel_cost`, whose land cost is `1 + 50·slope²`
//!   and is 1.000 on the median land cell (research §1.4, measured), so the
//!   cost-distance Voronoi was a weighted straight-line one. The friction field
//!   multiplies that cost by relief (height above the range-scale mean: ridges
//!   and uplands), by the biome friction the router already uses (forest,
//!   scrub, ice/tundra/desert) and by the lowland marsh penalty, discounts the
//!   relief on the route-corridor (pass) detector's cells, and charges rivers
//!   both ways through an edge cost: cheap to travel along, dear to cross.
//!   Canyons are dear through their rivers and are snapping walls (O2).
//! - **O2, seam snapping** ([`snap_seams`]). A finishing step: within a bounded
//!   band either side of each faction seam, the two sides are re-flooded from
//!   the band's edges with ridge crests, canyon floors and major rivers as
//!   walls, so a seam with such a feature inside the band comes to rest on it.
//! - **O4, the drawn border line** ([`border_polylines`]): the seams between
//!   owners as smoothed polylines in continuous grid coordinates, for the
//!   shell's overlay (which the export draws too).
//!
//! Owner Ruling HR (2026-11-23): provinces follow the same logic, behind the
//! same strength. [`provinces_terrain`] grows each faction's provinces from
//! their seats over the same friction and snaps the seams between them to the
//! same walls, never across a faction border; [`carry_provinces`] keeps that
//! shape when the claim grid is edited; [`province_polylines`] draws the seams.
//!
//! **The road cost is never touched.** The friction is a second field built
//! from it; `build_travel_cost` and everything that reads it (roads, the
//! journey planner, the settlement explanation) are bit-identical whatever
//! the strength.
//!
//! **Strength 0 is today's territory by control flow**, not by arithmetic:
//! [`assign_territory_terrain`] calls [`crate::assign_territory`] and returns.
//! No JS golden reads territory (`assign_territory`'s own doc: the reference
//! has no algorithmic territory), so this module has no reference to match;
//! it is new design under `DECISIONS.md` §7p, verified by the tests below and
//! by `cartalith-godot`'s `border_harness` measurement.
//!
//! What it must never do: change `build_travel_cost`'s output, leave a
//! settlement's own cell to another faction than the sweep gave it, or claim
//! a water cell.

use rayon::prelude::*;

use crate::{NamedSettlement, Province};

/// The rasters the friction and the snapping read, all `gw * gh`, all as
/// `compute_civilisation` already holds them.
pub struct BorderTerrain<'a> {
    /// The height field (`WorldState::field`, normalised units).
    pub field: &'a [f32],
    /// The civ layer's river order (the network roads are priced on).
    pub river_order: &'a [i16],
    /// `build_biome_raster`'s 1-based biome index.
    pub biome: &'a [u8],
    /// `build_route_corridors`' pass/corridor strength in `0..=1`.
    pub corridors: &'a [f32],
    /// Full-resolution flow accumulation, for the marsh penalty.
    pub flow: &'a [f32],
    /// `river_flow_thresh` for this grid, for the marsh penalty.
    pub flow_thresh: f64,
    pub gw: usize,
    pub gh: usize,
    pub sea: f64,
    /// X-wrapped world.
    pub world: bool,
}

/// Relief cost at strength 1: a cell [`TPI_REF`] or more above its window
/// mean costs `1 + RELIEF_K` times its road cost. Labelled judgement, swept
/// with the window on the three research seeds at 384 wide (`border_harness`,
/// 2026-11-22; 6, 12 and 24 tried): 12 put the most border on high ground
/// without a faction losing more than 1.7x its area.
const RELIEF_K: f64 = 12.0;

/// Local-relief window radius as a fraction of the grid width: `gw / 48`,
/// about 83 km on the 4 000 km research worlds, i.e. range scale. Swept
/// (`gw / 32`, `/48`, `/96`): the research's own `gw / 85`-ish ridge window
/// read the terrain's grid-scale texture and left borders ragged and no more
/// often on high ground; `/48` and `/32` both put borders on high ground,
/// `/32` at the cost of river borders on one seed. Labelled judgement.
const TPI_RADIUS_DIV: usize = 48;

/// Height above the window mean (normalised field units) at which the relief
/// term saturates. Between the 90th and 97th percentile of `|tpi|` on land at
/// this window on the research seeds, at 384 and 2048 wide (0.037-0.080 and
/// 0.071-0.139 measured 2026-11-22; `border_harness` prints the distribution),
/// so only the upper tenth or so of the land pays the full relief cost.
/// Only height ABOVE the mean costs: a valley floor is where a river and a
/// road run and is not a barrier (taking `|tpi|` made valleys walls).
/// Canyons enter as their rivers' crossings and as [`snap_seams`] walls.
/// A steepness term was tried too (slope at the route corridors' `/6`
/// normalisation, then at `/20`, `/30`, `/40`) and on every setting put less
/// border on high ground and shrank one faction up to 3x, so there is none.
const TPI_REF: f64 = 0.06;

/// How much of the relief term a full-strength corridor (pass) cell keeps
/// off: `1 - PASS_K * corridor`. Q6's default: passes keep low friction, so
/// they pull a border over the ridge to whoever holds the approach.
const PASS_K: f64 = 0.8;

/// Travel along a river (both cells on a major river) costs `1 - RIVER_ALONG`
/// of the bank cost at strength 1: the connector role (Q4's default). Swept
/// (0, 0.3, 0.5): 0.5 let a river-mouth capital ride its river far enough
/// that borders sat on rivers less often than land does on two seeds.
const RIVER_ALONG: f64 = 0.3;

/// Crossing a major river at strength 1 costs as much as this many flat cells
/// of a 1024-wide grid (scaled by `gw / 1024`, so a crossing is the same
/// fraction of a capital's reach at every resolution). The barrier role.
/// Labelled judgement, swept with `RELIEF_K`.
const RIVER_CROSS_1024: f64 = 30.0;

/// Seam-snapping reach as a fraction of the grid width (O2's bounded band):
/// a seam moves at most `gw / SNAP_REACH_DIV` cells (6 at 384, 32 at 2048;
/// about 60 km on the 4 000 km research worlds). Labelled judgement.
const SNAP_REACH_DIV: usize = 64;

/// Normalised relief (`|tpi| / TPI_REF`) a crest or canyon floor needs to be a
/// snapping wall. Labelled judgement: low enough that a real range's crest is
/// continuous, high enough that a plain's ripples are not walls.
const FEATURE_TPI: f64 = 0.35;

/// Signed local relief: each land cell's height minus the mean height of the
/// `(2r+1)^2` window around it (clamped at the grid edge), by a summed-area
/// table. Water cells read 0.
fn local_relief(t: &BorderTerrain) -> Vec<f32> {
    local_relief_of(t.field, t)
}

/// [`local_relief`] of `field` (the raw height field or a smoothed copy),
/// with `t`'s grid, sea level and window.
fn local_relief_of(field: &[f32], t: &BorderTerrain) -> Vec<f32> {
    let (gw, gh) = (t.gw, t.gh);
    let r = (gw / TPI_RADIUS_DIV).max(2);
    let w1 = gw + 1;
    let mut sat = vec![0f64; w1 * (gh + 1)];
    for y in 0..gh {
        let mut row = 0f64;
        for x in 0..gw {
            row += field[y * gw + x] as f64;
            sat[(y + 1) * w1 + x + 1] = sat[y * w1 + x + 1] + row;
        }
    }
    let mut out = vec![0f32; gw * gh];
    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        let h = field[i] as f64;
        if h < t.sea {
            return;
        }
        let (x, y) = (i % gw, i / gw);
        let (x0, x1) = (x.saturating_sub(r), (x + r + 1).min(gw));
        let (y0, y1) = (y.saturating_sub(r), (y + r + 1).min(gh));
        let s = sat[y1 * w1 + x1] - sat[y0 * w1 + x1] - sat[y1 * w1 + x0] + sat[y0 * w1 + x0];
        let mean = s / ((x1 - x0) * (y1 - y0)) as f64;
        *o = (h - mean) as f32;
    });
    out
}

/// O1's per-cell friction and river mask, plus the strength they were built
/// at. `node` is the cost [`crate::road_dijkstra_multi`]'s default edge reads;
/// [`Friction::edge`] adds the river's two signs.
pub struct Friction {
    /// Land: road cost x relief x biome x marsh multipliers. Water: infinite.
    pub node: Vec<f32>,
    /// A major river cell (order >= `river_class::NAVIGABLE_ORDER`).
    pub river: Vec<bool>,
    strength: f64,
    cross: f64,
    gw: usize,
}

/// Builds O1's friction from the road cost `cost` (`build_travel_cost`'s
/// output, read, never written). Per land cell: `cost x (1 + s·RELIEF_K·ridge·pass)
/// x biome^s x marsh^s`, with `ridge` the height above the window mean over
/// [`TPI_REF`] (clamped to `0..=1`) and `pass = 1 - PASS_K·corridor`. Water
/// stays infinite. `strength` in `0..=1` scales every term: the
/// relief and river terms linearly, the biome and marsh multipliers as powers,
/// so `strength == 0` gives back `cost` exactly and the river edge terms
/// vanish (though [`assign_territory_terrain`] never builds one at 0).
pub fn territory_friction(cost: &[f32], t: &BorderTerrain, strength: f64) -> Friction {
    let s = strength.clamp(0.0, 1.0);
    let tpi = local_relief(t);
    let node: Vec<f32> = (0..t.gw * t.gh)
        .into_par_iter()
        .map(|i| {
            let c = cost[i] as f64;
            if !c.is_finite() {
                return f32::INFINITY;
            }
            let ridge = (tpi[i] as f64 / TPI_REF).clamp(0.0, 1.0);
            let pass = 1.0 - PASS_K * (t.corridors[i] as f64).clamp(0.0, 1.0);
            let relief = 1.0 + s * RELIEF_K * ridge * pass;
            let biome = crate::civ_biome_friction(t.biome[i]).powf(s);
            let marsh = crate::civ_swamp_penalty(Some(t.flow), t.flow_thresh, t.field[i] as f64, t.sea, i).powf(s);
            (c * relief * biome * marsh) as f32
        })
        .collect();
    let river = t.river_order.iter().map(|&o| o >= crate::river_class::NAVIGABLE_ORDER).collect();
    Friction { node, river, strength: s, cross: RIVER_CROSS_1024 * t.gw as f64 / 1024.0, gw: t.gw }
}

/// Box mean of `a` over a `(2r+1)^2` window clamped at the grid edge.
fn box_blur(a: &[f32], gw: usize, gh: usize, r: usize) -> Vec<f32> {
    let w1 = gw + 1;
    let mut sat = vec![0f64; w1 * (gh + 1)];
    for y in 0..gh {
        let mut row = 0f64;
        for x in 0..gw {
            row += a[y * gw + x] as f64;
            sat[(y + 1) * w1 + x + 1] = sat[y * w1 + x + 1] + row;
        }
    }
    (0..gw * gh)
        .into_par_iter()
        .map(|i| {
            let (x, y) = (i % gw, i / gw);
            let (x0, x1) = (x.saturating_sub(r), (x + r + 1).min(gw));
            let (y0, y1) = (y.saturating_sub(r), (y + r + 1).min(gh));
            ((sat[y1 * w1 + x1] - sat[y0 * w1 + x1] - sat[y1 * w1 + x0] + sat[y0 * w1 + x0]) / ((x1 - x0) * (y1 - y0)) as f64) as f32
        })
        .collect()
}

impl Friction {
    /// The step cost from cell `i` to its neighbour `j` (`dx`, `dy` in
    /// `-1..=1`): the default `len * (node_i + node_j) / 2`, then
    ///
    /// - both on a major river: times `1 - s * RIVER_ALONG` (travel along);
    /// - one on a river and one off: plus half a crossing (a bank each way,
    ///   so entering and leaving a one-cell river costs one whole crossing);
    /// - a diagonal step between two off-river cells whose two corner cells
    ///   are both river (slipping between diagonal river cells): plus one
    ///   whole crossing, so an 8-connected river cannot be crossed for free.
    pub fn edge(&self, i: usize, j: usize, dx: isize, dy: isize) -> f64 {
        let diag = dx != 0 && dy != 0;
        let len = if diag { std::f64::consts::SQRT_2 } else { 1.0 };
        let base = len * 0.5 * (self.node[i] as f64 + self.node[j] as f64);
        let (ri, rj) = (self.river[i], self.river[j]);
        if ri && rj {
            base * (1.0 - self.strength * RIVER_ALONG)
        } else if ri != rj {
            base + 0.5 * self.strength * self.cross
        } else if diag {
            // The corners share a row with one end and a column with the
            // other; `j`'s column is `i`'s column plus `dx` (mod wrap), so
            // read them off `j` and `i` rather than re-deriving the wrap.
            let (xi, xj) = (i % self.gw, j % self.gw);
            let (ci, cj) = (i - xi + xj, j - xj + xi);
            if self.river[ci] && self.river[cj] {
                base + self.strength * self.cross
            } else {
                base
            }
        } else {
            base
        }
    }
}

/// Territory for a world under Ruling GO. `strength <= 0`: exactly
/// [`crate::assign_territory`] (by control flow). Otherwise the same
/// cost-distance sweep over [`territory_friction`]'s field with its river edge
/// cost, then [`snap_seams`].
pub fn assign_territory_terrain(
    settlements: &[NamedSettlement],
    cost: &[f32],
    t: &BorderTerrain,
    strength: f64,
) -> Vec<i32> {
    if strength <= 0.0 {
        return crate::assign_territory(settlements, cost, t.gw, t.gh, t.world);
    }
    let fr = territory_friction(cost, t, strength);
    let edge = |i: usize, j: usize, dx: isize, dy: isize| fr.edge(i, j, dx, dy);
    let mut owner = crate::assign_territory_with_edges(settlements, &fr.node, t.gw, t.gh, t.world, &edge);
    drop(fr);
    let features = feature_walls(t);
    let pins: Vec<usize> = settlements.iter().map(|s| s.placement.y * t.gw + s.placement.x).collect();
    snap_seams(&mut owner, &features, &pins, t.gw, t.gh, (t.gw / SNAP_REACH_DIV).max(2));
    owner
}

/// Owner Ruling HR: provinces for a world under Ruling GO. `strength <= 0`:
/// exactly [`crate::civ_generate_provinces`] (by control flow; the JS
/// reference `_civGenerateProvinces` is that path). Otherwise:
///
/// 1. the province list is [`crate::civ_generate_provinces`]' own, so every
///    `Province::id` (which the vault links by) is the one the Voronoi path
///    gives for the same seats;
/// 2. each seat grows its province over [`territory_friction`]'s field with
///    its river edge cost, through the same sweep nations use
///    ([`crate::assign_territory_with_edges`]: seats weighted by population),
///    with every step between cells of different factions impassable, so a
///    province never leaves its faction. A seat standing on another faction's
///    land seeds nothing (it would flood that faction);
/// 3. an owned cell no seat reached (an exclave, or a faction whose seat
///    stands outside it) takes the Voronoi path's province, so provinces still
///    partition their faction with no gaps;
/// 4. the seams between one faction's provinces snap to [`feature_walls`] by
///    [`snap_seams`], run once per faction on that faction's cells alone (the
///    rest read as unowned, which `snap_seams` never claims), with the
///    faction's settlements as pins.
///
/// What it must never do: put a cell in a province of another faction than
/// `territory` gives it, leave an owned cell of a seeded faction without a
/// province, or number a province differently from the Voronoi path.
pub fn provinces_terrain(
    settlements: &[NamedSettlement],
    territory: &[i32],
    cost: &[f32],
    t: &BorderTerrain,
    strength: f64,
) -> (Vec<i32>, Vec<Province>) {
    let (gw, gh) = (t.gw, t.gh);
    let (voronoi, list) = crate::civ_generate_provinces(settlements, territory, gw, gh);
    if strength <= 0.0 {
        return (voronoi, list);
    }
    let seats: Vec<NamedSettlement> = list
        .iter()
        .filter_map(|p| {
            let mut s = settlements[p.capital_settlement_index].clone();
            let own = territory[s.placement.y * gw + s.placement.x] == p.faction;
            s.placement.capital = true;
            s.placement.faction = p.id;
            own.then_some(s)
        })
        .collect();
    let fr = territory_friction(cost, t, strength);
    let edge = |i: usize, j: usize, dx: isize, dy: isize| {
        if territory[i] != territory[j] {
            f64::INFINITY
        } else {
            fr.edge(i, j, dx, dy)
        }
    };
    let grown = crate::assign_territory_with_edges(&seats, &fr.node, gw, gh, t.world, &edge);
    drop(fr);
    let mut province: Vec<i32> = grown.iter().zip(&voronoi).zip(territory).map(|((&g, &v), &f)| if f > 0 && g == 0 { v } else { g }).collect();
    let walls = feature_walls(t);
    let reach = (gw / SNAP_REACH_DIV).max(2);
    let factions: std::collections::BTreeSet<i32> = list.iter().map(|p| p.faction).collect();
    for f in factions {
        let mut own: Vec<i32> = province.iter().zip(territory).map(|(&p, &o)| if o == f { p } else { 0 }).collect();
        let pins: Vec<usize> = settlements.iter().filter(|s| s.placement.faction == f).map(|s| s.placement.y * gw + s.placement.x).collect();
        snap_seams(&mut own, &walls, &pins, gw, gh, reach);
        for ((p, &o), &f2) in province.iter_mut().zip(&own).zip(territory) {
            if f2 == f {
                *p = o;
            }
        }
    }
    (province, list)
}

/// Owner Ruling HR: provinces rebuilt over an edited claim grid (a paint
/// commit, a polity move, a city-state change) on a world whose provinces
/// follow the terrain. The list is [`crate::civ_generate_provinces`]' over the
/// new grid (ids as that path numbers them). A cell keeps the province it had
/// in `old` when that province is still seated (the old id at a new seat's own
/// cell names it; an old id two seats claim is dropped) and still belongs to
/// the cell's faction; any other owned cell takes the Voronoi path's province.
/// The friction rasters are not resident after a generate, so this keeps the
/// grown and snapped shape rather than regrowing it.
///
/// ponytail: painted-in cells get the nearest seat, not a friction growth or a
/// snap; regrowing needs the river, biome and corridor rasters rebuilt (a
/// Recompute does that).
///
/// `old` of the wrong length: exactly the Voronoi path.
pub fn carry_provinces(old: &[i32], settlements: &[NamedSettlement], territory: &[i32], gw: usize, gh: usize) -> (Vec<i32>, Vec<Province>) {
    let (mut province, list) = crate::civ_generate_provinces(settlements, territory, gw, gh);
    if old.len() != gw * gh {
        return (province, list);
    }
    let mut map: std::collections::HashMap<i32, Option<i32>> = Default::default();
    for p in &list {
        let s = &settlements[p.capital_settlement_index].placement;
        let o = old[s.y * gw + s.x];
        if o > 0 {
            map.entry(o).and_modify(|v| *v = None).or_insert(Some(p.id));
        }
    }
    for (i, c) in province.iter_mut().enumerate() {
        if let Some(Some(p)) = map.get(&old[i]) {
            if list[*p as usize - 1].faction == territory[i] {
                *c = *p;
            }
        }
    }
    (province, list)
}

/// O2's walls: major-river cells, ridge crests and canyon floors. A crest is
/// a land cell at least [`FEATURE_TPI`] above its window mean that is a strict
/// local maximum along at least one of the four axes (so the wall is a line,
/// not the whole high ground); a canyon floor is the same below the mean and a
/// minimum. Water is never a wall (it is never owned, so never in a band).
pub fn feature_walls(t: &BorderTerrain) -> Vec<bool> {
    let (gw, gh) = (t.gw, t.gh);
    let reach = (gw / SNAP_REACH_DIV).max(2);
    // Read on a lightly smoothed field: the raw field's grid-scale texture
    // makes a crest test fire in combs, and a comb of walls saws the seam.
    let fs = box_blur(t.field, gw, gh, (gw / 192).max(1));
    let tpi = local_relief_of(&fs, t);
    let mut out = vec![false; gw * gh];
    out.par_iter_mut().enumerate().for_each(|(i, o)| {
        if (t.field[i] as f64) < t.sea {
            return;
        }
        if t.river_order[i] >= crate::river_class::NAVIGABLE_ORDER {
            *o = true;
            return;
        }
        let rel = tpi[i] as f64 / TPI_REF;
        if rel.abs() < FEATURE_TPI {
            return;
        }
        let (x, y) = (i % gw, i / gw);
        if x == 0 || y == 0 || x + 1 == gw || y + 1 == gh {
            return;
        }
        let h = fs[i];
        let up = rel > 0.0;
        let pairs = [(i - 1, i + 1), (i - gw, i + gw), (i - gw - 1, i + gw + 1), (i - gw + 1, i + gw - 1)];
        *o = pairs.iter().any(|&(a, b)| if up { h > fs[a] && h > fs[b] } else { h < fs[a] && h < fs[b] });
    });
    // Drop wall fragments shorter than twice the reach (8-connected): a seam
    // should come to rest on a range or a river, not on a knoll.
    let min_len = 2 * reach;
    let mut seen = vec![false; gw * gh];
    let mut comp = Vec::new();
    for s0 in 0..gw * gh {
        if !out[s0] || seen[s0] {
            continue;
        }
        comp.clear();
        comp.push(s0);
        seen[s0] = true;
        let mut k = 0;
        while k < comp.len() {
            let i = comp[k];
            k += 1;
            let (x, y) = ((i % gw) as isize, (i / gw) as isize);
            for dy in -1..=1isize {
                for dx in -1..=1isize {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= gw as isize || ny >= gh as isize {
                        continue;
                    }
                    let j = ny as usize * gw + nx as usize;
                    if out[j] && !seen[j] {
                        seen[j] = true;
                        comp.push(j);
                    }
                }
            }
        }
        if comp.len() < min_len {
            for &i in &comp {
                out[i] = false;
            }
        }
    }
    out
}

/// O2: snaps each faction-to-faction seam onto a feature wall within `reach`.
///
/// The band is every owned cell within `reach` 4-steps of a seam (a cell
/// with a 4-neighbour of another owning faction). Each band cell is re-owned
/// by a 4-connected flood that starts, labelled with their own owner, from the
/// owned cells just outside the band, and that cannot enter a wall cell. With no wall in the band the two floods meet near where
/// the seam was; with a wall across it, each side stops at the wall, so the
/// seam lands on it. A wall cell then takes the owner of its nearest flooded
/// 4-neighbour (its current owner on a tie or when none is flooded); a band
/// cell no flood reaches keeps its owner. Finally, any 4-connected run of
/// changed cells holding one of the `pins` (settlement cells) is put back, so
/// no settlement changes faction. Unowned cells are never claimed and owned
/// cells are never released.
pub fn snap_seams(owner: &mut [i32], walls: &[bool], pins: &[usize], gw: usize, gh: usize, reach: usize) {
    use std::collections::VecDeque;
    let n = gw * gh;
    let nbrs = |i: usize| {
        let (x, y) = (i % gw, i / gw);
        let mut v = [usize::MAX; 4];
        if x > 0 { v[0] = i - 1; }
        if x + 1 < gw { v[1] = i + 1; }
        if y > 0 { v[2] = i - gw; }
        if y + 1 < gh { v[3] = i + gw; }
        v
    };
    // Band: BFS distance from the seam, up to `reach`.
    const FAR: u32 = u32::MAX;
    let mut band = vec![FAR; n];
    let mut q = VecDeque::new();
    for i in 0..n {
        let o = owner[i];
        if o > 0 && nbrs(i).iter().any(|&j| j != usize::MAX && owner[j] > 0 && owner[j] != o) {
            band[i] = 0;
            q.push_back(i);
        }
    }
    if q.is_empty() {
        return;
    }
    while let Some(i) = q.pop_front() {
        if band[i] as usize >= reach {
            continue;
        }
        for j in nbrs(i) {
            if j != usize::MAX && owner[j] > 0 && band[j] == FAR {
                band[j] = band[i] + 1;
                q.push_back(j);
            }
        }
    }
    // Seeds: owned cells just outside the band, and pinned cells inside it.
    let mut label = vec![0i32; n];
    let mut dist = vec![FAR; n];
    let mut q = VecDeque::new();
    for i in 0..n {
        if band[i] == FAR && owner[i] > 0 && nbrs(i).iter().any(|&j| j != usize::MAX && band[j] != FAR) {
            label[i] = owner[i];
            dist[i] = 0;
            q.push_back(i);
        }
    }
    while let Some(i) = q.pop_front() {
        for j in nbrs(i) {
            if j != usize::MAX && band[j] != FAR && !walls[j] && dist[j] == FAR {
                dist[j] = dist[i] + 1;
                label[j] = label[i];
                q.push_back(j);
            }
        }
    }
    // Wall cells in the band, and cells no flood reached.
    let mut out = owner.to_vec();
    // (Settlement cells are not seeds: a seed would claim a disc around a
    // town. Their protection is the undo below.)
    for i in 0..n {
        if band[i] == FAR {
            continue;
        }
        if dist[i] != FAR {
            out[i] = label[i];
        } else if walls[i] {
            let mut best = (FAR, owner[i]);
            for j in nbrs(i) {
                if j != usize::MAX && dist[j] != FAR && (dist[j] < best.0 || (dist[j] == best.0 && label[j] == owner[i])) {
                    best = (dist[j], label[j]);
                }
            }
            out[i] = best.1;
        }
    }
    // A move may not hand a settlement to another faction: every 4-connected
    // run of changed cells that holds a pinned cell is put back.
    let mut q = VecDeque::new();
    for &p in pins {
        if p < n && out[p] != owner[p] {
            out[p] = owner[p];
            q.push_back(p);
        }
    }
    while let Some(i) = q.pop_front() {
        for j in nbrs(i) {
            if j != usize::MAX && out[j] != owner[j] {
                out[j] = owner[j];
                q.push_back(j);
            }
        }
    }
    owner.copy_from_slice(&out);
}

/// O4: the borders of `owner` as smoothed polylines, in continuous grid
/// coordinates (cell `(x, y)` spans `[x, x+1) x [y, y+1)`, the roads'
/// convention, so the shell draws them with `_point_to_screen`).
///
/// A border edge is the shared side of two 4-adjacent land cells with
/// different owners, at least one owned (so a faction's edge against
/// unclaimed land is drawn, its coast is not). Edges are chained between
/// junctions (corners where other than two edges meet) into runs, and closed
/// rings are traced too; each run becomes its edge midpoints (which turns a
/// 45-degree staircase into a straight diagonal) with its junction ends kept,
/// then two rounds of Chaikin corner cutting. Smoothing moves no point more
/// than half a cell, so a border snapped to a river or crest stays on it.
/// The wrap seam of an x-wrapped world is not an edge.
pub fn border_polylines(owner: &[i32], field: &[f32], sea: f64, gw: usize, gh: usize) -> Vec<Vec<(f32, f32)>> {
    let land = |i: usize| field[i] as f64 >= sea;
    seam_polylines(gw, gh, &|i, j| land(i) && land(j) && owner[i] != owner[j] && (owner[i] > 0 || owner[j] > 0))
}

/// Ruling HR: the seams between provinces of one faction, as
/// [`border_polylines`] draws faction borders (same chaining, same
/// smoothing). Only an edge between two land cells of the same owning faction
/// whose provinces differ: a faction border is [`border_polylines`]' line and
/// is never drawn twice.
pub fn province_polylines(province: &[i32], territory: &[i32], field: &[f32], sea: f64, gw: usize, gh: usize) -> Vec<Vec<(f32, f32)>> {
    let land = |i: usize| field[i] as f64 >= sea;
    seam_polylines(gw, gh, &|i, j| land(i) && land(j) && territory[i] > 0 && territory[i] == territory[j] && province[i] != province[j])
}

/// The shared tracer behind [`border_polylines`] and [`province_polylines`]:
/// every side between 4-adjacent cells `i`, `j` for which `differ(i, j)` holds,
/// chained and smoothed as [`border_polylines`]' doc describes.
fn seam_polylines(gw: usize, gh: usize, differ: &dyn Fn(usize, usize) -> bool) -> Vec<Vec<(f32, f32)>> {
    use std::collections::HashMap;
    // Edges between corner vertices, corner id = vy * (gw + 1) + vx.
    let w1 = gw + 1;
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            if x + 1 < gw && differ(i, i + 1) {
                edges.push((y * w1 + x + 1, (y + 1) * w1 + x + 1));
            }
            if y + 1 < gh && differ(i, i + gw) {
                edges.push(((y + 1) * w1 + x, (y + 1) * w1 + x + 1));
            }
        }
    }
    let mut at: HashMap<usize, Vec<usize>> = HashMap::new();
    for (e, &(a, b)) in edges.iter().enumerate() {
        at.entry(a).or_default().push(e);
        at.entry(b).or_default().push(e);
    }
    let pt = |v: usize| ((v % w1) as f32, (v / w1) as f32);
    let mut used = vec![false; edges.len()];
    let mut runs: Vec<(Vec<usize>, bool)> = Vec::new();
    // Walk from `start` along edge `e` until a junction or the start again.
    let walk = |start: usize, e0: usize, used: &mut Vec<bool>| -> (Vec<usize>, bool) {
        let mut verts = vec![start];
        let (mut v, mut e) = (start, e0);
        loop {
            used[e] = true;
            let (a, b) = edges[e];
            v = if a == v { b } else { a };
            verts.push(v);
            let es = &at[&v];
            if v == start {
                return (verts, true);
            }
            if es.len() != 2 {
                return (verts, false);
            }
            match es.iter().find(|&&f| !used[f]) {
                Some(&f) => e = f,
                None => return (verts, false),
            }
        }
    };
    let mut junctions: Vec<usize> = at.iter().filter(|(_, es)| es.len() != 2).map(|(&v, _)| v).collect();
    junctions.sort_unstable();
    for v in junctions {
        for k in 0..at[&v].len() {
            let e = at[&v][k];
            if !used[e] {
                runs.push(walk(v, e, &mut used));
            }
        }
    }
    for e in 0..edges.len() {
        if !used[e] {
            runs.push(walk(edges[e].0, e, &mut used));
        }
    }
    runs.into_iter()
        .map(|(verts, closed)| {
            let p: Vec<(f32, f32)> = verts.iter().map(|&v| pt(v)).collect();
            let mid = |k: usize| ((p[k].0 + p[k + 1].0) * 0.5, (p[k].1 + p[k + 1].1) * 0.5);
            let mut line: Vec<(f32, f32)> = (0..p.len() - 1).map(mid).collect();
            if closed {
                line.push(line[0]);
            } else {
                line.insert(0, p[0]);
                line.push(p[p.len() - 1]);
            }
            for _ in 0..2 {
                line = chaikin(&line, closed);
            }
            line
        })
        .collect()
}

/// One round of Chaikin corner cutting (each segment replaced by its points at
/// 1/4 and 3/4). An open line keeps its two ends; a closed one (first point ==
/// last) stays closed.
fn chaikin(p: &[(f32, f32)], closed: bool) -> Vec<(f32, f32)> {
    if p.len() < 3 {
        return p.to_vec();
    }
    let lerp = |a: (f32, f32), b: (f32, f32), t: f32| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
    let mut out = Vec::with_capacity(p.len() * 2);
    if !closed {
        out.push(p[0]);
    }
    for k in 0..p.len() - 1 {
        out.push(lerp(p[k], p[k + 1], 0.25));
        out.push(lerp(p[k], p[k + 1], 0.75));
    }
    if closed {
        out.push(out[0]);
    } else {
        out.push(p[p.len() - 1]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NamedSettlement, SettlementKind, SettlementPlacement};

    fn capital(x: usize, y: usize, faction: i32, pop: u32) -> NamedSettlement {
        NamedSettlement {
            tid: 0,
            placement: SettlementPlacement { x, y, suit: 0.5, faction, capital: true, kind: SettlementKind::Capital, coastal: false },
            name: "Test".to_string(),
            pop,
        }
    }

    /// A flat `gw x gh` land world: height 0.5 over sea 0.3, no river, no
    /// corridor, grassland, no flow.
    struct Flat {
        field: Vec<f32>,
        order: Vec<i16>,
        biome: Vec<u8>,
        corr: Vec<f32>,
        flow: Vec<f32>,
        gw: usize,
        gh: usize,
    }
    impl Flat {
        fn new(gw: usize, gh: usize) -> Self {
            let n = gw * gh;
            Flat {
                field: vec![0.5; n],
                order: vec![0; n],
                biome: vec![crate::BIOME_GRASS; n],
                corr: vec![0.0; n],
                flow: vec![0.0; n],
                gw,
                gh,
            }
        }
        fn t(&self) -> BorderTerrain<'_> {
            BorderTerrain {
                field: &self.field,
                river_order: &self.order,
                biome: &self.biome,
                corridors: &self.corr,
                flow: &self.flow,
                flow_thresh: 1.0,
                gw: self.gw,
                gh: self.gh,
                sea: 0.3,
                world: false,
            }
        }
    }

    /// Protects: strength 0 is today's territory exactly (the §7p off state),
    /// on a world where the friction at strength 1 would move the border.
    #[test]
    fn strength_zero_is_assign_territory() {
        let w = river_world(510);
        let t = w.t();
        let cost = crate::build_travel_cost(&w.field, 1024, RH, 0.3);
        let s = [capital(100, 20, 1, 15000), capital(900, 20, 2, 15000)];
        let base = crate::assign_territory(&s, &cost, 1024, RH, false);
        assert_eq!(assign_territory_terrain(&s, &cost, &t, 0.0), base);
        assert_ne!(assign_territory_terrain(&s, &cost, &t, 1.0), base, "the fixture must reach the friction");
    }

    /// Protects: a river is dear to cross. Two equal capitals, a north-south
    /// river four cells east of their midpoint: with the friction the west
    /// capital's reach stops at the river, so the seam moves onto it.
    #[test]
    fn a_river_holds_the_border() {
        let w = river_world(510);
        let t = w.t();
        let cost = crate::build_travel_cost(&w.field, 1024, RH, 0.3);
        let s = [capital(100, 20, 1, 15000), capital(900, 20, 2, 15000)];
        let base = crate::assign_territory(&s, &cost, 1024, RH, false);
        let got = assign_territory_terrain(&s, &cost, &t, 1.0);
        // Before: the seam at the midpoint (x 499/500). After: on the river.
        let row = 20 * 1024;
        assert_eq!(base[row + 505], 2);
        assert_eq!(got[row + 505], 1, "west keeps its bank up to the river");
        assert_eq!(got[row + 515], 2, "east keeps the far bank");
        assert_eq!(got[row + 100], 1);
        assert_eq!(got[row + 900], 2);
    }

    /// Rows of [`river_world`]: more than twice the snapping reach at 1024
    /// wide (16), so its river survives `feature_walls`' fragment filter.
    const RH: usize = 40;

    /// A flat 1024 x 40 world with a north-south major river at column `x`.
    fn river_world(x: usize) -> Flat {
        let mut w = Flat::new(1024, RH);
        for y in 0..RH {
            w.order[y * 1024 + x] = 4;
        }
        w
    }

    /// Protects: the edge cost's three river branches, each against the
    /// literal it must produce on a flat cost-1 grid at strength 1.
    #[test]
    fn river_edge_costs_both_signs() {
        let mut w = Flat::new(1024, 3);
        w.order[1024 + 10] = 4;
        w.order[1024 + 11] = 4;
        // Diagonal pair for the slip test: (20,0) and (21,1) on the river.
        w.order[20] = 4;
        w.order[1024 + 21] = 4;
        let t = w.t();
        let cost = vec![1.0f32; 1024 * 3];
        let f = territory_friction(&cost, &t, 1.0);
        let i = |x: usize, y: usize| y * 1024 + x;
        assert_eq!(f.edge(i(10, 1), i(11, 1), 1, 0), 0.7, "along: 1 - RIVER_ALONG");
        assert_eq!(f.edge(i(9, 1), i(10, 1), 1, 0), 1.0 + 15.0, "a bank: half of a 30-cell crossing");
        assert_eq!(f.edge(i(3, 1), i(4, 1), 1, 0), 1.0, "dry ground untouched");
        // (21,0) -> (20,1): corners (20,0) and (21,1) are both river.
        let d = f.edge(i(21, 0), i(20, 1), -1, 1);
        assert!((d - (std::f64::consts::SQRT_2 + 30.0)).abs() < 1e-12, "diagonal slip pays a crossing, got {d}");
    }

    /// Protects: the relief, pass and biome terms against literals. A one-cell
    /// bump 0.25 above a flat 16 x 16 field sits 0.24 above its 5 x 5 window
    /// mean (radius 2, the floor at this width), far over `TPI_REF`, so it
    /// saturates; a second bump on a full corridor keeps a fifth of its
    /// relief; a forest cell pays the router's 1.6.
    #[test]
    fn node_friction_terms() {
        let mut w = Flat::new(16, 16);
        let bump = 4 * 16 + 4;
        let pass = 10 * 16 + 10;
        w.field[bump] = 0.75;
        w.field[pass] = 0.75;
        w.corr[pass] = 1.0;
        let forest = 12 * 16 + 3;
        w.biome[forest] = crate::BIOME_BOREAL;
        let t = w.t();
        let cost = vec![1.0f32; 256];
        let f = territory_friction(&cost, &t, 1.0);
        assert_eq!(f.node[bump], 13.0, "a ridge cell costs 1 + RELIEF_K");
        assert!((f.node[pass] - (1.0 + 12.0 * 0.2)).abs() < 1e-5, "a pass keeps 1 - PASS_K of it");
        assert_eq!(f.node[forest], 1.6);
        assert_eq!(f.node[bump + 1], 1.0, "below its window mean: no relief cost (valleys are not walls)");
        let half = territory_friction(&cost, &t, 0.5);
        assert_eq!(half.node[bump], 7.0);
        assert!(f.node.iter().zip(&cost).all(|(a, b)| a >= b), "friction never cheapens land");
    }

    /// Protects: `feature_walls` keeps a river longer than twice the reach
    /// (6 at 200 wide) and drops a shorter fragment.
    #[test]
    fn short_wall_fragments_are_dropped() {
        let mut w = Flat::new(200, 40);
        for y in 0..40 {
            w.order[y * 200 + 50] = 4;
        }
        for y in 0..3 {
            w.order[y * 200 + 150] = 4;
        }
        let walls = feature_walls(&w.t());
        assert!((0..40).all(|y| walls[y * 200 + 50]), "a long river is a wall");
        assert!((0..3).all(|y| !walls[y * 200 + 150]), "a 3-cell fragment is not");
        assert_eq!(walls.iter().filter(|&&b| b).count(), 40, "and nothing else is");
    }

    /// Protects: snapping moves a seam onto a wall inside the band, leaves a
    /// wall-free seam where it was, keeps pinned cells, and never claims an
    /// unowned cell.
    #[test]
    fn snap_moves_a_seam_onto_a_wall() {
        let (gw, gh) = (30, 5);
        let mut owner = vec![0i32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                owner[y * gw + x] = if x < 15 { 1 } else { 2 };
            }
        }
        owner[29] = 0; // one unowned cell
        let unchanged = {
            let mut o = owner.clone();
            snap_seams(&mut o, &vec![false; gw * gh], &[], gw, gh, 6);
            o
        };
        assert_eq!(unchanged, owner, "no wall: the seam stays");
        let mut walls = vec![false; gw * gh];
        for y in 0..gh {
            walls[y * gw + 11] = true;
        }
        let mut o = owner.clone();
        snap_seams(&mut o, &walls, &[], gw, gh, 6);
        assert_eq!(o[12], 2, "the strip between wall and seam goes east");
        assert_eq!(o[4 * gw + 14], 2);
        assert_eq!(o[10], 1, "west of the wall stays west");
        assert_eq!(o[11], 1, "the wall goes to its nearer flood");
        assert_eq!(o[29], 0, "unowned stays unowned");
        // A settlement in the strip: the move that would hand it over is undone.
        let pin = 2 * gw + 13;
        let mut o = owner.clone();
        snap_seams(&mut o, &walls, &[pin], gw, gh, 6);
        assert_eq!(o[pin], 1, "a settlement keeps its owner");
        assert_eq!(o[12], 1, "and so does the strip joined to it");
        // A wall beyond the reach does nothing.
        let mut far = vec![false; gw * gh];
        for y in 0..gh {
            far[y * gw + 5] = true;
        }
        let mut o = owner.clone();
        snap_seams(&mut o, &far, &[], gw, gh, 6);
        assert_eq!(o, owner);
    }

    /// Protects: the border line is drawn where owners differ on land, not on
    /// a coast, and its smoothing keeps it within half a cell of the seam.
    #[test]
    fn border_polylines_trace_seams_only() {
        let (gw, gh) = (10, 6);
        let mut field = vec![0.5f32; gw * gh];
        let mut owner = vec![0i32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                owner[y * gw + x] = if x < 5 { 1 } else { 2 };
            }
            field[y * gw + 9] = 0.1; // sea column: never a border
            owner[y * gw + 9] = 0;
        }
        let lines = border_polylines(&owner, &field, 0.3, gw, gh);
        assert_eq!(lines.len(), 1, "one seam");
        let l = &lines[0];
        assert!(l.len() > 2);
        assert!(l.iter().all(|&(x, _)| (x - 5.0).abs() < 1e-6), "a straight seam stays on x = 5");
        assert_eq!(l.first().unwrap().1.min(l.last().unwrap().1), 0.0);
        assert_eq!(l.first().unwrap().1.max(l.last().unwrap().1), 6.0);
        // Staircase: smoothed points stay within half a cell of the edge set.
        for y in 0..gh {
            for x in 0..gw {
                owner[y * gw + x] = if x < 2 + y { 1 } else { 2 };
            }
        }
        let lines = border_polylines(&owner, &field, 0.3, gw, gh);
        for l in &lines {
            for &(x, y) in l {
                let d = (x - (2.0 + y)).abs();
                assert!(d <= 1.0, "point ({x},{y}) strays {d} from the stair");
            }
        }
    }

    /// A city-tier settlement (a province seat, not a capital).
    fn city(x: usize, y: usize, faction: i32, pop: u32) -> NamedSettlement {
        let mut s = capital(x, y, faction, pop);
        s.placement.capital = false;
        s.placement.kind = SettlementKind::City;
        s
    }

    /// The province id at cell `(x, 20)` of a [`RH`]-row, 1024-wide grid.
    fn at(p: &[i32], x: usize) -> i32 {
        p[20 * 1024 + x]
    }

    /// Protects Ruling HR's growth and snap, and its strength-0 off state.
    /// One faction owns a flat 1024 x 40 world; seats at x 100 and 900 split
    /// it at x 500 by nearest seat. A major river at x 530: at strength 1 the
    /// crossing (30 cells) moves the grown seam to about x 515, inside the
    /// snapping reach (16) of the river, and the snap puts it on the river,
    /// so x 525 is the west province's. Plain cost would leave the seam at
    /// 500 (out of reach: x 525 east); growth without the snap leaves it at
    /// about 515 (x 525 east). Strength 0 is the Voronoi path, ids and all.
    #[test]
    fn province_seam_grows_and_snaps_to_a_river() {
        let w = river_world(530);
        let t = w.t();
        let cost = crate::build_travel_cost(&w.field, 1024, RH, 0.3);
        let s = [capital(100, 20, 1, 15000), city(900, 20, 1, 15000)];
        let territory = vec![1i32; 1024 * RH];
        let (vor, vlist) = crate::civ_generate_provinces(&s, &territory, 1024, RH);
        let (p0, l0) = provinces_terrain(&s, &territory, &cost, &t, 0.0);
        assert_eq!(p0, vor, "strength 0 is the Voronoi raster");
        assert_eq!(format!("{l0:?}"), format!("{vlist:?}"));
        assert_eq!((at(&vor, 499), at(&vor, 501), at(&vor, 525)), (1, 2, 2), "the fixture's Voronoi seam is at x 500");
        let (p1, l1) = provinces_terrain(&s, &territory, &cost, &t, 1.0);
        assert_eq!(format!("{l1:?}"), format!("{vlist:?}"), "ids, names and seats as the Voronoi path gives them");
        assert_eq!(at(&p1, 525), 1, "the seam came to rest on the river");
        assert_eq!(at(&p1, 535), 2, "the far bank stays east");
        assert_eq!((at(&p1, 100), at(&p1, 900)), (1, 2), "each seat holds its own id");
        assert!(p1.iter().all(|&p| p == 1 || p == 2), "no gap");
        // Strength 0 never reaches the growth or the snap, on a world where
        // both would move the seam: a river inside the reach of it, and
        // unequal seats (the growth weights them by population).
        let w2 = river_world(510);
        let s2 = [capital(100, 20, 1, 60000), city(900, 20, 1, 1000)];
        let (vor2, _) = crate::civ_generate_provinces(&s2, &territory, 1024, RH);
        let (q0, _) = provinces_terrain(&s2, &territory, &cost, &w2.t(), 0.0);
        assert_eq!(q0, vor2, "strength 0 is the Voronoi raster here too");
        let (q1, _) = provinces_terrain(&s2, &territory, &cost, &w2.t(), 1.0);
        assert_ne!(q1, vor2, "the fixture must reach the growth");
    }

    /// Protects: a seat never changes province in the snap (seats are pins).
    /// Seats at x 490 and 512 meet near x 501; a river at x 517 is inside the
    /// reach east of the eastern seat, so the snap alone would hand the cells
    /// up to the river, the eastern seat's own among them, to the west. The
    /// pin puts that whole run back.
    #[test]
    fn province_snap_never_moves_a_seat() {
        let w = river_world(517);
        let t = w.t();
        let cost = crate::build_travel_cost(&w.field, 1024, RH, 0.3);
        let s = [capital(490, 20, 1, 15000), city(512, 20, 1, 15000)];
        let territory = vec![1i32; 1024 * RH];
        let (p, _) = provinces_terrain(&s, &territory, &cost, &t, 1.0);
        assert_eq!(at(&p, 512), 2, "the eastern seat keeps its own province");
        assert_eq!(at(&p, 490), 1);
    }

    /// Protects: a province never crosses a faction border and provinces
    /// still partition every seeded faction. Faction 1 owns x < 300 plus
    /// nothing else; faction 2 owns x 300..960 and an exclave at x 100..120;
    /// faction 3 owns x >= 960 and has no settlement. Faction 1 also has a
    /// city standing on faction 2's land (x 700), which must seed nothing
    /// there.
    #[test]
    fn provinces_stay_inside_their_faction_with_no_gaps() {
        let w = Flat::new(1024, RH);
        let t = w.t();
        let cost = crate::build_travel_cost(&w.field, 1024, RH, 0.3);
        let mut territory = vec![0i32; 1024 * RH];
        for (i, o) in territory.iter_mut().enumerate() {
            let x = i % 1024;
            *o = if (100..120).contains(&x) { 2 } else if x < 300 { 1 } else if x < 960 { 2 } else { 3 };
        }
        let s = [
            capital(50, 20, 1, 15000),
            city(250, 20, 1, 15000),
            capital(500, 20, 2, 15000),
            city(800, 20, 2, 15000),
            city(700, 20, 1, 15000),
        ];
        let (p, list) = provinces_terrain(&s, &territory, &cost, &t, 1.0);
        let crossing = (0..p.len()).filter(|&i| p[i] > 0 && list[p[i] as usize - 1].faction != territory[i]).count();
        assert_eq!(crossing, 0, "cells in another faction's province");
        let gaps = (0..p.len()).filter(|&i| (territory[i] == 1 || territory[i] == 2) && p[i] == 0).count();
        assert_eq!(gaps, 0, "owned cells without a province");
        assert!((0..p.len()).filter(|&i| territory[i] == 3).all(|i| p[i] == 0), "a faction with no settlement has no province");
        for q in &list {
            let c = &s[q.capital_settlement_index].placement;
            if territory[c.y * 1024 + c.x] == q.faction {
                assert_eq!(p[c.y * 1024 + c.x], q.id, "seat {} holds its own id", q.name);
            }
        }
    }

    /// Protects `carry_provinces`: an unchanged grid keeps the grown raster
    /// cell for cell; cells painted to another faction take that faction's
    /// province; ids are the Voronoi path's; an old province two seats now
    /// claim is re-cut. A mutant that re-cuts by seat (ignores `old`) loses
    /// the snapped seam at x 525.
    #[test]
    fn carry_keeps_the_grown_shape_over_an_edit() {
        let w = river_world(530);
        let t = w.t();
        let cost = crate::build_travel_cost(&w.field, 1024, RH, 0.3);
        let mut s = vec![capital(100, 20, 1, 15000), city(900, 20, 1, 15000), capital(1000, 20, 2, 15000)];
        let mut territory = vec![1i32; 1024 * RH];
        for (i, o) in territory.iter_mut().enumerate() {
            if i % 1024 >= 980 {
                *o = 2;
            }
        }
        let (grown, list) = provinces_terrain(&s, &territory, &cost, &t, 1.0);
        let (same, l2) = carry_provinces(&grown, &s, &territory, 1024, RH);
        assert_eq!(same, grown);
        assert_eq!(format!("{l2:?}"), format!("{list:?}"));
        // Paint x 940..980 to faction 2, and put a town before the seats so
        // every settlement index shifts (the carry must not read indices).
        for (i, o) in territory.iter_mut().enumerate() {
            if (940..980).contains(&(i % 1024)) {
                *o = 2;
            }
        }
        s.insert(0, NamedSettlement { pop: 10, ..city(10, 0, 1, 10) });
        s[0].placement.kind = SettlementKind::Hamlet;
        let (p, l3) = carry_provinces(&grown, &s, &territory, 1024, RH);
        let east = l3.iter().find(|q| q.faction == 2).unwrap().id;
        assert_eq!(at(&p, 950), east, "painted cells take the new owner's province");
        let west = l3.iter().find(|q| s[q.capital_settlement_index].placement.x == 100).unwrap().id;
        assert_eq!(at(&p, 525), west, "the snapped seam survives the edit");
        // A new seat inside an old province: that old id now names two seats,
        // so its cells are re-cut by nearest seat and the new seat holds its own.
        s.push(city(300, 20, 1, 15000));
        let (p, l4) = carry_provinces(&grown, &s, &territory, 1024, RH);
        let new = l4.iter().find(|q| q.capital_settlement_index == s.len() - 1).unwrap().id;
        assert_eq!(at(&p, 300), new, "the new seat holds its own cell");
    }

    /// Protects `province_polylines`: only seams between provinces of one
    /// faction, never a faction border. Faction 1 (x < 5) holds provinces 1
    /// and 2 split at y 3; faction 2 (x >= 5) is province 3.
    #[test]
    fn province_lines_trace_inner_seams_only() {
        let (gw, gh) = (10, 6);
        let field = vec![0.5f32; gw * gh];
        let territory: Vec<i32> = (0..gw * gh).map(|i| if i % gw < 5 { 1 } else { 2 }).collect();
        let province: Vec<i32> = (0..gw * gh).map(|i| if i % gw >= 5 { 3 } else if i / gw < 3 { 1 } else { 2 }).collect();
        let lines = province_polylines(&province, &territory, &field, 0.3, gw, gh);
        assert_eq!(lines.len(), 1, "one inner seam");
        assert!(lines[0].iter().all(|&(x, y)| (y - 3.0).abs() < 1e-6 && x <= 5.0), "on y = 3, west of the faction border: {:?}", lines[0]);
    }
}
