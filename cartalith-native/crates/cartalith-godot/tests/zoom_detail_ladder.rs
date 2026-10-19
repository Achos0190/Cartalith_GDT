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
//! the terrain-streak spike's `ZD_STREAK` (see `world_params`), Ruling CO's
//! `ZD_FURROWS=<strength>` (the volcano tile furrows; `0` = before),
//! and the exploration overrides `ZD_GAIN`/`ZD_FLOOR`/`ZD_MARGIN`. `ZD_X1=1`
//! adds the fit frame; `ZD_MASK=1` writes each frame's land mask
//! (`mask_frame`, Ruling CU's land-only statistics); `ZD_APP=1` attaches the
//! app's lithology and peak altitude (see the law loop); `ZD_SET` writes
//! tunables over the look by name.
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

/// `ZD_X1=1` adds an `x1` frame per target: **the base raster**, as the app
/// draws x1 (`render::cell_color`, no river ink, no tiles), box-averaged to
/// the fit view's 950 px width over the whole map -- the same frame
/// `terrain_streak_spike.rs`' `write_fit_rgb` writes, here so the ladder's
/// four rungs come from one run. The target does not move it (x1 is the whole
/// map). Added for `EDIFICE_AND_STREAKS_2026-10-18.md`.
fn fit_frame(ctx: &RenderCtx) -> Vec<u8> {
    let (gw, gh) = (ctx.gw, ctx.gh);
    let ow = SCREEN_W;
    let oh = ((gh as f64 * ow as f64 / gw as f64).round() as usize).min(SCREEN_H);
    let mut rgb = vec![0u8; SCREEN_W * SCREEN_H * 3];
    for oy in 0..oh {
        for ox in 0..ow {
            let (x0, x1) = (ox * gw / ow, ((ox + 1) * gw / ow).max(ox * gw / ow + 1));
            let (y0, y1) = (oy * gh / oh, ((oy + 1) * gh / oh).max(oy * gh / oh + 1));
            let mut s = [0.0f64; 3];
            for y in y0..y1 {
                for x in x0..x1 {
                    let (r, g, b) = render::cell_color(ctx, x, y);
                    s[0] += r;
                    s[1] += g;
                    s[2] += b;
                }
            }
            let k = ((x1 - x0) * (y1 - y0)) as f64;
            for c in 0..3 {
                rgb[(oy * SCREEN_W + ox) * 3 + c] = (s[c] / k * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    rgb
}

fn world_params(w: &WorldSpec, gw: usize, gh: usize) -> cartalith_engine::WorldParams {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = w.seed;
    p.map_width_km = w.width_km;
    p.sea_level = 0.42;
    // `ZD_STREAK` (the terrain-streak spike, `TERRAIN_STREAK_SPIKE_2026-10-18.md`):
    // a comma list of edits to the shipped world -- the two
    // `cartalith_engine::StreakSpikeParams` ablations `no_light` and `no_cut`,
    // `carve_off`, `edifice_off` (`volc.edifice_model`), and `rain_bil`, the
    // reference's bilinear rain up-sample the app ran before Ruling CM
    // (`climate.rain_bspline` off), and `pits_unfilled`, the app before
    // Ruling CO's flank-pit fill (`volc.fill_edifice_pits` off), and
    // `orogen_off`, the app before Ruling CU item 1 (`tect.orogen_relief` off). Unset = the
    // shipped world. An unknown token is refused, so a typo cannot render the
    // head world under an arm's name (the retired `rain_bs`, `sub<N>`,
    // `rho8`, `plain_d8`, `gully_off`, `lumps_off`, `a_km`, `c_fill` and
    // `c_late` tokens included).
    if let Ok(v) = std::env::var("ZD_STREAK") {
        for t in v.split(',').filter(|t| !t.is_empty()) {
            match t {
                "rain_bil" => p.climate.rain_bspline = false,
                "no_light" => p.streak.skip_light_pass = true,
                "no_cut" => p.streak.skip_channel_carve = true,
                "carve_off" => p.carve_rivers = false,
                "edifice_off" => p.volc.edifice_model = false,
                // Ruling CO's before: the shipped flank-pit fill off.
                "pits_unfilled" => p.volc.fill_edifice_pits = false,
                // Ruling CU item 1's before: the orogen relief term off.
                "orogen_off" => p.tect.orogen_relief = false,
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
    // The edifice follow-up's pick: the land cell with the largest
    // `volcanic_field` (the youngest edifice's centre). `volcanic_field` is a
    // function of placement alone, so the pick is the same in every arm.
    let mut best_v = (0f32, 0, 0);
    for y in (m..gh - m).step_by(2) {
        for x in (m..gw - m).step_by(2) {
            let i = y * gw + x;
            if f[i] >= sea && ws.volcanic_field[i] > best_v.0 {
                best_v = (ws.volcanic_field[i], x, y);
            }
        }
    }
    if best_v.0 > 0.0 {
        out.push(("volcano", best_v.1 as f64, best_v.2 as f64));
    }
    // And a mid-flank point of the edifices under the volcano and mountain
    // picks: 0.6 of the recovered radius north of the recovered centre
    // (`edifice_pits::EdificeFrame`, from `volcanic_field` alone), where
    // the gullies are. The summit picks above sit inside the summit depression
    // at x64, where no gully is cut.
    let fr = cartalith_terrain::edifice_pits::EdificeFrame::from_volcanic_field(&ws.volcanic_field, gw, gh);
    for (name, px, py) in [("volcano_flank", best_v.1, best_v.2), ("mountain_flank", best_h.1, best_h.2)] {
        let o = fr.owner[py * gw + px];
        if o != cartalith_terrain::edifice_pits::EdificeFrame::NONE {
            let e = &fr.edifices[o as usize];
            let y = (e.cy - 0.6 * e.r).max(m as f64);
            out.push((name, e.cx, y));
        }
    }
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
/// Since Ruling CO item 3 it includes the tile's volcano furrows, through the
/// same `render::add_tile_furrows` call `tile_heights` makes.

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
    let (w, h, mut data) = pyramid_tile_padded(ctx.field, gw, gh, ChunkId::new(z as u32, col, row), TILE_PX, pad, &opts);
    render::add_tile_furrows(ctx, &mut data, w, h, pad, tb);
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

/// `ZD_MASK=1` (the Satellite relief land tone pass, Ruling CU, 2026-10-19):
/// a land mask for one frame, white where the screen pixel's coarse
/// position lies inside land on every side, black elsewhere. It is an
/// **input** mask -- read from the height field and the sea level, never from
/// the rendered colour under test (`MISTAKES.md`, "Measure an effect inside a
/// subset of pixels") -- and it is eroded by one cell (all four bilinear
/// corners must be land; at x1 every cell of the box) so coast and beach
/// pixels, which take the sea's shelf blend, never enter a land statistic.
/// The same screen mapping as [`level_frame`] (and [`fit_frame`] at x1), so
/// mask and frame are pixel-registered.
fn mask_frame(ctx: &RenderCtx, gx: f64, gy: f64, ppc: f64, x1: bool) -> Vec<u8> {
    let (gw, gh) = (ctx.gw, ctx.gh);
    let land = |x: usize, y: usize| ctx.field[y * gw + x] as f64 >= ctx.sea_level;
    let mut out = vec![0u8; SCREEN_W * SCREEN_H];
    if x1 {
        let ow = SCREEN_W;
        let oh = ((gh as f64 * ow as f64 / gw as f64).round() as usize).min(SCREEN_H);
        for oy in 0..oh {
            for ox in 0..ow {
                let (x0, x1) = (ox * gw / ow, ((ox + 1) * gw / ow).max(ox * gw / ow + 1));
                let (y0, y1) = (oy * gh / oh, ((oy + 1) * gh / oh).max(oy * gh / oh + 1));
                let all = (y0..y1).all(|y| (x0..x1).all(|x| land(x, y)));
                out[oy * SCREEN_W + ox] = if all { 255 } else { 0 };
            }
        }
        return out;
    }
    for sy in 0..SCREEN_H {
        let cy = (gy + (sy as f64 + 0.5 - SCREEN_H as f64 / 2.0) / ppc).clamp(0.0, (gh - 1) as f64);
        for sx in 0..SCREEN_W {
            let cx = (gx + (sx as f64 + 0.5 - SCREEN_W as f64 / 2.0) / ppc).clamp(0.0, (gw - 1) as f64);
            let (x0, y0) = (cx as usize, cy as usize);
            let (x1, y1) = ((x0 + 1).min(gw - 1), (y0 + 1).min(gh - 1));
            let all = land(x0, y0) && land(x1, y0) && land(x0, y1) && land(x1, y1);
            out[sy * SCREEN_W + sx] = if all { 255 } else { 0 };
        }
    }
    out
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
        // Ruling CO item 3: the world's edifice frame, attached to every
        // context as `LodSnapshot` attaches it, so the shipped look draws the
        // tile furrows. `ZD_FURROWS=<strength>` overrides the look's
        // `edifice_furrows` (`0` = the tiles before the ruling); unset = the
        // shipped look. A value that does not parse is refused.
        let frame_v = cartalith_terrain::edifice_pits::EdificeFrame::from_volcanic_field(&ws.volcanic_field, gw, gh);
        println!("FURROWS {}: {} edifices recovered", w.tag, frame_v.edifices.len());
        let fur_strength: Option<f64> = std::env::var("ZD_FURROWS").ok().map(|v| v.parse().expect("ZD_FURROWS: expected a number"));
        let x1 = std::env::var("ZD_X1").ok().is_some_and(|v| v == "1");
        let rungs: Vec<(&str, f64)> = x1.then_some(("x1", 0.0)).into_iter().chain(RUNGS.iter().copied()).collect();
        // `ZD_ONLY=<target,...>` keeps those targets only.
        if let Ok(o) = std::env::var("ZD_ONLY") {
            tg.retain(|t| o.split(',').any(|n| n == t.0));
        }
        for &law in &laws {
            let mut a = TerrainAppearance { zoom_detail_km: law, ..TerrainAppearance::for_tier(QualityTier::Quality).with_look(look) };
            if let Some(f) = fur_strength {
                a.edifice_furrows = f;
            }
            // `ZD_APP=1` (Ruling CU's tone pass): the two inputs the app's
            // screen context carries and this harness otherwise leaves out --
            // the world's peak altitude in the look (`WorldGen::
            // appearance_rebased` fills `npr.peak_m`, which the Satellite
            // relief slope stages need to fire at all) and the lithology
            // (`build_lithology` over the same field, as `lib.rs`'s screen
            // context attaches it), so rock is drawn from the seven rock
            // palettes the app draws it from. Unset = the frames as before.
            // `ZD_SET=key=value,...`: tunables written over the look by name
            // (`TerrainAppearance::set_tunable`, clamped to their declared
            // ranges), for tuning a look without a rebuild. An unknown key or
            // an unparsable value is refused rather than silently ignored.
            if let Ok(v) = std::env::var("ZD_SET") {
                for kv in v.split(',').filter(|t| !t.is_empty()) {
                    let (k, x) = kv.split_once('=').unwrap_or_else(|| panic!("ZD_SET: expected key=value, got {kv}"));
                    let x: f64 = x.parse().unwrap_or_else(|_| panic!("ZD_SET: {k} needs a number"));
                    assert!(a.set_tunable(k, x), "ZD_SET: unknown tunable {k}");
                }
            }
            let app = std::env::var("ZD_APP").ok().is_some_and(|v| v == "1");
            if app {
                a.npr.peak_m = p.peak_m;
            }
            let lith = app.then(|| cartalith_civ::build_lithology(&ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, ws.sea_level as f64));
            let pre = GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, &a, Some(p.map_width_km));
            let mut ctx = RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, 55.0, 5.0, a, &pre).expect("context").with_edifice_frame(&frame_v, p.peak_m);
            if let Some(l) = lith.as_ref() {
                ctx = ctx.with_lithology(l);
            }
            let tf = TileFields::new(&ctx, None);
            for &(name, gx, gy) in &tg {
                for &(rung, ppc) in &rungs {
                    let t1 = std::time::Instant::now();
                    let rgb = if rung == "x1" { fit_frame(&ctx) } else { frame(&ctx, &tf, gx, gy, ppc, w.seed) };
                    assert!(rgb.iter().any(|&v| v != rgb[0]), "an empty frame");
                    let png = cartalith_assets::raster::encode_png_rgb8(SCREEN_W as u32, SCREEN_H as u32, rgb, cartalith_assets::raster::PngColour::Data).unwrap();
                    let tag = std::env::var("ZD_TAG").unwrap_or_default();
                    let path = format!("{out}/{}_{name}_{rung}_law{law}{tag}.png", w.tag);
                    std::fs::write(&path, png).unwrap();
                    println!("  {name} ({gx},{gy}) {rung} ppc {ppc} law {law}: {:.2}s -> {path}", t1.elapsed().as_secs_f64());
                    // `ZD_MASK=1`: the frame's land mask beside it (see
                    // [`mask_frame`]); look-independent, so one per frame name.
                    if std::env::var("ZD_MASK").ok().is_some_and(|v| v == "1") {
                        let m = mask_frame(&ctx, gx, gy, ppc, rung == "x1");
                        let rgb: Vec<u8> = m.iter().flat_map(|&v| [v, v, v]).collect();
                        let png = cartalith_assets::raster::encode_png_rgb8(SCREEN_W as u32, SCREEN_H as u32, rgb, cartalith_assets::raster::PngColour::Data).unwrap();
                        std::fs::write(format!("{out}/{}_{name}_{rung}_mask.png", w.tag), png).unwrap();
                    }
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

/// Ruling CO item 3's cost: one deep-zoom tile with the volcano furrows off
/// (`edifice_furrows = 0`) and at the shipped strength, at levels 6 and 8,
/// over 24 tiles on 314159's largest volcano (the land block round cell
/// 322, 522, the ladder's `volcano` pick), alternated so drift hits both
/// arms. Prints median and min..max per arm. Run **alone** (`MISTAKES.md`,
/// "Quote a timing").
///
/// Protects: nothing (a measurement, `#[ignore]`d).
#[test]
#[ignore = "generates a 2048x1311 world and times tile synthesis; run explicitly and alone"]
fn furrow_tile_cost() {
    let (gw, gh) = (2048usize, 1311usize);
    let w = WorldSpec { tag: "s314159_earth_800km", seed: 314159, archetype: Some(EARTH), width_km: 800.0 };
    let p = world_params(&w, gw, gh);
    let ws = cartalith_engine::generate_terrain(&p);
    let fr = cartalith_terrain::edifice_pits::EdificeFrame::from_volcanic_field(&ws.volcanic_field, gw, gh);
    let shipped = TerrainAppearance::for_tier(QualityTier::Quality).with_look(render::LOOK_VIBRANT);
    assert!(shipped.edifice_furrows > 0.0, "the shipped look draws furrows");
    let arms = [("furrows_off", TerrainAppearance { edifice_furrows: 0.0, ..shipped.clone() }), ("furrows_on", shipped.clone())];
    let built: Vec<_> = arms
        .iter()
        .map(|(_, a)| GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, a, Some(p.map_width_km)))
        .collect();
    let ctxs: Vec<_> = arms
        .iter()
        .zip(&built)
        .map(|((_, a), pre)| RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, 55.0, 5.0, a.clone(), pre).unwrap().with_edifice_frame(&fr, p.peak_m))
        .collect();
    let tfs: Vec<_> = ctxs.iter().map(|c| TileFields::new(c, None)).collect();
    for z in [6i32, 8] {
        let n = pyramid_dims(z).cols;
        let mut t = [Vec::new(), Vec::new()];
        for k in 0..24u32 {
            let step = (gw - 1) as f64 / n as f64;
            let (c0, r0) = ((316.0 / step) as u32, (516.0 / step) as u32);
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
