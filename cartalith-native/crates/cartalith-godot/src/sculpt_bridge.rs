//! The Sculpt editor's Godot-facing bridge state — `UNIFIED_TOOL_PLAN.md`
//! milestone F (`STRANDED_TOOLS.md` rows 4-8: Raise/lower, Smooth,
//! Flatten/terrace, Stamp, River/water; `SCULPT_FUNCTION_CHART.md` §11's
//! "GDExtension binding — in progress" row).
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `params.rs`'s own doc comment argues for: `lib.rs` owns the thin
//! `Variant`<->`f64`/`String` conversion and the `#[func]` surface; this
//! module owns the actual state machine — current tool selection, the
//! draft stack, and the water hooks — with its own `#[cfg(test)]` suite
//! below, exercised by `cargo test -p cartalith-godot`'s ordinary unit-test
//! pass with no Godot runtime or engine process involved.
//!
//! ## Why this lives on `WorldGen`, not a second `GodotClass`
//!
//! [`SculptEditor`] is created and destroyed alongside a generated world
//! (see [`SculptEditor::new`], called from `WorldGen::absorb`) because
//! every one of its operations — preview, commit, the water hooks — needs
//! `WorldGen`'s own live `WorldState::field`, and only ever that one. A
//! sibling `GodotClass` would need its own `Gd<WorldGen>` handle and
//! re-borrow across the gdext boundary on every call for no benefit:
//! nothing about the Sculpt editor is independently constructible or
//! reusable across worlds, and `WorldGen` already keeps exactly this kind
//! of per-world layer as a plain field (`civ: Option<CivData>`,
//! `asset_pack: Option<pack::LoadedPack>`) rather than as a separate class.
//!
//! ## Registry vs. tool state vs. draft
//!
//! Three different things share this file, matching the reference's own
//! separation (`SCULPT_FUNCTION_CHART.md` §1/§5): the **registry**
//! (`SCULPT_FEATURES`/`SCULPT_PRESETS`, read-only, already public from
//! `cartalith_terrain::sculpt` and exposed by `lib.rs`'s
//! `get_sculpt_features`/`get_sculpt_presets`), the **current tool state**
//! (which feature is selected, its live parameter values, the shared
//! brush/noise globals, the in-progress stroke's captured points — the
//! reference's own `_sculptFeat`/`_sculptParams`/`_sculptGlobals`/
//! `_sculptPts` module globals), and the **draft** itself
//! (`PassBuffer<SculptStamp>`, milestone A). `sculpt_end_stroke` in
//! `lib.rs` is the seam: it freezes the current tool state plus the
//! captured points into one `SculptStamp` and pushes it.
//!
//! ## Defaults come from the engine, not the design spec — settled
//!
//! `SCULPT_FUNCTION_CHART.md` §4 found that five of the eight brush/noise
//! globals' defaults disagreed between `DCC_SHELL_SPEC.md` §5.2 (the design
//! team's own starting point) and `SculptGlobals::default()`
//! (`SCULPT_GLOBAL_DEF`, the reference's own value, golden-pinned). The
//! owner has since resolved this: the design numbers were placeholders,
//! and the engine's own set wins for all eight — on a concrete ground, not
//! just precedent: `cartalith-engine/tests/golden_parity_sculpt_water.rs`
//! spreads `..SculptGlobals::default()` into its fixtures, so these eight
//! numbers are golden-parity *inputs*, and changing even one would
//! invalidate a pinned test. [`global_controls`] therefore reports the
//! engine's default, read live from `SculptGlobals::default()` rather than
//! hand-copied — so this table can never drift from the value the golden
//! suite actually depends on. `DCC_SHELL_SPEC.md` §5.2's differing numbers
//! will be corrected at the design end, not here.

use cartalith_engine::sculpt_commit::{commit_sculpt_pass, SculptCommitSummary, WaterState};
use cartalith_spatial::{DirtyTracker, PassBuffer};
use cartalith_terrain::sculpt::{Control, Feature, FeatureParams, Point, SculptGlobals, SculptStamp};

use crate::selection::SelectionSet;

/// Tile granularity for the sculpt draft's `PassBuffer`/`DirtyTracker`
/// pair. The reference has no tiling concept at all for Sculpt (one
/// monolithic canvas), so there is no reference value to port — this
/// port's own choice, picked so a single stroke at the largest brush
/// (200px, `DCC_SHELL_SPEC.md` §5.2) touches a handful of tiles rather
/// than either one giant tile (no locality benefit for a future
/// partial-reupload renderer) or hundreds of tiny ones (bookkeeping
/// overhead for no reason at this port's typical 512-2048 grids).
pub const SCULPT_TILE_SIZE: usize = 64;

/// What writing one control value did — the same three-way shape
/// `params::Outcome` uses for generation parameters, kept as its own type
/// rather than reused so this module stays importable on its own (see the
/// module doc's isolation note) without pulling `params.rs` in too.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Stored exactly as given.
    Applied,
    /// Stored, but clamped to the control's own range (or rounded to a
    /// whole number for `octaves`).
    Clamped,
    /// Not stored — unknown key, or a non-finite value.
    Rejected,
}

/// [`SculptGlobals`]' 8 fields' `[min, max, step]`, transcribed from
/// `SculptGlobals`'s own field doc comments (themselves ported from the
/// reference's `SCULPT_GLOBAL_DEF` UI ranges — `DCC_SHELL_SPEC.md` §5.2's
/// "Brush & noise · global" table matches every one of these eight ranges
/// exactly, per `SCULPT_FUNCTION_CHART.md` §4). Only the *range* is
/// duplicated here; the *default* is never hand-copied — see
/// [`global_controls`].
const GLOBAL_RANGES: &[(&str, &str, f64, f64, f64)] = &[
    ("brush_size", "Brush size", 6.0, 200.0, 1.0),
    ("hardness", "Hardness", 0.0, 1.0, 0.01),
    ("intensity", "Intensity", 0.0, 1.5, 0.01),
    ("noise_scale", "Noise scale", 1.0, 20.0, 0.5),
    ("octaves", "Octaves", 1.0, 8.0, 1.0),
    ("persistence", "Persistence", 0.20, 0.90, 0.01),
    ("lacunarity", "Lacunarity", 1.40, 3.20, 0.05),
    ("edge_noise", "Edge noise", 0.0, 1.0, 0.01),
];

/// [`GLOBAL_RANGES`] joined with `SculptGlobals::default()`'s live values —
/// the same `Control` shape `Feature::meta().controls` uses per feature, so
/// a caller enumerates the shared brush/noise block exactly the way it
/// enumerates a feature's own parameters.
pub fn global_controls() -> Vec<Control> {
    let d = SculptGlobals::default();
    GLOBAL_RANGES
        .iter()
        .map(|&(key, label, min, max, step)| Control {
            key,
            label,
            min,
            max,
            step,
            default: get_global(&d, key).expect("GLOBAL_RANGES key must resolve against get_global"),
        })
        .collect()
}

/// Reads one field of `g` by [`GLOBAL_RANGES`] key. `None` for an unknown key.
pub fn get_global(g: &SculptGlobals, key: &str) -> Option<f64> {
    Some(match key {
        "brush_size" => g.brush_size,
        "hardness" => g.hardness,
        "intensity" => g.intensity,
        "noise_scale" => g.noise_scale,
        "octaves" => f64::from(g.octaves),
        "persistence" => g.persistence,
        "lacunarity" => g.lacunarity,
        "edge_noise" => g.edge_noise,
        _ => return None,
    })
}

/// Writes one field of `g` by key, clamped to [`GLOBAL_RANGES`]. Unknown
/// key -> [`Outcome::Rejected`]; non-finite value -> [`Outcome::Rejected`]
/// (same policy `params::set` documents, and for the same reason: a NaN
/// silently propagates through every stamp's noise —
/// `cartalith-rust-conventions`). `octaves` rounds before clamping, same
/// as any `Kind::Int` generation parameter.
pub fn set_global(g: &mut SculptGlobals, key: &str, value: f64) -> Outcome {
    let Some(&(_, _, min, max, _)) = GLOBAL_RANGES.iter().find(|(k, ..)| *k == key) else {
        return Outcome::Rejected;
    };
    if !value.is_finite() {
        return Outcome::Rejected;
    }
    let mut v = value;
    if key == "octaves" {
        v = v.round();
    }
    let clamped = v.clamp(min, max);
    match key {
        "brush_size" => g.brush_size = clamped,
        "hardness" => g.hardness = clamped,
        "intensity" => g.intensity = clamped,
        "noise_scale" => g.noise_scale = clamped,
        "octaves" => g.octaves = clamped as u32,
        "persistence" => g.persistence = clamped,
        "lacunarity" => g.lacunarity = clamped,
        "edge_noise" => g.edge_noise = clamped,
        _ => unreachable!("checked above against the same GLOBAL_RANGES table"),
    }
    if clamped == value { Outcome::Applied } else { Outcome::Clamped }
}

/// One [`FeatureParams`] variant's fields as `(control key, value)` pairs,
/// in [`Feature::meta`]'s own `controls` order — the reference's own
/// `f:{...}` bag, read back out. Freehand's `sub_mode` is not a numeric
/// control (`FREEHAND_CTL` has one entry, `amount`) and is reported
/// separately by the caller (`lib.rs`'s `sculpt_get_feature_params`), not
/// folded in here.
pub fn feature_param_pairs(p: &FeatureParams) -> Vec<(&'static str, f64)> {
    match *p {
        FeatureParams::Mountains { mtn_height, peak_sharpness, ridge_frequency, ruggedness } => vec![
            ("mtnHeight", mtn_height),
            ("peakSharpness", peak_sharpness),
            ("ridgeFrequency", ridge_frequency),
            ("ruggedness", ruggedness),
        ],
        FeatureParams::Hills { amplitude, rolling_frequency, softness } => vec![
            ("amplitude", amplitude),
            ("rollingFrequency", rolling_frequency),
            ("softness", softness),
        ],
        FeatureParams::Ridge { ridge_height, ridge_width, ridge_freq } => vec![
            ("ridgeHeight", ridge_height),
            ("ridgeWidth", ridge_width),
            ("ridgeFreq", ridge_freq),
        ],
        FeatureParams::Plateau { plateau_height, terraces, plateau_freq } => vec![
            ("plateauHeight", plateau_height),
            ("terraces", terraces),
            ("plateauFreq", plateau_freq),
        ],
        FeatureParams::Cliff { cliff_height, cliff_steep } => vec![
            ("cliffHeight", cliff_height),
            ("cliffSteep", cliff_steep),
        ],
        FeatureParams::Canyon { canyon_depth, wall_steepness, meander } => vec![
            ("canyonDepth", canyon_depth),
            ("wallSteepness", wall_steepness),
            ("meander", meander),
        ],
        FeatureParams::Valley { valley_depth, valley_width, meander } => vec![
            ("valleyDepth", valley_depth),
            ("valleyWidth", valley_width),
            ("meander", meander),
        ],
        FeatureParams::River { river_width, river_depth, river_meander, branch_noise } => vec![
            ("riverWidth", river_width),
            ("riverDepth", river_depth),
            ("riverMeander", river_meander),
            ("branchNoise", branch_noise),
        ],
        FeatureParams::Lake { lake_depth, lake_shore } => vec![
            ("lakeDepth", lake_depth),
            ("lakeShore", lake_shore),
        ],
        FeatureParams::Basin { basin_depth, basin_rough } => vec![
            ("basinDepth", basin_depth),
            ("basinRough", basin_rough),
        ],
        FeatureParams::Coastline { coast_amount, coast_ragged } => vec![
            ("coastAmount", coast_amount),
            ("coastRagged", coast_ragged),
        ],
        FeatureParams::Volcano { volc_height, crater_depth, volc_radius, flank_rough } => vec![
            ("volcHeight", volc_height),
            ("craterDepth", crater_depth),
            ("volcRadius", volc_radius),
            ("flankRough", flank_rough),
        ],
        FeatureParams::Freehand { amount, .. } => vec![("amount", amount)],
    }
}

/// The mirror of [`feature_param_pairs`]: writes one control value into
/// `p`, which must already be `feature`'s own variant (a caller switches
/// feature — which resets `p` to that feature's defaults — before tuning
/// its controls; see `lib.rs`'s `sculpt_set_feature`). Clamped to that
/// control's own `[min, max]` (`Feature::meta().controls`), same policy as
/// [`set_global`]. Unknown key -> [`Outcome::Rejected`] — including a key
/// that legitimately belongs to a *different* feature's controls (e.g.
/// `"riverWidth"` sent while Mountains is selected): a typo in a shell
/// control that silently did nothing would be worse than one that is
/// visibly rejected.
pub fn set_feature_param(p: &mut FeatureParams, feature: Feature, key: &str, value: f64) -> Outcome {
    let Some(c) = feature.meta().controls.iter().find(|c| c.key == key) else {
        return Outcome::Rejected;
    };
    if !value.is_finite() {
        return Outcome::Rejected;
    }
    let clamped = value.clamp(c.min, c.max);
    set_feature_field(p, key, clamped);
    if clamped == value { Outcome::Applied } else { Outcome::Clamped }
}

/// The actual field write `set_feature_param` delegates to, split out so
/// the range-check/clamp policy above stays in one place and this match is
/// pure plumbing. A `key` not recognised for `p`'s own variant is a no-op
/// — unreachable in practice, since `set_feature_param` already rejected
/// any key not in `feature.meta().controls`, but kept total rather than
/// panicking on the (impossible, but not type-enforced) mismatch.
fn set_feature_field(p: &mut FeatureParams, key: &str, v: f64) {
    match p {
        FeatureParams::Mountains { mtn_height, peak_sharpness, ridge_frequency, ruggedness } => match key {
            "mtnHeight" => *mtn_height = v,
            "peakSharpness" => *peak_sharpness = v,
            "ridgeFrequency" => *ridge_frequency = v,
            "ruggedness" => *ruggedness = v,
            _ => {}
        },
        FeatureParams::Hills { amplitude, rolling_frequency, softness } => match key {
            "amplitude" => *amplitude = v,
            "rollingFrequency" => *rolling_frequency = v,
            "softness" => *softness = v,
            _ => {}
        },
        FeatureParams::Ridge { ridge_height, ridge_width, ridge_freq } => match key {
            "ridgeHeight" => *ridge_height = v,
            "ridgeWidth" => *ridge_width = v,
            "ridgeFreq" => *ridge_freq = v,
            _ => {}
        },
        FeatureParams::Plateau { plateau_height, terraces, plateau_freq } => match key {
            "plateauHeight" => *plateau_height = v,
            "terraces" => *terraces = v,
            "plateauFreq" => *plateau_freq = v,
            _ => {}
        },
        FeatureParams::Cliff { cliff_height, cliff_steep } => match key {
            "cliffHeight" => *cliff_height = v,
            "cliffSteep" => *cliff_steep = v,
            _ => {}
        },
        FeatureParams::Canyon { canyon_depth, wall_steepness, meander } => match key {
            "canyonDepth" => *canyon_depth = v,
            "wallSteepness" => *wall_steepness = v,
            "meander" => *meander = v,
            _ => {}
        },
        FeatureParams::Valley { valley_depth, valley_width, meander } => match key {
            "valleyDepth" => *valley_depth = v,
            "valleyWidth" => *valley_width = v,
            "meander" => *meander = v,
            _ => {}
        },
        FeatureParams::River { river_width, river_depth, river_meander, branch_noise } => match key {
            "riverWidth" => *river_width = v,
            "riverDepth" => *river_depth = v,
            "riverMeander" => *river_meander = v,
            "branchNoise" => *branch_noise = v,
            _ => {}
        },
        FeatureParams::Lake { lake_depth, lake_shore } => match key {
            "lakeDepth" => *lake_depth = v,
            "lakeShore" => *lake_shore = v,
            _ => {}
        },
        FeatureParams::Basin { basin_depth, basin_rough } => match key {
            "basinDepth" => *basin_depth = v,
            "basinRough" => *basin_rough = v,
            _ => {}
        },
        FeatureParams::Coastline { coast_amount, coast_ragged } => match key {
            "coastAmount" => *coast_amount = v,
            "coastRagged" => *coast_ragged = v,
            _ => {}
        },
        FeatureParams::Volcano { volc_height, crater_depth, volc_radius, flank_rough } => match key {
            "volcHeight" => *volc_height = v,
            "craterDepth" => *crater_depth = v,
            "volcRadius" => *volc_radius = v,
            "flankRough" => *flank_rough = v,
            _ => {}
        },
        FeatureParams::Freehand { amount, .. } => {
            if key == "amount" {
                *amount = v;
            }
        }
    }
}

/// The live Sculpt-editor state for one generated world: current tool
/// selection (feature, its live parameters, the shared brush/noise
/// globals, the in-progress stroke's captured points), the non-destructive
/// draft stack, and the water-lock state the commit hooks read and write
/// across multiple commits (`WaterState`'s own doc comment: a second lake
/// painted on a later commit must still see the first's mask).
pub struct SculptEditor {
    pub draft: PassBuffer<SculptStamp>,
    pub tracker: DirtyTracker,
    pub water: WaterState,
    /// The feature the next stroke will paint. Defaults to `Mountains` —
    /// an arbitrary first selection (the reference's own panel opens on
    /// whichever `SCULPT_FEATURES` entry is first, which is also
    /// Mountains); a caller normally calls `sculpt_set_feature` or
    /// `sculpt_apply_preset` before painting.
    pub feature: Feature,
    /// `feature`'s own live control values — always the same variant as
    /// `feature` (`sculpt_set_feature`/`sculpt_apply_preset` keep the two
    /// in lockstep; nothing else in this module ever changes `feature`
    /// without also resetting `params`).
    pub params: FeatureParams,
    pub globals: SculptGlobals,
    /// The seed the *next* stroke captures into its `SculptStamp` — the
    /// reference's own project seed default, settable so a shell's dice
    /// button can randomise it before the next stroke.
    pub seed: u32,
    /// The in-progress stroke's captured points, grid-cell coordinates.
    /// Cleared by `sculpt_begin_stroke`/`sculpt_end_stroke`/
    /// `sculpt_cancel_stroke`.
    pub points: Vec<Point>,
    /// Which draft stamps are selected — [`crate::selection::SelectionSet`],
    /// whose `primary()` is what the old `selected: Option<usize>` field was
    /// and what `sculpt_get_selected_stamp` still reports. Step one of the
    /// owner's selection-sets ruling; see that module's own doc comment.
    pub selection: SelectionSet,
}

impl SculptEditor {
    /// A fresh editor over a `gw x gh` world — called once per
    /// `generate()`/`generate_sized()` (`WorldGen::absorb`), never reused
    /// across worlds: a draft over one world's dimensions is meaningless
    /// over another's, and `river_mask`/`river_floor` are this world's own
    /// (`from_generated` when the just-finished generation actually
    /// carved rivers, an empty lock state otherwise — mirroring
    /// `WaterState::from_generated`'s own "guards `enforce_river_channels`
    /// the same way the reference's global does" doc comment).
    pub fn new(
        gw: usize,
        gh: usize,
        river_mask: Option<Vec<u8>>,
        river_floor: Option<Vec<f32>>,
        seed: u32,
    ) -> Self {
        let n = gw * gh;
        let draft = PassBuffer::new(gw, gh, SCULPT_TILE_SIZE);
        let tracker = DirtyTracker::new(draft.tile_count());
        let water = match (river_mask, river_floor) {
            (Some(m), Some(f)) if m.len() == n && f.len() == n => WaterState::from_generated(m, f),
            _ => WaterState::new(n),
        };
        let feature = Feature::Mountains;
        Self {
            draft,
            tracker,
            water,
            feature,
            params: feature.default_params(),
            globals: SculptGlobals::default(),
            seed,
            points: Vec::new(),
            selection: SelectionSet::new(),
        }
    }

    /// The stamp a single-selection operation acts on — the set's primary.
    /// What `sculpt_get_selected_stamp` reports, and what the old `selected`
    /// field held before it became a set.
    pub fn selected(&self) -> Option<usize> {
        self.selection.primary()
    }

    /// Removes a stamp from the draft and re-points the selection at what
    /// survives — `WorldGen::sculpt_delete_stamp`'s whole body, moved here so
    /// the index bookkeeping is reachable from an ordinary unit test (a
    /// `#[func]` on `WorldGen` is not: it needs a Godot runtime). `false` for
    /// an out-of-range `index`.
    ///
    /// `PassBuffer::remove` renumbers every stamp after `index`, so the
    /// selection has to shift with them. See that binding's own doc comment
    /// for what the previous equal-case-only guard silently got wrong.
    pub fn delete_stamp(&mut self, index: usize) -> bool {
        if index >= self.draft.len() {
            return false;
        }
        self.draft.remove(index);
        self.selection.retain_after_remove(index);
        true
    }

    /// Bakes the whole draft into `field` — `commit_sculpt_pass` unchanged,
    /// see its own module doc for the exact five-step ordering
    /// (`SCULPT_FUNCTION_CHART.md` §7). Marks tiles stale in `self.tracker`
    /// via that call; deliberately does **not** recompute erosion,
    /// hydrology or climate — `UNIFIED_TOOL_PLAN.md` milestone C measured
    /// the eager form at ~7s/stroke at 2048² and rejected it. Clears the
    /// draft on return (`PassBuffer::commit`'s own contract), so
    /// `self.selected` is left pointing nowhere valid — callers should
    /// treat a commit as also deselecting.
    pub fn commit(&mut self, field: &mut [f32], sea_level: f64, reason: &str) -> SculptCommitSummary {
        let summary = commit_sculpt_pass(&mut self.draft, field, &mut self.water, &mut self.tracker, reason, sea_level);
        self.selection.clear();
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_terrain::sculpt::FEATURE_KEYS;

    // ---- globals ----

    #[test]
    fn global_controls_defaults_match_sculpt_globals_default() {
        let d = SculptGlobals::default();
        for c in global_controls() {
            let live = get_global(&d, c.key).unwrap();
            assert_eq!(c.default, live, "control {} default drifted from SculptGlobals::default()", c.key);
            assert!(c.min <= c.default && c.default <= c.max, "{} default outside its own range", c.key);
        }
    }

    #[test]
    fn set_global_round_trips_every_key_at_its_own_default() {
        let d = SculptGlobals::default();
        for c in global_controls() {
            let mut g = SculptGlobals::default();
            let before = get_global(&d, c.key).unwrap();
            assert_eq!(set_global(&mut g, c.key, before), Outcome::Applied, "{}", c.key);
            assert_eq!(get_global(&g, c.key), Some(before));
        }
    }

    #[test]
    fn set_global_clamps_out_of_range_and_reports_it() {
        let mut g = SculptGlobals::default();
        assert_eq!(set_global(&mut g, "brush_size", 9999.0), Outcome::Clamped);
        assert_eq!(g.brush_size, 200.0);
        assert_eq!(set_global(&mut g, "hardness", -5.0), Outcome::Clamped);
        assert_eq!(g.hardness, 0.0);
    }

    #[test]
    fn set_global_rounds_octaves() {
        let mut g = SculptGlobals::default();
        assert_eq!(set_global(&mut g, "octaves", 3.9), Outcome::Clamped);
        assert_eq!(g.octaves, 4);
    }

    #[test]
    fn set_global_rejects_unknown_key_and_non_finite() {
        let mut g = SculptGlobals::default();
        assert_eq!(set_global(&mut g, "nope", 1.0), Outcome::Rejected);
        assert_eq!(set_global(&mut g, "hardness", f64::NAN), Outcome::Rejected);
        assert!(!g.hardness.is_nan(), "a rejected NaN must not have been written");
    }

    // ---- feature params ----

    #[test]
    fn every_feature_param_round_trips_through_its_own_default() {
        for f in FEATURE_KEYS {
            let mut p = f.default_params();
            let pairs = feature_param_pairs(&p);
            assert_eq!(
                pairs.len(),
                f.meta().controls.len(),
                "{}: feature_param_pairs must report exactly one entry per control",
                f.meta().key
            );
            for (key, before) in pairs {
                assert_eq!(set_feature_param(&mut p, f, key, before), Outcome::Applied, "{}.{}", f.meta().key, key);
                let after = feature_param_pairs(&p).into_iter().find(|(k, _)| *k == key).unwrap().1;
                assert_eq!(before, after, "{}.{} did not round-trip", f.meta().key, key);
            }
        }
    }

    #[test]
    fn set_feature_param_clamps_to_the_controls_own_range() {
        let mut p = Feature::Mountains.default_params();
        let c = Feature::Mountains.meta().controls[0]; // mtnHeight, 0.10..0.55
        assert_eq!(set_feature_param(&mut p, Feature::Mountains, c.key, 999.0), Outcome::Clamped);
        assert_eq!(feature_param_pairs(&p)[0].1, c.max);
    }

    #[test]
    fn set_feature_param_rejects_a_key_from_a_different_feature() {
        let mut p = Feature::Mountains.default_params();
        // "riverWidth" is a real control key, just not one of Mountains'.
        assert_eq!(set_feature_param(&mut p, Feature::Mountains, "riverWidth", 10.0), Outcome::Rejected);
    }

    // ---- SculptEditor ----

    #[test]
    fn new_editor_starts_with_an_empty_draft_and_no_selection() {
        let e = SculptEditor::new(16, 12, None, None, 42);
        assert!(e.draft.is_empty());
        assert_eq!(e.selected(), None);
        assert!(e.points.is_empty());
        assert_eq!(e.seed, 42);
        assert!(!e.water.river_any, "no generated river state -> nothing locked yet");
    }

    #[test]
    fn new_editor_adopts_a_generated_river_lock() {
        let n = 16 * 12;
        let mut mask = vec![0u8; n];
        mask[5] = 1;
        let e = SculptEditor::new(16, 12, Some(mask), Some(vec![0.4f32; n]), 0);
        assert!(e.water.river_any);
    }

    #[test]
    fn new_editor_falls_back_to_empty_water_state_on_a_size_mismatch() {
        // Defensive: a caller passing mismatched-length arrays (should
        // never happen -- WorldState's own river_mask/river_floor are
        // always field-sized -- but this must not panic or index OOB).
        let e = SculptEditor::new(16, 12, Some(vec![1u8; 4]), Some(vec![0.5f32; 4]), 0);
        assert_eq!(e.water.river_mask.len(), 16 * 12);
    }

    #[test]
    fn commit_clears_the_draft_and_the_selection() {
        let mut e = SculptEditor::new(8, 8, None, None, 7);
        let stamp = SculptStamp::new(Feature::Mountains, 7, vec![cartalith_terrain::sculpt::Point::new(2.0, 2.0)], 0.5);
        e.selection.replace(e.draft.push(stamp));
        let mut field = vec![0.2f32; 64];
        e.commit(&mut field, 0.5, "test");
        assert!(e.draft.is_empty());
        assert_eq!(e.selected(), None);
    }

    // ---- the selection set (step one of the selection-sets ruling) ----

    fn editor_with_stamps(n: usize) -> SculptEditor {
        let mut e = SculptEditor::new(8, 8, None, None, 7);
        for k in 0..n {
            let stamp = SculptStamp::new(Feature::Mountains, 7, vec![Point::new(k as f64, 2.0)], 0.5);
            e.draft.push(stamp);
        }
        e
    }

    #[test]
    fn delete_stamp_clears_a_selection_on_the_removed_stamp() {
        // The rule `sculpt_delete_stamp` always had, unchanged.
        let mut e = editor_with_stamps(3);
        e.selection.replace(1);
        assert!(e.delete_stamp(1));
        assert_eq!(e.selected(), None);
        assert_eq!(e.draft.len(), 2);
    }

    #[test]
    fn delete_stamp_shifts_a_later_selection_down_instead_of_renaming_it() {
        // The hole the equal-case-only guard left: `PassBuffer::remove`
        // renumbers, so a selection at 2 with stamp 0 deleted used to keep
        // reporting 2 -- which by then was the stamp that had been at 3.
        let mut e = editor_with_stamps(4);
        e.selection.replace(2);
        assert!(e.delete_stamp(0));
        assert_eq!(e.selected(), Some(1), "the same logical stamp, one index down");
        assert_eq!(e.draft.len(), 3);
    }

    #[test]
    fn delete_stamp_leaves_an_earlier_selection_alone() {
        let mut e = editor_with_stamps(4);
        e.selection.replace(1);
        assert!(e.delete_stamp(3));
        assert_eq!(e.selected(), Some(1));
    }

    #[test]
    fn delete_stamp_rejects_an_out_of_range_index_without_touching_anything() {
        let mut e = editor_with_stamps(2);
        e.selection.replace(1);
        assert!(!e.delete_stamp(9));
        assert_eq!(e.draft.len(), 2);
        assert_eq!(e.selected(), Some(1));
    }

    #[test]
    fn a_multi_selection_of_stamps_survives_a_delete_below_it() {
        let mut e = editor_with_stamps(5);
        e.selection.set_from([1, 3, 4], 5);
        assert!(e.delete_stamp(0));
        assert_eq!(e.selection.sorted(), vec![0, 2, 3]);
        assert_eq!(e.selected(), Some(3), "the primary is still the last-added member");
    }

    #[test]
    fn select_all_then_delete_leaves_every_survivor_selected() {
        let mut e = editor_with_stamps(3);
        e.selection.select_all(3);
        assert_eq!(e.selection.sorted(), vec![0, 1, 2]);
        assert!(e.delete_stamp(1));
        assert_eq!(e.selection.sorted(), vec![0, 1], "2 became 1; the deleted one is gone");
    }
}
