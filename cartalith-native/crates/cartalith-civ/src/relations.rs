//! Pairwise faction relations — `GUI_GAP_REGISTER.md` **CV-26**.
//!
//! ## This one really is new, and deliberately small
//!
//! Unlike [`crate::military`], CV-26 has no reference implementation to
//! port: grep the frozen snapshot for `diplomacy`, `alliance`, `vassal`,
//! `treaty` or `rivalry` and the only hits are prose. The register's
//! structural objection was the real one — *"there is no edge between two
//! factions to hold a value, so a matrix would be a grid of blanks"* — so
//! this module creates that edge, and creates nothing else.
//!
//! **What it is:** one symmetric value per unordered faction pair,
//! **derived** from quantities the civ layer already computes and
//! **recomputed** on demand, exactly like [`crate::civ_faction_aggregates`]
//! and `wildlife_regions`. There is no stored relation, no state that
//! changes over time, and no action that writes one.
//!
//! **What it deliberately is not:** diplomacy actions, treaties, vassalage,
//! war declarations, or any transition of a relation over time. Those are a
//! real feature with real design questions (who acts, when, on what clock)
//! and inventing them here would be exactly the "improvising a game system"
//! the register warned against.
//!
//! ## The four terms, and why each one
//!
//! Every term is symmetric by construction — `rel(a,b)` and `rel(b,a)` are
//! the same expression, not the same expression evaluated twice and
//! averaged.
//!
//! | Term | Weight | Source | Reasoning |
//! |---|---|---|---|
//! | culture | `+0.30` | `civFactionCulture` | shared culture is the strongest affinity the roster actually records |
//! | religion | `±0.20` | `civFactionReligion` | shared faith binds, *different* faiths divide; `none` on either side is silence, not division |
//! | trade | `+0.25` | the aggregate's `imports`/`exports` | a polity that supplies what its neighbour lacks has a standing reason not to fight it |
//! | friction | `-0.55` | shared border × relative power | contested ground between evenly-matched powers; the only negative structural term |
//!
//! The border term is measured **relative to the widest border on this
//! map**, not against an absolute cell count — the same discipline the
//! reference's own v1.30/v1.32/v1.37 trade-balance and archetype fixes
//! settled on after absolute margins proved unreachable on some worlds.
//!
//! Friction is `border × (0.35 + 0.65 × rivalry)` rather than `border`
//! alone, because a long border with a weak neighbour is not a rivalry: it
//! is a frontier. `rivalry` is high only when *both* factions are strong
//! **and** evenly matched, which is the configuration that makes a border
//! contested.

use cartalith_jsmath::{js_max, js_min};

use crate::FactionAggregates;

/// Stance labels, in descending order of the value that produces them.
/// Read directly by the shell, so the vocabulary lives here rather than in
/// GDScript — the same reason [`crate::roster`] holds the dropdown tables.
pub const RELATION_STANCES: [&str; 5] = ["allied", "friendly", "neutral", "wary", "hostile"];

/// `value` → one of [`RELATION_STANCES`]. Thresholds are symmetric about
/// zero on purpose: a stance and its mirror should need the same magnitude
/// of evidence.
fn stance_for(value: f64) -> &'static str {
    if value >= 0.45 {
        "allied"
    } else if value >= 0.15 {
        "friendly"
    } else if value > -0.15 {
        "neutral"
    } else if value > -0.45 {
        "wary"
    } else {
        "hostile"
    }
}

/// One unordered pair's edge. Every term is reported alongside the verdict
/// so the shell can *show its work* rather than assert a number — the
/// discipline `SuitExplanation` and `civ_culture_terrain_fit` already set
/// in this crate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FactionRelation {
    /// The lower faction index. Always `>= 1`: index 0 is "Unclaimed" and
    /// is not a party to anything.
    pub a: usize,
    /// The higher faction index, always `> a`.
    pub b: usize,
    /// `-1..1`. Positive is affinity, negative is friction.
    pub value: f64,
    /// One of [`RELATION_STANCES`].
    pub stance: &'static str,
    /// 4-neighbour cell pairs where one side is `a` and the other `b`.
    pub border_cells: usize,
    /// `border_cells` against the widest border on this map, `0..1`.
    pub border_fraction: f64,
    /// `1` when the two factions share a culture key.
    pub culture_term: f64,
    /// `+1` shared faith, `-1` two different faiths, `0` when either is
    /// `"none"`.
    pub religion_term: f64,
    /// `0..1` — how much of what each side imports the other exports.
    pub trade_term: f64,
    /// `0..1` — both strong *and* evenly matched.
    pub rivalry_term: f64,
}

/// Every pair, plus the map-relative scale the border term was judged on.
#[derive(Debug, Clone, PartialEq)]
pub struct FactionRelations {
    pub faction_count: usize,
    /// Ascending by `(a, b)`; `n*(n-1)/2` entries for `n` real factions.
    pub pairs: Vec<FactionRelation>,
    /// The widest shared border on this map, in cell pairs — the
    /// denominator every `border_fraction` was divided by.
    pub max_border_cells: usize,
}

impl FactionRelations {
    /// The edge between two factions in either order, or `None` when either
    /// index is out of range or they are the same faction.
    pub fn get(&self, a: usize, b: usize) -> Option<&FactionRelation> {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        self.pairs.iter().find(|r| r.a == lo && r.b == hi)
    }
}

/// What [`civ_faction_relations`] reads beyond the aggregate.
#[derive(Clone, Copy)]
pub struct FactionRelationsInput<'a> {
    /// `CIV_FACTIONS.length`, index 0 = "Unclaimed".
    pub faction_count: usize,
    pub gw: usize,
    pub gh: usize,
    /// `civTerritory`: faction index per cell, `0` = unclaimed. Absent
    /// leaves every border at zero — a legitimate state (territory has not
    /// been computed), not an error.
    pub territory: Option<&'a [i32]>,
    /// `civFactionCulture[i]`, indexed by faction. A short slice means the
    /// missing factions have no recorded culture and match nobody.
    pub cultures: &'a [&'a str],
    /// `civFactionReligion[i]`, `"none"` where unset.
    pub religions: &'a [&'a str],
    /// Whether the map wraps in x, so the seam counts as a real border.
    /// `civTerritory`'s own producers already honour this.
    pub wrap_x: bool,
}

/// Shared border between every ordered-by-index pair, as 4-neighbour cell
/// pairs. One O(cells) pass, deliberately independent of any other
/// border/influence work: it needs a count, not a field.
fn shared_borders(input: &FactionRelationsInput) -> Vec<usize> {
    let n = input.faction_count;
    let mut counts = vec![0usize; n * n];
    let (Some(terr), gw, gh) = (input.territory, input.gw, input.gh) else {
        return counts;
    };
    if gw == 0 || gh == 0 || terr.len() != gw * gh {
        return counts;
    }
    let bump = |f: i32, g: i32, counts: &mut Vec<usize>| {
        if f <= 0 || g <= 0 || f == g {
            return;
        }
        let (f, g) = (f as usize, g as usize);
        if f >= n || g >= n {
            return;
        }
        counts[f * n + g] += 1;
        counts[g * n + f] += 1;
    };
    for y in 0..gh {
        for x in 0..gw {
            let f = terr[y * gw + x];
            if x + 1 < gw {
                bump(f, terr[y * gw + x + 1], &mut counts);
            } else if input.wrap_x && gw > 1 {
                bump(f, terr[y * gw], &mut counts);
            }
            if y + 1 < gh {
                bump(f, terr[(y + 1) * gw + x], &mut counts);
            }
        }
    }
    counts
}

/// How much of what `b` imports `a` exports, and vice versa, as a fraction
/// of everything the two of them import **that anyone on this map actually
/// supplies**. Symmetric, `0..1`, and `0` when neither needs anything the
/// other has.
///
/// `supplied` is the union of every faction's exports, and the denominator
/// counts only imports drawn from it. That is not a convenience: a deficit
/// nobody can fill is a shared shortage, not a relationship — the
/// reference's own v1.33 finding, in its own words, *"a food deficit is not
/// automatically an import when there is no direct trade that could sustain
/// [it]"* (reference line 24500). Without this rule a good that every
/// faction lacks and none produces would silently dilute every pair's trade
/// term toward zero, which is the opposite of what it means.
fn trade_complement(
    supplied: &std::collections::BTreeSet<&str>,
    a_ex: &[&'static str],
    a_im: &[&'static str],
    b_ex: &[&'static str],
    b_im: &[&'static str],
) -> f64 {
    let needs = |im: &[&str]| im.iter().filter(|k| supplied.contains(*k)).count();
    let met = |ex: &[&str], im: &[&str]| im.iter().filter(|k| ex.contains(k)).count();
    let total = needs(a_im) + needs(b_im);
    if total == 0 {
        return 0.0;
    }
    (met(a_ex, b_im) + met(b_ex, a_im)) as f64 / total as f64
}

/// Pairwise faction relations. See the module doc for the derivation and
/// for what is deliberately absent.
///
/// **NaN policy.** [`crate::civ_faction_aggregates`] can legitimately
/// produce a `NaN` power axis (an empty faction's `0/0` mean), and it
/// propagates it on purpose rather than clamping it into a plausible
/// number. A `NaN` reaching a stance comparison would fall through every
/// branch to `"hostile"` — the loudest possible answer from the least
/// information — so a non-finite value is collapsed to `0.0`/`"neutral"`
/// here, at the one place where a number becomes a claim about two
/// polities.
pub fn civ_faction_relations(
    input: &FactionRelationsInput,
    aggregates: &FactionAggregates,
) -> FactionRelations {
    let n = input.faction_count;
    let borders = shared_borders(input);
    let max_border = borders.iter().copied().max().unwrap_or(0);

    // Every good anyone on this map exports -- see `trade_complement`.
    let supplied: std::collections::BTreeSet<&str> =
        aggregates.by_faction.iter().flat_map(|f| f.exports.iter().copied()).collect();

    let culture_of = |i: usize| input.cultures.get(i).copied().unwrap_or("");
    let religion_of = |i: usize| input.religions.get(i).copied().unwrap_or("none");

    let mut pairs = Vec::new();
    for a in 1..n {
        for b in (a + 1)..n {
            let border_cells = borders.get(a * n + b).copied().unwrap_or(0);
            let border_fraction =
                if max_border == 0 { 0.0 } else { border_cells as f64 / max_border as f64 };

            let (ca, cb) = (culture_of(a), culture_of(b));
            let culture_term = if !ca.is_empty() && ca == cb { 1.0 } else { 0.0 };

            let (ra, rb) = (religion_of(a), religion_of(b));
            let religion_term = if ra == "none" || rb == "none" {
                0.0
            } else if ra == rb {
                1.0
            } else {
                -1.0
            };

            let (ag_a, ag_b) = (aggregates.by_faction.get(a), aggregates.by_faction.get(b));
            let trade_term = match (ag_a, ag_b) {
                (Some(x), Some(y)) => {
                    trade_complement(&supplied, &x.exports, &x.imports, &y.exports, &y.imports)
                }
                _ => 0.0,
            };
            let (pa, pb) = (
                ag_a.map_or(0.0, |x| x.power.overall),
                ag_b.map_or(0.0, |y| y.power.overall),
            );
            // Both strong AND evenly matched. `js_min`/`js_max` so a NaN
            // axis stays a NaN here rather than being absorbed; the final
            // collapse below is the single place it is dealt with.
            let rivalry_term = js_max(
                0.0,
                js_min(1.0, (js_min(pa, pb) / 100.0) * (1.0 - (pa - pb).abs() / 100.0)),
            );

            let friction = border_fraction * (0.35 + 0.65 * rivalry_term);
            let raw = 0.30 * culture_term + 0.20 * religion_term + 0.25 * trade_term
                - 0.55 * friction;
            let mut value = js_max(-1.0, js_min(1.0, raw));
            if !value.is_finite() {
                value = 0.0;
            }

            pairs.push(FactionRelation {
                a,
                b,
                value,
                stance: stance_for(value),
                border_cells,
                border_fraction,
                culture_term,
                religion_term,
                trade_term,
                rivalry_term: if rivalry_term.is_finite() { rivalry_term } else { 0.0 },
            });
        }
    }

    FactionRelations { faction_count: n, pairs, max_border_cells: max_border }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FactionAggregate, FactionPower};

    fn agg(powers: &[f64], trade: &[(Vec<&'static str>, Vec<&'static str>)]) -> FactionAggregates {
        let by_faction = (0..powers.len())
            .map(|i| {
                let (exports, imports) = trade
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| (Vec::new(), Vec::new()));
                FactionAggregate {
                    pop: 0.0,
                    territory_km2: 0.0,
                    food_production_capacity: 0.0,
                    food_surplus: 0.0,
                    trade_volume: 0.0,
                    mean_importance: 0.0,
                    fortified_fraction: 0.0,
                    settlement_count: 0,
                    capital: None,
                    resource_potential: Default::default(),
                    power: FactionPower { overall: powers[i], ..Default::default() },
                    tax_income: 0.0,
                    imports,
                    exports,
                    strategic_resources: Vec::new(),
                    sector_output: Default::default(),
                    craft_share: 0.0,
                    terrain_mix: Default::default(),
                }
            })
            .collect();
        FactionAggregates {
            by_faction,
            max_pop: 0.0,
            max_trade_volume: 0.0,
            max_territory_km2: 0.0,
            max_settlement_count: 0,
            world_mean_resource: Default::default(),
            world_mean_terrain: Default::default(),
        }
    }

    fn input<'a>(
        terr: Option<&'a [i32]>,
        cultures: &'a [&'a str],
        religions: &'a [&'a str],
    ) -> FactionRelationsInput<'a> {
        FactionRelationsInput {
            faction_count: cultures.len(),
            gw: 4,
            gh: 4,
            territory: terr,
            cultures,
            religions,
            wrap_x: false,
        }
    }

    /// A 4x4 grid split down the middle: factions 1 and 2 meet along one
    /// full column boundary (4 cell pairs), faction 3 holds nothing.
    const SPLIT: [i32; 16] = [1, 1, 2, 2, 1, 1, 2, 2, 1, 1, 2, 2, 1, 1, 2, 2];

    #[test]
    fn pairs_are_every_unordered_pair_of_real_factions() {
        let c = ["", "a", "b", "c"];
        let r = ["none", "none", "none", "none"];
        let out = civ_faction_relations(&input(None, &c, &r), &agg(&[0.0; 4], &[]));
        assert_eq!(
            out.pairs.iter().map(|p| (p.a, p.b)).collect::<Vec<_>>(),
            vec![(1, 2), (1, 3), (2, 3)]
        );
        // Index 0 is never a party.
        assert!(out.pairs.iter().all(|p| p.a >= 1));
    }

    #[test]
    fn lookup_is_order_independent() {
        let c = ["", "a", "b"];
        let r = ["none", "none", "none"];
        let out = civ_faction_relations(&input(None, &c, &r), &agg(&[0.0; 3], &[]));
        assert_eq!(out.get(1, 2), out.get(2, 1));
        assert!(out.get(1, 1).is_none());
        assert!(out.get(1, 9).is_none());
    }

    #[test]
    fn shared_border_is_counted_once_per_cell_pair() {
        let c = ["", "a", "b"];
        let r = ["none", "none", "none"];
        let out = civ_faction_relations(&input(Some(&SPLIT), &c, &r), &agg(&[0.0; 3], &[]));
        let e = out.get(1, 2).unwrap();
        assert_eq!(e.border_cells, 4);
        assert_eq!(out.max_border_cells, 4);
        assert!((e.border_fraction - 1.0).abs() < 1e-12);
    }

    #[test]
    fn wrap_x_adds_the_seam() {
        let c = ["", "a", "b"];
        let r = ["none", "none", "none"];
        let mut i = input(Some(&SPLIT), &c, &r);
        i.wrap_x = true;
        let out = civ_faction_relations(&i, &agg(&[0.0; 3], &[]));
        // The interior column plus the wrapped seam: 4 + 4.
        assert_eq!(out.get(1, 2).unwrap().border_cells, 8);
    }

    #[test]
    fn culture_and_religion_move_the_value_in_the_documented_directions() {
        let none = ["none", "none", "none"];
        let same_c = civ_faction_relations(&input(None, &["", "riverlands", "riverlands"], &none), &agg(&[0.0; 3], &[]));
        assert!((same_c.get(1, 2).unwrap().value - 0.30).abs() < 1e-12);

        let diff_faith =
            civ_faction_relations(&input(None, &["", "a", "b"], &["none", "sun", "moon"]), &agg(&[0.0; 3], &[]));
        assert!((diff_faith.get(1, 2).unwrap().value - -0.20).abs() < 1e-12);

        let same_faith =
            civ_faction_relations(&input(None, &["", "a", "b"], &["none", "sun", "sun"]), &agg(&[0.0; 3], &[]));
        assert!((same_faith.get(1, 2).unwrap().value - 0.20).abs() < 1e-12);

        // "none" on one side is silence, not division.
        let silent =
            civ_faction_relations(&input(None, &["", "a", "b"], &["none", "sun", "none"]), &agg(&[0.0; 3], &[]));
        assert_eq!(silent.get(1, 2).unwrap().religion_term, 0.0);
    }

    #[test]
    fn trade_term_is_symmetric_and_bounded() {
        let c = ["", "a", "b"];
        let r = ["none", "none", "none"];
        // 1 exports grain, imports ore; 2 exports ore, imports grain.
        let a = agg(
            &[0.0; 3],
            &[
                (vec![], vec![]),
                (vec!["grain"], vec!["ore"]),
                (vec!["ore"], vec!["grain"]),
            ],
        );
        let out = civ_faction_relations(&input(None, &c, &r), &a);
        let e = out.get(1, 2).unwrap();
        assert!((e.trade_term - 1.0).abs() < 1e-12);
        assert_eq!(e.stance, "friendly");
    }

    /// A good every faction imports and none exports is a shared shortage,
    /// not a relationship — it must not dilute the term. This is the exact
    /// shape `food` takes when the aggregate runs without a population
    /// density: every faction's surplus is negative and nobody's is
    /// positive.
    #[test]
    fn a_good_nobody_supplies_does_not_dilute_the_trade_term() {
        let c = ["", "a", "b"];
        let r = ["none", "none", "none"];
        let matched = agg(
            &[0.0; 3],
            &[
                (vec![], vec![]),
                (vec!["grain"], vec!["ore"]),
                (vec!["ore"], vec!["grain"]),
            ],
        );
        let base = civ_faction_relations(&input(None, &c, &r), &matched);
        assert!((base.get(1, 2).unwrap().trade_term - 1.0).abs() < 1e-12);

        // Same world, plus a good both of them lack and neither produces.
        let with_shortage = agg(
            &[0.0; 3],
            &[
                (vec![], vec![]),
                (vec!["grain"], vec!["ore", "food"]),
                (vec!["ore"], vec!["grain", "food"]),
            ],
        );
        let out = civ_faction_relations(&input(None, &c, &r), &with_shortage);
        assert!(
            (out.get(1, 2).unwrap().trade_term - 1.0).abs() < 1e-12,
            "an unsupplied shortage diluted the trade term"
        );

        // But a good only ONE of them lacks, which the other does supply,
        // still counts on both halves of the fraction.
        let one_sided = agg(
            &[0.0; 3],
            &[(vec![], vec![]), (vec!["grain"], vec!["ore"]), (vec!["ore"], vec![])],
        );
        let out = civ_faction_relations(&input(None, &c, &r), &one_sided);
        assert!((out.get(1, 2).unwrap().trade_term - 1.0).abs() < 1e-12);
    }

    /// The friction term is the only negative structural one, and it needs
    /// a border *and* a rivalry to bite. Two evenly-matched strong powers
    /// across a full border should read hostile; the same border between
    /// two nobodies should not.
    #[test]
    fn friction_needs_both_a_border_and_a_rivalry() {
        let c = ["", "a", "b"];
        let r = ["none", "none", "none"];

        let strong = civ_faction_relations(&input(Some(&SPLIT), &c, &r), &agg(&[0.0, 100.0, 100.0], &[]));
        let e = strong.get(1, 2).unwrap();
        assert!((e.rivalry_term - 1.0).abs() < 1e-12);
        assert!((e.value - -0.55).abs() < 1e-12);
        assert_eq!(e.stance, "hostile");

        let weak = civ_faction_relations(&input(Some(&SPLIT), &c, &r), &agg(&[0.0, 0.0, 0.0], &[]));
        let w = weak.get(1, 2).unwrap();
        assert_eq!(w.rivalry_term, 0.0);
        assert!((w.value - (-0.55 * 0.35)).abs() < 1e-12);
        assert_eq!(w.stance, "wary");

        // Lopsided: a strong power beside a weak one is a frontier, not a
        // rivalry.
        let lopsided = civ_faction_relations(&input(Some(&SPLIT), &c, &r), &agg(&[0.0, 100.0, 5.0], &[]));
        assert!(lopsided.get(1, 2).unwrap().rivalry_term < 0.01);
    }

    #[test]
    fn no_territory_means_no_friction_rather_than_an_error() {
        let c = ["", "a", "b"];
        let r = ["none", "none", "none"];
        let out = civ_faction_relations(&input(None, &c, &r), &agg(&[0.0, 100.0, 100.0], &[]));
        let e = out.get(1, 2).unwrap();
        assert_eq!(e.border_cells, 0);
        assert_eq!(e.border_fraction, 0.0);
        assert_eq!(e.stance, "neutral");
    }

    #[test]
    fn a_nan_power_axis_reads_neutral_not_hostile() {
        let c = ["", "a", "b"];
        let r = ["none", "none", "none"];
        let out = civ_faction_relations(&input(Some(&SPLIT), &c, &r), &agg(&[0.0, f64::NAN, 100.0], &[]));
        let e = out.get(1, 2).unwrap();
        assert_eq!(e.value, 0.0);
        assert_eq!(e.stance, "neutral");
        assert_eq!(e.rivalry_term, 0.0);
    }

    #[test]
    fn stance_thresholds_are_symmetric_about_zero() {
        assert_eq!(stance_for(0.45), "allied");
        assert_eq!(stance_for(0.449), "friendly");
        assert_eq!(stance_for(0.15), "friendly");
        assert_eq!(stance_for(0.149), "neutral");
        assert_eq!(stance_for(-0.149), "neutral");
        assert_eq!(stance_for(-0.15), "wary");
        assert_eq!(stance_for(-0.45), "hostile");
        assert_eq!(stance_for(-0.449), "wary");
    }
}
