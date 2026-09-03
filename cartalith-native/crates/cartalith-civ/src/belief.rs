//! Culture and religion as quantitative traits, the compatibility relation
//! between them, and the network diffusion that runs on top of it.
//!
//! `RELIGION_DIFFUSION_SCOPE.md` §1 carries the owner-supplied paper this
//! implements; §2 records what it maps onto.
//!
//! **Two halves, and the doc below is written from the first one's point of
//! view.** Everything down to [`compat_value`] is the trait/compatibility
//! foundation. Everything after it is §3 **milestone 1** — network exposure,
//! §19's conversion logistic, and per-settlement adherence — which was added
//! later, and which has its own header comment and its own two measured
//! disclosures. Where the text below says milestone 1 does not exist yet, the
//! sentence has been corrected in place and says so; nothing here is left
//! describing the state before it landed.
//!
//! ## Why a parallel table rather than fields on `Culture`
//!
//! [`crate::Culture`] is a *naming pool* — `key`, `syl`, `sfx` — and
//! `tests/golden_parity_settlement_naming.rs` pins its contents by index
//! (its own module doc: *"faction 1 -> `CIV_CULTURES[1 % 7]` = imperial"*,
//! which [`crate::civ_settle_name`] reaches through
//! [`crate::civ_default_culture`]'s `CIV_CULTURES[faction % 7]`). Belief
//! traits are a different subsystem's concern, and the crate already
//! established the pattern for exactly this:
//! [`crate::CIV_CULTURE_TERRAIN_KEY`] is a parallel table keyed by culture
//! key rather than a field on the struct. This follows it.
//!
//! The risk a parallel table carries is drift, so that risk is paid down
//! directly: `culture_profiles_cover_every_culture` and
//! `religion_profiles_cover_every_religion` fail if a key is ever added to
//! either vocabulary without a profile here.
//!
//! *Citation correction, made in the implementing pass:* the scaffold's doc
//! cited `golden_parity_settlement_naming` as a test **function**. It is a
//! test **file**
//! (`crates/cartalith-civ/tests/golden_parity_settlement_naming.rs`), and the
//! claim it was cited for is real and is quoted above — so the argument
//! stands unchanged and only the kind of thing being cited is corrected.
//! `civ_continents_names_a_landmass_in_its_plurality_factions_culture` in
//! lib.rs is a second, in-crate witness to the same index dependency.
//!
//! ## The discipline that governs every number below
//!
//! [`crate::CIV_CULTURE_TERRAIN_KEY`]'s own doc comment states the rule this
//! module inherits: `common` and `imperial` are *"identity-flavored, not
//! terrain-themed, and deliberately get no verdict (`None`) rather than a
//! fabricated one — same 'never fabricate a verdict without a real basis'
//! discipline the reference's own v1.35 `basis` field already established
//! for trade."*
//!
//! So compatibility here is **derived from axes, not hand-typed as a 56-cell
//! matrix**. A matrix of 8 religions × 7 cultures would be 56 magic numbers
//! with no basis, no way to check one against another, and no answer when
//! somebody adds a ninth religion. Deriving it from each side's own declared
//! affinities means the values are reproducible, testable, and extend by
//! construction — and it means a culture with no thematic basis gets a
//! stated neutral rather than an invented affinity.
//!
//! The paper's §30 asks for exactly this posture: dimensionless normalized
//! parameters treated as model assumptions requiring calibration, not
//! empirical constants.
//!
//! ## The one structural fact this design rests on
//!
//! [`crate::CIV_TERRAIN_MIX_KEYS`] (`["river", "coast", "arid", "forest",
//! "hills"]`) is the crate's single terrain vocabulary, and
//! [`crate::CIV_CULTURE_TERRAIN_KEY`] maps the five themed cultures
//! **bijectively** onto it — `hills`, `arid`, `river`, `forest`, `coast`,
//! each claimed exactly once. Authoring the religion side into that *same*
//! vocabulary, likewise injectively, makes plain equality a complete and
//! informative relation: every themed culture already has exactly one
//! matching religion, so a graded affinity table — and every authored number
//! in it — buys nothing. That is the whole reason this module is small.
//!
//! ## What is deliberately **not** shipped, and why
//!
//! **§6's eight-element religion trait vector is deferred, not dropped.**
//! `RELIGION_DIFFUSION_SCOPE.md` §3 milestone 3 *is* the authoring pass —
//! "the one milestone that is primarily content, not code — an owner-facing
//! pass". Shipping `C_comp`/`C_ritual`/`C_commit`/`C_inst`/`C_coh`/`C_pros`/
//! `C_mem` now would be 7 religions × 7 fields = 49 numbers with no basis
//! and no caller: this module doc's own 56-cell-matrix objection, wearing a
//! struct. It would also corner milestone 3, whose owner would then be
//! *editing this port's invented values* instead of authoring their own.
//! Checked against milestone 1's own text, six of the eight are excluded by
//! that milestone anyway: `C_inst`/`C_coh` are milestone 2; `C_pros` is
//! explicitly deferred ("defer missionary/institutional/direct terms");
//! `C_commit` is §19's `Cost`, not one of the three terms milestone 1 keeps;
//! `C_comp`/`C_mem` feed transmission fidelity, which milestone 1 does not
//! compute. The remaining two have no caller either. When the vector lands
//! it lands as **authored content**, which is what §30 asks for.
//!
//! Also absent, each for a stated reason:
//!
//! - **`p_conv` and the `β0`/`βE`/`βC`/`βF` coefficients** — *(BUILT: this
//!   said "milestone 1's work, not the trait model's". Milestone 1 has since
//!   landed in the second half of this file, and the shape predicted here —
//!   `1.0 / (1.0 + js_exp(-(b0 + bE*e + bC*compat + bF*freq)))` — is what
//!   [`belief_logistic`] and [`belief_step`] actually do, `js_exp` included.
//!   The prediction is kept because it was right and because a reader
//!   arriving at this list should be sent forward rather than left believing
//!   the terms are absent.)*
//! - **§14's conformity exponent `k`** — *(BUILT as [`BELIEF_CONFORMITY_K`],
//!   and exactly as this bullet demanded: **one global constant beside the
//!   diffusion step**, not eight per-religion values. This sentence is what
//!   the implementing pass was following.)*
//! - **`SettlementReligionState`** — *(BUILT, as the type of the same name.
//!   The stateless rule still holds and is what shaped it: this crate defines
//!   the type and the step; the **caller owns the storage**, which is
//!   `cartalith_godot::CivData::belief`.)*
//! - **A stored `ReligionCultureRelation`** — storing 56 rows *is* the
//!   forbidden matrix. It is a function ([`compat`]).
//! - **Culture-side language, kinship, ritual, authority, economic and
//!   openness attributes** (§29) — zero basis in this port and zero
//!   milestone-1 caller. `syl`/`sfx` are naming pools ported "verbatim as
//!   inert lookup data"; reading a suffix list as evidence about social
//!   structure would be interpretation wearing a derivation.
//! - **A `language_affinity(a, b)` over `syl` overlap** — no milestone-1
//!   caller (milestone 1 keeps exactly three terms: exposure, compatibility,
//!   frequency), and its narrow observed range invites misreading.
//!
//! ## Three disclosures a reader must have before using [`compat`]
//!
//! **1. 31 of the 56 pairs return the stated neutral.** Both unthemed
//! cultures against every religion (10), and all three unthemed religions
//! against every culture (21). So for a *majority* of pairs `βC·Compat` is a
//! constant offset indistinguishable from moving `β0`. That is the honest
//! state of the available data, not a modelling bug — milestone 3's authored
//! culture-side data for `M`/`S`/`Q` is the named fix. A reader who assumes
//! a compatibility function discriminates everywhere will mis-calibrate `βC`
//! against a term that is flat over half its domain.
//!
//! **2. When milestone 3 authors `M`/`S`/`Q`, [`COMPAT_WEIGHTS`] must be
//! re-derived and every existing [`Compat`] value changes.** That is a
//! deliberate future re-baseline, disclosed here *now* — the way the HTML
//! CHANGELOG discloses one — rather than discovered later as a regression.
//!
//! **3. This used to warn that the consts are public before milestone 1
//! exists**, so a shell could render a [`Compat`] with no diffusion behind
//! it. **That is no longer the risk, and the replacement risk is the
//! opposite one.** [`belief_step`] exists and calls [`compat_value`] on
//! every settlement on every step, so a rendered `Compat` now does have a
//! simulation behind it. What a surface must not assume is that the
//! simulation is *running*: the layer is inert until a caller runs it, and
//! inert again — all-secular — until a faction is given a religion by hand.
//! Both states are covered by [`belief_any_faith`] and by the milestone-1
//! header's two measured disclosures below.
//!
//! ## Where the judgement calls are
//!
//! Two rows of [`CIV_RELIGION_DOMAIN`] are soft — `earth_mother → river` and
//! `sky_pantheon → hills` — and each says so in its own `basis` string,
//! names the alternative, and carries a revisit order for milestone 3. The
//! other three (`sun_cult`, `sea_lords`, `old_gods`) are near-forced by
//! their labels.
//!
//! The bijection is **content, not law**: a ninth religion sharing `coast`
//! with `sea_lords` is perfectly legal and no test asserts uniqueness,
//! because asserting it would corner milestone 3's authoring pass for no
//! benefit.
//!
//! Finally, `culture_domain_agrees_with_the_terrain_key` deliberately makes
//! a lib.rs edit break a belief.rs test. That coupling is the point — it is
//! what makes "a read, not a second opinion" checked rather than merely
//! asserted in prose — and the test's failure message says so, because it
//! would otherwise read as a spurious break.

/// The stated neutral. Not an affinity estimate: the arithmetic midpoint of
/// [`Compat`]'s declared `[0.0, 1.0]` range, returned whenever no component
/// of §9 could be evaluated. It is the numeric form of the `None` that
/// [`crate::civ_culture_terrain_fit`] returns for an unthemed culture — the
/// same "never fabricate a verdict without a real basis" discipline, in a
/// place that must hand a logistic a number.
///
/// Consistent with `relations.rs`, not contradicting it: its `religion_term`
/// gives `none` a `0.0` on a `[-1, 1]` scale; this gives it `0.5` on a
/// `[0, 1]` scale. Same semantics — silence, not division — different number
/// because the scales differ. Do not "fix" the discrepancy.
pub const NEUTRAL_COMPAT: f64 = 0.5;

/// §9 has five components: `L`, `K`, `M`, `S`, `Q`. This module evaluates
/// one of them.
pub const COMPAT_COMPONENTS: usize = 5;

/// §9's `w1..w5`, in `[L, K, M, S, Q]` order.
///
/// **A weight exists only where its component can exist.** There is no
/// fabricated `w1`/`w3`/`w4`/`w5` sitting inert in this file waiting to be
/// mistaken for a calibrated value. `K` carries the whole weight today
/// because it is the only component with a basis; when milestone 3 authors a
/// culture-side datum for `M`, `S` or `Q`, this array and [`compat`]'s
/// `components` array are the two places that change, and
/// `weights_exist_exactly_where_components_can` is the guard that stops them
/// drifting apart.
pub const COMPAT_WEIGHTS: [Option<f64>; COMPAT_COMPONENTS] = [None, Some(1.0), None, None, None];

/// A culture's terrain domain, for §9's `K`.
///
/// **This is a read of [`crate::CIV_CULTURE_TERRAIN_KEY`], not a second
/// opinion about it.** That table already declares highland→hills,
/// desert→arid, riverlands→river, sylvan→forest, maritime→coast, and already
/// declares `common`/`imperial` to have no verdict. Duplicating those five
/// rows here would add no information and would create exactly the drift
/// this module's doc names as the cost of a parallel table. Reading it makes
/// drift structurally impossible rather than merely test-enforced, and the
/// `None` for `common`/`imperial` is inherited from that table's own
/// deliberate omission rather than re-decided here.
///
/// Returns `None` for an unknown key, matching
/// [`crate::civ_culture_terrain_fit`]'s already-pinned behaviour
/// (`culture_terrain_fit_unknown_key_returns_none` in lib.rs).
pub fn culture_domain(culture_key: &str) -> Option<&'static str> {
    crate::CIV_CULTURE_TERRAIN_KEY
        .iter()
        .find(|&&(c, _)| c == culture_key)
        .map(|&(_, t)| t)
}

/// One row of [`CIV_RELIGION_DOMAIN`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReligionDomain {
    /// A key from [`crate::roster::CIV_RELIGIONS`].
    pub religion: &'static str,
    /// A key from [`crate::CIV_TERRAIN_MIX_KEYS`] — the same vocabulary
    /// [`crate::CIV_CULTURE_TERRAIN_KEY`] uses, which is what lets [`Compat`]
    /// be a comparison rather than a 56-cell matrix.
    pub terrain: &'static str,
    /// The sentence naming the evidence, following the reference's own v1.35
    /// `basis` field that [`crate::CIV_CULTURE_TERRAIN_KEY`]'s doc comment
    /// cites as the precedent, and this crate's live `basis: String` on
    /// [`crate::RestDays`]. Non-empty, test-enforced. This is what lets the
    /// owner overturn one row in milestone 3 without re-deriving the table.
    pub basis: &'static str,
}

/// A religion's terrain domain — the mirror of
/// [`crate::CIV_CULTURE_TERRAIN_KEY`], in the same shape, over the same
/// vocabulary, with the same deliberate omissions.
///
/// **The evidence is the label.** [`crate::roster::CIV_RELIGIONS`] is eight
/// names and nothing else — `RELIGION_DIFFUSION_SCOPE.md` §2 says so in as
/// many words (*"`CIV_RELIGIONS` is eight names, nothing more"*). Reading
/// "Sea Lords" as maritime is parsing authored content; inventing a
/// `C_ritual` for it is fabrication. That line is where this table stops.
///
/// **Three religions are deliberately absent**, exactly as `common` and
/// `imperial` are absent from the culture table and for the same stated
/// reason — no thematic basis, so no verdict rather than a fabricated one:
///
/// - `none` — not a religion but the absence of one. `relations.rs`'
///   `religion_term` already treats it as silence rather than division; a row
///   here would make secularism a competing faith.
/// - `ancestor_rites` — kinship-themed, not terrain-themed. Ancestors are
///   carried by a lineage, not by a landscape.
/// - `flame_creed` — the strongest of the three omissions and the least
///   arguable: "flame" is simply not one of the five terrain keys, and
///   inventing a sixth to house it would fabricate the vocabulary itself.
///
/// **This table is injective onto the five terrain keys today.** That is what
/// makes equality a complete relation and why no graded affinity table is
/// needed. It is a property of the current content, *not a law* — a ninth
/// religion sharing `coast` with `sea_lords` is perfectly legal, and no test
/// asserts uniqueness, because doing so would corner milestone 3's authoring
/// pass for no benefit.
pub const CIV_RELIGION_DOMAIN: [ReligionDomain; 5] = [
    ReligionDomain {
        religion: "sun_cult",
        terrain: "arid",
        basis: "\"Sun Cult\" -- the sun as the ruling power of the waste rather \
                than merely its light; `arid` is the one terrain key the sun \
                defines. Near-forced by the label.",
    },
    ReligionDomain {
        religion: "earth_mother",
        terrain: "river",
        basis: "\"Earth Mother\" -- an agrarian fertility deity; `river` is this \
                vocabulary's only cultivable-floodplain key. SOFTEST CALL IN \
                THIS TABLE: `forest` is defensible. Revisit first in \
                milestone 3.",
    },
    ReligionDomain {
        religion: "sea_lords",
        terrain: "coast",
        basis: "\"Sea Lords\" -- forced by the label; `coast` is the \
                vocabulary's only maritime key.",
    },
    ReligionDomain {
        religion: "sky_pantheon",
        terrain: "hills",
        basis: "\"Sky Pantheon\" -- high places as the sky's ground; `hills` is \
                the vocabulary's only elevation key. SECOND-SOFTEST: a pantheon \
                is arguably civic rather than terrain-themed and could be \
                dropped to no-verdict, which would leave `hills`/highland with \
                no matching religion -- honest but degenerate. Revisit second \
                in milestone 3.",
    },
    ReligionDomain {
        religion: "old_gods",
        terrain: "forest",
        basis: "\"Old Gods\" -- old growth; the pre-institutional gods of the \
                wood. `forest` is the vocabulary's only wild-vegetation key.",
    },
];

/// `None` means "no thematic basis", not "unknown" — see
/// [`CIV_RELIGION_DOMAIN`]'s own doc. An unknown or misspelled key also
/// yields `None`; the guard against a typo *in the table* is
/// `religion_domains_are_shared_vocabulary`, at build time. A typo in a
/// caller's argument is the caller's bug and routes to [`NEUTRAL_COMPAT`],
/// which is the correct treatment of an unrecognised key.
pub fn religion_domain(religion_key: &str) -> Option<&'static str> {
    CIV_RELIGION_DOMAIN
        .iter()
        .find(|r| r.religion == religion_key)
        .map(|r| r.terrain)
}

/// Why a [`Compat`] came out the way it did — the machine-readable half of
/// the "show its work" contract this crate already keeps with
/// `SuitExplanation` and `FactionRelation`. Five variants, exhaustive over
/// the four (domain-present × domain-present) cases plus the shared-domain
/// split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatBasis {
    /// Both sides declare a domain and it is the same one. `K = 1.0`.
    SharedDomain,
    /// Both sides declare a domain and they differ. `K = 0.0`.
    DifferentDomain,
    /// The religion has no thematic basis (`none`, `ancestor_rites`,
    /// `flame_creed`, or an unrecognised key). No component evaluable.
    ReligionUnthemed,
    /// The culture has no thematic basis (`common`, `imperial`, or an
    /// unrecognised key). No component evaluable.
    CultureUnthemed,
    /// Neither side declares a domain.
    NeitherThemed,
}

/// §9's `Compat = w1·L + w2·K + w3·M + w4·S + w5·Q`, with the four components
/// that have no basis in this port carried as *declared absences* rather than
/// invented values.
///
/// - `l` — **structurally `None`.** No religion in this port has a liturgical
///   language; nothing in the eight labels implies one, so `L` is not a
///   function of `(R, C)` at all.
/// - `k` — cosmological. The one evaluated component: `Some` iff both sides
///   declare a terrain domain.
/// - `m` — **structurally `None`.** No culture-side moral-order datum exists
///   anywhere in the port. Half a term is not a term.
/// - `s` — **structurally `None`.** Same asymmetry: no culture-side "how much
///   formal ritual does this culture already practise" datum exists.
/// - `q` — **structurally `None`.** No culture-side institutional datum
///   exists. [`crate::Culture`] is `key`/`syl`/`sfx`; a suffix pool is naming
///   content, not evidence about social structure.
///
/// `value` is **always a number** so the §19 logistic never has to invent
/// one, and `evaluated` is what stops a UI presenting a one-of-five score as
/// a full verdict. Render `evaluated` of [`COMPAT_COMPONENTS`] beside it, or
/// do not render `value` at all.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Compat {
    pub l: Option<f64>,
    pub k: Option<f64>,
    pub m: Option<f64>,
    pub s: Option<f64>,
    pub q: Option<f64>,
    /// `Σ(wᵢ·xᵢ) / Σ(wᵢ)` over present components, or [`NEUTRAL_COMPAT`] when
    /// none is present. In `[0.0, 1.0]`.
    pub value: f64,
    /// How many of [`COMPAT_COMPONENTS`] actually contributed. `0` or `1`
    /// today.
    pub evaluated: u8,
    pub basis: CompatBasis,
}

/// §9's `Compat(R, C)`, and §24's interaction matrix computed rather than
/// typed.
///
/// **`culture_term`-shaped, per milestone 1's own wording** ("the existing
/// `culture_term`-shaped compatibility"). `relations.rs` is literally
/// `if !ca.is_empty() && ca == cb { 1.0 } else { 0.0 }` — a `[0, 1]` bonus
/// with no negative branch. This enters §19 as `+βC·Compat`, so `0.0` means
/// *no compatibility bonus*, never a hostility claim. That is what lets a
/// hard `0.0` for a genuine theme clash be honest rather than invented.
///
/// **Absence of evidence outranks evidence of mismatch, on purpose.** A
/// themed religion meeting an unthemed culture gets [`NEUTRAL_COMPAT`]
/// (`0.5`), which is *higher* than the `0.0` a themed culture with a
/// different theme gets. That ordering is deliberate and is exactly the
/// information [`crate::civ_culture_terrain_fit`] returns `None` to protect.
///
/// Float discipline: the only literals are `0.0`, `0.5`, `1.0` and a weight
/// of `1.0`, all exactly representable, and `1.0 * x / 1.0 == x` exactly. No
/// transcendental, no `HashMap`, fixed-order iteration over a fixed-size
/// array — determinism is structural and no `js_*` helper is needed.
/// Milestone 1's logistic will need `js_exp`; that is where float discipline
/// actually bites, not here.
pub fn compat(religion_key: &str, culture_key: &str) -> Compat {
    let (k, basis) = match (religion_domain(religion_key), culture_domain(culture_key)) {
        (Some(r), Some(c)) if r == c => (Some(1.0), CompatBasis::SharedDomain),
        (Some(_), Some(_)) => (Some(0.0), CompatBasis::DifferentDomain),
        (None, Some(_)) => (None, CompatBasis::ReligionUnthemed),
        (Some(_), None) => (None, CompatBasis::CultureUnthemed),
        (None, None) => (None, CompatBasis::NeitherThemed),
    };

    // [L, K, M, S, Q]. Milestone 3 flips a `None` here and the matching slot
    // in COMPAT_WEIGHTS; nothing else in this function changes.
    let components: [Option<f64>; COMPAT_COMPONENTS] = [None, k, None, None, None];

    // The weighted mean is the one place this module spends beyond today's
    // strict minimum (`k.unwrap_or(NEUTRAL_COMPAT)` would do). It is not dead
    // code -- it genuinely computes today's answer -- and it is what makes
    // milestone 3 a two-line edit rather than a rewrite. The `evaluated == 0`
    // guard is load-bearing: without it `den == 0.0` yields a NaN that would
    // escape into §19's logistic.
    let mut num = 0.0f64;
    let mut den = 0.0f64;
    let mut evaluated: u8 = 0;
    for i in 0..COMPAT_COMPONENTS {
        if let (Some(x), Some(w)) = (components[i], COMPAT_WEIGHTS[i]) {
            num += w * x;
            den += w;
            evaluated += 1;
        }
    }
    let value = if evaluated == 0 {
        NEUTRAL_COMPAT
    } else {
        num / den
    };

    Compat {
        l: components[0],
        k: components[1],
        m: components[2],
        s: components[3],
        q: components[4],
        value,
        evaluated,
        basis,
    }
}

/// [`compat`]'s `value` alone, for §19's logistic — which needs a scalar and
/// has no use for the provenance.
pub fn compat_value(religion_key: &str, culture_key: &str) -> f64 {
    compat(religion_key, culture_key).value
}

// ===========================================================================
// Milestone 1 — network exposure and conversion
// ===========================================================================
//
// Everything below `compat_value` is `RELIGION_DIFFUSION_SCOPE.md` §3
// milestone 1: *"network exposure and conversion, read-only."* It is the
// half this module's own doc above says is missing, and the three things
// that doc says milestone 1 would need — `p_conv`, §14's `k`, and
// `SettlementReligionState` — are here, each in the place that doc names.
//
// ## This is divergence by addition, not a port
//
// **Checked, not assumed:** `reference/FUNCTION_INDEX.md` has no religious
// diffusion, adherence or conversion function of any kind. Its only `Faith`
// entry is `buildFaithSites` (line 30588), the *city* generator placing
// churches and shrines on parcels — a building placer, not a belief model.
// The reference's religion is one categorical string per faction and the
// reference author declined this layer on purpose (`RELIGION_DIFFUSION_
// SCOPE.md` §0 quotes its own v1.10 changelog doing so).
//
// So **no golden test can exist for any number below**, there is nothing to
// re-baseline, and nothing here claims otherwise. The values are authored
// model assumptions in §30's sense, pinned by literal and by behaviour in
// this file's own tests.
//
// ## Nothing that generates a world calls any of this
//
// The layer is inert until a caller runs it: `compute_civilisation` does not
// build it, no `WorldParams` field switches it, and no existing value moves.
// That is why this ships with **no `cartalith_engine::WorldParams::defaults()`
// divergence flag and no `PARAMS`/`JS_PATHS` row** — the app-boundary pattern
// exists for a *generation-time* divergence, and this is not one. Adding a
// flag that nothing reads would be the fabricated-value habit in a different
// costume. When milestone 5 makes a state religion *derived* — the fork
// `RELIGION_DIFFUSION_SCOPE.md` §4 leaves open — that is the change that will
// move existing output, and that is where the flag belongs.
//
// ## Two measured properties a surface must be designed against
//
// Both were found by running this model, not predicted from it, and both
// change what a Religion screen can honestly show:
//
// 1. **A freshly generated world has no religion in it at all.**
//    `civ_roster_bridge::FactionEntry::default_for` sets `religion: "none"`
//    for every faction and nothing but the player's own Faction Inspector
//    edit ever changes that, so [`belief_seed`] seeds 100 % unaffiliated
//    everywhere and the all-secular state is a fixed point. The layer does
//    nothing until a faction is given a religion by hand. That is a
//    *configuration* state with a fix the user can act on, and it is not the
//    same thing as "the diffusion has not been run" — [`belief_any_faith`]
//    exists so a surface can tell them apart.
// 2. **There is no stable mixture.** §14's `p^k` with `k > 1` is a fixation
//    dynamic, so every settlement converges to one religion holding
//    essentially all of it — measured at >99 % even for a town placed
//    exactly between two equal rival neighbours in a culture favouring
//    neither. Coexistence is §18's `Competition` and §22's separate
//    retention function, both later milestones with no input in this port.
//    A breakdown widget designed around an imagined five-way split would be
//    designed against a model that does not exist yet.
//
// ## No RNG stream, and why that is the stronger answer
//
// Every function below is deterministic: fixed-order iteration over
// fixed-size arrays, no map, no draw. Two worlds with the same settlements,
// ways and roster get identical adherence, and it does not even depend on
// the world seed except through the settlements themselves. So there is no
// `BELIEF_RNG_SEED_INPUT` here — `crate::labels::CIV_LAKE_NAME_RNG_SEED_INPUT`
// exists because two *naming* streams sharing one fixed-seed generator hand
// out the same first word, and a stream with no draws in it cannot collide
// with anything. The one place a draw would have been reached for is
// splitting a fractional adherent count, and
// [`SettlementReligionState::adherents`] does that by largest remainder,
// which is both deterministic and exactly population-conserving where a
// draw would be neither.

/// `crate::roster::CIV_RELIGIONS`' length, as an array bound.
pub const CIV_RELIGION_COUNT: usize = crate::roster::CIV_RELIGIONS.len();

/// `CIV_RELIGIONS[0]` is `("none", "None / secular")`, and this model uses
/// that slot as the **unaffiliated share of the population**, not as an
/// absent value.
///
/// That distinction is the whole reason this is a named constant. `share[0]
/// == 0.4` means *"40 % of these people follow no religion"* — a measured
/// quantity the paper needs (§20's `ΔR` has to conserve population, and
/// §21's deconversion has to have somewhere to go). *"This world has no
/// belief layer"* is a different statement entirely and is carried by an
/// **empty state vector**, never by a share of zero.
pub const RELIGION_NONE: usize = 0;

/// §19's `βE` — exposure. The largest of the three, because it is the only
/// term with a network behind it (§4 is the whole point of the model) and
/// because §19 lists it first.
pub const BELIEF_BE: f64 = 3.0;

/// §19's `βC` — compatibility. Second, on §9's own claim that this is *"one
/// of the most important interaction terms in the system."*
pub const BELIEF_BC: f64 = 2.0;

/// §19's `βF` — frequency/conformity. Deliberately the smallest of the
/// three: §14's feedback loop (*"more adherents → more conformity → more
/// adherents"*) is the one term that reinforces its own input, so a weight
/// at parity with the others locks a settlement onto its founding faith on
/// the first step and the network never gets to do anything.
///
/// The 3 : 2 : 1 ordering is the authored part. The *sum* is not free — see
/// [`BELIEF_B0`].
pub const BELIEF_BF: f64 = 1.0;

/// §19's intercept, and the one constant here that is **derived rather than
/// chosen**: `-(βE + βC + βF) / 2`.
///
/// All three inputs are normalized to `[0, 1]` (§30's own instruction), so
/// the linear predictor spans `[β0, β0 + βE + βC + βF]`. Setting `β0` to
/// minus half that span centres the logistic on the domain: a settlement at
/// the midpoint of all three inputs gets `σ(0) = 0.5`, and the span becomes
/// `[-3, +3]`, i.e. `σ ∈ [0.047, 0.953]` — the part of the logistic that
/// still responds to its argument. A larger sum saturates both ends and the
/// three terms stop mattering; a smaller one flattens the whole model toward
/// `0.5` and the network stops mattering.
///
/// `belief_is_centred_on_its_own_input_domain` asserts the relationship
/// rather than the number, so changing a `β` without changing this one is a
/// test failure and not a silent recentring.
pub const BELIEF_B0: f64 = -(BELIEF_BE + BELIEF_BC + BELIEF_BF) / 2.0;

/// §14's conformity exponent `k`, in `T_R ∝ p_R^k`.
///
/// **Not tuned.** §14's own text requires `k > 1` for the term to be
/// conformist at all; this is the smallest integer above that bound. Picking
/// `2.7` instead would be a calibration claim this port has nothing to
/// calibrate against, which is exactly what §30 says not to do. The module
/// doc above already reserved this as *"one **global** calibration constant
/// belonging beside the diffusion step, **not** eight per-religion values"* —
/// this is that constant, in that place.
pub const BELIEF_CONFORMITY_K: f64 = 2.0;

/// The share of a settlement's population that can change allegiance in one
/// [`belief_step`], i.e. in one Timeline year.
///
/// Pinned by **behaviour, not by itself**:
/// `belief_takeover_is_a_generation_where_it_happens_at_all` measures how
/// long a religion takes to carry a settlement from a 1 % minority to a
/// plurality in three configurations and asserts each measured year count as
/// a literal, so any change to this constant or to a `β` moves a test. A rate
/// that converts a province in a season is a conquest, and §25 is explicit
/// that the model must not produce *"king converts → everyone converts
/// immediately."*
///
/// **The measured range is 17-25 years where conversion happens at all, and
/// "never" where it does not** — see that test and
/// `belief_has_no_syncretic_equilibrium` for why there is no slow-but-
/// eventual middle, and why that is §14 rather than a defect.
pub const BELIEF_STEP_RATE: f64 = 0.05;

/// §19's `σ(x) = 1/(1+e^−x)`.
///
/// `js_exp`, not `f64::exp` — the module doc above named this as the one
/// place in the belief subsystem where float discipline actually bites, and
/// `cartalith-jsmath`'s own doc records that milestone 5's first golden run
/// failed on a one-ulp `exp`. Nothing here is golden-tested, but a divergent
/// `exp` inside a feedback loop compounds, and having two `exp`s in one
/// workspace is the drift `JS_SEMANTICS_AUDIT.md` recommendation #2 exists
/// to stop.
pub fn belief_logistic(x: f64) -> f64 {
    1.0 / (1.0 + cartalith_jsmath::js_exp(-x))
}

/// A religion key's index in `crate::roster::CIV_RELIGIONS`, or `None` for a
/// key that vocabulary does not contain.
pub fn religion_index(key: &str) -> Option<usize> {
    crate::roster::CIV_RELIGIONS
        .iter()
        .position(|(k, _)| *k == key)
}

/// The inverse of [`religion_index`].
pub fn religion_key(index: usize) -> Option<&'static str> {
    crate::roster::CIV_RELIGIONS.get(index).map(|(k, _)| *k)
}

/// §29's `SettlementReligionState`, cut down to milestone 1's half of it.
///
/// The paper's full struct also carries institutional presence, clergy,
/// missionaries, prestige, political support and competition. Those are
/// milestones 2, 4, 5 and 6, and none of them has an input in this port yet,
/// so none of them is here — the same "a field exists only where its
/// evidence can exist" rule [`COMPAT_WEIGHTS`] follows.
///
/// # Shares, not counts, and why the storage type is the design
///
/// `share` sums to `1.0` and is indexed by `crate::roster::CIV_RELIGIONS`
/// position, with [`RELIGION_NONE`] at `0` carrying the unaffiliated
/// remainder. Adherent *counts* are derived on demand by
/// [`Self::adherents`].
///
/// Storing counts instead would quietly delete every minority. A hamlet of
/// 120 with a 0.4 % foreign community is 0.48 people; rounded to an integer
/// each step it is 0, and once it is 0 no amount of exposure brings it back,
/// because every term in [`belief_step`] is multiplicative in the share.
/// S-shaped diffusion (§14) *starts* from a seed that small — killing it is
/// killing the mechanism the milestone exists to demonstrate.
#[derive(Debug, Clone, PartialEq)]
pub struct SettlementReligionState {
    pub share: [f64; CIV_RELIGION_COUNT],
}

impl SettlementReligionState {
    /// A settlement wholly of one religion. Out-of-range indices fall to
    /// [`RELIGION_NONE`], which is the honest reading of an unrecognised
    /// religion key: unaffiliated, not a ninth faith.
    pub fn wholly(index: usize) -> Self {
        let mut share = [0.0; CIV_RELIGION_COUNT];
        share[if index < CIV_RELIGION_COUNT { index } else { RELIGION_NONE }] = 1.0;
        SettlementReligionState { share }
    }

    /// The largest share's index, ties going to the lower index so the
    /// answer is stable across runs and across machines.
    ///
    /// Note what this deliberately does **not** do: it does not skip
    /// [`RELIGION_NONE`]. A settlement that is 60 % unaffiliated has a
    /// plurality of `none`, and reporting the largest *faith* instead would
    /// be the "encode no value as a plausible value" failure in reverse —
    /// a majority-secular town would be labelled with a religion four in ten
    /// of its people follow.
    pub fn plurality(&self) -> usize {
        let mut best = 0usize;
        for i in 1..CIV_RELIGION_COUNT {
            if self.share[i] > self.share[best] {
                best = i;
            }
        }
        best
    }

    /// Adherent head-counts for a settlement of `pop`, by largest remainder.
    ///
    /// **Exactly** `pop` in total, for every input: floors first, then the
    /// leftover handed out one at a time to the largest fractional parts
    /// (ties to the lower index). Rounding each share independently would
    /// return 119 or 121 people for a town of 120 depending on the split,
    /// and a settlement inspector that does not add up is a defect a reader
    /// will find before any test does.
    pub fn adherents(&self, pop: u32) -> [u32; CIV_RELIGION_COUNT] {
        let mut out = [0u32; CIV_RELIGION_COUNT];
        let mut frac = [(0.0f64, 0usize); CIV_RELIGION_COUNT];
        let mut assigned: u64 = 0;
        for i in 0..CIV_RELIGION_COUNT {
            let raw = self.share[i].max(0.0) * pop as f64;
            let floor = raw.floor();
            let whole = if floor.is_finite() && floor >= 0.0 {
                (floor as u64).min(pop as u64)
            } else {
                0
            };
            out[i] = whole as u32;
            assigned += whole;
            frac[i] = (raw - floor, i);
        }
        // `assigned` can only undershoot: every term floored a non-negative
        // product whose exact sum is `pop`. The `min` above is what makes
        // that true even for a malformed share array.
        let mut left = (pop as u64).saturating_sub(assigned);
        // Descending by fractional part, ascending by index on a tie. Sorted
        // by an explicit comparator rather than `sort_by_key`, because the
        // key is an `f64`.
        frac.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal).then(a.1.cmp(&b.1)));
        let mut cursor = 0usize;
        while left > 0 && cursor < CIV_RELIGION_COUNT {
            out[frac[cursor].1] += 1;
            left -= 1;
            cursor += 1;
        }
        out
    }
}

/// One undirected link in §3's connectivity graph `G`.
///
/// `a`/`b` index a settlement list; `km` is the real routed length, not a
/// straight line. This is deliberately narrower than `crate::Way`: sea lanes
/// (`crate::SeaRoute`) carry no endpoint indices at all, so when a later
/// milestone adds a maritime term the new edges arrive through this same type
/// rather than through a second signature.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeliefLink {
    pub a: usize,
    pub b: usize,
    pub km: f64,
}

/// §3's graph, taken from the road network this port already generates —
/// `RELIGION_DIFFUSION_SCOPE.md` §2's *"reuse it, don't rebuild it."*
///
/// **Hidden ways are included.** `Way::hidden` means "this edge was
/// consolidated into a busier neighbour and must not be *drawn* twice"; the
/// two settlements are still connected by road. Dropping them would make
/// religion stop at a junction for a rendering reason.
///
/// **Sea lanes are not here, and are not faked.** `crate::SeaRoute` has
/// `pts`/`brks`/`km`/`name` and no `a_idx`/`b_idx` — there is no endpoint
/// index to build a link from without re-deriving one by nearest-point
/// search, which is a real piece of work and belongs to whichever milestone
/// wants a maritime term, not to a helper. So this port's religion spreads
/// overland only, and that is a stated limitation rather than a silent one.
pub fn belief_links_from_ways(ways: &[crate::Way]) -> Vec<BeliefLink> {
    ways.iter()
        .filter(|w| w.a_idx != w.b_idx)
        .map(|w| BeliefLink { a: w.a_idx, b: w.b_idx, km: w.km })
        .collect()
}

/// §3's `G_ij`, resolved and cached: for each settlement, its weighted
/// neighbours.
///
/// # The weight is the reference's own carriage decay, not a new constant
///
/// `w_ij = pop_j · deliverable(km_ij, Land)`, where
/// [`crate::trade::deliverable`] is the port of `_civFoodDeliverable`
/// (reference line 24004): `2^(−km/160)` with a hard `0` past 220 km.
/// Reaching for that rather than authoring a `BELIEF_DECAY_KM` is the point
/// — it is the reference's own answer to *"how far does overland contact
/// reach"*, it is already golden-tested, and §5.2 names trade as a principal
/// diffusion mechanism, so it is the right curve and not merely a convenient
/// one. The 220 km cutoff means a long way contributes exactly nothing,
/// which is a reference-sourced bound rather than an invented one.
///
/// The `pop_j` factor is §3's own sentence — *"a distant major port can have
/// substantially more contact with another settlement than a nearby isolated
/// village"* — and it is what stops this being the distance-only model §3's
/// first line rejects.
///
/// # Every settlement is its own neighbour, at distance zero
///
/// [`Self::build`] seeds each node with `w_ii = pop_i` (`deliverable(0) ==
/// 1`). That is §5.3's local kinship/household channel, which is otherwise
/// missing, and it is what makes the exposure vector below a genuine
/// probability distribution — `Σ_r E_{i,r} == 1` even for an isolated
/// settlement with no links at all. Without it an unconnected settlement has
/// zero exposure to everything including its own faith, and the step has to
/// invent a rule for that case.
#[derive(Debug, Clone, PartialEq)]
pub struct BeliefNetwork {
    /// `adj[i]` = `(j, w_ij)`, including `(i, pop_i)`.
    adj: Vec<Vec<(usize, f64)>>,
}

impl BeliefNetwork {
    pub fn build(pops: &[u32], links: &[BeliefLink]) -> Self {
        let n = pops.len();
        let mut adj: Vec<Vec<(usize, f64)>> = (0..n).map(|i| vec![(i, pops[i] as f64)]).collect();
        for l in links {
            if l.a >= n || l.b >= n || l.a == l.b {
                continue;
            }
            let w = crate::trade::deliverable(l.km, crate::trade::TradeMode::Land);
            // Negated, like `trade::deliverable`'s own guard and for the
            // same reason: `!(w > 0.0)` is true for NaN where `w <= 0.0` is
            // false (`cartalith-rust-conventions`). `deliverable` maps a NaN
            // km to `0.0`, so this is belt and braces -- but the belt is one
            // line and the alternative is a NaN weight poisoning every
            // exposure sum this node ever contributes to.
            #[allow(clippy::neg_cmp_op_on_partial_ord)]
            if !(w > 0.0) {
                continue;
            }
            adj[l.a].push((l.b, pops[l.b] as f64 * w));
            adj[l.b].push((l.a, pops[l.a] as f64 * w));
        }
        BeliefNetwork { adj }
    }

    pub fn len(&self) -> usize {
        self.adj.len()
    }

    pub fn is_empty(&self) -> bool {
        self.adj.is_empty()
    }

    /// How many settlements `i` draws exposure from, itself included. `1`
    /// means "no reachable neighbour", which is a real answer for an island
    /// or a settlement past every link's 220 km reach.
    pub fn degree(&self, i: usize) -> usize {
        self.adj.get(i).map_or(0, |v| v.len())
    }
}

/// §4's `E_{i,r} = Σ_j G_ij R_{j,r}`, normalized: a
/// population-and-distance-weighted mean prevalence over `{i} ∪ N(i)`.
///
/// Normalizing is §30's instruction (*"dimensionless normalized parameters
/// 0 ≤ x ≤ 1"*) and it is load-bearing rather than cosmetic: the raw sum is
/// in units of people and would range over four orders of magnitude between
/// a hamlet of 120 and a metropolis of 45 000, which no single `βE` can be
/// calibrated for. `Σ_r` of the returned array is `1.0`.
///
/// The paper's §4 additional terms — `M` missionary, `I` institutional, `X`
/// direct — are **absent**, per milestone 1's own text (*"defer
/// missionary/institutional/direct terms"*). §26's centrality multiplier is
/// absent for the same reason and a second one: `pop_j` weighting already
/// makes hubs hubs, and stacking a centrality factor on top would count the
/// same fact twice.
fn belief_exposure(
    net: &BeliefNetwork,
    states: &[SettlementReligionState],
    i: usize,
) -> [f64; CIV_RELIGION_COUNT] {
    let mut e = [0.0f64; CIV_RELIGION_COUNT];
    let mut total = 0.0f64;
    for &(j, w) in net.adj[i].iter() {
        total += w;
        for (r, slot) in e.iter_mut().enumerate() {
            *slot += w * states[j].share[r];
        }
    }
    if total > 0.0 {
        for slot in e.iter_mut() {
            *slot /= total;
        }
    } else {
        // Every neighbour, this settlement included, has population zero.
        // There is nobody to be exposed to anything, so the distribution
        // that changes nothing is this settlement's own -- not a uniform
        // eighth each, which would invent seven congregations out of an
        // empty town.
        e = states[i].share;
    }
    e
}

/// One diffusion step — one Timeline year — over every settlement at once.
///
/// # What this computes
///
/// For settlement `i` and religion `r`, with `E` from [`belief_exposure`],
/// `Compat` from [`compat_value`] against `i`'s own culture, and
/// `Freq = share_i[r]^k` (§14):
///
/// ```text
/// attract_r = E_r · σ(β0 + βE·E_r + βC·Compat_r + βF·Freq_r)
/// share'_r  = (1 − rate)·share_r + rate·(attract_r / Σ attract)
/// ```
///
/// **The leading `E_r` factor is the part that is not decoration.** §19's
/// logistic never returns zero anywhere this model can reach — its whole
/// domain is `σ ∈ [0.047, 0.953]` (see [`BELIEF_B0`]) — so a bare `σ(…)`
/// gives every one of the eight religions a nonzero pull in every settlement
/// on every step, and a sealed valley that has never met an outsider grows a
/// Sea Lords congregation out of arithmetic. Multiplying by exposure is
/// §17's own shape (`P_conv = 1 − e^{−kE}` is likewise zero at zero
/// exposure) and it makes "nobody here has heard of it" mean what it says.
///
/// # Read this before calibrating: three terms, not nine
///
/// §19 lists nine. This has three, exactly the three milestone 1 names, and
/// the six that are missing are missing because this port has no input for
/// them, not because they were judged unimportant. In particular
/// `Competition` is absent, so competition here is only whatever the shared
/// `Σ attract` denominator provides — a religion is crowded out by rivals
/// but is never specifically *hostile* to one, which is what §18's `w_Rq`
/// would add.
///
/// And [`compat_value`]'s own disclosure applies with full force: **31 of
/// the 56 (religion, culture) pairs return the stated neutral**, so over the
/// majority of the domain `βC·Compat` is a constant offset indistinguishable
/// from moving `β0`. Anyone calibrating `βC` against this model is
/// calibrating a term that is flat over half of it.
///
/// # Conversion and retention are one number here, on purpose
///
/// The paper's §2 insists exposure ≠ conversion ≠ retention, and this
/// collapses the last two into a single net `rate`. That is milestone 1's
/// own stated simplification, *"acceptable only because this milestone
/// exists to prove the network-diffusion mechanic before spending complexity
/// budget on the split."* Milestone 2 is the split. Nothing here should be
/// read as a claim about retention.
///
/// # Synchronous, not in place
///
/// Exposure for every settlement is read from the *old* state before any
/// share is written, so the result does not depend on settlement order. An
/// in-place sweep would let a religion cross three towns in one year at one
/// end of the list and one town at the other.
///
/// `culture_of` is one culture key per settlement — the caller's job, since
/// culture lives on the faction roster, which is boundary state this crate
/// cannot see (`ARCHITECTURE.md`). A short slice, or a key this port's
/// culture vocabulary does not contain, routes to [`NEUTRAL_COMPAT`] through
/// [`compat`]'s own unrecognised-key path.
pub fn belief_step(
    states: &mut [SettlementReligionState],
    net: &BeliefNetwork,
    culture_of: &[&str],
    rate: f64,
) {
    let n = states.len().min(net.len());
    if n == 0 {
        return;
    }
    let rate = rate.clamp(0.0, 1.0);
    let prev: Vec<SettlementReligionState> = states[..n].to_vec();
    for i in 0..n {
        let e = belief_exposure(net, &prev, i);
        let culture = culture_of.get(i).copied().unwrap_or("");
        let mut attract = [0.0f64; CIV_RELIGION_COUNT];
        let mut total = 0.0f64;
        for (r, slot) in attract.iter_mut().enumerate() {
            if e[r] <= 0.0 {
                continue;
            }
            let key = religion_key(r).unwrap_or("");
            let c = compat_value(key, culture);
            let freq = prev[i].share[r].max(0.0).powf(BELIEF_CONFORMITY_K);
            let a = e[r]
                * belief_logistic(BELIEF_B0 + BELIEF_BE * e[r] + BELIEF_BC * c + BELIEF_BF * freq);
            *slot = a;
            total += a;
        }
        // Negated for the NaN case, as in `BeliefNetwork::build` above.
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(total > 0.0) {
            // Unreachable while `Σ_r E_r == 1` -- some `E_r` is positive, and
            // the logistic never returns zero. Kept because the invariant
            // lives in another function: if `belief_exposure` ever returns an
            // all-zero vector, the alternative here is a `0/0` NaN escaping
            // into every downstream share.
            continue;
        }
        let mut sum = 0.0f64;
        for (r, slot) in states[i].share.iter_mut().enumerate() {
            let v = (1.0 - rate) * prev[i].share[r] + rate * (attract[r] / total);
            *slot = v;
            sum += v;
        }
        // Renormalize. The algebra sums to exactly 1.0; eight f64 additions
        // of it do not, and the drift compounds over a century of steps.
        if sum > 0.0 {
            for r in 0..CIV_RELIGION_COUNT {
                states[i].share[r] /= sum;
            }
        }
    }
}

/// Milestone 1's seeding rule: every settlement starts wholly in its
/// founding faction's state religion.
///
/// `faction_of` is one faction id per settlement (`0` = Unclaimed);
/// `faction_religion` is one religion key per faction id, index-aligned with
/// the roster, so `faction_religion[0]` is Unclaimed's.
///
/// # This produces an all-`none` world by default, and that is the finding
///
/// `civ_roster_bridge::FactionEntry::default_for` sets `religion: "none"` for
/// **every** faction, generation included — the reference's own module-load
/// default, and nothing in either codebase ever writes it except the
/// player, through the Faction Inspector's Religion dropdown. So a freshly
/// generated world seeds 100 % unaffiliated everywhere, and this model's
/// correct behaviour on it is to sit still forever: every settlement's only
/// exposure is to `none`, so `none` is the only religion with a nonzero
/// pull, and the fixed point is the seed.
///
/// That is not a bug in the seeding rule and it is not fixed by inventing a
/// default religion per faction — `CIV_RELIGIONS[i % 8]` would be exactly
/// the fabricated content [`CIV_RELIGION_DOMAIN`]'s doc refuses. It is a real
/// consequence of `RELIGION_DIFFUSION_SCOPE.md` milestone 1 choosing the
/// least invasive seed, and it means **the diffusion layer does nothing
/// until a faction is given a religion by hand.** A surface built on this
/// must say so in those words rather than render an empty breakdown.
pub fn belief_seed(faction_of: &[i32], faction_religion: &[&str]) -> Vec<SettlementReligionState> {
    faction_of
        .iter()
        .map(|&f| {
            let idx = usize::try_from(f)
                .ok()
                .and_then(|f| faction_religion.get(f))
                .and_then(|k| religion_index(k))
                .unwrap_or(RELIGION_NONE);
            SettlementReligionState::wholly(idx)
        })
        .collect()
}

/// Whether any settlement holds any share of any religion other than
/// [`RELIGION_NONE`].
///
/// The question a surface has to ask before it renders anything, and the
/// reason it exists as a function: `false` here is *"no faction has been
/// given a religion"* — a configuration fact with a fix the user can act on
/// — and it is a completely different thing from an empty state vector,
/// which is *"nobody has run the diffusion"*. Collapsing the two into one
/// blank panel is the mistake this separates.
pub fn belief_any_faith(states: &[SettlementReligionState]) -> bool {
    states
        .iter()
        .any(|s| s.share.iter().enumerate().any(|(r, &v)| r != RELIGION_NONE && v > 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roster::CIV_RELIGIONS;
    use crate::{CIV_CULTURE_TERRAIN_KEY, CIV_CULTURES, CIV_TERRAIN_MIX_KEYS};
    use std::collections::BTreeSet;

    /// Every `(religion, culture)` pair in the two live vocabularies: 8 × 7.
    fn all_pairs() -> Vec<(&'static str, &'static str)> {
        let mut v = Vec::new();
        for (r, _) in CIV_RELIGIONS.iter() {
            for c in CIV_CULTURES.iter() {
                v.push((*r, c.key));
            }
        }
        v
    }

    /// Drift guard 1, promised by name in the module doc: a culture added to
    /// `CIV_CULTURES` without a decision here fails this test.
    #[test]
    fn culture_profiles_cover_every_culture() {
        let themed: BTreeSet<&str> = CIV_CULTURES
            .iter()
            .filter(|c| culture_domain(c.key).is_some())
            .map(|c| c.key)
            .collect();
        let unthemed: BTreeSet<&str> = CIV_CULTURES
            .iter()
            .filter(|c| culture_domain(c.key).is_none())
            .map(|c| c.key)
            .collect();

        let expect_themed: BTreeSet<&str> =
            ["highland", "desert", "riverlands", "sylvan", "maritime"]
                .into_iter()
                .collect();
        let expect_unthemed: BTreeSet<&str> = ["common", "imperial"].into_iter().collect();

        assert_eq!(
            themed, expect_themed,
            "belief::culture_domain must be Some for exactly the five terrain-themed \
             cultures; if a culture was added to CIV_CULTURES, either give it a row in \
             CIV_CULTURE_TERRAIN_KEY or record it here as deliberately unthemed"
        );
        assert_eq!(
            unthemed, expect_unthemed,
            "common and imperial deliberately get no verdict (CIV_CULTURE_TERRAIN_KEY's \
             own doc comment); this set changing means that decision changed"
        );
        assert_eq!(
            themed.len() + unthemed.len(),
            CIV_CULTURES.len(),
            "the 5/2 split must account for all 7 cultures"
        );
    }

    /// Drift guard 2, and the real one: a ninth religion added to `roster.rs`
    /// fails here until someone either gives it a domain or records it as
    /// deliberately unthemed.
    #[test]
    fn religion_profiles_cover_every_religion() {
        let themed: BTreeSet<&str> = CIV_RELIGIONS
            .iter()
            .filter(|(r, _)| religion_domain(r).is_some())
            .map(|(r, _)| *r)
            .collect();
        let unthemed: BTreeSet<&str> = CIV_RELIGIONS
            .iter()
            .filter(|(r, _)| religion_domain(r).is_none())
            .map(|(r, _)| *r)
            .collect();

        let expect_themed: BTreeSet<&str> = [
            "sun_cult",
            "earth_mother",
            "sea_lords",
            "sky_pantheon",
            "old_gods",
        ]
        .into_iter()
        .collect();
        let expect_unthemed: BTreeSet<&str> = ["none", "ancestor_rites", "flame_creed"]
            .into_iter()
            .collect();

        assert_eq!(
            themed, expect_themed,
            "DRIFT GUARD: the set of religions with a terrain domain changed. If you added \
             a religion to roster.rs::CIV_RELIGIONS, either give it a row in \
             CIV_RELIGION_DOMAIN with a stated basis, or leave it out and add it to this \
             test's unthemed set with the reason -- a silently unthemed religion routes \
             every one of its pairs to NEUTRAL_COMPAT and looks plausible while modelling \
             nothing"
        );
        assert_eq!(
            unthemed, expect_unthemed,
            "DRIFT GUARD: the set of deliberately-unthemed religions changed. none, \
             ancestor_rites and flame_creed are omitted for stated reasons (see \
             CIV_RELIGION_DOMAIN's doc comment); a new member of this set needs its own"
        );
        assert_eq!(
            themed.len() + unthemed.len(),
            CIV_RELIGIONS.len(),
            "the 5/3 split must account for all 8 religions"
        );
    }

    /// The load-bearing typo guard. A `"forrest"` in the table compiles,
    /// passes every other test, and silently makes `old_gods` match nothing
    /// while routing to a plausible-looking `0.5` -- CLAUDE.md's "watch for
    /// silently-empty golden output" exactly. Not optional garnish.
    #[test]
    fn religion_domains_are_shared_vocabulary() {
        assert_eq!(CIV_RELIGION_DOMAIN.len(), 5, "five themed religions today");

        let religion_keys: BTreeSet<&str> = CIV_RELIGIONS.iter().map(|(r, _)| *r).collect();
        let terrain_keys: BTreeSet<&str> = CIV_TERRAIN_MIX_KEYS.iter().copied().collect();
        let culture_terrains: BTreeSet<&str> =
            CIV_CULTURE_TERRAIN_KEY.iter().map(|&(_, t)| t).collect();

        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for row in CIV_RELIGION_DOMAIN.iter() {
            assert!(
                religion_keys.contains(row.religion),
                "CIV_RELIGION_DOMAIN row {:?} is not a key in roster.rs::CIV_RELIGIONS -- a \
                 typo here matches nothing and silently returns NEUTRAL_COMPAT",
                row.religion
            );
            assert!(
                terrain_keys.contains(row.terrain),
                "CIV_RELIGION_DOMAIN row {:?} has terrain {:?}, which is not in \
                 CIV_TERRAIN_MIX_KEYS -- the shared vocabulary is what makes compat() a \
                 comparison instead of a 56-cell matrix",
                row.religion,
                row.terrain
            );
            assert!(
                culture_terrains.contains(row.terrain),
                "CIV_RELIGION_DOMAIN row {:?} has terrain {:?}, which no culture claims in \
                 CIV_CULTURE_TERRAIN_KEY -- that religion could never share a domain with \
                 any culture and would score 0.0 everywhere it was evaluated",
                row.religion,
                row.terrain
            );
            assert!(
                seen.insert(row.religion),
                "duplicate religion key {:?} in CIV_RELIGION_DOMAIN -- religion_domain() \
                 returns the first row and the second would be silently dead",
                row.religion
            );
        }
        assert_eq!(
            seen.len(),
            CIV_RELIGION_DOMAIN.len(),
            "every row must be reachable"
        );
    }

    /// Makes "a read, not a second opinion" a checked fact rather than a
    /// comment.
    #[test]
    fn culture_domain_agrees_with_the_terrain_key() {
        const WHY: &str = "`belief::culture_domain` reads `CIV_CULTURE_TERRAIN_KEY` in \
                           lib.rs -- if you just edited that table, this test is telling \
                           you the belief module followed you";

        assert!(!CIV_CULTURE_TERRAIN_KEY.is_empty(), "{}", WHY);
        for &(culture, terrain) in CIV_CULTURE_TERRAIN_KEY.iter() {
            assert_eq!(culture_domain(culture), Some(terrain), "{}", WHY);
        }
        let some_count = CIV_CULTURES
            .iter()
            .filter(|c| culture_domain(c.key).is_some())
            .count();
        assert_eq!(some_count, CIV_CULTURE_TERRAIN_KEY.len(), "{}", WHY);
    }

    /// Criterion 1 made mechanical: a row cannot be added without writing
    /// down why.
    #[test]
    fn religion_domain_basis_is_stated() {
        for row in CIV_RELIGION_DOMAIN.iter() {
            assert!(
                !row.basis.trim().is_empty(),
                "CIV_RELIGION_DOMAIN row {:?} has an empty basis -- every value in this \
                 module states its evidence or it does not ship",
                row.religion
            );
            assert!(
                row.basis.len() >= 40,
                "CIV_RELIGION_DOMAIN row {:?} has a {}-byte basis; a basis must be a \
                 sentence naming the evidence, not a label",
                row.religion,
                row.basis.len()
            );
        }
    }

    #[test]
    fn compat_is_one_only_on_a_shared_domain() {
        let pairs = all_pairs();
        assert_eq!(
            pairs.len(),
            56,
            "8 religions x 7 cultures; this sweep must not be empty"
        );

        let (mut ones, mut zeros, mut neutrals) = (0usize, 0usize, 0usize);
        for &(r, c) in pairs.iter() {
            let v = compat(r, c);
            assert!(
                v.value.is_finite(),
                "compat({r:?}, {c:?}).value is not finite"
            );
            assert!(
                (0.0..=1.0).contains(&v.value),
                "compat({r:?}, {c:?}).value = {} is outside the declared [0, 1]",
                v.value
            );
            if v.value == 1.0 {
                ones += 1;
            } else if v.value == 0.0 {
                zeros += 1;
            } else if v.value == NEUTRAL_COMPAT {
                neutrals += 1;
            } else {
                panic!(
                    "compat({r:?}, {c:?}).value = {} is none of 1.0 / 0.0 / the stated neutral",
                    v.value
                );
            }
        }
        assert_eq!(
            ones, 5,
            "exactly the five bijective (religion, culture) domain matches"
        );
        assert_eq!(
            zeros, 20,
            "5 themed religions x 5 themed cultures, minus the 5 matches"
        );
        assert_eq!(
            neutrals, 31,
            "10 (themed religion x unthemed culture) + 21 (unthemed religion x any culture) \
             -- this is the modal value the module doc discloses"
        );
        assert_eq!(
            ones + zeros + neutrals,
            56,
            "the three counts must partition the sweep"
        );

        let spot: [(&str, &str, f64, CompatBasis); 5] = [
            ("sea_lords", "maritime", 1.0, CompatBasis::SharedDomain),
            ("sea_lords", "highland", 0.0, CompatBasis::DifferentDomain),
            (
                "sea_lords",
                "imperial",
                NEUTRAL_COMPAT,
                CompatBasis::CultureUnthemed,
            ),
            (
                "ancestor_rites",
                "maritime",
                NEUTRAL_COMPAT,
                CompatBasis::ReligionUnthemed,
            ),
            ("none", "common", NEUTRAL_COMPAT, CompatBasis::NeitherThemed),
        ];
        for (r, c, want_value, want_basis) in spot {
            let got = compat(r, c);
            assert_eq!(got.value, want_value, "compat({r:?}, {c:?}).value");
            assert_eq!(got.basis, want_basis, "compat({r:?}, {c:?}).basis");
        }
    }

    /// The test milestone 3 will fail first if it edits one array and not the
    /// other.
    ///
    /// Note the exact claim: a weight exists where a component *can* be
    /// `Some`, not where it *is*. `k` is legitimately `None` for the 31
    /// unthemed pairs, so the per-pair direction is "a present component
    /// always has a weight" (nothing is silently dropped by the loop) and the
    /// over-the-sweep direction is "a present weight is used by at least one
    /// pair" (no weight sits inert).
    #[test]
    fn weights_exist_exactly_where_components_can() {
        assert_eq!(
            COMPAT_WEIGHTS.len(),
            COMPAT_COMPONENTS,
            "one weight slot per §9 component"
        );
        let present: Vec<f64> = COMPAT_WEIGHTS.iter().flatten().copied().collect();
        assert!(
            !present.is_empty(),
            "at least one component must be evaluable"
        );
        assert_eq!(
            present.iter().sum::<f64>(),
            1.0,
            "the present weights must sum to exactly 1.0, so `value` is a mean over the \
             components that exist rather than a partial sum"
        );

        let mut weight_used = [false; COMPAT_COMPONENTS];
        let pairs = all_pairs();
        assert_eq!(pairs.len(), 56, "this sweep must not be empty");
        for &(r, c) in pairs.iter() {
            let v = compat(r, c);
            let components = [v.l, v.k, v.m, v.s, v.q];
            for i in 0..COMPAT_COMPONENTS {
                if components[i].is_some() {
                    assert!(
                        COMPAT_WEIGHTS[i].is_some(),
                        "compat({r:?}, {c:?}) produced component {i} with no weight in \
                         COMPAT_WEIGHTS -- the loop would silently drop it from `value`"
                    );
                    weight_used[i] = true;
                }
            }
            assert_eq!(v.l, None, "L is structurally absent in this port");
            assert_eq!(v.m, None, "M is structurally absent in this port");
            assert_eq!(v.s, None, "S is structurally absent in this port");
            assert_eq!(v.q, None, "Q is structurally absent in this port");
            assert_eq!(
                v.evaluated as usize,
                components.iter().filter(|x| x.is_some()).count(),
                "compat({r:?}, {c:?}).evaluated must equal the number of present components"
            );
            assert_eq!(
                v.evaluated,
                v.k.is_some() as u8,
                "K is the only evaluable component today"
            );
        }
        for i in 0..COMPAT_COMPONENTS {
            assert_eq!(
                COMPAT_WEIGHTS[i].is_some(),
                weight_used[i],
                "COMPAT_WEIGHTS[{i}] and compat()'s `components` array disagree: either a \
                 weight exists for a component no pair can produce, or a producible \
                 component has no weight. Milestone 3 must edit both together"
            );
        }
    }

    #[test]
    fn compat_returns_the_stated_neutral_and_never_nan() {
        for (r, c) in [
            ("bogus", "bogus"),
            ("bogus", "highland"),
            ("sun_cult", "bogus"),
            ("", ""),
        ] {
            let v = compat(r, c);
            assert_eq!(
                v.value, NEUTRAL_COMPAT,
                "compat({r:?}, {c:?}) must route to the neutral"
            );
            assert_eq!(
                v.evaluated, 0,
                "compat({r:?}, {c:?}) evaluates no component"
            );
            assert!(v.value.is_finite());
        }

        let pairs = all_pairs();
        assert_eq!(pairs.len(), 56, "this sweep must not be empty");
        for &(r, c) in pairs.iter() {
            let v = compat_value(r, c);
            assert!(
                v.is_finite(),
                "compat_value({r:?}, {c:?}) = {v} -- the `evaluated == 0` guard on the \
                 `num / den` division is what stops a NaN reaching §19's logistic"
            );
        }
    }

    /// The paper's §32 encoded: *"the model should never assume
    /// ReligionQuality → Conversion ... the key conceptual safeguard against
    /// creating an artificial 'best religion'."* Without this, a model that
    /// had silently collapsed to a universal ranking would pass every other
    /// test in this file.
    #[test]
    fn compat_ranks_religions_differently_across_cultures() {
        // §9's own worked claim -- Compat(R, C1) high, Compat(R, C2) low --
        // as an actual rank reversal in this model.
        assert!(
            compat_value("sun_cult", "desert") > compat_value("sea_lords", "desert"),
            "the sun cult must beat the sea lords among the desert culture"
        );
        assert!(
            compat_value("sun_cult", "maritime") < compat_value("sea_lords", "maritime"),
            "and lose to them among the maritime culture -- that reversal is the point"
        );

        for (a, _) in CIV_RELIGIONS.iter() {
            for (b, _) in CIV_RELIGIONS.iter() {
                if a == b {
                    continue;
                }
                let mut a_never_worse = true;
                let mut a_sometimes_better = false;
                for c in CIV_CULTURES.iter() {
                    let (va, vb) = (compat_value(a, c.key), compat_value(b, c.key));
                    if va < vb {
                        a_never_worse = false;
                    }
                    if va > vb {
                        a_sometimes_better = true;
                    }
                }
                assert!(
                    !(a_never_worse && a_sometimes_better),
                    "religion {a:?} weakly dominates {b:?} across all {} cultures -- that is \
                     the artificial \"best religion\" §32 exists to forbid",
                    CIV_CULTURES.len()
                );
            }
        }
    }

    /// Both of this module's authored value sets, pinned to **literals**.
    ///
    /// Added after an adversarial mutation pass found two survivors, and
    /// they survived for the same reason: every other test in this file
    /// refers to these values *symbolically*. `NEUTRAL_COMPAT` was changed
    /// `0.5 -> 0.75` and all nine tests still passed, because assertions
    /// written as `v.value == NEUTRAL_COMPAT` move with the constant they
    /// are meant to pin -- and that constant is the **modal** output of the
    /// whole module (31 of 56 pairs). Three of the five
    /// [`CIV_RELIGION_DOMAIN`] rows -- `earth_mother`, `sky_pantheon`,
    /// `old_gods`, which are exactly the three the module doc flags as
    /// judgement calls -- were likewise pinned by nothing: only `sun_cult`
    /// and `sea_lords` appear in the spot-check array above.
    ///
    /// `CIV_CULTURE_TERRAIN_KEY`'s bijectivity is why the counts in
    /// `compat_is_one_only_on_a_shared_domain` cannot catch this: `ones == 5`
    /// holds for *any* assignment of five themed religions to the five
    /// terrain keys, including one that duplicates a key and orphans
    /// another. So injectivity is asserted here explicitly rather than
    /// inferred from a count that is arithmetically forced.
    ///
    /// This repository's own working rule: *"Golden-matching is necessary
    /// and not sufficient. Mutation-test the constants."*
    #[test]
    fn the_authored_values_are_pinned_to_literals() {
        // The stated neutral. A bare literal on the right-hand side, on
        // purpose -- writing `NEUTRAL_COMPAT` here would restore the exact
        // hole this test exists to close.
        assert_eq!(
            NEUTRAL_COMPAT, 0.5,
            "the stated neutral is the midpoint of Compat's declared [0, 1] range; \
             changing it changes the modal output of this module"
        );

        // Every row of the domain table, in declared order.
        let want: [(&str, &str); 5] = [
            ("sun_cult", "arid"),
            ("earth_mother", "river"),
            ("sea_lords", "coast"),
            ("sky_pantheon", "hills"),
            ("old_gods", "forest"),
        ];
        assert_eq!(
            CIV_RELIGION_DOMAIN.len(),
            want.len(),
            "a row was added or removed without updating this pin"
        );
        for (i, (religion, terrain)) in want.iter().enumerate() {
            assert_eq!(
                CIV_RELIGION_DOMAIN[i].religion, *religion,
                "CIV_RELIGION_DOMAIN[{i}].religion"
            );
            assert_eq!(
                CIV_RELIGION_DOMAIN[i].terrain, *terrain,
                "CIV_RELIGION_DOMAIN[{i}].terrain -- if this row was deliberately \
                 revisited (the doc names earth_mother first and sky_pantheon \
                 second), update the pin and say so in the basis string"
            );
        }

        // Injective onto the terrain vocabulary. NOT implied by `ones == 5`.
        let mut seen: Vec<&str> = CIV_RELIGION_DOMAIN.iter().map(|r| r.terrain).collect();
        seen.sort_unstable();
        let before = seen.len();
        seen.dedup();
        assert_eq!(
            seen.len(),
            before,
            "two religions claim the same terrain key, which orphans another -- \
             the bijection the module doc rests on is broken"
        );

        // And the basis strings for the two rows the doc calls judgement
        // calls still say so, so a later reader cannot mistake them for
        // forced readings the way `sea_lords` genuinely is.
        let by = |k: &str| -> &'static str {
            CIV_RELIGION_DOMAIN.iter().find(|r| r.religion == k).unwrap().basis
        };
        assert!(
            by("earth_mother").contains("SOFTEST"),
            "earth_mother's basis no longer discloses that it is the softest call"
        );
        assert!(
            by("sky_pantheon").contains("SECOND-SOFTEST"),
            "sky_pantheon's basis no longer discloses that it is the second-softest call"
        );
    }
}

#[cfg(test)]
mod diffusion_tests {
    use super::*;
    use crate::roster::CIV_RELIGIONS;

    /// A line of `n` settlements, each linked to the next at `km`, all of
    /// population `pop`. Returns the network plus a culture slice.
    fn chain(n: usize, pop: u32, km: f64) -> BeliefNetwork {
        let pops = vec![pop; n];
        let links: Vec<BeliefLink> =
            (0..n.saturating_sub(1)).map(|i| BeliefLink { a: i, b: i + 1, km }).collect();
        BeliefNetwork::build(&pops, &links)
    }

    fn sum(s: &SettlementReligionState) -> f64 {
        s.share.iter().sum()
    }

    /// The vocabulary bound the whole module indexes by. If `CIV_RELIGIONS`
    /// ever changes length, every fixed-size array here changes with it and
    /// this is the test that says so first.
    #[test]
    fn the_religion_index_is_the_roster_order() {
        assert_eq!(CIV_RELIGION_COUNT, 8, "eight religions, `none` included");
        assert_eq!(CIV_RELIGION_COUNT, CIV_RELIGIONS.len());
        assert_eq!(RELIGION_NONE, 0);
        assert_eq!(
            religion_key(RELIGION_NONE),
            Some("none"),
            "RELIGION_NONE must be the secular slot, not merely index zero"
        );
        for (i, (k, _)) in CIV_RELIGIONS.iter().enumerate() {
            assert_eq!(religion_index(k), Some(i));
            assert_eq!(religion_key(i), Some(*k));
        }
        assert_eq!(religion_index("cargo_cult"), None);
        assert_eq!(religion_index(""), None);
        assert_eq!(religion_key(CIV_RELIGION_COUNT), None);
    }

    /// The authored coefficients, pinned to **literals** — the hole the
    /// existing `the_authored_values_are_pinned_to_literals` was written to
    /// close, applied to this half of the module. Every other test below
    /// refers to these symbolically and would move with them.
    #[test]
    fn the_diffusion_coefficients_are_pinned_to_literals() {
        assert_eq!(BELIEF_BE, 3.0, "exposure is the largest of the three");
        assert_eq!(BELIEF_BC, 2.0, "compatibility second, per §9");
        assert_eq!(BELIEF_BF, 1.0, "conformity smallest, because it feeds itself");
        assert_eq!(BELIEF_B0, -3.0, "the derived intercept, -(3+2+1)/2");
        assert_eq!(BELIEF_CONFORMITY_K, 2.0, "§14's k, the smallest integer above 1");
        assert_eq!(BELIEF_STEP_RATE, 0.05, "one Timeline year's convertible share");
    }

    /// [`BELIEF_B0`]'s doc claims a *relationship*, not a value, and this is
    /// what makes the claim checkable: raise `βE` alone and the model stops
    /// being centred, so this fails rather than silently shifting every
    /// settlement's neutral toward conversion.
    // The two ordering assertions at the end are constant-valued on purpose:
    // they encode §19's and §14's own constraints on the *relationships*
    // between these constants, which is the thing a later calibration pass
    // would break. That is not the "assert a constant against itself" shape
    // MISTAKES.md forbids -- each compares two different authored values.
    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn belief_is_centred_on_its_own_input_domain() {
        let span = BELIEF_BE + BELIEF_BC + BELIEF_BF;
        assert_eq!(BELIEF_B0, -span / 2.0, "β0 is derived from the three weights");
        let midpoint = BELIEF_B0 + 0.5 * span;
        assert_eq!(midpoint, 0.0, "the domain midpoint must land on the logistic's own centre");
        assert_eq!(belief_logistic(midpoint), 0.5);
        // And the span really is the responsive part of the curve.
        let lo = belief_logistic(BELIEF_B0);
        let hi = belief_logistic(BELIEF_B0 + span);
        assert!(lo > 0.04 && lo < 0.06, "bottom of the domain is σ ≈ 0.047, got {lo}");
        assert!(hi > 0.94 && hi < 0.96, "top of the domain is σ ≈ 0.953, got {hi}");
        assert!(BELIEF_BE > BELIEF_BC && BELIEF_BC > BELIEF_BF, "the 3:2:1 ordering §19/§9/§14 argue for");
        assert!(BELIEF_CONFORMITY_K > 1.0, "§14 requires k > 1 or the term is not conformist");
    }

    /// `js_exp`, not `f64::exp`. Asserted against literal reference points
    /// rather than against `1.0/(1.0+(-x).exp())`, which would restate the
    /// implementation.
    #[test]
    fn the_logistic_is_the_papers_sigma() {
        assert_eq!(belief_logistic(0.0), 0.5);
        assert!((belief_logistic(1.0) - 0.7310585786300049).abs() < 1e-15);
        assert!((belief_logistic(-1.0) - 0.2689414213699951).abs() < 1e-15);
        // The claim `belief_step`'s leading `E_r` factor rests on: over this
        // model's whole reachable domain the logistic never returns zero, so
        // "no exposure" cannot be expressed by the logistic alone.
        let span = BELIEF_BE + BELIEF_BC + BELIEF_BF;
        for x in [BELIEF_B0, BELIEF_B0 + span / 2.0, BELIEF_B0 + span] {
            assert!(belief_logistic(x) > 0.0, "σ({x}) reached zero inside the model's domain");
            assert!(belief_logistic(x) < 1.0, "σ({x}) reached one inside the model's domain");
        }
        // Far outside it, σ does saturate to exact 0.0/1.0 rather than
        // returning a denormal or a NaN. Nothing in this model can reach
        // these arguments; they are pinned because a caller passing an
        // unclamped score would.
        assert_eq!(belief_logistic(-800.0), 0.0, "js_exp overflows to +inf, and 1/inf is 0");
        assert_eq!(belief_logistic(800.0), 1.0);
        for x in [-800.0f64, -40.0, -3.0, 0.0, 3.0, 40.0, 800.0, f64::MIN, f64::MAX] {
            let v = belief_logistic(x);
            assert!(v.is_finite() && (0.0..=1.0).contains(&v), "σ({x}) = {v}");
        }
    }

    /// Largest remainder: exactly `pop`, for every split and every size.
    #[test]
    fn adherents_conserve_the_settlement_population_exactly() {
        let mut s = SettlementReligionState { share: [0.0; CIV_RELIGION_COUNT] };
        // Eight equal eighths of 120 is exactly 15 each; of 100 it is 12.5,
        // the case independent rounding gets wrong.
        for v in s.share.iter_mut() {
            *v = 1.0 / CIV_RELIGION_COUNT as f64;
        }
        for pop in [0u32, 1, 3, 100, 120, 401, 45_000] {
            let a = s.adherents(pop);
            assert_eq!(a.iter().map(|&x| x as u64).sum::<u64>(), pop as u64, "even split of {pop}");
        }
        assert_eq!(
            s.adherents(100).iter().copied().max().unwrap()
                - s.adherents(100).iter().copied().min().unwrap(),
            1,
            "a 12.5-person eighth splits 13/12, never 13/11"
        );

        // A lopsided split, and a minority far below one whole person.
        let mut t = SettlementReligionState::wholly(1);
        t.share[1] = 0.996;
        t.share[3] = 0.004;
        assert_eq!(t.adherents(120).iter().map(|&x| x as u64).sum::<u64>(), 120);
        assert_eq!(
            t.adherents(120)[3],
            0,
            "0.48 of a person is nobody -- but the *share* survives, which is the point \
             of storing shares"
        );
        assert!(t.share[3] > 0.0, "the sub-person minority is still in the model");

        // Ties go to the lower index, deterministically.
        let mut u = SettlementReligionState { share: [0.0; CIV_RELIGION_COUNT] };
        u.share[2] = 0.5;
        u.share[5] = 0.5;
        assert_eq!(u.adherents(3), {
            let mut want = [0u32; CIV_RELIGION_COUNT];
            want[2] = 2;
            want[5] = 1;
            want
        }, "the odd person goes to the lower index");
    }

    #[test]
    fn plurality_reports_the_secular_share_when_it_leads() {
        let mut s = SettlementReligionState { share: [0.0; CIV_RELIGION_COUNT] };
        s.share[RELIGION_NONE] = 0.6;
        s.share[4] = 0.4;
        assert_eq!(
            s.plurality(),
            RELIGION_NONE,
            "a majority-secular town must not be labelled with a religion 4 in 10 follow"
        );
        s.share[RELIGION_NONE] = 0.3;
        s.share[4] = 0.7;
        assert_eq!(s.plurality(), 4);
        // Tie -> lower index, so the answer is stable.
        let mut t = SettlementReligionState { share: [0.0; CIV_RELIGION_COUNT] };
        t.share[3] = 0.5;
        t.share[6] = 0.5;
        assert_eq!(t.plurality(), 3);
    }

    /// The seeding finding, made mechanical: the shipped roster default
    /// yields a world with no religion in it, and this test is what stops a
    /// later pass "fixing" that by inventing one.
    #[test]
    fn the_shipped_roster_default_seeds_a_world_with_no_religion() {
        let roster: Vec<&str> = vec!["none"; 7];
        let factions = [0i32, 1, 2, 3, 4, 5, 6];
        let states = belief_seed(&factions, &roster);
        assert_eq!(states.len(), factions.len());
        for s in states.iter() {
            assert_eq!(s.share[RELIGION_NONE], 1.0);
            assert_eq!(s.plurality(), RELIGION_NONE);
        }
        assert!(
            !belief_any_faith(&states),
            "civ_roster_bridge::FactionEntry::default_for sets religion \"none\" for every \
             faction, generation included. If this now passes, either the roster default \
             changed (say where, and why) or somebody fabricated a per-faction religion -- \
             which CIV_RELIGION_DOMAIN's own doc refuses"
        );

        // And a hand-set religion is picked up, so the rule itself works.
        let edited: Vec<&str> = vec!["none", "sea_lords", "none", "none", "none", "none", "none"];
        let states = belief_seed(&factions, &edited);
        assert_eq!(states[1].plurality(), religion_index("sea_lords").unwrap());
        assert!(belief_any_faith(&states));
        assert_eq!(states[2].plurality(), RELIGION_NONE);
    }

    #[test]
    fn seeding_routes_every_unresolvable_faction_to_unaffiliated() {
        let roster: Vec<&str> = vec!["none", "cargo_cult", "old_gods"];
        // -1 is not a faction id; 9 is past the roster; 1 names a religion
        // this port's vocabulary does not have.
        let states = belief_seed(&[-1, 9, 1, 2], &roster);
        assert_eq!(states[0].plurality(), RELIGION_NONE, "a negative faction id");
        assert_eq!(states[1].plurality(), RELIGION_NONE, "a faction past the roster");
        assert_eq!(states[2].plurality(), RELIGION_NONE, "an unrecognised religion key");
        assert_eq!(states[3].plurality(), religion_index("old_gods").unwrap());
        for s in states.iter() {
            assert!((sum(s) - 1.0).abs() < 1e-12);
        }
    }

    /// The graph really is the road network, and hidden ways really are in
    /// it — the sentence in [`belief_links_from_ways`]' doc, checked.
    #[test]
    fn links_come_from_every_way_including_the_hidden_ones() {
        let way = |a: usize, b: usize, km: f64, hidden: bool| crate::Way {
            tid: 0,
            pts: vec![(0.0, 0.0), (1.0, 1.0)],
            brks: Vec::new(),
            km,
            name: String::new(),
            way_type: crate::WayType::Road,
            a_idx: a,
            b_idx: b,
            hidden,
        };
        let ways = vec![way(0, 1, 40.0, false), way(1, 2, 60.0, true), way(3, 3, 5.0, false)];
        let links = belief_links_from_ways(&ways);
        assert_eq!(links.len(), 2, "a self-loop is dropped; a hidden way is not");
        assert_eq!(links[0], BeliefLink { a: 0, b: 1, km: 40.0 });
        assert_eq!(links[1], BeliefLink { a: 1, b: 2, km: 60.0 });
    }

    /// The weight is `crate::trade::deliverable`, not a private curve —
    /// asserted through observable behaviour at the reference's own 220 km
    /// land cutoff.
    #[test]
    fn a_link_past_the_references_land_reach_carries_nothing() {
        assert_eq!(crate::trade::MAX_REACH_KM[0], 220.0, "the reference's FOOD_MAX_REACH_KM land value");
        let near = BeliefNetwork::build(&[1000, 1000], &[BeliefLink { a: 0, b: 1, km: 219.0 }]);
        assert_eq!(near.degree(0), 2, "inside the reach, the neighbour is there");
        let far = BeliefNetwork::build(&[1000, 1000], &[BeliefLink { a: 0, b: 1, km: 221.0 }]);
        assert_eq!(far.degree(0), 1, "past 220 km the link contributes nothing at all");
        assert_eq!(far.degree(1), 1);

        // Self is always present, at distance zero.
        let alone = BeliefNetwork::build(&[500], &[]);
        assert_eq!(alone.degree(0), 1, "every settlement is its own neighbour (§5.3)");

        // Malformed links are dropped, not panicked on.
        let bad = BeliefNetwork::build(&[10, 10], &[
            BeliefLink { a: 0, b: 7, km: 10.0 },
            BeliefLink { a: 1, b: 1, km: 10.0 },
            BeliefLink { a: 0, b: 1, km: f64::NAN },
            BeliefLink { a: 0, b: 1, km: -5.0 },
        ]);
        assert_eq!(bad.degree(0), 1);
        assert_eq!(bad.degree(1), 1);
    }

    /// The invariant everything else rests on: population is conserved, at
    /// every settlement, on every step, forever.
    #[test]
    fn every_step_conserves_the_population_of_every_settlement() {
        let net = chain(6, 4000, 30.0);
        let cultures = vec!["maritime", "highland", "common", "desert", "sylvan", "imperial"];
        let mut states: Vec<SettlementReligionState> = (0..6)
            .map(|i| SettlementReligionState::wholly(if i == 0 { 3 } else { RELIGION_NONE }))
            .collect();
        for step in 0..500 {
            belief_step(&mut states, &net, &cultures, BELIEF_STEP_RATE);
            for (i, s) in states.iter().enumerate() {
                assert!(
                    (sum(s) - 1.0).abs() < 1e-9,
                    "settlement {i} sums to {} after step {step}",
                    sum(s)
                );
                for (r, &v) in s.share.iter().enumerate() {
                    assert!(v.is_finite(), "settlement {i} religion {r} is {v} after step {step}");
                    assert!((-1e-12..=1.0 + 1e-12).contains(&v), "share {v} out of range");
                }
            }
        }
    }

    /// §17's shape, and the reason [`belief_step`]'s leading `E_r` factor is
    /// there: no exposure, no conversion. Without it the strictly-positive
    /// logistic grows a congregation for all eight religions in a sealed
    /// valley on the first step.
    #[test]
    fn an_unexposed_religion_never_gains_a_single_adherent() {
        let net = BeliefNetwork::build(&[8000], &[]);
        let cultures = vec!["maritime"];
        let mut states = vec![SettlementReligionState::wholly(religion_index("sea_lords").unwrap())];
        for _ in 0..1000 {
            belief_step(&mut states, &net, &cultures, BELIEF_STEP_RATE);
        }
        let sea = religion_index("sea_lords").unwrap();
        assert_eq!(states[0].share[sea], 1.0, "an isolated settlement keeps its own faith exactly");
        for (r, &v) in states[0].share.iter().enumerate() {
            if r != sea {
                assert_eq!(v, 0.0, "religion {:?} appeared out of nothing", religion_key(r));
            }
        }
    }

    /// The seed is a fixed point when nobody has a religion — the other half
    /// of the finding in [`belief_seed`]'s doc, and the thing a surface must
    /// report as a configuration state rather than as an empty panel.
    #[test]
    fn an_all_secular_world_is_a_fixed_point() {
        let net = chain(5, 2000, 25.0);
        let cultures = vec!["riverlands"; 5];
        let mut states = belief_seed(&[1, 1, 2, 2, 3], &["none", "none", "none", "none"]);
        for _ in 0..200 {
            belief_step(&mut states, &net, &cultures, BELIEF_STEP_RATE);
        }
        assert!(!belief_any_faith(&states), "nothing can grow where nothing was seeded");
        for s in states.iter() {
            assert_eq!(s.share[RELIGION_NONE], 1.0);
        }
    }

    /// The mechanic the milestone exists to prove: a faith seeded in one
    /// settlement reaches its road neighbours and not the far end of the
    /// chain first.
    #[test]
    fn a_faith_spreads_along_the_road_network_in_order() {
        let net = chain(5, 5000, 40.0);
        let cultures = vec!["desert"; 5];
        let sun = religion_index("sun_cult").unwrap();
        let mut states: Vec<SettlementReligionState> = (0..5)
            .map(|i| SettlementReligionState::wholly(if i == 0 { sun } else { RELIGION_NONE }))
            .collect();
        for _ in 0..40 {
            belief_step(&mut states, &net, &cultures, BELIEF_STEP_RATE);
        }
        for i in 0..4 {
            assert!(
                states[i].share[sun] > states[i + 1].share[sun],
                "settlement {i} ({}) must hold more of the faith than {} ({})",
                states[i].share[sun],
                i + 1,
                states[i + 1].share[sun]
            );
        }
        assert!(states[1].share[sun] > 0.0, "the neighbour has heard of it");
        assert!(
            states[4].share[sun] > 0.0,
            "and after 40 years it has reached the far end -- through the chain, not by radius"
        );
    }

    /// §9/§24's whole point, as behaviour rather than as a table: the same
    /// religion, the same network, the same seed — a different receiving
    /// culture, a different outcome.
    #[test]
    fn the_receiving_culture_changes_the_outcome() {
        let sea = religion_index("sea_lords").unwrap();
        let run = |culture: &str| -> f64 {
            let net = chain(2, 6000, 30.0);
            let cultures = vec![culture, culture];
            let mut states =
                vec![SettlementReligionState::wholly(sea), SettlementReligionState::wholly(RELIGION_NONE)];
            for _ in 0..60 {
                belief_step(&mut states, &net, &cultures, BELIEF_STEP_RATE);
            }
            states[1].share[sea]
        };
        let matched = run("maritime");
        let clashing = run("highland");
        let unthemed = run("imperial");
        assert!(
            matched > unthemed && unthemed > clashing,
            "Sea Lords must take a maritime town fastest ({matched}), an unthemed one next \
             ({unthemed}) and a highland one slowest ({clashing}) -- the compat ordering \
             §9 asks for, and the 'absence of evidence outranks evidence of mismatch' rule \
             compat()'s own doc states"
        );
        assert!(clashing > 0.0, "a theme clash is `no bonus`, never a hostility claim");
    }

    /// [`BELIEF_STEP_RATE`]'s real pin, and the model's actual speed range,
    /// **measured rather than estimated**.
    ///
    /// Every figure below is a literal, which is the whole point: a band
    /// assertion like `assert!((20..=400).contains(&years))` holds for a rate
    /// twice this one and is the "assert a constant against itself" shape in
    /// a wider costume. Change `BELIEF_STEP_RATE`, or any `β`, and every
    /// number here moves.
    ///
    /// The headline, stated plainly because it is not what the milestone's
    /// own prose would lead a reader to expect: **where this model converts
    /// at all it converts within a lifetime, and where it does not converge
    /// it never converts.** There is no slow-but-eventual middle. That is a
    /// property of the exposure term dominating — see the third case.
    #[test]
    fn belief_takeover_is_a_generation_where_it_happens_at_all() {
        let sun = religion_index("sun_cult").unwrap();
        let minority = |frac: f64| SettlementReligionState {
            share: {
                let mut s = [0.0; CIV_RELIGION_COUNT];
                s[sun] = frac;
                s[RELIGION_NONE] = 1.0 - frac;
                s
            },
        };
        const NEVER: usize = 5000;
        // Years until the receiving settlement (index 1) holds a plurality,
        // and the share it holds at the end. `NEVER` means it did not.
        let run = |pops: [u32; 2], km: f64, culture: &'static str| -> (usize, f64) {
            let net = BeliefNetwork::build(&pops, &[BeliefLink { a: 0, b: 1, km }]);
            let cultures = vec![culture, culture];
            let mut states = vec![SettlementReligionState::wholly(sun), minority(0.01)];
            let mut years = 0usize;
            while states[1].plurality() != sun && years < NEVER {
                belief_step(&mut states, &net, &cultures, BELIEF_STEP_RATE);
                years += 1;
            }
            (years, states[1].share[sun])
        };
        let years_to_plurality = |pops: [u32; 2], km: f64, culture: &'static str| run(pops, km, culture).0;

        // Best case this model has: equal neighbours 20 km apart, the
        // religion's own culture on both sides. Half of the receiving town's
        // whole social contact is with a wholly-converted neighbour.
        assert_eq!(
            years_to_plurality([5000, 5000], 20.0, "desert"),
            25,
            "the most favourable configuration takes one generation. Under it lies §25's \
             forbidden 'king converts -> everyone converts immediately'"
        );

        // §3's own claim, measured: *"a distant major port can have
        // substantially more contact with another settlement than a nearby
        // isolated village."* A village beside a city four times its size
        // 120 km away converts FASTER than the equal pair 20 km apart above,
        // even though its culture is unthemed and its Compat is only the
        // stated neutral -- because `w_ij = pop_j · deliverable(km)` puts
        // 70 % of that village's social contact outside itself against the
        // equal pair's 48 %. The size ratio beats six times the distance.
        // This is the behaviour that makes the model a network model rather
        // than a radius, and it is asserted here because it is the one
        // outcome a reader would most likely assume is a bug.
        assert_eq!(
            years_to_plurality([4000, 1000], 120.0, "imperial"),
            17,
            "a small village in a big city's shadow converts faster than an equal \
             neighbour next door"
        );

        // The other end is not "slow", it is "never" -- the same three
        // factors inverted. A city of 4 000 drawing on a village of 1 000
        // 210 km away (just inside the reference's own 220 km land reach)
        // whose religion clashes with the city's own culture, so `Compat` is
        // a hard `0.0` against secularism's stated `0.5`. Exposure is ~9 %.
        //
        // **This is §32's safeguard doing its job**, and it is the single
        // most important assertion in this test: a model that always
        // converted, given enough years, would have an artificial "best
        // religion" no matter what the compatibility table said.
        let (years, held) = run([1000, 4000], 210.0, "maritime");
        assert_eq!(years, NEVER, "an unfavourable configuration must not eventually convert");
        // And it does not merely stall -- it goes to zero. §14's `p^k` with
        // `k > 1` is a positive frequency-dependent bias, which is a
        // fixation dynamic by construction: below a threshold the minority's
        // own conformity term works against it. `belief_has_no_syncretic_
        // equilibrium` is where that is stated as the model-wide property it
        // is, because it is not obvious from the milestone's prose and a
        // surface built on it needs to know.
        assert!(
            held < 1e-30,
            "the minority did not merely stall, it vanished -- it holds {held}"
        );
        assert!(held >= 0.0 && held.is_finite(), "and it decayed to zero without going negative");
    }

    /// The property a screen designer needs before drawing anything, and the
    /// one this milestone's own prose does not lead a reader to expect:
    /// **there is no stable mixture in this model.** Every settlement
    /// converges to one religion holding essentially all of it.
    ///
    /// That is §14 behaving exactly as §14 says it does — positive
    /// frequency-dependent transmission with `k > 1` is a fixation dynamic,
    /// and *"can generate rapid changes in cultural prevalence and S-shaped
    /// diffusion curves"* is what a fixation dynamic looks like. Coexistence
    /// is what the paper's §18 (`Competition`, with an explicit *"multiple
    /// religions can coexist within a settlement"*) and §22 (retention as its
    /// own function) are for, and both are later milestones with no input in
    /// this port today.
    ///
    /// So: a settlement inspector fed by this milestone will show one faith
    /// at ~100 % and the other seven at ~0 %, not a pie chart. Designing a
    /// breakdown widget against an imagined five-way split would be
    /// designing against a model that does not exist yet.
    #[test]
    fn belief_has_no_syncretic_equilibrium() {
        // The most balanced case available: a town exactly between two equal
        // neighbours of different faiths, at equal distance, in a culture
        // that favours neither (`imperial` is unthemed, so both religions
        // get the stated neutral).
        let sun = religion_index("sun_cult").unwrap();
        let sea = religion_index("sea_lords").unwrap();
        let net = BeliefNetwork::build(&[3000, 3000, 3000], &[
            BeliefLink { a: 0, b: 1, km: 40.0 },
            BeliefLink { a: 1, b: 2, km: 40.0 },
        ]);
        let cultures = vec!["imperial"; 3];
        let mut states = vec![
            SettlementReligionState::wholly(sun),
            SettlementReligionState::wholly(RELIGION_NONE),
            SettlementReligionState::wholly(sea),
        ];
        for _ in 0..2000 {
            belief_step(&mut states, &net, &cultures, BELIEF_STEP_RATE);
        }
        let middle = &states[1];
        let top = middle.share.iter().copied().fold(0.0f64, f64::max);
        assert!(
            top > 0.99,
            "even the most balanced settlement this model can be given converges to a single \
             faith at {top}; if this ever drops below 0.99, a coexistence mechanism has \
             arrived and every surface that assumed a single winner needs revisiting"
        );
        assert!(
            (middle.share.iter().sum::<f64>() - 1.0).abs() < 1e-9,
            "and it is still a distribution"
        );
    }

    /// Order independence, which is what "synchronous" buys and what an
    /// in-place sweep would silently lose.
    #[test]
    fn a_step_does_not_depend_on_settlement_order() {
        // A 4-chain 0-1-2-3, and the same chain with the list reversed.
        let cultures = vec!["sylvan"; 4];
        let old = religion_index("old_gods").unwrap();
        let fwd_net = BeliefNetwork::build(&[3000, 3000, 3000, 3000], &[
            BeliefLink { a: 0, b: 1, km: 30.0 },
            BeliefLink { a: 1, b: 2, km: 30.0 },
            BeliefLink { a: 2, b: 3, km: 30.0 },
        ]);
        let rev_net = BeliefNetwork::build(&[3000, 3000, 3000, 3000], &[
            BeliefLink { a: 3, b: 2, km: 30.0 },
            BeliefLink { a: 2, b: 1, km: 30.0 },
            BeliefLink { a: 1, b: 0, km: 30.0 },
        ]);
        let seed = |first: usize| -> Vec<SettlementReligionState> {
            (0..4)
                .map(|i| SettlementReligionState::wholly(if i == first { old } else { RELIGION_NONE }))
                .collect()
        };
        let (mut a, mut b) = (seed(0), seed(3));
        for _ in 0..25 {
            belief_step(&mut a, &fwd_net, &cultures, BELIEF_STEP_RATE);
            belief_step(&mut b, &rev_net, &cultures, BELIEF_STEP_RATE);
        }
        for i in 0..4 {
            assert!(
                (a[i].share[old] - b[3 - i].share[old]).abs() < 1e-12,
                "the mirrored chain diverged at {i}: {} vs {}",
                a[i].share[old],
                b[3 - i].share[old]
            );
        }
    }

    /// Same inputs, same answer, bit for bit — the determinism claim, run
    /// rather than asserted in prose.
    #[test]
    fn the_same_world_produces_the_same_adherence_bit_for_bit() {
        let run = || -> Vec<[u32; CIV_RELIGION_COUNT]> {
            let net = chain(7, 1234, 55.0);
            let cultures = vec!["maritime", "highland", "common", "desert", "sylvan", "imperial", "riverlands"];
            let mut states = belief_seed(
                &[1, 2, 1, 3, 2, 1, 3],
                &["none", "sea_lords", "old_gods", "flame_creed"],
            );
            for _ in 0..120 {
                belief_step(&mut states, &net, &cultures, BELIEF_STEP_RATE);
            }
            states.iter().map(|s| s.adherents(1234)).collect()
        };
        let a = run();
        let b = run();
        assert_eq!(a, b, "two runs of the same inputs disagree");
        assert!(
            a.iter().any(|counts| counts.iter().enumerate().any(|(r, &c)| r != RELIGION_NONE && c > 0)),
            "this fixture must actually diffuse something, or it pins nothing"
        );
        for counts in a.iter() {
            assert_eq!(counts.iter().map(|&x| x as u64).sum::<u64>(), 1234);
        }
    }

    /// A rate of zero changes nothing, and a rate of one is still a
    /// distribution — the two boundaries a caller can reach.
    #[test]
    fn the_step_rate_boundaries_are_safe() {
        let net = chain(3, 900, 15.0);
        let cultures = vec!["desert"; 3];
        let before = belief_seed(&[1, 2, 2], &["none", "sun_cult", "none"]);

        let mut zero = before.clone();
        belief_step(&mut zero, &net, &cultures, 0.0);
        assert_eq!(zero, before, "rate 0 must be a no-op");

        let mut one = before.clone();
        belief_step(&mut one, &net, &cultures, 1.0);
        for s in one.iter() {
            assert!((sum(s) - 1.0).abs() < 1e-12);
        }

        // Out-of-range rates clamp rather than producing negative shares.
        let mut wild = before.clone();
        belief_step(&mut wild, &net, &cultures, 4.0);
        for s in wild.iter() {
            assert!(s.share.iter().all(|v| *v >= 0.0 && v.is_finite()));
        }
        let mut neg = before.clone();
        belief_step(&mut neg, &net, &cultures, -1.0);
        assert_eq!(neg, before, "a negative rate clamps to the no-op, not to a reversal");
    }

    /// Degenerate shapes a real world hands this: an empty list, a
    /// zero-population settlement, a culture slice shorter than the
    /// settlement list.
    #[test]
    fn degenerate_worlds_do_not_panic_or_produce_nan() {
        let mut none: Vec<SettlementReligionState> = Vec::new();
        belief_step(&mut none, &BeliefNetwork::build(&[], &[]), &[], BELIEF_STEP_RATE);
        assert!(none.is_empty());
        assert!(!belief_any_faith(&none));

        // A ghost town beside a living one.
        let net = BeliefNetwork::build(&[0, 4000], &[BeliefLink { a: 0, b: 1, km: 10.0 }]);
        let mut states = belief_seed(&[1, 2], &["none", "old_gods", "sun_cult"]);
        // Deliberately one culture short, so the missing entry is exercised.
        let cultures = vec!["sylvan"];
        for _ in 0..50 {
            belief_step(&mut states, &net, &cultures, BELIEF_STEP_RATE);
        }
        for s in states.iter() {
            assert!(s.share.iter().all(|v| v.is_finite()), "NaN reached a share");
            assert!((sum(s) - 1.0).abs() < 1e-9);
        }
        // A state list longer than the network is truncated, not indexed
        // out of bounds.
        let mut extra = belief_seed(&[1, 2, 1, 1], &["none", "old_gods", "sun_cult"]);
        belief_step(&mut extra, &net, &cultures, BELIEF_STEP_RATE);
        assert_eq!(extra.len(), 4);
    }
}
