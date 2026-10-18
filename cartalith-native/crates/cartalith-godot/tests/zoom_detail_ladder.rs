//! **Ruling CI item 1** (`LARGE_ITEM_RULINGS.md`, owner, 2026-10-17) -- the
//! zoom ladder `RENDER_TARGET_GAP_ASSESSMENT_2026-10-17.md` §4 measured, rebuilt
//! in Rust over the real deep-zoom tile synthesiser so the kilometre-keyed
//! zoom-detail law can be measured on and off without a display.
//!
//! What it is and is not:
//!
//! - It **is** the shipped tile path: `pyramid_tile_padded` with the opts
//!   `render::tile_amplify_opts` picks (the one function `lod_bridge::
//!   tile_heights` calls), coloured by `render::render_biome_tile_rgba_water`
//!   over a real generated world under the app's own generation parameters
//!   (`params::defaults()`), an archetype's knobs (or Classic's defaults), and
//!   the app's default look (Quality tier + Natural Vibrant) or, with
//!   `ZD_LOOK=satellite`, the Satellite relief look.
//! - It is **not** the screen: no river layer, no lithology, no RV-3 valley
//!   field, no LOD-D3 morph, and the screen is composited here by bilinear
//!   sampling of the tile at the level `pyramid_level_for_zoom` picks. Those
//!   omissions make it a *terrain-only* ladder, which is the stricter test of
//!   the requirement: the assessment found the river strokes were the one thing
//!   that stayed sharp. The windowed Godot ladder (`_zoomdetail_probe.gd`) is
//!   the screen measurement; this is the fast one.
//!
//! It writes one PNG per (world, target, zoom, law) into `ZD_OUT` and nothing
//! else; the statistics are computed from those files by the same Python
//! script that reads the Godot captures, so the two ladders share one set of
//! definitions. `#[ignore]`d: it generates up to four 2048 x 1311 worlds
//! (`ZD_WORLDS` picks them by tag prefix). Environment: `ZD_OUT` (required),
//! `ZD_LAWS`, `ZD_LOOK`, `ZD_TAG`, `ZD_WORLDS`, `ZD_PICKS`, `ZD_GW`/`ZD_GH`,
//! the terrain-streak spike's `ZD_STREAK` and `ZD_FLOW` (see `world_params`),
//! and the exploration overrides `ZD_GAIN`/`ZD_FLOOR`/`ZD_MARGIN`.
//!
//! ```text
//! ZD_OUT=<dir> cargo test -p cartalith-godot --test zoom_detail_ladder -- --ignored --nocapture
//! ```
//!
//! Protects: nothing by itself (a measurement harness, `#[ignore]`d); it
//! asserts only that each world is not flat and each frame is non-empty.

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
use render::{GridPrecompute, QualityTier, RenderCtx, TerrainAppearance, TileBounds, TileFields};

/// `lod_bridge::TILE_PX` -- the interactive tile's pixel size. Restated
/// because `lod_bridge.rs` cannot be `#[path]`-included (it reaches
/// `crate::river_stroke`); `the_harness_constants_are_the_bridges` is not
/// possible from here, so the value is quoted with its source.
const TILE_PX: usize = 256;
/// `lod_bridge::MAX_LEVEL`, same reason.
const MAX_LEVEL: i32 = 10;
/// The assessment's own capture size: the map viewport at its window (§1).
const SCREEN_W: usize = 950;
const SCREEN_H: usize = 858;

/// One world of the assessment's three (§1, "Worlds").
struct WorldSpec {
    tag: &'static str,
    seed: i32,
    /// An archetype's knobs, or `None` for Classic (the app's own
    /// `world_structure` defaults, as the shell's `archetype: ""` generates).
    archetype: Option<[f64; 5]>,
    width_km: f64,
}

/// `lib.rs`'s `ARCHETYPES` rows for the two archetypes the assessment used,
/// quoted (the table is private to the cdylib).
const EARTH: [f64; 5] = [0.30, 0.50, 0.60, 0.60, 0.20];
const ARCHIPELAGO: [f64; 5] = [0.15, 0.90, 0.80, 0.30, 0.50];

/// The assessment's zoom rungs as screen pixels per coarse cell (§1: x4 is
/// 1.9, x16 7.4, x64 29.7 px per cell on its 950 x 858 viewport). x1 is the
/// base raster, not tiles, and so is not this harness's to draw.
const RUNGS: [(&str, f64); 3] = [("x4", 1.86), ("x16", 7.42), ("x64", 29.7)];

fn world_params(w: &WorldSpec, gw: usize, gh: usize) -> cartalith_engine::WorldParams {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = w.seed;
    p.map_width_km = w.width_km;
    p.sea_level = 0.42;
    // `ZD_STREAK` (the terrain-streak spike, `TERRAIN_STREAK_SPIKE_2026-10-18.md`):
    // a comma list of `cartalith_engine::StreakSpikeParams` switches to turn
    // on -- `rain_bs`, `sub<N>` (carve_subcell = N/10), `rho8`, `no_light`,
    // `no_cut`, `plain_d8`, `carve_off`, and the existing `volc.edifice_model`
    // as `edifice_off`. Unset = the shipped world. An unknown
    // token is refused, so a typo cannot render the head world under a
    // variant's name.
    if let Ok(v) = std::env::var("ZD_STREAK") {
        for t in v.split(',').filter(|t| !t.is_empty()) {
            match t {
                "rain_bs" => p.streak.rain_bspline = true,
                "rho8" => p.streak.stream_rho8 = true,
                "no_light" => p.streak.skip_light_pass = true,
                "no_cut" => p.streak.skip_channel_carve = true,
                "plain_d8" => p.streak.channel_plain_d8 = true,
                "carve_off" => p.carve_rivers = false,
                "edifice_off" => p.volc.edifice_model = false,
                s if s.starts_with("sub") => p.streak.carve_subcell = s[3..].parse::<f64>().expect("ZD_STREAK sub<N>") / 10.0,
                other => panic!("ZD_STREAK: unknown switch {other}"),
            }
        }
    }
    if let Some(k) = w.archetype {
        p.world_structure = cartalith_engine::WorldStructureParams {
            enabled: true,
            continentality: k[0],
            fragmentation: k[1],
            tectonic_energy: k[2],
            ocean_depth: k[3],
            hotspot_density: k[4],
        };
    }
    p
}

/// Picks five targets on the grid, by **inputs** and never by the value under
/// test (`MISTAKES.md`, "Measure an effect inside a subset of pixels"):
/// mountain = highest land cell; coast = the land cell beside the sea closest
/// to the map centre; forest = the wettest temperate land cell at least 20
/// cells from the sea; forest2 = the same rule at least 250 cells from
/// forest; cold = the coldest land cell. Each is kept 40 cells
/// from the edge so the x4 frame stays on the map.
fn targets(ws: &cartalith_engine::WorldState, gw: usize, gh: usize) -> Vec<(&'static str, f64, f64)> {
    let sea = ws.sea_level as f32;
    let f = &ws.field;
    let m = 140usize;
    let inside = |x: usize, y: usize| x >= m && y >= m && x + m < gw && y + m < gh;
    let mut best_h = (f32::MIN, 0, 0);
    let mut best_c = (f64::MAX, 0, 0);
    let mut best_t = (f32::MAX, 0, 0);
    let mut best_w = (f32::MIN, 0, 0);
    // A coarse distance-to-sea: the forest pick needs "inland" without the
    // value under test, and a 20-cell box scan at a stride is enough.
    let near_sea = |x: usize, y: usize| -> bool {
        let r = 20i64;
        let mut yy = y as i64 - r;
        while yy <= y as i64 + r {
            let mut xx = x as i64 - r;
            while xx <= x as i64 + r {
                if xx >= 0 && yy >= 0 && (xx as usize) < gw && (yy as usize) < gh && f[yy as usize * gw + xx as usize] < sea {
                    return true;
                }
                xx += 5;
            }
            yy += 5;
        }
        false
    };
    for y in (m..gh - m).step_by(4) {
        for x in (m..gw - m).step_by(4) {
            if !inside(x, y) {
                continue;
            }
            let i = y * gw + x;
            if f[i] < sea {
                continue;
            }
            if f[i] > best_h.0 {
                best_h = (f[i], x, y);
            }
            let coast = f[i - 1] < sea || f[i + 1] < sea || f[i - gw] < sea || f[i + gw] < sea;
            if coast {
                let d = ((x as f64 - gw as f64 / 2.0).powi(2) + (y as f64 - gh as f64 / 2.0).powi(2)).sqrt();
                if d < best_c.0 {
                    best_c = (d, x, y);
                }
            }
            let t = ws.temperature[i];
            if (5.0..20.0).contains(&t) && ws.rainfall[i] > best_w.0 && !near_sea(x, y) {
                best_w = (ws.rainfall[i], x, y);
            }
        }
    }
    // The cold pick keeps 150 cells from the peak so the two frames are
    // different ground (the coldest land is often the summit itself).
    for y in (m..gh - m).step_by(4) {
        for x in (m..gw - m).step_by(4) {
            let i = y * gw + x;
            let far = (x as f64 - best_h.1 as f64).hypot(y as f64 - best_h.2 as f64) > 150.0;
            if f[i] >= sea && far && ws.temperature[i] < best_t.0 {
                best_t = (ws.temperature[i], x, y);
            }
        }
    }
    let mut out = vec![("mountain", best_h.1 as f64, best_h.2 as f64), ("cold", best_t.1 as f64, best_t.2 as f64)];
    if best_c.0 < f64::MAX {
        out.push(("coast", best_c.1 as f64, best_c.2 as f64));
    }
    if best_w.0 > f32::MIN {
        out.push(("forest", best_w.1 as f64, best_w.2 as f64));
        // A second forest, by the same inputs, at least 250 cells from the
        // first: one forest per world is one sample (`MISTAKES.md`).
        let mut best_w2 = (f32::MIN, 0, 0);
        for y in (m..gh - m).step_by(4) {
            for x in (m..gw - m).step_by(4) {
                let i = y * gw + x;
                let far = (x as f64 - best_w.1 as f64).hypot(y as f64 - best_w.2 as f64) > 250.0;
                if f[i] >= sea && far && (5.0..20.0).contains(&ws.temperature[i]) && ws.rainfall[i] > best_w2.0 && !near_sea(x, y) {
                    best_w2 = (ws.rainfall[i], x, y);
                }
            }
        }
        if best_w2.0 > f32::MIN {
            out.push(("forest2", best_w2.1 as f64, best_w2.2 as f64));
        }
    }
    out
}

/// One synthesized tile, as `lod_bridge::tile_heights` +
/// `synthesize_tile_rgba_with_z_base` build it minus the river layer and the
/// RV-3 water tile (this harness's context has no valley field, so the shaded
/// and the water heights are the same field).
fn tile(ctx: &RenderCtx, tf: &TileFields, z: i32, col: u32, row: u32, seed: i32) -> (Vec<u8>, usize, usize, TileBounds) {
    let (gw, gh) = (ctx.gw, ctx.gh);
    let b = pyramid_tile_bounds(gw, gh, z, col, row);
    let n = pyramid_dims(z).cols as usize;
    let d = tile_dims(&Region { x: 0, y: 0, w: gw - 1, h: gh - 1 }, n, n, TILE_PX);
    let tb = TileBounds { x: b.x, y: b.y, w: b.w, h: b.h };
    let pad = render::tile_halo_px(ctx, d.w, d.h, tb);
    let mut opts = render::tile_amplify_opts(ctx, seed, cartalith_terrain::amplify::z_base_for_tile_size(TILE_PX));
    // Exploration overrides of the shipped law's constants (`ZD_GAIN`,
    // `ZD_FLOOR`), for re-taking the design measurement; unset = shipped.
    if let Some(km) = opts.km.as_mut() {
        if let Some(g) = std::env::var("ZD_GAIN").ok().and_then(|v| v.parse().ok()) {
            km.octave_gain = g;
        }
        if let Some(f) = std::env::var("ZD_FLOOR").ok().and_then(|v| v.parse().ok()) {
            km.relief_floor = f;
        }
        if let Some(m) = std::env::var("ZD_MARGIN").ok().and_then(|v| v.parse().ok()) {
            km.nyquist_margin = m;
        }
    }
    let (w, h, data) = pyramid_tile_padded(ctx.field, gw, gh, ChunkId::new(z as u32, col, row), TILE_PX, pad, &opts);
    let rgba = render::render_biome_tile_rgba_water(ctx, &data, None, w, h, pad, tb, tf, None);
    assert_eq!(rgba.len(), w * h * 4, "the colouriser refused the tile");
    (rgba, w, h, tb)
}

/// A `SCREEN_W x SCREEN_H` frame centred on cell `(gx, gy)` at `ppc` screen
/// pixels per cell: every pixel samples the tile of the level the shell would
/// pick, bilinearly, at its own coarse coordinate. Returns RGB.
fn frame(ctx: &RenderCtx, tf: &TileFields, gx: f64, gy: f64, ppc: f64, seed: i32) -> Vec<u8> {
    let z = pyramid_level_for_zoom(ppc, ctx.gw as f64, TILE_PX as f64, Some(MAX_LEVEL));
    // LOD-D3's morph, as `lod_tile.gdshader` applies it: the level's tile over
    // its parent at `smoothstep(log2(want) − z + 0.5)` (`lod_bridge::
    // morph_for_zoom`, restated here for the reason `TILE_PX` is).
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

/// One level's tiles composited to the screen frame (see [`frame`]).
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

#[test]
#[ignore = "generates three 2048x1311 worlds and renders a zoom ladder; run explicitly"]
fn zoom_detail_ladder() {
    let out = std::env::var("ZD_OUT").expect("set ZD_OUT to an output directory");
    std::fs::create_dir_all(&out).unwrap();
    let (gw, gh) = (
        std::env::var("ZD_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(2048usize),
        std::env::var("ZD_GH").ok().and_then(|v| v.parse().ok()).unwrap_or(1311usize),
    );
    // `ZD_LAWS` = comma list of `zoom_detail_km` values to render (default
    // `0,1`: the reference law and the shipped kilometre law).
    let laws: Vec<f64> = std::env::var("ZD_LAWS").unwrap_or_else(|_| "0,1".into()).split(',').map(|s| s.parse().unwrap()).collect();
    let worlds = [
        WorldSpec { tag: "s24601_archipelago_800km", seed: 24601, archetype: Some(ARCHIPELAGO), width_km: 800.0 },
        WorldSpec { tag: "s314159_earth_800km", seed: 314159, archetype: Some(EARTH), width_km: 800.0 },
        WorldSpec { tag: "s24601_earth_3000km", seed: 24601, archetype: Some(EARTH), width_km: 3000.0 },
        WorldSpec { tag: "s71077345_classic_800km", seed: 71077345, archetype: None, width_km: 800.0 },
    ];
    // `ZD_LOOK` = `vibrant` (the default, the app's look) or `satellite`
    // (Ruling CI item 2's "Satellite relief"); anything else is refused.
    let look = match std::env::var("ZD_LOOK").unwrap_or_else(|_| "vibrant".into()).as_str() {
        "vibrant" => render::LOOK_VIBRANT,
        "satellite" => render::LOOK_SATELLITE,
        other => panic!("ZD_LOOK={other}: expected vibrant or satellite"),
    };
    let only = std::env::var("ZD_WORLDS").ok();
    for w in &worlds {
        if let Some(o) = &only
            && !o.split(',').any(|t| w.tag.starts_with(t))
        {
            continue;
        }
        let p = world_params(w, gw, gh);
        let t0 = std::time::Instant::now();
        let ws = cartalith_engine::generate_terrain(&p);
        println!("{}: generated in {:.1}s", w.tag, t0.elapsed().as_secs_f64());
        let lo = ws.field.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = ws.field.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(hi - lo > 0.2, "{} is nearly flat", w.tag);
        let mut tg = targets(&ws, gw, gh);
        // `ZD_PICKS=<file>`: the windowed probe's own picks
        // (`zoomdetail_<tag>.json`'s `targets`), so the two ladders frame the
        // same cells of the same world.
        if let Ok(f) = std::env::var("ZD_PICKS") {
            let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(f).unwrap()).unwrap();
            if let Some(arr) = j[w.tag]["targets"].as_array() {
                tg = arr
                    .iter()
                    .map(|t| {
                        let name: &'static str = Box::leak(t[0].as_str().unwrap().to_string().into_boxed_str());
                        (name, t[1].as_f64().unwrap(), t[2].as_f64().unwrap())
                    })
                    .collect();
            }
        }
        // One line per world the windowed probe's `--targets` file is built
        // from, so the two ladders frame the same cells.
        println!("TARGETS {} {} {:?}", w.tag, w.seed, tg);
        for &law in &laws {
            let a = TerrainAppearance { zoom_detail_km: law, ..TerrainAppearance::for_tier(QualityTier::Quality).with_look(look) };
            // `ZD_FLOW=mfd`: the streak spike's display-only flow (variant
            // (c), `cartalith_hydrology::compute_flow_mfd`) in place of the
            // D8 accumulation the render reads; unset = shipped.
            let mfd = (std::env::var("ZD_FLOW").as_deref() == Ok("mfd")).then(|| {
                let route = cartalith_hydrology::build_routing_surface(&ws.field, gw, gh, ws.sea_level as f64, p.world);
                cartalith_hydrology::compute_flow_mfd(gw, gh, &route, &ws.rainfall, p.world, 1.1)
            });
            let flow: &[f32] = mfd.as_deref().unwrap_or(&ws.flow_discharge);
            let pre = GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(flow), gw, gh, ws.sea_level as f64, p.world, &a, Some(p.map_width_km));
            let ctx = RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(flow), gw, gh, ws.sea_level as f64, p.world, 55.0, 5.0, a, &pre).expect("context");
            let tf = TileFields::new(&ctx, None);
            for &(name, gx, gy) in &tg {
                for &(rung, ppc) in &RUNGS {
                    let t1 = std::time::Instant::now();
                    let rgb = frame(&ctx, &tf, gx, gy, ppc, w.seed);
                    assert!(rgb.iter().any(|&v| v != rgb[0]), "an empty frame");
                    let png = cartalith_assets::raster::encode_png_rgb8(SCREEN_W as u32, SCREEN_H as u32, rgb, cartalith_assets::raster::PngColour::Data).unwrap();
                    let tag = std::env::var("ZD_TAG").unwrap_or_default();
                    let path = format!("{out}/{}_{name}_{rung}_law{law}{tag}.png", w.tag);
                    std::fs::write(&path, png).unwrap();
                    println!("  {name} ({gx},{gy}) {rung} ppc {ppc} law {law}: {:.2}s -> {path}", t1.elapsed().as_secs_f64());
                }
            }
        }
    }
}

/// The cost bar of Ruling CI item 1: one deep-zoom tile (height synthesis and
/// colour, `tile` above) with the law off (`zoom_detail_km = 0`, the
/// reference schedule) and at the shipped strength, at levels 6 and 8, over 24
/// tiles each, alternated so drift hits both arms. Prints median and min..max
/// per arm. Run **alone** (`MISTAKES.md`, "Quote a timing").
///
/// Protects: nothing (a measurement, `#[ignore]`d).
#[test]
#[ignore = "generates a 2048x1311 world and times tile synthesis; run explicitly and alone"]
fn zoom_detail_tile_cost() {
    let (gw, gh) = (2048usize, 1311usize);
    let w = WorldSpec { tag: "s314159_earth_800km", seed: 314159, archetype: Some(EARTH), width_km: 800.0 };
    let p = world_params(&w, gw, gh);
    let ws = cartalith_engine::generate_terrain(&p);
    let shipped = TerrainAppearance::for_tier(QualityTier::Quality).with_look(render::LOOK_VIBRANT);
    let arms = [("off", TerrainAppearance { zoom_detail_km: 0.0, ..shipped.clone() }), ("on", shipped.clone())];
    let built: Vec<_> = arms
        .iter()
        .map(|(_, a)| GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, a, Some(p.map_width_km)))
        .collect();
    let ctxs: Vec<_> = arms
        .iter()
        .zip(&built)
        .map(|((_, a), pre)| RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, 55.0, 5.0, a.clone(), pre).unwrap())
        .collect();
    let tfs: Vec<_> = ctxs.iter().map(|c| TileFields::new(c, None)).collect();
    for z in [6i32, 8] {
        let n = pyramid_dims(z).cols;
        let mut t = [Vec::new(), Vec::new()];
        for k in 0..24u32 {
            // A 6x4 block around the windowed probe's mountain pick for this
            // world (cell 796, 268): land, where the law does its work.
            let step = (gw - 1) as f64 / n as f64;
            let (c0, r0) = ((796.0 / step) as u32, (268.0 / step) as u32);
            let (col, row) = ((c0 + k % 6).min(n - 1), (r0 + k / 6).min(n - 1));
            for arm in 0..2 {
                let t0 = std::time::Instant::now();
                let _ = tile(&ctxs[arm], &tfs[arm], z, col, row, w.seed);
                t[arm].push(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
        for (arm, v) in t.iter_mut().enumerate() {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            println!("COST z{z} {}: median {:.2} ms ({:.2}..{:.2}) over {}", arms[arm].0, v[v.len() / 2], v[0], v[v.len() - 1], v.len());
        }
    }
}
