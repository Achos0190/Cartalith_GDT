//! `render::render_biome_tile_rgba` — the LOD/atlas biome tile
//! (`LOD_DETAIL_SCOPE.md` **LOD-D1**, Ruling K step 1a; reference HTML
//! `Cartalith Gen1 v2.11.html` `renderBiomeTileRGBA`, lines 11668-11779).
//!
//! The scope asks for three checks, and this file runs all three plus the
//! budget line. **Two of the three cannot both be exact**, which is the single
//! most important thing in this file and is stated here rather than discovered
//! at a red test:
//!
//! - **Check 1, against the reference.** `golden_*` below, against
//!   `tests/fixtures/tile_biome_fixture.rs`, captured by
//!   `cartalith-native/tools/tile_biome_capture.js` from the frozen v2.11
//!   engine running headlessly under Node.
//! - **Check 2, screen identity.** `screen_identity_*` below. `cell_color` and
//!   `renderBiomeTileRGBA` **disagree by construction** in the reference
//!   itself: the main map's `shadeFactor2` (7681) divides the exaggeration by
//!   its sample step and the tile's meso block (11742) does not, so a tile's
//!   meso gradient is about `ms` times the map's at the same ground scale.
//!   `LOD_DETAIL_SCOPE.md`'s owner question 1 assumes *"both"*; both is not
//!   available, and this file measures the gap instead of asserting an
//!   equality that cannot hold. The identity is therefore run in a
//!   configuration that **holds the shading term equal on both sides**
//!   (`exag = 0`, where every normal is `(0, 0, 1)` whatever the step is), so
//!   what it proves is that the *other* thirteen sampled fields, the material
//!   path and the whole port-only tail are the same function on both paths.
//!   A second test then measures the real-`exag` residual and attributes it.
//! - **Check 3, seams.** `adjacent_tiles_*` below, the reference's own v1.29
//!   method: the shared-column difference against the interior
//!   column-to-column spread.
//!
//! **Emptiness is asserted, not assumed** (`CLAUDE.md`'s "watch for silently
//! empty golden output"): the capture script refuses to emit a fixture with
//! fewer than 16 distinct colours, with a non-opaque pixel, or with either
//! colour branch unreached, and it refuses to emit two cases that came back
//! byte-identical. Those gates ran; their numbers are in the fixture's own
//! header comment.

#[path = "../src/render.rs"]
mod render;

#[path = "fixtures/tile_biome_fixture.rs"]
mod fx;

use render::{RenderCtx, TerrainAppearance, TileBounds, TileFields};

fn bounds_of(b: [f64; 4]) -> TileBounds {
    TileBounds { x: b[0], y: b[1], w: b[2], h: b[3] }
}

/// The appearance the reference itself renders at: every enhancement slider
/// zero, `state.viz` untouched. `js_reference()` plus the three scalars the
/// capture read straight off `state`, so the two sides cannot disagree about
/// the sun, the exaggeration or the grey blend.
fn reference_appearance() -> TerrainAppearance {
    TerrainAppearance {
        exag: fx::EXAG,
        sun_az_deg: fx::SUN_AZ_DEG,
        bio_blend: fx::BIO_BLEND,
        ..TerrainAppearance::js_reference()
    }
}

fn ctx_for(a: TerrainAppearance) -> RenderCtx<'static> {
    RenderCtx::with_appearance(&fx::FIELD, &fx::TEMPERATURE, &fx::RAINFALL, Some(&fx::FLOW), fx::GW, fx::GH, fx::SEA_LEVEL, false, fx::LAT_N, fx::LAT_S, a)
}

/// Per-channel deltas between two RGBA buffers, as `(worst, mean, n_over_1)`.
fn compare(got: &[u8], want: &[u8]) -> (i32, f64, usize) {
    assert_eq!(got.len(), want.len(), "buffer length");
    assert!(!got.is_empty(), "empty buffer -- the comparison would pass vacuously");
    let (mut worst, mut sum, mut over) = (0i32, 0f64, 0usize);
    for (g, w) in got.iter().zip(want.iter()) {
        let d = (*g as i32 - *w as i32).abs();
        worst = worst.max(d);
        sum += d as f64;
        if d > 1 {
            over += 1;
        }
    }
    (worst, sum / got.len() as f64, over)
}

// ===========================================================================
// Check 0 — the citation, and the one shared constant
// ===========================================================================

/// `MISTAKES.md`: *"Verify a scope document's line ranges against the real
/// reference before slicing"*, and *"Re-resolve a citation late in a long
/// pass"*. The capture script asserts this at capture time; this asserts it
/// at every `cargo test`, so a re-freeze of `reference/` cannot silently leave
/// this port's golden describing a function that has moved.
#[test]
fn the_reference_line_range_is_where_the_scope_says_it_is() {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../reference/Cartalith Gen1 v2.11.html");
    let src = std::fs::read_to_string(p).expect("the frozen v2.11 reference is readable");
    let lines: Vec<&str> = src.split('\n').map(|l| l.trim_end_matches('\r')).collect();
    assert!(lines.len() > 11779, "the reference is shorter than the cited range");
    assert_eq!(lines[11668 - 1], "function renderBiomeTileRGBA(tile,W,H,bounds){", "line 11668 is not renderBiomeTileRGBA's opening line");
    assert_eq!(lines[11779 - 1], "}", "line 11779 is not the function's closing brace");
}

/// The tile builds its **own** river SDF but with the **grid's** threshold
/// (reference 11683), so a tile and the map agree on which channels are
/// rivers. Asserted against the literal the reference itself computed for this
/// world rather than against the port's own function — `MISTAKES.md`: *"assert
/// a literal, or the independent thing the value must equal"*.
#[test]
fn the_river_threshold_matches_the_reference() {
    let ours = cartalith_hydrology::river_flow_thresh(fx::GW, fx::GH, fx::GW, fx::MAP_WIDTH_KM);
    assert!((ours - 0.04).abs() < 1e-12, "river_flow_thresh = {ours}, the reference computed 0.04");
    assert!((fx::RIVER_FLOW_THRESH - 0.04).abs() < 1e-12, "the fixture's captured threshold moved");
}

// ===========================================================================
// Check 1 — against the reference
// ===========================================================================

/// `LOD_DETAIL_SCOPE.md` LOD-D1 check 1: *"Against the reference
/// `renderBiomeTileRGBA` on a small fixture ... under `js_reference()`."*
///
/// The scope says *"to 1e-4 per channel"*. **It measured byte-identical** —
/// worst delta 0 over all 4 096 bytes — so that is what is asserted, on
/// `bake_raster.rs`'s own precedent in this crate (*"Bit-exact on this file's
/// fixture, not within a tolerance"*). The reference's output is already
/// quantized (`Uint8ClampedArray`, `ToUint8Clamp`, round-half-to-even), so a
/// `1e-4` float agreement would show at the byte as "identical except where
/// the two sides straddle a rounding boundary"; none do.
///
/// **If this ever goes red by one level, do not widen it.** A single-level
/// delta here means a libm divergence (`cartalith-rust-conventions`: *"V8's
/// libm is not Rust's"*) landing on a rounding boundary, and the thing to do
/// is find which call — `js_hypot`, `js_exp`, `js_log` — not to accept it.
/// `PARITY_TESTING.md`: *"A red test means re-read the JS, not widen the
/// tolerance."*
#[test]
fn golden_the_tile_matches_the_reference_at_default_viz() {
    let ctx = ctx_for(reference_appearance()).with_map_scale(fx::MAP_WIDTH_KM);
    let tf = TileFields::new(&ctx, None);
    let got = render::render_biome_tile_rgba(&ctx, &fx::REFERENCE_DEFAULTS_TILE, fx::TILE_W, fx::TILE_H, bounds_of(fx::REFERENCE_DEFAULTS_BOUNDS), &tf);
    assert_eq!(got.len(), fx::TILE_W * fx::TILE_H * 4, "the tile came back the wrong size (or empty, which would pass every comparison below)");
    let (worst, mean, over) = compare(&got, &fx::REFERENCE_DEFAULTS_EXPECTED_RGBA);
    println!("default viz: worst {worst}, mean {mean:.6}, bytes over 1 level: {over}");
    assert_eq!(over, 0, "{over} bytes differ by more than one quantization level");
    assert_eq!(worst, 0, "the tile is no longer byte-identical to the reference (mean {mean:.6}) -- read the doc comment above before touching this number");
}

/// The same check with the four stages the **per-tile prologue** builds turned
/// on — the crest field, the coast SDF, the river SDF and the biome-boundary
/// distance. Without this the golden would only ever exercise the material
/// path: every one of those four is `0.0` in `js_reference()`, so the test
/// above cannot tell whether the prologue is built at all.
///
/// `viz.ao` is deliberately **not** turned on here. See departure 1 in
/// `render.rs`'s tile section: this port's screen AO is a different algorithm
/// from the reference's, the tile samples the grid's, and a golden asserting
/// the reference's tile-local `aoMul` would be asserting a term this port's
/// own map does not have.
#[test]
fn golden_the_tile_matches_the_reference_with_the_prologue_stages_on() {
    let a = TerrainAppearance {
        crest_strength: 0.35,
        sdf_coast: 0.6,
        sdf_rivers: 0.5,
        sdf_biomes: 0.4,
        ..reference_appearance()
    };
    let ctx = ctx_for(a).with_map_scale(fx::MAP_WIDTH_KM);
    let tf = TileFields::new(&ctx, None);
    let got = render::render_biome_tile_rgba(&ctx, &fx::SDF_AND_CREST_ON_TILE, fx::TILE_W, fx::TILE_H, bounds_of(fx::SDF_AND_CREST_ON_BOUNDS), &tf);
    assert_eq!(got.len(), fx::TILE_W * fx::TILE_H * 4);
    let (worst, mean, over) = compare(&got, &fx::SDF_AND_CREST_ON_EXPECTED_RGBA);
    println!("prologue on: worst {worst}, mean {mean:.6}, bytes over 1 level: {over}");
    assert_eq!(over, 0, "{over} bytes differ by more than one quantization level");
    assert_eq!(worst, 0, "the tile is no longer byte-identical to the reference with the prologue stages on (mean {mean:.6})");
}

/// The negative control for both goldens above: the two fixture cases must not
/// be the same picture, or "it matches" would be one claim made twice.
///
/// It also pins that the prologue stages *reach the colour*: rendering case 2's
/// tile with the sliders at zero must differ from rendering it with them up.
#[test]
fn the_prologue_stages_actually_move_the_tile() {
    let off = ctx_for(reference_appearance()).with_map_scale(fx::MAP_WIDTH_KM);
    let tf_off = TileFields::new(&off, None);
    let plain = render::render_biome_tile_rgba(&off, &fx::SDF_AND_CREST_ON_TILE, fx::TILE_W, fx::TILE_H, bounds_of(fx::SDF_AND_CREST_ON_BOUNDS), &tf_off);
    assert!(!plain.is_empty());
    let n = plain.iter().zip(fx::SDF_AND_CREST_ON_EXPECTED_RGBA.iter()).filter(|(a, b)| a != b).count();
    assert!(n > 1000, "only {n} bytes move when the four prologue stages are switched on; they are not reaching the colour");
    println!("prologue stages move {n} of {} bytes", plain.len());
}

// ===========================================================================
// Check 2 — screen identity
// ===========================================================================

/// `build_color_texture`'s pipeline (`lib.rs`), reproduced here because that
/// function lives in the crate root and this target `#[path]`-includes
/// `render.rs` standalone. Everything except the river-ink tint, which the
/// tile takes through `TileFields::with_ink` and which is not attached on
/// either side here.
fn screen_rgb(ctx: &RenderCtx) -> Vec<u8> {
    let (gw, gh) = (fx::GW, fx::GH);
    let mut bytes = vec![0u8; gw * gh * 3];
    for y in 0..gh {
        for x in 0..gw {
            let (r, g, b) = render::cell_color(ctx, x, y);
            let o = (y * gw + x) * 3;
            bytes[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            bytes[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            bytes[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    bytes
}

fn screen_finished(ctx: &RenderCtx, a: &TerrainAppearance) -> Vec<u8> {
    let mut bytes = screen_rgb(ctx);
    let influence = render::build_grade_influence(ctx, fx::GW, fx::GH);
    render::finish_raster(a, &mut bytes, fx::GW, fx::GH, false, &influence, render::ColorSpace::Srgb);
    bytes
}

/// The tile at **one pixel per coarse cell**, over the whole grid, compared
/// against the finished screen raster.
///
/// Run at `exag = 0`, which is what makes the comparison meaningful rather
/// than merely favourable. With no vertical exaggeration every normal on both
/// sides is exactly `(0, 0, 1)` whatever sample step it was taken at, so the
/// **one term the reference defines differently for a tile** — the meso shade,
/// whose `/ s` normaliser `shadeFactor2` has and the tile does not — is held
/// equal, and what is left under test is every other term: the thirteen
/// sampled fields, the slope, TWI, aspect and curvature, the whole material
/// path, the crest, both SDF bands, the waves, the parchment, the plate frame,
/// local contrast, the grade and the colour space.
///
/// Lakes are switched off on the tile because the screen side here is built
/// without `RenderCtx::with_lakes`: the two draw a lake differently (the tile
/// with the v1.05 terrain-following shoreline, the screen with the v0.103
/// per-cell stamp, reference 8580), so leaving them on would measure that
/// shape difference rather than the terrain path this test is about. (This
/// note used to say the reference's main-map loop draws no lakes at all, and
/// cited a test `the_tile_draws_lakes_the_screen_does_not` that never existed;
/// both were wrong -- corrected 2026-09-23.)
///
/// What remains is the **quantization**: the tile stores through
/// `u8_clamped` (the reference's `ToUint8Clamp`, round-half-to-even) and
/// `build_color_texture` truncates, so the same colour can land one level
/// apart. That is the entire residual, and it is asserted as such.
#[test]
fn screen_identity_at_one_pixel_per_cell_with_the_shading_held_equal() {
    let a = TerrainAppearance { exag: 0.0, ..TerrainAppearance::default() };
    let ctx = ctx_for(a.clone()).with_map_scale(fx::MAP_WIDTH_KM);
    let screen = screen_finished(&ctx, &a);
    let tf = TileFields::new(&ctx, Some(&screen_rgb(&ctx))).without_lakes();
    let tile = render::render_biome_tile_rgba(&ctx, &fx::FIELD, fx::GW, fx::GH, TileBounds { x: 0.0, y: 0.0, w: (fx::GW - 1) as f64, h: (fx::GH - 1) as f64 }, &tf);
    assert_eq!(tile.len(), fx::GW * fx::GH * 4, "the tile came back empty or mis-sized");

    let mut rgb = Vec::with_capacity(fx::GW * fx::GH * 3);
    for px in tile.chunks(4) {
        rgb.extend_from_slice(&px[..3]);
    }
    let (worst, mean, over) = compare(&rgb, &screen);
    println!("screen identity (exag 0): worst {worst}, mean {mean:.6}, bytes over 1 level: {over}");
    assert_eq!(over, 0, "{over} bytes differ from the screen by more than the quantization level (worst {worst})");
    assert!(worst <= 1, "worst delta {worst} is more than one quantization level");
}

/// A smooth synthetic world at a chosen size, for the two measurements the
/// 10x10 capture fixture is too small to make.
///
/// **Built from the geometry under test** (`CLAUDE.md`: *"Shape fixtures to
/// reach the code"*): `ms = max(2, (min(W,H)/64)|0)` is `2` for every tile up
/// to 191 px, so the meso step a real 256 px LOD tile uses cannot be observed
/// on a 10x10 grid at all — it needs a 256-cell world to reach `ms = 4`. The
/// ridge term is there so the crest stage, which needs a **convex** and
/// **steep** land cell, has one.
fn synthetic_world(n: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut field = vec![0f32; n * n];
    let mut temp = vec![0f32; n * n];
    let mut rain = vec![0f32; n * n];
    for y in 0..n {
        for x in 0..n {
            let (u, v) = (x as f64 / n as f64, y as f64 / n as f64);
            let broad = 0.46 + 0.30 * (u * 6.0).sin() * (v * 5.0).cos();
            let ridge = 0.10 * (1.0 - ((v - 0.5) * 9.0).abs().min(1.0)).powf(2.0) * (1.0 + 0.4 * (u * 21.0).sin());
            let fine = 0.02 * (u * 61.0).sin() * (v * 47.0).cos();
            field[y * n + x] = (broad + ridge + fine) as f32;
            temp[y * n + x] = (22.0 - 34.0 * v) as f32;
            rain[y * n + x] = (0.30 + 0.55 * (u * 3.0 + 0.7).sin().abs()) as f32;
        }
    }
    (field, temp, rain)
}

/// The measurement `LOD_DETAIL_SCOPE.md`'s owner question 1 actually needs:
/// **how far a tile is from the screen it sits over, at LOD entry, at the
/// shipped look.**
///
/// The reference defines the two differently — the main map's `shadeFactor2`
/// (7681) divides the exaggeration by its sample step, the tile's meso block
/// (11742) does not — so a 256 px tile at one pixel per cell shades its meso
/// scale from a 4-cell span with no normaliser, against the map's 3-cell span
/// with one. This does not assert they agree; it prints how far apart they
/// are and pins an envelope, which is LOD-D2's own input (*"At the LOD entry
/// frame, mean |dL*| between the layer shown and hidden is <= 2.0"*).
///
/// A positive control runs first: the same comparison at `exag = 0`, where
/// every normal is `(0, 0, 1)` at any step, must collapse to the quantization
/// level. Without it a "small residual" could just as easily mean the meso
/// term is not being applied at all.
#[test]
fn screen_residual_at_lod_entry_is_the_meso_asymmetry_and_is_measured() {
    const N: usize = 256;
    let (field, temp, rain) = synthetic_world(N);
    let whole = TileBounds { x: 0.0, y: 0.0, w: (N - 1) as f64, h: (N - 1) as f64 };

    let measure = |a: TerrainAppearance| -> (i32, f64) {
        let ctx = RenderCtx::with_appearance(&field, &temp, &rain, None, N, N, fx::SEA_LEVEL, false, fx::LAT_N, fx::LAT_S, a.clone());
        let mut screen = vec![0u8; N * N * 3];
        for y in 0..N {
            for x in 0..N {
                let (r, g, b) = render::cell_color(&ctx, x, y);
                let o = (y * N + x) * 3;
                screen[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
                screen[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
                screen[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
            }
        }
        let base = screen.clone();
        let mut finished = screen;
        let influence = render::build_grade_influence(&ctx, N, N);
        render::finish_raster(&a, &mut finished, N, N, false, &influence, render::ColorSpace::Srgb);
        let tf = TileFields::new(&ctx, Some(&base)).without_lakes();
        let tile = render::render_biome_tile_rgba(&ctx, &field, N, N, whole, &tf);
        assert_eq!(tile.len(), N * N * 4, "the tile came back empty or mis-sized");
        let mut rgb = Vec::with_capacity(N * N * 3);
        for px in tile.chunks(4) {
            rgb.extend_from_slice(&px[..3]);
        }
        let (worst, mean, _) = compare(&rgb, &finished);
        (worst, mean)
    };

    // `ms` on a 256 px tile, from the reference's own expression. Stated so
    // the number below is attributable to a span and not to a mystery.
    let ms = ((N as f64 / 64.0).trunc() as usize).max(2);
    assert_eq!(ms, 4, "a 256 px tile's meso step is not 4; this measures a different span than it claims");

    let (flat_worst, flat_mean) = measure(TerrainAppearance { exag: 0.0, ..TerrainAppearance::default() });
    println!("LOD entry, exag 0 (positive control): worst {flat_worst}, mean {flat_mean:.4}");
    assert!(flat_worst <= 1, "with the shading held equal the tile and the screen differ by {flat_worst} levels -- something other than the meso term has diverged");

    let shipped = TerrainAppearance::default();
    let (worst, mean) = measure(shipped.clone());
    println!("LOD entry, exag {} (256 px tile, ms {ms} against the map's 3 with its /3): worst {worst}, mean {mean:.4}", shipped.exag);
    // Measured, not guessed. The envelope is deliberately generous: what this
    // test is for is noticing a CHANGE in the asymmetry, and the exact figure
    // is content-dependent (`MISTAKES.md`: one world is one sample).
    assert!(mean < 12.0, "mean per-channel residual {mean:.4} at LOD entry is outside the measured envelope");
}

/// The v1.05 lake shoreline (reference 11717-11740, issue #96 *"square lakes
/// when LOD zooming"*), on a world that **has a lake in it**.
///
/// The captured 10x10 fixture has no above-sea pool at all, so running this on
/// it would have measured `0 of 400 bytes move` and passed — a ~90-line block
/// of ported code with a green test over it and nothing exercised. That is the
/// silently-empty-golden shape this project has been bitten by four times, so
/// the fixture is built to reach the code instead: a broad basin ringed by a
/// rim, which `build_water_bodies`' priority-flood fills to a real
/// `fill_level` above the bed.
///
/// Three things are asserted, and the first is the one that matters:
///
/// 1. the lake branch **paints** — switching the mask off moves pixels;
/// 2. the painted colour is the reference's fresh-water re-tint and not the
///    ocean's, so a lake cannot silently render as sea;
/// 3. the shoreline follows the **terrain**, not the cell grid: a rim pixel
///    whose own amplified height is above the pool surface stays land even
///    though its coarse cell is inside the lake, which is the whole point of
///    the v1.05 fix.
#[test]
fn the_lake_branch_draws_a_lake_and_follows_the_terrain() {
    const N: usize = 64;
    let mut field = vec![0f32; N * N];
    let mut temp = vec![0f32; N * N];
    let mut rain = vec![0f32; N * N];
    for y in 0..N {
        for x in 0..N {
            let (dx, dy) = (x as f64 - 32.0, y as f64 - 32.0);
            let d = (dx * dx + dy * dy).sqrt();
            // A basin floor at 0.50 inside r=12, a rim rising to 0.72, then
            // ordinary land. Every cell is above `SEA_LEVEL` (0.42), so this is
            // an above-sea POOL and not a piece of ocean.
            let h = if d < 12.0 { 0.50 } else { (0.50 + 0.22 * ((d - 12.0) / 6.0).min(1.0)).min(0.72) };
            field[y * N + x] = h as f32;
            temp[y * N + x] = 12.0;
            rain[y * N + x] = 0.8;
        }
    }
    assert!(field.iter().all(|&h| (h as f64) >= fx::SEA_LEVEL), "the fixture has ocean in it; this test is about an above-sea pool");

    let a = TerrainAppearance { exag: 0.0, ..TerrainAppearance::default() };
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, None, N, N, fx::SEA_LEVEL, false, fx::LAT_N, fx::LAT_S, a);

    // The precondition, checked rather than assumed: the classifier must
    // actually call this a lake, or nothing below is about lakes.
    let wb = cartalith_civ::build_water_bodies(&field, N, N, fx::SEA_LEVEL, false, Some(&rain));
    let lake_cells = wb.classification.iter().filter(|&&c| c == 2).count();
    assert!(lake_cells > 50, "build_water_bodies found {lake_cells} lake cells in the basin; the fixture does not reach the branch");
    let pool = wb.fill_level[32 * N + 32] as f64;
    assert!(pool > 0.50, "the pool surface is {pool}, no higher than the basin floor -- nothing was impounded");
    println!("fixture: {lake_cells} lake cells, pool surface {pool:.4} over a 0.50 floor");

    // A tile at one pixel per cell, so tile pixel i is coarse cell i and the
    // comparison below is about the lake branch rather than about sampling.
    let b = TileBounds { x: 0.0, y: 0.0, w: (N - 1) as f64, h: (N - 1) as f64 };
    let with = render::render_biome_tile_rgba(&ctx, &field, N, N, b, &TileFields::new(&ctx, None));
    let without = render::render_biome_tile_rgba(&ctx, &field, N, N, b, &TileFields::new(&ctx, None).without_lakes());
    assert_eq!(with.len(), N * N * 4);
    let moved = with.iter().zip(without.iter()).filter(|(x, y)| x != y).count();
    assert!(moved > 4 * 100, "the lake mask moves only {moved} bytes -- the lake branch is not painting");
    println!("lake branch paints {moved} of {} bytes", with.len());

    // (2) The centre must be the reference's fresh-water re-tint of
    // `seaColorCore(0.30, ...)`: blue-dominant, and NOT the same colour the
    // ocean branch would give at that depth. Checked as an inequality against
    // the land colour under it, which is what "a lake is visible" means.
    let c = 32 * N + 32;
    let (lr, lg, lb) = (with[c * 4] as i32, with[c * 4 + 1] as i32, with[c * 4 + 2] as i32);
    let (dr, dg, db) = (without[c * 4] as i32, without[c * 4 + 1] as i32, without[c * 4 + 2] as i32);
    // Cool, not blue-DOMINANT: `seaColorCore` at depth 0.30 and T 12 lands on
    // the shelf/tropical ramp, which is a teal -- measured (97,154,151), green
    // a hair over blue. Asserting "blue is the largest channel" was a guess
    // about a palette this test never read, and it was wrong; what water
    // actually guarantees here is that both cool channels clear red by a wide
    // margin.
    assert!(lg > lr + 30 && lb > lr + 30, "the lake centre is ({lr},{lg},{lb}) -- not a water colour");
    assert!((lb - db).abs() > 8, "the lake centre ({lr},{lg},{lb}) is within 8 levels of the land under it ({dr},{dg},{db})");

    // (3) The shoreline follows the terrain. Raise one rim pixel's own tile
    // height above the pool surface while leaving the COARSE classification
    // untouched: the v1.05 test is `h < s`, so that pixel must stop being
    // water. A nearest-cell stamp -- the square-lake behaviour the fix exists
    // to remove -- would not notice.
    let mut edge = None;
    for y in 0..N {
        for x in 0..N {
            let i = y * N + x;
            if wb.classification[i] == 2 && (field[i] as f64) < pool && with[i * 4 + 2] > with[i * 4] {
                edge = Some(i);
            }
        }
    }
    let i = edge.expect("no lake pixel painted as water -- (3) cannot be tested");
    let mut raised = field.clone();
    raised[i] = (pool + 0.05) as f32;
    let after = render::render_biome_tile_rgba(&ctx, &raised, N, N, b, &TileFields::new(&ctx, None));
    let before_px = &with[i * 4..i * 4 + 3];
    let after_px = &after[i * 4..i * 4 + 3];
    assert_ne!(before_px, after_px, "raising a lake pixel above the pool surface did not change it -- the shoreline is a cell stamp, not the terrain");
    println!("shoreline follows terrain: cell {i} went {before_px:?} -> {after_px:?} when raised above the pool");

    // (4) The bilinear membership fraction, on a MAGNIFIED tile -- which is
    // the only place it can be reached, and the reason this leg exists.
    //
    // At one pixel per cell every `wx`/`wy` is an integer, so `tx = ty = 0`
    // and `fq` collapses to `lA ? 1 : 0`: it is never between, and
    // `LAKE_MEMBERSHIP_MIN` is inert. Found by mutation -- moving the constant
    // 0.35 -> 0.90 left every test above green. Zoom in past the grid (here
    // 64 px over 7 cells of shoreline, `cx` about 0.11) and the fraction is
    // continuous, which is exactly the "square lakes when LOD zooming" case
    // the v1.05 fix was written for.
    // Chosen to STRADDLE the shore, not to sit inside it: the basin fills to
    // its rim, so the waterline is where `field` reaches the pool surface at
    // about r = 18 from (32, 32), and this window spans r = 12 to r = 22.
    // The first window tried was entirely inside the lake and the assertion
    // below caught it, which is the point of having it.
    let zoom = TileBounds { x: 44.0, y: 29.0, w: 10.0, h: 6.0 };
    const ZW: usize = 64;
    let cx = zoom.w / (ZW - 1) as f64;
    let cy = zoom.h / (ZW - 1) as f64;
    let mut zt = vec![0f32; ZW * ZW];
    for y in 0..ZW {
        for x in 0..ZW {
            // Bilinear read of the coarse field, i.e. a tile with no added
            // detail: the shoreline must then come from `fq`, not from a
            // sub-cell ripple that happens to cross the pool surface.
            let (wx, wy) = (zoom.x + x as f64 * cx, zoom.y + y as f64 * cy);
            let (x0, y0) = (wx.floor() as usize, wy.floor() as usize);
            let (x1, y1) = ((x0 + 1).min(N - 1), (y0 + 1).min(N - 1));
            let (tx, ty) = (wx - x0 as f64, wy - y0 as f64);
            let v = (field[y0 * N + x0] as f64 * (1.0 - tx) + field[y0 * N + x1] as f64 * tx) * (1.0 - ty)
                + (field[y1 * N + x0] as f64 * (1.0 - tx) + field[y1 * N + x1] as f64 * tx) * ty;
            zt[y * ZW + x] = v as f32;
        }
    }
    let zr = render::render_biome_tile_rgba(&ctx, &zt, ZW, ZW, zoom, &TileFields::new(&ctx, None));
    assert_eq!(zr.len(), ZW * ZW * 4);
    // "Water" here is the fresh-water re-tint's own signature, the same
    // inequality (2) uses -- both cool channels well clear of red.
    let wet = (0..ZW * ZW)
        .filter(|&k| {
            let (r, g, b) = (zr[k * 4] as i32, zr[k * 4 + 1] as i32, zr[k * 4 + 2] as i32);
            g > r + 30 && b > r + 30
        })
        .count();
    println!("magnified shoreline tile ({ZW}px over {}x{} cells, cx {cx:.4}): {wet}/{} water px", zoom.w, zoom.h, ZW * ZW);
    assert!(wet > 0 && wet < ZW * ZW, "the magnified tile is entirely {} -- it straddles no shoreline and cannot see `fq`", if wet == 0 { "land" } else { "water" });
    // The literal the membership threshold produces on this fixture. Asserted
    // as a number, not as a range: `MISTAKES.md`'s *"assert a literal, or the
    // independent thing the value must equal"*, and it is what turns red when
    // `LAKE_MEMBERSHIP_MIN` moves.
    assert_eq!(wet, 2304, "the magnified shoreline moved; if `LAKE_MEMBERSHIP_MIN` changed deliberately, re-measure and say so");
}

// ===========================================================================
// Check 3 — seams
// ===========================================================================

/// `LOD_DETAIL_SCOPE.md` LOD-D1 check 3: *"Two adjacent tiles agree on their
/// shared column to within 2x the interior column-to-column spread."*
///
/// This is the reference's own v1.29 method, and it is the measurement that
/// found the LOD seam in the first place: the two tiles' height data at the
/// shared column was byte-identical while the colorised RGBA differed by
/// 6.7/255 against 0.3-0.5 for ordinary neighbouring columns.
///
/// Both tiles are built from the **same function of world coordinates**, so
/// their shared column carries identical heights and any difference is the
/// coloriser's. A tile-local sea blur, a tile-local AO, a clamped border
/// derivative or a tile-local local-contrast pass would each show up here.
#[test]
fn adjacent_tiles_agree_on_their_shared_column() {
    let a = TerrainAppearance { crest_strength: 0.35, sdf_coast: 0.6, sdf_rivers: 0.5, sdf_biomes: 0.4, ..TerrainAppearance::default() };
    let ctx = ctx_for(a.clone()).with_map_scale(fx::MAP_WIDTH_KM);
    let tf = TileFields::new(&ctx, Some(&screen_rgb(&ctx)));

    const TW: usize = 48;
    const TH: usize = 48;
    // Left tile spans world x 1..4, right tile 4..7: the left tile's LAST
    // column and the right tile's FIRST column are both world x = 4.
    let left_b = TileBounds { x: 1.0, y: 1.5, w: 3.0, h: 3.0 };
    let right_b = TileBounds { x: 4.0, y: 1.5, w: 3.0, h: 3.0 };

    let build = |b: TileBounds| -> Vec<f32> {
        let cx = b.w / (TW - 1) as f64;
        let cy = b.h / (TH - 1) as f64;
        let mut t = vec![0f32; TW * TH];
        for y in 0..TH {
            let wy = b.y + y as f64 * cy;
            for x in 0..TW {
                let wx = b.x + x as f64 * cx;
                // A smooth function of WORLD coordinates, so the shared column
                // is identical in both tiles by construction.
                t[y * TW + x] = (0.42 + 0.34 * (wx * 0.9).sin() * (wy * 0.7).cos() + 0.06 * (wx * 4.0 + wy * 3.0).sin()) as f32;
            }
        }
        t
    };
    let lt = build(left_b);
    let rt = build(right_b);
    for y in 0..TH {
        assert_eq!(lt[y * TW + (TW - 1)], rt[y * TW], "the fixture's own shared column is not identical at row {y}");
    }

    let lr = render::render_biome_tile_rgba(&ctx, &lt, TW, TH, left_b, &tf);
    let rr = render::render_biome_tile_rgba(&ctx, &rt, TW, TH, right_b, &tf);
    assert_eq!(lr.len(), TW * TH * 4);
    assert_eq!(rr.len(), TW * TH * 4);

    // Mean |delta| between the two tiles at the shared column.
    let mut seam = 0f64;
    for y in 0..TH {
        let a0 = (y * TW + (TW - 1)) * 4;
        let b0 = (y * TW) * 4;
        for k in 0..3 {
            seam += (lr[a0 + k] as f64 - rr[b0 + k] as f64).abs();
        }
    }
    seam /= (TH * 3) as f64;

    // The interior spread: mean |delta| between neighbouring columns inside
    // the left tile, which is what "an ordinary column boundary looks like"
    // means on this content.
    let mut interior = 0f64;
    let mut n = 0usize;
    for y in 0..TH {
        for x in 1..TW - 1 {
            let p = (y * TW + x) * 4;
            let q = (y * TW + x - 1) * 4;
            for k in 0..3 {
                interior += (lr[p + k] as f64 - lr[q + k] as f64).abs();
                n += 1;
            }
        }
    }
    interior /= n as f64;

    println!("seam {seam:.4} vs interior column spread {interior:.4} (ratio {:.3})", seam / interior.max(1e-9));
    assert!(interior > 0.05, "the interior spread is {interior:.4} -- the fixture is too flat for this test to mean anything");
    assert!(seam <= 2.0 * interior, "shared-column difference {seam:.4} is more than 2x the interior column spread {interior:.4}");
}

// ===========================================================================
// The budget, and the guards
// ===========================================================================

/// `LOD_DETAIL_SCOPE.md` LOD-D1: *"`TileFields` holds <= 32 MiB at
/// 2048x1311."*
///
/// Measured by **construction from the real grid size**, not by multiplying
/// the struct's field count on paper: the whole point of the line is what the
/// thing actually costs. A 2048x1311 fixture would be 10 MB of test data, so
/// the sizes are computed from `TileFields`' own per-cell cost measured on
/// this fixture and scaled — and the per-cell cost is asserted exactly rather
/// than estimated.
#[test]
fn tile_fields_stay_inside_the_scope_budget() {
    let a = TerrainAppearance::default();
    let ctx = ctx_for(a).with_map_scale(fx::MAP_WIDTH_KM);
    let screen = screen_rgb(&ctx);
    let tf = TileFields::new(&ctx, Some(&screen));
    let cells = fx::GW * fx::GH;
    let per_cell = tf.bytes() as f64 / cells as f64;
    let at_full = per_cell * (2048.0 * 1311.0);
    println!("TileFields: {} bytes over {cells} cells = {per_cell} B/cell; {:.2} MiB at 2048x1311", tf.bytes(), at_full / (1024.0 * 1024.0));
    // 4 (contrast band) + 1 (lake class) + 4 (lake fill) = 9 B/cell at the
    // shipped defaults, where the four grade field-weights are zero and the
    // influence raster is therefore empty. Asserted as the literal it is, so
    // adding a raster to `TileFields` turns this red rather than sliding.
    assert_eq!(per_cell, 9.0, "TileFields costs {per_cell} B/cell, not the 9 this budget was computed from");
    assert!(at_full <= 32.0 * 1024.0 * 1024.0, "{:.2} MiB at 2048x1311 is over the 32 MiB budget", at_full / (1024.0 * 1024.0));
}

/// No panic crosses the gdext boundary (`cartalith-rust-conventions`): every
/// malformed call returns an empty `Vec` instead of indexing out of bounds.
/// This is reached from the LOD bridge on every zoom notch, so a panic here
/// takes the process with it.
#[test]
fn malformed_calls_return_empty_rather_than_panicking() {
    let ctx = ctx_for(reference_appearance());
    let tf = TileFields::new(&ctx, None);
    let b = TileBounds { x: 0.0, y: 0.0, w: 4.0, h: 4.0 };
    assert!(render::render_biome_tile_rgba(&ctx, &fx::FIELD, 0, 8, b, &tf).is_empty(), "zero width");
    assert!(render::render_biome_tile_rgba(&ctx, &fx::FIELD, 8, 0, b, &tf).is_empty(), "zero height");
    assert!(render::render_biome_tile_rgba(&ctx, &fx::FIELD, 64, 64, b, &tf).is_empty(), "tile shorter than w*h");
    // A `TileFields` built for a different grid must be refused, not indexed.
    let small_field = [0.5f32; 16];
    let other = RenderCtx::with_appearance(&small_field, &small_field, &small_field, None, 4, 4, 0.42, false, 55.0, 5.0, reference_appearance());
    let other_tf = TileFields::new(&other, None);
    assert!(render::render_biome_tile_rgba(&ctx, &fx::FIELD, 8, 8, b, &other_tf).is_empty(), "TileFields from a different grid");
    // And the degenerate 1-pixel tile, where `w.max(2) - 1` is the guard
    // against a division by zero the reference writes as `Math.max(1, W-1)`.
    assert_eq!(render::render_biome_tile_rgba(&ctx, &fx::FIELD, 1, 1, b, &tf).len(), 4, "a 1x1 tile still renders one pixel");
}

/// `build_crest` grew `sx`/`sy` parameters for this milestone. The main map
/// passes `1.0, 1.0`, where `invc` is exactly `1.0` and `2.0 * sx` is exactly
/// `2.0` — so that call must be bit-identical to what it was before the
/// parameters existed. `golden_parity_render.rs` runs at `crest_strength = 0`
/// and could not have caught a change here, so this is the only thing that
/// can.
///
/// **On a synthetic ridge, not on the capture fixture.** The 10x10 captured
/// world has no convex steep land cell that clears the `G^1.5` slope weight,
/// so the stage moves zero pixels on it — measured, not assumed, and exactly
/// the silently-empty-golden shape `CLAUDE.md` warns about. The positive
/// control below fails loudly if that is ever true of this fixture too.
#[test]
fn build_crest_at_unit_scale_leaves_the_screen_unchanged() {
    const N: usize = 96;
    let (field, temp, rain) = synthetic_world(N);
    let screen_of = |a: TerrainAppearance| -> Vec<u8> {
        let ctx = RenderCtx::with_appearance(&field, &temp, &rain, None, N, N, fx::SEA_LEVEL, false, fx::LAT_N, fx::LAT_S, a);
        let mut bytes = vec![0u8; N * N * 3];
        for y in 0..N {
            for x in 0..N {
                let (r, g, b) = render::cell_color(&ctx, x, y);
                let o = (y * N + x) * 3;
                bytes[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
                bytes[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
                bytes[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
            }
        }
        bytes
    };
    const K: f64 = 0.45;
    // `js_reference()` plus the crest, deliberately: with paper, the plate
    // frame, AO, wetness, local contrast and both SDF legs all at `0.0`,
    // `cell_color` reduces to `apply_crest(land_color(..))` exactly, so the
    // comparison below isolates the crest instead of measuring it through
    // four later stages.
    let bare = TerrainAppearance { exag: fx::EXAG, sun_az_deg: fx::SUN_AZ_DEG, bio_blend: fx::BIO_BLEND, ..TerrainAppearance::js_reference() };
    let with_crest = screen_of(TerrainAppearance { crest_strength: K, ..bare.clone() });
    let plain = screen_of(TerrainAppearance { crest_strength: 0.0, ..bare.clone() });
    let moved = with_crest.iter().zip(plain.iter()).filter(|(x, y)| x != y).count();
    assert!(moved > 0, "the crest stage moves no pixel on this fixture -- this test cannot detect a change to it");
    println!("crest moves {moved} of {} screen bytes", with_crest.len());

    // The oracle: `buildCrestField` at `sx = sy = 1` and `applyCrest`, both
    // transcribed here from the reference (8047-8062) independently of
    // `build_crest`'s own code, so this compares two implementations rather
    // than one implementation with itself. `MISTAKES.md`: *"never assert a
    // constant against itself"*.
    let crest_at = |i: usize, x: usize, y: usize| -> f64 {
        let h = field[i] as f64;
        if h < fx::SEA_LEVEL {
            return 0.0;
        }
        let (xl, xr) = (if x > 0 { x - 1 } else { x }, if x + 1 < N { x + 1 } else { x });
        let (yu, yd) = (if y > 0 { y - 1 } else { y }, if y + 1 < N { y + 1 } else { y });
        let (l, r) = (field[y * N + xl] as f64, field[y * N + xr] as f64);
        let (u, d) = (field[yu * N + x] as f64, field[yd * N + x] as f64);
        let curv = l + r + u + d - 4.0 * h; // `* invc`, and `invc == 1` at unit scale
        if curv >= 0.0 {
            return 0.0;
        }
        let g = cartalith_jsmath::js_hypot((r - l) / 2.0, (d - u) / 2.0); // `/ (2 * sx)` at `sx == 1`
        // Stored through an `f32`, exactly as `build_crest` stores it
        // (`out[i] = (conv * sg) as f32`). Without this the oracle disagrees
        // by 2.9e-7 -- which is the storage, not the formula, and relaxing
        // the tolerance instead of reproducing it would have hidden a real
        // difference behind a round number.
        (((g / 0.05).min(1.0).powf(1.5) * (-curv * 250.0).clamp(0.0, 1.0)) as f32) as f64
    };
    // `applyCrest(c, s)` (reference 8062), on the reference's own literals.
    let apply = |c: (f64, f64, f64), s: f64| -> (f64, f64, f64) {
        if s <= 0.0 {
            return c;
        }
        let k = if s > 1.0 { 1.0 } else { s };
        (c.0 * (1.0 - k) + 240.0 * k, c.1 * (1.0 - k) + 238.0 * k, c.2 * (1.0 - k) + 232.0 * k)
    };

    let ctx_plain = RenderCtx::with_appearance(&field, &temp, &rain, None, N, N, fx::SEA_LEVEL, false, fx::LAT_N, fx::LAT_S, TerrainAppearance { crest_strength: 0.0, ..bare.clone() });
    let ctx_crest = RenderCtx::with_appearance(&field, &temp, &rain, None, N, N, fx::SEA_LEVEL, false, fx::LAT_N, fx::LAT_S, TerrainAppearance { crest_strength: K, ..bare.clone() });

    let mut best = (0usize, 0f64, 0usize, 0usize);
    let mut checked = 0usize;
    for y in 1..N - 1 {
        for x in 1..N - 1 {
            let i = y * N + x;
            let s = crest_at(i, x, y);
            if s <= 0.0 {
                continue;
            }
            let p = render::cell_color(&ctx_plain, x, y);
            let got = render::cell_color(&ctx_crest, x, y);
            // `cell_color` ends with `clamp01(x / 255.0)`, so a channel that
            // was already over 255 before the crest comes back as exactly
            // `1.0` and the oracle cannot recover what went in. Skipped rather
            // than fudged: the crest brightens toward (240, 238, 232), so a
            // saturated channel is where it has least to say anyway.
            if [p.0, p.1, p.2, got.0, got.1, got.2].iter().any(|v| *v <= 0.0 || *v >= 1.0) {
                continue;
            }
            checked += 1;
            let want = apply((p.0 * 255.0, p.1 * 255.0, p.2 * 255.0), s * K * 0.7);
            let (dr, dg, db) = ((got.0 * 255.0 - want.0).abs(), (got.1 * 255.0 - want.1).abs(), (got.2 * 255.0 - want.2).abs());
            assert!(dr < 1e-9 && dg < 1e-9 && db < 1e-9, "cell ({x},{y}): the unit-scale crest is not what the reference's own formula gives -- got {got:?}*255, want {want:?} (deltas {dr:.3e} {dg:.3e} {db:.3e})");
            if s > best.1 {
                best = (i, s, x, y);
            }
        }
    }
    assert!(checked > 200, "only {checked} cells carry a crest on this fixture -- the comparison is too thin to mean anything");
    assert!(best.1 > 0.0, "no convex steep land cell in the fixture -- the crest formula is untested");
    println!("crest oracle agreed on {checked} cells; strongest cell {} ({},{}) at {:.6}", best.0, best.2, best.3, best.1);
    println!("strongest independently-computed crest: cell {} at {:.6}", best.0, best.1);
}

/// `LOD_DETAIL_SCOPE.md` LOD-D1's budget line: *"desktop <= 40 ms median per
/// 256 squared tile, single thread"*, estimated there from the appearance M6
/// serial figure of about 16 ms per tile plus the per-tile blur, SDF and
/// crest.
///
/// **Two configurations, because the budget means different things in each**,
/// and reporting only one would be the single-sample shape:
///
/// - **the shipped look** (`default().with_look(LOOK_VIBRANT)`, which is what
///   `WorldGen` opens on) — crest at 0.12, AO at 0.20, and all three SDF
///   strengths at `0.0`, so the per-tile prologue builds the crest field and
///   nothing else. This is the one asserted against 40 ms.
/// - **every prologue stage on** — the coast SDF, the river SDF and the biome
///   boundary distance are each a full distance transform over the tile's own
///   65 536 pixels, and none of them is reachable at the shipped defaults.
///   Measured and printed, not asserted: the scope's estimate did not include
///   three distance transforms, so an assertion here would be pinning a
///   number the budget was never written against.
///
/// `#[ignore]` because a timing taken under a parallel test suite is not a
/// timing (`MISTAKES.md`: *"Run the harness **alone**, never under a parallel
/// suite"*). Run it on its own:
///
/// ```text
/// cargo test -p cartalith-godot --test golden_parity_tile_biome -- --ignored --nocapture --test-threads=1
/// ```
///
/// Single-threaded by construction — a one-thread rayon pool, since the
/// function is row-parallel and the budget is stated per thread. Median with
/// min..max, never a point estimate.
#[test]
#[ignore]
fn tile_synthesis_cost_at_256_squared_is_over_the_scope_budget() {
    const N: usize = 256;
    let (field, temp, rain) = synthetic_world(N);

    // A deep-zoom tile: 256 px over 8 coarse cells, `cx` about 0.0317, which
    // is the case the LOD path actually pays for. A tile at one pixel per cell
    // would understate the SDF and crest work, since both scale with tile
    // pixels and not with ground.
    let b = TileBounds { x: 40.0, y: 40.0, w: 8.0, h: 8.0 };
    let tile: Vec<f32> = (0..N * N).map(|k| field[((k / N) * 8 / N + 40) * N + ((k % N) * 8 / N + 40)]).collect();

    let time_it = |label: &str, a: TerrainAppearance, threads: usize| -> f64 {
        let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&field), N, N, fx::SEA_LEVEL, false, fx::LAT_N, fx::LAT_S, a.clone()).with_map_scale(fx::MAP_WIDTH_KM);
        let mut screen = vec![0u8; N * N * 3];
        for y in 0..N {
            for x in 0..N {
                let (r, g, bl) = render::cell_color(&ctx, x, y);
                let o = (y * N + x) * 3;
                screen[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
                screen[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
                screen[o + 2] = (bl.clamp(0.0, 1.0) * 255.0) as u8;
            }
        }
        let tf = TileFields::new(&ctx, Some(&screen));
        let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().expect("thread pool");
        pool.install(|| {
            // Two warm-ups, not timed: the first call pays for page faults on
            // a buffer nothing has touched yet.
            for _ in 0..2 {
                let v = render::render_biome_tile_rgba(&ctx, &tile, N, N, b, &tf);
                assert_eq!(v.len(), N * N * 4);
            }
            let mut ms: Vec<f64> = (0..11)
                .map(|_| {
                    let t0 = std::time::Instant::now();
                    let v = render::render_biome_tile_rgba(&ctx, &tile, N, N, b, &tf);
                    let e = t0.elapsed().as_secs_f64() * 1000.0;
                    assert_eq!(v.len(), N * N * 4, "the timed call returned nothing -- the number below would be meaningless");
                    e
                })
                .collect();
            ms.sort_by(|x, y| x.partial_cmp(y).unwrap());
            let med = ms[ms.len() / 2];
            println!("{label}: median {:.2} ms ({:.2}..{:.2}) over 11 runs, 256^2, {threads} thread(s)", med, ms[0], ms[ms.len() - 1]);
            med
        })
    };

    let shipped = TerrainAppearance::default().with_look(render::LOOK_VIBRANT);
    // The premise, checked rather than assumed: if the shipped look ever turns
    // an SDF on, this test is measuring something else and should say so.
    assert_eq!((shipped.sdf_coast, shipped.sdf_rivers, shipped.sdf_biomes), (0.0, 0.0, 0.0), "the shipped look now enables an SDF stage; re-read this test's doc comment");
    assert!(shipped.crest_strength > 0.0, "the shipped look no longer builds a crest field; this measurement is of a cheaper tile than it claims");

    let med = time_it("shipped look (crest on, SDFs off), 1 thread", shipped.clone(), 1);
    let all_on = time_it("every prologue stage on (3 distance transforms), 1 thread", TerrainAppearance { sdf_coast: 0.6, sdf_rivers: 0.5, sdf_biomes: 0.4, ..shipped.clone() }, 1);
    println!("the three SDF stages cost {:.2} ms of the second figure", all_on - med);

    // What a FRAME actually pays. The function is row-parallel, so the
    // single-thread figure the scope's budget is written against is not the
    // one LOD-D2's *"no frame over 50 ms from synthesis"* is about. Both are
    // reported because they answer different questions.
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    let par = time_it("shipped look, all cores", shipped, cores);
    println!("available_parallelism = {cores}; parallel speedup {:.2}x", med / par.max(1e-9));

    // **This is a finding, not a pass.** The budget is 40 ms and the measured
    // single-thread median is over it. The test asserts a REGRESSION ceiling
    // rather than the budget, because the budget is a decision about what to
    // spend and this file cannot make it -- and asserting the measured value
    // as if it were the budget would turn an overrun into a green test.
    //
    // Where the overrun comes from, measured rather than guessed: the scope
    // sized 40 ms as *"the appearance M6 serial figure of about 16 ms per
    // tile, plus the per-tile blur, SDF and crest"*, and that estimate
    // accounts for none of the PORT-ONLY tail this tile also runs at the
    // shipped look -- `paper_tone`'s parchment fibre (an fbm per pixel at
    // `paper_strength` 0.85), the forest stipple inside `land_color`, the
    // plate frame, and the local-contrast correction. The reference's own
    // tile has none of those, which is exactly why its 16 ms sibling was
    // cheaper.
    //
    // Three ways out, none of them this test's to choose: gate the parchment
    // and stipple off for tiles (they are a *sheet* property and arguably
    // wrong at deep zoom anyway), take the parallel figure as the real budget
    // since a frame has more than one core, or accept it and let LOD-D6 move
    // synthesis off the main thread as it already plans to.
    println!("BUDGET: scope says <= 40 ms single-thread; measured {med:.2} ms ({:.2}x over). See this test's doc comment.", med / 40.0);
    assert!(med <= 80.0, "median {med:.2} ms is a regression even against the measured 51.9 ms overrun, not just against the 40 ms budget");
}
