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
//! VA_OUT=<dir> [VA_MORPH=0] [VA_BEFORE=<arm>] [VA_ONLY=<tag part>] [VA_FRAMES=0] cargo test --release -p cartalith-godot --test volcano_appearance -- --ignored --nocapture --test-threads 1
//! ```
//!
//! Without `VA_BEFORE`, "before" is the same world with the volcano colours
//! off (VA-2..VA-4's own comparison). With `VA_BEFORE=<arm>` ([`before_arm`]),
//! "before" is the world generated with that Ruling CX switch turned back and
//! drawn with the colours on, so a generation-side change (item 1's
//! morphometry) is compared world against world. Both sides draw the app's
//! lakes (`water_bodies_floored`), so crater lakes show.

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
        PlacedEdifice { rift_bearing: None, setting: VolcanicSetting::Hotspot, cx: 24.0, cy: 32.0, r: 16.0, h: 0.25, kind: Some(EdificeKind::Shield), age: 0.02, years: None, crater: None },
        PlacedEdifice { rift_bearing: None, setting: VolcanicSetting::Arc, cx: 72.0, cy: 32.0, r: 16.0, h: 0.25, kind: Some(EdificeKind::Strato), age: 0.9, years: None, crater: None },
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

/// Protects owner Ruling CX item 5 through `cell_color` (the screen's own
/// lake branch): with a lake drawn in both craters, Satellite relief turns
/// the ACTIVE shield's crater lake turquoise-to-emerald (green above red by
/// 40 levels, a literal) while the EXTINCT stratocone's crater lake keeps the
/// lake palette byte for byte; every other look draws both lakes identically
/// with or without the table (identity by control flow).
#[test]
fn an_active_crater_lake_is_acid_and_only_in_satellite_relief() {
    let w = world();
    let idx = VolcanoIndex::new(&edifices(), GW, GH);
    let mut lakes = vec![0u8; GW * GH];
    for (cx, cy) in [(24usize, 32usize), (72, 32)] {
        for dy in 0..2 {
            for dx in 0..2 {
                lakes[(cy + dy) * GW + cx + dx] = 2;
            }
        }
    }
    let px = |c: &RenderCtx, x: usize, y: usize| {
        let (r, g, b) = render::cell_color(c, x, y);
        ((r * 255.0) as i32, (g * 255.0) as i32, (b * 255.0) as i32)
    };
    let with = ctx(&w, sat(), Some(&idx)).with_lakes(&lakes);
    let without = ctx(&w, sat(), None).with_lakes(&lakes);
    let (a, b) = (px(&with, 24, 32), px(&without, 24, 32));
    assert!(a.1 > a.0 + 40 && a != b, "the active crater lake is not acid: {a:?} (lake palette {b:?})");
    assert_eq!(px(&with, 72, 32), px(&without, 72, 32), "the extinct crater lake changed");
    for a in [TerrainAppearance::default().with_look(render::LOOK_VIBRANT), TerrainAppearance::js_reference()] {
        let (x, y) = (ctx(&w, a.clone(), Some(&idx)).with_lakes(&lakes), ctx(&w, a, None).with_lakes(&lakes));
        assert_eq!(px(&x, 24, 32), px(&y, 24, 32), "another look tinted the crater lake");
    }
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

/// Protects VA-6's two kinds in the look: a dormant lava dome is pale dacite
/// (mean luma well above the dormant stratocone's andesite core beside it,
/// literal margin), has no crater staining or sulphur, and gains obsidian
/// banding as the view zooms in (more distinct rock colours over the dome at
/// 4 m than at 400 m a pixel); an active dome glows under the daytime cap and
/// an active maar never does; a maar's ring reads the tephra slot; a lake
/// inside a maar keeps the lake palette even when the maar is active (its
/// water is groundwater, not an acid vent lake).
#[test]
fn a_dome_is_pale_dacite_and_a_maar_holds_a_plain_lake() {
    let a = sat();
    let row = |kind: EdificeKind, years: f64, crater: Option<(f64, f64)>, r: f64| PlacedEdifice {
        setting: VolcanicSetting::Arc,
        cx: 30.0,
        cy: 30.0,
        r,
        h: 0.03,
        kind: Some(kind),
        age: 0.1,
        years: Some(years),
        crater,
        rift_bearing: None,
    };
    let strato = PlacedEdifice { r: 20.0, h: 0.3, crater: Some((0.02, 0.05)), ..row(EdificeKind::Strato, 5_000.0, None, 20.0) };
    let dome = PlacedEdifice { cx: 40.0, ..row(EdificeKind::Dome, 5_000.0, None, 2.0) };
    let idx = VolcanoIndex::new(&[strato, dome], 96, 64);
    let luma = |c: (f64, f64, f64)| 0.2126 * c.0 + 0.7152 * c.1 + 0.0722 * c.2;
    let mean_rock = |x0: f64, px: f64| {
        let (mut n, mut l, mut cols) = (0.0, 0.0, std::collections::HashSet::new());
        for k in 0..400 {
            let (x, y) = (x0 + ((k % 20) as f64 / 20.0 - 0.5) * 1.6, 30.0 + ((k / 20) as f64 / 20.0 - 0.5) * 1.6);
            let v = volcano_look::sample(&idx, x, y, 0.4, px).expect("on an edifice");
            let g = volcano_look::ground(&a, &v, 0.2, 0.5);
            n += 1.0;
            l += luma(g.rock);
            cols.insert((g.rock.0 as i32, g.rock.1 as i32, g.rock.2 as i32));
        }
        (l / n, cols.len())
    };
    let (dome_l, dome_cols_far) = mean_rock(40.0, 0.4);
    let (_, dome_cols_near) = mean_rock(40.0, 0.004);
    let (core_l, _) = mean_rock(26.0, 0.4);
    assert!(dome_l > core_l + 30.0, "the dome is pale dacite: luma {dome_l:.1} against the cone's core {core_l:.1}");
    assert!(dome_cols_near > dome_cols_far, "obsidian bands and blocks resolve with zoom: {dome_cols_far} -> {dome_cols_near} colours");
    let at = |x: f64, idx: &VolcanoIndex| volcano_look::sample(idx, x, 30.0, 0.4, 0.01).expect("on an edifice");
    assert_eq!(volcano_look::ground(&a, &at(40.0, &idx), 0.2, 0.5).lith, LithoSample::One(2), "a dome reads the andesite slot (dacite)");
    // Active: a dome glows (capped), a maar does not.
    let hot_dome = PlacedEdifice { cx: 40.0, ..row(EdificeKind::Dome, 20.0, None, 2.0) };
    let hi = VolcanoIndex::new(&[hot_dome], 96, 64);
    // A cooling dome glows in its cracks, which need not cross the centre:
    // its top (a third of the radius) is searched.
    let glows: Vec<(f64, f64, f64)> = (0..81)
        .filter_map(|k| {
            let (x, y) = (40.0 + ((k % 9) as f64 / 8.0 - 0.5) * 1.3, 30.0 + ((k / 9) as f64 / 8.0 - 0.5) * 1.3);
            volcano_look::sample(&hi, x, y, 0.4, 0.002).and_then(|v| volcano_look::ground(&a, &v, 0.2, 0.5).glow)
        })
        .collect();
    assert!(!glows.is_empty(), "an active dome glows somewhere on its top");
    assert!(glows.iter().all(|&g| luma(g) <= 110.0 + 1e-9), "under the daytime cap (110, literal)");
    let cold = PlacedEdifice { cx: 40.0, ..row(EdificeKind::Dome, 5_000.0, None, 2.0) };
    let ci0 = VolcanoIndex::new(&[cold], 96, 64);
    assert!((0..81).all(|k| {
        let (x, y) = (40.0 + ((k % 9) as f64 / 8.0 - 0.5) * 1.3, 30.0 + ((k / 9) as f64 / 8.0 - 0.5) * 1.3);
        volcano_look::sample(&ci0, x, y, 0.4, 0.002).is_none_or(|v| volcano_look::ground(&a, &v, 0.2, 0.5).glow.is_none())
    }), "control: a dormant dome does not glow");
    let maar = row(EdificeKind::Maar, 20.0, Some((0.1875, 0.625)), 4.0);
    let mi = VolcanoIndex::new(&[maar], 96, 64);
    // Over the whole crater and ring, centre included: no glow.
    assert!((0..81).all(|k| {
        let (x, y) = (30.0 + ((k % 9) as f64 / 8.0 - 0.5) * 6.0, 30.0 + ((k / 9) as f64 / 8.0 - 0.5) * 6.0);
        volcano_look::sample(&mi, x, y, 0.4, 0.002).is_none_or(|v| volcano_look::ground(&a, &v, 0.2, 0.5).glow.is_none())
    }), "a maar has no vent to glow");
    let gm = volcano_look::ground(&a, &at(30.0 + 3.0, &mi), 0.2, 0.5);
    assert_eq!(gm.lith, LithoSample::One(volcano_look::LITH_TEPHRA), "a maar's ring is tuff");
    let base = (40.0, 70.0, 110.0);
    assert_eq!(volcano_look::crater_lake(&mi, 30.3, 30.2, 0.4, 0.01, base, 1.0), base, "a maar lake keeps the lake palette, even when the maar is active");
    // The control: an active cinder cone of the same size tints its lake.
    let cone = row(EdificeKind::Cinder, 20.0, None, 4.0);
    let ci = VolcanoIndex::new(&[cone], 96, 64);
    assert_ne!(volcano_look::crater_lake(&ci, 30.3, 30.2, 0.4, 0.01, base, 1.0), base, "control: an active cone's crater lake is acid");
}

/// Protects owner Ruling DJ's two looks. A **supervolcano** reads the tephra
/// slot (ignimbrite) and never glows or tints its caldera lake acid -- even
/// a row whose years say active (the kind itself is quiet, "not an active
/// vent") -- while an active stratocone at the same place does (control). A
/// **flood basalt** reads the basalt slot, and its trap riser
/// (`volcano_look::trap_riser`) is, against literals: 0 on flat ground (0.2
/// degrees), the
/// full 0.55 inside a step's top quarter (55 m of a 60 m step) on a 20-degree
/// slope at 1 m a pixel, 0 on the tread (30 m), the mean 0.25 x 0.55 at
/// both heights once the band is under the zoom law's Nyquist width (1 km a
/// pixel), and 0 for an unknown height.
#[test]
fn a_supervolcano_is_quiet_ignimbrite_and_a_flood_basalt_has_trap_steps() {
    let a = sat();
    let row = |kind: EdificeKind, years: f64, crater: Option<(f64, f64)>| PlacedEdifice {
        setting: VolcanicSetting::Hotspot,
        cx: 40.0,
        cy: 30.0,
        r: 22.0,
        h: 0.03,
        kind: Some(kind),
        age: 0.1,
        years: Some(years),
        crater,
        rift_bearing: None,
    };
    let grid = |idx: &VolcanoIndex, rr: f64| -> Vec<volcano_look::VolcanoGround> {
        (0..121)
            .filter_map(|k| {
                let (x, y) = (40.0 + ((k % 11) as f64 / 10.0 - 0.5) * rr, 30.0 + ((k / 11) as f64 / 10.0 - 0.5) * rr);
                volcano_look::sample(idx, x, y, 1.0, 0.002).map(|v| volcano_look::ground(&a, &v, 0.2, 0.5))
            })
            .collect()
    };
    let caldera = Some((0.85 / 2.2, 1.0 / 2.2));
    let sv = VolcanoIndex::new(&[row(EdificeKind::Supervolcano, 20.0, caldera)], 96, 64);
    let g = grid(&sv, 20.0);
    assert!(g.len() > 100 && g.iter().all(|g| g.glow.is_none()), "a supervolcano never glows, whatever its years");
    assert!(g.iter().all(|g| g.lith == LithoSample::One(volcano_look::LITH_TEPHRA)), "its caldera and apron are ignimbrite");
    let base = (40.0, 70.0, 110.0);
    let quiet = VolcanoIndex::new(&[row(EdificeKind::Supervolcano, 631_000.0, caldera)], 96, 64);
    assert_eq!(volcano_look::crater_lake(&quiet, 40.5, 30.2, 1.0, 0.01, base, 1.0), base, "its caldera lake keeps the lake palette");
    let hot = VolcanoIndex::new(&[PlacedEdifice { setting: VolcanicSetting::Arc, ..row(EdificeKind::Strato, 20.0, Some((0.05, 0.12))) }], 96, 64);
    assert!(grid(&hot, 4.0).iter().any(|g| g.glow.is_some()), "control: an active stratocone there glows");
    let fb = VolcanoIndex::new(&[PlacedEdifice { h: 0.0, ..row(EdificeKind::FloodBasalt, 3.2e6, None) }], 96, 64);
    assert!(grid(&fb, 20.0).iter().all(|g| g.lith == LithoSample::One(1) && g.glow.is_none()), "a flood basalt is basalt and quiet");
    let r = volcano_look::trap_riser;
    assert_eq!(r(55.0, 0.2, 0.001), 0.0, "flat ground (0.2 degrees) shows no riser");
    assert!((r(55.0, 20.0, 0.001) - 0.55).abs() < 1e-9, "inside a step's top quarter: {}", r(55.0, 20.0, 0.001));
    assert_eq!(r(30.0, 20.0, 0.001), 0.0, "on the tread");
    assert!((r(55.0, 20.0, 1.0) - 0.25 * 0.55).abs() < 1e-9 && (r(30.0, 20.0, 1.0) - 0.25 * 0.55).abs() < 1e-9, "under Nyquist the band's mean");
    assert_eq!(r(f64::NAN, 20.0, 0.001), 0.0);
}

/// Protects owner Ruling DJ item 2's maar exemption: on the census grid
/// (2048 x 1311, where Ruling CC's floor is 4 cells) two identical 3-cell
/// pits both fall under the floor, but with the volcano table one inside a
/// maar's crater stands (`water_bodies_floored_keep`), the other does not;
/// without a table, or with a table holding no maar, the result is the
/// floor's exactly; `keep_maar_lakes` restores 3 cells, none outside.
#[test]
fn a_maar_lake_stands_under_the_lake_area_floor() {
    let (gw, gh) = (2048usize, 1311usize);
    assert_eq!(cartalith_civ::lake_area_floor_cells(gw, gh), 4, "the census grid's floor (literal)");
    // A gentle tilt down to the west edge, so the plain drains and each pit
    // is its own depression (on a dead-flat plain the classifier's fill
    // joins both pits into one depression of 6 lake cells, above the floor).
    let mut field: Vec<f32> = (0..gw * gh).map(|i| 0.45 + 0.00002 * (i % gw) as f32).collect();
    let rain = vec![0.4f32; gw * gh];
    let pit = |f: &mut [f32], x: usize, y: usize| {
        for dx in 0..3 {
            f[y * gw + x + dx] -= 0.01;
        }
    };
    pit(&mut field, 400, 400);
    pit(&mut field, 1200, 800);
    let maar = PlacedEdifice { setting: VolcanicSetting::Rift, cx: 401.0, cy: 400.0, r: 4.8, h: 0.002, kind: Some(EdificeKind::Maar), age: 0.1, years: Some(3_000.0), crater: Some((0.3 / 1.6, 1.0 / 1.6)), rift_bearing: None };
    let lake = |c: &[u8], x: usize, y: usize| c[y * gw + x] == 2;
    let raw = cartalith_civ::build_water_bodies(&field, gw, gh, 0.42, false, Some(&rain)).classification;
    assert!(lake(&raw, 401, 400) && lake(&raw, 1201, 800), "both pits pool before the floor");
    let floored = render::water_bodies_floored(&field, &rain, gw, gh, 0.42, false, None).classification;
    assert!(!lake(&floored, 401, 400) && !lake(&floored, 1201, 800), "the floor removes both");
    let kept = render::water_bodies_floored_keep(&field, &rain, gw, gh, 0.42, false, None, Some(&[maar])).classification;
    assert!((400..403).all(|x| lake(&kept, x, 400)), "the maar's lake stands");
    assert!(!lake(&kept, 1201, 800), "the other small lake is still floored");
    let cone = PlacedEdifice { kind: Some(EdificeKind::Cinder), crater: None, ..maar };
    assert_eq!(render::water_bodies_floored_keep(&field, &rain, gw, gh, 0.42, false, None, Some(&[cone])).classification, floored, "no maar, no exemption");
    let mut c = floored.clone();
    assert_eq!(render::keep_maar_lakes(&mut c, &raw, &[maar], gw, gh), 3);
    assert_eq!(c, kept);
}

/// Protects §6.3's cover reaching the pixel through `ground`: a
/// 1 000-year-old cinder cone's flank (one age: monogenetic) is covered in
/// 1 800 mm of rain and about half bare in 900 mm (the curve's literal
/// points: half cover at about 1 230 years at 900 mm).
#[test]
fn the_cover_follows_rainfall_on_the_flank() {
    let a = sat();
    // log10(1000) = 3 = 1 + 5.5 * age  ->  age = 2 / 5.5.
    let idx = VolcanoIndex::new(&[PlacedEdifice { rift_bearing: None, setting: VolcanicSetting::Arc, cx: 20.0, cy: 20.0, r: 16.0, h: 0.25, kind: Some(EdificeKind::Cinder), age: 2.0 / 5.5, years: None, crater: None }], 64, 48);
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
    let mk = |age: f64| VolcanoIndex::new(&[PlacedEdifice { rift_bearing: None, setting: VolcanicSetting::Hotspot, cx: 20.0, cy: 20.0, r: 16.0, h: 0.25, kind: Some(EdificeKind::Shield), age, years: None, crater: None }], 64, 48);
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

/// Owner Ruling CX item 6 on a real app world: every ACTIVE edifice's summit
/// zone holds no lake cell in the classification the map draws
/// (`render::water_bodies_floored`, the one every map path and the civ
/// layer read), and its vent carries the daytime glow in Satellite relief;
/// a dormant or extinct summit zone may hold water. Seed 24601's
/// archipelago at 2048 x 1311 and 800 km, the census world whose active
/// craters held 14-20 lake cells before the step (`volcano_cx_bars.rs`).
#[test]
fn an_active_crater_holds_no_lake_and_glows_on_an_app_world() {
    let w = &WORLDS[1];
    assert_eq!(w.seed, 24601);
    let p = world_params(w, 2048, 1311);
    let ws = cartalith_engine::generate_terrain(&p);
    let (gw, gh) = (p.gw, p.gh);
    let vt = ws.volcanoes.as_deref().expect("the app records a volcano table");
    let class = render::water_bodies_floored(&ws.field, &ws.rainfall, gw, gh, ws.sea_level, p.world, None).classification;
    let idx = VolcanoIndex::new(vt, gw, gh);
    let mut wet = [0usize; 3];
    for i in 0..gw * gh {
        if class[i] != 2 {
            continue;
        }
        if let Some(h) = idx.winner_at((i % gw) as f64, (i / gw) as f64)
            && volcano_attr::summit_zone_t(h.edifice).is_some_and(|z| h.t < z)
        {
            wet[volcano_attr::activity(h.edifice) as usize] += 1;
        }
    }
    let active: Vec<&PlacedEdifice> = vt.iter().filter(|e| e.kind.is_some() && volcano_attr::activity(e) == Activity::Active).collect();
    assert!(!active.is_empty(), "premise: the world has an active edifice");
    assert_eq!(wet[0], 0, "lake cells inside active summit zones (active/dormant/extinct: {wet:?})");
    assert!(wet[1] + wet[2] > 0, "premise: dormant or extinct summit zones still hold water ({wet:?})");
    // The positive control: the same world with the step off pools water in
    // an active summit zone (measured 7 cells before the step existed).
    let mut q = p.clone();
    q.volc.dry_active_craters = false;
    let wsq = cartalith_engine::generate_terrain(&q);
    let cq = render::water_bodies_floored(&wsq.field, &wsq.rainfall, gw, gh, wsq.sea_level, q.world, None).classification;
    let wet_off = (0..gw * gh)
        .filter(|&i| cq[i] == 2)
        .filter(|&i| idx.winner_at((i % gw) as f64, (i / gw) as f64).is_some_and(|h| volcano_attr::summit_zone_t(h.edifice).is_some_and(|z| h.t < z) && volcano_attr::activity(h.edifice) == Activity::Active))
        .count();
    assert!(wet_off > 0, "premise: without the step an active crater on this world pools");
    // The glow, at each active vent, under the app's look.
    let mut a = TerrainAppearance::for_tier(QualityTier::Quality).with_look(LOOK_SATELLITE);
    a.npr.peak_m = p.peak_m;
    let kpc = p.map_width_km / gw as f64;
    for e in &active {
        // VA-6: a maar has no vent, so it never glows; a cooling dome glows
        // in its cracks (research §6.2's heat 0.35), which need not cross its
        // centre, so its top is searched (a 9 x 9 lattice over a third of its
        // radius) for a glowing pixel.
        match e.kind {
            Some(EdificeKind::Maar) => continue,
            Some(EdificeKind::Dome) => {
                let lit = (0..81).any(|k| {
                    let (x, y) = (e.cx + ((k % 9) as f64 / 8.0 - 0.5) * 0.66 * e.r, e.cy + ((k / 9) as f64 / 8.0 - 0.5) * 0.66 * e.r);
                    volcano_look::sample(&idx, x, y, kpc, 0.005).is_some_and(|v| volcano_look::ground(&a, &v, 0.6, 0.5).glow.is_some())
                });
                assert!(lit, "an active dome at ({}, {}) carries no glow on its top", e.cx, e.cy);
            }
            _ => {
                let v = volcano_look::sample(&idx, e.cx, e.cy, kpc, 0.005).expect("a sample at the vent");
                assert!(volcano_look::ground(&a, &v, 0.6, 0.5).glow.is_some(), "an active vent at ({}, {}) carries no glow", e.cx, e.cy);
            }
        }
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
const WORLDS: [WorldSpec; 6] = [
    WorldSpec { tag: "s71077345_classic", seed: 71077345, archetype: None },
    WorldSpec { tag: "s24601_archipelago", seed: 24601, archetype: Some(ARCHIPELAGO) },
    WorldSpec { tag: "s314159_classic", seed: 314159, archetype: None },
    WorldSpec { tag: "s483920_classic", seed: 483920, archetype: None },
    WorldSpec { tag: "s246371_classic", seed: 246371, archetype: None },
    // VA-6's brief added the Hawaii-like seed 8 (`lava_flows.rs`' world).
    WorldSpec { tag: "s8_hawaii", seed: 8, archetype: Some([0.05, 0.85, 0.25, 0.85, 1.0]) },
];

fn world_params(w: &WorldSpec, gw: usize, gh: usize) -> cartalith_engine::WorldParams {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = w.seed;
    p.map_width_km = 800.0;
    p.sea_level = 0.42;
    // `VA_MORPH=0`: VA-5's morphometry switch off, the app before Ruling CX
    // item 1 turned it on (`params::defaults()`); unset, the app's default.
    if std::env::var("VA_MORPH").is_ok_and(|v| v == "0") {
        p.volc.morphometry = false;
    }
    // `VA_CINDER=0`: Ruling CX item 4's grid-aware cinder law off (the app
    // before it); unset, the app's default.
    if std::env::var("VA_CINDER").is_ok_and(|v| v == "0") {
        p.volc.cinder_grid_law = false;
    }
    if let Some(k) = w.archetype {
        p.world_structure = cartalith_engine::WorldStructureParams { enabled: true, continentality: k[0], fragmentation: k[1], tectonic_energy: k[2], ocean_depth: k[3], hotspot_density: k[4] };
    }
    p
}

/// `VA_BEFORE`'s arms: the app with one Ruling CX switch turned back. Panics
/// on a name it does not know, so a typo cannot compare the app with itself.
fn before_arm(p: &mut cartalith_engine::WorldParams, name: &str) {
    match name {
        // The app before Ruling CX item 1.
        "no_morph" => p.volc.morphometry = false,
        // Ruling CX item 4's trial: the world without the cinder law.
        "no_cinder_law" => p.volc.cinder_grid_law = false,
        // Ruling CX item 6: the world without the dry active craters.
        "wet_active" => p.volc.dry_active_craters = false,
        // Ruling CX item 2's before is not a generation switch: the same
        // world, its table read under Ruling CW's age map (`years` dropped),
        // handled in `volcano_census` itself.
        "legacy_ages" => {}
        // VA-6: the world without one of its kinds, or without all three.
        "no_caldera" => p.volc.kind_caldera = false,
        "no_dome" => p.volc.kind_dome = false,
        "no_maar" => p.volc.kind_maar = false,
        "no_kinds" => {
            p.volc.kind_caldera = false;
            p.volc.kind_dome = false;
            p.volc.kind_maar = false;
        }
        // Owner Ruling DJ: the app before it (its four kinds off), or without
        // one of the two rare kinds.
        "pre_dj" => {
            p.volc.kind_caldera = false;
            p.volc.kind_maar = false;
            p.volc.kind_supervolcano = false;
            p.volc.kind_flood_basalt = false;
        }
        "no_super" => p.volc.kind_supervolcano = false,
        "no_flood" => p.volc.kind_flood_basalt = false,
        other => panic!("unknown VA_BEFORE arm {other:?}"),
    }
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
            let a = volcano_attr::activity(e);
            act[a as usize] += 1;
            let stamp_h_m = e.h * m_per_unit;
            match morphometry(&ws.field, gw, gh, e, m_per_unit, kpc) {
                Some((h_m, w_km, mean_deg, max_deg, cb)) => {
                    println!(
                        "   {k:>3} {:<12} {kind:<7} {:.3}  {:>10.0}  {:<8}  {:>5.1}  {:>7.0}  {:>7.0}  {:.3}  {:>6.1}  {:>6.1}   {:.3}",
                        volcano_attr::setting_key(e.setting),
                        e.age,
                        volcano_attr::years(e),
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
        let before = std::env::var("VA_BEFORE").ok();
        let mut a_off = a_on.clone();
        // The "before" world: the same world with the colours off, or (with
        // `VA_BEFORE`) the world generated with one switch turned back.
        let ws_b_owned = match before.as_deref() {
            Some("legacy_ages") => {
                println!("   before: legacy_ages (the same world, its table without years)");
                None
            }
            Some(arm) => {
                let mut pb = p.clone();
                before_arm(&mut pb, arm);
                println!("   before: {arm}");
                Some(cartalith_engine::generate_terrain(&pb))
            }
            None => {
                a_off.volcano_look = 0.0;
                None
            }
        };
        let ws_b = ws_b_owned.as_ref().unwrap_or(&ws);
        let lith = cartalith_civ::build_lithology(&ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, ws.sea_level);
        let lith_b = cartalith_civ::build_lithology(&ws_b.field, &ws_b.age_field, &ws_b.volcanic_field, &ws_b.crust_field, &ws_b.resistance_field, &ws_b.rainfall, ws_b.sea_level);
        let idx = VolcanoIndex::new(vt, gw, gh);
        let mut vt_b: Vec<PlacedEdifice> = ws_b.volcanoes.as_deref().expect("the before world records a table").to_vec();
        if before.as_deref() == Some("legacy_ages") {
            for e in vt_b.iter_mut() {
                e.years = None;
            }
        }
        let idx_b = VolcanoIndex::new(&vt_b, gw, gh);
        let lakes = render::water_bodies_floored(&ws.field, &ws.rainfall, gw, gh, ws.sea_level, p.world, None).classification;
        let lakes_b = render::water_bodies_floored(&ws_b.field, &ws_b.rainfall, gw, gh, ws_b.sea_level, p.world, None).classification;
        let pre_on = GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, &a_on, Some(p.map_width_km));
        let pre_off = GridPrecompute::build(&ws_b.field, &ws_b.temperature, &ws_b.rainfall, Some(&ws_b.flow_discharge), gw, gh, ws_b.sea_level, p.world, &a_off, Some(p.map_width_km));
        let c_on = RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, 55.0, 5.0, a_on.clone(), &pre_on)
            .expect("context")
            .with_lithology(&lith)
            .with_lakes(&lakes)
            .with_volcanoes(&idx);
        let c_off = RenderCtx::from_precomputed(&ws_b.field, &ws_b.temperature, &ws_b.rainfall, Some(&ws_b.flow_discharge), gw, gh, ws_b.sea_level, p.world, 55.0, 5.0, a_off.clone(), &pre_off)
            .expect("context")
            .with_lithology(&lith_b)
            .with_lakes(&lakes_b)
            .with_volcanoes(&idx_b);
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
        // Ruling CX item 4: the tallest scoria cone on land that owns its
        // centre, whatever its radius (the picks above need 4 km, which no
        // scoria cone reaches once it is sized to the cited H/W).
        if let Some(&k) = (0..vt.len())
            .filter(|&k| vt[k].kind == Some(EdificeKind::Cinder) && land(&vt[k]) && owns_centre(k))
            .collect::<Vec<_>>()
            .iter()
            .max_by(|&&a, &&b| vt[a].h.total_cmp(&vt[b].h))
        {
            picks.push(("cinder", k));
        }
        // VA-6: the tallest dome and the widest maar on land that own their
        // centre, and the widest arc caldera (an arc stratocone without a
        // recorded crater).
        for (what, kind) in [("dome", EdificeKind::Dome), ("maar", EdificeKind::Maar)] {
            if let Some(&k) = (0..vt.len())
                .filter(|&k| vt[k].kind == Some(kind) && land(&vt[k]) && owns_centre(k))
                .collect::<Vec<_>>()
                .iter()
                .max_by(|&&a, &&b| (vt[a].h * vt[a].r).total_cmp(&(vt[b].h * vt[b].r)))
            {
                picks.push((what, k));
            }
        }
        if let Some(&k) = (0..vt.len())
            .filter(|&k| vt[k].kind == Some(EdificeKind::Strato) && vt[k].crater.is_none() && vt[k].setting == cartalith_terrain::VolcanicSetting::Arc && land(&vt[k]))
            .collect::<Vec<_>>()
            .iter()
            .max_by(|&&a, &&b| vt[a].r.total_cmp(&vt[b].r))
        {
            picks.push(("caldera", k));
        }
        // Ruling DJ: the supervolcano and the flood basalt, where the world
        // has one (their own rows' centres).
        for (what, kind) in [("supervolcano", EdificeKind::Supervolcano), ("flood_basalt", EdificeKind::FloodBasalt)] {
            if let Some(k) = (0..vt.len()).find(|&k| vt[k].kind == Some(kind)) {
                picks.push((what, k));
            }
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
                volcano_attr::years(e),
                volcano_attr::activity(e).key(),
                e.r * kpc,
                e.cx,
                e.cy
            );
            let m1 = |b: &[u8]| disc_measures(b, &idx, k, gw as f64 / 2.0, gh as f64 / 2.0, fit);
            let (mo, mn) = (m1(&b1_off), m1(&b1_on));
            println!("     x1  before: n {} green {:.2} dark {:.2} L {:.1} colours {} grad {:.2} | after: n {} green {:.2} dark {:.2} L {:.1} colours {} grad {:.2}", mo.0, mo.1, mo.2, mo.3, mo.4, mo.5, mn.0, mn.1, mn.2, mn.3, mn.4, mn.5);
            // Ruling DJ item 4: a flood basalt's trap steps show on moderate
            // slopes and bare rock, so its zoom frames centre on the dry land
            // cell in the inner half of the province whose grid slope is
            // nearest 2 degrees (a coarse 4-cell scan), not on its flat centre.
            let (fx, fy) = if e.kind == Some(EdificeKind::FloodBasalt) {
                let k_deg = m_per_unit / (kpc * 1000.0);
                let mut best = (e.cx, e.cy, f64::MAX);
                for y in (2..gh - 2).step_by(4) {
                    for x in (2..gw - 2).step_by(4) {
                        let i = y * gw + x;
                        // Inside the inner half of the province, where its
                        // lobate margin has not faded it.
                        if (ws.field[i] as f64) < ws.sea_level || idx.winner_at(x as f64, y as f64).is_none_or(|h| h.index != k || h.t > 0.5) {
                            continue;
                        }
                        let sl = 0.5 * ((ws.field[i + 1] - ws.field[i - 1]) as f64).hypot((ws.field[i + gw] - ws.field[i - gw]) as f64);
                        // Dry ground first (rain under 0.3 of the 3 000 mm
                        // scale), where the plateau's rock is bare.
                        let off = ((sl * k_deg).atan().to_degrees() - 2.0).abs() + if ws.rainfall[i] < 0.3 { 0.0 } else { 100.0 };
                        if off < best.2 {
                            best = (x as f64, y as f64, off);
                        }
                    }
                }
                println!("     frames centred on the province's owned dry cell nearest a 2-degree slope ({:.0}, {:.0})", best.0, best.1);
                (best.0, best.1)
            } else {
                (e.cx, e.cy)
            };
            for &(rung, ppc) in &RUNGS {
                let f_off = frame(&c_off, &tf_off, fx, fy, ppc, w.seed);
                let f_on = frame(&c_on, &tf_on, fx, fy, ppc, w.seed);
                let mm = |b: &[u8]| disc_measures(b, &idx, k, fx, fy, ppc);
                let (mo, mn) = (mm(&f_off), mm(&f_on));
                println!("     {rung:<3} before: n {} green {:.2} dark {:.2} L {:.1} colours {} grad {:.2} | after: n {} green {:.2} dark {:.2} L {:.1} colours {} grad {:.2}", mo.0, mo.1, mo.2, mo.3, mo.4, mo.5, mn.0, mn.1, mn.2, mn.3, mn.4, mn.5);
                let (sw, sbs) = side_by_side(&f_off, &f_on);
                write_png(&format!("{o}/{}_{what}_{rung}_before_after.png", w.tag), sw, SCREEN_H, sbs);
            }
        }
    }
}
