//! Cartalith asset packs — the manifest layer.
//!
//! Phase 4 (`ROADMAP.md`, "Block 3, the sprite and texture pack system") in
//! this port's terms is two things bolted together in the reference: an
//! **asset pack**, a portable `.zip` of art over a frozen slot vocabulary, and
//! an **Asset Library**, the in-app workspace that authors one. The full
//! breakdown, and what this crate will and will not eventually hold, is in
//! `ASSET_LIBRARY_SCOPE.md` at the repo root.
//!
//! This crate started as milestone 1 of that plan: the pack **manifest** —
//! its data model, its parser, its validation warnings, and its
//! serialization. That is deliberately the piece with no images, no archive,
//! no renderer and no UI in it, and it is the piece everything else in Phase 4
//! is defined against.
//!
//! Milestone 3 added [`scatter`]: the [`ScatterRule`] model that decides
//! *where* an asset gets scattered on the map, its slot presets, and the
//! hardened normalizer that is the only way to build one out of a
//! user-supplied project file.
//!
//! Milestone 4 added [`placement`]: [`place_map_icons_ruled`], the placement
//! engine itself — positional and seeded, so a port either lands icons on
//! the identical cells or it does not — plus [`icon_slot_for_item`] and
//! [`sprite_draw_rect`].
//!
//! Milestone 5 added [`library`]: [`AssetDB`], [`AssetCollections`], the
//! `AssetValidator.run()` warnings ([`library::run`]), and the
//! `assetlib/library.json` record shape.
//!
//! Milestone 6 added [`raster`]: real PNG decode/encode, [`item_hash`] (a
//! real content hash from decoded pixels, feeding milestone 5's own
//! `duplicate_groups`/`slot_has_dupe`), [`fit_to_bottom`] and [`render_item`]
//! (the transform math applied to actual pixels, for thumbnails, the
//! inspector preview and a pack export's baked slot image alike),
//! [`finalize_pack_texture_inv_mean`], and
//! [`library::AssetDB::apply_library_file_with_items`] — the item
//! restoration milestone 5 deliberately left undone because it needed
//! pixels.
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
//! No `gdext`. Milestone 1 also had no dependency on any other Cartalith
//! crate; milestone 3 added exactly one, `cartalith-noise`, because
//! [`pick_weighted_variant`] falls through to the *exact* `pickIconVariant`
//! position hash when weights are absent — reimplementing that hash rather
//! than depending on the crate that already golden-matches it would be the
//! worse trade by a wide margin.
//!
//! Pack `.zip` read/write *is* here, in [`archive`], behind the on-by-default
//! `zip` feature — `default-features = false` gives back the archive-free
//! manifest model milestone 1 shipped. It is a thin policy layer over the
//! `zip` crate, not a hand-port: what is ported is the reference's own
//! STORE-the-PNGs / frozen-timestamp / `pack.json`-last export behaviour.
//!
//! Milestone 8 added [`slicer`]: the sprite-sheet slicer's real arithmetic
//! and pixel work — `computeCells`'s half-gutter cell geometry,
//! `cropCell`'s rounding and clipped blit, `applyChroma`'s background key,
//! and `isBlank`'s alpha-8 threshold, all golden-verified against the
//! reference — plus [`slicer::trim_transparent_edges`], the one control
//! `DCC_SHELL_SPEC.md` §8 asks for that the reference does not have (a
//! disclosed port-side addition; see that module's docs).
//!
//! Every part of the library UI, the sprite-sheet slicer's canvas
//! interaction (pan/zoom, draggable grid lines, click-to-select), and sprite
//! compositing into the actual map render/ground-texture sampling (milestone 7, genuinely Phase-3-adjacent rendering work
//! in `cartalith-godot`) are later milestones or explicitly out of scope; see
//! `ASSET_LIBRARY_SCOPE.md`. **Nothing in the workspace depends on this crate
//! yet** — by design, per this project's "don't wire in what nothing calls"
//! discipline.

#[cfg(feature = "zip")]
pub mod archive;
pub mod coast;
pub mod library;
pub mod manifest;
pub mod manual;
pub mod ordered_map;
pub mod placement;
pub mod raster;
pub mod scatter;
pub mod slicer;
pub mod slots;

#[cfg(feature = "zip")]
pub use archive::{
    ArchiveError, PackEntries, read_pack, read_pack_entries, write_pack, write_pack_entries,
    zip_store, zip_store_bytes,
};
pub use coast::{is_coast, is_water, snap_to_coast};
pub use library::{
    AssetCollections, AssetDB, DuplicateEntry, ItemRecord, ItemTransform, LIBRARY_POI_SLOTS,
    LibraryError, LibraryFile, LibraryItem, LibrarySlot, PackInfo, SlotMeta, SlotRecord,
    duplicate_groups, library_slot_ids, parse_library_json, slot_has_dupe, slot_title,
};
pub use raster::{
    DecodedImage, ImageError, decode_png, encode_png, finalize_pack_texture_inv_mean,
    fit_to_bottom, item_hash, render_item,
};
pub use manifest::{
    MANIFEST_CSV, MANIFEST_JSON, PackError, PackManifest, Paths, RawManifest, RawStructures,
    Structures, pack_summary, parse_pack_csv, parse_pack_entries, parse_pack_manifest,
};
pub use ordered_map::OrderedMap;
pub use placement::{
    IconCategory, IconKind, PlaceIconsRuledOpts, PlacedIcon, SpriteRect, TRAIT_BADGES_SHOWN_MAX,
    TraitBadge, icon_slot_for_item, place_map_icons_ruled, sprite_draw_rect, trait_badge_drop,
    trait_badge_layout, trait_badge_radius, trait_sprite_rect,
};
pub use slicer::{
    BLANK_ALPHA_THRESHOLD, CellGrid, CellRect, ChromaKey, GridRect, MAX_GRID_COUNT, SliceCounts,
    SliceGrid, SliceOptions, SlicedCell, apply_chroma, cell_source_rect, clamp_grid_count,
    compute_cells, count_cells, crop_cell, is_blank, move_line, sheet_base_name, slice_sheet,
    trim_transparent_edges, uniform_lines,
};
pub use scatter::{
    ScatterMode, ScatterRule, ScatterRuleTable, autopopulate_scatter_rules, current_scatter_rules,
    normalize_scatter_rule, pick_icon_variant, pick_weighted_variant, preset_scatter_rule,
    scatter_rule_key,
};
pub use slots::{
    Anchor, Family, PACK_BIOME_SLOTS, PACK_ICON_SLOTS, PACK_POI_SLOTS, PACK_SEAMARK_SLOTS,
    PACK_SETTLEMENT_SLOTS, PACK_TERRAIN_SLOTS, PACK_TEX_SLOTS, PACK_TRAIT_SLOTS, SPLAT_PAINT_SLOTS,
    slug_id,
};
