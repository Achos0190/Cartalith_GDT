//! `GUI_GAP_REGISTER.md` CA-03 / CA-04 and RD-10's precondition — the raster's
//! three terrain sub-layers as a **separable, ordered, blendable stack**.
//!
//! This file exists for one reason above all the others: to pin the shipped
//! image to a literal that was measured by the binary *before* the layer stack
//! was written. `quality_tier_is_exactly_the_default_look` next door compares
//! two renders of the same build and therefore cannot see a change that moves
//! both; a digest literal transcribed from the previous build can.

#[path = "../src/render.rs"]
mod render;

use render::{BlendMode, LayerEntry, LayerStack, RasterLayer, RenderCtx, TerrainAppearance};

// The same synthetic world `appearance_tiers.rs` renders against, reproduced
// rather than shared because integration-test targets are separate crates and
// this one must be able to `#[path]`-include `render.rs` on its own.
const GW: usize = 128;
const GH: usize = 79;

struct Synth {
    field: Vec<f32>,
    temperature: Vec<f32>,
    rainfall: Vec<f32>,
    flow: Vec<f32>,
    lith: Vec<u8>,
}

fn synth() -> Synth {
    let n = GW * GH;
    let mut field = vec![0f32; n];
    let mut temperature = vec![0f32; n];
    let mut rainfall = vec![0f32; n];
    let mut flow = vec![0f32; n];
    let mut lith = vec![0u8; n];
    for y in 0..GH {
        for x in 0..GW {
            let (xf, yf) = (x as f64, y as f64);
            let i = y * GW + x;
            let ridge = (xf * 0.11).sin() * (yf * 0.09).cos();
            let fine = (xf * 0.37 + yf * 0.29).sin() * 0.08;
            let bowl = 1.0 - ((xf / GW as f64 - 0.5).hypot(yf / GH as f64 - 0.5) * 1.9).min(1.0);
            field[i] = (0.30 + 0.34 * ridge + fine + 0.30 * bowl).clamp(0.0, 1.0) as f32;
            temperature[i] = (1.0 - yf / GH as f64).clamp(0.0, 1.0) as f32;
            rainfall[i] = (0.25 + 0.7 * ((xf * 0.05).sin() * 0.5 + 0.5)).clamp(0.0, 1.0) as f32;
            flow[i] = if (x + 2 * y) % 37 == 0 { 4000.0 } else { 3.0 };
            lith[i] = ((x / 13 + y / 9) % 7) as u8;
        }
    }
    Synth { field, temperature, rainfall, flow, lith }
}

fn ctx<'a>(s: &'a Synth, a: TerrainAppearance) -> RenderCtx<'a> {
    RenderCtx::with_appearance(&s.field, &s.temperature, &s.rainfall, Some(&s.flow), GW, GH, 0.42, false, 55.0, 5.0, a).with_lithology(&s.lith)
}

/// The whole on-screen pipeline `lib.rs::build_color_texture` runs, minus the
/// river tint and the icon composite (neither of which the layer stack can
/// reach): `cell_color`, then local contrast, then the grade.
fn render_all(s: &Synth, a: &TerrainAppearance) -> Vec<u8> {
    let c = ctx(s, a.clone());
    let mut out = vec![0u8; GW * GH * 3];
    for y in 0..GH {
        for x in 0..GW {
            let (r, g, b) = render::cell_color(&c, x, y);
            let o = (y * GW + x) * 3;
            out[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    render::apply_local_contrast(a, &mut out, GW, GH, false);
    render::apply_color_grade(a, &mut out, &render::build_grade_influence(&c, GW, GH));
    out
}

/// The export path's own pixel function over the same world, at integer cell
/// coordinates — `bake_rect`'s inner call, isolated. Separate from
/// `render_all` on purpose: a capability wired into `cell_color` and not into
/// `BakeFields::pixel` is the divergence `MISTAKES.md` records for
/// `with_ground_tiles`, and it moves no pixel at the default, so only an
/// explicit second render can see it.
fn bake_all(s: &Synth, a: &TerrainAppearance) -> Vec<u8> {
    let c = ctx(s, a.clone());
    let bf = render::BakeFields::new(&c);
    let mut out = vec![0u8; GW * GH * 3];
    for y in 0..GH {
        for x in 0..GW {
            let (r, g, b) = bf.pixel(&c, x as f64, y as f64);
            let o = (y * GW + x) * 3;
            out[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    out
}

/// FNV-1a over the finished raster. Written out rather than pulled from a
/// crate because this file may add no dependency, and because a digest whose
/// definition is three lines here cannot itself drift.
fn digest(px: &[u8]) -> u64 {
    let mut fnv: u64 = 0xcbf2_9ce4_8422_2325;
    for b in px {
        fnv ^= *b as u64;
        fnv = fnv.wrapping_mul(0x1000_0000_01b3);
    }
    fnv
}

/// A stack, top-first, from `(layer, visible, opacity, blend)` — written in the
/// order a panel lists it and reversed here, so a test reads the way the UI
/// will.
fn stack(top_first: [(RasterLayer, bool, f64, BlendMode); 3]) -> LayerStack {
    let mut s = LayerStack::DEFAULT;
    let mut e: Vec<LayerEntry> = top_first.iter().map(|&(layer, visible, opacity, blend)| LayerEntry { layer, visible, opacity, blend }).collect();
    e.reverse();
    assert!(s.set(<[LayerEntry; 3]>::try_from(e).unwrap()), "the stack under test was itself refused");
    s
}

fn with_stack(s: LayerStack) -> TerrainAppearance {
    TerrainAppearance { layers: s, ..TerrainAppearance::default() }
}

fn moved(a: &[u8], b: &[u8], tol: i32) -> f64 {
    let d = a.iter().zip(b).filter(|(p, q)| (**p as i32 - **q as i32).abs() > tol).count();
    d as f64 / a.len() as f64
}

/// Prints the digest of every appearance this file pins, so the literals below
/// can be re-derived by hand after a deliberate look change rather than
/// guessed. `cargo test -p cartalith-godot --test layer_stack -- --ignored --nocapture print_digests`
#[test]
#[ignore]
fn print_digests() {
    let s = synth();
    for (name, a) in [
        ("default", TerrainAppearance::default()),
        ("vibrant", TerrainAppearance::default().with_look(render::LOOK_VIBRANT)),
        ("antique", TerrainAppearance::default().with_look(render::LOOK_ANTIQUE)),
        ("js_reference", TerrainAppearance::js_reference()),
    ] {
        println!("screen {name:>13}: {:#018x}", digest(&render_all(&s, &a)));
        println!("bake   {name:>13}: {:#018x}", digest(&bake_all(&s, &a)));
    }
}

/// What the one unconditional per-pixel test added by CA-03/CA-04 costs,
/// against what a pixel of this renderer already costs.
///
/// `#[ignore]`d and reported as a **median over `min..max`**, never a single
/// sample, and it must be run alone — `MISTAKES.md` records three figures that
/// did not reproduce because they were one sample taken under a parallel
/// `cargo test`. Run:
/// `cargo test --release -p cartalith-godot --test layer_stack -- --ignored --nocapture --test-threads=1 cost_of_the_per_pixel_stack_test`
///
/// **Read the minimum, not the median, and expect no answer.** Measured
/// 2026-09-03 by A/B — this build against one with `is_default()` replaced by a
/// constant `true` — over five paired runs, the marginal cost came out
/// −15.6 %, −0.5 %, +3.9 %, +5.8 %, +7.5 %. The **sign flips**, so no factor
/// can be claimed from it; the minimum of fifteen samples is 235.9–242.2 ns/px
/// with the test and 237.9–241.7 ns/px without, which is the same number. The
/// honest statement is therefore *the per-pixel test is below this machine's
/// noise floor*, not a percentage. Part 1's isolated figure (~2.3 ns/px) is an
/// upper bound inflated by `black_box`, which forces a reload the real call
/// site does not make.
#[test]
#[ignore]
fn cost_of_the_per_pixel_stack_test() {
    use std::hint::black_box;
    use std::time::Instant;
    let s = synth();
    let a = TerrainAppearance::default();
    let c = ctx(&s, a.clone());

    // 1. `is_default()` alone, amortized over the same number of calls a real
    //    render makes.
    let reps = 256;
    let mut check = Vec::new();
    for _ in 0..15 {
        let t = Instant::now();
        let mut acc = 0usize;
        for _ in 0..reps {
            for _ in 0..(GW * GH) {
                acc += black_box(black_box(&a.layers).is_default()) as usize;
            }
        }
        black_box(acc);
        check.push(t.elapsed().as_secs_f64() / (reps * GW * GH) as f64);
    }
    check.sort_by(|x, y| x.partial_cmp(y).unwrap());

    // 2. One whole `cell_color` pixel, same fixture, same build.
    let mut px = Vec::new();
    for _ in 0..15 {
        let t = Instant::now();
        let mut acc = 0.0;
        for _ in 0..reps {
            for y in 0..GH {
                for x in 0..GW {
                    acc += render::cell_color(&c, x, y).0;
                }
            }
        }
        black_box(acc);
        px.push(t.elapsed().as_secs_f64() / (reps * GW * GH) as f64);
    }
    px.sort_by(|x, y| x.partial_cmp(y).unwrap());

    let ns = |v: &Vec<f64>| (v[7] * 1e9, v[0] * 1e9, v[14] * 1e9);
    let (cm, clo, chi) = ns(&check);
    let (pm, plo, phi) = ns(&px);
    println!("is_default()  median {cm:.3} ns/px  ({clo:.3}..{chi:.3})");
    println!("cell_color()  median {pm:.1} ns/px  ({plo:.1}..{phi:.1})");
    println!("share         {:.3}% of a pixel", cm / pm * 100.0);
    println!("at 2048x1311  {:.2} ms added per full render", cm * 2048.0 * 1311.0 / 1e6);
}

// ---------------------------------------------------------------------------
// 1. The identity — the whole reason this file exists
// ---------------------------------------------------------------------------

/// **The digests below were produced by the build immediately before the layer
/// stack was written**, by `print_digests` above, and transcribed by hand.
/// That is what makes them a real pin rather than a restatement: nothing in
/// this build computed them, so a "default" path that drifted by one ulp fails
/// here even though every relative comparison in the suite still agrees with
/// itself.
///
/// Four appearances, because the default arm has to be the identity for all of
/// them and not merely for `default()`: the two named looks change the light
/// curve and the colour data a composite would see, and `js_reference()` is the
/// golden-parity path.
///
/// If a deliberate look change moves these, re-derive them with
/// `cargo test -p cartalith-godot --test layer_stack -- --ignored --nocapture print_digests`
/// **and say which look moved and why** — a silently updated digest is the same
/// as no digest.
///
/// Screen and bake differ from each other for a reason that is not the stack:
/// `render_all` runs local contrast and the grade over the finished raster and
/// `bake_all` is the bare per-pixel path. Only `js_reference()`, which switches
/// both of those off, agrees across the two.
#[test]
fn the_default_stack_renders_the_pre_change_image() {
    let s = synth();
    for (name, a, screen, bake) in [
        ("default", TerrainAppearance::default(), 0x2e7b_4258_49e7_10d6u64, 0x9408_99ac_3349_694du64),
        ("vibrant", TerrainAppearance::default().with_look(render::LOOK_VIBRANT), 0xeb64_802f_ae20_df7b, 0x66cf_b547_b3b7_9dcc),
        ("antique", TerrainAppearance::default().with_look(render::LOOK_ANTIQUE), 0xae24_83aa_9cb4_63bf, 0x5cc8_aa1d_c354_b557),
        ("js_reference", TerrainAppearance::js_reference(), 0x4cba_6557_c30e_4029, 0x4cba_6557_c30e_4029),
    ] {
        assert!(a.layers.is_default(), "{name} did not start on the default stack");
        assert_eq!(digest(&render_all(&s, &a)), screen, "{name}: the on-screen raster moved");
        assert_eq!(digest(&bake_all(&s, &a)), bake, "{name}: the export raster moved");
    }
}

/// A stack a caller *constructed* to be the default has to hit the same branch
/// as the one nobody touched. Without this, `set()` could normalise a value and
/// produce a stack that renders through the composite while claiming to be the
/// shipped arrangement.
#[test]
fn an_explicitly_built_default_stack_is_the_identity() {
    let s = synth();
    let explicit = stack([
        (RasterLayer::Hillshade, true, 1.0, BlendMode::Multiply),
        (RasterLayer::ColourRelief, true, 1.0, BlendMode::Normal),
        (RasterLayer::Terrain, true, 1.0, BlendMode::Normal),
    ]);
    assert!(explicit.is_default(), "a hand-built copy of DEFAULT did not compare equal to it");
    assert_eq!(
        render_all(&s, &with_stack(explicit.clone())),
        render_all(&s, &TerrainAppearance::default()),
        "the explicit default moved the on-screen image"
    );
    assert_eq!(bake_all(&s, &with_stack(explicit)), bake_all(&s, &TerrainAppearance::default()), "the explicit default moved the export image");
}

/// The `DEFAULT` constant, asserted against literals rather than against
/// itself. `assert_eq!(DEFAULT.entries()[2].blend, BlendMode::Multiply)` would
/// hold for whatever the constant said; naming the arrangement out loud does
/// not.
#[test]
fn the_default_stack_is_the_arrangement_this_renderer_hardcoded() {
    let e = LayerStack::DEFAULT.entries();
    assert_eq!(e[0].layer.id(), "terrain", "terrain must composite first — it is the only opaque source");
    assert_eq!(e[1].layer.id(), "colour_relief");
    assert_eq!(e[2].layer.id(), "hillshade", "the light curve must be last, over everything");
    assert_eq!(e[0].blend.name(), "Normal");
    assert_eq!(e[1].blend.name(), "Normal", "CA-02's ramp blend is a lerp, which is Normal-over");
    assert_eq!(e[2].blend.name(), "Multiply", "`c * light` is a multiply and nothing else");
    for row in e {
        assert!(row.visible, "{} starts hidden", row.layer.id());
        assert_eq!(row.opacity, 1.0, "{} does not start opaque", row.layer.id());
    }
}

// ---------------------------------------------------------------------------
// 2. Every control is load-bearing — and reaches both consumer paths
// ---------------------------------------------------------------------------

/// Visibility, opacity, blend mode and order each have to move the picture, on
/// the screen path **and** on the export path.
///
/// The second half is not redundant. `MISTAKES.md` records `with_ground_tiles`
/// being attached to `build_color_texture` and not to
/// `export_raster.rs`'s render ctx, invisible to the whole suite because it
/// moved no pixel at the default. `cell_color` and `BakeFields::pixel` are two
/// hand-written transcriptions of the same pipeline, so "it is in `land_color`,
/// therefore both have it" is an argument — this is the measurement.
#[test]
fn every_stack_control_moves_both_consumer_paths() {
    let s = synth();
    let base_screen = render_all(&s, &TerrainAppearance::default());
    let base_bake = bake_all(&s, &TerrainAppearance::default());
    // A ramp the Colour relief row can actually carry: the shipped
    // `ramp_strength` is 0, so that row is empty until something turns it on,
    // and a test that reordered an empty layer would prove nothing.
    let mut lit = TerrainAppearance::default();
    lit.set_tunable("ramp_strength", 0.6);
    lit.ramp = render::ElevationRamp::preset("Elevation").expect("the Elevation preset must exist");
    let lit_screen = render_all(&s, &lit);
    let lit_bake = bake_all(&s, &lit);

    let cases: [(&str, LayerStack); 5] = [
        (
            "hillshade hidden",
            stack([
                (RasterLayer::Hillshade, false, 1.0, BlendMode::Multiply),
                (RasterLayer::ColourRelief, true, 1.0, BlendMode::Normal),
                (RasterLayer::Terrain, true, 1.0, BlendMode::Normal),
            ]),
        ),
        (
            "hillshade at half opacity",
            stack([
                (RasterLayer::Hillshade, true, 0.5, BlendMode::Multiply),
                (RasterLayer::ColourRelief, true, 1.0, BlendMode::Normal),
                (RasterLayer::Terrain, true, 1.0, BlendMode::Normal),
            ]),
        ),
        (
            "hillshade blended Overlay",
            stack([
                (RasterLayer::Hillshade, true, 1.0, BlendMode::Overlay),
                (RasterLayer::ColourRelief, true, 1.0, BlendMode::Normal),
                (RasterLayer::Terrain, true, 1.0, BlendMode::Normal),
            ]),
        ),
        (
            "hillshade blended Screen",
            stack([
                (RasterLayer::Hillshade, true, 1.0, BlendMode::Screen),
                (RasterLayer::ColourRelief, true, 1.0, BlendMode::Normal),
                (RasterLayer::Terrain, true, 1.0, BlendMode::Normal),
            ]),
        ),
        (
            "terrain hidden",
            stack([
                (RasterLayer::Hillshade, true, 1.0, BlendMode::Multiply),
                (RasterLayer::ColourRelief, true, 1.0, BlendMode::Normal),
                (RasterLayer::Terrain, false, 1.0, BlendMode::Normal),
            ]),
        ),
    ];
    for (name, st) in cases {
        assert!(!st.is_default(), "{name} compared equal to the default stack");
        let a = with_stack(st);
        let m_screen = moved(&base_screen, &render_all(&s, &a), 2);
        let m_bake = moved(&base_bake, &bake_all(&s, &a), 2);
        assert!(m_screen > 0.05, "{name}: on-screen raster moved only {:.3}% of samples", m_screen * 100.0);
        assert!(m_bake > 0.05, "{name}: EXPORT raster moved only {:.3}% of samples", m_bake * 100.0);
    }

    // Reorder: colour relief above the light instead of under it. Needs a live
    // ramp, hence the separate baseline.
    let reordered = stack([
        (RasterLayer::ColourRelief, true, 1.0, BlendMode::Normal),
        (RasterLayer::Hillshade, true, 1.0, BlendMode::Multiply),
        (RasterLayer::Terrain, true, 1.0, BlendMode::Normal),
    ]);
    assert!(!reordered.is_default());
    let a = TerrainAppearance { layers: reordered, ..lit.clone() };
    let m_screen = moved(&lit_screen, &render_all(&s, &a), 2);
    let m_bake = moved(&lit_bake, &bake_all(&s, &a), 2);
    assert!(m_screen > 0.05, "reordering colour relief above the light moved only {:.3}% on screen", m_screen * 100.0);
    assert!(m_bake > 0.05, "reordering colour relief above the light moved only {:.3}% in the export", m_bake * 100.0);

    // And colour relief's own opacity, over the same live ramp.
    let dimmed = stack([
        (RasterLayer::Hillshade, true, 1.0, BlendMode::Multiply),
        (RasterLayer::ColourRelief, true, 0.25, BlendMode::Normal),
        (RasterLayer::Terrain, true, 1.0, BlendMode::Normal),
    ]);
    let a = TerrainAppearance { layers: dimmed, ..lit };
    assert!(moved(&lit_screen, &render_all(&s, &a), 2) > 0.05, "colour relief opacity moved nothing on screen");
    assert!(moved(&lit_bake, &bake_all(&s, &a), 2) > 0.05, "colour relief opacity moved nothing in the export");
}

/// Hiding Terrain must leave the grey relief plate a reader means by
/// "hillshade alone", not a black one — the white ground `LayerStack::
/// composite` starts from is the only thing that makes that true, and a black
/// or transparent ground under a Multiply would render the whole plate at
/// level 0.
///
/// Measured with every *tinting* stage below the composite switched off —
/// parchment, plate frame, haze, atmosphere and the near-channel wetness are
/// all warm or cool by design and sit under everything in `cell_color`, so at
/// the shipped defaults a grey plate legitimately carries ~24 levels of chroma
/// that have nothing to do with the material. Stripping them is what makes the
/// claim about the *ground colour* rather than about the sheet: with them off,
/// a hidden Terrain must leave land **exactly** neutral and bright, which is
/// only true if the composite starts from white.
#[test]
fn terrain_hidden_leaves_a_bright_neutral_relief_plate() {
    let s = synth();
    let bare = TerrainAppearance {
        paper_strength: 0.0,
        border_width_frac: 0.0,
        haze_strength: 0.0,
        atmo_desaturation: 0.0,
        atmo_contrast: 0.0,
        hydro_wet_strength: 0.0,
        ..TerrainAppearance::default()
    };
    let a = TerrainAppearance {
        layers: stack([
            (RasterLayer::Hillshade, true, 1.0, BlendMode::Multiply),
            (RasterLayer::ColourRelief, true, 1.0, BlendMode::Normal),
            (RasterLayer::Terrain, false, 1.0, BlendMode::Normal),
        ]),
        ..bare
    };
    let px = render_all(&s, &a);
    // Land only: the sea has its own colour path and is deliberately outside
    // this stack (`sea_color_core` folds its own shade in).
    let (mut land, mut bright, mut luma) = (0usize, 0usize, 0.0f64);
    for (i, c) in px.chunks(3).enumerate() {
        if s.field[i] < 0.42 {
            continue;
        }
        land += 1;
        assert_eq!(c[0], c[1], "land pixel {i} is not neutral: {c:?}");
        assert_eq!(c[0], c[2], "land pixel {i} is not neutral: {c:?}");
        luma += c[0] as f64;
        if c[0] > 40 {
            bright += 1;
        }
    }
    assert!(land > 1000, "the fixture has no land to check ({land} cells)");
    assert!(
        bright as f64 / land as f64 > 0.9,
        "the relief plate came out black — {bright}/{land} land pixels above level 40, mean {:.1}",
        luma / land as f64
    );
}

/// The composite has to *be* the pipeline it replaced, not merely a plausible
/// one — otherwise the identity above is a bypass rather than a proof, and the
/// first user to touch a layer gets a different map for no stated reason.
///
/// Forced onto the composite branch without changing what it should compute:
/// Colour relief's blend is set to `Multiply`, which at the shipped
/// `ramp_strength = 0` reaches a row that contributes nothing and is skipped.
/// So the composite evaluates `white → Normal(material) → Multiply(light·255)`
/// where the default arm evaluates `material · light`, and the two must agree
/// to rounding. They cannot agree to the *bit* — `light · 255 / 255` is not
/// `light` to the last ulp, which is exactly why the shipped path is a branch
/// and not this loop — so the bound is stated as at most one level, on almost
/// no pixels.
#[test]
fn the_composite_reproduces_the_pipeline_it_replaced() {
    let s = synth();
    let mut inert = LayerStack::DEFAULT;
    let mut e = *LayerStack::DEFAULT.entries();
    e[1].blend = BlendMode::Multiply;
    assert!(inert.set(e));
    assert!(!inert.is_default(), "the forcing stack must not be the default, or this test proves nothing");
    assert_eq!(TerrainAppearance::default().ramp_strength, 0.0, "this test relies on the colour relief row being empty at the default");

    for (name, base, got) in [
        ("screen", render_all(&s, &TerrainAppearance::default()), render_all(&s, &with_stack(inert.clone()))),
        ("bake", bake_all(&s, &TerrainAppearance::default()), bake_all(&s, &with_stack(inert))),
    ] {
        let worst = base.iter().zip(&got).map(|(p, q)| (*p as i32 - *q as i32).abs()).max().unwrap_or(0);
        assert!(worst <= 1, "{name}: the composite disagrees with the hardcoded pipeline by {worst} levels");
        let any = moved(&base, &got, 0);
        assert!(any < 0.001, "{name}: {:.4}% of samples moved — that is a different picture, not rounding", any * 100.0);
    }
}

// ---------------------------------------------------------------------------
// 3. The blend formulas, against literals
// ---------------------------------------------------------------------------

/// Each mode's arithmetic, asserted against numbers worked out by hand rather
/// than against the implementation. `assert_eq!(Multiply.apply(..), d * s / 255)`
/// would hold whatever `apply` did.
#[test]
fn each_blend_mode_is_its_own_formula() {
    let d = (100.0, 100.0, 100.0);
    let s = (200.0, 200.0, 200.0);
    // Normal at full alpha is the source, exactly.
    assert_eq!(BlendMode::Normal.apply(d, s, 1.0), (200.0, 200.0, 200.0));
    // Multiply: 100 * 200 / 255 = 78.4313725...
    assert!((BlendMode::Multiply.apply(d, s, 1.0).0 - 78.431_372_549_019_6).abs() < 1e-9);
    // Screen: 255 - 155 * 55 / 255 = 221.5686274...
    assert!((BlendMode::Screen.apply(d, s, 1.0).0 - 221.568_627_450_980_4).abs() < 1e-9);
    // Overlay with d = 100 < 127.5: 2 * 100 * 200 / 255 = 156.8627450...
    assert!((BlendMode::Overlay.apply(d, s, 1.0).0 - 156.862_745_098_039_2).abs() < 1e-9);
    // Overlay with d = 200 >= 127.5: 255 - 2 * 55 * 55 / 255 = 231.2745098...
    assert!((BlendMode::Overlay.apply(s, s, 1.0).0 - 231.274_509_803_921_6).abs() < 1e-9);
    // Overlay's pivot is **mid-range, 127.5**, not 128 — a backdrop that sits
    // between the two takes the light branch. Written just above the pivot
    // because that is the only place the two candidate constants disagree, and
    // a test that only sampled 100 and 200 lets 128.0 survive (it did, on the
    // first mutation pass). Light branch at d = 127.75, s = 200:
    // 255 - 2 * 127.25 * 55 / 255 = 200.1078431...; the dark branch would give
    // 2 * 127.75 * 200 / 255 = 200.3921568...
    assert!((BlendMode::Overlay.apply((127.75, 127.75, 127.75), s, 1.0).0 - 200.107_843_137_254_9).abs() < 1e-9);
    // Add, unclamped on purpose.
    assert_eq!(BlendMode::Add.apply(d, s, 1.0), (300.0, 300.0, 300.0));
    // Alpha 0 is the backdrop for every mode; alpha 0.5 is halfway.
    for m in [BlendMode::Normal, BlendMode::Multiply, BlendMode::Screen, BlendMode::Overlay, BlendMode::Add] {
        assert_eq!(m.apply(d, s, 0.0), d, "{} at alpha 0 was not the backdrop", m.name());
    }
    assert_eq!(BlendMode::Normal.apply(d, s, 0.5), (150.0, 150.0, 150.0));

    // The one identity the whole composite path rests on: a Multiply whose
    // source is `light * 255` is `dst * light`. Written with a `light` above
    // 1.0, because `relief_ambient + relief_gain` is 1.47 at the default and a
    // formula that clamped would be wrong there, not merely conservative.
    let light = 1.25;
    assert!((BlendMode::Multiply.apply((80.0, 80.0, 80.0), (light * 255.0, light * 255.0, light * 255.0), 1.0).0 - 100.0).abs() < 1e-9);
}

/// Every mode name has to survive the round trip a panel makes, and the list
/// has to start with the identity.
#[test]
fn blend_mode_names_round_trip() {
    assert_eq!(render::BLEND_MODES[0], "Normal", "the identity must be the first row a picker draws");
    for name in render::BLEND_MODES {
        let m = BlendMode::from_name(name).unwrap_or_else(|| panic!("{name} is listed and not parseable"));
        assert_eq!(m.name(), *name);
        assert_eq!(BlendMode::from_name(&name.to_lowercase()), Some(m), "{name} must parse case-insensitively");
    }
    assert_eq!(BlendMode::from_name("Rhubarb"), None, "an unknown mode must be refused, not defaulted");
}

/// Layer ids round-trip, and are distinct from the labels — the id is what a
/// save file and a panel address a row by, so it must not be the display
/// string.
#[test]
fn layer_ids_round_trip() {
    for l in [RasterLayer::Terrain, RasterLayer::ColourRelief, RasterLayer::Hillshade] {
        assert_eq!(RasterLayer::from_id(l.id()), Some(l));
        assert!(!l.label().is_empty());
    }
    assert_eq!(RasterLayer::from_id("Terrain"), None, "ids are exact, not case-folded labels");
    assert_eq!(RasterLayer::from_id("hillshade "), None);
}

// ---------------------------------------------------------------------------
// 4. The setter refuses what it cannot represent
// ---------------------------------------------------------------------------

/// A stack that names a category twice, or omits one, is refused with nothing
/// changed. `MISTAKES.md`: never encode "no value" as a plausible value — a
/// missing `hillshade` row is not a hidden hillshade.
#[test]
fn an_incomplete_stack_is_refused_and_changes_nothing() {
    let mut s = LayerStack::DEFAULT;
    let dup = [
        LayerEntry { layer: RasterLayer::Terrain, visible: true, opacity: 1.0, blend: BlendMode::Normal },
        LayerEntry { layer: RasterLayer::Terrain, visible: false, opacity: 0.2, blend: BlendMode::Screen },
        LayerEntry { layer: RasterLayer::Hillshade, visible: true, opacity: 1.0, blend: BlendMode::Multiply },
    ];
    assert!(!s.set(dup), "a stack naming terrain twice was accepted");
    assert!(s.is_default(), "a refused set() still mutated the stack");
}

/// Opacity is clamped by the setter, so nothing downstream has to defend
/// against a negative or a 4.0 — asserted against literals, not against a range
/// constant.
#[test]
fn the_setter_clamps_opacity() {
    let mut s = LayerStack::DEFAULT;
    assert!(s.set([
        LayerEntry { layer: RasterLayer::Terrain, visible: true, opacity: -3.0, blend: BlendMode::Normal },
        LayerEntry { layer: RasterLayer::ColourRelief, visible: true, opacity: 4.0, blend: BlendMode::Normal },
        LayerEntry { layer: RasterLayer::Hillshade, visible: true, opacity: 0.25, blend: BlendMode::Multiply },
    ]));
    assert_eq!(s.entries()[0].opacity, 0.0);
    assert_eq!(s.entries()[1].opacity, 1.0);
    assert_eq!(s.entries()[2].opacity, 0.25);
}

/// A stack survives the CA-08 saved-look round trip, and a preset written
/// before the field existed still loads — `#[serde(default)]` on
/// `TerrainAppearance` is what makes the second half true, and it is the half
/// that breaks silently.
#[test]
fn a_stack_survives_the_appearance_preset_round_trip() {
    let a = with_stack(stack([
        (RasterLayer::ColourRelief, true, 0.4, BlendMode::Screen),
        (RasterLayer::Terrain, true, 1.0, BlendMode::Normal),
        (RasterLayer::Hillshade, false, 1.0, BlendMode::Overlay),
    ]));
    let json = serde_json::to_string(&a).expect("appearance must serialize");
    let back: TerrainAppearance = serde_json::from_str(&json).expect("appearance must deserialize");
    assert_eq!(back.layers, a.layers, "the stack did not survive the preset round trip");

    let older: TerrainAppearance = serde_json::from_str("{}").expect("an appearance file with no `layers` key must still load");
    assert!(older.layers.is_default(), "a preset written before the stack existed must load on the shipped arrangement");
}
