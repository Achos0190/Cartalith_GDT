# Cartalith command inventory, read from the shipped GDScript (2026-10-31)

Every command surface in `cartalith-native/godot-project/shell/` that a phone or tablet layout has to
give a home: the program menus (`menus.gd`), the phone MORE menu (`phone_menu.gd`), the phone chrome
(`dcc_shell.gd`), search, the command index and quick bar, the map context card, ring and phone tool
fan, and the keys that are not menu rows. Nothing in the repository was edited.

## 0. Method, legend and limits

- **Read at the symbols, not from design documents.** Menu builders followed from `DccMenus.build()`:
  `_file` (+ `_build_file_export`, `_refresh_recent_worlds`), `_edit` (`_build_undo_redo`,
  `_build_edit_actions`, `_build_reset_generation`, `_build_reset_stage_menu`), `_assets`
  (+ `_build_asset_pack_submenu`, `_build_landmark_types_menu`, `_build_landmark_family`), `_data`
  (the World menu; + `_build_vault_rows`, `_route_submenu`, `_add_route_rows`), `_story`, `_preferences`
  (+ the GPU, CPU, lighting, relief, LOD, atlas, theme, units and handedness helpers), `_window`
  (+ `_build_view3d_rows`, `_build_layouts_menu`, `_refresh_layouts_menu`, `_refresh_open_windows`),
  `_help`, and the tablet-only `_overflow`, `_tablet_world` and `_world`.
- **Loop-generated rows are expanded from the constants they loop over**, each named on the row.
  Engine-sourced lists that are static in Rust were resolved there: landmark kinds
  (`cartalith_civ::landmark::kinds()`), quality tiers (`QualityTier::ALL`, `render.rs`), sculpt features
  (`cartalith_terrain::sculpt::FEATURE_KEYS`), freehand modes (`FREEHAND_MODES`) and paint layers
  (`paint_bridge::PaintTarget::ALL`).
- **The live tree was not walked.** There is no Godot binary and no built native library in this
  checkout (`which godot godot4` printed nothing; `cartalith-native/target/` does not exist), so every
  row here comes from reading code. Lists whose length depends on the machine or session (recent worlds,
  GPU adapters, the CPU-core ladder, saved layouts, open windows) are given as patterns. Rows that change
  on an older engine build (a `_live` row that falls back to a `_todo`) are noted on the row.
- **Row-kind tag** (first token of every row bullet):
  - `live`: `_live()`, an enabled command with a dispatcher arm.
  - `todo`: `_todo()`. **Drawn disabled and does nothing**; its tooltip says what is missing.
  - `readout`: `_readout()`, a disabled row that carries a live value. Not a command.
  - `signpost`: `_signpost()`, disabled prose that points somewhere else. Not a command.
  - `item` / `check` / `radio` / `submenu`: a raw `add_item` / `add_check_item` /
    `add_radio_check_item` / `add_submenu_item` call, not routed through a helper. `item` rows are
    enabled commands unless the row says otherwise.
  - `band`: a labelled separator (`add_separator("CAPTION")`). `sep`: an unlabelled separator.
  - `×N` after a tag: N rows made by a loop over a fixed table, with the values listed.
  - `pattern`: a list whose length is decided at run time.
- Accelerators use Godot's notation. **Every accelerator in `menus.gd` goes through
  `_bind_accelerator()`, so all of them can be rebound** in Preferences ▸ Keyboard shortcuts….
- `[SM]` marks a row that Window ▸ Session mode disables (`_session_gate_row()`).
- Preferences parent rows are relabelled on every open to `<label>   <checked value>`
  (`_stamp_pref_values()`); the base label is given here.

## 1. The program menu bar (PC), which the phone also builds, hidden

`DccMenus.build()` (`menus.gd`) branches on `DccTheme.is_tablet()`:

- **Tablet:** `☰ · File · World · Story` (see §2.1).
- **PC and phone (the `else` branch, identical):**
  `File · Edit · Assets · World · Story · Preferences · Window · Help`, eight menus.

On the phone the same eight popups are built into a hidden `PhoneMenuModel` host
(`dcc_shell.gd::_build_phone_shell`) and re-presented by `phone_menu.gd` (§3). No other file adds a
top-level menu (`grep -rn "add_menu("` in §9 finds only `menus.gd`), and no file adds rows to a menu
popup from outside `menus.gd` (`grep -rn "get_popup()\.add_"` found nothing).

### 1.0 Menu-bar chrome that is not a menu row (PC and tablet)

Built by `dcc_shell.gd`'s menu-bar builder:

- `CARTALITH` wordmark (not interactive)
- the menus
- `↶` square: undo (`_menu_bar_undo` → `undo_last`)
- `↷` square: redo (`_menu_bar_redo`)
- readouts `top_world`, `top_res`, `top_cpu`, `top_gpu`, `top_mem` (text only)
- `◐` square: `toggle_theme()`, the one-tap dark/light flip

On the phone these readouts move to MORE ▸ Status and the world pill (§3, §4).

The domain rail (desktop and tablet; `DccShell.RAIL_NODES`) is navigation rather than a menu, but
each node needs a phone home:

- WORLD: Generate (mode `a`), Sculpt (mode `b`)
- CIVIL: Landmarks, Factions, Routes & ways, Journey planner
- CARTO: Style, Layers, Labels, Icons

On the phone, WORLD is the GENERATE tab, CARTO is the MAP tab, the Journey planner is the PLAN tab, and
the other CIVIL nodes are reached through MORE (§3).

### 1.1 File (`_file`)

- `live` **New world…** · Ctrl+N · [SM]: opens the New world dialog (seed, extent, resolution, archetype).
- `live` **Open project…** · Ctrl+O: the project picker.
- `submenu` **Recent worlds**: `RecentWorlds`, rebuilt on every open from `DccSettings.recent_projects()`
  (at most `MAX_RECENT` = 10).
  - `pattern` one `item` per project: `<world name> — <seed> · <age>` (age such as `5 d ago`, `3 w ago`;
    the full path is the tooltip). A moved file reads `<basename> — file not found`. Opens the project
    under the busy overlay.
  - `pattern` with no recent projects: `readout` "No recent projects".
- `live` **New world from selection…** · [SM]: replaces the world with a higher-resolution resample of
  the Region-select marquee. Disabled, with its reason, when the engine lacks `region_new_world`, when
  there is no world, while generating, or when no marquee exists.
- `sep`
- `live` **Save project** · Ctrl+S: disabled with no save writer or no world.
- `live` **Save as…** · Ctrl+Shift+S
- `check` **Autosave**: writes `world.autosave.ctl` beside the project; the tooltip carries the interval.
- `submenu` **Autosave interval** (relabelled `Autosave interval   off | every N min`): `AutosaveInterval`
  - `radio` **Off**
  - `radio×3` **Every 1 min**, **Every 5 min**, **Every 15 min** (`AUTOSAVE_MINUTES = [1, 5, 15]`)
- `live` **Revert to last save**: disabled until the world has been saved once.
- `live` **Close project** · Ctrl+W: disabled with no world.
- `sep`
- `submenu` **Export**: `FileExport` (`_build_file_export`)
  - `band` FOR THE TABLE
  - `live` **Player map…** · Ctrl+Shift+E: the map-image dialog with the players' preset (every layer
    except Journey routes).
  - `live` **DM / full map…**: the same dialog with every layer on.
  - `band` SHARE
  - `live` **Export map image…**: one flat PNG, 2K to 32K.
  - `todo` **Print sheet (PDF)…**: "No print or PDF writer exists…"
  - `todo` **VTT battle map (grid + walls)…**: "No virtual-tabletop format exists in this build…"
  - `band` DATA
  - `item×4` the four `DataManagerWindow.ROUTES` rows of group Export, as `<label>   <badge>`:
    **Maps   tiles**, **GIS / GeoJSON   .geojson**, **World Data   map + atlas**, **Assets   .zip**.
    Each opens the Data manager window on that route (the first three are working routes; Assets routes
    to the Asset library).
  - While this submenu is open, Player map…, DM / full map… and Export map image… are disabled when
    `export_map_image_refusal()` gives a reason (no engine API, generating, no world).
- `band` STORAGE LOCATIONS
- `readout×4` **projects   …/<path>**, **tile atlas cache   …**, **asset packs   …**, **exports   …**
  (`DccSettings.ROOT_KEYS` and `ROOT_LABELS`; the full path is the tooltip)
- `live` **Change locations…**: the storage-locations dialog (the same dialog as Preferences ▸ Storage
  locations…).
- `live` **Show project on disk**: opens the project folder in the OS file manager; disabled until a
  project has been opened.
- `signpost` "imports live under World ▸ Import"
- `signpost` "asset packs under Assets"

### 1.2 Edit (`_edit`)

- `live` **Undo** · Ctrl+Z: relabelled `Undo <operation>`. Steps back the look history or the height
  stack, whichever was edited last.
- `live` **Redo** · Ctrl+Shift+Z: relabelled `Redo <operation>`. On an engine build without
  `redo_available` this row is a `todo` instead.
- `live` **Undo history…**: the history ledger in the right dock.
- `sep`
- `live` **Cut** · Ctrl+X: copies the selected icons and labels to the in-process clipboard, then deletes
  them.
- `live` **Copy** · Ctrl+C
- `live` **Paste** · Ctrl+V: adds the clipboard, offset 4 cells, and selects what it added.
- `live` **Delete** · Delete: deletes the selection (the same action as the Delete key).
- `sep`
- `live` **Select all** · Ctrl+A: scoped to the domain. In CARTO it selects icons and labels, in WORLD the
  sculpt draft's stamps; in CIVIL it is disabled.
- `live` **Deselect** · Ctrl+D
- `sep`
- `live` **Find on map…** · Ctrl+F: the search (§5); on the desktop a "Find on map" dialog, on the phone
  the search overlay.
- `sep`
- `live` **Reset generation parameters** · [SM]: puts every generation parameter back to its default;
  takes effect on the next Generate.
- `submenu` **Reset one stage** · [SM]: `ResetStage`
  - `item×10` **01 Planet**, **02 Extent & scale**, **03 World structure**, **04 Tectonics**,
    **05 Volcanism & impacts**, **06 Erosion**, **07 Hydrology**, **08 Climate**,
    **09 Ecology & biomes**, **10 Resources & soils** (`WorldWorkspace.STAGES`). A stage with no
    parameters is disabled, and its tooltip carries that stage's own gap text.

### 1.3 Assets (`_assets`)

- `live` **⧉ Asset library** · Shift+A: the Asset library window.
- `live` **⧉ Sprite sheet slicer (▦)**: the Asset library in slicer mode.
- `live` **Import image…**: opens the Asset library window. This is the same call as ⧉ Asset library
  (`open_asset_library()`); there is no separate picker.
- `live` **Import asset pack .zip…**: loads a pack onto the map and into the library.
- `live` **Open pack in library…**: loads into the library only; the map is unchanged until Apply.
- `live` **Pack report…**: the library's read-only report card.
- `live` **Import folder of art…**: a loose, Wonderdraft or unpacked Dungeondraft folder.
- `live` **Compare pack with library…**: a slot-by-slot diff, then a merge choice.
- `sep`
- `submenu` **Asset pack**: `AssetPack`
  - `band` ACTIVE PACK
  - `readout×5` **NAME   <name|—>**, **AUTHOR   …**, **LICENSE   …**, **SCHEMA   2 · STORED zip**,
    **FILLED   <f> of <c> slots · <n> items**
  - `item` **Pack metadata…**: a modal (a stock `ConfirmationDialog`) for name, author and license.
  - `item` **Validate pack**: runs the validator; read-only.
  - `item` **Export pack .zip…** · Ctrl+Shift+P
- `submenu` **Icon families**: `IconFamilies`, the `AssetLibraryWindow.FAMILIES` entries with
  `texture: false`
  - `item×6` **Feature icons**, **Settlement pins**, **Settlement traits**, **Points of interest**,
    **Sea marks**, **Custom icons**. Each row shows `   <filled>/<capacity>` and opens the Asset
    library scoped to that family.
- `submenu` **Texture sets**: `TextureSets` (`texture: true`)
  - `item×3` **Splat channels**, **Biome textures**, **Terrain textures** (same behaviour)
- `submenu` **Landmark types**: `LandmarkTypes`, a read-only cascade. Every leaf opens CIVIL ▸ Landmarks.
  - `submenu×6` one per family, in `landmark_kinds()` first-seen order, titled from the family *key*:
    **Physical**, **Transport**, **Economic**, **Military**, **Cultural**, **Historical**. Each parent
    is relabelled `<Family>   <a> of <n> armed · <p> placed`. Inside each family:
    - `item×49` (across the six families) one row per type, relabelled
      `<Label>   <cap> max · <placed> placed · <limit>`, `off · was N`, `off` or `not buildable`:
      - Physical (15): Peak, Ridge, Saddle\*, Cliff, Gorge, Cave, Waterfall, Spring, Lake, Delta\*,
        River confluence, Volcanic feature, Rock formation, Glacial feature\*, Ancient forest\*
      - Transport (8): Mountain pass, River crossing\*, Ford, Bridge site, Road junction,
        Caravan station, Portage, Harbour
      - Economic (6): Mine, Quarry, Salt works\*, Resource extraction site, Market site, Trade depot
      - Military (6): Fort, Watchtower, Fortified pass, Fortified crossing, Battlefield, Border marker
      - Cultural (8): Shrine\*, Temple\*, Sacred grove\*, Sacred mountain\*, Pilgrimage site\*, Tomb\*,
        Monument\*, Ceremonial site\*
      - Historical (6): Ruin\*, Abandoned settlement, Ancient road, Historic battlefield\*,
        Destroyed fortress\*, Historic crossing\*
      - (\* `buildable: false` in `cartalith_civ::landmark::kinds()`, so the row reads "not buildable")
    - `sep` (in each family)
    - `item×6` **Open <Family> in the dock** (one per family): CIVIL ▸ Landmarks, opened at that family.
    - `signpost×6` "Rows here read state; they do not arm a type." (one per family)
  - `item` **Placed by hand   <n>**: the count of hand-stamped icons; opens CIVIL ▸ Landmarks.
  - `sep`
  - `item` **Landmark icons…**: the Asset library on the Points of interest family.
  - `item` **Landmark label style…**: CARTO ▸ Labels.
  - (On an engine without `landmark_kinds()`, a single `todo` "No landmark types" replaces the families
    and Placed by hand.)
- `sep`
- `live` **Apply library to map**
- `live` **Clear library…   destructive**: confirms in a modal card before clearing.

### 1.4 World (`_data`; the menu was called Data until SM-1)

- `live` **⧉ Data manager**: the Data manager window.
- `band` RECORDS
- `live` **Factions…**: the Factions window (identity, territory, settlements, economy, military).
- `live` **Places…**: the CIVIL dock's Settlements category (`select_domain_category`).
- `live` **Markdown vault…**: the vault panel, unscoped (link and attach notes; index of missing and
  orphan notes).
- `live` **Browse & edit a note…**: the vault panel with no entity scope; raw-text preview and edit.
- `live` **Places without notes…**: the vault panel's places-without-notes view.
- `live` **Timeline**: CIVIL ▸ Timeline (the year snapshots), and shows the timeline bar if it is hidden
  (the phone skips the bar).
- `band` TECHNICAL
- `live` **World data tables…**: the World data window.
- `submenu` **Import**: `WorldImport`, the `ROUTES` rows of group Import. Each opens the Data manager on
  that route.
  - `item×5` **Maps   tiles** (no importer; the pane says so), **Heightmaps   PNG**, **GIS / GeoJSON**,
    **World Data   .ctl** (the same loader as Open project…), **Assets   → Assets** (routes to Assets)
- `submenu` **Sources & validation**: `WorldSources`
  - `band` SOURCES
  - `item×3` **External Sources**, **Connected Sources**, **Source Registry**. No source registry exists,
    so all three panes are disclosed gaps.
  - `band` VALIDATION
  - `item×3` **Definitions** (working), **Check Data** (gap), **Repair / Normalize** (gap)
- `signpost` "Export is in File ▸ Export"

The route rows are plain enabled `add_item`s. Whether a route works is disclosed only inside the Data
manager window, never on the menu row (`DataManagerWindow.ROUTES[*].kind` is `live`, `gap` or `route`).

### 1.5 Story (`_story`; on all three form factors)

- `band` TIME
- `live` **Timeline** · Ctrl+Shift+T: the same action as World ▸ Timeline.
- `live` **Political history of this place…**: the Place editor on the selected settlement's Political
  history tab. While the menu is open it is disabled, with a reason, unless a settlement is selected and
  no generation is running; it is re-enabled when the menu closes.
- `live` **Faction history…**: the Factions window on its History tab.
- `band` JOURNEYS
- `live` **Journey planner**: has no accelerator column on purpose; Shift+J is an `app.gd` key (§7).
- `live` **Saved journeys…**: lists the world's saved journeys; picking one opens it in the planner.
- `live` **Travel library…** · Shift+L: animals, vehicles, vessels and parties.
- `band` CONFLICTS & CAMPAIGNS
- `live` **Conflicts…**: the conflict list; picking one selects it on the map and in the right dock.
- `live` **Draw a conflict**: selects CIVIL and arms the Conflict tool.
- `readout` **Campaigns this year**: relabelled `Campaigns in year <Y>: <N>`, or `… — generating`,
  `… — no world`, `… — not in this build`; the tooltip names up to six active conflicts.
- `band` NOTES
- `live` **Notes for this place** · Ctrl+Shift+N: opens the selected settlement's vault note, or attaches
  one. Gated like Political history.
- `live` **Places without notes…**: the same id and destination as the World row.
- `live` **Assistant panel…**: the optional LLM prose and Vault assistant. Always enabled; the panel says
  why it cannot send.

### 1.6 Preferences (`_preferences(p)`, PC and phone)

- `band` PERFORMANCE
- `check` **GPU acceleration**: relabelled `GPU acceleration   <backend> · on|off`. Toggles the `use_gpu`
  parameter for the next Generate. Disabled during a generation.
- `submenu` **Devices**: `GpuDevices`, which enumerates adapters only on its first open. Falls back to a
  `todo` "Devices" on an engine without the multi-GPU API.
  - `check` **Automatic (highest-performance GPU)**
  - `sep`
  - `pattern` one `check` per adapter: `<name> · <kind> · <backend>` (software rasterisers are listed but
    disabled). With no adapter, a `readout` "No GPU detected".
  - `sep`
  - `pattern` a `readout` "Memory: not measured yet", or one per device:
    `<name>: N MB allocated (M MB reserved)`
  - `sep`
  - `item` **Rescan devices…**
- `submenu` **Multi-GPU mode**: `GpuMultiMode` (falls back to a `todo`)
  - `radio` **Single device** (the default)
  - `radio` **Split tiles**
  - `radio` **Alternate frames**: always disabled ("Not built").
- `submenu` **CPU worker threads**: `CpuWorkerThreads` (falls back to a `todo`)
  - `radio` **Automatic**
  - `pattern` up to five more `radio` rows, with duplicates removed, for core count c:
    **1 — 1 thread**, **⌊c/4⌋ — quarter**, **⌊c/2⌋ — half**, **c−4 — cores - 4 (the spec's default)**,
    **c — all cores**
  - `readout` the running-count line: `Running N workers now`, `Running N workers — M takes effect at
    next start`, `Pool not started yet — the next choice applies immediately`, or `Running count
    unreadable while a generation holds the engine`.
- `submenu` **VRAM budget**: `GpuVramBudget` (falls back to a `todo`)
  - `radio×8` **No cap**, **1 GB**, **2 GB**, **4 GB**, **8 GB**, **12 GB**, **16 GB**, **24 GB**
    (`GPU_VRAM_CHOICES`)
  - `readout` `Current grid WxH needs about N MB`, or `Estimate available after the first generate`
- `submenu` **Fallback when VRAM full**: `GpuVramFallback` (falls back to a `todo`)
  - `radio` **CPU tile pass** (the default)
  - `radio` **Reduce working res**: always disabled ("Not built").
  - `radio` **Fail with error**
- `live` **Try the GPU again**: enabled only after a GPU readback failure; clears the ban. Becomes a
  `todo` on an engine without both bindings.
- `band` GRAPHICS
- `submenu` **Render quality**: `QualityTiers`
  - `radio×4` **performance**, **balanced**, **quality**, **ultra**. These are the engine's own lower-case
    names (`QualityTier::ALL`), drawn verbatim.
- `todo` **Anti-aliasing · anisotropy**: "…the 3D view … draws no terrain yet…"
- `signpost` "Colour management — CARTO ▸ Colours ▸ Colour management"
- `todo` **3D viewport defaults**: "…they arrive with its terrain."
- `submenu` **Lighting rig defaults**: `LightingRig`, the defaults a fresh Generate starts from
  - `readout` the source line: `Following the engine`, `<n> of 4 stored here` or `All four stored here`
  - `submenu` **Azimuth**
    - `radio×8` **0 deg**, **45 deg**, **90 deg**, **135 deg**, **180 deg**, **225 deg**, **270 deg**,
      **315 deg** (`LIGHT_AZ_STEPS`)
  - `submenu` **Elevation**
    - `radio×7` **15 deg**, **25 deg**, **35 deg**, **45 deg**, **55 deg**, **65 deg**, **75 deg**
      (`LIGHT_ALT_STEPS`)
  - `submenu` **Ambient**
    - `radio×6` **0.00**, **0.15**, **0.25**, **0.35**, **0.50**, **0.70** (`LIGHT_AMB_STEPS`)
  - `submenu` **Multidirectional**
    - `radio×6` **1 light (single sun)**, **2 lights**, **4 lights**, **6 lights**, **8 lights**,
      **12 lights** (`LIGHT_NUM_STEPS`)
  - `sep`
  - `item` **Use the engine's own rig**: forgets the four stored values.
- `submenu` **Relief exaggeration**: `ReliefExag`
  - `radio×3` **1x**, **2x**, **4x** (`DccSettings.RELIEF_EXAG_CHOICES`)
  - `sep`
  - `item` **Use the engine's own 3.4x**
- `band` TILES & LOD
- `submenu` **Tiled LOD**: `TiledLod`
  - `radio` **Auto on zoom**
  - `radio` **Manual**
  - `sep`
  - `item` **Enter deep detail now**: disabled while already in deep detail.
  - `item` **Leave deep detail**: enabled only in deep detail and in Manual mode.
- `submenu` **Tile size · LOD levels**: `AtlasTileSize`
  - `radio×3` **256 px**, **512 px**, **1024 px** (`ATLAS_TILE_SIZES`; disabled once chunks are baked)
  - `sep`
  - `submenu` **LOD levels**: the bake depth, the same setting as WORLD ▸ Finalize ▸ Bake depth.
    - `radio×9` **LOD 0–0   1 tile**, **LOD 0–1   5 tiles**, **LOD 0–2   21 tiles**,
      **LOD 0–3   85 tiles**, **LOD 0–4   341 tiles**, **LOD 0–5   1 365 tiles**,
      **LOD 0–6   5 461 tiles**, **LOD 0–7   21 845 tiles**, **LOD 0–8   87 381 tiles**
- `submenu` **Atlas cache**: `AtlasCache`
  - `readout` the store line, `<n> chunks · <size> · …` ("No atlas in this build" on an older engine)
  - `submenu` **Size cap · GB**: a `todo` on an engine without `atlas_evict_to`.
    - `radio×7` **No cap**, **0.5 GB**, **1 GB**, **2 GB**, **4 GB**, **8 GB**, **16 GB**
      (`DccSettings.ATLAS_CAP_CHOICES`)
  - `item` **Refine detail for the current view**
  - `item` **Refine viewshed for the current view**: a `todo` on an engine without the binding.
  - `readout` **Viewshed: never refined** (the refine status; absent when the row above is a `todo`)
  - `sep`
  - `item` **Export atlas…**: a `todo` on an engine without the binding.
  - `item` **Import atlas…**: a `todo` on an engine without the binding.
  - `sep`
  - `item` **Clear atlas cache now…**: the same action as Clear caches….
- `submenu` **Chunk debug overlay**: `LodDebug`
  - `check×3` **Grid**, **Colors**, **Labels** (drawn only while the tiled LOD view is up)
  - `sep`
  - `check` **Show tile borders on the map**: the export tile split, dashed over the map.
- `band` MEMORY
- `submenu` **Undo history**: `UndoBudget`
  - `radio×5` **64 MB**, **128 MB**, **256 MB**, **512 MB**, **1024 MB** (`UNDO_BUDGETS_MB`; each gains
    `— N steps here` once a world exists)
  - `sep`
  - `item` **Clear undo history now**
- `readout` **Working set   <used> of <RAM>**
- `live` **Clear caches…**: deletes this world's baked chunks, after a confirmation.
- `band` APPLICATION
- `live` **Storage locations…**
- `live` **Assistant…**: the LLM assistant settings window.
- `live` **Quick bar…**: the Customise window for pinned commands (§5).
- `check` **Open panels in their own window**: **desktop only**. Built only when `PopoutHost.supported()`,
  so it is absent on the phone and the tablet.
- `submenu` **Theme**: `ThemeChoice`
  - `radio` **Dark**
  - `radio` **Light**
  - `radio` **Follow system**: disabled where the OS reports no dark/light preference.
- `submenu` **Units**: `UnitsChoice`
  - `radio×3` **Kilometres**, **Miles**, **Nautical miles** (`DccUnits._LABEL`)
- `submenu` **Handedness**: `HandChoice`. Only the tablet's touch-hold card reads it.
  - `radio` **Left**
  - `radio` **Right**
- `live` **Keyboard shortcuts…**: the editable shortcuts dialog (rebinds the menu accelerators).

### 1.7 Window (`_window`)

- `check×5` **Left dock**, **Right dock**, **Timeline**, **Status bar**, **Domain rail**
  (`WIN_REGION_IDS`, `WIN_REGION_LABELS`; the checks are re-read from the real state on every open)
- `sep`
- `check` **Session mode** · Ctrl+Shift+M: a mode for the table. Hides WORLD, disables the [SM] rows and
  switches to the "Session" layout. Refused while generating or while a draft is open.
- `check` **Diagnostics overlay** · Shift+D
- `check` **3D view**: switches the viewport to 3D; the tooltip is the capability disclosure.
- `submenu` **3D capability**: `View3DCapability`
  - `readout` **3D tier: <Off|Basic|Standard|Full|not tested>[ (forced)]**
  - `live` **Re-test 3D**
  - `submenu` **Force tier**:
    - `radio` **Automatic (as measured)**
    - `radio×4` **Off**, **Basic**, **Standard**, **Full** (`View3DGate.TIER_NAMES`)
- `sep`
- `submenu` **Workspace**: `WorkspaceList`
  - `radio×3` **World** [SM], **Civilization**, **Cartography** (`DccShell.DOMAINS`)
- `submenu` **Open windows**: `OpenWindows`, rebuilt on every open
  - `pattern` one `check` per visible dialog among **New world…**, **World data tables**,
    **Generation info**, **Data manager**, **Asset library**, **Travel library**; picking one raises it
  - `pattern` with none open: `readout` "No windows open"
- `sep`
- `live` **Reset layout**
- `submenu` **Layouts**: `Layouts`, rebuilt on every open
  - `item` **Default (reset layout)**
  - `pattern` one `item` per saved layout. Seeded once on first run: **Generate**, **Sculpt**,
    **Cartography**, **Journey** (`SEED_LAYOUTS`). **Session** appears after Session mode is first used.
    User-saved names follow.
  - `sep`
  - `item` **Save layout as…**
  - `item` **Forget layout…**: a `todo` while no layouts are stored.

### 1.8 Help (`_help`)

- `live` **Documentation**: opens the repository's documents in the file manager. It is a `todo` when no
  `README.md` sits beside the project, which is true of any installed build.
- `live` **Keyboard shortcuts…**: the read-only shortcuts list, read from the live menus.
- `live` **Credits & academic principles**
- `live` **Generation info…**: every generation parameter, as text.
- `live` **Save diagnostic report…**: a review panel, then a local text file.
- `live` **About**

## 2. Tablet, and rows that differ by form factor

### 2.1 The tablet bar: `☰ · File · World · Story`

File (§1.1) and Story (§1.5) are identical on the tablet.

#### ☰ (`_overflow`)

- `live` **Undo** · Ctrl+Z
- `live` **Redo** · Ctrl+Shift+Z (a `todo` on an older engine, as on PC)
- `live` **Undo history…**
- `sep`
- `live` **⧉ Asset library** · Shift+A
- `submenu` **Landmark types**: the full cascade in §1.3 (not repeated here).
- `sep`
- `live` **Toggle theme**: flips the palette now on screen. Leaves Follow system if it was set.
- `submenu` **Units**: relabelled `Units — kilometres|miles|nautical miles`.
  - `radio×3` **Kilometres**, **Miles**, **Nautical miles**
- `sep`
- `live` **Keyboard shortcuts…** (Help's read-only list)
- `live` **About**
- `sep`
- `submenu` **Edit**: `_edit(p, false)`. Cut, Copy, Paste, Delete, sep, Select all, Deselect, sep,
  Find on map…. No Undo, Redo or reset rows.
- `submenu` **Assets**: `_assets(p, false)`. §1.3 without ⧉ Asset library and Landmark types.
- `submenu` **Preferences**: `_preferences(p, false, true)`. §1.6 without Units (Theme is kept), and
  without the pop-out row.
- `submenu` **Window**: §1.7 in full.
- `submenu` **Help**: `_help(p, false)`. Documentation, Credits & academic principles, Generation info…,
  Save diagnostic report…. No Keyboard shortcuts… and no About.

#### World (`_tablet_world` = `_world`, then a separator, then `_data`)

- `live` **Run pipeline** · Ctrl+R · [SM]: the WORLD dock's guarded Generate, which asks before
  discarding hand-authored work. This accelerator exists only on the tablet.
- `sep`
- `live` **Reset generation parameters** · [SM]
- `submenu` **Reset one stage** · [SM]: the ten stages of §1.2.
- `sep`
- `radio` **Generation pipeline** · [SM]: WORLD mode `a`.
- `radio` **Sculpt** · [SM]: WORLD mode `b`.
- `sep`
- `live` **Seed & size…** · [SM]: opens New world (the same dialog as File ▸ New world…).
- `sep`
- then all of §1.4 World, from ⧉ Data manager to the "Export is in File ▸ Export" signpost.

The tablet keeps the §1.0 chrome (`↶ ↷`, the readouts, `◐`). The readouts use `FS_MENU` size there.

### 2.2 Rows gated or changed by form factor

| Row or behaviour | PC | Tablet | Phone | Where decided |
|---|---|---|---|---|
| Bar shape | 8 menus | ☰ · File · World · Story | 8 menus built hidden, re-presented by MORE | `DccMenus.build()` |
| Preferences ▸ Open panels in their own window | `check` | **absent** | **absent** | `PopoutHost.supported()`: needs `FEATURE_SUBWINDOWS`, not mobile, not phone, not tablet |
| Undo / Redo / Undo history… | Edit | ☰ top level | Edit (MORE ▸ "Not on the MORE list"); also the `↶` chip | `_build_undo_redo` |
| Reset generation parameters, Reset one stage | Edit | World menu | Edit | `_build_reset_generation` |
| ⧉ Asset library, Landmark types | Assets | ☰ top level | Assets | `_assets(full)` |
| Theme | Preferences ▸ Theme, `◐` square | ☰ ▸ Preferences ▸ Theme; ☰ ▸ Toggle theme; `◐` | MORE ▸ Preferences (chips at the top); `⋮` ▸ Theme | `_build_theme_submenu`, `_overflow` |
| Units | Preferences ▸ Units | ☰ ▸ Units (`Units — <value>`) | MORE ▸ Preferences (chips at the top) | `_build_units_submenu(tablet)` |
| Help ▸ Keyboard shortcuts…, About | Help | ☰ top level | Help | `_help(full)` |
| Run pipeline (Ctrl+R), Generation pipeline / Sculpt, Seed & size… | absent | World menu | absent (GENERATE tab instead) | `_world` |
| World ▸ Timeline, Story ▸ Timeline | CIVIL ▸ Timeline, and shows the timeline bar | same | CIVIL ▸ Timeline only; returns before the bar | `_open_timeline` |
| Window ▸ Left dock / Right dock | toggle the docks | toggle the docks | open or close the phone's left or right sheet | `app.gd::toggle_region` |
| Window ▸ Status bar | toggle | toggle | `set_status_region_shown` | `toggle_region` |
| Window ▸ Domain rail | the rail | the rail | hides the MAP, GENERATE and PLAN tabs (MORE stays) | `_build_phone_menu_bar` |
| Window ▸ Session mode | gates the [SM] rows | also the tablet World rows | also hides the GENERATE tab; the seeded Session layout is CARTO/style (CIVIL/planner elsewhere) | `_session_seed`, `_pick_phone_tab` |
| Preferences ▸ Handedness | present, no effect | used by the touch-hold card | present, no effect | `context_broker.present` |
| Help ▸ Documentation | `live` from a repository checkout | `todo` on installed builds | `todo` on installed builds | `_docs_dir()` |
| Pack metadata… dialog | centred dialog | centred dialog | `phone_window` / `phone_fit` / `phone_present` | `_open_pack_metadata` |
| Disabled-row reason | tooltip | tooltip | drawn as the row's second line | `phone_menu.gd::_popup_row` |

## 3. The phone MORE menu (`phone_menu.gd`)

MORE is one navigation stack, with a back button and a close button below the root. Each screen still
fires the real `PopupMenu` items (`_activate` emits `id_pressed` and `index_pressed`), so no handler is
reimplemented.

Two ways a desktop popup is re-presented:

- **Bespoke screens** (`_rest_of`): rows in the popup's own order, with bands for captioned separators.
  A submenu with two or more enabled radio items and no nested submenu is drawn **inline as a chip row**
  (`_expandable`), except `AtlasCache` and `GpuDevices` (`NO_EXPAND`). Other submenus become drill rows
  with a three-name subtitle and a count.
- **Drilled menus** (`_fill_popup`, used for Edit and Window and for every nested submenu): every
  submenu is a drill row. A check row becomes a switch; a radio row gets a radio mark. A disabled row
  shows its tooltip as its second line.

### 3.1 Root (`_fill_root`), in display order

1. **Quick** (band). Only when something is pinned or the Assistant chip is shown (`_fill_quick`,
   `QuickBarPhone.entries`); with nothing pinned there is no band, no row and no gap.
   - Assistant row first: "Open the assistant panel".
   - One row per pin. The subtitle is `<Group> menu`, or the reason when the pin cannot run.
   - **Customise…**: "Pin, unpin and reorder commands"
2. The `ROOT_ROWS`, each glyph · label · subtitle → screen:
   - `⧉` **Project** · "save · recent · storage" → `project`
   - `◍` **Civilization** · "settlement · POI · way tools" → `civ`
   - **Story** (band, SM-2)
     - `◷` **Timeline** · "timeline · political · faction history" → the Story TIME band
     - `→` **Journeys** · "planner · saved journeys · travel library" → JOURNEYS band
     - `◇` **Conflicts & campaigns** · "list · draw · this year" → CONFLICTS & CAMPAIGNS band
     - `▤` **Notes** · "this place · places without notes" → NOTES band
   - **World** (band)
   - `⇅` **Data manager** · "import · export · sources · validation" → `data`
   - `▦` **Asset library** · "families · slots · packs · landmark types" · badge `<filled> / <total>`
     → `assets`
   - `≋` **Travel library** · "animals · vehicles · vessels · parties" → `travel`
   - `◷` **Simulation** · live subtitle `year <Y> · <state>` or "no cursor yet — generate a world first"
     → `sim`
   - `⚙` **Preferences** · "theme · units · performance · graphics" → `prefs`
   - `?` **Help & about** · "documentation · shortcuts · credits · about" → `help`
   - A glyph the mono font lacks falls back to the submenu arrow.
3. **Status** (band). Readouts, drawn only when they hold a real value: Next (the `hint` slot, wrapped),
   World, Resolution, Generator (the `pass` slot), Stale, Autosave, Atlas, CPU, GPU, Memory.
   - **Recompute stale stages**: an action row, shown only when stages are stale and a recompute is
     possible.
4. **Not on the MORE list** (band): every program menu no screen covers, which today is **Edit** and
   **Window**. Each is a drill row (subtitle: its first three enabled rows; trail: its row count) into
   the whole desktop popup.

### 3.2 The screens

**`project`** (`_fill_project`)

- File rows first: **Save project**, **Save as…**, **New world…**, **Open project…**
- **Recent worlds** (band): File ▸ Recent worlds drawn inline (or "No recent projects")
- **Autosave** (band): **Autosave** switch; **Interval** chips Off · Every 1 min · Every 5 min ·
  Every 15 min
- Then the rest of File in File's own order: **New world from selection…**, **Revert to last save**,
  **Close project**, **Export** (a drill into the FOR THE TABLE, SHARE and DATA bands), the
  **STORAGE LOCATIONS** band with its 4 readouts, **Change locations…**, **Show project on disk**, and
  the 2 signposts.

**`civ`** (`_fill_civ`)

- Info: "Arming a tool closes this screen — tap the map to place. Esc, or arming another tool, disarms."
- Six tool rows from `CIV_TOOLS`. Tapping one arms the tool and closes MORE; the armed one is badged
  ARMED.
  - **Settlement**: "tap drops a place · class and snapping from the CIVIL dock"
  - **Territory**: "drag to claim · commit or discard from the tool options"
  - **Territory lasso**: "taps place a ring · commit assigns its inside to the faction"
  - **Way**: "taps append waypoints · commit from the dock"
  - **Route**: "sea and river legs between two ports"
  - **Conflict**: "taps draw a front, arrow, siege or battle · commit, then fill it in the dock"
- **Point of interest**: a dim row. "No POI placement tool exists in this port…"
- **Landmark generation**: "`<types>` types · `<families>` families · cap + spacing" → `landmarks`
- **Open journey planner**: "Party, season, carriage, stages and cost."
- **Civilization dock**: selects CIVIL and opens the left sheet.

**`story`** (`_fill_story(band)`): the Story popup's rows between that band's separator and the next one.

- TIME: Timeline, Political history of this place…, Faction history…
- JOURNEYS: Journey planner, Saved journeys…, Travel library…
- CONFLICTS & CAMPAIGNS: Conflicts…, Draw a conflict, Campaigns this year (readout)
- NOTES: Notes for this place, Places without notes…, Assistant panel…

**`data`** (`_fill_data`)

- The whole World popup in order: ⧉ Data manager; RECORDS; TECHNICAL; Import (drill);
  Sources & validation (drill); the signpost "Export is in File ▸ Export".
- Then a phone-only row, **Maps · tile pyramid** · "leaflet · XYZ · TMS · WMTS · retina" · badge
  `<zmax+1> levels` → `data-tiles`.

**`data-tiles`** (phone only; `_fill_data_tiles`)

- Chip rows: **Scheme** XYZ · TMS · WMTS; **Tile size** 256 · 512; **Zoom levels 0 → N** 0 – 1 … 0 – 6
  (`DT_ZMAX_CHOICES`)
- **Retina @2x** switch
- **Skip all-ocean tiles**: a dim row ("Not built")
- **Estimate** (band): Tiles; then Size and Render time after an export at these settings, or a dim row
  until then
- **Export `<n>` tiles**: writes `tile-pyramid-<scheme>-z0-<N>[@2x].zip` to the Exports root
  (`slippy_export_tiles`)
- Info: the destination and what the zip holds.

**`assets`** (`_fill_assets`)

- Nine family rows (`AssetLibraryWindow.FAMILIES`): `<Title>` · `<group> · <size> px seamless tile` or
  `RGBA icon` · badge `<filled> / <total>` → `assets-grid`. In order: Splat channels, Biome textures,
  Terrain textures, Feature icons, Settlement pins, Settlement traits, Points of interest, Sea marks,
  Custom icons.
- **Pack** (band): Name, Author, Licence, Stored items (each drawn only when set)
- Then the whole Assets popup, from ⧉ Asset library to "Clear library…   destructive", with its
  submenus as drills.
- **`assets-grid`**: a "`<Family>` · slots" band, **Filled** `<f> of <n>`, a 4-column grid of slot
  cells (tap → `asset-slot`), and **Open `<Family>` in the asset library**.
- **`asset-slot`**: readouts Slot, Family, Variants, File, Scale, Pan, Content hash, Anchor, Bakes to,
  Tags, Duplicate art; **Open in the asset library**.

**`travel`** (`_fill_travel`)

- **Type** chips: Animals & mounts · Vehicles · Vessels · Party set-ups (`TravelLibraryWindow.KINDS`)
- One row per entry from `tl_list(kind)` (user entries are badged CUSTOM) → `travel-item`
- Info line; **Open the travel library**
- **`travel-item`**: readouts Entry, Class, Constraints, Source, Validation, Used by;
  **Load into planner** (party set-ups only); **Open in the travel library**.

**`landmarks`** (phone only; `_fill_landmarks`)

- Headroom info line
- **Crowding** slider (`× n.nn`), then an info line: "a `<class>` landmark keeps N km clear"
- **Types compete with each other** switch
- **Families** (band): six family rows → `lm-fam`
- **Run landmark pass**; info line
- **`lm-fam`**: a cap slider per type (a dim row for a type that cannot be built) with a "↳ last run"
  readout; **Arm every type in this family**; **Turn every type off**; info line.

**`sim`** (phone only; `_fill_sim`)

- **Transport strip on map** switch
- **Year** slider, −400 to 1200 (a dim row before a world exists)
- **Speed** chips ×1 · ×10 · ×100
- **Playback** switch
- **Simulation layers** (band): Climate, Population, Economy, Politics, Infrastructure, Warfare switches
- Info line
- **Simulation model** → CIVIL ▸ Timeline, in the left sheet

**`prefs`** (`_fill_prefs`)

- **Application** (band): **Theme** chips Dark · Light · Follow system; **Units** chips Kilometres ·
  Miles · Nautical miles
- Then the rest of Preferences in its own order:
  - PERFORMANCE: GPU acceleration (switch); Devices (drill); Multi-GPU mode chips, plus Alternate frames
    as a dim row; CPU worker threads chips plus the running-count readout; VRAM budget chips plus the
    estimate readout; Fallback chips, plus Reduce working res as a dim row; Try the GPU again.
  - GRAPHICS: Render quality chips; Anti-aliasing · anisotropy (dim); the Colour management signpost;
    3D viewport defaults (dim); Lighting rig defaults (drill); Relief exaggeration chips 1x · 2x · 4x
    plus Use the engine's own 3.4x.
  - TILES & LOD: Tiled LOD chips plus Enter and Leave deep detail; Tile size · LOD levels (drill);
    Atlas cache (drill); Chunk debug overlay (drill).
  - MEMORY: Undo history chips plus Clear undo history now; Working set (readout); Clear caches….
  - APPLICATION: Storage locations…, Assistant…, Quick bar…, Handedness chips Left · Right,
    Keyboard shortcuts….
- **Touch** (band): **Gesture reference** → `gestures`

**`help`** (`_fill_help`)

- Readouts: **Runtime** `Godot <version> · <OS>`; **Engine** (the GPU slot)
- **Gesture reference** → `gestures`
- Then the whole Help popup: Documentation (disabled on an installed build), Keyboard shortcuts…,
  Credits & academic principles, Generation info…, Save diagnostic report…, About
- Info: "The phone reorganises rather than truncates: every desktop function is reachable through MAP ·
  GENERATE · PLAN · MORE."
- **`gestures`**: descriptive rows, not actions: Drag, Pinch, Two fingers, Press and hold, Sheet handle,
  Tab re-tap, Undo chip, Back. A **Not available** band lists Double-tap and Edge-swipe inspector.

### 3.3 Phone compared with the desktop menus

**On the phone but not a desktop menu row.** The desktop home for each is in brackets.

- MORE ▸ Quick (the quick bar cluster on the tool-palette bar)
- Civilization's six tool rows (the tool palette, rail and ring)
- Point of interest, as a disclosed absence (nothing)
- The Landmark generation screen (the CIVIL ▸ Landmarks dock)
- The asset family grid and slot inspector (the Asset library window)
- Travel library browsing (the Travel library window)
- The Simulation screen (the timeline bar)
- Maps · tile pyramid / `data-tiles` (Data manager ▸ Export ▸ Maps)
- Gesture reference (nothing)
- Status readouts and Recompute stale stages (the status bar and the menu-bar readouts)
- Help's Runtime and Engine readouts (nothing)

**On the desktop menus but omitted on the phone.** Only Preferences ▸ **Open panels in their own
window**, which is not built on the phone. Every other row of the eight menus is drawn: File, World,
Story, Preferences, Assets and Help through their screens, Edit and Window through the "Not on the MORE
list" band. Help ▸ Documentation is drawn but disabled on an installed build. The phone shows no
accelerators.

## 4. Phone chrome (`dcc_shell.gd`)

### 4.1 App bar: `[world pill] · ⌕ · ⋮` (`_build_phone_app_bar`)

- **World pill**: `CARTALITH` over the `top_world` status (`<name> · <seed>`). Not interactive.
- `⌕` **Search**: `open_find_on_map()`, the search overlay of §5. Drawn only when `place_search.gd`
  exists.
- `⋮` **More actions**: the overflow popover (`_build_phone_overflow`), three rows:
  - **Save project**, with the time of the last save or `—` → `save_project()`
  - **Theme**, with `dark` or `light` → `toggle_theme()`
  - **Close world** → `close_project()`
  - Each row is drawn disabled, with a reason, when the shell lacks the method behind it.

### 4.2 Bottom tabs (`PHONE_TABS`)

| id | caption | glyph | does | tip (also the sheet subtitle) |
|---|---|---|---|---|
| `map` | MAP | ▤ | selects CARTO | "layers · style · annotation" |
| `gen` | GENERATE | ⌗ | selects WORLD; hidden in Session mode | "The generation pipeline, and Sculpt" |
| `plan` | PLAN | ➔ | opens the Journey planner | "Journey planner" |
| `more` | MORE | ⋯ | toggles the MORE menu | "program · data · preferences" |

CIVIL is not a tab. It is reached through PLAN, MORE ▸ Civilization and MORE ▸ Story.
Window ▸ Domain rail hides MAP, GENERATE and PLAN; MORE stays.

### 4.3 The tool sheet (`_build_phone_tool_sheet`)

**Detents.** Peek is 66 dp (`PHONE_DETENT_PEEK`). Half is 46 % of the frame
(`PHONE_DETENT_HALF_FRAC`), clamped to its content on MAP. Full is the frame minus 96 dp. While a drag is
in progress the sheet's height is clamped between 40 dp (`PHONE_DETENT_MIN_DRAG`) and full. Releasing
below 44 dp collapses the sheet to peek; otherwise it snaps to the nearest detent. A tab tap lifts peek to half; re-tapping
the lit tab returns it to peek; `✕` returns it to peek.

**Header**, per tab:

- GENERATE: "pipeline · seed N", "pipeline · generating…", "pipeline · no world yet",
  "sculpt · draft stamps" or "paint · draft cells"
- PLAN: "PLAN · STAGE n" over "journey · A → B" (otherwise the tab's caption and tip)
- MAP: "MAP" over "layers · style · annotation"

**Content**, per tab:

- **MAP**
  - The quick chip row (pins, from the half detent up; the last chip is `All…`), only when pins or the
    Assistant chip exist.
  - Then CARTO's tool-options row. By default it shows `CARTOGRAPHY · <MODE>` and the note
    "Presentation only — …". It is replaced by the armed tool's options (Label, Icon, Measure, Region,
    colour paint, …).
- **GENERATE**: `WorldWorkspace.build_phone_generate()`, with a segment PIPELINE · SCULPT · PAINT.
  - PIPELINE: a **SEED** row (the value and `↻` to roll a new seed); the idle block (**GENERATE WORLD**
    or **REGENERATE NN → 10**, last run, a stale note) or, while running, a progress card with a dashed
    **CANCEL** row that explains a run cannot be interrupted; ten collapsible stage groups,
    **01 Planet** … **10 Resources & soils**, each holding that stage's parameters; a footnote ("…GPU,
    LOD and render quality live under MORE ▸ Preferences").
  - SCULPT: GEOLOGICAL FEATURE (the 13 sculpt features), PRESETS, BRUSH · GLOBAL sliders,
    BRUSH · DYNAMICS, SYMMETRY, **+ ARM SCULPT · DRAW ON THE MAP**, **DISCARD**, **✓ COMMIT**,
    LAST COMMIT.
  - PAINT: TARGET FIELD; ROCK LAYER (rock targets); BRUSH (Size, Hardness, Softness); the Original
    compare slider; ERASE and LAND ONLY chips; CAP THICKNESS (rock cap); CLASS or ROCK palette chips;
    BRUSH · DYNAMICS (Gap between dabs, Scatter copies, Scatter spread); SYMMETRY (Copies, Mirror angle,
    PICK CENTRE, RESET CENTRE); **+ ARM PAINT · DRAW ON THE MAP**; commit and discard.
  - **Not in this sheet:** the desktop WORLD tool-options buttons **Center landmasses** and
    **Bake ALL & finalize** (`app.gd::_tool_options_generate`). On the phone they exist only in the WORLD
    dock (`world_workspace.gd`), which means the left sheet.
- **PLAN**: no sheet content of its own. It opens the Journey planner overlay, which on the phone also
  opens the left sheet for its controls.
- **MORE**: the `PhoneMenu` of §3.

### 4.4 Other phone chrome

- `↶` floating undo chip: a tap undoes; a hold opens the "EDIT HISTORY · TAP TO ROLL BACK" popover, one
  row per step.
- The tool-fan pill (§6.3).
- The navpad (touch builds): **Zoom out**, **Zoom in**, **Pan mode** (toggle), **Reset view**; the phone
  adds a **3D** pill (3D ↔ 2D).
- The map's Layers button (all form factors) opens the Layers popover (layer views, keys 1–8).
- The sim strip, switched on in MORE ▸ Simulation ▸ Transport strip on map.
- The left and right dock sheets: Window ▸ Left dock and Right dock. A map pick also opens the right
  sheet.

## 5. Search, the command index and the quick bar

### 5.1 Search scopes (`dcc_shell.gd::SEARCH_SCOPES`)

The phone overlay and the desktop "Find on map" dialog share the same scopes. The field's placeholder is
"Search places, factions, routes, commands…".

| prefix | chip text | searches |
|---|---|---|
| `s` | `s settlements` | PlaceSearch rows whose entity is `settlement` |
| `f` | `f factions` | `faction` |
| `lb` | `lb labels` | `label` |
| `r` | `r routes` | `route` (roads and sea routes) |
| `.` | `. commands` | `CommandIndex` |
| `l` | `l landmarks` | **declined**: a disabled chip, tooltip "Placed landmarks carry family / slot / set / scale — four closed vocabularies, no name…" |

- The prefix is the token before the first space, matched whole, so `l` and `lb` coexist.
- `SEARCH_HINT` names only the first five.
- Result bands: places use STARTS WITH / CONTAINS / MATCHED ON KIND OR SUBTITLE; commands use
  MATCHED THE NAME / MATCHED THE MENU / MATCHED THE DESCRIPTION.
- **Running a command from search** (`_run_search_command`):
  - On the phone a command result runs only if it is pinnable; otherwise tapping it toasts the reason.
  - On the desktop, command results are drawn disabled (display only).
  - Pinnable phone results carry a `☆`/`★` pin toggle.

### 5.2 What the command index holds (`command_index.gd::build`)

1. Every generation parameter (`param_keys()`), as kind `param`. Never pinnable.
2. Every leaf row of every `MenuButton` on the live bar. Signposts and silent disabled rows are skipped;
   readouts are kept as kind `readout`.
3. `EXTRAS`: **Zoom to fit**, **Point sample**, **Theme**. These are search answers only and cannot be
   pinned.

### 5.3 What can be pinned (`_why_not_pinnable`)

A menu row can be pinned only if **all** of these hold:

- it is not a readout;
- it is enabled in the state the menu was last **built** in (rows disabled at that moment cannot be
  pinned then);
- it is not a check or radio row;
- its id is not in a generated span: autosave interval 131–139, data routes 400–413 (but see §8),
  landmark dock rows 510–573, layouts 640–671, reset stages 610–619;
- its popup is not an "index popup", meaning any popup with an item whose id is 0. That covers Recent
  worlds, Icon families, Texture sets, every landmark family, Render quality, CPU threads, VRAM,
  Multi-GPU, Fallback, Undo history, Workspace and Open windows. It also covers, from reading the code,
  every popup whose first row is a submenu row or an id-less `_todo`/`_readout`, because Godot assigns
  such a row its index as its id: Landmark types, Lighting rig defaults, Atlas cache, 3D capability, and
  **Help on an installed build** (see §8);
- its popup's open handler does not evict or enumerate (Atlas cache, Devices);
- it is not inside an embedded ColorPicker popup.

The same id with the same title in two menus (Places without notes…) is pinned once.

### 5.4 Pin targets and caps (`DccSettings` `[quick_bar]`)

- **Bar**: up to 24 (`QUICK_BAR_MAX_BAR`)
- **Ring dock**: up to 4 (`QUICK_BAR_MAX_DOCK`)
- **Shared ring slots**: built-in children plus pins never exceed 9 per diagonal that has children
  (`QUICK_BAR_MAX_SHARE`)

### 5.5 Where pins are made and shown

- **Desktop and tablet:** the quick cluster at the right of the tool-palette bar (a `＋` chip, a `⋯`
  overflow chip, and a right-click menu per chip: remove, move, Customise…); the ring dock under the
  cartwheel; pins appended to a ring diagonal's sub-ring.
- **Any menu row:** right-click it to get "Pin to quick bar" or "Remove from quick bar"
  (`QuickBarMenuPin`).
- **Customise window** (Preferences ▸ Quick bar…): tabs **Bar · Ring dock · Ring slots**. On the phone
  only Bar, with stacked **Commands** and **Pinned** columns.
- **Phone:** the MAP sheet's chip row, with `All…` opening a list sheet; MORE ▸ Quick; the `☆` in search.
- **Assistant chip** (all forms): a built-in conditional item, not a pin.

## 6. Map context card, ring and phone tool fan

### 6.1 Context card (`context_broker.gd`, `context_card.gd`)

**Triggers**

- Desktop: a right-click; the Menu key or Shift+F10 at the mouse.
- Tablet: a 500 ms touch-hold opens the ring and the card together. The card docks away from the
  Preferences ▸ Handedness side.
- Phone: a 480 ms touch-hold opens the peek sheet (`PhoneMenu.peek_card`).

**Sections**, in order: draft · tool · object · place · go · info. The providers are GlobalTools plus the
active domain's workspace.

**Phone differences**

- Rows that carry `children` or `param` are **dropped**.
- The peek detent shows up to 4 chips (3 plus "Select ▸ N" when several objects are hit). Rows with a
  `chip` rank go first: Open note or Attach note, then City viewer.
- The half detent shows the full list.

**Global** (any domain, needs a world)

- go: **Centre view here**; **Measure from here**
- info: **Copy coordinate** ▸ `<x km> E · <y km> S  (map)` and `<lat>° N|S  (latitude, world CRS)`
  (card only; dropped on the phone)

**WORLD** (needs a world, not while generating)

- draft, sculpt: **Commit N stamp(s)**, **Undo last stamp**, **Discard sculpt draft**
- draft, paint: **Commit N paint dab(s)**, **Discard paint draft**
- object, a stamp: **Select `<stamp>`**, **Hide `<stamp>`** or **Show `<stamp>`**, **Move `<stamp>` up**,
  **Move `<stamp>` down**, **Delete `<stamp>`**
- object, a river: **Inspect river**, **Trace downstream**, **Show catchment**; **Clear river highlight**
  while one is shown
- tool, sculpt armed: brush size (param; card only)
- tool, paint armed: **Radius** and **Erase** (params; card only); **Paint with this biome (`<biome>`)**
  (an eyedropper)
- go: **Cross-section from here**

**CIVIL** (needs a world)

- object, a settlement: **Edit `<name>`**, **Move viewer to `<name>`**, **Open in city viewer…**,
  **Delete `<name>`**
- info, a settlement: **Open note for `<name>`**, or **Attach note to `<name>`…**
- object, a landmark: **Inspect `<Kind>`**, **Why here?**
- object, a journey route: **Open "`<name>`" in Journey Planner**, **Delete "`<name>`"**
- object, a way: **Inspect `<way>`**, **Delete `<way>`** (generated ways add "— generated; Generate roads
  brings it back")
- object, a controlling faction: **Open `<faction>`…**, **Claim for `<faction>`**
- place: **Drop settlement here**, **Start way here**, **Start route here**
- info: **Info here (settlement & ecology)**
- draft, territory armed: **Commit territory**, **Discard territory draft**
- tool, settlement armed: **Class** (param; card only)

**CARTO**

- object, a settlement: **Settlement actions in CIVIL ›** (this row appears even with no world)
- object, a label: **Edit text of "`<label>`"…**, **Delete label "`<label>`"**
- object, an icon: **Delete icon "`<icon>`"**
- place: **Label this: "`<name>`"** (on a settlement or landmark); **Add label here…**;
  **Stamp armed icon here** (disabled with a reason when no pack is loaded or no icon is armed)
- place: **View field** ▸ up to 8 available debug layers; **Style preset** ▸ all 18 `STYLE_PRESETS`
  (card only; dropped on the phone)
- go: **Start export region here**

### 6.2 Radial ring ("cartwheel"; desktop and tablet; `radial_ring.gd` over `ContextBroker.ring_collect`)

**Triggers.** Desktop: a right-button drag or hold; the Q key (hold, aim, release, or a tap for a sticky
ring). Tablet: the 500 ms touch-hold.

**Cardinal slots** (global, `GlobalTools.ring_cardinals`)

- N **Inspect** (V)
- E **Measure** (M) ▸ Distance, Bearing, Area, Radius, Cross-section, Δ vertical (`MEASURE_MODES`)
- W **Region** (R)
- S **Undo** (Ctrl+Z), relabelled `Undo <operation>`; disabled when there is nothing to undo or the
  world is finalized

**Diagonal slots** (per domain, from each workspace's `ring_slots`). Each is disabled "no world loaded"
before a world exists, and most are disabled "world finalized" afterwards.

| | NW | NE | SE | SW |
|---|---|---|---|---|
| WORLD | **Uplift** ▸ Mountains, Hills, Ridge, Plateau, Cliff / Escarpment, Volcano | **Carve** ▸ Canyon, Valley, River, Lake, Basin, Coastline | **Freehand** (F) ▸ Raise, Lower, Smooth, Cliff, Ridge, Canyon, Mesa, Volcano | **Paint** (B) ▸ Biome, Terrain, Splat |
| CIVIL | **Settle** (S) ▸ Metropolis, City, Town, Village, Hamlet | **Territory** (T) ▸ Add, Subtract | **Way** (W) ▸ Road, Track, Sea lane, Ancient | **Route** (⇧R), a single tool |
| CARTO | **Label** (L), a single tool | **Icon** (I) ▸ Settlement, Feature, POI, Custom (disabled until custom icons exist) | **View** (1-8) ▸ up to 8 available debug layers | **Style** ▸ Natural Vibrant, Default, Antique, Ink, Watercolor, Print, All styles... |

**User pins**

- Appended to a diagonal that has children, capped at 9 in total.
- The ring **dock** draws up to 4 pins beneath the ring (desktop and tablet only), plus the Assistant
  item when it is enabled.

### 6.3 Phone tool fan (`tool_fan.gd`)

- An always-visible pill above the tool sheet that reads the armed tool. It hides while the peek sheet
  or MORE is up.
- **Tap**: toggles Inspect and the previously armed tool.
- **Press and slide up** (260 ms): a half-fan of the **same 8 slots** as §6.2. W, NW, N, NE and E sit at
  the outer radius (150); SW, S and SE sit at the inner radius (84).
- It includes shared pins, because it calls `ring_collect`. It has **no ring dock**.

## 7. Keys that are not menu rows

`app.gd::_unhandled_key_input` (each is ignored while a text field has focus):

- **Esc**: closes an open ring; otherwise cancels the armed tool back to Inspect.
- **Q**: pressing opens the ring at the mouse; releasing is the ring's decision.
- **Backspace**: the armed tool's backspace handler (Measure drops its last point).
- **Ctrl+Y**: a second Redo key.
- **Delete**: deletes the selection. The same key is Edit ▸ Delete's accelerator.
- **Menu key** or **Shift+F10**: opens the context card at the mouse.
- **Shift+J**: opens the Journey planner.

Tool buttons (`DccWidgets._tool_shortcut` reads the `(X)` in a tool label; these are Godot `Shortcut`s on
the buttons):

- Global palette: **V** Inspect, **M** Measure, **R** Region select
- CIVIL: **S** Settlement, **T** Territory, **Shift+T** Territory lasso, **W** Way, **Shift+R** Route.
  Conflict has no key.
- CARTO: **I** Icon, **L** Label
- WORLD: **B** Biome paint; **F** Freehand sculpt (`world_workspace.gd`)

Elsewhere:

- **1–8** switch the layer view while the Layers popover is open (`layers_popover.gd`,
  `HOTKEY_ACTIONS`).
- **Space**, held, pans the map (`viewport_host.gd`).
- In the Customise window: **Enter** adds, **Delete** removes, **Alt+Up/Down** moves.

`shortcuts_dialog.gd::UNLISTED` documents only 1–8, Space, Esc, Delete/Backspace, Shift+J and Ctrl+Y.
It does **not** list Q, the Menu key, Shift+F10, or the tool keys (§8).

## 8. Findings: where code and comments disagree, and surprises

Each item is from reading code, with the symbol named. Nothing here was run.

1. **The bar order is confirmed:** `File, Edit, Assets, World, Story, Preferences, Window, Help` on PC and
   on the phone (hidden there); `☰, File, World, Story` on the tablet. But `menus.gd`'s own file header
   still reads "The seven program menus … File · Edit · Assets · Data · Preferences · Window · Help --
   and nothing else". That is stale twice over: SM-1 renamed Data to World, and SM-2 added Story, making
   eight.
2. **The asset family count is stale in comments.** `menus.gd` (the `Icon families` block in `_assets`)
   says "`cartalith-assets` ships EIGHT families", and `phone_menu.gd`'s `SCREEN_TITLES` and
   `_fill_assets` comments say eight. `AssetLibraryWindow.FAMILIES` has **nine** entries, and its own
   `FAMILIES_NOTE` says "Nine families…".
3. **One data-route row can be pinned and its siblings cannot.** `command_index.gd::_is_dynamic_id`
   treats ids `ID_DATA_ROUTE_FIRST + 0..13` (400–413) as generated. `DataManagerWindow.ROUTES` has
   **15** rows (400–414). So **Repair / Normalize** (id 414) is the one route row that passes as
   pinnable. The `menus.gd` comments disagree with each other: the 600-block note says the run is
   "fourteen long (400-413)", while `_add_route_rows` says "400-414".
4. **A reason string names a menu that does not hold the row.** CARTO's context-card row "Stamp armed icon
   here" gives the reason "no asset pack loaded — File ▸ Import asset pack". That row is
   **Assets ▸ Import asset pack .zip…**; File has no import row.
5. **The accelerator count in a comment is low.** `shortcuts_dialog.gd` says "Eighteen actions route
   through `DccMenus._bind_accelerator`". `menus.gd` binds **22** on PC and phone (19 `_live` calls with a
   key, plus 3 direct `_bind_accelerator` calls), and **23** with the tablet's Run pipeline (§9).
6. **Some non-menu keys are undocumented.** The Shortcuts dialog's `UNLISTED` omits Q (the ring), the
   Menu key and Shift+F10 (the context card), and the tool-button keys, all of which `app.gd` and the
   tool buttons handle.
7. **Search can run only pinnable commands.** On the phone a `.` command result runs only when it is
   pinnable; check and radio rows, generated rows and rows in index popups only toast a reason. On the
   desktop, command results never run.
8. **Help may be unpinnable on an installed build** (read, not run). On such a build Help ▸ Documentation
   is a `_todo` at index 0. `_todo` calls `add_item(text)` with no id, and Godot then uses the index, so
   the id is 0. `_is_index_popup` treats any popup holding an id-0 item as generated. If that reading is
   right, no Help row (Keyboard shortcuts…, Credits, Generation info…, Save diagnostic report…, About)
   can be pinned on a phone build. The same rule makes **Re-test 3D**, **Clear undo history now**,
   **Use the engine's own rig**, and **Placed by hand / Landmark icons… / Landmark label style…**
   unpinnable.
9. **On the phone, CARTO and most WORLD dock categories have only one door: MORE ▸ "Not on the MORE
   list" ▸ Window ▸ Left dock.** I searched every caller of `_set_sheet_open("left", true)` and
   `_open_left_sheet()` across `godot-project`, probes included (§9). The only shipping callers are
   Window ▸ Left dock (`app.gd::toggle_region`), MORE ▸ Civilization ▸ Civilization dock,
   MORE ▸ Simulation ▸ Simulation model, and the Journey planner. A related reading: menu rows that call
   only `select_domain_category()` (World ▸ Places…, both Timeline rows, the Landmark types leaves,
   Landmark label style…) change the domain and category but **do not open the phone's left sheet**,
   because `select_domain_category` and `_select_domain` have no sheet call. Whether the user sees
   anything was not run.
10. **The phone GENERATE sheet lacks two desktop WORLD tool-options buttons:** Center landmasses and
    Bake ALL & finalize. They are reachable on the phone only in the WORLD dock, behind item 9's single
    door.
11. **The phone context surfaces drop some rows.** The peek sheet drops every row that has `children` or
    `param`: Copy coordinate, View field, Style preset, the settlement Class, and the brush size, Radius
    and Erase rows. The phone tool fan has no ring dock, so dock pins never appear on the phone.
12. **Preferences ▸ Open panels in their own window is desktop-only.** Its comment says "the tablet's
    disabled-with-reason row is PW-5's"; that row is not built, so the tablet and the phone show
    nothing.
13. **The phone Preferences screen has two "Application" bands:** the screen's own band, which holds
    Theme and Units at the top, and the popup's APPLICATION band lower down.
14. **MORE ▸ Story ▸ Notes' subtitle is missing a row.** It reads "this place · places without notes",
    but the NOTES band also holds **Assistant panel…** (LLM-3).
15. **A detent comment contradicts the code.** `PHONE_DETENT_DISMISS`'s comment says a release under
    44 dp "closes the sheet"; the drag-release code calls `_set_phone_detent("peek")`, and the sheet has
    no closed state.
16. **Import image… is not an import.** It opens the same Asset library window as ⧉ Asset library and
    has no picker of its own.
17. **Render quality tiers display in lower case** (`performance`, `balanced`, `quality`, `ultra`, the
    engine strings), while the parent row's tooltip spells them in title case.
18. **The tablet's labels differ from the canvas.** `menus.gd` comments cite the canvas's "Shortcuts…" and
    "About Cartalith" for ☰; the code draws **Keyboard shortcuts…** and **About**.
19. **Data-manager routes that do nothing look enabled in the menu.** Six of the eleven World ▸ Import
    and Sources & validation rows are `kind: "gap"` in `DataManagerWindow.ROUTES`: Import ▸ Maps, all
    three Sources rows, Check Data, and Repair / Normalize. They are enabled `add_item` rows that open
    a Data-manager pane explaining the gap; the menu row itself never says so. Import ▸ Assets is
    `kind: "route"` and sends you to the Assets menu.

## 9. Counts, and the commands behind them

### 9.1 Rendered rows per menu, counted from this file

Method: a row is any tagged bullet except `band`, `sep` and `pattern`, at any depth, with `×N` counted
as N. Runtime-sized lists are counted separately as `pattern` bullets (one per list). The counter is
the inline script below, and nothing was written by it.

```
$ python3 -I - MENU_INVENTORY.md <<'EOF'
import re, sys, collections
lines = open(sys.argv[1], encoding="utf-8").read().split("\n")
KINDS = ["live","todo","readout","signpost","item","check","radio","submenu"]
sec = None; data = collections.OrderedDict()
tag = re.compile(r'^(\s*)- `(\w+)(?:×(\d+))?`')
for ln in lines:
    h = re.match(r'^(###|####) (.+)$', ln)
    if h:
        sec = h.group(2).strip(); data.setdefault(sec, {"top":0,"total":0,"pattern":0,"kinds":collections.Counter()}); continue
    if ln.startswith("## "): sec = None; continue
    if sec is None: continue
    m = tag.match(ln)
    if not m: continue
    indent, kind, n = len(m.group(1)), m.group(2), int(m.group(3) or 1)
    d = data[sec]
    if kind == "pattern": d["pattern"] += 1; continue
    if kind not in KINDS: continue
    d["total"] += n; d["kinds"][kind] += n
    if indent == 0: d["top"] += n
# ...prints one line per section (1.1-1.8, ☰, tablet World) and a total for 1.1-1.8
EOF
section                             top  total patterns  by kind, all depths
1.1 File                             19     32        2  live 12, todo 2, readout 4, signpost 2, item 4, check 1, radio 4, submenu 3
1.2 Edit                             12     22        0  live 11, item 10, submenu 1
1.3 Assets                           14    101        0  live 10, readout 5, signpost 6, item 70, submenu 10
1.4 World                            11     22        0  live 8, signpost 1, item 11, submenu 2
1.5 Story                            12     12        0  live 11, readout 1
1.6 Preferences                      28    138        3  live 6, todo 2, readout 6, signpost 1, item 11, check 7, radio 83, submenu 22
1.7 Window                           13     27        3  live 2, readout 1, item 3, check 8, radio 8, submenu 5
1.8 Help                              6      6        0  live 6
☰                                    14     17        0  live 7, radio 3, submenu 7
World                                 6      6        0  live 3, radio 2, submenu 1
All eight PC/phone menus: top 115, total 360, patterns 8; live 66, todo 4, readout 17, signpost 10, item 109, check 16, radio 95, submenu 43
```

How to read it:

- "top" counts the rows directly in the menu; "total" adds every nested row of fixed size.
- The phone has one row fewer in Preferences (27 at the top level), because "Open panels in their own
  window" is not built there.
- The ☰ and tablet World lines count their own rows only. Their nested Edit, Assets, Preferences,
  Window, Help, Landmark types and World contents are the §1 menus with the promoted rows removed (§2.1).
- The 8 patterns, which add rows at run time, are:
  - File: Recent worlds, and its empty readout
  - Preferences: GPU adapters, GPU memory readouts, CPU-thread rungs
  - Window: open windows, its empty readout, saved layouts
- **Always-`todo` rows: 4.**
  - File ▸ Export ▸ Print sheet (PDF)…
  - File ▸ Export ▸ VTT battle map (grid + walls)…
  - Preferences ▸ Anti-aliasing · anisotropy
  - Preferences ▸ 3D viewport defaults
- **Rows that become `todo` under conditions:**
  - Edit ▸ Redo
  - Preferences ▸ Devices, Multi-GPU mode, CPU worker threads, VRAM budget, Fallback when VRAM full,
    Try the GPU again
  - Atlas cache ▸ Size cap · GB, Refine viewshed for the current view, Export atlas…, Import atlas…
  - Assets ▸ Landmark types ("No landmark types")
  - Window ▸ Layouts ▸ Forget layout… (when none are saved)
  - Help ▸ Documentation, on any installed build, which includes every phone and tablet build
- **Permanently disabled radio rows** (not `todo`): Multi-GPU mode ▸ Alternate frames, and
  Fallback ▸ Reduce working res.
- **Accelerators: 22 on PC and phone, 23 on the tablet** (it adds Run pipeline, Ctrl+R). See §9.3.

### 9.2 Row-adding call sites per builder in `menus.gd` (a static cross-check)

This counts call sites, not rendered rows: a call inside a loop counts once. It confirms that every
builder named in §0 was read, and that each helper call has a row in §1. Comment lines are excluded.

```
$ cd cartalith-native/godot-project/shell && python3 -I - menus.gd <<'EOF'
# splits menus.gd at each top-level `func`, drops `#`/`##` lines, and counts lines matching
# ^\s*_live\(  ^\s*_todo\(  _readout\(  ^\s*_signpost\(  \.add_item\(  \.add_check_item\(
# \.add_radio_check_item\(  \.add_submenu_item\(  \.add_separator\(
EOF
function                       _live  _todo  _readout  _signpost  add_item  add_check_item  add_radio  add_submenu_item  add_separator
_file                              9      0         1          1         0               1          2                 2              3
_refresh_recent_worlds             0      0         1          0         2               0          0                 0              0
_build_undo_redo                   3      1         0          0         0               0          0                 0              0
_build_edit_actions                7      0         0          0         0               0          0                 0              2
_build_reset_generation            1      0         0          0         0               0          0                 0              0
_build_reset_stage_menu            0      0         0          0         1               0          0                 1              0
_build_asset_library_row           1      0         0          0         0               0          0                 0              0
_assets                            9      0         0          0         2               0          0                 2              2
_build_landmark_types_menu         0      1         0          0         3               0          0                 1              1
_build_landmark_family             0      0         0          1         2               0          0                 1              1
_build_asset_pack_submenu          0      0         1          0         3               0          0                 1              1
_data                              6      0         0          1         0               0          0                 0              2
_route_submenu                     0      0         0          0         0               0          0                 1              1
_add_route_rows                    0      0         0          0         1               0          0                 0              0
_build_file_export                 3      2         0          0         0               0          0                 1              3
_story                            11      0         1          0         0               0          0                 0              4
_build_vault_rows                  2      0         0          0         0               0          0                 0              0
_build_theme_submenu               0      0         0          0         0               0          3                 1              0
_build_units_submenu               0      0         0          0         0               0          3                 1              0
_build_handedness_submenu          0      0         0          0         0               0          2                 1              0
_preferences                       5      7         1          1         1               2          2                 2              6
_build_gpu_retry_row               1      1         0          0         0               0          0                 0              0
_build_relief_exag_menu            0      0         0          0         1               0          1                 1              1
_refresh_gpu_devices_menu          0      0         3          0         1               2          0                 0              3
_build_gpu_mode_menu               0      0         0          0         0               0          3                 1              0
_build_cpu_threads_menu            0      0         0          0         0               0          1                 1              0
_build_gpu_vram_menu               0      0         0          0         0               0          1                 1              0
_build_gpu_fallback_menu           0      0         0          0         0               0          3                 1              0
_build_lighting_menu               0      0         1          0         1               0          0                 1              1
_light_ladder                      0      0         0          0         0               0          1                 1              0
_build_tiled_lod_menu              0      0         0          0         2               0          2                 1              1
_build_atlas_tiles_menu            0      0         0          0         0               0          2                 2              1
_build_atlas_cache_menu            0      3         2          0         5               0          0                 1              2
_build_atlas_cap_menu              0      1         0          0         0               0          1                 1              0
_build_view3d_rows                 1      0         1          0         0               1          2                 2              0
_window                            1      0         0          0         0               3          1                 2              3
_refresh_layouts_menu              0      1         0          0         4               0          0                 0              1
_refresh_open_windows              0      0         1          0         0               1          0                 0              0
_build_help_shortcuts_row          1      0         0          0         0               0          0                 0              0
_help                              5      1         0          0         0               0          0                 0              0
_build_lod_debug_submenu           0      0         0          0         0               4          0                 1              1
_overflow                          2      0         0          0         0               0          0                 0              4
_world                             2      0         0          0         0               0          2                 0              3
```

`_preferences`' seven `_todo` call sites are the two permanent ones (Anti-aliasing · anisotropy,
3D viewport defaults) and the five engine-version fallbacks (Devices, Multi-GPU mode, CPU worker
threads, VRAM budget, Fallback when VRAM full).

### 9.3 Other commands run for this file

Run from `cartalith-native/godot-project/shell` unless another directory is shown. Line numbers are as
of this read.

```
$ grep -nE '^\s*_live\(.*KEY_' menus.gd | wc -l
20
   # these are the 19 PC/phone accelerators plus Run pipeline (tablet), listed by:
$ grep -noE '_live\([^"]*"[^"]+", ID_[A-Z_0-9]+, KEY_[A-Z_| ]+\)' menus.gd
   # New world…, Open project…, Save project, Save as…, Close project, Undo, Redo, Cut, Copy, Paste,
   # Delete, Select all, Deselect, Find on map…, ⧉ Asset library, Player map…, Timeline (Story),
   # Travel library…, Notes for this place, Run pipeline; plus one match at :7422 inside a ## comment
$ grep -nE '^\s*_bind_accelerator\(' menus.gd
736:		_bind_accelerator(p, p.item_count - 1, id, int(accel))        # _live's own call
2766:	_bind_accelerator(ap, …, ID_AP_EXPORT, KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_P)
6095:	_bind_accelerator(p, sess_idx, ID_WIN_SESSION_MODE, KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_M)
6111:	_bind_accelerator(p, diag_idx, ID_WIN_DIAG_OVERLAY, KEY_MASK_SHIFT | KEY_D)
   # => 19 + 3 = 22 accelerators on PC/phone, 23 on the tablet

$ cd .. && grep -rn "\.add_menu(\|^\s*add_menu(" --include=*.gd shell | grep -v "^\S*:\s*##"
shell/menus.gd:638-650   (the 4 tablet calls and the 8 PC/phone calls; no other file)
$ grep -rn "get_popup()\.add_\|menu_popup.*add_item\|_menu_popup(.*)\.add" --include=*.gd shell
(no output)

$ cd /home/user/Cartalith_GDT/cartalith-native/crates/cartalith-civ
$ awk 'NR>=322 && NR<=520' src/landmark.rs | grep -c 'LandmarkKindSpec { key:'
49
$ awk 'NR>=322 && NR<=520' src/landmark.rs | grep -o 'family: F::[A-Za-z]*' | sort | uniq -c
      8 family: F::Cultural
      6 family: F::Economic
      6 family: F::Historical
      6 family: F::Military
     15 family: F::Physical
      8 family: F::Transport

$ cd /home/user/Cartalith_GDT/cartalith-native/godot-project/shell
$ sed -n 146,190p asset_library_window.gd | grep -o '"key": "[a-z_]*"\|"title": "[^"]*"\|"texture": [a-z]*' | paste - - -
   # 9 rows: textures/Splat channels/true, biomes/Biome textures/true, terrains/Terrain textures/true,
   # icons/Feature icons/false, settlement/Settlement pins/false, trait/Settlement traits/false,
   # poi/Points of interest/false, seamarks/Sea marks/false, custom/Custom icons/false
$ cd workspaces && awk 'NR>=119 && NR<=200 && /\{"name":/' world_workspace.gd | wc -l     # STAGES
10
$ sed -n 371,411p data_manager_window.gd | cut -c1-260
   # ROUTES: Import 5 (maps gap, heightmap live, gis live, world live, assets route),
   # Export 4 (maps, gis, world live; assets route), Sources 3 (all gap),
   # Validation 3 (defs live, check gap, repair gap) = 15 rows, ids 400-414

$ cd /home/user/Cartalith_GDT/cartalith-native/godot-project && grep -rn \
  "_set_sheet_open(\"left\", true)\|_set_sheet_open(\"left\", not\|_open_left_sheet()\|toggle_region(DccMenus.ID_WIN_LEFT)\|toggle_region(ID_WIN_LEFT)" \
  --include=*.gd . | grep -v "^\S*:\s*##" | grep -v "/\.godot/"
   # shipping callers: shell/app.gd:5764 (toggle_region, Window ▸ Left dock),
   # shell/phone_menu.gd:3489 (_go_civilization) and :3501 (_go_simulation) via :3508 _open_left_sheet,
   # shell/journey_planner_view.gd:495 (inside open()); every other hit is a _*_probe.gd file

$ which godot godot4; ls /home/user/Cartalith_GDT/cartalith-native/target
   # (no Godot binary; target/ absent: the live menu tree could not be walked)
```
