//! GF-8 of `GEOLOGY_FIRST_SCOPE.md` (owner Ruling BJ): bar B11 (§5) on
//! generated worlds, through `cartalith_engine::rock_paint::commit`.
//!
//! **What these tests protect.**
//! - B11's identity: painting the rock that is already there moves the field
//!   by 0 bytes and the column by 0 bytes apart from `painted`. Exact, by
//!   control flow (the two chain runs agree bit for bit, so nothing is
//!   written). The mutation that makes the runs differ is run by hand (see
//!   `GEOLOGY_FIRST_SCOPE.md` §7 "GF-8 measured").
//! - B11's locality: painting shale into a granite interior at 800 km lowers
//!   the painted cells more than the unpainted cells of the same window; no
//!   cell outside the padded window `W` moves; the step at `W`'s ring is
//!   under 2 % of `W`'s relief (`tile_erode`'s own lip bar).
//! - Determinism: the same commit on two copies of a world gives the same
//!   bytes.
//!
//! Undo and the save round trip are `cartalith-godot`'s (`undo.rs`'s column
//! tests; the project round-trip tests), where the undo stack and the
//! archive live.

use cartalith_engine::rock_paint::{commit, CommitReport, RockPaint, Window, REERODE_PAD_CELLS};
use cartalith_engine::{generate_terrain, WorldParams, WorldState};
use cartalith_terrain::geology::Rock;

/// Fixture grid. Source: `geology_gf5.rs`'s second size, the smallest at
/// which its worlds expose every process (a judgement carried from there);
/// 800 km is `WorldParams::defaults()`'s own `map_width_km`, which B11 names.
const GW: usize = 512;
/// See [`GW`].
const GH: usize = 328;
/// Painted disc radius in cells. **Judgement**: big enough to hold a
/// drainage path (a 1-2 cell patch barely routes), small enough that a
/// granite interior that wide exists on the fixture seeds.
const R: i64 = 8;

/// An app-like world (the `params::defaults` divergences `geology_gf5.rs`
/// keeps) with the model, processes and karst on: the world on which painted
/// rock is meant to change erosion.
fn params(seed: i32) -> WorldParams {
    let mut p = WorldParams::defaults(GW, GH, seed);
    p.crater.physical_model = true;
    p.volc.exclude_transform = true;
    p.volc.edifice_model = true;
    p.integrate_drainage = true;
    p.tect.narrow_plate_base_blur = true;
    p.passes.glacial = true;
    p.geology_model = true;
    p.geology_processes = true;
    p.geology_karst = true;
    p
}

/// Bit view, for exact equality (NaN included).
fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// The disc of radius [`R`] around `(cx, cy)`, as world indices.
fn disc(cx: usize, cy: usize) -> Vec<usize> {
    let mut out = Vec::new();
    for dy in -R..=R {
        for dx in -R..=R {
            if dx * dx + dy * dy <= R * R {
                out.push((cy as i64 + dy) as usize * GW + (cx as i64 + dx) as usize);
            }
        }
    }
    out
}

/// A cell whose disc of radius `R + 4` is all land and all granite at the
/// surface (exposed and as `rock_top`), nearest the map centre: B11's
/// "granite interior". `None` if the seed has none.
fn granite_interior(ws: &WorldState) -> Option<(usize, usize)> {
    let col = ws.geology.column()?;
    let sea = ws.sea_level as f32;
    let rr = R + 4;
    let mut best: Option<((usize, usize), i64)> = None;
    for cy in (rr as usize)..(GH - rr as usize) {
        for cx in (rr as usize)..(GW - rr as usize) {
            let ok = (-rr..=rr).all(|dy| {
                (-rr..=rr).all(|dx| {
                    if dx * dx + dy * dy > rr * rr {
                        return true;
                    }
                    let i = (cy as i64 + dy) as usize * GW + (cx as i64 + dx) as usize;
                    ws.field[i] > sea && col.rock_top[i] == Rock::Granite as u8 && col.substrate(i).is_none()
                })
            });
            if ok {
                let d = (cx as i64 - GW as i64 / 2).pow(2) + (cy as i64 - GH as i64 / 2).pow(2);
                if best.is_none_or(|(_, bd)| d < bd) {
                    best = Some(((cx, cy), d));
                }
            }
        }
    }
    best.map(|(c, _)| c)
}

/// `top` paint of `value` over `cells`, zero elsewhere.
fn top_paint(cells: &[usize], value: impl Fn(usize) -> u8) -> Vec<u8> {
    let mut v = vec![0u8; GW * GH];
    for &i in cells {
        v[i] = value(i);
    }
    v
}

/// One commit of a surface-rock layer, nothing else painted.
fn commit_top(ws: &mut WorldState, p: &WorldParams, top: &[u8]) -> CommitReport {
    let zero = vec![0u8; GW * GH];
    commit(ws, p, GW, GH, &RockPaint { top, sub: &zero, cap: &zero }, REERODE_PAD_CELLS).expect("the world has a column")
}

/// The seeds B11 runs on: two of GF-0's five (`gf0_geology_harness.rs`), the
/// pair the identity hashes in this milestone use.
const SEEDS: [i32; 2] = [12345, 483920];

#[test]
fn b11_painting_the_rock_already_there_moves_nothing() {
    // Protects: B11's exact half. Field 0 bytes; column 0 bytes apart from
    // `painted`, which now flags exactly the painted cells.
    for seed in SEEDS {
        let p = params(seed);
        let ws0 = generate_terrain(&p);
        let (cx, cy) = granite_interior(&ws0).unwrap_or_else(|| panic!("seed {seed}: no granite interior"));
        let cells = disc(cx, cy);
        let col0 = ws0.geology.column().unwrap().clone();
        let mut ws = generate_terrain(&p); // `WorldState` is not `Clone`; generation is deterministic
        let top = top_paint(&cells, |i| col0.rock_top[i] + 1);
        let rep = commit_top(&mut ws, &p, &top);
        assert_eq!(rep.bake.cells_baked, cells.len(), "seed {seed}");
        assert_eq!(rep.bake.cells_changed, 0, "seed {seed}");
        let re = rep.reerode.as_ref().expect("processes on: the chain runs");
        assert!(re.stages.len() >= 2, "seed {seed}: the chain ran {:?}", re.stages);
        assert_eq!(re.cells_moved, 0, "seed {seed}");
        assert_eq!(bits(&ws.field), bits(&ws0.field), "seed {seed}: field");
        let col = ws.geology.column().unwrap();
        assert_eq!(col.rock_top, col0.rock_top, "seed {seed}: rock_top");
        assert_eq!(col.rock_sub, col0.rock_sub, "seed {seed}: rock_sub");
        assert_eq!(bits(&col.contact), bits(&col0.contact), "seed {seed}: contact");
        assert_eq!(bits(&col.regolith), bits(&col0.regolith), "seed {seed}: regolith");
        let painted = col.painted.as_ref().expect("a bake writes the mask");
        assert_eq!(painted.iter().filter(|&&v| v == 1).count(), cells.len(), "seed {seed}: mask");
        assert!(cells.iter().all(|&i| painted[i] == 1));
    }
}

#[test]
fn b11_shale_into_granite_lowers_the_patch_locally() {
    // Protects: B11's local half -- the response, its window and its ring.
    for seed in SEEDS {
        let p = params(seed);
        let ws0 = generate_terrain(&p);
        let (cx, cy) = granite_interior(&ws0).unwrap_or_else(|| panic!("seed {seed}: no granite interior"));
        let cells = disc(cx, cy);
        let top = top_paint(&cells, |_| Rock::Shale as u8 + 1);
        let mut ws = generate_terrain(&p); // `WorldState` is not `Clone`; generation is deterministic
        let t = std::time::Instant::now();
        let rep = commit_top(&mut ws, &p, &top);
        let ms = t.elapsed().as_secs_f64() * 1e3;
        let re = rep.reerode.as_ref().expect("processes on: the chain runs");
        let w: Window = re.window;
        let inside = |i: usize| {
            let (x, y) = (i % GW, i / GW);
            x >= w.x0 && x < w.x0 + w.w && y >= w.y0 && y < w.y0 + w.h
        };
        // No cell outside W moves.
        let outside_moved = (0..GW * GH).filter(|&i| !inside(i) && ws.field[i].to_bits() != ws0.field[i].to_bits()).count();
        assert_eq!(outside_moved, 0, "seed {seed}: cells outside W moved");
        // Painted cells lowered more, on average, than W's unpainted cells.
        let painted: std::collections::HashSet<usize> = cells.iter().copied().collect();
        let (mut sp, mut np, mut su, mut nu) = (0.0f64, 0usize, 0.0f64, 0usize);
        for y in w.y0..w.y0 + w.h {
            for x in w.x0..w.x0 + w.w {
                let i = y * GW + x;
                let low = ws0.field[i] as f64 - ws.field[i] as f64;
                if painted.contains(&i) {
                    sp += low;
                    np += 1;
                } else {
                    su += low;
                    nu += 1;
                }
            }
        }
        let (mp, mu) = (sp / np as f64, su / nu as f64);
        let lip = re.ring_step / re.relief;
        println!(
            "seed {seed}: W {}x{} at ({},{}), {} cells moved, painted mean lowering {mp:.3e}, unpainted {mu:.3e}, \
             ring step {:.3e} = {:.4}% of relief {:.4}, commit {ms:.1} ms, stages {:?}",
            w.w, w.h, w.x0, w.y0, re.cells_moved, re.ring_step, lip * 100.0, re.relief, re.stages
        );
        assert!(re.cells_moved > 0, "seed {seed}: shale into granite moved nothing");
        assert!(mp > mu, "seed {seed}: painted {mp:e} did not lower more than unpainted {mu:e}");
        assert!(mp > 0.0, "seed {seed}: the painted patch did not lower on average ({mp:e})");
        assert!(lip < 0.02, "seed {seed}: ring step {:.4}% of relief, bar 2%", lip * 100.0);
    }
}

#[test]
fn a_rock_paint_commit_is_deterministic() {
    // Protects: the commit is a pure function of the world and the paint.
    let p = params(SEEDS[0]);
    let ws0 = generate_terrain(&p);
    let (cx, cy) = granite_interior(&ws0).expect("granite interior");
    let top = top_paint(&disc(cx, cy), |_| Rock::Shale as u8 + 1);
    let (mut a, mut b) = (generate_terrain(&p), generate_terrain(&p));
    commit_top(&mut a, &p, &top);
    commit_top(&mut b, &p, &top);
    assert_eq!(bits(&a.field), bits(&b.field));
    let (ca, cb) = (a.geology.column().unwrap(), b.geology.column().unwrap());
    assert_eq!(bits(&ca.regolith), bits(&cb.regolith));
    assert_eq!(bits(&ca.contact), bits(&cb.contact));
}

#[test]
fn with_the_processes_off_the_bake_stands_and_nothing_erodes() {
    // Protects: the gate -- erosion does not read rock with the processes
    // off, so the commit bakes the column and leaves the field alone, and
    // says why.
    let mut p = params(SEEDS[0]);
    p.geology_processes = false;
    p.geology_karst = false;
    let ws0 = generate_terrain(&p);
    let (cx, cy) = granite_interior(&ws0).expect("granite interior");
    let cells = disc(cx, cy);
    let mut ws = generate_terrain(&p); // `WorldState` is not `Clone`; generation is deterministic
    let rep = commit_top(&mut ws, &p, &top_paint(&cells, |_| Rock::Shale as u8 + 1));
    assert_eq!(rep.reerode.unwrap_err(), cartalith_engine::rock_paint::ReerodeSkip::ProcessesOff);
    assert_eq!(bits(&ws.field), bits(&ws0.field));
    assert!(cells.iter().all(|&i| ws.geology.column().unwrap().rock_top[i] == Rock::Shale as u8));
}

#[test]
fn a_world_without_a_column_refuses() {
    // Protects: §2.6's reader rule -- no column, no rock paint, nothing
    // written, and the reason is the column's own.
    let mut p = params(SEEDS[0]);
    p.geology_model = false;
    let mut ws = generate_terrain(&p);
    let field0 = bits(&ws.field);
    let top = vec![1u8; GW * GH];
    let zero = vec![0u8; GW * GH];
    let e = commit(&mut ws, &p, GW, GH, &RockPaint { top: &top, sub: &zero, cap: &zero }, REERODE_PAD_CELLS).unwrap_err();
    assert_eq!(e.reason(), "no geology column: geology model off for this world");
    assert_eq!(bits(&ws.field), field0);
}
