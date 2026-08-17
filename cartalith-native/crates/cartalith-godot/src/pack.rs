//! Real asset-pack loading and sprite compositing — `ASSET_LIBRARY_SCOPE.md`
//! milestone 7, the final milestone of Phase 4. Consumes `cartalith-assets`
//! (milestones 1-6, all pure logic, none of it wired to anything before this
//! milestone) and does the actual pixel work: decoding a pack's images and
//! blitting/blending them onto the RGB8 buffer `lib.rs::build_color_texture`
//! already builds.
//!
//! ## What this module composites, and what it deliberately does not
//!
//! Two of the milestone's three named surfaces are real here:
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
//!
//! **The third named surface — the two "painted layers" (`_paintedTex`'s
//! `biomes`/`terrains` families, the Cartography paint-brush biome/terrain
//! override) — is deliberately NOT implemented this pass.** Read literally
//! (reference lines 7898-7900, 12187-12196): `pBio`/`pTer` are per-cell
//! indices into `state.cartoPaint.biome`/`.terrain`, sparse arrays a user
//! populates by hand with a paint-brush tool (`paintBiome`/`paintSplat`/
//! `paintTerrain` module globals, reference ~26200+). This port has never
//! ported that tool — there is no producer of a painted-cell array anywhere
//! in this workspace, and building one from scratch is itself a real,
//! separate UI+state effort this milestone's "no GUI controls" boundary
//! rules out (the paint tool has no meaning without a brush UI to drive it).
//! Unlike splat (gated only by `assetPack.texAny`, active by default the
//! moment a pack is loaded) and icons (gated by `state.viz.icons`, default
//! `false` — never on by default either way), the painted layers are gated
//! by a *third* piece of state this port simply does not have a producer
//! for. Decoding `biomes`/`terrains` pack images with nothing that could
//! ever set `pBio`/`pTer` to a nonzero value would be dead code — so
//! [`LoadedPack`] does not decode them, and `PackManifest`'s own `.biomes`/
//! `.terrains` fields are parsed (for a correct `packSummary`-equivalent and
//! warning count) but never turned into pixels. A future milestone that
//! ports the Cartography paint-brush tool is the natural place to pick this
//! back up.

use std::collections::HashMap;
use std::io::Cursor;

use cartalith_assets::{
    DecodedImage, PACK_TEX_SLOTS, PackManifest, PlaceIconsRuledOpts, PlacedIcon,
    ScatterRuleTable, autopopulate_scatter_rules, current_scatter_rules,
    decode_png, finalize_pack_texture_inv_mean, icon_slot_for_item, pick_weighted_variant,
    place_map_icons_ruled, read_pack, sprite_draw_rect,
};

use crate::render::SplatChannel;

/// A real, loaded asset pack — only the two families this milestone
/// composites decoded to pixels; see this module's own doc comment for why
/// `biomes`/`terrains`/`structures`/`custom` are parsed (`manifest`) but not
/// rasterised.
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

    Ok(LoadedPack { manifest, icons, splat })
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

    /// One real placed sprite, bottom-anchored via `sprite_draw_rect`'s
    /// destination rectangle (`spriteDrawRect`, milestone 4).
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
