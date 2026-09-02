//! Name shaping for things the reference does not name.
//!
//! Two jobs, both prompted by comparing against Nortantis 3.18 on
//! 2026-08-30, and both confined to callers with **no parity contract**.
//!
//! # Why nothing here touches `civ_settle_name`
//!
//! [`crate::civ_settle_name`]'s own doc states the constraint: it *"Consumes
//! `1 + n + 1` RNG calls in that exact order -- callers sharing this RNG
//! stream (population generation, same call site) depend on that count."* A
//! rejection loop draws a different number of values, so putting one inside it
//! desynchronises population variance for every settlement after the first and
//! re-baselines `golden_parity_settlement_naming`. That is a `DECISIONS.md`
//! §7a matter and an owner call, not an engineering one.
//!
//! So the filter lives *outside*, and is applied only where the reference has
//! no behaviour to match: continents, provinces, and the user-initiated
//! re-roll button. Generated settlement names are untouched and still
//! bit-identical.
//!
//! # 1. Length rejection, self-tuned per culture
//!
//! `golden_parity_settlement_naming` pins **`"Hurngarngarnhaskcairn"`** (21
//! characters) and `"Ghalbahrghaltazdune"` (19). Those are real outputs, and
//! on a map they read as noise rather than as places.
//!
//! Nortantis rejects the same shape -- `NameGenerator.generateName` retries up
//! to 10 times against `longestWord.length() > averageWordLength * 2.0`, with
//! the mean measured from the corpus rather than hardcoded, so the rule
//! self-tunes to whatever vocabulary it is given. [`culture_mean_name_len`]
//! does the same thing from the culture's own syllable and suffix pools, so a
//! culture with long syllables is not punished for having them.
//!
//! # 2. Feature-name templates
//!
//! Before this, exactly one template existed in the whole port --
//! `format!("{} Province", s.name)` -- and continents took the bare settlement
//! stem, so a continent came out called "Sevjuniana", which reads as a town.
//! Nortantis's `NameCreator.generateNameOfType` is weighted post-processing of
//! a bare stem per feature kind, entirely independent of how the stem was
//! made, which is why it transfers to syllable pools unchanged.

use crate::{CIV_CULTURES, Culture, civ_default_culture, civ_settle_name};
use std::collections::BTreeSet;

/// How far past the mean a name may run before it is rejected.
///
/// **Nortantis's own number is 2.0 and it does not transfer, which was
/// measured rather than assumed.** There, `averageWordLength` is the mean of a
/// *source corpus word* -- a real place name, six to eight characters -- so
/// `×2.0` lands around 14. Here the mean is of the *generated output*, which
/// is already a whole name (`3 · mean(syl) + mean(sfx)`, 12.1 to 16.6 across
/// the seven cultures). Applying 2.0 to that gave limits of 25 to 34, which
/// accept every one of the over-long names this rule exists to reject --
/// `golden_parity_settlement_naming`'s own "Hurngarngarnhaskcairn" is 21.
///
/// 1.4 is derived from the generator's own shape rather than picked: it draws
/// `n ∈ {2, 3, 4}` syllables uniformly and the mean above is computed at
/// `n = 3`, so a four-syllable name runs about `4/3 ≈ 1.33` of the mean. A
/// limit at 1.4 therefore keeps every ordinary four-syllable name and trims
/// only the tail where a long `n` meets long syllables -- which is the tail
/// those two fixture names sit in.
pub const NAME_LEN_FACTOR: f64 = 1.4;

/// Nortantis's own retry ceiling. After this many tries the last candidate is
/// returned rather than looping forever -- a name that is too long beats no
/// name, and a pool of very long syllables can legitimately have no short
/// member.
pub const NAME_MAX_TRIES: usize = 10;

/// The mean length `civ_settle_name` produces for one culture, in characters.
///
/// It draws `n ∈ {2, 3, 4}` syllables (uniform, from `2 + (f64 * 3.0) as
/// usize`) and one suffix, so the expectation is `3 · mean(syl) + mean(sfx)`.
/// Computed from the pools rather than stated, so it stays true if a pool is
/// ever edited.
pub fn culture_mean_name_len(cul: &Culture) -> f64 {
    let mean = |v: &[&str]| -> f64 {
        if v.is_empty() {
            return 0.0;
        }
        v.iter().map(|s| s.chars().count() as f64).sum::<f64>() / v.len() as f64
    };
    3.0 * mean(cul.syl) + mean(cul.sfx)
}

/// The length above which a name for this culture is rejected.
pub fn culture_name_len_limit(cul: &Culture) -> usize {
    (culture_mean_name_len(cul) * NAME_LEN_FACTOR).ceil() as usize
}

/// `civ_settle_name` with the two properties a map wants and the generator
/// does not provide: a length bound and per-map uniqueness.
///
/// **Not for generated settlements** -- see this module's header. `seen` is
/// the caller's own set, so uniqueness is a property of the map being built
/// rather than global state in this stateless crate (`ARCHITECTURE.md`).
///
/// Falls back to the last candidate after [`NAME_MAX_TRIES`], and if even that
/// collides it appends a numeric discriminator rather than returning a
/// duplicate -- a caller asking for uniqueness gets it or gets told, never a
/// silent repeat.
pub fn civ_settle_name_bounded(
    rng: &mut cartalith_rng::Mulberry32,
    faction: i32,
    seen: &mut BTreeSet<String>,
) -> String {
    let limit = culture_name_len_limit(civ_default_culture(faction));
    let mut last = String::new();
    for _ in 0..NAME_MAX_TRIES {
        let cand = civ_settle_name(rng, faction);
        if cand.chars().count() <= limit && !seen.contains(&cand) {
            seen.insert(cand.clone());
            return cand;
        }
        last = cand;
    }
    if seen.contains(&last) {
        for n in 2usize.. {
            let alt = format!("{last} {n}");
            if !seen.contains(&alt) {
                seen.insert(alt.clone());
                return alt;
            }
        }
    }
    seen.insert(last.clone());
    last
}

/// What a stem is being named, for [`decorate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureKind {
    /// A landmass. The case that prompted this: a continent called
    /// "Sevjuniana" reads as a settlement.
    Continent,
    /// An administrative division. The port's one pre-existing template.
    Province,
    /// A named stretch of water.
    Bay,
    /// A range of high ground.
    MountainRange,
}

/// Turn a bare stem into a name that reads as its own kind of thing.
///
/// Weighted, so a map does not read as a template: most continents take a
/// possessive or bare form and a minority take the grander ones. The weights
/// are this port's own -- Nortantis's exact percentages are tuned to its own
/// feature mix and are not a parity target, since none of this exists in the
/// reference at all.
///
/// Consumes exactly **one** RNG value, whatever the branch, so a caller can
/// reason about its stream. That is deliberate: the branch is chosen from a
/// single draw rather than one draw per test.
pub fn decorate(stem: &str, kind: FeatureKind, rng: &mut cartalith_rng::Mulberry32) -> String {
    let r = rng.next_f64();
    match kind {
        FeatureKind::Continent => match r {
            x if x < 0.18 => format!("The {stem} Reach"),
            x if x < 0.32 => format!("Greater {stem}"),
            x if x < 0.42 => format!("The {stem} Expanse"),
            _ => stem.to_string(),
        },
        // The port's existing form, kept as the dominant one so provinces do
        // not all change name the day this lands.
        FeatureKind::Province => match r {
            x if x < 0.70 => format!("{stem} Province"),
            x if x < 0.85 => format!("The {stem} March"),
            _ => format!("{stem} County"),
        },
        FeatureKind::Bay => match r {
            x if x < 0.50 => format!("{stem} Bay"),
            x if x < 0.75 => format!("Gulf of {stem}"),
            _ => format!("{stem} Sound"),
        },
        FeatureKind::MountainRange => match r {
            x if x < 0.55 => format!("The {stem} Range"),
            x if x < 0.80 => format!("The {stem} Mountains"),
            _ => format!("{stem} Heights"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_culture_has_a_workable_length_limit() {
        for cul in CIV_CULTURES.iter() {
            let mean = culture_mean_name_len(cul);
            let limit = culture_name_len_limit(cul);
            assert!(mean > 0.0, "{} has an empty pool", cul.key);
            assert!(
                limit >= 8 && limit <= 40,
                "{}'s limit {limit} (mean {mean:.1}) is outside anything sane",
                cul.key
            );
        }
    }

    /// The defect this exists for, pinned against the real fixture values.
    ///
    /// `golden_parity_settlement_naming`'s own module doc says *"faction 1 ->
    /// `CIV_CULTURES[1 % 7]` = imperial"*, and its expectations include
    /// "Hurngarngarnhaskcairn" (21 chars) and "Ghalbahrghaltazdune" (19). The
    /// bound must reject the 21 for the culture that can produce it.
    ///
    /// It deliberately does NOT assert that every culture rejects them.
    /// `maritime`'s syllables are genuinely longer (mean 16.6 against
    /// `common`'s 12.1), so a longer name there is in character -- that the
    /// limit rises with the pool is the self-tuning working, not a hole.
    #[test]
    fn the_golden_fixtures_own_overlong_name_is_rejected() {
        let imperial = civ_default_culture(1);
        assert_eq!(imperial.key, "imperial", "the fixture's own faction-1 culture");
        let limit = culture_name_len_limit(imperial);
        assert!(
            "Hurngarngarnhaskcairn".chars().count() > limit,
            "limit {limit} would accept the 21-character name this rule exists to reject"
        );
        let common = civ_default_culture(0);
        assert_eq!(common.key, "common");
        assert!(
            "Ghalbahrghaltazdune".chars().count() > culture_name_len_limit(common),
            "the 19-character fixture name should be over `common`'s limit too"
        );
    }

    #[test]
    fn bounded_names_respect_the_limit_and_never_repeat() {
        let mut rng = cartalith_rng::Mulberry32::new(24601);
        let mut seen = BTreeSet::new();
        let limit = culture_name_len_limit(civ_default_culture(1));
        let mut over = 0;
        for _ in 0..200 {
            let n = civ_settle_name_bounded(&mut rng, 1, &mut seen);
            assert!(!n.is_empty(), "an empty name is never acceptable");
            if n.chars().count() > limit {
                over += 1;
            }
        }
        assert_eq!(seen.len(), 200, "200 draws must yield 200 distinct names");
        // The fallback can return an over-long name after 10 tries; that is
        // deliberate. It must be rare, not absent.
        assert!(over < 20, "{over}/200 exceeded the limit -- the retry is not working");
    }

    #[test]
    fn uniqueness_survives_a_pool_too_small_to_satisfy_it() {
        // Force collisions by asking for far more names than a single culture
        // can plausibly produce, and assert the discriminator path holds
        // rather than silently repeating.
        let mut rng = cartalith_rng::Mulberry32::new(7);
        let mut seen = BTreeSet::new();
        for _ in 0..500 {
            civ_settle_name_bounded(&mut rng, 3, &mut seen);
        }
        assert_eq!(seen.len(), 500, "uniqueness must hold even under pressure");
    }

    #[test]
    fn decorate_consumes_exactly_one_rng_value_whatever_the_branch() {
        for kind in [
            FeatureKind::Continent,
            FeatureKind::Province,
            FeatureKind::Bay,
            FeatureKind::MountainRange,
        ] {
            let mut a = cartalith_rng::Mulberry32::new(99);
            let mut b = cartalith_rng::Mulberry32::new(99);
            let _ = decorate("Sevjunia", kind, &mut a);
            let _ = b.next_f64();
            assert_eq!(
                a.next_f64(),
                b.next_f64(),
                "{kind:?} did not consume exactly one value"
            );
        }
    }

    #[test]
    fn decorate_always_contains_its_stem_and_changes_something() {
        let mut rng = cartalith_rng::Mulberry32::new(5);
        let mut decorated = 0;
        for _ in 0..200 {
            let out = decorate("Sevjunia", FeatureKind::Continent, &mut rng);
            assert!(out.contains("Sevjunia"), "the stem must survive: {out}");
            if out != "Sevjunia" {
                decorated += 1;
            }
        }
        assert!(
            decorated > 40 && decorated < 160,
            "{decorated}/200 decorated -- the weighting is degenerate"
        );
    }

    #[test]
    fn province_keeps_its_existing_form_as_the_common_case() {
        let mut rng = cartalith_rng::Mulberry32::new(11);
        let mut plain = 0;
        for _ in 0..300 {
            if decorate("Kel", FeatureKind::Province, &mut rng) == "Kel Province" {
                plain += 1;
            }
        }
        assert!(
            plain > 150,
            "only {plain}/300 kept the port's existing '<name> Province' form"
        );
    }
}
