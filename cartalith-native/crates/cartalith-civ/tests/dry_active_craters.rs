//! Owner Ruling CX item 6 (2026-10-24): an active volcano keeps a dry crater;
//! a dormant or extinct one may hold a lake. The generation step
//! (`cartalith_terrain::edifice_pits::dry_active_craters`) is checked here
//! against the water-body classifier it has to satisfy
//! (`cartalith_civ::build_water_bodies`), which this crate owns.

use cartalith_terrain::edifice_pits::{dry_active_craters, CLASSIFIER_LAKE_DEPTH};
use cartalith_terrain::{EdificeKind, PlacedEdifice, VolcanicSetting};

/// Protects: the dry step's threshold is the classifier's own lake depth --
/// two crates, one number (the terrain crate cannot depend on this one).
/// Two independent definitions compared, not a constant against itself.
#[test]
fn the_dry_crater_threshold_is_the_classifiers_lake_depth() {
    assert_eq!(CLASSIFIER_LAKE_DEPTH, cartalith_civ::LAKE_DEPTH);
}

const GW: usize = 64;
const GH: usize = 48;
const SEA: f64 = 0.3;

/// A flat plain at 0.5 with two identical cones, each with a closed summit
/// pit 0.06 deep, and the edifice records that own them: `active` years for
/// the west cone, `other` years for the east one.
fn world(other_years: f64) -> (Vec<f32>, Vec<PlacedEdifice>) {
    let mut f = vec![0.5f32; GW * GH];
    let cones = [(18.0, 24.0), (46.0, 24.0)];
    for &(cx, cy) in &cones {
        for y in 0..GH {
            for x in 0..GW {
                let d = (x as f64 - cx).hypot(y as f64 - cy);
                let i = y * GW + x;
                if d < 10.0 {
                    // A cone 0.2 high with a pit inside 2.5 cells.
                    let h = 0.2 * (1.0 - d / 10.0);
                    let pit = if d < 2.5 { 0.06 * (1.0 - d / 2.5) + 0.03 } else { 0.0 };
                    f[i] = (0.5 + h - pit) as f32;
                }
            }
        }
    }
    let ed = |cx: f64, years: f64| PlacedEdifice {
        rift_bearing: None,
        setting: VolcanicSetting::Arc,
        cx,
        cy: 24.0,
        r: 10.0,
        h: 0.2,
        kind: Some(EdificeKind::Cinder),
        age: 0.0,
        years: Some(years),
        crater: None,
    };
    (f, vec![ed(18.0, 10.0), ed(46.0, other_years)])
}

/// Lake cells (class 2) within 4 cells of `(cx, 24)`.
fn lake_cells(class: &[u8], cx: f64) -> usize {
    (0..GW * GH).filter(|&i| class[i] == 2 && ((i % GW) as f64 - cx).hypot((i / GW) as f64 - 24.0) <= 4.0).count()
}

/// Protects the rule on a world where both craters pool: before the step
/// each holds a lake; after it the ACTIVE crater holds no water cell while a
/// DORMANT (1 000 yr) or EXTINCT (1 Myr) twin keeps its lake. No cell is
/// lowered, nothing outside the active summit zone moves, and the raised
/// floor stays below the pool level (a shallow closed depression, never a
/// new spill).
#[test]
fn an_active_crater_is_dry_and_a_dormant_or_extinct_one_keeps_its_lake() {
    let rain = vec![1.0f32; GW * GH];
    for other in [1_000.0, 1_000_000.0] {
        let (mut f, eds) = world(other);
        let before = cartalith_civ::build_water_bodies(&f, GW, GH, SEA, false, Some(&rain));
        assert!(lake_cells(&before.classification, 18.0) > 0 && lake_cells(&before.classification, 46.0) > 0, "premise: both craters pool");
        let f0 = f.clone();
        let r = dry_active_craters(GW, GH, false, &mut f, SEA, &eds);
        assert_eq!(r.craters, 1, "one active crater dried");
        assert!(r.cells > 0);
        let after = cartalith_civ::build_water_bodies(&f, GW, GH, SEA, false, Some(&rain));
        assert_eq!(lake_cells(&after.classification, 18.0), 0, "the active crater holds water");
        assert_eq!(lake_cells(&after.classification, 46.0), lake_cells(&before.classification, 46.0), "the {other}-year crater lost its lake");
        let zone = cartalith_terrain::volcano_attr::summit_zone_t(&eds[0]).unwrap() * eds[0].r;
        for i in 0..GW * GH {
            assert!(f[i] >= f0[i], "cell {i} was lowered");
            if f[i] != f0[i] {
                let d = ((i % GW) as f64 - 18.0).hypot((i / GW) as f64 - 24.0);
                assert!(d < zone, "cell {i} outside the active summit zone moved");
                // A recess stays: the floor sits at least 0.4 x the
                // classifier's lake depth (0.004, a literal) under the pool
                // level, the lava-lake level below the spill point.
                assert!(after.fill_level[i] as f64 - f[i] as f64 >= 0.4 * 0.004, "cell {i} raised to its pool level ({} vs {})", f[i], after.fill_level[i]);
            }
        }
    }
}

/// Protects the owner rule: a summit-zone cell that another edifice owns by
/// the stamp's rule (`volcano_attr::VolcanoIndex::winner_at`) is not raised
/// for the active one -- a dormant record centred 3 cells east of the active
/// crater wins the crater's eastern cells, and they keep their height.
#[test]
fn a_cell_another_edifice_owns_is_not_raised() {
    let (mut f, mut eds) = world(1_000.0);
    let mut other = eds[1];
    other.cx = 21.0;
    eds.push(other);
    let f0 = f.clone();
    dry_active_craters(GW, GH, false, &mut f, SEA, &eds);
    let idx = cartalith_terrain::volcano_attr::VolcanoIndex::new(&eds, GW, GH);
    let (mut kept, mut raised) = (0, 0);
    for i in 0..GW * GH {
        let (x, y) = ((i % GW) as f64, (i / GW) as f64);
        if (x - 18.0).hypot(y - 24.0) >= 2.5 {
            continue;
        }
        match idx.winner_at(x, y).map(|h| h.index) {
            Some(0) => raised += usize::from(f[i] > f0[i]),
            Some(_) => {
                assert_eq!(f[i], f0[i], "cell {i}, owned by another edifice, moved");
                kept += 1;
            }
            None => {}
        }
    }
    assert!(kept > 0 && raised > 0, "premise: the pit is split between two owners ({kept} kept, {raised} raised)");
}

/// Protects the gate: with no active edifice the step changes nothing (and
/// costs no flood), and an edifice with no morphology is never treated as
/// active.
#[test]
fn no_active_edifice_means_no_change() {
    let (mut f, mut eds) = world(1_000.0);
    eds[0].years = Some(500.0);
    let f0 = f.clone();
    assert_eq!(dry_active_craters(GW, GH, false, &mut f, SEA, &eds).cells, 0);
    assert_eq!(f, f0);
    eds[0].years = Some(10.0);
    eds[0].kind = None;
    assert_eq!(dry_active_craters(GW, GH, false, &mut f, SEA, &eds).cells, 0);
    assert_eq!(f, f0);
}
