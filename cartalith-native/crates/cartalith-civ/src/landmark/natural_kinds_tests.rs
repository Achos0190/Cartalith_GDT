//! Tests for `natural_kinds.rs`: fixtures shaped to reach each detector (`CLAUDE.md`: "shape
//! fixtures to reach the code"). The really-generated-world checks live in `landmark.rs`'s own
//! test module, beside `real_world`, so a kind cannot be flagged buildable and place nothing.

use super::*;

/// 0.42, as every fixture in `landmark.rs`: `WorldParams::defaults().sea_level`.
const SEA: f64 = 0.42;

/// Settings with exactly one kind armed at its default cap.
fn only(key: &str) -> LandmarkSettings {
    let mut s = LandmarkSettings::default();
    for k in kinds() {
        s.set_armed(k.key, k.key == key);
    }
    s
}

/// A west-to-east strip: land for `x < coast`, ocean from `coast`, with one channel along the
/// middle row flowing east into the sea. Every raster the delta and salt fixtures need.
struct Strip {
    gw: usize,
    gh: usize,
    field: Vec<f32>,
    water: Vec<u8>,
    chan: Vec<u8>,
    recv: Vec<i32>,
    flow: Vec<f32>,
}

fn strip(coast: usize, land_h: f32, flow_q: f32) -> Strip {
    let (gw, gh) = (60usize, 9usize);
    let row = gh / 2;
    let mut s = Strip {
        gw,
        gh,
        field: vec![0.10; gw * gh],
        water: vec![1u8; gw * gh],
        chan: vec![0u8; gw * gh],
        recv: vec![-1i32; gw * gh],
        flow: vec![0f32; gw * gh],
    };
    for y in 0..gh {
        for x in 0..coast {
            s.field[y * gw + x] = land_h;
            s.water[y * gw + x] = 0;
        }
    }
    for x in 0..coast {
        let i = row * gw + x;
        s.chan[i] = 1;
        s.recv[i] = (i + 1) as i32;
        s.flow[i] = flow_q;
    }
    s
}

fn run_delta(s: &Strip) -> LandmarkResult {
    let mut inp = LandmarkInputs::new(&s.field, s.gw, s.gh, SEA, false, s.gw as f64 * 4.0);
    inp.channel = Some(&s.chan);
    inp.recv = Some(&s.recv);
    inp.flow = Some(&s.flow);
    inp.water = Some(&s.water);
    generate(&inp, &only("delta"), 1)
}

/// Protects: the positive case. A large river entering the sea across land 3 m above it is a
/// delta, placed on the mouth (the last land cell of the channel), with the discharge and the
/// plain share in its causal chain.
#[test]
fn a_large_river_over_a_low_plain_is_a_delta_at_its_mouth() {
    let s = strip(50, 0.4204, 1.0e6);
    let r = run_delta(&s);
    let lm: Vec<&Landmark> = r.landmarks.iter().filter(|l| l.kind == "delta").collect();
    assert_eq!(lm.len(), 1, "one mouth, one delta: {:?}", r.funnel("delta"));
    assert_eq!((lm[0].x, lm[0].y), (49, 4), "the mouth is the last land cell of the channel");
    assert!(lm[0].causal.iter().any(|f| f.contains("a river mouth into the sea")), "{:?}", lm[0].causal);
    assert!(lm[0].causal.iter().any(|f| f.contains("low coastal plain")), "{:?}", lm[0].causal);
}

/// Protects: each of the delta's constraints is load-bearing. A mountain coast (0.55 = 860 m) and
/// a trickle (no flow, under the 8x threshold) place nothing and are counted as rejected,
/// not silently absent; a river into no water at all is not a candidate.
#[test]
fn a_delta_needs_discharge_a_low_plain_and_standing_water() {
    let high = run_delta(&strip(50, 0.55, 1.0e6));
    assert_eq!(high.placed("delta"), 0);
    assert!(high.funnel("delta").unwrap().rejected_constraint > 0, "rejected by the plain test");
    let trickle = run_delta(&strip(50, 0.4204, 0.0));
    assert_eq!(trickle.placed("delta"), 0);
    assert!(trickle.funnel("delta").unwrap().rejected_constraint > 0, "rejected by the flow test");
    let dry = run_delta(&strip(60, 0.4204, 1.0e6));
    assert_eq!(dry.funnel("delta").unwrap().candidates, 0);
}

/// Protects: honest degradation. Without the receiver tree there is no mouth to find, so the
/// kind reports `NoTerrain` with no candidates rather than a delta on the coast.
#[test]
fn a_delta_without_a_receiver_tree_is_no_terrain() {
    let s = strip(50, 0.4204, 1.0e6);
    let mut inp = LandmarkInputs::new(&s.field, s.gw, s.gh, SEA, false, s.gw as f64 * 4.0);
    inp.channel = Some(&s.chan);
    inp.flow = Some(&s.flow);
    inp.water = Some(&s.water);
    let r = generate(&inp, &only("delta"), 1);
    assert_eq!(r.funnel("delta").unwrap().limit, LandmarkLimit::NoTerrain);
    assert_eq!(r.funnel("delta").unwrap().candidates, 0);
}

fn run_salt(s: &Strip, biome: &[u8]) -> LandmarkResult {
    // A real coast has a shallow shelf, not a drop to 0.10: the gradient test reads the field
    // across the shoreline, so the sea here sits a hair under the sea level (1.5 m deep).
    let mut field = s.field.clone();
    for (f, w) in field.iter_mut().zip(&s.water) {
        if *w == 1 {
            *f = 0.4198;
        }
    }
    let mut inp = LandmarkInputs::new(&field, s.gw, s.gh, SEA, false, s.gw as f64 * 4.0);
    inp.water = Some(&s.water);
    inp.biome = Some(biome);
    generate(&inp, &only("salt_works"), 1)
}

/// Protects: salt works on a dry, flat, sheltered shore, and on no other. The same coast on
/// grassland places none; on a desert it places one, at the shore, with the biome named in the
/// chain. A 70 m step to the sea fails the elevation test.
#[test]
fn a_dry_flat_sheltered_coast_makes_salt_and_a_wet_one_does_not() {
    let s = strip(50, 0.4204, 0.0);
    let desert = vec![crate::BIOME_DESERT; s.gw * s.gh];
    let r = run_salt(&s, &desert);
    let lm: Vec<&Landmark> = r.landmarks.iter().filter(|l| l.kind == "salt_works").collect();
    assert!(!lm.is_empty(), "{:?}", r.funnel("salt_works"));
    assert!(lm.iter().all(|l| l.x == 49), "salt is made at the shore, not inland");
    assert!(lm[0].causal.iter().any(|f| f.contains("desert")), "{:?}", lm[0].causal);
    let grass = vec![crate::BIOME_GRASS; s.gw * s.gh];
    assert_eq!(run_salt(&s, &grass).funnel("salt_works").unwrap().candidates, 0, "a wet coast is not a domain");
    let cliff = strip(50, 0.43, 0.0);
    let r = run_salt(&cliff, &desert);
    assert_eq!(r.placed("salt_works"), 0);
    assert!(r.funnel("salt_works").unwrap().rejected_constraint > 0);
}

/// Protects: salt works needs the biome raster and says so (`NoTerrain`), instead of assuming
/// every coast is dry.
#[test]
fn salt_works_without_a_biome_is_no_terrain() {
    let s = strip(50, 0.4204, 0.0);
    let mut inp = LandmarkInputs::new(&s.field, s.gw, s.gh, SEA, false, s.gw as f64 * 4.0);
    inp.water = Some(&s.water);
    let r = generate(&inp, &only("salt_works"), 1);
    assert_eq!(r.funnel("salt_works").unwrap().limit, LandmarkLimit::NoTerrain);
}

/// A square forest, `side` cells a side, on an otherwise open 60 x 60 map of 4 km cells.
fn forest_world(side: usize) -> (Vec<f32>, Vec<u8>, Vec<u8>) {
    let (gw, gh) = (60usize, 60usize);
    let field = vec![0.60f32; gw * gh];
    let water = vec![0u8; gw * gh];
    let mut biome = vec![crate::BIOME_GRASS; gw * gh];
    for y in 5..5 + side {
        for x in 5..5 + side {
            biome[y * gw + x] = crate::BIOME_TEMP_FOREST;
        }
    }
    (field, water, biome)
}

fn run_forest(field: &[f32], water: &[u8], biome: &[u8], sites: &[LandmarkSite]) -> LandmarkResult {
    let mut inp = LandmarkInputs::new(field, 60, 60, SEA, false, 240.0);
    inp.water = Some(water);
    inp.biome = Some(biome);
    inp.settlements = sites;
    generate(&inp, &only("ancient_forest"), 1)
}

/// Protects: the intact-forest criteria at their two thresholds. A 30-cell (120 km) square is
/// 14 400 km2 and 120 km wide: its heart is an ancient forest. A 4-cell square is 256 km2, under
/// the 500 km2 floor, and places none however deep its middle. Both with a settlement in the far
/// corner of the map.
#[test]
fn a_big_unbroken_forest_has_a_heart_and_a_small_one_does_not() {
    let far = [LandmarkSite { x: 58, y: 58, population: 5000.0 }];
    let (f, w, b) = forest_world(30);
    let r = run_forest(&f, &w, &b, &far);
    let lm: Vec<&Landmark> = r.landmarks.iter().filter(|l| l.kind == "ancient_forest").collect();
    assert!(!lm.is_empty(), "{:?}", r.funnel("ancient_forest"));
    // The forest spans cells 5..35, so its middle is near (20, 20).
    assert!(
        lm.iter().all(|l| (l.x as i64 - 20).abs() <= 3 && (l.y as i64 - 20).abs() <= 3),
        "{:?}",
        lm.iter().map(|l| (l.x, l.y)).collect::<Vec<_>>()
    );
    assert!(lm[0].causal.iter().any(|f| f.contains("temperate forest") && f.contains("unbroken")), "{:?}", lm[0].causal);
    let (f, w, b) = forest_world(4);
    let r = run_forest(&f, &w, &b, &far);
    assert_eq!(r.placed("ancient_forest"), 0, "256 km2 is under the 500 km2 floor");
}

/// Protects: a settlement ends an intact landscape. The same 30-cell forest with a town in its
/// middle still has hearts (the ring around the town), and every one keeps the 5 km half-width from
/// it. Without any settlement the kind is `NoTerrain`, not "untouched".
#[test]
fn a_settlement_breaks_the_forest_and_no_settlements_is_no_terrain() {
    let (f, w, b) = forest_world(30);
    let mid = [LandmarkSite { x: 20, y: 20, population: 5000.0 }];
    let r = run_forest(&f, &w, &b, &mid);
    for l in r.landmarks.iter().filter(|l| l.kind == "ancient_forest") {
        let d = ((l.x as f64 - 20.0).powi(2) + (l.y as f64 - 20.0).powi(2)).sqrt() * 4.0;
        assert!(d >= 5.0, "a heart {d} km from the town it is meant to be clear of");
    }
    assert!(r.placed("ancient_forest") > 0, "the ring around the town is still a forest");
    let none = run_forest(&f, &w, &b, &[]);
    assert_eq!(none.funnel("ancient_forest").unwrap().limit, LandmarkLimit::NoTerrain);
    assert_eq!(none.funnel("ancient_forest").unwrap().candidates, 0);
}

/// Protects: [`chamfer_km`] against exact geometry on a case with a known answer. One source in the
/// middle of a 41-cell square of 1 km cells: an axis cell 20 away is 20 km, and the corner, a pure
/// diagonal, is 20 * sqrt(2) km. No source at all is infinite, never zero.
#[test]
fn the_chamfer_distance_is_exact_along_axes_and_diagonals() {
    let (gw, gh) = (41usize, 41usize);
    let field = vec![0.6f32; gw * gh];
    let inp = LandmarkInputs::new(&field, gw, gh, SEA, false, gw as f64);
    let ctx = Ctx::build(&inp, Needs::default());
    let mut src = vec![false; gw * gh];
    src[20 * gw + 20] = true;
    let d = chamfer_km(&ctx, &src);
    assert!((d[20 * gw + 40] - 20.0).abs() < 1e-3, "{}", d[20 * gw + 40]);
    assert!((d[0] - 20.0 * 2f32.sqrt()).abs() < 1e-3, "{}", d[0]);
    let none = chamfer_km(&ctx, &vec![false; gw * gh]);
    assert!(none.iter().all(|v| v.is_infinite()));
}

fn run_glacial(snow: Option<f64>, temp_c: f32, h: f32) -> LandmarkResult {
    let (gw, gh) = (40usize, 40usize);
    let mut field = vec![h; gw * gh];
    // A 0.05-deep (345 m) circular pit in the middle of the plateau.
    for y in 0..gh {
        for x in 0..gw {
            let d2 = (x as f64 - 20.0).powi(2) + (y as f64 - 20.0).powi(2);
            if d2 < 9.0 {
                field[y * gw + x] -= 0.05;
            }
        }
    }
    let temp = vec![temp_c; gw * gh];
    let mut inp = LandmarkInputs::new(&field, gw, gh, SEA, false, 160.0);
    inp.temperature = Some(&temp);
    inp.glacial_snow_el = snow;
    generate(&inp, &only("glacial_feature"), 1)
}

/// Protects: the glacial kind's three gates. A pit in a plateau above the snowline (0.797) and below
/// freezing is a glacial feature; the same pit with the pass off (`None`), above freezing, or
/// below the snowline has no candidate. The positive case names the snowline and temperature.
#[test]
fn a_hollow_inside_the_glacial_domain_is_a_glacial_feature_and_nowhere_else() {
    let snow = Some(SEA + (1.0 - SEA) * 0.65);
    let r = run_glacial(snow, -8.0, 0.95);
    assert!(r.placed("glacial_feature") > 0, "{:?}", r.funnel("glacial_feature"));
    let lm = r.landmarks.iter().find(|l| l.kind == "glacial_feature").unwrap();
    assert!(lm.causal.iter().any(|f| f.contains("above the snowline") && f.contains("-8 °C")), "{:?}", lm.causal);
    let off = run_glacial(None, -8.0, 0.95);
    assert_eq!(off.funnel("glacial_feature").unwrap().limit, LandmarkLimit::NoTerrain);
    assert_eq!(off.funnel("glacial_feature").unwrap().candidates, 0, "pass off: no claim about ice");
    assert_eq!(run_glacial(snow, 3.0, 0.95).funnel("glacial_feature").unwrap().candidates, 0, "too warm");
    assert_eq!(run_glacial(snow, -8.0, 0.70).funnel("glacial_feature").unwrap().candidates, 0, "under the snowline");
}

/// Protects: every cited constant, by literal, so a drifted threshold is a red test and not a silent
/// shift (`MISTAKES.md`: assert a literal, never the constant against itself). Each literal is
/// the source's own number.
#[test]
fn the_cited_constants_are_the_cited_numbers() {
    assert_eq!(DELTA_PLAIN_MARGIN, 0.06, "the router's marshland margin, measured in place of the 10 m zone");
    assert_eq!(SALT_MAX_ELEV_M, 30.0, "measured: three times the real 10 m zone, see the constant");
    assert_eq!(SALT_MAX_GRADIENT, 0.06, "the harbour's own limit");
    assert_eq!(ANCIENT_FOREST_MIN_WIDTH_KM, 10.0, "Potapov et al. 2008: 10 km wide");
    assert_eq!(ANCIENT_FOREST_MIN_AREA_KM2, 500.0, "Potapov et al. 2008: 50 000 ha");
    assert_eq!(GLACIAL_HEAD_FLOW, 100.0, "glacial_core: cirque overdeepening where discharge < 100");
    assert_eq!(crate::river_class::DELTA_FLOW_MULT, 8.0, "the router's marshland multiple");
}

/// Protects: the forest buffer around a settlement (`ANCIENT_FOREST_BUFFER_KM`, 2 km, one cell
/// here). With a town in the middle of a 30-cell forest the hearts are the ridge of the distance
/// transform between the town's square and the forest's edge, about 8 cells (32 km) from the town;
/// a buffer of 40 km would push that ridge out to about 13 cells (52 km). Every heart is within 10
/// cells (40 km), and there is at least one.
#[test]
fn the_forest_buffer_is_one_cell_round_a_town_not_ten() {
    let (f, w, b) = forest_world(30);
    let mid = [LandmarkSite { x: 20, y: 20, population: 5000.0 }];
    let r = run_forest(&f, &w, &b, &mid);
    let hearts: Vec<f64> = r
        .landmarks
        .iter()
        .filter(|l| l.kind == "ancient_forest")
        .map(|l| ((l.x as f64 - 20.0).powi(2) + (l.y as f64 - 20.0).powi(2)).sqrt() * 4.0)
        .collect();
    assert!(!hearts.is_empty());
    assert!(hearts.iter().all(|d| *d <= 40.0), "a 2 km buffer keeps hearts within 40 km: {hearts:?}");
}

/// A one-cell-wide spit of land (column 30) in an otherwise open sea, flat and low, on desert.
fn spit() -> Strip {
    let mut s = strip(0, 0.4204, 0.0);
    for y in 0..s.gh {
        s.field[y * s.gw + 30] = 0.4204;
        s.water[y * s.gw + 30] = 0;
    }
    s
}

/// Protects: salt works' shelter test. A flat desert spit is surrounded by water: the land share of
/// its 5 x 5 approach is 20 %, under the harbour's 45 % floor, so no pan is placed and the cells are
/// counted as rejected. The same flat desert on a straight coast (60 % land) places.
#[test]
fn salt_needs_sheltered_water_so_an_open_spit_gets_none() {
    let s = spit();
    let desert = vec![crate::BIOME_DESERT; s.gw * s.gh];
    let r = run_salt(&s, &desert);
    assert_eq!(r.placed("salt_works"), 0);
    assert!(r.funnel("salt_works").unwrap().rejected_constraint > 0, "{:?}", r.funnel("salt_works"));
}

/// Protects: salt works' level-shore test. The same straight desert coast with the sea floor 830 m
/// below the shoreline (field 0.30 against 0.4204; central differences halve the step) falls at about 10 %, over the 6 % limit, so
/// nothing is placed; the coast with a shallow shelf (the earlier test) places.
#[test]
fn salt_needs_a_level_shore_so_a_deep_drop_gets_none() {
    let s = strip(50, 0.4204, 0.0);
    let desert = vec![crate::BIOME_DESERT; s.gw * s.gh];
    let mut field = s.field.clone();
    for (f, w) in field.iter_mut().zip(&s.water) {
        if *w == 1 {
            *f = 0.30;
        }
    }
    let mut inp = LandmarkInputs::new(&field, s.gw, s.gh, SEA, false, s.gw as f64 * 4.0);
    inp.water = Some(&s.water);
    inp.biome = Some(&desert);
    let r = generate(&inp, &only("salt_works"), 1);
    assert_eq!(r.placed("salt_works"), 0, "{:?}", r.funnel("salt_works"));
    assert!(r.funnel("salt_works").unwrap().rejected_constraint > 0);
}

/// The glacial fixture with a pit `depth` field units deep (0.05 is 345 m).
fn run_glacial_depth(depth: f32) -> LandmarkResult {
    let (gw, gh) = (40usize, 40usize);
    let mut field = vec![0.95f32; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            if (x as f64 - 20.0).powi(2) + (y as f64 - 20.0).powi(2) < 9.0 {
                field[y * gw + x] -= depth;
            }
        }
    }
    let temp = vec![-8.0f32; gw * gh];
    let mut inp = LandmarkInputs::new(&field, gw, gh, SEA, false, 160.0);
    inp.temperature = Some(&temp);
    inp.glacial_snow_el = Some(SEA + (1.0 - SEA) * 0.65);
    generate(&inp, &only("glacial_feature"), 1)
}

/// Protects: the two glacial thresholds, each by a pit on one side of it. A 0.003 (21 m) dimple is
/// not a hollow (broad TPI above -60 m): no candidate at all. A 0.012 (83 m) pit is a hollow (TPI
/// about -80 m) with only 83 m of relief round it, under the 150 m headwall floor: a candidate
/// refused by constraint, none placed. The 0.05 (345 m) pit of the earlier test is both and places.
#[test]
fn a_glacial_hollow_needs_depth_and_a_headwall() {
    let dimple = run_glacial_depth(0.003);
    assert_eq!(dimple.funnel("glacial_feature").unwrap().candidates, 0, "a dimple is not a hollow");
    let shallow = run_glacial_depth(0.012);
    let f = shallow.funnel("glacial_feature").unwrap();
    assert_eq!(f.placed, 0, "{f:?}");
    assert!(f.rejected_constraint > 0, "a hollow without a headwall is rejected, not absent: {f:?}");
    assert!(run_glacial_depth(0.05).placed("glacial_feature") > 0);
}
