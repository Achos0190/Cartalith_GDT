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
//! ## Whose population is the era band a share of? (owner ruling, 2026-08-25)
//!
//! The first build of this module reported [`Manpower::era_band`] against
//! **total** population and found that the verdicts read `below` persistently
//! — recorded as finding 1 of `MILITARY_MANPOWER_SCOPE.md` §3.3, together
//! with the observation that the specification's era table disagrees with its
//! own worked example *and* with its own cited Imperial Rome figure, in one
//! consistent direction.
//!
//! **The owner has ruled that the table's percentages are shares of the
//! citizen / free population, not of the total.** The evidence is inside the
//! specification: its Republican Rome figure is stated as *"17-29 % of its
//! **citizen** population"* (Hopkins' reconstruction), and under that reading
//! Imperial Rome's ~250 000 regulars over 45-120 million stops being a factor
//! of two to five under a 1 % classical floor.
//!
//! So this module derives a [`Manpower::citizen_population`] and compares the
//! bands against **that**:
//!
//! ```text
//! citizen_fraction   = clamp(CITIZEN_SHARE[government]
//!                            + CITIZEN_MODERNISATION × urbanisation,
//!                            CITIZEN_FLOOR, CITIZEN_CEILING)
//! citizen_population = total_population × citizen_fraction
//! ```
//!
//! Two things about that, both deliberate:
//!
//! **The four outputs do not move.** The citizen fraction is a *denominator
//! for the verdict* and enters nothing else; a unit test pins every headcount
//! as a literal so that stays true.
//!
//! ## Two later corrections, owner ruling AI (b) and (c), 2026-09-23
//!
//! **(b) — geography is relative to the world, not to the map's scale.**
//! Live worlds go through [`civ_military_manpower_world`], which divides each
//! faction's `land_capacity` by the world's own land per person. The raw
//! ratio was mostly the settlement network's sparsity — forty fixed-size
//! settlements on any large map — and so tracked the map's area. See that
//! function for the measurement.
//!
//! **(c) — the soldier upkeep is the era table's, per agricultural-labour
//! bracket** ([`SOLDIER_UPKEEP_BY_BRACKET`]), replacing a flat `3.0` that
//! could not reproduce the table's Iron-Age-above-High-medieval shape. That
//! **re-baselined the standing army** of both worked examples, deliberately
//! and with the owner's acceptance: Kingdom A 5 846 → 9 661 (stated ~5 000),
//! Kingdom B 19 067 → 25 750 (stated ~20 000). Levy, field army and the
//! duration curve did not move; they are still the ones calibrated on the
//! worked examples.
//!
//! **It is derived from the government, which was already the fiscal driver,
//! plus how agrarian the society is.** Nothing in this crate distinguished a
//! citizen, free or full-status subset of population before this — grepped
//! before inventing one, the standing rule this repo has now been repaid for
//! eight times. [`crate::roster::FactionEntry`]-adjacent fields were checked
//! too: `culture` is [`crate::CIV_CULTURES`], which is name-syllable pools
//! and nothing else, and `religion` carries no social structure. Government
//! is what is actually there, and it is the right driver anyway — a citizen
//! body under a republic is a very different share of its polity than under a
//! slave-holding empire, which is precisely the distinction Hopkins' figure
//! and the Imperial Rome figure sit on either side of.
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

/// What one continuously-maintained soldier costs **the treasury**, in
/// subsistence-equivalents of captured surplus, per agricultural-labour
/// bracket — indexed by [`alpha_bracket`], the same six α thresholds
/// [`era_for`] splits the era table on, plus the `α < 0.10` remainder.
///
/// **Owner ruling AI, option (c), 2026-09-23.** Until then this was one
/// constant, `SOLDIER_UPKEEP = 3.0` (Roman legionary pay against a
/// subsistence wage). [`Manpower::standing_army`] is `budget / upkeep`, so a
/// single constant makes the standing share proportional to `(1 − α)` at
/// equal institutions — and the era table is **not** monotone in `α`: its
/// Iron Age row (1-2.5 %, α 0.85-0.93) sits *above* its High-medieval row
/// (0.5-2 %, α 0.70-0.85). No constant can reproduce that; one value per
/// bracket can.
///
/// **Derived, not chosen** — `soldier_upkeep_is_derived_from_the_era_table`
/// re-derives every entry from [`ERA_BANDS`], [`era_for`],
/// [`GOVERNMENT_EXTRACTION`] and [`CITIZEN_SHARE`] and fails if any of them
/// moves without this table moving too. The rule, applied identically to
/// each of the six [`crate::roster::AG_TECH_LEVELS`] rows (each lands in
/// exactly one bracket, so each bracket has exactly one roster α):
///
/// > over every roster government but `none` × capital reach 0, 0.1 … 1.0,
/// > at `ecological_factor = 1` and median logistics 0.5, the bracket's
/// > upkeep is the **geometric mean** of `share_at_upkeep_1 / band_centre`,
/// > where `share_at_upkeep_1` is the model's own standing share of the
/// > citizen body with upkeep 1 and `band_centre` is the arithmetic centre
/// > of the standing band of the era [`era_for`] assigns that polity.
///
/// So an average polity of each bracket lands at the centre of its own era's
/// band, and which era it lands in is still decided by state capacity, as
/// before. The `α < 0.10` entry has no roster row to derive from and repeats
/// the industrial one — extended, not derived.
///
/// **What the values say, read after the fact rather than fitted to:** the
/// cost of a soldier to the state peaks in the paid-army, fiscal-military
/// brackets (1.8-2.2 at α 0.45-0.85) and falls below one subsistence where
/// soldiers largely supported themselves (0.80 at α 0.85-0.93 — land-grant
/// and self-equipped levies, which the specification names: *"whether
/// soldiers are self-supporting"*) and again where they are conscripted
/// (1.07 industrial). A value under 1 is not a soldier eating less than a
/// peasant; it is a treasury paying for less than all of him.
///
/// Still flat in [`MilitaryDrivers::professionalization`], for the reason it
/// always was: folding professionalization in moved the two worked examples
/// in opposite directions.
pub const SOLDIER_UPKEEP_BY_BRACKET: [f64; 7] =
    [1.1200, 0.7983, 1.8153, 2.2214, 1.7696, 1.0714, 1.0714];

/// The agricultural-labour-ratio thresholds [`era_for`] splits the era table
/// on, highest first. Index `i` of [`alpha_bracket`] is `α ≥ ALPHA_BRACKETS[i]`
/// and below every earlier entry; `α` under the last one (or `NaN`) is
/// bracket `6`.
pub const ALPHA_BRACKETS: [f64; 6] = [0.93, 0.85, 0.70, 0.45, 0.25, 0.10];

/// Which [`ALPHA_BRACKETS`] bracket `alpha` falls in, `0..=6`. The one place
/// both [`era_for`] and [`soldier_upkeep`] read the thresholds from, so the
/// era a polity lands in and the upkeep calibrated for that era's bracket
/// cannot drift apart.
pub fn alpha_bracket(alpha: f64) -> usize {
    ALPHA_BRACKETS.iter().position(|&t| alpha >= t).unwrap_or(ALPHA_BRACKETS.len())
}

/// [`SOLDIER_UPKEEP_BY_BRACKET`] for this agricultural labour ratio.
pub fn soldier_upkeep(alpha: f64) -> f64 {
    SOLDIER_UPKEEP_BY_BRACKET[alpha_bracket(alpha)]
}

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

/// The floor and ceiling of [`MilitaryDrivers::ecological_factor`] -- how
/// many times over a faction's own territory feeds the population standing
/// on it.
///
/// **The ceiling was `2.0` until 2026-09-06, and it was deciding the answer
/// rather than guarding it** (owner ruling 11). Measured before it moved,
/// over 108 faction-samples -- six seeds x three world shapes, because
/// `land_capacity` integrates cell area while `nucleated_pop` does not, so
/// one world would have been one sample of the wrong thing. The raw
/// `land_capacity / total_pop` ratio runs **0.008 .. 17.9**, median 1.58,
/// p75 3.11, p90 9.88, and **45 of the 108 sat at or above 2.0** -- 24 of 36
/// on the largest shape. A bound that binds on two fifths of real inputs is
/// not an outlier guard.
///
/// **The ceiling is the reciprocal of the floor**: a factor of four either
/// way about 1.0 -- the land feeds four times the people on it, or a quarter
/// of them. The measured distribution picks the neighbourhood (4.0 sits
/// between its p75 and p90); the reciprocal picks the exact value, so the
/// constant is not fitted to the worlds it was measured on.
/// `the_ecological_clamp_is_symmetric_about_one` pins the identity, and
/// `the_ecological_ceiling_binds_where_the_ruling_put_it` pins the bound
/// itself from both sides.
///
/// **Both failure modes were live and both are named.** Lower still pins too
/// much of the sample to be a guard -- 37 of 108 at 2.5 and 30 of 108 at 3.0,
/// against 24 at 4.0 -- so the clamp would keep making the decision. Higher
/// -- the measured p90, near 10 -- pins only a tenth of the sample, but
/// [`Manpower::standing_army`] is *linear* in this factor, and at that
/// ceiling 20 of the same 108 factions read **above** their own [`ERA_BANDS`]
/// standing row where none does at 4.0. That direction is wrong on the
/// merits and not merely inconvenient: `military_budget`'s other term is the
/// *realised* surplus (the non-agricultural population the farmers already
/// feed), while this factor is only what the land could support, and paying
/// an army out of unfarmed potential is not a fiscal quantity. At 4.0, 24 of
/// 108 stay pinned and none moves above its band.
///
/// **What moving the ceiling did not fix -- and what fixed it later.** The
/// raw ratio's centre tracked world size at a fixed faction count -- median
/// 0.39 on a 512x384 800 km world against 5.04 on a 768x576 2 000 km one --
/// so part of the upper tail was map scale, not ecology. Owner ruling AI (b),
/// 2026-09-23, removed that: live worlds go through
/// [`civ_military_manpower_world`], which divides every faction's land by
/// the world's own land per person. **The distribution above was measured
/// before that normalisation** and describes raw, scale-bound ratios; the
/// bounds are kept because their rule (a factor of four either way about
/// 1.0, the reciprocal pair) never depended on it.
pub const ECOLOGICAL_FLOOR: f64 = 0.25;
/// See [`ECOLOGICAL_FLOOR`], which carries the measurement and the rule.
/// `1 / ECOLOGICAL_FLOOR`, written as a literal rather than a division so
/// that moving one end cannot silently move the other -- the identity is a
/// test's job, not a definition's.
pub const ECOLOGICAL_CEILING: f64 = 4.0;

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
/// `MILITARY_MANPOWER_SCOPE.md` and nothing else. Since owner ruling AI (c)
/// they are also an input to [`SOLDIER_UPKEEP_BY_BRACKET`]'s derivation, so
/// editing one means re-deriving that table (its test fails until you do).
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

/// The share of the total population that holds **full civic status** — the
/// citizen or free body — before [`CITIZEN_MODERNISATION`] is added, keyed by
/// [`crate::roster::CIV_GOVERNMENTS`].
///
/// **This is [`ERA_BANDS`]' denominator and nothing else at run time.** It
/// does not enter any of the four headcounts. It does enter the *derivation*
/// of [`SOLDIER_UPKEEP_BY_BRACKET`] (owner ruling AI (c)), which targets the
/// era bands as shares of this body — so editing a row here means
/// re-deriving that table, and `soldier_upkeep_is_derived_from_the_era_table`
/// fails until you do.
/// See [`Manpower::citizen_population`] for the owner's ruling that put it
/// here, and this module's *"whose population is the band a share of"*
/// section for the grounding of each row.
///
/// Ordering, and where each figure comes from:
///
/// - **Kin-based polities** (`none`, `chiefdom`, `tribal_confederacy`,
///   0.88-0.90) barely distinguish status at all: the owner's own caution
///   that a warrior band's fighters *are* its hunters and parents is the same
///   observation from the other side. Their military figures are naturally
///   quoted against the whole population, so this denominator hardly moves
///   them — which is the correct behaviour, not a missing effect.
/// - **`monarchy` / `theocracy` (0.55)** — a servile or half-free substrate
///   under free peasants, burghers and gentry. Domesday England counts about
///   10 % slaves and two-thirds villeins and bordars against roughly a
///   seventh free sokemen, and the militarily-relevant free body sits between
///   those two readings.
/// - **`republic` (0.50)** — Rome's own case, and the one the owner's ruling
///   turns on. Polybius' 225 BC figures give a citizen body of order a
///   million *with families* against an Italian population of some four
///   million including allies and slaves; the citizen half of that against
///   the polity Rome actually administered is about half.
/// - **`city_state` (0.45)** — Athens c. 431 BC: roughly 150 000 citizens
///   with families against 80-100 000 slaves and 25-50 000 metics with
///   theirs. Intensively slave-holding at a small scale.
/// - **`oligarchy` (0.40)** — enfranchisement narrows by definition. Sparta's
///   Spartiates against the helots is the extreme case and is far lower than
///   this; Venice's patriciate over a free populace is far higher. 0.40 is
///   between them and is the least-grounded row in this table.
/// - **`empire` (0.30)** — conquered subjects and large slave populations
///   stand outside the citizen body. Pre-Caracalla Roman citizens are usually
///   put at a fifth to a third of the empire, and this is the row that makes
///   the specification's own Imperial Rome figure stop being anomalous.
///
/// Unknown keys read as `chiefdom`, the same fallback
/// [`government_extraction`] takes and for the *opposite* reason: there, the
/// conservative direction is to deny an unclassifiable state an imperial
/// treasury; here, a **high** citizen fraction is the conservative one,
/// because it makes a share of that body *smaller* and so cannot flatter a
/// faction into its band.
pub const CITIZEN_SHARE: [(&str, f64); 9] = [
    ("none", 0.90),
    ("chiefdom", 0.90),
    ("tribal_confederacy", 0.88),
    ("monarchy", 0.55),
    ("theocracy", 0.55),
    ("oligarchy", 0.40),
    ("republic", 0.50),
    ("city_state", 0.45),
    ("empire", 0.30),
];

/// [`CITIZEN_SHARE`]'s lookup, with its stated fallback.
pub fn citizen_share(key: &str) -> f64 {
    CITIZEN_SHARE.iter().find(|&&(k, _)| k == key).map_or(0.90, |&(_, v)| v)
}

/// How much the citizen/free share widens as a society stops being agrarian,
/// applied to the same normalised urbanisation
/// [`MilitaryDrivers::state_capacity`] uses.
///
/// Legal servitude is an agrarian institution: chattel slavery, serfdom and
/// villeinage are all ways of binding labour to land, and they disappear —
/// everywhere, and within roughly a century of each other — as the
/// agricultural labour ratio collapses. A society where a seventh of the
/// population farms has essentially universal civic status; one where
/// nineteen twentieths do has whatever its government imposes. So the
/// denominator is not a constant per government: it is a government's floor
/// plus what modernisation adds.
///
/// **The value is derived rather than chosen**, from a single statement: *at
/// full industrialisation, civic status is universal whatever the government
/// is called*. That fixes it at `CITIZEN_CEILING − min(CITIZEN_SHARE)` =
/// `0.98 − 0.30` = `0.68`, so that even an `empire` reaches
/// [`CITIZEN_CEILING`] once its urbanisation term is 1, and every government
/// above it has already been clamped there. A unit test pins the identity, so
/// editing [`CITIZEN_SHARE`]'s lowest row without editing this fails loudly.
///
/// That also keeps the model's own [`ERA_BANDS`] internally consistent at the
/// top of the table: the industrial rows quote mobilization at 30-50 %, which
/// is only reachable at all against a denominator close to the whole
/// population.
pub const CITIZEN_MODERNISATION: f64 = 0.68;
/// The band denominator's floor — no polity is modelled as drawing its
/// soldiers from under a fifth of itself, whatever its government.
pub const CITIZEN_FLOOR: f64 = 0.20;
/// The band denominator's ceiling. Not `1.0`: children, the aged and the
/// infirm are never part of any "free population" a military figure was
/// quoted against, and the residue is what that costs.
pub const CITIZEN_CEILING: f64 = 0.98;

/// One row of the era table in `MILITARY_MANPOWER_SCOPE.md`: a name, the
/// sustainable standing-army band and the wartime-mobilization band, both as
/// shares of the **citizen / free** population
/// ([`Manpower::citizen_population`]) — the owner's 2026-08-25 ruling on
/// which denominator the supplied table meant, and the reading its own
/// *"17-29 % of its citizen population"* citation states outright.
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
    /// `land capacity / total population`, clamped to
    /// [`ECOLOGICAL_FLOOR`]..[`ECOLOGICAL_CEILING`] — whether the territory
    /// comfortably feeds the people on it (`>1`) or is drawn tight (`<1`).
    /// **This is the geography term**, and it is why two factions on the
    /// same ag-tech row do not get the same answer. The bounds are stated at
    /// [`ECOLOGICAL_FLOOR`] rather than repeated here, together with the
    /// measured distribution that set them; the ceiling last moved
    /// 2026-09-06.
    pub ecological_factor: f64,
    /// What one standing soldier costs the treasury, in subsistence-
    /// equivalents — [`soldier_upkeep`] of this polity's
    /// `agricultural_labour_ratio`. Reported because it is the divisor of
    /// [`Manpower::standing_army`] and, since owner ruling AI (c), no longer
    /// one number for every polity.
    pub soldier_upkeep: f64,
    /// `0..1` — the share of the population holding full civic status, from
    /// [`CITIZEN_SHARE`] and [`CITIZEN_MODERNISATION`].
    ///
    /// **Not a sixth variable.** It drives no headcount; its only consumer
    /// is [`Manpower::citizen_population`], which is the denominator the
    /// [`ERA_BANDS`] verdicts are read against per the owner's 2026-08-25
    /// ruling. Carried here rather than computed at the point of use so the
    /// one place that mixes governments with urbanisation stays one place.
    pub citizen_fraction: f64,
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
    /// `total_population × `[`MilitaryDrivers::citizen_fraction`] — the
    /// citizen or free body, and **the denominator [`Manpower::era_band`]'s
    /// two verdicts are read against**.
    ///
    /// The owner's 2026-08-25 ruling on an ambiguity in the supplied
    /// specification: its era table's percentages are shares of this, not of
    /// [`Manpower::total_population`], as its own *"17-29 % of its citizen
    /// population"* citation says outright. See this module's own
    /// *"Whose population is the era band a share of?"* section.
    ///
    /// **Nothing else reads it.** The four outputs are computed against
    /// total population and are unchanged by this; so is the war-duration
    /// curve, whose two anchors are stated as shares of a whole population.
    pub citizen_population: f64,
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
    /// `standing_army / total_population`. Reported for scale; it is **not**
    /// what the verdict reads — see
    /// [`standing_citizen_share`](Self::standing_citizen_share).
    pub standing_share: f64,
    /// `emergency_mobilization / total_population`. See
    /// [`standing_share`](Self::standing_share).
    pub emergency_share: f64,
    /// `standing_army / citizen_population` — **the figure
    /// [`era_standing_verdict`](Self::era_standing_verdict) compares**.
    pub standing_citizen_share: f64,
    /// `emergency_mobilization / citizen_population` — **the figure
    /// [`era_mobilization_verdict`](Self::era_mobilization_verdict)
    /// compares**.
    pub emergency_citizen_share: f64,
    /// `"within"`, `"above"` or `"below"` — where
    /// [`standing_citizen_share`](Self::standing_citizen_share) falls against
    /// [`EraBand::standing`].
    pub era_standing_verdict: &'static str,
    /// The same for
    /// [`emergency_citizen_share`](Self::emergency_citizen_share) against
    /// [`EraBand::mobilization`].
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
    ///
    /// **In whatever units the caller means "enough land for one person"**:
    /// [`civ_military_manpower`] reads `land_capacity / total_population` as
    /// is, so a hand-built input (the worked examples) states it absolutely,
    /// and [`civ_military_manpower_world`] rescales a live world's raw
    /// integral by the world's own land per person first (owner ruling AI
    /// (b)) — the raw integral grows with the map's area and the population
    /// does not.
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
    let (s, l) = (d.state_capacity, d.logistics_capacity);
    // The alpha thresholds live in ALPHA_BRACKETS, shared with
    // `soldier_upkeep`, which is calibrated per bracket.
    let idx = match alpha_bracket(d.agricultural_labour_ratio) {
        0 => {
            if s < 0.10 && l < 0.20 {
                0 // Hunter-gatherer — see ERA_BANDS' note on reachability.
            } else if s < 0.18 {
                1 // Early horticulture
            } else {
                2 // Neolithic agriculture
            }
        }
        1 => {
            if s < 0.30 {
                3 // Bronze Age state
            } else if s < 0.50 {
                4 // Iron Age agrarian state
            } else {
                5 // Classical agrarian state
            }
        }
        2 => {
            if s < 0.22 {
                6 // Late antique / early medieval
            } else if s < 0.45 {
                7 // High medieval
            } else {
                8 // Late medieval
            }
        }
        3 => {
            if s < 0.50 {
                9 // Early gunpowder
            } else {
                10 // Military-fiscal state
            }
        }
        4 => 11, // Early industrial
        5 => {
            if s >= 0.80 && l >= 0.80 {
                14 // Modern mechanized
            } else {
                13 // Total industrial mobilization
            }
        }
        _ => 14, // Modern mechanized
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
    // than open-ended in both directions -- an empty faction must not read
    // as infinitely fertile, and an over-drawn one is at famine rather than
    // at zero -- but the bounds live in ECOLOGICAL_FLOOR/ECOLOGICAL_CEILING,
    // which carry the measurement that set them and the reason the ceiling
    // moved on 2026-09-06.
    let ecological = if total_pop > 0.0 {
        js_max(ECOLOGICAL_FLOOR, js_min(ECOLOGICAL_CEILING, input.land_capacity / total_pop))
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
        soldier_upkeep: soldier_upkeep(alpha),
        // The era-band denominator, and nothing else. A government's floor,
        // widened by how far the society has left agriculture behind — see
        // CITIZEN_MODERNISATION for why those are the two terms.
        citizen_fraction: js_max(
            CITIZEN_FLOOR,
            js_min(
                CITIZEN_CEILING,
                citizen_share(input.government) + CITIZEN_MODERNISATION * urban_norm,
            ),
        ),
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
    // actually delivering, and a soldier costs `soldier_upkeep` of it --
    // one value per agricultural-labour bracket (SOLDIER_UPKEEP_BY_BRACKET).
    let non_agricultural = total_pop * (1.0 - d.agricultural_labour_ratio);
    let military_budget = non_agricultural * d.ecological_factor * d.fiscal_extraction_efficiency;
    let standing = military_budget / d.soldier_upkeep;
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

    // ---- The era band's denominator (owner ruling, 2026-08-25).
    //
    // The four outputs above are already final; this changes none of them.
    // It only decides what the table's percentages are percentages OF.
    let citizen_pop = total_pop * d.citizen_fraction;
    let citizen_share_of = |n: f64| if citizen_pop > 0.0 { n / citizen_pop } else { 0.0 };
    let standing_citizen_share = citizen_share_of(standing);
    let emergency_citizen_share = citizen_share_of(emergency);

    Manpower {
        drivers: d,
        total_population: total_pop,
        farming_population: farming,
        citizen_population: citizen_pop,
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
        standing_citizen_share,
        emergency_citizen_share,
        era_standing_verdict: band_verdict(standing_citizen_share, era.standing),
        era_mobilization_verdict: band_verdict(emergency_citizen_share, era.mobilization),
    }
}

/// People-per-unit-of-land-capacity across every faction passed in:
/// `Σ land_capacity / Σ total_population`, with the same non-finite and
/// negative coercions [`civ_military_manpower`] applies. `None` when either
/// sum is zero (no land figures, or nobody at all) — then there is nothing to
/// normalise against, and [`civ_military_manpower_world`] passes every input
/// through untouched rather than inventing a reference.
///
/// See [`civ_military_manpower_world`] for why this is the divisor.
pub fn world_land_reference(inputs: &[ManpowerInput]) -> Option<f64> {
    let (mut land, mut pop) = (0.0f64, 0.0f64);
    for i in inputs {
        let nucleated = if i.nucleated_pop.is_finite() { js_max(0.0, i.nucleated_pop) } else { 0.0 };
        if i.land_capacity.is_finite() {
            land += js_max(0.0, i.land_capacity);
        }
        pop += nucleated * (1.0 + js_max(0.0, i.farmers_per_urbanite));
    }
    let r = land / pop;
    (r.is_finite() && r > 0.0).then_some(r)
}

/// [`civ_military_manpower`] for every faction of one world, with
/// `land_capacity` expressed **relative to the world's own land per person**
/// ([`world_land_reference`]) — owner ruling AI, option (b), 2026-09-23.
///
/// ## Why the raw ratio was map scale, not ecology
///
/// `ecological_factor` is `land_capacity / total_population`, and the two
/// sides are sized by different things:
///
/// - `land_capacity` is Σ `dens × cellKm²` over the faction's territory — it
///   grows with the **physical area** of the map;
/// - `total_population` is Σ settlement populations × `(1 + f)`, and each
///   settlement is sized off a **fixed-km²** catchment per tier
///   (`civ_catchment_km2`: 6 km² hamlet … 2 500 km² metropolis), while the
///   number of settlements is `clamp(gw·gh/65536·20, 8, 40)` —
///   **grid cells, capped at 40**, never km²
///   (`place_settlements_with_water_edge_snap`).
///
/// So on a larger map the same forty settlements claim more land and the
/// ratio rises with the map's area: measured, 40 settlements on both an
/// 800 km (480 000 km²) and a 2 000 km (3 000 000 km²) world, and a median
/// raw factor of 0.32 on the first against the 4.0 ceiling on the second.
/// The ratio's absolute level is the settlement network's sparsity, which is
/// a property of the map's scale and not of anyone's farmland.
///
/// ## The anchor, and why it is not a fitted constant
///
/// Dividing by the world's own ratio removes the area term exactly (it is
/// common to every faction and to the world), and it anchors the
/// population-weighted average faction at `1.0`: *the world as a whole
/// lives at its carrying capacity*, the standard pre-modern assumption. That
/// is also the point the whole model is calibrated at — both worked examples
/// in `MILITARY_MANPOWER_SCOPE.md` are stated with land feeding exactly (A)
/// or 1.05× (B) their people. What survives is the **relative** geography:
/// a faction whose land per head is twice the world's reads 2.0.
///
/// **One coupling this introduces, stated rather than hidden:** the
/// reference is a sum over every faction, so a change that moves one
/// faction's total population (its ag-tech row, or its settlements) moves
/// every other faction's `ecological_factor` slightly. That is what a
/// relative measure means; the single-faction [`civ_military_manpower`]
/// is unchanged and still takes land in absolute terms.
pub fn civ_military_manpower_world(inputs: &[ManpowerInput]) -> Vec<Manpower> {
    let reference = world_land_reference(inputs);
    inputs
        .iter()
        .map(|i| {
            let land_capacity = reference.map_or(i.land_capacity, |r| i.land_capacity / r);
            civ_military_manpower(&ManpowerInput { land_capacity, ..*i })
        })
        .collect()
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

        // Stated: standing ~5 000. **Re-baselined by owner ruling AI (c),
        // 2026-09-23, and no longer reproduced**: the upkeep is now derived
        // from the era table rather than fitted to this example, and puts
        // Kingdom A at 9 661 (+93 % on the stated figure; 5 846 before). The
        // owner accepted exactly this re-baseline. What the new figure is
        // held to instead is the era table the upkeep was derived from:
        // High medieval, 0.5-2 % of citizens, and it lands at 1.30 % -- the
        // band's centre is 1.25 %.
        assert!((m.standing_army - 9_661.0).abs() < 1.0, "standing {}", m.standing_army);
        assert_eq!(m.era_band.name, "High medieval");
        assert_eq!(m.era_standing_verdict, "within");
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

        // Stated: standing ~20 000. Re-baselined by owner ruling AI (c):
        // 25 750 (+29 % on the stated figure; 19 067 before), 3.95 % of its
        // citizens against Military-fiscal's 1-4 % band -- inside, near the
        // top, as the strongest state the specification describes should be.
        assert!((m.standing_army - 25_750.0).abs() < 1.0, "standing {}", m.standing_army);
        assert_eq!(m.era_band.name, "Military-fiscal state");
        assert_eq!(m.era_standing_verdict, "within");
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

        // Geography alone, holding every institution fixed: a territory
        // that feeds its people four times over supports more than one that
        // is drawn tight. This is `ecological_factor`, and it is why two
        // factions on one ag-tech row and one government still differ.
        // `4_000_000` over a million people is exactly ECOLOGICAL_CEILING,
        // so this pair brackets the clamp rather than probing inside it --
        // `the_factor_still_discriminates_above_the_old_ceiling` does that.
        let fertile = civ_military_manpower(&ManpowerInput { land_capacity: 4_000_000.0, ..base });
        let barren = civ_military_manpower(&ManpowerInput { land_capacity: 300_000.0, ..base });
        assert!(fertile.standing_army > barren.standing_army * 2.0);
        assert!(fertile.drivers.food_surplus_per_farmer > barren.drivers.food_surplus_per_farmer);
        // The levy is demographic, so the land must NOT move it — a real
        // separation between the two chains rather than one number wearing
        // two hats.
        assert!((fertile.emergency_mobilization - barren.emergency_mobilization).abs() < 1e-9);
    }

    // ---- The ecological clamp's bounds (owner ruling 11, 2026-09-06).

    /// Every input below makes `total_pop` exactly `100 000`
    /// (`10 000 x (1 + 9)`), so `land_capacity / 100 000` **is** the raw
    /// ratio and each expectation is exact in `f64`.
    fn eco_at(land: f64) -> f64 {
        military_drivers(&ManpowerInput {
            nucleated_pop: 10_000.0,
            farmers_per_urbanite: 9.0,
            land_capacity: land,
            government: "monarchy",
            capital_road_reach: 0.5,
            road_density: 0.5,
            navigable_share: 0.5,
            sea_share: 0.5,
        })
        .ecological_factor
    }

    /// The clamp's bounds, pinned as **literals** from both sides. The
    /// ceiling was `2.0` until owner ruling 11 raised it; this is the test
    /// that says which value is live, and a mutant in either direction goes
    /// red here.
    #[test]
    fn the_ecological_ceiling_binds_where_the_ruling_put_it() {
        // Below the ceiling, passed through untouched. A ceiling of 2.0
        // (what shipped until the ruling), 2.5 or 3.0 fails these three.
        assert_eq!(eco_at(250_000.0), 2.5);
        assert_eq!(eco_at(350_000.0), 3.5);
        assert_eq!(eco_at(399_000.0), 3.99);
        // At and above it, pinned to 4.0. A ceiling of 5.0 fails these.
        assert_eq!(eco_at(400_000.0), 4.0);
        assert_eq!(eco_at(450_000.0), 4.0);
        assert_eq!(eco_at(100_000_000.0), 4.0);
        // The floor is outside the ruling and still binds, at the same value
        // it always had.
        assert_eq!(eco_at(30_000.0), 0.3);
        assert_eq!(eco_at(25_000.0), 0.25);
        assert_eq!(eco_at(10_000.0), 0.25);
    }

    /// The ceiling is `1 / ECOLOGICAL_FLOOR` -- a factor of four either way
    /// about 1.0 -- and that identity is the whole rule for its value, the
    /// same shape [`CITIZEN_MODERNISATION`]'s own derivation test pins.
    /// Editing one end without the other fails here.
    ///
    /// The two `assert_eq!`s against the constants check **wiring**, not
    /// value: they hold for any pair, which is exactly why the value is
    /// pinned by literals in
    /// [`the_ecological_ceiling_binds_where_the_ruling_put_it`] instead.
    #[test]
    fn the_ecological_clamp_is_symmetric_about_one() {
        assert!((ECOLOGICAL_CEILING * ECOLOGICAL_FLOOR - 1.0).abs() < 1e-12);
        assert_eq!(eco_at(1.0e12), ECOLOGICAL_CEILING);
        assert_eq!(eco_at(0.0), ECOLOGICAL_FLOOR);
    }

    /// What the ruling was *for*. Two factions whose own land feeds 2.5 and
    /// 3.5 times the people on it must get different answers; under the old
    /// `2.0` ceiling both saturated and the ratio below was exactly `1.0` --
    /// the ceiling, not the ecology, deciding.
    #[test]
    fn the_factor_still_discriminates_above_the_old_ceiling() {
        let at = |land: f64| {
            civ_military_manpower(&ManpowerInput {
                nucleated_pop: 10_000.0,
                farmers_per_urbanite: 9.0,
                land_capacity: land,
                government: "monarchy",
                capital_road_reach: 0.5,
                road_density: 0.5,
                navigable_share: 0.5,
                sea_share: 0.5,
            })
        };
        let (a, b) = (at(250_000.0), at(350_000.0));
        assert_eq!(a.drivers.ecological_factor, 2.5);
        assert_eq!(b.drivers.ecological_factor, 3.5);
        // `standing_army` is linear in the factor, so their ratio is the
        // ratio of the two land capacities and nothing else -- an
        // independent relationship rather than the constant restated.
        assert!(
            (b.standing_army / a.standing_army - 1.4).abs() < 1e-9,
            "{} / {} is not 3.5/2.5",
            b.standing_army,
            a.standing_army
        );
        // And the demographic chain does not move with it, which is what
        // makes the standing figure the *fiscal* answer of the four.
        assert_eq!(a.emergency_mobilization, b.emergency_mobilization);
        assert_eq!(a.field_army, b.field_army);
        assert_eq!(a.force_ladder[3].force, b.force_ladder[3].force);
        assert_eq!(a.emergency_duration_days, b.emergency_duration_days);
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
        }
        // A standing army is by definition indefinitely sustainable -- and
        // since owner ruling AI (c) that holds for Kingdom A only. Kingdom
        // B's derived standing army (25 750) is 6.7 % ABOVE its own 365-day
        // rung (24 129): the era table's Military-fiscal band (up to 4 % of
        // citizens) and the duration curve's "2 % for a year" anchor
        // disagree at the top of that band. Pinned as a disclosed tension
        // between two parts of the specification, not silently clamped --
        // capping one output by another is a ruling, not a fix.
        let a = civ_military_manpower(&kingdom_a());
        assert!(a.force_ladder[3].force > a.standing_army);
        let b = civ_military_manpower(&kingdom_b());
        let over = b.standing_army / b.force_ladder[3].force;
        assert!((over - 1.067).abs() < 0.001, "B standing / 365-day rung = {over}");
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

    // ---- The owner's 2026-08-25 ruling: the bands are shares of the
    //      citizen / free population, not of the total.

    /// The ruling changed a **denominator**, and this is the test that says
    /// so. Every headcount is pinned as a literal, so a future edit to
    /// [`CITIZEN_SHARE`] or [`CITIZEN_MODERNISATION`] that leaked into an
    /// output fails here rather than silently recalibrating the model.
    ///
    /// The levy and field figures are still the ones
    /// `MILITARY_MANPOWER_SCOPE.md` §3.1 published before the citizen
    /// population existed. The two standing figures are **re-baselined by
    /// owner ruling AI (c)** (5 846 -> 9 661 and 19 067 -> 25 750), which
    /// moved the soldier upkeep and nothing else -- levy and field did not
    /// move, which this test also pins.
    #[test]
    fn the_citizen_ruling_moves_no_headcount() {
        for (input, standing, levy, field) in [
            (kingdom_a(), 9_661.0, 41_221.0, 15_870.0),
            (kingdom_b(), 25_750.0, 98_889.0, 47_368.0),
        ] {
            let m = civ_military_manpower(&input);
            assert!((m.standing_army - standing).abs() < 1.0, "standing {}", m.standing_army);
            assert!(
                (m.emergency_mobilization - levy).abs() < 1.0,
                "levy {}",
                m.emergency_mobilization
            );
            assert!((m.field_army - field).abs() < 1.0, "field {}", m.field_army);
            // And the duration curve, whose anchors are stated as shares of a
            // whole population, is likewise on the total and not the citizen
            // body.
            assert!((m.standing_share - m.standing_army / m.total_population).abs() < 1e-12);
            for r in m.force_ladder {
                assert!((r.share - r.force / m.total_population).abs() < 1e-12);
            }
        }
    }

    /// What the ruling was *for*: both worked examples land inside both
    /// bands against the citizen body, where the first build read one of
    /// them `below`.
    #[test]
    fn the_citizen_ruling_lands_the_worked_examples_inside_their_bands() {
        for input in [kingdom_a(), kingdom_b()] {
            let m = civ_military_manpower(&input);
            assert!(m.citizen_population < m.total_population);
            assert!(
                (m.citizen_population - m.total_population * m.drivers.citizen_fraction).abs()
                    < 1e-6
            );
            assert!(m.standing_citizen_share > m.standing_share);
            assert!(m.emergency_citizen_share > m.emergency_share);
            assert_eq!(m.era_standing_verdict, "within", "{}", m.era_band.name);
            assert_eq!(m.era_mobilization_verdict, "within", "{}", m.era_band.name);
            // The verdict reads the citizen share and not the total one.
            assert_eq!(
                m.era_standing_verdict,
                band_verdict(m.standing_citizen_share, m.era_band.standing)
            );
            assert_eq!(
                m.era_mobilization_verdict,
                band_verdict(m.emergency_citizen_share, m.era_band.mobilization)
            );
        }
    }

    /// The denominator must actually be differentiated by government, or the
    /// ruling amounts to dividing everything by one constant — which would
    /// have been the arbitrary fraction it exists to avoid. Everything but
    /// the government key is held fixed.
    #[test]
    fn citizen_fraction_is_driven_by_government() {
        let at = |gov: &'static str| {
            civ_military_manpower(&ManpowerInput { government: gov, ..kingdom_a() })
                .drivers
                .citizen_fraction
        };
        // Kin-based polity, hardly any status distinction at all -> a
        // slave-holding empire, where the citizen body is a minority.
        assert!(at("chiefdom") > at("monarchy"));
        assert!(at("monarchy") > at("republic"));
        assert!(at("republic") > at("city_state"));
        assert!(at("city_state") > at("oligarchy"));
        assert!(at("oligarchy") > at("empire"));
        // A real spread, not six values within a rounding error.
        assert!(at("chiefdom") - at("empire") > 0.4);
    }

    /// Legal servitude is an agrarian institution: hold the government fixed
    /// and the citizen body widens as farming shrinks, converging near
    /// [`CITIZEN_CEILING`] for every government at industrial labour ratios.
    #[test]
    fn citizen_fraction_widens_as_the_society_leaves_agriculture() {
        let at = |f: f64, gov: &'static str| {
            civ_military_manpower(&ManpowerInput {
                farmers_per_urbanite: f,
                government: gov,
                ..kingdom_a()
            })
            .drivers
            .citizen_fraction
        };
        // Strictly rising across the four agrarian rows, then flat once the
        // ceiling binds -- which is the shape the claim actually makes.
        let mut prev = 0.0;
        for f in [19.0, 9.0, 4.0, 1.0] {
            let c = at(f, "monarchy");
            assert!(c > prev, "f={f} gave {c}, not above {prev}");
            prev = c;
        }
        for f in [0.45, 0.15] {
            assert!(at(f, "monarchy") >= prev);
        }
        // At the industrial row the government stops mattering entirely --
        // CITIZEN_MODERNISATION is derived from exactly that statement, and
        // this is the identity it is derived from. Editing CITIZEN_SHARE's
        // lowest row without editing CITIZEN_MODERNISATION fails here.
        let lowest = CITIZEN_SHARE.iter().map(|&(_, v)| v).fold(f64::INFINITY, f64::min);
        assert!((CITIZEN_MODERNISATION - (CITIZEN_CEILING - lowest)).abs() < 1e-12);
        for (gov, _) in CITIZEN_SHARE {
            assert!(
                (at(0.15, gov) - CITIZEN_CEILING).abs() < 1e-12,
                "{gov} does not converge at the industrial row: {}",
                at(0.15, gov)
            );
        }
        // And at subsistence a kin-based polity's band denominator is nearly
        // the whole population -- the owner's warrior-society caution, seen
        // from the denominator's side.
        assert!(at(19.0, "chiefdom") > 0.90);
    }

    #[test]
    fn citizen_share_table_is_the_roster_vocabulary_and_falls_back_safely() {
        for (k, _) in crate::roster::CIV_GOVERNMENTS {
            assert!(CITIZEN_SHARE.iter().any(|&(g, _)| g == k), "{k} has no citizen share");
        }
        assert_eq!(CITIZEN_SHARE.len(), crate::roster::CIV_GOVERNMENTS.len());
        for (k, v) in CITIZEN_SHARE {
            assert!((CITIZEN_FLOOR..=CITIZEN_CEILING).contains(&v), "{k} = {v}");
        }
        // The fallback is the *high* end here, the opposite direction to
        // `government_extraction`'s and for the reason CITIZEN_SHARE's doc
        // gives: a large denominator cannot flatter a faction into its band.
        assert_eq!(citizen_share("not-a-government"), citizen_share("chiefdom"));
        assert!(citizen_share("chiefdom") > citizen_share("empire"));
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
            assert_eq!(m.citizen_population, 0.0);
            assert!(m.standing_share.is_finite() && m.emergency_share.is_finite());
            assert!(
                m.standing_citizen_share.is_finite() && m.emergency_citizen_share.is_finite()
            );
            for r in m.force_ladder {
                assert!(r.force.is_finite() && r.share.is_finite());
            }
        }
    }

    /// Ruling AI (2026-09-23): *"a heavily industrialised nation needs less
    /// manpower to foot a larger army than an agricultural nation that is
    /// dependant on individual farmers without machines."* Option (a) of the
    /// owner's follow-up ruling accepted that the model already does this:
    /// [`Manpower::standing_army`] is paid out of the non-agricultural
    /// population, `total × (1 − α)` with `α = f/(1+f)` from
    /// [`crate::roster::AG_TECH_LEVELS`], and the same `(1 − α)` raises
    /// `state_capacity` through `urban_norm`.
    ///
    /// Pinned at **equal total population** (1 000 000), equal land (so
    /// `ecological_factor` is exactly `1.0` on every row) and Kingdom A's
    /// institutions, so the ag-tech row is the only thing that differs. The
    /// literals are the model's output, not a fit: mutating
    /// [`SOLDIER_UPKEEP_BY_BRACKET`], [`EXTRACTION_CEILING`],
    /// [`MAX_NON_AGRICULTURAL_SHARE`] or the `(1 − α)` term moves them.
    ///
    /// **Re-baselined by option (c), and no longer strictly increasing --
    /// on purpose.** Before (c) the six read 1 090 / 2 219 / 4 597 / 12 686 /
    /// 18 537 / 24 617, an 11.09x industrial/traditional spread. With the
    /// upkeep derived per era bracket, `traditionalAgrarian` (Bronze/Iron
    /// Age, self-supporting levies, upkeep 0.80) now fields MORE than
    /// `advancedAgrarian` (High medieval, paid soldiers, upkeep 1.82): the
    /// era table's own Iron-Age-above-High-medieval shape, which is exactly
    /// what (c) asked the model to reproduce. The owner's comparison still
    /// holds, and more strongly at the ends: industrial is the largest of
    /// the six by a wide margin.
    #[test]
    fn standing_army_rises_with_industrialisation_at_equal_population() {
        let at = |key: &str| {
            let f = crate::roster::civ_ag_tech_by_key(key).farmers_per_urbanite;
            civ_military_manpower(&ManpowerInput {
                nucleated_pop: 1_000_000.0 / (1.0 + f),
                farmers_per_urbanite: f,
                land_capacity: 1_000_000.0,
                ..kingdom_a()
            })
        };
        for (key, standing) in [
            ("subsistence", 2_919.0),
            ("traditionalAgrarian", 8_340.0),
            ("advancedAgrarian", 7_598.0),
            ("improvedAgrarian", 17_132.0),
            ("earlyIndustrial", 31_426.0),
            ("industrial", 68_929.0),
        ] {
            let m = at(key);
            assert!((m.total_population - 1_000_000.0).abs() < 1e-6, "{key}");
            assert_eq!(m.drivers.ecological_factor, 1.0, "{key}");
            assert!((m.standing_army - standing).abs() < 1.0, "{key}: {}", m.standing_army);
        }
        // The one deliberate inversion, stated as its own assertion so that
        // a future upkeep table which flattens it fails by name.
        assert!(at("traditionalAgrarian").standing_army > at("advancedAgrarian").standing_army);
        // Everything else still climbs, and industrial tops the six.
        for (lo, hi) in [
            ("subsistence", "traditionalAgrarian"),
            ("advancedAgrarian", "improvedAgrarian"),
            ("improvedAgrarian", "earlyIndustrial"),
            ("earlyIndustrial", "industrial"),
        ] {
            assert!(at(hi).standing_army > at(lo).standing_army, "{lo} -> {hi}");
        }
        // The owner's comparison as one number: the same million people
        // field about eight times the standing army once they no longer
        // farm by hand (11.09x before option (c)).
        let ratio = at("industrial").standing_army / at("traditionalAgrarian").standing_army;
        assert!((ratio - 8.265).abs() < 0.001, "{ratio}");
        // And the levy, which is demographic, does NOT scale that way -- the
        // industrial advantage is fiscal, which is the owner's point.
        assert!(
            at("industrial").emergency_mobilization
                < at("traditionalAgrarian").emergency_mobilization * 3.0
        );
    }

    // ---- Owner ruling AI, option (c): the upkeep is the era table's.

    /// Re-derives [`SOLDIER_UPKEEP_BY_BRACKET`] from [`ERA_BANDS`],
    /// [`era_for`], [`GOVERNMENT_EXTRACTION`] and [`CITIZEN_SHARE`] by the
    /// rule stated on the constant, independently of the constant -- so
    /// editing any of those tables without re-deriving this one fails here.
    #[test]
    fn soldier_upkeep_is_derived_from_the_era_table() {
        let govs: Vec<&str> = crate::roster::CIV_GOVERNMENTS
            .iter()
            .map(|&(k, _)| k)
            .filter(|&k| k != "none")
            .collect();
        assert_eq!(govs.len(), 8);
        let mut seen = [false; 7];
        for row in crate::roster::AG_TECH_LEVELS {
            let f = row.farmers_per_urbanite;
            let (mut log_sum, mut n) = (0.0f64, 0u32);
            let mut bracket = None;
            for &gov in &govs {
                for step in 0..=10 {
                    let d = military_drivers(&ManpowerInput {
                        nucleated_pop: 1_000.0,
                        farmers_per_urbanite: f,
                        land_capacity: 1_000.0 * (1.0 + f), // ecological 1.0
                        government: gov,
                        capital_road_reach: f64::from(step) / 10.0,
                        road_density: 0.0,
                        navigable_share: 0.0,
                        sea_share: 0.0,
                    });
                    assert_eq!(d.ecological_factor, 1.0);
                    let era = era_for(&MilitaryDrivers { logistics_capacity: 0.5, ..d });
                    let centre = (era.standing.0 + era.standing.1) / 2.0;
                    let share_at_upkeep_1 = (1.0 - d.agricultural_labour_ratio)
                        * d.fiscal_extraction_efficiency
                        / d.citizen_fraction;
                    log_sum += (share_at_upkeep_1 / centre).ln();
                    n += 1;
                    bracket = Some(alpha_bracket(d.agricultural_labour_ratio));
                }
            }
            let b = bracket.unwrap();
            assert!(!seen[b], "{}: two roster rows share bracket {b}", row.key);
            seen[b] = true;
            let derived = (log_sum / f64::from(n)).exp();
            assert!(
                (SOLDIER_UPKEEP_BY_BRACKET[b] - derived).abs() < 5e-5,
                "{}: bracket {b} holds {} but derives {derived}",
                row.key,
                SOLDIER_UPKEEP_BY_BRACKET[b]
            );
        }
        // Six roster rows, six distinct brackets; the seventh (alpha < 0.10)
        // has no row and repeats the industrial value, as documented.
        assert_eq!(seen, [true, true, true, true, true, true, false]);
        assert_eq!(SOLDIER_UPKEEP_BY_BRACKET[6], SOLDIER_UPKEEP_BY_BRACKET[5]);
    }

    /// The values themselves, as literals -- the derivation test above
    /// checks the rule, this one says which numbers are live.
    #[test]
    fn soldier_upkeep_by_alpha_literals() {
        assert_eq!(soldier_upkeep(0.95), 1.1200);
        assert_eq!(soldier_upkeep(0.90), 0.7983);
        assert_eq!(soldier_upkeep(0.80), 1.8153);
        assert_eq!(soldier_upkeep(0.50), 2.2214);
        assert_eq!(soldier_upkeep(0.31), 1.7696);
        assert_eq!(soldier_upkeep(0.13), 1.0714);
        assert_eq!(soldier_upkeep(0.05), 1.0714);
        // Bracket edges are era_for's: 0.85 is the Iron Age bracket, a hair
        // under it is High medieval's.
        assert_eq!(soldier_upkeep(0.85), 0.7983);
        assert_eq!(soldier_upkeep(0.8499), 1.8153);
        assert_eq!(soldier_upkeep(f64::NAN), 1.0714);
    }

    /// What option (c) was FOR: at each ag-tech row, a representative polity
    /// (eco 1, capital reach 0.5, median logistics) lands inside its own
    /// era's standing band -- including the non-monotone pair, where the
    /// Iron Age polity must out-arm the High-medieval one as a share of its
    /// citizens. Every share is a literal from the model; under the old flat
    /// `SOLDIER_UPKEEP = 3.0` three of the seven read `below` (Bronze 0.38 %,
    /// Iron 0.46 %, Total industrial 2.79 %) and the pair was inverted (Iron
    /// 0.46 % under High medieval 0.71 %).
    #[test]
    fn a_median_polity_of_each_bracket_lands_in_its_own_band() {
        let at = |f: f64, gov: &'static str| {
            let m = civ_military_manpower(&ManpowerInput {
                nucleated_pop: 100_000.0,
                farmers_per_urbanite: f,
                land_capacity: 100_000.0 * (1.0 + f),
                government: gov,
                capital_road_reach: 0.5,
                // logistics exactly 0.5: 0.15 + 0.45 x 7/9
                road_density: 7.0 / 9.0,
                navigable_share: 0.0,
                sea_share: 0.0,
            });
            assert!((m.drivers.logistics_capacity - 0.5).abs() < 1e-12);
            m
        };
        for (f, gov, era, share) in [
            (19.0, "monarchy", "Neolithic agriculture", 0.536),
            (9.0, "monarchy", "Bronze Age state", 1.439),
            (9.0, "republic", "Iron Age agrarian state", 1.719),
            (4.0, "monarchy", "High medieval", 1.171),
            (1.0, "monarchy", "Early gunpowder", 2.003),
            (0.45, "monarchy", "Early industrial", 3.548),
            (0.15, "monarchy", "Total industrial mobilization", 7.818),
        ] {
            let m = at(f, gov);
            let pct = 100.0 * m.standing_citizen_share;
            assert_eq!(m.era_band.name, era, "f={f} {gov}");
            assert!((pct - share).abs() < 0.001, "f={f} {gov}: {pct}");
            assert_eq!(m.era_standing_verdict, "within", "f={f} {gov}: {pct} in {era}");
        }
        // The non-monotone pair, as its own claim.
        let iron = at(9.0, "republic").standing_citizen_share;
        let high_medieval = at(4.0, "monarchy").standing_citizen_share;
        assert!(iron > high_medieval, "{iron} vs {high_medieval}");
    }

    // ---- Owner ruling AI, option (b): land per person, relative to the world.

    fn world_of(lands: [f64; 3]) -> Vec<ManpowerInput<'static>> {
        [(40_000.0, 9.0, "monarchy"), (25_000.0, 9.0, "empire"), (10_000.0, 4.0, "chiefdom")]
            .iter()
            .zip(lands)
            .map(|(&(pop, f, gov), land)| ManpowerInput {
                nucleated_pop: pop,
                farmers_per_urbanite: f,
                land_capacity: land,
                government: gov,
                capital_road_reach: 0.5,
                road_density: 0.4,
                navigable_share: 0.3,
                sea_share: 0.1,
            })
            .collect()
    }

    /// The property option (b) exists for. The same world measured at a
    /// larger map scale has every faction's `land_capacity` multiplied by the
    /// area ratio while its settlement populations do not move -- 6.25x is
    /// exactly an 800 km -> 2 000 km map at a fixed settlement count. Every
    /// output must be identical. Under the raw ratio the factor moved by
    /// 6.25x (clamp permitting) and the standing army with it.
    #[test]
    fn map_scale_does_not_move_the_ecological_factor() {
        let base = world_of([300_000.0, 180_000.0, 90_000.0]);
        let scaled = world_of([300_000.0 * 6.25, 180_000.0 * 6.25, 90_000.0 * 6.25]);
        let (a, b) = (civ_military_manpower_world(&base), civ_military_manpower_world(&scaled));
        for (x, y) in a.iter().zip(&b) {
            assert!((x.drivers.ecological_factor - y.drivers.ecological_factor).abs() < 1e-12);
            assert!((x.standing_army - y.standing_army).abs() < 1e-6);
            assert_eq!(x.emergency_mobilization, y.emergency_mobilization);
        }
        // And the control: the per-faction call, which takes land in
        // absolute terms, DOES move -- so the invariance is the
        // normalisation's doing, not an accident of the fixture.
        let raw = |w: &[ManpowerInput]| civ_military_manpower(&w[0]).drivers.ecological_factor;
        assert!(raw(&scaled) > raw(&base) * 3.0);
    }

    /// What survives the normalisation is the RELATIVE geography, exactly:
    /// the three factions' land-per-person ratios are 300k/400k, 180k/250k
    /// and 90k/50k; the world's is 570k/700k. Each factor is its own ratio
    /// over the world's, and the population-weighted mean is 1.
    #[test]
    fn the_world_anchor_keeps_relative_geography_and_averages_one() {
        let w = world_of([300_000.0, 180_000.0, 90_000.0]);
        assert!((world_land_reference(&w).unwrap() - 570_000.0 / 700_000.0).abs() < 1e-15);
        let m = civ_military_manpower_world(&w);
        let r = 57.0 / 70.0;
        for (row, e) in m.iter().zip([0.75 / r, 0.72 / r, 1.8 / r]) {
            assert!(
                (row.drivers.ecological_factor - e).abs() < 1e-12,
                "{} vs {e}",
                row.drivers.ecological_factor
            );
        }
        let weighted: f64 =
            m.iter().map(|r| r.drivers.ecological_factor * r.total_population).sum();
        let pop: f64 = m.iter().map(|r| r.total_population).sum();
        assert!((weighted / pop - 1.0).abs() < 1e-12);
        // A one-faction world is its own reference: exactly 1.0.
        let solo = civ_military_manpower_world(&w[..1]);
        assert!((solo[0].drivers.ecological_factor - 1.0).abs() < 1e-12);
    }

    /// No land figures or nobody at all: no reference, and the inputs pass
    /// through untouched rather than against an invented one.
    #[test]
    fn a_world_with_nothing_to_normalise_against_passes_through() {
        let w = world_of([0.0, 0.0, 0.0]);
        assert_eq!(world_land_reference(&w), None);
        for (a, b) in civ_military_manpower_world(&w).iter().zip(&w) {
            assert_eq!(*a, civ_military_manpower(b));
        }
        let empty: Vec<ManpowerInput> = world_of([1.0, 1.0, f64::NAN])
            .into_iter()
            .map(|i| ManpowerInput { nucleated_pop: 0.0, ..i })
            .collect();
        assert_eq!(world_land_reference(&empty), None);
        assert!(civ_military_manpower_world(&[]).is_empty());
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
                    // Literals, not the constants: an assertion written
                    // as `(ECOLOGICAL_FLOOR..=ECOLOGICAL_CEILING)` holds for
                    // every value of them and so pins nothing
                    // (`MISTAKES.md`, "a test that compares a constant
                    // against itself").
                    assert!((0.25..=4.0).contains(&d.ecological_factor));
                    assert!(
                        (CITIZEN_FLOOR..=CITIZEN_CEILING).contains(&d.citizen_fraction),
                        "citizen fraction {} out of range",
                        d.citizen_fraction
                    );
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

