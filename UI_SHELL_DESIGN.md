# UI shell design — DCC-style editor

> Imported verbatim from the owner's Claude Design project "UI mockups
> planning" (2026-08-17). The design team's own `github.md` places this at
> `docs/UI_SHELL_DESIGN.md`; this repo keeps its scope/decision documents at
> the root (`GUI_SHELL_SCOPE.md`, `VISION.md`, `ARCHITECTURE.md`, etc.) rather
> than in a `docs/` folder — `cartalith-native/docs/` is reserved for the
> living `CHANGELOG.md`/`STATUS.md`. Placed here to match this repo's actual
> convention; content otherwise unedited. See `DCC_SHELL_SCOPE.md` for how
> this maps onto the current Godot port, what it supersedes, and the real
> milestone plan.

The native port's screen layout. Supersedes the HTML app's single scrolling
control column: Cartalith becomes a **map editor with a toolchain**, in the
lineage of Nortantis, terrain editors, image editors and 3D DCC applications,
rather than a form with a preview attached.

Mockups live outside this repository (Omelette project *UI mockups planning*):
`Cartalith DCC Shell.dc.html` holds all three references — 1920×1080 desktop,
2560×1600 tablet, and 393×852 Android phone — in that order in the one file.
Read it for exact spacing, type sizes and colour values; this document is the
rule set.

## The governing split

| Region | Owns | Never holds |
|---|---|---|
| **Top menu bar** | program functions — files, save locations, import/export, asset manager, graphics and rendering options, session | anything you use while your hand is on the map |
| **Left tool rail** | the map-editing toolchain — one icon per tool, always visible, keyboard-bound | settings, values, lists |
| **Tool options bar** | the active tool's frequently-changed values, horizontally, plus its commit/discard | anything belonging to a different tool |
| **Right dock** | Layers, Properties (active tool or selection), Sample, History, Assets | tool invocation |
| **Viewport** | the map, the brush cursor, scale bar, projection/zoom readout, cursor coordinates | chrome that could live in the docks |
| **Status bar** | pass state, autosave, tile cache, the active tool's modifier hints | controls |

The load-bearing rule: **the top bar is about the program, the map is about the
world.** A control that changes the world belongs to a tool or a dock; a control
that changes the program belongs to a menu. `docs/UNIFIED_TOOL_PLAN.md` decides
what a tool *is*; this document decides where it appears.

## Top menu bar

Eight menus. Every item resolves to a dialog, a submenu, or a mode toggle — none
of them open a persistent side panel.

| Menu | Contents |
|---|---|
| **File** | New world, Open project `.zip`, Save, Save as, Recent, Import heightmap, Import asset pack, Export image/tiles, Export GeoJSON, Export region, Project settings |
| **Edit** | Undo, Redo, Undo history, Preferences, Theme |
| **Generate** | The pipeline stages in order — Tectonics, Volcanism, Erosion, Glacial & coastal, Hydrology, Climate, Ecology, Settlements, Infrastructure, Politics; each opens its parameter dialog and reports staleness |
| **Simulate** | Time controls, Collapse/recovery, Economy, Statistics, Logistics |
| **Render** | Map mode, Style preset, Terrain appearance, Painter styles (NPR), Lighting & shadows, 3D viewport, Tiled LOD & atlas cache, Render quality, Bake image/tiles |
| **Assets** | Asset library, Sprite sheet slicer, Asset pack (validate/import/export), assets by domain |
| **View** | Panel visibility, workspace tabs, analysis field overlay, performance readout |
| **Help** | Credits & academic principles, references, keyboard map |

Disclosure inside a menu keeps the five-level grammar from
`Cartalith Menu Structure v2`: menu → category → section → sub-group →
advanced. A submenu arrow is level 3; a dialog's collapsed *Advanced* block is
level 5 and holds only dials whose defaults are already correct.

## Left tool rail

Grouped by what the tool touches, thin hairline separators between groups.

1. **Navigate & inspect** — Select/inspect `V`, Pan `H`, Point sample `I`
2. **Terrain** — Raise/lower `B`, Smooth `S`, Flatten/terrace `F`, Stamp (landform library)
3. **Water & ecology** — River/water `R`, Biome paint `P`
4. **Civilization** — Place settlement, Draw route/way, Territory/faction
5. **Annotation & measure** — Label `T`, Icon stamp, Measure `M`, Region select/export

Tool preferences pin to the bottom. Only one tool is active; the active tool
owns the tool options bar, the Properties panel, and the viewport cursor.

## Workspace tabs

WORLD · CIVILIZATION · INFRASTRUCTURE · CARTOGRAPHY · RENDER, under the menu
bar. A tab swaps which tools and dock panels are shown around the same viewport
— it never swaps the application, and never changes the map. This is where the
HTML app's left navigator went; expressed as workspace switching, it costs one
row instead of a permanent column.

## Editing model

- A tool's stroke goes into a **pass buffer** and is visible immediately.
- **Commit pass** writes it to the field; **Discard** drops the buffer.
- Downstream stages are marked stale rather than recomputed mid-stroke; the
  status bar names what is deferred (`rivers · deferred`).
- Undo granularity is one committed pass, not one stroke.
- Presentation-only controls (Render → Terrain appearance and everything under
  it) never mark a stage stale and never touch heightmap, climate, hydrology,
  biome classification, settlements, routes or seed.

## Touch targets

Windows is pointer-first: 32px rail icons, 26px status bar. Tablet and Android
keep the same six regions and the same tool grouping; only the target size and,
on phone, the rail's edge change.

**Tablet (2560×1600 landscape)** — full desktop parity. Menu bar, workspace
tabs, tool options bar and left rail all scale to 44–52px targets (rail icons
48px in a 64px column); the right dock widens to 400px so Layers, Properties
and Sample stay two-column at the larger type size. Nothing is dropped or
tucked into a sheet.

**Android phone (393×852 portrait)** — the left tool rail has no room as a
column, so it moves to a 64px bottom bar (52px icons, horizontally scrollable)
directly below the viewport; this is the one region allowed to relocate rather
than resize. Workspace tabs collapse to short labels in a single row under the
app bar. The tool options bar becomes a bottom sheet anchored above the tool
bar, open whenever a tool is active, with 44px controls — same fields as
desktop (hardness, intensity, raise/lower/smooth, commit/discard), stacked
instead of inlined. Layers/Properties/Sample are reached from the app bar's
panel icon as their own full-height sheet, one panel at a time; the five
disclosure levels survive unchanged inside it.

## Visual language

Dark neutral `#0d0e0f`, hairline rules at `rgba(255,255,255,.10)` and no panel
fills, one amber accent `#e0a34a` reserved for the active tool, the active
layer, and committed-action affordances. Numeric readouts in a monospace face;
labels in the UI sans. A light theme (`#f4f2ee` paper, `#a4650f` accent) maps
one-for-one. Nothing in the chrome competes with the map for saturation.
