//! Scratch measurement for `OUTSTANDING_WORK.md`'s row "Military manpower: on
//! the industrial rows, a high-state-capacity standing army above its own
//! band" (Ruling DL Q2: measure first, build nothing;
//! `ECOLOGICAL_ARMY_TERM_SCOPE.md` §10.9, 2026-11-17).
//!
//! **Pins nothing and is `#[ignore]`d.** It reads the shipped model
//! (`civ_military_manpower`) over the same driver grid as
//! `manpower::tests::the_ordering_holds_across_the_driver_space_but_one_class`
//! on the `industrial` ag-tech row only, then re-prices the *standing army*
//! under candidate changes **analytically**, outside the model:
//!
//! `standing' = standing * fee'(s) / fee(s) * upkeep / upkeep'(s)`
//!
//! which is exact because chain 1 (`standing = non_agri * E-factor * fee /
//! upkeep`) is the only chain that reads the fiscal fee or the soldier
//! upkeep; the field army and the levy (chain 2) read neither. No shipped
//! constant is touched. Ruling BK: every candidate constant below is either
//! computed here from a cited band edge or labelled a judgement.
//!
//! Run (alone): `cargo test -p cartalith-civ --test industrial_army_driver_space -- --ignored --nocapture`

use cartalith_civ::manpower::{
    civ_military_manpower, military_drivers, ManpowerInput, ManpowerOrder, EXTRACTION_CEILING,
    EXTRACTION_FLOOR,
};
use cartalith_civ::roster::{AG_TECH_LEVELS, CIV_GOVERNMENTS, CIV_GOVERNMENTS_REFERENCE_LEN};

/// One driver-space reading of the shipped model, reduced to what the
/// analytic re-pricing needs.
#[derive(Clone, Copy)]
struct R {
    standing: f64,
    field: f64,
    levy: f64,
    pop: f64,
    cf: f64,
    s: f64,
    u: f64,
    band: (f64, f64),
    gov: usize,
    e: f64,
    l: f64,
}

/// The extraction fee under a given ceiling (the shipped one is
/// `EXTRACTION_CEILING`).
fn fee(s: f64, ceiling: f64) -> f64 {
    EXTRACTION_FLOOR + (ceiling - EXTRACTION_FLOOR) * s
}

/// Every reading of the shipped model on the `industrial` row over the
/// driver grid of `manpower::tests::the_ordering_holds_across_the_driver_space_but_one_class`
/// (17 governments x 5 capital reaches x 11 road densities x navigable and
/// sea each off and on x `E` 0.25..4.0 by 0.05 = 284 240), reduced to the
/// fields the analytic re-pricing reads. Never edits the model.
fn readings() -> Vec<R> {
    let row = AG_TECH_LEVELS.iter().find(|r| r.key == "industrial").unwrap();
    let f = row.farmers_per_urbanite;
    let mut v = Vec::new();
    for (gi, &(gov, _)) in CIV_GOVERNMENTS.iter().enumerate() {
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
                            v.push(R {
                                standing: m.standing_army,
                                field: m.field_army,
                                levy: m.emergency_mobilization,
                                pop: m.total_population,
                                cf: m.drivers.citizen_fraction,
                                s: m.drivers.state_capacity,
                                u: m.drivers.soldier_upkeep,
                                band: m.era_band.standing,
                                gov: gi,
                                e: m.drivers.ecological_factor,
                                l: m.drivers.logistics_capacity,
                            });
                            assert!(matches!(
                                m.ordering,
                                ManpowerOrder::Ordered | ManpowerOrder::StandingNotBelowField
                            ));
                        }
                    }
                }
            }
        }
    }
    v
}

/// Prints, for one candidate `(ceiling, upkeep(s, old))`, how many readings
/// stay unordered, how the era-13 standing verdicts fall, the standing share's
/// median and maximum, and which governments the unordered ones belong to.
/// Reports a measurement; asserts nothing, so no constant is pinned against
/// itself. The candidate constants are fitted to the 10 % band top, not
/// sourced.
fn score(name: &str, rs: &[R], ceiling: f64, up: &dyn Fn(f64, f64) -> f64) {
    let (mut unordered, mut field_ge_levy) = (0usize, 0usize);
    let (mut above, mut below, mut within) = (0usize, 0usize, 0usize);
    let mut shares: Vec<f64> = Vec::new();
    let mut by_gov = vec![0usize; CIV_GOVERNMENTS.len()];
    let (mut smin, mut emin, mut emax, mut lmax) = (9.0f64, 9.0f64, 0.0f64, 0.0f64);
    for r in rs {
        let st = r.standing * fee(r.s, ceiling) / fee(r.s, EXTRACTION_CEILING) * r.u / up(r.s, r.u);
        let ordered = st < r.field && r.field < r.levy;
        if !ordered {
            unordered += 1;
            by_gov[r.gov] += 1;
            smin = smin.min(r.s);
            emin = emin.min(r.e);
            emax = emax.max(r.e);
            lmax = lmax.max(r.l);
        }
        field_ge_levy += usize::from(r.field >= r.levy);
        let share = st / (r.pop * r.cf);
        if r.band.0 > 0.04 && r.band.1 < 0.11 {
            // era 13 (Total industrial mobilization, 5-10 %) only
            if share > r.band.1 {
                above += 1;
            } else if share < r.band.0 {
                below += 1;
            } else {
                within += 1;
            }
        }
        shares.push(share);
    }
    shares.sort_by(f64::total_cmp);
    let med = shares[shares.len() / 2];
    let worst: Vec<String> = by_gov
        .iter()
        .enumerate()
        .filter(|(_, n)| **n > 0)
        .map(|(i, n)| format!("{}:{n}", CIV_GOVERNMENTS[i].0))
        .collect();
    println!(
        "DS {name}: unordered {unordered} (field>=levy {field_ge_levy}); era-13 standing verdicts within {within} above {above} below {below}; median standing {:.2} % of citizens, max {:.2} %; unordered by government [{}]",
        100.0 * med,
        100.0 * shares[shares.len() - 1],
        worst.join(" ")
    );
    if unordered > 0 {
        println!("DS   unordered readings: state capacity from {smin:.3}, E {emin:.2}..{emax:.2}, logistics up to {lmax:.3}");
    }
}

#[test]
#[ignore = "scratch measurement for the industrial standing-army row; pins nothing; run alone"]
fn industrial_army_driver_space_candidates() {
    let rs = readings();
    assert_eq!(rs.len(), 284_240, "the grid is the shipped test's");
    let u0 = rs[0].u;
    let alpha = 0.15 / 1.15;
    // Highest state capacity reachable on this row: empire at full capital
    // reach with the urbanisation term at 1 = 0.70 * 1.0 * 1.30 = 0.91.
    let s_max = rs.iter().map(|r| r.s).fold(0.0, f64::max);
    println!("DS s_max over the grid {s_max:.4}, u0 {u0:.4}, alpha {alpha:.5}");
    let cf = 0.98;
    // The derivation set of SOLDIER_UPKEEP_BY_BRACKET: the reference's nine
    // governments but `none`, capital reach 0..1 by 0.1, E = 1.
    let mut ln_sum = 0.0;
    let mut n = 0.0;
    let mut s_list: Vec<f64> = Vec::new();
    let row = AG_TECH_LEVELS.iter().find(|r| r.key == "industrial").unwrap();
    for &(gov, _) in CIV_GOVERNMENTS[..CIV_GOVERNMENTS_REFERENCE_LEN].iter().filter(|g| g.0 != "none") {
        for step in 0..=10 {
            let d = military_drivers(&ManpowerInput {
                nucleated_pop: 1000.0,
                farmers_per_urbanite: row.farmers_per_urbanite,
                land_capacity: 1000.0 * (1.0 + row.farmers_per_urbanite),
                government: gov,
                capital_road_reach: f64::from(step) / 10.0,
                road_density: 0.0,
                navigable_share: 0.0,
                sea_share: 0.0,
            });
            ln_sum += d.fiscal_extraction_efficiency.ln();
            n += 1.0;
            s_list.push(d.state_capacity);
        }
    }
    let fee_ref = (ln_sum / n).exp();
    s_list.sort_by(f64::total_cmp);
    println!(
        "DS derivation set: geometric-mean fee {fee_ref:.5}, state capacity min {:.3} median {:.3} max {:.3} (n {})",
        s_list[0],
        s_list[s_list.len() / 2],
        s_list[s_list.len() - 1],
        s_list.len()
    );

    let band_top = 0.10;
    // C1: raise bracket 5's upkeep so the highest reachable capacity lands at
    // the band's top (a level change, every capacity scaled alike).
    let u1_exact = (1.0 - alpha) * fee(s_max, EXTRACTION_CEILING) / (cf * band_top);
    // C2: lower the fiscal ceiling on the industrial brackets so the same
    // polity lands at the band's top (a slope change, the floor untouched).
    let fee_needed = band_top * cf * u0 / (1.0 - alpha);
    let c2_exact = EXTRACTION_FLOOR + (fee_needed - EXTRACTION_FLOOR) / s_max;
    println!("DS exact: C1 upkeep {u1_exact:.4}, C2 ceiling {c2_exact:.4}, fee needed {fee_needed:.5}");

    let same = |_s: f64, u: f64| u;
    score("baseline", &rs, EXTRACTION_CEILING, &same);
    // Isolated scans: upkeep alone, then ceiling alone.
    for u in [1.10, 1.15, 1.20, 1.25, 1.30, 1.324, 1.33, 1.40, 1.50] {
        score(&format!("upkeep alone {u:.3}"), &rs, EXTRACTION_CEILING, &|_, _| u);
    }
    for c in [0.15, 0.14, 0.13, 0.128, 0.125, 0.12, 0.11, 0.10] {
        score(&format!("ceiling alone {c:.3}"), &rs, c, &same);
    }
    // Where each isolated change first clears the whole grid (finer scan).
    for k in 0..=12 {
        let u = 1.25 + 0.005 * f64::from(k);
        score(&format!("upkeep fine {u:.3}"), &rs, EXTRACTION_CEILING, &|_, _| u);
    }
    for k in 0..=10 {
        let c = 0.130 + 0.001 * f64::from(k);
        score(&format!("ceiling fine {c:.3}"), &rs, c, &same);
    }
    // C2': the ceiling moved AND the upkeep re-derived at it by the rule
    // `soldier_upkeep_is_derived_from_the_era_table` states (the geometric
    // mean of the standing share over the derivation set sits at the band
    // centre), which is what any ceiling change must do to keep that test
    // green. The upkeep scales with the geometric-mean fee over the set.
    let gm_fee = |c: f64| (s_list.iter().map(|&s| fee(s, c).ln()).sum::<f64>() / s_list.len() as f64).exp();
    for c in [0.15, 0.14, 0.13, 0.12, 0.11, 0.10, 0.09, 0.08, 0.06, 0.04] {
        let u = u0 * gm_fee(c) / gm_fee(EXTRACTION_CEILING);
        score(&format!("C2' ceiling {c:.2} with upkeep re-derived to {u:.4}"), &rs, c, &|_, _| u);
    }
    // C3: the upkeep scales with the fee the polity collects, about the
    // derivation set's geometric mean, so that mean is preserved exactly.
    for theta in [0.25, 0.5, 0.75, 1.0] {
        score(
            &format!("C3 scaled upkeep theta {theta}"),
            &rs,
            EXTRACTION_CEILING,
            &|s, u| u * (fee(s, EXTRACTION_CEILING) / fee_ref).powf(theta),
        );
    }
}
