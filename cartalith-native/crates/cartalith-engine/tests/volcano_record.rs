//! VA-1 of `VOLCANO_APPEARANCE_RESEARCH.md` (owner Ruling CW, 2026-10-24):
//! the per-volcano record `generate_terrain` now keeps
//! (`WorldState::volcanoes`).
//!
//! What these tests protect:
//! - **Recording only.** The table is written from values the stamp already
//!   had and nothing in generation reads it, so the height field and
//!   `volcanic_field` are bit-identical with the record (geology model on)
//!   and without it (geology model off) -- the ruling's "without changing the
//!   height field".
//! - **The record is the stamp's.** Re-evaluating the stamp's own owner rule
//!   from the table alone (`volcano_attr::owner_of`: the largest
//!   `(1 - t)(1 - age)`) reproduces `volcanic_field` exactly at every cell, so
//!   every recorded centre, radius and age is the one the stamp drew.
//! - **Present when it should be, absent otherwise**: a table with a kind on
//!   every edifice under the morphological model; none with the geology model
//!   off (the trace that records it only runs with the column).

use cartalith_engine::{generate_terrain, WorldParams};
use cartalith_terrain::volcano_attr;

const GW: usize = 256;
const GH: usize = 164;

/// The app boundary's volcano-relevant divergences (`cartalith-godot`'s
/// `params::defaults`, which this crate cannot import), plus the geology
/// model the app runs with.
fn params(seed: i32, geology: bool) -> WorldParams {
    let mut p = WorldParams::defaults(GW, GH, seed);
    p.volc.exclude_transform = true;
    p.volc.edifice_model = true;
    p.crater.physical_model = true;
    p.geology_model = geology;
    p
}

fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// Protects "recording only": the field and `volcanic_field` do not move
/// when the record is kept, on three seeds.
#[test]
fn recording_the_table_moves_no_generated_byte() {
    for seed in [24601, 314159, 71077345] {
        let on = generate_terrain(&params(seed, true));
        let off = generate_terrain(&params(seed, false));
        assert!(on.volcanoes.is_some(), "seed {seed}: the app-like world must keep a table");
        assert!(off.volcanoes.is_none(), "seed {seed}: no table without the geology model");
        assert_eq!(bits(&on.field), bits(&off.field), "seed {seed}: field");
        assert_eq!(bits(&on.volcanic_field), bits(&off.volcanic_field), "seed {seed}: volcanic_field");
    }
}

/// Protects that the table is the stamp's own: the owner rule evaluated from
/// the recorded centres, radii and ages alone reproduces `volcanic_field` at
/// every cell (rounded to `f32` as the stamp stores it), and every edifice
/// carries a kind and an age inside the draw's range.
#[test]
fn the_table_reproduces_volcanic_field_exactly() {
    for seed in [24601, 314159] {
        let ws = generate_terrain(&params(seed, true));
        let v = ws.volcanoes.as_ref().expect("a table");
        assert!(!v.is_empty());
        for e in v.iter() {
            assert!(e.kind.is_some(), "seed {seed}: the morphological model records a kind");
            assert!((0.0..=1.0).contains(&e.age), "seed {seed}: age {}", e.age);
            assert!(e.r >= 2.0, "seed {seed}: radius {} under the stamp's 2-cell floor", e.r);
        }
        let mut reached = 0usize;
        for y in 0..GH {
            for x in 0..GW {
                let want = ws.volcanic_field[y * GW + x];
                let got = volcano_attr::owner_of(v, x as f64, y as f64).map_or(0.0, |h| ((1.0 - h.t) * (1.0 - h.edifice.age)) as f32);
                assert_eq!(got.to_bits(), want.to_bits(), "seed {seed}: cell ({x}, {y}) {got} vs {want}");
                reached += usize::from(want > 0.0);
            }
        }
        assert!(reached > 200, "seed {seed}: only {reached} volcanic cells; the test reaches too little");
    }
}

/// Protects owner Ruling CX item 2 on real worlds: every recorded edifice
/// carries its years under the re-spread map (`years == respread(age)`, so the
/// draw itself, which shapes the stamp, is what it always was: arc and rift
/// draws stay `rng * 0.40` at the default slider), and the arc and rift
/// volcanoes of three worlds together hold all three states -- the mix the
/// ruling asked for, where Ruling CW's map left none extinct. Printed per
/// seed; the mix is asserted over the three together, a property of the
/// draw's spread rather than of one seed.
#[test]
fn the_respread_ages_give_arc_and_rift_volcanoes_every_state() {
    let mut arc_rift = [0usize; 3];
    for seed in [24601, 314159, 71077345] {
        let ws = generate_terrain(&params(seed, true));
        let v = ws.volcanoes.as_ref().expect("a table");
        let mut counts = std::collections::BTreeMap::new();
        for e in v.iter() {
            assert_eq!(e.years, Some(volcano_attr::respread_age_years(e.age)), "seed {seed}: a row without its re-spread years");
            let a = volcano_attr::activity(e);
            *counts.entry((volcano_attr::setting_key(e.setting), a.key())).or_insert(0) += 1;
            if matches!(e.setting, cartalith_terrain::VolcanicSetting::Arc | cartalith_terrain::VolcanicSetting::Rift) {
                assert!(e.age < 0.40, "seed {seed}: an arc/rift age {} above the slider", e.age);
                arc_rift[a as usize] += 1;
            }
        }
        println!("seed {seed}: {counts:?}");
    }
    println!("arc/rift active/dormant/extinct: {arc_rift:?}");
    assert!(arc_rift.iter().all(|&n| n > 0), "arc/rift states over three worlds: {arc_rift:?}");
}
