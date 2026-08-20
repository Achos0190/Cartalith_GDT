//! Golden-parity tests for `cartalith-assets`' scatter rules against the
//! **real** reference implementation — `defaultScatterRule` (reference
//! `Cartalith Gen1 v2.10.html` line 6938), `scatterRuleKey` (6952),
//! `SCATTER_RULE_PRESETS`/`presetScatterRule` (6959/6971),
//! `normalizeScatterRule` (6987), `pickWeightedVariant` (7014),
//! `currentScatterRules` (7030), `autopopulateScatterRules` (7088) and
//! `pickIconVariant` (12171).
//!
//! Generated from a Node `vm` extraction run (harness transient, not checked
//! in — the same technique `tests/golden_parity_pack_manifest.rs` and
//! `cartalith-civ`'s golden tests use) that loads those functions plus `hash`
//! straight out of the frozen HTML by line range and calls them on the
//! fixtures below. **The expected values here are that run's output verbatim**,
//! not a hand-derivation of what the code "should" do.
//!
//! Why these are worth golden-testing rather than unit-testing alone:
//!
//! - `normalizeScatterRule` is a **coercion** function over untrusted input,
//!   and almost every interesting case is a JavaScript idiom rather than an
//!   algorithm — `+"2.5"` is 2.5 but `+"x"` is not, `0` is falsy but `"no"` is
//!   truthy, `Number.isFinite` does not coerce so a `"4"` in a biome list is
//!   dropped while a `5.5` is kept, and `density` falls back to a literal `1`
//!   rather than to the slot's own preset. Getting any of those wrong is
//!   invisible until a user's project file loads wrong.
//! - `pickWeightedVariant` is deterministic-hash-driven, so it diffs
//!   **exactly**: a 36-position sweep is compared index for index, including
//!   the three degenerate cases that must fall through to `pickIconVariant`'s
//!   untouched v1.25 hash.
//!
//! Rules are rendered to a canonical one-line string on both sides so the
//! comparison is exact and a failure reads as a diff rather than a struct
//! dump. All output is text and integers — there is no float tolerance to
//! argue about.

use cartalith_assets::{
    PackManifest, ScatterRule, ScatterRuleTable, autopopulate_scatter_rules, current_scatter_rules,
    normalize_scatter_rule, pick_icon_variant, pick_weighted_variant, preset_scatter_rule,
    scatter_rule_key,
};
use serde_json::{Value, json};

/// The exact shape the harness printed on the JavaScript side: field order as
/// `defaultScatterRule()` builds it, numbers via `String(n)` (which Rust's
/// `f64: Display` matches for every value in range here).
fn canon(r: &ScatterRule) -> String {
    let n = |v: f64| v.to_string();
    let opt = |v: Option<f64>| v.map_or("null".to_string(), n);
    let list = |v: &[f64]| {
        format!(
            "[{}]",
            v.iter().map(|x| n(*x)).collect::<Vec<_>>().join(",")
        )
    };
    format!(
        "enabled={} mode={} biomes={} minSize={} maxSize={} density={} spacing={} elevMin={} elevMax={} requireWetland={} variantWeights={}",
        r.enabled,
        r.mode.as_str(),
        list(&r.biomes),
        n(r.min_size),
        n(r.max_size),
        n(r.density),
        opt(r.spacing),
        opt(r.elev_min),
        opt(r.elev_max),
        r.require_wetland,
        r.variant_weights
            .as_deref()
            .map_or("null".to_string(), list),
    )
}

/// A whole table rendered the way the harness rendered it.
fn canon_table<'a>(entries: impl IntoIterator<Item = (&'a str, &'a ScatterRule)>) -> String {
    entries
        .into_iter()
        .map(|(k, r)| format!("{k} => {}", canon(r)))
        .collect::<Vec<_>>()
        .join(" ; ")
}

// ============================================================================
// defaultScatterRule / SCATTER_RULE_PRESETS / presetScatterRule
// ============================================================================

const DEFAULT_RULE: &str = "enabled=true mode=scatter biomes=[] minSize=0.7 maxSize=1.2 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null";

/// Every preset, in `SCATTER_RULE_PRESETS` order, plus two keys that have
/// none. These ten values *are* v1.25's hard-coded biome→asset switch, so a
/// drift here is a drift in what a freshly imported pack looks like.
const PRESETS: &[(&str, &str)] = &[
    (
        "mountain",
        "enabled=true mode=relief biomes=[] minSize=0.55 maxSize=1 density=1 spacing=null elevMin=0.58 elevMax=null requireWetland=false variantWeights=null",
    ),
    (
        "hill",
        "enabled=true mode=relief biomes=[] minSize=0.5 maxSize=1 density=1 spacing=null elevMin=0.53 elevMax=0.58 requireWetland=false variantWeights=null",
    ),
    (
        "tree_conifer",
        "enabled=true mode=scatter biomes=[3,4] minSize=0.7 maxSize=1.2 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
    ),
    (
        "tree_broadleaf",
        "enabled=true mode=scatter biomes=[5] minSize=0.7 maxSize=1.2 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
    ),
    (
        "tree_rainforest",
        "enabled=true mode=scatter biomes=[6,12] minSize=0.7 maxSize=1.2 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
    ),
    (
        "tree_savanna",
        "enabled=true mode=scatter biomes=[10,11] minSize=0.7 maxSize=1.2 density=0.4 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
    ),
    (
        "tree_wetland",
        "enabled=true mode=scatter biomes=[] minSize=0.7 maxSize=1.2 density=0.55 spacing=null elevMin=null elevMax=null requireWetland=true variantWeights=null",
    ),
    (
        "shrub",
        "enabled=true mode=scatter biomes=[7,8] minSize=0.7 maxSize=1.2 density=0.4 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
    ),
    (
        "cactus",
        "enabled=true mode=scatter biomes=[9] minSize=0.7 maxSize=1.2 density=0.35 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
    ),
    (
        "boulder",
        "enabled=true mode=scatter biomes=[2] minSize=0.7 maxSize=1.2 density=0.35 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
    ),
    // No preset: the bare default, byte for byte.
    ("unknown_slot", DEFAULT_RULE),
    ("custom::Trees::oak", DEFAULT_RULE),
];

#[test]
fn default_rule_matches_the_reference() {
    assert_eq!(canon(&ScatterRule::default()), DEFAULT_RULE);
}

#[test]
fn slot_presets_match_the_reference() {
    for (slot, expected) in PRESETS {
        assert_eq!(canon(&preset_scatter_rule(slot)), *expected, "slot {slot}");
    }
}

#[test]
fn rule_keys_match_the_reference() {
    // The harness called scatterRuleKey with, in order: one argument, an
    // explicit `undefined`, an explicit `null`, a real set name, an empty set
    // name (falsy -> no prefix), and a set name containing the separator.
    assert_eq!(scatter_rule_key("mountain", None), "mountain");
    assert_eq!(scatter_rule_key("oak", Some("Trees")), "custom::Trees::oak");
    assert_eq!(scatter_rule_key("oak", Some("")), "oak");
    assert_eq!(
        scatter_rule_key("oak", Some("My Set::x")),
        "custom::My Set::x::oak"
    );
}

// ============================================================================
// normalizeScatterRule — the untrusted-input boundary
// ============================================================================

/// `(name, raw JSON, slot key, expected)`. The raw values are exactly what the
/// harness fed the reference.
fn normalize_fixtures() -> Vec<(&'static str, Value, &'static str, &'static str)> {
    vec![
        (
            "null-input",
            json!(null),
            "mountain",
            "enabled=true mode=relief biomes=[] minSize=0.55 maxSize=1 density=1 spacing=null elevMin=0.58 elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "string-input",
            json!("not an object"),
            "mountain",
            "enabled=true mode=relief biomes=[] minSize=0.55 maxSize=1 density=1 spacing=null elevMin=0.58 elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "empty-object",
            json!({}),
            "tree_conifer",
            "enabled=true mode=scatter biomes=[3,4] minSize=0.7 maxSize=1.2 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "empty-object-unknown",
            json!({}),
            "custom::Trees::oak",
            DEFAULT_RULE,
        ),
        // `+'x'` is NaN -> rejected -> the literal 1, NOT shrub's own 0.4.
        (
            "density-garbage",
            json!({"density": "x"}),
            "shrub",
            "enabled=true mode=scatter biomes=[7,8] minSize=0.7 maxSize=1.2 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        // The v1.26 `+x||fallback` idiom lost this one: 0 is falsy.
        (
            "density-zero",
            json!({"density": 0}),
            "shrub",
            "enabled=true mode=scatter biomes=[7,8] minSize=0.7 maxSize=1.2 density=0 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "density-string-num",
            json!({"density": "2.5"}),
            "shrub",
            "enabled=true mode=scatter biomes=[7,8] minSize=0.7 maxSize=1.2 density=2.5 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "density-bool",
            json!({"density": true}),
            "shrub",
            "enabled=true mode=scatter biomes=[7,8] minSize=0.7 maxSize=1.2 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "density-hex-string",
            json!({"density": "0x2"}),
            "shrub",
            "enabled=true mode=scatter biomes=[7,8] minSize=0.7 maxSize=1.2 density=2 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "density-over",
            json!({"density": 5}),
            "shrub",
            "enabled=true mode=scatter biomes=[7,8] minSize=0.7 maxSize=1.2 density=3 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "density-negative",
            json!({"density": -1}),
            "shrub",
            "enabled=true mode=scatter biomes=[7,8] minSize=0.7 maxSize=1.2 density=0 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "density-null",
            json!({"density": null}),
            "tree_savanna",
            "enabled=true mode=scatter biomes=[10,11] minSize=0.7 maxSize=1.2 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "density-empty-string",
            json!({"density": ""}),
            "tree_savanna",
            "enabled=true mode=scatter biomes=[10,11] minSize=0.7 maxSize=1.2 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        // The v1.27 aliasing fix, from the reference's own probe: minSize must
        // fall back to the PRESET's 0.55, and maxSize must survive as 2.
        (
            "alias-minsize-garbage",
            json!({"minSize": "x", "maxSize": 2}),
            "mountain",
            "enabled=true mode=relief biomes=[] minSize=0.55 maxSize=2 density=1 spacing=null elevMin=0.58 elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "minsize-above-maxsize",
            json!({"minSize": 3, "maxSize": 1}),
            "mountain",
            "enabled=true mode=relief biomes=[] minSize=3 maxSize=3 density=1 spacing=null elevMin=0.58 elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "spacing-garbage",
            json!({"spacing": "NaN"}),
            "mountain",
            "enabled=true mode=relief biomes=[] minSize=0.55 maxSize=1 density=1 spacing=null elevMin=0.58 elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "spacing-zero",
            json!({"spacing": 0}),
            "mountain",
            "enabled=true mode=relief biomes=[] minSize=0.55 maxSize=1 density=1 spacing=1 elevMin=0.58 elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "spacing-huge",
            json!({"spacing": 1e9}),
            "mountain",
            "enabled=true mode=relief biomes=[] minSize=0.55 maxSize=1 density=1 spacing=512 elevMin=0.58 elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "spacing-null",
            json!({"spacing": null}),
            "mountain",
            "enabled=true mode=relief biomes=[] minSize=0.55 maxSize=1 density=1 spacing=null elevMin=0.58 elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "spacing-ok",
            json!({"spacing": 12.5}),
            "mountain",
            "enabled=true mode=relief biomes=[] minSize=0.55 maxSize=1 density=1 spacing=12.5 elevMin=0.58 elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "elev-inverted",
            json!({"elevMin": 0.9, "elevMax": 0.2}),
            "hill",
            "enabled=true mode=relief biomes=[] minSize=0.5 maxSize=1 density=1 spacing=null elevMin=0.9 elevMax=0.9 requireWetland=false variantWeights=null",
        ),
        (
            "elev-out-of-range",
            json!({"elevMin": -3, "elevMax": 9}),
            "hill",
            "enabled=true mode=relief biomes=[] minSize=0.5 maxSize=1 density=1 spacing=null elevMin=0 elevMax=1 requireWetland=false variantWeights=null",
        ),
        (
            "elev-garbage",
            json!({"elevMin": "q", "elevMax": "q"}),
            "hill",
            "enabled=true mode=relief biomes=[] minSize=0.5 maxSize=1 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "enabled-zero",
            json!({"enabled": 0}),
            "cactus",
            "enabled=false mode=scatter biomes=[9] minSize=0.7 maxSize=1.2 density=0.35 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        // JS truthiness, not a boolean check: a non-empty string is `true`.
        (
            "enabled-string",
            json!({"enabled": "no"}),
            "cactus",
            "enabled=true mode=scatter biomes=[9] minSize=0.7 maxSize=1.2 density=0.35 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "mode-relief",
            json!({"mode": "relief"}),
            "cactus",
            "enabled=true mode=relief biomes=[9] minSize=0.7 maxSize=1.2 density=0.35 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "mode-uppercase",
            json!({"mode": "RELIEF"}),
            "cactus",
            "enabled=true mode=scatter biomes=[9] minSize=0.7 maxSize=1.2 density=0.35 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "mode-number",
            json!({"mode": 7}),
            "cactus",
            "enabled=true mode=scatter biomes=[9] minSize=0.7 maxSize=1.2 density=0.35 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        // `Number.isFinite` does NOT coerce: "4" and null go, 5.5 and -2 stay.
        (
            "biomes-mixed",
            json!({"biomes": [3, "4", null, 5.5, -2]}),
            "boulder",
            "enabled=true mode=scatter biomes=[3,5.5,-2] minSize=0.7 maxSize=1.2 density=0.35 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        // A non-array `biomes` means "any land", not "keep the preset's [2]".
        (
            "biomes-not-array",
            json!({"biomes": 5}),
            "boulder",
            "enabled=true mode=scatter biomes=[] minSize=0.7 maxSize=1.2 density=0.35 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "biomes-empty",
            json!({"biomes": []}),
            "tree_conifer",
            DEFAULT_RULE,
        ),
        (
            "weights-mixed",
            json!({"variantWeights": [1, "2", null, -5, 1e9, "x"]}),
            "mountain",
            "enabled=true mode=relief biomes=[] minSize=0.55 maxSize=1 density=1 spacing=null elevMin=0.58 elevMax=null requireWetland=false variantWeights=[1,2,0,0,100,0]",
        ),
        (
            "weights-not-array",
            json!({"variantWeights": "abc"}),
            "mountain",
            "enabled=true mode=relief biomes=[] minSize=0.55 maxSize=1 density=1 spacing=null elevMin=0.58 elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "weights-empty",
            json!({"variantWeights": []}),
            "mountain",
            "enabled=true mode=relief biomes=[] minSize=0.55 maxSize=1 density=1 spacing=null elevMin=0.58 elevMax=null requireWetland=false variantWeights=[]",
        ),
        (
            "require-wetland-truthy",
            json!({"requireWetland": "yes"}),
            "shrub",
            "enabled=true mode=scatter biomes=[7,8] minSize=0.7 maxSize=1.2 density=0.4 spacing=null elevMin=null elevMax=null requireWetland=true variantWeights=null",
        ),
        (
            "unknown-keys-dropped",
            json!({"nope": 1, "density": 2}),
            "shrub",
            "enabled=true mode=scatter biomes=[7,8] minSize=0.7 maxSize=1.2 density=2 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null",
        ),
        (
            "full-override",
            json!({"enabled": false, "mode": "relief", "biomes": [1, 2], "minSize": 0.1, "maxSize": 4, "density": 2.25, "spacing": 9, "elevMin": 0.1, "elevMax": 0.8, "requireWetland": true, "variantWeights": [3, 1]}),
            "tree_conifer",
            "enabled=false mode=relief biomes=[1,2] minSize=0.1 maxSize=4 density=2.25 spacing=9 elevMin=0.1 elevMax=0.8 requireWetland=true variantWeights=[3,1]",
        ),
    ]
}

#[test]
fn normalize_matches_the_reference_on_every_hostile_input() {
    for (name, raw, slot, expected) in normalize_fixtures() {
        assert_eq!(
            canon(&normalize_scatter_rule(&raw, slot)),
            expected,
            "fixture {name}"
        );
    }
}

/// One deliberate divergence, asserted so it cannot drift into an accident:
/// `Object.assign` copies unknown keys straight through, so the reference's
/// returned rule still carries a `{"nope":1}`. The typed model drops it, the
/// same way `parse_pack_manifest` drops unknown manifest keys. Nothing in the
/// engine reads such a key; the only observable effect is that a rule
/// re-serialized to `library.json` loses it.
#[test]
fn unknown_rule_keys_are_dropped_rather_than_carried_through() {
    let r = normalize_scatter_rule(&json!({"nope": 1, "density": 2}), "shrub");
    let round_tripped = serde_json::to_string(&r).unwrap();
    assert!(!round_tripped.contains("nope"));
    assert_eq!(r.density, 2.0);
}

// ============================================================================
// pickIconVariant / pickWeightedVariant — deterministic, so an exact diff
// ============================================================================

/// The harness' sweep: `x = i*7 - 3`, `y = j*11 + 1` over a 6x6 grid, chosen
/// to include negative x (the `|0` / `as i32` path) and to spread the hash
/// lattice rather than walking one row of it.
fn sweep_positions() -> Vec<(i32, i32)> {
    (0..6)
        .flat_map(|y| (0..6).map(move |x| (x * 7 - 3, y * 11 + 1)))
        .collect()
}

#[test]
fn plain_variant_pick_matches_the_reference() {
    const EXPECTED: &[usize] = &[
        2, 3, 3, 0, 0, 1, 3, 1, 1, 1, 0, 3, 3, 1, 3, 1, 2, 1, 1, 3, 1, 3, 3, 3, 1, 0, 2, 1, 1, 0,
        0, 1, 1, 0, 2, 2,
    ];
    let got: Vec<usize> = sweep_positions()
        .into_iter()
        .map(|(x, y)| pick_icon_variant(x, y, 7, 4))
        .collect();
    assert_eq!(got, EXPECTED);
}

#[test]
fn weighted_variant_pick_matches_the_reference() {
    // The v1.25 hash, which the three degenerate cases below must reproduce
    // exactly — that fall-through is what keeps an un-weighted asset's variant
    // selection unchanged.
    const UNIFORM: &[usize] = &[
        2, 3, 3, 0, 0, 1, 3, 1, 1, 1, 0, 3, 3, 1, 3, 1, 2, 1, 1, 3, 1, 3, 3, 3, 1, 0, 2, 1, 1, 0,
        0, 1, 1, 0, 2, 2,
    ];
    #[allow(clippy::type_complexity)]
    let cases: &[(&str, usize, Option<&[f64]>, i32, &[usize])] = &[
        ("no weights", 4, None, 7, UNIFORM),
        ("single variant", 1, Some(&[5.0]), 7, &[0; 36]),
        ("zero variants", 0, None, 7, &[0; 36]),
        ("length mismatch", 4, Some(&[1.0, 2.0]), 7, UNIFORM),
        ("all zero", 4, Some(&[0.0, 0.0, 0.0, 0.0]), 7, UNIFORM),
        (
            "1:2:3:4",
            4,
            Some(&[1.0, 2.0, 3.0, 4.0]),
            7,
            &[
                2, 3, 3, 0, 0, 2, 3, 2, 2, 1, 1, 3, 3, 2, 3, 2, 3, 2, 2, 3, 2, 3, 3, 3, 2, 0, 3, 2,
                1, 1, 1, 2, 1, 1, 2, 2,
            ],
        ),
        (
            // Negative and NaN weights contribute nothing but do not make the
            // whole weighting degenerate — total is 1 + 4 here.
            "negative and NaN weights",
            4,
            Some(&[1.0, -5.0, f64::NAN, 4.0]),
            7,
            &[
                3, 3, 3, 0, 0, 3, 3, 3, 3, 3, 0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0, 3, 3,
                3, 3, 0, 3, 3, 0, 3, 3,
            ],
        ),
        ("one-hot", 3, Some(&[0.0, 1.0, 0.0]), 7, &[1; 36]),
        (
            // Weights so small their sum is still > 0: must behave as uniform
            // thirds, not fall through.
            "uniform tiny weights",
            3,
            Some(&[1e-9, 1e-9, 1e-9]),
            11,
            &[
                2, 1, 0, 1, 2, 0, 2, 0, 0, 0, 2, 1, 2, 2, 2, 1, 1, 2, 0, 1, 2, 2, 1, 0, 1, 2, 0, 2,
                1, 2, 2, 0, 1, 2, 1, 1,
            ],
        ),
        (
            "7:3 at seed 0",
            2,
            Some(&[7.0, 3.0]),
            0,
            &[
                0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1,
                1, 0, 0, 1, 0, 1, 0, 0,
            ],
        ),
    ];
    for (name, n, weights, seed, expected) in cases {
        let got: Vec<usize> = sweep_positions()
            .into_iter()
            .map(|(x, y)| pick_weighted_variant(x, y, *seed, *n, *weights))
            .collect();
        assert_eq!(got, *expected, "case {name}");
    }
}

// ============================================================================
// currentScatterRules / autopopulateScatterRules
// ============================================================================

#[test]
fn current_scatter_rules_matches_the_reference() {
    let empty = ScatterRuleTable::new();
    assert!(current_scatter_rules(&empty).is_none(), "empty table");

    let mut all_off = ScatterRuleTable::new();
    all_off.insert(
        "a",
        ScatterRule {
            enabled: false,
            ..Default::default()
        },
    );
    assert!(current_scatter_rules(&all_off).is_none(), "all disabled");

    // Insertion order deliberately not alphabetical: the reference iterates
    // the object, and JavaScript iterates string keys in insertion order.
    let mut mixed = ScatterRuleTable::new();
    mixed.insert("zebra", preset_scatter_rule("mountain"));
    mixed.insert(
        "apple",
        ScatterRule {
            enabled: false,
            ..Default::default()
        },
    );
    mixed.insert("mango", preset_scatter_rule("cactus"));
    assert_eq!(
        canon_table(current_scatter_rules(&mixed).unwrap()),
        "zebra => enabled=true mode=relief biomes=[] minSize=0.55 maxSize=1 density=1 spacing=null elevMin=0.58 elevMax=null requireWetland=false variantWeights=null ; mango => enabled=true mode=scatter biomes=[9] minSize=0.7 maxSize=1.2 density=0.35 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null"
    );
}

/// `(slot, variants)` pairs, as the icons and custom sections hold them.
type Slots<'a> = &'a [(&'a str, &'a [&'a str])];

fn pack_with(icons: Slots<'_>, custom: &[(&str, Slots<'_>)]) -> PackManifest {
    let mut p = PackManifest::default();
    for (slot, variants) in icons {
        p.icons
            .insert(*slot, variants.iter().map(|s| s.to_string()).collect());
    }
    for (set, slots) in custom {
        let mut m = cartalith_assets::OrderedMap::new();
        for (slot, variants) in *slots {
            m.insert(*slot, variants.iter().map(|s| s.to_string()).collect());
        }
        p.custom.insert(*set, m);
    }
    p
}

#[test]
fn autopopulate_matches_the_reference() {
    // A pack with: a normal slot, a multi-variant slot, an EMPTY icon slot
    // (skipped), a slot with no preset (bare default), and two custom sets —
    // one of which has an empty slot, which the reference does NOT skip.
    let pack = pack_with(
        &[
            ("tree_conifer", &["a.png"]),
            ("mountain", &["m1.png", "m2.png"]),
            ("shrub", &[]),
            ("unknown_slot", &["u.png"]),
        ],
        &[
            ("Trees", &[("oak", &["o.png"]), ("elm", &[])]),
            ("My Props", &[("barrel", &["b.png"])]),
        ],
    );
    let mut table = ScatterRuleTable::new();
    autopopulate_scatter_rules(&mut table, &pack);
    assert_eq!(
        canon_table(table.iter()),
        "tree_conifer => enabled=true mode=scatter biomes=[3,4] minSize=0.7 maxSize=1.2 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null ; mountain => enabled=true mode=relief biomes=[] minSize=0.55 maxSize=1 density=1 spacing=null elevMin=0.58 elevMax=null requireWetland=false variantWeights=null ; unknown_slot => enabled=true mode=scatter biomes=[] minSize=0.7 maxSize=1.2 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null ; custom::Trees::oak => enabled=false mode=scatter biomes=[] minSize=0.7 maxSize=1.2 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null ; custom::Trees::elm => enabled=false mode=scatter biomes=[] minSize=0.7 maxSize=1.2 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null ; custom::My Props::barrel => enabled=false mode=scatter biomes=[] minSize=0.7 maxSize=1.2 density=1 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null"
    );
}

#[test]
fn autopopulate_never_clobbers_a_tuned_rule() {
    let mut table = ScatterRuleTable::new();
    table.insert(
        "mountain",
        ScatterRule {
            density: 0.11,
            ..Default::default()
        },
    );
    autopopulate_scatter_rules(
        &mut table,
        &pack_with(&[("mountain", &["m.png"]), ("hill", &["h.png"])], &[]),
    );
    assert_eq!(
        canon_table(table.iter()),
        "mountain => enabled=true mode=scatter biomes=[] minSize=0.7 maxSize=1.2 density=0.11 spacing=null elevMin=null elevMax=null requireWetland=false variantWeights=null ; hill => enabled=true mode=relief biomes=[] minSize=0.5 maxSize=1 density=1 spacing=null elevMin=0.53 elevMax=0.58 requireWetland=false variantWeights=null"
    );
}

#[test]
fn autopopulate_on_an_artless_pack_adds_nothing() {
    let mut table = ScatterRuleTable::new();
    autopopulate_scatter_rules(&mut table, &PackManifest::default());
    assert!(table.is_empty());
}
