//! Owner **Ruling AU** (`LARGE_ITEM_RULINGS.md`, 2026-09-24): a legacy flat
//! `.zip` opens with its settlements, labels and icons, not as terrain alone.
//!
//! `cartalith_io::legacy` reads the records out of the archive's `state`,
//! makes every substitution and writes the report of what did not map; this
//! module only translates those records into this crate's own types. It is
//! called from `WorldGen::load_save`, which both open paths go through
//! (`project_open` calls it first), so the flat branch needs no code in the
//! project restore.
//!
//! **The world stays `Loaded`.** A flat archive has no substrate (§8.3), so
//! everything built here sits over a terrain-only world: it is drawn and
//! edited, and every readout that needs flow, channels or the tectonic grids
//! refuses with `substrate::NEEDS_SUBSTRATE` exactly as it does for any other
//! `Loaded` world. Nothing here recomputes anything.

use crate::civ_roster_bridge::{FactionEntry, FactionRoster, PlaceExtras, PlaceExtrasTable};
use crate::{icon_bridge, label_bridge, CivData, CIV_FACTION_COUNT};
use cartalith_io::legacy::LegacyProject;

/// What a flat archive's records became.
pub(crate) struct Imported {
    /// `None` when the archive placed no settlement and claimed no cell —
    /// the same "no civilisation layer" a generated world with zero
    /// settlements has, not an empty one.
    pub civ: Option<CivData>,
    pub labels: Option<label_bridge::LabelBridge>,
    pub icons: Option<icon_bridge::IconEditor>,
}

/// The roster: the archive's own names and attributes, a column it did not
/// carry filled from this build's per-index default, and the colour always
/// from this build's palette, because the flat layout stores none (§15.3 —
/// `cartalith_io::legacy` has already reported that).
fn roster(legacy: &LegacyProject) -> FactionRoster {
    if legacy.factions.is_empty() {
        return FactionRoster::seeded(CIV_FACTION_COUNT as usize);
    }
    FactionRoster(
        legacy
            .factions
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let d = FactionEntry::default_for(i);
                FactionEntry {
                    name: f.name.clone(),
                    culture: f.culture.clone().unwrap_or(d.culture),
                    religion: f.religion.clone().unwrap_or(d.religion),
                    government: f.government.clone().unwrap_or(d.government),
                    ag_tech: f.ag_tech.clone().unwrap_or(d.ag_tech),
                    color: d.color,
                    color_override: None,
                    tariffs: Default::default(),
                }
            })
            .collect(),
    )
}

pub(crate) fn import(legacy: &LegacyProject, gw: usize, gh: usize) -> Imported {
    let n = gw * gh;
    let civ = (!legacy.settlements.is_empty() || legacy.territory.is_some()).then(|| {
        let mut extras = std::collections::HashMap::new();
        let mut village_tids = std::collections::HashSet::new();
        let settlements: Vec<cartalith_civ::NamedSettlement> = legacy
            .settlements
            .iter()
            .map(|s| {
                if !s.traits.is_empty() || s.history.is_some() || s.specialisation.is_some() || s.age.is_some() || s.walls.is_some() {
                    extras.insert(
                        s.tid,
                        PlaceExtras {
                            // The reference's own `'none'` is this type's empty.
                            specialisation: s.specialisation.clone().filter(|v| v != "none").unwrap_or_default(),
                            traits: s.traits.clone(),
                            history: s.history.clone().unwrap_or_default(),
                            age: s.age,
                            walls: s.walls,
                            ..PlaceExtras::default()
                        },
                    );
                }
                if s.village_seeded {
                    village_tids.insert(s.tid);
                }
                cartalith_civ::NamedSettlement {
                    tid: s.tid,
                    placement: cartalith_civ::SettlementPlacement {
                        x: s.x,
                        y: s.y,
                        suit: s.suitability,
                        faction: s.faction,
                        capital: s.capital,
                        // `cartalith_io::legacy` only ever hands over one of
                        // the six tier keys.
                        kind: crate::civ_tools_bridge::kind_from_str(s.kind).unwrap_or(cartalith_civ::SettlementKind::Town),
                        coastal: s.coastal,
                    },
                    name: s.name.clone(),
                    pop: s.population,
                }
            })
            .collect();
        let next_tid = settlements.iter().map(|s| s.tid + 1).max().unwrap_or(1);
        CivData {
            trade_balances: settlements
                .iter()
                .map(|_| cartalith_civ::TradeBalance { exports: Vec::new(), imports: Vec::new() })
                .collect(),
            settlements,
            ways: Vec::new(),
            road_edges: Vec::new(),
            sea_routes: Vec::new(),
            territory: legacy.territory.clone().filter(|t| t.len() == n).unwrap_or_else(|| vec![0; n]),
            // Provinces are not in the flat layout (§15.1): re-derived on
            // demand there, and here, rather than invented.
            provinces: vec![0; n],
            province_list: Vec::new(),
            continents: Vec::new(),
            explanations: Vec::new(),
            water_bodies: Vec::new(),
            next_tid,
            timeline: Vec::new(),
            year: 0,
            dens: Vec::new(),
            faction_roster: roster(legacy),
            place_extras: PlaceExtrasTable(extras),
            village_tids,
            belief: Vec::new(),
            belief_seed_key: Vec::new(),
            territory_year: None,
        }
    });

    let labels = (!legacy.labels.is_empty()).then(|| {
        let mut bridge = label_bridge::LabelBridge::new();
        bridge.labels = legacy
            .labels
            .iter()
            .map(|l| cartalith_civ::labels::MapLabel {
                x: l.x,
                y: l.y,
                name: l.name.clone(),
                angle: l.angle,
                arc: l.arc,
                size: l.size,
                font: l.font.clone(),
                color: l.color.clone(),
                size_mode: if l.fixed_size {
                    cartalith_civ::labels::LabelSizeMode::Fixed
                } else {
                    cartalith_civ::labels::LabelSizeMode::Zoom
                },
                // The reference has one label kind, and it is a region name:
                // its tool is "click-to-add-region-name" and it prompts
                // "Region name:" (v2.11 lines 9804-9809).
                class: cartalith_civ::labels::LabelClass::Region,
                weight: 0.0,
            })
            .collect();
        bridge
    });

    let icons = (!legacy.icons.is_empty()).then(|| {
        let mut editor = icon_bridge::IconEditor::new();
        editor.icons = legacy
            .icons
            .iter()
            .filter_map(|i| {
                Some(cartalith_assets::manual::ManualIcon {
                    x: i.x,
                    y: i.y,
                    // `cartalith_io::legacy` passes only §11.2's four names,
                    // every one of which this build places.
                    family: cartalith_assets::manual::ManualIconFamily::from_key(i.family)?,
                    slot: i.slot.clone(),
                    set: i.set.clone(),
                    scale: i.scale,
                    // Every icon the reference stored was placed by hand: it
                    // had no generated-icon pass writing into `mapIcons`.
                    origin: cartalith_assets::manual::IconOrigin::Manual,
                })
            })
            .collect();
        editor
    });

    Imported { civ, labels, icons }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fallback roster when an archive carries no `factionNames` is the
    /// reference's own seven rows, which `cartalith_io::legacy` range-checks
    /// settlement factions against. The two numbers are kept apart on
    /// purpose; this is the test that they agree.
    #[test]
    fn the_default_roster_is_the_length_the_reader_clamps_against() {
        assert_eq!(FactionRoster::seeded(CIV_FACTION_COUNT as usize).0.len(), cartalith_io::legacy::REFERENCE_DEFAULT_ROSTER_LEN);
    }

    /// Every icon family the reader lets through is one this build places, so
    /// the `filter_map` above can never drop a row silently.
    #[test]
    fn every_legacy_icon_family_resolves() {
        for f in cartalith_io::legacy::ICON_FAMILIES {
            assert!(cartalith_assets::manual::ManualIconFamily::from_key(f).is_some(), "{f}");
        }
        for k in cartalith_io::legacy::SETTLEMENT_KINDS {
            assert!(crate::civ_tools_bridge::kind_from_str(k).is_some(), "{k}");
        }
    }

    fn fixture(name: &str) -> cartalith_io::ProjectData {
        let path = format!("{}/../cartalith-io/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        cartalith_io::read_project(std::fs::File::open(&path).expect("fixture")).expect("read")
    }

    /// The real HTML export, with its records, through to this crate's types
    /// -- values read out of `params.json` here, independently of the reader.
    #[test]
    fn the_real_legacy_export_imports_as_settlements_labels_and_icons() {
        let data = fixture("legacy_records_seed24601.zip");
        let (gw, gh) = (data.save.params.gw, data.save.params.gh);
        let state = &data.save.state;
        let got = import(data.legacy.as_ref().expect("flat"), gw, gh);

        let civ = got.civ.expect("a civ layer");
        let places = state["places"].as_array().unwrap();
        let settle: Vec<&serde_json::Value> = places
            .iter()
            .filter(|p| p["kind"].as_str().is_some_and(|k| cartalith_io::legacy::SETTLEMENT_KINDS.contains(&k)))
            .collect();
        assert!(settle.len() >= 3, "fixture shape");
        assert_eq!(civ.settlements.len(), settle.len());
        for (s, p) in civ.settlements.iter().zip(&settle) {
            assert_eq!(s.name, p["name"].as_str().unwrap());
            assert_eq!(s.placement.x as f64, p["x"].as_f64().unwrap());
            assert_eq!(s.placement.y as f64, p["y"].as_f64().unwrap());
            assert_eq!(s.pop as i64, p["pop"].as_i64().unwrap());
        }
        let names = state["civ"]["factionNames"].as_array().unwrap();
        assert_eq!(civ.faction_roster.0.len(), names.len());
        for (e, n) in civ.faction_roster.0.iter().zip(names) {
            assert_eq!(e.name, n.as_str().unwrap());
        }
        // Renamed in the reference's faction editor by the capture script, so
        // it differs from this build's default table.
        assert_eq!(civ.faction_roster.0[2].name, "The Kessan March");
        assert_ne!(civ.faction_roster.0[2].name, FactionEntry::default_for(2).name);
        let claimed = state["civ"]["territory"].as_array().unwrap().len() / 2;
        assert_eq!(civ.territory.iter().filter(|&&f| f != 0).count(), claimed);

        let labels = got.labels.expect("labels");
        assert_eq!(labels.labels.len(), 2);
        assert_eq!(labels.labels[1].name, "The Sundering Sea");
        assert_eq!(labels.labels[1].size_mode, cartalith_civ::labels::LabelSizeMode::Fixed);
        let icons = got.icons.expect("icons");
        assert_eq!(icons.icons.len(), 3);
        assert_eq!(icons.icons[1].family, cartalith_assets::manual::ManualIconFamily::Custom);
        assert_eq!(icons.icons[1].set.as_deref(), Some("my_set"));
    }

    /// The repository's first real export carries no records at all: it
    /// imports nothing and the world opens as terrain, exactly as before.
    #[test]
    fn an_export_with_no_records_imports_nothing() {
        let data = fixture("real_export_seed24601.zip");
        let got = import(data.legacy.as_ref().expect("flat"), data.save.params.gw, data.save.params.gh);
        assert!(got.civ.is_none() && got.labels.is_none() && got.icons.is_none());
        assert!(data.warnings.is_empty(), "{:?}", data.warnings);
    }
}
