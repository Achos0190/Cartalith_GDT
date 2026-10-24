//! **Volcano appearance** (`VOLCANO_APPEARANCE_RESEARCH.md` §7, VA-0 to VA-4;
//! owner Ruling CW, 2026-10-24: all nine research questions at their
//! defaults).
//!
//! Two halves:
//!
//! * **Unit tests** on a synthetic two-volcano world -- a young active basalt
//!   shield and an old extinct andesitic stratocone in a humid climate -- that
//!   protect what the ruling requires: every look but Satellite relief is
//!   byte-identical with the volcano table attached (identity by control
//!   flow), non-volcanic ground under Satellite relief is byte-identical too,
//!   the screen, the export and a deep-zoom tile agree, the young cone stays
//!   bare dark rock while the old one keeps the climate's green (volcano age
//!   overriding the slope-rock warmth rule, question 6), the rock follows the
//!   edifice (basalt / andesite / tephra, question 9), and the daytime glow is
//!   on active craters only and under its luminance cap (question 5).
//! * **The VA-0 census harness** (`#[ignore]`d; run explicitly): real app
//!   worlds generated with the app's defaults, a per-edifice table measured
//!   from the FINAL height field (research G12: the stamp's `0.9 / (1 - sea)`
//!   factor was unverified end to end), and before/after zoom frames at
//!   x1/x4/x16/x64 with per-frame measures inside the edifice disc.
//!
//! ```text
//! VA_OUT=<dir> [VA_MORPH=1] [VA_ONLY=<tag part>] [VA_FRAMES=0] cargo test --release -p cartalith-godot --test volcano_appearance -- --ignored --nocapture --test-threads 1
//! ```

#[path = "../src/render.rs"]
#[allow(dead_code)]
mod render;
#[path = "../src/params.rs"]
#[allow(dead_code)]
mod params;

use std::collections::HashMap;

use cartalith_engine::bake::pyramid_tile_padded;
use cartalith_spatial::pyramid::{pyramid_dims, pyramid_level_for_zoom, pyramid_tile_bounds, ChunkId};
use cartalith_spatial::{tile_dims, Region};
use cartalith_terrain::volcano_attr::{self, Activity, VolcanoIndex};
use cartalith_terrain::{EdificeKind, PlacedEdifice, VolcanicSetting};
use render::volcano_look;
use render::{BakeFields, GridPrecompute, LithoSample, QualityTier, RenderCtx, TerrainAppearance, TileBounds, TileFields, LOOK_SATELLITE};

// ===========================================================================
// The synthetic world
// ===========================================================================

const GW: usize = 96;
const GH: usize = 64;
const SEA: f64 = 0.42;
/// One km per cell: a judgement for the fixture, so a 16-cell radius is a
/// 16 km cone and every kilometre pattern resolves at one pixel per cell.
const MAP_KM: f64 = 96.0;
const PEAK_M: f64 = 4000.0;

/// The two edifices: a young active basalt shield (draw 0.02: about 13
/// years) in the west, an old extinct arc stratocone (draw 0.9: about 1.3
/// Myr) in the east.
fn edifices() -> Vec<PlacedEdifice> {
    vec![
        PlacedEdifice { setting: VolcanicSetting::Hotspot, cx: 24.0, cy: 32.0, r: 16.0, h: 0.25, kind: Some(EdificeKind::Shield), age: 0.02, crater: None },
        PlacedEdifice { setting: VolcanicSetting::Arc, cx: 72.0, cy: 32.0, r: 16.0, h: 0.25, kind: Some(EdificeKind::Strato), age: 0.9, crater: None },
    ]
}

struct World {
    field: Vec<f32>,
    temp: Vec<f32>,
    rain: Vec<f32>,
}

/// Land at 0.55 everywhere but a sea strip in the north (y < 6), the two
/// cones added as a straight-sided profile reaching 0.25 above it (steep
/// enough to be well past the 6-degree cold knee and inside the 40-degree
/// warm one on the flanks). Warm (18 C) and humid (rain 0.6, 1 800 mm): a
/// climate that greens an old cone and in which the warmth rule alone would
/// have held a young one green too.
fn world() -> World {
    let n = GW * GH;
    let mut field = vec![0.55f32; n];
    for y in 0..GH {
        for x in 0..GW {
            let i = y * GW + x;
            if y < 6 {
                field[i] = (SEA - 0.05) as f32;
                continue;
            }
            for e in edifices() {
                let t = (x as f64 - e.cx).hypot(y as f64 - e.cy) / e.r;
                if t < 1.0 {
                    field[i] += (0.25 * (1.0 - t)) as f32;
                }
            }
        }
    }
    World { field, temp: vec![18.0; n], rain: vec![0.6; n] }
}

fn sat() -> TerrainAppearance {
    let mut a = TerrainAppearance::default().with_look(LOOK_SATELLITE);
    a.npr.peak_m = PEAK_M;
    a
}

fn ctx<'a>(w: &'a World, a: TerrainAppearance, index: Option<&'a VolcanoIndex>) -> RenderCtx<'a> {
    let c = RenderCtx::with_appearance(&w.field, &w.temp, &w.rain, None, GW, GH, SEA, false, 55.0, 5.0, a).with_map_scale(MAP_KM);
    match index {
        Some(i) => c.with_volcanoes(i),
        None => c,
    }
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

/// Mean RGB of the screen over the cells of edifice `k` with `t` in
/// `t0..t1`.
fn mean_on(buf: &[u8], idx: &VolcanoIndex, k: usize, t0: f64, t1: f64) -> (f64, f64, f64) {
    let (mut s, mut n) = ((0.0, 0.0, 0.0), 0.0);
    for y in 0..GH {
        for x in 0..GW {
            if let Some(h) = idx.winner_at(x as f64, y as f64) {
                if h.index == k && h.t >= t0 && h.t < t1 {
                    let o = (y * GW + x) * 3;
                    s.0 += buf[o] as f64;
                    s.1 += buf[o + 1] as f64;
                    s.2 += buf[o + 2] as f64;
                    n += 1.0;
                }
            }
        }
    }
    assert!(n >= 20.0, "edifice {k} band {t0}..{t1} reached only {n} cells");
    (s.0 / n, s.1 / n, s.2 / n)
}

/// Protects: the two new appearance fields are off in every look but
/// Satellite relief, and the look turns the stage on at full strength.
#[test]
fn the_volcano_stage_is_satellite_relief_only() {
    let looks = [
        ("default", TerrainAppearance::default()),
        ("js_reference", TerrainAppearance::js_reference()),
        ("Natural Vibrant", TerrainAppearance::default().with_look(render::LOOK_VIBRANT)),
        ("Antique Parchment", TerrainAppearance::default().with_look(render::LOOK_ANTIQUE)),
        ("Quality tier", TerrainAppearance::default().with_look(render::LOOK_TIER)),
    ];
    for (name, a) in &looks {
        assert_eq!(a.volcano_look, 0.0, "{name}: volcano_look must be 0.0");
    }
    assert_eq!(sat().volcano_look, 1.0);
}

/// Protects the ruling's "every other look byte-identical": attaching the
/// volcano table under every other look moves no byte, nor under Satellite
/// relief with the stage set to 0.0 -- identity by control flow. Positive
/// control: under Satellite relief it does move bytes.
#[test]
fn every_other_look_is_byte_identical_with_the_table_attached() {
    let w = world();
    let idx = VolcanoIndex::new(&edifices(), GW, GH);
    let mut off_sat = sat();
    off_sat.volcano_look = 0.0;
    for (name, a) in [
        ("default", TerrainAppearance::default()),
        ("js_reference", TerrainAppearance::js_reference()),
        ("Natural Vibrant", TerrainAppearance::default().with_look(render::LOOK_VIBRANT)),
        ("Antique Parchment", TerrainAppearance::default().with_look(render::LOOK_ANTIQUE)),
        ("Satellite, stage off", off_sat),
    ] {
        assert!(screen(&ctx(&w, a.clone(), Some(&idx))) == screen(&ctx(&w, a, None)), "{name}: the table moved bytes");
    }
    assert!(screen(&ctx(&w, sat(), Some(&idx))) != screen(&ctx(&w, sat(), None)), "positive control: the stage must reach Satellite relief");
}

/// Protects: under Satellite relief only the edifices' own cells move;
/// every cell off them (no owner) is byte-identical with and without the
/// table, so the stage cannot leak onto the surrounding ground.
#[test]
fn ground_off_the_edifices_is_byte_identical() {
    let w = world();
    let idx = VolcanoIndex::new(&edifices(), GW, GH);
    let (on, off) = (screen(&ctx(&w, sat(), Some(&idx))), screen(&ctx(&w, sat(), None)));
    let (mut moved_on, mut moved_off) = (0, 0);
    for y in 0..GH {
        for x in 0..GW {
            let o = (y * GW + x) * 3;
            let moved = on[o..o + 3] != off[o..o + 3];
            if idx.winner_at(x as f64, y as f64).is_some() {
                moved_on += usize::from(moved);
            } else {
                moved_off += usize::from(moved);
            }
        }
    }
    assert_eq!(moved_off, 0, "{moved_off} cells off every edifice moved");
    assert!(moved_on > 300, "only {moved_on} edifice cells moved");
}

/// Protects question 6 (volcano age overrides the slope-rock warmth rule):
/// in the same warm, humid climate and on the same slope, the young cone's
/// upper flank (t 0.12..0.3: the summit the last eruption resurfaced, so the
/// surface age is the edifice's own) is dark bare rock (low luminance, not
/// green) and the old cone's keeps a green cast. Before the stage, both read alike (control).
#[test]
fn a_young_cone_is_bare_and_an_old_one_green_on_the_same_slope() {
    let w = world();
    let idx = VolcanoIndex::new(&edifices(), GW, GH);
    let on = screen(&ctx(&w, sat(), Some(&idx)));
    let off = screen(&ctx(&w, sat(), None));
    let luma = |c: (f64, f64, f64)| 0.2126 * c.0 + 0.7152 * c.1 + 0.0722 * c.2;
    let young = mean_on(&on, &idx, 0, 0.12, 0.3);
    let old = mean_on(&on, &idx, 1, 0.12, 0.3);
    assert!(young.1 <= young.0 + 6.0, "the young flank must not read green: {young:?}");
    assert!(old.1 > old.0 + 10.0 && old.1 > old.2 + 10.0, "the old flank must keep its green: {old:?}");
    // The control: without the stage both flanks read green and the young
    // one is no darker than the old (it is lighter: the fixture's sun); the
    // stage must open a gap of at least 25 levels the other way.
    let (y0, o0) = (mean_on(&off, &idx, 0, 0.12, 0.3), mean_on(&off, &idx, 1, 0.12, 0.3));
    assert!(y0.1 > y0.0 + 10.0, "control: without the stage the young flank is green too ({y0:?})");
    let (gap_on, gap_off) = (luma(old) - luma(young), luma(o0) - luma(y0));
    assert!(gap_on > gap_off + 25.0, "young {young:?} must be much darker than old {old:?} (gap {gap_on:.1} vs {gap_off:.1} without the stage)");
}

/// Protects question 9 and VA-2: the rock follows the edifice -- basalt on
/// the shield, andesite on the stratocone's core, the tephra slot on its
/// outer flank, a blend between -- and the tephra index reads its own
/// palette slot.
#[test]
fn the_rock_follows_the_edifice() {
    let idx = VolcanoIndex::new(&edifices(), GW, GH);
    let a = sat();
    let at = |x: f64, y: f64| {
        let v = volcano_look::sample(&idx, x, y, 1.0, 1.0).expect("on an edifice");
        volcano_look::ground(&a, &v, 0.6, 0.5)
    };
    assert_eq!(at(28.0, 32.0).lith, LithoSample::One(1), "the shield is basalt");
    assert_eq!(at(74.0, 32.0).lith, LithoSample::One(2), "the stratocone's core is andesite");
    assert_eq!(at(84.0, 32.0).lith, LithoSample::One(volcano_look::LITH_TEPHRA), "the stratocone's flank is tephra");
    // Just outside the blend (t 0.65 > 0.62): tephra already.
    assert_eq!(at(82.4, 32.0).lith, LithoSample::One(volcano_look::LITH_TEPHRA), "t 0.65 is past the blend");
    assert!(matches!(at(80.0, 32.0).lith, LithoSample::Blend(_)), "between the two, a blend");
    assert_eq!(LithoSample::One(volcano_look::LITH_TEPHRA).ramp(&a, 0.5), volcano_look::TEPHRA_DEFAULT[1]);
}

/// Protects §6.3's cover reaching the pixel through `ground`: a
/// 1 000-year-old cinder cone's flank (one age: monogenetic) is covered in
/// 1 800 mm of rain and about half bare in 900 mm (the curve's literal
/// points: half cover at about 1 230 years at 900 mm).
#[test]
fn the_cover_follows_rainfall_on_the_flank() {
    let a = sat();
    // log10(1000) = 3 = 1 + 5.5 * age  ->  age = 2 / 5.5.
    let idx = VolcanoIndex::new(&[PlacedEdifice { setting: VolcanicSetting::Arc, cx: 20.0, cy: 20.0, r: 16.0, h: 0.25, kind: Some(EdificeKind::Cinder), age: 2.0 / 5.5, crater: None }], 64, 48);
    let v = volcano_look::sample(&idx, 29.6, 20.0, 1.0, 1.0).unwrap(); // t = 0.6
    assert!((v.log_years - 3.0).abs() < 1e-9);
    let wet = volcano_look::ground(&a, &v, 0.6, 0.5).bare;
    let mid = volcano_look::ground(&a, &v, 0.3, 0.5).bare;
    assert!(wet < 0.05, "1 800 mm: bare {wet}");
    assert!((0.3..0.7).contains(&mid), "900 mm: bare {mid}");
}

/// Protects question 5: only an active edifice glows; its glow is under the
/// luminance cap; a dormant one stains but does not glow.
#[test]
fn only_active_craters_glow_and_under_the_cap() {
    let a = sat();
    let mk = |age: f64| VolcanoIndex::new(&[PlacedEdifice { setting: VolcanicSetting::Hotspot, cx: 20.0, cy: 20.0, r: 16.0, h: 0.25, kind: Some(EdificeKind::Shield), age, crater: None }], 64, 48);
    for (age, want) in [(0.01, Activity::Active), (0.2, Activity::Dormant), (0.7, Activity::Extinct)] {
        let idx = mk(age);
        let mut any_glow = false;
        let mut max_l: f64 = 0.0;
        for k in 0..400 {
            let (x, y) = (20.0 + (k % 20) as f64 * 0.05 - 0.5, 20.0 + (k / 20) as f64 * 0.05 - 0.5);
            let v = volcano_look::sample(&idx, x, y, 1.0, 0.01).unwrap();
            assert_eq!(v.activity, want);
            if let Some(g) = volcano_look::ground(&a, &v, 0.6, 0.5).glow {
                any_glow = true;
                max_l = max_l.max(0.2126 * g.0 + 0.7152 * g.1 + 0.0722 * g.2);
            }
        }
        assert_eq!(any_glow, want == Activity::Active, "age {age}: glow {any_glow}");
        assert!(max_l <= volcano_look::GLOW_LUMA_CAP + 1e-9, "glow luminance {max_l} over the cap");
    }
}

/// Protects "tiles, screen and export must agree" under the stage: the
/// export's pixel at integer cells equals the screen within one level, and
/// a one-pixel-per-cell tile equals the finished screen within one level.
#[test]
fn screen_export_and_tile_agree_on_the_volcanoes() {
    let w = world();
    let idx = VolcanoIndex::new(&edifices(), GW, GH);
    let a = TerrainAppearance { exag: 0.0, ..sat() };
    let c = ctx(&w, a.clone(), Some(&idx));
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
    let mut finished = screen(&c);
    let infl = render::build_grade_influence(&c, GW, GH);
    render::finish_raster(&a, &mut finished, GW, GH, false, &infl, render::ColorSpace::Srgb);
    let tf = TileFields::new(&c, Some(&screen(&c))).without_lakes();
    let tile = render::render_biome_tile_rgba(&c, &w.field, GW, GH, TileBounds { x: 0.0, y: 0.0, w: (GW - 1) as f64, h: (GH - 1) as f64 }, &tf);
    let mut worst_t = 0u8;
    for (k, px) in tile.chunks(4).enumerate() {
        for ch in 0..3 {
            worst_t = worst_t.max(px[ch].abs_diff(finished[k * 3 + ch]));
        }
    }
    assert!(worst_t <= 1, "tile vs screen worst {worst_t} levels");
}

/// Protects the save of appearance documents: the two new fields serialise
/// only when changed from their defaults, so every existing look's document
/// (and the tile-cache fingerprint built from it) keeps its bytes; the
/// Satellite look's round-trips with the stage on.
#[test]
fn the_new_fields_serialise_only_when_set() {
    for a in [TerrainAppearance::default(), TerrainAppearance::js_reference(), TerrainAppearance::default().with_look(render::LOOK_VIBRANT)] {
        let j = serde_json::to_string(&a).unwrap();
        assert!(!j.contains("volcano_look") && !j.contains("rock_tephra"), "{j}");
    }
    let j = serde_json::to_string(&sat()).unwrap();
    assert!(j.contains("\"volcano_look\":1.0") && !j.contains("rock_tephra"));
    let back: TerrainAppearance = serde_json::from_str(&j).unwrap();
    assert_eq!(back.volcano_look, 1.0);
    assert_eq!(back.rock_tephra, volcano_look::TEPHRA_DEFAULT);
}

// ===========================================================================
// VA-0: the census harness on real worlds
// ===========================================================================

/// `lod_bridge::TILE_PX`, restated (as `zoom_detail_ladder.rs` does).
const TILE_PX: usize = 256;
/// `lod_bridge::MAX_LEVEL`, restated.
const MAX_LEVEL: i32 = 10;
/// The assessment's capture size (`RENDER_TARGET_GAP_ASSESSMENT_2026-10-17.md` §1).
const SCREEN_W: usize = 950;
const SCREEN_H: usize = 858;
/// Screen pixels per coarse cell at each rung (the assessment's).
const RUNGS: [(&str, f64); 3] = [("x4", 1.86), ("x16", 7.42), ("x64", 29.7)];

struct WorldSpec {
    tag: &'static str,
    seed: i32,
    archetype: Option<[f64; 5]>,
}

/// `lib.rs`'s `ARCHETYPES` Archipelago row, quoted (private to the cdylib).
const ARCHIPELAGO: [f64; 5] = [0.15, 0.90, 0.80, 0.30, 0.50];

/// The census worlds: the brief's three (71077345 dome and cone, 24601
/// archipelago, 314159) and two more seeds for the "at least five" bar.
const WORLDS: [WorldSpec; 5] = [
    WorldSpec { tag: "s71077345_classic", seed: 71077345, archetype: None },
    WorldSpec { tag: "s24601_archipelago", seed: 24601, archetype: Some(ARCHIPELAGO) },
    WorldSpec { tag: "s314159_classic", seed: 314159, archetype: None },
    WorldSpec { tag: "s483920_classic", seed: 483920, archetype: None },
    WorldSpec { tag: "s246371_classic", seed: 246371, archetype: None },
];

fn world_params(w: &WorldSpec, gw: usize, gh: usize) -> cartalith_engine::WorldParams {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = w.seed;
    p.map_width_km = 800.0;
    p.sea_level = 0.42;
    // `VA_MORPH=1`: VA-5's morphometry switch on (off at both boundaries).
    p.volc.morphometry = std::env::var("VA_MORPH").is_ok_and(|v| v == "1");
    if let Some(k) = w.archetype {
        p.world_structure = cartalith_engine::WorldStructureParams { enabled: true, continentality: k[0], fragmentation: k[1], tectonic_energy: k[2], ocean_depth: k[3], hotspot_density: k[4] };
    }
    p
}

fn tile(ctx: &RenderCtx, tf: &TileFields, z: i32, col: u32, row: u32, seed: i32) -> (Vec<u8>, usize, usize, TileBounds) {
    let (gw, gh) = (ctx.gw, ctx.gh);
    let b = pyramid_tile_bounds(gw, gh, z, col, row);
    let n = pyramid_dims(z).cols as usize;
    let d = tile_dims(&Region { x: 0, y: 0, w: gw - 1, h: gh - 1 }, n, n, TILE_PX);
    let tb = TileBounds { x: b.x, y: b.y, w: b.w, h: b.h };
    let pad = render::tile_halo_px(ctx, d.w, d.h, tb);
    let opts = render::tile_amplify_opts(ctx, seed, cartalith_terrain::amplify::z_base_for_tile_size(TILE_PX));
    let (w, h, data) = pyramid_tile_padded(ctx.field, gw, gh, ChunkId::new(z as u32, col, row), TILE_PX, pad, &opts);
    let rgba = render::render_biome_tile_rgba_water(ctx, &data, None, w, h, pad, tb, tf, None);
    assert_eq!(rgba.len(), w * h * 4, "the colouriser refused the tile");
    (rgba, w, h, tb)
}

/// The deep-zoom view at `ppc` screen pixels per cell around `(gx, gy)`, as
/// the shell composites it (child level morphed over its parent).
fn frame(ctx: &RenderCtx, tf: &TileFields, gx: f64, gy: f64, ppc: f64, seed: i32) -> Vec<u8> {
    let z = pyramid_level_for_zoom(ppc, ctx.gw as f64, TILE_PX as f64, Some(MAX_LEVEL));
    let want = f64::max(1.0, ctx.gw as f64 * ppc / TILE_PX as f64);
    let t = (want.log2() - z as f64 + 0.5).clamp(0.0, 1.0);
    let morph = t * t * (3.0 - 2.0 * t);
    let child = level_frame(ctx, tf, gx, gy, ppc, seed, z);
    if z == 0 || morph >= 1.0 {
        return child;
    }
    let parent = level_frame(ctx, tf, gx, gy, ppc, seed, z - 1);
    child.iter().zip(&parent).map(|(&c, &p)| (p as f64 + (c as f64 - p as f64) * morph).round() as u8).collect()
}

fn level_frame(ctx: &RenderCtx, tf: &TileFields, gx: f64, gy: f64, ppc: f64, seed: i32, z: i32) -> Vec<u8> {
    let (gw, gh) = (ctx.gw, ctx.gh);
    let n = pyramid_dims(z).cols;
    let step_x = (gw - 1) as f64 / n as f64;
    let step_y = (gh - 1) as f64 / n as f64;
    let mut cache: HashMap<(u32, u32), (Vec<u8>, usize, usize, TileBounds)> = HashMap::new();
    let mut out = vec![0u8; SCREEN_W * SCREEN_H * 3];
    for sy in 0..SCREEN_H {
        let cy = (gy + (sy as f64 + 0.5 - SCREEN_H as f64 / 2.0) / ppc).clamp(0.0, (gh - 1) as f64);
        let row = ((cy / step_y) as u32).min(n - 1);
        for sx in 0..SCREEN_W {
            let cx = (gx + (sx as f64 + 0.5 - SCREEN_W as f64 / 2.0) / ppc).clamp(0.0, (gw - 1) as f64);
            let col = ((cx / step_x) as u32).min(n - 1);
            let t = cache.entry((col, row)).or_insert_with(|| tile(ctx, tf, z, col, row, seed));
            let (rgba, w, h, tb) = (&t.0, t.1, t.2, t.3);
            let fx = ((cx - tb.x) / tb.w * (w - 1) as f64).clamp(0.0, (w - 1) as f64);
            let fy = ((cy - tb.y) / tb.h * (h - 1) as f64).clamp(0.0, (h - 1) as f64);
            let (x0, y0) = (fx as usize, fy as usize);
            let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
            let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
            for c in 0..3 {
                let p = |x: usize, y: usize| rgba[(y * w + x) * 4 + c] as f64;
                let v = (p(x0, y0) * (1.0 - tx) + p(x1, y0) * tx) * (1.0 - ty) + (p(x0, y1) * (1.0 - tx) + p(x1, y1) * tx) * ty;
                out[(sy * SCREEN_W + sx) * 3 + c] = v.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}

/// The base map at fit (x1): one pixel per cell (`cell_color` + the finish),
/// shown at `ppc` screen pixels per cell around `(gx, gy)`, nearest.
fn base_frame(ctx: &RenderCtx, gx: f64, gy: f64, ppc: f64) -> Vec<u8> {
    let (gw, gh) = (ctx.gw, ctx.gh);
    let mut rgb = vec![0u8; gw * gh * 3];
    for y in 0..gh {
        for x in 0..gw {
            let (r, g, b) = render::cell_color(ctx, x, y);
            let o = (y * gw + x) * 3;
            rgb[o] = (r * 255.0) as u8;
            rgb[o + 1] = (g * 255.0) as u8;
            rgb[o + 2] = (b * 255.0) as u8;
        }
    }
    let inf = render::build_grade_influence(ctx, gw, gh);
    render::finish_raster(ctx.appearance(), &mut rgb, gw, gh, ctx.world, &inf, render::ColorSpace::Srgb);
    let mut out = vec![0u8; SCREEN_W * SCREEN_H * 3];
    for sy in 0..SCREEN_H {
        let cy = gy + (sy as f64 + 0.5 - SCREEN_H as f64 / 2.0) / ppc;
        for sx in 0..SCREEN_W {
            let cx = gx + (sx as f64 + 0.5 - SCREEN_W as f64 / 2.0) / ppc;
            if cx < 0.0 || cy < 0.0 || cx >= gw as f64 || cy >= gh as f64 {
                continue;
            }
            let o = ((cy as usize) * gw + cx as usize) * 3;
            out[(sy * SCREEN_W + sx) * 3..(sy * SCREEN_W + sx) * 3 + 3].copy_from_slice(&rgb[o..o + 3]);
        }
    }
    out
}

fn write_png(path: &str, w: usize, h: usize, rgb: Vec<u8>) {
    assert!(rgb.iter().any(|&v| v != rgb[0]), "an empty frame: {path}");
    let png = cartalith_assets::raster::encode_png_rgb8(w as u32, h as u32, rgb, cartalith_assets::raster::PngColour::Data).unwrap();
    std::fs::write(path, png).unwrap();
}

/// Before and after side by side with a 6 px white gutter.
fn side_by_side(a: &[u8], b: &[u8]) -> (usize, Vec<u8>) {
    let w = SCREEN_W * 2 + 6;
    let mut out = vec![255u8; w * SCREEN_H * 3];
    for y in 0..SCREEN_H {
        out[y * w * 3..y * w * 3 + SCREEN_W * 3].copy_from_slice(&a[y * SCREEN_W * 3..(y + 1) * SCREEN_W * 3]);
        let o = y * w * 3 + (SCREEN_W + 6) * 3;
        out[o..o + SCREEN_W * 3].copy_from_slice(&b[y * SCREEN_W * 3..(y + 1) * SCREEN_W * 3]);
    }
    (w, out)
}

/// Per-frame measures inside edifice `k`'s disc (`t < 0.9`, by the index's
/// own owner rule at each screen pixel's grid position -- selected by the
/// INPUT, never by colour): pixel count, green share (`G > R + 4` and
/// `G > B + 4`), dark share (Rec.709 luma below 70), mean luma, distinct
/// colours, mean absolute luma gradient to the right/down neighbour.
fn disc_measures(buf: &[u8], idx: &VolcanoIndex, k: usize, gx: f64, gy: f64, ppc: f64) -> (usize, f64, f64, f64, usize, f64) {
    let luma = |o: usize| 0.2126 * buf[o] as f64 + 0.7152 * buf[o + 1] as f64 + 0.0722 * buf[o + 2] as f64;
    let (mut n, mut green, mut dark, mut lsum, mut grad, mut gn) = (0usize, 0usize, 0usize, 0.0, 0.0, 0usize);
    let mut colours = std::collections::HashSet::new();
    for sy in 0..SCREEN_H - 1 {
        let cy = gy + (sy as f64 + 0.5 - SCREEN_H as f64 / 2.0) / ppc;
        for sx in 0..SCREEN_W - 1 {
            let cx = gx + (sx as f64 + 0.5 - SCREEN_W as f64 / 2.0) / ppc;
            let Some(h) = idx.winner_at(cx, cy) else { continue };
            if h.index != k || h.t >= 0.9 {
                continue;
            }
            let o = (sy * SCREEN_W + sx) * 3;
            let (r, g, b) = (buf[o] as i32, buf[o + 1] as i32, buf[o + 2] as i32);
            n += 1;
            green += usize::from(g > r + 4 && g > b + 4);
            let l = luma(o);
            dark += usize::from(l < 70.0);
            lsum += l;
            colours.insert((r, g, b));
            grad += (luma(o + 3) - l).abs() + (luma(o + SCREEN_W * 3) - l).abs();
            gn += 2;
        }
    }
    let nf = n.max(1) as f64;
    (n, green as f64 / nf, dark as f64 / nf, lsum / nf, colours.len(), grad / gn.max(1) as f64)
}

/// FNV-1a 64 over `f32` bits: the field digest the VA-1 bar compares.
fn fnv(v: &[f32]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for x in v {
        for b in x.to_bits().to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
    }
    h
}

/// One edifice's morphometry from the FINAL field: a 48-bin radial median
/// profile from the centre to the toe; the rim at the profile's maximum; the
/// height above the base ring (`t` 0.95..1); mean flank slope rim to toe;
/// max flank slope between bins; crater/base as the rim's fraction (the
/// crater's diameter over the base's).
fn morphometry(field: &[f32], gw: usize, gh: usize, e: &PlacedEdifice, m_per_unit: f64, km_per_cell: f64) -> Option<(f64, f64, f64, f64, f64)> {
    // At most 48 bins, and never finer than one cell of radius each: a
    // sub-cell bin re-samples the same cells and turns a one-cell step into
    // a near-vertical "slope" (the first census run read 87 degrees).
    let bins_n = (e.r.floor() as usize).clamp(6, 48);
    #[allow(non_snake_case)]
    let BINS = bins_n;
    let mut bins: Vec<Vec<f32>> = vec![Vec::new(); BINS];
    let r = e.r;
    let (x0, x1) = ((e.cx - r).floor().max(0.0) as usize, ((e.cx + r).ceil() as usize).min(gw - 1));
    let (y0, y1) = ((e.cy - r).floor().max(0.0) as usize, ((e.cy + r).ceil() as usize).min(gh - 1));
    for y in y0..=y1 {
        for x in x0..=x1 {
            let t = (x as f64 - e.cx).hypot(y as f64 - e.cy) / r;
            if t < 1.0 {
                bins[((t * BINS as f64) as usize).min(BINS - 1)].push(field[y * gw + x]);
            }
        }
    }
    let prof: Vec<Option<f64>> = bins
        .iter_mut()
        .map(|b| {
            if b.is_empty() {
                return None;
            }
            b.sort_by(|p, q| p.total_cmp(q));
            Some(b[b.len() / 2] as f64)
        })
        .collect();
    // The base: the median of the outer ring's (t 0.9..1) bin medians -- a
    // cone on sloping ground has a base that is not one height; the median
    // is the middle of it.
    let mut ring: Vec<f64> = prof[(BINS * 9) / 10..].iter().flatten().copied().collect();
    ring.sort_by(|a, b| a.total_cmp(b));
    let base = ring.get(ring.len() / 2).copied().unwrap_or(f64::NAN);
    let (rim_k, rim_h) = prof.iter().enumerate().filter_map(|(k, v)| v.map(|v| (k, v))).max_by(|a, b| a.1.total_cmp(&b.1))?;
    if !base.is_finite() {
        return None;
    }
    let h_m = (rim_h - base) * m_per_unit;
    let w_km = 2.0 * r * km_per_cell;
    let bin_km = r * km_per_cell / BINS as f64;
    let t_rim = (rim_k as f64 + 0.5) / BINS as f64;
    let run_km = (1.0 - t_rim) * r * km_per_cell;
    let mean_deg = if run_km > 0.0 { ((rim_h - base) * m_per_unit / (run_km * 1000.0)).atan().to_degrees() } else { f64::NAN };
    let mut max_deg: f64 = 0.0;
    for k in rim_k..BINS - 1 {
        if let (Some(a), Some(b)) = (prof[k], prof[k + 1]) {
            max_deg = max_deg.max(((a - b) * m_per_unit / (bin_km * 1000.0)).atan().to_degrees());
        }
    }
    Some((h_m, w_km, mean_deg, max_deg, t_rim))
}

/// VA-0: per world, the per-edifice census from the final field; the field
/// and `volcanic_field` digests (VA-1's "field identical" bar is checked by
/// comparing these against a run of the same command at the base commit);
/// and, for up to three edifices per world (the largest, the youngest and the
/// oldest of at least 4 km radius), before/after frames at x1/x4/x16/x64
/// with the disc measures. Satellite relief at the Quality tier, as the app.
#[test]
#[ignore = "generates five 2048x1311 worlds and renders zoom frames; run explicitly with --release"]
fn volcano_census() {
    let out = std::env::var("VA_OUT").ok();
    if let Some(o) = out.as_ref() {
        std::fs::create_dir_all(o).unwrap();
    }
    let only: Option<String> = std::env::var("VA_ONLY").ok();
    let frames = std::env::var("VA_FRAMES").map(|v| v != "0").unwrap_or(true);
    let (gw, gh) = (2048usize, 1311usize);
    for w in &WORLDS {
        if only.as_ref().is_some_and(|o| !w.tag.contains(o.as_str())) {
            continue;
        }
        let p = world_params(w, gw, gh);
        let t0 = std::time::Instant::now();
        let ws = cartalith_engine::generate_terrain(&p);
        let kpc = p.map_width_km / gw as f64;
        let m_per_unit = p.peak_m / (1.0 - ws.sea_level);
        println!("== {}: generated in {:.1}s; field fnv {:016x}, volcanic_field fnv {:016x}", w.tag, t0.elapsed().as_secs_f64(), fnv(&ws.field), fnv(&ws.volcanic_field));
        let Some(vt) = ws.volcanoes.as_ref() else {
            println!("   no volcano table");
            continue;
        };
        println!("   {} edifices; m per unit {:.0}; km per cell {:.3}", vt.len(), m_per_unit, kpc);
        println!("   idx setting      kind    age01  years       activity  R_km   stampH_m finalH_m  H/W    mean_deg max_deg  crater/base");
        let mut by_kind: HashMap<&str, Vec<(f64, f64, f64)>> = HashMap::new();
        let mut act = [0usize; 3];
        for (k, e) in vt.iter().enumerate() {
            let kind = e.kind.map_or("-", |k| k.key());
            let a = volcano_attr::activity_of(e.age);
            act[a as usize] += 1;
            let stamp_h_m = e.h * m_per_unit;
            match morphometry(&ws.field, gw, gh, e, m_per_unit, kpc) {
                Some((h_m, w_km, mean_deg, max_deg, cb)) => {
                    println!(
                        "   {k:>3} {:<12} {kind:<7} {:.3}  {:>10.0}  {:<8}  {:>5.1}  {:>7.0}  {:>7.0}  {:.3}  {:>6.1}  {:>6.1}   {:.3}",
                        volcano_attr::setting_key(e.setting),
                        e.age,
                        volcano_attr::age_years(e.age),
                        a.key(),
                        e.r * kpc,
                        stamp_h_m,
                        h_m,
                        h_m / (w_km * 1000.0),
                        mean_deg,
                        max_deg,
                        cb
                    );
                    by_kind.entry(kind).or_default().push((h_m / (w_km * 1000.0), mean_deg, cb));
                }
                None => println!("   {k:>3} {:<12} {kind:<7} {:.3}  (no profile)", volcano_attr::setting_key(e.setting), e.age),
            }
        }
        println!("   activity: {} active, {} dormant, {} extinct", act[0], act[1], act[2]);
        for (kind, v) in &by_kind {
            let med = |f: &dyn Fn(&(f64, f64, f64)) -> f64| {
                let mut x: Vec<f64> = v.iter().map(f).collect();
                x.sort_by(|a, b| a.total_cmp(b));
                x[x.len() / 2]
            };
            println!("   median {kind}: H/W {:.3}, mean slope {:.1} deg, crater/base {:.3} (n {})", med(&|t| t.0), med(&|t| t.1), med(&|t| t.2), v.len());
        }
        if !frames {
            continue;
        }
        let Some(o) = out.as_ref() else { continue };
        // The look the app draws: the Quality tier under Satellite relief,
        // the world's peak altitude handed in as `appearance_rebased` does.
        let mut a_on = TerrainAppearance::for_tier(QualityTier::Quality).with_look(LOOK_SATELLITE);
        a_on.npr.peak_m = p.peak_m;
        let mut a_off = a_on.clone();
        a_off.volcano_look = 0.0;
        let lith = cartalith_civ::build_lithology(&ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, ws.sea_level);
        let idx = VolcanoIndex::new(vt, gw, gh);
        let pre_on = GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, &a_on, Some(p.map_width_km));
        let pre_off = GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, &a_off, Some(p.map_width_km));
        let c_on = RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, 55.0, 5.0, a_on.clone(), &pre_on)
            .expect("context")
            .with_lithology(&lith)
            .with_volcanoes(&idx);
        let c_off = RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, 55.0, 5.0, a_off.clone(), &pre_off)
            .expect("context")
            .with_lithology(&lith)
            .with_volcanoes(&idx);
        let (tf_on, tf_off) = (TileFields::new(&c_on, None), TileFields::new(&c_off, None));
        // The picks: largest, youngest, oldest (radius >= 4 km), on land.
        let land = |e: &PlacedEdifice| {
            let (x, y) = (e.cx as usize, e.cy as usize);
            x < gw && y < gh && ws.field[y * gw + x] as f64 > ws.sea_level
        };
        // ... and owning its own centre (an edifice buried under a younger
        // neighbour's disc has nothing of its own to show).
        let owns_centre = |k: usize| idx.winner_at(vt[k].cx, vt[k].cy).is_some_and(|h| h.index == k);
        let cand: Vec<usize> = (0..vt.len()).filter(|&k| vt[k].r * kpc >= 4.0 && land(&vt[k]) && vt[k].kind.is_some() && owns_centre(k)).collect();
        let mut picks: Vec<(&str, usize)> = Vec::new();
        if let Some(&k) = cand.iter().max_by(|&&a, &&b| vt[a].r.total_cmp(&vt[b].r)) {
            picks.push(("largest", k));
        }
        if let Some(&k) = cand.iter().min_by(|&&a, &&b| vt[a].age.total_cmp(&vt[b].age)) {
            picks.push(("youngest", k));
        }
        if let Some(&k) = cand.iter().max_by(|&&a, &&b| vt[a].age.total_cmp(&vt[b].age)) {
            picks.push(("oldest", k));
        }
        picks.dedup_by_key(|p| p.1);
        // x1: the whole map, once per state.
        let fit = SCREEN_W as f64 / gw as f64;
        let (b1_off, b1_on) = (base_frame(&c_off, gw as f64 / 2.0, gh as f64 / 2.0, fit), base_frame(&c_on, gw as f64 / 2.0, gh as f64 / 2.0, fit));
        let (sw, sbs) = side_by_side(&b1_off, &b1_on);
        write_png(&format!("{o}/{}_x1_before_after.png", w.tag), sw, SCREEN_H, sbs);
        for (what, k) in picks {
            let e = &vt[k];
            println!(
                "   frames: {what} = #{k} {} {} age {:.3} ({:.0} yr, {}) R {:.1} km at ({:.0}, {:.0})",
                volcano_attr::setting_key(e.setting),
                e.kind.map_or("-", |k| k.key()),
                e.age,
                volcano_attr::age_years(e.age),
                volcano_attr::activity_of(e.age).key(),
                e.r * kpc,
                e.cx,
                e.cy
            );
            let m1 = |b: &[u8]| disc_measures(b, &idx, k, gw as f64 / 2.0, gh as f64 / 2.0, fit);
            let (mo, mn) = (m1(&b1_off), m1(&b1_on));
            println!("     x1  before: n {} green {:.2} dark {:.2} L {:.1} colours {} grad {:.2} | after: n {} green {:.2} dark {:.2} L {:.1} colours {} grad {:.2}", mo.0, mo.1, mo.2, mo.3, mo.4, mo.5, mn.0, mn.1, mn.2, mn.3, mn.4, mn.5);
            for &(rung, ppc) in &RUNGS {
                let f_off = frame(&c_off, &tf_off, e.cx, e.cy, ppc, w.seed);
                let f_on = frame(&c_on, &tf_on, e.cx, e.cy, ppc, w.seed);
                let mm = |b: &[u8]| disc_measures(b, &idx, k, e.cx, e.cy, ppc);
                let (mo, mn) = (mm(&f_off), mm(&f_on));
                println!("     {rung:<3} before: n {} green {:.2} dark {:.2} L {:.1} colours {} grad {:.2} | after: n {} green {:.2} dark {:.2} L {:.1} colours {} grad {:.2}", mo.0, mo.1, mo.2, mo.3, mo.4, mo.5, mn.0, mn.1, mn.2, mn.3, mn.4, mn.5);
                let (sw, sbs) = side_by_side(&f_off, &f_on);
                write_png(&format!("{o}/{}_{what}_{rung}_before_after.png", w.tag), sw, SCREEN_H, sbs);
            }
        }
    }
}
