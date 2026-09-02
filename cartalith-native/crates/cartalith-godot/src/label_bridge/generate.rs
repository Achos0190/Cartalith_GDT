//! CARTO ▸ Labels — the generated labelling pass's Godot surface.
//!
//! `LARGE_ITEM_RULINGS.md`, owner ruling 2026-08-31: *"All three steps, in
//! order — (1) a `label_class` field on `MapLabel`; (2) a generated labelling
//! pass emitting per-class placements — this is what makes the drawn-count
//! column real; (3) a per-class typography record carrying size/halo/tracking.
//! Note `halo` and `tracking` do not exist anywhere in the engine today, so
//! step 3 creates them."*
//!
//! Steps 1 and 3 live in `cartalith_civ::labels` ([`LabelClass`],
//! [`LabelTypography`]); step 2's algorithm lives there too
//! ([`label_candidates`], [`generate_labels`]). This file is only the
//! `Variant` conversion over them, plus the assembly of a
//! [`LabelWorld`] out of `WorldGen`'s own fields, which is the one thing that
//! cannot live in `cartalith-civ` — it reads `CivData` and the landmark store.
//!
//! ## The backlog note this closes, and the one word of it that was wrong
//!
//! `OUTSTANDING_WORK.md` §2.2: *"Halo and tracking do not exist in the engine
//! today."* Verified 2026-09-02 and true for **tracking**: `grep -ri tracking`
//! over `crates/` matched only the word in prose and in `usage tracking`.
//! **Halo is more precisely a half-truth**: a halo *stroke width* did exist
//! (`labels::arc_label_line_width`, `max(1, size_px * 0.16)`, golden-pinned by
//! `golden_parity_labels.rs`) and `map_overlay.gd` restates it as `maxi(1,
//! int(font_px * 0.16))`. What did not exist is a halo anybody can *set* —
//! a per-class figure the design states five of. That is what
//! [`LabelTypography::halo`] adds, and it does not touch the golden-pinned
//! function, which still describes the reference's own labels.
//!
//! ## Four `#[func]`s, and what is deliberately not among them
//!
//! - [`WorldGen::label_class_table`] — the five class records and the three
//!   slider domains. The panel's own transcription of these is now
//!   redundant, which is the point: one table, in the engine.
//! - [`WorldGen::labels_generate`] — run the pass, retain it, report per-class
//!   counts.
//! - [`WorldGen::labels_render_list`] — every label the map should draw,
//!   generated and hand-placed, each with its class's resolved type spec.
//! - [`WorldGen::label_set_class`] — reclass one hand-placed label.
//!
//! **The collision culler is in the pass**, by the same ruling:
//! *"Measure-and-suppress rides in the same pass that places labels. Explicitly
//! not a standalone job."* So it is `cartalith_civ::labels::generate_labels`
//! that suppresses, and `suppressed` on every row here is now a real number.
//! What crosses at this boundary is the one input the engine cannot have: the
//! mean glyph advance of the font the shell actually draws with, sent as
//! `cull.advance_ratio` and defaulted to
//! [`cartalith_civ::labels::DEFAULT_LABEL_ADVANCE_RATIO`] when the caller does
//! not measure it. Wiring the culler *itself* at this boundary was the other
//! option and is the wrong one: the counters would then live apart from the
//! ranking that decides who wins.

use godot::prelude::*;

use cartalith_civ::labels::{
    label_candidates, LabelClass, LabelCullMetrics, LabelTypography, LabelWorld, LAKE_LABEL_MIN_CELLS,
    LABEL_CLASSES, LABEL_CLASS_HALO_RANGE, LABEL_CLASS_SIZE_RANGE, LABEL_CLASS_TRACKING_RANGE,
    LABEL_TYPOGRAPHY_DEFAULTS,
};

use crate::WorldGen;

/// One class's record as GDScript sees it.
///
/// `halo_em` and `tracking_em` are multipliers of the **rendered** font size,
/// not pixel figures, and that is the seam rather than an omission.
/// `map_overlay.gd` sizes a label with its own `_label_font_px` — a different
/// formula from `labels::label_font_size`, disclosed at length in both files —
/// so the engine cannot resolve a pixel width here without asserting a size it
/// does not own. [`LabelTypography::halo_px`] is the authority for what those
/// multipliers mean; the renderer's one-line restatement of it is marked at its
/// own call site.
fn class_dict(class: LabelClass, t: &LabelTypography) -> VarDictionary {
    let (halo_em, tracking_em) = class_multipliers(t);
    vdict! {
        "key" => class.key(),
        "label" => class.label(),
        "size" => t.size,
        "halo" => t.halo,
        "tracking" => t.tracking,
        "italic" => t.italic,
        "ink" => t.ink,
        "halo_em" => halo_em,
        "tracking_em" => tracking_em,
    }
}

/// `(halo_em, tracking_em)` — the two numbers the renderer multiplies by its
/// own font px.
///
/// The halo guard is not decorative: a class whose `size` was driven to zero
/// would otherwise hand GDScript an infinity, and an infinite outline width
/// takes the draw call, not the value, down. Its own zero case is
/// [`LabelTypography::halo_px`]'s: `halo == 0` is the slider's "off" end.
///
/// Its own function so it is testable without a Godot runtime — the reason
/// every other bridge in this crate keeps its logic off the `Dictionary`.
fn class_multipliers(t: &LabelTypography) -> (f64, f64) {
    let halo_em = if t.size > 0.0 && t.halo > 0.0 { t.halo / t.size } else { 0.0 };
    (halo_em, t.tracking)
}

fn count_dict(c: &cartalith_civ::labels::LabelClassCount) -> VarDictionary {
    vdict! {
        "key" => c.class.key(),
        "label" => c.class.label(),
        "available" => c.available as i64,
        "drawn" => c.drawn as i64,
        "over_cap" => c.over_cap as i64,
        "suppressed" => c.suppressed as i64,
    }
}

/// `options["cull"]` folded into the retained setting.
///
/// Three-state on purpose, and the middle state is the one that matters: an
/// **absent** `cull` key keeps whatever the last run used (every key in
/// `options` behaves that way, so a panel can push one dial without restating
/// the rest), `{"on": false}` turns culling off, and any other shape turns it
/// on with the metrics given. An absent `advance_ratio` inside a present `cull`
/// keeps the ratio already in force rather than snapping back to the shipped
/// estimate — the shell measures its font once and a later toggle must not
/// silently discard that measurement.
///
/// Its own function, off the `Dictionary`, for the reason
/// [`class_multipliers`] gives: it is testable without a Godot runtime.
fn fold_cull(current: Option<LabelCullMetrics>, on: bool, advance_ratio: Option<f64>) -> Option<LabelCullMetrics> {
    if !on {
        return None;
    }
    let mut m = current.unwrap_or_default();
    if let Some(r) = advance_ratio.filter(|r| *r > 0.0) {
        m.advance_ratio = r;
    }
    Some(m)
}

fn as_f64(d: &VarDictionary, key: &str) -> Option<f64> {
    d.get(key).and_then(|v| v.try_to::<f64>().ok()).filter(|f| f.is_finite())
}

impl WorldGen {
    /// Everything the candidate sweep reads, borrowed out of this world.
    ///
    /// Every source degrades to an empty slice rather than to an absent class:
    /// a world generated before the civilisation layer ran, or a loaded save
    /// (which carries no civilisation layer at all — `CivData`'s own doc
    /// comment), yields a run whose counts are honestly zero, which is a
    /// different and more useful answer than a refusal.
    fn label_world(&self, want_water: bool, lake_min_cells: usize) -> LabelWorld<'_> {
        let civ = self.civ.as_ref();
        LabelWorld {
            continents: civ.map_or(&[][..], |c| c.continents.as_slice()),
            provinces: civ.map_or(&[][..], |c| c.province_list.as_slice()),
            settlements: civ.map_or(&[][..], |c| c.settlements.as_slice()),
            landmarks: self.landmark_store.last.as_ref().map_or(&[][..], |r| r.landmarks.as_slice()),
            water: if want_water { civ.map(|c| c.water_bodies.as_slice()) } else { None },
            gw: self.gw.max(0) as usize,
            gh: self.gh.max(0) as usize,
            lake_min_cells,
        }
    }
}

#[godot_api(secondary)]
impl WorldGen {
    /// The five label classes, their type specs and the three slider domains.
    ///
    /// Available before any `generate()` — it is a design table, not world
    /// data, so a panel can build itself from it at launch. `classes` carries
    /// the values **currently in force** once a world exists (a caller's own
    /// overrides included); `defaults` always carries the shipped ones, so a
    /// "reset" needs no second call.
    ///
    /// Returned shape:
    /// - `classes` / `defaults` — five rows of `key`, `label`, `size`, `halo`,
    ///   `tracking`, `italic`, `ink`, `halo_em`, `tracking_em`.
    /// - `size_range` / `halo_range` / `tracking_range` — `[min, max]`.
    #[func]
    fn label_class_table(&self) -> VarDictionary {
        let live = self.labels.as_ref().map_or(&LABEL_TYPOGRAPHY_DEFAULTS, |b| &b.typography);
        let rows = |table: &[LabelTypography; 5]| -> Array<VarDictionary> {
            LABEL_CLASSES.into_iter().map(|c| class_dict(c, &table[c.index()])).collect()
        };
        let mut out = vdict! { "lake_min_cells" => LAKE_LABEL_MIN_CELLS as i64 };
        out.set("classes", &rows(live));
        out.set("defaults", &rows(&LABEL_TYPOGRAPHY_DEFAULTS));
        for (key, r) in [
            ("size_range", LABEL_CLASS_SIZE_RANGE),
            ("halo_range", LABEL_CLASS_HALO_RANGE),
            ("tracking_range", LABEL_CLASS_TRACKING_RANGE),
        ] {
            out.set(key, &varray![r.0, r.1]);
        }
        out
    }

    /// Run the generated labelling pass over this world and retain its output.
    ///
    /// `options` is read key by key and every key is optional; an absent key
    /// keeps whatever the last run used, so a panel can push one dial without
    /// restating the other fourteen. Recognised keys, all class-keyed by
    /// [`LabelClass::key`] (`continental`/`region`/`settlement`/`water`/
    /// `landmark`):
    ///
    /// - `typography` — `{class: {size, halo, tracking}}`, each clamped to its
    ///   own domain by [`LabelTypography::set_field`].
    /// - `enabled` — `{class: bool}`.
    /// - `max_per_class` — `{class: int}`, `0` for no cap. Anything a cap drops
    ///   is reported as `over_cap`, never hidden.
    /// - `lake_min_cells` — the floor under [`LAKE_LABEL_MIN_CELLS`].
    /// - `cull` — `{on: bool, advance_ratio: float}`. Collision culling: a
    ///   generated label whose estimated box hits one already placed, or hits
    ///   any hand-placed label, is suppressed and counted as `suppressed`.
    ///   `advance_ratio` is the caller's own font's mean glyph advance as a
    ///   fraction of the font size, which is the one measurement the engine
    ///   cannot take for itself; see [`fold_cull`] for what an absent key does.
    ///
    /// **`suppressed` is a real number now, and `0` still has two meanings** —
    /// culling off, or culling on and nothing overlapped. The caller knows
    /// which, because the caller set `cull.on`.
    ///
    /// Returns `{ok, classes, total, elapsed_ms}`, `classes` being one
    /// [`count_dict`] per class in drawing order. `ok` is `false` only before
    /// any `generate()` call, with `reason` saying so — a world that exists but
    /// has nothing to name returns `ok: true` and five zeroed rows, because
    /// "nothing to label" is an answer.
    ///
    /// **The water class is the one that costs a sweep.** Naming lakes needs a
    /// connected-component pass over the whole `build_water_bodies` raster
    /// (`labels::lake_features`), so it is skipped outright when that class is
    /// disabled — which is why a panel should re-run this on a slider's
    /// *release* rather than on every drag sample.
    #[func]
    fn labels_generate(&mut self, options: VarDictionary) -> VarDictionary {
        if self.labels.is_none() {
            let mut out = vdict! {
                "ok" => false,
                "reason" => "Labels need a generated world: label classes are placed on continents, provinces, \
                             settlements, lakes and landmarks, and none of those exist before generate().",
                "total" => 0i64,
                "elapsed_ms" => 0i64,
            };
            out.set("classes", &Array::<VarDictionary>::new());
            return out;
        }

        // ---- fold `options` into the retained settings ----
        let mut lake_min_cells = LAKE_LABEL_MIN_CELLS;
        {
            let bridge = self.labels.as_mut().expect("checked above");
            if let Some(t) = options.get("typography").and_then(|v| v.try_to::<VarDictionary>().ok()) {
                for class in LABEL_CLASSES {
                    let Some(row) = t.get(class.key()).and_then(|v| v.try_to::<VarDictionary>().ok()) else {
                        continue;
                    };
                    let slot = &mut bridge.typography[class.index()];
                    for field in ["size", "halo", "tracking"] {
                        if let Some(v) = as_f64(&row, field) {
                            slot.set_field(field, v);
                        }
                    }
                }
            }
            if let Some(e) = options.get("enabled").and_then(|v| v.try_to::<VarDictionary>().ok()) {
                for class in LABEL_CLASSES {
                    if let Some(on) = e.get(class.key()).and_then(|v| v.try_to::<bool>().ok()) {
                        bridge.gen_settings.enabled[class.index()] = on;
                    }
                }
            }
            if let Some(m) = options.get("max_per_class").and_then(|v| v.try_to::<VarDictionary>().ok()) {
                for class in LABEL_CLASSES {
                    if let Some(n) = m.get(class.key()).and_then(|v| v.try_to::<i64>().ok()) {
                        bridge.gen_settings.max_per_class[class.index()] = n.max(0) as usize;
                    }
                }
            }
            if let Some(n) = options.get("lake_min_cells").and_then(|v| v.try_to::<i64>().ok()) {
                lake_min_cells = n.max(1) as usize;
            }
            if let Some(c) = options.get("cull").and_then(|v| v.try_to::<VarDictionary>().ok()) {
                let on = c.get("on").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(true);
                bridge.gen_settings.cull = fold_cull(bridge.gen_settings.cull, on, as_f64(&c, "advance_ratio"));
            }
        }

        let want_water = self.labels.as_ref().expect("checked above").gen_settings.enabled[LabelClass::Water.index()];
        let t0 = std::time::Instant::now();
        // Assembled before the mutable borrow: `label_world` reads `self.civ`
        // and `self.landmark_store`, and `regenerate` writes `self.labels`.
        let candidates = label_candidates(&self.label_world(want_water, lake_min_cells));
        let bridge = self.labels.as_mut().expect("checked above");
        let g = bridge.place(&candidates);
        let classes: Array<VarDictionary> = g.counts.iter().map(count_dict).collect();
        let total = g.labels.len() as i64;

        let mut out = vdict! {
            "ok" => true,
            "reason" => "",
            "total" => total,
            "elapsed_ms" => t0.elapsed().as_millis() as i64,
        };
        out.set("classes", &classes);
        out
    }

    /// The last run's per-class counts, without re-running it. `[]` before the
    /// first [`Self::labels_generate`] — never a row of zeroes, because "has
    /// not run" and "ran and found nothing" are different claims and the
    /// panel draws them differently (`--` against `0`).
    #[func]
    fn labels_generated_counts(&self) -> Array<VarDictionary> {
        self.labels
            .as_ref()
            .and_then(|b| b.generated.as_ref())
            .map_or_else(Array::new, |g| g.counts.iter().map(count_dict).collect())
    }

    /// Drop the generated run, leaving hand-placed labels alone.
    #[func]
    fn labels_clear_generated(&mut self) {
        if let Some(b) = self.labels.as_mut() {
            b.invalidate_generated();
        }
    }

    /// Every label the map should draw: the generated run first, hand-placed
    /// labels over it.
    ///
    /// Each row is `label_get`'s own shape (`x`, `y`, `text`, `angle`, `arc`,
    /// `size`, `size_mode`, `font`, `color`) plus:
    ///
    /// - `class` — a [`LabelClass::key`].
    /// - `halo_em` / `tracking_em` — the class's type spec as multipliers of
    ///   the rendered font size (see [`class_dict`]).
    /// - `italic`.
    /// - `generated` — `true` for a pass output, `false` for a hand-placed
    ///   label. The Label tool's own hit-testing and index-based editing
    ///   (`label_hit_test`, `label_select`, `label_set`) still see **only**
    ///   `label_list()`, so an index from this list is not one of theirs; that
    ///   is why the flag is here rather than an index.
    ///
    /// A hand-placed label keeps its own `color`, which is a user choice; a
    /// generated one carries its class's ink because it never had one.
    #[func]
    fn labels_render_list(&self) -> Array<VarDictionary> {
        let Some(bridge) = self.labels.as_ref() else { return Array::new() };
        bridge
            .render_order()
            .map(|(lb, generated)| {
                let t = &bridge.typography[lb.class.index()];
                let (halo_em, tracking_em) = class_multipliers(t);
                let mut d = crate::label_dict(lb);
                d.set("class", lb.class.key());
                d.set("halo_em", halo_em);
                d.set("tracking_em", tracking_em);
                d.set("italic", t.italic);
                d.set("generated", generated);
                d
            })
            .collect()
    }

    /// One hand-placed label's [`LabelClass::key`], or `""` for an
    /// out-of-range index.
    ///
    /// Its own call rather than a tenth key on `label_get`: `label_dict` is
    /// shared with `label_get`/`label_list` in `lib.rs`, which two other agents
    /// are editing on this date, and one string for one index is cheaper than
    /// the coordination.
    #[func]
    fn label_class_of(&self, index: i64) -> GString {
        if index < 0 {
            return GString::new();
        }
        self.labels
            .as_ref()
            .and_then(|b| b.labels.get(index as usize))
            .map_or_else(GString::new, |lb| GString::from(lb.class.key()))
    }

    /// Reclass one hand-placed label. `false` for an out-of-range index or an
    /// unrecognised key.
    ///
    /// Commits immediately and is **not** revertible by `label_cancel_edit`,
    /// for the reason [`cartalith_civ::labels::MapLabel::class`] documents:
    /// the seven fields `_civLabelEditSnapshot` reverts are the reference's,
    /// and this is not one of them.
    #[func]
    fn label_set_class(&mut self, index: i64, key: GString) -> bool {
        let Some(class) = LabelClass::from_key(&key.to_string()) else { return false };
        let Some(bridge) = self.labels.as_mut() else { return false };
        if index < 0 {
            return false;
        }
        let Some(lb) = bridge.labels.get_mut(index as usize) else { return false };
        lb.class = class;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every key `labels_generate`'s `options` reads and every key
    /// `labels_render_list` writes, pinned as strings.
    ///
    /// These cross the gdext boundary and nothing on the GDScript side is
    /// type-checked against them, so a rename here is a silent break there.
    /// This is the cheapest possible guard against that, and it is the same
    /// reason the class keys themselves are pinned in `cartalith-civ`.
    #[test]
    fn the_class_keys_the_shell_sends_are_the_ones_the_engine_answers_to() {
        let keys: Vec<&str> = LABEL_CLASSES.into_iter().map(|c| c.key()).collect();
        assert_eq!(keys, vec!["continental", "region", "settlement", "water", "landmark"]);
        for k in &keys {
            assert!(LabelClass::from_key(k).is_some());
        }
    }

    /// The renderer multiplies these by its own font px, so they have to come
    /// back out as the design's own figures at the class's nominal size.
    ///
    /// **A `Dictionary` cannot be built without a live Godot runtime** (gdext
    /// panics with "Godot engine not available" in a unit test), so the shape
    /// under test is the pure function both `class_dict` and
    /// `labels_render_list` go through, not the dictionary itself. That is the
    /// same split every other bridge in this crate makes.
    #[test]
    fn the_multipliers_restate_the_design_figures_at_the_nominal_size() {
        for class in LABEL_CLASSES {
            let t = cartalith_civ::labels::label_typography_default(class);
            let (halo_em, tracking_em) = class_multipliers(&t);
            assert!((halo_em * t.size - t.halo).abs() < 1e-12, "{} halo", class.key());
            assert_eq!(tracking_em, t.tracking, "{} tracking", class.key());
        }
        // Continental: 2.5 px of halo on a 26 px glyph.
        let (halo_em, _) = class_multipliers(&cartalith_civ::labels::label_typography_default(LabelClass::Continental));
        assert!((halo_em - 2.5 / 26.0).abs() < 1e-12);
    }

    /// The three states `cull` folds into, and the one that would be easy to
    /// get wrong: turning the toggle back on must not throw away the font
    /// measurement the shell took.
    #[test]
    fn folding_the_cull_option_keeps_a_measured_ratio_across_a_toggle() {
        let measured = LabelCullMetrics { advance_ratio: 0.4271, ..Default::default() };
        let off = fold_cull(Some(measured), false, None);
        assert!(off.is_none());
        // Off, then on with no ratio in the payload: the *last known* metrics
        // are gone with the `None`, so this is the shipped estimate again --
        // which is why the panel sends the ratio on every run, and why this
        // test states the consequence rather than pretending otherwise.
        assert_eq!(fold_cull(off, true, None).unwrap().advance_ratio, cartalith_civ::labels::DEFAULT_LABEL_ADVANCE_RATIO);
        // On, with a ratio: stored.
        assert_eq!(fold_cull(off, true, Some(0.4271)).unwrap().advance_ratio, 0.4271);
        // On again with no ratio, from a state that already had one: kept.
        assert_eq!(fold_cull(Some(measured), true, None).unwrap().advance_ratio, 0.4271);
        // A nonsensical ratio is refused rather than stored: a zero or negative
        // advance makes every box zero-width and culls nothing at all, which
        // would look exactly like the toggle being broken.
        assert_eq!(fold_cull(Some(measured), true, Some(0.0)).unwrap().advance_ratio, 0.4271);
        assert_eq!(fold_cull(Some(measured), true, Some(-1.0)).unwrap().advance_ratio, 0.4271);
    }

    #[test]
    fn a_degenerate_spec_hands_the_renderer_zero_rather_than_an_infinity() {
        let zero_size = LabelTypography { size: 0.0, halo: 2.0, tracking: 0.1, italic: false, ink: "#000000" };
        assert_eq!(class_multipliers(&zero_size).0, 0.0);
        let no_halo = LabelTypography { size: 18.0, halo: 0.0, tracking: 0.1, italic: false, ink: "#000000" };
        assert_eq!(class_multipliers(&no_halo).0, 0.0, "the slider's own off end stays off");
    }
}
