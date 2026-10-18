//! Ruling CN's measurement harnesses (`COLOUR_PAINT_SCOPE.md` CP-1): the
//! frames the scope's acceptance is judged on, the cost bar, the fill
//! helper's feasibility, and the save size. Every test here is `#[ignore]`d
//! and asserts only that its inputs exist and its frames are non-empty; the
//! judgement is made by reading the frames, and the numbers are printed.
//!
//! The deep-zoom tile path is `tests/zoom_detail_ladder.rs`'s, restated (its
//! `tile`/`frame`/`level_frame` cannot be shared across test targets): the
//! shipped `pyramid_tile_padded` + `render_biome_tile_rgba_water` under the
//! app's generation defaults and the app's default look (Quality tier +
//! Natural Vibrant), composited to a 950 x 858 frame at the assessment's
//! rungs. **No river layer, no lithology, no valley field**, as there; the
//! paint is attached exactly as `WorldGen` will attach it (CP-2).
//!
//! ```text
//! CP_OUT=<dir> cargo test --release -p cartalith-godot --test colour_paint_frames -- --ignored --nocapture --test-threads 1
//! ```

#[path = "../src/render.rs"]
#[allow(dead_code)]
mod render;
#[path = "../src/params.rs"]
#[allow(dead_code)]
mod params;

use std::collections::{BinaryHeap, HashMap};

use cartalith_engine::bake::pyramid_tile_padded;
use cartalith_spatial::colour_paint::{ColourBlend, ColourMark, ColourPaint, ColourPaintDoc, MarkShape, DOC_VERSION};
use cartalith_spatial::pyramid::{pyramid_dims, pyramid_level_for_zoom, pyramid_tile_bounds, ChunkId};
use cartalith_spatial::{tile_dims, Region};
use render::{GridPrecompute, QualityTier, RenderCtx, TerrainAppearance, TileBounds, TileFields};

/// `lod_bridge::TILE_PX`, restated (see `zoom_detail_ladder.rs`).
const TILE_PX: usize = 256;
/// `lod_bridge::MAX_LEVEL`, restated.
const MAX_LEVEL: i32 = 10;
/// The assessment's capture size (`RENDER_TARGET_GAP_ASSESSMENT_2026-10-17.md` §1).
const SCREEN_W: usize = 950;
const SCREEN_H: usize = 858;
/// The assessment's rungs, screen pixels per coarse cell. x1 is the base map
/// at fit (the whole map across `SCREEN_W`), drawn by [`base_frame`].
const RUNGS: [(&str, f64); 3] = [("x4", 1.86), ("x16", 7.42), ("x64", 29.7)];

struct WorldSpec {
    tag: &'static str,
    seed: i32,
    archetype: Option<[f64; 5]>,
    width_km: f64,
}

/// `lib.rs`'s `ARCHETYPES` Archipelago row, quoted (private to the cdylib).
const ARCHIPELAGO: [f64; 5] = [0.15, 0.90, 0.80, 0.30, 0.50];

/// The two worlds: Classic 71077345 (the coordinator's "dome" world) and
/// Archipelago 24601 (volcanic islands), both 800 km wide.
const WORLDS: [WorldSpec; 2] = [
    WorldSpec { tag: "s71077345_classic_800km", seed: 71077345, archetype: None, width_km: 800.0 },
    WorldSpec { tag: "s24601_archipelago_800km", seed: 24601, archetype: Some(ARCHIPELAGO), width_km: 800.0 },
];

fn world_params(w: &WorldSpec, gw: usize, gh: usize) -> cartalith_engine::WorldParams {
    let mut p = params::defaults();
    p.gw = gw;
    p.gh = gh;
    p.tect.seed = w.seed;
    p.map_width_km = w.width_km;
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

/// The base map as the screen draws it below the deep-zoom switch: one pixel
/// per cell (`cell_color`, truncated as `build_color_texture` does, then the
/// finishing stages), shown at `ppc` screen pixels per cell around `(gx,
/// gy)` by nearest sampling (the shell's base view filter). x1 is the whole
/// map at fit: `ppc = SCREEN_W / gw`, centred.
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

/// Craters found by the INPUT height alone (never by colour): a land cell
/// that 15 or more of 16 rays out to 14 cells see rising at least 0.06
/// (normalised height) above it -- a rimmed pit. Scored by the mean rise;
/// the best, then the best at least 150 cells from it. Returns `(x, y,
/// rim)` with `rim` the median of the rays' maxima, and the median distance
/// to them in cells.
fn rimmed_pits(ws: &cartalith_engine::WorldState, gw: usize, gh: usize) -> Vec<(usize, usize, f32, f64)> {
    let f = &ws.field;
    let sea = ws.sea_level as f32;
    let m = 60usize;
    let mut cands: Vec<(f32, usize, usize, f32, f64)> = vec![];
    for y in (m..gh - m).step_by(2) {
        for x in (m..gw - m).step_by(2) {
            let h = f[y * gw + x];
            if h < sea {
                continue;
            }
            let (up, rise, rim, rim_d) = rays(f, gw, x, y);
            if up >= 15 {
                cands.push((rise, x, y, rim, rim_d));
            }
        }
    }
    cands.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut out: Vec<(usize, usize, f32, f64)> = vec![];
    for c in cands {
        if out.len() < 2 && out.iter().all(|o| (c.1 as f64 - o.0 as f64).hypot(c.2 as f64 - o.1 as f64) > 150.0) {
            out.push((c.1, c.2, c.3, c.4));
        }
    }
    out
}

/// The 16 rays out to 14 cells from `(x, y)`: how many rise at least 0.06
/// above it, the mean rise, the median of the rays' maxima (the rim) and the
/// median distance to them in cells.
fn rays(f: &[f32], gw: usize, x: usize, y: usize) -> (usize, f32, f32, f64) {
    let h = f[y * gw + x];
    let mut maxes = vec![];
    for k in 0..16 {
        let a = k as f64 * std::f64::consts::TAU / 16.0;
        let mut best = (f32::MIN, 0.0);
        for r in 1..=14 {
            let (qx, qy) = ((x as f64 + r as f64 * a.cos()).round() as usize, (y as f64 + r as f64 * a.sin()).round() as usize);
            let v = f[qy * gw + qx];
            if v > best.0 {
                best = (v, r as f64);
            }
        }
        maxes.push(best);
    }
    let up = maxes.iter().filter(|b| b.0 >= h + 0.06).count();
    let rise = maxes.iter().map(|b| b.0 - h).sum::<f32>() / 16.0;
    let mut ms: Vec<f32> = maxes.iter().map(|b| b.0).collect();
    ms.sort_by(f32::total_cmp);
    let mut ds: Vec<f64> = maxes.iter().map(|b| b.1).collect();
    ds.sort_by(f64::total_cmp);
    (up, rise, ms[8], ds[8])
}

/// **The fill helper's second mode** (`COLOUR_PAINT_SCOPE.md` CP-5): a
/// breached crater is not a closed depression, so "fill to the pour point"
/// stops at the breach. Instead: every cell below `level` connected to the
/// seed and within `cap` cells of it (the cap closes the breach with an arc),
/// as a mask, box-blurred twice so its contour at one half is a smooth curve
/// rather than a cell staircase, traced in km. Returns the ring and the cell
/// count.
fn capped_fill(field: &[f32], gw: usize, seed: (usize, usize), level: f32, cap: f64, kpc: f64) -> Option<(Vec<[f64; 2]>, usize)> {
    let half = cap.ceil() as usize + 3;
    let (x0, y0) = (seed.0 - half, seed.1 - half);
    let w = 2 * half + 1;
    let mut mask = vec![0f32; w * w];
    let mut stack = vec![(half, half)];
    mask[half * w + half] = 1.0;
    let mut count = 0;
    while let Some((x, y)) = stack.pop() {
        count += 1;
        for (dx, dy) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
            let (nx, ny) = ((x as i64 + dx) as usize, (y as i64 + dy) as usize);
            if nx >= w || ny >= w || mask[ny * w + nx] > 0.0 {
                continue;
            }
            if (nx as f64 - half as f64).hypot(ny as f64 - half as f64) > cap || field[(y0 + ny) * gw + x0 + nx] >= level {
                continue;
            }
            mask[ny * w + nx] = 1.0;
            stack.push((nx, ny));
        }
    }
    for _ in 0..2 {
        let src = mask.clone();
        for y in 1..w - 1 {
            for x in 1..w - 1 {
                let mut acc = 0.0;
                for dy in 0..3 {
                    for dx in 0..3 {
                        acc += src[(y + dy - 1) * w + x + dx - 1];
                    }
                }
                mask[y * w + x] = acc / 9.0;
            }
        }
    }
    let p = (half as f64 + 0.5, half as f64 + 0.5);
    let ring = cartalith_spatial::contour_polylines(&mask, w, w, 0.5)
        .into_iter()
        .filter(|pl| pl.len() > 3 && pl[0] == pl[pl.len() - 1])
        .map(|pl| pl[..pl.len() - 1].to_vec())
        .filter(|r| inside(r, [p.0, p.1]))
        .min_by_key(|r| r.len())?;
    Some((ring.into_iter().map(|(x, y)| [(x + x0 as f64) * kpc, (y + y0 as f64) * kpc]).collect(), count))
}

/// **The fill helper's prototype** (`COLOUR_PAINT_SCOPE.md` CP-5): "fill the
/// crater". From the lowest cell within `seek` cells of `(cx, cy)`, grow the
/// depression the way water fills it -- always take the lowest cell on the
/// region's boundary, raise the level to it, and stop at the first boundary
/// cell lower than the level reached (the pour point). Returns the spill
/// level, the region's cell count and the seed, or `None` when the region is
/// a single cell (no depression).
fn crater_fill(field: &[f32], gw: usize, gh: usize, cx: usize, cy: usize, seek: usize, max_cells: usize) -> Option<(f64, usize, (usize, usize))> {
    let mut seed = (cx, cy);
    for y in cy.saturating_sub(seek)..(cy + seek).min(gh) {
        for x in cx.saturating_sub(seek)..(cx + seek).min(gw) {
            if (x as f64 - cx as f64).hypot(y as f64 - cy as f64) <= seek as f64 && field[y * gw + x] < field[seed.1 * gw + seed.0] {
                seed = (x, y);
            }
        }
    }
    #[derive(PartialEq)]
    struct C(f32, usize);
    impl Eq for C {}
    impl PartialOrd for C {
        fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for C {
        // A min-heap on height, ties by index for determinism.
        fn cmp(&self, o: &Self) -> std::cmp::Ordering {
            o.0.total_cmp(&self.0).then(o.1.cmp(&self.1))
        }
    }
    // The field is normalised and clipped at its maximum: a summit rim at the
    // ceiling is a flat wall the flood can only cross by spreading over the
    // whole clipped plateau. Treat it as a wall: a crater whose rim is all
    // ceiling is closed at the ceiling (seen on 71077345's snow-rimmed cone).
    let ceiling = field.iter().copied().fold(f32::MIN, f32::max);
    let mut walled = false;
    let mut seen = vec![false; gw * gh];
    let mut heap = BinaryHeap::new();
    let s = seed.1 * gw + seed.0;
    heap.push(C(field[s], s));
    seen[s] = true;
    let mut level = field[s];
    let mut count = 0usize;
    while let Some(C(h, i)) = heap.pop() {
        if h < level {
            break; // the pour point: lower than the water already reached
        }
        level = h;
        count += 1;
        if count >= max_cells {
            return None; // not a closed depression within reach
        }
        let (x, y) = (i % gw, i / gw);
        for (dx, dy) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
            let (nx, ny) = (x as i64 + dx, y as i64 + dy);
            if nx < 0 || ny < 0 || nx >= gw as i64 || ny >= gh as i64 {
                return None; // reached the map edge
            }
            let j = ny as usize * gw + nx as usize;
            if !seen[j] {
                seen[j] = true;
                if field[j] >= ceiling {
                    walled = true;
                    continue;
                }
                heap.push(C(field[j], j));
            }
        }
    }
    if count <= 1 {
        return None;
    }
    // Emptied against the ceiling wall: the crater is closed at the ceiling.
    let level = if heap.is_empty() && walled { ceiling - 1e-4 } else { level };
    Some((level as f64, count, seed))
}

/// The closed level-set ring of `field` at `level` that encloses `seed`,
/// traced in a window of `half` cells around it, in km. Polylines from
/// `contour_polylines` are in cell-centre pixel space (`col + 0.5`), which is
/// exactly the colour-paint kilometre convention times km per cell.
fn ring_around(field: &[f32], gw: usize, gh: usize, seed: (usize, usize), level: f64, half: usize, kpc: f64) -> Option<Vec<[f64; 2]>> {
    let (x0, y0) = (seed.0.saturating_sub(half), seed.1.saturating_sub(half));
    let (x1, y1) = ((seed.0 + half).min(gw), (seed.1 + half).min(gh));
    let (w, h) = (x1 - x0, y1 - y0);
    let mut win = vec![0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            win[y * w + x] = field[(y0 + y) * gw + x0 + x];
        }
    }
    let p = [(seed.0 - x0) as f64 + 0.5, (seed.1 - y0) as f64 + 0.5];
    let mut best: Option<Vec<(f64, f64)>> = None;
    for pl in cartalith_spatial::contour_polylines(&win, w, h, level) {
        let closed = pl.len() > 3 && (pl[0].0 - pl[pl.len() - 1].0).abs() < 1e-9 && (pl[0].1 - pl[pl.len() - 1].1).abs() < 1e-9;
        if !closed {
            continue;
        }
        let ring: Vec<(f64, f64)> = pl[..pl.len() - 1].to_vec();
        if inside(&ring, p) && best.as_ref().is_none_or(|b| ring.len() < b.len()) {
            best = Some(ring);
        }
    }
    best.map(|r| r.into_iter().map(|(x, y)| [(x + x0 as f64) * kpc, (y + y0 as f64) * kpc]).collect())
}

/// Even-odd point in polygon (`cartalith_spatial::point_in_ring` takes
/// integer rings, these are fractional).
fn inside(ring: &[(f64, f64)], p: [f64; 2]) -> bool {
    let mut c = false;
    let mut j = ring.len() - 1;
    for i in 0..ring.len() {
        let (a, b) = (ring[i], ring[j]);
        if (a.1 > p[1]) != (b.1 > p[1]) && p[0] < a.0 + (p[1] - a.1) * (b.0 - a.0) / (b.1 - a.1) {
            c = !c;
        }
        j = i;
    }
    c
}

/// Signed shoelace area of a ring.
fn shoelace(r: &[[f64; 2]]) -> f64 {
    let n = r.len();
    (0..n).map(|i| r[i][0] * r[(i + 1) % n][1] - r[(i + 1) % n][0] * r[i][1]).sum::<f64>() / 2.0
}

fn mark(shape: MarkShape, rgb: [u8; 3], opacity: f64, hardness: f64, blend: ColourBlend) -> ColourMark {
    ColourMark { shape, rgb, opacity, hardness, blend, land_only: true, emissive: None }
}

fn appearance() -> TerrainAppearance {
    TerrainAppearance::for_tier(QualityTier::Quality).with_look(render::LOOK_VIBRANT)
}

fn write_png(path: &str, rgb: Vec<u8>) {
    assert!(rgb.iter().any(|&v| v != rgb[0]), "an empty frame: {path}");
    let png = cartalith_assets::raster::encode_png_rgb8(SCREEN_W as u32, SCREEN_H as u32, rgb, cartalith_assets::raster::PngColour::Data).unwrap();
    std::fs::write(path, png).unwrap();
}

/// The frames: per world, the volcano's crater filled red in each of the
/// three modes (and unpainted), the hand-drawn set (a coast tint, a
/// desert-to-grass blend, a mountain tint) before and after, at x1 / x4 /
/// x16 / x64. Prints the fill helper's numbers.
#[test]
#[ignore = "generates two 2048x1311 worlds and renders zoom frames; run explicitly with --release"]
fn colour_paint_frames() {
    let out = std::env::var("CP_OUT").expect("set CP_OUT to an output directory");
    std::fs::create_dir_all(&out).unwrap();
    let (gw, gh) = (2048usize, 1311usize);
    for w in &WORLDS {
        let p = world_params(w, gw, gh);
        let t0 = std::time::Instant::now();
        let ws = cartalith_engine::generate_terrain(&p);
        println!("{}: generated in {:.1}s, edifice model {}", w.tag, t0.elapsed().as_secs_f64(), p.volc.edifice_model);
        let kpc = p.map_width_km / gw as f64;
        let a = appearance();
        let pre = GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, &a, Some(p.map_width_km));
        let base = || RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, 55.0, 5.0, a.clone(), &pre).expect("context");

        // ---- the volcano: the crater fill, three modes --------------------
        // `CP_CLICK=<tag prefix>=x,y[;...]`: the user's click inside a crater
        // (the fill tool is click-seeded, CP-5), replacing the detector's
        // picks for that world. The detector is a stand-in for the click; on
        // 71077345 it rates a river gorge's rimmed pits above the snow-rimmed
        // cone, which a user would simply click.
        let clicks: Vec<(usize, usize)> = std::env::var("CP_CLICK").ok().into_iter().flat_map(|v| v.split(';').map(str::to_string).collect::<Vec<_>>()).filter_map(|e| {
            let (t, xy) = e.split_once('=')?;
            if !w.tag.starts_with(t) {
                return None;
            }
            let (x, y) = xy.split_once(',')?;
            Some((x.parse().ok()?, y.parse().ok()?))
        }).collect();
        let vs: Vec<(usize, usize, f32, f64)> = if clicks.is_empty() {
            rimmed_pits(&ws, gw, gh)
        } else {
            clicks.iter().map(|&(x, y)| {
                let (_, _, rim, d) = rays(&ws.field, gw, x, y);
                (x, y, rim, d)
            }).collect()
        };
        assert!(!vs.is_empty(), "{}: no rimmed crater on land", w.tag);
        for (k, &(vx, vy, rim, rim_d)) in vs.iter().enumerate() {
            // First the closed fill (to the pour point); if the crater is
            // breached that stops within a few cells, and the capped fill --
            // below the rim's median height less 0.01, within 1.3x the
            // median rim distance -- takes over.
            let t1 = std::time::Instant::now();
            let closed_fill = crater_fill(&ws.field, gw, gh, vx, vy, 0, 20_000).filter(|c| c.1 >= 20);
            let closed = closed_fill.and_then(|(level, cells, seed)| ring_around(&ws.field, gw, gh, seed, level - 1e-5, 80, kpc).map(|r| (r, cells)));
            let (ring, what, cells) = match closed {
                Some((r, cells)) => (Some(r), "closed fill to the pour point", cells),
                None => match capped_fill(&ws.field, gw, (vx, vy), rim - 0.01, 1.3 * rim_d, kpc) {
                    Some((r, c)) => (Some(r), "capped fill (breached or untraceable crater)", c),
                    None => (None, "capped fill", 0),
                },
            };
            let fill_ms = t1.elapsed().as_secs_f64() * 1e3;
            let Some(ring) = ring else {
                println!("  crater {k} at ({vx},{vy}): {what} found no ring");
                continue;
            };
            let area_km2 = shoelace(&ring).abs();
            println!(
                "  crater {k} at ({vx},{vy}), floor {:.3}, rim {rim:.3} at {rim_d} cells: {what}, {cells} cells, ring {} points, {area_km2:.1} km2 (~{:.1} km across); fill + ring {fill_ms:.2} ms",
                ws.field[vy * gw + vx],
                ring.len(),
                2.0 * (area_km2 / std::f64::consts::PI).sqrt()
            );
            let (fx, fy) = (ring.iter().map(|p| p[0]).sum::<f64>() / ring.len() as f64 / kpc - 0.5, ring.iter().map(|p| p[1]).sum::<f64>() / ring.len() as f64 / kpc - 0.5);
            let mut arms: Vec<(&str, Option<ColourPaint>)> = vec![("none", None)];
            // The fill's area, water included (`land_only` off: the crater
            // may hold a lake, the owner's case), grown outward by a 1.5 km
            // hard stroke along its ring so the inner walls up to the rim
            // are painted too -- the fill helper's "grow" option (CP-5).
            let mut closed = ring.clone();
            closed.push(ring[0]);
            for blend in ColourBlend::ALL {
                let mut fillm = mark(MarkShape::Area { ring_km: ring.clone(), edge_km: 0.0 }, [200, 30, 18], 1.0, 1.0, blend);
                fillm.land_only = false;
                let mut grow = mark(MarkShape::Line { points_km: closed.clone(), radius_km: 1.5 }, [200, 30, 18], 1.0, 1.0, blend);
                grow.land_only = false;
                let doc = ColourPaintDoc { version: DOC_VERSION, visible: true, marks: vec![fillm, grow] };
                arms.push((blend.key(), Some(ColourPaint::new(doc, 2.0 * kpc).unwrap())));
            }
            for (name, layer) in &arms {
                let ctx = base().with_colour_paint(layer.as_ref());
                let tf = TileFields::new(&ctx, None);
                write_png(&format!("{out}/{}_volcano{k}_x1_{name}.png", w.tag), base_frame(&ctx, gw as f64 / 2.0, gh as f64 / 2.0, SCREEN_W as f64 / gw as f64));
                for &(rung, ppc) in &RUNGS {
                    write_png(&format!("{out}/{}_volcano{k}_{rung}_{name}.png", w.tag), frame(&ctx, &tf, fx, fy, ppc, w.seed));
                }
            }
        }

        // ---- the hand-drawn set -------------------------------------------
        // Targets by INPUT: the highest land cell (mountain), the land cell
        // beside the sea nearest the centre (coast), the driest warm land
        // cell at least 20 cells from the sea (desert).
        let sea = ws.sea_level as f32;
        let f = &ws.field;
        let (mut mtn, mut coast, mut dry) = ((f32::MIN, 0, 0), (f64::MAX, 0, 0), (f32::MAX, 0, 0));
        for y in (100..gh - 100).step_by(3) {
            for x in (100..gw - 100).step_by(3) {
                let i = y * gw + x;
                if f[i] < sea {
                    continue;
                }
                if f[i] > mtn.0 {
                    mtn = (f[i], x, y);
                }
                if f[i - 1] < sea || f[i + 1] < sea || f[i - gw] < sea || f[i + gw] < sea {
                    let d = (x as f64 - gw as f64 / 2.0).hypot(y as f64 - gh as f64 / 2.0);
                    if d < coast.0 {
                        coast = (d, x, y);
                    }
                }
                let inland = (0..8).all(|k| {
                    let a = k as f64 * std::f64::consts::FRAC_PI_4;
                    let (qx, qy) = ((x as f64 + 20.0 * a.cos()) as usize, (y as f64 + 20.0 * a.sin()) as usize);
                    f[qy * gw + qx] >= sea
                });
                if inland && ws.temperature[i] > 12.0 && ws.rainfall[i] < dry.0 {
                    dry = (ws.rainfall[i], x, y);
                }
            }
        }
        let km = |x: usize, y: usize| [(x as f64 + 0.5) * kpc, (y as f64 + 0.5) * kpc];
        // The coast tint follows the coast: the sea-level contour through the
        // coast pick, 60 points of it either side.
        let coast_line: Vec<[f64; 2]> = {
            let (x0, y0) = (coast.1.saturating_sub(60), coast.2.saturating_sub(60));
            let (w2, h2) = (120usize, 120usize);
            let mut win = vec![0f32; w2 * h2];
            for y in 0..h2 {
                for x in 0..w2 {
                    win[y * w2 + x] = f[(y0 + y) * gw + x0 + x];
                }
            }
            let pls = cartalith_spatial::contour_polylines(&win, w2, h2, sea as f64);
            let c = ((coast.1 - x0) as f64 + 0.5, (coast.2 - y0) as f64 + 0.5);
            let pl = pls.into_iter().max_by_key(|pl| pl.len()).expect("a coastline near the coast pick");
            let near = pl.iter().enumerate().min_by(|a, b| ((a.1 .0 - c.0).hypot(a.1 .1 - c.1)).total_cmp(&(b.1 .0 - c.0).hypot(b.1 .1 - c.1))).unwrap().0;
            let (a, b) = (near.saturating_sub(60), (near + 60).min(pl.len()));
            pl[a..b].iter().step_by(2).map(|&(x, y)| [(x + x0 as f64) * kpc, (y + y0 as f64) * kpc]).collect()
        };
        let set = vec![
            ("coast", (coast.1, coast.2), mark(MarkShape::Line { points_km: coast_line, radius_km: 2.5 }, [40, 170, 165], 0.7, 0.3, ColourBlend::Colour)),
            ("desert", (dry.1, dry.2), mark(MarkShape::Line { points_km: vec![km(dry.1 - 40, dry.2 - 10), km(dry.1, dry.2 + 5), km(dry.1 + 40, dry.2 - 5)], radius_km: 12.0 }, [86, 140, 58], 0.75, 0.0, ColourBlend::Colour)),
            ("mountain", (mtn.1, mtn.2), mark(MarkShape::Line { points_km: vec![km(mtn.1 - 12, mtn.2 - 6), km(mtn.1 + 12, mtn.2 + 6)], radius_km: 5.0 }, [225, 160, 110], 1.0, 0.5, ColourBlend::Multiply)),
        ];
        println!("  targets: coast {:?} desert {:?} (rain {:.3}) mountain {:?}", (coast.1, coast.2), (dry.1, dry.2), dry.0, (mtn.1, mtn.2));
        let all = ColourPaint::new(ColourPaintDoc { version: DOC_VERSION, visible: true, marks: set.iter().map(|s| s.2.clone()).collect() }, 2.0 * kpc).unwrap();
        for (name, layer) in [("before", None), ("after", Some(&all))] {
            let ctx = base().with_colour_paint(layer);
            let tf = TileFields::new(&ctx, None);
            write_png(&format!("{out}/{}_set_x1_{name}.png", w.tag), base_frame(&ctx, gw as f64 / 2.0, gh as f64 / 2.0, SCREEN_W as f64 / gw as f64));
            for (target, (tx, ty), _) in &set {
                for &(rung, ppc) in &RUNGS {
                    write_png(&format!("{out}/{}_{target}_{rung}_{name}.png", w.tag), frame(&ctx, &tf, *tx as f64, *ty as f64, ppc, w.seed));
                }
            }
        }
    }
}

/// `n` hand-stroke-sized marks (12 points, about 6-12 km long, 0.5..3 km
/// in radius, the three modes in turn) scattered uniformly over the km box
/// `[x0, y0, x1, y1]`, from a fixed xorshift seed.
fn scatter(n: usize, b: [f64; 4]) -> Vec<ColourMark> {
    let mut rng = 0x2545_F491_4F6C_DD1Du64;
    let mut rnd = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        (rng >> 11) as f64 / (1u64 << 53) as f64
    };
    (0..n)
        .map(|k| {
            let (sx, sy) = (b[0] + rnd() * (b[2] - b[0]), b[1] + rnd() * (b[3] - b[1]));
            let (dx, dy) = (rnd() - 0.5, rnd() - 0.5);
            let pts: Vec<[f64; 2]> = (0..12).map(|j| [sx + j as f64 * dx * 1.6, sy + j as f64 * dy * 1.6 + (j as f64 * 0.9).sin() * 0.4]).collect();
            mark(MarkShape::Line { points_km: pts, radius_km: 0.5 + rnd() * 2.5 }, [200, 40, 30], 0.8, 0.5, ColourBlend::ALL[k % 3])
        })
        .collect()
}

/// Mean number of marks covering a point of the box, by sampling: the
/// overdraw the cost scales with.
fn overdraw(layer: &ColourPaint, b: [f64; 4]) -> f64 {
    // One single-mark layer per mark, built once; a mark "covers" a point
    // when it moves a mid grey there.
    let singles: Vec<ColourPaint> = layer.doc().marks.iter().map(|m| ColourPaint::new(ColourPaintDoc { version: DOC_VERSION, visible: true, marks: vec![m.clone()] }, 0.0).unwrap()).collect();
    let mut hits = 0usize;
    let n = 50usize;
    for j in 0..n {
        for i in 0..n {
            let q = cartalith_spatial::colour_paint::PaintQuery { x_km: b[0] + (i as f64 + 0.5) / n as f64 * (b[2] - b[0]), y_km: b[1] + (j as f64 + 0.5) / n as f64 * (b[3] - b[1]), px_km: 0.0, water: false };
            hits += singles.iter().filter(|one| one.apply([100.0, 101.0, 102.0], q) != [100.0, 101.0, 102.0]).count();
        }
    }
    hits as f64 / (n * n) as f64
}

/// The cost bar (`COLOUR_PAINT_SCOPE.md` bar 7): tiles and the base raster
/// with no layer and with 1 000 hand-sized strokes in view, alternated so
/// drift hits both arms. Three cases: (a) the bar's case, 1 000 strokes over
/// the x4 view (a 6 x 4 block of level-4 tiles, about 300 x 200 km); (b) the
/// same strokes seen through 24 level-6 tiles inside that block; (c) the base
/// raster (`cell_color` over the whole 2048 x 1311 grid) with 1 000 strokes
/// over the whole map; and (d) a stress case, the 1 000 strokes packed into
/// one 6 x 4 level-6 block, reported with its overdraw because the cost
/// scales with it. Prints median and min..max. Run **alone**, `--release`.
#[test]
#[ignore = "generates a 2048x1311 world and times tiles; run explicitly, alone, with --release"]
fn colour_paint_tile_cost() {
    let (gw, gh) = (2048usize, 1311usize);
    let w = &WORLDS[0];
    let p = world_params(w, gw, gh);
    let ws = cartalith_engine::generate_terrain(&p);
    let a = appearance();
    let kpc = p.map_width_km / gw as f64;
    let pre = GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, &a, Some(p.map_width_km));
    let off = RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, 55.0, 5.0, a.clone(), &pre).unwrap();
    let tf_off = TileFields::new(&off, None);
    let block = |z: i32, c0: u32, r0: u32| -> [f64; 4] {
        let b0 = pyramid_tile_bounds(gw, gh, z, c0, r0);
        let b1 = pyramid_tile_bounds(gw, gh, z, c0 + 5, r0 + 3);
        [b0.x * kpc, b0.y * kpc, (b1.x + b1.w) * kpc, (b1.y + b1.h) * kpc]
    };
    let n4 = pyramid_dims(4).cols;
    let (c4, r4) = (n4 / 2 - 3, n4 / 2 - 2);
    let view = block(4, c4, r4);
    let n6 = pyramid_dims(6).cols;
    // The level-6 block at the centre of the level-4 block.
    let (c6, r6) = (c4 * 4 + 9, r4 * 4 + 6);
    let dense = block(6, c6, r6);
    // `e`/`f`: the lava bar (the owner's 2026-10-18 brief) -- the same 1 000
    // strokes with every tenth an emissive Lava mark (heat 0.75, cooling
    // along, a 1.5 km halo).
    let cases: [(&str, i32, u32, u32, [f64; 4], bool); 5] = [
        ("a: x4 view, level 4", 4, c4, r4, view, false),
        ("b: same strokes, level 6", 6, c6, r6, view, false),
        ("d: stress, level 6", 6, c6, r6, dense, false),
        ("e: 1000 strokes, 100 of them lava, x4 view, level 4", 4, c4, r4, view, true),
        ("f: the same, level 6", 6, c6, r6, view, true),
    ];
    for (name, z, c0, r0, area, with_lava) in cases {
        let mut marks = scatter(1000, area);
        if with_lava {
            for m in marks.iter_mut().step_by(10) {
                *m = lava_mark(m.shape.clone(), 0.75, 0.6, cartalith_spatial::colour_paint::GlowRamp::Fire);
            }
        }
        let layer = ColourPaint::new(ColourPaintDoc { version: DOC_VERSION, visible: true, marks }, 2.0 * kpc).unwrap();
        let tile_box = block(z, c0, r0);
        let od = overdraw(&layer, tile_box);
        let on = RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, 55.0, 5.0, a.clone(), &pre).unwrap().with_colour_paint(Some(&layer));
        let tf_on = TileFields::new(&on, None);
        let mut t = [Vec::new(), Vec::new()];
        let mut moved = 0usize;
        for k in 0..24u32 {
            let (col, row) = ((c0 + k % 6).min(n6 - 1), r0 + k / 6);
            let t1 = std::time::Instant::now();
            let a0 = tile(&off, &tf_off, z, col, row, w.seed);
            t[0].push(t1.elapsed().as_secs_f64() * 1e3);
            let t2 = std::time::Instant::now();
            let a1 = tile(&on, &tf_on, z, col, row, w.seed);
            t[1].push(t2.elapsed().as_secs_f64() * 1e3);
            moved += a0.0.iter().zip(&a1.0).filter(|(x, y)| x != y).count();
        }
        assert!(moved > 0, "premise: the strokes are in view");
        for v in t.iter_mut() {
            v.sort_by(f64::total_cmp);
        }
        println!("{name}: overdraw {od:.2} marks per point; off {:.2} ms ({:.2}..{:.2}), on {:.2} ms ({:.2}..{:.2}); ratio of medians {:.3}; index {:?}", t[0][12], t[0][0], t[0][23], t[1][12], t[1][0], t[1][23], t[1][12] / t[0][12], layer.index_stats());
    }
    // (c) the base raster.
    let whole = [0.0, 0.0, p.map_width_km, gh as f64 * kpc];
    let layer = ColourPaint::new(ColourPaintDoc { version: DOC_VERSION, visible: true, marks: scatter(1000, whole) }, 2.0 * kpc).unwrap();
    let on = RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, 55.0, 5.0, a.clone(), &pre).unwrap().with_colour_paint(Some(&layer));
    let raster = |c: &RenderCtx| {
        use rayon::prelude::*;
        let mut b = vec![0u8; gw * gh * 3];
        b.par_chunks_mut(gw * 3).enumerate().for_each(|(y, row)| {
            for x in 0..gw {
                let (r, g, bb) = render::cell_color(c, x, y);
                row[x * 3] = (r * 255.0) as u8;
                row[x * 3 + 1] = (g * 255.0) as u8;
                row[x * 3 + 2] = (bb * 255.0) as u8;
            }
        });
        b
    };
    let mut t = [Vec::new(), Vec::new()];
    for _ in 0..7 {
        let t1 = std::time::Instant::now();
        let _ = raster(&off);
        t[0].push(t1.elapsed().as_secs_f64() * 1e3);
        let t2 = std::time::Instant::now();
        let _ = raster(&on);
        t[1].push(t2.elapsed().as_secs_f64() * 1e3);
    }
    for v in t.iter_mut() {
        v.sort_by(f64::total_cmp);
    }
    println!("c: base raster, 1000 strokes over the map: off {:.1} ms ({:.1}..{:.1}), on {:.1} ms ({:.1}..{:.1}); ratio of medians {:.3}", t[0][3], t[0][0], t[0][6], t[1][3], t[1][0], t[1][6], t[1][3] / t[0][3]);
}

/// The save-size figure (`COLOUR_PAINT_SCOPE.md` §9): 5 000 strokes of 40
/// points each (the cap on marks, a typical hand stroke after
/// simplification), serialised as `appearance_paint.json` would hold them.
/// Prints bytes and the build time of the layer. Not a bar by itself.
#[test]
#[ignore = "a measurement; run explicitly"]
fn colour_paint_save_size() {
    let marks: Vec<ColourMark> = (0..5000)
        .map(|k| {
            let pts: Vec<[f64; 2]> = (0..40).map(|j| [(k % 100) as f64 * 8.0 + j as f64 * 0.37, (k / 100) as f64 * 16.0 + (j as f64 * 0.7).sin() * 3.0]).collect();
            mark(MarkShape::Line { points_km: pts, radius_km: 1.5 }, [200, 40, 30], 0.8, 0.5, ColourBlend::Colour)
        })
        .collect();
    let doc = ColourPaintDoc { version: DOC_VERSION, visible: true, marks };
    let s = serde_json::to_string(&doc).unwrap();
    // The precision a saved coordinate needs: a metre (3 decimals of km).
    let rounded: ColourPaintDoc = {
        let mut d = doc.clone();
        for m in &mut d.marks {
            if let MarkShape::Line { points_km, .. } = &mut m.shape {
                for p in points_km {
                    p[0] = (p[0] * 1000.0).round() / 1000.0;
                    p[1] = (p[1] * 1000.0).round() / 1000.0;
                }
            }
        }
        d
    };
    let r = serde_json::to_string(&rounded).unwrap();
    let t0 = std::time::Instant::now();
    let layer = ColourPaint::new(doc, 0.8).unwrap();
    println!("5000 strokes x 40 points: JSON {} bytes (full precision), {} bytes (metre-rounded); layer build {:.1} ms, index {:?}", s.len(), r.len(), t0.elapsed().as_secs_f64() * 1e3, layer.index_stats());
}

/// Debug: the height profile through a point (`CP_AT=x,y`), for reading a
/// crater's shape off the grid. Prints only.
#[test]
#[ignore = "a probe; run explicitly"]
fn colour_paint_profile() {
    let at: Vec<usize> = std::env::var("CP_AT").expect("CP_AT=x,y").split(',').map(|v| v.parse().unwrap()).collect();
    let (gw, gh) = (2048usize, 1311usize);
    let ws = cartalith_engine::generate_terrain(&world_params(&WORLDS[0], gw, gh));
    let (cx, cy) = (at[0], at[1]);
    for dy in (-24i64..=24).step_by(3) {
        let row: Vec<String> = (-24i64..=24).step_by(3).map(|dx| format!("{:4.0}", ws.field[(cy as i64 + dy) as usize * gw + (cx as i64 + dx) as usize] * 1000.0)).collect();
        println!("{:+4} {}", dy, row.join(""));
    }
    println!("fill from the lowest cell within 12: {:?}", crater_fill(&ws.field, gw, gh, cx, cy, 12, 20_000));
}

/// A lava material with the given heat (the frames' and the shell's Lava
/// preset otherwise): near-black blue-grey crust, medium plates, a 1.5 km halo.
fn lava_mark(shape: MarkShape, heat: f64, heat_along: f64, ramp: cartalith_spatial::colour_paint::GlowRamp) -> ColourMark {
    let mut m = mark(shape, LAVA_CRUST, 1.0, 1.0, ColourBlend::Replace);
    m.emissive = Some(cartalith_spatial::colour_paint::EmissiveMaterial { ramp, heat, crack_density: 0.5, glow_radius_km: 1.5, heat_along });
    m
}

/// The Lava preset's crust colour: near-black with a slight blue-grey cast,
/// the colour of chilled basalt glass. Labelled judgement.
const LAVA_CRUST: [u8; 3] = [34, 33, 38];

/// A flow's path down a volcano's flank: from `(cx, cy)` along the ray to
/// the lowest point of the rim (the breach a flow would take), then steepest
/// descent on the grid for up to `steps` cells, thinned and smoothed
/// (Chaikin, twice), in km.
fn flank_flow(f: &[f32], gw: usize, gh: usize, cx: usize, cy: usize, steps: usize, kpc: f64) -> Vec<[f64; 2]> {
    let mut best = (f32::MAX, cx, cy);
    for k in 0..32 {
        let a = k as f64 * std::f64::consts::TAU / 32.0;
        let mut top = (f32::MIN, cx, cy);
        for r in 1..=16 {
            let (x, y) = ((cx as f64 + r as f64 * a.cos()).round() as usize, (cy as f64 + r as f64 * a.sin()).round() as usize);
            if f[y * gw + x] > top.0 {
                top = (f[y * gw + x], x, y);
            }
        }
        if top.0 < best.0 {
            best = top;
        }
    }
    let (mut x, mut y) = (best.1, best.2);
    let mut path = vec![(cx, cy), (x, y)];
    for _ in 0..steps {
        let mut nb = (f[y * gw + x], x, y);
        for (dx, dy) in [(-1i64, -1i64), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
            let (nx, ny) = ((x as i64 + dx) as usize, (y as i64 + dy) as usize);
            if nx < 2 || ny < 2 || nx >= gw - 2 || ny >= gh - 2 {
                continue;
            }
            // Outward only: a flow leaves the crater and runs down the flank.
            let dist = |x: usize, y: usize| (x as f64 - cx as f64).hypot(y as f64 - cy as f64);
            if dist(nx, ny) <= dist(x, y) {
                continue;
            }
            if f[ny * gw + nx] <= nb.0 || (nb.1, nb.2) == (x, y) && f[ny * gw + nx] <= f[y * gw + x] + 0.002 {
                nb = (f[ny * gw + nx], nx, ny);
            }
        }
        if (nb.1, nb.2) == (x, y) {
            break;
        }
        x = nb.1;
        y = nb.2;
        path.push((x, y));
    }
    let mut pts: Vec<[f64; 2]> = path.iter().step_by(2).map(|&(x, y)| [(x as f64 + 0.5) * kpc, (y as f64 + 0.5) * kpc]).collect();
    for _ in 0..2 {
        let mut s = vec![pts[0]];
        for w in pts.windows(2) {
            s.push([0.75 * w[0][0] + 0.25 * w[1][0], 0.75 * w[0][1] + 0.25 * w[1][1]]);
            s.push([0.25 * w[0][0] + 0.75 * w[1][0], 0.25 * w[0][1] + 0.75 * w[1][1]]);
        }
        s.push(pts[pts.len() - 1]);
        pts = s;
    }
    pts
}

/// The lava frames (the owner's 2026-10-18 feedback on CP-1): per world, the
/// crater filled with lava and a flow down the flank whose heat falls along
/// it, at x1 / x4 / x16 / x64, plus a heat sweep and the glow presets at x16
/// and x64. `CP_CLICK` as in [`colour_paint_frames`]. Writes PNGs to
/// `CP_OUT`; the judgement is made by reading them.
#[test]
#[ignore = "generates two 2048x1311 worlds and renders zoom frames; run explicitly with --release"]
fn colour_paint_lava_frames() {
    use cartalith_spatial::colour_paint::GlowRamp;
    let out = std::env::var("CP_OUT").expect("set CP_OUT to an output directory");
    std::fs::create_dir_all(&out).unwrap();
    let only = std::env::var("CP_ONLY").ok();
    let (gw, gh) = (2048usize, 1311usize);
    for w in &WORLDS {
        if only.as_deref().is_some_and(|o| !w.tag.starts_with(o)) {
            continue;
        }
        let p = world_params(w, gw, gh);
        let ws = cartalith_engine::generate_terrain(&p);
        let kpc = p.map_width_km / gw as f64;
        let a = appearance();
        let pre = GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, &a, Some(p.map_width_km));
        let base = || RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, 55.0, 5.0, a.clone(), &pre).expect("context");
        let clicks: Vec<(usize, usize)> = std::env::var("CP_CLICK").ok().into_iter().flat_map(|v| v.split(';').map(str::to_string).collect::<Vec<_>>()).filter_map(|e| {
            let (t, xy) = e.split_once('=')?;
            if !w.tag.starts_with(t) {
                return None;
            }
            let (x, y) = xy.split_once(',')?;
            Some((x.parse().ok()?, y.parse().ok()?))
        }).collect();
        let (vx, vy, rim, rim_d) = match clicks.first() {
            Some(&(x, y)) => {
                let (_, _, rim, d) = rays(&ws.field, gw, x, y);
                (x, y, rim, d)
            }
            None => rimmed_pits(&ws, gw, gh)[0],
        };
        let ring = match crater_fill(&ws.field, gw, gh, vx, vy, 0, 20_000).filter(|c| c.1 >= 20).and_then(|(level, _, seed)| ring_around(&ws.field, gw, gh, seed, level - 1e-5, 80, kpc)) {
            Some(r) => r,
            None => capped_fill(&ws.field, gw, (vx, vy), rim - 0.01, 1.3 * rim_d, kpc).expect("a crater ring").0,
        };
        let mut closed = ring.clone();
        closed.push(ring[0]);
        let (fx, fy) = (ring.iter().map(|p| p[0]).sum::<f64>() / ring.len() as f64 / kpc - 0.5, ring.iter().map(|p| p[1]).sum::<f64>() / ring.len() as f64 / kpc - 0.5);
        let flow = flank_flow(&ws.field, gw, gh, vx, vy, 70, kpc);
        let flen: f64 = flow.windows(2).map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1])).sum();
        println!("{}: crater at ({vx},{vy}), ring {} pts, area {:.1} km2; flow {} pts, {flen:.1} km", w.tag, ring.len(), shoelace(&ring).abs(), flow.len());
        let lava = |heat: f64, ramp: GlowRamp| -> Vec<ColourMark> {
            let mut grow = lava_mark(MarkShape::Line { points_km: closed.clone(), radius_km: 1.0 }, heat * 0.55, 0.0, ramp);
            grow.land_only = false;
            let mut fill = lava_mark(MarkShape::Area { ring_km: ring.clone(), edge_km: 0.0 }, heat, 0.0, ramp);
            fill.land_only = false;
            let flow_m = lava_mark(MarkShape::Line { points_km: flow.clone(), radius_km: 0.8 }, heat, 0.85, ramp);
            vec![grow, fill, flow_m]
        };
        // The flow's first third, the second view.
        let mid = flow[flow.len() / 3];
        let (mx, my) = (mid[0] / kpc - 0.5, mid[1] / kpc - 0.5);
        let arms: Vec<(&str, Option<ColourPaint>)> = vec![("none", None), ("lava", Some(ColourPaint::new(ColourPaintDoc { version: DOC_VERSION, visible: true, marks: lava(0.75, GlowRamp::Fire) }, 2.0 * kpc).unwrap()))];
        for (name, layer) in &arms {
            let ctx = base().with_colour_paint(layer.as_ref());
            let tf = TileFields::new(&ctx, None);
            write_png(&format!("{out}/{}_lava_x1_{name}.png", w.tag), base_frame(&ctx, gw as f64 / 2.0, gh as f64 / 2.0, SCREEN_W as f64 / gw as f64));
            for &(rung, ppc) in &RUNGS {
                write_png(&format!("{out}/{}_lava_crater_{rung}_{name}.png", w.tag), frame(&ctx, &tf, fx, fy, ppc, w.seed));
                write_png(&format!("{out}/{}_lava_flow_{rung}_{name}.png", w.tag), frame(&ctx, &tf, mx, my, ppc, w.seed));
            }
        }
        // The heat sweep and the glow presets, at x16 and x64 on the crater.
        let mut sweep: Vec<(String, Vec<ColourMark>)> = [0.25, 0.5, 0.95].iter().map(|&h| (format!("heat{:02}", (h * 100.0) as i32), lava(h, GlowRamp::Fire))).collect();
        sweep.push(("ice".into(), lava(0.75, GlowRamp::Ice)));
        sweep.push(("toxic".into(), lava(0.75, GlowRamp::Toxic)));
        for (name, marks) in sweep {
            let layer = ColourPaint::new(ColourPaintDoc { version: DOC_VERSION, visible: true, marks }, 2.0 * kpc).unwrap();
            let ctx = base().with_colour_paint(Some(&layer));
            let tf = TileFields::new(&ctx, None);
            for &(rung, ppc) in &RUNGS[1..] {
                write_png(&format!("{out}/{}_lava_crater_{rung}_{name}.png", w.tag), frame(&ctx, &tf, fx, fy, ppc, w.seed));
            }
        }
    }
}
