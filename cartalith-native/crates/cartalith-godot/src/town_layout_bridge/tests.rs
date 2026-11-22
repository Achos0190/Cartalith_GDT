//! Tests of the frozen-town slot (TL-2), the cheap tier (TL-5) and the chip (TL-7), over synthetic towns so they run
//! in the default suite; the 64-town census over real worlds is `census.rs` (`#[ignore]`d, a measurement).

use super::*;
use cartalith_civ::urban_adapter::{PlaceOverrides, UrbanWorld, run_layout, um_place_context_with};
use cartalith_civ::{NamedSettlement, SettlementKind, SettlementPlacement};

mod census;

/// Five synthetic towns (land rising north, a sea from row 48, a river column at `x == 20`), the app's switches on as
/// literals. Judgement: the shape `cartalith_civ::town_freeze`'s own fixtures use.
fn towns() -> Vec<(u64, UrbanLayout)> {
    let (gw, gh) = (64usize, 64usize);
    let mut field = vec![0.0f32; gw * gh];
    let mut flow = vec![0.0f32; gw * gh];
    let mut order = vec![0i16; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            field[y * gw + x] = if y >= 48 { 0.30 } else { 0.75 - 0.10 * (y as f32 / 63.0) + 0.01 * ((x as f32) * 0.7).sin() };
            if x == 20 && y < 48 {
                flow[y * gw + x] = 5_000.0;
                order[y * gw + x] = 3;
            }
        }
    }
    let rivers: Vec<Vec<(f64, f64)>> = vec![(0..49).map(|y| (20.5, y as f64 + 0.5)).collect()];
    let wb = vec![0u8; gw * gh];
    let uw = UrbanWorld {
        field: &field,
        flow: &flow,
        water_bodies: &wb,
        order: Some(&order),
        river_polys: &rivers,
        gw,
        gh,
        sea_level: 0.42,
        map_width_km: 40.0,
        flow_thresh: 1_000.0,
        world_seed: 7,
        river_kind_from_map: true,
    };
    let o = PlaceOverrides { wall_hug: true, riverbank: true, street_village: true, market_scale: true, harbour_front: true, market_road_bank: true, water_widen: true, town_bridge_search: true, ..Default::default() };
    [(1u64, 50usize, 8usize, 200u32), (2, 45, 12, 4_000), (3, 40, 20, 12_000), (4, 34, 47, 3_000), (5, 21, 30, 5_000)]
        .into_iter()
        .map(|(tid, x, y, pop)| {
            let s = NamedSettlement {
                tid,
                placement: SettlementPlacement { x, y, suit: 0.5, faction: 1, capital: false, kind: SettlementKind::Town, coastal: false },
                name: format!("T{tid}"),
                pop,
            };
            (tid, run_layout(&um_place_context_with(&uw, &s, &[], &o), None).expect("lays out"))
        })
        .collect()
}

/// A table of the five towns frozen, each with one anchored POI (ids 1..=5).
fn table() -> TownTable {
    let mut t = TownTable::default();
    for (tid, l) in towns() {
        let geom = tf::encode(&l);
        let layout = Arc::new(tf::decode(&geom).expect("decodes"));
        let a = tf::choose_anchor(&layout, layout.market);
        let mut anchors = BTreeMap::new();
        anchors.insert(tid as u32, StoredAnchor { a, at: (1000 * tid as i64, 2000), detached: false });
        let recipe = json!({ "rules_version": tf::URBAN_RULES_VERSION, "switches": { "urban_wall_hug": true }, "site_hash": "00" });
        let row = make_row(tid, 412, 2207, recipe, &geom, tid == 2);
        t.rows.insert(tid, Arc::new(FrozenTown { id: tid, row, layout: Some(layout), problem: None, anchors }));
    }
    t
}

fn reread(text: &str, pois: &[u32], w: &mut Vec<String>) -> TownTable {
    let v: Value = serde_json::from_str(text).expect("json");
    restore(Some(text), Some(&v), pois, w)
}

const POIS: [u32; 5] = [1, 2, 3, 4, 5];

/// Protects TL-2's round-trip bar: five frozen towns saved, read back and saved again give byte-identical text, the
/// rows come back usable with their anchors, and the text is integers and strings only (sorted keys, compact).
#[test]
fn the_slot_round_trips_byte_for_byte() {
    let t = table();
    let a = save_text(&t, &POIS).expect("five frozen towns write the member");
    let mut w = Vec::new();
    let back = reread(&a, &POIS, &mut w);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(back.rows.len(), 5);
    assert!(back.rows.values().all(|r| r.layout.is_some() && r.anchors.len() == 1));
    assert_eq!(save_text(&back, &POIS).expect("writes"), a, "load then save must be byte-identical");
    // No float anywhere: every number in the text is an integer (a string may hold a dot, a number may not).
    fn no_float(v: &Value) -> bool {
        match v {
            Value::Number(n) => n.is_i64() || n.is_u64(),
            Value::Array(a) => a.iter().all(no_float),
            Value::Object(o) => o.values().all(no_float),
            _ => true,
        }
    }
    assert!(no_float(&serde_json::from_str(&a).expect("json")), "the slot must be integer-only");
    assert!(!a.contains('\n'), "compact text");
    assert_eq!(back, t, "the table read back equals the one written");
}

/// Protects scope §7's first MUST: a world with no frozen town writes no member.
#[test]
fn no_frozen_town_writes_no_member() {
    assert_eq!(save_text(&TownTable::default(), &POIS), None);
}

/// Protects the older-save bar: the two real prior-format archives checked in carry no slot, restore to an empty
/// table with no warning, and write no member back.
#[test]
fn older_saves_open_with_no_frozen_town_and_write_no_member() {
    let fixtures: [(&str, &[u8]); 2] = [
        ("project_pre_substrate_2026-09-24", include_bytes!("../../tests/fixtures/project_pre_substrate_2026-09-24.zip")),
        ("project_384cap_512x328_2026-09-28", include_bytes!("../../tests/fixtures/project_384cap_512x328_2026-09-28.zip")),
    ];
    for (name, bytes) in fixtures {
        let data = cartalith_io::project::read_project(std::io::Cursor::new(bytes)).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(data.text_of(SLOT).is_none(), "{name}: premise, written before the slot");
        let mut w = Vec::new();
        let t = restore(data.text_of(SLOT), data.document(SLOT), &[], &mut w);
        assert!(w.is_empty(), "{name}: {w:?}");
        assert!(t.rows.is_empty() && t.verbatim.is_none());
        assert_eq!(save_text(&t, &[]), None, "{name}: no member written back");
    }
}

/// Protects scope §7's damage rows, each with its kept-verbatim write-back: a newer `version` and a document that is
/// not a town-layout document are kept whole; an unknown `enc`, a hash mismatch, a missing member and an out-of-range
/// index each keep their row verbatim (unusable, warned) while the other rows stay usable; a dropped anchor (its POI
/// gone, or its `fid` out of range) is counted.
#[test]
fn every_damage_row_is_kept_verbatim_and_reported() {
    let good = save_text(&table(), &POIS).expect("writes");
    let v: Value = serde_json::from_str(&good).expect("json");

    // A newer document: kept whole, written back untouched.
    let mut newer = v.clone();
    newer["version"] = json!(DOC_VERSION + 1);
    let newer_text = tf::canonical(&newer);
    let mut w = Vec::new();
    let t = reread(&newer_text, &POIS, &mut w);
    assert!(t.rows.is_empty() && w.len() == 1, "{w:?}");
    assert_eq!(save_text(&t, &POIS).as_deref(), Some(newer_text.as_str()));
    // Not a town-layout document at all: the same.
    let odd = r#"{"something":"else"}"#;
    let mut w = Vec::new();
    let t = reread(odd, &POIS, &mut w);
    assert_eq!((w.len(), save_text(&t, &POIS).as_deref()), (1, Some(odd)));

    for (what, edit) in [
        ("unknown enc", Box::new(|r: &mut Value| r["enc"] = json!("q20d9")) as Box<dyn Fn(&mut Value)>),
        ("hash mismatch", Box::new(|r: &mut Value| r["layout_hash"] = json!("0000000000000000"))),
        ("missing member", Box::new(|r: &mut Value| {
            r["geom"].as_object_mut().expect("geom").remove("buildings");
            r["layout_hash"] = json!(tf::layout_hash(&r["geom"]));
        })),
        ("index out of range", Box::new(|r: &mut Value| {
            r["geom"]["edges"][0] = json!(9_999_999);
            r["layout_hash"] = json!(tf::layout_hash(&r["geom"]));
        })),
    ] {
        let mut d = v.clone();
        edit(&mut d["towns"][1]);
        let text = tf::canonical(&d);
        let mut w = Vec::new();
        let t = reread(&text, &POIS, &mut w);
        assert_eq!(w.len(), 1, "{what}: {w:?}");
        assert!(t.rows[&2].layout.is_none() && t.rows[&2].problem.is_some(), "{what}: the damaged row is unusable");
        assert_eq!(t.rows.values().filter(|r| r.layout.is_some()).count(), 4, "{what}: the other rows stay usable");
        assert_eq!(save_text(&t, &POIS).as_deref(), Some(text.as_str()), "{what}: written back exactly as read");
    }

    // A POI gone: its anchor is dropped and counted; the town is unaffected.
    let mut w = Vec::new();
    let t = reread(&good, &[1, 2, 3, 4], &mut w);
    assert_eq!(t.rows.values().filter(|r| r.layout.is_some()).count(), 5);
    assert!(t.rows[&5].anchors.is_empty() && w.len() == 1, "{w:?}");
    // An anchor whose fid is out of range: dropped and counted.
    let mut d = v.clone();
    d["towns"][0]["anchors"][0]["kind"] = json!("building");
    d["towns"][0]["anchors"][0]["fid"] = json!(9_999_999);
    let mut w = Vec::new();
    let t = reread(&tf::canonical(&d), &POIS, &mut w);
    assert!(t.rows[&1].anchors.is_empty() && w.len() == 1, "{w:?}");
}

/// Protects TL-5's cheap tier: each of four injected differences (the rules version, a switch, the world, a
/// place-editor field) is named, and equal recipes give no reason.
#[test]
fn the_cheap_tier_names_each_cause() {
    let base = json!({
        "rules_version": 1, "switches": { "urban_wall_hug": true, "urban_parallel_streets": false },
        "site_hash": "aaaaaaaaaaaaaaaa", "culture": "", "variant": 0, "seed": 9,
    });
    assert!(cheap_reasons(&base, &base).is_empty());
    let mut a = base.clone();
    a["rules_version"] = json!(3);
    assert_eq!(cheap_reasons(&base, &a), vec!["drawn by rules v1 (current v3)".to_string()]);
    let mut b = base.clone();
    b["switches"]["urban_parallel_streets"] = json!(true);
    assert_eq!(cheap_reasons(&base, &b), vec!["parallel streets is now on".to_string()]);
    let mut c = base.clone();
    c["site_hash"] = json!("bbbbbbbbbbbbbbbb");
    assert_eq!(cheap_reasons(&base, &c), vec!["the ground or roads under this town changed since it was drawn".to_string()]);
    let mut d = base.clone();
    d["culture"] = json!("riverine");
    assert_eq!(cheap_reasons(&base, &d), vec!["the town's settings changed (culture)".to_string()]);
    // A switch an older recipe never recorded reads as off: no difference while it stays off.
    let mut older = base.clone();
    older["switches"].as_object_mut().expect("switches").remove("urban_parallel_streets");
    assert!(cheap_reasons(&older, &base).is_empty());
}

/// Protects TL-4's "0 pins lost" in the default suite: every mark of every synthetic town re-laid under the
/// `market_town` rules gets exactly one outcome (the census asserts the same over 64 real towns), a re-attached one moves
/// to a finite point, and at least one of each outcome the rules change produces is counted.
#[test]
fn relay_keeps_every_mark() {
    let t = towns();
    let mut total = 0usize;
    let mut attached = 0usize;
    for (i, (_, l)) in t.iter().enumerate() {
        // Today's town: the same town decoded twice, then disturbed (a rules change is the census's job; here the new
        // town has no buildings at all, so every building pin must detach while plot and street pins re-attach).
        let old = tf::decode(&tf::encode(l)).expect("decodes");
        let mut new = tf::decode(&tf::encode(l)).expect("decodes");
        new.buildings.clear();
        let mut marks = Vec::new();
        for (k, n) in [(FeatKind::Building, old.buildings.len()), (FeatKind::Plot, old.parcels.len()), (FeatKind::Street, old.edges.len())] {
            for j in (0..n).step_by((n / 7).max(1)) {
                let a = tf::anchor_on(&old, k, Some(j));
                let here = tf::anchor_point(&old, &a).expect("in range");
                marks.push((MarkRef::Poi((i * 1000 + marks.len()) as u32), a, here, (0, 0)));
            }
        }
        let n = marks.len();
        let (out, rep) = plan_moves(&old, &new, marks, |_| ("point of interest", String::new()));
        assert_eq!(out.len(), n, "a mark was dropped");
        assert_eq!(rep.attached + rep.ground + rep.detached.len(), n);
        for (_, to, _, det, _) in &out {
            if let Some(p) = to {
                assert!(p.x.is_finite() && p.y.is_finite() && !det);
            }
        }
        total += n;
        attached += rep.attached;
        assert!(!rep.detached.is_empty(), "town {i}: with no buildings left, building pins must detach");
    }
    assert!(attached > 0 && attached < total, "plots and streets re-attach, buildings do not: {attached} of {total}");
}

/// Protects TL-7's bar: the chip shows at a 2x population gap either way and not at 1.9x.
#[test]
fn the_chip_shows_at_two_x_and_not_at_one_point_nine() {
    assert!(pop_chip(1000, 2000));
    assert!(pop_chip(2000, 1000));
    assert!(!pop_chip(1000, 1900));
    assert!(!pop_chip(1900, 1000));
    assert!(pop_chip(0, 5));
    assert!(!pop_chip(500, 500));
}
