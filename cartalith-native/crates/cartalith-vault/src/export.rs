//! The exportable-field registry and the block renderer
//! (`MARKDOWN_VAULT_INTEGRATION.md` §19, §20, §25).
//!
//! §19's requirement is the load-bearing one: *"Exportable information shall
//! not be hardcoded into the Markdown UI."* So the vocabulary lives here, as
//! data, and three things read it — the checkbox list (§20), the block
//! renderer, and the author-field mapping the owner added on 2026-08-18. The
//! UI knows how to draw a checkbox; it does not know what a settlement has.
//!
//! §20's other rule is enforced by [`render_body`] rather than by the UI:
//! *"The UI must not expose information that the entity does not possess."*
//! A field whose value the caller could not supply is absent from the offer
//! list, not offered-and-blank — a blank row in someone's note is a small lie
//! about the world.
//!
//! ## §19's Map group (added 2026-09-02, milestone 2)
//!
//! [`MAP_RADII`] and the three `map_*` fields are §21's immediate/local/
//! regional snapshot. The *rendering* stays out of this crate — that is a
//! `cartalith-godot` concern and `vault_bridge.rs`'s `vault_snapshot` owns
//! it — so what lives here is only the vocabulary and the values, which is
//! the same split every other field already has.
//!
//! A Map field's **value is a Markdown image**, `![](relative/path.png)`,
//! written by the caller from the path `LinkStore::snapshot` remembers. §22
//! forbids base64-embedding the image itself and this obeys that; the path is
//! relative to the vault root, so the note stays readable after the vault is
//! copied to another machine (§5).
//!
//! ## What §19 lists and this still does not carry
//!
//! §19's *Open-in-Cartalith link* is absent: it is `obsidian://`-adjacent
//! URL-scheme registration, which the owner's 2026-08-18 clarification put
//! outside the core. Named in the scope document rather than stubbed here.

use crate::links::EntityKind;

/// One thing Cartalith can put in a note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportField {
    /// Stable id, used by the UI's selection set and by the save. Never
    /// shown to a user.
    pub key: &'static str,
    /// §19's own group headings, which become the block's bold sub-headers.
    pub group: &'static str,
    /// What the user sees, in the checkbox list and in the note.
    pub label: &'static str,
    /// Which entity kinds can ever supply this. Narrower than "does this
    /// entity have a value right now" — that second question is the caller's,
    /// and [`offer`] asks it.
    pub kinds: &'static [EntityKind],
}

const S: &[EntityKind] = &[EntityKind::Settlement];
/// The three *place* kinds — every one of these has a position on the map.
/// A faction does not, which is why it is not in here and `EVERY` exists.
const ALL: &[EntityKind] = &[EntityKind::Settlement, EntityKind::Province, EntityKind::Continent];
/// `GUI_GAP_REGISTER.md` **CV-22**: the two fields every addressable entity
/// has, factions and cultures included.
const EVERY: &[EntityKind] = &[
    EntityKind::Settlement,
    EntityKind::Province,
    EntityKind::Continent,
    EntityKind::Faction,
    EntityKind::Culture,
];
const F: &[EntityKind] = &[EntityKind::Faction];
/// `GUI_GAP_REGISTER.md` **CV-02**.
const C: &[EntityKind] = &[EntityKind::Culture];
/// Everything that resolves to a coordinate on the map. A faction does so
/// through its capital; a **culture does not resolve to one at all** — it is
/// a naming vocabulary several factions can share, so any point offered for
/// it would be a fabrication.
const PLACED: &[EntityKind] =
    &[EntityKind::Settlement, EntityKind::Province, EntityKind::Continent, EntityKind::Faction];
/// A named list of member settlements, and the population that follows from
/// it — a province partitions a faction, so both aggregate the same way, and
/// a culture aggregates over the factions that speak it.
const PF: &[EntityKind] = &[EntityKind::Province, EntityKind::Faction, EntityKind::Culture];

/// The registry. Order here is the order in the UI and in the note, so it is
/// §19's own group order.
pub const FIELDS: &[ExportField] = &[
    ExportField { key: "name", group: "Identity", label: "Name", kinds: EVERY },
    ExportField { key: "entity_type", group: "Identity", label: "Entity type", kinds: EVERY },
    // A faction's own three roster vocabularies (CV-22). `ECONOMY_SCOPE.md`
    // found none of them drives anything in either codebase -- which is the
    // argument for exporting them into a note, where an author's own prose
    // about their government is the thing that *does* carry meaning.
    ExportField { key: "culture", group: "Identity", label: "Culture", kinds: F },
    ExportField { key: "government", group: "Identity", label: "Government", kinds: F },
    ExportField { key: "religion", group: "Identity", label: "Religion", kinds: F },
    // A culture's own row. `terrain_affinity` is
    // `CIV_CULTURE_TERRAIN_KEY`'s thematic pairing (highland->hills and the
    // rest) and is absent for `common`/`imperial`, which `civ_culture_
    // terrain_fit` deliberately gives no verdict rather than a fabricated
    // one -- so those two are simply not offered the field.
    ExportField { key: "factions", group: "Identity", label: "Factions", kinds: C },
    ExportField { key: "terrain_affinity", group: "Geography", label: "Terrain affinity", kinds: C },
    // A faction's capital's position: it has no centroid of its own, and the
    // seat of power is the coordinate a note would want.
    ExportField { key: "coordinates", group: "Geography", label: "Coordinates", kinds: PLACED },
    ExportField { key: "elevation", group: "Geography", label: "Elevation", kinds: S },
    ExportField { key: "biome", group: "Geography", label: "Biome", kinds: S },
    ExportField { key: "region", group: "Geography", label: "Province", kinds: S },
    ExportField { key: "area", group: "Geography", label: "Area", kinds: &[EntityKind::Continent, EntityKind::Faction] },
    ExportField { key: "coastal", group: "Geography", label: "Coastal", kinds: S },
    ExportField { key: "settlement_type", group: "Settlement", label: "Settlement type", kinds: S },
    ExportField { key: "population", group: "Settlement", label: "Population", kinds: &[EntityKind::Settlement, EntityKind::Province, EntityKind::Faction, EntityKind::Culture] },
    ExportField { key: "faction", group: "Settlement", label: "Faction", kinds: ALL },
    ExportField { key: "capital", group: "Settlement", label: "Capital", kinds: S },
    ExportField { key: "specialisation", group: "Settlement", label: "Economy", kinds: S },
    ExportField { key: "exports", group: "Infrastructure", label: "Exports", kinds: S },
    ExportField { key: "imports", group: "Infrastructure", label: "Imports", kinds: S },
    ExportField { key: "river_order", group: "Infrastructure", label: "River order", kinds: S },
    ExportField { key: "settlements", group: "Infrastructure", label: "Settlements", kinds: PF },
    // §19's Map group — milestone 2. Offered only to the kinds that resolve
    // to a coordinate (`PLACED`), and, like every other field, only once the
    // caller has a value: `offer`'s availability test is what keeps these out
    // of the checkbox list until a snapshot has actually been written, so the
    // note can never carry a link to an image that is not there.
    ExportField { key: "map_immediate", group: "Map", label: "Immediate map", kinds: PLACED },
    ExportField { key: "map_local", group: "Map", label: "Local map", kinds: PLACED },
    ExportField { key: "map_regional", group: "Map", label: "Regional map", kinds: PLACED },
];

/// §21's three radii: `(field key, radius name, half-width in km)`.
///
/// **In kilometres, not in cells.** A cell is a different real distance in
/// every world — `map_width_km / gw` — so a radius in cells would make
/// "immediate" mean a village's fields on one map and a whole province on the
/// next. The caller converts through its own cell size and clamps; this table
/// is the vocabulary, and it lives here rather than in the bridge for §19's
/// own reason: *"Exportable information shall not be hardcoded into the
/// Markdown UI."*
///
/// The three numbers are a walk, a day's ride and a week's travel, which is
/// the scale §21's own words ("Immediate / Local / Regional") describe. §21
/// says *"the exact radius/scale may be configurable"*; it is not yet, and
/// that is a scope line rather than an oversight.
pub const MAP_RADII: &[(&str, &str, f64)] =
    &[("map_immediate", "immediate", 10.0), ("map_local", "local", 50.0), ("map_regional", "regional", 250.0)];

/// The radius name behind a Map field key, or `None` for every other field.
pub fn map_radius(key: &str) -> Option<&'static str> {
    MAP_RADII.iter().find(|(k, _, _)| *k == key).map(|(_, r, _)| *r)
}

/// The Map field key for a radius name — [`map_radius`] the other way round.
pub fn map_field(radius: &str) -> Option<&'static str> {
    MAP_RADII.iter().find(|(_, r, _)| *r == radius).map(|(k, _, _)| *k)
}

// There is deliberately no `continent` field for a settlement or a province,
// though §19's Geography group would want one. Answering "which landmass is
// this cell on" needs the per-cell component raster, and
// `cartalith_civ::civ_continents` does not keep one — 268 MB at this port's
// 8192² ceiling, for a lookup nothing else performs. Offering the field and
// filling it from bounding-box containment would be a guess, and a wrong one
// for any two landmasses whose boxes overlap. Recorded in
// `MARKDOWN_VAULT_SCOPE.md` as a known omission with its cost, rather than
// approximated.

/// Which author-template field each export field may populate, for the
/// owner's 2026-08-18 amendment to §23.
///
/// Names are the ones in `design/vault-templates/Settlement Template.md`
/// verbatim. A field with no entry here is block-only: Cartalith will put it
/// in its own block and never write it into the author's prose, because
/// there is no field of the author's for it to go in.
pub const AUTHOR_FIELDS: &[(&str, &str)] = &[
    ("population", "Size / Population"),
    ("settlement_type", "Type"),
    ("region", "Location"),
];

/// The fields offerable for `kind`, filtered to those the caller actually
/// has a value for — §20's "must not expose information that the entity does
/// not possess", enforced once here rather than in every panel.
pub fn offer(kind: EntityKind, available: &dyn Fn(&str) -> bool) -> Vec<&'static ExportField> {
    FIELDS.iter().filter(|f| f.kinds.contains(&kind) && available(f.key)).collect()
}

pub fn field(key: &str) -> Option<&'static ExportField> {
    FIELDS.iter().find(|f| f.key == key)
}

/// The Markdown body of a Cartalith block: §18's shape, built from
/// `(key, value)` pairs in registry order and grouped by §19's own groups.
///
/// `selected` is the user's checkbox set; a key not in the registry, or with
/// no value supplied, is skipped silently — the UI has already been told what
/// is offerable and a caller passing something else is asking for a blank
/// row, which §20 forbids.
///
/// Deliberately plain Markdown: a `##` heading, bold group labels, `-` list
/// rows. No callouts, no wikilinks, no `obsidian://`. It renders identically
/// in Obsidian, in a plain viewer and in a diff, which is what §10's "should
/// not require Obsidian" means for the text Cartalith itself writes.
pub fn render_body(heading: &str, selected: &[String], values: &dyn Fn(&str) -> Option<String>) -> String {
    let mut out = format!("\n## {heading}\n");
    let mut current_group = "";
    for f in FIELDS {
        if !selected.iter().any(|k| k == f.key) {
            continue;
        }
        let Some(v) = values(f.key).filter(|v| !v.trim().is_empty()) else { continue };
        if f.group != current_group {
            out.push_str(&format!("\n**{}**\n", f.group));
            current_group = f.group;
        }
        out.push_str(&format!("- {}: {}\n", f.label, v.trim()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_is_unique_and_every_author_mapping_names_a_real_field() {
        let mut keys: Vec<&str> = FIELDS.iter().map(|f| f.key).collect();
        let n = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), n, "duplicate export-field key");
        for (k, _) in AUTHOR_FIELDS {
            assert!(field(k).is_some(), "{k} maps an author field but is not in the registry");
        }
    }

    /// §21's three radii, pinned as **literals** rather than described.
    ///
    /// Every consumer reads the table. `vault_bridge.rs`'s
    /// `vault_snapshot_radii`, `vault_snapshot` and `entity_values` are the
    /// only three FUNCTIONS outside this file that read it (five references in
    /// all — corrected 2026-09-05 after a verifier counted them: `vault_snapshot`
    /// reads it twice and one hit is a doc link). None of them names a number,
    /// and the shell reads the km back out of
    /// `vault_snapshot_radii`'s own rows. So nothing else pins these values,
    /// and until this test any of them could move while the suite stayed
    /// green and every snapshot in every note silently covered a different
    /// area — `assert!(km > 0.0)`, what the test below has, holds for any
    /// replacement.
    ///
    /// Measured rather than argued: with this test skipped, 10->12, 50->60,
    /// 250->200 and immediate/regional swapped all **SURVIVE**
    /// `cargo test -p cartalith-vault`; with it, all four are killed here.
    ///
    /// Half-widths in km, in the table's own order, which is the order the
    /// checkbox list and the note's Map rows are drawn in. Strictly
    /// increasing, or "Immediate map" would offer a wider crop than
    /// "Regional map" while still being labelled the tighter one.
    #[test]
    fn the_three_radii_are_the_literals_every_written_note_was_scaled_to() {
        assert_eq!(
            MAP_RADII,
            &[
                ("map_immediate", "immediate", 10.0),
                ("map_local", "local", 50.0),
                ("map_regional", "regional", 250.0),
            ]
        );
        for pair in MAP_RADII.windows(2) {
            assert!(pair[0].2 < pair[1].2, "{} must crop tighter than {}", pair[0].1, pair[1].1);
        }
    }

    /// Milestone 2's Map group, from the three sides that have a wrong answer
    /// available: a culture is not a place and must never be offered a map; a
    /// placed entity with no snapshot yet must not be offered a blank one
    /// (§20); and the two lookups must agree with the table they read.
    #[test]
    fn the_map_group_is_offered_only_to_places_that_have_a_snapshot() {
        for (key, radius, km) in MAP_RADII {
            assert!(field(key).is_some(), "{key} is a radius with no registry row");
            assert_eq!(map_radius(key), Some(*radius));
            assert_eq!(map_field(radius), Some(*key));
            assert!(*km > 0.0);
        }
        assert_eq!(map_radius("population"), None);
        assert_eq!(map_field("planetary"), None);

        // A culture has no coordinate, so no radius means anything for it.
        let culture = offer(EntityKind::Culture, &|_| true);
        for (key, _, _) in MAP_RADII {
            assert!(culture.iter().all(|f| f.key != *key), "a culture was offered {key}");
        }
        // A settlement with no snapshot written is not offered a blank row...
        let none_yet = offer(EntityKind::Settlement, &|k| map_radius(k).is_none());
        for (key, _, _) in MAP_RADII {
            assert!(none_yet.iter().all(|f| f.key != *key), "{key} was offered with no image behind it");
        }
        // ...and is, once one exists.
        let one = offer(EntityKind::Settlement, &|k| k != "map_regional");
        assert!(one.iter().any(|f| f.key == "map_local"));
        assert!(one.iter().all(|f| f.key != "map_regional"));

        // The block puts them under one **Map** header, after Infrastructure.
        let body = render_body("Cartalith", &["name".into(), "map_local".into()], &|k| match k {
            "name" => Some("Nareth".into()),
            "map_local" => Some("![](.cartalith/maps/settlement_42_local.png)".into()),
            _ => None,
        });
        assert_eq!(
            body,
            "\n## Cartalith\n\n**Identity**\n- Name: Nareth\n\n**Map**\n- Local map: ![](.cartalith/maps/settlement_42_local.png)\n"
        );
    }

    #[test]
    fn offer_hides_what_the_entity_cannot_have_and_what_it_does_not_have() {
        // A continent has no settlement type at all, whatever the caller says.
        let all = offer(EntityKind::Continent, &|_| true);
        assert!(all.iter().all(|f| f.key != "settlement_type"));
        assert!(all.iter().any(|f| f.key == "area"));
        // And a settlement whose biome the caller could not read is not
        // offered a blank Biome row.
        let some = offer(EntityKind::Settlement, &|k| k != "biome");
        assert!(some.iter().all(|f| f.key != "biome"));
        assert!(some.iter().any(|f| f.key == "population"));
    }

    /// CV-02's kind, checked from both sides: what a culture *can* have, and
    /// what it definitely cannot. The second half is the one with a wrong
    /// answer available — a culture is not a place and must never be offered
    /// a coastline.
    #[test]
    fn a_culture_is_offered_its_own_fields_and_no_places_fields() {
        let all = offer(EntityKind::Culture, &|_| true);
        let keys: Vec<&str> = all.iter().map(|f| f.key).collect();
        for want in ["name", "entity_type", "factions", "settlements", "population", "terrain_affinity"] {
            assert!(keys.contains(&want), "a culture can have {want}: {keys:?}");
        }
        for never in ["coordinates", "elevation", "biome", "coastal", "settlement_type", "area", "faction"] {
            assert!(!keys.contains(&never), "a culture is not a place: {never} was offered");
        }
        // `common` and `imperial` have no terrain theme, so the caller
        // supplies no value and the field is not offered at all.
        let identity = offer(EntityKind::Culture, &|k| k != "terrain_affinity");
        assert!(identity.iter().all(|f| f.key != "terrain_affinity"));
    }

    #[test]
    fn render_body_groups_in_registry_order_and_skips_empties() {
        let selected: Vec<String> = ["name", "population", "biome", "elevation"].iter().map(|s| s.to_string()).collect();
        let body = render_body("Cartalith", &selected, &|k| match k {
            "name" => Some("Nareth".into()),
            "population" => Some("8,420".into()),
            "biome" => Some("   ".into()), // whitespace is not a value
            _ => None,
        });
        assert_eq!(
            body,
            "\n## Cartalith\n\n**Identity**\n- Name: Nareth\n\n**Settlement**\n- Population: 8,420\n"
        );
        assert!(!body.contains("Biome"));
        assert!(!body.contains("Elevation"));
        // Nothing selected is an empty block, not a malformed one.
        assert_eq!(render_body("Cartalith", &[], &|_| None), "\n## Cartalith\n");
    }
}
