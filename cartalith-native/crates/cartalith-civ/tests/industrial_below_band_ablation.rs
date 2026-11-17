//! Scratch measurement for `OUTSTANDING_WORK.md`'s row "Industrial factions
//! below the 5-10 % standing band" (Ruling FU item 2: measure first, build
//! nothing; `ECOLOGICAL_ARMY_TERM_SCOPE.md` section 10.12, 2026-11-17).
//!
//! **Pins nothing and is `#[ignore]`d.** It reads the shipped model (with
//! Ruling FU's C3 in it) over the 284 240-reading driver grid of
//! `manpower::tests::the_ordering_holds_across_the_driver_space_but_one_class`
//! on the `industrial` row, and re-prices the standing army **analytically**
//! under candidate changes to chain 1's ecological factor, `g(E) = min(E, 1)`
//! (`manpower_with_levy_bonus`: `budget_ecology = if e > 1 { e^0 } else { e }`).
//! That is exact because chain 1 is the only chain that reads `g`; the field
//! army and the levy do not read it below 1 (the forage term is `E > 1` only),
//! so the ordering `standing < field < levy` can be re-checked on the same
//! readings. Control: `g(E) * non_agri * fee / upkeep` must reproduce the
//! model's `standing_army` on every reading, or nothing below is read.
//!
//! Every candidate constant is a labelled judgement, fitted to nothing but
//! the 5 % band edge or a round number. The shipped constants are not touched.
//!
//! Run (alone): `cargo test -p cartalith-civ --test industrial_below_band_ablation -- --ignored --nocapture`

use cartalith_civ::manpower::{civ_military_manpower, ManpowerInput};
use cartalith_civ::roster::{AG_TECH_LEVELS, CIV_GOVERNMENTS};

/// One reading reduced to what the re-pricing reads.
struct R {
    standing: f64,
    field: f64,
    levy: f64,
    cit_pop: f64,
    e: f64,
    s: f64,
    fee: f64,
    band: (f64, f64),
    era13: bool,
}

#[test]
#[ignore = "scratch measurement; pins nothing; run alone"]
fn industrial_below_band_candidates() {
    let row = AG_TECH_LEVELS.iter().find(|r| r.key == "industrial").unwrap();
    let f = row.farmers_per_urbanite;
    let mut rs: Vec<R> = Vec::new();
    for &(gov, _) in CIV_GOVERNMENTS.iter() {
        for ri in 0..=4 {
            for road_i in 0..=10 {
                for nav in [0.0, 1.0] {
                    for sea in [0.0, 1.0] {
                        for ei in 0..=75 {
                            let e = 0.25 + 0.05 * f64::from(ei);
                            let nuc = 100_000.0;
                            let m = civ_military_manpower(&ManpowerInput {
                                nucleated_pop: nuc,
                                farmers_per_urbanite: f,
                                land_capacity: e * nuc * (1.0 + f),
                                government: gov,
                                capital_road_reach: f64::from(ri) * 0.25,
                                road_density: f64::from(road_i) * 0.1,
                                navigable_share: nav,
                                sea_share: sea,
                            });
                            let d = &m.drivers;
                            let g = d.ecological_factor.min(1.0);
                            // The control: chain 1 reconstructed from the drivers.
                            let own = m.total_population * (1.0 - d.agricultural_labour_ratio) * g
                                * d.fiscal_extraction_efficiency
                                / d.soldier_upkeep;
                            assert!(
                                (own - m.standing_army).abs() <= 1e-9 * m.standing_army.max(1.0),
                                "control: {own} vs {}",
                                m.standing_army
                            );
                            rs.push(R {
                                standing: m.standing_army,
                                field: m.field_army,
                                levy: m.emergency_mobilization,
                                cit_pop: m.citizen_population,
                                e: d.ecological_factor,
                                s: d.state_capacity,
                                fee: d.fiscal_extraction_efficiency,
                                band: m.era_band.standing,
                                era13: m.era_band.name == "Total industrial mobilization",
                            });
                        }
                    }
                }
            }
        }
    }
    assert_eq!(rs.len(), 284_240);
    let g = |e: f64| e.min(1.0);
    // (name, g'(E)); every one is 1 at E >= 1 so it moves only E < 1 readings.
    let cands: Vec<(&str, Box<dyn Fn(f64) -> f64>)> = vec![
        ("baseline g = E", Box::new(|e| e.min(1.0))),
        ("A  floor 0.5: g = max(E, 0.5)", Box::new(|e| e.min(1.0).max(0.5))),
        ("A' floor 0.67: g = max(E, 0.67)", Box::new(|e| e.min(1.0).max(0.67))),
        ("A'' floor 0.8: g = max(E, 0.8)", Box::new(|e| e.min(1.0).max(0.8))),
        ("B  sqrt arm: g = E^0.5", Box::new(|e| e.min(1.0).sqrt())),
        ("B' E^0.75", Box::new(|e| e.min(1.0).powf(0.75))),
        ("flat: g = 1 (ecology out of the budget)", Box::new(|_| 1.0)),
    ];
    // Where the below-band readings sit, for the attribution.
    let below_base: Vec<&R> = rs
        .iter()
        .filter(|r| r.era13 && r.standing / r.cit_pop < r.band.0)
        .collect();
    let mut es: Vec<f64> = below_base.iter().map(|r| r.e).collect();
    es.sort_by(f64::total_cmp);
    let share_of_below_with_e_lt_1 = below_base.iter().filter(|r| r.e < 1.0).count();
    println!(
        "DS below-band (era 13) readings {}; with E < 1: {}; median E {:.3}",
        below_base.len(),
        share_of_below_with_e_lt_1,
        es[es.len() / 2]
    );
    // How far below: the factor on standing that would bring each to the band's
    // lower edge, against g(E): is E the whole shortfall?
    let mut e_only_fixes = 0usize; // would be within (or above) with g = 1
    let mut level_short = 0usize; // still below with g = 1 (fee / upkeep level)
    let mut low_s_short = 0usize;
    for r in &below_base {
        let at_one = r.standing / g(r.e) / r.cit_pop;
        if at_one >= r.band.0 {
            e_only_fixes += 1;
        } else {
            level_short += 1;
            if r.s < 0.35 {
                low_s_short += 1;
            }
        }
    }
    println!(
        "DS attribution: of the below-band readings, {e_only_fixes} clear with the ecological factor alone (g = 1), {level_short} stay below at g = 1 (of which state capacity < 0.35: {low_s_short})"
    );
    for (name, gp) in &cands {
        let (mut w, mut a, mut b) = (0usize, 0usize, 0usize);
        let (mut unordered, mut moved) = (0usize, 0usize);
        let mut shares: Vec<f64> = Vec::new();
        for r in &rs {
            let st = r.standing * gp(r.e) / g(r.e);
            if (st - r.standing).abs() > 1e-9 * r.standing.max(1.0) {
                moved += 1;
            }
            if !(st < r.field && r.field < r.levy) {
                unordered += 1;
            }
            let share = st / r.cit_pop;
            shares.push(share);
            if r.era13 {
                if share > r.band.1 {
                    a += 1;
                } else if share < r.band.0 {
                    b += 1;
                } else {
                    w += 1;
                }
            }
        }
        shares.sort_by(f64::total_cmp);
        println!(
            "DS {name}: era-13 within {w} above {a} below {b}; unordered {unordered}; readings moved {moved} of {}; median standing {:.2} %, max {:.2} %",
            rs.len(),
            100.0 * shares[shares.len() / 2],
            100.0 * shares[shares.len() - 1]
        );
    }
    let _ = rs.iter().map(|r| (r.fee, r.s)).count();
}
