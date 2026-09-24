//! Owner Ruling AU: a legacy flat `.zip` imports its settlements, labels and
//! icons, with a report of what did not map (`cartalith_io::legacy`).
//!
//! Two real HTML-app exports:
//!
//! - `real_export_seed24601.zip` — the repository's first, exported straight
//!   after `generate()`. Its `places`, `labels` and `mapIcons` are empty and
//!   its `civ` is `null`, so it is the "no records" case: it must read exactly
//!   as before, with nothing imported and nothing reported.
//! - `legacy_records_seed24601.zip` — written by
//!   `tools/legacy_records_capture.js` from the unmodified v2.11 reference
//!   under Node: `generate()`, Auto-populate, the Settlement and POI tools, a
//!   designated place, two labels and three icons, then `_civSyncToState()`
//!   and `exportZip`'s own first entries through its own `zipStore`.
//!
//! Every expectation is read out of the archive's `params.json` **here**, with
//! `zip` and `serde_json` directly — not from the reader under test — or is a
//! literal the capture script placed.

use std::io::Read;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/");

fn open(name: &str) -> cartalith_io::ProjectData {
    let f = std::fs::File::open(format!("{FIXTURES}{name}")).expect("fixture opens");
    cartalith_io::read_project(f).expect("reads")
}

/// `params.json`'s `state`, straight out of the archive.
fn raw_state(name: &str) -> serde_json::Value {
    let f = std::fs::File::open(format!("{FIXTURES}{name}")).expect("fixture opens");
    let mut z = zip::ZipArchive::new(f).expect("a zip");
    let mut text = String::new();
    z.by_name("params.json").expect("params.json").read_to_string(&mut text).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    v["state"].clone()
}

#[test]
fn an_export_with_no_records_reads_exactly_as_before() {
    let raw = raw_state("real_export_seed24601.zip");
    assert_eq!(raw["places"], serde_json::json!([]), "fixture shape: this is the empty case");
    assert_eq!(raw["labels"], serde_json::json!([]));
    assert_eq!(raw["mapIcons"], serde_json::json!([]));
    assert!(raw["civ"].is_null());

    let data = open("real_export_seed24601.zip");
    assert_eq!(data.layout, cartalith_io::Layout::Flat);
    let legacy = data.legacy.as_ref().expect("a flat archive always has the legacy block");
    assert!(legacy.is_empty(), "{legacy:?}");
    assert!(legacy.factions.is_empty());
    assert!(data.warnings.is_empty(), "{:?}", data.warnings);
    assert!(data.documents.is_empty());
}

#[test]
fn a_tree_archive_has_no_legacy_block() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../cartalith-godot/tests/fixtures/project_pre_substrate_2026-09-24.zip");
    let data = cartalith_io::read_project(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(data.layout, cartalith_io::Layout::Tree);
    assert!(data.legacy.is_none());
}

#[test]
fn the_records_export_imports_every_settlement_label_and_icon() {
    const NAME: &str = "legacy_records_seed24601.zip";
    let raw = raw_state(NAME);
    let data = open(NAME);
    let legacy = data.legacy.as_ref().expect("flat");
    let (gw, gh) = (data.save.params.gw, data.save.params.gh);

    // --- settlements: every place whose kind is one of the six tiers ---
    let places = raw["places"].as_array().unwrap();
    let tiers = ["metropolis", "capital", "city", "town", "village", "hamlet"];
    let settle: Vec<&serde_json::Value> =
        places.iter().filter(|p| p["kind"].as_str().is_some_and(|k| tiers.contains(&k))).collect();
    assert!(settle.len() >= 4, "fixture shape: Auto-populate plus the Settlement tool, got {}", settle.len());
    assert_eq!(places.len() - settle.len(), 2, "fixture shape: one POI and one designated place");
    assert_eq!(legacy.settlements.len(), settle.len());
    for (s, p) in legacy.settlements.iter().zip(&settle) {
        assert_eq!(s.name, p["name"].as_str().unwrap());
        assert_eq!(s.x as f64, p["x"].as_f64().unwrap());
        assert_eq!(s.y as f64, p["y"].as_f64().unwrap());
        assert_eq!(s.kind, p["kind"].as_str().unwrap());
        assert_eq!(s.population as i64, p["pop"].as_i64().unwrap());
        assert_eq!(s.faction as i64, p["faction"].as_i64().unwrap());
        let port = p["traits"].as_array().unwrap().iter().any(|t| t == "port");
        assert_eq!(s.coastal, port);
        assert!(s.x < gw && s.y < gh);
    }
    // The Settlement tool's own record (reference `_civDropPlace`): no name,
    // `town`, population 1000 -- literals of that function.
    let dropped = legacy.settlements.last().unwrap();
    assert_eq!((dropped.name.as_str(), dropped.kind, dropped.population), ("", "town", 1000));
    // Ids are unique.
    let mut ids: Vec<u64> = legacy.settlements.iter().map(|s| s.tid).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), legacy.settlements.len());

    // --- the faction roster and territory ---
    let names = raw["civ"]["factionNames"].as_array().unwrap();
    assert_eq!(legacy.factions.len(), names.len());
    for (f, n) in legacy.factions.iter().zip(names) {
        assert_eq!(f.name, n.as_str().unwrap());
    }
    let pairs = raw["civ"]["territory"].as_array().unwrap();
    let territory = legacy.territory.as_ref().expect("the export claims territory");
    assert_eq!(territory.len(), gw * gh);
    assert_eq!(territory.iter().filter(|&&f| f != 0).count(), pairs.len() / 2);
    let (i0, f0) = (pairs[0].as_u64().unwrap() as usize, pairs[1].as_i64().unwrap() as i32);
    assert_eq!(territory[i0], f0);

    // --- labels: the capture script's two, literally ---
    assert_eq!(legacy.labels.len(), 2);
    let l0 = &legacy.labels[0];
    assert_eq!((l0.x, l0.y, l0.name.as_str(), l0.size), (12.4, 30.8, "Vharen Reach", 16.0));
    assert!(!l0.fixed_size);
    let l1 = &legacy.labels[1];
    assert_eq!((l1.name.as_str(), l1.angle, l1.arc, l1.size), ("The Sundering Sea", -12.0, 0.3, 22.0));
    assert!(l1.fixed_size);
    // The label editor fills a concrete font and colour when a label is
    // selected (reference 17297-17298), and they are carried as the archive
    // holds them, not replaced by this build's defaults.
    assert_eq!(l0.font.as_deref(), raw["labels"][0]["font"].as_str());
    assert_eq!(l0.color.as_deref(), raw["labels"][0]["color"].as_str());

    // --- icons: the capture script's three, literally ---
    assert_eq!(legacy.icons.len(), 3);
    let i = &legacy.icons;
    assert_eq!((i[0].x, i[0].y, i[0].family, i[0].slot.as_str(), i[0].scale), (40.5, 20.25, "poi", "ruin", 1.0));
    assert_eq!((i[1].family, i[1].set.as_deref(), i[1].slot.as_str()), ("custom", Some("my_set"), "tower"));
    assert_eq!((i[2].family, i[2].slot.as_str(), i[2].scale), ("feature", "mountain", 0.8125));
    assert_eq!(i[0].set, None, "only `custom` carries a set");

    // --- the report: what did not map, and it is the reader's warnings ---
    assert_eq!(data.warnings, legacy.unmapped);
    let r = legacy.unmapped.join("\n");
    for needle in [
        "1 point of kind \"shrine\" not imported",
        "1 place with no kind",
        "faction colours are not stored in the flat layout",
    ] {
        assert!(r.contains(needle), "missing {needle:?} in the report:\n{r}");
    }
    if raw["civ"]["ways"].as_array().is_some_and(|w| !w.is_empty()) {
        assert!(r.contains("state.civ.ways:"), "{r}");
    }
}
