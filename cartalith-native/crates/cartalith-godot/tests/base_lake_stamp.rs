//! `RenderCtx::with_lakes` -- the reference's v0.103 base-map lake stamp
//! (8460/8580): an above-sea lake cell is drawn as flat freshwater by
//! `cell_color`, land is untouched, and a context without the attachment
//! draws exactly what it drew before.
//!
//! `#[path]`-includes `render.rs` for the reason `nonsquare.rs` gives:
//! `cartalith-godot` is `cdylib`-only.
#![allow(dead_code)]

#[path = "../src/render.rs"]
mod render;

const GW: usize = 24;
const GH: usize = 24;
const SEA: f64 = 0.42;

/// A closed bowl above sea level: a 0.70 plateau with a 0.50 floor in the
/// middle, so the priority flood pools it into a lake (class 2).
fn bowl() -> Vec<f32> {
    let mut f = vec![0.70f32; GW * GH];
    for y in 8..16 {
        for x in 8..16 {
            f[y * GW + x] = 0.50;
        }
    }
    f
}

#[test]
fn an_above_sea_lake_is_drawn_as_water_and_land_is_not_touched() {
    let field = bowl();
    let temp = vec![14.0f32; GW * GH];
    let rain = vec![1.0f32; GW * GH];
    let lakes = cartalith_civ::build_water_bodies(&field, GW, GH, SEA, false, Some(&rain)).classification;
    let centre = 12 * GW + 12;
    assert_eq!(lakes[centre], 2, "the fixture must actually contain a lake, or the test is vacuous");
    assert_eq!(lakes[2 * GW + 2], 0, "the plateau corner must be land");

    let a = render::TerrainAppearance::default();
    let plain = render::RenderCtx::with_appearance(&field, &temp, &rain, None, GW, GH, SEA, false, 55.0, 5.0, a.clone());
    let stamped = render::RenderCtx::with_appearance(&field, &temp, &rain, None, GW, GH, SEA, false, 55.0, 5.0, a).with_lakes(&lakes);

    let (r0, g0, b0) = render::cell_color(&plain, 12, 12);
    let (r1, g1, b1) = render::cell_color(&stamped, 12, 12);
    assert!((r0, g0, b0) != (r1, g1, b1), "the lake cell must change once the classification is attached");
    assert!(b1 > r1, "an above-sea lake reads as water (blue over red): got ({r1}, {g1}, {b1})");
    assert_eq!(render::cell_color(&plain, 2, 2), render::cell_color(&stamped, 2, 2), "land must be byte-identical");
}

#[test]
fn a_classification_of_the_wrong_size_is_refused() {
    let field = bowl();
    let temp = vec![14.0f32; GW * GH];
    let rain = vec![1.0f32; GW * GH];
    let a = render::TerrainAppearance::default();
    let plain = render::RenderCtx::with_appearance(&field, &temp, &rain, None, GW, GH, SEA, false, 55.0, 5.0, a.clone());
    let short = vec![2u8; 10];
    let refused = render::RenderCtx::with_appearance(&field, &temp, &rain, None, GW, GH, SEA, false, 55.0, 5.0, a).with_lakes(&short);
    assert_eq!(render::cell_color(&plain, 12, 12), render::cell_color(&refused, 12, 12));
}
