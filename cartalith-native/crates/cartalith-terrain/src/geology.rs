//! **The lithology model and the rock column** -- GF-1 of
//! `GEOLOGY_FIRST_SCOPE.md` (owner Rulings BH and BJ).
//!
//! # What this is
//!
//! Eleven rock types (§2.2), one property table (§2.3), a derivation from
//! pre-erosion causes only (§2.4), and a two-layer column per cell (§2.5).
//! `cartalith-engine`'s `generate_terrain` builds it in a new geology stage,
//! after volcanism, craters and sea level and before the first flow.
//!
//! **At GF-1 no shaping process reads any of it.** It is stored on
//! `WorldState` and shown in the Sample panel. GF-2 onward makes the kernels
//! read the table.
//!
//! # Why it is not `cartalith_civ::build_lithology`
//!
//! The legacy label is derived from the *finished* terrain and climate:
//! limestone, sandstone and shale are "post-erosion lowland" split by
//! *present* rainfall. A model that erosion reads cannot be defined by
//! erosion's output, and a harness measuring "weak rock became lowland"
//! against it would find exactly what it defined (§5.1). Every input here
//! exists before erosion and before any climate is computed. The legacy
//! function stays, unchanged, for the civ layer until GF-9.
//!
//! # Numbers
//!
//! Every constant below is labelled either **verified** (opened at its cited
//! source for GF-1, with where) or a **judgement** (with what tunes it).
//! Nothing is labelled verified that was not opened.

use crate::btype;
use cartalith_noise::{fbm, pfbm};

/// The number of rock types (Ruling BJ: 11).
pub const ROCK_COUNT: usize = 11;

/// `rock_sub`'s "there is no second layer". Never a valid rock index
/// (those are `0..ROCK_COUNT`), and the value `GEOLOGY_FIRST_SCOPE.md` §2.6
/// reserves for the save format.
pub const NO_LAYER: u8 = 255;

/// The regolith thickness above which a cell's exposed rock is
/// [`Rock::Unconsolidated`], in metres (§2.5, amended for Ruling BJ).
/// **Judgement**: above the clock's largest weathered mantle (2.69 m, §4.12
/// arithmetic) and below the alluvium, talus and marine sediment that should
/// hide bedrock.
pub const R_EXPOSE_M: f64 = 5.0;

/// The eleven rock types of §2.2, in its order. The discriminant is the
/// stored `u8`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Rock {
    Granite = 0,
    Gneiss = 1,
    Schist = 2,
    PlateauBasalt = 3,
    OceanicBasalt = 4,
    Andesite = 5,
    Tuff = 6,
    Limestone = 7,
    Sandstone = 8,
    Shale = 9,
    Unconsolidated = 10,
}

impl Rock {
    pub const ALL: [Rock; ROCK_COUNT] = [
        Rock::Granite,
        Rock::Gneiss,
        Rock::Schist,
        Rock::PlateauBasalt,
        Rock::OceanicBasalt,
        Rock::Andesite,
        Rock::Tuff,
        Rock::Limestone,
        Rock::Sandstone,
        Rock::Shale,
        Rock::Unconsolidated,
    ];

    /// `None` for anything that is not a rock index, [`NO_LAYER`] included.
    pub fn from_u8(v: u8) -> Option<Rock> {
        Rock::ALL.get(v as usize).copied()
    }

    pub fn props(self) -> &'static RockProps {
        &ROCK_PROPS[self as usize]
    }
}

/// Selby's (1980) rock-mass-strength classes, used as labels.
///
/// The class boundaries the scope quotes (total rating: very weak < 26, weak
/// 26-50, moderate 51-70, strong 71-90, very strong 91-100) were **cited, not
/// verified**: Selby 1980 (*Z. Geomorph.* 24(1)) could not be opened for GF-1.
/// Nothing computes with them; each rock carries its class as data, and its
/// `s` lies inside that class's quoted band (asserted in the tests).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelbyClass {
    VeryWeak,
    Weak,
    Moderate,
    Strong,
    VeryStrong,
}

impl SelbyClass {
    pub fn name(self) -> &'static str {
        match self {
            SelbyClass::VeryWeak => "very weak",
            SelbyClass::Weak => "weak",
            SelbyClass::Moderate => "moderate",
            SelbyClass::Strong => "strong",
            SelbyClass::VeryStrong => "very strong",
        }
    }

    /// The quoted total-rating band, `[lo, hi]`, out of 100 (see the type's
    /// doc: cited, not verified).
    pub fn band(self) -> (f32, f32) {
        match self {
            SelbyClass::VeryWeak => (0.0, 25.0),
            SelbyClass::Weak => (26.0, 50.0),
            SelbyClass::Moderate => (51.0, 70.0),
            SelbyClass::Strong => (71.0, 90.0),
            SelbyClass::VeryStrong => (91.0, 100.0),
        }
    }
}

/// Permeability class (§2.3 column P). An ordinal class, not a number. The
/// ordering is the scope's reading of Freeze & Cherry (1979) Table 2.2, which
/// was **cited, not verified** for GF-1 (the table could not be opened).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permeability {
    VeryLow,
    Low,
    Moderate,
    High,
    /// Unconsolidated material: high for sand and gravel, low for clay and
    /// till (§2.3). The model does not know which, so it does not say.
    Variable,
}

impl Permeability {
    pub fn name(self) -> &'static str {
        match self {
            Permeability::VeryLow => "very low",
            Permeability::Low => "low",
            Permeability::Moderate => "moderate",
            Permeability::High => "high",
            Permeability::Variable => "variable (sand high, clay low)",
        }
    }
}

/// Jointing and layering (§2.3 column J). GF-6's LOD detail reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Jointing {
    Massive,
    Foliated,
    Columnar,
    Blocky,
    Bedded,
    Fissile,
    None,
}

/// Formation age as a relative order (§2.4 step 5): basement < cover <
/// volcanic < regolith. No years.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Formation {
    Basement = 0,
    Cover = 1,
    Volcanic = 2,
    Regolith = 3,
}

/// Hoek-Brown `mi` for intact rock, Marinos & Hoek (2000), Table 2 ("Values
/// of the constant mi for intact rock, by rock group"). **Verified** for GF-1
/// against the paper as published by Rocscience
/// (`2000-GSI-A-Geologically-Friendly-Tool-for-Rock-Mass-Strength-Estimation.pdf`).
/// `estimate` is the table's own parentheses ("values in parenthesis are
/// estimates").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoekBrownMi {
    pub mi: f32,
    pub plus_minus: f32,
    pub estimate: bool,
}

/// One rock's properties -- §2.3, one row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RockProps {
    pub name: &'static str,
    pub selby: SelbyClass,
    /// Normalised strength, 0-1. **Judgement** (§2.3), inside `selby`'s band.
    pub s: f32,
    /// Stream-power `K` multiplier. **Judgement**; ordering after Stock &
    /// Montgomery (1999) and Sklar & Dietrich (2001), not verified at source.
    /// GF-2 reads it.
    pub kappa: f32,
    /// Critical hillslope angle, degrees. **Judgement** after Selby (1980,
    /// 1993). GF-3 reads it.
    pub theta_c_deg: f32,
    /// Glacial erodibility multiplier. **Judgement**. GF-4 reads it.
    pub gamma: f32,
    /// Cliff-retreat multiplier. **Judgement**. GF-4 reads it.
    pub rho: f32,
    /// Solubility class σ: carbonate dissolution (Ford & Williams 2007).
    pub soluble: bool,
    pub permeability: Permeability,
    pub jointing: Jointing,
    /// `None` for unconsolidated material, which is not intact rock.
    pub mi: Option<HoekBrownMi>,
    pub formation: Formation,
    /// The legacy class (`cartalith_civ::LITH_NAMES` index) this projects to
    /// for civ, soil and render until those tables are extended (§2.2).
    pub legacy_class: u8,
}

const fn mi(mi: f32, plus_minus: f32, estimate: bool) -> Option<HoekBrownMi> {
    Some(HoekBrownMi { mi, plus_minus, estimate })
}

/// §2.3's table, indexed by [`Rock`]. Every value is asserted as a literal in
/// this module's tests.
///
/// Two departures from the scope's table, both from opening the source:
/// - **Limestone `mi`**: the scope gives "9-10". Table 2 has micritic 9 ± 2
///   and sparitic 10 ± 2, both in parentheses; the fine-grained (micritic)
///   entry is the one pinned.
/// - **Shale `mi`**: the scope gives "6 (mudstone 4)". Table 2 has no
///   *mudstone* entry: 4 ± 2 is **claystone**, and shale is (6 ± 2), an
///   estimate. Shale's 6 ± 2 is pinned.
pub const ROCK_PROPS: [RockProps; ROCK_COUNT] = [
    RockProps {
        name: "Granite / granitoid",
        selby: SelbyClass::Strong,
        s: 0.85,
        kappa: 0.30,
        theta_c_deg: 60.0,
        gamma: 0.5,
        rho: 0.10,
        soluble: false,
        permeability: Permeability::Low,
        jointing: Jointing::Massive,
        mi: mi(32.0, 3.0, false),
        formation: Formation::Basement,
        legacy_class: 0,
    },
    RockProps {
        name: "Gneiss",
        selby: SelbyClass::Strong,
        s: 0.80,
        kappa: 0.35,
        theta_c_deg: 55.0,
        gamma: 0.6,
        rho: 0.15,
        soluble: false,
        permeability: Permeability::Low,
        jointing: Jointing::Foliated,
        mi: mi(28.0, 5.0, false),
        formation: Formation::Basement,
        legacy_class: 6,
    },
    RockProps {
        name: "Schist / slate",
        selby: SelbyClass::Moderate,
        s: 0.60,
        kappa: 0.80,
        theta_c_deg: 40.0,
        gamma: 1.2,
        rho: 0.50,
        soluble: false,
        permeability: Permeability::Low,
        jointing: Jointing::Foliated,
        mi: mi(12.0, 3.0, false),
        formation: Formation::Basement,
        legacy_class: 6,
    },
    RockProps {
        name: "Plateau (flood) basalt",
        selby: SelbyClass::Strong,
        s: 0.75,
        kappa: 0.40,
        theta_c_deg: 60.0,
        gamma: 1.3,
        rho: 0.20,
        soluble: false,
        permeability: Permeability::High,
        jointing: Jointing::Columnar,
        mi: mi(25.0, 5.0, true),
        formation: Formation::Volcanic,
        legacy_class: 1,
    },
    RockProps {
        name: "Oceanic basalt",
        selby: SelbyClass::Strong,
        s: 0.75,
        kappa: 0.40,
        theta_c_deg: 45.0,
        gamma: 1.0,
        rho: 0.20,
        soluble: false,
        permeability: Permeability::High,
        jointing: Jointing::Blocky,
        mi: mi(25.0, 5.0, true),
        formation: Formation::Basement,
        legacy_class: 1,
    },
    RockProps {
        name: "Andesite",
        selby: SelbyClass::Moderate,
        s: 0.70,
        kappa: 0.50,
        theta_c_deg: 45.0,
        gamma: 1.0,
        rho: 0.30,
        soluble: false,
        permeability: Permeability::Moderate,
        jointing: Jointing::Blocky,
        mi: mi(25.0, 5.0, false),
        formation: Formation::Volcanic,
        legacy_class: 2,
    },
    RockProps {
        name: "Tuff / volcaniclastic",
        selby: SelbyClass::Weak,
        s: 0.35,
        kappa: 2.0,
        theta_c_deg: 35.0,
        gamma: 1.5,
        rho: 3.0,
        soluble: false,
        permeability: Permeability::Moderate,
        jointing: Jointing::Bedded,
        mi: mi(13.0, 5.0, true),
        formation: Formation::Volcanic,
        legacy_class: 2,
    },
    RockProps {
        name: "Limestone",
        selby: SelbyClass::Moderate,
        s: 0.70,
        kappa: 0.60,
        theta_c_deg: 55.0,
        gamma: 1.0,
        rho: 0.30,
        soluble: true,
        permeability: Permeability::High,
        jointing: Jointing::Bedded,
        mi: mi(9.0, 2.0, true),
        formation: Formation::Cover,
        legacy_class: 3,
    },
    RockProps {
        name: "Sandstone",
        selby: SelbyClass::Moderate,
        s: 0.60,
        kappa: 0.80,
        theta_c_deg: 50.0,
        gamma: 1.1,
        rho: 0.60,
        soluble: false,
        permeability: Permeability::Moderate,
        jointing: Jointing::Bedded,
        mi: mi(17.0, 4.0, false),
        formation: Formation::Cover,
        legacy_class: 4,
    },
    RockProps {
        name: "Shale / mudstone",
        selby: SelbyClass::Weak,
        s: 0.30,
        kappa: 2.5,
        theta_c_deg: 30.0,
        gamma: 1.8,
        rho: 1.5,
        soluble: false,
        permeability: Permeability::VeryLow,
        jointing: Jointing::Fissile,
        mi: mi(6.0, 2.0, true),
        formation: Formation::Cover,
        legacy_class: 5,
    },
    RockProps {
        name: "Unconsolidated",
        selby: SelbyClass::VeryWeak,
        s: 0.10,
        kappa: 4.0,
        theta_c_deg: 33.0,
        gamma: 3.0,
        rho: 10.0,
        soluble: false,
        permeability: Permeability::Variable,
        jointing: Jointing::None,
        mi: None,
        formation: Formation::Regolith,
        legacy_class: 5,
    },
];

// ===========================================================================
// Derivation constants (§2.4). Every one is a **judgement**, tuned by B10's
// rock-type-share report (§5.2). Widths are multiples of `tect.blur_r` in
// cells, so belts scale with the stress blur and so with plate size.
// ===========================================================================

/// Gneiss core of a collision orogen.
pub const W_CORE: f64 = 1.0;
/// Schist flanks of a collision orogen; also the "active margin" distance
/// sedimentary cover must clear.
pub const W_OROGEN: f64 = 3.0;
/// Batholith belt behind a continental arc (granite).
pub const W_ARC: f64 = 4.0;
/// Sheared belt along a continental transform (schist).
pub const W_SHEAR: f64 = 1.0;
/// Rift-basin fill (sandstone over shale).
pub const W_RIFT: f64 = 2.5;
/// Radius of the wide blur a structural low is measured against.
pub const W_LOW_BLUR: f64 = 4.0;
/// A structural low is deeper than this below its wide blur, metres.
pub const LOW_DEPTH_M: f64 = 30.0;
/// Cover thickness per metre of depression, before the facies factor.
pub const COVER_PER_DEPTH: f64 = 3.0;
/// Cover thinner than this leaves the basement exposed, metres.
pub const COVER_MIN_M: f64 = 20.0;
/// The cap's share of the cover; the rest is the shale beneath.
pub const CAP_FRACTION: f64 = 0.5;
/// Rift-fill sandstone cap thickness, metres.
pub const RIFT_CAP_M: f64 = 150.0;
/// `volcanic_field` above which the volcanic overlay applies. The legacy
/// `build_lithology` uses `> 0.35`; this is lower so flanks are covered too.
pub const V_TH: f64 = 0.2;
/// Normalised edifice radius inside which an arc edifice is andesite; tuff
/// beyond it.
pub const ARC_CORE_T: f64 = 0.4;
/// |latitude| below which a cover cap is limestone rather than sandstone,
/// degrees. Present latitude stands in for palaeolatitude (disclosed, §2.4).
pub const CARBONATE_LAT_DEG: f64 = 30.0;
/// Facies noise frequency, cycles across the map width.
pub const FACIES_CYCLES: f64 = 3.0;
/// Seed salt for the facies noise.
const FACIES_SALT: i32 = 0x6e01;

// ===========================================================================
// The column
// ===========================================================================

/// `volcanic_setting`'s stored codes. §2.6 lists 0-3; `UNCLASSIFIED` (4) is
/// `stamp_volcanoes_simple`'s placements (`volc.provinces = false`, off at
/// both boundaries), which the scope's table has no code for. Flagged for
/// §2.6 rather than folded into "none", which would claim no volcano.
pub mod setting_code {
    pub const NONE: u8 = 0;
    pub const ARC: u8 = 1;
    pub const RIFT: u8 = 2;
    pub const HOTSPOT: u8 = 3;
    pub const UNCLASSIFIED: u8 = 4;
}

fn setting_code_of(s: crate::VolcanicSetting) -> u8 {
    match s {
        crate::VolcanicSetting::Arc => setting_code::ARC,
        crate::VolcanicSetting::Rift => setting_code::RIFT,
        crate::VolcanicSetting::Hotspot => setting_code::HOTSPOT,
        crate::VolcanicSetting::Unclassified => setting_code::UNCLASSIFIED,
    }
}

/// The per-cell column of §2.5, plus the volcanic setting (§2.4).
#[derive(Clone, Debug)]
pub struct GeologyColumn {
    /// Uppermost bedrock unit, a [`Rock`] index.
    pub rock_top: Vec<u8>,
    /// The unit beneath the contact, or [`NO_LAYER`].
    pub rock_sub: Vec<u8>,
    /// **Absolute** normalised elevation of the cap/substrate contact.
    /// **NaN** wherever `rock_sub` is [`NO_LAYER`], so a reader that forgets
    /// the gate reads NaN, never a plausible elevation (§2.6's writer rule,
    /// adopted in memory too). Read it through [`GeologyColumn::substrate`].
    pub contact: Vec<f32>,
    /// Unconsolidated thickness on bedrock, normalised height units. `0.0` is
    /// a real value: bare rock. GF-1 writes 0 everywhere (§3.1); deposition
    /// becomes regolith at GF-2.
    pub regolith: Vec<f32>,
    /// [`setting_code`] of the edifice that won the cell's `volcanic_field`.
    pub volcanic_setting: Vec<u8>,
}

impl GeologyColumn {
    pub fn len(&self) -> usize {
        self.rock_top.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rock_top.is_empty()
    }

    /// The uppermost bedrock unit.
    pub fn top(&self, i: usize) -> Option<Rock> {
        self.rock_top.get(i).copied().and_then(Rock::from_u8)
    }

    /// The unit beneath the contact and the contact's absolute normalised
    /// elevation, or `None` for a single-layer column. The only sanctioned
    /// reader of `contact`.
    pub fn substrate(&self, i: usize) -> Option<(Rock, f32)> {
        let sub = Rock::from_u8(*self.rock_sub.get(i)?)?;
        let c = *self.contact.get(i)?;
        (!c.is_nan()).then_some((sub, c))
    }

    /// §2.5's exposure rule at surface height `surface` (normalised).
    /// `r_expose` is [`R_EXPOSE_M`] in normalised units ([`m_to_norm`]).
    pub fn exposed(&self, i: usize, surface: f32, r_expose: f32) -> Option<Rock> {
        if self.regolith.get(i).copied()? > r_expose {
            return Some(Rock::Unconsolidated);
        }
        match self.substrate(i) {
            Some((sub, c)) if surface < c => Some(sub),
            _ => self.top(i),
        }
    }

    /// What lies beneath the exposed rock, and how far down, in normalised
    /// units -- the Sample panel's *Beneath* row.
    pub fn beneath(&self, i: usize, surface: f32, r_expose: f32) -> Beneath {
        let reg = self.regolith.get(i).copied().unwrap_or(0.0);
        if reg > r_expose {
            // Bedrock under the regolith: whichever unit the bedrock
            // surface exposes.
            let bed = surface - reg;
            return match self.exposed_bedrock(i, bed) {
                Some(r) => Beneath::Unit { rock: r, depth: reg },
                None => Beneath::NoData,
            };
        }
        match self.substrate(i) {
            None => Beneath::SingleLayer,
            Some((sub, c)) if surface >= c => Beneath::Unit { rock: sub, depth: surface - c },
            Some(_) => Beneath::CapEroded,
        }
    }

    fn exposed_bedrock(&self, i: usize, bed: f32) -> Option<Rock> {
        match self.substrate(i) {
            Some((sub, c)) if bed < c => Some(sub),
            _ => self.top(i),
        }
    }
}

/// The Sample panel's *Beneath* reading. Each non-`Unit` arm is a reason,
/// never a value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Beneath {
    /// `rock` lies `depth` (normalised units) below the surface.
    Unit { rock: Rock, depth: f32 },
    /// A single-layer column: nothing different lies beneath.
    SingleLayer,
    /// A two-layer column whose cap is eroded through here: the substrate is
    /// what is exposed, and the model records nothing below it.
    CapEroded,
    /// The column is shorter than the grid.
    NoData,
}

/// Metres to normalised height units: `metersPerUnit`'s anchoring, `1 - sea`
/// maps to `peak_m`.
pub fn m_to_norm(m: f64, sea_level: f64, peak_m: f64) -> f64 {
    if peak_m <= 0.0 {
        0.0
    } else {
        m * (1.0 - sea_level) / peak_m
    }
}

// ===========================================================================
// Labelled boundary distance (§2.4 step 1)
// ===========================================================================

/// A two-pass chamfer distance from the nearest boundary cell, like
/// `build_age_field`'s, that also carries **which** boundary cell is nearest,
/// so its type and plate can be read. Returns `(distance in cells, nearest
/// boundary index or u32::MAX where the map has no boundary)`.
///
/// Unlike `build_age_field` (whose golden pins its non-wrapping), this wraps
/// in x when `world`: each pass reads its seam neighbour, and the pair of
/// passes runs twice so a distance can cross the seam in either direction.
/// A separate function, so that golden is untouched (§2.4).
pub fn labelled_boundary_distance(gw: usize, gh: usize, world: bool, boundary_mask: &[u8]) -> (Vec<f32>, Vec<u32>) {
    const D2: f32 = std::f32::consts::SQRT_2;
    let n = gw * gh;
    let mut d = vec![f32::INFINITY; n];
    let mut src = vec![u32::MAX; n];
    for i in 0..n {
        if boundary_mask[i] != 0 {
            d[i] = 0.0;
            src[i] = i as u32;
        }
    }
    if gw == 0 || gh == 0 {
        return (d, src);
    }
    let relax = |d: &mut [f32], src: &mut [u32], idx: usize, nb: usize, cost: f32| {
        let v = d[nb] + cost;
        if v < d[idx] {
            d[idx] = v;
            src[idx] = src[nb];
        }
    };
    let left = |x: usize| if x > 0 { Some(x - 1) } else if world { Some(gw - 1) } else { None };
    let right = |x: usize| if x + 1 < gw { Some(x + 1) } else if world { Some(0) } else { None };
    let sweeps = if world { 2 } else { 1 };
    for _ in 0..sweeps {
        for y in 0..gh {
            for x in 0..gw {
                let idx = y * gw + x;
                if let Some(xl) = left(x) {
                    relax(&mut d, &mut src, idx, y * gw + xl, 1.0);
                    if y > 0 {
                        relax(&mut d, &mut src, idx, (y - 1) * gw + xl, D2);
                    }
                }
                if y > 0 {
                    relax(&mut d, &mut src, idx, (y - 1) * gw + x, 1.0);
                    if let Some(xr) = right(x) {
                        relax(&mut d, &mut src, idx, (y - 1) * gw + xr, D2);
                    }
                }
            }
        }
        for y in (0..gh).rev() {
            for x in (0..gw).rev() {
                let idx = y * gw + x;
                if let Some(xr) = right(x) {
                    relax(&mut d, &mut src, idx, y * gw + xr, 1.0);
                    if y + 1 < gh {
                        relax(&mut d, &mut src, idx, (y + 1) * gw + xr, D2);
                    }
                }
                if y + 1 < gh {
                    relax(&mut d, &mut src, idx, (y + 1) * gw + x, 1.0);
                    if let Some(xl) = left(x) {
                        relax(&mut d, &mut src, idx, (y + 1) * gw + xl, D2);
                    }
                }
            }
        }
    }
    (d, src)
}

// ===========================================================================
// The derivation (§2.4)
// ===========================================================================

/// Everything the geology stage reads. **All of it exists before erosion and
/// before any climate is computed** -- that is the fix for the legacy label's
/// circularity, and `cartalith-engine`'s tests assert it by generating the
/// same world with erosion and climate changed and comparing the columns.
pub struct GeologyInputs<'a> {
    pub gw: usize,
    pub gh: usize,
    pub world: bool,
    pub seed: i32,
    /// The pre-erosion normalised surface (after volcanism, craters and the
    /// clamp).
    pub field: &'a [f32],
    pub sea_level: f64,
    pub peak_m: f64,
    /// `WorldState::crust_field`: `< 0` is oceanic.
    pub crust: &'a [f32],
    pub boundary_mask: &'a [u8],
    pub boundary_type: &'a [u8],
    pub volcanic_field: &'a [f32],
    /// `None` when volcanism stamped nothing.
    pub volcano: Option<&'a crate::VolcanoTrace>,
    /// Latitude in degrees for each row (`cartalith_climate::lat_at`).
    pub row_lat_deg: &'a [f64],
    pub blur_r: f64,
}

/// §2.4's derivation: labelled boundary distance, basement, sedimentary
/// cover, volcanic overlay. Deterministic in its inputs.
pub fn build_geology(inp: &GeologyInputs) -> GeologyColumn {
    let (gw, gh) = (inp.gw, inp.gh);
    let n = gw * gh;
    let (dist, nearest) = labelled_boundary_distance(gw, gh, inp.world, inp.boundary_mask);
    let smooth = crate::gauss_blur(inp.field, W_LOW_BLUR * inp.blur_r, gw, gh, inp.world);
    let m = |metres: f64| m_to_norm(metres, inp.sea_level, inp.peak_m);
    let (low_th, cover_min, rift_cap) = (m(LOW_DEPTH_M), m(COVER_MIN_M), m(RIFT_CAP_M));
    let br = inp.blur_r;
    let ff = FACIES_CYCLES / gw.max(1) as f64;
    let facies_seed = inp.seed ^ FACIES_SALT;

    let mut col = GeologyColumn {
        rock_top: vec![0; n],
        rock_sub: vec![NO_LAYER; n],
        contact: vec![f32::NAN; n],
        regolith: vec![0.0; n],
        volcanic_setting: vec![setting_code::NONE; n],
    };

    for i in 0..n {
        let (x, y) = (i % gw, i / gw);
        let surface = inp.field[i] as f64;
        let oceanic = inp.crust[i] < 0.0;
        let d = dist[i] as f64;
        let near_type = match nearest[i] {
            u32::MAX => btype::NONE,
            k => inp.boundary_type[k as usize],
        };

        // Step 2: basement, and the rift two-layer column.
        let mut top;
        let mut sub = None::<(Rock, f64)>;
        if oceanic {
            top = Rock::OceanicBasalt;
        } else {
            top = Rock::Granite;
            match near_type {
                btype::COLLISION if d < W_CORE * br => top = Rock::Gneiss,
                btype::COLLISION if d < W_OROGEN * br => top = Rock::Schist,
                btype::TRANSFORM if d < W_SHEAR * br => top = Rock::Schist,
                btype::RIFT if d < W_RIFT * br => {
                    top = Rock::Sandstone;
                    sub = Some((Rock::Shale, surface - rift_cap));
                }
                // SUBDUCTION_OC within W_ARC is the batholith belt, and
                // everything else is the old interior: granite either way.
                _ => {}
            }
        }

        // Step 3: sedimentary cover in structural lows away from margins.
        if !oceanic && sub.is_none() && d > W_OROGEN * br {
            let depression = smooth[i] as f64 - surface;
            if depression > low_th {
                let facies = if inp.world {
                    pfbm(x as f64 * ff, y as f64 * ff, facies_seed, FACIES_CYCLES as i32)
                } else {
                    fbm(x as f64 * ff, y as f64 * ff, facies_seed)
                };
                let cover = COVER_PER_DEPTH * depression * facies;
                if cover >= cover_min {
                    let lat = inp.row_lat_deg.get(y).copied().unwrap_or(90.0);
                    top = if lat.abs() < CARBONATE_LAT_DEG { Rock::Limestone } else { Rock::Sandstone };
                    sub = Some((Rock::Shale, surface - CAP_FRACTION * cover));
                }
            }
        }

        // Step 4: the volcanic overlay, with the setting the stamper kept.
        let vf = inp.volcanic_field[i] as f64;
        if let Some(e) = inp.volcano.and_then(|t| t.winner_at(i)) {
            col.volcanic_setting[i] = setting_code_of(e.setting);
            if vf > V_TH {
                let cap = vf * e.h;
                let basaltic = match e.setting {
                    crate::VolcanicSetting::Hotspot | crate::VolcanicSetting::Rift => true,
                    crate::VolcanicSetting::Arc => false,
                    // No province was rolled: read the setting off the margin.
                    crate::VolcanicSetting::Unclassified => near_type == btype::RIFT,
                };
                let cap_rock = if basaltic {
                    if oceanic {
                        None // basalt on oceanic basalt: no new unit
                    } else {
                        Some(Rock::PlateauBasalt)
                    }
                } else {
                    let t = (x as f64 - e.cx).hypot(y as f64 - e.cy) / e.r;
                    Some(if t < ARC_CORE_T { Rock::Andesite } else { Rock::Tuff })
                };
                if let Some(r) = cap_rock {
                    sub = Some((top, surface - cap));
                    top = r;
                }
            }
        }

        col.rock_top[i] = top as u8;
        if let Some((s, c)) = sub {
            col.rock_sub[i] = s as u8;
            col.contact[i] = c as f32;
        }
    }
    col
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every table value, as a literal (`MISTAKES.md`: never assert a constant
    /// against itself). Order: s, κ, θc, γ, ρ.
    #[test]
    fn rock_table_values_are_the_scope_values() {
        let want: [(&str, f32, f32, f32, f32, f32); ROCK_COUNT] = [
            ("Granite / granitoid", 0.85, 0.30, 60.0, 0.5, 0.10),
            ("Gneiss", 0.80, 0.35, 55.0, 0.6, 0.15),
            ("Schist / slate", 0.60, 0.80, 40.0, 1.2, 0.50),
            ("Plateau (flood) basalt", 0.75, 0.40, 60.0, 1.3, 0.20),
            ("Oceanic basalt", 0.75, 0.40, 45.0, 1.0, 0.20),
            ("Andesite", 0.70, 0.50, 45.0, 1.0, 0.30),
            ("Tuff / volcaniclastic", 0.35, 2.0, 35.0, 1.5, 3.0),
            ("Limestone", 0.70, 0.60, 55.0, 1.0, 0.30),
            ("Sandstone", 0.60, 0.80, 50.0, 1.1, 0.60),
            ("Shale / mudstone", 0.30, 2.5, 30.0, 1.8, 1.5),
            ("Unconsolidated", 0.10, 4.0, 33.0, 3.0, 10.0),
        ];
        for (k, w) in want.iter().enumerate() {
            let p = &ROCK_PROPS[k];
            assert_eq!((p.name, p.s, p.kappa, p.theta_c_deg, p.gamma, p.rho), *w, "row {k}");
        }
    }

    #[test]
    fn rock_table_classes_flags_and_projection() {
        use Jointing as J;
        use Permeability as P;
        use SelbyClass as S;
        let want: [(S, bool, P, J, Formation, u8); ROCK_COUNT] = [
            (S::Strong, false, P::Low, J::Massive, Formation::Basement, 0),
            (S::Strong, false, P::Low, J::Foliated, Formation::Basement, 6),
            (S::Moderate, false, P::Low, J::Foliated, Formation::Basement, 6),
            (S::Strong, false, P::High, J::Columnar, Formation::Volcanic, 1),
            (S::Strong, false, P::High, J::Blocky, Formation::Basement, 1),
            (S::Moderate, false, P::Moderate, J::Blocky, Formation::Volcanic, 2),
            (S::Weak, false, P::Moderate, J::Bedded, Formation::Volcanic, 2),
            (S::Moderate, true, P::High, J::Bedded, Formation::Cover, 3),
            (S::Moderate, false, P::Moderate, J::Bedded, Formation::Cover, 4),
            (S::Weak, false, P::VeryLow, J::Fissile, Formation::Cover, 5),
            (S::VeryWeak, false, P::Variable, J::None, Formation::Regolith, 5),
        ];
        for (k, w) in want.iter().enumerate() {
            let p = &ROCK_PROPS[k];
            assert_eq!((p.selby, p.soluble, p.permeability, p.jointing, p.formation, p.legacy_class), *w, "row {k}");
        }
        // Only limestone is soluble.
        assert_eq!(ROCK_PROPS.iter().filter(|p| p.soluble).count(), 1);
    }

    /// Marinos & Hoek (2000) Table 2, as read from the paper for GF-1.
    #[test]
    fn hoek_brown_mi_is_marinos_and_hoek_table_2() {
        let want: [Option<(f32, f32, bool)>; ROCK_COUNT] = [
            Some((32.0, 3.0, false)), // granite
            Some((28.0, 5.0, false)), // gneiss
            Some((12.0, 3.0, false)), // schists
            Some((25.0, 5.0, true)),  // basalt (estimate)
            Some((25.0, 5.0, true)),  // basalt (estimate)
            Some((25.0, 5.0, false)), // andesite
            Some((13.0, 5.0, true)),  // tuff (estimate)
            Some((9.0, 2.0, true)),   // micritic limestone (estimate)
            Some((17.0, 4.0, false)), // sandstones
            Some((6.0, 2.0, true)),   // shales (estimate)
            None,                     // not intact rock
        ];
        for (k, w) in want.iter().enumerate() {
            assert_eq!(ROCK_PROPS[k].mi.map(|m| (m.mi, m.plus_minus, m.estimate)), *w, "row {k}");
        }
    }

    /// Each rock's `s` lies inside its Selby class's quoted band, and the
    /// bands are the quoted literals.
    #[test]
    fn strength_sits_inside_its_selby_band() {
        assert_eq!(SelbyClass::VeryWeak.band(), (0.0, 25.0));
        assert_eq!(SelbyClass::Weak.band(), (26.0, 50.0));
        assert_eq!(SelbyClass::Moderate.band(), (51.0, 70.0));
        assert_eq!(SelbyClass::Strong.band(), (71.0, 90.0));
        assert_eq!(SelbyClass::VeryStrong.band(), (91.0, 100.0));
        for p in &ROCK_PROPS {
            let (lo, hi) = p.selby.band();
            let s = p.s * 100.0;
            assert!(s >= lo && s <= hi, "{}: s {} outside {:?}", p.name, p.s, p.selby);
        }
    }

    #[test]
    fn rock_indices_round_trip_and_no_layer_is_not_a_rock() {
        for (k, r) in Rock::ALL.iter().enumerate() {
            assert_eq!(*r as usize, k);
            assert_eq!(Rock::from_u8(k as u8), Some(*r));
        }
        assert_eq!(NO_LAYER, 255);
        assert_eq!(Rock::from_u8(NO_LAYER), None);
        assert_eq!(Rock::from_u8(11), None);
        assert_eq!(ROCK_COUNT, 11);
        assert_eq!(R_EXPOSE_M, 5.0);
    }

    #[test]
    fn derivation_constants_are_pinned() {
        assert_eq!(
            [W_CORE, W_OROGEN, W_ARC, W_SHEAR, W_RIFT, W_LOW_BLUR],
            [1.0, 3.0, 4.0, 1.0, 2.5, 4.0]
        );
        assert_eq!(
            [LOW_DEPTH_M, COVER_PER_DEPTH, COVER_MIN_M, CAP_FRACTION, RIFT_CAP_M],
            [30.0, 3.0, 20.0, 0.5, 150.0]
        );
        assert_eq!([V_TH, ARC_CORE_T, CARBONATE_LAT_DEG, FACIES_CYCLES], [0.2, 0.4, 30.0, 3.0]);
    }

    fn column(top: &[u8], sub: &[u8], contact: &[f32], regolith: &[f32]) -> GeologyColumn {
        GeologyColumn {
            rock_top: top.to_vec(),
            rock_sub: sub.to_vec(),
            contact: contact.to_vec(),
            regolith: regolith.to_vec(),
            volcanic_setting: vec![0; top.len()],
        }
    }

    /// §2.5's exposure rule, each branch, with the boundary cases.
    #[test]
    fn exposure_rule_each_branch() {
        let c = column(&[3, 0], &[9, NO_LAYER], &[0.5, f32::NAN], &[0.0, 0.0]);
        // Cap above the contact, at it, and below it.
        assert_eq!(c.exposed(0, 0.6, 0.01), Some(Rock::PlateauBasalt));
        assert_eq!(c.exposed(0, 0.5, 0.01), Some(Rock::PlateauBasalt));
        assert_eq!(c.exposed(0, 0.49, 0.01), Some(Rock::Shale));
        // A single-layer column is its top at any height.
        assert_eq!(c.exposed(1, 0.0, 0.01), Some(Rock::Granite));
        assert_eq!(c.substrate(1), None);
        assert_eq!(c.substrate(0), Some((Rock::Shale, 0.5)));
        // Regolith: at the threshold still bedrock, above it unconsolidated.
        let r = column(&[0], &[NO_LAYER], &[f32::NAN], &[0.01]);
        assert_eq!(r.exposed(0, 0.6, 0.01), Some(Rock::Granite));
        let r = column(&[0], &[NO_LAYER], &[f32::NAN], &[0.011]);
        assert_eq!(r.exposed(0, 0.6, 0.01), Some(Rock::Unconsolidated));
    }

    #[test]
    fn beneath_reports_a_reason_where_there_is_no_unit() {
        let c = column(&[3, 0], &[9, NO_LAYER], &[0.5, f32::NAN], &[0.0, 0.0]);
        assert_eq!(c.beneath(0, 0.625, 0.01), Beneath::Unit { rock: Rock::Shale, depth: 0.125 });
        assert_eq!(c.beneath(0, 0.4, 0.01), Beneath::CapEroded);
        // At the contact exactly, the cap is still exposed (the exposure
        // rule's `>=`), so the substrate lies 0 below.
        assert_eq!(c.beneath(0, 0.5, 0.01), Beneath::Unit { rock: Rock::Shale, depth: 0.0 });
        assert_eq!(c.beneath(1, 0.4, 0.01), Beneath::SingleLayer);
        // A sub code with a NaN contact is not a layer (a reader must not
        // invent an elevation for it).
        let bad = column(&[3], &[9], &[f32::NAN], &[0.0]);
        assert_eq!(bad.substrate(0), None);
        assert_eq!(bad.beneath(0, 0.5, 0.01), Beneath::SingleLayer);
    }

    #[test]
    fn m_to_norm_uses_the_sea_anchored_scale() {
        // 5 m at sea 0.42, peak 4000 m: 5 * 0.58 / 4000.
        assert!((m_to_norm(5.0, 0.42, 4000.0) - 0.000725).abs() < 1e-12);
    }

    #[test]
    fn labelled_distance_carries_the_nearest_boundary_and_wraps() {
        let (gw, gh) = (10usize, 3usize);
        let mut mask = vec![0u8; gw * gh];
        mask[gw + 1] = 1; // (1, 1)
        let (d, s) = labelled_boundary_distance(gw, gh, false, &mask);
        assert_eq!(d[gw + 1], 0.0);
        assert_eq!(d[gw + 4], 3.0);
        assert!(s.iter().all(|&k| k == (gw + 1) as u32));
        // (9, 1) is 8 cells away without wrap, 2 with it.
        assert_eq!(d[gw + 9], 8.0);
        let (dw, _) = labelled_boundary_distance(gw, gh, true, &mask);
        assert_eq!(dw[gw + 9], 2.0);
        // A diagonal step costs sqrt 2.
        assert_eq!(d[2 * gw + 2], std::f32::consts::SQRT_2);
        // The seam in the other direction: a boundary at x 8 reaches x 0 in
        // two steps across it.
        let mut m8 = vec![0u8; gw * gh];
        m8[gw + 8] = 1;
        let (d8, s8) = labelled_boundary_distance(gw, gh, true, &m8);
        assert_eq!(d8[gw], 2.0);
        assert_eq!(s8[gw], (gw + 8) as u32);
        // No boundary: infinite distance and no source, never 0.
        let (d0, s0) = labelled_boundary_distance(gw, gh, false, &vec![0u8; gw * gh]);
        assert!(d0.iter().all(|v| v.is_infinite()));
        assert!(s0.iter().all(|&k| k == u32::MAX));
    }

    /// A fixture that reaches every branch of the derivation.
    #[allow(clippy::type_complexity)]
    fn derive(
        crust: f32,
        btype_at_boundary: u8,
        boundary_x: usize,
        field_at: fn(usize, usize) -> f32,
        volcano: Option<crate::VolcanoTrace>,
        vf: f32,
        lat: f64,
    ) -> GeologyColumn {
        let (gw, gh) = (64usize, 16usize);
        let n = gw * gh;
        let mut mask = vec![0u8; n];
        let mut bt = vec![0u8; n];
        for y in 0..gh {
            mask[y * gw + boundary_x] = 1;
            bt[y * gw + boundary_x] = btype_at_boundary;
        }
        let field: Vec<f32> = (0..n).map(|i| field_at(i % gw, i / gw)).collect();
        let lats = vec![lat; gh];
        build_geology(&GeologyInputs {
            gw,
            gh,
            world: false,
            seed: 7,
            field: &field,
            sea_level: 0.4,
            peak_m: 4000.0,
            crust: &vec![crust; n],
            boundary_mask: &mask,
            boundary_type: &bt,
            volcanic_field: &vec![vf; n],
            volcano: volcano.as_ref(),
            row_lat_deg: &lats,
            blur_r: 2.0,
        })
    }

    fn flat(_: usize, _: usize) -> f32 {
        0.6
    }

    #[test]
    fn basement_follows_the_nearest_boundary_type() {
        // blur_r 2: core < 2 cells, orogen < 6, shear < 2, rift < 5.
        let c = derive(0.8, btype::COLLISION, 0, flat, None, 0.0, 45.0);
        assert_eq!(c.top(1), Some(Rock::Gneiss)); // d 1
        assert_eq!(c.top(2), Some(Rock::Schist)); // d 2 (not < 2)
        assert_eq!(c.top(5), Some(Rock::Schist));
        assert_eq!(c.top(6), Some(Rock::Granite)); // d 6 (not < 6)
        let c = derive(0.8, btype::TRANSFORM, 0, flat, None, 0.0, 45.0);
        assert_eq!(c.top(1), Some(Rock::Schist));
        assert_eq!(c.top(2), Some(Rock::Granite));
        let c = derive(0.8, btype::SUBDUCTION_OC, 0, flat, None, 0.0, 45.0);
        assert_eq!(c.top(1), Some(Rock::Granite));
        let c = derive(0.8, btype::RIFT, 0, flat, None, 0.0, 45.0);
        assert_eq!(c.top(4), Some(Rock::Sandstone));
        let (sub, contact) = c.substrate(4).expect("rift fill is two-layer");
        assert_eq!(sub, Rock::Shale);
        // 150 m at sea 0.4, peak 4000: 150 * 0.6 / 4000 = 0.0225 below 0.6.
        assert!((contact - 0.5775).abs() < 1e-6, "{contact}");
        assert_eq!(c.top(5), Some(Rock::Granite)); // d 5 (not < 5)
        assert_eq!(c.substrate(5), None);
        // Oceanic crust is oceanic basalt whatever the margin.
        let c = derive(-0.8, btype::COLLISION, 0, flat, None, 0.0, 45.0);
        assert!(c.rock_top.iter().all(|&r| r == Rock::OceanicBasalt as u8));
        // A single-layer cell stores NaN, never a plausible contact.
        assert!(c.contact.iter().all(|v| v.is_nan()));
        assert!(c.rock_sub.iter().all(|&v| v == NO_LAYER));
        assert!(c.regolith.iter().all(|&v| v == 0.0));
    }

    /// A basin far from the margin: limestone at low latitude, sandstone at
    /// high, both over shale; the rim outside the low keeps its basement.
    #[test]
    fn structural_lows_take_a_cover_by_latitude() {
        // A 0.1-deep bowl centred at x 40 (600 m at this scale), well beyond
        // the orogen width from the boundary at x 0.
        fn bowl(x: usize, _: usize) -> f32 {
            let d = (x as f32 - 40.0).abs();
            if d < 6.0 { 0.5 } else { 0.6 }
        }
        let warm = derive(0.8, btype::COLLISION, 0, bowl, None, 0.0, 10.0);
        let cold = derive(0.8, btype::COLLISION, 0, bowl, None, 0.0, 60.0);
        let i = 8 * 64 + 40;
        assert_eq!(warm.top(i), Some(Rock::Limestone));
        assert_eq!(cold.top(i), Some(Rock::Sandstone));
        assert_eq!(warm.substrate(i).map(|s| s.0), Some(Rock::Shale));
        let (_, c) = warm.substrate(i).unwrap();
        assert!(c < 0.5, "the contact lies below the surface: {c}");
        // Outside the bowl: no cover.
        assert_eq!(warm.top(8 * 64 + 60), Some(Rock::Granite));
        assert_eq!(warm.substrate(8 * 64 + 60), None);
        // The same bowl inside the active-margin width (orogen, 6 cells here)
        // takes no cover: a structural low at a margin is not a basin.
        fn near_bowl(x: usize, _: usize) -> f32 {
            if x.abs_diff(4) < 6 { 0.5 } else { 0.6 }
        }
        let margin = derive(0.8, btype::SUBDUCTION_OC, 0, near_bowl, None, 0.0, 10.0);
        assert_eq!(margin.top(8 * 64 + 4), Some(Rock::Granite));
        assert_eq!(margin.substrate(8 * 64 + 4), None);
    }

    fn one_volcano(setting: crate::VolcanicSetting) -> crate::VolcanoTrace {
        let mut t = crate::VolcanoTrace::new(64 * 16);
        t.edifices.push(crate::PlacedEdifice { setting, cx: 40.0, cy: 8.0, r: 10.0, h: 0.2 });
        for w in t.winner.iter_mut() {
            *w = 0;
        }
        t
    }

    #[test]
    fn volcanic_overlay_keeps_the_setting() {
        use crate::VolcanicSetting as V;
        let i_core = 8 * 64 + 41; // t = 0.1
        let i_flank = 8 * 64 + 47; // t = 0.7
        let hot = derive(0.8, btype::COLLISION, 0, flat, Some(one_volcano(V::Hotspot)), 0.5, 45.0);
        assert_eq!(hot.top(i_core), Some(Rock::PlateauBasalt));
        // The previous top (granite interior) is the substrate; cap = vf*h.
        let (sub, c) = hot.substrate(i_core).unwrap();
        assert_eq!(sub, Rock::Granite);
        assert!((c - 0.5).abs() < 1e-6, "{c}");
        assert_eq!(hot.volcanic_setting[i_core], setting_code::HOTSPOT);
        let rift = derive(0.8, btype::COLLISION, 0, flat, Some(one_volcano(V::Rift)), 0.5, 45.0);
        assert_eq!(rift.top(i_core), Some(Rock::PlateauBasalt));
        assert_eq!(rift.volcanic_setting[i_core], setting_code::RIFT);
        let arc = derive(0.8, btype::COLLISION, 0, flat, Some(one_volcano(V::Arc)), 0.5, 45.0);
        assert_eq!(arc.top(i_core), Some(Rock::Andesite));
        assert_eq!(arc.top(i_flank), Some(Rock::Tuff));
        assert_eq!(arc.volcanic_setting[i_core], setting_code::ARC);
        // Below the intensity threshold: setting recorded, no cap.
        // (0.2f32 widens to 0.2000000030 > V_TH, so the just-below case is
        // 0.195, which widens to 0.19499999.)
        let weak = derive(0.8, btype::COLLISION, 0, flat, Some(one_volcano(V::Arc)), 0.195, 45.0);
        assert_eq!(weak.top(i_core), Some(Rock::Granite));
        assert_eq!(weak.volcanic_setting[i_core], setting_code::ARC);
        // A hotspot on oceanic crust adds no unit over oceanic basalt.
        let sea = derive(-0.8, btype::COLLISION, 0, flat, Some(one_volcano(V::Hotspot)), 0.5, 45.0);
        assert_eq!(sea.top(i_core), Some(Rock::OceanicBasalt));
        assert_eq!(sea.substrate(i_core), None);
        // Unclassified reads the margin: rift -> basalt, anything else -> arc.
        let u_rift = derive(0.8, btype::RIFT, 0, flat, Some(one_volcano(V::Unclassified)), 0.5, 45.0);
        assert_eq!(u_rift.top(i_core), Some(Rock::PlateauBasalt));
        assert_eq!(u_rift.volcanic_setting[i_core], setting_code::UNCLASSIFIED);
        let u_col = derive(0.8, btype::COLLISION, 0, flat, Some(one_volcano(V::Unclassified)), 0.5, 45.0);
        assert_eq!(u_col.top(i_core), Some(Rock::Andesite));
        // No trace: no setting anywhere.
        let none = derive(0.8, btype::COLLISION, 0, flat, None, 0.5, 45.0);
        assert!(none.volcanic_setting.iter().all(|&s| s == setting_code::NONE));
        assert_eq!(
            [setting_code::NONE, setting_code::ARC, setting_code::RIFT, setting_code::HOTSPOT, setting_code::UNCLASSIFIED],
            [0, 1, 2, 3, 4]
        );
    }
}
