//! **A lake the water balance made has a smooth shore** (Ruling ET's balance, RV-4's shoreline; verifier finding,
//! 2026-11-13: the large lake Ruling EV leaves in the owner's 788 % view of seed 51203 was drawn cell-stepped --
//! straight grid-aligned shore runs, rectangular inlets, square one-cell islands).
//!
//! The cause, protected here: the shore field (`render::shore_margins`) takes the smaller of a depth margin and a
//! RAIN margin, because under the reference's rule a pooled cell is lake only where its rain reaches `LAKE_RAIN`.
//! The balance replaces that per-cell test for every pool (`cartalith_civ::lake_balance`: the lowest cells of a pool
//! up to a level, whatever their rain), so a balanced lake in dry country had a negative rain margin on every cell;
//! the class clamp lifted each to `SHORE_EPS` against land a whole margin below, and the contour hugged the water
//! cell centres. `shore_margins_with(.., rain_gated = false, ..)` drops the rain margin for such a world, and
//! `lakes_rain_gated` is the one definition of which worlds those are.
//!
//! The fixture is a dry (rain 0.1) bowl sloping 0.01 a column, so the expected numbers are worked by hand from its
//! geometry, never read back from the code under test: the balance keeps the two lowest columns (inflow 33 pays for
//! 16 cells at 2.0 each) and lowers the surface to the first dry column's 0.52; the depth margin at the edge lake cell
//! is 0.52 - `LAKE_DEPTH` - 0.51 = 0.006 and at the first dry cell -0.004, so the drawn shore crosses 0.6 of the way
//! between them -- exactly where the ground (0.50 + 0.01 a column) meets the water surface 0.516.
//!
//! `#[path]`-includes `render.rs` for the reason `base_lake_stamp.rs` gives: `cartalith-godot` is `cdylib`-only.
#![allow(dead_code)]

#[path = "../src/render.rs"]
mod render;

use render::LakeBalanceInputs;

/// Fixture width: a judgement -- room for an 8 x 8 bowl inside a plateau rim.
const GW: usize = 24;
/// Fixture height; see [`GW`].
const GH: usize = 24;
/// The project's default sea level; the bowl and plateau sit above it.
const SEA: f64 = 0.42;
/// Rain everywhere: under `LAKE_RAIN` (0.22), so the reference's rule makes no lake here at all.
const DRY_RAIN: f32 = 0.1;
/// Discharge in every cell: after the runoff share (`RUNOFF_SHARE` 0.58, so 32.48 arrives) it pays for 16 lake cells
/// at 2.0 each (25 C, 32.0) and not for 17 (34.0). Was 33.0 before the share (owner choice 2026-11-13): restated so
/// the fixture still holds 16 cells, the expectation below unchanged and not weakened.
const INFLOW: f32 = 56.0;
/// The middle row of the bowl.
const ROW: usize = 12;

/// A 0.70 plateau with a closed bowl in rows/columns 8..16 whose floor rises 0.01 a column from 0.50 at column 8.
fn bowl() -> Vec<f32> {
    let mut f = vec![0.70f32; GW * GH];
    for y in 8..16 {
        for x in 8..16 {
            f[y * GW + x] = 0.50 + 0.01 * (x - 8) as f32;
        }
    }
    f
}

/// The bowl's water under the balance (`Some`) or the reference's rain rule (`None`), and the balance inputs used.
fn water(balanced: bool) -> (cartalith_civ::WaterBodies, Vec<f32>, Vec<f32>) {
    let field = bowl();
    let rain = vec![DRY_RAIN; GW * GH];
    let flow = vec![INFLOW; GW * GH];
    let temp = vec![25.0f32; GW * GH];
    let wb = render::water_bodies_floored_balanced(&field, &rain, GW, GH, SEA, false, None, None, LakeBalanceInputs::when(balanced, Some(&flow), &temp));
    (wb, flow, temp)
}

/// Protects `lakes_rain_gated` as the one definition: a world the balance classified (inputs present) is not
/// rain-gated, every other one is.
#[test]
fn only_a_balanced_world_drops_the_rain_gate() {
    let (flow, temp) = (vec![1.0f32; 4], vec![25.0f32; 4]);
    assert!(render::lakes_rain_gated(None));
    assert!(render::lakes_rain_gated(LakeBalanceInputs::when(false, Some(&flow), &temp)));
    assert!(!render::lakes_rain_gated(LakeBalanceInputs::when(true, Some(&flow), &temp)));
    assert!(render::lakes_rain_gated(LakeBalanceInputs::when(true, None, &temp)), "no flow field: the balance cannot run, the rain test stands");
}

/// Protects the fixture's premises, worked by hand: the rain rule leaves the dry bowl without a lake, and the
/// balance keeps exactly columns 8 and 9 (16 cells) at a surface lowered to 0.52.
#[test]
fn the_balance_makes_a_two_column_lake_in_the_dry_bowl() {
    let (old, _, _) = water(false);
    assert!(old.classification.iter().all(|&c| c != 2), "premise: rain 0.1 is under LAKE_RAIN, so the reference's rule makes no lake");
    let (wb, _, _) = water(true);
    let lake: Vec<usize> = (0..GW * GH).filter(|&i| wb.classification[i] == 2).collect();
    assert_eq!(lake.len(), 16, "inflow 56 at the 0.58 runoff share (32.48) pays for 16 cells at 2.0 each");
    assert!(lake.iter().all(|&i| (8..16).contains(&(i / GW)) && (8..10).contains(&(i % GW))), "the two lowest columns: {lake:?}");
    assert_eq!(wb.fill_level[ROW * GW + 9], 0.52f32, "the surface is lowered to the first dry column's ground");
}

/// Protects the fix and its positive control: on the balanced lake's edge cell the rain-gated field is the clamp's
/// `SHORE_EPS` (the stepped shore), and the balanced world's field is the depth margin, 0.006; on the first dry cell
/// it is -0.004 against the gated -0.12; so the drawn shore crosses 0.6 of the way across, where the ground meets the
/// water (0.516 at column 9.6), instead of hugging the lake cell's centre.
#[test]
fn a_balanced_lake_in_dry_country_is_contoured_by_its_depth() {
    let field = bowl();
    let rain = vec![DRY_RAIN; GW * GH];
    let (wb, flow, temp) = water(true);
    let gated = render::lakes_rain_gated(LakeBalanceInputs::when(true, Some(&flow), &temp));
    assert!(!gated, "premise: this world's lakes are the balance's");
    let new = render::shore_field_forced_with(&field, &wb.classification, &wb.fill_level, &rain, gated, None, GW, GH, SEA, false);
    let old = render::shore_field_forced(&field, &wb.classification, &wb.fill_level, &rain, None, GW, GH, SEA, false);
    let (lake, dry) = (ROW * GW + 9, ROW * GW + 10);
    // The old field: the stepped shore (positive control).
    assert_eq!(old[lake] as f64, render::SHORE_EPS as f32 as f64, "the rain-gated field lifts the dry lake cell to SHORE_EPS");
    assert!((old[dry] as f64 + 0.12).abs() < 1e-6, "and its dry neighbour sits at the rain margin 0.1 - 0.22: {}", old[dry]);
    let t_old = old[lake] as f64 / (old[lake] as f64 - old[dry] as f64);
    assert!(t_old < 1e-3, "the old contour hugs the lake cell centre: {t_old}");
    // The fixed field: the depth margin on both sides.
    assert!((new[lake] as f64 - 0.006).abs() < 1e-6, "lake edge: 0.52 - 0.004 - 0.51 = 0.006, got {}", new[lake]);
    assert!((new[dry] as f64 + 0.004).abs() < 1e-6, "first dry cell: 0.516 - 0.52 = -0.004, got {}", new[dry]);
    let t = new[lake] as f64 / (new[lake] as f64 - new[dry] as f64);
    assert!((t - 0.6).abs() < 1e-3, "the shore crosses 0.6 of the way, at column 9.6: {t}");
    // The deepest cell keeps its full depth, and the plateau far from water stays land.
    assert!((new[ROW * GW + 8] as f64 - 0.016).abs() < 1e-6, "deepest column: 0.516 - 0.50, got {}", new[ROW * GW + 8]);
    assert!(new[2 * GW + 2] < 0.0, "the plateau corner is land");
}

/// Protects the boundary of the fix: a rain-test world (no balance) gets exactly the field it always got, so the
/// reference's rain-gated edges (seed 483920's lake at cell 671,97) still draw where the rain rule puts them.
#[test]
fn a_rain_test_world_keeps_its_shore_field_bit_for_bit() {
    let field = bowl();
    // Wet enough for the rain rule to make the bowl a lake, and a wet/dry split through it so the rain margin binds.
    let mut rain = vec![0.5f32; GW * GH];
    for y in 0..GH {
        for x in 12..GW {
            rain[y * GW + x] = 0.1;
        }
    }
    let wb = render::water_bodies_floored_balanced(&field, &rain, GW, GH, SEA, false, None, None, None);
    assert!(wb.classification.iter().any(|&c| c == 2), "premise: the wet half pools a lake");
    let old = render::shore_field_forced(&field, &wb.classification, &wb.fill_level, &rain, None, GW, GH, SEA, false);
    let with = render::shore_field_forced_with(&field, &wb.classification, &wb.fill_level, &rain, render::lakes_rain_gated(None), None, GW, GH, SEA, false);
    assert_eq!(old, with, "a rain-test world's field is unchanged");
    let ungated = render::shore_field_forced_with(&field, &wb.classification, &wb.fill_level, &rain, false, None, GW, GH, SEA, false);
    assert_ne!(old, ungated, "positive control: dropping the gate here WOULD move the rain-cut edge");
}
