//! **GF-6 §4.10's measurement harness** (`GEOLOGY_FIRST_SCOPE.md` §4.10, §7
//! GF-6): the zoom ladder of `zoom_detail_ladder.rs`, over worlds generated
//! with the landform processes reading rock (`geology_processes` on, the only
//! worlds `lib.rs` attaches a `RockDetail` to), each frame rendered twice --
//! without the rock detail (today's tile) and with it (GF-6 §4.10's styling).
//!
//! Same tile path as `zoom_detail_ladder.rs` (`pyramid_tile_padded` with
//! `render::tile_amplify_opts`, coloured by `render_biome_tile_rgba_water`,
//! the screen composited at the level `pyramid_level_for_zoom` picks with
//! LOD-D3's morph), the app's own generation parameters (`params::defaults()`)
//! with `geology_processes` switched on, and the app's default look. Terrain
//! only: no river layer, no lithology tint, no RV-3 valley field.
//!
//! Targets are picked by **inputs** (the exposed rock, the cap-edge flag, the
//! coarse slope, the distance to the sea), never by the value under test
//! (`MISTAKES.md`, "Measure an effect inside a subset of pixels"):
//! `granite` = the steepest granite land cell at least 20 cells from the sea;
//! `capedge` = the steepest bedded (limestone, sandstone, tuff) cap-edge cell;
//! `shale` = the steepest fissile cap-edge cell; `weak` = the steepest
//! unconsolidated or schist land cell. Each 140 cells from every edge, on a
//! 4-cell stride, as the zoom ladder keeps its picks.
//!
//! It writes `<world>_<target>_<rung>_<arm>.png` into `GF6L_OUT` and prints
//! each world's rock shares; statistics are computed from the PNGs offline.
//! `GF6L_SEEDS` overrides the seeds (comma list).
//!
//! ```text
//! GF6L_OUT=<dir> cargo test -p cartalith-godot --test gf6_rock_ladder -- --ignored --nocapture
//! ```
//!
//! Protects: nothing by itself (a measurement harness, `#[ignore]`d); it
//! asserts only that each world has a rock detail and each frame is non-empty.

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
use cartalith_terrain::geology::{Jointing, Rock};
use render::{GridPrecompute, QualityTier, RenderCtx, RockDetail, TerrainAppearance, TileBounds, TileFields, ROCK_CAP_EDGE_BIT};

/// `lod_bridge::TILE_PX`, restated for the reason `zoom_detail_ladder.rs`
/// gives (`lod_bridge.rs` cannot be `#[path]`-included).
const TILE_PX: usize = 256;
/// `lod_bridge::MAX_LEVEL`, same reason.
const MAX_LEVEL: i32 = 10;
/// The zoom ladder's capture size and rungs (`zoom_detail_ladder.rs`).
const SCREEN_W: usize = 950;
const SCREEN_H: usize = 858;
const RUNGS: [(&str, f64); 3] = [("x4", 1.86), ("x16", 7.42), ("x64", 29.7)];

/// The steepest land cell satisfying `keep`, 140 cells from every edge and
/// (`inland`) at least 20 cells from the sea, on a 4-cell stride; or `None`.
fn pick(ws: &cartalith_engine::WorldState, gw: usize, gh: usize, rd: &RockDetail, inland: bool, keep: impl Fn(u8) -> bool) -> Option<(f64, f64)> {
    let sea = ws.sea_level as f32;
    let f = &ws.field;
    let m = 140usize;
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
    let mut best: Option<(f32, usize, usize)> = None;
    for y in (m..gh - m).step_by(4) {
        for x in (m..gw - m).step_by(4) {
            let i = y * gw + x;
            if f[i] < sea || !keep(rd.cells[i]) || (inland && near_sea(x, y)) {
                continue;
            }
            let s = (f[i + 1] - f[i - 1]).hypot(f[i + gw] - f[i - gw]);
            if best.is_none_or(|b| s > b.0) {
                best = Some((s, x, y));
            }
        }
    }
    best.map(|b| (b.1 as f64, b.2 as f64))
}

/// One synthesized tile, exactly `zoom_detail_ladder.rs`' `tile`.
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

/// `zoom_detail_ladder.rs`' `frame`: one screen frame with LOD-D3's morph.
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

/// `zoom_detail_ladder.rs`' `level_frame`.
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

fn jointing(cell: u8) -> Option<Jointing> {
    Rock::from_u8(cell & !ROCK_CAP_EDGE_BIT).map(|r| r.props().jointing)
}

#[test]
#[ignore = "generates three 2048x1311 processes-on worlds and renders a zoom ladder; run explicitly"]
fn gf6_rock_ladder() {
    let out = std::env::var("GF6L_OUT").expect("set GF6L_OUT to an output directory");
    std::fs::create_dir_all(&out).unwrap();
    let (gw, gh) = (2048usize, 1311usize);
    let seeds: Vec<i32> = std::env::var("GF6L_SEEDS").unwrap_or_else(|_| "483920,24601,71077345".into()).split(',').map(|s| s.parse().unwrap()).collect();
    for seed in seeds {
        let mut p = params::defaults();
        p.gw = gw;
        p.gh = gh;
        p.tect.seed = seed;
        p.map_width_km = 800.0;
        p.sea_level = 0.42;
        p.geology_processes = true;
        assert!(p.geology_model, "the app's defaults must build a rock column");
        let t0 = std::time::Instant::now();
        let ws = cartalith_engine::generate_terrain(&p);
        println!("s{seed}: generated in {:.1}s", t0.elapsed().as_secs_f64());
        let col = ws.geology.column().expect("a processes-on world has a column");
        let rd = render::build_rock_detail(col, &ws.field, ws.sea_level as f64, p.peak_m, gw, gh).expect("a rock detail");
        // The world's rock shares over land, and the styled shares.
        let sea = ws.sea_level as f32;
        let land: Vec<usize> = (0..gw * gh).filter(|&i| ws.field[i] >= sea).collect();
        let mut by_rock = [0usize; 11];
        let (mut edge, mut ridged, mut terraced) = (0usize, 0usize, 0usize);
        for &i in &land {
            let c = rd.cells[i];
            by_rock[(c & !ROCK_CAP_EDGE_BIT) as usize] += 1;
            let st = render::rock_style(c).unwrap();
            edge += (c & ROCK_CAP_EDGE_BIT != 0) as usize;
            ridged += (st.ridge > 0.0) as usize;
            terraced += (st.terrace > 0.0) as usize;
        }
        let pc = |k: usize| 100.0 * k as f64 / land.len() as f64;
        let shares: Vec<String> = Rock::ALL.iter().map(|r| format!("{} {:.1}%", r.props().name, pc(by_rock[*r as usize]))).collect();
        println!("s{seed}: land {} cells; {}", land.len(), shares.join(", "));
        println!("s{seed}: cap-edge {:.2}%, ridged {:.1}%, terraced {:.2}% of land", pc(edge), pc(ridged), pc(terraced));
        // The grid-scale slope, in degrees, over granite land (the ridged
        // style's population), at the percentiles the slope gate is read on.
        let cell_m = p.map_width_km * 1000.0 / gw as f64;
        let mpu = rd.m_per_unit;
        let mut deg: Vec<f64> = land
            .iter()
            .filter(|&&i| rd.cells[i] & !ROCK_CAP_EDGE_BIT == Rock::Granite as u8)
            .filter(|&&i| i % gw > 0 && i % gw < gw - 1 && i / gw > 0 && i / gw < gh - 1)
            .map(|&i| {
                let f = &ws.field;
                let g = (((f[i + 1] - f[i - 1]) as f64 * 0.5).hypot((f[i + gw] - f[i - gw]) as f64 * 0.5)) * mpu / cell_m;
                g.atan().to_degrees()
            })
            .collect();
        deg.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let q = |p: f64| deg[((deg.len() - 1) as f64 * p) as usize];
        println!("s{seed}: granite grid slope deg p50 {:.2} p75 {:.2} p90 {:.2} p97 {:.2} p99 {:.2}", q(0.5), q(0.75), q(0.9), q(0.97), q(0.99));
        let targets: Vec<(&str, Option<(f64, f64)>)> = vec![
            ("granite", pick(&ws, gw, gh, &rd, true, |c| c & !ROCK_CAP_EDGE_BIT == Rock::Granite as u8)),
            ("capedge", pick(&ws, gw, gh, &rd, false, |c| c & ROCK_CAP_EDGE_BIT != 0 && jointing(c) == Some(Jointing::Bedded))),
            ("shale", pick(&ws, gw, gh, &rd, false, |c| c & ROCK_CAP_EDGE_BIT != 0 && jointing(c) == Some(Jointing::Fissile))),
            ("weak", pick(&ws, gw, gh, &rd, true, |c| matches!(Rock::from_u8(c & !ROCK_CAP_EDGE_BIT), Some(Rock::Unconsolidated) | Some(Rock::Schist)))),
        ];
        println!("TARGETS s{seed} {:?}", targets);
        let a = TerrainAppearance::for_tier(QualityTier::Quality).with_look(render::LOOK_VIBRANT);
        let pre = GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, &a, Some(p.map_width_km));
        let base = RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, 55.0, 5.0, a.clone(), &pre).expect("context");
        let styled = RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, p.world, 55.0, 5.0, a, &pre).expect("context").with_rock_detail(&rd);
        let tf = TileFields::new(&base, None);
        for (name, at) in &targets {
            let Some((gx, gy)) = *at else {
                println!("  {name}: no cell of this kind on s{seed}");
                continue;
            };
            let i = gy as usize * gw + gx as usize;
            println!("  {name} ({gx},{gy}): rock {}, edge {}", Rock::from_u8(rd.cells[i] & !ROCK_CAP_EDGE_BIT).unwrap().props().name, rd.cells[i] & ROCK_CAP_EDGE_BIT != 0);
            for &(rung, ppc) in &RUNGS {
                for (arm, ctx) in [("off", &base), ("on", &styled)] {
                    let rgb = frame(ctx, &tf, gx, gy, ppc, seed);
                    assert!(rgb.iter().any(|&v| v != rgb[0]), "an empty frame");
                    let png = cartalith_assets::raster::encode_png_rgb8(SCREEN_W as u32, SCREEN_H as u32, rgb, cartalith_assets::raster::PngColour::Data).unwrap();
                    std::fs::write(format!("{out}/s{seed}_{name}_{rung}_{arm}.png"), png).unwrap();
                }
            }
        }
    }
}
