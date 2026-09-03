//! The frozen slot vocabularies an asset pack is written against.
//!
//! Ported verbatim from `Cartalith Gen1 v2.10.html` lines 12029-12052
//! (`PACK_TEX_SLOTS`/`SPLAT_PAINT_SLOTS`/`PACK_ICON_SLOTS`/
//! `PACK_STRUCT_SLOTS`/`PACK_BIOME_SLOTS`/`PACK_TERRAIN_SLOTS`) plus the
//! per-family render metadata from the Asset Library's own `FAMILIES` table
//! (reference line ~26781).
//!
//! **These lists are frozen, and order is load-bearing.** The reference's own
//! comments say so twice: the biome/terrain lists are "1:1 with the FROZEN
//! CART_BIOMES / CART_TERRAINS vocabularies (invariant 13) — slot N here is
//! index N+1 in those arrays", and the structure lists "mirror the civ layer's
//! `CIV_SETTLEMENT_CLASSES`/`CIV_POI_TYPES` keys exactly". Reordering or
//! renaming an entry silently re-points every pack ever authored.

/// Ground-material texture channels — exactly the renderer's `materialWeights`
/// outputs, so splatting drops them in with no new logic.
pub const PACK_TEX_SLOTS: [&str; 7] = [
    "grass",
    "rock",
    "sand",
    "snow",
    "wetland",
    "canopy",
    "parchment",
];

/// The Splat paint layer's palette: `PACK_TEX_SLOTS` minus `parchment`, which
/// is a paper base multiplied over the whole map rather than a ground material.
/// (The reference writes this as `PACK_TEX_SLOTS.slice(0,6)`; the equality is
/// asserted by a test rather than expressed in the type, since `const` slicing
/// of an array of `&str` is not stable across the toolchain range this project
/// targets.)
pub const SPLAT_PAINT_SLOTS: [&str; 6] = ["grass", "rock", "sand", "snow", "wetland", "canopy"];

/// Scattered feature glyphs (`placeMapIcons`). Each slot holds 1..N variants.
pub const PACK_ICON_SLOTS: [&str; 10] = [
    "mountain",
    "hill",
    "tree_conifer",
    "tree_broadleaf",
    "tree_rainforest",
    "tree_savanna",
    "tree_wetland",
    "shrub",
    "cactus",
    "boulder",
];

/// Sea marks — the coastal/offshore glyph family.
///
/// **This one is not ported.** Every other list in this file is transcribed
/// from the reference (lines 12029-12052); the reference has no sea-marks
/// family at all, and neither did this port until the owner ruled *"Build, and
/// add a sea-marks asset family"* (2026-09-02). It exists because
/// `cartography_workspace.gd`'s `ICON_PLACEMENT_FAMILIES` carries the design's
/// four *placement* families — PLACES, TREES, SEA MARKS, POI — and that file's
/// own comment states the problem plainly: *"SEA MARKS has no counterpart in
/// the engine's three families at all. Mapping one onto the other would be
/// inventing a correspondence the design does not state."* The ruling's answer
/// is a real fourth family rather than a mapping, so this is it.
///
/// **Eight slots because the design says eight.** `cartalith-dcc-parts.js:364`
/// is `'SEA MARKS':[6,8]` — six filled of eight. The *count* is the design's;
/// the eight *names* are this port's, since the design canvas names none of
/// them. They are chosen against the one rule the design does state for this
/// family — *"snap sea marks to coast"* — so every entry is something that
/// belongs on a shoreline, and half of them float: `manual.rs`'s own
/// `place_manual_icon` doc already reached for the same vocabulary when it
/// argued the click path has no sea-level gate (*"a hand-placed lighthouse or
/// buoy is a legitimate thing to want"*), written months before this family
/// existed.
///
/// Frozen from here on, on the same terms as the ported lists above: order is
/// load-bearing and renaming an entry re-points every pack authored against it.
pub const PACK_SEAMARK_SLOTS: [&str; 8] = [
    "lighthouse",
    "beacon",
    "buoy",
    "anchorage",
    "shipwreck",
    "reef",
    "shoal",
    "whirlpool",
];

/// Ground tiles for the painted Cartography biome layer (v1.28).
pub const PACK_BIOME_SLOTS: [&str; 15] = [
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
    "ocean",
];

/// Ground tiles for the painted Cartography terrain layer (v1.28).
pub const PACK_TERRAIN_SLOTS: [&str; 13] = [
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
    "ruins",
];

/// Settlement size classes — mirrors `CIV_SETTLEMENT_CLASSES`.
pub const PACK_SETTLEMENT_SLOTS: [&str; 9] = [
    "hamlet",
    "village",
    "town",
    "city",
    "capital",
    "monastery",
    "fortress",
    "university",
    "industrial",
];

/// Point-of-interest markers — mirrors `CIV_POI_TYPES`.
///
/// Note the Asset Library's own `poi` family additionally carries `lake` and
/// `bridge`; the *pack import* vocabulary deliberately does not, because those
/// two have no engine POI kind to attach to (reference comment at line 12033).
pub const PACK_POI_SLOTS: [&str; 8] = [
    "ruin",
    "landmark",
    "mountain_peak",
    "named_forest",
    "battlefield",
    "shrine",
    "cave",
    "other",
];

/// Settlement role overlays — mirrors `CIV_TRAITS`. Imported since v1.28.
///
/// **This doc used to say the reference does not draw these. It does** —
/// `_traitSprite` (v2.11 line 15571) and `_civDrawTraitBadges` (15584), added
/// by the same v1.28 that added the import, drawing up to four badges in a row
/// beneath a settlement's pin. What the reference never revisited was its own
/// "not yet used by the live map" list, which still names `trait`.
///
/// The family *is* undrawn in **this port**, which is the real reason it is the
/// one clause left in [`crate::PackManifest::warnings`]' entry: nothing
/// composites a trait sprite here. `OUTSTANDING_WORK.md` §2.5 is that gap.
pub const PACK_TRAIT_SLOTS: [&str; 7] = [
    "fortified",
    "mining",
    "port",
    "administrative",
    "trade_hub",
    "military",
    "religious",
];

/// Which section of a `pack.json` an asset belongs to.
///
/// The four `structures.*` families and `custom` are schema-2 additions;
/// `textures` and `icons` are the schema-1 core the live renderer consumes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Family {
    /// `textures` — splat channels.
    Textures,
    /// `biomes` — painted biome-layer ground tiles.
    Biomes,
    /// `terrains` — painted terrain-layer ground tiles.
    Terrains,
    /// `icons` — scattered feature glyphs.
    Icons,
    /// `seamarks` — coastal and offshore marks. **The one family with no
    /// reference counterpart** (owner ruling 2026-09-02); see
    /// [`PACK_SEAMARK_SLOTS`].
    SeaMark,
    /// `structures.settlement` — settlement pins.
    Settlement,
    /// `structures.trait` — settlement role overlays.
    Trait,
    /// `structures.poi` — point-of-interest markers.
    Poi,
    /// `custom` — free-form user icon sets (open vocabulary, two-level
    /// `set -> slot -> [paths]`).
    Custom,
}

/// Where a sprite's art is anchored when drawn on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// Feature glyphs: the base of the drawing sits on the map cell, like a
    /// label (`spriteDrawRect`).
    Bottom,
    /// Settlement/trait/POI/custom symbols: centred on the point.
    Center,
    /// Ground textures are tiled, not anchored.
    None,
}

impl Family {
    /// Every family, in the order the Asset Library's own `FAMILIES` table
    /// lists them — plus [`Family::SeaMark`], which that table does not have.
    ///
    /// It is inserted after [`Family::Icons`] rather than appended, because
    /// this order is the exporter's file order ([`crate::archive`]'s
    /// `export_order`) and the Library's display order, and a sea mark is a
    /// top-level multi-variant sprite section exactly like `icons` — grouping
    /// it there rather than after `custom` (which is last on purpose: the open
    /// vocabulary sorts behind every frozen one) keeps both orders readable.
    /// Nothing about an existing pack moves: a pack with no `seamarks` section
    /// contributes nothing at the new position.
    pub const ALL: [Family; 9] = [
        Family::Textures,
        Family::Biomes,
        Family::Terrains,
        Family::Icons,
        Family::SeaMark,
        Family::Settlement,
        Family::Trait,
        Family::Poi,
        Family::Custom,
    ];

    /// The three families nested under the manifest's `structures` section.
    pub const STRUCTURES: [Family; 3] = [Family::Settlement, Family::Trait, Family::Poi];

    /// The family's key as the Asset Library addresses it.
    pub fn key(self) -> &'static str {
        match self {
            Family::Textures => "textures",
            Family::Biomes => "biomes",
            Family::Terrains => "terrains",
            Family::Icons => "icons",
            // Plural, like every other *top-level* section (`textures`,
            // `biomes`, `terrains`, `icons`); the singular keys below are the
            // three nested under `structures`. `ManualIconFamily::SeaMark`
            // reuses this exact string rather than coining a second spelling —
            // the `feature`/`icons` rename it has to live with elsewhere is the
            // reference's, not a pattern worth repeating on a new family.
            Family::SeaMark => "seamarks",
            Family::Settlement => "settlement",
            Family::Trait => "trait",
            Family::Poi => "poi",
            Family::Custom => "custom",
        }
    }

    /// The inverse of [`Family::key`] — the reference's
    /// `FAMILIES.find(f=>f.key===rec.fam)`, used to resolve a persisted
    /// `library.json` slot record's `fam` string back to a [`Family`].
    pub fn from_key(key: &str) -> Option<Family> {
        Family::ALL.into_iter().find(|f| f.key() == key)
    }

    /// The frozen slot list, or `&[]` for [`Family::Custom`]'s open vocabulary.
    pub fn slots(self) -> &'static [&'static str] {
        match self {
            Family::Textures => &PACK_TEX_SLOTS,
            Family::Biomes => &PACK_BIOME_SLOTS,
            Family::Terrains => &PACK_TERRAIN_SLOTS,
            Family::Icons => &PACK_ICON_SLOTS,
            Family::SeaMark => &PACK_SEAMARK_SLOTS,
            Family::Settlement => &PACK_SETTLEMENT_SLOTS,
            Family::Trait => &PACK_TRAIT_SLOTS,
            Family::Poi => &PACK_POI_SLOTS,
            Family::Custom => &[],
        }
    }

    /// Directory the exporter writes this family's PNGs into, relative to the
    /// ZIP root. A convention only — the manifest is the source of truth for
    /// what maps to what, and an importer must follow the declared paths.
    pub fn dir(self) -> &'static str {
        match self {
            Family::Textures => "textures",
            Family::Biomes => "biomes",
            Family::Terrains => "terrains",
            Family::Icons => "icons",
            Family::SeaMark => "seamarks",
            Family::Settlement => "structures/settlement",
            Family::Trait => "structures/trait",
            Family::Poi => "structures/poi",
            Family::Custom => "custom",
        }
    }

    /// Whether a slot in this family holds several variants (`true`) or a
    /// single image (`false`).
    pub fn is_multi(self) -> bool {
        !self.is_texture()
    }

    /// Ground textures are tiled and opaque; everything else is an RGBA sprite.
    pub fn is_texture(self) -> bool {
        matches!(self, Family::Textures | Family::Biomes | Family::Terrains)
    }

    /// Export bake size in pixels (square): 512 for ground textures, 256 for
    /// sprites.
    pub fn size(self) -> u32 {
        if self.is_texture() { 512 } else { 256 }
    }

    /// Ground textures bake opaque (alpha flattened on black); sprites keep
    /// their alpha.
    pub fn opaque(self) -> bool {
        self.is_texture()
    }

    /// Where the sprite is anchored on the map.
    pub fn anchor(self) -> Anchor {
        match self {
            Family::Textures | Family::Biomes | Family::Terrains => Anchor::None,
            Family::Icons => Anchor::Bottom,
            // **Centred, not bottom-anchored, and this is a choice rather than
            // a transcription** — no reference behaviour to copy. A feature
            // glyph is bottom-anchored because a mountain or a tree *stands* on
            // its cell; half of `PACK_SEAMARK_SLOTS` (buoy, reef, shoal,
            // whirlpool) has no base to stand on, and after the coast snap the
            // point IS the mark's position. One anchor for the family, and
            // centre is the one that is right for the floating half and merely
            // half a sprite high for the standing half.
            Family::SeaMark => Anchor::Center,
            _ => Anchor::Center,
        }
    }

    /// Whether `slot_id` is part of this family's frozen vocabulary.
    /// Always `false` for [`Family::Custom`], whose vocabulary is open.
    pub fn has_slot(self, slot_id: &str) -> bool {
        self.slots().contains(&slot_id)
    }

    /// The path the pack exporter writes a given asset to, relative to the ZIP
    /// root — the convention `PackManifestBuilder.build()` follows (reference
    /// line ~26968).
    ///
    /// `variant` is zero-based and ignored for the three texture families,
    /// which hold exactly one image per slot. `set_id` is used only by
    /// [`Family::Custom`] and must already be slugified ([`crate::slug_id`]).
    pub fn asset_path(self, slot_id: &str, set_id: &str, variant: usize) -> String {
        if self.is_texture() {
            return format!("{}/{}.png", self.dir(), slot_id);
        }
        if self == Family::Custom {
            return format!("custom/{}/{}_{:02}.png", set_id, slot_id, variant + 1);
        }
        format!("{}/{}_{:02}.png", self.dir(), slot_id, variant + 1)
    }
}

/// Slugify a user-supplied name into a filename-safe id — the reference's own
/// `slugId` (line 26825). Lowercases, collapses every non-alphanumeric run to a
/// single `_`, trims leading/trailing `_`, and falls back to `"icon"` when
/// nothing survives.
pub fn slug_id(s: &str) -> String {
    let lowered = s.to_lowercase();
    let mut out = String::with_capacity(lowered.len());
    let mut pending_underscore = false;
    for ch in lowered.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_underscore && !out.is_empty() {
                out.push('_');
            }
            pending_underscore = false;
            out.push(ch);
        } else {
            pending_underscore = true;
        }
    }
    if out.is_empty() {
        "icon".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splat_paint_slots_is_the_first_six_texture_slots() {
        // The reference's own `PACK_TEX_SLOTS.slice(0,6)`, asserted rather
        // than expressed, so the two lists cannot silently drift apart.
        assert_eq!(SPLAT_PAINT_SLOTS[..], PACK_TEX_SLOTS[..6]);
        assert!(!SPLAT_PAINT_SLOTS.contains(&"parchment"));
    }

    #[test]
    fn frozen_vocabulary_sizes_match_the_reference() {
        // The counts the format doc and the reference's own FAMILIES table state.
        assert_eq!(PACK_TEX_SLOTS.len(), 7);
        assert_eq!(PACK_ICON_SLOTS.len(), 10);
        assert_eq!(PACK_BIOME_SLOTS.len(), 15);
        assert_eq!(PACK_TERRAIN_SLOTS.len(), 13);
        assert_eq!(PACK_SETTLEMENT_SLOTS.len(), 9);
        assert_eq!(PACK_TRAIT_SLOTS.len(), 7);
        // 8, not the Asset Library's 10: `lake`/`bridge` have no engine POI kind.
        assert_eq!(PACK_POI_SLOTS.len(), 8);
        assert!(!PACK_POI_SLOTS.contains(&"lake"));
        assert!(!PACK_POI_SLOTS.contains(&"bridge"));
    }

    #[test]
    fn seamark_vocabulary_is_the_eight_the_design_asked_for() {
        // `cartalith-dcc-parts.js:364`'s `'SEA MARKS':[6,8]`. The panel's own
        // "N of M slots filled" line reads M off this list now, so the count is
        // load-bearing in both directions.
        assert_eq!(PACK_SEAMARK_SLOTS.len(), 8);
        assert_eq!(Family::SeaMark.slots().len(), 8);
        assert!(PACK_SEAMARK_SLOTS.contains(&"lighthouse"));
        assert!(PACK_SEAMARK_SLOTS.contains(&"buoy"));
        // A sea mark is a sprite, not a ground tile, and it is centred.
        assert!(Family::SeaMark.is_multi());
        assert!(!Family::SeaMark.opaque());
        assert_eq!(Family::SeaMark.size(), 256);
        assert_eq!(Family::SeaMark.anchor(), Anchor::Center);
        // Its own top-level section, NOT one of the three under `structures`.
        assert!(!Family::STRUCTURES.contains(&Family::SeaMark));
        assert_eq!(Family::SeaMark.dir(), "seamarks");
    }

    #[test]
    fn seamark_shares_no_slot_id_with_the_family_it_could_be_confused_for() {
        // The whole point of the ruling: SEA MARKS is not POI wearing a hat.
        // If these two ever share a slot id, `make_uid`'s `fam:slot` key still
        // separates them, but the vocabularies would have started to merge.
        for s in PACK_SEAMARK_SLOTS {
            assert!(!PACK_POI_SLOTS.contains(&s), "{s} is in both seamarks and poi");
            assert!(!PACK_ICON_SLOTS.contains(&s), "{s} is in both seamarks and icons");
        }
    }

    #[test]
    fn no_slot_id_repeats_inside_a_family() {
        for fam in Family::ALL {
            let mut seen = std::collections::BTreeSet::new();
            for s in fam.slots() {
                assert!(seen.insert(*s), "{} repeats slot {s}", fam.key());
            }
        }
    }

    #[test]
    fn family_metadata() {
        assert_eq!(Family::Textures.size(), 512);
        assert!(Family::Biomes.opaque());
        assert_eq!(Family::Icons.size(), 256);
        assert!(!Family::Icons.opaque());
        assert_eq!(Family::Icons.anchor(), Anchor::Bottom);
        assert_eq!(Family::Settlement.anchor(), Anchor::Center);
        assert_eq!(Family::Custom.anchor(), Anchor::Center);
        assert_eq!(Family::Textures.anchor(), Anchor::None);
        assert!(!Family::Textures.is_multi());
        assert!(Family::Poi.is_multi());
        assert!(Family::Custom.slots().is_empty());
    }

    #[test]
    fn asset_paths_follow_the_exporter_convention() {
        assert_eq!(
            Family::Textures.asset_path("grass", "", 0),
            "textures/grass.png"
        );
        assert_eq!(
            Family::Biomes.asset_path("jungle", "", 3),
            "biomes/jungle.png"
        );
        assert_eq!(
            Family::Icons.asset_path("mountain", "", 0),
            "icons/mountain_01.png"
        );
        assert_eq!(
            Family::Icons.asset_path("mountain", "", 9),
            "icons/mountain_10.png"
        );
        assert_eq!(
            Family::Settlement.asset_path("hamlet", "", 1),
            "structures/settlement/hamlet_02.png"
        );
        assert_eq!(
            Family::Trait.asset_path("port", "", 0),
            "structures/trait/port_01.png"
        );
        assert_eq!(
            Family::Custom.asset_path("lighthouse", "naval", 1),
            "custom/naval/lighthouse_02.png"
        );
    }

    #[test]
    fn from_key_is_the_inverse_of_key() {
        for fam in Family::ALL {
            assert_eq!(Family::from_key(fam.key()), Some(fam));
        }
        assert_eq!(Family::from_key("nope"), None);
        assert_eq!(Family::from_key(""), None);
    }

    #[test]
    fn slug_id_matches_the_references_own_examples() {
        // The two examples ASSET_PACK_FORMAT.md gives by name.
        assert_eq!(slug_id("Naval"), "naval");
        assert_eq!(slug_id("Wind Mill!!"), "wind_mill");
        // Leading/trailing separators are trimmed, interior runs collapse.
        assert_eq!(slug_id("  --Old Ruin--  "), "old_ruin");
        assert_eq!(slug_id("a...b"), "a_b");
        // Nothing survives -> the reference's own fallback.
        assert_eq!(slug_id("!!!"), "icon");
        assert_eq!(slug_id(""), "icon");
        // Non-ASCII is not alphanumeric here, exactly as the JS regex
        // `[^a-z0-9]+` treats it.
        assert_eq!(slug_id("Château"), "ch_teau");
    }
}
