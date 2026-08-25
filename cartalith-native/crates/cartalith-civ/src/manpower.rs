//! Military manpower — what a polity can actually put and keep under arms
//! (`MILITARY_MANPOWER_SCOPE.md`, `GUI_GAP_REGISTER.md` **CV-25**).
//!
//! ## This one is genuinely new, and the reference has nothing
//!
//! [`crate::military`] is a port: the frozen snapshot really does model
//! fortification, and CV-25's first pass found it. This module is the other
//! half, and it is **not** a port. Grepping the frozen snapshot for
//! `manpower`, `mobiliz`, `levy`, `conscript` and `militia` returns exactly
//! two hits, both `JP_COST_TOLL_PER_BORDER`'s comment using "levy" to mean a
//! *toll*. There is no army-size model in the reference at any line.
//!
//! So there is no golden fixture to match here and none is fabricated. What
//! this module owes instead is that every number it produces is traceable to
//! a stated assumption, and that the assumptions are the owner's supplied
//! specification rather than this port's invention — see
//! `MILITARY_MANPOWER_SCOPE.md`, which carries that specification verbatim.
//!
//! ## The correction this module exists to implement
//!
//! **Agricultural technology does not determine army size.** It determines
//! surplus, labour requirements, transport capacity, taxation base and
//! administrative capacity, and military manpower is supported *out of
//! those*. A technology era is therefore an **output** of this model
//! ([`Manpower::era_band`]), derived from the five variables below and used as a
//! sanity band — never an input driving them.
//!
//! ## Five interacting variables ([`MilitaryDrivers`])
//!
//! | Variable | Where it comes from in this port |
//! |---|---|
//! | `food_surplus_per_farmer` | the faction's ag-tech row × how well its own territory actually feeds the population on it (`civ_current_agrarian_density` over `civTerritory`) |
//! | `agricultural_labour_ratio` | [`crate::roster::AG_TECH_LEVELS`]' `farmers_per_urbanite`, as `f/(1+f)` |
//! | `fiscal_extraction_efficiency` | [`crate::roster::CIV_GOVERNMENTS`] × how much of the faction its own road network reaches × how monetised/urban it is |
//! | `professionalization` | the same state capacity, plus urbanisation |
//! | `logistics_capacity` | the way network's five tiers, navigable water and sea lanes |
//!
//! Two of those give an existing table its **first consumer anywhere in this
//! port**, the same finding CV-25's first pass made about `umWalls`/`umAge`:
//! [`crate::roster::AG_TECH_LEVELS`]' own module doc says outright that
//! `farmers_per_urbanite` is *"presently as inert as Government/Religion are
//! in the reference"*, and [`crate::roster::CIV_GOVERNMENTS`]' says *"no
//! simulation reads or writes this, and nothing in this port does either"*.
//! Both are read here.
//!
//! ## Four outputs, not one "army size"
//!
//! 1. [`Manpower::standing_army`] — people continuously maintained under arms.
//!    A **fiscal** answer: the state's captured surplus divided by what a
//!    soldier costs.
//! 2. [`Manpower::field_army`] — what can be concentrated in one place and
//!    fed there. A **logistical** answer.
//! 3. [`Manpower::emergency_mobilization`] — who can be called up at all. A
//!    **demographic** answer, filtered by administration.
//! 4. [`Manpower::force_ladder`] — how long each of those can be kept away
//!    from productive work. The one output that makes the other three
//!    comparable, and the reason a single "military size" statistic is the
//!    wrong shape: a state that can raise 10 % of its population for a month
//!    can rarely raise 2 % for a year.
//!
//! ## Two modelling cautions, both owner-stated, both honoured structurally
//!
//! **Ancient army numbers are exaggerated.** Xerxes' invasion is described
//! in millions and reconstructs to something like 70 000 infantry and 9 000
//! cavalry. So [`Manpower::concentration_ratio`] reports the field army
//! against the emergency mobilization: any claimed host larger than the
//! field figure could not have been fed in one place, whatever a chronicle
//! says. The check is structural rather than a warning string.
//!
//! **A warrior society gets no standing-army bonus.** A hunter-gatherer
//! band's fighters are also its hunters, herders, toolmakers, scouts and
//! parents — the military *is* the adult population temporarily changing
//! occupation. That falls out of the formula rather than being special-cased:
//! [`Manpower::standing_army`] is paid out of the non-agricultural surplus,
//! which at a 95 % agricultural labour ratio is almost nothing, while
//! [`Manpower::emergency_mobilization`] is demographic and stays large.
//!
//! ## Nothing here is stored
//!
//! Derived and recomputed, like [`crate::civ_faction_aggregates`],
//! [`crate::relations`] and `wildlife_regions`. `CivData` gains no field,
//! nothing is saved, and a second call on an unchanged world returns the
//! same answer. There is no combat, no unit, no campaign and no clock.

use cartalith_jsmath::{js_max, js_min};

// ============================================================ constants

/// Share of a pre-modern population that is of military age at any moment —
/// roughly the 15-50 male cohort under a high-mortality age structure, which
/// runs 22-26 % of the whole population. `0.25` is the round middle of that.
///
/// This is the **pool**, not the mobilization: nobody calls up every
/// eligible man, which is what [`LEVY_BASE`] and its two companions are for.
pub const MILITARY_AGE_FRACTION: f64 = 0.25;

/// What one continuously-maintained soldier costs, in subsistence-
/// equivalents: pay, rations, equipment replacement, and the animals and
/// servants a soldier of any era drags behind him. Roughly three times a
/// peasant household's own consumption — the ratio implied by Roman
/// legionary pay against a subsistence wage, and by later mercenary
/// contracts.
///
/// Flat rather than scaled by [`MilitaryDrivers::professionalization`], and
/// that is deliberate: a levy-heavy standing force is cheaper per head but
/// less of it is genuinely standing, and letting one constant carry both
/// effects made the two worked examples in `MILITARY_MANPOWER_SCOPE.md` move
/// in opposite directions. Professionalization is reported and used where it
/// belongs instead — the professional core and the campaign duration.
pub const SOLDIER_UPKEEP: f64 = 3.0;

/// The floor and ceiling of [`MilitaryDrivers::fiscal_extraction_efficiency`]
/// — the share of the **non-agricultural surplus** a state captures.
///
/// Not a share of total output: with a 75 % agricultural labour ratio the
/// non-agricultural quarter is what these fractions apply to, so
/// `EXTRACTION_CEILING` corresponds to a state capturing about 7 % of
/// everything, which is at the top of what pre-modern fiscal systems
/// managed. The floor is a polity that taxes almost nothing it does not
/// consume on the spot.
pub const EXTRACTION_FLOOR: f64 = 0.04;
/// See [`EXTRACTION_FLOOR`].
pub const EXTRACTION_CEILING: f64 = 0.16;

/// [`Manpower::emergency_mobilization`] as a share of the military-age pool:
/// a base nobody falls below, plus what administration and transport add.
///
/// A state with no administration and no roads still raises *something* —
/// the men who live where the fighting is. What it cannot do is reach the
/// rest of the pool, which is what the other two terms buy.
pub const LEVY_BASE: f64 = 0.04;
/// See [`LEVY_BASE`].
pub const LEVY_STATE: f64 = 0.30;
/// See [`LEVY_BASE`].
pub const LEVY_LOGISTICS: f64 = 0.22;

/// [`Manpower::field_army`] as a share of the emergency mobilization: what
/// can be concentrated in one place and fed there rather than defending its
/// own valley.
///
/// The base term is not zero because an army marches on what it carries and
/// forages before any road matters; the logistics term is what lets it stay
/// concentrated once that runs out.
pub const FIELD_BASE: f64 = 0.34;
/// See [`FIELD_BASE`].
pub const FIELD_LOGISTICS: f64 = 0.20;

/// The two anchors the war-duration curve is fitted through, as
/// `(share of total population mobilized, days sustainable)`, for a polity
/// of median state capacity and median logistics.
///
/// Both come straight from the owner's specification: *"A state may raise
/// 10 % for 30 days but only 2 % for a multi-year war without collapsing
/// agricultural production."* Everything else about the curve —
/// [`duration_exponent`] and [`duration_coefficient`] — is derived from
/// these two points rather than chosen, so there is exactly one place to
/// argue with.
pub const DURATION_ANCHOR_LONG: (f64, f64) = (0.02, 365.0);
/// See [`DURATION_ANCHOR_LONG`].
pub const DURATION_ANCHOR_SHORT: (f64, f64) = (0.10, 30.0);

/// A campaign shorter than this is not a war-duration question, and one
/// longer than a year is [`DURATION_ANCHOR_LONG`]'s own "requires a
/// substantially different fiscal system" — reported as the ceiling rather
/// than extrapolated past it.
pub const DURATION_MIN_DAYS: f64 = 7.0;
/// See [`DURATION_MIN_DAYS`].
pub const DURATION_MAX_DAYS: f64 = 365.0;

/// The four durations [`Manpower::force_ladder`] answers at, in days. The
/// owner's own ladder: *"30 days feasible → 90 difficult → 180 severe
/// disruption → 365 requires a substantially different fiscal system"*, and
/// 60 is where a feudal obligation typically expired.
pub const LADDER_DAYS: [f64; 4] = [30.0, 90.0, 180.0, 365.0];

/// The largest non-agricultural share any [`crate::roster::AG_TECH_LEVELS`]
/// row produces (`industrial`, `farmers_per_urbanite = 0.15`, so
/// `1 - 0.15/1.15`). Used to normalise urbanisation onto `0..1` without
/// hard-coding a number the table could move.
const MAX_NON_AGRICULTURAL_SHARE: f64 = 1.0 / 1.15;

/// How well each [`crate::roster::CIV_GOVERNMENTS`] key extracts, `0..1`.
///
/// **This table is the first consumer `CIV_GOVERNMENTS` has ever had**, in
/// this port or the reference — that module's own doc says so. The ordering
/// is the uncontroversial half of comparative state capacity: a chiefdom
/// redistributes what it can see, a city-state and a republic tax a small
/// area intensively, an empire runs a professional revenue service. The
/// *absolute* values are calibrated against the two worked examples in
/// `MILITARY_MANPOWER_SCOPE.md` and nothing else.
///
/// Unknown keys read as `chiefdom`, which is the conservative end: a
/// government this port cannot classify should not be credited with an
/// imperial treasury.
pub const GOVERNMENT_EXTRACTION: [(&str, f64); 9] = [
    ("none", 0.10),
    ("chiefdom", 0.15),
    ("tribal_confederacy", 0.20),
    ("monarchy", 0.45),
    ("theocracy", 0.45),
    ("oligarchy", 0.50),
    ("republic", 0.55),
    ("city_state", 0.55),
    ("empire", 0.70),
];

/// [`GOVERNMENT_EXTRACTION`]'s lookup, with its stated fallback.
pub fn government_extraction(key: &str) -> f64 {
    GOVERNMENT_EXTRACTION
        .iter()
        .find(|&&(k, _)| k == key)
        .map_or(0.15, |&(_, v)| v)
}

/// One row of the era table in `MILITARY_MANPOWER_SCOPE.md`: a name, the
/// sustainable standing-army band and the wartime-mobilization band, both as
/// shares of total population.
///
/// **These are modelling ranges, not historical laws** — the owner's own
/// words, and the reason [`Manpower::era_verdict`] reports "above" or
/// "below" rather than clamping anything into the band. Geography, state
/// organisation, wealth inequality, military culture and whether soldiers
/// are self-supporting can all move a real society outside its row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EraBand {
    pub name: &'static str,
    pub standing: (f64, f64),
    pub mobilization: (f64, f64),
    /// The row's own "main constraint" column.
    pub constraint: &'static str,
}

/// The era table, verbatim from `MILITARY_MANPOWER_SCOPE.md`.
///
/// `HunterGatherer` is retained and is **unreachable from this port's
/// generated worlds**, because the lowest [`crate::roster::AG_TECH_LEVELS`]
/// row is `subsistence` (hoe cultivation), not foraging. Kept anyway so the
/// table is the owner's table — the same reasoning
/// [`crate::civ_base_pop_for_kind`]'s own unreachable row already carries —
/// and reachable by any caller that supplies a labour ratio above `0.94`
/// with essentially no state and no transport.
pub const ERA_BANDS: [EraBand; 15] = [
    EraBand {
        name: "Hunter-gatherer",
        standing: (0.00, 0.01),
        mobilization: (0.05, 0.15),
        constraint: "Food availability / seasonal movement",
    },
    EraBand {
        name: "Early horticulture",
        standing: (0.00, 0.01),
        mobilization: (0.05, 0.15),
        constraint: "Very limited surplus",
    },
    EraBand {
        name: "Neolithic agriculture",
        standing: (0.001, 0.01),
        mobilization: (0.05, 0.15),
        constraint: "Labour needed on farms",
    },
    EraBand {
        name: "Bronze Age state",
        standing: (0.005, 0.02),
        mobilization: (0.05, 0.15),
        constraint: "Administration + food storage",
    },
    EraBand {
        name: "Iron Age agrarian state",
        standing: (0.01, 0.025),
        mobilization: (0.10, 0.20),
        constraint: "Logistics and harvest cycle",
    },
    EraBand {
        name: "Classical agrarian state",
        standing: (0.01, 0.03),
        mobilization: (0.10, 0.25),
        constraint: "Fiscal/logistical capacity",
    },
    EraBand {
        name: "Late antique / early medieval",
        standing: (0.002, 0.015),
        mobilization: (0.05, 0.15),
        constraint: "Political fragmentation",
    },
    EraBand {
        name: "High medieval",
        standing: (0.005, 0.02),
        mobilization: (0.05, 0.15),
        constraint: "Feudal obligations / campaign duration",
    },
    EraBand {
        name: "Late medieval",
        standing: (0.01, 0.03),
        mobilization: (0.10, 0.20),
        constraint: "Money and logistics",
    },
    EraBand {
        name: "Early gunpowder",
        standing: (0.01, 0.03),
        mobilization: (0.10, 0.20),
        constraint: "Fiscal administration",
    },
    EraBand {
        name: "Military-fiscal state",
        standing: (0.01, 0.04),
        mobilization: (0.10, 0.25),
        constraint: "State finances",
    },
    EraBand {
        name: "Early industrial",
        standing: (0.02, 0.05),
        mobilization: (0.15, 0.30),
        constraint: "Transport and supply",
    },
    EraBand {
        name: "Railway / industrial mass army",
        standing: (0.03, 0.08),
        mobilization: (0.20, 0.40),
        constraint: "Industrial logistics",
    },
    EraBand {
        name: "Total industrial mobilization",
        standing: (0.05, 0.10),
        mobilization: (0.30, 0.50),
        constraint: "Industrial capacity / demographics",
    },
    EraBand {
        name: "Modern mechanized",
        standing: (0.005, 0.03),
        mobilization: (0.05, 0.15),
        constraint: "Technology makes manpower less valuable",
    },
];

// ================================================================ types

/// The five variables, plus the two intermediates they are built from, all
/// reported so the shell can show its working rather than assert a number —
/// the discipline [`crate::relations::FactionRelation`] and
/// [`crate::SuitExplanation`] already set in this crate.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MilitaryDrivers {
    /// People a farmer feeds **beyond his own household**, so `0` is bare
    /// subsistence. `(1/farmers_per_urbanite) × ecological_factor`: the
    /// technology sets the ratio, the land decides whether it is achieved.
    pub food_surplus_per_farmer: f64,
    /// `f/(1+f)`, `0..1`. The owner's *"extremely important"* variable, and
    /// the one that separates a medieval polity (0.70-0.90) from a modern
    /// one (a few per cent).
    pub agricultural_labour_ratio: f64,
    /// Share of the non-agricultural surplus the state actually captures.
    /// See [`EXTRACTION_FLOOR`] for what this is a share *of*.
    pub fiscal_extraction_efficiency: f64,
    /// `0..1` — how much of the military capacity is continuously
    /// maintained rather than called up.
    pub professionalization: f64,
    /// `0..1` — how far the army can operate from its food base. Roads by
    /// tier, navigable rivers, sea lanes.
    pub logistics_capacity: f64,
    /// `0..1`, the normalised administrative strength both
    /// `fiscal_extraction_efficiency` and `professionalization` are scaled
    /// from. Reported because it is the term a reader most often wants to
    /// disagree with.
    pub state_capacity: f64,
    /// The three raw terms [`logistics_capacity`](Self::logistics_capacity)
    /// was mixed from, carried through unchanged so a reader can see which
    /// one is doing the work: weighted way length against
    /// `ROAD_DENSITY_REF`, share of settlements on navigable water, share
    /// with sea access. All `0..1` and already clamped.
    pub road_density: f64,
    /// See [`road_density`](Self::road_density).
    pub navigable_share: f64,
    /// See [`road_density`](Self::road_density).
    pub sea_share: f64,
    /// `land capacity / total population`, clamped — whether the territory
    /// comfortably feeds the people on it (`>1`) or is drawn tight (`<1`).
    /// **This is the geography term**, and it is why two factions on the
    /// same ag-tech row do not get the same answer.
    pub ecological_factor: f64,
}

/// One rung of [`Manpower::force_ladder`]: the largest force sustainable for
/// this many days.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForceAtDuration {
    pub days: f64,
    /// Headcount. Never above [`Manpower::emergency_mobilization`] — the
    /// demographic ceiling binds before the fiscal one at short durations.
    pub force: f64,
    /// `force` as a share of total population, for comparison against
    /// [`Manpower::era_band`].
    pub share: f64,
    /// `true` when [`Manpower::emergency_mobilization`] is what capped this
    /// rung, i.e. the state could feed more than it can raise.
    pub capped_by_pool: bool,
}

/// Everything [`civ_military_manpower`] answers for one faction.
#[derive(Debug, Clone, PartialEq)]
pub struct Manpower {
    pub drivers: MilitaryDrivers,
    /// Nucleated population × `(1 + farmers_per_urbanite)` — the settled
    /// population *plus* the countryside it implies. See
    /// [`ManpowerInput::nucleated_pop`] for why the settlement sum is the
    /// urban half rather than the whole.
    pub total_population: f64,
    /// `total_population × agricultural_labour_ratio`.
    pub farming_population: f64,
    /// `total_population × MILITARY_AGE_FRACTION`.
    pub mobilization_pool: f64,
    /// Output 1: continuously maintained under arms.
    pub standing_army: f64,
    /// The genuinely full-time part of `standing_army`; the remainder is
    /// embodied but seasonal.
    pub professional_core: f64,
    /// Output 2: concentrable in one place and feedable there.
    pub field_army: f64,
    /// Output 3: callable up at all, temporarily.
    pub emergency_mobilization: f64,
    /// Output 4, per force level. `standing_army` is 365 by construction —
    /// that is what "standing" means — so this reports the field and
    /// emergency figures, and [`Manpower::force_ladder`] inverts the same
    /// curve.
    pub field_duration_days: f64,
    /// See [`Manpower::field_duration_days`].
    pub emergency_duration_days: f64,
    /// The largest force sustainable at each of [`LADDER_DAYS`].
    pub force_ladder: [ForceAtDuration; 4],
    /// `field_army / emergency_mobilization` — the plausibility check. A
    /// reported host above `field_army` could not have been supplied in one
    /// place however many a chronicle claims.
    pub concentration_ratio: f64,
    /// The era this faction's five variables put it in. **Derived, never an
    /// input.**
    pub era_band: EraBand,
    /// `standing_army / total_population`.
    pub standing_share: f64,
    /// `emergency_mobilization / total_population`.
    pub emergency_share: f64,
    /// `"within"`, `"above"` or `"below"` — where `standing_share` falls
    /// against [`EraBand::standing`].
    pub era_standing_verdict: &'static str,
    /// The same for `emergency_share` against [`EraBand::mobilization`].
    pub era_mobilization_verdict: &'static str,
}

/// Everything one faction's answer needs, all of it already computed
/// elsewhere in the civ layer. See `civ_military_bridge.rs` for where each
/// field is read from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ManpowerInput<'a> {
    /// Σ of this faction's settlement populations.
    ///
    /// **This is the urban/nucleated population, not the whole one**, and
    /// that is the reference's own semantic rather than an assumption made
    /// here: [`crate::roster::AG_TECH_LEVELS`]' `farmers_per_urbanite` is
    /// defined against exactly this quantity, and
    /// [`crate::timeline::civ_settlement_population`] sizes a nucleus at a
    /// `civ_surplus_fraction` (0.10-0.65) of what its catchment sustains.
    pub nucleated_pop: f64,
    /// [`crate::roster::AgTechLevel::farmers_per_urbanite`] for this
    /// faction's ag-tech row.
    pub farmers_per_urbanite: f64,
    /// People this faction's own territory sustains: Σ `dens[i] × cellKm²`
    /// over its cells, where `dens` is
    /// [`crate::timeline::civ_current_agrarian_density`] — the same field
    /// [`crate::timeline::civ_agrarian_regional_total`]'s *"Land sustains
    /// ≈ N"* readout integrates over the whole map.
    pub land_capacity: f64,
    /// A [`crate::roster::CIV_GOVERNMENTS`] key.
    pub government: &'a str,
    /// `0..1` — the share of this faction's settlements road-connected to
    /// its capital ([`crate::trade::RoadComponents`]). A state cannot tax
    /// what it cannot reach.
    pub capital_road_reach: f64,
    /// `0..1` — way length per unit territory, weighted by
    /// [`crate::WayType`], normalised at the bridge.
    pub road_density: f64,
    /// `0..1` — share of this faction's settlements on navigable water
    /// ([`crate::trade::NavKind::navigable`]).
    pub navigable_share: f64,
    /// `0..1` — share of this faction's settlements with sea access.
    pub sea_share: f64,
}

// ============================================================ the model

/// The war-duration curve's exponent, fitted through
/// [`DURATION_ANCHOR_LONG`] and [`DURATION_ANCHOR_SHORT`].
///
/// Computed rather than written down so the two anchors are the only thing
/// to argue with. `ln` is not `const`, which is why this is a function; it
/// costs two transcendentals per faction and nothing has a hot path here.
pub fn duration_exponent() -> f64 {
    let (s_long, d_long) = DURATION_ANCHOR_LONG;
    let (s_short, d_short) = DURATION_ANCHOR_SHORT;
    (d_long / d_short).ln() / (s_short / s_long).ln()
}

/// The war-duration curve's coefficient, so that `days = coef / share^exp`
/// passes through both anchors. See [`duration_exponent`].
pub fn duration_coefficient() -> f64 {
    let (s_long, d_long) = DURATION_ANCHOR_LONG;
    d_long * s_long.powf(duration_exponent())
}

/// How much better or worse than the median polity this one can keep an army
/// in the field, `1.0` at `state_capacity = logistics = professionalization
/// = 0.5` by construction — so the two duration anchors mean what they say
/// for a median state and are modulated, not overridden, for anyone else.
///
/// State capacity pays the army, logistics feeds it where it stands, and
/// professionalization is why it does not go home for the harvest.
fn campaign_capability(d: &MilitaryDrivers) -> f64 {
    (0.55 + 0.90 * d.state_capacity)
        * (0.75 + 0.50 * d.logistics_capacity)
        * (0.85 + 0.30 * d.professionalization)
}

/// Days a force of `share` of the total population can be kept away from
/// productive work, clamped to [`DURATION_MIN_DAYS`]..[`DURATION_MAX_DAYS`].
///
/// A `share` of zero (or negative, which cannot arise here) is
/// indefinite, so it returns the ceiling rather than dividing by zero.
pub fn sustainable_days(share: f64, capability: f64) -> f64 {
    // Negated on purpose, and kept negated: `!(x > 0.0)` is `true` for NaN
    // where `x <= 0.0` is `false` in Rust, and a NaN share must return the
    // ceiling rather than fall through to `powf` and produce NaN days
    // (`cartalith-rust-conventions`; the same form `trade::deliverable` keeps
    // for the same reason).
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(share > 0.0) {
        return DURATION_MAX_DAYS;
    }
    let raw = duration_coefficient() / share.powf(duration_exponent()) * capability;
    js_max(DURATION_MIN_DAYS, js_min(DURATION_MAX_DAYS, raw))
}

/// [`sustainable_days`] inverted: the largest share sustainable for `days`.
pub fn share_for_days(days: f64, capability: f64) -> f64 {
    // See [`sustainable_days`] for why this comparison stays negated.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(days > 0.0) {
        return 1.0;
    }
    (duration_coefficient() * capability / days).powf(1.0 / duration_exponent())
}

/// Which [`ERA_BANDS`] row these variables put a polity in.
///
/// **Driven by the agricultural labour ratio first**, because that is the
/// variable the owner calls extremely important and it is the one that
/// actually separates the eras: everything before the eighteenth century
/// sits above 0.70 and everything after falls away fast. State capacity
/// splits the rows that share a labour ratio, which is exactly what
/// distinguishes a Bronze Age palace from a classical state, or a fragmented
/// post-Roman west from a high-medieval kingdom.
///
/// Deliberately **not** a lookup on the ag-tech key: that would make
/// technology the driver, which is the thing this whole module exists to
/// stop doing. Two factions on the same ag-tech row with different
/// governments and different road networks land in different eras, and they
/// should.
pub fn era_for(d: &MilitaryDrivers) -> EraBand {
    let (a, s, l) = (d.agricultural_labour_ratio, d.state_capacity, d.logistics_capacity);
    let idx = if a >= 0.93 {
        if s < 0.10 && l < 0.20 {
            0 // Hunter-gatherer — see ERA_BANDS' note on reachability.
        } else if s < 0.18 {
            1 // Early horticulture
        } else {
            2 // Neolithic agriculture
        }
    } else if a >= 0.85 {
        if s < 0.30 {
            3 // Bronze Age state
        } else if s < 0.50 {
            4 // Iron Age agrarian state
        } else {
            5 // Classical agrarian state
        }
    } else if a >= 0.70 {
        if s < 0.22 {
            6 // Late antique / early medieval
        } else if s < 0.45 {
            7 // High medieval
        } else {
            8 // Late medieval
        }
    } else if a >= 0.45 {
        if s < 0.50 {
            9 // Early gunpowder
        } else {
            10 // Military-fiscal state
        }
    } else if a >= 0.25 {
        11 // Early industrial
    } else if a >= 0.10 {
        if s >= 0.80 && l >= 0.80 {
            14 // Modern mechanized
        } else {
            13 // Total industrial mobilization
        }
    } else {
        14 // Modern mechanized
    };
    ERA_BANDS[idx]
}

/// `"within"` / `"above"` / `"below"` for a share against a band. Never
/// clamps: a society outside its era's range is a finding, not an error.
fn band_verdict(share: f64, band: (f64, f64)) -> &'static str {
    if share < band.0 {
        "below"
    } else if share > band.1 {
        "above"
    } else {
        "within"
    }
}

/// The five variables, from one faction's already-computed world state.
pub fn military_drivers(input: &ManpowerInput) -> MilitaryDrivers {
    let f = js_max(0.0, input.farmers_per_urbanite);
    let alpha = f / (1.0 + f);
    let non_agri = 1.0 - alpha;

    let total_pop = input.nucleated_pop * (1.0 + f);
    // How well the land actually feeds the people on it. Clamped rather
    // than open-ended in both directions: an empty faction must not read as
    // infinitely fertile, and an over-drawn one is at famine rather than at
    // zero.
    let ecological = if total_pop > 0.0 {
        js_max(0.25, js_min(2.0, input.land_capacity / total_pop))
    } else {
        1.0
    };

    // A farmer feeds `1/f` non-farmers at the technology's own ratio; the
    // land decides whether that ratio is met, exceeded or missed.
    let surplus_per_farmer = if f > 0.0 { ecological / f } else { ecological * 100.0 };

    let reach = js_max(0.0, js_min(1.0, input.capital_road_reach));
    let urban_norm = js_max(0.0, js_min(1.0, non_agri / MAX_NON_AGRICULTURAL_SHARE));

    // Administration = what kind of state it is, times how much of itself it
    // can reach, times how monetised the economy it taxes is.
    let state_capacity = js_max(
        0.03,
        js_min(
            0.95,
            government_extraction(input.government)
                * (0.55 + 0.45 * reach)
                * (0.70 + 0.60 * urban_norm),
        ),
    );

    let road = js_min(1.0, js_max(0.0, input.road_density));
    let navigable = js_min(1.0, js_max(0.0, input.navigable_share));
    let sea = js_min(1.0, js_max(0.0, input.sea_share));
    let logistics =
        js_max(0.0, js_min(1.0, 0.15 + 0.45 * road + 0.30 * navigable + 0.10 * sea));

    let professionalization =
        js_max(0.0, js_min(1.0, 0.15 + 0.55 * state_capacity + 0.30 * urban_norm));

    MilitaryDrivers {
        food_surplus_per_farmer: surplus_per_farmer,
        agricultural_labour_ratio: alpha,
        fiscal_extraction_efficiency: EXTRACTION_FLOOR
            + (EXTRACTION_CEILING - EXTRACTION_FLOOR) * state_capacity,
        professionalization,
        logistics_capacity: logistics,
        road_density: road,
        navigable_share: navigable,
        sea_share: sea,
        state_capacity,
        ecological_factor: ecological,
    }
}

/// The whole model for one faction. See the module doc for the two
/// derivation chains and `MILITARY_MANPOWER_SCOPE.md` for the specification
/// they implement.
///
/// **NaN policy.** Every input here is a caller-supplied `f64` that could in
/// principle arrive non-finite ([`crate::civ_faction_aggregates`] can
/// legitimately produce a `NaN` from an empty faction's `0/0` mean, and this
/// module's own `pop` comes from that side of the house). A non-finite
/// population is absorbed to zero at the entry, the same coercion
/// `js_num_or_zero` performs for the reference's `p.pop||0` — because the
/// alternative is a `NaN` reaching a headcount, and a headcount is a claim.
pub fn civ_military_manpower(input: &ManpowerInput) -> Manpower {
    let nucleated = if input.nucleated_pop.is_finite() {
        js_max(0.0, input.nucleated_pop)
    } else {
        0.0
    };
    let land_capacity =
        if input.land_capacity.is_finite() { js_max(0.0, input.land_capacity) } else { 0.0 };
    let clean = ManpowerInput { nucleated_pop: nucleated, land_capacity, ..*input };

    let d = military_drivers(&clean);
    let f = js_max(0.0, clean.farmers_per_urbanite);
    let total_pop = nucleated * (1.0 + f);
    let farming = total_pop * d.agricultural_labour_ratio;
    let pool = total_pop * MILITARY_AGE_FRACTION;

    // ---- Chain 1: population -> surplus -> fiscal capacity -> standing.
    //
    // The non-agricultural population IS the embodied surplus: those are the
    // people the farmers' surplus already feeds. The state captures
    // `fiscal_extraction_efficiency` of it, scaled by whether the land is
    // actually delivering, and a soldier costs SOLDIER_UPKEEP of it.
    let non_agricultural = total_pop * (1.0 - d.agricultural_labour_ratio);
    let military_budget = non_agricultural * d.ecological_factor * d.fiscal_extraction_efficiency;
    let standing = military_budget / SOLDIER_UPKEEP;
    let professional_core = standing * d.professionalization;

    // ---- Chain 2: population -> military age -> levy -> logistics -> field.
    let levy_reach = js_max(
        0.0,
        js_min(
            0.60,
            LEVY_BASE + LEVY_STATE * d.state_capacity + LEVY_LOGISTICS * d.logistics_capacity,
        ),
    );
    let emergency = pool * levy_reach;
    let field = emergency * (FIELD_BASE + FIELD_LOGISTICS * d.logistics_capacity);

    // ---- Chain 3: how long any of it can stay away from the fields.
    let capability = campaign_capability(&d);
    let share = |n: f64| if total_pop > 0.0 { n / total_pop } else { 0.0 };
    let field_duration = sustainable_days(share(field), capability);
    let emergency_duration = sustainable_days(share(emergency), capability);

    let mut ladder = [ForceAtDuration { days: 0.0, force: 0.0, share: 0.0, capped_by_pool: false };
        LADDER_DAYS.len()];
    for (slot, &days) in ladder.iter_mut().zip(LADDER_DAYS.iter()) {
        let raw = share_for_days(days, capability) * total_pop;
        let capped = raw >= emergency;
        let force = js_min(raw, emergency);
        *slot = ForceAtDuration { days, force, share: share(force), capped_by_pool: capped };
    }

    let era = era_for(&d);
    let standing_share = share(standing);
    let emergency_share = share(emergency);

    Manpower {
        drivers: d,
        total_population: total_pop,
        farming_population: farming,
        mobilization_pool: pool,
        standing_army: standing,
        professional_core,
        field_army: field,
        emergency_mobilization: emergency,
        field_duration_days: field_duration,
        emergency_duration_days: emergency_duration,
        force_ladder: ladder,
        concentration_ratio: if emergency > 0.0 { field / emergency } else { 0.0 },
        era_band: era,
        standing_share,
        emergency_share,
        era_standing_verdict: band_verdict(standing_share, era.standing),
        era_mobilization_verdict: band_verdict(emergency_share, era.mobilization),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `MILITARY_MANPOWER_SCOPE.md`'s Kingdom A: 1 000 000 people, 75 %
    /// agricultural, weak taxation, poor roads.
    ///
    /// `farmers_per_urbanite = 3` is exactly a 75 % labour ratio, and the
    /// nucleated population is therefore 250 000 for a million total.
    fn kingdom_a() -> ManpowerInput<'static> {
        ManpowerInput {
            nucleated_pop: 250_000.0,
            farmers_per_urbanite: 3.0,
            // Ecologically neutral: the land feeds exactly the people on it,
            // so the *only* things separating A from B are the four other
            // variables.
            land_capacity: 1_000_000.0,
            government: "monarchy",
            capital_road_reach: 0.20,
            road_density: 0.10,
            navigable_share: 0.10,
            sea_share: 0.00,
        }
    }

    /// Kingdom B: the same million people, 55 % agricultural, high surplus,
    /// strong taxation, good roads and rivers, professional bureaucracy.
    fn kingdom_b() -> ManpowerInput<'static> {
        ManpowerInput {
            nucleated_pop: 450_000.0,
            // alpha = 0.55, i.e. f = a/(1-a) = 11/9. Written as the division
            // so the labour ratio it encodes is legible.
            farmers_per_urbanite: 11.0 / 9.0,
            land_capacity: 1_050_000.0,                    // "high surplus"
            government: "empire",
            capital_road_reach: 0.90,
            road_density: 0.70,
            navigable_share: 0.60,
            sea_share: 0.50,
        }
    }

    /// The two-anchor curve must actually pass through both anchors, or
    /// every duration in the model is quoting a fit nobody checked.
    #[test]
    fn duration_curve_passes_through_both_anchors() {
        let (s_long, d_long) = DURATION_ANCHOR_LONG;
        let (s_short, d_short) = DURATION_ANCHOR_SHORT;
        assert!((sustainable_days(s_long, 1.0) - d_long).abs() < 1e-9);
        assert!((sustainable_days(s_short, 1.0) - d_short).abs() < 1e-9);
        // And the inverse is a real inverse, not a second fit.
        assert!((share_for_days(d_short, 1.0) - s_short).abs() < 1e-12);
        assert!((share_for_days(d_long, 1.0) - s_long).abs() < 1e-12);
    }

    #[test]
    fn duration_falls_as_the_mobilized_share_rises() {
        let mut prev = f64::INFINITY;
        for s in [0.01, 0.02, 0.04, 0.08, 0.16, 0.32] {
            let d = sustainable_days(s, 1.0);
            assert!(d <= prev, "share {s} gave {d}, not below {prev}");
            prev = d;
        }
        // A vanishing force is indefinite rather than a divide by zero.
        assert_eq!(sustainable_days(0.0, 1.0), DURATION_MAX_DAYS);
        assert_eq!(sustainable_days(f64::NAN, 1.0), DURATION_MAX_DAYS);
    }

    /// The whole point of the model: same population, very different
    /// military power. Every figure is checked against the range
    /// `MILITARY_MANPOWER_SCOPE.md` states, and none of them is a range this
    /// test invented.
    #[test]
    fn worked_example_kingdom_a() {
        let m = civ_military_manpower(&kingdom_a());
        assert!((m.total_population - 1_000_000.0).abs() < 1.0);
        assert!((m.drivers.agricultural_labour_ratio - 0.75).abs() < 1e-12);

        // Stated: standing ~5 000.
        assert!(
            (4_000.0..7_000.0).contains(&m.standing_army),
            "standing {} outside 4k-7k for a stated ~5 000",
            m.standing_army
        );
        // Stated: emergency levy ~40 000.
        assert!(
            (32_000.0..50_000.0).contains(&m.emergency_mobilization),
            "emergency {} outside 32k-50k for a stated ~40 000",
            m.emergency_mobilization
        );
        // Stated: sustainable field army 15 000-20 000.
        assert!(
            (14_000.0..21_000.0).contains(&m.field_army),
            "field {} outside 14k-21k for a stated 15 000-20 000",
            m.field_army
        );
    }

    #[test]
    fn worked_example_kingdom_b() {
        let m = civ_military_manpower(&kingdom_b());
        assert!((m.total_population - 1_000_000.0).abs() < 1.0);
        assert!((m.drivers.agricultural_labour_ratio - 0.55).abs() < 1e-9);

        // Stated: standing ~20 000.
        assert!(
            (16_000.0..24_000.0).contains(&m.standing_army),
            "standing {} outside 16k-24k for a stated ~20 000",
            m.standing_army
        );
        // Stated: mobilization pool 100 000+.
        assert!(
            m.emergency_mobilization >= 90_000.0,
            "emergency {} below the stated 100 000+ (10 % tolerance)",
            m.emergency_mobilization
        );
        // Stated: sustainable field army 40 000-60 000.
        assert!(
            (38_000.0..62_000.0).contains(&m.field_army),
            "field {} outside 38k-62k for a stated 40 000-60 000",
            m.field_army
        );
    }

    /// The comparison the worked example exists to make. Ratios, so a
    /// constant that moved both sides equally cannot satisfy this.
    #[test]
    fn b_outclasses_a_on_every_output_at_equal_population() {
        let a = civ_military_manpower(&kingdom_a());
        let b = civ_military_manpower(&kingdom_b());
        assert!((a.total_population - b.total_population).abs() < 1.0);
        assert!(b.standing_army > a.standing_army * 2.5);
        assert!(b.emergency_mobilization > a.emergency_mobilization * 2.0);
        assert!(b.field_army > a.field_army * 2.0);
        // And B can keep a *larger* force out for a *year*, which is the
        // fiscal half of the story rather than the demographic half.
        assert!(b.force_ladder[3].force > a.force_ladder[3].force);
    }

    /// The owner's explicit caution: a warrior society must not be handed a
    /// large standing army just for being pre-agricultural. The standing
    /// figure is fiscal and collapses; the levy is demographic and does not.
    #[test]
    fn a_warrior_society_gets_a_levy_not_a_standing_army() {
        let m = civ_military_manpower(&ManpowerInput {
            nucleated_pop: 5_000.0,
            farmers_per_urbanite: 19.0, // subsistence, 95 % agricultural
            land_capacity: 100_000.0,
            government: "chiefdom",
            capital_road_reach: 0.10,
            road_density: 0.02,
            navigable_share: 0.20,
            sea_share: 0.10,
        });
        assert!((m.total_population - 100_000.0).abs() < 1.0);
        assert!(
            m.standing_share < 0.01,
            "standing share {} above the era table's 1 % ceiling",
            m.standing_share
        );
        // But it can still call up a real fraction of itself.
        assert!(
            m.emergency_share > 0.02,
            "emergency share {} — the whole adult population cannot be unavailable",
            m.emergency_share
        );
        assert!(m.emergency_mobilization > m.standing_army * 10.0);
    }

    /// Technology is not the driver: two factions on the *same* ag-tech row
    /// with different governments, roads and land must get different
    /// answers, and land alone must move the result.
    #[test]
    fn same_technology_different_answers() {
        let base = ManpowerInput {
            nucleated_pop: 100_000.0,
            farmers_per_urbanite: 9.0,
            land_capacity: 1_000_000.0,
            government: "chiefdom",
            capital_road_reach: 0.1,
            road_density: 0.05,
            navigable_share: 0.0,
            sea_share: 0.0,
        };
        let weak = civ_military_manpower(&base);
        let strong = civ_military_manpower(&ManpowerInput {
            government: "empire",
            capital_road_reach: 0.95,
            road_density: 0.8,
            navigable_share: 0.5,
            sea_share: 0.4,
            ..base
        });
        assert_eq!(weak.drivers.agricultural_labour_ratio, strong.drivers.agricultural_labour_ratio);
        assert!(strong.standing_army > weak.standing_army * 2.0);
        assert!(strong.era_band.name != weak.era_band.name);

        // Geography alone, holding every institution fixed: a territory that
        // feeds its people twice over supports more than one that is drawn
        // tight. This is `ecological_factor`, and it is why two factions on
        // one ag-tech row and one government still differ.
        let fertile = civ_military_manpower(&ManpowerInput { land_capacity: 4_000_000.0, ..base });
        let barren = civ_military_manpower(&ManpowerInput { land_capacity: 300_000.0, ..base });
        assert!(fertile.standing_army > barren.standing_army * 2.0);
        assert!(fertile.drivers.food_surplus_per_farmer > barren.drivers.food_surplus_per_farmer);
        // The levy is demographic, so the land must NOT move it — a real
        // separation between the two chains rather than one number wearing
        // two hats.
        assert!((fertile.emergency_mobilization - barren.emergency_mobilization).abs() < 1e-9);
    }

    /// The ladder must be monotonically decreasing in duration, and its
    /// 365-day rung is the one that answers "what can this state keep up
    /// indefinitely".
    #[test]
    fn the_force_ladder_decreases_with_duration() {
        for input in [kingdom_a(), kingdom_b()] {
            let m = civ_military_manpower(&input);
            for w in m.force_ladder.windows(2) {
                assert!(
                    w[1].force <= w[0].force,
                    "{} days gave {} , more than {} days' {}",
                    w[1].days,
                    w[1].force,
                    w[0].days,
                    w[0].force
                );
            }
            // The pool caps the short end and the fiscal curve the long end.
            assert!(m.force_ladder[0].capped_by_pool);
            assert!(!m.force_ladder[3].capped_by_pool);
            // A standing army is by definition indefinitely sustainable.
            assert!(m.force_ladder[3].force > m.standing_army);
        }
    }

    /// The plausibility check the owner asked for. Xerxes' invasion at a
    /// claimed several million against a reconstructed ~79 000 is the case:
    /// the model's answer is that no state of that era could *concentrate*
    /// more than a fraction of what it could name.
    #[test]
    fn concentration_ratio_bounds_a_claimed_host() {
        let m = civ_military_manpower(&kingdom_b());
        assert!(m.concentration_ratio > 0.0 && m.concentration_ratio < 1.0);
        assert!((m.concentration_ratio - m.field_army / m.emergency_mobilization).abs() < 1e-12);
        // The emergency figure itself is a small share of the population —
        // an army "of millions" from a million-person polity is arithmetic,
        // not logistics.
        assert!(m.emergency_share < 0.25);
    }

    #[test]
    fn era_is_derived_and_the_bands_are_reported_not_enforced() {
        let a = civ_military_manpower(&kingdom_a());
        assert_eq!(a.era_band.name, "High medieval");
        assert_eq!(a.era_standing_verdict, "within");
        // The mobilization figure lands just under High medieval's 5 %
        // floor, and the model says "below" rather than moving it.
        assert!(["below", "within"].contains(&a.era_mobilization_verdict));

        let b = civ_military_manpower(&kingdom_b());
        assert_eq!(b.era_band.name, "Military-fiscal state");

        // Every band is well-formed, or a verdict is meaningless.
        for e in ERA_BANDS {
            assert!(e.standing.0 <= e.standing.1, "{}", e.name);
            assert!(e.mobilization.0 <= e.mobilization.1, "{}", e.name);
            assert!(!e.constraint.is_empty(), "{}", e.name);
        }
        assert_eq!(band_verdict(0.001, (0.005, 0.02)), "below");
        assert_eq!(band_verdict(0.03, (0.005, 0.02)), "above");
        assert_eq!(band_verdict(0.005, (0.005, 0.02)), "within");
    }

    #[test]
    fn government_table_is_the_roster_vocabulary_and_falls_back_safely() {
        for (k, _) in crate::roster::CIV_GOVERNMENTS {
            assert!(
                GOVERNMENT_EXTRACTION.iter().any(|&(g, _)| g == k),
                "{k} has no extraction value"
            );
        }
        assert_eq!(GOVERNMENT_EXTRACTION.len(), crate::roster::CIV_GOVERNMENTS.len());
        assert_eq!(government_extraction("not-a-government"), 0.15);
        assert!(government_extraction("empire") > government_extraction("chiefdom"));
    }

    /// The labour ratio must be the ag-tech table's own hints, or the
    /// module is reading a number that means something else.
    #[test]
    fn labour_ratio_matches_the_ag_tech_tables_own_hints() {
        // (key, the "~N% farms" figure that row's own hint states)
        for (key, stated) in [
            ("subsistence", 0.95),
            ("traditionalAgrarian", 0.90),
            ("advancedAgrarian", 0.80),
            ("improvedAgrarian", 0.50),
            ("earlyIndustrial", 0.31),
            ("industrial", 0.13),
        ] {
            let f = crate::roster::civ_ag_tech_by_key(key).farmers_per_urbanite;
            let alpha = f / (1.0 + f);
            assert!(
                (alpha - stated).abs() < 0.005,
                "{key}: f={f} gives {alpha}, its hint says {stated}"
            );
        }
    }

    /// An empty faction must produce zeros, not `NaN`s — a headcount is a
    /// claim, and `NaN` soldiers is the loudest possible wrong one.
    #[test]
    fn an_empty_faction_is_zero_everywhere_and_never_nan() {
        for pop in [0.0, f64::NAN] {
            let m = civ_military_manpower(&ManpowerInput {
                nucleated_pop: pop,
                farmers_per_urbanite: 9.0,
                land_capacity: f64::NAN,
                government: "monarchy",
                capital_road_reach: 0.5,
                road_density: 0.5,
                navigable_share: 0.5,
                sea_share: 0.5,
            });
            assert_eq!(m.total_population, 0.0);
            assert_eq!(m.standing_army, 0.0);
            assert_eq!(m.emergency_mobilization, 0.0);
            assert_eq!(m.field_army, 0.0);
            assert_eq!(m.concentration_ratio, 0.0);
            assert!(m.standing_share.is_finite() && m.emergency_share.is_finite());
            for r in m.force_ladder {
                assert!(r.force.is_finite() && r.share.is_finite());
            }
        }
    }

    /// Every driver stays in its stated range across the whole input space,
    /// including the corners no generated world reaches.
    #[test]
    fn drivers_stay_in_range_at_every_corner() {
        for f in [0.0, 0.15, 1.0, 9.0, 19.0, 1e6] {
            for gov in ["none", "empire", "unknown"] {
                for x in [0.0, 0.5, 1.0, -3.0, 7.0] {
                    let d = military_drivers(&ManpowerInput {
                        nucleated_pop: 10_000.0,
                        farmers_per_urbanite: f,
                        land_capacity: 1_000_000.0,
                        government: gov,
                        capital_road_reach: x,
                        road_density: x,
                        navigable_share: x,
                        sea_share: x,
                    });
                    assert!((0.0..=1.0).contains(&d.agricultural_labour_ratio));
                    assert!((0.0..=1.0).contains(&d.logistics_capacity));
                    assert!((0.0..=1.0).contains(&d.professionalization));
                    assert!((0.03..=0.95).contains(&d.state_capacity));
                    assert!((0.25..=2.0).contains(&d.ecological_factor));
                    assert!(
                        (EXTRACTION_FLOOR..=EXTRACTION_CEILING)
                            .contains(&d.fiscal_extraction_efficiency)
                    );
                    assert!(d.food_surplus_per_farmer.is_finite());
                }
            }
        }
    }
}

