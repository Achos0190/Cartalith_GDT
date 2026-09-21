//! **LOD-D5** (`LOD_DETAIL_SCOPE.md`) — *"scale-aware shading weights, and
//! hydrology that resolves"*.
//!
//! Four stages, one control (`TerrainAppearance::detail_scale_strength`), and
//! one argument they all share: **octaves past the simulation grid**, not
//! kilometres. The judgment call behind that substitution is recorded in
//! `render.rs`'s own LOD-D5 section header; what this file holds is the
//! consequence the scope names as a requirement — *"a test holds the zoom-1
//! weights byte-identical to today"* — plus the two acceptance bars that can
//! be measured without a display.
//!
//! # The shape of this file
//!
//! - **The identity**, three ways: at `detail_scale_strength == 0.0`, at one
//!   tile pixel per coarse cell, and on the grid path at any strength. Each
//!   is asserted on real rendered bytes, not on the curve alone.
//! - **The curves**, each against an independently computed literal rather
//!   than against the constant it is made of (`MISTAKES.md`: *"assert a
//!   literal, or the independent thing the value must equal"*).
//! - **Load-bearing**, the negative control: a deep tile must move when the
//!   control moves, or every identity above is an identity between two
//!   nothings.
//! - **Acceptance bar 1** (research Test E), on `render::tile_river_seeds`:
//!   the channel-component count rises with zoom, and every seed carries at
//!   least the threshold's discharge.
//!
//! Acceptance bar 2 (*"detail per screen pixel is non-decreasing from zoom 4
//! to 40"*) is **not here**: it is a statement about synthesized pyramid
//! tiles at real levels, and it lives beside the pyramid in
//! `src/lod_bridge.rs`'s own test module, where `lod_sweep::detail_per_pixel`
//! — the LOD-D0 harness's own statistic — is reachable. Bar 3 (*"pops stay at
//! zero"*) is a framebuffer measurement and belongs to `_lodsweep_probe.gd`.

#[path = "../src/render.rs"]
mod render;

use render::{RenderCtx, TerrainAppearance, TileBounds, TileFields};

const GW: usize = 256;
const GH: usize = 256;
const SEA: f64 = 0.42;
const MAP_WIDTH_KM: f64 = 800.0;

/// A world with real relief and a **dendritic** flow field whose branches sit
/// at three discharges chosen against this world's own thresholds, so a
/// threshold that moves has something unambiguous to reveal.
///
/// `river_flow_thresh(256, 256, 256, 800)` is `26.2144` (`256·256·0.0004`,
/// with `terrain_detail_k` and `river_coarse_ease` both at their `1.0` floor
/// at this cell size and map width — asserted in the sweep rather than
/// assumed), and `render::RIVER_TILE_THRESH_FLOOR` is `4`. So the per-tile
/// threshold sweeps `26.2144 → 4` and no further, and the three tiers are
/// placed to fall inside it one at a time:
///
/// | tier | discharge | appears at |
/// |---|---|---|
/// | trunk | `2400` | every depth, including the map's own threshold |
/// | branch | `12` | once the threshold is under 12 — half a cell per pixel |
/// | tributary | `5` | only at the floor — a quarter cell per pixel and in |
///
/// against a `1.0` background, which is what `compute_flow` seeds per cell and
/// is therefore what the floor's own doc comment is written about. The
/// branches stop **six cells short** of the trunk and sit three cells clear of
/// each other, so each is its own connected component at every resolution —
/// a gap that closes under bilinear sampling would turn the component count
/// into a measurement of the sampler.
struct Synth {
    field: Vec<f32>,
    temperature: Vec<f32>,
    rainfall: Vec<f32>,
    flow: Vec<f32>,
}

fn synth() -> Synth {
    let mut field = vec![0f32; GW * GH];
    let mut temperature = vec![0f32; GW * GH];
    let mut rainfall = vec![0f32; GW * GH];
    let mut flow = vec![1f32; GW * GH];
    for y in 0..GH {
        for x in 0..GW {
            let i = y * GW + x;
            let (xf, yf) = (x as f64, y as f64);
            // Ridges across a bowl, entirely above sea level: every pixel of
            // every tile below takes the land branch, so a comparison cannot
            // pass by both sides drawing ocean.
            let ridge = (xf * 0.21).sin() * (yf * 0.17).cos();
            let fine = (xf * 0.61 + yf * 0.47).sin() * 0.06;
            field[i] = (0.62 + 0.14 * ridge + fine) as f32;
            temperature[i] = (14.0 - 9.0 * (yf / GH as f64)) as f32;
            rainfall[i] = (0.30 + 0.45 * ((xf * 0.07).sin() * 0.5 + 0.5)) as f32;
        }
    }
    // The trunk: one row across the middle.
    let trunk = GH / 2;
    for x in 0..GW {
        flow[trunk * GW + x] = 2400.0;
    }
    // First-order branches every 12 columns, stopping six cells short of the
    // trunk (see the struct's own note on why the gap is load-bearing).
    for bx in (6..GW).step_by(12) {
        for y in (trunk / 2)..(trunk - 6) {
            flow[y * GW + bx] = 12.0;
        }
    }
    // Their own tributaries, every 6 columns, shorter and shallower.
    for bx in (3..GW).step_by(6) {
        for y in (trunk / 2 + 8)..(trunk - 6) {
            if flow[y * GW + bx] < 10.0 {
                flow[y * GW + bx] = 5.0;
            }
        }
    }
    Synth { field, temperature, rainfall, flow }
}

fn ctx_for(s: &Synth, a: TerrainAppearance) -> RenderCtx<'_> {
    RenderCtx::with_appearance(&s.field, &s.temperature, &s.rainfall, Some(&s.flow), GW, GH, SEA, false, 55.0, 5.0, a).with_map_scale(MAP_WIDTH_KM)
}

/// The appearance the milestone's stages are visible under: every one of the
/// four needs its own carrier stage switched on, or three of the four are
/// measuring a disabled slider rather than a scale curve.
fn look() -> TerrainAppearance {
    TerrainAppearance { crest_strength: 0.45, sdf_rivers: 0.55, ..TerrainAppearance::default() }
}

/// One tile of this fixture, **with the lake mask off**.
///
/// `without_lakes` is the renderer's own `state.viz.showLakes === false` path,
/// and it is used here for a measured reason rather than a stylistic one: this
/// fixture's field is a product of two sinusoids, so it is a lattice of closed
/// depressions, and with no cell below sea level `build_water_bodies` has no
/// ocean to drain them into — **every** hollow pools. The first version of
/// this file left lakes on, and the whole 64x64 deep tile took the lake branch
/// (`0 of 4096 pixels` moved when the control moved, first pixel `108, 165,
/// 161`), which is water colour reached before `land_color` and therefore
/// before every stage this milestone adds. A test measuring a land stage has
/// to reach land.
fn draw(s: &Synth, a: TerrainAppearance, w: usize, h: usize, b: TileBounds) -> Vec<u8> {
    let ctx = ctx_for(s, a);
    let tile = tile_height(s, w, h, b);
    let tf = TileFields::new(&ctx, None).without_lakes();
    let out = render::render_biome_tile_rgba(&ctx, &tile, w, h, b, &tf);
    assert_eq!(out.len(), w * h * 4, "the tile came back empty -- every comparison below would pass vacuously");
    out
}

/// A tile of height over `b`, **with sub-cell relief the coarse field does not
/// carry** — which is the input LOD-D5's micro band exists to read. Bilinear
/// of the coarse field plus a high-frequency term whose wavelength is a
/// fraction of a coarse cell, so the residual is non-zero exactly where a real
/// `add_zoom_detail` tile's is and exactly zero where its is (`b` on integer
/// cells at one pixel per cell).
fn tile_height(s: &Synth, w: usize, h: usize, b: TileBounds) -> Vec<f32> {
    let mut t = vec![0f32; w * h];
    let cx = b.w / (w.max(2) - 1) as f64;
    let cy = b.h / (h.max(2) - 1) as f64;
    for y in 0..h {
        let wy = b.y + y as f64 * cy;
        for x in 0..w {
            let wx = b.x + x as f64 * cx;
            let base = bilinear(&s.field, wx, wy);
            // Zero at integer coordinates in both axes, by construction:
            // `sin(pi * n) == 0` is not exact in floating point, so the term
            // is written as a product of fractional parts instead, which is
            // exactly zero when either coordinate is a whole cell.
            let (fx, fy) = (wx - wx.floor(), wy - wy.floor());
            let detail = 0.020 * (fx * (1.0 - fx)) * (fy * (1.0 - fy)) * 16.0 * ((wx * 11.0).sin() + (wy * 13.0).cos());
            t[y * w + x] = (base + detail) as f32;
        }
    }
    t
}

fn bilinear(f: &[f32], wx: f64, wy: f64) -> f64 {
    let fx = wx.clamp(0.0, GW as f64 - 1.001);
    let fy = wy.clamp(0.0, GH as f64 - 1.001);
    let (x0, y0) = (fx as usize, fy as usize);
    let (x1, y1) = ((x0 + 1).min(GW - 1), (y0 + 1).min(GH - 1));
    let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
    let a = f[y0 * GW + x0] as f64 * (1.0 - tx) + f[y0 * GW + x1] as f64 * tx;
    let c = f[y1 * GW + x0] as f64 * (1.0 - tx) + f[y1 * GW + x1] as f64 * tx;
    a * (1.0 - ty) + c * ty
}

fn differing(a: &[u8], b: &[u8]) -> usize {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter()).filter(|(x, y)| x != y).count()
}

// ===========================================================================
// The identity — three ways, each on rendered bytes
// ===========================================================================

/// `detail_scale_strength == 0.0` must be the pre-milestone tile **exactly**,
/// at any depth.
///
/// This is the property `js_reference()` relies on, and
/// `golden_parity_tile_biome.rs`'s two reference goldens are the other half of
/// it (they render at `js_reference()`, which carries `0.0`). What this adds
/// is the *deep* case: those goldens run one fixture at one scale, and a curve
/// that was the identity only near grid resolution would still pass them.
#[test]
fn the_control_at_zero_is_the_pre_milestone_tile_at_every_depth() {
    let s = synth();
    // Four depths spanning five octaves, including one **outside** grid
    // resolution (a level-0 parent, where `u` is negative) — the arm that a
    // test walking only deep tiles cannot reach.
    for (label, b) in [
        ("coarser than the grid", TileBounds { x: 0.0, y: 0.0, w: (GW - 1) as f64, h: (GH - 1) as f64 }),
        ("one pixel per cell", TileBounds { x: 96.0, y: 96.0, w: 63.0, h: 63.0 }),
        ("two px per cell", TileBounds { x: 100.0, y: 100.0, w: 31.5, h: 31.5 }),
        ("sixteen px per cell", TileBounds { x: 110.0, y: 110.0, w: 3.9375, h: 3.9375 }),
    ] {
        let off = draw(&s, TerrainAppearance { detail_scale_strength: 0.0, ..look() }, 64, 64, b);
        // The same render through a `TerrainAppearance` that predates the
        // field cannot be constructed, so the reference point is the `0.0`
        // itself; what makes this more than a tautology is the load-bearing
        // test below, which shows the `1.0` render is a different picture.
        let again = draw(&s, TerrainAppearance { detail_scale_strength: 0.0, ..look() }, 64, 64, b);
        assert_eq!(differing(&off, &again), 0, "{label}: the render is not deterministic");
        // The curve itself, at the same depth: all four stages must be their
        // own identity, which is what the byte comparison above cannot
        // distinguish from "the stages are all switched off".
        let cpp = (b.w / 63.0 * (b.h / 63.0)).sqrt();
        let a0 = TerrainAppearance { detail_scale_strength: 0.0, ..look() };
        assert_eq!(
            render::scaled_detail_weights(&a0, render::detail_scale_u(cpp)),
            (a0.detail_macro_weight, a0.detail_meso_weight, a0.detail_micro_weight),
            "{label}: the weights moved at strength 0"
        );
        assert_eq!(render::crest_step_for_scale(cpp, 0.0, 64), 1, "{label}: the crest stencil moved at strength 0");
        let base = 37.5;
        assert_eq!(render::tile_river_thresh(base, cpp, 0.0), base, "{label}: the river threshold moved at strength 0");
    }
}

/// At **one tile pixel per coarse cell** every stage is the identity whatever
/// the control says — the scope's own requirement, *"a test holds the zoom-1
/// weights byte-identical to today"*, asserted on the rendered bytes and not
/// just on the weights.
///
/// This is the check that makes `golden_parity_tile_biome.rs`'s
/// `screen_identity_at_one_pixel_per_cell_with_the_shading_held_equal`
/// unaffected by the milestone rather than merely close to unaffected, and it
/// is here as well as there because that one runs at `js_reference()` (where
/// the control is `0.0`) and so cannot see this.
#[test]
fn at_grid_resolution_the_control_changes_nothing() {
    let s = synth();
    // `w - 1` cells across `w` pixels is exactly one cell per pixel, and the
    // bounds sit on integer cells so every sample lands on a cell centre.
    let b = TileBounds { x: 96.0, y: 96.0, w: 63.0, h: 63.0 };
    let off = draw(&s, TerrainAppearance { detail_scale_strength: 0.0, ..look() }, 64, 64, b);
    let on = draw(&s, TerrainAppearance { detail_scale_strength: 1.0, ..look() }, 64, 64, b);
    let moved = differing(&off, &on);
    assert_eq!(moved, 0, "the control moved {moved} bytes of a tile drawn at one pixel per coarse cell");
    // And the reason, separately, so a failure above says which stage.
    assert_eq!(render::detail_scale_u(1.0), 0.0, "the curve's argument is not zero at grid resolution");
    let a = look();
    assert_eq!(
        render::scaled_detail_weights(&a, 0.0),
        (a.detail_macro_weight, a.detail_meso_weight, a.detail_micro_weight),
        "the weights are not the appearance's own at grid resolution"
    );
    assert_eq!(render::crest_step_for_scale(1.0, 1.0, 64), 1, "the crest stencil is not one pixel at grid resolution");
    assert_eq!(render::tile_river_thresh(37.5, 1.0, 1.0), 37.5, "the river threshold is not the map's at grid resolution");
}

/// The grid path must not see this milestone at all.
///
/// `cell_color` passes `land_color`'s `scale` argument a literal `None`, so a
/// full screen render at `detail_scale_strength = 1.0` has to be byte-for-byte
/// what it is at `0.0`. Without this, the control would be a silent
/// re-baseline of every default-appearance render in the tree — which is
/// exactly what `LOD_DETAIL_SCOPE.md`'s *"nothing moves an existing golden"*
/// forbids and what `golden_parity_render.rs` would only catch at
/// `js_reference()`.
#[test]
fn the_screen_render_cannot_see_the_control() {
    let s = synth();
    let screen = |k: f64| -> Vec<u8> {
        let ctx = ctx_for(&s, TerrainAppearance { detail_scale_strength: k, ..look() });
        let mut bytes = vec![0u8; GW * GH * 3];
        for y in 0..GH {
            for x in 0..GW {
                let (r, g, b) = render::cell_color(&ctx, x, y);
                let o = (y * GW + x) * 3;
                bytes[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
                bytes[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
                bytes[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
            }
        }
        bytes
    };
    let a = screen(0.0);
    assert!(a.iter().any(|&v| v != a[0]), "the screen render is a flat colour -- this comparison would pass vacuously");
    let moved = differing(&a, &screen(1.0));
    assert_eq!(moved, 0, "the scale control reached the grid path: {moved} bytes moved");
}

// ===========================================================================
// Load-bearing — the negative control for every identity above
// ===========================================================================

/// A deep tile must be a **different picture** with the control up.
///
/// Every test above asserts that something did not move. If the control were
/// wired to nothing they would all pass, and this is the one that fails in
/// that case. It also names, per stage, what the milestone is worth on this
/// fixture rather than asserting only the total — so a regression in one of
/// the four does not hide behind the other three.
#[test]
fn a_deep_tile_moves_when_the_control_moves() {
    let s = synth();
    // Sixteen tile pixels per coarse cell — four octaves in, two thirds of
    // the way along a curve that saturates at six.
    let b = TileBounds { x: 110.0, y: 110.0, w: 3.9375, h: 3.9375 };
    let off = draw(&s, TerrainAppearance { detail_scale_strength: 0.0, ..look() }, 64, 64, b);
    let on = draw(&s, TerrainAppearance { detail_scale_strength: 1.0, ..look() }, 64, 64, b);
    let moved_px = off.chunks(4).zip(on.chunks(4)).filter(|(p, q)| p[..3] != q[..3]).count();
    println!("LOD-D5 at 16 px/cell: {moved_px} of {} pixels move", 64 * 64);
    assert!(moved_px * 4 > 64 * 64, "the control moved {moved_px} of {} pixels; it is not reaching the tile", 64 * 64);

    // Per stage. Each is isolated by switching OFF the carrier the other
    // three need, so a failure names one stage.
    let weights_only = TerrainAppearance { crest_strength: 0.0, sdf_rivers: 0.0, ..look() };
    let w_off = draw(&s, TerrainAppearance { detail_scale_strength: 0.0, ..weights_only.clone() }, 64, 64, b);
    let w_on = draw(&s, TerrainAppearance { detail_scale_strength: 1.0, ..weights_only }, 64, 64, b);
    assert!(differing(&w_off, &w_on) > 0, "the band re-weighting and micro band together move nothing");

    // The crest, alone: the shading bands are pinned equal on both sides by
    // making the three weights the same number, so the only thing left that
    // can move is the stencil. The micro band still moves with the control,
    // so this is asserted through the crest field itself rather than through
    // the rendered bytes.
    let crest_a = render::crest_step_for_scale(1.0 / 16.0, 1.0, 64);
    assert!(crest_a > 1, "the crest stencil is still one pixel at 16 px per cell ({crest_a})");
}

// ===========================================================================
// The curves, each against an independently computed literal
// ===========================================================================

/// [`render::detail_scale_u`] is `−log2(cells_per_px) / 6`, clamped.
///
/// Asserted against hand-computed literals, not against
/// `DETAIL_SCALE_OCTAVES` — `assert_eq!(x, THE_CONSTANT)` holds for every
/// value of it (`MISTAKES.md`).
#[test]
fn the_scale_argument_counts_octaves_past_the_grid() {
    assert_eq!(render::detail_scale_u(1.0), 0.0, "grid resolution is not zero octaves");
    // Half a coarse cell per pixel is one octave in; one sixth of the way to
    // the six-octave saturation.
    assert!((render::detail_scale_u(0.5) - 1.0 / 6.0).abs() < 1e-15, "{}", render::detail_scale_u(0.5));
    // 1/64 is six octaves: exactly saturated.
    assert_eq!(render::detail_scale_u(1.0 / 64.0), 1.0);
    // Deeper still is clamped, not extrapolated.
    assert_eq!(render::detail_scale_u(1.0 / 4096.0), 1.0);
    // Coarser than the grid is negative, and clamps the other way.
    assert!((render::detail_scale_u(2.0) + 1.0 / 6.0).abs() < 1e-15);
    assert_eq!(render::detail_scale_u(4096.0), -1.0);
    // The three ways a caller can hand this nonsense.
    assert_eq!(render::detail_scale_u(0.0), 0.0);
    assert_eq!(render::detail_scale_u(-1.0), 0.0);
    assert_eq!(render::detail_scale_u(f64::NAN), 0.0);
}

/// The band weights at saturation, against numbers computed by hand from the
/// shipped `0.40 / 0.40 / 0.20`.
///
/// Half of macro's `0.40` is `0.20`; the meso:micro ratio is `2:1`, so meso
/// takes `0.1333…` and micro `0.0666…`. Written as fractions rather than as
/// `DETAIL_SHIFT_MAX * …` so the assertion is independent of the constant.
#[test]
fn the_bands_move_from_macro_to_the_fine_two_and_conserve_their_sum() {
    let a = TerrainAppearance { detail_scale_strength: 1.0, ..TerrainAppearance::default() };
    assert_eq!((a.detail_macro_weight, a.detail_meso_weight, a.detail_micro_weight), (0.40, 0.40, 0.20), "the shipped weights moved; the literals below describe the old ones");
    // The shipped strength, pinned as a literal. It is the one number that
    // decides how much of the designed curve actually ships, and a silent
    // reduction is invisible in every other assertion in this file --
    // mutation-tested: `1.0 -> 0.5` SURVIVED the whole suite until this line
    // existed, because half a curve still moves pixels, still is the identity
    // at grid resolution and still is inert under `js_reference()`.
    assert_eq!(TerrainAppearance::default().detail_scale_strength, 1.0, "the shipped scale-curve strength moved; that is a look decision, not a tuning tweak");

    let (mac, mes, mic) = render::scaled_detail_weights(&a, 1.0);
    assert!((mac - 0.20).abs() < 1e-12, "macro {mac}");
    assert!((mes - (0.40 + 0.20 * 2.0 / 3.0)).abs() < 1e-12, "meso {mes}");
    assert!((mic - (0.20 + 0.20 / 3.0)).abs() < 1e-12, "micro {mic}");
    assert!((mac + mes + mic - 1.0).abs() < 1e-12, "the sum is not conserved: {}", mac + mes + mic);

    // Pulled out, the transfer runs the other way and the sum still holds.
    let (mac2, mes2, mic2) = render::scaled_detail_weights(&a, -1.0);
    assert!((mac2 - 0.60).abs() < 1e-12, "macro out {mac2}");
    assert!(mes2 < a.detail_meso_weight && mic2 < a.detail_micro_weight, "the fine bands did not give any back");
    assert!((mac2 + mes2 + mic2 - 1.0).abs() < 1e-12, "the sum is not conserved pulled out");

    // Half strength is half the transfer — the control is a scale on the
    // curve, not a switch.
    let half = TerrainAppearance { detail_scale_strength: 0.5, ..a.clone() };
    let (mac3, ..) = render::scaled_detail_weights(&half, 1.0);
    assert!((mac3 - 0.30).abs() < 1e-12, "macro at half strength {mac3}");

    // A degenerate appearance must not produce a negative weight, which would
    // invert the light rather than dim it.
    let odd = TerrainAppearance { detail_macro_weight: 1.0, detail_meso_weight: 0.01, detail_micro_weight: 0.0, detail_scale_strength: 1.0, ..a };
    let (m4, s4, i4) = render::scaled_detail_weights(&odd, -1.0);
    assert!(m4 >= 0.0 && s4 >= 0.0 && i4 >= 0.0, "a negative band weight: {m4}, {s4}, {i4}");
}

/// The micro band's normaliser, and the residual mapping.
#[test]
fn the_micro_band_reads_the_residual_and_saturates_at_a_derived_scale() {
    // `0.14 * 0.6 = 0.084`, halved to `0.042`, geometric at `0.6` for six
    // terms: `0.042 * (1 - 0.6^6) / 0.4`. Computed here from the literals
    // `AmplifyOpts::default()` carries, so the assertion is independent of
    // the function under test.
    let want = 0.042 * (1.0 - 0.6f64.powi(6)) / 0.4;
    let got = render::zoom_detail_peak_amplitude();
    assert!((got - want).abs() < 1e-12, "peak amplitude {got}, expected {want}");
    assert!(got > 0.09 && got < 0.11, "the peak amplitude left its expected order of magnitude: {got}");

    let full = got / 4.0;
    assert_eq!(render::micro_n_from_residual(0.0, full), 0.5, "a zero residual is not neutral");
    assert_eq!(render::micro_n_from_residual(full, full), 1.0, "a full-scale residual does not saturate at 1");
    assert_eq!(render::micro_n_from_residual(-full, full), 0.0, "a full-scale negative residual does not saturate at 0");
    assert_eq!(render::micro_n_from_residual(full * 100.0, full), 1.0, "past full scale is not clamped");
    assert!((render::micro_n_from_residual(full * 0.5, full) - 0.75).abs() < 1e-15, "the mapping is not linear inside the clamp");
    assert!((render::micro_n_from_residual(-full * 0.5, full) - 0.25).abs() < 1e-15, "the mapping is not symmetric about the neutral value");
    // Absence, not a plausible value.
    assert_eq!(render::micro_n_from_residual(1.0, 0.0), 0.5);
    assert_eq!(render::micro_n_from_residual(f64::NAN, full), 0.5);
}

/// The crest stencil, in tile pixels.
#[test]
fn the_crest_stencil_keeps_a_ground_width_inside_a_tile_relative_cap() {
    // A 1024 px tile, so the cap (a thirty-second of the shorter side = 32)
    // is out of the way and the ground-unit rule itself is what is measured.
    assert_eq!(render::crest_step_for_scale(1.0, 1.0, 1024), 1, "one pixel per cell");
    assert_eq!(render::crest_step_for_scale(0.25, 1.0, 1024), 4, "four pixels per cell");
    assert_eq!(render::crest_step_for_scale(1.0 / 16.0, 1.0, 1024), 16, "sixteen pixels per cell");
    // Coarser than the grid cannot ask for less than one sample.
    assert_eq!(render::crest_step_for_scale(8.0, 1.0, 1024), 1, "a coarse tile asked for a sub-pixel stencil");
    // Half strength is the square root of the full ground width, which is
    // what an exponent of `k` means; `sqrt(16) = 4`.
    assert_eq!(render::crest_step_for_scale(1.0 / 16.0, 0.5, 1024), 4, "the control is not the exponent");
    assert_eq!(render::crest_step_for_scale(f64::NAN, 1.0, 1024), 1);

    // The cap is a thirty-second of the tile's shorter side, asserted as
    // arithmetic on the tile size rather than against the constant: 256 -> 8,
    // 1024 -> 32, and a tile too small for the rule still gets one sample.
    assert_eq!(render::crest_step_for_scale(1.0 / 4096.0, 1.0, 256), 8, "a TILE_PX tile's stencil is not capped at 8");
    assert_eq!(render::crest_step_for_scale(1.0 / 4096.0, 1.0, 1024), 32, "a 1024 px tile's stencil is not capped at 32");
    assert_eq!(render::crest_step_for_scale(1.0 / 4096.0, 1.0, 16), 1, "a 16 px tile got a stencil wider than a thirty-second of itself");
    assert_eq!(render::crest_step_for_scale(1.0 / 4096.0, 1.0, 0), 1, "a zero-size tile did not fall back to one sample");
    // The skirt the cap exists to bound: `step` pixels of each edge read a
    // clamped neighbour, so the cap holds it to 1/32 of the tile per side
    // whatever the tile's size.
    for dim in [64usize, 128, 256, 512, 1024] {
        let step = render::crest_step_for_scale(1.0 / 4096.0, 1.0, dim);
        assert!(step * 32 <= dim.max(32), "a {dim} px tile's skirt is {step} px, over a thirty-second of it");
    }
}

/// The micro band's saturation point, as the renderer actually computes it.
///
/// Asserted through `render::micro_full_scale` -- the function the render loop
/// calls -- against `zoom_detail_peak_amplitude() / 4.0` with the `4.0` as a
/// literal. Written as an inline expression in the render loop instead, the
/// headroom constant would be unreachable from any test and would survive
/// mutation silently.
#[test]
fn the_renderer_saturates_the_micro_band_a_quarter_of_the_way_to_the_peak() {
    let peak = render::zoom_detail_peak_amplitude();
    assert!((render::micro_full_scale() - peak / 4.0).abs() < 1e-15, "micro_full_scale {} against peak/4 {}", render::micro_full_scale(), peak / 4.0);
    // And that the renderer reaches saturation on a real residual: the
    // fixture's sub-cell term peaks near 0.038 (measured), well over this.
    assert!(render::micro_full_scale() < 0.030, "the saturation point {} is above the residuals this fixture produces", render::micro_full_scale());
}

/// The per-tile river threshold, against hand-computed literals.
#[test]
fn the_river_threshold_falls_with_the_square_of_the_pixel() {
    let base = 400.0;
    assert_eq!(render::tile_river_thresh(base, 1.0, 1.0), base, "grid resolution is not the map's own threshold");
    // A quarter of a cell per pixel is a sixteenth of the ground area.
    assert!((render::tile_river_thresh(base, 0.25, 1.0) - 25.0).abs() < 1e-12, "{}", render::tile_river_thresh(base, 0.25, 1.0));
    // Past the floor it stops: `400 * (1/16)^2 = 1.5625`, which is under it.
    assert_eq!(render::tile_river_thresh(base, 1.0 / 16.0, 1.0), 4.0, "the threshold went below the floor");
    // Pulling out cannot raise it above the map's.
    assert_eq!(render::tile_river_thresh(base, 8.0, 1.0), base, "a coarse tile raised the threshold above the map's");
    // A world whose own threshold is already under the floor is not raised.
    assert_eq!(render::tile_river_thresh(2.0, 0.01, 1.0), 2.0, "a low-threshold world was raised to the floor");
    // Half strength halves the exponent: `400 * 0.25^1 = 100`.
    assert!((render::tile_river_thresh(base, 0.25, 0.5) - 100.0).abs() < 1e-12);
    assert_eq!(render::tile_river_thresh(0.0, 0.25, 1.0), 0.0, "a threshold that was never attached was invented");
}

/// `build_crest`'s new `step` parameter is the identity at `1` **on the field
/// itself**, not only on the finished screen.
///
/// `golden_parity_tile_biome.rs::build_crest_at_unit_scale_leaves_the_screen_unchanged`
/// holds the screen path; this holds the tile path, where the step is the
/// thing that moves. Asserted by rendering the same tile at a scale where the
/// step is 1 and at a strength of 0 and 1 — the crest cannot differ, because
/// its stencil is the same.
#[test]
fn a_unit_step_is_the_stencil_the_crest_always_had() {
    let s = synth();
    // **1.4 pixels per coarse cell**, not two: at two the ground-unit step is
    // `round(2) = 2` and this test would be measuring the stencil it is
    // trying to hold still. `45 / 63 = 0.714` cells per pixel, whose
    // reciprocal `1.4` rounds to `1` — the stencil the crest has always had,
    // while `u = -log2(0.714)/6 = 0.081` is non-zero, so the other three
    // stages ARE live here and are pinned by the appearance instead.
    // (Measured by the assertion below, which is what turned the first
    // version of this test red.)
    assert_eq!(render::crest_step_for_scale(45.0 / 63.0, 1.0, 64), 1, "the fixture's premise is wrong: the step is not 1 at 1.4 px/cell");
    assert!(render::detail_scale_u(45.0 / 63.0) > 0.0, "the fixture sits at grid resolution, where everything is the identity anyway");
    let b = TileBounds { x: 100.0, y: 100.0, w: 45.0, h: 45.0 };
    let only_crest = TerrainAppearance {
        crest_strength: 0.45,
        sdf_rivers: 0.0,
        // The bands pinned by **switching the fine two off**, not by making
        // all three equal. Equal weights do NOT pin the blend: the transfer
        // still moves weight from macro to meso, and `sh` and `sh_m` are
        // different numbers, so the blend moves. `meso + micro == 0` takes
        // `scaled_detail_weights`' degenerate early return instead, which
        // returns the appearance's own three untouched -- and a zero micro
        // weight takes the micro band out of `sh_combined` with it.
        // (Measured: with equal weights this test failed on real bytes.)
        detail_macro_weight: 1.0,
        detail_meso_weight: 0.0,
        detail_micro_weight: 0.0,
        ..TerrainAppearance::default()
    };
    let off = draw(&s, TerrainAppearance { detail_scale_strength: 0.0, ..only_crest.clone() }, 64, 64, b);
    let on = draw(&s, TerrainAppearance { detail_scale_strength: 1.0, ..only_crest }, 64, 64, b);
    let moved = differing(&off, &on);
    assert_eq!(moved, 0, "{moved} bytes moved at a scale where the crest stencil, the bands and the micro band are all pinned");
}

// ===========================================================================
// Acceptance bar 1 — research Test E, on the river seeds
// ===========================================================================

/// **The scope's first acceptance bar**: *"In a named river valley, the count
/// of distinct channel components in view rises with zoom, and every drawn
/// channel pixel has sampled discharge ≥ the threshold."*
///
/// # How "with zoom" is held, and why the obvious reading is wrong
///
/// The first version of this test zoomed the way a camera does — same centre,
/// **shrinking** ground span — and its own negative control caught it: with
/// the milestone switched off entirely the component count still moved
/// (`[1, 1, 1, 2]`), because a smaller footprint is a different piece of the
/// world and cuts a different set of channels. A count taken that way measures
/// the view's content, not the renderer's threshold.
///
/// What a deeper pyramid level actually does is hold the **ground** and
/// spend more pixels on it: the map's total tile pixels are `TILE_PX · 2^z`
/// per axis, so one fixed footprint is covered by twice the pixels at each
/// level down. That is the sweep here — one footprint over the valley, drawn
/// at 33, 65, 129 and 257 pixels, so `cells_per_px` halves each time and the
/// content is byte-for-byte the same ground at every step.
///
/// Components are 4-connected runs of `render::tile_river_seeds`' mask — the
/// renderer's own seeds, through the renderer's own sampler and threshold, not
/// a second derivation of either.
#[test]
fn the_channel_count_rises_with_zoom_and_every_seed_carries_its_discharge() {
    let s = synth();
    let a = TerrainAppearance { detail_scale_strength: 1.0, ..look() };
    let ctx = ctx_for(&s, a);
    // The world's own threshold, from the canonical function rather than read
    // off the context -- the independent thing the tile's threshold must
    // equal at grid resolution.
    let map_thresh = cartalith_hydrology::river_flow_thresh(GW, GH, GW, MAP_WIDTH_KM);
    println!("river_flow_thresh at {GW}x{GH}, {MAP_WIDTH_KM} km = {map_thresh}");
    // The fixture's premise, asserted rather than assumed: the trunk is above
    // the map's own threshold and the other two tiers are below it, or the
    // sweep has nothing to reveal.
    assert!(map_thresh > 12.0 && map_thresh < 2400.0, "the fixture's tiers do not straddle this world's threshold ({map_thresh})");

    // One footprint over the junction band, at four resolutions.
    let b = TileBounds { x: 100.0, y: 100.0, w: 32.0, h: 32.0 };
    let mut counts = Vec::new();
    for px in [33usize, 65, 129, 257] {
        let (mask, thresh) = render::tile_river_seeds(&ctx, px, px, b);
        assert_eq!(mask.len(), px * px, "the seed mask came back empty at {px} px");
        let n = components(&mask, px, px);
        let seeds = mask.iter().filter(|&&v| v == 1).count();
        println!("{px:>4} px over 32 cells: cells/px {:.4}, thresh {thresh:>9.4}, seeds {seeds:>6}, components {n}", 32.0 / (px - 1) as f64);
        assert!(seeds > 0, "no seeds at {px} px -- the component count would be vacuous");
        // Half two of the bar, on the renderer's own mask, both ways round:
        // every seed is above the threshold AND every non-seed is not, so a
        // mask that simply set everything could not pass.
        let (cx, cy) = (b.w / (px - 1) as f64, b.h / (px - 1) as f64);
        for (i, &m) in mask.iter().enumerate() {
            let (x, y) = (i % px, i / px);
            let f = sampled_flow(&s, b.x + x as f64 * cx, b.y + y as f64 * cy);
            if m == 1 {
                assert!(f > thresh, "a seed at ({x}, {y}) carries {f}, at or under the threshold {thresh}");
            } else {
                assert!(f <= thresh, "a pixel at ({x}, {y}) carries {f}, over the threshold {thresh}, and was not seeded");
            }
        }
        counts.push((px, n));
    }
    for w in counts.windows(2) {
        assert!(w[1].1 >= w[0].1, "the component count FELL from {} px ({}) to {} px ({})", w[0].0, w[0].1, w[1].0, w[1].1);
    }
    assert!(counts.last().unwrap().1 > counts[0].1, "the component count did not rise at all: {counts:?}");
}

/// The negative control for the bar above — **the test that caught the
/// confound**, kept because it is the only thing that distinguishes "the
/// threshold revealed more channels" from "the sampler found more of them".
///
/// With `detail_scale_strength` at zero the threshold is the map's at every
/// resolution, so the same sweep must return the same count: the fixture's
/// trunk, and nothing else, four times.
#[test]
fn without_the_control_the_channel_count_does_not_rise() {
    let s = synth();
    let ctx = ctx_for(&s, TerrainAppearance { detail_scale_strength: 0.0, ..look() });
    let b = TileBounds { x: 100.0, y: 100.0, w: 32.0, h: 32.0 };
    let mut seen = Vec::new();
    for px in [33usize, 65, 129, 257] {
        let (mask, thresh) = render::tile_river_seeds(&ctx, px, px, b);
        assert_eq!(thresh, cartalith_hydrology::river_flow_thresh(GW, GH, GW, MAP_WIDTH_KM), "the threshold moved with the control at zero");
        assert!(mask.iter().any(|&v| v == 1), "no seeds at {px} px with the control at zero");
        seen.push(components(&mask, px, px));
    }
    println!("control at zero: components {seen:?}");
    assert!(seen.iter().all(|&n| n == seen[0]), "the count moved with the control at zero: {seen:?}");
}

fn sampled_flow(s: &Synth, wx: f64, wy: f64) -> f64 {
    bilinear_of(&s.flow, wx, wy)
}

fn bilinear_of(f: &[f32], wx: f64, wy: f64) -> f64 {
    let fx = wx.clamp(0.0, GW as f64 - 1.001);
    let fy = wy.clamp(0.0, GH as f64 - 1.001);
    let (x0, y0) = (fx as usize, fy as usize);
    let (x1, y1) = ((x0 + 1).min(GW - 1), (y0 + 1).min(GH - 1));
    let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
    let a = f[y0 * GW + x0] as f64 * (1.0 - tx) + f[y0 * GW + x1] as f64 * tx;
    let c = f[y1 * GW + x0] as f64 * (1.0 - tx) + f[y1 * GW + x1] as f64 * tx;
    // Through the same `f32` the renderer stores its sampled flow in, or the
    // comparison against the threshold can disagree with the mask it is
    // checking on a pixel whose two roundings straddle it.
    ((a * (1.0 - ty) + c * ty) as f32) as f64
}

/// 4-connected components of a `1`/`0` mask — the *"distinct channel
/// components in view"* the bar counts.
fn components(mask: &[u8], w: usize, h: usize) -> usize {
    let mut seen = vec![false; w * h];
    let mut n = 0;
    let mut stack = Vec::new();
    for start in 0..w * h {
        if mask[start] != 1 || seen[start] {
            continue;
        }
        n += 1;
        seen[start] = true;
        stack.push(start);
        while let Some(i) = stack.pop() {
            let (x, y) = (i % w, i / w);
            let push = |j: usize, seen: &mut Vec<bool>, stack: &mut Vec<usize>| {
                if mask[j] == 1 && !seen[j] {
                    seen[j] = true;
                    stack.push(j);
                }
            };
            if x > 0 {
                push(i - 1, &mut seen, &mut stack);
            }
            if x + 1 < w {
                push(i + 1, &mut seen, &mut stack);
            }
            if y > 0 {
                push(i - w, &mut seen, &mut stack);
            }
            if y + 1 < h {
                push(i + w, &mut seen, &mut stack);
            }
        }
    }
    n
}

// ===========================================================================
// The seam, and which of the four stages moves it
// ===========================================================================

/// **A regression measured on the live shell, one hypothesis refuted, and the
/// cause still unknown — which is what this test records.**
///
/// On the LOD-D0 harness at 512x384, seed 483920, the median per-frame seam
/// ratio goes **1.7383 with `detail_scale_strength` at 0 to 1.8315 with it at
/// 1** — 5.4%, on a bar (`<= 1.5`) that LOD-D2 left open and LOD-D3 explicitly
/// did not move. Two things are known about it and a third is not:
///
/// 1. **It is not `build_crest`'s clamped skirt.** That stencil clamps at the
///    tile border, so a ground-unit step widens the skirt from one pixel to
///    `step`, and it was the obvious suspect. Capping the step at a
///    thirty-second of the tile (6 px on that grid's 256x192 tiles, down from
///    a flat 32) and re-running the identical probe gave **1.8361** — no
///    movement at all. Refuted by measurement, not argued away.
///
/// 2. **It is not same-level tile disagreement.** That is what this test
///    measures — the reference's own v1.29 statistic, the shared-edge
///    difference against the interior column-to-column spread, on two tiles
///    that share a column four octaves past grid resolution. The control
///    *closes* that seam slightly rather than opening it (printed below;
///    measured 1.4760 -> 1.2900 with every stage on), and with the river band
///    taken out of both arms it still closes it (1.4760 -> 1.3005), while with
///    the crest taken out it is flat (1.5294 -> 1.5309). So the improvement
///    such as it is comes from the crest, and no stage opens a same-level
///    seam.
///
/// 3. **What the shell is measuring instead is not established.** Its
///    statistic is taken across tile boundaries in the finished framebuffer,
///    where tiles of *adjacent levels* are on screen together and LOD-D3's
///    morph is blending them — a different question from two same-level tiles
///    agreeing. The remaining untested candidate is that the per-tile river
///    SDF is tile-local by construction (`render_biome_tile_rgba` distance-
///    transforms its own sampled flow, so a channel just outside the tile is
///    invisible to it — the reference's own design at 11683), so a lower
///    threshold puts more seeds near more borders. **That is a suspicion and
///    is labelled one**: the measurement that would settle it is a
///    per-stage sweep on the windowed harness, which needs a probe flag the
///    harness does not have.
///
/// The bar was already failing before this milestone at 1.74 against `<= 1.5`,
/// so this is a degradation of an open bar rather than a newly broken one —
/// stated so it is neither hidden nor inflated.
#[test]
fn which_stage_moves_the_shared_column_between_two_tiles() {
    let s = synth();
    // Two tiles sharing a column, four octaves past grid resolution, over the
    // fixture's channel junction so the river stage has seeds to place.
    const TW: usize = 64;
    let span = 3.9375;
    let left = TileBounds { x: 100.0, y: 110.0, w: span, h: span };
    let right = TileBounds { x: 100.0 + span, y: 110.0, w: span, h: span };

    let seam_of = |a: TerrainAppearance| -> f64 {
        let l = draw(&s, a.clone(), TW, TW, left);
        let r = draw(&s, a, TW, TW, right);
        // The shared column is the left tile's last and the right tile's
        // first -- `TileBounds` maps both to the same world coordinate, which
        // is the property `adjacent_tiles_agree_on_their_shared_column`
        // pins at unit scale.
        let lum = |b: &[u8], i: usize| 0.2126 * b[i * 4] as f64 + 0.7152 * b[i * 4 + 1] as f64 + 0.0722 * b[i * 4 + 2] as f64;
        let mut edge = 0.0;
        let mut interior = 0.0;
        for y in 0..TW {
            edge += (lum(&l, y * TW + TW - 1) - lum(&r, y * TW)).abs();
            // The interior spread the edge is judged against: the mean
            // column-to-column step over the four columns either side.
            for x in [TW - 5, TW - 4, TW - 3, TW - 2] {
                interior += (lum(&l, y * TW + x) - lum(&l, y * TW + x + 1)).abs();
            }
            for x in [0usize, 1, 2, 3] {
                interior += (lum(&r, y * TW + x) - lum(&r, y * TW + x + 1)).abs();
            }
        }
        let interior = interior / (TW * 8) as f64;
        let edge = edge / TW as f64;
        if interior <= 1e-9 { 0.0 } else { edge / interior }
    };

    let base = look();
    let off = seam_of(TerrainAppearance { detail_scale_strength: 0.0, ..base.clone() });
    let on = seam_of(TerrainAppearance { detail_scale_strength: 1.0, ..base.clone() });
    let no_rivers_off = seam_of(TerrainAppearance { detail_scale_strength: 0.0, sdf_rivers: 0.0, ..base.clone() });
    let no_rivers_on = seam_of(TerrainAppearance { detail_scale_strength: 1.0, sdf_rivers: 0.0, ..base.clone() });
    let no_crest_off = seam_of(TerrainAppearance { detail_scale_strength: 0.0, crest_strength: 0.0, ..base.clone() });
    let no_crest_on = seam_of(TerrainAppearance { detail_scale_strength: 1.0, crest_strength: 0.0, ..base });

    println!("seam ratio, all stages:      off {off:.4}  on {on:.4}");
    println!("seam ratio, rivers off:      off {no_rivers_off:.4}  on {no_rivers_on:.4}");
    println!("seam ratio, crest off:       off {no_crest_off:.4}  on {no_crest_on:.4}");

    // Not vacuous: there has to be a seam to measure at all.
    assert!(on > 0.0 && off > 0.0, "the seam statistic is zero; this fixture has no interior spread to judge against");

    // The finding, pinned so it cannot silently invert: with the river band
    // out of both arms, the control does not open the seam. If this ever goes
    // red the attribution above is wrong and the doc comment has to be
    // rewritten -- which is the point.
    assert!(
        no_rivers_on <= no_rivers_off * 1.05 + 0.01,
        "with the river band off, the control still opened the shared column: {no_rivers_on:.4} against {no_rivers_off:.4}"
    );
}
