//! The output colour space on the render target (`LARGE_ITEM_RULINGS.md`,
//! **Colour management** — *"a colour space on the render target, threaded
//! through to the texture"*).
//!
//! The ruling carries one cost the owner stated and accepted: *"every
//! golden-parity fixture is sRGB, so this touches the one surface the parity
//! harnesses pin. Do it behind a default that leaves sRGB byte-identical, or
//! re-baseline deliberately and say so."* The first option was taken, and this
//! file is the proof rather than the claim.
//!
//! [`FINISHED_RENDER_FNV1A`] is a hash of the **whole finished raster** for a
//! deterministic synthetic world, taken through exactly the pipeline
//! `lib.rs::build_color_texture` runs — the `cell_color` loop, then
//! `apply_local_contrast`, then `apply_color_grade`. It was **measured before
//! `render::ColorSpace` existed** and pinned unchanged afterwards. A golden
//! that merely still passes is weaker evidence: those compare against a JS
//! reference at a tolerance, so a sub-tolerance drift would survive one. This
//! is a byte hash of the shipped default image and nothing survives it.

use rayon::prelude::*;

#[path = "../src/render.rs"]
mod render;

use render::{ColorSpace, RenderCtx, TerrainAppearance};

/// Non-square on purpose, matching `appearance_tiers.rs`: non-square maps are
/// real (`GENERATION_PARAMETERS.md`) and every radius in `render.rs` resolves
/// to more than its own floor at this size.
const GW: usize = 128;
const GH: usize = 79;

struct Synth {
    field: Vec<f32>,
    temperature: Vec<f32>,
    rainfall: Vec<f32>,
    flow: Vec<f32>,
    lith: Vec<u8>,
}

/// `appearance_tiers.rs`'s own synthetic world, byte for byte — deliberately
/// the same fixture, so the hash below describes the same picture that file's
/// tier and mutation tests already reason about.
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

/// `lib.rs::build_color_texture`'s own pass order, minus the two stages that
/// need state this fixture has no business inventing (the river channel tint
/// and the asset-pack icon composite). Everything the appearance system itself
/// contributes is here and in the order the shipped texture sees it.
fn finished_render(a: &TerrainAppearance) -> Vec<u8> {
    let s = synth();
    let c = ctx(&s, a.clone());
    let mut out = vec![0u8; GW * GH * 3];
    out.par_chunks_mut(GW * 3).enumerate().for_each(|(y, row)| {
        for x in 0..GW {
            let (r, g, b) = render::cell_color(&c, x, y);
            let o = x * 3;
            row[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            row[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            row[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
    });
    render::apply_local_contrast(a, &mut out, GW, GH, false);
    render::apply_color_grade(a, &mut out, &render::build_grade_influence(&c, GW, GH));
    out
}

/// FNV-1a, 64-bit. Written out rather than pulled in: this crate has no hasher
/// dependency, the algorithm is four lines, and a pinned constant is only
/// meaningful if the function that produced it is reproducible by inspection.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The finished default render, hashed. **Measured 2026-09-03, on the tree as
/// it stood before `render::ColorSpace` was written**, and unchanged by it.
const FINISHED_RENDER_FNV1A: u64 = 0x6154_1058_49e7_10d6;

#[test]
fn default_render_is_byte_identical_to_the_pre_color_space_tree() {
    assert_eq!(fnv1a(&finished_render(&TerrainAppearance::default())), FINISHED_RENDER_FNV1A);
}

/// The same hash, taken through the stage `lib.rs::build_color_texture` now
/// runs last. The default is `Srgb`, `apply_color_space` early-returns on it,
/// and the assertion is against the constant measured before the stage existed
/// — so this is the shipped path, not a re-measurement of it.
#[test]
fn srgb_leaves_the_finished_render_untouched() {
    let mut got = finished_render(&TerrainAppearance::default());
    render::apply_color_space(ColorSpace::Srgb, &mut got);
    assert_eq!(fnv1a(&got), FINISHED_RENDER_FNV1A);
}

/// Every colour an RGB8 raster can hold, in index order: pixel `i` is
/// `(i >> 16, (i >> 8) & 255, i & 255)`. 48 MiB, built once per test that wants
/// it rather than held in a static — this is the only fixture in the file that
/// is worth 16.7 M pixels, and both tests below need all of them.
fn full_cube() -> Vec<u8> {
    let mut cube = vec![0u8; (1 << 24) * 3];
    for i in 0..(1usize << 24) {
        cube[i * 3] = (i >> 16) as u8;
        cube[i * 3 + 1] = (i >> 8) as u8;
        cube[i * 3 + 2] = i as u8;
    }
    cube
}

/// Not the whole-image hash but the stronger, size-independent statement: over
/// the **entire 24-bit cube**, `Srgb` changes nothing. A hash proves the one
/// picture that happens to be pinned; this proves there is no input it could
/// have moved.
#[test]
fn srgb_is_the_identity_over_every_representable_pixel() {
    let mut cube = full_cube();
    render::apply_color_space(ColorSpace::Srgb, &mut cube);
    for i in 0..(1usize << 24) {
        let got = [cube[i * 3], cube[i * 3 + 1], cube[i * 3 + 2]];
        let want = [(i >> 16) as u8, (i >> 8) as u8, i as u8];
        assert_eq!(got, want, "sRGB moved pixel {i}");
    }
}

/// `ColorSpace::default()` is what `WorldGen` opens on and what every
/// golden-parity fixture assumes. The byte-identity guarantee is only worth
/// anything while this holds.
#[test]
fn the_default_space_is_srgb() {
    assert_eq!(ColorSpace::default(), ColorSpace::Srgb);
    assert_eq!(ColorSpace::default().name(), "sRGB");
    assert_eq!(render::COLOR_SPACES[0], "sRGB");
}

/// `name()` indexes `COLOR_SPACES` by discriminant, so the list order and the
/// enum order are one fact stored twice. This is the assertion that keeps them
/// the same fact.
#[test]
fn every_published_name_round_trips() {
    for n in render::COLOR_SPACES {
        let s = ColorSpace::from_name(n).unwrap_or_else(|| panic!("{n} is published but not parseable"));
        assert_eq!(s.name(), *n);
    }
    assert_eq!(ColorSpace::from_name("Rec. 2020"), None);
    assert_eq!(ColorSpace::from_name("srgb"), None, "names are exact, not case-folded");
}

// ---- Display P3: the three properties the transform's soundness rests on ----

/// `SRGB_TO_P3` is non-negative with rows summing to 1, so **no sRGB colour can
/// clip on the way into P3** — every output channel is a convex combination of
/// three values already in range. Asserted over the whole 24-bit cube by
/// construction: an out-of-range intermediate would have to survive the
/// `clamp`, so this checks the matrix directly instead of the pixels.
#[test]
fn the_matrix_cannot_clip() {
    for row in render::SRGB_TO_P3 {
        for v in row {
            assert!(v >= 0.0, "a negative term can drive a channel below 0: {row:?}");
        }
        let sum: f64 = row.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12, "row sums to {sum}, so white does not stay white");
    }
}

/// The consequence of the rows summing to 1: a neutral in is the same neutral
/// out, exactly, at every one of the 256 levels. The paper ground, the
/// neatlines and the grey hillshade view therefore do not move at all under
/// Display P3 — only chroma is re-encoded.
#[test]
fn neutrals_survive_display_p3_exactly() {
    let mut px: Vec<u8> = (0..=255u8).flat_map(|v| [v, v, v]).collect();
    let before = px.clone();
    render::apply_color_space(ColorSpace::DisplayP3, &mut px);
    assert_eq!(px, before);
}

/// Display P3 is **not** the identity on chroma — the whole point of the row.
/// A saturated primary is the strongest case: sRGB red is well outside what P3
/// needs to encode it, so its green and blue coordinates must move.
#[test]
fn display_p3_actually_re_encodes_chroma() {
    let mut px = vec![255u8, 0, 0, 0, 255, 0, 0, 0, 255];
    render::apply_color_space(ColorSpace::DisplayP3, &mut px);
    // Each primary comes **down** on its own channel and picks up the others.
    // That is the whole shape of a gamut-widening re-encode: P3 red is more
    // saturated than sRGB red, so less of it, plus some of the neighbours, is
    // what reproduces the same colour. Blue is the exception and the matrix
    // says why -- its row is the only one whose off-diagonal terms are both
    // zero in the *other* direction, so nothing feeds red or green from blue.
    assert_eq!(&px[0..3], &[234, 51, 35], "sRGB red");
    assert_eq!(&px[3..6], &[117, 251, 76], "sRGB green");
    assert_eq!(&px[6..9], &[0, 0, 245], "sRGB blue");
}

/// **The measured cost of doing this at 8 bits**, rather than a claim about it.
///
/// Encoding an sRGB-gamut colour into a wider container spends precision: the
/// P3 numbers for it occupy a smaller part of the range, so distinct sRGB
/// colours can land on the same P3 triple. This counts how many do, over the
/// whole 24-bit cube — the only measurement that answers the brief's question
/// (*"say plainly whether a colour space is meaningful before that row lands"*)
/// with a number instead of an opinion.
///
/// Both numbers are pinned, so they are a regression test rather than a
/// printout: an edit to the matrix, the transfer functions or the rounding rule
/// that makes the transform lossier fails here.
///
/// **The measurement, and what it means.** 8 764 261 of 16 777 216 outputs are
/// distinct — the transform collapses **47.8% of the code space**. That is
/// large, and it is not a bug: the sRGB gamut occupies roughly half the encoded
/// volume of the P3 cube, so re-encoding into the wider container necessarily
/// spends codes. Every collapse is a pair of neighbours merging, never a jump —
/// the map is monotone per channel and non-expanding, which is why the whole
/// cube moves by at most `MAX_CHANNEL_SHIFT` and nothing is displaced.
///
/// So: the **gamut** half of colour management is correct at 8 bits, and it
/// costs one level of gradient resolution — visible, if at all, as banding in
/// the smoothest washes (the bathymetric ramp, the haze), and `render_workspace
/// .gd`'s note says exactly that rather than leaving the user to find it. The
/// **working-space** half does not survive 8 bits at all, which is why
/// `ColorSpace` has no linear member; see its doc and Godot's own warning on
/// `Image::srgb_to_linear`. This is the plumbing plus a sound gamut transform,
/// and `OUTSTANDING_WORK.md` §2.5's high-precision pipeline is what buys the
/// resolution back.
#[test]
fn display_p3_costs_a_measured_share_of_the_24_bit_cube() {
    let mut cube = full_cube();
    render::apply_color_space(ColorSpace::DisplayP3, &mut cube);
    // One bit per representable output colour; 2 MiB, versus a 16.7 M-entry
    // hash set.
    let mut seen = vec![0u64; 1 << 18];
    let mut distinct = 0usize;
    let mut shift = 0i32;
    for (i, px) in cube.chunks_exact(3).enumerate() {
        let k = ((px[0] as usize) << 16) | ((px[1] as usize) << 8) | px[2] as usize;
        let (w, b) = (k >> 6, 1u64 << (k & 63));
        if seen[w] & b == 0 {
            seen[w] |= b;
            distinct += 1;
        }
        let src = [(i >> 16) as i32, ((i >> 8) & 255) as i32, (i & 255) as i32];
        for c in 0..3 {
            shift = shift.max((px[c] as i32 - src[c]).abs());
        }
    }
    // Measured 2026-09-03.
    assert_eq!(distinct, 8_764_261, "distinct outputs over the 24-bit cube");
    assert_eq!(shift, MAX_CHANNEL_SHIFT, "worst single-channel move, in levels");
}

/// The furthest any one channel moves under Display P3, over the whole cube:
/// sRGB's pure green `(0, 255, 0)`, whose P3 encoding is `(117, 251, 76)` —
/// 117 levels of red arriving on a channel that held none. Pinned beside the
/// distinct count so "how far does this move the picture" is a number in the
/// tree rather than a claim in a commit message.
const MAX_CHANNEL_SHIFT: i32 = 117;
