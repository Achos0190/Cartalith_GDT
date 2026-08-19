# GUI gap register — every disconnected control, its design, and where none exists

> Owner request, 2026-08-19, verbatim: *"verify that all GUI elements are tested,
> connected and where it doesn't connect to other menus or functions designs have
> been made to be implemented. If not, research the menu naming, documentation in
> the design, and where you still have gaps find references in similar
> applications."*

**The premise does not hold, and that is by design.** The shell does not have a
small number of stragglers to finish connecting: it has **123 catalogued
disconnected surfaces**, every one of them added *deliberately disabled with a
stated reason*, per the honesty rule `menus.gd`'s own header states —

> *an item with no engine behind it is added **disabled**, with a tooltip that
> says what is missing. It is never added enabled and silently inert, and never
> omitted — the menu is also the map of what the port still owes.*

So this document is the "if not" branch the request asks for. It is a **register**,
not a plan: it classifies every gap by whether a real design already exists, and
where none does, it researches how established applications solve the same problem
and proposes one.

It does not supersede `DCC_CONTROL_INDEX.md`. That document indexes the **design's**
452 controls against engine capability, and was written *before* the shell existed;
this one indexes the **shipped shell's** disclosed gaps against both the design and
the engine as they stand today, and it is the document that goes stale first.

## Contents

| § | What |
|---|---|
| [1](#1--method) | Method — what was read, and what "verified" means here |
| [2](#2--legend) | Legend: the A/B/C/D classification and the cost column |
| [3](#3--headline-counts) | Headline counts |
| [4](#4--stale-disclosed-reasons-five-fixed-in-this-pass) | Stale disclosed reasons — five fixed here |
| [5](#5--omissions-designed-not-present-not-even-as-a-disabled-item) | Omissions — designed, and not present at all |
| [6](#6--layer-1--2-the-catalogue) | **Layer 1 + 2 — the catalogue** (123 entries, classified) |
| [7](#7--layer-3--comparable-application-research-for-c) | **Layer 3 — comparable-application research** for every (C) |
| [8](#8--menu-naming-audit) | **Menu naming audit** |
| [9](#9--d-entries-owner-decisions-not-gaps) | (D) entries: owner decisions, not gaps |
| [10](#10--the-actionable-a-list-in-priority-order) | The actionable (A) list, in priority order |
| [11](#11--out-of-scope) | Out of scope for this register |
| [12](#12--verification) | Verification |

---

## 1 · Method

**Read in full, not grepped** — every file under
`cartalith-native/godot-project/shell/`: `menus.gd` (423), `app.gd` (553),
`dcc_shell.gd` (1 426), `right_dock.gd` (905), `dcc_settings.gd` (96),
`world_data_window.gd` (210), `data_manager_window.gd` (255),
`asset_library_window.gd` (925), `performance_window.gd` (78),
`layers_popover.gd` (181), `journey_planner_view.gd` (1 706),
`new_world_dialog.gd` (373), and every file under `workspaces/`
(`world_workspace.gd` 974, `civilization_workspace.gd` 929,
`infrastructure_workspace.gd` 417, `cartography_workspace.gd` 681,
`render_workspace.gd` 15, `workspace.gd` 38). `global_tools.gd`,
`viewport_host.gd`, `tool_overlay.gd`, `dcc_widgets.gd`, `dcc_theme.gd`,
`dcc_icons.gd`, `engine_bridge.gd` and `map_overlay.gd` were read for the
regions this register touches.

**Design sources read**: `DCC_SHELL_SPEC.md` in full (834 lines, including all
six correction blockquotes in its header — those are *deliberate divergences*
and are respected, never "fixed"), `DCC_CONTROL_INDEX.md` in full (1 093),
`JOURNEY_PLANNER_SPEC.md`, `TRAVEL_LIBRARY_SPEC.md`, `TIMELINE_SCOPE.md` §6,
`STRANDED_TOOLS.md`, `design/cartalith-menu-structure.md` (the owner's earlier,
superseded seven-menu structure — load-bearing for §8's naming audit), and
`cartalith-native/docs/STATUS.md`'s Known-open/Owner-only sections.

**Engine surface verified, not assumed.** The complete `#[func]` list was
enumerated from `cartalith-native/crates/cartalith-godot/src/` (15 modules:
`lib.rs` plus `civ_tools_bridge`, `icon_bridge`, `infra_tools_bridge`,
`journey_bridge`, `label_bridge`, `lod_bridge`, `pack`, `paint_bridge`,
`params`, `render`, `sample_bridge`, `sculpt_bridge`, `timeline_bridge`,
`travel_bridge`) — **151 methods**, up from the 38 `DCC_CONTROL_INDEX.md`
counted. Every (B) row below names the *specific* missing capability and was
checked by opening the crate, not inferred. Three claims that changed a
classification were read line-by-line: `cartalith_civ::jp_journey_cost`
(`cartalith-civ/src/lib.rs:6885`), `WorldGen::civ_faction_territory_stats` and
`WorldGen::get_factions` (`lib.rs:3442`), and `travel_bridge.rs`'s own
"What a later `#[func]` layer still needs to add" module doc.

**What "verified" means per row.** A row's *reason* is verified when the named
Rust item was opened and its presence or absence confirmed this pass. A row's
*design* is verified when the cited spec section was read. Anything not verified
says **uncertain** rather than guessing — there are six such rows and they are
marked.

`git show 595582d` (the 2026-08-19 GUI audit) was read first so its six fixes
are not re-reported. None of §4's five findings overlaps it.

---

## 2 · Legend

### Classification

| | Meaning |
|---|---|
| **(A) designed + engine-ready** | A design exists **and** the engine already exposes everything needed. Pure "someone should build the UI". No Rust at all. |
| **(B) designed but engine-blocked** | A design exists; the engine genuinely cannot back it yet. The row names the **specific** missing capability — a function, a crate, a `#[func]`. |
| **(C) undesigned** | No design exists anywhere: not in `DCC_SHELL_SPEC.md`, not in a subsystem spec, not in the mockups. A name in a spec table with no behaviour, no layout and no state model is **(C)**, not (A). These feed §7. |
| **(D) deliberate decision** | Not an oversight. Recorded, with where it is documented. **No design is proposed for these.** |

### Cost, for (B) only

(B) covers everything from a one-line dict field to a new subsystem, so it
carries a second axis. This is the register's most useful column for planning.

| Cost | Meaning |
|---|---|
| **wrapper** | The Rust exists and is tested; one `#[func]` (or one dict field) away. |
| **small** | Real Rust work, but bounded and already itemised in a scope document. |
| **large** | A subsystem. `DCC_CONTROL_INDEX.md` summary §2 sizes most of these. |

---

## 3 · Headline counts

**123 catalogued gap entries** (a group of identically-blocked sibling controls
— the ten Edit-menu items, the five erosion Run buttons — is one entry; the raw
count of individually disabled controls is ~180).

| Class | Count | Share |
|---|---:|---:|
| **(A)** designed + engine-ready | **17** | 14 % |
| **(B)** designed, engine-blocked | **71** | 58 % |
| **(C)** undesigned | **23** | 19 % |
| **(D)** deliberate decision | **12** | 10 % |
| **Total** | **123** | |

(B) by cost:

| Cost | Count | Notes |
|---|---:|---|
| **wrapper** | 22 | The single largest cheap win in the register. Nearly all of it is three subsystems: `TerrainAppearance` (RENDER + CARTO's LIGHT group), `AssetDB` (the whole Asset library window), and the Journey Planner's cost model. |
| **small** | 21 | |
| **large** | 28 | Dominated by five subsystems: the save writer, global undo + selection, the Data manager's import/conversion/validation routes, the colour-ramp/separable-layer system, and river-as-entity. |

**The shape.** Only 19 % of the shell's disclosed gaps are genuinely undesigned.
58 % have a design and are waiting on the engine — and **31 % of those (22 of 71)
are waiting on a boundary wrapper, not a capability**. That is the same finding
`DCC_CONTROL_INDEX.md` summary §1 reached from the other direction ("two whole
regions of this design are a boundary-wrapper problem, not a capability
problem"), now measured against the shipped shell rather than the design.

---

## 4 · Stale disclosed reasons: five fixed in this pass

Every `_todo()` tooltip, `_gap_button()` reason and `note()` in the shell was
re-checked against the engine as it stands today. Five were factually stale — a
lot of engine surface landed this session and the disclosures did not all move
with it. **Only the reason text was changed; no control changed state, and no
behaviour changed.** All five are corrections of fact, not design.

| # | File:line (pre-edit) | Was | Why it was wrong | Now |
|---|---|---|---|---|
| S1 | `right_dock.gd:608` | Faction ▸ Territory — *"no per-faction cell count or area query exists"* | **Two** now exist: `civ_faction_territory_stats(faction)` returns `claimed_cells`/`area_km2`/`contested_cells`, and `get_factions()` (`lib.rs:3442`) carries `claimed_cells` per faction. `civilization_workspace.gd:350-358` already flagged this in a comment ("true when that sentence was written, no longer true") but could not edit the file. | Says the queries exist, names both, points at CIVIL ▸ Territory's options row where the live numbers already show. |
| S2 | `app.gd:281-284` | CIVIL/INFRA idle context — *"the §4.5 tool palette to arm them is not built yet"* | The TOOLS block **is** built, in both docks (`civilization_workspace._build_tools()`, `infrastructure_workspace._build_tools()`). Both files say so in their own comments and note `app.gd` was out of their scope. | Names the tools each dock actually offers, and that POI is absent for a real engine reason. |
| S3 | `journey_planner_view.gd:1519` | Cost group — *"the reference's own cost model, if any exists past the HTML's own UI layer, has no Rust port"* | **False.** `cartalith_civ::jp_journey_cost` (`cartalith-civ/src/lib.rs:6885`, ported from `jpJourneyCost` reference line 18873) computes carriage/wages/crew/upkeep/tolls/transshipment/total/per-tonne-km/break-even, with golden tests (`journey_cost_prices_a_mixed_land_and_sea_trip`). It is simply never called. Every input is already computed inside `jp_plan` (`JpDerivedStage::claimed_frac`, `JpJourneyPlan::transshipments`, per-leg km/days/crew). | Says the model is ported, names it, names the three inputs, and calls it a boundary gap rather than a model gap. |
| S4 | `menus.gd:338` | Tiled LOD · tile size · atlas cache — *"No tile atlas yet."* | Stale in part. Deep-zoom LOD tiling is **live and automatic** (`lod_synthesize_tile`/`lod_tile_cells`, driven by `viewport_host.gd`'s `_lod_backlog`/`MAX_LOD_TILES_PER_UPDATE`). What does not exist is §2.5's *controls* and the *persistent* atlas. | Separates the two: tiling is live, the four preference rows and the on-disk cache are not. |
| S5 | `world_workspace.gd:292` | Finalize — *"cartalith-spatial exists standalone, unintegrated"* | Stale: `cartalith-spatial` gained real consumers on 2026-08-18 (`PassBuffer`/`StageGraph`, then LOD tiles). The bake/freeze half of the claim is still true. | Keeps the true half (nothing is written anywhere, so there is no atlas to freeze), drops the false half, cites `LOD_TILING_INTEGRATION_SCOPE.md`. |

### Borderline, deliberately not edited

- `right_dock.gd:674` Region select ▸ *"the Data Manager panel to call it doesn't exist yet"* — the Data manager **window** now exists, but the Export ▸ Maps **panel** genuinely does not. The wording says "panel". Accurate as written.
- `cartography_workspace.gd:277` *"no on-canvas resize handle yet for a placed icon (`icon_bridge.rs`'s own acknowledged gap)"* — `icon_resize`/`icon_hit_test` **are** exposed, so the attribution reads as more engine-blocked than it is; but `icon_bridge.rs:216` really does say *"`None` handle — no on-canvas resize-handle geometry"*, i.e. there is no `icon_handles()` to match `label_handles()`. The claim is true; only the emphasis is off. Left alone, recorded as entry **CA-05** below (an (A) item).
- `infrastructure_workspace.gd:13-14`'s class doc — *"Logistics … exports nothing past that crate boundary"* — is stale, but the same file's `_build_logistics()` says so explicitly two hundred lines later. A code comment, not user-facing text. Left alone.

---

## 5 · Omissions: designed, not present, not even as a disabled item

The honesty rule has two halves — *never enabled-and-inert*, and *never
omitted*. The first half holds everywhere. The second has **nine breaches**,
all of them designed surfaces that are simply absent, so a reader of the menus
cannot learn that the port owes them. Each is catalogued below with its class;
listed together here because they are a different kind of finding from a
disabled item.

| # | Missing surface | Designed in | Class |
|---|---|---|---|
| O1 | **`Data ▸ ⧉ Travel library… ⇧L`** — the whole menu item and window | `DCC_SHELL_SPEC.md` §2.4's 2026-08-19 addition; `TRAVEL_LIBRARY_SPEC.md` in full | (B) small |
| O2 | **`Assets ▸ Asset pack ▸`** — the entire submenu (Active pack / Pack metadata… / Edit / Batch / Build / Clear library…), 24 controls | `DCC_SHELL_SPEC.md` §2.3.1 | (B) wrapper |
| O3 | **`Preferences ▸ Performance ▸ Fallback when VRAM full`** | `DCC_SHELL_SPEC.md` §2.5 | (B) large |
| O4 | **`Preferences ▸ Application ▸ Theme ▸ follow system`** | `DCC_SHELL_SPEC.md` §2.5 | (A) |
| O5 | **`Window ▸` the workspace list**, and **open windows listed while open** | `DCC_SHELL_SPEC.md` §2.6 | (A) |
| O6 | **New world ▸ project *name* field** | `DCC_SHELL_SPEC.md` §2.1 ("Modal: name, seed, extent, working resolution") | (B) small |
| O7 | **The Journey Planner's timeline band** — "one band per day, coloured travel / water / weather hold / rest-layover". `timeline_bar` is *visible and empty* while JOURNEY is armed. | `JOURNEY_PLANNER_SPEC.md` §2 | (A) |
| O8 | **Blocked-stage inline resolutions** — "offers its resolutions inline (turn off closures, re-route land-only, depart earlier)" | `JOURNEY_PLANNER_SPEC.md` §9 | (A) |
| O9 | **The right dock's `Layers` context** — §6 lists eight contexts; seven are built, `Layers` is not (only the viewport popover and CARTO's toggles exist) | `DCC_SHELL_SPEC.md` §6 | (B) large |

Two more absences are **deliberate and documented in-file**, so they are not
breaches: the POI tool (`civilization_workspace.gd:94-101` — omitted rather than
built inert, because no `civ_drop_poi` exists) and the `Brush / Stamp` right-dock
context (`right_dock.gd:685-696` — merged into `Stamp stack` on the stated
ground that two views of one state would fight). Both are (D).

---

## 6 · Layer 1 + 2: the catalogue

Every disconnected surface, where it is, what it is called in the UI, its
current disclosed reason, whether that reason is still accurate, and its
classification with the design cited.

### 6.1 File menu — `menus.gd`

| # | UI label | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| FI-01 | Save project | 115 | no save writer (`cartalith-io` read-only) | yes | §2.1 | (B) large — `cartalith-io`'s only `zip::ZipWriter` is in `#[cfg(test)]`; `SAVEFILE_COMPAT.md` sets a byte-compatibility bar |
| FI-02 | Save as… | 116 | same | yes | §2.1 | (B) large — same writer |
| FI-03 | Autosave | 117 | requires a save writer | yes | §2.1 | (B) large — plus the default interval is owner policy |
| FI-04 | Revert to last save | 118 | requires a save writer | yes | §2.1 | (B) large — `load_save` + `sculpt_discard` are both real; only the writer's output is missing |
| FI-05 | Close project | 120 | no project lifecycle | yes | §2.1 | (B) small — needs a project entity `WorldGen` does not have |
| FI-06 | *(missing)* project name field | — | **none — omission O6** | — | §2.1 | (B) small — no name field on `WorldGen` or `cartalith_io::SaveData` |

### 6.2 Edit menu — `menus.gd`, all ten disabled

| # | UI label | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| ED-01 | Undo / Redo | 172-173 | no undo stack; generation one-shot, sculpt has no Godot binding | **partly stale in flavour, not in fact** — sculpt now has 34 bindings *and* draft-scoped undo/redo wired in `right_dock.gd`; what is absent is *global* undo. The sentence is still true of the global stack. | §2.2 | (B) large — `PassBuffer::undo` is draft-scoped and unlabelled; `FUNCTIONAL_CONTRACT.md` §12 calls global undo "absent entirely… necessarily new" |
| ED-02 | Undo history… | 174 | same | yes | §2.2 names it in one line; **no panel design exists** | **(C)** → §7.1 |
| ED-03 | Cut / Copy / Paste / Delete | 176-179 | nothing selectable beyond settlements, which are read-only | **stale**: labels, icons and sculpt stamps are now all individually selectable and deletable through their own panels. What is absent is a *uniform* selection model and a clipboard. | §2.2 | (B) large — one selection abstraction over `MapLabel`/`ManualIcon`/`NamedSettlement`/`SculptStamp` |
| ED-04 | Select all / Deselect | 181-182 | same | same | §2.2 | (B) large — same model |
| ED-05 | Find on map… | 184 | no search index; settlement search lives in the Data manager | yes | §2.2 gives one line; **no search UI design** | **(C)** → §7.2 |

> ED-01/ED-03/ED-04's reasons are stale in *emphasis* — they describe a shell
> that had no tools. They are not corrected here because rewriting them
> correctly means describing the global-undo/selection split, which is a
> paragraph, not a tooltip. Recorded rather than half-fixed.

### 6.3 Assets menu + Asset library window

| # | UI label | Where | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| AS-01 | Import image… | `menus.gd:201`, `asset_library_window.gd:402` | needs `AssetDB::addCustomSlot`, no `#[func]` | yes | §2.3, §8 | (B) wrapper — `raster::decode_png` + `AssetDB::add_item` are real |
| AS-02 | Apply library to map | `menus.gd:241`, `asset_library_window.gd:325` | only `load_asset_pack(path)` is exposed; no in-memory library session | yes (verified 2026-08-19) | §2.3, §8 | (B) wrapper — `apply_library_file_with_items` exists |
| AS-03 | Clear library… | `menus.gd:243`, `asset_library_window.gd:513` | no `AssetDB.clear()` equivalent exposed | yes | §2.3, §8 | (B) wrapper |
| AS-04 | Export pack .zip | `asset_library_window.gd:327` | `archive.rs::write_pack`/`zip_store` exist, no `#[func]` | yes | §8 | (B) wrapper — round-trip verified against a reference pack |
| AS-05 | Validate | `asset_library_window.gd:511` | `AssetValidator::run()` exists, not exposed | yes | §8 | (B) wrapper |
| AS-06 | Tag… / Collect… / Rename… / Duplicate / Delete (batch) | `asset_library_window.gd:436` | no engine call; `AssetDB`'s add/rename/remove/collection methods aren't exposed | yes | §8, §2.3.1 | (B) wrapper — note `rename` is engine-defined **only for custom slots** (a real spec/engine disagreement, `DCC_CONTROL_INDEX.md` §2.3.1) |
| AS-07 | Slot inspector: File / Scale / Tags / Pack metadata | `asset_library_window.gd:704-707` | need a live `AssetDB`/`PackInfo` query | yes | §8 | (B) wrapper |
| AS-08 | Per-slot fill state + thumbnails (grid is always a checkerboard) | `asset_library_window.gd:579, 690` | no `AssetDB` query exposed | yes | §8 | (B) wrapper — `filled_count`/`render_item` are real |
| AS-09 | Sprite-sheet **Slice** | `asset_library_window.gd:830` | `cartalith-assets` has no sheet-splitting function anywhere (`raster.rs`/`manifest.rs`/`archive.rs` checked) | yes | §8's slicer modal, in full | (B) small — no scope doc; `DCC_CONTROL_INDEX.md` summary §2 item 12 |
| AS-10 | Slicer: Trim transparent edges / Skip empty cells | `asset_library_window.gd:797-801` | no slice operation to apply them to | yes | §8 | (B) small — same |
| AS-11 | Slicer: Assign to family / Fill from | `asset_library_window.gd:810-813` | no in-memory library session for a result to land in | yes | §8 | (B) small — same |
| AS-12 | Family rail: **Collections** and **Unassigned imports** | *absent* | none | — | §8's rail lists both | (B) small — `AssetCollections` is real; "Unassigned imports" is a slot-less bucket the model does not have |
| AS-13 | **`Assets ▸ Asset pack ▸` submenu** (24 controls) | *absent — omission O2* | none | — | §2.3.1 in full | (B) wrapper — `DCC_CONTROL_INDEX.md` §2.3.1 scores it **19 backed-unwired against 1 engine gap** |
| AS-14 | Variants strip / "active variant" | *absent* | none | — | §8 | **(D)** — engine truth: variant choice at render time is weighted and seeded (`pick_weighted_variant`); a user-picked "active variant" has no counterpart. `DCC_CONTROL_INDEX.md` §3(f). |
| AS-15 | Per-slot Anchor (top/centre/base) | *absent* | none | — | §8 | **(D)** — engine truth: `Anchor` is a **family** property `sprite_draw_rect` depends on, not per-slot. §3(f). |
| AS-16 | 24-family rail vs the shipped 8 | `asset_library_window.gd:8-23, 360` | disclosed in the window's own note and header comment | yes | §8 says 24; mockup shows 11; engine has 8 | **(D)** — owner decision, `DCC_CONTROL_INDEX.md` summary §5 item 9 |

### 6.4 Data menu + Data manager window — `menus.gd`, `data_manager_window.gd`

All thirteen `"kind": "gap"` routes, plus the window's own foot and route pane.

| # | Route / control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| DM-01 | Import ▸ Maps · Heightmaps (PNG · TIFF) · GIS/GeoJSON | 47 | no image/heightmap/GeoJSON import anywhere in the workspace | yes | §2.4 names them; §9's pane is designed only for *Export ▸ Maps* | (B) large — no reader of any kind; TIFF is a new dependency decision |
| DM-02 | Export ▸ Maps (image · tiles) | 51 | `tile_render` draws per-tile PNGs; nothing assembles a Leaflet-style pyramid | yes | §9's route pane, **the one fully-designed route in the window** | (B) large — `region_export_tiles` is bound and tested; XYZ/TMS/WMTS addressing is new |
| DM-03 | Export ▸ GIS / GeoJSON | 53 | `cartalith-engine::geojson` exports region GeoJSON for Region-select only, no route in, no CRS | yes | §2.4 | (B) wrapper — `export_geojson` is golden-verified; needs one `#[func]` plus assembling `GeoJsonWorld` |
| DM-04 | Export ▸ World Data | 55 | no save writer | yes | §2.4 | (B) large — FI-01's writer |
| DM-05 | Export ▸ Assets (pack .zip) | 57 | routes to AS-04, itself disabled | yes | §2.4 | (B) wrapper — same as AS-04 |
| DM-06 | Sources ▸ External / Connected / Registry | 59-61 | no source registry exists | yes | §2.4 names three rows; **§9 designs no pane for any of them** | **(C)** → §7.3 |
| DM-07 | Conversion ▸ Coordinate Systems (EPSG ▸) | 62 | no CRS conversion; the engine works in one flat km projection | yes | §2.4 names it | **(D)** — owner decision, `DCC_CONTROL_INDEX.md` summary §5 item 8. Its pane is also (C); see §7.4 for what a CRS route would have to look like *if* the decision goes that way. |
| DM-08 | Conversion ▸ Format Conversion | 64 | no format-conversion routes | yes | **the spec itself leaves it undefined** — "which formats, to which" (`DCC_CONTROL_INDEX.md` §2.4) | **(C)** → §7.4 |
| DM-09 | Conversion ▸ Data Transformation | 65 | no data-transformation routes | yes | undefined in the spec | **(C)** → §7.4 |
| DM-10 | Validation ▸ Check Data | 66 | `load_save()` returns pass/fail only; no warning collection anywhere | yes | §2.4 names it ("shows current warning count"); what is validated, and against what invariant, is undefined | **(C)** → §7.5 |
| DM-11 | Validation ▸ Repair / Normalize | 68 | no validation pass to repair against | yes | undefined | **(C)** → §7.5 |
| DM-12 | Foot: "last run (`14:02 · 62 MB`)" | 160 | no export has run yet — said plainly rather than invented | yes | §9 | (B) small — needs a run-history store |
| DM-13 | §9's route pane: TILES / PROJECTION / LAYERS INCLUDED / OUTPUT / ESTIMATE / RECENT RUNS | *absent* | the pane shows the route's reason instead | n/a | §9, designed in full | (B) large — gated on DM-02 |
| DM-14 | §9's **MARKDOWN VAULT · LINKED** block | *absent* | — | — | §9 designs it; `MARKDOWN_VAULT_INTEGRATION.md` is explicitly *"Not started; no code exists"* and its §33 lists two-way sync as a V1 **non-goal** | **(D)** — owner decisions 3 and 4, `DCC_CONTROL_INDEX.md` summary §5 |
| DM-15 | **`Data ▸ ⧉ Travel library… ⇧L`** | *absent — omission O1* | none | — | §2.4's addition + `TRAVEL_LIBRARY_SPEC.md` in full (fields, validation states, placement, and its own §6 build-status) | (B) small — `cartalith-godot/src/travel_bridge.rs` holds the whole mutable store, CRUD, validation and usage tracking, with tests; `lib.rs`'s `WorldGen` has **no `travel_library` field and no `#[func]`**, and `jp_compute` calls `jp_plan` rather than `jp_plan_ex` with a resolver. That module's own doc names the exact wiring. |

### 6.5 Preferences menu — `menus.gd`

| # | UI label | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| PR-01 | Devices | 314 | multi-GPU device selection not exposed | yes | §2.5 designs a per-device checklist with live utilisation | (B) large — `cartalith_gpu::init_gpu()` requests **one** adapter; no enumeration, no utilisation, no dispatch partitioning |
| PR-02 | Multi-GPU mode | 315 | same | yes | §2.5 | **(D)** — owner decision 2 (`DCC_CONTROL_INDEX.md` summary §5): build it at all? |
| PR-03 | CPU worker threads | 316 | Rayon sizes its own pool; no override exposed | yes | §2.5 | (B) wrapper — one `ThreadPoolBuilder` call at startup; the *default* (cores − 4) is owner policy |
| PR-04 | VRAM budget | 317 | not exposed | yes | §2.5 | (B) large — no VRAM accounting anywhere |
| PR-05 | Fallback when VRAM full | *absent — omission O3* | none | — | §2.5 | (B) large — one of its three options is already what happens; the other two don't exist |
| PR-06 | Anti-aliasing · anisotropy | 333 | the 2D map path doesn't sample-antialias; belongs to the 3D viewport | yes | §2.5 | (B) large — gated on the 3D viewport |
| PR-07 | Colour management | 334 | the renderer is sRGB-only | yes | §2.5 gives **three values and nothing else** | **(C)** → §7.6 |
| PR-08 | 3D viewport defaults | 335 | no 3D viewport | yes | §2.5 names four fields | (B) large — `DECISIONS.md` §4 defers 3D; `ROADMAP.md` Phase 3 |
| PR-09 | Lighting rig defaults | 336 | no lighting rig yet | **stale in flavour**: there is no *rig*, but all six fields are real and drive the current render (`TerrainAppearance::{sun_az_deg, sun_alt_deg, relief_ambient, relief_gain, relief_lights, relief_directionality}`) | §2.5 | (B) **wrapper** — one `set_appearance()`-shaped `#[func]`; the same one CA-01 needs |
| PR-10 | Tiled LOD · tile size · atlas cache | 338 | **corrected — S4** | yes, now | §2.5 gives four rows of values | **(C)** for the atlas-cache design → §7.7 |
| PR-11 | Memory ▸ Undo history | 339 | no undo stack | yes | §2.5 gives a range and a default | **(C)** — depends on ED-02's undesigned model → §7.1 |
| PR-12 | Memory ▸ Clear caches… | 348 | no atlas or field cache exists to clear | yes | §2.5 | (B) small — gated on PR-10 |
| PR-13 | Theme ▸ Light | 362 | the light palette is defined (`DccTheme.LIGHT`) but styleboxes are built once at startup | yes | §2.5 + §11's full light token column | **(A)** — a rebuild pass in `DccTheme`/`DccShell`, no engine at all |
| PR-14 | Theme ▸ follow system | *absent — omission O4* | none | — | §2.5 | **(A)** — Godot exposes the OS preference; the rebuild pass is PR-13's |
| PR-15 | Units (km · mi) | 368 | the shell is km-only; the reference's mi toggle is not ported | yes | §2.5 gives two values, **and §5.1 stage 02 gives the same control a second home** — an unresolved ownership collision (`DCC_CONTROL_INDEX.md` §3(j), owner decision 15) | **(C)** → §7.8 |
| PR-16 | Keyboard shortcuts… | 369 | no shortcut table yet | yes | §2.5 says *"Editable table, per-context"* and nothing more | **(C)** → §7.9 |

### 6.6 Window menu — `menus.gd`

| # | UI label | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| WI-01 | Save layout as… | 407 | no layout store yet | yes | §2.6 names it | **(C)** → §7.10 |
| WI-02 | The workspace list | *absent — omission O5* | none | — | §2.6 | **(A)** — `_select_domain()` and `DOMAINS` already exist |
| WI-03 | Open windows listed while open | *absent — omission O5* | none | — | §2.6 | **(A)** — four windows exist and all are `AcceptDialog`s on `DccApp` |
| WI-04 | Dock width dragging (§1: "user-draggable within min/max") | *absent* | none | — | §1's geometry table gives min/max for both docks | **(A)** — pure GDScript; the collapse chevron already exists |

### 6.7 Help menu — `menus.gd`

| # | UI label | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| HE-01 | Documentation | 413 | no in-app documentation; the repository docs are the reference | yes | §2.7 names it | **(C)** → §7.11 |
| HE-02 | Keyboard shortcuts | 414 | no shortcut table | yes | §2.7 — and it duplicates PR-16 | **(C)** → §7.9 |
| HE-03 | Report an issue | 416 | no issue route wired | yes | §2.7 | **(C)** → §7.11; the *destination* is an owner decision |

### 6.8 Right dock — `right_dock.gd`

| # | Context ▸ field | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| RD-01 | Settlement ▸ Defensibility | 436 | `explain_settlement()` has no defensibility axis | yes | §6 lists it | (B) small — the reference itself treats it as a UI-only categorical (`PHASE2_SCOPE.md` m18) |
| RD-02 | Settlement ▸ Routes | 440 | `get_roads()`/`get_sea_routes()` are plain polylines with no settlement index | yes | §6 | (B) small — needs a settlement↔way association the engine does not model |
| RD-03 | Settlement ▸ **Economy / Politics / Logistics** buttons | 447-449 | "No per-settlement *x* panel exists yet — see Data ▸ World data tables" | **accurate but obsolete as a design position** | §6 names all three | **(A) — done 2026-08-19**: Economy → `app.open_world_data("Economy")` (new `WorldDataWindow.open(tab)`); Politics → `show_faction()`; Logistics → `app.open_journey_planner()`. Verified live via a scripted headless drive (`STATUS.md`/`CHANGELOG.md`). |
| RD-04 | Settlement ▸ government / agriculture | *absent* | none | — | §6 lists both | **(D)** — confirmed UI-only categorical labels with zero derived computation in the reference (`PHASE2_SCOPE.md` m18); adding them would be inventing data |
| RD-05 | **River** context (7 fields + 3 actions, and no way to reach it) | 571-586 | no hydrological river entity crosses the boundary; no `get_rivers()`, nothing selectable | yes | §6 designs the whole context | (B) large — rivers are a per-cell network, not entities with ids/names/catchments; `DCC_CONTROL_INDEX.md` summary §2 item 14 |
| RD-06 | Faction ▸ Territory | 608 | **corrected — S1** | yes, now | §6, §4.5.3 | **(A) — done 2026-08-19**: reads `civ_faction_territory_stats(faction)` live, same call/format `civilization_workspace.gd`'s Territory tool-options row uses. |
| RD-07 | Faction ▸ State religion | 611 | `has_religion` computed internally; `get_provinces()` doesn't carry it and there is no `get_faction_aggregates()` | yes — `get_factions()` carries id/culture/colour/settlement_count/claimed_cells, no religion | §6 | (B) wrapper — `civ_faction_aggregates` is golden-verified and unexposed |
| RD-08 | Faction ▸ Roster | 604 | reads province names only | n/a — works, but ignores `get_factions()`'s richer row | §6 | **(A) — done 2026-08-19**: reads `get_factions()` for Culture, a colour swatch, and Settlements; Provinces (count) kept separately. |
| RD-09 | Region select ▸ Send to Data ▸ Export | 672-674 | `region_export_tiles()` is bound and tested; the Data Manager panel to call it doesn't exist | yes | §4.5.1 + §9 | (B) large — gated on DM-02's route pane. *Cheap path*: call `region_export_tiles()` straight to a `FileDialog` save, which is (A); that is a design decision, not a fix. |
| RD-10 | **`Layers` context** | *absent — omission O9* | none | — | §6 designs it (ordered list, visibility dot, opacity bar, blend mode, nested children under Terrain) | (B) large — opacity is cheap (overlays carry alpha); blend mode and reorder need the three overlays to become independently compositable, an architecture change `GUI_FEATURE_PARITY_SCOPE.md` Category 3 already recommended deferring |
| RD-11 | Collapsed right dock's primary readout | — | none | — | §6's last line: *"elevation for Sample, layer dots for Layers, stamp count for the stack"*. `DccShell.set_dock_readout("right", …)` exists and **`right_dock.gd` never calls it** — the left dock's is wired (`world_workspace._push_dock_readout`), the right dock's is not | **(A) — done 2026-08-19**: `_push_dock_readout()` called at the end of `_rebuild()` and live from `on_cursor_sampled`; one real reading per existing context (elevation, settlement name, faction id+culture, route length, chain/region/stamp counts, journey days·km). No "Layers" context exists yet (RD-10). |
| RD-12 | `Brush / Stamp` context | 685-696 | merged into `Stamp stack`, with the reasoning stated in-file | yes | §6 lists both | **(D)** — deliberate: both read the same live state and the eight globals already have live editors in WORLD's dock |
| RD-13 | Stamp stack ▸ finalize-lock note | 731-737 | no finalize/lock state exists in this engine | yes | §6 | (B) large — gated on WW-01 |

### 6.9 Journey planner — `journey_planner_view.gd`

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| JP-01 | Carriage **Auto** pick | 366 | `jpAutoPickTransport` has no Rust port | yes | `JOURNEY_PLANNER_SPEC.md` §5 ("in auto, counts are computed (terrain × biome, km-weighted) and read-only") | (B) small — a real, bounded port of one reference function |
| JP-02 | Party preset | 394, 1315 | `JP_PRESETS` is JS-only; no `jp_presets()` binding | **stale**: `TRAVEL_LIBRARY_SPEC.md` §3.4 designs "Party set-ups" as exactly this, and `cartalith_civ::travel_library::stock_party_presets()` plus `travel_bridge.rs`'s `PartyPreset` CRUD are **built and tested** | §5 + `TRAVEL_LIBRARY_SPEC.md` §3.4 | (B) small — same `#[func]` layer DM-15 needs; not corrected in-place because the honest correction is "see the Travel library", which does not exist as a surface yet |
| JP-03 | Re-route for `<mode>`… | 1320 | `jpAutoPickTransport`/`_jpRerouteForMode` have no Rust port | yes | §6's "faster-mode advisories… with a **use here** action" | (B) small — sibling of JP-01 |
| JP-04 | **Cost** group | 1519 | **corrected — S3** | yes, now | §8 designs it in full (food/fodder · wages · tolls/ferry · animal upkeep · total · per km and per day) | (B) **wrapper** — `jp_journey_cost` is ported and golden-tested; `jp_compute` never calls it. **The single cheapest (B) in the register.** |
| JP-05 | Calculation trace ⧉ | 1553-1555 | no trace window; the `formula` string is deliberately not carried across the boundary (`jp_land_calc_dict`'s own doc: presentation, not engine) | yes | §8 says *"opens in its own window (⧉)"* and nothing about its contents | **(C)** → §7.12 |
| JP-06 | Save journey | 1325 | no save-writer for journeys or projects | yes | §2 lists it in the tool options bar | (B) large — FI-01's writer, plus a journey registry that does not exist |
| JP-07 | ⇧-drag spine trim | 1323 | `jp_compute` has no request field for trimming | yes | §3: *"⇧ drag trims"* | (B) small — a new `jp_compute` request field plus its handling |
| JP-08 | Journeys list = committed routes | 226, 250 | no named/persisted journey registry exists engine-side | yes | §3's "journeys list" | (B) large — same registry as JP-06 |
| JP-09 | Vessels ▸ sailing window | 1540 | not part of `jp_water_calc`'s return | yes | §8: *"per water leg: vessel, hold used, sailing window"* | (B) small — `TRAVEL_LIBRARY_SPEC.md` §3.3 models "sailing window (daylight / continuous)" per vessel; the resolver for vessels does not exist |
| JP-10 | Supply ▸ foraging offset | 1515 | folded into food/water totals; `jp_plan` doesn't break it out | yes | §8 lists it as its own figure | (B) small |
| JP-11 | Load ▸ speed penalty | 1500 | folded into each leg's km/day; `jp_plan` returns no separate percentage | yes | §8 | (B) small |
| JP-12 | Supply ▸ per-leg bar with resupply ticks | *absent* | none | — | §8 | **(A)** — `resupply_reach` already carries `max_gap_km`/`required_km`/`stops`/`unmet`, and `_bar()` exists in this file |
| JP-13 | **Timeline band** (one band per day) | *absent — omission O7* | none — and `timeline_bar` is *visible and empty* in INFRA while JOURNEY is armed | — | `JOURNEY_PLANNER_SPEC.md` §2 | **(A)** — `plan.stages[i].km` + `results[i].days` + `rest_days`/`layover_days` are all in hand; `DccShell.timeline_row` is a live container |
| JP-14 | **Blocked-stage inline resolutions** | *absent — omission O8* | none | — | §9: *"offers its resolutions inline (turn off closures, re-route land-only, depart earlier)"* | **(A)** — all three are `_plan_values` edits plus `_compute()`; no engine work |
| JP-15 | Auto fields showing `auto · <resolved>` | partial | none | — | §5: *"Auto-valued fields show `auto · <resolved value>` so the resolved value is never hidden"* — implemented for stage overrides (`_inherit_label`), **not** for the party form | **(A)** — `_last_result`'s `eff` dict already carries the resolved values |

### 6.10 WORLD workspace — `world_workspace.gd`

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| WW-01 | Finalize · LOD 0–3 · bake & freeze | 290-292 | **corrected — S5** | yes, now | §5.1's dock foot, §4's tool options bar (`app.gd:316-318` carries a second copy) | (B) large — no bake, no atlas write, no finalize-lock state |
| WW-02 | Run Droplet hydraulic / Hillslope diffuse / Velocity / Glacial / Coastal (5) | 368-373 | not ported; a separate manual pass in the reference with no `cartalith-engine` equivalent | yes | §5.1 stage 06 | (B) small ×5 — each itemised in `GENERATION_PARAMETERS.md`'s own "parameters the reference exposed that this port does not" (5, 2, 3, 4, 4 parameters respectively) |
| WW-03 | Sculpt ▸ **Brush shape** (8 falloff shapes, Import brush…, Operation, Falloff curves, Rotation) | 665-672 | no engine behind it, and **not in the reference either** | yes | `DCC_SHELL_SPEC.md`'s own header **correction #3**: *"New design work, not a port gap"* | **(C)** → §7.13 |
| WW-04 | Sculpt ▸ **Stroke & grid** (Add point / Duplicate / Rotate / Scale / Tilt / Push / Pull / Align) | 665-672 | same | yes | correction #3; `DCC_CONTROL_INDEX.md` §5.2 adds that it rests on a **"control grid" concept that exists nowhere** and cannot be sized until defined | **(C)** → §7.13 |
| WW-05 | Sculpt ▸ **Actions** (Flip X/Y, Rot L/R, Flatten) | 665-672 | same | yes | correction #3 | **(C)** → §7.13 |
| WW-06 | Paint ▸ Hardness / Softness | 860-863 | stored and echoed back but never consumed — painting is a hard disc with no soft falloff | yes | §4.5.2 lists both | (B) small — `paint_bridge.rs`'s own module doc |
| WW-07 | Stage 01 ▸ geoid sea level, tides (moon mass/distance/k₂) | 68 | default-off reference sub-systems with no `cartalith-engine` equivalent | yes | §5.1 stage 01 | (B) small ×2 |
| WW-08 | Stage 07 ▸ min stream order, lakes as water | 90 | reference **render** filters, not generation parameters — Cartography's work | yes | §5.1 stage 07 | (B) small — and `DCC_CONTROL_INDEX.md` marks "lakes as water" **uncertain** (classification switch or display switch?) |
| WW-09 | Stage 08 ▸ seasons & Köppen | 94 | not ported | yes | §5.1 stage 08 | (B) small |
| WW-10 | Stages 09 / 10 have no dials | 96-102 | not parameterised — biome classification runs off finished fields; no soil/ore/fertility dials exist in `cartalith-engine` | yes | §5.1 | **(D)** — engine truth, not a gap. Surfacing the *rasters* is a retention-vs-memory decision `MEMORY_OPTIMIZATION_SCOPE.md` already paid to avoid. |
| WW-11 | Per-stage `Run stage n` / `Run n → 10` / stale dots / `04 / 10` counter | *absent* | the dock's own "Not a generation stage" note and `app.gd:298-306` explain why | yes | §5.1 and §4 both design it | **(D)** — `DCC_SHELL_SPEC.md` header **correction #2**: verified by Playwright against the real reference; the capability exists **nowhere**, not in this engine and not in the app being ported. Building disabled buttons for it was rejected as clutter. |

### 6.11 CIVIL workspace — `civilization_workspace.gd`

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| CV-01 | **POI tool** | 94-101 (comment) | omitted, not built inert: `civ_tools_bridge.rs` says POI *"is not a ported concept"*; no Rust function drops one | yes | §4.5.3 designs it in full (kind · faction · name · snap to way, plus a POI inspector) | (B) small — one `civ_drop_poi` mirroring `civ_drop_settlement`; `cartalith-assets`' `poi` family already carries the 10-slot vocabulary |
| CV-02 | Culture ▸ Profiles | 518-523 | `cartalith-civ` generates culture profiles internally; no `#[func]` exports them | yes | §3 lists Culture as one of CIVIL's five subjects | (B) wrapper — `civ_default_culture` is already called inside `get_factions()`; a fuller `get_cultures()` is one binding |
| CV-03 | Timeline filters (Exist only / Ghost removed / Highlight new) can't touch map pins | 821-827 | `get_settlements()` carries no `tid` even though `NamedSettlement` has one | yes — already in `STATUS.md` Known-open | `TIMELINE_SCOPE.md` m6 | (B) small — add `tid` to `get_settlements()`'s dict, or a per-year snapshot getter |
| CV-04 | Settlement class list lacks **metropolis** | 233-239 (comment) | `civ_tools_bridge::kind_from_str` accepts exactly the five real `SettlementKind` tiers | yes | §4.5.3 lists six | **(D)** — `TIMELINE_SCOPE.md` §6: `_civSelectMetropolises` is *"a pre-existing, separately-scoped gap"*, explicitly out of scope for every Timeline milestone |
| CV-05 | Territory ▸ "respect coastlines" | 298-304 (comment) | `civ_territory_paint_at` always pushes an ungated circular dab (`PaintStamp::ungated`); no coastline mask behind it | yes | §4.5.3 | (B) small |
| CV-06 | Settlement ▸ "pick radius" | 236-239 (comment) | `civ_drop_settlement` computes its own pick radius internally and takes no argument | yes | §4.5.3 lists it | **(D)** — engine truth; a slider would be decoration |
| CV-07 | Faction roster add/remove, persistent identity | *absent* | none | — | §6's Faction context implies a roster; `design/cartalith-menu-structure.md` §3.11 names "add/remove faction, faction roster `#civOpenFactionsBtn`" | (B) large — new Rust state; `CIV_FACTION_COUNT` is a constant |
| CV-08 | `_civApplyRecovery` / auto-populate's static "Recovery phase" | *absent* | none | — | `design/cartalith-menu-structure.md` §4 names it | **(D)** — `TIMELINE_SCOPE.md` §6: *"adjacent, see §3 point 5. Its own scoping (if any) belongs to `PHASE2_SCOPE.md`, not here"* |
| CV-09 | The timeline bar's **six simulation-layer toggles** (Climate · Population · Economy · Politics · Infrastructure · Warfare) | `dcc_shell.gd:628-641` builds an empty `timeline_row` | none in-product — `TIMELINE_SCOPE.md` §4 explains why the bar was left untouched | yes | §10 designs the whole region | **(D)** — `DCC_CONTROL_INDEX.md` summary §5 item 5 and `VISION.md`: the engine is a one-shot static generator by explicit, repeated owner decision. **The bar is drawn and empty in CIVIL/INFRA** — see §11. |

### 6.12 INFRA workspace — `infrastructure_workspace.gd`

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| IN-01 | Rivers ▸ Hydrology | 314-319 | no `get_rivers()`; the only river output crossing the boundary is baked into the rendered raster | yes | §3 lists Rivers as one of INFRA's five subjects | (B) large — same entity gap as RD-05 |
| IN-02 | Committed manual ways/routes never appear on the map or in a list | 20-31 (class doc), 195, 213 | `get_roads()`/`get_sea_routes()` read `civ.ways`/`civ.sea_routes` only, never `infra.ways`/`infra.routes`; `way_commit`'s own doc says the getter is out of scope | yes | §4.5.4's "Way inspector: waypoint list, length, grade profile, surface" | (B) small — one getter mirroring `route_get`'s shape |
| IN-03 | Way / Route ↶ ↷ (per-waypoint undo) | 232-236 (comment) | no per-waypoint undo in the engine; `InfraTools` only discards the whole draft | yes | §4.5.4 lists ↶ ↷ | (B) small |
| IN-04 | Way ▸ routing mode (freehand / snap / least-cost) | 229-231 (comment) | `infra_tools_bridge`'s own doc: *"nothing to build a 'freehand' or distinct 'snap' routing mode out of"*; snap is real but automatic | yes | §4.5.4 | **(D)** — engine truth, recorded in-file |
| IN-05 | Way types: spec says road/track/trail/bridge, engine has road/track/sea_lane/ancient | 42-49 (comment) | `parse_way_type`'s own doc calls the spec list wrong against the tested four-entry enum | yes | §4.5.4 | **(D)** — spec/engine disagreement, resolved in the engine's favour and recorded |
| IN-06 | Route ▸ vessel / party reference in the options row | 252-256 (comment) | the journey planner exported nothing past the crate boundary when written | **stale**: `jp_options()` now returns the vessel and transport vocabularies, and `TRAVEL_LIBRARY_SPEC.md` §3.3 designs the vessel definitions | §4.5.4 | (B) small — gated on DM-15's `#[func]` layer |
| IN-07 | Trade ▸ route assignment | 370-373 | nothing ties a trade relationship to the road or sea lane that would carry it | yes | §3 lists Trade | (B) large |

### 6.13 CARTO workspace — `cartography_workspace.gd`

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| CA-01 | Layer properties ▸ **LIGHT** (azimuth · elevation · strength · multidirectional) | 110-114 | `TerrainAppearance` is implemented and settable in Rust but bound to no GDExtension method | yes | §7's LIGHT group | (B) **wrapper** — one `set_appearance()`; the same one PR-09 and RN-01 need. `DCC_CONTROL_INDEX.md` §3(g) flags that "Strength" is **uncertain** between `relief_gain` and `relief_directionality`. |
| CA-02 | Layer properties ▸ **FILL** (colour ramp picker, domain, range) + the **Stop editor** | 110-114 | same note | yes | §7 designs nine named ramps, a popover, and a full stop editor | (B) large — `render.rs`'s own module doc: *"there is no elevation-keyed colour breakpoint ramp anywhere in this renderer."* |
| CA-03 | Terrain sub-layer visibility (Hand-drawn hillshade / Hillshade / Colour relief) | 105-107 | terrain, hillshade and colour relief are one baked raster, so they toggle with the map | yes | §7's ten-row layer stack | (B) large — needs the single colour pass to become separable outputs |
| CA-04 | Layer opacity / blend mode / reorder | *absent* | none in this file (`layers_popover.gd` has a *debug-view* opacity slider, a different thing) | — | §6's Layers context, §7 | (B) — opacity is **wrapper** (overlays already carry alpha); blend/reorder is **large** |
| CA-05 | Icon ▸ on-canvas resize handle | 277-279 | *"no on-canvas resize handle yet… (`icon_bridge.rs`'s own acknowledged gap)"* | true, but the attribution understates what is available — `icon_resize`, `icon_hit_test` and `icon_get` are all exposed; what is missing is `icon_handles()`, the equivalent of `label_handles()`, which `icon_bridge.rs:216` names explicitly | §4.5.5 | **(A)** — the drag math already exists on the Label tool (`_begin_label_handle_drag`); handle geometry can be derived GDScript-side from `icon_get()`, as the reference itself does |
| CA-06 | Label ▸ letter-spacing, anchor | 643-648 | no backing field on `MapLabel` (`label_bridge.rs`'s own "Not modelled" note) | yes | §4.5.5's tool options row lists both | (B) small |
| CA-07 | Label ▸ font (the stored CSS string doesn't render) | 643-648 | Godot has no web-font fallback chain, so only size/angle/arc/colour render | yes | §4.5.5 says "font role" — **a role, not a CSS string** | **(C)** → §7.14 |
| CA-08 | Style presets (Atlas / Parchment / Physical / Ink) + `custom — edited since preset` + Reset + Save preset | *absent* | none | — | §4's Cartography row | **(C)** for three of four → §7.15. `TerrainAppearance::default()` plausibly *is* "Atlas"; Parchment/Physical/Ink are new looks, and `TerrainAppearance` doesn't derive `Serialize`. |
| CA-09 | Layer list ▸ search field; footer tabs **Blocks / Verticality** | *absent* | none | — | §7 names them | **(C)** — `DCC_CONTROL_INDEX.md` marks Blocks/Verticality **uncertain**: *"undefined in the spec beyond the two words"* → §7.16 |
| CA-10 | Layer properties ▸ **Visualization dropdown** | *absent* here; `layers_popover.gd` covers it with 18 debug views | the popover's own footer explains the split | yes | §7 lists it; §10's popover overlaps it | **(D)** — deliberately resolved as one popover rather than two competing pickers (`layers_popover.gd:10-15`) |

### 6.14 RENDER workspace — `render_workspace.gd`

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| RN-01 | The whole domain — Terrain appearance groups | 14-15 | `render.rs`'s `TerrainAppearance` is real but unbound; until it is, Preferences ▸ Render quality is the only live control | yes | §3 gives RENDER a dock; `design/cartalith-menu-structure.md` §5b designs the full subsystem (Preset · Colour relief · Colour · Material · Relief · Detail · Atmosphere · Preview · Quality) | (B) **wrapper** — ~40 real, tested fields driving the current render, reachable through **no `#[func]` at all**. The single largest cheap surface in the shell. |

### 6.15 Frame, viewport and phone — `dcc_shell.gd`, `viewport_host.gd`, `layers_popover.gd`

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| SH-01 | Rail expansion `›` → 200 px sub-node list | `dcc_shell.gd:333-337` (a bare `Label`, never wired) | none | — | §3 names it | **(C)** — `DCC_CONTROL_INDEX.md`: *"Sub-node lists per domain are not enumerated in the spec; the builder has no source for them"* → §7.17 |
| SH-02 | Phone: tool-sheet drag, gesture-inset handle | 1056-1058, 1099-1101 | *"the mockup pictures exactly one static sheet state; nothing here answers a drag gesture"* | yes | §13 | **(D)** — deliberate: inventing a gesture the design does not show |
| SH-03 | Phone: touch-pan-while-drawing (v2.10 `#sculptNavpad`) | 710-714 | `main.gd` carries no such handling to port forward — grepped | yes | §4.5.6 requires it | (B) small — a genuine gap for whoever wires sculpt touch input |
| SH-04 | Phone: battery / signal glyphs | 863-868 | checked against this Godot build's own `OS` class: no `power`/`battery` method exists | yes | §13's mockup | **(D)** — nothing real to back them cross-platform; only the clock gets real data |
| SH-05 | Layers popover: hotkey badges 1–8 | *absent* | none | — | §10: *"grouped rows with hotkey badges"* | **(A)** — the popover already enumerates every view from `debug_layers()`; badges plus `InputMap` entries are pure GDScript |
| SH-06 | Viewport ▸ `→ 1 582 m` (draft-stamp elevation under the cursor) | *absent* | none | — | §10 | **(A)** — `build_sculpt_preview_texture` composites the draft; `sample_cell` reads the live field |
| SH-07 | Status bar ▸ `autosave` and `atlas` slots | `dcc_shell.gd:657` builds both; nothing writes them | none | — | §10's middle group | (B) small — gated on FI-03 and PR-10 respectively |
| SH-08 | Menu accelerators for the disabled items (⌘S ⌘⇧S ⌘W ⌘Z ⌘⇧Z ⌘X ⌘C ⌘V ⌫ ⌘A ⌘D ⌘F ⌘⇧P) | `menus.gd` sets only `Ctrl+N`, `Ctrl+O`, `⇧A`, `⇧J` | none | — | §2's tables give every one | **(D)** — an accelerator on a permanently disabled item is dead weight; they arrive with their items |

---

## 7 · Layer 3 — comparable-application research for (C)

Every (C) entry, with how established applications in the same space actually
solve the problem, what they call it, where it sits in their information
architecture, and a proposal concrete enough to build from without re-searching.
Sources are linked so the research is checkable rather than asserted.

### 7.1 Undo history panel, and what "global undo" covers — ED-02, PR-11

**Photoshop.** *History* panel (Window ▸ History). Default **20 states**,
raisable to **1 000** under *Preferences ▸ Performance*. Two features matter
architecturally: **Snapshots** (unlimited, never evicted, taken explicitly with
the camera button — a named point you can always return to) and **Allow
Non-Linear History**, an opt-in that lets you branch from an earlier state
without discarding the states after it. Closing the document clears everything.
([Adobe: History panel overview](https://helpx.adobe.com/photoshop/desktop/get-started/set-up-toolbars-panels/history-panel-overview.html),
[Adobe: Use snapshots](https://helpx.adobe.com/photoshop/desktop/get-started/set-up-toolbars-panels/create-work-snapshots.html))

**Blender.** Undo is global and typed, and the *Adjust Last Operation* panel is
its most-copied idea: instead of undo/redo cycling, the last operator's
parameters stay editable in place, so tuning replaces re-doing.

**Krita / Affinity.** Both ship a linear history docker; Affinity persists
history *into the document* if you opt in, which is the model closest to a
world file that already carries a stamp stack.

**Why this is (C) and not (A).** `DCC_SHELL_SPEC.md` §2.2 names the panel in one
line and §2.5 gives it a depth default (5). Neither says *what a state is* in an
application whose edits span generation parameters, sculpt stamps, paint dabs,
territory paint, labels, icons, settlements and manual ways — seven undo domains
with three completely different commit models.

**Proposal for Cartalith.**

1. **Do not build one global stack.** Build a **history ledger**: an append-only
   list of *named* entries, each recording which subsystem it belongs to and how
   to reverse it. The engine already has the reversal primitives per subsystem
   (`PassBuffer::undo`, `paint_discard`, `civ_territory_discard`,
   `label_delete`, `icon_delete`, `param_set` with the previous value). What is
   missing is the ledger, not the reversals.
2. **Two tiers, following Photoshop's snapshot/state split, mapped onto this
   engine's own draft/commit split** — which is the natural seam and costs
   nothing to invent:
   - **Draft steps** — everything inside an uncommitted draft (stamps, dabs,
     territory strokes). Already reversible, already scoped, already capped.
     These are Photoshop's *states*.
   - **Commits** — `sculpt_commit`, `paint_commit`, `civ_territory_commit`,
     a settlement drop, a way commit, a generate. These are Photoshop's
     *snapshots*: fewer, named, and the only ones worth surfacing in a panel.
3. **The panel** (`Edit ▸ Undo history…`) is a right-dock context, not a
   window — it is selection-adjacent and the dock is already context-driven.
   One row per commit: an icon for the subsystem, the operation name, and the
   affected extent in cells. Clicking a row rolls back to it. Adopt Photoshop's
   **linear default**, and do **not** ship non-linear history: it is a documented
   source of user confusion and this engine has no cheap way to re-apply
   divergent branches over a regenerated world.
4. **`Preferences ▸ Memory ▸ Undo history`** then means what Photoshop's
   Performance setting means — a cap on *draft steps*, not commits. Keep the
   reference's own cap of 30 rather than the spec's 5, and say in the tooltip
   that commits are uncapped. Show the live memory cost next to it, the way
   Photoshop's own preference warns about it.
5. **Adopt Adjust Last Operation.** After any generation-parameter change the
   shell already regenerates on release; a small "last change: `tect.plates`
   14 → 16 · revert" chip in the status bar gives 90 % of undo's value for
   generation parameters at a fraction of the cost, and is (A) today.

### 7.2 Find on map — ED-05

**QGIS.** The **Locator bar** in the status bar, `Ctrl+K`. Its defining idea is
**prefix filters**: typing a short prefix scopes the search to one source —
`l` project layers, `f` active-layer features, `pl` layouts, `.` actions,
`=` calculator, `b` spatial bookmarks, `set` settings. Prefixes under three
characters are reserved for core filters. Plugins register their own filters
against the same bar.
([QGIS GUI docs](https://docs.qgis.org/3.44/en/docs/user_manual/introduction/qgis_gui.html),
[QgsLocatorFilter](https://api.qgis.org/api/classQgsLocatorFilter.html))

**Blender.** `F3` opens a fuzzy operator search over every registered operator,
context-scoped to the editor under the cursor.

**Fantasy-map tools.** Azgaar's generator ships a plain name search over its
burg/state/culture lists; Wonderdraft and Inkarnate have none — they rely on
the label list panel.

**Proposal.** Build QGIS's locator, not a modal dialog — the spec's `⌘F` should
focus a **search field in the status bar**, which is where §10 already puts
"the two or three shortcuts that apply right now" and has the room.

| Prefix | Scope | Backed by |
|---|---|---|
| *(none)* | everything below, ranked | |
| `s` | settlements | `get_settlements()` — already filtered by name in `world_data_window.gd` |
| `p` | provinces | `get_provinces()` |
| `f` | factions | `get_factions()` — id + culture |
| `l` | labels | `label_list()` |
| `i` | placed icons | `icon_list()` |
| `r` | roads / sea routes | `get_roads()`/`get_sea_routes()` — they carry `name` |
| `y` | timeline years | `get_civ_timeline_years()` |
| `.` | menu commands | the seven `PopupMenu`s |

Selecting a result pans and zooms the camera to it (`ViewportHost` has real
zoom/pan) and pins it in the right dock via the existing
`on_settlement_selected`/`show_faction`/`show_route` calls. **Every source above
is already exposed**, so once the search *design* exists this becomes (A) —
which is exactly why it is worth designing now.

### 7.3 Data manager ▸ Sources — DM-06

**QGIS.** Two surfaces, deliberately separate. The **Data Source Manager**
(`Ctrl+L`) is a modal with a left rail of *source types* (Vector, Raster,
Delimited Text, GeoPackage, PostgreSQL, WMS/WMTS, XYZ Tiles…), each with its own
connection form. The **Browser** panel is the persistent tree of *saved
connections* and the filesystem, from which layers are dragged into the project.
([QGIS Data Source Manager](https://guides.lib.utexas.edu/mapping_and_file_conversion_for_tabular_geospatial_data/qgis-guide))

**ArcGIS Pro.** The **Catalog** pane plays the Browser role; connections are
first-class project items stored in the `.aprx`.

**Mapbox Studio.** "Sources" are tilesets you upload or link; the list shows id,
type, size and last modified, and a source can be *used by* N styles — the usage
back-reference is the panel's most useful column.

**Proposal.** §2.4's three Sources rows map cleanly onto this split, and the
mapping tells you what each one is:

- **External Sources** = QGIS's Data Source Manager rail — *"a thing you could
  connect to but have not"*. In Cartalith today the only real candidates are the
  Markdown vault (DM-14) and an asset-pack folder. Ship it as a **type list**
  with one row per supported kind, each disabled with its own reason, rather
  than an empty pane.
- **Connected Sources** = QGIS's Browser / ArcGIS's Catalog — *"what this project
  is currently attached to"*. Today that is: the loaded `.zip` project
  (`current_project_path`), the loaded asset pack (`has_asset_pack()`), and the
  four storage roots (`DccSettings.all_roots()`). **All three are real now**, so
  this pane can ship immediately with genuine content — one row per connection
  with path, kind, and a Reveal action reusing `show_project_on_disk()`.
- **Source Registry** = Mapbox's tileset list — *"the durable catalogue across
  projects, with usage"*. Fold it into Connected Sources rather than shipping a
  third empty pane; there is nothing yet that outlives a project except the
  storage roots and the recent-projects list, both of which already have homes.

**Recommendation to the owner: collapse three rows to two.** "Source Registry"
earns its own route only once something persists across projects.

### 7.4 Data manager ▸ Conversion — DM-07, DM-08, DM-09

**QGIS.** There is no "Conversion" menu. Format conversion is *export*
(`Export ▸ Save Features As…`, driven by GDAL/OGR's driver list) and
reprojection is either **on-the-fly** (every layer is transformed into the
project CRS automatically) or the explicit **Reproject Layer** algorithm in the
Processing Toolbox. Datum transformations are configured once, globally, under
Settings ▸ Options ▸ Transformations.
([QGIS: Working with Projections](https://docs.qgis.org/3.44/en/docs/user_manual/working_with_projections/working_with_projections.html),
[QGIS: Reprojecting and Transforming](https://docs.qgis.org/3.44/en/docs/training_manual/vector_analysis/reproject_transform.html))

**The finding this yields is a naming finding.** No GIS application of
consequence has a top-level "Conversion" route, because conversion is not a
destination — it is a *parameter of an export* (which format?) and a *property of
a project* (which CRS?). §2.4 promotes both to routes, which is why two of its
three rows are undefined even in the spec.

**Proposal.**

1. **Delete the Conversion group.** Move its content to where the comparables
   put it: *Format Conversion* becomes the **format dropdown inside each Export
   route** (§9's route pane already has one for tiles); *Coordinate Systems*
   becomes a **project property**, not a route — a field in `File ▸ New world`
   and a read-only line in the Export ▸ Maps route's PROJECTION column, which
   §9 already designs.
2. *Data Transformation* has no comparable at all as a menu item. QGIS's nearest
   equivalent is the Processing Toolbox, which is a completely different
   product surface (a searchable algorithm catalogue with parameter dialogs and
   a model builder). **Recommend dropping the row** rather than designing a
   Processing Toolbox for a procedural generator that has no user-supplied data
   to transform.
3. If the owner keeps CRS (DM-07 is their decision), the honest form is QGIS's:
   **one project CRS, set at creation, applied everywhere**, plus an
   `EPSG:3857`/`EPSG:4326` choice in the export route only. Do not build
   per-layer on-the-fly transformation — there is one flat km grid and nothing
   to transform *between*.

### 7.5 Data manager ▸ Validation — DM-10, DM-11

**QGIS** ships two distinct validators, and the distinction is the design:

- **Check Geometries** (Geometry Checker plugin) — *per-feature* validity:
  self-intersections, unclosed rings, wrong ring orientation. Results are a
  table, one error per row, with layer, id, error type, coordinates, a value,
  and a **resolution column**. Errors are selectable and **fixable in bulk**
  with a chosen resolution method.
- **Topology Checker** — *between-feature* rules you configure yourself (no
  gaps, no overlaps, no duplicates), run over the whole layer or the current
  extent. Results are a table; clicking a row **zooms the canvas to the
  offending feature**; errors are **not auto-fixed** — the table is a worklist.

([QGIS Geometry Checker](https://docs.qgis.org/testing/en/docs/user_manual/plugins/core_plugins/plugins_geometry_checker.html),
[QGIS Topology Checker](https://docs.qgis.org/3.44/en/docs/user_manual/plugins/core_plugins/plugins_topology_checker.html))

**Proposal.** Adopt the two-validator split verbatim, because Cartalith's own
invariants fall into exactly those two shapes, and the UI is one table either
way:

| Route | QGIS analogue | Rules Cartalith can actually check today |
|---|---|---|
| **Check Data** | Geometry Checker | A settlement on water without `snap_to_water`; a way whose waypoints leave the grid; a label with empty text; a placed icon addressing a slot the loaded pack does not fill; a timeline year whose snapshot references a `tid` no live settlement carries (CV-03's own gap, surfaced as a check); a province whose `capital_settlement_index` is out of range |
| **Repair / Normalize** | Topology Checker's fix guidance | Only where a reversal already exists: delete the orphaned label/icon, clamp the way, clear the dangling year. **Never** auto-repair generated data. |

Ship it as `world_data_window.gd`'s fourth tab, not a Data-manager route — it is
a *table over world data*, which is precisely what that window already is, and
it inherits the filter field for free. Then `Data ▸ Validation` becomes a
shortcut into that tab, exactly as §2.4 says the Data dropdown is *"a shortcut,
not a second implementation"*. **Adopt QGIS's "click a row to zoom to it"** —
`ViewportHost` has real camera control, and it is the single feature that makes
a validation table useful rather than a wall of text.

### 7.6 Colour management — PR-07

**Blender 4.x** is the reference implementation for a creative app. Its Color
Management panel exposes exactly four things: **Display Device** (sRGB,
Display P3, Rec.1886), **View Transform** (Standard / AgX / Filmic / Raw /
False Color), **Look**, and **Exposure/Gamma**. Blender 4.0 replaced Filmic with
**AgX** as the default view transform and moved to an OCIO v2 config referenced
to CIE XYZ. Notably, Blender still has **no preference for choosing your own OCIO
config** — you replace the file.
([Blender 4.0 Color Management release notes](https://developer.blender.org/docs/release_notes/4.0/color_management/),
[Blender Manual: Color Management](https://docs.blender.org/manual/en/4.0/render/color_management.html))

**The lesson for Cartalith.** §2.5's row offers `sRGB · Display P3 · linear` as
if they were one axis. They are not: sRGB and Display P3 are **display devices**;
linear is a **working space**. Shipping them as one dropdown would be a category
error that becomes very expensive to unpick later.

**Proposal.**

1. **Do not ship the row as specified.** Replace it with Blender's two-axis
   form, and ship only the half that is meaningful today:
   - **Working space** — the engine writes 8-bit sRGB directly
     (`render.rs`), and `TERRAIN_APPEARANCE_SCOPE.md`'s own `Ultra` tier doc
     refuses to claim the precision/HDR half is built. So: a **read-only
     readout** saying `sRGB, 8-bit` with the reason, not a dropdown.
   - **Display device** — genuinely actionable and cheap, because Godot owns
     it, not the engine: sRGB / Display P3, applied to the viewport. Ship
     this one live when the renderer stops being the constraint.
2. **When HDR/precision lands, add a View Transform, not a colour space.** The
   Cartalith equivalent of AgX is a tone map over the relief composite, and the
   correct place for it is `TerrainAppearance`, alongside the ramp (CA-02) —
   *not* in Preferences. Blender puts the view transform in Render Properties
   for the same reason: it is part of the look, not part of the application.
3. Keep the name **Colour management** — it is the universal term across
   Blender, Affinity, Resolve and Photoshop, and no comparable calls it
   anything else.

### 7.7 Tiled LOD, tile size, atlas cache — PR-10

**Gaea 2** splits *what you build* from *how it is written*: **Build Types**
(Normal / Split / Tiled), a **tile size** per tile (e.g. 1024 × 1024) and a
**blending percentage** between adjacent tiles, all inside the Build dialog; the
**Build Manager** is a separate persistent list of every node marked for export,
with saved, organised, reusable build definitions.
([Gaea: Build Types](https://docs.quadspinner.com/Guide/Build/Build-Types.html),
[Gaea: Tiled builds](https://docs.quadspinner.com/Guide/Build/Tiled.html),
[Gaea: Build Manager](https://docs.quadspinner.com/Guide/Build/Manager.html))

**World Machine** keeps a global **Resolution** slider whose maximum "depends
upon the devices present in the world", and puts tiled output in a separate
Tiled Build setup that writes a rectangular set of files for effectively
unlimited extent.
([World Machine 2 User Guide](https://www.world-machine.com/WM2%20User%20Guide.pdf))

**The pattern both share, and Cartalith does not.** *Interactive* LOD and
*export* tiling are two different subsystems with two different homes. Neither
app has an "atlas cache size cap" preference at all — the working resolution is
a document property, and the tiled build is an export operation.

**Proposal.**

1. **Split §2.5's four rows across two homes.**
   - **Interactive LOD** (auto-on-zoom, chunk debug overlay, tile borders) is a
     **view** concern → the Layers popover, next to the debug views it already
     lists. It is live today (S4) and needs only a toggle and a debug draw.
   - **Tile size and LOD levels** are an **export** concern → §9's Export ▸ Maps
     route pane, which already has a TILES column with exactly these fields.
     They do not belong in Preferences at all.
2. **The atlas cache is the only genuine Preferences row**, and it should be
   modelled on a browser's cache pane rather than on a terrain tool (none of
   which has one): a **size cap in GB**, a **live "currently N MB in M tiles"**
   readout, and **Clear**. Ship it only when tiles are actually written to disk;
   today they are synthesized on demand and dropped, which is why S4 now says so.
3. Keep `DccSettings`' existing `atlas_cache` storage root — it is already the
   right shape and already user-settable, and `app.gd`'s note about having
   nothing to invalidate becomes true-and-obsolete on the same day the cache
   ships.

### 7.8 Units — PR-15

**Blender** puts units in **Scene Properties ▸ Units**: a Unit System
(None/Metric/Imperial), a Unit Scale, and per-quantity overrides (Length, Mass,
Rotation, Temperature). It is a **scene** property, not a preference — the file
carries it.

**QGIS** has both, and the split is instructive: measurement units for the
*measure tool* are an **application** Option; the *project's* display units for
coordinates and areas are a **Project Property**.
([QGIS Configuration: Options vs Project Properties](https://docs.qgis.org/3.44/en/docs/user_manual/introduction/qgis_configuration.html))

**The collision `DCC_CONTROL_INDEX.md` §3(j) flags is real and has a standard
answer.** Units appear twice in `DCC_SHELL_SPEC.md` — Preferences ▸ Application
and §5.1 stage 02 — because they genuinely are two things in every comparable:
a *display preference* (how I want to read numbers) and a *project property*
(what this world is measured in). Cartalith's engine is km-internal and always
will be, so there is no project property here.

**Proposal.**

1. **One control, in Preferences ▸ Application ▸ Units**, `km · mi`. Delete the
   §5.1 stage-02 occurrence — nothing about extent or scale changes with it.
2. **It is a display conversion only, applied at the leaf.** The places that
   would need it, all of which already format km by hand: `right_dock.gd`'s
   `_route_length_text`/`_build_measure`/`_build_region`,
   `journey_planner_view.gd`'s totals and matrix, `viewport_host.gd`'s scale
   bar, `new_world_dialog.gd`'s derived readout. Add one
   `DccTheme.km(value) -> String` helper reading `DccSettings`, and route all of
   them through it. **This is (A) work once the decision is made** — the whole
   cost is the decision.
3. **Follow Blender in naming the quantity, not the unit**: the row reads
   `Length: Kilometres / Miles`, so adding area or temperature later does not
   need the row renamed.

### 7.9 Keyboard shortcuts editor — PR-16, HE-02

**Blender** is the most complete implementation and the closest match to
Cartalith's problem, because it has the same difficulty: the same chord means
different things in different editors and modes. *Preferences ▸ Keymap* is a
**searchable tree** — keymap ▸ editor context ▸ operator ▸ the individual
binding, where expanding a binding exposes the full chord, its modifiers, the
mouse button, and the operator's own properties. Searching a chord shows every
context it is used in. Crucially, the everyday path is **not** the editor:
right-clicking any menu item or button offers **Assign Shortcut / Change
Shortcut** in place.
([Blender Keymap release notes](https://developer.blender.org/docs/release_notes/4.0/keymap/),
[Artisticrender: find, manage, change and reset shortcuts](https://artisticrender.com/blender-shortcut-keys-how-to-find-manage-change-and-reset-them/),
[brandon3d: custom shortcuts](https://brandon3d.com/how-to-create-custom-keyboard-shortcuts-in-blender-3d/))

**Photoshop** and **DaVinci Resolve** both ship a modal keyboard editor with a
**visual keyboard** and a **searchable command list**, plus **named preset sets**
that can be exported (Resolve ships "DaVinci Resolve", "Premiere Pro",
"Final Cut Pro 7" and "Avid Media Composer" sets out of the box) — the killer
feature for users migrating from another tool.

**Proposal.**

1. **Ship the two-surface model, in this order.** First the *in-place* path,
   because it is cheap and covers most real use: right-click any menu item or
   tool button → **Assign shortcut**, capture the next chord, write it to
   `DccSettings`. Second the table, when the shortcut count justifies it.
2. **The table's tree matches the shell's own structure**, which is what makes
   it per-context without inventing a context model: **Program menus** (the
   seven) → **Global tools** (§4.5.1's four) → **per-domain tools** (WORLD/CIVIL/
   INFRA/CARTO) → **Windows** (Asset library, Data manager, World data,
   Performance, Journey). That is exactly Blender's keymap-context tree, using
   containers the shell already has.
3. **Adopt Blender's conflict handling, not Photoshop's.** Do not block a
   duplicate chord — show every other binding that already uses it, scoped by
   context, and let the user decide. Cartalith has real per-context reuse
   (`Esc` commits Way/Route but disarms everything else) and a global uniqueness
   rule would be wrong.
4. **Adopt Resolve's preset sets** for one specific reason: `Cartalith Gen1
   v2.10`'s own key assignments are a real, documented set that existing users
   have in their fingers. Ship "Cartalith Gen1" alongside "Cartalith DCC".
5. **Merge PR-16 and HE-02.** `Help ▸ Keyboard shortcuts` should open a
   read-only reference sheet; `Preferences ▸ Keyboard shortcuts…` opens the
   editor. Every comparable does exactly this split; two editors is a bug.
6. Godot's `InputMap` plus `ConfigFile` covers all of it with no engine work,
   so this is **(A) the moment the design lands**.

### 7.10 Save layout as… — WI-01

**Blender** calls them **Workspaces**: named tabs across the top, each a
complete screen layout of areas and editors, geared to a task (Layout,
Modeling, Sculpting, Shading, Animation). They are **saved in the .blend file**,
and the default set comes from the startup file. New workspaces are added from a
template list or duplicated from the current one.
([Blender Manual: Workspaces](https://docs.blender.org/manual/en/latest/interface/window_system/workspaces.html))

**DaVinci Resolve** calls them **Layout Presets**: `Workspace ▸ Layout Presets ▸
Save layout as a preset`, name it, then `Load Preset`. Plus a persistent
`Reset UI Layout`.
([Resolve: Saving Custom Screen Layouts](https://www.steakunderwater.com/VFXPedia/__man/Resolve18-6/DaVinciResolve18_Manual_files/part83.htm),
[teckers: How to save Resolve layouts](https://teckers.io/how-to-save-davinci-resolve-layouts/))

**Photoshop** calls them **Workspaces** too (`Window ▸ Workspace ▸ New
Workspace…`), and stores panel positions *plus* keyboard shortcuts and menus in
the same preset — the one difference worth noting, because it bundles §7.9's
output into the same object.

**The naming finding.** Three of four comparables say **Workspace**; only
Resolve says *layout preset*. **But Cartalith already uses "workspace" for
something else** — the five domain-rail workspaces (WORLD/CIVIL/…). That
collision is the reason to keep the spec's *Layout* wording. Recommend
`Window ▸ Layout ▸ Save layout… / Load layout ▸ / Reset layout`, and **never**
introduce "workspace" as a second meaning.

**Proposal.**

1. **What a layout is**, concretely, given `DccShell`'s real state: the five
   `_region_nodes` visibilities, `_left_collapsed`/`_right_collapsed`,
   `_left_width`/`_right_width`, the active domain, and each domain's open L2
   category. All of it already lives in GDScript.
2. Store it in `DccSettings` under a `layouts` section, one key per named
   layout, exactly as `storage_roots`/`recent` already work. `Reset layout`
   already exists and works (`toggle_region(ID_WIN_RESET)`).
3. **Follow Blender in shipping defaults, not an empty list**: pre-seed
   `Generate`, `Sculpt`, `Cartography` and `Journey` layouts matching how those
   four tasks actually want the docks. A layout feature with nothing in it is
   the reason most users never find one.
4. This is **(A) once the definition above is fixed** — pure GDScript over
   existing state.

### 7.11 Documentation and Report an issue — HE-01, HE-03

There is no design and little to research: every comparable opens a URL.
Blender's Help menu opens the manual, the Python API, and *Report a Bug*
pre-filled with the system information. That last detail is the only one worth
copying.

**Proposal.** `Help ▸ Documentation` → `OS.shell_open` to the repository's docs.
`Help ▸ Report an issue` → `OS.shell_open` to a GitHub issue URL with a body
pre-filled from data the shell already has (`Engine.get_version_info()`,
`OS.get_name()`, `bridge.quality_tier()`, `bridge.gpu_stages_used()`,
`OS.get_static_memory_usage()` — every one of which `performance_window.gd`
already reads). **The destination URL is the only owner decision**; both items
are (A) once it exists.

### 7.12 Calculation trace window — JP-05

No comparable in the map/DCC space; the closest are **spreadsheet formula
auditing** (Excel's Evaluate Formula steps through a calculation one
substitution at a time) and **shader/node-graph inspectors** (Blender's node
editor showing intermediate outputs).

**The real constraint is already recorded and is a good one.** `jp_land_calc_dict`'s
own doc deliberately does not carry the reference's `formula` trace string
across the boundary, on the ground that it is presentation, not engine.
Re-deriving prose in Rust would repeat that mistake.

**Proposal.** Build the trace **in GDScript, from the dict values that already
cross**, and drop the `⧉` window in favour of an expandable group in the results
panel — a window for one journey stage is more chrome than the content earns,
and §8 lists it as the last of seven collapsible groups anyway. One row per
multiplicative term, in the engine's own application order, showing name,
factor and running value:

```
base            Walking, Steady          4.0 km/h
× terrain       hills                    ×0.82   3.28
× biome         temperate_forest         ×0.90   2.95
× weather       rain                     ×0.85   2.51
× load          78 % of capacity         ×0.94   2.36
× hours         9.0 h/day                        21.2 km/day
```

Every factor above is already in `results[i]`'s `eff` dict or derivable from the
stage. **Recommend the owner amend `JOURNEY_PLANNER_SPEC.md` §8's `⧉`** to an
inline group; it is the one place in that spec where the shell's own
"one window per subject" rule is over-applied.

### 7.13 Sculpt: brush shape, stroke & grid, actions — WW-03, WW-04, WW-05

These are the register's only gaps the design *itself* labels new work
(`DCC_SHELL_SPEC.md` correction #3), so the research is about scoping them
honestly rather than filling a hole.

**Blender sculpt brushes.** The falloff is a **curve widget** mapped from brush
centre (left) to border (right), with named presets (Smooth, Sphere, Root,
Sharp, Linear, Constant) plus a custom curve. Separately, brush **tip shape**
comes from a texture, and **Stroke** carries spacing, jitter and stroke method
(Space, Drag Dot, Anchored, Line…). Blender 5.x converted brushes whose custom
curve approximated smoothstep to a built-in "Smooth" preset — i.e. it moved
*toward* named presets, away from hand-drawn curves.
([Blender Manual: Falloff](https://docs.blender.org/manual/en/latest/sculpt_paint/brush/falloff.html),
[Blender Manual: Stroke & Curve](https://docs.blender.org/manual/en/2.79/sculpt_paint/stroke_curve.html),
[Blender 5.3 sculpt release notes](https://developer.blender.org/docs/release_notes/5.3/sculpt/))

**Krita** draws the distinction Cartalith needs: a **brush tip** is "only a stamp
of sorts"; a **brush preset** is a tip plus every other setting. Tips can be
predefined shapes, imported images, or heightmap-derived. Spacing is a separate,
first-class setting with a visible effect.
([Krita: Brush Tips](https://docs.krita.org/en/reference_manual/brushes/brush_settings/brush_tips.html),
[Krita: Loading and Saving Brushes](https://docs.krita.org/en/user_manual/loading_saving_brushes.html))

**Proposal — and a recommendation to cut two of the three.**

- **WW-03 Brush shape: build a reduced version.** Cartalith's engine has one
  coverage shape (distance-to-polyline / radial) modulated by `edge_noise`
  domain warp, and one falloff (`smoothstep(0,1,(R−dist)/feather)`,
  `feather = R·(1−hardness)`). The cheap, high-value half is **Blender's falloff
  curve preset list** — Smooth (what exists), Linear, Sharp, Constant — because
  each is a one-line change to that single formula and each visibly changes a
  ridge's profile. Ship those four, plus the **live falloff preview** §5.2 asks
  for, which is drawable from the real formula today. **Do not ship** the
  eight-shape gallery or Import brush: those need a stamp-mask mechanism the
  engine does not have, and Krita's own docs are clear that a tip is a
  fundamentally different object from a falloff.
- **WW-04 Stroke & grid: recommend deleting the block.** Its eight controls
  edit "the selected stamp's control points". `SculptStamp` stores a point list
  and frozen parameters; there is no control grid, no handles, and — critically
  — no comparable does this either. Blender and Krita both re-stroke rather
  than edit a stroke's control points; only vector tools (Illustrator, Inkscape)
  offer point editing, and a sculpt stamp is not a vector path. The honest
  replacement is what the shell already ships: delete the stamp and draw again,
  with Undo covering the mistake.
- **WW-05 Actions (Flip X/Y, Rot L/R, Flatten): build Flip and Rotate, drop
  Flatten.** Flip and rotate over a stamp's own point list are pure geometry on
  data the engine already stores — a real, small, well-defined addition, and
  the one part of §5.2's new work that has an obvious meaning. "Flatten
  selection" needs a selection model that does not exist (ED-03) and should
  wait for it.

### 7.14 Label font role — CA-07

**Wonderdraft** is the closest comparable and its documented failure is
instructive: labels support custom fonts and curved text along coastlines and
rivers, but suffer **"zoom vertigo"** — a size that reads correctly at one zoom
is unreadable at another, and *the same numeric size means different things on a
2048² map and an 8192² map*. **Inkarnate** avoids it by making label editing
one-click and per-layer rather than by solving the scaling.
([Loreteller: Wonderdraft Labels — Fonts, Sizing and the Zoom Trap](https://loreteller.com/learn/wonderdraft-labels-guide/),
[Loreteller: Inkarnate vs Wonderdraft](https://loreteller.com/learn/inkarnate-vs-wonderdraft/))

**Cartalith has already avoided half of it** — `MapLabel` carries a `size_mode`
of `fixed` / `zoom`, exposed in the dock, which is exactly the control
Wonderdraft lacks. What it has instead is a **CSS font string** the renderer
cannot honour.

**Proposal.**

1. **Read §4.5.5's "font role" literally** — it says *role*, not *font*. Replace
   the free-text CSS field with a **four-role dropdown**: `Region · Settlement ·
   Water · Annotation`. Each role maps to a bundled face, a weight, a tracking
   value and a default size, defined once in `DccTheme`. This is how every
   cartographic style system works (Mapbox styles, ArcGIS label classes), and it
   is the only version that survives export, where a CSS string is meaningless.
2. **Keep the raw string as an advanced override**, disabled with its reason, so
   loaded reference data round-trips rather than being silently dropped.
3. **Adopt Wonderdraft's lesson explicitly in the tooltip**: state that
   `size_mode: zoom` is the safe default and why. It is a real advantage over
   the best-known tool in the category and currently goes unremarked.

### 7.15 Style presets — CA-08

**Every comparable ships named looks and a "modified" indicator.** Mapbox
Studio's style gallery, ArcGIS Pro's basemap gallery, Affinity's adjustment
presets, Blender's material previews. The universal pattern is: named presets in
a gallery with a thumbnail, an accent outline on the active one, a **`custom —
edited`** state the moment any field diverges, and **Reset** / **Save as**.
`DCC_SHELL_SPEC.md` §4 already describes exactly this, which is why only the
*content* is (C).

**Proposal.**

1. **Ship the mechanism against what exists, not against the four names.**
   `TerrainAppearance::default()` (the atlas look) and `js_reference()`
   (bit-identical JS output) are two real, tested appearances. Ship the preset
   chip row with those two — named **Atlas** and **Reference** — plus
   `custom — edited since preset`, Reset, and Save preset. That is the whole §4
   row, honest, today, once CA-01's `set_appearance()` wrapper exists.
2. **Parchment / Physical / Ink are new looks**, not presets over existing
   fields, and `design/cartalith-menu-structure.md` §5b names sixteen more
   (Natural Terrain, Vibrant, Muted, Geological, Antique Atlas…). Recommend the
   owner pick the shipping set from that longer list rather than from §4's four,
   since §5b is the owner's own document and is far more specific.
3. **Save preset needs `TerrainAppearance: Serialize`**, which it does not
   derive. That is the one Rust line the whole feature depends on.

### 7.16 Layer list search; Blocks / Verticality — CA-09

The search field needs no research — §7.2's locator, scoped to the layer list.

**Blocks / Verticality is genuinely undefined**, and `DCC_CONTROL_INDEX.md`
already marks it uncertain. No comparable has footer tabs by those names. The
two plausible readings, from the vocabulary of the field:

- **Blocks** = a *block diagram* / 2.5D extruded view — a standard cartographic
  presentation of terrain, and the natural companion to "Verticality".
- **Verticality** = vertical exaggeration, i.e. `TerrainAppearance::exag`, which
  is real.

**Recommendation: ask the owner rather than design it.** This is the only (C) in
the register where a wrong guess would produce a whole pane of wrong controls,
and the answer costs one sentence. If "Verticality" is indeed `exag`, it is not
a footer tab at all — it is one slider in the LIGHT/relief group, and CA-01's
wrapper already covers it.

### 7.17 Rail expansion — SH-01

**Blender's** collapsed/expanded sidebar and **Photoshop's** icon/expanded panel
docks both expand to reveal *labels for the same items*, never new items.
**VS Code's** activity bar expands to a per-view sidebar with **different**
content per activity.

`DCC_SHELL_SPEC.md` §3 says the expanded rail shows *"the domain's sub-nodes as a
200 px list"*, which is the VS Code model — and, as `DCC_CONTROL_INDEX.md`
records, **the spec never enumerates the sub-nodes**, so there is nothing to
build.

**Proposal.** Take the Blender/Photoshop reading instead, which is buildable
today and loses nothing: expanding the rail shows each domain's **full label plus
its subtitle** — both already in `DccShell.DOMAINS` and already used as the
button tooltip — at 200 px. The phone drawer (`_build_phone_drawer`) already
renders exactly this, as `_phone_list_row(label, subtitle)`. **Reusing it makes
this (A).** If the owner wants VS Code's sub-node list instead, the sub-nodes are
the L2 categories each workspace already registers in `Workspace.categories`,
and that is the enumeration the spec is missing — worth confirming before
building either.

---

## 8 · Menu naming audit

The owner asked specifically about menu naming. This section audits Cartalith's
vocabulary against (a) `DCC_SHELL_SPEC.md`'s own prescribed names and (b) the
conventions of the comparable applications above. **It recommends; it does not
rename.** `DCC_SHELL_SPEC.md` is owner-supplied and renaming is an owner
decision.

### 8.1 The shipped vocabulary matches the spec exactly

Verified control by control against §2: the seven program menus are
**File · Edit · Assets · Data · Preferences · Window · Help**, in that order,
and every submenu label matches the spec's own table with three deliberate,
documented divergences and one omission:

| Divergence | Where | Status |
|---|---|---|
| `Storage locations` merged §2.1's two items (`Storage locations` + `Change locations…`) into one | `menus.gd:123-128` | Owner feedback, 2026-08-19 — recorded in-file |
| `Import ▸ …` / `Export ▸ …` etc. carry their sub-items inline in the label (`Import ▸ Maps · Heightmaps · GIS · World data`) rather than as real submenus | `menus.gd:273-277` | The Data dropdown is a shortcut into one window, per §2.4; inline labels avoid a submenu that duplicates the window's own rail |
| `⧉ Sprite sheet slicer (▦)` keeps both markers | `menus.gd:199` | Fixed in `595582d` to use the spec's own `⧉` window marker |
| **`⧉ Travel library… ⇧L` missing** | — | Omission O1 |

**So there is no naming drift to report against the spec.** The naming questions
worth raising are all questions about the spec itself.

### 8.2 The seven program menus, against convention

| Menu | Convention | Assessment |
|---|---|---|
| **File** | Universal. Blender, Photoshop, QGIS, ArcGIS Pro, Krita, Resolve all have it. | **Keep.** No note. |
| **Edit** | Universal. | **Keep.** But see 8.4 — it currently contains ten disabled items and nothing else, which is a *content* problem, not a naming one. |
| **Assets** | Uncommon as a top-level menu. Unreal and Unity put assets in a *browser panel*, not a menu; Blender has no Assets menu (asset browser is an editor); Affinity has an Assets *panel*. Photoshop's nearest is Libraries. | **Flag.** "Assets" as a menu is idiosyncratic, but defensible here: Cartalith's assets are a *pack you load and apply*, not a project tree you browse, which is genuinely closer to Photoshop's Libraries than to Unreal's Content Browser. **Recommend keeping**, and noting that its most-used item (`Import asset pack .zip…`) is the only one currently live. |
| **Data** | **The most idiosyncratic name in the set.** No comparable has a "Data" menu. QGIS has *Layer* and *Project*; ArcGIS Pro has *Insert* / *Analysis* / *View*; Mapbox Studio has no menu bar. GIS applications call this surface **Data Source Manager** (QGIS), **Catalog** (ArcGIS), or split it across Import/Export. | **Flag — and it is doing too much.** See 8.3. |
| **Preferences** | Split convention. **macOS-lineage apps** say *Preferences* (Photoshop, Affinity, Krita on macOS); **Windows/Linux-lineage** say *Options* (QGIS, LibreOffice) or *Settings* (VS Code, Blender's is literally "Preferences" but reached from Edit). **Nobody puts it at the top level of the menu bar** — it is universally inside Edit (Windows/Linux) or the app menu (macOS). | **Flag — placement, not name.** See 8.5. |
| **Window** | Universal in creative apps (Photoshop, Affinity, Illustrator, Resolve's is *Workspace*). Blender and QGIS use *View*. | **Keep.** Correct for this product's lineage. |
| **Help** | Universal. | **Keep.** |

### 8.3 "Data" is overloaded — the register's strongest naming finding

`Data` currently carries **seven items across three unrelated concerns**:

| Item | Actually is |
|---|---|
| World data tables… | a **read-only browser** over generated world state |
| Journey planner… ⇧J | a **tool**, which arms the INFRA JOURNEY tool and takes over the viewport |
| *(missing)* ⧉ Travel library… ⇧L | a **reference-data editor** |
| Import ▸ | file I/O |
| Export ▸ | file I/O |
| Sources ▸ | connections |
| Conversion ▸ | (undefined — §7.4 recommends deleting) |
| Validation ▸ | a **check over world state** |

Three of those are not data management at all. **Journey planner is a tool** —
`DCC_SHELL_SPEC.md` §4.5.4's own addition says so explicitly ("arms the JOURNEY
tool in INFRA… and takes over the viewport in place, the same way any other tool
does"), and it is the only menu item in the shell that arms a tool. That is a
real inconsistency: every other tool is armed from a dock's TOOLS block.

**Recommendations (owner decision, all of them):**

1. **Rename `Data` → `Project data`, or split it.** The comparable convention
   for "everything in and out of the project" is QGIS's *Data Source Manager*
   under a **Layer**/**Project** menu. If the name stays, at least drop
   Conversion (§7.4) so the menu reads as *in · out · sources · checks*.
2. **Move `Journey planner… ⇧J` out of Data.** It belongs where every other
   tool lives — INFRA's TOOLS block, where it already has a second entry point
   (`infrastructure_workspace._build_logistics`). Keeping the `⇧J` accelerator
   costs nothing. This also removes the only item in the menu bar that changes
   the viewport, which `UI_SHELL_DESIGN.md`'s own rule — *"the top bar is about
   the program, the map is about the world"* — arguably already forbids.
3. **`World data tables…` and `Validation ▸` are the same window** (§7.5
   proposes Validation as its fourth tab). Naming them as one thing —
   `World data…` with tabs Settlements · Provinces · Economy · Checks — removes
   a route and matches how QGIS pairs the attribute table with the geometry
   checker's result table.

### 8.4 Edit is a menu of ten disabled items

Not a naming problem, but the naming audit surfaced it: **`Edit` contains
nothing that works.** Every comparable's Edit menu is among its most-used. A
menu that is 100 % disabled trains users to stop opening it, and then the items
that eventually land there are never found.

**Recommendation:** when global undo (ED-01) is still far off, consider moving
`Find on map…` (§7.2, which becomes (A) as soon as it is designed) into Edit
early, so the menu has one live item. Alternatively, follow Blender and put
**Preferences** in Edit (see 8.5) — which would give it a live item today.

### 8.5 Preferences vs Settings vs Project settings

**The comparables split cleanly, and the split is about scope, not about the
word.** QGIS is the clearest: **Settings ▸ Options** are *application-wide,
saved to the user profile, applied to every new project*; **Project ▸
Properties** are *project-specific* — the example the docs give is that a white
background and WGS84 suit one project while a yellow background and UTM suit
another.
([QGIS Configuration](https://docs.qgis.org/3.44/en/docs/user_manual/introduction/qgis_configuration.html))

**Cartalith's `Preferences` currently mixes both scopes**, and one row of it is
already known to collide (PR-15, Units, which §5.1 also claims):

| Genuinely application-scope | Arguably project-scope |
|---|---|
| GPU acceleration, Devices, Multi-GPU, CPU threads, VRAM | Render quality (per world? per machine? `get_recommended_quality_tier()` says per machine) |
| Theme, Units, Keyboard shortcuts | Storage locations (per install, but the exports root is used per project) |
| Working set, Clear caches | Tiled LOD / tile size (§7.7 argues these are export parameters, i.e. project-scope) |
| 3D viewport / lighting rig **defaults** (the word "defaults" is doing scope work) | Colour management (§7.6 splits it: display = app, working space = document) |

**Recommendations:**

1. **Keep the word `Preferences`.** It is correct for this product's lineage
   (Photoshop/Affinity/Krita), and `design/cartalith-menu-structure.md` — the
   owner's own earlier document — already used *Project settings…* for the other
   scope, so the two words are available and distinct.
2. **Do not add a second top-level menu.** When project-scope settings become
   real (they largely are not yet), put them behind `File ▸ Project settings…`,
   which is exactly where `design/cartalith-menu-structure.md` §1 put it and
   where QGIS/ArcGIS both put their project properties.
3. **Consider moving `Preferences` inside `Edit`** — every Windows/Linux
   comparable does (Blender: *Edit ▸ Preferences*; QGIS: *Settings ▸ Options*;
   VS Code: *File ▸ Preferences*). Cartalith ships on Windows and Android, not
   macOS, so the top-level placement follows a macOS convention the product does
   not target. This would also solve 8.4.

### 8.6 The five domain-rail names

**WORLD · CIVIL · INFRA · CARTO · RENDER.** Two are abbreviations of the full
labels the tooltips and drawer already carry (Civilization, Infrastructure,
Cartography). The rail is vertical text at 9–11 px, so abbreviation is a
legitimate typographic constraint, and `DccShell.DOMAINS` already keeps the full
label and a subtitle for every entry.

| Name | Assessment |
|---|---|
| **WORLD** | Fine. |
| **CIVIL** | **Flag — genuinely ambiguous.** "Civil" reads as *civil engineering* in a product that also has an INFRA domain about roads and bridges. The full label is Civilization. **Recommend `CIV`** — shorter, unambiguous, and the engine's own crate is `cartalith-civ`, so it matches the codebase's vocabulary. |
| **INFRA** | Fine; standard abbreviation. |
| **CARTO** | Fine; standard in the field (CARTO is also a company name, but in context this is unambiguous). |
| **RENDER** | Fine, but see below. |

**One structural note.** `RENDER` and `CARTO` both concern how the map looks, and
`DCC_SHELL_SPEC.md` §3 draws the line as *"terrain appearance to CARTO"* while
§3's own table gives RENDER *"Terrain appearance groups"* — the two rows
contradict each other, and RENDER is currently an empty domain
(`render_workspace.gd` is 15 lines). `design/cartalith-menu-structure.md` §5b
resolves it the other way: terrain appearance is a **Map (cartography)**
sub-system. **Recommend the owner confirm which**, because RN-01 (the largest
cheap wrapper in the register) needs to know which dock it builds into.

### 8.7 Against the owner's own earlier structure

`design/cartalith-menu-structure.md` (2026-08-17, superseded) proposed a
different seven: **Project · World · Generate · Simulate · Map · Assets · View**,
with a six-item **mode bar** (WORLD · EDIT · ANALYSIS · SIMULATION ·
CARTOGRAPHIC · DEBUG) and a four-group left navigator.

`DCC_SHELL_SPEC.md` replaced it with **File · Edit · Assets · Data · Preferences
· Window · Help** plus a five-domain rail, and its own header explains why:
*"World generation, simulation, rendering and map styling are workspaces reached
through the domain rail (§3), never menu items."* That is a coherent and
defensible principle — menus hold *program* operations, the rail holds
*subjects* — and it is the same principle the earlier document's own
implementation note 2 stated (*"Menus hold operations and parameters; the left
navigator holds subjects"*).

**The two documents agree on the principle and disagree on where the line falls.**
Three things the earlier structure named have no home in the current one, and
all three are (C) or absent above:

| Earlier structure | Where it went |
|---|---|
| `Project ▸ Session ▸ Project settings…` | Nowhere — see 8.5 |
| `View ▸ Analysis field` (14 fields) | The Layers popover's 18 debug views — **arrived, better than specified** |
| `Map ▸ Terrain appearance` (§5b, ~60 controls) | Split ambiguously between CARTO and RENDER — see 8.6 |

**Recommendation:** treat `design/cartalith-menu-structure.md` §5b as the
authoritative content list for terrain appearance when RN-01/CA-01 get built. It
is far more specific than `DCC_SHELL_SPEC.md` §7, it is owner-supplied, and it
is the only document that enumerates the preset names §4 asks for.

---

## 9 · (D) entries: owner decisions, not gaps

Listed together so nobody proposes a design for them. **No design is proposed
for any row here.**

| # | Decision | Documented at |
|---|---|---|
| WW-11 | Per-stage `Run stage n` / `Run n → 10` / stale dots / stage counter — the capability exists in neither this engine nor the reference app; verified by Playwright against the real reference | `DCC_SHELL_SPEC.md` header correction #2; `world_workspace.gd:129-145`; `app.gd:298-306` |
| CV-09 | The timeline bar's six continuous simulation-layer toggles + Warfare | `DCC_CONTROL_INDEX.md` §10 and summary §5 item 5; `TIMELINE_SCOPE.md` §4, §6; `VISION.md` |
| CV-04 | The **metropolis** settlement tier (`_civSelectMetropolises`) | `TIMELINE_SCOPE.md` §6 — "a pre-existing, separately-scoped gap" |
| CV-08 | **`_civApplyRecovery`** (v0.82 static recovery phase) | `TIMELINE_SCOPE.md` §3 point 5, §6 — scoping belongs to `PHASE2_SCOPE.md` |
| PR-02 | Multi-GPU: build device selection / dispatch modes / VRAM budgeting at all? | `DCC_CONTROL_INDEX.md` summary §5 item 2 |
| DM-07 | Coordinate systems / EPSG as a first-class route | `DCC_CONTROL_INDEX.md` summary §5 item 8 |
| DM-14 | Markdown vault: two-way sync, `obsidian://` links in tiles, note links in GeoJSON — all V1 **non-goals** in the vault doc's own §33 | `DCC_CONTROL_INDEX.md` summary §5 items 3-4; `MARKDOWN_VAULT_INTEGRATION.md` §1, §33 |
| AS-16 | Asset family taxonomy: 24 (spec) vs 11 (mockup) vs 8 (engine, frozen) | `DCC_CONTROL_INDEX.md` summary §5 item 9; `ASSET_LIBRARY_SCOPE.md` §1 |
| AS-14, AS-15 | Per-slot "active variant" and per-slot Anchor — both contradict load-bearing engine semantics | `DCC_CONTROL_INDEX.md` §3(f) |
| IN-04, IN-05 | Way routing-mode dropdown; the spec's road/track/trail/bridge vocabulary | `infra_tools_bridge.rs`'s own module doc; `infrastructure_workspace.gd:42-49` |
| RD-04, CV-06, WW-10 | Government/agriculture (UI-only categoricals in the reference); settlement pick radius (computed internally); stages 09/10 having no dials (not parameterised) | `PHASE2_SCOPE.md` m18; `civ_tools_bridge.rs`; `world_workspace.gd:96-102` |
| RD-12, CA-10, SH-02, SH-04, SH-08 | Merged Brush/Stamp context; one Visualization picker not two; decorative phone gesture handles; placeholder battery glyphs; accelerators on permanently-disabled items | each recorded in the file that made the call |

---

## 10 · The actionable (A) list, in priority order

Every row here has a design **and** everything the engine needs. No Rust. Ordered
by value delivered per unit of work.

| Rank | # | What | Why first | Design |
|---:|---|---|---|---|
| 1 | **RD-03** — **done 2026-08-19** | Wire Settlement ▸ **Economy / Politics / Logistics** to their real destinations (`world_data_window` Economy tab · `show_faction()` · `open_journey_planner()`) | Three disabled buttons in the shell's most-used inspector, all three destinations already built. Highest visibility, lowest cost in the register. | §6, §4.5.4 |
| 2 | **RD-06 + RD-08** — **done 2026-08-19** | Faction ▸ Territory reads `civ_faction_territory_stats()`; Roster reads `get_factions()` | The dock is the last place still saying these queries don't exist (S1 just corrected the words; this corrects the behaviour). | §6, §4.5.3 |
| 3 | **JP-13** | Journey Planner's **timeline band** — one band per day, coloured travel / water / weather hold / rest-layover | `timeline_bar` is currently drawn **visible and empty** while JOURNEY is armed — the one place in the shell showing an empty region with no explanation. All the data is in `plan`. | `JOURNEY_PLANNER_SPEC.md` §2 |
| 4 | **JP-14** | Blocked-stage **inline resolutions** (turn off closures · re-route land-only · depart earlier) | A blocked journey currently ends in a dead end. All three are `_plan_values` edits plus `_compute()`. | `JOURNEY_PLANNER_SPEC.md` §9 |
| 5 | **RD-11** — **done 2026-08-19** | Right dock's collapsed **primary readout** | §6's own last line; `set_dock_readout()` exists and is wired for the left dock only. One call. | §6 |
| 6 | **PR-13 + PR-14** | **Light theme** + follow-system | `DccTheme.LIGHT` is fully defined and §11 gives the complete light token column; only the build-once stylebox pass blocks it. The single largest *visible* change available with no engine work. | §2.5, §11 |
| 7 | **WI-02 + WI-03 + WI-04** | Window menu: workspace list, open-windows list, **dock width dragging** | Three omissions against §1/§2.6; all three read state that already exists. | §1, §2.6 |
| 8 | **CA-05** | Icon **on-canvas resize handle** | `icon_resize`/`icon_hit_test` are exposed; the drag math already exists on the Label tool and can be copied. Handle geometry derives from `icon_get()`. | §4.5.5 |
| 9 | **JP-12 + JP-15** | Supply-reach **per-leg bar with resupply ticks**; party-form fields showing `auto · <resolved>` | `resupply_reach` and each result's `eff` dict already carry every value. | `JOURNEY_PLANNER_SPEC.md` §5, §8 |
| 10 | **SH-05** | Layers popover **hotkey badges 1–8** | The popover already enumerates every view; badges plus `InputMap` entries. | §10 |
| 11 | **SH-06** | Viewport `→ 1 582 m` draft-stamp elevation under the cursor | `sample_cell` + the draft preview both exist. | §10 |
| 12 | **SH-01** | Rail expansion showing label + subtitle at 200 px | Reuses `_phone_list_row()` verbatim; see §7.17 for why this reading beats the spec's unenumerated one. | §3 |

Four more become (A) **the moment their design lands** and are the best return on
a design decision rather than a build: **ED-05 Find on map** (§7.2 — every source
already exposed), **PR-15 Units** (§7.8 — one helper, the cost is the decision),
**PR-16 Keyboard shortcuts** (§7.9 — `InputMap` + `ConfigFile`), and **WI-01
Save layout** (§7.10 — all state is already in `DccShell`).

---

## 11 · Out of scope

- **This register proposes no implementation order and writes no GUI code.** The
  only edits made were the five factual corrections in §4.
- **`DCC_SHELL_SPEC.md`, `DCC_CONTROL_INDEX.md` and `design/` are untouched.**
  They are owner-supplied ground truth; this document cites them.
- **`DCC_SHELL_SPEC.md`'s six header corrections are respected, not re-litigated.**
  Every one of them (the sculpt commit prose, per-stage run, Brush shape/Stroke &
  grid/Actions, the sculpt global defaults, §12's text-symbol premise, the path
  note) appears here as a (C) or (D) with the correction cited, never as
  something to "fix".
- **No Rust was changed and none is proposed line-by-line.** (B) rows name the
  missing capability; they do not design it.
- **One real defect found and deliberately not fixed here**, because fixing it is
  a design change rather than a factual correction: **`timeline_bar` is visible
  and empty in CIVIL and INFRA.** `app.gd:271` shows it for both domains;
  `dcc_shell.gd:628-641` builds an empty `timeline_row`; and
  `TIMELINE_SCOPE.md` §4 explains why milestone 6 deliberately built its own
  panel instead. The result is a 70 px empty strip with no disclosure — the one
  place the shell shows a region with nothing in it and says nothing about why.
  Two honest fixes exist (hide it until something fills it, or put a
  one-line disclosure in it, per the `_todo()` convention), and JP-13 fills it
  for INFRA outright. **Recommended as the first follow-up dispatch.**
- **Not re-verified**: `DCC_CONTROL_INDEX.md`'s own 452-row counts. That
  document indexes the design; this one indexes the shell. Where they disagree
  about engine capability, this document is newer (the `#[func]` surface went
  from 38 to 151) and says so per row.

---

## 12 · Verification

This is a documentation task, so the verification is accuracy.

**Read in full** (not summarised, not grepped): all 18 files under
`godot-project/shell/` and `shell/workspaces/` (13 112 lines),
`DCC_SHELL_SPEC.md` (834), `DCC_CONTROL_INDEX.md` (1 093),
`JOURNEY_PLANNER_SPEC.md` (138), `TRAVEL_LIBRARY_SPEC.md` (128),
`design/cartalith-menu-structure.md` (202), `TIMELINE_SCOPE.md` §6,
`STRANDED_TOOLS.md`, and `git show 595582d` in full.

**Engine claims opened rather than inferred** — every one of these changed a
classification or a §4 correction:

| Claim | Where checked |
|---|---|
| `jp_journey_cost` is ported, golden-tested, and never called | `cartalith-civ/src/lib.rs:6885-6941` (the function), `:12659-12674` (the test), and `cartalith-godot/src/journey_bridge.rs` grepped for `jp_journey_cost`/`JpCost` — **zero hits** |
| Its inputs are all already computed | `JpDerivedStage::claimed_frac` (`lib.rs:9314`, written at `:9654`), `JpJourneyPlan::transshipments` (`:9887`, written at `:10164`) |
| `civ_faction_territory_stats` and `get_factions().claimed_cells` both exist | `cartalith-godot/src/lib.rs:3442-3461`, plus the full `#[func]` enumeration |
| The `#[func]` surface is 151 methods across 15 modules, not 38 | enumerated from `cartalith-godot/src/*.rs` |
| `travel_bridge.rs` has no `#[func]` layer and `WorldGen` holds no `TravelLibrary` | that module's own doc, lines 21-34, plus grep for `#[func]` in the file — **zero** |
| `icon_bridge.rs` genuinely has no handle geometry | `icon_bridge.rs:216` — *"`icon_hit_test`, `None` handle — no on-canvas resize-handle geometry"* |
| LOD tiling is live in the viewport | `viewport_host.gd:134-181, 609-637` (`_lod_backlog`, `MAX_LOD_TILES_PER_UPDATE`, `lod_synthesize_tile`) |
| `set_dock_readout("right", …)` is never called | grepped `right_dock.gd` — the left dock's call is in `world_workspace.gd:452-460` |

**Marked uncertain rather than guessed** (6 rows): §7's reading of *Blocks /
Verticality* (CA-09 — recommended as a question, not a design); whether §7's
"Strength" is `relief_gain` or `relief_directionality` (CA-01); whether stage
07's "lakes as water" is a classification or a display switch (WW-08); whether
the rail's expanded sub-nodes are labels or a different list (SH-01); whether
faster-mode advisories exist in `jp_compute`'s output at all (JP-03); and which
domain owns terrain appearance, CARTO or RENDER (§8.6).

**Build check**: `Godot_v4.7.1-stable_win64_console.exe --headless --path
godot-project --quit` — clean, after all five §4 edits. No parse errors, no
missing-method warnings.

**Web research**: 10 searches across Blender, Photoshop, Krita, DaVinci Resolve,
QGIS, ArcGIS Pro, Mapbox Studio, Gaea, World Machine, Unreal, Wonderdraft and
Inkarnate. Every claim in §7 that is attributed to a comparable application
carries its source URL inline.
