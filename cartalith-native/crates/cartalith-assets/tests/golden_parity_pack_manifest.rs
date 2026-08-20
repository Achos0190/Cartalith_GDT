//! Golden-parity tests for `cartalith-assets`' manifest parser against the
//! **real** reference implementation — `parsePackCsv` (reference
//! `Cartalith Gen1 v2.10.html` line 12093), `parsePackManifest` (line 12113)
//! and `packSummary` (line 12200).
//!
//! Generated from a Node `vm` extraction run (harness transient, not checked
//! in — same technique as `cartalith-civ`'s golden tests) that loads those
//! three functions plus their six `PACK_*_SLOTS` vocabularies straight out of
//! the frozen HTML by line range and calls them on the fixtures below. The
//! expected values here are that run's output verbatim, not a hand-derivation
//! of what the code "should" do.
//!
//! Why these functions are worth golden-testing rather than unit-testing
//! alone: the reference's behaviour on a *malformed* pack is full of details
//! that a rewrite gets plausibly wrong — which slots survive, which are
//! dropped whole vs. per-variant, the exact wording and **order** of the
//! warnings (which follows JavaScript object key-insertion order in three
//! places), a bare string standing in for a one-element variant list, an empty
//! path counting as a missing file, CSV variant ordering being a *stable* sort
//! with unnumbered rows pushed to the end, and JSON winning over CSV when both
//! are present. Every one of those is exercised below.
//!
//! All output here is text and structure, so an exact match is required —
//! there is no float tolerance to argue about.

use cartalith_assets::{
    Family, PACK_BIOME_SLOTS, PACK_ICON_SLOTS, PACK_POI_SLOTS, PACK_SETTLEMENT_SLOTS,
    PACK_TERRAIN_SLOTS, PACK_TEX_SLOTS, PACK_TRAIT_SLOTS, PackError, PackManifest, RawManifest,
    pack_summary, parse_pack_csv, parse_pack_entries, parse_pack_manifest,
};
use std::collections::{BTreeMap, BTreeSet};

fn files(list: &[&str]) -> BTreeSet<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn paths(m: &PackManifest, fam: Family, slot: &str) -> Vec<String> {
    m.slot_paths(fam, slot)
        .map(<[String]>::to_vec)
        .unwrap_or_default()
}

// ============================================================================
// Case A — a rich schema-2 manifest carrying every family and every failure
// mode at once: a missing texture file, an unknown texture slot, an unknown
// biome slot, one missing icon variant, an icon slot whose every variant is
// missing, an unknown icon slot, a bare string instead of a variant list, an
// unknown settlement slot, and a missing custom-set variant.
// ============================================================================

/// The manifest exactly as the harness fed it to the reference. **Key order is
/// load-bearing** — the "unknown ... slot" warnings are emitted by iterating
/// these objects, and JavaScript iterates string keys in insertion order.
const MANIFEST_A: &str = r#"{
  "schema": 2,
  "name": "Golden Test Pack",
  "author": "harness",
  "license": "CC0",
  "textures": {
    "grass": "textures/grass.png",
    "rock": "textures/rock.png",
    "gravel": "textures/gravel.png",
    "snow": "textures/missing_snow.png"
  },
  "biomes": { "jungle": "biomes/jungle.png", "swamp": "biomes/swamp.png" },
  "terrains": { "paved": "terrains/paved.png" },
  "icons": {
    "mountain": ["icons/mountain_01.png", "icons/mountain_02.png", "icons/mountain_gone.png"],
    "hill": "icons/hill_01.png",
    "tree_conifer": ["icons/absent_a.png"],
    "obelisk": ["icons/obelisk.png"]
  },
  "structures": {
    "settlement": {
      "hamlet": ["structures/settlement/hamlet_01.png"],
      "metropolis": ["structures/settlement/metropolis.png"]
    },
    "poi": { "ruin": "structures/poi/ruin_01.png" },
    "trait": { "port": ["structures/trait/port_01.png"] }
  },
  "custom": {
    "Naval": {
      "lighthouse": ["custom/naval/lighthouse_01.png", "custom/naval/lighthouse_02.png"],
      "anchor": ["custom/naval/anchor_missing.png"]
    },
    "Mining": { "pickaxe": "custom/mining/pickaxe_01.png" }
  }
}"#;

const FILES_A: [&str; 17] = [
    "textures/grass.png",
    "textures/rock.png",
    "textures/gravel.png",
    "biomes/jungle.png",
    "biomes/swamp.png",
    "terrains/paved.png",
    "icons/mountain_01.png",
    "icons/mountain_02.png",
    "icons/hill_01.png",
    "icons/obelisk.png",
    "structures/settlement/hamlet_01.png",
    "structures/settlement/metropolis.png",
    "structures/poi/ruin_01.png",
    "structures/trait/port_01.png",
    "custom/naval/lighthouse_01.png",
    "custom/naval/lighthouse_02.png",
    "custom/mining/pickaxe_01.png",
];

fn case_a() -> PackManifest {
    let raw: RawManifest = serde_json::from_str(MANIFEST_A).unwrap();
    parse_pack_manifest(&raw, &files(&FILES_A))
}

#[test]
fn case_a_warnings_match_the_reference_exactly_including_order() {
    // Reference output, verbatim. The ordering encodes the reference's own
    // traversal: textures (missing, then unknown) -> biomes -> terrains ->
    // icons (missing per variant, then unknown) -> structures in
    // settlement/poi/trait order -> custom sets in document order -> the
    // trailing "not yet used by the live map" summary.
    let expected = [
        "texture snow: file missing (textures/missing_snow.png)",
        "unknown texture slot: gravel",
        "unknown biomes slot: swamp",
        "icon mountain: file missing (icons/mountain_gone.png)",
        "icon tree_conifer: file missing (icons/absent_a.png)",
        "unknown icon slot: obelisk",
        "unknown settlement slot: metropolis",
        "custom Naval/anchor: file missing (custom/naval/anchor_missing.png)",
        "3 pack section(s) not yet used by the live map (trait, biomes, terrains)",
    ];
    assert_eq!(case_a().warnings, expected);
}

#[test]
fn case_a_resolved_art_matches_the_reference() {
    let m = case_a();

    assert_eq!(m.name, "Golden Test Pack");
    assert_eq!(m.author, "harness");
    assert_eq!(m.license, "CC0");

    // `gravel` is not a slot and `snow`'s file is absent, so two of four
    // declared textures survive -- in PACK_TEX_SLOTS order, not manifest order.
    assert_eq!(m.textures.keys().collect::<Vec<_>>(), ["grass", "rock"]);
    assert_eq!(m.textures.get("grass").unwrap(), "textures/grass.png");

    // `swamp` is a *terrain* slot, so it is unknown under `biomes` -- and is
    // dropped even though its file really is in the pack.
    assert_eq!(m.biomes.keys().collect::<Vec<_>>(), ["jungle"]);
    assert_eq!(m.terrains.keys().collect::<Vec<_>>(), ["paved"]);

    // One variant of `mountain` was missing: the slot survives with the other
    // two. Every variant of `tree_conifer` was missing: the slot is dropped
    // entirely, not left as an empty list.
    assert_eq!(m.icons.keys().collect::<Vec<_>>(), ["mountain", "hill"]);
    assert_eq!(
        paths(&m, Family::Icons, "mountain"),
        ["icons/mountain_01.png", "icons/mountain_02.png"]
    );
    // A bare string is a one-element variant list.
    assert_eq!(paths(&m, Family::Icons, "hill"), ["icons/hill_01.png"]);
    assert!(m.slot_paths(Family::Icons, "tree_conifer").is_none());

    assert_eq!(
        paths(&m, Family::Settlement, "hamlet"),
        ["structures/settlement/hamlet_01.png"]
    );
    assert!(m.structures.settlement.get("metropolis").is_none());
    assert_eq!(
        paths(&m, Family::Poi, "ruin"),
        ["structures/poi/ruin_01.png"]
    );
    assert_eq!(
        paths(&m, Family::Trait, "port"),
        ["structures/trait/port_01.png"]
    );
    assert_eq!(m.structures.len(), 3);

    // Custom sets keep their document order and their raw (un-slugified) set
    // names -- `assetPack.custom` is indexed by the name as written.
    assert_eq!(m.custom.keys().collect::<Vec<_>>(), ["Naval", "Mining"]);
    assert_eq!(
        m.custom_paths("Naval", "lighthouse").unwrap(),
        [
            "custom/naval/lighthouse_01.png",
            "custom/naval/lighthouse_02.png"
        ]
    );
    // `anchor`'s only variant was missing, so the slot is gone but the set
    // survives on the strength of its other slot.
    assert!(m.custom_paths("Naval", "anchor").is_none());
    assert_eq!(
        m.custom_paths("Mining", "pickaxe").unwrap(),
        ["custom/mining/pickaxe_01.png"]
    );
}

#[test]
fn case_a_summary_matches_the_reference() {
    assert_eq!(
        pack_summary(&case_a()),
        "Golden Test Pack · CC0 — 2 textures · 2 biome/terrain ground · mountain×2 hill×1 · 3 structure sprites · 2 custom icons"
    );
}

// ============================================================================
// Case B — a CSV-only pack. Exercises the header row, a blank line, CRLF line
// endings, an unknown slot (dropped silently, unlike the JSON path), a row
// with no commas, an all-empty row, and -- the interesting part -- variant
// ordering: two numbered rows written out of order plus two unnumbered rows,
// one of which points at a file that is not in the pack.
// ============================================================================

const CSV_B: &str = "type,slot,file,variant\r\n\
\r\n\
texture,grass,textures/grass.png,\r\n\
texture,unknown_slot,textures/nope.png,\r\n\
icon,mountain,icons/m_b.png,2\r\n\
icon,mountain,icons/m_a.png,1\r\n\
icon,mountain,icons/m_z.png,\r\n\
icon,mountain,icons/m_y.png,\r\n\
icon,shrub,icons/shrub_01.png,1\r\n\
icon,not_a_slot,icons/x.png,1\r\n\
biome,tundra,biomes/tundra.png,\r\n\
terrain,snow,terrains/snow.png,\r\n\
garbage row with no commas\r\n\
,,,";

const FILES_B: [&str; 7] = [
    "textures/grass.png",
    "icons/m_a.png",
    "icons/m_b.png",
    "icons/m_y.png",
    "icons/shrub_01.png",
    "biomes/tundra.png",
    "terrains/snow.png",
];

#[test]
fn case_b_csv_variant_ordering_matches_the_reference() {
    // Straight `parsePackCsv` output, before any file-existence validation:
    // numbered rows sort first by their number, unnumbered rows fall to the
    // end at 1e9 -- and keep the order they were written in, because
    // `Array.prototype.sort` is stable. `m_z` before `m_y` is that stability
    // showing, and is the detail an unstable sort would silently get wrong.
    let raw = parse_pack_csv(CSV_B);
    let mountain = raw.icons.get("mountain").unwrap().as_ref().unwrap();
    assert_eq!(
        mountain.as_slice(),
        [
            "icons/m_a.png",
            "icons/m_b.png",
            "icons/m_z.png",
            "icons/m_y.png"
        ]
    );
    assert_eq!(raw.icons.keys().collect::<Vec<_>>(), ["mountain", "shrub"]);
    assert_eq!(raw.textures.keys().collect::<Vec<_>>(), ["grass"]);
    assert_eq!(raw.biomes.keys().collect::<Vec<_>>(), ["tundra"]);
    assert_eq!(raw.terrains.keys().collect::<Vec<_>>(), ["snow"]);
    // No name/author/license -- the CSV form cannot carry them.
    assert!(raw.name.is_none() && raw.author.is_none() && raw.license.is_none());
}

#[test]
fn case_b_validated_manifest_and_summary_match_the_reference() {
    let mut entries: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    entries.insert("pack.csv".into(), CSV_B.as_bytes().to_vec());
    for f in FILES_B {
        entries.insert(f.into(), b"x".to_vec());
    }
    let m = parse_pack_entries(&entries).unwrap();

    // Defaults for the three fields a CSV cannot express.
    assert_eq!(m.name, "Asset pack");
    assert_eq!(m.author, "");
    assert_eq!(m.license, "");

    assert_eq!(
        paths(&m, Family::Icons, "mountain"),
        ["icons/m_a.png", "icons/m_b.png", "icons/m_y.png"]
    );
    assert_eq!(paths(&m, Family::Icons, "shrub"), ["icons/shrub_01.png"]);
    assert_eq!(m.textures.len(), 1);

    assert_eq!(
        m.warnings,
        [
            "icon mountain: file missing (icons/m_z.png)",
            "2 pack section(s) not yet used by the live map (biomes, terrains)",
        ]
    );
    assert_eq!(
        pack_summary(&m),
        "Asset pack · license? — 1 textures · 2 biome/terrain ground · mountain×3 shrub×1"
    );
}

// ============================================================================
// Case C — a minimal, entirely valid schema-1 pack. The "no warnings at all"
// baseline, and the check that a schema-1 manifest needs nothing schema 2 adds.
// ============================================================================

#[test]
fn case_c_clean_schema_1_pack_matches_the_reference() {
    let raw: RawManifest = serde_json::from_str(
        r#"{"schema":1,"textures":{"grass":"textures/grass.png"},"icons":{"boulder":["icons/b1.png","icons/b2.png"]}}"#,
    )
    .unwrap();
    let m = parse_pack_manifest(
        &raw,
        &files(&["textures/grass.png", "icons/b1.png", "icons/b2.png"]),
    );

    assert!(m.warnings.is_empty());
    assert_eq!(m.name, "Asset pack");
    assert_eq!(
        paths(&m, Family::Icons, "boulder"),
        ["icons/b1.png", "icons/b2.png"]
    );
    assert!(m.biomes.is_empty() && m.terrains.is_empty() && m.custom.is_empty());
    assert!(m.structures.is_empty());
    assert_eq!(
        pack_summary(&m),
        "Asset pack · license? — 1 textures · boulder×2"
    );
}

// ============================================================================
// Case D — JSON wins over CSV when both are present, an empty name falls back
// to the default, and an empty-string path counts as a missing file (the
// reference's `has` is a truthiness test, not an `undefined` test).
// ============================================================================

#[test]
fn case_d_json_wins_and_an_empty_path_is_a_missing_file() {
    let mut entries: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    entries.insert(
        "pack.json".into(),
        br#"{"name":"","author":"a","license":"","textures":{"grass":"","rock":"textures/rock.png"},"icons":{}}"#
            .to_vec(),
    );
    // A CSV that would have loaded a snow texture, if JSON did not win.
    entries.insert(
        "pack.csv".into(),
        b"type,slot,file,variant\ntexture,snow,textures/snow.png,".to_vec(),
    );
    entries.insert("textures/rock.png".into(), b"x".to_vec());
    entries.insert("textures/snow.png".into(), b"x".to_vec());

    let m = parse_pack_entries(&entries).unwrap();
    assert_eq!(m.name, "Asset pack");
    assert_eq!(m.author, "a");
    assert_eq!(m.license, "");
    assert_eq!(m.textures.keys().collect::<Vec<_>>(), ["rock"]);
    assert_eq!(m.warnings, ["texture grass: file missing ()"]);
    assert_eq!(pack_summary(&m), "Asset pack · license? — 1 textures");
}

// ============================================================================
// Case E — no manifest at all. The reference throws; this port returns an
// error carrying that exact message.
// ============================================================================

#[test]
fn case_e_missing_manifest_message_matches_the_reference() {
    let mut entries: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    entries.insert("foo.png".into(), b"x".to_vec());
    match parse_pack_entries(&entries) {
        Err(e @ PackError::NoManifest) => {
            assert_eq!(e.to_string(), "pack has no pack.json or pack.csv");
        }
        other => panic!("expected NoManifest, got {other:?}"),
    }
}

// ============================================================================
// The frozen vocabularies themselves, dumped from the same extraction run.
// A silent edit to any of these re-points every pack ever authored, so they
// are pinned here against the reference's own arrays rather than only against
// the design doc's prose.
// ============================================================================

#[test]
fn frozen_slot_vocabularies_match_the_reference() {
    assert_eq!(
        PACK_TEX_SLOTS,
        [
            "grass",
            "rock",
            "sand",
            "snow",
            "wetland",
            "canopy",
            "parchment"
        ]
    );
    assert_eq!(
        PACK_ICON_SLOTS,
        [
            "mountain",
            "hill",
            "tree_conifer",
            "tree_broadleaf",
            "tree_rainforest",
            "tree_savanna",
            "tree_wetland",
            "shrub",
            "cactus",
            "boulder"
        ]
    );
    assert_eq!(
        PACK_BIOME_SLOTS,
        [
            "coastal",
            "temperate_forest",
            "mediterranean",
            "wetlands",
            "steppe",
            "jungle",
            "boreal",
            "mountain",
            "cold_desert",
            "hot_desert",
            "tundra",
            "ruined",
            "hills",
            "lake_river",
            "ocean"
        ]
    );
    assert_eq!(
        PACK_TERRAIN_SLOTS,
        [
            "paved",
            "dirt",
            "hardpack",
            "plains",
            "forest_path",
            "hills",
            "rocky",
            "mtn_pass",
            "mtn_trail",
            "swamp",
            "deep_sand",
            "snow",
            "ruins"
        ]
    );
    assert_eq!(
        PACK_SETTLEMENT_SLOTS,
        [
            "hamlet",
            "village",
            "town",
            "city",
            "capital",
            "monastery",
            "fortress",
            "university",
            "industrial"
        ]
    );
    assert_eq!(
        PACK_POI_SLOTS,
        [
            "ruin",
            "landmark",
            "mountain_peak",
            "named_forest",
            "battlefield",
            "shrine",
            "cave",
            "other"
        ]
    );
    assert_eq!(
        PACK_TRAIT_SLOTS,
        [
            "fortified",
            "mining",
            "port",
            "administrative",
            "trade_hub",
            "military",
            "religious"
        ]
    );
}
