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
| [45](#45--mn-10-rl-01-ca-20-rf-02rf-05-fi-04--the-is-every-control-wired-sweep-2026-08-25--seven-fixed) | **The "is every control wired" sweep (2026-08-25)** — the owner's question asked of every control surface, by driving it. Seven fixed: a menu item whose handler nothing could reach (**MN-10**), a pair row that opened one side and was a visible no-op on 5 of 15 rows (**RL-01**), two Clear-alls live over an empty list (**CA-20**), the place editor and the faction roster left showing the *previous* world after a generate while still writing by index (**RF-02/RF-03**, both destructive), the first signal-**ordering** bug this register has had (**RF-04**), RF-01's fifth recurrence on a control that worked perfectly (**RF-05**), and copy pointing at a console the user has no access to (**FI-04**). Also carries the **negative results** — 89 surfaces across three worlds, 110 ranges and 11 option buttons, 148 menu items and all 35 Layers entries measured in pixels — so the next pass does not re-walk them. |
| 16-22, 27-38 | Sections added after the contents table was written; see the `## ` headings directly. **§40 narrows CV-25 and CV-26** — the military half turned out to be three unrecognised ports plus a dead `0.35` coefficient in an already-ported formula; the relations half needed a faction-to-faction edge that genuinely did not exist. **§38 is the 2026-08-25 conformance sweep** — FR-02 (selecting a faction silently *renamed* it), PE-01 (the place editor's name re-roll was a no-op on its first press), both one defect: a field that commits on `focus_exited`, torn down while focused. It also closes SH-11 (32.59 px of zoom-pivot drift, measured) and WW-13, and cleans up six pointers §37 left aimed at retired categories plus one disclosure that had lost its only caller. **§37 is the left-rail menu structure v3 pass** — what WORLD/CIVIL/CARTO became, the fifteen new IDs (CV-21…CV-26, IN-13, CA-16…CA-19, WW-14, WW-15, VA-01, VA-02) and what was wired rather than disclosed. §32 (deep zoom stopping twenty times short of the reference) is the same batch. §29 (roads drawn as chords), §30 (the map overlay rasterised in the wrong space) and §31 (the *tool* overlay with the same defect, plus four surfaces whose copy had gone stale) are the 2026-08-24 live-driving batch. |
| [25](#25--bk-01--androids-back-button-killed-the-process-unsaved-world-and-all-2026-08-24--fixed) | **BK-01 (2026-08-24)** — the highest-severity entry in this register, and the only one where a shipped control *destroyed the user's work*: Android's Back button ended the process outright, taking an unsaved generated world with it. Root cause, the navigation model that replaced it, and two related findings (BK-02 desktop close box, unfixed; BK-03 `KEYCODE_M`, a non-finding). **Fixed.** |
| [26](#26--bk-02--the-desktop-close-box-did-the-same-thing-and-the-reason-it-was-left-alone-was-answerable-2026-08-24--fixed) | **BK-02 (2026-08-24)** — BK-01's twin on the desktop: the title bar's × ended the process with an unsaved world in it. Fixed onto the *same* shared gate, with the four-branch argument for why `auto_accept_quit = false` cannot leave the app un-closeable — the objection §25 declined the fix over. **Fixed.** |
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
`WW`'s erosion-parameter rows, `SG-01`-`SG-03`, `PH-01`-`PH-10`, `SH-09`
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
| S4 | `menus.gd:338` | Tiled LOD · tile size · atlas cache — *"No tile atlas yet."* | Stale in part. Deep-zoom LOD tiling is **live and automatic** (`lod_synthesize_tile`/`lod_tile_cells`, driven by `viewport_host.gd`'s `_lod_backlog`/`MAX_LOD_TILES_PER_UPDATE`). What does not exist is §2.5's *controls* and the *persistent* atlas. | Separates the two: tiling is live, the four preference rows and the on-disk cache are not. **Re-corrected 2026-08-24**: the on-disk cache exists now (PR-10/WW-01), so the row names the *preference surface* as the remaining gap, not the cache. |
| S5 | `world_workspace.gd:292` | Finalize — *"cartalith-spatial exists standalone, unintegrated"* | Stale: `cartalith-spatial` gained real consumers on 2026-08-18 (`PassBuffer`/`StageGraph`, then LOD tiles). The bake/freeze half of the claim is still true. | Keeps the true half (nothing is written anywhere, so there is no atlas to freeze), drops the false half, cites `LOD_TILING_INTEGRATION_SCOPE.md`. **Obsolete 2026-08-24**: the disabled button and its whole disclosure are gone, replaced by the live three-row Finalize block (WW-01). |

### Borderline, deliberately not edited

- ~~`right_dock.gd:674` Region select ▸ *"the Data Manager panel to call it doesn't exist yet"* — the Data manager **window** now exists, but the Export ▸ Maps **panel** genuinely does not. The wording says "panel". Accurate as written.~~ **Superseded 2026-08-20**: the Export ▸ Maps panel was built (§14.7), so the sentence stopped being accurate and the disabled button became live. Both the tooltip and the disable are gone — RD-09 above.
- ~~`cartography_workspace.gd:277` *"no on-canvas resize handle yet for a placed icon (`icon_bridge.rs`'s own acknowledged gap)"* — `icon_resize`/`icon_hit_test` **are** exposed, so the attribution reads as more engine-blocked than it is; but `icon_bridge.rs:216` really does say *"`None` handle — no on-canvas resize-handle geometry"*, i.e. there is no `icon_handles()` to match `label_handles()`. The claim is true; only the emphasis is off. Left alone, recorded as entry **CA-05** below (an (A) item).~~ **CA-05 CLOSED 2026-08-24** — `icon_handles()` now exists and the note it flagged is gone from that file's own comment. See CA-05's row in §6.13.
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
| ED-02 | Undo history… | 174 | same | **still open, and now for a sharper reason** — the *stack* is real (`undo_stats()` reports depth, bytes, budget and the next label); what does not exist is a panel over it. Tooltip updated 2026-08-23 to say exactly that. The live depth/cost readout landed in `Preferences ▸ Memory ▸ Undo history` instead (PR-11), which is where the reference's own `#undoMem` sat | §2.2 names it in one line; **no panel design exists** | **CLOSED 2026-08-25 (§42)** — built as the *ledger* §7.1 asked for, not the five-row list. `undo::HistoryLedger` records **every** commit and reverses the ones it can, per row: a height snapshot (`▲`, revertible), a recorded-only commit (`·`, with the specific reason nothing is retained), or a floor (`◼`, a generate or a load). Reversibility is read from the live stack depth rather than stored, so the two cannot drift. `Edit ▸ Undo history…` opens it as a right-dock context, per proposal 3 |
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
| DM-04 | Export ▸ World Data | 55 | ~~no save writer~~ → ~~no route~~ | **live (2026-08-24), partly** | §2.4 | **The route is real.** The row was a `"gap"` whose stated reason — *"cartalith-io reads .zip saves but does not write them"* — had been untrue since FI-01 landed the writer on 2026-08-23, and the row outlived it; that is now fixed twice over. The pane offers the two capabilities `PARITY_AUDIT.md` §5 item 14 names and the reference puts in its header bar: the **export raster** at 2K/4K/8K with `bakeTiles` (`WorldGen::export_raster_png` → `render::bake_rect`, single `.png` or a `tile_{r}_{c}.png` grid plus `index.json`) and the **channel atlas** (`WorldGen::export_channel_atlas` → `cartalith_engine::channel_atlas`). A live `export_raster_estimate` readout shows the real `bakeDims` output size and the run's peak memory before the user commits to an 8K one. **What is still open is the archive decision the row already named**, unchanged: this route writes *loose files*, and whether "Export ▸ World Data" should additionally assemble `exportZip`'s single `.zip` (params + f32 layers + raster + atlas + features) or defer to File ▸ Save is still nobody's decision to make from inside an export task. **`layersPreviewChk` is real as of 2026-08-24** and was the last of the four header-bar controls drawn disabled: `WorldGen::export_layer_previews` writes the reference's own four PNGs -- `layers/biome.png`, `hillshade.png`, `temperature.png`, `rainfall.png` -- into a `layers/` folder beside whatever the raster run just wrote, each built from the pass the reference's own `layerBytes(mode, debug)` branch would have taken (`bake_rect` for biome; the new `render::hillshade_raster` for `renderNow`'s `mode === 'shade'` branch; the `temp`/`rain` debug rasters, which are whole-image palette replacements rather than overlays because the reference's `debugOpacity` defaults to `1`). At the **grid's** own size, as the reference's are, and generated worlds only. The `.f32` blobs themselves still belong to the archive decision above |
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
| PR-05 | Fallback when VRAM full | **done, 2026-08-20** | real | — | §2.5 | **Two of three real, the third refused.** `CPU tile pass` (default) is exactly what the engine already does whenever the GPU path is unavailable — wiring it discloses existing behaviour rather than adding any. `Fail with error` is real and lives in `EngineBridge.generate()`, because `generate_terrain` returns a world rather than a `Result` and refusing-with-a-reason is a UI act. **`Reduce working res` is refused** (`gpu_set_vram_fallback` returns `false`): nothing in this pipeline computes a stage at a reduced grid and resamples back up — LOD tile synthesis resamples an already-finished field, which is a different operation. Closes omission **O3**. **Update 2026-08-24 — the default was only *usually* graceful.** `CPU tile pass` describes what happens when the GPU path is *unavailable*; it said nothing about a GPU that accepts the work and then fails mid-run, which until now was a **panic**, i.e. a dead process, not a fallback. Two of those, one behind the other: ten `.expect`-on-readback sites in `cartalith-gpu` (every dispatch now returns `Option`, every engine call site `map` → `and_then`), and then — visible only once the first was fixed — a device that loses a `map_async` is *gone*, so the next stage died on a 32-byte uniform buffer. A readback failure now marks the live device lost and records the size against the adapter, which `device_supports_grid` reads, so `generate_terrain`'s existing filter steers the next generation away too. Measured: the 8192²-on-the-integrated-GPU run that used to kill the process now completes in 81.9 s on the CPU fallback. See `PERFORMANCE_BENCHMARKS.md` §9.2. |
| PR-06 | Anti-aliasing · anisotropy | 333 | the 2D map path doesn't sample-antialias; belongs to the 3D viewport | yes | §2.5 | (B) large — gated on the 3D viewport |
| PR-07 | Colour management | 334 | the renderer is sRGB-only | yes | §2.5 gives **three values and nothing else** | **(C)** → §7.6 |
| PR-08 | 3D viewport defaults | 335 | no 3D viewport | yes | §2.5 names four fields | (B) large — `DECISIONS.md` §4 defers 3D; `ROADMAP.md` Phase 3 |
| PR-09 | Lighting rig defaults | 336 | no lighting rig yet | **stale in flavour**: there is no *rig*, but all six fields are real and drive the current render (`TerrainAppearance::{sun_az_deg, sun_alt_deg, relief_ambient, relief_gain, relief_lights, relief_directionality}`) | §2.5 | **CLOSED 2026-08-24** with CA-01 — all six are live rows in CARTO ▸ Map view / Rendering-advanced ▸ Relief & light. They live in the map dock rather than in Preferences, which is where `DCC_SHELL_SCOPE.md` §8.6 already resolved terrain appearance to belong; Preferences keeps the *tier* those values start from |
| PR-10 | Tiled LOD · tile size · atlas cache | 338 | **corrected — S4** | yes, now | §2.5 gives four rows of values | **Half closed 2026-08-24.** §7.7's own proposal said to ship the atlas-cache row *"only when tiles are actually written to disk"*; they are now (WW-01). The cache is real, per-world, keyed by a hash of the generation parameters, rooted at `DccSettings`' existing `atlas_cache` path exactly as §7.7 item 3 asked, with a live readout (SH-07) and a Clear (PR-12). **Still open:** §7.7's *size cap in GB*, and its item 1 split — the interactive-LOD toggles into the Layers popover and tile size / LOD levels into Export ▸ Maps. The engine has `atlas_set_tile_size()` and `bake_visible()`; nothing in Preferences calls either |
| PR-11 | Memory ▸ Undo history | 339 | no undo stack | **CLOSED 2026-08-23** — a live submenu, and the one place the stack's real cost is visible: parent-row tooltip gives depth, bytes held, budget, and what one step costs *at this resolution*; the five budget rows each say how many steps that buys here; a `Clear undo history now` row frees it on demand | §2.5 gives a range and a default | ~~(C)~~ → **done, with one deliberate departure from §2.5**: the control is a **byte budget**, not a step count. One height field is 16 MB at 2048² and 256 MB at 8192², so a flat "5 deep" would commit to 1.25 GB on the largest world this shell offers. The step count (5, the reference's `MAX_UNDO`) remains the ceiling; the budget is what binds on a big world. Measured: 80 MB held at 2048², freed exactly on clear |
| PR-12 | Memory ▸ Clear caches… | 348 | no atlas or field cache exists to clear | **stale 2026-08-24** — the persistent tile atlas is a real cache now | yes | §2.5 | **DONE 2026-08-24** — a live row (`ID_PREF_CLEAR_CACHES`), reporting how many chunks went and what they had occupied. Un-finalizes as it clears: a lock protecting nothing would strand the world read-only for no reason |
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
| RD-13 | Stamp stack ▸ finalize-lock note | 731-737 | no finalize/lock state exists in this engine | **stale, closed 2026-08-24** — `FinalizeLock` exists and `sculpt_commit` is gated on it | yes | §6 | **(A) — done 2026-08-24.** `right_dock.gd`'s `_build_sculpt()` now calls `bridge.finalize_check("height_edit")` on every rebuild: the Commit button disables (`commit_btn.disabled = stamps.is_empty() or not lock_msg.is_empty()`) and, once finalized, the engine's own refusal sentence ("This world is finalized: the baked atlas is the authoritative surface, so the heightfield is read-only. Un-finalize first.") is shown as a note in the stack, replacing the placeholder text that used to claim no lock state existed. Verified live: unfinalized shows no refusal and an enabled Commit; after `bake_all`+`set_finalized(true)`, `finalize_check` returns the sentence, Commit disables, and that exact sentence appears as a Label in the dock. |

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

> **Re-parented by menu structure v3 (§37, 2026-08-24.)** The rows below still
> resolve — the file and its builders were not rewritten — but the two-button
> `GENERATION PIPELINE | SCULPT` switch is gone and the ten numbered stages are
> now L3 sections inside nine subject categories. §37 adds **WW-14** (ecology)
> and **WW-15** (coordinate system / projection).

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| WW-01 | Finalize · LOD 0–3 · bake & freeze | 290-292 | **corrected — S5** | yes, now | §5.1's dock foot, §4's tool options bar (`app.gd:316-318` carries a second copy) | **DONE 2026-08-24.** The whole bake/atlas/finalize system is built — `cartalith-spatial/src/pyramid.rs`, `cartalith-terrain`'s `add_zoom_detail`, `cartalith-io/src/atlas.rs` (a filesystem `AtlasStore` where the reference has IndexedDB), `cartalith-engine/src/bake.rs`, and `cartalith-godot/src/bake_bridge.rs` behind fourteen `#[func]`s. The dock foot takes **the canvas's three-row split** (Bake depth · Bake ALL levels & finalize · Un-finalize) exactly as §7's own note said to when this was built, plus a fourth row for Clear. 16 golden-parity tests, all matching first run. Measured: a 2048×1311 world at 1024 px tiles bakes depth 3 in 1.64 s to 85 chunks and 234 MiB, and a deep-zoom read comes back within one `rg16` LSB (7.63e-6) of live synthesis. ****Closed out 2026-08-24 (verification pass).** The tool-options bar's second copy is live: it presses the same `_on_bake_all` and takes its visible/disabled state *pushed* from `_refresh_finalize()`, the one owner, rather than recomputing it — its tooltip had gone on claiming "No bake/LOD pipeline exists yet" since the day WW-01 shipped. The same pass found and fixed a real dead end: `_refresh_finalize()` ran when the workspace was *built*, which is before any world exists, and nothing re-ran it on generation, so **"Bake ALL levels & finalize" was permanently disabled** — the only callers that would have re-enabled it were the bake and clear buttons, one of them the disabled one. `app.gd`'s `_refresh_world_dependent()` now fires on `generation_finished` and `world_loaded`. Found by pressing the real button in a windowed run, not by reading. **Still open:** the reference's own per-tile Burn-rivers/Micro-erode refinement passes, which `pyramid_tile` documents as deliberately unported |
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
| WW-13 | Paint ▸ Commit / Discard stay enabled after a commit | 911-914, `tool_bar.gd` 393 | *(none — not previously recorded)* | found 2026-08-24, **FIXED 2026-08-25** (§38) | — | **(A)**, closed — `PaintEditor::pending_stamps()` / `paint_draft_count()`, plus a cross-refresh between the dock's pair and the tool bar's chip. Both buttons gate on `paint_painted_counts()["total"]`, which is the composite of committed *and* pending cells, so after a commit they stay live with nothing left to commit or discard — "Discard draft" especially, which then reads as "remove the paint I can see" and does nothing. Wants a `paint_draft_count()` `#[func]` over `PaintEditor`'s three `PassBuffer`s (~15 lines, one `engine_bridge.gd` passthrough); deliberately left out of the WW-12 pass to keep that commit off a fourth file another session was holding. |

### 6.11 CIVIL workspace — `civilization_workspace.gd`

> **Re-parented by menu structure v3 (§37, 2026-08-24.)** Six categories plus
> INFRA's five became fourteen; Politics split into **Factions** (who the
> polities are), **Territories** (what ground they hold) and **Politics** (change
> over time), and the collapse simulator got its own **Simulation** category.
> §37 adds **CV-21**…**CV-26** and **VA-01**/**VA-02**.

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

> **Re-parented by menu structure v3 (§37, 2026-08-24.)** Roads/Ports/Trade/
> Logistics are now CIVIL ▸ **Routes & ways** / **Travel** / **Trade**; Rivers
> left for WORLD ▸ Hydrology, carrying IN-01 with it. §37 adds **IN-13**.

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| IN-01 | Rivers ▸ Hydrology | 314-319 | no `get_rivers()`; the only river output crossing the boundary is baked into the rendered raster | yes | §3 lists Rivers as one of INFRA's five subjects | (B) large — same entity gap as RD-05 |
| IN-02 | Committed manual ways never appear on the map or in a list | 20-31 (class doc), 195, 213 | `get_roads()`/`get_sea_routes()` read `civ.ways`/`civ.sea_routes` only, never `infra.ways`; `way_commit`'s own doc said the getter was out of scope | yes when written | §4.5.4's "Way inspector: waypoint list, length, grade profile, surface" | **CLOSED 2026-08-24** — see the note below the table |
| IN-03 | Way / Route ↶ ↷ (per-waypoint undo) | 232-236 (comment) | no per-waypoint undo in the engine; `InfraTools` only discards the whole draft | yes | §4.5.4 lists ↶ ↷ | (B) small |
| IN-04 | Way ▸ routing mode (freehand / snap / least-cost) | 229-231 (comment) | `infra_tools_bridge`'s own doc: *"nothing to build a 'freehand' or distinct 'snap' routing mode out of"*; snap is real but automatic | yes | §4.5.4 | **(D)** — engine truth, recorded in-file |
| IN-05 | Way types: spec says road/track/trail/bridge, engine has road/track/sea_lane/ancient | 42-49 (comment) | `parse_way_type`'s own doc calls the spec list wrong against the tested four-entry enum | yes | §4.5.4 | **(D)** — spec/engine disagreement, resolved in the engine's favour and recorded. The *names* diverge; the **drawn styles do not** — §36 measured every type this port emits against the reference's own literals and closed the gap that mattered |
| IN-06 | Route ▸ vessel / party reference in the options row | `journey_planner_view.gd` `_vessel_field`/`_mount_field`/`_build_animal_definitions` | the journey planner exported nothing past the crate boundary when written | **CLOSED where it can be, and the remainder stated in-UI (2026-08-20)**. The party form's Mount picker and its four per-species **animal definition** pickers are now library-backed (`tl_list("animal")`, custom rows tagged `· custom`), and the choice reaches the engine: `jp_compute`'s new `animal_entries` request key → `TravelLibrary::animal_overrides_selected` → `jp_plan_ex`'s resolver, so a custom entry's capacity/speed/fodder/water and its ten-row terrain table re-plan the journey. The **Vessel** picker lists every library vessel but disables the ones with no engine counterpart (`jp_ship_stats` is still a fixed built-in table — `TRAVEL_LIBRARY_SPEC.md` §6), with the reason on the item itself rather than omitted | §4.5.4 | **CLOSED (2026-08-23)** — the remainder this cell named is done. `TravelLibrary::vessel_overrides` (keyed by **name**, because `JpPlan::vessel` is a name and `jp_ship_stats` is a name lookup, so a vessel needs no `animal_species_slot` equivalent) → `travel_library::vessel_resolver_fn` → `JpVesselResolver` → `jp_calc_water_ex`, the exact sibling of the animal chain and with the same fall-back-to-the-built-in-table contract. Four of `ShipStats`' seven fields come straight off the definition; `river`/`sea` come from `modes` and `open_sea` from `water_rating == Open`, which is precisely `jp_vessel_water_block`'s own test. **The one field with no source is `invalid_water`**: §3.3 has no per-water-type blacklist, so a custom vessel is constrained by its mode and rating only, never by a named water type the way "River Barge cannot navigate River with Rapids" is — stated in the picker's own tooltip rather than papered over. The picker now enables every library vessel that validates `ok` and disables only the incomplete ones, because the resolver declines an incomplete definition rather than sailing a hull with a zero hold |
| IN-07 | Trade ▸ route assignment | 370-373 | nothing ties a trade relationship to the road or sea lane that would carry it | yes | §3 lists Trade | (B) large |
| IN-08 | **Roads, Ports, Trade and Logistics never rebuilt after a generate** | `_build()` | none — nothing disclosed this | **not a capability gap** | all four were designed *and* built; only the signal was missing | **FIXED 2026-08-24 — see §23 (RF-01)**. Roads had a partial path already (`_refresh_manual_ways`, on a way commit); the other three had none at all |
| IN-09 | **A committed Route-tool route appeared nowhere at all** — no map line, no list row | `_commit_route` (status hint: *"not shown on the map yet (no manual-route display getter…)"*) | the hint's own reason was **wrong**: `route_count()`/`route_get(i)` have existed since the Journey Planner milestone and return the whole solved polyline. Nothing on the GDScript side ever called either | the *disclosure* was accurate (the route really was invisible), the *reason* was not | §4.5.4's Route tool; the reference draws committed journeys as their own pass (`drawCivLayer` block 2b, lines 15552-15560) | **FIXED 2026-08-24 — see the note below**. Found by live verification, not by reading: the tool committed a 572 km, 506-point path with zero unreachable legs and drew none of it |
| IN-10 | **`Data ▸ Journey planner… ⇧J` did nothing visible from any domain but CIVIL** | `app.gd`'s `open_journey_planner()` | none — nothing disclosed this either | **not a capability gap** | the planner was engine-complete and the takeover painted correctly; only the domain was wrong | **FIXED 2026-08-24 — see §27**. The shell opens on WORLD; the takeover only paints in CIVIL. Two of the three entry points are reachable from anywhere, and both armed a tool and changed not one pixel |
| IN-11 | **Every tool's advertised letter was bound to nothing** — `Way (W)`, `Route (⇧R)`, and eight more across four domains | `dcc_widgets.gd`'s `_tools_row` | none — the tooltip *was* the disclosure, and it was false | **not a capability gap** | every tool works when clicked | **FIXED 2026-08-24 — see §27**. A `Shortcut` per button, parsed from the label the tooltip already shows; `BaseButton::shortcut_input`'s visibility rule gives cross-domain inertness for free |

> **IN-09 CLOSED (2026-08-24).** Found while auditing the whole manual
> map-authoring toolset live (assets · labels · routes · POI · settlements),
> and it is IN-02's failure mode exactly — *the engine does the right thing
> and nothing renders it* — one list over. That makes it the third of this
> shape in this register after IN-02 (ways) and WW-12 (painting), which is
> now enough of a pattern to state as a rule: **a `#[func]` that returns
> geometry proves nothing about whether anything draws it. Check the pixels.**
>
> IN-02's own closing note is why this survived that pass. It reasoned —
> correctly — that a committed route does not belong in `get_roads()`/
> `get_sea_routes()`, because a route is a journey *along* geometry rather
> than durable geometry itself. Then it stopped there, without noticing that
> the conclusion left routes belonging to **no** layer whatsoever. The note's
> sentence "committed *routes* were never part of this" was true and is the
> reason nobody looked again.
>
> The reference settles the question outright: `civJourneys` gets its own
> draw pass in `drawCivLayer` (block 2b), stroked dark (`rgba(40,25,5,.5)`,
> width 3) then dashed amber (`rgba(200,160,60,.85)`, width 1.5,
> `setLineDash([5,3])`), *and* its own list with a per-row delete button
> (line 17250) and a "No journeys yet. Draw one with the **Route** tool and
> press Esc to commit." empty state (line 17233). So drawing a route is a
> port, not an addition. `map_overlay.gd` carries that pass in
> `_manual_routes`/`_draw_manual_route_segment` with the reference's own
> colours and widths, honouring `brks` the way the sea-lane pass already
> does; `ViewportHost.manual_routes()` owns the `route_count`/`route_get`
> loop and `refresh()` pushes it, so a regenerate clears the old world's
> routes rather than leaving them over the new one.
>
> **Not closed with it, and a genuine (B):** the reference's journey list can
> select, name and delete a row. This one cannot — there is no
> `route_delete`/`route_set_name` `#[func]`, so the new "Routes committed
> this session" group is read-only and a route can only be cleared by
> regenerating. `map_overlay.gd` likewise has no selected-journey branch (the
> reference's brighter, thicker `sel` stroke), because there is no route
> selection in this shell to drive it. Both are the same shape as the
> `way_set_name`/`way_delete` gap IN-02 left open.
>
> **That half CLOSED too, later the same day (2026-08-24).** The (B) estimate
> held: two `#[func]`s and a row builder. `InfraTools::route_delete` is
> `Vec::remove` — the reference's own `civJourneys.splice(ji,1)`, not a
> tombstone — and `route_set_name` writes a new `CommittedRoute::name` field
> that `route_get` now returns. **Indices renumber**, which is stated in both
> doc comments and in `engine_bridge.gd`'s wrapper because `jp_compute`'s
> `route` key and `jp_reroute`'s `route_index` name routes by index; the
> alternative (a tombstone) would have kept those stable at the price of
> `route_count()` no longer meaning "how many routes there are", which every
> existing consumer already assumes it does.
>
> The list rows now carry the reference journey card's own three affordances
> in its own order — select glyph · name field · km · `×`
> (`_civRenderJourneyList`, line 17235) — and `map_overlay.gd` gained block
> 2b's `sel` branch verbatim (underlay width 5 instead of 3, amber
> `rgba(255,210,80,.98)` at width 2.5 instead of `rgba(200,160,60,.85)` at
> 1.5; the dash pattern is *not* selection-dependent in the reference and is
> not made so here). Two deliberate divergences, both disclosed in the code:
> the **name field renames per keystroke** (the reference's `oninput`) but
> does **not** rebuild the row, which would steal focus mid-word; and
> **deleting a lower-indexed route decrements the selection** instead of
> leaving it where it was — the reference only clears the selection when the
> index runs off the end (`if(_civSelectedJourneyIdx>=civJourneys.length)`),
> which silently moves it onto a *different* journey and would highlight the
> wrong line on the map. No planner summary on the row: that card only shows
> one for the selected journey and computes it with `_jpPlan`, which is
> `journey_planner_view.gd`'s own screen here, and duplicating it would mean
> two places computing a plan and disagreeing.
>
> **Still open after this**, and untouched by it: `jp_compute`/`jp_reroute`
> callers that cache a route index across a delete are the shell's own
> problem, and `journey_planner_view.gd`'s `_route_index` is not re-validated
> when the INFRA dock deletes a route out from under it (it re-reads
> `route_count()` when that screen is opened, so the window is small and the
> failure is a wrong selection, never a crash). And `way_set_name`/
> `way_delete` remain missing — only *routes* got theirs.

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
> **Corrected 2026-08-24 (IN-09).** The paragraph above is right that a route
> does not belong in the way layer, and wrong that it therefore needed
> nothing. Excluding routes from *these two getters* left them in *no* layer:
> a committed route drew nothing anywhere and appeared in no list, which a
> live check caught. See IN-09 above. The reasoning was sound and the
> conclusion it should have reached — "so routes need a layer of their own" —
> was simply never drawn.
>
> Still open, and not silently folded in: there is no `way_set_name` /
> `way_delete` / way-condition `#[func]`, so a committed way cannot be
> renamed, retyped or removed — the reference's way-properties editor has no
> counterpart here. §4.5.4's "grade profile / surface" half of the Way
> inspector is likewise unbacked. Those are separate (B) items, not IN-02.

### 6.13 CARTO workspace — `cartography_workspace.gd`

> **Re-parented by menu structure v3 (§37, 2026-08-24.)** Three categories plus
> RENDER's flat run of sections became ten. §37 adds **CA-16**…**CA-19**.

> **Domain merge (2026-08-20, owner instruction: "And render into carto."):**
> RENDER is no longer a rail domain — §6.14 below is now reached through
> CARTO, via `cartography_workspace.gd` composing a `RenderWorkspace`
> instance into its own dock. This also directly resolves the CA-01/RN-01
> row below: CARTO and RENDER are no longer two domains disagreeing about who
> owns `set_appearance()` — they are the same dock now.

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| CA-01 | Layer properties ▸ **LIGHT** (azimuth · elevation · strength · multidirectional) | 110-114 | `TerrainAppearance` is implemented and settable in Rust but bound to no GDExtension method | yes | §7's LIGHT group | **CLOSED 2026-08-24** — `WorldGen::{get_appearance, set_appearance, list_appearance_tunables, reset_appearance}` bind **21 scalars by name**, and `render_workspace.gd` draws them as CARTO ▸ Map view + Rendering-advanced. `DCC_CONTROL_INDEX.md` §3(g)'s "Strength" ambiguity is resolved by exposing **both** `relief_gain` and `relief_directionality` as their own rows rather than picking one. Same binding closes **PR-09** and the colour/relief half of **RN-01**. |
| CA-02 | Layer properties ▸ **FILL** (colour ramp picker, domain, range) + the **Stop editor** | 110-114 | same note | yes | §7 designs nine named ramps, a popover, and a full stop editor | **CLOSED 2026-08-24** — and it really was the renderer change this row said it was, not a binding. `render.rs` gains `ElevationRamp`/`RampStop` and a `ramp_strength` tunable: an ordered breakpoint list keyed to **relative land elevation** (0 = shoreline, 1 = the world's highest point, so a saved ramp means the same picture on a world with a different peak), sampled linearly, blended over the material colour **before the light curve** — which is the difference between a hypsometric tint over shaded relief and an elevation key pasted on top. Land only; the sea keeps `sea_color_core`'s own depth ramp. **Ships off** (`ramp_strength: 0.0`, whole stage skipped, `js_reference()` untouched), so nothing about the default look moved. Bound as `list_ramp_presets`/`get_color_ramp`/`set_color_ramp`/`load_ramp_preset`; **add, delete and reorder are all `set_color_ramp`** — the panel sends the list it wants and the engine sorts by position, so dragging a stop past its neighbour *is* the reorder. Panel: CARTO ▸ **Colour relief** — the design's nine named ramps, a live gradient bar, one row per stop (colour · position · metre readout · delete), Add stop, Reverse, and the strength slider. Verified non-headlessly at 2048×1311: all nine presets render distinct maps (mean |d| 21.0–50.7 levels), strength back to 0 returns the base at **0.0000 %**, and through the real dock a slider drag reaches the engine (0.6), Add lands a stop in the widest gap (0.39 between 0.28 and 0.50), a drag from index 7 to 0.02 lands it at index 1 **with its colour**, delete and Reverse both re-render. Seven tests. **Two of the five things this row still owed landed 2026-08-24, one commit later** — see the paragraph under this table. **Still owed**: duplicate, an absolute elevation domain, and Auto Fit / Auto Breakpoints — stated in the panel's own Still-owed block rather than left to be discovered. |
| CA-03 | Terrain sub-layer visibility (Hand-drawn hillshade / Hillshade / Colour relief) | 105-107 | terrain, hillshade and colour relief are one baked raster, so they toggle with the map | yes | §7's ten-row layer stack | (B) large — needs the single colour pass to become separable outputs |
| CA-04 | Layer opacity / blend mode / reorder | *absent* | none in this file (`layers_popover.gd` has a *debug-view* opacity slider, a different thing) | — | §6's Layers context, §7 | (B) — opacity is **wrapper** (overlays already carry alpha); blend/reorder is **large** |
| CA-05 | Icon ▸ on-canvas resize handle | 277-279 | *"no on-canvas resize handle yet… (`icon_bridge.rs`'s own acknowledged gap)"* | was true — see Now | §4.5.5 | **CLOSED (2026-08-24)** — `icon_bridge::icon_handle`/`IconEditor::handles` port the reference's `drawCivLayer` icon-handle geometry (lines 15883-15893: `hr=max(4,3.2*lsc)`, `hx=px+side/2*0.7`, `hy=py+side/2*0.7`, stored `r=hr*1.6`), transcribed the same way `label_bridge::handle_circles` was for the label's own three handles — `manual.rs` never had a home for it either, being inline canvas drawing rather than a callable reference function. `WorldGen::icon_handles(index, zoom)` returns `{"resize": {"x","y","r"}}`, the same shape `label_handles` already uses, so `tool_overlay.gd`'s existing `set_handles()` primitive needed no change. `cartography_workspace.gd` gained the one missing piece of state the engine has no reason to hold — `icon_get_selected()` (a new `#[func]`, `label_get_selected`'s own icon counterpart) plus `_on_icon_click`/`_on_icon_drag`/`_on_icon_release`, mirroring `_begin_label_handle_drag`'s pattern one handle down (no rotate/arc to capture, and `icon_resize` already commits the scale directly, unlike `label_resize_size` which only computes the value). Verified: place an icon, drag its handle, watch the sprite rescale live and the change survive a zoom/redraw. **Not folded in**: `icon_hit_test`'s own box-hit half is still unused by this file — selecting a *previously placed, now-unselected* icon by clicking its box has no GDScript wiring yet, a separate gap from the resize handle this row was about. |
| CA-06 | Label ▸ letter-spacing, anchor | 643-648 | no backing field on `MapLabel` (`label_bridge.rs`'s own "Not modelled" note) | yes | §4.5.5's tool options row lists both | (B) small |
| CA-07 | Label ▸ font (the stored CSS string doesn't render) | 643-648 | Godot has no web-font fallback chain, so only size/angle/arc/colour render | yes | §4.5.5 says "font role" — **a role, not a CSS string** | **(C)** → §7.14 |
| CA-08 | Style presets (Atlas / Parchment / Physical / Ink) + `custom — edited since preset` + Reset + Save preset | *absent* | none | — | §4's Cartography row | **CLOSED 2026-08-24** — CARTO ▸ Map style now carries the **reference's own five** (Default / Antique / Ink / Watercolor / Print, reference HTML 12850's `STYLE_PRESETS`) as absolute bundles, the `Custom — controls edited since the last preset` note, and Reset-to-quality-tier. **Save preset closed 2026-08-24, and the row with it.** `TerrainAppearance` (and `Npr`, and the new `ElevationRamp`) now derive `Serialize`/`Deserialize` — §7.15's *"the one Rust line the whole feature depends on"* — and `save_appearance_preset`/`load_appearance_preset`/`peek_appearance_preset` write a named look to its **own small JSON file** (`user://appearance_presets/<slug>.json`) rather than into the world `.zip`: a look is reusable *across* worlds, which is the whole reason to save one, and `SAVEFILE_COMPAT.md`'s format is the reference app's and shallow-merges `state`, so a block this port invented would be one more unshimmed key for that app to choke on. A loaded preset replaces the **quality tier** as the base layer (so a look saved at Ultra renders at Ultra wherever it is opened) and clears the override map, because otherwise loading a saved look would reproduce something other than the saved look. `reset_appearance()` drops all three layers. Panel: CARTO ▸ **Saved looks** (name field, Save look, picker, Load look). Verified non-headlessly: an authored look at 2048×1311 saved, the session then mangled to **99.999 %** different, and the preset loaded back at **0.0000 % moved, worst 0 levels** — including the hand-authored four-stop ramp; Reset then returned the tier's own look at 0.0000 %. `#[serde(default)]` at struct level, so a preset written before a field existed still loads. Three tests. The design's own four names are a separate question from the reference's five; the reference's won, being verifiable. |
| CA-09 | Layer list ▸ search field; footer tabs **Blocks / Verticality** | *absent* | none | — | §7 names them | **(C)** — `DCC_CONTROL_INDEX.md` marks Blocks/Verticality **uncertain**: *"undefined in the spec beyond the two words"* → §7.16 |
| CA-11 | **`hydro_wet_strength` (Wetness) renders nothing at working resolution** | *engine* | — | — | reference `wetnessR` | **CLOSED 2026-08-24 (owner-authorised retune; it moves the shipped look).** Found the day before by measurement, not by reading: the binding was correct end to end and the *stage* was invisible, and got worse as the grid got finer. Both halves of `build_hydro_wetness` had been tuned at a small grid. **(1)** The gate was a `smoothstep(0.55, 0.88, …)` over the world's own *min-max-normalized* log-flow range — but `flow / (gw*gh)` is already scale-free (it is the fraction of the map a cell drains), so re-normalizing it cost the threshold its meaning: `lo` pinned to the `1e-4` clamp floor and `hi` to the largest basin, putting the knee at ~0.8 % of map area drained, i.e. the trunk river and nothing else. Replaced with an **absolute** upstream-area gate, `6e-4 … 8e-3` — the same set of channels at any resolution. **(2)** The blur then diluted what survived: a box blur conserves the mean, so a one-cell line smeared over radius `r = gw * 0.006` loses about `1/(2r+1)` of its peak, and `r` grows with the grid (3 cells at 512 wide, 12 at 2048). The blur stays (it is what makes the halo soft); a matching **gain of `2r + 1`**, clamped, restores its peak. Measured on one generated world, 0 → 1, pixels moved: **1.216 % → 10.785 %** at 512×384, **0.184 % → 4.966 %** at 1024×768, **0.002 % → 2.589 %** at 2048×1311; at the shipped `0.38` default, **0.000 % → 1.422 %** at working resolution (worst per-channel delta 3 → 59 levels). The `6e-4/8e-3` pair was picked by sweeping: `1e-3 … 1.2e-2` left working resolution at 0.67 % and `3e-4 … 5e-3` took it to 3.4 %, which is a wet-valley wash rather than a river corridor. **Trade, stated:** the gate is absolute, so a world whose basins are all smaller than `6e-4` of the map gets no wetness — an island with no river has no river to tint. Verified non-headlessly at 2048×1311 (default → 0 moves 0.821 % of pixels, default → 1 moves 1.295 %, and the corridors read as wet valley floors along the real drainage). `hydro_wet_strength` left `every_tunable_is_load_bearing`'s exemption list, and `appearance_ab_dump.rs`'s new `hydro_wetness_visibility_by_resolution` fails if any of the three sizes goes quiet again. |
| CA-10 | Layer properties ▸ **Visualization dropdown** | *absent* here; `layers_popover.gd` covers it with 18 debug views | the popover's own footer explains the split | yes | §7 lists it; §10's popover overlaps it | **(D)** — deliberately resolved as one popover rather than two competing pickers (`layers_popover.gd:10-15`) |
| CA-12 | **The whole Icon tool is inert until an asset pack is imported** — and the app ships with none | `lib.rs`'s `icon_arm`: `if !self.has_asset_pack() { return false; }`; `cartography_workspace.gd:298/310/353` mirror the gate | *"arming a family/slot this port cannot yet draw would let a caller stamp icons with nothing to render, silently"* (`icon_arm`'s own doc comment) | **the disclosure is honest and its stated reason is now obsolete.** `map_overlay.gd`'s `_draw_manual_icons` draws every family from built-in vector shapes and never reads the pack at all — its own doc comment says so (*"No texture atlas from the asset pack is wired into Godot yet… these are honest placeholder glyphs"*). The pack path (`pack::composite_map_icons`) is the **scattered** auto-icon bake, a different feature. So there is no longer a family/slot this port cannot draw | §4.5.5; the reference has **no such gate** — `iconVariantsFor` (line 7304) returns *"pack or built-in glyphs"* and `drawIconGlyph` (7315) is the built-in vector fallback for exactly this case | **(B) small, but an owner call — not taken here.** Verified live 2026-08-24: on a freshly generated world `has_asset_pack()` is `false`, `icon_armed()` is `{}`, and clicking the map with Icon armed places nothing. Loading `cartalith-assets/tests/fixtures/reference_pack.zip` makes the same clicks place and draw three icons immediately. The fix is deleting the three-line gate plus its doc paragraph, but it reverses a written decision, so it is raised rather than done (`CLAUDE.md`: *"Do not deviate from `DECISIONS.md` silently"*) |
| CA-13 | **Region naming looked absent** (owner report) | `_build_label_panel` / `_rebuild_label_panel` | none | **not a capability gap** — it works end to end | the reference calls these *region labels*; the dock said "Placed labels" / "none placed" and no menu mentions labels at all | **FIXED 2026-08-24 — see §27**. Renamed to **Region labels**; the empty state now names the tool that ends it. A menu route is still owed and is a menu-structure change, not a wording fix |

#### CA-02a — the ramp's Ease/Step modes and per-stop alpha (2026-08-24)

**The two axes CA-02 shipped without, and both were `render.rs` rather than a
binding** — which is why they were deferred in the first place. `RampStop`
gains an `a`, and `ElevationRamp` a `RampMode` of `Linear` / `Ease` / `Step`.

- **The mode belongs to the ramp, not to a stop.** §7 draws one picker above
  the stop list, and it is the honest model: "banded" is a statement about the
  whole plate, not about one breakpoint. `Ease` is this file's own `k²(3-2k)`;
  `Step` tests `k >= 1.0` rather than returning a flat `0.0`, so a sample
  landing exactly *on* a stop takes that stop's colour and two coincident stops
  still draw the hard edge they draw under `Linear`.
- **Alpha rides the same `k` as the colour** and multiplies into
  `ramp_strength`, so an alpha-0 stop reveals the material model at that
  elevation — which is how a ramp is authored to tint only the summits.
- **Two traps, both taken.** `serde` gets `default = "one"` for the alpha, not
  `#[serde(default)]`: a look saved before the field existed described *opaque*
  stops, and `f64::default()` would load every one of them invisible. And
  `normalized` always returns a `Linear` ramp, so `set_color_ramp` and
  `load_ramp_preset` carry the mode over by hand — without that, editing one
  stop silently resets a user's Step plate to Linear.
- Bound as `list_ramp_modes`/`get_ramp_mode`/`set_ramp_mode`, behind
  `EngineBridge.ramp_mode_api` — a **third** feature flag, so an in-between
  binary loses the picker rather than failing to draw the stop list. Panel:
  a `Blend` picker above the gradient bar, and an alpha slider per stop row.
  The bar shows `Step` exactly and `Ease` **approximately** (`Gradient` offers
  cubic, not smoothstep), which the code says rather than hides.
- Ten tests. **Verified non-headlessly at 2048×1311** on a real world: the
  three modes render three distinct maps (Linear↔Step 67.4 % of pixels moved,
  mean |d| 14.1, worst 177; Linear↔Ease 41.5 %, and Step is visibly a banded
  hypsometric plate where Linear is a wash); an alpha-0 ramp at
  `ramp_strength = 1.0` returns the base at **0.0000 %**; `set_ramp_mode`
  survives `set_color_ramp` *and* `load_ramp_preset`; a bad mode name returns
  `false` and changes nothing; through the real dock an alpha drag re-renders
  (33.4 % moved) and **a colour edit afterwards leaves the alpha at 0.40**, the
  `edit_alpha = false` trap; and a saved look round-trips both axes at
  **0 moved**, with the picker following the reload rather than naming the old
  mode.

### 6.14 RENDER workspace — `render_workspace.gd` (now composed into CARTO, §6.13)

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| RN-01 | The whole domain — Terrain appearance groups | 14-15 | `render.rs`'s `TerrainAppearance` is real but unbound; until it is, Preferences ▸ Render quality is the only live control | yes | §3 gives RENDER a dock; `design/cartalith-menu-structure.md` §5b designs the full subsystem (Preset · Colour relief · Colour · Material · Relief · Detail · Atmosphere · Preview · Quality) | (B) **wrapper** — ~40 real, tested fields driving the current render, reachable through **no `#[func]` at all**. The single largest cheap surface in the shell. **PARTIALLY CLOSED (2026-08-23), see RN-02**; the colour/relief half **CLOSED 2026-08-24, see RN-03.** |
| RN-03 | The **colour/relief half** — `TerrainAppearance`'s scalar fields | was RN-01's remit | *"bound to no GDExtension method"* — true until this pass | — | §5b's Relief / Detail / Preset groups; the reference's Cartography ▸ Map view + Map style (HTML 1706-1783) | **CLOSED 2026-08-24.** Bound **by name**, not as ~20 `#[func]` pairs: `list_appearance_tunables()` publishes `(key, min, max, label)` for 21 scalars and `get_appearance`/`set_appearance`/`reset_appearance` read and write them on `set_npr`'s existing every-key-optional, returns-the-count-applied contract, so the panel builds itself from the engine's own ranges and cannot offer a value the engine will clamp. The key→field table is one `tunables!` list, and three tests hold it honest: round-trip, **no two keys aliasing one field**, and **every tunable is load-bearing** (a row that changes no pixel fails). Overrides layer *over* the quality tier rather than replacing it, so switching tier does not silently discard the user's sun azimuth. Panel: CARTO ▸ **Map view** (relief exaggeration · sun azimuth · sun elevation · relief↔biome) + **Map style** (the reference's own five presets, `Custom` note) + **Rendering — advanced** (Relief & light · The sheet · Materials, plus Reset to quality tier). Verified non-headlessly on a real world: 18 of 21 keys move the raster, all-restored returns byte-identical, `Default` reproduces the base look at **0.0000 %**, and the Reset button moves both the engine and the sliders. Of the three that did not move: `relief_lights` is live at 1 (3.81 %) and converged past 6; `splat_strength` is correctly inert with no asset pack; `hydro_wet_strength` is a **real engine defect**, registered as **CA-11**. What RN-01 still owed after this — the elevation-keyed **colour ramp** (CA-02) and saving a look (CA-08) — both closed the same day, one commit later. |
| RN-02 | The reference's **NPR block** — ten "Painter" styles, coastal wave lines, animated water, multi-sun lighting | was RN-01's remit | this half was not merely unbound, it was **unported**: `render.rs`'s own module doc listed *"the 'Painter' NPR block (watercolor/contours/ink/hachure), multi-sun hillshade"* on its Excluded list | — | `PARITY_AUDIT.md` §3.1's ~15 missing render paths | **CLOSED (2026-08-23).** The ten styles, the wave lines and the multi-sun rig are literal per-pixel ports (`render::apply_npr`/`apply_waves`/`multi_sun_from_normal`/`coast_distance`), golden-verified against the reference under Node in `tests/golden_parity_npr.rs` (37 mutants, none survived — four survived a first sweep and were killed by shaping four more fixtures onto the exact gates they hide behind, never by loosening a tolerance) and off at every default, so no shipped pixel moved. They cross the boundary through `WorldGen::get_npr`/`set_npr` and are live in `render_workspace.gd` ▸ **Painter styles** / **Water & light**. Animated water is the one member that is *not* in the raster: it is per-frame, so it is a Godot `ShaderMaterial` overlay (`water_anim_layer.gd` + `water_anim.gdshader`) over `sample_bridge.rs`'s new `waterfx` field — principled equivalence (`DECISIONS.md` §7a), not golden, and stated as such. The reference's own `GW*GH <= 400000` animation cap is deliberately **not** ported: it protects a JavaScript pixel loop that no longer exists. Verified non-headlessly on the real GPU with a per-style PNG and a per-style movement measurement, an all-off return to the byte-identical base raster, a frame-to-frame measure that is non-zero only while the water overlay is on, and one real slider drag through the dock reproducing the engine call's raster exactly — a pass that found three bugs no test could have (`npr_api` guarding on a method that was never written, so the panel silently did not build; `Npr::peak_m` never filled from `params.peak_m`; and `waterfx` intensity selecting six cells of a 512×384 world, now keyed to `river_flow_thresh` like the map's own channel tint). What RN-01 still owes is the *colour/relief* half — `TerrainAppearance`'s ~40 palette and lighting fields — which `set_npr` does not touch. |

### 6.15 Frame, viewport and phone — `dcc_shell.gd`, `viewport_host.gd`, `layers_popover.gd`

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| SH-01 | Rail expansion `›` → 200 px sub-node list | **built 2026-08-19, WITHDRAWN 2026-08-24 (§28)** — the canvas draws the rail at `width:40px` in all eight desktop artboards and never draws an expanded one; the owner reported the toggle as a defect. The `›` is kept as the chrome the canvas specifies, not an affordance | none | — | §3 names it; the canvas contradicts it | **(C)** — `DCC_CONTROL_INDEX.md`: *"Sub-node lists per domain are not enumerated in the spec; the builder has no source for them"* → §7.17, §28 |
| SH-02 | Phone: tool-sheet drag, gesture-inset handle | 1056-1058, 1099-1101 | *"the mockup pictures exactly one static sheet state; nothing here answers a drag gesture"* | yes | §13 | **(D)** — deliberate: inventing a gesture the design does not show |
| SH-03 | Phone: touch-pan-while-drawing (v2.10 `#sculptNavpad`) | 710-714 | `main.gd` carries no such handling to port forward — grepped | yes | §4.5.6 requires it | (B) small — a genuine gap for whoever wires sculpt touch input. **Narrowed 2026-08-24, not closed.** SH-14's ✋ pan-mode toggle now covers the *need* the navpad exists for — a single finger that pans instead of drawing — with one latched button rather than a velocity joystick, which is the same call the reference itself makes (`panMode` at 9623 is a pan route the joystick is not). What remains genuinely unbuilt is the joystick's own behaviour: **panning without lifting the drawing finger**, mid-stroke, which is the case `#sculptNavpad` was added for and which a mode toggle cannot serve. Whether that is worth building at all is now an open question rather than an assumed gap — reassess when the Sculpt tool has real touch usage to judge it against |
| SH-04 | Phone: battery / signal glyphs | 863-868 | checked against this Godot build's own `OS` class: no `power`/`battery` method exists | yes | §13's mockup | **(D)** — nothing real to back them cross-platform; only the clock gets real data |
| SH-05 | Layers popover: hotkey badges 1–8 | *done, 2026-08-19* | `layers_popover.gd`'s `_add_hotkey_badge`/`_register_hotkeys`/`_input` | yes | §10: *"grouped rows with hotkey badges"* | **(A)**, closed — badged the first 8 rows in `LAYER_GROUPS`' own real build order (Base/Climate/Tectonics, not the spec's SURFACE/TERRAIN FIELDS/CLIMATE, which has no matching row names — see the entry's own note); real `InputMap` actions, scoped to popover-open |
| SH-06 | Viewport ▸ `→ 1 582 m` (draft-stamp elevation under the cursor) | *baseline done, suffix genuinely blocked, 2026-08-19* | `viewport_host.gd`'s `_coords_text` | yes, corrected | §10 | **(A)** for the baseline km-E/km-N/elevation readout (built, `sample_cell`); **reclassified (B)** for the `→` draft suffix — `sample_cell` reads only `WorldState::field`, never the sculpt `PassBuffer` draft, and `build_sculpt_preview_texture` composites the draft into a colourised texture only, not a per-cell elevation `#[func]`. The register's premise that this call already existed was wrong. |
| SH-07 | Status bar ▸ `autosave` and `atlas` slots | `dcc_shell.gd:657` builds both; nothing writes them | none | — | §10's middle group | **`atlas` DONE 2026-08-24** — `app.gd`'s `refresh_atlas_status()` writes chunk count, deepest level, bytes and the finalize state, blank when nothing is baked (an empty slot is the honest reading of "no atlas"; a permanent "Atlas: empty" would spend a slot saying nothing). **`autosave` still open**, still gated on FI-03 |
| SH-09 | Layers popover: **Wind / Ocean currents are animated in the reference and were static here** | *done, 2026-08-23* | `shell/wind_fx_layer.gd`, attached from `layers_popover.gd::_attach_flow_fx` | yes | the reference's own `#windFxCanvas` particle-streak overlay (`_windFx*`, HTML lines 2113-2209) — not in any mockup | **(A)**, closed — owner-reported (*"the ocean current layer isnt animated as the HTML version is. (same for wind)"*). The static rasters were correct and are untouched; what was missing is that the reference stacks a **second**, independent overlay on those two views: 260/200 particles advected along the flow field at `0.315` cells/tick, drawn as fading streaks, respawned on leaving the map, ageing out, or (ocean only) beaching. Ported constant-for-constant. The one deliberate technique change is the trail — the reference fades a persistent canvas with `destination-out`; a per-particle history redraw reaches the same streak without a never-cleared `SubViewport` doing GPU work behind a closed layer. Nothing runs while the view is off (verified: 0.0000 frame-to-frame diff) |
| SH-08 | Menu accelerators for the disabled items (⌘S ⌘⇧S ⌘W ⌘Z ⌘⇧Z ⌘X ⌘C ⌘V ⌫ ⌘A ⌘D ⌘F ⌘⇧P) | `menus.gd` sets only `Ctrl+N`, `Ctrl+O`, `⇧A`, `⇧J` | none | — | §2's tables give every one | **(D)** — an accelerator on a permanently disabled item is dead weight; they arrive with their items |
| SH-10 | **Phone: pinch-to-zoom did nothing** | *fixed 2026-08-24* — `project.godot`, new `[input_devices]` block | n/a — previously undisclosed, because nothing looked missing | yes | §13's map is the whole screen; pinch is the only zoom affordance a phone has | **(A)**, closed. Owner-reported (*"zooming doesn't seem to work on the phone"*). Not a code gap: `viewport_host.gd:406` had always handled `InputEventMagnifyGesture` and called the same `_zoom_at()` the wheel does. Godot's Android layer only attaches its `ScaleGestureDetector` when `input_devices/pointing/android/enable_pan_and_scale_gestures` is on, and the engine default is **false**, so the event was never produced and the branch was dead on every phone. Confirmed three ways: `ProjectSettings.has_setting()` true / unset value `false` on 4.7.1; `dexdump` of the shipped APK showing `onScale`/`onScaleBegin` gating on `panningAndScalingEnabled` (and `setQuickScaleEnabled` never called, so no single-finger fallback existed either); and a real two-pointer MT-B pinch injected through AOSP `uinput` on the device — **z1.0 → z2.2** out, **z2.2 → z1.0** in, against a **control APK with the setting off that reproduces the bug exactly (z1.0, unchanged)** |
| SH-11 | **`ViewportHost._zoom_at()` pivots against the wrong origin** | found 2026-08-24 while fixing SH-10; **FIXED 2026-08-25** (§38) | n/a — previously undisclosed | — | §10's viewport is expected to zoom under the pointer | **(A)**, closed — §38. The two `_input` call sites convert; `_zoom_at()`'s own maths never needed changing. Measured **32.59 px** of drift per wheel notch before, **0.00 px** after, at three probe points. `_input()` delivers `event.position` in *viewport* coordinates, but `_camera` is a child of `ViewportHost`, so `_camera.position` is `ViewportHost`-local; `viewport_host.gd:427` subtracts one from the other, so the zoom pivot is off by `ViewportHost.global_position` — measured at **(412, 70)** on the desktop layout (left rail + menu/tab bars) from a headless `app.tscn` instantiation, not inferred. Wheel and pinch are both affected; the *pan* branch is not (a delta of two global positions is offset-invariant), and `move_view_to()`/`_update_lod()` already work in local space, so line 427 is the single inconsistent site. Barely visible on the phone (edge-to-edge map, offset ≈ 0), which is why SH-10's fix verified clean. Left for a deliberate pass: it changes desktop zoom behaviour the owner currently calls correct, and `viewport_host.gd` had concurrent work in it |
| SH-12 | **`DccWidgets.note()`'s `custom_minimum_size.x` was wider than the right dock's own documented minimum** | *fixed 2026-08-24* — `dcc_widgets.gd::note()`, `240` → `190` | disclosed only in `CHANGELOG.md`'s "Still open" (695821f), never registered — `PARITY_AUDIT.md` pass 2's **F8** | now yes, this row | `DccTheme.W_RIGHT_DOCK_MIN` (260) is the dock's documented floor | **(A)**, closed. Static per context, so it never jittered (unlike SH-11's cousin bug this same file fixed for `_field()`'s value labels) — it was simply wrong: 240 px plus `section()`'s own 26 px of margin (14 left + 12 right) is 266, and a `group()` nested one level deeper adds 10 more, so the tightest real call site (`right_dock.gd`'s Measure ▸ Actions, a note inside a group inside a section) needed 276 against a 260 px dock. The right dock could not actually be dragged to its own minimum on any context that draws a note — nearly all of them (Sample-with-no-world, River, every empty Measure mode, Region, Sculpt, Wildlife). Fixed at the shared widget (`note()` is called from 18 files, not just `right_dock.gd`), so every caller benefits; the other 17 already give it wider columns and were unaffected either way. `190` leaves 33 px of clearance in the tightest nesting for the `ScrollContainer`'s vertical scrollbar. Headless boot-check clean; **the left dock and workspace panels were the other unaudited half of F8's own "still open" note** — same shared widget, so this fix covers them too, but neither was separately measured against a documented minimum the way the right dock was |
| SH-13 | **Phone: the map could not be panned at all** | *fixed 2026-08-24* — `viewport_host.gd`, new `InputEventPanGesture` branch in `_input()` | n/a — previously undisclosed; `viewport_host.gd:407` disclosed only the *single-finger* half, and reasoned it away correctly, so nothing looked missing | yes | §4.5.1 makes Pan/zoom a global, always-available modifier; §13's map is the whole screen | **(A)**, closed. Owner-reported, asking after the reference's touch navigation (*"how to move around, snapping the view back to 100% etc."*). Pan was **MMB or Space+LMB only** — a handheld has neither, so with SH-10's pinch fixed in the same window the phone could zoom but never move. Measured before assuming: a real single-finger `adb shell input swipe` across the map changed **51 pixels**, all of them the hover cursor, and left every map pixel identical. The fix is the *other half of the gesture pair SH-10 already switched on* — `enable_pan_and_scale_gestures` gates pan **and** scale together, and `dexdump` of the shipped APK shows `GodotGestureHandler.onScroll` emitting `handlePanEvent` beside the `onScale` pair SH-10 confirmed — so no new setting, no new permission, and nothing to enable. It also matches the reference exactly: its one `touchmove` handler drives zoom about the centroid **and** pan by the centroid delta together (HTML lines 14014-14015), and gives the single finger to the tool, not the camera (*"one finger keeps painting/drawing"*, HTML line 13988), which is the same call `viewport_host.gd` had already made for its own reasons. Verified on the real device (OnePlus 6T, LineageOS/Android 15) with two-pointer MT-B drags injected through AOSP `uinput`, constant span so `ScaleGestureDetector` never fires and the pan is isolated from zoom: **finger −400 px → map −163 px**, **finger +400 px → map +163 px**, `z1.0` throughout, and the round trip returns to a **byte-identical frame (0.000 mean abs diff)** — no drift, correct direction, no zoom side-effect. Reproduced on both a 2048×1311 and a 1024×655 world. **One thing is deliberately left uncalibrated and is not a defect:** the gain. `dexdump` shows Godot's own `onScroll` divides the Android delta by **5.0** (`const/high16 0x40A00000`; `handlePanEvent`/`setPanEvent` then pass it through untouched), which predicts a 0.20× gain, but the measured gain is **0.41×** — a factor of ~2 this pass could not account for from the bytecode. A multiplier tuned to an unexplained one-device measurement is exactly what this port does not do, so the handler stays 1:1 with the platform's own delta and the discrepancy is recorded here instead. The result is usable and directionally correct; it is not yet finger-tracking, and calibrating it is a small, separate, deliberate pass — see SH-14, which is where the reference's own answer to the same problem lives |
| SH-14 | **The reference's mobile navigation cluster — the `#zoomOverlay` zoom pad, the ✦ pan toggle, `zoomReset`, and what "100%" means** | *closed 2026-08-24* — `viewport_host.gd`: `_build_navpad()` (the four-button column), `zoom_step()`, `set_pan_mode()`, a `MOUSE_BUTTON_LEFT` branch in `_input()`, and a rewritten `reset_view()`; three new glyphs in `dcc_icons.gd` | never disclosed — no register row, and no "zoom pad"/"reset view" entry before this one | yes | **now yes** — `design/Cartalith Android Phone.dc.html` supplied the language; the cluster itself was designed this pass (published canvas, "Cartalith Phone Navpad": one viewport artboard and one anatomy/states artboard) | **(D) → (A), closed.** Owner-reported alongside SH-13, and deliberately raised as a design decision rather than transliterated. **Two owner decisions, 2026-08-23, both taken:** (1) **reset means cover, not fit**, and (2) **the cluster is designed in this shell's language first**, because four floating web buttons are a mobile-web idiom §13 uses nowhere else. **What the reference does, checked line by line rather than assumed:** `zoomIn`/`zoomOut` (13464-13465) are `zoomAt(viewCenter(), 1.35)` and its inverse — the *view centre*, since a button carries no map position; `panBtn` ✋ (13963) is `panMode=!panMode`, a **latching toggle, not a press-and-hold** (the ✋ glyph is misleading, and the whole handler is that one assignment), which then routes a plain button-0 pointerdown to the pan drag (9623) and suppresses the armed tool (13924); `zoomReset` ⟳ (13466) **clears `panMode`** *and* calls **`_viewFill()`** (13294), never `resetView()` (13390) — so **"100%" in this app is the COVER scale, not scale 1**. **What was built:** one right-edge column of four 44 dp pills at the `right:14px` / 10 px-gap geometry the phone canvas's own artboard 01 already uses for a floating cluster, riding the existing `_safe_insets` so it clears the app bar, bottom bar, timeline and gesture strip with no second set of numbers; drawn glyphs (`zoom_in`, `zoom_out`, `view_fill`, and the existing `tool_pan`) rather than the reference's `+`/`−`/✋/⟳ text, because the four must read as one family and `⟳` (U+27F3) is missing from Plex Mono and its whole fallback chain anyway; the pan pill latches to **accent fill with a dark glyph**, the canvas's own on-toggle idiom. Pan mode reuses `_panning` wholesale, so the motion branch needed no change at all, and handling the press in `_input` (before GUI dispatch) is what keeps the armed tool from also seeing the finger — the reference's `!panMode` guard for free. **`reset_view()` was the larger half.** It was plain fit (`_zoom = 1`, `position = ZERO`) with **no UI caller anywhere** — dead code that ran only on generate/load, and visibly the letterboxed state the reference's v1.01 was raised to fix: measured here at 393×852 against a 2048×1311 world, the fit view is a 251 px band with **300 px of dead ground above and below it**. Now cover: `max(size.x/fit.x, size.y/fit.y)` over `overlay.displayed_rect()`, which is the reference's `_viewCoverScale` with its `max(1, …)` floor for free (this camera's `zoom == 1` is already the fit rect, not a natural pixel size). Measured after the change: **covers both axes exactly, centred, zoom 3.387**. **Two deviations, recorded not silent:** (i) the reference's `panX/panY = 0` lands the map exactly aligned on the tight axis but **asymmetrically cropped on the loose one** — an artifact of `transform-origin: 0 0` over a flex-centred wrap, not an intent (its own comment at 13290 says *"cover scale, centred"*); this centres, so the crop is even. (ii) The standing pan clamp (`_viewClampFill`, 13295) is **not** ported — it runs on every `applyView()`, so it is a change to all four pan routes rather than to reset, and it would fight `ZOOM_MIN = 0.4`, which lets this camera zoom *below* fit where the reference floors at fit. **Still open** (below). **Reachability: every touch device, not phones only** — gated on `_touch`, not `DccShell._phone`. What the reference's own `isMobile` gate is really testing is *"there is no wheel, no middle button and no space bar"*, which is as true of a tablet; and `_phone` is an **aspect-ratio** test that exists to pick a layout, which a tablet fails — taking the desktop shell, i.e. desktop chrome with no mouse, the case that needs this most. **Verified windowed at 393×852, not headless:** pad built, 4 buttons, each 44×44 at x=335 (14 px clear of the edge) stacked 10 px apart above the coordinate readout; reset zoom **3.3866811** against an independently-computed cover of **3.3866811**, `covers_x`/`covers_y`/`centred` all true; zoom in **×1.35** and out **×0.740741** exactly; a synthetic one-finger drag moves the camera **0 px with pan mode off and −120 px with it on**; a drag starting **on the pad** moves it 0 px; ⟳ restores cover **and** clears the latch. **Still open, deliberately:** the pan clamp above; and **desktop still has no `reset_view()` caller** — the navpad is touch-only by design, and adding a View-menu entry is a menu-naming decision §7's audit owns, not something to slip in here |

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
| UM-01 | **Town layouts drawn on the map at deep zoom** | `civUrbanLayoutsChk` | *partly closed, 2026-08-23; **substantially closed 2026-08-24*** — the layer draws real engine output, now including milestone 12's blocks and the lots platted in them. On by default since 2026-08-24, with `_umLayoutAlpha`'s own 24 km → 10 km crossfade; buildings and the wall circuit (13, 10) remain the ceiling |
| UM-02 | **City Viewer modal** — its own canvas, zoom/pan, legend, info panel | `cityViewerModal`, `cvCanvas`/`cvCloseBtn`/`cvLegend`/`cvInfoPanel`, `_cvDrawCity`, `_cvZoomAt` | *partly closed, 2026-08-23; **substantially closed 2026-08-24*** — `shell/city_viewer_window.gd` now draws a town plan rather than a wire diagram, and its fit is the reference's own built-mass fit at last. Same remaining engine ceiling, stated on screen |
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

> **Withdrawn 2026-08-24** — the zoom cap became `lodMaxZoom()`, the premise
> above expired with it, and the pixel gate turned out to be the wrong number
> once measured. `_umLayoutAlpha` is ported for real now; see "UM-01 — the
> owner could not see a town on the map at all" below.

### UM-01/UM-02 — what closed next, 2026-08-24

The owner asked for the viewer's rendering to be improved, against a
MapEffects-style battle-map illustration whose own caption is the brief:
*"mix up the brightness and saturation of the rooftops for a more natural
look."*

**That technique needs rooftops, and there were none** — a street graph has
nothing discrete in it to fill. So the answer was not a rendering change:
`URBAN_MORPHOLOGY_SCOPE.md` **milestone 12** (`buildBlocks`/`buildParcels`)
was ported out of order, because parcels are the smallest stage that produces
a colourable shape and every primitive they need was already golden-tested at
milestones 1-2. It was a smaller change than inventing a Voronoi subdivision
to fake the same shapes, and unlike one it is the reference's own algorithm.

- **Blocks and lots are drawn**, and the roofs carry the technique for real:
  every one a different brightness and saturation of one warm palette, from a
  stable per-parcel scalar the engine emits. Brightness up and saturation
  *down* together, because a weathered roof is both.
- **The City Viewer's fit is finally the reference's own.** It fitted the
  whole graph, and said so as a known degradation, because
  `_umDrawLayoutPreview`'s built-mass fit had no built mass to fit. Blocks are
  that mass, so the long approach roads no longer shrink a town to a speck.
- **The map palette and the shell's palette are kept apart on purpose.** Map
  content stays in its warm ink-and-parchment language and does not follow the
  light/dark theme (this register's own §6 rule, and `map_overlay.gd`'s for
  faction colour); the shell's amber `accent` appears only on annotation drawn
  *over* the map — market anchor, approach-road ends — never as map ink.

**Two measured findings changed the code**, neither predictable from reading:
a 6-town sheet redrew in 577 ms until every roof edge was folded into one
`draw_multiline` (102 ms; the viewer's 4,370-lot worst case, 46 ms), and a
dense city rendered as a black mass until the ink and ridge passes were gated
on the *measured* on-screen lot size rather than on zoom — at ~3 px a lot, the
outline is wider than the roof it surrounds.

**What is still not drawn**: buildings (milestone 13), districts and amenities
(13-14), the wall circuit and gates (10), harbour and quay (9), bridges and
fords (9), farmland (15). Still absent rather than stubbed — no dictionary key
at all.

**And two disclosures the info panel now makes in words**, because `stages`
cannot make them for itself:

1. ~~**There is no open market square.**~~ **Closed 2026-08-24.**
   `buildPlaza` (milestone 8) is ported and runs where `generate()` runs it —
   between `buildPrimaries` and `grow`, on the organic branch as well as the
   radial one. The market square is now real generated geometry: the engine
   flags the block containing `plaza.center`, plats no lots on it, and the
   bridge carries a `block_plaza` flag beside `blocks` plus the square's own
   `plaza` outline. `urban_layout_draw.gd` fills that block a shade lighter
   (the reference's own rgb(208,192,154) against rgb(182,172,148)) and strokes
   the outline over the roofs. The legend gains a "Market place" row and the
   info panel's note now *describes* the square rather than apologising for its
   absence — both conditional on the layout actually having one, since a site
   with no primary to widen gets no plaza and a swatch for a colour that is not
   on screen is a lie.
2. **A rooftop is a whole parcel, inset.** `buildBuildings` would put a
   smaller footprint inside each lot with a grammar per district and a terrain
   gate leaving some lots empty, so this town has no gaps and every roof is
   the same simple quad. This is the one place in the drawing that is ahead of
   the generator, and it is labelled as such rather than left to look finished.

### UM-01 — the owner could not see a town on the map at all, 2026-08-24

Owner report: *"I don't see the settlement rendered on the map itself, the dot
yes. But not the place."* Three defects, only one of which was the suspected
one, all measured live with `_umreveal_shot.gd` (800 km world, 440 px map area,
the deepest reachable zoom being 160 = a 5 km span).

**1 · The layer was off by default, on a row nobody would find.** The reveal
gate was working; the toggle in front of it was not reachable in practice.
`civUrbanLayoutsChk` shipped `on: false` in `cartography_workspace.gd`'s
"Visible layers" list — the CARTO rail dock — while the map canvas has its own
**Layers button**, and that popover lists *field rasters* only. Someone looking
at the map for a way to turn on town layouts opens the Layers button, does not
find it, and concludes the feature does not exist. It now defaults **on** (the
one divergence from the reference's own default, and the band below is what
makes it free: nothing is generated or drawn until the map spans under 24 km),
and the popover's footnote names town layouts among the overlays it points at.

**2 · The pixel reveal gate was measurably the wrong number.** Not unreachable
— the opposite. `URBAN_MIN_BOX_PX = 16` first fired at a **47 km** span, and
because a revealed town *replaces* its pin, the layer swapped a legible marker
for a 16 px speck two octaves before the town was worth looking at. The
reference's own band is ported now, verbatim (`UM_FADE_FAR_KM = 24`,
`UM_FADE_NEAR_KM = 10`, against `lodSpanKm()`), and `draw_layout`'s `alpha`
argument — plumbed since the layer was written and passed `1.0` ever since —
finally carries the crossfade it was built for. Measured after: α = 0.00 at
25 km, 0.03 at 23.5, 0.44 at 17.8, 0.76 at 13.3, 1.00 at 10.0 and below. The
pixel constant survives underneath as a floor, not as the gate — it is what
stops a narrow map area (a phone, or the map squeezed between two open docks)
drawing a sub-pixel town just because the *span* qualifies.

**3 · The pin ballooned into the town — and this one was visible with the layer
off, which is what the owner was actually looking at.** `_civ_zoom_k()` ported
`_civZoomK`'s `1/max(0.35, min(5, z))` including the `min(5, …)`. That cap is
free in the reference because `viewT.scale` **stays at 1 under Tiled LOD** —
its deep zoom lives in `_lodZoom`, a different number. Here `_camera_zoom` *is*
the deep zoom, so past 5 the inverse-zoom term stops cancelling and the pin,
glyph, name and label outline resume growing linearly: a 1.6x overshoot while
`ViewportHost` capped at 8.0, and **32x** once the cap became `lodMaxZoom()`.
At z=60 the pin and its label covered the entire settlement. The cap is not
ported any more; the `0.35` zoom-*out* floor is untouched.

**Alignment, checked because the owner asked and the HTML original had a bug
class here** (a "coastal" town whose layout did not touch the coast; a river
town whose streets floated off the river). Measured with `_umalign_shot.gd`,
not eyeballed: every geometry is pushed through the *same* local-metres → grid
transform `_draw_urban_layouts` draws with, then compared against
`sample_cell()`'s real water mask and Strahler order and against `roads()`'s
real way polylines. A **60 km** world is used deliberately — at 800 km the
1.7 km site box is about **one** grid cell across (1.56 km/cell) and no
displacement below a cell is measurable at all; at 60 km / 512 the box is 14.5
cells and a displacement would show. 41 settlements, 41 layouts.

- **No rotation is applied at all.** `orient` is `0.000000` on all 41. That is
  the reference's own rule (`const orient = water ? 0 : _umTerrainOrient(...)`)
  and every site in this world has real water, so the local frame is
  world-aligned and the whole rotation-displacement bug class cannot arise.
- **Rooftops: 0.00% land on a real water cell**, on every town measured (57,
  94, 101, 140, 141 and 325 rooftops). Sweeping the whole layout ±3 cells in
  half-cell steps cannot reduce that count below zero, and the minimising
  offset is exactly **(0.0, 0.0)** in every case.
- **River towns: exact.** Every drawn river vertex sits **0.71 cells** from the
  nearest real river cell centre — √2⁄2, the distance from a *cell corner* to
  the centres around it. The drawn river is the real river, vertex for vertex.
- **The coastal case: within one cell.** `Skalbjorkellwick`, a `bay` capital,
  carries a 78-vertex traced shoreline: **mean 0.75 cells (88 m), worst 1.12
  cells (131 m)** from the real coastline, where a cell is 117 m. 325 rooftops,
  none in the sea.
- **Roads connect.** Where a real way reaches the town, its approach-road ends
  land **3 m to 146 m** from the nearest real way polyline point — under 1.3
  cells. `Skalbjorkellwick` is the exception and is not a defect: the nearest
  real way is 4 km away, so its approach ends are the reference's own
  synthesised box-exit bearings with nothing to meet.

**One real gap found, and it is content rather than displacement**: three of
the four `bay` sites draw **no sea**. One box is 72.3% real ocean and draws
0.5% water; another is 32.1% real and draws 0.0%. Only the fourth gets a traced
shoreline. On the map this is invisible — the terrain raster underneath already
shows the real sea, and the town is correctly placed against it — but in the
City Viewer, which has no terrain under it, such a town shows no water at all.
That is `URBAN_MORPHOLOGY_SCOPE.md` milestone 9's ground (harbour, quay,
shoreline), recorded here rather than papered over. Related and worth knowing
when reading a world-scale screenshot: at 800 km the whole town is ~1.1 grid
cells, so its own water body is *finer than the terrain grid* and will overhang
a land cell. That is resolution, not displacement.

`URBAN_FINE_BOX_PX` (the per-roof ink outline, ridge and shadow) is confirmed
map-unreachable at that map width and that is correct, not a fourth defect: at
a 5 km span the site box tops out near 150 px, an ~11 m lot is ~1 px, and the
outline would be wider than the roof it surrounds — the measurement that put
the constant there in the first place. The fine pass belongs to the City
Viewer. On the map a town reads as a mass with its water and its approach
roads, which is what it reads as in the reference at the same span.

UM-03's `peCityPreview` (the thumbnail) is now *unblocked on the engine side*
— there is a layout worth previewing at icon size, where before there was a
wire diagram. It stays open as a UI task.

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
> - **Proposal 3's panel** — **built 2026-08-25, see §42.** It is the ledger
>   proposal 1 asks for rather than the flat list, and it took proposal 3's
>   own two recommendations: a right-dock context, not a window, and
>   Photoshop's linear default with no non-linear history. Where it goes
>   further than this section is the honest half — the ledger records
>   commits it *cannot* reverse as well, each carrying the specific reason
>   nothing is retained for it, so a history panel over one reversible
>   subsystem does not read as a history of the whole application. Proposal
>   1's per-subsystem reversal is still unbuilt; what changed is that turning
>   one on is now a row's kind changing rather than a redesign.
> - **Proposal 4's Preferences row** — shipped, and it kept this section's own
>   advice to show live memory cost. It diverged on the control's *unit*: a
>   budget in MB rather than a depth in steps, for the reason PR-11's row
>   gives. The reference's cap of 30 named here is wrong — `MAX_UNDO` is 5,
>   which is also what the shipped label says.
> - **Proposal 5's Adjust Last Operation** — untouched, still (A), still the
>   cheapest remaining win in this section. §42 did not take it: it is a
>   status-bar chip over generation *parameters*, which is a different
>   surface from the commit ledger and would have been scope creep on an
>   already-large batch.
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
   **Added 2026-08-24** — `Serialize`/`Deserialize` on `TerrainAppearance`,
   `Npr` and the new `ElevationRamp`, with `#[serde(default)]` at struct
   level so a preset written before a field existed still loads. The look
   goes to its own named JSON file rather than into the world `.zip`; the
   CA-08 row gives the reason.

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
| 8 | **CA-05** — **done 2026-08-24** | Icon **on-canvas resize handle** | `icon_resize`/`icon_hit_test` are exposed; the drag math already exists on the Label tool and can be copied. Handle geometry derives from `icon_get()`. | §4.5.5 |
| 9 | **JP-12 + JP-15** — **done 2026-08-19** | Supply-reach **per-leg bar with resupply ticks**; party-form fields showing `auto · <resolved>` | `resupply_reach` and each result's `eff` dict already carry every value. | `JOURNEY_PLANNER_SPEC.md` §5, §8 |
| 10 | **SH-05** — **done 2026-08-19** | Layers popover **hotkey badges 1–8** | The popover already enumerates every view; badges plus `InputMap` entries. | §10 |
| 11 | **SH-06** — **baseline done 2026-08-19, suffix reclassified (B)** | Viewport `4 812 km E · 1 093 km N · 1 462 m` cursor coordinates + elevation | `sample_cell` gives the committed elevation; the `→ 1 582 m` draft-stamp suffix turned out to need a new Rust entry point (`sample_cell` never reads the sculpt draft) — see the §6.15 row's own note. | §10 |
| 12 | **SH-01** — **done 2026-08-19, withdrawn 2026-08-24 (§28)** | Rail expansion showing label + subtitle at 200 px | Reused `_phone_list_row()` verbatim; §7.17 argued that reading beats the spec's unenumerated one. The canvas draws neither: the rail is 40 px in all eight desktop artboards, and the owner reported the toggle as a defect. | §3 |

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

*(CV-VS-01 has since been fixed — 2026-08-23. **JP-VS-01 too**, 2026-08-24 — §27's `open_journey_planner()` calls `select_domain("civilization")` first, which is the one-line fix its own entry below predicts. This line said "still open" until §38 checked it.)*

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

**Re-checked a second time, live, 2026-08-24** (the manual-authoring audit
that closed IN-09). Nothing has changed and nothing was reversed. The tool
options bar states it to the user in as many words — *"Settlement, Territory,
Way and Route tools are armed from the TOOLS block in the dock. POI has no
engine call (`civ_tools_bridge.rs`) and is not offered."* — which is visible
in the CIVIL workspace on any generated world. That is the honest treatment
this register asks for: omitted and *said*, not drawn and dead. One nuance
worth recording so it is not mistaken for a POI feature later: the **Icon**
tool's family vocabulary does include `"poi"` (`ManualIconFamily::Poi`, drawn
as a yellow diamond by `map_overlay.gd`), so a user can place a *marker that
looks like* a POI. It carries no record, no name, no faction and no
inspector — it is an icon, and conflating the two would misread the port's
state.

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
button. *(All three were subsequently designed and built to owner brief —
SG-02 on 2026-08-24, SG-01 and SG-03 the same day; the notes under the
register are those designs.)*

### 21.1 · The register

| Tag | What | Backed by | State | Why it is not built (or, for a closed row, where its design is) |
|---|---|---|---|---|
| **SG-01** | A **staleness indicator** — the DCC mockup's own *"downstream update: rivers · deferred"* status line, showing which stages are stale and why | `WorldGen::stale_stages()` is the read; the shell's own reserved `stale` status slot is the surface, and the Civilization dock's Recompute section carries a per-stage badge | **CLOSED 2026-08-24** | See the note below — where it lives, and the one staleness source the stage graph structurally cannot carry |
| **SG-02** | A **"Recompute now"** control for the stages a commit leaves stale — today that is always `civ` | `recompute_stale_stages()` exists and is callable; `recompute_civilisation()` is the civ half, and `civilization_workspace.gd`'s Settlements ▸ **Recompute** section is the control | **CLOSED 2026-08-24** | See the note below for the design (what is re-derived, what is preserved, what deliberately is not) and the measured cost |
| **SG-03** | **`param_set` marking the graph** — a moved dial invalidating the stage it actually affects, instead of `engine_bridge.gd`'s blanket *"a moved dial does not recompute a stage, it marks the world stale until the next full generate"* | `params::invalidates()` is the table; `set_params`/`reset_params` mark the graph from it | **CLOSED 2026-08-24** | See the note below for the table, the rule it is derived from, and the finding that the shipped World dock supersedes every mark it makes |

### SG-01, closed 2026-08-24 — where the indicator lives, and the one source the graph cannot carry

`WorldGen::stale_stages()` is the read: `{stage: {origin, reason, tiles}}`,
one entry per stale stage, `{}` for the healthy state. It is a pure query —
every `StageGraph` accessor takes `&self` — so a status bar can call it on a
clock without ever triggering work, which is what both surfaces do.

**Two surfaces, and neither is new chrome.** The shell has reserved a `stale`
status slot since `_build_status_bar()` was written, and a `stale` colour
token and an unused `DccWidgets.stale_mark()` beside it; the slot was
occupied by the last generation's *duration*, which has moved into `pass`
("generated · 3.2s") where the rest of that run's outcome already was. It
now reads `stale: climate · civ — sculpt`. The second surface is a badge
above the Civilization dock's Recompute button, which says the same thing in
that button's vocabulary ("Stale over 12 tiles — sculpt. Recompute to catch
it up." / "Up to date — nothing has changed under it since the last
recompute."). Both poll on a 1 s `Timer` rather than a signal: staleness is
produced by half a dozen unrelated `#[func]`s across three workspaces, and
six notification couplings for a plain query is the wrong trade.

**The button still does not grey itself out**, and the reason has changed.
The old note said a disabled button would report a state the user cannot
see; that objection is gone. It stays enabled because "stale" is not the
only reason to press it — a recompute is also how a user re-derives roads
and borders after an edit the engine cannot classify — and because the badge
already delivers what greying out was a proxy for: knowing in advance
whether it will do anything.

**What the stage graph structurally cannot represent, and what was done
about it.** A hand-dropped, hand-edited or deleted settlement makes roads,
territory, provinces and trade balances out of date — but those are `civ`'s
*own* outputs, and `civ` is the leaf. `mark_changed(Civ)` therefore marks
nothing stale at all (`staleness.rs`'s
`a_downstream_only_edit_recomputes_nothing_upstream_of_it`), and marking any
upstream node instead would be a lie that also drags a pointless
`refresh_climate` along. Without something, the indicator would read "up to
date" immediately after the edit `ED-03d`'s button exists for — so
`WorldGen::civ_dirty` is a plain flag set by exactly those three `#[func]`s,
cleared by `recompute_civilisation` and by `absorb`, reported as
`origin: "settlements"`, `reason: "place_edited"`, `tiles: 0`. A flag rather
than a mark, because the graph is a *dependency* structure and this is not a
dependency: it is one pass of one stage being behind another pass of the
same stage.

### SG-03, closed 2026-08-24 — the per-parameter → stage table

`params::invalidates(key) -> Option<PipelineStage>`, consulted by
`set_params` per applied key and by `reset_params` wholesale. **25 of the 81
parameters mark something; 56 mark nothing at all**, and that split is the
whole design.

**The rule, which is derived and not a judgement call.** A parameter belongs
in the table only if some function *other than* `generate_terrain` reads it —
because marking a stage stale is a promise that recomputing it will apply the
new value, and there is no live path that re-runs terrain. Exactly two
functions qualify today:

| Live consumer | Parameters | Marks | Effect |
|---|---|---|---|
| `refresh_climate` (all of what `recompute_stale` runs) | every `climate.*` row — the **climate** and **weather** groups, 20 keys — plus `peak_m`, `planet.g`, `planet.rotation_hours`, `planet.axial_tilt_deg` (24) | `Hydrology` | climate *and* civ go stale; `recompute_stale`'s gate fires and one `refresh_climate` runs |
| `compute_civilisation` via `recompute_civilisation` | `river_density` (1), through `fresh_river_order` → affordances, roads, territory | `Climate` | **only** civ goes stale; `recompute_stale` runs nothing at all, leaving `still_stale = ["civ"]` for the dock's button |
| — | the other 56: every tectonic, volcanic, crater, stream, erosion-pass and world-structure knob, plus `carve_rivers`, `use_gpu`, `sea_level` and `world` | nothing | generation-time only; the honest control is Generate |

`sea_level` and `world` are the two rows that *are* read by
`climate_params_for`/`weather_params_for` and still mark nothing:
`recompute_stale` is handed `WorldState::sea_level` (a World-Structure
archetype re-anchors it during generation, so the dial is not what the
recompute reads), and `WorldGen::recompute_params` pins `world` to the value
`absorb` snapshotted, because a moved geometry switch must not make a
recompute describe a different world.

**Why the node marked is one *above* the stage that goes stale.**
`StageGraph` has no "this stage's own inputs moved" state: `mark_changed(S)`
means *S's output changed*, which makes S's consumers stale and leaves S
itself current. So the node to mark is the one immediately upstream of the
shallowest stage the dial invalidates. For the climate half that is
`Hydrology`, which is not a fiction for the weather knobs —
`refresh_climate`'s first statement recomputes `flow_discharge` from the new
rainfall, and that *is* hydrology's output. It is one node coarser than the
truth for the few temperature-only dials (`lapse_rate`, `albedo_k`), where
discharge genuinely does not move; representing those exactly would need a
fifth, `params` source node, which is a change to the pinned four-node graph
and was not taken.

**The drift guard is mechanical, not a second list.**
`params_mapping.rs`'s `every_key_that_moves_refresh_climate_is_marked_and_no_other`
walks all 81 rows, moves each to the far end of its own range, re-runs
`refresh_climate` over a fixed height field, and asserts that "the output
moved" and "`invalidates()` returns `Hydrology`" agree. A new parameter
cannot be added without deciding this, and a wrong decision fails there
rather than in the shell. Its baseline deliberately turns `wind_manual` on
and widens the latitude band, because otherwise `wind_dir_deg` and `albedo_k`
are provably inert — true of the *default world*, false of the parameter.

**Marking only; nothing is recomputed in the setter.**
`world_workspace.gd` writes a slider's value on every drag tick, so a
recompute here would run `refresh_climate` sixty times a second.

**Finding, recorded because it bounds what SG-03 is worth today:** no
shipped GDScript path leaves one of these marks standing. Every parameter row
in `world_workspace.gd` calls `_regenerate_live()` on release — the
reference's own `tparam()` `change` behaviour, verified live in 2026-08-19's
Playwright pass — and a full `generate()` rebuilds the graph from scratch in
`absorb()`. `reset_params()` has no shell caller at all. So the table is
today a correct engine-boundary contract and a prerequisite, not a
user-visible change: what would consume it is a cheap "apply climate dials
without regenerating" path, which cannot simply be switched on, because a
full regenerate with a new `rain_k` produces different *terrain* (weather
runs inside the carve and the `evolve_cycles` loop), not merely different
rainfall. That is a parity decision, not a wiring one.

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

A second live audit later the same day added four more (`PH-07`-`PH-10`).
Every one of the four was invisible to a headless check and to the desktop
build alike, which is the standing lesson of this section rather than a new
one: three were a phone-only *scaling* rule that silently did not apply, and
the fourth a phone-only layout that had never been written.

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

### PH-07 · `phone_fit()`'s font walk cannot see a `RichTextLabel` — **fixed 2026-08-24**

The walk re-wrote a control's font size only where
`has_theme_font_size_override("font_size")` was true. **A `RichTextLabel` has
no such theme item at all.** Its sizes are five separate ones — `normal_`,
`bold_`, `italic_`, `bold_italic_` and `mono_font_size` — so every
`RichTextLabel` in a dock was skipped in silence, with no warning and no
visible failure anywhere else to hint at it.

One control was affected and it is a load-bearing one: the right dock's
**"Why here?" causal chain** (`right_dock.gd`, `normal_font_size` =
`FS_SMALL`), the block that explains why a settlement sits where it does. On a
1080 x 2340 handset it drew at a flat 11 *physical* px — measured at about a
third the height of every row above it in the same panel.

`phone_fit()` now asks for the override list per control class and scales
every name that is set. A `RichTextLabel` that overrides *nothing* is scaled
too, off the resolved theme value: it is pure text with no minimum-size floor
to catch it, so the untouched case is the same fault by another route
(`app.gd`'s credits body is the other one in this shell).

**The class, not the instance: a theme override's *name* is per control type,
and a walk that assumes one name silently skips the types that use another.**

### PH-08 · The dock TOOLS block is unlabelled marks on a touch screen — **fixed 2026-08-24**

§4.5's tool palette is `dcc_widgets.gd`'s `tool_button`: a 30 x 30 square with
a 15 px glyph, an **empty** `normal` stylebox, and the tool's name in a
tooltip. On a pointer that is a complete control — hover names it, and 30 px
is a comfortable target. On a handset it is neither half of that:

- **There is no hover, so the name is unreachable by any route.** CIVIL's
  block is seven such marks (Inspect, Measure, Region select, Settlement,
  Territory, Way, Route), WORLD's four and CARTO's five, with nothing on
  screen to tell any of them apart.
- **PH-04's floor grew the box and not its contents.** The button became
  121 physical px on the device; the glyph inside it stayed 15, about a
  millimetre on a 400 ppi panel, left-aligned in the cell with no border to
  say where the button even was.

`phone_fit()` now finishes a tool button rather than only sizing it: the glyph
is **re-rasterised from the SVG** at 0.42 of the box (which is why
`dcc_widgets.gd` stashes the glyph's *name* — `DccIcons` caches per `name@px`
and the 15 px texture cannot be grown without resampling), the `normal` state
gains a visible border, and the TOOLS block's buttons gain the caption the
tooltip can no longer deliver, stacked under the icon.

**Measured before deciding between a caption and an icon-only-but-bounded
button**, as the two are a real trade: at the phone reference the widest
caption ("Region select") asks for 112 dp and CIVIL's four-tool row for 338 dp
of a 386 dp sheet, so the labels fit — but only just, and only at today's
vocabulary. `tools_block()` therefore lays its rows out in an
`HFlowContainer`: a `BoxContainer` handed more minimum width than it has does
not clip, it **overlaps**, so a longer tool name would have put the last tool
on top of its neighbour instead of on a second line. This is not the
`HFlowContainer` the §22 note below reverted — that one was inside the tool
sheet's *horizontal* `ScrollContainer`, where a flow container is handed
unbounded width and can never wrap. A dock sheet scrolls vertically only.

Desktop is a verified no-op: same 30 x 30 buttons at the same 32 px pitch with
the same 15 px icons and no caption, in all three domains.

### PH-09 · PAINT ▸ Class collapses to its own arrow in the tool sheet — **fixed 2026-08-24**

PH-04 turned `OptionButton.fit_to_longest_item` **off** on a phone, because a
`fit_to_longest_item` control reports the width of the longest item in its
list and one 287 px vocabulary label was widening a whole 393 dp window. Down
a dock that is exactly right and invisible either way — the row is full width
and the control expands into it.

The tool-options row is not a dock. It is six controls side by side, none of
which expands, so `fit_to_longest_item = false` plus PH-04's `clip_text` left
the Class picker with **no content-derived minimum width at all**: measured at
35 px on the device, showing which class was selected nowhere.

`phone_fit()` gained a `wide` flag for the one caller whose subtree scrolls
horizontally, and `set_tool_options()` sets it. Both shrink measures are
skipped there; the picker now reports 230 dp and reads "Coastal Lowland", and
the sheet — which already scrolled sideways — is a little wider.

**The class: a "make it fit the screen" rule is wrong inside a container that
scrolls on that axis, and the two had to be told apart explicitly.**

### PH-10 · The welcome / open-project dialog was never phone-adapted — **fixed 2026-08-24**

This file wrote the *precedent* PH-06 generalised — fill the screen and let
`content_scale_factor` map the desktop composition onto the 393 dp reference —
and then never took the finished treatment. It kept the hand-rolled geometry
half and had none of the rest. Now on `DccWidgets.phone_window()` /
`phone_present()` plus `DccShell.phone_fit(self, 1.0)`, like the other two.

Three things beyond the shared treatment:

1. **The toolbar had to stack.** A search well that expands beside a
   three-chip scope row is one row too many for 393 dp: the chips' own
   minimum is ~230 dp, and an over-constrained `BoxContainer` overlaps rather
   than clipping — so the well's outlined panel drew straight over `Recent /
   All worlds / Shared`, and the `LineEdit` got the ~110 dp left over, which
   is where the reported "Search wo…" came from. Both device symptoms were
   the same fault. `phone_window()` returns a boolean for exactly this.
2. **An `AcceptDialog` sizes its content child on resize, and on nothing
   else.** Hiding the too-wide subtitle is a minimum-size change, not a
   resize, so the body kept the **497 dp** width it had been measured at with
   the subtitle still in it, inside a 393 dp window — the search well ran
   82 dp off the right edge and took the gallery tiles and the *Open selected*
   button with it. Found by dumping `get_combined_minimum_size()` down the
   tree against the window's real visible rect, and by measuring
   `new_world_dialog.gd` beside it in the same run: 377 dp, correct, which is
   what ruled out the shared `phone_present()` as the cause.
   `child_controls_changed()` is the engine's own re-measure for this, and it
   is called last in the phone fit and again at the end of every `_refresh()`,
   since the gallery it measures is rebuilt on every keystroke.
3. **The rotation relay is now the shared, self-disconnecting one.** This
   file's own hand-made `phone_insets_changed` connection re-presented the
   whole window; it now carries only what is specific to this screen (which
   head text does not fit, and how many tiles do).

### Not registered, because it is not a gap

The unified tool bar. It builds through `set_tool_options()`, which already
runs the touch fit over the finished row, and the phone tool sheet already
scrolls horizontally — so its mode and tool segments are 44 dp and reachable
as built, with no change of its own. An `HFlowContainer` was tried and
reverted: inside a horizontal `ScrollContainer` it is handed unbounded width
and can never wrap.

### PH-11 · A dock sheet remembered its scroll position across close/reopen — **fixed 2026-08-24**

Found on device: scroll a dock sheet down, close it, reopen it — it opens
still scrolled, never back at the top. Six earlier attempts this session
missed it.

**The cause was absence, not a stray override.** `_build_left_dock()` /
`_build_right_dock()` (`dcc_shell.gd`) each build their own `ScrollContainer`
via `_scroll()`, but the return value was a bare local — never kept on any
field, never touched again. `_set_sheet_open()` only ever toggled
`left_dock.visible` / `right_dock.visible`; a sheet's body is built once at
shell-build time and never torn down between opens (unlike `phone_menu.gd`'s
own sheet, which rebuilds its body and zeroes `scroll_vertical` on every
`_render()` — the working precedent this fix mirrors). So nothing anywhere
ever wrote `scroll_vertical` back to 0. The `ScrollContainer` just kept
whatever position it was left at, invisible or not.

Fixed with two new fields, `_left_dock_scroll` / `_right_dock_scroll`, set
where `_scroll()`'s return value used to be discarded, and a `_reset_dock_scroll()`
call from `_set_sheet_open()` on every open (not close — a sheet still
mid-close from a fast double-tap gets no benefit from a reset it can't see,
and open is the one transition that must be correct). The reset writes
`scroll_vertical = 0` twice — once immediately, once `call_deferred` — since a
`ScrollContainer` that was `visible = false` a moment ago has not necessarily
run its own sort/clamp pass yet.

Verified with a new `_sheetscroll_probe.gd` (`--resolution 393x852
--force-touch`, mirrors PH-05's `_scrolldrag_probe.gd`): opened each sheet,
appended a 4000 px filler control to guarantee overflow regardless of real
panel content, scrolled to the bottom (4287 / 3764 px respectively), closed,
reopened — both read back `scroll_vertical = 0`. Also confirmed on a
windowed 393x852 run without the filler, against the left dock's real WORLD
content (0 → 287 → close → reopen → 0).

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

## 24 · SB-01 — every `has_method()` guard failed silently, and a 21-commit-stale `.so` proved why (2026-08-24) — **FIXED**

**Class: not a disconnected control. A whole register's worth of controls that
*were* connected, reading as disconnected, because the engine behind them was
old.** Recorded here because this register's entire method — "open the surface,
see whether it does anything" — returns the wrong answer whenever the native
library is behind the shell, and until now nothing in the app distinguished the
two cases.

### The finding

`builds/android/Cartalith.apk`'s `lib/arm64-v8a/libcartalith_godot.so` was
sha256-identical to a **2026-08-23 14:34** build. `git log` over
`cartalith-native/crates/` since that timestamp: **25 commits**. Everything
listed below was live in the tree and dead on the handset:

| Surface | How it presented on the stale build |
|---|---|
| NPR "Painter styles" / "Water & light" (RN-01) | panel did not build at all — `npr_api` false |
| Measure ▸ Area / Radius / Cross-section (§16) | greyed; Distance and Bearing still worked, which reads as a deliberate design |
| Faction roster (§18) | opened, showed `?` for every name and `0` for every population |
| City Viewer | opened, drew nothing, and said **"no layout"** — a misleading answer, not a missing one |
| Save / Save as / Autosave / Revert (FI-01) | inert |
| Undo (Edit ▸ Undo) | inert |
| Erosion-pass parameters (§19) | absent |
| Geoid / tides / Köppen / wildlife debug views (§17) | absent |
| GeoJSON export (§20) | inert |
| Hand-drawn ways reaching the map (IN-02) | inert |
| Civ recompute | inert |
| Paint visibility (WW-12, fixed in `1099ca1`) | still invisible — the fix was in the tree, not in the binary |

**Not one of them logged anything.** Clean `logcat`, no crash, no ANR.

### Root cause of the *silence* (the part this register cares about)

`engine_bridge.gd` guards every wrapper with
`world_gen.has_method("…")` and returns a safe default on a miss — correct
behaviour, and the reason a shell can run against an older binary without
crashing. There were **200 such guards** and none of them said a word. A stale
library was therefore indistinguishable from a feature that is simply off,
which is exactly the confusion this register exists to resolve.

The City Viewer is the sharpest illustration: its own graceful-degradation
message (*"no layout"*) is a statement about the **world**, and the true
condition was a statement about the **binary**. A well-written empty state can
be worse than a blank screen when it answers the wrong question confidently.

### Fix

All 200 sites now go through `EngineBridge._has()`, which answers the same
question and `push_warning()`s on the first miss of each name:

> Cartalith: the loaded GDExtension has no WorldGen.`<name>`(). Whatever needed
> it is degraded to a safe default. This almost always means the native library
> is older than the shell (a stale libcartalith_godot.so) — rebuild and
> re-export before treating the missing feature as a bug.

Once per name (several wrappers are polled from a redraw), with
`missing_bindings()` exposing the accumulated set at runtime. `push_warning`
rather than `print` because it rides `_err_print_error`, the path this
project's `logcat` greps already target.

Verified both ways headlessly: **0** warnings against a current library across
every `_ready()` probe plus NPR, factions, undo, debug layers, urban layouts,
paint and routes; **exactly one** warning for three consecutive calls on a name
no binary exports.

### Re-verification after the rebuild

Rebuilt, re-exported (sha256-verified that the APK carries the new library) and
installed. On the handset: **zero** missing-binding warnings across boot and a
full generation; the NPR panel builds and its styles visibly re-render the map;
the erosion-pass parameters are live; the annotation/icon bindings are live.
The handset then dropped off USB, so the roster, City Viewer, paint visibility,
save/undo, debug views, GeoJSON export, ways and civ-recompute remain
**unverified on device** — the register does not upgrade them on the strength
of a desktop probe.

### What this register should take from it

**Before recording a control as disconnected, establish that the binary behind
it is current.** From now on that check is one `logcat`/console grep for
`Cartalith: the loaded GDExtension has no`, and an empty result is the
precondition for trusting anything else in a gap audit. Twelve entries above
would have been filed as regressions on the strength of a screenshot.

---

## 25 · BK-01 — Android's Back button killed the process, unsaved world and all (2026-08-24) — **FIXED**

**Class: not a disconnected control. A control that was connected to the wrong
thing, and the wrong thing destroyed the user's work.** Found on the handset,
not in a review: a tester pressed the hardware/gesture **Back** on a generated
but never-saved world, the process ended, and the world was gone. There is no
recovery path — autosave only writes beside a project that has already been
saved somewhere (`DccApp._autosave_tick()`), and this one never had been.

Severity: the highest in this register so far. Every other entry is a capability
that is missing. This one is a capability that is present and *harmful*.

### Root cause — two faults, either of which was sufficient

1. **The terminal step of the back chain was a bare `get_tree().quit()`.**
   `DccShell._notification()` already answered `NOTIFICATION_WM_GO_BACK_REQUEST`
   and already popped a phone-menu level, then a sheet, then an overlay — that
   was built with the phone menu (§15's resolution) and it worked. What it did
   once those ran out was quit, immediately. `SceneTree.quit()` does **not**
   raise `NOTIFICATION_WM_CLOSE_REQUEST`, so nothing downstream could have
   intervened even in principle: the three-button unsaved-changes prompt that
   File ▸ Close project had gained in the same session was simply never
   consulted.
2. **`quit_on_go_back = false` was set only when `_phone` was true.**
   `DccShell._compute_layout_mode()` classifies by *aspect ratio*, so every
   Android device the shell reads as a tablet — and a phone whose boot window is
   reported landscape — kept the SceneTree default, where the back request quits
   the app with no code of ours running at all. On those devices not even the
   sheet/menu popping happened.

Fault 1 is why the tester lost a world on a phone. Fault 2 is a second, wider
door onto the same outcome that nobody had opened yet.

### The navigation model, and why

Back means **leave exactly one level**, innermost first. One press, one step,
and only the last of them can end the app:

| Press lands on | What back does |
|---|---|
| a dialog or popup window | hides it — found anywhere in the tree, since a dialog is parented to whichever `Control` opened it, not to the root |
| a phone-menu level | `PhoneMenu.go_back()` — L5 → L4 → L3 → L2 → closed |
| a drawer, panel picker or dock sheet | `_close_all_phone_overlays()` |
| an armed tool | Escape's own action, then a real disarm |
| nothing, and a world exists | the **same** save/discard/cancel prompt as File ▸ Close project |
| nothing, and no world exists | quits |

Two decisions in there are worth stating, because both had a defensible
alternative:

- **Prompt, not "press back again to exit."** The double-press-with-a-toast
  pattern earns its place in an app whose back stack is one level deep, where a
  stray edge swipe is the only thing an unexpected press can be. Here back
  already walks four real levels before it can reach the exit at all, so a press
  that arrives there is a considered one — and the pattern has nowhere to draw
  its hint on the phone composition, where the status bar is parked hidden as
  the phone menu's model (§15). The prompt is the guard where there is something
  to lose; where there is not, back exits at once, which is the platform
  convention and the only way out of a full-screen app.
- **The tool step disarms unconditionally, unlike Escape.**
  `GlobalTools._measure_escape()` deliberately clears the measured chain and
  leaves Measure *armed*, which is right for a pointer user whose next action is
  another measurement. Back inheriting that made the gesture a **permanent
  no-op**: every press cleared an already-clear chain, `armed_tool` never
  reached `inspect`, and the exit was unreachable for as long as Measure was
  armed. Caught by the probe below, not by review.

### The prompt is shared, not duplicated

`DccApp.close_project()` and `DccApp._back_exhausted()` both call one
`confirm_unsaved_world()`. It keeps the close-project rule that the prompt
appears **whenever a world exists**, not only when `bridge.world_dirty` is set —
see that flag's own doc comment for what it cannot see, and why the last moment
before work is destroyed is the wrong one to under-report.

### Two measured phone-presentation traps this uncovered

The prompt is the only thing standing between a back gesture and a destroyed
world, so it has to be legible and tappable on the device, not merely present.
Both of these produced a silently 29 dp button row:

1. **`DccShell.phone_fit()` structurally cannot reach it.** It walks
   `get_children()`, and `AcceptDialog` parents its entire button bar as an
   **internal** child. Every stock OK/Cancel row in this shell is therefore
   outside every fit it performs. Elsewhere that has mattered little, because a
   window's real controls live in its content child; here the three buttons
   *are* the dialog.
2. **`Window.popup()` clears `custom_minimum_size` on those buttons.** Isolated
   in a two-node scene: the value survives `content_scale_mode`,
   `content_scale_aspect`, `content_scale_factor`, `min_size` and `max_size`,
   and reads `(0, 0)` the instant the window is shown. So the floor must be
   applied *after* the popup, and re-applied on every re-popup — which is what a
   rotation is, via `phone_window()`'s inset relay.

   A third, smaller trap on the way: `b.custom_minimum_size.y = 44` through an
   **untyped** loop element writes to a temporary copy of the vector and is
   silently lost. Typed `for b: Button` and a whole-`Vector2` assignment fix it.

### Verification

`godot-project/_backnav_probe.gd` (committed, reusable) drives the real shell
with a **really generated** world in memory and delivers the actual
`NOTIFICATION_WM_GO_BACK_REQUEST`. All checks pass in three compositions:
`393x852` (the canvas reference), `540x1170` (half the OnePlus 6T, exercising
`content_scale_factor` 1.374 and confirming the 44 dp floor holds through it)
and `1600x1000` (desktop/tablet, checked for regression of File ▸ Close project
and for fault 2).

**Not verified on the device.** The handset was `offline` to `adb` for this
entire pass — `kill-server`, `start-server` and `reconnect offline` all failed;
it needs a physical replug and re-authorisation. What a desktop probe cannot
prove is that Android *delivers* the notification at all; everything downstream
of delivery, which is where the data loss lived, is proven. Delivery itself was
verified on this handset in the §15 pass ("System back popped sheet → screen →
root without exiting, pid unchanged"), which is evidence for the mechanism but
not for this change.

### Two related findings, registered and NOT fixed *(BK-02 fixed since — §26)*

- **BK-02 (A) — the desktop window's close box has no such gate.** Nothing in
  the shell intercepts `NOTIFICATION_WM_CLOSE_REQUEST`, and `auto_accept_quit`
  is at its default, so closing the window with the title bar's × on Windows
  destroys an unsaved world exactly as the back button did on Android. Same
  class, same severity, different platform. Left alone here deliberately:
  `auto_accept_quit = false` makes the app unquittable if the prompt ever fails
  to appear, which is a worse failure than the one it fixes, and it is a desktop
  behaviour change nobody reported. `close_project()`'s gate is now shared and
  takes a continuation, so wiring it is a small change when the owner wants it.
  **Fixed the same day — see §26**, which answers the un-quittable objection
  rather than accepting it.
- **BK-03 (D) — `KEYCODE_M` does not reach Godot's shortcut path on Android.**
  Checked and closed as a non-finding: **`M` is bound to nothing, on any
  platform.** Every accelerator in `menus.gd` carries a Ctrl or Shift modifier
  (`Ctrl+N/O/S/W/Z`, `Shift+A/J/L/D`, `Ctrl+Shift+S/P`), and a search for a bare
  `KEY_<letter>` across the whole shell returns nothing. There is no on-screen
  equivalent to add because there is no desktop behaviour to mirror. Worth
  re-testing only if a bare-letter accelerator is ever introduced — at which
  point the phone needs a surface for it regardless, since handsets have no
  keyboard.

---

## 26 · BK-02 — the desktop close box did the same thing, and the reason it was left alone was answerable (2026-08-24) — **FIXED**

BK-01's own §25 registered this and declined it, for a reason worth quoting
because it was a real objection and not an excuse: `auto_accept_quit = false`
"makes the app unquittable if the prompt ever fails to appear, which is a worse
failure than the one it fixes." That is true of the naive form of the fix. It is
not true of a fix that carries its own escape hatch, which is what this is.

### The fault

Identical to BK-01, one platform over. Nothing in the shell handled
`NOTIFICATION_WM_CLOSE_REQUEST`; `SceneTree.auto_accept_quit` was at its default
`true`; so the title bar's ×, Alt+F4 and the taskbar's Close each ended the
process outright. A world generated and never saved was gone, with no prompt,
exactly as Back did on the handset — and with the same aggravation, that
autosave only writes beside a project already saved somewhere, so nothing
recovered it.

### The fix

`DccShell._ready()` sets `auto_accept_quit = false` beside the `quit_on_go_back`
line BK-01 added, and `DccShell._notification()` routes the close request to a
new `_close_requested()` hook. `DccApp` overrides that hook onto **the same
`confirm_unsaved_world()` gate** File ▸ Close project and the back gesture
already share — a third caller, not a third prompt. The only change the gate
itself needed was to return its dialog, so the caller can check that it really
went up.

Deliberately *not* routed through the back chain. Back means "leave the
innermost thing" and walks dialogs, menu levels, overlays and armed tools first;
the × means "close the application", so it goes straight to the exit gate.

One structural note that makes the wiring safe: Godot propagates a window's
close request **down its own subtree** (`Window::_propagate_window_notification`
stops at nested `Window`s). Every tool window and dialog in this shell is a
*child* of the shell, never a parent, so closing one cannot reach the shell's
handler — the main window is the only source.

### Why the app cannot be left un-closeable

The invariant `_close_requested()` keeps: **every close request either quits, or
leaves a visible prompt on screen whose three answers all resolve.** Its four
branches, in order:

1. **A visible prompt is already up** → re-raise it. Not quit: exiting on a
   double-click of × would destroy the world the first click just asked about,
   which is the bug and not a fallback.
2. **We already asked and nothing is on screen** → quit, unconditionally. This
   is the escape hatch and it covers exactly the failure the deferral named: a
   script error part-way through building the prompt, or a window that never
   shows. `_quit_asked` is set *before* the attempt, so it survives an attempt
   that dies halfway, and the next × ends the process.
3. **Nothing to lose** (`not bridge.has_world`) → quit at once.
4. Otherwise prompt, then **verify the dialog is actually visible** and quit
   immediately if it is not — so even the very first × is enough against a
   prompt that fails on its first use.

From the other side: Cancel hides the dialog, which frees it, which clears both
flags and re-arms the gate; Discard quits; Save writes and quits through the
same continuation. A *failed* save is the one path that neither quits nor
prompts — `_write_project()` does not call its continuation when the write
fails, which is correct (do not exit on a save that did not happen), and it is
not a trap either, because the flags are already cleared and the next × prompts
again.

### Verification

`_backnav_probe.gd` (the BK-01 harness, extended — `_close_box_pass()`,
`_resolve_pass()`, `_await_real_close()`) drives the real shell at `1600x1000`
with a really generated, never-saved world. All checks pass:

- the real `NOTIFICATION_WM_CLOSE_REQUEST` puts up the shared "Exit Cartalith"
  gate with Discard / Cancel / Save, and it is the object the shell tracks;
- a second request while it is up neither stacks a second prompt nor quits;
- Cancel dismisses it, clears the gate and re-arms it — a later × prompts again
  with a fresh dialog;
- both escape-hatch branches are asserted (by branch, since pressing them ends
  the harness).

Each of the three answers was then **pressed for real**, one process per answer
(`-- --resolve=discard|save|cancel`), because two of them end the process:
Discard exited; Save wrote a 420 KB `.zip` and *then* exited; Cancel left the
app running with nothing on screen.

The link a synthesised notification cannot prove — that the OS request reaches
this code at all — was closed too, and unlike BK-01's Android delivery it is
proven here: `-- --hold` boots the real shell with a real unsaved world and
waits, `WM_CLOSE` is posted to its `HWND` from outside (which is what the title
bar's × sends), and the process survives with the gate drawn over the map. The
screenshot the probe saves at that moment shows "This world has unsaved
changes. / Exit the app?" over a generated world whose status bar reads *unsaved
changes*. `cargo build -p cartalith-godot` and a headless boot are both clean.

---

## 27 · IN-10, IN-11, CA-13 — the owner could not reach the Journey Planner, the Route tool, or region naming (2026-08-24) — **FIXED**

Owner, live, using the app:

> There is no way to plan a Journey or draw a route.
>
> It isn't possible to drop a name for a region on the map as in the HTML
> version.

Both reports arrived the same day the Journey Planner reached 66/74 reference
functions and the day after IN-09 made a committed route draw. That contrast is
the finding: **none of the three capabilities was missing, and two of the three
paths to them were.** Everything below was established by driving the real
windowed shell (`_jpprobe_shot.gd`, `_labelprobe_shot.gd`, `_fixprobe_shot.gd`),
not by reading the files — reading them says all three work, which is exactly
what made this survive.

### What was actually broken

| id | Symptom the owner hit | Cause | Status |
|---|---|---|---|
| **IN-10** | `Data ▸ Journey planner… ⇧J` did **nothing visible** | the takeover only paints while CIVIL is the active domain; the shell opens on WORLD | **FIXED** |
| **IN-11** | every tool's advertised letter — `W`, `⇧R`, `L`, `B`, `S`, `T`, `V`, `M`, `R`, `I` — did nothing | no key was ever bound to any of them, anywhere | **FIXED** |
| **CA-13** | region naming looked absent | it works; nothing in the dock said the word "region" or named the tool | **FIXED (wording)** |

### IN-10 — a menu item that changed not one pixel

`journey_planner_view.gd`'s `_recompute_visibility()` requires
`app.armed_tool == "journey"` **and** `app.active_domain() == "civilization"`.
That condition is right: the view swaps the whole CIVIL region — left dock,
map, right dock, tool options bar, timeline band — and would otherwise paint
over WORLD's generation pipeline.

But `open_journey_planner()` only armed the tool. Of its three entry points,
one (the INFRA dock's Logistics button) satisfies the domain condition by being
unclickable outside CIVIL, and two do not: `Data ▸ Journey planner… ⇧J` and the
right dock's "Plan a journey" are reachable from anywhere. **The shell opens on
WORLD.** So the owner's most likely first action — launch, generate, open the
Data menu, pick the one item named after the thing they wanted — armed a tool,
printed `Journey armed — Esc to release` in ghost text in the far bottom-right
corner, and left every other pixel of the WORLD workspace exactly where it was.
Captured before the fix (`jpprobe_02_menu_fired_with_world.png`); the log line
is unambiguous:

```
AFTER MENU (domain=world): armed=journey jp_active=false center.visible=false viewport.visible=true
```

`open_journey_planner()` now calls `select_domain("civilization")` first. The
ordering is load-bearing but benign either way: the domain switch emits
`workspace_changed`, recomputes with the tool not yet armed (a no-op), then
`open()` arms and the recompute paints.

### IN-11 — ten tooltips, zero bindings

Every entry in every `tools_block` carries its key in the label: `"Way (W)"`,
`"Route (⇧R)"`, `"Label (L)"`, `"Biome paint (B)"`, `"Region select (R)"`,
and so on. A search of `shell/` for any key handling found `_unhandled_key_input`
matching Escape, Backspace and Delete, `layers_popover.gd` matching digits, and
**nothing matching a letter**. The tooltip was the whole feature — the exact
fake control this register exists to catch, sitting in the middle of the TOOLS
block in every domain since the block was built.

This is the other half of "there is no way to draw a route" that a working
Route button does not explain: the tooltip tells you to press `⇧R`, so you do,
and nothing happens.

The fix is a `Shortcut` per button in `DccWidgets._tools_row`, parsed from the
label the tooltip already shows, and it is a `Shortcut` rather than a key table
on `app.gd` for a reason that is not style: `BaseButton::shortcut_input` fires
only while the button `is_visible_in_tree()` and is enabled. Only the active
domain's panel is visible, so `W` arms Way exactly when CIVIL is showing and is
inert in WORLD — the rule we want, for free, instead of re-derived by hand. It
also runs *after* GUI input, so a focused `LineEdit` eats its own letters and
typing a settlement's name never arms a tool. `shortcut_in_tooltip` is off:
the tooltip already spells the key in the mockup's notation (`⇧R`) and Godot
would append a second, differently-spelled copy under it.

### CA-13 — region naming was never missing

The reference calls these **region labels** wherever it names them
(`FUNCTION_INDEX.md`: `_civPopulateLabelEditor` *"Build the region-label
editor"*, `_civRenderLabelList`, `clearLabels` *"Clear region labels"*), and
the port has had the whole thing since the label milestone: CARTO ▸ TOOLS ▸
Label, click empty ground, a "New label" prompt whose placeholder is literally
`Region name`, then `label_create`, drawn on the map with its three
resize/rotate/arc handles. Driven live end to end (`_labelprobe_shot.gd`): the
prompt appeared, "Vale of Ashen" was typed, it rendered on the map with handles
and the options bar read `CARTO · LABEL editing #0 Vale of Ashen ✓ Confirm`.
So this is **(c) — already possible, not found.** Two reasons it wasn't:

1. The dock section was titled **"Placed labels"** and its empty state read
   **"none placed"**. Neither says *region*, which is the word the owner was
   looking for, and neither names the tool that ends the empty state.
2. No menu anywhere mentions labels or annotation — a scan of every popup for
   `label`/`annot` returns nothing — so the only path is an unlabelled icon in
   one domain's TOOLS block, and the only thing that named it was a tooltip
   whose keyboard shortcut did not work (IN-11).

Fixed by taking the reference's own vocabulary: the section is now **Region
labels**, and the empty state says *"None yet — arm Label (L) in TOOLS above,
then click empty ground and type the region's name. Drag its handles to size,
rotate and arc it."* — the same shape as Logistics' own *"No committed routes
yet — draw one with the Route tool above"*. With IN-11 fixed, the `(L)` it
names is now real. **Not done, and stated rather than quietly skipped**: no
menu route to annotation exists, and adding one is a menu-structure change
(§13's audit territory), not a wording fix.

### What was working the whole time, and is proven to still be

Reproduced from a fresh launch with real synthesised pointer events, not by
calling handlers directly:

- CIVIL ▸ TOOLS carries Settlement, Territory, **Way** and **Route**, all
  enabled and visible; clicking Route arms it (`armed_tool = route`);
- two real clicks on the map surface reach `_route_click` and the options bar
  becomes `INFRA · ROUTE · 2 stops · ✓ Commit · Discard`;
- ✓ Commit takes `route_count()` from 0 to 1, and the Journey Planner then
  opens on `Route #0 — 506 km (mixed)` with a real route map, elevation
  profile, stage bands and stops strip (`jpprobe_06_journey_planner_open.png`).

So IN-09's own verification still holds, and "draw a route" was never broken —
only unreachable by the two means the owner had reason to try (the advertised
hotkey, and the menu that names the feature).

### A rule this adds to the register's own method

IN-09 left the rule *"a `#[func]` that returns geometry proves nothing about
whether anything draws it. Check the pixels."* This pass is the layer above it:
**a control that exists, is enabled, and works when invoked proves nothing about
whether a user can find it.** Both fixes here are one line of behaviour each,
and neither would have been found by any amount of reading — only by launching
the app, doing the obvious thing, and watching nothing happen.

### Verification

`_fixprobe_shot.gd` drives the real windowed shell with a really generated
world. **24 assertions, all passing:**

- `Data ▸ Journey planner…` fired from the launch domain now switches to CIVIL
  and paints the takeover; same from CARTO;
- in CIVIL, `W`/`⇧R`/`R`/`S`/`T`/`V`/`M` arm way/route/region/settlement/
  territory/inspect/measure; in CARTO, `L`/`I` arm label/icon; in WORLD, `B`
  arms paint;
- **cross-domain letters are inert**: `L` and `B` do nothing in CIVIL, `W`
  does nothing in CARTO, `⇧R` does nothing in WORLD — the visibility rule
  holds;
- `W` with a `LineEdit` focused types a `w` and leaves the armed tool alone;
- the CARTO dock reads "Region labels", carries the new empty state, and no
  longer says "Placed labels" (the one remaining "none placed" is the Icon
  panel's own, untouched).

One probe artifact worth recording, because it nearly produced a false bug: the
`LineEdit` check failed on the first run and passed once the synthesised
`InputEventKey` carried a `unicode`. A real keyboard always sends one, and it is
what makes `LineEdit` consume the key as text before the shortcut pass sees it.
A synthesised key event without `unicode` is not a key any keyboard produces —
assert against the event the hardware actually sends.

A headless boot of `shell/app.tscn` is clean.

---

## 28 · SH-01, IN-12, MT-01 — the rail collapsed, nothing scrolled, and the measure buttons were in the wrong order (2026-08-24) — **FIXED**

Three owner reports from one live session, against the two vendored canvases
(`design/Cartalith DCC Shell.dc.html`, `design/Cartalith Measurement
Toolbar.dc.html`) as ground truth:

> 1. the left rail is collapsible and shouldn't be
> 2. rail scrolling doesn't work properly on mouse hover
> 3. the measurement tool quick-buttons aren't in the same position as the design

The middle one turned out to be the largest: **no `ScrollContainer` anywhere in
the application could be scrolled with the wheel**, and had not been able to
since the map camera was built.

### SH-01 — the rail expansion is withdrawn

`_build_rail()` had made the mockup's head chevron a real `Button`
(§7.17's proposal, built 2026-08-19): pressing it grew the rail to
`W_RAIL_EXPANDED` (200 px) and swapped the domain column for a
`_phone_list_row()` list of each domain's sub-structure. `DCC_SHELL_SPEC.md` §3
does ask for that, which is why it was built.

**The canvas never draws it.** Every desktop artboard in both canvases opens the
rail with the same literal `width:40px;flex:none` — the dark theme, the light
theme, the tablet composition, and all three measurement states, eight in total.
There is no expanded-rail artboard to build against, and the state that got
built instead borrowed the *phone* drawer's type scale into a 200 px column:
screenshotted live, `CARTOGRAPHY` ran straight under the left dock.

Withdrawn, along with `_rail_subnodes_body`, `_rail_expanded`,
`_rail_expand_button`, `_rail_panel`, `DOMAINS[i].subnodes` and
`DccTheme.W_RAIL_EXPANDED`. **What the canvas does draw is kept**: the 29 px
head cell with its dim `›`, ruled off from the domains — a `Label` with
`MOUSE_FILTER_IGNORE` now, chrome the mockup specifies rather than an affordance
nothing behind it can honour.

**Not changed: `Window ▸ Domain rail`.** It still hides the whole region. That
is the same region toggle the other four layout regions have, reversible from
the same menu, and it is not what "collapsible" meant here — reported for the
record rather than removed on inference.

### IN-12 — one `_input` handler swallowed every wheel event in the shell

`viewport_host.gd::_input()` handled `MOUSE_BUTTON_WHEEL_UP`/`_DOWN`
**unconditionally**, with no rect test, and then called
`get_viewport().set_input_as_handled()`.

`_input` runs on *every node* for *every event* regardless of where the cursor
is, and it runs **before** GUI dispatch — which is exactly why the handler is
there (its own header records the empirical finding that `_unhandled_input`
never sees the MMB press). The hazard is the other half of the same fact: a
notch anywhere in the application zoomed the map and then cancelled GUI dispatch
for that event. So the left dock (measured live: 836 px of content in a 774 px
window), the right dock, every popover and every dialog body were unscrollable
by wheel. Reproduced at three separate hover points in the left dock, all
reading `scroll_vertical == 0` after five notches.

The fix is the guard the LMB branch already carried for the navpad, generalised:
a **press** only belongs to the camera when it lands on `ViewportHost`'s own
rect. `_input` still sees everything; it just stops claiming everything.
Releases are exempt on purpose — a pan that began on the map and ended over a
dock must still clear `_panning`, or the camera sticks to the cursor.

This is the third instance of the same pattern class this register has recorded
(`4e000a3` the phone sheet that would not flick, `695821f` the right-dock pane
that sized itself to its text): **a control that participates in input dispatch
affects nodes it does not own.** The first two were `mouse_filter`; this one is
`set_input_as_handled()`, one layer up.

### MT-01 — the measurement quick-buttons were one flat run of six

The canvas draws that row as **three button groups separated by rules**,
identically in all three of its states:

```
[Distance Bearing Area Radius] │ CROSS-SECTION [Elevation … Custom▾] │ [Δ vertical  3D distance]
```

`tool_bar.gd` flattened all six `MEASURE_MODES` into one run — Distance ·
Bearing · Area · Radius · **Cross-section · Δ vertical** — and put the channel
row behind a `Field` dropdown in the *options* bar that only appeared once
Cross-section was already armed. Every button after Radius therefore sat at the
wrong x (measured live: `Δ vertical` at x 533 against the canvas's third group),
and five of the canvas's own quick-buttons were not on the bar at all.

Rebuilt to the canvas's grouping, which is now explicit in `tool_bar.gd`
(`MEASURE_GROUP_POINT` / `MEASURE_GROUP_VERTICAL`) rather than derived from
`MEASURE_MODES`' declaration order — that const is the *engine's* list of six
readings, and the canvas's grouping is a presentation fact about this one row.

**A channel button is how the canvas arms Cross-section.** Its first group has
no Cross-section button; all three states draw exactly four. So picking a field
arms the section if it is not already, and picking another while the section is
live only swaps what the strip draws. The `Field` dropdown is gone rather than
duplicated — two controls over one static are only ever a chance to disagree.

Two canvas buttons are still not drawn, for the reasons already registered:
`Custom ▾` has no user-defined field to bind to, and `3D distance` is greyed
"3D only" in the canvas itself — this shell has no 3D view, and Δ vertical
already returns the 3D distance in the dock. Both are disclosed in the row's
trailing note on hover.

### Verification — live and windowed, because all three are invisible headless

`_railprobe_shot.gd` drives the real shell at 1600 × 900:

- **rail**: `has_method("_toggle_rail_expansion") == false`; three buttons in
  the rail, all domains; `Rect2(0, 70, 40, 804)` at window widths 1600, 1100,
  760 and 640 — it cannot be squeezed either;
- `Window ▸ Domain rail` still hides and re-shows the region;
- **wheel over the rail**: camera position and zoom both unchanged (before, it
  zoomed);
- **wheel over the left dock**: `scroll_vertical` 0 → **62** at all three hover
  points, which is the entire 836 − 774 range, including with a `Button` under
  the cursor;
- **wheel over the map still zooms**: 1.00 → 1.15, one `ZOOM_WHEEL_STEP`;
- **measure row**: `Distance 195 · Bearing 265 · Area 329 · Radius 375` │
  `CROSS-SECTION 440 · Elevation 556 · Terrain 632 · Climate 696 · Hydrology
  760 · Geology 836` │ `Δ vertical 907` — the canvas's three groups, in order;
- pressing `Climate` takes `measure_mode()` to `section` and
  `section_channel()` to `climate` in one click, and the CROSS-SECTION label
  goes accent with it (canvas state 2 against states 1 and 3).

Headless boot of `shell/app.tscn` clean.

## 29 · RD-01 — the roads curve, and the renderer was drawing their chords (2026-08-24) — **FIXED**

Owner report:

> settlement roads all render as straight lines — no organic curvature

`PARITY_AUDIT.md` pass 1 lists path smoothing as already-clean and
already-ported, and it is: `civ_smooth_path` (`_civSmoothPath`, reference line
21892) is a faithful port of `rdpSimplify(run, 1.5)` → `catmullRomSample(·, 3)`
→ `Math.round`, it runs on the live generation path
(`civ_consolidate_and_smooth_ways`, called from `compute_civilisation`), and
`golden_parity_road_consolidation.rs` asserts its exact point lists. So the
first three suspects were all wrong, and had to be eliminated one at a time.

### What was measured, in order

**The engine's ways really do curve.** A throwaway probe over a real 384×288
world (seed 483920, `generate_terrain` → `civ_hierarchical_network_topology` →
`civ_consolidate_and_smooth_ways`) measured **mean sinuosity 1.072 and ~11° of
turn per vertex** across 51 visible ways. Not straight.

**One false alarm worth recording, because it nearly became the answer.** The
first fixture placed settlements on an exact lattice, and 27 of 47 ways came
back with *precisely* zero deviation from their chord — which read as a broken
cost field. It was the fixture: an axis-aligned pair has exactly one
minimum-step 8-connected path, so it is forced straight whatever the terrain
costs. Jittering the placements off the lattice took the nearly-straight count
from 27/47 to 8/51. (`CLAUDE.md`'s "shape fixtures to reach the code", from the
other direction: a fixture can also *hide* the code by making the answer
degenerate.)

**The renderer draws every point it is given.** `map_overlay.gd`'s
`_draw_way_segment` is one `draw_polyline` over the whole run between `brks`
entries — no decimation, no endpoint-only drawing, nothing collapsing it.

**So the geometry is curved and all of it is drawn — and it still looks
straight.** Rasterising the ways directly from the engine, at 4 px per grid
cell and again at 24 px per cell, showed why: the way is a **polyline of 17-20
points whose chords are 3 grid cells long**, and at 24 px/cell those chords are
72 px of dead-straight line meeting at visible angles. The curve is real; what
reaches the screen is its chords.

### Root cause: a sampling rate calibrated for a canvas that never zoomed

`_civSmoothPath` samples the spline every **3 grid cells** and rounds each
sample to a whole cell. `rdpSimplify`'s own comment gives the units away —
*"eps in grid units (caller passes ~1 screen px)"*. The reference draws its map
at roughly one grid cell per screen pixel, so a 3-cell chord is a 3 px chord
and a ±0.5-cell rounding is ±0.5 px. Drawing `lineTo` between those points is
indistinguishable from drawing the curve.

This port's viewport is a zoomable DCC surface. A 384-cell grid fitted to the
centre panel is already ~3.6 px per cell, and `ViewportHost.ZOOM_MAX` is 8: one
grid cell can be **~29 screen px**, so the same chord is an ~87 px straight
segment. Nothing is wrong with the port; the reference's own sampling rate is
simply not a rendering resolution here.

The reference had already hit the near end of this exact problem and recorded
it. `_civSmoothPath`'s **v0.92** note is another owner report — *"roads nearly
miss settlements when zooming in"* — fixed by un-rounding a way's two
endpoints, with the reasoning verbatim: *"up to half a cell of drift that's
imperceptible at low zoom but, amplified by LOD zoom (one grid cell can span
many screen pixels), visibly..."*. It closes by saying interior points stay
rounded because *"their precision was never load-bearing"*. Under this port's
zoom, it is.

### The fix: the same curve, sampled at render density, in the boundary layer

`get_roads()` (`cartalith-godot/src/lib.rs`) now hands the renderer the way's
curve re-sampled through its own control points at `WAY_RENDER_STEP_CELLS =
0.25` cells, with `brks` remapped onto the new indices. It calls
`cartalith_civ::civ_catmull_rom_sample` — **the same one definition**, now
`pub`, not a second smoothing algorithm — so this is a refinement of the curve
the engine already computed, not a different one.

**Nothing upstream of the boundary moves.** `Way::pts` is untouched, so `km`,
`_civNetworkMetrics`, `urban_adapter::um_primary_paths` (which reads the ways
directly, not through this bridge) and every road golden-parity test all see
exactly what they saw before. Placing it in `cartalith-civ` instead would have
had to change `_civSmoothPath`'s own constants and re-baseline the goldens for
a purely presentational reason; placing it in `map_overlay.gd` would have put
geometry in GDScript against `ARCHITECTURE.md`, and that file was under
concurrent edit this round.

0.25 cells is not "as fine as possible": at `ZOOM_MAX` it is a ~7 px chord and
sub-pixel at every zoom below, which is where a finer step stops buying
anything. Each run between `brks` is re-sampled on its own — splining across a
break would draw the phantom curve through the seam the break exists to lift
the pen at — and each run's own two endpoints are re-asserted afterwards, so
v0.92's guarantee that a way meets its settlement exactly survives the
re-sample (the spline lands ~1e-16 off, below `f32`, but the invariant is
written rather than inferred).

### Verification — measured in the real shell, windowed at 1600 × 900

`_roadcurve_shot.gd` generates seed 483920 at 384×288 / 2400 km through the
real `app.tscn`, reads `get_roads()`, and drives the real camera to `ZOOM_MAX`
over a road junction. Same build, same seed, same pinned view, with and without
the re-sample:

| | before | after |
|---|---|---|
| points across 35 ways | 589 | **6,342** |
| mean chord | 2.78 cells | **0.245 cells** |
| max chord | 4.24 cells | **0.328 cells** |
| turn per vertex | 14.47° | **1.70°** |
| longest way `km` | 1243.3 | **1243.3** (unchanged) |
| longest way drawn length | 198.9 cells | 199.6 cells |

The turn figure is the point: the *total* turning is the same road, now spread
over twelve times the vertices — which is the difference between a corner and a
curve. The drawn length rises 0.35% because a polyline at 0.25-cell chords is
measuring the arc rather than cutting it; `km` is the engine's own number and
does not move.

Screenshots at `ZOOM_MAX` over the same junction, settlement pins and labels
hidden so the way itself is legible: before, the roads meet at hard angles with
straight runs between them; after, they are continuous sweeping curves.

Headless boot of `shell/app.tscn` clean. `cargo test -p cartalith-civ` 493
passing (every golden-parity suite included, all unchanged), `cargo test -p
cartalith-godot --lib` 334 passing including three new `way_render_tests`
covering the density, the `brks` remap, and empty/single-point/out-of-range
inputs.

### Still open

**Sea routes and committed Route-tool routes were deliberately left alone.**
~~`get_sea_routes()` and the manual-route list have the same shape and the same
chord problem, and the same three-line treatment would fix them — but this
round's report was about roads, and `map_overlay.gd`'s route rendering was
being edited concurrently. Registered here rather than bundled in.~~ **Closed by
§33** — the same treatment, plus two things that were *not* the same: a
committed route's `points` are indexed into by `jp_compute`, so that list had to
stay put and the re-sample became a second key; and measuring the sea lanes
turned up a NaN that this section's own fix had been shipping unnoticed.

**The long straight runs are real, and are not a defect.** Between corners the
ways stay straight because the route itself is straight there: the routing grid
is capped at 384 cells wide and the travel cost over land is piecewise-constant
within a biome (measured p10 1.0003 → p90 1.61, with the slope term contributing
a p50 of only 0.0014). A road across homogeneous flat ground *is* a straight
line. Making it meander would mean changing the cost model, which is a
`DECISIONS.md` conversation and not this fix.

## 30 · MR-01, MR-02, MR-03 — the map overlay rasterised in the wrong space, twice, and gated on a moved baseline (2026-08-24) — **FIXED**

Three owner reports against the live map:

> 1. Settlement name text goes blurry quickly and doesn't scale.
> 2. Minor settlements (villages/hamlets) are always visible instead of zoom-gated.
> 3. Routes draw slightly see-through and blurry.

| id | Symptom | Root cause | Fix |
|---|---|---|---|
| MR-01 | Settlement names blur within a notch or two of the default view, and grow with zoom instead of holding still | `draw_string`'s `font_size` is in `map_overlay.gd`'s **local** space, which `ViewportHost` then scales by `_camera.scale`. Godot re-scales a `CanvasItem`'s already-recorded draw commands rather than re-running them, so a glyph rasterised at 9 px is a 9 px bitmap stretched over 72 screen px at `ZOOM_MAX`. The `maxi(9, …)` floor (the reference's own `Math.max(9, sz+lsc)`) also defeated `_civ_zoom_k()`'s size compensation, because `sc` at this viewport is 0.63 and `radius + sc` never reaches 9 — so the label was pinned at 9 *local* px and its on-screen size became `9 × zoom` | `_crisp_begin()`/`_crisp_end()` — a `draw_set_transform` of `1/zoom` inside which every coordinate and size is a **screen** pixel. The glyph and name are measured, rasterised and drawn at their final on-screen size, so both are crisp and constant across the whole 0.4-8.0 range |
| MR-02 | Villages and 209 hamlets drawn full-size, with pins and names, on a map that had never been zoomed | Two causes, one dominant. **(a)** `lib.rs` folds `civ_seed_villages`' output into the settlement roster as plain `Hamlet`s, disclosing the choice as *"a village renders exactly like any other hamlet, which is what the reference's own hamlet-tier tagging for these already implies."* The reference does the opposite: it tags them `villageAddon` **so the renderer will not treat them as hamlets**, gates them at `CIV_VILLAGE_ADDON_LOD = 2.4` rather than `CIV_LOD_PLACE.hamlet = 1.4`, and hides them **outright** with no dot fallback — its own comment names the complaint ("waay too populated") the constant exists for. Measured: **200 of 209 hamlets are addon villages**, against 24 real settlements. And the shell defaults `villages` to `true` where the reference defaults it `false`. **(b)** `_settlement_below_lod` compared `SETTLEMENT_LOD` against the raw `_camera_zoom`, whose meaning moved on 2026-08-23 when `reset_view()` became the reference's **cover** scale: cover is `>= 1` by construction and window-shaped, so the same world opens at `z = 1.04` in one dock layout and `1.36` in another, and every threshold under 1.4 could be satisfied by the opening view alone | **(a)** `VILLAGE_ADDON_LOD = 2.4`, and an addon village below it draws nothing and is not hit-testable (the reference's `_civPlacePickVisible` excludes a still-hidden addon from picking too). **(b)** the thresholds are compared against zoom **normalised by `_lod_zoom_base()`**, re-derived from this control's own geometry, so `1.0` means "the view a world opens at" on every window shape. Measured at the default view: **33 places drawn instead of 233** |
| MR-03 | A committed route reads as a wide, translucent, blurred band rather than the reference's dark-underlay-plus-dashed-amber | Same space error as MR-01, in the other primitive. The reference multiplies **every** way and journey `lineWidth` and dash length by `rsc` (line 15470, `max(1,GW/512)*_civZoomK()*_civWayScale()`); the port dropped the term. A width fixed in local space is scaled by the camera **together with its antialiasing fringe** — at zoom 8 the 1.5 px amber dash is 12 px wide with ~8 px of soft fringe on each side, which is exactly "see-through and blurry". Ways and sea lanes had it too; routes are simply the layer whose alpha (`.5`/`.85`) makes it obvious | The three linear layers draw inside `_crisp_begin()`, so every width and dash constant in the file is now read as screen pixels and the AA fringe is generated at screen resolution. `ROAD_WIDTH_BY_TYPE`'s 1.6 is 1.6 px of road at any zoom |

**Not one common cause, but not three either.** MR-01 and MR-03 are the same
bug in two primitives — a quantity that must be screen-space computed in the
overlay's local space, which the camera transform then magnifies along with its
rasterisation. They share one fix. MR-02 is independent of both.

**Verified live, non-headlessly**, 1600 × 1000 and 2400 × 800 windows, seed
483920 over a 384 × 288 world with all six tiers present and one committed
2,070 km route, captured at the reset view and at 1.5×, 3× and 5.9× it. Before:
labels stretched to 54 px of bitmap mush, 233 places drawn at the default view,
the route a soft amber smear. After: labels crisp at every zoom, 33 places at
the default view with addons revealing at 2.4×, the route a thin dashed line
over its dark underlay. Headless boot and `smoke_test.gd` clean.

**Left open.** `VILLAGE_ADDON_POP` identifies an addon village by its
unconditional `pop: 0` — exact for the default pipeline (the smallest base tier
floors at `round(120 × 0.7 × 0.8) = 67`), and documented as such in both
`VillageSettlement` and `lib.rs`. It is still a proxy for a flag the engine
already keeps: `CivData::village_tids`, sitting beside the `tid` that
`get_settlements()` already emits. Exposing it is one line in `get_settlements()`
and would retire the proxy; not taken here because `crates/cartalith-godot/
src/lib.rs` was under concurrent edit. One case degrades until then — with the
static post-collapse recovery phase enabled, `civ_apply_recovery` floors every
population at 8, so an addon village stops reporting 0 and is drawn as an
ordinary hamlet. That is today's behaviour, so the degradation is "no
improvement", never a place wrongly hidden.

**Also noted, not changed.** `engine_bridge.gd` defaults `villages` to `true`
(`request.get("villages", true)`) where the reference's `_civVillages` is
`false`, *"OFF by default ⇒ auto-populate output bit-identical"*. Every world
this shell generates therefore carries the additive layer. That is a generation
default, not a rendering defect, and changing it changes what is generated —
`DECISIONS.md` territory, raised rather than taken.

## 31 · TO-01, TO-02, CV-20, MN-09, SH-15 — a second overlay in the wrong space, and four surfaces that had stopped telling the truth (2026-08-24) — **FIXED**

Owner report: *"plenty of minor discrepancies at the same time"*, alongside the
batch §29 and §30 answered. This pass repeated the method that keeps working —
read a `design/*.dc.html` canvas as ground truth, then **drive the live shell
non-headlessly** and measure — across areas §29/§30 had not touched.

| id | Symptom | Root cause | Fix |
|---|---|---|---|
| TO-01 | Every measure ruler, region marquee, path preview and A/B end label thickens and blurs as the map is zoomed — **the same defect §30 fixed, in the other overlay** | `ViewportHost` parents `map_overlay` **and** `tool_overlay` under `_camera` and scales that camera, but only `map_overlay` was ever told the zoom. Every constant in `tool_overlay.gd` — a 1.6 px `draw_polyline`, a 3 px `MEASURE_POINT_RADIUS`, an 11 px `draw_string`, a 1.4 px dashed marquee with a 6 px dash and 6 px corner squares — is therefore in the control's *local* pixels, magnified along with its rasterisation | `_crisp_begin()`/`_crisp_end()`, `map_overlay.gd`'s own mechanism, applied to the whole `_draw()` rather than to the text and linear layers alone: this control emits nothing but tool chrome, which is screen furniture by definition. The two radii that *are* real map distances — the brush ring, the Radius reading's circle — multiply by `cell_px` so they keep scaling. The zoom is **read off `_camera.scale.x` in `_process`** rather than pushed from `viewport_host.gd`, which was under concurrent edit; `set_notify_transform(true)` was tried first and does not work, since a `Control` ancestor's `scale` change does not propagate `NOTIFICATION_TRANSFORM_CHANGED` to its children |
| TO-02 | A selected label's or icon's resize/rotate/arc handle is a much smaller circle than the region that actually answers a click on it, and the mismatch grows with zoom | **`HandleCircle.r` is in grid cells, not pixels.** Both producers build it in the same space as `x`/`y` (`label_bridge::handle_circles` offsets from `LabelBox.px/py`, `icon_bridge::icon_handle` from `IconBox.px/py`), both floor it at `4.0` *cells*, and both hit-test it at that radius against a grid-space cursor. `tool_overlay.gd` passed it to `draw_circle` untouched — as four *pixels* | The same `cell_px` conversion the brush ring already used. Measured: `r = 6.4` cells at 2.31 screen px/cell now draws a 32 px circle against the ~30 px the hit test answers, where it drew ~13 px before |
| CV-20 | CIVIL ▸ Politics offers *Recalculate territories* and *Generate provinces* under a heading reading **Not built**, both greyed, both with tooltips asserting that no `#[func]` re-runs either — while **Recompute civilisation, eight rows up in the same dock, does both** | The two tooltips were written before SG-02 shipped and were never revisited. `civ_recompute()`'s own result dictionary reports `provinces` rebuilt, `_recompute_civ()` re-uploads `territory_texture()`, and this same file's Settlement-tool status hint has said so since that pass. Exactly §30's class of stale copy, and the same shape as the bake pass's *"No bake/LOD pipeline exists yet"* | Both are live **shortcuts onto `_recompute_civ`** — one owner of the action, two ways in, no second implementation to drift; the bake pass's own pattern. Each tooltip now names the real remaining limit (it does not re-*place* settlements; only Generate does). *Clear territory* stays disabled — genuinely absent — with its "Same:" premise corrected |
| MN-09 | Assets ▸ Asset pack ▸ Build ▸ *Export pack .zip… ⌘⇧P* prints its shortcut **twice**, once as a modifier key neither shipping platform has | The label baked `⌘⇧P` in as text *and* `set_item_accelerator` added `Ctrl+Shift+P`, so the popup rendered `Export pack .zip… ⌘⇧P    Ctrl+Shift+P`. The canvas draws `⌘⇧P` in the popup's own accelerator column, not inside the label — the port copied the glyph and kept the column | Label only; the accelerator alone renders the canvas's layout, in the notation the machine actually has. Sibling: Batch ▸ *Delete ⌫* advertised a Backspace binding that **exists nowhere** (`app.gd`'s `_unhandled_key_input` routes Backspace to the armed tool and no further; `asset_library_window.gd` has no key handling at all) on a row that opens a window rather than deleting anything. Glyph dropped |
| SH-15 | §10's timeline strip is **70 px of blank panel across the whole window** whenever CIVIL is the active domain, and `Window ▸ Timeline` toggles that blank band on and off | The timeline's controls deliberately live in the CIVIL dock's own Timeline category (`TIMELINE_SCOPE.md` §4: default to a dedicated panel rather than risk the wrong shell region). Leaving the reserved region *on screen and empty* was never part of that decision | The strip carries a pointer: a `TIMELINE` caption, one clipped line saying where the controls are, and an **Open Timeline** action that presses the dock category's own header (`CivilizationWorkspace.open_timeline_category()`). Re-filled by `toggle_region()` too, so turning it on from another domain does not bring the blank band back |

**What was checked and found clean**, since a negative result is worth as much
as a finding here and stops the next pass re-walking it:

- **The Layers popover's field views, all 37 rows, driven.** 33 available rows
  clicked through the real `pressed` path: every one set the view it claimed
  (`debug_view()` echoed the picked id), every one produced a distinct raster
  (37 FNV hashes, **no duplicates, no nulls**), all four unavailable rows were
  correctly disabled, and **hotkeys 1-8 each selected exactly their badged row**.
  Opacity live; still correct after a regenerate to a different grid size.
- **A dead-control sweep of the whole live tree** — every enabled, visible
  `Button`/`CheckBox`/`OptionButton`/`Slider`/`LineEdit`/`SpinBox` with no
  connection on any of its signals — across the shell chrome, all three domain
  docks, four right-dock contexts (settlement, faction, sculpt, region), **all
  nine tool-options bars** (sculpt, paint, measure, icon, label, settlement,
  territory, way, route) and eight windows (Asset library, Data manager,
  Faction roster, Place editor, City viewer, World data, Performance, Travel
  library). **No dead controls.** The four flagged by a first, cruder heuristic
  were all false positives: `toggle_mode` tabs connecting `pressed` rather than
  `toggled`, `ColorPickerButton`s connecting `color_changed`, and search fields
  filtering live on `text_changed` rather than `text_submitted`.
- **Every menu accelerator in the shell**, enumerated from the live popups:
  11 of them, each matching its label, none unreachable except `Ctrl+Z` while
  the undo stack is legitimately empty.
- **Camera-space rasterisation, exhaustively.** `map_overlay` (§30) and
  `tool_overlay` (here) are the only two `_camera` children that draw. The rest
  are `TextureRect`s; `wind_fx_layer.gd`'s one-cell stroke width is the
  reference's own and is *meant* to scale; `journey_planner_view.gd` and
  `section_strip.gd` draw in unscaled dock controls.

**Verified live, non-headlessly**, 1600 × 1000, seed 483920 over a 384 × 288
world, by difference against the same frame without the primitive so no
assumption about terrain colour enters. TO-01 before: the 1.6 px ruler rendered
**2 / 6 / 12 / 16 px** at zoom 1 / 2 / 4 / 6, and the 11 px `A` label's bounding
box went **17 × 18 → 69 × 74 px** between zoom 1 and 4. After: **2 px at every
zoom**, and **17 × 18 px at both**, while the 20-cell brush ring still grows
94 → 372 px as it must. TO-02 measured in the same run. CV-20 driven for real:
pressing *Recalculate territories* printed *"Recomputed in 0.8 s: 233
settlements kept, 60 ways and 8 provinces rebuilt against the current terrain."*
SH-15: 4 children in CIVIL, still 4 when switched on from WORLD via the Window
menu, strip minimum width 236 px against a 1600 px window and a 300 px right
dock — no squeeze.

**Left open, reported rather than taken:**

- ~~**The map's top-right readout does not carry what the canvas puts there.**~~
  **Closed, 2026-08-24.** `viewport_host.gd`'s `_update_zoom_readout()` now
  draws `2D · equirect · z%.1f` over the active style-preset name, matching
  `design/Cartalith DCC Shell.dc.html`'s `2D · equirect · z 5.2` / `relief ·
  atlas preset` structurally rather than literally — "2D" and "equirect" are
  honest constants (`DCC_SHELL_SPEC.md` §2.4: "this port works in one flat km
  projection throughout"), not a lookup, and the second line is the real
  active Map style preset (`render_workspace.gd`'s five chips plus "Custom"),
  pushed in via a new `ViewportHost.set_style_readout()` rather than polled.
  Grid size and extent (what the port used to show here) already have a home
  — the WORLD dock readout and the Sample panel — so nothing lost a display.
- ~~**The Asset Library has no keyboard delete.**~~ **Closed 2026-08-24.** The
  design question this row deferred was answered the least clever way
  available: `_unhandled_key_input` on the library `Window` routes Delete and
  Backspace **into `_on_batch_delete`**, so the key does exactly what the
  button does and raises the same confirmation, with the same count and the
  same "custom slots are removed entirely, frozen slots are emptied" wording. A
  second, key-only prompt would have been a second place for that wording to
  drift. Scope is the grid selection; undo stays what it was (there is none,
  and the prompt now says *"This cannot be undone."*). Two guards, both of them
  the ones `app.gd`'s own handler needed: **a focused text field wins** (`LineEdit`
  /`TextEdit`/`SpinBox` — Backspace in the rename prompt or the tag field is
  never a delete), and **an empty selection says so** in the status bar rather
  than returning silently, which would make the key look dead on exactly the
  press that teaches a user it exists. It lives on the window rather than on
  `DccApp` because a `Window` is its own `Viewport`: the key arrives only while
  the library has focus, which is why no "is the library open?" check is needed
  and why the slicer modal does not steal it. The menu glyph stays off —
  `menus.gd` says why: that row opens the window, it does not delete.
  **Verified non-headlessly** on a live 7-slot library: empty selection → 0
  dialogs and a hint; Delete with 2 selected → 1 dialog titled *"Delete 2
  asset(s)?"*, **Cancel keeps both**; Backspace → the same dialog, OK runs the
  batch (frozen slots emptied, so the slot count correctly stays 7); and
  Backspace with a `LineEdit` focused → **0 dialogs**.
- **Four `ID_*` constants in `menus.gd` are declared and referenced nowhere
  else**: `ID_REDO`, `ID_HELP_SHORTCUTS`, `ID_PREF_QUALITY`, and
  `ID_PREF_UNITS_KM`/`ID_PREF_UNITS_MI`. Not user-visible (Redo is a real
  `_todo` row with a documented reason; the units switch is the same gap
  `tool_bar.gd`'s Measure options already disclose), but they are the residue
  of four intended surfaces and are recorded here so the intent is not lost.

## 32 · LZ-01 — deep zoom stopped twenty times short of the reference, and the tile it drew had run out of octaves (2026-08-24) — **FIXED**

Owner report, verbatim: *"LOD zooming doesn't seem to go that deep either."*

Measured before changing anything, on a default 800 km × 512×384 world in a
1600×1000 window:

| | before | reference | after |
|---|---|---|---|
| deepest camera zoom | **8.0×** | `lodMaxZoom()` = **160×** | **160×** |
| closest visible span | **100 km** | **5 km** | **5.00 km** |
| deepest tile resolution | 16 px per coarse cell | 256 px | 256 px |
| procedural octaves at depth | **1, fixed** | `min(6, z − zBase)` | `min(6, z − zBase)` |
| synthesis cost, one tile | 251 ms (1024²) | — | 12–34 ms (256²) |
| tiles synthesised per viewful | 4, growing with depth | — | **24 at every depth** |
| `_update_lod()` per camera move | — | — | 0.1–0.2 ms, no backlog |

**Three separate ceilings, only the first of which is what it looked like.**

### 1. `ZOOM_MAX = 8.0` was the wrong constant, copied from the wrong camera

The reference caps `viewT.scale` at 8 (line 13381) — and this port took that
number. But the reference *hands the camera off* at 2.2×: `enterLodFromView`
(13953) pins `viewT.scale` back to 1 and gives zoom to the tiled-LOD viewer,
whose own `_lodZoom` runs to `lodMaxZoom()` = `max(64, ceil(mapWidthKm/5))`
(10672) — **160 on a default world**. That function exists because of an owner
report with the same shape as this one; its v0.88 comment says so outright:
*"highest zoom stops at 20km, I'd like to drop down to 5km … Scale the cap so a
real-world span of ≤5km is always reachable."*

So the reachable depth is a property of the map's real width, not a screen-space
constant. `ViewportHost` now computes it per world in `refresh()`.

### 2. The tile had a fixed footprint, so its resolution saturated

`lod_bridge` addressed tiles on a fixed 64-coarse-cell grid and grew the *output*
(256/512/1024 px) with a `detail_level` capped at 2 — a ceiling of 16 px per
cell, which is exactly where `ZOOM_MAX = 8` had been set. Raising the cap alone
would only have magnified a 16 px/cell tile.

It is now a **pyramid** tile: level `z` divides the map into `2^z × 2^z` chunks
of one fixed pixel size (`cartalith_spatial::pyramid`, `pyramid_tile_bounds`,
`pyramid_level_for_zoom` — all already ported and golden-tested for the bake),
so the *footprint* shrinks with depth while the cost per tile does not. That is
also why the tile count per viewful is now flat at 24 from z3 to z9 instead of
growing: it is bounded by screen area, not by zoom.

### 3. The tile carried no progressive detail at all

`synthesize_tile_rgba` called `amplify_region` alone — and the reference names
that exact failure in `addZoomDetail`'s own header: *"amplifyRegion adds detail
at a FIXED coarse-space frequency, so the fbm runs out of octaves at high zoom
and the surface goes smooth ('details don't get more intricate')."* The
synthesis is now `cartalith_engine::bake::pyramid_tile` verbatim —
`refine_tile` **plus** `add_zoom_detail`'s `min(6, z − zBase)` progressively
finer octaves — so a tile drawn on screen and a chunk baked into the atlas over
the same ground are the same numbers, by construction.

`z_base()` is shifted by `log2(1024 / TILE_PX)`: the reference's `zBase = 2` is
quoted against its 1024 px `_lodTile`, and a 256 px tile reaches the same ground
resolution two levels deeper, so using `2` unshifted would have added two octaves
past what the tile can resolve — noise, not detail.

### The three things the live driving found that reasoning had not

- **The hillshade faded to nothing with depth.** `shade_tile` differences
  *adjacent pixels* with a fixed `exag`, so the same ground slope shades
  `1/px_per_cell` as hard once a tile spreads one cell over many pixels.
  Measured on a dome fixture: 34% of mask pixels carried any shading at level 4,
  3% at level 7 — deep zoom converged on a flat mask over a smooth blur *even
  with the octaves in*. The exaggeration is now normalised by the tile's own
  pixels-per-cell, which makes the ratio scale-invariant; the same fixture then
  runs 2.44 → 3.36 → 4.31 → 5.49 in mean adjacent-pixel difference across four
  levels instead of 0.30 → 0.03. This is a free parameter, not a reference
  constant: the shade *ratio* is this port's own construct.
- **`gui/common/snap_controls_to_pixels` destroys a deep-zoom `TextureRect`.**
  It rounds a `Control`'s position and size to whole *local* pixels, and
  `_camera`'s local pixel is `_zoom` screen pixels — at z160 the whole map is
  5.5 local px wide, so a tile 1.74 local px across was snapped to 1 or 2, i.e.
  160 or 320 screen px instead of 278. Diffing the same frame with the layer
  shown and hidden came back with 40 px vertical and 120 px horizontal bands the
  layer changed *not at all*, from a tile set whose own arithmetic covered the
  screen with a one-pixel overlap. Tiles are `Sprite2D` now — `Node2D` carries a
  float transform and is never snapped. The old code got away with it because a
  64-cell tile at `ZOOM_MAX = 8` was several hundred local px.
- **The scale bar was the one readout that would have told the owner how deep
  they were, and it said the same thing at every zoom.** It printed the map's
  full width flat, so the deepest reachable view still read *"800 km across"*.
  It is `lodSpanKm()` now (reference 10675, whose own comment calls it *"the
  single source of truth for both the scale bar and any future 'current view
  width' readout"*), and reads **5.00 km across** at the cap.

### What is genuinely a separate milestone, checked rather than assumed

- **Reading the baked atlas at draw time is not the depth fix**, and wiring it
  as built would have reintroduced a bug that was fixed the day before. A baked
  chunk's PNG is `region_export::tile_png_bytes`, the **Relief** coloriser — the
  hypsometric ramp the 2026-08-23 pass removed from this path precisely because
  it disagrees with the biome map at every pixel ("a zoom action exposes the
  underlying heightmap"). The reusable half is the chunk's *height* (`rg16`,
  `cartalith_io::decode_chunk`), which has no `#[func]`. And the depths that
  matter are past baking anyway: a depth-7 pyramid is 21 845 tiles, so the atlas
  can only serve shallow levels — where live synthesis is now 12 ms. Real, but
  an optimisation, not the ceiling.
- **The colour at depth is still an interpolation of the coarse raster.** The
  relief is now genuinely sub-cell; the *palette* is not, because
  `renderBiomeTileRGBA` is unported (it needs temperature, rainfall, lithology
  and flow at sub-cell resolution). Named in `lod_bridge.rs`'s own header since
  2026-08-23 and unchanged by this pass.
- **The frame at extreme zoom costs ~27 ms, and the LOD layer is not why.**
  Measured at z160 with the layer visible, hidden, and at the reset view:
  27.17 / 26.74 / 16.68 ms. The layer is worth 0.4 ms; the other ~10 ms is
  whatever the base raster and `map_overlay` cost at 160× magnification, newly
  reachable rather than newly slow. Not chased here.
- **`_umLayoutAlpha`'s 24 km → 10 km crossfade is live ground for the first
  time.** `map_overlay.gd`'s urban reveal gate is a pixel rule written against
  the old ~100 km floor, with a comment stating that a ported 24 km threshold
  "would never once fire". That premise expired with this change. The comment
  now says so; the gate was deliberately not swapped in the same pass, because
  it is a visible behaviour change belonging with the rest of `_umLayoutAlpha`.

## 33 · RD-01b — the sea lanes and committed routes drew their chords too, and the road fix had a live NaN in it (2026-08-24) — **FIXED**

§29 fixed the roads and registered its own leftover: `get_sea_routes()` and the
committed Route-tool list have the same shape and the same chord problem. This
closes that, and found something bigger on the way in.

### The chord half, which was the expected part

`get_sea_routes()` (both halves — generated `sea_routes` and the manual `sea`
ways out of `InfraTools`) now goes through `way_render_geometry`, the same
boundary helper §29 introduced. `route_get()` does too, but as a **second key**
rather than in place, and that is the one real design difference from §29:

- `points`/`brks` stay exactly the engine's own list. `jp_compute` plans over
  `CommittedRoute::pts` and returns `plan.stages[i].{i0, i1}` as indices into
  that list, which `journey_planner_view.gd` slices to colour the route map per
  stage and to derive stop fractions. Densifying `points` would have silently
  mis-sliced every stage.
- `render_points`/`render_brks` are the drawn polyline. `map_overlay.gd`'s
  route pass reads those, falling back to `points` for an older GDExtension
  binary.

Measured on §29's own world and probe (seed 483920, 384×288, 2400 km,
`_routecurve_shot.gd`):

| | points | chord mean | max | turn/vertex |
|---|---|---|---|---|
| sea lanes, after | 807 over 2 lanes | 0.246 cells | 0.314 | 1.686° |
| committed route, before | 124 | 2.856 cells | 4.243 | 13.607° |
| committed route, after | 1437 | 0.245 cells | 0.330 | 1.665° |

`km` did not move (2195.460 engine-side, drawn length 351.27 → 352.18 cells —
the curve is marginally longer than its own chords, which is what a curve is),
and both endpoints are byte-identical before and after. Roads re-measured in
the same run come back at 6342 points across 35 ways, chord mean 0.2450 —
§29's recorded figures to the digit, so the shared helper did not disturb them.
Shot at 31× on the committed route: before is a polyline with a visible kink,
after is one continuous sweep.

### The NaN half, which was not

The sea-lane measurement's first run came back `chord mean -nan`.

`civ_catmull_rom_sample` parameterises each segment by `sqrt(chord)` and the
Barry-Goldman evaluation then divides by **all three** knot intervals, while
only the middle one (`t2 - t1`) is guarded. Two equal consecutive control
points make `t1 - t0` or `t3 - t2` exactly zero in a *neighbouring* window, so
`lerp` computes `0 * (x / 0)` and every point of that segment is NaN. One NaN
coordinate ruins the whole `PackedVector2Array` the renderer is handed.

It is unreachable from `civ_smooth_path`, the reference's only caller, because
that splines `civ_rdp_simplify`'s output and RDP always drops a duplicate (its
deviation from the chord is exactly zero). **§29's fix introduced the first
caller that can reach it**: `way_render_polyline` re-splines `_civSmoothPath`'s
*rounded* output, where two successive samples landing in the same cell is a
routine occurrence — `golden_parity_sea_routes.rs` records two case-1 routes
carrying `km: 0` for precisely that reason. So this was a live defect in
already-shipped road rendering, not only in the new code; it simply had not
been measured on a way that stalls.

Fixed **in `civ_catmull_rom_sample` itself**, not avoided at the new call site,
so roads, sea lanes and committed routes are all covered by one guard: repeated
consecutive control points are collapsed before the phantom endpoints are
built. That is parity-neutral rather than a deviation from the reference, and
the argument is exhaustive — for any input with no repeated consecutive point
`dedup` is the identity, and *every* input that has one previously produced
either NaN (runs of three or more) or an empty result (a two-point run, via the
existing `t2 - t1` skip, which the new `< 2` check reproduces exactly). No
fixture can tell the two versions apart. `WAY_RENDER_STEP_CELLS`'s local dedup,
written while chasing this, was removed in favour of the central one.

Mutation-tested rather than assumed: with the guard forced off, three of the
five new tests fail (`catmull_rom_survives_coincident_control_points`, which
walks a duplicate through every position in a five-point way,
`catmull_rom_survives_runs_of_repeats`, and the boundary-level
`a_repeated_cell_in_a_rounded_way_does_not_produce_nan`) and two stay green —
the two that pin *reference* behaviour the guard must not change
(`catmull_rom_degenerate_inputs_match_the_reference`, and
`catmull_rom_keeps_a_near_coincident_pair`, which exists so nobody later
"improves" the exact-equality test into an epsilon and quietly moves the curve).

`cargo test -p cartalith-civ` 372 lib + every golden-parity suite passing,
`cargo test -p cartalith-godot --lib` 337 passing (five new), `cargo build -p
cartalith-godot` and headless boot of `shell/app.tscn` both clean, and a real
non-headless run scanning all three getters reports 0 non-finite of 6342 road,
807 sea and 1437 route points.

## 34 - RN-04, CA-14 - the renderer was already sophisticated; its defaults were conservative (2026-08-24) - **FIXED**

Owner analysis, and a correct one: the reference's renderer runs a full pipeline
(climate -> material weights -> biome/material colour -> texture -> relief ->
multi-scale hillshade -> curvature/AO -> atmospheric haze -> optional painter
effects -> rivers/coast). What makes it look muted is that **most of its
enhancement sliders default to `0`** and its base palettes are low-chroma - not
a missing-features problem. The instruction was explicit: do not rewrite the
renderer. Nothing here is a rewrite.

| id | what | now |
|---|---|---|
| **RN-04** | The four remaining reference render stages the panel's own note listed as unported: **ridge crests**, **surface texture**, **ridged relief**, **curvature shading** | **CLOSED.** Literal ports in the reference's own pipeline slots - `build_crest`/`apply_crest` (8005-8023, applied at 8171 and 11971), and three blocks inside `land_color` (7841-7851, 7853-7862, 7870-7876). `cartalith-noise` gained `ridged_oct`, the reference's general `ridgedFbm(x, y, oct, s)`, **beside** the golden-verified fixed-six-octave `ridged` rather than by rewriting it. All four are `0.0` in `Default`, so `golden_parity_render.rs` is untouched. |
| **CA-14** | **Colour grading** - the group `render_workspace.gd`'s own Still-owed block has listed since the dock was built | **CLOSED (six of ten axes).** `render::apply_color_grade`, a presentation-only post-process over the **finished raster**, after `apply_local_contrast` and before the Godot overlays draw rivers, labels, icons, territory and the scale bar. Exposure, contrast, saturation, temperature, shadow tint, highlight tint - one pass, in that order. Saturation is exactly luminance-preserving and both hue axes are luminance-compensated, so a graded map keeps the value structure the relief pipeline built. **CLOSED (2026-08-24, all ten axes).** Gamma joined as a symmetric power curve (exponent `2^-gamma`) in the lift-gamma-gain slot straight after exposure, gated at `0` so no `powf` runs at rest. The **four field-influence weights** the design nests under COLOUR (`design/Cartalith Menu Structure v2.dc.html`: "+ Field influence weights - Biome / elevation / moisture / geology"; `TERRAIN_APPEARANCE_RESEARCH.md` SS17 lists the same four) are weights *on the grade*, not axes: `render::build_grade_influence` reduces each field to a `0..1` per-cell signal (relative land elevation, rainfall, `BIOME_VEGETATION_COVER[classify_biome(t,m)]`, and the lithology palette's own lightness), centres it, and sums to one multiplier per output pixel that scales every axis' departure from rest. All four at rest returns an **empty** buffer and the grade is byte-identical to the six-axis version; a weight with no grade under it is still the identity, and `grade_is_identity()` ignores the four deliberately for that reason. Both call sites pass it, so screen and export cannot disagree. In the dock as an adjacent **Grade field influence** group (this shell has no in-group nesting). **Still owed**: free colour pickers for the two tints -- they remain a blue-to-amber axis, not arbitrary colours. |

### The three controls that are not ports

- **`relief_chroma`** answers the owner's second point directly. The reference's
  relief blend is `grey = 185 * light`, and a `bio_blend` under 1 lerps toward
  it - which costs **value as well as chroma**, dragging every shaded pixel
  toward one fixed neutral. That is why the shipped `0.90` reads as a faded map
  rather than a lit one. At `relief_chroma = 1` the grey target becomes a grey
  of *the pixel's own* luminance (so the blend is exactly a desaturation) and
  the light factor additionally cools and slightly desaturates shadow while
  warming and slightly saturating sun. `0.0` is the reference byte for byte.
- **`biome_sat`** - chroma of the material mix about its own Rec.709 luma, so it
  can never move one material lighter or darker relative to its neighbour.
- **`haze_strength`** - the reference's own `0.18` literal, made adjustable. The
  haze colour `(208, 218, 230)` stays the reference's: it is the sky, not a
  taste.

### Named looks - how the shipped default moved without touching the parity path

`TerrainAppearance::js_reference()` is `Default` with the stage gates zeroed, so
**changing a palette in `Default` changes the JS-parity path**, and
`golden_parity_render.rs` is not re-baselineable (`DECISIONS.md` 7a's carve-out
is scoped to paths where JS parity is *impractical*, and says in as many words
that the CPU rendering port stays golden-verified).

So the re-pitched palettes, the enabled stages and the grade live in a **named
look** layered over the quality tier - `LOOK_PRESETS` /
`TerrainAppearance::with_look`, bound as `list_looks`/`get_look`/`set_look` -
and `WorldGen` opens on **Natural Vibrant**. The tier decides what the renderer
*spends*; the look decides what the picture *is*; a phone answers only the first
question differently, which is why a look never touches a radius, a light count,
or a stage a cheap tier switched off.

Three looks: **Quality tier** (the identity), **Natural Vibrant** (the new
default), **Antique Parchment**. The last one is section 7 of the owner's brief -
the warm hand-illustrated MapEffects-style plate - and it **refines rather than
duplicates**: the existing Antique Map-style chip was `{"sepia": 0.35}`, a
toning matrix over the muted base and not a palette, and it now names this look
as well, so Antique is a warm aged sheet *and* the sepia.

Every Map-style chip now carries a look alongside its Painter bundle, which is
what lets Ink put pen lines over the vibrant base rather than over the
reference's muted one. `reset_appearance()` deliberately does **not** clear the
look - it has its own picker, and a button in another section silently moving
that picker is the desync this register keeps having to fix one control at a
time.

### Where the specification and the port's state disagreed

The numbers were written against the reference, where every enhancement slider
is `0`. Three are not zero here, and only two moved:

- **Geology 25 %** - this port's equivalent is `litho_strength`/`litho_exposure`
  at `0.62`/`0.55` since milestone 5, i.e. *more* geology than asked for.
  Lowering them would have made the vibrant look less geological than the plain
  tier. **Left at the tier's values.**
- **AO 20 %** - taken **down** from the tier's `0.28`, as specified. Coherent,
  because crests, curvature and ridged relief now carry the local relief the
  broad cavity map used to carry alone.
- **Wetness 12 %** - taken down from the tier's `0.38`, which was itself set by
  the same day's owner-authorised CA-11 retune. A real reduction, made because
  this instruction is the later one and names the number.

### Verified

Non-headless, a real world at 2048x1311 (seed 483920). Quality tier -> Natural
Vibrant: **73.29 % of pixels moved, mean chroma 48.67 -> 63.37 (+30 %), mean
luma 139.61 -> 138.36 (unchanged), luma sd 42.71 -> 48.44 (+13 %)**. That is
"richer, more dimensional, still physically grounded" as a measurement, and it
is nowhere near the 2x that would be the rainbow biome map the owner named as
the failure mode. Every grade parameter back to rest returns the base at
**0.0000 % moved, worst 0 levels**. Through the real dock: the Base look picker
opens on Natural Vibrant, the right chip is lit, 35 appearance rows draw, a real
slider drag on Colour grade > Saturation reaches the engine and moves 98.5 % of
the raster, and the Antique chip lands the look and the sepia together. A saved
look round-trips the new fields at 0.0000 %.

**Disclosed rather than tuned away**: surface texture and ridged relief are
nearly invisible at the specified 18 % and 10 %. That is the reference's own
arithmetic - `1 + 0.2 * k * (T - 0.5)` at `k = 0.18` is a plus-or-minus 1.8 %
modulation, about 2.5 levels on a 140-luma pixel - not a porting error. Both
were left at the numbers the specification names rather than quietly multiplied
up; their sliders are live and reach real strength.

### Verified again, in the export (2026-08-24)

The block above measured the **viewport** only. The export raster arrived a
commit later and does carry the grade, in the same slot - but nothing proved
it, and the obvious proof does not work: the grid-resolution byte-for-byte
probe passes under `Natural Vibrant`, whose grade is the identity, so
`apply_color_grade` early-returns on both sides and a missing call is
invisible. Re-run under **`Antique Parchment`**, the one shipped look that
grades, non-headless at 2048x1312 (seed 20260824):

| | worst | of 8,060,928 bytes |
|---|---|---|
| Natural Vibrant, export vs viewport | 1 level | ~16 |
| **Antique Parchment, export vs viewport** | **2 levels** | **10** |
| Antique with the grade zeroed, export vs viewport | 1 level | 9 |

The grade isolated, by zeroing the six axes through `set_appearance` with the
look otherwise unchanged: **87.85 % of bytes moved, mean 4.23 levels, worst
16 - identical figures for the export and for the screen.** The worst of 2 in
row two is row three's worst of 1 amplified by Antique's `+0.08` contrast
(a slope of 1.064) across a `floor` boundary, not a second defect; the probe
asserts that relationship rather than the loosened number. Two tests in
`bake_raster.rs` now hold both halves offline. See `CHANGELOG.md`, *"The graded
export was right, and nothing could have told you"*.

---

## 35 · KV-01, KV-02, KV-03 — the Markdown vault reached the shell, and continents became addressable on the way (2026-08-24) — **NEW SURFACE**

Not a gap this register found: a **new subsystem** the owner scheduled on
2026-08-24, recorded here because it adds three connected surfaces to a
document whose job is to say which surfaces are connected, and because the
audit it started with turned up an entity that did not exist.

`MARKDOWN_VAULT_SCOPE.md` is the scope document; this is the register's view
of it.

| id | Surface | Where | Status |
|---|---|---|---|
| **KV-01** | Place editor ▸ **Knowledge** — linked-note count, worst status, and the affordance that opens the vault panel scoped to this settlement | `place_editor_window.gd` | **Connected.** Keyed on `tid`, not on the array index. |
| **KV-02** | CIVIL ▸ Politics ▸ **Linked notes** — every province and every continent, each with its own link count and status | `civilization_workspace.gd` | **Connected.** |
| **KV-03** | **Markdown vault** panel — connect, browse, attach (whole document or one heading), the working copy, the previewed section write-back, the Cartalith block, and author-field population | `vault_window.gd` | **Connected.** |

**The entity audit is the part worth carrying here.** `ROADMAP.md` required it
before any code, and it changed the plan:

- **Settlements** — real, and the strongest of the three. `NamedSettlement::tid`
  survives a rename, a move and a neighbouring deletion.
- **Provinces** — real. `Province::id` is sequential over the seed order and
  is re-derived by `civ_recompute()`.
- **Continents** — **did not exist.** What the roadmap audit called "world
  structure archetypes" is `generate_continentality_field`, a per-cell scalar
  with no per-instance identity, no name and no boundary. What *did* exist is
  `build_landmass_quality`'s golden-verified 8-neighbour flood fill, whose
  `comp`/`sizes`/`count` bookkeeping its own doc comment says was "kept for…
  later milestones" and which `compute_civilisation` has computed and
  discarded on every generate since Phase 2. `cartalith_civ::civ_continents`
  keeps it: rank by area, a name, a bounding box, a centroid, a plurality
  faction. `WorldGen::get_continents()`.
- **POIs** — confirmed absent, and **not built as a side effect**. CV-01's own
  entry stands. `EntityKind` has three variants and no `Poi`, and
  `MARKDOWN_VAULT_INTEGRATION.md` §35's criteria 6 and 7 are recorded as
  unsatisfiable in this port rather than faked.

**Two live-run findings**, both invisible to unit tests:

1. Continent 1 and settlement 1 came out with the **same name** in a real
   generated world, because `civ_name_rng`'s seed is a fixed reference quirk
   and both were drawing its first value. Continents have their own stream now
   (`civ_continent_name_rng`), with a test named after the failure.
2. `String(d.get("cells", 0))` in the new Politics rows — GDScript has no
   `String(int)` constructor, so the dock threw on every rebuild. Caught by
   driving the real shell, not by any Rust test.

**Deliberately not touched**: `DCC_SHELL_SPEC.md` §9's *MARKDOWN VAULT ·
LINKED* block in the Data manager. It assumes `obsidian://` links in exported
tiles, note links inside exported GeoJSON, and a **two-way sync toggle** —
which is `MARKDOWN_VAULT_INTEGRATION.md` §33's explicit V1 non-goal.
`DCC_CONTROL_INDEX.md` records the conflict and the design's own header
resolves it: that block stays deferred, and the vault path / note-count
readout is the only part of it consistent with V1. Nothing in this pass writes
an `obsidian://` link, a wikilink or a block reference anywhere.

**What KV-03 still owes**, each with a stated reason and each in the panel's
own footer as well as here: the map snapshot (§21 — needs a crop of the live
renderer at three radii, held as its own milestone rather than shipped as a
broken image link); Compare-with-source (§14 — this shell has no diff widget,
so a stale source offers Reload or Keep, which are the two actions that cannot
lose work); project-scoped links (§26 — blocked on the save format carrying a
civ layer at all, which `SAVEFILE_COMPAT.md`'s format does not); and the
Android SAF provider. See `MARKDOWN_VAULT_SCOPE.md` §5 and §8.

---

## 36 · RD-02, CA-15 — every land way was the same colour, and the way-type filter could not see two thirds of the network (2026-08-24) — **FIXED**

§29 and §33 fixed the *geometry* of ways, sea lanes and committed routes. This
is the matching pass over their *type and colour*: a type-by-type comparison of
`drawCivLayer` §2a/§2b (reference lines 15494-15560) against
`map_overlay.gd`, driven live and measured in pixels rather than read.

Two real defects, both found by measurement.

### RD-02 — five land way types, one colour

The reference's §2a is a six-branch ladder, and every branch strokes **twice**:
a dark underlayer, then the type's own colour on top of it — solid for the two
trunk tiers, dashed for the three minor ones. That two-stroke shape is what
makes a highway, a track and an ancient way tell apart at a glance.

This port drew land ways with **one** stroke, in one flat colour, with only the
width varying by type. Its own class doc admitted half of it (*"this control
strokes every land way in `ROAD_COLOR` regardless of type"*) as a note attached
to the `ancient` row, so it read as one type's cosmetic shortfall rather than
as the whole ladder being missing.

Measured before the fix, not assumed. `_waycolor_probe.gd` drives the real
`_draw_way_segment` on two known flat backgrounds — pure black and pure white —
which makes the drawn colour and its effective alpha exactly recoverable
(`b = C·a`, `w = C·a + (1−a)` ⇒ `a = 1 − (w − b)`, `C = b/a`), at
`set_camera_zoom(0.2)` so `_crisp_begin()`'s `1/k` transform lands every width
5× thicker and the stroke centre is fully covered. All five land types returned
the identical `C = (91, 75, 40)`, `a = 0.549` — i.e. `ROAD_COLOR` exactly, five
times.

`WAY_STYLE` now carries the reference's own five branches verbatim. Measured
after, same probe, against the composite the reference's literals predict:

| type | reference | underlayer | overlay | dash | measured composite over black | predicted | Δ |
|---|---|---|---|---|---|---|---|
| highway | 15515-17 | `rgba(20,10,5,.55)` w2.3 | `rgba(210,145,55,.98)` w1.45 | solid | (206,142,54) a=0.991 | (206.0,142.2,54.0) a=0.991 | 0.2/255 |
| regional | 15519-21 | `rgba(25,14,5,.45)` w1.8 | `rgba(178,118,52,.88)` w1.15 | solid | (158,105,46) a=0.935 | (158.0,104.6,46.0) a=0.934 | 0.4/255 |
| road | 15531-34 | `rgba(30,20,10,.4)` w1.2 | `rgba(160,100,60,.75)` w0.7 | `[1.8,1.3]` | (123,77,46) a=0.851 | (123.0,77.0,46.0) a=0.850 | 0.0/255 |
| track | 15523-26 | `rgba(30,20,10,.35)` w1.1 | `rgba(100,120,60,.75)` w0.6 | `[1.3,2]` | (78,92,46) a=0.839 | (77.6,91.8,45.9) a=0.838 | 0.4/255 |
| ancient | 15527-30 | `rgba(20,10,5,.35)` w1.1 | `rgba(120,110,100,.65)` w0.65 | `[2.5,1.3]` | (81,73,66) a=0.773 | (80.5,72.7,65.6) a=0.772 | 0.6/255 |
| sea lane | 15511-14 | `rgba(10,30,60,.4)` w1.5 | `rgba(30,130,200,.7)` w0.85 | `[2.6,2]` | (22,94,147) a=0.818 | (22.2,94.6,147.2) a=0.820 | 0.6/255 |
| route | 15555-58 | `rgba(40,25,5,.5)` w3 | `rgba(200,160,60,.85)` w1.5 | `[5,3]` | (173,138,51) a=0.924 | (173.0,137.9,51.4) a=0.925 | 0.4/255 |
| route (sel) | 15556-58 | `rgba(40,25,5,.5)` w5 | `rgba(255,210,80,.98)` w2.5 | `[5,3]` | (250,206,79) a=0.990 | (250.3,206.0,78.5) a=0.990 | 0.6/255 |

Every type within 0.6/255 and 0.002 alpha. Dash *periods* measured off the same
shots (the on/off split is not measurable — an antialiased dash cap bleeds
~width/2 past each end — but the period is): road 15 px vs. 15.5 expected,
track 16 vs. 16.5, ancient 19 vs. 19, sea lane 23 vs. 23, route 40 vs. 40, and
highway/regional flat solid (row range 0.000).

**The sea lane, route and selected route were already exact before this pass**
— those three were ported two-stroke from the start (§33, IN-09). Only the
five land types were flat. One real bug in the sea lane's own numbers, though:
its gap was **2.6, not 2.0**. `_draw_dashed_polyline`'s `gap_len` defaults to
`dash_len`, the sea lane was the one caller relying on that default, and the
reference's pattern is `setLineDash([2.6·rsc, 2·rsc])` — unequal. Measured
period 26 px where the reference's is 23. Every caller now passes the gap
explicitly; no pattern in `drawCivLayer` is actually square.

### Layering, and what it is not

The committed-route layer draws after both network layers, which §IN-09
established for the one case it checked. Re-verified as a measurement rather
than an assumption: a route was committed deliberately **along** the world's
longest highway (552 km solved over a 609 km host, 0 unreachable legs), and at
every pixel where the two coincide the rendered colour is explained by the
route's composite and not the host's — 13 of 13 coincident pixels, with the
route's least-squares residual lower at every one.

The remaining pair orders are unchanged and correct by construction: sea lanes
and land ways never contest a pixel (different getters, water vs. land), and
within `_roads` the reference itself draws in array order with no per-type
z-order, which this matches.

### CA-15 — the way-type filter listed the manual vocabulary, not the drawn one

`cartography_workspace.gd`'s **Ways · by type** group listed three keys —
`road`, `track`, `ancient`. Those are `infra_tools_bridge::parse_way_type`'s
*manual* vocabulary (IN-05's four, minus `sea_lane`). But the switches drive
`map_overlay.gd`'s filter on `get_roads()`' `way_type`, and the generated
network classifies by `cartalith_civ::WayType`, whose two busiest tiers are
`highway` and `regional` — neither of which was listed.

Measured on a real 384×288 world: **13 highways and 17 regional roads against 4
roads and 1 track**. Unchecking "Roads" hid 4 of 35 ways; the other 30 could
not be hidden at all. The reference lists all five (`CIV_WAY_TYPES`, line
14743) and always has.

`WAY_TYPES` is now that list minus `sea-lane` (which keeps its own top-level
layer row rather than a second, disagreeing switch). Verified live by toggling
each type and counting the pixels that vanished along a way of that type:
highway 284, regional 241, road 102, track 66 — all four present types now
filterable, none was before except road and track.

### Cataloguing — checked, and correct

INFRA's own lists were audited against `_civRenderWayList` (reference 17145)
and found honest: a per-tier tally, a "longest" ranking with the type in each
row's label, a **Sea lanes** group, a hand-drawn-ways group, and the editable
**Routes committed this session** list. The reference marks type with a per-row
emoji (`_wayTypeIcons`) where this uses the type word; same information.
Nothing was changed here.

### Files, and how it was verified

- `map_overlay.gd` — `WAY_STYLE` replaces `ROAD_COLOR`/`ROAD_WIDTH_BY_TYPE`;
  `_draw_way_segment` takes a style row and strokes twice; the sea lane passes
  its gap explicitly.
- `cartography_workspace.gd` — `WAY_TYPES` gains `highway` and `regional`.
- No Rust. The type data was already correct on the boundary — `get_roads()`
  has emitted `highway`/`regional`/`road`/`track` since Phase 2 milestone 14
  and appended `ancient` since IN-02. This was entirely a renderer and a filter
  list.
- `_waycolor_probe.gd` (synthetic, exact) and `_wayreal_probe.gd` (real app:
  world generation, a route committed along a road, per-type centred views,
  background-differenced pixel solving, the layering check and the filter
  check). Both temporary and untracked.

---

## 37 · The left-rail menu structure v3 pass — fifteen new IDs, and what the three rails became (2026-08-24)

`design/Cartalith Menu Structure v3.dc.html`, vendored at `8cef062`, revises
the left-rail domain menus. The owner scoped implementation to **those menus
only**: the top bar stays as it is, and v3's own top-level `Vault` menu goes
into the existing **Data** menu instead (owner, verbatim: *"the vault menu can
be shoved into data"*).

`DCC_SHELL_SPEC.md` carries the supersession disclosure — its top-of-file
notice plus inline blocks at §3, §5 and §7. This section is the register's
half: what v3 asked for that this port cannot give it, and where each unbacked
row is now disclosed rather than faked.

### What the three rails became

| Domain | Before | After (v3's own list and order) |
|---|---|---|
| WORLD | a `GENERATION PIPELINE \| SCULPT` mode switch over a numbered ten-stage accordion | 9 categories: Generate · Terrain · Geology · Hydrology · Climate · Biomes · Ecology · Resources · World data |
| CIVIL | 6 categories + INFRA's 5 appended below a rule | 14 categories: Civilizations · Factions · Territories · Settlements · Points of interest · Routes & ways · Travel · Trade · Economy · Culture · Politics · Military · Relationships · Simulation |
| CARTO | 3 categories + RENDER's flat run of sections appended below a rule | 10 categories: Map style · Terrain appearance · Colours · Layers · Roads & routes · Labels · Assets & landmarks · Political display · Visibility / zoom · Map presets |

**Nothing was rewritten.** Every builder is the one that was already there,
called with a different parent — `InfrastructureWorkspace` and
`RenderWorkspace` gained `build_*_into()` entry points and a flag that stops
them drawing categories of their own, and `world_workspace.gd`'s
`_build_stage()` became `_build_stage_body()`, which draws the same content
into a section instead of into a category of its own. v3's closing rule
(*"every #id keeps its wiring — this is re-parenting, not rewriting"*) held.

Three renames the rest of the shell had to follow, found by grepping for the
old names rather than by waiting for a user to hit one: the timeline strip's
hint and its `Open Timeline` tooltip (CIVIL ▸ Timeline → Politics /
Simulation), `layers_popover.gd`'s footer (the political and way-type switches
left Cartography ▸ Layers), and three "World ▸ Generation Pipeline" pointers in
`new_world_dialog.gd` and `tool_bar.gd`. `DccWidgets.stage_category()` lost its
only caller and is marked as such in place rather than deleted.

### The fifteen new IDs

Each is a row v3 draws that this port has nothing behind. All fifteen ship as a
disclosed note or a disabled control carrying its reason — none is drawn as a
working control, and none is silently omitted.

| # | Row v3 asks for | Where it is disclosed | Why there is nothing behind it | Class |
|---|---|---|---|---|
| **CV-21** | Faction **identity colour** and emblem | CIVIL ▸ Factions ▸ Not built | `FactionRoster` stores no colour field, and `map_overlay.gd` derives a faction's tint from its *index*. v3's own CIVIL-owns-the-colour / CARTO-owns-the-paint split has neither half | **CLOSED 2026-08-25 (§39)** — and the reason above is wrong: the roster *did* store a colour field, and nothing read it |
| **CV-22** | Faction **history, notes, lore** (v3 marks these `vault`) | CIVIL ▸ Factions ▸ Not built | `cartalith-vault`'s `EntityKind` covers settlement, province and continent. A faction is not addressable there yet | **CLOSED 2026-08-25 (§39)** — and the estimate was exact |
| **CV-23** | Borders, claims and **influence** as separate quantities; historical occupation | CIVIL ▸ Territories ▸ Not built | `CivData::territory` is one plurality-owner-per-cell grid: no contested-claim value, no influence field, no per-year ownership record beyond the timeline's settlement snapshots | **NARROWED 2026-08-25 (§41)** — §39's diagnosis was right and it is what got built: `best_effective` is kept now, beside the runner-up faction it already had to compute past (`cartalith_civ::territory_influence`). Built **on demand and retained nowhere**, so the 268 MB objection never arises. Open: **historical occupation over time**, which is timeline work, not territory work |
| **CV-24** | The year scrubber as **program scope** (v3: *"time is not a domain"*) | The timeline strip's own `Open Timeline` tooltip | Agreed in principle, and not moved. `dcc_shell.gd`'s reserved `timeline_bar` is one fixed-height `HBox` with no room for a year-pill list, an add-year field and three filter checkboxes; `TIMELINE_SCOPE.md` §4's standing instruction is to build a dedicated panel rather than guess the region. A shell-frame change, not a menu change | (C) — design first |
| **CV-25** | **Military**: garrisons, defensive strength, fortification network, campaigns | CIVIL ▸ Military ▸ Not built (whole category) | `cartalith-civ` models none of them and neither does the reference. New design, not a port gap. What exists is per-settlement *defensibility*, a terrain heuristic, on the right dock | **NARROWED 2026-08-25 (§40)** — and the reason above is wrong: the reference has `_umWallSpec`/`_umInferWalls` (22109) and `_civPlaceDefensibility` (23802), and `power.military` was already ported. Now built, golden-verified. Open: **garrison headcounts, campaigns, unit movement, combat** — and only those |
| **CV-26** | **Relationships**: diplomatic matrix, allies/rivals/subjects, treaties — and v3 Politics' vassalage/alliances/rivalries | CIVIL ▸ Relationships ▸ Not built (whole category), and CIVIL ▸ Politics ▸ Not built | There is no edge between two factions to hold a value, at any year, so a matrix would be a grid of blanks. The reference has none either. Absorbs the one-line gap the old Politics category disclosed | **NARROWED 2026-08-25 (§40)** — the reason above was right, and the edge now exists (`cartalith_civ::relations`): one derived, recomputed value per faction pair, shown as a ranked list. Open: **diplomacy actions, treaties, vassalage, and change over time** |
| **IN-13** | **Trade flows** as a routed quantity, imports/exports per settlement, route-cost field, trade-influence raster | CIVIL ▸ Trade ▸ Not built | `civ_resource_trade_balance` produces the hinterland surplus/deficit that *is* shown; nothing ties a trade relationship to the way that would carry it. `ECONOMY_SCOPE.md` holds the aggregation | **CLOSED 2026-08-25 (§42)** — and the sharpening was wrong on its second half: `_civFoodShed` **is** a bipartite supplier match and `_civRoadConnected` **is** a union-find over the way network. `cartalith_civ::trade` runs the reference's own machinery over all fifteen resources. Open, and disclosed on screen: prices, tariffs, caravans as entities, trade over time |
| **CA-16** | Per-class way **style**: colour, width, casing, dashes, route glow | CARTO ▸ Roads & routes ▸ Not built | `map_overlay.gd` draws every way from one hardcoded width-and-colour pair per type and takes no style argument; the reference's `#civWayScaleR` has no counterpart here. Visibility, which *is* wired, is the whole of what works | **CLOSED 2026-08-25 (§39)** — `#civWayScaleR`/`#wayOpacityR` ported; the reason above describes the file as it was *before* §36 |
| **CA-17** | Territory tint opacity, border width/style, claim hatching, influence gradient + legend | CARTO ▸ Political display ▸ Not built | One fixed-alpha fill with a fixed border, and no style record keyed to a faction id for anything to write to. Blocked on CV-21 at the CIVIL end | **CLOSED 2026-08-25 (§39)** for tint opacity (`#territoryOpacityR`) and, via CV-21, identity colour; the rest is CV-23's data gap |
| **CA-18** | **Zoom ladder** (what appears when) and the declutter budget | CARTO ▸ Visibility / zoom ▸ Not built | No per-layer zoom range exists anywhere in the shell; the one zoom-dependent behaviour is the urban-layout reveal band, which `map_overlay.gd` hardcodes. Label/icon collision is not resolved at all — overlapping annotation simply overlaps | **PARTLY CLOSED 2026-08-25 (§39)** — `CIV_LOD_ROAD` ported; the declutter budget and per-layer ranges stay open |
| **CA-19** | **Biome colour table** | CARTO ▸ Colours ▸ Colour grade note | `CART_BIOME_COLS` is a frozen reference table compiled into `cartalith-render` with no `#[func]` to read or rewrite an entry. The four field-influence weights beside it are live | **CLOSED 2026-08-25 (§39)** for *reading* — `debug_layers()` has carried all fifteen classes as the Biomes legend all along. *Rewriting* is a separate, larger item |
| **WW-14** | **Ecological productivity**, flora/fauna distribution | WORLD ▸ Ecology ▸ Not parameterised (whole category) | No crate computes either, here or in the reference. Vegetation density and soil *are* computed — derived off biome/climate/lithology with no dials — and are readable as analysis fields; the note points there | **CLOSED 2026-08-25 (§39)** — the reason above is wrong on **both** halves: `build_npp` and `cartalith_civ::wildlife` are both real and both golden-verified |
| **WW-15** | **Coordinate system · projection** | WORLD ▸ World data ▸ Read the fields | Every field is grid-space, the GeoJSON export writes a plain lon/lat-shaped frame with no CRS declared, and nothing reprojects. Units are km-only (PR-15) | **CLOSED 2026-08-25 (§39)** for the frame — a CRS *is* declared, in the document's own `note`; `world_crs()` now reports it in-app. Reprojection stays open |
| **VA-01** | **Backlinks · unlinked mentions** | CIVIL ▸ Settlements ▸ Not built, and Data ▸ *Missing & orphan notes…* | Both need a reverse index over the whole vault. The provider deliberately opens only the files it is asked for, which is what keeps a large vault cheap to browse and is exactly what an unbounded scan would undo | **CLOSED 2026-08-25 (§42)** — the choice the sharpening poses is a false pair: a `stat` is not a read, so the index is persisted **and** kept correct per file by `(modified, len)`. Mentions are filtered by a 64-bit word fingerprint that stores no prose, so only candidate files are opened |
| **VA-02** | **Create notes from template**, path convention `Settlements/{name}.md` | CIVIL ▸ Settlements ▸ Not built, and Data ▸ *Create notes from template…* | `cartalith-vault` attaches to notes that already exist and refuses a heading that does not — deliberately (`MARKDOWN_VAULT_SCOPE.md` milestone 1's boundary). There is no note *creator* and no template registry, so the owner's own `design/vault-templates/` cannot be instantiated | **CLOSED 2026-08-25 (§39)** — the boundary quoted is about *editing*, and creating a file cannot destroy one |

`VA-` is a new prefix: the vault's *gaps*, distinct from `KV-` in §35, which
records what the vault subsystem **connected**.

> **Nine of these fifteen closed on 2026-08-25 — see §39**, which also records
> that **four of the nine had working engine capability the whole time** and
> that the stated reason above is factually wrong for CV-21, WW-14, WW-15 and
> (as of §36) CA-16. The table is left as written rather than rewritten in
> place: the reasons are the record of what was believed, and §39 is the
> record of what was true.

### What was wired to real capability, not disclosed

Worth separating, because a menu pass that only produces gap IDs has not done
its job:

- **CIVIL ▸ Routes & ways / Travel / Trade** — INFRA's Roads, Ports, Logistics
  and Trade content, re-parented into v3's three names, with every live readout
  intact (per-tier tally, longest-ways ranking, sea lanes, hand-drawn ways, the
  committed-routes list, the journey list and planner). **Rivers** left CIVIL
  for WORLD ▸ Hydrology, which is where v3 puts the river network; its one
  honest finding (IN-01, no `get_rivers()`) travelled with it.
- **CIVIL ▸ Territories** — *Recalculate territories* and *Generate provinces*,
  both live shortcuts onto `civ_recompute()` (§31's CV-20 fix), now under a
  category named for what they do rather than buried under Politics.
- **CIVIL ▸ Settlements ▸ Linked notes** — the vault entry point, scoped to the
  settlement roster above it, keyed to `tid`.
- **CARTO ▸ Visibility / zoom ▸ Data overlays** — one button onto
  `layers_popover.gd`, the shell's existing analysis-field picker, with a live
  count of what this build's `debug_layers()` offers. Deliberately *not* a
  second copy of that list: two pickers over one `set_debug_layer()` is the
  shape this shell keeps having to undo.
- **WORLD ▸ Generate** — Generate world / New seed / Center landmasses as
  shortcuts onto `app.gd`'s own handlers, beside the surviving pipeline-status
  readout. Same one-owner discipline.
- **Data ▸ Markdown vault** — v3's `Vault` menu, folded into Data per the
  owner. One live row onto `vault_window.gd`, whose tooltip states where v3's
  frontmatter-mapping and sync-direction rows actually live (they are a
  per-write choice in the panel, not a global setting), plus VA-01 and VA-02 as
  `_todo` rows.

### Verification

`_v3menu_probe.gd` / `.tscn` (temporary, untracked), run **windowed** — a
headless boot proves the extension loads and the scripts parse, which was never
the half at risk. It boots the real app, generates a real world (384×288, seed
483920: 233 settlements, 8 provinces, 35 ways), and then:

1. Asserts each rail's L2 list is **exactly** v3's list in v3's order, and that
   none of the nine retired category names survives anywhere. That second check
   earned its place: the first cut built INFRA's five old categories *and* the
   three new ones, because `_dock_hosted` was set inside `build_ways_into()` —
   which runs **after** `setup()`, and `setup()` is what runs `_build()`. Both
   flags now go in before `setup()`, next to `_nested`.
2. Opens all 33 categories and asserts each drew something. An accordion hides
   its own bugs: a category that throws while building leaves an empty body
   that looks exactly like a closed one.
3. Asserts every disabled control carries a reason — the `_todo()` contract —
   with the four state-gated Sculpt/Paint Commit/Discard buttons named as
   explicit exemptions rather than pattern-matched past.
4. Drives the rows that claim capability: the Politics/Simulation split (years
   under one, the simulator under the other, neither under both), Territories'
   recompute shortcut pressed for real, the Layers/Political display split, and
   `Data ▸ Markdown vault` pressed through the real popup and asserted by the
   vault window being on screen afterwards.
5. Screenshots each rail with every category open, then each re-parented
   category alone.

**PASS, 0 failures**, plus a visual pass over the shots. `cargo check -p
cartalith-godot` clean — no Rust changed; this was entirely a shell pass.

One defect the parse check caught before any of that, and the reason this pass
was resumed rather than restarted: `_build_simulation()` assigned an undeclared
`_sim_body`, left mid-refactor by a session-limit kill. The fix is not the
declaration — it is that `_rebuild_timeline()` now refills **both** bodies,
each guarded independently, so the order `_build()` claims them in cannot leave
one empty.

## 38 · FR-02, PE-01, SH-11, WW-13 — three of the four were a control that lied about its own state (2026-08-25) — **FIXED**

A conformance sweep over the windows and the shell's cross-references, run the
way this session keeps finding things: by driving the shipped app. Four fixes,
two of them defects nothing in the repository had recorded, and one whole class
of stale pointer left behind by §37's re-parenting.

### FR-02 — selecting a faction renamed it

**The worst one, and silent.** `faction_roster_window.gd`'s inspector commits
the name field on `focus_exited`. Removing a focused `Control` from the tree
releases focus and fires that signal **synchronously**, so `_clear()` — the
first act of every rebuild — was itself an edit.

The list rows are `FOCUS_NONE`, deliberately, so clicking one does not take
focus off the name field. Their handler then runs `_selected = fid` *before*
`_rebuild_inspector()`. So the teardown wrote the **previous** faction's name
onto the faction the user had just selected.

Measured on a real 6-faction world, not reasoned about:

| | roster |
|---|---|
| before | `1:Aurelia, 2:Veldmark, 3:Korrath, 4:Sythe Dominion, 5:Mirelle, 6:Draumr League` |
| after clicking Veldmark with Aurelia's name field focused | `1:Aurelia, `**`2:Aurelia`**`, 3:Korrath, 4:Sythe Dominion, 5:Mirelle, 6:Draumr League` |

No prompt, no status line, no undo. Two factions called Aurelia.

### PE-01 — the place editor's ⟳ never took on its first press

Same mechanism, visible instead of destructive. `DCC_SHELL_SPEC.md` §4.5.3 has
`open_for()` focus the name field, so on desktop it holds focus for the whole
session. `⟳` re-rolls, then rebuilds; the rebuild's teardown wrote the
pre-roll name back before the rebuilt field ever read the new one.

Isolated three ways on the same settlement, which is what separated the
mechanism from the engine:

| path | result |
|---|---|
| A — open (focused), one press | `Yusnashharwell` → `Yusnashharwell` (no-op) |
| B — open, `release_focus()`, one press | `Yusnashharwell` → `Abedomarmarch` |
| C — `civ_reroll_settlement_name()` direct, ×10 | ten distinct names, every one read back correctly |

Only `open_for()` grabs focus, so **presses two onward worked** — which is
exactly why this survived to be found by a probe rather than by the eye. The
same file's history `TextEdit` had the cross-entity form of it: `open_for()`
sets `_index` and only then rebuilds, so re-opening the editor on another
place from the map committed the old form's text onto the new one.

**The fix is two halves, because a guard alone silently drops real edits.** A
`_rebuilding` flag set across `_clear()` and checked by every `focus_exited`
commit in both files stops the teardown write; `_commit_focused_field()`,
called before the id moves, releases focus so a pending edit lands on the
entity it was typed for. Verified: the roster is unchanged across a selection
switch, the first ⟳ renames, a sentinel typed into settlement 6's history does
not appear on settlement 7 — **and settlement 6 keeps it**, so the flush works
and the guard is not merely swallowing input.

The other five `focus_exited` commits in the shell were checked and are safe:
`cartography_workspace.gd`'s two capture `idx` in the closure, so a teardown
write goes to the right label; `asset_library_window.gd`'s pack fields are
built once and never rebuilt under the caret; `travel_library_window.gd`'s
writes to a local draft; `vault_window.gd`'s reader is not id-switched.

### SH-11 — the zoom pivot, closed

Open since 2026-08-24, deferred only for file contention that no longer
exists. `_zoom_at()`'s maths was always right; its two `_input` callers handed
it the wrong space. `InputEvent.position` is window-relative, `_camera.position`
is `ViewportHost`-relative, so the pivot was out by `global_position * (1/z0 -
1/z1)`.

Measured rather than inferred: `global_position` (412, 70), one wheel notch
(zoom 1.6727 → 1.9236), **32.59 px of drift** — and the *same* (32.13, 5.46) at
three different probe points, which is the signature of a constant offset
rather than a pivot error, and exactly what that formula predicts.
`zoom_step()`, which passes `size * 0.5` and was always local, measured
**0.00 px** on the same run and is untouched. After the fix: 0.00 px at all
three points.

### WW-13 — Paint Commit / Discard, closed

Open since 2026-08-24, deferred to keep that commit off a fourth file.
Both buttons gated on `paint_painted_counts()["total"]` — the composite of
committed *and* pending — which a commit does not change, so both stayed live
over an empty draft afterwards. New `PaintEditor::pending_stamps()` /
`paint_draft_count()` counts what `commit_all` would bake and `discard_all`
would throw away, across all three drafts (a layer switch does not discard the
layer left behind). Two Rust tests pin the divergence the fix turns on.

The two Commit controls — the WORLD dock's and the tool-options bar's — draw
over one draft and are on screen together, so each now refreshes the other;
without that, WW-13 simply reappeared one control over.

Measured, driving real dabs:

| state | pending | composite | dock Commit | dock Discard | bar Commit |
|---|---|---|---|---|---|
| before any dab | 0 | 0 | disabled | disabled | disabled |
| after 2 dabs | 2 | 70 | enabled | enabled | enabled |
| after dock commit | **0** | **70** | disabled | disabled | disabled |
| after a further dab, committed from the tool bar | 0 | 74 | disabled | disabled | disabled |

The unchanged composite across a commit is the premise, asserted rather than
assumed.

### Six stale pointers, and one disclosure that vanished

§37's re-parenting renamed nine categories. Three follow-up renames were caught
then; six were not, found here by extracting **every** rendered `A ▸ B` string
and checking it against the structure that shipped:

| where | said | is |
|---|---|---|
| `app.gd` atlas-cache tooltip | `WORLD ▸ Finalize` | `WORLD ▸ Generate ▸ Finalize` |
| `menus.gd` ×2 (comments) | same | same |
| `right_dock.gd` sculpt stamp empty state | `World ▸ Sculpt` | `World ▸ Terrain ▸ Sculpt` |
| `place_editor_window.gd` polity tooltip | `Politics ▸ Recalculate territories` | `Civilization ▸ Territories ▸ …` |
| `infrastructure_workspace.gd` way **and** route commit toasts | `Roads ▸ Hand-drawn` | `Civilization ▸ Routes & ways ▸ Hand-drawn` |

The toasts are the ones that mattered: they fire at the exact moment a user
goes looking for what they just drew.

**And `rivers_note()` had no caller at all.** §37 wrote it so IN-01 (no
`get_rivers()`, so v3's per-reach river rows have no entity to hang on) would
travel with the Rivers category to `WORLD ▸ Hydrology`, and §37 above says it
did. It did not: CIVIL stopped drawing it and WORLD never started, so for a day
the finding existed in the source and nowhere in the app. It is `static` now,
with one owner and one caller.

### The class behind them: a jump button that only did half its job

Every `→ Civilization ▸ Territories`-style button switched the rail and
stopped. Survivable when CIVIL had six categories; v3 gave it fourteen and
CARTO ten, in an accordion that opens one at a time, so the odds of guessing
right went from poor to negligible. `Workspace.open_category()` and
`DccShell.select_domain_category()` do both halves and `push_warning` on a
title that no longer exists — so the next rename is loud, not silent.

### Two disabled controls with no stated reason

The `_todo()` contract is that a greyed control says why on hover. A scan of
every window found two that did not, both disabled at build time and given
their reason only by a refresh that had not run: the welcome screen's
**Open selected** (its primary action), and the asset library's three anchor
chips before any slot is selected. Both now carry the general form of the
fact.

### Verification

Six temporary, untracked probes (`_deadwire_probe`, `_pressall_probe`,
`_conform_probe`, `_jump_probe`, `_reroll_probe`/`_reroll2_probe`,
`_focusbug_probe`), all run **windowed** against a real 384 × 288 world, seed
483920 — 233 settlements, 6 factions, 35 ways.

- **Unwired-control scan** over 14 windows and the docks: every interactive
  control checked for a connection on its activation signal, counting the
  alternates (`text_changed` for a live filter, `color_changed`, `toggled`,
  `item_activated`). **0 genuinely unwired** — the two reported are a
  `TabContainer` (Godot switches its own tabs) and a read-only `TextEdit`.
  **0 disabled-without-a-reason**, after the two fixes above.
- **Press-every-enabled-button**, one window at a time, snapshotting every
  rendered string in the whole app before and after each press, with
  destructive and blocking labels skipped by name. Of six no-change presses,
  five are false positives (an already-selected tab or nav row, and
  `CityViewer`'s Fit, which moves a canvas the text snapshot cannot see). The
  sixth was PE-01.
- **Menu accelerators** re-enumerated after the v3 pass: 11, each matching its
  label, none unreachable except `Ctrl+Z` while the undo stack is legitimately
  empty — unchanged, and the top bar needed no correction.
- `cargo check -p cartalith-godot` clean; two new Rust tests pass; headless
  boot-check clean.

### Not fixed, and why

- **CV-24 and the rest of §37's fifteen** are unchanged — all still want
  design or engine work. *(Superseded 2026-08-25: **nine of the fifteen
  closed** in §39, four of them because the capability already existed. CV-24
  itself is unchanged and still wants a design.)*
- **ED-02** (an undo *history panel*) stays (C): no design exists.
  *(Re-checked 2026-08-25, §39: still (C). `undo.rs` keeps a label per step,
  so the data is there — §7.1 asks for a ledger, not a five-row list.)*
- **The Data manager's five silent nav rows** — `Heightmaps`, `World Data`,
  `Assets` and the two Export rows carry no tooltip, but each opens a real
  pane that explains itself, so nothing is being hidden. Left alone rather
  than given filler.
- **§14.4's own status line is stale** and is corrected here rather than
  rewritten in place: **JP-VS-01 is closed**, by §27's `open_journey_planner()`
  → `select_domain("civilization")`. Its entry still reads "still open".

## 39 · §37's fifteen, worked — nine closed, and four of those nine were already built (2026-08-25)

The owner's instruction was to implement §37's backable items. §37 registered
all fifteen as "no backing capability", and this session's standing lesson —
that a capability the register calls missing has repeatedly turned out to
exist and be unwired — was applied first: **every one of the fifteen was
checked against the crates before anything was written.**

That check is the finding. **Four of the nine closed had working engine
capability already**, and one more had it in a form the register described
inaccurately. Only two are new ports and two are genuinely new code.

| # | Disposition | What was actually true |
|---|---|---|
| **WW-14** | **closed — was already built** | Both halves. `build_npp` is the Miami model, ported and golden-verified; `cartalith_civ::wildlife` is a fauna model with guild rosters and per-species populations. §37 said "no crate computes either, here or in the reference" |
| **CV-21** | **closed — was already built** | `FactionEntry::color` existed and **nothing read it**. §37 said the roster "stores no colour field" |
| **CA-19** | **closed — was already readable** | `debug_layers()` has always carried all fifteen `CART_BIOME_COLS` classes as the Biomes legend. §37 said there was "no `#[func]` to read or rewrite an entry"; *rewrite* is the real gap and it is not small |
| **WW-15** | **closed — was already declared** | The GeoJSON writes `geojson::CRS_NOTE` in its own `note` property, quoted verbatim from the reference. §37 said the export has "no CRS declared" |
| **CA-16** | **closed — reference port** | §37's stated reason (one hardcoded colour per type) had been fixed by §36 five days earlier. The real gap was `#civWayScaleR`/`#wayOpacityR` |
| **CA-17** | **closed — reference port** | `#territoryOpacityR`. The rest of the row is CV-23's data gap, not a control gap |
| **CA-18** | **partly closed — reference port** | `CIV_LOD_ROAD` is a real per-type zoom ladder and is now ported. The declutter budget and per-layer ranges stay open |
| **CV-22** | **closed — new, and exactly the size §37 estimated** | One `EntityKind` variant, two match arms, plus export rows |
| **VA-02** | **closed — new** | §37's reason was a boundary about *editing*, applied to *creating* |
| CV-23 · CV-24 · CV-25 · CV-26 · IN-13 · VA-01 | **still open** | Sharpened below. Three want a design decision, three want real engine work |

### The four that were already built, and why each was missed

Worth separating, because the same mistake produced all four and it is
cheap to avoid: **§37 was written during a large UI restructure and asked
"does the dock have a control for this?", which is a different question from
"does the engine have the quantity?"** Every one of these four answers no to
the first and yes to the second.

**WW-14** is the worst of them, because the register asserts a negative about
the reference too. `cartalith_civ::build_npp` is the Miami model — the lower
of a temperature and a precipitation ceiling, both capped at 3000 g/m²/yr —
and it has been golden-verified since the wildlife port. It was computed
**only inside `wildlife_regions`**, as one of the ecoregion scorer's five
inputs, and thrown away. The fauna half is `cartalith_civ::wildlife`'s
connected-component segmentation of the Cartalith biome grid, with a guild
roster and a population estimate per species, reachable only by clicking the
map while the Wildlife debug view happened to be open. Measured on the
verification world: mean **801 g/m²/yr** over 88,629 land cells, peak
**2590.7**, **70 ecoregions**, **235 species records**.

**CV-21** is the sharpest illustration. `FactionEntry` has carried a `color`
field since the roster bridge was written; the only thing that ever read it
was a unit test. The renderers went to `lib.rs`'s `FACTION_RGB` by index
instead — and *inconsistently*: `build_territory_texture` used
`faction_rgb`'s no-wrap rule while the Political-control analysis field
indexed `FACTION_RGB[(owner-1) % len]` directly, so on a seven-faction world
the field drew faction 7 in faction 1's colour and the map did not. One
`CivData::faction_rgb` is now the only path, so the two cannot disagree.

The override is a **second** field rather than a write into `color`, and that
is deliberate: `color` holds the *reference's* `CIV_FACTIONS` table, which
this port does not render in (`FACTION_RGB` is Okabe-Ito, colourblind-safe, a
divergence disclosed at both ends since it was made). Writing the override
into `color` would have made the reference table the render palette for
edited factions and not for unedited ones — two rules in one roster.
`color_override: None` is exactly today's behaviour, so a world at rest is
bit-identical.

**CA-19**: `debug_layers()`' `bclass` entry carries `(r, g, b, label)` for all
fifteen classes, which *is* the biome colour table, and `get_paint_palette`
plus `paint_bridge::swatch_color` read the same constant. So CARTO ▸ Colours
now points at that one legend rather than drawing a second copy — this shell
has been bitten by two pickers over one thing before. What is genuinely
missing is a **writable** palette, and that is not small: `render.rs` is
`#[path]`-included standalone by five test targets so it cannot reach shared
mutable state, `CART_BIOME_COLS` is what a painted biome cell blends toward
in `land_color`'s hot loop, and `paint_blend.rs`'s goldens are written
against the frozen values. Scope: a palette field on `RenderCtx` threaded to
`land_color`, a runtime table on `WorldGen`, and re-baselined goldens.

**WW-15**: RFC 7946 **deprecated** the `crs` member, so a `note` is the
declaration a GeoJSON file gets to make, and this port has always made it —
`CRS_NOTE`, quoted verbatim from the reference so a consumer learns the same
thing from either implementation. What was missing was any way to read the
frame *in the app*, and the frame is real and is **two different frames**:

- **World mode** — the grid wraps in X and rows run 90°N to 90°S. A plate
  carrée graticule over a whole planet; `climate.lat_n`/`lat_s` are ignored,
  as the climate pipeline's own `lat_of(y)` says.
- **Regional mode** — rows run `lat_n` to `lat_s` and X does not wrap, so
  latitude is real and drives the climate model while longitude is not
  modelled at all.

`world_crs()` reports both, plus cell size, degrees per row and the export's
own note. Measured on the verification world: regional, 55.0° to 5.0°, 384 ×
288 cells over 2400 × 1800 km, **6.25 km per cell, 0.1742° per row**. What
stays open is a **projection**: nothing reprojects, so the planar kilometres
are not a projection of the latitudes beside them — which is precisely what
the export's note already warns.

### The two reference ports, and the one thing §36 had already fixed

**CA-16.** §37's stated reason — *"`map_overlay.gd` draws every way with one
hardcoded width-and-colour pair per type"* — describes the state **before**
§36, five days earlier, which replaced the flat `ROAD_COLOR` with the
reference's five two-stroke styles. The register row was written against a
file that had already changed. What was genuinely absent is the reference's
own two per-layer style controls: `#civWayScaleR` (line 1485), the third term
of `rsc = max(1, GW/512) · _civZoomK() · _civWayScale()`, and `#wayOpacityR`
(line 1491), its `globalAlpha`. Both are now real and both are the identity
at their defaults, so the layer at rest is unchanged. The scale multiplies
dash lengths too, because the reference writes one `rsc` into both
(`setLineDash([1.8*rsc, 1.3*rsc])`) — a wider road gets a proportionally
longer dash rather than a wide line chopped into the same fine ticks.

Per-class colour, casing and dash pattern stay unbuilt, and the reason is now
sharper than "no style argument": the five styles are *ported literals* whose
whole job is to make a track read as a track, and making one editable means a
style record keyed by way type for the overlay to read instead of its own
`WAY_STYLE` constant.

**CA-17.** `#territoryOpacityR` (line 1490), applied at 15440 as
`Math.round(opacity * 255)`. This port had it as a hardcoded `82/255` in
`build_territory_texture`. The default deliberately stays this port's own and
not the reference's `130/255`: there is a hillshade, a splat and a colour
grade under this wash that the reference's flat biome fill has not, and a
heavier tint buries them. Measured: `0.322 → 1.000 → 0.102`, monotone.

**CA-18, partly.** `CIV_LOD_ROAD` (line 15380, read by `_civWayLodMin` at
15012) is a real per-type zoom ladder and is ported verbatim. Its effect here
is narrower than there and is **deliberately not widened**: `ViewportHost`'s
`ZOOM_MIN` is 0.4, so `road`'s 0.35 threshold is unreachable and only `track`
and `ancient` ever drop out, between 0.4× and 0.7×. The two trunk tiers are
`0` there, meaning "always", not "missing". A switch was added that the
reference does not have, because a per-layer zoom range whose effect you
cannot see is indistinguishable from a bug.

### The two that are new

**CV-22** cost exactly what §37 estimated: one `EntityKind::Faction` variant,
one `as_str` arm, one `parse` arm — which is what `links.rs`'s own module doc
reserved (*"adding a variant here plus a `key()` case is that whole change"*)
— plus the export-registry rows a faction can fill. `ALL` gained a sibling
`EVERY`, because a faction has no position of its own and the three *place*
kinds do. §20's rule holds by construction: `entity_values` returns an empty
map for faction 0 (Unclaimed) and for an id past the roster.

The three vocabulary fields (Culture, Government, Religion) are exported
deliberately. `ECONOMY_SCOPE.md` found nothing in either codebase simulates
Government or Religion — which is precisely the argument for writing them
into a note, where an author's own prose about them is the thing that carries
meaning.

**VA-02.** §37's reason was that `cartalith-vault` *"attaches to notes that
already exist and refuses a heading that does not — deliberately"*. That
boundary is about **editing**: §23 makes the machine block the only thing
Cartalith rewrites unattended, because a tool that reshapes an author's prose
is the failure the whole design is arranged against. Creating a file that was
not there is a different act and a safe one — an existing path is refused
outright, so it cannot destroy anything — and the body is the author's own
template copied verbatim with nothing but the entity's name substituted.

**Templates come from the vault, not from the program.** No registry, no
bundled content: a `.md` with "template" in its path is a template, which is
exactly how the owner's own `design/vault-templates/` names its files
(`Settlement Template.md`, `Landmark template.md`, `Region Template/`).
Compiling a template set into the binary would be Cartalith telling an author
how to write their notes. `discover` filters the same bounded listing the
file picker already walks — no second walk, still no file opened — and labels
a nested template by its folder, because that corpus has two byte-identical
`Landmark template.md` files and a picker showing both under one word offers
the same thing twice.

`fill_title` substitutes `{{…Name}}` and the literal `[Name]`, and leaves
`[If applicable]`, `[Optional]` and every other bracketed prompt alone: those
are instructions to the author, and answering them would be Cartalith
answering a question it was not asked. Path convention is v3's
`Settlements/{name}.md`, generalised per kind and editable in the field.

### Still open, sharpened

Each of these was checked against the crates too. What changed is not their
status but what is known about them.

- **CV-23 — borders, claims, influence.** Still (B) large, but **the influence
  field is computed today and thrown away**: `assign_territory`'s
  `best_effective` (`cartalith-civ/src/lib.rs:6040`) is the per-cell
  cost-distance to the winning capital, divided by that capital's population
  weight. A *contested* value is one more array — the runner-up from a
  different faction — not one more Dijkstra. Two real obstacles, both now
  named: retaining an `f32` per cell is **268 MB at this port's 8192²
  ceiling**, the same objection `civ_continents` already records, so it has
  to be an on-demand recompute like `wildlife_regions`; and that recompute
  needs `build_travel_cost`'s `cost` field, which `compute_civilisation`
  builds as a local and frees, keeping only per-settlement samples in
  `explanations`. Historical occupation over time is separately absent and
  is timeline work, not territory work.

  > **Narrowed by §41 (2026-08-25).** Every sentence above held up, including
  > both obstacles, and §41 is what it prescribed: the runner-up is one more
  > array, the field is built on demand like `wildlife_regions`, and the
  > `cost` field is *rebuilt* rather than retained — it is a pure function of
  > the height field and sea level, both of which the raster's own borrow
  > struct already holds. Historical occupation stays open, exactly as
  > scoped here.
- **CV-24 — the year scrubber as program scope.** Unchanged and correctly (C).
  `TIMELINE_SCOPE.md` §4's standing instruction is to build a dedicated panel
  rather than guess the region, and `dcc_shell.gd`'s reserved `timeline_bar`
  is one fixed-height `HBox`. A shell-frame change; **owner's to specify.**
- **CV-25 — military.** Unchanged (C). Neither `cartalith-civ` nor the
  reference models garrisons, fortification networks or campaigns. This is
  feature design, not wiring, and improvising it would be inventing a game
  system. What exists is per-settlement *defensibility*, a terrain heuristic,
  on the right dock.
  > **Superseded by §40 (2026-08-25).** The second sentence is wrong. The
  > reference models the fortification half twice over — `_umWallSpec` /
  > `_umInferWalls` at 22109 and `_civPlaceDefensibility` at 23802 — and this
  > port had already ported the per-faction half (`power.military`) without
  > noticing it was one. The *campaign* half is the only part that was ever
  > absent.
- **CV-26 — relationships.** Unchanged (C), and for a structural reason worth
  restating: there is no **edge** between two factions to hold a value, at any
  year, so a diplomatic matrix would be a grid of blanks. The reference's own
  inspector says "not yet implemented" here too. **Owner's to specify.**
  > **Narrowed by §40 (2026-08-25).** This diagnosis was right, and it is
  > what §40 built: the edge exists now. What stayed out is the half that
  > really does need specifying — actions, treaties, and change over time.
- **IN-13 — trade flows.** Still (B) large, and now precisely: `TradeBalance`
  is `{exports, imports}` — a per-settlement verdict on which of the fifteen
  resources a place has too much or too little of *against the world mean*.
  It names **what**, never **who**. Turning it into a flow needs a
  surplus-to-deficit match across settlements and then a routing of that match
  over the way graph — a bipartite assignment plus a network flow, neither of
  which exists in either codebase. `ECONOMY_SCOPE.md` holds the aggregation.

  > **Closed by §42 (2026-08-25), and the last sentence is wrong.** Both
  > exist in the reference: `_civFoodShed` (24050) enumerates every other
  > settlement as a candidate supplier, and `_civFoodConnected` (24044)
  > filters them through `_civRoadComponents` (24076), a union-find over the
  > way network's own endpoints. The reference runs that match for one good
  > and wrote `_civGoodReach` to classify twenty-two; §42 runs it for the
  > fifteen `TradeBalance` ranges over, and invents one rule — the
  > allocation — which is stated at the function.
- **VA-01 — backlinks and unlinked mentions.** Still (B) large, and the open
  question is the **index, not the scan**: built on demand it stalls a large
  vault, and persisted it is a second store to keep in step with a folder the
  user edits outside Cartalith. The provider's deliberate "open only what you
  are asked for" is what keeps browsing cheap, and an unbounded mention scan
  is exactly what would undo it.

  > **Closed by §42 (2026-08-25).** The question was the right one and the
  > two options it offers are a false pair: `FsVault::meta` returns
  > `(modified, len)` **without opening the file**, so the index is
  > persisted *and* kept correct per file, and a refresh over an untouched
  > vault opens nothing at all. The mention scan stays bounded by a 64-bit
  > word fingerprint per note — eight bytes, no prose — which narrows the
  > vault to candidates before a single file is read.

### Also carried over from §38

- **ED-02 (undo history panel)** — checked, **still (C)**, and the reason is
  unchanged rather than merely restated: `undo.rs` keeps a `label` on every
  step, so a list of the five is data the engine already holds and
  `undo_stats()` reports only the *next* one. §7.1's own box says the panel
  moved from "needs an engine" to "needs a design", and §7.1 asks for a
  *ledger* with per-subsystem reversal — a strictly larger thing than a
  five-row list. Building the five-row list would answer the register's easy
  half and foreclose the design question. **Owner's to specify.**

  > **Built by §42 (2026-08-25).** The judgement held up and this is the
  > thing it was protecting: the ledger records every commit and reverses
  > the ones it can, so the five-row list is a subset of it rather than a
  > substitute for it. Per-subsystem *reversal* is still unbuilt — what
  > changed is that a `Recorded` row already knows its subsystem, so
  > turning one on is a kind change rather than a redesign.
- **The Data manager's five silent nav rows** — checked again and **left alone
  again**, agreeing with §38. Each opens a pane that names itself in its own
  first line; a tooltip repeating the row's own label is filler, and this
  register's whole standard is that a disclosure has to say something the user
  could not already see.

### Verification

`_gap37_probe.gd` / `.tscn` (temporary, untracked), run **windowed** against a
real 384 × 288 world, seed 483920 — 233 settlements, 6 factions, 35 ways.
**PASS, 0 failures.** Every claim measured rather than reasoned about:

| what | measured |
|---|---|
| WW-14 | mean NPP 801 g/m²/yr over 88,629 land cells, peak 2590.7; 70 ecoregions, 235 species records; the `npp` raster drew and both jump buttons set the view they name |
| CV-21 | the territory wash and the Political-control field **both** moved on an identity colour and both returned on Reset; the picker wrote through the real window |
| CA-17 | wash alpha 0.322 (default) → 1.000 → 0.102, monotone |
| CA-16 | way opacity 0 moved **0.396 %** of screen pixels; width 2.5× moved **0.929 %** |
| CA-18 | at 0.5× zoom the ladder's single `track` (of 35 ways) is visible in the frame diff |
| CV-22 | four entity kinds; faction 1 offered 8 fields and produced a 636-character Cartalith block; faction 0 and faction 999 both returned empty |
| VA-02 | a real note written to a real folder from a real template, byte-identical on disk, the author's `[If applicable]` intact, and the duplicate refused with the file unchanged |
| WW-15 | regional frame, 55.0°→5.0°, 6.25 km/cell, 0.1742°/row, and the export's own note read back in-app |

Plus `cargo test`: `cartalith-godot` 343 lib tests and its integration
targets all green with one new roster test; `cartalith-vault` **41 → 48**,
including one end-to-end against a real folder that proves the created note
is attachable by the ordinary path — and therefore a real note and not a
special one. `cargo check -p cartalith-godot` clean; headless boot clean.

**One defect the parse check caught before any of it**, and the reason the
shell's own conventions earn their keep: the first cut of CIVIL ▸ Factions
added a second *Faction roster…* button to a category that already had one,
which GDScript rejects outright as a redeclared local. Two openers onto one
window is the shape this shell keeps having to undo, and this time the
language refused it.

**A note on the commits.** WW-15's engine and shell landed in `79396c2`
alongside the other six display IDs; that commit's message does not name it,
and this section is the record.

## 40 · CV-25, CV-26 — the military half was a port nobody had recognised, and the relations half needed an edge that did not exist (2026-08-25) — **NARROWED**

Owner's decision on the two §37 IDs that were parked pending design: *"build a
minimal version now"*. Both are built. Neither is closed, because in each case
a real, separable feature remains — and both categories say so on screen, in
the same words this section does.

The two turned out to be completely different jobs, and the difference is the
finding worth keeping.

### CV-25 was a port, and the register's reason for calling it a design was wrong

§37 recorded: *"`cartalith-civ` models none of them and neither does the
reference."* The second half is wrong, and this is the **fifth** §37 entry to
be wrong in exactly that direction. Grepping the frozen snapshot for
`military`, `garrison`, `war` and `fortif` found three real implementations
and one already-ported fourth:

| reference | line | what it is | status before today |
|---|---|---|---|
| `_umWallSpec` | 22109 | the `none · ditch · palisade · stone` ladder, from tier + function + threat + wealth + age + command of ground | **not ported** — `urban_adapter.rs`'s own table said "skipped: the whole fortification pipeline is milestone 10" |
| `_umInferWalls` | 22134 | its boolean view | **not ported**, same note |
| `_civPlaceDefensibility` | 23802 | per-settlement defensive strength `0..1` | **not ported** |
| `_civFactionAggregates` → `power.military` | 23716 | `0.45·normPop + 0.35·fortifiedFraction + 0.20·capitalTierNorm` | **already ported**, golden-verified, and with no reader |

So the "new design" was three small ports and a category to put them in.

**And the port had a live defect the ports themselves exposed.**
`FactionPlace::from_settlement` hard-wires `fortified: false`, because
`cartalith-civ` is stateless and the `umWalls` override lives at the boundary
in `place_extras`. Every caller of `civ_faction_aggregates` in this workspace
has therefore been feeding the military axis a **constant zero** for its
`0.35 · fortifiedFraction` term — a third of the formula, dead. The new
bridge composes the place rows itself (`um_infer_walls` per settlement, then
`FactionPlace { fortified, ..from_settlement(s) }`), which is exactly what the
reference's own aggregate pass does. Measured on seed 483920: de-walling one
faction's five settlements moves its military power **89.00 → 61.00**. That
term is reaching the formula now, and was not before.

It also gives `umWalls` and `umAge` their first consumer anywhere.
`civ_roster_bridge.rs`'s module doc has said outright, since ED-03 landed,
that an edited `umWalls`/`umAge` *"reaches nothing"*. It reaches this.

**What stays open, and only this:** garrison **headcounts**, campaigns, unit
movement, combat. The reference has none of them either, and none is derivable
from anything above — a headcount would be a fabricated number wearing a real
one's clothes.

### CV-26 was genuinely new, and the register's structural objection was the right one

§37/§39's diagnosis — *"there is no edge between two factions to hold a value,
so a matrix would be a grid of blanks"* — was correct, and it is what got
built: the edge, and nothing else. `cartalith_civ::relations` produces one
**symmetric** value per unordered faction pair, **derived and recomputed**
like the aggregates and the wildlife regions, stored nowhere and saved nowhere.

Four terms, each symmetric by construction, each reported beside the verdict
so the reader can disagree with it:

| term | weight | source |
|---|---|---|
| shared culture | `+0.30` | `civFactionCulture` |
| shared / opposed faith | `±0.20` | `civFactionReligion`; `none` on either side is silence, not division |
| trade complement | `+0.25` | the aggregate's own `imports`/`exports` |
| border friction | `−0.55` | shared-border cells × `(0.35 + 0.65 · rivalry)` |

Two of those deserve their reasoning stated rather than assumed.

**Friction is border × rivalry, not border.** A long border with a weak
neighbour is a frontier, not a rivalry; `rivalry` is high only when *both*
sides are strong **and** evenly matched, which is the configuration that makes
ground contested. And the border is measured against **the widest border on
this map**, not an absolute cell count — the same relative-not-absolute
discipline the reference's own v1.30/v1.32/v1.37 trade-balance fixes settled
on.

**A good nobody supplies is discounted.** The trade term's denominator counts
only imports that *some* faction on the map exports. This is not a
convenience: a deficit nobody can fill is a shared shortage, not a
relationship — the reference's own v1.33 finding, in its own words, that a
food deficit is not automatically an import *"when there is no direct trade
that could sustain [it]"* (line 24500). It matters concretely here, because
this port retains no `currentPopulationDensity()` equivalent (`CivData::dens`
is `civ_current_agrarian_density`, a **different field**, and substituting it
would silently move `foodProductionCapacity` off the reference's number). With
the food half of the balance absent, `food` lands in every faction's imports
and nobody's exports; without this rule it would have diluted every pair's
trade term toward zero.

**What stays open:** diplomacy actions, treaties, vassalage, and relations
that change over time. Each needs a decision this port should not make alone —
who acts, on what clock, and what a treaty does to the map. The value is a
reading of the world as it stands and stops there.

### Verification

Two temporary, untracked harnesses, both run against a real world, seed
483920.

`_military_probe.gd` (engine level, 384 × 288, 33 settlements, 6 factions) —
**PASS**:

| what | measured |
|---|---|
| military power differentiated | 45.43 … 89.00 across six factions, not all-equal and not all-zero |
| the ladder is a ladder | 12 stone · 2 ditch · 19 none; defensibility 0.000 … 0.988 |
| `fortifiedFraction` reaches `power.military` | de-walling faction 1's five settlements: **89.00 → 61.00** |
| every pair, symmetric | 15 pairs for 6 factions; `get(a,b) == get(b,a)` |
| border friction is live | widest shared border 250 cells; values −0.168 … +0.125 |
| trade term is live | 0.00 … 0.67 across pairs, once the aggregate is given real resource rasters |
| culture and faith are wired, not dead | setting two factions to one culture and one faith moved their value **+0.125 → +0.625** (exactly the documented `+0.30 +0.20`); switching one to a different faith moved it back to `+0.225` with `religion_term −1.0` |

`_mildock_shot.gd` (shell level, the real `app.tscn`, 233 settlements, 6
factions) — **PASS, windowed and headless**. Both categories render real rows:

```
Veldmark -- 66/100 · 7 of 49 fortified
Korrath  -- 59/100 · 2 of 33 fortified
...
15 of 233 settlements are fortified.
Garnstokgrimfornward -- ditch wall · defence 99%
Tibtibmarcoctcastra  -- stone wall · defence 97%
...
Korrath ↔ Sythe Dominion -- friendly (+20)
Veldmark ↔ Korrath       -- wary (-34)
```

and both of the old "Not built" disclosures are gone while both **narrowed**
gaps are still stated on screen — the probe asserts all four, because closing
a register entry by deleting its honest note is the failure mode this document
exists to catch.

Plus `cargo test`: `cartalith-civ` **401** lib tests (11 new in `relations`,
9 in `military`) and a new `golden_parity_military.rs` (3 tests);
`cartalith-godot` 343 lib tests and every integration target green.
`cargo check -p cartalith-godot` clean.

**Two boundary assertions failed on the first extraction, and both were real
range errors** — `CLAUDE.md`'s "verify the line ranges before slicing" rule
earning its keep for the sixth time. The `_umWallSpec` slice started at 22105,
inside the v1.17 provenance comment rather than at the `function` line
(22109). And the four-rung assertion was written as four `return 'x';`
statements, which the reference does not contain: `palisade` reaches its caller
only through the ternaries `pop>=1200?'stone':'palisade'` and
`rank>=1?'palisade':'ditch'`. Both failed loudly instead of emitting a short,
plausible golden.

**One equivalent mutant, recorded rather than chased.** `terrainD>0.9` and
`terrainD>=0.9` are the same function: `1 − 4·|r − 0.35|` never evaluates to
exactly `0.9` for any `f64` `r` in the neighbourhood — the reachable results
step from `0.9000000000000001` straight to `0.8999999999999999`. The *constant*
is what carries the meaning, and `commanding_village_digs_in` pins it from both
sides instead. Mutating `0.9 → 0.8` fails; mutating `1200 → 1100`,
`260 → 250`, `0.6 → 0.65` and `rank>=3 → rank>=4` all fail too.

## 41 · CV-23 — the influence field was being computed and thrown away, and keeping it costs nothing resident (2026-08-25) — **NARROWED**

§39 sharpened this row and got it exactly right, on both halves. This section
is what it prescribed, plus the measurement it asked for.

### What was already true, and what one array bought

`assign_territory` (`cartalith-civ/src/lib.rs`) runs one `road_dijkstra` per
capital and keeps a running per-cell minimum of `dist / territory_weight(pop)`
in a local called `best_effective`. **That local *is* the influence field.** It
was being computed on every `generate()` and dropped on the last line, which
returned only the `i32` owner id.

Contested-ness is the runner-up beside it — the lowest effective distance from
a capital of a *different* faction — and §39's claim that this is "one more
array, not one more Dijkstra" is correct and now proved: it is a single extra
compare in the same per-cell branch, exact in one pass.

The invariant that makes one pass enough: `rival_effective >= best_effective`
always, because the runner-up is only ever written from the *outgoing* winner
at a change of owning faction (and the incoming winner is strictly smaller) or
from a candidate that already lost. So when a new faction takes a cell, the
value discarded is always `>=` the value kept, and the value kept belongs to a
faction that is by construction not the new owner. `influence_rival_is_the_true_runner_up_faction`
checks it against the brute-force per-faction minimum over a ragged
three-faction fixture rather than trusting the argument.

`territory_sweep` is now the one implementation both callers share, so
`assign_territory` and `territory_influence` cannot disagree about who owns a
cell — `influence_owner_matches_assign_territory` pins that over four layouts
(same-faction displacement, cross-faction displacement, a third faction that
never wins, and an unreachable half).

### The two obstacles §39 named, and how each was actually paid

**Memory — the owner's decision, followed.** Nothing holds an influence grid.
`sample_bridge::territory_influence` builds one when the layer is opened or the
readout is pressed, reads it, and drops it — the `wildlife_regions` shape,
named by the owner. `CivData` gained no field.

**The freed `cost` field.** §39 recorded this as the blocker on any recompute:
`compute_civilisation` builds `build_travel_cost`'s output as a local and frees
it. It is *rebuilt*, not retained, and that is honest rather than a dodge:
`build_travel_cost` is a pure function of the height field and the sea level,
and `FieldRefs` — the borrow struct every analysis layer already reads through
— holds both. Recovering it costs one parallel pass over the grid and **zero**
resident bytes. Leaking a `gw × gh` f32 back into `CivData` to avoid that pass
was the alternative, and it is the exact thing `MEMORY_OPTIMIZATION_SCOPE.md`
exists to prevent.

`territory_sweep` takes a `want_rival` switch for the same reason. With it
`false` the loop body is character-for-character what `assign_territory` ran
before, and allocates exactly what it allocated: the two extra grids the
runner-up needs are `Vec::new()`. **Generation pays nothing for a layer nobody
may open.**

### What got built

| | |
|---|---|
| `cartalith_civ::TerritoryInfluence` | `owner` (borders), `rival` (claims), `influence` (the kept `best_effective`), `contested` (`influence / rival_influence`, `0..=1`) |
| `cartalith_civ::territory_influence` | one sweep, one Dijkstra per capital, nothing cached |
| `WorldGen::civ_territory_influence()` | per-faction rows and per-*pair* border rows, plus `transient_bytes` / `resident_bytes` |
| Layers ▸ Civilization ▸ **Contested borders** | `LAYER_GROUPS`' own convention — a row, a hint, a five-swatch legend, a per-world `layer_available` check |
| CIVIL ▸ Territories ▸ **Borders & influence** | the numbers, behind a button, because the computation costs something |

**The raster invents no hue.** Every colour is a faction's own swatch
(`CivData::faction_rgb`, identity colours included), dimmed to `0.26` in a
secure interior and lifted to full strength on a frontier by `0.26 + 0.74·t²`.
Past `t = 0.88` — "the runner-up is within 12 % of the winner" — the cell
alternates with the *rival's* swatch on a three-cell diagonal stripe, so a
frontier reads as a two-colour weave naming both claimants. That is the claim
hatching CA-17 asks for, drawn in the analysis layer rather than in the map's
territory wash.

**The contested band is wider far from either capital, and that is the model
talking.** One step in from a border the winner's distance is `d` and the
rival's about `d + 2·step`, so the ratio is near `1` when `d` is large and well
under `1` when the border runs close to a capital. A frontier between two
distant centres genuinely is more evenly balanced than one at a capital's gate.

### What stays open

**Historical occupation over time**, exactly as §39 scoped it: the timeline
records settlement snapshots per year, not a per-year ownership grid. That is
timeline work, not territory work, and the Territories ▸ Not built note now
says only that.

### Verification

`_cv23_probe.gd` / `.tscn` (temporary, untracked), run **windowed** against the
same real world §39 used — seed 483920, 384 × 288, 233 settlements, 6 factions,
35 ways. **PASS, 0 failures.**

| what | measured |
|---|---|
| the field | 88,621 owned land cells, **14,225 (16.05 %) on a frontier**, mean contest 0.595, mean influence 43.6 |
| per faction | all six hold ground; reach 32.4 (Draumr League) to 51.5 (Aurelia); frontier share 1,425 to 3,202 cells |
| **claims** | **9 faction pairs actually meet**, mean contest 0.938-0.969 each — Veldmark ↔ Korrath the longest at 4,412 cells, Aurelia ↔ Veldmark the shortest at 60 |
| determinism | two on-demand rebuilds agree cell-for-cell |
| the Layers row | group `Civilization`, available, 5 legend swatches; the real popover row clicked and the map really switched |
| **borders read as contested** | the contested scalar recovered back out of the pixels (the ramp is invertible): **mean t = 0.960 at a border cell vs 0.551 in an interior cell, +0.410 apart**, over 1,480 border and 78,934 interior cells |
| the hatch | 2,461 frontier pixels carry the rival faction's own colour |
| the dock | the section renders, and the old "there is no contested-claim value" denial is asserted **gone** |

**Memory, measured rather than claimed.** The process working set (read from
Windows, not from Godot's own allocator, because every byte in question is
Rust's) across **25 consecutive rebuilds** at 384 × 288: `517.8 MB → 518.0 MB
peak → 513.8 MB`, a net **−4.0 MB**. Twenty-five calls that each kept their
field would have been +140 MB. At 1024 × 768 — 786,432 cells, 510,600 owned,
59,717 on a frontier, **343 ms**, a **39.8 MB** build — the process's own peak
working set does not move **at all** (`891.5 → 891.5 MB`) and the resident
figure comes back 127.5 MB *below* where it started.

`transient_bytes` reports the honest **peak**, not a flattering subset: 53
bytes a cell — 4 for the rebuilt cost field, 24 for the sweep
(`owner`/`best_effective`/`rival_effective`/`rival`), 25 for one capital's
Dijkstra at a time (`dist`, `prev`, `visited`, and the heap's own
`with_capacity(n)`). **41 of those 53 are what `assign_territory` already
spends inside `generate()` on the same world.** Opening this layer costs 12
bytes a cell more than generating the world already cost, transiently, and
`resident_bytes` is `0`. The register's 268 MB objection was about *resident*
state; there is none.

Stated plainly rather than hidden behind the flattering measurement: at the
8192² ceiling that same arithmetic is **3.3 GB transient** — against the
**2.6 GB** `assign_territory` already peaks at on that world inside
`generate()`. Both are enormous, and the ceiling is theoretical for the whole
civ pipeline, not for this row in particular. What this row adds is 12 bytes
a cell, freed, on a layer the user has to open.

## 42 · IN-13, VA-01, ED-02 — the last three open items, designed and built (2026-08-25) — **CLOSED**

The owner's instruction was two-part: *"use /ui-ux-pro-max in combination with
your /design skill to create a proper menu for the missing items"*, then build
IN-13, VA-01 and ED-02 against those designs. §37's fifteen were down to three
genuinely open rows plus the narrowed remainders of CV-23, CV-25, CV-26,
CA-18, CA-19 and WW-15, all of which still showed as disabled controls and
bare notes rather than as designed surfaces.

The design is a five-artboard canvas in v3's own visual language — same
tokens, same category/section structure, same disclosure depth — covering the
three surfaces being built plus a consistent **anatomy for a "Not built"
row**, which is what the six narrowed remainders now share.

### IN-13 — the register's reason was wrong, for the sixth time this session

§39 sharpened IN-13 to *"a flow needs a bipartite match plus a network flow,
neither of which exists in either codebase."* The second half is false, and
the frozen reference says so in three functions:

| Reference | Line | What it is |
|---|---|---|
| `_civFoodShed` | 24050 | enumerates **every other settlement** as a candidate supplier — the bipartite match |
| `_civFoodConnected` | 24044 | filters candidates through `_civRoadConnected` … |
| `_civRoadComponents` | 24076 | … which is a **union-find over the way network's own endpoints** — the tie to the way that carries it |

Plus `_civFoodMode` (23997), `_civFoodDeliverable` (24004) and `_civGoodReach`
(24442), which decide per pair and per good whether the relationship is
possible at all and how much of it survives the distance. The reference runs
all of it for **one good**, `food`, and then — this is the part worth
recording — wrote `_civGoodReach` explicitly to classify *"the reach a given
good can achieve from this settlement"* across a bulk/luxury vocabulary of
twenty-two goods, and never used it for anything but display.

So `cartalith_civ::trade` is **five ports and one new step**: run the
reference's own match over the fifteen `CIV_RESOURCE_KEYS` that `TradeBalance`
already produces a verdict on, gated by the reach rule the reference wrote for
exactly that purpose. Constants ported literally: 160/880/8000 km doubling,
220/1600/9000 km reach cliff, the 50 km local supply radius, the 0.6 supplier
share.

**The one rule that is not a port**, stated at the function rather than
buried: a settlement's demand for a good is its **population** — the only
per-settlement scale this port holds that is not itself derived from trade
(`_civPlaceProsperity` reads `tradeVolume`, so using that would be circular).
It is split across reachable exporters in proportion to `_civFoodDeliverable`,
and each flow is capped at `SUPPLIER_SHARE × the supplier's own population`,
which is that constant's own sentence — *"one consumer never draws a
supplier's whole surplus"* — applied where the sentence is about. Demand the
cap leaves uncovered is not carried and does not reappear on another supplier.

**Two divergences, both in the module doc rather than silent.** Road
connectivity reads `Way::a_idx`/`Way::b_idx` instead of re-deriving them with
the reference's `nearest()` endpoint snap at `max(2, GW/50)` — this port's
consolidation tail already records which two settlements a way joins, so the
snap would re-derive something stored. And `_civPlaceNavigability` is ported
at branches (a) and (b) only: branch (c) reads `_umSiteProfile`'s
`coastDistKm`/`riverDistKm`/`riverOrder`, which here are locals inside the
layout builder's water context. It costs almost nothing — (b)'s
`um_site_kind_from_terrain` sweeps the *same* `um_water_reach_km` radius that
(c) thresholds against, and the reference's own comments call the site kind
"authoritative" over the traced polylines on both the coast and the river
branch.

**Nothing is stored.** `trade_flows` allocates, answers and drops, the way
`territory_influence` and `wildlife` do. `CivData` gained no field and the
save format is untouched. The shell holds the one result in `trade_store.gd`
so the dock, the place editor's ledger and the map overlay share one
computation; `app.gd` drops it on every world change, in the same function
that already re-runs every workspace's `on_world_changed()`.

**Three surfaces, and why the map one is not in the Layers popover.** CIVIL ▸
Trade carries the four rows v3 asks for in the disclosure ladder the design
settled on — world, good, pair, place. The place editor carries the
per-settlement ledger, because a partner is a *name* and only means something
next to the place it belongs to. The map draws way **thickness**, in CARTO ▸
Roads & routes: the Layers popover is the one picker for *field rasters*
(`set_debug_layer`), and trade load is a value on a way, drawn by the way
layer that section already owns. Width and not hue, because every faction
swatch is already spent on the territory wash and on contested borders, and a
way's colour is already its type (RD-02).

### VA-01 — the index is the question, and a stat is not a read

The register poses it as a choice between two bad options: *"built on demand
it stalls a large vault, and persisted it is a second store to keep in step
with a folder the user edits outside Cartalith."*

It is a false pair. `FsVault::meta` already returns `(modified, len)` **without
opening the file** — that is §14's own change-detection basis, and reusing it
makes the index persisted *and* correct:

| | |
|---|---|
| stored | per note: `(modified, len)`, its outgoing link targets, the entity keys of any Cartalith blocks in it, and a 64-bit word fingerprint. **Never the prose.** |
| built | only when a person presses Refresh. The first build reads every note once and says so first. |
| invalidated | per file, by `(modified, len)`. Ten edits in Obsidian cost ten reads. |
| never | a watcher, a background thread, or a scan nobody asked for. |

**Unlinked mentions without storing anyone's prose.** A mention appears to
need the text; it does not need the text *stored*. `NoteRecord::word_bits` is
a Bloom filter over the note's word tokens — eight bytes a note, from which no
word can be read back (asserted, by serialising a record and searching it for
its own words). `mention_candidates` returns the notes that *could* contain
every token of a name; the session then opens **only those** and confirms with
a real search. False positives cost one read; **false negatives are
impossible**, which is the property that matters and the one the tests pin.

**An entity finds its incoming notes by three routes**, and the third is the
one an index of note-to-note links alone would miss: its own linked notes'
reverse map, the notes that link to them — and every note carrying
`entity="settlement:42"` **directly**, which finds the entity even when it has
no note of its own, because a province's note can describe a place nobody has
written a page for.

`broken_links()`/`orphans()` fall out of the same index, which is exactly what
`Data ▸ Missing & orphan notes report…` has been disabled waiting for. One
index, one panel, not two walks — and that row is live now.

The index is saved in **its own file** (`user://markdown_vault_index.json`),
not as a key inside the link store: §5's store is portable project data, and
this is a cache of one folder on this device that is rebuilt in a single press
if it is lost.

### ED-02 — a ledger, which is what §7.1 asked for

§39 recorded that a previous pass *"deliberately declined to build a flat
five-row list because that would answer the register's easy half and foreclose
the design question."* That judgement held up, and this is the thing it was
protecting.

`undo::HistoryLedger` **records every commit and reverses the ones it can**,
saying per row which it is:

| Kind | Glyph | Meaning |
|---|---|---|
| `HeightSnapshot` | `▲` | a pre-operation height field is held; reverting is real |
| `Recorded` | `·` | it happened; no snapshot exists, and the row carries the **specific** reason |
| `Floor` | `◼` | a generate or a load; history starts here |

**The two structures are deliberately not cross-wired.** A height row is
reversible exactly while its snapshot is on the stack, and the stack evicts on
its own byte budget — so `rows()` takes the live `HeightUndo::depth()` and
marks the newest that many height rows live. One source of truth for *"is
there a snapshot"*, asked at read time, so the two cannot drift; an evicted
row reports `false` with a reason naming the budget.

Linear only, per §7.1's own conclusion: reverting to a row pops every snapshot
above it and the row leaves with them, because the snapshot **is** the state
before that operation. Recorded rows above it go too — an operation whose
height field has just been rolled back out from under it is not still in
effect, and leaving it listed would be the worse lie.

Call sites: `sculpt_commit` and `carve_fjords` record a snapshot beside their
existing `undo.push`; `paint_commit` and `civ_territory_commit` record with
the reason (*"the pre-commit layer is not retained; Discard reverts an
uncommitted draft only"*); `generate` and `load_save` record a floor, which
clears the rows for the same reason `undo.clear()` already runs there.

The panel is a **right-dock context**, per §7.1 proposal 3 — selection-adjacent,
and a window for a list read *against* the map would cover the map. Two tiers,
which are this engine's own draft/commit seam: the open Sculpt draft above,
reversible in place by its own tool and deliberately not a row; then the
commits, newest first.

### The six narrowed remainders, and the anatomy they now share

Not rewritten — they were already honest. What the design added is a
consistent three-part anatomy (**the noun, named exactly** · **the blocker,
named specifically** · **what does exist instead**) and a state chip that
separates three genuinely different situations:

| Chip | Means | Which |
|---|---|---|
| needs a decision | nothing is missing from the engine; somebody has to say what the feature *is* | CV-25 (garrisons, campaigns, combat — narrowed again by §43, which built the manpower half the owner specified; §44 settled that model's one open question, the era bands' denominator), CV-26 (treaties, vassalage, change over time), WW-15 (reprojection) |
| blocked on | the design is clear and waits on named work elsewhere | CV-23 (historical occupation — timeline work), CA-18 (declutter budget — CA-04's separable layer stack) |
| costs a re-baseline | buildable today, and doing it moves golden expectations `DECISIONS.md` §7a protects | CA-19 (a *writable* biome table) |

A fourth state is deliberately absent: *coming soon*. Nothing here has a date.

### Verification

`_in13_probe.gd` / `.tscn` (temporary, untracked), run **windowed** against a
real 384 × 288 world, seed 483920 — 233 settlements, 6 factions, 35 ways.
**PASS, 0 failures.** Every claim measured rather than reasoned about.

**IN-13, the match itself**

| what | measured |
|---|---|
| the match | **624 flows** over 7 of the 15 goods in **1 ms**, 0.18 MB transient, `resident_bytes` **0** |
| differentiation | volume 0.38 to 1 276.80; distance 106.3 to 2 408.6 km — not all-zero and not all-identical |
| water access | 51 settlements sea, 169 river, 13 landlocked — the navigability port discriminates |
| modes | 549 river flows, 75 sea, **0 land**, and that is the model talking: at 6.25 km/cell the nearest pair is 106 km apart, past the 50 km local radius a landlocked bulk exporter is held to |
| every flow | mode consistent with **both** ends' water, inside its mode's reach cliff, `deliverable` matching `2^(-d/D)` to 1e-6, volume inside the supplier cap, and no `local`-reach flow past 50 km — checked on all 624 against constants the probe restates independently |
| unmet | **30 settlements** carry a need nothing in reach can fill, named with the goods |
| way load | 13 of 35 ways carry something, busiest 18 278 |
| determinism | two matches agree **row for row** across 624 rows |
| memory | 20 consecutive matches move the process working set **+5.0 MB** — nothing accumulates |

**IN-13, the three surfaces**

| what | measured |
|---|---|
| the dock | the unmatched state disclosed; the real button pressed; By good, Busiest partners, Needs nothing can reach, Way load and the cost footnote all on screen afterwards |
| the ledger | the place editor names partner `Andkrunbrakridge`, that is settlement #8's **real** name, and #8's own ledger lists this settlement back as a customer |
| the map | Trade load ON moves **0.3342 %** of screen pixels; OFF returns **0.0000 %** — byte-identical to before the layer existed |

**VA-01**

| what | measured |
|---|---|
| nothing until asked | an unbuilt index returns no rows at all |
| first build | 5 notes seen, **5 re-read** |
| the whole claim | a refresh over an untouched vault re-reads **0**; one edited file costs **exactly 1** |
| backlinks | exactly `Factions/Veldmark.md` (wiki, count **2** from one note) and `People/Aldis.md` (markdown) — the settlement's own note is not a backlink to itself, and the prose mention is not counted as one |
| mentions | exactly `Journal/Thaw.md`, with an excerpt containing the hit |
| the report | 1 broken link, 4 orphans, from the same index |
| the panel | Index, Refresh index, Rebuild and Missing & orphan notes all really on screen |

**ED-02**

| what | measured |
|---|---|
| the floor | a generate leaves exactly **one** row, kind `floor` |
| the mix | after a territory commit and two carves: `floor / recorded (frozen) / height / height` |
| the reason | the frozen row's reason is **on screen**, not only in the dictionary |
| linear revert | reverting to the *older* height row takes **2** steps, depth 2 → 0, and no height row survives it |
| refusal | reverting to that row again returns 0 — refused, not half-applied |
| the panel | History, Committed, Cost and the live `Reversible:` budget all drawn |

Plus `cargo test`: `cartalith-civ` **401 → 421** (20 new in `trade`),
`cartalith-vault` **48 → 65**, `cartalith-godot` **343 → 351** (8 new ledger
tests). `cargo check -p cartalith-godot` clean; headless boot and import
clean, with `project.godot` diffed afterwards and unchanged.

**One defect only the windowed run could find, and one only the screenshot
could.** `way_load` was emitted in `CivData::ways` order while
`map_overlay.gd` indexes `get_roads()` order — which filters hidden ways and
appends manual ones — so on this world it handed the shell **60 entries for 35
rows**, silently misaligned from the first hidden way onward. And the
Generate-world floor row read `seed 0` against a status bar reading `483920`,
because it recorded before `self.seed` was assigned; the two are only visible
together in a screenshot. A third is an old shape: the Match button was built
once from CIVIL's `setup()`, before any world exists, and nothing re-enabled
it — `GUI_GAP_REGISTER.md` **RF-01** again, found the same way, by pressing
the real control instead of reading the source.

### What stays open

Nothing new. The six narrowed remainders above are unchanged in substance —
this pass gave them a consistent surface, not a capability — and IN-13's own
remainder is stated on screen for the first time: **prices, tariffs, caravans
as entities, and trade that changes over time**, none of which is derivable
from anything the civ layer holds and each of which needs a decision about
what a currency is here.

## 43 · CV-25's other half — the manpower model, on an owner-supplied specification (2026-08-25) — **STILL NARROWED**

§40 built CV-25 as a minimal military model and found, as its headline, that
the register's reason for calling it a design was wrong: the reference **does**
model fortification, three times over, and `power.military` was already ported.
The owner has now supplied a detailed, researched specification for the half
§40 could not derive — **how many people a polity can actually put and keep
under arms** — and it supersedes §40's implicit answer, which was "none, a
headcount would be a fabricated number wearing a real one's clothes".

That sentence was right about the *evidence available at the time* and wrong as
a permanent verdict. A headcount is fabricated when nothing implies it. With
five stated variables and two derivation chains, it is derived.

`MILITARY_MANPOWER_SCOPE.md` is the durable home: it carries the owner's
specification **verbatim**, the derivation of every constant, and the
verification. What follows is only what the register itself needs.

### The split, and what stays

| | disposition |
|---|---|
| `_umWallSpec` / `_umInferWalls` / `_civPlaceDefensibility` | **unchanged.** Fortification is a separate axis — how hard a place is to take, not how many people can be raised. Ports, golden-verified, untouched by this pass. |
| `_civFactionAggregates` → `power.military` | **unchanged, deliberately.** A golden-verified port of the reference's own formula. Rewriting it to derive from a model the reference does not have would break parity to gain nothing; and it answers a different question (*this faction against the others on this map*, relative 0-100) from the headcounts (*how many people*, absolute). Reported side by side, each labelled. |
| manpower | **new** — `cartalith_civ::manpower`, four outputs from five variables. |

### The reference really has nothing this time

Unlike §40, which found three implementations by grepping, this one found
none. `manpower`, `mobiliz`, `levy`, `conscript` and `militia` return **two
hits in the whole frozen snapshot**, both `JP_COST_TOLL_PER_BORDER`'s comment
using "levy" to mean a *toll*; `FUNCTION_INDEX.md` returns zero. So there is no
golden fixture and none is fabricated — 14 unit tests and two live probes
instead.

### Two more inert tables got their first consumer

The §40 pattern, twice over. `roster::AG_TECH_LEVELS`' own module doc said
`farmers_per_urbanite` was *"presently as inert as Government/Religion are in
the reference"*; `roster::CIV_GOVERNMENTS`' said *"no simulation reads or
writes this, and nothing in this port does either"*. Both are read now, and
both are proved live on a real world rather than asserted: `traditionalAgrarian
→ improvedAgrarian` moves a faction's standing army **1 435 → 2 615**, and
`chiefdom → empire` moves it **948 → 1 841**. Their two roster tooltips said so
in as many words and now say what they do instead.

### The four outputs, on a real 233-settlement world

Standing 87 … 1 509 · field 3 444 … 9 305 · levy 8 009 … 20 262 across six
factions, with a four-rung force/duration ladder each. **The worked examples
in the specification are reproduced**: Kingdom A at 5 846 / 41 221 / 15 870
against a stated ~5 000 / ~40 000 / 15 000-20 000, Kingdom B at 19 067 /
98 889 / 47 368 against ~20 000 / 100 000+ / 40 000-60 000. Every figure in
range; the worst is A's standing army at +17 %, left there rather than tuned.

Three things fall out that the specification does not state and so cannot have
been fitted: Kingdom A's full levy sustains **77 days** (the feudal ~2-month
obligation), Kingdom B's 90- and 180-day rungs are 59 455 and 38 045 (which
brackets its stated field army — a field army *is* a campaign-season force),
and the standing shares land at Imperial Rome's own ratio.

### Four findings, and one is a question for the owner

1. **The specification's era table and its worked example disagree**, and the
   table disagrees with its own Imperial Rome figure — Kingdom A's stated
   40 000 levy is 4 % of population, below the 5-15 % band of every pre-modern
   era listed; Rome's 250 000 over 45-120 million is 0.21-0.56 %, under the
   classical row's 1 % floor by two to five times. Calibrated on the worked
   example, with the band reported as the sanity check the specification asks
   for. **A plausible reconciliation, offered rather than implemented: the
   bands may be shares of a citizen or free population** — the specification's
   own Republican Rome citation says "17-29 % of its *citizen* population" —
   in which case the live figures land inside them. One owner decision changes
   every verdict and nothing else. **→ Ruled and built the same day; see §44.**
2. Standing shares agree with the specification's *example* and not its
   *table*, in the same direction and for the same reason.
3. `ecological_factor` saturates for five of six factions on a real world:
   generated territory sustains at least twice the population the model puts
   on it, the same divergence `civ_agrarian_regional_total`'s own readout has
   always shown. Geography therefore discriminates mainly at the low end,
   where it does real work — Draumr League's 87-strong standing army against
   Veldmark's 1 509 on otherwise identical institutions.
4. **The road-density reference was wrong on the first try and measuring found
   it.** Anchoring on the Roman empire's ~16 km/1 000 km² made roads a dead
   term; this port's way network is inter-settlement trunk roads only, with no
   lanes or streets, so it is not comparable to a road inventory. Recalibrated
   against what the network actually produces, roads spread 0.11-0.91 instead
   of 0.03-0.23.

### CV-25 stays narrowed, and the note on screen changed

The old note said *"garrisons · campaigns · unit movement · combat"*. Two of
those four words were doing different jobs, and the row now separates them:
**per-settlement garrisons** (the per-*faction* headcounts are real; which
settlement holds which part of a standing army is a placement rule nothing
implies), campaigns, unit movement and combat. Still `needs a decision`, still
disclosed in the category, and the shell probe asserts the disclosure is there
rather than trusting it.


## 44 · CV-25 — the owner answered §43's open question: the era bands are shares of the citizen population (2026-08-25) — **STILL NARROWED**

§43's finding 1 was a question back rather than an answer, and the owner has
answered it: **the era table's percentages are shares of the citizen / free
population, not of the total.** The evidence is inside the owner's own
specification — its Republican Rome figure is stated as *"17-29 % of its
**citizen** population"*, the one place the specification names a denominator
at all.

`MILITARY_MANPOWER_SCOPE.md` §1a carries the ruling (as an annotation, because
that document reproduces the specification verbatim and does not edit it), §2.6
the derivation, §3.2a the re-measured verdicts.

### What the denominator is, and what grounds it

Nothing in `cartalith-civ` distinguished a citizen, free or full-status subset
of population — grepped before inventing one, which this session has now been
repaid for **eight** times. `FactionEntry::culture` was checked too and turns
out to be `CIV_CULTURES`, name-syllable pools with no social content.

So it is derived from what does exist:
`clamp(CITIZEN_SHARE[government] + 0.68 × urbanisation, 0.20, 0.98)`.

Government is the driver on the merits and not merely by availability: the two
cases the specification cites sit on either side of exactly this distinction —
a republic's citizen body is a much larger share of its polity than a
pre-Caracalla empire's, which is what makes Hopkins' 17-29 % and Rome's
0.21-0.56 % consistent with one table. Shares run `chiefdom` 0.90 → `monarchy`
0.55 → `republic` 0.50 → `city_state` 0.45 → `oligarchy` 0.40 → `empire` 0.30,
each grounded in §2.6's table (Domesday, Attica c. 431 BC, Polybius' 225 BC
census). The modernisation term is **derived rather than chosen** — legal
servitude is an agrarian institution, and `0.68 = CITIZEN_CEILING −
min(CITIZEN_SHARE)` is what makes every government converge on universal civic
status at industrial labour ratios; a test pins the identity.

### The calibration did not move, and that is asserted rather than claimed

The four outputs are calibrated on the specification's own worked examples and
were **not** recalibrated: `the_citizen_ruling_moves_no_headcount` pins Kingdom
A and B to the figures §43 published, and the live probe pins that total
population does not move when only the government does and that every levy
restores exactly when the roster is put back. Only the two verdicts changed
basis.

### What the verdicts read now

On the 233-settlement world, **five of six factions read `within` on both
bands** where all six previously read `below` on standing. Draumr League is
still `below` on standing at 0.09 %, honestly — its `ecological_factor` is
0.428, which is finding 3, not a denominator problem, and no denominator was
going to move an 87-strong standing army into a band.

On a sparser 33-settlement world with one government per faction (the
denominator has to be *differentiated* to prove anything, and a default roster
is all-`monarchy`), the citizen fraction spreads **0.378 … 0.978**, mobilization
reads `within` for five of six, and standing reads `within` only for the
narrowest citizen body. That residual is §43's finding 2 — the model's standing
armies sit at Imperial Rome's ratio, which the table's standing column never
agreed with — and it is reported rather than tuned, because correcting it would
mean recalibrating outputs validated against the worked example.

### On screen

The denominator is **surfaced, not invisible** — a band verdict whose divisor a
reader cannot see is a number they cannot argue with. CIVIL ▸ Military gains a
*Who the bands are measured against* group: one line per faction with the
citizen headcount, its share of the total, both citizen-based shares with their
verdicts and the era, and a tooltip quoting what the same two figures would
read against total population, so the previous basis stays legible rather than
being deleted. The Faction Roster's Military block names the citizen population
and the government that conferred it on the line above its verdict. The
category's closing note now states the ruling instead of warning that `below`
should be expected.

CV-25's row is otherwise unchanged: still `needs a decision`, still open on
per-settlement garrisons, campaigns, unit movement and combat.

## 45 · MN-10, RL-01, CA-20, RF-02…RF-05, FI-04 — the "is every control wired" sweep (2026-08-25) — **SEVEN FIXED**

The owner asked for confirmation that every GUI control does what it claims,
reaches real capability, and does not silently do nothing. This section is that
sweep, and it is deliberately as much a record of what was driven and found
**clean** as of what was found broken — a surface walked and cleared is worth
recording so the next pass does not re-walk it.

Nothing here was found by reading. Every finding below came out of the live app.

### What was driven, and how much of it

Four of the six probes are new control classes no sweep in this repository had
covered before.

| Probe | What it covers | Never covered before |
|---|---|---|
| `_rf01_probe.gd` | 89 surfaces fingerprinted at **no world → world A → world B** | the whole RF-01 question, systematically |
| `_railpress_probe.gd` | every enabled button in all **33 rail categories**, the right dock in **7 contexts**, **11 tool-options rows**, the section strip, the menu bar | `_pressall_probe.gd` only ever pressed buttons in *windows* |
| `_valuectl_probe.gd` | **11 OptionButtons and 110 Ranges**, each moved to a different value and put back | **nothing had ever changed a value control and asked what happened** — both existing sweeps skip `OptionButton` by name |
| `_menuwire_probe.gd` | 148 menu items across 23 popups, with `about_to_popup` fired first | menu gating computed on popup was read cold, producing four false positives |
| `_winstale_probe.gd` | every window **left open across a generate** | §23 asked "what re-runs this?" of panels built at launch, never of windows built on `open()` |
| `_newsurf_probe.gd` | the newest surfaces end to end, plus all **35 Layers entries measured in pixels** | — |

Three of those probes were wrong on their first run, and each fault is worth
keeping because it is a way this class of sweep silently covers nothing:

1. **A jump button leaves a different workspace on screen.** The rails are full
   of `→ Cartography ▸ Political display` rows; one press and every later
   category reads `is_visible_in_tree() == false`. The first run "passed" twelve
   of CIVIL's fourteen categories by finding no controls in them at all.
2. **`emit_signal("pressed")` on a toggle changes no state and fires no
   `toggled`.** Every `DccWidgets.toggle` in the shell — 30-odd checkboxes —
   read as dead until the probe drove `button_pressed` instead.
3. **A menu never popped has not been asked its own gating question.** Half this
   shell's menus compute `disabled` in `about_to_popup`.

### MN-10 — a menu item with a written handler and no way to reach it

`Assets ▸ Asset pack ▸ Pack metadata… (name / author / license)` was enabled,
carried an id, and had a handler branch (`_on_assets`' `ID_AP_PACK_META`) that
nothing could ever reach: the `AssetPack` popup's **`id_pressed` was never
connected**. Its three child submenus each connect their own (`APEdit`,
`APBatch`, `APBuild`), which is exactly what made the omission invisible to the
eye — the submenu below it worked, so the group looked wired.

A submenu's `id_pressed` does **not** bubble to its parent in Godot 4; each
`PopupMenu` emits only for its own items. One line. Verified: connections
**0 → 1**, and the row now takes the Asset Library window from hidden to shown.

**The general check this adds**, and the reason `_menuwire_probe.gd` exists: for
every popup, if `id_pressed` has no connection, list its live items. That one
question would have caught this the day it was written.

### RL-01 — a row that named a pair, opened one side, and often did nothing at all

CIVIL ▸ Relationships lists one row per faction pair (`Aurelia ↔ Korrath —
wary (−22)`). Every row called `show_faction(a)` — the **left-hand** faction —
so a row claiming a pair opened one party of it, and any run of rows sharing
that party was a press with **no visible effect anywhere**.

Measured on the 233-settlement six-faction world: **5 of 15 rows dead** —
Korrath ↔ Draumr League, Veldmark ↔ Mirelle, Veldmark ↔ Draumr League,
Aurelia ↔ Sythe Dominion, Aurelia ↔ Korrath. The tooltip was honest about it
(*"Opens Aurelia in the right dock"*), which is what kept it looking deliberate.

Both halves are fixed together, without a new dock context: `show_faction` takes
the other party, and the faction panel gains a **Relations** section built from
the same `civ_faction_relations()` read the list itself makes — so the two
cannot disagree about a value — with the clicked pair marked `▸`. The tooltip
now says what it does. Verified: **all 15 rows move the dock**, and the dock
draws `▸ Sythe Dominion` among Korrath's relations.

### CA-20 — two Clear-all buttons live over an empty list

`DCC_SHELL_SPEC.md` §4.5.5 asks for both annotation panels *"with counts and
Clear-all"*. The count was never drawn, and the button was enabled at zero — a
press that could not change anything, with no tooltip to say why. The count now
sits on the button (its own subject), the button is disabled at zero with a
stated reason, and its state is owned by the `_rebuild_*_panel()` calls that
already run on every place, delete, clear and world change.

Verified: empty → `Clear all labels`, disabled, a 110-character reason; two real
labels → `Clear all labels (2)`, live; press → engine list **0** and dead again.

### RF-02 / RF-03 — §23's question, asked of windows this time

§23 asked *"what re-runs this, and on which signal?"* of every panel built at
launch. It never asked it of the windows, because those are built on `open()` —
which is correct only if nothing can change while they are up. A world can.

Both of the ones keyed to an identity a generate renumbers were stale, and both
are **destructive** rather than merely out of date, because every editable
control in them writes by that identity.

- **RF-02, the place editor.** Left open across a generate it showed
  `Sevjuniana` pop **19 332** at (142, 14) while the engine's settlement 0 was
  pop **19 774** at (208, 183) — the form character-for-character identical.
  Every field writes `civ_edit_settlement(_index, …)`, so a commit would have
  written the previous world's name, kind and traits onto whatever now sat at
  the index. PE-01's failure with a generate as the trigger instead of a click.
- **RF-03, the faction roster.** Aurelia:27 / Veldmark:49 / Mirelle:57 on screen
  against a live engine reading Aurelia:57 / Veldmark:27 / Mirelle:7 — plus two
  cached per-world fields (`_fits`, `_military`) taken at `open()` and never
  re-taken. FR-02's failure, same trigger.

The name RNG is seeded the same way in every world, so settlement 0 and faction
0 come out with the *same name* on both seeds — a name comparison proves nothing
here, and the first cut of the probe passed on exactly that. Population,
coordinates and settlement counts are what discriminate.

Both now subscribe to `generation_finished` and `world_loaded` and rebuild when
visible — the shape three windows in this shell already use (`city_viewer`,
`world_data`, `performance`). **Rebuilt and not closed**, deliberately: half of
`world_loaded`'s emitters (`load_asset_pack`, `as_apply_to_map`) do not touch a
single settlement, so closing would be wrong for them. And through `_rebuild()`
rather than `open_for()`, because `open_for` commits the focused field first and
committing this form against the world that has just replaced the one it was
typed for is the bug, not the fix — `_clear()`'s `_rebuilding` guard drops it.

### RF-04 — the first signal-*ordering* bug this register has had

`infrastructure_workspace.gd`'s own comment said the Flows body *"refills from
whatever `TradeStore` holds, which `app.gd` has just cleared on this same world
change"*. It had not. Godot delivers a signal in **connection order**;
`app._register_workspaces()` runs at line 313 and `_wire_status()` at line 333,
so on every generate the INFRA refill read the **previous** world's match, redrew
its flow count, its timing and its settlement names — and only then did
`_refresh_world_dependent()` drop the store, with nothing left to re-run the fill.

Measured: after regenerating under a live **624-flow** match, CIVIL ▸ Trade ▸
Flows still reported 624 while `TradeStore.last()` was empty, so the dock and its
two fellow readers (the place editor's per-partner ledger, the way-load overlay)
disagreed about whether a match existed at all.

Fixed by connecting the clear immediately after the bridge is constructed, before
anything else subscribes to either signal. The call in `_refresh_world_dependent()`
stays — it costs nothing and keeps that function's stated ownership true.
Verified: the body goes back to *"Not matched yet"* where it did not.

**The lesson, and it is new:** a comment asserting that another handler has
already run is a claim about connection order, and connection order is decided
by `_ready()`'s call sequence in a different file. Two correct handlers can still
be wrong together.

### RF-05 — RF-01 exactly, on a control that worked perfectly

**CARTO ▸ Roads & routes ▸ Trade load** disables itself when
`overlay.has_trade_load()` is false. That category is built once, at launch,
against an engine with no world — so the row was born disabled, and the match
that makes it valid runs in a **different workspace** with nothing to say so.

Measured: after a real 624-flow match, `disabled = true` while `has_trade_load()`
returned `true`. Forcing it on moved **0.6028 %** of the map's pixels and off
returned **0.0000 %** — a fully working control with no reachable route to it.
The fifth recurrence of RF-01, and the second on a control gated at build time
against data that arrives later.

Fixed at the funnel rather than at the caller: `map_overlay.set_trade_load()` is
the single path both the match and the world-change clear already pass through,
so it emits `trade_load_changed(available)` and CARTO follows it **in both
directions** — turning the switch off, not merely greying it, when the reading
goes away, because a live toggle over an empty reading is the same lie the other
way round.

### FI-04 — copy that named a place the user cannot look

`File ▸ Recent worlds` remembers a **path**, not a file, so every row in it can
outlive what it points at. All three rows of a real recent list named saves a
previous session had since deleted, and the only thing said about it was
*"load failed — see console"* — which is true of neither, and names somewhere an
exported build has no access to. The distinction costs one `file_exists` and the
two halves have different owners: a missing file is the list's problem, a refused
one is the save's. Now reads `<name> is no longer on disk`.

### Driven and found clean — the negative results

Recorded so the next pass does not re-walk them.

- **RF-01 across three worlds, 89 surfaces.** All 33 rail categories, the right
  dock, the Layers popover, 11 tool-options rows, the section strip and 23 menu
  popups, fingerprinted with no world, then with world A, then with a different
  world B. **Not one surface kept an empty state across a generate.** §23's
  eleven sections and §37's fifteen are all still refreshing.
- **Value controls: 11 OptionButtons and 110 Ranges, zero dead.** Every one
  moved something. This is the class that had never been driven at all.
- **Buttons in the rails and dock: only the findings above.** The one remaining
  no-op is the `SCULPT` mode chip, which rebuilds the row it lives in.
- **The menu bar:** 148 items, 41 pressed. `APBuild ▸ Apply to map` and
  `Assets ▸ Apply library to map` are the same action reached two ways and set
  the same status, which is a confirmation and not a defect.
  `GpuDevices ▸ Rescan devices…` re-queries and refills a submenu read on popup —
  idempotent on a stable machine, real capability, no visible change.
- **The Layers popover, in pixels.** 35 entries; 34 repaint the map and
  **all 34 hash differently** — no two layers draw the same frame. RD-02's class
  is clean here.
- **Windows across a generate:** City viewer and World data rebuild. Performance,
  Travel library, Vault, Data manager, Asset library and the Layers popover render
  identically and correctly so — none of them shows anything keyed to world
  identity.
- **The newest surfaces:** the trade match runs in 166 ms for 624 flows over 61
  importing and 31 supplied settlements; the ED-02 ledger's floor row names the
  live seed; CIVIL ▸ Territories ▸ *Analyse contested borders* is live and
  *Clear territory* is correctly disabled.

### Not fixed, and why

- **`Window ▸ Open windows ▸ "No windows open"`** is disabled with no tooltip.
  It is an **empty-state caption**, not a capability claim — the same shape as
  the asset-pack submenu's three disabled stat rows — and the honesty convention
  is about controls that claim something. Registered here rather than given a
  tooltip that would read as a disclosure of a gap that does not exist.
- **A recent-worlds row whose file has vanished stays in the list.** FI-04 makes
  it say so; whether the row should then be greyed, removed, or offer to forget
  itself is a product decision, not wiring.

---

## 46 · PH-12 — nine screens that never had a phone pass at all (2026-08-25) — **FIXED**

The owner ran the Android build on a **OnePlus 12** — 1440x3168, ~510 ppi, so
`DccShell._phone_scale` is `1440 / 393 = 3.664`. Every prior phone measurement
in this project was taken at a 1080 short side (scale 2.75), and none of them
was taken on any of the screens below. His words:

> *"not all screens are optimised for a mobile phone, among others the asset
> manager screen. Plus the layout is impractical and doesn't listen well to
> touch input and isn't intuitive."*

### The fault

The shell has a working, four-call phone pattern, written across PH-03 to
PH-11 and documented in `dcc_widgets.gd`'s own header:
`DccWidgets.phone_window()` at setup, `DccWidgets.phone_present()` on open,
`DccShell.phone_fit(self, 1.0)` after the body is built, and
`DccWidgets.phone_head()` for the header a borderless window draws in place of
its title bar.

**Nine `Window`-derived scripts called none of it — zero occurrences of any of
the four.** They are not degraded on a phone; they are simply the desktop
composition, drawn at the device's native resolution, which on a 510 ppi panel
means a 24 px `dcc_widgets` row is about 1.2 mm of glass.

Measured in a real windowed run at 1440x3168 with the force-touch path
(`_ph9_probe.gd`), before the fix. "Under floor" is §13's 44 dp minimum, which
at this scale is 161 physical px:

| Screen | Window | Tappables under floor | Smallest |
|---|---|---|---|
| `asset_library_window.gd` | 1440x1002 @ y=34, scale 1.0 | **59 of 59** | **13 px** |
| `data_manager_window.gd` | 1440x1002 @ y=34, scale 1.0 | 16 of 16 | 22 px |
| `travel_library_window.gd` | 1180x841 centred | 17 of 17 | 26 px |
| `layers_popover.gd` | 230x600 anchored | 40 of 40 | 14 px |
| `world_data_window.gd` | 760x620 centred | 1 of 1 | 29 px |
| `gen_info_dialog.gd` | 560x513 centred | 1 of 2 | 29 px |
| `performance_window.gd` | 560x437 centred | — (its only control is internal) | — |
| `credits.gd` / `app.gd::open_credits()` | 720x640 centred | — (same) | — |
| `journey_planner_view.gd` (centre panel) | not a `Window` | — | — |

A parallel sweep on a generated world adds the numbers the empty state hides:
the world-data settlements table is **1 470 individual `Label` nodes** across
240 rows at font size 9, reachable only through an ~8 px scrollbar; the data
manager's body labels are 39 of 69 under 11 px.

### What a content scale fixes, and what it does not

`phone_present()` maps the whole desktop-authored composition onto the
mockup's own 393 dp reference. That answers density, and `phone_fit()` answers
tap size. Neither answers **composition**, which is exactly why
`phone_window()` returns `is_phone`:

- **Asset library**: a 266 px family rail + a slot grid + a 330 px inspector.
  The rail and the inspector alone are 596 of the 393 dp available. → three
  panes behind a segmented switcher, one at a time (§13's own *"docks become
  full-height sheets, one at a time"*), with the switcher following the work:
  pick a family → SLOTS, tap a slot → SLOT.
- **Data manager**: 252 px rail + pane → two panes, same switcher.
- **Travel library**: 286 px rail + inspector → two panes, switched by the tab
  strip it already has plus a `‹ Entries` chip.
- **Slicer modal**: a flexible preview beside a fixed 274 px settings stack →
  stacked, with the settings column scrolling.
- **Journey planner centre panel**: a 196 dp totals column beside the route
  map, and a 642 dp stage matrix beside the stage inspector → both stack.
- **Layers popover**: not a composition problem — a **control-type** problem.
  See below.
- **World data**: not a composition problem either. Six columns across 393 dp
  is ~55 dp each. → two-line rows, name over the rest, and a 50-row page with
  a *"showing 50 of 240"* foot.

### Five things only measurement found

1. **`AcceptDialog`'s button bar is an internal child.** `phone_fit()` walks
   `get_children()`, so it has *never* reached the OK button — the only way
   out of `gen_info`, `performance`, `world_data` and the credits sheet.
   Measured 29 dp. Floored in `phone_present()` instead, **after** the
   `popup()`, because `Window.popup()` clears `custom_minimum_size` when it
   re-lays that bar (the trap `app.gd::_floor_prompt_buttons()` recorded for
   the quit prompt, hit again here).
2. **`TabContainer`'s tab strip is an internal `TabBar`** — same blind spot,
   same consequence. A tab has no height property; its height is the font plus
   the stylebox's vertical content margins, so those are the knob. 26 dp stock.
3. **`phone_fit()`'s ellipsis pass reaches only `Button`s.** A `Label` still
   reports its full natural width, and *a `Window` cannot be narrower than its
   content's minimum* (the PH-04 hazard). Three separate `Label` rows each
   widened a window past the screen on their own: the asset grid's header band
   with five batch verbs beside it (544 dp), its status line's two labels side
   by side (401 dp), and the data manager rail's autowrap foot at **394 dp** —
   one pixel over, because of the panel's own 1 px border.
4. **An embedded subwindow is laid out in its parent viewport's 2D space.**
   The slicer modal was a child of the asset library window, whose viewport is
   content-scaled by 3.664 on a phone — so a slicer sized to "fill the screen"
   from inside it would have been sized in units 3.66x larger than the screen.
   Reparented to the shell. This is the same physical-pixels-versus-parent-
   space confusion `_popup_full()` already records, one level further in.
5. **The credits body was empty, on every platform.** Not a phone bug at all,
   found while giving that window the phone treatment. `_ready()` fires when a
   node enters the tree, and `add_child(dlg)` is what put it there — so by the
   time `set_script()` attached `credits.gd`, its `_ready()` had already missed
   its only chance to run. Attaching a script to a node already in the tree
   does not re-run it. Measured: **0 characters** in the `RichTextLabel`, now
   4 420. The attribution `PROVENANCE.md` calls a standing obligation had been
   reaching nobody.

### The Layers popover: checked before it was built

A popover may simply be the wrong control on a handset, and §13 routes several
desktop affordances into the ⋯ overflow sheet instead — so the first question
was whether this one is reachable on a phone at all. **It is, by three
routes**: the map's own Layers button (`viewport.layers_button_pressed`),
Cartography ▸ *Data overlays…* (`cartography_workspace.gd`), and the Render
section's own entry (`render_workspace.gd`).

So it needed real work, and it became a **full-screen sheet**, not a scaled
popover. A popover is a pointer idiom: anchored to the control that opened it,
dismissed by clicking away from it. A phone has neither a stable anchor (the
Layers button moves with the safe insets) nor a reliable "away". That is also
why it grew an explicit Close — a sheet that covers the screen has no outside
left to tap — and why its foot moved *inside* the scroll: the six-line
Cartography cross-reference note, pinned below a 393 dp list, pushed its own
last two lines off the bottom edge where no scroll could reach them.

`DccWidgets.phone_window()` takes an `AcceptDialog` and this is a
`PopupPanel`, so only the halves that apply were used: `phone_present()`
(which takes any `Window`), a `phone_head()`, and `wrap_controls = false` by
hand.

### Verified

`_ph9_probe.gd` / `_ph9_probe.tscn`, driven at **1440x3168** (scale 3.664) and
**1080x2400** (scale 2.748), `--force-touch --nowelcome`, on a generated world.
Per window: `content_scale_factor`, `size` against the screen, every
tappable's height against the floor, every control's combined minimum width
against the **window's own 393 dp column** (not against the screen's 1440 px —
comparing dp against physical px finds nothing and misses this whole class),
and whether each body actually scrolls. Screenshots of all ten surfaces.

| Screen | tappables | under floor | min.x > 393 dp | scale |
|---|---|---|---|---|
| Asset library | 42 | **0** | **0** | 3.664 |
| Sprite-sheet slicer | 20 | 0 | 0 | 3.664 |
| Data manager | 18 | 0 | 0 | 3.664 |
| Travel library | 18 | 0 | 0 | 3.664 |
| Layers sheet | 41 | 0 | 0 | 3.664 |
| World data | 1 | 0 | 0 | 3.664 |
| Gen info | 2 | 0 | 0 | 3.664 |
| Performance | 0 | 0 | 0 | 3.664 |
| Credits | 0 | 0 | 0 | 3.664 |

Identical at 1080x2400 (column 393.0 dp, scale 2.748) — **no regression at the
size every prior pass used**. Desktop unchanged and re-measured: 1440x1002
under the menu bar for the two full-bleed windows, 1180x780 / 760x620 /
560x480 / 560x420 for the dialogs, 760x560 for the slicer, 230x600 for the
anchored popover, six-column asset grid, three columns side by side.

One back press closes the Layers sheet (`DccShell::_notification`'s
`_topmost_subwindow` reaches it, because it is a subwindow).

### Two probe artifacts, recorded so the next pass does not chase them

- **`--resolution 1440x3168` is silently clamped** to the dev monitor's work
  area, and `_compute_layout_mode()` then decides *tablet* off the boot size,
  so the whole run measures the wrong composition. Assign `get_window().size`
  at runtime **before** instantiating `app.tscn`.
- **`Window.popup(Rect2i)` clamps to `get_usable_parent_rect()`**, which for a
  subwindow on a desktop host resolves to
  `DisplayServer.screen_get_usable_rect()` — 1680x1002 here, **not** the
  1440x3168 root viewport the probe set up. A window that correctly asks to
  fill the screen is therefore reported at 1440x1002 on this box: the width is
  real, the height is a desktop artifact with no counterpart on Android, where
  there is one OS window and every subwindow is embedded in it. `is_embedded()`
  returns `true` in both cases, so that is not the discriminator. The probe
  re-asserts `size` after the popup — which *does* raise the resize
  notification, because the window is visible by then — and measures the
  result.

### Not fixed, and why

- **The Layers sheet and a phone overlay can be open at the same time.**
  Measured: with the sheet up, `_set_drawer_open(true)` leaves both visible.
  `DccShell._close_all_phone_overlays()` lists the drawer, the panel picker,
  the phone menu and both dock sheets, and does not know about subwindows. The
  one-line fix belongs in `dcc_shell.gd`, which a concurrent agent was editing
  during this pass; committing that file would have carried their in-flight
  work with it. Registered rather than done. Severity is low: one back press
  already closes the sheet first, so the state is reachable but not a trap.
- **The asset library's phone window bar is four rows tall** (search, plus
  three wrapped chip rows), about 24% of the screen. Every action stays
  reachable and the chips wrap by themselves rather than by a count fixed in
  code, so this is a proportion question, not a defect — but it is the obvious
  candidate if that screen is revisited.
- **Drag-a-tile-onto-a-Collection** is kept on a phone but its disclosure
  lives in a tooltip, which touch cannot reach. The two pointer-modifier hints
  beside it (`⇧-click ranges · Ctrl-click adds`) are dropped there; the drag
  itself is left alone rather than removed on a guess about touch drag.

## 47 · HD-01…HD-04 — the OnePlus 12 pass: the blur was the font raster, and two of the four leads were negatives (2026-08-25) — **FOUR FIXED, TWO PROVEN NEGATIVE**

The owner ran the Android build on a **OnePlus 12** (1440x3168, ~510 ppi) and
reported *blurriness*. Every prior phone measurement in this repository was
taken on a 1080-wide handset, where `DccShell._phone_scale` is **2.748**; on a
OnePlus 12 it is **3.664**, so a defect that scales with that number is 33 %
worse and crossed from marginal into visible.

Everything below is measured on the framebuffer. The metric for "is this text
rasterised or resampled" is the **maximum luminance step between horizontally
adjacent pixels**: a natively-rasterised glyph goes ground-to-ink in one pixel,
while a bitmap magnified by *k* cannot produce a step steeper than roughly
`1/k` of its own contrast. It is a discriminator, not an impression, and it is
why an adjective was not accepted as evidence anywhere in this section.

### HD-01 — Godot 4.7.1 does not oversample fonts for a Window's own content scale · **FIXED**

`DccWidgets.phone_present()` puts every phone modal in a
`CONTENT_SCALE_MODE_CANVAS_ITEMS` sub-Window at `content_scale_factor` 3.664,
so a 12 px label is authored in dp and magnified by the compositor. Godot 4.5
introduced dynamic font oversampling and 4.7.1 has `Viewport.oversampling` **on
by default** — but `Viewport.get_oversampling()` inside such a window returns
**1.0**. The automatic value does not account for a Window's own content scale,
so the font is rasterised at 12 texels and the canvas transform smears it.

Measured on this exact build (`_edge_probe.gd`), two windows drawing the same
physical glyph height:

| case | max ΔLum | hard edges (>0.5) |
|---|---|---|
| factor 3.664 / font 12 | **0.2667** | **0** |
| factor 1.000 / font 44 (control) | 0.9843 | 722 |
| factor 3.664 / font 12, `oversampling = false` | 0.2667 | 0 |
| factor 3.664 / font 12, `oversampling_override = 3.664` | **0.9804** | **518** |
| factor 2.750 / font 12 | 0.3569 | 0 |
| factor 2.750 / font 12, override 2.750 | 0.8431 | 375 |

0.2667 is 1/3.75 and 0.3569 is 1/2.80 — the magnification, recovered out of the
pixels. The boolean is **not** the lever: turning it off changed nothing to four
decimal places. `oversampling_override` is.

Two traps, both of which made the first cut of the fix measure exactly as if it
were absent:

1. **The property is inert until the window is in the tree.** Assigned in a
   constructor it reads back on the property and `get_oversampling()` ignores
   it.
2. **A resize clears it, and the value it reverts to is 1.0.** Measured in
   isolation: set on a content-scaled Window it survives eleven frames, a
   `popup()` and a hide/show cycle, then reads back 1.0 the frame after `size`
   is assigned — and reassigning `content_scale_factor` afterwards does not
   bring it back. This is the same trap `phone_present()` already carries for
   the `AcceptDialog` button bar. So `DccWidgets.oversample()` sets it *and*
   re-applies it from `size_changed`.

Live on the welcome screen at 1440x3168: **max ΔLum 0.1827 → 0.6126, hard edges
0 → 104.** At 1080x2400: 0.6322 / 74. (0.61 rather than 0.98 because the shell's
body text is `#c8cbcd` on `#17191a`, whose own maximum step is ~0.69 — this is
the native ceiling, not a partial fix.) One call, in one function, covering
every phone modal in the app with no call-site change. An embedded `PopupMenu`
inherits its parent window's transform but not its font raster, so
`phone_fit()`'s `OptionButton` branch routes its list through the same call.

### HD-02 — SVG glyphs were rasterised at authored size and then magnified · **FIXED**

`DccIcons.get_icon(name, px)` rasterised at exactly `px`, and inside a
content-scaled window `px` is dp. Measured at 1440x3168: all four glyphs on the
welcome screen at **0.27 texels per physical pixel** — 12 texels drawn at 44 px,
30 at 110, 26 at 95.

Corrected first, since the brief for this pass overstated it: there are **8**
`DccIcons` raster call sites, not 66. The other ~60 hits are `DccIcons.SYMBOLS`,
which are *text* and are drawn by the font — which is why HD-01, not this, is
where the owner's complaint actually lived.

`get_icon()` gains a `magnify` argument: rasterise at `px * magnify`, **present
at `px`** via `ImageTexture.set_size_override`, cache keyed on both. Nothing a
caller lays out moves, and `magnify` defaults to 1 so the main viewport — which
has no content scale, and where a finer raster would only be minified back
through a 1.2 px hairline — is byte-identical.

Three shapes of the fix were tried and each was wrong for a real call site:

- a static "device scale" set once by the shell — wrong in the main viewport;
- re-rasterising from `DccShell.phone_fit()`, which knows the number exactly —
  but only reaches a subtree that exists when it runs, and
  `open_project_dialog.gd` builds its action tiles and its import tile on
  `navigate()`, long after its one `phone_fit(self, 1.0)` call. **Measured: the
  search glyph was fixed and the other three were not**;
- re-rasterising on `tree_entered` — fires before `phone_present()` has set
  `content_scale_factor` for every glyph built during `setup()`.

What works is asking the node itself, at draw time. The call is
`get_screen_transform()` and **not** `get_global_transform_with_canvas()`, which
is the one that reads like the right answer: measured side by side on all four
glyphs, `gtwc` scale is (1.0, 1.0) and `screen` scale is (3.664122, 3.664122) —
a `CanvasLayer` transform is not a viewport's *final* transform, and a content
scale lives in the latter. With the wrong one the fix is silently inert, which
is exactly how it first measured.

After: all four glyphs at **1.00–1.01** texels per pixel at 1440x3168 and at
1080x2400, quantised to 1/16 so a float32 `content_scale_factor` cannot
re-rasterise on every frame. Desktop unchanged (12/12, 30/30, 26/26, 15/15).

### HD-03 — the viewport's floating chrome is 2.19 mm on a 510 ppi panel · **FIXED**

`viewport_host.gd` lives in the **main** viewport, which has no content scale,
so every constant in it is a real device pixel. `NAVPAD_HIT`'s own comment
asserted the opposite — *"the shipped phone's viewport is ~393 px, where that
scale is 1.0"* — and it is false: measured, `get_viewport_rect()` reports
1080x2400 and 1440x3168 and `_phone_scale` reports 2.748 and 3.664. A raw 44 px
pill is **2.83 mm** on a 395 ppi panel and **2.19 mm** on the OnePlus 12's,
against roughly 7 mm for the 44 dp this shell floors every other target at. Not
blurry, but the same complaint.

The scale rides through the existing `set_safe_insets()` dictionary rather than
a new setter, so `app.gd` — which owns the one call site, and belonged to
another agent this session — is untouched. Glyphs are re-rasterised rather than
stretched. Measured: layers button and navpad pills **44 → 161 px (2.19 →
8.02 mm)** at 1440x3168 and **44 → 121 px (2.83 → 7.78 mm)** at 1080x2400, icons
17 → 62 and 17 → 47 texels.

### HD-04 — `[display]` has no stretch key, and that is required rather than an oversight · **DOCUMENTED**

Checked rather than assumed, both modes side by side on a 1440x3168 window:

| `stretch/mode` | `get_visible_rect` | final transform | `_phone_scale` would be |
|---|---|---|---|
| `disabled` (shipped, unset default) | 1440 x 3168 | 1.00 | 3.6641 |
| `canvas_items` | 1152 x 648 | 1.25 | 1.6489 |

`canvas_items` breaks the shell three ways at once: the viewport reports the
project's reference size on *every* device, so `_phone_scale` collapses to one
constant and stops tracking the handset; the stretch transform then multiplies
that a second time, differently per device; and 1152x648 is **landscape**, so
`DccShell._landscape` would read true on a phone held in portrait and the whole
§13 portrait composition would be unreachable. Written into the `[display]`
comment block, in semicolons, so the next person does not "fix" it.

### PH-05 — one last hole in the touch-scroll fix · **FIXED**

Re-run at `_phone_scale` 2.748 and 3.664 rather than at the 393 dp reference
every earlier probe used. **6 of 8** points down the left sheet scrolled 329 px.
One of the two that did not is an `HSlider`, which is deliberate and documented.
The other is a **bare `Control`** — `DccTheme.spacer()` and the fixed-width gaps
beside it — which defaults to `MOUSE_FILTER_STOP` and so ends the event walk on
a node that exists only to take up room. `phone_fit()` now passes it through,
matched on the exact class and skipped if anything is listening on `gui_input`.
After: **7 of 8** at 1080x2400 and **7 of 7** at 1440x3168, the remainder being
the slider.

### §46's one carried-over item — a `Popup` is a `Window`, not a `Control` · **FIXED**

Picked up from §46's "Not fixed, and why", where the concurrent phone pass
registered it rather than editing `dcc_shell.gd` mid-flight: with the Layers
sheet up, `_set_drawer_open(true)` left **both** visible.
`_close_all_phone_overlays()` lists the drawer, the panel picker, the phone menu
and both dock sheets — all `Control`s — and `LayersPopover` is a `PopupPanel`,
so no Control walk has ever reached it.

Matched on `Popup` and deliberately **not** `Window`: a popover is transient and
going somewhere else is what dismisses it, while an `AcceptDialog` is a modal the
user is inside, and closing one out from under them would trade a cosmetic
overlap for lost input. `PopupMenu` is caught by the same test and should be.
`find_children(..., owned = false)` because these are built in code and have no
scene owner — the default `owned = true` returns an empty list, which would have
made this another silently-inert fix of the kind HD-02 already collected three
of.

Verified: popover open → `true`, then drawer opened → popover `false` and drawer
`true`.

### Proven negatives — two leads that are not defects

- **`phone_present()` fills the screen correctly; the 31 % fill is a dev-box
  artefact.** `Window.popup(rect)` clamps its rect componentwise to
  `DisplayServer.screen_get_usable_rect()`, which on this dev monitor is
  **1680 x 1002** — and 1002 is exactly the dialog height two independent
  harnesses reported. Established as a formula, not a guess: asked 1440x900 →
  1440x900 (unclamped), asked 1400x1100 → 1400x**1002**, asked 393x852 →
  unchanged, asked 1440x3168 → 1440x**1002**. `is_embedded()` is `true` in every
  case, so embedding is not the discriminator. On a OnePlus 12 that usable rect
  *is* 1440x3168 and nothing is clamped. Confirmed in the real app: at a
  phone-aspect viewport that fits the monitor (440x950) the same code fills
  **100.0 %**. Any probe that simulates a screen taller than the dev monitor
  must re-assert `size` after `popup()` — the window is visible by then, so the
  assignment raises the resize notification and sticks (measured: 1440x3168).
- **The left dock sheet scrolls.** The report of "scroll by zero" came from a
  sweep whose `ScrollContainer` had `max_value` 165.0 against a `page` of 2318 —
  `max_value` is the content length, not the overflow, and 165 < 2318 means
  there was **nothing to scroll**. Measured on a container with real overflow
  (`max` 5409, `page` 2318) the same gesture moves 439 px from 6 of 7 points
  before the PH-05 fix above and 7 of 7 after.

### Registered, not fixed

- **`RD-03` — the base map raster is `TEXTURE_FILTER_NEAREST` and the reference
  smooths.** `viewport_host.gd:792` sets NEAREST on `map_view`; the LOD tiles
  use LINEAR and say why. The frozen reference sets `imageSmoothingEnabled` in
  exactly four places and **all four are the asset library** (the sprite-sheet
  slicer and the item preview) — the map canvas never sets it at all, so it
  takes the HTML default, which is `true`. The port therefore diverges from the
  reference on the map's own filter. Both directions are visible on a phone: a
  512-cell grid on a 1440-wide screen is *magnified* 2.8x, which is the
  blockiness `docs/HANDOFF.md` already quotes the owner complaining about, and a
  2048-cell grid is *minified* 1.42x with no mipmap, which aliases under a
  pinch. Left alone deliberately: the filter belongs to
  `LOD_TILING_INTEGRATION_SCOPE.md` milestone M1, which `viewport_host.gd`'s own
  header already names as the thing that exists to close it, and flipping it
  would change every map screenshot in this repository on a pass whose subject
  was UI chrome.

### Harness

`_hidpi_probe.gd` and `_edge_probe.gd` (new, untracked, like every other probe
in `godot-project/`). The first drives the real shell inside a **`SubViewport`**
rather than the real window, because Windows clamps a window to the desktop work
area: `--resolution 1440x3168` came back as 1440x1031 and `DccShell` classified
the result as a *tablet* (`phone_scale` 2.62, `_phone` false), so the whole run
measured the wrong composition. A `SubViewport` has no such ceiling,
`get_viewport_rect()` inside it reports its own size, and `gui_embed_subwindows`
keeps the shell's dialogs rendering into the same texture. `--vp WxH` selects the
device. It measures fonts, icon texels, tap sizes in millimetres and the scroll
flick, and it is the first probe in this repository that runs at a phone scale
above 1 — **this entire class of defect is arithmetically invisible at the 393 dp
reference size every earlier probe booted at.**
