//! **Not the reference** -- deep burgage plots, a prototype (owner Ruling FE
//! item 3, 2026-11-15: *"Deep burgage plots with backland are built behind
//! their own switch, measured and shown in frames before the owner decides"*;
//! `TOWN_GENERATOR_JUDGEMENT.md` change 4). Behind `GenOpts::burgage`
//! (`WorldParams::urban_burgage`), which is **off in both
//! `WorldParams::defaults` and `params::defaults`** until the owner rules on
//! the measurements (`DECISIONS.md` §7p).
//!
//! What it changes, and only on the organic plan of a town that is not a
//! street village (`generate_pass`):
//! 1. **Streets further apart** ([`rules`]): `grow`'s parallel-street spacing
//!    [`STREET_SPACING_M`] (two plot depths) and a longer segment median
//!    [`SEGMENT_MEDIAN_M`]; its spacing test then looks as far as the spacing
//!    (`Graph::edges_within`), since the reference's query reaches only about
//!    one 26 m index cell.
//! 2. **Deeper lots** (`blocks::build_parcels_with`): a block-uniform target
//!    depth around [`DEPTH_CORE_M`] near the market and [`DEPTH_M`] beyond,
//!    capped at [`DEPTH_MAX_M`], each lot reaching [`WAIST`] of the way across
//!    its block (the reference: 22 / 30 m, cap 46 m, 0.42).
//! 3. **Fewer lanes cut through blocks** (`cleanup::lane_pass`): a face is
//!    split only from [`LANE_MIN_AREA_M2`] up, so a burgage block is not
//!    halved again by the interior lane pass.
//!
//! The street budget a head is **not** changed, though the judgement proposed
//! "the street budget per head reduced to match": measured 2026-11-17, with
//! the wider spacing `grow` already places less street than its budget (about
//! 10.0 of 12.6 km on two synthetic towns of 6 000) and the town houses fewer,
//! so there is nothing to give back, and a larger budget (x1.3, x1.6) placed
//! no more street (`STATUS.md`, "Deep burgage plots, a prototype").
//!
//! The house stays at the street (`districts::build_buildings`' main range,
//! at most about 11.5 m deep) and the burgage cycle's rear wing and
//! outbuilding (M-BLD-6), whose chance already grows with the lot's age, fill
//! the backland: nothing in the building grammar changes.
//!
//! Must never change a layout with the switch off: every value here is read
//! only behind `GenOpts::burgage`.

use crate::rules::Rules;

/// Parallel-street spacing, metres: room for two plot depths back to back.
/// **A labelled judgement** on `TOWN_GENERATOR_JUDGEMENT.md` change 4's
/// *"about two plot depths, 60 to 70 m"* (the reference's minimum is 24 m).
pub const STREET_SPACING_M: f64 = 60.0;

/// `grow`'s segment-length median, metres. **A labelled judgement**: the value
/// `MARKET_TOWN_RULES` takes for its long, gently curving cross-streets
/// (reference default 56 m), so blocks run long along their streets.
pub const SEGMENT_MEDIAN_M: f64 = 80.0;

/// The target lot depth near the market (within 160 m), metres. The reference
/// plats 22 m there. **A labelled judgement** at Freiburg's sourced 16 x 32 m
/// (`MEDIEVAL_TOWN_GROWTH_RESEARCH.md`, burgage plots), the shallowest of the
/// sourced plots, where land is dearest.
pub const DEPTH_CORE_M: f64 = 30.0;

/// The target lot depth elsewhere, metres (reference 30 m). **A labelled
/// judgement** between Freiburg's 32 m and the 100-140 m of Alresford and
/// Burford, held to what a 60 m street spacing can plat from both sides.
pub const DEPTH_M: f64 = 42.0;

/// The deepest target a block draws, metres (reference 46 m). **A labelled
/// judgement**: deep enough for the research's 100 m plots where a block is
/// that wide, and at a 16 m frontage within `build_parcels`' 2 600 m2 lot cap.
pub const DEPTH_MAX_M: f64 = 90.0;

/// The share of the way across its block a lot may reach, along its edge
/// normal and its corner bisectors (reference 0.42, which leaves a strip
/// unplatted down the middle of every block). **A labelled judgement**: lots
/// from opposite streets meet near the block's middle, with a little left for
/// `build_parcels`' overlap and area trims.
pub const WAIST: f64 = 0.48;

/// The least face area, m2, the interior lane pass splits (reference 12 000):
/// two deep plots back to back along a long street. **A labelled judgement**.
pub const LANE_MIN_AREA_M2: f64 = 24_000.0;

/// `base` with the burgage street rules: [`STREET_SPACING_M`] and
/// [`SEGMENT_MEDIAN_M`], every other rule unchanged.
pub fn rules(base: Rules) -> Rules {
    let mut r = base;
    r.street.parallel_street_spacing = STREET_SPACING_M;
    r.street.segment_length_median = SEGMENT_MEDIAN_M;
    r
}

/// This module's tests.
#[cfg(test)]
mod tests;
