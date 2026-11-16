//! **ZR-0 of `ZOOM_REFINEMENT_SCOPE.md`** (owner Ruling FH, 2026-11-16: "start
//! with ZR-0"): the measure-first harness for real zoom refinement. It
//! prototypes the scope's §4.1 per-tile chain **inside this test file only**
//! and measures it against the go/no-go bar of §5 ZR-0. No product code is
//! touched; nothing here is called by the app.
//!
//! What the prototype chain is, per tile at level `k` (texel `L / 2^k` km,
//! `L` the world's km per cell), from its parent at `k - 1`:
//!
//! 1. **Parent window** (in [`refine_tile`]): the parent mosaic's *stored*
//!    (quantised) heights and flows under the tile plus a margin.
//! 2. **Non-overshooting cubic x2** ([`up2_at`]): Catmull-Rom at the fixed child
//!    offsets `x/2 - 1/4`, clamped to the two bracketing parent taps, so it
//!    invents no pit and no spur (§4.1 step 2).
//! 3. **One FG kilometre-law band** ([`band_delta`]): the single FG lattice
//!    octave whose wavelength lies in `[2, 4)` child texels, with the shipped
//!    amplitude (`km_detail_delta` over the coarse relief and the biome
//!    roughness), from an **integer-hash** value noise in world km and a
//!    rational (Pythagorean) rotation, so no transcendental `libm` call is on
//!    the path (§6 R3).
//! 4. **Drainage over a 16-texel halo**: an epsilon priority-flood
//!    ([`fill_eps`]), then `cartalith_hydrology::tile::tile_flow` with the
//!    parent window as its "coarse" grid (refine 2), i.e. the parent's stored
//!    flow as the inflow boundary.
//! 5. **Burn in physical units** ([`burn`]): a channel is carved at the one
//!    level where its valley width `W = VALLEY_W_K * sqrt(A km^2)` first
//!    spans 2-4 texels; depth `0.5 * W * slope`, a quadratic cross-section.
//! 6. **Erosion, both Q1 candidates**: EF-3's `tile_erode` (fill, MFD area,
//!    implicit stream power, deposition) and a Braun-Willett implicit solve
//!    over step 4's own D8 routing ([`bw_erode`]), same coefficient
//!    expression as the world kernel's `cc`; then one talus pass ([`talus`]).
//! 7. **Edge taper and quantise** ([`refine_tile`]): the non-local change is
//!    multiplied by `1 - conform_weight(d, 16)` so it is exactly zero on the
//!    tile edge; flow is blended the same way toward the parent's upsampled
//!    flow; height is quantised to a **global** step [`Q_STEP`].
//!
//! What it measures (see `zr0_bars`, `zr0_timing`, `zr0_determinism`): the
//! refined 40 075 km world against the shipped FG noise tile and against the
//! native 1 200 km and 200 km maps, each rendered by the shipped tile
//! colouriser (`render::render_biome_tile_rgba_water`) at the same km per
//! pixel; land luma gradient and `bind_r`; seam bit-identity and channel
//! continuity; thread-count determinism; per-step cost per tile.
//!
//! **What it must never do:** be called from product code, or be read as the
//! ZR-1 implementation. Every constant below that is not the shipped value of
//! a named symbol is a labelled judgement of this prototype.
//!
//! ```text
//! ZR_OUT=<dir> cargo test --release -p cartalith-godot --test zr0_refine -- --ignored --nocapture zr0_bars
//! cargo test --release -p cartalith-godot --test zr0_refine -- --ignored --nocapture zr0_timing      (alone)
//! cargo test --release -p cartalith-godot --test zr0_refine -- --ignored --nocapture zr0_determinism
//! ```
//!
//! Protects: nothing by itself (a measurement harness, `#[ignore]`d). It
//! asserts only that the worlds are not flat, that every mosaic and frame is
//! non-empty, and that every refined tile keeps its edge on the base.

#[path = "../src/render.rs"]
#[allow(dead_code)]
mod render;
#[path = "../src/params.rs"]
#[allow(dead_code)]
mod params;

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;
use std::time::Instant;

use cartalith_engine::bake::pyramid_tile_padded_rough;
use cartalith_engine::elevation::conform_weight;
use cartalith_erosion::tile::tile_erode;
use cartalith_erosion::StreamPowerParams;
use cartalith_hydrology::tile::{tile_flow, TilePlacement};
use cartalith_spatial::pyramid::{pyramid_dims, pyramid_level_for_zoom, pyramid_tile_bounds, ChunkId};
use cartalith_spatial::{tile_dims, Region};
use cartalith_terrain::amplify::{km_detail_delta, km_local_relief, KmZoomDetail, ZOOM_DETAIL_REF_CELL_KM};
use rayon::prelude::*;
use render::{GridPrecompute, QualityTier, RenderCtx, TerrainAppearance, TileBounds, TileFields};

/// Interior texels per refined tile side: `lod_bridge::TILE_PX`, restated
/// (the bridge cannot be `#[path]`-included; `zoom_detail_ladder.rs` restates
/// it for the same reason).
const TILE: usize = 256;
/// The drainage halo, texels (§4.1 step 4's default `H`).
const HALO: usize = 16;
/// Padded tile side: interior plus a halo each side.
const PT: usize = TILE + 2 * HALO;
/// Parent texels under the padded tile (refine 2 is exact).
const PP: usize = PT / 2;
/// The parent window side: `PP` plus two texels each side, one for the cubic's
/// outer tap and one for `tile_flow`'s ring of donor cells.
const PW: usize = PP + 4;
/// The taper band, texels: `elevation::CONFORM_BAND_TEXELS`' value (§4.1
/// step 7's default `B`).
const BAND: f64 = 16.0;
/// The global height quantum, normalised height: `2^-20` (about 6.6 mm at the
/// app's 4 000 m peak). **Global, not per tile**, so two neighbours quantise
/// an equal base to equal bits; the scope's per-tile offset + u16 step would
/// need its offset to be a multiple of this step to keep that (a ZR-2 note).
const Q_STEP: f64 = 1.0 / 1_048_576.0;
/// Valley width per square root of drained area, km per km (labelled
/// judgement: a 100 km^2 basin's valley is 1 km wide, a 10 000 km^2 basin's
/// 10 km; the exponent 0.5 is Leopold & Maddock 1953's `W ~ Q^0.5` with `Q`
/// proportional to area).
const VALLEY_W_K: f64 = 0.1;
/// Valley side slope as a fraction of the local terrain slope (labelled
/// judgement): valleys are V-shaped where the land is steep and vanish on
/// flats, and the carved slope is scale-free in the renderer's units.
const VALLEY_SIDE: f64 = 0.5;
/// Braun-Willett passes (the scope's §3 row g2).
const BW_PASSES: usize = 3;
/// EF-3 iterations, matched to [`BW_PASSES`] (the scope's §3 row g).
const EF3_ITERS: i32 = 3;
/// `tan(35 deg)`, a literal so the talus threshold calls no `libm` (35 deg is
/// a labelled judgement for loose regolith; `critical_talus`'s own formula).
const TAN_35: f64 = 0.700_207_538_209_710_1;
/// The priority-flood epsilon, normalised height per texel (labelled: small
/// enough that a 200-texel lake rises 2e-4, under the global quantum x 200).
const FILL_EPS: f32 = 1e-6;
/// Frame side for every comparison frame, screen pixels.
const FRAME: usize = 768;
/// The two km-per-pixel the bar names (scope §5 ZR-0; FG findings §4 B4).
const KMPX_1200: f64 = 0.471;
const KMPX_200: f64 = 0.078;

// ---------------------------------------------------------------------------
// Worlds

/// One generated world and the render context it is drawn with.
struct World {
    ws: cartalith_engine::WorldState,
    p: cartalith_engine::WorldParams,
    pre: GridPrecompute,
    a: TerrainAppearance,
    /// `compute_flow` over the **final** field (scope §6 R5: the stored
    /// `flow_discharge` disagrees with the post-carve field).
    flow: Vec<f32>,
}

impl World {
    fn generate(seed: i32, gw: usize, gh: usize, width_km: f64, world: bool) -> World {
        let mut p = params::defaults();
        p.gw = gw;
        p.gh = gh;
        p.tect.seed = seed;
        p.map_width_km = width_km;
        p.sea_level = 0.42;
        p.world = world;
        let ws = cartalith_engine::generate_terrain(&p);
        let lo = ws.field.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = ws.field.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(hi - lo > 0.2, "world {width_km} km is nearly flat");
        let a = TerrainAppearance::for_tier(QualityTier::Quality).with_look(render::LOOK_VIBRANT);
        let pre = GridPrecompute::build(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), gw, gh, ws.sea_level as f64, world, &a, Some(width_km));
        let flow = cartalith_hydrology::compute_flow(gw, gh, &ws.field, Some(&ws.rainfall), true, world);
        World { ws, p, pre, a, flow }
    }
    fn ctx(&self) -> RenderCtx<'_> {
        let ws = &self.ws;
        RenderCtx::from_precomputed(&ws.field, &ws.temperature, &ws.rainfall, Some(&ws.flow_discharge), self.p.gw, self.p.gh, ws.sea_level as f64, self.p.world, 55.0, 5.0, self.a.clone(), &self.pre).unwrap()
    }
    fn l_km(&self) -> f64 {
        cartalith_spatial::cell_km(self.p.map_width_km, self.p.gw)
    }
}

/// FNV-1a over f32 bit patterns: the determinism digest.
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

/// Clamped bilinear sample in sample coordinates (cell centre = integer), the
/// amplifier's own `samp` convention, restated (it is private there).
fn samp(src: &[f32], w: usize, h: usize, fx: f64, fy: f64) -> f64 {
    let fx = fx.clamp(0.0, w as f64 - 1.0);
    let fy = fy.clamp(0.0, h as f64 - 1.0);
    let (x0, y0) = (fx as usize, fy as usize);
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
    let (a, b) = (src[y0 * w + x0] as f64, src[y0 * w + x1] as f64);
    let (c, d) = (src[y1 * w + x0] as f64, src[y1 * w + x1] as f64);
    (a * (1.0 - tx) + b * tx) * (1.0 - ty) + (c * (1.0 - tx) + d * tx) * ty
}

/// Ruling O's sea guard (`amplify::clamp_toward_sea`, `pub(crate)` there),
/// restated: an excursion toward sea level is capped at half the headroom.
fn clamp_toward_sea(base: f64, delta: f64, sea: f64) -> f64 {
    let half = (base - sea) * 0.5;
    if base >= sea { if delta < -half { -half } else { delta } } else if delta > -half { -half } else { delta }
}

// ---------------------------------------------------------------------------
// The coarse world as the chain reads it

/// Everything a refined tile reads from the base world (§4.2's "cone").
struct Coarse<'a> {
    field: &'a [f32],
    flow: &'a [f32],
    stress: &'a [f32],
    resist: &'a [f32],
    rain: &'a [f32],
    temp: &'a [f32],
    gw: usize,
    gh: usize,
    sea: f64,
    l_km: f64,
    km: KmZoomDetail,
    seed: i32,
    sp: StreamPowerParams,
}

impl<'a> Coarse<'a> {
    fn new(w: &'a World, ctx: &RenderCtx) -> Coarse<'a> {
        let opts = render::tile_amplify_opts(ctx, w.p.tect.seed, cartalith_terrain::amplify::z_base_for_tile_size(TILE));
        let km = opts.km.expect("the shipped kilometre law is on for this world");
        let p = &w.p;
        // The world's light pass (`ef3_tile_erosion.rs`' `light_params`), with
        // **uplift off**: a refinement pass incises, it does not uplift, and the
        // kernel normalises stress per tile, which would make the uplift itself
        // tile-dependent. Same for both candidates.
        let sp = StreamPowerParams {
            k: p.stream.k,
            uplift: 0.0,
            deposit: p.stream.deposit,
            climate_k: p.stream.climate_k,
            iters: EF3_ITERS,
            resist: p.tect.resist,
            g: p.planet.g,
            world: false,
            sea: w.ws.sea_level,
        };
        Coarse {
            field: &w.ws.field,
            flow: &w.flow,
            stress: &w.ws.stress_field,
            resist: &w.ws.resistance_field,
            rain: &w.ws.rainfall,
            temp: &w.ws.temperature,
            gw: p.gw,
            gh: p.gh,
            sea: w.ws.sea_level as f64,
            l_km: w.l_km(),
            km,
            seed: p.tect.seed,
            sp,
        }
    }
    /// Level-`k` global texel centre `X` in coarse sample coordinates.
    fn sx(&self, k: u32, x: i64) -> f64 {
        (x as f64 + 0.5) / (1u64 << k) as f64 - 0.5
    }
}

// ---------------------------------------------------------------------------
// Mosaics: one level's stitched tiles

/// One level's stored result over a texel rectangle: quantised height and flow
/// in that level's texel units (drained area = `flow * t^2` km^2).
#[derive(Clone)]
struct Mosaic {
    k: u32,
    x0: i64,
    y0: i64,
    w: usize,
    h: usize,
    hgt: Vec<f32>,
    flow: Vec<f32>,
}

impl Mosaic {
    fn base(c: &Coarse) -> Mosaic {
        Mosaic { k: 0, x0: 0, y0: 0, w: c.gw, h: c.gh, hgt: c.field.to_vec(), flow: c.flow.to_vec() }
    }
    /// Index of global texel `(x, y)`. The base level (`k == 0`, the world
    /// grid itself) wraps in `x` and clamps in `y`, as the world does; a
    /// refined level must contain the texel (a planning error otherwise).
    fn idx(&self, x: i64, y: i64) -> usize {
        if self.k == 0 {
            let lx = x.rem_euclid(self.w as i64) as usize;
            let ly = y.clamp(0, self.h as i64 - 1) as usize;
            return ly * self.w + lx;
        }
        let (lx, ly) = (x - self.x0, y - self.y0);
        assert!(lx >= 0 && ly >= 0 && (lx as usize) < self.w && (ly as usize) < self.h, "level {} texel ({x},{y}) outside the mosaic", self.k);
        ly as usize * self.w + lx as usize
    }
}

/// A rectangle of tiles at one level: tile columns `c0..c1`, rows `r0..r1`.
#[derive(Clone, Copy, Debug)]
struct TileRect {
    c0: i64,
    r0: i64,
    c1: i64,
    r1: i64,
}

impl TileRect {
    fn covering(x0: i64, y0: i64, x1: i64, y1: i64) -> TileRect {
        let t = TILE as i64;
        TileRect { c0: x0.div_euclid(t), r0: y0.div_euclid(t), c1: (x1 - 1).div_euclid(t) + 1, r1: (y1 - 1).div_euclid(t) + 1 }
    }
    fn union(self, o: TileRect) -> TileRect {
        TileRect { c0: self.c0.min(o.c0), r0: self.r0.min(o.r0), c1: self.c1.max(o.c1), r1: self.r1.max(o.r1) }
    }
    fn count(&self) -> usize {
        ((self.c1 - self.c0) * (self.r1 - self.r0)) as usize
    }
    /// The parent-level texel rectangle this tile rectangle reads (§4.1 step 1).
    fn parent_texels(&self) -> (i64, i64, i64, i64) {
        let t = TILE as i64;
        let p0x = (self.c0 * t - HALO as i64) / 2 - 2;
        let p0y = (self.r0 * t - HALO as i64) / 2 - 2;
        let p1x = (self.c1 * t + HALO as i64) / 2 + 2;
        let p1y = (self.r1 * t + HALO as i64) / 2 + 2;
        (p0x, p0y, p1x, p1y)
    }
}

// ---------------------------------------------------------------------------
// Step 2: the non-overshooting cubic

/// Catmull-Rom weights at `t = 0.75` (child `2i`, between parent `i-1` and
/// `i`) and `t = 0.25` (child `2i+1`, between `i` and `i+1`), exact binary
/// fractions computed by hand from `((-t^3+2t^2-t), (3t^3-5t^2+2),
/// (-3t^3+4t^2+t), (t^3-t^2)) / 2`.
const CR75: [f64; 4] = [-0.0234375, 0.2265625, 0.8671875, -0.0703125];
const CR25: [f64; 4] = [-0.0703125, 0.8671875, 0.2265625, -0.0234375];

/// One child sample along an axis from four parent taps, clamped to the
/// bracketing pair `p1, p2` (no new extremum).
#[inline]
fn cubic(p: [f64; 4], w: &[f64; 4]) -> f64 {
    let v = p[0] * w[0] + p[1] * w[1] + p[2] * w[2] + p[3] * w[3];
    let (lo, hi) = if p[1] < p[2] { (p[1], p[2]) } else { (p[2], p[1]) };
    v.clamp(lo, hi)
}

/// The taps and weights of child texel `c` (global, along one axis): returns
/// the first parent index of the four taps and the weight set.
#[inline]
fn taps(c: i64) -> (i64, &'static [f64; 4]) {
    let i = c.div_euclid(2);
    if c.rem_euclid(2) == 0 { (i - 2, &CR75) } else { (i - 1, &CR25) }
}

/// `src` (a parent mosaic's height or flow) upsampled x2 at child global texel
/// `(x, y)`: x pass on the four tap rows, then the y pass, both clamped.
/// A pure function of the parent's stored values and the global index, so two
/// tiles computing the same texel get the same bits (§4.2 "Seams", item 1).
fn up2_at(m: &Mosaic, src: &[f32], x: i64, y: i64) -> f64 {
    let (px, wx) = taps(x);
    let (py, wy) = taps(y);
    let mut col = [0.0f64; 4];
    for (r, c) in col.iter_mut().enumerate() {
        let yy = py + r as i64;
        let row = [0, 1, 2, 3].map(|q| src[m.idx(px + q, yy)] as f64);
        *c = cubic(row, wx);
    }
    cubic(col, wy)
}

// ---------------------------------------------------------------------------
// Step 3: one FG band from an integer hash

/// A 64-bit integer hash of a lattice corner (splitmix64's finaliser), mapped
/// to `[0, 1)`. Integer-only, so every platform agrees bit for bit.
#[inline]
fn hash01(ix: i64, iy: i64, s: u64) -> f64 {
    let mut z = (ix as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (iy as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ s;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 11) as f64 / (1u64 << 53) as f64
}

/// Value noise in `[-0.5, 0.5)` with a smoothstep blend, lattice in `u, v`.
#[inline]
fn vnoise_int(u: f64, v: f64, s: u64) -> f64 {
    let (fu, fv) = (u.floor(), v.floor());
    let (ix, iy) = (fu as i64, fv as i64);
    let (tx, ty) = (u - fu, v - fv);
    let (sx, sy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
    let a = hash01(ix, iy, s);
    let b = hash01(ix + 1, iy, s);
    let c = hash01(ix, iy + 1, s);
    let d = hash01(ix + 1, iy + 1, s);
    (a + (b - a) * sx) + ((c + (d - c) * sx) - (a + (b - a) * sx)) * sy - 0.5
}

/// Four rational rotations (Pythagorean triples), chosen by octave so no
/// octave keeps the axis-aligned lattice (the FG law rotates for the same
/// reason; these need no `sin`/`cos`).
const ROT: [(f64, f64); 4] = [(0.6, 0.8), (5.0 / 13.0, 12.0 / 13.0), (8.0 / 17.0, 15.0 / 17.0), (20.0 / 29.0, 21.0 / 29.0)];

/// The FG lattice index `j` whose wavelength `lam_ref * 2^-j` lies in
/// `[2t, 4t)` for texel `t` km, and that wavelength. Loops, no `log2`.
fn band_octave(t: f64) -> (i32, f64) {
    let lam_ref = ZOOM_DETAIL_REF_CELL_KM * 0.5;
    let mut j = 0i32;
    let lam = |j: i32| lam_ref * f64::powi(2.0, -j);
    while lam(j) >= 4.0 * t {
        j += 1;
    }
    while lam(j) < 2.0 * t {
        j -= 1;
    }
    (j, lam(j))
}

/// Step 3's excursion at child texel `(x, y)` of level `k` on base height
/// `base`: the band's noise times the shipped FG amplitude
/// (`km_detail_delta`: `strength * relief_gain * rho * max(R, floor)`, the
/// soft cap and the shore fade), times `(lam / L)^1` (the shipped octave gain
/// 0.5, `H = 1`), then Ruling O's sea guard. Water gets nothing (the FG law's
/// own hard cut).
fn band_delta(c: &Coarse, k: u32, x: i64, y: i64, base: f64, j: i32, lam: f64) -> f64 {
    if base < c.sea {
        return 0.0;
    }
    let (cx, cy) = (c.sx(k, x), c.sx(k, y));
    let (gx, gy) = (cx * c.l_km, cy * c.l_km);
    let (rc, rs) = ROT[j.rem_euclid(4) as usize];
    let (u, v) = ((gx * rc - gy * rs) / lam, (gx * rs + gy * rc) / lam);
    let s = (c.seed as u32 as u64) ^ ((j as i64 as u64).wrapping_mul(0x51_7CC1_B727_220A) ^ 0x2545_F491_4F6C_DD1D);
    let n = vnoise_int(u, v, s);
    let sum = n * (lam / c.l_km) * top_fade(lam, c.l_km);
    let (l, r) = (samp(c.field, c.gw, c.gh, cx - 1.0, cy), samp(c.field, c.gw, c.gh, cx + 1.0, cy));
    let (uu, dd) = (samp(c.field, c.gw, c.gh, cx, cy - 1.0), samp(c.field, c.gw, c.gh, cx, cy + 1.0));
    let cc = samp(c.field, c.gw, c.gh, cx, cy);
    let relief = km_local_relief((r - l) * 0.5, (dd - uu) * 0.5, (l + r + uu + dd) * 0.25 - cc);
    let rho = render::detail_roughness_at(c.temp, c.rain, c.gw, c.gh, cx, cy);
    clamp_toward_sea(base, km_detail_delta(sum, relief, rho, base, c.sea, &c.km), c.sea)
}

/// The FG law's **top fade** (`km_zoom_detail_sum`'s `smooth01(log2(L /
/// lam))`): 1 at `lam <= L/2`, 0 at `lam >= L`. The base grid already holds
/// every wavelength above one cell, so the first one or two refined levels
/// (whose 2-4 texel band is at or above `L`) add little or nothing; a first
/// run without it added octaves FG never draws. Its `log2` is a per-level
/// constant (one call per tile), not per texel; ZR-1 can tabulate it.
fn top_fade(lam: f64, l: f64) -> f64 {
    let t = (l / lam).log2().clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn quant(h: f64) -> f32 {
    ((h / Q_STEP).round() * Q_STEP) as f32
}

/// Steps 2+3, quantised: the **base** at a texel, the value every tile edge
/// must equal bit for bit.
fn base_at(c: &Coarse, parent: &Mosaic, k: u32, x: i64, y: i64, j: i32, lam: f64) -> f64 {
    // Rounded through `f32` exactly as `refine_tile` stores `up` and `base`,
    // so the quantised value is the same bits (a first run without this
    // rounding reported ~3 % of edge texels off the base: a harness bug, not a
    // seam).
    let b = up2_at(parent, &parent.hgt, x, y) as f32 as f64;
    (b + band_delta(c, k, x, y, b, j, lam)) as f32 as f64
}

// ---------------------------------------------------------------------------
// Step 4: fill and route

const D8: [(i64, i64, f64); 8] = [
    (-1, -1, std::f64::consts::SQRT_2),
    (0, -1, 1.0),
    (1, -1, std::f64::consts::SQRT_2),
    (-1, 0, 1.0),
    (1, 0, 1.0),
    (-1, 1, std::f64::consts::SQRT_2),
    (0, 1, 1.0),
    (1, 1, std::f64::consts::SQRT_2),
];

/// The D8 receiver and its distance, `hydrology::d8_receiver`'s rule restated
/// (it is `pub(crate)`): steepest `(h - h_n)/d`, first strictly greater wins,
/// scanned `dy` then `dx` ascending; `-1` for a pit.
fn d8(f: &[f32], w: usize, h: usize, i: usize) -> (i64, f64) {
    let (x, y) = ((i % w) as i64, (i / w) as i64);
    let hi = f[i] as f64;
    let (mut best, mut bd, mut dist) = (-1i64, 0.0f64, 1.0f64);
    for &(dx, dy, d) in &D8 {
        let (nx, ny) = (x + dx, y + dy);
        if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
            continue;
        }
        let j = (ny * w as i64 + nx) as usize;
        let drop = (hi - f[j] as f64) / d;
        if drop > bd {
            bd = drop;
            best = j as i64;
            dist = d;
        }
    }
    (best, dist)
}

#[inline]
fn on_ring(i: usize, w: usize, h: usize) -> bool {
    let (x, y) = (i % w, i / w);
    x == 0 || y == 0 || x == w - 1 || y == h - 1
}

/// An epsilon priority-flood (Barnes et al. 2014's `+eps` variant) seeded on
/// the outer ring, ties broken by index; returns the filled field and the pop
/// order, which is a receiver-before-donor order on the filled field.
fn fill_eps(src: &[f32], w: usize, h: usize) -> (Vec<f32>, Vec<u32>) {
    let n = w * h;
    let mut f = src.to_vec();
    let mut done = vec![false; n];
    let key = |v: f32| -> u32 {
        let b = v.to_bits();
        if b & 0x8000_0000 != 0 { !b } else { b | 0x8000_0000 }
    };
    let mut heap = BinaryHeap::new();
    for i in 0..n {
        if on_ring(i, w, h) {
            done[i] = true;
            heap.push(Reverse((key(f[i]), i as u32)));
        }
    }
    let mut order = Vec::with_capacity(n);
    while let Some(Reverse((_, i))) = heap.pop() {
        let i = i as usize;
        order.push(i as u32);
        let (x, y) = ((i % w) as i64, (i / w) as i64);
        for &(dx, dy, _) in &D8 {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                continue;
            }
            let j = (ny * w as i64 + nx) as usize;
            if done[j] {
                continue;
            }
            done[j] = true;
            if f[j] <= f[i] {
                f[j] = f[i] + FILL_EPS;
            }
            heap.push(Reverse((key(f[j]), j as u32)));
        }
    }
    (f, order)
}

// ---------------------------------------------------------------------------
// Step 5: burn

/// Burns every channel whose valley first spans 2-4 texels at this level.
/// `flow` in level texel units, `t` the texel in km. Depth is
/// `VALLEY_SIDE * W * slope` in normalised height (W in texels, slope per
/// texel from `base`), so the carved side slope is a fraction of the
/// terrain's; quadratic cross-section, max-combined; land never goes below
/// sea level (the renderer would draw it as sea). Returns texels burned.
fn burn(h: &mut [f32], base: &[f32], flow: &[f32], w: usize, hh: usize, t: f64, sea: f64) -> usize {
    let n = w * hh;
    let mut cut = vec![0f64; n];
    let mut count = 0;
    for i in 0..n {
        let a_km2 = flow[i] as f64 * t * t;
        let ratio = VALLEY_W_K * a_km2.sqrt() / t;
        if !(2.0..4.0).contains(&ratio) {
            continue;
        }
        let (x, y) = (i % w, i / w);
        if x == 0 || y == 0 || x == w - 1 || y == hh - 1 {
            continue;
        }
        let gx = (base[i + 1] as f64 - base[i - 1] as f64) * 0.5;
        let gy = (base[i + w] as f64 - base[i - w] as f64) * 0.5;
        let s = (gx * gx + gy * gy).sqrt();
        let d = VALLEY_SIDE * ratio * s;
        if d <= 0.0 {
            continue;
        }
        count += 1;
        let hw = ratio * 0.5;
        let r = hw.ceil() as i64;
        for oy in -r..=r {
            for ox in -r..=r {
                let (nx, ny) = (x as i64 + ox, y as i64 + oy);
                if nx < 0 || ny < 0 || nx >= w as i64 || ny >= hh as i64 {
                    continue;
                }
                let dist = ((ox * ox + oy * oy) as f64).sqrt();
                if dist >= hw {
                    continue;
                }
                let q = dist / hw;
                let j = ny as usize * w + nx as usize;
                cut[j] = cut[j].max(d * (1.0 - q * q));
            }
        }
    }
    for i in 0..n {
        if cut[i] > 0.0 {
            let b = h[i] as f64;
            let v = b - cut[i];
            h[i] = if b >= sea { v.max(sea + 1e-4).min(b) } else { v } as f32;
        }
    }
    count
}

// ---------------------------------------------------------------------------
// Step 6: the two erosion candidates and the talus pass

/// Q1's light candidate: implicit stream power (Braun & Willett 2013) over
/// step 4's own D8 receivers and D8 area, `BW_PASSES` passes in the fill's
/// receiver-before-donor order, ring pinned. Coefficient: the world kernel's
/// own `cc` expression (`stream_power_routing`, non-rock arm) with EF-3's
/// resolution correction `k * refine`, `refine = 2^k`, and area in base-cell
/// units.
#[allow(clippy::too_many_arguments)]
fn bw_erode(h: &mut [f32], filled: &[f32], order: &[u32], flow_child: &[f32], resist: &[f32], rain: &[f32], w: usize, hh: usize, sp: &StreamPowerParams, k: u32) {
    let n = w * hh;
    let refine = (1u64 << k) as f64;
    let to_base = 1.0 / (refine * refine);
    let mut rcv = vec![-1i64; n];
    let mut cc = vec![0f64; n];
    for i in 0..n {
        if on_ring(i, w, hh) {
            continue;
        }
        let (r, d) = d8(filled, w, hh, i);
        if r < 0 {
            continue;
        }
        rcv[i] = r;
        let res = sp.resist * 0.7 * resist[i] as f64;
        let ki = sp.k * refine * sp.g * (1.0 - res).max(0.05) * (1.0 + sp.climate_k * 2.0 * rain[i] as f64);
        cc[i] = ki * (flow_child[i] as f64 * to_base).max(0.0).sqrt() / d;
    }
    for _ in 0..BW_PASSES {
        for &i in order {
            let i = i as usize;
            let r = rcv[i];
            if r < 0 {
                continue;
            }
            let c = cc[i];
            h[i] = ((h[i] as f64 + c * h[r as usize] as f64) / (1.0 + c)) as f32;
        }
    }
}

/// One talus pass at 35 deg (`critical_talus`'s formula, literal `tan`): a
/// texel whose drop to its lowest 8-neighbour exceeds the threshold over that
/// distance moves half the excess there. Two buffers, so order-free. Returns
/// texels moved.
fn talus(h: &mut [f32], w: usize, hh: usize, t_km: f64, sea: f64, peak_m: f64) -> usize {
    let thr = TAN_35 * t_km * 1000.0 * (1.0 - sea) / peak_m;
    let src = h.to_vec();
    let mut moved = 0;
    for i in 0..w * hh {
        if on_ring(i, w, hh) {
            continue;
        }
        let (r, d) = d8(&src, w, hh, i);
        if r < 0 {
            continue;
        }
        let ex = (src[i] as f64 - src[r as usize] as f64) - thr * d;
        if ex > 0.0 {
            h[i] = (h[i] as f64 - ex * 0.5) as f32;
            h[r as usize] = (h[r as usize] as f64 + ex * 0.5) as f32;
            moved += 1;
        }
    }
    moved
}

// ---------------------------------------------------------------------------
// The step

/// Which erosion candidate a chain runs (`Base` = steps 2-3 only: the
/// "noise-only through the ZR pipeline" control).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Kernel {
    Base,
    Bw,
    Ef3,
}

impl Kernel {
    fn tag(self) -> &'static str {
        match self {
            Kernel::Base => "zrbase",
            Kernel::Bw => "zrbw",
            Kernel::Ef3 => "zref3",
        }
    }
}

/// Per-step wall time of one tile, ms.
#[derive(Default, Clone, Copy)]
struct Steps {
    window: f64,
    up: f64,
    band: f64,
    fill: f64,
    flow: f64,
    burn: f64,
    erode: f64,
    talus: f64,
    taper: f64,
    total: f64,
    land: bool,
}

/// One refined tile's output: the 256^2 interior's quantised height and flow,
/// the base it must equal on its edge, and counters.
struct TileOut {
    hgt: Vec<f32>,
    flow: Vec<f32>,
    base: Vec<f32>,
    steps: Steps,
    burned: usize,
    talus: usize,
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

/// **The §4.1 step**, for tile `(tc, tr)` at level `k` from `parent` (level
/// `k - 1`).
fn refine_tile(c: &Coarse, parent: &Mosaic, k: u32, tc: i64, tr: i64, kern: Kernel) -> TileOut {
    let t0 = Instant::now();
    let mut st = Steps::default();
    let t_km = c.l_km / (1u64 << k) as f64;
    let (j, lam) = band_octave(t_km);
    let px0 = tc * TILE as i64 - HALO as i64;
    let py0 = tr * TILE as i64 - HALO as i64;
    // Step 1: the parent window, PW x PW from (px0/2 - 2, py0/2 - 2).
    let s = Instant::now();
    let (wx0, wy0) = (px0 / 2 - 2, py0 / 2 - 2);
    let mut pwh = vec![0f32; PW * PW];
    let mut pwf = vec![0f32; PW * PW];
    for y in 0..PW {
        for x in 0..PW {
            let i = parent.idx(wx0 + x as i64, wy0 + y as i64);
            pwh[y * PW + x] = parent.hgt[i];
            pwf[y * PW + x] = parent.flow[i];
        }
    }
    let land = pwh.iter().any(|&v| v as f64 >= c.sea);
    st.window = ms(s);
    // Step 2: the cubic, heights and flow (flow x4: same drained area in
    // child texel units).
    let s = Instant::now();
    let mut up = vec![0f32; PT * PT];
    let mut fup = vec![0f32; PT * PT];
    for y in 0..PT {
        for x in 0..PT {
            let (gx, gy) = (px0 + x as i64, py0 + y as i64);
            up[y * PT + x] = up2_at(parent, &parent.hgt, gx, gy) as f32;
            fup[y * PT + x] = (up2_at(parent, &parent.flow, gx, gy) * 4.0) as f32;
        }
    }
    st.up = ms(s);
    // Step 3: the band. `base` is f64-exact up to the f32 store.
    let s = Instant::now();
    let mut base = vec![0f32; PT * PT];
    for y in 0..PT {
        for x in 0..PT {
            let b = up[y * PT + x] as f64;
            base[y * PT + x] = (b + band_delta(c, k, px0 + x as i64, py0 + y as i64, b, j, lam)) as f32;
        }
    }
    st.band = ms(s);
    let mut h = base.clone();
    let mut flow_c = fup.clone();
    let (mut burned, mut moved) = (0, 0);
    if kern != Kernel::Base && land {
        // Step 4.
        let s = Instant::now();
        let (filled, order) = fill_eps(&base, PT, PT);
        st.fill = ms(s);
        let s = Instant::now();
        let place = TilePlacement { coarse_w: PW, coarse_h: PW, x0: 2, y0: 2, cols: PP, rows: PP, refine: 2, world: false };
        let acc = tile_flow(&place, &filled, &pwh, &pwf);
        flow_c = acc.iter().map(|&a| a * 4.0).collect();
        st.flow = ms(s);
        // Step 5.
        let s = Instant::now();
        burned = burn(&mut h, &base, &flow_c, PT, PT, t_km, c.sea);
        st.burn = ms(s);
        // Step 6.
        let s = Instant::now();
        let mut resist = vec![0f32; PT * PT];
        let mut rain = vec![0f32; PT * PT];
        let mut stress = vec![0f32; PT * PT];
        for y in 0..PT {
            for x in 0..PT {
                let (cx, cy) = (c.sx(k, px0 + x as i64), c.sx(k, py0 + y as i64));
                resist[y * PT + x] = samp(c.resist, c.gw, c.gh, cx, cy) as f32;
                rain[y * PT + x] = samp(c.rain, c.gw, c.gh, cx, cy) as f32;
                stress[y * PT + x] = samp(c.stress, c.gw, c.gh, cx, cy) as f32;
            }
        }
        match kern {
            Kernel::Bw => bw_erode(&mut h, &filled, &order, &flow_c, &resist, &rain, PT, PT, &c.sp, k),
            Kernel::Ef3 => {
                let seed = ef3_seed(&filled, &pwh, &pwf, k);
                tile_erode(&mut h, &stress, &resist, &rain, PT, PT, &c.sp, 1usize << k, &seed);
            }
            Kernel::Base => unreachable!(),
        }
        st.erode = ms(s);
        let s = Instant::now();
        moved = talus(&mut h, PT, PT, t_km, c.sea, PEAK_M);
        st.talus = ms(s);
    }
    // Step 7: taper the non-local change and the flow, quantise.
    let s = Instant::now();
    let mut hgt = vec![0f32; TILE * TILE];
    let mut flow = vec![0f32; TILE * TILE];
    let mut bq = vec![0f32; TILE * TILE];
    for ly in 0..TILE {
        for lx in 0..TILE {
            let d = lx.min(TILE - 1 - lx).min(ly).min(TILE - 1 - ly) as f64;
            let m = 1.0 - conform_weight(d, BAND);
            let pi = (ly + HALO) * PT + lx + HALO;
            let b = base[pi] as f64;
            let delta = h[pi] as f64 - b;
            hgt[ly * TILE + lx] = quant(b + m * delta);
            bq[ly * TILE + lx] = quant(b);
            flow[ly * TILE + lx] = (flow_c[pi] as f64 * m + fup[pi] as f64 * (1.0 - m)) as f32;
        }
    }
    st.taper = ms(s);
    st.total = ms(t0);
    st.land = land && kern != Kernel::Base;
    TileOut { hgt, flow, base: bq, steps: st, burned, talus: moved }
}

/// The app's default peak altitude (`WorldParams::defaults().peak_m`, 4 000 m),
/// restated so [`talus`] needs no `WorldParams`.
const PEAK_M: f64 = 4000.0;

/// EF-3's area seed with the parent inflow: `1/refine^2` per texel (base-cell
/// units, `uniform_area_seed`'s scale) plus each parent-window donor's flow
/// that crosses into the tile, injected at the lowest child texel of the
/// crossed face and stepped one texel inside the ring: the rule
/// `hydrology::tile::boundary_inflows` documents, restated (it is private).
fn ef3_seed(filled: &[f32], pwh: &[f32], pwf: &[f32], k: u32) -> Vec<f32> {
    let refine = (1u64 << k) as f64;
    let mut seed = vec![(1.0 / (refine * refine)) as f32; PT * PT];
    let parent_to_base = 4.0 / (refine * refine);
    let inside = |x: i64, y: i64| (2..2 + PP as i64).contains(&x) && (2..2 + PP as i64).contains(&y);
    for gy in 1..=(PP as i64 + 2) {
        for gx in 1..=(PP as i64 + 2) {
            if inside(gx, gy) {
                continue;
            }
            let ci = (gy * PW as i64 + gx) as usize;
            let (r, _) = d8(pwh, PW, PW, ci);
            if r < 0 {
                continue;
            }
            let (rx, ry) = (r % PW as i64, r / PW as i64);
            if !inside(rx, ry) || !(pwf[ci] > 0.0) {
                continue;
            }
            let (dcx, dcy) = (gx - rx, gy - ry);
            let (bx, by) = (2 * (rx - 2), 2 * (ry - 2));
            let span = |b: i64, dc: i64| if dc < 0 { (b, b + 1) } else if dc > 0 { (b + 1, b + 2) } else { (b, b + 2) };
            let ((x0, x1), (y0, y1)) = (span(bx, dcx), span(by, dcy));
            let (mut best, mut low) = ((x0, y0), f32::INFINITY);
            for fy in y0..y1 {
                for fx in x0..x1 {
                    let v = filled[(fy * PT as i64 + fx) as usize];
                    if v < low {
                        low = v;
                        best = (fx, fy);
                    }
                }
            }
            let (fx, fy) = (best.0.clamp(1, PT as i64 - 2), best.1.clamp(1, PT as i64 - 2));
            let i = (fy * PT as i64 + fx) as usize;
            seed[i] = (seed[i] as f64 + pwf[ci] as f64 * parent_to_base) as f32;
        }
    }
    seed
}

/// One level: every tile of `rect` refined from `parent`, stitched. `par`
/// runs tiles as independent rayon jobs (the shipped pool's shape); each tile
/// is serial inside except `tile_erode`'s own parallel arms.
struct LevelOut {
    m: Mosaic,
    steps: Vec<Steps>,
    edge_mismatch: usize,
    edge_checked: usize,
    burned: usize,
    talus: usize,
}

fn refine_level(c: &Coarse, parent: &Mosaic, k: u32, rect: TileRect, kern: Kernel, par: bool) -> LevelOut {
    let tiles: Vec<(i64, i64)> = (rect.r0..rect.r1).flat_map(|r| (rect.c0..rect.c1).map(move |cc| (cc, r))).collect();
    let run = |&(tc, tr): &(i64, i64)| refine_tile(c, parent, k, tc, tr, kern);
    let outs: Vec<TileOut> = if par { tiles.par_iter().map(run).collect() } else { tiles.iter().map(run).collect() };
    let (w, h) = ((rect.c1 - rect.c0) as usize * TILE, (rect.r1 - rect.r0) as usize * TILE);
    let mut m = Mosaic { k, x0: rect.c0 * TILE as i64, y0: rect.r0 * TILE as i64, w, h, hgt: vec![0.0; w * h], flow: vec![0.0; w * h] };
    let (t_km, (j, lam)) = (c.l_km / (1u64 << k) as f64, band_octave(c.l_km / (1u64 << k) as f64));
    let _ = t_km;
    let (mut mis, mut chk, mut burned, mut tal) = (0, 0, 0, 0);
    let mut steps = Vec::new();
    for (&(tc, tr), o) in tiles.iter().zip(&outs) {
        let (ox, oy) = ((tc - rect.c0) as usize * TILE, (tr - rect.r0) as usize * TILE);
        for ly in 0..TILE {
            let row = (oy + ly) * w + ox;
            m.hgt[row..row + TILE].copy_from_slice(&o.hgt[ly * TILE..(ly + 1) * TILE]);
            m.flow[row..row + TILE].copy_from_slice(&o.flow[ly * TILE..(ly + 1) * TILE]);
        }
        // The seam guarantee: every edge texel is the base, bit for bit, and
        // the base recomputed here as a pure function of the global index
        // agrees with the tile's own.
        for q in 0..TILE {
            for (lx, ly) in [(q, 0), (q, TILE - 1), (0, q), (TILE - 1, q)] {
                let (gx, gy) = (tc * TILE as i64 + lx as i64, tr * TILE as i64 + ly as i64);
                let pure = quant(base_at(c, parent, k, gx, gy, j, lam));
                let i = ly * TILE + lx;
                chk += 1;
                if o.hgt[i].to_bits() != pure.to_bits() || o.base[i].to_bits() != pure.to_bits() {
                    mis += 1;
                }
            }
        }
        burned += o.burned;
        tal += o.talus;
        steps.push(o.steps);
    }
    assert!(m.hgt.iter().any(|&v| v != 0.0), "level {k} mosaic is empty");
    LevelOut { m, steps, edge_mismatch: mis, edge_checked: chk, burned, talus: tal }
}

/// The tile rectangles levels `1..=kmax` need so that `want[k]` (texel
/// rectangles a frame reads, already including the render halo) are covered:
/// each level's rectangle is its own want unioned with what its child reads.
fn plan(want: &HashMap<u32, (i64, i64, i64, i64)>, kmax: u32) -> HashMap<u32, TileRect> {
    let mut rects: HashMap<u32, TileRect> = HashMap::new();
    for k in (1..=kmax).rev() {
        let mut r: Option<TileRect> = want.get(&k).map(|&(a, b, cc, d)| TileRect::covering(a, b, cc, d));
        if let Some(child) = rects.get(&(k + 1)) {
            let (a, b, cc, d) = child.parent_texels();
            let pr = TileRect::covering(a, b, cc, d);
            r = Some(r.map_or(pr, |x| x.union(pr)));
        }
        if let Some(r) = r {
            rects.insert(k, r);
        }
    }
    rects
}

/// Runs a whole chain: levels `1..=kmax` over `rects`, coarsest first.
fn run_chain(c: &Coarse, rects: &HashMap<u32, TileRect>, kmax: u32, kern: Kernel, par: bool) -> Vec<LevelOut> {
    let mut out: Vec<LevelOut> = Vec::new();
    let base = Mosaic::base(c);
    for k in 1..=kmax {
        let parent = if k == 1 { &base } else { &out.last().unwrap().m };
        let lv = refine_level(c, parent, k, rects[&k], kern, par);
        out.push(lv);
    }
    out
}

// ---------------------------------------------------------------------------
// Seams

/// Channel continuity across every tile seam of a mosaic (ZR-1's bar, scope
/// §5): channel texels (valley width >= 2 texels, the level's burn threshold
/// and above) within 2 texels of a seam on one side, and the fraction with a
/// channel texel within 2 texels across it. Also the seam step ratio: the mean
/// `|h(X) - h(X-1)|` across seam columns/rows over the same quantity one
/// texel inside (1.0 = no seam signature).
fn seam_stats(m: &Mosaic, t_km: f64) -> (usize, usize, f64) {
    let is_ch = |i: usize| VALLEY_W_K * (m.flow[i] as f64 * t_km * t_km).sqrt() / t_km >= 2.0;
    let (w, h) = (m.w, m.h);
    let (mut tot, mut cont) = (0usize, 0usize);
    let (mut s_seam, mut n_seam, mut s_in, mut n_in) = (0.0, 0usize, 0.0, 0usize);
    // Vertical seams at local x = multiples of TILE (excluding 0).
    for sx in (TILE..w).step_by(TILE) {
        for y in 0..h {
            s_seam += (m.hgt[y * w + sx] as f64 - m.hgt[y * w + sx - 1] as f64).abs();
            n_seam += 1;
            s_in += (m.hgt[y * w + sx - 3] as f64 - m.hgt[y * w + sx - 4] as f64).abs();
            n_in += 1;
            for (side, other) in [((sx - 2)..sx, sx..(sx + 2)), (sx..(sx + 2), (sx - 2)..sx)] {
                for x in side {
                    if !is_ch(y * w + x) {
                        continue;
                    }
                    tot += 1;
                    let hit = other.clone().any(|ox| (y.saturating_sub(2)..(y + 3).min(h)).any(|oy| is_ch(oy * w + ox)));
                    if hit {
                        cont += 1;
                    }
                }
            }
        }
    }
    for sy in (TILE..h).step_by(TILE) {
        for x in 0..w {
            s_seam += (m.hgt[sy * w + x] as f64 - m.hgt[(sy - 1) * w + x] as f64).abs();
            n_seam += 1;
            s_in += (m.hgt[(sy - 3) * w + x] as f64 - m.hgt[(sy - 4) * w + x] as f64).abs();
            n_in += 1;
            for (side, other) in [((sy - 2)..sy, sy..(sy + 2)), (sy..(sy + 2), (sy - 2)..sy)] {
                for y in side {
                    if !is_ch(y * w + x) {
                        continue;
                    }
                    tot += 1;
                    let hit = other.clone().any(|oy| (x.saturating_sub(2)..(x + 3).min(w)).any(|ox| is_ch(oy * w + ox)));
                    if hit {
                        cont += 1;
                    }
                }
            }
        }
    }
    let ratio = if n_in > 0 && s_in > 0.0 { (s_seam / n_seam as f64) / (s_in / n_in as f64) } else { f64::NAN };
    (tot, cont, ratio)
}

// ---------------------------------------------------------------------------
// Rendering, through the shipped colouriser

/// A `FRAME x FRAME` RGB frame of a refined level mosaic centred on sample
/// coordinate `(gx, gy)` at `ppc` screen pixels per coarse cell: the mosaic is
/// coloured in 256-texel tiles by `render_biome_tile_rgba_water` (the shell's
/// deep-zoom colouriser, no river layer, as `zoom_detail_ladder.rs`), then
/// sampled bilinearly.
fn zr_frame(ctx: &RenderCtx, tf: &TileFields, m: &Mosaic, gx: f64, gy: f64, ppc: f64) -> Vec<u8> {
    let k = m.k;
    let sc = (1u64 << k) as f64;
    let tex = |s: f64| (s + 0.5) * sc - 0.5;
    let half = FRAME as f64 / 2.0 / ppc;
    let (x0, x1) = (tex(gx - half).floor() as i64 - 1, tex(gx + half).ceil() as i64 + 2);
    let (y0, y1) = (tex(gy - half).floor() as i64 - 1, tex(gy + half).ceil() as i64 + 2);
    let rect = TileRect::covering(x0, y0, x1, y1);
    let (iw, ih) = ((rect.c1 - rect.c0) as usize * TILE, (rect.r1 - rect.r0) as usize * TILE);
    let mut img = vec![0u8; iw * ih * 4];
    let tb_w = (TILE - 1) as f64 / sc;
    let pad = render::tile_halo_px(ctx, TILE, TILE, TileBounds { x: 0.0, y: 0.0, w: tb_w, h: tb_w });
    let tiles: Vec<(i64, i64)> = (rect.r0..rect.r1).flat_map(|r| (rect.c0..rect.c1).map(move |cc| (cc, r))).collect();
    let rgbas: Vec<Vec<u8>> = tiles
        .par_iter()
        .map(|&(tc, tr)| {
            let (tx0, ty0) = (tc * TILE as i64, tr * TILE as i64);
            let pw = TILE + 2 * pad;
            let mut data = vec![0f32; pw * pw];
            for y in 0..pw {
                for x in 0..pw {
                    data[y * pw + x] = m.hgt[m.idx(tx0 - pad as i64 + x as i64, ty0 - pad as i64 + y as i64)];
                }
            }
            let tb = TileBounds { x: (tx0 as f64 + 0.5) / sc - 0.5, y: (ty0 as f64 + 0.5) / sc - 0.5, w: tb_w, h: tb_w };
            let rgba = render::render_biome_tile_rgba_water(ctx, &data, None, TILE, TILE, pad, tb, tf, None);
            assert_eq!(rgba.len(), TILE * TILE * 4, "the colouriser refused a refined tile");
            rgba
        })
        .collect();
    for (&(tc, tr), rgba) in tiles.iter().zip(&rgbas) {
        let (ox, oy) = ((tc - rect.c0) as usize * TILE, (tr - rect.r0) as usize * TILE);
        for ly in 0..TILE {
            let d = ((oy + ly) * iw + ox) * 4;
            img[d..d + TILE * 4].copy_from_slice(&rgba[ly * TILE * 4..(ly + 1) * TILE * 4]);
        }
    }
    let (ix0, iy0) = (rect.c0 * TILE as i64, rect.r0 * TILE as i64);
    let mut out = vec![0u8; FRAME * FRAME * 3];
    for sy in 0..FRAME {
        let cy = gy + (sy as f64 + 0.5 - FRAME as f64 / 2.0) / ppc;
        let fy = tex(cy) - iy0 as f64;
        for sx in 0..FRAME {
            let cx = gx + (sx as f64 + 0.5 - FRAME as f64 / 2.0) / ppc;
            let fx = tex(cx) - ix0 as f64;
            bilerp_into(&img, iw, ih, fx, fy, &mut out[(sy * FRAME + sx) * 3..(sy * FRAME + sx) * 3 + 3]);
        }
    }
    out
}

fn bilerp_into(img: &[u8], w: usize, h: usize, fx: f64, fy: f64, o: &mut [u8]) {
    let fx = fx.clamp(0.0, (w - 1) as f64);
    let fy = fy.clamp(0.0, (h - 1) as f64);
    let (x0, y0) = (fx as usize, fy as usize);
    let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
    let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
    for c in 0..3 {
        let p = |x: usize, y: usize| img[(y * w + x) * 4 + c] as f64;
        let v = (p(x0, y0) * (1.0 - tx) + p(x1, y0) * tx) * (1.0 - ty) + (p(x0, y1) * (1.0 - tx) + p(x1, y1) * tx) * ty;
        o[c] = v.round().clamp(0.0, 255.0) as u8;
    }
}

/// The shipped deep-zoom tile (`zoom_detail_ladder.rs`' `tile`, the FG law):
/// `pyramid_tile_padded_rough` + `add_tile_furrows` +
/// `render_biome_tile_rgba_water`.
fn fg_tile(ctx: &RenderCtx, tf: &TileFields, z: i32, col: u32, row: u32, seed: i32) -> (Vec<u8>, usize, usize, TileBounds) {
    let (gw, gh) = (ctx.gw, ctx.gh);
    let b = pyramid_tile_bounds(gw, gh, z, col, row);
    let n = pyramid_dims(z).cols as usize;
    let d = tile_dims(&Region { x: 0, y: 0, w: gw - 1, h: gh - 1 }, n, n, TILE);
    let tb = TileBounds { x: b.x, y: b.y, w: b.w, h: b.h };
    let pad = render::tile_halo_px(ctx, d.w, d.h, tb);
    let opts = render::tile_amplify_opts(ctx, seed, cartalith_terrain::amplify::z_base_for_tile_size(TILE));
    let rho = |cx: f64, cy: f64| render::detail_roughness_at(ctx.temperature, ctx.rainfall, gw, gh, cx, cy);
    let (w, h, mut data) = pyramid_tile_padded_rough(ctx.field, gw, gh, ChunkId::new(z as u32, col, row), TILE, pad, &opts, Some(&rho));
    render::add_tile_furrows(ctx, &mut data, w, h, pad, tb);
    let rgba = render::render_biome_tile_rgba_water(ctx, &data, None, w, h, pad, tb, tf, None);
    assert_eq!(rgba.len(), w * h * 4, "the colouriser refused the tile");
    (rgba, w, h, tb)
}

/// The shell's frame of the shipped tiles (`zoom_detail_ladder.rs`' `frame`
/// and `level_frame`, with the frame size and the level cap as arguments:
/// the 40 075 km world reaches level 16 under FG's F2, `lod_bridge::
/// max_level_for_map`; the native maps stay at `BASE_MAX_LEVEL` 10).
#[allow(clippy::too_many_arguments)]
fn fg_frame(ctx: &RenderCtx, tf: &TileFields, gx: f64, gy: f64, ppc: f64, seed: i32, fw: usize, fh: usize, max_level: i32) -> Vec<u8> {
    let z = pyramid_level_for_zoom(ppc, ctx.gw as f64, TILE as f64, Some(max_level));
    let want = f64::max(1.0, ctx.gw as f64 * ppc / TILE as f64);
    let t = (want.log2() - z as f64 + 0.5).clamp(0.0, 1.0);
    let morph = t * t * (3.0 - 2.0 * t);
    let child = fg_level_frame(ctx, tf, gx, gy, ppc, seed, z, fw, fh);
    if z == 0 || morph >= 1.0 {
        return child;
    }
    let parent = fg_level_frame(ctx, tf, gx, gy, ppc, seed, z - 1, fw, fh);
    child.iter().zip(&parent).map(|(&c, &p)| (p as f64 + (c as f64 - p as f64) * morph).round() as u8).collect()
}

#[allow(clippy::too_many_arguments)]
fn fg_level_frame(ctx: &RenderCtx, tf: &TileFields, gx: f64, gy: f64, ppc: f64, seed: i32, z: i32, fw: usize, fh: usize) -> Vec<u8> {
    let (gw, gh) = (ctx.gw, ctx.gh);
    let n = pyramid_dims(z).cols;
    let step_x = (gw - 1) as f64 / n as f64;
    let step_y = (gh - 1) as f64 / n as f64;
    // Which tiles the frame touches, rendered in parallel first.
    let mut need: Vec<(u32, u32)> = Vec::new();
    for sy in (0..fh).step_by(8).chain([fh - 1]) {
        let cy = (gy + (sy as f64 + 0.5 - fh as f64 / 2.0) / ppc).clamp(0.0, (gh - 1) as f64);
        let row = ((cy / step_y) as u32).min(n - 1);
        for sx in (0..fw).step_by(8).chain([fw - 1]) {
            let cx = (gx + (sx as f64 + 0.5 - fw as f64 / 2.0) / ppc).clamp(0.0, (gw - 1) as f64);
            let col = ((cx / step_x) as u32).min(n - 1);
            if !need.contains(&(col, row)) {
                need.push((col, row));
            }
        }
    }
    let rendered: Vec<((u32, u32), (Vec<u8>, usize, usize, TileBounds))> = need.par_iter().map(|&(c, r)| ((c, r), fg_tile(ctx, tf, z, c, r, seed))).collect();
    let cache: HashMap<(u32, u32), (Vec<u8>, usize, usize, TileBounds)> = rendered.into_iter().collect();
    let mut out = vec![0u8; fw * fh * 3];
    for sy in 0..fh {
        let cy = (gy + (sy as f64 + 0.5 - fh as f64 / 2.0) / ppc).clamp(0.0, (gh - 1) as f64);
        let row = ((cy / step_y) as u32).min(n - 1);
        for sx in 0..fw {
            let cx = (gx + (sx as f64 + 0.5 - fw as f64 / 2.0) / ppc).clamp(0.0, (gw - 1) as f64);
            let col = ((cx / step_x) as u32).min(n - 1);
            let t = cache.get(&(col, row)).expect("a frame pixel's tile was not pre-rendered");
            let (rgba, w, h, tb) = (&t.0, t.1, t.2, t.3);
            let fx = (cx - tb.x) / tb.w * (w - 1) as f64;
            let fy = (cy - tb.y) / tb.h * (h - 1) as f64;
            bilerp_into(rgba, w, h, fx, fy, &mut out[(sy * fw + sx) * 3..(sy * fw + sx) * 3 + 3]);
        }
    }
    out
}

/// An **input** land mask for a frame (`zoom_detail_ladder.rs`' `mask_frame`):
/// land where all four bilinear corners of the pixel's coarse position are at
/// or above sea level, never chosen by the rendered colour.
fn land_mask(ctx: &RenderCtx, gx: f64, gy: f64, ppc: f64, fw: usize, fh: usize) -> Vec<bool> {
    let (gw, gh) = (ctx.gw, ctx.gh);
    let land = |x: usize, y: usize| ctx.field[y * gw + x] as f64 >= ctx.sea_level;
    let mut out = vec![false; fw * fh];
    for sy in 0..fh {
        let cy = (gy + (sy as f64 + 0.5 - fh as f64 / 2.0) / ppc).clamp(0.0, (gh - 1) as f64);
        for sx in 0..fw {
            let cx = (gx + (sx as f64 + 0.5 - fw as f64 / 2.0) / ppc).clamp(0.0, (gw - 1) as f64);
            let (x0, y0) = (cx as usize, cy as usize);
            let (x1, y1) = ((x0 + 1).min(gw - 1), (y0 + 1).min(gh - 1));
            out[sy * fw + sx] = land(x0, y0) && land(x1, y0) && land(x0, y1) && land(x1, y1);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Metrics

fn luma(rgb: &[u8]) -> Vec<f64> {
    rgb.chunks(3).map(|p| 0.2126 * p[0] as f64 + 0.7152 * p[1] as f64 + 0.0722 * p[2] as f64).collect()
}

/// Separable box blur, radius `r`, three passes (about a Gaussian of
/// `sigma = r`), clamped at the frame edge.
fn blur(v: &[f64], w: usize, h: usize, r: usize) -> Vec<f64> {
    let mut a = v.to_vec();
    for _ in 0..3 {
        let mut b = vec![0.0; w * h];
        for y in 0..h {
            for x in 0..w {
                let (lo, hi) = (x.saturating_sub(r), (x + r).min(w - 1));
                b[y * w + x] = (lo..=hi).map(|q| a[y * w + q]).sum::<f64>() / (hi - lo + 1) as f64;
            }
        }
        for y in 0..h {
            for x in 0..w {
                let (lo, hi) = (y.saturating_sub(r), (y + r).min(h - 1));
                a[y * w + x] = (lo..=hi).map(|q| b[q * w + x]).sum::<f64>() / (hi - lo + 1) as f64;
            }
        }
    }
    a
}

/// The frame statistics: land luma gradient (mean `|grad luma|`, forward
/// differences, both pixels land: the FG findings' `grad`) and `bind_r`.
///
/// `bind_r` here, **one definition for every frame** (the FG harness's needs an
/// on/off pair, and a native map at fit has no off frame: its B4 `bind_r` was
/// never measured): Pearson r across 32 px land blocks (>= 90 % land) between
/// the std of the fine luma (`luma - blur16(luma)`) and the mean gradient of
/// the blurred luma. Near 0 means the fine detail ignores the terrain one to
/// two octaves up; high means it follows it.
struct Stats {
    grad: f64,
    bind: f64,
    blocks: usize,
    land: f64,
}

fn stats(rgb: &[u8], mask: &[bool], w: usize, h: usize) -> Stats {
    let l = luma(rgb);
    let mut s = 0.0;
    let mut n = 0usize;
    for y in 0..h - 1 {
        for x in 0..w - 1 {
            let i = y * w + x;
            if mask[i] && mask[i + 1] && mask[i + w] {
                s += (l[i + 1] - l[i]).hypot(l[i + w] - l[i]);
                n += 1;
            }
        }
    }
    let lp = blur(&l, w, h, 16);
    let (mut sa, mut sr) = (Vec::new(), Vec::new());
    const B: usize = 32;
    for by in (0..h - B).step_by(B) {
        for bx in (0..w - B).step_by(B) {
            let mut land = 0;
            let mut vals = Vec::with_capacity(B * B);
            let mut g = 0.0;
            for y in by..by + B {
                for x in bx..bx + B {
                    let i = y * w + x;
                    if mask[i] {
                        land += 1;
                    }
                    vals.push(l[i] - lp[i]);
                    g += (lp[i + 1] - lp[i]).hypot(lp[i + w] - lp[i]);
                }
            }
            if (land as f64) < 0.9 * (B * B) as f64 {
                continue;
            }
            let mean = vals.iter().sum::<f64>() / vals.len() as f64;
            sa.push((vals.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / vals.len() as f64).sqrt());
            sr.push(g / (B * B) as f64);
        }
    }
    let bind = pearson(&sa, &sr);
    Stats { grad: if n > 0 { s / n as f64 } else { f64::NAN }, bind, blocks: sa.len(), land: n as f64 / ((w - 1) * (h - 1)) as f64 }
}

fn pearson(a: &[f64], b: &[f64]) -> f64 {
    if a.len() < 8 {
        return f64::NAN;
    }
    let n = a.len() as f64;
    let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        sab += (x - ma) * (y - mb);
        saa += (x - ma) * (x - ma);
        sbb += (y - mb) * (y - mb);
    }
    if saa <= 0.0 || sbb <= 0.0 { f64::NAN } else { sab / (saa * sbb).sqrt() }
}

// ---------------------------------------------------------------------------
// Places, picked by inputs (never by a value under test)

/// A mountain and a plain on `w`: candidates on a stride grid, kept `r` cells
/// from the edges and (on a wrapping world) off the poles; the window
/// `(2r+1)^2` must be at least `land_min` land; mountain = highest mean coarse
/// local relief (`km_local_relief`), plain = lowest, among land windows; plus
/// the FG pivot.
fn places(w: &World, r: usize) -> Vec<(&'static str, f64, f64, f64)> {
    let (gw, gh) = (w.p.gw, w.p.gh);
    let f = &w.ws.field;
    let sea = w.ws.sea_level;
    let stride = (r / 3).max(2);
    let relief_at = |x: usize, y: usize| {
        let (l, rr) = (f[y * gw + x - 1] as f64, f[y * gw + x + 1] as f64);
        let (u, d) = (f[(y - 1) * gw + x] as f64, f[(y + 1) * gw + x] as f64);
        km_local_relief((rr - l) * 0.5, (d - u) * 0.5, (l + rr + u + d) * 0.25 - f[y * gw + x] as f64)
    };
    let (ylo, yhi) = if w.p.world { (gh / 5, gh - gh / 5) } else { (r + 2, gh - r - 2) };
    for land_min in [0.97, 0.9, 0.8, 0.7, 0.6] {
        let mut best_hi = (f64::NEG_INFINITY, 0, 0, 0.0);
        let mut best_lo = (f64::INFINITY, 0, 0, 0.0);
        let mut y = ylo.max(r + 2);
        while y < yhi.min(gh - r - 2) {
            let mut x = r + 2;
            while x < gw - r - 2 {
                let (mut land, mut rel, mut cnt) = (0usize, 0.0, 0usize);
                for yy in (y - r..=y + r).step_by(2) {
                    for xx in (x - r..=x + r).step_by(2) {
                        cnt += 1;
                        if f[yy * gw + xx] as f64 >= sea {
                            land += 1;
                            rel += relief_at(xx, yy);
                        }
                    }
                }
                let lf = land as f64 / cnt as f64;
                if lf >= land_min {
                    let m = rel / land as f64;
                    if m > best_hi.0 {
                        best_hi = (m, x, y, lf);
                    }
                    if m < best_lo.0 {
                        best_lo = (m, x, y, lf);
                    }
                }
                x += stride;
            }
            y += stride;
        }
        if best_hi.0.is_finite() && best_lo.0.is_finite() {
            // Third place: the FG harness's own pivot, `(0.40, 0.625)` of the
            // grid (`fg_deep_tile_cost`, `_fg_probe.gd`), so these frames tie to
            // FG's screen metrics. Its land fraction is not a pick criterion.
            let pv = (0.40 * (gw - 1) as f64, 0.625 * (gh - 1) as f64);
            return vec![
                ("mountain", best_hi.1 as f64, best_hi.2 as f64, best_hi.3),
                ("plain", best_lo.1 as f64, best_lo.2 as f64, best_lo.3),
                ("pivot", pv.0.round(), pv.1.round(), f64::NAN),
            ];
        }
    }
    panic!("no land window of radius {r} on this world");
}

fn write_jpg(path: &str, rgb: &[u8], w: usize, h: usize) {
    let mut buf = Vec::new();
    let enc = jpeg_encoder::Encoder::new(&mut buf, 88);
    enc.encode(rgb, w as u16, h as u16, jpeg_encoder::ColorType::Rgb).unwrap();
    std::fs::write(path, buf).unwrap();
}

fn write_png(path: &str, rgb: &[u8], w: usize, h: usize) {
    let png = cartalith_assets::raster::encode_png_rgb8(w as u32, h as u32, rgb.to_vec(), cartalith_assets::raster::PngColour::Data).unwrap();
    std::fs::write(path, png).unwrap();
}

/// Side-by-side sheet of equal-size RGB frames with a 6 px gutter.
fn sheet(frames: &[&[u8]], w: usize, h: usize) -> (Vec<u8>, usize) {
    let g = 6;
    let sw = frames.len() * w + (frames.len() - 1) * g;
    let mut out = vec![255u8; sw * h * 3];
    for (k, f) in frames.iter().enumerate() {
        for y in 0..h {
            let d = (y * sw + k * (w + g)) * 3;
            out[d..d + w * 3].copy_from_slice(&f[y * w * 3..(y + 1) * w * 3]);
        }
    }
    (out, sw)
}

/// The refined level a view at `ppc` screen pixels per base cell draws, and
/// its morph over the level above: the shell's rule
/// (`pyramid_level_for_zoom`'s `round(log2(want))`, `lod_bridge::
/// morph_for_zoom`'s `smoothstep(log2(want) - z + 0.5)`), with `want` the
/// texels per base cell a level holds (`2^k`). So the finest band of the
/// drawn level sits at 2-5 screen pixels, as the FG tile's Nyquist margin
/// keeps it, rather than at pixel scale (a first run drew the next level
/// minified and measured pixel grain). Display-side `log2`, never tile content.
fn zr_view(ppc: f64) -> (u32, f64) {
    let lw = ppc.max(2.0).log2();
    let z = (lw.round() as u32).max(2);
    let t = (lw - z as f64 + 0.5).clamp(0.0, 1.0);
    (z, t * t * (3.0 - 2.0 * t))
}

/// The texel rectangle a `FRAME` frame at `ppc` around `(gx, gy)` reads at
/// level `k`, with a render halo margin.
fn frame_want(k: u32, gx: f64, gy: f64, ppc: f64, margin: i64) -> (i64, i64, i64, i64) {
    let sc = (1u64 << k) as f64;
    let tex = |s: f64| (s + 0.5) * sc - 0.5;
    let half = FRAME as f64 / 2.0 / ppc;
    let rect = TileRect::covering(tex(gx - half).floor() as i64 - 1, tex(gy - half).floor() as i64 - 1, tex(gx + half).ceil() as i64 + 2, tex(gy + half).ceil() as i64 + 2);
    (rect.c0 * TILE as i64 - margin, rect.r0 * TILE as i64 - margin, rect.c1 * TILE as i64 + margin, rect.r1 * TILE as i64 + margin)
}

fn median(v: &mut [f64]) -> (f64, f64, f64) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (v[v.len() / 2], v[0], v[v.len() - 1])
}

// ---------------------------------------------------------------------------
// The tests

/// **The go/no-go measurement.** Generates the 40 075 km world (seed 1200,
/// `ZR_GW` x `ZR_GW/2`, default 2048) and the native 1 200 km and 200 km maps
/// (seed 1200, 1024 x 512, region mode: the FG harness's sweep maps); picks a
/// mountain and a plain on each by inputs; runs the refinement chain with
/// both kernels (and the steps-2-3-only control) down to the levels whose
/// texel matches 0.471 and 0.078 km/px; renders native | FG | ZR-base |
/// ZR-BW | ZR-EF3 at equal km/px; prints every statistic, seam and channel
/// continuity figure. Frames go to `ZR_OUT`.
///
/// Protects: nothing (a measurement, `#[ignore]`d).
#[test]
#[ignore = "generates three worlds and refines and renders them; run explicitly"]
fn zr0_bars() {
    let out = std::env::var("ZR_OUT").expect("set ZR_OUT to an output directory");
    std::fs::create_dir_all(&out).unwrap();
    let gw: usize = std::env::var("ZR_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(2048);
    let t0 = Instant::now();
    let world = World::generate(1200, gw, gw / 2, 40075.0, true);
    println!("world 40075 km {}x{} generated in {:.1}s, field fnv {:016x}", gw, gw / 2, t0.elapsed().as_secs_f64(), fnv(&world.ws.field));
    let n1200 = World::generate(1200, 1024, 512, 1200.0, false);
    let n200 = World::generate(1200, 1024, 512, 200.0, false);
    let wctx = world.ctx();
    let wtf = TileFields::new(&wctx, None);
    let coarse = Coarse::new(&world, &wctx);
    let l = world.l_km();
    println!("world km/cell {l:.4}; FG km law strength {} relief_gain {} floor {} cap {}", coarse.km.strength, coarse.km.relief_gain, coarse.km.relief_floor, coarse.km.cap);
    let wmax = 16; // `lod_bridge::max_level_for_map(Some(19.57))` = 16 (FG F2).
    let mut lines: Vec<String> = Vec::new();
    for (kmpx, native, ntag) in [(KMPX_1200, &n1200, "n1200"), (KMPX_200, &n200, "n200")] {
        let (k, morph) = zr_view(l / kmpx);
        let t_km = l / (1u64 << k) as f64;
        let wppc = l / kmpx;
        let nl = native.l_km();
        let nppc = nl / kmpx;
        let nctx = native.ctx();
        let ntf = TileFields::new(&nctx, None);
        let wr = (FRAME as f64 / 2.0 / wppc).ceil() as usize + 1;
        let nr = (FRAME as f64 / 2.0 / nppc).ceil() as usize + 1;
        let wpl = places(&world, wr.max(3));
        let npl = places(native, nr.max(3));
        println!("== {kmpx} km/px: world level {k} (texel {t_km:.4} km) over level {} at morph {morph:.3}, world ppc {wppc:.2}, native {ntag} ppc {nppc:.3}", k - 1);
        println!("   world places {wpl:?}; native places {npl:?}");
        // The native map at fit, whole map, at this km/px.
        let (fw, fh) = ((native.p.gw as f64 * nppc).round() as usize, (native.p.gh as f64 * nppc).round() as usize);
        let (cxm, cym) = ((native.p.gw - 1) as f64 / 2.0, (native.p.gh - 1) as f64 / 2.0);
        let nfit = fg_frame(&nctx, &ntf, cxm, cym, nppc, 1200, fw, fh, 10);
        let nfit_mask = land_mask(&nctx, cxm, cym, nppc, fw, fh);
        let nfs = stats(&nfit, &nfit_mask, fw, fh);
        write_jpg(&format!("{out}/{ntag}_fit_whole.jpg"), &nfit, fw, fh);
        let line = format!("{ntag}_fit_whole        grad {:6.3} bind_r {:5.2} blocks {:4} land {:.2}", nfs.grad, nfs.bind, nfs.blocks, nfs.land);
        println!("{line}");
        lines.push(format!("{kmpx} {line}"));
        for (pi, &(pname, px, py, plf)) in wpl.iter().enumerate() {
            let (gx, gy) = (px, py);
            let (nname, nx, ny, nlf) = npl[pi];
            assert_eq!(pname, nname);
            let mask_w = land_mask(&wctx, gx, gy, wppc, FRAME, FRAME);
            let mask_n = land_mask(&nctx, nx, ny, nppc, FRAME, FRAME);
            let nat = fg_frame(&nctx, &ntf, nx, ny, nppc, 1200, FRAME, FRAME, 10);
            let fg = fg_frame(&wctx, &wtf, gx, gy, wppc, 1200, FRAME, FRAME, wmax);
            let ns = stats(&nat, &mask_n, FRAME, FRAME);
            let fs = stats(&fg, &mask_w, FRAME, FRAME);
            let mut frames: Vec<(String, Vec<u8>)> = vec![(format!("{ntag}_native"), nat), ("fg_noise".into(), fg)];
            let report = |name: &str, s: &Stats, lines: &mut Vec<String>| {
                let r = format!(
                    "{kmpx} {pname:8} {name:12} grad {:6.3} ratio_vs_native {:5.2} ratio_vs_native_whole {:5.2} bind_r {:5.2} (native {:5.2}, diff {:5.2}) blocks {:4} land {:.2}",
                    s.grad,
                    s.grad / ns.grad,
                    s.grad / nfs.grad,
                    s.bind,
                    ns.bind,
                    (s.bind - ns.bind).abs(),
                    s.blocks,
                    s.land
                );
                println!("{r}");
                lines.push(r);
            };
            println!("   {pname}: world cell ({gx},{gy}) land {plf:.2}; native cell ({nx},{ny}) land {nlf:.2}");
            report("native", &ns, &mut lines);
            report("fg_noise", &fs, &mut lines);
            // The chains.
            let mut want = HashMap::new();
            want.insert(k, frame_want(k, gx, gy, wppc, 24));
            want.insert(k - 1, frame_want(k - 1, gx, gy, wppc, 24));
            let rects = plan(&want, k);
            for kern in [Kernel::Base, Kernel::Bw, Kernel::Ef3] {
                let ts = Instant::now();
                let chain = run_chain(&coarse, &rects, k, kern, true);
                let el = ts.elapsed().as_secs_f64();
                let child = zr_frame(&wctx, &wtf, &chain[k as usize - 1].m, gx, gy, wppc);
                let parent = zr_frame(&wctx, &wtf, &chain[k as usize - 2].m, gx, gy, wppc);
                let zr: Vec<u8> = child.iter().zip(&parent).map(|(&c, &p)| (p as f64 + (c as f64 - p as f64) * morph).round() as u8).collect();
                let zs = stats(&zr, &mask_w, FRAME, FRAME);
                report(kern.tag(), &zs, &mut lines);
                let tiles: usize = chain.iter().map(|lv| lv.steps.len()).sum();
                let mis: usize = chain.iter().map(|lv| lv.edge_mismatch).sum();
                let chk: usize = chain.iter().map(|lv| lv.edge_checked).sum();
                let burned: usize = chain.iter().map(|lv| lv.burned).sum();
                let tal: usize = chain.iter().map(|lv| lv.talus).sum();
                let r = format!("{kmpx} {pname:8} {:12} chain: {tiles} tiles over levels 1..={k} in {el:.2}s (all cores); edge texels off the base {mis} of {chk}; burned {burned} texels; talus moved {tal}", kern.tag());
                println!("{r}");
                lines.push(r);
                if kern != Kernel::Base {
                    for lv in chain.iter().filter(|lv| lv.m.k + 2 >= k) {
                        let tk = l / (1u64 << lv.m.k) as f64;
                        let (tot, cont, ratio) = seam_stats(&lv.m, tk);
                        let r = format!(
                            "{kmpx} {pname:8} {:12} seams level {}: {}x{} tiles, channel texels at seams {tot}, continued {cont} ({:.1} %), seam step ratio {ratio:.3}",
                            kern.tag(),
                            lv.m.k,
                            lv.m.w / TILE,
                            lv.m.h / TILE,
                            if tot > 0 { 100.0 * cont as f64 / tot as f64 } else { f64::NAN }
                        );
                        println!("{r}");
                        lines.push(r);
                    }
                    // Cone and cold-jump ancestry (scope §5 ZR-0's last item):
                    // the tiles one final-level tile needs at every level.
                    if pi == 0 && kern == Kernel::Bw {
                        let one = TileRect { c0: rects[&k].c0, r0: rects[&k].r0, c1: rects[&k].c0 + 1, r1: rects[&k].r0 + 1 };
                        let mut cur = one;
                        let mut anc = Vec::new();
                        for kk in (1..k).rev() {
                            let (a, b, c2, d) = cur.parent_texels();
                            cur = TileRect::covering(a, b, c2, d);
                            anc.push(format!("L{kk}:{}", cur.count()));
                        }
                        let (a, b, c2, d) = cur.parent_texels();
                        let r = format!("{kmpx} cold jump to one level-{k} tile: ancestors {} (total {}); base-cell cone {}x{} cells", anc.join(" "), anc.iter().map(|s| s.split(':').nth(1).unwrap().parse::<usize>().unwrap()).sum::<usize>(), c2 - a, d - b);
                        println!("{r}");
                        lines.push(r);
                    }
                }
                frames.push((kern.tag().into(), zr));
            }
            for (name, f) in &frames {
                write_png(&format!("{out}/{kmpx}_{pname}_{name}.png"), f, FRAME, FRAME);
            }
            let order = [0usize, 1, 3, 4];
            let refs: Vec<&[u8]> = order.iter().map(|&i| frames[i].1.as_slice()).collect();
            let (sh, sw) = sheet(&refs, FRAME, FRAME);
            write_jpg(&format!("{out}/sheet_{kmpx}_{pname}_native_fg_zrbw_zref3.jpg"), &sh, sw, FRAME);
            let refs2: Vec<&[u8]> = [1usize, 2, 3].iter().map(|&i| frames[i].1.as_slice()).collect();
            let (sh2, sw2) = sheet(&refs2, FRAME, FRAME);
            write_jpg(&format!("{out}/sheet_{kmpx}_{pname}_fg_zrbase_zrbw.jpg"), &sh2, sw2, FRAME);
        }
    }
    std::fs::write(format!("{out}/zr0_bars.txt"), lines.join("\n") + "\n").unwrap();
}

/// **Per-step cost, one thread and all threads.** The 40 075 km world, the
/// mountain place, levels down to 0.078 km/px; every tile timed per step.
/// Runs the chain twice in a 1-thread rayon pool (both kernels) and once in
/// the default pool, and prints median (min..max) per step over land tiles at
/// levels >= 4, the colouring cost of one tile, and the level-8 mosaic digest
/// (the cross-device determinism check: compare PC and handset). Run alone
/// (`MISTAKES.md`, "Quote a timing"). `ZR_GW` as `zr0_bars`.
///
/// Protects: nothing (a measurement, `#[ignore]`d).
#[test]
#[ignore = "generates a world and times the refinement chain; run explicitly and alone"]
fn zr0_timing() {
    let gw: usize = std::env::var("ZR_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(2048);
    let t0 = Instant::now();
    let world = World::generate(1200, gw, gw / 2, 40075.0, true);
    println!("world generated in {:.1}s, field fnv {:016x}", t0.elapsed().as_secs_f64(), fnv(&world.ws.field));
    let wctx = world.ctx();
    let wtf = TileFields::new(&wctx, None);
    let coarse = Coarse::new(&world, &wctx);
    let l = world.l_km();
    let k = zr_view(l / KMPX_200).0;
    let wppc = l / KMPX_200;
    let wr = (FRAME as f64 / 2.0 / wppc).ceil() as usize + 1;
    let pl = places(&world, wr.max(3));
    let (_, gx, gy, _) = pl[0];
    let mut want = HashMap::new();
    want.insert(k, frame_want(k, gx, gy, wppc, 24));
    let rects = plan(&want, k);
    let one = rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap();
    for kern in [Kernel::Bw, Kernel::Ef3] {
        let mut all: Vec<Steps> = Vec::new();
        let mut digest = 0u64;
        for rep in 0..2 {
            let chain = one.install(|| run_chain(&coarse, &rects, k, kern, false));
            digest = fnv(&chain.last().unwrap().m.hgt);
            if rep == 1 {
                for lv in &chain {
                    if lv.m.k >= 4 {
                        all.extend(lv.steps.iter().filter(|s| s.land).copied());
                    }
                }
            }
        }
        assert!(!all.is_empty(), "no land tile was timed");
        println!("ZRTIME {} one thread, {} land tiles at levels 4..={k}, level-{k} digest {digest:016x}", kern.tag(), all.len());
        type Get = fn(&Steps) -> f64;
        let cols: [(&str, Get); 10] = [
            ("window", |s| s.window),
            ("upsample", |s| s.up),
            ("band", |s| s.band),
            ("fill", |s| s.fill),
            ("tile_flow", |s| s.flow),
            ("burn", |s| s.burn),
            ("erode", |s| s.erode),
            ("talus", |s| s.talus),
            ("taper+quant", |s| s.taper),
            ("whole chain", |s| s.total),
        ];
        for (name, get) in cols {
            let mut v: Vec<f64> = all.iter().map(get).collect();
            let (m, lo, hi) = median(&mut v);
            println!("ZRTIME {} {name:12} median {m:.2} ms ({lo:.2}..{hi:.2})", kern.tag());
        }
        // All threads, tiles in parallel: wall time per tile of the deepest level.
        let ts = Instant::now();
        let chain = run_chain(&coarse, &rects, k, kern, true);
        let el = ts.elapsed().as_secs_f64() * 1e3;
        let tiles: usize = chain.iter().map(|lv| lv.steps.len()).sum();
        println!("ZRTIME {} all threads ({}): {tiles} tiles in {el:.0} ms, {:.2} ms per tile wall; level-{k} digest {:016x}", kern.tag(), rayon::current_num_threads(), el / tiles as f64, fnv(&chain.last().unwrap().m.hgt));
        if kern == Kernel::Bw {
            // The colouring cost of one 256^2 refined tile, separated from the
            // height synthesis (scope §5 ZR-0), one thread.
            let m = &chain.last().unwrap().m;
            let sc = (1u64 << k) as f64;
            let tb_w = (TILE - 1) as f64 / sc;
            let pad = render::tile_halo_px(&wctx, TILE, TILE, TileBounds { x: 0.0, y: 0.0, w: tb_w, h: tb_w });
            let mut v = Vec::new();
            for q in 0..9i64 {
                let (tx0, ty0) = (m.x0 + TILE as i64 + (q % 3) * 64, m.y0 + TILE as i64 + (q / 3) * 64);
                let pw = TILE + 2 * pad;
                let mut data = vec![0f32; pw * pw];
                for y in 0..pw {
                    for x in 0..pw {
                        data[y * pw + x] = m.hgt[m.idx(tx0 - pad as i64 + x as i64, ty0 - pad as i64 + y as i64)];
                    }
                }
                let tb = TileBounds { x: (tx0 as f64 + 0.5) / sc - 0.5, y: (ty0 as f64 + 0.5) / sc - 0.5, w: tb_w, h: tb_w };
                let s = Instant::now();
                let rgba = one.install(|| render::render_biome_tile_rgba_water(&wctx, &data, None, TILE, TILE, pad, tb, &wtf, None));
                v.push(ms(s));
                assert_eq!(rgba.len(), TILE * TILE * 4);
            }
            let (m2, lo, hi) = median(&mut v);
            println!("ZRTIME colour one 256^2 tile (one thread) median {m2:.2} ms ({lo:.2}..{hi:.2}) over {}", v.len());
        }
    }
}

/// **Determinism across thread counts.** The 0.078 km/px mountain chain, both
/// kernels, tiles serial in a 1-thread pool and parallel in 4- and 16-thread
/// pools; prints each level's digest and whether all three agree.
///
/// Protects: nothing (a measurement, `#[ignore]`d).
#[test]
#[ignore = "generates a world and runs the refinement chain three times; run explicitly"]
fn zr0_determinism() {
    let gw: usize = std::env::var("ZR_GW").ok().and_then(|v| v.parse().ok()).unwrap_or(2048);
    let world = World::generate(1200, gw, gw / 2, 40075.0, true);
    let wctx = world.ctx();
    let coarse = Coarse::new(&world, &wctx);
    let l = world.l_km();
    let k = zr_view(l / KMPX_200).0;
    let wppc = l / KMPX_200;
    let pl = places(&world, ((FRAME as f64 / 2.0 / wppc).ceil() as usize + 1).max(3));
    let (_, gx, gy, _) = pl[0];
    let mut want = HashMap::new();
    want.insert(k, frame_want(k, gx, gy, wppc, 24));
    let rects = plan(&want, k);
    for kern in [Kernel::Bw, Kernel::Ef3] {
        let mut digests: Vec<Vec<u64>> = Vec::new();
        for (threads, par) in [(1usize, false), (4, true), (16, true)] {
            let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap();
            let chain = pool.install(|| run_chain(&coarse, &rects, k, kern, par));
            let d: Vec<u64> = chain.iter().map(|lv| fnv(&lv.m.hgt) ^ fnv(&lv.m.flow).rotate_left(1)).collect();
            println!("ZRDET {} threads {threads:2}: {}", kern.tag(), d.iter().map(|x| format!("{x:016x}")).collect::<Vec<_>>().join(" "));
            digests.push(d);
        }
        let same = digests.windows(2).all(|w| w[0] == w[1]);
        println!("ZRDET {} identical across 1/4/16 threads: {same}", kern.tag());
    }
}
