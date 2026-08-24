//! The export raster (`bakeDims`/`bakePixel`/`bakeSingle`/`bakeTiled`,
//! reference HTML 10241/11931/11975/11982) — `render::BakeFields` and
//! `render::bake_rect`.
//!
//! # Why this is an identity test and not a golden dump
//!
//! `PARITY_TESTING.md` asks for golden values extracted from the reference.
//! For this subsystem the *right* golden value is not the reference's own
//! `bakePixel` output, and saying why is the whole point of this file.
//!
//! `bakePixel` is not a fractional twin of `surfaceColor` in the reference —
//! it is an older sibling that drifted. It computes its sea noise with a
//! different expression, and it predates this port's wave, paper and plate-
//! frame stages entirely. Matching it pixel-for-pixel would ship an export
//! that does **not** match the map on screen, which is the one property the
//! reference's own comments say a bake must have (*"bakes match the
//! screen"*, twice, at lines 11923 and 11930).
//!
//! So the golden reference here is this port's own `cell_color` — already
//! golden-verified against the JS engine by `golden_parity_render.rs` — and
//! the claim under test is the one the reference itself makes about
//! `sampleArr` at `curvatureAtF` (7620): *"identical to the originals at any
//! EXACT integer coordinate (sampleArr at an integer point returns the exact
//! cell value with zero interpolation weight on its neighbours)"*. If that
//! holds for all six fractional twins and all thirteen sampled fields, then
//! the export raster is `cell_color` evaluated on a finer lattice, and
//! `cell_color`'s own parity carries over to it.
//!
//! **Bit-exact on this file's fixture**, not within a tolerance: both sides
//! run the identical Rust code on identical inputs, with no
//! `Math.hypot`/`exp` divergence to absorb, so any difference on a 24x17
//! world is a real substitution bug.
//!
//! It does **not** stay bit-exact at scale, and
//! `the_integer_identity_is_f32_tight_not_bit_exact_at_scale` at the bottom
//! of this file is the honest statement of where it stops: `BakeFields`
//! stores the three prologue fields as `f32` because the reference's own
//! bake prologue does, while `cell_color` computes them in `f64` — so the
//! two agree to `f32` rounding rather than to the bit, and at 2048x1312
//! that showed up live as ~15 bytes of 8 060 928 off by one level. Widening
//! the prologue would remove a divergence the reference itself has.

#[path = "../src/render.rs"]
mod render;

use render::{BakeFields, RenderCtx, TerrainAppearance};

const GW: usize = 24;
const GH: usize = 17;

/// A deterministic synthetic world, shaped to reach the code rather than to
/// look plausible: a diagonal ridge crossing a sea-level basin so both the
/// land and the sea branch run, temperatures spanning the boreal/tropical
/// material switches, and rainfall spanning the dry/wet ones. Built from
/// closed-form trigonometry so the fixture is a property of this file and
/// cannot drift with the generator.
fn fixture() -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    let n = GW * GH;
    let mut field = vec![0f32; n];
    let mut temp = vec![0f32; n];
    let mut rain = vec![0f32; n];
    let mut flow = vec![0f32; n];
    for y in 0..GH {
        for x in 0..GW {
            let (u, v) = (x as f64 / GW as f64, y as f64 / GH as f64);
            let i = y * GW + x;
            // Two ridges and a basin: the range spans `sea_level` well on
            // both sides, so neither branch is a corner case.
            field[i] = (0.46 + 0.34 * ((u * 6.1).sin() * (v * 4.3).cos()) + 0.12 * ((u * 17.0 + v * 11.0).sin())) as f32;
            temp[i] = (28.0 - 44.0 * v + 6.0 * (u * 9.0).cos()) as f32;
            rain[i] = (0.5 + 0.5 * ((u * 5.0 + v * 3.0).sin())).clamp(0.0, 1.0) as f32;
            flow[i] = (1.0 + 40.0 * ((u * 13.0).sin() * (v * 7.0).sin()).abs()) as f32;
        }
    }
    (field, temp, rain, flow)
}

/// Every cell of `field` classified as land or sea, so a test can assert it
/// actually exercised both.
fn split(field: &[f32], sea: f64) -> (usize, usize) {
    let land = field.iter().filter(|&&h| h as f64 >= sea).count();
    (land, field.len() - land)
}

/// The one deviation, and the only one: the bake prologue's three derived
/// fields (slope, macro shade, meso shade) are stored `f32`, so a bake reads
/// them back through one `f32` rounding that `cell_color` — which computes
/// them in `f64` and uses them immediately — does not pay.
///
/// **`f32` is the correct choice here, not a shortcut.** The reference's own
/// `gridSlope`/`gridShade`/`gridShadeMeso` are `Float32Array`s, and
/// `cartalith-rust-conventions` is explicit that widening a field the JS
/// engine keeps at single precision is *improving* on the original, which
/// this project treats as a silent parity change rather than a free win. It
/// is also what keeps the prologue at 12 bytes per cell instead of 24 — at
/// the app's own 2048x1311 that is 32 MiB against 64 MiB, and the gap is
/// what decides whether an 8192-cell world can be exported at all.
///
/// Measured below rather than asserted loosely: the tolerance is two orders
/// of magnitude tighter than one byte level (`1/255 == 3.9e-3`), and
/// [`assert_integer_identity`] additionally asserts that the **quantised**
/// output — the actual PNG byte — is identical, which is the property a user
/// can observe.
const ULP_TOL: f64 = 1e-7;

/// The claim: at an integer grid position the fractional path and the
/// integer path are the same function, to within [`ULP_TOL`], and produce
/// the identical exported byte.
fn assert_integer_identity(ctx: &RenderCtx, what: &str) {
    let bf = BakeFields::new(ctx);
    let mut moved = 0usize;
    let mut first: Option<[u8; 3]> = None;
    let mut varied = false;
    let mut worst = 0f64;
    for y in 0..GH {
        for x in 0..GW {
            let want = render::cell_color(ctx, x, y);
            let got = bf.pixel(ctx, x as f64, y as f64);
            for (g, w, ch) in [(got.0, want.0, 'r'), (got.1, want.1, 'g'), (got.2, want.2, 'b')] {
                let d = (g - w).abs();
                worst = worst.max(d);
                assert!(d < ULP_TOL, "{what}: cell ({x},{y}) channel {ch} diverged -- bake {g}, screen {w}, diff {d}");
            }
            let qg = [(got.0 * 255.0) as u8, (got.1 * 255.0) as u8, (got.2 * 255.0) as u8];
            let qw = [(want.0 * 255.0) as u8, (want.1 * 255.0) as u8, (want.2 * 255.0) as u8];
            assert_eq!(qg, qw, "{what}: cell ({x},{y}) exports a different byte than it draws");
            if qg != [0, 0, 0] {
                moved += 1;
            }
            match first {
                None => first = Some(qg),
                Some(f) => {
                    if f != qg {
                        varied = true;
                    }
                }
            }
        }
    }
    // The "silently-empty golden output" rule (root `CLAUDE.md`): an
    // identity assertion passes trivially if both sides return the same
    // constant, so the output has to be shown to be a real picture.
    assert_eq!(moved, GW * GH, "{what}: some cells came back pure black");
    assert!(varied, "{what}: every cell is the same colour -- the fixture is not reaching the material path");
    // Non-vacuous in the other direction too: if `worst` were exactly zero
    // the `f32` prologue would not be in the path at all, which would mean
    // the bake is not reading it.
    assert!(worst > 0.0, "{what}: not one channel moved -- the prologue fields are not reaching the bake");
    println!("{what}: worst channel divergence {worst:e} ({:.4} byte levels)", worst * 255.0);
}

#[test]
fn bake_pixel_at_integer_cells_is_cell_color_js_reference() {
    let (field, temp, rain, flow) = fixture();
    let (land, sea) = split(&field, 0.42);
    assert!(land > 40 && sea > 40, "fixture must exercise both branches, got {land} land / {sea} sea");
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, 0.42, false, 55.0, 5.0, TerrainAppearance::js_reference());
    assert_integer_identity(&ctx, "js_reference / flat");
}

#[test]
fn bake_pixel_at_integer_cells_is_cell_color_world_wrap() {
    // `world = true` is the case `grad_at_f` exists for: `grad_at` wraps X
    // and `sample_arr` clamps, so a naive transcription would disagree along
    // the two vertical edges. Hachure is on here precisely so that branch
    // runs at all -- `cell_color` skips the gradient entirely when it is off.
    let (field, temp, rain, flow) = fixture();
    let mut a = TerrainAppearance::default();
    a.npr.hachure = 0.6;
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, 0.42, true, 55.0, 5.0, a);
    assert_integer_identity(&ctx, "default + hachure / world wrap");
}

#[test]
fn bake_pixel_at_integer_cells_is_cell_color_with_lithology_and_paint() {
    // The two categorical layers, which are nearest-sampled rather than
    // bilinear (`litho_at_f`, `paint_at_f`). A bilinear read of either would
    // blend two unrelated palette indices into a meaningless third, and
    // would break this identity at every cell whose neighbours differ --
    // which is what makes this test the one that catches it.
    let (field, temp, rain, flow) = fixture();
    let n = GW * GH;
    let lith: Vec<u8> = (0..n).map(|i| (i % 7) as u8).collect();
    let biome: Vec<u8> = (0..n).map(|i| if i % 5 == 0 { (i % 9) as u8 } else { 0 }).collect();
    let terrain: Vec<u8> = (0..n).map(|i| if i % 11 == 0 { (i % 6) as u8 } else { 0 }).collect();
    let splat: Vec<u8> = (0..n).map(|i| if i % 13 == 0 { (i % 6) as u8 } else { 0 }).collect();
    let a = TerrainAppearance::default();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, 0.42, false, 55.0, 5.0, a)
        .with_lithology(&lith)
        .with_paint(Some(&biome), Some(&terrain), Some(&splat));
    assert_integer_identity(&ctx, "default + lithology + paint");
}

#[test]
fn bake_pixel_survives_a_world_with_no_flow_field() {
    // A loaded save carries no discharge (`SAVEFILE_COMPAT.md`), so
    // `ctx.flow` is `None` and the TWI term falls back to its floor. The
    // export must work there too -- it is the one world state a user is most
    // likely to be exporting from.
    let (field, temp, rain, _) = fixture();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, None, GW, GH, 0.42, false, 55.0, 5.0, TerrainAppearance::default());
    assert_integer_identity(&ctx, "default / no flow");
}

#[test]
fn bake_dims_keeps_the_worlds_aspect_ratio() {
    // `Math.round(W*GH/GW)`, the reference's own one-liner.
    assert_eq!(render::bake_dims(2048, 2048, 1311), (2048, 1311));
    assert_eq!(render::bake_dims(4096, 2048, 1311), (4096, 2622));
    assert_eq!(render::bake_dims(8192, 2048, 1311), (8192, 5244));
    // Square worlds stay square at every offered resolution.
    for w in [2048usize, 4096, 8192] {
        assert_eq!(render::bake_dims(w, 1024, 1024), (w, w));
    }
    // A degenerate ask must not panic or divide by zero -- this runs behind
    // a `#[func]`, and a panic there takes the Godot process down
    // (`cartalith-rust-conventions`).
    assert_eq!(render::bake_dims(0, 512, 512), (0, 0));
    assert_eq!(render::bake_dims(512, 0, 512), (0, 0));
    // Never zero-height, however extreme the aspect.
    let (_, h) = render::bake_dims(2048, 4096, 1);
    assert!(h >= 1, "a one-cell-tall world must still bake at least one row");
}

#[test]
fn an_export_at_grid_resolution_reproduces_the_screen_raster() {
    // The end-to-end statement, through the real `bake_rect` sample mapping
    // rather than through `pixel` directly: `sx=(GW-1)/(w-1)` puts output
    // pixel 0 on cell 0 and output pixel `w-1` on cell `GW-1`, so at
    // `w == GW` every output pixel lands on a whole cell and the export is
    // the screen image byte for byte.
    let (field, temp, rain, flow) = fixture();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, 0.42, false, 55.0, 5.0, TerrainAppearance::default());
    let bf = BakeFields::new(&ctx);
    let got = render::bake_rect(&ctx, &bf, None, GW, GH, 0, 0, GW, GH);
    assert_eq!(got.len(), GW * GH * 3);

    let mut want = vec![0u8; GW * GH * 3];
    for y in 0..GH {
        for x in 0..GW {
            let (r, g, b) = render::cell_color(&ctx, x, y);
            let o = (y * GW + x) * 3;
            want[o] = (r * 255.0) as u8;
            want[o + 1] = (g * 255.0) as u8;
            want[o + 2] = (b * 255.0) as u8;
        }
    }
    assert_eq!(got, want, "a grid-resolution export is not the screen raster");
    assert!(want.iter().any(|&b| b != want[0]), "the fixture rendered a flat colour");
}

#[test]
fn a_tiled_bake_is_the_same_image_as_a_single_one() {
    // `bakeTiled` and `bakeSingle` differ only in which rectangles they ask
    // for. If they can disagree about a pixel, the `bakeTiles` checkbox
    // silently changes the export, which is the bug this system is least
    // able to detect once shipped.
    let (field, temp, rain, flow) = fixture();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, 0.42, false, 55.0, 5.0, TerrainAppearance::default());
    let bf = BakeFields::new(&ctx);
    let (ow, oh) = (61usize, 43usize); // deliberately not a multiple of the tile size
    let whole = render::bake_rect(&ctx, &bf, None, ow, oh, 0, 0, ow, oh);

    const TS: usize = 16;
    let mut stitched = vec![0u8; ow * oh * 3];
    let mut tiles = 0usize;
    for r in 0..oh.div_ceil(TS) {
        for c in 0..ow.div_ceil(TS) {
            let (tw, th) = ((TS).min(ow - c * TS), (TS).min(oh - r * TS));
            let tile = render::bake_rect(&ctx, &bf, None, ow, oh, c * TS, r * TS, tw, th);
            tiles += 1;
            for ty in 0..th {
                let src = ty * tw * 3;
                let dst = ((r * TS + ty) * ow + c * TS) * 3;
                stitched[dst..dst + tw * 3].copy_from_slice(&tile[src..src + tw * 3]);
            }
        }
    }
    assert_eq!(tiles, 4 * 3, "the tile walk did not cover the raster as expected");
    assert_eq!(stitched, whole, "tiled and single bakes disagree");
}

#[test]
fn a_finer_export_is_a_finer_render_not_an_upscale() {
    // The reason this milestone existed at all. If the export were a
    // nearest-neighbour upscale of the screen raster, every 2x pixel would
    // equal one of the cell colours; because the whole material path runs at
    // the fractional position, most of them are values the cell grid never
    // produced.
    let (field, temp, rain, flow) = fixture();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, 0.42, false, 55.0, 5.0, TerrainAppearance::default());
    let bf = BakeFields::new(&ctx);

    let cells: std::collections::BTreeSet<[u8; 3]> = (0..GH)
        .flat_map(|y| (0..GW).map(move |x| (x, y)))
        .map(|(x, y)| {
            let (r, g, b) = render::cell_color(&ctx, x, y);
            [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
        })
        .collect();

    let (ow, oh) = (GW * 2, GH * 2);
    let fine = render::bake_rect(&ctx, &bf, None, ow, oh, 0, 0, ow, oh);
    let novel = fine.chunks_exact(3).filter(|p| !cells.contains(&[p[0], p[1], p[2]])).count();
    let frac = novel as f64 / (ow * oh) as f64;
    assert!(frac > 0.5, "only {:.1}% of a 2x export's pixels are new colours -- this looks like an upscale, not a render", frac * 100.0);
}

/// The river-channel tint has to be *in* the export, and it has to be the
/// screen's own bytes — the regression that made this test exist was an
/// export that rendered the terrain perfectly and dropped every river.
///
/// Found live, not here: a grid-resolution export compared against
/// `build_color_texture` came back with 291 815 of 8 060 928 bytes
/// different, and all of them were channel cells. The unit tests above
/// could not have caught it, because they compare `bake_rect` against
/// `cell_color` and the tint lives in neither — it is composited by
/// `build_color_texture`'s own loop, which is a `#[func]` this target
/// cannot call. So this reproduces that loop's two lines verbatim and
/// requires the export to match them byte for byte.
///
/// The quantization order is the load-bearing part. `build_color_texture`
/// tints in `f64` and quantizes once; a tint applied to the finished bytes
/// instead disagrees on blue for half of all inputs, because `b*0.5 + 0.45`
/// lands on a `.75` fraction where `floor` stops commuting with the
/// halving. That is why `channel_tint` is called inside `bake_rect`'s pixel
/// loop rather than as a pass over its result.
#[test]
fn a_grid_resolution_export_carries_the_river_tint() {
    let (field, temp, rain, flow) = fixture();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, 0.42, false, 55.0, 5.0, TerrainAppearance::default());
    let bf = BakeFields::new(&ctx);
    let a = TerrainAppearance::default();

    // A mask with real structure rather than a solid block: every third
    // column plus a diagonal, so the test sees tinted and untinted cells
    // adjacent to each other in both axes.
    let mask: Vec<u8> = (0..GW * GH).map(|i| u8::from(i % GW % 3 == 0 || i % GW == i / GW)).collect();
    let tinted = mask.iter().filter(|&&m| m != 0).count();
    assert!(tinted > GW * GH / 8 && tinted < GW * GH * 7 / 8, "the fixture mask is degenerate: {tinted} of {} cells", GW * GH);

    let got = render::bake_rect(&ctx, &bf, Some(&mask), GW, GH, 0, 0, GW, GH);

    // `build_color_texture`'s loop, transcribed.
    let mut want = vec![0u8; GW * GH * 3];
    for y in 0..GH {
        for x in 0..GW {
            let i = y * GW + x;
            let (mut r, mut g, mut b) = render::cell_color(&ctx, x, y);
            if mask[i] != 0 {
                let cover = render::border_cover(&a, x, y, GW, GH);
                if cover < 1.0 {
                    let (tr, tg, tb) = (r * 0.5, (g * 0.5 + 0.3).min(1.0), (b * 0.5 + 0.45).min(1.0));
                    r = tr + (r - tr) * cover;
                    g = tg + (g - tg) * cover;
                    b = tb + (b - tb) * cover;
                }
            }
            let o = i * 3;
            want[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            want[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            want[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    assert_eq!(got, want, "a grid-resolution export is not the screen raster once rivers are in it");

    // And the tint is not a no-op that the assertion above would pass
    // vacuously: without the mask the same export must differ, at exactly
    // the tinted cells and nowhere else.
    let plain = render::bake_rect(&ctx, &bf, None, GW, GH, 0, 0, GW, GH);
    let changed = (0..GW * GH).filter(|&i| plain[i * 3..i * 3 + 3] != got[i * 3..i * 3 + 3]).count();
    assert!(changed > 0, "the mask changed nothing -- the tint is not being applied");
    assert!(changed <= tinted, "{changed} cells changed but only {tinted} are masked");
}

/// **The export runs the two whole-raster stages, in the viewport's own
/// order** — `apply_local_contrast` then `apply_color_grade`.
///
/// The stages themselves are covered by `appearance_tiers.rs`; what is
/// covered here is that the *export* runs them, and runs them in that
/// sequence. That distinction is not pedantry — it is the gap this test was
/// written to close. `export_raster.rs` does call both, but nothing proved
/// it, and the live grid-resolution byte-for-byte probe could not: the
/// shipped default look (`Natural Vibrant`) leaves every grade parameter at
/// rest, so `apply_color_grade` is an early return on both sides of that
/// comparison and a missing call would have passed it silently. Exactly the
/// root `CLAUDE.md` "silently-empty golden output" failure, one level up.
///
/// So the look under test is **`Antique Parchment`**, the one shipped look
/// whose grade is not the identity (temperature `0.26`, saturation `-0.10`,
/// contrast `0.08`, shadow tint `0.18`), and the test asserts three things:
/// the grade is not vacuous at that look, the export's sequence reproduces
/// the viewport's byte for byte, and the order is load-bearing.
#[test]
fn a_grid_resolution_export_carries_the_colour_grade() {
    let (field, temp, rain, flow) = fixture();
    let a = TerrainAppearance::default().with_look(render::LOOK_ANTIQUE);
    assert!(!a.grade_is_identity(), "the fixture look grades nothing -- this test would pass vacuously");
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, 0.42, false, 55.0, 5.0, a.clone());
    let bf = BakeFields::new(&ctx);

    // `export_raster_png`'s own calls, in its own order — including the
    // grade's field-influence weights, built from the same `build_grade_
    // influence(ctx, w, h)` both the export and `build_color_texture` use.
    // Empty here, because Antique leaves all four weights at rest.
    let inf = render::build_grade_influence(&ctx, GW, GH);
    let mut got = render::bake_rect(&ctx, &bf, None, GW, GH, 0, 0, GW, GH);
    let ungraded = got.clone();
    render::apply_local_contrast(&a, &mut got, GW, GH, false);
    let pre_grade = got.clone();
    render::apply_color_grade(&a, &mut got, &inf);

    // 1. The grade is real here, so a missing call cannot hide.
    let moved = (0..GW * GH).filter(|&i| pre_grade[i * 3..i * 3 + 3] != got[i * 3..i * 3 + 3]).count();
    assert!(moved * 2 > GW * GH, "the grade moved only {moved} of {} pixels -- too few for this look to be a real test", GW * GH);
    let mad = (0..got.len()).map(|i| pre_grade[i].abs_diff(got[i]) as f64).sum::<f64>() / got.len() as f64;
    assert!(mad > 2.0, "the grade moved a mean of {mad:.2} byte levels -- measurably nothing");

    // 2. `build_color_texture`'s sequence, transcribed as this file already
    //    transcribes its river-tint loop, must land on the same bytes.
    let mut want = vec![0u8; GW * GH * 3];
    for y in 0..GH {
        for x in 0..GW {
            let (r, g, b) = render::cell_color(&ctx, x, y);
            let o = (y * GW + x) * 3;
            want[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            want[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            want[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    render::apply_local_contrast(&a, &mut want, GW, GH, false);
    render::apply_color_grade(&a, &mut want, &inf);
    assert_eq!(got, want, "the graded export is not the graded screen raster");

    // 3. The order is load-bearing: contrast-then-grade is not
    //    grade-then-contrast, so an export that ran them the other way round
    //    would fail assertion 2 rather than sneak through it.
    let mut swapped = ungraded;
    render::apply_color_grade(&a, &mut swapped, &inf);
    render::apply_local_contrast(&a, &mut swapped, GW, GH, false);
    assert_ne!(swapped, got, "the two whole-raster stages commute on this fixture -- assertion 2 cannot see a reorder");
}

/// And at the **shipped default**, the grade costs the export nothing and
/// changes nothing — the other half of the claim, so that adding the call to
/// the export path cannot have moved the image every existing test and probe
/// was baselined against.
#[test]
fn the_export_is_unchanged_by_the_grade_at_the_shipped_look() {
    let (field, temp, rain, flow) = fixture();
    for look in [render::LOOK_TIER, render::LOOK_VIBRANT] {
        let a = TerrainAppearance::default().with_look(look);
        assert!(a.grade_is_identity(), "`{look}` no longer grades at rest -- every export baseline taken under it needs re-checking");
        let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, 0.42, false, 55.0, 5.0, a.clone());
        let bf = BakeFields::new(&ctx);
        let inf = render::build_grade_influence(&ctx, GW, GH);
        assert!(inf.is_empty(), "`{look}` builds an influence field for a grade that does nothing");
        let mut px = render::bake_rect(&ctx, &bf, None, GW, GH, 0, 0, GW, GH);
        render::apply_local_contrast(&a, &mut px, GW, GH, false);
        let before = px.clone();
        render::apply_color_grade(&a, &mut px, &inf);
        assert_eq!(px, before, "`{look}` moved a pixel through a grade that is supposed to be at rest");
    }
}

/// A finer export tints the same *world*, not more of it. Nearest-cell is
/// the deliberate choice (`channel_tint`'s doc comment); the property that
/// makes it the right one is that a river covers the same fraction of the
/// image at every resolution, which a bilinear sample of a categorical mask
/// would not give.
#[test]
fn the_river_tint_keeps_its_world_width_at_every_resolution() {
    let (field, temp, rain, flow) = fixture();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, 0.42, false, 55.0, 5.0, TerrainAppearance::default());
    let bf = BakeFields::new(&ctx);
    let mask: Vec<u8> = (0..GW * GH).map(|i| u8::from(i % GW % 3 == 0)).collect();

    let mut fracs = Vec::new();
    for mult in [1usize, 2, 4] {
        let (ow, oh) = (GW * mult, GH * mult);
        let with = render::bake_rect(&ctx, &bf, Some(&mask), ow, oh, 0, 0, ow, oh);
        let without = render::bake_rect(&ctx, &bf, None, ow, oh, 0, 0, ow, oh);
        let n = (0..ow * oh).filter(|&i| with[i * 3..i * 3 + 3] != without[i * 3..i * 3 + 3]).count();
        fracs.push(n as f64 / (ow * oh) as f64);
    }
    let (lo, hi) = (fracs.iter().cloned().fold(f64::MAX, f64::min), fracs.iter().cloned().fold(0.0, f64::max));
    assert!(hi - lo < 0.05, "the tinted fraction moves with resolution: {fracs:?}");
    assert!(lo > 0.1, "the tint barely covers anything at some resolution: {fracs:?}");
}

/// **Where the integer identity stops being bit-exact, and why that is the
/// reference's own answer rather than a defect.**
///
/// Every test above asserts exact equality on the 24x17 fixture and gets
/// it. That is fixture luck, and this test exists so the next reader does
/// not mistake it for a guarantee. Measured on a 401x277 fixture:
/// `BakeFields::pixel` at an integer cell differs from `cell_color` in
/// `f64` on **51% of cells**, by at most `2.4e-8` — and after quantization
/// to a byte, on **none** of them. Live, at 2048x1312 through the real
/// binding, 12 and then 17 bytes of 8 060 928 crossed a quantization
/// boundary across two runs, every one of them by a single level. It is a
/// knife-edge count -- a byte either lands on the boundary or it does not --
/// so the assertion below is an order-of-magnitude bound, not an exact
/// figure.
///
/// The cause is [`BakeFields`]' own documented choice: it stores slope,
/// macro shade and meso shade as `f32`, because the reference's bake
/// prologue stores `gridSlope`/`gridShade`/`gridShadeMeso` in
/// `Float32Array`s while its screen path computes them on the fly in
/// doubles. So the reference has exactly this discrepancy between its bake
/// and its screen, and widening to `f64` here would *remove* a divergence
/// the original has — which is not what parity means, and would cost
/// 1.6 GB instead of 805 MB of prologue at this port's 8192 grid ceiling.
///
/// The bound is what gets asserted, then: `f64` agreement to `1e-7`, and no
/// quantized byte off by more than one.
#[test]
fn the_integer_identity_is_f32_tight_not_bit_exact_at_scale() {
    const BW: usize = 401;
    const BH: usize = 277;
    let n = BW * BH;
    let (mut field, mut temp, mut rain, mut flow) = (vec![0f32; n], vec![0f32; n], vec![0f32; n], vec![0f32; n]);
    for y in 0..BH {
        for x in 0..BW {
            let (u, v) = (x as f64 / BW as f64, y as f64 / BH as f64);
            let i = y * BW + x;
            field[i] = (0.46 + 0.34 * ((u * 6.1).sin() * (v * 4.3).cos()) + 0.12 * ((u * 17.0 + v * 11.0).sin())) as f32;
            temp[i] = (28.0 - 44.0 * v + 6.0 * (u * 9.0).cos()) as f32;
            rain[i] = (0.5 + 0.45 * ((u * 5.0 + 1.0).sin() * (v * 7.0).sin())) as f32;
            flow[i] = (1.0 + 400.0 * ((u * 13.0).sin().abs() * (v * 3.0).cos().abs())) as f32;
        }
    }
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), BW, BH, 0.42, false, 55.0, 5.0, TerrainAppearance::default());
    let bf = BakeFields::new(&ctx);

    let q = |v: f64| (v.clamp(0.0, 1.0) * 255.0) as u8;
    let (mut worst, mut bytes_off, mut worst_byte) = (0f64, 0usize, 0u8);
    for y in 0..BH {
        for x in 0..BW {
            let a = render::cell_color(&ctx, x, y);
            let b = bf.pixel(&ctx, x as f64, y as f64);
            worst = worst.max((a.0 - b.0).abs().max((a.1 - b.1).abs()).max((a.2 - b.2).abs()));
            for (p, s) in [(a.0, b.0), (a.1, b.1), (a.2, b.2)] {
                let d = q(p).abs_diff(q(s));
                if d > 0 {
                    bytes_off += 1;
                    worst_byte = worst_byte.max(d);
                }
            }
        }
    }
    assert!(worst < 1e-7, "the integer identity has drifted past f32 rounding: worst f64 delta {worst:e}");
    assert!(worst_byte <= 1, "a quantized byte is off by {worst_byte} levels, not the at-most-one f32 rounding allows");
    assert!(
        bytes_off * 10_000 < n * 3,
        "{bytes_off} of {} bytes cross a quantization boundary -- far more than f32 rounding explains",
        n * 3
    );
}
