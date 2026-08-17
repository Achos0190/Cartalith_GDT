//! Scatter rules — the data that decides *where* an asset gets scattered.
//!
//! Ported from `Cartalith Gen1 v2.10.html` lines 6919-7039 and 7088-7101
//! (`SCATTER_MODES`, `defaultScatterRule`, `scatterRuleKey`,
//! `SCATTER_RULE_PRESETS`, `presetScatterRule`, `normalizeScatterRule`,
//! `pickWeightedVariant`, `currentScatterRules`, `autopopulateScatterRules`),
//! plus `pickIconVariant` (line 12171), which `pickWeightedVariant` falls
//! through to.
//!
//! # What a scatter rule is for
//!
//! Until reference v1.25 the biome→asset mapping was **hard-coded** inside
//! `placeMapIcons` (`bi===3||bi===4 ⇒ conifer`), so an asset's placement
//! behaviour was fixed by *which* of the ten [`PACK_ICON_SLOTS`] it happened
//! to occupy: nothing was tunable and no asset outside that frozen list could
//! ever scatter. A [`ScatterRule`] makes that mapping **data**, so an
//! arbitrary custom-set asset can join in and the mapping can be edited.
//!
//! [`SCATTER_RULE_PRESETS`](preset_scatter_rule) reproduce v1.25's hard-coded
//! behaviour exactly, which is what lets a user who never touches a rule see
//! the map they always did. The table is a generalisation of the old switch,
//! not a new look.
//!
//! `biomes` holds **`BIOME_INDEX` values** (ocean 0, ice 1, tundra 2, boreal
//! 3, conifer 4, tempForest 5, tempRain 6, grass 7, shrub 8, desert 9,
//! savanna 10, tropDry 11, tropWet 12, lake 13) — the frozen *climate*
//! vocabulary `buildBiomeRaster` emits, deliberately **not** `CART_BIOMES`
//! (that raster only exists where the user has painted, so rules keyed to it
//! would silently scatter nothing on a fresh map). An empty `biomes` means
//! "any land cell".
//!
//! # The v1.27 hardening, and what it means in Rust
//!
//! Rules are loaded out of `assetlib/library.json` inside a **user-supplied
//! project `.zip`**, so every field arriving at [`normalize_scatter_rule`] is
//! untrusted input. Reference v1.26 merged it with `+x||fallback`, which
//! mishandled two whole classes of value — a legitimate `0` fell through to
//! the default (`0` is falsy), and a non-numeric value produced `NaN` that
//! then *propagated* rather than being rejected. v1.27 replaced that with a
//! `num()` helper that clamps, and falls back only on genuinely non-finite
//! input. Three concrete failures are named in the reference's own comments;
//! each is reproduced as a test in `tests/hardening_v1_27.rs`, and each
//! translates differently here:
//!
//! 1. **A `NaN` density scattered on every cell.** *Still a real hazard in
//!    Rust, by a different mechanism.* In JS the scatter predicate is
//!    `keep >= Math.min(1, rule.density)`, and `Math.min(1, NaN)` is `NaN`,
//!    so the comparison is false for every cell and the rule is never
//!    rejected. Rust's `f64::min` does the opposite — it *absorbs* NaN, so
//!    `1.0f64.min(f64::NAN)` is `1.0` — but `keep` is a hash in `[0, 1]`, so
//!    `keep >= 1.0` is still false essentially everywhere and the corrupt
//!    rule still carpets the map. Same catastrophe, opposite IEEE rule. The
//!    fix — reject non-finite input at the boundary — is what makes both
//!    mechanisms unreachable.
//! 2. **A `NaN` spacing collapsed an O(1) neighbour test to O(n²).**
//!    `placeMapIconsRuled` buckets placed icons on a grid of `cell` = the
//!    largest rule spacing; a `NaN` cell makes every bucket index `NaN` and
//!    the nine-bucket neighbour scan degenerates into a scan over every icon
//!    already placed. Rust's NaN-absorbing `f64::max` would *accidentally*
//!    rescue [`ScatterRule::spacing_cells`] here — which is exactly the kind
//!    of implicit dependency the fix existed to remove, so the finite check
//!    is written out explicitly rather than left to `max`'s semantics.
//! 3. **An `Object.assign` aliasing bug made every fallback read the garbage
//!    it was meant to replace.** *Structurally unreachable in Rust, and not
//!    for the reason one might guess.* The JS bug needed the defaults and the
//!    untrusted input to inhabit **one mutable object**: `Object.assign(base,
//!    r)` mutated `base` and returned it, so `out` and `base` aliased and
//!    `num(out.minSize, …, base.minSize)` fell back to the very `'x'` it was
//!    rejecting. Here `base` is an owned [`ScatterRule`] and the input is a
//!    [`serde_json::Value`]: no merge-in-place is even *expressible*, because
//!    garbage cannot be stored in a `f64` field to begin with. The test that
//!    names this fix therefore asserts the observable outcome rather than
//!    guarding a bug that cannot occur.
//!
//! Rust adds a fourth guarantee the reference cannot have: [`ScatterRule`]
//! deliberately implements [`Serialize`] but **not** `Deserialize`. There is
//! no way to conjure one straight out of a project file; untrusted JSON can
//! only become a rule by passing through [`normalize_scatter_rule`], so the
//! hardening cannot be bypassed by a future caller reaching for
//! `serde_json::from_str`.
//!
//! [`PACK_ICON_SLOTS`]: crate::PACK_ICON_SLOTS

use crate::manifest::PackManifest;
use crate::ordered_map::OrderedMap;
use cartalith_noise::hash;
use serde::Serialize;
use serde_json::Value;

/// How a rule places its asset. The reference's `SCATTER_MODES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScatterMode {
    /// Jittered-grid vegetation and ground cover: one visit per grid cell,
    /// kept with probability `density`.
    #[default]
    Scatter,
    /// Elevation-ranked, spacing-rejected landforms (mountains, hills):
    /// highest cells first, each accepted only if no placed icon is within
    /// [`ScatterRule::spacing_cells`].
    Relief,
}

impl ScatterMode {
    /// The reference's `SCATTER_MODES` array, in its order.
    pub const ALL: [ScatterMode; 2] = [ScatterMode::Scatter, ScatterMode::Relief];

    /// The wire spelling used in `pack.json` / `library.json`.
    pub fn as_str(self) -> &'static str {
        match self {
            ScatterMode::Scatter => "scatter",
            ScatterMode::Relief => "relief",
        }
    }

    /// Parse a wire spelling. Case-sensitive and total, matching
    /// `SCATTER_MODES.includes(...)`: anything else is not a mode.
    pub fn from_wire(s: &str) -> Option<ScatterMode> {
        ScatterMode::ALL.into_iter().find(|m| m.as_str() == s)
    }
}

/// Per-asset scattering control — the reference's `ScatterRule` object.
///
/// [`Default`] is the reference's `defaultScatterRule()`. Every field is
/// guaranteed finite and in range on any value this module produces; see the
/// module docs for why that guarantee is load-bearing.
///
/// **No `Deserialize`, deliberately** — see the module docs. Build one from
/// untrusted input with [`normalize_scatter_rule`].
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScatterRule {
    /// Whether this asset participates in procedural scatter at all.
    pub enabled: bool,
    /// Which placement engine to use.
    pub mode: ScatterMode,
    /// `BIOME_INDEX` values this rule accepts; empty = any land cell.
    ///
    /// `f64`, not `i32`, on purpose: the reference filters with
    /// `Number.isFinite`, so a hand-edited `5.5` is **kept** and simply never
    /// matches an integer biome index. Truncating it to `5` here would make
    /// it start matching, and round-tripping the rule back out to
    /// `library.json` would silently rewrite the author's file.
    pub biomes: Vec<f64>,
    /// Lower bound of the per-instance size multiplier (uniform in range).
    pub min_size: f64,
    /// Upper bound of the per-instance size multiplier. Never below
    /// `min_size`.
    pub max_size: f64,
    /// `0..3`. Above 1 packs tighter: in `Relief` mode a smaller derived
    /// spacing, in `Scatter` mode a higher keep-probability.
    pub density: f64,
    /// Explicit minimum separation in **grid cells**; `None` derives it from
    /// `density` (see [`ScatterRule::spacing_cells`]).
    pub spacing: Option<f64>,
    /// Land-relative elevation band `[0,1]`; `None` is unbounded. `Relief`
    /// mode only.
    pub elev_min: Option<f64>,
    /// Upper end of the elevation band; never below `elev_min`.
    pub elev_max: Option<f64>,
    /// Place only on cells flagged by the wetland mask. **ANDed** with the
    /// biome test in both modes since v1.27 (in v1.26's scatter path it
    /// wrongly *replaced* it).
    pub require_wetland: bool,
    /// Per-variant relative weights; `None` is uniform, matching
    /// [`pick_icon_variant`].
    pub variant_weights: Option<Vec<f64>>,
}

impl Default for ScatterRule {
    /// The reference's `defaultScatterRule()`.
    fn default() -> Self {
        ScatterRule {
            enabled: true,
            mode: ScatterMode::Scatter,
            biomes: Vec::new(),
            min_size: 0.7,
            max_size: 1.2,
            density: 1.0,
            spacing: None,
            elev_min: None,
            elev_max: None,
            require_wetland: false,
            variant_weights: None,
        }
    }
}

impl ScatterRule {
    /// Minimum separation in grid cells for a `Relief` rule — the reference's
    /// `spaceOf` (line 7215), lifted out of `placeMapIconsRuled` so the rule
    /// model owns the whole of v1.27's spacing hardening rather than half of
    /// it. `map_width` is the grid width in cells.
    ///
    /// Two reference quirks are reproduced rather than tidied:
    ///
    /// - The density fallback is `+r.density || 1`, so a density of **exactly
    ///   zero** derives spacing as if it were 1 (`0` is falsy in JS). Since
    ///   density is only a spacing input in relief mode, "0" there has never
    ///   meant "place nothing"; changing it would move mountains on existing
    ///   maps.
    /// - The floor of 3 cells stops a runaway density from degenerating into
    ///   a solid mat of overlapping sprites.
    ///
    /// The `is_finite` guard is v1.27 fix #2's engine-side half: it applies
    /// to the *computed* value, so a rule that reached the engine without
    /// passing through [`normalize_scatter_rule`] (a direct caller, a unit
    /// test) still yields a finite spacing.
    pub fn spacing_cells(&self, map_width: usize) -> f64 {
        let raw = match self.spacing {
            Some(s) => s,
            None => {
                // `+r.density||1`: zero and non-finite both fall back to 1.
                let d = if self.density.is_finite() && self.density != 0.0 {
                    self.density
                } else {
                    1.0
                };
                (f64::max(5.0, map_width as f64 / 90.0) / f64::max(0.15, d)).round()
            }
        };
        f64::max(3.0, if raw.is_finite() { raw } else { 3.0 })
    }
}

/// Canonical rule key for an asset — the reference's `scatterRuleKey`.
///
/// A frozen feature slot addresses itself (`"mountain"`); a user-defined
/// custom-set asset becomes `"custom::<setName>::<slotId>"`. This is the
/// single place that spelling is decided, so the scatterer, the renderer, the
/// Library bridge and the icon brush can never drift apart on it.
///
/// An empty set name is treated as absent, matching JavaScript's falsy test.
pub fn scatter_rule_key(slot: &str, set_name: Option<&str>) -> String {
    match set_name {
        Some(set) if !set.is_empty() => format!("custom::{set}::{slot}"),
        _ => slot.to_string(),
    }
}

/// The defaults for a slot — the reference's `presetScatterRule`, i.e.
/// `defaultScatterRule()` overlaid with `SCATTER_RULE_PRESETS[slotKey]`.
///
/// The presets exist for exactly the ten [`PACK_ICON_SLOTS`](crate::PACK_ICON_SLOTS);
/// every other key (including every `custom::…` asset) gets the bare default.
/// Their elevation bands and densities mirror `placeMapIcons`' own
/// `mountainTh`/`hillTh` constants, which is what makes an un-tuned pack look
/// like v1.25.
pub fn preset_scatter_rule(slot_key: &str) -> ScatterRule {
    let mut r = ScatterRule::default();
    match slot_key {
        "mountain" => {
            r.mode = ScatterMode::Relief;
            r.elev_min = Some(0.58);
            r.min_size = 0.55;
            r.max_size = 1.00;
        }
        "hill" => {
            r.mode = ScatterMode::Relief;
            r.elev_min = Some(0.53);
            r.elev_max = Some(0.58);
            r.min_size = 0.50;
            r.max_size = 1.00;
        }
        "tree_conifer" => r.biomes = vec![3.0, 4.0],
        "tree_broadleaf" => r.biomes = vec![5.0],
        "tree_rainforest" => r.biomes = vec![6.0, 12.0],
        "tree_savanna" => {
            r.biomes = vec![10.0, 11.0];
            r.density = 0.4;
        }
        "tree_wetland" => {
            r.require_wetland = true;
            r.density = 0.55;
        }
        "shrub" => {
            r.biomes = vec![7.0, 8.0];
            r.density = 0.4;
        }
        "cactus" => {
            r.biomes = vec![9.0];
            r.density = 0.35;
        }
        "boulder" => {
            r.biomes = vec![2.0];
            r.density = 0.35;
        }
        _ => {}
    }
    r
}

// ---------------------------------------------------------------------------
// JavaScript value coercion — the untrusted-input boundary
// ---------------------------------------------------------------------------

/// JavaScript truthiness (`!!v`).
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0 && !f.is_nan()),
        Value::String(s) => !s.is_empty(),
        // Arrays and objects are always truthy in JS, `[]` and `{}` included.
        _ => true,
    }
}

/// JavaScript's unary `+` on the value kinds a JSON document can hold.
///
/// Deliberate divergence, documented rather than reproduced: JS coerces an
/// array through `ToPrimitive` (`+[2] === 2`, `+[] === 0`). Here an array or
/// object yields `NaN`, so a numeric rule field spelled as an array falls back
/// to its preset default. That is a *safer* outcome than the reference's, it
/// has no plausible real input behind it, and it is asserted by a test so the
/// difference stays deliberate.
fn js_number(v: &Value) -> f64 {
    match v {
        Value::Null => 0.0,
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        Value::String(s) => {
            let t = s.trim();
            if t.is_empty() {
                // `+"   "` is 0 in JS; `+""` never reaches here (see `num`).
                return 0.0;
            }
            // JS accepts the ES2015 radix literal forms in string coercion.
            let radix = t
                .strip_prefix("0x")
                .or_else(|| t.strip_prefix("0X"))
                .map(|d| (d, 16))
                .or_else(|| {
                    t.strip_prefix("0o")
                        .or_else(|| t.strip_prefix("0O"))
                        .map(|d| (d, 8))
                })
                .or_else(|| {
                    t.strip_prefix("0b")
                        .or_else(|| t.strip_prefix("0B"))
                        .map(|d| (d, 2))
                });
            match radix {
                Some((digits, base)) => u64::from_str_radix(digits, base)
                    .map(|n| n as f64)
                    .unwrap_or(f64::NAN),
                None => t.parse::<f64>().unwrap_or(f64::NAN),
            }
        }
        _ => f64::NAN,
    }
}

/// The reference's `num(v, lo, hi, dflt)`, split so the `null`-defaulting
/// fields can express "no value" as `None`.
///
/// **This is v1.27 fix #1 and #2's shared implementation.** Missing, empty and
/// non-numeric input all become `None` — never a `NaN` that propagates into
/// the placement engine — and everything else is clamped into `[lo, hi]`.
fn num_opt(v: Option<&Value>, lo: f64, hi: f64) -> Option<f64> {
    let n = match v {
        // `v == null` catches both `undefined` and `null`; `v === ''` is a
        // separate, deliberate reject (an empty field means "unset", whereas
        // a whitespace-only string coerces to 0 in JS and is left to do so).
        None | Some(Value::Null) => return None,
        Some(Value::String(s)) if s.is_empty() => return None,
        Some(other) => js_number(other),
    };
    n.is_finite().then(|| n.clamp(lo, hi))
}

/// `num` with a non-null default.
fn num(v: Option<&Value>, lo: f64, hi: f64, dflt: f64) -> f64 {
    num_opt(v, lo, hi).unwrap_or(dflt)
}

/// Merge an untrusted, partial or legacy rule record onto a slot's preset —
/// the reference's `normalizeScatterRule`.
///
/// `raw` is the value straight out of a project file. Anything that is not a
/// JSON object (including an array, which JS's `typeof` calls an object but
/// which carries no matching keys) yields the preset unchanged. Every field
/// is independently optional, so an old save or a hand-edited manifest can
/// omit any of them.
///
/// This is the **only** way to build a [`ScatterRule`] from untrusted input,
/// and it cannot produce a non-finite field. See the module docs for the three
/// v1.27 failures it exists to prevent.
///
/// One deliberate divergence: JavaScript's `Object.assign` carries unknown
/// keys through into the returned rule (a `{"nope":1}` survives a round trip).
/// The typed model drops them, the same way [`crate::parse_pack_manifest`]
/// drops unknown manifest keys.
pub fn normalize_scatter_rule(raw: &Value, slot_key: &str) -> ScatterRule {
    let base = preset_scatter_rule(slot_key);
    let Some(obj) = raw.as_object() else {
        return base;
    };
    let g = |k: &str| obj.get(k);

    let min_size = num(g("minSize"), 0.05, 8.0, base.min_size);
    let mut out = ScatterRule {
        enabled: g("enabled").map_or(base.enabled, truthy),
        mode: match g("mode") {
            Some(v) => v
                .as_str()
                .and_then(ScatterMode::from_wire)
                .unwrap_or_default(),
            None => base.mode,
        },
        biomes: match g("biomes") {
            // `Number.isFinite` does not coerce: a `"4"` is filtered out, a
            // `5.5` is kept (and simply never matches a biome index).
            Some(Value::Array(a)) => a.iter().filter_map(Value::as_f64).collect(),
            Some(_) => Vec::new(),
            None => base.biomes,
        },
        min_size,
        max_size: f64::max(min_size, num(g("maxSize"), 0.05, 8.0, base.max_size)),
        // Density is the one field whose fallback is *not* the slot preset:
        // the reference merges first and then runs `num(out.density,0,3,1)`,
        // so an absent key keeps the preset's own density but a **rejected**
        // one lands on a literal 1. Golden-verified both ways — `cactus` with
        // no density stays 0.35, `cactus` with `"x"` becomes 1.
        density: match g("density") {
            None => base.density,
            v => num(v, 0.0, 3.0, 1.0),
        },
        spacing: match g("spacing") {
            None => base.spacing,
            v => num_opt(v, 1.0, 512.0),
        },
        elev_min: match g("elevMin") {
            None => base.elev_min,
            v => num_opt(v, 0.0, 1.0),
        },
        elev_max: match g("elevMax") {
            None => base.elev_max,
            v => num_opt(v, 0.0, 1.0),
        },
        require_wetland: g("requireWetland").map_or(base.require_wetland, truthy),
        variant_weights: match g("variantWeights") {
            Some(Value::Array(a)) => {
                Some(a.iter().map(|w| num(Some(w), 0.0, 100.0, 0.0)).collect())
            }
            Some(_) => None,
            None => base.variant_weights,
        },
    };
    if let (Some(lo), Some(hi)) = (out.elev_min, out.elev_max)
        && hi < lo
    {
        out.elev_max = Some(lo);
    }
    out
}

// ---------------------------------------------------------------------------
// Variant selection
// ---------------------------------------------------------------------------

/// Deterministic variant pick by position hash — the reference's
/// `pickIconVariant` (line 12171).
///
/// Same world ⇒ same icons ⇒ stable re-exports, which is the whole point of
/// letting one slot hold several drawings. The `min(n - 1, …)` is not
/// decoration: [`hash`] divides by `2^32 - 1`, so it can return exactly `1.0`
/// and the product can reach `n`.
pub fn pick_icon_variant(x: i32, y: i32, seed: i32, n: usize) -> usize {
    if n <= 1 {
        return 0;
    }
    ((hash(x, y, seed) * n as f64) as usize).min(n - 1)
}

/// Weighted variant pick — the reference's `pickWeightedVariant`.
///
/// Falls through to [`pick_icon_variant`]'s **exact** hash whenever the
/// weights are absent, the wrong length, or degenerate (all zero, all
/// negative, all non-finite), so an asset with no configured weighting keeps
/// its v1.25 variant selection bit for bit.
pub fn pick_weighted_variant(
    x: i32,
    y: i32,
    seed: i32,
    n: usize,
    weights: Option<&[f64]>,
) -> usize {
    if n <= 1 {
        return 0;
    }
    // `Math.max(0, w||0)`: negatives and NaN alike contribute nothing. Rust's
    // `f64::max` absorbs NaN, so `w.max(0.0)` covers both cases as JS does.
    let clean = |w: f64| w.max(0.0);
    let Some(weights) = weights.filter(|w| w.len() == n) else {
        return pick_icon_variant(x, y, seed, n);
    };
    let total: f64 = weights.iter().copied().map(clean).sum();
    // The reference writes this as `!(total>0)`, whose point is that it also
    // catches a NaN total. Spelled out here so the NaN case is deliberate
    // rather than a side effect of operator negation.
    if total <= 0.0 || total.is_nan() {
        return pick_icon_variant(x, y, seed, n);
    }
    let mut t = hash(x, y, seed) * total;
    for (i, &w) in weights.iter().enumerate() {
        t -= clean(w);
        if t <= 0.0 {
            return i;
        }
    }
    n - 1
}

// ---------------------------------------------------------------------------
// The rule table
// ---------------------------------------------------------------------------

/// The runtime rule table: rule key → rule, in the order the keys were added.
///
/// The reference's `assetRules` module global. Order is preserved because the
/// reference's is (JavaScript iterates string keys in insertion order) and
/// because it reaches [`current_scatter_rules`]' output — though since v1.27
/// it no longer decides which rule *wins* a cell, that being the whole point
/// of the specificity sort in `placeMapIconsRuled`.
pub type ScatterRuleTable = OrderedMap<ScatterRule>;

/// Flatten a rule table into the keyed list the placement engine wants,
/// dropping disabled assets — the reference's `currentScatterRules`.
///
/// `None` when nothing is configured. That is not a mere emptiness signal: it
/// is precisely what keeps `placeMapIcons` on its **legacy** path, so a
/// pack-less map renders bit-identically to v1.25.
pub fn current_scatter_rules(table: &ScatterRuleTable) -> Option<Vec<(&str, &ScatterRule)>> {
    let out: Vec<_> = table.iter().filter(|(_, r)| r.enabled).collect();
    (!out.is_empty()).then_some(out)
}

/// Bind each slot of a freshly imported pack to that slot's default
/// behaviour — the reference's `autopopulateScatterRules`.
///
/// Dropping art into `tree_conifer` scatters it across boreal/conifer cells
/// with no manual setup. Custom-set assets default to **disabled**: a set name
/// is arbitrary user vocabulary with no meaningful default biome, so
/// scattering them would be inventing intent the user never expressed — they
/// light up once a rule is configured.
///
/// Never clobbers an existing rule, so re-importing a pack cannot silently
/// undo the user's tuning.
pub fn autopopulate_scatter_rules(table: &mut ScatterRuleTable, pack: &PackManifest) {
    for (slot, variants) in pack.icons.iter() {
        if variants.is_empty() || table.contains_key(slot) {
            continue;
        }
        table.insert(slot, preset_scatter_rule(slot));
    }
    for (set, slots) in pack.custom.iter() {
        for (slot_id, _) in slots.iter() {
            // Reproduced faithfully: unlike the icons loop above, the
            // reference does *not* skip a custom slot with no variants, so an
            // empty set entry still gets a disabled rule.
            let key = scatter_rule_key(slot_id, Some(set));
            if table.contains_key(&key) {
                continue;
            }
            table.insert(
                key,
                ScatterRule {
                    enabled: false,
                    ..ScatterRule::default()
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn default_rule_matches_the_reference() {
        let d = ScatterRule::default();
        assert!(d.enabled);
        assert_eq!(d.mode, ScatterMode::Scatter);
        assert_eq!((d.min_size, d.max_size, d.density), (0.7, 1.2, 1.0));
        assert!(d.biomes.is_empty());
        assert_eq!((d.spacing, d.elev_min, d.elev_max), (None, None, None));
        assert!(!d.require_wetland);
        assert!(d.variant_weights.is_none());
    }

    /// The preset table's keys are exactly `PACK_ICON_SLOTS` — a fact worth
    /// pinning, since it is what makes an imported icon pack behave like
    /// v1.25 with no configuration at all.
    #[test]
    fn every_frozen_icon_slot_has_a_preset_and_nothing_else_does() {
        for slot in crate::PACK_ICON_SLOTS {
            assert_ne!(
                preset_scatter_rule(slot),
                ScatterRule::default(),
                "{slot} should carry a preset"
            );
        }
        for other in ["custom::Trees::oak", "settlement_city", "", "Mountain"] {
            assert_eq!(preset_scatter_rule(other), ScatterRule::default());
        }
    }

    #[test]
    fn rule_keys_address_frozen_and_custom_assets() {
        assert_eq!(scatter_rule_key("mountain", None), "mountain");
        assert_eq!(scatter_rule_key("mountain", Some("")), "mountain");
        assert_eq!(scatter_rule_key("oak", Some("Trees")), "custom::Trees::oak");
    }

    #[test]
    fn mode_parsing_is_case_sensitive_and_total() {
        assert_eq!(ScatterMode::from_wire("relief"), Some(ScatterMode::Relief));
        assert_eq!(ScatterMode::from_wire("RELIEF"), None);
        assert_eq!(ScatterMode::from_wire("wander"), None);
        assert_eq!(ScatterMode::Relief.as_str(), "relief");
    }

    #[test]
    fn non_object_input_yields_the_preset() {
        for raw in [json!(null), json!("relief"), json!(7), json!([1, 2])] {
            assert_eq!(
                normalize_scatter_rule(&raw, "mountain"),
                preset_scatter_rule("mountain")
            );
        }
    }

    #[test]
    fn elevation_band_is_ordered_and_clamped() {
        let r = normalize_scatter_rule(&json!({"elevMin": 0.9, "elevMax": 0.2}), "hill");
        assert_eq!((r.elev_min, r.elev_max), (Some(0.9), Some(0.9)));
        let r = normalize_scatter_rule(&json!({"elevMin": -3, "elevMax": 9}), "hill");
        assert_eq!((r.elev_min, r.elev_max), (Some(0.0), Some(1.0)));
        // An explicit null clears the preset's own band.
        let r = normalize_scatter_rule(&json!({"elevMin": null}), "mountain");
        assert_eq!(r.elev_min, None);
    }

    #[test]
    fn max_size_never_falls_below_min_size() {
        let r = normalize_scatter_rule(&json!({"minSize": 3, "maxSize": 1}), "mountain");
        assert_eq!((r.min_size, r.max_size), (3.0, 3.0));
    }

    #[test]
    fn biomes_filter_keeps_only_real_numbers_without_coercing() {
        let r = normalize_scatter_rule(&json!({"biomes": [3, "4", null, 5.5, -2]}), "boulder");
        assert_eq!(r.biomes, vec![3.0, 5.5, -2.0]);
        // A non-array `biomes` means "any land", not "keep the preset's".
        let r = normalize_scatter_rule(&json!({"biomes": 5}), "boulder");
        assert!(r.biomes.is_empty());
    }

    #[test]
    fn variant_weights_are_clamped_per_entry() {
        let r = normalize_scatter_rule(
            &json!({"variantWeights": [1, "2", null, -5, 1e9, "x"]}),
            "mountain",
        );
        assert_eq!(
            r.variant_weights.unwrap(),
            vec![1.0, 2.0, 0.0, 0.0, 100.0, 0.0]
        );
        let r = normalize_scatter_rule(&json!({"variantWeights": "abc"}), "mountain");
        assert!(r.variant_weights.is_none());
    }

    /// JS truthiness, not a JSON-boolean check: `0` is off, `"no"` is on.
    #[test]
    fn boolean_fields_use_javascript_truthiness() {
        assert!(!normalize_scatter_rule(&json!({"enabled": 0}), "cactus").enabled);
        assert!(normalize_scatter_rule(&json!({"enabled": "no"}), "cactus").enabled);
        assert!(normalize_scatter_rule(&json!({"requireWetland": "yes"}), "shrub").require_wetland);
        // Absent leaves the preset's own value alone.
        assert!(normalize_scatter_rule(&json!({}), "tree_wetland").require_wetland);
    }

    /// The one place this port knowingly differs from `+v`, kept deliberate
    /// by being asserted: JS coerces `[2]` to `2` via `ToPrimitive`; here an
    /// array is not a number, so the field falls back to its default.
    #[test]
    fn array_valued_number_fields_fall_back_rather_than_coercing() {
        let r = normalize_scatter_rule(&json!({"density": [2]}), "shrub");
        assert_eq!(r.density, 1.0);
    }

    #[test]
    fn string_numbers_coerce_the_way_javascript_does() {
        let d = |v: Value| normalize_scatter_rule(&json!({ "density": v }), "shrub").density;
        assert_eq!(d(json!("2.5")), 2.5);
        assert_eq!(d(json!("0x2")), 2.0); // ES radix literal
        assert_eq!(d(json!(true)), 1.0);
        assert_eq!(d(json!("   ")), 0.0); // `+"   "` is 0
        assert_eq!(d(json!("")), 1.0); // but `''` is an explicit "unset"
    }

    #[test]
    fn spacing_derives_from_density_and_floors_at_three() {
        // Explicit spacing wins.
        let r = ScatterRule {
            spacing: Some(12.0),
            ..Default::default()
        };
        assert_eq!(r.spacing_cells(900), 12.0);
        // Derived: round(max(5, 900/90) / max(0.15, 1)) == 10.
        assert_eq!(ScatterRule::default().spacing_cells(900), 10.0);
        // Density 0 is falsy in the reference and derives as if it were 1.
        let r = ScatterRule {
            density: 0.0,
            ..Default::default()
        };
        assert_eq!(r.spacing_cells(900), 10.0);
        // Runaway density still cannot pack tighter than 3 cells.
        let r = ScatterRule {
            density: 3.0,
            ..Default::default()
        };
        assert_eq!(r.spacing_cells(180), 3.0);
    }

    #[test]
    fn weighted_pick_falls_through_to_the_plain_hash_when_degenerate() {
        for (x, y) in [(0, 0), (13, 7), (-4, 91)] {
            let plain = pick_icon_variant(x, y, 7, 4);
            assert_eq!(pick_weighted_variant(x, y, 7, 4, None), plain);
            assert_eq!(pick_weighted_variant(x, y, 7, 4, Some(&[1.0, 2.0])), plain);
            assert_eq!(pick_weighted_variant(x, y, 7, 4, Some(&[0.0; 4])), plain);
            assert_eq!(pick_weighted_variant(x, y, 7, 4, Some(&[-1.0; 4])), plain);
        }
        assert_eq!(pick_weighted_variant(3, 3, 7, 1, Some(&[5.0])), 0);
        assert_eq!(pick_weighted_variant(3, 3, 7, 0, None), 0);
    }

    #[test]
    fn weighted_pick_respects_a_one_hot_weighting() {
        for x in 0..50 {
            assert_eq!(
                pick_weighted_variant(x, x * 3, 7, 3, Some(&[0.0, 1.0, 0.0])),
                1
            );
        }
    }

    #[test]
    fn current_rules_drops_disabled_and_signals_none_when_empty() {
        let mut t = ScatterRuleTable::new();
        assert!(current_scatter_rules(&t).is_none());
        t.insert("zebra", preset_scatter_rule("mountain"));
        t.insert(
            "apple",
            ScatterRule {
                enabled: false,
                ..Default::default()
            },
        );
        t.insert("mango", preset_scatter_rule("cactus"));
        let out = current_scatter_rules(&t).unwrap();
        assert_eq!(
            out.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
            ["zebra", "mango"]
        );

        let mut all_off = ScatterRuleTable::new();
        all_off.insert(
            "a",
            ScatterRule {
                enabled: false,
                ..Default::default()
            },
        );
        assert!(current_scatter_rules(&all_off).is_none());
    }

    #[test]
    fn rule_serializes_with_the_reference_field_names() {
        let json = serde_json::to_string(&preset_scatter_rule("hill")).unwrap();
        assert_eq!(
            json,
            r#"{"enabled":true,"mode":"relief","biomes":[],"minSize":0.5,"maxSize":1.0,"density":1.0,"spacing":null,"elevMin":0.53,"elevMax":0.58,"requireWetland":false,"variantWeights":null}"#
        );
    }
}
