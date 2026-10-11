//! River-channel classes, read by **both** the Route tool's mixed-domain router
//! and the Journey Planner (Ruling CD item 1, owner, 2026-10-11).
//!
//! What it is for. `PLANNER_AUDIT_2026-10-11.md` Q3 measured that River
//! Transport on a real river was costed as walking: a 57-cell traced river came
//! out as 14 land stages, every leg a portage, because `jp_derive_stages` (the
//! reference's `_jpDeriveStages`, `Cartalith Gen1 v2.11.html` line 18981) only
//! ever calls a **lake** cell near shore a river, and its "current" is the
//! lake-bed gradient (`_jpRiverCondition`, line 18911), not the flow. Ruling CD
//! item 1 orders real channels navigable for River Transport, with direction
//! from the flow and speed scaled by discharge and order. This module holds the
//! few numbers that decision needs, beside [`crate::way_class`], so the planner
//! and the router read one definition of "navigable" and "how strong".
//!
//! What it shares rather than invents. Every threshold here is either a
//! reference constant this crate already ports or a labelled judgement:
//!
//! - [`NAVIGABLE_ORDER`] is the reference's own `order>=3` gate in
//!   `_civNavigableRiverDiscount` (v2.11 line 21441, "barge/raft-navigable").
//!   The router's mixed-domain discount ([`crate::civ_navigable_river_discount`])
//!   reads this same constant, so a cell the router discounts as navigable is a
//!   cell the planner costs as river, and the two cannot drift.
//! - [`magnitude`] is the log-compressed "how far above the river threshold"
//!   measure the router's ford/bridge cost already uses
//!   ([`crate::civ_river_crossing_cost`], `_civEnhancedTravelCost`, v2.11 line
//!   21483), shared by
//!   both callers through one function.
//! - The delta test reuses the router's low-lying/high-flow marshland rule
//!   ([`crate::civ_swamp_penalty`], `_civEnhancedTravelCost`'s "near-sea land
//!   with high flow accumulation = marshland" comment, v2.11 line 21478).
//! - The rapids/moderate gradients are the reference's own river-gradient
//!   constants [`crate::JP_RIVER_GRAD_STRONG`] / [`crate::JP_RIVER_GRAD_MILD`]
//!   (35 / 8 m/km, `_jpRiverCondition`'s "genuinely fast reach" / "real, felt
//!   current").
//! - The current's five bands reuse the sea condition's band edges
//!   [`crate::JP_SEA_BAND_MILD`] / [`crate::JP_SEA_BAND_STRONG`] (0.25 / 0.60),
//!   so both water conditions band one normalised score the same way. That
//!   reuse is a **judgement** (the reference has no flow-based river score to
//!   calibrate against).
//!
//! What it must never do. It must never make a cell a river for lack of data:
//! no network, no discharge or a non-positive threshold answers `None`, never a
//! zero-strength river (MISTAKES.md "never encode no value as a plausible
//! value"). It must never change the router's bit-exact output: the two router
//! functions that now read it compute exactly the expressions they computed
//! before. And it must never touch lakes: a lake cell stays the reference's
//! `Calm River` with the reference's elevation condition.

use crate::{JP_RIVER_GRAD_MILD, JP_RIVER_GRAD_STRONG, JP_SEA_BAND_MILD, JP_SEA_BAND_STRONG};

/// The Strahler order from which a traced channel is navigable by a river
/// hull: the reference's `_civNavigableRiverDiscount` gate (`order>=3`, v2.11
/// line 21441, "barge/raft-navigable"). One constant for the router's discount
/// and the planner's river classification.
pub const NAVIGABLE_ORDER: i16 = 3;

/// The marshland rule's "low-lying" margin, in normalised `field` units above
/// sea level (v2.11 line 21479, `_civEnhancedTravelCost`; ported as
/// `civ_swamp_penalty`). A channel cell this low **and** carrying more than
/// [`DELTA_FLOW_MULT`] times the river threshold is a delta reach.
pub const DELTA_LOWLAND_DH: f64 = 0.06;

/// The marshland rule's flow multiple of the river threshold (same source as
/// [`DELTA_LOWLAND_DH`]).
pub const DELTA_FLOW_MULT: f64 = 8.0;

/// The router's log-compressed river magnitude, exactly as
/// `civ_river_crossing_cost` has always computed it: `min(1, ln(flow/thresh +
/// 1) / 5)` (v2.11 line 21483). Kept as one expression so the router's bit-exact output cannot
/// move; [`magnitude`] is the guarded planner entry point.
pub(crate) fn magnitude_raw(flow: f64, thresh: f64) -> f64 {
    (((flow / thresh) + 1.0).ln() / 5.0).min(1.0)
}

/// How strong a river's current is, on `[0, 1]`: [`magnitude_raw`] of the
/// cell's discharge against the river threshold. Discharge `0` gives `0`
/// (a still channel, which bands as `Neutral`); the log saturates at `1` once
/// the discharge reaches about `e^5 - 1 ≈ 147` times the threshold.
///
/// `None` -- never a plausible strength -- when the threshold is not a
/// positive finite number or the discharge is not finite: there is then no
/// measurement to scale by. Negative discharge (not produced by any flow
/// accumulator in this workspace) is clamped to `0`.
pub fn magnitude(flow: f64, thresh: f64) -> Option<f64> {
    if !(thresh.is_finite() && thresh > 0.0) || !flow.is_finite() {
        return None;
    }
    Some(magnitude_raw(flow.max(0.0), thresh))
}

/// The `JP_TERRAIN.river` row (reference v2.11 line 17940: Calm 1.00, Moderate
/// 0.90, Shallows 0.75, Delta 0.65, Rapids 0.50) for one navigable channel cell.
///
/// `order` is the cell's Strahler order (callers only ask for cells at or above
/// [`NAVIGABLE_ORDER`]); `grad_m_per_km` the bed drop from the cell to its
/// receiver, `None` when the cell has no receiver (then no gradient rule
/// applies -- an unknown gradient is not a flat one); `delta` the marshland
/// test ([`DELTA_LOWLAND_DH`], [`DELTA_FLOW_MULT`]).
///
/// The order, first match wins (a **judgement** mapping the reference's row
/// names onto quantities this engine measures; the reference itself never
/// derives these rows, `PLANNER_AUDIT_2026-10-11.md` Q2 finding 4):
///
/// 1. `River with Rapids` where the bed falls at least
///    [`JP_RIVER_GRAD_STRONG`] (35 m/km). The rows' own vessel rules make this
///    load-bearing: River Barge and River Galley list it as `invalidWater`.
/// 2. `River Delta` where the marshland rule holds.
/// 3. `River with Shallows` at the navigability floor itself
///    ([`NAVIGABLE_ORDER`]): the reference's navigable discount is weakest
///    there (`1 - 0.35 * (order - 2) / 4`, full only from order 6).
/// 4. `Moderate River` one order up, or wherever the bed falls at least
///    [`JP_RIVER_GRAD_MILD`] (8 m/km).
/// 5. `Calm River` otherwise: a big, gentle river.
pub fn channel_terrain(order: i16, grad_m_per_km: Option<f64>, delta: bool) -> &'static str {
    if grad_m_per_km.is_some_and(|g| g >= JP_RIVER_GRAD_STRONG) {
        "River with Rapids"
    } else if delta {
        "River Delta"
    } else if order <= NAVIGABLE_ORDER {
        "River with Shallows"
    } else if order == NAVIGABLE_ORDER + 1 || grad_m_per_km.is_some_and(|g| g >= JP_RIVER_GRAD_MILD) {
        "Moderate River"
    } else {
        "Calm River"
    }
}

/// The `JP_ROUTE.river` condition (reference v2.11 line 17956: Strong
/// Downstream 1.40, Mild Downstream 1.30, Neutral 1.00, Mild Upstream 0.80,
/// Strong Upstream 0.55) for a channel stage's flow score.
///
/// `score` is the length-weighted mean, over the stage's channel points, of
/// `cos(travel, flow) x magnitude` -- `+1` straight down a saturated river,
/// `-1` straight up it, `0` across it or on still water. Banded at the sea
/// condition's edges (see the module doc: a judgement). A NaN score (which no
/// caller produces) falls through every comparison to `Neutral`.
pub fn condition_from_score(score: f64) -> &'static str {
    if score >= JP_SEA_BAND_STRONG {
        "Strong Downstream"
    } else if score >= JP_SEA_BAND_MILD {
        "Mild Downstream"
    } else if score <= -JP_SEA_BAND_STRONG {
        "Strong Upstream"
    } else if score <= -JP_SEA_BAND_MILD {
        "Mild Upstream"
    } else {
        "Neutral"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: the navigability gate stays the reference's `order>=3`
    /// (`_civNavigableRiverDiscount`, v2.11 line 21441), pinned as a literal.
    #[test]
    fn navigable_order_is_the_reference_gate() {
        assert_eq!(NAVIGABLE_ORDER, 3);
        assert_eq!((DELTA_LOWLAND_DH, DELTA_FLOW_MULT), (0.06, 8.0));
    }

    /// Protects: the strength scale. Zero discharge is a still river (0, not a
    /// missing one); the log saturates at 1; no threshold or a NaN discharge is
    /// no measurement (`None`), never a plausible strength. Literals computed
    /// by hand: `ln(2)/5`, `ln(4)/5`.
    #[test]
    fn magnitude_scales_with_discharge_and_refuses_missing_data() {
        assert_eq!(magnitude(0.0, 10.0), Some(0.0));
        assert_eq!(magnitude(10.0, 10.0), Some(0.138_629_436_111_989_06));
        assert_eq!(magnitude(30.0, 10.0), Some(0.277_258_872_223_978_1));
        assert_eq!(magnitude(1.0e9, 10.0), Some(1.0));
        assert_eq!(magnitude(-5.0, 10.0), Some(0.0));
        assert_eq!(magnitude(10.0, 0.0), None);
        assert_eq!(magnitude(10.0, f64::INFINITY), None);
        assert_eq!(magnitude(f64::NAN, 10.0), None);
    }

    /// Protects: the terrain mapping, one literal row per rule and the
    /// first-match order (rapids beat delta beat order).
    #[test]
    fn channel_terrain_maps_order_gradient_and_delta() {
        assert_eq!(channel_terrain(3, Some(1.0), false), "River with Shallows");
        assert_eq!(channel_terrain(4, Some(1.0), false), "Moderate River");
        assert_eq!(channel_terrain(5, Some(1.0), false), "Calm River");
        assert_eq!(channel_terrain(7, None, false), "Calm River");
        assert_eq!(channel_terrain(6, Some(8.0), false), "Moderate River");
        assert_eq!(channel_terrain(6, Some(7.99), false), "Calm River");
        assert_eq!(channel_terrain(6, Some(35.0), false), "River with Rapids");
        assert_eq!(channel_terrain(6, Some(34.99), false), "Moderate River");
        assert_eq!(channel_terrain(6, Some(1.0), true), "River Delta");
        assert_eq!(channel_terrain(3, Some(1.0), true), "River Delta");
        assert_eq!(channel_terrain(6, Some(40.0), true), "River with Rapids");
    }

    /// Protects: the five-step band, both edges of each step as literals, and
    /// the symmetry that makes "upstream" the mirror of "downstream".
    #[test]
    fn condition_bands_the_score_in_five_steps() {
        assert_eq!(condition_from_score(1.0), "Strong Downstream");
        assert_eq!(condition_from_score(0.60), "Strong Downstream");
        assert_eq!(condition_from_score(0.59), "Mild Downstream");
        assert_eq!(condition_from_score(0.25), "Mild Downstream");
        assert_eq!(condition_from_score(0.24), "Neutral");
        assert_eq!(condition_from_score(0.0), "Neutral");
        assert_eq!(condition_from_score(-0.24), "Neutral");
        assert_eq!(condition_from_score(-0.25), "Mild Upstream");
        assert_eq!(condition_from_score(-0.59), "Mild Upstream");
        assert_eq!(condition_from_score(-0.60), "Strong Upstream");
        assert_eq!(condition_from_score(f64::NAN), "Neutral");
    }
}
