//! Golden-parity tests for the Cartography paint brush —
//! `UNIFIED_TOOL_PLAN.md` milestone C, Biome paint.
//!
//! ## The harness
//!
//! Node `vm.runInContext`, transient and not checked in. The reference's
//! paint block is genuinely small and self-contained, so one slice is the
//! whole dependency set:
//!
//! | lines | what |
//! |---|---|
//! | 4758-4795 | `_paintMode`..`_paintAt` — `getPaintLayer`, `_paintSampleAt`, `_paintAt` |
//!
//! Block-comment balance plus start/end top-level-boundary assertions, same
//! as every slice in this workspace. The bound is genuinely tight here:
//! `4754-4757` is the block comment introducing the layers and `4796` opens
//! the next one, so a slice one line wide in either direction splices a
//! comment.
//!
//! Two stubs, disclosed: `currentWaterBodies()` returns a harness-supplied
//! fixture array (the real one needs a generated world), and `render()` is a
//! no-op (DOM). Neither affects what `_paintAt` writes — it reads the
//! classification only as `wb[i] !== 0` and calls `render()` after the last
//! write.
//!
//! One thing the harness got wrong on the first run, worth recording because
//! it would have produced *silently empty* results rather than an error: the
//! reference declares `paintBiome`/`_paintLayer`/`_paintValue`/`_paintRadius`
//! with `let`, which in a `vm` script are **lexical bindings, not properties
//! of the context object**. Setting `ctx._paintRadius` from the host creates
//! a shadow the reference code never reads. Everything therefore drives
//! `_paintAt` from inside the context.
//!
//! ## Fixtures
//!
//! 64x64, matching the sibling sculpt suites. Three water-body
//! classifications:
//!
//! * **none** — all land, isolating the disc geometry.
//! * **sea** — `wb[i] = 1` where `field[i] < 0.5` on milestone B's own base
//!   field (`((i*37) % 101)/200 + 0.2`, built in `f64`, rounded once at the
//!   `f32` store).
//! * **lakeband** — rows 30..=34 set to `2`, deliberately **not** `1`. The
//!   reference's gate is `wb[i] !== 0`, and its own comment insists this
//!   *"excludes BOTH ocean(1) and lake(2), never a bare `field[i] < sea`
//!   check, which misses above-sea-level lakes."* A port that gated on `== 1`
//!   would pass every ocean case and silently paint over lakes.
//!
//! Each case is checked exactly: the painted-cell count, an order-sensitive
//! fold over every painted `(index, value)` pair, the six sampled cells, and
//! an assertion that **no** painted cell sits on a nonzero classification.
//! No tolerance — this is integer data.

use std::sync::Arc;

use cartalith_spatial::{PaintStamp, Stamp};

const GW: usize = 64;
const GH: usize = 64;
const SEA: f32 = 0.5;

const SAMPLES: [usize; 6] = [
    32 * GW + 32,
    32 * GW + 12,
    26 * GW + 32,
    38 * GW + 32,
    20 * GW + 20,
    44 * GW + 50,
];

fn base_field() -> Vec<f32> {
    (0..GW * GH)
        .map(|i| ((((i * 37) % 101) as f64) / 200.0 + 0.2) as f32)
        .collect()
}

fn wb_none() -> Arc<[u8]> {
    vec![0u8; GW * GH].into()
}

fn wb_sea() -> Arc<[u8]> {
    let f = base_field();
    (0..GW * GH)
        .map(|i| u8::from(f[i] < SEA))
        .collect::<Vec<u8>>()
        .into()
}

/// Rows 30..=34 classified `2` (lake), not `1`.
fn wb_lake_band() -> Arc<[u8]> {
    let mut m = vec![0u8; GW * GH];
    for y in 30..=34 {
        for x in 0..GW {
            m[y * GW + x] = 2;
        }
    }
    m.into()
}

/// The harness's own fold: `sum = (sum*31 + i*7 + v) >>> 0` over painted
/// cells in ascending index order. JS's `>>> 0` is ToUint32, i.e. mod 2^32,
/// which is `u32` wrapping arithmetic.
fn fold(cells: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    for (i, &v) in cells.iter().enumerate() {
        if v != 0 {
            sum = sum
                .wrapping_mul(31)
                .wrapping_add((i * 7) as u32)
                .wrapping_add(v as u32);
        }
    }
    sum
}

struct Golden {
    name: &'static str,
    painted: usize,
    fold: u32,
    samples: [u8; 6],
}

/// `taps` are `(gx, gy, value)`; value `0` is the reference's `_paintErase`.
#[track_caller]
fn check(g: &Golden, wb: Arc<[u8]>, radius: f64, taps: &[(i64, i64, u8)]) {
    let mut cells = vec![0u8; GW * GH];
    for &(gx, gy, v) in taps {
        PaintStamp::new(gx, gy, radius, v, wb.clone()).apply(&mut cells, GW, GH);
    }

    let painted = cells.iter().filter(|&&v| v != 0).count();
    assert_eq!(painted, g.painted, "{}: painted-cell count", g.name);
    assert_eq!(fold(&cells), g.fold, "{}: painted (index, value) fold", g.name);
    for (n, &i) in SAMPLES.iter().enumerate() {
        assert_eq!(cells[i], g.samples[n], "{}: sample {n} at cell {i}", g.name);
    }
    // The land-only gate, checked independently of the fold: no painted cell
    // may sit on a nonzero classification, ocean or lake.
    for i in 0..GW * GH {
        assert!(
            cells[i] == 0 || wb[i] == 0,
            "{}: painted over water at cell {i} (wb = {})",
            g.name,
            wb[i]
        );
    }
}

#[test]
fn a_disc_on_open_land_matches_the_reference() {
    // Radius 6 -> 113 cells, the integer-lattice disc of pi*r^2 ~= 113.1.
    check(
        &Golden {
            name: "paint_no_water",
            painted: 113,
            fold: 1_962_874_917,
            samples: [5, 0, 5, 5, 0, 0],
        },
        wb_none(),
        6.0,
        &[(32, 32, 5)],
    );
}

#[test]
fn the_ocean_gate_matches_the_reference() {
    check(
        &Golden {
            name: "paint_sea_gated",
            painted: 46,
            fold: 3_396_237_546,
            samples: [5, 0, 0, 5, 0, 0],
        },
        wb_sea(),
        6.0,
        &[(32, 32, 5)],
    );
}

#[test]
fn the_lake_gate_matches_the_reference() {
    // Classification 2, not 1: a `== 1` gate would paint straight through
    // this band and still pass every other case in this file.
    check(
        &Golden {
            name: "paint_lake_gated",
            painted: 120,
            fold: 848_244_480,
            samples: [0, 0, 7, 7, 0, 0],
        },
        wb_lake_band(),
        8.0,
        &[(32, 32, 7)],
    );
}

#[test]
fn erasing_over_an_existing_disc_matches_the_reference() {
    check(
        &Golden {
            name: "paint_erase_over",
            painted: 24,
            fold: 2_144_451_283,
            samples: [0, 0, 3, 3, 0, 0],
        },
        wb_none(),
        6.0,
        &[(32, 32, 3), (34, 32, 0)],
    );
}

#[test]
fn a_disc_clipped_by_the_grid_edge_matches_the_reference() {
    check(
        &Golden {
            name: "paint_edge_clamped",
            painted: 48,
            fold: 2_640_520_818,
            samples: [0, 0, 0, 0, 0, 0],
        },
        wb_none(),
        6.0,
        &[(1, 1, 2)],
    );
}

#[test]
fn overlapping_discs_let_the_last_one_win() {
    check(
        &Golden {
            name: "paint_overlap_last_wins",
            painted: 119,
            fold: 709_132_905,
            samples: [9, 0, 0, 0, 0, 0],
        },
        wb_none(),
        5.0,
        &[(30, 32, 4), (34, 32, 9)],
    );
}

#[test]
fn radius_one_matches_the_reference() {
    // Five cells, not nine: `hypot(1,1)` is 1.41, and the gate is `> R`.
    check(
        &Golden {
            name: "paint_radius_one",
            painted: 5,
            fold: 601_559_462,
            samples: [6, 0, 0, 0, 0, 0],
        },
        wb_none(),
        1.0,
        &[(32, 32, 6)],
    );
}
