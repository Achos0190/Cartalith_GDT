//! Per-volcano attributes derived from the edifice record (VA-1,
//! `VOLCANO_APPEARANCE_RESEARCH.md` §6.1 and §7, owner Ruling CW 2026-10-24).
//!
//! # What this is
//!
//! The stampers record every edifice they place ([`crate::PlacedEdifice`]:
//! setting, centre, radius, height, morphology and its own `0..=1` age draw).
//! This module turns that record into what a consumer that is not the stamp
//! needs: the age in **years**, the **activity** state the owner approved, the
//! summit-zone fractions of each morphology, and a spatial index that answers
//! "which edifice owns this point, and how far out on it is the point" at a
//! **fractional** grid position.
//!
//! # Why
//!
//! Before VA-1 the age draw and the morphology were used to shape the stamp
//! and then discarded (`PlacedEdifice` held `setting, cx, cy, r, h` only, and
//! the trace was dropped after the geology stage), so the renderer could not
//! tell a fresh cinder cone from a 2 Ma shield and painted every volcanic cell
//! as andesite (research §4.1, §4.3; re-opened at `stamp_one_volcano` and
//! `cartalith_civ::build_lithology` before this was written).
//!
//! # What it must never do
//!
//! - Never change the height field or `volcanic_field`: it reads the record
//!   after generation and writes nothing back.
//! - Never invent a value: an edifice drawn by the reference model has no
//!   morphology (`kind: None`) and every function here that needs one returns
//!   `None` rather than a default kind.

use crate::{EdificeKind, PlacedEdifice, VolcanicSetting};

// # Two age maps, and which one a row is read under
//
// Ruling CW (2026-10-24) approved research §6.1's map, `10^(1 + 5.5·draw)`,
// with activity cut on the draw (active < 0.05, dormant < 0.4). The draw is
// `rng × volc.age` (0.40 by default) for every arc, rift and scattered
// volcano, so on that map every one of them was 10 years to about 1.6 ka and
// almost none was extinct (VA-1's census: 0-3 extinct per world, all on
// hotspot chains), and the research's own reason for the 0.4 threshold --
// "about 500 ka on the log map" -- did not hold (500 ka is a draw of 0.855
// there). Owner Ruling CX item 2 (2026-10-24) re-spread the ages: the
// re-spread map ([`respread_age_years`]) puts 500 ka at a draw of
// [`RESPREAD_KNEE`], and activity is cut on the years themselves. Every
// edifice stamped since records its years (`PlacedEdifice::years`); a table
// saved before has none and is read under the old map ([`legacy_age_years`],
// [`legacy_activity_of`]), so an old project reopens as it was saved. The
// draw itself, the stamp and `volcanic_field` are untouched by either map.

/// **Ruling CW's map, kept only to read tables saved before Ruling CX.**
/// Edifice age draws below this were **active** (research §8 question 3:
/// "active <0.05"), younger than about 18 years on [`legacy_age_years`].
pub const LEGACY_ACTIVE_BELOW: f64 = 0.05;

/// **Ruling CW's map, kept only to read tables saved before Ruling CX.**
/// Draws below this (and at or above [`LEGACY_ACTIVE_BELOW`]) were
/// **dormant**, at or above it **extinct** ("dormant <0.4"; about 1.6 ka on
/// [`legacy_age_years`] -- the mismatch with the research's "500 ka" that
/// Ruling CX item 2 corrected).
pub const LEGACY_DORMANT_BELOW: f64 = 0.4;

/// `log10` of the years at a draw of 0 (10 years) and the decades the draw
/// spans (5.5, so a draw of 1 is `10^6.5` ≈ 3.16 Myr), on both maps: Ruling
/// CW's "a per-volcano age in years on a log scale up to 3 million years".
/// The span matches the cited ages (Wood 1980's 0.001-6 Ma cinder cones,
/// Davidson & De Silva 2000's composite systems to about 500 ka, Hawaii's
/// <0.7-5.6 Ma islands).
pub const AGE_LOG10_MIN: f64 = 1.0;
/// See [`AGE_LOG10_MIN`].
pub const AGE_LOG10_SPAN: f64 = 5.5;

/// **Ruling CW's map, kept only to read tables saved before Ruling CX**: the
/// age in years of an edifice whose `0..=1` draw is `age01`,
/// `10^(1 + 5.5·age01)`. Clamped to the draw's own range first, so a value
/// outside it maps to the nearest end rather than to an extrapolated age.
pub fn legacy_age_years(age01: f64) -> f64 {
    10f64.powf(AGE_LOG10_MIN + AGE_LOG10_SPAN * age01.clamp(0.0, 1.0))
}

/// **Ruling CX item 2**: the oldest an edifice can be and still be
/// **dormant**; older is **extinct**. About 500 ka, the research's §6.1
/// rationale (composite systems stay active up to about 500 ka, cited:
/// Davidson & De Silva 2000).
pub const DORMANT_LIMIT_YEARS: f64 = 500_000.0;

/// **Ruling CX item 2**: an edifice that erupted less than this many years
/// ago is **active** (an open vent or a lava lake in the crater; the glow
/// and the dry crater of item 6 follow it). A labelled judgement: "within
/// living memory". At the default `volc.age` 0.40 it makes about 9 % of the
/// arc, rift and scattered volcanoes active (a draw below about 0.037).
pub const ACTIVE_LIMIT_YEARS: f64 = 50.0;

/// **Ruling CX item 2**: the draw at which the re-spread map reaches
/// [`DORMANT_LIMIT_YEARS`]. A labelled judgement chosen for the mix it gives
/// at the default `volc.age` 0.40, whose draws are uniform on `0..0.40`:
/// about 9 % active (under 50 years), 31 % dormant and young (under 10 ka,
/// mostly bare), 22 % dormant and old (10-500 ka, greening), 38 % extinct
/// (500-720 ka). Raising `volc.age` gives an older mix, lowering it a younger
/// one, as the slider promised before. Hotspot chains, whose draws run
/// 0..~1 along the chain, keep extinct members from about a quarter of the
/// way along.
pub const RESPREAD_KNEE: f64 = 0.25;

/// **Ruling CX item 2**: the age in years of an edifice drawn at `age01`, on
/// a log scale in two straight pieces: 10 years at 0, [`DORMANT_LIMIT_YEARS`]
/// at [`RESPREAD_KNEE`], `10^6.5` (about 3.16 Myr, Ruling CW's ceiling) at 1.
/// Monotone, so an older draw is never a younger volcano. Clamped to the
/// draw's own range first. The draw is the stamp's own value, so neither the
/// height field nor any random draw moves.
pub fn respread_age_years(age01: f64) -> f64 {
    let a = age01.clamp(0.0, 1.0);
    let knee = DORMANT_LIMIT_YEARS.log10();
    let top = AGE_LOG10_MIN + AGE_LOG10_SPAN;
    let lg = if a <= RESPREAD_KNEE {
        AGE_LOG10_MIN + (knee - AGE_LOG10_MIN) * a / RESPREAD_KNEE
    } else {
        knee + (top - knee) * (a - RESPREAD_KNEE) / (1.0 - RESPREAD_KNEE)
    };
    10f64.powf(lg)
}

/// The edifice's age in years under the map its row was recorded with: its
/// own `years` (every edifice stamped since Ruling CX), else Ruling CW's map
/// of its draw (a table saved before).
pub fn years(e: &PlacedEdifice) -> f64 {
    e.years.unwrap_or_else(|| legacy_age_years(e.age))
}

/// An edifice's eruptive state, derived from its age draw (Ruling CW: derived,
/// with a per-volcano override left to a later GUI row).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity {
    /// Erupting or freshly erupted: open vent or lava lake, sulphur at the
    /// vent, no vegetation in the crater.
    Active,
    /// Not erupting but not dead: hydrothermal alteration still stains the
    /// summit (research §3.3: alteration "persists after activity ends").
    Dormant,
    /// No activity; the edifice only weathers and greens.
    Extinct,
}

impl Activity {
    /// A stable key for a reader (the Sample panel, a census print). Never
    /// derived from `Debug`.
    pub fn key(self) -> &'static str {
        match self {
            Activity::Active => "active",
            Activity::Dormant => "dormant",
            Activity::Extinct => "extinct",
        }
    }
}

/// **Ruling CW's rule, kept only to read tables saved before Ruling CX**:
/// the activity of an edifice drawn at `age01` ([`LEGACY_ACTIVE_BELOW`],
/// [`LEGACY_DORMANT_BELOW`]).
pub fn legacy_activity_of(age01: f64) -> Activity {
    if age01 < LEGACY_ACTIVE_BELOW {
        Activity::Active
    } else if age01 < LEGACY_DORMANT_BELOW {
        Activity::Dormant
    } else {
        Activity::Extinct
    }
}

/// The activity of an edifice of `years` (Ruling CX item 2): active under
/// [`ACTIVE_LIMIT_YEARS`], dormant under [`DORMANT_LIMIT_YEARS`], else extinct.
pub fn activity_of_years(years: f64) -> Activity {
    if years < ACTIVE_LIMIT_YEARS {
        Activity::Active
    } else if years < DORMANT_LIMIT_YEARS {
        Activity::Dormant
    } else {
        Activity::Extinct
    }
}

/// The edifice's activity under the map its row was recorded with: by its
/// years ([`activity_of_years`]) when it carries them, else Ruling CW's draw
/// thresholds ([`legacy_activity_of`]), so an old table keeps its states.
pub fn activity(e: &PlacedEdifice) -> Activity {
    match e.years {
        Some(y) => activity_of_years(y),
        None => legacy_activity_of(e.age),
    }
}

/// How young an **active** edifice is inside the active band, `0..=1`: 1 at
/// 10 years, 0 at the band's upper edge, on the log of its years. The band's
/// edge is the map the row was recorded with ([`ACTIVE_LIMIT_YEARS`], or
/// Ruling CW's draw 0.05 for an old table, where this is exactly the old
/// `1 - age/0.05`). For the vent's heat (`volcano_look`); `0` past the band.
pub fn active_youth(e: &PlacedEdifice) -> f64 {
    let limit = if e.years.is_some() { ACTIVE_LIMIT_YEARS } else { legacy_age_years(LEGACY_ACTIVE_BELOW) };
    let span = limit.log10() - AGE_LOG10_MIN;
    (1.0 - (years(e).log10() - AGE_LOG10_MIN) / span).clamp(0.0, 1.0)
}

impl EdificeKind {
    /// Every kind, in [`EdificeKind::code`] order.
    pub const ALL: [EdificeKind; 5] = [EdificeKind::Cinder, EdificeKind::Strato, EdificeKind::Shield, EdificeKind::Dome, EdificeKind::Maar];

    /// A stable storage / display key (the save's `volcanoes` table writes
    /// it). A format name: changing one orphans every archive that holds it.
    pub fn key(self) -> &'static str {
        match self {
            EdificeKind::Cinder => "cinder",
            EdificeKind::Strato => "strato",
            EdificeKind::Shield => "shield",
            // VA-6. A table holding either is refused by a build before
            // VA-6 ("unknown kind"), which is the honest answer.
            EdificeKind::Dome => "dome",
            EdificeKind::Maar => "maar",
        }
    }

    /// The inverse of [`EdificeKind::key`]; `None` for a key this build does
    /// not know (a reader refuses it rather than guessing a kind).
    pub fn from_key(key: &str) -> Option<EdificeKind> {
        EdificeKind::ALL.into_iter().find(|k| k.key() == key)
    }
}

/// A stable storage / display key for a volcanic setting (the save's
/// `volcanoes` table). A format name, like [`EdificeKind::key`].
pub fn setting_key(s: VolcanicSetting) -> &'static str {
    match s {
        VolcanicSetting::Arc => "arc",
        VolcanicSetting::Rift => "rift",
        VolcanicSetting::Hotspot => "hotspot",
        VolcanicSetting::Unclassified => "unclassified",
    }
}

/// The inverse of [`setting_key`]; `None` for a key this build does not know.
pub fn setting_from_key(key: &str) -> Option<VolcanicSetting> {
    [VolcanicSetting::Arc, VolcanicSetting::Rift, VolcanicSetting::Hotspot, VolcanicSetting::Unclassified]
        .into_iter()
        .find(|&s| setting_key(s) == key)
}

/// `(floor, rim)` of `kind`'s summit depression as fractions of the edifice
/// radius: the flat floor's radius and the rim's. Read from the stamp's own
/// shape constants (`edifice_shape`), so a renderer's crater zone and the
/// height field's crater are the same circle -- one source, never restated.
pub fn zone_fractions(kind: EdificeKind) -> (f64, f64) {
    let s = crate::edifice_shape(kind);
    (s.floor, s.rim)
}

/// `(floor, rim)` of this edifice's own summit depression: its recorded
/// VA-5 crater ([`PlacedEdifice::crater`]) when it has one, else its kind's
/// table ([`zone_fractions`]). `None` without a morphology.
pub fn zones(e: &PlacedEdifice) -> Option<(f64, f64)> {
    e.crater.or_else(|| e.kind.map(zone_fractions))
}

/// The summit zone of an edifice, as a fraction of its radius: the region
/// owner Ruling CO's flank-pit fill leaves alone (`edifice_pits::SUMMIT_T`)
/// or the crater rim with a quarter's margin, whichever is wider. The crater
/// region owner Ruling CX reads: item 6's dry active craters
/// (`edifice_pits::dry_active_craters`) and item 5's crater-lake colour.
/// `None` without a morphology. The margin is a judgement: a crater lake's
/// shore sits on the inner wall, just outside the rim radius the stamp's
/// shape table names.
pub fn summit_zone_t(e: &PlacedEdifice) -> Option<f64> {
    zones(e).map(|(_, rim)| (rim * 1.25).max(crate::edifice_pits::SUMMIT_T))
}

/// Whether an edifice's lava is basaltic (low-silica, dark): every shield and
/// every scoria cone outside a subduction arc (research §2.2: rift and hotspot
/// volcanism is basaltic; a scoria cone is "basalt to basaltic andesite").
/// Arc cinder cones are basaltic andesite and also count. Only arc
/// stratocones (and the unclassified stratocones of the simple placement) are
/// andesitic. `None` without a morphology.
pub fn is_basaltic(e: &PlacedEdifice) -> Option<bool> {
    let k = e.kind?;
    Some(match k {
        EdificeKind::Shield | EdificeKind::Cinder => true,
        EdificeKind::Strato => matches!(e.setting, VolcanicSetting::Rift | VolcanicSetting::Hotspot),
        // VA-6: a lava dome is dacite-rhyolite (research §2.2), a maar's
        // magma basaltic (phreatomagmatic, same source).
        EdificeKind::Dome => false,
        EdificeKind::Maar => true,
    })
}

/// An edifice's claim on a point at `t` of its radius: the stamp's own
/// `volcanic_field` candidate `(1 - t)(1 - age)`, at most 1 -- except a VA-6
/// dome or maar (`volcano_kinds::late_kinds`), which is stamped on the
/// finished field on top of whatever it sits on, so it claims its whole disc
/// over any stamped edifice: `1 + (1 - t)`. Tables without either kind (every
/// table saved before VA-6) answer exactly as before.
pub fn owner_value(e: &PlacedEdifice, t: f64) -> f64 {
    if matches!(e.kind, Some(EdificeKind::Dome | EdificeKind::Maar)) {
        1.0 + (1.0 - t)
    } else {
        (1.0 - t) * (1.0 - e.age)
    }
}

/// Bucket edge, in cells, of [`VolcanoIndex`]'s lattice. A judgement for
/// cost: the median edifice radius at 2048 cells across 800 km is about 15
/// cells, so a bucket holds a handful of candidates.
const BUCKET_CELLS: f64 = 16.0;

/// The edifices of one world with a coarse spatial lattice over them, so a
/// per-pixel lookup tests only the edifices whose disc can reach the pixel.
#[derive(Clone, Debug)]
pub struct VolcanoIndex {
    edifices: Vec<PlacedEdifice>,
    bw: usize,
    bh: usize,
    /// Per bucket, the indices (into `edifices`, ascending) of every edifice
    /// whose bounding box overlaps it.
    buckets: Vec<Vec<u32>>,
    /// The world's traced lava flows ([`crate::lava_flows`]), attached by
    /// [`Self::with_lava_flows`] only where a look draws them; `None` -- no
    /// flows -- otherwise, so an index built as before answers as before.
    flows: Option<std::sync::Arc<crate::lava_flows::LavaFlows>>,
}

/// The edifice that owns a point and where on it the point is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VolcanoHit<'a> {
    /// Index into [`VolcanoIndex::edifices`].
    pub index: usize,
    pub edifice: &'a PlacedEdifice,
    /// Distance from the centre over the radius, `0..=1`.
    pub t: f64,
}

impl VolcanoIndex {
    /// Indexes `edifices` over a `gw × gh` grid. Order is kept: it decides
    /// ties exactly as the stamp's placement order did.
    pub fn new(edifices: &[PlacedEdifice], gw: usize, gh: usize) -> VolcanoIndex {
        let bw = ((gw as f64 / BUCKET_CELLS).ceil() as usize).max(1);
        let bh = ((gh as f64 / BUCKET_CELLS).ceil() as usize).max(1);
        let mut buckets = vec![Vec::new(); bw * bh];
        for (k, e) in edifices.iter().enumerate() {
            if !(e.cx.is_finite() && e.cy.is_finite() && e.r.is_finite() && e.r > 0.0) {
                continue;
            }
            let b = |v: f64, n: usize| ((v / BUCKET_CELLS).floor().max(0.0) as usize).min(n - 1);
            let (x0, x1) = (b(e.cx - e.r, bw), b(e.cx + e.r, bw));
            let (y0, y1) = (b(e.cy - e.r, bh), b(e.cy + e.r, bh));
            for by in y0..=y1 {
                for bx in x0..=x1 {
                    buckets[by * bw + bx].push(k as u32);
                }
            }
        }
        VolcanoIndex { edifices: edifices.to_vec(), bw, bh, buckets, flows: None }
    }

    /// This index with the world's lava flows traced over `inp`'s height
    /// field ([`crate::lava_flows::LavaFlows::trace`] of the indexed
    /// edifices). The trace is a pure function of the table and the field,
    /// so every consumer that builds its index this way from the same world
    /// draws the same flows.
    pub fn with_lava_flows(mut self, inp: crate::lava_flows::TraceInput) -> VolcanoIndex {
        self.flows = Some(std::sync::Arc::new(crate::lava_flows::LavaFlows::trace(&self.edifices, inp)));
        self
    }

    /// The attached lava flows, if any ([`Self::with_lava_flows`]).
    pub fn lava_flows(&self) -> Option<&crate::lava_flows::LavaFlows> {
        self.flows.as_deref()
    }

    /// The indexed edifices, in placement order.
    pub fn edifices(&self) -> &[PlacedEdifice] {
        &self.edifices
    }

    /// The edifice owning the point `(x, y)` (grid cells, fractional), by the
    /// stamp's own rule (`stamp_one_volcano`'s `volcanic_field` candidate): the
    /// largest [`owner_value`] (`(1 - t)(1 - age)` for a stamped edifice) among the discs that reach the point
    /// (`d <= r`), strictly greater than zero, the earliest edifice on a tie.
    /// At an integer cell that is the cell the stamp gave the edifice (up to
    /// the `f32` rounding of the stored maximum); between cells it is the same
    /// rule continued, so an owner boundary is a smooth curve at any zoom
    /// rather than a cell staircase.
    pub fn winner_at(&self, x: f64, y: f64) -> Option<VolcanoHit<'_>> {
        if !(x.is_finite() && y.is_finite()) || x < -BUCKET_CELLS || y < -BUCKET_CELLS {
            return None;
        }
        let bx = ((x / BUCKET_CELLS).floor().max(0.0) as usize).min(self.bw - 1);
        let by = ((y / BUCKET_CELLS).floor().max(0.0) as usize).min(self.bh - 1);
        let mut best: Option<VolcanoHit<'_>> = None;
        let mut best_v = 0.0;
        for &k in &self.buckets[by * self.bw + bx] {
            let e = &self.edifices[k as usize];
            let d = (x - e.cx).hypot(y - e.cy);
            if d > e.r {
                continue;
            }
            let t = d / e.r;
            let v = owner_value(e, t);
            if v > best_v {
                best_v = v;
                best = Some(VolcanoHit { index: k as usize, edifice: e, t });
            }
        }
        best
    }

    /// The two strongest owners of `(x, y)` by [`Self::winner_at`]'s rule,
    /// with their owner values ([`owner_value`]): the winner first, then the
    /// runner-up if a second disc reaches the point. For a consumer that
    /// must cross an owner boundary without a seam (the renderer blends the
    /// two near the boundary, where the values meet). The winner is exactly
    /// [`Self::winner_at`]'s.
    pub fn top2_at(&self, x: f64, y: f64) -> Option<((VolcanoHit<'_>, f64), Option<(VolcanoHit<'_>, f64)>)> {
        if !(x.is_finite() && y.is_finite()) || x < -BUCKET_CELLS || y < -BUCKET_CELLS {
            return None;
        }
        let bx = ((x / BUCKET_CELLS).floor().max(0.0) as usize).min(self.bw - 1);
        let by = ((y / BUCKET_CELLS).floor().max(0.0) as usize).min(self.bh - 1);
        let mut first: Option<(VolcanoHit<'_>, f64)> = None;
        let mut second: Option<(VolcanoHit<'_>, f64)> = None;
        for &k in &self.buckets[by * self.bw + bx] {
            let e = &self.edifices[k as usize];
            let d = (x - e.cx).hypot(y - e.cy);
            if d > e.r {
                continue;
            }
            let t = d / e.r;
            let v = owner_value(e, t);
            if v <= 0.0 {
                continue;
            }
            let hit = (VolcanoHit { index: k as usize, edifice: e, t }, v);
            match first {
                Some((_, fv)) if v > fv => {
                    second = first;
                    first = Some(hit);
                }
                None => first = Some(hit),
                _ => {
                    if second.is_none_or(|(_, sv)| v > sv) {
                        second = Some(hit);
                    }
                }
            }
        }
        first.map(|f| (f, second))
    }
}

/// [`VolcanoIndex::winner_at`] without an index: every edifice tested. For a
/// one-off read (the Sample panel's cell) where building the lattice would
/// cost more than the scan; the index's own test holds the two equal.
pub fn owner_of(edifices: &[PlacedEdifice], x: f64, y: f64) -> Option<VolcanoHit<'_>> {
    let mut best: Option<VolcanoHit<'_>> = None;
    let mut best_v = 0.0;
    for (k, e) in edifices.iter().enumerate() {
        let d = (x - e.cx).hypot(y - e.cy);
        if !(d <= e.r) {
            continue;
        }
        let t = d / e.r;
        let v = owner_value(e, t);
        if v > best_v {
            best_v = v;
            best = Some(VolcanoHit { index: k, edifice: e, t });
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ed(cx: f64, cy: f64, r: f64, age: f64, kind: Option<EdificeKind>, setting: VolcanicSetting) -> PlacedEdifice {
        PlacedEdifice { rift_bearing: None, setting, cx, cy, r, h: 0.2, kind, age, years: None, crater: None }
    }

    /// Protects Ruling CW's map (still read for old tables) at its two ends
    /// and one interior point, against hand-worked literals: 10 years at 0,
    /// 10^6.5 at 1, 10^3.75 at 0.5; and the clamp outside the draw's range.
    #[test]
    fn the_legacy_age_map_is_ten_years_to_three_million() {
        assert!((legacy_age_years(0.0) - 10.0).abs() < 1e-9);
        assert!((legacy_age_years(1.0) - 3_162_277.660_168_379).abs() < 1e-3);
        assert!((legacy_age_years(0.5) - 5_623.413_251_903_491).abs() < 1e-6);
        assert_eq!(legacy_age_years(-1.0), legacy_age_years(0.0));
        assert_eq!(legacy_age_years(2.0), legacy_age_years(1.0));
    }

    /// Protects Ruling CW's draw thresholds (0.05, 0.4) at both sides of each
    /// boundary, still the rule for a table saved before Ruling CX.
    #[test]
    fn legacy_activity_follows_the_ruling_cw_thresholds() {
        assert_eq!(legacy_activity_of(0.0), Activity::Active);
        assert_eq!(legacy_activity_of(0.049), Activity::Active);
        assert_eq!(legacy_activity_of(0.05), Activity::Dormant);
        assert_eq!(legacy_activity_of(0.399), Activity::Dormant);
        assert_eq!(legacy_activity_of(0.4), Activity::Extinct);
        assert_eq!(legacy_activity_of(1.0), Activity::Extinct);
    }

    /// Protects Ruling CX item 2's map against hand-worked literals: 10 years
    /// at 0, 500 ka exactly at the knee 0.25, 10^6.5 at 1, 10^3.349485 at
    /// 0.125 (half way up the first piece: (1 + 5.69897)/2) and 10^6.099485
    /// at 0.625 (half way up the second: (5.69897 + 6.5)/2); monotone;
    /// clamped.
    #[test]
    fn the_respread_map_puts_500_ka_at_the_knee() {
        assert!((respread_age_years(0.0) - 10.0).abs() < 1e-9);
        assert!((respread_age_years(0.25) - 500_000.0).abs() < 1e-6);
        assert!((respread_age_years(1.0) - 3_162_277.660_168_379).abs() < 1e-3);
        assert!((respread_age_years(0.125).log10() - 3.349_485).abs() < 1e-5);
        assert!((respread_age_years(0.625).log10() - 6.099_485).abs() < 1e-5);
        let mut prev = 0.0;
        for k in 0..=1000 {
            let y = respread_age_years(k as f64 / 1000.0);
            assert!(y > prev, "not monotone at {k}");
            prev = y;
        }
        assert_eq!(respread_age_years(-1.0), respread_age_years(0.0));
        assert_eq!(respread_age_years(2.0), respread_age_years(1.0));
    }

    /// Protects the re-spread activity: 50 years and 500 ka are the
    /// boundaries (literals), on both sides; and a row with no years is read
    /// under Ruling CW's thresholds, one with years under the new ones -- the
    /// same draw 0.3 is dormant on an old table and extinct on a new one.
    #[test]
    fn activity_follows_the_years_or_the_old_draw_rule() {
        assert_eq!(activity_of_years(49.9), Activity::Active);
        assert_eq!(activity_of_years(50.0), Activity::Dormant);
        assert_eq!(activity_of_years(499_999.0), Activity::Dormant);
        assert_eq!(activity_of_years(500_000.0), Activity::Extinct);
        let old = ed(0.0, 0.0, 5.0, 0.3, Some(EdificeKind::Cinder), VolcanicSetting::Arc);
        assert_eq!(activity(&old), Activity::Dormant);
        // Ruling CW's thresholds are on the draw, not on its years: a draw
        // of 0.45 (about 3 ka) is extinct and 0.08 (about 28 years) dormant
        // on an old table, where cutting its legacy years at 50 yr / 500 ka
        // would read dormant and active.
        assert_eq!(activity(&PlacedEdifice { age: 0.45, ..old }), Activity::Extinct);
        assert_eq!(activity(&PlacedEdifice { age: 0.08, ..old }), Activity::Dormant);
        assert!((years(&old) - 10f64.powf(1.0 + 5.5 * 0.3)).abs() < 1e-6);
        let new = PlacedEdifice { years: Some(respread_age_years(0.3)), ..old };
        assert_eq!(activity(&new), Activity::Extinct);
        assert_eq!(years(&new), respread_age_years(0.3));
        // The default slider's draws (uniform on 0..0.40) give the mix the
        // knee's doc states: about 9 % active, 38 % extinct.
        let draws: Vec<f64> = (0..4000).map(|k| (k as f64 + 0.5) / 4000.0 * 0.40).collect();
        let n = |a: Activity| draws.iter().filter(|&&d| activity_of_years(respread_age_years(d)) == a).count() as f64 / draws.len() as f64;
        assert!((n(Activity::Active) - 0.093).abs() < 0.005, "active share {}", n(Activity::Active));
        assert!((n(Activity::Extinct) - 0.375).abs() < 0.005, "extinct share {}", n(Activity::Extinct));
    }

    /// Protects the vent heat's youth: 1 at 10 years, 0 at and past the band
    /// edge, half way at the log midpoint, and exactly Ruling CW's
    /// `1 - age/0.05` on an old table.
    #[test]
    fn active_youth_spans_the_active_band() {
        let mut e = ed(0.0, 0.0, 5.0, 0.0, Some(EdificeKind::Cinder), VolcanicSetting::Arc);
        assert!((active_youth(&e) - 1.0).abs() < 1e-12);
        e.age = 0.02;
        assert!((active_youth(&e) - 0.6).abs() < 1e-9, "old table: 1 - 0.02/0.05");
        e.years = Some(50.0);
        assert_eq!(active_youth(&e), 0.0);
        e.years = Some(10f64.powf(1.0 + 0.5 * (50f64.log10() - 1.0)));
        assert!((active_youth(&e) - 0.5).abs() < 1e-9);
    }

    /// Protects the kind keys as a format: each round-trips, an unknown key
    /// is refused rather than mapped to a kind.
    #[test]
    fn kind_keys_round_trip_and_unknown_is_refused() {
        for k in EdificeKind::ALL {
            assert_eq!(EdificeKind::from_key(k.key()), Some(k));
        }
        // VA-6 added "dome" and "maar" as format keys; an unknown key is
        // still refused.
        assert_eq!(EdificeKind::from_key("lava_dome"), None);
        assert_eq!(EdificeKind::Dome.key(), "dome");
        assert_eq!(EdificeKind::Maar.key(), "maar");
        assert_eq!(EdificeKind::Cinder.key(), "cinder");
        assert_eq!(EdificeKind::Strato.key(), "strato");
        assert_eq!(EdificeKind::Shield.key(), "shield");
        for s in [VolcanicSetting::Arc, VolcanicSetting::Rift, VolcanicSetting::Hotspot, VolcanicSetting::Unclassified] {
            assert_eq!(setting_from_key(setting_key(s)), Some(s));
        }
        assert_eq!(setting_key(VolcanicSetting::Hotspot), "hotspot");
        assert_eq!(setting_from_key("transform"), None);
    }

    /// Protects that the zone fractions are the stamp's own constants
    /// (literals copied from `edifice_shape`'s table: a mutation there must
    /// show here too, which is the point of reading rather than restating).
    #[test]
    fn zone_fractions_are_the_stamps_crater() {
        assert_eq!(zone_fractions(EdificeKind::Cinder), (0.16, 0.38));
        assert_eq!(zone_fractions(EdificeKind::Strato), (0.11, 0.24));
        assert_eq!(zone_fractions(EdificeKind::Shield), (0.05, 0.12));
        // A recorded crater wins over the table; none falls back to it.
        let mut e = ed(0.0, 0.0, 10.0, 0.1, Some(EdificeKind::Strato), VolcanicSetting::Arc);
        assert_eq!(zones(&e), Some((0.11, 0.24)));
        e.crater = Some((0.02, 0.05));
        assert_eq!(zones(&e), Some((0.02, 0.05)));
        assert_eq!(zones(&ed(0.0, 0.0, 10.0, 0.1, None, VolcanicSetting::Arc)), None);
    }

    /// Protects the basaltic split: shields and cinder cones basaltic, arc
    /// stratocones andesitic, no answer without a morphology.
    #[test]
    fn basaltic_follows_kind_and_setting() {
        assert_eq!(is_basaltic(&ed(0.0, 0.0, 1.0, 0.0, Some(EdificeKind::Shield), VolcanicSetting::Hotspot)), Some(true));
        assert_eq!(is_basaltic(&ed(0.0, 0.0, 1.0, 0.0, Some(EdificeKind::Cinder), VolcanicSetting::Arc)), Some(true));
        assert_eq!(is_basaltic(&ed(0.0, 0.0, 1.0, 0.0, Some(EdificeKind::Strato), VolcanicSetting::Arc)), Some(false));
        assert_eq!(is_basaltic(&ed(0.0, 0.0, 1.0, 0.0, Some(EdificeKind::Strato), VolcanicSetting::Unclassified)), Some(false));
        assert_eq!(is_basaltic(&ed(0.0, 0.0, 1.0, 0.0, None, VolcanicSetting::Arc)), None);
    }

    /// Protects the owner rule: the larger `(1-t)(1-age)` wins, outside every
    /// disc there is no owner, and on an exact tie the earlier edifice keeps
    /// the point (the stamp's strict `>`).
    #[test]
    fn the_winner_is_the_stamps_rule_at_fractional_points() {
        let a = ed(20.0, 20.0, 10.0, 0.0, Some(EdificeKind::Strato), VolcanicSetting::Arc);
        let b = ed(32.0, 20.0, 10.0, 0.5, Some(EdificeKind::Cinder), VolcanicSetting::Arc);
        let idx = VolcanoIndex::new(&[a, b], 64, 48);
        // At x = 28: a has t 0.8 -> 0.2; b has t 0.4 -> 0.6 * 0.5 = 0.3.
        let h = idx.winner_at(28.0, 20.0).unwrap();
        assert_eq!(h.index, 1);
        assert!((h.t - 0.4).abs() < 1e-12);
        // At x = 26: a 0.4, b t 0.6 -> 0.2: a wins.
        assert_eq!(idx.winner_at(26.0, 20.0).unwrap().index, 0);
        assert!(idx.winner_at(5.0, 40.0).is_none());
        // A tie: two identical discs; the first keeps it.
        let idx2 = VolcanoIndex::new(&[a, a], 64, 48);
        assert_eq!(idx2.winner_at(21.0, 20.0).unwrap().index, 0);
        // An age-1 edifice never owns a point (its candidate is 0), as it
        // never raises `volcanic_field`.
        let dead = ed(20.0, 20.0, 10.0, 1.0, Some(EdificeKind::Shield), VolcanicSetting::Hotspot);
        assert!(VolcanoIndex::new(&[dead], 64, 48).winner_at(20.0, 20.0).is_none());
    }

    /// Protects the lattice: an edifice whose disc crosses a bucket edge is
    /// found from either side, so the index changes nothing but cost (the
    /// brute-force owner equals the indexed one at every point of a sweep).
    #[test]
    fn the_index_agrees_with_brute_force() {
        let es: Vec<PlacedEdifice> = (0..12)
            .map(|k| {
                let f = k as f64;
                ed(5.0 + f * 9.7, 7.0 + (f * 13.3) % 40.0, 3.0 + (f * 5.1) % 14.0, (f * 0.37) % 1.0, Some(EdificeKind::Cinder), VolcanicSetting::Rift)
            })
            .collect();
        let idx = VolcanoIndex::new(&es, 128, 64);
        let brute = |x: f64, y: f64| owner_of(&es, x, y).map(|h| h.index);
        let mut hits = 0;
        for yi in 0..128 {
            for xi in 0..256 {
                let (x, y) = (xi as f64 * 0.5 + 0.13, yi as f64 * 0.5 + 0.07);
                let got = idx.winner_at(x, y).map(|h| h.index);
                assert_eq!(got, brute(x, y), "at ({x}, {y})");
                hits += usize::from(got.is_some());
            }
        }
        assert!(hits > 1000, "the sweep reached too few edifice points ({hits})");
        // `top2_at`'s winner is `winner_at`'s, and its runner-up is never
        // stronger than the winner.
        let mut seconds = 0;
        for yi in 0..64 {
            for xi in 0..128 {
                let (x, y) = (xi as f64 + 0.3, yi as f64 + 0.6);
                let t2 = idx.top2_at(x, y);
                assert_eq!(t2.map(|((h, _), _)| h.index), idx.winner_at(x, y).map(|h| h.index));
                if let Some(((_, v1), Some((h2, v2)))) = t2 {
                    assert!(v2 <= v1);
                    assert_ne!(Some(h2.index), idx.winner_at(x, y).map(|h| h.index));
                    seconds += 1;
                }
            }
        }
        assert!(seconds > 20, "the sweep reached too few overlaps ({seconds})");
    }
}
