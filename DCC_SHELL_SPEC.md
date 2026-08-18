# DCC shell — implementation spec

> **Imported from the owner's Claude Design project "UI mockups planning",
> sync 2026-08-19T00:20Z** (supersedes the 23:05Z import). This revision adds
> **§4.5 Tool palette** and the twelve tool glyphs in §12, and it closes the gap
> `STRANDED_TOOLS.md` opened: every tool that had a built engine and no surface
> now has one.
>
> **The UI hold is lifted** — owner, 2026-08-18: *"Replace the current GUI and
> replace it in full by the DCC version including all it's wiring and
> functionality."*
>
> Two conflicts with the engine are recorded rather than silently resolved:
>
> 1. §5.2's commit prose says it "re-runs erosion, hydrology and climate once".
>    `commit_sculpt_pass` deliberately marks tiles stale instead. The engine is
>    right and this line is stale — see `SCULPT_FUNCTION_CHART.md` §7.
> 2. §5.1's "Run stage *n*" and "Run *n* → 10" have no engine entry point:
>    `generate_terrain` is one-shot, with no per-stage recompute. Those buttons
>    ship disabled with a tooltip saying so.
>
> §5.2's global **defaults** also differ from `SculptGlobals::default()` on five
> of eight. Settled 2026-08-19 (owner: the values are placeholders, pick one):
> the engine's values win, because `golden_parity_sculpt_water.rs` spreads
> `..SculptGlobals::default()`, making them golden-parity inputs rather than
> preferences. A test in `sculpt_bridge.rs` now pins the UI table to them.
>
> **§12's text-symbol premise is partly false, found by building it.** §12 says
> the text symbols "stay text… since they are typographic, inherit type metrics,
> and need no drawing". IBM Plex Mono — the face §11 specifies for exactly these
> contexts — is missing seven of them, checked against the font's own cmap
> rather than assumed: **✕ ● ○ ▾ ▸ ▶ ＋**. Present: ✓ → § ‹ › ↶ ↷ · • ×. The
> shell falls those seven back to a system face, so they render but lose Plex's
> metrics; the state dots ● / ○ that §5.1 leans on are among them. Drawing them
> as glyphs is the alternative and is a question for the design.
>
> Path note: the design team writes to a `docs/`-rooted convention. In this
> repository `docs/` holds the **source project's** own documentation, and two
> filenames collide (`UNIFIED_TOOL_PLAN.md`, `ROADMAP.md`) — `docs/README.md`
> records which is which. References below to `GENERATOR_PARAMETERS.md` are the
> source project's; this port's equivalent is `GENERATION_PARAMETERS.md` at the
> root. `terrain-appearance-rendering.md` is on file here as
> `TERRAIN_APPEARANCE_RESEARCH.md`.

Complete control-by-control specification of the Cartalith editor shell. Every
region, every button, its behaviour, its state rules, and the v2.10 element it
replaces. Nothing here is decorative: if a control is listed, it ships.

Companion documents: `UI_SHELL_DESIGN.md` (why the regions are arranged this
way), `UNIFIED_TOOL_PLAN.md` (what a tool is), `GENERATOR_PARAMETERS.md`
(parameter ranges and defaults), `terrain-appearance-rendering.md` (the render
pipeline the Cartography workspace drives).

Reference mockup: `Cartalith DCC Shell.dc.html` in the Omelette project
*UI mockups planning*, 9 screens plus one rule card:

| Screen (`data-screen-label`) | Shows |
|---|---|
| `DCC shell 1920` | Default state, Terrain workspace, Raise/Lower tool active, Assets menu + Asset pack submenu open, layers popover open |
| `DCC shell 1920 light` | Same screen, light theme |
| `DCC Generate World 1920` | Left dock in Generation Pipeline mode, Preferences menu open |
| `DCC Generate Sculpt 1920` | Left dock in Sculpt mode, stamp stack in right dock |
| `DCC Cartography style 1920` | Layer list → layer properties → colour ramp popover → stop editor, File menu open |
| `Asset library window 1920` | Asset library window with sprite-sheet slicer modal |
| `Data manager window 1920` | Data manager window, Export ▸ Maps ▸ Leaflet tile pyramid route |
| `DCC shell tablet 2560` | Tablet parity, Data menu open |
| `DCC shell android phone` | Phone layout with cutout insets |
| `Phone inset rules` | The cutout/inset rule card that travels with the phone screen |

---

## 1 · Frame geometry

Six regions, in DOM order. All heights are fixed; widths of docks are
user-draggable within the stated min/max.

| Region | Desktop | Tablet 2560 | Phone 393 |
|---|---|---|---|
| Menu bar | 34 px | 52 px | — (app bar 52 px) |
| Tool options bar | 34 px | 52 px | bottom sheet |
| Domain rail | 40 px collapsed / 200 px expanded | 48 / 240 | 44 px collapsed only |
| Left dock | 372 px (min 300, max 520) | 400 px | full-screen sheet |
| Viewport | fills | fills | fills |
| Right dock | 284–340 px (min 260, max 460) | 400 px | full-screen sheet |
| Timeline bar | 70 px | 88 px | 52 px |
| Status bar | 26 px | 36 px | 22 px |

Rules:

- The viewport never scrolls; docks scroll independently (`overflow-y:auto`).
- Docks collapse to their rail width by clicking the `‹` / `›` chevron in their
  header. A collapsed dock keeps its primary readout visible (see §6).
- Only one modal may be open at a time. Modals are children of their window, not
  of the document.
- Menus open on click, close on outside-click or `Esc`. A menu overlays the tool
  options bar; it never pushes layout.

---

## 2 · Menu bar — program scope only

Seven menus: **File · Edit · Assets · Data · Preferences · Window · Help**.

The menu bar holds *program* functions. World generation, simulation, rendering
and map styling are workspaces reached through the domain rail (§3), never menu
items. `⧉` on an item means it opens a dedicated window; `▸` a submenu; `…` a
modal dialog.

### 2.1 File

| Item | Shortcut | Behaviour |
|---|---|---|
| New world… | ⌘N | Modal: name, seed, extent (region/world), working resolution. Creates an empty project, discards nothing until confirmed. |
| Open project… | ⌘O | File picker, `.zip` project archives. Replaces `#loadZipBtn`. |
| Recent worlds | ▸ | Submenu, last 10 projects, path shown as secondary text. |
| Save project | ⌘S | Writes the project archive in place. Disabled when no changes. |
| Save as… | ⌘⇧S | File picker; the new path becomes the project path. |
| Autosave | — | Toggle + interval submenu (off, 1, 5, 15 min). Default 5 min. Status bar reports last autosave. |
| Revert to last save | — | Confirmation dialog; discards all in-memory changes including sculpt drafts. |
| Close project | ⌘W | Confirmation if unsaved. |
| Storage locations | — | Read-only list of the four roots: projects `~/Cartalith/Worlds`, tile atlas `~/Cartalith/Cache/atlas`, asset packs `~/Cartalith/Packs`, exports `~/Cartalith/Exports`. |
| Change locations… | — | Modal, one folder picker per root. Moving the atlas root invalidates the cache. |
| Show project on disk | — | Reveals the project folder in the OS file manager. |

Imports are **not** in File. The menu carries a static note: *imports live under
Data ▸ Import, asset packs under Assets*.

### 2.2 Edit

| Item | Shortcut | Behaviour |
|---|---|---|
| Undo | ⌘Z | Global undo. Depth from Preferences ▸ Memory ▸ Undo history (default 5). Replaces `#undoBtn` / `#undoMem`. |
| Redo | ⌘⇧Z | — |
| Undo history… | — | Panel listing the stack; clicking an entry rolls back to it. |
| Cut / Copy / Paste | ⌘X ⌘C ⌘V | Operate on the current selection (labels, icons, places, stamps). |
| Delete | ⌫ | Deletes the selection; never deletes a generation stage. |
| Select all / Deselect | ⌘A ⌘D | Scoped to the active layer. |
| Find on map… | ⌘F | Search places, labels, factions, routes; result pans the viewport. |

### 2.3 Assets

| Item | Behaviour |
|---|---|
| ⧉ Asset library (⇧A) | Opens the Asset library window (§8). |
| ⧉ Sprite sheet slicer (▦) | Opens the library window with the slicer modal already open. |
| Import image… | File picker, `image/*`. Lands in *Unassigned imports*. Replaces `#alFilePicker`. |
| Import asset pack .zip… | Loads a pack into the library for editing. Replaces `#alImportPackBtn`. |
| Asset pack ▸ | Submenu, §2.3.1. |
| Icon families ▸ | Submenu listing the 24 families with filled/capacity counts; picking one opens the library scoped to it. |
| Texture sets ▸ | Same for texture families. |
| Apply library to map | Compiles the library and loads it as the live pack. Replaces `#alApplyBtn`. |
| Clear library… | Destructive, confirmation required. Replaces `#alClearBtn`. |

#### 2.3.1 Asset pack submenu

Groups, in order:

- **Active pack** — read-only block: name, author, license, schema (`2 · STORED
  zip`), filled slots (`148 of 212 · 26 MB`). Sources: `#alPackName`,
  `#alPackAuthor`, `#alPackLicense`, `#alStats`.
- **Pack metadata…** — modal editing name / author / license.
- **Edit** — Open library workspace; Import image into slot…; Sprite sheet
  slicer… (cols · rows · margin); Add variant to slot (`#alAddVar`); Replace ·
  delete slot art (`#alReplace`, `#alDelVar`); Slot transform (scale · fit ·
  reset — `#alScale`, `#alFit`, `#alReset`); Preview background (`#alBgSw`,
  five swatches: white, checker, `#101218`, `#3bbf5a`, `#3b6fe2`).
- **Batch** — header shows the live selection count; Tag…, Collect into set…,
  Rename…, Duplicate, Delete. All disabled at zero selection. Sources
  `#alBatchTag`, `#alBatchColl`, `#alBatchRename`, `#alBatchDup`,
  `#alBatchDel`.
- **Build** — Validate pack (reports warning count, `#alValidateBtn`); Apply to
  map; Import pack .zip…; Export pack .zip… (⌘⇧P, `#alExportBtn`, writes
  `pack.json` schema 2 + PNGs as a STORED zip).
- **Clear library…** — destructive.

### 2.4 Data — the Data Manager

Dropdown mirrors the window's five groups. Every item opens the Data manager
window (§9) on the matching route; the dropdown is a shortcut, not a second
implementation.

| Group | Items |
|---|---|
| Import | Maps · Heightmaps (PNG · TIFF) · GIS / GeoJSON · World Data (.zip · fields) · Assets (routes to the Assets menu) |
| Export | Maps (image · tiles) · GIS / GeoJSON · World Data · Assets (pack .zip) |
| Sources | External Sources · Connected Sources · Source Registry |
| Conversion | Coordinate Systems (EPSG ▸) · Format Conversion · Data Transformation |
| Validation | Check Data (shows current warning count) · Repair / Normalize |

### 2.5 Preferences

| Group | Item | Values / behaviour |
|---|---|---|
| Performance | GPU acceleration | Toggle + backend readout (`WebGPU · on`). Replaces `#gpuToggle` / `#gpuTag`. Off falls back to CPU tile passes. |
| | Devices | Expands to a per-device checklist with live utilisation: `GPU 0 · discrete 16 GB 71%`, `GPU 1 · discrete 16 GB 64%`, `iGPU · shared idle`. Unchecking a device excludes it from dispatch. |
| | Multi-GPU mode | `split tiles` (default) · `alternate frames` · `single device`. Split tiles partitions the working grid; alternate frames only helps the 3D viewport. |
| | CPU worker threads | Integer, 1…logical cores, default cores − 4 (`12 of 16`). |
| | VRAM budget | GB, default 75 % of the smallest active device. |
| | Fallback when VRAM full | `CPU tile pass` (default) · `reduce working res` · `fail with error`. |
| Graphics | Render quality | `performance · balanced · quality · ultra`. Controls sample counts in the appearance pipeline only. |
| | Anti-aliasing · anisotropy | `off · MSAA 2× · MSAA 4× · MSAA 8×`; anisotropy 1–16. |
| | Colour management | `sRGB` · `Display P3` · `linear`. |
| | 3D viewport defaults | Submenu: relief exaggeration, detail, light, flatten oceans. Replaces `#genV3dSec` — exempt from the finalize lock. |
| | Lighting rig defaults | Azimuth, elevation, ambient, multidirectional on/off. |
| Tiles & LOD | Tiled LOD | `auto on zoom` (default) · `manual`. Replaces `#lodAutoChk`. |
| | Tile size · LOD levels | 256/512/1024; levels 0–8 (`#lodMaxLevel`). |
| | Atlas cache | Size cap in GB + Clear (`#lodBakeBtn`, `#lodClearAtlasBtn`). |
| | Chunk debug overlay | `off · grid · colours` (`#lodDbgSeg`) + tile borders. |
| Memory | Undo history | Steps, 1–50, default 5. |
| | Working set | Read-only, `1.6 GB of 12 GB`. |
| | Clear caches… | Confirmation; clears atlas + field caches, never project data. |
| Application | Storage locations… | Same modal as File. |
| | Theme | `dark` · `light` · `follow system`. Tokens in §11. |
| | Units | `km` · `mi` (`#calUnitSeg`). |
| | Keyboard shortcuts… | Editable table, per-context. |

### 2.6 Window

Toggles for each dock and bar (Left dock, Right dock, Timeline, Status bar,
Domain rail), Reset layout, Save layout as…, and the workspace list. Windows
opened from other menus (Asset library, Data manager) appear here while open.

### 2.7 Help

Documentation, Keyboard shortcuts, Credits & academic principles
(`#creditsBtn`), Report an issue, About (version + build).

---

## 3 · Domain rail

40 px collapsed column between the frame edge and the left dock. Five domains,
vertical labels, active in accent:

| Domain | Left dock shows | Right dock shows |
|---|---|---|
| WORLD | Generation Pipeline / Sculpt switch (§5) | Sample readout |
| CIVIL | Settlements, population, economy, politics, culture | Selection inspector |
| INFRA | Roads, rivers, ports, trade, logistics | Route/journey inspector |
| CARTO | Layer list + layer properties (§7) | Ramp / stop editor |
| RENDER | Terrain appearance groups | Preview & quality |

Every left dock opens with the TOOLS block described in §4.5; below it comes the
domain's own structure.

Behaviour: clicking a domain swaps both docks and the tool options bar; the
viewport, camera, selection and the armed tool persist. Expanding the rail (`›`) shows the
domain's sub-nodes as a 200 px list. The rail foot shows the active context
(`TERRAIN`, `SCULPT`, `STYLE`) and, in the World domain, the stage counter
(`04 / 10`).

Nothing else is a workspace. Rendering, LOD and 3D belong to Preferences ▸
Graphics / Tiles & LOD; settlements and routes to CIVIL; terrain appearance to
CARTO.

---

## 4 · Tool options bar

Contextual, one row, always reflecting the active tool or workspace. Layout is
always: context label (accent) → parameters → spacer → commit/discard.

Raise/Lower brush (default): `SCULPT · RAISE` · hardness 0.35 · intensity
+120 m · raise / lower / smooth · **commit pass** · discard.

Generation Pipeline: `GENERATE · WORLD` · Run stage 04 · Run 04 → 10 · New seed
· stale-from readout · 🔒 Bake ALL & finalize.

Sculpt: `SCULPT · RAISE` · feature · preset · radius · falloff · mode · ↶ Undo ·
↷ Redo · ✓ Commit to map · Discard draft.

Cartography: `CARTOGRAPHY · STYLE` · preset chips (Atlas / Parchment / Physical
/ Ink) · `custom — edited since preset` · Reset · Save preset.

---

## 4.5 · Tool palette

Ports v2.10's unified tool palette (`_civTool`, `[data-civtool]`, `_civSetTool`).
**One tool is armed at a time, globally.** Arming a tool never changes the
workspace, and switching workspace never disarms the tool — the two are
orthogonal, which is why the palette is a block and not a mode.

Every left dock opens with a **TOOLS** block: first the four global tools, then
that domain's own. This does not change what §3 says a dock is for — the dock
still presents the domain's structure; the palette is how the map is touched.

### 4.5.1 Global tools — every domain

| Tool | Key | Glyph | Drag / click | Tool options row | Right dock |
|---|---|---|---|---|---|
| Inspect | V | Arrow | Click selects the topmost object under the cursor — place, POI, label, icon, way, route, faction area, stamp | `INSPECT` · what-to-hit filter chips (places · labels · icons · ways) | The matching inspector from §6. **Inspect is what makes every inspector reachable** — without it armed, clicks pan. |
| Measure | M | Ruler with ticks | Click to drop points; double-click or Esc ends | `MEASURE` · mode segment / path · running total in project units · clear · ✕ | Segment table (bearing, length), total, straight-line vs along-path difference |
| Region select | R | Dashed rectangle | Drag a marquee; handles resize it, Esc clears | `REGION` · x / y / w / h in cells and km · lock aspect · snap to tile grid · Use as export extent | Extent in both units, cell count, tile estimate per LOD, and *Send to Data ▸ Export* |
| Pan / zoom | Space (held), MMB | Hand | Always available as a modifier, even with another tool armed | — | unchanged |

Region select is the marquee §9's export route was missing: dragging it fills the
route's world-bounds fields, and the route's fields write back to the marquee.
Neither is authoritative — they are two views of one rect (`region_export.rs`).

### 4.5.2 WORLD tools

| Tool | Key | Drag / click | Tool options row | Right dock |
|---|---|---|---|---|
| Sculpt features (13) | — | Stroke or radial per §5.2 | Per §4 | Stamp stack |
| Freehand | F | Continuous drag / tap | Sub-mode row per §5.2 | Stamp stack |
| Biome paint | B | Drag paints cells, ⇧ erases | `PAINT · BIOME` · target field · value swatch from the target's legend · radius 6 · hardness · softness · erase · land only · ✓ Commit | Painted-cell count, the target's legend with painted counts per class, Commit / Discard |

Biome paint edits world data, so it belongs to WORLD, not to Cartography — in
v2.10 the paint brush sat in the Cartography branch (`#carPaintChk`, with value ·
radius · softness · erase, land only), which the presentation-only rule in §7 now
forbids. The controls port unchanged; only the home moves.

Paint is draft-then-commit like sculpting: strokes accumulate in a scratch layer,
Commit writes them and marks stages 09 and 10 stale (a painted biome overrides
classification for the cells it covers; soils and resources depend on it).

*Answered by the engine, 2026-08-19* — the open question this section raised was
which fields `PaintStamp` may legally write. Read from `cartalith-spatial/src/
paint.rs` and the reference it ports (`_paintAt`/`getPaintLayer`, lines
4754-4795): there are **three** paint layers, not four, and soil, lithology and
vegetation cover are not among them.

| Target | Palette | Reference array |
|---|---|---|
| Biome | `CART_BIOMES` | `paintBiome` |
| Terrain — surface underfoot | `CART_TERRAINS` | `paintTerrain` |
| Splat — asset-pack ground texture | `SPLAT_PAINT_SLOTS` | `paintSplat` |

All three are lazily-allocated `u8` grids where **0 means unpainted** and the
render falls through to the procedural pipeline; any other value is a 1-based
palette index. They differ only in which palette the value indexes, which is why
one `PaintStamp` serves all three and the caller owns which array it writes.
Soil and lithology are *computed* fields with no override array behind them, so
offering them would be inventing a feature rather than surfacing one. **The
target selector lists exactly those three.**

### 4.5.3 CIVIL tools

| Tool | Key | Drag / click | Tool options row | Right dock |
|---|---|---|---|---|
| Settlement | S | Click drops a place (`civ_drop_place`) | `CIVIL · SETTLEMENT` · class (metropolis / city / town / village / hamlet) · faction · name (blank = generated) · snap to water · pick radius | The new settlement's inspector, live, focused on the name field |
| POI | P | Click drops a point of interest (`_civDropPOI`) | `CIVIL · POI` · kind · faction · name · snap to way | POI inspector |
| Territory | T | Drag paints the armed faction's claim (`merge_territory_paint`), ⇧ subtracts | `CIVIL · TERRITORY` · faction swatch · radius · add / subtract · respect coastlines | Faction inspector with live area, claimed-cell count, and contested-cell warning |

Settlement and POI are **two tools, not one** — v2.10 keeps `place` and
`place_poi` separate because they write different records. Territory paint takes
pointer capture and is LOD-aware, so it lands on the right cells under deep zoom.

### 4.5.4 INFRA tools

| Tool | Key | Drag / click | Tool options row | Right dock |
|---|---|---|---|---|
| Way | W | Click appends a waypoint; Esc commits (`_civCommitWay`) | `INFRA · WAY` · way type (road / track / trail / bridge) · routing mode freehand / snap / least-cost (`DijkstraPath`) · snap to places · ↶ ↷ · ✓ Commit | Way inspector: waypoint list, length, grade profile, surface |
| Route | ⇧R | Click appends a stop; Esc commits (`_civCommitRoute`) | `INFRA · ROUTE` · vessel / party reference · snap to places · ↶ ↷ · ✓ Commit | Route inspector with the cost trace and break-even from §6, per-stage overrides |

Way and Route are also **two tools**: a way is durable geometry others route
over, a route is a journey along existing geometry. v2.10 separates them
(`draw_way` vs `route`) and so does this.

While either is armed, hovering shows the live snap preview — the place or way a
click would land on is highlighted. Snap to places is a shared modifier, on by
default.

### 4.5.5 CARTO tools

| Tool | Key | Drag / click | Tool options row | Right dock |
|---|---|---|---|---|
| Label | L | Click an empty spot creates a label; click an existing one edits it in place | `CARTO · LABEL` · text · size mode (fixed / scale with zoom) · arc curvature · letter-spacing · anchor · font role | Text field, size, arc, anchor, on-canvas handles for the baseline and its two arc handles |
| Icon | I | Click stamps the armed icon (`place_manual_icon`) | `CARTO · ICON` · family · variant · scale · rotation · jitter | Placed-icon properties; the armed icon is shown as a chip so it is obvious the map is loaded |

Labels and icons are cartographic annotation, so both are presentation — the
exception §7's prohibition allows, because they add nothing to and take nothing
from the world model. Both keep their list panels (`#carLabelList`,
`#carIconList`) with counts and Clear-all.

The Asset library arms an icon and closes; the Icon tool is what places it. If a
library slot is armed while no Icon tool is active, arming it switches the tool —
the two are one gesture.

### 4.5.6 Shared rules

- **Escape** commits an in-progress multi-click tool (way, route, measure) and
  otherwise disarms back to Inspect.
- **Delete** removes the current selection: a place, POI, label, icon, way,
  route, or stamp. Never a generation stage.
- Arming any tool clears the previous tool's in-progress geometry and its armed
  icon or paint mode.
- A tool that writes world data (Biome paint, Settlement, POI, Territory, Way)
  reports its staleness consequence in the status bar the moment it commits.
- All tools are locked while the world is finalized, except Inspect, Measure,
  Region select, Label and Icon.
- On touch, every armed tool gets the pan joystick (v2.10 `#sculptNavpad`) so a
  single-finger drag can pan without fighting the stroke.

---

## 5 · Left dock · World domain

Header is a two-button switch: **GENERATION PIPELINE | SCULPT**. One is always
active; the switch persists per project.

### 5.1 Generation Pipeline

Ten stages, ordered by dependency. Each row: number, state dot, name, state
label, disclosure chevron, then a `needs` line and a `produces … → consumed by`
line. States: `resolved` (✓, dim), `editing` (● accent), `stale` (○ accent).

| # | Stage | Needs | Produces | Key controls |
|---|---|---|---|---|
| 01 | Planet | — | gravity, rotation, tilt, geoid, tides | gravity 1.00 g, day 24 h, tilt 23.4°, geoid sea level, tides + moon mass/distance/k₂ |
| 02 | Extent & scale | 01 | land/sea split, all distances | region/world, working res 512–8K, sea level 42 %, peak altitude 4000 m, units |
| 03 | World structure | 01 | `continentality.f32` | archetype (Earth, Super, Islands, Volcanic, Rift, Custom); continentality 0.30, fragmentation 0.50, tectonic energy 0.60, ocean depth 0.60, hotspot density 0.20 (`#wsEnabled`, `#wsPanel`) |
| 04 | Tectonics | 01, 03 | elevation, plate_id, boundary_type, resistance | plates 14, drift ×1.00, warp 0.45, uplift spread 18 px, α 0.85, β 0.22, erosion/age 0.60; advanced: flexure F 0.20, heterogeneity C 0.08, rock resistance 0.50 |
| 05 | Volcanism & impacts | 04 | cones, provinces, craters | volcanoes 20, volcano age 0.40, provinces on, craters 100 |
| 06 | Erosion | 04, 08 | final surface | droplet, hillslope diffuse, stream-power, velocity, glacial, coastal — each its own group with its own run button |
| 07 | Hydrology | 06 | rivers, lakes, drainage, flow accumulation | river density, min stream order, lakes as water |
| 08 | Climate | 01, 02, 06 | temperature, rainfall, wind, currents | latitude band, equator/pole °C, lapse rate, seasons & Köppen, currents, ice albedo, weather sim iterations |
| 09 | Ecology & biomes | 07, 08 | biome classification, ecotones | rivers in biome view, ecotone sharpness |
| 10 | Resources & soils | 04, 08, 09 | soil depth, ore, fertility | — |

Stale propagation: editing stage *n* marks every downstream stage stale. The
tool options bar, status bar and right dock all report it; fields owned by stale
stages read `—` until re-run. Run stage *n* re-runs only that stage; Run *n* → 10
walks the chain.

The dock foot carries **Finalize · LOD 0–3 · 85 tiles / bake & freeze**
(`#bakeAllBtn`, `#unfinalizeBtn`). Finalizing locks stages 01–10 and Sculpt; the
3D viewport stays available.

Below the stage list, a **NOT A GENERATION STAGE** block states where
non-generation settings live: GPU acceleration and multi-GPU → Preferences ▸
Performance; render quality, lighting, 3D viewport → Preferences ▸ Graphics;
tiled LOD, atlas cache, chunk debug → Preferences ▸ Tiles & LOD; terrain
appearance, style presets, ramps → Cartography workspace; settlements, routes,
politics → Civilization workspace.

### 5.2 Sculpt

Stamp-based, non-destructive. Ports v2.10 `#genSculpt` and its `SCULPT_FEATURES`
registry unchanged — the panel is a view onto that registry, so adding a feature
in code adds it here with no UI work.

Panel order follows how the tool is actually used: pick the feature, seed it from
a preset, tune that feature's own parameters, then the shared brush block.

**Geological feature** (`#sculptFeatureSeg`) — 13 entries, each with a bespoke
line glyph (12 px, 1.2 px stroke, currentColor — no emoji), the registry's label
and its hint. Selecting one swaps the parameter block below
and the hint line (`#sculptFeatHint`).

| Feature | Interaction | Mode | Parameters (min–max, default) |
|---|---|---|---|
| Mountains | stroke | add | Height 0.10–0.55 (0.42) · Peak sharpness 0.6–3.0 (1.5) · Ridge freq 0.6–5.0 (1.6) · Ruggedness 0–1 (0.55) |
| Hills | stroke | add | Amplitude 0.02–0.30 (0.11) · Rolling freq 0.5–4.0 (1.4) · Softness 0–1 (0.7) |
| Ridge | stroke | add | Height 0.02–0.35 (0.15) · Width frac 0.1–0.6 (0.28) · Detail freq 0.5–4.0 (1.5) |
| Plateau | stroke | set | Rise 0.03–0.45 (0.26) · Terraces 1–8 (4) · Detail freq 0.4–3.0 (1.1) — never lowers existing terrain |
| Cliff / Escarpment | stroke, direction-sensitive | add | Rise 0.05–0.45 (0.22) · Steepness 0.2–1.0 (0.75) — high side is left of the stroke |
| Canyon | stroke | add (negative) | Depth 0.03–0.35 (0.18) · Wall steepness 0–1 (0.7) · Meander 0–0.8 (0.35) |
| Valley | stroke | add (negative) | Depth 0.03–0.30 (0.14) · Width frac 0.3–1.0 (0.85) · Meander 0–0.8 (0.3) |
| River | stroke | set | Width 2–26 px (7) · Depth 0.02–0.22 (0.09) · Meander 0–0.6 (0.28) · Branch noise 0–1 (0.5) — writes riverMask/riverFloor on commit |
| Lake | radial, brush = radius | set | Depth 0.03–0.30 (0.13) · Shore 0.05–0.6 (0.25) — fills lakeMask on commit |
| Basin | stroke | add (negative) | Depth 0.02–0.25 (0.1) · Floor rough 0–1 (0.4) — endorheic, no outlet |
| Coastline | stroke | set | Amount 0.1–1.0 (0.85) · Raggedness 0.4–4.0 (1.6) — pulls toward sea level |
| Volcano | radial, brush = radius | add | Cone height 0.15–0.6 (0.45) · Crater depth 0–0.9 (0.5) · Radius 30–200 px (110) · Flank rough 0–1 (0.6) |
| Freehand | continuous drag or tap | per sub-mode | Amount 0.02–0.30 (0.12) |

**Presets** (`#sculptPresetSeg`) — eight one-click parameter seeds, each bound to
a feature: Rolling Hills (hills), Alps (mountains), Rockies (mountains),
Badlands (canyon), Volcanic Isle (volcano), Mesa (plateau), Karst (hills),
Glacial Valley (valley). A preset sets the feature and its parameters; it never
paints.

**Feature parameters** (`#sculptFeatureControls`) — the selected feature's own
controls from the table above, titled with the feature name. Radial features show
their radius control here rather than using the global brush size.

**Freehand tools · direct drag** (`#sculptModeSeg`, shown only for Freehand) —
Raise · Lower · Smooth · Cliff · Ridge · Canyon · Mesa · Volcano. Raise/Lower/
Smooth follow the drag; Cliff/Ridge/Canyon follow its direction; Mesa/Volcano
stamp once at a tap (a one-point stroke degenerates to radial distance).

**Brush & noise · global** — applies to every feature (`#sBrush` … `#sSeed`):

| Control | Range | Default | Notes |
|---|---|---|---|
| Brush size | 6–200 px | 64 | Shows the km equivalent at the working resolution (`#sBrushKm`) |
| Hardness | 0–1 | 0.35 | Feather = radius × (1 − hardness) |
| Intensity | 0–1.5 | 1.00 | Scales the feature's own amplitude |
| Noise scale | 1–20 | 6.0 | — |
| Octaves | 1–8 | 5 | — |
| Persistence | 0.20–0.90 | 0.52 | — |
| Lacunarity | 1.40–3.20 | 2.00 | — |
| Edge noise | 0–1 | 0.45 | Multiplied by each feature's `edgeChar` / `edgeFreqMul` |
| Seed | integer | project seed | Dice button randomises |

**Brush shape** — falloff-profile preview, eight built-in shapes (circle,
directional, spatter, spiral, dots, cloud, checker, hatch), Import brush…
(greyscale height stamp, alpha respected), Operation (defaults to the feature's
own mode — add or set — overridable to subtract/multiply/min/max), Falloff
(smooth / linear / sharp / constant / custom), Rotation 0–360°, Spacing 0–1,
Mirror across the stroke axis.

**Stroke & grid** — Add point · Duplicate · Rotate · Scale · Tilt · Push · Pull ·
Align. These edit the selected stamp's control points, not the heightfield.

**Actions** — Flip X · Flip Y · Rot Left · Rot Right · Flatten selection.
Immediate, applied to the selection, undoable.

Every stroke becomes a live procedural stamp (`sculptStamps`). Nothing touches
the real heightfield until Commit (`#sculptCommitBtn`), which bakes the whole
stack in one pass and re-runs erosion, hydrology and climate once. Discard
(`#sculptDiscardBtn`) drops the draft. Sculpting is locked while the world is
finalized (`#sculptFinalizedNote`).


## 6 · Right dock · contexts

Contents follow the selection, not the workspace. Selections are made with the
Inspect tool (§4.5.1); with any other tool armed a click performs that tool's
action instead.

| Context | Contents |
|---|---|
| No selection (Sample) | X, Y, elevation (large accent readout), slope, aspect, plate + type, boundary + distance, resistance, lithology, temperature, precipitation, drainage, biome, soil, control, nearest settlement. Fields from stale stages read `—`. |
| Layers | Ordered list with visibility dot, name, opacity bar, blend mode; nested children under Terrain. |
| Stamp stack (Sculpt) | Stamps newest-first with index, visibility, type, parameter summary; actions Deselect · Hide/show · Move up · Move down · Delete; selected-stamp parameters (length, width, asymmetry, ridge noise, blend); ↶ Undo · ↷ Redo; ✓ Commit to map · Discard; finalize-lock note. |
| River | Name, length, source elevation, discharge, catchment, tributaries, navigation + Hydrology / Edit geometry / Analyse catchment. |
| Settlement | Name, population, class, government, agriculture, trade, water access, defensibility, routes + Economy / Politics / Logistics. |
| Faction | Roster entry, territory, provinces, state religion (`#civFactionInspectorHost`). |
| Route | Stages, vessels, cost trace, per-stage overrides, daily stages. |
| Brush / Stamp | Size, hardness, intensity, noise scale, octaves, persistence, lacunarity, edge noise, stamp stack, commit / discard. |

Collapsed, the dock keeps the primary readout visible: elevation for Sample,
layer dots for Layers, stamp count for the stack.

---

## 7 · Cartography → Style

Three panes, left to right, mirroring how a map style is actually edited.

**Layer list** — search field, then the ordered stack: Labels & annotation,
Settlements, Ways & routes, Political (off), Water, Vegetation, Terrain
(selected) with children Hand-drawn hillshade (off), Hillshade (active), Colour
relief, then Land, Background. Footer tabs Blocks / Verticality.

**Layer properties** — header shows the selected layer with a close affordance.
Groups: LAYER (visibility, visualization dropdown, opacity 78), FILL (colour
ramp picker, domain World/View/Absolute, range −410 → 4 210 m), LIGHT (azimuth
315°, elevation 45°, strength 0.62, multidirectional 8 lights). Footer tabs
Style / Data / `{ }` (JSON view).

**Colour ramp popover** — anchored to the Fill row. Nine named ramps as
full-width swatches (Earth, Elevation, Atlas, Mono, Imhof, Ice, Dark ice,
Desert, Dark atlas) plus *Create custom ramp…*. The active ramp is outlined in
accent and its row filled; filled rows use reversed paper-coloured type.

**Stop editor** (right dock) — ramp bar with draggable stops, ＋ add / delete /
reverse, then the selected stop: elevation (2 640 m), colour swatch + hex
(#B9A878), alpha, interpolation to next stop (Linear / Ease / Step). Footer
Compare · Apply.

Presentation only: no control in this workspace may alter heightmap, climate,
hydrology, biome classification, settlements, routes or seed, and none marks a
generation stage stale. Status bar reports `style edited — 2 layers differ from
preset Atlas` and repaint time.

---

## 8 · Asset library window

Opens over the map (`⇧A`); the map is hidden while open. Titled `⧉ ASSET
LIBRARY`. Replaces v2.10 `#assetLibrary`.

**Window bar** — search (`name · type · category · tag · file`, `#alSearch`),
Sort (slot order / name / filled first, `#alSort`), ▦ Sprite sheet…
(`#alSlicerBtn`), ☑ Select with live count (`#alSelModeBtn`), then Apply to map,
Export pack .zip, Close.

**Family rail** — 24 families grouped Settlements (Places, Buildings, Walls &
gates), Terrain (Trees & cover, Rock & scree, Textures, Hachure & hatch),
Cartography (Compass & frame, Label plaques, Ship & sea marks, Map furniture),
plus Collections (tag sets, Unassigned imports with count). Each row: code,
name, `filled/capacity` — accent when incomplete. Footer: Import image… ·
Import pack….

**Slot grid** — family-scoped, six columns. Header states
`P · PLACES · 10 OF 12 FILLED` and the selection count, with the batch actions
Tag… Collect… Rename… Duplicate Delete. Cells show a checkerboard when empty,
the art when filled, `×3` when the slot has variants, and ☑ when selected.
Footer: drop-to-fill hint, ⇧-click ranges, ⌘-click adds, zoom control.

**Slot inspector** — slot code + name; preview on a checkerboard; file readout
(`capital-star.png · 512 × 512 · PNG · 84 KB`); Scale 118 % with Fit / Reset /
Replace… / + Variant; Preview bg five swatches; Anchor top/centre/base; Tags
with ＋; Variants strip (3, active outlined); Pack metadata name / author /
license. Footer: Validate · 8 warnings, Clear library….

**Sprite sheet slicer modal** — sheet preview with the cell grid overlaid and a
readout (`towns-sheet.png · 3072 × 2048 · 24 cells detected · 19 non-empty`);
Columns 6, Rows 4, Margin 8 px, Spacing 4 px, Trim transparent edges, Skip empty
cells, Assign to family, Fill from `first empty` / `overwrite`; summary line;
Cancel · Slice 19 cells. Non-destructive — originals stay in the library.

Status bar: `library edited — apply to map to use it`, unassigned import count.

---

## 9 · Data manager window

Titled `⧉ DATA MANAGER`, subtitle *import · export · sources · conversion ·
validation*. Every route in and out of the project; nothing here alters world
data.

**Routes rail** — the five groups from §2.4. The Assets rows link back to the
Assets menu rather than duplicating pack handling. Foot: exports root and last
run (`14:02 · 62 MB`).

**Route pane** — breadcrumb header (`EXPORT ▸ MAPS ▸ LEAFLET TILE PYRAMID`),
then two columns:

- TILES — scheme (XYZ / TMS / WMTS), zoom range, tile size 256 px, format
  (PNG-8 · WebP fallback), Retina @2x, Skip all-ocean tiles (−1 842 tiles).
- PROJECTION — CRS (EPSG:3857 / EPSG:4326 / custom), world bounds, write world
  file (.wld + .prj).
- LAYERS INCLUDED — relief + hillshade, political tint, labels & icons (raster),
  rivers & coastlines.
- OUTPUT — destination, packaging (folder / .zip / MBTiles), emit
  `leaflet-preview.html`, emit `style.json` + attribution.
- ESTIMATE — tiles (5 461 → 3 619 after ocean skip), size (~214 MB), render time
  (~3 min 40 s · 2 GPUs), source (`baked atlas L0–L3 · no re-gen`).
- MARKDOWN VAULT · LINKED — path and note count (`~/Vaults/Eldra · 412 notes`);
  settlements, factions and journeys resolve to notes by name; exported tiles
  carry `obsidian://` links. Toggles: two-way sync (write place notes back, off),
  link labels to notes in GeoJSON, include front-matter as properties. Actions:
  Re-scan vault · Change folder… · Unlink.
- RECENT RUNS — timestamp, route, size, result.

Footer: destination path, Save as preset · Dry run · Export 3 619 tiles.

---

## 10 · Viewport, timeline, status bar

**Layers button** — 36 px canvas button, top-left of the viewport (v2.10
`#layersFab`). Its popover carries a master opacity slider then grouped rows with
hotkey badges: SURFACE (Relief 1, Biome 2, Political 3), TERRAIN FIELDS
(Elevation 4, Slope 5, Flow accumulation 6), CLIMATE (Temperature 7, Rainfall 8).
The active row is filled accent with reversed type. The popover must not overlap
an open menu; it opens below the menu band.

**Tool overlays** — the measure path with per-segment lengths and bearings, the
region marquee with corner handles and a live cell/km readout, the brush ring for
paint and sculpt, the armed-icon ghost under the cursor, and the snap-preview
highlight while a way or route is being drawn.

**Other viewport furniture** — top-left context readout (active stage or draft
state), top-right projection/zoom/field, bottom-left scale bar, bottom-right
cursor coordinates and elevation (`4 812 km E · 1 093 km N · 1 462 m`, showing
`→ 1 582 m` while a draft stamp is under the cursor), brush ring with radius and
strength label.

**Timeline bar** — scrub track spanning the project's year range with the
current year marked in accent (`YEAR −400 … YEAR 412 … YEAR 1200`), transport
▶ Play · ⏸ Pause · Step ◀ ▶, speeds ×1 / ×10 / ×100, run state (`PAUSED`), and
simulation-layer toggles Climate · Population · Economy · Politics ·
Infrastructure · Warfare. Absent from generation and style screens — generation
is not time-based.

**Status bar** — left: the one thing that needs attention, in accent
(`editing terrain — 3 uncommitted strokes`, `stage 04 edited — 6 downstream
stages stale`, `style edited — 2 layers differ from preset Atlas`,
`draft — 5 stamps uncommitted`). Middle: last heavy pass / repaint / autosave.
Right: the two or three shortcuts that apply right now.

---

## 11 · Theme tokens

| Token | Dark | Light |
|---|---|---|
| Surface | `#0d0e0f` | `#f4f2ee` |
| Panel | `#121314` | `#fbfaf7` |
| Inset / cell | `#191c1e` | `#eceae4` |
| Ink primary | `#e8ebec` | `#111210` |
| Ink body | `#c8cbcd` | `#23241f` |
| Ink secondary | `#a9adb0` | `#3d3f39` |
| Ink dim | `#8d9296` | `#6b6f6a` |
| Ink faint | `#6f7478` | `#8d9088` |
| Ink disabled | `#5f6468` | `#9a9d95` |
| Accent | `#e0a34a` | `#a4650f` |
| Accent hover | `#f0bd72` | `#8a5309` |
| Hairline | `rgba(255,255,255,.10)` | `rgba(0,0,0,.14)` |
| Divider | `rgba(255,255,255,.07)` | `rgba(0,0,0,.08)` |
| Border | `rgba(255,255,255,.16)` | `rgba(0,0,0,.20)` |
| Active wash | `rgba(224,163,74,.09)` | `rgba(164,101,15,.09)` |
| Menu shadow | `0 14px 34px rgba(0,0,0,.55)` | `0 14px 34px rgba(35,36,31,.16)` |
| Viewport wash | `radial-gradient(#17191a, #101112, #0d0e0f)` | `radial-gradient(#faf9f5, #f1efe9, #eae7e0)` |

Type: UI in Helvetica Neue / system sans at 11–11.5 px desktop, 13–14 px tablet,
13 px phone. All numeric readouts, codes, shortcuts and section labels in IBM
Plex Mono, 9–11 px, letter-spacing .12–.22 em for labels. Filled accent surfaces
carry reversed paper-coloured type in both themes, never near-black on light
amber.

No fills on panels: regions are separated by hairlines only. Radius 0 everywhere.

---

## 12 · Iconography

No emoji anywhere in the product. Every glyph is a bespoke inline SVG on a
16 × 16 viewBox, rendered at 12 px in panels and 14–17 px on canvas buttons,
`fill:none`, `stroke:currentColor`, `stroke-width:1.2`, round caps and joins.
Because they take `currentColor` they inherit the accent when their row is
active and invert with the light theme without a second asset.

The thirteen sculpt features are drawn as terrain cross-sections, read as one
family, and are the only place icons carry meaning rather than decoration:

| Feature | Glyph |
|---|---|
| Mountains | Two overlapping peaks, the taller behind |
| Hills | A rolling profile above a baseline |
| Ridge | One peak with its strike axis drawn down the centre |
| Plateau | Flat-topped mesa with the top edge emphasised |
| Cliff | A single step, high side left |
| Canyon | Two walls closing to a flat floor |
| Valley | A U-shaped trough |
| River | Two braided meanders |
| Lake | A closed basin with one shoreline mark |
| Basin | Nested shallow bowls, no outlet |
| Coastline | A ragged edge above a water line |
| Volcano | Truncated cone with a crater notch |
| Freehand | A pencil |

The tool palette adds twelve glyphs, drawn to the same rules:

| Tool | Glyph |
|---|---|
| Inspect | A plain arrow cursor |
| Measure | A ruler edge with three ticks and an end pin |
| Region select | A dashed rectangle with solid corner ticks |
| Pan / zoom | An open hand |
| Biome paint | A round brush head with a single drip |
| Settlement | A house outline |
| POI | A diamond with a centre dot |
| Territory | A hatched quadrilateral |
| Way | Two rails with three sleepers |
| Route | A dashed path with an arrowhead |
| Label | A tag with its hole |
| Icon | A hollow outlined diamond |

Other drawn glyphs: the layers button (three stacked sheets), the dice on the
seed row, and the `⧉` window marker in menus. Text symbols stay text — `▾ ▸ ‹ › ⌄
● ○ ☑ ☐ ✓ ✕ ＋ ⌫ ↶ ↷ ▶ ⏸ ☰ ▤ ⋯ 🔒` — since they are typographic, inherit type
metrics, and need no drawing.

Rules: one weight only (1.2 px) so a glyph never reads bolder than the hairlines
around it; no fills except 0.7 px dots where a mark must survive at 12 px; nothing
inside a glyph smaller than 1 px at render size; and every icon is legible in
both themes at 12 px before it ships.

---

## 13 · Touch behaviour

Tablet keeps full desktop parity — same regions, same menus, same disclosure
depth, targets 44–52 px, docks 400 px.

Phone reorganises rather than truncates:

- Map draws edge-to-edge behind every inset.
- Top 44 px is a keep-clear safe area: status glyphs only, in left and right
  pockets, with a 108 px centre lane reserved for a punch-hole or notch. Nothing
  is centred there. A gradient scrim, not an opaque bar, carries legibility.
- The app bar below it is the first row allowed to hold controls: ☰ (domain
  drawer), title + seed, ▤ (panels), ⋯ (overflow menu carrying the full menu
  bar).
- Domain rail is a 44 px column with each domain in a 44 px hit box.
- Tool options become a bottom sheet; docks become full-height sheets, one at a
  time; all five disclosure levels survive inside them.
- Bottom 26 px is the gesture inset — no tappable target inside it. Timeline and
  sheets stop above it.
- In landscape the cutout moves to a side edge; apply the same reserve
  horizontally.

Minimum target 44 px, measured inside the safe area, with no exceptions.
