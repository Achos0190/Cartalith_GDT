//! Emissive paint **materials** for the colour-paint layer: lava and its
//! glowing relatives (`COLOUR_PAINT_SCOPE.md` §17, the owner's 2026-10-18
//! feedback on CP-1: *"Currently it doesn't look convincing at all for
//! lava/magma"*).
//!
//! # What it is
//!
//! A mark with an [`EmissiveMaterial`] is not a colour but a small procedural
//! surface, evaluated per output pixel from the mark's own geometry and the
//! pixel's kilometre position:
//!
//! - a **temperature** field `t` in `0..=1`: hottest deep inside the mark
//!   (a stroke's centre line, an area's interior), cooling toward the rim, cooling
//!   along a stroke when `heat_along > 0` (a flow cools away from its vent), and
//!   broken up by low-frequency noise in kilometres so the hot core is not a
//!   ring;
//! - a dark **crust** (the mark's `rgb`, near-black by default) broken into
//!   plates by a multi-octave cellular crack network whose cell sizes are in
//!   kilometres, so zooming in keeps revealing finer cracks;
//! - **glowing cracks and molten cores**: where the crust is open, an emissive
//!   colour from a temperature ramp (white-yellow, orange, red, dark red,
//!   black) is ADDED after the light, so the hillshade and ambient occlusion
//!   never darken the glow;
//! - a soft **halo**: emissive light spilling a little onto the ground around
//!   the mark, falling off smoothly over `glow_radius_km`.
//!
//! # Why this shape (what real lava looks like)
//!
//! Active lava seen from above is not red paint. Its surface is a chilled,
//! near-black, slightly blue-grey crust broken into plates, with the incandescent
//! interior visible only in the cracks between them and at vents and
//! breakouts. Colour is temperature: dull red around 600-700 C, orange near
//! 900-1000 C, yellow to white above 1100 C (incandescence; the Draper point
//! and blackbody colour, e.g. Lockwood and Hazlett, *Volcanoes: Global
//! Perspectives*, 2010, ch. 6 on lava surfaces; the ramp's stops are a labelled
//! judgement made by eye, not fitted to a spectrum). In daylight the crust
//! reads dark and the cracks orange; at night the cracks dominate and the
//! surroundings pick up a red glow. The crack network is self-similar over a
//! wide range of scales (metre-scale plates on a pahoehoe sheet, centimetre
//! cracks within a plate), which is what lets it keep gaining detail at zoom.
//!
//! # The zoom law
//!
//! An octave of cracks with cell size `L` is drawn only once `L` spans
//! [`NYQ_PX`] output pixels, fading in over one doubling (the canopy texture's
//! rule, `render.rs::CANOPY_NYQ_PX`). An octave not yet drawn is not dropped:
//! it contributes its **mean** open fraction, so the overall brightness of the
//! flow is the same at x1 and at x64 and only the structure resolves --
//! nothing pops when an octave arrives.
//!
//! # What it must never do
//!
//! - Never depend on anything but the kilometre position, the pixel footprint
//!   and the mark: the screen, the deep-zoom tiles and every band of a banded
//!   export evaluate the same pure function, so they agree at equal pixel size.
//! - Never read a screen-space neighbourhood (no blur): the halo is computed
//!   per pixel from the distance to the mark, with the mark's reach widened by
//!   the halo radius in the spatial index.
//! - Never animate. Static by scope (the owner's brief, item 5).

use serde::{Deserialize, Serialize};

/// Which colour family the temperature ramp runs through. The mechanism
/// (crust, cracks, halo) is the same for each; only the ramp differs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GlowRamp {
    /// Incandescence: black, dark red, red, orange, yellow, white-yellow.
    /// Lava, magma, embers.
    #[default]
    Fire,
    /// A cold-light fantasy ramp: black, deep blue, azure, ice-white. Not a
    /// physical material; it falls out of the same mechanism for free.
    Ice,
    /// A fantasy ramp: black, deep green, acid green, pale yellow-green.
    Toxic,
}

impl GlowRamp {
    /// Every ramp, in the order a picker lists them.
    pub const ALL: [GlowRamp; 3] = [GlowRamp::Fire, GlowRamp::Ice, GlowRamp::Toxic];

    /// The ramp's stops, `(t, rgb)` with `rgb` an **emissive** amount on the
    /// renderer's 0-255 scale (added to the lit colour). Labelled judgement,
    /// chosen by eye on the 2026-10-18 frames against photographs of active
    /// pahoehoe and lava lakes; the Fire stops follow blackbody order (dark
    /// red, orange, yellow, white) without claiming a calibrated temperature.
    fn stops(self) -> &'static [(f64, [f64; 3])] {
        match self {
            GlowRamp::Fire => &[
                (0.00, [0.0, 0.0, 0.0]),
                (0.18, [55.0, 3.0, 0.0]),
                (0.38, [150.0, 18.0, 0.0]),
                (0.58, [235.0, 68.0, 4.0]),
                (0.76, [255.0, 138.0, 18.0]),
                (0.90, [255.0, 205.0, 80.0]),
                (1.00, [255.0, 248.0, 200.0]),
            ],
            GlowRamp::Ice => &[
                (0.00, [0.0, 0.0, 0.0]),
                (0.25, [0.0, 22.0, 70.0]),
                (0.55, [20.0, 110.0, 225.0]),
                (0.80, [120.0, 215.0, 255.0]),
                (1.00, [235.0, 252.0, 255.0]),
            ],
            GlowRamp::Toxic => &[
                (0.00, [0.0, 0.0, 0.0]),
                (0.25, [12.0, 50.0, 0.0]),
                (0.55, [70.0, 185.0, 8.0]),
                (0.80, [175.0, 255.0, 55.0]),
                (1.00, [240.0, 255.0, 200.0]),
            ],
        }
    }

    /// The emissive colour at temperature `t` (`0..=1`, clamped), linear
    /// between stops.
    pub fn at(self, t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        let s = self.stops();
        for w in s.windows(2) {
            let ((t0, c0), (t1, c1)) = (w[0], w[1]);
            if t <= t1 {
                let k = if t1 > t0 { (t - t0) / (t1 - t0) } else { 1.0 };
                return [c0[0] + (c1[0] - c0[0]) * k, c0[1] + (c1[1] - c0[1]) * k, c0[2] + (c1[2] - c0[2]) * k];
            }
        }
        s[s.len() - 1].1
    }
}

/// A mark's emissive material. Present on a mark ([`super::ColourMark::emissive`])
/// it replaces the flat colour with the procedural surface this module
/// describes; the mark's `rgb` becomes the **crust** colour (what is left when
/// the lava has cooled), its `blend` how that crust combines with the ground
/// before the light, its `opacity` scales both crust and glow.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmissiveMaterial {
    #[serde(default)]
    pub ramp: GlowRamp,
    /// `0..=1`: how much of the surface is open and hot. `0` is a cooled,
    /// all-crust flow with faint red seams; `1` is a lava lake, molten and
    /// yellow-white over most of its interior.
    pub heat: f64,
    /// `0..=1`: plate size, coarse (`0`, plates [`PLATE_KM_COARSE`] across)
    /// to fine (`1`, [`PLATE_KM_FINE`]), geometric between.
    pub crack_density: f64,
    /// The halo's reach beyond the mark's edge, in km. `0` turns it off.
    pub glow_radius_km: f64,
    /// `0..=1`, strokes only: how much the heat falls from the stroke's first
    /// point to its last (a flow cooling away from its vent). Ignored by areas.
    #[serde(default)]
    pub heat_along: f64,
}

/// The coarsest plate size (`crack_density == 0`), km. Labelled judgement:
/// at the app's default 800 km map a 6 km crater then holds a handful of
/// plates at x16, which reads as a crusted lake rather than a texture.
pub const PLATE_KM_COARSE: f64 = 2.0;
/// The finest top-octave plate size (`crack_density == 1`), km. Labelled
/// judgement.
pub const PLATE_KM_FINE: f64 = 0.25;
/// Ratio between neighbouring crack octaves' cell sizes. Labelled judgement:
/// not a power of two so octaves never share cell corners.
pub const LACUNARITY: f64 = 2.3;
/// The finest crack cell drawn, km (4 m): a plate on a pahoehoe sheet is
/// decimetres to metres across, and nothing in this product draws finer than
/// a few metres a pixel. Labelled judgement.
pub const MIN_CELL_KM: f64 = 0.004;
/// Most crack octaves. With the defaults (0.71 km top cell) the floor stops
/// the sum at seven; the cap only bounds a pathological density.
pub const MAX_OCTAVES: usize = 9;
/// An octave is drawn once its cell spans this many output pixels, fading in
/// over one doubling: the canopy texture's Nyquist rule
/// (`render.rs::CANOPY_NYQ_PX`, the zoom-detail law), restated here because
/// this crate cannot see the renderer.
pub const NYQ_PX: f64 = 2.5;
/// Half-width of a crack as a fraction of its cell, cold and hot ends:
/// `CRACK_COLD + CRACK_HOT * t^1.5`. Labelled judgement: thin seams on a
/// cool crust, wide open leads on a hot one.
const CRACK_COLD: f64 = 0.012;
const CRACK_HOT: f64 = 0.17;
/// Each finer octave's cracks are this much narrower (relative to their
/// cell) and dimmer than the one above: secondary fractures inside a plate
/// are thinner and cooler than the seams between plates. Labelled judgement.
const OCTAVE_FADE: f64 = 0.72;
/// How much hotter (in `t`) the crust must be for each finer octave of
/// fractures to open. Labelled judgement: without it every octave opened at
/// the same temperature and a cooling crust read as an even mesh (the third
/// set of lava frames, 2026-10-18), where real crust shows a few wide seams
/// between large plates and breaks up finely only where it is hot.
const CRACK_ON_STEP: f64 = 0.08;
/// The mean of the finer octaves' `present` mask (`smoothstep(0.38, 0.62, n)`
/// over value noise `n`, symmetric about one half): the fraction of their
/// edges a finer octave draws, which its not-yet-drawn mean must carry too.
const BROKEN_MEAN: f64 = 0.5;
/// The halo's strength at the mark's edge, as a fraction of the ramp colour
/// at the edge's temperature. Labelled judgement: enough to warm the ground a
/// little, never enough to read as more lava.
const HALO_GAIN: f64 = 0.9;
/// Glow seen through a hot crust (dull red, before the crust has fully
/// chilled), as a fraction of the ramp at half the temperature. Labelled
/// judgement.
const CRUST_GLOW: f64 = 0.35;

/// The edge irregularity's longest wavelength, in interior depths. Labelled
/// judgement: lobes a little longer than the flow is wide.
const EDGE_WAVE: f64 = 1.6;
/// Each edge octave's amplitude as a fraction of its wavelength (so the sum
/// is bounded by `2 * EDGE_AMP * EDGE_WAVE * depth`). Labelled judgement: at
/// 0.16 a flow's margin wanders by about a third of its radius.
const EDGE_AMP: f64 = 0.16;

/// One mark's material, resolved once when the layer is built.
#[derive(Clone, Debug)]
pub(super) struct Resolved {
    pub ramp: GlowRamp,
    pub heat: f64,
    pub plate_km: f64,
    pub glow_km: f64,
    pub heat_along: f64,
    /// Interior depth, km, over which the temperature rises from the rim to
    /// the core: a stroke's radius; an area's equivalent radius halved.
    pub depth_km: f64,
}

impl Resolved {
    /// Clamps and resolves `e` for a mark whose interior depth is `depth_km`.
    /// `None` for a non-finite value (the caller refuses the mark).
    pub fn new(e: &EmissiveMaterial, depth_km: f64) -> Option<Resolved> {
        if !(e.heat.is_finite() && e.crack_density.is_finite() && e.glow_radius_km.is_finite() && e.heat_along.is_finite()) {
            return None;
        }
        let d = e.crack_density.clamp(0.0, 1.0);
        Some(Resolved {
            ramp: e.ramp,
            heat: e.heat.clamp(0.0, 1.0),
            plate_km: PLATE_KM_COARSE * (PLATE_KM_FINE / PLATE_KM_COARSE).powf(d),
            glow_km: e.glow_radius_km.max(0.0),
            heat_along: e.heat_along.clamp(0.0, 1.0),
            depth_km: depth_km.max(1e-6),
        })
    }

    /// The longest wavelength of the edge's irregularity, km: the interior
    /// depth times [`EDGE_WAVE`].
    fn edge_wl0(&self) -> f64 {
        self.depth_km * EDGE_WAVE
    }

    /// The most the edge can move, either way, km: the octave amplitudes'
    /// sum bound (`EDGE_AMP` of each wavelength, halving), which the spatial
    /// index adds to the mark's reach.
    pub fn edge_reach(&self) -> f64 {
        2.0 * EDGE_AMP * self.edge_wl0()
    }

    /// How far the mark's edge is pushed out (positive) or pulled in at `p`,
    /// km, for a pixel `px` km wide: lobes and embayments in kilometres, so a
    /// flow's margin is ragged rather than a pen stroke's parallel offset
    /// (the first lava frames' flow read as a pipe). Octaves halve in
    /// wavelength and amplitude down to [`MIN_CELL_KM`], each faded in by the
    /// zoom law ([`NYQ_PX`]) -- a zero-mean octave not yet drawn contributes
    /// nothing, so the edge only gains detail as it zooms.
    pub fn edge_offset(&self, p: [f64; 2], px: f64) -> f64 {
        let mut wl = self.edge_wl0();
        let mut sum = 0.0;
        for o in 0..MAX_OCTAVES {
            if wl < MIN_CELL_KM {
                break;
            }
            let vis = if px > 0.0 { smoothstep(NYQ_PX * px, 2.0 * NYQ_PX * px, wl) } else { 1.0 };
            if vis <= 0.0 {
                break;
            }
            let (rx, ry) = ROT[(o + 1) % ROT.len()];
            let q = [p[0] * rx - p[1] * ry, p[0] * ry + p[1] * rx];
            sum += (value_noise(q, wl, 0xED6E + o as u32 * 0x95) - 0.5) * 2.0 * EDGE_AMP * wl * vis;
            wl *= 0.5;
        }
        sum
    }

    /// The mark's own contribution to the temperature `s` km inside its edge
    /// (negative outside), `u` of the way along a stroke: its heat, cooled
    /// along the stroke by `heat_along`, rising from the rim to the core.
    /// Several overlapping lava marks take the **maximum** of this (they are
    /// one body of lava, `super::LavaGroup`), so a stroke laid over a fill
    /// does not draw a cool seam where its own rim falls.
    pub fn base_heat(&self, s: f64, u: f64) -> f64 {
        let h = self.heat * (1.0 - self.heat_along * u);
        let prof = (s / self.depth_km).clamp(0.0, 1.0).powf(0.8);
        h * (0.15 + 0.85 * prof)
    }

    /// The temperature at `p` from a base heat ([`Resolved::base_heat`]):
    /// low-frequency variation in km is added -- hot channels and cooler
    /// rafts, at three plate sizes and up, never pixel-gated because its
    /// finest cell is larger than any crack.
    fn temperature(&self, p: [f64; 2], base: f64) -> f64 {
        let n = fbm(p, self.plate_km * 3.0, 2, 0x51A7) * 2.0 - 1.0;
        (base + 0.24 * n * (0.4 + 0.6 * base.min(1.0))).clamp(0.0, 1.0)
    }

    /// The material at `p` (km) for a pixel whose base heat is `base`
    /// ([`Resolved::base_heat`]) and which is `px` km wide: the crust colour
    /// multiplier (around 1, applied to the mark's `rgb`) and the emissive
    /// colour (0-255, added after the light). Pure: a function of its
    /// arguments only.
    pub fn sample(&self, p: [f64; 2], base: f64, px: f64) -> (f64, [f64; 3]) {
        let t = self.temperature(p, base);
        // Domain warp at the top plate size, so seams wander rather than
        // run straight between jittered points.
        let wl = self.plate_km * 1.7;
        let wp = [p[0] + (fbm(p, wl, 2, 0x3C61) - 0.5) * 0.7 * self.plate_km, p[1] + (fbm(p, wl, 2, 0x9E15) - 0.5) * 0.7 * self.plate_km];
        // Primary seams open at a low temperature; each finer octave of
        // fractures needs a hotter crust ([`CRACK_ON_STEP`] more per octave),
        // so a cool crust shows a few seams and a hot one breaks up finely.
        let crack_on = |o: usize| smoothstep(0.06 + CRACK_ON_STEP * o as f64, 0.32 + CRACK_ON_STEP * o as f64, t);
        let mut keep = 1.0; // product of (1 - crack coverage) over octaves
        let mut crust_k = 1.0;
        let mut l = self.plate_km;
        let mut width = CRACK_COLD + CRACK_HOT * t.powf(1.5);
        let mut bright = 1.0;
        for o in 0..MAX_OCTAVES {
            if l < MIN_CELL_KM {
                break;
            }
            let hw = l * width; // crack half-width, km
            // The octave's mean open fraction: a band of half-width hw along
            // the edges of cells of size l covers about 2 * 1.9 * hw / l of
            // the area (hexagon-like cells have perimeter/area ~ 3.8 / l).
            // Finer octaves are drawn only over about half their edges (the
            // `present` mask below), so their mean is halved with them.
            let mean = (3.8 * hw / l).min(1.0) * if o == 0 { 1.0 } else { BROKEN_MEAN };
            let vis = if px > 0.0 { smoothstep(NYQ_PX * px, 2.0 * NYQ_PX * px, l) } else { 1.0 };
            let c = if vis > 0.0 {
                let (rx, ry) = ROT[o % ROT.len()];
                let q = [(wp[0] * rx - wp[1] * ry) / l, (wp[0] * ry + wp[1] * rx) / l];
                let (edge, id) = voronoi_edge(q, 0x7F4A + o as u32 * 0x1F1D);
                let ed = edge * l;
                // Secondary fractures are discontinuous and uneven: a km-keyed
                // mask lets each finer octave's crack run only part of the way
                // along its cell edges, and every crack's width wanders along
                // its length (mean 1). Without these the octaves nested into
                // even polygons inside polygons -- the fourth set of lava
                // frames read as a stylised mesh, not as fractured crust.
                let present = if o == 0 { 1.0 } else { smoothstep(0.38, 0.62, value_noise(wp, l * 1.8, 0xB40C + o as u32)) };
                let hw = hw * (0.55 + 0.9 * value_noise(wp, l * 0.9, 0x1D17 + o as u32));
                // The crack band, antialiased over one pixel and energy-kept:
                // a crack thinner than a pixel covers it in proportion.
                let we = hw.max(px * 0.5);
                let band = 1.0 - smoothstep(we - px * 0.5, we + px * 0.5, ed);
                let drawn = band * (hw / we).min(1.0) * present;
                // Plate-to-plate albedo variation, only once the plate shows.
                crust_k *= 1.0 + 0.22 * (unit(id) - 0.5) * vis * bright;
                drawn * vis + mean * (1.0 - vis)
            } else {
                mean
            };
            keep *= 1.0 - (c * bright * crack_on(o)).min(1.0);
            l /= LACUNARITY;
            width *= OCTAVE_FADE;
            bright *= OCTAVE_FADE;
        }
        let cracks = 1.0 - keep;
        let molten = smoothstep(0.80, 0.98, t);
        let open = 1.0 - (1.0 - cracks) * (1.0 - molten);
        let hot = self.ramp.at((t * 1.06).min(1.0));
        let dull = self.ramp.at(t * 0.5);
        let dull_k = CRUST_GLOW * smoothstep(0.35, 0.85, t) * (1.0 - open);
        let e = [hot[0] * open + dull[0] * dull_k, hot[1] * open + dull[1] * dull_k, hot[2] * open + dull[2] * dull_k];
        (crust_k, e)
    }

    /// The halo's emissive colour `o` km outside the mark's edge (`o > 0`), for
    /// a mark whose edge heat at the nearest point is `u` along a stroke.
    pub fn halo(&self, o: f64, u: f64) -> [f64; 3] {
        if self.glow_km <= 0.0 || o <= 0.0 || o >= self.glow_km {
            return [0.0; 3];
        }
        let h = self.heat * (1.0 - self.heat_along * u);
        let f = 1.0 - o / self.glow_km;
        let k = HALO_GAIN * f * f * f * (0.25 + 0.75 * h);
        let c = self.ramp.at(0.35 + 0.35 * h);
        [c[0] * k, c[1] * k, c[2] * k]
    }
}

/// The emissive material evaluated **without a mark**: the same pure
/// functions a lava mark uses (edge irregularity, rim-to-core heat, crust and
/// crack network, halo), for a caller that places the material itself from
/// the world model -- VA-4's procedural crater glow on active volcanoes
/// (`VOLCANO_APPEARANCE_RESEARCH.md` §7, owner Ruling CW: "reuse the
/// colour-paint lava material approach"). A thin public face over the
/// layer's private resolved material, so the two can never drift apart: one
/// crack network, one zoom law ([`NYQ_PX`]), one ramp.
///
/// Must never be used to change what a user's mark draws: it holds its own
/// copy and touches no layer.
#[derive(Clone, Debug)]
pub struct ProceduralLava(Resolved);

impl ProceduralLava {
    /// `e` resolved for a body whose interior depth (rim to core) is
    /// `depth_km`. `None` for a non-finite parameter, as a mark is refused.
    pub fn new(e: &EmissiveMaterial, depth_km: f64) -> Option<ProceduralLava> {
        Resolved::new(e, depth_km).map(ProceduralLava)
    }

    /// [`Resolved::edge_offset`]: how far the edge is pushed out at `p` (km)
    /// for a pixel `px` km wide.
    pub fn edge_offset(&self, p: [f64; 2], px: f64) -> f64 {
        self.0.edge_offset(p, px)
    }

    /// [`Resolved::base_heat`] at `s` km inside the edge (`u` = 0: no cooling
    /// along a stroke).
    pub fn base_heat(&self, s: f64) -> f64 {
        self.0.base_heat(s, 0.0)
    }

    /// [`Resolved::sample`]: the crust multiplier and the emissive colour
    /// (0-255, added after the light) at `p` for a pixel `px` km wide.
    pub fn sample(&self, p: [f64; 2], base: f64, px: f64) -> (f64, [f64; 3]) {
        self.0.sample(p, base, px)
    }

    /// [`Resolved::halo`] `o` km outside the edge.
    pub fn halo(&self, o: f64) -> [f64; 3] {
        self.0.halo(o, 0.0)
    }
}

/// Per-octave lattice rotations `(cos, sin)`, away from the grid axes and the
/// diagonals so no octave's cells line up with another's or with the raster.
const ROT: [(f64, f64); 4] = [(0.923_879_5, 0.382_683_4), (0.382_683_4, 0.923_879_5), (-0.382_683_4, 0.923_879_5), (-0.923_879_5, 0.382_683_4)];

/// `smoothstep(e0, e1, x)`; a step at `e0` when the edges coincide.
pub(super) fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    if e1 <= e0 {
        return if x >= e0 { 1.0 } else { 0.0 };
    }
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A 32-bit hash of a lattice cell and a salt (SplitMix64's finaliser over a
/// mixed key). Deterministic on every platform: integer arithmetic only.
fn hash2(ix: i64, iy: i64, salt: u32) -> u32 {
    let mut h = (ix as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (iy as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ (salt as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    (h >> 32) as u32
}

/// `0..1` from a hash.
fn unit(h: u32) -> f64 {
    h as f64 / 4_294_967_296.0
}

/// Value noise in `0..1` at `p / cell` (km over km), smoothstep-interpolated.
fn value_noise(p: [f64; 2], cell: f64, salt: u32) -> f64 {
    let (x, y) = (p[0] / cell, p[1] / cell);
    let (fx, fy) = (x.floor(), y.floor());
    let (ix, iy) = (fx as i64, fy as i64);
    let (tx, ty) = (x - fx, y - fy);
    let (sx, sy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
    let v = |dx: i64, dy: i64| unit(hash2(ix + dx, iy + dy, salt));
    let a = v(0, 0) + (v(1, 0) - v(0, 0)) * sx;
    let b = v(0, 1) + (v(1, 1) - v(0, 1)) * sx;
    a + (b - a) * sy
}

/// `octaves` of value noise from `cell` km down by halves, normalised to `0..1`.
fn fbm(p: [f64; 2], cell: f64, octaves: usize, salt: u32) -> f64 {
    let (mut sum, mut amp, mut norm, mut c) = (0.0, 1.0, 0.0, cell);
    for o in 0..octaves {
        sum += value_noise(p, c, salt.wrapping_add(o as u32 * 0x2F)) * amp;
        norm += amp;
        amp *= 0.5;
        c *= 0.5;
    }
    sum / norm
}

/// The jittered feature point of lattice cell `(ix, iy)`, in cell units.
fn feature(ix: i64, iy: i64, salt: u32) -> [f64; 2] {
    let h = hash2(ix, iy, salt);
    let h2 = hash2(ix, iy, salt ^ 0xA5A5_5A5A);
    [ix as f64 + 0.1 + 0.8 * unit(h), iy as f64 + 0.1 + 0.8 * unit(h2)]
}

/// Distance (in cell units) from `q` to the nearest Voronoi edge of jittered
/// lattice points, and the hash of the cell `q` lies in. Two passes over the
/// 3 x 3 neighbourhood: the nearest point, then the minimum over the other
/// points of the distance to the bisector between it and the nearest (Inigo
/// Quilez's "Voronoi edges" construction, 2012, restricted to 3 x 3 -- an
/// approximation near far-reaching cells, invisible at crack widths).
fn voronoi_edge(q: [f64; 2], salt: u32) -> (f64, u32) {
    let (bx, by) = (q[0].floor() as i64, q[1].floor() as i64);
    let mut best = (f64::INFINITY, [0.0, 0.0], 0i64, 0i64);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let f = feature(bx + dx, by + dy, salt);
            let d = (f[0] - q[0]).powi(2) + (f[1] - q[1]).powi(2);
            if d < best.0 {
                best = (d, f, bx + dx, by + dy);
            }
        }
    }
    let (a, ax, ay) = (best.1, best.2, best.3);
    let mut edge = f64::INFINITY;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (cx, cy) = (ax + dx, ay + dy);
            if cx == ax && cy == ay {
                continue;
            }
            let b = feature(cx, cy, salt);
            let (nx, ny) = (b[0] - a[0], b[1] - a[1]);
            let len = (nx * nx + ny * ny).sqrt();
            if len <= 1e-12 {
                continue;
            }
            let m = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
            let d = ((m[0] - q[0]) * nx + (m[1] - q[1]) * ny) / len;
            edge = edge.min(d);
        }
    }
    (edge.max(0.0), hash2(ax, ay, salt ^ 0x00C0_FFEE))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lava(heat: f64) -> Resolved {
        Resolved::new(&EmissiveMaterial { ramp: GlowRamp::Fire, heat, crack_density: 0.5, glow_radius_km: 2.0, heat_along: 0.0 }, 3.0).unwrap()
    }

    /// Protects: the Fire ramp's order against literal stops -- black at 0,
    /// red before orange before yellow, white-yellow at 1 -- so a reordered or
    /// mis-typed stop is caught (the look depends on this order).
    #[test]
    fn fire_ramp_runs_black_red_orange_yellow_white() {
        let r = GlowRamp::Fire;
        assert_eq!(r.at(0.0), [0.0, 0.0, 0.0]);
        assert_eq!(r.at(1.0), [255.0, 248.0, 200.0]);
        let red = r.at(0.38);
        let orange = r.at(0.76);
        let yellow = r.at(0.9);
        assert!(red[0] > 4.0 * red[1], "red: {red:?}");
        assert!(orange[1] > red[1] && orange[1] < yellow[1], "{orange:?}");
        assert!(yellow[2] > orange[2], "{yellow:?}");
        assert_eq!(r.at(-1.0), r.at(0.0));
        assert_eq!(r.at(2.0), r.at(1.0));
    }

    /// Protects: the temperature is hottest inside and cools to the rim, and
    /// cools along a stroke with `heat_along` -- the "hot core inside" law.
    #[test]
    fn temperature_falls_toward_the_rim_and_along_the_flow() {
        let m = lava(0.8);
        let mean = |s: f64, u: f64, m: &Resolved| (0..400).map(|k| m.temperature([k as f64 * 0.37, k as f64 * 0.11], m.base_heat(s, u))).sum::<f64>() / 400.0;
        let (core, rim) = (mean(3.0, 0.0, &m), mean(0.05, 0.0, &m));
        assert!(core > rim + 0.4, "core {core} rim {rim}");
        let mut f = m.clone();
        f.heat_along = 1.0;
        let (vent, toe) = (mean(3.0, 0.0, &f), mean(3.0, 1.0, &f));
        assert!(vent > toe + 0.5, "vent {vent} toe {toe}");
    }

    /// Protects: the zoom law's energy rule -- the mean open (glowing)
    /// fraction is the same whether the octaves are drawn (a fine pixel) or
    /// replaced by their means (a coarse pixel), within a tolerance, so the
    /// flow's brightness does not jump with zoom. And the fine pixel has
    /// STRUCTURE (cracks and plates differ) where the coarse one is flat.
    #[test]
    fn brightness_is_kept_across_zoom_and_detail_appears() {
        let m = lava(0.55);
        let stats = |px: f64| {
            let v: Vec<f64> = (0..4000).map(|k| m.sample([(k % 80) as f64 * 0.0731 + 10.0, (k / 80) as f64 * 0.0657 + 10.0], m.base_heat(3.0, 0.0), px).1[0]).collect();
            let mean = v.iter().sum::<f64>() / v.len() as f64;
            let var = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / v.len() as f64;
            (mean, var.sqrt())
        };
        let (fine_mean, fine_sd) = stats(0.005);
        let (coarse_mean, coarse_sd) = stats(1.0);
        assert!(fine_mean > 5.0, "premise: the sample glows ({fine_mean})");
        assert!((fine_mean - coarse_mean).abs() < 0.35 * fine_mean, "mean glow fine {fine_mean} vs coarse {coarse_mean}");
        assert!(fine_sd > 2.0 * coarse_sd, "structure at zoom: sd fine {fine_sd} coarse {coarse_sd}");
    }

    /// Protects: a cold crust barely glows, a hot one glows a lot (the `heat`
    /// option is live), and the halo falls to zero at its radius.
    #[test]
    fn heat_controls_the_glow_and_the_halo_ends_at_its_radius() {
        let glow = |h: f64| (0..900).map(|k| lava(h).sample([(k % 30) as f64 * 0.21, (k / 30) as f64 * 0.19], lava(h).base_heat(3.0, 0.0), 0.01).1[0]).sum::<f64>() / 900.0;
        let (cold, hot) = (glow(0.05), glow(0.95));
        assert!(hot > 4.0 * cold.max(1.0), "hot {hot} cold {cold}");
        let m = lava(0.8);
        assert!(m.halo(0.01, 0.0)[0] > 20.0, "{:?}", m.halo(0.01, 0.0));
        assert_eq!(m.halo(2.0, 0.0), [0.0; 3]);
        assert_eq!(m.halo(-0.1, 0.0), [0.0; 3]);
        assert!(m.halo(1.5, 0.0)[0] < m.halo(0.5, 0.0)[0]);
    }

    /// Protects: the edge distance is a distance -- zero on a bisector, and
    /// positive and bounded inside a cell -- and the cell id is stable.
    #[test]
    fn voronoi_edge_is_a_distance() {
        let mut max_e: f64 = 0.0;
        for k in 0..2000 {
            let q = [k as f64 * 0.0137, (k * 7 % 113) as f64 * 0.031];
            let (e, id) = voronoi_edge(q, 9);
            assert!(e >= 0.0);
            max_e = max_e.max(e);
            assert_eq!(voronoi_edge(q, 9).1, id);
        }
        assert!(max_e > 0.2 && max_e < 1.0, "{max_e}");
    }
}
