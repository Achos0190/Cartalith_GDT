//! Ruling AW (`LARGE_ITEM_RULINGS.md`, 2026-09-24) -- per-settlement
//! garrisons, **derived**: each faction's standing army
//! ([`crate::manpower::Manpower::standing_army`]) split across its settlements
//! by the rule `MILITARY_MANPOWER_SCOPE.md` §5.6 states. STATUS MM-6.
//!
//! It splits a number the manpower model already produced; it raises no
//! soldier and moves none. No reference ancestor (`DECISIONS.md` §7d).
//!
//! ## The rule, in one line
//!
//! `weight = pop × (1 + 0.35/0.45·walled + 0.20/0.45·capital·tier/5) ×
//! (1 + scale × exposure)`, then a largest-remainder split of the rounded
//! standing army. The three weights are `_civFactionAggregates`' own
//! military-axis weights with population as the unit ([`POP_WEIGHT`]);
//! exposure is the share of the faction's foreign frontier nearest to the
//! place, relative to the faction's most exposed place ([`border_exposure`]).
//! The scope carries the reason for every term.
//!
//! **`scale`, Ruling AZ (`LARGE_ITEM_RULINGS.md`, 2026-09-28):** a user
//! setting ([`GarrisonInput::exposure_scale`]), not the fixed value this rule
//! shipped with. [`EXPOSURE_SCALE`] is only its default, at `1.0` -- the
//! value every caller used before the setting existed. `0` removes the
//! border term entirely (every place keeps its base weight, whatever its
//! exposure); this function does not itself constrain `scale` to be
//! non-negative or finite -- the caller does (`cartalith-godot`'s setter
//! clamps it before it ever reaches here).
//!
//! **`is_capital`, same ruling:** true only for the settlement actually
//! flagged as its faction's seat (`SettlementPlacement::capital`), never a
//! fallback pick such as `_civFactionAggregates`' own "highest-pop of any
//! kind when there is no marked capital" (`FactionAggregates::capital`,
//! shared with the capital-tier term of the aggregate's own military score
//! and with the manpower model's `capital_road_reach` -- this rule no longer
//! reads that pick at all). A faction whose real seat was lost to conquest,
//! or was never placed, gets no capital weight anywhere: no settlement of
//! its is `is_capital`, matching the dock's own "Capital: no" for each of
//! them.
//!
//! ## No-value discipline (`MISTAKES.md`)
//!
//! A settlement with no reading is `None`, never `0`: a settlement of no
//! faction, a faction whose standing army is not finite or cannot be placed
//! (positive with zero total weight), and every settlement when the territory
//! raster is not the grid's size. A real zero -- a village whose quota rounded
//! down and won no remainder -- is `Some` with `garrison == 0`.

use crate::SettlementKind;

/// `_civFactionAggregates`' military axis is
/// `0.45·normPop + 0.35·fortifiedFraction + 0.20·capitalTierNorm`
/// (`cartalith-civ/src/lib.rs`, golden-verified). This is its population
/// weight, which this rule makes the unit the other two are read against.
pub const POP_WEIGHT: f64 = 0.45;
/// The same formula's fortification weight. See [`POP_WEIGHT`].
pub const WALL_WEIGHT: f64 = 0.35;
/// The same formula's capital-tier weight. See [`POP_WEIGHT`].
pub const CAPITAL_WEIGHT: f64 = 0.20;
/// `capitalTierNorm`'s denominator: the tier table's top rank (`metropolis`),
/// the value `civ_faction_aggregates` divides by.
pub const MAX_TIER_RANK: f64 = 5.0;
/// The *default* of [`GarrisonInput::exposure_scale`] -- what every caller
/// used before Ruling AZ made it a user setting: `weight × (1 + EXPOSURE_SCALE
/// × exposure)`, so the faction's most exposed settlement gained double its
/// interior weight.
///
/// **The rule's least-grounded number**, and said as such
/// (`MILITARY_MANPOWER_SCOPE.md` §5.6): nothing the scope cites gives a
/// frontier-to-interior garrison ratio. `1` -- the most exposed place counts
/// double -- was the smallest whole-number scale at which the term was a
/// clear multiple rather than a rounding error; it is kept as the default a
/// reader gets before touching the setting, not as a claim that it is right.
pub const EXPOSURE_SCALE: f64 = 1.0;

/// One settlement as the rule reads it. Positions are grid cells, continuous
/// (the same frame as `Placement::x`/`y`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GarrisonPlace {
    /// Owning faction; `<= 0` is unclaimed and gets no garrison.
    pub faction: i32,
    pub pop: f64,
    /// `cartalith_civ::military::um_infer_walls`' verdict for this place.
    pub walled: bool,
    /// Whether this settlement is actually its faction's marked capital seat
    /// (`SettlementPlacement::capital`) -- **never** a fallback pick. Ruling
    /// AZ: a faction with no such settlement gets no capital weight
    /// anywhere, so this must be `false` for every one of its places rather
    /// than true for a stand-in.
    pub is_capital: bool,
    pub kind: SettlementKind,
    pub x: f64,
    pub y: f64,
}

/// What [`civ_garrisons`] reads besides the places.
#[derive(Debug, Clone, Copy)]
pub struct GarrisonInput<'a> {
    pub places: &'a [GarrisonPlace],
    /// Owner per cell, row-major `gw × gh`. Any other length and exposure
    /// cannot be read, so every garrison is `None`.
    pub territory: &'a [i32],
    pub gw: usize,
    pub gh: usize,
    /// Whether the map wraps in x, so the seam is a real border (as in
    /// [`crate::relations`]).
    pub wrap_x: bool,
    /// Standing army per faction id (index = faction id). `None`, a
    /// non-finite value or an index past the end means "no standing army to
    /// split", and that faction's places are `None`.
    pub standing: &'a [Option<f64>],
    /// The border-exposure term's scale, Ruling AZ (2026-09-28,
    /// `LARGE_ITEM_RULINGS.md`): `weight × (1 + exposure_scale × exposure)`.
    /// [`EXPOSURE_SCALE`] is only this value's default. `0` removes the term.
    pub exposure_scale: f64,
}

/// One settlement's garrison and the terms behind it, so the shell can show
/// its working.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Garrison {
    /// Headcount. The faction's parts sum to its rounded standing army exactly.
    pub garrison: u64,
    /// This place's weight over its faction's total weight, `0..1`.
    pub share: f64,
    /// The unnormalised weight.
    pub weight: f64,
    /// `0..1`, see [`border_exposure`].
    pub exposure: f64,
    /// The walls/capital multiplier, `1 .. 1/POP_WEIGHT`.
    pub standing_multiplier: f64,
}

/// `SettlementKind`'s rank as `standing_multiplier`'s capital-tier term reads
/// it: `0` for Hamlet up to `5` (== [`MAX_TIER_RANK`]) for Metropolis.
fn tier_rank(kind: SettlementKind) -> f64 {
    match kind {
        SettlementKind::Hamlet => 0.0,
        SettlementKind::Village => 1.0,
        SettlementKind::Town => 2.0,
        SettlementKind::City => 3.0,
        SettlementKind::Capital => 4.0,
        SettlementKind::Metropolis => 5.0,
    }
}

/// `1 + (WALL/POP)·walled + (CAPITAL/POP)·capital·tier/MAX`.
pub fn standing_multiplier(p: &GarrisonPlace) -> f64 {
    let wall = if p.walled { WALL_WEIGHT / POP_WEIGHT } else { 0.0 };
    let cap = if p.is_capital {
        CAPITAL_WEIGHT / POP_WEIGHT * tier_rank(p.kind) / MAX_TIER_RANK
    } else {
        0.0
    };
    1.0 + wall + cap
}

/// Each place's border exposure, `0..1`, or `None` for every place when the
/// raster is not `gw × gh`.
///
/// A faction's **frontier** is every cell it holds that is 4-adjacent to a cell
/// another faction (`> 0`, different) holds; unclaimed land is not a frontier.
/// Each frontier cell counts toward the faction's settlement nearest its centre
/// (squared Euclidean distance, x measured round the seam on a wrapping map;
/// ties to the lower index). A place's exposure is its count over the largest
/// count among its faction's places -- `0` for every place of a faction with no
/// foreign frontier, and for an unclaimed place.
pub fn border_exposure(
    places: &[GarrisonPlace],
    territory: &[i32],
    gw: usize,
    gh: usize,
    wrap_x: bool,
) -> Option<Vec<f64>> {
    if gw == 0 || gh == 0 || territory.len() != gw * gh {
        return None;
    }
    let mut counts = vec![0u64; places.len()];
    // Places per faction, so a frontier cell searches only its own faction's.
    let mut by_faction: std::collections::BTreeMap<i32, Vec<usize>> = Default::default();
    for (i, p) in places.iter().enumerate() {
        if p.faction > 0 {
            by_faction.entry(p.faction).or_default().push(i);
        }
    }
    let foreign = |f: i32, j: usize| {
        let g = territory[j];
        g > 0 && g != f
    };
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let f = territory[i];
            if f <= 0 {
                continue;
            }
            let mut front = false;
            if x > 0 {
                front |= foreign(f, i - 1);
            } else if wrap_x && gw > 1 {
                front |= foreign(f, i + gw - 1);
            }
            if x + 1 < gw {
                front |= foreign(f, i + 1);
            } else if wrap_x && gw > 1 {
                front |= foreign(f, i + 1 - gw);
            }
            if y > 0 {
                front |= foreign(f, i - gw);
            }
            if y + 1 < gh {
                front |= foreign(f, i + gw);
            }
            if !front {
                continue;
            }
            let Some(mine) = by_faction.get(&f) else { continue };
            let (cx, cy) = (x as f64 + 0.5, y as f64 + 0.5);
            let mut best: Option<(f64, usize)> = None;
            for &k in mine {
                let p = &places[k];
                let mut dx = (p.x - cx).abs();
                if wrap_x {
                    dx = dx.min(gw as f64 - dx);
                }
                let dy = p.y - cy;
                let d = dx * dx + dy * dy;
                // `<` keeps the lower index on a tie; `mine` is ascending.
                if best.is_none_or(|(bd, _)| d < bd) {
                    best = Some((d, k));
                }
            }
            if let Some((_, k)) = best {
                counts[k] += 1;
            }
        }
    }
    let mut out = vec![0.0; places.len()];
    for idx in by_faction.values() {
        let max = idx.iter().map(|&k| counts[k]).max().unwrap_or(0);
        if max > 0 {
            for &k in idx {
                out[k] = counts[k] as f64 / max as f64;
            }
        }
    }
    Some(out)
}

/// Largest-remainder (Hamilton) apportionment of `total` over `weights`.
/// `None` when `total > 0` and the weights sum to zero or are not finite --
/// there is nothing to apportion by, and spreading evenly would be a second
/// rule. Remainder ties go to the lower index.
pub fn largest_remainder(total: u64, weights: &[f64]) -> Option<Vec<u64>> {
    let sum: f64 = weights.iter().sum();
    if total == 0 {
        return Some(vec![0; weights.len()]);
    }
    if !(sum.is_finite() && sum > 0.0) {
        return None;
    }
    let quotas: Vec<f64> = weights.iter().map(|w| total as f64 * w / sum).collect();
    let mut parts: Vec<u64> = quotas.iter().map(|q| q.floor() as u64).collect();
    let given: u64 = parts.iter().sum();
    // Float division can put Σ floors a hair over `total` only if a quota
    // rounded up past an integer; guard rather than underflow.
    let mut left = total.saturating_sub(given);
    let mut order: Vec<usize> = (0..weights.len()).collect();
    order.sort_by(|&a, &b| {
        let ra = quotas[a] - quotas[a].floor();
        let rb = quotas[b] - quotas[b].floor();
        rb.total_cmp(&ra).then(a.cmp(&b))
    });
    for &k in order.iter().cycle() {
        if left == 0 {
            break;
        }
        parts[k] += 1;
        left -= 1;
    }
    Some(parts)
}

/// `MILITARY_MANPOWER_SCOPE.md` §5.6: every place's garrison, in `places`
/// order. See the module doc for when a place is `None`.
pub fn civ_garrisons(input: &GarrisonInput) -> Vec<Option<Garrison>> {
    let n = input.places.len();
    let Some(exposure) =
        border_exposure(input.places, input.territory, input.gw, input.gh, input.wrap_x)
    else {
        return vec![None; n];
    };
    let mut out = vec![None; n];
    let mut factions: Vec<i32> = input.places.iter().map(|p| p.faction).filter(|&f| f > 0).collect();
    factions.sort_unstable();
    factions.dedup();
    for f in factions {
        let Some(standing) = input.standing.get(f as usize).copied().flatten() else { continue };
        if !standing.is_finite() {
            continue;
        }
        let total = standing.max(0.0).round() as u64;
        let idx: Vec<usize> = (0..n).filter(|&k| input.places[k].faction == f).collect();
        let mults: Vec<f64> = idx.iter().map(|&k| standing_multiplier(&input.places[k])).collect();
        let weights: Vec<f64> = idx
            .iter()
            .zip(&mults)
            .map(|(&k, m)| {
                let p = &input.places[k];
                let pop = if p.pop.is_finite() { p.pop.max(0.0) } else { 0.0 };
                pop * m * (1.0 + input.exposure_scale * exposure[k])
            })
            .collect();
        let Some(parts) = largest_remainder(total, &weights) else { continue };
        let sum: f64 = weights.iter().sum();
        for (j, &k) in idx.iter().enumerate() {
            out[k] = Some(Garrison {
                garrison: parts[j],
                share: if sum > 0.0 { weights[j] / sum } else { 0.0 },
                weight: weights[j],
                exposure: exposure[k],
                standing_multiplier: mults[j],
            });
        }
    }
    out
}

/// Unit tests for the garrison split: the apportionment rule, its
/// standing/capital/border terms, and the no-value discipline the module
/// doc requires.
#[cfg(test)]
mod tests {
    use super::*;

    /// A [`GarrisonPlace`] built from its fields in argument order, for
    /// fixtures that need many of them at once without the struct syntax.
    fn place(faction: i32, pop: f64, walled: bool, cap: bool, kind: SettlementKind, x: f64, y: f64) -> GarrisonPlace {
        GarrisonPlace { faction, pop, walled, is_capital: cap, kind, x, y }
    }

    /// 8 x 4, faction 1 on columns 0..4, faction 2 on 4..8: the border runs
    /// between columns 3 and 4.
    fn two_halves() -> Vec<i32> {
        (0..32).map(|i| if i % 8 < 4 { 1 } else { 2 }).collect()
    }

    /// [`civ_garrisons`] on an 8x4 grid at the default [`EXPOSURE_SCALE`],
    /// reduced to just the headcount per place for the fixtures that don't
    /// need the rest of [`Garrison`].
    fn garrisons(places: &[GarrisonPlace], t: &[i32], standing: &[Option<f64>]) -> Vec<Option<u64>> {
        garrisons_scaled(places, t, standing, EXPOSURE_SCALE)
    }

    /// [`garrisons`] with an explicit border-exposure scale, for the tests
    /// that check the setting itself rather than the rest of the rule.
    fn garrisons_scaled(places: &[GarrisonPlace], t: &[i32], standing: &[Option<f64>], scale: f64) -> Vec<Option<u64>> {
        civ_garrisons(&GarrisonInput {
            places, territory: t, gw: 8, gh: 4, wrap_x: false, standing, exposure_scale: scale,
        })
        .iter()
        .map(|g| g.map(|g| g.garrison))
        .collect()
    }

    // Protects: three literal fixtures' exact per-place headcounts (a
    // round standing army, a fractional one that rounds first, and two
    // factions split independently), and that every fixture's parts sum
    // to the rounded standing army exactly.
    #[test]
    fn the_sum_is_exact_on_literal_fixtures() {
        let t = two_halves();
        // Fixture 1: three faction-1 places, one of them a walled capital on
        // the border (x = 3.5), standing army 1000.
        let p1 = [
            place(1, 5000.0, true, true, SettlementKind::Capital, 3.5, 1.5),
            place(1, 2000.0, false, false, SettlementKind::Town, 0.5, 0.5),
            place(1, 500.0, false, false, SettlementKind::Village, 0.5, 3.5),
        ];
        let g = garrisons(&p1, &t, &[None, Some(1000.0)]);
        assert_eq!(g, vec![Some(895), Some(84), Some(21)]);
        assert_eq!(g.iter().flatten().sum::<u64>(), 1000);

        // Fixture 2: a fractional standing army rounds first (733.6 -> 734),
        // and the parts sum to the rounded figure.
        let g = garrisons(&p1, &t, &[None, Some(733.6)]);
        assert_eq!(g, vec![Some(657), Some(62), Some(15)]);
        assert_eq!(g.iter().flatten().sum::<u64>(), 734);

        // Fixture 3: both factions at once, faction 2 with two equal
        // interior-and-border towns.
        let p3 = [
            place(1, 1000.0, false, false, SettlementKind::Town, 3.5, 1.5),
            place(2, 1000.0, false, false, SettlementKind::Town, 4.5, 1.5),
            place(2, 1000.0, false, false, SettlementKind::Town, 7.5, 1.5),
            place(1, 1000.0, false, false, SettlementKind::Town, 0.5, 1.5),
        ];
        let g = garrisons(&p3, &t, &[None, Some(10.0), Some(7.0)]);
        assert_eq!(g, vec![Some(7), Some(5), Some(2), Some(3)]);
    }

    /// Ruling AZ (2026-09-28): the border-exposure scale is a user setting,
    /// not the fixed `1.0` it used to be. The parts still sum to the
    /// standing army exactly at every scale, and `0` removes the border term
    /// -- fixture 1 above ([`the_sum_is_exact_on_literal_fixtures`]) is this
    /// same faction at `scale = 1.0` (`895/84/21`), kept as the middle point
    /// a reader can cross-check against.
    // Protects: `exposure_scale` reaches `civ_garrisons` and changes the
    // split (0.0, 0.6, 1.0 default, 2.5 all give different results), while
    // the parts still sum to the standing army exactly at every scale.
    #[test]
    fn the_split_still_sums_exactly_at_several_exposure_scales() {
        let t = two_halves();
        let p1 = [
            place(1, 5000.0, true, true, SettlementKind::Capital, 3.5, 1.5),
            place(1, 2000.0, false, false, SettlementKind::Town, 0.5, 0.5),
            place(1, 500.0, false, false, SettlementKind::Village, 0.5, 3.5),
        ];
        // `0`: the border term drops out entirely -- only walls/capital and
        // population separate the three places.
        let g = garrisons_scaled(&p1, &t, &[None, Some(1000.0)], 0.0);
        assert_eq!(g, vec![Some(810), Some(152), Some(38)]);
        assert_eq!(g.iter().flatten().sum::<u64>(), 1000);

        // A fractional setting, not one of the round numbers a wiring bug
        // could produce by accident.
        let g = garrisons_scaled(&p1, &t, &[None, Some(1000.0)], 0.6);
        assert_eq!(g, vec![Some(872), Some(102), Some(26)]);
        assert_eq!(g.iter().flatten().sum::<u64>(), 1000);

        // Well past the default: the border capital's share keeps climbing.
        let g = garrisons_scaled(&p1, &t, &[None, Some(1000.0)], 2.5);
        assert_eq!(g, vec![Some(937), Some(50), Some(13)]);
        assert_eq!(g.iter().flatten().sum::<u64>(), 1000);
    }

    /// Ruling AZ: `is_capital` is only ever true for a settlement actually
    /// flagged as its faction's seat, never a fallback pick. A faction with
    /// no such settlement -- its seat lost to conquest, or simply never
    /// marked -- gets no capital weight anywhere, not even on the place a
    /// "best guess" (highest population) would have picked.
    // Protects: `is_capital == false` on every place of a faction leaves
    // `standing_multiplier` at exactly 1.0 even for its largest settlement,
    // so the split falls back to population alone.
    #[test]
    fn no_marked_capital_gives_no_settlement_the_capital_weight() {
        let biggest = place(1, 9000.0, false, false, SettlementKind::Metropolis, 0.5, 1.5);
        let smallest = place(1, 100.0, false, false, SettlementKind::Hamlet, 0.5, 2.5);
        assert_eq!(standing_multiplier(&biggest), 1.0, "no capital term despite being the largest place");
        assert_eq!(standing_multiplier(&smallest), 1.0);
        let t = two_halves();
        let g = garrisons(&[biggest, smallest], &t, &[None, Some(910.0)]);
        // Population alone: 9000/9100 and 100/9100 of 910.
        assert_eq!(g, vec![Some(900), Some(10)]);
    }

    // Protects: `largest_remainder`'s Hamilton apportionment -- equal
    // weights split the remainder to the lowest index on a tie, and an
    // unequal pair gives the extra unit to the larger remainder rather
    // than the larger weight.
    #[test]
    fn a_remainder_tie_goes_to_the_lower_index() {
        // Three equal weights and 10 men: quotas 3.333 each, one leftover.
        assert_eq!(largest_remainder(10, &[1.0, 1.0, 1.0]), Some(vec![4, 3, 3]));
        // Two leftovers over four equal weights of 2.5 each.
        assert_eq!(largest_remainder(10, &[1.0, 1.0, 1.0, 1.0]), Some(vec![3, 3, 2, 2]));
        // Largest remainder wins over index.
        assert_eq!(largest_remainder(10, &[1.0, 2.0]), Some(vec![3, 7]));
    }

    // Protects: at equal population, a walled border capital (higher
    // standing_multiplier and exposure 1.0) outranks an unwalled interior
    // village (exposure 0.0) in both garrison size and the exposure fields
    // reported on each.
    #[test]
    fn a_walled_border_capital_outranks_an_interior_village() {
        let t = two_halves();
        // Same population, so only walls, capital and exposure separate them.
        let p = [
            place(1, 800.0, false, false, SettlementKind::Village, 0.5, 1.5),
            place(1, 800.0, true, true, SettlementKind::Capital, 3.5, 1.5),
        ];
        let g = civ_garrisons(&GarrisonInput {
            places: &p, territory: &t, gw: 8, gh: 4, wrap_x: false,
            standing: &[None, Some(500.0)], exposure_scale: EXPOSURE_SCALE,
        });
        let (v, c) = (g[0].unwrap(), g[1].unwrap());
        assert_eq!((v.garrison, c.garrison), (95, 405));
        assert_eq!((v.exposure, c.exposure), (0.0, 1.0));
        assert!(c.garrison > v.garrison);
    }

    // Protects: a faction with exactly one settlement gets the whole
    // (rounded) standing army in that one place.
    #[test]
    fn a_faction_with_one_settlement_puts_everything_there() {
        let t = two_halves();
        let p = [place(2, 12.0, false, false, SettlementKind::Hamlet, 6.0, 2.0)];
        let g = garrisons(&p, &t, &[None, None, Some(4321.4)]);
        assert_eq!(g, vec![Some(4321)]);
    }

    // Protects: the no-value discipline the module doc states -- an
    // unclaimed place, a faction with no standing figure, and a faction
    // with men but no weight to place them by are all `None`; a
    // non-finite standing army is `None`; a wrong-sized territory raster
    // makes every place `None`; but a standing army of exactly `0.0` is a
    // real `Some(0)`, not absent.
    #[test]
    fn no_reading_is_absent_never_zero() {
        let t = two_halves();
        let p = [
            place(0, 900.0, false, false, SettlementKind::Town, 1.0, 1.0),
            place(1, 900.0, false, false, SettlementKind::Town, 1.0, 1.0),
            place(2, 0.0, false, false, SettlementKind::Town, 6.0, 1.0),
        ];
        // Unclaimed; faction 1 with no standing figure; faction 2 with men and
        // no weight to place them by.
        assert_eq!(garrisons(&p, &t, &[None, None, Some(50.0)]), vec![None, None, None]);
        // Non-finite standing army.
        assert_eq!(garrisons(&p, &t, &[None, Some(f64::NAN), None])[1], None);
        // A raster of the wrong size reads nothing at all.
        let g = civ_garrisons(&GarrisonInput {
            places: &p, territory: &t[..31], gw: 8, gh: 4, wrap_x: false,
            standing: &[None, Some(10.0), Some(10.0)], exposure_scale: EXPOSURE_SCALE,
        });
        assert!(g.iter().all(Option::is_none));
        // A real zero is a value: zero men split is zero each.
        assert_eq!(garrisons(&p, &t, &[None, Some(0.0), None])[1], Some(0));
    }

    // Protects: `border_exposure` assigns each frontier cell to its
    // faction's nearest place by squared distance, unclaimed neighbours
    // are never a frontier, and on a wrapping map the seam is a real
    // border whose cells are measured the short way round.
    #[test]
    fn exposure_is_the_nearest_share_of_the_foreign_frontier() {
        let t = two_halves();
        let p = [
            place(1, 1.0, false, false, SettlementKind::Town, 3.5, 0.5),
            place(1, 1.0, false, false, SettlementKind::Town, 3.5, 3.5),
            place(1, 1.0, false, false, SettlementKind::Town, 0.5, 2.0),
        ];
        // Faction 1's frontier is column 3, rows 0..4: rows 0-1 are nearer the
        // first town, rows 2-3 the second; the western town is nearest none.
        let e = border_exposure(&p, &t, 8, 4, false).unwrap();
        assert_eq!(e, vec![1.0, 1.0, 0.0]);
        // Unclaimed neighbours are not a frontier.
        let lone: Vec<i32> = (0..32).map(|i| if i % 8 < 4 { 1 } else { 0 }).collect();
        assert_eq!(border_exposure(&p, &lone, 8, 4, false).unwrap(), vec![0.0, 0.0, 0.0]);
        // On a wrapping map the seam is a border: column 0 now fronts column 7,
        // and all four of its cells are nearest the western town (4 against 2).
        let e = border_exposure(&p, &t, 8, 4, true).unwrap();
        assert_eq!(e, vec![0.5, 0.5, 1.0]);
    }
}
