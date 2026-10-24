//! FH-R2 milestone R2-6, the research document's **S6: a per-faction
//! religious policy** (`RELIGION_GOVERNMENT_EXPANSION_RESEARCH.md`, "New small
//! axis: religious policy (per faction)"; `FH_R2_PLAN.md` §7, R2-6).
//!
//! **What it is.** Toleration is a property of the ruler, not of the religion
//! (the research document's own finding: the variable that decided whether
//! Karakorum or Venice kept its minorities). Each faction carries one of four
//! policies, and the belief step reads it in that faction's settlements as a
//! **modifier only** on the two terms the research document names --
//! minority retention (out-flow) and conversion into a minority -- never as a
//! write to any share ("the king converts is not everyone converts").
//!
//! **Defaults applied, for the owner to overrule.** `FH_R2_PLAN.md` Q7 ("add
//! the per-faction religious-policy axis as part of FH-R2?") has no ruling;
//! Ruling CW told this pass to take a plan's recommended default where a
//! question is unanswered. Q7's default is "yes, after S2, with every
//! faction's default policy carrying an exactly neutral modifier (1.0 on
//! retention and conversion), so adding the axis moves nothing until a policy
//! is changed". So:
//!
//! - the default is [`ReligiousPolicy::Plural`], whose modifiers are exactly
//!   `1.0`, and the belief layer does not even build a table for it (identity
//!   by control flow, `belief::BeliefTerms::with_policy`);
//! - the other three policies' numbers below are **labelled judgements**: the
//!   research document gives only their direction ("persecution raises
//!   minority out-flow, tolerant regimes lower it"), not magnitudes.
//!
//! **Who is a minority.** Every religion slot other than the faction's own
//! state religion (its engine key) and the unaffiliated slot: the
//! unaffiliated are not a rival faith (the same silence `relations.rs`'
//! `religion_term` and `belief_competition_weight` give `none`). A faction
//! whose state religion is `none` has no establishment, so every policy is
//! inert for it, by control flow; the shell says so.
//!
//! **What it must never do.** Never set or move a share directly; never apply
//! to the state religion or the unaffiliated slot; never make a minority's
//! yearly out-flow exceed its whole share (the step floors the effective
//! retention so `rate * (1 - ret) <= 1`); never change a run for a faction left
//! on the default.

/// One faction's religious policy. Keys are saved by string
/// (`SAVEFILE_COMPAT.md` §9.2's `religious_policy`); never rename one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReligiousPolicy {
    /// Non-established, all cults licit (the research document's Mongol
    /// practice). The default; exactly neutral.
    #[default]
    Plural,
    /// A state faith with protected minorities (millet, dhimma).
    TolerantEstablishment,
    /// A state faith; others restricted.
    ExclusiveEstablishment,
    /// Minorities persecuted.
    Persecuting,
}

/// `(key, label)` for the four policies, in the order the shell lists them.
pub const CIV_RELIGIOUS_POLICIES: [(&str, &str); 4] = [
    ("plural", "Plural (no established faith)"),
    ("tolerant_establishment", "Tolerant establishment"),
    ("exclusive_establishment", "Exclusive establishment"),
    ("persecuting", "Persecuting"),
];

impl ReligiousPolicy {
    /// The saved key.
    pub fn key(self) -> &'static str {
        CIV_RELIGIOUS_POLICIES[self as usize].0
    }

    /// The shell's label.
    pub fn label(self) -> &'static str {
        CIV_RELIGIOUS_POLICIES[self as usize].1
    }

    /// The policy for a saved key, or `None` for a key this build does not
    /// know. A caller that must still run treats `None` as no modifier (the
    /// default's behaviour) and keeps the key itself, never rewriting it.
    pub fn from_key(key: &str) -> Option<Self> {
        [Self::Plural, Self::TolerantEstablishment, Self::ExclusiveEstablishment, Self::Persecuting]
            .into_iter()
            .find(|p| p.key() == key)
    }

    /// `φ`, the multiplier on a minority's yearly out-flow
    /// `rate · (1 − ret) · share`, applied as the effective retention
    /// `1 − (1 − ret)·φ`. Labelled judgements on the research document's
    /// direction (tolerant lowers it, persecution raises it):
    ///
    /// - plural `1.0` -- neutral (Q7's default);
    /// - tolerant establishment `0.5` -- a protected community (millet) keeps
    ///   its people: half the out-flow;
    /// - exclusive establishment `2.0` -- restricted faiths leak twice as fast;
    /// - persecuting `4.0` -- four times, which at the step rate
    ///   `BELIEF_STEP_RATE` (0.05) is a fifth of a folk minority re-choosing
    ///   every year.
    pub fn minority_outflow(self) -> f64 {
        match self {
            Self::Plural => 1.0,
            Self::TolerantEstablishment => 0.5,
            Self::ExclusiveEstablishment => 2.0,
            Self::Persecuting => 4.0,
        }
    }

    /// `κ`, the multiplier on a minority's conversion pull (its `attract`
    /// term) in the faction's settlements. Labelled judgements:
    ///
    /// - plural `1.0` -- neutral;
    /// - tolerant establishment `0.5` -- protected but not proselytising (the
    ///   dhimma forbade seeking converts from the state faith);
    /// - exclusive establishment `0.25`;
    /// - persecuting `0.1` -- joining a persecuted faith is rare, not
    ///   impossible (the policy never sets a share).
    pub fn minority_conversion(self) -> f64 {
        match self {
            Self::Plural => 1.0,
            Self::TolerantEstablishment => 0.5,
            Self::ExclusiveEstablishment => 0.25,
            Self::Persecuting => 0.1,
        }
    }

    /// True for the policy that changes nothing (Q7's neutral default).
    pub fn is_neutral(self) -> bool {
        self.minority_outflow() == 1.0 && self.minority_conversion() == 1.0
    }
}

/// Tests of the policy vocabulary and its modifiers.
#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: the saved keys (literals: saves store them), the round trip,
    /// and that the default is plural.
    #[test]
    fn keys_are_stable_and_round_trip() {
        let keys: Vec<&str> = CIV_RELIGIOUS_POLICIES.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, ["plural", "tolerant_establishment", "exclusive_establishment", "persecuting"]);
        for (k, _) in CIV_RELIGIOUS_POLICIES {
            assert_eq!(ReligiousPolicy::from_key(k).map(ReligiousPolicy::key), Some(k));
        }
        assert_eq!(ReligiousPolicy::from_key("theocratic"), None);
        assert_eq!(ReligiousPolicy::default(), ReligiousPolicy::Plural);
    }

    /// Protects: Q7's default -- only plural is neutral, and its modifiers are
    /// exactly 1.0 -- plus the research document's direction as literals:
    /// out-flow tolerant < plural < exclusive < persecuting; conversion into a
    /// minority falls monotonically from plural to persecuting and never
    /// reaches zero (a policy never sets a share).
    #[test]
    fn only_plural_is_neutral_and_the_rest_follow_the_direction() {
        use ReligiousPolicy::*;
        assert!(Plural.is_neutral());
        assert_eq!((Plural.minority_outflow(), Plural.minority_conversion()), (1.0, 1.0));
        for p in [TolerantEstablishment, ExclusiveEstablishment, Persecuting] {
            assert!(!p.is_neutral(), "{p:?}");
            assert!(p.minority_conversion() > 0.0, "{p:?}");
        }
        assert_eq!(
            [TolerantEstablishment, Plural, ExclusiveEstablishment, Persecuting].map(ReligiousPolicy::minority_outflow),
            [0.5, 1.0, 2.0, 4.0]
        );
        assert_eq!(
            [Plural, TolerantEstablishment, ExclusiveEstablishment, Persecuting].map(ReligiousPolicy::minority_conversion),
            [1.0, 0.5, 0.25, 0.1]
        );
    }
}
