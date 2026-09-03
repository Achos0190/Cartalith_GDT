//! CIVIL ▸ Landmarks — the rejected-candidate list's Godot surface.
//!
//! `LARGE_ITEM_RULINGS.md`, owner ruling 2026-08-31, the Landmark-funnel row:
//! *"Both halves — a crowding parameter on the placement pass **and** a
//! rejected-candidate coordinate list plus a new overlay layer to draw it.
//! `landmark_funnels()` returns eight scalars today and carries no coordinates,
//! so the dict grows."*
//!
//! The crowding half was already live before this file existed
//! (`LandmarkSettings::crowding`, `landmark_bridge::set_crowding`, and a wired
//! `Crowding` dial). This file is the second half's boundary: the list itself
//! is [`cartalith_civ::landmark::LandmarkResult::rejects`], produced by the
//! same walk that produces the funnel's counters, and everything here is
//! conversion over it.
//!
//! ## Why this is a `#[godot_api(secondary)]` block and not more of `lib.rs`
//!
//! The same reason `erode_bridge.rs`, `civ_military_bridge.rs` and
//! `label_bridge/generate.rs` are: gdext allows exactly one *primary*
//! `#[godot_api] impl WorldGen`, and every additional surface is a `secondary`
//! block in its own file. `landmark_bridge.rs` proper stays free of any `godot`
//! dependency — its own doc comment argues for that isolation and this file
//! does not disturb it, which is why the `godot`-facing half is a submodule
//! rather than an addition to it.
//!
//! ## Two bindings, and why the second exists
//!
//! - [`WorldGen::landmark_rejects`] — the whole record per row, for a panel or
//!   an inspector: `{kind, x, y, score, reason, needs_crowding}`.
//! - [`WorldGen::landmark_reject_points`] — the same positions as one
//!   `PackedVector2Array`, filtered by reason, for the renderer.
//!
//! The split is the boundary-cost rule this project learned expensively.
//! Without `experimental-threads` every `Dictionary`/`Array`/`GString`
//! operation routes through gdext's `ensure_main_thread()`, so a payload of N
//! dictionaries costs N × (a Dictionary allocation + six keyed writes, two of
//! them `GString`), while a `PackedVector2Array` of the same N is one buffer.
//! The POI pass froze this app for 4.14 s on exactly that difference and came
//! back at 0.39 s once its payload was made primitive. `map_overlay.gd` draws
//! dots and needs nothing but positions, so it gets the buffer; the record
//! shape is there for the surface that genuinely reads fields.
//!
//! Both are `&self` reads off the retained store, like `landmarks()` and
//! `landmark_funnels()` beside them — neither is reachable from the worker
//! thread `landmark_run()` uses, which marshals nothing and replies through
//! `landmark_last_run()` on the main thread.
//!
//! ## The list is bounded, and the funnel still holds the truth
//!
//! `cartalith_civ::landmark::REJECT_LIST_MAX_PER_KIND` caps the rows retained
//! per kind at the best-scoring 256. `landmark_funnels()`'s
//! `rejected_score`/`rejected_spacing`/`rejected_cap` are untouched and still
//! carry the real totals, so a caller can always say *"showing 256 of
//! 39 999"* without a second binding. See [`cartalith_civ::landmark::
//! LandmarkReject`] for why an unbounded list is a freeze waiting for one
//! slider drag.

use godot::prelude::*;

use cartalith_civ::landmark::LandmarkReject;

use crate::WorldGen;

/// One rejected candidate as GDScript sees it.
///
/// `needs_crowding` is **`0.0` for "does not apply"**, not a null: it is
/// carried only by a spacing rejection, and only when the blocking landmark is
/// not on the candidate's own cell. Zero is unambiguous as a sentinel because
/// `LandmarkSettings::crowding_in_force` clamps at `0.05`, so no real answer
/// can ever be it.
///
/// It is deliberately **not clamped to `crowding`'s own `3.0` ceiling**. A
/// value of `7.4` is the honest answer *"not at any setting this app offers"*;
/// clamping it would turn that into the false answer *"set it to 3"*.
fn reject_dict(e: &LandmarkReject) -> VarDictionary {
    let mut d = vdict! {
        "kind" => e.kind,
        "x" => e.x as i64,
        "y" => e.y as i64,
        "score" => e.score,
        "reason" => e.reason.as_str(),
    };
    // **Absent, never `0.0`.** This marshalled `needs_crowding.unwrap_or(0.0)`
    // until 2026-09-03, and `0.0` is a *plausible* crowding figure rather than
    // an obvious sentinel, so "no answer" and "an answer of zero" arrived at
    // GDScript identical. Measured on a real world at the time: 44 of 614
    // spacing rows carried the sentinel, and every one of them read as a
    // genuine measurement.
    //
    // Omitting the key pushes the distinction into `has()`, which is the same
    // idiom `civilization_workspace.gd`'s diagnostics card settled on the same
    // day for `harbour_scale` and `wall_spec` after the identical defect --
    // a defaulted value printed as though it had been measured.
    if let Some(n) = e.needs_crowding {
        d.set("needs_crowding", n);
    }
    d
}

/// `""` matches every reason; anything else matches
/// [`LandmarkRejectReason::as_str`] exactly.
///
/// An **unrecognised** reason matches nothing, which is the safe direction: a
/// caller that misspells `"spacing"` draws an empty layer rather than every
/// reject in one colour, and an empty layer is a visible bug where a wrong
/// colour is not.
fn reason_matches(e: &LandmarkReject, want: &str) -> bool {
    want.is_empty() || e.reason.as_str() == want
}

#[godot_api(secondary)]
impl WorldGen {
    /// The last `landmark_run()`'s rejected candidates — every cell the pass
    /// offered and did not place, with **where it is and why it lost**.
    ///
    /// Empty before any run, and after every `generate()`/`absorb()`/project
    /// open, exactly like `landmarks()` and for the same reason (the store's
    /// `invalidate()`).
    ///
    /// Row shape: `{kind, x, y, score, reason, needs_crowding}`.
    ///
    /// - `kind` — a `kinds()` key, the same vocabulary `landmark_funnels()`
    ///   uses.
    /// - `x`, `y` — **grid cells**, the space `landmarks()` uses, not the
    ///   continuous click coordinates the Icon tool uses. A renderer must map
    ///   these with `_cell_to_screen`.
    /// - `score` — the §17 suitability, `0..1`, comparable with a placed
    ///   landmark's own `score` within a kind.
    /// - `reason` — `"score"`, `"spacing"` or `"cap"`; machine keys, on the
    ///   same wire-format-not-wording rule `LandmarkLimit::as_str` states.
    ///   **There is deliberately no `"constraint"`**: that bucket is counted
    ///   per scanned cell inside each detector — "this cell is not a
    ///   waterfall" — and at the shell's default 2048×1311 it runs to
    ///   millions. `landmark_funnels()` reports it as a number, which is the
    ///   only shape it has ever been useful in.
    /// - `needs_crowding` — see [`reject_dict`].
    ///
    /// Ordered by class, then by kind in table order, then best-scoring first
    /// within a kind: the same order the pass walked, so the head of each
    /// kind's block is its near-misses.
    #[func]
    fn landmark_rejects(&self) -> Array<VarDictionary> {
        self.landmark_store
            .last
            .as_ref()
            .map_or_else(Array::new, |r| r.rejects.iter().map(reject_dict).collect())
    }

    /// The rejected candidates' positions alone, as one packed buffer — the
    /// renderer's half of [`Self::landmark_rejects`].
    ///
    /// `reason` filters: `""` for all, or `"score"` / `"spacing"` / `"cap"`.
    /// An unrecognised string returns an empty buffer rather than everything.
    ///
    /// Cells, as `Vector2(x, y)` — the same space `landmarks()` reports and
    /// `_cell_to_screen` consumes, kept as floats because that is what the
    /// draw call takes and rounding here would only push the conversion into
    /// GDScript.
    #[func]
    fn landmark_reject_points(&self, reason: GString) -> PackedVector2Array {
        let want = reason.to_string();
        self.landmark_store.last.as_ref().map_or_else(PackedVector2Array::new, |r| {
            r.rejects
                .iter()
                .filter(|e| reason_matches(e, &want))
                .map(|e| Vector2::new(e.x as f32, e.y as f32))
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_civ::landmark::LandmarkRejectReason;

    fn row(reason: LandmarkRejectReason, needs: Option<f64>) -> LandmarkReject {
        LandmarkReject { kind: "peak", x: 3, y: 4, score: 0.5, reason, needs_crowding: needs }
    }

    /// `""` is every row and a misspelling is none — the safe direction, since
    /// an empty layer is a visible bug and a wrongly-coloured one is not.
    #[test]
    fn the_reason_filter_passes_everything_on_empty_and_nothing_on_a_typo() {
        let e = row(LandmarkRejectReason::Spacing, Some(1.5));
        assert!(reason_matches(&e, ""));
        assert!(reason_matches(&e, "spacing"));
        assert!(!reason_matches(&e, "cap"));
        assert!(!reason_matches(&e, "Spacing"), "the wire format is lower-case and exact");
        assert!(!reason_matches(&e, "spacnig"));
    }

    /// Every reason the crate can emit must be addressable through this filter.
    /// A fourth variant added without a key here would silently become
    /// undrawable, so the coverage is asserted rather than assumed.
    #[test]
    fn every_reason_key_selects_its_own_row_and_no_other() {
        let all = [
            LandmarkRejectReason::Score,
            LandmarkRejectReason::Spacing,
            LandmarkRejectReason::Cap,
        ];
        for a in all {
            for b in all {
                assert_eq!(
                    reason_matches(&row(b, None), a.as_str()),
                    a == b,
                    "{} vs {}",
                    a.as_str(),
                    b.as_str()
                );
            }
        }
    }

    /// `0.0` is the sentinel, and it can never collide with a real answer:
    /// `crowding_in_force` clamps at `0.05`, and a spacing reject's figure is
    /// strictly greater than the crowding that produced it.
    #[test]
    fn an_absent_crowding_figure_becomes_zero_and_zero_is_not_a_real_answer() {
        assert_eq!(row(LandmarkRejectReason::Cap, None).needs_crowding.unwrap_or(0.0), 0.0);
        assert_eq!(
            row(LandmarkRejectReason::Spacing, Some(1.0625)).needs_crowding.unwrap_or(0.0),
            1.0625
        );
        let mut s = cartalith_civ::landmark::LandmarkSettings::default();
        s.crowding = 0.0;
        assert!(s.crowding_in_force() > 0.0, "zero must not be reachable as a real figure");
    }
}
