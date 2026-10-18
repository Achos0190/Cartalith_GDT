//! **GF-6 §4.10: rock-styled deep-zoom detail** (`GEOLOGY_FIRST_SCOPE.md`
//! §4.10 and §7 GF-6; owner Ruling BH).
//!
//! What these tests protect, and why each is shaped the way it is:
//! * the style table -- every rock's `α`, ridged / terraced / fbm, and the
//!   fissile step -- against literals worked from §2.3's `s` and jointing,
//!   never against the constants themselves (`MISTAKES.md`);
//! * the terrace's own law (benches, risers, continuity, monotonicity) and
//!   that the styled height is the unstyled one exactly when every weight is
//!   off, and that terraces arrive with zoom rather than aliasing at x1;
//! * the cap-edge flag of `build_rock_detail` on a hand-built column;
//! * on a rendered kilometre-law tile: styling moves land pixels in granite
//!   and on a bedded cap edge, **never moves a water pixel** (§4.10's numeric
//!   bar: a terrace may never move the coastline), and is off by control flow
//!   with no `RockDetail`, under `js_reference()`, and without a map width.

#[path = "../src/render.rs"]
#[allow(dead_code)]
mod render;

use cartalith_terrain::geology::{Construction, GeologyColumn, Rock, NO_LAYER};
use render::{RenderCtx, RockDetail, RockStyle, TerrainAppearance, TileBounds, TileFields, ROCK_CAP_EDGE_BIT};

const GW: usize = 64;
const GH: usize = 48;
const SEA: f64 = 0.42;
/// 25 km over 64 cells: the app's 0.39 km a cell (800 km / 2048).
const MAP_KM: f64 = 25.0;

/// Protects: §4.10's table, rock by rock, as literals. `α = 0.6 + 0.6·s`
/// worked by hand from §2.3's `s`; ridged for massive and columnar jointing;
/// terraced only for bedded or fissile rock **on a cap edge**; fissile steps
/// a quarter of the ladder's; a byte naming no rock has no style.
#[test]
fn the_style_table_is_the_scopes() {
    let st = |r: Rock, edge: bool| render::rock_style(r as u8 | if edge { ROCK_CAP_EDGE_BIT } else { 0 }).expect("a rock has a style");
    let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
    // (rock, alpha, ridge, terrace on an edge, step_k)
    let want: [(Rock, f64, f64, f64, f64); 11] = [
        (Rock::Granite, 1.11, 1.0, 0.0, 1.0),
        (Rock::Gneiss, 1.08, 0.0, 0.0, 1.0),
        (Rock::Schist, 0.96, 0.0, 0.0, 1.0),
        (Rock::PlateauBasalt, 1.05, 1.0, 0.0, 1.0),
        (Rock::OceanicBasalt, 1.05, 0.0, 0.0, 1.0),
        (Rock::Andesite, 1.02, 0.0, 0.0, 1.0),
        (Rock::Tuff, 0.81, 0.0, 1.0, 1.0),
        (Rock::Limestone, 1.02, 0.0, 1.0, 1.0),
        (Rock::Sandstone, 0.96, 0.0, 1.0, 1.0),
        (Rock::Shale, 0.78, 0.0, 1.0, 0.25),
        (Rock::Unconsolidated, 0.66, 0.0, 0.0, 1.0),
    ];
    for (r, alpha, ridge, terr, step_k) in want {
        let e = st(r, true);
        assert!(close(e.alpha, alpha), "{r:?} alpha {} want {alpha}", e.alpha);
        assert!(close(e.ridge, ridge), "{r:?} ridge {}", e.ridge);
        assert!(close(e.terrace, terr), "{r:?} terrace on an edge {}", e.terrace);
        assert!(close(e.step_k, step_k), "{r:?} step_k {}", e.step_k);
        assert_eq!(st(r, false).terrace, 0.0, "{r:?}: no terrace off a cap edge");
        assert_eq!(st(r, false).alpha, e.alpha, "{r:?}: the edge moves only the terrace");
    }
    assert_eq!(render::rock_style(0x7f), None);
    assert_eq!(render::rock_style(NO_LAYER & !ROCK_CAP_EDGE_BIT), None);
}

/// Protects: the terrace step function, by hand at step 1 with the 0.3 riser:
/// a bench holds the step's floor up to 0.7 of the way, the riser's midpoint
/// (0.85) is half way up, a whole step maps to itself, and the function is
/// monotone and continuous over a sweep. And the ladder's literals.
#[test]
fn the_terrace_is_benches_and_risers() {
    assert_eq!(render::terrace_height(0.5, 1.0), 0.0);
    assert_eq!(render::terrace_height(0.69, 1.0), 0.0);
    assert!((render::terrace_height(0.85, 1.0) - 0.5).abs() < 1e-12);
    assert_eq!(render::terrace_height(2.0, 1.0), 2.0);
    assert_eq!(render::terrace_height(2.3, 1.0), 2.0);
    assert_eq!(render::terrace_height(-0.5, 1.0), -1.0);
    assert_eq!(render::terrace_height(0.37, 0.0), 0.37, "no step, no terrace");
    let mut last = f64::NEG_INFINITY;
    for k in 0..4000 {
        let h = -1.0 + k as f64 * 0.001;
        let t = render::terrace_height(h, 0.25);
        assert!(t >= last - 1e-12, "not monotone at {h}");
        if k > 0 {
            assert!((t - last).abs() < 0.02, "a jump at {h}: {last} -> {t}");
        }
        last = t;
    }
    assert_eq!(render::ROCK_TERRACE_STEPS_M, [256.0, 64.0, 16.0, 4.0]);
    assert_eq!(render::ROCK_TERRACE_RISER, 0.3);
    assert_eq!(render::ROCK_TERRACE_FADE_PX, (6.0, 14.0));
    assert_eq!(render::ROCK_TERRACE_FADE_OUT_PX, (40.0, 120.0));
    assert_eq!(render::ROCK_TERRACE_ROUGH, 0.4);
    assert_eq!(render::ROCK_RIDGE_SLOPE_DEG, (4.0, 12.0));
    assert_eq!(render::ROCK_TERRACE_MIX, 0.85);
}

const NEUTRAL: RockStyle = RockStyle { alpha: 1.0, ridge: 0.0, terrace: 0.0, step_k: 1.0 };

/// Protects: with every weight off and `α = 1` the styled height IS the
/// unstyled one, bit for bit (so fbm rock with `α = 1` changes nothing);
/// `α` scales only the sub-cell part; ridged folds it to `-|r|`.
#[test]
fn styled_height_is_identity_when_neutral_and_folds_when_ridged() {
    for (h, c) in [(0.61, 0.6), (0.59, 0.6), (0.6, 0.6)] {
        assert_eq!(render::rock_styled_height(h, c, 0.01, 0.1, 9000.0, NEUTRAL), h);
    }
    let a = RockStyle { alpha: 2.0, ..NEUTRAL };
    assert!((render::rock_styled_height(0.61, 0.6, 0.01, 0.1, 9000.0, a) - 0.62).abs() < 1e-12);
    let r = RockStyle { ridge: 1.0, ..NEUTRAL };
    assert!((render::rock_styled_height(0.61, 0.6, 0.01, 0.1, 9000.0, r) - 0.59).abs() < 1e-12, "a crest folds down");
    assert!((render::rock_styled_height(0.59, 0.6, 0.01, 0.1, 9000.0, r) - 0.59).abs() < 1e-12, "a hollow folds to the same depth");
}

/// Protects: terraces arrive with zoom. On a 30 % slope over 0.39 km cells
/// (0.3 x 390 m / 8 700 m a unit = 0.0134 height per cell), at one pixel per
/// cell the coarsest (256 m) benches fall 2.2 px apart, under the 6 px fade,
/// and the terrace does nothing; at 1/16 of a cell per pixel the 256 m benches
/// are 35 px apart and move the height; at 1/64 the 256 m benches (140 px)
/// have faded OUT and the 64 m (35 px) and 16 m (8.7 px) ones move it by at
/// least a quarter of a 64 m step. A finer step (fissile) gives benches only
/// at the deeper zoom. 8 700 m a unit is `peak_m 5046 / (1 - 0.42)`, the app's.
#[test]
fn terraces_fade_in_with_zoom() {
    let mpu = 5046.0 / (1.0 - SEA);
    let slope = 0.3 * 390.0 / mpu;
    let t = RockStyle { terrace: 1.0, ..NEUTRAL };
    let moved = |cpp: f64, st: RockStyle| -> f64 {
        (0..200).map(|k| 0.6 + k as f64 * 1e-4).map(|h| (render::rock_styled_height(h, h, slope, cpp, mpu, st) - h).abs()).fold(0.0, f64::max)
    };
    assert_eq!(moved(1.0, t), 0.0, "x1: benches below 6 px must not draw");
    assert!(moved(1.0 / 16.0, t) > 0.25 * 64.0 / mpu, "x16: the 256 m benches must draw");
    let deep = moved(1.0 / 64.0, t);
    assert!(deep > 0.25 * 64.0 / mpu, "x64: the terrace barely moved ({deep})");
    // A slope so gentle that even the 4 m benches fall 342 px apart: every
    // octave has faded out, so nothing moves (the band-pass's far side).
    let only_coarse = RockStyle { step_k: 1.0, ..t };
    assert_eq!(render::rock_styled_height(0.6123, 0.6123, slope * 1e-4, 1.0, mpu, only_coarse), 0.6123, "benches thousands of px apart must not draw");
    let shale = RockStyle { step_k: 0.25, ..t };
    assert_eq!(moved(1.0, shale), 0.0);
    assert!(moved(1.0 / 64.0, shale) > 0.0);
}

/// A column with `top` everywhere; where `sub` is `Some`, a substrate whose
/// contact lies at `contact`.
fn column(top: Rock, sub: Option<(Rock, f32)>, n: usize) -> GeologyColumn {
    GeologyColumn {
        rock_top: vec![top as u8; n],
        rock_sub: vec![sub.map_or(NO_LAYER, |s| s.0 as u8); n],
        contact: vec![sub.map_or(f32::NAN, |s| s.1); n],
        regolith: vec![0.0; n],
        volcanic_setting: vec![0; n],
        construction: Construction::Off,
    }
}

/// Protects: `build_rock_detail`'s exposure and cap edge. A limestone cap
/// over shale, contact at 0.60, on a 12 x 4 grid whose surface steps from
/// 0.65 (cap exposed) to 0.55 (shale exposed) between columns 5 and 6:
/// columns 5 and 6 are the escarpment (edge bit on), every other cell not;
/// the exposed rock is limestone west of it and shale east. A single-layer
/// column has no edge anywhere. A short field refuses (`None`), as does a
/// non-positive peak.
#[test]
fn cap_edges_are_where_cap_meets_substrate() {
    let (gw, gh) = (12usize, 4usize);
    let n = gw * gh;
    let field: Vec<f32> = (0..n).map(|i| if i % gw <= 5 { 0.65 } else { 0.55 }).collect();
    let col = column(Rock::Limestone, Some((Rock::Shale, 0.60)), n);
    let rd = render::build_rock_detail(&col, &field, SEA, 5046.0, gw, gh).expect("a column gives a detail");
    assert!((rd.m_per_unit - 5046.0 / 0.58).abs() < 1e-9);
    for i in 0..n {
        let x = i % gw;
        let edge = rd.cells[i] & ROCK_CAP_EDGE_BIT != 0;
        assert_eq!(edge, x == 5 || x == 6, "cell {i} (x {x}) edge {edge}");
        let rock = rd.cells[i] & !ROCK_CAP_EDGE_BIT;
        assert_eq!(rock, if x <= 5 { Rock::Limestone as u8 } else { Rock::Shale as u8 }, "cell {i}");
    }
    let single = render::build_rock_detail(&column(Rock::Granite, None, n), &field, SEA, 5046.0, gw, gh).unwrap();
    assert!(single.cells.iter().all(|&c| c == Rock::Granite as u8), "a single-layer column has no edge");
    assert!(render::build_rock_detail(&col, &field[1..], SEA, 5046.0, gw, gh).is_none());
    assert!(render::build_rock_detail(&col, &field, SEA, 0.0, gw, gh).is_none());
}

struct World {
    field: Vec<f32>,
    temp: Vec<f32>,
    rain: Vec<f32>,
}

/// Sea west of column 8 (0.30), land east of it climbing 0.002 a cell; mild
/// and moist (grass, not snow or forest), so the land colour is the one the
/// light reads.
fn world() -> World {
    let n = GW * GH;
    let mut field = vec![0f32; n];
    for y in 0..GH {
        for x in 0..GW {
            field[y * GW + x] = if x < 8 { 0.30 } else { (0.45 + 0.002 * (x - 8) as f64 + 0.0005 * y as f64) as f32 };
        }
    }
    World { field, temp: vec![12.0; n], rain: vec![0.4; n] }
}

/// Granite (massive) north of row 24; south of it a limestone cap over shale,
/// the contact laid so the escarpment runs through columns 20-24 -- the cap
/// edge the terrace reads.
fn rock_detail(w: &World) -> RockDetail {
    let n = GW * GH;
    let mut col = column(Rock::Limestone, Some((Rock::Shale, 0.0)), n);
    for y in 0..GH {
        for x in 0..GW {
            let i = y * GW + x;
            if y < 24 {
                col.rock_top[i] = Rock::Granite as u8;
                col.rock_sub[i] = NO_LAYER;
                col.contact[i] = f32::NAN;
            } else {
                // The contact sits 0.001 above the surface west of column 22
                // and below it east, so the substrate shows in the west.
                col.contact[i] = w.field[i] + if x < 22 { 0.001 } else { -0.001 };
            }
        }
    }
    let rd = render::build_rock_detail(&col, &w.field, SEA, 5046.0, GW, GH).unwrap();
    assert!(rd.cells.iter().any(|&c| c & ROCK_CAP_EDGE_BIT != 0), "the fixture must have a cap edge");
    rd
}

fn plain() -> TerrainAppearance {
    TerrainAppearance { border_width_frac: 0.0, paper_strength: 0.0, haze_strength: 0.0, ..TerrainAppearance::default() }
}

/// A 64x magnified tile over `bounds` (the coarse plane bilinearly, plus a
/// ripple of four-pixel period and amplitude 0.0004 on land: the sub-cell
/// relief the kilometre law would add). Sea pixels stay at the sea floor.
fn tile(w: &World, bounds: TileBounds, tw: usize, th: usize) -> Vec<f32> {
    let mut t = vec![0f32; tw * th];
    for j in 0..th {
        for i in 0..tw {
            let gx = bounds.x + i as f64 / (tw - 1) as f64 * bounds.w;
            let gy = bounds.y + j as f64 / (th - 1) as f64 * bounds.h;
            let (x0, y0) = (gx.floor() as usize, gy.floor() as usize);
            let (x1, y1) = ((x0 + 1).min(GW - 1), (y0 + 1).min(GH - 1));
            let (tx, ty) = (gx - x0 as f64, gy - y0 as f64);
            let f = |x: usize, y: usize| w.field[y * GW + x] as f64;
            let c = (f(x0, y0) * (1.0 - tx) + f(x1, y0) * tx) * (1.0 - ty) + (f(x0, y1) * (1.0 - tx) + f(x1, y1) * tx) * ty;
            let rip = if c >= SEA { 0.0004 * ((i as f64) * 0.5 * std::f64::consts::PI).sin() * ((j as f64) * 0.5 * std::f64::consts::PI).cos() } else { 0.0 };
            t[j * tw + i] = (c + rip) as f32;
        }
    }
    t
}

/// Renders `bounds` at 64 x 64 with or without the rock detail attached.
fn render_tile(w: &World, rd: Option<&RockDetail>, a: TerrainAppearance, scale: bool, bounds: TileBounds) -> (Vec<u8>, Vec<f32>) {
    let c = RenderCtx::with_appearance(&w.field, &w.temp, &w.rain, None, GW, GH, SEA, false, 55.0, 5.0, a);
    let c = if scale { c.with_map_scale(MAP_KM) } else { c };
    let c = match rd {
        Some(r) => c.with_rock_detail(r),
        None => c,
    };
    let tf = TileFields::new(&c, None).without_lakes();
    let t = tile(w, bounds, 64, 64);
    let px = render::render_biome_tile_rgba(&c, &t, 64, 64, bounds, &tf);
    assert_eq!(px.len(), 64 * 64 * 4);
    assert!(px.iter().any(|&v| v != 0), "an all-black tile proves nothing");
    (px, t)
}

fn changed(a: &[u8], b: &[u8], k: usize) -> bool {
    (0..3).any(|ch| a[k * 4 + ch] != b[k * 4 + ch])
}

/// Protects: on a kilometre-law tile, the rock styling moves land pixels in
/// granite and on the limestone cap edge (positive controls: if styling were
/// inert these counts would be zero), and moves **no** water pixel. The tile
/// straddles the coast (columns 6-10) so the water test has pixels to fail on.
#[test]
fn styling_moves_rock_and_never_water() {
    let w = world();
    let rd = rock_detail(&w);
    for (name, bounds, min_moved) in [
        ("granite coast", TileBounds { x: 6.0, y: 8.0, w: 4.0, h: 4.0 }, 200),
        ("limestone cap edge", TileBounds { x: 19.0, y: 30.0, w: 4.0, h: 4.0 }, 200),
    ] {
        let (off, t) = render_tile(&w, None, plain(), true, bounds);
        let (on, _) = render_tile(&w, Some(&rd), plain(), true, bounds);
        let (mut land, mut water_moved, mut water) = (0, 0, 0);
        for k in 0..64 * 64 {
            if (t[k] as f64) < SEA {
                water += 1;
                if changed(&off, &on, k) {
                    water_moved += 1;
                }
            } else if changed(&off, &on, k) {
                land += 1;
            }
        }
        println!("{name}: land moved {land}, water {water} px, water moved {water_moved}");
        assert_eq!(water_moved, 0, "{name}: rock styling moved a water pixel");
        assert!(land >= min_moved, "{name}: rock styling barely reached the land ({land} px)");
        if name == "granite coast" {
            assert!(water > 500, "{name}: the tile must hold water to test it");
        }
    }
}

/// Protects: off by control flow -- with the rock detail attached, a tile
/// under `js_reference()` (no kilometre law) and a context without a map width
/// both render byte-identical to the same tile without it.
#[test]
fn styling_is_off_without_the_kilometre_law() {
    let w = world();
    let rd = rock_detail(&w);
    let b = TileBounds { x: 4.0, y: 8.0, w: 4.0, h: 4.0 };
    for (name, a, scale) in [("js_reference", TerrainAppearance::js_reference(), true), ("no map width", plain(), false), ("law at 0", TerrainAppearance { zoom_detail_km: 0.0, ..plain() }, true)] {
        let (off, _) = render_tile(&w, None, a.clone(), scale, b);
        let (on, _) = render_tile(&w, Some(&rd), a, scale, b);
        assert_eq!(off, on, "{name}: rock styling acted off the kilometre law");
    }
}

/// Protects: a mismatched rock detail is ignored by the builder (no
/// out-of-bounds read inside the tile loop): the tile is the unstyled one.
#[test]
fn a_mismatched_detail_is_ignored() {
    let w = world();
    let bad = RockDetail { cells: vec![Rock::Granite as u8; 10], m_per_unit: 8700.0 };
    let b = TileBounds { x: 4.0, y: 8.0, w: 4.0, h: 4.0 };
    assert_eq!(render_tile(&w, None, plain(), true, b).0, render_tile(&w, Some(&bad), plain(), true, b).0);
}

/// Protects: the ridged style's slope gate, worked by hand: at 390 m a cell
/// and 8 700 m a unit, a grid slope whose angle is 4 degrees or less gives 0,
/// 12 or more gives 1, and 8 (the middle of the smoothstep) gives exactly 0.5;
/// a degenerate scale gives 0, never a plausible weight.
#[test]
fn ridging_is_gated_by_slope() {
    let (mpu, cell) = (8700.0, 390.0);
    let at = |deg: f64| render::ridge_slope_gate(deg.to_radians().tan() * cell / mpu, mpu, cell);
    assert_eq!(at(0.5), 0.0);
    assert_eq!(at(3.99), 0.0);
    assert!((at(8.0) - 0.5).abs() < 1e-9, "8 degrees: {}", at(8.0));
    assert_eq!(at(12.01), 1.0);
    assert_eq!(at(30.0), 1.0);
    assert_eq!(render::ridge_slope_gate(0.01, 0.0, cell), 0.0);
    assert_eq!(render::ridge_slope_gate(0.01, mpu, f64::NAN), 0.0);
}

/// Protects: the styled height stays **monotone** in the unstyled one with
/// every terrace octave that can be in view at once (x4 to x64, gentle to
/// steep ground) -- no bench tilts backwards. That is what normalising the
/// octaves' sum (`ROCK_TERRACE_MIX / max(1, Σβ)`) is for: two octaves fully
/// in view at a full mix each would turn a bench's slope negative.
#[test]
fn terraced_height_is_monotone_at_every_zoom() {
    let mpu = 5046.0 / (1.0 - SEA);
    let t = RockStyle { terrace: 1.0, ..NEUTRAL };
    for deg in [0.5f64, 2.0, 8.0, 30.0] {
        let slope = deg.to_radians().tan() * 390.0 / mpu;
        for cpp in [1.0 / 1.86, 1.0 / 7.42, 1.0 / 29.7, 1.0 / 64.0] {
            let mut last = f64::NEG_INFINITY;
            for k in 0..20000 {
                let h = 0.5 + k as f64 * 2e-6;
                let s = render::rock_styled_height(h, h, slope, cpp, mpu, t);
                assert!(s >= last - 1e-12, "{deg} deg, {cpp} cells/px: not monotone at {h} ({last} -> {s})");
                last = s;
            }
        }
    }
}
