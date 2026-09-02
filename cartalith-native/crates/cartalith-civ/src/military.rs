//! Fortification and defensive strength — `GUI_GAP_REGISTER.md` **CV-25**.
//!
//! ## This is a port, not a design
//!
//! CV-25 was registered as "new design, not a port gap": *"cartalith-civ
//! models none of them and neither does the reference."* The second half of
//! that is wrong, and this module exists because of it. The reference has
//! three real, connected pieces of military modelling:
//!
//! | Reference | Line | What it is |
//! |---|---|---|
//! | `_umWallSpec` | 22105 | the four-rung fortification ladder — `none` \| `ditch` \| `palisade` \| `stone` |
//! | `_umInferWalls` | 22134 | its boolean view, which `_civFactionAggregates` reads |
//! | `_civPlaceDefensibility` | 23802 | per-settlement defensive strength `0..1` |
//!
//! and a fourth, `_civFactionAggregates`' `power.military` axis, is
//! **already ported** ([`crate::civ_faction_aggregates`]) — its
//! `0.45*normPop + 0.35*fortifiedFraction + 0.20*capitalTierNorm` is the
//! per-faction military-strength readout CV-25 asks for.
//!
//! It was, however, ported with `FactionPlace::fortified` hard-wired
//! `false`, because `_umInferWalls` had no port: the `0.35` term was dead
//! weight on every world. Porting the ladder here is what makes that
//! coefficient reachable, so this module is the missing input to an
//! existing formula rather than a parallel invention.
//!
//! ## What is still genuinely absent
//!
//! Garrison **headcounts**, campaigns, unit movement and combat. The
//! reference has none of them either, and none is derivable from anything
//! here — a headcount would be a fabricated number wearing a real one's
//! clothes. The per-settlement figure this module reports is the
//! reference's own [`civ_place_defensibility`], and the register entry is
//! narrowed to say so rather than closed.

use cartalith_jsmath::{js_max, js_min, js_round};

use crate::{SettlementKind, terrain_ruggedness_d};
use crate::urban_adapter::um_infer_age;

/// The four rungs of `_umWallSpec`'s ladder, in ascending order. Returned
/// as `&'static str` rather than an enum to match the reference's own
/// string returns exactly, the same convention
/// [`crate::civ_culture_terrain_fit`]'s `verdict` already uses.
pub const WALL_SPECS: [&str; 4] = ["none", "ditch", "palisade", "stone"];

/// Everything `_umWallSpec` reads off a place.
///
/// The reference's `p.kind==='fortress'` branch has no analogue here:
/// `SettlementKind` carries the six tiers this port's pipeline produces
/// and `fortress` is one of the four it does not (the same four
/// [`crate::civ_tax_rate`] lists for provenance and declines to
/// approximate). The `specialisation==='garrison'` branch **does** apply —
/// `garrison` is a real key in [`crate::roster::CIV_SPECIALISATIONS`] and
/// `cartalith-godot`'s place editor can set it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallPlace<'a> {
    /// `p.umWalls`: `Some(true)` → `stone`, `Some(false)` → `none`,
    /// `None` → the ladder decides. The reference's explicit per-settlement
    /// override, and the reason `PlaceExtras::walls` is a tri-state.
    pub walls_override: Option<bool>,
    pub kind: SettlementKind,
    pub pop: f64,
    /// `p.traits.includes('fortified')`.
    pub fortified_trait: bool,
    /// `p.umAge`; `None` falls through to [`um_infer_age`], exactly as
    /// `(p.umAge!=null)?p.umAge:_umInferAge(pop)` does.
    pub age_override: Option<f64>,
    pub specialisation: Option<&'a str>,
    /// The settlement's relative elevation `(field[i]-sea)/max(1e-6,1-sea)`.
    ///
    /// Passed in rather than sampled here so this stays a pure per-place
    /// function with no grid parameters — [`civ_relative_elevation`] is the
    /// sampler, and a caller with no `field` at all passes the reference's
    /// own no-field answer, `0.0` (which makes `terrainD` 0 and the
    /// commanding-village rung unreachable, as it is there).
    pub relative_elevation: f64,
}

/// `CIV_SETTLEMENT_CLASSES[].rank` as `_umWallSpec` reads it. Identical to
/// the private `civ_tier_rank` [`crate::civ_faction_aggregates`] uses; kept
/// as its own `i32`-valued function because the wall ladder branches on
/// exact ranks (`>=3`, `==2`, `==1`) and comparing floats for equality to
/// pick a branch is precisely the thing this project's own conventions
/// warn about.
fn wall_tier_rank(kind: SettlementKind) -> i32 {
    match kind {
        SettlementKind::Hamlet => 0,
        SettlementKind::Village => 1,
        SettlementKind::Town => 2,
        SettlementKind::City => 3,
        SettlementKind::Capital => 4,
        SettlementKind::Metropolis => 5,
    }
}

/// `_umWallSpec` (reference line 22105) — the fortification ladder derived
/// from tier + function + threat + wealth + age + command of ground.
///
/// Ported rung for rung. The one branch that cannot be reached is
/// `p.kind==='fortress'`; see [`WallPlace`].
pub fn um_wall_spec(p: &WallPlace) -> &'static str {
    match p.walls_override {
        Some(true) => return "stone",
        Some(false) => return "none",
        None => {}
    }
    let rank = wall_tier_rank(p.kind);
    let fortified = p.fortified_trait;
    // `p.pop||0` — JS falsiness, so a NaN population reads as 0 rather than
    // poisoning every comparison below into `false`.
    let pop = if p.pop.is_nan() { 0.0 } else { p.pop };
    let age = p.age_override.unwrap_or_else(|| um_infer_age(pop));
    if p.specialisation == Some("garrison") {
        return if pop >= 1200.0 { "stone" } else { "palisade" };
    }
    if rank >= 3 {
        return "stone";
    }
    if rank == 2 {
        return if pop >= 1200.0 || age >= 260.0 || fortified { "stone" } else { "palisade" };
    }
    if fortified {
        return if rank >= 1 { "palisade" } else { "ditch" };
    }
    let terrain_d = terrain_ruggedness_d(p.relative_elevation);
    if rank == 1 && terrain_d > 0.9 && pop >= 250.0 {
        return "ditch";
    }
    "none"
}

/// `_umInferWalls` (reference line 22134) — the boolean view of the spec
/// that `_civFactionAggregates`' `fortifiedCount` reads.
pub fn um_infer_walls(p: &WallPlace) -> bool {
    um_wall_spec(p) != "none"
}

/// `(field[i]-sea)/Math.max(1e-6,1-sea)` at a settlement's own cell, with
/// the reference's `Math.max(0,Math.min(GW-1,Math.round(p.x)))` clamp —
/// the per-point sampling convention `_civPlaceDefensibility`,
/// `_umWallSpec` and `_civPlaceGrainYield` all share.
///
/// Returns `0.0` for an absent or wrong-length field, which is the
/// reference's own `if(typeof field==='undefined'||!field||!field.length)`
/// answer.
pub fn civ_relative_elevation(field: &[f32], gw: usize, gh: usize, sea: f64, x: f64, y: f64) -> f64 {
    if gw == 0 || gh == 0 || field.len() != gw * gh {
        return 0.0;
    }
    let xi = js_max(0.0, js_min((gw - 1) as f64, js_round(x))) as usize;
    let yi = js_max(0.0, js_min((gh - 1) as f64, js_round(y))) as usize;
    let denom = js_max(1e-6, 1.0 - sea);
    (field[yi * gw + xi] as f64 - sea) / denom
}

/// `_civPlaceDefensibility` (reference line 23802) — defensive strength
/// `0..1`: the terrain-ruggedness term `buildSettlementSuitability` scores
/// sites on (mild upland `r≈0.35` highest), blended with whether the
/// settlement is actually walled.
///
/// `walled` is [`um_infer_walls`]' answer; it is a parameter rather than an
/// internal call because the reference's own comment at `_umWallSpec` warns
/// that wall existence must not depend on defensibility, and keeping the
/// two one-directional here makes that impossible to reintroduce.
pub fn civ_place_defensibility(relative_elevation: f64, walled: bool) -> f64 {
    let terrain_d = terrain_ruggedness_d(relative_elevation);
    js_max(0.0, js_min(1.0, 0.6 * terrain_d + if walled { 0.4 } else { 0.0 }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(kind: SettlementKind, pop: f64) -> WallPlace<'static> {
        WallPlace {
            walls_override: None,
            kind,
            pop,
            fortified_trait: false,
            age_override: None,
            specialisation: None,
            relative_elevation: 0.0,
        }
    }

    #[test]
    fn override_wins_over_every_rung() {
        let mut p = place(SettlementKind::Hamlet, 10.0);
        p.walls_override = Some(true);
        assert_eq!(um_wall_spec(&p), "stone");
        let mut p = place(SettlementKind::Metropolis, 900_000.0);
        p.walls_override = Some(false);
        assert_eq!(um_wall_spec(&p), "none");
        assert!(!um_infer_walls(&p));
    }

    #[test]
    fn rank_three_and_above_is_always_stone() {
        for k in [SettlementKind::City, SettlementKind::Capital, SettlementKind::Metropolis] {
            assert_eq!(um_wall_spec(&place(k, 1.0)), "stone", "{k:?}");
        }
    }

    /// The town rung is the one with three independent ways to earn stone;
    /// each is checked alone so a mutation of any single threshold shows up.
    #[test]
    fn town_earns_stone_by_wealth_age_or_threat_and_palisades_otherwise() {
        // pop 900 -> um_infer_age(900) = round(60+240*log10(9)) = 289 >= 260,
        // so age alone already qualifies; drop to a genuinely young town.
        let young = place(SettlementKind::Town, 100.0);
        assert!(um_infer_age(100.0) < 260.0);
        assert_eq!(um_wall_spec(&young), "palisade");

        let mut wealthy = young;
        wealthy.pop = 1200.0;
        wealthy.age_override = Some(30.0);
        assert_eq!(um_wall_spec(&wealthy), "stone");

        let mut old = young;
        old.age_override = Some(260.0);
        assert_eq!(um_wall_spec(&old), "stone");

        let mut threatened = young;
        threatened.fortified_trait = true;
        assert_eq!(um_wall_spec(&threatened), "stone");

        // Just below each boundary, to pin the constants rather than the
        // direction of the comparison.
        let mut nearly = young;
        nearly.pop = 1199.0;
        nearly.age_override = Some(259.0);
        assert_eq!(um_wall_spec(&nearly), "palisade");
    }

    #[test]
    fn garrison_specialisation_outranks_tier() {
        let mut p = place(SettlementKind::Hamlet, 1200.0);
        p.specialisation = Some("garrison");
        assert_eq!(um_wall_spec(&p), "stone");
        p.pop = 1199.0;
        assert_eq!(um_wall_spec(&p), "palisade");
    }

    #[test]
    fn threatened_village_palisades_and_threatened_hamlet_ditches() {
        let mut v = place(SettlementKind::Village, 50.0);
        v.fortified_trait = true;
        assert_eq!(um_wall_spec(&v), "palisade");
        let mut h = place(SettlementKind::Hamlet, 50.0);
        h.fortified_trait = true;
        assert_eq!(um_wall_spec(&h), "ditch");
    }

    /// The commanding-village rung needs all three of rank 1, `terrainD>0.9`
    /// and `pop>=250` — `terrainD>0.9` means `|r-0.35| < 0.025`, so the
    /// window is `r ∈ (0.325, 0.375)`.
    ///
    /// **The strictness of that `>` is deliberately not asserted, because it
    /// is unobservable.** `1 - 4*|r-0.35|` never evaluates to exactly `0.9`
    /// for any `f64` `r` in the neighbourhood — the achievable results step
    /// straight from `0.9000000000000001` to `0.8999999999999999` — so
    /// `> 0.9` and `>= 0.9` are the same function here. That is an
    /// equivalent mutant, recorded rather than chased: it is the *constant*
    /// that carries the meaning, and the two cases just outside the window
    /// below are what pin it.
    #[test]
    fn commanding_village_digs_in() {
        let mut v = place(SettlementKind::Village, 250.0);
        v.relative_elevation = 0.35;
        assert_eq!(um_wall_spec(&v), "ditch");

        // Population is a separate, independent gate.
        v.pop = 249.0;
        assert_eq!(um_wall_spec(&v), "none");
        v.pop = 250.0;

        // Just inside the window on each side: still commanding ground.
        v.relative_elevation = 0.35 + 0.0249;
        assert_eq!(um_wall_spec(&v), "ditch");
        v.relative_elevation = 0.35 - 0.0249;
        assert_eq!(um_wall_spec(&v), "ditch");

        // Just outside on each side: not. A mutated 0.9 widens or narrows
        // this window and one of these four flips.
        v.relative_elevation = 0.35 + 0.0251;
        assert_eq!(um_wall_spec(&v), "none");
        v.relative_elevation = 0.35 - 0.0251;
        assert_eq!(um_wall_spec(&v), "none");
    }

    #[test]
    fn ordinary_hamlet_has_nothing() {
        assert_eq!(um_wall_spec(&place(SettlementKind::Hamlet, 40.0)), "none");
        assert!(!um_infer_walls(&place(SettlementKind::Hamlet, 40.0)));
    }

    #[test]
    fn defensibility_blends_ground_and_walls() {
        // Perfect mild upland, unwalled: 0.6*1 = 0.6.
        assert!((civ_place_defensibility(0.35, false) - 0.6).abs() < 1e-12);
        // Same ground, walled: clamped at 1.
        assert!((civ_place_defensibility(0.35, true) - 1.0).abs() < 1e-12);
        // Sea-level flat, unwalled: terrainD = max(0, 1-4*0.35) = 0 -> 0.
        assert_eq!(civ_place_defensibility(0.0, false), 0.0);
        // Sea-level flat, walled: the wall term alone.
        assert!((civ_place_defensibility(0.0, true) - 0.4).abs() < 1e-12);
    }

    #[test]
    fn relative_elevation_clamps_and_guards() {
        let field = [0.0f32, 0.5, 1.0, 0.25];
        // 2x2, sea 0.5, denom 0.5.
        assert!((civ_relative_elevation(&field, 2, 2, 0.5, 0.0, 0.0) - (-1.0)).abs() < 1e-12);
        assert!((civ_relative_elevation(&field, 2, 2, 0.5, 99.0, 99.0) - (-0.5)).abs() < 1e-12);
        assert_eq!(civ_relative_elevation(&[], 2, 2, 0.5, 0.0, 0.0), 0.0);
        assert_eq!(civ_relative_elevation(&field, 0, 0, 0.5, 0.0, 0.0), 0.0);
    }
}
