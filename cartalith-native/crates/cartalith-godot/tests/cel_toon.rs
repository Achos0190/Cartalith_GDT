//! The **Cel / Toon** style (owner, 2026-09-27: *"cel shading for a bit of a
//! more stylized look 'cartoonish'"*; `OUTSTANDING_WORK.md`'s "Cel / Toon" row):
//! `render.rs`'s `toon_strength` (banded light, flat albedo) and `toon_outline`
//! (a coast and lake-shore keyline).
//!
//! What this file pins, and why each is here rather than in a probe:
//!
//! 1. **The shipped looks are bit-identical to the tree before the style
//!    existed.** Twelve FNV-1a digests -- `default()`, `js_reference()` and the
//!    two named looks, each through the grid (`cell_color`), the export bake
//!    (`bake_rect`) and the deep-zoom tile (`render_biome_tile_rgba`) path --
//!    measured on the tree **before** `toon_*` was written (render.rs had no
//!    diff against `2cf0143`) and unchanged after it. Every other style preset
//!    is a look plus Painter/appearance keys none of which is `toon_*`, so its
//!    image is reached only through code these digests already cover.
//! 2. **`toon_band`'s ladder**: literal band edges, the soft terminator's
//!    literal width, and exactly four light levels on a full ramp for any sun.
//! 3. **The flat albedo, measured**: local colour variance on flat ground falls.
//! 4. **The keyline**: land-side only, within its radius, on lake shores as
//!    well as the coast on all three paths, and the export draws the screen's
//!    keyline at grid resolution.
//!
//! The 1024x656 on-screen measurement (mottle, band share, deep-zoom levels)
//! is `godot-project/_stylepresets_probe.gd`'s Cel section -- `cargo test`
//! cannot see the shell.

#[path = "../src/render.rs"]
#[allow(dead_code)]
mod render;

use render::{BakeFields, RenderCtx, TerrainAppearance, TileBounds, TileFields};

const GW: usize = 128;
const GH: usize = 79;

/// `appearance_tiers.rs`'s synthetic world, verbatim: real relief, a real
/// coastline (the bowl), a climate gradient, drainage and lithology.
struct Synth {
    gw: usize,
    gh: usize,
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
    Synth { gw: GW, gh: GH, field, temperature, rainfall, flow, lith }
}

/// A **flat** world: one height well above sea level everywhere, a smooth
/// climate gradient. Every pixel-to-pixel colour difference on it is either
/// the gradient (tiny per pixel) or texture noise -- which is what the flat
/// albedo exists to remove.
///
/// 512 x 316, not the synth's 128 x 79: the render's noise octaves are
/// map-relative (`x / gw`), so on a 128-wide grid even the broad biome jitter
/// is a few cells a feature and reads as speckle. 512 is the smallest size
/// the app generates at, and where those octaves have the proportions they
/// have on screen.
fn flat_world() -> Synth {
    let (gw, gh) = (512, 316);
    let n = gw * gh;
    let mut s = Synth { gw, gh, field: vec![0.7; n], temperature: vec![0f32; n], rainfall: vec![0f32; n], flow: vec![3.0; n], lith: vec![0u8; n] };
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            // Temperate (12-14 °C, moderate rain): grass, far from every
            // material threshold, so the fixture measures texture and not a
            // biome boundary that the jitter drags back and forth.
            s.temperature[i] = 12.0 + 2.0 * (y as f32 / gh as f32);
            s.rainfall[i] = 0.40 + 0.1 * (x as f32 / gw as f32);
        }
    }
    s
}

/// The app's own attachments (`appearance_tiers.rs`'s `ctx`, same reasons).
fn ctx<'a>(s: &'a Synth, a: TerrainAppearance) -> RenderCtx<'a> {
    RenderCtx::with_appearance(&s.field, &s.temperature, &s.rainfall, Some(&s.flow), s.gw, s.gh, 0.42, false, 55.0, 5.0, a)
        .with_lithology(&s.lith)
        .with_map_scale(800.0)
}

/// `cell_color` over the whole grid, RGB8, truncated like `build_color_texture`.
fn grid(s: &Synth, a: &TerrainAppearance) -> Vec<u8> {
    let c = ctx(s, a.clone());
    let mut out = vec![0u8; s.gw * s.gh * 3];
    for y in 0..s.gh {
        for x in 0..s.gw {
            let (r, g, b) = render::cell_color(&c, x, y);
            let o = (y * s.gw + x) * 3;
            out[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    out
}

/// The export bake at a size that is not the grid's, so its fractional path runs.
fn bake(s: &Synth, a: &TerrainAppearance) -> Vec<u8> {
    let c = ctx(s, a.clone());
    let bf = BakeFields::new(&c);
    render::bake_rect(&c, &bf, None, 300, 185, 0, 0, 300, 185)
}

/// One 96x96 deep-zoom tile over 40 cells of the synth world, nearest-sampled.
fn tile(s: &Synth, a: &TerrainAppearance) -> Vec<u8> {
    let c = ctx(s, a.clone());
    let px = 96;
    let (x0, y0, span) = (30.0, 20.0, 40.0);
    let step = span / (px - 1) as f64;
    let mut t = vec![0f32; px * px];
    for y in 0..px {
        for x in 0..px {
            let (wx, wy) = (x0 + x as f64 * step, y0 + y as f64 * step);
            t[y * px + x] = s.field[(wy.round() as usize).min(GH - 1) * GW + (wx.round() as usize).min(GW - 1)];
        }
    }
    render::render_biome_tile_rgba(&c, &t, px, px, TileBounds { x: x0, y: y0, w: span, h: span }, &TileFields::new(&c, None))
}

/// FNV-1a, 64-bit (`color_space.rs`'s, for the same reason: reproducible by
/// inspection, no hasher dependency).
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Rec.709 luma of pixel `i` of an RGB8 buffer.
fn luma(px: &[u8], i: usize) -> f64 {
    0.2126 * px[i * 3] as f64 + 0.7152 * px[i * 3 + 1] as f64 + 0.0722 * px[i * 3 + 2] as f64
}

/// The full cel look minus its keyline: banded light and flat albedo only.
fn toon(k: f64) -> TerrainAppearance {
    TerrainAppearance { toon_strength: k, ..TerrainAppearance::default() }
}

/// Protects: the shipped looks and the JS-parity appearance, on all three
/// render paths, against any change the toon stages could make while off.
/// The digests were measured on the tree before `toon_strength`/`toon_outline`
/// existed; a regression here means an `if toon > 0` gate leaked.
#[test]
fn default_js_reference_and_the_looks_are_bit_identical_to_the_pre_toon_tree() {
    let s = synth();
    let pinned: [(&str, TerrainAppearance, u64, u64, u64); 4] = [
        ("default", TerrainAppearance::default(), 0xb7d8_fdf5_72a0_07e1, 0x6453_156b_c54b_e4c3, 0x5b5a_5233_a7d7_694f),
        ("js_reference", TerrainAppearance::js_reference(), 0x5599_a657_c30e_4029, 0x6c8d_5b5b_8c05_f0ba, 0x730d_4ccc_05c5_da94),
        ("Natural Vibrant", TerrainAppearance::default().with_look(render::LOOK_VIBRANT), 0x3410_c3d0_bdf5_58b3, 0xbe2d_b6c1_cc3a_bb44, 0xf095_6a2c_ac79_4eec),
        ("Antique Parchment", TerrainAppearance::default().with_look(render::LOOK_ANTIQUE), 0x50d3_fb83_172e_4d90, 0x3e74_8951_f1f4_9ed3, 0x5e33_2455_552d_1a99),
    ];
    for (name, a, g, b, t) in pinned {
        assert_eq!(fnv1a(&grid(&s, &a)), g, "{name}: the screen (cell_color) render moved");
        assert_eq!(fnv1a(&bake(&s, &a)), b, "{name}: the export bake moved");
        assert_eq!(fnv1a(&tile(&s, &a)), t, "{name}: the deep-zoom tile moved");
    }
}

/// Protects: both toon keys are off in every shipped appearance -- the
/// premise of the digest test above. Literals, not `default().toon_*`.
#[test]
fn the_toon_keys_are_off_everywhere_but_the_preset() {
    for a in [TerrainAppearance::default(), TerrainAppearance::js_reference()] {
        assert_eq!(a.toon_strength, 0.0);
        assert_eq!(a.toon_outline, 0.0);
    }
}

/// Protects: the ladder's band edges and soft terminator, as literals. Flat
/// ground at `0.625` puts the four band centres at `0.125 / 0.375 / 0.625 /
/// 0.875`, the edges at `0.25 / 0.5 / 0.75`, and each edge's softening over
/// `[edge - 0.015, edge + 0.015]` (a `0.03`-wide terminator).
#[test]
fn toon_band_has_literal_edges_and_a_0_03_wide_terminator() {
    let f = 0.625;
    // Mid-band: exactly the band centre.
    assert_eq!(render::toon_band(0.625, f), 0.625);
    assert_eq!(render::toon_band(0.30, f), 0.375);
    assert_eq!(render::toon_band(0.80, f), 0.875);
    assert_eq!(render::toon_band(0.20, f), 0.125);
    // Exactly on an edge: exactly halfway between the two centres.
    assert_eq!(render::toon_band(0.5, f), 0.5);
    assert_eq!(render::toon_band(0.75, f), 0.75);
    // The terminator: flat outside +-0.015 of the edge, moving inside it.
    assert_eq!(render::toon_band(0.4849, f), 0.375);
    assert!(render::toon_band(0.4851, f) > 0.375);
    assert!(render::toon_band(0.5149, f) < 0.625);
    assert_eq!(render::toon_band(0.5151, f), 0.625);
    // The partial bands at each end join their neighbour: no fifth level.
    assert_eq!(render::toon_band(0.0, f), 0.125);
    assert_eq!(render::toon_band(1.0, f), 0.875);
}

/// Distinct plateau values of `toon_band` over a 100 001-sample `0..1` ramp:
/// runs of at least 100 identical samples.
fn plateaus(flat: f64) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::new();
    let (mut prev, mut run) = (f64::NAN, 0);
    for k in 0..=100_000 {
        let v = render::toon_band(k as f64 / 100_000.0, flat);
        if v == prev {
            run += 1;
            if run == 100 && !out.contains(&v) {
                out.push(v);
            }
        } else {
            prev = v;
            run = 1;
        }
    }
    out
}

/// Protects: exactly four light levels on a full ramp whatever the flat-ground
/// shade -- the 40° default sun (`sin 40°`), the multi-sun rig (`~0.755`, the
/// value a fixed quarter ladder put ON an edge), an exact quarter (`0.5`, the
/// 30° sun) and both extremes. And flat ground always sits at its band's centre.
#[test]
fn a_full_ramp_has_exactly_four_light_levels_for_any_sun() {
    for flat in [40f64.to_radians().sin(), 0.7549, 0.5, 0.2, 0.93, 0.625] {
        let p = plateaus(flat);
        assert_eq!(p.len(), 4, "flat {flat}: plateaus {p:?}");
        assert_eq!(render::toon_band(flat, flat), flat, "flat ground is not its band's centre at {flat}");
        // Adjacent levels are one band (0.25) apart.
        let mut q = p.clone();
        q.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for w in q.windows(2) {
            assert!((w[1] - w[0] - 0.25).abs() < 1e-12, "flat {flat}: levels {q:?}");
        }
    }
    // The ladder for the flat-at-0.625 case, as literals.
    let mut p = plateaus(0.625);
    p.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(p, vec![0.125, 0.375, 0.625, 0.875]);
}

/// Protects: `toon_band` is monotone (a brighter slope never gets a darker
/// step) and stays in `[0, 1]`.
#[test]
fn toon_band_is_monotone_and_bounded() {
    for flat in [0.1, 0.4, 40f64.to_radians().sin(), 0.9] {
        let mut prev = -1.0;
        for k in 0..=20_000 {
            let v = render::toon_band(k as f64 / 20_000.0, flat);
            assert!((0.0..=1.0).contains(&v));
            assert!(v >= prev, "flat {flat}: not monotone at s = {}", k as f64 / 20_000.0);
            prev = v;
        }
    }
}

/// Protects: the anchor is the rig's real flat-ground shade. Single sun and
/// the multidirectional rig: `sin(alt)` for every band. Multi-sun: its macro
/// band is `0.4 sin45° + 0.3 sin35° + 0.2 + 0.1`, derived here from the rig's
/// own published weights rather than read back from the function.
#[test]
fn the_ladder_is_anchored_on_the_rigs_flat_ground_shade() {
    let a = TerrainAppearance::default();
    let s40 = 40f64.to_radians().sin();
    assert!((render::toon_flat_shade(&a, 0.4, 0.4, 0.2) - s40).abs() < 1e-12);
    let mut m = TerrainAppearance::default();
    m.npr.multi_sun = true;
    let ms = 0.4 * 45f64.to_radians().sin() + 0.3 * 35f64.to_radians().sin() + 0.2 + 0.1;
    assert!((ms - 0.75491).abs() < 1e-4, "the multi-sun rig's flat shade is {ms}");
    let want = 0.4 * ms + 0.4 * s40 + 0.2 * ms;
    assert!((render::toon_flat_shade(&m, 0.4, 0.4, 0.2) - want).abs() < 1e-12);
}

/// Protects: the material sharpening's exponent (8) by its arithmetic --
/// `0.6 / 0.4` becomes `0.6^8 / (0.6^8 + 0.4^8)`, a tie stays a tie, and an
/// all-zero input comes back unchanged rather than as NaN.
#[test]
fn material_weights_sharpen_toward_the_dominant_material() {
    let w = render::toon_sharpen_weights([0.6, 0.4, 0.0, 0.0, 0.0, 0.0]);
    assert!((w[0] - 0.962_446_824_116_180_1).abs() < 1e-12, "{w:?}");
    assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    let tie = render::toon_sharpen_weights([0.0, 0.0, 0.5, 0.0, 0.5, 0.0]);
    assert_eq!((tie[2], tie[4]), (0.5, 0.5));
    assert_eq!(render::toon_sharpen_weights([0.0; 6]), [0.0; 6]);
}

/// Protects: the keyline's radius (2) and its half-coverage anti-aliased rim.
#[test]
fn the_outline_cover_is_full_inside_one_cell_half_at_two_and_nothing_beyond() {
    let at = |wx: i64, wy: i64| render::toon_outline_cover(move |dx, dy| dx == wx && dy == wy);
    assert_eq!(at(1, 0), 1.0);
    assert_eq!(at(-1, -1), 1.0);
    assert_eq!(at(0, 2), 0.5);
    assert_eq!(at(2, 1), 0.0); // sqrt(5) > 2
    assert_eq!(at(3, 0), 0.0);
    assert_eq!(render::toon_outline_cover(|_, _| false), 0.0);
    // The nearest water decides, not the first found.
    assert_eq!(render::toon_outline_cover(|dx, dy| (dx == 2 && dy == 0) || (dx == 0 && dy == -1)), 1.0);
}

/// Protects: the keyline's ink (a literal, independent of `TOON_INK`) and the
/// zero-cover identity.
#[test]
fn the_outline_blends_to_its_slate_ink() {
    let c = (200.0, 150.0, 100.0);
    assert_eq!(render::apply_toon_outline(c, 0.0, 1.0), c);
    assert_eq!(render::apply_toon_outline(c, 1.0, 0.0), c);
    assert_eq!(render::apply_toon_outline(c, 1.0, 1.0), (30.0, 34.0, 46.0));
    assert_eq!(render::apply_toon_outline(c, 0.5, 1.0), (115.0, 92.0, 73.0));
}

/// Median 3x3 luma standard deviation over windows wholly on land, away from
/// the plate frame.
fn median_local_sd(px: &[u8], s: &Synth) -> f64 {
    let mut sds = Vec::new();
    for y in 6..s.gh - 6 {
        for x in 6..s.gw - 6 {
            let mut v = Vec::with_capacity(9);
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    let i = ((y as i64 + dy) as usize) * s.gw + (x as i64 + dx) as usize;
                    if (s.field[i] as f64) < 0.42 {
                        v.clear();
                        break;
                    }
                    v.push(luma(px, i));
                }
                if v.is_empty() {
                    break;
                }
            }
            if v.len() == 9 {
                let m = v.iter().sum::<f64>() / 9.0;
                sds.push((v.iter().map(|l| (l - m) * (l - m)).sum::<f64>() / 9.0).sqrt());
            }
        }
    }
    sds.sort_by(|a, b| a.partial_cmp(b).unwrap());
    sds[sds.len() / 2]
}

/// Protects: the flat-albedo half. On flat ground the only thing that varies
/// pixel to pixel is texture noise, so its local variance must collapse.
/// `stipple_strength`, `local_contrast` and the paper grain/mottle are
/// zeroed in BOTH renders (they are the preset's job, not this stage's), so
/// the difference is `toon_strength`'s own `tt` flattening, grain fade and
/// material sharpening, and nothing else.
#[test]
fn toon_strength_flattens_flat_ground() {
    let s = flat_world();
    let quiet = TerrainAppearance { stipple_strength: 0.0, local_contrast: 0.0, paper_grain: 0.0, paper_mottle: 0.0, ..TerrainAppearance::default() };
    let base = median_local_sd(&grid(&s, &quiet), &s);
    let flat = median_local_sd(&grid(&s, &TerrainAppearance { toon_strength: 1.0, ..quiet.clone() }), &s);
    println!("flat-ground mottle (median 3x3 luma sd): default {base:.3}, toon {flat:.3}");
    assert!(base > 0.5, "the fixture has no texture to remove ({base})");
    assert!(flat < 0.25 * base, "toon mottle {flat} vs default {base}");
}

/// Protects: the banded-light half on a real render. Grey (`bio_blend` 0,
/// `relief_chroma` 0, so land is `185 * light`), frame/vignette-dominated
/// border excluded: the toon render's land lands on at most four luma values
/// (plus soft terminator pixels), where the smooth render spreads over many.
#[test]
fn a_grey_toon_render_has_at_most_four_light_levels() {
    let s = synth();
    // Everything else that modulates land luma continuously is off too:
    // occlusion, the near-channel tint and the plate-edge haze would each
    // smear a plateau into a ramp that has nothing to do with the light.
    let grey = TerrainAppearance {
        bio_blend: 0.0,
        relief_chroma: 0.0,
        ao_strength: 0.0,
        hydro_wet_strength: 0.0,
        haze_strength: 0.0,
        stipple_strength: 0.0,
        local_contrast: 0.0,
        paper_strength: 0.0,
        border_width_frac: 0.0,
        ..TerrainAppearance::default()
    };
    let count = |a: &TerrainAppearance| -> (usize, usize) {
        let px = grid(&s, a);
        let mut hist = [0usize; 256];
        let mut n = 0;
        // The centre, where the vignette is exactly 1 (`smoothstep(0.34, ..)`).
        for y in GH / 4..3 * GH / 4 {
            for x in GW / 4..3 * GW / 4 {
                let i = y * GW + x;
                if (s.field[i] as f64) >= 0.42 {
                    hist[luma(&px, i).round() as usize] += 1;
                    n += 1;
                }
            }
        }
        // Values holding at least 3% of the land: the plateaus.
        (hist.iter().filter(|&&h| h * 100 >= 3 * n).count(), hist.iter().filter(|&&h| h > 0).count())
    };
    let (smooth_plateaus, smooth_values) = count(&grey);
    let (toon_plateaus, _) = count(&TerrainAppearance { toon_strength: 1.0, ..grey.clone() });
    assert!((2..=4).contains(&toon_plateaus), "toon plateaus {toon_plateaus}");
    assert!(smooth_values > 20, "the smooth render is not smooth ({smooth_values} values, {smooth_plateaus} plateaus)");
}

/// Protects: the keyline is drawn on land within two cells of water and
/// nowhere else -- water is untouched, and inland ground is untouched.
#[test]
fn the_outline_touches_only_land_near_water() {
    let s = synth();
    let base = grid(&s, &TerrainAppearance::default());
    let inked = grid(&s, &TerrainAppearance { toon_outline: 1.0, ..TerrainAppearance::default() });
    let water = |x: i64, y: i64| x >= 0 && y >= 0 && (x as usize) < GW && (y as usize) < GH && (s.field[y as usize * GW + x as usize] as f64) < 0.42;
    let mut moved = 0;
    for y in 0..GH {
        for x in 0..GW {
            let i = y * GW + x;
            if base[i * 3..i * 3 + 3] == inked[i * 3..i * 3 + 3] {
                continue;
            }
            moved += 1;
            assert!(!water(x as i64, y as i64), "a water pixel moved at ({x}, {y})");
            let near = (-2i64..=2).any(|dy| (-2i64..=2).any(|dx| dx * dx + dy * dy <= 4 && water(x as i64 + dx, y as i64 + dy)));
            assert!(near, "an inland pixel moved at ({x}, {y})");
        }
    }
    assert!(moved > 50, "the keyline drew {moved} pixels");
}

/// Protects: the export draws the screen's keyline. `bake_rect` at the grid's
/// own size samples every cell at an integer position, where
/// `BakeFields::pixel` is `cell_color` up to its `f32` prologue
/// (`bake_raster.rs`), so with the full cel look on the two must agree within
/// one level per channel.
#[test]
fn an_export_at_grid_size_draws_the_screens_cel_look() {
    let s = synth();
    let a = TerrainAppearance { toon_strength: 1.0, toon_outline: 1.0, ..TerrainAppearance::default() };
    let screen = grid(&s, &a);
    let c = ctx(&s, a);
    let bf = BakeFields::new(&c);
    let export = render::bake_rect(&c, &bf, None, GW, GH, 0, 0, GW, GH);
    let worst = screen.iter().zip(export.iter()).map(|(p, q)| (*p as i32 - *q as i32).abs()).max().unwrap();
    assert!(worst <= 1, "export differs from screen by {worst} levels");
    // And the keyline is actually in it: the same export without it differs.
    let c0 = ctx(&s, TerrainAppearance { toon_strength: 1.0, ..TerrainAppearance::default() });
    let bf0 = BakeFields::new(&c0);
    let unlined = render::bake_rect(&c0, &bf0, None, GW, GH, 0, 0, GW, GH);
    assert!(export.chunks(3).zip(unlined.chunks(3)).filter(|(p, q)| p != q).count() > 50, "the export drew no keyline");
}

/// Protects: the tile path reaches both stages (a tile is where the owner
/// zooms in): each moves pixels on its own.
#[test]
fn the_deep_zoom_tile_draws_both_toon_stages() {
    let s = synth();
    let base = tile(&s, &TerrainAppearance::default());
    let banded = tile(&s, &toon(1.0));
    let inked = tile(&s, &TerrainAppearance { toon_outline: 1.0, ..TerrainAppearance::default() });
    let moved = |a: &[u8], b: &[u8]| a.chunks(4).zip(b.chunks(4)).filter(|(p, q)| p != q).count();
    assert!(moved(&base, &banded) > 500, "toon_strength moved {} tile pixels", moved(&base, &banded));
    assert!(moved(&base, &inked) > 20, "toon_outline moved {} tile pixels", moved(&base, &inked));
}

/// A plateau at 0.7 with a round pit, 14 cells in radius, sunk to 0.55 -- above the 0.42 sea level,
/// so the priority flood fills it as an above-sea **lake** (class 2) and there
/// is no sea anywhere.
fn lake_world() -> Synth {
    let mut s = flat_world_small();
    for y in 0..s.gh {
        for x in 0..s.gw {
            let d = ((x as f64 - 64.0).powi(2) + (y as f64 - 40.0).powi(2)).sqrt();
            if d < 14.0 {
                s.field[y * s.gw + x] = 0.55;
            }
        }
    }
    s
}

/// [`flat_world`]'s climate on the synth's own 128 x 79 grid.
fn flat_world_small() -> Synth {
    let n = GW * GH;
    Synth { gw: GW, gh: GH, field: vec![0.7; n], temperature: vec![13.0; n], rainfall: vec![0.45; n], flow: vec![3.0; n], lith: vec![0u8; n] }
}

/// Protects: the keyline follows LAKE shores too, on the screen, the export
/// and the deep-zoom tile -- the half of "coasts and lake shores" a world with
/// no sea can show on its own. The lake is the render's own classification
/// (`cartalith_civ::build_water_bodies`, which `TileFields::new` also calls).
#[test]
fn the_outline_follows_lake_shores_on_every_path() {
    let s = lake_world();
    let wb = cartalith_civ::build_water_bodies(&s.field, s.gw, s.gh, 0.42, false, Some(&s.rainfall));
    let lakes = wb.classification.iter().filter(|&&c| c == 2).count();
    assert!(lakes > 300, "the fixture's pit is not a lake ({lakes} lake cells)");
    let on = TerrainAppearance { toon_outline: 1.0, ..TerrainAppearance::default() };
    let render_grid = |a: &TerrainAppearance| {
        let c = ctx(&s, a.clone()).with_lakes(&wb.classification);
        (0..s.gw * s.gh).map(|i| render::cell_color(&c, i % s.gw, i / s.gw)).collect::<Vec<_>>()
    };
    let (g0, g1) = (render_grid(&TerrainAppearance::default()), render_grid(&on));
    let moved: Vec<usize> = (0..g0.len()).filter(|&i| g0[i] != g1[i]).collect();
    assert!(moved.len() > 40, "the lake shore drew {} keyline cells", moved.len());
    for &i in &moved {
        assert_ne!(wb.classification[i], 2, "a lake cell was inked");
    }
    // The export, at the grid's size.
    let bake_at = |a: &TerrainAppearance| {
        let c = ctx(&s, a.clone()).with_lakes(&wb.classification);
        render::bake_rect(&c, &BakeFields::new(&c), None, s.gw, s.gh, 0, 0, s.gw, s.gh)
    };
    let (b0, b1) = (bake_at(&TerrainAppearance::default()), bake_at(&on));
    assert!(b0.chunks(3).zip(b1.chunks(3)).filter(|(p, q)| p != q).count() > 40, "the export drew no lake keyline");
    // The tile, over the lake, with the tile path's own lake test.
    let tile_at = |a: &TerrainAppearance| {
        let c = ctx(&s, a.clone());
        let px = 80;
        let (x0, y0, span) = (44.0, 20.0, 40.0);
        let step = span / (px - 1) as f64;
        let t: Vec<f32> = (0..px * px)
            .map(|k| s.field[((y0 + (k / px) as f64 * step).round() as usize).min(GH - 1) * GW + ((x0 + (k % px) as f64 * step).round() as usize).min(GW - 1)])
            .collect();
        render::render_biome_tile_rgba(&c, &t, px, px, TileBounds { x: x0, y: y0, w: span, h: span }, &TileFields::new(&c, None))
    };
    let (t0, t1) = (tile_at(&TerrainAppearance::default()), tile_at(&on));
    assert!(t0.chunks(4).zip(t1.chunks(4)).filter(|(p, q)| p != q).count() > 40, "the tile drew no lake keyline");
}

/// Protects: the plate edge is not a coast. On a world with no water at all
/// the keyline must draw nothing -- screen and export alike -- so neither
/// treats the off-plate neighbours of an edge cell as water.
#[test]
fn a_world_without_water_draws_no_keyline() {
    let s = flat_world_small();
    // The plate frame off: it paints over the edge cells and would hide an
    // inked edge (measured: with the frame on, "off-plate is water" survived).
    let off = TerrainAppearance { border_width_frac: 0.0, ..TerrainAppearance::default() };
    let on = TerrainAppearance { toon_outline: 1.0, ..off.clone() };
    assert_eq!(grid(&s, &off), grid(&s, &on), "the screen inked the plate edge");
    let bake_at = |a: &TerrainAppearance| {
        let c = ctx(&s, a.clone());
        render::bake_rect(&c, &BakeFields::new(&c), None, 200, 123, 0, 0, 200, 123)
    };
    assert_eq!(bake_at(&off), bake_at(&on), "the export inked the plate edge");
}

/// Protects: the ladder is anchored on THIS rig's flat-ground shade, so flat
/// ground keeps exactly the light the smooth map gives it (band centre = flat
/// shade). Grey render, everything that varies luma off: every cell of a flat
/// world must come out identical with and without banding.
#[test]
fn banding_leaves_flat_ground_at_its_own_light() {
    let s = flat_world_small();
    let grey = TerrainAppearance {
        bio_blend: 0.0,
        relief_chroma: 0.0,
        ao_strength: 0.0,
        hydro_wet_strength: 0.0,
        detail_micro_weight: 0.0,
        ..TerrainAppearance::default()
    };
    let banded = TerrainAppearance { toon_strength: 1.0, ..grey.clone() };
    let (a, b) = (grid(&s, &grey), grid(&s, &banded));
    let worst = a.iter().zip(b.iter()).map(|(p, q)| (*p as i32 - *q as i32).abs()).max().unwrap();
    assert!(worst <= 1, "flat ground moved {worst} levels under banding");
}

/// Protects: the material sharpening, on a render. Temperature ramps
/// -12 -> +12 °C across a flat world (no relief, no texture), so snow (luma
/// ~254 on this fixture) hands over to vegetation (<= ~165) in the middle of
/// the row. The hand-over width is the number of columns whose luma lies
/// strictly inside that gap, 175..245. Measured: sharpened, 3 columns (248,
/// 236, 212, 183 -> 162); with the sharpening removed (the "material sharpen
/// not applied" mutant), the same toon render ramps 243 -> 177 over ~17.
#[test]
fn the_biome_hand_over_is_crisp_under_toon() {
    let mut s = flat_world_small();
    for y in 0..s.gh {
        for x in 0..s.gw {
            s.temperature[y * s.gw + x] = -12.0 + 24.0 * x as f32 / (s.gw - 1) as f32;
        }
    }
    let a = TerrainAppearance { toon_strength: 1.0, stipple_strength: 0.0, local_contrast: 0.0, paper_grain: 0.0, paper_mottle: 0.0, border_width_frac: 0.0, ..TerrainAppearance::default() };
    let px = grid(&s, &a);
    let y = s.gh / 2;
    // From column 16: the first dozen columns brighten from 225 toward the
    // snow plateau (the coldest snow ramp stop), which is not the hand-over.
    let row: Vec<f64> = (16..s.gw - 4).map(|x| luma(&px, y * s.gw + x)).collect();
    assert!(row.iter().any(|&v| v > 250.0) && row.iter().any(|&v| v < 170.0), "the row has no snow/vegetation hand-over");
    let width = row.iter().filter(|&&v| v > 175.0 && v < 245.0).count();
    println!("snow hand-over width under toon: {width} columns");
    assert!(width <= 5, "toon hand-over is {width} columns wide");
}
