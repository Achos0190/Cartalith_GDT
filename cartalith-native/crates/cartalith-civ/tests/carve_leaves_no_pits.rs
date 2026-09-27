//! RV-1 (`LARGE_ITEM_RULINGS.md` Ruling BD): the river carve must not leave
//! closed depressions along its channels, because `build_water_bodies`
//! classifies every one of them as a lake.
//!
//! Measured before the fix by `_riverzoom_probe` and by the scratch harness
//! disclosed in `STATUS.md`: 10-15% of traced river cells were lake-class on
//! the probe's worlds, and most of those lakes were 1-3 cells. These fixtures
//! reproduce each mechanism in a few hundred cells and run the REAL classifier
//! over both carves: the per-run `enforce_channel_descent` loop generation
//! used to run (which must show the lake -- the positive control that the
//! fixture reaches the mechanism) and `carve_channel_network` (which must
//! not). `cartalith-hydrology`'s own unit tests pin the same fixtures against a
//! re-implemented flood; this file is the claim in the classifier's own terms.

use cartalith_civ::build_water_bodies;
use cartalith_hydrology::{carve_channel_network, enforce_channel_descent};

const SEA: f64 = 0.42;

fn centres(cells: &[(usize, usize)]) -> Vec<(f64, f64)> {
    cells.iter().map(|&(x, y)| (x as f64 + 0.5, y as f64 + 0.5)).collect()
}

/// Each point's receiver is the next point; every run's last point drains
/// east, as every fixture here does.
fn recv_of(w: usize, h: usize, polys: &[Vec<(f64, f64)>]) -> Vec<i32> {
    let mut recv = vec![-1i32; w * h];
    let at = |p: &(f64, f64)| p.1 as usize * w + p.0 as usize;
    for p in polys {
        for k in 0..p.len() - 1 {
            recv[at(&p[k])] = at(&p[k + 1]) as i32;
        }
        let (x, y) = (p[p.len() - 1].0 as usize, p[p.len() - 1].1 as usize);
        if recv[y * w + x] < 0 && x + 1 < w {
            recv[y * w + x] = (y * w + x + 1) as i32;
        }
    }
    recv
}

fn old_carve(fld: &mut [f32], w: usize, h: usize, polys: &[Vec<(f64, f64)>], half_w: f64) {
    for p in polys {
        enforce_channel_descent(fld, w, h, p, SEA, half_w, 0.0006);
    }
}

/// Classification (`0` land, `1` ocean, `2` lake) at each cell of the runs,
/// last point of each run excluded as `_riverzoom_probe` excludes it.
fn on_path(cls: &[u8], w: usize, polys: &[Vec<(f64, f64)>]) -> Vec<u8> {
    polys
        .iter()
        .flat_map(|p| p[..p.len() - 1].iter().map(|&(x, y)| cls[y as usize * w + x as usize]))
        .collect()
}

/// A lowland valley stepping diagonally, walls 0.001 high, floor falling
/// 0.0002 a step (slower than `drop`): no lake before any carve.
#[test]
fn a_diagonal_trench_no_longer_classifies_as_a_string_of_lakes() {
    let (w, h) = (20usize, 20usize);
    let mut base = vec![0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let diag = 0.50 - 0.0002 * x.max(y) as f64 + 0.001 * (x as f64 - y as f64).abs();
            let east = if x >= 16 { 0.50 - 0.0002 * x as f64 + 0.001 * (y as f64 - 16.0).abs() } else { f64::INFINITY };
            base[y * w + x] = if x == w - 1 { 0.30 } else { diag.min(east) as f32 };
        }
    }
    let mut cells: Vec<(usize, usize)> = (1..=16).map(|k| (k, k)).collect();
    cells.extend([(17, 16), (18, 16)]);
    let polys = vec![centres(&cells)];

    let lakes = |f: &[f32]| build_water_bodies(f, w, h, SEA, false, None).classification.iter().filter(|&&c| c == 2).count();
    assert_eq!(lakes(&base), 0, "fixture: the uncarved valley holds no lake");

    let mut old = base.clone();
    old_carve(&mut old, w, h, &polys, 0.5);
    let old_cls = build_water_bodies(&old, w, h, SEA, false, None).classification;
    let old_lake_on_path = on_path(&old_cls, w, &polys).iter().filter(|&&c| c == 2).count();
    assert!(old_lake_on_path >= 4, "positive control: the per-run carve left {old_lake_on_path} lake cells on the channel");

    let mut new = base.clone();
    carve_channel_network(&mut new, w, h, false, &polys, &[0.5], &recv_of(w, h, &polys), None, SEA, 0.0006);
    assert_eq!(lakes(&new), 0, "the network carve leaves no lake anywhere on the map");
}

/// A tributary that crossed a deep hole reaches the trunk far below the
/// trunk's floor; the trunk is carved first, as trace order has it.
#[test]
fn a_confluence_no_longer_classifies_as_a_lake() {
    let (w, h) = (16usize, 12usize);
    let mut base = vec![0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let trunk = 0.60 - 0.005 * x as f64 + 0.03 * (y as f64 - 6.0).abs();
            let trib = if y < 6 { 0.56 + 0.004 * (6 - y) as f64 + 0.03 * (x as f64 - 8.0).abs() } else { f64::INFINITY };
            base[y * w + x] = if x == w - 1 { 0.30 } else { trunk.min(trib) as f32 };
        }
    }
    base[2 * w + 8] = 0.50;
    let trunk: Vec<(usize, usize)> = (1..=14).map(|x| (x, 6)).collect();
    let trib: Vec<(usize, usize)> = (0..=6).map(|y| (8, y)).collect();
    let polys = vec![centres(&trunk), centres(&trib)];
    let junction = 6 * w + 8;

    let mut old = base.clone();
    old_carve(&mut old, w, h, &polys, 0.5);
    let old_cls = build_water_bodies(&old, w, h, SEA, false, None).classification;
    assert_eq!(old_cls[junction], 2, "positive control: the per-run carve's confluence pit is a lake");

    let mut new = base.clone();
    carve_channel_network(&mut new, w, h, false, &polys, &[0.5, 0.5], &recv_of(w, h, &polys), None, SEA, 0.0006);
    let new_cls = build_water_bodies(&new, w, h, SEA, false, None).classification;
    assert_eq!(new_cls[junction], 0, "the confluence is land");
    assert!(on_path(&new_cls, w, &polys).iter().all(|&c| c == 0), "every channel cell is land");
}

/// A long trunk across a plain 0.005 above sea level: the old accumulated
/// floor walked under sea level and the ocean flooded the valley from the
/// mouth -- the "trunk vanishes into a sea-classed valley" report.
#[test]
fn a_lowland_trunk_is_not_drowned_by_the_ocean() {
    let (w, h) = (32usize, 9usize);
    let mut base = vec![0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            base[y * w + x] = if x == w - 1 { 0.30 } else { (0.425 - 0.00005 * x as f64 + 0.01 * (y as f64 - 4.0).abs()) as f32 };
        }
    }
    let polys = vec![centres(&(1..=30).map(|x| (x, 4)).collect::<Vec<_>>())];

    let mut old = base.clone();
    old_carve(&mut old, w, h, &polys, 1.2);
    let old_cls = build_water_bodies(&old, w, h, SEA, false, None).classification;
    let old_ocean = on_path(&old_cls, w, &polys).iter().filter(|&&c| c == 1).count();
    assert!(old_ocean >= 10, "positive control: {old_ocean} channel cells were ocean");

    let mut new = base.clone();
    carve_channel_network(&mut new, w, h, false, &polys, &[1.2], &recv_of(w, h, &polys), None, SEA, 0.0006);
    let new_cls = build_water_bodies(&new, w, h, SEA, false, None).classification;
    assert!(on_path(&new_cls, w, &polys).iter().all(|&c| c == 0), "the whole trunk stays land to its mouth");
    let ocean = |c: &[u8]| c.iter().filter(|&&v| v == 1).count();
    assert_eq!(ocean(&new_cls), ocean(&build_water_bodies(&base, w, h, SEA, false, None).classification), "the sea is exactly the uncarved sea");
}

/// A run that stops short of its outlet: the channel ends at x = 15 and the
/// valley goes on down to the sea uncarved. The old floor, falling `drop` a
/// step, sank the run's last reach below the cell it drains through -- the
/// long channel-shaped lakes the probe's ×8 frames showed.
#[test]
fn a_run_that_stops_short_no_longer_ends_in_a_lake() {
    let (w, h) = (24usize, 9usize);
    let mut base = vec![0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let v = 0.50 - 0.0002 * x as f64 + 0.01 * (y as f64 - 4.0).abs();
            base[y * w + x] = if x == w - 1 { 0.30 } else { v as f32 };
        }
    }
    let polys = vec![centres(&(1..=15).map(|x| (x, 4)).collect::<Vec<_>>())];
    let lakes = |f: &[f32]| build_water_bodies(f, w, h, SEA, false, None).classification.iter().filter(|&&c| c == 2).count();
    assert_eq!(lakes(&base), 0, "fixture: the uncarved valley holds no lake");

    let mut old = base.clone();
    old_carve(&mut old, w, h, &polys, 0.5);
    assert!(lakes(&old) >= 3, "positive control: the per-run carve's dead end is a lake ({} cells)", lakes(&old));

    let mut new = base.clone();
    carve_channel_network(&mut new, w, h, false, &polys, &[0.5], &recv_of(w, h, &polys), None, SEA, 0.0006);
    assert_eq!(lakes(&new), 0, "the run drains through its receiver");
}
