# Asset pack refinement tools: proposal

**A proposal of 2026-10-18, from an owner request.** Q1–Q10 were approved at
their defaults the same day (Ruling CQ, `LARGE_ITEM_RULINGS.md`). §9, the
line-art research, was added later that day; it proposes AP-10 and asks
Q-L1…Q-L6, approved at their defaults the same day (Ruling CR). §10 records
what building AP-1 and AP-2 settled. It defines milestones (AP-1…AP-10) and
the reasoning behind them. It has no status
column: status lives in `cartalith-native/docs/STATUS.md` and nowhere else.

**The request (2026-10-18).** Put "tools to refine asset packs and all that
jazz" on the roadmap. The brief asked for an audit of what exists, the gaps
for someone who wants to *refine* a pack rather than author one from nothing,
a comparison with the map tools the owner's video listed (Inkarnate,
Wonderdraft, Dungeondraft, Dungeon Alchemist, DungeonFog, Canvas of Kings),
a menu home, risks, milestones and owner questions.

**Evidence base, stated per Ruling BK.** Every fact about this program names
the symbol I opened on 2026-10-18, at `2446ed81`. Four window captures were
taken (§1.6). The "Measured" label is used only for numbers produced today;
everything else is read from code or cited. Nothing in the engine or the
shell was changed. Third-party formats are taken from public pages, and §3
names, for each source, whether I fetched the page or only saw a search
summary of it.

**Two rulings bind everything below** (`LARGE_ITEM_RULINGS.md`, quoted in
`ASSET_LIBRARY_SCOPE.md`'s header):

- **Ruling F (2026-09-07):** packs stay `.zip`, and their format stays
  identical to the HTML app's. Every tool here therefore writes **pixels and
  existing manifest fields only**. A tool that needs a new `pack.json` key is
  an owner question, not a design choice. The one precedent is `seamarks`,
  which an owner ruling added (`manifest.rs`, `RawManifest::seamarks` doc).
- **Ruling AP (2026-09-23):** library images are embedded in the project save.
  *(Ruled, not yet built: Ruling AP itself calls it "real save-format work";
  see §10's last item.)*
  Refinement state can therefore live in the project's `library.json`
  (`AssetDB::to_library_json`) without touching the pack format.

---

## 1. What exists today (opened 2026-10-18)

### 1.1 The model: `cartalith-assets`

- **What an asset is.** One PNG bound to one slot of a frozen, ordered
  vocabulary, in nine families (`slots.rs`, `Family::ALL`). Three of them are
  512² opaque tiled ground textures: `textures` (7 splat channels), `biomes`
  (15) and `terrains` (13). Six are 256² RGBA sprites: `icons` (10),
  `settlement` (9), `trait` (7), `poi` (10 in the library), `seamarks` (8),
  and the open `custom` family (sets of named slots). The anchor is **per
  family**, not per sprite (`Family::anchor`, AS-15 in
  `asset_library_window.gd`'s header).
- **The pack format** (`manifest.rs::RawManifest`, `archive.rs`). `pack.json`
  holds `schema`, `name`, `author`, `license` and one path list per family.
  Nothing else: **no per-slot tags, licence, author, source or scatter rules
  travel in a pack.** Those live in `SlotMeta` (`author`, `copyright`,
  `license`, `source`, `notes`, `version`, `tags`) and the scatter-rule table,
  which are saved in the project's `library.json`, not in the pack.
- **The library** (`library.rs`): `AssetDB`, `AssetCollections`, the
  validator `run`, `duplicate_groups`, `rename_custom_slot` and
  `add_custom_slot`.
- **Per-item display transform** (`ItemTransform`): `scale`, `panX`, `panY`.
  That is the whole of it: no rotation, flip, trim, pivot or tint. It is baked
  into the exported pixels by `raster::render_item` at the family's bake size
  (`asset_bridge.rs::export_pack_bytes`).
- **Duplicates** (`raster::item_hash`): FNV-1a over a 32×32 triangle-filtered
  thumbnail, with the original size appended. It finds **exact or
  thumbnail-identical** images. It does not find a recoloured, flipped or
  re-cropped near-duplicate.
- **The slicer** (`slicer.rs`): a golden-verified port of the reference's
  uniform grid (`compute_cells` and its half-gutter spacing), plus draggable
  interior lines (`move_line`), the chroma key (`apply_chroma`), blank
  detection (`is_blank`, alpha > 8) and one port-side addition,
  `trim_transparent_edges`. **There is no automatic detection** of sprite
  bounds on an irregular sheet: the grid is always the user's.
- **Colour.** Import goes through `colour_import.rs::decode_png_managed`
  (ET-E2). An image tagged with a non-sRGB colour space is converted to sRGB,
  and the shell prints one line about it (`AssetLibraryWindow.colour_note_for`).
  Untagged files are treated as sRGB. Packs are exported untagged.

### 1.2 The shell

- **The window** (`shell/asset_library_window.gd`, `AssetLibraryWindow`). It
  has four parts:
  - a family rail (`FAMILIES`, nine entries) with Collections and an
    *Unassigned imports* bucket (`UNASSIGNED_SET`);
  - a slot grid with search over name, id, code, set and tag
    (`SEARCH_PLACEHOLDER`, `_slot_matches`);
  - an inspector showing the file line, Scale, Pan, Fit, Reset, Replace…,
    + Variant, five preview backgrounds (`PREVIEW_SWATCHES`), the anchor
    segment, tags, the variant strip and the pack metadata fields;
  - BATCH verbs on a selection: Tag…, Collect…, Rename…, Duplicate and Delete
    (`_on_batch_*` → `as_batch_*`).
  The footer has Import image…, Import pack…, Validate and Clear library….
  The header has Apply to map and Export pack .zip.
- **The slicer modal** (`_build_slicer_modal`). Its controls are columns,
  rows, margin and spacing; trim, skip empty, and background → transparent
  with key and tolerance; and the targets in `SLICE_TARGETS`.
- **Menus** (`menus.gd::_assets`). The Assets menu holds:
  - ⧉ Asset library (⇧A), ⧉ Sprite sheet slicer, Import image… and Import
    asset pack .zip…;
  - the `Asset pack ▸` submenu (`_build_asset_pack_submenu`). It is nine rows
    by owner ruling (2026-09-05 item 4): five reads (NAME, AUTHOR, LICENSE,
    SCHEMA, FILLED) and three items (Pack metadata…, Validate pack, Export pack
    .zip…);
  - `Icon families` and `Texture sets`, which show live filled/capacity counts
    (`_refresh_family_counts`);
  - `Landmark types ▸`, Apply library to map, and Clear library….
- **The current menu bar** (`menus.gd::build`) is File, Edit, Assets, World,
  Story, Preferences, Window and Help. This follows `STORY_MENUS_PROPOSAL.md`,
  whose SM-1…SM-7 are recorded as built in `STATUS.md`. On a tablet the
  Library sits in ☰ (`TABLET_UI_SPEC.md` §2.1). On a phone the window splits
  into panes (PH-12, `_show_phone_pane`).

### 1.3 How a pack reaches the map

This is the part that decides what "refining" can even change.

| Family | Drawn on the map? | Path | At deep zoom |
|---|---|---|---|
| `textures` (splat, 6 of 7) | yes | `render.rs::splat_sample`: procedural ramp × texel × `inv` (the texture's per-channel inverse mean, `finalize_pack_texture_inv_mean`). Nearest-neighbour, wrapped, one texel per output pixel | `lod_worker.rs::OwnedSplat` carries the channels into deep-zoom tiles |
| `textures.parchment` | **no** | parsed, not decoded (`LoadedPack::splat` doc) | — |
| `biomes`, `terrains` | yes, where cells are painted | `render.rs` paint blend at the reference's 0.60 weight, one texel per grid cell, wrapped | carried as `GroundTile`s |
| `icons` (10 feature slots) | yes | `pack::composite_map_icons`. It is **blitted into the grid-resolution RGB8 terrain buffer** inside `WorldGen::build_color_texture`, with `base = max(gw/110, 3.5)` px | **No tile path mentions it**: `lod_worker.rs` and `lod_bridge.rs` contain no `composite_map_icons` and no icon term |
| `trait` | yes | `WorldGen::civ_trait_badge_row` → `ImageTexture` → `map_overlay.gd` `draw_texture_rect`, at a constant on-screen size | screen-space, so it stays sharp |
| `settlement`, `poi`, `custom`, `seamarks` | **no** | `map_overlay.gd`'s comment above `ICON_FAMILY_COLORS`: "`settlement`, `poi` and `custom` have no compositor in the port at all". `LoadedPack` has no field for them, and `seamarks` likewise | placeholder glyphs |

The table leads to three facts that shape this proposal:

1. **Refining a settlement pin, a POI marker, a sea mark or a custom icon
   changes nothing on the map today.** It changes only the exported pack,
   which the HTML app can read.
2. **Feature icons cannot keep their quality at ×16 or ×64**, by
   construction. On a 2048-wide grid, an `s = 1` sprite is about 41 grid
   pixels tall: `placement::sprite_draw_rect` gives `dh = base × 2.2 × s`,
   and `base` is 2048 / 110 = 18.6. *(Corrected 2026-10-18 in §9's pass:
   this line first gave `base`, 18.6, as the sprite's size.)* It is
   resampled from 256² down to that once, into the base texture, and then
   magnified. This is the owner's hard
   zoom-detail requirement (memory note; `RENDER_TARGET_GAP_ASSESSMENT_2026-10-17.md`).
   I did not capture a zoom sweep of it; AP-5 does.
3. **Splat textures harmonise with the Look by design.** Their colour comes
   from the procedural ramp, and the texture contributes only its variation
   about its mean. Painted biome and terrain tiles, and every sprite, carry
   **absolute** colour. The composite runs *after* the grade and the
   adjustment stacks, deliberately: the comment in
   `WorldGen::build_color_texture` says those "have no business touching drawn
   artwork". Recolouring to the Look is therefore a question about sprites and
   ground tiles, not about splat.

**Export.** The export raster attaches splat and ground tiles
(`export_raster.rs`, `with_splat`). No export path calls
`composite_map_icons`: the only callers are in `lib.rs`'s
`build_color_texture`. I did not open what the export's `icons` overlay switch
(`export_options.rs::ExportContent::icons`) draws.

### 1.4 Getting a pack in, and back out

- *(As read on 2026-10-18 before AP-1; §10 has what was built.)*
  **Import pack… loads a pack into the renderer, not into the library.** The
  window's footer chip calls `app.open_asset_pack_picker()`, which calls
  `WorldGen::load_asset_pack`. That decodes into `self.asset_pack` and never
  touches `AssetLibrarySession`. No `as_*` binding reads a pack. The reference
  does both: `AssetImporter.importPackZip` ("Load an existing pack ZIP … into
  the library for editing", v2.10 line 27067), and `window._alImportPackZip`
  absorbs a header-loaded pack into the library (line 27933).
  `ASSET_LIBRARY_SCOPE.md` §6 records this as "a named non-port". I found it
  routed nowhere in `OUTSTANDING_WORK.md`.
  **So nobody can open an existing pack and refine it today.** They can only
  rebuild it slot by slot from loose PNGs.
- **Export** (`AssetLibrarySession::export_pack_bytes`). An empty name exports
  as "Custom Asset Pack", an empty author as "Cartalith Gen1", and **an empty
  licence as "CC0"**. That is the reference's own behaviour
  (`license:…||'CC0'`, v2.10 line 26973), so it is faithful. It also means a
  pack whose licence nobody entered ships labelled public-domain. The window
  shows the licence field blank (capture 02).
- **The validator** (`library::run`) checks only: name empty, library empty,
  duplicate identifier, invalid custom id, identical images, and collections
  pointing at missing assets. Its own doc says two of these cannot fire
  through the public API.

### 1.5 The reference pack

`crates/cartalith-assets/tests/fixtures/reference_pack.zip` holds 17 PNGs
plus `pack.json` (schema 2, "Reference Round-Trip Pack", CC0-1.0). Its
textures are 512×512 and its sprites 256×256, all RGBA8 (IHDR read today).
The PNGs are STOREd and `pack.json` is deflated. It is a format fixture, not
art: every image is a flat colour, which is why capture 02 shows three
"mountain" variants flagged as identical.

### 1.6 Captures

`cartalith-native/docs/evidence/asset-library-2026-10-18/`. They were taken
at 1600×900 in a scratch copy of `HEAD` (`git archive`), with the dark palette
(the probe printed `palette=dark`) and the existing debug DLL (built
2026-10-17 21:47; HEAD's last crate change is `a8681328`, 2026-10-18 20:42,
which is geology save code, not assets). Slots were filled from the fixture
through the real `as_import_item`. The scratch copy was deleted afterwards.

| File | Shows |
|---|---|
| `01_library_feature_icons_empty_focus.png` | the rail with live counts (TX 2/7, IC 2/10); the duplicate ⚠ on Mountain and Conifer; no slot focused |
| `02_library_slot_inspector.png` | the inspector on IC-01: Scale/Pan/Fit/Reset/Replace/+Variant, the preview background, the anchor lit at `base` with the others disabled, the three-variant strip, and empty pack metadata |
| `03_library_splat_textures.png` | Splat channels; anchor `tiled`; the status line "4 / 79 slots · 7 items" |
| `04_sprite_sheet_slicer.png` | the slicer modal with no sheet loaded |

---

## 2. The gaps for someone who wants to refine a pack

Value is judged for a writer or DM who already owns some art and wants it to
look right on their map. Effort counts new code, not just UI. "Format" says
whether the item stays inside Ruling F.

| Rank | Gap | Value | Effort | Format | Why it ranks here |
|---|---|---|---|---|---|
| 1 | **Open a pack in the library** (port `importPackZip`) | very high | small–medium | identical | Every other tool here edits a library. Without this, refining means rebuilding. All the parts exist: `read_pack_entries`, `parse_pack_entries`, `decode_png_managed`, `item_hash`, `AssetDB::add_item` |
| 2 | **Pack report**: validation beyond `library::run` | high | small–medium | identical (report only) | It catches the silent CC0 licence; slots the map never draws (§1.3); sources below their bake size (upscaled); non-square sprites; icons with no transparent pixel; non-seamless ground tiles (gap 4); non-sRGB conversions; and empty slots per family. It reads only |
| 3 | **Draw the families the map ignores** (settlement, poi, custom, seamarks) as screen-space sprites | very high | medium–large | identical | Refining a pin has no visible effect until this exists. The trait-badge path (`civ_trait_badge_row` → `ImageTexture` → `draw_texture_rect`) is the working precedent, and it is sharp at any zoom |
| 4 | **Seam checker and make-seamless** for the 35 ground slots | high | small–medium | identical (writes pixels) | Splat and ground tiles wrap nearest-neighbour at one texel per pixel or cell (§1.3), so a seam repeats every 512 px across the whole map. Check: edge-difference metric. Fix: offset-and-blend, or mirror |
| 5 | **Zoom preview, and feature icons that hold up at ×16/×64** | very high | large (engine) | identical | Feature icons are baked into the grid-resolution raster (§1.3), so no refinement of the art survives zoom. Measure first, then move icons to a screen-space or tile-resolution sprite pass. It changes rendered output, so it needs a ruling (Q3) |
| 6 | **Near-duplicate finder** | medium | small | identical | `item_hash` finds exact copies only. A 64-bit difference hash (dHash) over flips and small tints finds the "same tree, slightly different" rows a merged pack accumulates |
| 7 | **Harmonise to the Look**: tint, hue, saturation and value baked into sprites and ground tiles | high | medium | identical (writes pixels) | Splat already harmonises (§1.3); ground tiles and sprites do not. Bake-time, so the pack stays portable. The alternative, sprites following the Look at render time, is a rendering change (Q4) |
| 8 | **Batch fit, trim and pad**, and per-item pivot via pan | medium | small | identical | `fit_to_bottom` exists per item; there is no batch "fit all" and no trim on import. A per-sprite *anchor* is not in the format (anchor is per family), so a pivot can only be baked as pan (Q5) |
| 9 | **Variant generation**: flip, tint jitter, scale jitter, saved as new variants | medium | small | identical | Variants are already weighted at render time (AS-14). Mirrored trees and slightly varied mountains are the cheapest way past "forty copies of one drawing" (`lib.rs` doc) |
| 10 | **Outline and drop-shadow generator** (baked) | medium | small–medium | identical | Readability on busy terrain. A per-pixel alpha dilation plus an offset blur at the 256² bake. Optional per batch |
| 11 | **Heuristic auto-tagging** (no ML): dominant hue name, alpha coverage, aspect, light/dark | low–medium | small | library only | Tags are searchable today (`_slot_matches`) but typed by hand. Note that **tags do not travel in a pack** (§1.1), so this helps the library, not the shared pack |
| 12 | **Import adapters**: a Wonderdraft sprite folder, an *unpacked* Dungeondraft asset tree, a loose PNG folder | medium | medium | identical | Each maps folders to a `custom` set, or to a slot by a name table. A packed `.dungeondraft_pack` is a binary container (§3) and is out of scope. Licence caution, §5 |
| 13 | **Pack diff and merge** | medium | medium | identical | Slot by slot: which slots each pack fills, `item_hash` equality, and which variant wins. Needs rank 1 |
| 14 | **Scale normalisation per km footprint** | low–medium | medium | needs a ruling | Sprite size is `base = max(gw/110, 3.5)` × the item's `s`, a fraction of the *grid width*, not kilometres. A per-km rule changes every map with a pack (Q6) |
| 15 | **Browse conveniences**: favourites, recently used, drag an OS file onto a slot | medium | small | none | No favourite or recent store exists (grep of `shell/`). OS drag-drop is a disclosed gap in the window header ("hit-testing the drop position against the slot grid") |
| 16 | **Scatter rules per slot** (density, variant weights) | medium | medium | library only | `ScatterRule` and `variant_weights` exist in the model (`scatter.rs`), but no `#[func]` in `lib.rs` names a scatter rule, so the shell cannot edit one. Adjacent to refinement rather than part of it |

**Recommended first cut:** ranks 1, 2, 4, 6, 8 and 9. They are all
small–medium and all format-identical, and together they make a usable
"open, check, fix, re-export" loop. Ranks 3 and 5 are the ones that make
refinement *visible* on the map. They are larger and cross into rendering, so
they wait on Q2 and Q3.

---

## 3. How other tools handle packs (public sources)

| Tool | Pack structure, as published | Workflow notes | Source, and how I read it |
|---|---|---|---|
| **Dungeondraft** | A base folder holding `preview.png` (256×320). It has `textures/objects/` (subfolders allowed), `terrain/` (seamless, "ideally 2048 x 2048"), `materials/` (`_tile.png` + `_border.png`), `tilesets/`, `patterns/normal|colorable/`, `walls/`, `paths/`, `portals/` (exactly 256 px wide) and `lights/`. It has `data/default.dungeondraft_tags` with "Tags" and "Sets", and `data/tilesets/*.dungeondraft_tileset` and `data/walls/*.dungeondraft_wall`. Assets are authored at **256 px per grid square**. Name, Author and Version are entered at packaging, and the editor writes a single `.dungeondraft_pack` file | Colourable art marks recolourable areas in "a generally reddish tone". Tags drive browsing. Loading many packs "can slow Dungeondraft or cause crashes", so users disable packs they don't need | Megasploot's *Custom Assets Guide*, github.com/Megasploot/Dungeondraft/wiki/Custom-Assets-Guide (**fetched**); 2-Minute Tabletop, *Beginner's guide to Dungeondraft custom assets*, 2minutetabletop.com/beginners-guide-to-dungeondraft-custom-assets/ (**fetched**) |
| **Wonderdraft** | A package folder under the user `assets` directory, with `sprites/` (symbols, mountains, trees) and `textures/`; brushes are separate. Each sprite folder carries a hidden JSON `.wonderdraft_symbols` with per-symbol name, radius, offsets and a draw mode (normal / sample_color / custom_colors / custom_colors_2). Fill textures must be seamless PNG | Sellers ship each asset in normal, custom-colour and sample-colour versions, so users can recolour on the map | community mirror of the r/wonderdraft wiki *symbol metadata* page (**search summary only**: the mirror returned 403 and old.reddit could not be fetched); fantasymapassets.com FAQ and a DeviantArt guide (*Tolkien-style Map Brushes*) (**search summary only**) |
| **Inkarnate, Dungeon Alchemist, DungeonFog, Canvas of Kings** | not established | — | Inkarnate's page returned only a redirect stub; I did not research the other three. **Nothing about them is claimed here** |

**What users expect**, synthesised from the two fetched sources and from
Cartalith's own design canvases rather than from user research:

- browse by tag and by set;
- search;
- recolour on the map;
- turn packs on and off, for performance;
- one pixel scale per grid unit, so packs mix;
- seamless ground textures.

Favourites, recently used, scatter brushes and "smart tiling" (Dungeondraft's
smart tilesets) were listed in the brief. I found no published source for
them in these tools beyond the tileset sizes, so they are design judgements
here, not citations.

**What an import adapter would need:**

- **Wonderdraft folder.** Walk `sprites/<folder>/*.png` into a `custom` set
  named after the folder. Read `.wonderdraft_symbols` only for display names;
  radius and offsets have no Cartalith field. `textures/` PNGs are candidates
  for ground slots, picked by the user.
- **Dungeondraft, unpacked.** Map `textures/objects/**` into `custom` sets.
  Map `textures/terrain/*` into ground-slot candidates, downscaled from 2048
  to 512 with a recorded note. `default.dungeondraft_tags` "Tags" become
  library tags. Scale: Dungeondraft's 256 px is one *battle-map square*, and
  Cartalith's 256² sprite is one *map icon*, so there is no meaningful
  automatic size conversion.
- **Not proposed:** reading a packed `.dungeondraft_pack`. Its container is
  the editor's own, and I did not verify its format. Bundling or
  redistributing anyone's art is also not proposed (§5).

---

## 4. Where it lives in the UI

**The rule this follows.** `UI_SHELL_DESIGN.md`: a dropdown that opens a
window is a shortcut into it, never a second implementation. The `Asset pack ▸`
submenu's nine rows are an owner ruling (2026-09-05 item 4), so this proposal
does **not** grow it.

- **Assets menu**: three new rows between `Import asset pack .zip…` and the
  first rule.
  - `Open pack in library…` (AP-1). It is distinct from today's *Import asset
    pack .zip…*, which applies a pack to the map. The tooltips must say which
    is which (Q1 asks whether to merge them).
  - `Pack report…` (AP-2): opens the Library at its Report view.
  - `Preview pack on map…` (AP-5).
- **The Library window gains two things:**
  - a **REFINE** section in the inspector, under Tags: Fit all, Trim, Pad,
    Flip variant, Tint variant, Outline, Shadow, Make seamless (textures
    only), Harmonise to Look. Each acts on the focused slot, or on the
    selection when BATCH is active, and each writes a *new* variant by
    default, so the original survives (Q7);
  - a **Report** view, a fourth column state beside the grid with a filter
    chip per finding kind. A row selects its slot.
- **World ▸ Import ▸** gains `Assets from a folder…` (AP-8), routed into the
  Library, the way `data_manager_window.gd`'s `import_assets` already routes.
- **File ▸ Export ▸** keeps its existing Asset pack route. No new export row
  is needed, because a refined pack exports through the same Export pack .zip.
- **Tablet.** The REFINE section lives in the inspector, which is already a
  column. The menu rows go into ☰ ▸ Assets.
- **Phone.** The inspector is a pane (PH-12). REFINE rows become a scrolling
  list with 48 dp targets. Make seamless, Harmonise and Report run in Rust and
  return a thumbnail, so no image work runs in GDScript.

---

## 5. Risks

- **Licensing.**
  - Export writes "CC0" when the licence is blank (§1.4). A refined pack that
    started as someone's paid art would ship mislabelled. AP-2 must flag it,
    and Q8 asks whether to stop defaulting at all, which would diverge from
    the reference.
  - Per-slot licence and source exist only in `library.json` and are lost on
    export (§1.1).
  - Import adapters must never fetch, bundle or redistribute third-party art.
    They read files the user already has. Many commercial Wonderdraft and
    Dungeondraft packs are licensed for use inside those tools, so the adapter
    shows the pack's licence text, if the folder has one, and stores it in
    `SlotMeta.license`.
- **Memory.**
  - The bake sizes are fixed. 35 ground slots × 512² × 4 B = 35 MiB, and 44
    frozen sprite slots × 256² × 4 B = 11 MiB, at one variant each.
  - The session also keeps every **original** at its source size
    (`AssetLibrarySession::images`), and a 4096² import is 64 MiB.
  - A merged pack of a few hundred variants is fine on desktop and tight on
    the 6T phone, whose generation peak is already measured near 486 MiB on
    desktop (`MEMORY_OPTIMIZATION_SCOPE.md`).
  - The refine tools should work on the baked size, not the original, and
    the Report view should page its thumbnails.
- **Performance.** Seam checks, dHash and outlines are O(pixels) per item:
  256² or 512², a few ms each in Rust (not measured). A whole-pack report at
  hundreds of variants must run off the main thread. The precedent is
  `lod_worker.rs`.
- **Golden drift.** Anything in AP-3 and AP-5 that changes how the map draws
  pack art changes rendered output. It ships behind a switch, `false` in
  `WorldParams::defaults()` and `true` in the app's, per `MISTAKES.md`'s
  "Change generated output" row. The pixel tools change pack *content* only,
  which moves no golden.
- **Format creep.** Every "it would be nice if the pack carried…" (tags,
  per-sprite anchor, licence per slot, scale) breaks Ruling F. Each one is
  listed as an owner question rather than designed in.

---

## 6. Milestones

The numeric bars are acceptance thresholds. They are **not** measurements.
Each is checked on the reference pack fixture plus at least two
owner-supplied or generated packs.

| Id | What | Acceptance |
|---|---|---|
| **AP-1** | Open a pack in the library: an `as_import_pack(bytes)` over `read_pack_entries`, `parse_pack_entries`, `decode_png_managed` and `add_item`, following the reference's `importPackZip` (unknown slots go to a Custom/Imported set with a warning) | (a) `reference_pack.zip` opens to exactly 17 items in 12 slots, plus name, author and licence; (b) re-exporting it unchanged gives a `pack.json` with the same entries, and every PNG decodes to identical RGBA bytes; (c) a pack with an unknown slot id gives one warning per unknown slot, and the import still succeeds |
| **AP-2** | The pack report: a Rust `pack_report(db) -> Vec<Finding>` and the Report view. `library::run`'s warnings are kept verbatim and first, in their golden order; the new findings come after | Every finding kind has a fixture that fires it and one that does not: blank licence, slot not drawn on the map, source < bake size, non-square, opaque sprite, seam score > threshold, colour converted, empty family. Zero false positives on `reference_pack.zip` apart from the expected "identical images" |
| **AP-3** | Draw `settlement`, `poi`, `seamarks` and `custom` art as screen-space sprites through the trait-badge path, falling back to today's glyphs | With a pack applied, every placed icon of a filled slot draws its sprite. On-screen size matches today's glyph within ±1 px. With no pack, `map_overlay` draws byte-identically to before (framebuffer diff = 0). It ships behind a switch (Q2) |
| **AP-4** | Seam checker and make-seamless (offset by half, then cross-blend over a band of w/8) | The seam score (mean absolute RGB difference across the wrap edges, compared with the same measure across the centre lines) of a made-seamless tile is ≤ 1.25× its interior baseline. A tile already seamless by that score is left untouched (byte-identical) |
| **AP-5** | Zoom preview and the feature-icon pass. **Measure first:** capture ×1, ×4, ×16 and ×64 with a pack applied, and record icon edge sharpness. **Then** draw feature icons at screen or tile resolution | (a) the measurement is committed with frames; (b) after: at ×16 and ×64, an icon's edge (10–90 % luminance rise) spans ≤ 2 screen pixels, against today's figure from (a); (c) at ×1 the base map with icons differs from today's by ≤ 1 % of pixels (placement unchanged). Behind a switch; needs Q3 |
| **AP-6** | Near-duplicate finder (dHash 64-bit, also tested flipped) | On a fixture of 10 sprites × {flip, ±10 % tint, 1 px shift}, every pair is grouped (Hamming ≤ 10), and no pair among 10 distinct sprites is |
| **AP-7** | Refine verbs: fit all, trim, pad, flip, tint jitter, outline, shadow and harmonise-to-Look, each writing a new variant | Each verb is undoable, and the original variant's bytes are unchanged afterwards. Harmonise: the mean hue of the result is within 10° of the target palette's nearest swatch, and alpha is untouched (byte-equal alpha channel) |
| **AP-8** | Folder import adapters (loose PNGs, a Wonderdraft folder, an unpacked Dungeondraft tree) into `custom` sets and ground candidates | A synthetic tree in each layout (made for the test, with no third-party art) imports with the expected set names and item counts. Licence text, if present, lands in `SlotMeta.license`. Nothing outside the chosen folder is read |
| **AP-9** | Pack diff and merge, plus favourites and recent | The diff of a pack against itself is empty. Merging A into B with "keep B" gives B byte-identical. Favourites and recent persist across a restart. The phone pane shows them |

**Order:** AP-1 → AP-2 → (AP-4, AP-6, AP-7 in parallel) → AP-8 → AP-9.
AP-3 and AP-5 are independent of that chain and gated on Q2 and Q3.

---

## 7. Owner questions, with recommended defaults

| # | Question | Recommended default |
|---|---|---|
| Q1 | Should *Import asset pack .zip…* also open the pack in the library, as the reference's header import does (`_alImportPackZip`), or should *Open pack in library…* stay a separate row? | **Do both, as the reference does:** loading a pack also fills the library. Keep the separate row for opening without applying |
| Q2 | Draw settlement, POI, sea-mark and custom pack art on the map (AP-3)? It changes what a map with a pack looks like | **Yes**, behind a switch, on in the app |
| Q3 | Move feature icons out of the baked grid raster into a sprite pass that stays sharp at zoom (AP-5)? It changes rendered output and export | **Yes, after AP-5's measurement**, behind a switch; export follows the screen |
| Q4 | Harmonise by baking pixels (portable), or have sprites follow the Look at render time (not portable, changes rendering)? | **Bake.** It keeps Ruling F and the HTML app's view of the pack identical |
| Q5 | Per-sprite pivot or anchor needs a format key. Add one, or bake it as pan? | **Bake as pan**; no format change |
| Q6 | Size sprites per kilometre instead of per grid width? | **No for now.** Revisit with AP-5's frames |
| Q7 | Should refine verbs write a new variant (non-destructive) or replace in place? | **New variant**, with a one-click "replace original" |
| Q8 | Keep the reference's blank-licence → "CC0" export default? | **Keep the byte behaviour**, but block export behind a confirmation when the licence is blank (a shell-side check; the writer is unchanged) |
| Q9 | Carry per-slot tags and licence in `pack.json`, as an additive key like `seamarks`? | **No.** Keep them in `library.json`; revisit if packs start being shared between users |
| Q10 | Import adapters for Wonderdraft and Dungeondraft folders (AP-8): build them, or leave packs to manual import? | **Build the loose-folder adapter first**; the two named layouts only if you use them |

---

## 8. What was checked, and how

- **Symbols opened today:**
  - `cartalith-assets`: `lib.rs`, `manifest.rs::RawManifest`,
    `library.rs` (`SlotMeta`, `ItemTransform`, `PackInfo`, `run`,
    `duplicate_groups`), `raster.rs::item_hash` and `slicer.rs`'s public
    functions;
  - `cartalith-godot`: `asset_bridge.rs::export_pack_bytes` and its module
    doc, `pack.rs` (`LoadedPack`, `composite_map_icons`, `blit_sprite`),
    `render.rs::splat_sample`, `lib.rs` (`load_asset_pack`,
    `build_color_texture`'s icon block), `export_raster.rs`'s splat block,
    `lod_worker.rs` and `lod_bridge.rs` (grep for icon and pack terms);
  - the shell: `map_overlay.gd`'s `ICON_FAMILY_COLORS` comment,
    `asset_library_window.gd` (header, `FAMILIES`, `colour_note_for`, the
    batch verbs, the footer), and `menus.gd` (`_assets`,
    `_build_asset_pack_submenu`, `build`);
  - the reference: v2.10 lines 26973, 27067 and 27933.
- **Read with Python today:** the fixture's entries and IHDR sizes.
- **Captures:** four PNGs, 84–149 KB each, read back after saving.
- **Not done:** no zoom sweep; no timing; no export with a pack; no reading
  of Wonderdraft's or Dungeondraft's own binaries. Inkarnate, Dungeon
  Alchemist, DungeonFog and Canvas of Kings were not researched.

---

## 9. Line art to proper assets: research (2026-10-18)

**The request.** The owner asked for better methods from PhotoCraft
(`github.com/storytold/photocraft`; its `NOTICE` names the ArtCraft Team, so
"the art craft github" is this repository), and said: *"be conscious about
what might be needed to convert LINE ART to proper assets. In the html
version the lines weren't solid enough and often it became a blobby mess."*

The data, the method table, the PhotoCraft comparison and the citations are
in `LINEART_METHODS_RESEARCH_2026-10-18.md`. The frames and the generator are
in `cartalith-native/docs/evidence/lineart-2026-10-18/`. This section holds
the recommendation, the milestone and the owner questions. Every number below
was produced today by that generator, on four synthetic samples:

- a JPEG pine;
- a castle with a solid door, windows and a dashed line;
- a hatched mountain;
- a bad scan of the castle.

### 9.1 Why it went wrong (read at the symbols, then measured)

| Symptom | Cause in the code | Measured at ×1 (40 px sprite) |
|---|---|---|
| lines "not solid enough" | `pack::Canvas::blit_sprite` takes **one bilinear sample per pixel** (`sample_bilinear`) while minifying 256² to `sprite_draw_rect`'s 29–49 px, so thin strokes are hit or missed. On top of that, `raster::render_item` bakes with CatmullRom on **straight** alpha | ink kept: 0.31–0.45 of the truth today. An area-filtered blit of the same 256² bake keeps 0.55–0.81; an area filter of the 1024² original keeps 0.90–1.01 (clean samples) |
| "blobby mess" | `slicer::apply_chroma` keys out a single paper colour. Darker paper, and the light edge of every stroke, stay **opaque**. Then the base raster is shown NEAREST (`viewport_host.gd::_raster`), so ×16 is 16 px blocks | scan: 22 % of the frame is opaque paper; 577 fragments against 7. Castle: holes **0 against 14**, so every window is filled. Edge width 5.8–19.6 px at ×16 and 22–77 px at ×64 |

### 9.2 The ranked method

1. **Ink extraction, at import, at source resolution (≥ 1024²).**
   - **Flatten the paper.** Divide luminance by a max-then-mean filter whose
     window is **15 % of the sprite**. A 25 px window hollowed out the castle's
     123 px solid door.
   - **Threshold with Otsu** on the flattened luminance.
   - **Remove specks** under 24 px and **fill pinholes** under 12 px (at
     1024²).
   - **Closing:** r = 1 for scans only. r = 2 merged two hatch lines.
   - Measured on the scan: IoU **0.961** and 7/7 components, against today's
     0.267 and 577. Sauvola (0.778, 70 components) and Bradley (0.824, 66)
     were **worse** than flattening plus Otsu on every sample, so they are
     the fallback, not the default.
2. **Store a signed distance field (SDF)** of the cleaned mask: the Valve
   method, one L8 channel.
   - **Resolution:** at least **2 texels across the thinnest stroke**. For
     1024² line art with 4 px hatching that means 512². The 256² copy broke
     the hatching at ×64: 15 components against 14, and 98 at 128².
   - **Mip levels** are built from the mask, read at no more than about 4
     texels per output pixel. 128² serves ×1; 512² read at ×1 aliases, with
     4.6–8.6 px edges.
   - **Size:** 512² is 10.8–21.0 KB and 128² is 3.1–3.9 KB as PNG. Today's
     256² RGBA bake is 15.9–51.8 KB.
3. **Render with a 1-pixel ramp and a stroke floor.**
   - The floor is `alpha = clamp(d_out + δ + 0.5)`, with
     `δ = max(0, (1 − w_p2) / 2)` output pixels, where `w_p2` is the asset's
     2nd-percentile stroke width.
   - **×1:** solidity **0.72–0.75** (today 0.09–0.15); stroke p5 **1.00 px**;
     edges 1.42–1.83 px.
   - **×16 and ×64:** components **exactly** the target's on all four
     samples; edges **0.55–0.79 px** (today 5.8–77; a 1024² coverage raster
     2.5–6.6 at ×64); ink mass 0.98–1.03; halo 0.
   - **The cost:** at ×1 the tone darkens 2.7–4.4×, and hatching spaced under
     about 2 output pixels becomes a solid band (`grid_mountain.png`). No
     method resolves 1 px-spaced hatching at 40 px; the choice is a faint
     tone or a solid one (Q-L1). A gap-aware floor was tried and **failed**
     in its first form (research file, §4.2).
4. **Colour.** RGB is one ink colour picked from the Look palette and baked
   into the variant (Ruling CQ **Q4**). No paper RGB survives, so the halo is
   0 by construction.
5. **The pack stays Ruling F.** The exported PNG is the coverage bake at the
   family's 256², rendered **from the SDF with the floor tuned for 256**. The
   HTML app therefore gets a cleaner sprite through the same format.

**Not recommended, with the evidence:**

- **A coverage raster alone** (local colour-to-alpha plus an area filter). It
  is right at ×16, but faint at ×1 (solidity 0.20–0.35). It keeps scan noise
  (55–61 components against 7) and blurs at ×64 (2.5–6.6 px).
- **Skeleton re-stroking or a medial-axis field.** It broke the hatching at
  ×16 (164 components against 14) and rebuilt the solid door wrongly. It is
  kept only as an optional "even line weight" edit at source resolution.
- **Potrace.** It is GPL-2.0+ and cannot be linked into this MIT/Apache-2.0
  workspace. PhotoCraft's `vector/src/trace.rs` is permissive and could back
  an SVG export later (Q-L5). It was not tested.
- **MSDF.** Untested. The single-channel SDF rounds corners by about one
  texel, which is slight on the castle at ×64.

**From PhotoCraft** (MIT OR Apache-2.0, the same licence as this workspace,
so code may be reused with its notice kept; no art; the trademarks are
excluded by its `NOTICE`), these building blocks are worth reusing or
re-deriving:

- `color_to_alpha.rs`: `opacity` and `remap`;
- `selection/distance.rs`: an exact Felzenszwalb–Huttenlocher EDT. Ours,
  `cartalith_civ::build_coast_sdf`, is a jump flood that is approximate
  beyond one seed;
- `compose/src/effects.rs`: `edge_offset`, a sub-pixel signed distance from
  an anti-aliased pixel. It is a better SDF seed than the lab's 2×
  supersample;
- `resample.rs`: premultiplied resampling;
- `vector/src/raster.rs`: exact-area coverage;
- `scancrop.rs`: Otsu with components.

PhotoCraft has **no** Sauvola or Bradley, no thinning and no SDF asset output
(grep, `9185b64b`).

### 9.3 New milestone

| Id | What | Acceptance (numeric bars, checked on the four-sample set regenerated as Rust fixtures, plus at least one owner-supplied scan) |
|---|---|---|
| **AP-10** | **Line-art conversion**, a REFINE verb. It is `cartalith-assets::lineart`: flatten → Otsu → despeckle → pinhole fill → optional close r ≤ 1 → exact EDT → SDF mips (top level sized by the 2-texel rule) → ink colour. It writes a **new variant** (Ruling CQ **Q7**) whose PNG is the 256² floor-tuned coverage bake. The SDF is a **derived cache** rebuilt from the original kept in the project (Ruling AP), so it adds no save or pack key | **Bars, the measured value in brackets:** (a) on a scan, binarised IoU against truth ≥ 0.95 with the component count equal (0.961, 7/7); (b) at ×1, solidity ≥ 0.65 (0.72–0.75) and stroke p5 ≥ 1.0 px (1.00); (c) at ×16 and ×64, components equal to the floored target (4 of 4 samples) and holes ≥ the target's − 1 (castle 14/14, scan 14/15); at ×4, components ≤ the target's and holes ≥ the target's − 1; (d) edge width ≤ 2.0 px at ×1 (1.42–1.83), ≤ 1.5 at ×4 (0.88–1.35), ≤ 1.0 at ×16 and ×64 (0.55–0.79); (e) halo 0; (f) ink mass 0.95–1.05 at ×16 and ×64 (0.98–1.03); (g) stored SDF ≤ 32 KB per sprite (14–25 KB for the 512² + 128² levels); (h) **deterministic:** two runs give byte-identical SDF and PNG, pinned by a golden hash per fixture (integer morphology, an exact EDT and a fixed quantiser; no ML, no randomness) |

**Changes to existing milestones:**

- **AP-5** (Ruling CQ **Q3**). The sprite pass that stays sharp at deep zoom
  is the SDF path. It draws a line-art variant as a screen-space quad with a
  canvas shader: one bilinear fetch, then a clamp ramp of one screen pixel
  using `fwidth`. It is the measured answer to "keeps quality at ×16 and
  ×64". Painted, non-line sprites take a premultiplied, mipmapped raster
  path instead, with the variant's line-art flag deciding which. AP-5(b)'s
  "≤ 2 screen px" bar tightens to **≤ 1.0 px for line art**.
- **AP-5's measure-first step** has a predicted baseline from this lab for
  the composite path: 5.8–19.6 px edges at ×16 and 22–77 at ×64. The in-app
  sweep must still be captured.
- **Independent of Q3: two cheap fixes to today's path** (AP-7 territory,
  since they change pixels).
  - `blit_sprite` should area-filter (premultiplied) rather than point-sample:
    ink kept goes from 0.31–0.45 to 0.55–0.81.
  - `render_item` should premultiply before its CatmullRom resize.
  - Both change rendered output, so each needs its own switch and
    disclosure, per `MISTAKES.md`'s "Change generated output" row.
- **AP-2 (pack report)** gains a finding: *"sprite looks like line art on an
  opaque or keyed background"*, which suggests AP-10.
- **Order:** AP-10 after AP-1, in parallel with AP-4, AP-6 and AP-7. AP-5's
  SDF pass consumes AP-10's cache.

### 9.4 Owner questions, with defaults

| # | Question | Recommended default |
|---|---|---|
| Q-L1 | At map scale (×1), should thin lines be **solid** (floored to 1 px, darker, with close hatching merging into a tone) or **faithful** (true tone, fainter)? | **Solid**, because that is the complaint. Add a per-variant "line weight at map scale" control from 0.5 to 1.5 px (default 1.0) for art that should stay light |
| Q-L2 | Default threshold: flatten + Otsu (measured best) or Sauvola (the textbook choice for uneven paper)? | **Flatten + Otsu**, with Sauvola as a manual option. Revisit with the owner's own scans |
| Q-L3 | The SDF is a derived cache rebuilt from the original kept in the project. Is the import-time cost (not measured in Rust) acceptable, or should the SDF be saved in `library.json`'s blobs? | **Rebuild, not save.** It adds no format, and it is deterministic |
| Q-L4 | A pack re-opened from a `.zip` has only 256² sprites, which caps SDF quality (strokes of 1–2 px). Should export also accept a larger bake (512² or 1024²) for line-art variants? That is a size change, not a key, but the HTML app's 256² families would downscale it | **Keep 256²** (Ruling F). The project keeps the original; a pack is a portable copy |
| Q-L5 | Vector tracing to SVG (PhotoCraft's `trace.rs` design, permissive) as a later export? | **Later, not in AP-10.** The SDF already meets the ×64 bar |
| Q-L6 | Should the conversion also offer "even line weight" (skeleton re-stroke at a chosen width) as a stylistic verb? | **Yes, optional and off by default.** It measured worse as storage, but it is a legitimate look for icon sets |

---

## 10. Settled while building (2026-10-18)

What building AP-1 decided or found, so the next milestone does not have to
rediscover it. Status is in `cartalith-native/docs/STATUS.md`, not here.

### AP-1: opening a pack into the library

- **Where it lives.** `cartalith-assets::pack_import` (`read_pack_entries_limited`,
  `check_png_header`, `plan_pack_import`) does everything that needs no
  pixels; `cartalith-godot`'s `AssetLibrarySession::import_pack` decodes
  (`decode_png_managed`, so ET-E2's colour conversion applies) and writes the
  store. The `#[func]` is `WorldGen::as_import_pack(bytes, source, mode)`.
- **Q1 as built.** *Import asset pack .zip…* loads the map (`load_asset_pack`,
  unchanged) and then opens the same bytes into the library; *Open pack in
  library…* (Assets menu, directly below it) does only the second. Both share
  `app.gd::_import_asset_pack`.
- **Routing follows the reference's raw-manifest walk**, not the renderer's
  validator: an unknown id in a frozen section goes to a custom slot in the
  `Imported` set with the reference's own warning text; items are named after
  their slot id; bottom-anchored families are fitted on intake. **Sea marks
  are imported too** (the reference predates them; the exporter writes them).
- **Never overwrite unasked.** The reference replaced single-image slots and
  appended to multi-variant ones, so importing a pack twice doubled its icons.
  Here a filled slot, or a pack name, author or licence that differs from the
  library's, is a conflict: a dry run lists them and the shell asks
  *Replace* / *Cancel*. Replace empties a conflicting slot before filling it,
  so a second import of the same pack leaves the same items.
- **Provenance** goes into existing `library.json` fields only:
  `SlotMeta::source` = `"<file> (<section>/<id>)"` and, when the pack has
  one, the pack licence into `SlotMeta::license`, each only where the slot's
  own field is blank or the slot was replaced. No save key was added.
- **Caps (labelled judgements, `ImportLimits::DEFAULT`).** Archive 256 MiB;
  4 096 entries; 64 MiB per entry and 512 MiB in total, decompressed, read
  through `take()` so a size header that lies still stops at the cap; manifest
  4 MiB; images ≤ 8 192 px a side and ≤ 4 096² pixels, checked from the IHDR
  before any decode. An entry name that is absolute, names a drive, carries a
  control character or climbs with `..` refuses the whole pack. A non-PNG,
  oversize or undecodable image refuses only its own row, with a reason.
- **The renderer's reader is not capped**; `load_asset_pack` is unchanged
  (switch-off identity). Instead the shell runs the library's capped dry run
  first and does not call `load_asset_pack` on a pack it refuses. Per-image
  IHDR caps apply to the library path only.
- **Finding: the bake is not a fixed point for soft edges.** `render_item`
  composites with `imageops::overlay` onto a transparent canvas; for any
  alpha strictly between 0 and 255 that moves RGB by one level on every bake
  (measured 1, 1, 1 over three export/import cycles), and it zeroes RGB under
  alpha 0. So export → open → export is byte-identical only for sprites with
  binary alpha (the reference fixture, and the round-trip test's synthetic
  pack). Pinned by `the_bake_drifts_semi_transparent_rgb_by_one_level`;
  the fix belongs with Ruling CR's switched `render_item` change.
- **Library pixels are not in the project save yet.** Ruling AP (2026-09-23)
  ruled that they should be and called it "real save-format work"; this
  file's header read as if it were done. Today
  `project_bridge.rs::asset_library_document_json` writes `library.json`
  (slots, metadata, collections, rules) and its own doc states that item
  pixels are not written and a restore brings back no items. So a pack
  opened into the library survives a save only as metadata, and AP-1 does
  not change that (save format unchanged).

### AP-2: the pack report and Q8

- **Where it lives.** `cartalith-assets::pack_report` (findings, `seam_score`,
  `SEAM_RATIO` 1.25 = AP-4's own bar reused as the detection threshold,
  `SEAM_FLOOR` 2 levels, a labelled judgement) is pixel-only and read-only;
  `AssetLibrarySession::pack_report` adds colour findings from the
  `ColourReport`s it now keeps per item (keyed by content hash, session
  only, never saved). Bound as `WorldGen::as_pack_report`.
- **Order.** `library::run`'s warnings first, verbatim; then a blank
  licence; per slot in store order: blank, opaque sprite, undersized,
  non-square (a warning for ground tiles, which letterbox onto black; a note
  for sprites, which pad), seam (ground tiles, measured on the bake), and
  "not drawn"; then exact duplicates (one row per group, with a slot to jump
  to, so they appear twice: once as Validate's line, once jumpable); then
  colour notes; then empty frozen families (only when the library holds
  something).
- **"Not drawn" names five cases plainly**: settlement, POI, sea-mark and
  custom art (`LoadedPack` has no field for them; the map draws its own
  glyph) and the `parchment` splat texture (parsed, never decoded). AP-3 is
  what will draw the first four.
- **Not in AP-2:** near-duplicates (AP-6) and the line-art finding §9.3 asks
  for (it needs AP-10's detector). The report card has no filter chips; it is
  a modal card with a Show per slot row (the inspector foot's *Report* chip,
  and *Assets ▸ Pack report…*, which is how a phone reaches it).
- **Q8 as built.** The window's *Export pack .zip* (and the menu's, which
  runs the same handler) asks *Export without a licence* → *Cancel* /
  *Export as CC0* when the licence is blank. The bytes are unchanged:
  pack.json still says "CC0". A licence being typed counts: the field is
  committed first only when it has focus, so a closed window's stale fields
  never overwrite metadata set from *Pack metadata…*.
- **Fixed on the way:** `engine_bridge.gd::as_export_pack_bytes` answered
  "not available on this binary" while generating; it now gives the busy
  text like its siblings (`_gsw_probe` expectation updated). On a phone, a
  card wider than `MODAL_W` ran off the screen, and on desktop an
  autowrapped card ballooned to the full window height; both are handled
  for these cards only (`_card_width`, `_settle_card`), not in
  `DccWidgets.modal_card`, which other cards share.
