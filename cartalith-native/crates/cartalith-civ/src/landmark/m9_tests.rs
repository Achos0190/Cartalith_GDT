// Tests for `temporal_kinds.rs` and for the M8/M9 kinds on a generated world. `include!`d into
// `landmark.rs`'s own `mod tests` so they share its fixtures (`flat`, `straight`, `way_tid`,
// `year`, `named`, `only`, `real_world`); items only, no inner attributes.

/// Settings with exactly the given kinds armed, each at its default cap.
fn only_kinds(keys: &[&str]) -> LandmarkSettings {
    let mut s = LandmarkSettings::default();
    for k in kinds() {
        s.set_armed(k.key, keys.contains(&k.key));
    }
    s
}

/// Protects: the present-year split of one record into two kinds. At year 200, a battle that ended
/// in year 104 is a Historic battlefield, an ongoing one and one that ends in year 220 are
/// Battlefields, and the chain of the historic one says how long ago it ended (96 years). With no
/// history at all (a caller with no Timeline) every battle is a Battlefield, as before the split,
/// and Historic battlefield is `NoTerrain`.
#[test]
fn a_battle_that_ended_before_the_present_year_is_a_historic_battlefield() {
    let (gw, gh, width_km) = (128usize, 64usize, 512.0f64);
    let field = flat(gw, gh);
    let battles = [
        BattleMark { x: 20.0, y: 10.0, name: "Old Ford".into(), start_year: 100, end_year: Some(104) },
        BattleMark { x: 60.0, y: 10.0, name: "Open War".into(), start_year: 190, end_year: None },
        BattleMark { x: 100.0, y: 10.0, name: "Coming War".into(), start_year: 180, end_year: Some(220) },
    ];
    let hist = LandmarkHistory { now_year: 200, ..Default::default() };
    let mut inp = LandmarkInputs::new(&field, gw, gh, SEA, false, width_km);
    inp.battles = &battles;
    inp.history = Some(&hist);
    let r = generate(&inp, &only_kinds(&["battlefield", "battlefield_historic"]), 1);
    let at = |k: &str| -> Vec<(usize, usize)> {
        r.landmarks.iter().filter(|l| l.kind == k).map(|l| (l.x, l.y)).collect()
    };
    assert_eq!(at("battlefield_historic"), vec![(20, 10)], "{:?}", r.funnels);
    let mut cur = at("battlefield");
    cur.sort_unstable();
    assert_eq!(cur, vec![(60, 10), (100, 10)]);
    let old = r.landmarks.iter().find(|l| l.kind == "battlefield_historic").unwrap();
    assert!(old.causal.join(" | ").contains("ended 96 years before year 200"), "{:?}", old.causal);
    // No Timeline: nothing is "past".
    let mut bare = LandmarkInputs::new(&field, gw, gh, SEA, false, width_km);
    bare.battles = &battles;
    let r = generate(&bare, &only_kinds(&["battlefield", "battlefield_historic"]), 1);
    assert_eq!(r.placed("battlefield"), 3, "unchanged without a present year");
    assert_eq!(r.funnel("battlefield_historic").unwrap().limit, LandmarkLimit::NoTerrain);
    assert_eq!(r.placed("battlefield_historic"), 0);
}

/// A recorded year whose collapse run left `flags` (tid, fortified, ruins).
fn collapse_year(
    y: i64,
    settlements: Vec<crate::NamedSettlement>,
    flags: &[(u64, bool, bool)],
) -> crate::timeline::TimelineSnapshot {
    let mut s = year(y, settlements, vec![]);
    for (tid, fortified, ruins) in flags {
        s.collapse_flags.insert(*tid, crate::timeline::CollapseFlags { fortified: *fortified, ruins: *ruins });
    }
    s
}

/// Protects: `LandmarkHistory::from_timeline`'s ruin rule, each clause by its own record. Live year
/// 25, live settlements tid 2 and 6. tid 1 is walled and in ruins in year 10 and gone: a ruin, walled.
/// tid 2 is in ruins in year 10 and still lives: a ruin that stands. tid 3 is recorded unflagged:
/// not a ruin. tid 4 is in ruins in year 10 and recovered (an unflagged entry) in year 20: not a
/// ruin, because the LATEST record decides. tid 5 has no flag entry at all: "not recorded", neither
/// ruin nor recovered. tid 7 is in ruins in year 25, the live year itself: a ruin (`<=`, where the
/// former lists use `<`). A ruined settlement leaves the abandoned list.
#[test]
fn the_latest_collapse_record_decides_what_is_in_ruins() {
    let timeline = vec![
        collapse_year(
            10,
            vec![
                named(1, 1, 1, 100, "Walled"),
                named(2, 2, 2, 100, "Standing"),
                named(3, 3, 3, 100, "Fine"),
                named(4, 4, 4, 100, "Mended"),
                named(5, 5, 5, 100, "Unrecorded"),
            ],
            &[(1, true, true), (2, false, true), (3, false, false), (4, false, true)],
        ),
        collapse_year(20, vec![named(4, 4, 4, 90, "Mended")], &[(4, false, false)]),
        collapse_year(25, vec![named(7, 7, 7, 30, "Late")], &[(7, false, true)]),
    ];
    let live_s = vec![named(2, 2, 2, 5, "Standing"), named(6, 6, 6, 50, "Other")];
    let h = LandmarkHistory::from_timeline(&timeline, 25, &live_s, &[]);
    let ruins: Vec<(u64, bool, bool)> = h.ruins.iter().map(|r| (r.site.tid, r.fortified, r.still_live)).collect();
    assert_eq!(ruins, vec![(1, true, false), (2, false, true), (7, false, false)], "{:?}", h.ruins);
    let former: Vec<u64> = h.settlements.iter().map(|s| s.tid).collect();
    assert_eq!(former, vec![3, 4, 5], "tid 1 and 7 are ruins, tid 2 is live: {former:?}");
}

/// Protects: Ruin and Destroyed fortress partition the ruined sites on `fortified`, each placed on
/// its own cell with the year and population in the chain; a ruin a settlement still occupies says
/// so; a ruined site now under water is a constraint reject; the walled one names the nearest
/// recorded battle within 50 km (a battle 5 cells = 20 km away) and a battle 60 cells away is not
/// named; with no collapse record at all both kinds are `NoTerrain`.
#[test]
fn ruins_and_destroyed_fortresses_partition_the_ruined_sites() {
    let (gw, gh, width_km) = (128usize, 64usize, 512.0f64);
    let mut field = flat(gw, gh);
    field[50 * gw + 100] = 0.10;
    let site = |tid: u64, x: usize, y: usize, name: &str| FormerSettlement {
        tid,
        x,
        y,
        name: name.into(),
        last_year: 60,
        population: 700.0,
    };
    let hist = LandmarkHistory {
        now_year: 100,
        ruins: vec![
            RuinedSite { site: site(1, 30, 20, "Brokenhold"), fortified: true, still_live: false },
            RuinedSite { site: site(2, 70, 20, "Hollowtown"), fortified: false, still_live: true },
            RuinedSite { site: site(3, 100, 50, "Drowned"), fortified: false, still_live: false },
        ],
        ..Default::default()
    };
    let battles = [
        BattleMark { x: 35.0, y: 20.0, name: "Siege Field".into(), start_year: 55, end_year: Some(58) },
        BattleMark { x: 95.0, y: 20.0, name: "Far Field".into(), start_year: 55, end_year: Some(58) },
    ];
    let mut inp = LandmarkInputs::new(&field, gw, gh, SEA, false, width_km);
    inp.history = Some(&hist);
    inp.battles = &battles;
    let r = generate(&inp, &only_kinds(&["ruin", "destroyed_fortress"]), 1);
    let pos = |k: &str| -> Vec<(usize, usize)> {
        r.landmarks.iter().filter(|l| l.kind == k).map(|l| (l.x, l.y)).collect()
    };
    assert_eq!(pos("ruin"), vec![(70, 20)], "{:?}", r.funnels);
    assert_eq!(pos("destroyed_fortress"), vec![(30, 20)]);
    let ruin = r.landmarks.iter().find(|l| l.kind == "ruin").unwrap();
    let chain = ruin.causal.join(" | ");
    assert!(chain.contains("Hollowtown was recorded in ruins in 60"), "{chain}");
    assert!(chain.contains("population 700"), "{chain}");
    assert!(chain.contains("still stands on the site"), "{chain}");
    let fort = r.landmarks.iter().find(|l| l.kind == "destroyed_fortress").unwrap();
    let chain = fort.causal.join(" | ");
    assert!(chain.contains("Brokenhold was recorded walled and in ruins in 60"), "{chain}");
    assert!(chain.contains("Siege Field was fought 20 km away"), "{chain}");
    assert!(!chain.contains("Far Field"), "a battle 260 km away is no part of this story: {chain}");
    assert_eq!(r.funnel("ruin").unwrap().rejected_constraint, 1, "the drowned site");
    let none = LandmarkInputs::new(&field, gw, gh, SEA, false, width_km);
    let r = generate(&none, &only_kinds(&["ruin", "destroyed_fortress"]), 1);
    for k in ["ruin", "destroyed_fortress"] {
        assert_eq!(r.funnel(k).unwrap().limit, LandmarkLimit::NoTerrain, "{k}");
        assert_eq!(r.funnel(k).unwrap().candidates, 0, "{k}");
    }
}

/// Protects: Historic crossing's definition. A recorded road (20, 32) -> (60, 32) crosses a river down
/// column 40: one crossing, on the river cell, with the road's class and the river's flow in the
/// chain. The same crossing under a live road is rejected (the world still carries it); and the river
/// the road never reached yields none. Without the channel mask the kind is `NoTerrain`.
#[test]
fn a_historic_crossing_is_a_river_a_recorded_road_crossed_and_no_live_road_does() {
    let (gw, gh, width_km) = (128usize, 64usize, 512.0f64);
    let field = flat(gw, gh);
    let mut chan = vec![0u8; gw * gh];
    let mut flow = vec![0f32; gw * gh];
    for y in 0..gh {
        chan[y * gw + 40] = 1;
        flow[y * gw + 40] = 1000.0;
    }
    let hist = LandmarkHistory {
        now_year: 100,
        ways: vec![FormerWay { way: way_tid(9, (20, 32), (60, 32), false), last_year: 80 }],
        ..Default::default()
    };
    let run = |live: &[crate::Way], with_chan: bool| {
        let mut inp = LandmarkInputs::new(&field, gw, gh, SEA, false, width_km);
        if with_chan {
            inp.channel = Some(&chan);
            inp.flow = Some(&flow);
        }
        inp.ways = live;
        inp.history = Some(&hist);
        generate(&inp, &only_kinds(&["historic_crossing"]), 1)
    };
    let r = run(&[], true);
    let lm: Vec<&Landmark> = r.landmarks.iter().filter(|l| l.kind == "historic_crossing").collect();
    assert_eq!(lm.len(), 1, "{:?}", r.funnel("historic_crossing"));
    assert_eq!((lm[0].x, lm[0].y), (40, 32));
    let chain = lm[0].causal.join(" | ");
    assert!(chain.contains("crossed this river, last recorded in 80"), "{chain}");
    assert!(chain.contains("1 000 cells of flow"), "{chain}");
    let live = [straight((30, 32), (50, 32), crate::WayType::Road)];
    let r = run(&live, true);
    assert_eq!(r.placed("historic_crossing"), 0, "a live road still crosses there");
    assert_eq!(r.funnel("historic_crossing").unwrap().rejected_constraint, 1);
    let r = run(&[], false);
    assert_eq!(r.funnel("historic_crossing").unwrap().limit, LandmarkLimit::NoTerrain);
}

/// A landmark of `kind` at `(x, y)` for the phase tests; every other field is inert there.
fn phase_lm(kind: &str, x: usize, y: usize) -> Landmark {
    Landmark {
        id: 1,
        kind: kind.into(),
        class: LandmarkClass::Local,
        x,
        y,
        elevation: 0.0,
        score: 0.0,
        importance: 0.0,
        causal: vec![],
        seed: 0,
    }
}

/// Protects: the phase of a landmark at the present year, in the order the rules say. On a 4 km grid
/// a waterfall 2 cells (8 km) from a settlement is Utilised, one 25 cells (100 km) away is Natural;
/// a fort is Developed; a battlefield Monumentalised; a ruin Ruined and an ancient road Abandoned
/// whatever stands beside them. With no settlement at all there is no record of people either way, so the
/// phase is `None`, not Natural. The same waterfall changes phase when the world's settlements
/// change, which is what makes it a temporal state.
#[test]
fn a_landmarks_phase_follows_the_world_at_the_present_year() {
    let near = [LandmarkSite { x: 12, y: 10, population: 500.0 }];
    let far = [LandmarkSite { x: 35, y: 10, population: 500.0 }];
    let at = |sites: &[LandmarkSite], kind: &str| {
        let inp = PhaseInputs { gw: 128, world: false, cell_km: 4.0, settlements: sites };
        landmark_phase(&phase_lm(kind, 10, 10), &inp)
    };
    assert_eq!(at(&near, "waterfall").unwrap().0, LandmarkPhase::Utilised);
    assert_eq!(at(&far, "waterfall").unwrap().0, LandmarkPhase::Natural);
    assert_eq!(at(&far, "fort").unwrap().0, LandmarkPhase::Developed);
    assert_eq!(at(&far, "battlefield").unwrap().0, LandmarkPhase::Monumentalised);
    assert_eq!(at(&near, "ruin").unwrap().0, LandmarkPhase::Ruined);
    assert_eq!(at(&near, "ancient_road").unwrap().0, LandmarkPhase::Abandoned);
    assert!(at(&[], "waterfall").is_none(), "no settlement anywhere: no claim either way");
    assert_eq!(at(&near, "waterfall").unwrap().1, "a settlement lives 8 km away");
    assert_eq!(LandmarkPhase::Utilised.key(), "utilised");
}

/// Protects: the 10 km boundary of the phase's use radius, by literal: a settlement 10 km away
/// uses a feature, one 12 km away does not (`PHASE_USE_KM` is 10, a labelled judgement).
#[test]
fn the_phase_use_radius_is_ten_kilometres() {
    assert_eq!(PHASE_USE_KM, 10.0);
    let phase = |cells: usize| {
        let sites = [LandmarkSite { x: 10 + cells, y: 10, population: 1.0 }];
        let inp = PhaseInputs { gw: 128, world: false, cell_km: 1.0, settlements: &sites };
        landmark_phase(&phase_lm("cliff", 10, 10), &inp).unwrap().0
    };
    assert_eq!(phase(10), LandmarkPhase::Utilised);
    assert_eq!(phase(12), LandmarkPhase::Natural);
}

/// Protects: every new kind places on a world `generate_terrain` really made, or says why not, so
/// a flag flipped to buildable cannot ship a kind that is silently empty. The world is
/// `WorldParams::defaults` (the glacial pass off, so Glacial feature is `NoTerrain` here by
/// design) at 256 x 192 seed 24601; the temporal kinds need a Timeline this fixture lacks, so they
/// are `NoTerrain` too. Delta, Ancient forest and Salt works are the three that must have
/// candidates, and the print is the measurement `STATUS.md` quotes.
#[test]
fn the_m8_and_m9_kinds_place_on_a_world_generate_terrain_really_made() {
    for seed in [24601, 483920, 912345] {
        let r = real_world(256, 192, seed);
        let pairs = real_resource_pairs(&r.resources);
        let inp = real_inputs(&r, &pairs);
        let out = generate(&inp, &LandmarkSettings::default(), 7);
        for k in [
            "delta",
            "ancient_forest",
            "salt_works",
            "glacial_feature",
            "sacred_mountain",
            "sacred_grove",
            "shrine",
            "ceremonial_site",
        ] {
            let f = out.funnel(k).unwrap();
            eprintln!("seed {seed} {k:16} candidates {:5} placed {:3} limit {:?}", f.candidates, f.placed, f.limit);
        }
        assert_eq!(out.funnel("glacial_feature").unwrap().limit, LandmarkLimit::NoTerrain, "pass off by design");
        // Delta and Ancient forest are the physical kinds that must find ground on every seed.
        assert!(out.placed("delta") > 0, "seed {seed}: no delta: {:?}", out.funnel("delta"));
        assert!(out.placed("ancient_forest") > 0, "seed {seed}: no intact forest: {:?}", out.funnel("ancient_forest"));
    }
}

/// Protects: the same, with the glacial pass on (the app's setting) and its snowline read, on a
/// world cold enough to have ice: a plain `WorldParams` world with `passes.glacial` set generates
/// one. Prints the funnel; asserts it ran and that every placement is inside the pass's own domain
/// (above the snowline, below freezing) - the one claim the kind makes.
#[test]
fn glacial_features_sit_only_inside_the_glacial_domain_on_a_generated_world() {
    let mut p = cartalith_engine::WorldParams::defaults(256, 192, 24601);
    p.world = false;
    p.passes.glacial = true;
    let ws = cartalith_engine::generate_terrain(&p);
    let (gw, gh, sea) = (256usize, 192usize, ws.sea_level);
    let wb = crate::build_water_bodies(&ws.field, gw, gh, sea, false, Some(&ws.rainfall));
    let snow = sea + (1.0 - sea) * p.passes.glacial_snowline;
    let mut inp = LandmarkInputs::new(&ws.field, gw, gh, sea, false, p.map_width_km);
    inp.flow = Some(&ws.flow_discharge);
    inp.channel = ws.channels.as_ref().map(|c| c.chan.as_slice());
    inp.water = Some(&wb.classification);
    inp.temperature = Some(&ws.temperature);
    inp.glacial_snow_el = Some(snow);
    let out = generate(&inp, &only_kinds(&["glacial_feature"]), 7);
    let f = out.funnel("glacial_feature").unwrap();
    eprintln!("glacial_feature  candidates {:5} placed {:3} limit {:?}", f.candidates, f.placed, f.limit);
    for l in &out.landmarks {
        let i = l.y * gw + l.x;
        assert!(ws.temperature[i] < 0.0 && ws.field[i] as f64 >= snow, "{:?} is outside the glacial domain", (l.x, l.y));
    }
}

/// Measurement, not a bar: how much the new kinds move what a world already showed. Runs the
/// default settings with the 12 new kinds disarmed and armed on three seeds and prints, per
/// seed, the landmarks of the old kinds that are in one result and not the other. `cargo test
/// -- --ignored --nocapture displacement` reproduces the figures `STATUS.md` quotes.
#[test]
#[ignore = "measurement; prints per-seed displacement of the old kinds"]
fn displacement_of_old_kinds_by_the_new_ones() {
    const NEW: [&str; 12] = [
        "delta",
        "ancient_forest",
        "salt_works",
        "glacial_feature",
        "battlefield_historic",
        "ruin",
        "destroyed_fortress",
        "historic_crossing",
        "shrine",
        "sacred_grove",
        "sacred_mountain",
        "ceremonial_site",
    ];
    // The three 256 x 192 census seeds, and the owner's world (246371 at 2048 x 1311).
    for (gw, gh, seed) in [(256, 192, 24601), (256, 192, 483920), (256, 192, 912345), (2048, 1311, 246371)] {
        let r = real_world(gw, gh, seed);
        let pairs = real_resource_pairs(&r.resources);
        let inp = real_inputs(&r, &pairs);
        let mut off = LandmarkSettings::default();
        for k in NEW {
            off.set_armed(k, false);
        }
        let before = generate(&inp, &off, 7);
        let after = generate(&inp, &LandmarkSettings::default(), 7);
        let key = |l: &Landmark| (l.kind.clone(), l.x, l.y);
        let old_after: std::collections::HashSet<_> =
            after.landmarks.iter().filter(|l| !NEW.contains(&l.kind.as_str())).map(key).collect();
        let lost = before.landmarks.iter().filter(|l| !old_after.contains(&key(l))).count();
        let added = after.landmarks.iter().filter(|l| NEW.contains(&l.kind.as_str())).count();
        eprintln!(
            "seed {seed}: {} old landmarks before, {} lost to the new kinds, {} new kinds placed",
            before.landmarks.len(),
            lost,
            added
        );
        for k in NEW {
            let f = after.funnel(k).unwrap();
            eprintln!("    {k:20} candidates {:5} placed {:3} {:?}", f.candidates, f.placed, f.limit);
        }
    }
}

/// Measurement, not a bar: why Delta's candidates are rejected on a real world. For each river
/// mouth it prints how many pass the flow test and the distribution of the plain share, so the
/// 0.35 floor and the 10 m elevation are chosen against data (`MISTAKES.md`: measure before
/// pinning a constant). `cargo test -- --ignored --nocapture delta_plain` reproduces it.
#[test]
#[ignore = "measurement; prints the delta plain-share distribution per seed"]
fn delta_plain_share_distribution() {
    for (gw, gh, seed) in [(256, 192, 24601), (256, 192, 483920), (256, 192, 912345), (2048, 1311, 246371)] {
        let r = real_world(gw, gh, seed);
        let ch = r.ws.channels.as_ref().unwrap();
        let fl = &r.ws.flow_discharge;
        let thresh = cartalith_hydrology::river_flow_thresh(r.gw, r.gh, r.gw, r.width_km);
        
        let rad = (12.0 / (r.width_km / r.gw as f64)).round().clamp(1.0, 40.0) as i64;
        let mut flows_ok = 0;
        let mut fracs: Vec<f64> = vec![];
        let mut mouths = 0;
        let mut max_ratio = 0f64;
        let mut order_ok = 0;
        for i in 0..r.gw * r.gh {
            if ch.chan[i] == 0 || r.wb[i] != 0 {
                continue;
            }
            let rv = ch.recv[i];
            if rv < 0 || r.wb[rv as usize] == 0 {
                continue;
            }
            mouths += 1;
            max_ratio = max_ratio.max(fl[i] as f64 / thresh);
            if r.order[i] >= crate::river_class::NAVIGABLE_ORDER {
                order_ok += 1;
            }
            if (fl[i] as f64) < thresh * 8.0 {
                continue;
            }
            flows_ok += 1;
            let (x, y) = ((i % r.gw) as i64, (i / r.gw) as i64);
            let (mut land, mut plain) = (0, 0);
            for dy in -rad..=rad {
                for dx in -rad..=rad {
                    let (xx, yy) = (x + dx, y + dy);
                    if xx < 0 || yy < 0 || xx >= r.gw as i64 || yy >= r.gh as i64 {
                        continue;
                    }
                    let j = yy as usize * r.gw + xx as usize;
                    if r.wb[j] == 0 {
                        land += 1;
                        if r.ws.field[j] as f64 <= r.sea + crate::river_class::DELTA_LOWLAND_DH {
                            plain += 1;
                        }
                    }
                }
            }
            fracs.push(plain as f64 / land.max(1) as f64);
        }
        fracs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let q = |p: f64| fracs.get(((fracs.len() as f64 - 1.0) * p) as usize).copied().unwrap_or(f64::NAN);
        eprintln!(
            "seed {seed}: mouths {mouths}, flow-ok {flows_ok} (largest mouth {max_ratio:.1}x the channel threshold; {order_ok} mouths at Strahler order >= 3), plain share min/med/max {:.2}/{:.2}/{:.2}, cell {:.2} km, rad {rad}",
            q(0.0),
            q(0.5),
            q(1.0),
            r.width_km / r.gw as f64
        );
    }
}

/// Faiths for faction ids `0..=n`, cycling through the owner-approved religions each cultural kind
/// reads (0 is the unclaimed entry, `none`): Sky Pantheon (hills), Old Gods (forest and shrines),
/// Earth Mother (shrines, place-bound), Ancestor Rites (lineage-bound), Sea Lords, Sun Cult. A
/// fixture, so the cultural kinds have a world to read; the real mapping is the roster's.
fn fixture_faiths(n: usize) -> Vec<FactionFaith> {
    const CYCLE: [(&str, &str); 6] = [
        ("sky_pantheon", "Sky Pantheon"),
        ("old_gods", "Old Gods"),
        ("earth_mother", "Earth Mother"),
        ("ancestor_rites", "Ancestor Rites"),
        ("sea_lords", "Sea Lords"),
        ("sun_cult", "Sun Cult"),
    ];
    (0..=n)
        .map(|i| {
            let (key, label) = if i == 0 { ("none", "No religion") } else { CYCLE[(i - 1) % CYCLE.len()] };
            FactionFaith {
                faction_name: format!("Faction {i}"),
                religion: key.to_string(),
                religion_label: label.to_string(),
            }
        })
        .collect()
}

/// Protects: each row of the four readings against the owner-approved tables, one faith at a time.
/// Sky Pantheon (the hills' faith) reads high places and nothing else; Old Gods reads groves and
/// raises shrines; Spirit Ways reads groves by its own row; Earth Mother raises shrines and is bound
/// to a place; Ancestor Rites is bound to a lineage (ceremonial) and raises no shrine; Sun Cult
/// (civic, arid) reads none of the four; `none` and an unknown key read nothing.
#[test]
fn each_reading_follows_the_approved_religion_tables() {
    use cultural_kinds::Reading::*;
    let faith = |key: &str| FactionFaith {
        faction_name: "F".into(),
        religion: key.into(),
        religion_label: key.into(),
    };
    let reads = |r: cultural_kinds::Reading, key: &str| r.reads(&faith(key)).is_some();
    // (faith, [high places, grove, shrine, ceremonial])
    for (key, want) in [
        ("sky_pantheon", [true, false, false, false]),
        ("old_gods", [false, true, true, true]),
        ("spirit_ways", [false, true, false, true]),
        ("earth_mother", [false, false, true, true]),
        ("sea_lords", [false, false, true, true]),
        ("ancestor_rites", [false, false, false, true]),
        ("sun_cult", [false, false, false, false]),
        ("none", [false, false, false, false]),
        ("", [false, false, false, false]),
        ("no_such_faith", [false, false, false, false]),
    ] {
        let got = [reads(HighPlaces, key), reads(Grove, key), reads(Shrine, key), reads(Ceremonial, key)];
        assert_eq!(got, want, "{key}");
    }
    assert!(HighPlaces.reads(&faith("sky_pantheon")).unwrap().contains("hills"));
}

/// Protects: a cultural kind is a reading of a physical one, not a second detector. On the fixture
/// world with cross-type competition ON (the app's setting): with every faction holding Sky
/// Pantheon, some Sacred mountain stands on the very cell of a Peak (an overlay is not blocked by,
/// and does not block, the shared field); with every faction holding no faith, the same peaks are
/// candidates and none is sacred; and with no faith list at all the kind is `NoTerrain`.
#[test]
fn a_sacred_mountain_is_a_peak_a_faith_reads_and_shares_its_cell() {
    let w = world(256, 192, 1000.0);
    let res: Vec<(&str, &[f32])> = vec![];
    let settings = only_kinds(&["peak", "sacred_mountain"]);
    let run = |faiths: &[FactionFaith]| {
        let mut inp = inputs(&w, &res);
        inp.faiths = faiths;
        generate(&inp, &settings, 3)
    };
    let sky: Vec<FactionFaith> = (0..=8)
        .map(|i| FactionFaith {
            faction_name: format!("F{i}"),
            religion: if i == 0 { "none".into() } else { "sky_pantheon".into() },
            religion_label: "Sky Pantheon".into(),
        })
        .collect();
    let r = run(&sky);
    let peaks: std::collections::HashSet<(usize, usize)> =
        r.landmarks.iter().filter(|l| l.kind == "peak").map(|l| (l.x, l.y)).collect();
    let sacred: Vec<&Landmark> = r.landmarks.iter().filter(|l| l.kind == "sacred_mountain").collect();
    assert!(!sacred.is_empty(), "{:?}", r.funnel("sacred_mountain"));
    assert!(sacred.iter().any(|l| peaks.contains(&(l.x, l.y))), "no sacred mountain shares a peak's cell");
    assert!(sacred[0].causal[0].contains("holds high places sacred"), "{:?}", sacred[0].causal);
    let none: Vec<FactionFaith> = (0..=8)
        .map(|i| FactionFaith { faction_name: format!("F{i}"), religion: "none".into(), religion_label: "no religion".into() })
        .collect();
    let r = run(&none);
    assert_eq!(r.placed("sacred_mountain"), 0, "a peak in a land with no faith is simply a peak");
    assert!(r.funnel("sacred_mountain").unwrap().rejected_constraint > 0, "the candidates were refused, not absent");
    assert!(r.placed("peak") > 0);
    let r = run(&[]);
    assert_eq!(r.funnel("sacred_mountain").unwrap().limit, LandmarkLimit::NoTerrain);
    assert_eq!(r.funnel("sacred_mountain").unwrap().candidates, 0);
}

/// Protects: the boundary of the present-year split. A battle whose end year IS the present year has
/// not ended before it: it stays a Battlefield (end 200 at year 200), and one that ended in year 199
/// is the Historic battlefield. The pair of cells are far apart so spacing cannot be the reason.
#[test]
fn a_battle_ending_in_the_present_year_is_still_a_battlefield() {
    let (gw, gh, width_km) = (128usize, 64usize, 512.0f64);
    let field = flat(gw, gh);
    let battles = [
        BattleMark { x: 20.0, y: 10.0, name: "Ends Today".into(), start_year: 190, end_year: Some(200) },
        BattleMark { x: 90.0, y: 10.0, name: "Ended Yesterday".into(), start_year: 190, end_year: Some(199) },
    ];
    let hist = LandmarkHistory { now_year: 200, ..Default::default() };
    let mut inp = LandmarkInputs::new(&field, gw, gh, SEA, false, width_km);
    inp.battles = &battles;
    inp.history = Some(&hist);
    let r = generate(&inp, &only_kinds(&["battlefield", "battlefield_historic"]), 1);
    let at = |k: &str| -> Vec<(usize, usize)> {
        r.landmarks.iter().filter(|l| l.kind == k).map(|l| (l.x, l.y)).collect()
    };
    assert_eq!(at("battlefield"), vec![(20, 10)]);
    assert_eq!(at("battlefield_historic"), vec![(90, 10)]);
}

/// Protects: ground nobody owns is nobody's holy ground. With every cell unclaimed (territory 0) and the
/// unclaimed entry (faction 0) holding Sky Pantheon, no Sacred mountain is placed, though the peaks are
/// there and are counted as refused.
#[test]
fn unowned_ground_is_not_sacred_even_if_the_unclaimed_entry_has_a_faith() {
    let w = world(256, 192, 1000.0);
    let res: Vec<(&str, &[f32])> = vec![];
    // Every cell unclaimed (territory 0), and the unclaimed entry itself holds Sky Pantheon: were
    // ground nobody owns read through faction 0's faith, every peak would be sacred.
    let nobody = vec![0i32; w.gw * w.gh];
    let faiths: Vec<FactionFaith> = (0..=8)
        .map(|i| FactionFaith {
            faction_name: format!("F{i}"),
            religion: if i == 0 { "sky_pantheon".into() } else { "none".into() },
            religion_label: "x".into(),
        })
        .collect();
    let mut inp = inputs(&w, &res);
    inp.territory = Some(&nobody);
    inp.faiths = &faiths;
    let r = generate(&inp, &only_kinds(&["peak", "sacred_mountain"]), 3);
    assert!(r.placed("peak") > 0, "the fixture has peaks");
    assert_eq!(r.placed("sacred_mountain"), 0, "{:?}", r.funnel("sacred_mountain"));
    assert!(r.funnel("sacred_mountain").unwrap().rejected_constraint > 0, "the peaks were refused, not absent");
}

/// Measurement, not a bar: which of Salt works' three tests refuses the coastal dry cells of the
/// owner's world (246371 at 2048 x 1311) and of a 256 x 192 census seed. Prints the share of
/// candidates failing each test alone, so a km-scaled threshold that a cell cap has distorted at
/// high resolution is seen (`SCALE_MAX_CELLS`' own caveat, §28).
#[test]
#[ignore = "measurement; prints which salt-works constraint refuses candidates"]
fn salt_constraint_breakdown() {
    for (gw, gh, seed) in [(256, 192, 912345), (2048, 1311, 246371)] {
        let r = real_world(gw, gh, seed);
        let mut inp = LandmarkInputs::new(&r.ws.field, gw, gh, r.sea, r.world, r.width_km);
        inp.water = Some(&r.wb);
        inp.biome = Some(&r.biome);
        let ctx = Ctx::build(&inp, Needs::of("salt_works"));
        let rr = ctx.cells(HARBOUR_SHELTER_KM, HARBOUR_SHELTER_MAX_CELLS).max(2);
        let (mut n, mut elev, mut grad, mut shel) = (0usize, 0usize, 0usize, 0usize);
        let mut pass = [0usize; 4];
        let mut elevs: Vec<f64> = vec![];
        for i in 0..gw * gh {
            if !ctx.is_land(i) || !natural_kinds::SALT_BIOMES_FOR_TEST.contains(&r.biome[i]) {
                continue;
            }
            let (x, y) = (i % gw, i / gw);
            let coastal = [(-1i64, 0i64), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)]
                .iter()
                .any(|(dx, dy)| ctx.nb(x, y, *dx, *dy).is_some_and(|j| ctx.is_ocean(j)));
            if !coastal {
                continue;
            }
            n += 1;
            elevs.push(ctx.elev_m(i));
            if ctx.elev_m(i) > 10.0 { elev += 1; }
            if ctx.gradient(i) > 0.02 { grad += 1; }
            if natural_kinds::approach(&ctx, x, y, rr).0 < HARBOUR_MIN_SHELTER { shel += 1; }
            let sheltered = natural_kinds::approach(&ctx, x, y, rr).0 >= HARBOUR_MIN_SHELTER;
            for (k, (g, e)) in [(0.06, 30.0), (0.06, 60.0), (0.06, 100.0), (0.10, 60.0)].iter().enumerate() {
                if sheltered && ctx.gradient(i) <= *g && ctx.elev_m(i) <= *e {
                    pass[k] += 1;
                }
            }
        }
        eprintln!("  pass (grad<=.06,elev<=30 | .06,60 | .06,100 | .10,60): {pass:?}");
        elevs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = elevs.get(elevs.len() / 2).copied().unwrap_or(f64::NAN);
        eprintln!(
            "{gw}x{gh} seed {seed}: cell {:.2} km, {n} dry coastal cells; fail elevation {elev}, gradient {grad}, shelter {shel}; median elevation {med:.1} m; shelter window {rr} cells",
            r.width_km / gw as f64
        );
    }
}
