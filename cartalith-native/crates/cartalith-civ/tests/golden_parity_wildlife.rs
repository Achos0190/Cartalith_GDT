//! Golden-parity test for the wildlife / ecoregion layer
//! (`PARITY_TESTING.md`) — reference HTML lines 6489-6620 (`buildTRI`,
//! `WILD_GUILDS`/`WILD_GUILD_LABELS`/`guildTrophic`, `WILD_ROSTERS`,
//! `buildEcoregions`, `wildSig2`, `regionRichness`, `assignWildlife`,
//! `wildRegionColor`, `currentWildlife`) plus `wildFmtPop` (line 8257).
//!
//! Captured from a real `generate()` at `gw=48 gh=32 seed=24601 world=true
//! mapWidthKm=4000` in the Node `vm.runInContext` harness this port already
//! uses, followed by a real `currentWildlife()` call. Every *input* to the
//! segmentation — the Cartalith biome grid, NPP, TRI, water access and
//! carrying capacity — was captured alongside the output, so this suite
//! feeds the reference's own upstream fields in rather than re-deriving
//! them (each of those four already has its own golden suite in this crate).
//!
//! `buildNPP` is **not** re-tested here: it was ported for the carrying-
//! capacity chain long before this layer existed, and `currentNPP()` is
//! simply consumed as an input the way the reference consumes it.
//!
//! Assertions are exact on the raster and on every scalar the reference
//! prints. The one thing compared as a JSON-shaped whole rather than
//! bit-for-bit is `biomassRel`, which the reference itself rounds to two
//! decimals before storing.

use cartalith_civ::wildlife::{
    build_ecoregions, build_tri, current_wildlife, guild_trophic, region_richness, wild_fmt_pop, wild_region_color, wild_roster, wild_sig2,
    Ecoregion, RichnessOpts, Trophic, WILD_GUILDS, WILD_GUILD_LABELS,
};

fn f32s(v: &serde_json::Value) -> Vec<f32> {
    v.as_array().unwrap().iter().map(|x| x.as_f64().unwrap() as f32).collect()
}

fn u8s(v: &serde_json::Value) -> Vec<u8> {
    v.as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u8).collect()
}

/// Every non-integer `f64` in this fixture is stored as a **string**, and
/// read back with Rust's own `str::parse` rather than `Value::as_f64()`.
///
/// That is not decoration: `serde_json`'s float parser is off by one ULP on
/// at least one of the captured aggregates (`0.13124039136818139` parses to
/// bits `3fc0cc7c326b981f`, where `str::parse::<f64>` and V8 both give
/// `…981e`), which would fail an exact golden assertion for a reason with
/// nothing to do with the port. JS's own `Number -> String` is
/// shortest-round-trip, so `parse` recovers the exact double. Integers are
/// still emitted as JSON numbers and are exact either way.
fn num(v: &serde_json::Value) -> f64 {
    match v {
        serde_json::Value::String(s) => s.parse().expect("captured f64 string should parse"),
        other => other.as_f64().expect("captured value should be a number"),
    }
}

fn fixture() -> serde_json::Value {
    let s = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/wildlife_captured.json"))
        .expect("wildlife_captured.json fixture should read");
    serde_json::from_str(&s).expect("fixture should parse")
}

struct World {
    gw: usize,
    gh: usize,
    sea: f64,
    cell_km: f64,
    field: Vec<f32>,
    cart_biome: Vec<u8>,
    npp: Vec<f32>,
    tri: Vec<f32>,
    water: Vec<f32>,
    k: Vec<f32>,
}

fn world(v: &serde_json::Value) -> World {
    World {
        gw: v["gw"].as_u64().unwrap() as usize,
        gh: v["gh"].as_u64().unwrap() as usize,
        sea: num(&v["sea"]),
        cell_km: num(&v["cell_km"]),
        field: f32s(&v["field"]),
        cart_biome: u8s(&v["cart_biome"]),
        npp: f32s(&v["npp"]),
        tri: f32s(&v["tri"]),
        water: f32s(&v["water"]),
        k: f32s(&v["K"]),
    }
}

/// The reference's `latAt(y)` for the captured world (world mode, poles at
/// the grid edges) — captured as a whole ladder so this suite does not
/// re-derive it either.
fn lat_ladder(v: &serde_json::Value) -> Vec<f64> {
    let gh = v["gh"].as_u64().unwrap() as usize;
    (0..gh).map(|y| 90.0 - (y as f64 / (gh - 1) as f64) * 180.0).collect()
}

#[test]
fn build_tri_matches_the_reference_wrapped_and_clamped() {
    let v = fixture();
    let w = world(&v);
    let got = build_tri(&w.field, w.gw, w.gh, true);
    assert_eq!(got.len(), w.gw * w.gh, "the fixture must be a whole grid");
    assert_eq!(got, w.tri);

    // The non-wrapping variant is a genuinely different field, so `wrap`
    // is not dead.
    let clamped = build_tri(&w.field, w.gw, w.gh, false);
    assert_eq!(clamped, f32s(&v["tri_nowrap"]));
    assert_ne!(clamped, got);

    let distinct: std::collections::BTreeSet<u32> = got.iter().map(|x| x.to_bits()).collect();
    assert!(
        distinct.len() > 500,
        "only {} distinct values -- too flat to be measuring anything",
        distinct.len()
    );
}

#[test]
fn the_guild_tables_match_the_reference() {
    let v = fixture();
    let guilds: Vec<String> = v["guilds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g.as_str().unwrap().to_string())
        .collect();
    assert_eq!(WILD_GUILDS.to_vec(), guilds);
    let trophic: Vec<String> = v["guild_trophic"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g.as_str().unwrap().to_string())
        .collect();
    for (i, g) in WILD_GUILDS.iter().enumerate() {
        let want = match trophic[i].as_str() {
            "pred" => Trophic::Pred,
            "scav" => Trophic::Scav,
            _ => Trophic::Herb,
        };
        assert_eq!(guild_trophic(g), want, "guildTrophic({g})");
    }
    let labels = v["guild_labels"].as_object().unwrap();
    for (i, g) in WILD_GUILDS.iter().enumerate() {
        assert_eq!(WILD_GUILD_LABELS[i], labels[*g].as_str().unwrap(), "label for {g}");
    }
}

/// Every roster, entry for entry, in the reference's own order — the order
/// is load-bearing, since `assignWildlife` slices the first `rich` of them.
#[test]
fn every_roster_matches_the_reference_entry_for_entry_and_in_order() {
    let v = fixture();
    let rosters = v["rosters"].as_object().unwrap();
    let mut seen = 0;
    for (key, arr) in rosters {
        let biome: u8 = key.parse().unwrap();
        let expected = arr.as_array().unwrap();
        let got = wild_roster(biome);
        assert_eq!(got.len(), expected.len(), "roster length for biome {biome}");
        for (i, en) in expected.iter().enumerate() {
            let a = en.as_array().unwrap();
            assert_eq!(got[i].name, a[0].as_str().unwrap(), "biome {biome} slot {i} name");
            assert_eq!(got[i].guild, a[1].as_str().unwrap(), "biome {biome} slot {i} guild");
            assert_eq!(got[i].mass_kg, num(&a[2]), "biome {biome} slot {i} mass");
            let gate = a.get(3).and_then(|g| g.as_str());
            let got_gate = match got[i].gate {
                cartalith_civ::wildlife::Gate::Ridge => Some("ridge"),
                cartalith_civ::wildlife::Gate::Coastal => Some("coastal"),
                cartalith_civ::wildlife::Gate::None => None,
            };
            assert_eq!(got_gate, gate, "biome {biome} slot {i} gate");
        }
        seen += 1;
    }
    assert_eq!(seen, 15, "the reference defines a roster for all 15 Cartalith biomes");
}

#[test]
fn wild_sig2_and_wild_fmt_pop_match_the_reference() {
    let v = fixture();
    let inputs = [0.0, -3.0, 1.0, 7.0, 999.0, 1234.0, 98765.0, 0.004];
    let expected: Vec<f64> = v["sig2_str"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().parse().unwrap())
        .collect();
    for (i, x) in inputs.iter().enumerate() {
        assert_eq!(wild_sig2(*x), expected[i], "wildSig2({x})");
    }
    let pops = [
        0.0, 1.0, 7.0, 999.0, 1000.0, 1049.0, 1050.0, 12345.0, 999999.0, 1000000.0, 1049999.0, 12345678.0,
    ];
    let expected: Vec<String> = v["fmt_pop"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    for (i, n) in pops.iter().enumerate() {
        assert_eq!(wild_fmt_pop(*n), expected[i], "wildFmtPop({n})");
    }
}

#[test]
fn build_ecoregions_matches_the_reference_on_a_real_world() {
    let v = fixture();
    let w = world(&v);
    let lats = lat_ladder(&v);
    let eco = build_ecoregions(
        &w.cart_biome,
        &w.field,
        &w.npp,
        &w.tri,
        &w.water,
        &w.k,
        w.gw,
        w.gh,
        w.sea,
        true,
        None,
        |y| lats[y],
    );
    let expected_ids: Vec<i32> = v["region_id"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_i64().unwrap() as i32)
        .collect();
    assert_eq!(eco.region_id, expected_ids);
    assert_eq!(eco.marker_min, v["marker_min"].as_u64().unwrap() as usize);

    let recs = v["regions"].as_array().unwrap();
    assert_eq!(eco.regions.len(), recs.len());
    assert!(eco.regions.len() >= 10, "the captured world must produce a real set of regions");
    for (i, r) in eco.regions.iter().enumerate() {
        let x = &recs[i];
        assert_eq!(r.id, x["id"].as_u64().unwrap() as usize, "region {i} id");
        assert_eq!(r.biome, x["biome"].as_u64().unwrap() as u8, "region {i} biome");
        assert_eq!(r.cells, x["cells"].as_u64().unwrap() as usize, "region {i} cells");
        // Every aggregate is an order-dependent running f64 sum over f32
        // reads, so these are exact, not tolerant: a different traversal
        // order would show up here.
        assert_eq!(r.nppn, num(&x["nppn"]), "region {i} nppn");
        assert_eq!(r.tri, num(&x["tri"]), "region {i} tri");
        assert_eq!(r.water, num(&x["water"]), "region {i} water");
        assert_eq!(r.k, num(&x["K"]), "region {i} K");
        assert_eq!(r.lat_abs, num(&x["latAbs"]), "region {i} latAbs");
        assert_eq!(r.ridge_frac, num(&x["ridgeFrac"]), "region {i} ridgeFrac");
        assert_eq!(r.valley_frac, num(&x["valleyFrac"]), "region {i} valleyFrac");
        assert_eq!(r.coastal, x["coastal"].as_bool().unwrap(), "region {i} coastal");
        assert_eq!(r.cx, x["cx"].as_u64().unwrap() as usize, "region {i} cx");
        assert_eq!(r.cy, x["cy"].as_u64().unwrap() as usize, "region {i} cy");
    }
    assert_eq!(
        v["min_area"].as_u64().unwrap() as usize,
        12,
        "the captured world uses the max(12, ...) floor"
    );
}

#[test]
fn current_wildlife_matches_the_reference_roster_for_roster() {
    let v = fixture();
    let w = world(&v);
    let lats = lat_ladder(&v);
    let eco = current_wildlife(
        &w.cart_biome,
        &w.field,
        &w.npp,
        &w.tri,
        &w.water,
        &w.k,
        w.gw,
        w.gh,
        w.sea,
        true,
        w.cell_km,
        |y| lats[y],
    );
    let recs = v["regions"].as_array().unwrap();
    assert_eq!(eco.regions.len(), recs.len());
    let mut total_species = 0;
    for (i, r) in eco.regions.iter().enumerate() {
        let x = &recs[i];
        assert_eq!(r.richness, x["richness"].as_u64().unwrap() as usize, "region {i} richness");
        assert_eq!(r.summary, x["summary"].as_str().unwrap(), "region {i} summary");
        assert_eq!(r.area_km2, num(&x["areaKm2"]), "region {i} areaKm2");
        let col = x["col"].as_array().unwrap();
        assert_eq!(
            r.col,
            (
                col[0].as_u64().unwrap() as u8,
                col[1].as_u64().unwrap() as u8,
                col[2].as_u64().unwrap() as u8
            ),
            "region {i} colour"
        );
        let gl = x["guilds"].as_array().unwrap();
        assert_eq!(r.guilds.len(), gl.len(), "region {i} guild count");
        for (j, g) in r.guilds.iter().enumerate() {
            let e = &gl[j];
            assert_eq!(g.guild, e["guild"].as_str().unwrap(), "region {i} guild {j}");
            assert_eq!(g.biomass_rel, num(&e["biomassRel"]), "region {i} guild {j} biomassRel");
            let sp = e["species"].as_array().unwrap();
            assert_eq!(g.species.len(), sp.len(), "region {i} guild {j} species count");
            for (k, s) in g.species.iter().enumerate() {
                assert_eq!(s.name, sp[k]["name"].as_str().unwrap());
                assert_eq!(s.mass_kg, num(&sp[k]["massKg"]));
                assert_eq!(
                    s.population_est,
                    num(&sp[k]["populationEst"]),
                    "region {i} guild {j} species {k} pop"
                );
                total_species += 1;
            }
        }
        // ...and `wild_region_color` on its own agrees with what
        // `current_wildlife` stored.
        assert_eq!(wild_region_color(r), r.col);
    }
    assert!(
        total_species > 40,
        "only {total_species} species assigned -- the fixture is too thin to be measuring anything"
    );
}

/// Mutation checks on `regionRichness`' four constants: each has to move the
/// answer on a real captured record, or a wrong literal would sail through
/// the golden above.
#[test]
fn every_richness_constant_is_load_bearing() {
    let v = fixture();
    let x = &v["regions"].as_array().unwrap()[0];
    let rec = Ecoregion {
        biome: x["biome"].as_u64().unwrap() as u8,
        cells: x["cells"].as_u64().unwrap() as usize,
        nppn: num(&x["nppn"]),
        tri: num(&x["tri"]),
        lat_abs: num(&x["latAbs"]),
        ridge_frac: num(&x["ridgeFrac"]),
        ..Ecoregion::default()
    };
    let cell_km = num(&v["cell_km"]);
    let base_o = RichnessOpts {
        cell_km2: cell_km * cell_km,
        ..RichnessOpts::default()
    };
    let base = region_richness(&rec, &base_o);
    assert!(base > 0.0, "the captured region must have real richness");
    for m in [
        RichnessOpts { c: 2.4, ..base_o },
        RichnessOpts { k_h: 0.0, ..base_o },
        RichnessOpts { tri_ref: 0.2, ..base_o },
        RichnessOpts { cell_km2: 1.0, ..base_o },
    ] {
        assert_ne!(region_richness(&rec, &m), base);
    }
    // The species-area exponent switches on ridgeFrac > 0.25 -- a real
    // branch, so it must actually change the answer.
    let flat = Ecoregion {
        ridge_frac: 0.0,
        ..rec.clone()
    };
    let rugged = Ecoregion {
        ridge_frac: 0.9,
        ..rec.clone()
    };
    assert_ne!(region_richness(&flat, &base_o), region_richness(&rugged, &base_o));
    // Latitude enrichment is clamped to [1, 1.6]: the equator gets 1.0 and
    // the pole is capped, not infinite.
    let equator = Ecoregion {
        lat_abs: 0.0,
        ..rec.clone()
    };
    let pole = Ecoregion {
        lat_abs: 89.9,
        ..rec.clone()
    };
    assert!(region_richness(&pole, &base_o) / region_richness(&equator, &base_o) <= 1.6 + 1e-9);
}

/// The X-wrap flag has to reach both the flood fill and the circular mean —
/// the captured world wraps, so running it as a region must move the answer.
#[test]
fn the_wrap_flag_reaches_the_segmentation() {
    let v = fixture();
    let w = world(&v);
    let lats = lat_ladder(&v);
    let seg = |wrap: bool| {
        build_ecoregions(
            &w.cart_biome,
            &w.field,
            &w.npp,
            &w.tri,
            &w.water,
            &w.k,
            w.gw,
            w.gh,
            w.sea,
            wrap,
            None,
            |y| lats[y],
        )
    };
    let wrapped = seg(true);
    let clamped = seg(false);
    assert_ne!(
        wrapped.region_id, clamped.region_id,
        "the seam must merge components only when wrapping"
    );
}
