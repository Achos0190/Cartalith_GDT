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
| [13](#13--the-v210-menu-structure-audit-2026-08-20) | **The v2.10 menu-structure audit** — `design/Cartalith Menu Structure v2.dc.html` against the shipped shell, and the 17 undisclosed omissions it found |
| [14](#14--visual-sweep-2026-08-20) | **Visual sweep (2026-08-20)** — the shell driven live, screenshotted, and compared against the DCC Shell / Journey Planner mockups. **§14.6 corrects one of its own verdicts**: the Asset library window was passed on function rather than layout, and has been rebuilt against the canvas. |
| [15](#15--the-phone-overflow-menu-is-wired-but-inoperable-2026-08-20) | **The phone overflow menu (2026-08-20)** — (C): the real menu bar is wired into the phone sheet but is unscaled, buried in desktop status chrome, and inert to touch. Device evidence, kept as the brief for the mobile menu design; **not fixed**. |

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

**Stale as of 2026-08-20**: ten of the (B)-wrapper rows counted above
(AS-01 through AS-08, AS-13, DM-05) moved to done in that pass
(`ASSET_LIBRARY_SCOPE.md` §10), and two more (**JP-02**, **IN-06**) closed
with the Travel Library's party-form wiring the same day
(`TRAVEL_LIBRARY_SPEC.md` §6) — the totals/percentages in this section are
not yet re-derived across the whole 123-entry register to reflect either;
§6.3, §6.4, §6.9 and §6.12's own rows are the accurate, current source for
the Assets/Data/Journey/INFRA sections specifically. **Also stale as of
2026-08-23**: §6.16 (Urban morphology, `PARITY_AUDIT.md` C3) added three
more (B)-large entries (UM-01/02/03) that were not previously catalogued
anywhere in this register, and §5's O4/O5/O7/O8 moved from open to done
(`PARITY_AUDIT.md` C5) — neither is folded into the totals below either, for
the same reason: a full re-derivation is `PARITY_AUDIT.md` §8 item 2's own
recommendation for a dedicated pass, not a mechanical correction.

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

- ~~`right_dock.gd:674` Region select ▸ *"the Data Manager panel to call it doesn't exist yet"* — the Data manager **window** now exists, but the Export ▸ Maps **panel** genuinely does not. The wording says "panel". Accurate as written.~~ **Superseded 2026-08-20**: the Export ▸ Maps panel was built (§14.7), so the sentence stopped being accurate and the disabled button became live. Both the tooltip and the disable are gone — RD-09 above.
- `cartography_workspace.gd:277` *"no on-canvas resize handle yet for a placed icon (`icon_bridge.rs`'s own acknowledged gap)"* — `icon_resize`/`icon_hit_test` **are** exposed, so the attribution reads as more engine-blocked than it is; but `icon_bridge.rs:216` really does say *"`None` handle — no on-canvas resize-handle geometry"*, i.e. there is no `icon_handles()` to match `label_handles()`. The claim is true; only the emphasis is off. Left alone, recorded as entry **CA-05** below (an (A) item).
- `infrastructure_workspace.gd:13-14`'s class doc — *"Logistics … exports nothing past that crate boundary"* — is stale, but the same file's `_build_logistics()` says so explicitly two hundred lines later. A code comment, not user-facing text. Left alone.

---

## 5 · Omissions: designed, not present, not even as a disabled item

The honesty rule has two halves — *never enabled-and-inert*, and *never
omitted*. The first half holds everywhere. The second had **nine breaches**
when this table was first built; **six are now closed** (O1, O3, O4, O5, O7,
O8 — O4/O5/O7/O8 corrected 2026-08-23, `PARITY_AUDIT.md` C5, having sat
marked open here while §6.5/§6.6/§6.9 of this same file already recorded
them done). **Three remain real breaches**: O2, O6, O9. Each is catalogued
below with its class; listed together here because they are a different
kind of finding from a disabled item.

| # | Missing surface | Designed in | Class |
|---|---|---|---|
| O1 | **`Data ▸ ⧉ Travel library… ⇧L`** — the whole menu item and window | `DCC_SHELL_SPEC.md` §2.4's 2026-08-19 addition; `TRAVEL_LIBRARY_SPEC.md` in full | **done, 2026-08-19** — see DM-15 |
| O2 | **`Assets ▸ Asset pack ▸`** — the entire submenu (Active pack / Pack metadata… / Edit / Batch / Build / Clear library…), 24 controls | `DCC_SHELL_SPEC.md` §2.3.1 | (B) wrapper |
| O3 | **`Preferences ▸ Performance ▸ Fallback when VRAM full`** | `DCC_SHELL_SPEC.md` §2.5 | **done, 2026-08-20** — see PR-05 |
| O4 | **`Preferences ▸ Application ▸ Theme ▸ follow system`** | `DCC_SHELL_SPEC.md` §2.5 | **done, 2026-08-19** — corrected 2026-08-23 (`PARITY_AUDIT.md` C5); see PR-14 §6.5, verified live via `DisplayServer.is_dark_mode()`/`is_dark_mode_supported()` in `menus.gd:570,890` |
| O5 | **`Window ▸` the workspace list**, and **open windows listed while open** | `DCC_SHELL_SPEC.md` §2.6 | **done, 2026-08-19** — corrected 2026-08-23 (`PARITY_AUDIT.md` C5); see WI-02/WI-03 §6.6, verified live: `menus.gd:927-937` builds the `Workspace` submenu over `DccShell.DOMAINS`/`select_domain()`, and the Open-windows list rebuilds every `about_to_popup` |
| O6 | **New world ▸ project *name* field** | `DCC_SHELL_SPEC.md` §2.1 ("Modal: name, seed, extent, working resolution") | (B) small |
| O7 | **The Journey Planner's timeline band** — "one band per day, coloured travel / water / weather hold / rest-layover". `timeline_bar` is *visible and empty* while JOURNEY is armed. | `JOURNEY_PLANNER_SPEC.md` §2 | **done, 2026-08-19** — corrected 2026-08-23 (`PARITY_AUDIT.md` C5); see JP-13 §6.9, verified live: `_rebuild_timeline_band()`/`_TimelineBandView` in `journey_planner_view.gd` |
| O8 | **Blocked-stage inline resolutions** — "offers its resolutions inline (turn off closures, re-route land-only, depart earlier)" | `JOURNEY_PLANNER_SPEC.md` §9 | **done, 2026-08-19** — corrected 2026-08-23 (`PARITY_AUDIT.md` C5); see JP-14 §6.9, verified live: `_blocked_resolution_row()` in `journey_planner_view.gd` |
| O9 | **The right dock's `Layers` context** — §6 lists eight contexts; seven are built, `Layers` is not (only the viewport popover and CARTO's toggles exist) | `DCC_SHELL_SPEC.md` §6 | (B) large |

Two more absences are **deliberate and documented in-file**, so they are not
breaches: the POI tool (`civilization_workspace.gd:94-101` — omitted rather than
built inert, because no `civ_drop_poi` exists) and the `Brush / Stamp` right-dock
context (`right_dock.gd:685-696` — merged into `Stamp stack` on the stated
ground that two views of one state would fight). Both are (D).

> **This list was incomplete, and knowably so: it was derived from
> `DCC_SHELL_SPEC.md`, which is a design for the shell rather than an inventory
> of the app being ported.** Auditing the same shell against
> `design/Cartalith Menu Structure v2.dc.html` — the exhaustive v2.10 surface
> catalogue — found **seventeen more** omissions of exactly this kind, eleven
> of them whole-network civ operations and generation passes that `generate()`
> absorbed. All seventeen are catalogued and closed in **§13**, which is the
> continuation of this section rather than a separate finding. **CV-07** is the
> one row this section and §13 share: it was registered here as absent with no
> disclosure, and now has one.

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
| ED-03 | Cut / Copy / Paste / Delete | 176-179 | nothing selectable beyond settlements, which are read-only | **corrected 2026-08-23** (`PARITY_AUDIT.md` C3/§3.2/§5 item 3) — this was mischaracterized as a clipboard/selection gap. The real finding: `civ_drop_settlement` **creates** a settlement and nothing **edits, moves or deletes** one — there is no place-edit popup (the reference's `placeEditPopup`/`_civPopulatePlaceEditor` has no port, name/kind/faction/pop/specialisation/traits/history/walls-override/delete all absent), no right-click context-menu handler on the map (`_civCtxShow`'s six operations have no counterpart — `PopupMenu` appears only in `menus.gd`/`dcc_shell.gd`, never on `MOUSE_BUTTON_RIGHT` over the viewport), and no `KEY_DELETE` handler anywhere under `godot-project/` (grep confirms). Labels, icons and sculpt stamps genuinely are selectable and deletable through their own panels, which is why the *original* framing looked plausible — but a user who drops a settlement by mistake, or wants to rename/relocate/remove one, has no path to do so at all, not merely a missing uniform selection model. | §2.2 | (B) large — a place-edit popup, a map context menu and a Delete-key handler are three separate missing pieces, not one selection abstraction |
| ED-04 | Select all / Deselect | 181-182 | same | same | §2.2 | (B) large — same model |
| ED-05 | Find on map… | 184 | no search index; settlement search lives in the Data manager | yes | §2.2 gives one line; **no search UI design** | **(C)** → §7.2 |

> ED-01/ED-03/ED-04's reasons are stale in *emphasis* — they describe a shell
> that had no tools. They are not corrected here because rewriting them
> correctly means describing the global-undo/selection split, which is a
> paragraph, not a tooltip. Recorded rather than half-fixed.

### 6.3 Assets menu + Asset library window

| # | UI label | Where | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| AS-01 | Import image… | `menus.gd:201`, `asset_library_window.gd:402` | **done, 2026-08-20** | real | §2.3, §8 | `as_import_item`/`as_add_custom_slot` (`asset_bridge.rs`) are wired; targets whichever slot is focused in the grid |
| AS-02 | Apply library to map | `menus.gd:241`, `asset_library_window.gd:325` | **done, 2026-08-20** | real | §2.3, §8 | `as_apply_to_map` — the reference's own `applyToMap()`: bake the session in memory (`export_pack_bytes`), load it straight into the renderer, no round trip through a file |
| AS-03 | Clear library… | `menus.gd:243`, `asset_library_window.gd:513` | **done, 2026-08-20** | real | §2.3, §8 | `as_clear_library` -> `AssetDB::clear` |
| AS-04 | Export pack .zip | `asset_library_window.gd:327` | **done, 2026-08-20** | real | §8 | `as_export_pack_bytes` — bakes every item, builds a schema-2 manifest, `archive::write_pack`; disk round-trip verified headlessly (`ASSET_LIBRARY_SCOPE.md` §10) |
| AS-05 | Validate | `asset_library_window.gd:511` | **done, 2026-08-20** | real | §8 | `as_validate` -> `library::run`, shown in a modal |
| AS-06 | Tag… / Collect… / Rename… / Duplicate / Delete (batch) | `asset_library_window.gd:436` | **done, 2026-08-20** | real | §8, §2.3.1 | `as_batch_tag`/`_collect`/`_rename`/`_duplicate`/`_delete`, each read off the reference's own `alBatch*` handlers. `rename` stays honestly split: a custom slot is renamed for real, a frozen slot renames its *item variants* (`AssetDB::item_mut`, new this pass) — frozen slot names are the constant `slot_title`, not editable at all (the real spec/engine disagreement is unchanged, just no longer blocked on a missing binding) |
| AS-07 | Slot inspector: File / Scale / Tags / Pack metadata | `asset_library_window.gd:704-707` | **done, 2026-08-20** | real | §8 | `as_slot_summary`/`as_item_summary`/`as_pack_info` — File/Scale/Tags/Pack metadata all show real values now. Editing scale/pan is not yet wired (no `as_set_item_transform`); noted honestly in the inspector, not silently absent |
| AS-08 | Per-slot fill state + thumbnails (grid is always a checkerboard) | `asset_library_window.gd:579, 690` | **done, 2026-08-20** | real | §8 | `as_family_slots`/`as_thumbnail_png` — every filled slot shows a real `render_item`-baked thumbnail; empty slots still show the honest checkerboard |
| AS-09 | Sprite-sheet **Slice** | `asset_library_window.gd` slicer modal | **done, 2026-08-20** | real | §8's slicer modal | `cartalith-assets::slicer` is a golden-verified port of the reference's `SpriteSheetImporter` (`computeCells`/`cropCell`/`applyChroma`/`isBlank`, HTML lines 27465-27870); `as_load_sheet`/`as_slice_preview`/`as_slice_apply` expose it. The `N cells detected · M non-empty` readout is now the engine's **real** detection pass — the 8×8 GDScript sample it replaced was labelled approximate and is gone — and the grid overlay draws engine-computed cell spans, so it shows the exact rectangles the slice cuts. Non-destructive: the sheet stays loaded for a re-slice |
| AS-10 | Slicer: Trim transparent edges / Skip empty cells | `asset_library_window.gd` slicer modal | **done, 2026-08-20**, with one disclosure | real | §8 | *Skip empty cells* is a straight port (`isBlank`, alpha **> 8**, golden-pinned on both sides of the boundary). *Trim transparent edges* is a **port-side addition, not a port** — the reference slicer has no trim operation at all; its second pixel toggle is `background → transparent` chroma keying, which is now wired here too. Trim reuses the reference's own alpha>8 threshold so it can never disagree with `isBlank` about what content is (`slicer.rs` module docs; `CHANGELOG.md` discloses it per `CLAUDE.md`'s no-silent-deviation rule) |
| AS-11 | Slicer: Assign to family / Fill from | `asset_library_window.gd` slicer modal | **done, 2026-08-20** | real | §8 | All four targets are offered. §8's *Assign to family* + *Fill from first-empty/overwrite* is the framing the **reference expresses as a flat target-slot dropdown** instead, so the family target is composed from the reference's own primitives (one cell per slot, in frozen vocabulary order) rather than ported; the reference's own three targets (focused slot, one new custom icon, separate custom icons per cell) are ported exactly, including `store[uid]=[item]`'s replace-and-stop for a single-image family |
| AS-12 | Family rail: **Collections** and **Unassigned imports** | *absent* | ~~none~~ | — | §8's rail lists both | **PARTLY CLOSED 2026-08-23** — a real **Collections** rail section exists (`_build_family_rail`/`_refresh_collections_rail`), listing every `as_collections()` entry with a live member count, selectable into a real collection-scoped grid view (`_select_collection`/`_refresh_grid_collection`, resolving each member uid through `as_slot_summary`). Also gained real drag-and-drop: one or more selected slot tiles dragged onto a Collections row add themselves to it (`SlotCell._get_drag_data` / `CollectionRow._can_drop_data`/`_drop_data`, calling the same `as_batch_collect` the Collect… prompt uses). New `#[func] as_collections` (`lib.rs`) is the read side `as_batch_collect`/`as_slot_summary` never had — nothing could previously enumerate every collection that exists. **"Unassigned imports" stays open** — still a slot-less bucket the model does not have. |
| AS-13 | **`Assets ▸ Asset pack ▸` submenu** (24 controls) | *absent — omission O2* | **done, 2026-08-20** | real | §2.3.1 in full | `menus.gd::_build_asset_pack_submenu` — Active pack (live name/author/license/schema/filled-item stats), Pack metadata…, Build ▸ (Validate/Apply to map/Import pack/Export pack, all direct engine calls), Edit ▸ and Batch ▸ (both open the real window, since every one of their controls needs slot/selection context only the grid provides — real navigation, not a disabled item). The one still-gap item (Slot transform editing) is disabled with its real reason, matching AS-07's note |
| AS-14 | Variants strip / "active variant" | *absent* | none | — | §8 | **(D)** — engine truth: variant choice at render time is weighted and seeded (`pick_weighted_variant`); a user-picked "active variant" has no counterpart. `DCC_CONTROL_INDEX.md` §3(f). |
| AS-15 | Per-slot Anchor (top/centre/base) | *absent* | none | — | §8 | **(D)** — engine truth: `Anchor` is a **family** property `sprite_draw_rect` depends on, not per-slot. §3(f). |
| AS-16 | 24-family rail vs the shipped 8 | `asset_library_window.gd:8-23, 360` | disclosed in the window's own note and header comment | yes | §8 says 24; mockup shows 11; engine has 8 | **(D)** — owner decision, `DCC_CONTROL_INDEX.md` summary §5 item 9 |
| AS-17 | Slicer: canvas interaction | `asset_library_window.gd` slicer modal | ~~the reference's pan/zoom, draggable grid lines and click-to-select-cells are unported~~ | — | §8 | **PARTLY CLOSED 2026-08-23** — `SheetPreview` (the slicer's sheet-preview canvas) gained real wheel-zoom (centred on the cursor, reversible — zooming in then back out returns to the exact prior pan), middle-drag pan, and click-to-select-a-cell (a real picker/highlight, hit-tested against the engine's own `as_slice_preview` cell spans). One real draggable grid line: a handle on the grid's own **Margin** boundary (`GridRect::inset`, the one uniform parameter the engine actually has), writing straight to the Margin spinbox so the existing `_refresh_slicer_summary` pipeline stays the single source of truth. **Stays open, honestly**: per-*interior*-line dragging — `cartalith_assets::SliceGrid`/`compute_cells` computes a uniform grid from `cols`/`rows`/`margin`/`spacing` only, so no engine call has anything for an arbitrary interior line to move; and picking a cell does not narrow what Slice cuts — `as_slice_apply`/`slice_target_from` (`lib.rs`) have no cell-selection parameter, so the modal still slices the whole uniform grid, just with a real click-through picker on top of it now. |

### 6.4 Data menu + Data manager window — `menus.gd`, `data_manager_window.gd`

All thirteen `"kind": "gap"` routes, plus the window's own foot and route pane.

| # | Route / control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| DM-01 | Import ▸ Heightmaps (PNG) | 52 | **done, 2026-08-20** | real | §2.4 names it | now a `"live"` route: `DccApp.open_heightmap_import()` → `EngineBridge.import_heightmap` → `WorldGen::import_heightmap`, which decodes the PNG (`cartalith-assets::raster::decode_png`), resamples it at the *image's* aspect ratio and runs `cartalith_engine::import::infer_tectonics` under it — MS-02's other half, same pass |
| DM-01b | Import ▸ Maps (tiles) **and** Import ▸ GIS / GeoJSON — two rail rows since 2026-08-20, as the canvas has them (they were one concatenated row) | 53 | no tile-map or GeoJSON **import** path exists; TIFF absent | yes | §2.4 | (B) large — the remainder of DM-01 after the heightmap half landed. **TIFF is now a closed question, not a pending dependency decision**: the reference's own file input is `accept="image/*"` decoded by the browser, which does not read TIFF either, so PNG-only is parity rather than a shortfall |
| DM-02 | Export ▸ Maps (image · tiles) | 51 | **half done, 2026-08-20** — tile export is real; the *pyramid* is not | partly | §9's route pane, **the one fully-designed route in the window** | **The route is live.** §9's full pane shape is built (§14.7) and calls `region_export_tiles` over the live Region-select marquee, writing a zipped `cols × rows` grid — verified end to end: 33 entries, `tiles/index.json` present. What remains of this row is the *slippy-map* half the canvas draws and the engine has no notion of: XYZ/TMS/WMTS addressing, a zoom ladder, retina @2x variants, ocean-tile skipping, `leaflet-preview.html`/`style.json`. All of those are drawn in the pane and disabled with that reason. Still (B), now medium rather than large |
| DM-03 | Export ▸ GIS / GeoJSON | 53 | `cartalith-engine::geojson` exports region GeoJSON for Region-select only, no route in, no CRS | yes | §2.4 | (B) wrapper — `export_geojson` is golden-verified; needs one `#[func]` plus assembling `GeoJsonWorld` |
| DM-04 | Export ▸ World Data | 55 | no save writer | yes | §2.4 | (B) large — FI-01's writer |
| DM-05 | Export ▸ Assets (pack .zip) | 57 | **done, 2026-08-20** | real | §2.4 | now a `"route"` (was `"gap"`) into the Asset library window's real `export_pack_now()`, same "routes, doesn't reimplement" shape as `import_assets` — same as AS-04 |
| DM-06 | Sources ▸ External / Connected / Registry | 59-61 | no source registry exists | yes | §2.4 names three rows; **§9 designs no pane for any of them** | **(C)** → §7.3 |
| DM-07 | Conversion ▸ Coordinate Systems (EPSG ▸) | 62 | ~~no CRS conversion~~ | — | ~~§2.4 names it~~ | **RESOLVED BY DELETION** (owner, 2026-08-20) — §7.4's research accepted in full. The route is gone from `menus.gd::_data()` and from `data_manager_window.gd`'s `ROUTES`/`GROUP_ORDER`; the Data manager now has **four** groups. |
| DM-08 | Conversion ▸ Format Conversion | 64 | ~~no format-conversion routes~~ | — | ~~undefined even in the spec~~ | **RESOLVED BY DELETION** (owner, 2026-08-20) — same decision, same commit. |
| DM-09 | Conversion ▸ Data Transformation | 65 | ~~no data-transformation routes~~ | — | ~~undefined in the spec~~ | **RESOLVED BY DELETION** (owner, 2026-08-20) — same decision, same commit. §7.4 recommended dropping this row outright and that is what happened. |
| DM-10 | Validation ▸ Check Data | 66 | `load_save()` returns pass/fail only; no warning collection anywhere | yes | §2.4 names it ("shows current warning count"); what is validated, and against what invariant, is undefined | **(C)** → §7.5 |
| DM-11 | Validation ▸ Repair / Normalize | 68 | no validation pass to repair against | yes | undefined | **(C)** → §7.5 |
| DM-12 | Foot: "last run (`14:02 · 62 MB`)" | 160 | **partly done, 2026-08-20** — real, session-scoped | partly | §9 | The rail foot and the RECENT RUNS column both report the real runs of *this session* (stamp, label, measured bytes, ✓/✕) and say plainly that nothing persists across a launch. (B) small — a persisted history is a `DccSettings` section nothing writes yet |
| DM-13 | §9's route pane: TILES / PROJECTION / LAYERS INCLUDED / OUTPUT / ESTIMATE / RECENT RUNS | **done, 2026-08-20** | real | n/a | §9, designed in full | Built for Export ▸ Maps, the one route §9 designs a pane for: the canvas's two-column grid, all seven column blocks, the `120px label · control` row grammar, the segment/well/`☑` vocabulary, the bordered ESTIMATE block and the `Save as preset · Dry run · Export N tiles` footer. Controls with no engine behind them are drawn in place and disabled with their reason. Every other route keeps a one-column pane in the same grammar. See §14.7 |
| DM-14 | §9's **MARKDOWN VAULT · LINKED** block | *drawn, quiet, disabled* | — | — | §9 designs it; `MARKDOWN_VAULT_INTEGRATION.md` is explicitly *"Not started; no code exists"* and its §33 lists two-way sync as a V1 **non-goal** | **(D)** — owner decisions 3 and 4, `DCC_CONTROL_INDEX.md` summary §5. Since 2026-08-20 the block exists in the pane in the canvas's shape, but bordered **quiet rather than accent** and reading `MARKDOWN VAULT · NOT LINKED` / `○ no vault linked · 0 notes`: the canvas's block asserts a live link, and drawing that would be the one kind of fiction this window avoids. All six controls disabled with the reason |
| DM-15 | **`Data ▸ ⧉ Travel library… ⇧L`** | **done, 2026-08-19** | real | — | §2.4's addition + `TRAVEL_LIBRARY_SPEC.md` in full (fields, validation states, placement, §6 build-status) | **Done.** `lib.rs`'s `WorldGen` now carries a live `travel_library` field (persists across a re-generate, like `asset_pack`) and a full `tl_*` `#[func]` CRUD+query surface; `jp_compute` builds a `JpAnimalResolver` from it and calls `jp_plan_ex` unconditionally (a stock-only library is regression-tested identical to the old `jp_plan` call). `travel_library_window.gd` is the real `2a`/`2b` window, wired at `⇧L`. See `TRAVEL_LIBRARY_SPEC.md` §6 for the full record and the two things still honestly not wired (the planner's own party-form dropdown does not yet offer a custom entry; only the four built-in species can affect a computed plan). |

### 6.5 Preferences menu — `menus.gd`

| # | UI label | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| PR-01 | Devices | **done, 2026-08-20** | real | — | §2.5's per-device checklist, minus its "live utilisation" | **Done, with one disclosed impossibility.** New `cartalith-gpu/src/multi.rs`: `enumerate_devices()` folds `wgpu`'s per-*adapter* rows into one entry per *physical* GPU (this machine returns **six adapters for three devices**, and the OpenGL row reports `vendor = device = 0`, so grouping keys on PCI identity with a name-matching fallback — both cases unit-tested, and keying on name alone would have merged two identical cards). `Preferences ▸ Devices` is a live checklist; unchecking everything returns to automatic. Selecting a device really opens *that* device, asserted per device by `every_enumerated_device_can_be_selected_and_opened`. **§2.5's `71%` utilisation is not implementable and is not faked**: `wgpu` 30 has no system-wide utilisation query and no VRAM size on any backend. The footer shows the one real number there is — this app's own allocation total from `Device::generate_allocator_report()`, measured at the last GPU generation and labelled as ours. See `HARDWARE_ACCELERATION.md`'s 2026-08-20 section. |
| PR-02 | Multi-GPU mode | **done, 2026-08-20** | real | — | §2.5 | **Owner decision 2 answered: build it.** `single device` and `split tiles` are real; **`alternate frames` is refused** (`gpu_set_multi_mode` returns `false`) — §2.5's own note is that it only helps the 3D viewport, and there is none. `split tiles` partitions **the domain-warp stage only**, the one GPU stage here whose kernel reads nothing outside its own cell; blur needs a halo, JFA and flow accumulation read across the whole grid (the full audit is in `warp_grid_gpu_split`'s doc comment). Measured on this machine (RX 7800 XT + integrated Radeon): **1.22-1.54x at 4096², but 0.73-0.81x at 2048² and below** — the second device's ~1.8 ms fixed cost exceeds a sixth of a small dispatch. Band sizes come from measured per-device throughput (integrated = 0.17 of discrete), not a guess. The default ships as `single device` rather than §2.5's `split tiles`, with those numbers in the row's tooltip. |
| PR-03 | CPU worker threads | 316 | Rayon sizes its own pool; no override exposed | yes | §2.5 | (B) wrapper — one `ThreadPoolBuilder` call at startup; the *default* (cores − 4) is owner policy |
| PR-04 | VRAM budget | **done, 2026-08-20** | real | — | §2.5 | **Done as a cap; its stated default is impossible.** `Preferences ▸ VRAM budget` sets a GB cap that gates the GPU path per grid, compared against a documented upper bound (`gpu_working_set_bytes` — ten `f32` grids, derived from the storage-buffer count the heaviest stage binds plus its staging buffers, not guessed). **§2.5's "default 75 % of the smallest active device" cannot be computed**: `wgpu` 30 reports no VRAM size for an adapter, and `Adapter::limits()` is an API limit — this machine reports the same 2 GB `max_buffer_size` for a 16 GB card and a shared-memory iGPU. The default is therefore *no cap*, with that reason in the row rather than a fabricated percentage of an unknown quantity. |
| PR-05 | Fallback when VRAM full | **done, 2026-08-20** | real | — | §2.5 | **Two of three real, the third refused.** `CPU tile pass` (default) is exactly what the engine already does whenever the GPU path is unavailable — wiring it discloses existing behaviour rather than adding any. `Fail with error` is real and lives in `EngineBridge.generate()`, because `generate_terrain` returns a world rather than a `Result` and refusing-with-a-reason is a UI act. **`Reduce working res` is refused** (`gpu_set_vram_fallback` returns `false`): nothing in this pipeline computes a stage at a reduced grid and resamples back up — LOD tile synthesis resamples an already-finished field, which is a different operation. Closes omission **O3**. |
| PR-06 | Anti-aliasing · anisotropy | 333 | the 2D map path doesn't sample-antialias; belongs to the 3D viewport | yes | §2.5 | (B) large — gated on the 3D viewport |
| PR-07 | Colour management | 334 | the renderer is sRGB-only | yes | §2.5 gives **three values and nothing else** | **(C)** → §7.6 |
| PR-08 | 3D viewport defaults | 335 | no 3D viewport | yes | §2.5 names four fields | (B) large — `DECISIONS.md` §4 defers 3D; `ROADMAP.md` Phase 3 |
| PR-09 | Lighting rig defaults | 336 | no lighting rig yet | **stale in flavour**: there is no *rig*, but all six fields are real and drive the current render (`TerrainAppearance::{sun_az_deg, sun_alt_deg, relief_ambient, relief_gain, relief_lights, relief_directionality}`) | §2.5 | (B) **wrapper** — one `set_appearance()`-shaped `#[func]`; the same one CA-01 needs |
| PR-10 | Tiled LOD · tile size · atlas cache | 338 | **corrected — S4** | yes, now | §2.5 gives four rows of values | **(C)** for the atlas-cache design → §7.7 |
| PR-11 | Memory ▸ Undo history | 339 | no undo stack | yes | §2.5 gives a range and a default | **(C)** — depends on ED-02's undesigned model → §7.1 |
| PR-12 | Memory ▸ Clear caches… | 348 | no atlas or field cache exists to clear | yes | §2.5 | (B) small — gated on PR-10 |
| PR-13 | Theme ▸ Light | 362 | **done 2026-08-19** — `DccTheme.apply_theme()`/`remap()` + `DccShell.rebuild_theme()` walk the tree and repaint every token-derived colour in place; Light is a live radio choice | yes | §2.5 + §11's full light token column | **(A)** — a rebuild pass in `DccTheme`/`DccShell`, no engine at all |
| PR-14 | Theme ▸ follow system | **done 2026-08-19** — a third radio item, `DisplayServer.is_dark_mode()` resolved once | none | — | §2.5 | **(A)** — Godot exposes the OS preference; the rebuild pass is PR-13's |
| PR-15 | Units (km · mi) | 368 | the shell is km-only; the reference's mi toggle is not ported | yes | §2.5 gives two values, **and §5.1 stage 02 gives the same control a second home** — an unresolved ownership collision (`DCC_CONTROL_INDEX.md` §3(j), owner decision 15) | **(C)** → §7.8 |
| PR-16 | Keyboard shortcuts… | 369 | no shortcut table yet | yes | §2.5 says *"Editable table, per-context"* and nothing more | **(C)** → §7.9 |

### 6.6 Window menu — `menus.gd`

| # | UI label | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| WI-01 | Save layout as… | 407 | no layout store yet | yes | §2.6 names it | **(C)** → §7.10 |
| WI-02 | The workspace list | **done 2026-08-19** — Window ▸ Workspace submenu over `DccShell.DOMAINS`, via a new public `select_domain()` | none | — | §2.6 | **(A)** — `_select_domain()` and `DOMAINS` already exist |
| WI-03 | Open windows listed while open | **done 2026-08-19** — Window ▸ Open windows, rebuilt every `about_to_popup`; the count is five now, not four (`new_world_dialog` had joined the other four) | none | — | §2.6 | **(A)** — four windows exist and all are `AcceptDialog`s on `DccApp` |
| WI-04 | Dock width dragging (§1: "user-draggable within min/max") | **done 2026-08-19** — a real 6 px grip per dock, clamped to §1's min/max | none | — | §1's geometry table gives min/max for both docks | **(A)** — pure GDScript; the collapse chevron already exists |

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
| RD-09 | Region select ▸ Send to Data ▸ Export | 672-674 | **done, 2026-08-20** | real | §4.5.1 + §9 | **Closed.** DM-13's route pane is built, so the button is live and opens the Data manager straight onto Export ▸ Maps with the marquee already read (`DataManagerWindow.open_tile_export()`). Deliberately *not* the cheap path the old note offered (a bare `FileDialog` off the dock button): §4.5.1's own wording is "Send to Data ▸ Export", and the marquee and that route's world-bounds fields are two views of one rect, which only holds if the route is where the export happens. |
| RD-10 | **`Layers` context** | *absent — omission O9* | none | — | §6 designs it (ordered list, visibility dot, opacity bar, blend mode, nested children under Terrain) | (B) large — opacity is cheap (overlays carry alpha); blend mode and reorder need the three overlays to become independently compositable, an architecture change `GUI_FEATURE_PARITY_SCOPE.md` Category 3 already recommended deferring |
| RD-11 | Collapsed right dock's primary readout | — | none | — | §6's last line: *"elevation for Sample, layer dots for Layers, stamp count for the stack"*. `DccShell.set_dock_readout("right", …)` exists and **`right_dock.gd` never calls it** — the left dock's is wired (`world_workspace._push_dock_readout`), the right dock's is not | **(A) — done 2026-08-19**: `_push_dock_readout()` called at the end of `_rebuild()` and live from `on_cursor_sampled`; one real reading per existing context (elevation, settlement name, faction id+culture, route length, chain/region/stamp counts, journey days·km). No "Layers" context exists yet (RD-10). |
| RD-12 | `Brush / Stamp` context | 685-696 | merged into `Stamp stack`, with the reasoning stated in-file | yes | §6 lists both | **(D)** — deliberate: both read the same live state and the eight globals already have live editors in WORLD's dock |
| RD-13 | Stamp stack ▸ finalize-lock note | 731-737 | no finalize/lock state exists in this engine | yes | §6 | (B) large — gated on WW-01 |

### 6.9 Journey planner — `journey_planner_view.gd`

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| JP-01 | Carriage **Auto** pick | 366 | `jpAutoPickTransport` has no Rust port | yes | `JOURNEY_PLANNER_SPEC.md` §5 ("in auto, counts are computed (terrain × biome, km-weighted) and read-only") | (B) small — a real, bounded port of one reference function |
| JP-02 | Party set-up picker + capture | `_preset_controls`/`_apply_preset`/`_capture_preset` | `JP_PRESETS` is JS-only; no `jp_presets()` binding | **CLOSED (2026-08-20)**. The tool-options bar now carries a live `set-up` dropdown over `tl_list("preset")` (stock and captured alike, custom rows tagged `· custom` and ⚠-marked by §4 validation state) plus a `capture party…` action writing the current form back through `tl_capture_preset_from_plan`. Deliberately **not** the reference's `JP_PRESETS`: this port's set-ups are the Travel Library's own stored rows, which is the strictly larger thing. Applying assigns only the keys `jp_default_plan()` owns — `tl_get("preset", id)` returns exactly `PRESET_FIELD_KEYS`, `PartyPreset::apply_to`'s own inverse — and leaves per-stage overrides untouched per §3.4 | §5 + `TRAVEL_LIBRARY_SPEC.md` §3.4 | — |
| JP-03 | Re-route for `<mode>`… | 1320 | `jpAutoPickTransport`/`_jpRerouteForMode` have no Rust port | yes | §6's "faster-mode advisories… with a **use here** action" | (B) small — sibling of JP-01 |
| JP-04 | **Cost** group | 1519 | **corrected — S3** | yes, now | §8 designs it in full (food/fodder · wages · tolls/ferry · animal upkeep · total · per km and per day) | (B) **wrapper** — `jp_journey_cost` is ported and golden-tested; `jp_compute` never calls it. **The single cheapest (B) in the register.** |
| JP-05 | Calculation trace ⧉ | 1553-1555 | no trace window; the `formula` string is deliberately not carried across the boundary (`jp_land_calc_dict`'s own doc: presentation, not engine) | yes | §8 says *"opens in its own window (⧉)"* and nothing about its contents | **(C)** → §7.12 |
| JP-06 | Save journey | 1325 | no save-writer for journeys or projects | yes | §2 lists it in the tool options bar | (B) large — FI-01's writer, plus a journey registry that does not exist |
| JP-07 | ⇧-drag spine trim | 1323 | `jp_compute` has no request field for trimming | yes | §3: *"⇧ drag trims"* | (B) small — a new `jp_compute` request field plus its handling |
| JP-08 | Journeys list = committed routes | 226, 250 | no named/persisted journey registry exists engine-side | yes | §3's "journeys list" | (B) large — same registry as JP-06 |
| JP-09 | Vessels ▸ sailing window | 1540 | not part of `jp_water_calc`'s return | yes | §8: *"per water leg: vessel, hold used, sailing window"* | (B) small — `TRAVEL_LIBRARY_SPEC.md` §3.3 models "sailing window (daylight / continuous)" per vessel; the resolver for vessels does not exist |
| JP-10 | Supply ▸ foraging offset | 1515 | folded into food/water totals; `jp_plan` doesn't break it out | yes | §8 lists it as its own figure | (B) small |
| JP-11 | Load ▸ speed penalty | 1500 | folded into each leg's km/day; `jp_plan` returns no separate percentage | yes | §8 | (B) small |
| JP-12 | Supply ▸ per-leg bar with resupply ticks | **done 2026-08-19** — `_build_reach_bar()`: one segment per gap between consecutive resupply stops (`_stop_fractions()`, the stops strip's own chord-length projection), `block` when that leg's own km exceeds `resupply_reach.required_km`, a tick at every stop | yes | §8 | **(A)**, closed — `resupply_reach` carries `max_gap_km`/`required_km`/`stops`/`unmet`; verified against a real route (9 real `ColorRect` children, 4 lit segments + 5 ticks, 2 legitimately zero-width where a stop coincided with the route's own start) |
| JP-13 | **Timeline band** (one band per day) | **done 2026-08-19** — `_rebuild_timeline_band()` populates `app.timeline_row` with a real `_draw()` day-band strip (`_TimelineBandView`) while JOURNEY is armed, cleared on disarm | yes | `JOURNEY_PLANNER_SPEC.md` §2 | **(A)**, closed — real `results[i].days` segments (land `accent`, water `water`) plus one trailing `rest_days+layover_days` block (`text_dim`); "weather hold" stays in the legend, never lit (`jp_plan` has no discrete weather-hold day count) — verified against a real 21.47-day plan, 15 segments summing exactly to `total_days` |
| JP-14 | **Blocked-stage inline resolutions** | **done 2026-08-19** — `_blocked_resolution_row()` in the verdict card and the stage inspector's own `BLOCKED:` box | yes | §9: *"offers its resolutions inline (turn off closures, re-route land-only, depart earlier)"* | **(A)**, closed — turn off seasonal closures (when `blocked_seasonal`), force Walking land-only (transport flip + zeroing carts/wagons — the wheel-block reads cart/wagon count, not `transport`), depart a season earlier (when not already first); the real Dijkstra "re-route land-only" reroute stays out of scope (JP-01/JP-03) — verified against two real blocked scenarios (an overload the buttons honestly cannot fix, and confirmed unreachable-otherwise) |
| JP-15 | Auto fields showing `auto · <resolved>` | **done 2026-08-19** — `_auto_label()`/`_party_auto_resolved()`, refreshed post-compute via `_refresh_auto_labels()` without a full form rebuild | yes | §5: *"Auto-valued fields show `auto · <resolved value>` so the resolved value is never hidden"* — implemented for stage overrides (`_inherit_label`), **now also** for the party form | **(A)**, closed — `route_cond`/`infra`/`rest_cadence`/`mount_animal`/`desert_water` resolve from the first applicable stage/leg; `weather_override` stays plain `Auto` (its auto is a continuous blend, not one resolvable value) — verified against real resolved strings (`"Auto · Dense Oasis Route"` etc.) |

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

> **Domain merge (2026-08-20, owner instruction: "Infra can be dropped as a
> name and can be absorbed by civil"):** INFRA is no longer a rail domain —
> §6.12 below is now reached through CIVIL, via `civilization_workspace.gd`
> composing an `InfrastructureWorkspace` instance into its own dock rather
> than that class getting a rail button of its own. §6.12's own rows are
> otherwise unchanged; the file and line numbers they cite still resolve,
> since `infrastructure_workspace.gd` itself was not rewritten, only
> repositioned. See `DCC_SHELL_SPEC.md`'s own correction notice for the full
> disclosure.

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| CV-01 | **POI tool** | 94-101 (comment) | omitted, not built inert: `civ_tools_bridge.rs` says POI *"is not a ported concept"*; no Rust function drops one | yes | §4.5.3 designs it in full (kind · faction · name · snap to way, plus a POI inspector) | (B) small — one `civ_drop_poi` mirroring `civ_drop_settlement`; `cartalith-assets`' `poi` family already carries the 10-slot vocabulary |
| CV-02 | Culture ▸ Profiles | 518-523 | `cartalith-civ` generates culture profiles internally; no `#[func]` exports them | yes | §3 lists Culture as one of CIVIL's five subjects | (B) wrapper — `civ_default_culture` is already called inside `get_factions()`; a fuller `get_cultures()` is one binding |
| CV-03 | Timeline filters (Exist only / Ghost removed / Highlight new) can't touch map pins | 821-827 | ~~`get_settlements()` carries no `tid` even though `NamedSettlement` has one~~ | — | `TIMELINE_SCOPE.md` m6 | **PARTLY CLOSED 2026-08-23** — `get_settlements()` (`lib.rs`) now carries `tid`. **Exist only** is wired for real: `civilization_workspace.gd`'s `_tl_apply_filters` filters the array handed to `map_overlay.gd`'s `set_civ_data` down to the active year's `civ_year_diff().present` tids, upstream of that file rather than inside it (territory). **Ghost removed / Highlight new** stay disclosed-open: both need per-pin fade/halo drawing (`map_overlay.gd`'s own `_draw()`, still territory this pass), and "removed" specifically needs the OLD snapshot's settlement data (position/name), which no `#[func]` exposes yet (`civ_year_diff()` returns tid sets only). |
| CV-04 | Settlement class list lacks **metropolis** | 233-239 (comment) | ~~five real `SettlementKind` tiers~~ | — | ~~§4.5.3 lists six~~ | **CLOSED 2026-08-20** — `_civSelectMetropolises` (reference 24961-24989) ported on the owner's decision. `SettlementKind::Metropolis` exists with the reference's own rank-5 tables; `kind_from_str` accepts it, `get_settlements()` reports it, `map_overlay.gd` draws it at rank 5 / glyph ★, and the promotion runs inside `compute_civilisation` behind `set_metropolis_enabled` (reference default OFF). Spec and engine now list the same six. |
| CV-05 | Territory ▸ "respect coastlines" | 298-304 (comment) | `civ_territory_paint_at` always pushes an ungated circular dab (`PaintStamp::ungated`); no coastline mask behind it | yes | §4.5.3 | (B) small |
| CV-06 | Settlement ▸ "pick radius" | 236-239 (comment) | `civ_drop_settlement` computes its own pick radius internally and takes no argument | yes | §4.5.3 lists it | **(D)** — engine truth; a slider would be decoration |
| CV-07 | Faction roster add/remove, persistent identity | *absent* | none | — | §6's Faction context implies a roster; `design/cartalith-menu-structure.md` §3.11 names "add/remove faction, faction roster `#civOpenFactionsBtn`" | (B) large — new Rust state; `CIV_FACTION_COUNT` is a constant |
| CV-08 | `_civApplyRecovery` / auto-populate's static "Recovery phase" | *absent* | ~~none~~ | — | `design/cartalith-menu-structure.md` §4 names it | **CLOSED 2026-08-20** — ported (reference 24619-24640) on the owner's decision, wired at the reference's own call site (line 25761) behind `set_recovery_phase`, and surfaced as a five-entry **Recovery phase** dropdown in `File ▸ New world ▸ Generation`, filled from the engine's own `_CIV_RECOVERY_NAME` table. Phase Stable is a strict no-op. |
| CV-09 | The timeline bar's **six simulation-layer toggles** (Climate · Population · Economy · Politics · Infrastructure · Warfare) | `dcc_shell.gd:628-641` builds an empty `timeline_row` | none in-product — `TIMELINE_SCOPE.md` §4 explains why the bar was left untouched | yes | §10 designs the whole region | **(D)** — `DCC_CONTROL_INDEX.md` summary §5 item 5 and `VISION.md`: the engine is a one-shot static generator by explicit, repeated owner decision. **The bar is drawn and empty in CIVIL/INFRA** — see §11. |

### 6.12 INFRA workspace — `infrastructure_workspace.gd` (now composed into CIVIL, §6.11)

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| IN-01 | Rivers ▸ Hydrology | 314-319 | no `get_rivers()`; the only river output crossing the boundary is baked into the rendered raster | yes | §3 lists Rivers as one of INFRA's five subjects | (B) large — same entity gap as RD-05 |
| IN-02 | Committed manual ways/routes never appear on the map or in a list | 20-31 (class doc), 195, 213 | `get_roads()`/`get_sea_routes()` read `civ.ways`/`civ.sea_routes` only, never `infra.ways`/`infra.routes`; `way_commit`'s own doc says the getter is out of scope | yes | §4.5.4's "Way inspector: waypoint list, length, grade profile, surface" | (B) small — one getter mirroring `route_get`'s shape |
| IN-03 | Way / Route ↶ ↷ (per-waypoint undo) | 232-236 (comment) | no per-waypoint undo in the engine; `InfraTools` only discards the whole draft | yes | §4.5.4 lists ↶ ↷ | (B) small |
| IN-04 | Way ▸ routing mode (freehand / snap / least-cost) | 229-231 (comment) | `infra_tools_bridge`'s own doc: *"nothing to build a 'freehand' or distinct 'snap' routing mode out of"*; snap is real but automatic | yes | §4.5.4 | **(D)** — engine truth, recorded in-file |
| IN-05 | Way types: spec says road/track/trail/bridge, engine has road/track/sea_lane/ancient | 42-49 (comment) | `parse_way_type`'s own doc calls the spec list wrong against the tested four-entry enum | yes | §4.5.4 | **(D)** — spec/engine disagreement, resolved in the engine's favour and recorded |
| IN-06 | Route ▸ vessel / party reference in the options row | `journey_planner_view.gd` `_vessel_field`/`_mount_field`/`_build_animal_definitions` | the journey planner exported nothing past the crate boundary when written | **CLOSED where it can be, and the remainder stated in-UI (2026-08-20)**. The party form's Mount picker and its four per-species **animal definition** pickers are now library-backed (`tl_list("animal")`, custom rows tagged `· custom`), and the choice reaches the engine: `jp_compute`'s new `animal_entries` request key → `TravelLibrary::animal_overrides_selected` → `jp_plan_ex`'s resolver, so a custom entry's capacity/speed/fodder/water and its ten-row terrain table re-plan the journey. The **Vessel** picker lists every library vessel but disables the ones with no engine counterpart (`jp_ship_stats` is still a fixed built-in table — `TRAVEL_LIBRARY_SPEC.md` §6), with the reason on the item itself rather than omitted | §4.5.4 | (B) small — remaining: a vessel/vehicle resolver equivalent to the animal one |
| IN-07 | Trade ▸ route assignment | 370-373 | nothing ties a trade relationship to the road or sea lane that would carry it | yes | §3 lists Trade | (B) large |

### 6.13 CARTO workspace — `cartography_workspace.gd`

> **Domain merge (2026-08-20, owner instruction: "And render into carto."):**
> RENDER is no longer a rail domain — §6.14 below is now reached through
> CARTO, via `cartography_workspace.gd` composing a `RenderWorkspace`
> instance into its own dock. This also directly resolves the CA-01/RN-01
> row below: CARTO and RENDER are no longer two domains disagreeing about who
> owns `set_appearance()` — they are the same dock now.

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

### 6.14 RENDER workspace — `render_workspace.gd` (now composed into CARTO, §6.13)

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| RN-01 | The whole domain — Terrain appearance groups | 14-15 | `render.rs`'s `TerrainAppearance` is real but unbound; until it is, Preferences ▸ Render quality is the only live control | yes | §3 gives RENDER a dock; `design/cartalith-menu-structure.md` §5b designs the full subsystem (Preset · Colour relief · Colour · Material · Relief · Detail · Atmosphere · Preview · Quality) | (B) **wrapper** — ~40 real, tested fields driving the current render, reachable through **no `#[func]` at all**. The single largest cheap surface in the shell. |

### 6.15 Frame, viewport and phone — `dcc_shell.gd`, `viewport_host.gd`, `layers_popover.gd`

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| SH-01 | Rail expansion `›` → 200 px sub-node list | **done 2026-08-19** — a real `Button`, growing the rail to `W_RAIL_EXPANDED` and listing each domain's real dock sub-structure via `_phone_list_row()` (§7.17's own proposal) | none | — | §3 names it | **(C)** — `DCC_CONTROL_INDEX.md`: *"Sub-node lists per domain are not enumerated in the spec; the builder has no source for them"* → §7.17 |
| SH-02 | Phone: tool-sheet drag, gesture-inset handle | 1056-1058, 1099-1101 | *"the mockup pictures exactly one static sheet state; nothing here answers a drag gesture"* | yes | §13 | **(D)** — deliberate: inventing a gesture the design does not show |
| SH-03 | Phone: touch-pan-while-drawing (v2.10 `#sculptNavpad`) | 710-714 | `main.gd` carries no such handling to port forward — grepped | yes | §4.5.6 requires it | (B) small — a genuine gap for whoever wires sculpt touch input |
| SH-04 | Phone: battery / signal glyphs | 863-868 | checked against this Godot build's own `OS` class: no `power`/`battery` method exists | yes | §13's mockup | **(D)** — nothing real to back them cross-platform; only the clock gets real data |
| SH-05 | Layers popover: hotkey badges 1–8 | *done, 2026-08-19* | `layers_popover.gd`'s `_add_hotkey_badge`/`_register_hotkeys`/`_input` | yes | §10: *"grouped rows with hotkey badges"* | **(A)**, closed — badged the first 8 rows in `LAYER_GROUPS`' own real build order (Base/Climate/Tectonics, not the spec's SURFACE/TERRAIN FIELDS/CLIMATE, which has no matching row names — see the entry's own note); real `InputMap` actions, scoped to popover-open |
| SH-06 | Viewport ▸ `→ 1 582 m` (draft-stamp elevation under the cursor) | *baseline done, suffix genuinely blocked, 2026-08-19* | `viewport_host.gd`'s `_coords_text` | yes, corrected | §10 | **(A)** for the baseline km-E/km-N/elevation readout (built, `sample_cell`); **reclassified (B)** for the `→` draft suffix — `sample_cell` reads only `WorldState::field`, never the sculpt `PassBuffer` draft, and `build_sculpt_preview_texture` composites the draft into a colourised texture only, not a per-cell elevation `#[func]`. The register's premise that this call already existed was wrong. |
| SH-07 | Status bar ▸ `autosave` and `atlas` slots | `dcc_shell.gd:657` builds both; nothing writes them | none | — | §10's middle group | (B) small — gated on FI-03 and PR-10 respectively |
| SH-09 | Layers popover: **Wind / Ocean currents are animated in the reference and were static here** | *done, 2026-08-23* | `shell/wind_fx_layer.gd`, attached from `layers_popover.gd::_attach_flow_fx` | yes | the reference's own `#windFxCanvas` particle-streak overlay (`_windFx*`, HTML lines 2113-2209) — not in any mockup | **(A)**, closed — owner-reported (*"the ocean current layer isnt animated as the HTML version is. (same for wind)"*). The static rasters were correct and are untouched; what was missing is that the reference stacks a **second**, independent overlay on those two views: 260/200 particles advected along the flow field at `0.315` cells/tick, drawn as fading streaks, respawned on leaving the map, ageing out, or (ocean only) beaching. Ported constant-for-constant. The one deliberate technique change is the trail — the reference fades a persistent canvas with `destination-out`; a per-particle history redraw reaches the same streak without a never-cleared `SubViewport` doing GPU work behind a closed layer. Nothing runs while the view is off (verified: 0.0000 frame-to-frame diff) |
| SH-08 | Menu accelerators for the disabled items (⌘S ⌘⇧S ⌘W ⌘Z ⌘⇧Z ⌘X ⌘C ⌘V ⌫ ⌘A ⌘D ⌘F ⌘⇧P) | `menus.gd` sets only `Ctrl+N`, `Ctrl+O`, `⇧A`, `⇧J` | none | — | §2's tables give every one | **(D)** — an accelerator on a permanently disabled item is dead weight; they arrive with their items |

### 6.16 Urban morphology — added 2026-08-23, previously undisclosed entirely

**Added by this correction pass** (`PARITY_AUDIT.md` C3): before this, this
2 027-line register had **zero occurrences** of "urban", "city viewer" or
"town layout" — no coverage anywhere for what `README.md`/`STATUS.md`
themselves call the largest single unported subsystem. `URBAN_MORPHOLOGY_SCOPE.md`
is the authoritative status document; this table is this register's
required cross-reference to it, not a restatement.

`cartalith-urban` is real: 4,516 lines across milestones 1-7 of ~17 (RNG
substreams, geometry kernel, planar street graph, A\* over the cost raster,
generation rules + culture profiles, the site model, anchors and primary
routes, organic growth — each with its own `tests/golden.rs`). **Verified
for this pass**: it has zero consumers in the workspace —
`grep -rn 'cartalith-urban' crates/*/Cargo.toml` returns only its own
manifest, `cartalith-godot/Cargo.toml` does not depend on it, and the only
mention anywhere under `godot-project/` is a disclosure comment in
`civilization_workspace.gd:490-491`. Milestones 8-17 (radial streets/plaza/
waterway, water infrastructure, fortification, graph cleanup, blocks/
parcels, districts/buildings, amenities, hinterland/decay/details/metrics,
`generate()`/`hashModel`, and the 28-function civ adapter) are entirely
unbuilt.

| # | Missing surface | Reference control | Class |
|---|---|---|---|
| UM-01 | **Town layouts drawn on the map at deep zoom** | `civUrbanLayoutsChk` | (B) large — no engine output to draw; blocked on milestones 8-17 |
| UM-02 | **City Viewer modal** — its own canvas, zoom/pan, legend, info panel | `cityViewerModal`, `cvCanvas`/`cvCloseBtn`/`cvLegend`/`cvInfoPanel`, `_cvDrawCity`, `_cvZoomAt` | (B) large — same blocker, plus a whole modal with no design in `DCC_SHELL_SPEC.md` |
| UM-03 | **Layout thumbnail in the place-edit popup, and its launcher** | `peCityPreview`, `peCityOpen` | (B) large — doubly blocked: no place-edit popup exists at all (ED-03) and no city layout to preview even if it did |

All three are (B) rather than (C) or (D): the reference precedent is exact
and line-cited (`URBAN_MORPHOLOGY_SCOPE.md`), so this is an engine gap, not
a design gap — the honest opposite of most of this register's (C) entries.

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

> **Decision, 2026-08-20 (owner): recommendation 1 accepted in full, and
> recommendation 3 declined.** The Conversion group is **deleted** — all three
> rows, from `menus.gd::_data()` and from `data_manager_window.gd`'s `ROUTES`
> and `GROUP_ORDER`. The Data manager has four groups (*in · out · sources ·
> checks*), which is also what recommendation 1 of §8's menu-shape section
> asked for. CRS was not kept as a project property either: with one flat km
> grid there is nothing to transform *between*, exactly as the analysis below
> says. The research recorded below stands unchanged as the reasoning; nothing
> here was re-argued after the fact.


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
| **`⧉ Travel library… ⇧L`** | `menus.gd`'s `_data()` | Built, 2026-08-19 — Omission O1 / DM-15, closed |

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
| ~~Conversion ▸~~ | ~~(undefined — §7.4 recommends deleting)~~ — **deleted 2026-08-20**, recommendation accepted |
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
| ~~CV-04~~ | ~~The **metropolis** settlement tier (`_civSelectMetropolises`)~~ | **ported 2026-08-20** — no longer deferred |
| ~~CV-08~~ | ~~**`_civApplyRecovery`** (v0.82 static recovery phase)~~ | **ported 2026-08-20** — no longer deferred |
| ~~PR-02~~ | ~~Multi-GPU: build device selection / dispatch modes / VRAM budgeting at all?~~ | **answered 2026-08-20 — build it** (owner instruction). See PR-01/PR-02/PR-04/PR-05 above and `HARDWARE_ACCELERATION.md`'s 2026-08-20 section |
| ~~DM-07~~ | ~~Coordinate systems / EPSG as a first-class route~~ | **deleted 2026-08-20** — owner accepted §7.4; there is no route to defer |
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
| 3 | **JP-13** — **done 2026-08-19** | Journey Planner's **timeline band** — one band per day, coloured travel / water / weather hold / rest-layover | `timeline_bar` is currently drawn **visible and empty** while JOURNEY is armed — the one place in the shell showing an empty region with no explanation. All the data is in `plan`. | `JOURNEY_PLANNER_SPEC.md` §2 |
| 4 | **JP-14** — **done 2026-08-19** | Blocked-stage **inline resolutions** (turn off closures · re-route land-only · depart earlier) | A blocked journey currently ends in a dead end. All three are `_plan_values` edits plus `_compute()`. | `JOURNEY_PLANNER_SPEC.md` §9 |
| 5 | **RD-11** — **done 2026-08-19** | Right dock's collapsed **primary readout** | §6's own last line; `set_dock_readout()` exists and is wired for the left dock only. One call. | §6 |
| 6 | **PR-13 + PR-14** — **done 2026-08-19** | **Light theme** + follow-system | `DccTheme.LIGHT` is fully defined and §11 gives the complete light token column; only the build-once stylebox pass blocks it. The single largest *visible* change available with no engine work. | §2.5, §11 |
| 7 | **WI-02 + WI-03 + WI-04** — **done 2026-08-19** | Window menu: workspace list, open-windows list, **dock width dragging** | Three omissions against §1/§2.6; all three read state that already exists. | §1, §2.6 |
| 8 | **CA-05** | Icon **on-canvas resize handle** | `icon_resize`/`icon_hit_test` are exposed; the drag math already exists on the Label tool and can be copied. Handle geometry derives from `icon_get()`. | §4.5.5 |
| 9 | **JP-12 + JP-15** — **done 2026-08-19** | Supply-reach **per-leg bar with resupply ticks**; party-form fields showing `auto · <resolved>` | `resupply_reach` and each result's `eff` dict already carry every value. | `JOURNEY_PLANNER_SPEC.md` §5, §8 |
| 10 | **SH-05** — **done 2026-08-19** | Layers popover **hotkey badges 1–8** | The popover already enumerates every view; badges plus `InputMap` entries. | §10 |
| 11 | **SH-06** — **baseline done 2026-08-19, suffix reclassified (B)** | Viewport `4 812 km E · 1 093 km N · 1 462 m` cursor coordinates + elevation | `sample_cell` gives the committed elevation; the `→ 1 582 m` draft-stamp suffix turned out to need a new Rust entry point (`sample_cell` never reads the sculpt draft) — see the §6.15 row's own note. | §10 |
| 12 | **SH-01** — **done 2026-08-19** | Rail expansion showing label + subtitle at 200 px | Reuses `_phone_list_row()` verbatim; see §7.17 for why this reading beats the spec's unenumerated one. | §3 |

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

---

## 13 · The v2.10 menu-structure audit (2026-08-20)

### 13.1 What was audited, and against what

`design/Cartalith Menu Structure v2.dc.html` — one 1920-wide canvas,
`data-screen-label="Menu structure nested"`, freshly re-checked against the
live Claude Design project and current — catalogues, in its own words, *"every
surface in v2.10, carrying its real disclosure depth into the seven domains of
design 1a."* It is the most complete single inventory of the reference app's
control surface this repository holds: **202 menu rows across 9 domain columns
and 41 L2 categories**, plus a 22-node workspace navigator and 6 inspector
contexts — **230 catalogued entries**. Many rows carry an `n` marker standing
for *n* sibling sliders in the same v2.10 section, so the underlying control
count is ~330.

**This is not a request to restructure the shell, and nothing here proposes
one.** The canvas's top bar shows the earlier seven menus (Project · World ·
Generate · Simulate · Map · Assets · View); `DCC_SHELL_SPEC.md` §2 replaced
them with File · Edit · Assets · Data · Preferences · Window · Help plus a
domain rail, and the owner merged that rail to three domains on 2026-08-20
(commit `42547d9`). §8.7 above already settles that disagreement in the spec's
favour, and this audit inherits that ruling. What the canvas is used for here
is only its **exhaustive surface inventory with disclosure depth** — the
question "does every control the original app had exist somewhere in this
shell, live or honestly disabled, or is it simply absent?"

### 13.2 The split

| Class | Rows | Share |
|---|---:|---:|
| **(a)** present and live | **71** | 35 % |
| **(b)** present as an honestly-disabled `_todo`, a disclosed gap route, or an in-product "Not built" note | **97** | 48 % |
| **(c)** **absent entirely, no disclosure anywhere — including this register** | **17** | 8 % |
| **(d)** deliberately superseded by a later decision | **17** | 8 % |
| **Total** | **202** | |

The 22 navigator nodes and 6 inspector contexts are counted separately: the
navigator's four groups map onto the rail's three domains plus the merged
INFRA/RENDER subjects (all present); five of six inspector contexts are built
(§6.8), the sixth is `Layers`, already registered as **RD-10** / omission
**O9**.

**The headline is that the honesty rule held for 83 % of the reference's own
surface without anyone auditing for it.** Nearly half the inventory is a
disabled item, a disclosed Data-manager route or an in-product "not ported"
note that names the specific missing Rust. The 17 in (c) are the real finding,
and they cluster: eleven of them are *whole-network civ operations and
generation passes that `generate()` absorbed*, which is exactly the kind of
gap a one-shot pipeline hides — there is no button missing from a panel, there
is a panel that never needed the button, and no reader could tell that apart
from an oversight.

### 13.3 The (c) list in full — every undisclosed omission

Each row names the reference's own `#id` where it has one, and what was done
about it in this pass. **Nine became disabled controls with a real reason;
seven became in-product prose (a stage `gap` string, a route reason, a "Not
built" note); one pair was wired live.**

> **Update, 2026-08-20.** MS-02 (*Infer tectonics from heightmap*) is no
> longer on this list — it was built, not disclosed, in the heightmap-import
> pass. Its row below records what closed it. The counts in the paragraph
> above are left as the audit found them, since they describe that audit's
> own result rather than the current state.

| # | Missing surface | Reference `#id` | Where it now lives | Why it could not be wired |
|---|---|---|---|---|
| MS-01 | **Center landmasses** | `#centerBtn` | `app.gd` — disabled button in the GENERATE · WORLD tool-options bar, beside Generate world / New seed | `generate_terrain` places plate seeds from the seed alone; no centring pass and no post-generate offset exist |
| MS-02 | **Infer tectonics from heightmap** | `#inferTectBtn` | **done, 2026-08-20** — `Data ▸ Import ▸ Heightmaps (PNG)`, and the welcome screen's own *Import a heightmap* tile | Both halves closed in one pass. The reader is `cartalith-assets::raster::decode_png` + `cartalith_terrain::infer::heightmap_to_field`; the inference is `cartalith_terrain::infer` (`buildReliefField`/`pickPlateSeeds`/`classifyPlateCrust`/`reconstructBoundaryStress`/`stampVolcanicArcs`/`inferPlateVelocities`, reference HTML 6641-6752) orchestrated by `cartalith_engine::import::infer_tectonics`. Golden-parity tested bit-exact against the reference (`golden_parity_infer.rs`, 8 tests) |
| MS-03 | **Fold intensity · trench depth · fault blocks** (structured orogeny) | `foldI`/`trenchD`/`faultB` | `world_workspace.gd` — stage 04 Tectonics' `gap` string, which was **empty** | `generate_terrain` hardcodes the reference's own defaults (0.16, 1.0, 0), so behaviour matches; exposing them threads three fields through `OrogenyParams`' call site |
| MS-04 | **Evolve climate ↔ terrain · Evolve cycles** | `#evolveBtn`/`#evoCyc` | `world_workspace.gd` — stage 06 Erosion's `gap`, which named five passes and not these | `evolveCoupled()` has no `cartalith-engine` equivalent. It is not one of the five that got an honest empty group, because it is not a pass over this stage's inputs — it re-runs erosion and climate against each other |
| MS-05 | **Sediment fill** | `#sedimentBtn` | same stage `gap` | same |
| MS-06 | **Auto-populate world** (+ capitals / towns / hamlets counts) | `#civAutoPopulateBtn` | `civilization_workspace.gd` — disabled button in Settlements ▸ Not built | `compute_civilisation` runs inside `generate()`; no `civ_populate` `#[func]`, and `params.rs`'s 58 entries carry no civ parameter |
| MS-07 | **Clear places & routes** | `#civClearPlacesBtn` | same | `CivData` is rebuilt wholesale by `generate()`, never mutated in place — there is no partial teardown to expose |
| MS-08 | **Generate roads** | `#civAutoRoutesBtn` | `infrastructure_workspace.gd` — disabled button in Roads ▸ Not built | same shape as MS-06; the Way/Route tools are the wired alternative |
| MS-09 | **Clear ways & journeys** | `#civClearRoadsBtn` | same | same shape as MS-07, compounded by **IN-02** (committed manual ways have no getter) |
| MS-10 | **Recalculate territories** | — | `civilization_workspace.gd` — disabled button in Politics ▸ Not built | `assign_territory()` runs inside `compute_civilisation`; nothing re-runs it against edited settlements |
| MS-11 | **Clear territory** | — | same | same |
| MS-12 | **Generate provinces** | — | same | provinces are produced inside `generate()` and only read out. The *tint* half of the canvas's row is live (CARTO ▸ Layers ▸ Political — provinces) |
| MS-13 | **Add / remove faction** | — | same | **CV-07** was registered as absent-with-no-disclosure; it now has one. `CIV_FACTION_COUNT` is a compile-time constant and factions have no identity across a re-generate |
| MS-14 | **Show rivers in biome view · Rivers as ways · sharper ecotones** | `#showRivers` | `cartography_workspace.gd` — Layers ▸ Not built | Reference *render* filters over a river network that never crosses the boundary (the same entity gap as **RD-05**/**IN-01**); ecotone sharpening is unparameterised |
| MS-15 | **Refine detail · Burn rivers into tiles · Micro-erode tiles · Chunk debug overlay · Show tile borders** | `#lodRefineBtn`, `#lodDbgSeg` | `menus.gd` — Preferences ▸ Tiled LOD's tooltip, now also renamed to end `· chunk debug` | `lod_synthesize_tile` resamples the existing field and runs no erosion or river burn-in; nothing draws the tile grid. **This also fixed a dangling pointer**: `world_workspace.gd`'s "Not a generation stage" note sent readers to "Preferences ▸ Tiles & LOD" for chunk debug, and that row did not mention it |
| MS-16 | **Sample ▸ Route cost** and **Sample ▸ E–W elevation profile** | — | `right_dock.gd` — two permanently-dashed rows in the Sample panel | §6's no-selection list has both. Route cost is per-**leg** inside `jp_plan`, meaningless at one cell. The profile's data all exists (`sample_cell` reads any cell) but there is no row-slice `#[func]`, so drawing it means 1 000–4 000 boundary crossings per mouse-move — a binding gap, not a data gap |
| MS-17 | **Settlements ▸ per-class filter** and **Ways ▸ by-type filter** | `#explSettlementFilterList`, `#explShowRoads` | **wired live** — see 13.4 | — |

### 13.4 The one thing wired live, and why

**MS-17 was the only (c) row with its engine backing already present**, so it
was built rather than disclosed. `get_settlements()` emits a `kind` on every
row and `get_roads()` a `way_type`; the filter is therefore a draw-time test,
not a missing capability. Three files, ~60 lines:

- `map_overlay.gd` — two *hidden*-set dictionaries (`_hidden_settlement_kinds`,
  `_hidden_way_types`) and one `continue` in each draw loop. Hidden sets, not
  shown sets, so an empty dictionary means "show everything" — which is what an
  untouched shell and a freshly-loaded world both are; a shown-set would need
  seeding from a roster that does not exist until the first generate. The
  settlement test sits **before** any geometry, so a hidden tier never reserves
  label occupancy a visible place would then be pushed out of.
- `viewport_host.gd` — `set_settlement_kind_visible` / `set_way_type_visible`,
  kept as separate entry points from `set_layer_visible` because they take a
  *sub*-key: folding them into one string namespace would collide
  `"settlements"` with `"settlements/hamlet"`.
- `cartography_workspace.gd` — two L4 groups under Layers, five settlement
  tiers and three land way types (`sea_lane` keeps its existing top-level row
  rather than gaining a second, disagreeing switch).

A hidden class stays hoverable and clickable. Hiding a tier is a cartographic
choice and is not a reason to make a place unselectable — the same
independence `_show_settlements` already keeps for the whole layer.

### 13.5 (d) — superseded, with the decision that superseded it

No design is proposed for any of these, per §9's rule.

| Canvas surface | Superseded by |
|---|---|
| The seven menus **Project / World / Generate / Simulate / Map / Assets / View** | `DCC_SHELL_SPEC.md` §2 + its header: *"World generation, simulation, rendering and map styling are workspaces reached through the domain rail (§3), never menu items."* §8.7 above |
| The **six-item mode bar** (WORLD · EDIT · ANALYSIS · SIMULATION · CARTOGRAPHIC · DEBUG) and the **four-group navigator** | The domain rail, merged to three on 2026-08-20 (`42547d9`; `dcc_shell.gd`'s `DOMAINS` doc) |
| **Phase chip · Atlas / Generate** (`#phaseChip`) | The rail foot's own context + stage counter (`app.gd::_refresh_rail_foot`). The reference's chip tracked its `generate`/`explore` tab pair, which this shell does not have |
| **Map view ▸ Mode · relief / biome / political** (`#modeSeg`) | The Layers popover's 18 debug views — §8.7's own "arrived, better than specified". Relief is the popover's `off` row (base map); Biome and Political are real rows |
| **Measurement tool**, marked `new` on the canvas | Already **live** as one of §4.5.1's three global tools (`global_tools.gd`) — the canvas under-counts the shell here, not the reverse |
| **Simulation layers** (climate · population · economy · politics · infrastructure · warfare) | **CV-09** — the engine is a one-shot static generator by repeated owner decision (`VISION.md`, `TIMELINE_SCOPE.md` §4) |
| **Imperial-seat tier (metropolis)** | ~~CV-04~~ — **built 2026-08-20**, a checkbox in `File ▸ New world ▸ Generation` |
| **Villages (suitability-weighted)** | Live, but in `File ▸ New world`, not here — `set_villages_enabled` is a creation-time argument |
| **Way type ▸ trail / bridge** | **IN-05** — spec/engine disagreement resolved in the engine's favour |
| **Per-stage run controls** implied by the numbered `01…10` stage columns | **WW-11** — `DCC_SHELL_SPEC.md` header correction #2, Playwright-verified against the reference |
| **Undo history (5 steps)** (`#undoMem`) | **ED-02** + **PR-11**, both disclosed |
| **Project settings…** | §8.5 — still nowhere in-product, and still a naming/ownership question rather than a build |

### 13.6 Naming recommendations — surfaced, not applied

Per this dispatch's own constraint, nothing below was changed. All three are
cases where the canvas's wording reads better than the shipped wording.

1. **"Analysis field" beats "Layers popover".** The canvas calls the debug-view
   picker `VIEW ▸ ANALYSIS FIELD`, which says what the thing *is* — a choice of
   which field to analyse — where "Layers" collides with CARTO ▸ Layers, a
   different control governing vector overlays. The popover's own footer
   already has to explain that collision in prose. Renaming the viewport button
   to `FIELD` or `ANALYSIS` would remove the need for the footnote.
2. **"Finalize world" beats "Finalize · LOD 0–3 · bake & freeze".** The canvas
   splits the reference's three controls cleanly (Bake depth · Bake ALL levels
   & finalize · Un-finalize); the shell compresses all three into one disabled
   button whose label reads as a specification rather than an action. When
   **WW-01** is built, take the canvas's three-row split.
3. **"Frame furniture" is a better section name than nothing.** Scale bar and
   the measurement readout currently live as unnamed viewport chrome
   (`viewport_host.gd`'s `_chrome()`); the canvas groups them under `§ Frame
   furniture`, which is the standard cartographic term and gives the compass /
   scale bar / neatline a home to grow into.

Two further observations worth recording rather than acting on:

- **The canvas's `+ ADVANCED` (L5) rule is stronger than the shell's.** It
  requires *"Advanced holds only dials whose defaults are already correct.
  Nothing required to finish a world may sit at L5."* `world_workspace.gd`'s
  `ADVANCED_KEYS` was chosen by a different rule (the reference buried it, or
  this port surfaces it as a superset). The two agree today by luck; if a
  future parameter is added to `ADVANCED_KEYS` because the reference hid it
  *and* it changes whether a world finishes, they will diverge.
- **The canvas's DEPTH CAP is five levels; the shell reaches six in one place.**
  CARTO ▸ Layers ▸ *Settlements · by class* ▸ toggle is L1→L2→L3→L4→L5, which
  is legal — but adding one more nesting level under either new filter group
  would break the cap, and the canvas's own remedy (*"a sixth means the L2
  category is wrong and should be split"*) would mean splitting Layers.

### 13.7 Verification

- **The canvas was read in full**, every menu column, nested row and annotation
  (521 lines of HTML), together with its companion
  `design/cartalith-menu-structure.md` (203 lines) — the prose version of the
  same inventory, already cited by §8.7.
- **Cross-referenced against**: `menus.gd`, `app.gd`, `right_dock.gd`,
  `layers_popover.gd`, `data_manager_window.gd`, `performance_window.gd`,
  `global_tools.gd`, `new_world_dialog.gd`, `map_overlay.gd`,
  `viewport_host.gd` and all five workspace files.
- **Engine claims opened, not inferred.** The full `#[func]` list was
  re-enumerated from `cartalith-godot/src/` and the absence of
  `civ_populate`/`civ_clear_places`/`civ_auto_routes`/`civ_clear_roads`/
  `civ_recalc_territory`/`civ_generate_provinces`/`civ_add_faction` confirmed
  by name. `params.rs`'s 58 entries were listed key-by-group and checked
  against every canvas parameter row;
  `GENERATION_PARAMETERS.md`'s own "Parameters the reference exposed that this
  port does not" was read in full and is the source for MS-03/MS-04/MS-05.
- **Parse-check**: all 11 edited files, `--check-only --script`, clean.
- **Boot-check**: `--headless --path godot-project --quit` — clean. The main
  scene is `shell/app.tscn`, so this builds every workspace.
- **Scripted headless drive**: instantiated the shell, exercised both new
  filter entry points in both directions, and walked the whole node tree
  fingerprinting tooltips — **all nine new disabled disclosures found and
  reachable** (Center landmasses · Auto-populate world · Clear places & routes
  · Recalculate territories · Clear territory · Generate provinces · Add /
  remove faction · Generate roads · Clear ways & journeys).
- **Not touched**: `asset_library_window.gd` and `asset_bridge.rs`, both
  mid-edit by a concurrent sprite-slicer dispatch.

---

## 14 · Visual sweep (2026-08-20)

Every prior pass in this register verified structurally or headlessly and
said so explicitly — "nothing graphical verified." This pass is the first
that actually looked: a real, non-headless boot of `shell/app.tscn`, a
512×512 world generated, and every major surface driven and screenshotted,
compared frame-by-frame against `design/Cartalith DCC Shell.dc.html` and
`design/Journey Planner DCC.dc.html`.

### 14.1 · Method

**Driver**: a temporary harness scene (`_visual_sweep.gd`/`.tscn`, same
uncommitted-dev-tooling convention `_shot.gd`/`_shot_phone.gd` already use —
see those files' own header comments) that instantiates `shell/app.tscn`,
generates a deterministic world (`seed 483920, 512×512, sea_level 0.42,
villages on`), and walks every surface listed below, saving a PNG after a
multi-frame settle.

**Renderer**: this machine's GLES3/Compatibility path (`gl_compatibility` per
`project.godot`) crashes deterministically on this AMD RX 7800 XT — confirmed
against the project's own pre-existing `_shot.tscn`, not introduced by this
pass — with `ERROR: Condition "!texture_allocs_cache.has(p_id)" is true` and a
segfault, every time `open_project_dialog.gd`'s `popup_centered()` runs for
the first time (the cold-start welcome prompt). `--rendering-driver
opengl3_angle` (Godot's bundled ANGLE→D3D11 path) avoids it entirely with an
identical visual result; a real device/driver combination may not need this
workaround, but it is the reason this pass's screenshots were captured under
ANGLE rather than native GLES3, and it is worth a `TOOLCHAIN.md` note if
another AMD/Compatibility-renderer machine hits the same crash.

**Surfaces swept**: welcome prompt, shell default (empty + generated, dark),
light theme, Generate World dialog, Generate Sculpt mode + stamp stack, CIVIL
dock default + Timeline category + a selected settlement's right dock +
territory overlay, CARTO dock default, Layers popover (open, a view picked,
and the picked view rendered over the map with the popover closed), Asset
library window + sprite-sheet slicer with a real loaded sheet, Data manager
window, Travel library window, Journey Planner takeover with a real committed
route, and the map at three zoom levels including deep-zoom LOD tiles.

### 14.2 · Per-surface verdict

| Surface | Verdict | Notes |
|---|---|---|
| Welcome prompt | **PASS** | Matches "Open project dialog" screen's welcome mode closely: three tiles (Create/Import/Drop a .zip), search well, Recent/All worlds/Shared tabs. |
| Shell default (dark, generated) | **PASS** | 3-domain rail, menu bar, tool options row, SAMPLE right dock all present and laid out per `DCC_SHELL_SPEC.md`. |
| Light theme | **PASS** | Full repaint is consistent — no leftover dark-token styleboxes anywhere swept. |
| Generate World dialog | **PASS** | Matches the "Generate World" mockup's ten-stage pipeline list + Planet sliders. |
| Generate Sculpt mode | **PASS** | Stamp stack (Undo/Redo/Commit/Discard) appears correctly in the right dock the moment Sculpt mode is selected. |
| CIVIL dock default | **DEFECT (fixed 2026-08-23)** | See CV-VS-01 below — a thin horizontal seam across the map, CIVIL-domain-only. It was a deep-zoom LOD tile boundary, visible because the LOD layer was painting the reference's *Relief* ramp over the *Biome* map. |
| CIVIL right dock: stuck Sculpt context | **DEFECT (fixed)** | See §14.3. |
| CIVIL Timeline category | **PASS** | Expands correctly; years/filters/simulate-collapse rows all present with an honest "not wired to the map" disclosure. |
| CIVIL territory overlay | **PASS** (after correcting the sweep itself) | Painting + committing territory does not itself show it — `cartography_workspace.gd`'s "Political — territory" layer defaults off, same as the design's own opt-in layer model. Not a bug; the first sweep pass mistook it for one. |
| CARTO dock default | **PASS** | Layers/Layer properties/Annotation categories match "Cartography style" screen. |
| Layers popover + z-order | **PASS** | The popover itself renders correctly on top of the map; a picked debug view (Elevation) stays visibly on top of the map after the popover closes — the z-order fix `CHANGELOG.md` records earlier is confirmed live, not just headlessly. |
| Asset library window | ~~PASS~~ → **FAIL (corrected 2026-08-20; rebuilt)** | The original verdict — "family rail (8 families), slot grid, inspector, empty-library state all honest and correctly laid out" — checked that the controls *worked* and that the disclosures were honest. It never checked the layout against the canvas, and the layout did not match: a floating dialog with an OS title bar instead of a full-bleed workspace window, stock Godot slabs instead of the canvas's outline chips, no status line, no window-bar title, tile captions outside their tiles, and an inspector that was a stack of label/value pairs. The owner reported it in exactly those terms. See **§14.6** for the full delta list and the rebuild. |
| Asset library slicer, real sheet | **PASS** (arithmetic) / **FAIL (corrected; rebuilt)** (layout) | The grid overlay does land exactly on a synthetic 6×4 sprite sheet's cell boundaries and the detection readout is correct — that half of the verdict stands, and the rebuild did not touch the span arithmetic. The modal's *layout* was a single vertical stack of stock widgets wide enough to clip its own labels, against the canvas's 760 px two-column card. See **§14.6**. |
| Asset library: slicer left open on Close | **DEFECT (fixed)** | See §14.3. |
| Data manager window | ~~PASS (after fix)~~ → **FAIL (corrected 2026-08-20; rebuilt)** | The original verdict — "Conversion group confirmed gone from the routes rail; the subtitle text still advertising it was the one leftover" — is a *content* check, and it was right as far as it went. It is not a layout check, and the layout did not match: a floating 920×600 `AcceptDialog` with an OS title bar and a stock OK button, a `§`-sigil routes rail of autowrapping flat buttons with no badges and no selected-row ground, no window bar, no pane footer, no status line, and a route pane that showed one grey paragraph where the canvas designs seven labelled columns. Exactly the same class of miss as the Asset library row above, found by the same test the owner applied there. See **§14.7** for the 20-item delta list and the rebuild. |
| Travel library window | **PASS** | Animals & mounts tab, 7 stock entries, correct read-only-stock footer. |
| Journey Planner takeover | **PASS** (after correcting the sweep itself) | Full takeover — spine map, profile/stage selector, stage matrix, party form, right-dock journey summary (time/load/supply reach/cost/vessels) — all render correctly once the sweep armed the tool from the CIVIL domain. The first sweep pass armed it from CARTOGRAPHY instead and saw no visible change; that is correct, documented behaviour (`journey_planner_view.gd`'s `_recompute_visibility()`), not a defect — though see JP-VS-01 below for whether it should be. |
| Map, 3 zoom levels + deep-zoom LOD | **PASS** | Settlement pins and labels tier in correctly with zoom; deep-zoom (z8.0) tiles are visibly pixelated, which is the disclosed, known characteristic `tool_overlay.gd`'s own header comment already quotes the owner on ("there is still a certain pixilated quality to the map when we zoom") — not a new finding. |

### 14.3 · Defects found and fixed

**AL-VS-01 — Sprite-sheet slicer modal stranded on top of the whole app.**
`asset_library_window.gd`'s Close button called only `hide()` on the parent
`AcceptDialog`; the slicer (`_slicer`, a separate child `Window`) has its own
independent visibility and was never told to close too. Reproduced with the
exact same code path a real user's mouse click uses (`close_btn.pressed`),
not a driver-script artifact: open the library, open the sprite-sheet slicer,
click the library's own Close button — the slicer remains floating on top of
*everything* opened afterward (confirmed across Data manager, Travel
library, and the Journey Planner takeover in the first sweep pass, before
the fix). Fixed by closing the slicer alongside the library on all three
paths (Close button, Escape, titlebar ✕) — `close_btn.pressed`,
`close_requested`, `canceled` all now call `_close_slicer()` before/on
`hide()`. Re-verified in the second sweep pass: `08c_asset_library_closed_
slicer_gone_check.png` shows the slicer genuinely gone.

**CV-VS-02 — Stamp stack stuck in the right dock outside WORLD domain.**
`right_dock.gd`'s `show_sculpt_stack()` claims (in its own doc comment)
that "Sample stays the default everywhere else" — but nothing ever reset
`_context` back to `CTX_SAMPLE` on a domain switch. Arm Sculpt mode in
WORLD (which calls `show_sculpt_stack()`), then switch to CIVIL or
CARTOGRAPHY: the right dock kept showing the Stamp Stack panel, a World-only
tool's UI, with no sculpt tool armed and no sculpt panel visible anywhere
else on screen. Fixed with a narrowly-scoped `leave_sculpt_context()` on
`RightDock`, called from `app.gd`'s `_on_workspace_changed()` whenever the
new domain isn't `"world"` — it only resets when the context is specifically
`CTX_SCULPT`, leaving a real settlement/route/faction selection alone (those
are meaningful across a domain switch by design, since Inspect's own
selection is wired domain-independently).

**DM-VS-01 — Data manager subtitle still advertised the deleted Conversion
group.** `data_manager_window.gd`'s header subtitle hardcoded "import ·
export · sources · conversion · validation" verbatim from §9 of the design —
but the Conversion group itself was deliberately deleted 2026-08-20 (this
same file's own `GROUP_ORDER` doc comment, `DCC_SHELL_SPEC.md` §2.4's
correction note). The routes rail correctly shows only four groups; the
subtitle line above it was the one place that missed the deletion pass and
kept promising a fifth. Fixed: subtitle now reads "import · export · sources
· validation," matching `GROUP_ORDER`.

### 14.4 · Defects catalogued, not fixed at the time

*(CV-VS-01 has since been fixed — 2026-08-23. JP-VS-01 is still open.)*

**CV-VS-01 — A thin horizontal seam across the map, CIVIL-domain-only.
FIXED 2026-08-23** — it was a deep-zoom LOD **tile boundary**, and the reason it
read as a coloured hairline is that the LOD layer was painting a different
picture from the map underneath it. Two causes, both now removed:

1. `lod_bridge::synthesize_tile_rgba` passed `tile_bounds(...).to_float()`
   straight to `amplify_region`, whose output index maps to
   `rx + ox/(out-1)*(rw-1)` — endpoints inclusive, a *sample* convention. The
   base raster is texels: cell `i` covers screen `[i, i+1)`. So a tile
   stretched `TILE_CELLS` cells' worth of screen over `TILE_CELLS - 1` cells'
   worth of data (1.6%) and sat half a cell out of register, leaving a real
   discontinuity down every tile edge. New `tile_sample_region` solves
   `cx + 0.5 == bx + (ox + 0.5) * bw / out` instead, so adjacent tiles sample
   exactly one texel apart across a shared edge — unit-tested at both ends and
   on an edge-clipped tile.
2. The LOD layer was rendering the reference's *Relief*-mode tile
   (`render_height_tile_rgba`'s hypsometric ramp) over the *Biome*-mode base
   map, so the seam had two differently-coloured sides to be visible between.
   Gold-toned because the hypso ramp's own `0.38` stop is `[201,178,74]`. Tiles
   now take their colour from `map_view`'s own texture through
   `shell/lod_tile.gdshader` and carry only a relief-detail shade ratio.

Measured, not assumed: on the pre-fix build, in CIVIL at the fit view, a
row-discontinuity scan across the map rect read a **median of 2.26** with
spikes of **19.03** at y=599 and **10.30** at y=378 — both exactly on a
`TILE_CELLS` row boundary of that letterbox rect (map top 154, 111 px per tile
row, boundaries at 265/376/487/598/709). CIVIL-only in the original sweep for
the reason §14.4 itself suspected: that domain's taller dock changes the
letterbox rect, which moves a tile row into the conspicuous middle of the
picture. The original investigation's negative findings all stand — it was
neither the sea-route dash styling nor a transient backlog artifact, and the
letterbox-rect correlation was the right lead. Full account in
`cartalith-native/docs/CHANGELOG.md`, "Deep-zoom LOD: the tiles were the
reference's *Relief* view, not its *Biome* view".

The original entry, as catalogued:
Screenshotted consistently in `06_civil_dock_default.png` and
`06b_civil_timeline_category.png`: a dashed, gold-toned hairline running the
full visible width of the map at roughly the vertical midpoint of the
letterboxed map rect. Absent from every WORLD- and CARTOGRAPHY-domain
screenshot of the identical generated world at the identical camera state.
Investigated, not blind-fixed, because the cause does not point at an
obvious one-line change:

- **Not a data/logic bug.** Diagnostic instrumentation (temporary, not
  committed) confirmed `map_overlay.gd`'s `_roads` (48), `_sea_routes` (4),
  `_settlements` (240), `_show_roads`/`_show_sea_routes` (both `true`), and
  `_border_frac` are byte-identical immediately before and immediately after
  the CIVIL domain switch. Nothing in the overlay's own drawable data
  changes.
- **Not the sea-route dash styling**, despite the visual match to
  `SEA_ROUTE_DASH_COLOR` — the four real sea routes' logged point ranges
  (e.g. `y: 282→44`, `y: 330→313`) stay within plausible coastal bands, never
  spanning the map's full width at a near-constant `y` the way the seam does.
- **Not a transient LOD-backlog mid-rebuild artifact** — persisted unchanged
  after an extra 1.5s settle (`06z_diag_after_settle.png` in the diagnostic
  run, not part of the final screenshot set).
- **Correlates with a real letterbox-rect change.** `ViewportHost`'s
  displayed map rect measurably differs between domains at the same zoom —
  `[P: (18.5, 0.0), S: (935, 935)]` in WORLD vs. `[P: (53.5, 0.0), S: (865,
  865)]` in CIVIL (CIVIL's taller left dock content changes the available
  viewport width) — which is exactly the kind of resize `map_overlay.gd`'s
  own `resized.connect(func(): queue_redraw())` reacts to. The seam sits at
  very close to 50% of the *new* rect's height in both crops examined,
  which is suspicious but not, on the evidence gathered, conclusively tied
  to any specific draw call inspected (`_interior_rect`'s clip/inset math
  was checked and rejected — it insets from all four edges symmetrically,
  not a midline).

Best next step for whoever picks this up: reproduce interactively with the
Godot editor's remote scene tree inspector open, or bisect by temporarily
disabling `map_overlay.gd`'s draw blocks (roads/sea routes/settlements/
labels) one at a time while resizing into CIVIL, since the resize-correlated
letterbox change is the strongest lead this pass found.

**JP-VS-01 — Arming Journey from outside CIVIL gives no visible feedback.**
`journey_planner_view.gd`'s `_recompute_visibility()` deliberately gates the
takeover on `app.active_domain() == "civilization"` (documented, and reads
as intentional given the `JP-13` reference in `_hide()`'s own comment) — but
`Data ▸ Journey planner… ⇧J` and `open_journey_planner()` (`app.gd`) arm the
tool from *any* domain, with no domain switch and no visual cue beyond a
status-bar line ("Journey armed — Esc to release") that a user in
CARTOGRAPHY or WORLD would have no reason to read as "and now go to CIVIL
to see it." Confirmed directly: the first sweep pass armed Journey from
CARTOGRAPHY and captured a screenshot with the CARTO dock fully intact and
only that one status-bar string different — indistinguishable from a broken
takeover at a glance. Not fixed here because it is unclear whether this is
an oversight or a considered `JP-13` decision this pass doesn't have full
context on; the candidate fix, if it is an oversight, is one line —
`open_journey_planner()` calling `select_domain("civilization")` before
`journey_planner_view.open()`.

### 14.5 · Verification

- **Non-headless boot**, real GPU-composited frames via
  `get_viewport().get_texture().get_image()`, not a `SubViewport` fallback.
- **Parse-check**: the four edited files load and run correctly inside a full
  app boot (proven by the sweep itself completing end-to-end after each
  edit, including exercising the exact fixed code paths — `_close_slicer()`
  via the real Close-button call, `leave_sculpt_context()` via a live domain
  switch away from a Sculpt-armed WORLD).
- **Boot-check**: `--headless --path godot-project --quit` — clean, no errors.
- **Screenshots**: `cartalith-native/godot-project/_visual_sweep.gd`/`.tscn`
  (temporary harness, uncommitted, same convention as `_shot.gd`) produced 23
  PNGs; not committed to the repo (screenshots are not source).

### 14.6 · The Asset library window was passed too leniently — corrected and rebuilt (2026-08-20)

The owner, after this sweep shipped: *"The asset manager menu looks nothing
like the DCC work from Claude design."* He is right, and §14.2's row above
said **PASS**. That verdict is corrected in the table; this section is why it
was wrong and what replaced it.

**How a passing check missed it.** The sweep asked, of this surface, whether
the controls were present, whether they were wired to real engine calls, and
whether the disabled ones carried honest reasons. All three were true, and
none of them is the test the owner applied. The test he applied — *does it
look like the canvas* — was never run, because `asset_library_window.gd` was
written from `DCC_SHELL_SPEC.md` §8's **prose** before its bindings existed
(the 20+ real `#[func]`s came later in `8506f13`, the slicer in `e96a7ae`),
and nothing in that history ever laid the shape against
`design/Cartalith DCC Shell.dc.html`'s `Asset library window 1920` screen.
The lesson generalises: **a functional check and a visual check are different
passes, and a sweep that only runs the first must say so rather than record a
PASS.**

**The 19 deltas**, read off the canvas element by element against the sweep's
own `08_asset_library_window.png`:

| # | Canvas | What shipped |
|---|---|---|
| 1 | Full-bleed workspace window, 34 px window bar of its own | Floating 1180×760 `AcceptDialog` with an OS title bar |
| 2 | One control vocabulary: `padding:4px 9px; border:1px solid` outline chips, Plex Mono 11 px | Stock Godot filled slabs, stock `OptionButton`, filled search well |
| 3 | `⧉ ASSET LIBRARY` · `map hidden while open` · divider; `☑ Select · 3`; `Sort: slot order ⌄` | No title, no subtitle, no divider; `Select (0)`; `Slot order ⌄` |
| 4 | Pack metadata is a block in the **inspector** | A NAME/AUTHOR/LICENSE row bolted under the window bar |
| 5 | 26 px status line (`● library edited — apply to map to use it`, counts, keys) | Absent entirely |
| 6 | Rail opens with a 28 px `FAMILIES · N` band | Rail opened with ~90 px of grey disclosure prose |
| 7 | Plain tracked group headers; rows are 26 px code · name · `filled/capacity`, accent when incomplete | `§`-sigil `DccWidgets.section()` headers; rows one concatenated string at one colour |
| 8 | Selected row: `accent_wash` ground, accent code, brightened name | No selected-row treatment at all |
| 9 | `Import image…` / `Import pack…` side by side, equal flex | Stacked vertically, full width |
| 10 | `P · PLACES · 10 OF 12 FILLED`; `3 SELECTED` in accent | `… 7 OF 7 SHOWN · 0 FILLED`; lowercase `0 selected` in `text_dim` |
| 11 | Batch verbs folded into the grid header band as quiet text | A row of five filled slabs on their own line |
| 12 | Tile = one bordered box: 76 px art band, hairline, `code · name` caption **inside** it | Caption floated outside the tile — a scatter of squares with text under them |
| 13 | `×N` badge, `☑` selection mark, the word `empty` on the art band | None; the variant count was appended to the caption string |
| 14 | Visible checkerboard on empties | `SlotCell` used `sunken`/`panel_alt` — two tokens one level apart, so it was invisible |
| 15 | Selected tile: 1 px accent border + 35 %-accent outline, offset 1 | A 2 px accent rect |
| 16 | `grid-template-columns:repeat(6,1fr)` | Fixed-width cells, unfilled right margin, a horizontal scrollbar |
| 17 | `P01 · CAPITAL` band, 150 px preview on a 12 px checkerboard, `name · W × H · …` line, Scale slider, Fit/Reset/Replace…/+Variant, 20 px swatches with a selected marker, anchor segment, tag chips with `＋`, VARIANTS strip, PACK METADATA block, equal-flex Validate/Clear | A stack of `_kv_row` label/value pairs; 18 px unmarked swatches |
| 18 | (implementation) | The inspector rebuilt every child on every selection change — hence the `has_focus()` guard the pack fields needed to survive typing |
| 19 | Slicer: 760 px card — title bar with `✕`, preview column, 274 px settings column ending in a summary and Cancel / Slice | A single vertical stack of stock widgets, clipping its own labels (`Trim transparent edg`, `Background → transpa`) |

**Rebuilt** in `shell/asset_library_window.gd`, laid out from the canvas:
borderless and sized under the app menu bar; a chip / segment / well /
text-button vocabulary defined once and used throughout; rail 266, inspector
330, bands 28, tile art 76, variants 56, swatches 20, slicer 760·274·296 —
every number off the canvas, every colour a `DccTheme` token, no hex in the
file. The inspector is built once and refreshed in place. The slicer keeps its
engine-computed grid overlay (dashed at 35 % accent now — a stroke change
only; the span arithmetic is untouched) and the `cd29266` close-with-parent
fix. **Every live binding stayed on the control it was already on.**

**Not regressed — reshaped around.** AS-16 (eight families, not the canvas's
24) keeps the canvas's rail grammar while listing the real eight, with the
disclosure moved from a prose block to the FAMILIES band's tooltip. AS-15
(family-level anchor) draws the canvas's three-way segment, lights the real
one, and disables the other two with that reason. AS-14 (render-time weighted
variants) gets the VARIANTS strip, which selects what the *preview* shows and
says so. The read-only per-item transform gets the canvas's Scale/Fit/Reset
row, disabled with its reason. Replace… and ＋ Variant in that same row are
newly **real**, built from `as_import_item` + `as_remove_item`.

**Verified by looking.** Non-headless `opengl3_angle` boot, a real 512×512
world, `reference_pack.zip` loaded, 12 real items imported (one slot with
three variants), four screenshot/compare iterations against the canvas, pixel
probes confirming rail = exactly 266 px and the selected row's exact
`accent_wash` blend, the slicer smoke path re-run against a real 6×4 sheet
(`24 cells detected · 24 non-empty`, overlay on the boundaries), both close
paths including Escape driven through `Input.parse_input_event`, and
`--headless --path . --quit-after 120` clean. `cartalith-native/docs/
CHANGELOG.md` carries the five Godot layout traps this pass surfaced.

**Update, 2026-08-23 (AS-12/AS-17):** the Collections group, in-app
drag-and-drop (tiles onto a Collections row), and the slicer's pan/zoom/
click-to-select/Margin-handle are now real — see AS-12 and AS-17's own rows
above for the full account, and the grid footer hint now says what drag-and-
drop actually does rather than that it's unwired. **Still open:** dragging a
file from *outside* Godot onto a slot to fill it (Godot's own drag-and-drop
is two unrelated systems — OS file drops reach `Window.files_dropped`, never
a Control's `_can_drop_data`/`_drop_data`, so a slot cannot structurally be
that kind of drop target); per-interior-line grid dragging (the engine's
grid is uniform, so only the Margin boundary has a real parameter to drag);
"Unassigned imports" (still a slot-less bucket the model does not have); and
picking a cell in the slicer still does not narrow what Slice cuts (no
cell-selection parameter on the engine side).

### 14.7 · The Data manager window was passed too leniently — corrected and rebuilt (2026-08-20)

§14.6 closed with a lesson: *"a functional check and a visual check are
different passes, and a sweep that only runs the first must say so rather than
record a PASS."* The Data manager was the other window in the same sweep with
the same history — written from `DCC_SHELL_SPEC.md` §9's **prose** before its
export bindings had a caller — and it was passed the same way, on content
rather than layout. §14.2's row is corrected in the table above; this section
is the delta list and the rebuild.

**The 20 deltas**, read off `design/Cartalith DCC Shell.dc.html`'s
`Data manager window 1920` screen element by element, against the sweep's own
`09_data_manager_window.png`:

| # | Canvas | What shipped |
|---|---|---|
| 1 | Full-bleed workspace window under the app menu bar, with its own 34 px window bar and a 26 px status line | Floating 920×600 `AcceptDialog`, OS title bar, stock **OK** button as the only footer |
| 2 | Window bar: `⧉ DATA MANAGER` accent · subtitle · spacer · `Close ✕` outline chip | Title in the OS title bar; the subtitle a lone mono label *inside* the body; no Close chip |
| 3 | Rail is 252 px and opens with a 28 px `ROUTES` band | 260 px, no band |
| 4 | Group headers are plain tracked `IMPORT` / `EXPORT` / `SOURCES` / `VALIDATION` (`padding:9px 14px 4px`) | `DccWidgets.section()`'s `§ IMPORT` sigil headers — the *dock* L3 grammar, on a window rail |
| 5 | Route rows `padding:5px 14px 5px 24px`, 11.5 px, one line | Flat 26 px `Button`s at no indent, `AUTOWRAP_WORD_SMART`, wrapping onto two lines |
| 6 | Rows carry a quiet right-hand badge (`tiles`, `→ Assets`, `.zip`) | The qualifier concatenated into the label itself (`Assets (routes to the Assets menu)`) |
| 7 | Selected row: `rgba(224,163,74,.09)` ground, brightened name, accent `▸` | Font colour change only — no ground, no caret |
| 8 | Import ▸ **Maps** and Import ▸ **GIS / GeoJSON** are two rows | Merged into one `Maps (tiles) · GIS / GeoJSON` row |
| 9 | Rail footer: `exports → …` / `last run 14:02 · 62 MB`, two mono lines under a hairline | A `§ EXPORTS ROOT` sigil header plus a wrapped path plus a third line |
| 10 | Pane header band, 28 px: breadcrumb left, `web-map ready · XYZ scheme` right | A 12 px-padded breadcrumb, no band, no ground, no right-hand descriptor |
| 11 | Pane body is `grid-template-columns:1fr 1fr; gap:0 34px` | A single-column stack |
| 12 | Seven labelled columns — TILES / PROJECTION / LAYERS INCLUDED / OUTPUT / ESTIMATE / MARKDOWN VAULT / RECENT RUNS | One grey paragraph and, at most, one action button — **DM-13** |
| 13 | Row grammar: `120px label · control`, `padding:4px 0` | No row grammar at all in the pane |
| 14 | Controls are segments (`3px 9px`, one lit), wells (`4px 9px`, mono) and `☑`/`☐` rows with a right-hand note | None of these existed in this file |
| 15 | ESTIMATE is a bordered block with four `space-between` rows | Absent |
| 16 | MARKDOWN VAULT is an accent-bordered block: `●` status, prose, three checks, three equal-flex buttons | Absent — **DM-14** |
| 17 | RECENT RUNS: three `space-between` mono rows | Absent |
| 18 | Pane footer under a hairline: `writes to …` · `Save as preset` · `Dry run` · accent `Export 3 619 tiles` | Absent |
| 19 | Status line: `idle · no pass running` · vault state · `Esc close window` | Absent |
| 20 | *(divergence, not a delta to fix)* The canvas still carries a **CONVERSION** group in its rail and subtitle | Correctly absent — deleted by owner decision `17ccc18`; the canvas predates it |

**Rebuilt** in `shell/data_manager_window.gd` from that screen: borderless and
sized under the app menu bar, rail 252, bands 28, status 26, row label column
120, pane padding 18, column gap 34 — every number off the canvas, every colour
a `DccTheme` token, no hex in the file. The canvas's chip / segment / well /
text-button / band vocabulary **moved out of `asset_library_window.gd` into
`dcc_widgets.gd`** in this pass, which is what that file's own note asked for
(*"if a second window needs them, they move"*); its eight private statics stay
as one-line delegators, so none of its 74 call sites moved.

**Export ▸ Maps is now wired — DM-02 partially closed, RD-09 closed.**
`region_export_tiles` was bound and golden-tested and had **no caller** in the
shell; this pane is that caller. It exports the live Region-select marquee as a
zipped `cols × rows` tile grid and the run is real end to end: verified writing
a 5.17 MB archive of **33 entries** (16 × `tiles/refined_{r}_{c}_rg16.bin`,
16 × `.png`, plus `tiles/index.json`) through the same button handler a mouse
click uses. `right_dock.gd`'s Region select ▸ *Send to Data ▸ Export* — disabled
since it was written, with the tooltip *"the Data Manager panel to call it
doesn't exist yet"* — now opens this pane.

**What the canvas draws that the engine cannot do, drawn and disabled with its
reason** rather than omitted or faked: XYZ / TMS / WMTS addressing (the export
writes a flat row/col grid plus an index, not a slippy-map pyramid), every CRS
and the world file, `folder` and `MBTiles` packaging, `leaflet-preview.html` and
`style.json`, skip-all-ocean-tiles, political tint / labels / rivers as export
layers, and Save as preset. The MARKDOWN VAULT block is drawn in the canvas's
shape but **quiet rather than accent-bordered** — the canvas's vault is linked
and this one cannot be (DM-14).

**Two things the canvas invents that this window measures instead.** The
ESTIMATE block's `~ 214 MB` / `~ 3 min 40 s` are a size *model*; this port has
none, so **Dry run** performs the whole export and reports the real byte count
and elapsed time without writing a file, and the block reads `measured by Dry
run` until it has. RECENT RUNS and the rail footer's `last run` are
session-scoped and say so (DM-12 wants a persisted history; nothing persists
one).

**Three Godot traps found, all of which had shipped:**

1. **`AcceptDialog` enables `wrap_controls` in its constructor**, so the window
   grows to its contents' minimum size on every `child_controls_changed()` —
   and only ever grows. This window popped correctly at 997 px and was then
   grown to **2032 px inside a 1031 px viewport**, putting its own pane footer
   and status line permanently past the bottom edge where no scroll could reach
   them. `wrap_controls = false` on both full-bleed windows.
2. **The autowrap-Label trap §14.6 recorded, again.** The rail footer's two
   autowrap labels had no minimum *width*, so they reported an enormous minimum
   *height* — which is what fed trap 1.
3. **`theme/dark_theme.tres` gives `ScrollContainer/styles/panel` the
   `SB_FieldDisabled` box** — an input-well stylebox, with
   `content_margin_left/right = 10`, a border and a **4 px corner radius**, on a
   container that draws no chrome on either canvas screen. Every scrolled region
   in the shell is inset 10 px against its own header band; here it put the
   column headers 10 px right of the breadcrumb above them. Overridden per
   scroll region in this window; **the theme itself is untouched and still has
   this, shell-wide** — a global fix belongs in its own pass with its own visual
   check.

Trap 1 was **a live regression in the shipped Asset library window too** — its
status line was off the bottom edge on this display — and is fixed there in the
same commit, confirmed by screenshot.

**Verified by looking.** Non-headless boot on the native GL driver (no ANGLE
fallback needed; `6a97911`'s launcher fix holds), a real 2048×1311 world, a real
1024×590-cell marquee set through `region_set`, five screenshot/compare
iterations against the canvas, a real export written and re-opened with
`ZIPReader` (33 entries, `tiles/index.json` present), a dry run measured at
5.2 MB / 0.58 s, Escape close driven through `Input.parse_input_event`, the
Asset library re-shot to confirm the `wrap_controls` fix, and
`--headless --path . --quit-after 120` clean.

**Still open on this window:** every Sources and Validation route, GIS/GeoJSON
in both directions, the save writer, and the pyramid half of DM-02 (zoom-level
addressing, CRS, retina variants, ocean-tile skipping). All are disclosed in
place, in the canvas's own shape.

---

## 15 · The phone overflow menu is wired but inoperable (2026-08-20)

Classification: **(C)** — a real, connected affordance with no phone design
behind its presentation. Recorded here as the brief for the mobile menu design
the owner is having produced separately; **deliberately not fixed**, because
building one now would be discarded when that design lands.

Owner report, from the OnePlus 6T: *"not much from the menus work on android."*

### What is real

`DccShell._build_phone_overflow()` does not draw a placeholder. It reparents the
**actual desktop menu bar** into the phone sheet, so all seven genuine program
menus — File, Edit, Assets, Data, Preferences, Window, Help — and every one of
their roughly 41 items and 15 submenus are present and connected, exactly as on
desktop. `DCC_SHELL_SPEC.md` §13's promise that the `⋯` affordance carries "the
full menu bar" is kept structurally. **The routing is worth keeping; only the
presentation is missing.**

### What is broken, with device evidence

| # | Fault | Evidence |
|---|---|---|
| 1 | Nothing in the menu path is phone-scaled. `add_menu()` uses `DccTheme.inset(11, 9, 11, 9)` and `FS_MENU` (12 px), raw desktop values, no `_pscale`/`_ptap`. | The row renders ~12 physical px tall — about 1 mm at 314 dpi, against §13's 44 px floor. |
| 2 | Desktop status chrome is reparented with the menus: the `CARTALITH` wordmark (150 px min) and the five `world/res/cpu/gpu/mem` readouts with 22 px gaps. | Pre-generation those labels are empty, so most of the 220-px sheet is blank and the menu row is squeezed into a bottom strip. |
| 3 | The menus do not respond to touch. | Tapping `File` at its centre produced no popup and no pressed state; holding the touch down (`adb shell input motionevent DOWN`, captured while held) produced neither. Not conclusively separated from a simple miss, given (1) — but the observable result is an inert menu. |
| 4 | 15 `add_submenu_*` calls assume hover-to-open, which touch does not have; a nested `PopupMenu` positioned for a pointer has nowhere sane to go at 1080 px wide. | Even with (1) and (3) fixed, ~41 items behind 15 hover-opened submenus is not a phone menu. |

### The brief this implies

A phone menu that keeps the seven menus and their ~41 destinations but
re-presents them as a **full-screen, touch-sized, drill-down list** — one level
per screen, 44 px minimum rows, back-navigation instead of hover — would inherit
all the existing wiring in `menus.gd` unchanged. The desktop wordmark and readout
cluster belong in the app bar or the status sheet, not in the menu surface.

Note that the same root cause was fixed this pass for the *tool* sheet
(`ANDROID_BUILD_SCOPE.md`, fourth device pass §4): desktop-authored contents
placed into phone chrome without scaling. The fix there was applied at
`set_tool_options()`, and deliberately does **not** reach the overflow sheet,
precisely so it does not pre-empt this design.

## 16 · The top-left global tool overlay has no drawn presentation in the DCC canvas (2026-08-23)

Owner-reported, verified by direct search: the top-left tool set (Measure,
Region select — `global_tools.gd`, `tool_overlay.gd`, per `DCC_SHELL_SPEC.md`
§4.5.1) is real and live in every domain, but does not appear anywhere in
`design/Cartalith DCC Shell.dc.html` as a drawn UI element. Confirmed by
searching the canvas source for "measure"/"paint": the only hits are
incidental prose ("44 px minimum, **measured** inside the safe area",
"**repaint** 180 ms") — no toolset artboard exists. `design/Cartalith
GUI.dc.html` was checked too and has neither.

This is a distinct finding from §13.5(d)'s line on the *Measurement tool* —
that entry addressed an older, superseded canvas (`Cartalith Menu
Structure.dc.html`) marking the tool `new` and concluded the canvas
under-counted an already-shipped feature. This entry is about the *current*
DCC Shell canvas never having drawn the overlay at all, which is a gap in
the canvas, not in the shell.

**Owner instruction: do not touch `global_tools.gd`/`tool_overlay.gd` to
"fix" this.** The owner is designing a more refined presentation for this
toolset directly. Until that design lands, this is classified **(D)** —
a deliberate hold, not an open defect — and no agent should reduce, remove,
or reshape the existing Measure/Region-select functionality to chase visual
parity with a canvas that simply hasn't drawn it yet.
