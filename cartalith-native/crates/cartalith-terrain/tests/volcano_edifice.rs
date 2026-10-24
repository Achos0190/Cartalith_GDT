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

// Fixture grid width -- large enough that the boundary-mask stride below
// (every 5th/7th cell) still stamps enough volcanoes to be statistically
// meaningful.
const GW: usize = 48;
// Fixture grid height, paired with GW.
const GH: usize = 40;
const N: usize = GW * GH;

/// Runs `stamp_volcanoes_simple_shaped` once with the given edifice model
/// over a fixed synthetic boundary mask, returning the resulting height and
/// volcanic-age fields for comparison between models.
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

/// Runs `stamp_volcanoes_provinces_shaped` once with the given edifice
/// model over a fixed synthetic plate/stress setup, returning the
/// resulting height and volcanic-age fields for comparison between models.
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

/// Protects: `EdificeModel::default()` staying `Reference`. Anything else
/// silently re-baselines every caller that does not name a model.
#[test]
fn the_default_edifice_model_is_the_reference() {
    assert_eq!(EdificeModel::default(), EdificeModel::Reference);
}

/// Protects: turning on EdificeModel::Morphological not consuming any extra RNG draws relative to
/// Reference (volcanic_field, which depends only on placement/radius/age, must stay bit-identical)
/// while the height field itself actually differs and stays within [0,1].
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

/// Protects: a seeded world rendering identically every time under the new
/// model too -- the noise terms are keyed on the edifice centre, not on
/// iteration order.
#[test]
fn morphological_is_deterministic() {
    assert_eq!(provinces(EdificeModel::Morphological), provinces(EdificeModel::Morphological));
    assert_eq!(simple(EdificeModel::Morphological), simple(EdificeModel::Morphological));
}

/// Protects: the un-suffixed entry points -- the ones every golden calls --
/// still meaning the reference. This is the guard against a future edit
/// "helpfully" defaulting them to the new model.
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

/// The traced provinces run, for VA-5's comparisons: the same fixture as
/// [`provinces`], with the edifice table kept.
fn provinces_traced(model: EdificeModel) -> (Vec<f32>, cartalith_terrain::VolcanoTrace) {
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
    let mut trace = cartalith_terrain::VolcanoTrace::new(N);
    cartalith_terrain::stamp_volcanoes_provinces_traced(
        GW, GH, 12345, 800.0, 4000.0, &boundary_mask, &stress_field, None, &plate_id, &plates, 24, 0.4, &mut field, &mut volcanic_field, model, Some(&mut trace), false,
    );
    (field, trace)
}

/// Protects VA-5 (`EdificeModel::Morphometric`, Ruling CW): no RNG draw
/// moves (every edifice's position, setting, kind and age equal the
/// morphological model's); a scoria cone's radius is its standing height
/// over twice the cited fresh H/W of 0.179 (a literal), floored at the
/// stamp's 2 cells; most stratocones get a small summit crater recorded on
/// the edifice; and the height field moves.
#[test]
fn morphometric_resizes_cinders_and_splits_strato_craters_without_moving_the_stream() {
    let m_per_unit = 4000.0 / (1.0 - 0.42);
    let (f_morph, t_morph) = provinces_traced(EdificeModel::Morphological);
    let (f_va5, t_va5) = provinces_traced(EdificeModel::Morphometric { m_per_unit, cinder_grid_law: false });
    assert_eq!(t_morph.edifices.len(), t_va5.edifices.len());
    assert!(t_va5.edifices.len() >= 10, "the fixture places too few edifices");
    let cell_km = 800.0 / GW as f64;
    let (mut cinders, mut craters, mut stratos) = (0, 0, 0);
    for (a, b) in t_morph.edifices.iter().zip(&t_va5.edifices) {
        assert_eq!((a.cx, a.cy, a.age, a.setting, a.kind), (b.cx, b.cy, b.age, b.setting, b.kind), "a draw moved");
        match b.kind {
            Some(cartalith_terrain::EdificeKind::Cinder) => {
                cinders += 1;
                // The stamp's `h` is age-damped: undo it to get the drawn
                // standing height, then the cited H/W.
                let stand_m = b.h / (1.0 - b.age * 0.5) * m_per_unit;
                let want_cells = (stand_m / 1000.0 / (2.0 * 0.179) / cell_km).max(2.0);
                assert!((b.r - want_cells).abs() < 1e-6, "cinder radius {} not {want_cells}", b.r);
                assert!(b.crater.is_none());
            }
            Some(cartalith_terrain::EdificeKind::Strato) => {
                stratos += 1;
                if let Some((floor, rim)) = b.crater {
                    craters += 1;
                    assert!(rim <= 0.24 && floor < rim, "crater ({floor}, {rim})");
                }
                assert!(a.crater.is_none(), "the morphological model records no crater");
            }
            _ => assert!(b.crater.is_none()),
        }
    }
    assert!(cinders > 0, "the fixture has no cinder cone");
    if stratos >= 4 {
        assert!(craters > 0, "every stratocone kept its caldera");
    }
    assert_ne!(f_morph, f_va5, "the morphometry moved nothing");
}

/// Protects VA-5's cinder radius law on a grid fine enough to resolve it
/// (40 km over 200 cells, 0.2 km per cell; the feature clamp is 0.12 of 160
/// = 19.2 cells): every scoria cone between the stamp's 2-cell floor and that
/// clamp has exactly its standing height over twice the cited fresh H/W of
/// 0.179 (a literal, so re-pointing the constant fails here), with the
/// stamp's own `0.9 * m_per_unit / peak_m` normalisation undone through the
/// recorded `h`.
#[test]
fn morphometric_cinder_radius_is_the_cited_aspect_ratio() {
    let (gw, gh) = (200usize, 160usize);
    let n = gw * gh;
    let boundary_mask: Vec<u8> = (0..n).map(|i| u8::from(i % 7 == 0)).collect();
    let m_per_unit = 4000.0 / (1.0 - 0.42);
    let mut field = vec![0.3f32; n];
    let mut volcanic_field = vec![0.0f32; n];
    let mut trace = cartalith_terrain::VolcanoTrace::new(n);
    cartalith_terrain::stamp_volcanoes_simple_traced(
        gw, gh, 4242, 40.0, 4000.0, &boundary_mask, 40, 0.4, &mut field, &mut volcanic_field,
        EdificeModel::Morphometric { m_per_unit, cinder_grid_law: false }, Some(&mut trace), false,
    );
    let cell_km = 40.0 / gw as f64;
    let mut exact = 0;
    for e in &trace.edifices {
        if e.kind != Some(cartalith_terrain::EdificeKind::Cinder) {
            continue;
        }
        // `h = h_m / peak * 0.9 * (1 - age/2)`, so the standing metres are
        // `h / (1 - age/2) * m_per_unit`.
        let stand_m = e.h / (1.0 - e.age * 0.5) * m_per_unit;
        let want = stand_m / 1000.0 / (2.0 * 0.179) / cell_km;
        if want > 2.0 && want < 19.2 {
            assert!((e.r - want).abs() < 1e-6, "cinder radius {} cells, want {want}", e.r);
            exact += 1;
        }
    }
    assert!(exact >= 5, "only {exact} cinder cones inside the law's range; the test reaches too little");
}

/// Protects owner Ruling CX item 4's grid-aware cinder law on the same
/// fixture: with it on, every scoria cone's radius is `max(0.45 * standing
/// metres / (2 * 0.179), 2.5 cells)` (literals: the measured kept share and
/// floor, and the cited fresh H/W), the stamp's own 2-cell floor and the
/// feature clamp aside; no draw moves (the same edifices, positions, kinds
/// and ages as with the law off); and the floor is reached by at least one
/// cone, so both branches of the `max` are exercised.
#[test]
fn the_grid_aware_cinder_law_sizes_to_the_kept_height_with_a_floor() {
    let (gw, gh) = (200usize, 160usize);
    let n = gw * gh;
    let boundary_mask: Vec<u8> = (0..n).map(|i| u8::from(i % 7 == 0)).collect();
    let m_per_unit = 4000.0 / (1.0 - 0.42);
    let run = |law: bool| {
        let mut field = vec![0.3f32; n];
        let mut volcanic_field = vec![0.0f32; n];
        let mut trace = cartalith_terrain::VolcanoTrace::new(n);
        cartalith_terrain::stamp_volcanoes_simple_traced(
            gw, gh, 4242, 40.0, 4000.0, &boundary_mask, 40, 0.4, &mut field, &mut volcanic_field,
            EdificeModel::Morphometric { m_per_unit, cinder_grid_law: law }, Some(&mut trace), false,
        );
        trace.edifices
    };
    let (off, on) = (run(false), run(true));
    assert_eq!(off.len(), on.len());
    let cell_km = 40.0 / gw as f64;
    let (mut sized, mut floored) = (0, 0);
    for (a, b) in off.iter().zip(on.iter()) {
        assert_eq!((a.cx, a.cy, a.kind, a.age, a.setting), (b.cx, b.cy, b.kind, b.age, b.setting), "the law moved a draw");
        if b.kind != Some(cartalith_terrain::EdificeKind::Cinder) {
            assert_eq!(a.r, b.r, "the law reached a non-cinder edifice");
            continue;
        }
        let stand_m = b.h / (1.0 - b.age * 0.5) * m_per_unit;
        let law_cells = 0.45 * stand_m / 1000.0 / (2.0 * 0.179) / cell_km;
        let want = law_cells.max(2.5);
        if want < 19.2 {
            assert!((b.r - want).abs() < 1e-6, "cinder radius {} cells, want {want}", b.r);
            if law_cells < 2.5 {
                floored += 1;
            } else {
                sized += 1;
            }
        }
    }
    assert!(sized >= 3 && floored >= 1, "the fixture reaches the law {sized} times and the floor {floored} times");
}
