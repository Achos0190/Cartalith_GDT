//! Cartalith asset packs — the manifest layer.
//!
//! Phase 4 (`ROADMAP.md`, "Block 3, the sprite and texture pack system") in
//! this port's terms is two things bolted together in the reference: an
//! **asset pack**, a portable `.zip` of art over a frozen slot vocabulary, and
//! an **Asset Library**, the in-app workspace that authors one. The full
//! breakdown, and what this crate will and will not eventually hold, is in
//! `ASSET_LIBRARY_SCOPE.md` at the repo root.
//!
//! This crate is milestone 1 of that plan: the pack **manifest** — its data
//! model, its parser, its validation warnings, and its serialization. That is
//! deliberately the piece with no images, no archive, no renderer and no UI in
//! it, and it is the piece everything else in Phase 4 is defined against.
//!
//! ```
//! use cartalith_assets::{parse_pack_manifest, pack_summary, Family, RawManifest};
//! use std::collections::BTreeSet;
//!
//! let raw: RawManifest = serde_json::from_str(
//!     r#"{"schema":1,"name":"Woodcut","license":"CC0",
//!         "textures":{"grass":"textures/grass.png"},
//!         "icons":{"mountain":["icons/m1.png","icons/m2.png"]}}"#,
//! )
//! .unwrap();
//! let files: BTreeSet<String> =
//!     ["textures/grass.png", "icons/m1.png", "icons/m2.png"].iter().map(|s| s.to_string()).collect();
//!
//! let pack = parse_pack_manifest(&raw, &files);
//! assert!(pack.warnings.is_empty());
//! assert_eq!(pack.slot_paths(Family::Icons, "mountain").unwrap().len(), 2);
//! assert_eq!(pack_summary(&pack), "Woodcut · CC0 — 1 textures · mountain×2");
//! ```
//!
//! ## What "an asset" is here
//!
//! Not an arbitrary named image. An asset is **one PNG bound to one slot** out
//! of a frozen vocabulary the engine already knows how to draw — a splat
//! channel, a painted-biome ground tile, a scattered feature glyph, a
//! settlement/trait/POI symbol — or, for the one open-vocabulary family, a
//! user-named icon inside a user-named set. Slots may hold several *variants*
//! so a ridge of forty peaks is not forty copies of one drawing. A slot with
//! no art falls back to procedural art for that slot alone, which is why a
//! two-file pack is as valid as a hundred-file one.
//!
//! ## Design notes carried from the reference
//!
//! - **The manifest is the source of truth**, not the directory layout. Paths
//!   are ZIP-root-relative and may be anything; `textures/`, `icons/` and the
//!   rest are a convention the exporter follows and an importer must not
//!   assume.
//! - **The slot vocabulary is frozen and ordered** ([`slots`]). It is the
//!   contract between the pack author and the renderer, and two of the lists
//!   are index-aligned with vocabularies elsewhere in the engine.
//! - **Schema 2 is a strict superset of schema 1.** A schema-1 consumer reads
//!   a schema-2 pack by ignoring the sections it does not know, and unknown
//!   keys anywhere are dropped with a warning rather than rejected. Parsing a
//!   pack therefore never fails on content — only on a missing or malformed
//!   manifest.
//! - **Warnings are data, not errors.** They are ordered, and their order is
//!   golden-verified against the reference
//!   (`tests/golden_parity_pack_manifest.rs`); a UI reports the count and lets
//!   the import proceed.
//!
//! ## Not in this crate
//!
//! No `gdext`, and no dependency on any other Cartalith crate — the same
//! standalone shape `cartalith-spatial` set. Image decoding, ZIP reading and
//! writing, the Asset Library's own item store and project-embedded
//! `assetlib/library.json`, the scatter-rule engine, and every part of the
//! library UI are later milestones or explicitly out of scope; see
//! `ASSET_LIBRARY_SCOPE.md`. **Nothing in the workspace depends on this crate
//! yet** — by design, per this project's "don't wire in what nothing calls"
//! discipline.

pub mod manifest;
pub mod ordered_map;
pub mod slots;

pub use manifest::{
    MANIFEST_CSV, MANIFEST_JSON, PackError, PackManifest, Paths, RawManifest, RawStructures,
    Structures, pack_summary, parse_pack_csv, parse_pack_entries, parse_pack_manifest,
};
pub use ordered_map::OrderedMap;
pub use slots::{
    Anchor, Family, PACK_BIOME_SLOTS, PACK_ICON_SLOTS, PACK_POI_SLOTS, PACK_SETTLEMENT_SLOTS,
    PACK_TERRAIN_SLOTS, PACK_TEX_SLOTS, PACK_TRAIT_SLOTS, SPLAT_PAINT_SLOTS, slug_id,
};
