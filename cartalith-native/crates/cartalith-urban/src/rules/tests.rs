//! Milestone 4's tests. **All fifty rule cases, both profiles and all fifteen
//! `resolveProfile` cases are golden** — every item in this milestone is on
//! `UME`'s public export, so the capture calls the reference's own functions
//! directly and nothing here is a hand-computed expectation.
//!
//! Each scenario is rebuilt on this side by the same sequence of calls the
//! capture script made on the reference's side, then compared field for field
//! (bit-for-bit via [`f64::to_bits`], so a `NaN` must be a `NaN` and a `-0`
//! could not pass for a `+0`). No tolerances anywhere.
//!
//! # The verification requirement milestone 3 established, applied here
//!
//! Milestone 3's finding was that a passing golden suite proves nothing until
//! mutation testing says it does, and that the discriminators come from
//! **quantised and symmetric** inputs rather than continuous random ones. This
//! milestone is all constants and one clamp, so the equivalent of a quantised
//! fixture is a slider argument that lands *exactly on* a clamp bound or a
//! rounding boundary. Those are here on purpose:
//!
//! - `wild_0`, `wild_2`, `wild_neg1`, `wild_3` and both infinities straddle or
//!   saturate every one of the eleven clamps, so each `lo`/`hi` literal is the
//!   value that actually comes out somewhere.
//! - `wild_NaN` and `chaos_NaN` are the discriminator for [`super::js_min`] /
//!   [`super::js_max`] against `f64::min` / `f64::max` — the single mutation
//!   most likely to be made by a later reader "simplifying" [`super::clamp`],
//!   and one that **no finite input can detect**, exactly the shape of hole
//!   milestone 3 found.
//! - `chaos_0p75`, `chaos_1p25` and `chaos_1p75` put `2 * c` on `1.5`, `2.5`
//!   and `3.5` exactly, the only inputs that can tell `Math.round` from
//!   `floor`, `ceil` or `trunc`; and `chaos_0p7475` / `chaos_1p2475` /
//!   `chaos_1p7475` sit just *below* those boundaries, which is what makes a
//!   small perturbation of the `2` multiplier observable at all. Those three
//!   exist because the first mutation round found the multiplier unexercised:
//!   `subdivision_cap` is a **quantised output**, and a rounded value cannot
//!   observe any change to its inputs smaller than its own step unless some
//!   input sits within that step of a boundary. Same lesson as milestone 3's
//!   `ties*` scenarios, arriving from the output side rather than the input
//!   side.
//! - `wildTwice1p5` / `wildThrice2` / `wildFive2` are the discriminator for
//!   `dead_end_bias`'s accumulation, which a single application cannot see.
//!
//! Fifteen mutations were run; see `URBAN_MORPHOLOGY_SCOPE.md` for the table
//! and the one reported survivor.

use super::*;

mod golden;

/// Bit-exact comparison, so `NaN == NaN` and `+0.0 != -0.0`.
fn same(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits()
}

fn check(name: &str, rules: &Rules, want: &[f64]) {
    let got = rules.flatten();
    assert_eq!(got.len(), want.len(), "{name}: width");
    assert_eq!(
        got.len(),
        golden::FIELDS.len(),
        "{name}: flatten() and the captured field list disagree"
    );
    for (i, (&g, &w)) in got.iter().zip(want.iter()).enumerate() {
        assert!(
            same(g, w),
            "{name}: field {} ({}) = {g:?}, reference = {w:?}",
            i,
            golden::FIELDS[i]
        );
    }
}

/// Rebuilds one captured scenario by the same call sequence the capture script
/// ran against the reference. Returns `None` for a name this port does not
/// know, so [`every_captured_scenario_is_rebuilt`] can fail loudly rather than
/// let a scenario silently go unchecked.
fn rebuild(name: &str) -> Option<Rules> {
    let fresh = DEFAULT_RULES;
    let wild = |w: f64| {
        let mut r = fresh;
        apply_wildness(&mut r, w);
        r
    };
    let chaos = |c: f64| {
        let mut r = fresh;
        apply_plot_chaos(&mut r, c);
        r
    };
    Some(match name {
        "defaults" | "defaultsAfterMerges" => DEFAULT_RULES,
        // `cloneRules` does not survive as a function; `Copy` is the deep clone.
        "cloned" => DEFAULT_RULES,

        "resolveNull" | "resolveUndefined" => resolve_rules(None),
        "resolveEmpty" => resolve_rules(Some(&RulesPatch::default())),
        "resolveOneField" => resolve_rules(Some(&RulesPatch {
            street: Some(StreetPatch {
                pierce_chance: Some(0.5),
                ..Default::default()
            }),
            ..Default::default()
        })),
        "resolveWholeStreetGroupPartial" => resolve_rules(Some(&RulesPatch {
            street: Some(StreetPatch {
                branch_angle_jitter: Some(1.5),
                dead_end_bias: Some(0.4),
                ..Default::default()
            }),
            ..Default::default()
        })),
        "resolveTwoGroups" => resolve_rules(Some(&RulesPatch {
            parcels: Some(ParcelPatch {
                subdivision_cap: Some(4.0),
                ..Default::default()
            }),
            settlement: Some(SettlementPatch {
                max_wall_generations: Some(1.0),
                ..Default::default()
            }),
            ..Default::default()
        })),
        "resolveMeta" => resolve_rules(Some(&RulesPatch {
            meta: Some(MetaPatch {
                wildness: Some(0.3),
                plot_chaos: Some(1.9),
            }),
            ..Default::default()
        })),
        // The reference's partial also carried a `bogus` group, which its loop
        // over `Object.keys(out)` ignores. A typed patch has no such group at
        // all, so the *observable* case is the one that survives: the known
        // group still merges.
        "resolveUnknownGroup" => resolve_rules(Some(&RulesPatch {
            street: Some(StreetPatch {
                pierce_chance: Some(0.02),
                ..Default::default()
            }),
            ..Default::default()
        })),
        // `{street:null, parcels:0, settlement:false, meta:''}` — every group
        // falsy, so `if(partial[grp])` skips all four and the defaults survive.
        "resolveFalsyGroups" => resolve_rules(Some(&RulesPatch::default())),
        "resolveAllGroups" => resolve_rules(Some(&RulesPatch {
            street: Some(StreetPatch {
                branch_angle_jitter: Some(0.01),
                continuation_jitter: Some(0.02),
                exploration_start: Some(0.03),
                exploration_decay: Some(0.04),
                exploration_minimum: Some(0.05),
                segment_length_median: Some(6.0),
                segment_length_variance: Some(0.07),
                pierce_chance: Some(0.08),
                junction_angle_limit: Some(0.09),
                market_gradient_decay: Some(10.0),
                parallel_street_spacing: Some(11.0),
                dead_end_bias: Some(0.12),
                bridgehead_distance: Some(13.0),
                bridgehead_probability: Some(0.14),
            }),
            parcels: Some(ParcelPatch {
                frontage_width_variance: Some(0.15),
                plot_depth_variance: Some(0.16),
                subdivision_cap: Some(17.0),
            }),
            settlement: Some(SettlementPatch {
                wall_generation_threshold: Some(0.18),
                wall_generation_min_age_gap: Some(19.0),
                wall_generation_extramural_share: Some(0.2),
                max_wall_generations: Some(21.0),
                carrying_capacity_weight: Some(0.22),
            }),
            meta: Some(MetaPatch {
                wildness: Some(0.23),
                plot_chaos: Some(0.24),
            }),
        })),

        "wild_0" => wild(0.0),
        "wild_0p25" => wild(0.25),
        "wild_0p5" => wild(0.5),
        "wild_0p75" => wild(0.75),
        "wild_1" => wild(1.0),
        "wild_1p25" => wild(1.25),
        "wild_1p5" => wild(1.5),
        "wild_1p75" => wild(1.75),
        "wild_2" => wild(2.0),
        "wild_2p5" => wild(2.5),
        "wild_3" => wild(3.0),
        "wild_neg1" => wild(-1.0),
        "wild_NaN" => wild(f64::NAN),
        "wild_Infinity" => wild(f64::INFINITY),
        "wild_negInfinity" => wild(f64::NEG_INFINITY),
        "wildTwice1p5" => {
            let mut r = fresh;
            apply_wildness(&mut r, 1.5);
            apply_wildness(&mut r, 1.5);
            r
        }
        "wildThrice2" => {
            let mut r = fresh;
            for _ in 0..3 {
                apply_wildness(&mut r, 2.0);
            }
            r
        }
        "wildFive2" => {
            let mut r = fresh;
            for _ in 0..5 {
                apply_wildness(&mut r, 2.0);
            }
            r
        }
        "wildOverCustom" => {
            let mut r = resolve_rules(Some(&RulesPatch {
                street: Some(StreetPatch {
                    branch_angle_jitter: Some(9.0),
                    segment_length_median: Some(999.0),
                    dead_end_bias: Some(0.3),
                    ..Default::default()
                }),
                ..Default::default()
            }));
            apply_wildness(&mut r, 1.0);
            r
        }

        "chaos_0" => chaos(0.0),
        "chaos_0p4" => chaos(0.4),
        "chaos_0p5" => chaos(0.5),
        "chaos_0p7475" => chaos(0.7475),
        "chaos_0p75" => chaos(0.75),
        "chaos_1" => chaos(1.0),
        "chaos_1p2475" => chaos(1.2475),
        "chaos_1p25" => chaos(1.25),
        "chaos_1p5" => chaos(1.5),
        "chaos_1p7475" => chaos(1.7475),
        "chaos_1p75" => chaos(1.75),
        "chaos_2" => chaos(2.0),
        "chaos_2p2" => chaos(2.2),
        "chaos_3" => chaos(3.0),
        "chaos_neg1" => chaos(-1.0),
        "chaos_NaN" => chaos(f64::NAN),
        "chaos_Infinity" => chaos(f64::INFINITY),
        "chaosTwice1p25" => {
            let mut r = fresh;
            apply_plot_chaos(&mut r, 1.25);
            apply_plot_chaos(&mut r, 1.25);
            r
        }
        "wildThenChaos" => {
            let mut r = fresh;
            apply_wildness(&mut r, 1.6);
            apply_plot_chaos(&mut r, 0.6);
            r
        }
        "chaosThenWild" => {
            let mut r = fresh;
            apply_plot_chaos(&mut r, 0.6);
            apply_wildness(&mut r, 1.6);
            r
        }
        "mergeThenBoth" => {
            let mut r = resolve_rules(Some(&RulesPatch {
                street: Some(StreetPatch {
                    pierce_chance: Some(0.13),
                    ..Default::default()
                }),
                parcels: Some(ParcelPatch {
                    subdivision_cap: Some(1.0),
                    ..Default::default()
                }),
                ..Default::default()
            }));
            apply_wildness(&mut r, 0.5);
            apply_plot_chaos(&mut r, 1.8);
            r
        }

        _ => return None,
    })
}

/// **Golden.** Every captured rule set, rebuilt and compared bit for bit.
#[test]
fn rules_match_the_reference() {
    for case in golden::RULES {
        let rules = rebuild(case.name)
            .unwrap_or_else(|| panic!("no rebuild for captured scenario {:?}", case.name));
        check(case.name, &rules, case.values);
    }
}

/// The emptiness / shape gate, on this side of the wire as well as the
/// capture's. Three subsystems in this project have shipped a harness whose
/// output was silently empty and passed every structural check; an assertion
/// that the goldens are actually populated *and actually vary* is the only one
/// that catches all three.
#[test]
fn every_captured_scenario_is_rebuilt() {
    assert!(golden::RULES.len() >= 53, "capture shrank");
    assert_eq!(golden::FIELDS.len(), 24);
    assert_eq!(golden::PROFILES.len(), 2);
    assert_eq!(golden::RESOLVE_PROFILE.len(), 15);
    for case in golden::RULES {
        assert!(
            rebuild(case.name).is_some(),
            "captured scenario {:?} has no rebuild — it is being silently skipped",
            case.name
        );
        assert_eq!(case.values.len(), 24, "{}", case.name);
    }
    let base = golden::RULES[0].values;
    let varied = golden::RULES
        .iter()
        .filter(|c| c.values.iter().zip(base).any(|(a, b)| !same(*a, *b)))
        .count();
    assert!(varied >= 30, "goldens barely vary: {varied}");
}

/// **Golden.** Both culture profiles, every field, including the two the
/// reference leaves off `medieval` entirely.
#[test]
fn culture_profiles_match_the_reference() {
    assert_eq!(CULTURE_PROFILES.len(), golden::PROFILES.len());
    for (got, want) in CULTURE_PROFILES.iter().zip(golden::PROFILES) {
        let n = want.key;
        assert_eq!(got.id, want.key, "{n}: key order");
        assert_eq!(got.id, want.id, "{n}: id");
        assert_eq!(got.name, want.name, "{n}: name");
        assert_eq!(got.planning, want.planning, "{n}: planning");
        assert_eq!(got.parcel_pattern, want.parcel_pattern, "{n}: parcelPattern");
        assert_eq!(
            got.building_grammar, want.building_grammar,
            "{n}: buildingGrammar"
        );
        assert_eq!(got.default_faith, want.default_faith, "{n}: defaultFaith");
        assert_eq!(got.default_civic, want.default_civic, "{n}: defaultCivic");
        assert_eq!(got.markets, want.markets, "{n}: markets");
        assert_eq!(
            got.wall_gates_scheme, want.wall_gates_scheme,
            "{n}: wallGates.scheme"
        );
        assert_eq!(got.orientation, want.orientation, "{n}: orientation");
        assert_eq!(
            got.civic_anchor_label, want.civic_anchor_label,
            "{n}: civicAnchorLabel"
        );
        assert_eq!(got.default_walls, want.default_walls, "{n}: defaultWalls");
        assert_eq!(got.waterway, want.waterway, "{n}: waterway");
        assert_eq!(got.prov, want.prov, "{n}: prov");
    }
}

/// **Golden, and a finding milestone 11 depends on.** `privatizeAlleys` reads
/// `(profile.deadEndBias||0)`, and neither live profile defines the key — so
/// the profile side of that sum is always zero. The capture asserts the absence
/// against the reference's own key list; this asserts the value the port
/// carries is the one `||0` yields.
#[test]
fn no_live_profile_defines_dead_end_bias() {
    for want in golden::PROFILES {
        assert!(
            !want.keys.contains(&"deadEndBias"),
            "{}: the reference now defines deadEndBias — milestone 11's \
             clamp((profile.deadEndBias||0)+…) is no longer a no-op on the profile side",
            want.key
        );
    }
    for got in &CULTURE_PROFILES {
        assert!(same(got.dead_end_bias, 0.0), "{}", got.id);
    }
}

/// **Golden, with one deliberate divergence.** `resolveProfile` falls back to
/// `medieval` for every unknown id — except that the reference indexes a plain
/// object literal, so five `Object.prototype` names come back truthy and
/// *escape the fallback entirely*, returning a function or `Object.prototype`
/// instead of a profile. This port returns `medieval` for those too, and the
/// test states the divergence rather than asserting the port matches.
#[test]
fn resolve_profile_matches_the_reference_except_on_the_prototype_chain() {
    let mut hazards = 0;
    for case in golden::RESOLVE_PROFILE {
        // `null`/`undefined` stringify to those two ids in the capture; the
        // reference reaches them as `CULTURE_PROFILES[null]`, i.e. the string
        // key `"null"`, which is equally absent.
        let got = resolve_profile(case.id);
        if case.is_profile {
            assert_eq!(
                Some(got.id),
                case.got,
                "resolveProfile({:?}) diverges",
                case.id
            );
        } else {
            hazards += 1;
            assert!(
                case.type_of == "function" || case.type_of == "object",
                "unexpected prototype hazard shape"
            );
            assert_eq!(
                got.id, "medieval",
                "the port must harden {:?} to medieval, not reproduce the \
                 prototype-chain leak",
                case.id
            );
        }
    }
    assert_eq!(
        hazards, 5,
        "the reference's prototype-chain leak changed shape"
    );
}

/// Unit test of the port's own logic, labelled as such: `clamp` is only
/// *observable* through the two sliders, so its NaN behaviour is asserted
/// directly as well as golden-enforced through `wild_NaN`/`chaos_NaN`.
///
/// The `assert_ne!` is the same device `geom::js_hypot` carries: it exists so
/// that anyone who "simplifies" [`clamp`] to `lo.max(hi.min(v))` fails here
/// with the reason written out, not three milestones later inside `grow`.
#[test]
fn clamp_propagates_nan_where_rust_min_max_would_absorb_it() {
    assert!(clamp(f64::NAN, 0.15, 0.70).is_nan());
    assert!(clamp(f64::NAN, 0.0, 0.15).is_nan());
    // What the naive transliteration `lo.max(hi.min(v))` would have produced
    // instead: `f64::min` absorbs the NaN and hands back `hi`, `f64::max` then
    // keeps it, so *every* clamped field lands on its own upper bound — a
    // maximally-wild rule set that looks entirely plausible.
    assert!(!0.15f64.max(0.70f64.min(f64::NAN)).is_nan());
    assert_eq!(0.15f64.max(0.70f64.min(f64::NAN)), 0.70);
    assert_eq!(0.0f64.max(0.15f64.min(f64::NAN)), 0.15);

    // Ordinary clamping, and both saturation directions.
    assert_eq!(clamp(0.5, 0.15, 0.70), 0.5);
    assert_eq!(clamp(-2.0, 0.15, 0.70), 0.15);
    assert_eq!(clamp(9.0, 0.15, 0.70), 0.70);
    assert_eq!(clamp(f64::INFINITY, 0.15, 0.70), 0.70);
    assert_eq!(clamp(f64::NEG_INFINITY, 0.15, 0.70), 0.15);
    // Exactly on a bound.
    assert_eq!(clamp(0.15, 0.15, 0.70), 0.15);
    assert_eq!(clamp(0.70, 0.15, 0.70), 0.70);
    // `lo > hi` returns `lo`, where `f64::clamp` would panic. Unreachable from
    // the eleven call sites (all literal, all `lo < hi`), asserted so the
    // difference is on the record.
    assert_eq!(clamp(0.5, 0.9, 0.1), 0.9);
}

/// Unit test: the rounding boundary `applyPlotChaos` sits on. Golden-covered by
/// `chaos_0p75`/`chaos_1p25`/`chaos_1p75`, asserted here in the form that names
/// what is being distinguished.
#[test]
fn subdivision_cap_rounds_halves_up_like_math_round() {
    let cap = |c: f64| {
        let mut r = DEFAULT_RULES;
        apply_plot_chaos(&mut r, c);
        r.parcels.subdivision_cap
    };
    assert_eq!(cap(0.75), 2.0, "2*0.75 = 1.5 -> 2");
    assert_eq!(cap(1.25), 3.0, "2*1.25 = 2.5 -> 3");
    assert_eq!(cap(1.75), 4.0, "2*1.75 = 3.5 -> 4");
    // The clamp bounds hold on either side.
    assert_eq!(cap(0.0), 1.0);
    assert_eq!(cap(-5.0), 1.0);
    assert_eq!(cap(99.0), 4.0);
    assert!(cap(f64::NAN).is_nan(), "a NaN slider must stay NaN");
}

/// Unit test of a structural property the port gets for free and the reference
/// had to write `cloneRules` for: nothing can alias [`DEFAULT_RULES`].
///
/// It also records the one place `cloneRules` is **not** a deep clone. It is
/// `JSON.parse(JSON.stringify(r))`, so a NaN-poisoned rule set comes back with
/// `null` in place of every NaN — the capture pins that the reference really
/// does this. A typed [`Rules`] has no `null` to land on, so the port keeps the
/// NaN. Unreachable inside the engine: `resolveRules` clones the all-finite
/// defaults and assigns the caller's partial on top of the clone, so nothing a
/// caller supplies is ever round-tripped.
#[test]
fn the_defaults_cannot_be_mutated_and_clone_rules_does_not_survive() {
    assert!(
        golden::CLONE_NAN_BECOMES.is_none(),
        "the reference's cloneRules no longer turns NaN into JSON null"
    );
    let mut r = DEFAULT_RULES;
    apply_wildness(&mut r, f64::NAN);
    assert!(r.street.branch_angle_jitter.is_nan());
    // The port's clone keeps it, where the reference's would produce null.
    let cloned = r;
    assert!(cloned.street.branch_angle_jitter.is_nan());
    // And the module constant is untouched by any of it.
    assert_eq!(DEFAULT_RULES.street.branch_angle_jitter, 0.26);
    assert_eq!(DEFAULT_RULES.meta.wildness, 1.0);
}

/// Unit test: which fields each compound slider is allowed to touch. The
/// reference's own comment claims the sliders "compute new values for the
/// individual street/parcel fields"; this pins exactly which, so a later
/// milestone that finds `market_gradient_decay` unexpectedly moved knows it did
/// not come from here.
#[test]
fn the_sliders_touch_only_their_own_fields() {
    let mut r = DEFAULT_RULES;
    apply_wildness(&mut r, 1.7);
    // Four street fields, and both other rule groups, are untouched.
    assert_eq!(r.street.exploration_decay, 0.05);
    assert_eq!(r.street.segment_length_median, 56.0);
    assert_eq!(r.street.market_gradient_decay, 200.0);
    assert_eq!(r.street.bridgehead_distance, 190.0);
    assert_eq!(r.parcels, DEFAULT_RULES.parcels);
    assert_eq!(r.settlement, DEFAULT_RULES.settlement);
    assert_eq!(r.meta.plot_chaos, 1.0);

    let mut r = DEFAULT_RULES;
    apply_plot_chaos(&mut r, 1.7);
    assert_eq!(r.street, DEFAULT_RULES.street);
    assert_eq!(r.settlement, DEFAULT_RULES.settlement);
    assert_eq!(r.meta.wildness, 1.0);
}

/// Unit test naming the non-idempotence directly, since it is the one property
/// of this milestone a later reader is most likely to "fix". `apply_plot_chaos`
/// is idempotent; `apply_wildness` is not, and only because of one field.
#[test]
fn apply_wildness_accumulates_dead_end_bias_and_is_therefore_not_idempotent() {
    let mut once = DEFAULT_RULES;
    apply_wildness(&mut once, 1.6);
    let mut twice = once;
    apply_wildness(&mut twice, 1.6);
    assert_ne!(once, twice, "the accumulation must be observable");
    assert_eq!(once.street.dead_end_bias, twice.street.dead_end_bias / 2.0);
    // Everything else is stable under re-application.
    let mut once_but_bias_matched = once;
    once_but_bias_matched.street.dead_end_bias = twice.street.dead_end_bias;
    assert_eq!(once_but_bias_matched, twice);

    // And it saturates at the 0.40 cap rather than growing without bound.
    let mut many = DEFAULT_RULES;
    for _ in 0..20 {
        apply_wildness(&mut many, 2.0);
    }
    assert_eq!(many.street.dead_end_bias, 0.40);

    let mut once = DEFAULT_RULES;
    apply_plot_chaos(&mut once, 1.3);
    let mut twice = once;
    apply_plot_chaos(&mut twice, 1.3);
    assert_eq!(once, twice, "apply_plot_chaos is idempotent");
}
