# Cartalith — screens still needing a design

For Claude Design. 14 surfaces below still have no artboard anywhere: not in the DCC shell spec, not in a subsystem spec, not in the mockups. Each already carries comparable-application research from `GUI_GAP_REGISTER.md` §7 — use it, it was done against real applications.

## Read these first

- **`design/proposed-2026-09-05/`** (9 artboards, approved and built) and **`design/proposed-2026-09-05-round2/`** (13 artboards). These set the vocabulary. A new screen extends them; it does not invent a second language.
- **`cartalith-native/godot-project/shell/dcc_theme.gd`** is the token source of truth. Colours, roles and spacing resolve there, not in a canvas.
- **`DESIGN_HANDOFF.md`** — resolved tokens, frame geometry for all three shells, and the widget inventory a design has to map onto.

## Non-negotiables

1. **When two design canvases disagree, the newer one wins.** Where none exists, derive from the DCC canvases' own vocabulary. An owner decision is newer than any canvas.
2. **Three breakpoints, and they are not interchangeable:** desktop **1920** (and **1366**, which carries its own 3-token override), tablet **2560×1600** and **1600×2560** (a full 17-token touch density override), and phone. Draw the ones where the screen differs.
3. **Touch floor is 44 dp** (`DccTheme.PHONE_TAP_MIN`), scaled by `_phone_scale` — 115 px at 2.62. Anything tappable clears it.
4. **Never draw a fabricated value, and never draw a fabricated reason.** If a field has no engine source, draw it dashed and say what is missing. Three dashes with false stated causes shipped once and all three had to be corrected — a dash's reason is a claim.
5. **Say what backs every field.** A design that names a number the engine cannot produce becomes a build that fabricates one.
6. **Both palettes.** The shell ships light and dark; this machine boots light.

## The screens

### 1. Find on map
*Register: ED-05*

QGIS. The Locator bar in the status bar, Ctrl+K. Its defining idea is prefix filters: typing a short prefix scopes the search to one source — l project layers, f active-layer features, pl layouts, . actions, = calculator, b spatial bookmarks, set settings. Prefixes under three characters are reserved for core filters. Plugins register their own filters against the same bar. ([QGIS GUI docs](https://docs.qgis.org/3.44/en/docs/user_manual/introduction/qgis_gui.html), [QgsLocatorFilter](https://api.qgis.org/api/classQgsLocatorFilter.html))

**Comparables already researched:**
- QGIS. The Locator bar in the status bar, Ctrl+K.
- Blender. F3 opens a fuzzy operator search over every registered operator, context-scoped to the editor under the cursor.
- Fantasy-map tools. Azgaar's generator ships a plain name search over its burg/state/culture lists; Wonderdraft and Inkarnate have none — they rely on the label list panel.
- Proposal. Build QGIS's locator, not a modal dialog — the spec's ⌘F should focus a search field in the status bar, which is where §10 already puts "the two or three shortcuts that apply right now" and has the room.

### 2. Data manager ▸ Validation
*Register: DM-10, DM-11*

self-intersections, unclosed rings, wrong ring orientation. Results are a table, one error per row, with layer, id, error type, coordinates, a value, and a resolution column. Errors are selectable and fixable in bulk with a chosen resolution method.

**Comparables already researched:**
- QGIS ships two distinct validators, and the distinction is the design:
- Adopt QGIS's "click a row to zoom to it" — ViewportHost has real camera control, and it is the single feature that makes a validation table useful rather than a wall of text.

### 3. Colour management
*Register: PR-07*

Blender 4.x is the reference implementation for a creative app. Its Color Management panel exposes exactly four things: Display Device (sRGB, Display P3, Rec.1886), View Transform (Standard / AgX / Filmic / Raw / False Color), Look, and Exposure/Gamma. Blender 4.0 replaced Filmic with AgX as the default view transform and moved to an OCIO v2 config referenced to CIE XYZ. Notably, Blender still has no preference for choosing your own OCIO config — you replace the file. ([Blender 4.0 Color Management release notes](https://developer.blender.org/docs/release_notes/4.0/color_management/), [Blender Manual: Color Management](https://docs.blender.org/manual/en/4.0/render/color_management.html))

**Comparables already researched:**
- Blender 4.x is the reference implementation for a creative app.
- Blender 4.0 replaced Filmic with AgX as the default view transform and moved to an OCIO v2 config referenced to CIE XYZ.
- Notably, Blender still has no preference for choosing your own OCIO config — you replace the file.
- Do not ship the row as specified. Replace it with Blender's two-axis form, and ship only the half that is meaningful today:

### 4. Tiled LOD, tile size, atlas cache
*Register: PR-10*

Gaea 2 splits *what you build* from *how it is written*: Build Types (Normal / Split / Tiled), a tile size per tile (e.g. 1024 × 1024) and a blending percentage between adjacent tiles, all inside the Build dialog; the Build Manager is a separate persistent list of every node marked for export, with saved, organised, reusable build definitions. ([Gaea: Build Types](https://docs.quadspinner.com/Guide/Build/Build-Types.html), [Gaea: Tiled builds](https://docs.quadspinner.com/Guide/Build/Tiled.html), [Gaea: Build Manager](https://docs.quadspinner.com/Guide/Build/Manager.html))

**Comparables already researched:**
- Gaea 2 splits *what you build* from *how it is written*: Build Types (Normal / Split / Tiled), a tile size per tile (e.g.
- World Machine keeps a global Resolution slider whose maximum "depends upon the devices present in the world", and puts tiled output in a separate Tiled Build setup that writes a rectangular set of files for effectively unlimited extent.

### 5. Units
*Register: PR-15*

Blender puts units in Scene Properties ▸ Units: a Unit System (None/Metric/Imperial), a Unit Scale, and per-quantity overrides (Length, Mass, Rotation, Temperature). It is a scene property, not a preference — the file carries it.

**Comparables already researched:**
- Blender puts units in Scene Properties ▸ Units: a Unit System (None/Metric/Imperial), a Unit Scale, and per-quantity overrides (Length, Mass, Rotation, Temperature).
- QGIS has both, and the split is instructive: measurement units for the *measure tool* are an application Option; the *project's* display units for coordinates and areas are a Project Property.
- Follow Blender in naming the quantity, not the unit: the row reads Length: Kilometres / Miles, so adding area or temperature later does not need the row renamed.

### 6. Keyboard shortcuts editor
*Register: HE-02, PR-16*

Blender is the most complete implementation and the closest match to Cartalith's problem, because it has the same difficulty: the same chord means different things in different editors and modes. *Preferences ▸ Keymap* is a searchable tree — keymap ▸ editor context ▸ operator ▸ the individual binding, where expanding a binding exposes the full chord, its modifiers, the mouse button, and the operator's own properties. Searching a chord shows every context it is used in. Crucially, the everyday path is not the editor: right-clicking any menu item or button offers Assign Shortcut / Change Shortcut in place. ([Blender Keymap release notes](https://developer.blender.org/docs/release_notes/4.0/key

**Comparables already researched:**
- Blender is the most complete implementation and the closest match to Cartalith's problem, because it has the same difficulty: the same chord means different things in different editors and modes.
- Photoshop and DaVinci Resolve both ship a modal keyboard editor with a visual keyboard and a searchable command list, plus named preset sets that can be exported (Resolve ships "DaVinci Resolve", "Premiere Pro", "Final Cut Pro 7" and "Avid Media Composer" sets out of the box) — the killer feature for users migrating from another tool.
- That is exactly Blender's keymap-context tree, using containers the shell already has.
- Adopt Blender's conflict handling, not Photoshop's. Do not block a duplicate chord — show every other binding that already uses it, scoped by context, and let the user decide.

### 7. Save layout as…
*Register: WI-01*

Blender calls them Workspaces: named tabs across the top, each a complete screen layout of areas and editors, geared to a task (Layout, Modeling, Sculpting, Shading, Animation). They are saved in the .blend file, and the default set comes from the startup file. New workspaces are added from a template list or duplicated from the current one. ([Blender Manual: Workspaces](https://docs.blender.org/manual/en/latest/interface/window_system/workspaces.html))

**Comparables already researched:**
- Blender calls them Workspaces: named tabs across the top, each a complete screen layout of areas and editors, geared to a task (Layout, Modeling, Sculpting, Shading, Animation).
- Photoshop calls them Workspaces too (Window ▸ Workspace ▸ New Workspace…), and stores panel positions *plus* keyboard shortcuts and menus in the same preset — the one difference worth noting, because it bundles §7.9's output into the same object.
- Follow Blender in shipping defaults, not an empty list: pre-seed Generate, Sculpt, Cartography and Journey layouts matching how those four tasks actually want the docks.

### 8. Documentation and Report an issue
*Register: HE-01, HE-03*

There is no design and little to research: every comparable opens a URL. Blender's Help menu opens the manual, the Python API, and *Report a Bug* pre-filled with the system information. That last detail is the only one worth copying.

**Comparables already researched:**
- Blender's Help menu opens the manual, the Python API, and *Report a Bug* pre-filled with the system information.

### 9. Calculation trace window
*Register: JP-05*

No comparable in the map/DCC space; the closest are spreadsheet formula auditing (Excel's Evaluate Formula steps through a calculation one substitution at a time) and shader/node-graph inspectors (Blender's node editor showing intermediate outputs).

**Comparables already researched:**
- No comparable in the map/DCC space; the closest are spreadsheet formula auditing (Excel's Evaluate Formula steps through a calculation one substitution at a time) and shader/node-graph inspectors (Blender's node editor showing intermediate outputs).

### 10. Sculpt: brush shape, stroke & grid, actions
*Register: WW-03, WW-04, WW-05*

These are the register's only gaps the design *itself* labels new work (DCC_SHELL_SPEC.md correction #3), so the research is about scoping them honestly rather than filling a hole.

**Comparables already researched:**
- Blender sculpt brushes. The falloff is a curve widget mapped from brush centre (left) to border (right), with named presets (Smooth, Sphere, Root, Sharp, Linear, Constant) plus a custom curve.
- Blender 5.x converted brushes whose custom curve approximated smoothstep to a built-in "Smooth" preset — i.e.
- Krita draws the distinction Cartalith needs: a brush tip is "only a stamp of sorts"; a brush preset is a tip plus every other setting.
- The cheap, high-value half is Blender's falloff curve preset list — Smooth (what exists), Linear, Sharp, Constant — because each is a one-line change to that single formula and each visibly changes a ridge's profile.

### 11. Label font role
*Register: CA-07*

Wonderdraft is the closest comparable and its documented failure is instructive: labels support custom fonts and curved text along coastlines and rivers, but suffer "zoom vertigo" — a size that reads correctly at one zoom is unreadable at another, and *the same numeric size means different things on a 2048² map and an 8192² map*. Inkarnate avoids it by making label editing one-click and per-layer rather than by solving the scaling. ([Loreteller: Wonderdraft Labels — Fonts, Sizing and the Zoom Trap](https://loreteller.com/learn/wonderdraft-labels-guide/), [Loreteller: Inkarnate vs Wonderdraft](https://loreteller.com/learn/inkarnate-vs-wonderdraft/))

**Comparables already researched:**
- Wonderdraft is the closest comparable and its documented failure is instructive: labels support custom fonts and curved text along coastlines and rivers, but suffer "zoom vertigo" — a size that reads correctly at one zoom is unreadable at another, and *the same numeric size means different things on a 2048² map and an 8192² map*.
- Inkarnate avoids it by making label editing one-click and per-layer rather than by solving the scaling.
- Cartalith has already avoided half of it — MapLabel carries a size_mode of fixed / zoom, exposed in the dock, which is exactly the control Wonderdraft lacks.
- This is how every cartographic style system works (Mapbox styles, ArcGIS label classes), and it is the only version that survives export, where a CSS string is meaningless.

### 12. Style presets
*Register: CA-08*

Every comparable ships named looks and a "modified" indicator. Mapbox Studio's style gallery, ArcGIS Pro's basemap gallery, Affinity's adjustment presets, Blender's material previews. The universal pattern is: named presets in a gallery with a thumbnail, an accent outline on the active one, a custom — edited state the moment any field diverges, and Reset / Save as. DCC_SHELL_SPEC.md §4 already describes exactly this, which is why only the *content* is (C).

**Comparables already researched:**
- Every comparable ships named looks and a "modified" indicator. Mapbox Studio's style gallery, ArcGIS Pro's basemap gallery, Affinity's adjustment presets, Blender's material previews.

### 13. Layer list search; Blocks / Verticality — **no screen needed (2026-09-06)**
*Register: CA-09*

**Both halves are settled and neither needs an artboard.** The **search field** shipped into `layers_popover.gd` — one field over two bands, `VISIBLE LAYERS` and `DATA OVERLAYS`, drawn from the Find-on-map dialog's own vocabulary (field, band header, right-aligned count, one no-match sentence), so there was nothing new to draw. The **footer tabs** were ruled **not built** (`GUI_GAP_REGISTER.md` §7.16, reading C): both names describe something the shell already does — Verticality is `exag`, a live slider in CARTO ▸ Map style ▸ § Map view, and Blocks is either a 2.5D view this 2D port does not have or a name already taken (tiles are Preferences ▸ Tiles & LOD; style bundles are `STYLE_PRESETS`). A design would have been a pane of controls that already have two homes.

The original entry, kept because it is what the ruling answers — Blocks / Verticality is genuinely undefined, `DCC_CONTROL_INDEX.md` already marked it uncertain, and no comparable has footer tabs by those names. The two plausible readings, from the vocabulary of the field:

### 14. Rail expansion
*Register: SH-01*

Blender's collapsed/expanded sidebar and Photoshop's icon/expanded panel docks both expand to reveal *labels for the same items*, never new items. VS Code's activity bar expands to a per-view sidebar with different content per activity.

**Comparables already researched:**
- Blender's collapsed/expanded sidebar and Photoshop's icon/expanded panel docks both expand to reveal *labels for the same items*, never new items.
- VS Code's activity bar expands to a per-view sidebar with different content per activity.
- DCC_SHELL_SPEC.md §3 says the expanded rail shows *"the domain's sub-nodes as a 200 px list"*, which is the VS Code model — and, as DCC_CONTROL_INDEX.md records, the spec never enumerates the sub-nodes, so there is nothing to build.
- Proposal. Take the Blender/Photoshop reading instead, which is buildable today and loses nothing: expanding the rail shows each domain's full label plus its subtitle — both already in DccShell.DOMAINS and already used as the button tooltip — at 200 px.

## Not in the list, and why

- **Undo history panel, and what "global undo" covers** (ED-02, PR-11) — History artboard, round 1 — built and verified; the redo tail is being wired now
- **Data manager ▸ Sources** (DM-06) — DataManager artboard, round 2
- **Data manager ▸ Conversion** (DM-07, DM-08, DM-09) — Data ▸ Conversion removed by owner decision 2026-08-20

## One caveat on the count

`OUTSTANDING_WORK.md` still says "14 no-design surfaces remain". That row predates the second canvas entirely and is stale. The list above is derived from `GUI_GAP_REGISTER.md` §7 — the undesigned register, which carries the research — checked against both canvases. Artboards do not consistently cite register ids, so coverage was confirmed by reading rather than by grep alone, and two entries are excluded on that basis.