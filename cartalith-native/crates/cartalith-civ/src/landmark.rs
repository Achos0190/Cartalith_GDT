//! **Landmark generation** — the record, the cap table and one pass.
//!
//! `LANDMARK_GENERATION_RESEARCH.md` is the owner-supplied specification this
//! file implements; `LANDMARK_GENERATION_SCOPE.md` is the inventory of what
//! already existed for it; `design/landmark-generation/LANDMARK_UI_DESIGN.md`
//! §9 is the wiring table the shell reads against. That design's own §9.4
//! states what this file is for:
//!
//! > Every value the panel reads or writes is owed, because the landmark record
//! > does not exist. That is one gap, not twelve … Build the record, the cap
//! > table and one pass, and all twelve resolve at once.
//!
//! ## What this is, in one paragraph
//!
//! [`kinds`] declares all 49 landmark types of research §29 — which family
//! (§29), which class (§23), a default cap, whether the type leans on the
//! viewshed field this port does not have (§9.3 of the UI design names exactly
//! six), and whether the type is **actually generated**. Twenty are: fifteen
//! off the terrain and hydrology rasters, plus the five M8 closed by threading
//! the **routed way graph** in as [`LandmarkInputs::ways`] — Road junction,
//! Bridge site, Market site, Caravan station and Trade depot, each of which
//! named exactly that missing input in the `not_built` reason it used to
//! carry. See [`EXTRACTION_RESOURCES`] for why Resource extraction site is not
//! the duplicate its name suggests, and `pool_trade_road` for why the last two
//! of the five are one function.
//! [`generate`] runs research §30's twelve steps over whichever inputs the
//! caller really has, and returns both the landmarks and — the part the UI
//! actually needs — a [`LandmarkFunnel`] per kind saying how many candidates
//! there were, where each one was lost, and **which of the limits actually
//! bound this kind**. That last field is the owner's whole question:
//! *"a maximum number does not mean that the maximum number should be placed."*
//!
//! ## §31 categories, stated in source as §31 requires
//!
//! Research §31 asks that Category A (established geographic computation),
//! Category B (empirically-inspired modelling) and Category C (Cartalith
//! synthesis) stay explicit "in both documentation and source code".
//!
//! - **Category A** — every analytical field this file reads is computed by
//!   [`cartalith_terrain::analysis`], which carries its own Category A notice:
//!   slope, curvature, TPI, local relief, normalisation. The channel-initiation
//!   threshold comes from [`cartalith_hydrology::river_flow_thresh`], the
//!   engine's one canonical copy. The spatial-competition filter is bucketed
//!   minimum-separation rejection, deliberately **not** Bridson — the private
//!   `Buckets` type carries the argument and the measurement behind it.
//! - **Category B** — the settlement-influence term is research §13's gravity
//!   form, with its one simplification stated at `Ctx::influence`.
//! - **Category C** — **every weight and every threshold below**. Each one is
//!   a named constant with a comment saying so. None of them is science; they
//!   are engineering choices, tunable, and the reason they are constants rather
//!   than literals is so a later calibration pass can find them all at once.
//!
//! ## Honest degradation is a hard rule here
//!
//! Every optional input is an `Option<&[…]>` and is additionally length-checked
//! against `gw * gh`. A kind whose input is absent reports
//! [`LandmarkLimit::NoTerrain`] with `candidates: 0` and places nothing. It
//! never invents a placement, and it never panics — this pass is called across
//! the gdext boundary, where a panic takes the Godot process down with it.

// Every `!(a > b)` / `!(a >= b)` in this file is deliberate NaN-hardening, not
// a clumsy `<`. `a < b` accepts a NaN `a` and `!(a >= b)` rejects it, and this
// pass reads f32 rasters and a settings blob that can both carry one — the
// same "JS propagates NaN where Rust absorbs it" hazard `CLAUDE.md` names, and
// the same failure reference v1.27 hardened `ScatterRule::spacing` against. The
// lint is right in general and wrong here, so it is turned off once with the
// reason rather than argued with eleven times.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use cartalith_terrain::analysis;
use rayon::prelude::*;
use std::collections::BTreeMap;

// ===========================================================================
// §23 — hierarchical landmark classes
// ===========================================================================

/// Research §23's four classes. The index order is load-bearing: it is the
/// order of [`LandmarkSettings::class_radius_km`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LandmarkClass {
    /// "Extremely rare: highest mountains, enormous lakes, exceptional
    /// deserts, major geological formations."
    Continental,
    /// "Moderately rare: major waterfalls, mountain passes, large caves, major
    /// forests, important ruins, large fortifications."
    Regional,
    /// "Common: springs, minor waterfalls, isolated rocks, small ruins,
    /// unusual trees, shrines."
    Local,
    /// "Dependent on civilization: temples, tombs, monuments, battlefields,
    /// pilgrimage sites, royal roads, border markers."
    Cultural,
}

impl LandmarkClass {
    /// `0..=3`, matching [`LandmarkSettings::class_radius_km`]'s order.
    pub fn index(self) -> usize {
        match self {
            LandmarkClass::Continental => 0,
            LandmarkClass::Regional => 1,
            LandmarkClass::Local => 2,
            LandmarkClass::Cultural => 3,
        }
    }

    /// The three-letter badge for a row's left gutter
    /// (`LANDMARK_UI_DESIGN.md` §3.1).
    ///
    /// **The dock does not call this.** `civilization_workspace.gd`'s
    /// `_lm_badge` derives the same four strings from the engine's own class
    /// key, deliberately, so a fifth class gets a badge instead of a blank
    /// gutter; `UNWIRED_FUNCTIONS.md` registers the pair and that reason.
    /// Kept as the engine-side spelling of §3.1 -- corrected 2026-09-01,
    /// where this claimed a caller it does not have.
    pub fn badge(self) -> &'static str {
        match self {
            LandmarkClass::Continental => "CON",
            LandmarkClass::Regional => "REG",
            LandmarkClass::Local => "LOC",
            LandmarkClass::Cultural => "CUL",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            LandmarkClass::Continental => "continental",
            LandmarkClass::Regional => "regional",
            LandmarkClass::Local => "local",
            LandmarkClass::Cultural => "cultural",
        }
    }

    pub fn all() -> [LandmarkClass; 4] {
        [
            LandmarkClass::Continental,
            LandmarkClass::Regional,
            LandmarkClass::Local,
            LandmarkClass::Cultural,
        ]
    }
}

// ===========================================================================
// §29 — families
// ===========================================================================

/// Research §29's six groupings. `LANDMARK_UI_DESIGN.md` §3.1 makes these the
/// L4 groups of the dock, one open at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LandmarkFamily {
    Physical,
    Transport,
    Economic,
    Military,
    Cultural,
    Historical,
}

impl LandmarkFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            LandmarkFamily::Physical => "physical",
            LandmarkFamily::Transport => "transport",
            LandmarkFamily::Economic => "economic",
            LandmarkFamily::Military => "military",
            LandmarkFamily::Cultural => "cultural",
            LandmarkFamily::Historical => "historical",
        }
    }

    /// The group header the dock draws (`LANDMARK_UI_DESIGN.md` §3.2).
    pub fn label(self) -> &'static str {
        match self {
            LandmarkFamily::Physical => "Physical",
            LandmarkFamily::Transport => "Transportation",
            LandmarkFamily::Economic => "Economic",
            LandmarkFamily::Military => "Military",
            LandmarkFamily::Cultural => "Religious · cultural",
            LandmarkFamily::Historical => "Historical",
        }
    }

    pub fn all() -> [LandmarkFamily; 6] {
        [
            LandmarkFamily::Physical,
            LandmarkFamily::Transport,
            LandmarkFamily::Economic,
            LandmarkFamily::Military,
            LandmarkFamily::Cultural,
            LandmarkFamily::Historical,
        ]
    }
}

// ===========================================================================
// The kind table
// ===========================================================================

/// One row of the type table — research §29's list, graded by §23 and tagged
/// with what this port can and cannot honestly produce.
#[derive(Clone, Copy, Debug)]
pub struct LandmarkKindSpec {
    /// Stable key. Used in [`LandmarkSettings`]' maps, in [`Landmark::kind`]
    /// and in [`LandmarkFunnel::kind`]; never shown to a user.
    pub key: &'static str,
    /// The row label the dock draws.
    pub label: &'static str,
    pub family: LandmarkFamily,
    pub class: LandmarkClass,
    /// The cap [`LandmarkSettings::default`] starts this kind at. Where the
    /// design's own artboards name a number for a type (`Dock.dc.html`: Peak
    /// 24, Cliff 30, Gorge 8, Waterfall 40), that number is used here, so the
    /// shipped panel and the drawn one agree.
    pub default_cap: u32,
    /// `LANDMARK_UI_DESIGN.md` §9.3: six of the 49 lean on a viewshed field
    /// that does not exist anywhere in this workspace, and "the panel must say
    /// so on the row, not in a footnote". These are those six.
    pub needs_viewshed: bool,
    /// `false` = the type is **declared and honestly not generated**. The UI
    /// lists it, shows [`not_built`](Self::not_built) as the reason and never
    /// pretends a run could place one.
    pub buildable: bool,
    /// Why a `buildable: false` kind is not generated. Empty for buildable
    /// kinds.
    ///
    /// **Not in the agreed contract** — added because "declared and honestly
    /// NOT generated" is only honest if the panel can say *why*, and the
    /// alternative was three dozen-odd reasons living in GDScript where they
    /// would drift from the engine that owns them (36 at the time this field
    /// was added; 34 after the M8 residual pass built Resource extraction
    /// site; 29 once the way graph closed the five that named it). Purely
    /// additive.
    pub not_built: &'static str,
}

/// Research §29's full list — 15 Physical, 8 Transportation, 6 Economic,
/// 6 Military, 8 Religious/Cultural, 6 Historical = **49**, the same count
/// `LANDMARK_UI_DESIGN.md` §3 works from.
///
/// **One divergence from §29, disclosed rather than smoothed over.** §29 lists
/// *Battlefield* twice — once under Military, once under Historical — and a
/// key must be unique, so the Historical entry is keyed `battlefield_historic`
/// and labelled "Historic battlefield". The alternative was 48 rows and a
/// Historical family of five, which would silently contradict the design's own
/// per-family counts.
///
/// The class column is a **Category C** judgement: §23 gives examples, not a
/// mapping, and every assignment here is a Cartalith engineering choice.
#[rustfmt::skip]
pub fn kinds() -> &'static [LandmarkKindSpec] {
    use LandmarkClass as C;
    use LandmarkFamily as F;
    &[
        // ---------------- Physical (15) ----------------
        LandmarkKindSpec { key: "peak", label: "Peak", family: F::Physical, class: C::Regional, default_cap: 24, needs_viewshed: true, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "ridge", label: "Ridge", family: F::Physical, class: C::Regional, default_cap: 20, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "saddle", label: "Saddle", family: F::Physical, class: C::Local, default_cap: 16, needs_viewshed: false, buildable: false,
            not_built: "A saddle with connectivity is a mountain pass, which is generated; a saddle without it is a shape, not a landmark. Generating both would put two records on one cell." },
        LandmarkKindSpec { key: "cliff", label: "Cliff", family: F::Physical, class: C::Local, default_cap: 30, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "gorge", label: "Gorge", family: F::Physical, class: C::Regional, default_cap: 8, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "cave", label: "Cave", family: F::Physical, class: C::Local, default_cap: 12, needs_viewshed: false, buildable: false,
            not_built: "Needs a karst/void model. Lithology classifies rock type but nothing in this engine dissolves it, so a cave could only be placed at random on limestone." },
        LandmarkKindSpec { key: "waterfall", label: "Waterfall", family: F::Physical, class: C::Regional, default_cap: 40, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "spring", label: "Spring", family: F::Physical, class: C::Local, default_cap: 30, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "lake", label: "Lake", family: F::Physical, class: C::Regional, default_cap: 16, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "delta", label: "Delta", family: F::Physical, class: C::Regional, default_cap: 6, needs_viewshed: false, buildable: false,
            not_built: "A river mouth is detectable; a delta is a deposition landform and this engine carries no sediment budget. Placing one at every mouth would be a rename, not a detection." },
        LandmarkKindSpec { key: "river_confluence", label: "River confluence", family: F::Physical, class: C::Local, default_cap: 20, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "volcanic_feature", label: "Volcanic feature", family: F::Physical, class: C::Regional, default_cap: 10, needs_viewshed: true, buildable: false,
            not_built: "The volcanism raster now reaches this pass (LandmarkInputs::volcanism, plumbed from WorldState::volcanic_field alongside lithology and resistance) — but nothing here reads it yet, and §9.3 still marks this as one of the six types whose dominant term is the missing viewshed. Owner decision 2 (LM-7) is the real blocker, not the input." },
        LandmarkKindSpec { key: "rock_formation", label: "Rock formation", family: F::Physical, class: C::Local, default_cap: 20, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "glacial_feature", label: "Glacial feature", family: F::Physical, class: C::Regional, default_cap: 10, needs_viewshed: false, buildable: false,
            not_built: "No general glaciation model. cartalith-terrain's fjord module reconstructs one specific glacial landform from paleoclimate temperature, relief and lithology, but covers coastal fjords only — not a cirque, moraine or esker inland — and needs three inputs this pass does not take." },
        LandmarkKindSpec { key: "ancient_forest", label: "Ancient forest", family: F::Physical, class: C::Continental, default_cap: 8, needs_viewshed: false, buildable: false,
            not_built: "Biome says where forest is; nothing says how old it is. Age is what makes the forest a landmark." },
        // ---------------- Transportation (8) ----------------
        LandmarkKindSpec { key: "mountain_pass", label: "Mountain pass", family: F::Transport, class: C::Regional, default_cap: 12, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "river_crossing", label: "River crossing", family: F::Transport, class: C::Local, default_cap: 20, needs_viewshed: false, buildable: false,
            not_built: "A crossing is a route crossing a river, and now that the routed way graph reaches this pass that is exactly what Ford and Bridge site already are — the same channel cells, partitioned at the flow a traveller can wade. A third record on them would be a rename, not a detection. (This reason used to read 'without a routed way network'; that blocker is gone, the double-count is not.)" },
        LandmarkKindSpec { key: "ford", label: "Ford", family: F::Transport, class: C::Local, default_cap: 24, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "bridge_site", label: "Bridge site", family: F::Transport, class: C::Local, default_cap: 12, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "road_junction", label: "Road junction", family: F::Transport, class: C::Local, default_cap: 20, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "caravan_station", label: "Caravan station", family: F::Transport, class: C::Local, default_cap: 10, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "portage", label: "Portage", family: F::Transport, class: C::Local, default_cap: 8, needs_viewshed: false, buildable: false,
            not_built: "Needs labelled drainage basins to know that two navigable waters belong to different ones. The scope inventory confirmed no basin entity exists." },
        LandmarkKindSpec { key: "harbour", label: "Harbour", family: F::Transport, class: C::Regional, default_cap: 16, needs_viewshed: false, buildable: true, not_built: "" },
        // ---------------- Economic (6) ----------------
        LandmarkKindSpec { key: "mine", label: "Mine", family: F::Economic, class: C::Local, default_cap: 24, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "quarry", label: "Quarry", family: F::Economic, class: C::Local, default_cap: 20, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "salt_works", label: "Salt works", family: F::Economic, class: C::Local, default_cap: 8, needs_viewshed: false, buildable: false,
            not_built: "The salt potential exists, but rock salt and a coastal salt pan are different installations in the same field and nothing here separates them. build_resource_potentials computes it as one arid-lowland evaporite-basin model with no coastal term at all, so a coastal split would be invented, not read. Mine already covers the rock-salt shape." },
        // Built from EXTRACTION_RESOURCES (timber, sulfur, alum) — the three
        // potentials Mine and Quarry do not read — via the same pool_resource
        // test those two use. See EXTRACTION_RESOURCES's own doc for why this
        // is not the duplicate the name suggests.
        LandmarkKindSpec { key: "resource_extraction_site", label: "Resource extraction site", family: F::Economic, class: C::Local, default_cap: 16, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "market_site", label: "Market site", family: F::Economic, class: C::Local, default_cap: 12, needs_viewshed: false, buildable: true, not_built: "" },
        LandmarkKindSpec { key: "trade_depot", label: "Trade depot", family: F::Economic, class: C::Local, default_cap: 10, needs_viewshed: false, buildable: true, not_built: "" },
        // ---------------- Military (6) ----------------
        LandmarkKindSpec { key: "fort", label: "Fort", family: F::Military, class: C::Regional, default_cap: 16, needs_viewshed: true, buildable: false,
            not_built: "§18's own model puts F_visibility at 0.20 — the joint-largest term — and there is no viewshed field anywhere in this workspace. A fort scored without it is a defensible hill, not a fort." },
        LandmarkKindSpec { key: "watchtower", label: "Watchtower", family: F::Military, class: C::Local, default_cap: 20, needs_viewshed: true, buildable: false,
            not_built: "A watchtower is visibility and nothing else. Without a viewshed there is no term left to score." },
        LandmarkKindSpec { key: "fortified_pass", label: "Fortified pass", family: F::Military, class: C::Regional, default_cap: 8, needs_viewshed: false, buildable: false,
            not_built: "Downstream of Fort. The pass half is generated; the fortification half waits on the same viewshed." },
        LandmarkKindSpec { key: "fortified_crossing", label: "Fortified crossing", family: F::Military, class: C::Local, default_cap: 8, needs_viewshed: false, buildable: false,
            not_built: "Downstream of Fort, over a Ford or a bridge site." },
        LandmarkKindSpec { key: "battlefield", label: "Battlefield", family: F::Military, class: C::Cultural, default_cap: 12, needs_viewshed: false, buildable: false,
            not_built: "There is no conflict entity in this port. STORY_PLANNING_SCOPE.md SP-4 is not started, so a battlefield could only be a place where nothing recorded happened." },
        LandmarkKindSpec { key: "border_marker", label: "Border marker", family: F::Military, class: C::Cultural, default_cap: 16, needs_viewshed: true, buildable: false,
            not_built: "Territory boundaries exist, but §9.3 lists this among the six viewshed-dominant types: a marker is placed to be seen from the border." },
        // ---------------- Religious / cultural (8) ----------------
        LandmarkKindSpec { key: "shrine", label: "Shrine", family: F::Cultural, class: C::Local, default_cap: 30, needs_viewshed: false, buildable: false,
            not_built: "§26 is explicit that cultural meaning must not be hardcoded into geography — one mountain, three civilisations, three readings. That needs the civilisation's own traits as an input, which this pass does not take." },
        LandmarkKindSpec { key: "temple", label: "Temple", family: F::Cultural, class: C::Cultural, default_cap: 16, needs_viewshed: false, buildable: false,
            not_built: "The same gap as Shrine: awaiting §26's cultural-interpretation layer." },
        LandmarkKindSpec { key: "sacred_grove", label: "Sacred grove", family: F::Cultural, class: C::Cultural, default_cap: 12, needs_viewshed: false, buildable: false,
            not_built: "The same gap as Shrine, plus the forest-age gap Ancient forest names." },
        LandmarkKindSpec { key: "sacred_mountain", label: "Sacred mountain", family: F::Cultural, class: C::Cultural, default_cap: 6, needs_viewshed: true, buildable: false,
            not_built: "§19's model is 0.20 F_visibility and 0.15 F_cultural. Both are missing; the physical half is already generated as Peak." },
        LandmarkKindSpec { key: "pilgrimage_site", label: "Pilgrimage site", family: F::Cultural, class: C::Cultural, default_cap: 8, needs_viewshed: false, buildable: false,
            not_built: "§35's chain reaches a pilgrimage route through a shrine. The shrine is not generated, so this cannot be either." },
        LandmarkKindSpec { key: "tomb", label: "Tomb", family: F::Cultural, class: C::Cultural, default_cap: 12, needs_viewshed: false, buildable: false,
            not_built: "Needs a named historical figure to be the tomb of. No such entity exists." },
        LandmarkKindSpec { key: "monument", label: "Monument", family: F::Cultural, class: C::Cultural, default_cap: 12, needs_viewshed: false, buildable: false,
            not_built: "§24 wants a monument to be the product of world history — a battle, a founding, a victory. None of those events exist as records yet." },
        LandmarkKindSpec { key: "ceremonial_site", label: "Ceremonial site", family: F::Cultural, class: C::Cultural, default_cap: 10, needs_viewshed: false, buildable: false,
            not_built: "The same gap as Shrine: awaiting §26's cultural-interpretation layer." },
        // ---------------- Historical (6) ----------------
        LandmarkKindSpec { key: "ruin", label: "Ruin", family: F::Historical, class: C::Regional, default_cap: 20, needs_viewshed: false, buildable: false,
            not_built: "§20 is explicit that a ruin comes from a settlement's own decline chain. timeline.rs's civ_collapse_step models that decline but drops an abandoned settlement from its output entirely rather than retaining the site's coordinates, so there is nothing to place a Ruin at even before the Conflict link in that chain (SP-4) is reached." },
        LandmarkKindSpec { key: "abandoned_settlement", label: "Abandoned settlement", family: F::Historical, class: C::Local, default_cap: 12, needs_viewshed: false, buildable: false,
            not_built: "The same chain as Ruin, one state earlier in §25 — and the same missing coordinates: civ_collapse_step drops the site rather than marking it abandoned in place." },
        LandmarkKindSpec { key: "ancient_road", label: "Ancient road", family: F::Historical, class: C::Regional, default_cap: 8, needs_viewshed: false, buildable: false,
            not_built: "Needs a superseded route to be the ghost of. Way history is not retained." },
        LandmarkKindSpec { key: "battlefield_historic", label: "Historic battlefield", family: F::Historical, class: C::Cultural, default_cap: 8, needs_viewshed: false, buildable: false,
            not_built: "§29 lists Battlefield in both Military and Historical; this is the second listing, keyed apart so the table has 49 unique rows. Blocked on the same missing conflict entity." },
        LandmarkKindSpec { key: "destroyed_fortress", label: "Destroyed fortress", family: F::Historical, class: C::Regional, default_cap: 8, needs_viewshed: false, buildable: false,
            not_built: "Downstream of Fort and of the conflict entity — both missing." },
        LandmarkKindSpec { key: "historic_crossing", label: "Historic crossing", family: F::Historical, class: C::Local, default_cap: 8, needs_viewshed: false, buildable: false,
            not_built: "A crossing that mattered. The crossing is generated as Ford; what made it matter is route history, which is not retained." },
    ]
}

/// The spec for one key, or `None` if the key is not in [`kinds`].
pub fn kind_spec(key: &str) -> Option<&'static LandmarkKindSpec> {
    kinds().iter().find(|k| k.key == key)
}

// ===========================================================================
// §22 — the landmark record
// ===========================================================================

/// Research §22's object model, restricted to the fields this pass can fill
/// honestly. The bases it does **not** carry (`geological_basis`,
/// `ecological_basis`, `visibility_score`, `political_associations`,
/// `historical_state`) are absent rather than zeroed, because a zero in a
/// suitability model is a claim and "we did not measure this" is not.
#[derive(Clone, Debug, PartialEq)]
pub struct Landmark {
    /// Sequential within one [`generate`] call, from 1. Deterministic, because
    /// the emission order is.
    pub id: u64,
    /// A [`LandmarkKindSpec::key`].
    pub kind: String,
    pub class: LandmarkClass,
    pub x: usize,
    pub y: usize,
    /// **Metres above sea level**, using the same `metersPerUnit` anchoring as
    /// the Sample and Measure panels: `(field - sea) / (1 - sea) * peak_m`.
    pub elevation: f64,
    /// `0..1` suitability — §17's weighted sum of normalised terms, divided by
    /// the weight of the terms that were actually measurable.
    pub score: f64,
    /// `0..1` emergent importance — §24. Never a random rarity roll.
    pub importance: f64,
    /// §22's `causal_chain`, ordered cause → consequence → landmark, with the
    /// real measured values in it. The last element is the type's own label.
    pub causal: Vec<String>,
    /// §27's `seed_L = Hash(worldSeed, featureID, landmarkClass,
    /// generationVersion)`. Stable for a given world and cell across runs and
    /// across cap changes; it is what a later cultural or naming pass should
    /// derive from, never the [`id`](Self::id).
    pub seed: u64,
}

// ===========================================================================
// The funnel — §5 of the UI design
// ===========================================================================

/// Which of the limits actually bound a kind. The one field the owner's brief
/// is entirely about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandmarkLimit {
    /// The cap was the binding constraint — `placed == cap`. Dragging the
    /// slider right will produce more.
    AtCap,
    /// The exclusion radius rejected the rest. Dragging right will not help;
    /// lowering Crowding will.
    Spacing,
    /// Every candidate failed this type's own physical requirements or scored
    /// below the floor — or the input the type needs is not present at all.
    NoTerrain,
    /// The candidate pool was exhausted before either the cap or spacing bound
    /// it. The world is too small or too coarse for more of this type.
    Candidates,
    /// The type is armed off, or its cap is zero. Nothing was attempted.
    Disarmed,
    /// The type is declared but not generated by this engine — see
    /// [`LandmarkKindSpec::not_built`].
    NotBuildable,
}

impl LandmarkLimit {
    /// The one-word token the dock draws after the placed count
    /// (`LANDMARK_UI_DESIGN.md` §2.2). All wording beyond this is the
    /// caller's, the same division `explain_settlement` already draws.
    pub fn as_str(self) -> &'static str {
        match self {
            // **Machine keys, not display words**, and the distinction cost a
            // real defect before it was fixed.
            //
            // These first shipped as `"at cap"`, `"no terrain"`, `"off"` and
            // `"not generated"` -- the words a user reads. The shell compares
            // this token against `"at_cap"` to decide whether to draw a row's
            // second line in accent, per the panel's own §2.2 rule that "a
            // panel where nothing is at cap has no accent on any second line".
            //
            // The mismatch failed in the worst available way: the printed word
            // still read correctly, because the shell falls back to echoing an
            // unrecognised token, so the row LOOKED right while the accent and
            // its whole tooltip were silently dead -- measured at 0 of 11
            // at-cap rows accented. Caught by `_landmark_probe.gd`, which
            // asserts the styling rather than the text.
            //
            // Wording belongs to the shell, which owns `LM_LIMIT_WORD` and
            // would own localisation; the wire format belongs here and is
            // stable. `as_str()` was doing both jobs and could only do one.
            LandmarkLimit::AtCap => "at_cap",
            LandmarkLimit::Spacing => "spacing",
            LandmarkLimit::NoTerrain => "no_terrain",
            LandmarkLimit::Candidates => "candidates",
            LandmarkLimit::Disarmed => "disarmed",
            LandmarkLimit::NotBuildable => "not_buildable",
        }
    }
}

/// The "why fewer than I asked for" arithmetic, per kind, from the last run —
/// `LANDMARK_UI_DESIGN.md` §5's popover.
///
/// **The counters are the product, not a debug aid.** The identity
///
/// ```text
/// candidates == rejected_constraint + rejected_score + rejected_spacing
///             + rejected_cap + placed
/// ```
///
/// holds for every funnel this module emits, and is asserted in
/// `funnel_arithmetic_closes_on_every_kind`.
///
/// ## Why there is a fifth bucket
///
/// There were four. When the **cap** bound, the walk over the score-sorted
/// survivors stopped and every candidate it never reached was counted in
/// `rejected_score`. That was arguable — the list is score-sorted, so an
/// unreached candidate is by construction at or below the last placed one's
/// score, and the cap does raise the effective floor to exactly that value.
///
/// It was also wrong for this popover's one job. §5 exists to answer *"why
/// fewer than I asked for"*, and when the cap binds the honest answer is **you
/// got exactly what you asked for** — nothing was rejected for being unsuitable.
/// Folding those into `rejected_score` made the popover report a quality
/// judgement the generator never made: a row reading `rejected by score: 3`
/// when the truth is `3 more would have fit, you capped at 24`. Two different
/// sentences, and the owner's whole question is which one is true.
///
/// So `rejected_score` now carries exactly one meaning — **scored below the
/// floor** — and `rejected_cap` carries **passed everything and the cap ran
/// out**. The distinction is visible in the popover and in `limit`.
#[derive(Clone, Debug, PartialEq)]
pub struct LandmarkFunnel {
    pub kind: String,
    /// How many cells entered the funnel for this kind.
    pub candidates: usize,
    /// Failed a hard physical requirement (§2.1) — the constraint block
    /// §7 writes out longhand for a waterfall.
    pub rejected_constraint: usize,
    /// Scored below the floor, and nothing else. See the note above.
    pub rejected_score: usize,
    /// Lost to an exclusion radius (§16).
    pub rejected_spacing: usize,
    /// Passed every constraint and every spacing test, and would have been
    /// placed — the cap ran out first. **This is the bucket that means the
    /// user got what they asked for**, not that anything was found wanting.
    pub rejected_cap: usize,
    /// The cap in force for this kind on this run.
    pub cap: usize,
    pub placed: usize,
    /// Which of the limits actually bound this kind.
    pub limit: LandmarkLimit,
}

impl LandmarkFunnel {
    /// A funnel for a kind that was never run — disarmed, not buildable, or
    /// missing its input.
    fn empty(kind: &str, cap: usize, limit: LandmarkLimit) -> Self {
        LandmarkFunnel {
            kind: kind.to_string(),
            candidates: 0,
            rejected_constraint: 0,
            rejected_score: 0,
            rejected_spacing: 0,
            rejected_cap: 0,
            cap,
            placed: 0,
            limit,
        }
    }

    /// The identity this module guarantees. Public so a caller can assert it
    /// too rather than trusting this file.
    pub fn closes(&self) -> bool {
        self.candidates
            == self.rejected_constraint
                + self.rejected_score
                + self.rejected_spacing
                + self.rejected_cap
                + self.placed
    }
}

// ===========================================================================
// Rejected candidates — the funnel's coordinates
// ===========================================================================

/// Why a candidate that reached the ranked walk did not become a placement.
///
/// **Three, not four.** [`LandmarkFunnel::rejected_constraint`] has no member
/// here and deliberately never will: that bucket is counted per *scanned cell*
/// inside each detector, so its "reason" is "this cell is not a waterfall" and
/// its population is the land raster. Measured at the shell's default
/// 2048×1311 it runs to millions — see [`LandmarkReject`]'s own note. The
/// three below are the ones the ranked walk sees, and they are exactly the
/// ones where the answer is actionable: raise the cap, change the spacing, or
/// lower the floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandmarkRejectReason {
    /// Scored below [`SCORE_FLOOR`]. With the shipped floor of `0.0` this is
    /// unreachable by construction — kept because the floor is the one knob a
    /// calibration pass would turn, and a reason that appears the day it moves
    /// is better than one added then.
    Score,
    /// Fell inside a placed landmark's exclusion radius (§16). The one reason
    /// that carries [`LandmarkReject::needs_crowding`].
    Spacing,
    /// Passed every constraint and every spacing test, and the cap ran out
    /// first — [`LandmarkFunnel::rejected_cap`]'s own meaning. **These are the
    /// rows that say "you got what you asked for"**, not that anything was
    /// found wanting.
    Cap,
}

impl LandmarkRejectReason {
    /// The machine key, on the same wire-format-not-wording rule
    /// [`LandmarkLimit::as_str`] states at length: the shell owns the words.
    pub fn as_str(self) -> &'static str {
        match self {
            LandmarkRejectReason::Score => "score",
            LandmarkRejectReason::Spacing => "spacing",
            LandmarkRejectReason::Cap => "cap",
        }
    }
}

/// One candidate the pass offered and did not place: **where it is, and why it
/// lost**.
///
/// `LARGE_ITEM_RULINGS.md`'s Landmark-funnel row asks for "a rejected-candidate
/// coordinate list". A coordinate alone would be a worse diagnostic than the
/// scalar it supplements — a dot that cannot say why it is a dot — so every row
/// carries its [`reason`](Self::reason), and a spacing rejection additionally
/// carries the number that would undo it.
///
/// ## Why the list is capped, and why that is not a compromise
///
/// [`REJECT_LIST_MAX_PER_KIND`] bounds the rows retained per kind. Two reasons,
/// and the first is a measured hazard rather than a tidiness argument:
///
/// 1. **This list crosses the gdext boundary.** Without `experimental-threads`
///    every `Dictionary`/`Array`/`GString` operation routes through
///    `ensure_main_thread()`, and a per-candidate payload marshalled from a
///    worker is precisely the shape that froze this app once already (the POI
///    pass, 4.14 s → 0.39 s once the payload was made primitive). `rejected_cap`
///    is unbounded by construction — set a cap of 1 on a kind with 40 000
///    candidates and 39 999 of them are "rejected by the cap" — so an unbounded
///    list is a freeze waiting for one slider drag.
/// 2. **The walk is score-sorted**, so retaining the *first* N per kind retains
///    the N best-scoring near-misses. Those are the ones the question is about;
///    the ten-thousandth-ranked cliff is not a fact anybody wants drawn.
///
/// Nothing is hidden by the cap: [`LandmarkFunnel`]'s counters are unchanged and
/// still carry the true totals, so a caller can always say "showing 256 of
/// 39 999" from data it already has. That is why no `truncated` field was added
/// here — it would be a third copy of a number the funnel already owns.
#[derive(Clone, Debug, PartialEq)]
pub struct LandmarkReject {
    /// [`LandmarkKindSpec::key`]. A `&'static str` rather than the `String`
    /// [`Landmark::kind`] and [`LandmarkFunnel::kind`] use, on purpose: those
    /// are one row per *placement* and one per *kind*, this is one row per
    /// *rejected candidate*, and a heap allocation per row is a cost the other
    /// two do not pay. The keys are `'static` in [`kinds`] already.
    pub kind: &'static str,
    /// Grid cell, the same space [`Landmark::x`]/[`Landmark::y`] use — the
    /// overlay must draw these with `_cell_to_screen`, not `_point_to_screen`.
    pub x: usize,
    pub y: usize,
    /// The §17 suitability this candidate scored, `0..1`. Comparable with
    /// [`Landmark::score`] within a kind, which is what makes "it was nearly
    /// good enough" a statement rather than a feeling.
    pub score: f64,
    pub reason: LandmarkRejectReason,
    /// **The smallest [`LandmarkSettings::crowding`] at which this candidate
    /// would have cleared the ring that blocked it.** `None` for every reason
    /// but [`LandmarkRejectReason::Spacing`], and `None` when the blocking
    /// landmark sits on this very cell, where no finite crowding helps.
    ///
    /// Higher crowding packs *tighter* (the radius is divided by it), so this
    /// is always greater than the crowding in force. It may exceed
    /// `crowding`'s own `3.0` ceiling, and is deliberately **not** clamped to
    /// it: a value of `7.4` is the honest answer "not at any setting", and
    /// clamping would turn that into the false answer "set it to 3".
    pub needs_crowding: Option<f64>,
}

/// How many rejected candidates are retained per kind, best-scoring first.
///
/// **Category C** — an engineering bound, not a modelling one. Sized so that
/// all 20 buildable kinds together stay in the low thousands of rows: enough
/// that a map at fit zoom shows the shape of the competition, few enough that
/// the boundary crossing stays in the sub-millisecond band the measurement in
/// [`LandmarkReject`]'s note is about. See that note for why a bound exists at
/// all.
pub const REJECT_LIST_MAX_PER_KIND: usize = 256;

// ===========================================================================
// Settings — the cap table
// ===========================================================================

/// Default exclusion radius per class, in km, at `crowding == 1.0`.
///
/// **Category C.** §16 asks for `r = f(class, importance, terrain, region)`
/// and gives a four-step ladder in words, not numbers. The Regional figure is
/// the one the design's own artboards draw (`Dock.dc.html`, `TypeRow.dc.html`,
/// `Phone.dc.html`, `WhyFewer.dc.html` all read "a regional landmark keeps
/// 34 km clear"), so the shipped default and the drawn one agree; the other
/// three are a ladder around it.
pub const DEFAULT_CLASS_RADIUS_KM: [f64; 4] = [200.0, 34.0, 10.0, 6.0];

/// **Category C.** The suitability a candidate must reach to be placed at all,
/// on §17's `0..1` scale.
///
/// **Zero, and that is a decision rather than an oversight.** Two independent
/// reasons:
///
/// 1. §30 has no suitability-rejection step. Step 6 rejects on physical
///    constraints, step 8 on spacing; step 7 only *ranks*. A score floor is
///    this port's own addition, not the research's.
/// 2. The terms are normalised over the candidate pool with
///    [`analysis::normalise`], so the pool's weakest candidate scores exactly
///    `0.0` on every term **by construction**. Any positive floor therefore
///    rejects it as a matter of arithmetic, not of quality — and on a pool of
///    two strong candidates it would reject one of them outright.
///
/// It is kept as a named constant rather than deleted because it is the one
/// place a later calibration pass would add a quality gate, and because
/// [`LandmarkFunnel::rejected_score`] still carries the candidates that lost
/// the ranked competition once the cap ran out.
pub const SCORE_FLOOR: f64 = 0.0;

/// The generator version that goes into §27's seed hash. **Bump this when a
/// change to this file should legitimately move existing landmarks** — that is
/// what §27's "permitting versioned generator changes" is for.
pub const GENERATOR_VERSION: u64 = 1;

/// Everything the panel writes. One [`LandmarkStore`] holds one of these.
#[derive(Clone, Debug, PartialEq)]
pub struct LandmarkSettings {
    /// Per-kind maximum. Keyed by [`LandmarkKindSpec::key`].
    pub caps: BTreeMap<String, u32>,
    /// Per-kind armed flag. Kept **separate from the cap** although the drawn
    /// control is one slider whose zero stop reads `off`
    /// (`LANDMARK_UI_DESIGN.md` §2.1): a user who switches a type off briefly
    /// gets their tuned number back. `ScatterRule` keeps `enabled` and
    /// `density` apart for the identical reason.
    pub armed: BTreeMap<String, bool>,
    /// `0..3`, `1.0` default. **Higher packs tighter** — the same sense
    /// `ScatterRule::density` already uses ("Above 1 packs tighter"). The
    /// class radius is *divided* by it.
    pub crowding: f64,
    /// Indexed by [`LandmarkClass::index`].
    pub class_radius_km: [f64; 4],
    /// `true`: one exclusion field over every type, so a dense Physical family
    /// genuinely crowds out the Religious one — §16's own intent. `false`:
    /// per-type fields, and the families stop interacting.
    pub cross_type_competition: bool,
}

impl Default for LandmarkSettings {
    /// Every kind at its [`LandmarkKindSpec::default_cap`]; **armed exactly
    /// when the kind is buildable**, since arming a type this engine does not
    /// generate would promise a run that cannot happen.
    fn default() -> Self {
        let mut caps = BTreeMap::new();
        let mut armed = BTreeMap::new();
        for k in kinds() {
            caps.insert(k.key.to_string(), k.default_cap);
            armed.insert(k.key.to_string(), k.buildable);
        }
        LandmarkSettings {
            caps,
            armed,
            crowding: 1.0,
            class_radius_km: DEFAULT_CLASS_RADIUS_KM,
            cross_type_competition: true,
        }
    }
}

impl LandmarkSettings {
    /// The cap in force for a key, falling back to the spec default when the
    /// map has no row (a settings blob saved before the key existed).
    pub fn cap(&self, key: &str) -> u32 {
        if let Some(v) = self.caps.get(key) {
            return *v;
        }
        kind_spec(key).map(|k| k.default_cap).unwrap_or(0)
    }

    /// Whether a key is armed, falling back to "armed iff buildable".
    pub fn is_armed(&self, key: &str) -> bool {
        if let Some(v) = self.armed.get(key) {
            return *v;
        }
        kind_spec(key).map(|k| k.buildable).unwrap_or(false)
    }

    pub fn set_cap(&mut self, key: &str, cap: u32) {
        self.caps.insert(key.to_string(), cap);
    }

    pub fn set_armed(&mut self, key: &str, armed: bool) {
        self.armed.insert(key.to_string(), armed);
    }

    /// [`crowding`](Self::crowding) exactly as [`radius_km`](Self::radius_km)
    /// applies it: clamped to `[0.05, 3.0]`, and `1.0` for a non-finite value.
    ///
    /// A literal zero would send the radius to infinity and take the whole map
    /// with it, and `NaN` from a mis-parsed settings blob would collapse the
    /// spacing bucket grid — the exact hazard reference v1.27 hardened
    /// `ScatterRule::spacing` against.
    ///
    /// Public, and one copy, because [`LandmarkReject::needs_crowding`] has to
    /// answer "what would this have to be" against the value that was actually
    /// in force. A second clamp there could disagree with this one, and a
    /// diagnostic that disagrees with the pass it is diagnosing is worse than
    /// none.
    pub fn crowding_in_force(&self) -> f64 {
        if self.crowding.is_finite() { self.crowding.clamp(0.05, 3.0) } else { 1.0 }
    }

    /// The exclusion radius in km for one class, after Crowding.
    pub fn radius_km(&self, class: LandmarkClass) -> f64 {
        let base = self.class_radius_km[class.index()];
        let base = if base.is_finite() && base > 0.0 { base } else { 0.0 };
        base / self.crowding_in_force()
    }
}

// ===========================================================================
// The result, and the store
// ===========================================================================

/// One run's output.
#[derive(Clone, Debug, PartialEq)]
pub struct LandmarkResult {
    pub landmarks: Vec<Landmark>,
    /// One entry per row of [`kinds`], **in table order**, so the panel can
    /// walk its rows and this list together. Kinds that were never run carry
    /// zeros and the reason.
    pub funnels: Vec<LandmarkFunnel>,
    /// The candidates this run offered and did not place, in the order the
    /// ranked walk met them — so within a kind, best-scoring first. Capped per
    /// kind at [`REJECT_LIST_MAX_PER_KIND`]; [`funnels`](Self::funnels) keeps
    /// the true totals. See [`LandmarkReject`] for why both exist.
    pub rejects: Vec<LandmarkReject>,
    /// Wall-clock seconds, for the `LAST RUN` note.
    pub seconds: f64,
}

impl LandmarkResult {
    pub fn funnel(&self, key: &str) -> Option<&LandmarkFunnel> {
        self.funnels.iter().find(|f| f.kind == key)
    }

    pub fn placed(&self, key: &str) -> usize {
        self.funnel(key).map(|f| f.placed).unwrap_or(0)
    }

    /// `(armed_count, placed_count)` for one family — the collapsed group
    /// header of `LANDMARK_UI_DESIGN.md` §3.2.
    pub fn family_summary(&self, family: LandmarkFamily, settings: &LandmarkSettings) -> (usize, usize) {
        let mut armed = 0usize;
        let mut placed = 0usize;
        for k in kinds().iter().filter(|k| k.family == family) {
            if settings.is_armed(k.key) && settings.cap(k.key) > 0 && k.buildable {
                armed += 1;
            }
            placed += self.placed(k.key);
        }
        (armed, placed)
    }
}

/// The store the shell holds: the settings the panel writes, and the last
/// run's result the panel reads.
///
/// It lives here rather than in `cartalith-godot` for the reason
/// `ARCHITECTURE.md` gives generally and `LANDMARK_UI_DESIGN.md` §9.2 gives
/// specifically: the panel must not own the vocabulary, or the 49 keys, ranges
/// and labels get hardcoded a second time on the GDScript side and drift.
#[derive(Clone, Debug, Default)]
pub struct LandmarkStore {
    pub settings: LandmarkSettings,
    /// `None` until the first run. The panel's second line reads `—` then, not
    /// `0 placed`: never having run and having placed nothing are different
    /// claims.
    pub last: Option<LandmarkResult>,
}

impl LandmarkStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the pass and retain the result.
    pub fn run(&mut self, inputs: &LandmarkInputs<'_>, world_seed: u64) -> &LandmarkResult {
        let r = generate(inputs, &self.settings, world_seed);
        self.last = Some(r);
        self.last.as_ref().expect("just assigned")
    }

    /// Clears the retained run. Call when the world changes underneath it, so
    /// the panel shows `—` rather than counts from a world that no longer
    /// exists.
    pub fn invalidate(&mut self) {
        self.last = None;
    }

    /// `caps total`, the first third of §4.4's headroom line.
    pub fn caps_total(&self) -> u64 {
        kinds()
            .iter()
            .filter(|k| self.settings.is_armed(k.key))
            .map(|k| self.settings.cap(k.key) as u64)
            .sum()
    }
}

// ===========================================================================
// Inputs
// ===========================================================================

/// One settlement, as much of it as this pass reads. Deliberately not the
/// engine's own settlement type: this module needs three numbers, and taking
/// the whole record would put a dependency on the settlement layer's shape
/// into the landmark layer for no gain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LandmarkSite {
    pub x: usize,
    pub y: usize,
    /// Used as `P_i` in §13's gravity term. Zero is legal and means "known to
    /// be there, weight nothing".
    pub population: f64,
}

/// Everything [`generate`] reads.
///
/// **Every field after the first six is optional and length-checked.** A slice
/// whose length is not `gw * gh` is treated as absent — it is a caller bug, and
/// the honest response to a caller bug at the gdext boundary is to degrade,
/// not to index past the end of it.
///
/// Build one with [`LandmarkInputs::new`] and then assign the fields you have:
///
/// ```ignore
/// let mut inp = LandmarkInputs::new(&field, gw, gh, sea, world, width_km);
/// inp.flow = Some(&flow);
/// inp.channel = Some(&ch.chan);
/// inp.recv = Some(&ch.recv);
/// ```
#[derive(Clone, Debug)]
pub struct LandmarkInputs<'a> {
    pub gw: usize,
    pub gh: usize,
    /// X wraps when true, matching every other field in this workspace.
    pub world: bool,
    /// The heightfield. Required; everything else degrades.
    pub field: &'a [f32],
    pub sea_level: f64,
    /// Real km across the map. `width_km / gw` is this workspace's only
    /// km-per-cell quotient, applied isotropically.
    pub width_km: f64,
    /// Metres at `field == 1.0`. `WorldParams::defaults` uses 4000.
    pub peak_m: f64,
    /// `compute_flow`'s accumulation.
    pub flow: Option<&'a [f32]>,
    /// `ChannelResult::chan` — non-zero where a channel runs.
    pub channel: Option<&'a [u8]>,
    /// `ChannelResult::recv` — the single-receiver tree. `-1` for no receiver.
    pub recv: Option<&'a [i32]>,
    /// `strahler_from_receivers`' order.
    pub order: Option<&'a [i16]>,
    /// `WaterBodies::classification` — `0` land, `1` ocean, `2` lake.
    pub water: Option<&'a [u8]>,
    /// `build_route_corridors` — the golden-verified pass/pinch-point field
    /// `DECISIONS.md` §7i measured. Mountain passes read this and nothing else
    /// can substitute for it.
    pub corridors: Option<&'a [f32]>,
    /// `build_lithology`'s categorical rock type, `0..=6` indexing
    /// `crate::LITH_NAMES`. Rock formation reads this for its causal fact
    /// string only — nothing scores on it, since a bare category says
    /// nothing about contrast with its neighbours.
    pub lithology: Option<&'a [u8]>,
    /// `WorldState::volcanic_field`, raw — not the `> 0.35` cut
    /// `build_lithology` reduces it to. Threaded through so a viewshed pass
    /// can reach it; Volcanic feature is blocked on that being absent, not
    /// on this (`kinds()`'s own `not_built` string for that kind).
    pub volcanism: Option<&'a [f32]>,
    /// `WorldState::resistance_field`, `[0,1]` — `compute_resistance`'s
    /// erosion resistance. Rock formation's whole domain is differential
    /// erosion, resistant rock standing proud of soft rock, and this is the
    /// "resistant" half of that test.
    pub resistance: Option<&'a [f32]>,
    /// Named resource-potential fields, `0..1`, keyed by `RESOURCE_KEYS`.
    /// Empty is legal and disarms Mine and Quarry.
    pub resources: &'a [(&'a str, &'a [f32])],
    /// Settlements, for §13's gravity term. Empty is legal: the term is then
    /// simply not part of the weighted sum, rather than contributing zero.
    pub settlements: &'a [LandmarkSite],
    /// **The routed way network** — `civ_consolidate_and_smooth_ways`' output,
    /// the same `Vec<Way>` the map draws. Empty is legal and disarms every
    /// kind that reads it.
    ///
    /// This is the graph §12 asks for, not a rasterised road mask, and the
    /// distinction is the whole reason five kinds were unbuildable until it
    /// was threaded in: a way carries its own classification
    /// ([`crate::WayType`], which `civ_classify_way` derives from **peak
    /// corridor usage** — §13's "transportation demand", already measured by
    /// the network pass) and its own endpoints, and a cell's junction-ness is
    /// how many distinct polylines pass through it rather than how many
    /// stroke pixels overlap.
    ///
    /// `Way::hidden` entries are skipped, matching every other consumer:
    /// a hidden way is not on the map and cannot put a landmark anywhere.
    /// `pts` are in whole-grid cell coordinates — see
    /// `civ_consolidate_and_smooth_ways`, which anchors a run's ends on
    /// `placement.x`/`.y` directly.
    pub ways: &'a [crate::Way],
}

/// `RESOURCE_KEYS` entries a Mine is generated from — the metallic and
/// gemstone potentials. **Category C**: which ores make a mine rather than a
/// quarry is a vocabulary decision, not a geological law.
pub const MINE_RESOURCES: [&str; 8] =
    ["copper", "tin", "iron", "gold", "silver", "lead", "gems", "salt"];

/// `RESOURCE_KEYS` entries a Quarry is generated from — the worked-stone and
/// earth potentials. **Category C**, same note as [`MINE_RESOURCES`].
pub const QUARRY_RESOURCES: [&str; 4] = ["buildstone", "clay", "flint", "obsidian"];

/// `RESOURCE_KEYS` entries Resource extraction site is generated from —
/// **the three potentials [`MINE_RESOURCES`] and [`QUARRY_RESOURCES`] do not
/// read at all**: `timber` (forestry), `sulfur` and `alum` (both computed by
/// `build_resource_potentials`, `cartalith-civ/src/lib.rs`, and already
/// carried this whole way — `RESOURCE_KEYS`' own doc calls the list "frozen/
/// append-only", and `landmark_resource_pairs` in `cartalith-godot/src/
/// lib.rs` zips all fifteen into `LandmarkInputs::resources` unconditionally,
/// not just the twelve Mine and Quarry claim).
///
/// **Why this kind is buildable and not a duplicate.** §29's own list gives
/// "Resource Extraction Site" no formula and no distinguishing test — unlike
/// Waterfall (§7) or Castle (§18), it is a bare name with no elaboration
/// anywhere in `LANDMARK_GENERATION_RESEARCH.md`. Read as "the same resources
/// Mine and Quarry already claim," it is indeed a pure duplicate and was
/// correctly left unbuilt for that reason. Read instead as **the residual
/// extraction economy neither of those two covers**, it claims zero cells
/// they do, using the identical, already-validated [`pool_resource`] test —
/// a local maximum of potential, above a floor, scored by potential/ground/
/// settlement access. That is a Category C naming decision, not a relaxed
/// rule: the underlying detector is the one Mine and Quarry already use.
pub const EXTRACTION_RESOURCES: [&str; 3] = ["timber", "sulfur", "alum"];

impl<'a> LandmarkInputs<'a> {
    /// The required half. Everything else defaults to absent.
    pub fn new(
        field: &'a [f32],
        gw: usize,
        gh: usize,
        sea_level: f64,
        world: bool,
        width_km: f64,
    ) -> Self {
        LandmarkInputs {
            gw,
            gh,
            world,
            field,
            sea_level,
            width_km,
            peak_m: 4000.0,
            flow: None,
            channel: None,
            recv: None,
            order: None,
            water: None,
            corridors: None,
            lithology: None,
            volcanism: None,
            resistance: None,
            resources: &[],
            settlements: &[],
            ways: &[],
        }
    }

    fn n(&self) -> usize {
        self.gw.saturating_mul(self.gh)
    }

    /// `Some` only if the slice is really `gw * gh` long. See the struct doc
    /// for why a wrong length degrades instead of panicking.
    fn grid<T>(&self, s: Option<&'a [T]>) -> Option<&'a [T]> {
        match s {
            Some(v) if v.len() == self.n() && self.n() > 0 => Some(v),
            _ => None,
        }
    }

    fn resource(&self, key: &str) -> Option<&'a [f32]> {
        self.resources
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
            .filter(|v| v.len() == self.n() && self.n() > 0)
    }

    /// Real km per grid cell.
    fn cell_km(&self) -> f64 {
        if self.gw == 0 || !(self.width_km > 0.0) {
            0.0
        } else {
            self.width_km / self.gw as f64
        }
    }

    /// `metersPerUnit()` — the same anchoring `FieldRefs::elevation_m` uses,
    /// so a drop reported here and a drop reported by the Measure tool are the
    /// same number.
    fn mpu(&self) -> f64 {
        let d = 1.0 - self.sea_level;
        self.peak_m / if d == 0.0 { 1e-6 } else { d }
    }

    fn elevation_m(&self, i: usize) -> f64 {
        (self.field[i] as f64 - self.sea_level) * self.mpu()
    }

    /// A field-unit height difference in metres.
    fn dh_m(&self, dh_field: f64) -> f64 {
        dh_field * self.mpu()
    }

    /// [`analysis::slope`]'s resolution-scaled output as a real gradient
    /// (metres of rise per metre of run), which is what a threshold on
    /// steepness has to be stated in if it is to mean the same thing at 512
    /// and at 8192.
    fn gradient(&self, slope_scaled: f32) -> f64 {
        let run_m = self.width_km * 1000.0;
        if !(run_m > 0.0) {
            return 0.0;
        }
        slope_scaled as f64 * self.mpu() / run_m
    }
}

// ===========================================================================
// §30 step 1 — the derived analytical fields
// ===========================================================================

/// **Category C.** The two analysis scales, in km rather than in cells, so a
/// threshold expressed against them means the same thing at 512 and at 8192.
/// §28 asks for exactly this ("the exact scales should depend on world
/// resolution").
const SCALE_FINE_KM: f64 = 3.0;
/// **Category C.** See [`SCALE_FINE_KM`].
const SCALE_BROAD_KM: f64 = 25.0;
/// **Category C, and a compute budget rather than a geographic claim.** Every
/// field below is separable and therefore `O(n · r)`; at the 8192² ceiling
/// `MEMORY_OPTIMIZATION_SCOPE.md` documents, an unclamped broad radius would be
/// 256 cells and the blur alone would dominate the pass. The consequence is
/// stated rather than hidden: above roughly 2 000 cells across, the broad scale
/// saturates and starts to mean a smaller real distance than
/// [`SCALE_BROAD_KM`] says.
const SCALE_MAX_CELLS: i64 = 40;
/// **Category C.** Below two cells a box window is the field itself and the
/// derived signal is per-cell noise — `analysis::tpi_multiscale` floors its own
/// fine radius at 2 for the same reason.
const SCALE_MIN_CELLS: i64 = 2;

/// Which derived fields the active kind set actually needs. Six `f32` rasters
/// is 1.5 GB at the documented 8192² ceiling, so the ones nobody asked for are
/// not built.
#[derive(Clone, Copy, Default)]
struct Needs {
    slope: bool,
    curv: bool,
    tpi: bool,
    extrema: bool,
    /// The [`WayGrid`], which is one pass over the polylines rather than over
    /// the grid — cheap, but still not built when nothing reads it.
    ways: bool,
}

impl Needs {
    fn of(key: &str) -> Needs {
        let (slope, curv, tpi, extrema, ways) = match key {
            "peak" => (false, false, true, true, false),
            "ridge" => (true, false, true, true, false),
            "cliff" => (true, true, false, true, false),
            "gorge" => (true, false, true, true, false),
            "rock_formation" => (false, false, true, true, false),
            "waterfall" => (true, false, false, true, false),
            "spring" => (false, false, false, true, false),
            "river_confluence" => (false, false, false, true, false),
            "lake" => (false, false, false, true, false),
            "mountain_pass" => (true, false, false, true, false),
            "ford" => (true, false, false, false, false),
            "harbour" => (true, false, false, false, false),
            "mine" | "quarry" | "resource_extraction_site" => (true, false, false, false, false),
            // Road junction reads the graph and nothing else — not even the
            // slope the other four want for their "workable ground" term.
            "road_junction" => (false, false, false, false, true),
            "bridge_site" | "market_site" | "caravan_station" | "trade_depot" => {
                (true, false, false, false, true)
            }
            _ => (false, false, false, false, false),
        };
        Needs { slope, curv, tpi, extrema, ways }
    }

    fn merge(self, o: Needs) -> Needs {
        Needs {
            slope: self.slope || o.slope,
            curv: self.curv || o.curv,
            tpi: self.tpi || o.tpi,
            extrema: self.extrema || o.extrema,
            ways: self.ways || o.ways,
        }
    }
}

/// The reusable analytical layers §3.1 asks for — "calculated once and reused"
/// (§34) rather than recomputed inside every detector.
///
/// **Category A throughout**: every value here comes from
/// [`cartalith_terrain::analysis`], whose own module header carries the
/// citation and the Category A notice. The one computation done locally is the
/// separable min/max pair, and it is done locally for a stated reason — see
/// [`sep_min_max`].
struct Derived {
    slope: Vec<f32>,
    curv: Vec<f32>,
    tpi_fine: Vec<f32>,
    tpi_broad: Vec<f32>,
    hmin: Vec<f32>,
    hmax: Vec<f32>,
    r_fine: i64,
    r_broad: i64,
}

impl Derived {
    fn build(inp: &LandmarkInputs<'_>, need: Needs) -> Derived {
        let (gw, gh) = (inp.gw, inp.gh);
        let cell_km = inp.cell_km();
        let to_cells = |km: f64| -> i64 {
            if !(cell_km > 0.0) {
                return SCALE_MIN_CELLS;
            }
            ((km / cell_km).round() as i64).clamp(SCALE_MIN_CELLS, SCALE_MAX_CELLS)
        };
        let r_fine = to_cells(SCALE_FINE_KM);
        let r_broad = to_cells(SCALE_BROAD_KM).max(r_fine);
        let (hmin, hmax) = if need.extrema {
            sep_min_max(inp.field, gw, gh, r_broad, inp.world)
        } else {
            (Vec::new(), Vec::new())
        };
        Derived {
            slope: if need.slope { analysis::slope(inp.field, gw, gh) } else { Vec::new() },
            // §5: curvature is evaluated after a blur, never on the raw field,
            // "so Cartalith does not interpret individual high-frequency DEM
            // noise as geological structure".
            curv: if need.curv {
                analysis::curvature_at(inp.field, gw, gh, r_fine, inp.world)
            } else {
                Vec::new()
            },
            tpi_fine: if need.tpi {
                analysis::tpi(inp.field, gw, gh, r_fine, inp.world)
            } else {
                Vec::new()
            },
            tpi_broad: if need.tpi {
                analysis::tpi(inp.field, gw, gh, r_broad, inp.world)
            } else {
                Vec::new()
            },
            hmin,
            hmax,
            r_fine,
            r_broad,
        }
    }

    fn slope(&self, i: usize) -> f32 {
        self.slope.get(i).copied().unwrap_or(0.0)
    }
    fn curv(&self, i: usize) -> f32 {
        self.curv.get(i).copied().unwrap_or(0.0)
    }
    fn tpi_fine(&self, i: usize) -> f32 {
        self.tpi_fine.get(i).copied().unwrap_or(0.0)
    }
    fn tpi_broad(&self, i: usize) -> f32 {
        self.tpi_broad.get(i).copied().unwrap_or(0.0)
    }
    /// `max − min` over the broad window — **identical to
    /// [`analysis::local_relief`] at the same radius**, and
    /// `relief_agrees_with_the_analysis_module` pins that.
    fn relief(&self, i: usize) -> f32 {
        match (self.hmax.get(i), self.hmin.get(i)) {
            (Some(hi), Some(lo)) if hi.is_finite() && lo.is_finite() => hi - lo,
            _ => 0.0,
        }
    }
}

/// The separable min and max of `field` over a `rad`-cell window — the two
/// halves [`analysis::local_relief`] subtracts.
///
/// Written here rather than called from there because this pass needs the two
/// **separately**: the maximum answers "is this cell a local summit" (the peak
/// detector's domain test) and the minimum answers "how far does it fall away"
/// (its prominence proxy). Calling `local_relief` as well would run the same
/// `O(n · r)` separable pass a second time for a difference this already has.
/// The window rule matches it exactly — X wraps when `world`, Y always clamps.
///
/// Both passes are parallel over output rows, added 2026-09-01 for
/// [`cartalith_terrain::analysis`]'s `box_h` reason and with its guarantee:
/// this is called five times per landmark run (once at `r_broad` from
/// `Derived::build`, then again at `r_fine` from `pool_ridge` and from each of
/// `pool_resource`'s three invocations) and was the single largest line in the
/// pass. Each output cell still takes its own min and max over its own window
/// in the same order; only whole rows go to different cores, so the output is
/// bit-identical and `relief_agrees_with_the_analysis_module` still pins it.
fn sep_min_max(field: &[f32], gw: usize, gh: usize, rad: i64, world: bool) -> (Vec<f32>, Vec<f32>) {
    let n = gw * gh;
    let mut hmin = vec![0f32; n];
    let mut hmax = vec![0f32; n];
    if n == 0 || field.len() != n {
        return (hmin, hmax);
    }
    let rad = rad.max(0);
    let (w, h) = (gw as i64, gh as i64);
    let mut rmin = vec![0f32; n];
    let mut rmax = vec![0f32; n];
    rmin.par_chunks_mut(gw).zip(rmax.par_chunks_mut(gw)).enumerate().for_each(
        |(y, (lo_row, hi_row))| {
            let row = y * gw;
            for x in 0..gw {
                let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
                for d in -rad..=rad {
                    let mut xx = x as i64 + d;
                    if world {
                        xx = xx.rem_euclid(w);
                    } else if xx < 0 || xx >= w {
                        continue;
                    }
                    let v = field[row + xx as usize];
                    if v < lo {
                        lo = v;
                    }
                    if v > hi {
                        hi = v;
                    }
                }
                lo_row[x] = lo;
                hi_row[x] = hi;
            }
        },
    );
    hmin.par_chunks_mut(gw).zip(hmax.par_chunks_mut(gw)).enumerate().for_each(
        |(y, (lo_row, hi_row))| {
            for x in 0..gw {
                let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
                for d in -rad..=rad {
                    let yy = y as i64 + d;
                    if yy < 0 || yy >= h {
                        continue;
                    }
                    let j = yy as usize * gw + x;
                    if rmin[j] < lo {
                        lo = rmin[j];
                    }
                    if rmax[j] > hi {
                        hi = rmax[j];
                    }
                }
                lo_row[x] = lo;
                hi_row[x] = hi;
            }
        },
    );
    (hmin, hmax)
}

// ===========================================================================
// §12 — the way network, as a graph rather than a mask
// ===========================================================================

/// A way's class as a `0..=3` load rank. **Not Category C**: `civ_classify_way`
/// already draws these four lines at peak corridor usage 8 / 5 / 3, and that
/// usage count is §13's "transportation demand" as the network pass itself
/// measured it. This function only orders what that one decided.
fn way_rank(t: crate::WayType) -> u8 {
    match t {
        crate::WayType::Track => 0,
        crate::WayType::Road => 1,
        crate::WayType::Regional => 2,
        crate::WayType::Highway => 3,
    }
}

/// [`way_rank`]'s inverse, for a causal chain. The four words are
/// `civ_classify_way`'s own class names, so a chain that says "a regional road"
/// names the class the network pass actually assigned rather than a second
/// vocabulary invented here.
fn rank_label(r: u8) -> &'static str {
    match r {
        3 => "highway",
        2 => "regional road",
        1 => "road",
        _ => "track",
    }
}

/// Per-cell facts read off [`LandmarkInputs::ways`], built once.
///
/// **Two arrays, and the first one is the point.** `ways_here` counts
/// *distinct polylines*, not stroke pixels — which is exactly what
/// `road_junction`'s own `not_built` reason demanded when it refused "a
/// junction at every crossing pixel". A way that doubles back through a cell
/// still counts once; two ways sharing a cell count twice.
struct WayGrid {
    /// Distinct ways through this cell. `0` means no way runs here, and is the
    /// presence test every way-reading detector uses for its domain.
    ways_here: Vec<u8>,
    /// [`way_rank`] of the busiest way on this cell. Meaningless where
    /// `ways_here` is `0`.
    rank: Vec<u8>,
}

impl WayGrid {
    /// Walks each polyline and stamps the cells it crosses.
    ///
    /// The polylines are **sparse** — `civ_smooth_path` emits a Catmull-Rom
    /// control set, and the road-consolidation golden's own case-1 ways are
    /// three points spanning a whole map — so a segment is stepped at
    /// [`WAY_STEP_CELLS`] rather than taken as two endpoints. The step is a
    /// sampling rate, not a threshold: any value below one cell visits every
    /// cell the segment crosses.
    fn build(ways: &[crate::Way], gw: usize, gh: usize, world: bool) -> WayGrid {
        let n = gw.saturating_mul(gh);
        let mut g = WayGrid { ways_here: vec![0u8; n], rank: vec![0u8; n] };
        if n == 0 {
            return g;
        }
        // One stamp per cell, so a way that revisits a cell is counted once.
        let mut stamp = vec![u32::MAX; n];
        for (wi, w) in ways.iter().enumerate() {
            if w.hidden || w.pts.len() < 2 {
                continue;
            }
            let rank = way_rank(w.way_type);
            let mut mark = |x: f64, y: f64| {
                if !x.is_finite() || !y.is_finite() {
                    return;
                }
                let yy = y.floor();
                if !(yy >= 0.0) || yy >= gh as f64 {
                    return;
                }
                let mut xx = x.floor() as i64;
                if world {
                    xx = xx.rem_euclid(gw as i64);
                } else if xx < 0 || xx >= gw as i64 {
                    return;
                }
                let i = yy as usize * gw + xx as usize;
                if stamp[i] != wi as u32 {
                    stamp[i] = wi as u32;
                    g.ways_here[i] = g.ways_here[i].saturating_add(1);
                }
                if rank > g.rank[i] {
                    g.rank[i] = rank;
                }
            };
            for pair in w.pts.windows(2) {
                let (ax, ay) = pair[0];
                let (bx, by) = pair[1];
                let mut dx = bx - ax;
                let dy = by - ay;
                // The same seam rule `Ctx::influence` uses: on a wrapping
                // world the short way round the cylinder is the real segment.
                if world {
                    let gwf = gw as f64;
                    if dx > gwf * 0.5 {
                        dx -= gwf;
                    } else if dx < -gwf * 0.5 {
                        dx += gwf;
                    }
                }
                let len = (dx * dx + dy * dy).sqrt();
                if !len.is_finite() {
                    continue;
                }
                let steps = ((len / WAY_STEP_CELLS).ceil() as i64).clamp(1, WAY_MAX_STEPS);
                for s in 0..=steps {
                    let t = s as f64 / steps as f64;
                    mark(ax + dx * t, ay + dy * t);
                }
            }
        }
        g
    }
}

/// **Category C, and a sampling rate rather than a claim.** Half a cell, so no
/// cell a segment passes through is skipped.
const WAY_STEP_CELLS: f64 = 0.5;
/// **A compute budget, not a geography.** A single corrupt or absurd control
/// point cannot turn one segment into an unbounded loop.
const WAY_MAX_STEPS: i64 = 100_000;

// ===========================================================================
// §30 step 8 — spatial competition
// ===========================================================================

/// Bucketed minimum-separation rejection over a suitability-sorted candidate
/// list.
///
/// **This is deliberately not Bridson (2007), and the reason is not
/// expedience.** `LANDMARK_UI_DESIGN.md` §9.3 makes the argument and this file
/// agrees with it: Bridson generates points *where none exist*, by dart-
/// throwing into empty space. Landmarks have a **fixed** candidate set — §30
/// generates candidates at step 5, scores them at step 7 and spaces them at
/// step 8 — so the correct algorithm is rejection over a sorted list, and
/// dart-throwing would be answering a different question.
///
/// The bucketing follows `cartalith-assets/src/placement.rs`'s shipped
/// `relief_fits`/`relief_take` pair, deliberately and in shape rather than by
/// dependency (this crate does not and should not depend on `cartalith-assets`
/// for a spacing test). Three things are carried across on purpose:
///
/// - the bucket edge is the **largest** separation any placed point can exert,
///   so a 3×3 bucket neighbourhood is sufficient to find every possible
///   conflict;
/// - the test is on squared distance, never a square root;
/// - the edge is hardened against a non-finite or zero separation, which is
///   what reference v1.27 fixed in `ScatterRule::spacing` after a NaN collapsed
///   the bucket grid to one cell.
///
/// One thing `placement.rs` does not need is a rule for what happens when two
/// candidates want *different* radii, which §16's `r = f(class, …)` forces.
///
/// **The rule is the candidate's own radius, and the alternative was tried and
/// measured before being rejected.** The obvious reading of §16 — "a major
/// landmark should also exert a competition/exclusion radius" — is
/// `sep = max(r_candidate, r_placed)`, so that a continental landmark keeps a
/// local one away and not merely the other way round. That was implemented
/// first. It makes every minor class vestigial, and not by a little:
///
/// > A set of landmarks packed at their own separation `r` sits at roughly
/// > `0.866·r²` of area per point, while each one's exclusion disc is `π·r²` —
/// > about 3.6× larger. So the moment a class is placed at anything near its
/// > own packing density, the **union of its discs covers the map**, and under
/// > a `max` rule every smaller-radius class after it is excluded everywhere.
///
/// Measured on this file's own fixture: with `max`, six Regional kinds filling
/// their caps left Cliff, Mine and Waterfall at **zero placed** with every
/// surviving candidate rejected on spacing. That is not §16's "prevents
/// procedural landmark saturation"; it is a class hierarchy that deletes three
/// of its four tiers.
///
/// So `r` is read as **how much room a landmark of this class needs**, which is
/// what §16's own ladder actually says ("Minor landmark → small exclusion
/// radius … World landmark → very large exclusion radius"). Majors are placed
/// first (see [`generate`]), so they still get their space; minors then fill in
/// between them at their own, smaller separation. The invariant this leaves is
/// `d(a, b) >= min(r_a, r_b)` for every placed pair, and
/// `a_placed_landmark_is_never_inside_its_own_exclusion_radius` asserts it.
///
/// One disclosure: `design/landmark-generation/Dock.dc.html`'s viewport legend
/// reads "rejected candidate — inside a placed one's ring", which is the `max`
/// reading. The artboard draws one highlighted type at a time, where the two
/// rules are identical; they differ only across classes, and there the measured
/// consequence above decides it.
struct Buckets {
    cell: f64,
    bw: usize,
    bh: usize,
    gw: f64,
    world: bool,
    b: Vec<Vec<(f64, f64)>>,
}

impl Buckets {
    fn new(gw: usize, gh: usize, world: bool, max_radius: f64) -> Buckets {
        let cell = if max_radius.is_finite() && max_radius > 1.0 { max_radius } else { 1.0 };
        let bw = ((gw as f64 / cell).ceil() as usize).max(1);
        let bh = ((gh as f64 / cell).ceil() as usize).max(1);
        Buckets { cell, bw, bh, gw: gw as f64, world, b: vec![Vec::new(); bw * bh] }
    }

    fn dx(&self, a: f64, b: f64) -> f64 {
        let mut d = a - b;
        if self.world && self.gw > 0.0 {
            if d > self.gw * 0.5 {
                d -= self.gw;
            } else if d < -self.gw * 0.5 {
                d += self.gw;
            }
        }
        d
    }

    fn fits(&self, x: f64, y: f64, r: f64) -> bool {
        let r = if r.is_finite() && r > 0.0 { r } else { 0.0 };
        let bx = (x / self.cell) as i64;
        let by = (y / self.cell) as i64;
        for dy in -1..=1i64 {
            for dx in -1..=1i64 {
                let ny = by + dy;
                if ny < 0 || ny as usize >= self.bh {
                    continue;
                }
                let mut nx = bx + dx;
                if self.world {
                    nx = nx.rem_euclid(self.bw as i64);
                } else if nx < 0 || nx as usize >= self.bw {
                    continue;
                }
                for &(qx, qy) in &self.b[ny as usize * self.bw + nx as usize] {
                    let ddx = self.dx(qx, x);
                    let ddy = qy - y;
                    if r > 0.0 && ddx * ddx + ddy * ddy < r * r {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// The squared distance, in cells, to the **nearest** placed point inside
    /// `r` — `None` when nothing is, which is the same answer [`fits`](Self::
    /// fits) gives as `true`.
    ///
    /// Called only for a candidate [`fits`](Self::fits) has already turned
    /// away, so it is off the hot path by construction: it exists to answer
    /// *how close* the blocker was, which is the whole of
    /// [`LandmarkReject::needs_crowding`]. `fits` keeps its early return
    /// precisely because this one cannot have it — "is there a conflict" can
    /// stop at the first, "which conflict is nearest" cannot.
    fn nearest_conflict_sq(&self, x: f64, y: f64, r: f64) -> Option<f64> {
        let r = if r.is_finite() && r > 0.0 { r } else { 0.0 };
        if r == 0.0 {
            return None;
        }
        let bx = (x / self.cell) as i64;
        let by = (y / self.cell) as i64;
        let mut best: Option<f64> = None;
        for dy in -1..=1i64 {
            for dx in -1..=1i64 {
                let ny = by + dy;
                if ny < 0 || ny as usize >= self.bh {
                    continue;
                }
                let mut nx = bx + dx;
                if self.world {
                    nx = nx.rem_euclid(self.bw as i64);
                } else if nx < 0 || nx as usize >= self.bw {
                    continue;
                }
                for &(qx, qy) in &self.b[ny as usize * self.bw + nx as usize] {
                    let ddx = self.dx(qx, x);
                    let ddy = qy - y;
                    let d2 = ddx * ddx + ddy * ddy;
                    // `!(d2 >= r * r)` rather than `d2 < r * r`: a NaN
                    // coordinate must be rejected here the way it is in
                    // `fits`, not silently accepted as "far away".
                    if !(d2 >= r * r) && !best.is_some_and(|b| b <= d2) {
                        best = Some(d2);
                    }
                }
            }
        }
        best
    }

    fn take(&mut self, x: f64, y: f64) {
        let bx = ((x / self.cell) as usize).min(self.bw - 1);
        let by = ((y / self.cell) as usize).min(self.bh - 1);
        self.b[by * self.bw + bx].push((x, y));
    }
}

// ===========================================================================
// Candidates and the suitability sum (§30 steps 5-7, §17)
// ===========================================================================

/// One surviving candidate: where it is, and the real measured facts that put
/// it there.
struct Cand {
    i: usize,
    x: usize,
    y: usize,
    /// §22's `causal_chain`, cause first. The type label is appended at
    /// placement, so this is the part derived from the fields.
    facts: Vec<String>,
}

/// A kind's candidate pool: the survivors, how many were lost to a hard
/// constraint before scoring, and the terms of its §17 sum.
struct Pool {
    cands: Vec<Cand>,
    rejected_constraint: usize,
    /// `(label, weight, raw value per candidate)`. **Every weight is
    /// Category C** — see the constants each detector uses.
    terms: Vec<(&'static str, f64, Vec<f32>)>,
}

impl Pool {
    fn new() -> Pool {
        Pool { cands: Vec::new(), rejected_constraint: 0, terms: Vec::new() }
    }
    /// How many cells entered the funnel.
    fn candidates(&self) -> usize {
        self.cands.len() + self.rejected_constraint
    }
}

/// [`analysis::normalise`] over a compact per-candidate vector, with one
/// substitution.
///
/// `normalise` answers "no variation here" with all zeros, which is the right
/// answer for a raster and the wrong one for a suitability term: a pool of one
/// candidate has no variation and is not therefore a pool of zero quality, and
/// a term nobody differs on discriminates nobody. A degenerate term is
/// substituted with the neutral `0.5` so it neither promotes nor demotes.
fn norm_term(raw: &[f32]) -> Vec<f32> {
    if raw.is_empty() {
        return Vec::new();
    }
    let out = analysis::normalise(raw, |_| true);
    if out.iter().all(|v| *v == 0.0) {
        return vec![0.5f32; raw.len()];
    }
    out
}

// ===========================================================================
// Formatting for the causal chain
// ===========================================================================

/// `12400` → `12 400`. The design's own worked example writes flow that way.
fn thousands(v: f64) -> String {
    let neg = v < 0.0;
    let s = format!("{}", v.abs().round() as i64);
    let mut out = String::new();
    let b = s.as_bytes();
    for (k, ch) in b.iter().enumerate() {
        if k > 0 && (b.len() - k) % 3 == 0 {
            out.push(' ');
        }
        out.push(*ch as char);
    }
    if neg {
        format!("-{}", out)
    } else {
        out
    }
}

fn fmt_m(v: f64) -> String {
    format!("{} m", thousands(v))
}

fn fmt_pct(v: f64) -> String {
    format!("{:.0} %", v * 100.0)
}

/// Only ever asked of a distance to a settlement that exists — every caller
/// is behind a non-empty-settlements test — so there is no infinity to handle.
fn fmt_km(v: f64) -> String {
    format!("{} km", thousands(v))
}

// ===========================================================================
// §27 — determinism
// ===========================================================================

/// §27's `seed_L = Hash(worldSeed, featureID, landmarkClass,
/// generationVersion)`.
///
/// A plain 64-bit mixer, not `cartalith_rng::Mulberry32` and not
/// `cartalith_noise::hash`: this pass draws no random numbers at all, so what
/// is wanted here is a stable *identity*, not a stream. Deriving it from the
/// cell index rather than from the placement order is what makes it survive a
/// cap change — move Waterfall from 40 to 41 and the forty that were already
/// there keep their seeds.
fn landmark_seed(world_seed: u64, feature: u64, class: LandmarkClass, version: u64) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in [world_seed, feature, class.index() as u64, version] {
        h ^= v;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
        h ^= h >> 33;
    }
    h
}

// ===========================================================================
// Category C — every threshold and every weight, in one block
// ===========================================================================
//
// Research §31 is explicit that "S_castle, S_sacred, Importance = f(...)" are
// **Cartalith synthesis**, not established science, and that the distinction
// must stay visible in source. Everything in this block is Category C. Nothing
// in it is measured, cited or derived; all of it is an engineering choice made
// so the pass produces a defensible landmark set on a real world, and all of it
// is meant to be recalibrated. The two exceptions that are NOT Category C are
// named where they are used: the channel-initiation threshold comes from
// `cartalith_hydrology::river_flow_thresh` (the engine's one canonical copy),
// and PEAK_MIN_PROMINENCE_M is deliberately the value the shipped Measure tool
// already calls prominent.

/// A waterfall's minimum drop from the cell to its downstream receiver.
const WATERFALL_MIN_DROP_M: f64 = 40.0;
/// A waterfall's minimum gradient, as rise over run. Grid-scale, not
/// waterfall-scale: a real fall is sub-cell, and what this asks is that the
/// *reach* be steep.
const WATERFALL_MIN_GRADIENT: f64 = 0.06;
/// A waterfall needs a river, not a rill: this many times the canonical
/// channel-initiation threshold.
const WATERFALL_MIN_FLOW_MULT: f64 = 2.0;
/// §7's four terms. `S = w_g·G + w_q·Q + w_c·C + w_r·R`, with two
/// substitutions stated rather than hidden: §7's `C` (channel confinement) is
/// read as local relief around the channel, and §7's `R` (geological
/// resistance / lithological contrast) is **dropped**, because a per-cell
/// contrast field does not exist in this engine and inventing one would be the
/// fabrication this module refuses everywhere else.
const WATERFALL_TERMS: [(&str, f64); 4] = [
    ("river gradient", 0.35),
    ("vertical drop", 0.30),
    ("flow magnitude", 0.20),
    ("channel confinement", 0.15),
];

/// A spring is a headwater, and a headwater on flat ground is a puddle.
const SPRING_MIN_RELIEF_M: f64 = 60.0;
const SPRING_TERMS: [(&str, f64); 3] =
    [("local relief", 0.40), ("emergence slope", 0.30), ("headwater flow", 0.30)];

/// The **second** largest tributary must itself be a real channel, or this is a
/// rill joining a river rather than two rivers meeting.
const CONFLUENCE_MIN_MINOR_FLOW_MULT: f64 = 1.0;
const CONFLUENCE_TERMS: [(&str, f64); 3] =
    [("combined flow", 0.45), ("stream order", 0.35), ("valley relief", 0.20)];

/// Below this a lake is a pond. Stated in km² so it means the same thing at
/// every resolution.
const LAKE_MIN_AREA_KM2: f64 = 25.0;
const LAKE_TERMS: [(&str, f64); 3] =
    [("surface area", 0.55), ("shore relief", 0.30), ("elevation", 0.15)];

/// `build_route_corridors` returns roughly `0..1`, and `DECISIONS.md` §7i
/// measured only 1.02 % of land above half strength on a real world. This
/// floor sits below that half-strength line on purpose: a pass is rare enough
/// already without asking for the rarest 1 %.
const PASS_MIN_CORRIDOR: f64 = 0.35;
/// §8: a pass needs "surrounding high terrain". Without a real barrier either
/// side this is a gap in a hill.
const PASS_MIN_FLANK_RELIEF_M: f64 = 200.0;
/// §8's `S_pass = w1·P + w2·C + w3·R + w4·A`. `P` is the corridor field, `C`
/// the flanking relief that makes it a connection rather than open ground, `R`
/// route accessibility (gentler is better), `A` accumulated transport demand —
/// the last read as settlement gravity, and simply absent from the sum when
/// there are no settlements.
const PASS_TERMS: [(&str, f64); 4] = [
    ("corridor strength", 0.40),
    ("flanking relief", 0.25),
    ("crossing ease", 0.20),
    ("settlement demand", 0.15),
];

/// **Not Category C.** `RIDGE_PROMINENCE_M` in
/// `cartalith-godot/src/measure_bridge.rs` is 100.0, and that is what this
/// project's shipped Measure tool already calls a prominent summit along a
/// drawn section. `LANDMARK_GENERATION_SCOPE.md` M4 asks the 2D generalisation
/// to agree with the 1D tool rather than silently invent a second meaning of
/// "prominent"; using the same number is how.
const PEAK_MIN_PROMINENCE_M: f64 = 100.0;
const PEAK_TERMS: [(&str, f64); 3] =
    [("prominence", 0.40), ("elevation", 0.30), ("topographic position", 0.30)];

/// A ridge crest must stand above its own surroundings, or it is a slope.
const RIDGE_MIN_TPI_M: f64 = 40.0;
const RIDGE_MIN_RELIEF_M: f64 = 100.0;
const RIDGE_TERMS: [(&str, f64); 3] =
    [("topographic position", 0.40), ("crest relief", 0.35), ("flank slope", 0.25)];

/// **Category C**, echoing `build_lithology`'s own inline `res_hard` cut
/// (`cartalith-civ/src/lib.rs`) at the same value: no basis exists to call a
/// cell "resistant" here by any different line than the one that already
/// decides granite/metamorphic there.
const ROCK_FORMATION_MIN_RESISTANCE: f64 = 0.55;
/// **Category C.** The neighbour-contrast this kind's own `not_built` reason
/// named as the missing half even with resistance wired in — uniformly hard
/// ground is not a formation, a hard patch inside soft ground is.
const ROCK_FORMATION_MIN_CONTRAST: f64 = 0.20;
/// **Category C**, smaller than [`RIDGE_MIN_TPI_M`]: an outcrop is a point
/// feature, not a crest line, and reads as "standing proud" at a smaller
/// rise than a whole ridge needs.
const ROCK_FORMATION_MIN_TPI_M: f64 = 30.0;
const ROCK_FORMATION_TERMS: [(&str, f64); 3] = [
    ("resistance contrast", 0.45),
    ("stands above its surroundings", 0.30),
    ("local relief", 0.25),
];

/// A face, not a hillside — and stated for the grid this engine actually runs
/// on rather than for a rock climber.
///
/// A cell here spans kilometres, so no cell-scale gradient is ever literally
/// vertical: a true cliff is a sub-cell feature and this port cannot resolve
/// one. `0.18` is a fall of nearly a kilometre in five, which is the steepest
/// thing a kilometre-scale DEM can represent, and what it detects is the
/// escarpment that contains the cliff rather than the cliff itself. Said out
/// loud because the alternative is a threshold that reads as physical and
/// silently never fires.
const CLIFF_MIN_GRADIENT: f64 = 0.18;
const CLIFF_MIN_RELIEF_M: f64 = 120.0;
const CLIFF_TERMS: [(&str, f64); 3] =
    [("face gradient", 0.45), ("relief across the face", 0.35), ("convex break", 0.20)];

/// A gorge sits *below* its surroundings — negative TPI is the whole test.
const GORGE_MAX_TPI_M: f64 = -50.0;
const GORGE_MIN_RELIEF_M: f64 = 200.0;
const GORGE_TERMS: [(&str, f64); 4] = [
    ("incision below the surroundings", 0.40),
    ("wall relief", 0.30),
    ("wall slope", 0.20),
    ("flow magnitude", 0.10),
];

/// Resource potentials are `0..1` after `build_resource_potentials`' scarcity
/// cut. This is the potential at which §14's "a valuable resource should
/// increase the probability of exploitation" becomes worth a landmark.
const MINE_MIN_POTENTIAL: f64 = 0.55;
const QUARRY_MIN_POTENTIAL: f64 = 0.55;
/// Same floor as [`MINE_MIN_POTENTIAL`]/[`QUARRY_MIN_POTENTIAL`], for the
/// same reason: no basis exists to call the residual resources rarer or
/// commoner than the ones the other two kinds already claim.
const EXTRACTION_MIN_POTENTIAL: f64 = 0.55;
/// §14's `P(L | R, S, C, T)`: `R` the resource value, `S`+`C` the settlement
/// structure and its connectivity (read as gravity, absent when there are no
/// settlements), `T` folded into `S`. "Workable ground" is the one term §14
/// does not name and this engine can measure: nobody sinks a shaft into a
/// cliff face.
const MINE_TERMS: [(&str, f64); 3] =
    [("ore potential", 0.55), ("workable ground", 0.20), ("settlement access", 0.25)];
const QUARRY_TERMS: [(&str, f64); 3] =
    [("stone potential", 0.55), ("exposed face", 0.20), ("settlement access", 0.25)];
/// [`EXTRACTION_RESOURCES`]' own §14 chain. `prefer_flat: true` at the call
/// site, same sense as Mine: a logging camp or a sulfur/alum pit wants ground
/// it can work, not a cliff.
const EXTRACTION_TERMS: [(&str, f64); 3] =
    [("extraction potential", 0.55), ("workable ground", 0.20), ("settlement access", 0.25)];

/// A shore you can build on.
const HARBOUR_MAX_GRADIENT: f64 = 0.06;
/// The fraction of the approach that must be land for the water to be
/// sheltered rather than open coast.
const HARBOUR_MIN_SHELTER: f64 = 0.45;
/// How far out "the approach" reaches, in km, capped in cells for the same
/// compute reason [`SCALE_MAX_CELLS`] exists.
const HARBOUR_SHELTER_KM: f64 = 8.0;
const HARBOUR_SHELTER_MAX_CELLS: i64 = 12;
const HARBOUR_TERMS: [(&str, f64); 4] = [
    ("shelter", 0.40),
    ("approach depth", 0.25),
    ("level shore", 0.15),
    ("settlement access", 0.20),
];

/// A ford is a river small enough to wade: at least a channel, at most this
/// many times the channel threshold.
const FORD_MIN_FLOW_MULT: f64 = 1.0;
const FORD_MAX_FLOW_MULT: f64 = 8.0;
const FORD_MAX_GRADIENT: f64 = 0.08;
const FORD_TERMS: [(&str, f64); 4] = [
    ("shallow crossing", 0.35),
    ("gentle banks", 0.30),
    ("corridor value", 0.20),
    ("route demand", 0.15),
];

/// Two or more distinct **visible** way polylines on one cell.
///
/// **Category C in the number only**: that a junction is a graph property
/// rather than a raster crossing is [`WayGrid`]'s design. The number is `2`,
/// and it was `3` first — the argument for `3` was that "two is a corridor two
/// strokes happen to share", and that argument is wrong for the output this
/// pass actually reads, which is worth stating rather than quietly fixing.
///
/// `civ_consolidate_and_smooth_ways` claims cells **busiest-corridor-first**
/// and emits only each edge's unclaimed sub-runs, plus one already-claimed
/// connector cell at each cut. An edge whose whole path a busier one already
/// claimed emits a two-point way flagged `hidden`, which [`WayGrid`] skips
/// like every other consumer. So two *visible* strokes never merely share a
/// corridor: they touch only at a cut, and a cut is a branch leaving or
/// rejoining the trunk — a junction. Measured, on a world
/// `cartalith_engine::generate_terrain` really made (15 settlements, 31 ways
/// of which 15 hidden): 447 cells carry a way, **13** carry two and **2**
/// carry three. At `3` this kind placed two landmarks against a cap of twenty,
/// which is the "flag flipped, nothing generated" failure the funnel exists to
/// expose.
const JUNCTION_MIN_WAYS: usize = 2;
const JUNCTION_TERMS: [(&str, f64); 3] =
    [("roads meeting", 0.45), ("busiest road", 0.30), ("settlement access", 0.25)];

/// **Category C.** [`way_rank`] a way must reach before a market or a caravan
/// station is worth siting on it: `1` is `WayType::Road`, `civ_classify_way`'s
/// own peak-usage-3 line. A track carries no trade.
const TRADE_MIN_WAY_RANK: u8 = 1;

/// **Category C.** A market serves a catchment it sits inside: this is how far
/// from the nearest settlement a cell on a trade road can be and still be that
/// settlement's market rather than open road.
const MARKET_MAX_SETTLEMENT_KM: f64 = 40.0;
const MARKET_TERMS: [(&str, f64); 3] =
    [("settlement access", 0.45), ("road class", 0.30), ("workable ground", 0.25)];

/// **Category C, and deliberately more than [`MARKET_MAX_SETTLEMENT_KM`].**
/// §14's chain is `trade route → caravan station → market town`, so the two
/// are the same measurement with opposite senses: a market is inside a
/// settlement's reach, a station is in the gap between them. The 20 km band
/// between the two is neither, and is left empty on purpose — road that needs
/// no waystation and serves no market.
const CARAVAN_MIN_SETTLEMENT_KM: f64 = 60.0;
const CARAVAN_TERMS: [(&str, f64); 3] =
    [("distance from any settlement", 0.40), ("road class", 0.35), ("workable ground", 0.25)];

/// **Category C.** How many times the canonical channel-initiation threshold a
/// river must carry before a boat can work it, and therefore before a road
/// reaching it is a transhipment point rather than a bank. Well above
/// [`FORD_MAX_FLOW_MULT`]: water you can wade is not water you can ship on.
const DEPOT_NAVIGABLE_FLOW_MULT: f64 = 20.0;
const DEPOT_TERMS: [(&str, f64); 3] =
    [("road class", 0.40), ("settlement access", 0.35), ("workable ground", 0.25)];

/// §12's own worked example: "a bridge located at a major river bottleneck can
/// score highly because it reduces travel cost between otherwise separated
/// regions". The bottleneck test needs no new constant — the boundary is
/// [`FORD_MAX_FLOW_MULT`], read from the other side. Below it the road wades
/// across and the landmark is a Ford; above it the road needs a bridge, and
/// the two kinds partition the same channel by the one number that separates
/// them rather than double-counting the cells.
const BRIDGE_TERMS: [(&str, f64); 4] = [
    ("flow to be spanned", 0.35),
    ("road class", 0.30),
    ("abutment ground", 0.20),
    ("settlement access", 0.15),
];

/// §13's distance-decay exponent `β`.
const GRAVITY_BETA: f64 = 1.5;

/// §24's importance model. "A feature becomes important because something
/// makes it important" — so the terms are the physical case the pass already
/// made for it, the class it was graded into, and how much human activity is
/// within reach of it. **The viewshed term §18 and §19 both weight most
/// heavily is missing**, and every importance below is therefore computed on a
/// model short of its largest human-significance term; the panel discloses
/// that per row rather than in a footnote.
const IMPORTANCE_TERMS: [(&str, f64); 3] =
    [("suitability", 0.45), ("class", 0.25), ("settlement reach", 0.30)];

/// The raw `class` term of [`IMPORTANCE_TERMS`], before normalisation.
fn class_weight(c: LandmarkClass) -> f32 {
    match c {
        LandmarkClass::Continental => 1.00,
        LandmarkClass::Regional => 0.70,
        LandmarkClass::Cultural => 0.50,
        LandmarkClass::Local => 0.40,
    }
}

// ===========================================================================
// The working context
// ===========================================================================

/// Per-cell upstream structure read off the receiver tree — the confluence
/// entity `LANDMARK_GENERATION_SCOPE.md` M2 calls "the one real new piece",
/// since the reference leaves confluences implicit in `recv`.
struct Upstream {
    count: Vec<u16>,
    /// Largest and second-largest upstream channel flow entering this cell.
    max1: Vec<f32>,
    max2: Vec<f32>,
}

struct Ctx<'a> {
    inp: &'a LandmarkInputs<'a>,
    d: Derived,
    flow: Option<&'a [f32]>,
    chan: Option<&'a [u8]>,
    recv: Option<&'a [i32]>,
    order: Option<&'a [i16]>,
    water: Option<&'a [u8]>,
    corridors: Option<&'a [f32]>,
    lithology: Option<&'a [u8]>,
    resistance: Option<&'a [f32]>,
    up: Option<Upstream>,
    /// `None` when nothing armed reads the way network, or when the caller
    /// passed no ways at all — the same "absent input, honest `NoTerrain`"
    /// contract every optional raster here has.
    ways: Option<WayGrid>,
    flow_thresh: f64,
    cell_km: f64,
    n: usize,
}

impl<'a> Ctx<'a> {
    fn build(inp: &'a LandmarkInputs<'a>, need: Needs) -> Ctx<'a> {
        let n = inp.n();
        let flow = inp.grid(inp.flow);
        let chan = inp.grid(inp.channel);
        let recv = inp.grid(inp.recv);
        let up = match (chan, recv, flow) {
            (Some(ch), Some(rv), Some(fl)) => {
                let mut u = Upstream {
                    count: vec![0u16; n],
                    max1: vec![0f32; n],
                    max2: vec![0f32; n],
                };
                for i in 0..n {
                    if ch[i] == 0 {
                        continue;
                    }
                    let r = rv[i];
                    if r < 0 || r as usize >= n {
                        continue;
                    }
                    let j = r as usize;
                    u.count[j] = u.count[j].saturating_add(1);
                    let f = fl[i];
                    if f > u.max1[j] {
                        u.max2[j] = u.max1[j];
                        u.max1[j] = f;
                    } else if f > u.max2[j] {
                        u.max2[j] = f;
                    }
                }
                Some(u)
            }
            _ => None,
        };
        // The canonical channel-initiation threshold, not a local invention.
        // `world_gw` is `gw` here: this pass runs on the world's own grid, not
        // on an LOD tile.
        let flow_thresh = if inp.width_km > 0.0 && inp.gw > 0 && inp.gh > 0 {
            cartalith_hydrology::river_flow_thresh(inp.gw, inp.gh, inp.gw, inp.width_km)
        } else {
            (inp.gw * inp.gh) as f64 * 0.0004
        };
        Ctx {
            inp,
            d: Derived::build(inp, need),
            flow,
            chan,
            recv,
            order: inp.grid(inp.order),
            water: inp.grid(inp.water),
            corridors: inp.grid(inp.corridors),
            lithology: inp.grid(inp.lithology),
            resistance: inp.grid(inp.resistance),
            up,
            ways: if need.ways && !inp.ways.is_empty() {
                Some(WayGrid::build(inp.ways, inp.gw, inp.gh, inp.world))
            } else {
                None
            },
            flow_thresh: if flow_thresh.is_finite() && flow_thresh > 0.0 { flow_thresh } else { 1.0 },
            cell_km: inp.cell_km(),
            n,
        }
    }

    /// The water mask decides when it is present, because it knows about
    /// above-sea lakes the bare height test cannot see.
    fn is_land(&self, i: usize) -> bool {
        match self.water {
            Some(w) => w[i] == 0,
            None => self.inp.field[i] as f64 >= self.inp.sea_level,
        }
    }

    fn is_ocean(&self, i: usize) -> bool {
        match self.water {
            Some(w) => w[i] == 1,
            None => (self.inp.field[i] as f64) < self.inp.sea_level,
        }
    }

    fn nb(&self, x: usize, y: usize, dx: i64, dy: i64) -> Option<usize> {
        let (gw, gh) = (self.inp.gw as i64, self.inp.gh as i64);
        let yy = y as i64 + dy;
        if yy < 0 || yy >= gh {
            return None;
        }
        let mut xx = x as i64 + dx;
        if self.inp.world {
            xx = xx.rem_euclid(gw);
        } else if xx < 0 || xx >= gw {
            return None;
        }
        Some(yy as usize * self.inp.gw + xx as usize)
    }

    /// §13's `I(x) = Σ P_i / d_c(x, i)^β` — **Category B**, and with one
    /// simplification stated rather than glossed: `d_c` is straight-line
    /// distance in km, not least-cost distance. §12 is right that 10 km across
    /// a range is not 10 km along a valley, and the least-cost machinery does
    /// exist in this crate (`civ_dijkstra_path`, `WayRouter`) — but running a
    /// Dijkstra per candidate is not affordable inside a button-driven pass,
    /// and the honest description of what this returns is "how much settled
    /// weight is nearby", not "how reachable this is".
    fn influence(&self, x: usize, y: usize) -> f64 {
        if self.inp.settlements.is_empty() {
            return 0.0;
        }
        let cell = if self.cell_km > 0.0 { self.cell_km } else { 1.0 };
        let mut acc = 0f64;
        for s in self.inp.settlements {
            let mut dx = s.x as f64 - x as f64;
            if self.inp.world {
                let gw = self.inp.gw as f64;
                if dx > gw * 0.5 {
                    dx -= gw;
                } else if dx < -gw * 0.5 {
                    dx += gw;
                }
            }
            let dy = s.y as f64 - y as f64;
            let d_km = ((dx * dx + dy * dy).sqrt() * cell).max(cell);
            let p = if s.population.is_finite() && s.population > 0.0 { s.population } else { 0.0 };
            acc += p / d_km.powf(GRAVITY_BETA);
        }
        acc
    }

    /// Straight-line km to the nearest settlement, `f64::INFINITY` when there
    /// are none.
    ///
    /// **Deliberately straight-line, and only ever asked of a cell that is
    /// already on a way.** The objection `market_site`'s own `not_built`
    /// reason raised — that straight-line proximity "would put markets on the
    /// wrong side of mountains" — is answered by the domain, not by the
    /// metric: a cell on a routed way is by construction on a least-cost path
    /// the network pass already laid down, so there is no mountain between it
    /// and the network. Running a Dijkstra per candidate to refine the last
    /// few km is the cost `Ctx::influence` already declines for the same
    /// reason.
    fn nearest_settlement_km(&self, x: usize, y: usize) -> f64 {
        let cell = if self.cell_km > 0.0 { self.cell_km } else { 1.0 };
        let mut best = f64::INFINITY;
        for s in self.inp.settlements {
            let mut dx = s.x as f64 - x as f64;
            if self.inp.world {
                let gw = self.inp.gw as f64;
                if dx > gw * 0.5 {
                    dx -= gw;
                } else if dx < -gw * 0.5 {
                    dx += gw;
                }
            }
            let dy = s.y as f64 - y as f64;
            let d = (dx * dx + dy * dy).sqrt() * cell;
            if d < best {
                best = d;
            }
        }
        best
    }

    fn elev_m(&self, i: usize) -> f64 {
        self.inp.elevation_m(i)
    }

    fn relief_m(&self, i: usize) -> f64 {
        self.inp.dh_m(self.d.relief(i) as f64)
    }

    fn gradient(&self, i: usize) -> f64 {
        self.inp.gradient(self.d.slope(i))
    }
}

// ===========================================================================
// §30 steps 5-6 — candidates and their hard constraints, one detector per kind
// ===========================================================================
//
// Every detector below returns `None` when an input it genuinely needs is
// absent. `None` becomes `LandmarkLimit::NoTerrain` with `candidates: 0`, and
// never a fabricated placement. Every detector's *domain* is deliberately a
// sparse, principled subset of the grid rather than "every land cell": a funnel
// that reports six million candidates has told the user nothing, and a sort
// over six million has cost the pass its interactivity.

/// §7's constraint block, verbatim: `river = true AND gradient > threshold AND
/// vertical drop > threshold AND flow accumulation > minimum`.
fn pool_waterfall(c: &Ctx<'_>) -> Option<Pool> {
    let chan = c.chan?;
    let flow = c.flow?;
    let recv = c.recv?;
    let mut p = Pool::new();
    let (mut t_g, mut t_d, mut t_q, mut t_c) = (vec![], vec![], vec![], vec![]);
    let min_flow = c.flow_thresh * WATERFALL_MIN_FLOW_MULT;
    for i in 0..c.n {
        if chan[i] == 0 || !c.is_land(i) {
            continue;
        }
        let r = recv[i];
        let drop_m = if r >= 0 && (r as usize) < c.n {
            c.inp.dh_m(c.inp.field[i] as f64 - c.inp.field[r as usize] as f64)
        } else {
            0.0
        };
        let grad = c.gradient(i);
        let q = flow[i] as f64;
        if drop_m < WATERFALL_MIN_DROP_M
            || grad < WATERFALL_MIN_GRADIENT
            || !(q >= min_flow)
        {
            p.rejected_constraint += 1;
            continue;
        }
        let relief = c.relief_m(i);
        let (x, y) = (i % c.inp.gw, i / c.inp.gw);
        p.cands.push(Cand {
            i,
            x,
            y,
            facts: vec![
                format!("high local relief {}", fmt_m(relief)),
                format!("steep river gradient {}", fmt_pct(grad)),
                format!("vertical drop {}", fmt_m(drop_m)),
                format!("flow {} cells", thousands(q)),
            ],
        });
        t_g.push(grad as f32);
        t_d.push(drop_m as f32);
        t_q.push(q as f32);
        t_c.push(relief as f32);
    }
    p.terms = vec![
        (WATERFALL_TERMS[0].0, WATERFALL_TERMS[0].1, t_g),
        (WATERFALL_TERMS[1].0, WATERFALL_TERMS[1].1, t_d),
        (WATERFALL_TERMS[2].0, WATERFALL_TERMS[2].1, t_q),
        (WATERFALL_TERMS[3].0, WATERFALL_TERMS[3].1, t_c),
    ];
    Some(p)
}

/// A spring is where a channel *starts*: a channel cell nothing upstream feeds.
fn pool_spring(c: &Ctx<'_>) -> Option<Pool> {
    let chan = c.chan?;
    let flow = c.flow?;
    let up = c.up.as_ref()?;
    let mut p = Pool::new();
    let (mut t_r, mut t_s, mut t_q) = (vec![], vec![], vec![]);
    for i in 0..c.n {
        if chan[i] == 0 || !c.is_land(i) || up.count[i] != 0 {
            continue;
        }
        let relief = c.relief_m(i);
        if relief < SPRING_MIN_RELIEF_M {
            p.rejected_constraint += 1;
            continue;
        }
        let (x, y) = (i % c.inp.gw, i / c.inp.gw);
        let q = flow[i] as f64;
        p.cands.push(Cand {
            i,
            x,
            y,
            facts: vec![
                "the head of a channel, with nothing upstream of it".to_string(),
                format!("local relief {}", fmt_m(relief)),
                format!("emerges at {}", fmt_m(c.elev_m(i))),
            ],
        });
        t_r.push(relief as f32);
        t_s.push(c.gradient(i) as f32);
        t_q.push(q as f32);
    }
    p.terms = vec![
        (SPRING_TERMS[0].0, SPRING_TERMS[0].1, t_r),
        (SPRING_TERMS[1].0, SPRING_TERMS[1].1, t_s),
        (SPRING_TERMS[2].0, SPRING_TERMS[2].1, t_q),
    ];
    Some(p)
}

/// Two channels meeting — extracted as a first-class list from the receiver
/// tree, which is where the reference leaves it implicit.
fn pool_confluence(c: &Ctx<'_>) -> Option<Pool> {
    let chan = c.chan?;
    let flow = c.flow?;
    let up = c.up.as_ref()?;
    let mut p = Pool::new();
    let (mut t_q, mut t_o, mut t_r) = (vec![], vec![], vec![]);
    let min_minor = c.flow_thresh * CONFLUENCE_MIN_MINOR_FLOW_MULT;
    for i in 0..c.n {
        if chan[i] == 0 || !c.is_land(i) || up.count[i] < 2 {
            continue;
        }
        let minor = up.max2[i] as f64;
        if !(minor >= min_minor) {
            p.rejected_constraint += 1;
            continue;
        }
        let (x, y) = (i % c.inp.gw, i / c.inp.gw);
        let q = flow[i] as f64;
        let ord = c.order.map(|o| o[i] as f64).unwrap_or(0.0);
        let relief = c.relief_m(i);
        let mut facts = vec![
            format!("{} channels meet here", up.count[i]),
            format!("minor branch carries {} cells", thousands(minor)),
            format!("combined flow {} cells", thousands(q)),
        ];
        if c.order.is_some() {
            facts.push(format!("stream order {}", ord as i64));
        }
        p.cands.push(Cand { i, x, y, facts });
        t_q.push(q as f32);
        t_o.push(ord as f32);
        t_r.push(relief as f32);
    }
    p.terms = vec![
        (CONFLUENCE_TERMS[0].0, CONFLUENCE_TERMS[0].1, t_q),
        (CONFLUENCE_TERMS[2].0, CONFLUENCE_TERMS[2].1, t_r),
    ];
    // The order term joins the sum only when the caller supplied a Strahler
    // field — an absent input renormalises the weights, it does not contribute
    // zero.
    if c.order.is_some() {
        p.terms.insert(1, (CONFLUENCE_TERMS[1].0, CONFLUENCE_TERMS[1].1, t_o));
    }
    Some(p)
}

/// One candidate per connected lake body, at its deepest cell — not one per
/// water cell, which would make a single lake into four hundred landmarks.
fn pool_lake(c: &Ctx<'_>) -> Option<Pool> {
    let water = c.water?;
    let gw = c.inp.gw;
    let mut p = Pool::new();
    let (mut t_a, mut t_r, mut t_e) = (vec![], vec![], vec![]);
    let cell_area = c.cell_km * c.cell_km;
    let mut seen = vec![false; c.n];
    let mut stack: Vec<usize> = Vec::new();
    for s in 0..c.n {
        if seen[s] || water[s] != 2 {
            continue;
        }
        seen[s] = true;
        stack.clear();
        stack.push(s);
        let mut area = 0usize;
        let mut low = s;
        while let Some(i) = stack.pop() {
            area += 1;
            if c.inp.field[i] < c.inp.field[low] {
                low = i;
            }
            let (x, y) = (i % gw, i / gw);
            for (dx, dy) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
                if let Some(j) = c.nb(x, y, dx, dy).filter(|j| !seen[*j] && water[*j] == 2) {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        let area_km2 = area as f64 * cell_area;
        if area_km2 < LAKE_MIN_AREA_KM2 {
            p.rejected_constraint += 1;
            continue;
        }
        let relief = c.relief_m(low);
        let (x, y) = (low % gw, low / gw);
        p.cands.push(Cand {
            i: low,
            x,
            y,
            facts: vec![
                "a closed basin holding standing water".to_string(),
                format!("surface {} km2", thousands(area_km2)),
                format!("shore relief {}", fmt_m(relief)),
            ],
        });
        t_a.push(area_km2 as f32);
        t_r.push(relief as f32);
        t_e.push(c.elev_m(low) as f32);
    }
    p.terms = vec![
        (LAKE_TERMS[0].0, LAKE_TERMS[0].1, t_a),
        (LAKE_TERMS[1].0, LAKE_TERMS[1].1, t_r),
        (LAKE_TERMS[2].0, LAKE_TERMS[2].1, t_e),
    ];
    Some(p)
}

/// §8, reusing the field `DECISIONS.md` §7i already measured on a real world
/// rather than reimplementing a saddle test that was measured and found
/// wanting.
fn pool_pass(c: &Ctx<'_>) -> Option<Pool> {
    let cor = c.corridors?;
    let gw = c.inp.gw;
    let (_, cmax) = sep_min_max(cor, gw, c.inp.gh, c.d.r_broad, c.inp.world);
    let mut p = Pool::new();
    let (mut t_c, mut t_f, mut t_e, mut t_s) = (vec![], vec![], vec![], vec![]);
    let has_settle = !c.inp.settlements.is_empty();
    for i in 0..c.n {
        let v = cor[i] as f64;
        // Domain: a local maximum of the corridor field. One pinch point, one
        // candidate.
        if !(v > 0.0) || !c.is_land(i) || cor[i] < cmax[i] {
            continue;
        }
        let relief = c.relief_m(i);
        if v < PASS_MIN_CORRIDOR || relief < PASS_MIN_FLANK_RELIEF_M {
            p.rejected_constraint += 1;
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        let grad = c.gradient(i);
        p.cands.push(Cand {
            i,
            x,
            y,
            facts: vec![
                "a barrier on both sides of the axis, and a way through".to_string(),
                format!("corridor strength {:.2}", v),
                format!("flanking relief {}", fmt_m(relief)),
                format!("crossed at {}", fmt_m(c.elev_m(i))),
            ],
        });
        t_c.push(v as f32);
        t_f.push(relief as f32);
        t_e.push(-grad as f32);
        t_s.push(c.influence(x, y) as f32);
    }
    p.terms = vec![
        (PASS_TERMS[0].0, PASS_TERMS[0].1, t_c),
        (PASS_TERMS[1].0, PASS_TERMS[1].1, t_f),
        (PASS_TERMS[2].0, PASS_TERMS[2].1, t_e),
    ];
    if has_settle {
        p.terms.push((PASS_TERMS[3].0, PASS_TERMS[3].1, t_s));
    }
    Some(p)
}

/// Local maxima with a radius-bounded prominence, the 2D generalisation of the
/// Measure tool's own 1D prominence walk.
fn pool_peak(c: &Ctx<'_>) -> Option<Pool> {
    if c.d.hmax.is_empty() {
        return None;
    }
    let gw = c.inp.gw;
    let mut p = Pool::new();
    let (mut t_p, mut t_e, mut t_t) = (vec![], vec![], vec![]);
    for i in 0..c.n {
        if !c.is_land(i) || c.inp.field[i] < c.d.hmax[i] {
            continue;
        }
        // The drop from this summit to the lowest ground within the broad
        // window. A radius-bounded proxy for true prominence, which would need
        // a global key-saddle walk; the bound is the same shape of bound
        // `measure_bridge.rs` already accepts by walking only along one drawn
        // section.
        let prom = c.inp.dh_m((c.inp.field[i] - c.d.hmin[i]) as f64);
        if prom < PEAK_MIN_PROMINENCE_M {
            p.rejected_constraint += 1;
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        let elev = c.elev_m(i);
        let tpi = c.inp.dh_m(c.d.tpi_broad(i) as f64);
        p.cands.push(Cand {
            i,
            x,
            y,
            facts: vec![
                "a local summit — no ground within reach is higher".to_string(),
                format!("prominence {}", fmt_m(prom)),
                format!("stands {} above its surroundings", fmt_m(tpi)),
                format!("{} above sea level", fmt_m(elev)),
            ],
        });
        t_p.push(prom as f32);
        t_e.push(elev as f32);
        t_t.push(tpi as f32);
    }
    p.terms = vec![
        (PEAK_TERMS[0].0, PEAK_TERMS[0].1, t_p),
        (PEAK_TERMS[1].0, PEAK_TERMS[1].1, t_e),
        (PEAK_TERMS[2].0, PEAK_TERMS[2].1, t_t),
    ];
    Some(p)
}

/// A crest, not a summit: high along one axis and not the other.
fn pool_ridge(c: &Ctx<'_>) -> Option<Pool> {
    if c.d.tpi_fine.is_empty() {
        return None;
    }
    let gw = c.inp.gw;
    let (_, tmax) = sep_min_max(&c.d.tpi_fine, gw, c.inp.gh, c.d.r_fine, c.inp.world);
    let mut p = Pool::new();
    let (mut t_t, mut t_r, mut t_s) = (vec![], vec![], vec![]);
    for (i, &tm) in tmax.iter().enumerate() {
        if !c.is_land(i) {
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        let h = c.inp.field[i];
        let ax = match (c.nb(x, y, -1, 0), c.nb(x, y, 1, 0)) {
            (Some(l), Some(r)) => h >= c.inp.field[l] && h >= c.inp.field[r],
            _ => false,
        };
        let ay = match (c.nb(x, y, 0, -1), c.nb(x, y, 0, 1)) {
            (Some(u), Some(d)) => h >= c.inp.field[u] && h >= c.inp.field[d],
            _ => false,
        };
        // Exactly one axis: both is a summit (Peak's domain), neither is a
        // slope. The XOR is what makes this a line rather than a point.
        if ax == ay || c.d.tpi_fine[i] < tm {
            continue;
        }
        let tpi = c.inp.dh_m(c.d.tpi_fine(i) as f64);
        let relief = c.relief_m(i);
        if tpi < RIDGE_MIN_TPI_M || relief < RIDGE_MIN_RELIEF_M {
            p.rejected_constraint += 1;
            continue;
        }
        p.cands.push(Cand {
            i,
            x,
            y,
            facts: vec![
                "a crest along one axis and a slope across the other".to_string(),
                format!("stands {} above its surroundings", fmt_m(tpi)),
                format!("local relief {}", fmt_m(relief)),
            ],
        });
        t_t.push(tpi as f32);
        t_r.push(relief as f32);
        t_s.push(c.gradient(i) as f32);
    }
    p.terms = vec![
        (RIDGE_TERMS[0].0, RIDGE_TERMS[0].1, t_t),
        (RIDGE_TERMS[1].0, RIDGE_TERMS[1].1, t_r),
        (RIDGE_TERMS[2].0, RIDGE_TERMS[2].1, t_s),
    ];
    Some(p)
}

/// **Category C** — §29 gives Rock formation a name and no formula, the same
/// position `resource_extraction_site` was in (see that kind's own doc
/// comment). The domain is the phrase this kind's own `not_built` reason used
/// before this pass closed it: **resistant rock standing proud of soft
/// rock**. Both halves are load-bearing constraints, not one scoring the
/// other:
///
/// - **resistant**: this cell's own [`Ctx::resistance`] is the local maximum
///   of its window *and* meaningfully higher than that window's minimum —
///   the neighbour-contrast the `not_built` reason named as missing even
///   with resistance wired in. Uniformly hard ground the whole way across a
///   window is not a formation.
/// - **standing proud**: positive TPI at the fine scale, the same signal
///   [`pool_ridge`]/[`pool_peak`] read — differential erosion means the
///   softer rock around it eroded away, so the resistant patch reads as
///   topography, not just as a geology-map anomaly with no expression.
///
/// [`Ctx::lithology`] is read only for the causal fact string, when present —
/// it degrades to a generic description rather than gating the kind, since a
/// bare rock-type category says nothing about contrast on its own.
fn pool_rock_formation(c: &Ctx<'_>) -> Option<Pool> {
    let resist = c.resistance?;
    if c.d.tpi_fine.is_empty() {
        return None;
    }
    let gw = c.inp.gw;
    let (rmin, rmax) = sep_min_max(resist, gw, c.inp.gh, c.d.r_fine, c.inp.world);
    let mut p = Pool::new();
    let (mut t_c, mut t_t, mut t_r) = (vec![], vec![], vec![]);
    for i in 0..c.n {
        // Domain: the local maximum of resistance represents the outcrop —
        // without this every hard-rock cell on its flank is its own
        // candidate, the same reasoning `pool_resource`'s own comment gives
        // for a deposit.
        if !c.is_land(i) || resist[i] < rmax[i] {
            continue;
        }
        let contrast = resist[i] as f64 - rmin[i] as f64;
        let tpi = c.inp.dh_m(c.d.tpi_fine(i) as f64);
        let relief = c.relief_m(i);
        if (resist[i] as f64) < ROCK_FORMATION_MIN_RESISTANCE
            || contrast < ROCK_FORMATION_MIN_CONTRAST
            || tpi < ROCK_FORMATION_MIN_TPI_M
        {
            p.rejected_constraint += 1;
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        let rock_name = c.lithology.and_then(|l| crate::LITH_NAMES.get(l[i] as usize)).copied();
        let headline = match rock_name {
            Some(name) => format!("{name}, standing proud of its softer surroundings"),
            None => "resistant rock standing proud of its softer surroundings".to_string(),
        };
        p.cands.push(Cand {
            i,
            x,
            y,
            facts: vec![
                headline,
                format!("resistance {:.2} against {:.2} nearby", resist[i], rmin[i]),
                format!("stands {} above its surroundings", fmt_m(tpi)),
            ],
        });
        t_c.push(contrast as f32);
        t_t.push(tpi as f32);
        t_r.push(relief as f32);
    }
    p.terms = vec![
        (ROCK_FORMATION_TERMS[0].0, ROCK_FORMATION_TERMS[0].1, t_c),
        (ROCK_FORMATION_TERMS[1].0, ROCK_FORMATION_TERMS[1].1, t_t),
        (ROCK_FORMATION_TERMS[2].0, ROCK_FORMATION_TERMS[2].1, t_r),
    ];
    Some(p)
}

/// §5: curvature "can contribute to candidate detection for cliffs" — but the
/// load-bearing test is gradient over a real height difference, with curvature
/// only deciding whether the face is a convex break or a smooth bowl.
fn pool_cliff(c: &Ctx<'_>) -> Option<Pool> {
    if c.d.slope.is_empty() {
        return None;
    }
    let gw = c.inp.gw;
    let (_, smax) = sep_min_max(&c.d.slope, gw, c.inp.gh, c.d.r_fine, c.inp.world);
    let mut p = Pool::new();
    let (mut t_g, mut t_r, mut t_c) = (vec![], vec![], vec![]);
    for i in 0..c.n {
        // Domain: the steepest cell of a face represents that face. Without
        // this every cell on a mountainside is a cliff candidate.
        if !c.is_land(i) || c.d.slope[i] < smax[i] {
            continue;
        }
        if c.chan.is_some_and(|ch| ch[i] != 0) {
            continue;
        }
        let grad = c.gradient(i);
        let relief = c.relief_m(i);
        if grad < CLIFF_MIN_GRADIENT || relief < CLIFF_MIN_RELIEF_M {
            p.rejected_constraint += 1;
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        // Negative Laplacian is convex — a nose or a lip, which is what breaks
        // into a face. See `analysis::curvature`'s own sign note.
        let convex = -(c.d.curv(i) as f64);
        p.cands.push(Cand {
            i,
            x,
            y,
            facts: vec![
                "a break in the slope, not a hillside".to_string(),
                format!("face gradient {}", fmt_pct(grad)),
                format!("relief across the face {}", fmt_m(relief)),
            ],
        });
        t_g.push(grad as f32);
        t_r.push(relief as f32);
        t_c.push(convex as f32);
    }
    p.terms = vec![
        (CLIFF_TERMS[0].0, CLIFF_TERMS[0].1, t_g),
        (CLIFF_TERMS[1].0, CLIFF_TERMS[1].1, t_r),
        (CLIFF_TERMS[2].0, CLIFF_TERMS[2].1, t_c),
    ];
    Some(p)
}

/// A river that has cut *down* into its surroundings.
fn pool_gorge(c: &Ctx<'_>) -> Option<Pool> {
    let chan = c.chan?;
    if c.d.tpi_fine.is_empty() {
        return None;
    }
    let gw = c.inp.gw;
    let mut p = Pool::new();
    let (mut t_i, mut t_r, mut t_s, mut t_q) = (vec![], vec![], vec![], vec![]);
    for i in 0..c.n {
        if chan[i] == 0 || !c.is_land(i) {
            continue;
        }
        let tpi = c.inp.dh_m(c.d.tpi_fine(i) as f64);
        let relief = c.relief_m(i);
        if tpi > GORGE_MAX_TPI_M || relief < GORGE_MIN_RELIEF_M {
            p.rejected_constraint += 1;
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        let q = c.flow.map(|f| f[i] as f64).unwrap_or(0.0);
        let mut facts = vec![
            "a channel running below the ground either side of it".to_string(),
            format!("incised {} below its surroundings", fmt_m(-tpi)),
            format!("wall relief {}", fmt_m(relief)),
        ];
        if c.flow.is_some() {
            facts.push(format!("flow {} cells", thousands(q)));
        }
        p.cands.push(Cand { i, x, y, facts });
        t_i.push(-tpi as f32);
        t_r.push(relief as f32);
        t_s.push(c.gradient(i) as f32);
        t_q.push(q as f32);
    }
    p.terms = vec![
        (GORGE_TERMS[0].0, GORGE_TERMS[0].1, t_i),
        (GORGE_TERMS[1].0, GORGE_TERMS[1].1, t_r),
        (GORGE_TERMS[2].0, GORGE_TERMS[2].1, t_s),
    ];
    if c.flow.is_some() {
        p.terms.push((GORGE_TERMS[3].0, GORGE_TERMS[3].1, t_q));
    }
    Some(p)
}

/// §14's resource chain, shared by Mine and Quarry — the two differ only in
/// which of `RESOURCE_KEYS` they read and in what the second term measures.
fn pool_resource(
    c: &Ctx<'_>,
    keys: &[&str],
    min_potential: f64,
    terms: &[(&'static str, f64); 3],
    prefer_flat: bool,
) -> Option<Pool> {
    let fields: Vec<(&str, &[f32])> =
        keys.iter().filter_map(|k| c.inp.resource(k).map(|v| (*k, v))).collect();
    if fields.is_empty() {
        return None;
    }
    let gw = c.inp.gw;
    // The best potential at each cell, and which resource it belongs to.
    let mut best = vec![0f32; c.n];
    let mut which = vec![0u8; c.n];
    for (k, (_, f)) in fields.iter().enumerate() {
        for i in 0..c.n {
            if f[i] > best[i] {
                best[i] = f[i];
                which[i] = k as u8;
            }
        }
    }
    let (_, bmax) = sep_min_max(&best, gw, c.inp.gh, c.d.r_fine, c.inp.world);
    let mut p = Pool::new();
    let (mut t_p, mut t_g, mut t_s) = (vec![], vec![], vec![]);
    let has_settle = !c.inp.settlements.is_empty();
    for i in 0..c.n {
        // Domain: a deposit, meaning a local maximum of the potential — §14's
        // chain starts "Ore deposit -> Mine", not "ore-bearing region -> mine
        // in every cell of it".
        if !c.is_land(i) || !(best[i] > 0.0) || best[i] < bmax[i] {
            continue;
        }
        let pot = best[i] as f64;
        if pot < min_potential {
            p.rejected_constraint += 1;
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        let grad = c.gradient(i);
        let name = fields[which[i] as usize].0;
        let inf = c.influence(x, y);
        let mut facts = vec![
            format!("{} potential {:.2}", name, pot),
            if prefer_flat {
                format!("workable ground at {}", fmt_pct(grad))
            } else {
                format!("stone exposed at {}", fmt_pct(grad))
            },
        ];
        if has_settle {
            facts.push(format!(
                "{} settlements within reach",
                c.inp.settlements.len()
            ));
        }
        p.cands.push(Cand { i, x, y, facts });
        t_p.push(pot as f32);
        // A mine wants ground it can dig; a quarry wants rock it can see.
        t_g.push(if prefer_flat { -grad as f32 } else { grad as f32 });
        t_s.push(inf as f32);
    }
    p.terms = vec![(terms[0].0, terms[0].1, t_p), (terms[1].0, terms[1].1, t_g)];
    if has_settle {
        p.terms.push((terms[2].0, terms[2].1, t_s));
    }
    Some(p)
}

/// A sheltered landing: land against ocean, level enough to build on, with
/// enough land around the approach that the water is a bay rather than open
/// coast.
fn pool_harbour(c: &Ctx<'_>) -> Option<Pool> {
    let _water = c.water?;
    let gw = c.inp.gw;
    let r = if c.cell_km > 0.0 {
        ((HARBOUR_SHELTER_KM / c.cell_km).round() as i64).clamp(2, HARBOUR_SHELTER_MAX_CELLS)
    } else {
        2
    };
    let mut p = Pool::new();
    let (mut t_sh, mut t_dp, mut t_lv, mut t_st) = (vec![], vec![], vec![], vec![]);
    let has_settle = !c.inp.settlements.is_empty();
    for i in 0..c.n {
        if !c.is_land(i) {
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        // Domain: the coastline, meaning land with ocean against it.
        let coastal = [(-1i64, 0i64), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)]
            .iter()
            .any(|(dx, dy)| c.nb(x, y, *dx, *dy).map(|j| c.is_ocean(j)).unwrap_or(false));
        if !coastal {
            continue;
        }
        let mut land_n = 0usize;
        let mut total = 0usize;
        let mut depth_acc = 0f64;
        let mut depth_n = 0usize;
        for dy in -r..=r {
            for dx in -r..=r {
                if let Some(j) = c.nb(x, y, dx, dy) {
                    total += 1;
                    if c.is_land(j) {
                        land_n += 1;
                    } else if c.is_ocean(j) {
                        depth_acc += -c.elev_m(j);
                        depth_n += 1;
                    }
                }
            }
        }
        let shelter = if total > 0 { land_n as f64 / total as f64 } else { 0.0 };
        let grad = c.gradient(i);
        if shelter < HARBOUR_MIN_SHELTER || grad > HARBOUR_MAX_GRADIENT {
            p.rejected_constraint += 1;
            continue;
        }
        let depth = if depth_n > 0 { depth_acc / depth_n as f64 } else { 0.0 };
        let inf = c.influence(x, y);
        let mut facts = vec![
            "land against open water, with the water enclosed".to_string(),
            format!("{} of the approach is land", fmt_pct(shelter)),
            format!("approach averages {} deep", fmt_m(depth)),
            format!("shore falls at {}", fmt_pct(grad)),
        ];
        if has_settle {
            facts.push(format!("{} settlements within reach", c.inp.settlements.len()));
        }
        p.cands.push(Cand { i, x, y, facts });
        t_sh.push(shelter as f32);
        t_dp.push(depth as f32);
        t_lv.push(-grad as f32);
        t_st.push(inf as f32);
    }
    p.terms = vec![
        (HARBOUR_TERMS[0].0, HARBOUR_TERMS[0].1, t_sh),
        (HARBOUR_TERMS[1].0, HARBOUR_TERMS[1].1, t_dp),
        (HARBOUR_TERMS[2].0, HARBOUR_TERMS[2].1, t_lv),
    ];
    if has_settle {
        p.terms.push((HARBOUR_TERMS[3].0, HARBOUR_TERMS[3].1, t_st));
    }
    Some(p)
}

/// A river small enough to wade, with banks shallow enough to walk down.
fn pool_ford(c: &Ctx<'_>) -> Option<Pool> {
    let chan = c.chan?;
    let flow = c.flow?;
    let gw = c.inp.gw;
    let mut p = Pool::new();
    let (mut t_q, mut t_b, mut t_c, mut t_s) = (vec![], vec![], vec![], vec![]);
    let has_settle = !c.inp.settlements.is_empty();
    let has_cor = c.corridors.is_some();
    let lo = c.flow_thresh * FORD_MIN_FLOW_MULT;
    let hi = c.flow_thresh * FORD_MAX_FLOW_MULT;
    for i in 0..c.n {
        if chan[i] == 0 || !c.is_land(i) {
            continue;
        }
        let q = flow[i] as f64;
        let grad = c.gradient(i);
        if !(q >= lo) || q > hi || grad > FORD_MAX_GRADIENT {
            p.rejected_constraint += 1;
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        let cor = c.corridors.map(|v| v[i] as f64).unwrap_or(0.0);
        let inf = c.influence(x, y);
        let mut facts = vec![
            "a channel small enough to cross on foot".to_string(),
            format!("flow {} cells", thousands(q)),
            format!("banks at {}", fmt_pct(grad)),
        ];
        if has_cor {
            facts.push(format!("corridor value {:.2}", cor));
        }
        p.cands.push(Cand { i, x, y, facts });
        // Less water is a better ford, so the term is negated before it is
        // normalised.
        t_q.push(-q as f32);
        t_b.push(-grad as f32);
        t_c.push(cor as f32);
        t_s.push(inf as f32);
    }
    p.terms =
        vec![(FORD_TERMS[0].0, FORD_TERMS[0].1, t_q), (FORD_TERMS[1].0, FORD_TERMS[1].1, t_b)];
    if has_cor {
        p.terms.push((FORD_TERMS[2].0, FORD_TERMS[2].1, t_c));
    }
    if has_settle {
        p.terms.push((FORD_TERMS[3].0, FORD_TERMS[3].1, t_s));
    }
    Some(p)
}

// ---------------------------------------------------------------------------
// The five way-graph kinds
// ---------------------------------------------------------------------------
//
// All five read [`Ctx::ways`] and nothing else can substitute for it: each one
// was unbuildable until `LandmarkInputs::ways` carried the **routed graph**
// here, and each one's own `not_built` reason said exactly that. `c.ways`
// is `None` when the caller passed no ways, so all five degrade to
// `NoTerrain` with `candidates: 0` on a world whose network has not been
// built — the same contract every optional raster above has.
//
// **They partition the way network rather than sharing it.** A cell is a
// Bridge site only if a channel runs through it, a Trade depot only if one
// does *not* and a navigable one runs beside it, and Market site and Caravan
// station split the remaining trade road by the one measurement that
// distinguishes them (`MARKET_MAX_SETTLEMENT_KM` vs
// `CARAVAN_MIN_SETTLEMENT_KM`, with a deliberate empty band between). Only
// Road junction is free to coincide with another, and only where three ways
// meet.

/// Three or more distinct ways on one cell.
///
/// The domain is the way network itself, so `rejected_constraint` counts road
/// cells that are road and not junction — which is the honest funnel: "there
/// are this many cells of road, and this few of them are junctions".
fn pool_junction(c: &Ctx<'_>) -> Option<Pool> {
    let ways = c.ways.as_ref()?;
    let gw = c.inp.gw;
    let mut p = Pool::new();
    let (mut t_j, mut t_r, mut t_s) = (vec![], vec![], vec![]);
    let has_settle = !c.inp.settlements.is_empty();
    for i in 0..c.n {
        if ways.ways_here[i] == 0 || !c.is_land(i) {
            continue;
        }
        let meeting = ways.ways_here[i] as usize;
        if meeting < JUNCTION_MIN_WAYS {
            p.rejected_constraint += 1;
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        let rank = ways.rank[i];
        let inf = c.influence(x, y);
        let mut facts = vec![
            format!("{} separate ways meet on one cell", meeting),
            format!("the busiest of them is a {}", rank_label(rank)),
        ];
        if has_settle {
            facts.push(format!(
                "nearest settlement {}",
                fmt_km(c.nearest_settlement_km(x, y))
            ));
        }
        p.cands.push(Cand { i, x, y, facts });
        t_j.push(meeting as f32);
        t_r.push(rank as f32);
        t_s.push(inf as f32);
    }
    p.terms = vec![
        (JUNCTION_TERMS[0].0, JUNCTION_TERMS[0].1, t_j),
        (JUNCTION_TERMS[1].0, JUNCTION_TERMS[1].1, t_r),
    ];
    if has_settle {
        p.terms.push((JUNCTION_TERMS[2].0, JUNCTION_TERMS[2].1, t_s));
    }
    Some(p)
}

/// §12's bottleneck: a routed way meeting water too big to wade.
///
/// The constraint is [`FORD_MAX_FLOW_MULT`] read from the other side, so the
/// same channel cells become Fords below it and Bridge sites above it and
/// neither kind double-counts the other's.
fn pool_bridge(c: &Ctx<'_>) -> Option<Pool> {
    let ways = c.ways.as_ref()?;
    let chan = c.chan?;
    let flow = c.flow?;
    let gw = c.inp.gw;
    let mut p = Pool::new();
    let (mut t_q, mut t_r, mut t_g, mut t_s) = (vec![], vec![], vec![], vec![]);
    let has_settle = !c.inp.settlements.is_empty();
    let min_flow = c.flow_thresh * FORD_MAX_FLOW_MULT;
    for i in 0..c.n {
        // Domain: where the routed network crosses a channel.
        if ways.ways_here[i] == 0 || chan[i] == 0 || !c.is_land(i) {
            continue;
        }
        let q = flow[i] as f64;
        if !(q > min_flow) {
            // Wadeable. This is a Ford, and Ford generates it.
            p.rejected_constraint += 1;
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        let rank = ways.rank[i];
        let grad = c.gradient(i);
        let inf = c.influence(x, y);
        let mut facts = vec![
            format!("a {} meets a channel too big to wade", rank_label(rank)),
            format!("flow {} cells", thousands(q)),
            format!("abutments at {}", fmt_pct(grad)),
        ];
        if has_settle {
            facts.push(format!("{} settlements within reach", c.inp.settlements.len()));
        }
        p.cands.push(Cand { i, x, y, facts });
        t_q.push(q as f32);
        t_r.push(rank as f32);
        t_g.push(-grad as f32);
        t_s.push(inf as f32);
    }
    p.terms = vec![
        (BRIDGE_TERMS[0].0, BRIDGE_TERMS[0].1, t_q),
        (BRIDGE_TERMS[1].0, BRIDGE_TERMS[1].1, t_r),
        (BRIDGE_TERMS[2].0, BRIDGE_TERMS[2].1, t_g),
    ];
    if has_settle {
        p.terms.push((BRIDGE_TERMS[3].0, BRIDGE_TERMS[3].1, t_s));
    }
    Some(p)
}

/// Market site and Caravan station, which are one measurement with opposite
/// senses — see [`CARAVAN_MIN_SETTLEMENT_KM`].
///
/// `near` selects which: `true` keeps the trade road inside a settlement's
/// catchment (Market), `false` keeps only the road in the gap between
/// settlements (Caravan station). The `bound` is the km line each is stated
/// against, and the first suitability term is the settlement distance itself,
/// signed so that "better" is always larger.
///
/// **Both return `None` without settlements**, rather than degrading: with no
/// settlement anywhere `nearest_settlement_km` is infinite, which would reject
/// every cell for a market and accept every cell for a caravan station. The
/// second is the fabrication this module refuses everywhere else — a waystation
/// between two towns that do not exist.
fn pool_trade_road(
    c: &Ctx<'_>,
    near: bool,
    bound: f64,
    terms: &[(&'static str, f64); 3],
) -> Option<Pool> {
    let ways = c.ways.as_ref()?;
    if c.inp.settlements.is_empty() {
        return None;
    }
    let gw = c.inp.gw;
    let mut p = Pool::new();
    let (mut t_d, mut t_r, mut t_g) = (vec![], vec![], vec![]);
    for i in 0..c.n {
        if ways.ways_here[i] == 0 || !c.is_land(i) {
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        let rank = ways.rank[i];
        let d = c.nearest_settlement_km(x, y);
        let in_band = if near { d <= bound } else { d >= bound };
        if rank < TRADE_MIN_WAY_RANK || !in_band {
            p.rejected_constraint += 1;
            continue;
        }
        let grad = c.gradient(i);
        p.cands.push(Cand {
            i,
            x,
            y,
            facts: vec![
                format!("on a {}", rank_label(rank)),
                format!("nearest settlement {}", fmt_km(d)),
                format!("workable ground at {}", fmt_pct(grad)),
            ],
        });
        // A market wants to be close, a station wants to be far.
        t_d.push(if near { -d as f32 } else { d as f32 });
        t_r.push(rank as f32);
        t_g.push(-grad as f32);
    }
    p.terms = vec![
        (terms[0].0, terms[0].1, t_d),
        (terms[1].0, terms[1].1, t_r),
        (terms[2].0, terms[2].1, t_g),
    ];
    Some(p)
}

/// A transhipment point: a trade road on the **bank** of water a boat can work.
///
/// Deliberately not on the channel cell itself — that cell is where the road
/// crosses, which is a Ford or a Bridge site. Goods come off the road and onto
/// the water beside it, so the depot is the land cell and the navigability test
/// is asked of its channel neighbour.
fn pool_depot(c: &Ctx<'_>) -> Option<Pool> {
    let ways = c.ways.as_ref()?;
    let chan = c.chan?;
    let flow = c.flow?;
    let gw = c.inp.gw;
    let mut p = Pool::new();
    let (mut t_r, mut t_s, mut t_g) = (vec![], vec![], vec![]);
    let has_settle = !c.inp.settlements.is_empty();
    let navigable = c.flow_thresh * DEPOT_NAVIGABLE_FLOW_MULT;
    for i in 0..c.n {
        if ways.ways_here[i] == 0 || chan[i] != 0 || !c.is_land(i) {
            continue;
        }
        let (x, y) = (i % gw, i / gw);
        // The busiest channel touching this cell.
        let mut beside = 0f64;
        for (dx, dy) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)]
        {
            if let Some(j) = c.nb(x, y, dx, dy)
                && chan[j] != 0
                && (flow[j] as f64) > beside
            {
                beside = flow[j] as f64;
            }
        }
        let rank = ways.rank[i];
        if rank < TRADE_MIN_WAY_RANK || !(beside >= navigable) {
            p.rejected_constraint += 1;
            continue;
        }
        let grad = c.gradient(i);
        let inf = c.influence(x, y);
        let mut facts = vec![
            format!("a {} on the bank of navigable water", rank_label(rank)),
            format!("the channel beside it carries {} cells", thousands(beside)),
            format!("workable ground at {}", fmt_pct(grad)),
        ];
        if has_settle {
            facts.push(format!(
                "nearest settlement {}",
                fmt_km(c.nearest_settlement_km(x, y))
            ));
        }
        p.cands.push(Cand { i, x, y, facts });
        t_r.push(rank as f32);
        t_s.push(inf as f32);
        t_g.push(-grad as f32);
    }
    p.terms = vec![(DEPOT_TERMS[0].0, DEPOT_TERMS[0].1, t_r)];
    if has_settle {
        p.terms.push((DEPOT_TERMS[1].0, DEPOT_TERMS[1].1, t_s));
    }
    p.terms.push((DEPOT_TERMS[2].0, DEPOT_TERMS[2].1, t_g));
    Some(p)
}

/// The one place a key becomes a detector. A key with no arm here is not
/// buildable, and [`kinds`] must say so.
fn detect(key: &str, c: &Ctx<'_>) -> Option<Pool> {
    match key {
        "waterfall" => pool_waterfall(c),
        "spring" => pool_spring(c),
        "river_confluence" => pool_confluence(c),
        "lake" => pool_lake(c),
        "mountain_pass" => pool_pass(c),
        "peak" => pool_peak(c),
        "ridge" => pool_ridge(c),
        "cliff" => pool_cliff(c),
        "gorge" => pool_gorge(c),
        "rock_formation" => pool_rock_formation(c),
        "mine" => pool_resource(c, &MINE_RESOURCES, MINE_MIN_POTENTIAL, &MINE_TERMS, true),
        "quarry" => {
            pool_resource(c, &QUARRY_RESOURCES, QUARRY_MIN_POTENTIAL, &QUARRY_TERMS, false)
        }
        "resource_extraction_site" => {
            pool_resource(c, &EXTRACTION_RESOURCES, EXTRACTION_MIN_POTENTIAL, &EXTRACTION_TERMS, true)
        }
        "harbour" => pool_harbour(c),
        "ford" => pool_ford(c),
        "road_junction" => pool_junction(c),
        "bridge_site" => pool_bridge(c),
        "market_site" => {
            pool_trade_road(c, true, MARKET_MAX_SETTLEMENT_KM, &MARKET_TERMS)
        }
        "caravan_station" => {
            pool_trade_road(c, false, CARAVAN_MIN_SETTLEMENT_KM, &CARAVAN_TERMS)
        }
        "trade_depot" => pool_depot(c),
        _ => None,
    }
}

// ===========================================================================
// §30 steps 7-11 — the pass
// ===========================================================================

/// §17's `S_L(x) = Σ w_k · F_k(x)`, divided by the weight of the terms that
/// were actually measurable, so an absent input renormalises the sum instead
/// of dragging every candidate toward zero.
fn weighted_sum(terms: &[(&'static str, f64, Vec<f32>)], n: usize) -> Vec<f64> {
    let mut acc = vec![0f64; n];
    let mut wsum = 0f64;
    for (_, w, raw) in terms {
        if raw.len() != n || !(*w > 0.0) {
            continue;
        }
        let nz = norm_term(raw);
        for (a, v) in acc.iter_mut().zip(nz.iter()) {
            *a += w * *v as f64;
        }
        wsum += *w;
    }
    if !(wsum > 0.0) {
        return vec![0.0; n];
    }
    acc.iter().map(|a| (a / wsum).clamp(0.0, 1.0)).collect()
}

/// What a kind's funnel says when the pass never ran it.
fn skipped_limit(spec: &LandmarkKindSpec, settings: &LandmarkSettings) -> LandmarkLimit {
    if !spec.buildable {
        LandmarkLimit::NotBuildable
    } else if !settings.is_armed(spec.key) || settings.cap(spec.key) == 0 {
        LandmarkLimit::Disarmed
    } else {
        LandmarkLimit::NoTerrain
    }
}

fn all_skipped(settings: &LandmarkSettings, t0: std::time::Instant) -> LandmarkResult {
    LandmarkResult {
        landmarks: Vec::new(),
        // Nothing was walked, so nothing was rejected on the walk. A kind
        // skipped here is `NoTerrain`/`Disarmed`/`NotBuildable`, which the
        // funnel already says; inventing reject rows for it would be this list
        // claiming candidates that were never offered.
        rejects: Vec::new(),
        funnels: kinds()
            .iter()
            .map(|k| {
                LandmarkFunnel::empty(
                    k.key,
                    settings.cap(k.key) as usize,
                    skipped_limit(k, settings),
                )
            })
            .collect(),
        seconds: t0.elapsed().as_secs_f64(),
    }
}

/// **The pass.** Research §30's twelve steps, over whichever inputs the caller
/// really has.
///
/// Deterministic per §27: same world, same settings, same `world_seed` ⇒ the
/// same landmarks in the same order with the same ids. It draws no random
/// numbers at all — the only place a seed appears is [`Landmark::seed`], which
/// is an identity for later passes to derive from, not a source of variation
/// here. `two_runs_of_the_same_world_are_identical` pins this.
///
/// ## Order of work, and why kinds are processed by class
///
/// Kinds are processed **Continental first, then Regional, Local, Cultural**,
/// and within a class in table order. That ordering is load-bearing when
/// [`LandmarkSettings::cross_type_competition`] is on: §16 says a major
/// landmark exerts a large exclusion radius, which is only true if the major
/// one is placed first. Processing in table order instead would let a Physical
/// Local type claim the ground a Continental one needed, purely because
/// Physical is the first family in §29's list.
pub fn generate(
    inputs: &LandmarkInputs<'_>,
    settings: &LandmarkSettings,
    world_seed: u64,
) -> LandmarkResult {
    let t0 = std::time::Instant::now();
    let n = inputs.n();
    if n == 0 || inputs.field.len() != n {
        // A caller bug or an empty world. Neither is a reason to panic across
        // the gdext boundary, and neither is a reason to invent a landmark.
        return all_skipped(settings, t0);
    }

    let active: Vec<&'static LandmarkKindSpec> = kinds()
        .iter()
        .filter(|k| k.buildable && settings.is_armed(k.key) && settings.cap(k.key) > 0)
        .collect();
    if active.is_empty() {
        return all_skipped(settings, t0);
    }

    let need = active.iter().fold(Needs::default(), |acc, k| acc.merge(Needs::of(k.key)));
    let ctx = Ctx::build(inputs, need);
    let cell_km = ctx.cell_km;

    // Radii in cells, and the largest of them, which is the bucket edge.
    let radius_cells = |class: LandmarkClass| -> f64 {
        if !(cell_km > 0.0) {
            return 0.0;
        }
        let r = settings.radius_km(class) / cell_km;
        if r.is_finite() && r > 0.0 {
            r
        } else {
            0.0
        }
    };
    let max_radius = active
        .iter()
        .map(|k| radius_cells(k.class))
        .fold(0.0f64, |a, b| if b > a { b } else { a });

    // Continental first. See the doc comment.
    let mut order: Vec<&'static LandmarkKindSpec> = active.clone();
    order.sort_by_key(|k| {
        (
            k.class.index(),
            kinds().iter().position(|q| q.key == k.key).unwrap_or(usize::MAX),
        )
    });

    let mut shared = Buckets::new(inputs.gw, inputs.gh, inputs.world, max_radius);
    let mut funnels: BTreeMap<&'static str, LandmarkFunnel> = BTreeMap::new();
    let mut out: Vec<Landmark> = Vec::new();
    let mut rejects: Vec<LandmarkReject> = Vec::new();
    // The crowding actually in force, read once. `needs_crowding` is a ratio
    // against this, and reading it from the same accessor `radius_km` uses is
    // what stops the two disagreeing.
    let crowding = settings.crowding_in_force();

    for spec in order {
        let cap = settings.cap(spec.key) as usize;
        let pool = match detect(spec.key, &ctx) {
            Some(p) => p,
            None => {
                // The input this kind needs is absent. Zero candidates, zero
                // placed, and the reason said out loud.
                funnels.insert(
                    spec.key,
                    LandmarkFunnel::empty(spec.key, cap, LandmarkLimit::NoTerrain),
                );
                continue;
            }
        };
        let candidates = pool.candidates();
        let scores = weighted_sum(&pool.terms, pool.cands.len());

        // §30 step 8. Sorted by suitability, ties broken by cell index so the
        // order is total and therefore reproducible.
        let mut ord: Vec<usize> = (0..pool.cands.len()).collect();
        ord.sort_by(|a, b| {
            scores[*b]
                .partial_cmp(&scores[*a])
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| pool.cands[*a].i.cmp(&pool.cands[*b].i))
        });

        let r = radius_cells(spec.class);
        let mut own = if settings.cross_type_competition {
            None
        } else {
            Some(Buckets::new(inputs.gw, inputs.gh, inputs.world, r))
        };

        let mut rejected_score = 0usize;
        let mut rejected_spacing = 0usize;
        let mut placed = 0usize;
        let mut walked = 0usize;
        // Retained rows for THIS kind, so one prolific kind cannot spend the
        // whole budget. See `REJECT_LIST_MAX_PER_KIND`.
        let mut listed = 0usize;
        while walked < ord.len() {
            if placed >= cap {
                break;
            }
            let ci = ord[walked];
            walked += 1;
            if scores[ci] < SCORE_FLOOR {
                rejected_score += 1;
                if listed < REJECT_LIST_MAX_PER_KIND {
                    listed += 1;
                    let c = &pool.cands[ci];
                    rejects.push(LandmarkReject {
                        kind: spec.key,
                        x: c.x,
                        y: c.y,
                        score: scores[ci],
                        reason: LandmarkRejectReason::Score,
                        needs_crowding: None,
                    });
                }
                continue;
            }
            let cand = &pool.cands[ci];
            let (fx, fy) = (cand.x as f64, cand.y as f64);
            let fits = match &own {
                Some(b) => b.fits(fx, fy, r),
                None => shared.fits(fx, fy, r),
            };
            if !fits {
                rejected_spacing += 1;
                if listed < REJECT_LIST_MAX_PER_KIND {
                    listed += 1;
                    // `d < r` by construction — `fits` said so — and the ring
                    // scales as `base / crowding`, so it clears at
                    // `crowding * r / d`. Guarded on `d > 0`: a blocker on this
                    // very cell is not reachable by any finite setting, and
                    // saying `None` there is the honest answer.
                    let d = match &own {
                        Some(b) => b.nearest_conflict_sq(fx, fy, r),
                        None => shared.nearest_conflict_sq(fx, fy, r),
                    }
                    .map(f64::sqrt);
                    let needs = d.filter(|d| *d > 0.0).map(|d| crowding * r / d).filter(|v| v.is_finite());
                    rejects.push(LandmarkReject {
                        kind: spec.key,
                        x: cand.x,
                        y: cand.y,
                        score: scores[ci],
                        reason: LandmarkRejectReason::Spacing,
                        needs_crowding: needs,
                    });
                }
                continue;
            }
            match &mut own {
                Some(b) => b.take(fx, fy),
                None => shared.take(fx, fy),
            }
            placed += 1;
            let mut causal = cand.facts.clone();
            causal.push(spec.label.to_string());
            out.push(Landmark {
                id: 0,
                kind: spec.key.to_string(),
                class: spec.class,
                x: cand.x,
                y: cand.y,
                elevation: ctx.elev_m(cand.i),
                score: scores[ci],
                importance: 0.0,
                causal,
                seed: landmark_seed(
                    world_seed,
                    cand.i as u64,
                    spec.class,
                    GENERATOR_VERSION,
                ),
            });
        }
        // Everything the walk never reached, because the cap ran out. Its own
        // bucket since 2026-08-30 — see `LandmarkFunnel`'s doc comment. These
        // candidates passed every test the generator applies; the only thing
        // that stopped them is the number the user set.
        let rejected_cap = ord.len() - walked;
        // The best-scoring of them, and only the best-scoring: `ord` is
        // score-sorted, so this is the head of the queue the cap cut off. The
        // tail is unbounded (a cap of 1 on a 40 000-candidate kind rejects
        // 39 999 here) and is exactly the payload `LandmarkReject`'s note is
        // about, so it is never materialised at all — not built and then
        // truncated.
        //
        // These rows are NOT a claim that the candidate would have been placed.
        // The walk stopped, so nothing tested them against the spacing field a
        // higher cap would have grown; several of them would collide with each
        // other. They are "the next in line", which is what the cap bucket
        // means and all it means.
        for &ci in ord[walked..].iter().take(REJECT_LIST_MAX_PER_KIND.saturating_sub(listed)) {
            let c = &pool.cands[ci];
            rejects.push(LandmarkReject {
                kind: spec.key,
                x: c.x,
                y: c.y,
                score: scores[ci],
                reason: LandmarkRejectReason::Cap,
                needs_crowding: None,
            });
        }

        let limit = if candidates == 0 {
            LandmarkLimit::NoTerrain
        } else if rejected_cap > 0 || (cap > 0 && placed >= cap) {
            // `rejected_cap > 0` is the direct statement — something was turned
            // away purely by the number. The second clause still matters for
            // the exact-fit case, where the walk ended on the last candidate
            // AND on the cap, so nothing was turned away but the cap is still
            // what is binding: dragging the slider right would place more if
            // more existed. Both are `at cap` to a reader.
            LandmarkLimit::AtCap
        } else if placed == 0 && pool.rejected_constraint + rejected_score == candidates {
            LandmarkLimit::NoTerrain
        } else if rejected_spacing > 0 {
            LandmarkLimit::Spacing
        } else {
            LandmarkLimit::Candidates
        };

        funnels.insert(
            spec.key,
            LandmarkFunnel {
                kind: spec.key.to_string(),
                candidates,
                rejected_constraint: pool.rejected_constraint,
                rejected_score,
                rejected_spacing,
                rejected_cap,
                cap,
                placed,
                limit,
            },
        );
    }

    // §30 step 11 — importance, after every kind is placed, because §24 wants
    // it to be a property of the world rather than of one detector.
    assign_importance(&mut out, &ctx);
    for (k, lm) in out.iter_mut().enumerate() {
        lm.id = k as u64 + 1;
    }

    LandmarkResult {
        funnels: kinds()
            .iter()
            .map(|k| {
                funnels.remove(k.key).unwrap_or_else(|| {
                    LandmarkFunnel::empty(
                        k.key,
                        settings.cap(k.key) as usize,
                        skipped_limit(k, settings),
                    )
                })
            })
            .collect(),
        landmarks: out,
        rejects,
        seconds: t0.elapsed().as_secs_f64(),
    }
}

/// §24 — "Cartalith should avoid assigning importance solely through a random
/// rarity variable … A feature becomes important because something makes it
/// important."
///
/// The three terms are [`IMPORTANCE_TERMS`], normalised across the whole placed
/// set so importance is comparable between a waterfall and a mine rather than
/// only within a kind. The settlement term is simply absent from the sum when
/// there are no settlements — the same renormalisation every suitability model
/// here uses.
fn assign_importance(out: &mut [Landmark], ctx: &Ctx<'_>) {
    if out.is_empty() {
        return;
    }
    let n = out.len();
    let mut t_s: Vec<f32> = Vec::with_capacity(n);
    let mut t_c: Vec<f32> = Vec::with_capacity(n);
    let mut t_i: Vec<f32> = Vec::with_capacity(n);
    for lm in out.iter() {
        t_s.push(lm.score as f32);
        t_c.push(class_weight(lm.class));
        t_i.push(ctx.influence(lm.x, lm.y) as f32);
    }
    let mut terms: Vec<(&'static str, f64, Vec<f32>)> = vec![
        (IMPORTANCE_TERMS[0].0, IMPORTANCE_TERMS[0].1, t_s),
        (IMPORTANCE_TERMS[1].0, IMPORTANCE_TERMS[1].1, t_c),
    ];
    if !ctx.inp.settlements.is_empty() {
        terms.push((IMPORTANCE_TERMS[2].0, IMPORTANCE_TERMS[2].1, t_i));
    }
    let imp = weighted_sum(&terms, n);
    for (lm, v) in out.iter_mut().zip(imp) {
        lm.importance = v;
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const SEA: f64 = 0.42;
    const PEAK_M: f64 = 4000.0;

    /// A small world built to reach the code rather than to look pretty: a
    /// rough mountain belt in the west, a **north-south escarpment** at 30 % of
    /// the width (the one landform that produces a real cell-scale cliff at
    /// kilometre resolution), a closed basin that pools into a lake, three
    /// summits, and a coast in the east. Deterministic, so a failure names a
    /// shape rather than a seed.
    ///
    /// Sized against the real defaults: 256 x 192 at 1 000 km across is a
    /// 3.9 km cell, the same order as `WorldParams::defaults`' 512 at 800 km.
    /// That matters — the class radii are in km, so a fixture on a 200 km map
    /// would have a 34 km regional exclusion swallowing a fifth of the world
    /// and every kind after the first would starve on spacing.
    fn test_field(gw: usize, gh: usize) -> Vec<f32> {
        let mut f = vec![0f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                let fx = x as f64 / gw as f64;
                let fy = y as f64 / gh as f64;
                // Tilt: land to fx ~ 0.40, then sea.
                let mut h = 0.80 - 0.633 * fx;
                // The escarpment. Half a step either side, over about one cell.
                h += 0.14 * ((0.30 - fx) * 400.0).tanh();
                // Roughness, west only — this is what makes gradients steep
                // enough for a waterfall to exist at all.
                let mask = ((0.35 - fx) * 4.0).clamp(0.0, 1.0);
                let rough = (x as f64 * 0.55).sin() * (y as f64 * 0.47).cos()
                    + 0.5 * (x as f64 * 1.3).sin() * (y as f64 * 1.1).cos();
                h += mask * 0.030 * rough;
                // Six east-west troughs. Without them the drainage runs in
                // parallel rills and no cell ever accumulates enough flow to
                // be a river, so the waterfall constraint could never fire —
                // exactly the "shape the fixture to reach the code" rule.
                h -= 0.050 * (fy * std::f64::consts::PI * 6.0).sin().abs();
                for (px, py, amp, sig) in [
                    (0.10f64, 0.22f64, 0.14f64, 0.030f64),
                    (0.18, 0.62, 0.11, 0.028),
                    (0.07, 0.86, 0.09, 0.025),
                ] {
                    let d2 = (fx - px).powi(2) + (fy - py).powi(2);
                    h += amp * (-d2 / (2.0 * sig * sig)).exp();
                }
                // A closed basin west of the escarpment, which the priority
                // flood fills as an above-sea lake.
                let d2 = (fx - 0.20f64).powi(2) + (fy - 0.50f64).powi(2);
                h -= 0.20 * (-d2 / (2.0 * 0.040f64 * 0.040)).exp();
                f[y * gw + x] = h.clamp(0.0, 1.0) as f32;
            }
        }
        f
    }

    /// A Gaussian potential blob, standing in for one of
    /// `build_resource_potentials`' fifteen fields.
    fn blob(gw: usize, gh: usize, px: f64, py: f64, sig: f64) -> Vec<f32> {
        let mut v = vec![0f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                let d2 = (x as f64 / gw as f64 - px).powi(2) + (y as f64 / gh as f64 - py).powi(2);
                v[y * gw + x] = (-d2 / (2.0 * sig * sig)).exp() as f32;
            }
        }
        v
    }

    struct World {
        gw: usize,
        gh: usize,
        width_km: f64,
        field: Vec<f32>,
        flow: Vec<f32>,
        chan: Vec<u8>,
        recv: Vec<i32>,
        order: Vec<i16>,
        water: Vec<u8>,
        corridors: Vec<f32>,
        iron: Vec<f32>,
        stone: Vec<f32>,
        /// Stands in for one of [`EXTRACTION_RESOURCES`] — Mine and Quarry's
        /// fixtures get `iron`/`stone`, so Resource extraction site needs its
        /// own so `every_buildable_kind_can_actually_place_one` can reach it.
        timber: Vec<f32>,
        /// A tight, high-contrast patch centred on `test_field`'s own first
        /// summit blob (cell ~(26,42) at this fixture's size) — an outcrop
        /// with real topographic expression, standing well above the
        /// uniform low background everywhere else. Rock formation's whole
        /// domain; nothing else reads this field.
        resistance: Vec<f32>,
        settlements: Vec<LandmarkSite>,
        /// The **routed** way network over this same terrain, produced by the
        /// engine's own `civ_hierarchical_network_topology` +
        /// `civ_consolidate_and_smooth_ways` rather than by hand-drawn
        /// polylines — the five way-graph kinds read a graph, so a fixture
        /// that faked one would test nothing they will meet in the shell.
        ways: Vec<crate::Way>,
    }

    /// The fixture's towns, in one place because three things read them: the
    /// landmark pass's own gravity term, the network pass that routes roads
    /// between them, and Market/Caravan/Depot, whose whole constraint is the
    /// distance from one.
    ///
    /// Eight rather than the two this fixture began with, and the reason is
    /// [`JUNCTION_MIN_WAYS`]: a two-settlement world routes one edge and has
    /// no junction anywhere, so `road_junction` could never fire and the
    /// "flag flipped, places nothing" failure would ship. All eight sit on the
    /// western land strip (`test_field` runs to sea at about `fx = 0.38`) and
    /// clear of the closed basin at `(0.20, 0.50)` that floods into the lake.
    fn towns(gw: usize, gh: usize) -> Vec<(usize, usize, u32)> {
        // The first two are this fixture's original pair, spelled the way they
        // were spelled before (`gw / 4, gh / 3` and `gw / 3, gh * 2 / 3`), so
        // the six added for the network do not silently move them.
        vec![
            (gw / 4, gh / 3, 12_000u32),
            (gw / 3, gh * 2 / 3, 4_000),
            (gw * 9 / 100, gh * 12 / 100, 9_000),
            (gw * 11 / 100, gh * 52 / 100, 6_000),
            (gw * 8 / 100, gh * 83 / 100, 5_000),
            (gw * 27 / 100, gh / 10, 7_000),
            (gw * 21 / 100, gh * 89 / 100, 3_000),
            (gw * 34 / 100, gh * 47 / 100, 8_000),
        ]
    }

    /// The fixture wired through the engine's own hydrology and corridor
    /// functions rather than through hand-written masks, so the detectors are
    /// exercised against the shapes they will really see.
    fn world(gw: usize, gh: usize, width_km: f64) -> World {
        let field = test_field(gw, gh);
        let flow = cartalith_hydrology::compute_flow(gw, gh, &field, None, false, false);
        let ch = cartalith_hydrology::build_channels(
            &field, &flow, gw, gh, SEA, false, 1.0, width_km,
        );
        let order = cartalith_hydrology::strahler_from_receivers(&ch.recv, &flow, &ch.chan);
        let wb = crate::build_water_bodies(&field, gw, gh, SEA, false, None);
        let raw_slope = crate::build_raw_slope_field(&field, gw, gh, false);
        let thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, width_km);
        let corridors = crate::build_route_corridors(
            &field,
            &raw_slope,
            Some(&flow),
            gw,
            gh,
            SEA,
            false,
            thresh,
        );
        // The real routed network, over this same terrain: the engine's own
        // MST/Dijkstra topology pass, then the busiest-first consolidation
        // that is what puts three strokes through one cell and makes a
        // junction exist at all. `biome` is a uniform `0`, whose
        // `civ_biome_friction` is exactly `1.0` — the neutral choice, stated
        // rather than a fabricated vegetation map; `order` is the same
        // Strahler field this fixture already computes.
        let places: Vec<crate::NamedSettlement> = towns(gw, gh)
            .iter()
            .enumerate()
            .map(|(k, (x, y, pop))| crate::NamedSettlement {
                tid: 0,
                placement: crate::SettlementPlacement {
                    x: *x,
                    y: *y,
                    suit: 0.0,
                    faction: k as i32 + 1,
                    capital: true,
                    kind: crate::SettlementKind::Capital,
                    coastal: false,
                },
                name: format!("T{}", k),
                pop: *pop,
            })
            .collect();
        let placements: Vec<crate::SettlementPlacement> =
            places.iter().map(|p| p.placement).collect();
        let biome = vec![0u8; gw * gh];
        let topology = crate::civ_hierarchical_network_topology(
            &placements,
            gw,
            gh,
            SEA,
            &field,
            &flow,
            &order,
            &biome,
            &wb.classification,
            false,
            width_km,
        );
        let ways = crate::civ_consolidate_and_smooth_ways(
            &topology,
            &places,
            &field,
            &wb.classification,
            gw,
            gh,
            width_km,
        );
        World {
            gw,
            gh,
            width_km,
            field,
            flow,
            chan: ch.chan,
            recv: ch.recv,
            order,
            water: wb.classification,
            corridors,
            iron: {
                let mut v = blob(gw, gh, 0.24, 0.20, 0.028);
                for (px, py) in [(0.26f64, 0.72f64), (0.13, 0.10)] {
                    let b = blob(gw, gh, px, py, 0.030);
                    for i in 0..v.len() {
                        if b[i] > v[i] {
                            v[i] = b[i];
                        }
                    }
                }
                v
            },
            stone: {
                let mut v = blob(gw, gh, 0.23, 0.45, 0.028);
                let b = blob(gw, gh, 0.26, 0.92, 0.028);
                for i in 0..v.len() {
                    if b[i] > v[i] {
                        v[i] = b[i];
                    }
                }
                v
            },
            timber: {
                let mut v = blob(gw, gh, 0.30, 0.30, 0.028);
                let b = blob(gw, gh, 0.10, 0.55, 0.028);
                for i in 0..v.len() {
                    if b[i] > v[i] {
                        v[i] = b[i];
                    }
                }
                v
            },
            resistance: {
                // Centred on `test_field`'s first summit blob (px=0.10,
                // py=0.22) so the outcrop has genuine positive TPI, not just
                // a geology-map anomaly with no topographic expression.
                // sigma=1 cell: within `r_fine` (2 cells at this fixture's
                // resolution) the value has already fallen most of the way
                // back to baseline, so the contrast is real, not smoothed
                // away by too wide a window -- the failure
                // `rock_formation_needs_contrast_not_just_high_resistance`
                // pins directly.
                let (cx, cy) = ((0.10 * gw as f64).round(), (0.22 * gh as f64).round());
                let mut v = vec![0f32; gw * gh];
                for y in 0..gh {
                    for x in 0..gw {
                        let d2 = (x as f64 - cx).powi(2) + (y as f64 - cy).powi(2);
                        v[y * gw + x] = (0.10 + 0.85 * (-d2 / 2.0).exp()) as f32;
                    }
                }
                v
            },
            settlements: towns(gw, gh)
                .iter()
                .map(|(x, y, pop)| LandmarkSite {
                    x: *x,
                    y: *y,
                    population: *pop as f64,
                })
                .collect(),
            ways,
        }
    }

    fn inputs<'a>(w: &'a World, res: &'a [(&'a str, &'a [f32])]) -> LandmarkInputs<'a> {
        let mut i = LandmarkInputs::new(&w.field, w.gw, w.gh, SEA, false, w.width_km);
        i.peak_m = PEAK_M;
        i.flow = Some(&w.flow);
        i.channel = Some(&w.chan);
        i.recv = Some(&w.recv);
        i.order = Some(&w.order);
        i.water = Some(&w.water);
        i.corridors = Some(&w.corridors);
        i.resistance = Some(&w.resistance);
        i.resources = res;
        i.settlements = &w.settlements;
        i.ways = &w.ways;
        i
    }

    /// The whole shipped civilisation pipeline, on a world
    /// `cartalith_engine::generate_terrain` really produced — not this
    /// module's hand-shaped fixture.
    ///
    /// `compute_civilisation` (`cartalith-godot/src/lib.rs`) is a private
    /// function in a cdylib, so its call sequence is reproduced here.
    ///
    /// *This cited `examples/_peakaudit_peak.rs` as a second place doing the
    /// same thing until 2026-09-03, when that probe and its two siblings were
    /// deleted with the memory audit they were built for. This is now the only
    /// reproduction of that sequence, which is worth knowing before editing
    /// it: nothing else will disagree with you if it drifts.*
    struct RealWorld {
        gw: usize,
        gh: usize,
        width_km: f64,
        sea: f64,
        world: bool,
        ws: cartalith_engine::WorldState,
        wb: Vec<u8>,
        corridors: Vec<f32>,
        lithology: Vec<u8>,
        order: Vec<i16>,
        sites: Vec<LandmarkSite>,
        ways: Vec<crate::Way>,
        resources: crate::ResourcePotentials,
    }

    fn real_world(gw: usize, gh: usize, seed: i32) -> RealWorld {
        let mut p = cartalith_engine::WorldParams::defaults(gw, gh, seed);
        p.world = false;
        let ws = cartalith_engine::generate_terrain(&p);
        let (sea, world, width_km) = (ws.sea_level, p.world, p.map_width_km);
        let wb = crate::build_water_bodies(&ws.field, gw, gh, sea, world, Some(&ws.rainfall));
        let biome = crate::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
        let soil_slope = crate::build_slope_field(&ws.field, gw, gh, world);
        let lithology = crate::build_lithology(
            &ws.field,
            &ws.age_field,
            &ws.volcanic_field,
            &ws.crust_field,
            &ws.resistance_field,
            &ws.rainfall,
            sea,
        );
        let soil = crate::build_soil_fertility(
            &lithology,
            &ws.temperature,
            &ws.rainfall,
            &soil_slope,
            &ws.age_field,
        );
        let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, width_km);
        let water_access =
            crate::build_water_access(&ws.flow_discharge, &ws.field, gw, gh, sea, flow_thresh);
        let carrying_cap = crate::build_carrying_capacity(
            &soil,
            &water_access,
            Some(&biome),
            &ws.temperature,
            &ws.field,
            sea,
            0.0,
            None,
        );
        let resources = crate::build_resource_potentials(
            &lithology,
            Some(&ws.boundary_type),
            Some(&ws.shear_field),
            Some(&ws.flow_discharge),
            Some(&biome),
            &ws.field,
            &ws.rainfall,
            &ws.age_field,
            gw,
            gh,
            sea,
            Some(&ws.volcanic_field),
            true,
            false,
        );
        let raw_slope = crate::build_raw_slope_field(&ws.field, gw, gh, world);
        let corridors = crate::build_route_corridors(
            &ws.field,
            &raw_slope,
            Some(&ws.flow_discharge),
            gw,
            gh,
            sea,
            world,
            flow_thresh,
        );
        let landmass =
            crate::build_landmass_quality(&ws.field, Some(&carrying_cap), gw, gh, sea, world);
        let coast_sdf = crate::build_coast_sdf(&ws.field, gw, gh, sea);
        let flood =
            crate::build_flood_field(&ws.field, &ws.flow_discharge, &raw_slope, gw, gh, sea);
        let order = crate::fresh_river_order(
            &ws.field,
            &ws.flow_discharge,
            gw,
            gh,
            sea,
            world,
            p.river_density,
            width_km,
        );
        let ctx = crate::SuitabilityCtx {
            water_bodies: Some(&wb.classification),
            corridor: Some(&corridors),
            landmass: Some(&landmass.quality),
            flow: Some(&ws.flow_discharge),
            river_order: Some(&order),
            coast_sdf: Some(&coast_sdf),
            resources: Some(&resources),
            rain: Some(&ws.rainfall),
            flood: Some(&flood),
            slope_raw: Some(&raw_slope),
            flow_thresh,
        };
        let suit = crate::build_settlement_suitability(
            &soil,
            &water_access,
            &carrying_cap,
            &ws.field,
            &soil_slope,
            gw,
            gh,
            sea,
            Some(&ctx),
        );
        let seeds = crate::find_settlement_seeds(
            &suit,
            gw,
            gh,
            0.42,
            (gw as f64 / 22.0).floor().max(6.0),
        );
        let placements = crate::place_settlements_with_water_edge_snap(
            &seeds,
            &suit,
            &ws.field,
            &wb.classification,
            &wb.fill_level,
            gw,
            gh,
            sea,
            world,
            6,
            &flood,
            &ws.flow_discharge,
            flow_thresh,
            width_km,
        );
        let topology = crate::civ_hierarchical_network_topology(
            &placements,
            gw,
            gh,
            sea,
            &ws.field,
            &ws.flow_discharge,
            &order,
            &biome,
            &wb.classification,
            world,
            width_km,
        );
        let mut rng = crate::civ_name_rng();
        let settlements = crate::name_and_populate_settlements_with_rng(&placements, &mut rng);
        let ways = crate::civ_consolidate_and_smooth_ways(
            &topology,
            &settlements,
            &ws.field,
            &wb.classification,
            gw,
            gh,
            width_km,
        );
        let sites = settlements
            .iter()
            .map(|s| LandmarkSite {
                x: s.placement.x,
                y: s.placement.y,
                population: s.pop as f64,
            })
            .collect();
        RealWorld {
            gw,
            gh,
            width_km,
            sea,
            world,
            wb: wb.classification,
            corridors,
            lithology,
            order,
            sites,
            ways,
            resources,
            ws,
        }
    }

    fn real_resource_pairs(rp: &crate::ResourcePotentials) -> Vec<(&str, &[f32])> {
        vec![
            ("copper", rp.copper.as_slice()),
            ("tin", rp.tin.as_slice()),
            ("iron", rp.iron.as_slice()),
            ("gold", rp.gold.as_slice()),
            ("salt", rp.salt.as_slice()),
            ("timber", rp.timber.as_slice()),
            ("lead", rp.lead.as_slice()),
            ("silver", rp.silver.as_slice()),
            ("gems", rp.gems.as_slice()),
            ("clay", rp.clay.as_slice()),
            ("buildstone", rp.buildstone.as_slice()),
            ("flint", rp.flint.as_slice()),
            ("obsidian", rp.obsidian.as_slice()),
            ("sulfur", rp.sulfur.as_slice()),
            ("alum", rp.alum.as_slice()),
        ]
    }

    fn real_inputs<'a>(
        r: &'a RealWorld,
        res: &'a [(&'a str, &'a [f32])],
    ) -> LandmarkInputs<'a> {
        let mut i =
            LandmarkInputs::new(&r.ws.field, r.gw, r.gh, r.sea, r.world, r.width_km);
        i.flow = Some(&r.ws.flow_discharge);
        i.channel = r.ws.channels.as_ref().map(|c| c.chan.as_slice());
        i.recv = r.ws.channels.as_ref().map(|c| c.recv.as_slice());
        i.order = Some(&r.order);
        i.water = Some(&r.wb);
        i.corridors = Some(&r.corridors);
        i.lithology = Some(&r.lithology);
        i.volcanism = Some(&r.ws.volcanic_field);
        i.resistance = Some(&r.ws.resistance_field);
        i.resources = res;
        i.settlements = &r.sites;
        i.ways = &r.ways;
        i
    }

    // -- the table ----------------------------------------------------------

    #[test]
    fn the_kind_table_matches_the_research_and_the_design() {
        let ks = kinds();
        assert_eq!(ks.len(), 49, "§29 lists 49 types");
        let mut seen = std::collections::HashSet::new();
        for k in ks {
            assert!(seen.insert(k.key), "duplicate key {}", k.key);
            assert!(!k.label.is_empty());
            if k.buildable {
                assert!(k.not_built.is_empty(), "{} is built and needs no excuse", k.key);
            } else {
                assert!(!k.not_built.is_empty(), "{} must say why it is not built", k.key);
            }
        }
        // LANDMARK_UI_DESIGN.md §3: Physical 15, Transportation 8, Economic 6,
        // Military 6, Religious/Cultural 8, Historical 6.
        for (fam, want) in [
            (LandmarkFamily::Physical, 15),
            (LandmarkFamily::Transport, 8),
            (LandmarkFamily::Economic, 6),
            (LandmarkFamily::Military, 6),
            (LandmarkFamily::Cultural, 8),
            (LandmarkFamily::Historical, 6),
        ] {
            let n = ks.iter().filter(|k| k.family == fam).count();
            assert_eq!(n, want, "{:?} should have {} types", fam, want);
        }
        // §9.3 names exactly six viewshed-dependent types, by name: Peak,
        // Volcanic feature, Watchtower, Fort, Sacred mountain, Border marker.
        let mut vs: Vec<&str> = ks.iter().filter(|k| k.needs_viewshed).map(|k| k.key).collect();
        vs.sort_unstable();
        assert_eq!(
            vs,
            vec![
                "border_marker",
                "fort",
                "peak",
                "sacred_mountain",
                "volcanic_feature",
                "watchtower",
            ],
            "§9.3's six viewshed-dependent types"
        );
        let mut built: Vec<&str> = ks.iter().filter(|k| k.buildable).map(|k| k.key).collect();
        built.sort_unstable();
        assert_eq!(
            built,
            vec![
                "bridge_site",
                "caravan_station",
                "cliff",
                "ford",
                "gorge",
                "harbour",
                "lake",
                "market_site",
                "mine",
                "mountain_pass",
                "peak",
                "quarry",
                "resource_extraction_site",
                "ridge",
                "river_confluence",
                "road_junction",
                "rock_formation",
                "spring",
                "trade_depot",
                "waterfall",
            ],
            "the twenty kinds this engine actually generates"
        );
        // Every buildable key must have a detector, and no non-buildable key
        // may have one — otherwise the table and the pass disagree about what
        // is honest.
        for k in ks {
            assert_eq!(
                k.buildable,
                matches!(
                    k.key,
                    "waterfall"
                        | "spring"
                        | "river_confluence"
                        | "lake"
                        | "mountain_pass"
                        | "peak"
                        | "ridge"
                        | "cliff"
                        | "gorge"
                        | "rock_formation"
                        | "mine"
                        | "quarry"
                        | "resource_extraction_site"
                        | "harbour"
                        | "ford"
                        | "road_junction"
                        | "bridge_site"
                        | "market_site"
                        | "caravan_station"
                        | "trade_depot"
                ),
                "{} disagrees with `detect`",
                k.key
            );
        }
    }

    #[test]
    fn default_settings_arm_exactly_the_buildable_kinds() {
        let s = LandmarkSettings::default();
        for k in kinds() {
            assert_eq!(s.is_armed(k.key), k.buildable, "{}", k.key);
            assert_eq!(s.cap(k.key), k.default_cap, "{}", k.key);
        }
        assert_eq!(s.crowding, 1.0);
        assert_eq!(s.class_radius_km, DEFAULT_CLASS_RADIUS_KM);
        // The design's own artboards draw 34 km for a regional landmark.
        assert_eq!(s.radius_km(LandmarkClass::Regional), 34.0);
    }

    #[test]
    fn an_unknown_key_falls_back_rather_than_panicking() {
        let s = LandmarkSettings::default();
        assert_eq!(s.cap("no_such_kind"), 0);
        assert!(!s.is_armed("no_such_kind"));
        assert!(kind_spec("no_such_kind").is_none());
        assert!(kind_spec("waterfall").is_some());
    }

    // -- the analysis module ------------------------------------------------

    #[test]
    fn relief_agrees_with_the_analysis_module() {
        for world_wrap in [false, true] {
            let (gw, gh) = (48usize, 32usize);
            let f = test_field(gw, gh);
            for r in [2i64, 5, 9] {
                let (lo, hi) = sep_min_max(&f, gw, gh, r, world_wrap);
                let lr = analysis::local_relief(&f, gw, gh, r, world_wrap);
                for i in 0..f.len() {
                    let mine = hi[i] - lo[i];
                    assert!(
                        (mine - lr[i]).abs() < 1e-6,
                        "cell {} r {} wrap {}: {} vs {}",
                        i,
                        r,
                        world_wrap,
                        mine,
                        lr[i]
                    );
                }
            }
        }
    }

    // -- the funnel ---------------------------------------------------------

    #[test]
    fn funnel_arithmetic_closes_on_every_kind() {
        let w = world(256, 192, 1000.0);
        let res: Vec<(&str, &[f32])> = vec![
            ("iron", w.iron.as_slice()),
            ("buildstone", w.stone.as_slice()),
            ("timber", w.timber.as_slice()),
        ];
        let inp = inputs(&w, &res);
        let r = generate(&inp, &LandmarkSettings::default(), 12345);
        assert_eq!(r.funnels.len(), kinds().len());
        for f in &r.funnels {
            // Printed rather than only asserted: `cargo test -- --nocapture`
            // then shows the whole funnel table, which is the one artefact a
            // reader can check this pass against by eye.
            if f.candidates > 0 || f.placed > 0 {
                println!(
                    "{:24} cand {:6} = con {:6} + sco {:6} + spa {:5} + cap {:5} + placed {:4}   cap {:4}  {}",
                    f.kind,
                    f.candidates,
                    f.rejected_constraint,
                    f.rejected_score,
                    f.rejected_spacing,
                    f.rejected_cap,
                    f.placed,
                    f.cap,
                    f.limit.as_str()
                );
            }
            assert!(
                f.closes(),
                "{}: {} != {} + {} + {} + {} + {}",
                f.kind,
                f.candidates,
                f.rejected_constraint,
                f.rejected_score,
                f.rejected_spacing,
                f.rejected_cap,
                f.placed
            );
            // **The fifth bucket must actually separate the two meanings.**
            // A kind that hit its cap and had survivors left over must report
            // them under `rejected_cap` and NOT under `rejected_score` -- the
            // whole reason the bucket exists is that "you asked for fewer" is
            // not "these were not good enough".
            if f.rejected_cap > 0 {
                assert_eq!(
                    f.limit,
                    LandmarkLimit::AtCap,
                    "{}: turned {} away on the cap but reports {}",
                    f.kind,
                    f.rejected_cap,
                    f.limit.as_str()
                );
                assert!(
                    f.placed >= f.cap,
                    "{}: cap-rejected {} while below its own cap ({} of {})",
                    f.kind,
                    f.rejected_cap,
                    f.placed,
                    f.cap
                );
            }
        }
        // The identity is only interesting if the funnels are not all empty.
        let total: usize = r.funnels.iter().map(|f| f.candidates).sum();
        assert!(total > 0, "no kind produced a single candidate");
    }

    #[test]
    fn a_real_world_places_landmarks_of_several_kinds() {
        let w = world(256, 192, 1000.0);
        let res: Vec<(&str, &[f32])> = vec![
            ("iron", w.iron.as_slice()),
            ("buildstone", w.stone.as_slice()),
            ("timber", w.timber.as_slice()),
        ];
        let inp = inputs(&w, &res);
        let r = generate(&inp, &LandmarkSettings::default(), 7);
        // CLAUDE.md's standing rule: a pass that places nothing must fail a
        // test, not pass one.
        assert!(!r.landmarks.is_empty(), "the pass placed nothing at all");
        let kinds_placed: std::collections::BTreeSet<&str> =
            r.landmarks.iter().map(|l| l.kind.as_str()).collect();
        assert!(
            kinds_placed.len() >= 10,
            "only {:?} produced anything on a whole world",
            kinds_placed
        );
        assert!(r.landmarks.len() > 100, "only {} landmarks", r.landmarks.len());
        for l in &r.landmarks {
            assert!((0.0..=1.0).contains(&l.score), "{} score {}", l.kind, l.score);
            assert!(
                (0.0..=1.0).contains(&l.importance),
                "{} importance {}",
                l.kind,
                l.importance
            );
            assert!(l.x < w.gw && l.y < w.gh);
            assert!(l.id > 0);
            assert!(l.causal.len() >= 2, "{} has a bare causal chain", l.kind);
            let spec = kind_spec(&l.kind).expect("placed kind is in the table");
            assert_eq!(l.causal.last().map(String::as_str), Some(spec.label));
            assert!(spec.buildable, "{} is not supposed to be generated", l.kind);
        }
        let ids: std::collections::BTreeSet<u64> = r.landmarks.iter().map(|l| l.id).collect();
        assert_eq!(ids.len(), r.landmarks.len(), "ids are not unique");
        assert!(r.seconds >= 0.0);
    }

    /// The two kinds this file's hand-shaped fixture **cannot** reach, stated
    /// as a constant so no third can join them silently.
    ///
    /// Both want a trunk river, and `test_field` has none: measured on it, the
    /// largest channel anywhere carries 18.5x the channel-initiation threshold
    /// and the largest a routed way crosses carries 4.6x. Bridge site needs
    /// more than [`FORD_MAX_FLOW_MULT`] (8x) at a crossing and Trade depot
    /// needs [`DEPOT_NAVIGABLE_FLOW_MULT`] (20x) beside one, so neither can
    /// fire there.
    ///
    /// **Bending the fixture or the constants until they did would be
    /// backwards.** Both are verified instead where the shell will really run
    /// them, by
    /// `every_way_graph_kind_places_on_a_world_generate_terrain_really_made`,
    /// and that test asserts exactly this list — so a kind cannot be exempted
    /// here and left unverified there.
    const FIXTURE_HAS_NO_TRUNK_RIVER: [&str; 2] = ["bridge_site", "trade_depot"];

    /// **Every detector must be able to place something.** Run with cross-type
    /// competition off, because with it on a gorge and a waterfall at the same
    /// river crossing are one landmark by design and the second reports
    /// `spacing` — which is correct behaviour and would otherwise hide a
    /// detector that never fires at all.
    #[test]
    fn every_buildable_kind_can_actually_place_one() {
        let w = world(256, 192, 1000.0);
        let res: Vec<(&str, &[f32])> = vec![
            ("iron", w.iron.as_slice()),
            ("buildstone", w.stone.as_slice()),
            ("timber", w.timber.as_slice()),
        ];
        let inp = inputs(&w, &res);
        let s = LandmarkSettings { cross_type_competition: false, ..Default::default() };
        let r = generate(&inp, &s, 7);
        for k in kinds().iter().filter(|k| k.buildable) {
            if FIXTURE_HAS_NO_TRUNK_RIVER.contains(&k.key) {
                continue;
            }
            assert!(
                r.placed(k.key) > 0,
                "{} placed nothing: {:?}",
                k.key,
                r.funnel(k.key)
            );
        }
    }

    /// **The five kinds M8 closed, on a world nobody shaped for them.**
    ///
    /// `real_world` reproduces `compute_civilisation`'s own call sequence over
    /// `cartalith_engine::generate_terrain`'s output, so the way graph these
    /// read is the one the shell hands them — real settlement placement, real
    /// MST/Dijkstra routing, real busiest-first consolidation. A flag flipped
    /// to `buildable` on a kind that then places nothing is the failure this
    /// project keeps `UNWIRED_FUNCTIONS.md` for; this is what stops it.
    #[test]
    fn every_way_graph_kind_places_on_a_world_generate_terrain_really_made() {
        let r = real_world(256, 192, 24601);
        assert!(r.sites.len() >= 5, "only {} settlements placed", r.sites.len());
        assert!(r.ways.len() >= 5, "only {} ways routed", r.ways.len());
        let pairs = real_resource_pairs(&r.resources);
        let inp = real_inputs(&r, &pairs);
        let s = LandmarkSettings { cross_type_competition: false, ..Default::default() };
        let out = generate(&inp, &s, 7);
        for key in
            ["road_junction", "bridge_site", "market_site", "caravan_station", "trade_depot"]
        {
            let f = out.funnel(key).expect("every kind has a funnel");
            println!("{:18} placed {:3} of {:3}  {:?}", key, f.placed, f.cap, f);
            assert!(f.placed > 0, "{} placed nothing on a real world: {:?}", key, f);
        }
        // The exemption above may not hide a kind from this test.
        for key in FIXTURE_HAS_NO_TRUNK_RIVER {
            assert!(out.placed(key) > 0, "{} is exempted and unverified", key);
        }
        // Every placement is on a cell a way really runs through — the domain
        // all five share, and the one claim a rasterised road mask could not
        // have made.
        let g = WayGrid::build(&r.ways, r.gw, r.gh, r.world);
        for l in out.landmarks.iter().filter(|l| {
            matches!(
                l.kind.as_str(),
                "road_junction"
                    | "bridge_site"
                    | "market_site"
                    | "caravan_station"
                    | "trade_depot"
            )
        }) {
            assert!(
                g.ways_here[l.y * r.gw + l.x] > 0,
                "{} at ({}, {}) is not on the way network",
                l.kind,
                l.x,
                l.y
            );
        }
    }

    /// Ford and Bridge site **partition** the channel cells a way crosses;
    /// they never both claim one. That is the whole content of
    /// [`FORD_MAX_FLOW_MULT`] being read from both sides, and the reason
    /// `river_crossing` stays unbuilt.
    #[test]
    fn a_way_crossing_a_channel_is_a_ford_or_a_bridge_and_never_both() {
        let r = real_world(256, 192, 24601);
        let pairs = real_resource_pairs(&r.resources);
        let inp = real_inputs(&r, &pairs);
        let s = LandmarkSettings { cross_type_competition: false, ..Default::default() };
        let out = generate(&inp, &s, 7);
        let cells = |k: &str| -> std::collections::BTreeSet<(usize, usize)> {
            out.landmarks.iter().filter(|l| l.kind == k).map(|l| (l.x, l.y)).collect()
        };
        let fords = cells("ford");
        let bridges = cells("bridge_site");
        assert!(!fords.is_empty() && !bridges.is_empty(), "one of the two never fired");
        assert!(
            fords.intersection(&bridges).next().is_none(),
            "a cell is both a Ford and a Bridge site: {:?}",
            fords.intersection(&bridges).collect::<Vec<_>>()
        );
        // And a Trade depot is never on a channel at all — it is the bank.
        let chan = r.ws.channels.as_ref().expect("a real world has channels");
        for (x, y) in cells("trade_depot") {
            assert_eq!(chan.chan[y * r.gw + x], 0, "a depot sits in the channel at ({x}, {y})");
        }
    }

    /// The honest-degradation contract, for the one input all five share: a
    /// caller that passes no ways gets `NoTerrain` and zero candidates from
    /// every one of them, not a fabricated placement off some other field.
    #[test]
    fn the_way_graph_kinds_degrade_when_no_ways_are_passed() {
        let w = world(256, 192, 1000.0);
        let mut inp = inputs(&w, &[]);
        inp.ways = &[];
        let r = generate(&inp, &LandmarkSettings::default(), 11);
        for key in
            ["road_junction", "bridge_site", "market_site", "caravan_station", "trade_depot"]
        {
            let f = r.funnel(key).expect("every kind has a funnel");
            assert_eq!(f.limit, LandmarkLimit::NoTerrain, "{}: {:?}", key, f);
            assert_eq!(f.candidates, 0, "{}: {:?}", key, f);
            assert_eq!(f.placed, 0, "{}: {:?}", key, f);
        }
        // The rest of the pass is unaffected — an absent way graph is not a
        // reason for the terrain kinds to stop.
        assert!(r.placed("waterfall") + r.placed("peak") > 0);
    }

    /// Market site and Caravan station are the same measurement with opposite
    /// senses, so with settlements gone the first must not simply fall silent
    /// while the second fires **everywhere** on an infinite distance. Both
    /// refuse instead — `pool_trade_road`'s early return.
    #[test]
    fn a_trade_road_kind_refuses_rather_than_fabricating_without_settlements() {
        let w = world(256, 192, 1000.0);
        let mut inp = inputs(&w, &[]);
        inp.settlements = &[];
        let r = generate(&inp, &LandmarkSettings::default(), 11);
        for key in ["market_site", "caravan_station"] {
            let f = r.funnel(key).expect("every kind has a funnel");
            assert_eq!(f.limit, LandmarkLimit::NoTerrain, "{}: {:?}", key, f);
            assert_eq!(f.candidates, 0, "{}: {:?}", key, f);
        }
        // Road junction needs no settlement and must be unaffected.
        assert!(r.placed("road_junction") > 0, "{:?}", r.funnel("road_junction"));
    }

    /// A straight way of a stated class through whole-cell coordinates, for
    /// the two geometry tests below. `civ_consolidate_and_smooth_ways` anchors
    /// a run's ends on `placement.x`/`.y` directly, so whole numbers are the
    /// real coordinate convention rather than a test shortcut.
    fn straight(
        (ax, ay): (usize, usize),
        (bx, by): (usize, usize),
        t: crate::WayType,
    ) -> crate::Way {
        crate::Way {
            tid: 0,
            pts: vec![(ax as f64, ay as f64), (bx as f64, by as f64)],
            brks: Vec::new(),
            km: 0.0,
            name: String::new(),
            way_type: t,
            a_idx: 0,
            b_idx: 1,
            hidden: false,
        }
    }

    /// 128 x 64 at 512 km — a 4 km cell, flat and entirely above sea, so the
    /// only thing under test is the way graph and the distance to the one
    /// settlement. Every terrain kind degrades to nothing here, which is the
    /// point.
    fn flat(gw: usize, gh: usize) -> Vec<f32> {
        vec![0.60f32; gw * gh]
    }

    /// **The 20 km band between [`MARKET_MAX_SETTLEMENT_KM`] and
    /// [`CARAVAN_MIN_SETTLEMENT_KM`] is empty on purpose**, and
    /// [`TRADE_MIN_WAY_RANK`] really is the line at `Road`. Four short ways at
    /// measured distances from one settlement, on a flat map where nothing
    /// else can fire:
    ///
    /// | way | distance | class | expected |
    /// |---|---|---|---|
    /// | A | 16-28 km | road | markets, no stations |
    /// | B | 44-56 km | road | **neither** — the empty band |
    /// | C | 84-96 km | road | stations, no markets |
    /// | D | 23-32 km | track | neither — a track carries no trade |
    #[test]
    fn a_market_sits_inside_a_settlements_reach_and_a_caravan_station_between_two() {
        let (gw, gh, width_km) = (128usize, 64usize, 512.0f64);
        let field = flat(gw, gh);
        let sites = vec![LandmarkSite { x: 10, y: 32, population: 5_000.0 }];
        let ways = vec![
            straight((14, 32), (17, 32), crate::WayType::Road),
            straight((21, 32), (24, 32), crate::WayType::Road),
            straight((31, 32), (34, 32), crate::WayType::Road),
            straight((14, 36), (17, 36), crate::WayType::Track),
        ];
        let mut inp = LandmarkInputs::new(&field, gw, gh, SEA, false, width_km);
        inp.settlements = &sites;
        inp.ways = &ways;
        let s = LandmarkSettings { cross_type_competition: false, ..Default::default() };
        let r = generate(&inp, &s, 3);

        let at = |k: &str| -> Vec<(usize, usize)> {
            r.landmarks.iter().filter(|l| l.kind == k).map(|l| (l.x, l.y)).collect()
        };
        let markets = at("market_site");
        let stations = at("caravan_station");
        assert!(!markets.is_empty(), "no market on a road inside the catchment: {:?}", r.funnel("market_site"));
        assert!(!stations.is_empty(), "no station on the far road: {:?}", r.funnel("caravan_station"));
        for (x, y) in &markets {
            assert_eq!(*y, 32, "a market left way A for the track at y = 36");
            assert!((14..=17).contains(x), "a market at x = {} is outside way A", x);
        }
        for (x, y) in &stations {
            assert_eq!(*y, 32, "a station left way C for the track at y = 36");
            assert!((31..=34).contains(x), "a station at x = {} is outside way C", x);
        }
    }

    /// **A junction is where a branch leaves a trunk, and nowhere else.** One
    /// T on a flat map: 41 cells of trunk and 23 of branch, and exactly one
    /// cell carrying both. [`JUNCTION_MIN_WAYS`] at `1` would make every one
    /// of the 63 a junction; at `3` it would find none.
    #[test]
    fn a_road_junction_is_the_one_cell_two_ways_share() {
        let (gw, gh, width_km) = (128usize, 64usize, 512.0f64);
        let field = flat(gw, gh);
        let sites = vec![LandmarkSite { x: 10, y: 32, population: 5_000.0 }];
        let ways = vec![
            straight((20, 32), (60, 32), crate::WayType::Road),
            straight((40, 32), (40, 10), crate::WayType::Road),
        ];
        let mut inp = LandmarkInputs::new(&field, gw, gh, SEA, false, width_km);
        inp.settlements = &sites;
        inp.ways = &ways;
        let s = LandmarkSettings { cross_type_competition: false, ..Default::default() };
        let r = generate(&inp, &s, 3);
        let junctions: Vec<(usize, usize)> =
            r.landmarks.iter().filter(|l| l.kind == "road_junction").map(|l| (l.x, l.y)).collect();
        assert_eq!(
            junctions,
            vec![(40, 32)],
            "the T's own crossing is the only junction on this map: {:?}",
            r.funnel("road_junction")
        );
        // And the funnel must say the other 62 road cells were road, not
        // junction — the `1` mutant's tell.
        let f = r.funnel("road_junction").unwrap();
        assert!(f.rejected_constraint >= 60, "road cells not counted as road: {:?}", f);
    }

    /// The three flow lines the way-graph kinds draw on one channel, pinned
    /// against a hand-built hydrology so each is a number rather than a
    /// hope: **4x** the channel-initiation threshold is a Ford, **10x** is a
    /// Bridge site, and only **30x** — over
    /// [`DEPOT_NAVIGABLE_FLOW_MULT`]'s 20 — puts a Trade depot on the bank.
    #[test]
    fn ford_bridge_and_depot_split_one_road_by_how_much_water_it_meets() {
        let (gw, gh, width_km) = (128usize, 64usize, 512.0f64);
        let field = flat(gw, gh);
        let thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, width_km);
        let mut chan = vec![0u8; gw * gh];
        let mut flow = vec![0f32; gw * gh];
        // Three north-south channels, at 4x, 10x and 30x the threshold.
        for (x, mult) in [(40usize, 10.0f64), (60, 4.0), (80, 30.0)] {
            for y in 0..gh {
                chan[y * gw + x] = 1;
                flow[y * gw + x] = (thresh * mult) as f32;
            }
        }
        let sites = vec![LandmarkSite { x: 10, y: 32, population: 5_000.0 }];
        let ways = vec![
            straight((30, 32), (90, 32), crate::WayType::Road),
            // A track reaching the same navigable water. A bridge is still a
            // bridge on it — spanning the river is a physical fact — but no
            // depot, because [`TRADE_MIN_WAY_RANK`] says a track carries no
            // trade to tranship.
            straight((70, 40), (90, 40), crate::WayType::Track),
        ];
        let mut inp = LandmarkInputs::new(&field, gw, gh, SEA, false, width_km);
        inp.channel = Some(&chan);
        inp.flow = Some(&flow);
        inp.settlements = &sites;
        inp.ways = &ways;
        let s = LandmarkSettings { cross_type_competition: false, ..Default::default() };
        let r = generate(&inp, &s, 3);
        let at = |k: &str| -> Vec<(usize, usize)> {
            r.landmarks.iter().filter(|l| l.kind == k).map(|l| (l.x, l.y)).collect()
        };

        let fords = at("ford");
        assert!(!fords.is_empty(), "the 4x channel produced no ford: {:?}", r.funnel("ford"));
        for (x, _) in &fords {
            assert_eq!(*x, 60, "a ford on a channel too big to wade");
        }

        let mut bridges = at("bridge_site");
        bridges.sort_unstable();
        assert_eq!(
            bridges,
            vec![(40, 32), (80, 32), (80, 40)],
            "one bridge per unwadeable crossing, on the track as well as the road"
        );

        let depots = at("trade_depot");
        assert!(!depots.is_empty(), "no depot beside the 30x channel: {:?}", r.funnel("trade_depot"));
        for (x, y) in &depots {
            assert_eq!(*y, 32, "a depot on the track at y = 40 — a track carries no trade");
            assert!(
                *x == 79 || *x == 81,
                "a depot at x = {} — only the 30x channel is navigable, and the bank is not the crossing",
                x
            );
        }
    }

    /// [`WayGrid`] counts **distinct polylines**, not stroke pixels — the
    /// claim `road_junction`'s old `not_built` reason made a condition of
    /// being built at all. Pinned on a way that doubles back through one cell:
    /// it must count once, and a second way through it must count twice.
    #[test]
    fn a_way_grid_counts_polylines_and_not_pixels() {
        let way = |pts: Vec<(f64, f64)>, t: crate::WayType, hidden: bool| crate::Way {
            tid: 0,
            pts,
            brks: Vec::new(),
            km: 0.0,
            name: String::new(),
            way_type: t,
            a_idx: 0,
            b_idx: 1,
            hidden,
        };
        // One way that leaves cell (2,2), turns around and comes back.
        let doubling = way(
            vec![(2.5, 2.5), (6.5, 2.5), (2.5, 2.5)],
            crate::WayType::Track,
            false,
        );
        let g = WayGrid::build(std::slice::from_ref(&doubling), 8, 8, false);
        assert_eq!(g.ways_here[2 * 8 + 2], 1, "one polyline counted twice");
        assert_eq!(g.rank[2 * 8 + 2], 0, "a track is rank 0");
        // A second, busier way through the same cell counts separately and
        // raises the rank; a hidden third is not on the map and counts not at
        // all.
        let g = WayGrid::build(
            &[
                doubling.clone(),
                way(vec![(2.5, 0.5), (2.5, 6.5)], crate::WayType::Highway, false),
                way(vec![(2.5, 0.5), (2.5, 6.5)], crate::WayType::Highway, true),
            ],
            8,
            8,
            false,
        );
        assert_eq!(g.ways_here[2 * 8 + 2], 2, "two visible ways, and only two");
        assert_eq!(g.rank[2 * 8 + 2], 3, "the busiest way on the cell sets the rank");
    }

    /// **The whole justification for closing `rock_formation`'s `not_built`
    /// reason rather than just relaxing it**: uniformly hard rock is not a
    /// formation, only a hard patch standing out from its own neighbourhood
    /// is — the neighbour-contrast the reason named as missing even with
    /// resistance wired in. Pinned directly, by mutation of the fixture
    /// rather than trusted from the doc comment: flatten the contrast and
    /// the kind must fall silent, restore it and it must fire again.
    #[test]
    fn rock_formation_needs_contrast_not_just_high_resistance() {
        let w = world(256, 192, 1000.0);
        // Off for the same reason `every_buildable_kind_can_actually_place_one`
        // turns it off: this fixture's outcrop sits on the same summit Peak
        // and Ridge also read, and cross-type competition would let whichever
        // of them is processed first claim the cell before Rock formation's
        // own turn — testing spacing order, not the contrast constraint.
        let s = LandmarkSettings { cross_type_competition: false, ..Default::default() };

        // Uniformly high resistance: every land cell ties for its own
        // window's maximum, and none of them contrast with it.
        let flat = vec![0.95f32; w.gw * w.gh];
        let mut inp = inputs(&w, &[]);
        inp.resistance = Some(&flat);
        let r = generate(&inp, &s, 5);
        assert_eq!(
            r.placed("rock_formation"),
            0,
            "uniformly hard ground placed a formation with no contrast to stand out from"
        );
        let f = r.funnel("rock_formation").unwrap();
        assert!(
            f.rejected_constraint > 0,
            "nothing even reached the contrast test: {:?}",
            f
        );

        // The fixture's own resistance field — one outcrop standing out from
        // a low background, over real topography — places at least one.
        let inp2 = inputs(&w, &[]);
        let r2 = generate(&inp2, &s, 5);
        assert!(
            r2.placed("rock_formation") > 0,
            "an outcrop with real contrast and real relief placed nothing: {:?}",
            r2.funnel("rock_formation")
        );
    }

    /// **The M8 residual's whole justification for `resource_extraction_site`
    /// being buildable at all**: it reads a disjoint resource set from Mine
    /// and Quarry, so it is a new kind rather than a second record on cells
    /// they already claim. Pinned directly rather than trusted from the doc
    /// comment.
    #[test]
    fn resource_extraction_site_reads_a_disjoint_resource_set_from_mine_and_quarry() {
        let w = world(256, 192, 1000.0);
        // Only what Mine and Quarry read — no timber, no sulfur, no alum.
        let mine_quarry_only: Vec<(&str, &[f32])> =
            vec![("iron", w.iron.as_slice()), ("buildstone", w.stone.as_slice())];
        let inp = inputs(&w, &mine_quarry_only);
        let r = generate(&inp, &LandmarkSettings::default(), 21);
        let f = r.funnel("resource_extraction_site").unwrap();
        assert_eq!(
            f.limit,
            LandmarkLimit::NoTerrain,
            "resource_extraction_site fired on Mine/Quarry's own resources: {:?}",
            f
        );
        assert_eq!(f.candidates, 0);
        assert_eq!(r.placed("resource_extraction_site"), 0);

        // The same world, with only its residual resource added, places one —
        // proof the kind is not simply dead code that happens to pass
        // `every_buildable_kind_can_actually_place_one` through some other
        // fixture field.
        let with_timber: Vec<(&str, &[f32])> = vec![("timber", w.timber.as_slice())];
        let inp2 = inputs(&w, &with_timber);
        let r2 = generate(&inp2, &LandmarkSettings::default(), 21);
        assert!(
            r2.placed("resource_extraction_site") > 0,
            "resource_extraction_site placed nothing off timber alone: {:?}",
            r2.funnel("resource_extraction_site")
        );
        // And it must not have borrowed a Mine/Quarry cell to do it — Mine and
        // Quarry see no resources of their own in `with_timber` and must stay
        // at zero.
        assert_eq!(r2.placed("mine"), 0);
        assert_eq!(r2.placed("quarry"), 0);
    }

    #[test]
    fn causal_chains_carry_measured_values_not_a_fixed_string_per_kind() {
        let w = world(256, 192, 1000.0);
        let inp = inputs(&w, &[]);
        let s = LandmarkSettings { cross_type_competition: false, ..Default::default() };
        let r = generate(&inp, &s, 3);
        let falls: Vec<&Landmark> =
            r.landmarks.iter().filter(|l| l.kind == "waterfall").collect();
        assert!(falls.len() >= 2, "need two waterfalls to compare, got {}", falls.len());
        let a = &falls[0].causal;
        let b = &falls[1].causal;
        assert_ne!(a, b, "two waterfalls share one canned chain: {:?}", a);
        assert!(
            a.iter().any(|s| s.contains("drop") && s.contains(" m")),
            "no measured drop in {:?}",
            a
        );
    }

    // -- determinism (§27) --------------------------------------------------

    #[test]
    fn two_runs_of_the_same_world_are_identical() {
        let w = world(192, 144, 1000.0);
        let res: Vec<(&str, &[f32])> = vec![("iron", w.iron.as_slice())];
        let inp = inputs(&w, &res);
        let s = LandmarkSettings::default();
        let a = generate(&inp, &s, 999);
        let b = generate(&inp, &s, 999);
        assert_eq!(a.landmarks, b.landmarks, "landmark list is not reproducible");
        assert_eq!(a.funnels, b.funnels, "funnels are not reproducible");
        assert_eq!(a.rejects, b.rejects, "the rejected-candidate list is not reproducible");
        assert!(!a.landmarks.is_empty());
    }

    #[test]
    fn a_landmark_seed_survives_a_cap_change() {
        let w = world(192, 144, 1000.0);
        let inp = inputs(&w, &[]);
        let mut s = LandmarkSettings::default();
        s.set_cap("peak", 4);
        let a = generate(&inp, &s, 41);
        s.set_cap("peak", 8);
        let b = generate(&inp, &s, 41);
        let first: Vec<(usize, usize, u64)> = a
            .landmarks
            .iter()
            .filter(|l| l.kind == "peak")
            .map(|l| (l.x, l.y, l.seed))
            .collect();
        assert!(!first.is_empty());
        for (x, y, seed) in first {
            let same = b
                .landmarks
                .iter()
                .find(|l| l.kind == "peak" && l.x == x && l.y == y)
                .expect("a peak placed at cap 4 is still placed at cap 8");
            assert_eq!(same.seed, seed, "§27's seed moved when only the cap did");
        }
    }

    // -- honest degradation -------------------------------------------------

    #[test]
    fn a_kind_whose_input_is_absent_reports_no_terrain_and_places_none() {
        let w = world(192, 144, 1000.0);
        // Height only: no channels, no water bodies, no corridors, no ore.
        let mut inp = LandmarkInputs::new(&w.field, w.gw, w.gh, SEA, false, w.width_km);
        inp.peak_m = PEAK_M;
        let r = generate(&inp, &LandmarkSettings::default(), 5);
        for key in ["waterfall", "spring", "river_confluence", "ford", "gorge"] {
            let f = r.funnel(key).expect("every kind has a funnel");
            assert_eq!(f.limit, LandmarkLimit::NoTerrain, "{}", key);
            assert_eq!(f.candidates, 0, "{}", key);
            assert_eq!(f.placed, 0, "{}", key);
        }
        for key in ["lake", "harbour"] {
            assert_eq!(r.funnel(key).unwrap().limit, LandmarkLimit::NoTerrain, "{}", key);
        }
        for key in ["mountain_pass", "mine", "quarry", "resource_extraction_site"] {
            let f = r.funnel(key).unwrap();
            assert_eq!(f.limit, LandmarkLimit::NoTerrain, "{}", key);
            assert_eq!(f.placed, 0, "{}", key);
        }
        assert!(
            r.landmarks.iter().any(|l| l.kind == "peak"),
            "peak needs only the heightfield and should still place"
        );
        assert!(
            !r.landmarks.iter().any(|l| l.kind == "waterfall"),
            "a waterfall was invented without a river"
        );
    }

    #[test]
    fn a_wrongly_sized_optional_input_degrades_rather_than_panicking() {
        let w = world(128, 96, 1000.0);
        let short: Vec<u8> = vec![1; 10];
        let mut inp = LandmarkInputs::new(&w.field, w.gw, w.gh, SEA, false, w.width_km);
        inp.peak_m = PEAK_M;
        inp.channel = Some(&short);
        inp.flow = Some(&w.flow);
        let r = generate(&inp, &LandmarkSettings::default(), 1);
        assert_eq!(r.funnel("waterfall").unwrap().limit, LandmarkLimit::NoTerrain);
    }

    #[test]
    fn a_disarmed_or_unbuildable_kind_says_so() {
        let w = world(128, 96, 1000.0);
        let inp = inputs(&w, &[]);
        let mut s = LandmarkSettings::default();
        s.set_armed("peak", false);
        let r = generate(&inp, &s, 1);
        assert_eq!(r.funnel("peak").unwrap().limit, LandmarkLimit::Disarmed);
        assert_eq!(r.placed("peak"), 0);
        assert_eq!(r.funnel("shrine").unwrap().limit, LandmarkLimit::NotBuildable);
        assert_eq!(r.funnel("fort").unwrap().limit, LandmarkLimit::NotBuildable);
        // A cap of zero is the slider's own `off` stop.
        let mut s2 = LandmarkSettings::default();
        s2.set_cap("peak", 0);
        let r2 = generate(&inp, &s2, 1);
        assert_eq!(r2.funnel("peak").unwrap().limit, LandmarkLimit::Disarmed);
    }

    #[test]
    fn an_arming_a_kind_this_engine_does_not_build_still_reports_not_buildable() {
        let w = world(128, 96, 1000.0);
        let inp = inputs(&w, &[]);
        let mut s = LandmarkSettings::default();
        s.set_armed("fort", true);
        let r = generate(&inp, &s, 1);
        assert_eq!(r.funnel("fort").unwrap().limit, LandmarkLimit::NotBuildable);
        assert_eq!(r.placed("fort"), 0);
    }

    #[test]
    fn degenerate_grids_do_not_panic() {
        let s = LandmarkSettings::default();
        // Empty.
        let empty: Vec<f32> = Vec::new();
        let inp = LandmarkInputs::new(&empty, 0, 0, SEA, false, 200.0);
        let r = generate(&inp, &s, 1);
        assert!(r.landmarks.is_empty());
        assert_eq!(r.funnels.len(), kinds().len());
        // One cell.
        let one = vec![0.9f32];
        let inp = LandmarkInputs::new(&one, 1, 1, SEA, false, 200.0);
        let r = generate(&inp, &s, 1);
        assert_eq!(r.funnels.len(), kinds().len());
        for f in &r.funnels {
            assert!(f.closes(), "{}", f.kind);
        }
        // All ocean.
        let sea = vec![0.05f32; 32 * 24];
        let inp = LandmarkInputs::new(&sea, 32, 24, SEA, false, 200.0);
        let r = generate(&inp, &s, 1);
        assert!(r.landmarks.is_empty(), "an all-ocean world grew landmarks");
        for f in &r.funnels {
            assert!(f.closes(), "{}", f.kind);
        }
        // A field of the wrong length for its declared size.
        let wrong = vec![0.5f32; 7];
        let inp = LandmarkInputs::new(&wrong, 32, 24, SEA, false, 200.0);
        let r = generate(&inp, &s, 1);
        assert!(r.landmarks.is_empty());
        // A zero map width, which makes every km conversion meaningless.
        let f = test_field(32, 24);
        let inp = LandmarkInputs::new(&f, 32, 24, SEA, false, 0.0);
        let r = generate(&inp, &s, 1);
        for fu in &r.funnels {
            assert!(fu.closes(), "{}", fu.kind);
        }
    }

    // -- spacing (§16) ------------------------------------------------------

    /// Two cones on a flat plain, sixteen cells apart. At `width_km = 128` on a
    /// 64-wide grid a cell is 2 km, so the separation is 32 km — inside the
    /// 34 km a Regional landmark keeps clear, and outside the broad analysis
    /// window, so both summits are genuinely local maxima and genuinely
    /// candidates. Quantised on purpose: the whole point is a fixture shaped to
    /// reach the code rather than a random world that might.
    fn two_cones() -> (Vec<f32>, usize, usize, f64) {
        let (gw, gh) = (64usize, 48usize);
        let mut f = vec![0.50f32; gw * gh];
        for (cx, cy, amp) in [(16usize, 24usize, 0.30f64), (32, 24, 0.20)] {
            for y in 0..gh {
                for x in 0..gw {
                    let d2 =
                        ((x as f64 - cx as f64).powi(2) + (y as f64 - cy as f64).powi(2)) / 8.0;
                    let v = 0.50 + amp * (-d2).exp();
                    if v as f32 > f[y * gw + x] {
                        f[y * gw + x] = v as f32;
                    }
                }
            }
        }
        (f, gw, gh, 128.0)
    }

    fn only_peaks(cap: u32) -> LandmarkSettings {
        let mut s = LandmarkSettings::default();
        for k in kinds() {
            s.set_armed(k.key, k.key == "peak");
        }
        s.set_cap("peak", cap);
        s
    }

    #[test]
    fn spacing_rejects_the_weaker_of_two_candidates_inside_one_radius() {
        let (f, gw, gh, width) = two_cones();
        let mut inp = LandmarkInputs::new(&f, gw, gh, SEA, false, width);
        inp.peak_m = PEAK_M;
        let r = generate(&inp, &only_peaks(5), 1);
        let fu = r.funnel("peak").unwrap();
        assert_eq!(fu.placed, 1, "both cones were placed inside one radius");
        assert_eq!(fu.rejected_spacing, 1, "the loser was not counted as spacing");
        assert_eq!(fu.limit, LandmarkLimit::Spacing);
        assert!(fu.closes());
        // The one placed is the taller: §16 is a competition, and the stronger
        // candidate must win it.
        let p = r.landmarks.iter().find(|l| l.kind == "peak").unwrap();
        assert_eq!((p.x, p.y), (16, 24), "the weaker cone won the competition");
    }

    #[test]
    fn at_cap_and_spacing_are_different_answers() {
        let (f, gw, gh, width) = two_cones();
        let mut inp = LandmarkInputs::new(&f, gw, gh, SEA, false, width);
        inp.peak_m = PEAK_M;

        // Cap 1: the cap binds, even though spacing would also have.
        let at_cap = generate(&inp, &only_peaks(1), 1);
        let a = at_cap.funnel("peak").unwrap();
        assert_eq!(a.limit, LandmarkLimit::AtCap);
        assert_eq!(a.placed, 1);
        assert_eq!(a.cap, 1);
        assert!(a.closes());

        // Cap 5 with the radius shrunk until both fit: neither the cap nor
        // spacing binds, so the honest answer is that the world ran out.
        let mut wide = only_peaks(5);
        wide.crowding = 3.0; // radius / 3 = 11.3 km, under the 32 km gap
        let loose = generate(&inp, &wide, 1);
        let b = loose.funnel("peak").unwrap();
        assert_eq!(b.placed, 2, "both cones should fit once packed tighter");
        assert_eq!(b.rejected_spacing, 0);
        assert_eq!(b.limit, LandmarkLimit::Candidates);
        assert!(b.closes());
    }

    // -- the rejected-candidate list (`LARGE_ITEM_RULINGS.md`, second half) --

    /// The list is a partition of the funnel, not a second opinion on it: as
    /// long as no kind hits [`REJECT_LIST_MAX_PER_KIND`], the rows per kind per
    /// reason must equal that kind's own three walk counters exactly.
    ///
    /// This is the assertion that makes the overlay trustworthy. A dot the
    /// funnel does not count, or a count with no dot, is the drift the whole
    /// `LandmarkStore`-lives-in-the-crate argument exists to prevent.
    #[test]
    fn every_reject_is_counted_by_the_funnel_and_every_count_has_its_rejects() {
        let w = world(192, 144, 1000.0);
        let res: Vec<(&str, &[f32])> = vec![("iron", w.iron.as_slice())];
        let inp = inputs(&w, &res);
        let r = generate(&inp, &LandmarkSettings::default(), 7);
        assert!(!r.rejects.is_empty(), "a world this size rejected nothing at all");
        for f in &r.funnels {
            let mut n = [0usize; 3];
            for e in r.rejects.iter().filter(|e| e.kind == f.kind) {
                n[match e.reason {
                    LandmarkRejectReason::Score => 0,
                    LandmarkRejectReason::Spacing => 1,
                    LandmarkRejectReason::Cap => 2,
                }] += 1;
            }
            let listed: usize = n.iter().sum();
            let total = f.rejected_score + f.rejected_spacing + f.rejected_cap;
            if total > REJECT_LIST_MAX_PER_KIND {
                assert_eq!(listed, REJECT_LIST_MAX_PER_KIND, "{} over-listed", f.kind);
                continue;
            }
            assert_eq!(
                (n[0], n[1], n[2]),
                (f.rejected_score, f.rejected_spacing, f.rejected_cap),
                "{}'s rejects disagree with its funnel",
                f.kind
            );
        }
        // Every row is a real cell of this grid, and carries a real key.
        for e in &r.rejects {
            assert!(e.x < inp.gw && e.y < inp.gh, "{:?} is off the grid", e);
            assert!(kind_spec(e.kind).is_some(), "{} is not a landmark kind", e.kind);
            assert!(e.score.is_finite(), "{:?} scored a non-finite value", e);
            assert!(
                e.needs_crowding.is_none() || e.reason == LandmarkRejectReason::Spacing,
                "{:?} carries a crowding figure it cannot have earned",
                e
            );
        }
        // A placed landmark is never also a reject — the walk `continue`s past
        // every rejection, so a cell cannot be in both lists for one kind.
        for lm in &r.landmarks {
            assert!(
                !r.rejects.iter().any(|e| e.kind == lm.kind && e.x == lm.x && e.y == lm.y),
                "{} at ({}, {}) is both placed and rejected",
                lm.kind,
                lm.x,
                lm.y
            );
        }
    }

    /// **The number the `Lower crowding to fit` chip is owed.**
    ///
    /// `two_cones` puts two Regional peaks 16 cells apart at 2 km per cell — 32
    /// km, inside the 34 km a Regional landmark keeps clear at `crowding 1.0`.
    /// The radius is 17 cells and the gap is 16, so the exact multiplier that
    /// clears it is `1.0 × 17 / 16 = 1.0625`, and the test brackets it: at that
    /// figure both cones are placed, and one per cent below it they are not.
    ///
    /// The bracket is the point. An arithmetic slip — the un-clamped crowding,
    /// the wrong radius, a `d` from the first conflict found rather than the
    /// nearest — still produces a plausible-looking float, and only a test that
    /// *uses* the number catches that.
    ///
    /// It is asserted **from two different starting crowdings**, and that is
    /// not belt-and-braces. The answer is an absolute setting, not a delta, so
    /// `1.0625` must come back from a run at `1.0` and from a run at `0.8`
    /// alike — and at `1.0` the `crowding *` factor is invisible, since
    /// multiplying by one changes nothing. A version that forgot the factor
    /// entirely would pass a single-crowding test and fail this one.
    #[test]
    fn a_spacing_reject_says_the_exact_crowding_that_would_have_fitted_it() {
        let (f, gw, gh, width) = two_cones();
        let mut inp = LandmarkInputs::new(&f, gw, gh, SEA, false, width);
        inp.peak_m = PEAK_M;
        for from in [1.0f64, 0.8, 0.5] {
            let base = LandmarkSettings { crowding: from, ..only_peaks(5) };
            let r = generate(&inp, &base, 1);
            let e = r
                .rejects
                .iter()
                .find(|e| e.kind == "peak" && e.reason == LandmarkRejectReason::Spacing)
                .unwrap_or_else(|| panic!("no spacing reject from crowding {}", from));
            assert_eq!((e.x, e.y), (32, 24), "the wrong cone was listed");
            let needs = e.needs_crowding.expect("a spacing reject owes a crowding figure");
            assert!(
                (needs - 17.0 / 16.0).abs() < 1e-9,
                "17 cells of radius over a 16-cell gap is 1.0625, not {} (from {})",
                needs,
                from
            );
            let at = generate(&inp, &LandmarkSettings { crowding: needs, ..only_peaks(5) }, 1);
            assert_eq!(at.placed("peak"), 2, "the stated crowding did not fit it after all");
            let below =
                generate(&inp, &LandmarkSettings { crowding: needs * 0.99, ..only_peaks(5) }, 1);
            assert_eq!(below.placed("peak"), 1, "one per cent below fitted it anyway");
        }
    }

    /// A blocker on the candidate's own cell has no finite answer, and saying
    /// so beats reporting an infinity. Cross-type competition puts a Regional
    /// peak and a Local rock formation in one field, so a Local candidate can
    /// land on a cell a Regional landmark already holds.
    #[test]
    fn needs_crowding_is_finite_or_absent_never_an_infinity() {
        let w = world(192, 144, 1000.0);
        let inp = inputs(&w, &[]);
        for crowding in [0.05, 1.0, 3.0, 0.0, f64::NAN] {
            let r = generate(&inp, &LandmarkSettings { crowding, ..Default::default() }, 3);
            for e in &r.rejects {
                if let Some(v) = e.needs_crowding {
                    assert!(v.is_finite() && v > 0.0, "{:?} at crowding {}", e, crowding);
                }
            }
        }
    }

    /// The bound exists so one slider drag cannot marshal 40 000 rows across
    /// the gdext boundary. A cap of 1 on a kind with hundreds of candidates is
    /// exactly that drag, and the funnel must still report the true total.
    #[test]
    fn the_reject_list_is_capped_per_kind_while_the_funnel_keeps_the_truth() {
        let w = world(512, 384, 1000.0);
        let inp = inputs(&w, &[]);
        // The most prolific kind on this fixture, found rather than guessed —
        // which kind that is depends on the terrain `test_field` happens to
        // make, and pinning one by name is how this test would rot.
        let survey = generate(&inp, &LandmarkSettings::default(), 5);
        let key = survey
            .funnels
            .iter()
            .max_by_key(|f| f.candidates - f.rejected_constraint)
            .map(|f| f.kind.clone())
            .expect("kinds() is not empty");
        let mut s = LandmarkSettings::default();
        for k in kinds() {
            s.set_armed(k.key, k.key == key);
        }
        s.set_cap(&key, 1);
        let r = generate(&inp, &s, 5);
        let f = r.funnel(&key).unwrap();
        assert!(
            f.rejected_cap > REJECT_LIST_MAX_PER_KIND,
            "fixture too small to reach the bound: {:?}",
            f
        );
        assert_eq!(
            r.rejects.len(),
            REJECT_LIST_MAX_PER_KIND,
            "the list must stop at the bound, not at the funnel's own total"
        );
        assert!(f.closes(), "capping the list must not touch the counters");
        // Score-sorted, so what survives the bound is the head of the queue —
        // the near-misses, not an arbitrary slice.
        let placed = r.landmarks.iter().find(|l| l.kind == key).unwrap().score;
        for e in &r.rejects {
            assert!(!(e.score > placed), "a reject out-scored the placement: {:?}", e);
        }
        for pair in r.rejects.windows(2) {
            assert!(!(pair[1].score > pair[0].score), "the retained head is not ranked");
        }
    }

    /// `nearest_conflict_sq` must agree with `fits` on the question `fits`
    /// answers, or `needs_crowding` is measured against a ring the pass never
    /// applied. Asserted directly on `Buckets`, since the disagreement would be
    /// invisible in a placement count.
    #[test]
    fn nearest_conflict_agrees_with_fits_and_finds_the_nearest() {
        for world_wrap in [false, true] {
            let mut b = Buckets::new(64, 48, world_wrap, 10.0);
            b.take(20.0, 20.0);
            b.take(26.0, 20.0);
            // Outside both: fits, and no conflict.
            assert!(b.fits(50.0, 40.0, 5.0));
            assert_eq!(b.nearest_conflict_sq(50.0, 40.0, 5.0), None);
            // (32, 20) is 6 cells from one blocker and 12 from the other.
            //
            // **The radius has to admit BOTH**, or this asserts nothing: at
            // `r = 8` the far blocker is already outside the disc and gets
            // filtered before the comparison, so a version that keeps the
            // *farthest* conflict passes. Measured — that mutant survived this
            // test until `r` was raised to 13, which puts 12 and 6 both inside
            // and makes "nearest" a real choice.
            assert!(!b.fits(32.0, 20.0, 13.0));
            let d2 =
                b.nearest_conflict_sq(32.0, 20.0, 13.0).expect("a conflict was found by fits");
            assert!((d2 - 36.0).abs() < 1e-9, "took the far blocker: {}", d2);
            // And at a radius only the near one reaches, the far one must not
            // appear at all — the filter and the minimum are two rules.
            assert!(!b.fits(32.0, 20.0, 8.0));
            let near =
                b.nearest_conflict_sq(32.0, 20.0, 8.0).expect("the near blocker is inside 8");
            assert!((near - 36.0).abs() < 1e-9, "{}", near);
            // A zero radius excludes nothing, in both.
            assert!(b.fits(20.0, 20.0, 0.0));
            assert_eq!(b.nearest_conflict_sq(20.0, 20.0, 0.0), None);
        }
        // The wrap seam: a point at x=1 and a blocker at x=63 on a 64-wide
        // wrapped grid are 2 cells apart, not 62.
        let mut w = Buckets::new(64, 48, true, 10.0);
        w.take(63.0, 10.0);
        let d2 = w.nearest_conflict_sq(1.0, 10.0, 5.0).expect("the seam must conflict");
        assert!((d2 - 4.0).abs() < 1e-9, "the seam was measured the long way round: {}", d2);
    }

    #[test]
    fn crowding_higher_packs_tighter() {
        let w = world(256, 192, 1000.0);
        let inp = inputs(&w, &[]);
        let sparse = LandmarkSettings { crowding: 0.5, ..Default::default() };
        let dense = LandmarkSettings { crowding: 2.5, ..Default::default() };
        let a = generate(&inp, &sparse, 2).landmarks.len();
        let b = generate(&inp, &dense, 2).landmarks.len();
        assert!(b > a, "crowding 2.5 placed {} and crowding 0.5 placed {}", b, a);
    }

    #[test]
    fn a_zero_or_nan_crowding_does_not_take_the_map_with_it() {
        let s_zero = LandmarkSettings { crowding: 0.0, ..Default::default() };
        assert!(s_zero.radius_km(LandmarkClass::Regional).is_finite());
        let s_nan = LandmarkSettings { crowding: f64::NAN, ..Default::default() };
        assert_eq!(s_nan.radius_km(LandmarkClass::Regional), 34.0);
        let (f, gw, gh, width) = two_cones();
        let mut inp = LandmarkInputs::new(&f, gw, gh, SEA, false, width);
        inp.peak_m = PEAK_M;
        let mut s = only_peaks(5);
        s.crowding = f64::NAN;
        let r = generate(&inp, &s, 1);
        assert!(r.funnel("peak").unwrap().closes());
    }

    #[test]
    fn cross_type_competition_changes_the_answer() {
        let w = world(256, 192, 1000.0);
        let res: Vec<(&str, &[f32])> = vec![("iron", w.iron.as_slice())];
        let inp = inputs(&w, &res);
        let on = LandmarkSettings { cross_type_competition: true, ..Default::default() };
        let off = LandmarkSettings { cross_type_competition: false, ..Default::default() };
        let a = generate(&inp, &on, 4);
        let b = generate(&inp, &off, 4);
        assert!(
            b.landmarks.len() >= a.landmarks.len(),
            "turning competition off should never place fewer: {} vs {}",
            b.landmarks.len(),
            a.landmarks.len()
        );
        for f in a.funnels.iter().chain(b.funnels.iter()) {
            assert!(f.closes(), "{}", f.kind);
        }
    }

    #[test]
    fn a_placed_landmark_is_never_inside_its_own_exclusion_radius() {
        let w = world(256, 192, 1000.0);
        let inp = inputs(&w, &[]);
        let s = LandmarkSettings::default();
        let r = generate(&inp, &s, 8);
        assert!(r.landmarks.len() > 20);
        let cell_km = w.width_km / w.gw as f64;
        for (a, la) in r.landmarks.iter().enumerate() {
            for lb in r.landmarks.iter().skip(a + 1) {
                let d_km = ((la.x as f64 - lb.x as f64).powi(2)
                    + (la.y as f64 - lb.y as f64).powi(2))
                .sqrt()
                    * cell_km;
                // See `Buckets`: the rule is the candidate's own radius, so the
                // guarantee over an unordered pair is the smaller of the two.
                let sep = s.radius_km(la.class).min(s.radius_km(lb.class));
                assert!(
                    d_km >= sep - 1e-6,
                    "{} at ({},{}) and {} at ({},{}) are {:.2} km apart, under {:.2}",
                    la.kind,
                    la.x,
                    la.y,
                    lb.kind,
                    lb.x,
                    lb.y,
                    d_km,
                    sep
                );
            }
        }
    }

    // -- the hard constraints (§2.1, §7) ------------------------------------

    /// One straight channel running east across a 32 x 8 grid at 3.125 km per
    /// cell, with a single step of `step` field units between x = 15 and
    /// x = 16. Quantised on purpose: the whole grid is arithmetic, so what
    /// passes and what fails is decided by the constants under test rather
    /// than by a generated landscape.
    fn one_channel(step: f64, flow_mult: f64) -> (Vec<f32>, Vec<u8>, Vec<i32>, Vec<f32>, f64) {
        let (gw, gh, width_km) = (32usize, 8usize, 100.0f64);
        let thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, width_km);
        let mut field = vec![0f32; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                let mut h = 0.90 - 0.002 * x as f64;
                if x >= 16 {
                    h -= step;
                }
                field[y * gw + x] = h as f32;
            }
        }
        let mut chan = vec![0u8; gw * gh];
        let mut recv = vec![-1i32; gw * gh];
        let mut flow = vec![0f32; gw * gh];
        let row = 4 * gw;
        for x in 0..gw {
            chan[row + x] = 1;
            flow[row + x] = (thresh * flow_mult) as f32;
            if x + 1 < gw {
                recv[row + x] = (row + x + 1) as i32;
            }
        }
        (field, chan, recv, flow, width_km)
    }

    fn only_waterfalls() -> LandmarkSettings {
        let mut s = LandmarkSettings::default();
        for k in kinds() {
            s.set_armed(k.key, k.key == "waterfall");
        }
        s
    }

    fn run_one_channel(step: f64, flow_mult: f64) -> LandmarkFunnel {
        let (field, chan, recv, flow, width_km) = one_channel(step, flow_mult);
        let mut inp = LandmarkInputs::new(&field, 32, 8, SEA, false, width_km);
        inp.peak_m = PEAK_M;
        inp.channel = Some(&chan);
        inp.recv = Some(&recv);
        inp.flow = Some(&flow);
        let r = generate(&inp, &only_waterfalls(), 1);
        r.funnel("waterfall").cloned().expect("waterfall has a funnel")
    }

    #[test]
    fn each_waterfall_constraint_is_load_bearing() {
        // A 414 m step on a channel carrying four times the channel threshold
        // is a waterfall, and exactly one cell in the grid is that step.
        let ok = run_one_channel(0.06, 4.0);
        assert_eq!(ok.candidates, 32, "every channel cell is a candidate");
        assert_eq!(ok.placed, 1, "the step should be the one waterfall");
        assert_eq!(ok.rejected_constraint, 31);
        assert!(ok.closes());

        // Same step, a channel carrying half the threshold: §7's flow term is
        // what rejects it. Nothing else changed.
        let dry = run_one_channel(0.06, 0.5);
        assert_eq!(dry.placed, 0, "a rill down a cliff is not a waterfall");
        assert_eq!(dry.rejected_constraint, 32);
        assert_eq!(dry.limit, LandmarkLimit::NoTerrain);
        assert!(dry.closes());

        // A 152 m step on the same channel: the drop clears
        // WATERFALL_MIN_DROP_M by nearly four times, and the gradient does not
        // clear WATERFALL_MIN_GRADIENT. Only the gradient can be rejecting it.
        let gentle = run_one_channel(0.02, 4.0);
        assert_eq!(gentle.placed, 0, "a 5 % reach is not a waterfall");
        assert_eq!(gentle.rejected_constraint, 32);
        assert!(gentle.closes());

        // No step at all: nothing to fall down.
        let flat = run_one_channel(0.0, 4.0);
        assert_eq!(flat.placed, 0);
        assert_eq!(flat.rejected_constraint, 32);
        assert_eq!(flat.limit, LandmarkLimit::NoTerrain);
    }

    // -- scoring ------------------------------------------------------------

    #[test]
    fn a_degenerate_term_is_neutral_rather_than_zero() {
        // `analysis::normalise` answers "no variation" with zeros, which would
        // send a single-candidate pool below the score floor and report a
        // landmark-free world as `no terrain`.
        let flat = vec![3.0f32; 4];
        assert_eq!(norm_term(&flat), vec![0.5f32; 4]);
        let one = vec![7.0f32];
        assert_eq!(norm_term(&one), vec![0.5f32]);
        let varied = norm_term(&[0.0f32, 1.0, 2.0]);
        assert_eq!(varied[0], 0.0);
        assert_eq!(varied[2], 1.0);
    }

    #[test]
    fn an_absent_term_renormalises_instead_of_scoring_zero() {
        let terms_both: Vec<(&'static str, f64, Vec<f32>)> =
            vec![("a", 0.5, vec![0.0, 1.0]), ("b", 0.5, vec![0.0, 1.0])];
        let terms_one: Vec<(&'static str, f64, Vec<f32>)> =
            vec![("a", 0.5, vec![0.0, 1.0])];
        assert_eq!(weighted_sum(&terms_both, 2), weighted_sum(&terms_one, 2));
        assert_eq!(weighted_sum(&terms_one, 2)[1], 1.0);
    }

    #[test]
    fn dropping_the_strahler_field_still_places_confluences() {
        let w = world(256, 192, 1000.0);
        let mut with = inputs(&w, &[]);
        let a = generate(&with, &LandmarkSettings::default(), 6);
        with.order = None;
        let b = generate(&with, &LandmarkSettings::default(), 6);
        assert!(a.placed("river_confluence") > 0, "fixture has no confluences to test");
        assert!(
            b.placed("river_confluence") > 0,
            "dropping the order field silenced confluences entirely"
        );
        for l in b.landmarks.iter().filter(|l| l.kind == "river_confluence") {
            assert!((0.0..=1.0).contains(&l.score));
            assert!(
                !l.causal.iter().any(|s| s.contains("stream order")),
                "an order fact survived without the order field: {:?}",
                l.causal
            );
        }
    }

    #[test]
    fn importance_is_emergent_and_not_a_constant() {
        let w = world(256, 192, 1000.0);
        let res: Vec<(&str, &[f32])> = vec![("iron", w.iron.as_slice())];
        let inp = inputs(&w, &res);
        let r = generate(&inp, &LandmarkSettings::default(), 11);
        assert!(r.landmarks.len() > 4);
        let lo = r.landmarks.iter().map(|l| l.importance).fold(f64::INFINITY, f64::min);
        let hi = r.landmarks.iter().map(|l| l.importance).fold(f64::NEG_INFINITY, f64::max);
        assert!(hi - lo > 0.05, "importance is flat across {} landmarks", r.landmarks.len());
    }

    // -- the seed (§27) -----------------------------------------------------

    #[test]
    fn the_seed_depends_on_every_one_of_its_four_inputs() {
        let base = landmark_seed(1, 2, LandmarkClass::Regional, 1);
        assert_ne!(base, landmark_seed(2, 2, LandmarkClass::Regional, 1));
        assert_ne!(base, landmark_seed(1, 3, LandmarkClass::Regional, 1));
        assert_ne!(base, landmark_seed(1, 2, LandmarkClass::Local, 1));
        assert_ne!(base, landmark_seed(1, 2, LandmarkClass::Regional, 2));
        assert_eq!(base, landmark_seed(1, 2, LandmarkClass::Regional, 1));
    }

    // -- the store ----------------------------------------------------------

    #[test]
    fn the_store_retains_a_run_and_can_forget_it() {
        let w = world(192, 144, 1000.0);
        let inp = inputs(&w, &[]);
        let mut store = LandmarkStore::new();
        assert!(store.last.is_none(), "a store that has not run must say so");
        let placed = store.run(&inp, 21).landmarks.len();
        assert!(placed > 0);
        assert!(store.last.is_some());
        assert_eq!(store.caps_total(), kinds().iter().filter(|k| k.buildable).map(|k| k.default_cap as u64).sum::<u64>());
        store.invalidate();
        assert!(store.last.is_none());
    }

    #[test]
    fn family_summaries_add_up_to_the_landmark_list() {
        let w = world(256, 192, 1000.0);
        let inp = inputs(&w, &[]);
        let s = LandmarkSettings::default();
        let r = generate(&inp, &s, 13);
        let mut total = 0usize;
        for fam in LandmarkFamily::all() {
            let (armed, placed) = r.family_summary(fam, &s);
            total += placed;
            let buildable = kinds().iter().filter(|k| k.family == fam && k.buildable).count();
            assert_eq!(armed, buildable, "{:?}", fam);
        }
        assert_eq!(total, r.landmarks.len());
    }

    // -- formatting ---------------------------------------------------------



    #[test]
    fn thousands_groups_the_way_the_design_writes_it() {
        assert_eq!(thousands(12400.0), "12 400");
        assert_eq!(thousands(84.0), "84");
        assert_eq!(thousands(1000.0), "1 000");
        assert_eq!(thousands(0.0), "0");
        assert_eq!(thousands(-1500.0), "-1 500");
    }

    /// **The wire format, pinned.**
    ///
    /// These six strings are a contract between three components written by
    /// three different passes: this crate emits them, `landmark_bridge.rs`
    /// forwards them into a `VarDictionary`, and
    /// `civilization_workspace.gd`'s `LM_LIMIT_WORD` / `LM_LIMIT_TIP` key off
    /// them to decide a row's wording, its tooltip, and — the one that matters
    /// — whether §2.2's accent is drawn.
    ///
    /// They shipped once spelled as display words (`"at cap"`, `"no terrain"`)
    /// and the shell compared against `"at_cap"`. Nothing failed loudly: the
    /// shell echoes an unrecognised token, so the row still read correctly
    /// while the accent and tooltip were dead. **No test caught it**, in either
    /// language, which is why this one exists — a rename here now fails here,
    /// rather than three files away and only under a probe that thinks to
    /// assert on styling.
    #[test]
    fn limit_tokens_are_the_wire_format_the_shell_keys_off() {
        assert_eq!(LandmarkLimit::AtCap.as_str(), "at_cap");
        assert_eq!(LandmarkLimit::Spacing.as_str(), "spacing");
        assert_eq!(LandmarkLimit::NoTerrain.as_str(), "no_terrain");
        assert_eq!(LandmarkLimit::Candidates.as_str(), "candidates");
        assert_eq!(LandmarkLimit::Disarmed.as_str(), "disarmed");
        assert_eq!(LandmarkLimit::NotBuildable.as_str(), "not_buildable");

        // Machine keys, so: no spaces, no capitals. A future variant added
        // without reading the above fails on this rather than on a silently
        // unstyled row.
        for t in [
            LandmarkLimit::AtCap.as_str(),
            LandmarkLimit::Spacing.as_str(),
            LandmarkLimit::NoTerrain.as_str(),
            LandmarkLimit::Candidates.as_str(),
            LandmarkLimit::Disarmed.as_str(),
            LandmarkLimit::NotBuildable.as_str(),
        ] {
            assert!(!t.contains(' '), "`{t}` is a wire key, not a display word");
            assert!(
                t.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "`{t}` must be lower_snake"
            );
        }
    }

    /// Same argument for the class keys, which the shell reverse-looks-up in
    /// `landmark_set_class_radius` and uses to pick a row's `CON`/`REG`/`LOC`/
    /// `CUL` badge and its map-marker radius.
    #[test]
    fn class_tokens_are_the_wire_format_too() {
        assert_eq!(LandmarkClass::Continental.as_str(), "continental");
        assert_eq!(LandmarkClass::Regional.as_str(), "regional");
        assert_eq!(LandmarkClass::Local.as_str(), "local");
        assert_eq!(LandmarkClass::Cultural.as_str(), "cultural");
        // The index is what `class_radius_km: [f64; 4]` is ordered by, so it
        // must not drift from the string either.
        assert_eq!(LandmarkClass::Continental.index(), 0);
        assert_eq!(LandmarkClass::Regional.index(), 1);
        assert_eq!(LandmarkClass::Local.index(), 2);
        assert_eq!(LandmarkClass::Cultural.index(), 3);
    }
}
