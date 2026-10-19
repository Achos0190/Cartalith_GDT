//! **The "Satellite relief" look** (Ruling CI item 2, owner 2026-10-17, and its
//! 2026-10-18 addendum): the coast shelf and shore halo in the sea colour, snow
//! shed on steep ground and slope rock keyed to real degrees -- each a new
//! appearance field that is `0.0` in `default()`, in every other look and in
//! `js_reference()`. Since Ruling CU (owner, 2026-10-19) also the land tone
//! pass: `land_relief_contrast` and `land_tone_contrast`, the same kind of
//! field, and the look's re-pitched land ramps.
//!
//! What these tests protect, and why each is shaped the way it is:
//! * the off state is off **by control flow**, including when the look is on
//!   but the scale it needs (map width, peak altitude) is unknown -- a render
//!   that would otherwise guess a scale must instead equal the term-off render
//!   byte for byte (`MISTAKES.md`: never encode "no value" as a plausible one);
//! * the curves' numbers, against hand-worked literals rather than the
//!   expressions under test (`MISTAKES.md`: never assert a constant against
//!   itself);
//! * the three paths that draw the sea -- the screen (`cell_color`), the export
//!   (`BakeFields::pixel`) and a deep-zoom tile -- agree under the look, which
//!   is the ruling's "tiles and export must agree";
//! * the shelf never switches the wave contours on, although the two now share
//!   one distance field.
//!
//! The shipped looks' own bytes are pinned elsewhere and are not restated here:
//! `export_raster.rs`'s `empty_stacks_render_every_consumer_byte_identical_to_head`
//! (four looks, eight consumers, plus the fingerprint), `color_space.rs`'s
//! `FINISHED_RENDER_FNV1A` and `golden_parity_render.rs`.

#[path = "../src/render.rs"]
mod render;

use render::{BakeFields, RenderCtx, TerrainAppearance, TileBounds, TileFields, LOOK_SATELLITE};

const GW: usize = 64;
const GH: usize = 48;
const SEA: f64 = 0.42;
/// One km per cell, so a modest per-cell slope is a real 10-20 degrees (see
/// [`world`]). A judgement for the fixture, not the app's scale.
const MAP_KM: f64 = 64.0;
/// The app's default peak altitude (`WorldParams`' 4000 m), handed in as
/// `WorldGen::appearance_rebased` hands it, through `Npr::peak_m`.
const PEAK_M: f64 = 4000.0;

/// The fixture: sea in the west (x < 20) deepening westward; a flat cold
/// plateau (x 20..40); a steep cold ridge (x 40..48, 0.05 height units per
/// cell) and a flat summit beyond. With `PEAK_M` and `MAP_KM` one height unit
/// is `4000 / 0.58` = 6897 m, so the ridge's 0.05 per 1 km cell is
/// `atan(0.345)` = 19 degrees and the plateau is flat. Every land cell is
/// -12 C, so the temperature-only rule makes all of it snow.
struct World {
    field: Vec<f32>,
    temp: Vec<f32>,
    rain: Vec<f32>,
}

fn world() -> World {
    let n = GW * GH;
    let (mut field, mut temp) = (vec![0f32; n], vec![0f32; n]);
    for y in 0..GH {
        for x in 0..GW {
            let i = y * GW + x;
            let h = if x < 20 {
                SEA - 0.01 * (20 - x) as f64
            } else if x < 40 {
                0.60
            } else {
                (0.60 + 0.05 * (x - 39) as f64).min(1.0)
            };
            field[i] = h as f32;
            temp[i] = if x < 20 { 8.0 } else { -12.0 };
        }
    }
    World { field, temp, rain: vec![0.5; n] }
}

/// The look as the app hands it over: the world's peak altitude filled in.
fn sat() -> TerrainAppearance {
    let mut a = TerrainAppearance::default().with_look(LOOK_SATELLITE);
    a.npr.peak_m = PEAK_M;
    a
}

/// A context with (`scale`) or without the app's map width.
fn ctx<'a>(w: &'a World, a: TerrainAppearance, scale: bool) -> RenderCtx<'a> {
    let c = RenderCtx::with_appearance(&w.field, &w.temp, &w.rain, None, GW, GH, SEA, false, 55.0, 5.0, a);
    if scale { c.with_map_scale(MAP_KM) } else { c }
}

/// `cell_color` over the grid, quantized as `build_color_texture` does.
fn screen(c: &RenderCtx) -> Vec<u8> {
    let mut out = vec![0u8; GW * GH * 3];
    for y in 0..GH {
        for x in 0..GW {
            let (r, g, b) = render::cell_color(c, x, y);
            let o = (y * GW + x) * 3;
            out[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            out[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    assert!(out.iter().any(|&v| v != 0), "an all-black render proves nothing");
    out
}

/// Rec.709 luma of pixel `(x, y)` in a [`screen`] buffer.
fn luma(b: &[u8], x: usize, y: usize) -> f64 {
    let o = (y * GW + x) * 3;
    0.2126 * b[o] as f64 + 0.7152 * b[o + 1] as f64 + 0.0722 * b[o + 2] as f64
}

/// `a` with the five Satellite relief fields at `0.0`, everything else kept.
fn terms_off(mut a: TerrainAppearance) -> TerrainAppearance {
    a.sea_shelf = 0.0;
    a.sea_shelf_km = 0.0;
    a.snow_shed = 0.0;
    a.snow_shed_deg = 0.0;
    a.rock_slope_deg = 0.0;
    a
}

const EPS: f64 = 1e-12;

/// Protects: every new field is `0.0` everywhere but the new look -- the
/// tier's default, the reference path, and the three other named looks. A
/// non-zero here would move a shipped look's image (and its fingerprint).
#[test]
fn the_new_terms_are_off_outside_the_satellite_look() {
    let looks = [
        ("default", TerrainAppearance::default()),
        ("js_reference", TerrainAppearance::js_reference()),
        ("Natural Vibrant", TerrainAppearance::default().with_look(render::LOOK_VIBRANT)),
        ("Antique Parchment", TerrainAppearance::default().with_look(render::LOOK_ANTIQUE)),
        ("Quality tier", TerrainAppearance::default().with_look(render::LOOK_TIER)),
    ];
    for (name, a) in looks {
        for (k, v) in [
            ("sea_shelf", a.sea_shelf),
            ("sea_shelf_km", a.sea_shelf_km),
            ("snow_shed", a.snow_shed),
            ("snow_shed_deg", a.snow_shed_deg),
            ("rock_slope_deg", a.rock_slope_deg),
            ("land_relief_contrast", a.land_relief_contrast),
            ("land_tone_contrast", a.land_tone_contrast),
        ] {
            assert_eq!(v, 0.0, "{name}: {k} must be 0.0");
        }
    }
    // And the default look is still Natural Vibrant's place: the new look is
    // listed, last, and is not the first (identity) entry.
    assert_eq!(render::LOOK_PRESETS.last(), Some(&LOOK_SATELLITE));
    assert_eq!(render::LOOK_PRESETS[0], render::LOOK_TIER);
}

/// Protects: what the look sets, as literals -- no frame, no paper, no haze,
/// the shelf at full strength over 48 km, snow shed and the degree rock knee
/// on -- and that it is layered over Natural Vibrant's palettes (one of
/// Vibrant's re-pitched ramps is read back), so a change to either is seen.
#[test]
fn the_satellite_look_sets_its_terms() {
    let a = TerrainAppearance::default().with_look(LOOK_SATELLITE);
    assert_eq!(a.paper_strength, 0.0);
    assert_eq!(a.border_width_frac, 0.0);
    assert_eq!(a.haze_strength, 0.0);
    assert_eq!(a.sea_shelf, 1.0);
    assert_eq!(a.sea_shelf_km, 48.0);
    assert_eq!(a.snow_shed, 1.0);
    assert_eq!(a.snow_shed_deg, 8.0);
    assert_eq!(a.rock_slope, 1.0);
    assert_eq!(a.rock_slope_deg, 6.0);
    // Natural Vibrant's red-desert ramp, first stop (render.rs `with_look`),
    // which the land tone pass does not re-pitch.
    assert_eq!(a.sand_red[0], (174.0, 83.0, 47.0), "the look must sit over Natural Vibrant's palettes");
    // Ruling CU's land tone pass, as literals.
    assert_eq!(a.land_relief_contrast, 0.3);
    assert_eq!(a.land_tone_contrast, 0.5);
    assert_eq!(a.bio_blend, 1.0);
    assert_eq!(a.biome_sat, 0.30);
    assert_eq!(a.relief_chroma, 0.5);
    assert_eq!(a.curve_shade, 0.5);
    assert_eq!(a.ao_strength, 0.35);
    assert_eq!(a.crest_strength, 0.25);
    assert_eq!(a.grass_temp[1], (102.0, 142.0, 55.0), "the re-pitched temperate grass");
    assert_eq!(a.wood_temp[1], (48.0, 78.0, 36.0), "the re-pitched temperate forest");
    assert_eq!(a.rock_granite[1], (134.0, 136.0, 132.0), "the re-pitched granite");
    assert_eq!(a.snow_perm[1], (206.0, 210.0, 214.0), "the re-pitched permanent snow, below white");
    // And Natural Vibrant itself does not carry them (the look is layered
    // over it, never written back into it).
    let v = TerrainAppearance::default().with_look(render::LOOK_VIBRANT);
    assert_eq!(v.grass_temp[1], (128.0, 158.0, 68.0));
    assert_eq!(v.bio_blend, 0.90);
    // The zoom-detail law is the tier's (on in the app look since 1fc9208f); the
    // look must not switch it off.
    assert_eq!(a.zoom_detail_km, TerrainAppearance::default().zoom_detail_km);
    assert!(a.zoom_detail_km > 0.0);
}

/// Protects: the measured sea ramp's shape -- a bright shore, darkening at
/// every stop, ending blue-dominant navy -- independently of its literals, so
/// a mistyped stop that breaks the order is caught.
#[test]
fn the_satellite_sea_ramp_darkens_from_shore_to_navy() {
    let r = render::SEA_RAMP_SATELLITE;
    assert!(r.len() >= 3);
    assert_eq!(r[0].0, 0.0, "the ramp must start at the shoreline");
    assert_eq!(r[r.len() - 1].0, 1.0, "the ramp must end at the shelf width");
    let y = |c: (u8, u8, u8)| 0.2126 * c.0 as f64 + 0.7152 * c.1 as f64 + 0.0722 * c.2 as f64;
    for w in r.windows(2) {
        assert!(w[1].0 > w[0].0, "stops must be strictly ordered");
        assert!(y(w[1].1) < y(w[0].1), "every stop must be darker than the one before: {:?} -> {:?}", w[0], w[1]);
    }
    let navy = r[r.len() - 1].1;
    assert!(navy.2 > navy.1 && navy.1 > navy.0, "the open sea must be blue over green over red, got {navy:?}");
    assert!(y(r[0].1) - y(navy) > 80.0, "the halo must stand well clear of the open sea");
}

/// Protects: the shed curve's numbers -- zero below 40 % of the full-shed slope,
/// half at the midpoint, the strength at and past it, strength clamped.
/// `smoothstep(4.8, 12, 8.4)` is exactly `0.5` (the midpoint), written out.
#[test]
fn snow_shed_ramps_between_forty_percent_and_the_full_slope() {
    use render::snow_shed_frac as f;
    assert_eq!(f(0.0, 1.0, 12.0), 0.0, "flat ground keeps its snow");
    assert_eq!(f(4.8, 1.0, 12.0), 0.0, "shedding starts at 40 % of the full slope, not before");
    assert!((f(8.4, 1.0, 12.0) - 0.5).abs() < EPS, "the midpoint must shed half, got {}", f(8.4, 1.0, 12.0));
    assert!((f(8.4, 0.5, 12.0) - 0.25).abs() < EPS, "the strength scales the share linearly");
    assert_eq!(f(12.0, 1.0, 12.0), 1.0);
    assert_eq!(f(60.0, 0.7, 12.0), 0.7, "past the full slope the share is the strength");
    assert_eq!(f(60.0, 2.0, 12.0), 1.0, "the strength is clamped to 1");
}

/// Protects: shedding moves snow onto rock and nothing else, keeping the
/// weights' sum -- it must never create or destroy a material share.
#[test]
fn shedding_moves_snow_to_rock_and_keeps_the_sum() {
    let w = render::Weights { snow: 0.6, rock: 0.1, sand: 0.05, wetland: 0.05, canopy: 0.1, grass: 0.1, c: 0.3, meff: 0.4, is_mangrove: false };
    let sum = |w: &render::Weights| w.snow + w.rock + w.sand + w.wetland + w.canopy + w.grass;
    let before = sum(&w);
    let s = render::shed_snow(w, 0.5);
    assert!((s.snow - 0.3).abs() < EPS, "half the snow must go, got {}", s.snow);
    assert!((s.rock - 0.4).abs() < EPS, "and land on rock, got {}", s.rock);
    assert!((sum(&s) - before).abs() < EPS, "the sum must be kept");
    assert_eq!((s.sand, s.wetland, s.canopy, s.grass), (0.05, 0.05, 0.1, 0.1), "no other material may move");
}

/// Protects: the degree rock knee is the reference's own curve with the knee
/// swapped -- `0.25^1.5 = 0.125`, `0.5^1.5 = 1/(2 sqrt 2)` -- saturating at the
/// knee, scaled by the strength, and `0` for a non-positive knee.
#[test]
fn the_degree_rock_knee_is_the_reference_curve() {
    use render::rock_slope_mix_deg as m;
    assert!((m(2.5, 10.0, 1.0) - 0.125).abs() < EPS);
    assert!((m(5.0, 10.0, 1.0) - 1.0 / (2.0 * std::f64::consts::SQRT_2)).abs() < EPS);
    assert_eq!(m(10.0, 10.0, 1.0), 1.0);
    assert!((m(100.0, 10.0, 0.1) - 0.1).abs() < EPS, "the inner min(1, .) must saturate before the strength");
    assert_eq!(m(5.0, 0.0, 1.0), 0.0, "no knee, no tint");
    assert_eq!(m(0.0, 10.0, 1.0), 0.0, "flat ground takes no rock");
}

/// Protects: without the scale the terms need, the look renders exactly as the
/// same look with the terms off -- the shelf without a map width, the slope
/// stages without a peak altitude -- rather than guessing one.
#[test]
fn no_scale_means_the_terms_are_off_not_guessed() {
    let w = world();
    // No map width: all five are off.
    let on = screen(&ctx(&w, sat(), false));
    let off = screen(&ctx(&w, terms_off(sat()), false));
    assert_eq!(on, off, "with no map width the Satellite terms must be off");
    // A map width but no peak altitude: the slope stages are off. The shelf
    // is not, so compare against the look with only the slope terms zeroed.
    let mut no_peak = sat();
    no_peak.npr.peak_m = 0.0;
    let mut slope_off = no_peak.clone();
    slope_off.snow_shed = 0.0;
    slope_off.rock_slope_deg = 0.0;
    slope_off.rock_slope = 0.0;
    let mut ported_knee = no_peak.clone();
    ported_knee.snow_shed = 0.0;
    ported_knee.rock_slope_deg = 0.0;
    // `rock_slope_deg` falls back to the ported knee (not off), so the fair
    // control keeps `rock_slope` and drops only the degree knee.
    assert_eq!(screen(&ctx(&w, no_peak.clone(), true)), screen(&ctx(&w, ported_knee, true)), "with no peak altitude the degree stages must fall back, never guess");
    // And the control is a control: the shelf does move pixels here.
    assert_ne!(screen(&ctx(&w, no_peak, true)), screen(&ctx(&w, terms_off(slope_off), true)));
}

/// Protects: the shelf moves sea pixels only, brightens water beside the coast
/// against open water, and each of its two fields is load-bearing (the
/// replacement proof `appearance_tiers.rs`'s `every_tunable_is_load_bearing`
/// cites for exempting them: neither moves the image alone).
#[test]
fn the_shelf_brightens_the_coast_and_touches_only_the_sea() {
    let w = world();
    let mut no_shelf = sat();
    no_shelf.sea_shelf = 0.0;
    let on = screen(&ctx(&w, sat(), true));
    let off = screen(&ctx(&w, no_shelf, true));
    let (mut sea_moved, mut land_moved) = (0, 0);
    for y in 0..GH {
        for x in 0..GW {
            let o = (y * GW + x) * 3;
            if on[o..o + 3] != off[o..o + 3] {
                if (w.field[y * GW + x] as f64) < SEA { sea_moved += 1 } else { land_moved += 1 }
            }
        }
    }
    assert_eq!(land_moved, 0, "the shelf must never touch land");
    assert!(sea_moved > (20 * GH) / 2, "the shelf must move most of the sea, moved {sea_moved}");
    // Selected by input (distance from the coast column x = 19), never by the
    // colour under test: the cell beside land is brighter than 18 cells out.
    let y = GH / 2;
    assert!(luma(&on, 19, y) > luma(&on, 1, y) + 30.0, "beside the coast {} vs open sea {}", luma(&on, 19, y), luma(&on, 1, y));
    // Each field load-bearing: the width alone moves the shelf once it is on.
    let mut narrow = sat();
    narrow.sea_shelf_km = 4.0;
    assert_ne!(screen(&ctx(&w, narrow, true)), on, "sea_shelf_km must move the shelf");
    // ... and a zero width switches it off.
    let mut zero_km = sat();
    zero_km.sea_shelf_km = 0.0;
    let mut zero_both = sat();
    zero_both.sea_shelf_km = 0.0;
    zero_both.sea_shelf = 0.0;
    assert_eq!(screen(&ctx(&w, zero_km, true)), screen(&ctx(&w, zero_both, true)), "a zero shelf width must be off, never a division");
}

/// Protects: shedding takes snow off the steep ridge and leaves the flat
/// plateau alone (cells selected by the field's own slope, an input), and both
/// shed fields are load-bearing (each alone at 0 is the off render).
#[test]
fn snow_sheds_from_the_steep_ridge_only() {
    let w = world();
    // The slope-rock tint off on both sides: at the look's own knee it paints
    // the 19-degree ridge fully rock-coloured OVER the material mix, which
    // would hide what shedding did to the mix underneath.
    let mut shed = sat();
    shed.rock_slope = 0.0;
    let mut no_shed = shed.clone();
    no_shed.snow_shed = 0.0;
    let on = screen(&ctx(&w, shed, true));
    let off = screen(&ctx(&w, no_shed, true));
    let y = GH / 2;
    // The plateau's interior (x 24..36) is flat: identical.
    for x in 24..36 {
        let o = (y * GW + x) * 3;
        assert_eq!(on[o..o + 3], off[o..o + 3], "flat plateau cell x={x} must keep its snow");
    }
    // The ridge's interior (x 42..47; x 41's central difference still reads
    // the plateau edge) is 19 degrees, past the 8-degree full slope: darker
    // than full snow.
    for x in 42..47 {
        assert!(luma(&on, x, y) + 20.0 < luma(&off, x, y), "ridge x={x}: shed {} vs snow {}", luma(&on, x, y), luma(&off, x, y));
    }
    let mut no_deg = sat();
    no_deg.snow_shed_deg = 0.0;
    let mut no_deg_ctrl = no_deg.clone();
    no_deg_ctrl.snow_shed = 0.0;
    assert_eq!(screen(&ctx(&w, no_deg, true)), screen(&ctx(&w, no_deg_ctrl, true)), "a zero shed slope must be off");
}

/// Protects: the degree knee is load-bearing and differs from the ported
/// per-cell knee on this ground (where the ported one, `slope / 0.08`, is a
/// lighter tint at 0.05 per cell than a 10-degree knee at 19 degrees).
#[test]
fn the_degree_knee_tints_rock_where_the_ported_knee_barely_does() {
    let w = world();
    let mut a = terms_off(sat());
    a.rock_slope = 0.8;
    a.snow_shed = 0.0;
    let ported = screen(&ctx(&w, a.clone(), true));
    let mut d = a.clone();
    d.rock_slope_deg = 10.0;
    let deg = screen(&ctx(&w, d, true));
    assert_ne!(ported, deg, "rock_slope_deg must move the image");
    let y = GH / 2;
    let o = (y * GW + 43) * 3;
    assert_ne!(ported[o..o + 3], deg[o..o + 3], "the ridge must change under the degree knee");
}

/// Protects: the shelf shares `coast_d` with the wave contours and must not
/// switch them on. With waves off, the waves' own parameter (`wave_dist`, read
/// only by `apply_waves`) must move nothing; without the `npr.waves` gate in
/// the three consumers it moves the sea.
#[test]
fn the_shelf_does_not_switch_the_waves_on() {
    let w = world();
    let a = sat();
    assert!(!a.npr.waves);
    let mut b = a.clone();
    b.npr.wave_dist = 3.0;
    assert_eq!(screen(&ctx(&w, a.clone(), true)), screen(&ctx(&w, b.clone(), true)), "screen: waves drawn with waves off");
    let (ca, cb) = (ctx(&w, a.clone(), true), ctx(&w, b, true));
    let (fa, fb) = (BakeFields::new(&ca), BakeFields::new(&cb));
    for x in 0..20 {
        assert_eq!(fa.pixel(&ca, x as f64 + 0.5, 20.25), fb.pixel(&cb, x as f64 + 0.5, 20.25), "export: waves drawn with waves off at x={x}");
    }
}

/// Protects: the export draws the same Satellite relief picture as the screen
/// -- `BakeFields::pixel` at every integer cell against `cell_color`, to the
/// prologue's f32 rounding (the bound `bake_raster.rs` documents) -- so the
/// shelf distance and the degree stages reach the export, not only the screen.
#[test]
fn the_export_agrees_with_the_screen_under_the_look() {
    let w = world();
    let c = ctx(&w, sat(), true);
    let bf = BakeFields::new(&c);
    let mut worst: f64 = 0.0;
    for y in 0..GH {
        for x in 0..GW {
            let s = render::cell_color(&c, x, y);
            let e = bf.pixel(&c, x as f64, y as f64);
            worst = worst.max((s.0 - e.0).abs()).max((s.1 - e.1).abs()).max((s.2 - e.2).abs());
        }
    }
    assert!(worst < 1.0 / 255.0, "export vs screen worst {worst}");
    // Positive control: the look really is on in this context.
    assert_ne!(screen(&c), screen(&ctx(&w, terms_off(sat()), true)));
}

/// Protects: a deep-zoom tile at one pixel per cell draws the same Satellite
/// relief picture as the finished screen raster (within one quantization
/// level), the same identity `golden_parity_tile_biome.rs` holds for the
/// default look -- so the shelf and the slope stages reach the tiles. `exag 0`
/// holds the one term the reference defines differently for a tile (the meso
/// shade) equal, as there.
#[test]
fn a_tile_agrees_with_the_screen_under_the_look() {
    let w = world();
    let a = TerrainAppearance { exag: 0.0, ..sat() };
    let c = ctx(&w, a.clone(), true);
    let mut finished = screen(&c);
    let infl = render::build_grade_influence(&c, GW, GH);
    render::finish_raster(&a, &mut finished, GW, GH, false, &infl, render::ColorSpace::Srgb);
    let tf = TileFields::new(&c, Some(&screen(&c))).without_lakes();
    let tile = render::render_biome_tile_rgba(&c, &w.field, GW, GH, TileBounds { x: 0.0, y: 0.0, w: (GW - 1) as f64, h: (GH - 1) as f64 }, &tf);
    assert_eq!(tile.len(), GW * GH * 4, "the tile came back empty or mis-sized");
    let mut worst = 0u8;
    for (k, px) in tile.chunks(4).enumerate() {
        for ch in 0..3 {
            worst = worst.max(px[ch].abs_diff(finished[k * 3 + ch]));
        }
    }
    assert!(worst <= 1, "tile vs screen worst {worst} levels");
    // Positive control: the tile under the look differs from the tile without
    // its terms.
    let a0 = terms_off(a);
    let c0 = ctx(&w, a0, true);
    let tf0 = TileFields::new(&c0, Some(&screen(&c0))).without_lakes();
    let tile0 = render::render_biome_tile_rgba(&c0, &w.field, GW, GH, TileBounds { x: 0.0, y: 0.0, w: (GW - 1) as f64, h: (GH - 1) as f64 }, &tf0);
    assert_ne!(tile, tile0, "the look's terms must reach the tile");
}

/// Protects: the five fields serialize only when on, so every look that leaves
/// them at `0.0` keeps its serialized form -- and so its fingerprint, which
/// keys the deep-zoom tile cache -- exactly as before they existed; and a
/// round trip of the Satellite look keeps every value.
#[test]
fn the_new_fields_serialize_only_when_on_and_round_trip() {
    let j = serde_json::to_string(&TerrainAppearance::default()).unwrap();
    for k in ["sea_shelf", "sea_shelf_km", "snow_shed", "snow_shed_deg", "rock_slope_deg", "land_relief_contrast", "land_tone_contrast"] {
        assert!(!j.contains(&format!("\"{k}\"")), "default must not serialize {k}");
    }
    let a = TerrainAppearance::default().with_look(LOOK_SATELLITE);
    let s = serde_json::to_string(&a).unwrap();
    let back: TerrainAppearance = serde_json::from_str(&s).unwrap();
    assert_eq!((back.sea_shelf, back.sea_shelf_km, back.snow_shed, back.snow_shed_deg, back.rock_slope_deg), (1.0, 48.0, 1.0, 8.0, 6.0));
    assert_eq!((back.land_relief_contrast, back.land_tone_contrast), (0.3, 0.5));
}

/// Protects: under the shelf the reference's surf whitening is skipped (the
/// ramp's own first stop is the shore halo) and the shelf sits after the
/// tropical tint -- so at full strength the water colour does not depend on
/// the bathymetric depth or on warm-water tint, only on the distance -- while
/// without a distance the reference order (surf at the shallowest water) is
/// untouched.
#[test]
fn the_shelf_replaces_surf_and_the_warm_tint_but_not_without_a_distance() {
    let a = sat();
    let c = |depth: f64, t: f64, d: Option<f64>| render::sea_color_core(&a, depth, t, 0.5, 1.0, 1.0, d);
    // Shallowest water (surf would be at full) against water past the surf band.
    assert_eq!(c(0.0, 10.0, Some(2.0)), c(0.1, 10.0, Some(2.0)), "surf must not whiten the shelf");
    // Warm water: the tropical tint must be replaced at full strength.
    assert_eq!(c(0.1, 27.0, Some(2.0)), c(0.1, 10.0, Some(2.0)), "the warm tint must not show through the shelf");
    // Control: with no distance the surf still whitens the shallowest water.
    let (s0, s1) = (c(0.0, 10.0, None), c(0.1, 10.0, None));
    assert!(s0.0 > s1.0 + 10.0, "without the shelf the reference surf must stay, got {s0:?} vs {s1:?}");
}

/// `a` with Ruling CU's two land tone fields at `0.0`, everything else kept.
fn tone_off(mut a: TerrainAppearance) -> TerrainAppearance {
    a.land_relief_contrast = 0.0;
    a.land_tone_contrast = 0.0;
    a
}

/// Protects: the land tone curve's numbers, against hand-worked literals --
/// ground at the 120 pivot is unchanged; below it `Y - 0.3 (120 - Y) Y / 120`
/// (grey 60 -> 51), above it `Y + 0.3 (Y - 120)(255 - Y) / 135` (grey 200 ->
/// 209.777...); black and white are held (no clip: the reason the curve has
/// shoulders); a coloured pixel `(100, 50, 20)` (Rec.709 luma 58.464) goes
/// to luma `58.464 - 0.3 * 61.536 * 58.464 / 120 = 49.46989824` with its
/// channel ratios kept (hue and saturation ratio unchanged); a lit highlight
/// above 255 is untouched; and the curve is monotone across 0..255.
#[test]
fn the_land_tone_curve_is_a_luminance_pivot_contrast() {
    use render::land_tone as t;
    assert_eq!(render::LAND_TONE_PIVOT, 120.0);
    let p = t((120.0, 120.0, 120.0), 0.3);
    assert!((p.0 - 120.0).abs() < 1e-9 && (p.1 - 120.0).abs() < 1e-9 && (p.2 - 120.0).abs() < 1e-9, "the pivot must not move, got {p:?}");
    let g = t((60.0, 60.0, 60.0), 0.3);
    assert!((g.0 - 51.0).abs() < 1e-9 && (g.1 - 51.0).abs() < 1e-9, "grey 60 must go to 51, got {g:?}");
    let w = t((200.0, 200.0, 200.0), 0.3);
    assert!((w.0 - 209.777_777_777_777_8).abs() < 1e-9, "grey 200 must go to 209.78, got {w:?}");
    let c = t((100.0, 50.0, 20.0), 0.3);
    let y = 0.2126 * c.0 + 0.7152 * c.1 + 0.0722 * c.2;
    assert!((y - 49.469_898_24).abs() < 1e-9, "luma must be 49.46989824, got {y}");
    assert!((c.0 / c.1 - 2.0).abs() < 1e-12 && (c.1 / c.2 - 2.5).abs() < 1e-12, "channel ratios must be kept, got {c:?}");
    let white = t((254.9, 254.9, 254.9), 1.0);
    assert!(white.0 <= 255.0 && white.0 > 254.0, "near-white must stay below white, got {white:?}");
    assert_eq!(t((300.0, 300.0, 300.0), 0.3), (300.0, 300.0, 300.0), "a highlight above white is untouched");
    assert_eq!(t((0.0, 0.0, 0.0), 0.3), (0.0, 0.0, 0.0));
    let mut last = -1.0;
    for v in 0..=255 {
        let o = t((v as f64, v as f64, v as f64), 1.0).0;
        assert!(o >= last && o >= 0.0 && o <= 255.0 + 1e-9, "monotone and in range at {v}: {o} after {last}");
        last = o;
    }
}

/// Protects: the relief contrast's pivot is flat ground's light, as a
/// literal -- the default rig at 40 degrees gives `sin 40 = 0.642788`, and
/// `0.34 + 1.16 * 0.642788^0.85 = 1.136738` -- and it follows the band
/// weights it is handed (all-zero weights leave only the ambient).
#[test]
fn the_flat_light_pivot_is_the_curve_at_flat_ground() {
    let a = TerrainAppearance::default();
    let l = render::flat_land_light(&a, 0.40, 0.40, 0.20);
    assert!((l - 1.136_737_688_785_760_8).abs() < 1e-12, "flat light {l}");
    assert!((render::flat_land_light(&a, 0.0, 0.0, 0.0) - 0.34).abs() < EPS, "no bands, only the ambient");
}

/// Protects: the two land terms touch land only (every sea cell is
/// byte-identical with them on and off), keep flat ground's MEAN brightness
/// (the relief contrast pivots at flat light; the plateau's micro grain is
/// zero-mean, so only its spread grows), move the sloped ridge, and each is
/// load-bearing on its own. Cells are chosen by the fixture's geometry (the
/// sea columns, the flat plateau, the ridge), an input, never by the colour
/// under test.
#[test]
fn the_land_tone_terms_touch_land_only_and_pivot_at_flat_ground() {
    let w = world();
    let on = screen(&ctx(&w, sat(), true));
    let off = screen(&ctx(&w, tone_off(sat()), true));
    let y = GH / 2;
    for yy in 0..GH {
        for x in 0..20 {
            let o = (yy * GW + x) * 3;
            assert_eq!(on[o..o + 3], off[o..o + 3], "sea cell ({x},{yy}) moved");
        }
    }
    // Relief contrast alone: the plateau's mean luma within half a level
    // (8-bit rounding of each pixel), the ridge moved.
    let mut relief_only = tone_off(sat());
    relief_only.land_relief_contrast = 0.3;
    let r_on = screen(&ctx(&w, relief_only, true));
    let mean = |b: &[u8]| (24..36).map(|x| luma(b, x, y)).sum::<f64>() / 12.0;
    assert!((mean(&r_on) - mean(&off)).abs() < 0.5, "flat plateau mean moved: {} vs {}", mean(&r_on), mean(&off));
    let ridge_moved = (42..47).filter(|&x| (luma(&r_on, x, y) - luma(&off, x, y)).abs() >= 1.0).count();
    assert!(ridge_moved >= 3, "the relief contrast must move the sloped ridge, moved {ridge_moved} of 5");
    // Tone contrast alone is load-bearing and also leaves the sea alone.
    let mut tone_only = tone_off(sat());
    tone_only.land_tone_contrast = 0.3;
    let t_on = screen(&ctx(&w, tone_only, true));
    assert_ne!(t_on, off, "land_tone_contrast must move the land");
    for yy in 0..GH {
        let o = yy * GW * 3;
        assert_eq!(t_on[o..o + 20 * 3], off[o..o + 20 * 3], "land_tone_contrast must not touch the sea (row {yy})");
    }
}

/// Protects: the two land terms reach all three paths -- the export bake
/// (`BakeFields::pixel`) and a deep-zoom tile move with them as the screen
/// does -- the ruling's "tiles, screen and export must agree" for this pass.
/// The full-look agreement is `the_export_agrees_with_the_screen_under_the_look`
/// and `a_tile_agrees_with_the_screen_under_the_look`; this is their positive
/// control for the two new terms.
#[test]
fn the_land_tone_terms_reach_the_export_and_the_tile() {
    let w = world();
    let (c1, c0) = (ctx(&w, sat(), true), ctx(&w, tone_off(sat()), true));
    let (b1, b0) = (BakeFields::new(&c1), BakeFields::new(&c0));
    let moved = (20..GW).filter(|&x| b1.pixel(&c1, x as f64, (GH / 2) as f64) != b0.pixel(&c0, x as f64, (GH / 2) as f64)).count();
    assert!(moved > 10, "the export must take the land terms, moved {moved}");
    let bounds = TileBounds { x: 0.0, y: 0.0, w: (GW - 1) as f64, h: (GH - 1) as f64 };
    let tf1 = TileFields::new(&c1, Some(&screen(&c1))).without_lakes();
    let tf0 = TileFields::new(&c0, Some(&screen(&c0))).without_lakes();
    let t1 = render::render_biome_tile_rgba(&c1, &w.field, GW, GH, bounds, &tf1);
    let t0 = render::render_biome_tile_rgba(&c0, &w.field, GW, GH, bounds, &tf0);
    assert_ne!(t1, t0, "the tile must take the land terms");
}
