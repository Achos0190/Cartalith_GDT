# DCC control index: every control in the spec, against what this program can do

Owner's request, verbatim: *"Before implementing the GUI I want you to properly
index all functions and buttons in the design and compare it to the current
program and functionalities."* This is that index. It writes no application
code and recommends no implementation order — that comes after the owner has
read it.

**Design source**: `DCC_SHELL_SPEC.md` (imported `116cbcb`), `UI_SHELL_DESIGN.md`
(the rule set), `design/Cartalith DCC Shell.dc.html` (the mockup — **ten**
screens, not the nine the spec's own table lists; the tenth is
`Phone inset rules`). Organised by the spec's own section numbering, one row
per control.

**All UI work is on hold** (owner, 2026-08-18; `DCC_SHELL_SCOPE.md`). Nothing
here is a plan to build; it is a map of the distance between the design and
the program.

---

## Method, and what each status means

Read directly, not inferred from a summary:
`cartalith-native/crates/cartalith-godot/src/lib.rs` in full (the **complete**
`#[func]` surface — 38 methods on `WorldGen` plus `WalkingSkeleton::ping`, and
**no other class in the workspace exposes anything to GDScript**: `render.rs`,
`params.rs` and `pack.rs` carry no `#[func]` at all);
`cartalith-native/godot-project/main.gd` (2 358 lines) and the `main.tscn` node
tree; `GENERATION_PARAMETERS.md`'s 58-key table against
`cartalith-godot/src/params.rs`'s groups; `GUI_FEATURE_PARITY_SCOPE.md`;
`FUNCTIONAL_CONTRACT.md`; `UNIFIED_TOOL_PLAN.md` (milestones A–E2, all
engine-side, all done; **F, the shell wiring, is not built**);
`cartalith-native/docs/STATUS.md`; the tool-system crates as they exist today
(`cartalith-terrain::sculpt`, `cartalith-spatial::{pass,staleness,paint,measure,region,geo}`,
`cartalith-engine::{staleness,sculpt_commit,region_export,geojson}`,
`cartalith-civ::{tools,labels}`, `cartalith-assets::{manual,library,archive,raster}`,
`cartalith-io::{tiles,gzip}`); `cartalith-gpu`'s adapter handling;
`MARKDOWN_VAULT_INTEGRATION.md`; `LOD_TILING_BASE_SCOPE.md`;
`TERRAIN_APPEARANCE_SCOPE.md`; `ASSET_LIBRARY_SCOPE.md`;
`JOURNEY_PLANNER_SCOPE.md`; `ECONOMY_SCOPE.md`; `PHASE2_SCOPE.md`;
`URBAN_MORPHOLOGY_SCOPE.md`; and the mockup's own DOM text for all ten screens.

| Status | Means |
|---|---|
| **wired** | A control exists in `main.gd`/`main.tscn` today and drives a real engine call. The control's *home* may move under this design; the working part does not have to be rebuilt from nothing. |
| **backed, unwired** | Correct, tested Rust exists today — either an existing `#[func]`, or a crate function needing only a boundary wrapper mirroring an existing pattern (`set_sea_level`'s shape). No GUI drives it. |
| **engine gap** | Real Rust work beyond a boundary wrapper. The subsystem is named, and its size cited from a scope document where one exists. |
| **new** | No reference precedent and no engine backing — a design invention. Includes pure chrome (layout, theme, gesture rules) that needs no engine at all; those rows say so in Notes. |

**The boundary matters more than usual here.** Milestones A–E2 of
`UNIFIED_TOOL_PLAN.md` built a large, golden-verified tool engine — the sculpt
stamp registry, the pass buffer, the staleness graph, manual settlement/way/
territory tools, labels, icon stamps, measure, region export, GeoJSON writing.
**None of it has a `#[func]`.** So a great many rows below read "backed,
unwired" where the backing is real and only the wrapper is missing. That is a
different, much cheaper problem than an engine gap, and this document keeps the
two apart deliberately.

Where a control could not be settled confidently it is marked **uncertain** in
Notes rather than guessed.

---

## 1 · Frame geometry

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Menu bar, 34 px | §1 | — | none (chrome) | wired | The region exists (`Shell/TopBar`), but with **eight** menus of different content. See §2. |
| Tool options bar, 34 px | §1 | — | none (chrome) | wired | `Shell/ToolOptionsBar` exists; shows only the active tool's name plus an honest "not implemented" hint. |
| Domain rail, 40 px collapsed / 200 px expanded | §1, §3 | — | none (chrome) | new | The five domains exist today as a **horizontal** `WorkspaceTabsBar`. Same five subjects, new geometry. |
| Left dock, 372 px (min 300, max 520) | §1, §5 | — | none (chrome) | new | No left dock exists. The current shell's left column is a 40 px tool rail of 16 inert tools. |
| Viewport | §1 | — | `build_color_texture` | wired | `Shell/MainRow/ViewportArea`, real map rendering. |
| Right dock, 284–340 px (min 260, max 460) | §1, §6 | — | none (chrome) | wired | Exists as Layers / Properties / Sample. Contents differ — see §6. |
| Timeline bar, 70 px | §1, §10 | — | none | new | No timeline region exists in `main.tscn`. Backed by nothing — see §10. |
| Status bar, 26 px | §1, §10 | — | none (chrome) | wired | Exists with four slots: status, autosave, tile cache, hint. Three of the four report placeholder text. |
| Viewport never scrolls; docks scroll independently | §1 | — | none (chrome) | wired | `RightDockScroll` is already a `ScrollContainer`; the viewport does not scroll. |
| Dock collapse via `‹` / `›`, keeping the primary readout | §1, §6 | — | none (chrome) | new | Pure GDScript. `GUI_SHELL_SCOPE.md` deferred panel collapse/rails once already. |
| One modal at a time; modals are children of their window | §1 | — | none (chrome) | new | The current shell uses top-level `AcceptDialog`s parented to `Main`, not to a window. |
| Menus open on click, close on outside-click or `Esc`; overlay, never push layout | §1 | — | none (chrome) | wired | Godot `MenuBar`/`PopupMenu` behaviour, already correct. |
| Tablet 2560 / phone 393 geometry | §1, §12 | — | none (chrome) | new | Responsive breakpoints deferred by `GUI_SHELL_SCOPE.md` and unchanged since. `ANDROID_BUILD_SCOPE.md` holds real measurements. |

---

## 2 · Menu bar

**Structural note before the rows.** The current shell has **eight** menus —
File · Edit · **Generate · Simulate · Render** · Assets · **View** · Help. The
spec has **seven** — File · Edit · **Assets · Data · Preferences · Window** ·
Help. Generate, Simulate, Render and View are deleted as menus and their
contents redistributed to the domain rail (§3) and Preferences (§2.5). Four
menus of real, working GDScript are affected; what happens to each is itemised
in summary §4.

### 2.1 File

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| New world… ⌘N | §2.1 | — | `generate_sized`, `generate_world_structure_sized`, `set_sea_level`, `set_params`, `reference_grid_height`, `get_map_width_km`/`get_map_height_km` | wired | The richest real thing in the shell: DCC milestone 3's World Setup dialog (extent, map width km with six presets, resolution 512–8K, aspect with seven presets, live derived grid/extent/cell-size readout). The spec's four fields (name, seed, extent, working resolution) are a **subset** of what already works — except **name**, which nothing stores. |
| — project name field | §2.1 | — | none | engine gap | Small. No project entity exists; `WorldGen` has no name field and `cartalith_io::SaveData` carries none. |
| Open project… ⌘O | §2.1 | `#loadZipBtn` | `WorldGen::load_save(path) -> bool` → `cartalith_io::load_save` | wired | `File ▸ Open project (.zip)…`, `_on_save_file_selected`. Verified: reads a real reference-produced `.zip` (MVP criterion 7). |
| Recent worlds ▸ (last 10, path as secondary text) | §2.1 | — | none | new | No project-path history exists. Pure GDScript over a settings file, but `GUI_FEATURE_PARITY_SCOPE.md` Category 3 found "Recent worlds" has **zero reference grounding** and deleted the fabricated original. The spec re-introduces it. |
| Save project ⌘S (disabled when no changes) | §2.1 | — | **none** | engine gap | **`cartalith-io` has no writer.** `load_save` is the whole module surface; `SaveData`/`SaveParams`/`SaveFields` are read-only shapes. `SAVEFILE_COMPAT.md` documents the format; writing it is real work with a byte-compatibility bar. Also needs a dirty-state model that does not exist. |
| Save as… ⌘⇧S | §2.1 | — | none | engine gap | Same writer. |
| Autosave (toggle + off/1/5/15 min, default 5) | §2.1 | — | none | engine gap | Blocked on the writer above. The status bar already has an `AutosaveLabel` slot showing placeholder text. Default interval is an owner decision. |
| Revert to last save (discards sculpt drafts too) | §2.1 | — | `load_save` + `PassBuffer::discard` | backed, unwired | Both halves are real; both need the writer to have produced something first. |
| Close project ⌘W | §2.1 | — | none | new | No project lifecycle exists; `WorldGen` holds one world for the life of the process. |
| Storage locations (read-only list of four roots) | §2.1 | — | none | new | Four roots invented by the spec (`~/Cartalith/Worlds`, `/Cache/atlas`, `/Packs`, `/Exports`). No path convention exists anywhere in this port; dialogs currently root at the project directory. Owner decision. |
| Change locations… (one picker per root; moving the atlas root invalidates the cache) | §2.1 | — | none | new | There is no atlas cache to invalidate — see §2.5 Tiles & LOD. |
| Show project on disk | §2.1 | — | none | new | `OS.shell_show_in_file_manager()` — pure GDScript, no engine involvement. |
| Static note: *imports live under Data ▸ Import, asset packs under Assets* | §2.1 | — | none (chrome) | new | **Contradicts the current shell**, where `File ▸ Import asset pack…` is the one working import. |

### 2.2 Edit

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Undo ⌘Z (global) | §2.2 | `#undoBtn` / `#undoMem` | `PassBuffer::undo` / `can_undo` (draft-scoped only) | engine gap | Two different undos. `cartalith-spatial::PassBuffer` has real draft-scoped undo/redo over the stamp list. A **global** undo across generation parameters, style edits, selections and committed passes exists nowhere — `FUNCTIONAL_CONTRACT.md` §12 calls undo "absent entirely" and notes its implementation is necessarily new. |
| Redo ⌘⇧Z | §2.2 | — | `PassBuffer::redo` / `can_redo` | engine gap | Same split. |
| Undo history… (panel; click an entry to roll back) | §2.2 | — | none | engine gap | Needs an addressable, labelled global history. `PassBuffer`'s stack is unlabelled `Vec<PassEntry>` snapshots. |
| Cut / Copy / Paste ⌘X ⌘C ⌘V (labels, icons, places, stamps) | §2.2 | — | `MapLabel`, `ManualIcon`, `NamedSettlement`, `SculptStamp` all real and cloneable | engine gap | The entities exist; a selection model and a clipboard do not. Medium — it needs one uniform selection abstraction over four unrelated types. |
| Delete ⌫ (never deletes a generation stage) | §2.2 | — | `PassBuffer::remove(index)`, `AssetDB::remove_item`, `AssetCollections::drop_uid` | backed, unwired | Deletion primitives are real per entity type; the selection model above is what is missing. |
| Select all / Deselect ⌘A ⌘D (scoped to the active layer) | §2.2 | — | none | engine gap | Same selection model. |
| Find on map… ⌘F (places, labels, factions, routes; pans the viewport) | §2.2 | — | `get_settlements()`, `get_provinces()`, `cartalith_civ::tools::civ_pick_place_at` | backed, unwired | Settlement and province names are already read into the world-data browser and already filterable there (`_wd_row_matches`). Faction and route *names* do not exist as entities — see §6. Viewport panning does not exist (no camera). |

### 2.3 Assets

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| ⧉ Asset library (⇧A) | §2.3, §8 | `#assetLibrary` | `cartalith_assets::library::{AssetDB, AssetCollections}` | backed, unwired | Phase 4 milestone 5 built the whole library model. No `#[func]` reaches any of it; `WorldGen` exposes only `load_asset_pack`/`has_asset_pack`. |
| ⧉ Sprite sheet slicer (▦) | §2.3, §8 | `#alSlicerBtn` | **none** | engine gap | Searched `cartalith-assets` for a slicer: nothing. `raster.rs` has decode/encode/resize/`render_item` but no grid-slicing function. Small-to-medium (cell grid, trim, empty-cell detection, fill policy). |
| Import image… (`image/*`, lands in *Unassigned imports*) | §2.3 | `#alFilePicker` | `cartalith_assets::raster::decode_png`, `AssetDB::add_item` | backed, unwired | **PNG only** — `decode_png` uses `image` with `png` features and no defaults. The spec says `image/*`; JPEG/WebP is a feature-flag change plus a re-decision, since `item_hash` is content-derived. |
| Import asset pack .zip… | §2.3 | `#alImportPackBtn` | `WorldGen::load_asset_pack(path) -> bool` | wired | `File ▸ Import asset pack…`, `_on_asset_pack_file_selected`. Relocating it to Assets is a menu move, not a rebuild. |
| Icon families ▸ (24 families with filled/capacity counts) | §2.3 | — | `Family` enum (**8 variants**), `AssetDB::filled_count(family)` | engine gap | **The vocabulary disagrees.** The engine's `Family` is `Textures, Biomes, Terrains, Icons, Settlement, Trait, Poi, Custom` — 8, frozen, ported from the reference. The spec's 24 (Places, Buildings, Walls & gates, Trees & cover, Rock & scree, Textures, Hachure & hatch, Compass & frame, Label plaques, Ship & sea marks, Map furniture, …) is a different taxonomy at a different granularity. The mockup's own rail shows **11** families plus 2 collections, not 24. See summary §3. |
| Texture sets ▸ | §2.3 | — | `Family::Textures`, `SPLAT_PAINT_SLOTS` | backed, unwired | The six splat channels are real and already composited by `pack.rs`. |
| Apply library to map | §2.3 | `#alApplyBtn` | `AssetDB::to_library_json`, `apply_library_file_with_items` | backed, unwired | The compile step exists; nothing wires it to the live renderer, which today only consumes a loaded `.zip` pack. |
| Clear library… (destructive, confirmed) | §2.3 | `#alClearBtn` | `AssetDB::clear`, `AssetCollections::clear` | backed, unwired | |

#### 2.3.1 Asset pack submenu

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Active pack — name | §2.3.1 | `#alPackName` | `PackInfo`, `cartalith_assets::manifest` | backed, unwired | `load_asset_pack` parses the manifest today but `WorldGen` returns only `has_asset_pack() -> bool`; no metadata crosses the boundary. |
| Active pack — author | §2.3.1 | `#alPackAuthor` | `PackInfo` | backed, unwired | Same. |
| Active pack — license | §2.3.1 | `#alPackLicense` | `PackInfo` | backed, unwired | Same. |
| Active pack — schema (`2 · STORED zip`) | §2.3.1 | — | `cartalith_assets::archive` (schema 2, STORED, timestamps frozen at 1980-01-01) | backed, unwired | Verified byte-reproducible in both directions against a reference-produced pack (Phase 4 m2; the two archives differ by 2 bytes total). |
| Active pack — filled slots (`148 of 212 · 26 MB`) | §2.3.1 | `#alStats` | `AssetDB::filled_count`, `total_items`, `pack_summary` | backed, unwired | Byte size is not computed anywhere; trivial from the store. |
| Pack metadata… (name / author / license) | §2.3.1 | — | `PackInfo` fields | backed, unwired | |
| Edit ▸ Open library workspace | §2.3.1 | — | none (chrome) | new | Window navigation. |
| Edit ▸ Import image into slot… | §2.3.1 | — | `AssetDB::add_item`, `raster::decode_png` | backed, unwired | |
| Edit ▸ Sprite sheet slicer… (cols · rows · margin) | §2.3.1 | — | none | engine gap | See above. |
| Edit ▸ Add variant to slot | §2.3.1 | `#alAddVar` | `AssetDB::add_item` (a slot holds `Vec<LibraryItem>`) | backed, unwired | Variants are the model's native shape; `pick_weighted_variant` is golden-verified. |
| Edit ▸ Replace · delete slot art | §2.3.1 | `#alReplace`, `#alDelVar` | `AssetDB::remove_item` + `add_item` | backed, unwired | |
| Edit ▸ Slot transform (scale · fit · reset) | §2.3.1 | `#alScale`, `#alFit`, `#alReset` | `ItemTransform{scale,pan_x,pan_y}`, `raster::fit_to_bottom`, `render_item` | backed, unwired | `fit_to_bottom` is golden-verified. |
| Edit ▸ Preview background (5 swatches) | §2.3.1 | `#alBgSw` | none (chrome) | new | Pure presentation; the five colours are given in the spec. |
| Batch header — live selection count | §2.3.1 | — | none | new | Selection model, GDScript-side. |
| Batch ▸ Tag… | §2.3.1 | `#alBatchTag` | `SlotMeta::tags: Vec<String>`, `normalize_meta` | backed, unwired | Tags are real, parsed, and hardened against wrong-typed JSON. |
| Batch ▸ Collect into set… | §2.3.1 | `#alBatchColl` | `AssetCollections::{add, remove, rename_uid, membership, drop_collection}` | backed, unwired | The whole collections model is real. |
| Batch ▸ Rename… | §2.3.1 | `#alBatchRename` | `AssetDB::rename_custom_slot` | backed, unwired | Renaming is defined **only for custom slots**; the frozen vocabulary's slot names are constants (`slot_title`). Renaming a `P01 Capital` is not something the model supports — a real spec/engine disagreement. |
| Batch ▸ Duplicate | §2.3.1 | `#alBatchDup` | `AssetDB::add_custom_slot` + `add_item`; `duplicate_groups`/`slot_has_dupe` for detection | backed, unwired | |
| Batch ▸ Delete ⌫ | §2.3.1 | `#alBatchDel` | `AssetDB::remove_item`, `remove_custom_slot` | backed, unwired | |
| Build ▸ Validate pack (warning count) | §2.3.1 | `#alValidateBtn` | `cartalith_assets::library::run(&AssetDB) -> Vec<String>` (`AssetValidator`) | backed, unwired | Returns the real warning strings; 32 of milestone 5's tests are golden-verified against a real reference run. |
| Build ▸ Apply to map | §2.3.1 | — | see *Apply library to map* | backed, unwired | |
| Build ▸ Import pack .zip… | §2.3.1 | — | `load_asset_pack` | wired | Same call as File ▸ Import asset pack. |
| Build ▸ Export pack .zip… ⌘⇧P | §2.3.1 | `#alExportBtn` | `cartalith_assets::archive::{write_pack, zip_store, zip_store_bytes}` | backed, unwired | Writes `pack.json` schema 2 + PNGs as a STORED zip exactly as the spec says, round-trip verified against the reference's own `zipStore`. Needs one `#[func]`. |
| Clear library… | §2.3.1 | — | `AssetDB::clear` | backed, unwired | |

### 2.4 Data — the Data Manager dropdown

Every row opens the §9 window on a route. Backing is assessed once, here; §9
carries the route panes' own controls.

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Import ▸ Maps | §2.4 | — | none | engine gap | No raster map importer exists anywhere. |
| Import ▸ Heightmaps (PNG · TIFF) | §2.4 | — | none (`raster::decode_png` decodes PNG, and only for assets) | engine gap | `FUNCTIONAL_CONTRACT.md` §9: absent. `GUI_FEATURE_PARITY_SCOPE.md` Category 2 small: *"needs its own short investigation before a milestone estimate is trustworthy."* **TIFF is a new dependency decision.** The reference also had "infer tectonics from heightmap", which the spec does not mention. |
| Import ▸ GIS / GeoJSON | §2.4 | — | **none** | engine gap | `cartalith-engine::geojson` is a **writer only** (`export_geojson`, `stringify`). Grep found no parser anywhere in the workspace. |
| Import ▸ World Data (.zip · fields) | §2.4 | — | `load_save` | wired | Reading a `.zip` save is the one import that genuinely works. Importing individual *fields* is not supported. |
| Import ▸ Assets (routes to the Assets menu) | §2.4 | — | `load_asset_pack` | wired | A link, not a route. |
| Export ▸ Maps (image · tiles) | §2.4 | — | `cartalith_engine::region_export::{export_region_tiles, tile_png_bytes, zip_region_export}`, `cartalith_terrain::tile_render::render_height_tile_rgba`, `cartalith_io::{tiles, gzip}` | backed, unwired | Milestone E2 built the whole encoding pipeline: per-tile PNG, gzip, the `.zip` assembly, a byte-exact tile manifest. But it exports a **region selection in the reference's own scheme**, not a Leaflet XYZ/TMS/WMTS pyramid — see §9. |
| Export ▸ GIS / GeoJSON | §2.4 | — | `cartalith_engine::geojson::export_geojson(&GeoJsonWorld)` | backed, unwired | Real, golden-verified as a whole-string comparison. Needs a `#[func]` plus assembling `GeoJsonWorld` from live state. The shell's `File ▸ Export GeoJSON` item is present, disabled, and carries a now-**stale** tooltip claiming "No Rust writer exists yet" — true when written, no longer. |
| Export ▸ World Data | §2.4 | — | none | engine gap | The save writer again. |
| Export ▸ Assets (pack .zip) | §2.4 | `#alExportBtn` | `archive::write_pack` | backed, unwired | |
| Sources ▸ External Sources | §2.4 | — | none | new | No concept of an external data source exists. |
| Sources ▸ Connected Sources | §2.4 | — | none | new | The mockup shows `1` = the Markdown vault. |
| Sources ▸ Source Registry | §2.4 | — | none | new | |
| Conversion ▸ Coordinate Systems (EPSG ▸) | §2.4 | — | none | engine gap | `GUI_FEATURE_PARITY_SCOPE.md` Category 3 recommends **defer**, with reasoning that still holds: Cartalith's world is a flat, non-georeferenced procedural grid with no real-world CRS to convert between. The spec re-introduces it as a first-class route. Owner decision. |
| Conversion ▸ Format Conversion | §2.4 | — | none | new | Undefined in the spec — which formats, to which. |
| Conversion ▸ Data Transformation | §2.4 | — | none | new | Undefined in the spec. |
| Validation ▸ Check Data (current warning count) | §2.4 | — | `cartalith_assets::library::run` validates a **pack**, not world data | engine gap | The mockup's `8` matches the asset-library warning count, suggesting the two are the same number. Nothing validates world data. |
| Validation ▸ Repair / Normalize | §2.4 | — | `normalize_scatter_rule`, `normalize_meta` (asset-side only) | engine gap | No world-data repair exists. What is repaired, and against what invariant, is an owner/design question. |

### 2.5 Preferences

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Performance ▸ GPU acceleration (toggle + backend readout) | §2.5 | `#gpuToggle` / `#gpuTag` | `use_gpu` param key; `WorldGen::get_gpu_stages_used()` | backed, unwired | **Readout is wired** — `View ▸ Performance readout…` lists six GPU-eligible stages, GPU or CPU each. The **toggle is deliberately excluded** (`EXCLUDED_KEYS` in `main.gd`): per `DECISIONS.md` §7c the GPU noise primitive is a different hash, so the same seed makes a different world, and `GPU_LAYER_INTEGRATION_SCOPE.md`'s current milestone is still the GPU-safe noise redesign. The spec's "Off falls back to CPU tile passes" is already true **per stage**, not as a global switch. |
| Performance ▸ Devices (per-device checklist, live utilisation, uncheck to exclude) | §2.5 | — | **none** | engine gap | Large. `cartalith_gpu::init_gpu()` requests **one** high-performance adapter; there is no enumeration, no per-device utilisation, no dispatch partitioning. `HETEROGENEOUS_COMPUTE_RESEARCH.md` is on file but its own annotation records that **no capability-tier classifier or adaptive scheduler has been built and none is scheduled**, and that the measured GPU bottleneck was per-dispatch context creation, not scheduling. |
| Performance ▸ Multi-GPU mode (`split tiles` / `alternate frames` / `single device`) | §2.5 | — | none | engine gap | Same subsystem; `alternate frames` presupposes a 3D viewport that does not exist. Owner decision before any scoping — summary §5. |
| Performance ▸ CPU worker threads (1…cores, default cores − 4) | §2.5 | — | Rayon global pool (`CPU_MULTITHREADING_SCOPE.md`, milestones 2–3 landed) | backed, unwired | Rayon's thread count is settable once at startup via `ThreadPoolBuilder`; the engine takes the default today. A thin `#[func]` plus a startup-order rule. The **default of cores − 4** is an owner decision, not a porting question. |
| Performance ▸ VRAM budget (GB, default 75 % of smallest active device) | §2.5 | — | none | engine gap | No VRAM accounting exists; `GpuContext` carries adapter name/vendor/backend only. |
| Performance ▸ Fallback when VRAM full (`CPU tile pass` / `reduce working res` / `fail with error`) | §2.5 | — | per-stage CPU fallback exists inside `generate_terrain` | engine gap | The *first* option is effectively what already happens on any GPU failure. The other two do not exist, and `reduce working res` would silently change the world. |
| Graphics ▸ Render quality (`performance · balanced · quality · ultra`) | §2.5 | — | `WorldGen::set_quality_tier(name)`, `get_quality_tier()`, `list_quality_tiers()`, `get_recommended_quality_tier()` | backed, unwired | **The four `#[func]`s already exist and the four tier names match the spec exactly.** `main.gd` calls none of them. Presentation-only by contract — the doc comment states it never touches heightmap/climate/hydrology/biome/settlements/routes/seed, so it needs no regeneration, only `build_color_texture()` again. The cheapest real control in the whole design. |
| Graphics ▸ Anti-aliasing · anisotropy | §2.5 | — | Godot project/viewport settings | backed, unwired | Godot-side, not engine. The map is a 2D `TextureRect`, so MSAA/anisotropy do essentially nothing until a 3D viewport exists. |
| Graphics ▸ Colour management (`sRGB` / `Display P3` / `linear`) | §2.5 | — | none | engine gap | `render.rs` writes 8-bit sRGB directly; `TERRAIN_APPEARANCE_SCOPE.md`'s own `Ultra` tier doc says the precision/HDR half of research §20 **is not built** and refuses to claim it. |
| Graphics ▸ 3D viewport defaults (relief exaggeration, detail, light, flatten oceans) | §2.5 | `#genV3dSec` | `TerrainAppearance::exag` exists; a 3D view does not | engine gap | 3D is deferred by `DECISIONS.md` §4; `ROADMAP.md` Phase 3 brings it back and has not yet. Four controls for a viewport that does not exist. |
| Graphics ▸ Lighting rig defaults (azimuth, elevation, ambient, multidirectional on/off) | §2.5 | — | `TerrainAppearance::{sun_az_deg, sun_alt_deg, relief_ambient, relief_gain, relief_lights, relief_directionality}` | backed, unwired | All six fields are real, tested and drive the current render. **No `#[func]` reaches any of them.** A `set_appearance(...)`-shaped wrapper is the whole gap — the same one §7's LIGHT group needs. |
| Tiles & LOD ▸ Tiled LOD (`auto on zoom` / `manual`) | §2.5 | `#lodAutoChk` | `cartalith-spatial` (`TiledField`, `QuadTree`, `DirtyTracker`) — standalone, **unintegrated** | engine gap | Large. `LOD_TILING_BASE_SCOPE.md` built the foundation deliberately wired to nothing; there is no camera, no zoom, no quadtree-driven rendering. |
| Tiles & LOD ▸ Tile size · LOD levels (256/512/1024; levels 0–8) | §2.5 | `#lodMaxLevel` | `TiledField::tile_size` is a constructor parameter; `region_export` carries its own tile size | engine gap | Same integration. |
| Tiles & LOD ▸ Atlas cache (size cap GB + Clear) | §2.5 | `#lodBakeBtn`, `#lodClearAtlasBtn` | **none** | engine gap | No atlas cache exists in any form. The status bar's `TileCacheLabel` shows placeholder text today. |
| Tiles & LOD ▸ Chunk debug overlay (`off · grid · colours`) + tile borders | §2.5 | `#lodDbgSeg` | none | engine gap | Needs the tiling to be real first. |
| Memory ▸ Undo history (1–50, default 5) | §2.5 | — | `PassBuffer` history cap is a constant | engine gap | Depends on the global undo that does not exist (§2.2). The reference's own cap is 30 and is draft-scoped. The default of 5 is a spec invention. |
| Memory ▸ Working set (read-only, `1.6 GB of 12 GB`) | §2.5 | — | Godot `Performance` singleton | wired | Already shown by `View ▸ Performance readout…` (`_perf_runtime_labels`), sourced from Godot's own singletons — `GUI_FEATURE_PARITY_SCOPE.md` predicted exactly this and it landed. |
| Memory ▸ Clear caches… (atlas + field, never project data) | §2.5 | — | none | engine gap | Nothing to clear. |
| Application ▸ Storage locations… | §2.5 | — | none | new | Same modal as File; see §2.1. |
| Application ▸ Theme (`dark` / `light` / `follow system`) | §2.5 | — | none (chrome) | new | A real dark `Theme` resource exists (`theme/dark_theme.tres`); the light theme is deferred and the toggle is present-and-disabled by design. `follow system` is new. Known open defect: `dark_theme.tres` has **no `PopupMenu`, tooltip or scrollbar entries**, so every dropdown renders in Godot's default grey (`GUI_FEATURE_PARITY_SCOPE.md` Category 4, confirmed still open). |
| Application ▸ Units (`km` / `mi`) | §2.5 | `#calUnitSeg` | `map_width_km`, `cell_km`, `Measurement::km` — everything is km internally | backed, unwired | A pure display conversion. Note it appears **twice** in the spec: here and as a control in §5.1 stage 02. One value, two homes — an ownership collision the builder must resolve. |
| Application ▸ Keyboard shortcuts… (editable, per-context) | §2.5 | — | none (chrome) | new | Godot `InputMap` — GDScript only. The current shell has a present-and-disabled `Help ▸ Keyboard map`. |

### 2.6 Window

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Toggle Left dock | §2.6 | — | none (chrome) | new | The dock itself is new. |
| Toggle Right dock | §2.6 | — | none (chrome) | new | The dock exists; a visibility toggle does not. `View ▸ Panel visibility` is present and disabled today. |
| Toggle Timeline | §2.6 | — | none (chrome) | new | |
| Toggle Status bar | §2.6 | — | none (chrome) | new | |
| Toggle Domain rail | §2.6 | — | none (chrome) | new | |
| Reset layout | §2.6 | — | none (chrome) | new | |
| Save layout as… | §2.6 | — | none (chrome) | new | Needs layout persistence — no user-settings store exists in this port at all. |
| Workspace list | §2.6 | — | none (chrome) | wired | The five workspaces exist as tabs (`_select_tab`); listing them in a menu is trivial. |
| Open windows appear here while open | §2.6 | — | none (chrome) | new | Depends on the Asset library and Data manager windows existing. |

### 2.7 Help

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Documentation | §2.7 | — | none | new | No shipped documentation target. |
| Keyboard shortcuts | §2.7 | — | none (chrome) | new | Present and disabled today as `Keyboard map`. |
| Credits & academic principles | §2.7 | `#creditsBtn` | `credits.gd` + `CreditsDialog` | wired | Real and reachable; carries the reference's attribution plus this port's license-audit findings. |
| Report an issue | §2.7 | — | none | new | Owner decision — where does it go. |
| About (version + build) | §2.7 | — | none | new | No version string is surfaced. `WalkingSkeleton::ping()` is the only build-liveness probe and nothing calls it. |

---

## 3 · Domain rail

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| WORLD domain | §3 | — | the whole generation pipeline | wired | Exists as the `WORLD` workspace tab. |
| CIVIL domain (settlements, population, economy, politics, culture) | §3 | — | `get_settlements`, `get_provinces`, `get_trade_balances`, `civ_faction_aggregates`, `civ_culture_terrain_fit` | wired | Exists as `CIVILIZATION`. Its content today lives in `Simulate ▸ Statistics…`/`Economy…` dialogs, not in a dock. Culture and faction aggregates are backed but have **no `#[func]`** (see §6 Faction). |
| INFRA domain (roads, rivers, ports, trade, logistics) | §3 | — | `get_roads`, `get_sea_routes`, Journey Planner (engine-complete) | wired | Exists as `INFRASTRUCTURE`. Journey Planner has no GUI at all (`Simulate ▸ Logistics`, present and disabled). |
| CARTO domain | §3 | — | `TerrainAppearance`, `build_color_texture` | wired | Exists as `CARTOGRAPHY`. Contents are §7 — nothing there is built. |
| RENDER domain (terrain appearance groups) | §3 | — | `TerrainAppearance` (~40 fields), `QualityTier` | wired | Exists as `RENDER`. All items inside are present-and-disabled. |
| Clicking a domain swaps both docks and the tool options bar | §3 | — | none (chrome) | new | Today a tab click restyles tool-rail emphasis only and deliberately never touches the viewport. Swapping dock **contents** is new behaviour. |
| Viewport, camera and selection persist across domain switch | §3 | — | none (chrome) | wired | Trivially true today (no camera, and the tab switch is presentation-only). |
| Rail expansion (`›`) showing sub-nodes as a 200 px list | §3 | — | none (chrome) | new | Sub-node lists per domain are not enumerated in the spec; the builder has no source for them. **Uncertain** what belongs in each. |
| Rail foot: active context (`TERRAIN`, `SCULPT`, `STYLE`) | §3 | — | none (chrome) | new | |
| Rail foot: stage counter (`04 / 10`) in the World domain | §3 | — | see §5.1 | engine gap | Requires the ten-stage model that does not exist — see summary §3. |

---

## 4 · Tool options bar

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Layout: context label → parameters → spacer → commit/discard | §4 | — | none (chrome) | new | The bar exists; it holds a name and a hint, never parameters. |
| Raise/Lower context: hardness 0.35 | §4, §5.2 | — | `SculptGlobals::hardness` (0..1, step 0.01, default **0.5**) | backed, unwired | Default disagrees: engine `0.5` (the reference's `SCULPT_GLOBAL_DEF`), spec `0.35`. |
| Raise/Lower context: intensity +120 m | §4, §5.2 | — | `SculptGlobals::intensity` (0..1.5, **unitless**, default 1.0) | backed, unwired | The engine's intensity is a dimensionless coverage-weight scale. Converting it to metres needs `peak_m`/`sea_level` and is a **presentation** layer that does not exist — `UNIFIED_TOOL_PLAN.md` names this explicitly as a Godot-side concern. |
| Raise/lower/smooth mode segment | §4, §5.2 | — | `FreehandMode::{Raise, Lower, Smooth}` | backed, unwired | Three of the engine's eight freehand sub-modes; the other five (Cliff, Ridge, Canyon, Mesa, Volcano) are real and unnamed by the bar. |
| **commit pass** | §4 | — | `PassBuffer::commit`, `cartalith_engine::sculpt_commit::commit_sculpt_pass` | backed, unwired | Fully real, including the River/Lake special path (channel descent + locking, lake deposit). Needs `#[func]`s and stroke capture — `UNIFIED_TOOL_PLAN.md` **milestone F**, the only unbuilt tool milestone. |
| discard | §4 | — | `PassBuffer::discard` | backed, unwired | |
| Generation Pipeline context: Run stage 04 | §4, §5.1 | — | **none** | engine gap | `generate_terrain` runs the whole pipeline or none of it. There is no per-stage entry point. See summary §3. |
| Generation Pipeline context: Run 04 → 10 | §4, §5.1 | — | none | engine gap | Same. |
| Generation Pipeline context: New seed | §4 | — | `generate_sized(seed, …)`, `get_seed()` | backed, unwired | Seed is a call argument, already driven by the New World dialog's `%SeedInput`. `get_seed()` exists as a `#[func]` and **nothing calls it**. |
| Generation Pipeline context: stale-from readout | §4, §5.1 | — | `cartalith_spatial::StageGraph`, `cartalith_engine::pipeline_stage_graph` | engine gap | The staleness machinery is real but models **four** stages, not ten — see summary §3. |
| Generation Pipeline context: 🔒 Bake ALL & finalize | §4, §5.1 | `#bakeAllBtn` | none | engine gap | No bake, no atlas, no finalize lock exists. |
| Sculpt context: feature · preset · radius · falloff · mode | §4, §5.2 | `#sculptFeatureSeg`, `#sculptPresetSeg` | `Feature` (13), `SCULPT_PRESETS` (8), `SculptGlobals` | backed, unwired | See §5.2 for the per-control disagreements. |
| Sculpt context: ↶ Undo / ↷ Redo | §4 | — | `PassBuffer::{undo, redo, can_undo, can_redo}` | backed, unwired | Draft-scoped, exactly as the reference's `sculptHistory` is. |
| Sculpt context: ✓ Commit to map / Discard draft | §4 | — | `commit_sculpt_pass` / `PassBuffer::discard` | backed, unwired | |
| Cartography context: preset chips (Atlas / Parchment / Physical / Ink) | §4, §7 | — | `TerrainAppearance::{default, js_reference, for_tier}` | engine gap | Two named appearances exist (`default` = "the atlas look", `js_reference` = bit-identical JS output) plus four quality tiers. **Parchment, Physical and Ink do not exist**; naming `default` "Atlas" is plausible but the other three are new looks, not presets over existing fields. |
| Cartography context: `custom — edited since preset` | §4, §7 | — | none | new | Needs a dirty-vs-preset diff over the appearance struct. |
| Cartography context: Reset | §4 | — | `TerrainAppearance::default()` | backed, unwired | |
| Cartography context: Save preset | §4 | — | none | engine gap | No preset serialization exists for appearance; `TerrainAppearance` is not `Serialize`. |

---

## 5 · Left dock · World domain

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Header switch: GENERATION PIPELINE / SCULPT, persisted per project | §5 | — | none (chrome) | new | Two-button switch, one always active. No per-project settings store exists anywhere in this port. |

### 5.1 Generation Pipeline — the ten stages

**Read summary §3 alongside this table.** The spec's ten stages are a
*dependency* ordering invented for the design; `GENERATION_PARAMETERS.md`'s
eight groups (`world, planet, world_structure, tectonics, volcanism, erosion,
climate, weather`) are the engine's own, and `main.gd`'s Generate menu uses a
third decomposition of ten *dialogs*. None of the three partition identically.

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Stage row: number, state dot, name, state label, chevron | §5.1 | — | none (chrome) | new | |
| Stage row: `needs` line | §5.1 | — | `pipeline_stage_graph`'s edges (4 stages) | engine gap | The engine's dependency graph is height → hydrology → climate → civ. The spec's ten-node graph has no engine counterpart, and stage 06↔08 is a **cycle** (06 needs 08; 08 needs 06 and, per the mockup, "feeds back into 06"). `StageGraph` is acyclic by construction. |
| Stage row: `produces … → consumed by` line | §5.1 | — | `WorldState`'s real fields | backed, unwired | The named products (`continentality.f32`, `elevation`, `plate_id`, `boundary_type`, `resistance`) are all real `WorldState` fields. None is exposed to GDScript. |
| States: `resolved` / `editing` / `stale` | §5.1 | — | `StageGraph::{is_stale, staleness, stale_stages}` | engine gap | Real for four stages, at **tile** granularity, and only after a committed tool pass marks height changed. Nothing marks a stage stale when a *parameter* changes — DCC milestone 2 decided, deliberately and on the record, not to show a fake per-stage pip. |
| **01 Planet** — gravity 1.00 g | §5.1 | `pg` | `planet.g` (0.30–2.50, step 0.05, default 1.00) | wired | Lives in `Generate ▸ Climate…` PLANET today, not a Planet stage. |
| 01 — day length 24 h | §5.1 | `prot` | `planet.rotation_hours` (6–96, step 1) | wired | Same. |
| 01 — axial tilt 23.4° | §5.1 | `ptilt` | `planet.axial_tilt_deg` (0–45, step 0.5) | wired | Same. |
| 01 — geoid sea level | §5.1 | `geoidChk`, `geoidAmp` | **none** | engine gap | Unported, default-off in the reference (`PlanetParams`' own doc comment says so). Small. |
| 01 — tides + moon mass / distance / k₂ | §5.1 | `tidesChk`, `tideMass`, `tideDist`, `tideK2` | **none** | engine gap | Unported, default-off in the reference. Four parameters. Small-to-medium. |
| **02 Extent & scale** — region / world | §5.1 | `extentSeg` | `world` param key + `PROXY_KEYS` onto `%ExtentInput` | wired | One node behind two surfaces (File ▸ New world and Generate ▸ Climate) — already solved. |
| 02 — working resolution 512–8K | §5.1 | — | `generate_sized(seed, width_km, grid_w, grid_h)` argument | wired | Plus a free 4–8192 entry and seven aspect presets the spec does not mention. |
| 02 — sea level 42 % | §5.1 | `sea` | `set_sea_level(f)` / `sea_level` param | wired | Excluded from the Generate dialogs on purpose so there is one control for one value. |
| 02 — peak altitude 4000 m | §5.1 | `peak` | `peak_m` (1–30000, step 50) | wired | In `Generate ▸ Climate…` today. |
| 02 — units | §5.1 | `calUnitSeg` | none | new | **Duplicated with Preferences ▸ Application ▸ Units.** |
| — *map width km* (engine has it, the spec does not name it) | §5.1 | — | `generate_sized`'s `width_km` argument, `get_map_width_km()` | wired | A creation-time decision the reference itself refuses to make editable mid-project. Six scale presets already ship. |
| **03 World structure** — archetype (Earth, Super, Islands, Volcanic, Rift, Custom) | §5.1 | `#wsEnabled`, `#wsPanel` | `apply_archetype(name)`, `get_archetypes()` → `["earth","supercontinent","archipelago","volcanic","rift"]` | wired | Wired **but not through those `#[func]`s** — `main.gd` hardcodes `WORLD_SHAPES` and dispatches `generate_world_structure_sized`. `get_archetypes()`/`apply_archetype()` exist and nothing calls them. Naming: spec "Islands" = engine `archipelago`; spec "Custom" = `world_structure.enabled` with raw knobs; the spec has **no "Classic"**, which is `main.gd`'s current default (structure off). |
| 03 — continentality 0.30 | §5.1 | `wsCont` | `world_structure.continentality` (0.01–0.90) | wired | `Generate ▸ Tectonics…` WORLD STRUCTURE. |
| 03 — fragmentation 0.50 | §5.1 | `wsFrag` | `world_structure.fragmentation` (0–1) | wired | |
| 03 — tectonic energy 0.60 | §5.1 | `wsTect` | `world_structure.tectonic_energy` (0–1) | wired | |
| 03 — ocean depth 0.60 | §5.1 | `wsOcean` | `world_structure.ocean_depth` (0–1) | wired | |
| 03 — hotspot density 0.20 | §5.1 | `wsHot` | `world_structure.hotspot_density` (0–1) | wired | |
| **04 Tectonics** — plates 14 | §5.1 | `plates` | `tect.plates` (4–40) | wired | Ignored when World Structure is on (derived) — the design does not say so. |
| 04 — drift ×1.00 | §5.1 | `vel` | `tect.vel` (0–2, step 0.02) | wired | Also derived-over when World Structure is on. |
| 04 — warp 0.45 | §5.1 | `warp` | `tect.warp` (0–1) | wired | |
| 04 — uplift spread 18 px | §5.1 | `sigma` | `tect.blur_r` (2–42, step 0.4, default 18.0) | wired | Naming disagreement: engine calls it blur radius. |
| 04 — tectonic α 0.85 | §5.1 | `alpha` | `tect.alpha` (0–1.2) | wired | |
| 04 — noise β 0.22 | §5.1 | `beta` | `tect.beta` (0–0.6) | wired | |
| 04 — erosion / age 0.60 | §5.1 | `age` | `tect.age_inf` (0–1) | wired | |
| 04 advanced — flexure F 0.20 | §5.1 | `flexure` | `tect.flexure` (0–0.36) | wired | Range is the **reference-reachable** one, deliberately narrower than the field. |
| 04 advanced — heterogeneity C 0.08 | §5.1 | `hetero` | `tect.hetero` (0–0.16) | wired | Same. |
| 04 advanced — rock resistance 0.50 | §5.1 | `resist` | `tect.resist` (0–1) | wired | |
| — 04, engine-only: ridged fractal | §5.1 | `ridged` | `tect.ridged` (bool, default true) | wired | Real, exposed, **absent from the spec**. |
| — 04, engine-only: dynamic lithology | §5.1 | `dynLithChk` | `tect.dynamic_lithology` (bool) | wired | Real, exposed, absent from the spec. |
| — 04, engine-only: Lloyd relaxation passes | §5.1 | — | `tect.lloyd` (0–8) | wired | This port's own superset (`DECISIONS.md` §7d); absent from the spec. |
| — 04, missing from both: graph-driven orogeny | §5.1 | `foldI`, `trenchD`, `faultB` | hardcoded in `OrogenyParams` at the reference's own defaults | engine gap | Exposing them means threading three fields through the `OrogenyParams` call site. Small, already itemised in `GENERATION_PARAMETERS.md`. |
| **05 Volcanism & impacts** — volcanoes 20 | §5.1 | `volc` | `volc.count` (0–100) | wired | |
| 05 — volcano age 0.40 | §5.1 | `volca` | `volc.age` (0–1) | wired | |
| 05 — provinces on | §5.1 | `volcProv` | `volc.provinces` (bool) | wired | Proxied onto the New World checkbox. |
| 05 — craters 100 | §5.1 | `crat` | `crater.count` (0–200, step 2) | wired | |
| — 05, engine-only: crater age 0.50 | §5.1 | `crata` | `crater.age` (0–1) | wired | Real, exposed, absent from the spec. |
| **06 Erosion** — stream-power | §5.1 | `sUp`, `sK`, `sIt`, `sDep`, `sClim` | `stream.{uplift,k,iters,deposit,climate_k}` | wired | The only erosion pass this engine runs, at `max(4, round(iters·0.6))` iterations inside `generate()`. |
| 06 — droplet hydraulic | §5.1 | `drops`, `estr`, `edep`, `ethr`, `etal` | **none** | engine gap | Unported. A separate *manual* op in the reference. Five parameters. |
| 06 — hillslope diffuse | §5.1 | `edD`, `edPas` | **none** | engine gap | Unported. Manual op. |
| 06 — velocity (momentum) | §5.1 | `vIt`, `vStr`, `vMnd` | **none** | engine gap | Unported. The reference itself says it never auto-runs. |
| 06 — glacial | §5.1 | `gSnow`, `gKg`, `gUF`, `gPas` | **none** | engine gap | Unported pass. |
| 06 — coastal | §5.1 | `cWave`, `cEst`, `cMar`, `cPas` | **none** | engine gap | Unported pass. |
| 06 — "each its own group with its own run button" | §5.1 | — | none | engine gap | Per-pass invocation over a live world does not exist; `generate_terrain` is one shot. Also unmentioned by the spec: `evolveCoupled`/`evoCyc`, the reference's manual evolve tool. |
| **07 Hydrology** — river density | §5.1 | `riverDensR` | `river_density` (0.30–3.00) | wired | `Generate ▸ Hydrology…`. |
| 07 — min stream order | §5.1 | `minOrderR` | **none** — a render filter, not a generation parameter | engine gap | `stream_order` is retained on `WorldState`; filtering by it is a render-layer control that does not exist. |
| 07 — lakes as water | §5.1 | — | `build_water_bodies` classification exists; `lake_mask` exists in `sculpt_commit` | engine gap | No generation parameter and no render toggle. **Uncertain** whether the spec means a classification switch or a display switch. |
| — 07, engine-only: carve rivers on generation | §5.1 | `carveRiversChk` | `carve_rivers` (bool, default true) | wired | Real, exposed, absent from the spec — and it is the switch that decides whether river topology exists at all. |
| **08 Climate** — latitude band | §5.1 | `latN`, `latS` | `climate.lat_n`, `climate.lat_s` (−90..90) | wired | |
| 08 — equator / pole °C | §5.1 | `teq`, `tpo` | `climate.equator_temp` (0–45), `climate.pole_temp` (−50..10) | wired | |
| 08 — lapse rate | §5.1 | `lapse` | `climate.lapse_rate` (0–12, step 0.1) | wired | |
| 08 — seasons & Köppen | §5.1 | `seasons` | **none** | engine gap | `computeSeasons()` deliberately deferred; Köppen classification unported. Medium. |
| 08 — currents | §5.1 | `currents` | `climate.currents` (bool) + `climate.current_k` (advanced) | wired | Proxied onto the New World checkbox. |
| 08 — ice albedo | §5.1 | `albedo` | `climate.albedo_k` (0–1, default 0) | wired | |
| 08 — weather sim iterations | §5.1 | `wIters` | `climate.w_iters` (20–200, step 5) | wired | |
| — 08, engine-only: the rest of the `weather` group | §5.1 | `rainK`, `evap`, `rainDep`, `ocean`, `windModeSeg`, `windDir`, `pressK`, `zonalK` (+ `ocean_hum`, `bulk_evap` with no reference control) | `climate.{rain_k, evap, rain_dep, ocean, wind_manual, wind_dir_deg, press_k, zonal_k, ocean_hum, bulk_evap}` | wired | **Ten real, exposed, working parameters the spec does not mention at all.** Plus `climate.terrain_wind_deflection`. |
| **09 Ecology & biomes** — rivers in biome view | §5.1 | — | none | engine gap | A render-layer toggle; `build_biome_raster` output is not retained (see §6). |
| 09 — ecotone sharpness | §5.1 | — | **none** | engine gap | No such parameter exists in `cartalith-civ`. Biome classification runs off finished fields with no dials in either engine — `main.gd`'s Ecology stage is disabled today with exactly that reason. |
| **10 Resources & soils** — soil depth / ore / fertility | §5.1 | — | `cartalith_civ::build_resource_potentials` (15 fields) | engine gap | Real and computed, but the rasters are **freed** after settlement placement (`MEMORY_OPTIMIZATION_SCOPE.md`), and the stage has no dials in either engine. Surfacing them is a retention-vs-memory decision, not a wiring job. |
| Stale propagation: editing *n* marks every downstream stage stale | §5.1 | — | `StageGraph::mark_changed_tiles` + lazy version comparison | engine gap | Real mechanism, wrong shape: four stages, tile-granular, triggered by a committed **tool pass**, not by a parameter edit. |
| Fields owned by stale stages read `—` until re-run | §5.1 | — | `Staleness{origin_name, reason}` | engine gap | Depends on a per-cell sampler that does not exist (§6). |
| Run stage *n* re-runs only that stage | §5.1 | — | none | engine gap | **The single largest structural disagreement in the document.** See summary §3. |
| Run *n* → 10 walks the chain | §5.1 | — | none | engine gap | Same. |
| Dock foot: Finalize · LOD 0–3 · 85 tiles / bake & freeze | §5.1 | `#bakeAllBtn` | none | engine gap | No bake, no LOD, no atlas. Depends on `LOD_TILING_BASE_SCOPE.md` integration. |
| Un-finalize | §5.1 | `#unfinalizeBtn` | none | engine gap | Same. |
| Finalizing locks stages 01–10 and Sculpt; 3D stays available | §5.1 | — | none | engine gap | A lock state over subsystems that mostly do not exist, plus a 3D viewport that does not exist. |
| NOT A GENERATION STAGE block (five redirect lines) | §5.1 | — | none (chrome) | new | Static text. Cheap, and genuinely useful: it is the spec's own answer to where the deleted menus went. |

### 5.2 Sculpt

`cartalith-terrain::sculpt` is a complete, bit-exact port of the reference's
Sculpt editor (23 golden cases): 13 features, 8 presets, three noise families,
the stamp bbox/coverage/domain-warp pipeline. `PassBuffer` and
`commit_sculpt_pass` complete the draft model. **All of it is unwired.** The
rows below are therefore mostly "backed, unwired" — except where the spec asks
for something the reference never had.

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Brush Tools ▸ Raise | §5.2 | `#genSculpt` | `FreehandMode::Raise` | backed, unwired | |
| Brush Tools ▸ Lower | §5.2 | — | `FreehandMode::Lower` | backed, unwired | |
| Brush Tools ▸ Smooth | §5.2 | — | `FreehandMode::Smooth` | backed, unwired | |
| Brush Tools ▸ Flatten | §5.2 | — | `FreehandMode::Mesa` (nearest), `Feature::Plateau` | backed, unwired | No `Flatten` mode by that name; `Mesa` is the flattening sub-mode. Naming disagreement. |
| Brush Tools ▸ Rotate | §5.2 | — | **none** | new | The reference has no transform brushes. A stamp carries a point list and parameters; there is no rotate operation on it. |
| Brush Tools ▸ Scale | §5.2 | — | none | new | Same. |
| Brush Tools ▸ Tilt | §5.2 | — | none | new | Same. |
| Brush Tools ▸ Push / Pull | §5.2 | — | none | new | Same. |
| Brush Tools ▸ Align | §5.2 | — | none | new | Same. |
| Grid Tools ▸ Add · Duplicate · Rotate · Scale · Tilt · Push · Pull · Align (8) | §5.2 | — | **none** | new | *"Operate on the selected stamp's control grid rather than the heightfield."* **No control-grid concept exists anywhere** — not in `SculptStamp`, not in the reference. Eight controls resting on an unspecified new data model. This is a design invention that needs a definition before it can be sized. |
| Actions ▸ Flip X · Flip Y · Rot Left · Rot Right · Flatten (5) | §5.2 | — | none | new | Same absence: no stamp transform operations exist. |
| Geological feature ▸ Mountain range | §5.2 | `#sculptFeatureSeg` | `Feature::Mountains` | backed, unwired | |
| Geological feature ▸ Volcano | §5.2 | — | `Feature::Volcano` (radial) | backed, unwired | |
| Geological feature ▸ Plateau | §5.2 | — | `Feature::Plateau` | backed, unwired | |
| Geological feature ▸ Rift | §5.2 | — | **none** | engine gap | No `Rift` feature. `Feature::Basin` and `Canyon` are the nearest. A new `apply()` body plus its parameter set. |
| Geological feature ▸ Canyon | §5.2 | — | `Feature::Canyon` | backed, unwired | |
| Geological feature ▸ Crater | §5.2 | — | **none** as a sculpt feature | engine gap | `stamp_craters` exists in the *generation* pipeline (`crater.count`/`crater.age`), which is a different thing from a paintable stamp. |
| Geological feature ▸ Island arc | §5.2 | — | **none** | engine gap | `Feature::Coastline` is the nearest and is not the same landform. |
| Geological feature ▸ Basin | §5.2 | — | `Feature::Basin` | backed, unwired | |
| Geological feature ▸ Dune field | §5.2 | — | **none** | engine gap | No aeolian feature exists anywhere in the port or the reference. |
| — engine-only features the spec drops: Hills, Ridge, Cliff/Escarpment, Valley, River, Lake, Coastline, Freehand | §5.2 | — | `Feature::{Hills, Ridge, Cliff, Valley, River, Lake, Coastline, Freehand}` | backed, unwired | **Eight of the engine's thirteen features are not in the spec's list**, including River and Lake, which own the only special commit path in `sculpt_commit`. Dropping them would discard bit-exact ported work. |
| One-line feature description (`#sculptFeatHint`) | §5.2 | `#sculptFeatHint` | `FeatureMeta` | backed, unwired | Metadata per feature already exists. |
| Feature presets (e.g. mountain range: fold belt, young alpine, eroded massif, coastal scarp) | §5.2 | `#sculptPresetSeg` | `SCULPT_PRESETS` — **Rolling Hills, Alps, Rockies, Badlands, Volcanic Isle, Mesa, Karst, Glacial Valley** | engine gap | The engine's eight presets are the reference's own, are **global not per-feature**, and each overrides exactly one global (`noise_scale`) plus its feature params — verified, not assumed. The spec's four mountain-range presets are new names with no engine counterpart. A preset seeding parameters only (never painting) **is** the engine's model, so that rule holds. |
| Brush Settings ▸ Falloff profile preview (live curve + footprint) | §5.2 | — | falloff is `smoothstep(0,1,(R−dist)/feather)`, `feather = max(floor, R·(1−hardness))` | backed, unwired | Drawable from the real formula. |
| Brush Settings ▸ Brush shape gallery (8 built-in: circle, directional, spatter, spiral, dots, cloud, checker, hatch) | §5.2 | — | **none** | engine gap | The engine has one coverage shape (distance-to-polyline / radial) modulated by `edge_noise` domain warp. Eight arbitrary stamp masks is a new mechanism. |
| Brush Settings ▸ Import brush… (greyscale height stamp, alpha respected) | §5.2 | — | none | engine gap | Same mechanism; plus image decode, which exists for assets. |
| Brush Settings ▸ Operation (Set · Add · Subtract · Multiply · Min · Max), default Set | §5.2 | — | internal `Mode::{Add, Set}`, chosen **per feature**, not by the user | engine gap | The engine has two modes and the feature picks one. Subtract/Multiply/Min/Max do not exist, and making the mode user-selectable changes every feature's semantics. |
| Brush Settings ▸ Falloff (Smooth · Linear · Sharp · Constant · Custom), default Smooth | §5.2 | — | one falloff curve (smoothstep), parameterised by `hardness` | engine gap | Only "Smooth" exists. The other four are new curves. |
| Brush Settings ▸ Radius 0.05–20.0, default 2.00, `[` `]` adjust | §5.2 | — | `SculptGlobals::brush_size` — **6–200 cells, step 1, default 32** | engine gap / disagreement | The ranges are not the same quantity. The spec's 0.05–20.0 is presumably km or a normalised unit; the engine's is cells. Reconciling them needs the km↔cell conversion and a decision about which unit the user sees. |
| Brush Settings ▸ Smooth 0–1, default 0.50 | §5.2 | — | `SculptGlobals::hardness` (0–1, default 0.5)? | backed, unwired | **Uncertain.** The mockup shows `falloff smooth 0.50` in the tool options bar and `Smooth 0.50` in Brush Settings; the engine's 0–1/0.5 control with that behaviour is `hardness`, which the spec **also** lists separately at 0.35 in §4. Two spec controls may be one engine parameter. Needs an owner/design ruling. |
| Brush Settings ▸ Strength ±500 m, default +120 m, ⇧ inverts | §5.2 | — | `SculptGlobals::intensity` (0–1.5, unitless, default 1.0) | engine gap | Signed metres vs unsigned dimensionless. The sign is how Raise/Lower differ in the engine (two sub-modes), not a negative strength. |
| Brush Settings ▸ Rotation 0–360°, default 0 | §5.2 | — | **none** | new | No stamp rotation exists. The mockup shows `Rotation 34°`. |
| — engine-only globals the spec drops: noise scale (1–20), octaves (1–8), persistence (0.20–0.90), lacunarity (1.40–3.20), edge noise (0–1) | §5.2 | — | `SculptGlobals::{noise_scale, octaves, persistence, lacunarity, edge_noise}` | backed, unwired | Five real, defaulted, reference-derived controls. The mockup's Properties panel shows `noise scale` and `octaves · persistence`, so they are in the design; §5.2's Brush Settings table omits them. |
| Every stroke becomes a live procedural stamp; nothing touches the heightfield until Commit | §5.2 | — | `PassBuffer` + `SculptStamp` + `preview_into` | backed, unwired | Exactly the engine's model, exactly the reference's. The strongest agreement in the whole document. |
| Commit bakes the whole stack in one pass | §5.2 | — | `PassBuffer::commit` / `commit_sculpt_pass` | backed, unwired | |
| …*and re-runs erosion, hydrology and climate once* | §5.2 | — | `StageGraph::mark_changed_tiles` (marks stale; does **not** re-run) | engine gap | Direct contradiction. The engine's model is deliberately lazy — `UNIFIED_TOOL_PLAN.md` measures the eager version at ~7 s per stroke at 2048² and rejects it explicitly. There is also no per-stage re-run entry point to call. See summary §3. |

---

## 6 · Right dock · contexts

Contents follow the selection, not the workspace. The current right dock is
Layers / Properties / Sample, fixed.

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| **Sample** — X, Y | §6 | — | `map_overlay.gd` cursor → grid cell | wired | The only two Sample fields that are real today. |
| Sample — elevation (large accent readout) | §6 | — | `WorldState::field` retained; **no accessor** | backed, unwired | One `#[func]` (`sample_cell(x, y) -> Dictionary`) covers this row and the next several. `main.gd` says so on screen today: *"Per-cell fields … need a new engine query this milestone doesn't add."* |
| Sample — slope | §6 | — | `cartalith_civ::build_slope_field` (computed, **not retained**) | engine gap | Recompute-on-demand or retain. |
| Sample — aspect | §6 | — | derivable from `field` | backed, unwired | |
| Sample — plate + type | §6 | — | `WorldState::{plate_id, crust_field}` retained | backed, unwired | |
| Sample — boundary + distance | §6 | — | `WorldState::{boundary_mask, boundary_type}` retained; **distance-to-boundary is not computed** | engine gap | The mockup shows `convergent · 41 km`. A distance field would be new. |
| Sample — resistance | §6 | — | `WorldState::resistance_field` retained | backed, unwired | |
| Sample — lithology | §6 | — | `cartalith_civ::build_lithology` (computed inside `compute_civilisation`, **not retained**) | engine gap | Same retention-vs-recompute decision; `MEMORY_OPTIMIZATION_SCOPE.md` is the tension. |
| Sample — temperature | §6 | — | `WorldState::temperature` retained | backed, unwired | |
| Sample — precipitation | §6 | — | `WorldState::rainfall` retained | backed, unwired | |
| Sample — drainage | §6 | — | `WorldState::{flow_area, flow_discharge}` retained | backed, unwired | |
| Sample — biome | §6 | — | `build_biome_raster` (**not retained**; only a per-settlement `biome: u8` survives on `SettlementExplanation`) | engine gap | Same. |
| Sample — soil | §6 | — | `build_resource_potentials` (**freed** after placement) | engine gap | Same, and this one is the field the memory pass explicitly freed. |
| Sample — control (owning faction) | §6 | — | `CivData::territory` retained (`0` = unowned) | backed, unwired | |
| Sample — nearest settlement | §6 | — | `get_settlements()` + `cartalith_civ::tools::civ_pick_place_at` | backed, unwired | Distance in km from `cartalith_spatial::measure::cell_km`. |
| Sample — fields from stale stages read `—` | §6 | — | `StageGraph` | engine gap | See §5.1. |
| **Layers** — ordered list, visibility dot | §6 | — | five real toggles in `main.tscn` (`ShowSettlementsCheck`, `ShowRoadsCheck`, `ShowSeaRoutesCheck`, `TerritoryLayerCheck`, `ProvinceLayerCheck`) | wired | Independent per layer since DCC m1 (`GUI_FEATURE_PARITY_SCOPE.md` item 9). |
| Layers — opacity bar | §6, §7 | — | the three overlay textures already support alpha | backed, unwired | `GUI_FEATURE_PARITY_SCOPE.md` Category 3: *"build opacity now (cheap)."* Still open. |
| Layers — blend mode | §6, §7 | — | none | engine gap | The layers are baked per-pixel in `render.rs`'s single pass, not independently compositable. Category 3's recommendation to **defer** until the render architecture separates them still holds. |
| Layers — reorder | §6 | — | none | engine gap | Same architecture change. |
| Layers — nested children under Terrain | §6, §7 | — | none | engine gap | Same. |
| **Stamp stack** — newest-first list with index, visibility, type, parameter summary | §6 | — | `PassBuffer::{entries, get, len}`, `PassEntry{stamp, hidden}` | backed, unwired | |
| Stamp stack — Deselect | §6 | — | none (chrome) | new | Selection is GDScript-side. |
| Stamp stack — Hide / show | §6 | — | `PassBuffer::set_hidden(index, bool)` | backed, unwired | Hidden stamps are skipped by both preview and commit — verified in `commit_sculpt_pass`. |
| Stamp stack — Move up | §6 | — | `PassBuffer::move_up(index) -> bool` | backed, unwired | |
| Stamp stack — Move down | §6 | — | `PassBuffer::move_down(index) -> bool` | backed, unwired | |
| Stamp stack — Delete | §6 | — | `PassBuffer::remove(index) -> S` | backed, unwired | |
| Stamp stack — selected-stamp parameters (length, width, asymmetry, ridge noise, blend) | §6 | — | `FeatureParams::Mountains{…}` and siblings | backed, unwired | **Names do not match.** The engine's per-feature params come from the reference; `length`/`width`/`asymmetry`/`ridge noise`/`blend` need mapping to real `FeatureParams` variants case by case. `blend` in particular looks like the internal `Mode`, which is not user-settable. |
| Stamp stack — ↶ Undo / ↷ Redo | §6 | — | `PassBuffer::{undo, redo}` | backed, unwired | |
| Stamp stack — ✓ Commit to map / Discard | §6 | — | `commit_sculpt_pass` / `discard` | backed, unwired | |
| Stamp stack — finalize-lock note | §6 | — | none | engine gap | No finalize state exists. |
| **River** — name, length, source elevation, discharge, catchment, tributaries, navigation | §6 | — | `ChannelResult`, `stream_order`, `flow_discharge` retained; `cartalith_civ::labels` can name things | engine gap | Rivers are not **entities**: there is no river object with an id, a name, a catchment or a tributary list. `build_channels` produces a per-cell network. Medium — a real river-entity extraction pass. |
| River — Edit geometry button | §6 | — | `Feature::River` sculpt stamp; `cartalith_engine::sculpt_commit::enforce_channel_descent` | backed, unwired | Editing a river's geometry is exactly what the River sculpt feature plus its special commit path do. |
| River — Hydrology / Analyse catchment buttons | §6 | — | none | engine gap | No catchment entity or analysis pass exists. |
| **Settlement** — name, population, class, government, agriculture, trade, water access, defensibility, routes | §6 | — | `get_settlements()`, `explain_settlement(index)`'s thirteen weighted terms, `get_trade_balances()` | wired | The strongest existing inspector: click-to-pin shows the full causal "WHY HERE?" chain. Government / agriculture are confirmed **UI-only categorical labels with zero derived computation** in the reference itself (`PHASE2_SCOPE.md` m18) — cheap, but invented data. |
| Settlement — Economy / Politics / Logistics buttons | §6 | — | `get_trade_balances()` (wired), `get_provinces()` (wired), Journey Planner (engine-complete, no `#[func]`) | backed, unwired | Logistics is the Journey Planner GUI — `GUI_FEATURE_PARITY_SCOPE.md` calls it the largest ready-to-build GUI surface left. |
| **Faction** — roster entry | §6 | `#civFactionInspectorHost` | `cartalith_civ::civ_faction_aggregates` (population, tax, five-axis power, sector output, `terrain_mix`, `world_mean_terrain`) — golden-verified, **no `#[func]`** | backed, unwired | Phase 2 m20 unblocked this; `GUI_FEATURE_PARITY_SCOPE.md` item 5 is now a wiring job held only by the UI hold. |
| Faction — territory, provinces | §6 | — | `CivData::{territory, provinces, province_list}` + `get_provinces()` | wired | Province list already shows in `Simulate ▸ Statistics…`. |
| Faction — state religion | §6 | — | none | engine gap | Confirmed UI-only categorical in the reference; no computation. Roster *mechanics* (add/remove, persistent identity) are new Rust state. |
| **Route** — stages, vessels, cost trace, per-stage overrides, daily stages | §6 | — | Journey Planner: ~65 of 74 functions ported (`jp_journey_cost`, `jp_biome_key`, the sail polar, the day-wage cost model, `civ_dijkstra_path`) | backed, unwired | Engine-complete per `JOURNEY_PLANNER_SCOPE.md` and `STATUS.md`; **no `#[func]` and no GUI**. This is a large GUI milestone, not a small one. |
| **Brush / Stamp** — size, hardness, intensity, noise scale, octaves, persistence, lacunarity, edge noise | §6 | — | `SculptGlobals` (all eight) | backed, unwired | This right-dock context maps **1:1** onto the engine's globals — closer than §5.2's own Brush Settings table does. |
| Brush / Stamp — stamp stack, commit / discard | §6 | — | `PassBuffer` | backed, unwired | Duplicated with the Stamp stack context above. |
| Collapsed dock keeps the primary readout (elevation / layer dots / stamp count) | §6 | — | none (chrome) | new | |

---

## 7 · Cartography → Style

Nothing in this section exists in the shell today; `Render ▸ Terrain
appearance…` is present and disabled. The engine side is
`TerrainAppearance` — roughly forty real, tested fields driving the current
render — reachable through **no `#[func]` at all**. So most rows are "backed,
unwired" for the *value* and "engine gap" for the *editing model* (layers,
ramps, presets), which is a different thing.

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Layer list — search field | §7 | — | none (chrome) | new | |
| Layer list — ordered stack: Labels & annotation | §7 | — | `cartalith_civ::labels` (`MapLabel`, arc layout), `cartalith_assets::manual` | backed, unwired | Milestone E ported label layout and manual icons; nothing renders them in Godot. |
| Layer list — Settlements | §7 | — | `get_settlements()` + `map_overlay.gd` | wired | As a visibility toggle. |
| Layer list — Ways & routes | §7 | — | `get_roads()`, `get_sea_routes()` | wired | Two toggles today. |
| Layer list — Political (off) | §7 | — | `build_territory_texture()`, `build_province_boundary_texture()` | wired | Two toggles today. |
| Layer list — Water | §7 | — | baked into `build_color_texture` | engine gap | Not an independent layer; water is painted into the single colour pass. |
| Layer list — Vegetation | §7 | — | baked into `land_color`'s material weights | engine gap | Same. |
| Layer list — Terrain (with children) | §7 | — | `build_color_texture` | engine gap | Same — the children below are not separable outputs today. |
| Layer child — Hand-drawn hillshade (off) | §7 | — | **none** | engine gap | NPR/painterly rendering is unported (`FUNCTIONAL_CONTRACT.md` §6: "Map rendering (NPR/geology/SDF toggles) — Absent"). |
| Layer child — Hillshade (active) | §7 | — | `render.rs::shade`, `relief_lights`, `relief_directionality`, `relief_ambient`, `relief_gain` | backed, unwired | Real and golden-verified; multidirectional relief plus AO landed in Phase 3 m2. |
| Layer child — Colour relief | §7 | — | `land_color`'s material palette | engine gap | Real as a *material* palette, **not** as an elevation-breakpoint ramp — `render.rs`'s own module doc says so explicitly: *"there is no elevation-keyed colour breakpoint ramp anywhere in this renderer."* |
| Layer list — Land, Background | §7 | — | `paper_tint`, `paper_strength`, `paper_grain` | backed, unwired | The paper/vellum ground is real (Phase 3 m4). |
| Layer list footer tabs — Blocks / Verticality | §7 | — | none | new | Undefined in the spec beyond the two words. **Uncertain** what either does. |
| Layer properties — Visibility (the five overlay layers) | §7 | — | the five `main.tscn` toggles + `map_overlay.gd` draw categories | wired | |
| Layer properties — Visibility (terrain sub-layers) | §7 | — | none | engine gap | Terrain's children are baked into one colour pass; they cannot be hidden independently. |
| Layer properties — Visualization dropdown | §7 | `#debugSeg` | **none** | engine gap | The reference's analysis-field switcher (elevation, slope, aspect, curvature, flow, drainage, temperature, rainfall, wind, currents, soil, lithology, biome). `render.rs` computes several internally per pixel but exposes no selectable output channel. `FUNCTIONAL_CONTRACT.md` §11 flags this as **ambiguous and unresolved** — still unresolved here. |
| Layer properties — Opacity (78) | §7 | — | overlay textures support alpha | backed, unwired | |
| Fill — Colour ramp picker | §7 | — | **none** | engine gap | See below; the whole ramp system is new. |
| Fill — Domain (World / View / Absolute) | §7 | — | none | engine gap | Depends on a ramp existing. |
| Fill — Range (−410 → 4 210 m) | §7 | — | `peak_m`, `sea_level`, `metres_per_unit = peak_m/(1−sea_level)` | backed, unwired | The metre mapping is real; nothing exposes the world's min/max elevation. |
| Light — Azimuth 315° | §7 | — | `TerrainAppearance::sun_az_deg` | backed, unwired | Needs a `set_appearance` `#[func]`. |
| Light — Elevation 45° | §7 | — | `TerrainAppearance::sun_alt_deg` | backed, unwired | Same. |
| Light — Strength 0.62 | §7 | — | `TerrainAppearance::relief_gain` (+ `relief_ambient`) | backed, unwired | Naming: "strength" is presumably gain; **uncertain** whether the design means gain, directionality, or both. |
| Light — Multidirectional (8 lights) | §7 | — | `TerrainAppearance::relief_lights` (`1` reproduces the reference's exact single sun; the default look uses six) | backed, unwired | **Value disagreement**: spec says 8, the engine's own default is 6 and `Ultra` uses 10. Free to set — the field is a count, not an enum. |
| Layer properties footer tabs — Style / Data / `{ }` (JSON view) | §7 | — | `TerrainAppearance` is **not** `Serialize` | engine gap | A JSON view of a layer's style needs a serialization the struct does not derive. Small. |
| Colour ramp popover — nine named ramps (Earth, Elevation, Atlas, Mono, Imhof, Ice, Dark ice, Desert, Dark atlas) | §7 | — | `cartalith_terrain::tile_render::hypso` + `SEA`/`LAND` palettes — **one** fixed hypsometric ramp, used only by region-export tiles | engine gap | Medium-to-large and genuinely new: nine named ramps, editable stops, and a renderer that consumes an elevation ramp at all. `TERRAIN_APPEARANCE_SCOPE.md` m1's own audit recorded this absence as a finding. |
| Colour ramp popover — Create custom ramp… | §7 | — | none | engine gap | Same. |
| Stop editor — ramp bar with draggable stops | §7 | — | none | engine gap | Same. |
| Stop editor — ＋ add / delete / reverse | §7 | — | none | engine gap | Same. |
| Stop editor — selected stop: elevation | §7 | — | none | engine gap | |
| Stop editor — colour swatch + hex | §7 | — | `Rgb` triples throughout `TerrainAppearance` | engine gap | The colour type exists; the stop model does not. |
| Stop editor — alpha | §7 | — | none | engine gap | `Rgb` has no alpha channel. |
| Stop editor — interpolation to next stop (Linear / Ease / Step) | §7 | — | `ramp3` (a fixed 3-stop linear micro-ramp for texture variety) | engine gap | Not the same mechanism. |
| Stop editor footer — Compare | §7 | — | none | engine gap | `GUI_FEATURE_PARITY_SCOPE.md` Category 3 recommends deferring A/B preview until the appearance GUI exists at all. |
| Stop editor footer — Apply | §7 | — | `build_color_texture()` re-run | backed, unwired | Presentation-only re-render is already how `set_quality_tier` is contracted to work. |
| Rule: no control here alters heightmap/climate/hydrology/biome/settlements/routes/seed | §7 | — | `set_quality_tier`'s doc comment states exactly this contract | backed, unwired | The engine already honours the rule for the one presentation control it exposes. |
| Rule: no control here marks a generation stage stale | §7 | — | `StageGraph` is only marked by committed tool passes | wired | Trivially true today. |
| Status bar reports `style edited — 2 layers differ from preset Atlas` + repaint time | §7, §10 | — | none | engine gap | Needs the preset-diff model above. Repaint timing is GDScript-side. |

---

## 8 · Asset library window

Engine side: `cartalith-assets` is **Phase 4 complete** (7 milestones), and
`cartalith-godot::pack` already composites real pack sprites and ground splat
into the rendered map. What does not exist is any `#[func]` into the library
model, and any authoring UI — `ASSET_LIBRARY_SCOPE.md` §8 puts the library
page UI explicitly outside milestone 7.

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Window opens over the map (⇧A); the map is hidden while open | §8 | `#assetLibrary` | none (chrome) | new | |
| Window bar — search (`name · type · category · tag · file`) | §8 | `#alSearch` | `SlotMeta::tags`, `LibraryItem::name`, `slot_title` | backed, unwired | Every searched attribute is real. |
| Window bar — Sort (slot order / name / filled first) | §8 | `#alSort` | `AssetDB::uids_in_order`, `filled_count` | backed, unwired | |
| Window bar — ▦ Sprite sheet… | §8 | `#alSlicerBtn` | none | engine gap | See §2.3. |
| Window bar — ☑ Select with live count | §8 | `#alSelModeBtn` | none (chrome) | new | Selection model. |
| Window bar — Apply to map | §8 | `#alApplyBtn` | `apply_library_file_with_items` + reload | backed, unwired | |
| Window bar — Export pack .zip | §8 | `#alExportBtn` | `archive::write_pack` | backed, unwired | |
| Window bar — Close | §8 | — | none (chrome) | new | |
| Family rail — 24 families in three groups | §8 | — | `Family` (8 variants) | engine gap | See §2.3 — the taxonomy disagrees, and the mockup itself shows 11. |
| Family rail — Collections (tag sets) | §8 | — | `AssetCollections` | backed, unwired | |
| Family rail — Unassigned imports with count | §8 | — | none as a distinct bucket | engine gap | Small: a slot-less staging area the model does not have. |
| Family rail row — code, name, `filled/capacity`, accent when incomplete | §8 | — | `library_slot_ids(family)`, `slot_title`, `filled_count` | backed, unwired | Capacity is the frozen vocabulary's length — real. |
| Family rail footer — Import image… | §8 | `#alFilePicker` | `raster::decode_png` + `AssetDB::add_item` | backed, unwired | PNG only — see §2.3. |
| Family rail footer — Import pack… | §8 | `#alImportPackBtn` | `WorldGen::load_asset_pack` | wired | Same call as File ▸ Import asset pack. |
| Slot grid — family-scoped, six columns | §8 | — | `AssetDB::slots_in_family` | backed, unwired | |
| Slot grid header — `P · PLACES · 10 OF 12 FILLED` | §8 | — | `filled_count` + vocabulary length | backed, unwired | Family codes (`P`, `B`, `W`, `T`, `R`, `X`, `H`, `C`, `L`, `S`, `M`) are the spec's, not the engine's. |
| Slot grid — batch actions (Tag… Collect… Rename… Duplicate Delete) | §8 | see §2.3.1 | see §2.3.1 | backed, unwired | |
| Slot cell — checkerboard when empty | §8 | — | `LibrarySlot` with no items | backed, unwired | |
| Slot cell — art when filled | §8 | — | `raster::render_item`, `pack.rs`'s blit | backed, unwired | |
| Slot cell — `×3` variant badge | §8 | — | `AssetDB::items(uid).len()` | backed, unwired | |
| Slot cell — ☑ when selected | §8 | — | none (chrome) | new | |
| Slot grid footer — drop-to-fill, ⇧-click ranges, ⌘-click adds, zoom | §8 | — | none (chrome) | new | Drag-and-drop file handling is Godot-side. |
| Slot inspector — slot code + name | §8 | — | `slot_title`, `library_slot_ids` | backed, unwired | |
| Slot inspector — preview on a checkerboard | §8 | — | `render_item` | backed, unwired | |
| Slot inspector — file readout (`capital-star.png · 512 × 512 · PNG · 84 KB`) | §8 | — | `DecodedImage{w,h}`, `LibraryItem::name`, byte length | backed, unwired | |
| Slot inspector — Scale 118 % | §8 | `#alScale` | `ItemTransform::scale` | backed, unwired | |
| Slot inspector — Fit / Reset | §8 | `#alFit`, `#alReset` | `raster::fit_to_bottom`, `ItemTransform::default` | backed, unwired | |
| Slot inspector — Replace… / + Variant | §8 | `#alReplace`, `#alAddVar` | `remove_item` + `add_item` | backed, unwired | |
| Slot inspector — Preview bg (5 swatches) | §8 | `#alBgSw` | none (chrome) | new | |
| Slot inspector — Anchor top / centre / base | §8 | — | `Anchor::{Bottom, Center, None}` — **assigned per family, not per slot** | engine gap | The engine's anchor is a property of the family (icons anchor Bottom, settlement/POI Center, textures None) and is load-bearing for `sprite_draw_rect`. Making it per-slot user-settable changes placement semantics. |
| Slot inspector — Tags with ＋ | §8 | — | `SlotMeta::tags` | backed, unwired | |
| Slot inspector — Variants strip (3, active outlined) | §8 | — | `Vec<LibraryItem>` per slot, `pick_weighted_variant` | backed, unwired | Note: variant choice at render time is **weighted and seeded**, not a user-picked "active" one. A UI "active variant" has no engine counterpart. |
| Slot inspector — Pack metadata (name / author / license) | §8 | `#alPackName`, `#alPackAuthor`, `#alPackLicense` | `PackInfo` | backed, unwired | |
| Footer — Validate · 8 warnings | §8 | `#alValidateBtn` | `library::run(&AssetDB)` | backed, unwired | |
| Footer — Clear library… | §8 | `#alClearBtn` | `AssetDB::clear` | backed, unwired | |
| Slicer modal — sheet preview with cell grid overlay + readout | §8 | — | none | engine gap | |
| Slicer modal — Columns / Rows / Margin / Spacing | §8 | — | none | engine gap | |
| Slicer modal — Trim transparent edges | §8 | — | none (`render_item` crops/fits but does not trim-detect) | engine gap | |
| Slicer modal — Skip empty cells | §8 | — | none | engine gap | Needs alpha-coverage detection per cell. |
| Slicer modal — Assign to family | §8 | — | `Family` | backed, unwired | |
| Slicer modal — Fill from `first empty` / `overwrite` | §8 | — | `AssetDB::add_item` (the policy is the caller's) | backed, unwired | |
| Slicer modal — Cancel / Slice N cells; non-destructive | §8 | — | none | engine gap | The whole slicer is one small-to-medium engine addition; the rows above split it only because the spec does. |
| Status bar — `library edited — apply to map to use it`, unassigned import count | §8, §10 | — | none | engine gap | Dirty-state plus the unassigned bucket. |

---

## 9 · Data manager window

**This is the largest genuinely new region in the design.** Five route groups,
of which two (Sources, Conversion) have no engine counterpart at all, one
(Validation) has one only for asset packs, and two (Import, Export) are half
real. The §2.4 table carries the per-route backing; the rows below are the
window's own controls.

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Window frame, title, subtitle, Close | §9 | — | none (chrome) | new | |
| Routes rail — the five groups | §9 | — | none (chrome) | new | Navigation only; per-route backing is in §2.4. |
| Routes rail foot — exports root and last run (`14:02 · 62 MB`) | §9 | — | none | new | Depends on the storage-locations decision and a run history that does not exist. |
| Route pane — breadcrumb header | §9 | — | none (chrome) | new | |
| TILES — scheme (XYZ / TMS / WMTS) | §9 | — | **none** | engine gap | `region_export` writes the *reference's own* region-tile scheme with a `buildTileManifest` JSON, not a slippy-map pyramid. XYZ/TMS/WMTS are three new addressing schemes. |
| TILES — zoom range (0–4 / 0–6 / 0–8 / custom) | §9 | — | `cartalith_terrain::{amplify_region, refine_tile}` produce refined levels | engine gap | Refinement exists; a zoom *pyramid* with per-level tile addressing does not. Depends on `LOD_TILING_BASE_SCOPE.md` integration. |
| TILES — tile size 256 px | §9 | — | `region_export`'s own tile size, `TileDims` | backed, unwired | |
| TILES — format (PNG-8 · WebP fallback) | §9 | — | `raster::encode_png` (RGBA8), `tile_png_bytes` | engine gap | PNG-8 (paletted) and WebP are both new encoders. |
| TILES — Retina @2x variants | §9 | — | `refine_tile` | backed, unwired | Plausibly `refine_tile` at 2×; **uncertain**, since @2x means the same extent at double resolution, which is what refinement does. Verify before building. |
| TILES — Skip all-ocean tiles (−1 842 tiles) | §9 | — | `sea_level` + `field`; `CoarseBounds` in `cartalith_io::tiles` | backed, unwired | The min/max bounds a tile record already carries make this a cheap test. |
| PROJECTION — CRS (EPSG:3857 / EPSG:4326 / custom) | §9 | — | **none** | engine gap | See §2.4 — there is no real-world CRS to project from. `geo_xy(gx, gy, gh, cell_km)` produces a **local** planar coordinate, deliberately. Owner decision. |
| PROJECTION — World bounds | §9 | — | `get_map_width_km`/`get_map_height_km`, `world` | backed, unwired | The world's own extent is real; expressing it as `−180 −85 · 180 85` presumes the CRS above. |
| PROJECTION — Write world file (.wld + .prj) | §9 | — | none | engine gap | Small once a CRS decision exists; meaningless before it. |
| LAYERS INCLUDED — relief + hillshade | §9 | — | `render_height_tile_rgba` (hypsometric tint × hillshade) | backed, unwired | |
| LAYERS INCLUDED — political tint | §9 | — | `build_territory_texture`, `territory_feature` | backed, unwired | |
| LAYERS INCLUDED — labels & icons (raster) | §9 | — | `cartalith_civ::labels`, `cartalith_assets::manual`/`placement` | engine gap | The layout maths is ported; rasterising labels into export tiles needs a text renderer, which the crate deliberately does not have (`labels` splits at text measurement so it never touches a canvas). |
| LAYERS INCLUDED — rivers & coastlines | §9 | — | `ChannelResult`, `mask_outline_coords`, `trace_mask_rings` | backed, unwired | The vector tracer is real and golden-verified. |
| OUTPUT — Destination | §9 | — | none | new | Storage-locations decision. |
| OUTPUT — Packaging (folder / .zip / MBTiles) | §9 | — | `zip_store` / `zip_region_export` (`.zip` real); folder trivial; **MBTiles absent** | engine gap | MBTiles is a SQLite container — a new dependency and a new format. |
| OUTPUT — Emit `leaflet-preview.html` | §9 | — | none | new | Trivial text emission, but it assumes the XYZ pyramid above. |
| OUTPUT — Emit `style.json` + attribution | §9 | — | `geojson::stringify` (a JSON writer exists) | engine gap | A Mapbox-GL style document is a different spec entirely. |
| ESTIMATE — tiles, size, render time, source | §9 | — | `TileDims`, `build_tile_manifest` | backed, unwired | Tile counts are computable today; "~3 min 40 s · 2 GPUs" presumes the multi-GPU model that does not exist. |
| RECENT RUNS — timestamp, route, size, result | §9 | — | none | new | Run history store. |
| Footer — Save as preset | §9 | — | none | engine gap | Export-preset serialization. |
| Footer — Dry run | §9 | — | `export_region_tiles` could be run without writing | backed, unwired | The estimate half is already separable. |
| Footer — Export N tiles | §9 | — | `export_region_tiles`, `zip_region_export` | backed, unwired | Real for the region scheme; not for a pyramid. |
| **MARKDOWN VAULT · LINKED** — path + note count | §9 | — | **none** | engine gap | `MARKDOWN_VAULT_INTEGRATION.md` is an owner-supplied **design**, explicitly *"Not started; no code exists for this yet"*, and its own note asks for a real `MARKDOWN_VAULT_SCOPE.md` to be written first. Large. |
| Vault — settlements/factions/journeys resolve to notes by name | §9 | — | none | engine gap | The vault doc's §3 V1 entity scope is Settlements, POIs, Regions, Region labels — **journeys are not in V1**, and the doc warns that POIs/regions as addressable entities may not exist in this port (they do not). |
| Vault — exported tiles carry `obsidian://` links | §9 | — | none | engine gap | **Not in the vault doc at all.** Its §33 non-goals list "Obsidian plugin" and "editor extensions"; a tile-embedded deep link is a new requirement the spec introduces. |
| Vault — Two-way sync (write place notes back) | §9 | — | none | engine gap | **Directly contradicts the vault doc**, whose §1 says V1 is *"deliberately pull-oriented"* and whose §33 lists "automatic bidirectional synchronization" and "automatic Markdown writes" as V1 non-goals. The spec shows it as a toggle (off, "read-only now"), which is compatible in spirit — but shipping the toggle means shipping the capability. Owner decision. |
| Vault — Link labels to notes in GeoJSON | §9 | — | `export_geojson` (writer real; no link property) | engine gap | Not in the vault doc; a new export-schema requirement. |
| Vault — Include front-matter as properties | §9 | — | none | engine gap | The vault doc has a front-matter/JSON layer (§25) but does not specify GeoJSON property injection. |
| Vault — Re-scan vault · Change folder… · Unlink | §9 | — | none | engine gap | The vault doc's §7 Vault Connection and §27 offline/missing-vault states cover these; none is built. |

---

## 10 · Viewport, timeline, status bar

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Layers button (36 px, top-left of viewport) | §10 | `#layersFab` | none (chrome) | new | The five real layer toggles live in the right dock today, not a viewport popover. |
| Layers popover — master opacity slider | §10 | — | overlay alpha | backed, unwired | |
| Layers popover — SURFACE: Relief (1) | §10 | — | `build_color_texture` | wired | The default view. |
| Layers popover — SURFACE: Biome (2) | §10 | — | `build_biome_raster` (**not retained**, and not a selectable output) | engine gap | Same retention problem as §6. |
| Layers popover — SURFACE: Political (3) | §10 | — | `build_territory_texture()` | wired | |
| Layers popover — TERRAIN FIELDS: Elevation (4) | §10 | `#debugSeg` | `WorldState::field` retained | backed, unwired | Needs a field-to-texture builder mirroring `build_territory_texture`'s shape. |
| Layers popover — TERRAIN FIELDS: Slope (5) | §10 | — | `build_slope_field` (not retained) | engine gap | |
| Layers popover — TERRAIN FIELDS: Flow accumulation (6) | §10 | — | `WorldState::flow_area` retained | backed, unwired | `GUI_FEATURE_PARITY_SCOPE.md` Category 2 small: a texture-builder `#[func]` mirroring `build_territory_texture`. Same for `travel_cost` and `build_route_corridors`, which the spec does not list. |
| Layers popover — CLIMATE: Temperature (7) | §10 | — | `WorldState::temperature` retained | backed, unwired | |
| Layers popover — CLIMATE: Rainfall (8) | §10 | — | `WorldState::rainfall` retained | backed, unwired | |
| Popover must not overlap an open menu | §10 | — | none (chrome) | new | |
| Viewport — top-left context readout (active stage / draft state) | §10 | — | see §5.1 | engine gap | |
| Viewport — top-right projection / zoom / field | §10 | — | none | engine gap | There is no camera and no zoom; the map is a fitted `TextureRect`. "equirect" is only meaningful in `world` mode. |
| Viewport — bottom-left scale bar | §10 | — | `get_map_width_km()` + `_update_scale_bar()` | wired | Real, in km. |
| Viewport — bottom-right cursor coordinates and elevation | §10 | — | cursor → cell is wired; **elevation is not** | backed, unwired | `CoordinatesLabel` shows `E · N (cell)` today. Elevation needs the per-cell sampler. |
| Viewport — `→ 1 582 m` while a draft stamp is under the cursor | §10 | — | `PassBuffer::preview_into` | backed, unwired | The composite-preview read is exactly this. |
| Viewport — brush ring with radius and strength label | §10 | — | `SculptGlobals::{brush_size, intensity}` | backed, unwired | Drawing is Godot-side. |
| **Timeline — scrub track over the project's year range** | §10 | — | **none** | engine gap | There is no time axis, no year, and no project year range. |
| Timeline — transport ▶ Play · ⏸ Pause · Step ◀ ▶ | §10 | — | none | engine gap | |
| Timeline — speeds ×1 / ×10 / ×100 | §10 | — | none | engine gap | |
| Timeline — run state (`PAUSED`) | §10 | — | none | engine gap | |
| Timeline — simulation-layer toggles: Climate · Population · Economy · Politics · Infrastructure · Warfare | §10 | — | none | engine gap | **The engine is a one-shot static generator by explicit, repeated owner decision** (`HARDWARE_ACCELERATION.md`'s own scope correction). `GUI_FEATURE_PARITY_SCOPE.md` puts year-by-year playback and Warfare in *Out of scope*, both pending an explicit product decision (`VISION.md`). The current shell's `Simulate ▸ Time controls` is present and disabled with exactly that reason. **Owner decision — see summary §5.** |
| Timeline absent from generation and style screens | §10 | — | none (chrome) | new | |
| Status bar — left: the one thing needing attention, in accent | §10 | — | `ShellStatusLabel` exists | wired | Currently reports generation results. The four example strings the spec gives all depend on subsystems that do not exist (uncommitted strokes, stage staleness, style diff, draft stamps). |
| Status bar — middle: last heavy pass / repaint / autosave | §10 | — | generation timing is measurable GDScript-side; autosave does not exist | backed, unwired | `AutosaveLabel` and `TileCacheLabel` exist and show placeholder text. |
| Status bar — right: the two or three shortcuts that apply now | §10 | — | `StatusHintLabel` exists and is written by `_on_tool_selected` | wired | DCC m1 found and fixed this slot; it currently names the tool and why it does nothing. |

---

## 11 · Theme tokens

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| 17 colour tokens × dark + light | §11 | — | none (chrome) | new | A real dark `Theme` resource exists (`theme/dark_theme.tres`) but was authored against the previous design; the token values here are new and the light column has never been built. |
| Type: Helvetica Neue / system sans 11–11.5 px desktop, 13–14 tablet, 13 phone | §11 | — | none (chrome) | new | |
| Type: IBM Plex Mono for numerics/codes/shortcuts/labels, 9–11 px, letter-spacing .12–.22 em | §11 | — | none (chrome) | new | **A font dependency the project does not have.** Licensing and packaging are an owner decision (IBM Plex is OFL, so this is a packaging question, not a blocker). |
| Filled accent surfaces carry reversed paper-coloured type in both themes | §11 | — | none (chrome) | new | |
| No fills on panels; hairline separation only; radius 0 everywhere | §11 | — | none (chrome) | new | The current theme uses filled `StyleBoxFlat` panels. A real restyle, not a token swap. |
| — known defect against these tokens | §11 | — | — | engine gap | `dark_theme.tres` has **no `PopupMenu`, `TooltipPanel`/`TooltipLabel` or `ScrollBar` entries**, so those controls fall back to Godot's default chrome regardless of which tokens are chosen (`GUI_FEATURE_PARITY_SCOPE.md` Category 4, still open). |

---

## 12 · Touch behaviour

| Control | Spec ref | v2.10 id | Engine capability | Status | Notes |
|---|---|---|---|---|---|
| Tablet: full desktop parity, 44–52 px targets, 400 px docks | §12 | — | none (chrome) | new | Responsive breakpoints deferred since `GUI_SHELL_SCOPE.md` m1 and never revisited. |
| Phone: map draws edge-to-edge behind every inset | §12 | — | none (chrome) | new | |
| Phone: top 44 px keep-clear, 108 px centre lane reserved, gradient scrim | §12 | — | none (chrome) | new | `ANDROID_BUILD_SCOPE.md` holds real per-region touch-target measurements and a density-independent-pixel correction — still valid input. |
| Phone: app bar (☰ domain drawer, title + seed, ▤ panels, ⋯ overflow) | §12 | — | `get_seed()` for the title | backed, unwired | |
| Phone: domain rail as a 44 px column | §12 | — | none (chrome) | new | |
| Phone: tool options as a bottom sheet; docks as full-height sheets, one at a time | §12 | — | none (chrome) | new | |
| Phone: all five disclosure levels survive inside sheets | §12 | — | none (chrome) | new | |
| Phone: bottom 26 px gesture inset, no tappable targets | §12 | — | none (chrome) | new | |
| Landscape: cutout moves to a side edge, same reserve horizontally | §12 | — | none (chrome) | new | |
| Minimum target 44 px inside the safe area, no exceptions | §12 | — | none (chrome) | new | |

---

# Summary

## 1 · Counts per status, per region

452 controls indexed.

| Region | wired | backed, unwired | engine gap | new | total |
|---|---:|---:|---:|---:|---:|
| §1 Frame geometry | 7 | 0 | 0 | 6 | 13 |
| §2.1 File | 2 | 1 | 4 | 6 | 13 |
| §2.2 Edit | 0 | 2 | 5 | 0 | 7 |
| §2.3 Assets | 1 | 5 | 2 | 0 | 8 |
| §2.3.1 Asset pack submenu | 1 | 19 | 1 | 3 | 24 |
| §2.4 Data dropdown | 2 | 3 | 7 | 5 | 17 |
| §2.5 Preferences | 1 | 6 | 12 | 3 | 22 |
| §2.6 Window | 1 | 0 | 0 | 8 | 9 |
| §2.7 Help | 1 | 0 | 0 | 4 | 5 |
| §3 Domain rail | 6 | 0 | 1 | 3 | 10 |
| §4 Tool options bar | 0 | 10 | 6 | 2 | 18 |
| §5 Left dock header | 0 | 0 | 0 | 1 | 1 |
| §5.1 Generation Pipeline | 42 | 1 | 24 | 3 | 70 |
| §5.2 Sculpt | 0 | 16 | 12 | 8 | 36 |
| §6 Right dock contexts | 4 | 24 | 13 | 2 | 43 |
| §7 Cartography → Style | 5 | 11 | 20 | 2 | 38 |
| §8 Asset library window | 1 | 26 | 10 | 6 | 43 |
| §9 Data manager window | 0 | 10 | 16 | 7 | 33 |
| §10 Viewport, timeline, status bar | 5 | 9 | 9 | 3 | 26 |
| §11 Theme tokens | 0 | 0 | 1 | 5 | 6 |
| §12 Touch behaviour | 0 | 1 | 0 | 9 | 10 |
| **Total** | **79** | **144** | **143** | **86** | **452** |

**The shape this makes.**

- **17 % of the design already works** (79), and almost all of it is one
  region: **42 of the 79 are generation parameters in §5.1**. The generation
  half of this design is close to done and is the thing most at risk of being
  disturbed by a restructure.
- **32 % is backed and unwired** (144) — real, tested, mostly golden-verified
  Rust with **no `#[func]`**. Three regions carry most of it: the Asset library
  (26), the right dock (24), and the asset-pack submenu (19). This is the
  cheapest work in the document and the least visible from the outside, because
  none of it appears in the GUI at all today.
- **32 % is an engine gap** (143), but it is not spread evenly: §7 Cartography
  (20), §5.1's pipeline mechanics (24), §9 Data manager (16), §6's per-cell
  sampler and entity model (13), §5.2's new brush mechanics (12), §2.5
  Preferences' performance/LOD block (12).
- **19 % is new** (85) — and 42 of those are pure chrome (frame geometry,
  window/layout toggles, theme, touch), which costs GDScript time and no engine
  time at all.

The single most useful number: **§8's Asset library window is 26 backed-and-
unwired against 10 engine gaps**, and §2.3.1 is 19 against 1. Two whole regions
of this design are a boundary-wrapper problem, not a capability problem.

## 2 · The engine gaps, largest first

**Genuinely large new subsystems, not controls:**

1. **A re-runnable, ten-stage generation pipeline.** `generate_terrain(&p)` runs
   the whole pipeline or none of it — that is not an omission, it is the
   engine's shape, and DCC milestone 2 already decided *on the record* not to
   fake a per-stage stale pip on top of it. The spec's `Run stage 04`,
   `Run 04 → 10`, per-stage `resolved/editing/stale` dots, the rail's `04 / 10`
   counter, the status bar's "6 downstream stages stale", and the rule that
   stale-stage fields read `—` **all hang off this one capability**. Nothing
   scopes it. `cartalith_engine::pipeline_stage_graph` models **four** stages
   (height → hydrology → climate → civ) at tile granularity, triggered by a
   committed tool pass — a different mechanism for a different purpose. Treat
   this as the design's structural spine and the largest unscoped item in the
   document.
2. **The Data manager's import / conversion / validation routes.** No GeoJSON
   *reader* (only a writer), no heightmap reader, no TIFF, no raster map
   import, no CRS, no format conversion, no data transformation, no world-data
   validation or repair, no source registry. **No scope document exists for any
   of it.** 23 of §9's 33 rows and 12 of §2.4's 17. Note the reference had one
   more import the spec drops — "infer tectonics from heightmap".
3. **Markdown vault integration.** `MARKDOWN_VAULT_INTEGRATION.md` is a
   36-section owner-supplied design, explicitly *"Not started; no code exists
   for this yet"*, whose own header asks for a real `MARKDOWN_VAULT_SCOPE.md`
   before anything is built. **§9 assumes more than that document scopes** —
   see disagreement (k) below.
4. **The save writer, project lifecycle and autosave.** `cartalith-io` is
   read-only; `load_save` is its entire surface. `SAVEFILE_COMPAT.md`
   documents the format and `FUNCTIONAL_CONTRACT.md` §10 sets the bar
   (byte-for-byte compatibility, already proven on the read side). Blocks
   Save, Save as, Autosave, Revert, Close project and Export ▸ World Data.
5. **Global undo/redo plus a selection and clipboard model.**
   `FUNCTIONAL_CONTRACT.md` §12: *"absent entirely… its implementation is
   necessarily new."* `PassBuffer`'s undo is draft-scoped and unlabelled.
   Underpins all seven §2.2 rows, Memory ▸ Undo history, and the Delete /
   Cut / Copy / Paste operations over four unrelated entity types.
6. **Tiled LOD, the atlas cache, and a camera.** `LOD_TILING_BASE_SCOPE.md`
   built `TiledField`/`QuadTree`/`DirtyTracker` deliberately standalone and
   unintegrated; there is no camera, no zoom, no quadtree-driven rendering and
   no atlas cache of any kind. Blocks Preferences ▸ Tiles & LOD (4 rows),
   Finalize/bake, the viewport's projection/zoom readout, and §9's zoom range.
7. **Multi-GPU dispatch, device selection and VRAM budgeting.**
   `cartalith_gpu::init_gpu()` requests **one** high-performance adapter.
   `HETEROGENEOUS_COMPUTE_RESEARCH.md` is on file, but its own annotation
   records that no capability-tier classifier or adaptive scheduler has been
   built or scheduled — and that the measured GPU bottleneck was per-dispatch
   context creation, not scheduling, which runs counter to that document's
   framing.
8. **A colour-ramp system, and separable layers.** `render.rs`'s own module
   doc: *"there is no elevation-keyed colour breakpoint ramp anywhere in this
   renderer"* — recorded as a finding by `TERRAIN_APPEARANCE_SCOPE.md`
   milestone 1. Nine named ramps with draggable stops, alpha and per-stop
   interpolation is new. Separately, per-layer opacity is cheap (the overlay
   textures already carry alpha) but **blend mode and reorder need the three
   overlays to become independently compositable layers first** — a real
   architecture change `GUI_FEATURE_PARITY_SCOPE.md` Category 3 already flagged
   and recommended deferring.
9. **The timeline and temporal simulation.** The engine is a one-shot static
   generator by explicit, repeated owner decision. Play/pause/step/speeds/year
   range/six simulation-layer toggles have no engine counterpart and none is
   planned. Not a gap to close — a product decision (§5).
10. **A per-cell sampler, and field retention.** Ten of §6's Sample fields want
    rasters that exist only transiently: `build_slope_field`,
    `build_biome_raster`, `build_lithology` and `build_resource_potentials` are
    computed inside `compute_civilisation` and dropped, and the resource
    rasters are **deliberately freed** by `MEMORY_OPTIMIZATION_SCOPE.md`'s own
    work. So `sample_cell(x, y)` is not one `#[func]` — it is a
    retention-versus-recompute decision with a memory cost the project already
    paid to avoid. Medium.

**Smaller, each already itemised somewhere:**

11. **Unported generation passes**, all listed in `GENERATION_PARAMETERS.md`'s
    own "Parameters the reference exposed that this port does not": droplet
    hydraulic erosion (5 params), hillslope diffusion (2), velocity erosion
    (3), glacial (4), coastal (4), geoid (2), tides (4), seasons/Köppen, the
    three structured-orogeny knobs, evolve cycles. Small-to-medium each.
12. **The sprite-sheet slicer** and the *Unassigned imports* bucket. No scope
    doc; small-to-medium.
13. **New sculpt features and brush mechanics**: Rift, Crater, Island arc, Dune
    field (four new `apply()` bodies); the eight-shape brush gallery and brush
    import; six operations where the engine has two internal modes; five
    falloff curves where the engine has one; stamp rotation; and the **Grid
    Tools / Actions transform set, which rests on a "control grid" concept that
    exists nowhere** and cannot be sized until it is defined.
14. **River, faction and route as addressable entities.** Rivers are a per-cell
    network, not objects with ids, names, catchments or tributaries. Faction
    *aggregates* are real and golden-verified (`civ_faction_aggregates`) but
    roster mechanics — add/remove, persistent identity — are new Rust state.
15. **NPR / hand-drawn hillshade** — `FUNCTIONAL_CONTRACT.md` §6 lists the
    reference's NPR/geology/SDF toggles as absent.
16. **The analysis-field switcher** (`#debugSeg`, §7's Visualization dropdown,
    §10's layer popover). `FUNCTIONAL_CONTRACT.md` §11 flags this as
    **ambiguous and unresolved** — `render.rs` computes several of these fields
    per pixel but may not expose them as selectable output channels. Still
    unresolved; I did not resolve it either. Marked uncertain in the rows.

## 3 · Where the spec and the engine disagree

**(a) The pipeline is decomposed three different ways, and none of them
matches.** This is the most important disagreement in the document.

| | Partition | Count |
|---|---|---|
| Spec §5.1 | Planet · Extent & scale · World structure · Tectonics · Volcanism & impacts · Erosion · Hydrology · Climate · Ecology & biomes · Resources & soils | 10, dependency-ordered |
| `GENERATION_PARAMETERS.md` / `params.rs` | `world · planet · world_structure · tectonics · volcanism · erosion · climate · weather` | 8, each matching a real reference panel heading |
| `main.gd`'s `GEN_STAGES` | Tectonics · Volcanism · Erosion · Glacial & coastal · Hydrology · Climate · Ecology · Settlements · Infrastructure · Politics | 10 dialogs, 6 live |
| `cartalith_engine::pipeline_stage_graph` | height · hydrology · climate · civ | 4, the real dependency graph |

Concretely: the engine's `world` group is **three of the spec's stages at
once** (extent/scale, hydrology's river density, and a GPU switch) — `main.gd`
already splits it by individual key for exactly this reason. The engine's
`climate` + `weather` groups plus `planet` are **two of the spec's stages**
(01 and 08). The spec's 09 and 10 have **no engine parameters at all**. And the
spec's 06 Erosion covers six passes of which the engine implements **one**.

**(b) The spec's dependency graph has a cycle.** Stage 06 Erosion "needs 04,
08"; stage 08 Climate "needs 01, 02, 06" and, per the mockup, "feeds back into
06". `StageGraph` is acyclic by construction (`add_stage(name, upstream)` over
already-added ids), and its real edges are height → hydrology → climate → civ.
A cyclic ten-node graph is not a re-labelling of a four-node DAG.

**(c) Commit does not re-run anything.** §5.2: *"committing… re-runs erosion,
hydrology and climate once."* `commit_sculpt_pass` bakes the stack, runs the
River/Lake water hooks, and **marks tiles dirty** — it deliberately does not
recompute. `UNIFIED_TOOL_PLAN.md` measured the eager version (terrain ~5.1 s,
terrain+civ ~7.07 s at 2048², excluding climate/erosion/hydrology) and rejected
it explicitly; there is also no per-stage re-run entry point to call. The
mockup's own status line ("downstream update: rivers · deferred") agrees with
the engine; §5.2's prose does not.

**(d) The sculpt brush's units and ranges are different quantities.**

| Spec §5.2 | Engine (`SculptGlobals`, from `SCULPT_GLOBAL_DEF`) |
|---|---|
| Radius 0.05–20.0, default 2.00 | `brush_size` 6–200 **cells**, step 1, default 32 |
| Strength ±500 m, default +120 m, ⇧ inverts | `intensity` 0–1.5, **dimensionless**, default 1.0; sign is a sub-mode, not a value |
| hardness 0.35 (§4) | `hardness` 0–1, default **0.5** |
| Smooth 0–1, default 0.50 (§5.2) | probably the same `hardness` — **uncertain**, two spec controls may be one parameter |
| Operation: Set · Add · Subtract · Multiply · Min · Max | internal `Mode::{Add, Set}`, chosen **per feature**, not by the user |
| Falloff: Smooth · Linear · Sharp · Constant · Custom | one curve: `smoothstep(0,1,(R−dist)/feather)` |
| Rotation 0–360° | does not exist |
| — | `noise_scale`, `octaves`, `persistence`, `lacunarity`, `edge_noise` all real, all absent from §5.2's table |

**(e) The sculpt feature vocabulary overlaps by five of thirteen.** Engine
(reference-derived, bit-exact over 23 golden cases): Mountains, Hills, Ridge,
Plateau, Cliff, Canyon, Valley, River, Lake, Basin, Coastline, Volcano,
Freehand. Spec: mountain range, volcano, plateau, rift, canyon, crater, island
arc, basin, dune field. **Four spec features do not exist** (rift, crater,
island arc, dune field) and **eight engine features are dropped** — including
River and Lake, which own the only special commit path in `sculpt_commit`.
Presets disagree too: the engine has **eight global** presets (Rolling Hills,
Alps, Rockies, Badlands, Volcanic Isle, Mesa, Karst, Glacial Valley), each
overriding exactly one global; the spec shows **four per-feature** presets for
mountain range alone.

**(f) The asset family taxonomy disagrees three ways.** Engine `Family`: 8
frozen variants (Textures, Biomes, Terrains, Icons, Settlement, Trait, Poi,
Custom). Spec §2.3/§8: "24 families" in three groups. The mockup's own rail:
**11** families plus 2 collections. Also inside §8: slot **rename** is defined
only for custom slots in the engine (the vocabulary's names are constants);
**anchor** is a per-family property that `sprite_draw_rect` depends on, not a
per-slot user choice; and the "active variant" the inspector shows has no
counterpart, since variant selection at render time is weighted and seeded.

**(g) Multidirectional lighting: 8 vs 6 vs 10.** `TerrainAppearance::relief_lights`
is a count, so the spec's 8 is settable — but the engine's default look uses 6
and `Ultra` uses 10, and `1` is the special value that reproduces the
reference's exact single-sun shading bit-for-bit. Naming: §7's "Strength 0.62"
is presumably `relief_gain`; **uncertain** whether it means gain,
directionality, or both.

**(h) Style presets.** Spec: Atlas / Parchment / Physical / Ink. Engine:
`TerrainAppearance::default()` (the atlas look), `js_reference()` (bit-identical
JS output), and four `QualityTier`s. Only "Atlas" plausibly maps. The three
others are new looks, not presets over existing fields — and there is no
appearance-preset serialization (`TerrainAppearance` does not derive
`Serialize`).

**(i) Fifteen-plus real, working controls have no home in the spec.**
`climate.{rain_k, evap, rain_dep, ocean, wind_manual, wind_dir_deg, press_k,
zonal_k, ocean_hum, bulk_evap, terrain_wind_deflection}`, `crater.age`,
`tect.ridged`, `tect.dynamic_lithology`, `tect.lloyd`, `carve_rivers`. All are
live in the current shell's six Generate dialogs and verified control → engine
→ visibly different world. `carve_rivers` in particular decides whether river
topology exists at all.

**(j) `Units` appears twice** — Preferences ▸ Application ▸ Units and §5.1
stage 02. One value, two homes.

**(k) The vault block exceeds what `MARKDOWN_VAULT_INTEGRATION.md` scopes.**
That document's §1 states V1 is *"deliberately pull-oriented"* and its §33
lists **automatic bidirectional synchronization**, **automatic Markdown
writes**, **Obsidian plugin** and **editor extensions** as V1 non-goals. §9
ships a two-way-sync toggle (off, "read-only now"), `obsidian://` deep links in
exported tiles, and note links in GeoJSON — the latter two appear nowhere in
the vault doc. Its §3 V1 entity scope is Settlements, POIs, Regions and Region
labels; §9 says "settlements, factions and **journeys**", and the vault doc's
own header already warns that POIs and regions as addressable entities *may not
exist in this port* — they do not.

**(l) Smaller factual mismatches.** The mockup has **ten** screens; the spec's
table lists nine. The spec's Assets menu carries a note that "imports live
under Data ▸ Import", but `File ▸ Import asset pack…` is the one import that
works today. The archetype list drops "Classic" (structure disabled), which is
`main.gd`'s current default, and renames `archipelago` to "Islands".

**(m) One exact agreement, worth recording.** Preferences ▸ Graphics ▸ Render
quality's four values — `performance · balanced · quality · ultra` — match
`QualityTier`'s four names exactly, and four `#[func]`s already exist for them.

## 4 · What the old shell built that this design deletes or relocates

| Built and working | Fate under this design |
|---|---|
| **Generate menu — ten stage dialogs, 57 live controls**, built at runtime from `get_param_info()`/`get_param_defaults()` with `EXCLUDED_KEYS`/`PROXY_KEYS`/`ADVANCED_KEYS`, per-stage and global reset, five-level disclosure | **Relocated** into the left dock's ten stages — but the two decompositions differ (§3a), so the mapping is not 1:1 and the 15+ parameters in §3i lose their home. The load-bearing part to preserve is the *discipline*: no range, step, label or default is hardcoded in GDScript. That should survive verbatim. |
| **Simulate ▸ Statistics… / Economy…** — a three-tab world-data browser (Settlements, Provinces, Economy), sortable, filterable, row-click pins the causal chain | **No home in the new design.** The CIVIL domain's right dock is a "Selection inspector", not a roster, and no window is specified for world data. This is the largest piece of finished, verified GUI work at risk of being discarded. Needs an owner ruling (§5). |
| **View ▸ Performance readout…** — six GPU-eligible stages GPU-or-CPU each, plus Godot runtime numbers, plus a present-and-disabled `use_gpu` checkbox carrying its reason | **Relocated** to Preferences ▸ Performance (the GPU rows) and Preferences ▸ Memory ▸ Working set. Content survives; the per-stage GPU table has no explicit row in the spec and should not be lost. |
| **Left tool rail — 16 tools across 5 groups**, honestly inert, each with a tooltip naming what does not work | **Deleted as a region.** Only the terrain brushes reappear, inside Sculpt. Select/inspect, Pan, Point sample, Biome paint, Place settlement, Draw route/way, Territory/faction, Label, Icon stamp, Measure and Region select/export have **no home in the new left dock at all** — and `UNIFIED_TOOL_PLAN.md` milestones C, D and E built golden-verified engine halves for almost every one of them. A design that has no surface for them strands finished engine work. |
| **Workspace tabs** (WORLD/CIVILIZATION/INFRASTRUCTURE/CARTOGRAPHY/RENDER) + `TAB_TO_GROUP_INDEX` emphasis logic | **Relocated** to the vertical domain rail. Same five subjects, renamed (CIVIL/INFRA/CARTO). Geometry change; the selection logic is reusable. |
| **Right dock — Layers (five independent toggles), Properties (click-to-pin causal chain), Sample (live hover)** | **Restructured** into eight selection-driven contexts. Layers survives as one context; Properties' causal chain becomes the Settlement context; Sample grows from 2 real fields to 16, ten of which need the per-cell sampler. |
| **File ▸ New world** — the World Setup dialog (extent, six width presets, resolution, seven aspect presets, live derived readout, two conditional warnings) | **Survives, and is richer than the spec asks for.** The spec names four fields; this dialog has more, and its aspect/derived-readout work is the GUI half of the non-square-map effort. Do not narrow it to the spec's list. |
| **File ▸ Import asset pack…**, **File ▸ Open project (.zip)…**, **Help ▸ Credits** | **Relocated** (Assets, File, Help). All three keep working. |
| **`theme/dark_theme.tres`** | **Largely re-authored.** §11 mandates no panel fills, hairline separation and radius 0; the current theme uses filled `StyleBoxFlat` panels. Its known gaps (no `PopupMenu`, tooltip or scrollbar entries) carry forward regardless. |
| **Render menu, Edit menu, Assets menu (all inert)** | **Deleted.** Nothing real is lost. |

## 5 · Decisions only the owner can make

1. **Does the ten-stage pipeline mean real per-stage re-execution?** If yes,
   that is a large engine re-architecture with no scope document. If it is a
   presentational grouping over one-shot generation, most of §5.1's 24 engine
   gaps collapse into presentation. Everything downstream of this answer
   changes shape.
2. **Multi-GPU**: build device selection, dispatch modes, VRAM budgeting and
   fallback policy at all? Nothing exists, nothing is scheduled, and the one
   measured GPU finding this project has runs counter to the research
   document's framing.
3. **Two-way vault sync** — the vault doc lists it as a V1 non-goal; the spec
   ships the toggle. Shipping the toggle means shipping the capability.
4. **`obsidian://` links in exported tiles and note links in GeoJSON** — both
   are outside the vault doc entirely. In or out of V1?
5. **The timeline, the six simulation-layer toggles, and Warfare.**
   `VISION.md` already flags temporal simulation and Warfare as requiring an
   explicit product decision, and `HARDWARE_ACCELERATION.md`'s scope correction
   says the engine is a one-shot static generator on purpose. The spec ships a
   whole timeline region regardless.
6. **Storage-location roots** — are `~/Cartalith/{Worlds,Cache/atlas,Packs,Exports}`
   the convention, and what are the Android equivalents? Nothing in this port
   has a path convention today.
7. **Autosave default (5 min)** — and whether autosave is offered before the
   save *writer* exists.
8. **Coordinate systems / EPSG.** `GUI_FEATURE_PARITY_SCOPE.md` recommends
   deferring it because a flat procedural grid has no CRS to convert between;
   the spec makes it a first-class Data-manager route with three CRS choices
   and world-file output.
9. **The asset family taxonomy** — 24 (spec) vs 8 (engine, frozen, ported from
   the reference) vs 11 (mockup). Which is authoritative, and if it is the
   spec's, is the engine's frozen vocabulary being replaced?
10. **Sculpt brush units** — radius in cells or km; strength in metres or the
    engine's dimensionless intensity; and whether §4's `hardness` and §5.2's
    `Smooth` are one control or two.
11. **Do the world-data tables survive?** Statistics and Economy are finished,
    verified GUI work with no home in this design.
12. **Render-quality default per device.** `get_recommended_quality_tier()`
    exists and deliberately applies nothing — its own doc comment says *"what a
    phone should default to is an owner policy decision, not this crate's."*
13. **CPU worker-thread default of cores − 4** — a policy number, not a port.
14. **IBM Plex Mono packaging** (OFL, so a packaging question rather than a
    licensing blocker) and the light-theme token set, which has never been
    built.
15. **`Units` ownership** — Preferences, stage 02, or one proxied control
    behind both (the pattern `main.gd` already uses for `world` and the four
    experimental flags).

---

## Verification

This is a documentation task, so the verification is accuracy. Beyond reading
`lib.rs` and `main.gd` in full, these claims were checked by opening the named
function or handler rather than inferred:

**"wired" spot-checks (11).** `load_save` → `main.gd:1733` `_on_save_file_selected`;
`load_asset_pack`/`has_asset_pack` → `main.gd:1758-1759`; `get_settlements` →
`main.gd:2001`/`2087` (world-data browser + causal chain); `get_provinces` →
`main.gd:2015`; `get_trade_balances` → `main.gd:2025`; `get_gpu_stages_used` →
`main.gd:2331-2333` (with the `has_method` guard); `explain_settlement` →
`main.gd:1518`; `build_territory_texture` / `build_province_boundary_texture`
→ `main.gd:1681-1682`; `generate_sized` / `generate_world_structure_sized` →
`main.gd:1652-1658`; `reference_grid_height` → `main.gd:571-573`;
`get_map_width_km`/`get_map_height_km` → `main.gd:1688-1689` (read *back* after
generation rather than echoed); `set_params`/`get_param_info`/`reset_params` →
`main.gd:1118-1247`.

**"backed, unwired" spot-checks (12).** `set_quality_tier`/`get_quality_tier`/
`list_quality_tiers`/`get_recommended_quality_tier` — read in `lib.rs:1215-1257`,
confirmed present as `#[func]`s, and confirmed **absent** from the full list of
`world_gen.` call sites in `main.gd`/`map_overlay.gd`; `apply_archetype` /
`get_archetypes` / `get_seed` / `get_villages_enabled` / `get_param_groups` —
same, all `#[func]`, none called; `PassBuffer::{push, remove, set_hidden,
move_up, move_down, undo, redo, preview_into, commit, discard}` read in
`cartalith-spatial/src/pass.rs`; `commit_sculpt_pass` read in
`cartalith-engine/src/sculpt_commit.rs`, including the ordering of the
re-clamp, the per-stamp `enforce_channel_descent` and the lock — and confirming
it marks tiles dirty rather than recomputing; `SculptGlobals` ranges and
defaults read from `cartalith-terrain/src/sculpt.rs:782-830`; the 13 `Feature`
variants and 8 `SCULPT_PRESETS` read at `sculpt.rs:375-407` and `894-943`;
`TerrainAppearance`'s lighting fields read at `cartalith-godot/src/render.rs:200-245`;
`AssetDB`/`AssetCollections`/`AssetValidator::run` method lists read from
`cartalith-assets/src/library.rs`; `archive::{write_pack, zip_store}` read from
`archive.rs`; `export_geojson` and `region_export`'s public surface read
directly; `measure`/`measure_path` read in full.

**"engine gap" spot-checks.** Grepped the whole workspace for a GeoJSON
*parser*, a heightmap importer, a save writer and a sprite-sheet slicer —
**zero hits each**. Read `cartalith-engine/src/staleness.rs` in full to confirm
the four-stage acyclic graph and its tests. Read `cartalith-gpu/src/lib.rs`'s
adapter functions to confirm single-adapter initialisation. Read
`cartalith-godot/src/render.rs`'s module doc for the explicit "no
elevation-keyed colour breakpoint ramp" statement. Confirmed `CivData`'s
retained fields by reading the struct: biome, soil, lithology and slope rasters
are **not** among them.

**Marked uncertain rather than guessed**: the analysis-field switcher's real
exposure (`FUNCTIONAL_CONTRACT.md` §11 leaves it unresolved and so does this
pass); whether §5.2's `Smooth` and §4's `hardness` are one engine parameter;
whether §7's "Strength" means `relief_gain` or `relief_directionality`; whether
§9's Retina @2x is `refine_tile`; what §5.1 stage 07's "lakes as water" and
§7's Blocks/Verticality footer tabs actually do; and what the domain rail's
expanded sub-node lists should contain, which the spec does not enumerate.
