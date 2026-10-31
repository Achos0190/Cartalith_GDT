# Cartalith shell: domains, nodes, tools and docks, as the code builds them

Read-only inventory for the phone and tablet redesign. Taken from the shipped GDScript shell and the Rust registries behind it on 2026-10-31. Nothing in the repository was edited.

**Where code and a design disagree, the code is what this file reports. The design's version is given beside it** (section 8 collects them). Labels are quoted as the code writes them. Dynamic lists (sculpt features, landmark kinds, parameter rows, layer fields, presets) are read from the Rust tables that feed the GDScript, not from a design.

---

## 0. Method, sources, and what to trust

**Method (MISTAKES.md "Enumerate a surface for an audit").** I walked the code first, then checked each item against the designs. Entry points opened:

- `cartalith-native/godot-project/shell/dcc_shell.gd`: `DOMAINS`, `RAIL_NODES`, `_MODE_SWITCH_LABELS`, `_build_rail()`, `_build_rail_expansion()`, `_on_rail_node_pressed()`, `_select_domain()`, `select_domain_mode()`, `select_domain_category()`, `PHONE_TABS`, `_pick_phone_tab()`, `_refresh_phone_gen_panel()`, `_refresh_phone_sheet_header()`, `TL_LAYERS`, `TL_SPEEDS`, `TL_YEAR_MIN/MAX`
- `app.gd`: `_install_tool_palette_bar()`, `set_domain_tools()`, `_strip_entries()`, `_on_workspace_changed()`, `_tool_options_generate()`, `_tool_options_cartography_default()`, `_fill_timeline_strip()`, `_build_timeline_expanded()`, `_refresh_rail_foot()`, `toggle_region()`, `arm_tool()`
- `dcc_widgets.gd`: `GLOBAL_TOOL_ENTRIES`, `tools_block()`, `tool_strip()`, `_tool_entry()`, `_tool_shortcut()`
- `tool_bar.gd`: `DccToolBar`, the unified Sculpt / Paint / Measure bar, plus `build_brush_controls()`
- `global_tools.gd`: `MEASURE_MODES`, `SECTION_CHANNELS`, `ring_cardinals()`
- `workspaces/world_workspace.gd`: `STAGES`, `CATEGORIES`, `ADVANCED_KEYS`, `_build_categories()` and every builder it calls, the phone `build_phone_generate()` / `_pg_*` column, and `ring_slots()`
- `workspaces/civilization_workspace.gd`: `_build()` (the 13 categories), `_build_tools()`, every `_build_*` / `_fill_*` / `_lm_*` / `_tl_*` / `_tool_options_*`, and `ring_slots()`
- `workspaces/infrastructure_workspace.gd`: the `build_*_into()` entry points, `_tool_options_way/route()`, `WAY_DRAW_TYPES`
- `workspaces/cartography_workspace.gd`: `_build()`, `LIVE_LAYERS`, `WAY_TYPES`, `SETTLEMENT_KINDS`, `ICON_FAMILIES`, the label and icon builders and their tool-options rows, and `ring_slots()`
- `workspaces/render_workspace.gd`: `build_style_into` / `relief_into` / `colours_into` / `layer_stack_into`, `STYLES`, `STYLE_PRESETS`, `APPEARANCE_GROUPS`, `APPEARANCE_VIEW`, `APPEARANCE_BIOME`
- `colour_paint_tool.gd`, `adjust_panel.gd`, `paint_original.gd`, `landmark_filter_panel.gd`, `rock_paint_ui.gd`
- `right_dock.gd`: `CTX_*`, `TOOL_*`, `_tool_section()`, `_append_tool()`, `_append_layers()`, `SAMPLE_FIELDS`
- `phone_menu.gd`: `ROOT_ROWS`, `ROOT_REST`, `CIV_TOOLS`, `_fill_civ()`, `_fill_sim()`, `_fill_landmarks()`
- `journey_planner_view.gd`, `new_world_dialog.gd`, `context_broker.gd` (`ring_collect`), `tool_fan.gd`, `viewport_host.gd`
- Rust:
  - `crates/cartalith-godot/src/params.rs` (`PARAMS`, `defaults()`)
  - `crates/cartalith-engine/src/lib.rs` (`WorldParams::defaults`, `ErosionPassParams::off`)
  - `crates/cartalith-terrain/src/sculpt.rs` (`FEATURE_KEYS`, `*_CTL`, `SCULPT_PRESETS`, `SculptGlobals`)
  - `crates/cartalith-godot/src/sculpt_bridge.rs` (`GLOBAL_RANGES`, `GRID_SNAP_STEPS`)
  - `crates/cartalith-civ/src/landmark.rs` (`kinds()`)
  - `crates/cartalith-godot/src/sample_bridge.rs` (`LAYER_GROUPS`)
  - `crates/cartalith-godot/src/render.rs` (`tunables!`, `TUNABLE_LIGHTS`, `LOOK_PRESETS`, `RAMP_PRESETS`, `RAMP_MODES`, `BLEND_MODES`)
  - `crates/cartalith-godot/src/paint_bridge.rs` (`PaintTarget`, `palette()`)
  - `crates/cartalith-civ/src/labels.rs` (`LABEL_CLASSES`, `LABEL_TYPOGRAPHY_DEFAULTS`, ranges)
  - `crates/cartalith-godot/src/icon_bridge.rs` (`PlacementFamily`)

**Counts quoted in this file, with the command each came from** (run from `/home/user/Cartalith_GDT/cartalith-native/godot-project/shell` unless stated):

| Figure | Command | Result |
|---|---|---|
| rail nodes | `awk '/^const RAIL_NODES: Array = \[/,/^\]/' dcc_shell.gd \| grep -c '"kind": "node"'` | 10 |
| WORLD dock categories | `awk '/^const CATEGORIES: Array = \[/,/^\]/' workspaces/world_workspace.gd \| grep -o '{"name": "[^"]*"'` | 9 |
| CIVIL dock categories | `grep -no 'DccWidgets.category(self, "[^"]*"' workspaces/civilization_workspace.gd` | 13 |
| CARTO dock categories | same grep on `workspaces/cartography_workspace.gd` | 7 |
| generation parameters | `grep -c "ParamSpec { key:" ../../crates/cartalith-godot/src/params.rs` | 120 |
| parameters parsed into Appendix A | a scratch parser over `PARAMS` (in `scripts/params_dump.py` next to this file) | 120 of 120 |
| parameters by group | `grep -o 'group: "[a-z_]*"' params.rs \| sort \| uniq -c` | civ 14, climate 9, erosion 29, planet 3, tectonics 16, volcanism 19, weather 12, world 12, world_structure 6 |
| app-boundary flags forced `true` | `awk '/^pub fn defaults\(\) -> WorldParams/,/^}/' params.rs \| grep -cE "p\.[a-z_.]+ = true;"` | 20 |
| landmark kinds | `awk 'NR>=322 && NR<=460' crates/cartalith-civ/src/landmark.rs \| grep -c 'LandmarkKindSpec { key:'` | 49 |
| analysis fields | a scratch parser over `LAYER_GROUPS` (`scripts/layer_groups.py`) | 43 in 6 groups |
| appearance tunables | a scratch extraction of the `tunables!` block in `render.rs`, plus `TUNABLE_LIGHTS` | 81, plus `relief_lights` |
| style presets | `awk 'NR>=135 && NR<=425' workspaces/render_workspace.gd \| grep -o '^\s*\["[^"]*", "[^"]*"'` | 18 |

**What I did not do.** I ran neither the app nor a probe. Nothing here was measured on a device. Layout claims, such as "the phone shows X in the sheet", come from reading the branch the code takes. They were not taken from a capture.

---

## 1. The frame at a glance

### 1.1 Form factors

- **Desktop and tablet use one composition** (`DccShell._build_desktop_shell()`): domain rail, rail expansion (the node list), left dock, map, right dock, tool palette bar, tool options bar, timeline bar (CIVIL only), and status bar.
  - Tablet is that same shell with touch sizing (`_touch`).
  - Tablet portrait reflows slider rows into two lines (`WorldWorkspace._wrap_slider_cell`, `_on_portrait_changed`).
- **Phone has its own composition** (`_build_phone_shell()`): app bar, map full-bleed, a bottom tool sheet with detents (peek / half / full), and a four-tab bottom bar (MAP · GENERATE · PLAN · MORE).
  - The left and right docks still exist as sheets, hidden by default.
  - There is no domain rail and no node list on the phone. `_build_rail()` and `_build_rail_expansion()` are called only from the desktop composition (`dcc_shell.gd:991`, `:3551`).
- Phone is decided by `_compute_layout_mode()`: touch, an aspect ratio below `_PHONE_ASPECT_MAX`, and not tablet-sized.

### 1.2 Domains (`DccShell.DOMAINS`)

| Order | id | Rail caption (vertical, desktop/tablet) | Menu / dock title | Subtitle (rail expansion, tooltip) |
|---|---|---|---|---|
| 1 | `world` | WORLD | World | Terrain, hydrology, climate and ecology |
| 2 | `civilization` | CIVIL | Civilization | Settlements, factions, provinces, trade, roads, sea routes and journeys |
| 3 | `cartography` | CARTO | Cartography | Layers, styles, labels, annotation and terrain appearance |

- The desktop rail draws **no icons**, only rotated mono captions (owner: "those icons don't exist", `_build_rail()`).
- The phone bottom bar uses literal glyphs instead (see 7.2).
- The rail head is a chevron button, "Show or hide the node list", that opens the rail expansion.
- **Session mode** (Window ▸ Session mode, Ctrl+Shift+M) hides WORLD everywhere: rail mark, node rows, phone GENERATE tab, and Window ▸ Workspace (`SESSION_HIDDEN_DOMAINS = ["world"]`).

### 1.3 The node tree (`DccShell.RAIL_NODES`, rendered flat by `_build_rail_expansion()`)

| # | Domain header | Node label (as displayed) | mode id | Opens category | Lights for (`owns`) | Dock gate (`shows`) | Mode pill text | Rail footer |
|---|---|---|---|---|---|---|---|---|
| 1 | WORLD | Generate | `a` | Generate | Generate, Planet, Geology, Hydrology, Climate, Ecology, World data | same 7 | PIPELINE | `NN / 10` stage counter (`00 / 10` with no world, `10 / 10` with one) |
| 2 | WORLD | Sculpt | `b` | Terrain | Terrain, Biomes | Terrain, Biomes | SCULPT | `SCULPT` |
| 3 | CIVIL | Landmarks | `landmarks` | Landmarks | Landmarks | (none: ungated) | — | `LANDMARKS` |
| 4 | CIVIL | Factions | `factions` | Factions | Populate, Settlements, Factions, Territories, Relationships, Military, Culture, Religion, Economy, Timeline | — | — | `FACTIONS` |
| 5 | CIVIL | Routes & ways | `infra` | Routes & ways | Routes & ways | — | — | `INFRA` |
| 6 | CIVIL | Journey planner | `planner` | Travel, **and arms the Journey takeover** | Travel | — | — | `PLANNER` |
| 7 | CARTO | Style | `style` | Style | Style, Relief & light, Colours, Feature style | — | — | `STYLE` |
| 8 | CARTO | Layers | `layers` | Layers | Layers | — | — | `LAYERS` |
| 9 | CARTO | Labels | `labels` | Labels | Labels | — | — | `LABELS` |
| 10 | CARTO | Icons | `icons` | Icons | Icons | — | — | `ICONS` |

How the tree behaves:

- **Only WORLD gates.** A node press opens its category and lights the rail. In CIVIL and CARTO it never hides a sibling category: each domain's dock is one accordion of all its categories.
- In WORLD the mode pill (PIPELINE | SCULPT, pinned above the dock's scroll body by `_build_mode_switch()`) is the only gate. PIPELINE shows 7 categories; SCULPT shows Terrain and Biomes.
- **Default modes.** Each domain is seeded from its first node (`_default_modes()`): WORLD `a`, CIVIL `landmarks`, CARTO `style`. The boot domain is WORLD (`_active_domain := "world"`).
- **Default-open categories:**
  - WORLD opens Generate (`i == 0`).
  - CIVIL opens Populate, its "floor". Because the floor is owned by `factions` while the rail lights `landmarks` on entry, entering CIVIL lights Landmarks over an open Populate. The `RAIL_NODES` header says this is deliberate.
  - CARTO opens Style.
- **The Journey planner node's extra behaviour lives on the node press** (`_on_rail_node_pressed`), not on the mode write. Pressing any other CIVIL node while Journey is armed disarms it first. ⇧J opens the planner from the keyboard (`app.gd::_unhandled_key_input`).
- **The rail footer prints the mode id upper-cased** (`app.gd::_refresh_rail_foot()`), so CIVIL's footers are the mode ids (`INFRA`, `PLANNER`), not the node labels.
- **The tree's Settlements node is not built.** The owner tree (Ruling L) lists Settlements as the first CIVIL node. The code comment says it would need a fifth CIVIL mode id, which is the owner's call (plan C1).

### 1.4 Bars and docks, desktop and tablet

| Region | What it holds | Source |
|---|---|---|
| Tool palette bar | 4 global cells, a divider, the active domain's tools, a divider, then the user's pinned quick cluster (`DccQuickBar`). Off the phone only | `app.gd::_install_tool_palette_bar()`, `DccWidgets.tool_strip()` |
| Tool options bar | Content depends on the armed tool and domain (sections 2–5). Tablet wraps it in a horizontal `ScrollContainer` | `DccShell.set_tool_options()` callers |
| Left dock | Header (the `DOMAINS` label upper-cased: WORLD / CIVILIZATION / CARTOGRAPHY), the WORLD-only mode pill, then the domain's category accordion | `register_workspace()` per domain; header written in `_select_domain()` |
| Right dock | One selection context, then the appended tool section, then CARTO's "Layers" section (section 6) | `right_dock.gd::_rebuild()` |
| Timeline bar | Visible only while CIVIL is active (`timeline_bar.visible = id == "civilization"`); collapsed by default | `app.gd::_on_workspace_changed()`, `_fill_timeline_strip()` |
| Status bar | Program state slots | `_build_status_bar()` (not inventoried here) |
| Map furniture | Layers button (top-left), which opens the Layers popover with the 43 analysis fields (Appendix C) and a shared opacity; context chip; readout; scale bar; coordinates; cross-section strip under the map while Measure ▸ Cross-section is in use | `viewport_host.gd`, `layers_popover.gd`, `section_strip.gd` |
| Radial ring (desktop/tablet) and thumb fan (phone) | 8 compass slots: N Inspect, E Measure, W Region, S Undo, plus 4 domain diagonals (listed per domain below) | `context_broker.gd::ring_collect()`, `global_tools.gd::ring_cardinals()`, each workspace's `ring_slots()` |

---

## 2. Global tools (every domain)

From `DccWidgets.GLOBAL_TOOL_ENTRIES`. On desktop and tablet they sit at the left of the tool palette bar. On the phone they sit in the dock's TOOLS block and in the thumb fan's cardinals.

| Order | Label (tooltip) | id | Glyph (icon name) | Key | What it does | Tool options bar | Right dock |
|---|---|---|---|---|---|---|---|
| 1 | Inspect (V) | `inspect` | `tool_inspect` | V | Default tool. Hover or tap samples the cell; click selects a settlement, route, river, landmark, icon or faction | Per domain, idle row: WORLD → "GENERATE · WORLD" row (3.0); CIVIL → "CIVIL · INSPECT" plus a note; CARTO → "CARTOGRAPHY · \<MODE\>" plus a note | Sample (default), or the selection's context |
| 2 | Measure (M) | `measure` | `tool_measure` | M | Ruler and analysis measurements | Unified tool bar in MEASURE mode (see below) | Measure context |
| 3 | Region select (R) | `region` | `tool_region` | R | Drag a rectangle; Esc clears | None found. I searched `set_tool_options` callers, `REGION` label strings and every `tool_armed` listener, and found no region row | "Region select" context |
| 4 | Pan | `pan` | `tool_pan` | — | **A legend, not a tool.** Drawn disabled; tooltip "Pan / zoom — always available" | — | — |

**The unified Sculpt / Paint / Measure tool bar** (`tool_bar.gd`) appears when `sculpt`, `paint` or `measure` is armed. It has three rows:

- **Row 1:** mode segments SCULPT / PAINT / MEASURE, then the active mode's tools.
- **Row 2:** that mode's options.
- **Row 3:** a brush row, for Paint and Sculpt only.

In MEASURE mode the rows hold:

- **Row 1:**
  - Segments Distance, Bearing, Area, Radius.
  - A CROSS-SECTION label with channel segments Elevation, Terrain, Climate, Hydrology, Geology. Picking one arms Cross-section.
  - A Δ vertical segment.
  - A note.
- **Row 2:** "MEASURE · \<MODE\>", then per mode:
  - Cross-section: **Samples** slider 32–1024, step 32; **V. exag** slider 1–6, step 1, unit ×.
  - Other modes: a one-line hint.
  - Every mode: a status readout and a **Clear** chip.
- **Measure modes** (`GlobalTools.MEASURE_MODES`):

  | Mode | Points | Reads |
  |---|---|---|
  | Distance | multi-segment; ⌫ drops the last point | distance |
  | Bearing | 2 | bearing |
  | Area | closed ring; needs a world | area |
  | Radius | 2; needs a world | radius |
  | Cross-section | 2; needs a world | profile in the strip under the map |
  | Δ vertical | 2; needs a world | vertical difference |

---

## 3. WORLD

### 3.0 Domain-level

- **Domain tool** (palette, after the divider): **Biome paint (B)**, id `paint`, glyph `tool_paint`, key B, entry `modes: ["b"]`. **Shown only in SCULPT mode** (`app.gd::_strip_entries()`); in PIPELINE the WORLD palette is the four global cells alone.
- **Hidden tool, armed from the dock:** `sculpt`. Armed by any of the 13 feature buttons in Sculpt ▸ Terrain (3.2). **F** arms Freehand (a `Shortcut` on that grid button, live only while it is visible).
- **Idle tool options** (Inspect armed, either mode; `app.gd::_tool_options_generate()`):
  - label "GENERATE · WORLD"
  - [Generate world] (primary), [New seed], [Center landmasses] (tooltip `CENTRE_TOOLTIP`)
  - a busy label ("generating…")
  - spacer, then [Bake ALL & finalize]
- **Ring / fan diagonals** (`WorldWorkspace.ring_slots`):
  - **NW Uplift ▸** Mountains, Hills, Ridge, Plateau, Cliff / Escarpment, Volcano
  - **NE Carve ▸** Canyon, Valley, River, Lake, Basin, Coastline
  - **SE Freehand (F) ▸** Raise, Lower, Smooth, Cliff, Ridge, Canyon, Mesa, Volcano
  - **SW Paint (B) ▸** Biome, Terrain, Splat
  - All four are disabled with a reason when the world is finalized or absent.
- **Right dock in WORLD** (`right_dock.gd::_tool_section()`):
  - Sample by default.
  - **Stamp stack** appended while `sculpt` is armed or the draft has stamps.
  - **Paint · \<Layer\>** while `paint` is armed in WORLD.
- **Bottom bar:** none. The timeline is CIVIL-only.

### 3.1 Node "Generate" (mode `a`, pill PIPELINE)

#### Tools

The global tools. Biome paint is hidden in this mode.

#### Left dock: seven categories, in order

Each category opens with a one-line lead note, `CATEGORIES[i].lead`. Parameter rows are built from the engine's live table (`bridge.param_info`):

- A bool parameter becomes a **toggle**; an int or float becomes a **slider** with the engine's min, max, step and unit.
- Right-clicking a row resets it to the engine default; the tooltip says so.
- Keys in `ADVANCED_KEYS` go into a closed **"advanced"** fold at the end of their section.
- Every row's full range, default and unit is in Appendix A.

**1. Generate** (open by default)

- **§ Run**
  - [Generate world] (primary). Runs all ten stages.
  - [New seed]. Rolls the seed into File ▸ New world; does not regenerate.
  - [Center landmasses]. Whole-world mode only.
- **§ Import**
  - [Load heightmap…]. Opens World ▸ Import.
  - [Infer tectonics from heightmap…]. Opens the same import.
  - A note.
- **§ Pipeline status**
  - A stale-from note.
  - **Ten read-only stage rows:** number, state dot, name, state ("pending", "no world", "running…", "done N.NNs …").
    - 01 Planet · 02 Extent & scale · 03 World structure · 04 Tectonics · 05 Volcanism & impacts · 06 Erosion · 07 Hydrology · 08 Climate · 09 Ecology & biomes · 10 Resources & soils
  - A rolling stage log (12 lines).
  - Two notes.
- **§ Finalize**
  - A bake status note.
  - **Bake depth**, dropdown: "LOD 0–0 1 tile" … "LOD 0–8 87,381 tiles". The default comes from `DccSettings.bake_depth()`, which is 3; it shares the setting with Preferences ▸ Tiles & LOD.
  - [Bake ALL levels & finalize] (primary).
  - [Un-finalize].
  - [Clear this world's atlas].
- **§ LOD terrain data**: a note only.

**2. Planet** (stages 01–03; each stage gets its own numbered section)

- **§ 01 Planet**
  - needs / produces / gap notes
  - Gravity (slider)
  - Day length (slider)
  - Axial tilt (slider)
- **§ 02 Extent & scale**
  - notes
  - Whole world (wrap in longitude) (toggle)
  - Sea level (slider)
  - Peak altitude (slider)
  - a live sea-level note (`sea_level_live_note`)
- **§ 03 World structure**
  - notes
  - Enable continental steering (toggle)
  - Continentality, Fragmentation, Tectonic energy, Ocean depth, Hotspot density (sliders)

**3. Geology**

- **§ 04 Tectonics**
  - notes, then 11 rows:
    - Plates, Drift, Warp, Uplift spread, Tectonic α, Noise β, Erosion / age (sliders)
    - Ridged mountain detail (toggle)
    - Coastline leaves the plate outlines (v2.57 blur) (toggle)
    - Ridged mountain ranges in the orogenic belts (toggle)
    - Higher world ceiling (tall volcanoes keep their summits) (toggle)
  - **advanced** fold: Flexure F, Heterogeneity C, Rock resistance, Dynamic lithology (exhumation hardens rock), Lloyd relaxation passes
  - Plates and Drift dim, with the reason in the tooltip, while World structure is on (`WS_OVERRIDDEN_KEYS`).
- **§ 05 Volcanism & impacts**
  - notes, then 19 rows:
    - Volcanoes, Volcano age (sliders)
    - Provinces & arc/rift placement (toggle)
    - 13 model toggles:
      - Exclude transform margins
      - Shaped edifices (shield/strato/cone)
      - Drain volcano flank pits
      - Volcano morphometry (VA-5)
      - Scoria cones sized for the grid (Ruling CX)
      - Active craters stay dry (Ruling CX)
      - Young lava flows build benches at the sea (Ruling DB)
      - Lava domes on young stratocones (VA-6)
      - Maars beside scoria cones on wet ground (VA-6)
      - Old arc stratocones collapse into calderas (VA-6)
      - A supervolcano caldera where a hotspot sits under a continent (VA-6)
      - A flood-basalt plateau, drawn only (VA-6)
      - Physical crater model (map area + wear)
    - Craters, Crater age, Surface age (sliders)
  - Volcanoes dims while World structure is on.
- **06 Erosion · hillslope diffuse**: a closed expander holding a note only (generation-time toggle).

**4. Hydrology**

- **§ 06 Erosion · water & ice**
  - needs / produces / gap notes
  - **Stream-power carve** expander (open): all 29 `erosion`-group rows, in `PARAMS` order:
    - Geological age, with its readout note. Dimmed in the app because `geology_processes` is off.
    - Uplift, Channeling, Iterations, Deposition, Rain → erosion
    - Velocity (momentum) erosion + 3 dials
    - Glacial erosion + 5 dials
    - Coastal processes + 4 dials
    - Hillslope diffuse + 2 dials
    - Sediment fill (route + redeposit) + capacity
    - Evolve climate ↔ terrain cycles
    - Tidal flats (mudflat accretion) + rate
  - **Droplet hydraulic** expander (closed):
    - a note
    - 5 sliders (shell-side state, not engine parameters):
      - **Droplets** 0–150 000, step 1 500, default 60 000
      - **Strength** 0–1, step 0.01, default 0.35
      - **Deposition** 0–1, step 0.01, default 0.30
      - **Thermal** 0–30, step 1, default 8
      - **Slope limit** 0.001–0.040, step 0.001, default 0.012
    - [Erode (droplet)] (primary; disabled without the `erode_op` binding)
    - [Reset dials]
  - **Velocity (momentum)** expander (closed): a note only.
  - **Glacial** expander (closed): a note, a fjord note, and [Carve fjords].
  - **Coastal** expander (closed): a note only.
- Then stage 07's needs / produces / gap notes.
- **§ River network**
  - Carve river valleys on generation (toggle)
  - River density (slider)
  - Integrated drainage (route over filled depressions) (toggle)
- **§ River entities**: a note (`InfrastructureWorkspace.rivers_note()`).

**5. Climate** (stage 08)

- notes
- **§ Climate & temperature**
  - North edge, South edge, Equator temperature, Pole temperature, Lapse rate, Ice albedo (sliders)
  - Ocean currents (toggle)
  - **advanced:** Ocean-current strength, Terrain wind deflection
- **§ Weather · rainfall sim**
  - Iterations, Orographic, Evaporation, Dryness, Ocean supply (sliders)
  - Manual wind direction (toggle)
  - Wind direction, Pressure influence, Zonal belts (sliders)
  - Smooth (B-spline) rain up-sample (toggle)
  - **advanced:** Sea-surface humidity, Bulk aerodynamic evaporation

**6. Ecology** (stage 09)

- **§ 09 Ecology & biomes**: notes only (not parameterised).
- **§ Productivity**: a readout note, then [Show productivity on the map] (sets the `npp` field).
- **§ Fauna**: a readout, up to 8 ecoregion lines, then [Show fauna on the map] (sets the `wildlife` field).
- **§ Not parameterised**: notes.

**7. World data**

- **§ Read the fields**
  - [World data tables…]
  - [Export GeoJSON…] (opens Data manager ▸ Export)
- **§ Coordinate system**: readout notes, plus a "Reprojection · needs a decision" note.

#### Right dock

**Sample** context, with these rows:

- Position, Cell, Elevation (accent)
- the 17 `SAMPLE_FIELDS`:
  - Slope, Aspect
  - Plate + type, Boundary + distance, Resistance
  - Lithology, Rock (surface), Beneath, Strength, Soluble · permeable, Regolith
  - Temperature, Precipitation, Drainage
  - Biome, Soil, Control
- Nearest, Route cost, E–W profile

Other contexts appear on selection; see section 6.

#### Bottom bar

None.

#### Phone

- The **GENERATE** tab (lit at boot, `_phone_tab := "gen"`) fills the bottom sheet from `WorldWorkspace.build_phone_generate()`, not from the dock.
- Sheet header: "GENERATE", with subtitle "pipeline · seed N", "pipeline · no world yet" or "pipeline · generating…".
- **Column order in PIPELINE:**
  1. Mode segment: **PIPELINE | SCULPT | PAINT**. The phone adds PAINT.
  2. **SEED** row: the seed value and a ↻ dice button.
  3. If generating, a progress card (stage, %, log, a dashed CANCEL row because nothing backs it). Otherwise a [GENERATE WORLD] or [REGENERATE NN → 10] button with the stale note.
  4. **Ten collapsible stage groups**, 01–10. Each has number, name, state (resolved / stale / no world) and a chevron; 01 is open by default.
     - Inside each group: its parameters (same keys as the desktop, grouped **by stage**, not by Ruling L category), then an ADVANCED subheading.
     - Inside each group: dashed "not settable here" rows from `PHONE_GEN_ABSENT`:
       - 02: Working resolution → routes to File ▸ New world
       - 03: Archetype → routes to New world
       - 06: Erosion strength (no engine parameter)
       - 07: Min stream order (not settable)
       - 09: Ecotone sharpness (not a generation setting)
       - 09: Rivers in biome view (it is CARTO ▸ Layers ▸ Rivers)
  5. A footnote: "10 stages · dependency order".
- **Not in the phone sheet:**
  - Center landmasses, Import, Finalize / bake, Droplet erode, Carve fjords
  - the Ecology readouts, World data, Coordinate system

  I searched lines 4988–6560 (the `_pg_*` block) for `erode|fjord|Finalize|Bake|Import|Load heightmap|Center|New seed`; the only match is a rock-paint "re-erodes" note.
- These remain reachable on the phone only through the WORLD left dock as a sheet (MORE ▸ Window ▸ Left dock, while WORLD is active) and through the menus.

### 3.2 Node "Sculpt" (mode `b`, pill SCULPT)

#### Tools

- **Global tools, then Biome paint (B).**
- **The sculpt feature grid in the dock** (Terrain ▸ Geological feature) arms `sculpt`. Each button's glyph is the feature key; its tooltip reads "Label -- hint".
- **The 13 features** (`cartalith_terrain::sculpt::FEATURE_KEYS`, registry order) and their own controls, as min–max, step, default:

| # | Feature (label) | key | Controls |
|---|---|---|---|
| 1 | Mountains | `mountains` | Height 0.1–0.55, 0.01, 0.42 · Peak sharpness 0.6–3.0, 0.05, 1.5 · Ridge freq 0.6–5.0, 0.1, 1.6 · Ruggedness 0–1, 0.01, 0.55 |
| 2 | Hills | `hills` | Amplitude 0.02–0.3, 0.005, 0.11 · Rolling freq 0.5–4.0, 0.1, 1.4 · Softness 0–1, 0.01, 0.7 |
| 3 | Ridge | `ridge` | Height 0.02–0.35, 0.005, 0.15 · Width frac 0.1–0.6, 0.01, 0.28 · Detail freq 0.5–4.0, 0.1, 1.5 |
| 4 | Plateau | `plateau` | Rise 0.03–0.45, 0.005, 0.26 · Terraces 1–8, 1, 4 · Detail freq 0.4–3.0, 0.1, 1.1 |
| 5 | Cliff / Escarpment | `cliff` | Rise 0.05–0.45, 0.005, 0.22 · Steepness 0.2–1.0, 0.01, 0.75 |
| 6 | Canyon | `canyon` | Depth 0.03–0.35, 0.005, 0.18 · Wall steepness 0–1, 0.01, 0.7 · Meander 0–0.8, 0.01, 0.35 |
| 7 | Valley | `valley` | Depth 0.03–0.3, 0.005, 0.14 · Width frac 0.3–1.0, 0.02, 0.85 · Meander 0–0.8, 0.01, 0.3 |
| 8 | River | `river` | Width (px) 2–26, 1, 7 · Depth 0.02–0.22, 0.005, 0.09 · Meander 0–0.6, 0.01, 0.28 · Branch noise 0–1, 0.01, 0.5 |
| 9 | Lake (radial) | `lake` | Depth 0.03–0.3, 0.005, 0.13 · Shore 0.05–0.6, 0.01, 0.25 |
| 10 | Basin | `basin` | Depth 0.02–0.25, 0.005, 0.1 · Floor rough 0–1, 0.01, 0.4 |
| 11 | Coastline | `coastline` | Amount 0.1–1.0, 0.01, 0.85 · Raggedness 0.4–4.0, 0.1, 1.6 |
| 12 | Volcano (radial) | `volcano` | Cone height 0.15–0.6, 0.01, 0.45 · Crater depth 0–0.9, 0.01, 0.5 · Radius (px) 30–200, 2, 110 · Flank rough 0–1, 0.01, 0.6 |
| 13 | Freehand (**F**) | `freehand` | Amount 0.02–0.3, 0.005, 0.12. Sub-modes: Raise, Lower, Smooth, Cliff, Ridge, Canyon, Mesa, Volcano |

**Presets** (`SCULPT_PRESETS`, 8). A preset seeds a feature and its parameters; it never paints.

| Preset | Feature | Noise scale |
|---|---|---|
| Rolling Hills | Hills | 6 |
| Alps | Mountains | 5 |
| Rockies | Mountains | 4 |
| Badlands | Canyon | 9 |
| Volcanic Isle | Volcano | 5 |
| Mesa | Plateau | 4 |
| Karst | Hills | 15 |
| Glacial Valley | Valley | 4 |

**Brush and noise globals** (`sculpt_bridge::GLOBAL_RANGES`, defaults from `SculptGlobals::default()`):

| Global | Range | Step | Default |
|---|---|---|---|
| Brush size | 6–200 px | 1 | 32 |
| Hardness | 0–1 | 0.01 | 0.5 |
| Intensity | 0–1.5 | 0.01 | 1.0 |
| Noise scale | 1–20 | 0.5 | 5 |
| Octaves | 1–8 | 1 | 5 |
| Persistence | 0.20–0.90 | 0.01 | 0.5 |
| Lacunarity | 1.40–3.20 | 0.05 | 2.0 |
| Edge noise | 0–1 | 0.01 | 0.55 |
| Falloff (dropdown) | Smooth / Linear / Sharp / Constant | — | Smooth |

Grid snap is a dropdown: Off, 1 cell, 2 cells, 4 cells, 8 cells (`GRID_SNAP_STEPS`).

**Tool options while `sculpt` is armed** (unified bar, SCULPT mode):

- **Row 1:** mode segments, then the eight Freehand sub-mode segments (Raise … Volcano), then a note.
- **Row 2:**
  - "SCULPT · \<FEATURE\>"
  - Brush size, Intensity, Hardness sliders
  - an **Amount** slider, −1.0 to 1.0, when the feature has `amount` (see 9.2)
  - "N stamps", then [Commit] and [Discard] chips
- **Row 3, BRUSH:** Pressure toggle (→ strength), then Symmetry (see Paint).

**Tool options while `paint` is armed** (unified bar, PAINT mode):

- **Row 1:**
  - mode segments
  - layer segments Biome, Terrain, Splat, plus Rock (disabled with a reason when unavailable)
  - an Erase segment (⇧ while painting does the same)
  - a note
  - an **Original** slider, 0–100 %, step 5, on Biome and Terrain only (`paint_original.gd`). It overlays the generator's unpainted layer for comparison.
- **Row 2:**
  - "PAINT · \<LAYER\>"
  - Class (dropdown of the layer's palette)
  - Size 1–40 c
  - Land only (toggle, default on)
  - "N painted", then [Commit] and [Discard]
  - **Rock variant:** "PAINT · ROCK", then Layer (Surface / Beneath / Cap thickness), Thickness (cap, 10 m classes up to 2 550 m) or Rock (11 rocks), Size, Land only, "N in draft", [Commit], [Discard].
- **Row 3, BRUSH:**
  - Spacing (toggle), Gap 1–100 %, Pressure (toggle, → size), Scatter 1–16, Spread 0–3 ⌀
  - Symmetry dropdown: Off / Vertical / Horizontal / Dual / Radial
    - Copies 2–16 (Radial) or Angle −90° to 90°
    - [Pick centre] and [Reset centre]

**Paint palettes** (`PaintTarget::palette()`):

- **Biome:** the first 13 of `CART_BIOMES`, which are Coastal Lowland, Temperate Forest, Mediterranean Scrub, Wetlands / Marshes, Steppe / Grassland, Tropical Jungle, Boreal Taiga, Mountain Highland, Cold Desert / Badlands, Hot Desert, Tundra / Polar, Ruined Wastes, Hills.
- **Terrain** (13): Paved Road, Dirt Track, Desert Hardpack, Open Plains, Forest Path, Hills, Rocky Terrain, Mountain Pass, Mountain Trails, Swamp / Marsh, Deep Sand, Snow / Ice, Ruins / Debris.
- **Splat** (6): grass, rock, sand, snow, wetland, canopy.
- **Rock** (11, `geology::Rock`): Granite, Gneiss, Schist, Plateau basalt, Oceanic basalt, Andesite, Tuff, Limestone, Sandstone, Shale, Unconsolidated. Beneath adds "none — single layer".

#### Left dock: two categories

**8. Terrain** (the sculpt body; the dock's floor in SCULPT)

- Without a world, or for a loaded save with no draft session, this is a § Sculpt section with a note only.
- **§ Geological feature**: a 5-column grid of the 13 feature tool buttons, plus the current feature's hint note.
- **§ Presets**: 8 buttons and a note.
- **§ \<Feature\> parameters**: one slider per control of the current feature.
- **§ Freehand · direct drag** (only when the feature is Freehand): **Sub-mode** dropdown with the 8 modes, plus a note.
- **§ Brush & noise · global**:
  - 8 sliders and the Falloff dropdown
  - Grid snap dropdown
  - Seed readout ("—" when there is no seed) and a dice button
- **§ Not built**: a note (brush shape, stroke and grid control-point editing, and actions have no engine).
- **§ Draft**:
  - "N stamps on the draft."
  - **Commit** group: [✓ Commit to map] (primary) and [Discard draft], both disabled at 0 stamps.
  - a note
  - **Painted lakes** expander (closed): a note and [Count painted lakes as water].

**9. Biomes** (the paint body)

- Without a world: a § Biome paint section with a note.
- **§ Biome paint**:
  - a note
  - **Target field**: Biome, Terrain, Splat, Rock (Rock disabled with a reason when unavailable)
  - **Value**: the palette
  - **Radius** 1–40 cells, step 1, default 6
  - **Hardness** 0–1, step 0.01, default 1.0
  - **Softness** 0–1, step 0.01, default 0.0
  - **Erase** (toggle, default off)
  - **Land only** (toggle, default on)
  - **Legend · painted counts** group
  - **Commit** group: [✓ Commit] and [Discard draft], disabled when nothing is pending
  - a note
- **Rock target:**
  - **Rock layer**: Surface / Beneath / Cap thickness
  - Thickness slider (cap), or the Rock dropdown
  - Radius, Hardness, Softness
  - Erase, disabled: "Erase (⇧) is off for Rock …"
  - Land only
  - **Legend · painted in the draft**
  - Commit / Discard, then the last-commit and save notes

#### Right dock

- **Stamp stack**:
  - History group: Undo, Redo
  - Commit group: Commit to map, Discard draft
  - **Selected stamp**, with an Actions group: Deselect, Show, Move up, …
- **Paint · \<Layer\>**: Legend · painted counts, plus a Commit group with Commit and Discard draft. For Rock: **Paint · Rock · \<layer\>**.
- Ruling L calls the right-dock paint copy a duplicate to retire. It is still built; `_refresh_right_dock_paint()` keeps it in step.

#### Bottom bar

None.

#### Phone

- **SCULPT segment** (header subtitle "sculpt · draft stamps"):
  - GEOLOGICAL FEATURE chips (13)
  - PRESETS chips (8)
  - BRUSH · GLOBAL: one slider per global, **including Falloff and Octaves**. The phone slider helper has no enum branch, so Falloff draws as a numeric 0–3 slider (9.3).
  - BRUSH · DYNAMICS: PRESSURE → STRENGTH chip, then SYMMETRY chips, Copies or Mirror angle, PICK CENTRE and RESET CENTRE
  - [＋ ARM SCULPT · DRAW ON THE MAP]
  - "N stamps on the draft", then DISCARD and ✓ COMMIT
  - **No per-feature parameters, no Freehand sub-mode, no Grid snap, and no Seed on the phone.** I searched `_pg_sculpt` for `PARAMETERS|feature_params|FREEHAND|grid_snap|SEED` and found none.
- **PAINT segment** (subtitle "paint · draft cells"):
  - TARGET FIELD chips plus Rock
  - ROCK LAYER
  - BRUSH: Size, Hardness, Softness, and Original (Biome and Terrain)
  - ERASE and LAND ONLY chips
  - CAP THICKNESS, then CLASS or ROCK chips
  - BRUSH · DYNAMICS: SPACING, PRESSURE → SIZE, Gap between dabs, Scatter copies, Scatter spread, then symmetry
  - [＋ ARM PAINT · DRAW ON THE MAP]
  - DISCARD and ✓ COMMIT
- Picking SCULPT or PAINT writes WORLD mode `b`; PIPELINE writes `a` (`_pg_set_mode`).

---

## 4. CIVIL

### 4.0 Domain-level

**Domain tools** (palette after the divider; `CivilizationWorkspace._build_tools()`):

| Label | id | Glyph | Key | What it does | Tool options bar |
|---|---|---|---|---|---|
| Settlement (S) | `settlement` | `tool_settlement` | S | Click drops a settlement | "CIVIL · SETTLEMENT": **Class** (Metropolis, Capital, City, Town, Village, Hamlet; default Town) · **Faction** ("id · Culture") · **Name** (field, "(blank = generated)") · **Snap to water** (toggle, default off) |
| Territory (T) | `territory` | `tool_territory` | T | Drag to paint a faction's claim | "CIVIL · TERRITORY": **Faction** · **Radius** 1–20 c (default 5) · **Mode**: Add / Subtract (⇧) · faction stats |
| Territory lasso (⇧T) | `territory_lasso` | `tool_lasso` | ⇧T | Click a ring; commit assigns its inside | "CIVIL · TERRITORY LASSO": **Faction** · **Mode**: Add / Subtract · vertex count and hint · [✓ Commit] · [Discard] |
| Way (W) | `way` | `tool_way` | W | Draw a road or track by waypoints or drag | "INFRA · WAY": **Type** (Road, Track, Sea lane, Ancient) · waypoint count · [✓ Commit] · [Discard]. Second row, ROAD DRAG: **Mode** (Routed / Freehand), **Smooth** (toggle), [Advanced ▸] showing **Waypoints** 1–8, **Corridor** 1–16 c, **Pull** 0–20, then the Symmetry controls |
| Route (⇧R) | `route` | `tool_route` | ⇧R | Sea and river legs between ports | "INFRA · ROUTE": stop count · [✓ Commit] · [Discard] |
| Conflict | `conflict` | `tool_conflict` | (none) | Draw a front or arrow, or place a siege or battle | "CIVIL · CONFLICT": **Kind** (Front, Arrow, Siege, Battle; default Front) · point count · [✓ Commit] · [Discard] |

- **Idle tool options** (Inspect): "CIVIL · INSPECT", plus "Settlement, Territory, Way and Route tools are armed from the tool bar above. Points of interest cannot be placed yet."
- **No POI tool.** The code omits it on purpose: no engine function drops one (`_build_tools()` comment). The phone's MORE ▸ Civilization lists "Point of interest" as a missing row.
- **Journey** (`journey`) is armed by the Journey planner node, by ⇧J, or by Travel ▸ Open Journey Planner.
- **Ring / fan diagonals:**
  - **NW Settle (S) ▸** one row per class
  - **NE Territory (T) ▸** Add, Subtract
  - **SE Way (W) ▸** one row per type
  - **SW Route (⇧R)**

**The 13 categories, in dock order** (`CivilizationWorkspace._build()`). Default-open floor: Populate. Owning node in brackets.

1. Populate [Factions]
2. Settlements [Factions]
3. Landmarks [Landmarks]
4. Routes & ways [Routes & ways]
5. Travel [Journey planner]
6. Factions [Factions]
7. Territories [Factions]
8. Relationships [Factions]
9. Military [Factions]
10. Culture [Factions]
11. Religion [Factions]
12. Economy [Factions]
13. Timeline [Factions]

**Right dock in CIVIL** (section 6):

- Sample by default.
- **Settlement** on a settlement click.
- **Faction**, **Route**, **River**, **Landmark** or **Conflict** on selection.
- **Territory** appended while Territory or Lasso is armed.
- **Way draft** appended while a Way or Route draft exists.
- **Journey results** appended while the planner is up.

**Bottom bar:** the timeline (4.5). It is visible whenever CIVIL is active, for every CIVIL node.

### 4.1 Node "Landmarks"

#### Left dock: 3. Landmarks

**§ Placement**

- headroom note
- **Crowding** slider, 0.25–2.00, step 0.05, unit ×
- crowding note
- **Types compete with each other** (toggle, default on), plus a note
- **advanced** fold (closed):
  - a note
  - **Continental**, **Regional**, **Local**, **Cultural** exclusion radius sliders, 1–400 km, step 1
  - [Reset every cap and radius to its default]

**§ Types**

- notes
- a chip row: **all**, plus one chip per class badge with a count (Continental, Regional, Local, Cultural)
- **one expander per family**, the first open:
  - Physical
  - Transportation
  - Economic
  - Military
  - Religious · cultural
  - Historical
- Each family has **arm all** and **off** chips.
- Each **type row** is a slider over the 1-2-3-5 ladder (0, 1, 2, 3, 5, 8, 12, 20, 30, 50, 80, 120, 200), with an icon, a class badge, an optional [viewshed] tag, and a reason-token button that opens the funnel popover.

**§ Last run**

- stale note
- [Run landmark pass] (primary)
- run note
- **placed by hand** expander (closed): [Place an icon → Cartography ▸ Icons]

**§ Not wired** (only if the bridge reports no kinds): notes and the same icon jump.

**The 49 landmark types** (`cartalith_civ::landmark::kinds()`). Format: label (class, default cap). "vs" marks needs-viewshed; "not built" means the generator does not place it and the row says why.

- **Physical (15):**
  - Peak (Regional, 24, vs)
  - Ridge (Regional, 20)
  - Saddle (Local, 16, not built)
  - Cliff (Local, 30)
  - Gorge (Regional, 8)
  - Cave (Local, 12)
  - Waterfall (Regional, 40)
  - Spring (Local, 30)
  - Lake (Regional, 16)
  - Delta (Regional, 6, not built)
  - River confluence (Local, 20)
  - Volcanic feature (Regional, 10, vs)
  - Rock formation (Local, 20)
  - Glacial feature (Regional, 10, not built)
  - Ancient forest (Continental, 8, not built)
- **Transportation (8):**
  - Mountain pass (Regional, 12)
  - River crossing (Local, 20, not built)
  - Ford (Local, 24)
  - Bridge site (Local, 12)
  - Road junction (Local, 20)
  - Caravan station (Local, 10)
  - Portage (Local, 8)
  - Harbour (Regional, 16)
- **Economic (6):**
  - Mine (Local, 24)
  - Quarry (Local, 20)
  - Salt works (Local, 8, not built)
  - Resource extraction site (Local, 16)
  - Market site (Local, 12)
  - Trade depot (Local, 10)
- **Military (6):**
  - Fort (Regional, 16, vs)
  - Watchtower (Local, 20, vs)
  - Fortified pass (Regional, 8, vs)
  - Fortified crossing (Local, 8, vs)
  - Battlefield (Cultural, 12)
  - Border marker (Cultural, 16, vs)
- **Religious · cultural (8), all not built:**
  - Shrine (Local, 30)
  - Temple (Cultural, 16)
  - Sacred grove (Cultural, 12)
  - Sacred mountain (Cultural, 6)
  - Pilgrimage site (Cultural, 8)
  - Tomb (Cultural, 12)
  - Monument (Cultural, 12)
  - Ceremonial site (Cultural, 10)
- **Historical (6):**
  - Ruin (Regional, 20, not built)
  - Abandoned settlement (Local, 12)
  - Ancient road (Regional, 8)
  - Historic battlefield (Cultural, 8, not built)
  - Destroyed fortress (Regional, 8, not built)
  - Historic crossing (Local, 8, not built)

#### Right dock

**Landmark** context on a landmark click.

#### Phone

MORE ▸ Civilization ▸ **Landmark generation** pushes the "landmarks" screen (title "Landmarks", subtitle "caps · spacing · one run"):

- headroom info
- **Crowding** slider row
- the class-radius line
- **Types compete with each other**
- **Families** list, each pushing an "lm-fam" screen of type rows on the same ladder

### 4.2 Node "Factions" (it owns 10 of the 13 categories)

**1. Populate** (default-open floor)

- **§ How people get placed**: a note.
- [Placement model → File ▸ New world…]
- **§ Populate**
  - [Auto-populate world]
  - [Clear places & routes]

**2. Settlements**

- **§ Recompute**
  - staleness badge
  - [Recompute civilisation]
  - a note
- **§ Roster**
  - [Compute regional population…], only when a regional total exists
  - **Largest, by population** group: per settlement, a button and ✎ (opens the Place Editor)
- **§ Diagnostics**: an "N of M settlements" group, with per-settlement buttons and fact notes.
- **§ Totals**: a **By faction** group with per-faction buttons.
- **§ Coastal settlements**: a **Coastal settlements** group (`InfrastructureWorkspace._fill_ports`).
- **§ Linked notes**: [Markdown vault…]

**6. Factions**

- **§ Roster**
  - [Open factions…] (primary; the Factions window)
  - [Settlement types…]
  - **By province count** group: per-faction buttons that open the roster on the territory tab
- That is the whole category. The owner tree's "Identity colour", "Linked notes" and "Not built" are not in `_fill_factions()`, which builds only `Roster`.

**7. Territories**

- **§ Recompute**
  - [Recalculate territories]
  - [Generate provinces]
  - a note on the Territory brush
  - [Clear territory] (not undoable)
- **§ Borders & influence**: [Analyse contested borders], then **By faction** and **Contested borders** groups.
- **§ Linked notes**
  - **Provinces** group: per-province buttons
  - **Continents** group: per-continent buttons
- **§ Not built**: a note on occupation history.

**8. Relationships**

- **§ Standing**: an **Every pair** group, with one button per pair, "A ↔ B -- standing (±n)".
- **§ Not built**

**9. Military**

- **§ Conflicts**: one button per recorded conflict.
- **§ Faction strength**: a **By military power** group, "Faction -- n/100 · k of m fortified".
- **§ Manpower** groups:
  - **Standing · field · emergency**, with per-faction buttons
  - **How long each can stay out**
  - **What drives it**
  - **Who the bands are measured against**
- **§ Garrisons**
  - a note
  - **Border-exposure weight** slider, 0–5, step 0.1, unit ×. Writes `civ.garrison_exposure_scale`; default 1.0.
  - per-faction groups with per-place buttons
- **§ Fortifications**: a **Strongest places** group.
- **§ Campaigns**: a note pointing at CARTO ▸ Conflict.
- **§ Not built**

**10. Culture**

- **§ Profiles**: per-culture rows.
- [Which faction has which culture → Factions ▸ Identity]

**11. Religion**

- The header shows a faith count, **hidden on the phone** (`civilization_workspace.gd:3628`).
- A 3-way segment: **ADHERENCE | DIFFUSION | DIVERGENCE**, default ADHERENCE. One pane shows at a time.
- **Adherence pane**: totals and share strip, a table, and settlement lists (with, and without, a population).
- **Diffusion pane**:
  - a note
  - **Years** number field, 1–2000, step 1, default 50
  - [Run diffusion]
  - [Set a faction's religion → Faction roster…]
  - "Run diffusion → N year(s)" or "→ N more year(s)"
- **Divergence pane**: a diverged list, plus a show-on-map toggle.

**12. Economy**

- **§ Trade balance**: a readout.
- **§ Trade flows**: [Match trade flows], then these groups:
  - By good
  - Busiest partners
  - Needs nothing can reach
  - Way load
  - Caravans
- **By faction** expander: a **Territory, food and resources** group.
- **§ Not built**: a note on tariffs and time.

**13. Timeline**

- Without a world, the whole body is the note "Generate a world first."
- **§ Years**
  - a year field (−9999 to 9999, step 1)
  - [Add year]
  - per recorded year: a year button and ✕
- **§ Scrub** (only with 2 or more recorded years): **Year** slider from the first to the last recorded year.
- **§ Playback** (only with 2 or more years): [▶ Animate] or [⏸ Stop], and [Step].
- **§ Filters**: three toggles, all default off:
  - Exist only
  - Ghost removed
  - Highlight new
- **Simulate collapse / recovery** expander (closed):
  - **Mode**: Collapse (decline + migration) / Recovery (regrowth)
  - Collapse only:
    - **Character**: Mixed (default) / Trade -- hubs fall hardest / Disease -- dense/connected fall first / Conflict -- undefended fall first
    - **Severity** 0–100 %, default 50
  - Recovery only: **Regrowth rate** 0.1–3.0 %/yr, default 1.0
  - **Start year**: number, −9999 to 9999, default 0
  - **Duration (yr)**: 1 to 1 000 000, default 100
  - **Step years**: 1 to 1 000 000, default 10
  - [Simulate] (primary), plus a note
- **§ Not built**

#### Right dock

| Context | Contents |
|---|---|
| **Settlement** | Sections, plus an Actions group: Economy, Politics, Logistics |
| **Faction** | An Actions group with [Open in Factions…], plus a Relations section |
| **History** | Open draft, Steps, Cost |
| **Conflict** | Conflict, Manpower by side, Besieged |
| **Ecoregion** | Wildlife; opened from the Wildlife field |

#### Phone

- There is no CIVIL tab. MORE ▸ **Civilization** shows:
  - an info line
  - the six `CIV_TOOLS` rows (Settlement, Territory, Territory lasso, Way, Route, Conflict), each showing ARMED or a chevron
  - a missing "Point of interest" row
  - "Landmark generation"
  - "Open journey planner"
  - **"Civilization dock"**, which calls `select_domain("civilization")` and opens the **left sheet**: the full 13-category accordion, with the TOOLS block drawn in the dock on the phone
- **The selected settlement opens the right sheet** (`_show_on_phone()` on a pick).

### 4.3 Node "Routes & ways"

#### Left dock: 4. Routes & ways

- **§ Network**
  - [Generate roads]
  - [Clear ways & journeys]
  - a count note ("N ways -- …")
  - **Longest, by point count** group: up to 6 road buttons, each opening the Route context
- **Sea lanes** expander (only if sea routes exist): up to 6 lane buttons.
- **§ Hand-drawn**
  - **Committed this session** group
  - **Routes committed this session** group: per route, ● select, a name field, and × delete
- **§ Where the style lives**: a note pointing to Cartography ▸ Feature style ▸ Ways.

#### Tools

Way (W) and Route (⇧R) do the work here. Their options are in 4.0.

#### Right dock

- **Route** on a route click.
- **Way draft** while drawing.
- **River** on a river pick.

#### Phone

Same routes as 4.2: MORE ▸ Civilization tools, or the Civilization dock sheet. The Way tool's options row wraps (`HFlowContainer` on the phone).

### 4.4 Node "Journey planner"

#### Left dock: 5. Travel

This is what the dock shows before the takeover, and when Travel is opened another way.

- **§ Journey planning**
  - count note and planner note
  - **Journey Planner** group: [Open Journey Planner] (primary)
  - **Routes** group: per-route buttons
- **§ Travel library**: [Travel library… (⇧L)]

#### The takeover

Pressing the node also arms `journey`, which **swaps the CIVIL region** (`journey_planner_view.gd`).

**The left dock becomes the planner panel:**

- **§ Journeys**: the saved list.
- **§ Party · Traveler**
  - Group size (number, 1–100 000, "people")
  - Pace (dropdown)
  - Hours/day (land) 1–16, step 0.5
  - Trade cargo (kg) 0–500 000
  - Supplies carried (d) 1–90
  - Carry food (off = live off the land)
  - Grazing, Foraging (dropdowns)
- **§ Season & weather**
  - Season, Weather, Rest days (dropdowns)
  - Season advances during the journey
- **§ Carriage**
  - Transport (dropdown)
  - a mount or vessel field
  - Re-pack per stage where it pays
  - Auto-promote Walking → Baggage Train if overloaded
  - animal definitions
- **§ Route conditions**
  - Road quality
  - Infrastructure
  - Desert water
  - Respect seasonal closures (winter passes)

**The rest of the takeover:**

- **The map region becomes the stage spine on desktop and tablet.** On the phone the map stays live underneath.
- **Tool options** read "JOURNEY PLANNER":
  - **journey** dropdown (Route #n)
  - preset controls
  - "carriage: auto/manual"
  - [re-route for \<transport\>…]
  - a trim hint
  - [save journey], [export table]
- **The right dock** appends **Journey results**:
  - Route totals, Time, Load, Supply reach, Resupply stops, Cost, Vessels · water legs
  - vessel reference and vessels-on-route groups
  - Calculation trace
- **The timeline row** is handed to the planner's band while Journey is up (JP-13).

#### Phone

The **PLAN** tab (➔) calls `open_journey_planner()`:

- The planner opens with its left panel as the left sheet, and the map stays live.
- Sheet header: "PLAN", or "PLAN · STAGE n", with subtitle "journey · from → to" once two stops resolve.
- MORE ▸ Civilization ▸ Open journey planner does the same.

### 4.5 Timeline bar (CIVIL only; `app.gd::_fill_timeline_strip()`)

**Collapsed** (the default; `_tl_expanded := false`): a whole-strip button reading "TIMELINE · \<year\> · \<status\> · ▴ expand".

**Expanded:**

- **Row 1, transport:**
  - Play/pause, ◀ step back, ▶ step forward
  - speed segments **×1 / ×10 / ×100**, default ×10 (`TL_SPEEDS`, `tl_speed := 10`)
  - a state label and an end-of-track hint
  - six layer pills (`TL_LAYERS`), with their default states:
    - Climate (on)
    - Population (on)
    - Economy (off)
    - Politics (on)
    - Infrastructure (off)
    - Warfare (off)
  - a collapse button (⌄)
- **Row 2:** the scrub track over **year −400 … 1200** (`TL_YEAR_MIN/MAX`).
- **Row 3:**
  - "YEAR −400"
  - a readout: year, recorded state, and counts "N present / +n / −n / since Y"
  - "YEAR 1200"
- **Row 4:** DATE, a day-in-year slider (365-day calendar).
- **Row 5:** the note "Simulation layers — they record which layer you want; no layer renders yet."

**Phone:**

- The timeline row is one button, "TIMELINE · \<year\>", which toggles a **sim strip** above the nav bar: play, year scrub, and ×1/×10/×100.
- MORE ▸ **Simulation** gives:
  - Transport strip on map (switch)
  - Year slider
  - Speed chips
  - Playback
  - Simulation layers (six switches)
  - Simulation model

### 4.6 CIVIL on the phone, in one place

| Need | Phone route today |
|---|---|
| Arm a CIVIL tool | MORE ▸ Civilization ▸ tool row; the thumb fan's CIVIL diagonals; or the Civilization dock sheet's TOOLS block |
| Any CIVIL category | MORE ▸ Civilization ▸ Civilization dock (left sheet), or MORE ▸ Window ▸ Left dock while CIVIL is active |
| Landmarks dials | MORE ▸ Civilization ▸ Landmark generation (a bespoke screen) or the dock sheet |
| Journey planner | **PLAN** tab |
| Timeline | The TIMELINE row and sim strip, or MORE ▸ Simulation |
| Inspector | The right sheet opens on a settlement, landmark, icon or conflict pick |
| Rail nodes | **None.** The node list does not exist on the phone |

---

## 5. CARTO

### 5.0 Domain-level

**Domain tools:**

| Label | id | Glyph | Key | What it does | Tool options bar |
|---|---|---|---|---|---|
| Icon (I) | `icon` | `tool_icon` | I | Click stamps the armed icon; with Brush on, a drag paints a stand | "CARTO · ICON". Without a pack, only a note: "no asset pack loaded — File ▸ Import asset pack to place icons". Otherwise: **Family** (Settlement / Feature / POI) · **Variant** (the slots) · **Scale** 0.2–4.0, step 0.1, × (default 1.0) · **Rotation** −180° to 180° (0) · **Jitter** 0–1, step 0.05 (0) · **Brush** (toggle, off) · **Radius** 2–60 cells (12) · **Density** 0.05–2.0, step 0.05 (0.6) |
| Label (L) | `label` | `tool_label` | L | Click empty ground to create a label, or a label to edit it | "CARTO · LABEL": a hint; while editing, "editing #n" plus the text, then [Confirm] and [Cancel] |

- **The colour paint tool** (`colour_paint`, no palette cell) is armed from Colours ▸ Paint colour ▸ [Paint on the map].
- **Idle tool options:** "CARTOGRAPHY · \<MODE\>" (for example CARTOGRAPHY · STYLE), then "Presentation only — nothing here marks a generation stage stale. Map view, Map style and Rendering-advanced apply live; the quality tier they start from is set in Preferences."
- **Ring / fan diagonals:**
  - **NW Label (L)**
  - **NE Icon (I) ▸** one row per icon family, plus "Custom"
  - **SE View (1-8) ▸** the first 8 available analysis fields
  - **SW Style ▸** the first 6 `STYLE_PRESETS`, plus "All styles…"
- **Right dock in CARTO:**
  - With Inspect armed: **Ramp · stops**, which holds:
    - Blend
    - Stops group
    - Edit group: + add, reverse
    - Actions group: Compare
    - Selected stop
  - With Label or Icon armed: **Annotation · placed this session** (with Clear all), plus **Selected label** (✕ delete label).
  - On an icon pick: the **Icon** context.
  - **Always in CARTO:** an appended **Layers** section. It repeats the terrain raster's three rows (Terrain, Colour relief, Hillshade) with a visibility dot and order, plus [Layer properties...].
- **Bottom bar:** none.

### 5.1 Node "Style" (it owns Style, Relief & light, Colours, Feature style)

**1. Style** (default open)

- **§ Look**
  - **18 style preset tiles**, each with **Dark / Light / Pastel** variant chips (5.5).
  - **Base look** dropdown: Quality tier / Natural Vibrant / Antique Parchment / Satellite relief.
- **§ Painter styles**
  - a note
  - **10 sliders, each 0–1, step 0.01** (`STYLES`):
    - Watercolor
    - Contour veins
    - Ink linework
    - Hachure
    - Posterize
    - Engraving
    - Stipple
    - Sepia
    - Risograph
    - Pointillism
  - **Tanaka lighting**, 0–1
  - **contour interval** fold: **Interval**, 0–50 m, step 5. 0 means automatic.
- **§ Water**
  - **Coastal wave lines** (toggle)
  - **Wave reach**, 0.25–3, step 0.05, ×
  - **Animate water** (toggle; screen-only)
  - **Village map** (toggle)
- **§ Saved looks**
  - "Name this look" field
  - [Save look] (primary)
  - **Saved** dropdown
  - [Load look]
  - notes
- **§ Still owed**: notes.

**2. Relief & light**

- **§ Map view**
  - **Relief exaggeration** 0–12, step 0.1, x
  - **Sun azimuth** 0–360, step 5, deg
  - **Sun elevation** 5–85, step 1, deg
  - **Multi-sun lighting** (toggle)
- **§ Rendering - advanced**: six closed expanders (`APPEARANCE_GROUPS[0:6]`). Every slider's range is the engine tunable's own.
  - **Relief & light:**

    | Slider | Range (unit) |
    |---|---|
    | Light directions | 1–12 |
    | Directionality | 0–1 |
    | Ambient floor | 0–1 |
    | Light gain | 0–2 |
    | Chroma-preserving light | 0–1 |
    | Ambient occlusion | 0–1 |
    | AO radius | 0–5 % |
    | Sky view factor | 0–1 |
    | Cast shadows | 0–1 |
    | Ridge crests | 0–1 |
    | Curvature shading | 0–1 |
    | Ridged relief | 0–1 |
    | Toon light bands | 0–1 |
    | Toon outline | 0–1 |

  - **The sheet:**

    | Slider | Range (unit) |
    |---|---|
    | Parchment | 0–1 |
    | Paper grain | 0–20 % |
    | Paper mottle | 0–20 % |
    | Paper wash | 0–0.6 |
    | Forest stipple | 0–1 |
    | Forest canopy texture | 0–1 |
    | Plate border | 0–6 % |

  - **Materials:**

    | Slider | Range |
    |---|---|
    | Biome saturation | −1 to 1 |
    | Surface texture | 0–1 |
    | Slope rock | 0–1 |
    | Geology tint | 0–1 |
    | Bedrock exposure | 0–1 |
    | Rock microtexture | 0–1 |
    | Wetness (near channels) | 0–1 |
    | Wet ground (TWI) | 0–1 |
    | Coast bands (SDF) | 0–1 |
    | River bands (SDF) | 0–1 |
    | Biome blend (SDF) | 0–1 |
    | Local contrast | 0–1 |
    | Texture strength | 0–1 |
    | Ocean grain warp | 0–1 |
    | Land grain warp | 0–1 |

  - **Ice & snow:** Glacier ice, 0–1.
  - **Atmosphere:**

    | Slider | Range |
    |---|---|
    | Atmospheric haze | 0–0.6 |
    | Distance desaturation | 0–1 |
    | Distance contrast loss | 0–1 |

  - **Multi-scale detail:** Macro detail, Meso detail, Micro detail, each 0–1.
  - Then [Reset to quality tier] and two notes.

**3. Colours**

- **§ Colour relief**
  - **Colour relief** slider (`ramp_strength`, 0–1)
  - **Ramp** dropdown: 13 presets plus Custom (5.5)
  - **Blend**: Linear / Ease / Step
  - the ramp bar: drag stops; per-stop colour, position, alpha and delete
  - [Add stop], [Reverse]
  - notes
- **§ Colour grade**: three closed expanders (`APPEARANCE_GROUPS[6:]`).
  - **Rivers:**

    | Slider | Range (unit) |
    |---|---|
    | River width | 0.25–3, x |
    | River opacity | 0–1 |
    | River ink | 0–1 |
    | River ink red | 0–255 |
    | River ink green | 0–255 |
    | River ink blue | 0–255 |
    | River under styles | 0–1 |

  - **Colour grade:** Exposure, Gamma, Contrast, Saturation, Temperature, Shadow tint, Highlight tint, each −1 to 1.
  - **Grade field influence:** Biome influence, Elevation influence, Moisture influence, Geology influence, each −1 to 1.
- **§ Biome colours**
  - **Relief <-> biome** slider (`bio_blend`, 0–1)
  - [Show the Biomes field (N classes) → Layers ▸ Biomes]
  - per class: a colour swatch (confirmable picker) and a Reset link
  - [Reset all biome colours]
- **§ Look** (quick look over the adjustments)
  - **Apply to**: All groups or a layer
  - **Hue** −360° to 360°
  - **Saturation** −100 % to 100 %
  - **Brightness** −100 % to 100 %
- **§ Adjustments** (`adjust_panel.gd`)
  - **Layer** dropdown: Terrain, Water, Labels & overlays
  - a per-entry card list with: toggle, Up, Down, Delete, Opacity 0–1, and kind-specific sliders
    - Levels: In black, In white, Gamma, Out black, Out white
    - Curves: Point n in and out, Add point, Remove point
    - Hue / Sat / Light: Hue, Saturation, Lightness
    - Gradient map: Blend dropdown and stops
  - add buttons: + Levels, + Curves, + Hue / Sat / Light, + Gradient map
  - [Reset layer]
- **§ Colour management**: **Display** dropdown (sRGB / Display P3), plus notes.
- **§ Paint colour** (`colour_paint_tool.gd`)
  - [Paint on the map] (primary; arms `colour_paint`)
  - **Material**: Colour / Lava / Embers / Ice glow / Toxic glow
  - a colour swatch
  - **Blend**: Colour (tint) / Replace / Multiply (default Colour)
  - **Radius** 0.05–25 km, step 0.05 (1.0)
  - **Hardness** 0–1, step 0.05 (0.8)
  - **Opacity** 0–1, step 0.05 (1.0)
  - **Land only** (toggle, on)
  - material-only sliders:
    - **Heat** 0–1 (0.75)
    - **Crack density** 0–1 (0.5)
    - **Glow radius** 0–10 km (1.5)
    - **Cool along stroke** 0–1 (0)
  - the layers list
  - [Undo], [Redo]
  - **Hide colour paint** (toggle)
  - [Clear colour paint]

**4. Feature style**

- **§ Ways**
  - **Line width** 0.2–2.5, step 0.05, ×
  - **Opacity** 0–1, step 0.01
  - **Drop minor ways when zoomed out** (toggle)
  - **Thicken ways by carried volume** (toggle)
  - [Draw and edit ways → Civilization ▸ Routes & ways]
- **§ Territories**
  - **Fill opacity** 0–1, step 0.01, with a [Reset to x.xx] link
  - [Faction identity colours → Factions ▸ Identity]
  - [Edit territories → Civilization ▸ Territories]
- **§ Not built**: [Claim hatching and the influence ramp → Layers ▸ Civilization ▸ Contested borders]

**Engine appearance tunables with no dock slider.** They appear only as values inside style presets, or nowhere in the shell:

- Scale-aware detail, Zoom detail (km), Volcano flank furrows
- Bathymetric sea ramp, Coast shelf and halo, Shelf width (km)
- Snow sheds on steep ground, Snow shed slope (deg)
- Slope rock knee (deg), Slope rock knee below treeline (deg)
- Volcano colours, Lava flows
- Forest canopy coarsest scale (km)
- Land relief contrast, Land tone contrast
- River bank outline

Method: I grepped each key quoted in `shell/*.gd`. Only `sea_ramp_strength` (1 hit) and `river_bank` (8 hits) appear, and all of those hits are inside `STYLE_PRESETS` or the shader.

### 5.2 Node "Layers"

**5. Layers**

- **§ Visible layers**: 8 toggles, in order:
  1. Settlements
  2. Ways & routes
  3. Sea routes
  4. Town layouts (deep zoom)
  5. Landmarks
  6. Landmark rejects (diagnostic)
  7. Conflict
  8. Rivers

  A note follows.
- **Settlements · by class** expander (closed): 6 toggles, all on, labelled by code as **Metropoliss, Capitals, Citys, Towns, Villages, Hamlets** (`String(kind).capitalize() + "s"`; see 9.1). A note follows.
- **Landmarks · by type** expander (closed; `LandmarkFilterPanel`):
  - [Show all], [Hide all]
  - one toggle per landmark type, with its count
  - **Scale with zoom** (toggle)
  - a note
- **§ Ways · by type**: 5 toggles, all on (Trade highways, Regional roads, Roads, Tracks, Ancient routes), then a note.
- **§ Political layers**: **Political — provinces**, **Political — territory**, then a note.
- **§ Terrain raster**: three rows, top drawn last: **Terrain**, **Colour relief**, **Hillshade**. Each row has:
  - a visibility dot
  - name, Up, Down
  - **Opacity** 0–1
  - **Blend**: Normal / Multiply / Screen / Overlay / Add

  Notes follow.
- **§ Data overlays**: a note ("N analysis fields across N groups …") and [Data overlays…] (primary), which opens the map's Layers popover; hotkeys 1–8.
- **§ Not built** and **§ Partly built**: notes.

### 5.3 Node "Labels"

**6. Labels**

- **§ Label classes**
  - a "Generate labels" cue, shown only when the labels are stale: a note and [Generate labels]
  - **Classes** group (open): one row per class, with a swatch, the name, a spec line "size/halo · tracking em [italic]", and drawn and placed counts
  - **Type** group (open):
    - **Class** dropdown
    - **size** slider 8–34
    - **halo** 0–4
    - **tracking** 0–0.40 em
    - **collision culling** (toggle)
    - a note
  - **Role defaults** group (open):
    - **Size mode**: Fixed / Zoom with map
    - [Apply to this role's labels]
    - **Font family**, **Weight**, **Case**: dead text rows, with no engine
- **§ Region labels**
  - per-label rows: edit and delete
  - [Clear all labels]
  - **the edit form** (while a label is selected):
    - Text (field)
    - Class
    - Size 8–48 px
    - Size mode (Fixed / Zoom with map)
    - Arc −1 to 1
    - Angle −180° to 180°
    - Font: Default / IBM Plex Mono / Fira Code
    - Colour (picker)
    - [Confirm] (primary), [Cancel], [Delete]

**Label classes** (engine `LABEL_CLASSES` with `LABEL_TYPOGRAPHY_DEFAULTS`):

| Class | Size | Halo | Tracking | Italic | Ink |
|---|---|---|---|---|---|
| Continental | 26 | 2.5 | 0.28 | no | #e0a34a |
| Region | 18 | 2.0 | 0.20 | no | #c8cbcd |
| Settlement | 13 | 1.5 | 0.06 | no | #a9adb0 |
| Water | 15 | 1.5 | 0.14 | yes | #6f9fb5 |
| Landmark | 11 | 1.2 | 0.06 | no | #8d9296 |

### 5.4 Node "Icons"

**7. Icons**

- **§ Automatic placement**
  - a note
  - **Family** group (open): chips per engine family: **PLACES**, **TREES**, **SEA MARKS**, **POI** (`PlacementFamily`)
  - **icon scale** slider 0.5–2.0, step 0.01, × (default 1.0)
  - **min spacing** slider 0–40 cells (default 14)
  - **Placement rules** group (open):
    - **avoid label boxes** (on)
    - **enforce min spacing** (on)
    - **snap sea marks to coast** (off)
  - [Place this family]
  - Without the binding these are drawn inert, with a note.
- **§ Placed icons**: per-icon rows with delete, then [Clear all icons].

**Icon families for the Icon tool** (`ICON_FAMILIES`):

- **Settlement:** hamlet, village, town, city, capital, monastery, fortress, university, industrial.
- **Feature:** mountain, hill, tree_conifer, tree_broadleaf, tree_rainforest, tree_savanna, tree_wetland, shrub, cactus, boulder.
- **POI:** ruin, landmark, mountain_peak, named_forest, battlefield, shrine, cave, other.

### 5.5 CARTO lists, in order

**Style preset tiles** (`RenderWorkspace.STYLE_PRESETS`, 18). Format: tile label (base look).

1. Natural Vibrant (Natural Vibrant)
2. Default (Quality tier)
3. Antique (Antique Parchment)
4. Ink (Natural Vibrant)
5. Watercolor (Natural Vibrant)
6. Print (Natural Vibrant)
7. Village (Quality tier)
8. Atlas (Natural Vibrant)
9. Imhof relief (Quality tier)
10. Blueprint (Quality tier)
11. Ink wash (Quality tier)
12. Woodcut (Antique Parchment)
13. Vintage atlas (Antique Parchment)
14. Nautical (Quality tier)
15. Night (Quality tier)
16. Cel / Toon (Natural Vibrant)
17. Tanaka relief (Quality tier)
18. Satellite relief (Satellite relief)

Every tile has variant chips **Dark / Light / Pastel** (`STYLE_VARIANTS`).

**Other lists:**

- **Base looks** (`render::LOOK_PRESETS`): Quality tier, Natural Vibrant, Antique Parchment, Satellite relief.
- **Painter styles**: see 5.1.
- **Ramp presets** (`render::RAMP_PRESETS`):
  - Earth, Elevation, Atlas, Mono, Imhof, Ice, Dark ice
  - Desert, Dark atlas, Blueprint, Ink wash, Night, Vintage atlas
  - plus "Custom" in the picker
- **Ramp blend**: Linear, Ease, Step.
- **Raster blend modes**: Normal, Multiply, Screen, Overlay, Add.
- **Map layer toggles, in order:** the 8 visible layers, then the 6 settlement classes, then the landmark types, then the 5 way types, then the 2 political layers, then the 3 terrain raster layers (5.2).
- **Analysis fields** (the Layers popover and Data overlays): 43 in 6 groups (Appendix C).

### 5.6 CARTO on the phone

- **MAP tab** (▤): selects the CARTO domain. Header "MAP", subtitle "layers · style · annotation".
- **The sheet holds the CARTO tool options row and nothing else**, plus the user's pinned quick row (CQ-6) at half detent or above:
  - With Inspect armed, that row is the CARTOGRAPHY · \<MODE\> note.
  - With Label or Icon armed, it is their options row.
  - There is no Style, Layers, Labels or Icons content in the MAP sheet. I searched `build_phone_map|phone_map|_phone_tab == "map"`. The only MAP-specific code is quick-row visibility and the half-detent height (`dcc_shell.gd:7542-7650`).
- **CARTO categories are reachable on the phone only as the left sheet**, through MORE ▸ Window ▸ Left dock while CARTO is active. That is the same 7-category accordion, with the TOOLS block (Inspect, Measure, Region, Pan, Icon, Label) drawn in the dock.
- **Arming Label or Icon, picking a View field, or picking a Style preset** is also possible through the **thumb fan** (pill above the sheet: press and slide up), using the CARTO diagonals in 5.0.
- **Analysis fields:** the map's **Layers** button is built on every form factor. I found no phone gate in `viewport_host.gd` where `_layers_btn` is built.

---

## 6. Right dock contexts (all domains)

`right_dock.gd` builds one **selection context**, then the **armed tool's section** (`_append_tool`), then **CARTO's Layers section**. On the phone it is the right sheet. It opens itself on a settlement, landmark, icon or conflict pick; otherwise use MORE ▸ Window ▸ Right dock.

**Selection contexts** (`CTX_TITLES`):

| Context | Opened by | Sections |
|---|---|---|
| **Sample** (default) | always | Sample: 23 rows, see 3.1 |
| **Settlement** | settlement click or pick | Settlement, with an Actions group: Economy, Politics, Logistics. Also faith, layout and "why here" blocks (`_build_settlement_faith/_layout/_why`) |
| **Route** | Routes & ways list, or route click | Route |
| **River** | river pick | River |
| **Landmark** | landmark click | Landmark |
| **Icon** | icon click | Icon |
| **Conflict** | conflict select | Conflict, Manpower by side, Besieged |
| **Faction** | faction select | Faction, with an Actions group holding [Open in Factions…], plus Relations |
| **Measure** | Measure tool | Measure · distance or bearing, Derived, actions, saved measurements, area, radius, vertical, cross-section |
| **Region select** | Region tool drag | Region select |
| **Ecoregion** | Wildlife field click | Ecoregion, with fauna |
| **History** | `show_history()` | Open draft, Steps, Cost |

**Appended tool sections** (`_tool_section()`):

| Section | Shown when |
|---|---|
| Paint · \<Layer\> | WORLD with Paint armed |
| Ramp · stops | CARTO with Inspect armed |
| Annotation · placed this session, plus Selected label | Label or Icon armed |
| Territory | Territory or Territory lasso armed |
| Way draft | a Way or Route draft exists |
| Journey results | CIVIL with Journey armed |
| Stamp stack | WORLD with Sculpt armed or a draft holding stamps. It also follows any other section while the draft is live |

The design spec has more contexts than this; they are listed in 8.3.

---

## 7. Phone and tablet today

### 7.1 Tablet

The desktop composition with touch sizing:

- rail, node list, both docks, the palette bar, and the scrollable options bar
- sizes from 44 px tap floors (`DccTheme.role_px`)
- portrait reflows WORLD slider rows into two-line cells

Every node and category above exists on the tablet exactly as on desktop.

### 7.2 Phone structure

- **App bar:**
  - a pill reading "CARTALITH" plus the world subtitle
  - Search (when place search exists)
  - ⋮ **More actions**: Save project, Theme, Close world
- **Bottom bar** (`PHONE_TABS`):

  | Tab | Glyph | Domain | Header subtitle |
  |---|---|---|---|
  | MAP | ▤ | cartography | "layers · style · annotation" |
  | GENERATE | ⌗ | world | "The generation pipeline, and Sculpt" (live: pipeline / sculpt / paint) |
  | PLAN | ➔ | (none) | "Journey planner" (live when planned) |
  | MORE | ⋯ | (none) | "program · data · preferences" |

  - **CIVIL has no tab.** It lives under MORE.
  - Window ▸ Domain rail hides MAP, GENERATE and PLAN, but never MORE.
  - Landscape turns the bar into a side rail.
- **Tool sheet:** detents peek (66 dp), half and full. A tab tap lifts a peeking sheet to half; re-tapping the lit tab drops it to peek.
- **What the sheet holds:**
  - GENERATE: the phone generate column (3.1, 3.2).
  - MAP: the tool-options row plus the quick row.
  - PLAN and MORE open full overlays instead.
- **Thumb fan** (`tool_fan.gd`):
  - an armed-tool pill above the sheet
  - tap toggles between Inspect and the last tool
  - press and slide opens a half-fan of the same 8 ring slots as desktop: global cardinals plus the active domain's diagonals
- **MORE root** (`ROOT_ROWS`), followed by the STATUS block and then the **Edit** and **Window** bands (`ROOT_REST`, appended after STATUS):
  - Project
  - Civilization
  - Data manager
  - Asset library
  - Travel library
  - Simulation
  - Preferences
  - Help & about
- **Window** holds:
  - region toggles: Left dock, Right dock, Timeline, Status bar, Domain rail. On the phone, Left and Right dock open the sheets.
  - Workspace (World / Civilization / Cartography)
  - Open windows, Reset layout, Layouts, Session mode
- **Docks as sheets:** the left sheet is the active domain's full accordion; the right sheet is the inspector.

### 7.3 Per-node phone route (summary)

| Node | Phone route today | Gaps a mockup should know about |
|---|---|---|
| WORLD ▸ Generate | **GENERATE** tab, PIPELINE segment | Center landmasses, Import, Finalize / bake, Droplet erode, Carve fjords, Ecology readouts, World data and CRS are only in the left sheet |
| WORLD ▸ Sculpt | **GENERATE** tab, SCULPT and PAINT segments | Per-feature parameters, Freehand sub-mode, Grid snap and Seed are only in the left sheet. Falloff shows as a numeric slider |
| CIVIL ▸ Landmarks | MORE ▸ Civilization ▸ Landmark generation | — |
| CIVIL ▸ Factions (10 categories) | MORE ▸ Civilization ▸ Civilization dock (left sheet) | No bespoke phone screens for Populate … Timeline |
| CIVIL ▸ Routes & ways | MORE ▸ Civilization (Way and Route tools), or the Civilization dock | — |
| CIVIL ▸ Journey planner | **PLAN** tab | — |
| CARTO ▸ Style / Layers / Labels / Icons | **MAP** tab, which shows the tool options only, plus the fan (Label, Icon, View, Style). Full categories via MORE ▸ Window ▸ Left dock | The spec's MAP sheet (TOOLS, LAYERS, STYLE) is not built (8.4) |

---

## 8. Code against the design: differences (the code wins; both reported)

### 8.1 `design/dcc-environment-2026-08-31/spec/02-rail-and-domains.md`

- **§0 says the canvas is truncated at 262 144 bytes. It is stale.** `Cartalith DCC Environment.dc.html` is 239 712 bytes and ends `</script></body></html>`, which I checked with `wc -c` and `tail -c`. MISTAKES.md "Cite a document as authority for an absence" records the same staleness.
- **§3 node labels: Generation pipeline, Sculpt, Landmarks, Factions & settlements, Ways & routes, Journey planner, Layers & style, Labels, Icons, Terrain appearance.**
  - Code (Ruling L): **Generate**, Sculpt, Landmarks, **Factions**, **Routes & ways**, Journey planner, **Style, Layers**, Labels, Icons.
  - The canvas itself (line 1824) now gives CARTO modes `style`, `labels`, `icons` and `terrain`. The spec's §3a claim that the "four CARTO nodes carry no mode" is stale against the canvas.
  - The code has no `terrain` / Terrain appearance node. Its fourth CARTO node is Layers, and Relief & light hangs off Style. The code flags the conflicting "footer TERRAIN → RELIEF" instruction as left for the owner.
- **§4d tool palette:**
  - WORLD in the spec: Sculpt (no key), Freehand F, Biome paint B. Code: only **Biome paint (B)**, SCULPT-only. Sculpt and Freehand are armed from the feature grid, with F bound there.
  - CIVIL in the spec: Settlement S, **POI P**, Territory T, Way W, Route ⇧R. Code: **no POI**; adds **Territory lasso (⇧T)** and **Conflict**.
  - CARTO matches: Label L, Icon I.
- **§4 dock blocks:**
  - The spec gates CIVIL bodies by `civCat`, one body open. Code: all 13 CIVIL categories are always shown as an accordion (per `RAIL_NODES` header and Ruling L).
  - The spec's CARTO dock is `ldCarto` plus `ldRender`. Code: 7 categories.
- **§4c:** the spec says no key switches domain. The code also has none; ⇧J opens the planner.

### 8.2 `spec/04-left-dock.md` (the spec's ten-stage pipeline, against code)

The spec's §4.3 parameter table is the canvas mock's, roughly 35 illustrative dials. The code draws the engine's 120-row table (Appendix A). Notable differences:

| Spec row | Code |
|---|---|
| 01 Planet: gravity 0.5–2, day 6–48, tilt step 0.1 | Gravity 0.30–2.50 · 0.05; Day length 6–96 · 1; Axial tilt 0–45 · 0.5 |
| 02 `working res` segmented 512–8K | Not in the dock: File ▸ New world. Phone shows a route row |
| 02 sea level 0–100 % | Sea level 0–1 · 0.01 |
| 02 peak 1000–9000 m | Peak altitude 1–30 000 · 50 m |
| 03 `archetype` segmented | File ▸ New world ▸ World structure ▸ Archetype |
| 04 nine sliders (drift 0.1–3, uplift 2–60, α 0–2, β 0–1, flexure 0–1, heterogeneity 0–0.5) | 16 rows: Drift 0–2; Uplift spread 2–42 px; α 0–1.2; β 0–0.6; Flexure 0–0.36; Heterogeneity 0–0.16; plus Erosion / age, Ridged, Lloyd, Dynamic lithology and three port switches |
| 05: 4 dials (craters 0–400 · 10) | 19 rows: Craters 0–200 · 2; plus 13 model toggles and Surface age |
| 06: six 0–1 sliders (droplet, hillslope, stream, velocity, glacial, coastal) | 29 `erosion` rows, plus the Droplet tool group and Carve fjords. The phone row "Erosion strength" is dashed: no such parameter |
| 07: river density 0–1, min stream order, lakes as water | River density 0.30–3.0 ×; Carve rivers; Integrated drainage. **Min stream order and lakes as water are not settable** (`STAGES[6].gap`, `PHONE_GEN_ABSENT`) |
| 08: equator 10–45, pole −60–10, lapse 3–10, rainfall ×, Köppen toggle, currents | Equator 0–45; Pole −50–10; Lapse 0–12; 9 + 12 rows across two groups; no Köppen toggle (computed lazily on view) |
| 09: ecotone sharpness, rivers in biome view | Not parameterised (dashed rows on the phone) |

Spec §5 (Sculpt) Brush shape and Stroke & grid are not built (§ Not built note). Spec §6 CIVIL blocks become the 13 categories.

### 8.3 `spec/05-right-dock-and-bars.md`

- **§1.2 contexts.** The spec has stamps, paint, anno, terr, place, stops, way and plan as one *replacing* mode. Code: selection contexts plus *appended* tool sections (owner dock ruling: "selection wins"). Journey became an appended section on 2026-09-04.
- **§1.3 titles.** The spec has "PAINT · SOIL / VEGETATION". Code paint targets are Biome / Terrain / Splat / Rock.
- **§2.2 tool options rows.**
  - The spec has POI (`tbPoi`). Code: no POI.
  - The code adds Territory lasso, Conflict, Rock paint, the colour paint tool, and the unified Sculpt / Paint / Measure bar (from `design/Cartalith Paint Toolbar.dc.html`).
- **§4 timeline.** The spec gate `tlShow` is unspecified, and its Window item is "Timeline (CIVIL · INFRA)". Code: visible whenever CIVIL is active, collapsed by default. The six layer toggles, speeds and year range match the spec. The code adds a DATE (day) row and a counts readout.
- **§6 tool palette.** Same differences as 8.1. The code moved the TOOLS block from the dock to a top palette bar off the phone (owner, 2026-09-23).

### 8.4 `spec/06-phone.md` (read for the mobile redesign)

- **§2 tabs:** MAP / GENERATE / PLAN / MORE, matching the code. The spec's default is "no sheet"; the code lights GENERATE at boot (`_phone_tab := "gen"`).
- **§6.3 MAP sheet:**
  - The spec draws TOOLS (Inspect, Measure, Label, Icon, with an icon variant row), LAYERS (Surface base, Terrain fields, Climate overlays) and STYLE (four presets: Atlas, Parchment, Physical, Ink; nine ramps).
  - **Code: none of these is in the MAP sheet.** It holds the CARTO tool options row and the pinned quick row. Tools are reached via the thumb fan or the left sheet; layers via the map's Layers button or the left sheet; style via the left sheet or the fan's Style diagonal.
- **§6.4 GENERATE:**
  - The spec has two modes, PIPELINE and SCULPT. Code: **three**, adding PAINT.
  - The spec pipeline's parameter set is the mock's. Code: the engine table, by stage, plus `PHONE_GEN_ABSENT` dashed and route rows.
- **§6.6 MORE `civ`:** the code matches in spirit, and adds Territory lasso, Conflict and the Civilization dock row.

### 8.5 Owner tree, Ruling L (`design/owner-references-2026-09-12/left_rail_tree_resorted.md`)

The code follows this tree and says so. These are the points where it does not:

- **Node list:** the tree has 5 CIVIL nodes (Settlements first). Code has 4: no Settlements node (1.3).
- **WORLD Hydrology ▸ River network:** the tree has 2 rows. Code has 3 (adds Integrated drainage).
- **CIVIL Tools:** the tree has Settlement, Territory, Way, Route. Code adds Territory lasso and Conflict.
- **CIVIL Factions:** the tree has Roster, Identity colour, Linked notes, Not built. Code has **Roster only**.
- **CARTO Style ▸ Look:** the tree has "6 style tiles". Code has **18 tiles × 3 variants**. The Water section adds a **Village map** toggle.
- **CARTO Relief & light ▸ Rendering · advanced:** the tree has 5 expanders. Code has **6**, adding Ice & snow (LOD-D4). The section title in code is "Rendering - advanced".
- **CARTO Colours:**
  - The tree has Colour relief, Colour grade (2 expanders), Biome colours and Colour management.
  - Code has Colour relief; **Colour grade (3 expanders: Rivers, Colour grade, Grade field influence)**; Biome colours; **Look (quick)**; **Adjustments**; Colour management; and **Paint colour**.
- **CARTO Layers ▸ Map layers:** the tree has 6. Code has **8**, adding Conflict and Rivers, in a section titled "Visible layers". Code also adds the **Landmarks · by type** expander.
- **Rail footer:** the tree says TERRAIN → RELIEF. Code prints the mode id, so the fourth CARTO node shows `LAYERS` (1.3).

### 8.6 `GENERATION_PARAMETERS.md`

- 113 of the 120 `PARAMS` keys have a table row. The 7 without one:
  - `geology_model`, `geology_processes`, `relief_model`, `geology_karst`, `geology_carve`: no GUI.
  - `geo.age`: default 1.0, from `geo_clock::GEO_AGE_DEFAULT`.
  - `civ.garrison_exposure_scale`: default 1.0, from `WorldParams::defaults`.
- The document's defaults are `WorldParams::defaults`. I spot-checked them against `cartalith-engine/src/lib.rs` and `ErosionPassParams::off()`; all matched. The document's ¹ marks the keys whose app default differs; Appendix A marks the app value.

---

## 9. Oddities found while reading (not fixed: read-only lane)

1. **"Metropoliss" and "Citys".** `cartography_workspace.gd::_build_settlement_class_filter()` builds labels as `String(kind).capitalize() + "s"`. For `metropolis` and `city` that yields **Metropoliss** and **Citys**. The owner tree spells "Metropolises".
2. **Freehand Amount range.** The dock's Freehand parameter is **Amount 0.02–0.3** (engine `FREEHAND_CTL`). The unified tool bar draws **Amount −1.0 to 1.0** for the same key (`tool_bar.gd::_build_sculpt_options`). Which range the engine clamps to on write is not verified here.
3. **Phone sculpt globals.**
   - `_pg_sculpt_slider` reads `spec.get("int")`, but `get_sculpt_globals_info()` returns `type`. Octaves is therefore treated as float. Its step is 1, so values still snap; the readout shows "%.2f".
   - **Falloff** (an enum) draws as a numeric 0–3 slider with no Smooth / Linear / Sharp / Constant labels.
   - Inferred from the code, not seen on a device.
4. **Stale counts in comments** (MISTAKES.md "Quote a count…"):
   - `RAIL_NODES`' header says a probe asserts "all thirty-two categories". The live docks hold **29** (9 + 13 + 7, greps in §0).
   - `params.rs::defaults()`'s doc says "thirteen flags". **20** `= true` lines (command in §0).
   - `world_workspace.gd`'s header says "99 parameters in 9 groups". **120 in 9 groups**. Its own text says to re-count.
5. **`signal tool_changed`** in `dcc_shell.gd` is declared and, per its own comment, never emitted. Do not design against it.

---

## Appendix A: all 120 generation parameters

Read mechanically from `crates/cartalith-godot/src/params.rs::PARAMS`, in file order.

- **Label, kind, range, step, unit:** from that table.
- **Default:** `WorldParams::defaults`, via `GENERATION_PARAMETERS.md`, spot-checked against `cartalith-engine/src/lib.rs`. Where the shipped app differs (`params::defaults()`, 20 flags), the app value is shown in bold.
- **Desktop home:** derived from `WorldWorkspace.STAGES` / `CATEGORIES` / `ADVANCED_KEYS` and the `civ` builders.
- **Phone column:** the GENERATE sheet's stage group number. "—" means not in the phone sheet.

| # | key | label (as drawn) | control | range · step | unit | default | desktop home | phone GENERATE group |
|---|---|---|---|---|---|---|---|---|
| 1 | `world` | Whole world (wrap in longitude) | toggle | — | — | false | WORLD ▸ Planet ▸ 02 Extent & scale | 02 |
| 2 | `sea_level` | Sea level | slider | 0.0 – 1.0 · 0.01 | — | 0.42 | WORLD ▸ Planet ▸ 02 Extent & scale | 02 |
| 3 | `peak_m` | Peak altitude | slider | 1.0 – 30000.0 · 50.0 | m | 4000 | WORLD ▸ Planet ▸ 02 Extent & scale | 02 |
| 4 | `carve_rivers` | Carve river valleys on generation | toggle | — | — | true | WORLD ▸ Hydrology ▸ River network | 07 |
| 5 | `river_density` | River density | slider | 0.30 – 3.0 · 0.05 | × | 1.00 | WORLD ▸ Hydrology ▸ River network | 07 |
| 6 | `integrate_drainage` | Integrated drainage (route over filled depressions) | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Hydrology ▸ River network | 07 |
| 7 | `geology_model` | Geology-first lithology model | toggle | — | — | false (engine) / **true (app)** | no GUI control (save-record switch; params.rs says no workspace lists it) | — |
| 8 | `geology_processes` | Landform processes read the rock column | toggle | — | — | false | no GUI control (save-record switch; params.rs says no workspace lists it) | — |
| 9 | `relief_model` | Rock-aware relief construction | toggle | — | — | false | no GUI control (save-record switch; params.rs says no workspace lists it) | — |
| 10 | `geology_karst` | Karst dissolution and permeable runoff | toggle | — | — | false | no GUI control (save-record switch; params.rs says no workspace lists it) | — |
| 11 | `geology_carve` | Valley width reads rock | toggle | — | — | false | no GUI control (save-record switch; params.rs says no workspace lists it) | — |
| 12 | `use_gpu` | GPU acceleration | toggle | — | — | false (engine); shell sets true at boot | not in a dock: Preferences menu ▸ PERFORMANCE ▸ GPU acceleration (check item) | — |
| 13 | `planet.g` | Gravity | slider | 0.30 – 2.50 · 0.05 | g | 1.00 | WORLD ▸ Planet ▸ 01 Planet | 01 |
| 14 | `planet.rotation_hours` | Day length | slider | 6.0 – 96.0 · 1.0 | h | 24.0 | WORLD ▸ Planet ▸ 01 Planet | 01 |
| 15 | `planet.axial_tilt_deg` | Axial tilt | slider | 0.0 – 45.0 · 0.5 | ° | 23.4 | WORLD ▸ Planet ▸ 01 Planet | 01 |
| 16 | `world_structure.enabled` | Enable continental steering | toggle | — | — | false | WORLD ▸ Planet ▸ 03 World structure | 03 |
| 17 | `world_structure.continentality` | Continentality | slider | 0.01 – 0.90 · 0.01 | — | 0.30 | WORLD ▸ Planet ▸ 03 World structure | 03 |
| 18 | `world_structure.fragmentation` | Fragmentation | slider | 0.0 – 1.0 · 0.01 | — | 0.50 | WORLD ▸ Planet ▸ 03 World structure | 03 |
| 19 | `world_structure.tectonic_energy` | Tectonic energy | slider | 0.0 – 1.0 · 0.01 | — | 0.60 | WORLD ▸ Planet ▸ 03 World structure | 03 |
| 20 | `world_structure.ocean_depth` | Ocean depth | slider | 0.0 – 1.0 · 0.01 | — | 0.60 | WORLD ▸ Planet ▸ 03 World structure | 03 |
| 21 | `world_structure.hotspot_density` | Hotspot density | slider | 0.0 – 1.0 · 0.01 | — | 0.20 | WORLD ▸ Planet ▸ 03 World structure | 03 |
| 22 | `tect.plates` | Plates | slider (int) | 4.0 – 40.0 · 1.0 | — | 14 | WORLD ▸ Geology ▸ 04 Tectonics | 04 |
| 23 | `tect.vel` | Drift | slider | 0.0 – 2.0 · 0.02 | × | 1.00 | WORLD ▸ Geology ▸ 04 Tectonics | 04 |
| 24 | `tect.warp` | Warp | slider | 0.0 – 1.0 · 0.01 | — | 0.45 | WORLD ▸ Geology ▸ 04 Tectonics | 04 |
| 25 | `tect.blur_r` | Uplift spread | slider | 2.0 – 42.0 · 0.4 | px | 18.0 | WORLD ▸ Geology ▸ 04 Tectonics | 04 |
| 26 | `tect.alpha` | Tectonic α | slider | 0.0 – 1.2 · 0.012 | — | 0.85 | WORLD ▸ Geology ▸ 04 Tectonics | 04 |
| 27 | `tect.beta` | Noise β | slider | 0.0 – 0.6 · 0.006 | — | 0.22 | WORLD ▸ Geology ▸ 04 Tectonics | 04 |
| 28 | `tect.age_inf` | Erosion / age | slider | 0.0 – 1.0 · 0.01 | — | 0.60 | WORLD ▸ Geology ▸ 04 Tectonics | 04 |
| 29 | `tect.ridged` | Ridged mountain detail | toggle | — | — | true | WORLD ▸ Geology ▸ 04 Tectonics | 04 |
| 30 | `tect.flexure` | Flexure F | slider | 0.0 – 0.36 · 0.006 | — | 0.20 | WORLD ▸ Geology ▸ 04 Tectonics ▸ advanced | 04 |
| 31 | `tect.hetero` | Heterogeneity C | slider | 0.0 – 0.16 · 0.004 | — | 0.08 | WORLD ▸ Geology ▸ 04 Tectonics ▸ advanced | 04 |
| 32 | `tect.resist` | Rock resistance | slider | 0.0 – 1.0 · 0.01 | — | 0.50 | WORLD ▸ Geology ▸ 04 Tectonics ▸ advanced | 04 |
| 33 | `tect.dynamic_lithology` | Dynamic lithology (exhumation hardens rock) | toggle | — | — | false | WORLD ▸ Geology ▸ 04 Tectonics ▸ advanced | 04 |
| 34 | `tect.lloyd` | Lloyd relaxation passes | slider (int) | 0.0 – 8.0 · 1.0 | — | 2 | WORLD ▸ Geology ▸ 04 Tectonics ▸ advanced | 04 |
| 35 | `tect.narrow_plate_base_blur` | Coastline leaves the plate outlines (v2.57 blur) | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 04 Tectonics | 04 |
| 36 | `tect.orogen_relief` | Ridged mountain ranges in the orogenic belts | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 04 Tectonics | 04 |
| 37 | `tect.high_ceiling` | Higher world ceiling (tall volcanoes keep their summits) | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 04 Tectonics | 04 |
| 38 | `volc.count` | Volcanoes | slider (int) | 0.0 – 100.0 · 1.0 | — | 20 | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 39 | `volc.age` | Volcano age | slider | 0.0 – 1.0 · 0.01 | — | 0.40 | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 40 | `volc.provinces` | Provinces & arc/rift placement | toggle | — | — | true | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 41 | `volc.exclude_transform` | Exclude transform margins | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 42 | `volc.edifice_model` | Shaped edifices (shield/strato/cone) | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 43 | `volc.fill_edifice_pits` | Drain volcano flank pits | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 44 | `volc.morphometry` | Volcano morphometry (VA-5) | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 45 | `volc.cinder_grid_law` | Scoria cones sized for the grid (Ruling CX) | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 46 | `volc.dry_active_craters` | Active craters stay dry (Ruling CX) | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 47 | `volc.lava_bench` | Young lava flows build benches at the sea (Ruling DB) | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 48 | `volc.kind_dome` | Lava domes on young stratocones (VA-6) | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 49 | `volc.kind_maar` | Maars beside scoria cones on wet ground (VA-6) | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 50 | `volc.kind_caldera` | Old arc stratocones collapse into calderas (VA-6) | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 51 | `volc.kind_supervolcano` | A supervolcano caldera where a hotspot sits under a continent (VA-6) | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 52 | `volc.kind_flood_basalt` | A flood-basalt plateau, drawn only (VA-6) | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 53 | `crater.count` | Craters | slider (int) | 0.0 – 200.0 · 2.0 | — | 100 | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 54 | `crater.age` | Crater age | slider | 0.0 – 1.0 · 0.01 | — | 0.50 | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 55 | `crater.physical_model` | Physical crater model (map area + wear) | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 56 | `crater.surface_age_myr` | Surface age | slider | 0.0 – 4000.0 · 10.0 | Myr | 100.0 | WORLD ▸ Geology ▸ 05 Volcanism & impacts | 05 |
| 57 | `geo.age` | Geological age | slider | 0.25 – 4.0 · 0.05 | × | 1.0 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 58 | `stream.uplift` | Uplift | slider | 0.0 – 0.4 · 0.004 | — | 0.00 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 59 | `stream.k` | Channeling | slider | 0.0 – 0.03 · 0.0003 | — | 0.012 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 60 | `stream.iters` | Iterations | slider (int) | 4.0 – 40.0 · 1.0 | — | 15 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 61 | `stream.deposit` | Deposition | slider | 0.0 – 1.0 · 0.01 | — | 0.30 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 62 | `stream.climate_k` | Rain → erosion | slider | 0.0 – 1.0 · 0.01 | — | 0.50 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 63 | `passes.velocity` | Velocity (momentum) erosion | toggle | — | — | false | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 64 | `passes.velo_iters` | Velocity iterations | slider (int) | 10.0 – 160.0 · 1.0 | — | 60 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 65 | `passes.velo_strength` | Velocity strength | slider | 0.0 – 1.0 · 0.01 | — | 0.50 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 66 | `passes.velo_meander` | Velocity meander | slider | 0.0 – 1.0 · 0.01 | — | 0.60 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 67 | `passes.glacial` | Glacial erosion | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 68 | `passes.glacial_snowline` | Snowline | slider | 0.0 – 1.0 · 0.01 | — | 0.65 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 69 | `passes.glacial_kg` | Glacial intensity | slider | 0.01 – 1.0 · 0.01 | — | 0.15 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 70 | `passes.glacial_mg` | Glacial discharge exponent | slider | 0.0 – 2.0 · 0.05 | — | 0.40 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 71 | `passes.glacial_u_factor` | U-width | slider | 0.0 – 1.0 · 0.01 | — | 0.60 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 72 | `passes.glacial_passes` | Glacial passes | slider (int) | 1.0 – 30.0 · 1.0 | — | 8 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 73 | `passes.coastal` | Coastal processes | toggle | — | — | false | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 74 | `passes.wave_str` | Wave strength | slider | 0.0 – 1.0 · 0.01 | — | 0.50 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 75 | `passes.estuary_depth` | Estuary depth | slider | 0.0 – 0.2 · 0.002 | — | 0.08 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 76 | `passes.marsh_band` | Marsh band | slider | 0.0 – 0.1 · 0.001 | — | 0.03 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 77 | `passes.coastal_passes` | Coastal passes | slider (int) | 1.0 – 15.0 · 1.0 | — | 4 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 78 | `passes.hillslope` | Hillslope diffuse | toggle | — | — | false | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 79 | `passes.diffuse_d` | Diffusivity D (also weathers craters) | slider | 0.002 – 0.2 · 0.002 | — | 0.15 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 80 | `passes.diffuse_passes` | Diffuse passes | slider (int) | 1.0 – 40.0 · 1.0 | — | 6 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 81 | `passes.sediment_fill` | Sediment fill (route + redeposit) | toggle | — | — | false | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 82 | `passes.sediment_capacity` | Sediment transport capacity | slider | 0.0 – 20.0 · 0.5 | — | 6.0 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 83 | `passes.evolve_cycles` | Evolve climate ↔ terrain cycles | slider (int) | 0.0 – 12.0 · 1.0 | — | 0 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 84 | `passes.tidal_flats` | Tidal flats (mudflat accretion) | toggle | — | — | false | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 85 | `passes.tidal_k` | Tidal accretion rate | slider | 0.0 – 1.0 · 0.01 | — | 0.45 | WORLD ▸ Hydrology ▸ 06 Erosion · water & ice ▸ Stream-power carve | 06 |
| 86 | `climate.lat_n` | North edge | slider | -90.0 – 90.0 · 1.0 | ° | 55 | WORLD ▸ Climate ▸ Climate & temperature | 08 |
| 87 | `climate.lat_s` | South edge | slider | -90.0 – 90.0 · 1.0 | ° | 5 | WORLD ▸ Climate ▸ Climate & temperature | 08 |
| 88 | `climate.equator_temp` | Equator temperature | slider | 0.0 – 45.0 · 1.0 | °C | 30 | WORLD ▸ Climate ▸ Climate & temperature | 08 |
| 89 | `climate.pole_temp` | Pole temperature | slider | -50.0 – 10.0 · 1.0 | °C | −25 | WORLD ▸ Climate ▸ Climate & temperature | 08 |
| 90 | `climate.lapse_rate` | Lapse rate | slider | 0.0 – 12.0 · 0.1 | °C/km | 6.5 | WORLD ▸ Climate ▸ Climate & temperature | 08 |
| 91 | `climate.albedo_k` | Ice albedo | slider | 0.0 – 1.0 · 0.01 | — | 0.00 | WORLD ▸ Climate ▸ Climate & temperature | 08 |
| 92 | `climate.currents` | Ocean currents | toggle | — | — | true | WORLD ▸ Climate ▸ Climate & temperature | 08 |
| 93 | `climate.current_k` | Ocean-current strength | slider | 0.0 – 3.0 · 0.05 | × | 1.00 | WORLD ▸ Climate ▸ Climate & temperature ▸ advanced | 08 |
| 94 | `climate.terrain_wind_deflection` | Terrain wind deflection | toggle | — | — | true | WORLD ▸ Climate ▸ Climate & temperature ▸ advanced | 08 |
| 95 | `climate.w_iters` | Iterations | slider (int) | 20.0 – 200.0 · 5.0 | — | 70 | WORLD ▸ Climate ▸ Weather · rainfall sim | 08 |
| 96 | `climate.rain_k` | Orographic | slider | 0.0 – 2.0 · 0.01 | × | 1.00 | WORLD ▸ Climate ▸ Weather · rainfall sim | 08 |
| 97 | `climate.evap` | Evaporation | slider | 0.0 – 0.3 · 0.003 | — | 0.12 | WORLD ▸ Climate ▸ Weather · rainfall sim | 08 |
| 98 | `climate.rain_dep` | Dryness | slider | 0.0 – 1.0 · 0.01 | — | 0.35 | WORLD ▸ Climate ▸ Weather · rainfall sim | 08 |
| 99 | `climate.ocean` | Ocean supply | slider | 0.0 – 2.0 · 0.01 | × | 1.00 | WORLD ▸ Climate ▸ Weather · rainfall sim | 08 |
| 100 | `climate.wind_manual` | Manual wind direction | toggle | — | — | false | WORLD ▸ Climate ▸ Weather · rainfall sim | 08 |
| 101 | `climate.wind_dir_deg` | Wind direction | slider | 0.0 – 360.0 · 5.0 | ° | 0 | WORLD ▸ Climate ▸ Weather · rainfall sim | 08 |
| 102 | `climate.press_k` | Pressure influence | slider | 0.0 – 1.5 · 0.05 | — | 0.60 | WORLD ▸ Climate ▸ Weather · rainfall sim | 08 |
| 103 | `climate.zonal_k` | Zonal belts | slider | 0.0 – 1.5 · 0.05 | — | 0.50 | WORLD ▸ Climate ▸ Weather · rainfall sim | 08 |
| 104 | `climate.ocean_hum` | Sea-surface humidity | slider | 0.0 – 2.0 · 0.01 | — | 1.00 | WORLD ▸ Climate ▸ Weather · rainfall sim ▸ advanced | 08 |
| 105 | `climate.bulk_evap` | Bulk aerodynamic evaporation | toggle | — | — | true | WORLD ▸ Climate ▸ Weather · rainfall sim ▸ advanced | 08 |
| 106 | `climate.rain_bspline` | Smooth (B-spline) rain up-sample | toggle | — | — | false (engine) / **true (app)** | WORLD ▸ Climate ▸ Weather · rainfall sim | 08 |
| 107 | `civ.villages` | Seed villages | toggle | — | — | false | not in a dock: File ▸ New world ▸ Generation (placement model) | — |
| 108 | `civ.metropolis` | Imperial seats (metropolis tier) | toggle | — | — | false | not in a dock: File ▸ New world ▸ Generation (placement model) | — |
| 109 | `civ.recovery_phase` | Post-collapse recovery phase | slider (int) | 0.0 – 4.0 · 1.0 | — | 0 | not in a dock: File ▸ New world ▸ Generation (placement model) | — |
| 110 | `civ.biome_k` | Biome carrying-capacity residual | toggle | — | — | false | not in a dock: File ▸ New world ▸ Generation (placement model) | — |
| 111 | `civ.factions` | Factions | slider (int) | 1.0 – 24.0 · 1.0 | — | 6 | not in a dock: File ▸ New world ▸ Generation (placement model) | — |
| 112 | `civ.seed_thresh` | Settlement suitability floor | slider | 0.10 – 0.80 · 0.01 | — | 0.42 | not in a dock: File ▸ New world ▸ Generation (placement model) | — |
| 113 | `civ.seed_suppress_div` | Settlement packing | slider | 8.0 – 60.0 · 1.0 | — | 22.0 | not in a dock: File ▸ New world ▸ Generation (placement model) | — |
| 114 | `civ.fixed_counts` | Fixed settlement counts | toggle | — | — | false | not in a dock: File ▸ New world ▸ Generation (placement model) | — |
| 115 | `civ.n_capital` | Capitals | slider (int) | 0.0 – 50.0 · 1.0 | — | 0 | not in a dock: File ▸ New world ▸ Generation (placement model) | — |
| 116 | `civ.n_city` | Cities | slider (int) | 0.0 – 200.0 · 1.0 | — | 0 | not in a dock: File ▸ New world ▸ Generation (placement model) | — |
| 117 | `civ.n_town` | Towns | slider (int) | 0.0 – 500.0 · 1.0 | — | 0 | not in a dock: File ▸ New world ▸ Generation (placement model) | — |
| 118 | `civ.n_village` | Villages | slider (int) | 0.0 – 1000.0 · 1.0 | — | 0 | not in a dock: File ▸ New world ▸ Generation (placement model) | — |
| 119 | `civ.n_hamlet` | Hamlets | slider (int) | 0.0 – 2000.0 · 1.0 | — | 0 | not in a dock: File ▸ New world ▸ Generation (placement model) | — |
| 120 | `civ.garrison_exposure_scale` | Garrison border-exposure scale | slider | 0.0 – 5.0 · 0.1 | × | 1.0 | CIVIL ▸ Military ▸ Garrisons ▸ Border-exposure weight (dock slider, via set_garrison_exposure_scale) | — |

Parameter rows that need context:

- `tect.plates`, `tect.vel` and `volc.count` dim while World structure is enabled.
- `geo.age` dims while `geology_processes` is off, which it always is in the app.
- `sea_level` is re-anchored by World structure.

---

## Appendix B: File ▸ New world (creation-time controls)

The "Placement model → File ▸ New world…" route. On the phone it is a card. From `new_world_dialog.gd`:

- **§ Seed**: Seed (number), a dice button and a lock.
- **§ Extent**: Extent, Region / Whole world. The phone uses chips.
- **§ Size & resolution → Grid & aspect**
  - Map width (dropdown) and Width (km)
  - Resolution: 512 / 1K / 2K / 4K / 8K / custom; default 2K
  - Grid columns
  - Aspect (dropdown)
  - Grid rows
- **Plates** row: −/+ steppers.
- **§ World structure**: Archetype. "Classic" plus Earth-like, Supercontinent, Archipelago, Volcanic, Rift.
- **§ Generation**
  - Village seeding (additive hamlets)
  - Imperial-seat tier (metropolis ★)
  - Biome carrying-capacity residual
  - Recovery phase (dropdown)
  - placement dials:
    - Factions (number)
    - Suitability floor (slider)
    - Packing (slider)
    - Fixed settlement counts (toggle), with Capitals, Cities, Towns, Villages, Hamlets
- A preview and history strip.

---

## Appendix C: the 43 analysis fields

The map's Layers popover, CARTO ▸ Layers ▸ Data overlays, and the CARTO View diagonal (first 8 available). Source: `sample_bridge.rs::LAYER_GROUPS`, in order. A field whose input the world lacks is greyed with its reason.

- **Base**
  1. No overlay (base map) (`off`)
  2. Elevation (`elevation`)
- **Climate**
  3. Temperature (`temp`)
  4. Köppen climate (`koppen`)
  5. Rainfall (`rain`)
  6. Wind (`wind`)
  7. Ocean currents (`ocean`)
- **Tectonics**
  8. Plates (`plates`)
  9. Plate boundaries (`bounds`)
  10. Tectonic type (`btype`)
  11. Orogeny (`oro`)
  12. Stress (`stress`)
  13. Crust age (`age`)
  14. Geoid (`geoid`)
  15. Tides (`tides`)
  16. Rock resistance (`resistance`)
- **Hydrology**
  17. River flow (`flow`)
  18. Strahler order (`strahler`)
  19. Velocity (`velo`)
  20. Fjord mask (`fjord`)
  21. Flood (`flood`)
- **Surface**
  22. Biomes (`bclass`)
  23. Terrain (`cterrain`)
  24. Lithology (`lith`)
  25. Landforms (`landform`)
  26. Soil fertility (`soil`)
  27. Net primary productivity (`npp`)
  28. Water access (`water`)
  29. Slope (`slope`)
  30. Aspect (`aspect`)
  31. Local relief (`relief`)
  32. Topographic position (`tpi_multi`)
- **Civilization**
  33. Resources (`rsrc`)
  34. Carrying capacity (`carry`)
  35. Pop density (persons/km²) (`popdensity`)
  36. Settlement suitability (`settle`)
  37. Site profile (buildability) (`siteprofile`)
  38. Wildlife (`wildlife`)
  39. Wind-throw (`windthrow`)
  40. Political control (`control`)
  41. Contested borders (`contested`)
  42. Route corridors (`corridor`)
  43. Travel cost (`travel_cost`)
