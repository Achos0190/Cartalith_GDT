//! Real asset-pack loading and sprite compositing — `ASSET_LIBRARY_SCOPE.md`
//! milestone 7, the final milestone of Phase 4. Consumes `cartalith-assets`
//! (milestones 1-6, all pure logic, none of it wired to anything before this
//! milestone) and does the actual pixel work: decoding a pack's images and
//! blitting/blending them onto the RGB8 buffer `lib.rs::build_color_texture`
//! already builds.
//!
//! ## What this module composites, and what it deliberately does not
//!
//! **All three of the milestone's named surfaces are real here** — the third
//! landed last, long after the first two:
//!
//! - **Sprite compositing** (`drawMapIcons`' painter's pass, `composite_map_icons`
//!   below) — the `icons` family (`PACK_ICON_SLOTS`, 10 scattered-feature
//!   slots), plus a real per-slot procedural glyph fallback (`draw_icon_glyph`)
//!   for a slot a loaded pack doesn't cover, matching the reference's own
//!   "shrub — also the honest catch-all" comment.
//! - **Ground-texture splat sampling** (`PACK_TEX_SLOTS`' six material
//!   channels — grass/rock/sand/snow/wetland/canopy) — decoded here
//!   ([`SplatChannel`]) and consumed by `render.rs`'s `land_color`, since
//!   `materialWeights`' own six fractions and each material's own procedural
//!   ramp colour already live there and the splat blend is a read-only
//!   consumer of both.
//! - **The two "painted layers"** (`_paintedTex`'s `biomes`/`terrains`
//!   families, the Cartography paint-brush biome/terrain override) — decoded
//!   here ([`crate::render::GroundTile`], `LoadedPack::biomes`/`::terrains`)
//!   and consumed by `render.rs`'s own paint blend, which had been taking the
//!   flat-swatch branch unconditionally because nothing ever handed it a
//!   tile. **This was the milestone's last unbuilt surface**; the two
//!   paragraphs below are the record of why, kept because they name the
//!   blocker that lifted and the one that never existed.
//!
//! **A fourth surface landed later still, and it is not one of that
//! milestone's three:** trait-badge art (`structures.trait`, `LoadedPack::
//! traits`) — `OUTSTANDING_WORK.md` §2.5, the first of the `structures`
//! families to be rasterised at all. It differs from the three above in the
//! way that matters when reading the rest of this file: **it does not go
//! through `build_color_texture` and it is not painted by this module's
//! rasterizer.** A trait badge hangs off a settlement pin at a constant
//! on-screen size, and pins are drawn by `godot-project/map_overlay.gd`, so
//! [`composite_map_icons`] never sees one and a badge baked into the terrain
//! buffer would be at map scale.
//!
//! What this module supplies for it is the *decisions* —
//! [`resolve_trait_badges`]: the row layout, the pack lookup, the variant
//! pick and each sprite's destination box. `WorldGen::civ_trait_badge_row`
//! (`lib.rs`) hands that answer to `map_overlay.gd` with an `ImageTexture`
//! per badge, and Godot's own renderer does the blit.
//! [`composite_trait_badges`] is the same decisions painted into an RGB8
//! raster instead, and it still has no caller — see its own doc.
//!
//! Read literally (reference lines 7898-7900,
//! 12187-12196): `pBio`/`pTer` are per-cell indices into
//! `state.cartoPaint.biome`/`.terrain`, and `_paintedTex(fam, slots, idx,
//! px, py)` samples the loaded pack's image for that index — one texel per
//! grid cell, wrapped — falling back to the flat palette swatch (`_t ||
//! CART_BIOME_COLS[pBio-1]`) whenever no texture is loaded for it.
//!
//! When this milestone shipped, this doc said the port had no producer of a
//! painted-cell array anywhere in the workspace and stopped there. **That
//! ceased to be true on 2026-08-24**; `render.rs`'s own module doc (lines
//! 32-35, the "and so was the **paint-brush biome/terrain override**"
//! sentence) took the correction that day and this one never did. The producer
//! is real and reachable end to end: `PaintEditor` (`paint_bridge.rs:235`,
//! `pub struct PaintEditor`) over three `PaintLayer`s
//! (`cartalith-spatial/src/paint.rs:207`, `pub struct PaintLayer`), baked by
//! `PaintEditor::commit_all` (`paint_bridge.rs:494`);
//! `WorldGen::get_paint_layers` (`lib.rs:6660`); the committed cells handed to
//! the renderer by the `ctx.with_paint(...)` call site (`lib.rs:4800`, in
//! `build_color_texture`; the builder itself is `render.rs:2264`); and a brush
//! UI driving all of it in the DCC shell — `_build_paint`
//! (`shell/workspaces/world_workspace.gd:1960`) for the WORLD dock's
//! "Biome paint" panel and `_build_paint_options`
//! (`shell/tool_bar.gd:407`) for the canvas tool-options row.
//! `land_color`'s paint blend (`render.rs:3106`, the `if !paint.is_empty()`
//! branch of `fn land_color` at `render.rs:2740`) already applies
//! `pBio`/`pTer`, at the reference's own `0.60` weight and in its own
//! position.
//!
//! Every line number in this paragraph was re-checked against the working
//! tree on 2026-08-31; cite the symbol beside the number when adding to it,
//! because concurrent edits move these files faster than the comment is
//! re-read.
//!
//! What was missing was exactly one thing, and it is now built:
//! **decoding a pack's `biomes`/`terrains` images, so that blend has a
//! texture to prefer over the flat swatch.** `PackManifest`'s own
//! `.biomes`/`.terrains` (`cartalith-assets/src/manifest.rs`, `pub biomes`/
//! `pub terrains`, keyed by `PACK_BIOME_SLOTS`/`PACK_TERRAIN_SLOTS`) were
//! parsed — for a correct `packSummary`-equivalent and warning count — but
//! [`LoadedPack`] never turned them into pixels, so [`load_pack_from_bytes`]
//! decoded `icons` and `splat` and nothing else and every painted cell took
//! the reference's own no-texture branch.
//!
//! [`decode_ground_family`] below closes it, carrying both families to
//! `render.rs` the way [`SplatChannel`] already was — as
//! [`crate::render::GroundTiles`], borrowed by `RenderCtx::with_ground_tiles`
//! at `lib.rs`'s existing `if let Some(loaded) = self.asset_pack` site.
//!
//! **Nothing about the default render moved**, and that is a property, not
//! an accident: a tile is reachable only through a *painted* cell, painted
//! cells start empty, and this port bundles no pack at all — so the flat
//! swatch is still what every default pixel takes, and
//! `golden_parity_render.rs` never enters the branch.

use std::collections::HashMap;
use std::io::Cursor;

use cartalith_assets::{
    DecodedImage, PACK_BIOME_SLOTS, PACK_TERRAIN_SLOTS, PACK_TEX_SLOTS, PackEntries, PackManifest,
    PlaceIconsRuledOpts, PlacedIcon,
    ScatterRuleTable, autopopulate_scatter_rules, current_scatter_rules,
    finalize_pack_texture_inv_mean, icon_slot_for_item, pick_icon_variant,
    pick_weighted_variant, place_map_icons_ruled, read_pack, sprite_draw_rect,
    trait_badge_layout, trait_sprite_rect,
};

use crate::render::{GroundTile, SplatChannel};

/// ET-E2's colour-managed PNG decode (`EDITING_TOOLS_SCOPE.md` §6). Declared
/// here, by `#[path]`, rather than in `lib.rs` because `pack.rs` is also
/// compiled standalone by two integration-test targets
/// (`tests/pack_compositing.rs`, `tests/pack_trait_badges.rs`) and every pack
/// image decodes through it; `lib.rs` and `asset_bridge.rs` reach it as
/// `crate::pack::colour_import`.
#[path = "colour_import.rs"]
pub mod colour_import;

use colour_import::{ColourReport, decode_png_managed};

/// Decode one pack entry colour-managed (ET-E2) and, when the file carried
/// any colour chunk, record its report under its archive path in `notes`. An
/// untagged entry -- every pack this port writes, and all 17 PNGs of
/// `cartalith-assets/tests/fixtures/reference_pack.zip`, checked 2026-10-11 --
/// leaves no note and decodes to the same bytes `decode_png` gave.
fn decode_entry(path: &str, data: &[u8], notes: &mut Vec<(String, ColourReport)>) -> Option<DecodedImage> {
    let (img, report) = decode_png_managed(data).ok()?;
    if report.tagged {
        notes.push((path.to_string(), report));
    }
    Some(img)
}

/// A real, loaded asset pack — the families this module composites or hands
/// to the overlay, decoded to pixels: `icons`, the six splat channels,
/// `biomes`, `terrains`, (since `OUTSTANDING_WORK.md` §2.5) `structures.trait`
/// and (since AP-3, Ruling CQ Q2, 2026-10-18) the four manual-icon marker
/// families in [`Self::markers`].
///
/// **Corrected by AP-3.** This doc said `structures.settlement`,
/// `structures.poi`, `custom` and `seamarks` were parsed and never decoded
/// "because nothing draws them". AP-3 is what draws them, so they are now
/// decoded into [`Self::markers`]; `textures.parchment` is still the one
/// family parsed and not decoded (see [`Self::splat`]).
pub struct LoadedPack {
    pub manifest: PackManifest,
    /// Slot name (`PACK_ICON_SLOTS` member, e.g. `"mountain"`) → decoded
    /// variants in manifest order.
    pub icons: HashMap<String, Vec<DecodedImage>>,
    /// Slot name (one of `SPLAT_PAINT_SLOTS`) → decoded channel. `parchment`
    /// (`PACK_TEX_SLOTS`' 7th slot) is a paper-base multiply the reference's
    /// own splat block never samples (`SPLAT_PAINT_SLOTS` excludes it), so
    /// it is parsed but not decoded here either.
    pub splat: HashMap<&'static str, SplatChannel>,
    /// Painted-biome ground tiles, **positional**: index `n` is
    /// `PACK_BIOME_SLOTS[n]`, so a painted index `p` reads `biomes[p - 1]`
    /// (`slots.rs`' frozen "slot N here is index N+1 in `CART_BIOMES`").
    /// Always `PACK_BIOME_SLOTS.len()` long, `None` where the pack has no
    /// art — a `HashMap` would push that `- 1` arithmetic into the render
    /// loop, which is where it is easiest to get wrong.
    pub biomes: Vec<Option<GroundTile>>,
    /// Painted-terrain ground tiles, on the same terms —
    /// `PACK_TERRAIN_SLOTS.len()` long, indexed by `p - 1`.
    pub terrains: Vec<Option<GroundTile>>,
    /// Trait-badge art (`structures.trait`) — slot key (a
    /// [`cartalith_assets::PACK_TRAIT_SLOTS`] member) → decoded variants in
    /// manifest order, consumed by [`resolve_trait_badges`] (and through it
    /// by both [`composite_trait_badges`] and `WorldGen::civ_trait_badge_row`,
    /// which hands each variant to GDScript as an `ImageTexture`).
    ///
    /// **An absent key and a key holding an empty `Vec` are different
    /// states, and that is the reason this map is not filtered the way
    /// `icons` above is.** Absent: the pack never declared trait art for
    /// that slot. Empty: it declared some and not one variant decoded — a
    /// missing ZIP entry or a broken PNG. Both draw the same thing (nothing;
    /// the reference's own `if(!arr||!arr.length) return false` at
    /// `_traitSprite`, v2.11 line 15573, does not distinguish them either),
    /// and [`resolve_trait_badges`] returns them as two different
    /// [`TraitArtMiss`] reasons so the surface reporting the miss can tell a
    /// pack author *"you never added port art"* from *"your port art is a
    /// broken PNG"*. Collapsing them here would make that impossible to
    /// recover later.
    ///
    /// **Which surface reports which, as of 2026-09-04.** The *map* reports
    /// neither and must not: `_civDrawTraitBadges` draws the same dark disc
    /// and glyph for both (v2.11 lines 15592-15598), so the two are
    /// deliberately indistinguishable on the pin. The surface that tells them
    /// apart is `WorldGen::pack_trait_art_status`, a per-slot answer for an
    /// asset-library / pack-import readout. `WorldGen::civ_trait_badge_row`
    /// carries the same reason per badge so a caller that wants to highlight
    /// one settlement's missing art does not have to re-derive it.
    pub traits: HashMap<String, Vec<DecodedImage>>,
    /// AP-3 (Ruling CQ Q2): the art a hand-placed or generated map icon of
    /// the `settlement`, `poi`, `seamarks` or `custom` family draws instead
    /// of its placeholder glyph. Keyed by [`marker_art_key`], decoded variants
    /// in manifest order, consumed by [`resolve_marker_sprite`] and, through
    /// `WorldGen::icon_marker_art`, by `map_overlay.gd`.
    ///
    /// **Only slots with at least one decoded variant are present**, unlike
    /// [`Self::traits`]: the map draws the same glyph for "never declared"
    /// and "declared but broken" (the reference's `_structSprite` returns
    /// `false` for both, v2.11 line 15510), and no surface reports the two
    /// apart for these families — the pack report (AP-2) does that from the
    /// library's own pixels. So there is no absent-vs-empty state to keep.
    ///
    /// The `feature` manual family is **not** here: its art is the pack's
    /// `icons` family, already decoded into [`Self::icons`], and AP-3's scope
    /// is the four families with no map path at all (`ASSET_PACK_TOOLS_PROPOSAL.md`
    /// §6). A manual feature icon keeps its glyph; AP-5 owns feature icons.
    pub markers: HashMap<String, Vec<DecodedImage>>,
    /// ET-E2: one colour report per decoded pack image that carried any
    /// colour chunk, keyed by its archive path, in decode order. Empty for an
    /// untagged pack. Read by `WorldGen::asset_pack_colour`.
    pub colour: Vec<(String, ColourReport)>,
}

/// Decode one painted-ground family into the positional table
/// [`LoadedPack::biomes`]/[`LoadedPack::terrains`] describes.
///
/// **No `finalize_pack_texture_inv_mean` here, unlike the splat loop below**,
/// and that is the reference's asymmetry rather than an omission: a splat
/// channel modulates a procedural ramp by `texel/mean`, while a painted tile
/// is blended as true colour (`ASSET_LIBRARY_SCOPE.md` §3 — "dividing out a
/// tile's absolute hue is right for splat and wrong for paint"; the reference
/// says so at its own line 12246). [`GroundTile`] has no `inv` field to fill,
/// so the mistake cannot be made silently.
///
/// A missing slot, a manifest path with no ZIP entry, and a PNG that fails to
/// decode all land on the same `None` — per-slot tolerance, matching the rest
/// of this module and the format's own per-slot fallback rule.
fn decode_ground_family(
    slots: &[&'static str],
    table: &cartalith_assets::OrderedMap<String>,
    entries: &PackEntries,
    notes: &mut Vec<(String, ColourReport)>,
) -> Vec<Option<GroundTile>> {
    slots
        .iter()
        .map(|&slot| {
            let path = table.get(slot)?;
            let img = decode_entry(path, entries.get(path)?, notes)?;
            Some(GroundTile { w: img.w, h: img.h, rgba: img.rgba })
        })
        .collect()
}

/// Read a pack `.zip`'s bytes and decode the two families this milestone
/// composites. A per-image decode failure is skipped, not fatal — matching
/// the reference's own per-slot tolerance (`decodePackImage` throwing for one
/// slot doesn't abort loading the rest of a pack in spirit, though the
/// reference's own `loadAssetPack` is a flat `await` loop; skip-and-continue
/// is the honest Rust equivalent for a `Result`-returning decode instead of
/// letting one bad PNG fail the entire pack).
pub fn load_pack_from_bytes(bytes: Vec<u8>) -> Result<LoadedPack, String> {
    let (manifest, entries) = read_pack(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let mut colour: Vec<(String, ColourReport)> = Vec::new();

    let mut icons: HashMap<String, Vec<DecodedImage>> = HashMap::new();
    for (slot, paths) in manifest.icons.iter() {
        let mut variants = Vec::new();
        for path in paths {
            let Some(data) = entries.get(path) else { continue };
            if let Some(img) = decode_entry(path, data, &mut colour) {
                variants.push(img);
            }
        }
        if !variants.is_empty() {
            icons.insert(slot.to_string(), variants);
        }
    }

    let mut splat: HashMap<&'static str, SplatChannel> = HashMap::new();
    for &slot in PACK_TEX_SLOTS.iter().filter(|&&s| s != "parchment") {
        let Some(path) = manifest.textures.get(slot) else { continue };
        let Some(data) = entries.get(path) else { continue };
        let Some(img) = decode_entry(path, data, &mut colour) else { continue };
        let inv = finalize_pack_texture_inv_mean(img.w, img.h, &img.rgba);
        splat.insert(slot, SplatChannel { w: img.w, h: img.h, rgba: img.rgba, inv });
    }

    let biomes = decode_ground_family(&PACK_BIOME_SLOTS, &manifest.biomes, &entries, &mut colour);
    let terrains = decode_ground_family(&PACK_TERRAIN_SLOTS, &manifest.terrains, &entries, &mut colour);

    // `structures.trait`. Same per-variant tolerance as `icons` above, with
    // one deliberate difference: the slot is inserted **even when no variant
    // decoded**, because [`LoadedPack::traits`] uses absent-vs-empty to carry
    // exactly that distinction. Do not add an `is_empty()` filter here.
    let mut traits: HashMap<String, Vec<DecodedImage>> = HashMap::new();
    for (slot, paths) in manifest.structures.traits.iter() {
        let mut variants = Vec::new();
        for path in paths {
            let Some(data) = entries.get(path) else { continue };
            if let Some(img) = decode_entry(path, data, &mut colour) {
                variants.push(img);
            }
        }
        traits.insert(slot.to_string(), variants);
    }

    // AP-3: the four marker families. Same per-variant tolerance as `icons`;
    // a slot none of whose variants decoded is left out (see
    // `LoadedPack::markers` for why that is safe here and not for traits).
    let mut markers: HashMap<String, Vec<DecodedImage>> = HashMap::new();
    let mut decode_marker = |key: String, paths: &[String], colour: &mut Vec<(String, ColourReport)>| {
        let variants: Vec<DecodedImage> = paths
            .iter()
            .filter_map(|path| decode_entry(path, entries.get(path)?, colour))
            .collect();
        if !variants.is_empty() {
            markers.insert(key, variants);
        }
    };
    for (family, table) in [
        ("settlement", &manifest.structures.settlement),
        ("poi", &manifest.structures.poi),
        ("seamarks", &manifest.seamarks),
    ] {
        for (slot, paths) in table.iter() {
            if let Some(key) = marker_art_key(family, slot, "") {
                decode_marker(key, paths, &mut colour);
            }
        }
    }
    for (set, slots) in manifest.custom.iter() {
        for (slot, paths) in slots.iter() {
            if let Some(key) = marker_art_key("custom", slot, set) {
                decode_marker(key, paths, &mut colour);
            }
        }
    }

    Ok(LoadedPack { manifest, icons, splat, biomes, terrains, traits, markers, colour })
}

// ---------------------------------------------------------------------------
// Marker art for manual icons (AP-3, Ruling CQ Q2)
// ---------------------------------------------------------------------------

/// The [`LoadedPack::markers`] key for one manual icon's art, from the three
/// strings `icon_dict` carries (`family`, `slot`, `set`).
///
/// `None` for a family AP-3 does not draw: `feature` (its art is the pack's
/// `icons` family and AP-5's subject) and anything unrecognised. `set` is read
/// only for `custom`, whose art is two-level (`custom[set][slot]`, the
/// reference's `_customSprite`, v2.11 line 15607); the other three ignore it.
///
/// The separator is U+001F (unit separator), not `/`: a custom set name is
/// free text and may contain `/`, and `"a/b" + "c"` must never collide with
/// `"a" + "b/c"`. U+001F cannot be typed into a set name field.
pub fn marker_art_key(family: &str, slot: &str, set: &str) -> Option<String> {
    match family {
        "settlement" | "poi" | "seamarks" => Some(format!("{family}\u{1f}{slot}")),
        "custom" => Some(format!("custom\u{1f}{set}\u{1f}{slot}")),
        _ => None,
    }
}

/// One manual icon's resolved pack art: which [`LoadedPack::markers`] entry
/// and variant to draw, and that variant's aspect ratio for the caller's own
/// rectangle.
#[derive(Debug, Clone, PartialEq)]
pub struct MarkerSprite {
    /// The [`marker_art_key`] the variant lives under.
    pub key: String,
    /// Index into `LoadedPack::markers[key]`.
    pub variant: usize,
    /// `w / max(1, h)` of that variant: the reference's
    /// `dw = dh * (v.w / Math.max(1, v.h))` (`_structSprite`, v2.11 line 15513).
    pub aspect: f64,
}

/// `_structSprite` / `_customSprite` (reference v2.11 lines 15508 and 15607)
/// reduced to their decision: does the loaded pack have art for this icon,
/// and which variant.
///
/// `x`/`y` are the icon's continuous grid coordinates (`icon_dict`'s `x`/`y`).
/// The variant hash is `pickIconVariant(px|0, py|0, seed, n)` at the
/// reference's own `civToScreen` without a view rect, `[gx + 0.5, gy + 0.5]`
/// (v2.11 line 15874), so the choice is the reference's fit-view choice and
/// does not change as the camera moves. **Divergence, disclosed:** under its
/// LOD view rect the reference hashes the *view* position instead, so its
/// variant flickers with pan and zoom; this port does not reproduce that.
///
/// The reference's `_customSprite` and `_featureSprite` call
/// `pickWeightedVariant` with the asset's scatter-rule weights. Those weights
/// live in the project library's rule table, not in a pack (Ruling F), and a
/// [`LoadedPack`] carries none — [`composite_map_icons`] autopopulates its
/// rules for the same reason — so with no weights `pickWeightedVariant` is
/// `pickIconVariant` exactly (`pick_weighted_variant`'s own doc), and that is
/// what is called.
///
/// Must never invent art: `None` when the family is not one of the four,
/// when the pack has no decoded variant for the key, or for a degenerate
/// coordinate (a non-finite `x`/`y`, which no placement writes).
pub fn resolve_marker_sprite(family: &str, slot: &str, set: &str, x: f64, y: f64, seed: i32, pack: &LoadedPack) -> Option<MarkerSprite> {
    if !x.is_finite() || !y.is_finite() {
        return None;
    }
    let key = marker_art_key(family, slot, set)?;
    let variants = pack.markers.get(&key)?;
    // `px|0`: JS ToInt32 truncation. Grid coordinates are non-negative and far
    // below 2^31, so Rust's saturating `as i32` (toward zero) agrees with it.
    let variant = pick_icon_variant((x + 0.5) as i32, (y + 0.5) as i32, seed, variants.len());
    let img = variants.get(variant)?;
    let aspect = img.w as f64 / cartalith_jsmath::js_max(1.0, img.h as f64);
    Some(MarkerSprite { key, variant, aspect })
}

/// Long-side sizes, in pixels, a marker sprite is pre-filtered to before the
/// overlay draws it. The overlay draws with plain bilinear filtering and no
/// mipmaps, so a 256² sprite drawn at 11 px would be sampled about one texel
/// in 23 and alias. Instead the smallest raster at or above the sprite's
/// ON-SCREEN size is drawn and shrunk by the GPU by at most the ratio between
/// adjacent entries (≤ 1.5×, where bilinear still averages every texel).
///
/// The same ladder as `map_overlay.gd`'s `LANDMARK_GLYPH_RASTERS`, for the
/// same reason (a labelled judgement there: adjacent buckets ≤ 1.5× apart),
/// topped at 256, the sprite families' bake size (`Family::size`). A sprite
/// whose own long side is smaller than a bucket is never enlarged here: the
/// bucket is capped at the source's long side.
pub const MARKER_RASTERS: [u32; 10] = [12, 16, 24, 32, 48, 64, 96, 128, 192, 256];

/// The [`MARKER_RASTERS`] entry to draw a sprite of long side `native` at
/// `screen_px` on-screen pixels: the smallest entry `>= screen_px`, capped at
/// `native` (never upsampled here; the GPU enlarges past it). A non-finite or
/// non-positive `screen_px` reads as the smallest entry.
pub fn marker_raster_for(screen_px: f64, native: u32) -> u32 {
    let want = if screen_px.is_finite() && screen_px > 0.0 { screen_px } else { 0.0 };
    let bucket = MARKER_RASTERS.iter().copied().find(|&b| f64::from(b) >= want).unwrap_or(MARKER_RASTERS[MARKER_RASTERS.len() - 1]);
    bucket.min(native.max(1))
}

/// Shrink `img` so its long side is `long_side` pixels, by exact area
/// averaging in **premultiplied** alpha, then un-premultiply.
///
/// Why this filter: each output pixel is the coverage-weighted mean of every
/// source texel under it, so thin ink keeps its mass instead of being point-
/// sampled away (`LINEART_METHODS_RESEARCH_2026-10-18.md` measured a single
/// bilinear sample keeping 0.31-0.45 of a line's ink at map scale, against
/// 0.55-0.81 for an area filter). Premultiplying first is what keeps a
/// transparent texel's RGB (usually black) from bleeding a dark fringe into
/// the sprite's edge (Ruling CR names both points).
///
/// Returns a clone when `long_side` is not smaller than the source's long
/// side; never enlarges. The short side is `round(short * long_side / long)`,
/// floored at 1. Integer output by round-half-up of the mean.
pub fn area_downscale(img: &DecodedImage, long_side: u32) -> DecodedImage {
    let (sw, sh) = (img.w.max(1), img.h.max(1));
    let long = sw.max(sh);
    if long_side == 0 || long_side >= long {
        return img.clone();
    }
    let k = f64::from(long_side) / f64::from(long);
    let dw = ((f64::from(sw) * k).round() as u32).max(1);
    let dh = ((f64::from(sh) * k).round() as u32).max(1);
    // The source span each output pixel covers along one axis, as
    // (first texel, weight) runs: texel t covers [t, t+1), output pixel o
    // covers [o*s, (o+1)*s) with s = src/dst; the weight is the overlap.
    let spans = |src: u32, dst: u32| -> Vec<Vec<(usize, f64)>> {
        let s = f64::from(src) / f64::from(dst);
        (0..dst)
            .map(|o| {
                let (a, b) = (f64::from(o) * s, (f64::from(o) + 1.0) * s);
                let (t0, t1) = (a.floor() as u32, (b.ceil() as u32).min(src));
                (t0..t1)
                    .map(|t| (t as usize, (b.min(f64::from(t) + 1.0) - a.max(f64::from(t))).max(0.0)))
                    .filter(|&(_, w)| w > 0.0)
                    .collect()
            })
            .collect()
    };
    let (xs, ys) = (spans(sw, dw), spans(sh, dh));
    let mut out = vec![0u8; (dw * dh * 4) as usize];
    for (oy, yrun) in ys.iter().enumerate() {
        for (ox, xrun) in xs.iter().enumerate() {
            let mut acc = [0.0f64; 4]; // premultiplied r, g, b, and alpha
            let mut area = 0.0;
            for &(ty, wy) in yrun {
                for &(tx, wx) in xrun {
                    let w = wx * wy;
                    let o = (ty * sw as usize + tx) * 4;
                    let a = f64::from(img.rgba[o + 3]);
                    for (c, slot) in acc.iter_mut().take(3).enumerate() {
                        *slot += f64::from(img.rgba[o + c]) * a * w;
                    }
                    acc[3] += a * w;
                    area += w;
                }
            }
            let d = (oy * dw as usize + ox) * 4;
            let alpha = if area > 0.0 { acc[3] / area } else { 0.0 };
            for c in 0..3 {
                // Un-premultiply: sum(c*a*w) / sum(a*w). A fully transparent
                // output pixel keeps RGB 0, the decoder's own convention.
                out[d + c] = if acc[3] > 0.0 { (acc[c] / acc[3] + 0.5).floor().clamp(0.0, 255.0) as u8 } else { 0 };
            }
            out[d + 3] = (alpha + 0.5).floor().clamp(0.0, 255.0) as u8;
        }
    }
    DecodedImage::new(dw, dh, out).expect("dw*dh*4 bytes by construction")
}

// ---------------------------------------------------------------------------
// Sprite compositing (`placeMapIconsRuled` -> `drawMapIcons`)
// ---------------------------------------------------------------------------

/// `slopeAt` (render.rs's own copy, non-wrapping edge-clamp case only) —
/// duplicated rather than made `pub(crate)` on `RenderCtx` so this module
/// stays a plain function of its own inputs, not a second caller reaching
/// into the renderer's internals. World-wrap is not reproduced here (the
/// wetland mask is an approximation already, see [`build_biome_and_wetland`]).
fn slope_at(field: &[f32], gw: usize, gh: usize, x: usize, y: usize) -> f64 {
    let idx = |xx: usize, yy: usize| yy * gw + xx;
    let l = field[idx(x.saturating_sub(1), y)] as f64;
    let r = field[idx((x + 1).min(gw - 1), y)] as f64;
    let u = field[idx(x, y.saturating_sub(1))] as f64;
    let d = field[idx(x, (y + 1).min(gh - 1))] as f64;
    ((r - l) * 0.5).hypot((d - u) * 0.5)
}

/// `buildBiomeRaster` + `buildWetlandMask` (reference lines 6798-6849),
/// derived purely from the already-generated height/temperature/rainfall
/// fields at render time — no new world-generation data, the same category
/// of presentation-side computation `render.rs`'s own `material_weights`
/// already is.
///
/// One honest simplification from the reference: `buildBiomeRaster` splits
/// water into ocean (0) vs. lake (13) via `currentWaterBodies()`, a
/// flood-fill classifier this port has never built. Every water cell here is
/// simply `BIOME_OCEAN`. This only matters to a scatter rule that
/// specifically targets `biomes:[13]` (lake shoreline dressing) — none of
/// the ten frozen [`cartalith_assets::PACK_ICON_SLOTS`] presets do.
fn build_biome_and_wetland(field: &[f32], temperature: &[f32], rainfall: &[f32], gw: usize, gh: usize, sea_level: f64) -> (Vec<u8>, Vec<u8>) {
    let n = gw * gh;
    let mut biome = vec![0u8; n];
    let mut wetland = vec![0u8; n];
    let denom = (1.0 - sea_level).max(1e-6);
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let h = field[i] as f64;
            if h < sea_level {
                continue; // BIOME_OCEAN == 0, already the vec's fill value
            }
            biome[i] = cartalith_civ::classify_biome(temperature[i] as f64, rainfall[i] as f64);
            let r = (h - sea_level) / denom;
            let sn = slope_at(field, gw, gh, x, y) * gw as f64;
            let m = rainfall[i] as f64;
            if m > 0.62 && r < 0.18 && sn < 1.0 {
                wetland[i] = 1;
            }
        }
    }
    (biome, wetland)
}

/// Bilinear-sample a decoded sprite at normalized `(u, v)` (`[0, 1)` each) —
/// the port's stand-in for `ctx.drawImage`'s resample kernel, same
/// "matches itself, not the reference" carve-out `cartalith-assets::raster`
/// already documents for `render_item`.
fn sample_bilinear(img: &DecodedImage, u: f64, v: f64) -> [f64; 4] {
    let x = (u * img.w as f64 - 0.5).max(0.0);
    let y = (v * img.h as f64 - 0.5).max(0.0);
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(img.w - 1);
    let y1 = (y0 + 1).min(img.h - 1);
    let (fx, fy) = (x - x0 as f64, y - y0 as f64);
    let px = |xx: u32, yy: u32| {
        let o = ((yy * img.w + xx) * 4) as usize;
        [img.rgba[o] as f64, img.rgba[o + 1] as f64, img.rgba[o + 2] as f64, img.rgba[o + 3] as f64]
    };
    let (p00, p10, p01, p11) = (px(x0, y0), px(x1, y0), px(x0, y1), px(x1, y1));
    let mut out = [0.0; 4];
    for c in 0..4 {
        let top = p00[c] * (1.0 - fx) + p10[c] * fx;
        let bot = p01[c] * (1.0 - fx) + p11[c] * fx;
        out[c] = top * (1.0 - fy) + bot * fy;
    }
    out
}

/// A tiny software rasterizer over the RGB8 (no destination alpha) map
/// buffer — just enough primitives for [`blit_sprite`] and
/// [`draw_icon_glyph`], bundling `bytes`/`gw`/`gh` once instead of
/// threading them through every fill/stroke call (clippy's own
/// `too_many_arguments` flagged the unbundled version).
struct Canvas<'b> {
    bytes: &'b mut [u8],
    gw: usize,
    gh: usize,
}

impl Canvas<'_> {
    /// Straight-alpha source-over. Bounds-checked per pixel rather than
    /// pre-clipped, since a sprite or glyph near the map edge is
    /// legitimately partly off-canvas.
    fn blend_px(&mut self, x: i64, y: i64, rgb: (f64, f64, f64), a: f64) {
        if a <= 0.0 || x < 0 || y < 0 || x as usize >= self.gw || y as usize >= self.gh {
            return;
        }
        let di = (y as usize * self.gw + x as usize) * 3;
        self.bytes[di] = (rgb.0 * a + self.bytes[di] as f64 * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
        self.bytes[di + 1] = (rgb.1 * a + self.bytes[di + 1] as f64 * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
        self.bytes[di + 2] = (rgb.2 * a + self.bytes[di + 2] as f64 * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
    }

    /// One real placed sprite, at whatever destination rectangle the caller
    /// supplies.
    ///
    /// **Corrected 2026-09-04:** this said "bottom-anchored via
    /// `sprite_draw_rect`'s destination rectangle (`spriteDrawRect`,
    /// milestone 4)", which stopped being true three lines above it when a
    /// second caller landed. Trait badges are **centre-anchored** --
    /// `trait_sprite_rect` ports `_traitSprite`'s `drawImage(v.bmp,
    /// px - dw/2, py - dh/2, dw, dh)` (v2.11:15576) -- so this function is
    /// anchor-agnostic and the anchoring belongs to whichever `*_rect` helper
    /// the caller chose. Missed by the lane that added the second caller, in
    /// the file it was editing.
    fn blit_sprite(&mut self, img: &DecodedImage, rect: cartalith_assets::SpriteRect) {
        if rect.dw <= 0.0 || rect.dh <= 0.0 {
            return;
        }
        let x0 = rect.dx.floor().max(0.0) as i64;
        let x1 = (rect.dx + rect.dw).ceil().min(self.gw as f64) as i64;
        let y0 = rect.dy.floor().max(0.0) as i64;
        let y1 = (rect.dy + rect.dh).ceil().min(self.gh as f64) as i64;
        for py in y0..y1 {
            for px in x0..x1 {
                let u = ((px as f64 + 0.5) - rect.dx) / rect.dw;
                let v = ((py as f64 + 0.5) - rect.dy) / rect.dh;
                if !(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v) {
                    continue;
                }
                let s = sample_bilinear(img, u, v);
                self.blend_px(px, py, (s[0], s[1], s[2]), s[3] / 255.0);
            }
        }
    }

    /// Fills the triangle `p0 p1 p2` (pixel-centre sampling, either winding, inclusive
    /// edges) with `rgb` at alpha `a` through [`Canvas::blend_px`]. A degenerate
    /// triangle (|area x 2| < 1e-9, a numerical-zero judgement) paints nothing.
    fn fill_triangle(&mut self, p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), rgb: (f64, f64, f64), a: f64) {
        let minx = p0.0.min(p1.0).min(p2.0).floor().max(0.0) as i64;
        let maxx = p0.0.max(p1.0).max(p2.0).ceil().min(self.gw as f64) as i64;
        let miny = p0.1.min(p1.1).min(p2.1).floor().max(0.0) as i64;
        let maxy = p0.1.max(p1.1).max(p2.1).ceil().min(self.gh as f64) as i64;
        let edge = |a: (f64, f64), b: (f64, f64), p: (f64, f64)| (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0);
        if edge(p0, p1, p2).abs() < 1e-9 {
            return;
        }
        for y in miny..maxy {
            for x in minx..maxx {
                let p = (x as f64 + 0.5, y as f64 + 0.5);
                let (w0, w1, w2) = (edge(p1, p2, p), edge(p2, p0, p), edge(p0, p1, p));
                if (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0) {
                    self.blend_px(x, y, rgb, a);
                }
            }
        }
    }

    /// Fills the axis-aligned ellipse centred `(cx, cy)` with radii `rx`, `ry` (pixel-centre
    /// sample inside `dx^2 + dy^2 <= 1`). A non-positive radius paints nothing.
    fn fill_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, rgb: (f64, f64, f64), a: f64) {
        if rx <= 0.0 || ry <= 0.0 {
            return;
        }
        let minx = (cx - rx).floor().max(0.0) as i64;
        let maxx = (cx + rx).ceil().min(self.gw as f64) as i64;
        let miny = (cy - ry).floor().max(0.0) as i64;
        let maxy = (cy + ry).ceil().min(self.gh as f64) as i64;
        for y in miny..maxy {
            for x in minx..maxx {
                let dx = (x as f64 + 0.5 - cx) / rx;
                let dy = (y as f64 + 0.5 - cy) / ry;
                if dx * dx + dy * dy <= 1.0 {
                    self.blend_px(x, y, rgb, a);
                }
            }
        }
    }

    /// Fills the axis-aligned rectangle `(x, y, w, h)` with `rgb` at alpha `a`; every
    /// pixel touched by the rectangle is blended, bounds-checked by `blend_px`.
    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64, rgb: (f64, f64, f64), a: f64) {
        let x0 = x.floor().max(0.0) as i64;
        let x1 = (x + w).ceil().min(self.gw as f64) as i64;
        let y0 = y.floor().max(0.0) as i64;
        let y1 = (y + h).ceil().min(self.gh as f64) as i64;
        for yy in y0..y1 {
            for xx in x0..x1 {
                self.blend_px(xx, yy, rgb, a);
            }
        }
    }

    /// A thick stroke, approximated by stamping small discs along the
    /// segment — close enough at this port's icon scale (a handful of
    /// pixels) that a real polyline-with-joins renderer would be
    /// gold-plating.
    fn draw_line(&mut self, p0: (f64, f64), p1: (f64, f64), rgb: (f64, f64, f64), a: f64, width: f64) {
        let len = ((p1.0 - p0.0).powi(2) + (p1.1 - p0.1).powi(2)).sqrt();
        let steps = (len * 2.0).ceil().max(1.0) as usize;
        let r = (width * 0.5).max(0.4);
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            self.fill_ellipse(p0.0 + (p1.0 - p0.0) * t, p0.1 + (p1.1 - p0.1) * t, r, r, rgb, a);
        }
    }
}

/// `drawIconGlyph` (reference line 7315) — the per-slot procedural fallback
/// for a slot the loaded pack (if any) has no real sprite for. Faithful to
/// the reference's silhouettes and colours; two purely-decorative variants
/// are dropped for scope (documented, not silently lost): the arid jagged
/// hill outline and the cold-mountain snow-cap overlay, both of which the
/// reference itself describes as "procedural-fallback variety only" — the
/// base silhouette every slot draws is unconditional and is what's ported.
fn draw_icon_glyph(canvas: &mut Canvas, slot: &str, item: &PlacedIcon, base: f64) {
    let (x, y) = (item.x as f64, item.y as f64);
    match slot {
        "mountain" => {
            let s = base * item.s;
            canvas.fill_triangle((x - s, y), (x, y - s * 1.35), (x + s, y), (238.0, 232.0, 218.0), 0.85);
            canvas.fill_triangle((x, y - s * 1.35), (x + s, y), (x + s * 0.35, y), (90.0, 74.0, 52.0), 0.30);
        }
        "hill" => {
            let s = base * 0.55 * item.s;
            canvas.draw_line((x - s, y), (x, y - s * 1.1), (60.0, 48.0, 30.0), 0.55, 1.0);
            canvas.draw_line((x, y - s * 1.1), (x + s, y), (60.0, 48.0, 30.0), 0.55, 1.0);
        }
        "tree_conifer" => {
            let s = base * 0.5 * item.s;
            canvas.fill_triangle((x, y - s * 1.6), (x + s * 0.6, y), (x - s * 0.6, y), (44.0, 84.0, 52.0), 0.70);
        }
        "tree_rainforest" => {
            let s = base * 0.5 * item.s;
            canvas.fill_ellipse(x - s * 0.32, y - s * 0.75, s * 0.55, s * 0.55, (30.0, 90.0, 48.0), 0.68);
            canvas.fill_ellipse(x + s * 0.32, y - s * 0.85, s * 0.6, s * 0.6, (30.0, 90.0, 48.0), 0.68);
            canvas.draw_line((x, y), (x, y - s * 0.5), (20.0, 60.0, 32.0), 0.5, 0.8);
        }
        "tree_savanna" => {
            let s = base * 0.5 * item.s;
            canvas.draw_line((x, y), (x, y - s * 0.9), (70.0, 54.0, 30.0), 0.6, 0.8);
            canvas.fill_ellipse(x, y - s * 1.05, s * 0.9, s * 0.28, (120.0, 124.0, 70.0), 0.6);
        }
        "tree_wetland" => {
            let s = base * 0.5 * item.s;
            for dx in [-0.35, 0.0, 0.35] {
                canvas.draw_line((x + dx * s, y), (x + dx * s, y - s * 0.7), (50.0, 64.0, 44.0), 0.55, 0.8);
            }
            canvas.fill_ellipse(x, y - s * 0.85, s * 0.65, s * 0.65, (52.0, 88.0, 64.0), 0.6);
        }
        "tree_broadleaf" => {
            let s = base * 0.5 * item.s;
            canvas.fill_ellipse(x, y - s * 0.8, s * 0.7, s * 0.7, (58.0, 104.0, 58.0), 0.65);
            canvas.draw_line((x, y), (x, y - s * 0.5), (40.0, 60.0, 36.0), 0.5, 0.8);
        }
        "cactus" => {
            let s = base * 0.5 * item.s;
            let w = s * 0.22;
            let rgb = (70.0, 110.0, 74.0);
            canvas.fill_rect(x - w * 0.5, y - s * 1.1, w, s * 1.1, rgb, 0.68);
            canvas.fill_rect(x - w * 1.6, y - s * 0.65, w, s * 0.5, rgb, 0.68);
            canvas.fill_rect(x + w * 0.6, y - s * 0.85, w, s * 0.55, rgb, 0.68);
        }
        "boulder" => {
            let s = base * 0.5 * item.s;
            let rgb = (120.0, 118.0, 112.0);
            canvas.fill_ellipse(x - s * 0.25, y - s * 0.18, s * 0.42, s * 0.3, rgb, 0.65);
            canvas.fill_ellipse(x + s * 0.28, y - s * 0.12, s * 0.32, s * 0.24, rgb, 0.65);
        }
        // shrub — also the honest catch-all for a custom asset with no real
        // sprite, matching the reference's own comment at this exact branch.
        _ => {
            let s = base * 0.5 * item.s;
            canvas.fill_ellipse(x, y - s * 0.28, s * 0.4, s * 0.4, (96.0, 110.0, 58.0), 0.6);
        }
    }
}

/// `drawMapIcons` (reference line 7366): place (`placeMapIconsRuled`) and
/// composite every scattered feature icon a loaded pack's scatter-rule table
/// produces, real sprite art where the pack has it, the procedural glyph
/// fallback everywhere else — one Y-sorted painter's pass over the lot, same
/// as the reference (v1.26, "a mountain no longer always paints over a tree
/// standing in front of it").
///
/// A no-op whenever [`current_scatter_rules`] returns `None` — no rules
/// configured (a pack with no `icons` slots at all, or none loaded) is
/// exactly the condition that keeps a pack-less render bit-identical, the
/// same contract `cartalith_assets::current_scatter_rules`'s own doc comment
/// names for the reference's legacy fallback.
///
/// Since AP-5 `build_color_texture` calls the two halves itself (it keeps the
/// placements for the overlay), so the library build no longer calls this
/// function; it stays as the one-call baked path the integration tests
/// (`pack_compositing.rs`, `nonsquare.rs`, `feature_icon_overlay.rs`) drive.
#[allow(clippy::too_many_arguments, dead_code)]
pub fn composite_map_icons(bytes: &mut [u8], field: &[f32], temperature: &[f32], rainfall: &[f32], gw: usize, gh: usize, sea_level: f64, seed: i32, pack: &LoadedPack) {
    // AP-5 split this function into its decision ([`place_feature_icons`])
    // and its drawing ([`composite_placed_features`]) so the overlay path can
    // take the decisions without the drawing. With `sprites: true` the two
    // halves run exactly the statements this body ran before the split, in
    // the same order, so this path's bytes did not move (identity by control
    // flow; pinned by `tests/feature_icon_overlay.rs`).
    let placed = place_feature_icons(field, temperature, rainfall, gw, gh, sea_level, seed, pack);
    composite_placed_features(bytes, gw, gh, &placed, pack, true);
}

/// One scattered feature icon after every decision `drawMapIcons` makes and
/// before any pixel is drawn (AP-5): the placed item, its slot, and either
/// the pack sprite it draws (which variant, at which grid-space rectangle) or
/// the procedural glyph fallback.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedFeature {
    /// `place_map_icons_ruled`'s item, unchanged.
    pub item: PlacedIcon,
    /// `icon_slot_for_item(item)`.
    pub slot: String,
    /// What it draws.
    pub art: FeatureArt,
}

/// What a [`PlacedFeature`] draws.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FeatureArt {
    /// `LoadedPack::icons[slot][variant]`, drawn into `rect` (grid pixels,
    /// continuous: pixel `i` spans `[i, i + 1)`), bottom-anchored by
    /// `sprite_draw_rect`.
    Sprite { variant: usize, rect: cartalith_assets::SpriteRect },
    /// The slot has no decoded variant: [`draw_icon_glyph`]'s silhouette.
    Glyph,
}

/// The placement and per-item art decisions of [`composite_map_icons`]
/// (`placeMapIconsRuled` + `drawMapIcons`' variant pick and `spriteDrawRect`),
/// in painter's order (sorted by `y`, as `drawMapIcons` sorts), drawing
/// nothing.
///
/// Empty whenever [`composite_map_icons`] would be a no-op: no scatter rule
/// (`current_scatter_rules` is `None`) or an empty grid. Must stay the one
/// definition of these decisions: the baked path and AP-5's overlay both
/// read it, so they cannot disagree about where an icon is.
#[allow(clippy::too_many_arguments)]
pub fn place_feature_icons(field: &[f32], temperature: &[f32], rainfall: &[f32], gw: usize, gh: usize, sea_level: f64, seed: i32, pack: &LoadedPack) -> Vec<PlacedFeature> {
    let mut table = ScatterRuleTable::default();
    autopopulate_scatter_rules(&mut table, &pack.manifest);
    let Some(rules) = current_scatter_rules(&table) else {
        return Vec::new();
    };
    if gw == 0 || gh == 0 {
        return Vec::new();
    }

    let (biome, wetland) = build_biome_and_wetland(field, temperature, rainfall, gw, gh, sea_level);
    let fld64: Vec<f64> = field.iter().map(|&v| v as f64).collect();
    let mut opts = PlaceIconsRuledOpts::new(gw, &rules);
    opts.sea = sea_level;
    opts.seed = seed;
    opts.wetland_mask = Some(&wetland);

    let mut items = place_map_icons_ruled(&fld64, Some(&biome), gw, gh, &opts);
    // `drawMapIcons`' own `.sort((a,b)=>a.y-b.y)` — one Y-sorted painter's
    // pass so a mountain doesn't always paint over a tree standing in front
    // of it (v1.26's own fix, ported here because a two-item scatter result
    // deserves the same painter's-order discipline a hundred-item one does).
    items.sort_by_key(|it| it.y);

    let base = feature_icon_base(gw);
    items
        .into_iter()
        .map(|item| {
            let slot = icon_slot_for_item(&item);
            let art = match pack.icons.get(&slot) {
                Some(variants) if !variants.is_empty() => {
                    let weights = table.get(&slot).and_then(|r| r.variant_weights.as_deref());
                    let variant = pick_weighted_variant(item.x, item.y, seed, variants.len(), weights);
                    let img = &variants[variant];
                    let rect = sprite_draw_rect(item.x as f64, item.y as f64, item.s, base, img.w as f64, img.h as f64);
                    FeatureArt::Sprite { variant, rect }
                }
                _ => FeatureArt::Glyph,
            };
            PlacedFeature { item, slot, art }
        })
        .collect()
}

/// `drawMapIcons`' `base` for a grid `gw` cells wide: `max(gw / 110, 3.5)`
/// grid pixels (the reference's own expression, as `composite_map_icons`
/// always used it).
pub fn feature_icon_base(gw: usize) -> f64 {
    (gw as f64 / 110.0).max(3.5)
}

/// Draw `placed` into the RGB8 map buffer in its order: each sprite with
/// [`Canvas::blit_sprite`] (only when `sprites` is true) and each glyph
/// fallback with [`draw_icon_glyph`] (always).
///
/// `sprites: true` is the baked path, [`composite_map_icons`]. `sprites:
/// false` is the base raster under AP-5's overlay switch: the sprites are
/// drawn by `map_overlay.gd` at screen resolution instead, and only the
/// glyph fallbacks (a slot whose every variant failed to decode) stay baked.
/// **Disclosed consequence:** with the switch on, a baked glyph sits under
/// every overlay sprite, so where a glyph and a sprite overlap the painter's
/// order between those two is lost. Glyphs appear only for undecodable art.
pub fn composite_placed_features(bytes: &mut [u8], gw: usize, gh: usize, placed: &[PlacedFeature], pack: &LoadedPack, sprites: bool) {
    if gw == 0 || gh == 0 {
        return;
    }
    let base = feature_icon_base(gw);
    let mut canvas = Canvas { bytes, gw, gh };
    for p in placed {
        match p.art {
            FeatureArt::Sprite { variant, rect } => {
                if sprites && let Some(img) = pack.icons.get(&p.slot).and_then(|v| v.get(variant)) {
                    canvas.blit_sprite(img, rect);
                }
            }
            FeatureArt::Glyph => draw_icon_glyph(&mut canvas, &p.slot, &p.item, base),
        }
    }
}

// ---------------------------------------------------------------------------
// AP-5: feature-icon textures for the screen-space overlay
// ---------------------------------------------------------------------------

/// How far, in 8-bit levels per channel, a pixel's RGB may sit from the ink
/// and still count as the ink for [`one_ink_colour`]. **Measured source of
/// the margin:** AP-1 found `render_item`'s bake moves RGB by one level on
/// every soft-alpha pixel (`ASSET_PACK_TOOLS_PROPOSAL.md` §10, AP-1, "the bake
/// is not a fixed point"), so an AP-10 variant exported into a pack is one
/// ink within 1 level, not exactly; 2 is that plus one level of margin
/// (labelled judgement).
pub const ONE_INK_TOLERANCE: u8 = 2;

/// The sprite's single ink colour when every pixel with alpha > 0 has the
/// same RGB (within [`ONE_INK_TOLERANCE`]), else `None`.
///
/// **This is how AP-5 finds a line-art variant** (`AP-10` left no flag or
/// save key, and a pack carries none, Ruling F). AP-10's output is exactly
/// this shape ("one ink colour where alpha > 0", `convert_line_art`), so
/// every line-art variant qualifies wherever it travelled; so does any other
/// single-colour sprite (a flat silhouette), for which a distance field is
/// equally exact, because such a sprite is wholly described by its alpha.
/// The ink is the RGB of the first pixel with the image's highest alpha.
/// `None` for a fully transparent image. Pixels with alpha 0 are ignored
/// (AP-10 writes black there; other tools write anything).
pub fn one_ink_colour(img: &DecodedImage) -> Option<[u8; 3]> {
    let px = img.rgba.chunks_exact(4);
    let max_a = px.clone().map(|p| p[3]).max()?;
    if max_a == 0 {
        return None;
    }
    let ink = px.clone().find(|p| p[3] == max_a).map(|p| [p[0], p[1], p[2]])?;
    let close = |a: u8, b: u8| a.abs_diff(b) <= ONE_INK_TOLERANCE;
    px.filter(|p| p[3] > 0).all(|p| close(p[0], ink[0]) && close(p[1], ink[1]) && close(p[2], ink[2])).then_some(ink)
}

/// One feature sprite ready for the GPU: an RGBA8 image with its whole
/// mipmap chain, in Godot's layout (level `i` is `max(1, w >> i) × max(1, h
/// >> i)`, down to 1 × 1, levels back to back), plus what the overlay's
/// shader needs to read it.
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureTexture {
    pub w: u32,
    pub h: u32,
    /// Every mip level, Godot's order and sizes.
    pub data: Vec<u8>,
    /// `true`: the signed-distance path (one-ink art); RGB is the ink and A
    /// the quantised distance (`lineart::Sdf`'s 128-is-the-edge code, in
    /// that level's own texels). `false`: premultiplied RGBA, area-filtered
    /// per level.
    pub sdf: bool,
    /// The part of the texture the sprite occupies, as a fraction of each
    /// side (`1, 1` except for a non-square sprite's distance field, which
    /// `SdfPyramid` lays out in the top-left of a square).
    pub extent: (f64, f64),
    /// Distance field only: the sprite's 2nd-percentile stroke width in
    /// TOP-level texels (`SdfPyramid::stroke_p2` rescaled), which the
    /// shader's 1 px stroke floor reads. 0 for the raster path.
    pub stroke_top: f64,
    /// Distance field only: the deepest mip level that holds a real
    /// distance level (`SdfPyramid::levels.len() - 1`); the levels below it
    /// are filler Godot requires and the shader never samples. 0 for the
    /// raster path.
    pub max_lod: u32,
}

/// Godot's mip-chain sizes for a `w × h` image: `(max(1, w >> i), max(1, h
/// >> i))` from `i = 0` until both are 1 (`Image::get_image_mipmap_size`).
pub fn mip_sizes(w: u32, h: u32) -> Vec<(u32, u32)> {
    let mut out = vec![(w.max(1), h.max(1))];
    let mut i = 0;
    while out[i] != (1, 1) {
        i += 1;
        out.push(((w >> i).max(1), (h >> i).max(1)));
    }
    out
}

/// Area-average `img` to exactly `dw × dh` in premultiplied alpha: each
/// output texel is the coverage-weighted mean of every source texel under
/// it (the same exact spans as [`area_downscale`]), written premultiplied,
/// rounded half up. Used for every level of the raster path's mip chain,
/// each straight from the full-size source rather than cascaded, so no level
/// carries an earlier level's rounding.
pub fn area_resample_premultiplied(img: &DecodedImage, dw: u32, dh: u32) -> Vec<u8> {
    let (sw, sh) = (img.w.max(1), img.h.max(1));
    let (dw, dh) = (dw.max(1), dh.max(1));
    let spans = |src: u32, dst: u32| -> Vec<Vec<(usize, f64)>> {
        let s = f64::from(src) / f64::from(dst);
        (0..dst)
            .map(|o| {
                let (a, b) = (f64::from(o) * s, (f64::from(o) + 1.0) * s);
                let (t0, t1) = (a.floor() as u32, (b.ceil() as u32).min(src));
                (t0..t1)
                    .map(|t| (t as usize, (b.min(f64::from(t) + 1.0) - a.max(f64::from(t))).max(0.0)))
                    .filter(|&(_, w)| w > 0.0)
                    .collect()
            })
            .collect()
    };
    let (xs, ys) = (spans(sw, dw), spans(sh, dh));
    let mut out = vec![0u8; (dw * dh * 4) as usize];
    for (oy, yrun) in ys.iter().enumerate() {
        for (ox, xrun) in xs.iter().enumerate() {
            let mut acc = [0.0f64; 4];
            let mut area = 0.0;
            for &(ty, wy) in yrun {
                for &(tx, wx) in xrun {
                    let w = wx * wy;
                    let o = (ty * sw as usize + tx) * 4;
                    let a = f64::from(img.rgba[o + 3]);
                    for (c, slot) in acc.iter_mut().take(3).enumerate() {
                        *slot += f64::from(img.rgba[o + c]) * a / 255.0 * w;
                    }
                    acc[3] += a * w;
                    area += w;
                }
            }
            let d = (oy * dw as usize + ox) * 4;
            for c in 0..4 {
                let v = if area > 0.0 { acc[c] / area } else { 0.0 };
                out[d + c] = (v + 0.5).floor().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}

/// The raster path's texture: `img` premultiplied and area-filtered at every
/// level of Godot's mip chain.
///
/// Why premultiplied: the overlay's shader filters it bilinearly (and
/// between levels) and divides the colour back out, so a transparent texel's
/// RGB never bleeds a dark fringe into the edge (Ruling CR's second point,
/// AP-3's `area_downscale` reasoning). Why every level by area: a sprite
/// drawn at 24 px from a 256² source reads a level whose texels each average
/// the whole source footprint under them, so thin ink keeps its mass
/// (`LINEART_METHODS_RESEARCH_2026-10-18.md`: 0.55-0.81 kept by an area
/// filter against 0.31-0.45 by one bilinear sample).
pub fn feature_raster_texture(img: &DecodedImage) -> FeatureTexture {
    let (w, h) = (img.w.max(1), img.h.max(1));
    let mut data = Vec::new();
    for (lw, lh) in mip_sizes(w, h) {
        data.extend(area_resample_premultiplied(img, lw, lh));
    }
    FeatureTexture { w, h, data, sdf: false, extent: (1.0, 1.0), stroke_top: 0.0, max_lod: 0 }
}

/// The distance-field path's texture for a one-ink sprite (`ink` from
/// [`one_ink_colour`]): AP-10's own `SdfPyramid` built from the sprite's
/// alpha, its levels as the top of the mip chain.
///
/// **The mask is the pack sprite's alpha at 50 %, bilinearly upsampled 2×**
/// (a pack sprite is 256², Ruling F / Q-L4, so the mask is 512²). The pack is
/// what the map has: AP-10's higher-resolution original lives only in the
/// library session (and not in the project save, AP-1's finding), so it is
/// not used here. Why 2×, measured 2026-10-19 (`_ap5_probe`, x16 and x64): at
/// the sprite's own size `SdfPyramid::build` sized a 256² sprite's field at
/// 128², so a 3 px hatch line was 1.5 texels across, under Ruling CR's
/// 2-texel rule (its sizing reads a stroke as `2·edt`, 4 for that line), and
/// bilinear sampling between texel centres cut its peak distance: the line
/// drew 1-2 px wide at x16 where 4.5 were due. From the 2× mask the field is
/// 256² (3 texels across), and the 50 % contour sits where the coverage
/// crosses half rather than on whole-pixel steps. Corners still round by
/// about one 256² texel, the research's measured cost of a single-channel
/// field (§9.2, "MSDF").
///
/// Levels below the pyramid's smallest (`SDF_LEVEL_MIN`, 32²) are filler:
/// Godot needs the chain to 1 × 1, and the shader clamps its level to
/// `max_lod`, so they are 2 × 2 box means of the level above and never read.
/// `None` for a sprite with no ink at 50 % (nothing to outline).
pub fn feature_sdf_texture(img: &DecodedImage, ink: [u8; 3]) -> Option<FeatureTexture> {
    use cartalith_assets::lineart::{Mask, SdfPyramid};
    let (sw, sh) = (img.w.max(1) as usize, img.h.max(1) as usize);
    let (w, h) = (2 * sw, 2 * sh);
    let alpha = |x: usize, y: usize| f64::from(img.rgba[(y.min(sh - 1) * sw + x.min(sw - 1)) * 4 + 3]);
    let mut on = Vec::with_capacity(w * h);
    for y in 0..h {
        // The 2× pixel's centre in source pixel coordinates (centres at
        // integers), clamped at the border.
        let fy = ((y as f64 + 0.5) / 2.0 - 0.5).max(0.0);
        let (y0, ty) = (fy.floor() as usize, fy - fy.floor());
        for x in 0..w {
            let fx = ((x as f64 + 0.5) / 2.0 - 0.5).max(0.0);
            let (x0, tx) = (fx.floor() as usize, fx - fx.floor());
            let top = alpha(x0, y0) * (1.0 - tx) + alpha(x0 + 1, y0) * tx;
            let bot = alpha(x0, y0 + 1) * (1.0 - tx) + alpha(x0 + 1, y0 + 1) * tx;
            on.push(top * (1.0 - ty) + bot * ty >= 127.5);
        }
    }
    if !on.iter().any(|b| *b) {
        return None;
    }
    let mask = Mask { w, h, on };
    let pyr = SdfPyramid::build(&mask);
    let top = pyr.levels[0].size;
    let n = pyr.mask_size as f64;
    let mut data = Vec::new();
    let mut prev: Vec<u8> = Vec::new();
    let mut prev_side = 0u32;
    for (i, (side, _)) in mip_sizes(top, top).into_iter().enumerate() {
        let q: Vec<u8> = match pyr.levels.get(i) {
            Some(l) => l.q.clone(),
            None => {
                // Filler: a 2 x 2 box mean of the level above (never sampled).
                let s = side as usize;
                let ps = prev_side as usize;
                (0..s * s)
                    .map(|k| {
                        let (x, y) = (k % s, k / s);
                        let at = |xx: usize, yy: usize| u32::from(prev[yy.min(ps - 1) * ps + xx.min(ps - 1)]);
                        ((at(2 * x, 2 * y) + at(2 * x + 1, 2 * y) + at(2 * x, 2 * y + 1) + at(2 * x + 1, 2 * y + 1) + 2) / 4) as u8
                    })
                    .collect()
            }
        };
        for v in &q {
            data.extend_from_slice(&[ink[0], ink[1], ink[2], *v]);
        }
        prev = q;
        prev_side = side;
    }
    Some(FeatureTexture {
        w: top,
        h: top,
        data,
        sdf: true,
        extent: (w as f64 / n, h as f64 / n),
        stroke_top: pyr.stroke_p2 * f64::from(top) / n,
        max_lod: (pyr.levels.len() - 1) as u32,
    })
}

/// The texture AP-5's overlay draws for one pack feature sprite: the
/// distance-field path for one-ink art ([`one_ink_colour`]), else the
/// premultiplied raster path. The single place that choice is made.
pub fn feature_texture(img: &DecodedImage) -> FeatureTexture {
    one_ink_colour(img).and_then(|ink| feature_sdf_texture(img, ink)).unwrap_or_else(|| feature_raster_texture(img))
}

// ---------------------------------------------------------------------------
// Trait badges (`_civDrawTraitBadges` -> `_traitSprite`)
// ---------------------------------------------------------------------------

/// Why a laid-out trait badge got no sprite painted for it.
///
/// The reference collapses both into one `return false` (`_traitSprite`,
/// v2.11 line 15573: `if(!arr||!arr.length) return false`) because both take
/// the same drawing path. They are kept apart here because they are different
/// things to *tell a pack author*, and once collapsed the difference cannot
/// be recovered — see [`LoadedPack::traits`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraitArtMiss {
    /// The loaded pack's `structures.trait` never mentions this slot.
    NoArtInPack,
    /// The pack declares art for this slot and not one variant decoded — a
    /// manifest path with no ZIP entry, or a PNG `decode_png` rejected.
    ArtFailedToDecode,
}

impl TraitArtMiss {
    /// The stable snake_case name this reason crosses the gdext boundary
    /// under (`WorldGen::civ_trait_badge_row`'s `miss` value, and
    /// `WorldGen::pack_trait_art_status`'s per-slot value).
    ///
    /// A name rather than an integer because the two are meant to be told
    /// apart by whoever reads them — a `0`/`1` at a GDScript call site is
    /// exactly the "plausible value" shape `MISTAKES.md` warns about, and
    /// neither of these is a default.
    pub fn key(self) -> &'static str {
        match self {
            Self::NoArtInPack => "no_art_in_pack",
            Self::ArtFailedToDecode => "art_failed_to_decode",
        }
    }
}

/// What one laid-out badge should draw: the pack's sprite, or nothing (with
/// the reason).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TraitBadgeArt {
    /// `variant` indexes `LoadedPack::traits[key]` — [`pick_icon_variant`]'s
    /// choice for this badge's own centre — and `rect` is
    /// [`trait_sprite_rect`]'s centre-anchored destination box for that
    /// variant's own aspect.
    Sprite {
        variant: usize,
        rect: cartalith_assets::SpriteRect,
    },
    /// No sprite, and which of [`TraitArtMiss`]' two reasons.
    Miss(TraitArtMiss),
}

/// One trait badge, laid out **and** resolved against a loaded pack: where it
/// goes, which glyph labels it, and whether the pack supplied art for it.
///
/// This is all of `_civDrawTraitBadges`' decision-making with none of its
/// drawing, so that both consumers make the same decisions from one
/// definition: [`composite_trait_badges`], which paints into the RGB8 map
/// buffer, and `WorldGen::civ_trait_badge_row`, which hands the same answer
/// across the gdext boundary for Godot's own renderer to blit at a pin's
/// constant on-screen size. *"Two functions answering one question WILL
/// drift"* is the reference's own comment about `_civTraitDrop` (v1.73);
/// this is that lesson applied one call deeper.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedTraitBadge {
    /// The trait key, as it appeared in the settlement's own `traits` list.
    pub key: String,
    /// Badge centre and radius, straight from
    /// [`cartalith_assets::trait_badge_layout`] — the ported geometry, so a
    /// consumer's own fallback lands exactly where the sprite would have.
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
    /// The glyph the reference draws on the fallback disc, from
    /// `cartalith_civ::roster::CIV_TRAITS`.
    ///
    /// **`None` means `key` is not a real trait**, and the reference draws
    /// *nothing at all* for it — its fallback is guarded by `const
    /// t=CIV_TRAITS.find(...); if(t){...}`, so an unknown key produces no
    /// disc and no glyph. A consumer must skip such a badge, not draw a
    /// blank disc in its place.
    pub glyph: Option<&'static str>,
    pub art: TraitBadgeArt,
}

/// `_civDrawTraitBadges` (reference v2.11 line 15584) reduced to its
/// decisions: lay the row out, resolve each key against a loaded pack's
/// `structures.trait` art, pick the variant and compute its destination box.
/// Draws nothing — see [`ResolvedTraitBadge`] for why that split exists.
///
/// The geometry is not computed here either. It is
/// [`cartalith_assets::trait_badge_layout`]'s, already ported term-for-term
/// and mutation-guarded (the `slice(0,4)` cap, the `r*2.35` spacing, the
/// `py+sz+r+1.2*sc` row position), with
/// [`cartalith_assets::trait_sprite_rect`] for each sprite's own
/// centre-anchored `radius*2` box.
///
/// **Variants are picked with [`pick_icon_variant`], not
/// [`pick_weighted_variant`]** — the reference's own asymmetry, not an
/// oversight here: `_traitSprite` (15574) calls `pickIconVariant` with no
/// weights, while its two centre-anchored siblings `_customSprite` (15614)
/// and `_featureSprite` (15628) both consult `assetRules[...].variantWeights`.
/// A trait badge therefore ignores a variant weighting set in the Library,
/// and matching the reference is the contract even where the asymmetry looks
/// accidental.
///
/// `px`/`py` are the pin's centre and `sz`/`sc` the pin radius and layer
/// scale, all four in whatever space the consumer draws in — grid texels for
/// [`composite_trait_badges`], the overlay control's own local pixels for
/// `WorldGen::civ_trait_badge_row`. The returned `cx`/`cy`/`r` and the
/// sprite rect come back in that same space.
pub fn resolve_trait_badges(
    px: f64,
    py: f64,
    traits: &[String],
    sz: f64,
    sc: f64,
    seed: i32,
    pack: &LoadedPack,
) -> Vec<ResolvedTraitBadge> {
    trait_badge_layout(px, py, traits, sz, sc)
        .into_iter()
        .map(|badge| {
            let art = match pack.traits.get(&badge.key) {
                None => TraitBadgeArt::Miss(TraitArtMiss::NoArtInPack),
                Some(variants) if variants.is_empty() => TraitBadgeArt::Miss(TraitArtMiss::ArtFailedToDecode),
                Some(variants) => {
                    // `pickIconVariant(px|0, py|0, seed, arr.length)` — the
                    // badge's own centre, truncated, exactly as the reference
                    // hashes it.
                    let variant = pick_icon_variant(badge.cx as i32, badge.cy as i32, seed, variants.len());
                    let img = &variants[variant];
                    let rect = trait_sprite_rect(badge.cx, badge.cy, badge.r, img.w as f64, img.h as f64);
                    TraitBadgeArt::Sprite { variant, rect }
                }
            };
            let glyph = cartalith_civ::roster::CIV_TRAITS
                .iter()
                .find(|&&(k, _, _)| k == badge.key)
                .map(|&(_, _, g)| g);
            ResolvedTraitBadge { key: badge.key, cx: badge.cx, cy: badge.cy, r: badge.r, glyph, art }
        })
        .collect()
}

/// One badge [`composite_trait_badges`] laid out and **did not paint**,
/// with everything a caller needs to finish it where a font exists.
///
/// The reference's fallback for a missing sprite is a dark disc with the
/// trait's own `CIV_TRAITS` glyph drawn on it (`_civDrawTraitBadges`, v2.11
/// lines 15592-15598) — *text*, which this module's software rasterizer
/// cannot draw. So nothing is painted for a miss rather than the disc alone:
/// a bare disc is a plausible-looking badge that says nothing about which
/// trait it is, and would be indistinguishable between all seven.
#[allow(dead_code, reason = "returned only by `composite_trait_badges`, which has no caller yet — see its doc")]
pub struct TraitBadgeFallback {
    /// The trait key, as it appeared in the settlement's own `traits` list.
    pub key: String,
    /// Badge centre and radius, straight from
    /// [`cartalith_assets::trait_badge_layout`] — the ported geometry, so a
    /// caller's fallback lands exactly where the sprite would have.
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
    /// The glyph the reference draws on the disc, from
    /// `cartalith_civ::roster::CIV_TRAITS`.
    ///
    /// **`None` means `key` is not a real trait**, and the reference draws
    /// *nothing at all* for it — its fallback is guarded by `const
    /// t=CIV_TRAITS.find(...); if(t){...}`, so an unknown key produces no
    /// disc and no glyph. A caller must skip such a badge, not draw a blank
    /// disc in its place.
    pub glyph: Option<&'static str>,
    pub miss: TraitArtMiss,
}

/// `_civDrawTraitBadges` (reference v2.11 line 15584), sprite half: paint one
/// settlement's row of trait badges from a loaded pack's `structures.trait`
/// art into an RGB8 raster, and return every badge the pack had no sprite for.
///
/// [`resolve_trait_badges`] makes every decision — layout, lookup, variant
/// pick, destination box. This function is the blit and nothing else, so a
/// badge painted here and a badge drawn by `map_overlay.gd` from
/// `WorldGen::civ_trait_badge_row` cannot disagree about which sprite goes
/// where.
///
/// `px`/`py` are the pin's centre **in the coordinate space of `bytes`**, and
/// `sz`/`sc` the pin radius and layer scale in that same space.
///
/// # No in-tree caller, and why the `dead_code` allow is here
///
/// **Still true on 2026-09-04, and the reason narrowed rather than lifted.**
/// The route to a settlement pin now exists and is not this function: pins
/// are drawn by `godot-project/map_overlay.gd` at a constant on-screen size,
/// and [`resolve_trait_badges`] reaches it through
/// `WorldGen::civ_trait_badge_row`, which hands GDScript an `ImageTexture`
/// per badge for Godot's own renderer to blit. This function paints into the
/// terrain buffer at grid resolution, where a badge would be baked at map
/// scale instead — the same mismatch that rules out [`composite_map_icons`]
/// as the caller.
///
/// What it is still for: an offline raster of the map (`export_raster.rs`'s
/// domain) has no Godot renderer to blit with and would need exactly this.
/// Nothing there draws settlement pins today, so the allow stays.
#[allow(dead_code, reason = "no caller: the live map now blits via `civ_trait_badge_row`, and no offline raster draws pins — see the section above")]
#[allow(clippy::too_many_arguments)]
#[must_use = "the returned badges are the ones with no art; drop the list and those traits vanish from the map with no fallback drawn"]
pub fn composite_trait_badges(
    bytes: &mut [u8],
    gw: usize,
    gh: usize,
    px: f64,
    py: f64,
    traits: &[String],
    sz: f64,
    sc: f64,
    seed: i32,
    pack: &LoadedPack,
) -> Vec<TraitBadgeFallback> {
    let mut misses = Vec::new();
    if gw == 0 || gh == 0 {
        return misses;
    }
    let mut canvas = Canvas { bytes, gw, gh };
    for badge in resolve_trait_badges(px, py, traits, sz, sc, seed, pack) {
        match badge.art {
            TraitBadgeArt::Sprite { variant, rect } => {
                canvas.blit_sprite(&pack.traits[&badge.key][variant], rect);
            }
            TraitBadgeArt::Miss(miss) => {
                misses.push(TraitBadgeFallback {
                    key: badge.key,
                    cx: badge.cx,
                    cy: badge.cy,
                    r: badge.r,
                    glyph: badge.glyph,
                    miss,
                });
            }
        }
    }
    misses
}
