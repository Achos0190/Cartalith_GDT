//! Ruling CR's switched sprite-chain fix (2026-10-18, last paragraph:
//! "Fixing the blit and the bake is a separate, cheaper change behind a
//! switch"), measured.
//!
//! The chain a pack sprite takes onto the base map: an author's PNG (here a
//! synthetic 1024² source) -> the library's 256² bake
//! (`cartalith_assets::render_item_with`, [`BakeFilter`]) -> the map raster
//! blit at `sprite_draw_rect`'s ~41 grid px (`pack::composite_placed_features`,
//! [`pack::SpriteFilter`]) -> the view, which draws the base raster with
//! `TEXTURE_FILTER_NEAREST` (`viewport_host.gd::_raster`), so ×4/×16/×64 are
//! nearest magnifications of the ×1 grid pixels.
//!
//! Four arms (bake × blit) on three synthetic samples, measured against an
//! analytic truth at ×1, ×4, ×16 and ×64 with
//! `LINEART_METHODS_RESEARCH_2026-10-18.md` §3's metrics (ink mass, edge
//! width, halo; the halo here counts any pixel off the ground-to-ink line,
//! the dark fringe as well as light paper). The table prints with
//! `--nocapture`. The asserted bars are on the app's arm and are labelled
//! where they are judgements.
//!
//! Same `#[path]`-include technique as `pack_compositing.rs` (this crate is a
//! cdylib).
#![allow(dead_code)]

#[path = "../src/render.rs"]
mod render;
/// The sprite compositor under test.
#[path = "../src/pack.rs"]
mod pack;

use cartalith_assets::{BakeFilter, DecodedImage, IconCategory, ItemTransform, PlacedIcon, SpriteRect, render_item_with};
use pack::{FeatureArt, PlacedFeature, SpriteFilter};

/// The map green the sprite is drawn over (a fixture value, the same role as
/// the research lab's ground; any colour far from the inks works).
const GROUND: [f64; 3] = [96.0, 140.0, 70.0];
/// The grid the blit draws into (fixture size; the rectangle sits inside it).
const GRID: usize = 64;
/// `sprite_draw_rect`'s box for `s = 1` on a 2048-wide grid:
/// `base = 2048 / 110`, drawn `2.2 * base` = 40.96 px tall
/// (`LINEART_METHODS_RESEARCH_2026-10-18.md` §1 step 3), at a fractional
/// origin so no edge is grid-aligned.
const RECT: SpriteRect = SpriteRect { dx: 10.3, dy: 7.6, dw: 40.96, dh: 40.96 };

/// One synthetic sample: an ink colour and an analytic coverage function in
/// the sprite's unit square.
struct Sample {
    name: &'static str,
    ink: [u8; 3],
    /// Thin strokes: segments `(x0, y0, x1, y1, width)`, unit coordinates.
    segs: Vec<(f64, f64, f64, f64, f64)>,
    /// Painted: a soft disc's alpha at a point instead of segments.
    soft: Option<fn(f64, f64) -> f64>,
}

/// The pine of the research lab, simplified: a trunk and nine pairs of
/// branches, every stroke 6 px wide at 1024 (0.24 grid px at ×1).
fn pine() -> Sample {
    let w = 6.0 / 1024.0;
    let mut segs = vec![(0.5, 0.08, 0.5, 0.95, w)];
    for i in 0..9 {
        let y = 0.15 + 0.08 * f64::from(i);
        let reach = 0.1 + 0.03 * f64::from(i);
        segs.push((0.5, y, 0.5 - reach, y + 0.07, w));
        segs.push((0.5, y, 0.5 + reach, y + 0.07, w));
    }
    Sample { name: "pine", ink: [30, 40, 25], segs, soft: None }
}

/// Outlines with enclosed holes (the castle's role): a keep, two towers and
/// four windows, every stroke 12 px wide at 1024 (0.48 grid px at ×1).
fn castle() -> Sample {
    let w = 12.0 / 1024.0;
    let mut segs = Vec::new();
    let mut rect = |x0: f64, y0: f64, x1: f64, y1: f64| {
        segs.extend([(x0, y0, x1, y0, w), (x1, y0, x1, y1, w), (x1, y1, x0, y1, w), (x0, y1, x0, y0, w)]);
    };
    rect(0.25, 0.35, 0.75, 0.92);
    rect(0.12, 0.2, 0.27, 0.92);
    rect(0.73, 0.2, 0.88, 0.92);
    for (x, y) in [(0.36, 0.5), (0.56, 0.5), (0.36, 0.7), (0.56, 0.7)] {
        rect(x, y, x + 0.08, y + 0.1);
    }
    Sample { name: "castle", ink: [45, 35, 30], segs, soft: None }
}

/// Painted art: a light disc whose rim fades out over 12 px at 1024 (3 px
/// of the 256² bake), over transparent black (the dark-fringe case).
fn painted() -> Sample {
    fn alpha(u: f64, v: f64) -> f64 {
        let r = (u - 0.5).hypot(v - 0.5);
        ((0.42 - r) / (12.0 / 1024.0)).clamp(0.0, 1.0)
    }
    Sample { name: "painted", ink: [235, 210, 140], segs: Vec::new(), soft: Some(alpha) }
}

/// Distance from `(px, py)` to the segment `(x0, y0)-(x1, y1)`.
fn seg_dist(px: f64, py: f64, (x0, y0, x1, y1): (f64, f64, f64, f64)) -> f64 {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let t = (((px - x0) * dx + (py - y0) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    (px - x0 - t * dx).hypot(py - y0 - t * dy)
}

/// Coverage of `sample` on an `nx × ny` pixel grid whose pixel `(i, j)`
/// covers unit square `[(i - ox) / scale, ...)`, by `ss × ss` supersampling;
/// strokes are a union (stamped per segment over its own bounding box).
/// `floor_px`: every stroke at least that many output pixels wide (the
/// research's "floored target", §2), 0 for the true widths.
fn coverage(sample: &Sample, nx: usize, ny: usize, ox: f64, oy: f64, scale: f64, ss: usize, floor_px: f64) -> Vec<f64> {
    let (sx, sy) = (nx * ss, ny * ss);
    let to_u = |s: usize, o: f64| ((s as f64 + 0.5) / ss as f64 - o) / scale;
    let mut acc = vec![0.0f64; nx * ny];
    if let Some(f) = sample.soft {
        for y in 0..sy {
            for x in 0..sx {
                acc[(y / ss) * nx + x / ss] += f(to_u(x, ox), to_u(y, oy));
            }
        }
    } else {
        let mut hit = vec![false; sx * sy];
        for &(x0, y0, x1, y1, w) in &sample.segs {
            let r = w.max(floor_px / scale) / 2.0;
            let to_s = |u: f64, o: f64| ((u * scale + o) * ss as f64).floor();
            let xa = to_s(x0.min(x1) - r, ox).max(0.0) as usize;
            let xb = (to_s(x0.max(x1) + r, ox) + 1.0).clamp(0.0, sx as f64) as usize;
            let ya = to_s(y0.min(y1) - r, oy).max(0.0) as usize;
            let yb = (to_s(y0.max(y1) + r, oy) + 1.0).clamp(0.0, sy as f64) as usize;
            for y in ya..yb {
                for x in xa..xb {
                    if seg_dist(to_u(x, ox), to_u(y, oy), (x0, y0, x1, y1)) <= r {
                        hit[y * sx + x] = true;
                    }
                }
            }
        }
        for y in 0..sy {
            for x in 0..sx {
                if hit[y * sx + x] {
                    acc[(y / ss) * nx + x / ss] += 1.0;
                }
            }
        }
    }
    let n = (ss * ss) as f64;
    acc.iter().map(|v| v / n).collect()
}

/// The author's 1024² PNG: the ink colour where coverage > 0, alpha the
/// coverage (4×4 supersampled), black under alpha 0.
fn source(sample: &Sample) -> DecodedImage {
    let n = 1024usize;
    let cov = coverage(sample, n, n, 0.0, 0.0, n as f64, 4, 0.0);
    let mut rgba = Vec::with_capacity(n * n * 4);
    for c in cov {
        let a = (c * 255.0).round() as u8;
        let rgb = if a > 0 { sample.ink } else { [0, 0, 0] };
        rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], a]);
    }
    DecodedImage::new(n as u32, n as u32, rgba).unwrap()
}

/// A `LoadedPack` whose `mountain` slot holds just `sprite` (the reference
/// fixture pack with its icons replaced).
fn pack_with(sprite: DecodedImage) -> pack::LoadedPack {
    let bytes = std::fs::read("../cartalith-assets/tests/fixtures/reference_pack.zip").expect("reference_pack.zip fixture must exist");
    let mut p = pack::load_pack_from_bytes(bytes).expect("the reference pack loads");
    p.icons.clear();
    p.icons.insert("mountain".into(), vec![sprite]);
    p
}

/// The 256² bake under `bake`, blitted at [`RECT`] under `blit` into a
/// [`GROUND`] grid: the RGB8 raster the view magnifies.
fn chain(src: &DecodedImage, bake: BakeFilter, blit: SpriteFilter) -> Vec<u8> {
    let baked = render_item_with(src, &ItemTransform::default(), 256, false, bake);
    let p = pack_with(baked);
    let mut bytes: Vec<u8> = (0..GRID * GRID).flat_map(|_| GROUND.map(|c| c as u8)).collect();
    let item = PlacedIcon { x: 30, y: 48, s: 1.0, key: Some("mountain".into()), cat: IconCategory::Ruled, kind: None };
    let placed = [PlacedFeature { item, slot: "mountain".into(), art: FeatureArt::Sprite { variant: 0, rect: RECT } }];
    pack::composite_placed_features(&mut bytes, GRID, GRID, &placed, &p, true, blit);
    bytes
}

/// What one arm measures at one zoom.
#[derive(Debug, Clone, Copy)]
struct Measure {
    /// Total effective ink over the un-floored truth's coverage (1.0 is a
    /// faithful tone).
    ink: f64,
    /// Pixels with 0.1 < ink < 0.9 per boundary pixel of the floored
    /// target (every stroke at least 1 output px wide, §2's target), in
    /// screen px. A 1 px stroke drawn faint counts its own pixels too.
    edge: f64,
    /// Line breaks: the share of the floored target's pixels whose
    /// effective ink is below 0.05 (a stroke "hit or missed by chance").
    gaps: f64,
    /// Share of the footprint more than 6 levels off the ground-to-ink line
    /// (a dark fringe or light paper), or below -0.03 / above 1.03 along it.
    halo: f64,
}

/// Measure the raster `bytes` at zoom `k` (nearest magnification) against
/// the analytic truth at that zoom, over the rectangle's pixel box.
fn measure(sample: &Sample, bytes: &[u8], k: usize) -> Measure {
    let (gx0, gx1) = (RECT.dx.floor() as usize, (RECT.dx + RECT.dw).ceil() as usize);
    let (gy0, gy1) = (RECT.dy.floor() as usize, (RECT.dy + RECT.dh).ceil() as usize);
    let (nx, ny) = ((gx1 - gx0) * k, (gy1 - gy0) * k);
    // Truth at zoom k: unit coordinate u = (map_x - dx) / dw, map_x = gx0 + i / k.
    let scale = RECT.dw * k as f64;
    let (ox, oy) = ((RECT.dx - gx0 as f64) * k as f64, (RECT.dy - gy0 as f64) * k as f64);
    let ss = match k {
        1 => 16,
        4 => 4,
        16 => 2,
        _ => 1,
    };
    let truth = coverage(sample, nx, ny, ox, oy, scale, ss, 0.0);
    let target = coverage(sample, nx, ny, ox, oy, scale, ss, 1.0);
    let ink = sample.ink.map(f64::from);
    let d = [ink[0] - GROUND[0], ink[1] - GROUND[1], ink[2] - GROUND[2]];
    let dd = d.iter().map(|v| v * v).sum::<f64>();
    let (mut mass, mut tsum, mut mid, mut boundary, mut halo, mut foot) = (0.0, 0.0, 0usize, 0usize, 0usize, 0usize);
    let (mut tpx, mut gap) = (0usize, 0usize);
    for j in 0..ny {
        for i in 0..nx {
            let (gx, gy) = (gx0 + i / k, gy0 + j / k);
            let o = (gy * GRID + gx) * 3;
            let p = [f64::from(bytes[o]), f64::from(bytes[o + 1]), f64::from(bytes[o + 2])];
            let v = [p[0] - GROUND[0], p[1] - GROUND[1], p[2] - GROUND[2]];
            let e = (v[0] * d[0] + v[1] * d[1] + v[2] * d[2]) / dd;
            let off = (0..3).map(|c| (v[c] - e * d[c]).powi(2)).sum::<f64>().sqrt();
            let t = truth[j * nx + i];
            mass += e.clamp(0.0, 1.0);
            tsum += t;
            if e > 0.1 && e < 0.9 {
                mid += 1;
            }
            let inside = target[j * nx + i] >= 0.5;
            if inside {
                tpx += 1;
                if e < 0.05 {
                    gap += 1;
                }
            }
            let nb = |a: usize, b: usize| target[b * nx + a] >= 0.5;
            if inside && ((i > 0 && !nb(i - 1, j)) || (i + 1 < nx && !nb(i + 1, j)) || (j > 0 && !nb(i, j - 1)) || (j + 1 < ny && !nb(i, j + 1))) {
                boundary += 1;
            }
            if t > 0.0 || e.abs() > 0.01 || off > 0.5 {
                foot += 1;
                if off > 6.0 || e < -0.03 || e > 1.03 {
                    halo += 1;
                }
            }
        }
    }
    Measure { ink: mass / tsum, edge: mid as f64 / boundary.max(1) as f64, gaps: gap as f64 / tpx.max(1) as f64, halo: halo as f64 / foot.max(1) as f64 }
}

/// The four arms, the app's last.
const ARMS: [(&str, BakeFilter, SpriteFilter); 4] = [
    ("straight+bilinear (shipped)", BakeFilter::Straight, SpriteFilter::Bilinear),
    ("premul+bilinear", BakeFilter::Premultiplied, SpriteFilter::Bilinear),
    ("straight+area", BakeFilter::Straight, SpriteFilter::Area),
    ("premul+area (app)", BakeFilter::Premultiplied, SpriteFilter::Area),
];

/// Protects Ruling CR's switched fix with measured bars, on the app's arm
/// (premultiplied bake + area blit), at ×1, ×4, ×16 and ×64:
/// - **halo 0** on every sample and zoom (no dark fringe, no paper);
/// - **ink mass 0.9-1.1** on every sample (the research's 0.95-1.05 band
///   for the SDF path, widened for a filter: a labelled judgement; measured
///   0.990-1.061);
/// - **strokes do not break**: at most 3 % of the floored target's pixels
///   missed (measured 0.000-0.019), where the shipped single sample misses
///   15-56 % (the positive control, at least 10 %);
/// - **edge width at ×1 between 0.8 and 1.5 px**, i.e. a crisp
///   anti-aliased edge by the research's definition ("about 1"; measured
///   0.98-1.12). The shipped sample's 0.41-0.68 is narrower because it is
///   aliased, not sharper.
/// Positive control for the halo: the shipped straight bake fringes the
/// castle's dark-brown ink with black (measured 0.107 of the footprint).
///
/// **Disclosed, not a bar:** the view magnifies the base raster with
/// nearest filtering, so on this baked path every arm's edge width grows
/// with the zoom (the app's: about 3, 9 and 34 px at ×4, ×16 and ×64 on the
/// pine). Sharp sprites at deep zoom are AP-5's overlay path, which the app
/// uses for feature icons; this fix is what the baked raster draws, which is
/// every zoom when that overlay is off. And an area filter is faithful, so a
/// stroke thinner than a pixel draws continuous but faint (the pine's
/// 0.24 px strokes): solid thin lines need AP-10's stroke floor, not this.
#[test]
fn the_switched_chain_keeps_ink_and_draws_no_halo_at_every_zoom() {
    for sample in [pine(), castle(), painted()] {
        let src = source(&sample);
        let rasters: Vec<Vec<u8>> = ARMS.iter().map(|&(_, b, f)| chain(&src, b, f)).collect();
        let mut table = Vec::new();
        for k in [1usize, 4, 16, 64] {
            let row: Vec<Measure> = rasters.iter().map(|r| measure(&sample, r, k)).collect();
            for (arm, m) in ARMS.iter().zip(&row) {
                eprintln!("{:8} x{:<3} {:30} ink {:.3}  gaps {:.3}  edge {:.2} px  halo {:.4}", sample.name, k, arm.0, m.ink, m.gaps, m.edge, m.halo);
            }
            table.push(row);
        }
        for (zi, row) in table.iter().enumerate() {
            let (shipped, app) = (row[0], row[3]);
            let z = [1, 4, 16, 64][zi];
            assert_eq!(app.halo, 0.0, "{} x{z}: the app's arm draws a halo", sample.name);
            assert!((0.9..=1.1).contains(&app.ink), "{} x{z}: ink mass {:.3}", sample.name, app.ink);
            if sample.soft.is_none() {
                assert!(app.gaps <= 0.03, "{} x{z}: the app's strokes break ({:.3} of the target missed)", sample.name, app.gaps);
                assert!(shipped.gaps >= 0.1, "positive control, {} x{z}: the shipped sample breaks strokes ({:.3})", sample.name, shipped.gaps);
            }
        }
        let e1 = table[0][3].edge;
        assert!((0.8..=1.5).contains(&e1), "{}: x1 edge {e1:.2} px is not a crisp anti-aliased edge", sample.name);
        if sample.name == "castle" {
            assert!(table[0][0].halo > 0.05, "positive control: the straight bake's fringe on the castle ({:.4})", table[0][0].halo);
        }
    }
}

/// Protects: the `Straight` arm this file measures as "shipped" is the
/// shipped bake, `render_item` byte for byte (the `Bilinear` blit is pinned
/// as the shipped compositor by `feature_icon_overlay.rs`'s FNV digest).
#[test]
fn both_switches_off_are_the_shipped_bake() {
    let src = source(&castle());
    let t = ItemTransform::default();
    assert_eq!(render_item_with(&src, &t, 256, false, BakeFilter::Straight), cartalith_assets::render_item(&src, &t, 256, false));
}

/// Protects the area blit's arithmetic on a case solvable by hand: an
/// opaque 4×4 red sprite drawn at exactly 2×2 grid pixels on integer
/// bounds paints those four pixels pure red and nothing else; drawn at
/// (0.5, 0.5) it paints the corner pixel a quarter covered:
/// round(255 * 0.25 + 96 * 0.75) = 136 in red.
#[test]
fn the_area_blit_covers_exactly_its_rectangle() {
    let red = DecodedImage::new(4, 4, [255u8, 0, 0, 255].repeat(16)).unwrap();
    let p = pack_with(red);
    let draw = |rect: SpriteRect| {
        let mut bytes: Vec<u8> = (0..8 * 8).flat_map(|_| GROUND.map(|c| c as u8)).collect();
        let item = PlacedIcon { x: 0, y: 0, s: 1.0, key: Some("mountain".into()), cat: IconCategory::Ruled, kind: None };
        pack::composite_placed_features(&mut bytes, 8, 8, &[PlacedFeature { item, slot: "mountain".into(), art: FeatureArt::Sprite { variant: 0, rect } }], &p, true, SpriteFilter::Area);
        bytes
    };
    let b = draw(SpriteRect { dx: 2.0, dy: 3.0, dw: 2.0, dh: 2.0 });
    for y in 0..8 {
        for x in 0..8 {
            let px = &b[(y * 8 + x) * 3..(y * 8 + x) * 3 + 3];
            let want: [u8; 3] = if (2..4).contains(&x) && (3..5).contains(&y) { [255, 0, 0] } else { GROUND.map(|c| c as u8) };
            assert_eq!(px, want, "({x}, {y})");
        }
    }
    let b = draw(SpriteRect { dx: 0.5, dy: 0.5, dw: 2.0, dh: 2.0 });
    assert_eq!(&b[0..3], &[136, 105, 53], "a quarter-covered corner: 0.25 red over 0.75 ground");
}
