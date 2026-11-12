//! **RR-6, the per-basin lake water balance** (`RIVER_REALISM_SCOPE.md` RR-6;
//! owner Ruling ES item 4 "Measure first", then Ruling ET, 2026-11-12: *"In
//! regards to the lakes, make it the new default on setting"*).
//!
//! Today ([`crate::build_water_bodies`]) every pooled above-sea cell deeper
//! than [`crate::LAKE_DEPTH`] and wetter than [`crate::LAKE_RAIN`] is lake, and
//! every lake stands at its spill height whatever feeds it. The balance
//! (`LAKE_FILL_RESEARCH.md` Part 1(b)) lets a closed lake's area adjust until
//! evaporation over it matches its inflow: a lake holds water only where
//! inflow covers evaporation, otherwise it is smaller or a **playa**.
//!
//! **Unit: a pool**, a 4-connected component of above-sea cells deeper than
//! `LAKE_DEPTH` under the fill level, whatever their rain (the balance
//! replaces the per-cell rain test). Per pool:
//! * inflow `I` = the highest routed discharge (`flow_discharge`, rain-weighted
//!   cell-units, mean 1 a cell) among its cells: the water the network brings
//!   to it, the rain on the pool included;
//! * evaporation per pool cell `e(T) = EVAP_AT_25 * clamp(T, 0, T_MAX) / 25`
//!   in the same cell-units;
//! * the balanced lake is the lowest `k` cells (by height) with
//!   `sum(e) <= I`; all cells means the pool spills and keeps its level; a
//!   balanced lake cell must stand `LAKE_DEPTH` under the balanced level.
//!
//! **One implementation.** [`lake_pools`] is called by [`apply_lake_balance`]
//! (generation) and by `cartalith-godot/tests/lake_balance_harness.rs` (the
//! measurement), so the harness cannot drift from what ships.
//!
//! **A playa** is the dry floor of a pool the balance emptied. It is *not* a
//! new water-body class: the classification stays 0/1/2 so every consumer of
//! lakes keeps its meaning, and the cell is land, drawn by the climate biome
//! beneath it (arid country reads as sand). Recorded as the minimal choice in
//! Ruling ET. The dried floor is marked separately in `WaterBodies::playa`
//! (built by `render::water_bodies_floored_balanced`), which the salt-flat tint
//! and the trade salt-lake source read; it never moves the classification.
//!
//! Must never run with a missing or wrong-length flow or temperature field
//! (no value is not a plausible value: the classification is then left as
//! `build_water_bodies` made it), and never touches an ocean or below-sea cell.

use crate::{LAKE_DEPTH, WaterBodies};

/// Open-water evaporation at a mean annual temperature of 25 degrees C, in the
/// discharge's cell-units (1 = the map's mean rain per cell). **Labelled
/// judgement** (RR-6 harness, Ruling ES): tropical open-water evaporation of
/// about 2 m a year against a global mean precipitation near 1 m a year
/// (textbook orders, SECONDARY), so a warm lake cell loses about twice a mean
/// cell's rain.
pub const EVAP_AT_25: f64 = 2.0;
/// The temperature above which evaporation stops rising in this proxy.
/// **Labelled judgement** (the Thornthwaite-shaped curve flattens in the
/// tropics; 35 C is the upper end of the mean annual temperatures the climate
/// stage produces).
pub const T_MAX: f64 = 35.0;
/// The reference temperature of [`EVAP_AT_25`].
const T_REF: f64 = 25.0;

/// Evaporation from one lake cell at mean temperature `t`, in cell-units.
pub fn lake_evaporation(t: f32) -> f64 {
    EVAP_AT_25 * (t as f64).clamp(0.0, T_MAX) / T_REF
}

/// One pool and its balance.
pub struct LakePool {
    /// Every cell of the pool.
    pub cells: Vec<usize>,
    /// The cells that hold water under the balance.
    pub balanced: Vec<usize>,
    /// Inflow `I`.
    pub inflow: f64,
    /// Evaporation over the whole pool (the spill-level area).
    pub e_spill: f64,
    /// Whether the whole pool is held (inflow covers evaporation at spill).
    pub spills: bool,
    /// The balanced surface height: the spill level when `spills`, else the
    /// height of the first cell left dry.
    pub level: f32,
}

fn nb4(i: usize, gw: usize, gh: usize, wrap: bool) -> impl Iterator<Item = usize> {
    let (x, y) = (i % gw, i / gw);
    let l = if x > 0 { Some(i - 1) } else if wrap { Some(i + gw - 1) } else { None };
    let r = if x + 1 < gw { Some(i + 1) } else if wrap { Some(i + 1 - gw) } else { None };
    let u = if y > 0 { Some(i - gw) } else { None };
    let d = if y + 1 < gh { Some(i + gw) } else { None };
    [l, r, u, d].into_iter().flatten()
}

/// The pools of a world and each one's balance at evaporation scale `ek`
/// (`1.0` the model, `0.0` the do-nothing control where every pool spills).
/// `fill` is [`WaterBodies::fill_level`].
#[allow(clippy::too_many_arguments)]
pub fn lake_pools(field: &[f32], flow: &[f32], temp: &[f32], fill: &[f32], gw: usize, gh: usize, sea: f64, wrap: bool, ek: f64) -> Vec<LakePool> {
    let n = gw * gh;
    let deep = |i: usize| field[i] as f64 > sea && (fill[i] - field[i]) as f64 > LAKE_DEPTH;
    let mut seen = vec![false; n];
    let mut out = Vec::new();
    for s in 0..n {
        if seen[s] || !deep(s) {
            continue;
        }
        let mut cells = vec![s];
        seen[s] = true;
        let mut k = 0;
        while k < cells.len() {
            let i = cells[k];
            k += 1;
            for j in nb4(i, gw, gh, wrap) {
                if !seen[j] && deep(j) {
                    seen[j] = true;
                    cells.push(j);
                }
            }
        }
        let inflow = cells.iter().map(|&i| flow[i] as f64).filter(|q| q.is_finite()).fold(0.0, f64::max);
        let mut by_h = cells.clone();
        by_h.sort_by(|&a, &b| field[a].total_cmp(&field[b]));
        let e_spill: f64 = cells.iter().map(|&i| ek * lake_evaporation(temp[i])).sum();
        let mut acc = 0.0;
        let mut keep = 0;
        for &i in &by_h {
            let e = ek * lake_evaporation(temp[i]);
            if acc + e > inflow {
                break;
            }
            acc += e;
            keep += 1;
        }
        let spills = keep == by_h.len();
        let level = if spills { cells.iter().map(|&i| fill[i]).fold(f32::MIN, f32::max) } else { field[by_h[keep]] };
        let balanced: Vec<usize> = by_h[..keep].iter().copied().filter(|&i| (level - field[i]) as f64 > LAKE_DEPTH).collect();
        out.push(LakePool { cells, balanced, inflow, e_spill, spills, level });
    }
    out
}

/// What [`apply_lake_balance`] changed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LakeBalanceReport {
    /// Pools examined.
    pub pools: usize,
    /// Cells that were lake and are now land (playa floor).
    pub cells_dried: usize,
    /// Cells that were land and are now lake (a river feeds a pool the rain
    /// gate left dry).
    pub cells_flooded: usize,
}

/// Applies the balance to a freshly built [`WaterBodies`] (before Ruling CC's
/// floor and the forced lakes, so those see the balanced water): every pool
/// cell is lake exactly when the balance holds it, and a pool that does not
/// spill gets its `fill_level` lowered to the balanced surface so the drawn
/// shoreline (`fill_level - LAKE_DEPTH`) contours the same water this
/// classification holds. `flow` and `temp` must be `gw * gh` long or nothing
/// changes and the default report returns.
#[allow(clippy::too_many_arguments)]
pub fn apply_lake_balance(wb: &mut WaterBodies, field: &[f32], flow: &[f32], temp: &[f32], gw: usize, gh: usize, sea: f64, world: bool) -> LakeBalanceReport {
    let n = gw * gh;
    let mut rep = LakeBalanceReport::default();
    if n == 0 || field.len() < n || flow.len() != n || temp.len() != n || wb.classification.len() != n || wb.fill_level.len() != n {
        return rep;
    }
    let pools = lake_pools(field, flow, temp, &wb.fill_level, gw, gh, sea, world, 1.0);
    rep.pools = pools.len();
    for q in &pools {
        let keep: std::collections::HashSet<usize> = q.balanced.iter().copied().collect();
        for &i in &q.cells {
            let was = wb.classification[i] == 2;
            let now = keep.contains(&i);
            wb.classification[i] = if now { 2 } else { 0 };
            if was && !now {
                rep.cells_dried += 1;
            } else if !was && now {
                rep.cells_flooded += 1;
            }
            if !q.spills {
                wb.fill_level[i] = wb.fill_level[i].min(q.level);
            }
        }
    }
    rep
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_water_bodies;

    const GW: usize = 9;
    const GH: usize = 5;
    const SEA: f64 = 0.1;

    /// A 9x5 plateau at 0.6 with a three-cell bowl (heights 0.50, 0.52, 0.54)
    /// in the middle row, so the flood fills it to the rim and every bowl cell
    /// is deeper than `LAKE_DEPTH`. Returns the field and the bowl's indices.
    fn bowl() -> (Vec<f32>, [usize; 3]) {
        let mut f = vec![0.6f32; GW * GH];
        let at = |x: usize| 2 * GW + x;
        f[at(3)] = 0.50;
        f[at(4)] = 0.52;
        f[at(5)] = 0.54;
        (f, [at(3), at(4), at(5)])
    }

    fn run(inflow: f32, temp: f32, rain: f32) -> (WaterBodies, LakeBalanceReport, [usize; 3]) {
        let (field, cells) = bowl();
        let rainfall = vec![rain; GW * GH];
        let mut wb = build_water_bodies(&field, GW, GH, SEA, false, Some(&rainfall));
        let mut flow = vec![0.0f32; GW * GH];
        flow[cells[0]] = inflow;
        let rep = apply_lake_balance(&mut wb, &field, &flow, &vec![temp; GW * GH], GW, GH, SEA, false);
        (wb, rep, cells)
    }

    fn lakes(wb: &WaterBodies, cells: [usize; 3]) -> [bool; 3] {
        [wb.classification[cells[0]] == 2, wb.classification[cells[1]] == 2, wb.classification[cells[2]] == 2]
    }

    /// Protects the evaporation proxy's three literal anchors: 2.0 at 25 C,
    /// flat above 35 C, zero below freezing (the constants' labelled
    /// judgements; a mutant on any one fails here).
    #[test]
    fn evaporation_curve_anchors() {
        assert_eq!(lake_evaporation(25.0), 2.0);
        assert!((lake_evaporation(35.0) - 2.8).abs() < 1e-12);
        assert!((lake_evaporation(50.0) - 2.8).abs() < 1e-12);
        assert_eq!(lake_evaporation(-5.0), 0.0);
    }

    /// Protects the spill case: inflow 10 covers 3 cells x 2.0, so the whole
    /// bowl stays lake and its fill level is untouched.
    #[test]
    fn inflow_covering_evaporation_keeps_the_whole_lake() {
        let (wb, rep, cells) = run(10.0, 25.0, 1.0);
        assert_eq!(lakes(&wb, cells), [true, true, true]);
        assert_eq!(rep, LakeBalanceReport { pools: 1, cells_dried: 0, cells_flooded: 0 });
        assert!(wb.fill_level[cells[2]] > 0.59, "a spilling pool keeps its spill level");
    }

    /// Protects the shrink case: inflow 5 pays for two cells (4.0), the
    /// shallowest-bed third dries, and the pool's fill level drops to the
    /// first dry cell's height so the drawn shore agrees (0.54).
    #[test]
    fn partial_inflow_shrinks_the_lake_and_lowers_its_level() {
        let (wb, rep, cells) = run(5.0, 25.0, 1.0);
        assert_eq!(lakes(&wb, cells), [true, true, false]);
        assert_eq!(rep.cells_dried, 1);
        assert_eq!(wb.fill_level[cells[0]], 0.54);
        assert_eq!(wb.fill_level[cells[1]], 0.54);
    }

    /// Protects the tie: inflow exactly equal to two cells' evaporation (4.0)
    /// holds two cells (`acc + e > inflow` is strict). A `>=` would hold one.
    #[test]
    fn inflow_equal_to_evaporation_holds_the_cell() {
        let (wb, _, cells) = run(4.0, 25.0, 1.0);
        assert_eq!(lakes(&wb, cells), [true, true, false]);
    }

    /// Protects the playa: inflow 1 cannot pay for even one cell (2.0), so the
    /// whole bowl is dry land and counted as dried.
    #[test]
    fn starved_basin_is_a_playa() {
        let (wb, rep, cells) = run(1.0, 25.0, 1.0);
        assert_eq!(lakes(&wb, cells), [false, false, false]);
        assert_eq!(rep.cells_dried, 3);
        assert!(wb.classification.iter().all(|&c| c != 2));
    }

    /// Protects the replacement of the rain gate: a bowl the per-cell rain test
    /// leaves dry (rain 0.0 under `LAKE_RAIN`) is a lake when a river feeds it.
    #[test]
    fn river_fed_basin_in_dry_country_is_a_lake() {
        let (field, cells) = bowl();
        let dry = vec![0.0f32; GW * GH];
        let before = build_water_bodies(&field, GW, GH, SEA, false, Some(&dry));
        assert_eq!(lakes(&before, cells), [false, false, false], "premise: the rain test leaves it dry");
        let (wb, rep, cells) = run(10.0, 25.0, 0.0);
        assert_eq!(lakes(&wb, cells), [true, true, true]);
        assert_eq!(rep.cells_flooded, 3);
    }

    /// Protects the do-nothing control: at 0 C nothing evaporates, so even a
    /// pool with no inflow spills.
    #[test]
    fn no_evaporation_means_every_pool_spills() {
        let (wb, _, cells) = run(0.0, 0.0, 1.0);
        assert_eq!(lakes(&wb, cells), [true, true, true]);
    }

    /// Protects "no value is not a plausible value": a flow field of the
    /// wrong length changes nothing and reports nothing.
    #[test]
    fn missing_fields_leave_the_classification_alone() {
        let (field, _) = bowl();
        let rain = vec![1.0f32; GW * GH];
        let mut wb = build_water_bodies(&field, GW, GH, SEA, false, Some(&rain));
        let before = wb.classification.clone();
        let rep = apply_lake_balance(&mut wb, &field, &[], &vec![25.0; GW * GH], GW, GH, SEA, false);
        assert_eq!(rep, LakeBalanceReport::default());
        assert_eq!(wb.classification, before);
    }
}
