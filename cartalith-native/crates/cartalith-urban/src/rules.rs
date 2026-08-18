//! Generation rules and culture profiles — reference lines **28193-28280**.
//!
//! Data, not algorithm, but every later milestone reads it: `grow`
//! (milestone 7) reads all fourteen `street` fields and all five `settlement`
//! ones, `buildParcels` (milestone 12) reads all three `parcels` ones,
//! `privatizeAlleys` (milestone 11) reads `street.deadEndBias`, and
//! `generate()` (milestone 16) threads the resolved [`CultureProfile`] down the
//! whole call chain. Getting a literal wrong here moves every street in every
//! town, silently.
//!
//! All eight items are on `UME`'s **public** export rather than its `_test`
//! one, so this is the first milestone in the subsystem that needs no
//! indirection at all to golden-verify: the capture calls
//! `CULTURE_PROFILES` / `resolveProfile` / `DEFAULT_RULES` / `resolveRules` /
//! `cloneRules` / `applyWildness` / `applyPlotChaos` directly, and observes
//! `clamp` through the two sliders that use it.
//!
//! # The one real porting hazard, and it is a big one
//!
//! `clamp` is `Math.max(lo, Math.min(hi, v))`. The obvious Rust transliteration
//! is `lo.max(hi.min(v))` — **and it is wrong**, by exactly the inversion
//! `cartalith-rust-conventions` exists to catch. JS `Math.min`/`Math.max`
//! *propagate* NaN; Rust's `f64::min`/`f64::max` *absorb* it and return the
//! other operand. So `applyWildness(rules, f64::NAN)` leaves eight NaN street
//! fields in the reference, while `lo.max(hi.min(NAN))` hands back `hi` from
//! the inner call and keeps it through the outer one — landing every clamped
//! field on its own **upper** bound. A naive port turns a NaN slider into a
//! maximally-wild rule set that looks entirely plausible, is not the
//! reference's, and is then fed to `grow`, where a NaN jitter rejects every
//! candidate segment while a `0.70` one quietly builds a different town. Same
//! shape of trap `cartalith-assets` milestone 3 hit from the other direction
//! (`f64::min` absorbing a NaN density where `Math.min` propagated it).
//!
//! [`clamp`] therefore goes through explicit [`js_min`] / [`js_max`], written
//! to mirror the source expression, and `wild_NaN` / `chaos_NaN` goldens pin
//! it. There is one documented divergence left in them, on signed zero — see
//! [`js_min`].
//!
//! # What is *not* here, and why
//!
//! `cloneRules` is `JSON.parse(JSON.stringify(r))`. On a well-formed rule set
//! that is a deep clone, which `#[derive(Clone)]` already is, so it does not
//! survive as a separate function — the same call milestone 2 made about
//! `gKey`. It is not *quite* a deep clone in JS, though: a NaN round-trips to
//! `null`, and the capture pins that the reference really does do this
//! (`CLONE_NAN_BECOMES`). A typed [`Rules`] has no `null` to land on, so the
//! port keeps the NaN. Unreachable inside the engine — `resolveRules` clones
//! the all-finite `DEFAULT_RULES` and `Object.assign`s the caller's partial on
//! *top* of the clone, so nothing a caller supplies is ever round-tripped — but
//! stated rather than hidden.

/// `Math.min(a, b)`, with JS semantics rather than Rust's.
///
/// The difference that matters: **JS propagates NaN, Rust absorbs it.**
/// `Math.min(0.70, NaN)` is `NaN`; `f64::min(0.70, NaN)` is `0.70`.
///
/// **One documented divergence, on signed zero.** `Math.min(+0, -0)` is `-0`
/// and `Math.max(+0, -0)` is `+0`; this returns whichever argument the `<`
/// comparison happens to land on, since `-0.0 < 0.0` is false. Only two of
/// `applyWildness`/`applyPlotChaos`'s eleven clamps have a zero bound
/// (`pierceChance` and `deadEndBias`, both `lo = 0`), and neither can reach a
/// `-0` argument: `0.10 * (2 - w)` is `-0` only if `2 - w` is `-0`, which
/// subtraction of two finite doubles never produces, and
/// `deadEndBias + (w - 1) * 0.15` is `+0` at `w == 1`. So the divergence is
/// unreachable, and handling it would be four lines of code for a case no
/// caller can construct. Recorded, not coded around.
fn js_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if b < a {
        b
    } else {
        a
    }
}

/// `Math.max(a, b)`, with JS semantics. See [`js_min`] for the NaN rule and the
/// signed-zero divergence.
fn js_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if b > a {
        b
    } else {
        a
    }
}

/// `const clamp=(v,lo,hi)=>Math.max(lo,Math.min(hi,v));` — reference line 28256.
///
/// Argument order is the reference's `(v, lo, hi)`, deliberately not Rust's
/// `f64::clamp(min, max)` receiver form, so the eleven call sites below read
/// line-for-line against the source they are checked against.
///
/// `f64::clamp` would in fact agree on every reachable input here (it is
/// written as comparisons, so it propagates NaN too) — but it *panics* when
/// `min > max`, where the reference returns `lo`, and it would hide the
/// `js_min`/`js_max` question this whole module turns on. Written out.
pub fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    js_max(lo, js_min(hi, v))
}

/// `Math.round(x)` for the one place `applyPlotChaos` needs it.
///
/// JS rounds halves toward `+Infinity`; Rust's `f64::round` rounds halves away
/// from zero. They differ only for negative halves, and this function's only
/// argument is `clamp(2 * c, 1, 4)`, whose range is `[1, 4]` plus NaN — so
/// `f64::round` is exact here, and the goldens include the three `c` values
/// (`0.75`, `1.25`, `1.75`) that land it on `1.5`, `2.5` and `3.5` exactly.
fn js_round(x: f64) -> f64 {
    x.round()
}

// ---------------------------------------------------------------- profiles --

/// One culture profile — reference lines 28199-28211.
///
/// The reference's own header comment states the design rule this type exists
/// to hold: *"the core engine never branches on a culture; every
/// tradition-specific choice is data on the resolved profile object, read once
/// in `generate()` and threaded down the call chain. Adding a 3rd/4th
/// civilization is a new table row, not an engine change."*
///
/// Every field is `&'static str` where the reference's is a string, for the
/// same reason milestone 2 kept `Edge::cls` as one: the engine compares these
/// by string value (`profile.planning === 'radial'`,
/// `profile.buildingGrammar === 'venus-mixed'`,
/// `profile.wallGates.scheme === 'organic'`), and `profile.id` is used as a
/// **lookup key** into `GAMES_SPEC` and `FARM_SPEC` (milestones 13-15), so the
/// string *is* the value.
///
/// **Four fields are read by nothing**, verified by grep across all 2,937 lines
/// of block 4 and the whole host app: `parcel_pattern`, `orientation`,
/// `civic_anchor_label` and `default_walls`. They are carried anyway — see each
/// field's own note for what killed it. `prov` is likewise unread, and is
/// carried because it is the derivation, and `CLAUDE.md`'s standing rule is
/// that constants without reachable derivations get "cleaned up" by someone who
/// cannot see why they hold.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CultureProfile {
    /// Also the key into `CULTURE_PROFILES`, and the lookup key `buildGames`
    /// (milestone 14) and `buildFarmland` (milestone 15) use into their own
    /// spec tables.
    pub id: &'static str,
    /// Surfaced by `generate()`'s returned metrics as `cultureName`.
    pub name: &'static str,
    /// `'organic'` or `'radial'`. `generate()` line 31011 dispatches the whole
    /// street-planning mode on this.
    pub planning: &'static str,
    /// **Dead.** The reference documents it dead itself, at lines 30225-30227:
    /// the insula (bounding-box) platting method it dispatched was removed with
    /// the other 17 profiles, and both survivors set `'strip'`. Kept as the
    /// hook a future third profile would need.
    pub parcel_pattern: &'static str,
    /// `'burgage'` or `'venus-mixed'`; `buildBuildings` (milestone 13) branches
    /// on it twice.
    pub building_grammar: &'static str,
    /// `generate()`: `opts.faith || profile.defaultFaith`.
    pub default_faith: &'static str,
    /// `generate()`: `opts.civicStyle || profile.defaultCivic`.
    pub default_civic: &'static str,
    /// Gates `buildMarkets` entirely. The reference calls it "a data-driven
    /// hook for a profile whose commerce ran through a different institution";
    /// both live profiles set it true.
    pub markets: bool,
    /// The reference nests this one field inside `wallGates:{scheme:'organic'}`.
    /// Flattened here — it is the object's only key, it is only ever read as
    /// `profile.wallGates.scheme === 'organic'` (the anachronism guard on the
    /// star fort, milestone 16 line 30964), and the whole object is never
    /// passed anywhere as a unit.
    pub wall_gates_scheme: &'static str,
    /// **Dead.** `'terrain'` / `'radial'`. Zero reads anywhere; the street
    /// orientation is decided by `buildPrimaries` and `buildRadialStreets`
    /// choosing themselves, not by reading this.
    pub orientation: &'static str,
    /// **Dead.** Zero reads. A UI label for the civic anchor that no live code
    /// path renders.
    pub civic_anchor_label: &'static str,
    /// **Dead, and the reference's own provenance prose is stale about it.**
    /// `venus`'s `prov` says "the UI unchecks the wall box on selecting this
    /// profile", but `defaultWalls` has **zero** reads in v2.10, inside block 4
    /// or out. `None` where the reference leaves the key off the object
    /// entirely (`medieval`), rather than collapsing absent into `false` — the
    /// two are the same *truthiness* but not the same *intent*, and a host that
    /// ever honours this needs to tell "profile has no opinion" from "profile
    /// says no".
    pub default_walls: Option<bool>,
    /// Absent on `medieval`, so `profile.waterway` is `undefined` there and
    /// both of the engine's reads (`generate()` lines 31017 and 31063) are
    /// truthiness tests — `bool` is the exact partition, unlike
    /// [`Self::default_walls`], which nothing reads at all.
    pub waterway: bool,
    /// **Dead within `CULTURE_PROFILES`, and the reason it is not.** Neither
    /// live profile defines `deadEndBias`, so `privatizeAlleys`' expression
    /// `clamp((profile.deadEndBias||0) + (rules.street.deadEndBias||0), 0, 0.40)`
    /// (line 30097, milestone 11) always contributes `0` from the profile side.
    /// The capture asserts that absence rather than trusting it, and fails if a
    /// re-freeze ever adds the key. Carried as `0.0` — the value `||0` yields —
    /// so milestone 11 can write the expression as the reference writes it.
    pub dead_end_bias: f64,
    /// The reference's own derivation for the profile. Read by nothing; kept
    /// because a constant without its derivation is a constant somebody
    /// deletes.
    pub prov: &'static str,
}

/// `medieval` — reference lines 28200-28204.
pub const MEDIEVAL: CultureProfile = CultureProfile {
    id: "medieval",
    name: "Organic Growth (Medieval Western European)",
    planning: "organic",
    parcel_pattern: "strip",
    building_grammar: "burgage",
    default_faith: "church",
    default_civic: "auto",
    markets: true,
    wall_gates_scheme: "organic",
    orientation: "terrain",
    civic_anchor_label: "market",
    default_walls: None,
    waterway: false,
    dead_end_bias: 0.0,
    prov: "Accretive growth over centuries: epoch-looped organic streets, series-platted burgage strip parcels, a parish church per few thousand souls, market squares that specialise with rank (docs/01 §1.1; docs/03 M-* register). Presented as the general \"Organic Growth\" pattern rather than one culture among many: a post-launch review across this register's original 19 profiles found every other organic-planning culture (Islamic, Byzantine, Chinese, Aztec, Viking, Celtic, Greek, Egyptian, Mesopotamian, Mayan, Inca, Japanese, Colonial, Frontier, Industrial, Palimpsest) rendering near-indistinguishably from this one at the level this tool actually draws — a rendering/visual-distinctiveness problem, not a defect in their underlying research, which remains valid and citable but is no longer separately modelled here (docs/07 §3.10). Kept as this pattern's concrete, best-attested reference point, alongside the radial Venus Project as the one structurally distinct alternative.",
};

/// `venus` — reference lines 28205-28210.
pub const VENUS: CultureProfile = CultureProfile {
    id: "venus",
    name: "The Venus Project (resource-based circular city)",
    planning: "radial",
    parcel_pattern: "strip",
    building_grammar: "venus-mixed",
    default_faith: "none",
    default_civic: "dome",
    markets: true,
    wall_gates_scheme: "organic",
    orientation: "radial",
    civic_anchor_label: "Center for Resource Management",
    default_walls: Some(false),
    waterway: true,
    dead_end_bias: 0.0,
    prov: "Jacque Fresco's Venus Project circular city (thevenusproject.com/resource-based-economy/environment/circular-city), taken here as a deliberate design fusion rather than a literal reconstruction: several concentric ring streets at regular intervals connected by radial spokes (with extra cross-spokes in the wider outer band) to a domed central hub, the Center for Resource Management (M-VEN-2) — a genuinely new radial planning mode (M-VEN-1), not a re-skin of the grid or organic models, using the shape-aware bisector parcel method rather than the bounding-box-based insula method since the wedge-shaped ring blocks need lot boundaries that respect the true (curved) block outline. Per the brief, the clean circular geometry is mixed with the lived-in amenity richness of medieval-European and Asian/Japanese towns (M-VEN-5): circular pavilions cluster at the hub and inner rings, the residential rings blend the standardized modular apartment with Asian courtyard houses and Japanese machiya rowhouses, market/amenity squares scale with rank, and the outermost ring carries the logistics warehouses. Fortification is optional and off by default (defaultWalls:false — the UI unchecks the wall box on selecting this profile), but the medieval curtain wall and the bastioned star fort are both available on request (wallGates.scheme:'organic', so the anachronism guard permits the trace). A circular irrigation waterway (M-VEN-3) encircles the built city: when the star fort is built the canal supplies its wet moat around the bastions even on a landlocked site, otherwise it is the irrigation ring outside the (optional) curtain wall — drawn as a fully-closed circle capped to the map so it never terminates in a straight clipped edge. The concentric rings and spokes carry a seeded low-frequency wobble (post-review organic softening, M-VEN-1) so the radial skeleton reads as hand-drawn rather than compass-drawn, consistent with this tool's own hand-drawn-cadastral aesthetic.",
};

/// `CULTURE_PROFILES` — in the reference's own key order, which is also the
/// order a UI would list them in.
///
/// Only the two live profiles. The other 17 the reference's register once
/// carried are documented history (docs/07 §3.10) after a post-launch review
/// found them visually indistinguishable at the level this tool draws; they are
/// explicitly out of scope for every milestone.
pub const CULTURE_PROFILES: [CultureProfile; 2] = [MEDIEVAL, VENUS];

/// `function resolveProfile(id){return CULTURE_PROFILES[id]||CULTURE_PROFILES.medieval;}`
/// — reference line 28212. Unknown id falls back to `medieval`.
///
/// **One deliberate divergence, and it is a hardening.** The reference indexes
/// a plain object literal, so it walks `Object.prototype`: `resolveProfile
/// ('toString')` returns `Function.prototype.toString`, `resolveProfile
/// ('constructor')` returns `Object`, and `resolveProfile('__proto__')` returns
/// `Object.prototype` — all truthy, so all sail past the `||` fallback and
/// return something that is not a profile at all. `generate()` would then read
/// `profile.planning` as `undefined`, take the organic branch, and crash at
/// `profile.wallGates.scheme`. The capture pins all five of those (`toString`,
/// `constructor`, `__proto__`, `valueOf`, `hasOwnProperty`) as the reference's
/// real behaviour, and a golden asserts this port returns `medieval` for every
/// one of them instead. A `match` has no prototype chain; reproducing the
/// hazard would mean building one on purpose.
pub fn resolve_profile(id: &str) -> CultureProfile {
    match id {
        "medieval" => MEDIEVAL,
        "venus" => VENUS,
        _ => MEDIEVAL,
    }
}

// ------------------------------------------------------------------- rules --

/// `rules.street` — reference lines 28223-28237. Read by `grow` (milestone 7),
/// which is the single most behaviour-defining function in the subsystem, and
/// by `privatizeAlleys` (milestone 11).
///
/// The reference's own comment on this block is worth carrying: *"Every value
/// below was previously an inline literal inside `grow()`/`buildParcels()`/
/// `buildWall()`'s bridgehead check. `DEFAULT_RULES` reproduces those exact
/// literals, so `generate()` with no `rules` option is BYTE-IDENTICAL to every
/// prior version."* That is a parity contract, not decoration — the defaults
/// are not tunable-by-taste, they *are* the previous versions' behaviour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StreetRules {
    /// Sigma, radians (~15°) — deviation from perpendicular when branching
    /// (M-NET-3).
    pub branch_angle_jitter: f64,
    /// Sigma, radians (~12.6°) — wander of dead-end exploration continuations
    /// (M-GRW-1).
    pub continuation_jitter: f64,
    /// Exploration share at epoch 1 (M-GRW-1).
    pub exploration_start: f64,
    /// Per-epoch decay. **Neither compound slider touches this one.**
    pub exploration_decay: f64,
    /// Floor.
    pub exploration_minimum: f64,
    /// Metres (M-NET-4). **Untouched by the sliders.**
    pub segment_length_median: f64,
    /// Sigma.
    pub segment_length_variance: f64,
    /// Chance a candidate crosses a street instead of T-junctioning.
    pub pierce_chance: f64,
    /// Radians (~25°) — acute junctions below this are rejected (M-NET-3).
    pub junction_angle_limit: f64,
    /// Divisor in `1/(1+dM/decay)`, M-DEN-3. **Untouched by the sliders.**
    pub market_gradient_decay: f64,
    /// Metres, minimum near-parallel street spacing (M-BLK-4).
    pub parallel_street_spacing: f64,
    /// Fraction of minor streets privatized into cul-de-sacs (M-ISL-2). The
    /// reference's comment calls `profile.deadEndBias` "the per-culture floor
    /// this adds to" — see [`CultureProfile::dead_end_bias`] for why that floor
    /// is always zero in v2.10.
    ///
    /// **The one field [`apply_wildness`] accumulates rather than recomputes**,
    /// which is what makes that function non-idempotent. See its docs.
    pub dead_end_bias: f64,
    /// Metres from the bridge point the far-bank bridgehead is confined to.
    /// **Untouched by the sliders.**
    pub bridgehead_distance: f64,
    /// Chance a far-bank candidate is kept at all.
    pub bridgehead_probability: f64,
}

/// `rules.parcels` — reference lines 28238-28241. Read by `buildParcels`
/// (milestone 12).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParcelRules {
    /// Sigma; the median frontage of 11 m stays fixed (M-PAR-1).
    pub frontage_width_variance: f64,
    /// Sigma; the median depth of 22/30 m by core-vs-fringe stays fixed
    /// (M-PAR-2).
    pub plot_depth_variance: f64,
    /// Max burgage-cycle re-subdivisions of a frontage grant.
    ///
    /// **`f64`, not an integer**, deliberately. `applyPlotChaos` writes
    /// `Math.round(clamp(2*c,1,4))` into it, which is `NaN` for a `NaN` slider,
    /// and milestone 12 reads it only through
    /// `Math.min(P.subdivisionCap, Math.floor(age/3))` — where a `NaN` makes
    /// the whole expression `NaN` and the re-subdivision loop run zero times.
    /// Typing it `u32` would have to decide what `NaN` becomes, and every
    /// choice is a divergence.
    pub subdivision_cap: f64,
}

/// `rules.settlement` — reference lines 28242-28248. Successive wall
/// generations (M-GRW-2), reached only through the opt-in `wallGenerations`
/// toggle; read by `grow` (milestone 7). **Neither compound slider touches this
/// group at all.**
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettlementRules {
    /// Interior-only built hull fills this fraction of the active wall's area
    /// to be eligible.
    pub wall_generation_threshold: f64,
    /// Years since the active wall was built before another may supersede it.
    /// The reference cites the real successive-circuit gaps it is drawn from:
    /// Cologne ~74 y, Florence ~94-111 y, Paris ~158-168 y.
    pub wall_generation_min_age_gap: f64,
    /// Exterior (ribbon-suburb) built nodes must reach this share of interior
    /// ones too.
    pub wall_generation_extramural_share: f64,
    /// M-GRW-2: "1-3 typical for towns that persist".
    pub max_wall_generations: f64,
    /// `0` = ignore the placeholder carrying-capacity factor, `1` = full
    /// effect. `grow` reads it as
    /// `(1-w) + w*estimateCarryingCapacity(...)`, so it is a blend weight, not
    /// a flag.
    pub carrying_capacity_weight: f64,
}

/// `rules.meta` — reference lines 28249.
///
/// The reference is explicit that these two are **not consumed by the engine**:
/// they are UI-side compound sliders, and [`apply_wildness`] /
/// [`apply_plot_chaos`] are what turn them into values on the individual
/// `street`/`parcels` fields, which stay the single source of truth. The
/// sliders write their own argument back here as a record of what was applied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetaRules {
    pub wildness: f64,
    pub plot_chaos: f64,
}

/// `DEFAULT_RULES` / a resolved rule set — reference lines 28222-28249.
///
/// `Copy` as well as `Clone`: the whole thing is 24 `f64`s, and `cloneRules`'
/// only role in the reference is to stop callers aliasing the module-level
/// `DEFAULT_RULES` object, which a value type makes structurally impossible.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rules {
    pub street: StreetRules,
    pub parcels: ParcelRules,
    pub settlement: SettlementRules,
    pub meta: MetaRules,
}

/// `DEFAULT_RULES` — the exact literals the reference's own comment promises
/// are byte-identical to the inline constants they replaced.
pub const DEFAULT_RULES: Rules = Rules {
    street: StreetRules {
        branch_angle_jitter: 0.26,
        continuation_jitter: 0.22,
        exploration_start: 0.55,
        exploration_decay: 0.05,
        exploration_minimum: 0.12,
        segment_length_median: 56.0,
        segment_length_variance: 0.5,
        pierce_chance: 0.10,
        junction_angle_limit: 0.44,
        market_gradient_decay: 200.0,
        parallel_street_spacing: 24.0,
        dead_end_bias: 0.0,
        bridgehead_distance: 190.0,
        bridgehead_probability: 0.35,
    },
    parcels: ParcelRules {
        frontage_width_variance: 0.22,
        plot_depth_variance: 0.28,
        subdivision_cap: 2.0,
    },
    settlement: SettlementRules {
        wall_generation_threshold: 0.8,
        wall_generation_min_age_gap: 120.0,
        wall_generation_extramural_share: 0.15,
        max_wall_generations: 3.0,
        carrying_capacity_weight: 1.0,
    },
    meta: MetaRules {
        wildness: 1.0,
        plot_chaos: 1.0,
    },
};

macro_rules! patch_group {
    (
        $(#[$meta:meta])*
        $name:ident => $target:ident { $($field:ident),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Default)]
        pub struct $name {
            $(pub $field: Option<f64>,)+
        }
        impl $name {
            fn assign_onto(&self, dst: &mut $target) {
                $(if let Some(v) = self.$field { dst.$field = v; })+
            }
        }
    };
}

patch_group! {
    /// A partial `rules.street`, as `Object.assign(out.street, partial.street)`
    /// applies it: **per field**, not per group.
    StreetPatch => StreetRules {
        branch_angle_jitter, continuation_jitter, exploration_start, exploration_decay,
        exploration_minimum, segment_length_median, segment_length_variance, pierce_chance,
        junction_angle_limit, market_gradient_decay, parallel_street_spacing, dead_end_bias,
        bridgehead_distance, bridgehead_probability,
    }
}

patch_group! {
    /// A partial `rules.parcels`.
    ParcelPatch => ParcelRules {
        frontage_width_variance, plot_depth_variance, subdivision_cap,
    }
}

patch_group! {
    /// A partial `rules.settlement`.
    SettlementPatch => SettlementRules {
        wall_generation_threshold, wall_generation_min_age_gap,
        wall_generation_extramural_share, max_wall_generations, carrying_capacity_weight,
    }
}

patch_group! {
    /// A partial `rules.meta`.
    MetaPatch => MetaRules {
        wildness, plot_chaos,
    }
}

/// The `partial` argument to [`resolve_rules`] — `generate()`'s `opts.rules`.
///
/// `None` on a group is the reference's falsy-group case: `resolveRules`' loop
/// is `if(partial[grp]) Object.assign(...)`, so a group that is absent, `null`,
/// `0`, `false` or `''` is skipped wholesale and that whole group keeps its
/// defaults.
///
/// **Two structural divergences from `Object.assign`, both unobservable.** The
/// reference iterates `Object.keys(out)`, so an *unknown group* on the partial
/// is ignored — a typed struct has no unknown groups. And `Object.assign`
/// copies every own enumerable key, so an *unknown field* inside a known group
/// really does land on the resolved object — where nothing reads it, since
/// every consumer names its fields. Both are pinned by goldens
/// (`resolveUnknownGroup`, `resolveFalsyGroups`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RulesPatch {
    pub street: Option<StreetPatch>,
    pub parcels: Option<ParcelPatch>,
    pub settlement: Option<SettlementPatch>,
    pub meta: Option<MetaPatch>,
}

/// `resolveRules(partial)` — reference lines 28251-28255.
///
/// Clone the defaults, then shallow-merge each supplied group over its clone.
/// `resolve_rules(None)` is exactly `DEFAULT_RULES`, and — pinned by the
/// `defaultsAfterMerges` golden, captured from the reference *after* its
/// heaviest merge — no call can ever mutate the defaults.
pub fn resolve_rules(partial: Option<&RulesPatch>) -> Rules {
    let mut out = DEFAULT_RULES;
    if let Some(p) = partial {
        if let Some(g) = &p.street {
            g.assign_onto(&mut out.street);
        }
        if let Some(g) = &p.parcels {
            g.assign_onto(&mut out.parcels);
        }
        if let Some(g) = &p.settlement {
            g.assign_onto(&mut out.settlement);
        }
        if let Some(g) = &p.meta {
            g.assign_onto(&mut out.meta);
        }
    }
    out
}

/// `applyWildness(rules, w)` — reference lines 28260-28273.
///
/// The primary organic-vs-planned control, nominally `0..2` with `1.0` the
/// `DEFAULT_RULES` baseline. The reference flags its own formulas as *"a PoC
/// convention (monotonic, clamped to the ranges in the design spec), not
/// independently sourced"*, which is the same honesty flag this project uses
/// for tuned-not-measured constants; they are reproduced exactly, including the
/// no-op `/1` in the `junctionAngleLimit` and `parallelStreetSpacing` lines.
///
/// # Three behaviours that are the reference's, not accidents of the port
///
/// 1. **It is not idempotent.** Ten of the eleven fields are recomputed from a
///    *hardcoded literal* times `w`, so re-applying the same `w` is a no-op for
///    them — but `dead_end_bias` is
///    `clamp(dead_end_bias + (w-1)*0.15, 0, 0.40)`, reading its own current
///    value. `apply_wildness(r, 2.0)` five times walks it 0.15 → 0.30 → 0.40
///    (capped) and leaves everything else unchanged. Golden-pinned by
///    `wildTwice1p5`, `wildThrice2` and `wildFive2`.
/// 2. **It overwrites custom values it never reads.** A caller who set
///    `branch_angle_jitter` through [`resolve_rules`] and then calls this loses
///    it: the formula's base is the literal `0.26`, not the current field.
///    Golden-pinned by `wildOverCustom`.
/// 3. **It touches four `street` fields not at all** — `exploration_decay`,
///    `segment_length_median`, `market_gradient_decay`, `bridgehead_distance` —
///    and neither `parcels` nor `settlement`.
///
/// A non-finite `w` propagates: `NaN` leaves eight NaN street fields (**not**
/// the clamp bounds — see [`clamp`]), and `+Infinity` saturates every field to
/// its own upper clamp while `-Infinity` saturates to the lower one. All three
/// are goldens.
pub fn apply_wildness(rules: &mut Rules, w: f64) {
    let s = &mut rules.street;
    s.branch_angle_jitter = clamp(0.26 * w, 0.15, 0.70);
    s.continuation_jitter = clamp(0.22 * w, 0.10, 0.50);
    s.segment_length_variance = clamp(0.5 * w, 0.25, 1.1);
    s.exploration_start = clamp(0.55 * w, 0.20, 0.90);
    s.exploration_minimum = clamp(0.12 * w, 0.05, 0.30);
    s.pierce_chance = clamp(0.10 * (2.0 - w), 0.0, 0.15);
    s.junction_angle_limit = clamp(0.44 * (2.0 - w) / 1.0, 0.14, 0.52);
    s.parallel_street_spacing = clamp(24.0 * (2.0 - w) / 1.0, 10.0, 30.0);
    s.dead_end_bias = clamp(s.dead_end_bias + (w - 1.0) * 0.15, 0.0, 0.40);
    s.bridgehead_probability = clamp(0.35 * w, 0.1, 0.7);
    rules.meta.wildness = w;
}

/// `applyPlotChaos(rules, c)` — reference lines 28274-28280.
///
/// The parcel-metrology counterpart to [`apply_wildness`]: three `parcels`
/// fields and `meta.plot_chaos`, nothing else. Unlike `apply_wildness` it *is*
/// idempotent — all three formulas recompute from a literal, none accumulate.
///
/// `subdivision_cap` is the one rounded field; see [`js_round`] for why
/// `f64::round` is exact on its `[1, 4]` domain, and [`ParcelRules::
/// subdivision_cap`] for why the result stays a float.
pub fn apply_plot_chaos(rules: &mut Rules, c: f64) {
    let p = &mut rules.parcels;
    p.frontage_width_variance = clamp(0.22 * c, 0.10, 0.50);
    p.plot_depth_variance = clamp(0.28 * c, 0.10, 0.60);
    p.subdivision_cap = js_round(clamp(2.0 * c, 1.0, 4.0));
    rules.meta.plot_chaos = c;
}

impl Rules {
    /// The canonical field order the milestone-4 goldens are captured in — the
    /// reference's own key order within each group, and the groups in the
    /// reference's own order. The capture asserts the reference's objects still
    /// carry exactly this key set in exactly this order, so a rule added
    /// upstream cannot silently drop out of the comparison.
    #[cfg(test)]
    pub(crate) fn flatten(&self) -> [f64; 24] {
        let (s, p, t, m) = (&self.street, &self.parcels, &self.settlement, &self.meta);
        [
            s.branch_angle_jitter,
            s.continuation_jitter,
            s.exploration_start,
            s.exploration_decay,
            s.exploration_minimum,
            s.segment_length_median,
            s.segment_length_variance,
            s.pierce_chance,
            s.junction_angle_limit,
            s.market_gradient_decay,
            s.parallel_street_spacing,
            s.dead_end_bias,
            s.bridgehead_distance,
            s.bridgehead_probability,
            p.frontage_width_variance,
            p.plot_depth_variance,
            p.subdivision_cap,
            t.wall_generation_threshold,
            t.wall_generation_min_age_gap,
            t.wall_generation_extramural_share,
            t.max_wall_generations,
            t.carrying_capacity_weight,
            m.wildness,
            m.plot_chaos,
        ]
    }
}

#[cfg(test)]
mod tests;
