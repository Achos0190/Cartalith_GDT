//! **Terrain-following lava flows** (owner Ruling DA step 1, 2026-10-25;
//! `cartalith_terrain::lava_flows`, `volcano_look::flow_ground`).
//!
//! Two halves:
//!
//! * **Unit tests** on a synthetic world -- a basalt shield on a plain that
//!   falls east to the sea, with a valley south of the cone -- that protect
//!   what the ruling requires: every look but Satellite relief is
//!   byte-identical with the flows traced (identity by control flow), and so
//!   is Satellite relief with `lava_flows` 0; flows reach ground off the
//!   edifice and only ground downhill of their vents; a young flow is dark,
//!   neutral rock (not water-blue); an old flow in a humid climate fades into
//!   the surrounding ground; an active flow glows under the daytime cap; the
//!   flow's area keeps its size across a 100x zoom while its margin gains
//!   structure (the zoom law); and the screen, the export and a tile agree.
//! * **The census harness** (`#[ignore]`d; run explicitly): real app worlds,
//!   the per-world flow statistics (count, lengths, how many reach the sea,
//!   how many are active) and before/after frames at x1/x4/x16/x64 centred
//!   on the longest flow, a flow that reached the sea and an active flow.
//!
//! ```text
//! LV_OUT=<dir> [LV_ONLY=<tag part>] [LV_FRAMES=0] cargo test --release -p cartalith-godot --test lava_flows -- --ignored --nocapture --test-threads 1
//! ```
//!
//! "Before" is the same world under Satellite relief with `lava_flows` 0
//! (the look before Ruling DA); "after" is the shipped look.

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
use cartalith_terrain::lava_flows::{FlowEnd, TraceInput};
use cartalith_terrain::volcano_attr::{self, VolcanoIndex};
use cartalith_terrain::{EdificeKind, PlacedEdifice, VolcanicSetting};
use render::volcano_look;
use render::{BakeFields, GridPrecompute, QualityTier, RenderCtx, TerrainAppearance, TileBounds, TileFields, LOOK_SATELLITE};

// ===========================================================================
// The synthetic world
// ===========================================================================

const GW: usize = 160;
const GH: usize = 96;
const SEA: f64 = 0.42;
/// Half a km per cell (a judgement for the fixture: an 80 km map, so a
/// shield's 8-50 km flows can run to the sea inside it).
const MAP_KM: f64 = 80.0;
const PEAK_M: f64 = 4000.0;

/// A basalt shield at (75, 40), radius 14 cells (7 km), of `years`: 35 km
/// from the sea, inside a shield flow's 8-50 km band.
fn shield(years: f64) -> PlacedEdifice {
    PlacedEdifice { setting: VolcanicSetting::Hotspot, cx: 75.0, cy: 40.0, r: 14.0, h: 0.12, kind: Some(EdificeKind::Shield), age: 0.0, years: Some(years), crater: None }
}

struct World {
    field: Vec<f32>,
    temp: Vec<f32>,
    rain: Vec<f32>,
}

/// A plain falling east from 0.62 to below the sea (0.40 at x = GW), a
/// valley along y = 64, and the shield's cone on top. Warm (20 C); rain
/// `rain` everywhere.
fn world(rain: f32) -> World {
    let n = GW * GH;
    let mut field = vec![0.0f32; n];
    let e = shield(10.0);
    for y in 0..GH {
        for x in 0..GW {
            let base = 0.62 - 0.22 * x as f64 / GW as f64;
            let valley = (-((y as f64 - 64.0) / 5.0).powi(2)).exp() * 0.03;
            let t = (x as f64 - e.cx).hypot(y as f64 - e.cy) / e.r;
            let cone = if t < 1.0 { 0.12 * (1.0 - t * t) } else { 0.0 };
            field[y * GW + x] = (base - valley + cone) as f32;
        }
    }
    World { field, temp: vec![20.0; n], rain: vec![rain; n] }
}

fn sat() -> TerrainAppearance {
    let mut a = TerrainAppearance::default().with_look(LOOK_SATELLITE);
    a.npr.peak_m = PEAK_M;
    a
}

/// The index as every app consumer builds it (`render::volcano_index_for`).
fn index_for(w: &World, years: f64, a: &TerrainAppearance) -> VolcanoIndex {
    render::volcano_index_for(&[shield(years)], &w.field, GW, GH, SEA, Some(MAP_KM), a)
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

/// Cells off the edifice whose top flow covers at least `min_cov` (by the
/// renderer's own coverage at one pixel per cell) and is younger than
/// `10^max_lg` years, with their coverage. Selected by the INPUT (the
/// flow's coverage and age), never by colour.
fn flow_cells(idx: &VolcanoIndex, min_cov: f64, max_lg: f64) -> Vec<(usize, usize, f64)> {
    let fl = idx.lava_flows().expect("premise: the index carries flows");
    let kpc = MAP_KM / GW as f64;
    let mut out = Vec::new();
    for y in 0..GH {
        for x in 0..GW {
            if idx.winner_at(x as f64, y as f64).is_some() {
                continue;
            }
            if let Some(f) = volcano_look::flow_px(fl, x as f64, y as f64, kpc, kpc).layers[0]
                && f.cov >= min_cov
                && f.log_years < max_lg
            {
                out.push((x, y, f.cov));
            }
        }
    }
    out
}

/// Protects: `lava_flows` is on in Satellite relief only, and every other
/// look -- and Satellite relief with the term at 0 -- builds no flows and
/// draws byte-identically to the look before (identity by control flow).
/// Positive control: Satellite relief with the flows moves bytes.
#[test]
fn lava_flows_are_satellite_relief_only() {
    let w = world(0.6);
    let looks = [
        ("default", TerrainAppearance::default()),
        ("js_reference", TerrainAppearance::js_reference()),
        ("Natural Vibrant", TerrainAppearance::default().with_look(render::LOOK_VIBRANT)),
        ("Antique Parchment", TerrainAppearance::default().with_look(render::LOOK_ANTIQUE)),
        ("Quality tier", TerrainAppearance::default().with_look(render::LOOK_TIER)),
    ];
    for (name, a) in &looks {
        assert_eq!(a.lava_flows, 0.0, "{name}: lava_flows must be 0.0");
        // With the peak altitude known, so only the look's term can be what
        // keeps the flows out.
        let mut a = a.clone();
        a.npr.peak_m = PEAK_M;
        assert!(index_for(&w, 20.0, &a).lava_flows().is_none(), "{name}: traced flows");
    }
    assert_eq!(sat().lava_flows, 1.0);
    let idx_on = index_for(&w, 20.0, &sat());
    assert!(idx_on.lava_flows().is_some_and(|f| !f.is_empty()), "premise: the fixture traces flows");
    let plain = VolcanoIndex::new(&[shield(20.0)], GW, GH);
    let mut off = sat();
    off.lava_flows = 0.0;
    assert!(screen(&ctx(&w, off.clone(), Some(&idx_on))) == screen(&ctx(&w, off, Some(&plain))), "lava_flows 0 with flows traced moved bytes");
    for (name, a) in &looks {
        let mut a = a.clone();
        a.npr.peak_m = PEAK_M;
        let traced = VolcanoIndex::new(&[shield(20.0)], GW, GH).with_lava_flows(TraceInput { field: &w.field, gw: GW, gh: GH, sea_level: SEA, km_per_cell: MAP_KM / GW as f64, m_per_unit: PEAK_M / (1.0 - SEA) });
        assert!(screen(&ctx(&w, a.clone(), Some(&traced))) == screen(&ctx(&w, a, Some(&plain))), "{name}: flows moved bytes");
    }
    assert!(screen(&ctx(&w, sat(), Some(&idx_on))) != screen(&ctx(&w, sat(), Some(&plain))), "positive control: the flows must reach Satellite relief");
}

/// Protects "terrain-following": flows reach well beyond the edifice (at
/// least 40 cells off it), every cell a flow covers off the edifice is lower
/// than the vent it came from, and on this plain some flow reaches the sea.
#[test]
fn flows_leave_the_edifice_and_run_downhill() {
    let w = world(0.6);
    let idx = index_for(&w, 20.0, &sat());
    let cells = flow_cells(&idx, 0.5, 9.0);
    assert!(cells.len() >= 40, "only {} cells off the edifice are lava", cells.len());
    let fl = idx.lava_flows().unwrap();
    let h = |x: f64, y: f64| w.field[(y.round() as usize).min(GH - 1) * GW + (x.round() as usize).min(GW - 1)];
    for l in fl.flows() {
        let v = l.pts[0];
        let hv = h(v.x as f64, v.y as f64);
        for p in &l.pts[l.pts.len() / 4..] {
            assert!(h(p.x as f64, p.y as f64) < hv, "a flow climbed above its vent");
        }
    }
    assert!(fl.flows().iter().any(|l| l.reached_sea), "no flow reached the sea on a plain falling into it");
}

/// Protects "distinct from rivers": a young flow off the edifice is dark
/// (Rec.709 luma under 90) and neutral to warm -- blue never more than 6
/// levels above red -- where the fixture's sea is blue well above red.
#[test]
fn a_young_flow_is_dark_rock_not_water() {
    let w = world(0.6);
    let idx = index_for(&w, 20.0, &sat());
    let s = screen(&ctx(&w, sat(), Some(&idx)));
    let cells = flow_cells(&idx, 0.95, 2.0);
    assert!(cells.len() >= 20, "premise: {} fully covered flow cells", cells.len());
    let (mut bluest, mut lmax) = (i32::MIN, 0.0f64);
    for (x, y, _) in &cells {
        let o = (y * GW + x) * 3;
        let (r, g, b) = (s[o] as i32, s[o + 1] as i32, s[o + 2] as i32);
        bluest = bluest.max(b - r);
        lmax = lmax.max(0.2126 * r as f64 + 0.7152 * g as f64 + 0.0722 * b as f64);
    }
    assert!(bluest <= 6, "a flow cell reads blue: b - r = {bluest}");
    assert!(lmax < 90.0, "a young flow cell is light: luma {lmax:.1}");
    let o = (40 * GW + GW - 2) * 3;
    assert!(s[o + 2] as i32 > s[o] as i32 + 30, "control: the fixture's sea is blue");
}

/// Protects "weathering by eruption age": in 1 800 mm of rain a 400 kyr
/// (dormant, the last age that still traces flows) shield's flows off the
/// edifice lie within 6 RGB levels (Euclidean, on average) of the flow-less
/// render -- faded into the ground -- where the same world's flows at 20
/// years lie more than 40 away.
#[test]
fn old_flows_fade_into_the_surrounding_ground() {
    let w = world(0.6);
    let plain = VolcanoIndex::new(&[shield(400_000.0)], GW, GH);
    let base = screen(&ctx(&w, sat(), Some(&plain)));
    let young = index_for(&w, 20.0, &sat());
    let old = index_for(&w, 400_000.0, &sat());
    // Mean RGB distance to the flow-less render over the cells.
    let diff = |idx: &VolcanoIndex, cells: &[(usize, usize, f64)]| {
        let s = screen(&ctx(&w, sat(), Some(idx)));
        let mut d = 0.0;
        for (x, y, _) in cells {
            let o = (y * GW + x) * 3;
            d += (0..3).map(|c| (s[o + c] as f64 - base[o + c] as f64).powi(2)).sum::<f64>().sqrt();
        }
        d / cells.len().max(1) as f64
    };
    let (cy, co) = (flow_cells(&young, 0.9, 2.0), flow_cells(&old, 0.9, 9.0));
    assert!(cy.len() >= 20 && co.len() >= 20, "premise: {} / {} flow cells", cy.len(), co.len());
    let (dy, dold) = (diff(&young, &cy), diff(&old, &co));
    assert!(dy > 40.0, "a young flow barely shows: {dy:.1}");
    assert!(dold < 6.0, "an old flow in a wet climate still shows: {dold:.1}");
}

/// Protects `PxGround::over_snow`: under snow (the fixture at -10 C), a
/// 20-year flow off the edifice still shows -- more than 40 RGB levels from
/// the flow-less render -- while a 1 000-year flow is snowed over (under 6).
#[test]
fn a_fresh_flow_shows_through_snow() {
    let mut w = world(0.6);
    w.temp = vec![-10.0; GW * GH];
    let plain = VolcanoIndex::new(&[shield(20.0)], GW, GH);
    let base = screen(&ctx(&w, sat(), Some(&plain)));
    let dist = |idx: &VolcanoIndex| {
        let cells = flow_cells(idx, 0.9, 9.0);
        assert!(cells.len() >= 20, "premise: {} flow cells", cells.len());
        let s = screen(&ctx(&w, sat(), Some(idx)));
        cells.iter().map(|(x, y, _)| (0..3).map(|c| (s[(y * GW + x) * 3 + c] as f64 - base[(y * GW + x) * 3 + c] as f64).powi(2)).sum::<f64>().sqrt()).sum::<f64>() / cells.len() as f64
    };
    let (young, old) = (dist(&index_for(&w, 20.0, &sat())), dist(&index_for(&w, 1000.0, &sat())));
    assert!(young > 40.0, "a fresh flow is hidden by snow: {young:.1}");
    assert!(old < 6.0, "a 1 000-year flow shows through snow: {old:.1}");
}

/// Protects the daytime glow on an active flow: some flow cell off the
/// edifice glows (orange: red over blue), and no glow passes the luminance
/// cap; a dormant (500-year) shield's flows do not glow.
#[test]
fn an_active_flow_glows_under_the_cap() {
    let w = world(0.6);
    let a = sat();
    let kpc = MAP_KM / GW as f64;
    for (years, want) in [(20.0, true), (500.0, false)] {
        let idx = index_for(&w, years, &a);
        let mut glowing = 0;
        let mut max_l: f64 = 0.0;
        for y in 0..GH {
            for x in 0..GW {
                if idx.winner_at(x as f64, y as f64).is_some() {
                    continue;
                }
                let Some(v) = volcano_look::sample_px(&idx, x as f64, y as f64, kpc, kpc * 0.1, true) else { continue };
                if let Some(g) = volcano_look::ground_px(&a, &v, 0.6, 0.5, 1.0).glow {
                    glowing += 1;
                    assert!(g.0 > g.2, "the glow is not fire-coloured: {g:?}");
                    max_l = max_l.max(0.2126 * g.0 + 0.7152 * g.1 + 0.0722 * g.2);
                }
            }
        }
        assert_eq!(glowing > 0, want, "{years} yr: {glowing} glowing cells");
        assert!(max_l <= volcano_look::GLOW_LUMA_CAP + 1e-9, "glow {max_l} over the cap");
    }
}

/// Protects the zoom law on the flows: over the same patch of a flow's
/// margin, the covered share stays within 3 points from a 0.5 km pixel to a
/// 5 m pixel (the flow's area does not move with zoom) while its margin
/// grows longer -- more crossings of half coverage between neighbouring
/// samples -- as finer lobes arrive (the margin gains detail), but by less
/// than three times (a margin with a scrambled noise phase grows hairs).
#[test]
fn the_flow_margin_gains_detail_without_moving() {
    let w = world(0.6);
    let idx = index_for(&w, 20.0, &sat());
    let fl = idx.lava_flows().unwrap();
    let kpc = MAP_KM / GW as f64;
    let l = fl.flows().iter().max_by(|a, b| a.len_km.total_cmp(&b.len_km)).unwrap();
    let p = l.pts[l.pts.len() / 2];
    // A 6 x 6 cell patch centred on the flow's middle, sampled 300 x 300.
    let measure = |px_km: f64| {
        let n = 300;
        let cov: Vec<f64> = (0..n * n)
            .map(|k| {
                let (x, y) = (p.x as f64 - 3.0 + 6.0 * (k % n) as f64 / n as f64, p.y as f64 - 3.0 + 6.0 * (k / n) as f64 / n as f64);
                volcano_look::flow_px(fl, x, y, kpc, px_km).layers.iter().flatten().map(|f| f.cov).fold(0.0, f64::max)
            })
            .collect();
        let mut cross = 0;
        for y in 0..n - 1 {
            for x in 0..n - 1 {
                let c = cov[y * n + x] >= 0.5;
                cross += usize::from(c != (cov[y * n + x + 1] >= 0.5)) + usize::from(c != (cov[(y + 1) * n + x] >= 0.5));
            }
        }
        (cov.iter().sum::<f64>() / (n * n) as f64, cross)
    };
    let (c_far, x_far) = measure(0.5);
    let (c_near, x_near) = measure(0.005);
    assert!(c_far > 0.05, "premise: the patch is on the flow ({c_far})");
    assert!((c_far - c_near).abs() < 0.03, "the flow's area moved with zoom: {c_far:.3} -> {c_near:.3}");
    // Measured 324 -> 399 (2026-10-25, the lava material's fractal edge).
    // The upper bound is the hair test: a margin whose noise phase is
    // scrambled from pixel to pixel crosses half coverage at almost every
    // sample at the fine scale.
    assert!(x_near as f64 > 1.1 * x_far as f64, "the margin gained no detail: {x_far} -> {x_near} crossings");
    assert!((x_near as f64) < 3.0 * x_far as f64, "the margin is hairy: {x_far} -> {x_near} crossings");
}

/// Protects "consistent in screen, tiles and export": with the flows on,
/// the export's pixel at integer cells equals the screen within one level,
/// and a one-pixel-per-cell tile equals the finished screen within one
/// level.
#[test]
fn screen_export_and_tile_agree_on_the_flows() {
    let w = world(0.6);
    let a = TerrainAppearance { exag: 0.0, ..sat() };
    let idx = index_for(&w, 20.0, &a);
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

/// Protects the save of appearance documents: `lava_flows` serialises only
/// when set, so every existing look's document (and the tile-cache
/// fingerprint built from it) keeps its bytes, and a Satellite relief
/// document saved before the term (no key) reloads with the flows off --
/// as it was saved.
#[test]
fn lava_flows_serialises_only_when_set() {
    for a in [TerrainAppearance::default(), TerrainAppearance::js_reference(), TerrainAppearance::default().with_look(render::LOOK_VIBRANT)] {
        assert!(!serde_json::to_string(&a).unwrap().contains("lava_flows"));
    }
    let j = serde_json::to_string(&sat()).unwrap();
    assert!(j.contains("\"lava_flows\":1.0"), "{j}");
    let old = j.replace(",\"lava_flows\":1.0", "").replace("\"lava_flows\":1.0,", "");
    assert!(!old.contains("lava_flows"));
    let back: TerrainAppearance = serde_json::from_str(&old).unwrap();
    assert_eq!(back.lava_flows, 0.0, "an old document must reload without flows");
}

/// Protects reproducibility through the app's constructor: the same world
/// gives the same flows, and a table that differs only in one edifice's
/// recorded position gives different ones (the salt reads the position).
#[test]
fn flows_are_reproducible_from_the_table_and_the_field() {
    let w = world(0.6);
    let (a, b) = (index_for(&w, 20.0, &sat()), index_for(&w, 20.0, &sat()));
    assert_eq!(a.lava_flows().unwrap().flows(), b.lava_flows().unwrap().flows());
    let mut e = shield(20.0);
    e.cx += 1.0;
    let moved = render::volcano_index_for(&[e], &w.field, GW, GH, SEA, Some(MAP_KM), &sat());
    assert_ne!(a.lava_flows().unwrap().flows(), moved.lava_flows().unwrap().flows());
    // And no scale, no flows: an unknown peak altitude traces nothing.
    let mut nopeak = sat();
    nopeak.npr.peak_m = 0.0;
    assert!(index_for(&w, 20.0, &nopeak).lava_flows().is_none());
}

// ===========================================================================
// The census harness on real worlds
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
/// A Hawaii-like world: the World Structure model with little continent,
/// much fragmentation, low tectonic energy, a deep ocean and the hotspot
/// density at its top, so the land is mostly hotspot shields in open ocean.
/// A judgement of the five knobs, not an archetype the app lists.
const HAWAII_LIKE: [f64; 5] = [0.05, 0.85, 0.25, 0.85, 1.0];

/// The brief's worlds (71077345, 24601's archipelago, 314159) and the
/// Hawaii-like one.
const WORLDS: [WorldSpec; 4] = [
    WorldSpec { tag: "s71077345_classic", seed: 71077345, archetype: None },
    WorldSpec { tag: "s24601_archipelago", seed: 24601, archetype: Some(ARCHIPELAGO) },
    WorldSpec { tag: "s314159_classic", seed: 314159, archetype: None },
    // Seed 8: the first seed of `find_young_shield_worlds`' 1..30 scan whose
    // youngest land shield is active (24 years, 35 km radius).
    WorldSpec { tag: "s8_hawaii", seed: 8, archetype: Some(HAWAII_LIKE) },
];

fn world_params(w: &WorldSpec, gw: usize, gh: usize) -> cartalith_engine::WorldParams {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = w.seed;
    p.map_width_km = 800.0;
    p.sea_level = 0.42;
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

/// Median of a list (empty: NaN).
fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

/// Per frame, over the screen pixels a flow covers by at least half (the
/// renderer's own coverage at each pixel's grid position -- selected by the
/// INPUT, never by colour): pixel count, mean luma, the largest `b - r`
/// (blueness; water reads well above 20), distinct colours and the mean
/// absolute luma gradient to the right/down neighbour (detail).
fn flow_measures(buf: &[u8], fl: &cartalith_terrain::lava_flows::LavaFlows, kpc: f64, gx: f64, gy: f64, ppc: f64) -> (usize, f64, i32, usize, f64) {
    let luma = |o: usize| 0.2126 * buf[o] as f64 + 0.7152 * buf[o + 1] as f64 + 0.0722 * buf[o + 2] as f64;
    let (mut n, mut lsum, mut blue, mut grad, mut gn) = (0usize, 0.0, i32::MIN, 0.0, 0usize);
    let mut colours = std::collections::HashSet::new();
    let px_km = kpc / ppc;
    for sy in (0..SCREEN_H - 1).step_by(2) {
        let cy = gy + (sy as f64 + 0.5 - SCREEN_H as f64 / 2.0) / ppc;
        for sx in (0..SCREEN_W - 1).step_by(2) {
            let cx = gx + (sx as f64 + 0.5 - SCREEN_W as f64 / 2.0) / ppc;
            let Some(f) = volcano_look::flow_px(fl, cx, cy, kpc, px_km).layers[0] else { continue };
            if f.cov < 0.5 {
                continue;
            }
            let o = (sy * SCREEN_W + sx) * 3;
            let (r, g, b) = (buf[o] as i32, buf[o + 1] as i32, buf[o + 2] as i32);
            n += 1;
            let l = luma(o);
            lsum += l;
            blue = blue.max(b - r);
            colours.insert((r, g, b));
            grad += (luma(o + 3) - l).abs() + (luma(o + SCREEN_W * 3) - l).abs();
            gn += 2;
        }
    }
    (n, lsum / n.max(1) as f64, blue, colours.len(), grad / gn.max(1) as f64)
}

/// Ruling DA step 1's census: per world, the traced flows' statistics and,
/// for the longest flow, a flow that reached the sea and an active flow,
/// before/after frames at x1/x4/x16/x64 with the flow measures. Satellite
/// relief at the Quality tier, as the app draws it.
#[test]
#[ignore = "generates 2048x1311 worlds and renders zoom frames; run explicitly with --release"]
fn lava_census() {
    let out = std::env::var("LV_OUT").ok();
    if let Some(o) = out.as_ref() {
        std::fs::create_dir_all(o).unwrap();
    }
    let only: Option<String> = std::env::var("LV_ONLY").ok();
    let frames = std::env::var("LV_FRAMES").map(|v| v != "0").unwrap_or(true);
    let (gw, gh) = (2048usize, 1311usize);
    for w in &WORLDS {
        if only.as_ref().is_some_and(|o| !w.tag.contains(o.as_str())) {
            continue;
        }
        let p = world_params(w, gw, gh);
        let ws = cartalith_engine::generate_terrain(&p);
        let kpc = p.map_width_km / gw as f64;
        let Some(vt) = ws.volcanoes.as_ref() else {
            println!("== {}: no volcano table", w.tag);
            continue;
        };
        let land = ws.field.iter().filter(|&&h| h as f64 > ws.sea_level).count() as f64 / ws.field.len() as f64;
        let mut a_on = TerrainAppearance::for_tier(QualityTier::Quality).with_look(LOOK_SATELLITE);
        a_on.npr.peak_m = p.peak_m;
        let mut a_off = a_on.clone();
        a_off.lava_flows = 0.0;
        let t0 = std::time::Instant::now();
        let idx = render::volcano_index_for(vt, &ws.field, gw, gh, ws.sea_level, Some(p.map_width_km), &a_on);
        let trace_ms = t0.elapsed().as_secs_f64() * 1e3;
        let fl = idx.lava_flows().expect("the app look traces flows");
        let kinds = |k: EdificeKind| vt.iter().filter(|e| e.kind == Some(k)).count();
        let acts = |a: volcano_attr::Activity| vt.iter().filter(|e| e.kind.is_some() && volcano_attr::activity(e) == a).count();
        println!(
            "== {}: land {:.2}; {} edifices (cinder {}, strato {}, shield {}; active {}, dormant {}, extinct {}); {} flows traced in {:.1} ms",
            w.tag,
            land,
            vt.len(),
            kinds(EdificeKind::Cinder),
            kinds(EdificeKind::Strato),
            kinds(EdificeKind::Shield),
            acts(volcano_attr::Activity::Active),
            acts(volcano_attr::Activity::Dormant),
            acts(volcano_attr::Activity::Extinct),
            fl.flows().len(),
            trace_ms
        );
        let lens: Vec<f64> = fl.flows().iter().map(|l| l.len_km).collect();
        let sea = fl.flows().iter().filter(|l| l.reached_sea).count();
        let active = fl.flows().iter().filter(|l| l.active).count();
        let branches = fl.flows().iter().filter(|l| l.pts[0].s_km > 0.0).count();
        let maxlen = lens.iter().copied().fold(0.0, f64::max);
        println!("   lengths km: median {:.1}, max {:.1}; reached the sea {sea}; active {active}; branches {branches}", median(lens.clone()), maxlen);
        for kind in [EdificeKind::Shield, EdificeKind::Strato, EdificeKind::Cinder] {
            let ends: Vec<String> = [FlowEnd::Budget, FlowEnd::Sea, FlowEnd::Ponded, FlowEnd::Stalled, FlowEnd::Edge]
                .iter()
                .map(|&e| format!("{e:?} {}", fl.flows().iter().filter(|l| l.end == e && vt[l.edifice as usize].kind == Some(kind)).count()))
                .collect();
            println!("   {:<7} ends: {}", kind.key(), ends.join(", "));
        }
        for kind in [EdificeKind::Shield, EdificeKind::Strato, EdificeKind::Cinder] {
            let l: Vec<f64> = fl.flows().iter().filter(|l| vt[l.edifice as usize].kind == Some(kind)).map(|l| l.len_km).collect();
            let w_km: Vec<f64> = fl.flows().iter().filter(|l| vt[l.edifice as usize].kind == Some(kind)).flat_map(|l| l.pts.iter().map(|p| 2.0 * p.hw_km as f64)).collect();
            if !l.is_empty() {
                println!("   {:<7} {} flows, length median {:.1} km (max {:.1}), width median {:.2} km (max {:.2})", kind.key(), l.len(), median(l.clone()), l.iter().copied().fold(0.0, f64::max), median(w_km.clone()), w_km.iter().copied().fold(0.0, f64::max));
            }
        }
        // What each pick's flow is, and what the look decides at its pick
        // point (the flow's own age, width and bare share in this rain).
        let describe = |what: &str, l: &cartalith_terrain::lava_flows::LavaFlow, u: f64| {
            let pt = l.pts[((l.pts.len() - 1) as f64 * u) as usize];
            let (x, y) = (pt.x as f64, pt.y as f64);
            let i = (y as usize).min(gh - 1) * gw + (x as usize).min(gw - 1);
            let v = volcano_look::sample_px(&idx, x, y, kpc, kpc / 7.42, true);
            let g = v.map(|v| volcano_look::ground_px(&a_on, &v, ws.rainfall[i] as f64, 0.5, 1.0));
            println!(
                "   {what}: edifice #{} {} {}, {:.0} yr, {:.1} km, sea {}, hw at pick {:.2} km, rain {:.2}, bare {:.2}, rock {:?}",
                l.edifice,
                vt[l.edifice as usize].kind.map_or("-", |k| k.key()),
                volcano_attr::activity(&vt[l.edifice as usize]).key(),
                l.years,
                l.len_km,
                l.reached_sea,
                l.pts[((l.pts.len() - 1) as f64 * u) as usize].hw_km,
                ws.rainfall[i],
                g.map_or(-1.0, |g| g.bare),
                g.map(|g| (g.rock.0 as i32, g.rock.1 as i32, g.rock.2 as i32))
            );
        };
        for (k, e) in vt.iter().enumerate().filter(|(_, e)| e.kind.is_some() && volcano_attr::years(e) < 1000.0) {
            // Per flow: its length, why it ended, and the straight-line
            // distance from its first point to its last (km) -- a walk that
            // circles shows a distance far under its length.
            let mine: Vec<String> = fl
                .flows()
                .iter()
                .filter(|l| l.edifice as usize == k)
                .map(|l| {
                    let (a, b) = (l.pts[0], *l.pts.last().unwrap());
                    format!("{:.1}km/{:?}/net {:.1}", l.len_km - a.s_km as f64, l.end, (a.x - b.x).hypot(a.y - b.y) as f64 * kpc)
                })
                .collect();
            println!("   young #{k} {} {:.0} yr r {:.1} km at ({:.0},{:.0}): {}", e.kind.unwrap().key(), volcano_attr::years(e), e.r * kpc, e.cx, e.cy, mine.join(" "));
        }
        for l in fl.flows().iter().filter(|l| l.len_km > 8.0) {
            describe("long flow", l, 0.5);
        }
        // A diagnostic plan (`LV_PLAN=1`): around the edifice of the longest
        // flow, the height field in grey (sea blue) at 3 px a cell with every
        // flow's centre line drawn over it (red young to yellow old), so the
        // walk itself can be judged apart from the look.
        if let (Some(o), true) = (out.as_ref(), std::env::var("LV_PLAN").is_ok_and(|v| v == "1"))
            && let Some(lmax) = fl.flows().iter().max_by(|a, b| a.len_km.total_cmp(&b.len_km))
        {
            let ek = lmax.edifice;
            let pts: Vec<_> = fl.flows().iter().filter(|l| l.edifice == ek).flat_map(|l| l.pts.iter()).collect();
            let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
            for q in &pts {
                x0 = x0.min(q.x);
                y0 = y0.min(q.y);
                x1 = x1.max(q.x);
                y1 = y1.max(q.y);
            }
            let (x0, y0) = ((x0 as i64 - 20).max(0) as usize, (y0 as i64 - 20).max(0) as usize);
            let (x1, y1) = (((x1 as usize) + 20).min(gw - 1), ((y1 as usize) + 20).min(gh - 1));
            let sc = 3usize;
            let (pw, ph) = ((x1 - x0 + 1) * sc, (y1 - y0 + 1) * sc);
            let mut img = vec![0u8; pw * ph * 3];
            for py in 0..ph {
                for px in 0..pw {
                    let (gx, gy) = (x0 + px / sc, y0 + py / sc);
                    let h = ws.field[gy * gw + gx] as f64;
                    let o3 = (py * pw + px) * 3;
                    if h < ws.sea_level {
                        img[o3..o3 + 3].copy_from_slice(&[30, 60, 120]);
                    } else {
                        let v = (40.0 + 215.0 * ((h - ws.sea_level) / (1.0 - ws.sea_level)).sqrt()) as u8;
                        img[o3..o3 + 3].copy_from_slice(&[v, v, v]);
                    }
                }
            }
            for l in fl.flows().iter().filter(|l| l.edifice == ek) {
                let t = ((l.years.log10() - 1.0) / 4.5).clamp(0.0, 1.0);
                let col = [255u8, (255.0 * t) as u8, 0];
                for w2 in l.pts.windows(2) {
                    for k in 0..12 {
                        let f = k as f32 / 12.0;
                        let (gx, gy) = (w2[0].x + (w2[1].x - w2[0].x) * f, w2[0].y + (w2[1].y - w2[0].y) * f);
                        let (px, py) = (((gx - x0 as f32) * sc as f32) as usize, ((gy - y0 as f32) * sc as f32) as usize);
                        if px < pw && py < ph {
                            img[(py * pw + px) * 3..(py * pw + px) * 3 + 3].copy_from_slice(&col);
                        }
                    }
                }
            }
            write_png(&format!("{o}/{}_plan.png", w.tag), pw, ph, img);
        }
        if !frames {
            continue;
        }
        let Some(o) = out.as_ref() else { continue };
        let lith = cartalith_civ::build_lithology(&ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, ws.sea_level);
        let lakes = render::water_bodies_floored(&ws.field, &ws.rainfall, gw, gh, ws.sea_level, p.world, None).classification;
        let pre_on = GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, &a_on, Some(p.map_width_km));
        let pre_off = GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, &a_off, Some(p.map_width_km));
        let idx_off = render::volcano_index_for(vt, &ws.field, gw, gh, ws.sea_level, Some(p.map_width_km), &a_off);
        let c_on = RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, 55.0, 5.0, a_on.clone(), &pre_on)
            .expect("context")
            .with_lithology(&lith)
            .with_lakes(&lakes)
            .with_volcanoes(&idx);
        let c_off = RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, 55.0, 5.0, a_off.clone(), &pre_off)
            .expect("context")
            .with_lithology(&lith)
            .with_lakes(&lakes)
            .with_volcanoes(&idx_off);
        let (tf_on, tf_off) = (TileFields::new(&c_on, None), TileFields::new(&c_off, None));
        let fit = SCREEN_W as f64 / gw as f64;
        let (b1_off, b1_on) = (base_frame(&c_off, gw as f64 / 2.0, gh as f64 / 2.0, fit), base_frame(&c_on, gw as f64 / 2.0, gh as f64 / 2.0, fit));
        let (sw, sbs) = side_by_side(&b1_off, &b1_on);
        write_png(&format!("{o}/{}_x1_before_after.png", w.tag), sw, SCREEN_H, sbs);
        // The picks: the longest flow (its middle), a flow that reached the
        // sea (its end), an active flow (a third of the way down).
        let mut picks: Vec<(&str, f64, f64)> = Vec::new();
        let at = |l: &cartalith_terrain::lava_flows::LavaFlow, u: f64| {
            let p = l.pts[((l.pts.len() - 1) as f64 * u) as usize];
            (p.x as f64, p.y as f64)
        };
        if let Some(l) = fl.flows().iter().max_by(|a, b| a.len_km.total_cmp(&b.len_km)) {
            let (x, y) = at(l, 0.5);
            picks.push(("longest", x, y));
        }

        if let Some(l) = fl.flows().iter().filter(|l| l.reached_sea).max_by(|a, b| a.len_km.total_cmp(&b.len_km)) {
            let (x, y) = at(l, 0.95);
            picks.push(("sea", x, y));
        }
        if let Some(l) = fl.flows().iter().filter(|l| l.active).max_by(|a, b| a.len_km.total_cmp(&b.len_km)) {
            let (x, y) = at(l, 0.33);
            picks.push(("active", x, y));
        }
        // Owner check (the brief): 71077345's largest edifice at x4, for
        // the caldera-lake straight edge an earlier agent noticed.
        if w.seed == 71077345
            && let Some(e) = vt.iter().filter(|e| e.kind.is_some()).max_by(|a, b| a.r.total_cmp(&b.r))
        {
            picks.push(("largest_edifice", e.cx, e.cy));
        }
        for (what, gx, gy) in picks {
            println!("   frames: {what} at ({gx:.0}, {gy:.0})");
            for &(rung, ppc) in &RUNGS {
                let f_off = frame(&c_off, &tf_off, gx, gy, ppc, w.seed);
                let f_on = frame(&c_on, &tf_on, gx, gy, ppc, w.seed);
                let (mo, mn) = (flow_measures(&f_off, fl, kpc, gx, gy, ppc), flow_measures(&f_on, fl, kpc, gx, gy, ppc));
                println!(
                    "     {rung:<3} on flows: before n {} L {:.1} b-r max {} colours {} grad {:.2} | after n {} L {:.1} b-r max {} colours {} grad {:.2}",
                    mo.0, mo.1, mo.2, mo.3, mo.4, mn.0, mn.1, mn.2, mn.3, mn.4
                );
                let (sw, sbs) = side_by_side(&f_off, &f_on);
                write_png(&format!("{o}/{}_{what}_{rung}_before_after.png", w.tag), sw, SCREEN_H, sbs);
                // `LV_PLAN=1`: the same "after" frame with every flow's
                // centre line drawn over it in red (young) to yellow (old),
                // so the walk and the look can be read against each other.
                if std::env::var("LV_PLAN").is_ok_and(|v| v == "1") {
                    let mut ov = f_on.clone();
                    for l in fl.flows() {
                        let t = ((l.years.log10() - 1.0) / 4.5).clamp(0.0, 1.0);
                        for w2 in l.pts.windows(2) {
                            for k in 0..8 {
                                let f = k as f64 / 8.0;
                                let (cx, cy) = (w2[0].x as f64 + (w2[1].x - w2[0].x) as f64 * f, w2[0].y as f64 + (w2[1].y - w2[0].y) as f64 * f);
                                let sx = ((cx - gx) * ppc + SCREEN_W as f64 / 2.0) as i64;
                                let sy = ((cy - gy) * ppc + SCREEN_H as f64 / 2.0) as i64;
                                if sx >= 0 && sy >= 0 && (sx as usize) < SCREEN_W && (sy as usize) < SCREEN_H {
                                    let o3 = (sy as usize * SCREEN_W + sx as usize) * 3;
                                    ov[o3..o3 + 3].copy_from_slice(&[255, (255.0 * t) as u8, 0]);
                                }
                            }
                        }
                    }
                    write_png(&format!("{o}/{}_{what}_{rung}_plan.png", w.tag), SCREEN_W, SCREEN_H, ov);
                }
            }
        }
    }
}


/// Finds Hawaii-like worlds (`HAWAII_LIKE`) whose youngest shield is young:
/// a seed search for the census, printing each seed's youngest shield's age.
/// `LV_SEEDS=<from>..<to>`.
#[test]
#[ignore = "generates many worlds; run explicitly with --release"]
fn find_young_shield_worlds() {
    let r = std::env::var("LV_SEEDS").unwrap_or_else(|_| "1..20".into());
    let (a, b) = r.split_once("..").map(|(a, b)| (a.parse::<i32>().unwrap(), b.parse::<i32>().unwrap())).unwrap();
    for seed in a..b {
        let w = WorldSpec { tag: "scan", seed, archetype: Some(HAWAII_LIKE) };
        let p = world_params(&w, 2048, 1311);
        let ws = cartalith_engine::generate_terrain(&p);
        let Some(vt) = ws.volcanoes.as_ref() else { continue };
        let young = vt.iter().filter(|e| e.kind == Some(EdificeKind::Shield) && ws.field[(e.cy as usize).min(1310) * 2048 + (e.cx as usize).min(2047)] as f64 > ws.sea_level).map(|e| (volcano_attr::years(e), e.r * 800.0 / 2048.0)).min_by(|x, y| x.0.total_cmp(&y.0));
        println!("seed {seed}: youngest shield on land {young:?}");
    }
}

/// The brief's "caldera lake cut by a straight edge" on 71077345, re-opened
/// (2026-10-25): the edge is the contour where H2's soft ceiling
/// (`orogen::soft_ceiling`, knee 0.9) saturates. A tall cone stamped on
/// ground already near the top leaves a stamped height far above 1, which
/// the tanh squeezes into the last few thousandths, so the part of the cone
/// on the high side is flat at the ceiling and its edge follows a contour of
/// the ground beneath, not the cone. This census counts, per world, the
/// land cells at the ceiling (final height at least 0.999) and how many of
/// them lie on an edifice's disc, and lists the edifices with any.
///
/// Measured 2026-10-25 (app defaults before Ruling DB; `CEIL_ARM=old` now): 849, 328, 2 114 and 33 625 ceiling
/// cells on 71077345, 24601's archipelago, 314159 and the Hawaii-like seed 8
/// (0.05, 0.08, 0.11 and 25.04 % of the land), every one on an edifice. A
/// trial that scaled each cone to keep its summit under the ceiling cut them
/// to 493 / 4 / 57 / 12 759 but left 71077345's edge visible (its largest
/// stratocone's disc covers a plateau already at the ceiling) and failed
/// the settlement bar on the archipelago (0.265 within 3 cells: World
/// Structure re-anchors the sea level on the height histogram), so it was
/// not kept (`STATUS.md`, Ruling DA).
///
/// Ruling DB item 3 (owner, 2026-10-25) chose the higher world ceiling
/// (`TectonicParams::high_ceiling`, top `orogen::high_ceiling_top`). `CEIL_ARM`
/// picks the arm: `app` (default, the ceiling on) or `old` (off, the top at
/// 1). Each world's line adds the arm's top, the cells within 0.001 of it
/// (a flat cap at the top), the field's maximum in metres, the sea level and
/// an FNV digest of the land mask, so two arms' coasts compare by digest.
/// Measured 2026-10-25 with `app`: maxima 1.2795 / 1.2077 / 1.2444 / 1.1420
/// and 0 / 0 / 0 / 2 592 cells within 0.001 of the top (`STATUS.md`, Ruling DB
/// item 3). The per-edifice lines below keep the 0.999 count of the old arm,
/// which under `app` counts high ground, not a cap.
#[test]
#[ignore = "measurement; run explicitly with --release"]
fn summit_ceiling_census() {
    let arm = std::env::var("CEIL_ARM").unwrap_or_else(|_| "app".into());
    for w in &WORLDS {
        let mut p = world_params(w, 2048, 1311);
        match arm.as_str() {
            "app" => {}
            "old" => p.tect.high_ceiling = false,
            other => panic!("unknown CEIL_ARM {other:?}"),
        }
        let (gw, gh) = (2048usize, 1311usize);
        let ws = cartalith_engine::generate_terrain(&p);
        let top = if p.tect.orogen_relief && p.tect.high_ceiling { cartalith_terrain::orogen::high_ceiling_top(ws.sea_level) } else { 1.0 };
        let (mut at_top, mut hmax, mut digest) = (0usize, 0.0f64, 0xcbf2_9ce4_8422_2325u64);
        for &v in ws.field.iter() {
            let h = v as f64;
            hmax = hmax.max(h);
            if h > ws.sea_level && h >= top - 0.001 {
                at_top += 1;
            }
            digest = (digest ^ u64::from(h > ws.sea_level)).wrapping_mul(0x100_0000_01b3);
        }
        let metres = (hmax - ws.sea_level) / (1.0 - ws.sea_level) * p.peak_m;
        println!("== {} [{arm}, top {top}]: sea {:.6}, max {hmax:.4} ({metres:.0} m), within 0.001 of the top {at_top}, land-mask digest {digest:016x}", w.tag, ws.sea_level);
        let Some(vt) = ws.volcanoes.as_ref() else { continue };
        let idx = VolcanoIndex::new(vt, gw, gh);
        let (mut sat_all, mut sat_volc, mut land) = (0usize, 0usize, 0usize);
        let mut per: std::collections::BTreeMap<usize, (usize, usize)> = Default::default();
        for y in 0..gh {
            for x in 0..gw {
                let h = ws.field[y * gw + x] as f64;
                if h <= ws.sea_level {
                    continue;
                }
                land += 1;
                let owner = idx.winner_at(x as f64, y as f64);
                if let Some(o) = owner {
                    per.entry(o.index).or_default().1 += 1;
                }
                if h >= 0.999 {
                    sat_all += 1;
                    if let Some(o) = owner {
                        sat_volc += 1;
                        per.entry(o.index).or_default().0 += 1;
                    }
                }
            }
        }
        println!("== {}: land {land}, at the ceiling {sat_all} ({:.2} %), of them on an edifice {sat_volc}", w.tag, 100.0 * sat_all as f64 / land.max(1) as f64);
        for (k, (s, n)) in per.iter().filter(|(_, (s, _))| *s > 0) {
            let e = &vt[*k];
            println!("   #{k} {} r {:.1} km h {:.2}: {s} of {n} disc cells at the ceiling ({:.0} %)", e.kind.map_or("-", |k| k.key()), e.r * 800.0 / 2048.0, e.h, 100.0 * *s as f64 / *n as f64);
        }
    }
}

/// Ruling DB item 3's frames: the same world generated with the old top
/// (`tect.high_ceiling` off) and the higher ceiling (the app), rendered in
/// Satellite relief at the Quality tier at x1, x4, x16 and x64, side by side
/// (old left, new right), around each world's largest edifice with ground at
/// the old ceiling. `LV_OUT=<dir>` (required), `LV_ONLY=<tag part>`.
///
/// Protects: nothing by itself (a measurement, `#[ignore]`d); it asserts
/// only that each frame is not empty (`write_png`).
#[test]
#[ignore = "generates 2048x1311 worlds twice and renders zoom frames; run explicitly with --release"]
fn summit_ceiling_frames() {
    let o = std::env::var("LV_OUT").expect("LV_OUT=<dir>");
    std::fs::create_dir_all(&o).unwrap();
    let only: Option<String> = std::env::var("LV_ONLY").ok();
    let (gw, gh) = (2048usize, 1311usize);
    // One arm's frames at (gx, gy), or the pick when `at` is None: the
    // largest edifice with at least one disc cell at or above 0.999 in the
    // old arm (the flat-capped one).
    let render_arm = |w: &WorldSpec, high: bool, at: Option<(f64, f64)>| -> ((f64, f64), Vec<Vec<u8>>) {
        let mut p = world_params(w, gw, gh);
        p.tect.high_ceiling = high;
        let ws = cartalith_engine::generate_terrain(&p);
        let vt = ws.volcanoes.as_ref().expect("a volcano table");
        let mut a = TerrainAppearance::for_tier(QualityTier::Quality).with_look(LOOK_SATELLITE);
        a.npr.peak_m = p.peak_m;
        let idx = render::volcano_index_for(vt, &ws.field, gw, gh, ws.sea_level, Some(p.map_width_km), &a);
        let at = at.unwrap_or_else(|| {
            let capped = |e: &PlacedEdifice| {
                let (cx, cy, r) = (e.cx as i64, e.cy as i64, e.r.max(1.0) as i64);
                (-r..=r).any(|dy| (-r..=r).any(|dx| {
                    let (x, y) = (cx + dx, cy + dy);
                    x >= 0 && y >= 0 && (x as usize) < gw && (y as usize) < gh && ws.field[y as usize * gw + x as usize] >= 0.999
                }))
            };
            let e = vt.iter().filter(|e| e.kind.is_some() && capped(e)).max_by(|a, b| a.r.total_cmp(&b.r)).expect("a capped edifice");
            (e.cx, e.cy)
        });
        let ci = at.1 as usize * gw + at.0 as usize;
        println!("   [{}] at the pick: height {:.4}, temperature {:.2}, rain {:.3}, sea {:.4}", if high { "new" } else { "old" }, ws.field[ci], ws.temperature[ci], ws.rainfall[ci], ws.sea_level);
        let lith = cartalith_civ::build_lithology(&ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, ws.sea_level);
        let lakes = render::water_bodies_floored(&ws.field, &ws.rainfall, gw, gh, ws.sea_level, p.world, None).classification;
        let pre = GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, &a, Some(p.map_width_km));
        let c = RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level, p.world, 55.0, 5.0, a.clone(), &pre)
            .expect("context")
            .with_lithology(&lith)
            .with_lakes(&lakes)
            .with_volcanoes(&idx);
        let tf = TileFields::new(&c, None);
        let mut frames = vec![base_frame(&c, gw as f64 / 2.0, gh as f64 / 2.0, SCREEN_W as f64 / gw as f64)];
        for &(_, ppc) in &RUNGS {
            frames.push(frame(&c, &tf, at.0, at.1, ppc, w.seed));
        }
        (at, frames)
    };
    for w in &WORLDS {
        if only.as_ref().is_some_and(|t| !w.tag.contains(t.as_str())) {
            continue;
        }
        let (at, old) = render_arm(w, false, None);
        let (_, new) = render_arm(w, true, Some(at));
        println!("== {}: frames at ({:.0}, {:.0})", w.tag, at.0, at.1);
        for (k, rung) in ["x1", "x4", "x16", "x64"].iter().enumerate() {
            let moved = old[k].chunks(3).zip(new[k].chunks(3)).filter(|(a, b)| a != b).count();
            println!("   {rung}: {moved} of {} pixels differ", SCREEN_W * SCREEN_H);
            let (sw, sbs) = side_by_side(&old[k], &new[k]);
            write_png(&format!("{o}/{}_ceiling_{rung}_old_new.png", w.tag), sw, SCREEN_H, sbs);
        }
    }
}
