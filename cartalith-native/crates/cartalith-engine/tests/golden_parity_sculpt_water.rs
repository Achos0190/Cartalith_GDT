//! Golden-parity tests for `sculptCommit`'s water hooks —
//! `UNIFIED_TOOL_PLAN.md` milestone C, River/water.
//!
//! ## The harness
//!
//! Node `vm.runInContext`, transient and not checked in, the same technique
//! milestone B used. Six contiguous line slices of the real reference:
//!
//! | lines | what |
//! |---|---|
//! | 2292-2293 | `hash`, `vnoise` |
//! | 7568-7569 | `clamp01`, `smoothstep` |
//! | 8304 | `lerp` |
//! | **8725-8745** | `enforceChannelDescent` + `enforceRiverChannels` |
//! | 8821-9081 | the Sculpt pure core (`sculptApplyStamp` and the registry) |
//! | **4758-4795** | the paint-brush block (`getPaintLayer`, `_paintSampleAt`, `_paintAt`) — used by the sibling `cartalith-spatial` paint tests |
//!
//! Every slice carries a **block-comment balance assertion** plus start- and
//! end-of-slice top-level-boundary checks. It earned its keep again: the
//! first run threw on `hash/vnoise` because the end-boundary check was too
//! strict for a one-line function, which is exactly the class of silent
//! mis-slice the assertion exists to surface. The `8725-8745` bound is the
//! delicate one — `enforceChannelDescent` is *preceded* by a four-line
//! `/* ... */` and `enforceRiverChannels` is *followed* by one, so both
//! edges sit against comment boundaries.
//!
//! ## What is transcribed rather than sliced, and why
//!
//! `sculptCommit` itself (line 9318) cannot be called: it opens with
//! `_sculptEditorActive()` and closes with `computeFlow`/`refreshClimate`/
//! `renderNow`/`sculptSyncUI` — DOM and whole-pipeline recompute. Its
//! **water-hook body, lines 9320-9346, is copied verbatim** into the harness
//! with `sculptStamps` as a parameter and those six calls dropped. That is a
//! dozen lines of plain loop; disclosed here rather than implied, because a
//! transcription is weaker evidence than a slice.
//!
//! ## Fixtures
//!
//! Milestone B's fixtures exactly, so the two suites are directly
//! comparable: 64x64, `field[i] = ((i*37) % 101)/200 + 0.2` built in `f64`
//! and rounded once at the `f32` store, `seaLevel = 0.5`, `seed = 1234`,
//! `brushSize = 12`, `SCULPT_GLOBAL_DEF` otherwise.
//!
//! One fixture milestone B did not need: a **dense** stroke (23 points, 2
//! cells apart) beside the 2-point one. That is not cosmetic —
//! `enforceChannelDescent` walks the stroke's *own* points and never
//! resamples, so the 2-point stroke locks 3 cells while the dense one locks
//! 46. Testing only the coarse stroke would have exercised the lock barely
//! at all.
//!
//! No tolerance is used. Heights are compared as raw `f32` bit patterns,
//! folded FNV-1a-64 over the whole field, so a one-ULP difference in any of
//! 4096 cells fails.

use cartalith_engine::sculpt_commit::{commit_sculpt_pass, WaterState};
use cartalith_spatial::{DirtyTracker, PassBuffer};
use cartalith_terrain::sculpt::{Feature, Point, SculptGlobals, SculptStamp};

const GW: usize = 64;
const GH: usize = 64;
const SEA: f64 = 0.5;
const SEED: u32 = 1234;

/// The same six cells milestone B's harness sampled.
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

fn fnv(field: &[f32]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in field {
        h ^= v.to_bits() as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

fn stamp(feature: Feature, points: Vec<Point>) -> SculptStamp {
    let mut s = SculptStamp::new(feature, SEED, points, SEA);
    s.globals = SculptGlobals {
        brush_size: 12.0,
        ..SculptGlobals::default()
    };
    s
}

fn stroke() -> Vec<Point> {
    vec![Point::new(10.0, 32.0), Point::new(54.0, 32.0)]
}

/// 23 points, 2 cells apart — see the module docs on why this exists.
fn stroke_dense() -> Vec<Point> {
    (0..=22)
        .map(|k| Point::new(10.0 + k as f64 * 2.0, 32.0))
        .collect()
}

fn tap() -> Vec<Point> {
    vec![Point::new(32.0, 32.0)]
}

struct Golden {
    name: &'static str,
    changed: usize,
    hash: &'static str,
    river_any: bool,
    river_mask_count: usize,
    /// `None` when the reference left `lakeMask` unallocated.
    lake_mask_count: Option<usize>,
    samples: [u32; 6],
    river_floor_samples: [u32; 6],
    river_mask_samples: [u8; 6],
    lake_mask_samples: Option<[u8; 6]>,
}

/// A commit against a pristine field, optionally with pre-existing locks.
#[track_caller]
fn check(g: &Golden, stamps: Vec<(SculptStamp, bool)>, pre: Option<&dyn Fn(&mut WaterState)>) {
    let mut buf: PassBuffer<SculptStamp> = PassBuffer::new(GW, GH, 16);
    for (s, hidden) in stamps {
        let i = buf.push(s);
        if hidden {
            buf.set_hidden(i, true);
        }
    }
    let base = base_field();
    let mut field = base.clone();
    let mut water = WaterState::new(GW * GH);
    if let Some(f) = pre {
        f(&mut water);
    }
    let mut tracker = DirtyTracker::new(buf.tile_count());
    commit_sculpt_pass(&mut buf, &mut field, &mut water, &mut tracker, "sculpt", SEA);

    let changed = (0..GW * GH).filter(|&i| field[i] != base[i]).count();
    assert_eq!(changed, g.changed, "{}: changed-cell count", g.name);
    assert_eq!(water.river_any, g.river_any, "{}: river_any", g.name);

    let rm = water.river_mask.iter().filter(|&&v| v != 0).count();
    assert_eq!(rm, g.river_mask_count, "{}: river_mask count", g.name);

    let lm = water
        .lake_mask
        .as_ref()
        .map(|m| m.iter().filter(|&&v| v != 0).count());
    assert_eq!(lm, g.lake_mask_count, "{}: lake_mask count", g.name);

    for (n, &i) in SAMPLES.iter().enumerate() {
        assert_eq!(
            field[i].to_bits(),
            g.samples[n],
            "{}: height sample {n} at cell {i} ({} vs {})",
            g.name,
            field[i],
            f32::from_bits(g.samples[n])
        );
        assert_eq!(
            water.river_floor[i].to_bits(),
            g.river_floor_samples[n],
            "{}: river_floor sample {n} at cell {i}",
            g.name
        );
        assert_eq!(
            water.river_mask[i], g.river_mask_samples[n],
            "{}: river_mask sample {n} at cell {i}",
            g.name
        );
        if let Some(exp) = g.lake_mask_samples {
            assert_eq!(
                water.lake_mask.as_ref().unwrap()[i],
                exp[n],
                "{}: lake_mask sample {n} at cell {i}",
                g.name
            );
        }
    }
    assert_eq!(fnv(&field), g.hash, "{}: whole-field hash", g.name);
}

const ZERO6: [u32; 6] = [0; 6];
const ZEROM: [u8; 6] = [0; 6];

// ---------------------------------------------------------------------------
// River
// ---------------------------------------------------------------------------

#[test]
fn river_only_matches_the_reference() {
    check(
        &Golden {
            name: "river_only",
            changed: 1364,
            hash: "64c26d26e208a96c",
            river_any: true,
            // Only 3 -- the 2-point stroke carves at exactly two sites and
            // the stamp's own bed is already below the parabolic target
            // almost everywhere. Real reference behaviour, see the dense case.
            river_mask_count: 3,
            lake_mask_count: None,
            samples: [
                1058726216, 1054951341, 1050823501, 1055427976, 1050924810, 1059732794,
            ],
            river_floor_samples: ZERO6,
            river_mask_samples: ZEROM,
            lake_mask_samples: None,
        },
        vec![(stamp(Feature::River, stroke()), false)],
        None,
    );
}

#[test]
fn river_with_a_dense_stroke_matches_the_reference() {
    check(
        &Golden {
            name: "river_dense_stroke",
            changed: 1364,
            hash: "e284e542e1de9a13",
            river_any: true,
            river_mask_count: 46,
            lake_mask_count: None,
            samples: [
                1054951342, 1054951341, 1050823501, 1055427976, 1050924810, 1059732794,
            ],
            river_floor_samples: [1054951342, 0, 0, 0, 0, 0],
            river_mask_samples: [1, 0, 0, 0, 0, 0],
            lake_mask_samples: None,
        },
        vec![(stamp(Feature::River, stroke_dense()), false)],
        None,
    );
}

#[test]
fn two_rivers_in_one_pass_match_the_reference() {
    // The second stamp's descent runs over the first's already-carved field
    // -- stack order carries through the water hooks too.
    check(
        &Golden {
            name: "two_rivers",
            changed: 2247,
            hash: "ebb59c6989550ebf",
            river_any: true,
            river_mask_count: 9,
            lake_mask_count: None,
            samples: [
                1057216267, 1054951341, 1047031204, 1052408077, 1050914948, 1059732794,
            ],
            river_floor_samples: ZERO6,
            river_mask_samples: ZEROM,
            lake_mask_samples: None,
        },
        vec![
            (stamp(Feature::River, stroke()), false),
            (
                stamp(
                    Feature::River,
                    vec![Point::new(32.0, 8.0), Point::new(32.0, 56.0)],
                ),
                false,
            ),
        ],
        None,
    );
}

#[test]
fn mountains_then_river_matches_the_reference() {
    check(
        &Golden {
            name: "mountains_then_river",
            changed: 1471,
            hash: "9e02d0b323215eed",
            river_any: true,
            river_mask_count: 4,
            lake_mask_count: None,
            samples: [
                1058738505, 1059135707, 1053079581, 1059296398, 1050924810, 1059732794,
            ],
            river_floor_samples: ZERO6,
            river_mask_samples: ZEROM,
            lake_mask_samples: None,
        },
        vec![
            (stamp(Feature::Mountains, stroke()), false),
            (stamp(Feature::River, stroke()), false),
        ],
        None,
    );
}

#[test]
fn mountains_then_dense_river_matches_the_reference() {
    check(
        &Golden {
            name: "mountains_then_dense_river",
            changed: 1471,
            hash: "1b509544c6413163",
            river_any: true,
            river_mask_count: 68,
            lake_mask_count: None,
            samples: [
                1054951342, 1059135707, 1053079581, 1059296398, 1050924810, 1059732794,
            ],
            river_floor_samples: [1054951342, 0, 0, 0, 0, 0],
            river_mask_samples: [1, 0, 0, 0, 0, 0],
            lake_mask_samples: None,
        },
        vec![
            (stamp(Feature::Mountains, stroke_dense()), false),
            (stamp(Feature::River, stroke_dense()), false),
        ],
        None,
    );
}

#[test]
fn a_hidden_river_leaves_the_water_hooks_inert() {
    // The hash here is milestone B's own `mountains` golden verbatim
    // (`golden_parity_sculpt.rs`), which is the cross-check: a hidden river
    // must make this commit indistinguishable from a plain Mountains bake.
    check(
        &Golden {
            name: "hidden_river_is_skipped",
            changed: 1418,
            hash: "c6884a23f7c5e73b",
            river_any: false,
            river_mask_count: 0,
            lake_mask_count: None,
            samples: [
                1060248454, 1060645656, 1054355295, 1060568030, 1050924810, 1059732849,
            ],
            river_floor_samples: ZERO6,
            river_mask_samples: ZEROM,
            lake_mask_samples: None,
        },
        vec![
            (stamp(Feature::Mountains, stroke()), false),
            (stamp(Feature::River, stroke()), true),
        ],
        None,
    );
}

// ---------------------------------------------------------------------------
// enforceRiverChannels — the step-2 re-clamp
// ---------------------------------------------------------------------------

#[test]
fn a_preexisting_lock_is_reclamped_matching_the_reference() {
    // Row 32, x in 20..44, locked at floor 0.30 before the commit; a
    // Mountains stamp then paints straight over it. The hash differs from
    // `hidden_river_is_skipped`'s despite the identical stamp and identical
    // changed-cell count, which is precisely the re-clamp doing work.
    check(
        &Golden {
            name: "preexisting_lock_reclamped",
            changed: 1418,
            hash: "1081ae5ef55df6a4",
            river_any: true,
            river_mask_count: 24,
            lake_mask_count: None,
            samples: [
                1050253722, 1060645656, 1054355295, 1060568030, 1050924810, 1059732849,
            ],
            river_floor_samples: [1050253722, 0, 0, 0, 0, 0],
            river_mask_samples: [1, 0, 0, 0, 0, 0],
            lake_mask_samples: None,
        },
        vec![(stamp(Feature::Mountains, stroke()), false)],
        Some(&|w: &mut WaterState| {
            for x in 20..44 {
                let i = 32 * GW + x;
                w.river_mask[i] = 1;
                w.river_floor[i] = 0.30;
            }
            w.river_any = true;
        }),
    );
}

// ---------------------------------------------------------------------------
// Lake — the water_only dry run
// ---------------------------------------------------------------------------

#[test]
fn lake_only_matches_the_reference() {
    check(
        &Golden {
            name: "lake_only",
            changed: 439,
            hash: "41ea7c908a9ae6a3",
            river_any: false,
            river_mask_count: 0,
            lake_mask_count: Some(256),
            samples: [
                1058084444, 1057467924, 1048723301, 1054905159, 1050924810, 1059732849,
            ],
            river_floor_samples: ZERO6,
            river_mask_samples: ZEROM,
            lake_mask_samples: Some([1, 0, 1, 1, 0, 0]),
        },
        vec![(stamp(Feature::Lake, tap()), false)],
        None,
    );
}

#[test]
fn river_then_lake_matches_the_reference() {
    // The ordering case: the lake's water surface is tested against the
    // post-river-carve height, so its deposited cell count (248) differs
    // from the lake-alone case (256).
    check(
        &Golden {
            name: "river_then_lake",
            changed: 1377,
            hash: "6d6e87d72574e1e6",
            river_any: true,
            river_mask_count: 3,
            lake_mask_count: Some(248),
            samples: [
                1056184383, 1054951341, 1046319175, 1052361895, 1050924810, 1059732794,
            ],
            river_floor_samples: ZERO6,
            river_mask_samples: ZEROM,
            lake_mask_samples: Some([1, 0, 1, 1, 0, 0]),
        },
        vec![
            (stamp(Feature::River, stroke()), false),
            (stamp(Feature::Lake, tap()), false),
        ],
        None,
    );
}

#[test]
fn dense_river_then_lake_matches_the_reference() {
    check(
        &Golden {
            name: "dense_river_then_lake",
            changed: 1377,
            hash: "98f839e32e7c057b",
            river_any: true,
            river_mask_count: 35,
            lake_mask_count: Some(248),
            samples: [
                1054951342, 1054951341, 1046319175, 1052361895, 1050924810, 1059732794,
            ],
            river_floor_samples: [1054951342, 0, 0, 0, 0, 0],
            river_mask_samples: [1, 0, 0, 0, 0, 0],
            lake_mask_samples: Some([1, 0, 1, 1, 0, 0]),
        },
        vec![
            (stamp(Feature::River, stroke_dense()), false),
            (stamp(Feature::Lake, tap()), false),
        ],
        None,
    );
}

/// A cross-check the individual cases cannot make: no two of these commits
/// produce the same field. A harness carrying the same copy-paste error in
/// every case would still pass each one on its own.
#[test]
fn every_case_produces_a_distinct_field() {
    let hashes = [
        "64c26d26e208a96c",
        "e284e542e1de9a13",
        "ebb59c6989550ebf",
        "9e02d0b323215eed",
        "1b509544c6413163",
        "c6884a23f7c5e73b",
        "1081ae5ef55df6a4",
        "41ea7c908a9ae6a3",
        "6d6e87d72574e1e6",
        "98f839e32e7c057b",
    ];
    let mut sorted = hashes;
    sorted.sort_unstable();
    let before = sorted.len();
    let mut dedup = sorted.to_vec();
    dedup.dedup();
    assert_eq!(dedup.len(), before, "two cases share a field hash");
}
