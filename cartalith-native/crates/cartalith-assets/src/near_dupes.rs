//! AP-6 (`ASSET_PACK_TOOLS_PROPOSAL.md` §6, Ruling CQ): find the *near*
//! duplicates a merged pack accumulates -- the same drawing mirrored, tinted
//! a little, or nudged by a pixel -- which [`crate::item_hash`] cannot see,
//! because it is an exact content hash and any of those edits changes it.
//!
//! **The method** is a 64-bit difference hash ("dHash", N. Krawetz, *Kind of
//! Like That*, hackerfactor.com, 2013), also tested mirrored:
//!
//! 1. take the image's **content box**: the bounding box of pixels with alpha
//!    above [`BLANK_ALPHA_THRESHOLD`] (the slicer's own blank test), so a
//!    sprite nudged by a pixel inside its canvas, or padded, hashes the same
//!    drawing. An opaque ground tile's box is the whole tile;
//! 2. reduce it to a grey `9 × 8` grid by exact area averaging (every source
//!    pixel contributes by the fraction of it each cell covers, so the grid
//!    of a mirrored image is exactly the mirrored grid). A cell's grey value
//!    is the **shading** of the ink in it (alpha-weighted Rec. 601 luma; a
//!    cell with no ink takes the drawing's own mean, so empty space adds no
//!    edge of its own) plus [`SILHOUETTE_WEIGHT`] × 255 × its **coverage**
//!    (mean alpha). Both halves carry information: the shading separates two
//!    drawings of one outline, the coverage term separates outlines and
//!    keeps flat line art (all luma 0) from hashing to nothing, and since a
//!    multiplicative tint scales the shading of every cell alike, neither
//!    flips a bit under it. *Measured while building* (two rejected
//!    designs): plain luma over mid-grey 128 let a ±10 % tint flip every edge
//!    cell whose ink sat near 128 (12 bits between the +10 % and −10 % forms
//!    of one fixture sprite); ink luma lifted to `128 + luma / 2` and
//!    premultiplied over black let the outline swamp the shading, so two
//!    differently shaded drawings of one diamond outline came 3-7 bits apart;
//! 3. bit `(r, c)` is set when cell `c + 1` of row `r` is brighter than cell
//!    `c` by more than [`DEAD_BAND`] levels.
//!
//! Two images are near-duplicates when the Hamming distance between their
//! hashes -- or between one and the *mirrored* hash of the other -- is at
//! most [`MAX_HAMMING`], **and** their mean colours agree within
//! [`MAX_MEAN_DELTA`] on every channel. The colour gate exists because dHash
//! sees only brightness gradients: two flat tiles of different colours both
//! hash to zero, and without it every flat image in a pack (the reference
//! fixture is nothing but flat colours) would be one "near-duplicate" group.
//!
//! Must never: change the library or its pixels (read-only), or report a
//! group whose members are all exact copies -- [`crate::duplicate_groups`]
//! already names those, and the pack report lists them once.

use crate::library::AssetDB;
use crate::raster::{DecodedImage, render_item};
use crate::slicer::BLANK_ALPHA_THRESHOLD;

/// The largest Hamming distance (of 64 bits) still called "the same
/// drawing". AP-6's own acceptance bar (proposal §6: "every pair is grouped
/// (Hamming ≤ 10)"), which is also the threshold Krawetz suggests for dHash
/// ("a distance above 10 is probably a different image").
pub const MAX_HAMMING: u32 = 10;

/// The largest per-channel difference of mean colour (8-bit levels) between
/// two near-duplicates. Labelled judgement: AP-6's fixture tints by ±10 %,
/// which moves a 255-level channel by 25.5 levels; 32 leaves margin above
/// that while still separating, say, a green tree from a brown one.
pub const MAX_MEAN_DELTA: f64 = 32.0;

/// A grid-cell difference (8-bit grey levels) at or below which no bit is
/// set. Labelled judgement: classic dHash sets a bit on any `>`, which in a
/// flat region turns float rounding (cell means that differ by 1e-13) into
/// random bits. Half a level is below anything a PNG can encode as a real
/// gradient after 9 × 8 averaging, and well above the rounding.
pub const DEAD_BAND: f64 = 0.5;

/// How much a cell's ink coverage counts against its shading, as a fraction
/// of full scale (so 25.5 levels for a fully covered cell next to an empty
/// one). Labelled judgement, chosen on AP-6's fixture (ten distinct 256²
/// drawings, each also flipped, tinted ±10 % and shifted 1 px). *Measured
/// 2026-10-19*, as (worst distance between two forms of one sprite, closest
/// distance between two distinct sprites, hash alone): 0.05 -> (2, 7);
/// 0.1 -> (3, 8); 0.2 -> (2, 11). So at 0.2 the hash alone separates the
/// ten drawings by one bit more than [`MAX_HAMMING`] -- a thin margin, which
/// is why the colour gate stays part of the rule -- while flat black line art
/// keeps its outline in the hash (`black_line_art_keeps_its_outline`) and
/// two shadings of one outline stay 14 bits apart.
pub const SILHOUETTE_WEIGHT: f64 = 0.2;

/// One image's AP-6 fingerprint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NearHash {
    /// The dHash of the image as it is.
    pub hash: u64,
    /// The dHash of its left-right mirror.
    pub mirrored: u64,
    /// Alpha-weighted mean R, G, B of the content box, in 8-bit levels.
    pub mean: [f64; 3],
}

impl NearHash {
    /// The distance AP-6 groups by: the smaller of the direct and the
    /// mirrored Hamming distances (a flipped copy is still a copy).
    ///
    /// Both mirror pairings are tried, so the distance is symmetric: the dead
    /// band makes a mirrored hash something other than a bit permutation of
    /// the direct one, so `self` against `other`'s mirror and `self`'s mirror
    /// against `other` can differ by a bit or two.
    pub fn distance(&self, other: &NearHash) -> u32 {
        let direct = (self.hash ^ other.hash).count_ones();
        let flipped = (self.hash ^ other.mirrored).count_ones().min((self.mirrored ^ other.hash).count_ones());
        direct.min(flipped)
    }

    /// Whether the image had no gradient anywhere the 9 × 8 grid can see
    /// (both hashes zero): a flat swatch. [`near_duplicate_groups`] leaves
    /// flat images out, because with no structure to compare, "the same
    /// drawing" would rest on the colour gate alone.
    pub fn is_flat(&self) -> bool {
        self.hash == 0 && self.mirrored == 0
    }

    /// Whether `self` and `other` are near-duplicates: within
    /// [`MAX_HAMMING`] and within [`MAX_MEAN_DELTA`] on every channel.
    pub fn is_near(&self, other: &NearHash) -> bool {
        self.distance(other) <= MAX_HAMMING
            && (0..3).all(|c| (self.mean[c] - other.mean[c]).abs() <= MAX_MEAN_DELTA)
    }
}

/// The content box `(x0, y0, x1, y1)` (exclusive ends) of pixels with alpha
/// above [`BLANK_ALPHA_THRESHOLD`], or `None` for a blank image.
fn content_box(img: &DecodedImage) -> Option<(usize, usize, usize, usize)> {
    let (w, h) = (img.w as usize, img.h as usize);
    let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
    for y in 0..h {
        for x in 0..w {
            if img.rgba[(y * w + x) * 4 + 3] > BLANK_ALPHA_THRESHOLD {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x + 1);
                y1 = y1.max(y + 1);
            }
        }
    }
    (x0 != usize::MAX).then_some((x0, y0, x1, y1))
}

/// The weights with which source pixels `0..len` (offset into the box)
/// cover each of `cells` equal cells across `len`: `out[cell]` lists
/// `(pixel, fraction of that pixel inside the cell)`. Exact area overlap, so
/// the weights of the mirrored axis are the mirrored weights.
fn axis_weights(len: usize, cells: usize) -> Vec<Vec<(usize, f64)>> {
    let step = len as f64 / cells as f64;
    (0..cells)
        .map(|c| {
            let (a, b) = (c as f64 * step, (c + 1) as f64 * step);
            let first = a.floor() as usize;
            let last = (b.ceil() as usize).min(len);
            (first..last)
                .filter_map(|p| {
                    let ov = (b.min(p as f64 + 1.0) - a.max(p as f64)).max(0.0);
                    (ov > 0.0).then_some((p, ov))
                })
                .collect()
        })
        .collect()
}

/// The dHash bits of a `9 × 8` grey grid (row-major), reading columns left to
/// right, or right to left when `mirror` is set.
fn bits(grid: &[[f64; 9]; 8], mirror: bool) -> u64 {
    let mut out = 0u64;
    for (r, row) in grid.iter().enumerate() {
        for c in 0..8 {
            let (l, rt) = if mirror { (row[8 - c], row[7 - c]) } else { (row[c], row[c + 1]) };
            if rt - l > DEAD_BAND {
                out |= 1 << (r * 8 + c);
            }
        }
    }
    out
}

/// AP-6's fingerprint of `img`, or `None` for a blank image (nothing to
/// compare; the pack report already names blank images).
pub fn near_hash(img: &DecodedImage) -> Option<NearHash> {
    let (x0, y0, x1, y1) = content_box(img)?;
    let w = img.w as usize;
    let px = |x: usize, y: usize| -> (f64, f64, [f64; 3]) {
        let o = (y * w + x) * 4;
        let p = &img.rgba[o..o + 4];
        let rgb = [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])];
        (f64::from(p[3]) / 255.0, 0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2], rgb)
    };
    // The alpha-weighted mean colour and luma of the whole content box.
    let (mut m, mut lsum, mut asum) = ([0.0f64; 3], 0.0, 0.0);
    for y in y0..y1 {
        for x in x0..x1 {
            let (a, luma, rgb) = px(x, y);
            for c in 0..3 {
                m[c] += rgb[c] * a;
            }
            lsum += luma * a;
            asum += a;
        }
    }
    // `content_box` found a pixel with alpha > 8, so `asum > 0`.
    let mean = [m[0] / asum, m[1] / asum, m[2] / asum];
    let mean_luma = lsum / asum;

    let wx = axis_weights(x1 - x0, 9);
    let wy = axis_weights(y1 - y0, 8);
    let mut grid = [[0.0f64; 9]; 8];
    for (r, ry) in wy.iter().enumerate() {
        for (c, cx) in wx.iter().enumerate() {
            // Per cell: the area, the ink (alpha) in it, and the ink's luma.
            let (mut area, mut ink, mut ink_luma) = (0.0, 0.0, 0.0);
            for &(py, fy) in ry {
                for &(pxx, fx) in cx {
                    let (a, luma, _) = px(x0 + pxx, y0 + py);
                    let wt = fx * fy;
                    area += wt;
                    ink += a * wt;
                    ink_luma += a * luma * wt;
                }
            }
            let shade = if ink > 0.0 { ink_luma / ink } else { mean_luma };
            let cover = if area > 0.0 { ink / area } else { 0.0 };
            grid[r][c] = shade + SILHOUETTE_WEIGHT * 255.0 * cover;
        }
    }
    Some(NearHash { hash: bits(&grid, false), mirrored: bits(&grid, true), mean })
}

/// One member of a near-duplicate group.
#[derive(Debug, Clone, PartialEq)]
pub struct NearEntry {
    /// The slot's uid.
    pub uid: String,
    /// The variant's index in that slot.
    pub idx: usize,
    /// The item's exact content hash ([`crate::item_hash`]), so a caller can
    /// tell an exact copy inside a near group.
    pub item_hash: String,
}

/// Group every item whose bake is a near-duplicate of another
/// ([`NearHash::is_near`]), transitively (a chain A~B~C is one group).
///
/// Each item is fingerprinted on **its bake** (`render_item` at its family's
/// size, opaque for ground tiles), which is what a pack ships and the map
/// draws, not the source at its import size. `pixels(uid, index)` is the
/// session's source store; an item it cannot return, or a blank one, is left
/// out rather than guessed at. Groups and their members are in store order
/// ([`AssetDB::uids_in_order`], the same order as
/// [`crate::duplicate_groups`]); a group whose members all share one exact
/// hash is dropped (exact duplicates are reported as such already).
///
/// Two rules keep it to drawings, both *found on the reference pack* (a
/// format fixture of flat swatches, where the first version reported its
/// rock texture, road tile and four settlement and POI sprites as one
/// six-member group):
///
/// - ground tiles are compared only with ground tiles and sprites only with
///   sprites: a 512² opaque tile and a 256² sprite are never interchangeable
///   in a pack, whatever their pixels (that rule alone left two groups);
/// - a flat image ([`NearHash::is_flat`]) is left out: with no gradient,
///   only the colour gate would be deciding, and two swatches of similar
///   colour are not "the same drawing" (with it, the pack reports none).
pub fn near_duplicate_groups<'a>(db: &AssetDB, pixels: impl Fn(&str, usize) -> Option<&'a DecodedImage>) -> Vec<Vec<NearEntry>> {
    // (entry, fingerprint, is a ground tile)
    let mut entries: Vec<(NearEntry, NearHash, bool)> = Vec::new();
    for uid in db.uids_in_order() {
        let Some(slot) = db.get(&uid) else { continue };
        for (idx, item) in db.items(&uid).iter().enumerate() {
            let Some(img) = pixels(&uid, idx) else { continue };
            let baked = render_item(img, &item.transform, slot.family.size(), slot.family.opaque());
            if let Some(h) = near_hash(&baked).filter(|h| !h.is_flat()) {
                entries.push((NearEntry { uid: uid.clone(), idx, item_hash: item.hash.clone() }, h, slot.family.is_texture()));
            }
        }
    }
    // Union-find over every pair; n is a pack's variant count (hundreds at
    // most), so the O(n^2) pair scan is a few hundred thousand XORs.
    let n = entries.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn root(p: &mut [usize], mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    for i in 0..n {
        for j in i + 1..n {
            if entries[i].2 == entries[j].2 && entries[i].1.is_near(&entries[j].1) {
                let (a, b) = (root(&mut parent, i), root(&mut parent, j));
                if a != b {
                    // Keep the earlier entry as the root so groups come out in
                    // store order of their first member.
                    parent[a.max(b)] = a.min(b);
                }
            }
        }
    }
    let mut groups: Vec<(usize, Vec<NearEntry>)> = Vec::new();
    for i in 0..n {
        let r = root(&mut parent, i);
        match groups.iter_mut().find(|(k, _)| *k == r) {
            Some((_, v)) => v.push(entries[i].0.clone()),
            None => groups.push((r, vec![entries[i].0.clone()])),
        }
    }
    groups
        .into_iter()
        .map(|(_, v)| v)
        .filter(|v| v.len() > 1 && v.iter().any(|e| e.item_hash != v[0].item_hash))
        .collect()
}

/// AP-6's acceptance fixture and the finder's unit behaviour.
#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic 32-bit LCG (Numerical Recipes constants), so the
    /// fixture is the same on every run and every machine.
    struct Lcg(u32);
    impl Lcg {
        fn next(&mut self) -> f64 {
            self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            f64::from(self.0 >> 8) / f64::from(1u32 << 24)
        }
    }

    /// Sprite `k` of the AP-6 fixture: a 256² RGBA drawing with its own
    /// silhouette (shape `k % 10`, one of ten: ellipse, triangle, diamond,
    /// cross, ring, crescent, hourglass, L, five-point star, half-disc; sized
    /// and placed by `k`) filled with its own coarse 6 × 6 value-noise
    /// shading in its own base colour, on transparency. Ten different
    /// drawings, as the bar's "10 distinct sprites" asks. Built for the test:
    /// no third-party art.
    fn sprite(k: u32) -> DecodedImage {
        sprite_shaped(k, k % 10)
    }

    /// [`sprite`] with the silhouette chosen separately, so a test can hold
    /// the shape fixed and vary only the shading and colour.
    fn sprite_shaped(k: u32, shape: u32) -> DecodedImage {
        let mut rng = Lcg(0x9E37_79B9 ^ k.wrapping_mul(2_654_435_761));
        let lattice: Vec<f64> = (0..49).map(|_| rng.next()).collect();
        let base = [60.0 + 150.0 * rng.next(), 60.0 + 150.0 * rng.next(), 60.0 + 150.0 * rng.next()];
        let (cx, cy) = (100.0 + 56.0 * rng.next(), 100.0 + 56.0 * rng.next());
        let (rx, ry) = (60.0 + 50.0 * rng.next(), 60.0 + 50.0 * rng.next());
        let mut px = vec![0u8; 256 * 256 * 4];
        for y in 0..256 {
            for x in 0..256 {
                let (dx, dy) = ((x as f64 - cx) / rx, (y as f64 - cy) / ry);
                let inside = match shape {
                    0 => dx * dx + dy * dy <= 1.0,
                    1 => dy >= -1.0 && dy <= 1.0 && dx.abs() <= (dy + 1.0) / 2.0,
                    2 => dx.abs() + dy.abs() <= 1.0,
                    3 => (dx.abs() <= 0.3 && dy.abs() <= 1.0) || (dy.abs() <= 0.3 && dx.abs() <= 1.0),
                    4 => (0.3..=1.0).contains(&(dx * dx + dy * dy)),
                    5 => dx * dx + dy * dy <= 1.0 && (dx - 0.45).powi(2) + dy * dy > 0.5,
                    6 => dx.abs() <= dy.abs() && dy.abs() <= 1.0,
                    7 => dx.abs() <= 1.0 && dy.abs() <= 1.0 && (dx <= -0.4 || dy >= 0.4),
                    8 => {
                        let (r, t) = ((dx * dx + dy * dy).sqrt(), dy.atan2(dx));
                        r <= 0.55 + 0.45 * (2.5 * t).cos().max(0.0)
                    }
                    _ => dx * dx + dy * dy <= 1.0 && dy >= 0.0,
                };
                if !inside {
                    continue;
                }
                // Bilinear value noise over a 7 × 7 lattice spanning the canvas.
                let (fx, fy) = (x as f64 / 255.0 * 6.0, y as f64 / 255.0 * 6.0);
                let (ix, iy) = ((fx as usize).min(5), (fy as usize).min(5));
                let (tx, ty) = (fx - ix as f64, fy - iy as f64);
                let l = |i: usize, j: usize| lattice[j * 7 + i];
                let v = (l(ix, iy) * (1.0 - tx) + l(ix + 1, iy) * tx) * (1.0 - ty)
                    + (l(ix, iy + 1) * (1.0 - tx) + l(ix + 1, iy + 1) * tx) * ty;
                let shade = 0.45 + 0.55 * v;
                let o = (y * 256 + x) * 4;
                for c in 0..3 {
                    px[o + c] = (base[c] * shade).round().clamp(0.0, 255.0) as u8;
                }
                px[o + 3] = 255;
            }
        }
        DecodedImage::new(256, 256, px).unwrap()
    }

    fn flip(img: &DecodedImage) -> DecodedImage {
        let (w, h) = (img.w as usize, img.h as usize);
        let mut out = vec![0u8; img.rgba.len()];
        for y in 0..h {
            for x in 0..w {
                let (s, d) = ((y * w + x) * 4, (y * w + (w - 1 - x)) * 4);
                out[d..d + 4].copy_from_slice(&img.rgba[s..s + 4]);
            }
        }
        DecodedImage::new(img.w, img.h, out).unwrap()
    }

    fn tint(img: &DecodedImage, k: f64) -> DecodedImage {
        let mut out = img.clone();
        for p in out.rgba.chunks_exact_mut(4) {
            for c in 0..3 {
                p[c] = (f64::from(p[c]) * k).round().clamp(0.0, 255.0) as u8;
            }
        }
        out
    }

    /// One pixel right and one down on the same canvas (the bottom row and
    /// right column fall off; the top row and left column become clear).
    fn shift(img: &DecodedImage) -> DecodedImage {
        let (w, h) = (img.w as usize, img.h as usize);
        let mut out = vec![0u8; img.rgba.len()];
        for y in 1..h {
            for x in 1..w {
                let (s, d) = (((y - 1) * w + x - 1) * 4, (y * w + x) * 4);
                out[d..d + 4].copy_from_slice(&img.rgba[s..s + 4]);
            }
        }
        DecodedImage::new(img.w, img.h, out).unwrap()
    }

    /// The five forms of sprite `k` the bar names: as drawn, mirrored,
    /// tinted +10 % and −10 %, and shifted by 1 px.
    fn forms(k: u32) -> Vec<DecodedImage> {
        let s = sprite(k);
        vec![flip(&s), tint(&s, 1.1), tint(&s, 0.9), shift(&s), s]
    }

    /// Protects: AP-6's acceptance bar (proposal §6). On 10 sprites ×
    /// {flip, ±10 % tint, 1 px shift}, every pair of forms of one sprite is
    /// near (Hamming ≤ 10, colour gate passed), and no pair among the 10
    /// distinct sprites is. Prints the worst same-sprite distance and the
    /// closest distinct pair by hash alone, which are the measured margins
    /// (see [`SILHOUETTE_WEIGHT`]: the hash-alone margin is thin, and the
    /// colour gate is part of what keeps distinct sprites apart).
    #[test]
    fn ap6_bar_flip_tint_shift_group_and_distinct_sprites_do_not() {
        let hashes: Vec<Vec<NearHash>> = (0..10).map(|k| forms(k).iter().map(|f| near_hash(f).unwrap()).collect()).collect();
        let mut worst_same = 0;
        for set in &hashes {
            for i in 0..set.len() {
                for j in i + 1..set.len() {
                    worst_same = worst_same.max(set[i].distance(&set[j]));
                    assert!(set[i].is_near(&set[j]), "forms {i} and {j}: distance {}", set[i].distance(&set[j]));
                }
            }
        }
        // `forms` puts the sprite as drawn last.
        let originals: Vec<NearHash> = hashes.iter().map(|set| set[4]).collect();
        let mut closest = 64;
        for a in 0..10 {
            for b in a + 1..10 {
                closest = closest.min(originals[a].distance(&originals[b]));
                assert!(!originals[a].is_near(&originals[b]), "sprites {a} and {b} grouped at distance {}", originals[a].distance(&originals[b]));
            }
        }
        println!("AP-6 fixture: worst same-sprite distance {worst_same}, closest distinct pair {closest} (bar: <= {MAX_HAMMING} / > {MAX_HAMMING})");
        assert!(worst_same <= MAX_HAMMING);
    }

    /// Protects (and discloses) what the finder does with one outline drawn
    /// twice: the outline carries much of a 9 × 8 hash, so two shadings of
    /// one shape in one colour may come out near (the "same tree, slightly
    /// different" case the proposal's rank 6 is for), while a green and a
    /// brown drawing of that shape are kept apart by the colour gate.
    #[test]
    fn one_outline_two_shadings_is_kept_apart_by_colour() {
        let (a, b) = (sprite_shaped(11, 0), sprite_shaped(12, 0));
        let recolour = |img: &DecodedImage, to: [f64; 3]| {
            let mut out = img.clone();
            for p in out.rgba.chunks_exact_mut(4).filter(|p| p[3] > 0) {
                let l = (0.299 * f64::from(p[0]) + 0.587 * f64::from(p[1]) + 0.114 * f64::from(p[2])) / 128.0;
                for c in 0..3 {
                    p[c] = (to[c] * l).round().clamp(0.0, 255.0) as u8;
                }
            }
            out
        };
        let (ga, gb, bb) = (recolour(&a, [70.0, 140.0, 60.0]), recolour(&b, [70.0, 140.0, 60.0]), recolour(&b, [150.0, 90.0, 50.0]));
        let (hga, hgb, hbb) = (near_hash(&ga).unwrap(), near_hash(&gb).unwrap(), near_hash(&bb).unwrap());
        println!("one outline, two shadings: green/green distance {} (near: {}), green/brown distance {}", hga.distance(&hgb), hga.is_near(&hgb), hga.distance(&hbb));
        assert!(!hga.is_near(&hbb), "a green and a brown drawing are not near-duplicates");
    }

    /// Protects: flat black line art (luma 0 everywhere) still hashes its
    /// outline through the coverage term, so two different line drawings are
    /// not grouped, and one is near its own mirror.
    #[test]
    fn black_line_art_keeps_its_outline() {
        let ink = |shape: u32| {
            let mut s = sprite_shaped(1, shape);
            for p in s.rgba.chunks_exact_mut(4) {
                p[0] = 0;
                p[1] = 0;
                p[2] = 0;
            }
            s
        };
        let (ring, star) = (near_hash(&ink(4)).unwrap(), near_hash(&ink(8)).unwrap());
        assert_ne!(ring.hash, 0, "the outline sets bits even with no shading");
        assert!(!ring.is_near(&star), "a black ring and a black star differ: distance {}", ring.distance(&star));
        assert!(near_hash(&flip(&ink(7))).unwrap().is_near(&near_hash(&ink(7)).unwrap()));
    }

    /// Protects: the colour gate. Two flat tiles of different colours hash
    /// identically (zero: no gradient) yet are not near; two of nearly the
    /// same colour are.
    #[test]
    fn flat_tiles_group_by_colour_not_by_hash() {
        let flat = |c: [u8; 3]| DecodedImage::new(64, 64, [c[0], c[1], c[2], 255].repeat(64 * 64)).unwrap();
        let (r, b, r2) = (near_hash(&flat([200, 40, 40])).unwrap(), near_hash(&flat([40, 40, 200])).unwrap(), near_hash(&flat([210, 50, 35])).unwrap());
        assert_eq!((r.hash, b.hash), (0, 0));
        assert_eq!(r.distance(&b), 0);
        assert!(!r.is_near(&b), "different colours are not near-duplicates");
        assert!(r.is_near(&r2), "a 10-level nudge is");
    }

    /// Protects: the dead band. A flat region's float rounding sets no bit,
    /// and a real one-level-per-column ramp sets every bit (a dHash with
    /// `>=` or no band would differ on one of the two).
    #[test]
    fn dead_band_ignores_rounding_but_not_a_real_ramp() {
        let mut px = Vec::new();
        for _y in 0..72 {
            for x in 0..90u32 {
                let v = (x * 2) as u8;
                px.extend_from_slice(&[v, v, v, 255]);
            }
        }
        let ramp = near_hash(&DecodedImage::new(90, 72, px).unwrap()).unwrap();
        assert_eq!(ramp.hash, u64::MAX, "every column step is ~20 levels: all 64 bits set");
        assert_eq!(ramp.mirrored, 0, "mirrored, it only darkens: no bit set");
        let odd = DecodedImage::new(37, 29, [90, 90, 90, 255].repeat(37 * 29)).unwrap();
        assert_eq!(near_hash(&odd).unwrap().hash, 0, "a flat tile of awkward size sets no bit");
    }

    /// Protects: a blank image has no fingerprint, and the content box makes
    /// padding irrelevant (the same drawing on a larger clear canvas hashes
    /// the same).
    #[test]
    fn blank_has_none_and_padding_does_not_matter() {
        assert!(near_hash(&DecodedImage::new(8, 8, vec![0; 256]).unwrap()).is_none());
        let s = sprite(3);
        let mut big = vec![0u8; 300 * 300 * 4];
        for y in 0..256 {
            for x in 0..256 {
                let (a, b) = ((y * 256 + x) * 4, ((y + 20) * 300 + x + 30) * 4);
                big[b..b + 4].copy_from_slice(&s.rgba[a..a + 4]);
            }
        }
        let padded = DecodedImage::new(300, 300, big).unwrap();
        assert_eq!(near_hash(&s).unwrap().hash, near_hash(&padded).unwrap().hash);
    }

    /// Protects: the library-level grouping. Forms of one sprite stored in
    /// two slots form one group in store order; a distinct sprite stays out;
    /// a pair of exact copies alone is not reported (the exact-duplicate
    /// finding owns it), but an exact copy inside a near group is kept.
    #[test]
    fn groups_in_store_order_and_skip_exact_only_groups() {
        let mut db = AssetDB::new();
        let imgs: Vec<(String, DecodedImage)> = vec![
            ("icons:mountain".into(), sprite(1)),
            ("icons:mountain".into(), sprite(2)),
            ("icons:hill".into(), flip(&sprite(1))),
            ("icons:hill".into(), sprite(5)),
            ("icons:hill".into(), sprite(5)),
        ];
        let mut store: std::collections::HashMap<(String, usize), DecodedImage> = Default::default();
        for (uid, img) in &imgs {
            let n = db.items(uid).len();
            db.add_item(uid, crate::LibraryItem::new(format!("{uid}#{n}"), crate::item_hash(img)));
            store.insert((uid.clone(), n), img.clone());
        }
        let groups = near_duplicate_groups(&db, |u, i| store.get(&(u.to_string(), i)));
        let flat: Vec<Vec<(String, usize)>> = groups.iter().map(|g| g.iter().map(|e| (e.uid.clone(), e.idx)).collect()).collect();
        let order = db.uids_in_order();
        let (m, h) = (order.iter().position(|u| u == "icons:mountain").unwrap(), order.iter().position(|u| u == "icons:hill").unwrap());
        let expect = if m < h {
            vec![("icons:mountain".to_string(), 0), ("icons:hill".to_string(), 0)]
        } else {
            vec![("icons:hill".to_string(), 0), ("icons:mountain".to_string(), 0)]
        };
        assert_eq!(flat, vec![expect], "one group: sprite 1 and its mirror; sprite 5's exact pair is not reported here");
    }

    /// Protects: the two grouping rules found on the reference pack. A
    /// sprite and a ground tile carrying the same drawing are never grouped
    /// (the sprite/ground split), and two flat swatches of nearly the same
    /// colour are left out (`NearHash::is_flat`), though `is_near` alone
    /// would call both pairs near.
    #[test]
    fn sprites_and_ground_tiles_never_group_and_flat_swatches_are_left_out() {
        let mut db = AssetDB::new();
        let opaque = {
            let mut s = sprite_shaped(4, 0);
            for p in s.rgba.chunks_exact_mut(4) {
                p[3] = 255;
            }
            s
        };
        let flat = |c: u8| DecodedImage::new(256, 256, [c, 100, 60, 255].repeat(256 * 256)).unwrap();
        let rows: Vec<(&str, DecodedImage)> =
            vec![("icons:hill", opaque.clone()), ("biomes:jungle", opaque.clone()), ("icons:shrub", flat(120)), ("icons:boulder", flat(126))];
        let mut store: std::collections::HashMap<(String, usize), DecodedImage> = Default::default();
        for (uid, img) in &rows {
            db.add_item(uid, crate::LibraryItem::new(*uid, format!("{uid}-distinct-hash")));
            store.insert((uid.to_string(), 0), img.clone());
        }
        // The pure comparison says near for both pairs ...
        let h = |i: usize| near_hash(&rows[i].1).unwrap();
        assert!(h(0).is_near(&h(1)) && h(2).is_near(&h(3)) && h(2).is_flat());
        // ... and the grouping keeps neither.
        let groups = near_duplicate_groups(&db, |u, i| store.get(&(u.to_string(), i)));
        assert!(groups.is_empty(), "{groups:?}");
    }
}
