//! Culture and religion as quantitative traits, and the compatibility
//! relation between them.
//!
//! `RELIGION_DIFFUSION_SCOPE.md` §1 carries the owner-supplied paper this
//! implements; §2 records what it maps onto. This module is the foundation
//! both milestone 1 (network exposure and conversion) and milestone 3 (the
//! authored trait vectors) need, and nothing above it.
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
//! - **`p_conv` and the `β0`/`βE`/`βC`/`βF` coefficients** — milestone 1's
//!   work, not the trait model's. The shape is one line:
//!   `1.0 / (1.0 + js_exp(-(b0 + bE*e + bC*compat + bF*freq)))`, and that
//!   `js_exp` is where float discipline actually bites (see [`compat`]).
//! - **§14's conformity exponent `k`** — one *global* calibration constant
//!   belonging beside the diffusion step, **not** eight per-religion values.
//!   Saying so here is what stops milestone 1 landing it on a `Religion`
//!   struct as the eight-unauthored-numbers outcome this design exists to
//!   avoid.
//! - **`SettlementReligionState`** — milestone 1's per-settlement runtime
//!   state. This crate is stateless per `ARCHITECTURE.md`; the caller owns
//!   it.
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
//! **3. These consts are public before milestone 1 exists**, so the shell
//! can render a [`Compat`] with no diffusion behind it, implying a
//! simulation that is not there. That is the same trap
//! [`crate::civ_culture_terrain_fit`] sits in today (ported, correct, wired
//! to no caller), and it is accepted on that precedent — but only because
//! this doc says so as plainly as that function's own doc does.
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
