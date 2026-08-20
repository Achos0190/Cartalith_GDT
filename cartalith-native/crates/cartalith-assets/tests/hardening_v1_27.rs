//! The three v1.27 hardening fixes, each named and each pinned by a test.
//!
//! These are not incidental implementation detail. Scatter rules arrive from
//! `assetlib/library.json` **inside a user-supplied project `.zip`**, so every
//! field reaching [`normalize_scatter_rule`] is untrusted input. Reference
//! v1.26 merged that input with the `+x||fallback` idiom, which mishandled two
//! whole classes of value: a legitimate `0` fell through to the default (`0`
//! is falsy in JavaScript), and a non-numeric value produced a `NaN` that then
//! *propagated* into the placement engine instead of being rejected at the
//! boundary. v1.27 replaced it with a `num()` helper that clamps into range
//! and falls back only on genuinely non-finite input.
//!
//! Three concrete failures are called out in the reference's own comments
//! (lines 6981-6986 and 7209-7213). Each gets a test below, and — because a
//! Rust port has different natural failure modes than JavaScript — each is
//! re-derived here rather than transcribed:
//!
//! | # | The v1.26 failure | In Rust |
//! |---|---|---|
//! | 1 | `NaN` density scattered on **every** cell | still reachable, by the opposite IEEE rule |
//! | 2 | `NaN` spacing collapsed an O(1) neighbour test to O(n²) | reachable; `f64::max` would mask it by accident |
//! | 3 | `Object.assign` aliasing made every fallback read its own garbage | **structurally unreachable** — see below |
//!
//! Tests 1 and 2 reproduce the *downstream* arithmetic inline (four lines
//! each) rather than depending on the placement engine, which is milestone 4.
//! That keeps the demonstration honest — it is the real predicate out of
//! `placeMapIconsRuled`, not a paraphrase — without dragging an unwritten
//! module into a milestone-3 test.

use cartalith_assets::{ScatterRule, normalize_scatter_rule, preset_scatter_rule};
use serde_json::json;

// ============================================================================
// Fix 1 — a NaN `density` scattered on every cell
// ============================================================================

/// `placeMapIconsRuled`'s scatter predicate, verbatim from reference line
/// 7275: a jittered grid cell is **rejected** when `keep >= min(1, density)`,
/// where `keep` is a position hash in `[0, 1]`. Returns whether an icon lands.
// Kept as the negated comparison the reference actually writes: this test is
// *about* what that comparison does against a NaN threshold, so rewriting it
// into a NaN-explicit form would erase the thing being demonstrated.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn scatter_accepts(keep: f64, density: f64) -> bool {
    !(keep >= f64::min(1.0, density))
}

/// **v1.27 fix #1.** A rule whose `density` is not a number must never reach
/// the placement engine as `NaN`, because the predicate above cannot reject
/// anything against a `NaN` threshold — the corrupt rule then places an icon
/// on *every* jittered grid cell it visits, carpeting the map.
///
/// Worth noting because it is a genuine Rust-vs-JS difference and not a
/// transcription: the JS failure runs through `Math.min(1, NaN) === NaN`, so
/// `keep >= NaN` is false. Rust's `f64::min` does the **opposite** — it
/// absorbs NaN, so `f64::min(1.0, NAN)` is `1.0`. The bug survives the
/// translation anyway, because `keep` is a hash in `[0, 1]` and `keep >= 1.0`
/// is false for every value the hash can practically produce. Same
/// catastrophe, opposite IEEE rule; the fix is what makes both mechanisms
/// unreachable.
#[test]
fn fix1_a_non_numeric_density_cannot_reach_the_engine_as_nan() {
    // First, the failure this prevents, demonstrated on the real predicate.
    assert!(
        scatter_accepts(0.99, f64::NAN),
        "a NaN density accepts a cell it should have rejected"
    );
    assert!(
        (0..1000).all(|i| scatter_accepts(i as f64 / 1000.0, f64::NAN)),
        "a NaN density accepts EVERY cell -- this is the carpeted map"
    );

    // Now the fix: every non-numeric spelling normalizes to a usable density.
    for garbage in [json!("x"), json!("NaN"), json!({}), json!([]), json!(null)] {
        let r = normalize_scatter_rule(&json!({ "density": garbage }), "shrub");
        assert!(
            r.density.is_finite(),
            "density from {garbage} is not finite"
        );
        assert!((0.0..=3.0).contains(&r.density));
    }
    // And the predicate is a real predicate again: a tuned density both
    // accepts and rejects, which a NaN threshold could never do.
    let tuned = normalize_scatter_rule(&json!({"density": 0.25}), "shrub");
    assert!(!scatter_accepts(0.99, tuned.density));
    assert!(scatter_accepts(0.1, tuned.density));

    // The other half of the same v1.26 idiom: `+0||fallback` lost a real zero.
    // A density the user deliberately set to 0 must stay 0, not silently
    // become 1 (which in scatter mode means "place on every cell").
    let zero = normalize_scatter_rule(&json!({"density": 0}), "shrub");
    assert_eq!(zero.density, 0.0);
    assert!(
        !scatter_accepts(0.0, zero.density),
        "density 0 must place nothing"
    );
}

// ============================================================================
// Fix 2 — a NaN `spacing` collapsed an O(1) neighbour test to O(n²)
// ============================================================================

/// `placeMapIconsRuled`'s bucket grid, from reference line 7223:
/// `bw = Math.ceil(W/cell)||1`. Relief icons are hashed into a `bw x bh` grid
/// so the "is anything within `space` of here?" test only ever scans nine
/// buckets. `NaN` is falsy in JavaScript, so a `NaN` cell size takes the
/// `||1` branch and yields a **1x1** grid — every icon in one bucket, which is
/// the collapse this fix prevents.
fn bucket_grid(map_width: usize, map_height: usize, cell: f64) -> (usize, usize) {
    let dim = |n: usize| {
        let v = (n as f64 / cell).ceil();
        if v.is_finite() && v != 0.0 {
            v as usize
        } else {
            1
        }
    };
    (dim(map_width), dim(map_height))
}

/// **v1.27 fix #2.** A `NaN` spacing makes the bucket grid degenerate: every
/// `(x/cell)|0` is `NaN|0 == 0`, so all icons land in one bucket and the
/// nine-bucket neighbour scan becomes a scan over every icon placed so far —
/// an O(1) test silently turned O(n²), on a map that can place thousands of
/// peaks.
///
/// The fix has two halves and this port keeps both, one per assertion below:
/// [`normalize_scatter_rule`] rejects a non-finite `spacing` at the boundary,
/// and [`ScatterRule::spacing_cells`] guards the **computed** value so a rule
/// that reached the engine without being normalized (a direct caller, a unit
/// test) still yields a finite spacing.
///
/// Rust note: `f64::max` absorbs NaN, so `f64::max(3.0, NAN) == 3.0` would
/// rescue `spacing_cells` *by accident*. The explicit `is_finite` check is
/// kept anyway — an implicit dependency on an IEEE corner is exactly what this
/// fix existed to remove, and `f64::min`'s behaviour in fix #1 above shows how
/// little that intuition can be trusted.
#[test]
fn fix2_a_non_finite_spacing_cannot_collapse_the_bucket_grid() {
    // The failure this prevents: 5400 buckets become 1, so `fits()` stops
    // being a nine-bucket lookup and becomes a scan of every placed icon.
    assert_eq!(bucket_grid(900, 600, 10.0), (90, 60));
    assert_eq!(bucket_grid(900, 600, f64::NAN), (1, 1));

    // Half one: normalization rejects garbage, and clamps a real value into
    // [1, 512] so it can neither be zero (division by zero) nor so large the
    // grid is a single bucket by arithmetic rather than by accident.
    for garbage in [json!("NaN"), json!("x"), json!(""), json!(null), json!([])] {
        let r = normalize_scatter_rule(&json!({ "spacing": garbage }), "mountain");
        assert_eq!(r.spacing, None, "spacing from {garbage} should be unset");
    }
    assert_eq!(
        normalize_scatter_rule(&json!({"spacing": 0}), "mountain").spacing,
        Some(1.0)
    );
    assert_eq!(
        normalize_scatter_rule(&json!({"spacing": 1e9}), "mountain").spacing,
        Some(512.0)
    );

    // Half two: the engine-side guard, on a rule built by hand rather than
    // normalized -- the case the reference's own comment calls out.
    for hostile in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -50.0] {
        let r = ScatterRule {
            spacing: Some(hostile),
            ..Default::default()
        };
        let cell = r.spacing_cells(900);
        assert!(
            cell.is_finite() && cell >= 3.0,
            "spacing_cells({hostile}) == {cell}"
        );
        assert_ne!(bucket_grid(900, 600, cell), (1, 1), "grid collapsed");
    }
    // Same for a hand-built rule with a garbage density, which feeds the
    // derived-spacing branch.
    let r = ScatterRule {
        density: f64::NAN,
        ..Default::default()
    };
    assert!(r.spacing_cells(900).is_finite());
}

// ============================================================================
// Fix 3 — the Object.assign aliasing bug
// ============================================================================

/// **v1.27 fix #3, which is structurally unreachable in this port** — verified
/// rather than assumed, and so ported as an assertion about the outcome rather
/// than as defensive code.
///
/// The JavaScript bug: `const out = Object.assign(base, r)` **mutates** `base`
/// and returns it, so `out` and `base` alias. Every later line of the form
/// `out.minSize = num(out.minSize, lo, hi, base.minSize)` then fell back to
/// the very garbage it was rejecting — a `minSize: 'x'` "defaulted" to `'x'`,
/// and `maxSize` went `NaN` through `Math.max('x', …)`. v1.27 fixed it by
/// copying into a fresh object: `Object.assign({}, base, r)`.
///
/// Why it cannot happen here, and it is **not** simply "Rust's ownership
/// rules": the bug requires the defaults and the untrusted input to inhabit
/// one mutable object. In this port they are different *types* — `base` is an
/// owned [`ScatterRule`] whose `min_size` is an `f64`, and the input is a
/// [`serde_json::Value`]. There is no merge-in-place operation to get wrong,
/// because a `"x"` can never be stored in the field it would have to corrupt.
/// No defensive code is written for it; this test pins the observable
/// behaviour so a future refactor toward a "merge" helper would fail loudly.
#[test]
fn fix3_a_rejected_field_falls_back_to_the_preset_not_to_itself() {
    // The reference's own v1.27 probe case.
    let r = normalize_scatter_rule(&json!({"minSize": "x", "maxSize": 2}), "mountain");
    let preset = preset_scatter_rule("mountain");
    assert_eq!(r.min_size, preset.min_size, "must fall back to the PRESET");
    assert_eq!(r.min_size, 0.55);
    assert_eq!(
        r.max_size, 2.0,
        "maxSize must survive its neighbour's rejection"
    );

    // The preset itself must be untouched by the call -- the aliasing bug's
    // other symptom was that `base` came back mutated.
    assert_eq!(preset_scatter_rule("mountain"), preset);

    // Generalised: no single garbage field may poison any other, and no field
    // may end up non-finite, whatever the input.
    let hostile = json!({
        "enabled": "x", "mode": "x", "biomes": "x", "minSize": "x", "maxSize": "x",
        "density": "x", "spacing": "x", "elevMin": "x", "elevMax": "x",
        "requireWetland": "x", "variantWeights": "x",
    });
    for slot in [
        "mountain",
        "hill",
        "tree_wetland",
        "cactus",
        "custom::Set::a",
    ] {
        let r = normalize_scatter_rule(&hostile, slot);
        assert!(r.min_size.is_finite() && r.max_size.is_finite() && r.density.is_finite());
        assert!(r.min_size <= r.max_size);
        assert!(r.spacing.is_none_or(f64::is_finite));
        assert!(r.elev_min.is_none_or(f64::is_finite));
        assert!(r.elev_max.is_none_or(f64::is_finite));
        assert!(r.biomes.iter().all(|b| b.is_finite()));
        assert!(
            r.variant_weights
                .as_deref()
                .is_none_or(|w| w.iter().all(|x| x.is_finite()))
        );
    }
}

/// The one hardening guarantee this port has and the reference cannot: there
/// is **no** `Deserialize` for [`ScatterRule`], so a rule cannot be conjured
/// straight out of a project file — [`normalize_scatter_rule`] is the only
/// door in, and the three fixes above are therefore not bypassable by a future
/// caller reaching for `serde_json::from_str`.
///
/// This is a compile-time property, so the "test" is the commented snippet:
/// uncommenting it must fail to compile. The runtime half checks the other
/// direction — a rule still serializes with the reference's field names, which
/// is what milestone 5's `library.json` round trip needs.
#[test]
fn rules_serialize_but_deliberately_do_not_deserialize() {
    // let _: ScatterRule = serde_json::from_str("{}").unwrap();  // must not compile
    let json = serde_json::to_string(&preset_scatter_rule("tree_wetland")).unwrap();
    assert!(json.contains(r#""requireWetland":true"#));
    assert!(json.contains(r#""density":0.55"#));
}
