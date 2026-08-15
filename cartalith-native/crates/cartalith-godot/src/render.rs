//! Real default-settings 2D map rendering, ported from the reference HTML's
//! material-synthesis renderer (`materialWeights`/`landColorCore`/
//! `seaColorCore`, reference HTML lines ~7560-8370) — replaces the previous
//! placeholder hypsometric tint (`color_for_height`/`hillshade` in
//! `lib.rs`, removed).
//!
//! Presentation-only math for the 2D map view: no simulation logic, no new
//! subsystem crate, matching `ARCHITECTURE.md`'s existing precedent of the
//! old placeholder living directly in `cartalith-godot`.
//!
//! Deliberately excludes every `state.viz.*`-gated stretch feature the
//! reference renderer supports, all `0`/`false` at JS's own defaults so
//! omitting them changes nothing about the *default* view.
//!
//! Excluded: splat texturing, rockSlope refinement, wetness darkening,
//! geology microtexture and dune ripples, procedural texture synthesis,
//! ridged-relief creases, curvature shading, the paint-brush biome/terrain
//! override, the "Painter" NPR block (watercolor/contours/ink/hachure),
//! multi-sun hillshade, AO/SVF/shadow fields, and the coast/river SDF
//! tinting plus vector river overlay (the last two depend on subsystems
//! this port hasn't built yet; the existing simple river channel-mask tint
//! in `lib.rs` stays as this port's stand-in for "rivers visible",
//! `MVP_SCOPE.md`'s point 2).
//!
//! Ported despite being extras: the `bioBlend` grey-desaturation blend
//! (0.90 default) and the edge haze fade, both unconditional in the
//! reference at its own default settings.

use cartalith_noise::vnoise;

type Rgb = (f64, f64, f64);

const W_ABYSS: [Rgb; 3] = [(8.0, 36.0, 58.0), (10.0, 45.0, 70.0), (18.0, 59.0, 89.0)];
const W_DEEP: [Rgb; 3] = [(16.0, 58.0, 87.0), (26.0, 75.0, 104.0), (42.0, 96.0, 122.0)];
const W_SHELF: [Rgb; 3] = [(47.0, 118.0, 150.0), (76.0, 151.0, 182.0), (111.0, 179.0, 207.0)];
const W_TROP: [Rgb; 3] = [(88.0, 184.0, 181.0), (121.0, 206.0, 197.0), (149.0, 222.0, 210.0)];
const W_GLAC: [Rgb; 3] = [(127.0, 174.0, 190.0), (165.0, 197.0, 207.0), (194.0, 215.0, 222.0)];
const SAND_BEACH: [Rgb; 3] = [(200.0, 180.0, 138.0), (215.0, 195.0, 154.0), (227.0, 208.0, 167.0)];
const SAND_TROP: [Rgb; 3] = [(228.0, 212.0, 181.0), (239.0, 226.0, 197.0), (246.0, 234.0, 213.0)];
const SAND_DESERT: [Rgb; 3] = [(201.0, 169.0, 104.0), (215.0, 182.0, 118.0), (226.0, 197.0, 138.0)];
const SAND_RED: [Rgb; 3] = [(168.0, 101.0, 61.0), (191.0, 119.0, 75.0), (208.0, 137.0, 92.0)];
const GRASS_DRY: [Rgb; 3] = [(154.0, 138.0, 93.0), (176.0, 154.0, 106.0), (192.0, 171.0, 119.0)];
const GRASS_TEMP: [Rgb; 3] = [(127.0, 138.0, 86.0), (143.0, 155.0, 97.0), (162.0, 175.0, 112.0)];
const GRASS_BOREAL: [Rgb; 3] = [(102.0, 114.0, 79.0), (115.0, 128.0, 90.0), (133.0, 145.0, 107.0)];
const GRASS_SAV: [Rgb; 3] = [(181.0, 160.0, 94.0), (198.0, 176.0, 109.0), (216.0, 193.0, 128.0)];
const WOOD_TEMP: [Rgb; 3] = [(53.0, 65.0, 40.0), (66.0, 82.0, 50.0), (85.0, 104.0, 67.0)];
const WOOD_DENSE: [Rgb; 3] = [(40.0, 51.0, 31.0), (50.0, 64.0, 38.0), (64.0, 80.0, 48.0)];
const WOOD_BOREAL: [Rgb; 3] = [(47.0, 56.0, 44.0), (57.0, 68.0, 53.0), (70.0, 84.0, 69.0)];
const WOOD_TROP: [Rgb; 3] = [(29.0, 71.0, 37.0), (40.0, 96.0, 50.0), (52.0, 120.0, 63.0)];
const ROCK_GRANITE: [Rgb; 3] = [(123.0, 117.0, 108.0), (147.0, 139.0, 128.0), (170.0, 161.0, 149.0)];
const ROCK_SANDSTONE: [Rgb; 3] = [(167.0, 122.0, 87.0), (188.0, 141.0, 103.0), (208.0, 159.0, 118.0)];
const ROCK_SCREE: [Rgb; 3] = [(106.0, 102.0, 95.0), (122.0, 118.0, 110.0), (141.0, 137.0, 128.0)];
const SNOW_SEAS: [Rgb; 3] = [(217.0, 215.0, 210.0), (232.0, 231.0, 228.0), (245.0, 245.0, 245.0)];
const SNOW_PERM: [Rgb; 3] = [(237.0, 240.0, 242.0), (245.0, 247.0, 248.0), (252.0, 252.0, 252.0)];
const SNOW_GLAC: [Rgb; 3] = [(184.0, 210.0, 219.0), (203.0, 224.0, 230.0), (221.0, 236.0, 239.0)];
const WETLAND_TEMP: [Rgb; 3] = [(58.0, 72.0, 52.0), (72.0, 88.0, 63.0), (89.0, 108.0, 78.0)];
const WETLAND_TROP: [Rgb; 3] = [(46.0, 68.0, 44.0), (60.0, 86.0, 55.0), (76.0, 106.0, 68.0)];
const MANGROVE: [Rgb; 3] = [(38.0, 56.0, 42.0), (50.0, 72.0, 52.0), (64.0, 90.0, 65.0)];

/// `state.exag`/`state.sunAz`'s literal defaults (reference HTML line
/// 2260) — this port has no exposure/UI for either, so both are fixed at
/// their JS defaults rather than becoming new `WorldParams` knobs.
const EXAG: f64 = 3.4;
const SUN_AZ_DEG: f64 = 315.0;
/// `state.bioBlend`'s literal default (reference HTML line 2260) — the
/// grey-desaturation blend in `land_color` is unconditional at this value
/// (`blend < 1`), not a `state.viz`-gated stretch feature.
const BIO_BLEND: f64 = 0.90;

fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let d = b - a;
    let d = if d == 0.0 { 1e-6 } else { d };
    let t = clamp01((x - a) / d);
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    (lerp(a.0, b.0, t), lerp(a.1, b.1, t), lerp(a.2, b.2, t))
}

fn ramp3(p: &[Rgb; 3], t: f64) -> Rgb {
    let t = clamp01(t);
    if t < 0.5 { mix(p[0], p[1], t / 0.5) } else { mix(p[1], p[2], (t - 0.5) / 0.5) }
}

/// `boxH`/`boxV` (reference HTML lines 2511-2512) — separable box blur,
/// sliding-window accumulator. `f32` storage throughout (`dst` writes),
/// matching JS's `Float32Array` truncate-on-every-store semantics
/// (`cartalith-rust-conventions`).
fn box_h(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: i64, wrap: bool) {
    let norm = 1.0 / (2 * r + 1) as f64;
    for y in 0..h {
        let row = y * w;
        let mut acc = 0.0f64;
        let idx = |k: i64| -> usize {
            if wrap {
                (((k % w as i64) + w as i64) % w as i64) as usize
            } else {
                k.clamp(0, w as i64 - 1) as usize
            }
        };
        for k in -r..=r {
            acc += src[row + idx(k)] as f64;
        }
        for x in 0..w {
            dst[row + x] = (acc * norm) as f32;
            let o = idx(x as i64 - r);
            let i = idx(x as i64 + r + 1);
            acc += src[row + i] as f64 - src[row + o] as f64;
        }
    }
}

fn box_v(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: i64) {
    let norm = 1.0 / (2 * r + 1) as f64;
    let clamp_y = |k: i64| -> usize { k.clamp(0, h as i64 - 1) as usize };
    for x in 0..w {
        let mut acc = 0.0f64;
        for k in -r..=r {
            acc += src[clamp_y(k) * w + x] as f64;
        }
        for y in 0..h {
            dst[y * w + x] = (acc * norm) as f32;
            let o = clamp_y(y as i64 - r);
            let i = clamp_y(y as i64 + r + 1);
            acc += src[i * w + x] as f64 - src[o * w + x] as f64;
        }
    }
}

/// `smoothSeaH` (7966-7970) — two separable box passes, radius ∝
/// resolution, flatten the bathymetry into broad shelf/deep/abyss zones.
fn smooth_sea_h(src: &[f32], gw: usize, gh: usize, world: bool) -> Vec<f32> {
    let rad = ((gw as f64 / 200.0).round() as i64).max(1);
    let mut a = src.to_vec();
    let mut b = vec![0f32; src.len()];
    for _ in 0..2 {
        box_h(&a, &mut b, gw, gh, rad, world);
        box_v(&b, &mut a, gw, gh, rad);
    }
    a
}

/// `seaShadeFrom` (8112-8121) — single-sun hillshade of the smoothed
/// bathymetry, edge-clamped (never wraps, even in world mode, matching the
/// reference exactly).
fn sea_shade_from(hf: &[f32], gw: usize, gh: usize) -> Vec<f32> {
    let az = SUN_AZ_DEG.to_radians();
    let alt = 40.0_f64.to_radians();
    let (lx, ly, lz) = (alt.cos() * az.sin(), -alt.cos() * az.cos(), alt.sin());
    let mut out = vec![0f32; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let l = if x > 0 { hf[i - 1] } else { hf[i] } as f64;
            let r = if x + 1 < gw { hf[i + 1] } else { hf[i] } as f64;
            let u = if y > 0 { hf[i - gw] } else { hf[i] } as f64;
            let d = if y + 1 < gh { hf[i + gw] } else { hf[i] } as f64;
            let (nx, ny, nz) = (-(r - l) * EXAG, -(d - u) * EXAG, 1.0_f64);
            let il = 1.0 / nx.hypot(ny).hypot(nz);
            let (nx, ny, nz) = (nx * il, ny * il, nz * il);
            out[i] = (nx * lx + ny * ly + nz * lz).max(0.0) as f32;
        }
    }
    out
}

/// Everything the renderer needs about the last generated/loaded world.
/// `flow` is `None` for a loaded save (`SAVEFILE_COMPAT.md`'s save format
/// carries no flow field) — TWI-driven wetland placement falls back to the
/// driest case (`a` floored at its own `1e-4` minimum) rather than
/// guessing a value the save never stored.
pub struct RenderCtx<'a> {
    pub field: &'a [f32],
    pub temperature: &'a [f32],
    pub rainfall: &'a [f32],
    pub flow: Option<&'a [f32]>,
    pub gw: usize,
    pub gh: usize,
    pub sea_level: f64,
    pub world: bool,
    pub lat_n: f64,
    pub lat_s: f64,
    /// `smoothSeaH(field)` / `seaShadeFrom(_seaH)` (7966-8121) — `seaColor`
    /// reads these instead of the raw field/macro-shade whenever the app's
    /// default `state.mode==='biome'` map view is active (`renderNow`,
    /// 8422-8428), which is JS's own literal default (`mode:'biome'`, line
    /// 2260) so this isn't a stretch feature to skip: without it, shallow
    /// water reads with visible per-cell seabed noise the real app never
    /// shows. Computed once in `RenderCtx::new` rather than per cell.
    sea_h: Vec<f32>,
    sea_shade: Vec<f32>,
}

impl<'a> RenderCtx<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        field: &'a [f32],
        temperature: &'a [f32],
        rainfall: &'a [f32],
        flow: Option<&'a [f32]>,
        gw: usize,
        gh: usize,
        sea_level: f64,
        world: bool,
        lat_n: f64,
        lat_s: f64,
    ) -> Self {
        let sea_h = smooth_sea_h(field, gw, gh, world);
        let sea_shade = sea_shade_from(&sea_h, gw, gh);
        RenderCtx { field, temperature, rainfall, flow, gw, gh, sea_level, world, lat_n, lat_s, sea_h, sea_shade }
    }

    fn h(&self, x: usize, y: usize) -> f64 {
        self.field[y * self.gw + x] as f64
    }

    /// `latAt` (reference HTML line 4965).
    fn lat_at(&self, y: usize) -> f64 {
        if self.world {
            90.0 - (y as f64 / (self.gh.max(2) - 1) as f64) * 180.0
        } else {
            self.lat_n + (y as f64 / (self.gh.max(2) - 1) as f64) * (self.lat_s - self.lat_n)
        }
    }

    /// `slopeAt` (7584) — X wraps in world mode, Y never wraps.
    fn slope_at(&self, x: usize, y: usize) -> f64 {
        let (gw, gh) = (self.gw, self.gh);
        let (xl, xr) = if self.world {
            ((x + gw - 1) % gw, (x + 1) % gw)
        } else {
            (if x > 0 { x - 1 } else { x }, if x + 1 < gw { x + 1 } else { x })
        };
        let (yu, yd) = (if y > 0 { y - 1 } else { y }, if y + 1 < gh { y + 1 } else { y });
        let l = self.h(xl, y);
        let r = self.h(xr, y);
        let u = self.h(x, yu);
        let d = self.h(x, yd);
        ((r - l) * 0.5).hypot((d - u) * 0.5)
    }

    /// `vignetteAt` (7585).
    fn vignette_at(&self, x: usize, y: usize) -> f64 {
        let vx = x as f64 / (self.gw.max(2) - 1) as f64 - 0.5;
        let vy = y as f64 / (self.gh.max(2) - 1) as f64 - 0.5;
        1.0 - smoothstep(0.34, 0.74, vx.hypot(vy)) * 0.42
    }

    /// `aspectFactor` (7590) — never wraps, matching the reference exactly.
    fn aspect_factor(&self, x: usize, y: usize) -> f64 {
        let gh = self.gh;
        let u = if y > 0 { self.h(x, y - 1) } else { self.h(x, y) };
        let d = if y + 1 < gh { self.h(x, y + 1) } else { self.h(x, y) };
        let dzdy = (d - u) * 0.5;
        let lat = self.lat_at(y);
        if lat >= 0.0 { -dzdy } else { dzdy }
    }

    /// `curvatureAt` (7599) — clamps on both axes; unlike `slope_at` this
    /// never wraps even in world mode, matching the reference exactly.
    fn curvature_at(&self, x: usize, y: usize) -> f64 {
        let (gw, gh) = (self.gw, self.gh);
        let xl = if x > 0 { x - 1 } else { x };
        let xr = if x + 1 < gw { x + 1 } else { x };
        let yu = if y > 0 { y - 1 } else { y };
        let yd = if y + 1 < gh { y + 1 } else { y };
        self.h(xl, y) + self.h(xr, y) + self.h(x, yu) + self.h(x, yd) - 4.0 * self.h(x, y)
    }

    /// `shadeFactor` (8342, "macro") / `shadeFactor2` (7642, "meso") share
    /// the same single-sun light vector; `step` is 1 for macro, 3 for meso.
    fn shade(&self, x: usize, y: usize, step: usize) -> f64 {
        let (gw, gh) = (self.gw, self.gh);
        let xl = x.saturating_sub(step);
        let xr = (x + step).min(gw - 1);
        let yu = y.saturating_sub(step);
        let yd = (y + step).min(gh - 1);
        let l = self.h(xl, y);
        let r = self.h(xr, y);
        let u = self.h(x, yu);
        let d = self.h(x, yd);
        let ex = EXAG / step as f64;
        let dzdx = (r - l) * ex;
        let dzdy = (d - u) * ex;
        let (nx, ny, nz) = (-dzdx, -dzdy, 1.0_f64);
        let il = 1.0 / nx.hypot(ny).hypot(nz);
        let (nx, ny, nz) = (nx * il, ny * il, nz * il);
        let az = SUN_AZ_DEG.to_radians();
        let alt = 40.0_f64.to_radians();
        let (lx, ly, lz) = (alt.cos() * az.sin(), -alt.cos() * az.cos(), alt.sin());
        (nx * lx + ny * ly + nz * lz).max(0.0)
    }

    fn macro_shade(&self, x: usize, y: usize) -> f64 {
        self.shade(x, y, 1)
    }

    fn meso_shade(&self, x: usize, y: usize) -> f64 {
        self.shade(x, y, 3)
    }
}

/// `grassCol`/`forestCol`/`sandCol`/`rockCol`/`snowCol`/`wetlandCol`
/// (7632-7638).
fn grass_col(t: f64, m: f64, r: f64, tt: f64) -> Rgb {
    let c = if t < 4.0 {
        mix(ramp3(&GRASS_BOREAL, tt), ramp3(&GRASS_TEMP, tt), clamp01(m))
    } else if t > 22.0 && m < 0.4 {
        mix(ramp3(&GRASS_SAV, tt), ramp3(&GRASS_DRY, tt), clamp01(m * 2.0))
    } else {
        mix(ramp3(&GRASS_DRY, tt), ramp3(&GRASS_TEMP, tt), clamp01(m))
    };
    let d = 1.0 - r * 0.16;
    (c.0 * d, c.1 * d, c.2 * d)
}

fn forest_col(t: f64, m: f64, tt: f64) -> Rgb {
    if t < 3.0 {
        ramp3(&WOOD_BOREAL, tt)
    } else if t > 20.0 && m > 0.45 {
        ramp3(&WOOD_TROP, tt)
    } else if m > 0.62 {
        ramp3(&WOOD_DENSE, tt)
    } else {
        ramp3(&WOOD_TEMP, tt)
    }
}

fn sand_col(t: f64, m: f64, tt: f64) -> Rgb {
    if t > 24.0 && m < 0.1 { ramp3(&SAND_RED, tt) } else { ramp3(&SAND_DESERT, tt) }
}

fn rock_col(t: f64, m: f64, r: f64, tt: f64) -> Rgb {
    if r > 0.82 {
        ramp3(&ROCK_SCREE, tt)
    } else if t > 18.0 && m < 0.32 {
        ramp3(&ROCK_SANDSTONE, tt)
    } else {
        ramp3(&ROCK_GRANITE, tt)
    }
}

fn snow_col(t: f64, tt: f64) -> Rgb {
    if t < -12.0 {
        ramp3(&SNOW_GLAC, tt)
    } else if t < -4.0 {
        ramp3(&SNOW_PERM, tt)
    } else {
        ramp3(&SNOW_SEAS, tt)
    }
}

fn wetland_col(t: f64, mangrove: bool, tt: f64) -> Rgb {
    if mangrove { ramp3(&MANGROVE, tt) } else { ramp3(if t > 20.0 { &WETLAND_TROP } else { &WETLAND_TEMP }, tt) }
}

/// `materialWeights` (7655-7707) — the six material fractions, Σ=1.
struct Weights {
    snow: f64,
    rock: f64,
    sand: f64,
    wetland: f64,
    canopy: f64,
    grass: f64,
    c: f64,
    meff: f64,
    is_mangrove: bool,
}

fn material_weights(t: f64, m: f64, slope: f64, r: f64, twi: f64, asp: f64, curv: f64) -> Weights {
    let slope_str = (slope / 0.04).min(1.0);
    let asp_dry = clamp01(asp * slope_str * 0.22);
    let asp_wet = clamp01(-asp * slope_str * 0.12);

    let curv_norm = clamp01(curv.abs() * 300.0);
    let concave = if curv > 0.0 { curv_norm } else { 0.0 };
    let convex = if curv < 0.0 { curv_norm } else { 0.0 };

    let m_adj = clamp01(m - asp_dry + asp_wet + concave * 0.12);

    let fire = smoothstep(18.0, 26.0, t) * smoothstep(0.45, 0.15, m_adj) * smoothstep(0.08, 0.30, m_adj);

    let tn = clamp01((t + 5.0) / 35.0);
    let sl = (slope / 0.08).min(1.0);
    let sd0 = (-1.5 * sl).exp() * (0.4 + 0.6 * m_adj);
    let vp0 = m_adj.powf(0.7) * tn.powf(0.5) * sd0.max(0.0).powf(0.8);
    let c0 = 1.0 - (-2.0 * vp0).exp();

    let recycle = clamp01(0.1 + (t - 10.0).max(0.0) / 50.0);
    let meff = clamp01(m_adj + c0 * recycle * 0.5);
    let soil_d = (-1.5 * sl).exp() * (0.4 + 0.6 * meff);
    let vp_raw = meff.powf(0.7) * tn.powf(0.5) * soil_d.max(0.0).powf(0.8);
    let vp = vp_raw * (1.0 - fire * 0.40);
    let c = 1.0 - (-2.0 * vp).exp();

    let snow = smoothstep(3.0, -5.0, t);
    let mut bud = 1.0 - snow;

    let rexp = sl.powf(1.8) * (1.0 - vp) * (1.0 - meff) + convex * 0.25;
    let rock = clamp01(rexp * 0.8 + smoothstep(0.7, 0.95, r) * 0.35) * bud;
    bud -= rock;

    let sand = smoothstep(17.0, 26.0, t) * smoothstep(0.24, 0.05, meff) * (1.0 - vp * 0.7) * bud;
    bud -= sand;

    let mangrove_frac = smoothstep(18.0, 24.0, t) * smoothstep(0.08, 0.0, r) * smoothstep(0.10, 0.32, m_adj) * bud * 0.55;
    let wet_base = smoothstep(-1.0, 2.0, twi) * smoothstep(0.08, 0.28, m_adj) * smoothstep(0.06, 0.01, slope) * bud * 0.50;
    let wet_curv = concave * smoothstep(0.08, 0.28, m_adj) * bud * 0.22;
    let wetland = bud.min(mangrove_frac.max(wet_base + wet_curv));
    bud -= wetland;
    let is_mangrove = mangrove_frac > wet_base + wet_curv;

    let canopy = c * bud;
    bud -= canopy;
    let grass = bud.max(0.0);

    Weights { snow, rock, sand, wetland, canopy, grass, c, meff, is_mangrove }
}

/// `bioJitter` (7715-7719) at `state.viz.sharpBiomes`'s default (`true`).
fn bio_jitter(x: usize, y: usize, gw: usize) -> f64 {
    let (xf, yf) = (x as f64, y as f64);
    let gw = gw as f64;
    0.6 * vnoise(xf / gw * 44.0, yf / gw * 44.0, 31) + 0.4 * vnoise(xf / gw * 150.0, yf / gw * 150.0, 33)
}

/// `landColorCore`'s unconditional core (7720-7960): eco-jitter, the
/// six-material blend with canopy understory shadow, the beach rim, fine
/// noise grain, multi-scale hillshade, the `bioBlend` grey blend, the edge
/// haze fade, and the final `ao * vignette` multiply (7959-7960 — easy to
/// miss since it sits after the whole gated "Painter" NPR block, but is
/// itself unconditional; `ao` is fixed at `1.0` here, matching this port's
/// AO/SVF/shadow fields all being off). Every other `state.viz.*`-gated
/// extra is omitted — see this module's doc comment.
#[allow(clippy::too_many_arguments)]
fn land_color(t: f64, m: f64, slope: f64, r: f64, twi: f64, asp: f64, curv: f64, sh: f64, sh_m: f64, vig: f64, x: usize, y: usize, gw: usize, gh: usize) -> Rgb {
    let n_low = vnoise(x as f64 * 0.06, y as f64 * 0.06, 11);
    let n_hi = vnoise(x as f64 * 96.0 / gw as f64, y as f64 * 96.0 / gw as f64, 23);
    let n_bio = bio_jitter(x, y, gw);

    let te = t + (n_bio - 0.5) * 7.0 + (n_low - 0.5) * 2.5;
    let me = clamp01(m + (n_bio - 0.5) * 0.15 + (n_hi - 0.5) * 0.05);
    let twi_e = twi + (n_bio - 0.5) * 0.7;
    let asp_e = asp * (1.0 + (n_low - 0.5) * 0.3);

    let w = material_weights(te, me, slope, r, twi_e, asp_e, curv);
    let tt = clamp01(0.5 + (n_low - 0.5) * 1.1 + (n_hi - 0.5) * 0.5);

    let mut c = (0.0, 0.0, 0.0);
    let add = |c: &mut Rgb, m: Rgb, w: f64| {
        c.0 += m.0 * w;
        c.1 += m.1 * w;
        c.2 += m.2 * w;
    };
    add(&mut c, snow_col(te, tt), w.snow);
    add(&mut c, rock_col(te, me, r, tt), w.rock);
    add(&mut c, sand_col(te, me, tt), w.sand);
    add(&mut c, wetland_col(te, w.is_mangrove, tt), w.wetland);

    if w.canopy > 0.0 {
        let understory = smoothstep(0.70, 0.94, w.c) * w.canopy * 0.28;
        c.0 += 20.0 * understory;
        c.1 += 43.0 * understory;
        c.2 += 25.0 * understory;
        add(&mut c, forest_col(te, w.meff, tt), w.canopy - understory);
    }

    add(&mut c, grass_col(te, me, r, tt), w.grass);

    let beach_t = smoothstep(0.03, 0.0, r) * 0.6;
    if beach_t > 0.0 {
        let bc = ramp3(if te > 22.0 { &SAND_TROP } else { &SAND_BEACH }, tt);
        c.0 += (bc.0 - c.0) * beach_t;
        c.1 += (bc.1 - c.1) * beach_t;
        c.2 += (bc.2 - c.2) * beach_t;
    }

    let g = (n_hi - 0.5) * 9.0;
    c.0 += g;
    c.1 += g;
    c.2 += g;

    let sh_micro = clamp01(sh + (n_hi - 0.5) * 0.20);
    let sh_combined = 0.40 * sh + 0.40 * sh_m + 0.20 * sh_micro;
    let light = 0.45 + 1.02 * clamp01(sh_combined).powf(0.85);
    let mut l = (c.0 * light, c.1 * light, c.2 * light);
    if BIO_BLEND < 1.0 {
        let grey = 185.0 * light;
        l = (grey + (l.0 - grey) * BIO_BLEND, grey + (l.1 - grey) * BIO_BLEND, grey + (l.2 - grey) * BIO_BLEND);
    }

    let dx = x as f64 / gw as f64 - 0.5;
    let dy = y as f64 / gh as f64 - 0.5;
    let haze = clamp01(dx.hypot(dy) * 1.9).powf(2.2) * 0.18;
    let l = (l.0 + (208.0 - l.0) * haze, l.1 + (218.0 - l.1) * haze, l.2 + (230.0 - l.2) * haze);

    let ao = 1.0;
    let k = ao * vig;
    (l.0 * k, l.1 * k, l.2 * k)
}

/// `seaColorCore` (8122-8130).
fn sea_color_core(depth: f64, t: f64, n_low: f64, sh: f64, vig: f64) -> Rgb {
    let mut wc = if depth < 0.2 {
        ramp3(&W_SHELF, n_low)
    } else if depth < 0.55 {
        mix(ramp3(&W_SHELF, n_low), ramp3(&W_DEEP, n_low), (depth - 0.2) / 0.35)
    } else {
        mix(ramp3(&W_DEEP, n_low), ramp3(&W_ABYSS, n_low), (depth - 0.55) / 0.45)
    };
    if t > 22.0 {
        wc = mix(wc, ramp3(&W_TROP, n_low), smoothstep(22.0, 28.0, t) * (1.0 - depth) * 0.8);
    }
    if t < 5.0 {
        wc = mix(wc, ramp3(&W_GLAC, n_low), smoothstep(5.0, -3.0, t) * 0.7);
    }
    if t < -2.0 {
        wc = mix(wc, (226.0, 233.0, 239.0), clamp01((-2.0 - t) / 6.0) * 0.85);
    }
    let surf = smoothstep(0.03, 0.0, depth);
    if surf > 0.0 {
        wc = mix(wc, (176.0, 214.0, 221.0), surf * 0.5);
    }
    let tex = (n_low - 0.5) * 5.0;
    let sh2 = 0.82 + 0.18 * clamp01(sh);
    ((wc.0 + tex) * sh2 * vig, (wc.1 + tex) * sh2 * vig, (wc.2 + tex) * sh2 * vig)
}

/// Top-level per-cell colour, `[0,1]` per channel — `isWater(v) ?
/// seaColor(...) : surfaceColor(...)` (`debugBaseColor`'s `'biome'`
/// branch, 8204; the main renderer's own default mode).
pub fn cell_color(ctx: &RenderCtx, x: usize, y: usize) -> (f64, f64, f64) {
    let i = y * ctx.gw + x;
    let h = ctx.h(x, y);
    let t = ctx.temperature[i] as f64;

    let (r, g, b) = if h < ctx.sea_level {
        // `seaColor` (8277-8281) — reads the smoothed bathymetry/shade
        // (`ctx.sea_h`/`ctx.sea_shade`), not the raw field/macro-shade;
        // see `RenderCtx::new`'s doc comment on why that's the real
        // default, not a stretch feature.
        let hs = ctx.sea_h[i] as f64;
        let shw = ctx.sea_shade[i] as f64;
        let depth = if ctx.sea_level <= 0.0 { 0.0 } else { clamp01((ctx.sea_level - hs) / ctx.sea_level) };
        let n_low = vnoise(x as f64 * 25.6 / ctx.gw as f64, y as f64 * 25.6 / ctx.gw as f64, 5);
        sea_color_core(depth, t, n_low, shw, ctx.vignette_at(x, y))
    } else {
        // `surfaceColor` (8145-8196), unconditional parts only.
        let m = ctx.rainfall[i] as f64;
        let r_frac = if (1.0 - ctx.sea_level) <= 0.0 { 0.0 } else { (h - ctx.sea_level) / (1.0 - ctx.sea_level) };
        let slope = ctx.slope_at(x, y);
        let flow = ctx.flow.map(|f| f[i] as f64).unwrap_or(0.0);
        let a = (flow / (ctx.gw * ctx.gh) as f64).max(1e-4);
        let beta = slope.max(0.002);
        let twi = (a / beta).ln();
        let asp = ctx.aspect_factor(x, y);
        let curv = ctx.curvature_at(x, y);
        land_color(t, m, slope, r_frac, twi, asp, curv, ctx.macro_shade(x, y), ctx.meso_shade(x, y), ctx.vignette_at(x, y), x, y, ctx.gw, ctx.gh)
    };

    (clamp01(r / 255.0), clamp01(g / 255.0), clamp01(b / 255.0))
}
