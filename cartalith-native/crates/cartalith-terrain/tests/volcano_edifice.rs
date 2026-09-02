//! End-to-end behaviour of `EdificeModel::Morphological` (Lane 3: the volcanic
//! edifice).
//!
//! **Not a golden-parity test.** The morphological model is a deliberate
//! divergence from `stampOneVolcano`, gated and default-off; parity for the
//! reference path is pinned by `golden_parity_volc_craters.rs` and
//! `golden_parity_volc_provinces.rs`, whose call sites deliberately still use
//! the un-suffixed entry points and are untouched by this work.
//!
//! What is asserted here is the invariant that makes the divergence safe to
//! carry: **turning the model on must not move the random stream by one draw.**
//! `volcanic_field` is `max((1-t)*(1-age))`, a function of placement and radius
//! alone — it never sees the profile — so it is bit-identical between the two
//! models if and only if every placement, every size class and every age draw
//! came out the same. A single extra `rng()` call anywhere in the edifice code
//! would break it loudly. The height field, meanwhile, must actually differ, or
//! the model is a no-op.

use cartalith_terrain::{
    stamp_volcanoes_provinces_shaped, stamp_volcanoes_simple_shaped, EdificeModel, Plate,
};

const GW: usize = 48;
const GH: usize = 40;
const N: usize = GW * GH;

fn simple(model: EdificeModel) -> (Vec<f32>, Vec<f32>) {
    let boundary_mask: Vec<u8> = (0..N).map(|i| u8::from(i % 7 == 0)).collect();
    let mut field = vec![0.3f32; N];
    let mut volcanic_field = vec![0.0f32; N];
    stamp_volcanoes_simple_shaped(
        GW,
        GH,
        4242,
        800.0,
        4000.0,
        &boundary_mask,
        24,
        0.4,
        &mut field,
        &mut volcanic_field,
        model,
    );
    (field, volcanic_field)
}

fn provinces(model: EdificeModel) -> (Vec<f32>, Vec<f32>) {
    let boundary_mask: Vec<u8> = (0..N).map(|i| u8::from(i % 5 == 0)).collect();
    let stress_field: Vec<f32> = (0..N).map(|i| if i % 2 == 0 { 0.3 } else { -0.3 }).collect();
    let plate_id: Vec<u16> = (0..N).map(|i| (i % 4) as u16).collect();
    let plates = vec![
        Plate { x: 2.0, y: 2.0, vx: 1.0, vy: 0.2, base: 0.1 },
        Plate { x: 10.0, y: 4.0, vx: -0.5, vy: 0.8, base: -0.2 },
        Plate { x: 5.0, y: 10.0, vx: 0.3, vy: -0.6, base: 0.05 },
        Plate { x: 15.0, y: 12.0, vx: -0.2, vy: -0.4, base: -0.1 },
    ];
    let mut field = vec![0.3f32; N];
    let mut volcanic_field = vec![0.0f32; N];
    stamp_volcanoes_provinces_shaped(
        GW,
        GH,
        12345,
        800.0,
        4000.0,
        &boundary_mask,
        &stress_field,
        None,
        &plate_id,
        &plates,
        24,
        0.4,
        &mut field,
        &mut volcanic_field,
        model,
    );
    (field, volcanic_field)
}

/// The default must be the reference. Anything else silently re-baselines every
/// caller that does not name a model.
#[test]
fn the_default_edifice_model_is_the_reference() {
    assert_eq!(EdificeModel::default(), EdificeModel::Reference);
}

#[test]
fn morphological_does_not_move_the_random_stream() {
    for (name, run) in [
        ("simple", simple as fn(EdificeModel) -> (Vec<f32>, Vec<f32>)),
        ("provinces", provinces as fn(EdificeModel) -> (Vec<f32>, Vec<f32>)),
    ] {
        let (ref_field, ref_volc) = run(EdificeModel::Reference);
        let (morph_field, morph_volc) = run(EdificeModel::Morphological);

        assert!(ref_volc.iter().any(|&v| v > 0.0), "{name}: nothing was stamped at all");
        assert_eq!(
            ref_volc, morph_volc,
            "{name}: volcanic_field must be bit-identical -- the edifice model consumed an RNG draw"
        );

        let moved = ref_field
            .iter()
            .zip(&morph_field)
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            moved > 40,
            "{name}: only {moved} cells differ; the morphological model is doing nothing"
        );
        assert!(
            morph_field.iter().all(|&v| (0.0..=1.0).contains(&v)),
            "{name}: field must stay within [0,1]"
        );
    }
}

/// A seeded world must render identically every time under the new model too —
/// the noise terms are keyed on the edifice centre, not on iteration order.
#[test]
fn morphological_is_deterministic() {
    assert_eq!(provinces(EdificeModel::Morphological), provinces(EdificeModel::Morphological));
    assert_eq!(simple(EdificeModel::Morphological), simple(EdificeModel::Morphological));
}

/// The un-suffixed entry points -- the ones every golden calls -- must still
/// mean the reference. This is the guard against a future edit "helpfully"
/// defaulting them to the new model.
#[test]
fn the_unsuffixed_entry_points_still_stamp_the_reference() {
    let boundary_mask: Vec<u8> = (0..N).map(|i| u8::from(i % 7 == 0)).collect();
    let mut field = vec![0.3f32; N];
    let mut volcanic_field = vec![0.0f32; N];
    cartalith_terrain::stamp_volcanoes_simple(
        GW,
        GH,
        4242,
        800.0,
        4000.0,
        &boundary_mask,
        24,
        0.4,
        &mut field,
        &mut volcanic_field,
    );
    assert_eq!((field, volcanic_field), simple(EdificeModel::Reference));
}
