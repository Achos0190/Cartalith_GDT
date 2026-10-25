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
//! - **Ruling DA, lava flows**: the flows each active or dormant vent sent
//!   down the real height field (`cartalith_terrain::lava_flows`), laid over
//!   the edifice and the ground beyond it ([`ground_px`], [`flow_ground`]):
//!   pahoehoe turning to 'a'a downstream, hot and glowing while active,
//!   cooling to dark basalt, then weathering and greening by each flow's own
//!   eruption age under the same rules as the flank. They replaced the seven
//!   radial ribbons this module once drew inside each cone.
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
//!   The lava flows need `TerrainAppearance::lava_flows > 0` as well, and an
//!   index built with them (`render::volcano_index_for`).
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
    /// `log10` of [`volcano_attr::years`] (the age map the row was recorded
    /// with, Ruling CX item 2).
    pub log_years: f64,
    pub activity: Activity,
    /// [`volcano_attr::active_youth`]: how young an active edifice is inside
    /// the active band (the vent's heat).
    pub youth: f64,
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
        log_years: volcano_attr::years(e).log10(),
        activity: volcano_attr::activity(e),
        youth: volcano_attr::active_youth(e),
        t: hit.t,
        floor,
        rim,
        r_km: e.r * km_per_cell,
        d_km: hit.t * e.r * km_per_cell,
        p_km: [x * km_per_cell, y * km_per_cell],
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
    // So the surface age is the edifice's on the summit and at least
    // `building_log_years` on the outer flank. A cinder cone is monogenetic
    // (research §2.2): its one eruption built and surfaced all of it, so it
    // keeps one age. The flows of the last eruptions are not drawn here:
    // since Ruling DA they are traced down the real height field
    // (`cartalith_terrain::lava_flows`) and laid over this ground by
    // [`ground_px`], each at its own age -- which retired the seven radial
    // ribbons (`flow_tongues`) this stage drew inside the disc before.
    let lg = match building_log_years(v.kind) {
        Some(lb) if lb > lg => lg + (lb - lg) * smoothstep(0.2, 0.7, v.t),
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
    let mut rock = aged_rock(weathered, fresh_c, lg, rain01);
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
        let heat = (if v.kind == EdificeKind::Shield { 0.9 } else { 0.72 }) * (0.8 + 0.2 * v.youth);
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

/// Bare rock of age `10^lg` years in `rain01` of rain: `fresh_c` under 100
/// years brightening to the weathered palette `weathered` by 10 ka (the
/// brightening of bare lava over decades to millennia, research §3.2; cited:
/// the Tenerife study, Rose et al. 2011; log-years 2..4); in arid climates a
/// hematite tint after a few decades (log 2..3.5) toward the old weathered
/// red-brown past 500 ka (log 4..5.7), `rain01` 0.25..0.12 being 750..360
/// mm, the research's arid band (<500 mm); in humid ones, once plants
/// arrive, the dark Andosol soil that shows between them (research §3.2;
/// `rain01` 0.30..0.55 is 900..1 650 mm). One rule for an edifice's flank
/// ([`ground`]) and a lava flow ([`flow_ground`]), so a flow and the flank
/// it lies on age alike.
fn aged_rock(weathered: Rgb, fresh_c: Rgb, lg: f64, rain01: f64) -> Rgb {
    let fresh = 1.0 - smoothstep(2.0, 4.0, lg);
    let mut rock = mix(weathered, fresh_c, fresh);
    let arid = smoothstep(0.25, 0.12, rain01);
    if arid > 0.0 {
        let ox = mix(ARID_OX_YOUNG, ARID_OX_OLD, smoothstep(4.0, 5.7, lg));
        rock = mix(rock, ox, arid * smoothstep(2.0, 3.5, lg) * 0.85);
    }
    let humid = smoothstep(0.30, 0.55, rain01) * smoothstep(2.0, 3.0, lg);
    if humid > 0.0 {
        rock = mix(rock, ANDOSOL, humid * 0.6);
    }
    rock
}

/// `log10` of the years a polygenetic edifice's outer flank has stood since
/// it was last built over (see `ground`'s surface-age note), or `None` for a
/// monogenetic cinder cone: the terrain crate's
/// [`cartalith_terrain::lava_flows::flank_log_years`], so the flank and the
/// older flows traced on it read one figure (a stratocone about 30 ka, a
/// shield about 100 ka; judgements inside research §2.1's cited spans).
pub fn building_log_years(kind: EdificeKind) -> Option<f64> {
    cartalith_terrain::lava_flows::flank_log_years(kind)
}

// ===========================================================================
// Ruling DA step 1: terrain-following lava flows
// ===========================================================================

/// Fresh pahoehoe: the smooth glassy rind, a little lighter than the
/// research's fresh basalt (VNIR 0.02-0.04, research §3.1, cited: Reeder) for
/// its sheen. Neutral, not blue, so a fresh flow never reads as water (the
/// owner's "distinct from rivers"). Judgement.
const FRESH_PAHOEHOE: Rgb = (52.0, 51.0, 52.0);
/// Fresh 'a'a: rubbly clinker, darker and a little brown (research §3.1: "a
/// colour difference: no source found"; the warmth is a judgement that keeps
/// it apart from water).
const FRESH_AA: Rgb = (40.0, 36.0, 34.0);
/// The clinker of an 'a'a flow's levees, lighter than its channel where both
/// resolve (the levee-and-channel form of channelised 'a'a, USGS/HVO).
/// Judgement.
const AA_CLINKER: Rgb = (64.0, 57.0, 52.0);
/// The most the ragged margin moves the edge, in half widths: the lava
/// material's edge reach (`ProceduralLava::edge_offset`, bounded by
/// `2 x 0.16 x 1.6` interior depths) with the half width as the depth --
/// lobes and embayments a little longer than the flow is wide, octaves down
/// to 4 m under the zoom law. Kept inside
/// `cartalith_terrain::lava_flows::REACH_HW` by a test.
const MARGIN_AMP: f64 = 0.52;
/// The material the margin is drawn from (only its edge is read; the heat
/// and glow fields are unused).
const MARGIN_MAT: EmissiveMaterial = EmissiveMaterial { ramp: GlowRamp::Fire, heat: 0.0, crack_density: 0.5, glow_radius_km: 0.0, heat_along: 0.0 };
/// How far a flow's width swells and pinches along its length, as a share of
/// its half width (lobes and breakouts; judgement).
const LOBE_AMP: f64 = 0.33;

/// One traced flow at one pixel.
#[derive(Clone, Copy, Debug)]
pub struct FlowSample {
    /// Index into the world's `LavaFlows::flows` (seeds its patterns).
    pub flow: usize,
    /// `log10` of the flow's age in years.
    pub log_years: f64,
    /// Still moving (drawn hot).
    pub active: bool,
    pub basaltic: bool,
    /// How much of the pixel the flow covers, `0..=1` (antialiased over one
    /// pixel at its ragged margin).
    pub cov: f64,
    /// Km inside the ragged margin (negative outside).
    pub inside_km: f64,
    /// Km from the centre line, and the half width there.
    pub d_km: f64,
    pub hw_km: f64,
    /// How far along the flow, `0` at the vent, `1` at its end.
    pub u: f64,
    pub p_km: [f64; 2],
    pub px_km: f64,
}

/// The flows at one pixel, youngest (on top) first; at most two are drawn
/// (a third flow under two others is hidden). `Default` is none.
#[derive(Clone, Copy, Debug, Default)]
pub struct FlowPx {
    pub layers: [Option<FlowSample>; 2],
}

/// Everything volcanic at one pixel: the edifice under it (if any) and the
/// lava flows over it (if any).
#[derive(Clone, Copy, Debug)]
pub struct VolcanoPx {
    pub edifice: Option<VolcanoPair>,
    pub flows: FlowPx,
}

/// The pixel at fractional grid position `(x, y)`: [`sample_pair`] for the
/// edifice and, when `flows_on`, the attached flows ([`flow_px`]). `None`
/// when neither reaches it.
pub fn sample_px(index: &VolcanoIndex, x: f64, y: f64, km_per_cell: f64, px_km: f64, flows_on: bool) -> Option<VolcanoPx> {
    let edifice = sample_pair(index, x, y, km_per_cell, px_km);
    let flows = match index.lava_flows() {
        Some(f) if flows_on && km_per_cell.is_finite() && km_per_cell > 0.0 && px_km.is_finite() && px_km > 0.0 => flow_px(f, x, y, km_per_cell, px_km),
        _ => FlowPx::default(),
    };
    if edifice.is_none() && flows.layers[0].is_none() {
        return None;
    }
    Some(VolcanoPx { edifice, flows })
}

/// The two youngest flows reaching `(x, y)`, with their coverage under the
/// ragged margin: the centre line's distance against the half width, the
/// half width narrowed at the vent (a flow leaves its vent narrow and
/// spreads), the margin moved by kilometre noise whose octaves arrive as the
/// view zooms in (the zoom law), so the margin keeps gaining lobes at x4,
/// x16 and x64 without the flow's area moving. An active flow is kept just
/// outside its margin too, for its halo.
pub fn flow_px(fl: &cartalith_terrain::lava_flows::LavaFlows, x: f64, y: f64, km_per_cell: f64, px_km: f64) -> FlowPx {
    let mut out = FlowPx::default();
    let mut n = 0;
    let p_km = [x * km_per_cell, y * km_per_cell];
    for h in fl.hits_at(x, y) {
        if n == out.layers.len() {
            break;
        }
        let f = &fl.flows()[h.flow];
        let seed = (h.flow as i32).wrapping_mul(613).wrapping_add(17);
        // Lobes along the flow: the width swells and pinches by up to a
        // third over a few widths of its length (a flow advances by lobes
        // and breakouts; judgement), read from the distance along the flow,
        // so it is the same at every zoom.
        // Every noise wavelength here is the flow's own base half width (its
        // first point's), never the pixel's interpolated one: a wavelength
        // that varies from pixel to pixel scrambles the noise's phase and
        // drew hair-like spikes across the margin (2026-10-25).
        let hw0 = f.pts[0].hw_km.max(0.05) as f64;
        let lobe = 1.0 + LOBE_AMP * (vnoise(h.s_km / (3.0 * hw0), 0.61, seed + 3) - 0.5) * 2.0;
        let hw = h.hw_km * lobe * (0.55 + 0.45 * smoothstep(0.0, 3.0 * h.hw_km, h.s_km));
        // The colour-paint lava material's own fractal margin (octave
        // amplitude proportional to wavelength, so lobes at every scale and
        // each octave arriving under the zoom law), scaled to the flow's
        // half width. Two earlier margins failed on the 2026-10-25 x64
        // frames of 314159: a normalised fbm drew a ruler-straight edge, and
        // the same stretched eightfold drew hair-like streaks.
        let rag = ProceduralLava::new(&MARGIN_MAT, hw0).map_or(0.0, |m| m.edge_offset([p_km[0] + seed as f64 * 0.37, p_km[1]], px_km)) * (hw / hw0);
        let inside = hw + rag - h.d_km;
        let soft = 0.5 * px_km;
        let cov = smoothstep(-soft, soft, inside);
        if cov <= 0.0 && !(f.active && inside > -hw) {
            continue;
        }
        let u = if f.len_km > 0.0 { (h.s_km / f.len_km).clamp(0.0, 1.0) } else { 0.0 };
        out.layers[n] = Some(FlowSample { flow: h.flow, log_years: f.years.max(1.0).log10(), active: f.active, basaltic: f.basaltic, cov, inside_km: inside, d_km: h.d_km, hw_km: hw, u, p_km, px_km });
        n += 1;
    }
    out
}

/// One flow's colour at its pixel: the bare-rock colour, how much of the
/// pixel shows it (coverage times the share vegetation has not yet reached),
/// and its emission (0-255, before the daylight scale) while active.
///
/// - **Lava type.** Basaltic flows are pahoehoe near the vent turning to
///   'a'a downstream (the usual change with distance and cooling on
///   Hawaiian flows; the 0.2-0.55 split of the length is a judgement);
///   andesitic flows are blocky throughout. 'A'a is rubble speckle in
///   kilometres from 80 m down to 4 m, pahoehoe smooth lobes; where the
///   flow is wide enough to resolve, a darker channel between lighter
///   clinker levees.
/// - **Age.** The same ageing as the flank it lies on ([`aged_rock`]), and
///   the same log-time cover ([`cover`]) -- pahoehoe a third of a decade
///   ahead (Kinney et al. 2015: pahoehoe vegetates faster than 'a'a), so
///   older flows fade into the surrounding ground at the climate's rate.
/// - **Activity.** An active flow is the colour-paint lava material
///   ([`ProceduralLava`]) -- chilled crust with glowing cracks, hottest at
///   the vent and cooling along its length, a hot front, a little halo.
pub fn flow_ground(a: &TerrainAppearance, f: &FlowSample, rain01: f64, tt: f64) -> (Rgb, f64, [f64; 3]) {
    let rain01 = rain01.clamp(0.0, 1.0);
    let seed = (f.flow as i32).wrapping_mul(613).wrapping_add(29);
    let weathered = LithoSample::One(if f.basaltic { 1 } else { 2 }).ramp(a, tt);
    let aa = if f.basaltic { smoothstep(0.2, 0.55, f.u) } else { 1.0 };
    let fresh_c = if f.basaltic { mix(FRESH_PAHOEHOE, FRESH_AA, aa) } else { FRESH_ANDESITE };
    let r = f.d_km / f.hw_km.max(1e-6);
    let lev_vis = smoothstep(NYQ_PX * f.px_km, 2.0 * NYQ_PX * f.px_km, 0.3 * f.hw_km);
    let levee = smoothstep(0.25, 0.38, r) * smoothstep(0.7, 0.52, r) * aa * lev_vis;
    let fresh_c = mix(fresh_c, AA_CLINKER, levee * 0.7);
    // Texture: +-17 % rubble on 'a'a, +-8 % lobes on pahoehoe (judgement).
    let rub = zoom_fbm(f.p_km, 0.08, f.px_km, seed + 5);
    let lobes = zoom_fbm(f.p_km, 0.4, f.px_km, seed + 9);
    let tex = 1.0 + aa * 0.34 * (rub - 0.5) + (1.0 - aa) * 0.16 * (lobes - 0.5);
    let rock = aged_rock(weathered, fresh_c, f.log_years, rain01);
    let mut rock = (rock.0 * tex, rock.1 * tex, rock.2 * tex);
    let c = cover(f.log_years + 0.3 * (1.0 - aa), rain01 * RAIN_MM_FULL);
    let mut alpha = f.cov * (1.0 - c);
    let mut e = [0.0; 3];
    if f.active {
        // Hottest at the vent, cooling to a third along the flow; the front
        // a little hotter again (a moving front shows incandescence as its
        // crust breaks). Judgements on the first frames' scale.
        let heat = (0.9 * (1.0 - 0.7 * f.u)).max(0.2);
        let mat = EmissiveMaterial { ramp: GlowRamp::Fire, heat, crack_density: 0.75, glow_radius_km: (f.hw_km * 0.8).max(0.08), heat_along: 0.0 };
        if let Some(lava) = ProceduralLava::new(&mat, f.hw_km.max(0.05)) {
            if f.cov > 0.0 {
                let front = 0.35 * smoothstep(0.85, 1.0, f.u);
                let base = (lava.base_heat(f.inside_km.max(0.0)) + front).min(1.0);
                let (ck, em) = lava.sample(f.p_km, base, f.px_km);
                // A moving flow is all crust and cracks: nothing has grown on
                // it, whatever the rain.
                rock = (CRUST.0 * ck, CRUST.1 * ck, CRUST.2 * ck);
                alpha = f.cov;
                e = [em[0] * f.cov, em[1] * f.cov, em[2] * f.cov];
            }
            if f.inside_km < 0.5 * f.px_km {
                let h = lava.halo((-f.inside_km).max(0.0));
                let k = 1.0 - f.cov;
                e = [e[0] + h[0] * k, e[1] + h[1] * k, e[2] + h[2] * k];
            }
        }
    }
    (rock, alpha, e)
}

/// What [`ground_px`] decides for one pixel: [`VolcanoGround`] with the rock
/// category optional (`None` -- a flow off every edifice that does not
/// cover the pixel's middle -- keeps the lithology `land_color` was handed).
#[derive(Clone, Copy, Debug)]
pub struct PxGround {
    pub lith: Option<LithoSample>,
    pub rock: Rgb,
    pub bare: f64,
    pub vis: f64,
    pub glow: Option<Rgb>,
    /// The share of `bare` that shows through snow: a flow that is moving
    /// or a few decades old has melted or shed the snow on it (the dark
    /// fresh flows across Etna's and Mauna Loa's snowfields), fading to none
    /// by about 300 years (log-years 1.7..2.5, a judgement). `0` for an
    /// edifice's own ground, which keeps `land_color`'s snow rule.
    pub over_snow: f64,
}

/// The ground at a [`VolcanoPx`]: the edifice's ([`ground_pair`]) with the
/// flows laid over it, oldest first, each by its bare share times
/// `flow_k` (the look's `lava_flows`). Without flows it is exactly
/// [`ground_pair`]'s (identity by control flow).
pub fn ground_px(a: &TerrainAppearance, v: &VolcanoPx, rain01: f64, tt: f64, flow_k: f64) -> PxGround {
    let g = v.edifice.map(|p| ground_pair(a, &p, rain01, tt));
    let mut out = match g {
        Some(g) => PxGround { lith: Some(g.lith), rock: g.rock, bare: g.bare, vis: g.vis, glow: g.glow, over_snow: 0.0 },
        None => PxGround { lith: None, rock: (0.0, 0.0, 0.0), bare: 0.0, vis: 0.0, glow: None, over_snow: 0.0 },
    };
    if flow_k <= 0.0 || v.flows.layers[0].is_none() {
        return out;
    }
    let mut em = [0.0; 3];
    for f in v.flows.layers.iter().rev().flatten() {
        let (fr, fa, fe) = flow_ground(a, f, rain01, tt);
        let fa = (fa * flow_k).clamp(0.0, 1.0);
        let na = fa + out.bare * (1.0 - fa);
        if na > 0.0 {
            let wb = out.bare * (1.0 - fa);
            out.rock = ((fr.0 * fa + out.rock.0 * wb) / na, (fr.1 * fa + out.rock.1 * wb) / na, (fr.2 * fa + out.rock.2 * wb) / na);
        }
        let fresh = if f.active { 1.0 } else { 1.0 - smoothstep(1.7, 2.5, f.log_years) };
        out.over_snow = if na > 0.0 { (fa * fresh + out.over_snow * out.bare * (1.0 - fa)) / na } else { 0.0 };
        out.bare = na;
        out.vis = out.vis.max(fa);
        // A flow is one rock: off the edifices it sets the category where it
        // covers the pixel's middle (a category is never averaged).
        if out.lith.is_none() && f.cov >= 0.5 && fa > 0.0 {
            out.lith = Some(LithoSample::One(if f.basaltic { 1 } else { 2 }));
        }
        // The younger flow hides the older one's glow where it covers it.
        em = [em[0] * (1.0 - f.cov) + fe[0] * flow_k, em[1] * (1.0 - f.cov) + fe[1] * flow_k, em[2] * (1.0 - f.cov) + fe[2] * flow_k];
    }
    if let Some(fg) = capped_glow((em[0], em[1], em[2])) {
        let sum = match out.glow {
            Some(g0) => (g0.0 + fg.0, g0.1 + fg.1, g0.2 + fg.2),
            None => fg,
        };
        out.glow = Some(cap_luma(sum));
    }
    out
}

/// `e` scaled down to [`GLOW_LUMA_CAP`] luminance if brighter (hue kept):
/// for summing two already daylight-scaled glows without passing the cap.
fn cap_luma(e: Rgb) -> Rgb {
    let y = luma(e);
    if y > GLOW_LUMA_CAP {
        let k = GLOW_LUMA_CAP / y;
        (e.0 * k, e.1 * k, e.2 * k)
    } else {
        e
    }
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

/// Acid crater-lake water at its turquoise end: colloidal sulphur scatters
/// blue (research §3.5 and §6.2; cited: Ohsawa et al. 2010, Aso's Yudamari
/// "often turquoise or emerald green"). The RGB is the research's §6.2
/// target, a labelled judgement.
pub const ACID_TURQUOISE: Rgb = (64.0, 168.0, 160.0);
/// Acid crater-lake water at its emerald end: dissolved ferrous iron
/// (same citation; RGB the research's §6.2 target, a judgement).
pub const ACID_EMERALD: Rgb = (70.0, 140.0, 96.0);
/// A pale sulphur slick on the water (research §6.2: "flat; a pale sulphur
/// slick"). Judgement.
const SULPHUR_SLICK: Rgb = (196.0, 206.0, 168.0);
/// How far the acid colour replaces the lake palette at the middle of a
/// crater lake (owner Ruling CX item 5). Judgement: most of the way, so the
/// lake reads turquoise or emerald, keeping a little of the look's own lake
/// tone and its grain.
pub const ACID_STRENGTH: f64 = 0.85;

/// Owner Ruling CX item 5 (2026-10-24, research G10): the lake surface at
/// grid position `(x, y)` when it lies inside the summit zone
/// (`volcano_attr::summit_zone_t`) of an **active or dormant** edifice --
/// an acid crater lake, turquoise to emerald -- else `base` unchanged. An
/// extinct crater's lake is neutral and keeps the generic lake palette
/// (research §3.5). Each edifice draws its own place on the turquoise-emerald
/// line from a hash of its index (Yudamari moved between the two over
/// years), mottled in kilometre noise and streaked with a pale sulphur slick
/// under the zoom law, so the lake keeps gaining structure as the view zooms
/// in without its overall tone moving. The colour fades in over the outer
/// fifth of the zone so a lake straddling the zone's edge has no seam.
/// `strength` is the look's `volcano_look`. What it must never do: tint a
/// lake off every active or dormant summit zone, or change water/land: it
/// only colours a pixel the caller already drew as lake.
pub fn crater_lake(index: &VolcanoIndex, x: f64, y: f64, km_per_cell: f64, px_km: f64, base: Rgb, strength: f64) -> Rgb {
    if !(strength > 0.0 && km_per_cell.is_finite() && km_per_cell > 0.0 && px_km.is_finite() && px_km > 0.0) {
        return base;
    }
    let Some(h) = index.winner_at(x, y) else { return base };
    let e = h.edifice;
    if volcano_attr::activity(e) == Activity::Extinct {
        return base;
    }
    let Some(zt) = volcano_attr::summit_zone_t(e) else { return base };
    let w = smoothstep(zt, zt * 0.8, h.t);
    if w <= 0.0 {
        return base;
    }
    let seed = (h.index as i32).wrapping_mul(977) + 401;
    let p_km = [x * km_per_cell, y * km_per_cell];
    let r_km = (e.r * zt * km_per_cell).max(0.05);
    // Where this lake sits on the turquoise-emerald line, then a mottle of
    // +-0.2 around it in kilometre noise.
    let pick = cartalith_noise::hash(seed, 7, 11);
    let mottle = zoom_fbm(p_km, r_km * 0.5, px_km, seed + 3) - 0.5;
    let acid = mix(ACID_TURQUOISE, ACID_EMERALD, (pick + 0.4 * mottle).clamp(0.0, 1.0));
    // The sulphur slick: thin pale blotches, a sixth of the surface at most.
    let slick = smoothstep(0.64, 0.74, zoom_fbm(p_km, r_km * 0.25, px_km, seed + 5)) * 0.45;
    let acid = mix(acid, SULPHUR_SLICK, slick);
    mix(base, acid, ACID_STRENGTH * w * strength.min(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects owner Ruling CX item 5: a lake pixel at the centre of an
    /// active or a dormant crater turns turquoise-to-emerald (green and blue
    /// both above red by a margin, literals), an extinct crater's lake and a
    /// lake outside every summit zone keep `base` exactly, strength 0 is
    /// identity, and the mean colour over a crater stays put across a 100x
    /// zoom (the zoom law: detail is added, tone is not moved).
    #[test]
    fn active_and_dormant_crater_lakes_are_acid_and_extinct_ones_are_not() {
        use cartalith_terrain::{PlacedEdifice, VolcanicSetting};
        let ed = |years: f64| PlacedEdifice { setting: VolcanicSetting::Arc, cx: 20.0, cy: 20.0, r: 10.0, h: 0.2, kind: Some(EdificeKind::Cinder), age: 0.0, years: Some(years), crater: None };
        let base: Rgb = (40.0, 70.0, 110.0);
        for (years, acid) in [(10.0, true), (5_000.0, true), (2_000_000.0, false)] {
            let idx = VolcanoIndex::new(&[ed(years)], 64, 48);
            let c = crater_lake(&idx, 20.3, 20.2, 0.4, 0.01, base, 1.0);
            if acid {
                assert!(c.1 > c.0 + 40.0 && c.2 > c.0 + 20.0, "{years} yr: {c:?} is not turquoise-emerald");
            } else {
                assert_eq!(c, base, "an extinct crater lake keeps the lake palette");
            }
            assert_eq!(crater_lake(&idx, 20.3, 20.2, 0.4, 0.01, base, 0.0), base, "strength 0 is identity");
            if acid {
                assert_ne!(crater_lake(&idx, 20.3, 20.2, 0.4, 0.01, base, 0.5), c, "half strength is half the tint");
            }
            assert_eq!(crater_lake(&idx, 20.0 + 8.0, 20.0, 0.4, 0.01, base, 1.0), base, "outside the summit zone");
        }
        let idx = VolcanoIndex::new(&[ed(10.0)], 64, 48);
        let mean = |px: f64| {
            let n = 60;
            let mut s = (0.0, 0.0, 0.0);
            for k in 0..n * n {
                let (x, y) = (20.0 + ((k % n) as f64 / n as f64 - 0.5) * 3.0, 20.0 + ((k / n) as f64 / n as f64 - 0.5) * 3.0);
                let c = crater_lake(&idx, x, y, 0.4, px, base, 1.0);
                s = (s.0 + c.0, s.1 + c.1, s.2 + c.2);
            }
            let m = (n * n) as f64;
            (s.0 / m, s.1 / m, s.2 / m)
        };
        let (a, b) = (mean(0.2), mean(0.002));
        assert!((a.0 - b.0).abs() < 8.0 && (a.1 - b.1).abs() < 8.0 && (a.2 - b.2).abs() < 8.0, "tone moved with zoom: {a:?} vs {b:?}");
    }

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

    /// Protects the lookup's reach against the drawn margin: the widest lobe
    /// times the widest ragged margin must stay inside the half widths the
    /// terrain crate's lattice answers for, or a flow is cut by a straight
    /// line along a bucket edge. Not a constant against itself: two
    /// constants from two crates, one bounding the other.
    #[test]
    fn the_lookup_reach_covers_the_widest_margin() {
        assert!(cartalith_terrain::lava_flows::REACH_HW >= (1.0 + LOBE_AMP) * (1.0 + MARGIN_AMP), "reach {} under {}", cartalith_terrain::lava_flows::REACH_HW, (1.0 + LOBE_AMP) * (1.0 + MARGIN_AMP));
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
