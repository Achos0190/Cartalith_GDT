//! Volcano colours for the Satellite relief look (VA-2, VA-3, VA-4 of
//! `VOLCANO_APPEARANCE_RESEARCH.md` §7; owner Ruling CW, 2026-10-24, all nine
//! of the research's §8 questions at their defaults).
//!
//! # What it does
//!
//! For a pixel on a volcanic edifice it decides, from the world model alone
//! (the edifice's kind, setting, age in years and activity, the pixel's place
//! on the edifice and the cell's rainfall):
//!
//! - **VA-2, which rock** shows: basalt for every shield and scoria cone
//!   (rift, hotspot and arc scoria alike), andesite on an andesitic
//!   stratocone's core, and the new **tephra** slot on its outer flank and
//!   apron, where ash and lapilli lie (research §2.2, §3.1; question 9's
//!   eighth palette). Before this every volcanic cell painted as andesite
//!   (`cartalith_civ::build_lithology`'s code 2, research §4.3).
//! - **VA-3, how weathered and how green**: fresh glassy rock (near-black
//!   basalt, black scoria) brightening and oxidising with age, an arid
//!   hematite tint, dark Andosol soil between plants in humid climates, red
//!   oxidised scoria inside young cinder craters, and the vegetation cover the
//!   flank has reached by its age in its rainfall (research §6.3's log-time
//!   logistic). Young bare cones therefore stay rock on any slope and old
//!   covered ones keep the climate's own green: **volcano age overrides the
//!   slope-rock warmth rule** (question 6).
//! - **VA-4, activity**: hydrothermal staining (kaolinite white, jarosite
//!   ochre, hematite red) on the summits of active and dormant edifices,
//!   native sulphur at active vents, and a **subtle, capped daytime glow** in
//!   active craters (question 5), drawn by the colour-paint layer's own lava
//!   material ([`cartalith_spatial::colour_paint::ProceduralLava`]) placed
//!   from the world model instead of by a brush.
//!
//! # Why this shape
//!
//! Colour on a real volcano is age × climate more than rock type (research
//! §1 item 3): fresh basalt is among the darkest natural surfaces, weathering
//! brightens and reddens it over decades to millennia, and vegetation then
//! covers it at a rate set by rainfall -- three to four orders of magnitude
//! in time-to-green between about 400 and 2 300 mm a year. Every RGB target
//! below is the research's §6.2 table, a labelled judgement unless it says
//! otherwise.
//!
//! # The zoom law
//!
//! Every pattern (stains, sulphur blotches, scoria oxidation, the vent's crust
//! and cracks) is value noise in **kilometres**, octaves halving down to a few
//! metres, each drawn only once it spans [`NYQ_PX`] output pixels and
//! contributing its mean until then, the canopy texture's and the lava
//! material's rule. So the screen, the deep-zoom tiles and every export band
//! evaluate the same pure function, agree at equal pixel size, and the view
//! keeps gaining structure as it zooms (the owner's hard requirement) without
//! the overall tone changing.
//!
//! # What it must never do
//!
//! - Never run outside the look that turns it on: `land_color` asks for a
//!   sample only when `TerrainAppearance::volcano_look > 0` (Satellite relief
//!   only), a volcano table is attached and the map width is known; every
//!   other look, `default()` and `js_reference()` never reach this module.
//! - Never change world data or generation: it reads the edifice record and
//!   the climate and returns colours.
//! - Never guess a kind: an edifice with no morphology (the reference model)
//!   yields no sample.

use super::{luma, mix, LithoSample, Rgb, TerrainAppearance};
use cartalith_jsmath::smoothstep;
use cartalith_noise::vnoise;
use cartalith_spatial::colour_paint::{EmissiveMaterial, GlowRamp, ProceduralLava};
use cartalith_terrain::volcano_attr::{self, Activity, VolcanoIndex};
use cartalith_terrain::EdificeKind;

/// The render-only eighth lithology index: tephra / tuff (research question
/// 9). Not a `cartalith_civ::LITH_KEYS` entry -- the civ layer never emits it
/// -- so it lives outside [`super::LITHO_PALETTE_ORDER`]'s checked seven and
/// is reachable only through this module's override.
pub const LITH_TEPHRA: u8 = 7;

/// Fresh glassy basalt, research §6.2 row 1: VNIR reflectance 0.02-0.04
/// (cited: Reeder) is sRGB 39-56 by §6.1's conversion; a slightly blue grey.
const FRESH_BASALT: Rgb = (40.0, 41.0, 45.0);
/// Fresh black scoria (research §6.2 row 2; cited: Ejima 2017, the black
/// outer wall of Kasayama).
const FRESH_SCORIA: Rgb = (46.0, 40.0, 40.0);
/// Fresh andesite lava on a young stratocone ("often dark, pristine lava
/// flows", Davidson & De Silva stage 1, cited); the value a little below the
/// research's (104, 98, 98) so a young cone reads darker than its weathered
/// palette. Judgement.
const FRESH_ANDESITE: Rgb = (88.0, 84.0, 85.0);
/// Hot-oxidised red-brown scoria inside a young cinder crater (research
/// §6.2; cited: Ejima 2017, Kīlauea fissure 8).
///
/// Toned from the research's (118, 52, 38) to a duller (108, 64, 50) on the
/// first frames, where the look's material chroma (`biome_sat`) and warm
/// light pushed the cited colour to a glowing orange that read as lava.
const OX_SCORIA: Rgb = (108.0, 64.0, 50.0);
/// Arid weathered basalt, 100 yr - 10 ka, with the hematite tint grown in
/// (research §6.2 row 3; cited: Rose et al. 2011, the Tenerife study).
const ARID_OX_YOUNG: Rgb = (84.0, 70.0, 62.0);
/// Arid weathered volcanic rock older than about 500 ka (research §6.2,
/// "weathered rock"; judgement).
const ARID_OX_OLD: Rgb = (132.0, 104.0, 86.0);
/// Dark Andosol between the plants on a humid flank (research §3.2; cited:
/// ISRIC -- *an*, "black"; the RGB a judgement).
const ANDOSOL: Rgb = (55.0, 48.0, 42.0);
/// Acid-sulfate alteration: kaolinite / alunite white, jarosite ochre,
/// hematite red (research §3.3; minerals cited, colours judgement).
const BLEACH: Rgb = (214.0, 208.0, 192.0);
const OCHRE: Rgb = (176.0, 138.0, 62.0);
const ALTER_RED: Rgb = (140.0, 62.0, 44.0);
/// Native sulphur at an active vent (research §6.2; popular sources).
const SULPHUR: Rgb = (222.0, 204.0, 70.0);
/// The chilled crust over an active vent: the shipped colour-paint Lava
/// preset's crust (research §6.2).
const CRUST: Rgb = (34.0, 33.0, 38.0);

/// The tephra / tuff palette slot's default ramp: a pale grey-buff, the
/// research's "tephra-grey fans (128, 124, 120)" as the middle stop
/// (question 9: "pale grey-buff ... so arc flanks read as ash, not desert").
/// Judgement.
pub const TEPHRA_DEFAULT: [Rgb; 3] = [(112.0, 108.0, 102.0), (128.0, 124.0, 118.0), (148.0, 144.0, 136.0)];

/// An octave is drawn once its wavelength spans this many output pixels,
/// fading in over one doubling: the canopy texture's and the lava material's
/// Nyquist rule (`cartalith_spatial::colour_paint::material::NYQ_PX`).
const NYQ_PX: f64 = cartalith_spatial::colour_paint::material::NYQ_PX;
/// The finest pattern wavelength, km (4 m): the lava material's floor
/// (`MIN_CELL_KM`). Below it nothing here should invent structure.
const MIN_WL_KM: f64 = 0.004;

/// The daytime glow's scale over the lava material's own emission (research
/// question 5: "a subtle daytime glow ... capped"). Judgement: at 0.55 the
/// cracks read orange in sunlight without the vent reading as paint.
pub const DAY_GLOW: f64 = 0.55;
/// The daytime glow's luminance cap, on the 0-255 scale the emission is
/// added on (VA-4's bar: "daytime glow luminance stays below a stated cap").
/// Judgement: a little under half of white, so even a lava lake's core stays
/// below the brightness of snow.
pub const GLOW_LUMA_CAP: f64 = 110.0;

/// One pixel's place on one edifice, as [`ground`] reads it.
#[derive(Clone, Copy, Debug)]
pub struct VolcanoSample {
    /// Index of the edifice in the world's table (seeds its patterns).
    pub index: usize,
    pub kind: EdificeKind,
    /// `volcano_attr::is_basaltic`.
    pub basaltic: bool,
    /// The edifice's own `0..=1` age draw.
    pub age01: f64,
    /// `log10` of [`volcano_attr::age_years`].
    pub log_years: f64,
    pub activity: Activity,
    /// Distance from the centre over the radius, `0..=1`.
    pub t: f64,
    /// The stamp's crater floor and rim, fractions of the radius
    /// (`volcano_attr::zone_fractions`).
    pub floor: f64,
    pub rim: f64,
    /// The edifice radius, km.
    pub r_km: f64,
    /// The pixel's distance from the centre, km.
    pub d_km: f64,
    /// The pixel's position, km from the map's origin.
    pub p_km: [f64; 2],
    /// The edifice's centre, km from the map's origin (the radial frame the
    /// lava-flow lobes are laid out in).
    pub c_km: [f64; 2],
    /// Km between neighbouring output pixels (the zoom law's footprint).
    pub px_km: f64,
}

/// The owner-value margin over which two overlapping edifices' colours are
/// blended at their shared boundary, in `(1 - t)(1 - age)` units: at the
/// boundary each counts half, and the runner-up fades out once the winner
/// leads by this much. Judgement: without it the first frames drew two
/// cones' colours meeting along a hard line (24601, x16).
pub const OWNER_BLEND: f64 = 0.05;

/// A pixel's winner and, near a shared boundary, the runner-up with its
/// blend weight (`0..=0.5`).
#[derive(Clone, Copy, Debug)]
pub struct VolcanoPair {
    pub a: VolcanoSample,
    pub b: Option<(VolcanoSample, f64)>,
}

/// [`sample`] for the winner and the runner-up together ([`VolcanoPair`]).
/// The runner-up is kept only within [`OWNER_BLEND`] of the winner.
pub fn sample_pair(index: &VolcanoIndex, x: f64, y: f64, km_per_cell: f64, px_km: f64) -> Option<VolcanoPair> {
    if !(km_per_cell.is_finite() && km_per_cell > 0.0 && px_km.is_finite() && px_km > 0.0) {
        return None;
    }
    let ((h1, v1), second) = index.top2_at(x, y)?;
    let a = from_hit(h1, x, y, km_per_cell, px_km)?;
    let b = second.and_then(|(h2, v2)| {
        let w = 0.5 * (1.0 - smoothstep(0.0, OWNER_BLEND, v1 - v2));
        if w <= 0.0 {
            return None;
        }
        from_hit(h2, x, y, km_per_cell, px_km).map(|s| (s, w))
    });
    Some(VolcanoPair { a, b })
}

/// The ground for a [`VolcanoPair`]: the winner's, mixed toward the
/// runner-up's by its weight (rock, bare share, footprint and glow). The
/// rock category stays the winner's -- a category is never averaged
/// (`LithoSample`'s rule) -- so only colours cross the boundary smoothly.
pub fn ground_pair(a: &TerrainAppearance, p: &VolcanoPair, rain01: f64, tt: f64) -> VolcanoGround {
    let ga = ground(a, &p.a, rain01, tt);
    let Some((sb, w)) = p.b else { return ga };
    let gb = ground(a, &sb, rain01, tt);
    let glow = match (ga.glow, gb.glow) {
        (None, None) => None,
        (x, y) => {
            let (x, y) = (x.unwrap_or((0.0, 0.0, 0.0)), y.unwrap_or((0.0, 0.0, 0.0)));
            Some(mix(x, y, w))
        }
    };
    VolcanoGround { lith: ga.lith, rock: mix(ga.rock, gb.rock, w), bare: ga.bare + (gb.bare - ga.bare) * w, vis: ga.vis + (gb.vis - ga.vis) * w, glow }
}

/// The sample at the fractional grid position `(x, y)`, or `None` off every
/// edifice, on an edifice with no morphology, or with an unusable scale.
pub fn sample(index: &VolcanoIndex, x: f64, y: f64, km_per_cell: f64, px_km: f64) -> Option<VolcanoSample> {
    if !(km_per_cell.is_finite() && km_per_cell > 0.0 && px_km.is_finite() && px_km > 0.0) {
        return None;
    }
    from_hit(index.winner_at(x, y)?, x, y, km_per_cell, px_km)
}

/// One owner hit as a [`VolcanoSample`]; `None` without a morphology.
fn from_hit(hit: cartalith_terrain::volcano_attr::VolcanoHit<'_>, x: f64, y: f64, km_per_cell: f64, px_km: f64) -> Option<VolcanoSample> {
    let e = hit.edifice;
    let kind = e.kind?;
    let (floor, rim) = volcano_attr::zones(e)?;
    Some(VolcanoSample {
        index: hit.index,
        kind,
        basaltic: volcano_attr::is_basaltic(e)?,
        age01: e.age,
        log_years: volcano_attr::age_years(e.age).log10(),
        activity: volcano_attr::activity_of(e.age),
        t: hit.t,
        floor,
        rim,
        r_km: e.r * km_per_cell,
        d_km: hit.t * e.r * km_per_cell,
        p_km: [x * km_per_cell, y * km_per_cell],
        c_km: [e.cx * km_per_cell, e.cy * km_per_cell],
        px_km,
    })
}

/// What [`ground`] decides for one pixel.
#[derive(Clone, Copy, Debug)]
pub struct VolcanoGround {
    /// The rock under the pixel (VA-2): replaces the lithology `land_color`
    /// was handed, so the material blend's rock, the slope tint, the bedrock
    /// exposure and the microtexture all read the volcano's own rock.
    pub lith: LithoSample,
    /// The bare ground's colour: weathered, oxidised, stained, crusted.
    pub rock: Rgb,
    /// How much of the pixel is bare (`0..=1`): the footprint weight times
    /// the cover the flank has not yet reached. `land_color` mixes `rock`
    /// over its non-snow ground by this much.
    pub bare: f64,
    /// The footprint weight (`1` on the edifice, fading to `0` at its toe), so
    /// the slope tint's rock colour follows the volcano only on it.
    pub vis: f64,
    /// The capped daytime glow to add after the light (0-255), active vents
    /// only.
    pub glow: Option<Rgb>,
}

/// Per-octave lattice rotations `(cos, sin)`, away from the axes and the
/// diagonals (the lava material's own set), so no octave's lattice lines up
/// with another's or with the raster.
const ROT: [(f64, f64); 4] = [(0.923_879_5, 0.382_683_4), (0.382_683_4, 0.923_879_5), (-0.382_683_4, 0.923_879_5), (-0.923_879_5, 0.382_683_4)];

/// Value-noise fbm in kilometres under the zoom law: octaves from `wl0` km
/// halving (amplitude 0.55 per octave) down to [`MIN_WL_KM`], each faded in
/// once it spans [`NYQ_PX`] pixels of `px_km` and contributing its mean
/// (`0.5`) until then. `0..1`, mean `0.5` at every zoom.
fn zoom_fbm(p: [f64; 2], wl0: f64, px_km: f64, seed: i32) -> f64 {
    let (mut wl, mut amp, mut sum, mut norm) = (wl0.max(MIN_WL_KM), 0.5, 0.0, 0.0);
    for o in 0..10 {
        if wl < MIN_WL_KM {
            break;
        }
        let vis = smoothstep(NYQ_PX * px_km, 2.0 * NYQ_PX * px_km, wl);
        let n = if vis > 0.0 {
            let (rx, ry) = ROT[o % ROT.len()];
            vnoise((p[0] * rx - p[1] * ry) / wl, (p[0] * ry + p[1] * rx) / wl, seed + o as i32 * 131)
        } else {
            0.5
        };
        sum += amp * (0.5 + (n - 0.5) * vis);
        norm += amp;
        wl *= 0.5;
        amp *= 0.55;
    }
    sum / norm
}

/// `log10` of the age in years at which a flank reaches half its cover, for
/// `rain_mm` of rain a year: research §6.3's four fitted points (≥ 2 000 mm:
/// 50 years; 1 500 mm: 150; 500 mm: 5 000; 400 mm: 50 000), linear in `log10`
/// years between them, flat above 2 000 mm and rising 0.5 decades per 100 mm
/// below 400 (so a hyper-arid flank never greens inside the 3 Myr span). The
/// points are the research's labelled judgement fitted by eye to cited data
/// (Wood 1980; Kinney et al. 2015).
pub fn half_cover_log_years(rain_mm: f64) -> f64 {
    const PTS: [(f64, f64); 4] = [(400.0, 4.699), (500.0, 3.699), (1500.0, 2.176), (2000.0, 1.699)];
    if rain_mm <= PTS[0].0 {
        return PTS[0].1 + (PTS[0].0 - rain_mm.max(0.0)) / 100.0 * 0.5;
    }
    for w in PTS.windows(2) {
        let ((m0, l0), (m1, l1)) = (w[0], w[1]);
        if rain_mm <= m1 {
            return l0 + (l1 - l0) * (rain_mm - m0) / (m1 - m0);
        }
    }
    PTS[3].1
}

/// The vegetation cover `0..=1` a flank of `log_years` has reached in
/// `rain_mm` of rain: research §6.3's log-time logistic,
/// `smoothstep(log τ50 - 1, log τ50 + 1, log10 years)`. Multiplies the
/// climate's own cover (the caller keeps the material blend's vegetation
/// where this is 1), so it can only take green away, never add it.
pub fn cover(log_years: f64, rain_mm: f64) -> f64 {
    let l50 = half_cover_log_years(rain_mm);
    smoothstep(l50 - 1.0, l50 + 1.0, log_years)
}

/// The rainfall field's `0..=1` mapped to mm a year: the Köppen
/// classifier's own `max_rain_mm` default (`cartalith_climate::koppen`,
/// 3 000 mm), so this stage and the climate classes read one scale.
pub const RAIN_MM_FULL: f64 = 3000.0;

/// The ground on a volcano at this pixel (VA-2 to VA-4). `rain01` is the
/// cell's own rainfall (`0..=1`), `tt` the material ramp position
/// `land_color` already chose, `a` the look.
pub fn ground(a: &TerrainAppearance, v: &VolcanoSample, rain01: f64, tt: f64) -> VolcanoGround {
    let lg = v.log_years;
    let rain01 = rain01.clamp(0.0, 1.0);
    let seed = (v.index as i32).wrapping_mul(977);

    // ---- VA-2: the rock --------------------------------------------------
    // Andesitic stratocones: andesite on the core, tephra on the outer flank
    // and apron, blended over t 0.38..0.62 (a judgement: the research's
    // "upper flank rim..0.5" / "lower flank 0.5..1" zones, softened so the
    // contact is not a ring).
    let core = if v.basaltic { 1.0 } else { smoothstep(0.62, 0.38, v.t) };
    let lith = if v.basaltic {
        LithoSample::One(1)
    } else if core >= 1.0 {
        LithoSample::One(2)
    } else if core <= 0.0 {
        LithoSample::One(LITH_TEPHRA)
    } else {
        LithoSample::Blend([(2, core), (LITH_TEPHRA, 1.0 - core), (0, 0.0), (0, 0.0)])
    };
    let weathered = lith.ramp(a, tt);

    // ---- VA-3: the surface's own age ----------------------------------------
    // The edifice's age is its LAST eruption, and a polygenetic edifice's
    // last eruption does not resurface all of it: the summit and the lava
    // flows it sent down are that young, while the rest of the flank is as
    // old as the edifice's building -- composite cones grow "in spurts" over
    // episodes of up to ~100 ka, shields over tens to hundreds of ka
    // (research §2.1, cited: Davidson & De Silva 2000), and one shield
    // carries flows of many ages (Kinney et al. 2015, Kitayama et al. 1995).
    // So the surface age is the edifice's on the summit and along the flow
    // tongues and at least `building_log_years` on the outer flank. A cinder
    // cone is monogenetic (research §2.2): its one eruption built and
    // surfaced all of it, so it keeps one age. The flows radiate from the
    // centre: bearing from a smooth angular noise (one per edifice,
    // resolution free), length and ragged edges from kilometre noise under
    // the zoom law.
    let flow = flow_tongues(v, seed);
    let lg = match building_log_years(v.kind) {
        Some(lb) if lb > lg => lg + (lb - lg) * smoothstep(0.2, 0.7, v.t) * (1.0 - flow),
        _ => lg,
    };

    // ---- VA-3: age and climate on the bare rock --------------------------
    // Fresh glass under 100 years, weathered (the palette) by 10 ka: the
    // brightening of bare lava over decades to millennia (research §3.2;
    // cited: the Tenerife study, Rose et al. 2011). Log-years 2..4.
    let fresh_c = if v.basaltic {
        if v.kind == EdificeKind::Cinder {
            FRESH_SCORIA
        } else {
            FRESH_BASALT
        }
    } else {
        mix(weathered, FRESH_ANDESITE, core)
    };
    let fresh = 1.0 - smoothstep(2.0, 4.0, lg);
    let mut rock = mix(weathered, fresh_c, fresh);
    // Arid: hematite grows on bare rock after a few decades (log 2..3.5),
    // toward the old weathered red-brown past 500 ka (log 4..5.7). `rain01`
    // 0.25..0.12 is 750..360 mm: the research's arid band (<500 mm).
    let arid = smoothstep(0.25, 0.12, rain01);
    if arid > 0.0 {
        let ox = mix(ARID_OX_YOUNG, ARID_OX_OLD, smoothstep(4.0, 5.7, lg));
        rock = mix(rock, ox, arid * smoothstep(2.0, 3.5, lg) * 0.85);
    }
    // Humid: once plants arrive, what shows between them is dark Andosol
    // soil, not rock (research §3.2). `rain01` 0.30..0.55 is 900..1 650 mm.
    let humid = smoothstep(0.30, 0.55, rain01) * smoothstep(2.0, 3.0, lg);
    if humid > 0.0 {
        rock = mix(rock, ANDOSOL, humid * 0.6);
    }
    // Young cinder craters: the interior oxidised red-brown, the outer wall
    // black (cited: Ejima 2017). Patchy, fading with age over log 3..4.5.
    if v.kind == EdificeKind::Cinder {
        let zone = smoothstep(v.rim * 1.1, v.rim * 0.75, v.t);
        let young = 1.0 - smoothstep(3.0, 4.5, lg);
        if zone * young > 0.0 {
            let n = zoom_fbm(v.p_km, (v.rim * v.r_km * 0.5).max(0.02), v.px_km, seed + 11);
            rock = mix(rock, OX_SCORIA, zone * young * (0.45 + 0.55 * smoothstep(0.35, 0.65, n)));
        }
    }

    // ---- VA-4: alteration and sulphur -------------------------------------
    // Acid-sulfate staining on the summit of an active or dormant edifice
    // (research §3.3: persists after activity ends). Patches in km, the
    // colour chosen by a second field.
    // Strengths are judgements set on the 2026-10-24 frames: at 0.85 / 0.6
    // an active cone's summit read as a ring of fire at x16.
    let stain_k = match v.activity {
        Activity::Active => 0.5,
        Activity::Dormant => 0.35,
        Activity::Extinct => 0.0,
    };
    if stain_k > 0.0 {
        let zone = smoothstep(v.rim * 1.35, v.rim * 0.85, v.t);
        if zone > 0.0 {
            let wl = (v.rim * v.r_km * 0.45).max(0.02);
            let patch = smoothstep(0.42, 0.62, zoom_fbm(v.p_km, wl, v.px_km, seed + 23));
            let pick = zoom_fbm(v.p_km, wl * 1.3, v.px_km, seed + 37);
            let col = if pick < 0.5 { mix(BLEACH, OCHRE, smoothstep(0.38, 0.5, pick)) } else { mix(OCHRE, ALTER_RED, smoothstep(0.55, 0.7, pick)) };
            rock = mix(rock, col, zone * patch * stain_k);
        }
    }
    if v.activity == Activity::Active {
        let ring = smoothstep(v.floor * 0.4, v.floor, v.t) * smoothstep(v.rim * 1.15, v.rim * 0.9, v.t);
        if ring > 0.0 {
            let blot = smoothstep(0.62, 0.76, zoom_fbm(v.p_km, (v.floor * v.r_km * 0.3).max(0.01), v.px_km, seed + 53));
            rock = mix(rock, SULPHUR, ring * blot * 0.55);
        }
    }

    // ---- VA-3: how much of it is bare --------------------------------------
    let mut c = cover(lg, rain01 * RAIN_MM_FULL);
    match v.activity {
        // An active crater and its inner wall carry no plants.
        Activity::Active => c *= smoothstep(v.rim * 0.9, v.rim * 1.4, v.t),
        // A dormant crater floor stays mostly bare (alteration, gas).
        Activity::Dormant => c *= 1.0 - 0.7 * smoothstep(v.rim * 1.2, v.floor * 0.8, v.t),
        Activity::Extinct => {}
    }
    // The footprint: the whole edifice, fading out over its outer third, the
    // fade's radius wandering by kilometre noise so the toe meets the
    // surrounding ground along a ragged line, never a circle (judgement: the
    // first frames' 0.8..1 fade drew a disc with a visible edge).
    let tw = v.t + 0.22 * (zoom_fbm(v.p_km, (v.r_km * 0.3).max(0.05), v.px_km, seed + 83) - 0.5);
    let vis = smoothstep(1.0, 0.66, tw);
    let mut bare = vis * (1.0 - c);

    // ---- VA-4: the active vent ---------------------------------------------
    let mut glow = None;
    if v.activity == Activity::Active {
        // A lava lake on a shield, an open vent on a cone (research §6.2's
        // heat by sub-type, 0.85 / 0.6, raised to 0.9 / 0.72 on the first
        // frames, where 0.6 under the daylight scale showed one dull seam),
        // a little hotter the younger.
        let youth = (1.0 - v.age01 / volcano_attr::ACTIVE_BELOW).clamp(0.0, 1.0);
        let heat = (if v.kind == EdificeKind::Shield { 0.9 } else { 0.72 }) * (0.8 + 0.2 * youth);
        // The vent's radius: inside the crater floor, 120 m to 900 m
        // (judgement: Halema'uma'u's lake is about 1 km across; a cone's open
        // vent a few hundred metres).
        let rv = (v.floor * v.r_km * 0.6).clamp(0.12, 0.9);
        let mat = EmissiveMaterial { ramp: GlowRamp::Fire, heat, crack_density: 0.6, glow_radius_km: rv * 0.8, heat_along: 0.0 };
        if let Some(lava) = ProceduralLava::new(&mat, rv) {
            let s = rv + lava.edge_offset(v.p_km, v.px_km) - v.d_km;
            let half = 0.5 * v.px_km;
            let cov = smoothstep(-half, half, s);
            let mut e = [0.0; 3];
            if cov > 0.0 {
                let (ck, em) = lava.sample(v.p_km, lava.base_heat(s.max(0.0)), v.px_km);
                rock = mix(rock, (CRUST.0 * ck, CRUST.1 * ck, CRUST.2 * ck), cov);
                bare = bare + (1.0 - bare) * cov;
                e = [em[0] * cov, em[1] * cov, em[2] * cov];
            }
            if s < half {
                let h = lava.halo((-s).max(0.0));
                let k = 1.0 - cov;
                e = [e[0] + h[0] * k, e[1] + h[1] * k, e[2] + h[2] * k];
            }
            glow = capped_glow((e[0], e[1], e[2]));
        }
    }

    VolcanoGround { lith, rock, bare, vis, glow }
}

/// `log10` of the years a polygenetic edifice's outer flank has stood since
/// it was last built over (see `ground`'s surface-age note), or `None` for a
/// monogenetic cinder cone. Judgements inside the cited spans (research
/// §2.1: composite "cone-building episodes generally <100 ka"; shields
/// "many eruptions over tens to hundreds of ka"): a stratocone's flank about
/// 30 ka (`10^4.5`), a shield's about 100 ka (`10^5`).
pub fn building_log_years(kind: EdificeKind) -> Option<f64> {
    match kind {
        EdificeKind::Cinder => None,
        EdificeKind::Strato => Some(4.5),
        EdificeKind::Shield => Some(5.0),
    }
}

/// How many lava-flow ribbons the last eruption of a polygenetic edifice
/// sent down its flanks. Judgement: enough to read as several flows at x4,
/// few enough that the older flank stays the larger share.
const FLOWS: usize = 7;

/// `0..=1`: whether the pixel lies on one of the last eruption's lava flows.
/// Each flow is a ribbon from near the summit down a bearing drawn from a
/// hash of the edifice (no RNG stream touched): a width in kilometres (0.3
/// to 2.5 km, 3-7 % of the radius; a flow keeps its width downhill
/// rather than widening like a wedge), a reach of 45-100 % of the radius,
/// and a meander and ragged margin in kilometre noise under the zoom law, so
/// the margin keeps gaining detail as the view zooms. Cinder cones erupt
/// tephra rather than flows and return `0`.
fn flow_tongues(v: &VolcanoSample, seed: i32) -> f64 {
    if v.kind == EdificeKind::Cinder || v.d_km <= 0.0 {
        return 0.0;
    }
    let (rx, ry) = (v.p_km[0] - v.c_km[0], v.p_km[1] - v.c_km[1]);
    let h = |k: usize, salt: i32| cartalith_noise::hash(seed, k as i32, salt);
    let mut m: f64 = 0.0;
    for k in 0..FLOWS {
        let phi = std::f64::consts::TAU * h(k, 911);
        let (ux, uy) = (phi.cos(), phi.sin());
        let along = rx * ux + ry * uy;
        if along <= 0.0 {
            continue;
        }
        let reach = v.r_km * (0.45 + 0.55 * h(k, 913));
        let width = (v.r_km * (0.03 + 0.04 * h(k, 917))).clamp(0.3, 2.5);
        // The meander: the ribbon's centre line wanders sideways by up to
        // about three widths over a few kilometres.
        let wander = 3.0 * width * (zoom_fbm([along, k as f64 * 37.0], (reach * 0.35).max(0.5), v.px_km, seed + 131 + k as i32) - 0.5) * 2.0;
        let perp = (rx * -uy + ry * ux - wander).abs();
        let rag = width * 0.5 * (zoom_fbm(v.p_km, (width * 0.8).max(0.02), v.px_km, seed + 151 + k as i32) - 0.5) * 2.0;
        let w = width + rag;
        let side = smoothstep(w + 0.5 * v.px_km.max(width * 0.15), w - 0.5 * v.px_km.max(width * 0.15), perp);
        let end = smoothstep(reach, reach - width * 2.0, along) * smoothstep(v.r_km * v.floor, v.r_km * v.rim, along);
        m = m.max(side * end);
    }
    m
}

/// The emission scaled to daylight ([`DAY_GLOW`]) and capped at
/// [`GLOW_LUMA_CAP`] luminance (the hue kept: the whole colour is scaled).
/// `None` for no emission.
pub fn capped_glow(e: Rgb) -> Option<Rgb> {
    let e = (e.0 * DAY_GLOW, e.1 * DAY_GLOW, e.2 * DAY_GLOW);
    let y = luma(e);
    if y <= 0.0 {
        return None;
    }
    let k = if y > GLOW_LUMA_CAP { GLOW_LUMA_CAP / y } else { 1.0 };
    Some((e.0 * k, e.1 * k, e.2 * k))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects §6.3's four fitted points, read back from the function
    /// against the research's literals (50 yr, 150 yr, 5 000 yr, 50 000 yr).
    #[test]
    fn half_cover_passes_through_the_four_points() {
        assert!((half_cover_log_years(2000.0) - 50f64.log10()).abs() < 1e-3);
        assert!((half_cover_log_years(1500.0) - 150f64.log10()).abs() < 1e-3);
        assert!((half_cover_log_years(500.0) - 5000f64.log10()).abs() < 1e-3);
        assert!((half_cover_log_years(400.0) - 50_000f64.log10()).abs() < 1e-3);
        assert_eq!(half_cover_log_years(3000.0), half_cover_log_years(2000.0));
        // 200 mm: one more decade.
        assert!((half_cover_log_years(200.0) - 5.699).abs() < 1e-9);
    }

    /// Protects the logistic's shape: half cover at τ50, none a decade
    /// before, full a decade after; a 1 000-year cone covered in the wet and
    /// bare in the dry (the research's "three to four orders of magnitude").
    #[test]
    fn cover_is_the_log_time_logistic() {
        let l50 = half_cover_log_years(1500.0);
        assert!((cover(l50, 1500.0) - 0.5).abs() < 1e-12);
        assert_eq!(cover(l50 - 1.0, 1500.0), 0.0);
        assert_eq!(cover(l50 + 1.0, 1500.0), 1.0);
        assert!(cover(3.0, 2200.0) > 0.99);
        assert!(cover(3.0, 450.0) < 0.01);
    }

    /// Protects the daylight cap: a white-hot emission is scaled to the cap's
    /// luminance with its hue kept; a dim one is only scaled by `DAY_GLOW`.
    #[test]
    fn the_glow_is_capped_in_daylight() {
        // Literals, not the constants (a constant asserted against itself
        // pins nothing): the cap is 110 luma, the daylight scale 0.55.
        let g = capped_glow((255.0, 248.0, 200.0)).unwrap();
        assert!((luma(g) - 110.0).abs() < 1e-9, "capped at {}", luma(g));
        assert!(g.0 > g.2, "the hue survives the cap");
        let d = capped_glow((40.0, 4.0, 0.0)).unwrap();
        assert!((d.0 - 22.0).abs() < 1e-12, "scaled to {}", d.0);
        assert!(capped_glow((0.0, 0.0, 0.0)).is_none());
    }

    /// Protects the zoom law: the fbm's mean stays 0.5 whatever the
    /// footprint, and at a footprint wider than every octave it is exactly the
    /// mean (no structure invented below the pixel).
    #[test]
    fn zoom_fbm_keeps_its_mean_and_draws_nothing_below_the_pixel() {
        assert_eq!(zoom_fbm([3.1, 7.7], 0.5, 10.0, 5), 0.5);
        // A 2-D sample over 80 x 80 wavelengths of the coarsest octave, so
        // value noise's own sampling spread stays inside the tolerance.
        let mean = |px: f64| (0..160 * 160).map(|k| zoom_fbm([(k % 160) as f64 * 0.25 + 0.013, (k / 160) as f64 * 0.25 + 0.007], 0.5, px, 5)).sum::<f64>() / (160.0 * 160.0);
        for px in [0.001, 0.01, 0.1] {
            assert!((mean(px) - 0.5).abs() < 0.03, "mean {} at px {px}", mean(px));
        }
    }
}
