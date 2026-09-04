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
//! traits`, [`composite_trait_badges`]) — `OUTSTANDING_WORK.md` §2.5, the
//! first of the `structures` families to be rasterised at all. It differs from
//! the three above in one way that matters when reading the rest of this file:
//! **it has no in-tree caller.** The other three are reached from
//! `build_color_texture`; this one's consumer is the settlement-pin pass in
//! `godot-project/map_overlay.gd`, because a trait badge hangs off a pin and
//! [`composite_map_icons`] never sees one. See that function's own doc for
//! the whole of it.
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
    decode_png, finalize_pack_texture_inv_mean, icon_slot_for_item, pick_icon_variant,
    pick_weighted_variant, place_map_icons_ruled, read_pack, sprite_draw_rect,
    trait_badge_layout, trait_sprite_rect,
};

use crate::render::{GroundTile, SplatChannel};

/// A real, loaded asset pack — the five families this module composites,
/// decoded to pixels: `icons`, the six splat channels, `biomes`, `terrains`
/// and (since `OUTSTANDING_WORK.md` §2.5) `structures.trait`.
///
/// `structures.settlement`, `structures.poi`, `custom` and `seamarks` are
/// parsed into [`Self::manifest`] and **not** decoded here, because nothing
/// draws them: `map_overlay.gd`'s `_draw_manual_icons` draws a settlement as
/// a filled rectangle and a POI as a diamond, never a pack sprite. This
/// sentence used to point at the module doc above for that reason; the module
/// doc has never carried it, so it is stated here instead.
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
    /// manifest order, consumed by [`composite_trait_badges`].
    ///
    /// **An absent key and a key holding an empty `Vec` are different
    /// states, and that is the reason this map is not filtered the way
    /// `icons` above is.** Absent: the pack never declared trait art for
    /// that slot. Empty: it declared some and not one variant decoded — a
    /// missing ZIP entry or a broken PNG. Both draw the same thing (nothing;
    /// the reference's own `if(!arr||!arr.length) return false` at
    /// `_traitSprite`, v2.11 line 15573, does not distinguish them either),
    /// and [`composite_trait_badges`] returns them as two different
    /// [`TraitArtMiss`] reasons so the surface reporting the miss can tell a
    /// pack author *"you never added port art"* from *"your port art is a
    /// broken PNG"*. Collapsing them here would make that impossible to
    /// recover later.
    #[allow(dead_code, reason = "read only by `composite_trait_badges`, whose caller is GDScript")]
    pub traits: HashMap<String, Vec<DecodedImage>>,
}

/// Decode one painted-ground family into the positional table
/// [`LoadedPack::biomes`]/[`LoadedPack::terrains`] describes.
///
/// **No `finalize_pack_texture_inv_mean` here, unlike the splat loop below**,
/// and that is the reference's asymmetry rather than an omission: a splat
/// channel modulates a procedural ramp by `texel/mean`, while a painted tile
/// is blended as true colour (`ASSET_LIBRARY_SCOPE.md` §1 — "dividing out a
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
) -> Vec<Option<GroundTile>> {
    slots
        .iter()
        .map(|&slot| {
            let img = decode_png(entries.get(table.get(slot)?)?).ok()?;
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

    let mut icons: HashMap<String, Vec<DecodedImage>> = HashMap::new();
    for (slot, paths) in manifest.icons.iter() {
        let mut variants = Vec::new();
        for path in paths {
            let Some(data) = entries.get(path) else { continue };
            if let Ok(img) = decode_png(data) {
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
        let Ok(img) = decode_png(data) else { continue };
        let inv = finalize_pack_texture_inv_mean(img.w, img.h, &img.rgba);
        splat.insert(slot, SplatChannel { w: img.w, h: img.h, rgba: img.rgba, inv });
    }

    let biomes = decode_ground_family(&PACK_BIOME_SLOTS, &manifest.biomes, &entries);
    let terrains = decode_ground_family(&PACK_TERRAIN_SLOTS, &manifest.terrains, &entries);

    // `structures.trait`. Same per-variant tolerance as `icons` above, with
    // one deliberate difference: the slot is inserted **even when no variant
    // decoded**, because [`LoadedPack::traits`] uses absent-vs-empty to carry
    // exactly that distinction. Do not add an `is_empty()` filter here.
    let mut traits: HashMap<String, Vec<DecodedImage>> = HashMap::new();
    for (slot, paths) in manifest.structures.traits.iter() {
        let mut variants = Vec::new();
        for path in paths {
            let Some(data) = entries.get(path) else { continue };
            if let Ok(img) = decode_png(data) {
                variants.push(img);
            }
        }
        traits.insert(slot.to_string(), variants);
    }

    Ok(LoadedPack { manifest, icons, splat, biomes, terrains, traits })
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
#[allow(clippy::too_many_arguments)]
pub fn composite_map_icons(bytes: &mut [u8], field: &[f32], temperature: &[f32], rainfall: &[f32], gw: usize, gh: usize, sea_level: f64, seed: i32, pack: &LoadedPack) {
    let mut table = ScatterRuleTable::default();
    autopopulate_scatter_rules(&mut table, &pack.manifest);
    let Some(rules) = current_scatter_rules(&table) else {
        return;
    };
    if gw == 0 || gh == 0 {
        return;
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

    let base = (gw as f64 / 110.0).max(3.5);
    let mut canvas = Canvas { bytes, gw, gh };
    for item in &items {
        let slot = icon_slot_for_item(item);
        if let Some(variants) = pack.icons.get(&slot)
            && !variants.is_empty()
        {
            let weights = table.get(&slot).and_then(|r| r.variant_weights.as_deref());
            let idx = pick_weighted_variant(item.x, item.y, seed, variants.len(), weights);
            let img = &variants[idx];
            let rect = sprite_draw_rect(item.x as f64, item.y as f64, item.s, base, img.w as f64, img.h as f64);
            canvas.blit_sprite(img, rect);
            continue;
        }
        draw_icon_glyph(&mut canvas, &slot, item, base);
    }
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
#[allow(dead_code, reason = "returned only by `composite_trait_badges`, whose caller is GDScript")]
pub enum TraitArtMiss {
    /// The loaded pack's `structures.trait` never mentions this slot.
    NoArtInPack,
    /// The pack declares art for this slot and not one variant decoded — a
    /// manifest path with no ZIP entry, or a PNG `decode_png` rejected.
    ArtFailedToDecode,
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
#[allow(dead_code, reason = "returned only by `composite_trait_badges`, whose caller is GDScript")]
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
/// art, and return every badge the pack had no sprite for.
///
/// The geometry is not computed here — it is
/// [`cartalith_assets::trait_badge_layout`]'s, already ported term-for-term
/// and mutation-guarded (the `slice(0,4)` cap, the `r*2.35` spacing, the
/// `py+sz+r+1.2*sc` row position), with
/// [`cartalith_assets::trait_sprite_rect`] for each sprite's own
/// centre-anchored `radius*2` box. This function is the lookup, the variant
/// pick and the blit, and nothing else.
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
/// `px`/`py` are the pin's centre **in the coordinate space of `bytes`**, and
/// `sz`/`sc` the pin radius and layer scale in that same space.
///
/// # No in-tree caller yet, and why the `dead_code` allow is here
///
/// The consumer is the settlement-pin pass in
/// `godot-project/map_overlay.gd`: it draws the pins and their labels, and it
/// is the only surface that knows where a pin sits on screen.
/// [`composite_map_icons`] cannot be that caller — it works on the terrain
/// buffer at grid resolution and never sees a pin, so a badge composited
/// there would be baked at map scale rather than the pin's constant on-screen
/// size. That wiring is GDScript, and is the remaining half of
/// `OUTSTANDING_WORK.md` §2.5. The allow states the gap once here instead of
/// restating it as four build warnings; it goes with the first caller.
#[allow(dead_code, reason = "the caller is GDScript — see the section above")]
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
    for badge in trait_badge_layout(px, py, traits, sz, sc) {
        let miss = match pack.traits.get(&badge.key) {
            None => TraitArtMiss::NoArtInPack,
            Some(variants) if variants.is_empty() => TraitArtMiss::ArtFailedToDecode,
            Some(variants) => {
                // `pickIconVariant(px|0, py|0, seed, arr.length)` — the badge's
                // own centre, truncated, exactly as the reference hashes it.
                let idx = pick_icon_variant(badge.cx as i32, badge.cy as i32, seed, variants.len());
                let img = &variants[idx];
                let rect = trait_sprite_rect(badge.cx, badge.cy, badge.r, img.w as f64, img.h as f64);
                canvas.blit_sprite(img, rect);
                continue;
            }
        };
        let glyph = cartalith_civ::roster::CIV_TRAITS
            .iter()
            .find(|&&(k, _, _)| k == badge.key)
            .map(|&(_, _, g)| g);
        misses.push(TraitBadgeFallback { key: badge.key, cx: badge.cx, cy: badge.cy, r: badge.r, glyph, miss });
    }
    misses
}
