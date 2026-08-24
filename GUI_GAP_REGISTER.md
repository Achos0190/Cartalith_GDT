# GUI gap register — every disconnected control, its design, and where none exists

> Owner request, 2026-08-19, verbatim: *"verify that all GUI elements are tested,
> connected and where it doesn't connect to other menus or functions designs have
> been made to be implemented. If not, research the menu naming, documentation in
> the design, and where you still have gaps find references in similar
> applications."*

**The premise does not hold, and that is by design.** The shell does not have a
small number of stragglers to finish connecting: it has **215 catalogued
disconnected surfaces** (123 at this document's original writing; recounted
2026-08-24, §3 has the method and the caveats), every one of them added
*deliberately disabled with a stated reason*, per the honesty rule `menus.gd`'s
own header states —

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
| [6](#6--layer-1--2-the-catalogue) | **Layer 1 + 2 — the catalogue** (215 entries as of the 2026-08-24 recount, classified; 123 at original writing) |
| [7](#7--layer-3--comparable-application-research-for-c) | **Layer 3 — comparable-application research** for every (C) |
| [8](#8--menu-naming-audit) | **Menu naming audit** |
| [9](#9--d-entries-owner-decisions-not-gaps) | (D) entries: owner decisions, not gaps |
| [10](#10--the-actionable-a-list-in-priority-order) | The actionable (A) list, in priority order |
| [11](#11--out-of-scope) | Out of scope for this register |
| [12](#12--verification) | Verification |
| [13](#13--the-v210-menu-structure-audit-2026-08-20) | **The v2.10 menu-structure audit** — `design/Cartalith Menu Structure v2.dc.html` against the shipped shell, and the 17 undisclosed omissions it found |
| [14](#14--visual-sweep-2026-08-20) | **Visual sweep (2026-08-20)** — the shell driven live, screenshotted, and compared against the DCC Shell / Journey Planner mockups. **§14.6 corrects one of its own verdicts**: the Asset library window was passed on function rather than layout, and has been rebuilt against the canvas. |
| [15](#15--the-phone-overflow-menu-is-wired-but-inoperable-2026-08-20) | **The phone overflow menu (2026-08-20)** — (C): the real menu bar is wired into the phone sheet but is unscaled, buried in desktop status chrome, and inert to touch. Device evidence, kept as the brief for the mobile menu design; **not fixed**. |
| 16-22 | Seven sections added after the contents table was written; see the `## ` headings directly. |
| [23](#23--rf-01--the-civil-dock-never-rebuilt-after-a-world-generated-2026-08-24--fixed) | **RF-01 (2026-08-24)** — a new class, and not a capability gap: the whole CIVIL dock (ten sections across two files) was built once at launch and never rebuilt when a world generated or loaded, so it showed "generate a world first" over a finished world. **Fixed**, with the presentation-vs-recompute cost check that shows why this one is safe to hang off every generate. |

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

**Recounted 2026-08-24** (`PARITY_AUDIT.md` pass 2, F7 — the audit's own
"~65% off" finding). The "123 catalogued gap entries" figure below was
computed once, near this document's original writing, and never re-derived
as the register grew by seven whole sections (§16-§22) and roughly 80 rows
in the batch pass 2 reviewed. It is superseded by a real recount, method
disclosed so it is reproducible:

**215 distinct gap IDs**, found by pattern-matching every ID this document
uses — `grep`-ing for markdown table rows beginning `| <PREFIX>-<NN>` (the
form nearly every ID in §6, §9, §10, §17-§21 uses: `AS-`, `CA-`, `CV-`,
`CX-`, `DM-`, `DV-`, `ED-` including its lettered sub-rows `ED-03a`-`ED-03d`,
`FI-`, `FR-`, `HE-`, `IN-`, `JP-`, `MEA-`, `MS-`, `PR-`, `RD-`, `RN-`, `SG-`,
`SH-`, `UM-`, `WI-`, `WL-`, `WW-`) plus the six `### PH-0N ·` entries §22
writes as headings rather than table rows — then de-duplicating (several IDs
are legitimately restated across sections: §9 regroups every (D) row, §13
cross-references the v2.10 audit against existing IDs, and §10's priority
table restates (A) rows already catalogued in §6 — none of that is a new
entry). **This does not exactly match the audit's own "~203"** — that was
an estimate, not a grep result: its own §14 point 3 asked for the counts to
be re-derived rather than deriving them itself, and §13 says why ("a
classification pass, not an arithmetic one"); 215
is the reproducible figure as of this correction. `O1`-`O9` (§5's
"omissions") and `S1`-`S5` (§4's "stale disclosed reasons") use a different,
non-hyphenated numbering convention and are **not** counted here — they are
each their own small, separately-headlined table, not part of the 123/215
catalogue this section has always described.

The original framing still holds — a group of identically-blocked sibling
controls (the ten Edit-menu items, the five erosion Run buttons) is one
entry, so the raw count of individually disabled controls is higher than
215 — only the entry total itself changed, from 123 to 215.

**The A/B/C/D classification breakdown below is left as originally computed
and is now known-stale, not re-derived — a genuine gap, not a rounding
error.** Re-deriving it mechanically the way the total count was
re-derived is not possible: `grep`-ing every ID's own row for its
`**(A)**`/`**(B)**`/`**(C)**`/`**(D)**` marker finds one on only 54 of the
215 rows. The other 161 lost their letter when the row was later edited to
record closure — e.g. `AS-01`'s row now reads *"done, 2026-08-20 ... real ...
`as_import_item`/`as_add_custom_slot` are wired"* with no `(A)`/`(B)` marker
anywhere in it, and this is the common case for anything closed rather than
the exception. Recovering each dropped letter means reading the row's own
history (what kind of gap it *was* when catalogued, not whether it is now
closed) one at a time across all 215 — exactly the "classification pass,
not an arithmetic one... a judgment per row" `PARITY_AUDIT.md` pass 2 itself
declined to do for this same reason (§13, its own explanation for leaving
this section's counts alone). The table below is retained as a historical
snapshot against the old 123-entry total; treat its percentages as
describing that earlier, smaller catalogue, not the current 215-entry one.

| Class | Count (of 123, stale) | Share (of 123, stale) |
|---|---:|---:|
| **(A)** designed + engine-ready | **17** | 14 % |
| **(B)** designed, engine-blocked | **71** | 58 % |
| **(C)** undesigned | **23** | 19 % |
| **(D)** deliberate decision | **12** | 10 % |
| **Total (superseded, see above)** | **123** | |

(B) by cost, same caveat — computed against the old 123-entry/71-(B) total,
not re-derived against 215:

| Cost | Count | Notes |
|---|---:|---|
| **wrapper** | 22 | The single largest cheap win in the register. Nearly all of it is three subsystems: `TerrainAppearance` (RENDER + CARTO's LIGHT group), `AssetDB` (the whole Asset library window), and the Journey Planner's cost model. |
| **small** | 21 | |
| **large** | 28 | Dominated by five subsystems: the save writer, global undo + selection, the Data manager's import/conversion/validation routes, the colour-ramp/separable-layer system, and river-as-entity. |

**Stale as of 2026-08-20**: ten of the (B)-wrapper rows counted above
(AS-01 through AS-08, AS-13, DM-05) moved to done in that pass
(`ASSET_LIBRARY_SCOPE.md` §10), and two more (**JP-02**, **IN-06**) closed
with the Travel Library's party-form wiring the same day
(`TRAVEL_LIBRARY_SPEC.md` §6). **Also stale as of 2026-08-23**: §6.16 (Urban
morphology, `PARITY_AUDIT.md` C3) added three more (B)-large entries
(UM-01/02/03) that were not previously catalogued anywhere in this register,
and §5's O4/O5/O7/O8 moved from open to done (`PARITY_AUDIT.md` C5); the
same day the Journey Planner's own closing pass took **JP-01, JP-03, JP-04,
JP-05, JP-07 and JP-09** to closed and **JP-06 / JP-08** to partly closed
(in-session only, blocked on FI-01's save-writer by design), and re-closed
**IN-06**'s stated remainder with the vessel resolver — §6.9's rows carry
each account. **Further stale as of 2026-08-24**: §17-§22 alone added
roughly 45 more IDs (`DV-01`-`DV-11`, `ED-03a`-`ED-03d`, `CV-10`-`CV-13`,
`WW`'s erosion-parameter rows, `SG-01`-`SG-03`, `PH-01`-`PH-06`, `SH-09`
through `SH-12`, and others), none reflected in the 123/17/71/23/12 figures
above. §6.3, §6.4, §6.9, §6.12, §17, §18, §19, §21 and §22's own rows are
the accurate, current source for their respective areas; a full re-derivation
of the class breakdown against the real 215-entry total is
`PARITY_AUDIT.md` pass 2 §14 point 3's own recommendation for a dedicated
pass, not something this correction attempts.

**The shape**, as computed against the old 123-entry catalogue (see above —
not re-derived against 215). Only 19 % of the shell's disclosed gaps were
genuinely undesigned. 58 % had a design and were waiting on the engine — and
**31 % of those (22 of 71) were waiting on a boundary wrapper, not a
capability**. That was the same finding `DCC_CONTROL_INDEX.md` summary §1
reached from the other direction ("two whole regions of this design are a
boundary-wrapper problem, not a capability problem"), measured against the
shipped shell rather than the design at the time. Whether the shape still
holds at 215 entries is exactly the open question the class-breakdown
re-derivation above would answer.

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
| FI-01 | Save project | 115 | ~~no save writer (`cartalith-io` read-only)~~ | **CLOSED (2026-08-23)** | §2.1 | **DONE.** `cartalith_io::write_save` (`crates/cartalith-io/src/save.rs`) + `WorldGen::save_project`. The row's own estimate held — the byte-compatibility bar was the work, not the zip. Writes the seven documented entries in `exportZip()`'s own order, DEFLATE, and carries every generation parameter **twice**: at its reference `state` path so the HTML app can reopen the file, and under `state.cartalith` so this port's own ten reference-less parameters are not silently lost. Built in memory and written once, so a failed save never truncates the file it was replacing. Verified three ways, including re-writing a **real** HTML-app export and checking it against that fixture's independent value capture. Format decisions and the one disclosed limitation (`state.erosion` — unshimmed by `loadZip()`, only 2/16 keys modelled, so writing it partially would be worse than not writing it) are in `SAVEFILE_COMPAT.md` |
| FI-02 | Save as… | 116 | ~~same~~ | **CLOSED (2026-08-23)** | §2.1 | **DONE.** `DccBrowseDialog` grew a third `PickKind`: `SAVE` is `FILES` plus a name field in the foot, which is the entire difference between “which of these?” and “where shall I put this?” — so no stock `FileDialog` survives on this path either. Clicking an existing save fills its name in (the overwrite gesture), and overwriting asks first |
| FI-03 | Autosave | 117 | ~~requires a save writer~~ | **CLOSED (2026-08-23)** | §2.1 | **DONE**, as a check item plus a `Timer`, interval in `DccSettings` — machine state, not world state, the same reasoning the GPU block there already carries. **Off by default**: a background writer that starts unasked is the wrong first impression for a tool that writes hundreds of megabytes per save. That is this row's “owner policy” half, decided this way and disclosed rather than left open. Writes **beside** the project (`world.zip` → `world.autosave.zip`), never over it, and deliberately does not clear the unsaved flag — an autosave that made File ▸ Save look unnecessary would be worse than none. Reports through the status bar's `autosave` slot, empty since the day it was built, which also makes `phone_menu.gd`'s own `autosave` readout row real for the first time |
| FI-04 | Revert to last save | 118 | ~~requires a save writer~~ | **CLOSED (2026-08-23)** | §2.1 | **DONE**, and exactly what the row predicted: `load_save` on `current_project_path`, with a confirm in front of it, since the discard is irreversible and the item sits two rows under Save |
| FI-05 | Close project | 120 | ~~no project lifecycle~~ | **CLOSED (2026-08-23)** | §2.1 | **DONE.** `EngineBridge.close_world()` replaces the `WorldGen` handle — the engine has no `unload`, and this is also the only way to release the field memory — and re-reads the two caches that were taken off the old instance. The prompt in front of it is the part that could not exist before: with no writer there was no **Save** to offer, only “discard or cancel”, which is not a choice. It prompts whenever a world exists rather than only when `world_dirty` is set — that flag rides `generation_finished`/`world_loaded` and cannot see a Milestone-F tool commit, and a close is the wrong moment to under-report |
| FI-06 | *(missing)* project name field | — | **none — omission O6** | — | §2.1 | (B) small — no name field on `WorldGen` or `cartalith_io::SaveData` |

### 6.2 Edit menu — `menus.gd`, all ten disabled

| # | UI label | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| ED-01 | Undo / Redo | 172-173 | no undo stack; generation one-shot, sculpt has no Godot binding | **Undo: CLOSED 2026-08-23.** Global heightmap undo is live — `Edit ▸ Undo` (Ctrl+Z), `undo.rs` + five `#[func]`s, pushed by `sculpt_commit` and `carve_fjords`, exactly the reference's own `pushUndo` call sites minus the eight erosion passes this port does not run. The row shows the operation name and depth. **Redo stays disabled and always will**: the reference has no global redo either — `undoLast()` *pops* the snapshot rather than moving a cursor, so an undone step is gone. The Sculpt draft's own Redo (right dock) is a different, real thing. | §2.2 | ~~(B) large~~ → **done**. The scope this row assumed (a general command/diff framework) was not what the reference does: 3 functions, a `Float32Array.slice()` and a 5-deep stack. See §7.1's revised entry |
| ED-02 | Undo history… | 174 | same | **still open, and now for a sharper reason** — the *stack* is real (`undo_stats()` reports depth, bytes, budget and the next label); what does not exist is a panel over it. Tooltip updated 2026-08-23 to say exactly that. The live depth/cost readout landed in `Preferences ▸ Memory ▸ Undo history` instead (PR-11), which is where the reference's own `#undoMem` sat | §2.2 names it in one line; **no panel design exists** | **(C)** → §7.1 |
| ED-03 | Cut / Copy / Paste / Delete | 176-179 | nothing selectable beyond settlements, which are read-only | **CLOSED (Delete and edit) 2026-08-23** — see §18: the place-edit popup (`place_editor_window.gd`), the right-click context menu (`map_overlay.gd`'s `map_right_clicked` → `civilization_workspace.gd`) and the `KEY_DELETE` handler (`app.gd`) all exist now; §18.3 lists the four residual sub-gaps (ED-03a..d). Cut/Copy/Paste specifically remain open — no clipboard model exists for any entity. The correction below is what this row said before that: **corrected 2026-08-23** (`PARITY_AUDIT.md` C3/§3.2/§5 item 3) — this was mischaracterized as a clipboard/selection gap. The real finding: `civ_drop_settlement` **creates** a settlement and nothing **edits, moves or deletes** one — there is no place-edit popup (the reference's `placeEditPopup`/`_civPopulatePlaceEditor` has no port, name/kind/faction/pop/specialisation/traits/history/walls-override/delete all absent), no right-click context-menu handler on the map (`_civCtxShow`'s six operations have no counterpart — `PopupMenu` appears only in `menus.gd`/`dcc_shell.gd`, never on `MOUSE_BUTTON_RIGHT` over the viewport), and no `KEY_DELETE` handler anywhere under `godot-project/` (grep confirms). Labels, icons and sculpt stamps genuinely are selectable and deletable through their own panels, which is why the *original* framing looked plausible — but a user who drops a settlement by mistake, or wants to rename/relocate/remove one, has no path to do so at all, not merely a missing uniform selection model. | §2.2 | (B) large — a place-edit popup, a map context menu and a Delete-key handler are three separate missing pieces, not one selection abstraction |
| ED-04 | Select all / Deselect | 181-182 | same | same | §2.2 | (B) large — same model |
| ED-05 | Find on map… | 184 | no search index; settlement search lives in the Data manager | yes | §2.2 gives one line; **no search UI design** | **(C)** → §7.2 |

> ED-03/ED-04's reasons are stale in *emphasis* — they describe a shell
> that had no tools. They are not corrected here because rewriting them
> correctly means describing the selection split, which is a paragraph, not a
> tooltip. Recorded rather than half-fixed. (ED-01's own share of this note is
> obsolete as of 2026-08-23: its tooltip is gone, because the item is live.)
>
> **Edit is no longer 100 % disabled.** §8.4's naming-audit recommendation —
> move something into Edit so the menu has one live item — was overtaken by
> ED-01 landing: `Undo` is that item, and it is the one every comparable
> application puts there first anyway.

### 6.3 Assets menu + Asset library window

| # | UI label | Where | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| AS-01 | Import image… | `menus.gd:201`, `asset_library_window.gd:402` | **done, 2026-08-20** | real | §2.3, §8 | `as_import_item`/`as_add_custom_slot` (`asset_bridge.rs`) are wired; targets whichever slot is focused in the grid |
| AS-02 | Apply library to map | `menus.gd:241`, `asset_library_window.gd:325` | **done, 2026-08-20** | real | §2.3, §8 | `as_apply_to_map` — the reference's own `applyToMap()`: bake the session in memory (`export_pack_bytes`), load it straight into the renderer, no round trip through a file |
| AS-03 | Clear library… | `menus.gd:243`, `asset_library_window.gd:513` | **done, 2026-08-20** | real | §2.3, §8 | `as_clear_library` -> `AssetDB::clear` |
| AS-04 | Export pack .zip | `asset_library_window.gd:327` | **done, 2026-08-20** | real | §8 | `as_export_pack_bytes` — bakes every item, builds a schema-2 manifest, `archive::write_pack`; disk round-trip verified headlessly (`ASSET_LIBRARY_SCOPE.md` §10) |
| AS-05 | Validate | `asset_library_window.gd:511` | **done, 2026-08-20** | real | §8 | `as_validate` -> `library::run`, shown in a modal |
| AS-06 | Tag… / Collect… / Rename… / Duplicate / Delete (batch) | `asset_library_window.gd:436` | **done, 2026-08-20** | real | §8, §2.3.1 | `as_batch_tag`/`_collect`/`_rename`/`_duplicate`/`_delete`, each read off the reference's own `alBatch*` handlers. `rename` stays honestly split: a custom slot is renamed for real, a frozen slot renames its *item variants* (`AssetDB::item_mut`, new this pass) — frozen slot names are the constant `slot_title`, not editable at all (the real spec/engine disagreement is unchanged, just no longer blocked on a missing binding) |
| AS-07 | Slot inspector: File / Scale / Tags / Pack metadata | `asset_library_window.gd:704-707` | **done, 2026-08-23** | real | §8 | `as_slot_summary`/`as_item_summary`/`as_pack_info` — File/Scale/Tags/Pack metadata all show real values. **Editing closed 2026-08-23**: new `#[func] as_set_item_transform`/`as_reset_item_transform` (`lib.rs`) write straight into `LibraryItem::transform` (`db.item_mut`); the Scale slider (now 5..600%, matching the reference's own `#alScale` bounds) and two Pan X/Pan Y SpinBoxes write live via `as_set_item_transform`, and Fit/Reset call `as_reset_item_transform` (identity, plus `fit_to_bottom` for a bottom-anchored family when Fit is pressed) rather than recomputing the reference's `defaultTransform()`/`fitToBottom` arithmetic in GDScript. Pan is two SpinBoxes, not the reference's drag-on-canvas — disclosed in `asset_library_window.gd`'s own header note as the deliberate substitute for a headless-unfriendly drag surface |
| AS-08 | Per-slot fill state + thumbnails (grid is always a checkerboard) | `asset_library_window.gd:579, 690` | **done, 2026-08-20** | real | §8 | `as_family_slots`/`as_thumbnail_png` — every filled slot shows a real `render_item`-baked thumbnail; empty slots still show the honest checkerboard |
| AS-09 | Sprite-sheet **Slice** | `asset_library_window.gd` slicer modal | **done, 2026-08-20** | real | §8's slicer modal | `cartalith-assets::slicer` is a golden-verified port of the reference's `SpriteSheetImporter` (`computeCells`/`cropCell`/`applyChroma`/`isBlank`, HTML lines 27465-27870); `as_load_sheet`/`as_slice_preview`/`as_slice_apply` expose it. The `N cells detected · M non-empty` readout is now the engine's **real** detection pass — the 8×8 GDScript sample it replaced was labelled approximate and is gone — and the grid overlay draws engine-computed cell spans, so it shows the exact rectangles the slice cuts. Non-destructive: the sheet stays loaded for a re-slice |
| AS-10 | Slicer: Trim transparent edges / Skip empty cells | `asset_library_window.gd` slicer modal | **done, 2026-08-20**, with one disclosure | real | §8 | *Skip empty cells* is a straight port (`isBlank`, alpha **> 8**, golden-pinned on both sides of the boundary). *Trim transparent edges* is a **port-side addition, not a port** — the reference slicer has no trim operation at all; its second pixel toggle is `background → transparent` chroma keying, which is now wired here too. Trim reuses the reference's own alpha>8 threshold so it can never disagree with `isBlank` about what content is (`slicer.rs` module docs; `CHANGELOG.md` discloses it per `CLAUDE.md`'s no-silent-deviation rule) |
| AS-11 | Slicer: Assign to family / Fill from | `asset_library_window.gd` slicer modal | **done, 2026-08-20** | real | §8 | All four targets are offered. §8's *Assign to family* + *Fill from first-empty/overwrite* is the framing the **reference expresses as a flat target-slot dropdown** instead, so the family target is composed from the reference's own primitives (one cell per slot, in frozen vocabulary order) rather than ported; the reference's own three targets (focused slot, one new custom icon, separate custom icons per cell) are ported exactly, including `store[uid]=[item]`'s replace-and-stop for a single-image family |
| AS-12 | Family rail: **Collections** and **Unassigned imports** | *absent* | **done, 2026-08-23** | real | §8's rail lists both | A real **Collections** rail section exists (`_build_family_rail`/`_refresh_collections_rail`), listing every `as_collections()` entry with a live member count, selectable into a real collection-scoped grid view (`_select_collection`/`_refresh_grid_collection`, resolving each member uid through `as_slot_summary`). Also real drag-and-drop: one or more selected slot tiles dragged onto a Collections row add themselves to it (`SlotCell._get_drag_data` / `CollectionRow._can_drop_data`/`_drop_data`, calling the same `as_batch_collect` the Collect… prompt uses). New `#[func] as_collections` (`lib.rs`) is the read side `as_batch_collect`/`as_slot_summary` never had. **"Unassigned imports" closed 2026-08-23** — the engine has no slot-less item concept (`AssetDB` requires a uid on every item), so this is a reserved custom-slot `set` (`UNASSIGNED_SET = "Unassigned imports"`) instead: a real pinned rail row (`_build_unassigned_row`, live count folded into `_refresh_rail_counts`/the status line's own `N unassigned`) that browses every custom slot in that set (`_refresh_grid_unassigned`, filtering `as_family_slots("custom")`'s new `set` field). The footer's Import image… button, previously disabled with nothing focused, now lands the file in a fresh slot there via the same `as_add_custom_slot` its own doc comment already named as this bucket's real engine call. Honest limit: no engine primitive *moves* an already-assigned item into this bucket (only into/out of a Collection), so it is reachable from imports only, not from reassigning existing art — disclosed in the window's own header note. |
| AS-13 | **`Assets ▸ Asset pack ▸` submenu** (24 controls) | *absent — omission O2* | **done, 2026-08-20** | real | §2.3.1 in full | `menus.gd::_build_asset_pack_submenu` — Active pack (live name/author/license/schema/filled-item stats), Pack metadata…, Build ▸ (Validate/Apply to map/Import pack/Export pack, all direct engine calls), Edit ▸ and Batch ▸ (both open the real window, since every one of their controls needs slot/selection context only the grid provides — real navigation, not a disabled item). The one still-gap item (Slot transform editing) is disabled with its real reason, matching AS-07's note |
| AS-14 | Variants strip / "active variant" | *absent* | none | — | §8 | **(D)** — engine truth: variant choice at render time is weighted and seeded (`pick_weighted_variant`); a user-picked "active variant" has no counterpart. `DCC_CONTROL_INDEX.md` §3(f). |
| AS-15 | Per-slot Anchor (top/centre/base) | *absent* | none | — | §8 | **(D)** — engine truth: `Anchor` is a **family** property `sprite_draw_rect` depends on, not per-slot. §3(f). |
| AS-16 | 24-family rail vs the shipped 8 | `asset_library_window.gd:8-23, 360` | disclosed in the window's own note and header comment | yes | §8 says 24; mockup shows 11; engine has 8 | **(D)** — owner decision, `DCC_CONTROL_INDEX.md` summary §5 item 9 |
| AS-17 | Slicer: canvas interaction | `asset_library_window.gd` slicer modal | **done, 2026-08-23** | real | §8 | `SheetPreview` has real wheel-zoom (centred on the cursor, reversible), middle-drag pan, click-to-select-a-cell (toggle: clicking the same cell again deselects), and a draggable handle on the grid's own **Margin** boundary. **Per-interior-line dragging and cell-scoped slicing closed 2026-08-23**: `cartalith_assets::SliceGrid` gained `col_lines`/`row_lines` (`with_lines`, `Option<Vec<f64>>`, `cols+1`/`rows+1` fractions), `compute_cells` reads them instead of always computing uniform `i/cols` — golden-verified default behaviour unchanged (`resolve_lines` falls back to `uniform_lines` for `None` or a length mismatch), plus `move_line` (clamps a dragged line strictly between its neighbours) and `CellGrid::col_line_px`/`row_line_px` (the undisplaced line positions a drag handle hit-tests against, distinct from `column_spans()`'s gutter-narrowed cell edges). Two new `#[func]`s expose the interaction primitives directly (`as_slicer_move_line`, `as_uniform_lines`) so GDScript never reimplements the clamp-so-lines-never-cross rule; `SheetPreview` gained real handles on every interior line (a small dot per line, draggable the same way the Margin handle already was), hit-tested in `_find_line`. Cell-scoped slicing: `SliceParams`/`as_slice_apply` gained `only_cell` (a flat cell index) — `apply_slice` filters `slice_sheet`'s output down to that one cell before naming/placing runs, so a selected cell narrows what Slice actually cuts (`slice_btn.text` reads "Slice this cell"); `as_slice_preview`'s "N detected" readout still describes the whole grid, since only the cut narrows, not the detection pass. 30 `asset_bridge` tests plus 22 `slicer` unit tests cover both additions; verified non-headlessly via real (`Input`-level for the higher-level flows, direct `_gui_input()` dispatch for the drag/click itself — this project's established fallback where OS-level global-coordinate routing into a script-driven check is unreliable) synthesised events: a dragged column line visibly moved the cut off the sprite's own colour boundary, and a selected cell both highlighted and changed the Slice button's text and the `only_cell` sent to the engine. |

### 6.4 Data menu + Data manager window — `menus.gd`, `data_manager_window.gd`

All thirteen `"kind": "gap"` routes, plus the window's own foot and route pane.

| # | Route / control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| DM-01 | Import ▸ Heightmaps (PNG) | 52 | **done, 2026-08-20** | real | §2.4 names it | now a `"live"` route: `DccApp.open_heightmap_import()` → `EngineBridge.import_heightmap` → `WorldGen::import_heightmap`, which decodes the PNG (`cartalith-assets::raster::decode_png`), resamples it at the *image's* aspect ratio and runs `cartalith_engine::import::infer_tectonics` under it — MS-02's other half, same pass |
| DM-01b | Import ▸ Maps (tiles) **and** Import ▸ GIS / GeoJSON — two rail rows since 2026-08-20, as the canvas has them (they were one concatenated row) | 53 | no tile-map or GeoJSON **import** path exists; TIFF absent | yes | §2.4 | (B) large — the remainder of DM-01 after the heightmap half landed. **TIFF is now a closed question, not a pending dependency decision**: the reference's own file input is `accept="image/*"` decoded by the browser, which does not read TIFF either, so PNG-only is parity rather than a shortfall |
| DM-02 | Export ▸ Maps (image · tiles) | 51 | **half done, 2026-08-20** — tile export is real; the *pyramid* is not | partly | §9's route pane, **the one fully-designed route in the window** | **The route is live.** §9's full pane shape is built (§14.7) and calls `region_export_tiles` over the live Region-select marquee, writing a zipped `cols × rows` grid — verified end to end: 33 entries, `tiles/index.json` present. What remains of this row is the *slippy-map* half the canvas draws and the engine has no notion of: XYZ/TMS/WMTS addressing, a zoom ladder, retina @2x variants, ocean-tile skipping, `leaflet-preview.html`/`style.json`. All of those are drawn in the pane and disabled with that reason. Still (B), now medium rather than large |
| DM-03 | Export ▸ GIS / GeoJSON | 53 | was: "no route in, no CRS" | **CLOSED (2026-08-24)** | §2.4 | **DONE.** The row's own estimate was exact: one `#[func]` (`geojson_bridge.rs`, `WorldGen::export_geojson`) plus assembling a `GeoJsonWorld` off `CivData` + `WorldState`. Data manager ▸ Export ▸ GIS / GeoJSON is now a `live` route with a real picker and writer. It exports the **whole world**, not the marquee. Three inputs the reference has and this port does not are handled by omission and disclosed in the pane: no `poi` layer (this port's civ layer has no POI kind), `sea` derived from which collection a way came out of rather than read off a shared record, and rivers re-traced from `WorldState`'s receiver tree rather than a `_riverNet` cache. CRS is still not a thing — the document says so in its own `note`, verbatim from the reference. Verified: 305,646 B, 511 features (239 settlement / 43 way / 216 river / 6 territory / 7 province), valid JSON, every coordinate inside the world's 1200 × 900 km box |
| DM-04 | Export ▸ World Data | 55 | ~~no save writer~~ — the writer exists (FI-01, 2026-08-23); what is left is this row's own route | partly | §2.4 | **(B) small, was large.** `WorldGen::save_project` writes the whole world already. What remains is a Data-manager pane that calls it, and one decision nobody has made: whether “Export ▸ World Data” means the same `.zip` File ▸ Save writes, or a subset. Deliberately not improvised as part of FI-01 — the export pane has a designed shape (§9), and inventing a second meaning for the same bytes from inside a File-menu task would have been the wrong place to decide it |
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
| PR-11 | Memory ▸ Undo history | 339 | no undo stack | **CLOSED 2026-08-23** — a live submenu, and the one place the stack's real cost is visible: parent-row tooltip gives depth, bytes held, budget, and what one step costs *at this resolution*; the five budget rows each say how many steps that buys here; a `Clear undo history now` row frees it on demand | §2.5 gives a range and a default | ~~(C)~~ → **done, with one deliberate departure from §2.5**: the control is a **byte budget**, not a step count. One height field is 16 MB at 2048² and 256 MB at 8192², so a flat "5 deep" would commit to 1.25 GB on the largest world this shell offers. The step count (5, the reference's `MAX_UNDO`) remains the ceiling; the budget is what binds on a big world. Measured: 80 MB held at 2048², freed exactly on clear |
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
| WI-05 | **Diagnostics overlay (Shift+D)** — the reference's `#resOverlay` | *absent, and undisclosed anywhere* — `PARITY_AUDIT.md` §5 item 5 | **done 2026-08-23** — new `Window ▸ Diagnostics overlay` check item, `KEY_MASK_SHIFT \| KEY_D` accelerator | none | reference lines 10182-10229 (no port-side design document named it before the audit) | **(A)** — `resource_overlay.gd`, reading `EngineBridge.grid_size()`/`param_get("use_gpu")`/`gpu_stages_used()`/`quality_tier()` plus three real `WorldParams` flags (`tect.dynamic_lithology`, `climate.currents`, `volc.provinces`). **Not a 1:1 port** — the reference's `resOverlay` is misnamed "resource" (it is short for "resolution"; §5 item 5's own description guessed hover-driven resource-potential, which the actual reference code is not — it is a static engine/perf HUD refreshed after render). IndexedDB/Worker availability (browser-only concepts) and the `PERF.gen`/`PERF.render` per-stage millisecond breakdown (no Rust-side collector exists anywhere in `cartalith-godot`) are honestly dropped rather than invented. |

### 6.7 Help menu — `menus.gd`

| # | UI label | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| HE-01 | Documentation | 413 | no in-app documentation; the repository docs are the reference | yes | §2.7 names it | **(C)** → §7.11 |
| HE-02 | Keyboard shortcuts | 414 | no shortcut table | yes | §2.7 — and it duplicates PR-16 | **(C)** → §7.9 |
| HE-03 | Report an issue | 416 | no issue route wired | yes | §2.7 | **(C)** → §7.11; the *destination* is an owner decision |
| HE-04 | **Generation info…** — the reference's ℹ️ `#genInfoBtn`/`#generationInfoText` | *absent, and undisclosed anywhere* — `PARITY_AUDIT.md` §5 item 6 | **done 2026-08-23** — new `Help ▸ Generation info…` item opens `gen_info_dialog.gd`: a read-only, selectable `TextEdit` dump plus a `Copy to clipboard` button (`DisplayServer.clipboard_set`, same pattern `journey_planner_view.gd`'s stage-table export already uses) | none | reference lines 9824-9868 | **(A)** — almost entirely "call the existing function, format it": `WorldGen.get_params()` (`cartalith-godot/src/lib.rs`) already returns every generation parameter as a flat dotted-key dict, exactly the reference's own "`JSON.stringify`, not hand-picked, so a future slider needs no update" reasoning. **Honestly narrower than the reference in one place**: `generationInfoText()` leads with a hand-picked summary (temperature range, altitude range, max grade) read off live JS field arrays this port has no `#[func]` returning min/max for — that summary line is real engine-side work, out of this ticket's "presentation only" scope, and is not invented here. This dialog leads with what *is* free (grid, seed, extent, quality tier, GPU) and lets `get_params()` cover the rest. |

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
| JP-01 | Carriage **Auto** pick | 366 | `jpAutoPickTransport` has no Rust port | **stale — it did** | `JOURNEY_PLANNER_SPEC.md` §5 ("in auto, counts are computed (terrain × biome, km-weighted) and read-only") | **CLOSED (2026-08-23)**. The disclosed reason was wrong: `cartalith_civ::jp_auto_pick_transport` has existed since milestone 6, with eleven tests. What was missing was the *call*. `jp_compute` gained an `auto_carriage` key which runs the picker over the derived route and mutates the plan **before** it is computed — the reference's `_jpRunAuto` (line 19614), one call site per refresh so it can never run twice or fight a promoted transport — and returns its outcome as `auto` (`jp_auto_transport_dict`: the picked species, count, carts/wagons, `promoted`, `fodder_infeasible`, and the mutated plan). `_sync_auto_carriage()` writes the ten carriage keys back into the (disabled) form and rebuilds it when the picker promoted Walking → Baggage Train, which is `_jpSyncAssetInputs` (line 19632) and its own `structural=true` rule |
| JP-02 | Party set-up picker + capture | `_preset_controls`/`_apply_preset`/`_capture_preset` | `JP_PRESETS` is JS-only; no `jp_presets()` binding | **CLOSED (2026-08-20)**. The tool-options bar now carries a live `set-up` dropdown over `tl_list("preset")` (stock and captured alike, custom rows tagged `· custom` and ⚠-marked by §4 validation state) plus a `capture party…` action writing the current form back through `tl_capture_preset_from_plan`. Deliberately **not** the reference's `JP_PRESETS`: this port's set-ups are the Travel Library's own stored rows, which is the strictly larger thing. Applying assigns only the keys `jp_default_plan()` owns — `tl_get("preset", id)` returns exactly `PRESET_FIELD_KEYS`, `PartyPreset::apply_to`'s own inverse — and leaves per-stage overrides untouched per §3.4 | §5 + `TRAVEL_LIBRARY_SPEC.md` §3.4 | — |
| JP-03 | Re-route for `<mode>`… | 1320 | `jpAutoPickTransport`/`_jpRerouteForMode` have no Rust port | half — see JP-01 | §6's "faster-mode advisories… with a **use here** action" | **CLOSED (2026-08-23)**. `_jpRerouteForMode` (reference line 20391) ported as `cartalith_civ::jp_reroute_for_mode` and bound as `jp_reroute(route_index, transport, force_mode)`: it re-paths the committed route's two endpoints under the domain the transport implies (`jp_mode_for_route`), or under v1.100's explicit `force_mode` override, and rewrites the route in place so `route_get`/`jp_compute`'s `route` index still names it. Both of the reference's refusals are verbatim, and an **unreachable** answer is refused outright rather than drawn as the straight-line fallback `route_commit` tolerates — which is the whole reason `DijkstraPath::reachable` exists. Per-stage overrides, layovers and the trim are cleared on success: the geometry under every stage index changed. **Not** in scope and still open: §6's per-stage *faster-mode advisory* with a "use here" action — `jp_best_land_transport_for_stage` is ported and `jp_compute` does not surface it (that is JP-11's own row) |
| JP-04 | **Cost** group | 1519 | **corrected — S3** | yes | §8 designs it in full (food/fodder · wages · tolls/ferry · animal upkeep · total · per km and per day) | **CLOSED (2026-08-23)**, and it was as cheap as this row predicted. `jp_plan_cost` is the adaptor from a finished `JpJourneyPlan` to `jp_journey_cost`'s caller-supplied inputs — the reference's own call site (line 19854), including its `totalDays ?? days` preference (wages and upkeep are paid on calendar days, rest days included) and its `if(plan.blocked) return null` gate. `jp_compute` returns it as `cost`; the results panel renders carriage / wages / crew / animals & vehicles / tolls / transshipment / total / per-tonne-km / break-even. Priced in **day-wages**, never a currency — that is `JP_COST_*`'s own unit and the reason the model separates the historically-grounded Diocletian ratios from a world's invented money |
| JP-05 | Calculation trace ⧉ | 1553-1555 | no trace window; the `formula` string is deliberately not carried across the boundary | yes | §8 says *"opens in its own window (⧉)"* and nothing about its contents | **CLOSED (2026-08-23)**, built exactly as §7.12 proposed and with its recommendation taken: an **inline collapsible group over the selected stage**, not a `⧉` window — a window for one stage is more chrome than the content earns, and §8 lists it as the last of seven groups anyway. §7.12's one wrong assumption is corrected in passing: the factors were *not* all "already in `results[i]`'s `eff` dict or derivable from the stage" — re-deriving `t_mod`/`w_w`/`col_mod`/the converged load term in GDScript would have meant a second copy of every engine table. So the *structured* chain crosses instead (`JpTerm { key, detail, factor }`, `jp_calc_land_ex`/`jp_calc_water_ex`'s own variables, assigned rather than recomputed), while the reference's `formula` **prose** still stays out of the engine. Engine-side invariant, asserted on a real multi-stage journey: `∏ factor == daily_km` |
| JP-06 | Save journey | 1325 | no save-writer for journeys or projects | yes | §2 lists it in the tool options bar | **PARTLY CLOSED (2026-08-23) — in-session only, said so on the button.** "save journey" names the selected route *plus* the whole party form (plan, per-stage overrides, layovers, animal entries, trim) and adds it to the Journeys list, which reloads it in one click. What is **not** closed is the half this row named: persisting one across sessions still needs FI-01's `.zip` save-**writer**, which `ROADMAP.md` keeps under "Options kept open, not scheduled", and building one as a side effect of a planner button would have been a far larger thing than this control. The list is GDScript-owned rather than a `cartalith-civ` registry for the same reason — with no writer behind it there is nothing for the engine to own: a saved journey is exactly the request `jp_compute` already takes **Ceiling update (2026-08-23):** FI-01's `.zip` save-**writer** now exists, so the reason this row gave for staying open is gone. What actually remains is smaller and different: a channel for GDScript-owned project state to reach `params.json`'s `state`. `save_project` builds that object in Rust from the parameter table alone, so nothing the shell owns can get into it yet. Not built as part of FI-01 — a generic “extras” bag added speculatively from a File-menu task is the wrong shape to commit to before a second consumer (MEA-07, the Travel Library) says what it needs. |
| JP-07 | ⇧-drag spine trim | 1323 | `jp_compute` has no request field for trimming | yes | §3: *"⇧ drag trims"* | **CLOSED (2026-08-23)**, and it was the small thing this row predicted. `jp_compute` gained a `trim` key (a `Vector2` of two 0-1 fractions of the route's own arc length) which cuts the polyline through `cartalith_civ::jp_trim_points` **before** anything else reads it — so every stage index, stop key and per-stage override that comes back belongs to the trimmed route, and a trim is indistinguishable from having drawn the shorter route by hand. Endpoints are interpolated on the segment they fall in (continuous, not vertex-snapped); interior vertices are kept. `_ProfileView` owns the gesture: ⇧-press starts it, motion previews it as a veil over the trimmed-away margins, release commits, and a ⇧ *click* (zero-width) clears it. No reference counterpart exists — v2.10 has no spine to drag on — and none is invented: the trimmed polyline goes through the same `jp_plan` every untrimmed route does |
| JP-08 | Journeys list = committed routes | 226, 250 | no named/persisted journey registry exists engine-side | yes | §3's "journeys list" | **PARTLY CLOSED (2026-08-23)** — same work as JP-06, same honest limit. The left dock now lists named journeys above the committed routes, each reloading its whole party form and trim in one click, with a × to forget it. **Session-scoped**: nothing is written to disk, so the list is empty again next launch. That remains FI-01's writer, and this row stays open until it exists **Ceiling update (2026-08-23):** FI-01's `.zip` save-**writer** now exists, so the reason this row gave for staying open is gone. What actually remains is smaller and different: a channel for GDScript-owned project state to reach `params.json`'s `state`. `save_project` builds that object in Rust from the parameter table alone, so nothing the shell owns can get into it yet. Not built as part of FI-01 — a generic “extras” bag added speculatively from a File-menu task is the wrong shape to commit to before a second consumer (MEA-07, the Travel Library) says what it needs. |
| JP-09 | Vessels ▸ sailing window | 1540 | not part of `jp_water_calc`'s return | yes | §8: *"per water leg: vessel, hold used, sailing window"* | **CLOSED (2026-08-23)**. `JpWaterCalc` now carries `sailing_window_h` — `jp_water_window(cat, terrain)`, the hours actually under way per day, which was already a *factor* of the leg's `daily_km` and simply never surfaced. The Vessels group prints it per water leg beside the vessel and the hold, completing §8's three. **One thing is deliberately not done**: the engine's window is a property of the **water type** (a sheltered bay is worked in daylight, open sea is stood through the night at 22 h), while `TRAVEL_LIBRARY_SPEC.md` §3.3's `sailing_window` is a property of the **hull**. Nothing in the engine couples the two, so the two are not conflated — the group says which one it is showing rather than quietly blending them into a model this port made up |
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
| WW-02 | Run Droplet hydraulic / Hillslope diffuse / Velocity / Glacial / Coastal (5) | 368-373 | was: "not ported; a separate manual pass in the reference with no `cartalith-engine` equivalent" | **no longer true for four of the five** | §5.1 stage 06 | **DONE for 4/5, 2026-08-23 — §19.** All five kernels are ported and bit-exact: `droplet_kernel` (Phase 1), and `hillslope_diffuse` / `velocity_erode_kernel` / `glacial_kernel` / `coastal_process` in `cartalith-erosion/src/passes.rs` (26 golden tests, 98 of 115 mutants killed). Four of them now have a **run path**: `cartalith_engine::ErosionPassParams`, run at the end of `generate_terrain`, exposed as 21 `params.rs` rows in the `erosion` group — six toggles and fifteen knobs, **every toggle off by default** under `DECISIONS.md` §7d, asserted bit-identical rather than assumed (23 rows and a seventh toggle since 2026-08-24, when tidal flats joined them — §19.5). Verified non-headlessly: each pass alone visibly moves the map (38 %/91 %/6 %/45 % of pixels), all-off returns to the base at 0.0000 %. **Droplet is the one still open** — kernel only, no parameter, because its `erodeFinish` tail is a second orchestration and it was outside that pass's remit. The reference's own *button* idiom is still available on top and now cheap (§19.2 (a)); it was not built because UI work is on hold |
| WW-03 | Sculpt ▸ **Brush shape** (8 falloff shapes, Import brush…, Operation, Falloff curves, Rotation) | 665-672 | no engine behind it, and **not in the reference either** | yes | `DCC_SHELL_SPEC.md`'s own header **correction #3**: *"New design work, not a port gap"* | **(C)** → §7.13 |
| WW-04 | Sculpt ▸ **Stroke & grid** (Add point / Duplicate / Rotate / Scale / Tilt / Push / Pull / Align) | 665-672 | same | yes | correction #3; `DCC_CONTROL_INDEX.md` §5.2 adds that it rests on a **"control grid" concept that exists nowhere** and cannot be sized until defined | **(C)** → §7.13 |
| WW-05 | Sculpt ▸ **Actions** (Flip X/Y, Rot L/R, Flatten) | 665-672 | same | yes | correction #3 | **(C)** → §7.13 |
| WW-06 | Paint ▸ Hardness / Softness | 860-863 | stored and echoed back but never consumed — painting is a hard disc with no soft falloff | yes | §4.5.2 lists both | (B) small — `paint_bridge.rs`'s own module doc |
| WW-07 | Stage 01 ▸ geoid sea level, tides (moon mass/distance/k₂) | 68 | was: "default-off reference sub-systems with no `cartalith-engine` equivalent" | **half of it is now wrong** | §5.1 stage 01 | **ENGINE HALF CLOSED (2026-08-23)** — `cartalith_climate::geoid` and `::tides` are complete, bit-exact ports (`buildGeoid`/`refreshGeoid`/`geoAt`/`currentGeoidPreview`; `tidalForcing`/`computeTideField`/`buildTideField`/`refreshTides`/`currentTideField`), 13 golden tests between them, both live as debug views (DV-06, DV-07). **What is left is the parameters**: `PlanetParams` carries no geoid amplitude and no moon roster, so both views preview at the reference's own defaults — which is exactly the state the reference itself previews in, since both toggles default off. Wiring the knobs means deciding where a *default-off* sub-system's enable flag lives in this shell's parameter model. **`#tidalFlatsBtn`'s input side is now closed (2026-08-24, MS-05)**: `passes.tidal_flats` builds the tide field from the finished surface and runs `apply_tidal_sedimentation` over it — which is also the first answer to the enable-flag question, for one sub-system: *the pass toggle is the enable*, because a pass that needs the field is exactly the thing that should pay to build it. The geoid half has no equivalent consumer yet and stays open. (B) small ×2, now genuinely small |
| WW-08 | Stage 07 ▸ min stream order, lakes as water | 90 | reference **render** filters, not generation parameters — Cartography's work | yes | §5.1 stage 07 | (B) small — and `DCC_CONTROL_INDEX.md` marks "lakes as water" **uncertain** (classification switch or display switch?) |
| WW-09 | Stage 08 ▸ seasons & Köppen | 94 | was: "not ported" | **wrong as of 2026-08-23** | §5.1 stage 08 | **ENGINE HALF CLOSED (2026-08-23)** — `cartalith_climate::koppen` ports `computeTempInto`/`computeSeasons`/`classifyKoppen`/`buildKoppen`/`koppenColor` with the frozen 30-key order and the Peel et al. palette verbatim; 6 golden tests, classifier bit-exact. Live as debug view DV-04. **What is left is the control**: seasons are a *derived* product, not a generation parameter — the reference builds them lazily when the view is picked and this port does the same, so a "seasons" checkbox would toggle nothing. The honest remaining gap is exposing `axialTiltDeg` and `maxRainMm`, which the classifier reads and the shell does not surface. (B) small |
| WW-10 | Stages 09 / 10 have no dials | 96-102 | not parameterised — biome classification runs off finished fields; no soil/ore/fertility dials exist in `cartalith-engine` | yes | §5.1 | **(D)** — engine truth, not a gap. Surfacing the *rasters* is a retention-vs-memory decision `MEMORY_OPTIMIZATION_SCOPE.md` already paid to avoid. |
| WW-11 | Per-stage `Run stage n` / `Run n → 10` / stale dots / `04 / 10` counter | *absent* | the dock's own "Not a generation stage" note and `app.gd:298-306` explain why | yes | §5.1 and §4 both design it | **(D)** — `DCC_SHELL_SPEC.md` header **correction #2**: verified by Playwright against the real reference; the capability exists **nowhere**, not in this engine and not in the app being ported. Building disabled buttons for it was rejected as clutter. |

| WW-12 | Paint ▸ **the map never showed a painted cell** | *absent* | `render.rs`'s own module doc listed *"the paint-brush biome/terrain override"* on its **Excluded** list, and `paint_bridge::swatch_color`'s doc said no literal RGB table had been ported | **was accurate when written, wrong by 2026-08-24** — both `CART_BIOME_COLS` and `CART_TERRAIN_COLS` had been in the same crate since the debug views, and milestone C had built the producer the exclusion note assumed did not exist | reference `landColorCore` 7897-7901 (Biome/Terrain 0.60 tint) and 7765-7773 (Splat texture override) | **CLOSED (2026-08-24).** The tool was fully functional and completely invisible: a commit wrote real cells and `build_color_texture()` never changed a pixel, while the reference's `_paintAt` ends in `render()` and tints the map on the first dab. `render::land_color` now takes a `PaintOverride`; `RenderCtx::with_paint` carries the three committed grids; `build_color_texture` supplies them. Pinned exactly by `tests/paint_blend.rs` (9 tests, mutation-checked: the 0.60 weight and the Biome-then-Terrain order each fail it), with `golden_parity_render.rs` unchanged and passing because `PaintOverride::default()` *is* the unpainted state. `swatch_color` now returns the same two reference tables, so overlay and map name a class with one colour. Verified non-headlessly in the real shell: draft → flat opaque discs, Commit → the same discs blended at 0.60 with relief showing through, erase → the exact clean pixel back. Both `_on_paint_commit` handlers gained the `map_view.texture` refresh + `set_preview_texture(null)` pair `_on_sculpt_commit` already had — without it the fix is invisible in the app and the opaque overlay covers the blend it was standing in for. |
| WW-13 | Paint ▸ Commit / Discard stay enabled after a commit | 911-914, `tool_bar.gd` 393 | *(none — not previously recorded)* | new, found 2026-08-24 | — | **(A)** small, **open.** Both buttons gate on `paint_painted_counts()["total"]`, which is the composite of committed *and* pending cells, so after a commit they stay live with nothing left to commit or discard — "Discard draft" especially, which then reads as "remove the paint I can see" and does nothing. Wants a `paint_draft_count()` `#[func]` over `PaintEditor`'s three `PassBuffer`s (~15 lines, one `engine_bridge.gd` passthrough); deliberately left out of the WW-12 pass to keep that commit off a fourth file another session was holding. |

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
| CV-01 | **POI tool** | 94-101 (comment) | **Re-checked and upheld 2026-08-23** (§18.2): omitted, not built inert: `civ_tools_bridge.rs` says POI *"is not a ported concept"*; no Rust function drops one | yes | §4.5.3 designs it in full (kind · faction · name · snap to way, plus a POI inspector) | (B) small — one `civ_drop_poi` mirroring `civ_drop_settlement`; `cartalith-assets`' `poi` family already carries the 10-slot vocabulary |
| CV-02 | Culture ▸ Profiles | 518-523 | `cartalith-civ` generates culture profiles internally; no `#[func]` exports them | yes | §3 lists Culture as one of CIVIL's five subjects | (B) wrapper — `civ_default_culture` is already called inside `get_factions()`; a fuller `get_cultures()` is one binding |
| CV-03 | Timeline filters (Exist only / Ghost removed / Highlight new) can't touch map pins | 821-827 | ~~`get_settlements()` carries no `tid` even though `NamedSettlement` has one~~ | — | `TIMELINE_SCOPE.md` m6 | **PARTLY CLOSED 2026-08-23** — `get_settlements()` (`lib.rs`) now carries `tid`. **Exist only** is wired for real: `civilization_workspace.gd`'s `_tl_apply_filters` filters the array handed to `map_overlay.gd`'s `set_civ_data` down to the active year's `civ_year_diff().present` tids, upstream of that file rather than inside it (territory). **Ghost removed / Highlight new** stay disclosed-open: both need per-pin fade/halo drawing (`map_overlay.gd`'s own `_draw()`, still territory this pass), and "removed" specifically needs the OLD snapshot's settlement data (position/name), which no `#[func]` exposes yet (`civ_year_diff()` returns tid sets only). |
| CV-04 | Settlement class list lacks **metropolis** | 233-239 (comment) | ~~five real `SettlementKind` tiers~~ | — | ~~§4.5.3 lists six~~ | **CLOSED 2026-08-20** — `_civSelectMetropolises` (reference 24961-24989) ported on the owner's decision. `SettlementKind::Metropolis` exists with the reference's own rank-5 tables; `kind_from_str` accepts it, `get_settlements()` reports it, `map_overlay.gd` draws it at rank 5 / glyph ★, and the promotion runs inside `compute_civilisation` behind `set_metropolis_enabled` (reference default OFF). Spec and engine now list the same six. |
| CV-05 | Territory ▸ "respect coastlines" | 298-304 (comment) | `civ_territory_paint_at` always pushes an ungated circular dab (`PaintStamp::ungated`); no coastline mask behind it | yes | §4.5.3 | (B) small |
| CV-06 | Settlement ▸ "pick radius" | 236-239 (comment) | `civ_drop_settlement` computes its own pick radius internally and takes no argument | yes | §4.5.3 lists it | **(D)** — engine truth; a slider would be decoration |
| CV-07 | Faction roster add/remove, persistent identity | **CLOSED 2026-08-23**, §18.1 — `civ_roster_bridge::FactionRoster` on `CivData`, `civ_add_faction`/`civ_remove_faction`/`civ_set_faction_field`, and the Faction Roster window behind CIVIL ▸ Politics. The reason below was true when written: `CIV_FACTION_COUNT` now *seeds* a real roster instead of *being* it | none | — | §6's Faction context implies a roster; `design/cartalith-menu-structure.md` §3.11 names "add/remove faction, faction roster `#civOpenFactionsBtn`" | (B) large — new Rust state; `CIV_FACTION_COUNT` is a constant |
| CV-08 | `_civApplyRecovery` / auto-populate's static "Recovery phase" | *absent* | ~~none~~ | — | `design/cartalith-menu-structure.md` §4 names it | **CLOSED 2026-08-20** — ported (reference 24619-24640) on the owner's decision, wired at the reference's own call site (line 25761) behind `set_recovery_phase`, and surfaced as a five-entry **Recovery phase** dropdown in `File ▸ New world ▸ Generation`, filled from the engine's own `_CIV_RECOVERY_NAME` table. Phase Stable is a strict no-op. |
| CV-10 | **The whole dock never rebuilt after a generate** — Settlements, Population, Economy, Politics all kept their empty state over a finished world | `_build()`, `_rebuild_readouts()` | none — nothing disclosed this, because nothing knew | **not a capability gap at all** | the sections were already designed *and* built; only the signal was missing | **FIXED 2026-08-24 — see §23 (RF-01)**, which owns the finding, the ten-section table, the measured 14 ms rebuild cost and the windowed verification |
| CV-09 | The timeline bar's **six simulation-layer toggles** (Climate · Population · Economy · Politics · Infrastructure · Warfare) | `dcc_shell.gd:628-641` builds an empty `timeline_row` | none in-product — `TIMELINE_SCOPE.md` §4 explains why the bar was left untouched | yes | §10 designs the whole region | **(D)** — `DCC_CONTROL_INDEX.md` summary §5 item 5 and `VISION.md`: the engine is a one-shot static generator by explicit, repeated owner decision. **The bar is drawn and empty in CIVIL/INFRA** — see §11. |

### 6.12 INFRA workspace — `infrastructure_workspace.gd` (now composed into CIVIL, §6.11)

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| IN-01 | Rivers ▸ Hydrology | 314-319 | no `get_rivers()`; the only river output crossing the boundary is baked into the rendered raster | yes | §3 lists Rivers as one of INFRA's five subjects | (B) large — same entity gap as RD-05 |
| IN-02 | Committed manual ways never appear on the map or in a list | 20-31 (class doc), 195, 213 | `get_roads()`/`get_sea_routes()` read `civ.ways`/`civ.sea_routes` only, never `infra.ways`; `way_commit`'s own doc said the getter was out of scope | yes when written | §4.5.4's "Way inspector: waypoint list, length, grade profile, surface" | **CLOSED 2026-08-24** — see the note below the table |
| IN-03 | Way / Route ↶ ↷ (per-waypoint undo) | 232-236 (comment) | no per-waypoint undo in the engine; `InfraTools` only discards the whole draft | yes | §4.5.4 lists ↶ ↷ | (B) small |
| IN-04 | Way ▸ routing mode (freehand / snap / least-cost) | 229-231 (comment) | `infra_tools_bridge`'s own doc: *"nothing to build a 'freehand' or distinct 'snap' routing mode out of"*; snap is real but automatic | yes | §4.5.4 | **(D)** — engine truth, recorded in-file |
| IN-05 | Way types: spec says road/track/trail/bridge, engine has road/track/sea_lane/ancient | 42-49 (comment) | `parse_way_type`'s own doc calls the spec list wrong against the tested four-entry enum | yes | §4.5.4 | **(D)** — spec/engine disagreement, resolved in the engine's favour and recorded |
| IN-06 | Route ▸ vessel / party reference in the options row | `journey_planner_view.gd` `_vessel_field`/`_mount_field`/`_build_animal_definitions` | the journey planner exported nothing past the crate boundary when written | **CLOSED where it can be, and the remainder stated in-UI (2026-08-20)**. The party form's Mount picker and its four per-species **animal definition** pickers are now library-backed (`tl_list("animal")`, custom rows tagged `· custom`), and the choice reaches the engine: `jp_compute`'s new `animal_entries` request key → `TravelLibrary::animal_overrides_selected` → `jp_plan_ex`'s resolver, so a custom entry's capacity/speed/fodder/water and its ten-row terrain table re-plan the journey. The **Vessel** picker lists every library vessel but disables the ones with no engine counterpart (`jp_ship_stats` is still a fixed built-in table — `TRAVEL_LIBRARY_SPEC.md` §6), with the reason on the item itself rather than omitted | §4.5.4 | **CLOSED (2026-08-23)** — the remainder this cell named is done. `TravelLibrary::vessel_overrides` (keyed by **name**, because `JpPlan::vessel` is a name and `jp_ship_stats` is a name lookup, so a vessel needs no `animal_species_slot` equivalent) → `travel_library::vessel_resolver_fn` → `JpVesselResolver` → `jp_calc_water_ex`, the exact sibling of the animal chain and with the same fall-back-to-the-built-in-table contract. Four of `ShipStats`' seven fields come straight off the definition; `river`/`sea` come from `modes` and `open_sea` from `water_rating == Open`, which is precisely `jp_vessel_water_block`'s own test. **The one field with no source is `invalid_water`**: §3.3 has no per-water-type blacklist, so a custom vessel is constrained by its mode and rating only, never by a named water type the way "River Barge cannot navigate River with Rapids" is — stated in the picker's own tooltip rather than papered over. The picker now enables every library vessel that validates `ok` and disables only the incomplete ones, because the resolver declines an incomplete definition rather than sailing a hull with a zero hold |
| IN-07 | Trade ▸ route assignment | 370-373 | nothing ties a trade relationship to the road or sea lane that would carry it | yes | §3 lists Trade | (B) large |
| IN-08 | **Roads, Ports, Trade and Logistics never rebuilt after a generate** | `_build()` | none — nothing disclosed this | **not a capability gap** | all four were designed *and* built; only the signal was missing | **FIXED 2026-08-24 — see §23 (RF-01)**. Roads had a partial path already (`_refresh_manual_ways`, on a way commit); the other three had none at all |

> **IN-02 CLOSED (2026-08-24).** The audit's diagnosis was exactly right and
> the "(B) small — one getter" estimate held: the whole engine-side fix is
> `get_roads()` and `get_sea_routes()` appending `InfraTools::ways` to what
> they already return, each entry tagged `manual: true` (plus `km`, which
> both `Way` and `ManualWay` already carried and neither getter emitted).
>
> **No new getter was written, deliberately.** The register's own estimate
> assumed a `way_get`/`way_count` pair mirroring `route_get`, and the
> reference says not to: `_civCommitWay` (line 26077) pushes a hand-drawn way
> straight onto the same flat `civWays` array as the generated network,
> tagged `manual:true`, and the draw pass (line ~15494) branches on `rt.type`
> alone — a hand-drawn `road` and a generated `road` are drawn identically,
> and `manual` exists so the way *survives a network rebuild*
> (`_civAutoRoutes` filters `civWays.filter(w => w.manual)`) and can be
> listed, never so it can be styled apart. A separate getter would have made
> two lists out of what the reference deliberately keeps as one, and every
> consumer (`map_overlay.gd`, `right_dock.gd`'s Route context, the workspace
> lists) would have needed a second code path for no behavioural difference.
> Only the *sea lane* splits, into `get_sea_routes()` — that is the one
> distinction the reference's draw pass does make (`type === 'sea-lane'`
> takes the navy/dashed branch), and this port already splits those two
> styles across these two getters.
>
> Also closed with it: `_commit_way` now repaints the map
> (`CivilizationWorkspace._refresh_civ_data()`, camera-preserving) and
> refills a **Roads ▸ Hand-drawn** list instead of printing "not shown on the
> map yet"; the right dock's Route context gains a **Source** field
> (Hand-drawn / Generated) and reports the engine's own `km` rather than
> re-measuring the `f32` point array. **Committed *routes* were never part of
> this** — `route_count`/`route_get` have existed since the Journey Planner
> milestone, and a route is a journey along existing geometry, not durable
> geometry, so it belongs to the planner's registry and not the way layer
> (`infra_tools_bridge.rs`'s "Way and Route are two separate commit paths, on
> purpose"). The IN-02 row's original wording said "ways/routes"; only the
> ways half was ever a real gap.
>
> Still open, and not silently folded in: there is no `way_set_name` /
> `way_delete` / way-condition `#[func]`, so a committed way cannot be
> renamed, retyped or removed — the reference's way-properties editor has no
> counterpart here. §4.5.4's "grade profile / surface" half of the Way
> inspector is likewise unbacked. Those are separate (B) items, not IN-02.

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
| RN-01 | The whole domain — Terrain appearance groups | 14-15 | `render.rs`'s `TerrainAppearance` is real but unbound; until it is, Preferences ▸ Render quality is the only live control | yes | §3 gives RENDER a dock; `design/cartalith-menu-structure.md` §5b designs the full subsystem (Preset · Colour relief · Colour · Material · Relief · Detail · Atmosphere · Preview · Quality) | (B) **wrapper** — ~40 real, tested fields driving the current render, reachable through **no `#[func]` at all**. The single largest cheap surface in the shell. **PARTIALLY CLOSED (2026-08-23), see RN-02.** |
| RN-02 | The reference's **NPR block** — ten "Painter" styles, coastal wave lines, animated water, multi-sun lighting | was RN-01's remit | this half was not merely unbound, it was **unported**: `render.rs`'s own module doc listed *"the 'Painter' NPR block (watercolor/contours/ink/hachure), multi-sun hillshade"* on its Excluded list | — | `PARITY_AUDIT.md` §3.1's ~15 missing render paths | **CLOSED (2026-08-23).** The ten styles, the wave lines and the multi-sun rig are literal per-pixel ports (`render::apply_npr`/`apply_waves`/`multi_sun_from_normal`/`coast_distance`), golden-verified against the reference under Node in `tests/golden_parity_npr.rs` (37 mutants, none survived — four survived a first sweep and were killed by shaping four more fixtures onto the exact gates they hide behind, never by loosening a tolerance) and off at every default, so no shipped pixel moved. They cross the boundary through `WorldGen::get_npr`/`set_npr` and are live in `render_workspace.gd` ▸ **Painter styles** / **Water & light**. Animated water is the one member that is *not* in the raster: it is per-frame, so it is a Godot `ShaderMaterial` overlay (`water_anim_layer.gd` + `water_anim.gdshader`) over `sample_bridge.rs`'s new `waterfx` field — principled equivalence (`DECISIONS.md` §7a), not golden, and stated as such. The reference's own `GW*GH <= 400000` animation cap is deliberately **not** ported: it protects a JavaScript pixel loop that no longer exists. Verified non-headlessly on the real GPU with a per-style PNG and a per-style movement measurement, an all-off return to the byte-identical base raster, a frame-to-frame measure that is non-zero only while the water overlay is on, and one real slider drag through the dock reproducing the engine call's raster exactly — a pass that found three bugs no test could have (`npr_api` guarding on a method that was never written, so the panel silently did not build; `Npr::peak_m` never filled from `params.peak_m`; and `waterfx` intensity selecting six cells of a 512×384 world, now keyed to `river_flow_thresh` like the map's own channel tint). What RN-01 still owes is the *colour/relief* half — `TerrainAppearance`'s ~40 palette and lighting fields — which `set_npr` does not touch. |

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
| SH-10 | **Phone: pinch-to-zoom did nothing** | *fixed 2026-08-24* — `project.godot`, new `[input_devices]` block | n/a — previously undisclosed, because nothing looked missing | yes | §13's map is the whole screen; pinch is the only zoom affordance a phone has | **(A)**, closed. Owner-reported (*"zooming doesn't seem to work on the phone"*). Not a code gap: `viewport_host.gd:406` had always handled `InputEventMagnifyGesture` and called the same `_zoom_at()` the wheel does. Godot's Android layer only attaches its `ScaleGestureDetector` when `input_devices/pointing/android/enable_pan_and_scale_gestures` is on, and the engine default is **false**, so the event was never produced and the branch was dead on every phone. Confirmed three ways: `ProjectSettings.has_setting()` true / unset value `false` on 4.7.1; `dexdump` of the shipped APK showing `onScale`/`onScaleBegin` gating on `panningAndScalingEnabled` (and `setQuickScaleEnabled` never called, so no single-finger fallback existed either); and a real two-pointer MT-B pinch injected through AOSP `uinput` on the device — **z1.0 → z2.2** out, **z2.2 → z1.0** in, against a **control APK with the setting off that reproduces the bug exactly (z1.0, unchanged)** |
| SH-11 | **`ViewportHost._zoom_at()` pivots against the wrong origin** | found 2026-08-24 while fixing SH-10; **not fixed** | n/a — previously undisclosed | — | §10's viewport is expected to zoom under the pointer | **(A)** small, open. `_input()` delivers `event.position` in *viewport* coordinates, but `_camera` is a child of `ViewportHost`, so `_camera.position` is `ViewportHost`-local; `viewport_host.gd:427` subtracts one from the other, so the zoom pivot is off by `ViewportHost.global_position` — measured at **(412, 70)** on the desktop layout (left rail + menu/tab bars) from a headless `app.tscn` instantiation, not inferred. Wheel and pinch are both affected; the *pan* branch is not (a delta of two global positions is offset-invariant), and `move_view_to()`/`_update_lod()` already work in local space, so line 427 is the single inconsistent site. Barely visible on the phone (edge-to-edge map, offset ≈ 0), which is why SH-10's fix verified clean. Left for a deliberate pass: it changes desktop zoom behaviour the owner currently calls correct, and `viewport_host.gd` had concurrent work in it |
| SH-12 | **`DccWidgets.note()`'s `custom_minimum_size.x` was wider than the right dock's own documented minimum** | *fixed 2026-08-24* — `dcc_widgets.gd::note()`, `240` → `190` | disclosed only in `CHANGELOG.md`'s "Still open" (695821f), never registered — `PARITY_AUDIT.md` pass 2's **F8** | now yes, this row | `DccTheme.W_RIGHT_DOCK_MIN` (260) is the dock's documented floor | **(A)**, closed. Static per context, so it never jittered (unlike SH-11's cousin bug this same file fixed for `_field()`'s value labels) — it was simply wrong: 240 px plus `section()`'s own 26 px of margin (14 left + 12 right) is 266, and a `group()` nested one level deeper adds 10 more, so the tightest real call site (`right_dock.gd`'s Measure ▸ Actions, a note inside a group inside a section) needed 276 against a 260 px dock. The right dock could not actually be dragged to its own minimum on any context that draws a note — nearly all of them (Sample-with-no-world, River, every empty Measure mode, Region, Sculpt, Wildlife). Fixed at the shared widget (`note()` is called from 18 files, not just `right_dock.gd`), so every caller benefits; the other 17 already give it wider columns and were unaffected either way. `190` leaves 33 px of clearance in the tightest nesting for the `ScrollContainer`'s vertical scrollbar. Headless boot-check clean; **the left dock and workspace panels were the other unaudited half of F8's own "still open" note** — same shared widget, so this fix covers them too, but neither was separately measured against a documented minimum the way the right dock was |

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
| UM-01 | **Town layouts drawn on the map at deep zoom** | `civUrbanLayoutsChk` | *partly closed, 2026-08-23* — the layer is live and draws real engine output; what it draws is a **street skeleton**, because blocks/buildings/walls are milestones 10-13 |
| UM-02 | **City Viewer modal** — its own canvas, zoom/pan, legend, info panel | `cityViewerModal`, `cvCanvas`/`cvCloseBtn`/`cvLegend`/`cvInfoPanel`, `_cvDrawCity`, `_cvZoomAt` | *partly closed, 2026-08-23* — `shell/city_viewer_window.gd`; same engine ceiling, stated on screen in the window's own info panel |
| UM-03 | **Layout thumbnail in the place-edit popup, and its launcher** | `peCityPreview`, `peCityOpen` | **`peCityOpen` CLOSED 2026-08-23** — the place-edit popup now exists (§18.1) and its Actions section calls `app.open_city_viewer(index)`, which is exactly the one line this row predicted. `peCityPreview` (the *thumbnail* inside the popup) stays open: it needs a rendered layout at icon size, not a modal |

### UM-01/UM-02 — what closed, 2026-08-23

`cartalith-urban` has its first consumer. Three new pieces, no change to the
engine crate itself:

- **`cartalith-civ::urban_adapter`** — the reference's block-2 `_um*` adapter
  (`_umSiteBoxKm`, `_umWaterNearKm`, `_umWaterReachKm`,
  `_umSiteKindFromTerrain`, `_umInferAge`, `_umRayBoxExit`,
  `_umWayBearingFrom`, `_umRouteEnds`, `_umPrimaryPaths`, `_umTerrainOrient`,
  `_umWaterCtx`, `_umTerrainCtx`, `_umPlaceContext`), plus the prefix of
  `generate()` that milestones 1-7 supply. `URBAN_MORPHOLOGY_SCOPE.md`
  milestone 17's own named home, started early and deliberately partial. Its
  module header carries the function-by-function boundary, including the eight
  `_um*` functions deliberately **not** ported and why each one's *consumer*
  is milestone 8+.
- **`cartalith-godot::urban_bridge`** — one `#[func]`, `urban_layouts(indices)`,
  batched so the full-grid river trace `_umWaterCtx` needs happens once per
  batch rather than once per town.
- **`shell/urban_layout_draw.gd`**, **`shell/city_viewer_window.gd`**, and
  `map_overlay.gd`'s "Urban layouts" block.

**What is still not drawn, and is absent rather than stubbed**: blocks and
plazas (milestone 12), parcels and buildings (12-13), districts and amenities
(13-14), the wall circuit and its gates (10), harbour and quay (9), bridges
and fords (9), farmland and hinterland detail (15). The bridge emits **no
key** for any of them — an empty `buildings` array would read as "this town
has none" rather than "this port cannot generate any yet" — and the City
Viewer's info panel names the list on screen.

**One deliberate divergence from the reference, stated rather than silent**:
the reveal gate is *not* `_umLayoutAlpha`'s 24 km → 10 km crossfade. That band
works in the reference because its LOD region window lets the camera reach a
few-km span; this port's camera clamps at `ViewportHost.ZOOM_MAX` (8.0), so on
the default 800 km world the closest reachable span is ~100 km and a ported
24 km threshold would never once fire — a toggle that silently draws nothing.
The gate is the town's site box measured in screen pixels instead. See
`map_overlay.gd`'s own block for the full reasoning.

UM-03 stays (B) rather than (C) or (D): the reference precedent is exact and
line-cited (`URBAN_MORPHOLOGY_SCOPE.md`), so it is an engine/UI gap, not a
design gap — the honest opposite of most of this register's (C) entries.

---

## 7 · Layer 3 — comparable-application research for (C)

Every (C) entry, with how established applications in the same space actually
solve the problem, what they call it, where it sits in their information
architecture, and a proposal concrete enough to build from without re-searching.
Sources are linked so the research is checkable rather than asserted.

### 7.1 Undo history panel, and what "global undo" covers — ED-02, PR-11

> **Partly overtaken by events, 2026-08-23 — read this box before the
> research below.** Global undo (ED-01) and the memory row (PR-11) are built.
> They are **not** what this section proposed, and the difference is worth
> stating rather than quietly leaving two documents to disagree.
>
> This section's proposal 1 — *"do not build one global stack; build a history
> ledger"* — was written from the comparable applications, before anyone had
> read `pushUndo`/`undoLast`. The reference's global undo is **three functions,
> one `Float32Array.slice()` and a five-deep array**. It snapshots the height
> field and nothing else: not `riverMask`, not `riverFloor`, not climate, not
> civ. A ledger with per-subsystem reversal is a strictly larger thing than
> the gap `PARITY_AUDIT.md` §3.1 actually names, and building it to close a
> three-function gap would have been the definition of over-engineering.
>
> So what shipped is the reference's own design with one bound changed
> (a byte budget, because 8192² worlds exist here and not there). What this
> section proposed remains the right *eventual* shape, and every piece of it
> is still unbuilt and still undesigned:
>
> - **Proposal 2's draft/commit tiering** — accurate and unchanged. What
>   shipped is the *commit* tier only, for the two commits that write height.
> - **Proposal 3's panel** — still (C), still undesigned, still ED-02. The
>   engine now has the data a panel would read (`undo_stats()`), which moves
>   the panel from "needs an engine" to "needs a design".
> - **Proposal 4's Preferences row** — shipped, and it kept this section's own
>   advice to show live memory cost. It diverged on the control's *unit*: a
>   budget in MB rather than a depth in steps, for the reason PR-11's row
>   gives. The reference's cap of 30 named here is wrong — `MAX_UNDO` is 5,
>   which is also what the shipped label says.
> - **Proposal 5's Adjust Last Operation** — untouched, still (A), still the
>   cheapest remaining win in this section.
>
> One correction to a *source* rather than to this section:
> `reference/FUNCTION_INDEX.md` line 61 describes the reference's undo as
> *"one level per destructive op"*. It is five (`const MAX_UNDO=5`), and the
> reference's own header label reads "Up to 5 steps saved in memory".

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

> **Acted on, 2026-08-23 — the proposal below was built, with its own
> recommendation taken** (inline group, not a `⧉` window). One assumption
> in it turned out to be wrong and is corrected in JP-05's row: the
> factors were *not* all already across the boundary, so the chain now
> crosses as structured `JpTerm` rows rather than being re-derived in
> GDScript from a second copy of the engine's tables. The reference's
> `formula` **prose** still stays out of the engine, which was the real
> constraint this section identified.

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

> **Resolved differently, 2026-08-23.** Global undo turned out *not* to be far
> off — it was three reference functions and a 5-deep snapshot array, not the
> command framework this register had assumed (see §7.1's box). `Edit ▸ Undo`
> is live, so the menu has its live item and it is the one it should have had.
> The finding behind this recommendation still stands as a lesson: the reason
> Edit sat 100 % disabled for so long was a scope estimate nobody had checked
> against the reference.

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
| MS-01 | **Center landmasses** | `#centerBtn` | **done, 2026-08-23** — the button in `app.gd`'s GENERATE · WORLD tool-options bar is live and calls `WorldGen::center_landmasses()`. Engine: `cartalith_terrain::center` (`bestEmptyColumn`/`shiftGridX`/`featherSeamX`, reference HTML 3156-3177) orchestrated by `cartalith_engine::center::center_landmasses` (`centerLandmasses`, 3179-3199). Golden-parity bit-exact (`golden_parity_center.rs`, 6 tests) | Was: "`generate_terrain` places plate seeds from the seed alone; no centring pass and no post-generate offset exist." The premise was right and the conclusion wrong — the reference does not re-roll seeds either, it circular-shifts every grid after the fact, which needs no generation hook at all. The old tooltip's "re-rolls the plate seeds until the land mass lands nearer the middle" was a misreading of the reference; corrected in the same pass |
| MS-02 | **Infer tectonics from heightmap** | `#inferTectBtn` | **done, 2026-08-20** — `Data ▸ Import ▸ Heightmaps (PNG)`, and the welcome screen's own *Import a heightmap* tile | Both halves closed in one pass. The reader is `cartalith-assets::raster::decode_png` + `cartalith_terrain::infer::heightmap_to_field`; the inference is `cartalith_terrain::infer` (`buildReliefField`/`pickPlateSeeds`/`classifyPlateCrust`/`reconstructBoundaryStress`/`stampVolcanicArcs`/`inferPlateVelocities`, reference HTML 6641-6752) orchestrated by `cartalith_engine::import::infer_tectonics`. Golden-parity tested bit-exact against the reference (`golden_parity_infer.rs`, 8 tests) |
| MS-03 | **Fold intensity · trench depth · fault blocks** (structured orogeny) | `foldI`/`trenchD`/`faultB` | `world_workspace.gd` — stage 04 Tectonics' `gap` string, which was **empty** | `generate_terrain` hardcodes the reference's own defaults (0.16, 1.0, 0), so behaviour matches; exposing them threads three fields through `OrogenyParams`' call site |
| MS-04 | **Evolve climate ↔ terrain · Evolve cycles** | `#evolveBtn`/`#evoCyc` | `world_workspace.gd` — stage 06 Erosion's `gap`, which named five passes and not these | **DONE, 2026-08-23 — §19.** `evolveCoupled` is pure orchestration with no kernel of its own, and every piece it composes already existed here except one: a reusable climate refresh over a changed surface. That function is now written — **`cartalith_engine::refresh_climate`**, `pub`, the reference's `computeFlow(true); refreshClimate();` tail (line 5154) — and Evolve is `passes.evolve_cycles` (`0` is off; the reference's slider starts at 2 because pressing the *button* is its "on", which a parameter has no equivalent of). One `refresh_climate` per cycle, which is the whole point: the rain driving the next cycle's incision reflects the orography the last one built. Verified non-headlessly at 4 cycles — 44.0 % of pixels moved |
| MS-05 | **Sediment fill** | `#sedimentBtn` | same stage `gap` | **DONE, 2026-08-23 — §19.** `depositSediment`'s kernel is `cartalith_erosion::route_sediment` (mass-conserving, golden-parity bit-exact), and the op composing it is now written into `generate_terrain`'s pass block: stream-power carve → per-cell eroded-column supply → `compute_flow` on the carved surface → `route_sediment`. Exposed as `passes.sediment_fill` + `passes.sediment_capacity` (the reference's own `6.0` default). Verified non-headlessly — 43.7 % of pixels moved. **`#tidalFlatsBtn` closed too, 2026-08-24**: with `cartalith_climate::tides` landed, `passes.tidal_flats` (+ `passes.tidal_k`, the reference's own `0.45`) is the seventh pass. It runs last, matching the reference's own source order, and builds the tide field from the finished surface right before the kernel reads it — which is what `refreshTides()` does there. **The pass toggle *is* the tides enable**, since `PlanetParams` has no moon roster: the field is built with the reference's own default single moon at this world's `planet.g`, the same substitution the Tides debug view already makes. Verified non-headlessly — 9.00 % of pixels moved, all-off returns to base at 0.0000 % |
| MS-06 | **Auto-populate world** (+ capitals / towns / hamlets counts) | `#civAutoPopulateBtn` | `civilization_workspace.gd` — disabled button in Settlements ▸ Not built | `compute_civilisation` runs inside `generate()`; no `civ_populate` `#[func]`, and `params.rs`'s 58 entries carry no civ parameter |
| MS-07 | **Clear places & routes** | `#civClearPlacesBtn` | same | `CivData` is rebuilt wholesale by `generate()`, never mutated in place — there is no partial teardown to expose |
| MS-08 | **Generate roads** | `#civAutoRoutesBtn` | `infrastructure_workspace.gd` — disabled button in Roads ▸ Not built | same shape as MS-06; the Way/Route tools are the wired alternative |
| MS-09 | **Clear ways & journeys** | `#civClearRoadsBtn` | same | same shape as MS-07. IN-02 closing (2026-08-24) makes committed manual ways *readable* but not clearable — `InfraTools::ways` still has no clear `#[func]`, so this stays disabled for both halves |
| MS-10 | **Recalculate territories** | — | `civilization_workspace.gd` — disabled button in Politics ▸ Not built | `assign_territory()` runs inside `compute_civilisation`; nothing re-runs it against edited settlements |
| MS-11 | **Clear territory** | — | same | same |
| MS-12 | **Generate provinces** | — | same | provinces are produced inside `generate()` and only read out. The *tint* half of the canvas's row is live (CARTO ▸ Layers ▸ Political — provinces) |
| MS-13 | **Add / remove faction** | — | same | **CLOSED 2026-08-23**, §18.1 — built, together with CV-07. The rest of this row is the pre-2026-08-23 state: `CIV_FACTION_COUNT` was a compile-time constant and factions had no identity across a re-generate |
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
| **Undo history (5 steps)** (`#undoMem`) | **PR-11 built 2026-08-23** (`Preferences ▸ Memory ▸ Undo history`, the live depth/cost readout the canvas asks for); **ED-02**, the panel, still open |
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
drop actually does rather than that it's unwired.

**Second update, same day (AS-07/AS-12/AS-17 closeout pass):** all three
closed for real. Per-item scale/pan editing (`as_set_item_transform`/
`as_reset_item_transform`), "Unassigned imports" (a reserved custom-slot
`set`, browsable and reachable from the footer's Import image… with no slot
focused), per-interior-line grid dragging (`SliceGrid::with_lines`/
`move_line`, real handles on every interior line) and cell-scoped slicing
(`as_slice_apply`'s new `only_cell`) are all real engine calls with test
coverage, not UI dressed over a gap. **Still open, honestly:** dragging a
file from *outside* Godot onto a slot to fill it (Godot's own drag-and-drop
is two unrelated systems — OS file drops reach `Window.files_dropped`, never
a Control's `_can_drop_data`/`_drop_data`, so a slot cannot structurally be
that kind of drop target); moving an already-assigned item *into* Unassigned
imports (only into/out of a Collection has an engine call); and per-item pan
is two SpinBoxes rather than the reference's own drag-on-canvas.

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

## 15 · The phone overflow menu is wired but inoperable (2026-08-20) — **RESOLVED 2026-08-23**

> **Resolved 2026-08-23**, and verified on the real OnePlus 6T rather than in an
> editor preview. The design this section was written to wait for arrived as
> `design/Cartalith Android Phone.dc.html`, and is implemented as
> `godot-project/shell/phone_menu.gd` plus an L1 bottom bar in
> `DccShell._build_phone_menu_bar()`. All four faults below are closed; the
> per-fault disposition is in **§15.1**, and the device evidence in **§15.2**.
>
> The section's own recommendation held up exactly as written: the fix
> re-presents `menus.gd` and **reimplements none of it** — every row is read off
> the real `PopupMenu` objects and every tap goes back out through
> `id_pressed`/`index_pressed`, so no menu id, callback or label is duplicated.
> Adding an item to `menus.gd` still makes it appear on the phone with no change.

Classification: **(C)** — a real, connected affordance with no phone design
behind its presentation. Recorded here as the brief for the mobile menu design
the owner is having produced separately; not fixed *at the time*, because
building one then would have been discarded when that design landed.

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

### 15.1 · How each fault was closed (2026-08-23)

| # | Fault | Disposition |
|---|---|---|
| 1 | Nothing phone-scaled | Closed. `phone_menu.gd` routes every size through its own `_ps()`/`_pt()` over `DccShell.phone_scale()` — the same helpers the rest of the phone chrome uses, not a second set of numbers. Rows are `_pt(52)`, icon buttons `_pt(44)`, both floored at `DccTheme.PHONE_TAP_MIN`. Measured on device: list rows land at **~129 physical px ≈ 66 dp**, clearing both the canvas's 44 dp bar and Android's 48 dp floor. |
| 2 | Desktop status chrome reparented into the sheet | Closed. The desktop menu bar and status bar are no longer drawn at all on the phone: they are parked in a hidden `PhoneMenuModel` host and used purely as the **model** — `menu_bar_row` for the seven real `MenuButton`s, `_status_labels` for readouts. The readouts return as real 52 dp list rows under a `Status` band on the menu's root screen, via the new `DccShell.status_slot_text()`. The 150 px wordmark is gone. |
| 3 | Menus do not respond to touch | Closed. Rows are `PanelContainer`s with their own `gui_input`, handling `InputEventScreenTouch` and `InputEventMouseButton` alike, with a pressed wash for feedback. Driven on device with real `adb shell input tap`/`swipe` at every level — see §15.2. |
| 4 | ~41 items behind 15 hover-opened submenus | Closed. No `PopupMenu` is ever popped up on the phone. The tree is re-presented as the canvas's five disclosure levels: L1 bottom bar, L2 root list, L3 one menu's items, L4 a 60 %-height sheet, L5 a full screen. Drilling replaces rather than stacks, and Android's system back pops one level at a time (`quit_on_go_back = false` plus `DccShell._notification()`). |

**One honest shortfall, unchanged from the design:** the canvas draws L3 bands
with titles ("§ HYDRAULIC PASSES"). Every `add_separator()` in `menus.gd` is
unlabelled, so a band draws as the hairline-and-gap the desktop menu itself
draws. Giving a separator text turns it into a caption with no change to
`phone_menu.gd` — but today it is a rule, not a heading, and that is stated
rather than faked with invented headings.

### 15.2 · Device verification (OnePlus 6T, 1080×2340, 2026-08-23)

Driven with real `adb shell input tap`/`swipe` and read back with
`adb exec-out screencap`. Not an editor preview — this is the same bug class
that produced the original gap, and a previous "buttons too small" fix was
wrongly marked done once already on editor-only evidence.

- **Portrait composition seen for the first time.** `STATUS.md` had recorded it
  as "still unseen" because `"sensor"` orientation defeats `adb`'s rotation
  override. The phone was in portrait for this pass, so the canvas's primary
  composition is now confirmed rather than inferred.
- **L1** — bottom bar renders `WORLD · CIVIL · CARTO · PANELS · MENU`, active
  cell in accent.
- **L2** — root screen lists all seven real menus under `Project` / `Content` /
  `System` bands with live item counts (File 11, Edit 10, Assets 9, Data 7,
  Preferences 19, Window 9, Help 5) and the `Status` readout rows.
- **L3** — `Preferences` drills in with breadcrumb `Menu · L3`; disabled rows
  draw their reason as a wrapped second line, which is *more* legible than the
  desktop, where that text is hover-only.
- **L4** — `Devices` opens as a sheet over a dimmed L3, breadcrumb
  `Menu · Preferences · L4`, and enumerates **real hardware live**:
  `Adreno (TM) 630 · integrated · vulkan`. Confirms `about_to_popup` fires on
  entry, so self-rebuilding submenus are as live here as on desktop.
- **L5** — `Assets ▸ Asset pack ▸ Edit` reaches a full screen, breadcrumb
  `Menu · Assets · Asset pack · L5`.
- **Back** — Android's system back popped sheet → screen → root without exiting
  the app (pid unchanged), matching the canvas's BACK rule.
- **Both palettes** — `Preferences ▸ Theme ▸ Light` fired from the L4 sheet
  repainted the sheet, the scrim, the screen behind it, the toggle and the
  radio. The light-theme path is verified on hardware, not assumed.
- **`Window ▸ Domain rail`** toggled off hides only the three domain cells;
  `PANELS` and `MENU` remain, so the menu that un-hides the rail is still
  reachable. Toggled back on from that same menu, all five cells returned.

### 15.3 · A 29.6-second freeze this work uncovered, and fixed

Making `Preferences ▸ Theme` reachable by finger for the first time exposed a
pre-existing defect in `DccShell.rebuild_theme()` that no desktop run had ever
made visible.

**Every `set_color()`/`set_stylebox()` on a live `Theme` emits `changed`, and
that re-propagates `NOTIFICATION_THEME_CHANGED` to every `Control` in the
tree.** `_recolor_project_theme()` performs one such write per remapped entry,
so each edit cost a whole-tree relayout. Measured on the device:

```
projectTheme=27336ms  windowChrome=1597ms  subtree=670ms  total=29603ms
```

The giveaway is `windowChrome`: **5** theme writes, 1597 ms — ~320 ms *per
write*. The cost is per mutation, not per colour examined. A first attempt at
memoising the colour lookups was therefore wasted (27336 → 27632 ms, no change)
and was removed rather than kept as decoration. Batching the writes behind
`set_block_signals(true)` and firing `changed` **once** (`_bulk_theme_edit()`)
gives:

```
projectTheme=274ms    windowChrome=417ms   subtree=685ms   total=1376ms
```

— 27.3 s → 274 ms for the dominant phase, **29.6 s → 1.4 s** overall.

Worth recording as a method note: the freeze presented as a **dead tap**, not as
slowness, and was twice misdiagnosed as a lost touch event — once as "the tap
did nothing", once as "the second tap worked" (it had not; the *first* tap was
still finishing). Only log timestamps either side of the emit showed the 29.6 s
round trip. A screenshot taken 3 s after a tap is not evidence that the tap was
lost.

## 16 · The top-left global tool overlay has no drawn presentation in the DCC canvas (2026-08-23) — **RESOLVED 2026-08-24**

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

### RESOLVED 2026-08-24 — the design landed, and it was two canvases

The hold is lifted and this section is closed. `design/Cartalith Paint
Toolbar.dc.html` and `design/Cartalith Measurement Toolbar.dc.html` were
vendored (commit `e7c10ab`) and implemented together, because they are one
design: the Paint canvas is the *unifying* bar — **one bar, three mode
buttons (Sculpt · Paint · Measure) on the left, the active mode's tools
beside them, an options bar below** — and the Measurement canvas is that
bar's Measure mode drawn in detail, plus a cross-section strip and a
right-dock readout block.

**What landed.** `tool_bar.gd` (the unified bar, hosted as a two-row
`VBoxContainer` inside the shell's single `tool_options_row` — no new shell
region), `section_strip.gd` (the bottom cross-section strip, a
`viewport_content` overlay in `resource_overlay.gd`'s mould),
`measure_bridge.rs` (the engine half), six Measure modes in
`global_tools.gd`, and six presentations of one `CTX_MEASURE` context in
`right_dock.gd`. Sculpt and Paint are **re-presented, not reimplemented**:
every control in those two modes writes through the same `bridge.sculpt_*` /
`bridge.paint_*` call `world_workspace.gd`'s left-dock panels already use.

**Every interaction decision the old file recorded survives** — Measure has
no commit, Escape clears the chain but leaves Measure armed (§4.5.6's own
exception), Region select is *not* one of those exceptions and still
disarms to Inspect after cleanup, and leaving either tool clears its draft
while Region's rect survives in the engine for *Send to Data ▸ Export*.

**New gaps this design opened, registered rather than silently skipped:**

| Tag | Control | State |
|---|---|---|
| **MEA-01** | Sculpt ▸ **Flatten** and **Noise** | **(B)** — the canvas draws seven sculpt tools; `FreehandMode` has eight and neither of these is among them. The row is built from `get_sculpt_freehand_modes()` live, so it shows the eight that exist. Two new kernels, not a wiring gap. |
| **MEA-02** | Paint ▸ **Water** and **Lithology** layers | **(B)** — `PaintTarget` is Biome/Terrain/Splat. Neither Water nor Lithology has an override array for a dab to write into; adding one is a `cartalith-spatial::paint` change plus a staleness edge, not a control. |
| **MEA-03** | Sculpt/Paint ▸ **Mask** | **(C)** — no mask channel exists in either editor, and none is designed. Disclosed in the bar's own note. |
| **MEA-04** | Distance ▸ **path ▸ great circle** | **(D)** — this map is equirectangular and `cartalith_spatial::measure` is planar with a seam rule. There is no spherical path to offer; offering one would report a distance that disagreed with every route length beside it. |
| **MEA-05** | Distance ▸ **snap ▸ settlements · rivers** | **(C)** — `DCC_SHELL_SPEC.md` §4.5.1 lists no snap modifier for Measure, deliberately (Way/Route have one; the ruler is raw). The canvas adds it. |
| **MEA-06** | **units ▸ km/mi** | **(A)** — the canvas itself says this *inherits* the app-wide switch (`#calUnitSeg`). The reference has one (`_setUnits`, line 13722, "switch km/mi and re-render all unit-bearing labels"); this shell has no unit preference at all, so every reading in the app is km. App-wide, not Measure's. |
| **MEA-07** | **Saved measurements** list, **Save**, **CSV**, **export PNG**, **save section** | **(C)** — no measurement store exists. `Copy reading` is built instead and puts every number on the clipboard as tab-separated text; a store is a persistence feature, not a measuring one, and shares FI-01's `.zip`-writer ceiling. **The writer landed 2026-08-23**; the remaining gap is the same one JP-06 names — a way for shell-owned state to reach the save's `state` object. |
| **MEA-08** | Cross-section ▸ draggable **A/B line-end handles** | **(C)** — a third click starts a new section instead. Same two clicks, no new on-canvas hit test. |
| **MEA-09** | Cross-section ▸ **Custom ▾** field | **(C)** — there is no user-defined field to bind it to. The other five channels (Elevation · Terrain · Climate · Hydrology · Geology) are live. |
| **MEA-10** | Area ▸ **rectangle** / **freehand** ring modes, and **⌥ subtract a hole** | **(C)** — polygon only. A rectangle is four clicks; a hole needs a second ring and a signed-area subtraction the readout has no place for yet. |
| **MEA-11** | Vertical tools **disabled in 2D** | **(D)** — the canvas gates Δ vertical and 3D distance on a 3D relief view. This port reads the same height field in both, so they stay live in 2D and the dock says why. |
| **MEA-12** | Crossings by **river name** | **(D)** — no river entity crosses the GDExtension boundary (this register's own River-context line, `right_dock.gd`). Crossings are described by Strahler order instead of an invented toponym. |

**Ridge crossings needed a definition and did not have one.** The canvas
prints "ridge crossings 2" and nothing anywhere — canvas, spec, or reference
— says what one is. `measure_bridge::RIDGE_PROMINENCE_M` is this port's own
answer, stated in the code and in the dock's tooltip: a local maximum
standing at least 100 m above the lower of the two valleys flanking it.
Without a prominence rule every ripple in a 1 024-sample profile counts.

---

## 17 · Debug-view gaps were never registered here — seven now closed (2026-08-23)

`PARITY_AUDIT.md` §5 item 8 caught the structural hole: the eleven
honestly-unavailable rows in `sample_bridge.rs`'s `GAP_LAYERS` were
disclosed **in code only**. Their reasons live in each `LAYER_GROUPS` row's
hint string, which the Layers popover shows to a user, but no row in this
register ever named them — so a reader walking `reference/FUNCTION_INDEX.md`
Part 0's Layers list against this file found nothing at all. The audit spotted
it via Wildlife; it applied to all eleven.

This section gives them ids. **Seven of the eleven have since been closed
rather than merely registered** — all on 2026-08-23, all from
`PARITY_AUDIT.md` §3.1: first the fjord, landform and windthrow clusters
(DV-01/02/03), then the geoid, tides, Köppen and wildlife clusters
(DV-06/07/04/11). `GAP_LAYERS` is down from eleven ids to **four**: two
still lack a *computation* (`oro`, `velo`) and two lack a *composite*
(`popdensity`, `siteprofile`).

### 17.1 · The register the debug views never had

| # | Reference view | `LAYER_GROUPS` id | Reference functions | State |
|---|---|---|---|---|
| **DV-01** | **Fjord mask** | `fjord` (Hydrology) | `buildFjordMask`/`carveFjords`/`currentFjordMask`/`carveFjordsOp`, HTML 3208-3249 | **done, 2026-08-23.** `cartalith_terrain::fjord`; view live; `#fjordBtn` live as *Carve fjords* in `world_workspace.gd`'s Glacial group. `golden_parity_fjord.rs`, 6 tests, bit-exact |
| **DV-02** | **Landforms** | `landform` (Surface) | `buildLandformField`/`currentLandform`, HTML 8082-8107 | **done, 2026-08-23.** `cartalith_terrain::landform`; view live with the reference's own `LANDFORM_COLS` as its legend. `golden_parity_landform.rs`, 6 tests, bit-exact |
| **DV-03** | **Wind-throw** | `windthrow` (Civilization) | `buildWindThrowField`/`currentWindThrowField`, HTML 5602-5636 | **done, 2026-08-23.** `cartalith_climate::windthrow`; view live, gated on the civilisation layer's water bodies (the biome raster is a real input, so a loaded save still reports unavailable). `golden_parity_windthrow.rs`, 4 tests, bit-exact |
| **DV-04** | **Köppen climate** | `koppen` (Climate) | `computeSeasons`/`classifyKoppen`/`buildKoppen`/`koppenColor` (7) | **done, 2026-08-23.** `cartalith_climate::koppen`; view live with a five-class legend off the frozen `KOPPEN_KEYS`. `golden_parity_koppen.rs`, 6 tests: the classifier is bit-exact against the reference's own captured seasonal fields. Picking it runs the temperature and weather models twice more (one solstice each) — the same cost the reference's own lazy build pays, and the slowest view in the popover |
| **DV-05** | **Orogeny** | `oro` (Tectonics) | the signed orogeny preview | open — needs the boundary-polyline structure `generate_terrain` folds into height and never retains |
| **DV-06** | **Geoid** | `geoid` (Tectonics) | `buildGeoid`/`refreshGeoid`/`geoAt`/`currentGeoidPreview` (4) | **done, 2026-08-23.** `cartalith_climate::geoid`; view live on a diverging ramp. `golden_parity_geoid.rs`, 7 tests, bit-exact. Previewed at the reference's own `0.015` default amplitude, which is precisely the state the reference previews in too — its own toggle defaults off. **WW-07's *parameters* stay open** |
| **DV-07** | **Tides** | `tides` (Tectonics) | `tidalForcing`/`computeTideField`/`buildTideField`/`refreshTides`/`currentTideField` (5) | **done, 2026-08-23.** `cartalith_climate::tides`; view live, water only. `golden_parity_tides.rs`, 6 tests, bit-exact — including the Green's-law cap and the geoid-on path. Previewed with the reference's own default single moon, the same substitution `currentTideField` makes while the toggle is off. **WW-07's *parameters* stay open** |
| **DV-08** | **Velocity** | `velo` (Hydrology) | the Mei virtual-pipe velocity-erosion pass | open — WW-02 |
| **DV-09** | **Pop density** | `popdensity` (Civilization) | the regional persons/km² estimator | open |
| **DV-10** | **Site profile** | `siteprofile` (Civilization) | the flood + slope buildability composite | open — both inputs exist individually |
| **DV-11** | **Wildlife** | `wildlife` (Civilization) | `buildTRI`/`guildTrophic`/`buildEcoregions`/`assignWildlife`/`regionRichness`/`wildRegionColor`/`currentWildlife` | **done, 2026-08-23.** `cartalith_civ::wildlife`; view live, gated on the civilisation layer's water bodies for the same reason `windthrow` is. `golden_parity_wildlife.rs`, 8 tests, bit-exact. `buildNPP` was already ported and is **consumed**, not re-implemented. Its **roster click popup** is now **WL-01** below, no longer an unregistered interaction gap |

`GAP_LAYERS` is now four ids, not eleven. `layer_available` gained two
per-world cases rather than two unconditional availabilities: `windthrow`
and `wildlife` both join `bclass`/`cterrain` on "needs the civilisation
layer", because both read the Cartalith biome grid and that needs water
bodies. A loaded save therefore still reports them unavailable, honestly,
instead of drawing an empty raster.

### 17.1a · WL-01 — the wildlife roster click popup

`PARITY_AUDIT.md` §5 item 8 listed this as a class-(d) row: a *reference
interaction* with no disclosure anywhere, because the register only ever
tracked the debug **layer**, never the click behaviour layered on it.

| # | Reference surface | Reference functions | State |
|---|---|---|---|
| **WL-01** | **Wildlife ecoregion roster popup** | `showWildInfo`/`hideWildInfo`/`wildFmtPop` (HTML 8257-8276), and the map-click branch at 9785-9791 | **done, 2026-08-23** |

Three notes on how it was built, each a deliberate choice:

- **The hit test is the reference's own.** `WorldGen::wildlife_region_at`
  takes the nearest region marker within `max(8, GW/40)` cells, skipping any
  region below `markerMin` — line for line with HTML 9787-9789. Outside that
  radius it returns an empty dictionary and the dock falls back to the
  ordinary sample context, which is what `hideWildInfo()` does.
- **The popup is a RIGHT-dock context, not a floating panel.** The reference
  positions a `#wildInfo` div at the cursor; this shell already routes every
  "you clicked something" readout through the dock's context switch, and a
  clicked ecoregion is a selection like any other. Every field
  `showWildInfo` renders is present and in the reference's own order.
- **`wildFmtPop` stays engine-side.** `wild_fmt_pop` is exported from
  `cartalith-civ` and the dock prints its string, so the `~4.5M` wording has
  exactly one implementation rather than a GDScript copy that could drift.

### 17.2 · Two divergences from the reference, disclosed

1. **No hillshade under the class rasters.** The reference draws `landform`
   and `fjord` over `shadeFactor(x,y)` (HTML 8486-8488). This port's debug
   rasters are flat-coloured across the board — `lith`, `soil`, `btype` and
   every other implemented row already are, and `sample_bridge.rs` computes
   no hillshade at all. Following the module's own convention beat matching
   the reference on two rows and leaving the other sixteen inconsistent.
   The fjord view's *non-fjord land* tone uses the reference's own
   `v*235*s` formula with `s` fixed at 0.47, the midpoint of the range
   `shadeFactor` would have produced.
2. **`carve_fjords()` does not re-run flow, rivers or climate.** The
   reference's `carveFjordsOp` follows the carve with
   `enforceRiverChannels()`, `computeFlow(true)` and `refreshClimate()`.
   This port has no re-runnable path for those — the identical gap
   `sculpt_commit` documents, and for the identical reason. The height
   field and everything derived from it alone are correct after the call;
   the flow, Strahler and climate rasters are as they were. Stated in the
   `#[func]`'s own doc comment and in the button's own note, not only here.

---

## 18 · The civ-interaction surface: place editing, the context menu, the Delete key and the faction roster (2026-08-23)

`PARITY_AUDIT.md` §5 called items 2 and 3 "the substantive ones" and item 3
"a live usability hole, not just an inventory gap: a user can add a
settlement they can never fix or undo." Eight of that list's fourteen rows
are civ-interaction rows; this section closes six of them, upholds one
existing decision, and registers the last as blocked with the real reason.

### 18.1 · What closed

| # | Reference surface | Where it lives now | State |
|---|---|---|---|
| **CX-01** | **Right-click context menu**, `_civCtxShow` (HTML 25857) and the `contextmenu` handler that fills it (25888) | `map_overlay.gd`'s new `map_right_clicked` signal → `viewport_host.gd` re-emit → `app.gd`'s broadcast → `civilization_workspace.gd`'s `on_map_right_clicked` | **done.** Five of the reference's six ops: Edit · Move viewer to · Delete · Drop settlement here · Info here. The sixth is Drop POI — see 18.2 |
| **CX-02** | **Delete key deletes the selected place** (block 2 keydown, HTML 26096) | `app.gd`'s `_unhandled_key_input`, broadcast to any workspace implementing `on_delete_key()` | **done.** Guarded against firing while a `LineEdit`/`TextEdit`/`SpinBox` has focus, and routed through the same confirmation the editor's own Delete button uses |
| **ED-03** | **Place edit popup**, `placeEditPopup`/`_civPopulatePlaceEditor` (HTML 16694) | `shell/place_editor_window.gd`, over five new `#[func]`s (`civ_settlement_details`, `civ_edit_settlement`, `civ_settlement_toggle_trait`, `civ_reroll_settlement_name`, `civ_delete_settlement`) | **done.** Name (plus the reference's culture-aware re-roll), class, polity, population, economy, the seven traits, age and walls overrides, history, focus camera, delete |
| **CV-07 / MS-13** | **Add / remove faction, persistent faction identity** | `civ_roster_bridge::FactionRoster` on `CivData`; `civ_add_faction`/`civ_remove_faction`/`civ_set_faction_field`/`civ_faction_count` | **done.** `_civAddFaction`/`_civRemoveFaction` ported including the revert-to-Unclaimed side effect. `CIV_FACTION_COUNT` now *seeds* the roster instead of *being* it |
| **FR-01** | **Faction Roster modal**, `_civOpenFactionsModal`/`_civRenderFactionList`/`_civPopulateFactionEditor` (HTML 16177/16247) | `shell/faction_roster_window.gd`, opened from CIVIL ▸ Politics ▸ *Faction roster…* | **done in part** — see 18.3 for the two blocks that are not built and why |
| **FR-02** | **Procedural faction banners**, `_civFactionBannerCanvas` (HTML 14849) | `shell/faction_banner.gd`, a `Control` with a custom `_draw()` | **done.** A port of the reference's own composition — shield outline with two quadratic sweeps, faction colour fill, one of six glyphs by `fid % 6` at 85% white. `Curve2D` with the exact quadratic-to-cubic control offsets, not an eyeballed approximation |
| **CV-10** | **"Land sustains ≈ N" readout**, `civPopEstimateOut` / `_civAgrarianRegionalTotal` (HTML 23516) | `cartalith_civ::timeline::civ_agrarian_regional_total` plus `civ_agrarian_regional_total()`; shown in CIVIL ▸ Settlements ▸ Roster and in the roster window's world overview | **done.** No such function existed anywhere in `cartalith-civ`; ported with `golden_parity_roster.rs` (Node `vm` over HTML 23516-23528, two cases pinning `cellKm²`) |
| **CV-11** | **Biome carrying-capacity residual**, `civBiomeKChk` / `_biomeK` (HTML 1406 / 6441) | `CivOptions::biome_k`, `set_biome_k_enabled`/`get_biome_k_enabled`; File ▸ New world ▸ Generation | **done.** `build_carrying_capacity` always took the parameter and nothing could turn it on. Default OFF, matching `_biomeK = 0` ("bit-identical") — and the wetland mask is built only when it is on, exactly as `currentCarryingCapacity` does |

Two more `#[func]` families landed with them, because the pickers need the
engine's own tables rather than a second transcription in GDScript:
`civ_trait_vocabulary`, `civ_specialisation_vocabulary`,
`civ_religion_vocabulary`, `civ_government_vocabulary`,
`civ_ag_tech_vocabulary`, `civ_culture_vocabulary` (all from the new
`cartalith_civ::roster`, ported verbatim), and `civ_faction_terrain_fits`,
which finally gives `civ_culture_terrain_fit` — ported earlier and labelled
"**Not wired to any caller yet**" in its own doc comment — a real caller.

`get_factions()` grew `name`, `religion`, `government`, `ag_tech` and
`population`, and its `culture` field changed from a recomputed
`civ_default_culture(f)` into a read of real roster state. Its own doc
comment used to assert that "the reference has no faction *name* registry
beyond this"; it does, and the correction is recorded in that comment.

### 18.2 · CV-01 (POI) — the decision was checked, and upheld

`civ_tools_bridge.rs`'s module doc says POI "is not a ported concept," and
CV-01 records the tool as *omitted rather than built inert*. That was
re-read before touching anything here. It is a real state-of-the-port fact,
not a stale exploratory note: `cartalith-civ/src/tools.rs` ports Settlement
and Territory only, `civ_place_pick_weight`'s own doc says the reference's
POI branch "is likewise absent because this port has no POI concept," and
there is no POI record type anywhere in the workspace to attach a drop to.

**Nothing in this pass reverses it.** Concretely:

- the context menu ships five ops, not six — "Drop POI here" is *absent*,
  not shown disabled, matching how `civilization_workspace.gd`'s own
  `_build_tools()` already treats the POI tool;
- the place editor ships no **Category** selector (settlement ↔ POI), which
  in a port with one category would have had exactly one option;
- `civ_settlement_details`/`civ_edit_settlement` are settlement-only, and
  the new `#[godot_api]` block's own header comment says so.

If POI is ever wanted it is still CV-01's own estimate — one `civ_drop_poi`
mirroring `civ_drop_settlement`, plus a real record type — and it remains an
owner call, not an implementation detail.

### 18.3 · What is registered open, with the real reason

| # | Surface | Why not built |
|---|---|---|
| **FR-03** | The Faction Inspector's **Power breakdown** (military / economic / political / cultural / religious) and its **Economy** block (food production and surplus, tax income, trade income, primary exports and imports, strategic resources, craft share) | Both read `_civFactionAggregates`' resource- and density-fed half. `civ_faction_aggregates` **is** ported and is now called for real (`civ_faction_terrain_fits`) — but with `resources: None, density: None`, which is all the terrain-mix half needs. Filling those in means retaining the 15 resource rasters and a population-density field past `compute_civilisation`, which `MEMORY_OPTIMIZATION_SCOPE.md` deliberately paid to avoid. A memory decision plus an `ECONOMY_SCOPE.md` milestone, not a widget |
| **FR-04** | **Diplomatic relations** | No model in either codebase. The reference's own inspector renders "Diplomatic relations — not yet implemented"; so does this |
| **CV-12** | **Placement-diagnostics overlay**, `civDiagnosticsChk` (HTML 1415, `drawCivLayer` §2.6 at 15617) | **Blocked on urban morphology, not on UI.** Every line of the fact card it draws is `_um*` data: `_umWallSpec`'s wall ladder, `_umSiteProfile`'s river classification and coast distance, and a peek into `_umModelCache` for bridge/ford/harbour validity — inside a `SITE_WM × SITE_HM` footprint box. `cartalith-urban` milestones 8-17 are unported, so the overlay would have nothing to draw. Registered as a **disabled control carrying that reason**, in CIVIL ▸ Settlements ▸ Not built |
| **ED-03a** | An edited **specialisation** does not reach `civ_faction_aggregates`' sector output | `FactionPlace::specialisation` is a field that function reads, and every caller passes `None`. Feeding user edits in would change already-golden economy numbers on an interactive edit — a decision to take deliberately, not a wiring detail. Stated in `civ_roster_bridge`'s module doc, in the editor's own Economy tooltip, and here. **SG-02 does not close this** (checked 2026-08-24): `recompute_civilisation` rebuilds `trade_balances`, which is `civ_resource_trade_balance` over the settlement's own catchment — `civ_faction_aggregates` is not on the path at all, so `specialisation` still reaches nothing |
| **ED-03b** | The **age** and **walls** overrides are stored and consumed by nothing | Their only readers are `_umInferAge`/`_umInferWalls`/`_umWallSpec`. Same block as CV-12 |
| **ED-03c** | The seven **traits** are stored and never drawn on the map | The reference draws them as glyphs beside the marker; `map_overlay.gd` has no per-trait glyph pass, and that file is deliberately minimal-touch this pass |
| **UM-03** | The layout thumbnail (`peCityPreview`) and its City Viewer launcher (`peCityOpen`) inside the place popup | UM-03 called this "doubly blocked: no place-edit popup exists at all (ED-03) and no city layout to preview even if it did." **Half of that is now false** — the popup exists. The remaining half stands |
| **ED-03d** | A place edit or delete does **not** recompute provinces, trade balances, roads, territory or `explanations` | **CLOSED 2026-08-24 (SG-02).** `recompute_civilisation()` rebuilds all five against the current roster and terrain, and the Civilization dock's Settlements ▸ Recompute button calls it; verified in a real run — a hand-dropped capital moved territory, roads, provinces *and* trade balances, all four of which were unmoved before. It stays an explicit button rather than a cascade on every edit: 4.22 s at 2048² is not a per-keystroke cost. The disclosures this row was written about (`civ_delete_settlement`'s doc comment, the delete-confirmation dialog, `_settlement_click`'s status hint) now name that button instead of saying "not recomputed" full stop |
| **CV-13** | A faction added after generation owns nothing until something is assigned to it | Not a gap — the reference's `_civAddFaction` behaves identically (it appends to `CIV_FACTIONS` and touches nothing already placed). `assign_factions` runs inside `generate()` at `CIV_FACTION_COUNT`; the status hint after Add says exactly this |

### 18.4 · Verification

- `cargo test -p cartalith-civ` — all suites green, including the new
  `golden_parity_roster.rs` (3 tests: 13 golden `_civFactionColor` values
  across every hue sector, two `_civAgrarianRegionalTotal` cases, and a
  land-gate negative control).
- `cargo test -p cartalith-godot --lib` — 263 passed, including
  `civ_roster_bridge`'s 10 new unit tests (roster seed / add / remove floor,
  revert-to-Unclaimed, vocabulary rejection, trait toggle order, the
  age/walls clamps, delete-by-index).
- Headless boot (`--headless --path godot-project --quit`) clean.
- Interactive verification is recorded in this section's own entry in
  `cartalith-native/docs/CHANGELOG.md`.

---

## 19 · The manual erosion passes: kernels ported, wired as generation parameters (2026-08-23)

**`PARITY_AUDIT.md` §3.1's row read "kernels partly absent, no run-button
path".** Both halves close in this pass. The kernels are bit-exact ports; the
run path is **generation-time parameters, every one off by default**, under
`DECISIONS.md` §7d. This section states it once so WW-02, MS-04 and MS-05 can
all point here.

### 19.1 · What was ported

`cartalith-erosion/src/passes.rs`, a new module, bit-exact against the frozen
reference and mutation-swept:

| Kernel | Reference | Lines |
|---|---|---|
| `hillslope_diffuse` | `hillslopeDiffuseCPU` | 3872-3882 |
| `centrifugal_shear` | `centrifugalShear` (+ `_bilin`, inlined) | 3919-3930 |
| `velocity_erode_kernel` | `velocityErodeKernel` — Mei virtual pipes | 3936-3994 |
| `glacial_kernel` | `glacialKernel` | 4198-4257 |
| `coastal_process` | `coastalProcess` + `coastalProcessCPU` | 4388-4424 |
| `route_sediment` | `routeSediment` | 4286-4307 |
| `apply_tidal_sedimentation` | `applyTidalSedimentation` | 4324-4334 |

`VelocityParams`, `GlacialParams` and `CoastalParams` carry every knob
`GENERATION_PARAMETERS.md` itemised for these rows (3, 4 and 4 respectively),
and `hillslope_diffuse` takes its 2 as arguments.

### 19.2 · The decision, and which way it went

**The reference runs none of these inside `generate()`, and says so in its own
comments** — `evolveCoupled`: *"A new op (never auto-runs) → generate()
bit-identical at defaults"*; `glacialKernel`: *"Manual Glacial erosion button
+ its worker path only — not part of default generate()"*. Each is a button
that mutates the finished field and then re-derives flow and climate
(`erodeFinish` / `eroFinish` / `veloFinish`).

That leaves two shapes, and they are not equivalent:

- **(a) Opt-in run buttons**, the reference's own shape. *This is not new
  architecture in this port* — `WorldGen::carve_fjords()` (`#fjordBtn`) and
  `WorldGen::center_landmasses()` (`#centerBtn`) are both live
  post-generation field-mutating ops, and the fjord one already sits **inside
  `world_workspace.gd::_build_erosion_passes`**, in the Glacial group, next to
  the five disabled placeholders, with the note *"it never runs during
  generate, so a default world is unchanged by this control existing."*
  Distinct from **WW-11** (per-stage `Run stage n` re-execution), which is
  (D) — a capability that exists in neither this engine nor the reference.
- **(b) Generation-time parameters**, default-off. Permitted by
  `DECISIONS.md` §7d — the default reproduces the reference exactly — but it
  requires choosing *where in `generate_terrain`* a pass the reference never
  inserts anywhere should be inserted. That is a pipeline-order decision with
  no reference answer and therefore no golden fixture for the composed result.

**(b) was taken.** §7d permits a superset exactly when the default reproduces
reference behaviour, and here every toggle is off, so it does — verified as an
assertion, not asserted as an intention (19.4). The pipeline-order question
(b) raises is answered honestly rather than dissolved: the passes run **at the
very end of `generate_terrain`, after `carve_rivers`**, because "the finished
field" is what each of these buttons operates on in the reference, and the
order among them is the reference's own panel order — `velocity → glacial →
coastal → hillslope → evolve → sediment_fill`. There is still **no golden
fixture for the composed result**, because the reference never composes two of
these in one op; each kernel is bit-exact alone, the sequence is this port's
choice, and `ErosionPassParams`' own doc comment says which is which.

(a) is **not** foreclosed and is now cheap: the kernels and the run path both
exist, so a run button would be a `#[func]` over the same code. It was not
built in this pass because UI work is on hold (`CLAUDE.md`) and the parameter
path needs no UI to be real.

### 19.3 · What each row got

- **WW-02** — `cartalith_engine::ErosionPassParams` on `WorldParams`: six
  toggles (`velocity`, `glacial`, `coastal`, `hillslope`, `sediment_fill`,
  and `evolve_cycles` as a count where `0` is off — a seventh, `tidal_flats`,
  joined them on 2026-08-24, see 19.5) plus fifteen knobs, each
  knob defaulting to the reference's own `state` literal. **21 rows in
  `params.rs`**, in the existing `erosion` group; each knob row names its
  reference slider and carries that slider's real reachable range through its
  own `eparam` mapping. The six toggle rows have an empty `reference_control`
  and say why — the reference's control is a *button*, not a checkbox, so the
  toggle is the §7d addition itself. **Droplet stays open**: its kernel has
  existed since Phase 1, but it was not in this pass's remit and its
  `erodeFinish` tail (thermal + clamp + rebound) is a second orchestration.
- **MS-05** (`#sedimentBtn`) — `depositSediment`'s orchestration, transcribed
  into the pass block: stream-power carve → per-cell eroded-column supply →
  `compute_flow` on the carved surface → `route_sediment`.
  **`#tidalFlatsBtn` closed a day later (2026-08-24)** — see 19.5.
- **MS-04** (`#evolveBtn`) — the missing engine function was written:
  **`cartalith_engine::refresh_climate`**, `pub`, the
  `computeFlow(true); refreshClimate();` tail (reference line 5154) over a
  changed surface. Evolve calls it once per cycle, which is the entire point
  of Evolve — the rain driving the next cycle's incision must reflect the
  orography the last one built — and the pass block calls it once at the end
  for the whole sequence.

**One deliberate deviation, disclosed.** The pass block ends with
`erodeFinish`'s own `if(f<0)f=0; else if(f>1)f=1;` clamp (reference line
3894), which the reference applies only after the *droplet* pass. Found by a
test rather than reasoned about in advance: `velocity_erode_kernel` carries
only a ±1e9 finite guard and `route_sediment` adds without an upper bound, so
both genuinely can leave a cell outside 0..1. In the reference that is a
transient a user re-runs past; here it would be baked into a `WorldState`
whose 0..1 field range the renderer, every downstream stage and
`generate_terrain`'s own end-to-end test all assume. Applied once after the
last pass, so no pass reads a clamped value the reference would have left
unclamped.

### 19.4 · Verification

- `cargo test -p cartalith-erosion -p cartalith-hydrology -p cartalith-engine`
  — green, including the new `golden_parity_passes.rs` (26 tests, `assert_eq!`
  on `f32`, no tolerance, bit-exact on the first run) and
  `cargo test -p cartalith-godot --test params_mapping` (the 21 new rows pass
  the existing range/uniqueness/reachability tests unmodified).
- Two new `cartalith-engine` tests for the *wiring*, which golden tests on the
  kernels cannot reach. `erosion_passes_off_leave_generation_bit_identical`
  moves five knobs with every toggle still off and `assert_eq!`s field,
  temperature, rainfall **and** discharge — a knob alone must do nothing at
  all, or "default-off" is only half true.
  `each_erosion_pass_changes_the_field_on_its_own` runs each of the six alone
  and asserts the surface moved, stayed finite and stayed in 0..1; its glacial
  case drops the snowline *and* freezes the world, because ice needs both, and
  compares against its own climate-only twin so the climate override cannot be
  mistaken for the pass.
- **Mutation sweep: 115 literal sites, 98 killed.** Four fixture passes were
  shaped to reach what the first sweep missed. The 17 survivors are each
  explained in `passes.rs`' module header — dead branches in the reference,
  thresholds redundant with a constant that *is* pinned, and razor-edge
  windows narrower than the perturbation. One is a real finding:
  `applyTidalSedimentation`'s `tr <= 1e-5` floor is **unreachable**, because
  any cell it could gate is already excluded by the `sea - 1e-4 - h` headroom
  cap.
- `cargo clippy -p cartalith-erosion -p cartalith-engine --all-targets` —
  clean.
- `cargo build -p cartalith-godot` — fresh shared
  `target/debug/cartalith_godot.dll`.
- **Non-headless, in the real app** (`_erosion_shot.gd`, an untracked harness
  in the `_npr_shot.gd` mould): one 512×384 world at seed 483920, driven
  through the real `EngineBridge` with `reset_params()` → `param_set()` → a
  full re-generate per case, because these are *generation* parameters and so
  the pass has to be measured across two generations rather than one render.
  Share of pixels moved by more than 3 levels: velocity 38.2 %, glacial
  91.3 %, coastal 6.4 %, hillslope 44.5 %, sediment fill 43.7 %, evolve
  44.0 %. **All-off returns to the base map at 0.0000 %** — the default-off
  guarantee holding through the whole GUI path, not just in a unit test. The
  maps were looked at, not only measured: hillslope has visibly rounded the
  ridge detail, velocity has reworked the drainage and crenulated the
  coastline.
- The honest control in that run: **`glacial` with the snowline dropped but
  the world left temperate moves 0.24 %**, because ice needs `temp < 0` as
  well as altitude. A wire-up that ignored the temperature gate would have
  scored like the frozen case; this one scores like nothing happening, which
  is exactly what should happen.

### 19.5 · The seventh pass: tidal flats (2026-08-24)

`#tidalFlatsBtn` was the one row 19.3 left open, and for a specific reason:
`apply_tidal_sedimentation` was ported and tested in that pass, but the field
it reads had no producer, and *a toggle over an always-absent field is a
control that cannot work*. `cartalith_climate::tides` (WW-07's engine half,
landed the same day) is that producer. The field shapes were checked rather
than assumed: `compute_tide_field` returns a `Vec<f32>` of `gw*gh` spring
tidal ranges with land exactly `0`, and `apply_tidal_sedimentation` takes
`&[f32]` of `w*h` and skips `tr <= 1e-5` — which land satisfies exactly. They
line up.

**`passes.tidal_flats` + `passes.tidal_k`** (the reference's own `0.45`) are
the seventh toggle and its knob, in the same `erosion` group as the other six,
with the same empty `reference_control` and the same reason. The pass runs
**last**, which is both the reference's own source order (`applyTidalSedimenta
tion` sits immediately after `depositSediment`) and the right physical order:
mudflats accrete onto the coastline the passes above finished shaping.

**One thing about this pass is not like the other six, and it is the
interesting part.** The reference gates its button on `tideField`, which only
exists while `state.planet.tides.enabled` is on — so the reference has *two*
switches where this port has one. This toggle is both: turning it on builds
the tide field from the finished surface right before the kernel reads it,
which is exactly what `refreshTides()` does there before the button is
reachable. `PlanetParams` carries no moon roster, so the field is built with
`TideParams::default()`'s single Earth–Moon-equivalent companion at this
world's own `planet.g` — the same substitution the Tides debug view (DV-07)
already documents for itself. That is a partial answer to WW-07's open
parameter question, for one sub-system: **the consumer's toggle is the enable**.
The geoid half has no consumer yet and stays open.

**Verification.**

- `cargo test -p cartalith-engine -p cartalith-erosion -p cartalith-hydrology
  -p cartalith-godot` — green. `each_erosion_pass_changes_the_field_on_its_own`
  now runs seven cases, and `erosion_passes_off_leave_generation_bit_identical`
  moves `tidal_k` too.
- One new test the "something moved" table cannot substitute for:
  `the_tidal_flats_pass_only_raises_submerged_cells_toward_sea_level` asserts
  the pass's actual *shape* — every changed cell was submerged, every change is
  upward, none is pushed past sea level, and sea level itself did not move. A
  sign error or a swapped `sea`/`depth` would still move cells and would still
  pass the table.
- **Measured, at grid resolution**, on a 256×192 world at seed 4242: 3,051
  cells accreted — 6.21 % of the grid, **19.58 % of every water cell** — mean
  rise 0.01968 of the 0..1 range, max 0.05129. Water only, upward only.
- **Non-headless, in the real app**, same harness shape as 19.4: 9.00 % of
  pixels moved on a 512×384 world at seed 483920, and **all-off returns to the
  base map at 0.0000 %**. The maps were looked at: the shoals inside the
  bays — the bottom-right bay and the central sea's margins — have visibly
  lightened as they fill toward sea level, and no coastline has moved, which is
  what accretion capped at `sea - 1e-4` should look like.

## 20 · GeoJSON export: the engine was finished, the boundary was missing (2026-08-24)

DM-03's own row estimated this at *"one `#[func]` plus assembling a
`GeoJsonWorld`"*. That estimate was exact, and it is worth saying why the gap
looked bigger than it was from outside: `FUNCTIONAL_CONTRACT.md` read the
capability as **Absent**, when `cartalith_engine::geojson` had been a complete,
golden-verified port of nine reference functions since milestone E2 —
character-for-character against the reference's own document, including the
hand-written JSON writer that exists because `serde_json` renders an integral
`f64` as `16.0` where `JSON.stringify` renders `16`. What was absent was a
caller. This is the same shape as Export ▸ Maps (DM-02): ported, tested,
callerless, and therefore invisible.

### 20.1 · The binding

`crates/cartalith-godot/src/geojson_bridge.rs` — one `#[godot_api(secondary)]`
block, one `#[func] fn export_geojson(&self) -> GString`, in its own file
rather than in `lib.rs`, which is a shared hot file. It returns `""` before the
first `generate()`/`load_save()`; everything else is assembly.

Three of the reference's inputs have no equivalent here, and the document
handles each by **omission rather than invention**:

- **POIs.** `CIV_POI_KEYS` splits `state.places` into settlements and points of
  interest. This port's `SettlementKind` is the six settlement tiers and
  nothing else, so there is no `poi` layer — not an empty one, which would
  claim the world has no POIs rather than that this port has no POI concept.
- **`w.sea`.** The reference keeps land ways and sea lanes in one flat
  `civWays` and distinguishes them with a flag. This port keeps them as two
  typed collections (`CivData::ways` / `::sea_routes`), so the flag is
  *derived* from which collection a way came out of — the same information,
  flattened at the one place that needs it flat.
- **Rivers.** `_riverNet` is a lazy global cache there; here the receiver tree
  and Strahler orders are already on `WorldState`, so the polylines are
  re-traced (`trace_river_polylines`, exactly as `urban_bridge` does). The
  export's own `min_order` is the reference's **2**, not the `1` the carve
  pipeline traces with.

**One function had to be ported to do this**, and it is not in the geojson
module: `splitRiverPolylines` (reference 4596-4608), which cuts a chain
wherever the next point is not reachable by a straight stroke — the
antimeridian seam, and optionally an open-water predicate. Without it a wrapped
receiver chain exports as one `LineString` drawing back across the whole map in
any GIS consumer. It is pure hydrology geometry, so it went to
`cartalith_hydrology::split_river_polylines`, next to the tracer, with two
golden tests whose fixtures were produced by **running the reference's own
function under node**, not by reading it. The export passes no skip predicate,
matching the reference's own comment: a lake reach is real hydrology and
belongs in the exported geometry; only the unrepresentable seam jump is cut.

### 20.2 · The route

`data_manager_window.gd`'s Export ▸ GIS / GeoJSON row flips from `gap` to
`live`: a pane, two disclosures (the CRS note, which is the document's own
`note` property verbatim, and the civ-layer note), and one `Export .geojson…`
chip over a save picker. There is no options pane because the binding has no
options — `export_geojson` describes the whole world, and every layer it can
emit it always emits. It exports the **whole world, not the Region-select
marquee**; the marquee is Export ▸ Maps' input, and the pane says so.

CRS handling still does not exist anywhere in the workspace and this does not
add any. That is disclosed in the pane and in the document itself.

### 20.3 · Verification

- `cargo test -p cartalith-engine -p cartalith-erosion -p cartalith-hydrology
  -p cartalith-godot` — green, including the two new
  `split_river_polylines_matches_the_reference_*` goldens.
- `cargo build -p cartalith-godot` — fresh shared cdylib.
- **Non-headless, in the real app**, through the real `EngineBridge`: a
  512×384 world at seed 483920 exported **305,646 bytes in 21 ms** to a real
  file. It parses as JSON. It carries **511 features — 239 settlement, 43 way,
  216 river, 6 territory, 7 province** — and those counts were cross-checked
  against the bridge's own getters in the same run (239 settlements, 43 roads,
  0 sea routes; this world genuinely has no sea lanes, so the `sea:true` branch
  is exercised by no feature here and is the one path that went unverified in
  the real app). Every coordinate lies inside the world's own 1200 × 900 km
  box. Numbers render the JS way — `"mapWidthKm":1200`, not `1200.0`. River
  `strahlerOrder` runs 2 · 3 · 4, confirming the export's own min-order of 2
  rather than the pipeline's 1.

## 21 · The staleness consumer is wired; its UI deliberately is not (2026-08-24)

`GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` §3.2.4 found the pipeline's
staleness graph (`cartalith_engine::staleness::pipeline_stage_graph`) correct,
tested and **consumed by nothing**, so every post-generation edit stopped at
the height field. The owner authorised the engine-side half. That half is now
built and wired into the commit paths, so an edit takes effect end to end
without any new control existing:

| Path | Marks | Runs |
|---|---|---|
| `WorldGen::sculpt_commit` | `Height`, at the pass's own tiles | hydrology + climate, one `refresh_climate` |
| `WorldGen::carve_fjords` | `Height`, whole map | hydrology + climate |
| `WorldGen::paint_commit` | `Civ`, at the painted tiles | nothing — a mid-chain edit does not make its own upstreams stale |
| `WorldGen::recompute_stale_stages()` (new `#[func]`) | nothing | whatever is already stale |

All four return `recomputed` and `still_stale` as `PackedStringArray`s.
Measured `--release`: **76.5 ms @512², 97.8 ms @1024², 188.9 ms @2048²** —
18.8× cheaper than the full generation it replaces. See
`cartalith-native/docs/CHANGELOG.md`, "The staleness graph gets its consumer".

**No GUI was built for it, on purpose.** `CLAUDE.md`'s UI hold stands, and the
brief for that work was explicitly out of scope. What the engine half unblocks
is registered here so it is picked up as a design rather than improvised as a
button.

### 21.1 · The register

| Tag | What | Backed by | State | Why it is not built |
|---|---|---|---|---|
| **SG-01** | A **staleness indicator** — the DCC mockup's own *"downstream update: rivers · deferred"* status line, showing which stages are stale and why | `StageGraph::stale_stages` / `staleness()` return the stage names *and* the most-upstream reason string, already; `sculpt_commit`/`carve_fjords`/`paint_commit` return `still_stale` on every call | **open, no design** | Where it lives (status bar? per-stage chips in the World workspace's stage list? both?) is a shell-layout decision, and `DCC_SHELL_SPEC.md` has no surface for it |
| **SG-02** | A **"Recompute now"** control for the stages a commit leaves stale — today that is always `civ` | `recompute_stale_stages()` exists and is callable; `recompute_civilisation()` is the civ half, and `civilization_workspace.gd`'s Settlements ▸ **Recompute** section is the control | **CLOSED 2026-08-24** | See the note below for the design (what is re-derived, what is preserved, what deliberately is not) and the measured cost |
| **SG-03** | **`param_set` marking the graph** — a moved dial invalidating the stage it actually affects, instead of `engine_bridge.gd`'s blanket *"a moved dial does not recompute a stage, it marks the world stale until the next full generate"* | Nothing yet: the graph is marked only by the three commit paths | **open, needs a design first** | Needs a per-parameter → stage table over `params.rs`'s entries. That is a real design decision (does `climate.rain_k` invalidate climate only, or civ too? does `tect.seed` invalidate everything, i.e. a full regenerate?), not something to improvise inside a setter |

### SG-02, closed 2026-08-24 — what "recompute civilisation" was decided to mean

The engine half is `WorldGen::recompute_civilisation()`; the control is a
**Recompute** section in the Civilization dock's Settlements category
(`civilization_workspace.gd`), chosen over a menu item because that dock is
where every readout the call fixes already lives.

The design question was not whether to rebuild the civ layer but **how much
of it**, and the answer is deliberately not "all of it":

- **Re-derived** — everything downstream of the settlement list, against the
  *current* terrain: water bodies, biome, lithology and soil, resource
  potentials, the hierarchical road topology and its consolidated ways, sea
  lanes, territory, provinces, per-settlement trade balances, the suitability
  `explanations` (correctly re-indexed) and agrarian density.
- **Preserved** — the settlement list itself, and with it everything keyed to
  it: hand-dropped places (`civ_drop_settlement`), hand-edited names, tiers,
  populations and factions (`civ_edit_settlement`), the `tid`-keyed
  `place_extras` side table (traits, specialisation, history, age/walls
  overrides), the faction roster, the recorded timeline and year, and
  hand-painted territory — which `CivTools::rebase` re-anchors onto the newly
  computed borders instead of erasing. `CivTools::commit` could not do that
  job: it is driven by the in-progress draft and returns early when it is
  empty, which it always is at recompute time.
- **Not done** — settlement *placement* is not re-derived. Re-running
  `find_settlement_seeds`/`place_settlements` would move every settlement,
  re-roll every name from a fresh RNG, and drop every hand-authored place and
  every side-table entry keyed to a `tid` that no longer exists. Re-placing
  from terrain already has a control: Generate. The metropolis promotion, the
  village seeding pass and the recovery phase are skipped on this path for
  the same reason — each of them *authors* settlements, and re-running them
  would overwrite a user's own edit or append a second copy.

Consequence worth stating plainly: sculpt a mountain under a city and the
recompute reroutes its roads and redraws its borders, but the city stays on
the mountain.

**Found by the real-shell run, not by reasoning: villages are not road
network nodes.** The reference seeds them *after* `_civHierarchicalNetwork`
has run, so an auto-populated village-enabled world has roads between its
placed settlements and none to its villages. Feeding the whole kept list back
into the topology — which the first implementation did — took a 384 × 288
world from **35 ways to 240 on one button press**, restructuring the map
rather than catching it up, and tripled the call's cost (4.3 s → 0.7 s once
fixed). `CivData::village_tids` now records which settlements
`civ_seed_villages` added, keyed by `tid` because neither an index nor a
trailing range survives `civ_delete_settlement`'s splice or
`civ_drop_settlement`'s append; the recompute builds the network from the
non-village settlements and remaps the edge endpoints back. Same world after
the fix: 35 ways before, 35 after, rerouted around the new mountain.

**Measured** (release, CPU path, square grids at 1200 km): **0.94 s at 512²,
1.60 s at 1024², 4.22 s at 2048²** — about half the cost of a full
`generate()` of the same world on the same run (1.28 s / 2.59 s / 8.16 s),
and below `UNIFIED_TOOL_PLAN.md` milestone C's ~7 s/stroke figure precisely
because placement and naming are the parts it skips. No fast path: a second
call on unchanged input costs the same, to within a few ms.

**Not registered, because it is not a gap:** the carve-time river network
(`channels`, `stream_order`, `river_mask`) staying as it was after an edit.
`refresh_climate` re-derives drainage, not the vector network, and neither
does the reference's own post-edit tail (`computeFlow(true);
refreshClimate();`). Re-extracting rivers after a sculpt stroke would be a
*new* behaviour with no reference precedent, and is a separate question from
wiring a UI to what already runs.

---

## 22 · The phone pass on the civ / urban / render windows (2026-08-24)

Four subsystems landed on desktop and tablet this session with no phone pass:
the civ-interaction popups, the City Viewer, the unified Sculpt/Paint/Measure
bar, and the NPR Painter block. Driving them on a real OnePlus 6T found one
structural fault under all of them and four smaller ones. `CHANGELOG.md`'s
entry of the same date carries the full reasoning; these are the register rows.

### PH-01 · The phone chrome swallowed every tap on the map — **fixed**

`_phone_content_gap` was `MOUSE_FILTER_PASS` and the two containers above it
were `STOP` by default; all three cover the whole screen. A `PASS` control is
still picked and forwards to its **parent**, not to what is behind it, so
`map_overlay.gd`'s `_gui_input()` had never run on a phone. Dead by touch:
tap-to-select a settlement, and every registered tool click/drag/release
handler — Settlement, Territory, Way, Route, Measure, Sculpt and Paint dabs.
Camera pan and pinch masked it, because those come through
`ViewportHost._input()`, which never consults a `mouse_filter`.

Recorded as a class, not just an instance: **a full-screen layout container in
the phone chrome must be `MOUSE_FILTER_IGNORE`.** Its children are picked
independently, so nothing tappable is lost.

### PH-02 · The map context menu had no touch route — **fixed**

Right-click has no finger. Press-and-hold (500 ms, under 28 px drift) now emits
the same `map_right_clicked`, and `civilization_workspace.gd`'s own menu is
re-presented as the phone canvas's L4 sheet rather than popped as a
pointer-sized `PopupMenu` that clips at a screen edge. The withheld-press rule
in `map_overlay.gd` is what stops the hold from also firing the armed tool.

### PH-03 · `wrap_controls` on three more windows — **fixed**

`place_editor_window.gd`, `faction_roster_window.gd` and
`city_viewer_window.gd` all shipped with `AcceptDialog`'s constructor default
left on — the third, fourth and fifth instances of §14's bug class. The roster
even carried a `max_size` whose comment describes the symptom it was treating.
Now off in the shared `DccWidgets.phone_window()`, on every platform.

### PH-04 · Desktop-pixel touch targets in every dock and window — **fixed**

`dcc_widgets.gd` is the single source of every row in the shell and is authored
in desktop pixels (`_row` 24, `slider` 14, `action` 26, `tool_button` 30x30).
`DccShell.phone_fit()` — `_phone_fit_tool_options()` generalised — now floors
them at §13's 44 dp across the dock sheets and the three windows. Four
non-obvious cases it also had to cover are listed in `CHANGELOG.md`:
`OptionButton.fit_to_longest_item`, `clip_text` on expanding buttons only,
`PopupMenu` row height, and `MOUSE_FILTER_PASS` on layout containers.

### PH-05 · A dock sheet does not scroll from a drag on its content — **fixed 2026-08-24**

The scrollbar drag worked and the category accordion worked, so no dock control
was unreachable; but a flick on the rows did nothing, which is the gesture a
phone user reaches for first.

**PH-04's `MOUSE_FILTER_PASS` on the rows was not merely insufficient — it was
a no-op.** Measured against 4.7.1 rather than read from the class reference:
`Container` already *defaults* to `MOUSE_FILTER_PASS`, not to the `Control`
default of `STOP` the fix assumed, so `dcc_widgets.gd`'s `HBoxContainer` rows
had never blocked anything. **A `Button` had.** Godot delivers a GUI event to
the picked control and then up its parents, stopping at the first `STOP`; a
`Button` is `STOP`, so a press that starts on one never reaches the
`ScrollContainer` above it, and `ScrollContainer`'s own drag-to-scroll is
driven by exactly those mouse events (emulated from touch), gated on
`DisplayServer.is_touchscreen_available()`.

`_scrolldrag_probe.gd` (new, in the Godot project beside `_shot_phone.gd`)
flicked twenty points down the open left sheet and named the control under
each. Twelve did not scroll: **nine `Button`s and three `HSlider`s, and
nothing else.** From the accordion down, the sheet is nothing but buttons —
the L2 `category()` headers, the L4 `group()` headers, every `action()` — so
the whole lower two thirds was dead to a flick while the labels above it
already scrolled. That also explains the subwindow that "worked": the place
editor's form is mostly labels and margins, not the mistaken conclusion that
the main viewport was at fault.

**Fix, in `DccShell.phone_fit()`:** a `BaseButton` is set to
`MOUSE_FILTER_PASS`, and the `ScrollContainer` gets a `scroll_deadzone` (new
`PHONE_SCROLL_DEADZONE`, 10 authored px, scaled with the subtree).

- `PASS` is safe on a button because `ScrollContainer` and `BaseButton`
  already cooperate: past the deadzone the scroll propagates
  `NOTIFICATION_SCROLL_BEGIN` and the button cancels its pending press.
- **The deadzone is load-bearing, not tidiness.** Godot's default is `0`, at
  which the ~2 px of wobble in a real thumb tap counts as a drag and silently
  eats the press — the fix would have traded "the sheet does not scroll" for
  "the buttons do not press". Measured at both settings: at 0 a 2 px jitter
  tap scrolls 1 px and fires nothing; at 10 a clean tap, a 2 px and a 6 px
  wobble all fire, and an eight-sample flick still scrolls 96 px and fires
  nothing.
- **`HSlider` is excluded on purpose**: a drag that starts on a slider means
  "move this slider", on every touch platform there is.
- **`OptionButton`, `MenuButton` and `ColorPickerButton` are excluded**, and
  not for symmetry: a control that opens a `Popup` on *press* pops it
  mid-flick and the popup then grabs the drag, so the gesture neither scrolls
  nor is undone (measured: popup open, scroll 0). Their rows still scroll
  from the label beside them.

`browse_dialog.gd`'s file rows carry the same rule inline, because they are
`PanelContainer`s with their own `gui_input` — the one shape the shared walk
deliberately does not touch.

**Verified on the OnePlus 6T**, not only in the probe: a flick starting on
*05 Volcanism & impacts* scrolls the World sheet to its foot and toggles
nothing; a tap on *06 Erosion* opens it.

### PH-06 · The New world dialog and the file browser are not phone-shaped — **fixed 2026-08-24**

Both opened at their desktop size in the middle of a 1080 x 2340 screen, with
10 px type. Both now take `DccWidgets.phone_window()`/`phone_present()` plus
`DccShell.phone_fit(self, 1.0)`, the treatment the three civ windows already
carried — so neither dialog gained a second set of phone constants.

Three things the shared treatment did not already cover, each a real
difference rather than boilerplate:

1. **New world needed a way out.** `phone_window()` goes borderless, and this
   dialog's OK button is *Create*, not Close — so on a phone it gained
   `add_cancel_button("Cancel")` and a `DccWidgets.phone_head()` in place of
   the title bar. `phone_window()` is called *before* the OK button is
   renamed, since it sets `ok_button_text` for the read-only windows it was
   written for. The browser needed neither: it already draws its own ✕ head
   and its own Cancel foot.
2. **The browser is spawned, not owned.** `_spawn()` is handed whatever node
   the caller had — usually `DccApp`, but `open_project_dialog.gd` passes
   `self`, a `Window` that answers neither `is_phone()` nor `phone_scale()`.
   New `_shell_of()` walks up to the nearest `DccShell`. Because it is also
   the first *transient* window to take this treatment, `phone_window()`'s
   rotation hook is now guarded and self-disconnecting: the lambda is created
   in a `static` function, so nothing auto-disconnects it, and a rotation
   after the dialog closed would have touched a freed object.
3. **The breadcrumb widened the window off the screen — on Android only.** A
   `Button` reports its own text as its minimum width (the hazard PH-04
   already records for the faction roster), and Android's home directory is
   `/data/data/org.cartalith.walkingskeleton/files`. Measured: the crumb row's
   minimum was **715 px inside a 393 dp window**, and it dragged the list, the
   foot and the Open button off the right edge with it. On Windows the same
   dialog fits, because `C:/Users/Vincent` does — **the desktop run is not
   evidence for this class of bug.** The crumb row is now a horizontal
   `ScrollContainer` on phone: it contributes no minimum on the axis it
   scrolls, every segment stays reachable rather than being trimmed behind an
   ellipsis, and it drag-scrolls for free off PH-05's `PASS`.

**Verified on the OnePlus 6T.** New world: full screen, phone header,
0 controls under 44 dp, the form drag-scrolls to its foot, Cancel and Create
both reachable. Browser: fits 1080 px wide with the ✕, the crumbs, the path
well, the list and the foot all inside it; the crumb row scrolls sideways
under a drag without navigating; a row tap still selects.

### Not registered, because it is not a gap

The unified tool bar. It builds through `set_tool_options()`, which already
runs the touch fit over the finished row, and the phone tool sheet already
scrolls horizontally — so its mode and tool segments are 44 dp and reachable
as built, with no change of its own. An `HFlowContainer` was tried and
reverted: inside a horizontal `ScrollContainer` it is handed unbounded width
and can never wrap.

---

## 23 · RF-01 — the CIVIL dock never rebuilt after a world generated (2026-08-24) — **FIXED**

A new class for this register, and the reason it needs its own tag: every one
of the 215 catalogued entries above is a **capability** gap — a control with no
engine behind it, disclosed and disabled on purpose. RF-01 is the opposite. The
engine was complete, the surface was finished, the data was correct, and the two
were simply never connected on the one signal that matters. Nothing in §6.11 or
§6.12 could have caught it, because every row there asks *"is the disclosed
reason accurate?"* and every disclosed reason here was accurate — *"No
settlements — generate a world first"* is a true sentence about an empty engine.
It just kept being displayed over a world that had 233 settlements in it.

**`RF` = refresh: a wired, finished surface that never rebuilds when the world
it reads changes.** Not (A)/(B)/(C)/(D) — those classify *missing* things. This
is a bug.

### What was wrong

`app.gd:386-400` constructs every workspace once, at launch, before any world
exists: `ws.setup()` → `Workspace.setup()` → `_build()`. `CivilizationWorkspace
._build()` drew its Settlements/Population/Economy/Politics/Culture/Timeline
categories, and `_infra.setup()` drew Roads/Rivers/Ports/Trade/Logistics, all
against an engine with no `civ` in it — so all ten rendered their empty state,
correctly.

Nothing then re-ran them:

- `app.gd`'s `generation_finished` handler (415-426) writes status-bar text only.
- The only other subscribers were `world_workspace`, `cartography_workspace`,
  `right_dock`, `viewport_host` — and, inside CIVIL, **Timeline alone** (the old
  `_build_timeline()` connection at line ~980).
- `_rebuild_readouts()` existed but rebuilt `_settlements_body` and nothing else,
  and only fired on a place/roster **edit** (`_on_civ_edited`), never on a
  generate.

So a fresh generate refreshed exactly one of eleven sections. Verified live
against 40 settlements, 6 factions and a full road network: Settlements ▸ ROSTER
said *"No settlements — generate a world first"*, Politics ▸ FACTIONS said *"No
provinces"*, Roads ▸ NETWORK said *"No roads"* — while the map drew all of it
and the right dock showed it on click.

**Why it survived this long.** Any verification workflow that edited a
settlement or a faction on its way to checking something else tripped
`_on_civ_edited` → `_rebuild_readouts()`, which refilled the roster and made the
dock look alive. The bug is only visible if you generate and then touch nothing.

### The fix

Both files now split each data-backed category into `_build_*` (runs once,
claims the category body node) and `_fill_*` (re-runnable), exactly the shape
`_rebuild_timeline`/`_tl_body` already used, and both subscribe to
`generation_finished` **and** `world_loaded` — the second covering load, revert
and reopen, which had the same hole.

| Section | File | Before | After |
|---|---|---|---|
| Settlements ▸ Roster / Land sustains | `civilization_workspace.gd` | edit only | edit + generate + load |
| Population ▸ Totals | `civilization_workspace.gd` | **never** | edit + generate + load |
| Economy ▸ Trade balance | `civilization_workspace.gd` | **never** | edit + generate + load |
| Politics ▸ Factions | `civilization_workspace.gd` | **never** | edit + generate + load |
| Timeline | `civilization_workspace.gd` | generate + load | unchanged (folded into one handler) |
| Roads ▸ Network / Hand-drawn | `infrastructure_workspace.gd` | commit only | commit + generate + load |
| Ports ▸ Coastal / Sea lanes | `infrastructure_workspace.gd` | **never** | generate + load |
| Trade ▸ Flows | `infrastructure_workspace.gd` | **never** | generate + load |
| Logistics ▸ Journey planning | `infrastructure_workspace.gd` | **never** | generate + load |

Culture and Rivers are deliberately **not** in the table and hold no body field:
each writes one fixed note about a binding that does not exist (CV-02, IN-01).
A world does not change either sentence, so rebuilding them would be motion
without content.

Two incidental corrections fell out of it. `_rebuild_readouts()`'s own comment
claimed Population and Economy *"read nothing this touches"* — wrong on both:
Population sums `get_settlements()`, and SG-02's **Recompute civilisation**
routes through the same `_on_civ_edited` and rewrites exactly the trade balances
Economy reads and the provinces Politics reads. Both now refresh after a
recompute too. And `_on_world_changed()` resets `_selected_index`, which
otherwise indexed into a settlement list the new world had replaced.

### Cost — measured, not assumed

`8e666ac` (the staleness work) established a standing rule that eagerly
cascading civ recompute is too expensive to hang off an edit (~7 s/stroke), and
SG-02 kept it behind an explicit button for the same reason. That rule is about
**recompute**. This is **presentation**: rendering already-computed state into
Control nodes, calling nothing that derives anything.

Checked rather than asserted, against `lib.rs`: `get_settlements`,
`get_provinces`, `get_trade_balances`, `get_roads`, `get_sea_routes`,
`get_factions` and `route_count` are all `civ.<field>.iter().map(…).collect()`
over stored `Vec`s. The only O(grid) call in the set,
`civ_agrarian_regional_total`, is one linear pass over the already-stored
`civ.dens` / `ws.field` — no normalisation, no recompute.

**Measured in the real app** (`_civdock_shot.gd`, 384×288, 233 settlements):

```
CIVDOCK REBUILD COST 13.99 ms for all ten sections (presentation only)
```

against a **1 350 ms** generate on the same world — roughly **1 %** added to a
generate, once per generate. Nothing here is a recompute, and the staleness
rule is not weakened by it.

### Verification

`_civdock_shot.gd` / `.tscn` (untracked harness, run **windowed** — a headless
boot proves the extension loads, which is precisely what never caught this):

1. Assert the empty state IS present before generating, so the rest means
   something.
2. Generate, switch to CIVIL, **make no edit of any kind**, then read the real
   `Label`/`Button` text out of the live node tree and assert all ten sections
   have dropped the empty state and show real numbers.
3. Delete a settlement through the real `place_editor_window.place_deleted`
   path and assert the roster follows 233 → 232 — the pre-existing edit
   refresh, not regressed.
4. Generate a **second, different** world and re-run all ten, plus assert the
   roster is not still showing the first world's count. A rebuild that only
   ever runs once is the same bug with a longer fuse.

`CIVDOCK RESULT PASS`, plus a screenshot with every accordion body forced open.
One check needed relaxing and it was not a defect: seed 771155 genuinely
produces **0 coastal settlements**, so Ports ▸ Coastal correctly reads *"No
coastal settlements in this world."* rather than the N-of-M sentence — a real
readout, not an empty state. Confirmed by counting the `coastal` flag directly
rather than by loosening the assertion and hoping.

### The lesson this register should keep

**A finished surface built before its data exists is not verified by looking at
it.** The dock was screenshotted in §14's visual sweep and read line-by-line in
§6.11/§6.12, and both passed, because both looked at it *after* doing something.
The question that finds this class of bug is *"what re-runs this, and on which
signal?"* — and for eleven sections across two files the answer was "nothing".
Worth asking of every other panel built at launch.
