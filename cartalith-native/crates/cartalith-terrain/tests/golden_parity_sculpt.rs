//! Golden-parity tests for the Sculpt editor's landform stamps —
//! `UNIFIED_TOOL_PLAN.md` milestone B.
//!
//! ## The harness
//!
//! Node `vm.runInContext`, this project's established practice, transient
//! and not checked in. The reference's Sculpt core is genuinely *"pure,
//! DOM-free"* (its own section header), so unlike the civ/hydrology golden
//! tests this one needs no `generate()` run, no `state`, no canvas — four
//! contiguous line slices of the real file are the entire dependency set:
//!
//! | lines | what |
//! |---|---|
//! | 2292-2293 | `hash`, `vnoise` |
//! | 7568-7569 | `clamp01`, `smoothstep` |
//! | 8304 | `lerp` |
//! | 8821-9081 | the whole Sculpt pure core (noise families, geometry, `SCULPT_GLOBAL_DEF`, `_sculptCtx`, `SCULPT_FEATURES`, `SCULPT_PRESETS`, `sculptStampRadius`/`sculptStampBBox`/`sculptApplyStamp`) |
//!
//! Every slice carries a **block-comment balance assertion** (`/*` count ==
//! `*/` count) plus a "starts at a top-level boundary" check — the harness
//! technique Journey Planner milestone 4 designed after two genuine
//! boundary errors. That matters more than usual here: the 8821-9081 block
//! both opens and closes with a long `/* ... */` comment, so an off-by-one
//! at either end would have spliced a comment open and silently swallowed
//! code rather than throwing a syntax error.
//!
//! One shim, disclosed: `sculptDefaultParams` (line 9102) lives just past
//! the pure core, in the UI half. The harness re-declares its three-line
//! body (`for(const c of feat.controls) out[c[0]]=c[5]`) rather than
//! widening the slice into DOM-dependent code. It reads the registry's own
//! control tuples, so the defaults themselves still come from the reference.
//!
//! ## Fixtures
//!
//! A 64×64 grid, `field[i] = ((i*37) % 101)/200 + 0.2` (an `f32` sawtooth
//! that is deliberately *not* flat — a flat base would hide every
//! `h0`-dependent branch, and River/Lake/Plateau/Coastline/Mesa are all
//! `h0`-dependent). `seaLevel = 0.5`, `seed = 1234`, `brushSize = 12`,
//! everything else at `SCULPT_GLOBAL_DEF`. Stroke `(10,32) -> (54,32)`;
//! the radial and tap-once cases use the single point `(32,32)`. Volcano
//! gets `volcRadius = 20` so its cone fits the grid.
//!
//! Twenty-two cases: the twelve non-Freehand features, Freehand's eight
//! sub-modes, the "Alps" preset, and Lake's commit-time `waterOnly` dry
//! run. Each is checked **exactly**: an FNV-1a-64 fold over every cell's
//! raw `f32` bit pattern (so a one-ULP difference anywhere in 4096 cells
//! fails), the changed-cell count, six sampled cells as raw bits, and the
//! stamp's bounding box. No tolerance is used or needed — see below.
//!
//! ## Exactness, and why it was not obvious in advance
//!
//! `cartalith-native/docs/CHANGELOG.md` records a prior `1e-4` tolerance
//! for `Math.pow`/`exp`/`hypot` against Rust's `f64`. These stamps use all
//! three (Mountains' `pow`, Ridge's `exp`, every distance's `hypot`) and
//! still diff **bit-exactly**. The reason is the `f32` store: every value
//! is rounded to `f32` at exactly the point the JS `Float32Array`
//! assignment rounds it, which absorbs the last-ULP `f64` disagreement
//! `pow`/`exp` can produce between V8's fdlibm and the platform libm.
//!
//! That same `f32` store is why these fixtures do **not** distinguish
//! `js_hypot`'s V8-faithful Kahan form from plain `sqrt(x*x+y*y)` — tried,
//! measured, still 23/23. See `js_hypot`'s own doc comment: it is kept for
//! fidelity, not because this file enforces it.
//!
//! The one thing that *is* razor-thin: `base_field` must do its arithmetic
//! in `f64` and round once at the store. Computing it in `f32` shifts the
//! whole field by an ULP and every case here fails.

use cartalith_spatial::Stamp;
use cartalith_terrain::sculpt::{
    FEATURE_KEYS, Feature, FeatureParams, FreehandMode, Point, SCULPT_PRESETS, SculptGlobals,
    SculptStamp,
};

const GW: usize = 64;
const GH: usize = 64;
const SEA: f64 = 0.5;
const SEED: u32 = 1234;

/// Same six cells the harness sampled: stroke centre, stroke interior,
/// above/below the stroke, and two cells the stamps mostly miss.
const SAMPLES: [usize; 6] = [
    32 * GW + 32,
    32 * GW + 12,
    26 * GW + 32,
    38 * GW + 32,
    20 * GW + 20,
    44 * GW + 50,
];

fn base_field() -> Vec<f32> {
    // f64 arithmetic, rounded to f32 only at the store — exactly where JS's
    // `Float32Array` assignment rounds. Doing the division in f32 shifts
    // the base field by an ULP and every case below fails, which is a
    // useful reminder of how tight this comparison is.
    (0..GW * GH)
        .map(|i| ((((i * 37) % 101) as f64) / 200.0 + 0.2) as f32)
        .collect()
}

/// FNV-1a 64 over each cell's raw `f32` bit pattern — the same fold the
/// harness ran over the `Float32Array`'s `Uint32Array` view.
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

fn tap() -> Vec<Point> {
    vec![Point::new(32.0, 32.0)]
}

struct Golden {
    name: &'static str,
    changed: usize,
    hash: &'static str,
    /// Inclusive `{x0,y0,x1,y1}`, as `sculptStampBBox` returns it.
    bbox: (usize, usize, usize, usize),
    samples: [u32; 6],
}

#[track_caller]
fn check(g: &Golden, s: &SculptStamp) -> Vec<f32> {
    let base = base_field();
    let mut field = base.clone();
    s.apply(&mut field, GW, GH);

    let b = s.bounds(GW, GH);
    assert_eq!(
        (b.x, b.y, b.x + b.w - 1, b.y + b.h - 1),
        g.bbox,
        "{}: bounding box",
        g.name
    );

    let changed = (0..GW * GH).filter(|&i| field[i] != base[i]).count();
    assert_eq!(changed, g.changed, "{}: changed-cell count", g.name);

    for (n, &i) in SAMPLES.iter().enumerate() {
        assert_eq!(
            field[i].to_bits(),
            g.samples[n],
            "{}: sample {n} at cell {i} ({} vs {})",
            g.name,
            field[i],
            f32::from_bits(g.samples[n])
        );
    }
    assert_eq!(fnv(&field), g.hash, "{}: whole-field hash", g.name);
    field
}

// ---------------------------------------------------------------------------
// The twelve non-Freehand features
// ---------------------------------------------------------------------------

#[test]
fn mountains_matches_the_reference() {
    check(
        &Golden {
            name: "mountains",
            changed: 1418,
            hash: "c6884a23f7c5e73b",
            bbox: (0, 7, 63, 57),
            samples: [
                1060248454, 1060645656, 1054355295, 1060568030, 1050924810, 1059732849,
            ],
        },
        &stamp(Feature::Mountains, stroke()),
    );
}

#[test]
fn hills_matches_the_reference() {
    check(
        &Golden {
            name: "hills",
            changed: 1436,
            hash: "e6f7936c5aaceb84",
            bbox: (0, 9, 63, 55),
            samples: [
                1061223741, 1058370334, 1053931338, 1058166614, 1050933697, 1059740081,
            ],
        },
        &stamp(Feature::Hills, stroke()),
    );
}

#[test]
fn ridge_matches_the_reference() {
    check(
        &Golden {
            name: "ridge",
            changed: 1080,
            hash: "d53c1fa98b9a94af",
            bbox: (0, 8, 63, 56),
            samples: [
                1062214140, 1059690435, 1052105711, 1057468364, 1050924810, 1059732849,
            ],
        },
        &stamp(Feature::Ridge, stroke()),
    );
}

#[test]
fn plateau_matches_the_reference() {
    check(
        &Golden {
            name: "plateau",
            changed: 1455,
            hash: "996dd5bba9f778c6",
            bbox: (0, 8, 63, 56),
            samples: [
                1061452513, 1061326684, 1061443712, 1061326684, 1050924810, 1059915596,
            ],
        },
        &stamp(Feature::Plateau, stroke()),
    );
}

#[test]
fn cliff_matches_the_reference() {
    check(
        &Golden {
            name: "cliff",
            changed: 1422,
            hash: "bcf93aad6aee312c",
            bbox: (0, 9, 63, 55),
            samples: [
                1059791369, 1057754844, 1048240455, 1059292141, 1050924810, 1059732849,
            ],
        },
        &stamp(Feature::Cliff, stroke()),
    );
}

#[test]
fn canyon_matches_the_reference() {
    check(
        &Golden {
            name: "canyon",
            changed: 1333,
            hash: "1b84f3de35f122ac",
            bbox: (0, 8, 63, 56),
            samples: [
                1057537057, 1052512910, 1048881177, 1053080622, 1050924810, 1059732849,
            ],
        },
        &stamp(Feature::Canyon, stroke()),
    );
}

#[test]
fn valley_matches_the_reference() {
    check(
        &Golden {
            name: "valley",
            changed: 1207,
            hash: "e79dcdbc40d0adfa",
            bbox: (0, 9, 63, 55),
            samples: [
                1058038958, 1053373398, 1051596823, 1055139961, 1050924810, 1059732849,
            ],
        },
        &stamp(Feature::Valley, stroke()),
    );
}

#[test]
fn basin_matches_the_reference() {
    check(
        &Golden {
            name: "basin",
            changed: 1434,
            hash: "4d3f0c344f1579b0",
            bbox: (0, 9, 63, 55),
            samples: [
                1058620243, 1054948546, 1051175893, 1056934038, 1050924159, 1059732849,
            ],
        },
        &stamp(Feature::Basin, stroke()),
    );
}

#[test]
fn coastline_matches_the_reference() {
    check(
        &Golden {
            name: "coastline",
            changed: 1434,
            hash: "4aba128c5fca31f3",
            bbox: (0, 7, 63, 57),
            samples: [
                1057028808, 1055790270, 1055184396, 1054944064, 1051058332, 1059732849,
            ],
        },
        &stamp(Feature::Coastline, stroke()),
    );
}

#[test]
fn volcano_matches_the_reference() {
    // Radial, and the one feature that sizes itself from its own control
    // (`volcRadius`) rather than the shared brush size.
    let mut s = stamp(Feature::Volcano, stroke());
    s.params = FeatureParams::Volcano {
        volc_height: 0.45,
        crater_depth: 0.5,
        volc_radius: 20.0,
        flank_rough: 0.6,
    };
    check(
        &Golden {
            name: "volcano",
            changed: 1173,
            hash: "cbc9cb3ae3b55931",
            bbox: (0, 0, 63, 63),
            samples: [
                1064339175, 1057467924, 1062161433, 1062754528, 1052782575, 1059732849,
            ],
        },
        &s,
    );
}

// ---------------------------------------------------------------------------
// The two water features — height *and* the water-surface array
// ---------------------------------------------------------------------------

#[test]
fn river_matches_the_reference_including_its_water_surface() {
    let s = stamp(Feature::River, stroke());
    check(
        &Golden {
            name: "river",
            changed: 1364,
            hash: "279c2f0f35f50c49",
            bbox: (0, 10, 63, 54),
            samples: [
                1058726216, 1054951341, 1050823501, 1055427976, 1050924810, 1059732794,
            ],
        },
        &s,
    );
    let mut field = base_field();
    let mut water = vec![-1.0f32; GW * GH];
    s.apply_into(&mut field, Some(&mut water), GW, GH, false);
    assert_eq!(water.iter().filter(|&&v| v >= 0.0).count(), 759);
    assert_eq!(fnv(&water), "d221ceb4788fde11");
}

#[test]
fn lake_matches_the_reference_including_its_water_surface() {
    let s = stamp(Feature::Lake, tap());
    check(
        &Golden {
            name: "lake",
            changed: 439,
            hash: "41ea7c908a9ae6a3",
            bbox: (8, 8, 56, 56),
            samples: [
                1058084444, 1057467924, 1048723301, 1054905159, 1050924810, 1059732849,
            ],
        },
        &s,
    );
    let mut field = base_field();
    let mut water = vec![-1.0f32; GW * GH];
    s.apply_into(&mut field, Some(&mut water), GW, GH, false);
    assert_eq!(water.iter().filter(|&&v| v >= 0.0).count(), 256);
    assert_eq!(fnv(&water), "8c9189ef06914191");
}

#[test]
fn the_lake_water_only_dry_run_matches_the_reference() {
    // `sculptCommit`'s real ordering: bake the whole stack first, *then*
    // re-run each Lake stamp with `waterOnly` so `h0` is the final
    // post-bake height. The reference's own comment says calling it again
    // with `waterOnly = false` "would double-carve the bowl", and this
    // pins both halves: the height is untouched and the deposited surface
    // matches bit for bit.
    let s = stamp(Feature::Lake, tap());
    let mut field = base_field();
    s.apply(&mut field, GW, GH);
    let baked = field.clone();

    let mut water = vec![-1.0f32; GW * GH];
    s.apply_into(&mut field, Some(&mut water), GW, GH, true);

    assert_eq!(
        field.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        baked.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        "the water-only pass must not write height"
    );
    assert_eq!(fnv(&field), "41ea7c908a9ae6a3");
    assert_eq!(water.iter().filter(|&&v| v >= 0.0).count(), 256);
    assert_eq!(fnv(&water), "94b41d1e0cea4951");
}

// ---------------------------------------------------------------------------
// Freehand's eight sub-modes
// ---------------------------------------------------------------------------

fn freehand(mode: FreehandMode, points: Vec<Point>) -> SculptStamp {
    let mut s = stamp(Feature::Freehand, points);
    s.params = FeatureParams::Freehand {
        amount: 0.12,
        sub_mode: mode,
    };
    s
}

#[test]
fn freehand_raise_matches_the_reference() {
    check(
        &Golden {
            name: "freehand:raise",
            changed: 1432,
            hash: "ae511cb0c9686fbd",
            bbox: (0, 10, 63, 54),
            samples: [
                1062249431, 1059481190, 1056125747, 1059481190, 1050924810, 1059732849,
            ],
        },
        &freehand(FreehandMode::Raise, stroke()),
    );
}

#[test]
fn freehand_lower_matches_the_reference() {
    check(
        &Golden {
            name: "freehand:lower",
            changed: 1432,
            hash: "0e8c4f4d93287336",
            bbox: (0, 10, 63, 54),
            samples: [
                1058222899, 1053944708, 1047569366, 1053944708, 1050924810, 1059732849,
            ],
        },
        &freehand(FreehandMode::Lower, stroke()),
    );
}

#[test]
fn freehand_smooth_matches_the_reference() {
    // The one feature that bypasses the per-pixel `apply()` path entirely:
    // a 4-neighbour blur over a *stable pre-loop snapshot*. A port that
    // read the live-mutating buffer instead would still smooth, still look
    // plausible, and fail here — which is the whole point of golden-testing
    // it rather than asserting "roughness went down".
    check(
        &Golden {
            name: "freehand:smooth",
            changed: 1267,
            hash: "58057dae84ac744e",
            bbox: (0, 10, 63, 54),
            samples: [
                1055035228, 1049498747, 1058768159, 1049498747, 1050924810, 1059732849,
            ],
        },
        &freehand(FreehandMode::Smooth, stroke()),
    );
}

#[test]
fn freehand_cliff_matches_the_reference() {
    check(
        &Golden {
            name: "freehand:cliff",
            changed: 1432,
            hash: "eea206de1c702105",
            bbox: (0, 10, 63, 54),
            samples: [
                1059995916, 1057330398, 1047780870, 1059427476, 1050924810, 1059732849,
            ],
        },
        &freehand(FreehandMode::Cliff, stroke()),
    );
}

#[test]
fn freehand_ridge_matches_the_reference() {
    check(
        &Golden {
            name: "freehand:ridge",
            changed: 1408,
            hash: "5cc0bfe4ac34d826",
            bbox: (0, 10, 63, 54),
            samples: [
                1062244203, 1059479492, 1052248864, 1057546854, 1050924810, 1059732849,
            ],
        },
        &freehand(FreehandMode::Ridge, stroke()),
    );
}

#[test]
fn freehand_canyon_matches_the_reference() {
    check(
        &Golden {
            name: "freehand:canyon",
            changed: 578,
            hash: "9c4a03f02788e8e0",
            bbox: (0, 10, 63, 54),
            samples: [
                1057458244, 1052207138, 1052099215, 1057467924, 1050924810, 1059732849,
            ],
        },
        &freehand(FreehandMode::Canyon, stroke()),
    );
}

#[test]
fn freehand_mesa_matches_the_reference_from_a_single_tap() {
    // A 1-point "stroke" degenerating to radial distance is the mechanism,
    // not an edge case — one registry entry serving both drag and tap.
    check(
        &Golden {
            name: "freehand:mesa",
            changed: 440,
            hash: "e82177415848b13a",
            bbox: (10, 10, 54, 54),
            samples: [
                1062752747, 1057467924, 1057048494, 1059984506, 1050924810, 1059732849,
            ],
        },
        &freehand(FreehandMode::Mesa, tap()),
    );
}

#[test]
fn freehand_volcano_matches_the_reference_from_a_single_tap() {
    check(
        &Golden {
            name: "freehand:volcano",
            changed: 439,
            hash: "f85eeca08b72bd76",
            bbox: (10, 10, 54, 54),
            samples: [
                1063143237, 1057467924, 1053751101, 1058306681, 1050924810, 1059732849,
            ],
        },
        &freehand(FreehandMode::Volcano, tap()),
    );
}

// ---------------------------------------------------------------------------
// A preset, end to end
// ---------------------------------------------------------------------------

#[test]
fn the_alps_preset_reproduces_the_reference_s_own_parameter_seed() {
    // Runs the preset the way the UI does — `apply` writes its `noiseScale`
    // into the globals and returns the feature params — so a wrong preset
    // value fails here rather than silently producing plausible mountains.
    let preset = SCULPT_PRESETS
        .iter()
        .find(|p| p.name == "Alps")
        .expect("Alps preset");
    let mut s = stamp(preset.feature, stroke());
    s.params = preset.apply(&mut s.globals);
    assert_eq!(s.globals.noise_scale, 5.0);
    check(
        &Golden {
            name: "preset:Alps",
            changed: 1418,
            hash: "96629666ecc58f1b",
            bbox: (0, 7, 63, 57),
            samples: [
                1062186328, 1057872407, 1054082826, 1059592630, 1050924810, 1059732849,
            ],
        },
        &s,
    );
}

// ---------------------------------------------------------------------------
// Cross-cutting
// ---------------------------------------------------------------------------

#[test]
fn no_two_features_produce_the_same_field_at_the_same_seed() {
    // The `(feature_index + 1) * 1013` seed term plus thirteen distinct
    // formulas: if a copy-paste error made two entries identical, the
    // per-feature goldens above would still each pass against a harness run
    // with the same mistake in it. This one would not.
    let mut hashes = Vec::new();
    for f in FEATURE_KEYS {
        let mut s = stamp(f, if f == Feature::Lake { tap() } else { stroke() });
        if f == Feature::Volcano {
            s.params = FeatureParams::Volcano {
                volc_height: 0.45,
                crater_depth: 0.5,
                volc_radius: 20.0,
                flank_rough: 0.6,
            };
        }
        let mut field = base_field();
        s.apply(&mut field, GW, GH);
        hashes.push((f.meta().key, fnv(&field)));
    }
    for i in 0..hashes.len() {
        for j in i + 1..hashes.len() {
            assert_ne!(
                hashes[i].1, hashes[j].1,
                "{} and {} produced identical fields",
                hashes[i].0, hashes[j].0
            );
        }
    }
}
