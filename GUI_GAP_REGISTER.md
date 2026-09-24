# GUI gap register — every disconnected control, its design, and where none exists

> **Read this document as history, not as status.** It is a register of what
> each audit and device pass *found and decided on its own date* — the design
> research, the root causes, the measurements and the rulings, all of which stay
> useful. It is not a statement of where any of it stands now: a row reading
> "done" or "not fixed" records the pass that wrote it, not the tree today.
> **Current status lives in `cartalith-native/docs/STATUS.md`**, the only place
> progress is recorded, and the live successor to this register's open half is
> `UNWIRED_FUNCTIONS.md`.

> Owner request, 2026-08-19, verbatim: *"verify that all GUI elements are tested,
> connected and where it doesn't connect to other menus or functions designs have
> been made to be implemented. If not, research the menu naming, documentation in
> the design, and where you still have gaps find references in similar
> applications."*

**The premise does not hold, and that is by design.** The shell does not have a
small number of stragglers to finish connecting: it has **241 catalogued
disconnected surfaces**, of which **73 are still open** (123 at this document's
original writing; recounted 2026-08-24 and 2026-08-25; re-derived and split from
the 101 *driven-defect* entries that share its numbering on 2026-09-07 — §3 has
the method and the caveats), every one of them added
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
| [6](#6--layer-1--2-the-catalogue) | **Layer 1 + 2 — the catalogue** (241 catalogue entries as re-derived 2026-09-07 — 139 closed, 61 (B), 12 (C), 23 (D), 6 unresolved, **0 (A)**; 215 at the 2026-08-24 recount, 123 at original writing) |
| [7](#7--layer-3--comparable-application-research-for-c) | **Layer 3 — comparable-application research** for every (C) |
| [8](#8--menu-naming-audit) | **Menu naming audit** |
| [9](#9--d-entries-owner-decisions-not-gaps) | (D) entries: owner decisions, not gaps |
| [10](#10--the-actionable-a-list-in-priority-order) | The actionable (A) list, in priority order |
| [11](#11--out-of-scope) | Out of scope for this register |
| [12](#12--verification) | Verification |
| [13](#13--the-v210-menu-structure-audit-2026-08-20) | **The v2.10 menu-structure audit** — `design/Cartalith Menu Structure v2.dc.html` against the shipped shell, and the 17 undisclosed omissions it found |
| [14](#14--visual-sweep-2026-08-20) | **Visual sweep (2026-08-20)** — the shell driven live, screenshotted, and compared against the DCC Shell / Journey Planner mockups. **§14.6 corrects one of its own verdicts**: the Asset library window was passed on function rather than layout, and has been rebuilt against the canvas. |
| [15](#15--the-phone-overflow-menu-is-wired-but-inoperable-2026-08-20--resolved-2026-08-23) | **The phone overflow menu (2026-08-20)** — (C): the real menu bar is wired into the phone sheet but is unscaled, buried in desktop status chrome, and inert to touch. Device evidence, kept as the brief for the mobile menu design; **not fixed** when written, **resolved 2026-08-23** (§15.1-§15.2). |
| [45](#45--mn-10-rl-01-ca-20-rf-02rf-05-fi-04--the-is-every-control-wired-sweep-2026-08-25--seven-fixed) | **The "is every control wired" sweep (2026-08-25)** — seven fixed: **MN-10**, **RL-01**, **CA-20**, **RF-02/RF-03** (both destructive), **RF-04** (the first signal-**ordering** bug), **RF-05** (RF-01's fifth recurrence) and **FI-04**; plus the **negative results** — 89 surfaces across three worlds, 110 ranges and 11 option buttons, 148 menu items and all 35 Layers entries measured in pixels — so the next pass does not re-walk them. |
| 16-22, 27-38 | Sections added after the contents table was written; see the `## ` headings directly. The 2026-08-25 batch: **§40** narrows CV-25 and CV-26; **§38** is the conformance sweep (FR-02, PE-01 — one defect: a field that commits on `focus_exited`, torn down while focused — plus SH-11's 32.59 px zoom-pivot drift and WW-13); **§37** is the left-rail menu structure v3 pass and its fifteen new IDs (CV-21…CV-26, IN-13, CA-16…CA-19, WW-14, WW-15, VA-01, VA-02); §32 (deep zoom) is the same batch. §29, §30 and §31 are the 2026-08-24 live-driving batch. |
| [25](#25--bk-01--androids-back-button-killed-the-process-unsaved-world-and-all-2026-08-24--fixed) | **BK-01 (2026-08-24)** — the highest-severity entry in this register: Android's Back button ended the process, taking an unsaved generated world with it. Root cause, the navigation model that replaced it, and two related findings (BK-02, BK-03). **Fixed.** |
| [26](#26--bk-02--the-desktop-close-box-did-the-same-thing-and-the-reason-it-was-left-alone-was-answerable-2026-08-24--fixed) | **BK-02 (2026-08-24)** — BK-01's twin on the desktop close box, fixed onto the *same* shared gate, with the four-branch argument for why `auto_accept_quit = false` cannot leave the app un-closeable. **Fixed.** |
| [23](#23--rf-01--the-civil-dock-never-rebuilt-after-a-world-generated-2026-08-24--fixed) | **RF-01 (2026-08-24)** — a new class, not a capability gap: the whole CIVIL dock (ten sections across two files) was built once at launch and never rebuilt on generate or load. **Fixed**, with the presentation-vs-recompute cost check. |

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

**The current reading is the 2026-09-07 re-derivation below.** The total was
counted four times; every earlier figure went stale the same way — sections were
added faster than the hand-maintained prefix list moved. The dated history,
oldest first, each with the reason it went stale:

| Date | Total | How it was derived | Why it went stale |
|---|---|---|---|
| original writing | **123** catalogue entries | computed once | never re-derived as §16-§22 and roughly 80 rows were added (`PARITY_AUDIT.md` pass 2, F7 — the audit's own "~65% off" finding) |
| 2026-08-24 | **215** distinct gap IDs | grep of markdown table rows (below) over a hand-kept prefix list — `AS-`, `CA-`, `CV-`, `CX-`, `DM-`, `DV-`, `ED-` (including `ED-03a`-`ED-03d`), `FI-`, `FR-`, `HE-`, `IN-`, `JP-`, `MEA-`, `MS-`, `PR-`, `RD-`, `RN-`, `SG-`, `SH-`, `UM-`, `WI-`, `WL-`, `WW-` — plus the `### PH-0N ·` headings of §22, de-duplicated (§9 regroups every (D) row, §13 cross-references existing IDs, §10 restates (A) rows; none is a new entry) | §37-§48 added eleven sections and the prefix list did not move with them |
| 2026-08-25 (audit pass 4) | **249** as table rows; **300** as bare tokens | pass 2's row grep re-run unchanged (`PARITY_AUDIT.md` pass 3 §19 point 1 reached 249 independently); the bare-token grep (`<PREFIX>-<NN>` anywhere) also catches the families written as **headings rather than rows** — `PH-01`…`PH-12` (§22, §46), `DS-01`…`DS-14` (§48), `HD-01`…`HD-04` (§47), plus `BK`, `RF`, `SB`, `LZ`, `MT`, `PE`, `RL` and `VS` | its own totals were wrong: the same two greps return 250 / 342 on 2026-09-07 |
| 2026-09-07 | **250** / **342** IDs; **241** catalogue entries | below | — |

Notes that still apply to the earlier figures. The 2026-08-24 row grep matched rows beginning `| <PREFIX>-<NN>` — the form nearly every ID in §6, §9, §10 and §17-§21 uses. The 215 did not match the
audit's own "~203" because that was an estimate, not a grep result: its §14
point 3 asked for the counts to be re-derived rather than deriving them, and
§13 says why ("a classification pass, not an arithmetic one"). At 2026-08-25
thirty-nine prefixes were in use, **fifteen** of them missing from the
2026-08-24 list — `BK`, `DS`, `HD`, `KV`, `LZ`, `MN`, `MR`, `MT`, `PE`, `RF`,
`RL`, `SB`, `TO`, `VA`, `VS` — and its "six `### PH-0N` entries" was twelve.
`O1`-`O9` (§5's omissions) and `S1`-`S5` (§4's stale reasons) use a
different, non-hyphenated numbering and are **not** counted: each is its own
small table, not part of the catalogue. And a group of identically-blocked
sibling controls (the ten Edit-menu items, the five erosion Run buttons) is one
entry, so the raw count of individually disabled controls is higher than any
of these totals.

**The A/B/C/D split was not re-derived at any of the three earlier recounts.**
The stated reason: grepping each row for its `**(A)**`/`**(B)**`/`**(C)**`/`**(D)**`
marker (the `grep` the 2026-08-24 pass used) finds one on only 54 of the 215 rows; the other 161 lost their letter
when the row was edited to record closure (e.g. `AS-01`'s row reads *"done,
2026-08-20 ... real ... `as_import_item`/`as_add_custom_slot` are wired"* with
no `(A)`/`(B)` marker), and recovering each dropped letter means reading the row's
history — the "judgment per row" `PARITY_AUDIT.md` pass 2 itself declined (§13).
That was true, and it was **the wrong thing to recover**: the letter that
matters is the one each entry **earns today**, from its own verdict cell and,
where that is not enough, from the shipped symbol. That is what the 2026-09-07
pass did.

---

### Re-derived 2026-09-07 — the class split, rebuilt, and why it could be rebuilt this time

Pass 4's own two greps, re-run on 2026-09-07 (figures are as of that date):

```
# markdown table rows beginning "| <PREFIX>-<NN>"  ->  250 distinct IDs (280 rows)
grep -oE '^\| *\*{0,2}[A-Z]{2,4}-[0-9]{2}[a-z]?\*{0,2} *(\||[[:space:]])' GUI_GAP_REGISTER.md \
  | grep -oE '[A-Z]{2,4}-[0-9]{2}[a-z]?' | sort -u | wc -l
# the bare token <PREFIX>-<NN> anywhere, de-duplicated  ->  342 distinct IDs
grep -oE '\b[A-Z]{2,4}-[0-9]{2}[a-z]?\b' GUI_GAP_REGISTER.md | sort -u | wc -l
```

The row grep matches **280 rows** for those 250 IDs — several IDs legitimately
carry a row in more than one section. **Forty-two** prefixes are in use
(`… | sed 's/-.*//' | sort -u | wc -l`) — pass 4's thirty-nine plus `BI`, `FX`
and `MEM`, from §52 and §54-§56.

#### The obstacle: two different kinds of entry share one numbering scheme

**The legend classifies a *missing or disconnected surface*. Roughly a third of
today's IDs are not that — they are *defects in shipped behaviour*,** found by
driving the app: the map right-click menu having no touch route (**PH-02**), the
phone menu covering the bottom bar so two taps in five did nothing (**PH-22**),
the flow overlay never seeing the camera (**FX-01**), a `has_method()` guard
failing silently against a stale `.so` (**SB-01**). Asking which letter those
take is a category error, and treating the register as one population forces
that error onto a third of it — which is why the three previous passes stopped.

So the population is split first, mechanically:

- **Catalogue entries — 241.** An ID whose defining occurrence is a row in one of
  the six gap-catalogue tables — **§6** (the catalogue proper), **§13**'s v2.10
  omissions, **§16**'s measurement-toolbar rows together with **§17**'s debug
  views, **§18**'s civ-interaction rows, **§21**'s staleness rows and **§37**'s
  v3-rail rows — plus the five capability entries §34 and §35 write as headings
  rather than rows (`RN-04`, `CA-14`, `KV-01`-`KV-03`). Resolve those by
  heading, not by line. **This is the population §3 has always been counting**,
  and it is the only one the legend describes.
- **Driven findings — 101.** Everything else: every ID first written by the
  live-driving, phone, hi-DPI, conformance, menu-by-menu, memory and overlay
  passes of §22-§58. They are not classified here; their own section headings
  carry their verdicts. **Six are explicitly registered-not-fixed or
  re-opened** — DS-03's tablet *interior* and DS-13's phone-map *composition*
  (both re-opened by §57), `KV-04`, `WW-16`, `PH-16` and `PH-28`.

#### Method for the 241

1. Each entry was read at its own row. Where the row's own verdict did not settle
   it, the **shipped symbol** was opened — that is how `HE-01`/`HE-04`
   (`menus.gd::_help`), `WI-01`-`WI-05` (`menus.gd`'s Layouts popup,
   `_dock_drag_handle`, `resource_overlay.gd`), `SH-01`
   (`dcc_shell.gd::_build_rail`), `ED-05` (`place_search.gd`), `PR-07`
   (`render_workspace.gd`'s Colour management section) and `WW-03`
   (`Falloff::ALL` in `cartalith-terrain/src/sculpt.rs`) were settled.
2. **A partly-closed entry is counted by its open remainder's class**, since that
   is the question this table exists to answer. `DM-02` is (B) because the
   slippy-map half is what is left; `CV-23`/`CV-25`/`CV-26` are (B) for the same
   reason; `CA-19` is closed because its named remainder is a separate larger
   item, not this entry's residue.
3. **The register was checked for staleness in both directions.** It is stale
   *pessimistically*: **nine §6 rows still print `**(C)** → §7.x` in their
   Class cell for a surface that is built** — `ED-05`, `PR-07`, `PR-15`,
   `PR-16`, `WI-01`, `HE-01`, `HE-02`, `HE-03` and `WW-03`.
   (`grep -cE '^\| [A-Z]{2,4}-[0-9]{2}[a-z]? \|.*\*\*\(C\)\*\* → §7'` returns
   **15** — anchored to the row start because this paragraph contains the same
   string; the six the cell still describes correctly are `DM-06`, `DM-10`,
   `DM-11`, `WW-04`, `WW-05` and — differently — `CA-07`.) Two examples of how
   far a cell can drift: `HE-02`/`PR-16`, the shortcuts editor, was **found
   already built** when a lane opened `shortcuts_dialog.gd`; and
   `PR-15`/`MEA-06`'s *"this shell has no unit preference at all"* is false —
   `Preferences ▸ Units` is a three-way `km / mi / nmi` radio with
   `dcc_units.gd` as the display layer under it. **Those nine cells are left
   standing on purpose**: rewriting Class cells is a reclassification pass and
   would put two authorities in one document. §3 is the authority; the cells
   are the history.
4. **Six entries are left unresolved rather than guessed**, and are counted as
   their own row below.

| Class | Count (of 241) | Share |
|---|---:|---:|
| **(A)** designed + engine-ready | **0** | 0 % |
| **(B)** designed, engine-blocked | **61** | 25 % |
| **(C)** undesigned | **12** | 5 % |
| **(D)** deliberate decision | **23** | 10 % |
| **(✓)** closed — built, fixed, or resolved by decision | **139** | 58 % |
| **unresolved** — state not established this pass | **6** | 2 % |
| **Total (catalogue entries)** | **241** | |

**(A) is zero, and it is a result rather than an artefact.** Every row this
register ever classed (A) has shipped: `RD-03`/`RD-06`/`RD-08`/`RD-11`/`RD-13`
(2026-08-19), `SH-05`/`SH-09`/`SH-10`/`SH-11`/`SH-12`/`SH-13`/`SH-14`,
`JP-12`-`JP-15`, `WW-13`, `HE-04`, `PR-13`/`PR-14`, `WI-02`-`WI-05` and
`MEA-06`. §10's actionable list is empty of live rows. **This is the register's
strongest single finding and the one most worth re-checking**, because a zero is
exactly the shape a missed row hides in.

**(C) fell from 23 of 123 to 12 of 241** — a real collapse, not a denominator
effect: fifteen of §7's twenty-five (C) IDs are closed (`ED-02`, `ED-05`,
`PR-07`, `PR-11`, `PR-15`, `PR-16`, `HE-01`, `HE-02`, `HE-03`, `WI-01`, `JP-05`,
`WW-03`, `CA-08`, `CA-09`, `SH-01`); three became (D) when the owner deleted the
Conversion route outright (`DM-07`-`DM-09`, 2026-08-20); **`CA-07` moved (C) →
(B)** — the label *role* system shipped 2026-09-06 (a counted apply over a role
template, driven against a real slider). **The reason first written here was
wrong and is corrected 2026-09-07:** it said family, weight and case all have
no field in the engine's label model. **Family does** — `MapLabel::font:
Option<String>` (`cartalith-civ/src/labels.rs:87`) with `font_or_default()`, a
`set_font` bridge setter, a getter and a `LabelStyleSnapshot` round-trip;
`lib.rs` says *"sending the literal string via `font` already works today"*.
**Weight and case genuinely have none** — the `weight` at `labels.rs:945` is a
placement rank within a class, not a font weight. So (B) survives, but on the
RENDERER, which is what this row's own Blocked-by cell always said: Godot has
no web-font fallback chain to resolve the string the model already carries.
Six of §7's remain: `DM-06`, `DM-10`, `DM-11`, `PR-10`, `WW-04`, `WW-05`. The
other six (C) entries come from outside §7 — `CV-24` (§37) and
`MEA-03`/`MEA-05`/`MEA-08`/`MEA-09`/`MEA-10` (§16). **Two of the sixteen were
closed by discovering they were already built**, which is why the count moved
further than any build log would predict.

**The (B)-by-cost table is *not* re-derived.** Only 28 of the 61 (B) rows state
a cost word in their own text — 2 wrapper, 16 small, 10 large — and inferring
the other 33 means sizing a crate change per row, a different exercise from
classification. Left stated rather than estimated.

**The six unresolved, with the reason each is unresolved:**

| ID | Why it is not classified |
|---|---|
| `CA-18` | Zoom ladder: `CIV_LOD_ROAD` is ported, but whether the remaining declutter budget and per-layer ranges are engine-blocked or merely undrawn was not established |
| `CV-03` | Timeline filters: *Exist only* is wired; the row does not say what became of *Ghost removed* and *Highlight new*, and neither was driven this pass |
| `ED-03a` | Specialisation → `civ_faction_aggregates`: the row calls it *"a decision to take deliberately"*. An undecided decision is neither (B) nor (D) |
| `ED-03c` | Per-trait map glyphs: the data is stored and the reference draws them, but no design in this shell's own vocabulary was found for them |
| `JP-06`, `JP-08` | Both were *"partly closed, session-only, blocked on FI-01's save writer"*. **FI-01 has since closed**, so the stated blocker is gone — whether persistence followed was not verified |

#### Three ID collisions, found while counting

**`RD-01`, `RD-02` and `FI-04` each name two different things**, so the
distinct-ID count under-counts entries by three:

| ID | §6-catalogue meaning | Later-pass meaning |
|---|---|---|
| `RD-01` | §6.8 *Settlement ▸ Defensibility* | §29 *the roads curve, and the renderer was drawing their chords* |
| `RD-02` | §6.8 *Settlement ▸ Routes* | §36 *five land way types, one colour* |
| `FI-04` | §6.1 *Revert to last save* | §45 *copy that named a place the user cannot look* |

`RD-03`-`RD-13` are all right-dock; `RD-01b` belongs to the *roads* family, not
to the right dock. Nothing is renumbered — a register that renumbers breaks
every citation into it from `OUTSTANDING_WORK.md` and from the shell's own
comments — but a reader resolving an `RD-` citation must check which family it
means, and the counts above resolve each collision by its catalogue meaning.

**A fourth, found 2026-09-23 while condensing this register:** `RD-03` is §6.8's
right-dock *Settlement ▸ Economy / Politics / Logistics* buttons and also §47's
*base map raster is `TEXTURE_FILTER_NEAREST`* (registered, not fixed). The
2026-09-07 counts above predate the finding and were not re-run.

**The earlier reading, kept as the historical snapshot.** Computed once against
the 123-entry catalogue; superseded by the 241-entry table above.

| Class | Count (of 123, stale) | Share (of 123, stale) |
|---|---:|---:|
| **(A)** designed + engine-ready | **17** | 14 % |
| **(B)** designed, engine-blocked | **71** | 58 % |
| **(C)** undesigned | **23** | 19 % |
| **(D)** deliberate decision | **12** | 10 % |
| **Total (superseded, see above)** | **123** | |

(B) by cost, against the same 123-entry / 71-(B) total:

| Cost | Count | Notes |
|---|---:|---|
| **wrapper** | 22 | The single largest cheap win in the register. Nearly all of it is three subsystems: `TerrainAppearance` (RENDER + CARTO's LIGHT group), `AssetDB` (the whole Asset library window), and the Journey Planner's cost model. |
| **small** | 21 | |
| **large** | 28 | Dominated by five subsystems: the save writer, global undo + selection, the Data manager's import/conversion/validation routes, the colour-ramp/separable-layer system, and river-as-entity. |

How quickly that snapshot went stale: **2026-08-20** — ten of the (B)-wrapper
rows (AS-01 through AS-08, AS-13, DM-05) closed (`ASSET_LIBRARY_SCOPE.md` §10),
and **JP-02**/**IN-06** closed with the Travel Library's party-form wiring
(`TRAVEL_LIBRARY_SPEC.md` §6). **2026-08-23** — §6.16 (`PARITY_AUDIT.md` C3)
added three previously uncatalogued (B)-large entries (UM-01/02/03); §5's
O4/O5/O7/O8 moved to done (`PARITY_AUDIT.md` C5); the Journey Planner's closing
pass took **JP-01, JP-03, JP-04, JP-05, JP-07 and JP-09** to closed and **JP-06 /
JP-08** to partly closed, and re-closed **IN-06**'s remainder with the vessel
resolver. **2026-08-24** — §17-§22 added roughly 45 more IDs (`DV-01`-`DV-11`,
`ED-03a`-`ED-03d`, `CV-10`-`CV-13`, `WW`'s erosion-parameter rows,
`SG-01`-`SG-03`, `PH-01`-`PH-10`, `SH-09` through `SH-12`, and others).

**The shape, as read against the 123.** Only 19 % of the shell's disclosed gaps
were genuinely undesigned. 58 % had a design and were waiting on the engine —
and **31 % of those (22 of 71) were waiting on a boundary wrapper, not a
capability**, the same finding `DCC_CONTROL_INDEX.md` summary §1 reached from
the other direction ("two whole regions of this design are a boundary-wrapper
problem, not a capability problem").

**Answered 2026-09-07, and the shape has inverted.** Against the 241 catalogue
entries: **58 % are closed**, and of the 73 still open, **84 % are (B) —
designed and waiting on the engine — against 16 % (C)**. Most of the register is
now history, the undesigned share collapsed from 23 entries to 12, and **(A) is
empty**. The one part of the old shape that could not be re-tested is the
wrapper share: 33 of the 61 (B) rows state no cost (corrected from *"32 of the 60"*, which disagreed with the table's 61 and the 28-stated / 33-unstated split above), so *"31 % of (B) is a
boundary wrapper"* is neither confirmed nor refuted here.

---

## 4 · Stale disclosed reasons: five fixed in this pass

Every `_todo()` tooltip, `_gap_button()` reason and `note()` in the shell was
re-checked against the engine as it stood. Five were factually stale. **Only the
reason text was changed; no control changed state, and no behaviour changed.**
All five are corrections of fact, not design.

| # | File:line (pre-edit) | Was | Why it was wrong | Now |
|---|---|---|---|---|
| S1 | `right_dock.gd:608` | Faction ▸ Territory — *"no per-faction cell count or area query exists"* | **Two** now exist: `civ_faction_territory_stats(faction)` returns `claimed_cells`/`area_km2`/`contested_cells`, and `get_factions()` (`lib.rs:3442`) carries `claimed_cells` per faction. `civilization_workspace.gd:350-358` already flagged this in a comment ("true when that sentence was written, no longer true") but could not edit the file. | Says the queries exist, names both, points at CIVIL ▸ Territory's options row where the live numbers already show. |
| S2 | `app.gd:281-284` | CIVIL/INFRA idle context — *"the §4.5 tool palette to arm them is not built yet"* | The TOOLS block **is** built, in both docks (`civilization_workspace._build_tools()`, `infrastructure_workspace._build_tools()`). Both files say so in their own comments and note `app.gd` was out of their scope. | Names the tools each dock actually offers, and that POI is absent for a real engine reason. |
| S3 | `journey_planner_view.gd:1519` | Cost group — *"the reference's own cost model, if any exists past the HTML's own UI layer, has no Rust port"* | **False.** `cartalith_civ::jp_journey_cost` (`cartalith-civ/src/lib.rs:6885`, ported from `jpJourneyCost` reference line 18873) computes carriage/wages/crew/upkeep/tolls/transshipment/total/per-tonne-km/break-even, with golden tests (`journey_cost_prices_a_mixed_land_and_sea_trip`). It is simply never called. Every input is already computed inside `jp_plan` (`JpDerivedStage::claimed_frac`, `JpJourneyPlan::transshipments`, per-leg km/days/crew). | Says the model is ported, names it, names the three inputs, and calls it a boundary gap rather than a model gap. |
| S4 | `menus.gd:338` | Tiled LOD · tile size · atlas cache — *"No tile atlas yet."* | Stale in part. Deep-zoom LOD tiling is **live and automatic** (`lod_synthesize_tile`/`lod_tile_cells`, driven by `viewport_host.gd`'s `_lod_backlog`/`MAX_LOD_TILES_PER_UPDATE`). What does not exist is §2.5's *controls* and the *persistent* atlas. | Separates the two: tiling is live, the four preference rows and the on-disk cache are not. **Re-corrected 2026-08-24**: the on-disk cache exists now (PR-10/WW-01), so the row names the *preference surface* as the remaining gap. |
| S5 | `world_workspace.gd:292` | Finalize — *"cartalith-spatial exists standalone, unintegrated"* | Stale: `cartalith-spatial` gained real consumers on 2026-08-18 (`PassBuffer`/`StageGraph`, then LOD tiles). The bake/freeze half of the claim is still true. | Keeps the true half, drops the false half, cites `LOD_TILING_INTEGRATION_SCOPE.md`. **Obsolete 2026-08-24**: the disabled button and its disclosure are gone, replaced by the live three-row Finalize block (WW-01). |

### Borderline, deliberately not edited

- `right_dock.gd:674` Region select ▸ *"the Data Manager panel to call it doesn't exist yet"* — accurate as written at the time (the window existed, the Export ▸ Maps **panel** did not). **Superseded 2026-08-20**: the panel was built (§14.7); tooltip and disable are gone — RD-09.
- `cartography_workspace.gd:277` *"no on-canvas resize handle yet for a placed icon (`icon_bridge.rs`'s own acknowledged gap)"* — `icon_resize`/`icon_hit_test` **are** exposed, so the emphasis read more engine-blocked than it was, but `icon_bridge.rs:216` really did say *"`None` handle — no on-canvas resize-handle geometry"* (no `icon_handles()` to match `label_handles()`). Recorded as **CA-05**, an (A) item. **CA-05 CLOSED 2026-08-24** — `icon_handles()` exists; see CA-05's row in §6.13.
- `infrastructure_workspace.gd:13-14`'s class doc — *"Logistics … exports nothing past that crate boundary"* — is stale, but the same file's `_build_logistics()` says so explicitly two hundred lines later. A code comment, not user-facing text. Left alone.

---

## 5 · Omissions: designed, not present, not even as a disabled item

The honesty rule has two halves — *never enabled-and-inert*, and *never
omitted*. The first half holds everywhere. The second had **nine breaches**
when this table was first built; **six are now closed** (O1, O3, O4, O5, O7,
O8 — O4/O5/O7/O8 corrected 2026-08-23, `PARITY_AUDIT.md` C5, having sat marked
open here while §6.5/§6.6/§6.9 already recorded them done). **Three remain real
breaches**: O2, O6, O9.

| # | Missing surface | Designed in | Class |
|---|---|---|---|
| O1 | **`Data ▸ ⧉ Travel library… ⇧L`** — the whole menu item and window | `DCC_SHELL_SPEC.md` §2.4's 2026-08-19 addition; `TRAVEL_LIBRARY_SPEC.md` in full | **done, 2026-08-19** — see DM-15 |
| O2 | **`Assets ▸ Asset pack ▸`** — the entire submenu (Active pack / Pack metadata… / Edit / Batch / Build / Clear library…), 24 controls | `DCC_SHELL_SPEC.md` §2.3.1 | (B) wrapper |
| O3 | **`Preferences ▸ Performance ▸ Fallback when VRAM full`** | `DCC_SHELL_SPEC.md` §2.5 | **done, 2026-08-20** — see PR-05 |
| O4 | **`Preferences ▸ Application ▸ Theme ▸ follow system`** | `DCC_SHELL_SPEC.md` §2.5 | **done, 2026-08-19** — see PR-14 §6.5; verified live via `DisplayServer.is_dark_mode()`/`is_dark_mode_supported()` in `menus.gd:570,890` |
| O5 | **`Window ▸` the workspace list**, and **open windows listed while open** | `DCC_SHELL_SPEC.md` §2.6 | **done, 2026-08-19** — see WI-02/WI-03 §6.6; verified live at `menus.gd:927-937` (the `Workspace` submenu over `DccShell.DOMAINS`/`select_domain()`; Open windows rebuilt every `about_to_popup`) |
| O6 | **New world ▸ project *name* field** | `DCC_SHELL_SPEC.md` §2.1 ("Modal: name, seed, extent, working resolution") | (B) small |
| O7 | **The Journey Planner's timeline band** — "one band per day, coloured travel / water / weather hold / rest-layover". `timeline_bar` is *visible and empty* while JOURNEY is armed. | `JOURNEY_PLANNER_SPEC.md` §2 | **done, 2026-08-19** — see JP-13 §6.9 (`_rebuild_timeline_band()`/`_TimelineBandView`) |
| O8 | **Blocked-stage inline resolutions** — "offers its resolutions inline (turn off closures, re-route land-only, depart earlier)" | `JOURNEY_PLANNER_SPEC.md` §9 | **done, 2026-08-19** — see JP-14 §6.9 (`_blocked_resolution_row()`) |
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
> absorbed. All seventeen are catalogued and closed in **§13**, the
> continuation of this section. **CV-07** is the one row this section and §13
> share: registered here as absent with no disclosure, it now has one.

---

## 6 · Layer 1 + 2: the catalogue

Every disconnected surface, where it is, what it is called in the UI, its
current disclosed reason, whether that reason is still accurate, and its
classification with the design cited.

### 6.1 File menu — `menus.gd`

| # | UI label | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| FI-01 | Save project | 115 | ~~no save writer (`cartalith-io` read-only)~~ | **CLOSED (2026-08-23)** | §2.1 | **DONE.** `cartalith_io::write_save` (`crates/cartalith-io/src/save.rs`) + `WorldGen::save_project`. The byte-compatibility bar was the work, not the zip. Writes the seven documented entries in `exportZip()`'s own order, DEFLATE, and carries every generation parameter **twice**: at its reference `state` path so the HTML app can reopen the file, and under `state.cartalith` so this port's ten reference-less parameters are not lost. Built in memory and written once, so a failed save never truncates the file it replaces. Verified three ways, including re-writing a **real** HTML-app export against that fixture's independent value capture. Format decisions and the one disclosed limitation (`state.erosion` — unshimmed by `loadZip()`, only 2/16 keys modelled, so writing it partially would be worse than not writing it) are in `SAVEFILE_COMPAT.md` |
| FI-02 | Save as… | 116 | ~~same~~ | **CLOSED (2026-08-23)** | §2.1 | **DONE.** `DccBrowseDialog` grew a third `PickKind`: `SAVE` is `FILES` plus a name field in the foot — so no stock `FileDialog` survives on this path. Clicking an existing save fills its name in (the overwrite gesture), and overwriting asks first |
| FI-03 | Autosave | 117 | ~~requires a save writer~~ | **CLOSED (2026-08-23)** | §2.1 | **DONE**, as a check item plus a `Timer`, interval in `DccSettings` (machine state, not world state). **Off by default** — a background writer of hundreds of megabytes that starts unasked is the wrong first impression; this is the row's "owner policy" half, decided and disclosed. Writes **beside** the project (`world.zip` → `world.autosave.zip`), never over it, and does not clear the unsaved flag. Reports through the status bar's `autosave` slot, which also makes `phone_menu.gd`'s `autosave` readout row real for the first time |
| FI-04 | Revert to last save | 118 | ~~requires a save writer~~ | **CLOSED (2026-08-23)** | §2.1 | **DONE**, exactly as predicted: `load_save` on `current_project_path`, behind a confirm, since the discard is irreversible and the item sits two rows under Save |
| FI-05 | Close project | 120 | ~~no project lifecycle~~ | **CLOSED (2026-08-23)** | §2.1 | **DONE.** `EngineBridge.close_world()` replaces the `WorldGen` handle — the engine has no `unload`, and this is the only way to release the field memory — and re-reads the two caches taken off the old instance. The prompt in front of it could not exist before: with no writer there was no **Save** to offer. It prompts whenever a world exists, not only when `world_dirty` is set — that flag rides `generation_finished`/`world_loaded` and cannot see a Milestone-F tool commit |
| FI-06 | *(missing)* project name field | — | **none — omission O6** | — | §2.1 | (B) small — no name field on `WorldGen` or `cartalith_io::SaveData` |

### 6.2 Edit menu — `menus.gd`, all ten disabled

| # | UI label | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| ED-01 | Undo / Redo | 172-173 | no undo stack; generation one-shot, sculpt has no Godot binding | **Undo: CLOSED 2026-08-23.** Global heightmap undo is live — `Edit ▸ Undo` (Ctrl+Z), `undo.rs` + five `#[func]`s, pushed by `sculpt_commit` and `carve_fjords`, exactly the reference's own `pushUndo` call sites minus the eight erosion passes this port does not run. The row shows the operation name and depth. **Redo stays disabled and always will**: the reference has no global redo — `undoLast()` *pops* the snapshot rather than moving a cursor. The Sculpt draft's own Redo (right dock) is a different, real thing. | §2.2 | ~~(B) large~~ → **done**. The reference is not a general command/diff framework: 3 functions, a `Float32Array.slice()` and a 5-deep stack. See §7.1's revised entry |
| ED-02 | Undo history… | 174 | same | **Open at 2026-08-23, for a sharper reason** — the *stack* is real (`undo_stats()` reports depth, bytes, budget and the next label); what did not exist is a panel over it. The live depth/cost readout landed in `Preferences ▸ Memory ▸ Undo history` instead (PR-11), where the reference's own `#undoMem` sat | §2.2 names it in one line; **no panel design exists** | **CLOSED 2026-08-25 (§42)** — built as the *ledger* §7.1 asked for, not the five-row list: `undo::HistoryLedger` records every commit and reverses the ones it can (`▲` snapshot / `·` recorded-only with its reason / `◼` floor), reversibility read from the live stack depth so the two cannot drift. Opens as a right-dock context, per proposal 3. Full account in §42 |
| ED-03 | Cut / Copy / Paste / Delete | 176-179 | nothing selectable beyond settlements, which are read-only | **CLOSED (Delete and edit) 2026-08-23** — see §18: the place-edit popup (`place_editor_window.gd`), the right-click context menu (`map_overlay.gd`'s `map_right_clicked` → `civilization_workspace.gd`) and the `KEY_DELETE` handler (`app.gd`); §18.3 lists the four residual sub-gaps (ED-03a..d). Cut/Copy/Paste remain open — no clipboard model exists for any entity. **Earlier correction, 2026-08-23** (`PARITY_AUDIT.md` C3/§3.2/§5 item 3): the row had been mischaracterised as a clipboard/selection gap. The real finding was that `civ_drop_settlement` **creates** a settlement and nothing **edits, moves or deletes** one — no place-edit popup (the reference's `placeEditPopup`/`_civPopulatePlaceEditor` unported), no map right-click handler (`_civCtxShow`'s six operations; `PopupMenu` appeared only in `menus.gd`/`dcc_shell.gd`, never on `MOUSE_BUTTON_RIGHT` over the viewport), and no `KEY_DELETE` handler anywhere under `godot-project/`. Labels, icons and sculpt stamps are selectable and deletable through their own panels, which is why the original framing looked plausible. | §2.2 | (B) large — a place-edit popup, a map context menu and a Delete-key handler are three separate missing pieces, not one selection abstraction |
| ED-04 | Select all / Deselect | 181-182 | same | same | §2.2 | (B) large — same model |
| ED-05 | Find on map… | 184 | no search index; settlement search lives in the Data manager | yes | §2.2 gives one line; **no search UI design** | **(C)** → §7.2 |

> ED-03/ED-04's reasons are stale in *emphasis* — they describe a shell
> that had no tools. Not corrected here because rewriting them correctly means
> describing the selection split, which is a paragraph, not a tooltip.
>
> **Edit is no longer 100 % disabled.** §8.4's recommendation — move something
> into Edit so the menu has one live item — was overtaken by ED-01: `Undo` is
> that item, and the one every comparable application puts there first anyway.

### 6.3 Assets menu + Asset library window

| # | UI label | Where | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| AS-01 | Import image… | `menus.gd:201`, `asset_library_window.gd:402` | **done, 2026-08-20** | real | §2.3, §8 | `as_import_item`/`as_add_custom_slot` (`asset_bridge.rs`) are wired; targets whichever slot is focused in the grid |
| AS-02 | Apply library to map | `menus.gd:241`, `asset_library_window.gd:325` | **done, 2026-08-20** | real | §2.3, §8 | `as_apply_to_map` — the reference's own `applyToMap()`: bake the session in memory (`export_pack_bytes`), load it straight into the renderer, no round trip through a file |
| AS-03 | Clear library… | `menus.gd:243`, `asset_library_window.gd:513` | **done, 2026-08-20** | real | §2.3, §8 | `as_clear_library` -> `AssetDB::clear` |
| AS-04 | Export pack .zip | `asset_library_window.gd:327` | **done, 2026-08-20** | real | §8 | `as_export_pack_bytes` — bakes every item, builds a schema-2 manifest, `archive::write_pack`; disk round-trip verified headlessly (`ASSET_LIBRARY_SCOPE.md` §10) |
| AS-05 | Validate | `asset_library_window.gd:511` | **done, 2026-08-20** | real | §8 | `as_validate` -> `library::run`, shown in a modal |
| AS-06 | Tag… / Collect… / Rename… / Duplicate / Delete (batch) | `asset_library_window.gd:436` | **done, 2026-08-20** | real | §8, §2.3.1 | `as_batch_tag`/`_collect`/`_rename`/`_duplicate`/`_delete`, each read off the reference's own `alBatch*` handlers. `rename` stays honestly split: a custom slot is renamed for real, a frozen slot renames its *item variants* (`AssetDB::item_mut`, new this pass) — frozen slot names are the constant `slot_title`, not editable (the spec/engine disagreement is unchanged, just no longer blocked on a missing binding) |
| AS-07 | Slot inspector: File / Scale / Tags / Pack metadata | `asset_library_window.gd:704-707` | **done, 2026-08-23** | real | §8 | `as_slot_summary`/`as_item_summary`/`as_pack_info` show real values. **Editing closed 2026-08-23**: new `#[func] as_set_item_transform`/`as_reset_item_transform` (`lib.rs`) write straight into `LibraryItem::transform` (`db.item_mut`); the Scale slider (5..600%, the reference's own `#alScale` bounds) and two Pan X/Pan Y SpinBoxes write live, and Fit/Reset call `as_reset_item_transform` (identity, plus `fit_to_bottom` for a bottom-anchored family) rather than recomputing the reference's `defaultTransform()`/`fitToBottom` arithmetic in GDScript. Pan is two SpinBoxes, not the reference's drag-on-canvas — disclosed in the window's header note as the deliberate substitute for a headless-unfriendly drag surface |
| AS-08 | Per-slot fill state + thumbnails (grid is always a checkerboard) | `asset_library_window.gd:579, 690` | **done, 2026-08-20** | real | §8 | `as_family_slots`/`as_thumbnail_png` — every filled slot shows a real `render_item`-baked thumbnail; empty slots still show the honest checkerboard |
| AS-09 | Sprite-sheet **Slice** | `asset_library_window.gd` slicer modal | **done, 2026-08-20** | real | §8's slicer modal | `cartalith-assets::slicer` is a golden-verified port of the reference's `SpriteSheetImporter` (`computeCells`/`cropCell`/`applyChroma`/`isBlank`, HTML lines 27465-27870); `as_load_sheet`/`as_slice_preview`/`as_slice_apply` expose it. The `N cells detected · M non-empty` readout is the engine's **real** detection pass (the approximate 8×8 GDScript sample it replaced is gone), and the grid overlay draws engine-computed cell spans — the exact rectangles the slice cuts. Non-destructive: the sheet stays loaded for a re-slice |
| AS-10 | Slicer: Trim transparent edges / Skip empty cells | `asset_library_window.gd` slicer modal | **done, 2026-08-20**, with one disclosure | real | §8 | *Skip empty cells* is a straight port (`isBlank`, alpha **> 8**, golden-pinned on both sides of the boundary). *Trim transparent edges* is a **port-side addition, not a port** — the reference slicer has no trim; its second pixel toggle is `background → transparent` chroma keying, now wired here too. Trim reuses the alpha>8 threshold so it can never disagree with `isBlank` (`slicer.rs` module docs; `CHANGELOG.md` discloses it per `CLAUDE.md`'s no-silent-deviation rule) |
| AS-11 | Slicer: Assign to family / Fill from | `asset_library_window.gd` slicer modal | **done, 2026-08-20** | real | §8 | All four targets are offered. §8's *Assign to family* + *Fill from first-empty/overwrite* is framing the **reference expresses as a flat target-slot dropdown**, so the family target is composed from the reference's own primitives (one cell per slot, in frozen vocabulary order); the reference's own three targets (focused slot, one new custom icon, separate custom icons per cell) are ported exactly, including `store[uid]=[item]`'s replace-and-stop for a single-image family |
| AS-12 | Family rail: **Collections** and **Unassigned imports** | *absent* | **done, 2026-08-23** | real | §8's rail lists both | A real **Collections** rail section (`_build_family_rail`/`_refresh_collections_rail`) lists every `as_collections()` entry with a live member count, selectable into a collection-scoped grid (`_select_collection`/`_refresh_grid_collection`, resolving each member uid through `as_slot_summary`). Drag-and-drop: selected slot tiles dragged onto a Collections row join it (`SlotCell._get_drag_data` / `CollectionRow._can_drop_data`/`_drop_data`, calling the same `as_batch_collect` as Collect…). New `#[func] as_collections` (`lib.rs`) is the read side `as_batch_collect`/`as_slot_summary` never had. **"Unassigned imports"**: the engine has no slot-less item concept (`AssetDB` requires a uid on every item), so this is a reserved custom-slot `set` (`UNASSIGNED_SET = "Unassigned imports"`): a pinned rail row (`_build_unassigned_row`, count folded into `_refresh_rail_counts`/the status line's `N unassigned`) browsing every custom slot in that set (`_refresh_grid_unassigned`, filtering `as_family_slots("custom")`'s new `set` field). The footer's Import image…, previously disabled with nothing focused, now lands the file in a fresh slot there via `as_add_custom_slot`. Honest limit: no engine primitive *moves* an already-assigned item into this bucket, so it is reachable from imports only — disclosed in the window's header note. |
| AS-13 | **`Assets ▸ Asset pack ▸` submenu** (24 controls) | *absent — omission O2* | **done, 2026-08-20** | real | §2.3.1 in full | `menus.gd::_build_asset_pack_submenu` — Active pack (live name/author/license/schema/filled-item stats), Pack metadata…, Build ▸ (Validate/Apply to map/Import pack/Export pack, all direct engine calls), Edit ▸ and Batch ▸ (both open the real window, since their controls need slot/selection context only the grid provides). The one still-gap item (Slot transform editing) is disabled with its real reason, matching AS-07's note |
| AS-14 | Variants strip / "active variant" | *absent* | none | — | §8 | **(D)** — engine truth: variant choice at render time is weighted and seeded (`pick_weighted_variant`); a user-picked "active variant" has no counterpart. `DCC_CONTROL_INDEX.md` §3(f). |
| AS-15 | Per-slot Anchor (top/centre/base) | *absent* | none | — | §8 | **(D)** — engine truth: `Anchor` is a **family** property `sprite_draw_rect` depends on, not per-slot. §3(f). |
| AS-16 | 24-family rail vs the shipped 8 | `asset_library_window.gd:8-23, 360` | disclosed in the window's own note and header comment | yes | §8 says 24; mockup shows 11; engine has 8 | **(D)** — owner decision, `DCC_CONTROL_INDEX.md` summary §5 item 9 |
| AS-17 | Slicer: canvas interaction | `asset_library_window.gd` slicer modal | **done, 2026-08-23** | real | §8 | `SheetPreview` has wheel-zoom (centred on the cursor, reversible), middle-drag pan, click-to-select-a-cell (clicking again deselects), and a draggable handle on the grid's **Margin** boundary. **Per-interior-line dragging and cell-scoped slicing**: `cartalith_assets::SliceGrid` gained `col_lines`/`row_lines` (`with_lines`, `Option<Vec<f64>>`, `cols+1`/`rows+1` fractions) which `compute_cells` reads instead of uniform `i/cols` — golden-verified default unchanged (`resolve_lines` falls back to `uniform_lines` for `None` or a length mismatch) — plus `move_line` (clamps a dragged line strictly between its neighbours) and `CellGrid::col_line_px`/`row_line_px` (the undisplaced line positions a handle hit-tests against, distinct from `column_spans()`'s gutter-narrowed cell edges). `as_slicer_move_line` and `as_uniform_lines` expose them so GDScript never reimplements the no-crossing rule; `SheetPreview` draws a draggable dot per interior line, hit-tested in `_find_line`. Cell-scoped slicing: `SliceParams`/`as_slice_apply` gained `only_cell` — `apply_slice` filters `slice_sheet`'s output to that cell before naming/placing, and `slice_btn.text` reads "Slice this cell"; `as_slice_preview`'s "N detected" still describes the whole grid. 30 `asset_bridge` tests plus 22 `slicer` unit tests; verified non-headlessly with synthesised events (`Input`-level for the higher-level flows, direct `_gui_input()` dispatch for the drag/click — the established fallback where OS-level routing into a script-driven check is unreliable): a dragged column line visibly moved the cut off the sprite's colour boundary, and a selected cell changed both the Slice button's text and the `only_cell` sent to the engine. |

### 6.4 Data menu + Data manager window — `menus.gd`, `data_manager_window.gd`

All thirteen `"kind": "gap"` routes, plus the window's own foot and route pane.

| # | Route / control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| DM-01 | Import ▸ Heightmaps (PNG) | 52 | **done, 2026-08-20** | real | §2.4 names it | now a `"live"` route: `DccApp.open_heightmap_import()` → `EngineBridge.import_heightmap` → `WorldGen::import_heightmap`, which decodes the PNG (`cartalith-assets::raster::decode_png`), resamples it at the *image's* aspect ratio and runs `cartalith_engine::import::infer_tectonics` under it — MS-02's other half, same pass |
| DM-01b | Import ▸ Maps (tiles) **and** Import ▸ GIS / GeoJSON — two rail rows since 2026-08-20, as the canvas has them (they were one concatenated row) | 53 | no tile-map or GeoJSON **import** path exists; TIFF absent | yes | §2.4 | (B) large — the remainder of DM-01 after the heightmap half landed. **TIFF is a closed question, not a pending dependency decision**: the reference's own file input is `accept="image/*"` decoded by the browser, which does not read TIFF either, so PNG-only is parity |
| DM-02 | Export ▸ Maps (image · tiles) | 51 | **half done, 2026-08-20** — tile export is real; the *pyramid* is not | partly | §9's route pane, **the one fully-designed route in the window** | **The route is live.** §9's full pane is built (§14.7) and calls `region_export_tiles` over the live Region-select marquee, writing a zipped `cols × rows` grid — verified end to end: 33 entries, `tiles/index.json` present. What remains is the *slippy-map* half the canvas draws and the engine has no notion of: XYZ/TMS/WMTS addressing, a zoom ladder, retina @2x variants, ocean-tile skipping, `leaflet-preview.html`/`style.json` — drawn in the pane and disabled with that reason. Still (B), now medium rather than large |
| DM-03 | Export ▸ GIS / GeoJSON | 53 | was: "no route in, no CRS" | **CLOSED (2026-08-24)** | §2.4 | **DONE** — one `#[func]` (`geojson_bridge.rs`, `WorldGen::export_geojson`) assembling a `GeoJsonWorld` off `CivData` + `WorldState`, exactly as estimated; a `live` route exporting the **whole world**, not the marquee. Three reference inputs this port lacks are handled by omission and disclosed in the pane (no `poi` layer; `sea` derived from which collection a way came from; rivers re-traced from `WorldState`'s receiver tree rather than a `_riverNet` cache). CRS is still not a thing — the document's own `note` says so, verbatim from the reference. Verified: 305,646 B, 511 features (239 settlement / 43 way / 216 river / 6 territory / 7 province), valid JSON, every coordinate inside the world's 1200 × 900 km box. Full account in §20 |
| DM-04 | Export ▸ World Data | 55 | ~~no save writer~~ → ~~no route~~ | **live (2026-08-24), partly** | §2.4 | **The route is real.** Its stated reason — *"cartalith-io reads .zip saves but does not write them"* — had been untrue since FI-01 (2026-08-23). The pane offers the two capabilities `PARITY_AUDIT.md` §5 item 14 names and the reference puts in its header bar: the **export raster** at 2K/4K/8K with `bakeTiles` (`WorldGen::export_raster_png` → `render::bake_rect`, single `.png` or a `tile_{r}_{c}.png` grid plus `index.json`) and the **channel atlas** (`WorldGen::export_channel_atlas` → `cartalith_engine::channel_atlas`). A live `export_raster_estimate` readout shows the real `bakeDims` output size and peak memory before an 8K run. **Still open, unchanged:** this route writes *loose files*, and whether it should also assemble `exportZip`'s single `.zip` (params + f32 layers + raster + atlas + features) or defer to File ▸ Save is not a decision to make from inside an export task. **`layersPreviewChk` real as of 2026-08-24** (the last of the four header-bar controls): `WorldGen::export_layer_previews` writes the reference's four PNGs into a `layers/` folder — `layers/biome.png`, `hillshade.png`, `temperature.png`, `rainfall.png` — beside the raster run, each from the pass the reference's `layerBytes(mode, debug)` branch would take (`bake_rect` for biome; the new `render::hillshade_raster` for `renderNow`'s `mode === 'shade'`; the `temp`/`rain` debug rasters as whole-image palette replacements, because the reference's `debugOpacity` defaults to `1`). At the **grid's** own size, generated worlds only. The `.f32` blobs still belong to the archive decision |
| DM-05 | Export ▸ Assets (pack .zip) | 57 | **done, 2026-08-20** | real | §2.4 | now a `"route"` (was `"gap"`) into the Asset library window's real `export_pack_now()`, the same "routes, doesn't reimplement" shape as `import_assets` — same as AS-04 |
| DM-06 | Sources ▸ External / Connected / Registry | 59-61 | no source registry exists | yes | §2.4 names three rows; **§9 designs no pane for any of them** | **(C)** → §7.3 |
| DM-07 | Conversion ▸ Coordinate Systems (EPSG ▸) | 62 | ~~no CRS conversion~~ | — | ~~§2.4 names it~~ | **RESOLVED BY DELETION** (owner, 2026-08-20) — §7.4's research accepted in full. The route is gone from `menus.gd::_data()` and from `data_manager_window.gd`'s `ROUTES`/`GROUP_ORDER`; the Data manager now has **four** groups. |
| DM-08 | Conversion ▸ Format Conversion | 64 | ~~no format-conversion routes~~ | — | ~~undefined even in the spec~~ | **RESOLVED BY DELETION** (owner, 2026-08-20) — same decision, same commit. |
| DM-09 | Conversion ▸ Data Transformation | 65 | ~~no data-transformation routes~~ | — | ~~undefined in the spec~~ | **RESOLVED BY DELETION** (owner, 2026-08-20) — same decision, same commit; §7.4 had recommended dropping this row outright. |
| DM-10 | Validation ▸ Check Data | 66 | `load_save()` returns pass/fail only; no warning collection anywhere | yes | §2.4 names it ("shows current warning count"); what is validated, and against what invariant, is undefined | **(C)** → §7.5 |
| DM-11 | Validation ▸ Repair / Normalize | 68 | no validation pass to repair against | yes | undefined | **(C)** → §7.5 |
| DM-12 | Foot: "last run (`14:02 · 62 MB`)" | 160 | **partly done, 2026-08-20** — real, session-scoped | partly | §9 | The rail foot and the RECENT RUNS column report the real runs of *this session* (stamp, label, measured bytes, ✓/✕) and say plainly that nothing persists across a launch. (B) small — a persisted history is a `DccSettings` section nothing writes yet |
| DM-13 | §9's route pane: TILES / PROJECTION / LAYERS INCLUDED / OUTPUT / ESTIMATE / RECENT RUNS | **done, 2026-08-20** | real | n/a | §9, designed in full | Built for Export ▸ Maps, the one route §9 designs a pane for: the canvas's two-column grid, all seven column blocks, the `120px label · control` row grammar, the segment/well/`☑` vocabulary, the bordered ESTIMATE block and the `Save as preset · Dry run · Export N tiles` footer. Controls with no engine behind them are drawn in place and disabled with their reason; every other route keeps a one-column pane in the same grammar. See §14.7 |
| DM-14 | §9's **MARKDOWN VAULT · LINKED** block | *drawn, quiet, disabled* | — | — | §9 designs it; `MARKDOWN_VAULT_INTEGRATION.md` is explicitly *"Not started; no code exists"* and its §33 lists two-way sync as a V1 **non-goal** | **(D)** — owner decisions 3 and 4, `DCC_CONTROL_INDEX.md` summary §5. Since 2026-08-20 the block is drawn in the canvas's shape but bordered **quiet rather than accent** and reading `MARKDOWN VAULT · NOT LINKED` / `○ no vault linked · 0 notes`: the canvas asserts a live link, and drawing that would be fiction. All six controls disabled with the reason. *Stale, re-checked 2026-09-24:* the block now reads the live vault (`data_manager_window.gd`, `VAULT_NOTE`) and its header says `LINKED` or `NOT LINKED` by whether a vault is bound. What is still DM-14 is the export half only: `obsidian://` links in exported tiles, note links in exported GeoJSON, and two-way sync, which is a V1 non-goal |
| DM-15 | **`Data ▸ ⧉ Travel library… ⇧L`** | **done, 2026-08-19** | real | — | §2.4's addition + `TRAVEL_LIBRARY_SPEC.md` in full (fields, validation states, placement, §6 build-status) | **Done.** `lib.rs`'s `WorldGen` carries a live `travel_library` field (persists across a re-generate, like `asset_pack`) and a full `tl_*` `#[func]` CRUD+query surface; `jp_compute` builds a `JpAnimalResolver` from it and calls `jp_plan_ex` unconditionally *(2026-09-24: it now calls `jp_plan_full`, with a vessel resolver too; `jp_plan_ex` survives only as `travel_bridge.rs`'s baseline)* (a stock-only library is regression-tested identical to the old `jp_plan` call). `travel_library_window.gd` is the real `2a`/`2b` window, wired at `⇧L`. `TRAVEL_LIBRARY_SPEC.md` §6 has the full record and the two things then still not wired (the planner's party-form dropdown offering a custom entry; only the four built-in species could affect a computed plan). Since then, `c1e0a2a` (2026-09-24) added `Import definitions .csv…`: `travel_library_window.gd::import_csv`, probe `_tlcsv_probe.gd`. |

### 6.5 Preferences menu — `menus.gd`

| # | UI label | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| PR-01 | Devices | **done, 2026-08-20** | real | — | §2.5's per-device checklist, minus its "live utilisation" | **Done, with one disclosed impossibility.** New `cartalith-gpu/src/multi.rs`: `enumerate_devices()` folds `wgpu`'s per-*adapter* rows into one entry per *physical* GPU (this machine returns **six adapters for three devices**, and the OpenGL row reports `vendor = device = 0`, so grouping keys on PCI identity with a name-matching fallback — both unit-tested; keying on name alone would merge two identical cards). `Preferences ▸ Devices` is a live checklist; unchecking everything returns to automatic. Selecting a device really opens *that* device, asserted by `every_enumerated_device_can_be_selected_and_opened`. **§2.5's `71%` utilisation is not implementable and is not faked**: `wgpu` 30 has no system-wide utilisation query and no VRAM size on any backend. The footer shows this app's own allocation total from `Device::generate_allocator_report()`, measured at the last GPU generation and labelled as ours. See `HARDWARE_ACCELERATION.md`'s 2026-08-20 section. |
| PR-02 | Multi-GPU mode | **done, 2026-08-20** | real | — | §2.5 | **Owner decision 2 answered: build it.** `single device` and `split tiles` are real; **`alternate frames` is refused** (`gpu_set_multi_mode` returns `false`) — §2.5's own note is that it only helps the 3D viewport, and there is none. `split tiles` partitions **the domain-warp stage only**, the one GPU stage whose kernel reads nothing outside its own cell; blur needs a halo, JFA and flow accumulation read the whole grid (audit in `warp_grid_gpu_split`'s doc comment). Measured (RX 7800 XT + integrated Radeon): **1.22-1.54x at 4096², but 0.73-0.81x at 2048² and below** — the second device's ~1.8 ms fixed cost exceeds a sixth of a small dispatch. Band sizes come from measured per-device throughput (integrated = 0.17 of discrete). The default ships as `single device` rather than §2.5's `split tiles`, with those numbers in the tooltip. |
| PR-03 | CPU worker threads | 316 | Rayon sizes its own pool; no override exposed | yes | §2.5 | (B) wrapper — one `ThreadPoolBuilder` call at startup; the *default* (cores − 4) is owner policy |
| PR-04 | VRAM budget | **done, 2026-08-20** | real | — | §2.5 | **Done as a cap; its stated default is impossible.** `Preferences ▸ VRAM budget` sets a GB cap gating the GPU path per grid, against a documented upper bound (`gpu_working_set_bytes` — ten `f32` grids, derived from the storage-buffer count the heaviest stage binds plus staging buffers). **§2.5's "default 75 % of the smallest active device" cannot be computed**: `wgpu` 30 reports no VRAM size, and `Adapter::limits()` is an API limit — this machine reports the same 2 GB `max_buffer_size` for a 16 GB card and a shared-memory iGPU. The default is therefore *no cap*, with that reason in the row. |
| PR-05 | Fallback when VRAM full | **done, 2026-08-20** | real | — | §2.5 | **Two of three real, the third refused.** `CPU tile pass` (default) is what the engine already does whenever the GPU path is unavailable — wiring it discloses existing behaviour. `Fail with error` lives in `EngineBridge.generate()`, because `generate_terrain` returns a world rather than a `Result`. **`Reduce working res` is refused** (`gpu_set_vram_fallback` returns `false`): nothing computes a stage at a reduced grid and resamples back up — LOD tile synthesis resamples a finished field, a different operation. Closes omission **O3**. **Update 2026-08-24 — the default was only *usually* graceful**: a GPU that accepted work and failed mid-run was a **panic**, not a fallback. Two faults, one behind the other: ten `.expect`-on-readback sites in `cartalith-gpu` (every dispatch now returns `Option`, every call site `map` → `and_then`); then a device that loses a `map_async` is *gone*, so the next stage died on a 32-byte uniform buffer. A readback failure now marks the device lost and records the size against the adapter, which `device_supports_grid` reads, so `generate_terrain`'s filter steers the next generation away. Measured: the 8192²-on-the-integrated-GPU run that killed the process now completes in 81.9 s on the CPU fallback. See `PERFORMANCE_BENCHMARKS.md` §9.2. |
| PR-06 | Anti-aliasing · anisotropy | 333 | the 2D map path doesn't sample-antialias; belongs to the 3D viewport | yes | §2.5 | (B) large — gated on the 3D viewport |
| PR-07 | Colour management | 334 | the renderer is sRGB-only | yes | §2.5 gives **three values and nothing else** | **(C)** → §7.6 |
| PR-08 | 3D viewport defaults | 335 | no 3D viewport | yes | §2.5 names four fields | (B) large — `DECISIONS.md` §4 defers 3D; `ROADMAP.md` Phase 3 |
| PR-09 | Lighting rig defaults | 336 | no lighting rig yet | **stale in flavour**: there is no *rig*, but all six fields are real and drive the current render (`TerrainAppearance::{sun_az_deg, sun_alt_deg, relief_ambient, relief_gain, relief_lights, relief_directionality}`) | §2.5 | **CLOSED 2026-08-24** with CA-01 — all six are live rows in CARTO ▸ Map view / Rendering-advanced ▸ Relief & light, in the map dock rather than Preferences, where `DCC_SHELL_SCOPE.md` §8.6 already resolved terrain appearance to belong; Preferences keeps the *tier* those values start from |
| PR-10 | Tiled LOD · tile size · atlas cache | 338 | **corrected — S4** | yes, now | §2.5 gives four rows of values | **Half closed 2026-08-24.** §7.7 said to ship the atlas-cache row *"only when tiles are actually written to disk"*; they are now (WW-01). The cache is real, per-world, keyed by a hash of the generation parameters, rooted at `DccSettings`' `atlas_cache` path as §7.7 item 3 asked, with a live readout (SH-07) and a Clear (PR-12). **Still open:** §7.7's *size cap in GB*, and its item 1 split — interactive-LOD toggles into the Layers popover, tile size / LOD levels into Export ▸ Maps. The engine has `atlas_set_tile_size()` and `bake_visible()`; nothing in Preferences calls either |
| PR-11 | Memory ▸ Undo history | 339 | no undo stack | **CLOSED 2026-08-23** — a live submenu, and the one place the stack's real cost is visible: the parent-row tooltip gives depth, bytes held, budget, and what one step costs *at this resolution*; the five budget rows each say how many steps that buys; a `Clear undo history now` row frees it on demand | §2.5 gives a range and a default | ~~(C)~~ → **done, with one deliberate departure from §2.5**: the control is a **byte budget**, not a step count. One height field is 16 MB at 2048² and 256 MB at 8192², so a flat "5 deep" would commit 1.25 GB on the largest world this shell offers. The step count (5, the reference's `MAX_UNDO`) remains the ceiling; the budget binds on a big world. Measured: 80 MB held at 2048², freed exactly on clear |
| PR-12 | Memory ▸ Clear caches… | 348 | no atlas or field cache exists to clear | **stale 2026-08-24** — the persistent tile atlas is a real cache now | §2.5 | **DONE 2026-08-24** — a live row (`ID_PREF_CLEAR_CACHES`), reporting how many chunks went and what they occupied. Un-finalizes as it clears: a lock protecting nothing would strand the world read-only |
| PR-13 | Theme ▸ Light | 362 | **done 2026-08-19** — `DccTheme.apply_theme()`/`remap()` + `DccShell.rebuild_theme()` walk the tree and repaint every token-derived colour in place; Light is a live radio choice | yes | §2.5 + §11's full light token column | **(A)** — a rebuild pass in `DccTheme`/`DccShell`, no engine at all |
| PR-14 | Theme ▸ follow system | **done 2026-08-19** — a third radio item, `DisplayServer.is_dark_mode()` resolved once | none | — | §2.5 | **(A)** — Godot exposes the OS preference; the rebuild pass is PR-13's |
| PR-15 | Units (km · mi) | 368 | the shell is km-only; the reference's mi toggle is not ported | yes | §2.5 gives two values, **and §5.1 stage 02 gives the same control a second home** — an unresolved ownership collision (`DCC_CONTROL_INDEX.md` §3(j), owner decision 15) | **(C)** → §7.8 |
| PR-16 | Keyboard shortcuts… | 369 | no shortcut table yet | yes | §2.5 says *"Editable table, per-context"* and nothing more | **(C)** → §7.9 |

### 6.6 Window menu — `menus.gd`

| # | UI label | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| WI-01 | Save layout as… | 407 | no layout store yet | yes | §2.6 names it | **(C)** → §7.10 |
| WI-02 | The workspace list | **done 2026-08-19** — Window ▸ Workspace submenu over `DccShell.DOMAINS`, via a new public `select_domain()` | none | — | §2.6 | **(A)** — `_select_domain()` and `DOMAINS` already exist |
| WI-03 | Open windows listed while open | **done 2026-08-19** — Window ▸ Open windows, rebuilt every `about_to_popup`; the count is five, not four (`new_world_dialog` had joined the other four) | none | — | §2.6 | **(A)** — four windows exist and all are `AcceptDialog`s on `DccApp` |
| WI-04 | Dock width dragging (§1: "user-draggable within min/max") | **done 2026-08-19** — a real 6 px grip per dock, clamped to §1's min/max | none | — | §1's geometry table gives min/max for both docks | **(A)** — pure GDScript; the collapse chevron already exists |
| WI-05 | **Diagnostics overlay (Shift+D)** — the reference's `#resOverlay` | *absent, and undisclosed anywhere* — `PARITY_AUDIT.md` §5 item 5 | **done 2026-08-23** — new `Window ▸ Diagnostics overlay` check item, `KEY_MASK_SHIFT \| KEY_D` accelerator | none | reference lines 10182-10229 (no port-side design document named it before the audit) | **(A)** — `resource_overlay.gd`, reading `EngineBridge.grid_size()`/`param_get("use_gpu")`/`gpu_stages_used()`/`quality_tier()` plus three real `WorldParams` flags (`tect.dynamic_lithology`, `climate.currents`, `volc.provinces`). **Not a 1:1 port** — the reference's `resOverlay` is short for "resolution", not "resource" (§5 item 5's description guessed a hover-driven resource-potential readout; it is a static engine/perf HUD refreshed after render). IndexedDB/Worker availability (browser-only) and the `PERF.gen`/`PERF.render` per-stage millisecond breakdown (no Rust-side collector exists in `cartalith-godot`) are dropped rather than invented. |

### 6.7 Help menu — `menus.gd`

| # | UI label | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| HE-01 | Documentation | 413 | no in-app documentation; the repository docs are the reference | yes | §2.7 names it | **(C)** → §7.11 |
| HE-02 | Keyboard shortcuts | 414 | no shortcut table | yes | §2.7 — and it duplicates PR-16 | **(C)** → §7.9 |
| HE-03 | Report an issue | 416 | no issue route wired | yes | §2.7 | **(C)** → §7.11; the *destination* is an owner decision |
| HE-04 | **Generation info…** — the reference's ℹ️ `#genInfoBtn`/`#generationInfoText` | *absent, and undisclosed anywhere* — `PARITY_AUDIT.md` §5 item 6 | **done 2026-08-23** — `Help ▸ Generation info…` opens `gen_info_dialog.gd`: a read-only, selectable `TextEdit` dump plus `Copy to clipboard` (`DisplayServer.clipboard_set`, the pattern `journey_planner_view.gd`'s stage-table export uses) | none | reference lines 9824-9868 | **(A)** — `WorldGen.get_params()` (`cartalith-godot/src/lib.rs`) already returns every generation parameter as a flat dotted-key dict, the reference's own "`JSON.stringify`, not hand-picked, so a future slider needs no update" reasoning. **Narrower than the reference in one place**: `generationInfoText()` leads with a hand-picked summary (temperature range, altitude range, max grade) read off live JS field arrays this port has no `#[func]` returning min/max for — real engine-side work, outside this ticket's presentation-only scope, and not invented. This dialog leads with what *is* free (grid, seed, extent, quality tier, GPU) and lets `get_params()` cover the rest. |

### 6.8 Right dock — `right_dock.gd`

| # | Context ▸ field | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| RD-01 | Settlement ▸ Defensibility | 436 | `explain_settlement()` has no defensibility axis | yes | §6 lists it | (B) small — the reference itself treats it as a UI-only categorical (`PHASE2_SCOPE.md` m18) |
| RD-02 | Settlement ▸ Routes | 440 | `get_roads()`/`get_sea_routes()` are plain polylines with no settlement index | yes | §6 | (B) small — needs a settlement↔way association the engine does not model |
| RD-03 | Settlement ▸ **Economy / Politics / Logistics** buttons | 447-449 | "No per-settlement *x* panel exists yet — see Data ▸ World data tables" | **accurate but obsolete as a design position** | §6 names all three | **(A) — done 2026-08-19**: Economy → `app.open_world_data("Economy")` (new `WorldDataWindow.open(tab)`); Politics → `show_faction()`; Logistics → `app.open_journey_planner()`. Verified live via a scripted headless drive (`STATUS.md`/`CHANGELOG.md`). |
| RD-04 | Settlement ▸ government / agriculture | *absent* | none | — | §6 lists both | **(D)** — confirmed UI-only categorical labels with zero derived computation in the reference (`PHASE2_SCOPE.md` m18); adding them would be inventing data |
| RD-05 | **River** context (7 fields + 3 actions, and no way to reach it) | 571-586 | no hydrological river entity crosses the boundary; no `get_rivers()`, nothing selectable | yes | §6 designs the whole context | (B) large — rivers are a per-cell network, not entities with ids/names/catchments; `DCC_CONTROL_INDEX.md` summary §2 item 14 |
| RD-06 | Faction ▸ Territory | 608 | **corrected — S1** | yes, now | §6, §4.5.3 | **(A) — done 2026-08-19**: reads `civ_faction_territory_stats(faction)` live, same call/format as `civilization_workspace.gd`'s Territory tool-options row. |
| RD-07 | Faction ▸ State religion | 611 | `has_religion` computed internally; `get_provinces()` doesn't carry it and there is no `get_faction_aggregates()` | yes — `get_factions()` carries id/culture/colour/settlement_count/claimed_cells, no religion | §6 | (B) wrapper — `civ_faction_aggregates` is golden-verified and unexposed |
| RD-08 | Faction ▸ Roster | 604 | reads province names only | n/a — works, but ignores `get_factions()`'s richer row | §6 | **(A) — done 2026-08-19**: reads `get_factions()` for Culture, a colour swatch, and Settlements; Provinces (count) kept separately. |
| RD-09 | Region select ▸ Send to Data ▸ Export | 672-674 | **done, 2026-08-20** | real | §4.5.1 + §9 | **Closed.** DM-13's route pane is built, so the button opens the Data manager straight onto Export ▸ Maps with the marquee already read (`DataManagerWindow.open_tile_export()`). Deliberately *not* the cheap path the old note offered (a bare `FileDialog` off the dock button): §4.5.1 says "Send to Data ▸ Export", and the marquee and that route's world-bounds fields are two views of one rect, which only holds if the route is where the export happens. |
| RD-10 | **`Layers` context** | *absent — omission O9* | none | — | §6 designs it (ordered list, visibility dot, opacity bar, blend mode, nested children under Terrain) | (B) large — opacity is cheap (overlays carry alpha); blend mode and reorder need the three overlays to become independently compositable, an architecture change `GUI_FEATURE_PARITY_SCOPE.md` Category 3 already recommended deferring |
| RD-11 | Collapsed right dock's primary readout | — | none | — | §6's last line: *"elevation for Sample, layer dots for Layers, stamp count for the stack"*. `DccShell.set_dock_readout("right", …)` exists and **`right_dock.gd` never calls it** — the left dock's is wired (`world_workspace._push_dock_readout`), the right dock's is not | **(A) — done 2026-08-19**: `_push_dock_readout()` called at the end of `_rebuild()` and live from `on_cursor_sampled`; one real reading per existing context (elevation, settlement name, faction id+culture, route length, chain/region/stamp counts, journey days·km). No "Layers" context exists yet (RD-10). |
| RD-12 | `Brush / Stamp` context | 685-696 | merged into `Stamp stack`, with the reasoning stated in-file | yes | §6 lists both | **(D)** — deliberate: both read the same live state and the eight globals already have live editors in WORLD's dock |
| RD-13 | Stamp stack ▸ finalize-lock note | 731-737 | no finalize/lock state exists in this engine | **stale, closed 2026-08-24** — `FinalizeLock` exists and `sculpt_commit` is gated on it | §6 | **(A) — done 2026-08-24.** `right_dock.gd`'s `_build_sculpt()` calls `bridge.finalize_check("height_edit")` on every rebuild: Commit disables (`commit_btn.disabled = stamps.is_empty() or not lock_msg.is_empty()`) and, once finalized, the engine's own refusal sentence ("This world is finalized: the baked atlas is the authoritative surface, so the heightfield is read-only. Un-finalize first.") is shown in the stack, replacing the placeholder that claimed no lock state existed. Verified live: unfinalized shows no refusal and an enabled Commit; after `bake_all`+`set_finalized(true)`, `finalize_check` returns the sentence, Commit disables, and that exact sentence appears as a Label in the dock. |

### 6.9 Journey planner — `journey_planner_view.gd`

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| JP-01 | Carriage **Auto** pick | 366 | `jpAutoPickTransport` has no Rust port | **stale — it did** | `JOURNEY_PLANNER_SPEC.md` §5 ("in auto, counts are computed (terrain × biome, km-weighted) and read-only") | **CLOSED (2026-08-23)**. The reason was wrong: `cartalith_civ::jp_auto_pick_transport` had existed since milestone 6 with eleven tests; only the *call* was missing. `jp_compute` gained an `auto_carriage` key that runs the picker over the derived route and mutates the plan **before** computing it — the reference's `_jpRunAuto` (line 19614), one call site per refresh so it cannot run twice or fight a promoted transport — and returns `auto` (`jp_auto_transport_dict`: species, count, carts/wagons, `promoted`, `fodder_infeasible`, the mutated plan). `_sync_auto_carriage()` writes the ten carriage keys back into the disabled form and rebuilds it on a Walking → Baggage Train promotion — `_jpSyncAssetInputs` (line 19632) and its `structural=true` rule |
| JP-02 | Party set-up picker + capture | `_preset_controls`/`_apply_preset`/`_capture_preset` | `JP_PRESETS` is JS-only; no `jp_presets()` binding | **CLOSED (2026-08-20)**. A live `set-up` dropdown over `tl_list("preset")` (stock and captured; custom rows tagged `· custom` and ⚠-marked by §4 validation state) plus a `capture party…` action through `tl_capture_preset_from_plan`. Deliberately **not** the reference's `JP_PRESETS`: set-ups are the Travel Library's stored rows, the strictly larger thing. Applying assigns only the keys `jp_default_plan()` owns — `tl_get("preset", id)` returns exactly `PRESET_FIELD_KEYS`, `PartyPreset::apply_to`'s inverse — and leaves per-stage overrides untouched per §3.4 | §5 + `TRAVEL_LIBRARY_SPEC.md` §3.4 | — |
| JP-03 | Re-route for `<mode>`… | 1320 | `jpAutoPickTransport`/`_jpRerouteForMode` have no Rust port | half — see JP-01 | §6's "faster-mode advisories… with a **use here** action" | **CLOSED (2026-08-23)**. `_jpRerouteForMode` (reference line 20391) ported as `cartalith_civ::jp_reroute_for_mode`, bound as `jp_reroute(route_index, transport, force_mode)`: re-paths the committed route's endpoints under the domain the transport implies (`jp_mode_for_route`) or v1.100's `force_mode`, rewriting in place so `route_get`/`jp_compute`'s `route` index still names it. Both reference refusals are verbatim, and an **unreachable** answer is refused rather than drawn as `route_commit`'s straight-line fallback — the reason `DijkstraPath::reachable` exists. Overrides, layovers and trim are cleared on success. **Still open**: §6's per-stage *faster-mode advisory* — `jp_best_land_transport_for_stage` is ported, `jp_compute` does not surface it (JP-11's row) |
| JP-04 | **Cost** group | 1519 | **corrected — S3** | yes | §8 designs it in full (food/fodder · wages · tolls/ferry · animal upkeep · total · per km and per day) | **CLOSED (2026-08-23)**, as cheap as predicted. `jp_plan_cost` adapts a finished `JpJourneyPlan` to `jp_journey_cost`'s inputs — the reference's call site (line 19854), including `totalDays ?? days` (wages and upkeep paid on calendar days, rest days included) and the `if(plan.blocked) return null` gate. `jp_compute` returns it as `cost`; the panel renders carriage / wages / crew / animals & vehicles / tolls / transshipment / total / per-tonne-km / break-even. Priced in **day-wages**, never a currency — `JP_COST_*`'s own unit, which keeps the Diocletian ratios apart from a world's invented money |
| JP-05 | Calculation trace ⧉ | 1553-1555 | no trace window; the `formula` string is deliberately not carried across the boundary | yes | §8 says *"opens in its own window (⧉)"* and nothing about its contents | **CLOSED (2026-08-23)**, as §7.12 proposed: an **inline collapsible group over the selected stage**, not a `⧉` window. §7.12's one wrong assumption — that the factors were all "already in `results[i]`'s `eff` dict or derivable from the stage" — is corrected: re-deriving `t_mod`/`w_w`/`col_mod`/the converged load term in GDScript would duplicate every engine table. So the *structured* chain crosses (`JpTerm { key, detail, factor }`, `jp_calc_land_ex`/`jp_calc_water_ex`'s own variables, assigned not recomputed) while the `formula` **prose** stays out. Invariant asserted on a real multi-stage journey: `∏ factor == daily_km` |
| JP-06 | Save journey | 1325 | no save-writer for journeys or projects | yes | §2 lists it in the tool options bar | **PARTLY CLOSED (2026-08-23) — in-session only, said so on the button.** "save journey" names the selected route plus the whole party form (plan, per-stage overrides, layovers, animal entries, trim) into the Journeys list, reloadable in one click. Cross-session persistence was left to FI-01's `.zip` save-**writer** (then under `ROADMAP.md`'s "Options kept open, not scheduled"); the list is GDScript-owned rather than a `cartalith-civ` registry because with no writer there was nothing for the engine to own — a saved journey is exactly the request `jp_compute` already takes. **Ceiling update (2026-08-23):** FI-01's writer now exists, so that reason is gone. What remains is smaller and different: a channel for GDScript-owned project state to reach `params.json`'s `state` — `save_project` builds it in Rust from the parameter table alone. Not built as part of FI-01: a speculative generic “extras” bag is the wrong shape before a second consumer (MEA-07, the Travel Library) says what it needs. |
| JP-07 | ⇧-drag spine trim | 1323 | `jp_compute` has no request field for trimming | yes | §3: *"⇧ drag trims"* | **CLOSED (2026-08-23)**, small as predicted. `jp_compute` gained `trim` (a `Vector2` of two 0-1 arc-length fractions) cutting the polyline through `cartalith_civ::jp_trim_points` **before** anything reads it, so every stage index, stop key and override belongs to the trimmed route — indistinguishable from drawing the shorter route. Endpoints interpolate on their segment (not vertex-snapped). `_ProfileView` owns the gesture: ⇧-press starts, motion previews a veil, release commits, a ⇧ *click* clears. No reference counterpart (v2.10 has no spine) and none invented: the trimmed polyline goes through the same `jp_plan` |
| JP-08 | Journeys list = committed routes | 226, 250 | no named/persisted journey registry exists engine-side | yes | §3's "journeys list" | **PARTLY CLOSED (2026-08-23)** — same work and limit as JP-06. Named journeys list above the committed routes, each reloading its party form and trim in one click, × to forget. **Session-scoped**: empty again next launch. The same **ceiling update (2026-08-23)** applies — see JP-06: the writer exists, and the remainder is the GDScript-state → `params.json` channel. |
| JP-09 | Vessels ▸ sailing window | 1540 | not part of `jp_water_calc`'s return | yes | §8: *"per water leg: vessel, hold used, sailing window"* | **CLOSED (2026-08-23)**. `JpWaterCalc` carries `sailing_window_h` — `jp_water_window(cat, terrain)`, already a *factor* of the leg's `daily_km`, never surfaced — printed per water leg beside vessel and hold. **Deliberately not done**: the engine's window is a property of the **water type** (a bay worked in daylight, open sea stood through the night at 22 h), `TRAVEL_LIBRARY_SPEC.md` §3.3's `sailing_window` a property of the **hull**; nothing couples them, so the group says which it shows rather than blending them into an invented model |
| JP-10 | Supply ▸ foraging offset | 1515 | folded into food/water totals; `jp_plan` doesn't break it out | yes | §8 lists it as its own figure | (B) small |
| JP-11 | Load ▸ speed penalty | 1500 | folded into each leg's km/day; `jp_plan` returns no separate percentage. *Stale, re-checked 2026-09-24:* each plan leg's `land.trace` (`jp_leg_result_dict` → `jp_land_calc_dict`) carries a `"load"` term, with `jp_load_penalty`'s factor as its `factor`. The planner's Calculation-trace section already reads that array, but `journey_planner_view.gd`'s Load note still says `jp_plan` "does not return one" | yes | §8 | (B) small |
| JP-12 | Supply ▸ per-leg bar with resupply ticks | — | **done 2026-08-19** — `_build_reach_bar()`: one segment per gap between resupply stops (`_stop_fractions()`, the stops strip's chord-length projection), `block` when that leg's km exceeds `resupply_reach.required_km`, a tick at every stop | yes | §8 | **(A)**, closed — `resupply_reach` carries `max_gap_km`/`required_km`/`stops`/`unmet`; verified on a real route (9 `ColorRect` children: 4 lit segments + 5 ticks, 2 legitimately zero-width where a stop coincided with the start) |
| JP-13 | **Timeline band** (one band per day) | — | **done 2026-08-19** — `_rebuild_timeline_band()` fills `app.timeline_row` with a `_draw()` day-band strip (`_TimelineBandView`) while JOURNEY is armed, cleared on disarm | yes | `JOURNEY_PLANNER_SPEC.md` §2 | **(A)**, closed — `results[i].days` segments (land `accent`, water `water`) plus one trailing `rest_days+layover_days` block (`text_dim`); "weather hold" stays in the legend, never lit (`jp_plan` has no discrete weather-hold count) — verified on a real 21.47-day plan, 15 segments summing exactly to `total_days` |
| JP-14 | **Blocked-stage inline resolutions** | — | **done 2026-08-19** — `_blocked_resolution_row()` in the verdict card and the stage inspector's `BLOCKED:` box | yes | §9: *"offers its resolutions inline (turn off closures, re-route land-only, depart earlier)"* | **(A)**, closed — turn off seasonal closures (when `blocked_seasonal`), force Walking land-only (transport flip + zeroing carts/wagons — the wheel-block reads cart/wagon count, not `transport`), depart a season earlier; the real Dijkstra re-route stays out of scope (JP-01/JP-03) — verified on two real blocked scenarios (an overload the buttons honestly cannot fix, and an unreachable-otherwise case) |
| JP-15 | Auto fields showing `auto · <resolved>` | — | **done 2026-08-19** — `_auto_label()`/`_party_auto_resolved()`, refreshed post-compute via `_refresh_auto_labels()` without a full rebuild | yes | §5: *"Auto-valued fields show `auto · <resolved value>` so the resolved value is never hidden"* — already done for stage overrides (`_inherit_label`), **now also** the party form | **(A)**, closed — `route_cond`/`infra`/`rest_cadence`/`mount_animal`/`desert_water` resolve from the first applicable stage/leg; `weather_override` stays plain `Auto` (a continuous blend, not one value) — verified against real strings (`"Auto · Dense Oasis Route"` etc.) |

### 6.10 WORLD workspace — `world_workspace.gd`

> **Re-parented by menu structure v3 (§37, 2026-08-24.)** The rows still
> resolve, but the `GENERATION PIPELINE | SCULPT` switch is gone and the ten
> numbered stages are L3 sections inside nine subject categories. §37 adds
> **WW-14** (ecology) and **WW-15** (coordinate system / projection).

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| WW-01 | Finalize · LOD 0–3 · bake & freeze | 290-292 | **corrected — S5** | yes, now | §5.1's dock foot, §4's tool options bar (`app.gd:316-318` carries a second copy) | **DONE 2026-08-24.** The bake/atlas/finalize system is built — `cartalith-spatial/src/pyramid.rs`, `cartalith-terrain`'s `add_zoom_detail`, `cartalith-io/src/atlas.rs` (a filesystem `AtlasStore` where the reference has IndexedDB), `cartalith-engine/src/bake.rs`, `cartalith-godot/src/bake_bridge.rs` behind fourteen `#[func]`s. The dock foot takes **the canvas's three-row split** (Bake depth · Bake ALL levels & finalize · Un-finalize) plus a Clear row. 16 golden-parity tests, all matching first run. Measured: a 2048×1311 world at 1024 px tiles bakes depth 3 in 1.64 s to 85 chunks and 234 MiB; a deep-zoom read is within one `rg16` LSB (7.63e-6) of live synthesis. **Verification pass, same day:** the tool-options bar's copy now presses the same `_on_bake_all` and takes its state *pushed* from `_refresh_finalize()` (its tooltip had kept claiming "No bake/LOD pipeline exists yet"). And a real dead end, found by pressing the button windowed: `_refresh_finalize()` ran only when the workspace was built, before any world, so **"Bake ALL levels & finalize" was permanently disabled**; `app.gd`'s `_refresh_world_dependent()` now fires on `generation_finished` and `world_loaded`. **Still open:** the reference's per-tile Burn-rivers/Micro-erode refinement passes, which `pyramid_tile` documents as deliberately unported |
| WW-02 | Run Droplet hydraulic / Hillslope diffuse / Velocity / Glacial / Coastal (5) | 368-373 | was: "not ported; a separate manual pass in the reference with no `cartalith-engine` equivalent" | **no longer true for four of the five** | §5.1 stage 06 | **DONE for 4/5, 2026-08-23 — §19**, which owns the account. All five kernels ported bit-exact: `droplet_kernel` (Phase 1), and `hillslope_diffuse` / `velocity_erode_kernel` / `glacial_kernel` / `coastal_process` in `cartalith-erosion/src/passes.rs` (26 golden tests, 98 of 115 mutants killed). Four run via `cartalith_engine::ErosionPassParams` at the end of `generate_terrain`, as 21 `params.rs` rows in the `erosion` group (six toggles, fifteen knobs), **every toggle off by default** under `DECISIONS.md` §7d, asserted bit-identical (23 rows and a seventh toggle from 2026-08-24, tidal flats — §19.5). Verified windowed: each pass alone moves 38 %/91 %/6 %/45 % of pixels; all-off returns the base at 0.0000 %. **Droplet stays open** — kernel only, because its `erodeFinish` tail is a second orchestration. The reference's button idiom is now cheap on top (§19.2 (a)); not built while UI work was on hold |
| WW-03 | Sculpt ▸ **Brush shape** (8 falloff shapes, Import brush…, Operation, Falloff curves, Rotation) | 665-672 | no engine behind it, and **not in the reference either** | yes | `DCC_SHELL_SPEC.md`'s own header **correction #3**: *"New design work, not a port gap"* | **(C)** → §7.13 |
| WW-04 | Sculpt ▸ **Stroke & grid** (Add point / Duplicate / Rotate / Scale / Tilt / Push / Pull / Align) | 665-672 | same | yes | correction #3; `DCC_CONTROL_INDEX.md` §5.2 adds that it rests on a **"control grid" concept that exists nowhere** and cannot be sized until defined | **(C)** → §7.13 |
| WW-05 | Sculpt ▸ **Actions** (Flip X/Y, Rot L/R, Flatten) | 665-672 | same | yes | correction #3 | **(C)** → §7.13 |
| WW-06 | Paint ▸ Hardness / Softness | 860-863 | stored and echoed back but never consumed — painting is a hard disc with no soft falloff | yes | §4.5.2 lists both | (B) small — `paint_bridge.rs`'s own module doc |
| WW-07 | Stage 01 ▸ geoid sea level, tides (moon mass/distance/k₂) | 68 | was: "default-off reference sub-systems with no `cartalith-engine` equivalent" | **half of it is now wrong** | §5.1 stage 01 | **ENGINE HALF CLOSED (2026-08-23)** — `cartalith_climate::geoid` and `::tides` are bit-exact ports (`buildGeoid`/`refreshGeoid`/`geoAt`/`currentGeoidPreview`; `tidalForcing`/`computeTideField`/`buildTideField`/`refreshTides`/`currentTideField`), 13 golden tests, live as debug views DV-06, DV-07. **Left: the parameters** — `PlanetParams` carries no geoid amplitude and no moon roster, so both preview at the reference's defaults (as the reference itself does, both toggles defaulting off). The open question is where a default-off sub-system's enable flag lives. **`#tidalFlatsBtn`'s input side closed 2026-08-24 (MS-05)**: `passes.tidal_flats` builds the tide field from the finished surface and runs `apply_tidal_sedimentation` — the first answer to that question: *the pass toggle is the enable*, since the pass needing the field should pay to build it. The geoid half has no such consumer and stays open. (B) small ×2, now genuinely small |
| WW-08 | Stage 07 ▸ min stream order, lakes as water | 90 | reference **render** filters, not generation parameters — Cartography's work | yes | §5.1 stage 07 | (B) small — and `DCC_CONTROL_INDEX.md` marks "lakes as water" **uncertain** (classification switch or display switch?) |
| WW-09 | Stage 08 ▸ seasons & Köppen | 94 | was: "not ported" | **wrong as of 2026-08-23** | §5.1 stage 08 | **ENGINE HALF CLOSED (2026-08-23)** — `cartalith_climate::koppen` ports `computeTempInto`/`computeSeasons`/`classifyKoppen`/`buildKoppen`/`koppenColor` with the frozen 30-key order and the Peel et al. palette verbatim; 6 golden tests, classifier bit-exact; live as debug view DV-04. **Left: the control** — seasons are *derived*, built lazily when the view is picked (as the reference does), so a "seasons" checkbox would toggle nothing. The real gap is exposing `axialTiltDeg` and `maxRainMm`, which the classifier reads and the shell does not surface. (B) small |
| WW-10 | Stages 09 / 10 have no dials | 96-102 | not parameterised — biome classification runs off finished fields; no soil/ore/fertility dials exist in `cartalith-engine` | yes | §5.1 | **(D)** — engine truth, not a gap. Surfacing the *rasters* is a retention-vs-memory decision `MEMORY_OPTIMIZATION_SCOPE.md` already paid to avoid. |
| WW-11 | Per-stage `Run stage n` / `Run n → 10` / stale dots / `04 / 10` counter | *absent* | the dock's own "Not a generation stage" note and `app.gd:298-306` explain why | yes | §5.1 and §4 both design it | **(D)** — `DCC_SHELL_SPEC.md` header **correction #2**: verified by Playwright against the real reference; the capability exists **nowhere**, not in this engine and not in the app being ported. Disabled buttons for it were rejected as clutter. |
| WW-12 | Paint ▸ **the map never showed a painted cell** | *absent* | `render.rs`'s module doc listed *"the paint-brush biome/terrain override"* as **Excluded**, and `paint_bridge::swatch_color`'s doc said no literal RGB table was ported | **accurate when written, wrong by 2026-08-24** — `CART_BIOME_COLS` and `CART_TERRAIN_COLS` had been in the crate since the debug views, and milestone C had built the producer the note assumed missing | reference `landColorCore` 7897-7901 (Biome/Terrain 0.60 tint) and 7765-7773 (Splat texture override) | **CLOSED (2026-08-24).** The tool worked and was invisible: a commit wrote real cells and `build_color_texture()` never changed a pixel, where the reference's `_paintAt` ends in `render()` and tints on the first dab. `render::land_color` takes a `PaintOverride`; `RenderCtx::with_paint` carries the three committed grids; `build_color_texture` supplies them. Pinned by `tests/paint_blend.rs` (9 tests, mutation-checked: the 0.60 weight and the Biome-then-Terrain order each fail it); `golden_parity_render.rs` unchanged because `PaintOverride::default()` *is* the unpainted state. `swatch_color` returns the same two tables, so overlay and map agree. Verified windowed: draft → opaque discs, Commit → the same blended at 0.60 with relief through, erase → the exact clean pixel. Both `_on_paint_commit` handlers gained the `map_view.texture` refresh + `set_preview_texture(null)` pair `_on_sculpt_commit` already had — without it the opaque overlay hides the blend. |
| WW-13 | Paint ▸ Commit / Discard stay enabled after a commit | 911-914, `tool_bar.gd` 393 | *(none — not previously recorded)* | found 2026-08-24, **FIXED 2026-08-25** (§38) | — | **(A)**, closed — §38 owns the account and the measured state table. Both buttons gated on `paint_painted_counts()["total"]`, committed *plus* pending, so after a commit they stayed live over nothing ("Discard draft" then reading as "remove the paint I can see"). Fixed with `PaintEditor::pending_stamps()` / `paint_draft_count()` (over `PaintEditor`'s three `PassBuffer`s) and a cross-refresh between the dock's pair and the tool bar's chip; deferred from the WW-12 pass only to keep that commit off a fourth file. |

### 6.11 CIVIL workspace — `civilization_workspace.gd`

> **Re-parented by menu structure v3 (§37, 2026-08-24.)** Six categories plus
> INFRA's five became fourteen; Politics split into **Factions** (who the
> polities are), **Territories** (what ground they hold) and **Politics** (change
> over time), and the collapse simulator got its own **Simulation** category.
> §37 adds **CV-21**…**CV-26** and **VA-01**/**VA-02**.

> **Domain merge (2026-08-20, owner instruction: "Infra can be dropped as a
> name and can be absorbed by civil"):** INFRA is no longer a rail domain;
> `civilization_workspace.gd` composes an `InfrastructureWorkspace` into its
> own dock, so §6.12 is reached through CIVIL. `infrastructure_workspace.gd`
> was repositioned, not rewritten, so §6.12's file and line citations still
> resolve. Full disclosure in `DCC_SHELL_SPEC.md`'s correction notice.

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| CV-01 | **POI tool** | 94-101 (comment) | **Re-checked and upheld 2026-08-23** (§18.2): omitted, not built inert: `civ_tools_bridge.rs` says POI *"is not a ported concept"*; no Rust function drops one | yes | §4.5.3 designs it in full (kind · faction · name · snap to way, plus a POI inspector) | (B) small — one `civ_drop_poi` mirroring `civ_drop_settlement`; `cartalith-assets`' `poi` family already carries the 10-slot vocabulary |
| CV-02 | Culture ▸ Profiles | 518-523 | ~~`cartalith-civ` generates culture profiles internally; no `#[func]` exports them~~ | — | §3 lists Culture as one of CIVIL's five subjects | **CLOSED 2026-08-25 (engine), UI open.** The original entry was half wrong: culture was **not** unexported — `civ_culture_vocabulary()` already shipped the seven keys, `get_factions()` reports each faction's `culture`, and `civ_set_faction_field` validates and sets it. Missing was a culture as a *row* and an *addressable thing*: `get_cultures()` (`lib.rs`) now returns `{id, key, name, terrain_affinity, faction_count, factions, settlement_count, population}` — the "(B) wrapper" asked for, non-empty before any `generate()` because `CIV_CULTURES` is compile-time — and `EntityKind::Culture` makes a culture attachable in the Markdown Vault (`MARKDOWN_VAULT_SCOPE.md` milestone 6), addressed by its 0-based `CIV_CULTURES` index. **CLOSED 2026-09-21 (UI, verified) — `shell/culture_profiles_window.gd`**: a three-column window (`DCC_SHELL_SPEC.md` §8's chrome convention) — the 7 `get_cultures()` rows, a detail pane, and a faction roster with per-faction assignment (an `OptionButton` writing through `civ_set_faction_field`). Opened via `app.open_culture_profiles()`, beside Faction roster in `civilization_workspace.gd`. Verified windowed by driving the control and re-reading `get_factions()`. Commit `5f111a7`. |
| CV-03 | Timeline filters (Exist only / Ghost removed / Highlight new) can't touch map pins | 821-827 | ~~`get_settlements()` carries no `tid` even though `NamedSettlement` has one~~ | — | `TIMELINE_SCOPE.md` m6 | **PARTLY CLOSED 2026-08-23** — `get_settlements()` (`lib.rs`) carries `tid`. **Exist only** is wired: `_tl_apply_filters` filters the array handed to `map_overlay.gd`'s `set_civ_data` down to the active year's `civ_year_diff().present` tids, upstream of that file. **Ghost removed / Highlight new** stay disclosed-open: both need per-pin fade/halo drawing in `map_overlay.gd`'s `_draw()`, and "removed" also needs the OLD snapshot's settlement data, which no `#[func]` exposes (`civ_year_diff()` returns tid sets only). |
| CV-04 | Settlement class list lacks **metropolis** | 233-239 (comment) | ~~five real `SettlementKind` tiers~~ | — | ~~§4.5.3 lists six~~ | **CLOSED 2026-08-20** — `_civSelectMetropolises` (reference 24961-24989) ported on the owner's decision. `SettlementKind::Metropolis` with the reference's rank-5 tables; `kind_from_str` accepts it, `get_settlements()` reports it, `map_overlay.gd` draws it at rank 5 / glyph ★, promotion runs inside `compute_civilisation` behind `set_metropolis_enabled` (reference default OFF). Spec and engine now list the same six. |
| CV-05 | Territory ▸ "respect coastlines" | 298-304 (comment) | `civ_territory_paint_at` always pushes an ungated circular dab (`PaintStamp::ungated`); no coastline mask behind it | yes | §4.5.3 | (B) small |
| CV-06 | Settlement ▸ "pick radius" | 236-239 (comment) | `civ_drop_settlement` computes its own pick radius internally and takes no argument | yes | §4.5.3 lists it | **(D)** — engine truth; a slider would be decoration |
| CV-07 | Faction roster add/remove, persistent identity | **CLOSED 2026-08-23**, §18.1 — `civ_roster_bridge::FactionRoster` on `CivData`, `civ_add_faction`/`civ_remove_faction`/`civ_set_faction_field`, and the Faction Roster window behind CIVIL ▸ Politics. The reason below was true when written: `CIV_FACTION_COUNT` now *seeds* a real roster instead of *being* it | none | — | §6's Faction context implies a roster; `design/cartalith-menu-structure.md` §3.11 names "add/remove faction, faction roster `#civOpenFactionsBtn`" | (B) large — new Rust state; `CIV_FACTION_COUNT` is a constant |
| CV-08 | `_civApplyRecovery` / auto-populate's static "Recovery phase" | *absent* | ~~none~~ | — | `design/cartalith-menu-structure.md` §4 names it | **CLOSED 2026-08-20** — ported (reference 24619-24640) on the owner's decision, at the reference's call site (line 25761) behind `set_recovery_phase`, surfaced as a five-entry **Recovery phase** dropdown in `File ▸ New world ▸ Generation` filled from the engine's `_CIV_RECOVERY_NAME` table. Phase Stable is a strict no-op. |
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
| IN-06 | Route ▸ vessel / party reference in the options row | `journey_planner_view.gd` `_vessel_field`/`_mount_field`/`_build_animal_definitions` | the journey planner exported nothing past the crate boundary when written | **CLOSED where it can be, remainder stated in-UI (2026-08-20)**. The Mount picker and the four per-species **animal definition** pickers are library-backed (`tl_list("animal")`, custom rows tagged `· custom`) and reach the engine: `jp_compute`'s `animal_entries` → `TravelLibrary::animal_overrides_selected` → `jp_plan_ex`'s resolver, so a custom entry's capacity/speed/fodder/water and ten-row terrain table re-plan the journey. The **Vessel** picker listed every library vessel but disabled those with no engine counterpart (`jp_ship_stats` was a fixed built-in table — `TRAVEL_LIBRARY_SPEC.md` §6), reason on the item | §4.5.4 | **CLOSED (2026-08-23)** — the remainder is done. `TravelLibrary::vessel_overrides` (keyed by **name**, because `JpPlan::vessel` is a name and `jp_ship_stats` a name lookup, so no `animal_species_slot` equivalent is needed) → `travel_library::vessel_resolver_fn` → `JpVesselResolver` → `jp_calc_water_ex`, the animal chain's sibling with the same fall-back-to-built-in contract. Four of `ShipStats`' seven fields come off the definition; `river`/`sea` from `modes`, `open_sea` from `water_rating == Open` (`jp_vessel_water_block`'s own test). **`invalid_water` has no source**: §3.3 has no per-water-type blacklist, so a custom vessel is constrained by mode and rating only, never by a named water type the way "River Barge cannot navigate River with Rapids" is — stated in the picker's tooltip. The picker enables every vessel that validates `ok` and disables only incomplete ones, because the resolver declines an incomplete definition rather than sailing a zero-hold hull |
| IN-07 | Trade ▸ route assignment | 370-373 | nothing ties a trade relationship to the road or sea lane that would carry it | yes | §3 lists Trade | (B) large |
| IN-08 | **Roads, Ports, Trade and Logistics never rebuilt after a generate** | `_build()` | none — nothing disclosed this | **not a capability gap** | all four were designed *and* built; only the signal was missing | **FIXED 2026-08-24 — see §23 (RF-01)**. Roads had a partial path already (`_refresh_manual_ways`, on a way commit); the other three had none at all |
| IN-09 | **A committed Route-tool route appeared nowhere at all** — no map line, no list row | `_commit_route` (status hint: *"not shown on the map yet (no manual-route display getter…)"*) | the hint's reason was **wrong**: `route_count()`/`route_get(i)` had existed since the Journey Planner milestone and return the whole solved polyline; nothing in GDScript called either | the *disclosure* was accurate (the route really was invisible), the *reason* was not | §4.5.4's Route tool; the reference draws committed journeys as their own pass (`drawCivLayer` block 2b, lines 15552-15560) | **FIXED 2026-08-24 — see the note below**. Found by live verification: the tool committed a 572 km, 506-point path with zero unreachable legs and drew none of it |
| IN-10 | **`Data ▸ Journey planner… ⇧J` did nothing visible from any domain but CIVIL** | `app.gd`'s `open_journey_planner()` | none — nothing disclosed this either | **not a capability gap** | the planner was engine-complete and the takeover painted correctly; only the domain was wrong | **FIXED 2026-08-24 — see §27**. The shell opens on WORLD; the takeover only paints in CIVIL. Two of the three entry points are reachable from anywhere, and both armed a tool and changed not one pixel |
| IN-11 | **Every tool's advertised letter was bound to nothing** — `Way (W)`, `Route (⇧R)`, and eight more across four domains | `dcc_widgets.gd`'s `_tools_row` | none — the tooltip *was* the disclosure, and it was false | **not a capability gap** | every tool works when clicked | **FIXED 2026-08-24 — see §27**. A `Shortcut` per button, parsed from the label the tooltip already shows; `BaseButton::shortcut_input`'s visibility rule gives cross-domain inertness for free |

> **IN-09 CLOSED (2026-08-24).** Found auditing the manual map-authoring
> toolset live (assets · labels · routes · POI · settlements). It is IN-02's
> failure mode one list over — *the engine does the right thing and nothing
> renders it* — and the third of that shape after IN-02 (ways) and WW-12
> (painting), so stated as a rule: **a `#[func]` that returns geometry proves
> nothing about whether anything draws it. Check the pixels.**
>
> IN-02's closing note is why it survived: it reasoned, correctly, that a
> committed route does not belong in `get_roads()`/`get_sea_routes()` (a route
> is a journey *along* geometry, not durable geometry) and stopped there,
> leaving routes in **no** layer at all.
>
> The reference settles it: `civJourneys` gets its own draw pass in
> `drawCivLayer` (block 2b), stroked dark (`rgba(40,25,5,.5)`, width 3) then
> dashed amber (`rgba(200,160,60,.85)`, width 1.5, `setLineDash([5,3])`), and
> its own list with a per-row delete (line 17250) and a "No journeys yet. Draw
> one with the **Route** tool and press Esc to commit." empty state (line
> 17233). So drawing a route is a port, not an addition. `map_overlay.gd`
> carries the pass in `_manual_routes`/`_draw_manual_route_segment` with the
> reference's colours and widths, honouring `brks` as the sea-lane pass does;
> `ViewportHost.manual_routes()` owns the `route_count`/`route_get` loop and
> `refresh()` pushes it, so a regenerate clears the old world's routes.
>
> **The list half closed later the same day (2026-08-24)**, after first being
> registered as a (B) — select, name and delete had no `route_delete`/
> `route_set_name` `#[func]` and no selected-journey branch in `map_overlay.gd`.
> The estimate held: two `#[func]`s and a row builder. `InfraTools::route_delete`
> is `Vec::remove` — the reference's `civJourneys.splice(ji,1)`, not a
> tombstone — and `route_set_name` writes a new `CommittedRoute::name` that
> `route_get` returns. **Indices renumber**, stated in both doc comments and in
> `engine_bridge.gd`'s wrapper because `jp_compute`'s `route` key and
> `jp_reroute`'s `route_index` name routes by index; a tombstone would have kept
> them stable at the price of `route_count()` no longer meaning "how many routes
> there are", which every consumer assumes.
>
> Rows carry the reference journey card's three affordances in its order —
> select glyph · name field · km · `×` (`_civRenderJourneyList`, line 17235) —
> and `map_overlay.gd` gained block 2b's `sel` branch verbatim (underlay width 5
> instead of 3, amber `rgba(255,210,80,.98)` at width 2.5 instead of
> `rgba(200,160,60,.85)` at 1.5; the dash is not selection-dependent in the
> reference and is not made so here). Two disclosed divergences: the **name
> field renames per keystroke** (`oninput`) but does **not** rebuild the row,
> which would steal focus; and **deleting a lower-indexed route decrements the
> selection** — the reference only clears it when the index runs off the end
> (`if(_civSelectedJourneyIdx>=civJourneys.length)`), silently moving it onto a
> *different* journey. No planner summary on the row: the reference computes it
> with `_jpPlan`, which is `journey_planner_view.gd`'s screen here, and two
> places computing a plan would disagree.
>
> **Still open after this:** callers caching a route index across a delete
> are the shell's problem — `journey_planner_view.gd`'s `_route_index` is not
> re-validated when the INFRA dock deletes a route under it (it re-reads
> `route_count()` on open, so the failure is a wrong selection, never a crash).
> And `way_set_name`/`way_delete` remain missing — only *routes* got theirs.

> **IN-02 CLOSED (2026-08-24).** The audit's diagnosis and its "(B) small —
> one getter" estimate held: `get_roads()` and `get_sea_routes()` append
> `InfraTools::ways` to what they already return, tagged `manual: true` (plus
> `km`, which `Way` and `ManualWay` both carried and neither getter emitted).
>
> **No new getter, deliberately.** The estimate assumed a `way_get`/`way_count`
> pair mirroring `route_get`; the reference says not to. `_civCommitWay` (line
> 26077) pushes a hand-drawn way onto the same flat `civWays` array as the
> generated network, tagged `manual:true`, and the draw pass (line ~15494)
> branches on `rt.type` alone, so a hand-drawn `road` and a generated `road` are drawn identically — `manual` exists so a way *survives a network
> rebuild* (`_civAutoRoutes` filters `civWays.filter(w => w.manual)`) and can be
> listed, never to style it apart. A separate getter would split one list into
> two and give every consumer (`map_overlay.gd`, `right_dock.gd`'s Route
> context, the workspace lists) a second code path for no behavioural
> difference. Only the *sea lane* splits into `get_sea_routes()`, the one
> distinction the reference's draw pass makes (`type === 'sea-lane'` takes the
> navy/dashed branch).
>
> Also closed: `_commit_way` repaints the map
> (`CivilizationWorkspace._refresh_civ_data()`, camera-preserving) and refills a
> **Roads ▸ Hand-drawn** list instead of printing "not shown on the map yet";
> the right dock's Route context gains a **Source** field (Hand-drawn /
> Generated) and reports the engine's `km` rather than re-measuring the `f32`
> point array. The row's original "ways/routes" wording was only half a real
> gap: routes belong to the planner's registry, not the way layer
> (`infra_tools_bridge.rs`'s "Way and Route are two separate commit paths, on
> purpose"). **Corrected the same day by IN-09** — that reasoning was sound and
> its conclusion should have been "so routes need a layer of their own", not
> "so they need nothing"; see the IN-09 note above.
>
> Still open, and not folded in: no `way_set_name` / `way_delete` /
> way-condition `#[func]`, so a committed way cannot be renamed, retyped or
> removed (the reference's way-properties editor has no counterpart), and
> §4.5.4's "grade profile / surface" half of the Way inspector is unbacked.
> Separate (B) items, not IN-02.

### 6.13 CARTO workspace — `cartography_workspace.gd`

> **Re-parented by menu structure v3 (§37, 2026-08-24.)** Three categories plus
> RENDER's flat run of sections became ten. §37 adds **CA-16**…**CA-19**.

> **Domain merge (2026-08-20, owner instruction: "And render into carto."):**
> RENDER is no longer a rail domain — `cartography_workspace.gd` composes a
> `RenderWorkspace` into its own dock, so §6.14 is reached through CARTO. This
> also resolves the CA-01/RN-01 question of which domain owns
> `set_appearance()`: they are the same dock now.

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| CA-01 | Layer properties ▸ **LIGHT** (azimuth · elevation · strength · multidirectional) | 110-114 | `TerrainAppearance` is implemented and settable in Rust but bound to no GDExtension method | yes | §7's LIGHT group | **CLOSED 2026-08-24** — `WorldGen::{get_appearance, set_appearance, list_appearance_tunables, reset_appearance}` bind **21 scalars by name**, drawn by `render_workspace.gd` as CARTO ▸ Map view + Rendering-advanced. `DCC_CONTROL_INDEX.md` §3(g)'s "Strength" ambiguity is resolved by exposing **both** `relief_gain` and `relief_directionality`. The same binding closes **PR-09** and the colour/relief half of **RN-01** (see RN-03). |
| CA-02 | Layer properties ▸ **FILL** (colour ramp picker, domain, range) + the **Stop editor** | 110-114 | same note | yes | §7 designs nine named ramps, a popover, and a full stop editor | **CLOSED 2026-08-24** — a renderer change, as the row said, not a binding. `render.rs` gains `ElevationRamp`/`RampStop` and a `ramp_strength` tunable: an ordered breakpoint list keyed to **relative land elevation** (0 = shoreline, 1 = the world's highest point, so a saved ramp means the same picture on a world with a different peak), sampled linearly and blended over the material colour **before the light curve** — a hypsometric tint under shaded relief, not an elevation key pasted on top. Land only; the sea keeps `sea_color_core`'s depth ramp. **Ships off** (`ramp_strength: 0.0`, stage skipped, `js_reference()` untouched). Bound as `list_ramp_presets`/`get_color_ramp`/`set_color_ramp`/`load_ramp_preset`; **add, delete and reorder are all `set_color_ramp`** — the panel sends the list and the engine sorts by position, so dragging a stop past its neighbour *is* the reorder. Panel: CARTO ▸ **Colour relief** — nine named ramps, a live gradient bar, one row per stop (colour · position · metre readout · delete), Add stop, Reverse, strength slider. Verified windowed at 2048×1311: all nine presets render distinct maps (mean \|d\| 21.0–50.7 levels); strength 0 returns the base at **0.0000 %**; through the real dock a slider drag reaches the engine (0.6), Add lands a stop in the widest gap (0.39 between 0.28 and 0.50), a drag from index 7 to 0.02 lands it at index 1 **with its colour**, delete and Reverse re-render. Seven tests. Two of the five owed items landed one commit later — see CA-02a below. **Still owed**: duplicate, an absolute elevation domain, Auto Fit / Auto Breakpoints — stated in the panel's Still-owed block. |
| CA-03 | Terrain sub-layer visibility (Hand-drawn hillshade / Hillshade / Colour relief) | 105-107 | terrain, hillshade and colour relief are one baked raster, so they toggle with the map | yes | §7's ten-row layer stack | (B) large — needs the single colour pass to become separable outputs |
| CA-04 | Layer opacity / blend mode / reorder | *absent* | none in this file (`layers_popover.gd` has a *debug-view* opacity slider, a different thing) | — | §6's Layers context, §7 | (B) — opacity is **wrapper** (overlays already carry alpha); blend/reorder is **large** |
| CA-05 | Icon ▸ on-canvas resize handle | 277-279 | *"no on-canvas resize handle yet… (`icon_bridge.rs`'s own acknowledged gap)"* | was true — see Now | §4.5.5 | **CLOSED (2026-08-24)** — `icon_bridge::icon_handle`/`IconEditor::handles` port `drawCivLayer`'s icon-handle geometry (lines 15883-15893: `hr=max(4,3.2*lsc)`, `hx=px+side/2*0.7`, `hy=py+side/2*0.7`, stored `r=hr*1.6`), transcribed as `label_bridge::handle_circles` was (inline canvas drawing, so `manual.rs` had no home for it). `WorldGen::icon_handles(index, zoom)` returns `{"resize": {"x","y","r"}}`, `label_handles`' shape, so `tool_overlay.gd`'s `set_handles()` needed no change. `cartography_workspace.gd` gained the one piece of state the engine has no reason to hold — `icon_get_selected()` (a new `#[func]`, `label_get_selected`'s counterpart) plus `_on_icon_click`/`_on_icon_drag`/`_on_icon_release`, mirroring `_begin_label_handle_drag` one handle down (`icon_resize` commits the scale directly, unlike `label_resize_size`). Verified: drag the handle, the sprite rescales live and survives a zoom/redraw. **Not folded in**: `icon_hit_test`'s box-hit half is unused — re-selecting a placed, unselected icon by clicking its box has no wiring, a separate gap. |
| CA-06 | Label ▸ letter-spacing, anchor | 643-648 | no backing field on `MapLabel` (`label_bridge.rs`'s own "Not modelled" note) | yes | §4.5.5's tool options row lists both | (B) small |
| CA-07 | Label ▸ font (the stored CSS string doesn't render) | 643-648 | Godot has no web-font fallback chain, so only size/angle/arc/colour render | yes | §4.5.5 says "font role" — **a role, not a CSS string** | **(C)** → §7.14 **Reclassified (C) → (B), 2026-09-07.** §4.5.5's *role* is built — label roles landed 2026-09-06 as a role **template** plus an explicit counted apply (never a silent overwrite, deliberately not a fallback); `halo` and `tracking` were found **fully live** on the way, so a brief to dash them was refused. The remainder, corrected 2026-09-07 (the account is in §3): **family HAS a field** (`MapLabel::font`, `set_font`, `font_or_default()`, a `LabelStyleSnapshot` round-trip — `lib.rs`: *"sending the literal string via `font` already works today"*); **weight and case have none** (`labels.rs:945`'s `weight` is a placement rank). **(B) stands on the RENDERER**, as the Blocked-by cell always said: Godot has no web-font fallback chain for the string the model stores. |
| CA-08 | Style presets (Atlas / Parchment / Physical / Ink) + `custom — edited since preset` + Reset + Save preset | *absent* | none | — | §4's Cartography row | **CLOSED 2026-08-24** — CARTO ▸ Map style carries the **reference's own five** (Default / Antique / Ink / Watercolor / Print, reference HTML 12850's `STYLE_PRESETS`) as absolute bundles, the `Custom — controls edited since the last preset` note, and Reset-to-quality-tier. **Save preset closed the same day**: `TerrainAppearance` (and `Npr`, and `ElevationRamp`) derive `Serialize`/`Deserialize` — §7.15's *"the one Rust line the whole feature depends on"* — and `save_appearance_preset`/`load_appearance_preset`/`peek_appearance_preset` write a named look to its **own small JSON file** (`user://appearance_presets/<slug>.json`), not the world `.zip`: a look is reusable *across* worlds, and `SAVEFILE_COMPAT.md`'s format is the reference app's and shallow-merges `state`, so an invented block would be one more unshimmed key. A loaded preset replaces the **quality tier** as the base layer (a look saved at Ultra renders at Ultra anywhere) and clears the override map, else loading would not reproduce the look. `reset_appearance()` drops all three layers. Panel: CARTO ▸ **Saved looks** (name, Save look, picker, Load look). Verified windowed at 2048×1311: a look saved, the session mangled to **99.999 %** different, the preset loaded back at **0.0000 % moved, worst 0 levels** (including a four-stop ramp); Reset returned the tier's look at 0.0000 %. Struct-level `#[serde(default)]`, so an older preset still loads. Three tests. The design's four names lost to the reference's five, being verifiable. |
| CA-09 | Layer list ▸ search field; footer tabs **Blocks / Verticality** | `layers_popover.gd` | `EngineBridge.debug_layers()` + `CartographyWorkspace.LIVE_LAYERS` | yes (search) | §7 names them | **SEARCH CLOSED 2026-09-06; TABS RULED NOT BUILT** → §7.16. One field over **two lists that stay two lists**: `VISIBLE LAYERS` (the eight `LIVE_LAYERS` toggles, writing through `ViewportHost.set_layer_visible()` so CARTO ▸ Layers follows in the same frame) and `DATA OVERLAYS` (the 43 `debug_layers()` field rasters). Matches label **and** engine id; count reads `3 of 51 layers`; an empty field shows everything; no match says so once. Hotkeys 1-8 are assigned over the **unfiltered** order, so a filter cannot silently rebind them — 8/8 still reach their view with their rows filtered off screen (`_layersearch_probe.gd` §5). Blocks/Verticality: reading **C, both are dead names** — see §7.16 and the note beside the Layers category in `cartography_workspace.gd`. |
| CA-11 | **`hydro_wet_strength` (Wetness) renders nothing at working resolution** | *engine* | — | — | reference `wetnessR` | **CLOSED 2026-08-24 (owner-authorised retune; it moves the shipped look).** Found by measurement: the binding was correct and the *stage* invisible, worse as the grid got finer — both halves of `build_hydro_wetness` had been tuned at a small grid. **(1)** The gate was `smoothstep(0.55, 0.88, …)` over the world's *min-max-normalized* log-flow range, but `flow / (gw*gh)` is already scale-free (the fraction of the map a cell drains), so re-normalizing cost the threshold its meaning: `lo` pinned to the `1e-4` clamp floor, `hi` to the largest basin, the knee at ~0.8 % of map area — the trunk river and nothing else. Replaced by an **absolute** upstream-area gate, `6e-4 … 8e-3`, the same channels at any resolution. **(2)** The blur then diluted what survived: a mean-conserving box blur over radius `r = gw * 0.006` cuts a one-cell line's peak by about `1/(2r+1)`, and `r` grows with the grid (3 cells at 512 wide, 12 at 2048). The blur stays (it softens the halo); a matching clamped **gain of `2r + 1`** restores the peak. Measured on one world, 0 → 1, pixels moved: **1.216 % → 10.785 %** at 512×384, **0.184 % → 4.966 %** at 1024×768, **0.002 % → 2.589 %** at 2048×1311; at the shipped `0.38` default, **0.000 % → 1.422 %** at working resolution (worst per-channel delta 3 → 59 levels). `6e-4/8e-3` was picked by sweeping: `1e-3 … 1.2e-2` left working resolution at 0.67 %, `3e-4 … 5e-3` took it to 3.4 % (a wet-valley wash, not a river corridor). **Trade, stated:** the gate is absolute, so a world whose basins are all below `6e-4` of the map gets no wetness. Verified windowed at 2048×1311 (default → 0 moves 0.821 %, default → 1 moves 1.295 %, corridors read as wet valley floors along real drainage). `hydro_wet_strength` left `every_tunable_is_load_bearing`'s exemption list, and `appearance_ab_dump.rs`'s new `hydro_wetness_visibility_by_resolution` fails if any of the three sizes goes quiet. |
| CA-10 | Layer properties ▸ **Visualization dropdown** | *absent* here; `layers_popover.gd` covers it with 18 debug views | the popover's own footer explains the split | yes | §7 lists it; §10's popover overlaps it | **(D)** — deliberately resolved as one popover rather than two competing pickers (`layers_popover.gd:10-15`) |
| CA-12 | **The whole Icon tool is inert until an asset pack is imported** — and the app ships with none | `lib.rs`'s `icon_arm`: `if !self.has_asset_pack() { return false; }`; `cartography_workspace.gd:298/310/353` mirror the gate | *"arming a family/slot this port cannot yet draw would let a caller stamp icons with nothing to render, silently"* (`icon_arm`'s own doc comment) | **honest disclosure, obsolete reason.** `map_overlay.gd`'s `_draw_manual_icons` draws every family from built-in vector shapes and never reads the pack (its doc: *"No texture atlas from the asset pack is wired into Godot yet… these are honest placeholder glyphs"*); the pack path (`pack::composite_map_icons`) is the **scattered** auto-icon bake, a different feature. No family/slot is undrawable any more | §4.5.5; the reference has **no such gate** — `iconVariantsFor` (line 7304) returns *"pack or built-in glyphs"* and `drawIconGlyph` (7315) is the built-in fallback for exactly this case | **(B) small, but an owner call — not taken here.** Verified live 2026-08-24: on a fresh world `has_asset_pack()` is `false`, `icon_armed()` is `{}`, and clicks with Icon armed place nothing; loading `cartalith-assets/tests/fixtures/reference_pack.zip` makes the same clicks place and draw three icons. The fix is deleting a three-line gate and its doc paragraph, but it reverses a written decision, so it is raised, not done (`CLAUDE.md`: *"Do not deviate from `DECISIONS.md` silently"*) |
| CA-13 | **Region naming looked absent** (owner report) | `_build_label_panel` / `_rebuild_label_panel` | none | **not a capability gap** — it works end to end | the reference calls these *region labels*; the dock said "Placed labels" / "none placed" and no menu mentions labels at all | **FIXED 2026-08-24 — see §27**. Renamed to **Region labels**; the empty state now names the tool that ends it. A menu route is still owed and is a menu-structure change, not a wording fix |

#### CA-02a — the ramp's Ease/Step modes and per-stop alpha (2026-08-24)

**The two axes CA-02 shipped without**, deferred because both were `render.rs`
work rather than a binding. `RampStop` gains an `a`, and `ElevationRamp` a
`RampMode` of `Linear` / `Ease` / `Step`.

- **The mode belongs to the ramp, not a stop** — §7 draws one picker above the
  stop list, and "banded" is a statement about the whole plate. `Ease` is this
  file's `k²(3-2k)`; `Step` tests `k >= 1.0` rather than returning a flat
  `0.0`, so a sample exactly *on* a stop takes its colour and two coincident
  stops still draw the hard edge they draw under `Linear`.
- **Alpha rides the same `k` as the colour** and multiplies into
  `ramp_strength`, so an alpha-0 stop reveals the material model at that
  elevation — how a ramp tints only the summits.
- **Two traps, both taken.** `serde` gets `default = "one"` for alpha, not
  `#[serde(default)]`: a look saved before the field described *opaque* stops,
  and `f64::default()` would load them invisible. And `normalized` always
  returns a `Linear` ramp, so `set_color_ramp` and `load_ramp_preset` carry the
  mode over by hand — else editing one stop silently resets a Step plate.
- Bound as `list_ramp_modes`/`get_ramp_mode`/`set_ramp_mode`, behind
  `EngineBridge.ramp_mode_api` — a **third** feature flag, so an in-between
  binary loses the picker rather than the stop list. Panel: a `Blend` picker
  above the gradient bar, an alpha slider per stop row. The bar shows `Step`
  exactly and `Ease` **approximately** (`Gradient` offers cubic, not
  smoothstep), which the code says.
- Ten tests. **Verified windowed at 2048×1311** on a real world: three modes,
  three distinct maps (Linear↔Step 67.4 % of pixels moved, mean |d| 14.1, worst
  177; Linear↔Ease 41.5 %; Step visibly a banded hypsometric plate); an alpha-0
  ramp at `ramp_strength = 1.0` returns the base at **0.0000 %**;
  `set_ramp_mode` survives `set_color_ramp` *and* `load_ramp_preset`; a bad
  mode name returns `false` and changes nothing; through the real dock an alpha
  drag re-renders (33.4 % moved) and **a later colour edit leaves the alpha at
  0.40** (the `edit_alpha = false` trap); a saved look round-trips both axes at
  **0 moved**, the picker following the reload.

### 6.14 RENDER workspace — `render_workspace.gd` (now composed into CARTO, §6.13)

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| RN-01 | The whole domain — Terrain appearance groups | 14-15 | `render.rs`'s `TerrainAppearance` is real but unbound; until it is, Preferences ▸ Render quality is the only live control | yes | §3 gives RENDER a dock; `design/cartalith-menu-structure.md` §5b designs the full subsystem (Preset · Colour relief · Colour · Material · Relief · Detail · Atmosphere · Preview · Quality) | (B) **wrapper** — ~40 real, tested fields driving the current render, reachable through **no `#[func]` at all**. The single largest cheap surface in the shell. **PARTIALLY CLOSED (2026-08-23), see RN-02**; the colour/relief half **CLOSED 2026-08-24, see RN-03.** |
| RN-03 | The **colour/relief half** — `TerrainAppearance`'s scalar fields | was RN-01's remit | *"bound to no GDExtension method"* — true until this pass | — | §5b's Relief / Detail / Preset groups; the reference's Cartography ▸ Map view + Map style (HTML 1706-1783) | **CLOSED 2026-08-24.** Bound **by name**, not as ~20 `#[func]` pairs: `list_appearance_tunables()` publishes `(key, min, max, label)` for 21 scalars, and `get_appearance`/`set_appearance`/`reset_appearance` follow `set_npr`'s every-key-optional, returns-the-count-applied contract, so the panel builds from the engine's own ranges. The key→field table is one `tunables!` list, held by three tests: round-trip, **no two keys aliasing one field**, **every tunable is load-bearing** (a row that moves no pixel fails). Overrides layer *over* the quality tier, so switching tier keeps the user's sun azimuth. Panel: CARTO ▸ **Map view** (relief exaggeration · sun azimuth · sun elevation · relief↔biome) + **Map style** (the reference's five presets, `Custom` note) + **Rendering — advanced** (Relief & light · The sheet · Materials, Reset to quality tier). Verified windowed: 18 of 21 keys move the raster, all-restored is byte-identical, `Default` reproduces the base at **0.0000 %**, Reset moves engine and sliders. The three that did not move: `relief_lights` (live at 1, 3.81 %, converged past 6); `splat_strength` (correctly inert with no pack); `hydro_wet_strength`, a **real engine defect**, registered as **CA-11**. RN-01's remaining owed items — the colour ramp (CA-02) and saving a look (CA-08) — closed one commit later. |
| RN-02 | The reference's **NPR block** — ten "Painter" styles, coastal wave lines, animated water, multi-sun lighting | was RN-01's remit | this half was not merely unbound but **unported**: `render.rs`'s module doc listed *"the 'Painter' NPR block (watercolor/contours/ink/hachure), multi-sun hillshade"* as Excluded | — | `PARITY_AUDIT.md` §3.1's ~15 missing render paths | **CLOSED (2026-08-23).** Ten styles, wave lines and the multi-sun rig are literal per-pixel ports (`render::apply_npr`/`apply_waves`/`multi_sun_from_normal`/`coast_distance`), golden-verified under Node in `tests/golden_parity_npr.rs` (37 mutants, none survived — four survived a first sweep and were killed by shaping four more fixtures onto the gates they hid behind, never by loosening a tolerance), off at every default. Live through `WorldGen::get_npr`/`set_npr` in `render_workspace.gd` ▸ **Painter styles** / **Water & light**. Animated water is per-frame, so it is a Godot `ShaderMaterial` overlay (`water_anim_layer.gd` + `water_anim.gdshader`) over `sample_bridge.rs`'s new `waterfx` field — principled equivalence (`DECISIONS.md` §7a), not golden. The reference's `GW*GH <= 400000` animation cap is deliberately **not** ported: it protects a JavaScript pixel loop that no longer exists. Verified windowed on the real GPU (per-style PNG and movement, all-off byte-identical, frame-to-frame non-zero only with water on, a slider drag reproducing the engine call's raster exactly) — which found three bugs no test could: `npr_api` guarding on a never-written method, so the panel silently did not build; `Npr::peak_m` never filled from `params.peak_m`; `waterfx` intensity selecting six cells of a 512×384 world, now keyed to `river_flow_thresh`. The *colour/relief* half, which `set_npr` does not touch, is RN-03. |

### 6.15 Frame, viewport and phone — `dcc_shell.gd`, `viewport_host.gd`, `layers_popover.gd`

| # | Control | Line | Disclosed reason | Accurate? | Design | Class |
|---|---|---|---|---|---|---|
| SH-01 | Rail expansion `›` → 200 px sub-node list | **built 2026-08-19, WITHDRAWN 2026-08-24 (§28)** — the canvas draws the rail at `width:40px` in all eight desktop artboards and never draws an expanded one; the owner reported the toggle as a defect. The `›` is kept as the chrome the canvas specifies, not an affordance | none | — | §3 names it; the canvas contradicts it | **(C)** — `DCC_CONTROL_INDEX.md`: *"Sub-node lists per domain are not enumerated in the spec; the builder has no source for them"* → §7.17, §28 |
| SH-02 | Phone: tool-sheet drag, gesture-inset handle | 1056-1058, 1099-1101 | *"the mockup pictures exactly one static sheet state; nothing here answers a drag gesture"* | yes | §13 | **(D)** — deliberate: inventing a gesture the design does not show |
| SH-03 | Phone: touch-pan-while-drawing (v2.10 `#sculptNavpad`) | 710-714 | `main.gd` carries no such handling to port forward — grepped | yes | §4.5.6 requires it | (B) small. **Narrowed 2026-08-24, not closed:** SH-14's ✋ pan-mode toggle covers the *need* (a single finger that pans instead of drawing) with one latched button rather than a velocity joystick — the reference's own call (`panMode` at 9623 is a pan route the joystick is not). Genuinely unbuilt: **panning without lifting the drawing finger**, mid-stroke, the case `#sculptNavpad` was added for. Whether that is worth building is an open question, to reassess against real Sculpt touch usage |
| SH-04 | Phone: battery / signal glyphs | 863-868 | checked against this Godot build's own `OS` class: no `power`/`battery` method exists | yes | §13's mockup | **(D)** — nothing real to back them cross-platform; only the clock gets real data |
| SH-05 | Layers popover: hotkey badges 1–8 | *done, 2026-08-19* | `layers_popover.gd`'s `_add_hotkey_badge`/`_register_hotkeys`/`_input` | yes | §10: *"grouped rows with hotkey badges"* | **(A)**, closed — badged the first 8 rows in `LAYER_GROUPS`' real build order (Base/Climate/Tectonics, not the spec's SURFACE/TERRAIN FIELDS/CLIMATE, which has no matching row names); real `InputMap` actions, scoped to popover-open |
| SH-06 | Viewport ▸ `→ 1 582 m` (draft-stamp elevation under the cursor) | *baseline done, suffix genuinely blocked, 2026-08-19* | `viewport_host.gd`'s `_coords_text` | yes, corrected | §10 | **(A)** for the km-E/km-N/elevation baseline (built, `sample_cell`); **reclassified (B)** for the `→` draft suffix — `sample_cell` reads only `WorldState::field`, never the sculpt `PassBuffer` draft, and `build_sculpt_preview_texture` composites the draft into a colourised texture, not a per-cell elevation `#[func]`. The register's premise that this call existed was wrong. |
| SH-07 | Status bar ▸ `autosave` and `atlas` slots | `dcc_shell.gd:657` builds both; nothing writes them | none | — | §10's middle group | **`atlas` DONE 2026-08-24** — `app.gd`'s `refresh_atlas_status()` writes chunk count, deepest level, bytes and finalize state, blank when nothing is baked (a permanent "Atlas: empty" would spend a slot saying nothing). **`autosave` still open**, gated on FI-03 |
| SH-09 | Layers popover: **Wind / Ocean currents are animated in the reference and were static here** | *done, 2026-08-23* | `shell/wind_fx_layer.gd`, attached from `layers_popover.gd::_attach_flow_fx` | yes | the reference's own `#windFxCanvas` particle-streak overlay (`_windFx*`, HTML lines 2113-2209) — not in any mockup | **(A)**, closed — owner-reported (*"the ocean current layer isnt animated as the HTML version is. (same for wind)"*). The static rasters were correct and untouched; the reference stacks a **second** overlay on those two views: 260/200 particles advected along the flow field at `0.315` cells/tick, drawn as fading streaks, respawned on leaving the map, ageing out, or (ocean only) beaching. Ported constant-for-constant. One deliberate technique change: the reference fades a persistent canvas with `destination-out`; a per-particle history redraw reaches the same streak without a never-cleared `SubViewport` working behind a closed layer. Nothing runs while the view is off (verified: 0.0000 frame-to-frame diff) |
| SH-08 | Menu accelerators for the disabled items (⌘S ⌘⇧S ⌘W ⌘Z ⌘⇧Z ⌘X ⌘C ⌘V ⌫ ⌘A ⌘D ⌘F ⌘⇧P) | `menus.gd` sets only `Ctrl+N`, `Ctrl+O`, `⇧A`, `⇧J` | none | — | §2's tables give every one | **(D)** — an accelerator on a permanently disabled item is dead weight; they arrive with their items |
| SH-10 | **Phone: pinch-to-zoom did nothing** | *fixed 2026-08-24* — `project.godot`, new `[input_devices]` block | n/a — previously undisclosed, because nothing looked missing | yes | §13's map is the whole screen; pinch is the only zoom affordance a phone has | **(A)**, closed. Owner-reported (*"zooming doesn't seem to work on the phone"*). Not a code gap: `viewport_host.gd:406` always handled `InputEventMagnifyGesture` via the wheel's `_zoom_at()`, but Godot's Android layer only attaches its `ScaleGestureDetector` when `input_devices/pointing/android/enable_pan_and_scale_gestures` is on, and the engine default is **false** — the branch was dead on every phone. Confirmed three ways: `ProjectSettings.has_setting()` true / unset value `false` on 4.7.1; `dexdump` of the shipped APK showing `onScale`/`onScaleBegin` gating on `panningAndScalingEnabled` (`setQuickScaleEnabled` never called, so no single-finger fallback either); and a real two-pointer MT-B pinch injected through AOSP `uinput` — **z1.0 → z2.2** out, **z2.2 → z1.0** in, against a **control APK with the setting off reproducing the bug (z1.0, unchanged)** |
| SH-11 | **`ViewportHost._zoom_at()` pivots against the wrong origin** | found 2026-08-24 while fixing SH-10; **FIXED 2026-08-25** (§38) | n/a — previously undisclosed | — | §10's viewport is expected to zoom under the pointer | **(A)**, closed — §38 owns the fix and its measurement (**32.59 px** of drift per wheel notch before, **0.00 px** after, at three probe points). Cause: `_input()` delivers `event.position` in *viewport* coordinates while `_camera` is a child of `ViewportHost`, so `_camera.position` is `ViewportHost`-local, and `viewport_host.gd:427` subtracted one from the other: the pivot was off by `ViewportHost.global_position` — measured **(412, 70)** on the desktop layout from a headless `app.tscn` instantiation, not inferred. Wheel and pinch affected; *pan* not (a delta of two global positions is offset-invariant), and `move_view_to()`/`_update_lod()` were already local, so line 427 was the single inconsistent site. Barely visible on the phone (edge-to-edge map, offset ≈ 0), which is why SH-10 verified clean. Deferred on 2026-08-24 only because it changed desktop zoom the owner then called correct and `viewport_host.gd` had concurrent work in it |
| SH-12 | **`DccWidgets.note()`'s `custom_minimum_size.x` was wider than the right dock's own documented minimum** | *fixed 2026-08-24* — `dcc_widgets.gd::note()`, `240` → `190` | disclosed only in `CHANGELOG.md`'s "Still open" (695821f), never registered — `PARITY_AUDIT.md` pass 2's **F8** | now yes, this row | `DccTheme.W_RIGHT_DOCK_MIN` (260) is the dock's documented floor | **(A)**, closed. Static per context, so it never jittered (unlike the `_field()` value-label bug this file fixed) — simply wrong: 240 px plus `section()`'s 26 px of margin (14 left + 12 right) is 266, and a `group()` one level deeper adds 10, so the tightest call site (`right_dock.gd`'s Measure ▸ Actions, a note in a group in a section) needed 276 against a 260 px dock. The right dock could not reach its own minimum on nearly every context that draws a note (Sample-with-no-world, River, every empty Measure mode, Region, Sculpt, Wildlife). Fixed at the shared widget (`note()` is called from 18 files; the other 17 give it wider columns). `190` leaves 33 px for the `ScrollContainer`'s scrollbar in the tightest nesting. Headless boot-check clean; **the left dock and workspace panels were the other unaudited half of F8's note** — covered by the same fix, but not separately measured against a documented minimum |
| SH-13 | **Phone: the map could not be panned at all** | *fixed 2026-08-24* — `viewport_host.gd`, new `InputEventPanGesture` branch in `_input()` | n/a — previously undisclosed; `viewport_host.gd:407` disclosed only the *single-finger* half, and reasoned it away correctly | yes | §4.5.1 makes Pan/zoom a global, always-available modifier; §13's map is the whole screen | **(A)**, closed. Owner-reported (*"how to move around, snapping the view back to 100% etc."*). Pan was **MMB or Space+LMB only** — a handheld has neither, so with SH-10 fixed the phone could zoom but never move. Measured first: a single-finger `adb shell input swipe` changed **51 pixels**, all the hover cursor. The fix is the *other half of the gesture pair SH-10 switched on* — `enable_pan_and_scale_gestures` gates both, and `dexdump` shows `GodotGestureHandler.onScroll` emitting `handlePanEvent` beside `onScale` — so no new setting or permission. It matches the reference: one `touchmove` handler drives zoom about the centroid **and** pan by its delta (HTML lines 14014-14015), and gives the single finger to the tool (*"one finger keeps painting/drawing"*, HTML line 13988), as `viewport_host.gd` already did. Verified on device (OnePlus 6T, LineageOS/Android 15) with two-pointer MT-B drags via AOSP `uinput`, constant span so `ScaleGestureDetector` never fires: **finger −400 px → map −163 px**, **finger +400 px → map +163 px**, `z1.0` throughout, round trip to a **byte-identical frame (0.000 mean abs diff)**, on both a 2048×1311 and a 1024×655 world. **Deliberately uncalibrated, not a defect: the gain.** `dexdump` shows Godot's `onScroll` divides the delta by **5.0** (`const/high16 0x40A00000`; `handlePanEvent`/`setPanEvent` pass it through), predicting 0.20×, but the measured gain is **0.41×** — a factor of ~2 the bytecode does not explain. The handler stays 1:1 with the platform delta rather than tuning to an unexplained one-device number; calibration is a separate pass — see SH-14 |
| SH-14 | **The reference's mobile navigation cluster — the `#zoomOverlay` zoom pad, the ✦ pan toggle, `zoomReset`, and what "100%" means** | *closed 2026-08-24* — `viewport_host.gd`: `_build_navpad()` (the four-button column), `zoom_step()`, `set_pan_mode()`, a `MOUSE_BUTTON_LEFT` branch in `_input()`, and a rewritten `reset_view()`; three new glyphs in `dcc_icons.gd` | never disclosed — no register row, and no "zoom pad"/"reset view" entry before this one | yes | **now yes** — `design/Cartalith Android Phone.dc.html` supplied the language; the cluster was designed this pass (published canvas "Cartalith Phone Navpad": a viewport artboard and an anatomy/states artboard) | **(D) → (A), closed.** Owner-reported with SH-13 and raised as a design decision. **Two owner decisions, 2026-08-23:** (1) **reset means cover, not fit**; (2) **the cluster is designed in this shell's language first** — four floating web buttons are a mobile-web idiom §13 uses nowhere else. **The reference, checked line by line:** `zoomIn`/`zoomOut` (13464-13465) are `zoomAt(viewCenter(), 1.35)` and its inverse — the view centre, since a button carries no map position; `panBtn` ✋ (13963) is `panMode=!panMode`, a **latching toggle, not press-and-hold**, routing a plain button-0 pointerdown to the pan drag (9623) and suppressing the armed tool (13924); `zoomReset` ⟳ (13466) **clears `panMode`** and calls **`_viewFill()`** (13294), never `resetView()` (13390) — so **"100%" is the COVER scale, not scale 1**. **Built:** a right-edge column of four 44 dp pills at the `right:14px` / 10 px-gap geometry of the phone canvas's artboard 01, riding `_safe_insets` to clear app bar, bottom bar, timeline and gesture strip; drawn glyphs (`zoom_in`, `zoom_out`, `view_fill`, existing `tool_pan`) rather than `+`/`−`/✋/⟳ text, for one family and because `⟳` (U+27F3) is missing from Plex Mono and its fallback chain; the pan pill latches to **accent fill with a dark glyph**, the canvas's on-toggle idiom. Pan mode reuses `_panning`, and handling the press in `_input` (before GUI dispatch) keeps the armed tool from seeing the finger — the reference's `!panMode` guard for free. **`reset_view()` was the larger half:** plain fit (`_zoom = 1`, `position = ZERO`) with **no UI caller**, running only on generate/load — the letterboxed state the reference's v1.01 fixed: at 393×852 on a 2048×1311 world, a 251 px band with **300 px of dead ground above and below**. Now cover: `max(size.x/fit.x, size.y/fit.y)` over `overlay.displayed_rect()`, the reference's `_viewCoverScale` with its `max(1, …)` floor for free (`zoom == 1` is already the fit rect). After: **covers both axes, centred, zoom 3.387**. **Two recorded deviations:** (i) the reference's `panX/panY = 0` crops the loose axis asymmetrically — an artifact of `transform-origin: 0 0` over a flex-centred wrap, not intent (its comment at 13290 says *"cover scale, centred"*); this centres. (ii) The standing pan clamp (`_viewClampFill`, 13295) is **not** ported — it runs on every `applyView()`, changing all four pan routes, and would fight `ZOOM_MIN = 0.4`, which lets this camera zoom below fit. **Reachability: every touch device, not phones only** — gated on `_touch`, not `DccShell._phone`: the reference's `isMobile` is really testing *"no wheel, no middle button, no space bar"*, as true of a tablet, while `_phone` is an **aspect-ratio** layout test a tablet fails. **Verified windowed at 393×852:** 4 buttons, each 44×44 at x=335 (14 px clear), 10 px apart above the coordinate readout; reset zoom **3.3866811** against an independent cover of **3.3866811**, `covers_x`/`covers_y`/`centred` true; zoom in **×1.35**, out **×0.740741** exactly; a one-finger drag moves the camera **0 px with pan mode off, −120 px on**; a drag starting **on the pad** moves it 0 px; ⟳ restores cover **and** clears the latch. **Still open, deliberately:** the pan clamp; and **desktop still has no `reset_view()` caller** — adding a View-menu entry is a menu-naming decision for §7's audit, not something to slip in here |

### 6.16 Urban morphology — added 2026-08-23, previously undisclosed entirely

**Added by this correction pass** (`PARITY_AUDIT.md` C3): before it, this
2 027-line register had **zero occurrences** of "urban", "city viewer" or
"town layout" — no coverage of what `README.md`/`STATUS.md` then called the
largest single unported subsystem. `URBAN_MORPHOLOGY_SCOPE.md` is the
authoritative document; this table is the register's cross-reference to it.

At the time, `cartalith-urban` was 4,516 lines across milestones 1-7 of ~17
(RNG substreams, geometry kernel, planar street graph, A\* over the cost
raster, generation rules + culture profiles, the site model, anchors and
primary routes, organic growth — each with its own `tests/golden.rs`), with
**zero consumers**: `grep -rn 'cartalith-urban' crates/*/Cargo.toml` returned
only its own manifest (`cartalith-godot/Cargo.toml` did not depend on it), and the only mention under `godot-project/` was a
disclosure comment in `civilization_workspace.gd:490-491`. Milestones 8-17
(radial streets/plaza/waterway, water infrastructure, fortification, graph
cleanup, blocks/parcels, districts/buildings, amenities, hinterland/decay/
details/metrics, `generate()`/`hashModel`, the 28-function civ adapter) were
unbuilt.

| # | Missing surface | Reference control | Class |
|---|---|---|---|
| UM-01 | **Town layouts drawn on the map at deep zoom** | `civUrbanLayoutsChk` | *partly closed, 2026-08-23; **substantially closed 2026-08-24*** — the layer draws real engine output, now including milestone 12's blocks and the lots platted in them. On by default since 2026-08-24, with `_umLayoutAlpha`'s own 24 km → 10 km crossfade; buildings and the wall circuit (13, 10) remain the ceiling. *(Stale, 2026-09-24: `urban_layout_draw.gd` now draws `buildings`, `wall_ring`/`wall_gates`/`wall_spurs`, `fort_trace`/`fort_ravelins`, the citadel, `bridges` and farmland. `ALIGNMENT_AUDIT.md` Part 1 item 29 lists the undrawn remainder: ford, churches, civic buildings, and harbour piers, mole and defences.)* |
| UM-02 | **City Viewer modal** — its own canvas, zoom/pan, legend, info panel | `cityViewerModal`, `cvCanvas`/`cvCloseBtn`/`cvLegend`/`cvInfoPanel`, `_cvDrawCity`, `_cvZoomAt` | *partly closed, 2026-08-23; **substantially closed 2026-08-24*** — `shell/city_viewer_window.gd` now draws a town plan rather than a wire diagram, and its fit is the reference's own built-mass fit at last. Same remaining engine ceiling, stated on screen. *(Stale, 2026-09-24: same renderer as UM-01, so the same correction applies. Its on-screen stage list and not-drawn list are themselves stale; see `ALIGNMENT_AUDIT.md` Part 1 item 14c.)* |
| UM-03 | **Layout thumbnail in the place-edit popup, and its launcher** | `peCityPreview`, `peCityOpen` | **`peCityOpen` CLOSED 2026-08-23** — the place-edit popup now exists (§18.1) and its Actions section calls `app.open_city_viewer(index)`, which is exactly the one line this row predicted. `peCityPreview` (the *thumbnail* inside the popup) stays open: it needs a rendered layout at icon size, not a modal. *(2026-09-24: that reason is gone. The right dock draws an 84 × 84 plan with `urban_layout_draw.gd`; only the popup lacks one. See §18.3.)* |

### UM-01/UM-02 — what closed, 2026-08-23

`cartalith-urban` got its first consumer. Three new pieces, no change to the
engine crate:

- **`cartalith-civ::urban_adapter`** — the reference's block-2 `_um*` adapter
  (`_umSiteBoxKm`, `_umWaterNearKm`, `_umWaterReachKm`,
  `_umSiteKindFromTerrain`, `_umInferAge`, `_umRayBoxExit`,
  `_umWayBearingFrom`, `_umRouteEnds`, `_umPrimaryPaths`, `_umTerrainOrient`,
  `_umWaterCtx`, `_umTerrainCtx`, `_umPlaceContext`), plus the prefix of
  `generate()` that milestones 1-7 supply — milestone 17's named home, started
  early and deliberately partial. Its module header gives the function-by-
  function boundary, including the eight `_um*` functions not ported and why
  each one's *consumer* is milestone 8+.
- **`cartalith-godot::urban_bridge`** — one `#[func]`, `urban_layouts(indices)`,
  batched so the full-grid river trace `_umWaterCtx` needs runs once per batch,
  not once per town.
- **`shell/urban_layout_draw.gd`**, **`shell/city_viewer_window.gd`**, and
  `map_overlay.gd`'s "Urban layouts" block.

**Not drawn, and absent rather than stubbed**: blocks and plazas (milestone
12), parcels and buildings (12-13), districts and amenities (13-14), the wall
circuit and gates (10), harbour and quay (9), bridges and fords (9), farmland
and hinterland detail (15). The bridge emits **no key** for any of them — an
empty `buildings` array would read as "this town has none" rather than "this
port cannot generate any yet" — and the City Viewer's info panel names the
list on screen.

**A divergence, later withdrawn**: the reveal gate was the town's site box
in screen pixels rather than `_umLayoutAlpha`'s 24 km → 10 km crossfade,
because the camera then clamped at `ViewportHost.ZOOM_MAX` (8.0) — ~100 km at
best on the default 800 km world — so a ported 24 km threshold would never
fire. **Withdrawn 2026-08-24**: the cap became `lodMaxZoom()`, the pixel gate
measured wrong, and `_umLayoutAlpha` is ported for real — see "UM-01 — the
owner could not see a town on the map at all" below.

### UM-01/UM-02 — what closed next, 2026-08-24

The owner asked for better viewer rendering against a MapEffects-style
battle-map illustration whose caption is the brief: *"mix up the brightness
and saturation of the rooftops for a more natural look."*

**That technique needs rooftops, and there were none** — a street graph has
nothing discrete to fill. So `URBAN_MORPHOLOGY_SCOPE.md` **milestone 12**
(`buildBlocks`/`buildParcels`) was ported out of order: parcels are the
smallest stage that yields a colourable shape, every primitive they need was
already golden-tested at milestones 1-2, and it was smaller than inventing a
Voronoi subdivision to fake the same shapes — and it is the reference's own
algorithm.

- **Blocks and lots are drawn**, each roof a different brightness and
  saturation of one warm palette from a stable per-parcel scalar the engine
  emits — brightness up and saturation *down* together, because a weathered
  roof is both.
- **The City Viewer's fit is the reference's own.** It had fitted the whole
  graph (a disclosed degradation) because `_umDrawLayoutPreview`'s built-mass
  fit had no mass; blocks are that mass, so long approach roads no longer
  shrink a town to a speck.
- **Map palette and shell palette are kept apart.** Map content stays warm ink
  and parchment and does not follow the light/dark theme (this register's §6
  rule, and `map_overlay.gd`'s for faction colour); the shell's amber `accent`
  appears only on annotation *over* the map — market anchor, approach-road ends.

**Two measured findings changed the code**: a 6-town sheet redrew in 577 ms
until every roof edge was folded into one `draw_multiline` (102 ms; the
viewer's 4,370-lot worst case, 46 ms); and a dense city rendered as a black
mass until the ink and ridge passes were gated on the *measured* on-screen lot
size rather than zoom — at ~3 px a lot, the outline is wider than the roof.

**Still not drawn**: buildings (13), districts and amenities (13-14), the wall
circuit and gates (10), harbour and quay (9), bridges and fords (9), farmland
(15) — still no dictionary key at all.

**Two disclosures the info panel makes in words**, because `stages` cannot:

1. ~~**There is no open market square.**~~ **Closed 2026-08-24.** `buildPlaza`
   (milestone 8) is ported and runs where `generate()` runs it — between
   `buildPrimaries` and `grow`, on the organic and radial branches. The engine
   flags the block containing `plaza.center` and plats no lots on it; the
   bridge carries a `block_plaza` flag beside `blocks` plus the square's
   `plaza` outline. `urban_layout_draw.gd` fills that block a shade lighter
   (the reference's rgb(208,192,154) against rgb(182,172,148)) and strokes the
   outline over the roofs. The "Market place" legend row and the panel note
   are conditional on a plaza existing — a site with no primary to widen gets
   none, and a swatch for a colour not on screen is a lie.
2. **A rooftop is a whole parcel, inset.** `buildBuildings` would put a smaller
   footprint in each lot, with a grammar per district and a terrain gate
   leaving some empty; here there are no gaps and every roof is the same quad.
   The one place the drawing is ahead of the generator, labelled as such.

### UM-01 — the owner could not see a town on the map at all, 2026-08-24

Owner report: *"I don't see the settlement rendered on the map itself, the dot
yes. But not the place."* Three defects, only one the suspected one, all
measured live with `_umreveal_shot.gd` (800 km world, 440 px map area, deepest
zoom 160 = a 5 km span).

**1 · The layer was off by default, on a row nobody would find.**
`civUrbanLayoutsChk` shipped `on: false` in `cartography_workspace.gd`'s
"Visible layers" list (the CARTO rail dock), while the map's own **Layers
button** popover lists *field rasters* only — so a user looking there concludes
the feature does not exist. It now defaults **on** (the one divergence from the
reference's default, made free by the band below: nothing is generated or drawn
until the map spans under 24 km), and the popover's footnote names town layouts.

**2 · The pixel reveal gate was the wrong number** — not unreachable, the
opposite. `URBAN_MIN_BOX_PX = 16` first fired at a **47 km** span, and because a
revealed town *replaces* its pin, a legible marker became a 16 px speck two
octaves early. The reference's band is ported verbatim (`UM_FADE_FAR_KM = 24`,
`UM_FADE_NEAR_KM = 10`, against `lodSpanKm()`), and `draw_layout`'s `alpha`
argument — plumbed from the start and passed `1.0` ever since — carries the
crossfade. Measured: α = 0.00 at 25 km, 0.03 at 23.5, 0.44 at 17.8, 0.76 at
13.3, 1.00 at 10.0 and below. The pixel constant survives as a floor, stopping
a narrow map area (a phone, or a map squeezed between two docks) drawing a
sub-pixel town just because the *span* qualifies.

**3 · The pin ballooned into the town — visible with the layer off, which is
what the owner was looking at.** `_civ_zoom_k()` ported `_civZoomK`'s
`1/max(0.35, min(5, z))` including the `min(5, …)`. That cap is free in the
reference because `viewT.scale` **stays at 1 under Tiled LOD** (its deep zoom
is `_lodZoom`). Here `_camera_zoom` *is* the deep zoom, so past 5 pin, glyph,
name and label outline grow linearly: 1.6x overshoot at the old 8.0 cap, **32x**
once the cap became `lodMaxZoom()`; at z=60 pin and label covered the whole
settlement. The cap is no longer ported; the `0.35` zoom-*out* floor is
untouched.

**Alignment, checked because the owner asked and the HTML had a bug class
here** (a "coastal" town not touching the coast; a river town whose streets
floated off the river). Measured with `_umalign_shot.gd`: every geometry goes
through the *same* local-metres → grid transform `_draw_urban_layouts` uses,
then is compared against `sample_cell()`'s water mask and Strahler order and
`roads()`'s way polylines. A **60 km** world deliberately — at 800 km the
1.7 km site box is about **one** cell (1.56 km/cell) and sub-cell displacement
is unmeasurable; at 60 km / 512 the box is 14.5 cells. 41 settlements, 41
layouts.

- **No rotation is applied.** `orient` is `0.000000` on all 41 — the
  reference's rule (`const orient = water ? 0 : _umTerrainOrient(...)`), and
  every site here has water, so the rotation-displacement bug class cannot
  arise.
- **Rooftops: 0.00% on a real water cell**, every town measured (57, 94, 101,
  140, 141 and 325 rooftops). Sweeping the layout ±3 cells in half-cell steps
  finds the minimising offset at exactly **(0.0, 0.0)** in every case.
- **River towns: exact.** Every drawn river vertex is **0.71 cells** from the
  nearest river cell centre — √2⁄2, a *cell corner* to the centres around it.
  The drawn river is the real river, vertex for vertex.
- **Coastal: within one cell.** `Skalbjorkellwick`, a `bay` capital, carries a
  78-vertex traced shoreline: **mean 0.75 cells (88 m), worst 1.12 cells
  (131 m)** from the real coastline, a cell being 117 m. 325 rooftops, none in
  the sea.
- **Roads connect.** Where a real way reaches the town, approach-road ends land
  **3 m to 146 m** from the nearest way point — under 1.3 cells.
  `Skalbjorkellwick` is the exception and not a defect: the nearest way is 4 km
  off, so its ends are the reference's synthesised box-exit bearings.

**One real gap — content, not displacement**: three of the four `bay` sites
draw **no sea**. One box is 72.3% real ocean and draws 0.5% water; another
32.1% and 0.0%. Only the fourth gets a traced shoreline. Invisible on the map
(the terrain raster shows the sea and the town is correctly placed) but the
City Viewer, with no terrain under it, shows no water at all.
`URBAN_MORPHOLOGY_SCOPE.md` milestone 9's ground (harbour, quay, shoreline).
And at 800 km a whole town is ~1.1 grid cells, so its water is *finer than the
terrain grid* and will overhang a land cell — resolution, not displacement.

`URBAN_FINE_BOX_PX` (per-roof ink outline, ridge, shadow) is map-unreachable at
that width, correctly: at a 5 km span the site box tops out near 150 px, an
~11 m lot is ~1 px, and the outline would be wider than the roof — the
measurement that placed the constant. The fine pass belongs to the City Viewer;
on the map a town reads as a mass with its water and approach roads, as in the
reference at the same span.

UM-03's `peCityPreview` is now *unblocked engine-side* — there is a layout worth
previewing at icon size — and stays open as a UI task. It stays (B) rather
than (C) or (D): the reference precedent is exact and line-cited
(`URBAN_MORPHOLOGY_SCOPE.md`), so it is an engine/UI gap, not a design gap.

---

## 7 · Layer 3 — comparable-application research for (C)

Every (C) entry, with how established applications in the same space actually
solve the problem, what they call it, where it sits in their information
architecture, and a proposal concrete enough to build from without re-searching.
Sources are linked so the research is checkable rather than asserted.

### 7.1 Undo history panel, and what "global undo" covers — ED-02, PR-11

> **Partly overtaken by events, 2026-08-23 — read this box before the
> research below.** Global undo (ED-01) and the memory row (PR-11) are built,
> and they are **not** what this section proposed. Proposal 1 was written from
> the comparables before anyone read `pushUndo`/`undoLast`: the reference's
> global undo is **three functions, one `Float32Array.slice()` and a five-deep
> array**, snapshotting the height field only (not `riverMask`, `riverFloor`,
> climate or civ). A per-subsystem ledger is strictly larger than the gap
> `PARITY_AUDIT.md` §3.1 names, so what shipped is the reference's design with
> one bound changed — a byte budget, because 8192² worlds exist here. Per
> proposal, as of that pass:
>
> - **Proposal 2 (draft/commit tiering)** — unchanged; only the *commit* tier
>   shipped, for the two commits that write height.
> - **Proposal 3 (the panel)** — **built 2026-08-25, see §42**, as proposal 1's
>   ledger in a right-dock context with the linear default. It also records
>   commits it *cannot* reverse, each with its reason, so it does not read as a
>   history of the whole application. Per-subsystem reversal is still unbuilt;
>   turning one on is now a row's kind changing rather than a redesign.
> - **Proposal 4 (Preferences row)** — shipped with the live memory cost, but as
>   a budget in MB, not a depth (PR-11's row gives why). The cap of 30 named
>   below is wrong: `MAX_UNDO` is 5, as the shipped label says.
> - **Proposal 5 (Adjust Last Operation)** — untouched, still (A); §42 left it
>   out as a different surface (a status-bar chip over generation parameters).
>
> Correction to a *source*: `reference/FUNCTION_INDEX.md` line 61 calls the
> reference's undo *"one level per destructive op"*. It is five
> (`const MAX_UNDO=5`; header label "Up to 5 steps saved in memory").

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

> **Decision, 2026-08-20 (owner): recommendation 1 accepted in full,
> recommendation 3 declined.** The Conversion group is **deleted** — all three
> rows, from `menus.gd::_data()` and `data_manager_window.gd`'s `ROUTES` and
> `GROUP_ORDER` — leaving four groups (*in · out · sources · checks*), as §8.3's
> recommendation 1 also asked. CRS was not kept as a project property either.
> The research below stands unchanged as the reasoning.

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

> **Acted on, 2026-08-23** — built as an inline group, not a `⧉` window. The
> assumption that every factor already crossed the boundary was wrong; the
> chain crosses as structured `JpTerm` rows instead (JP-05's row, §6.9, has
> the account). The `formula` **prose** still stays out of the engine.

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
   **Added 2026-08-24**: `Serialize`/`Deserialize` on `TerrainAppearance`,
   `Npr` and `ElevationRamp`, with struct-level `#[serde(default)]` so older
   presets still load; the look is its own named JSON file, not part of the
   world `.zip` (CA-08's row gives why).

### 7.16 Layer list search; Blocks / Verticality — CA-09

The search field needs no research — §7.2's locator, scoped to the layer list.

**Blocks / Verticality is genuinely undefined**, and `DCC_CONTROL_INDEX.md`
already marks it uncertain. No comparable has footer tabs by those names. The
two plausible readings, from the vocabulary of the field:

- **Blocks** = a *block diagram* / 2.5D extruded view — a standard cartographic
  presentation of terrain, and the natural companion to "Verticality".
- **Verticality** = vertical exaggeration, i.e. `TerrainAppearance::exag`, which
  is real.

**Recommendation was: ask the owner rather than design it**, since a wrong guess
would produce a whole pane of wrong controls.

#### Ruled 2026-09-06 — reading **C**: both are dead names, nothing is built

The search field **shipped** (`layers_popover.gd`; the CA-09 row records its
shape). The footer tabs did not, for reasons checked at the symbol:

- **Verticality is `exag`, already a live slider** — not in the LIGHT/relief
  group as guessed above, but in `render_workspace.gd`'s `APPEARANCE_VIEW`,
  **CARTO ▸ Map style ▸ § Map view**, beside the two sun angles (tooltip:
  *"Vertical exaggeration of the relief the hillshade is computed from. The
  reference's own Relief slider"*). Backed by `render.rs`'s
  `TerrainAppearance::exag`.
- **Blocks has no view to switch to.** A 2.5D block diagram is a render mode this
  2D port does not have, and the word is **already taken** —
  `city_viewer_window.gd` prints a town layout's block count as "Blocks".
- **Read as tiles or as a style bundle, it is built and named**: Preferences ▸
  Tiles & LOD (`dcc_settings.gd` §2.5), and `render_workspace.gd`'s
  `STYLE_PRESETS` in Map style.

Two footer tabs would be a **third** route to controls that already have two.
The reasoning is also carried in `cartography_workspace.gd` beside the Layers
category. **This closes the (C) as a decision, not as a design**: if the owner
meant something the three readings miss, it costs one sentence to say so.

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

> **Resolved differently, 2026-08-23.** Global undo was three reference
> functions and a 5-deep snapshot array, not the assumed command framework
> (§7.1's box), so `Edit ▸ Undo` became the live item. The lesson stands: Edit
> sat 100 % disabled on a scope estimate nobody had checked against the
> reference.

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

> **This list is seven entries short of the real (D) set (2026-09-07
> re-count).** It carries 16 live rows plus struck `DM-07`; §3's re-derivation
> counts **23**. Missing: `DM-08` and `DM-09` (deleted by the same owner
> decision as `DM-07`), `CV-13` (its own row says *"Not a gap"*), and `MEA-04`,
> `MEA-11`, `MEA-12` (explicit `**(D)**` rows §16 added later). Nothing here is
> wrong; the list stopped being swept.

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

> **All four have since landed, and two of the twelve rows above need a
> correction (2026-09-07 re-count).** `ED-05` is `place_search.gd`; `PR-15` is
> `Preferences ▸ Units`' three-way `km / mi / nmi` radio over `dcc_units.gd`;
> `PR-16`/`HE-02` is `shortcuts_dialog.gd`'s `open_editable()`, **found already
> built** when a lane opened the symbol; `WI-01` is `menus.gd`'s Layouts popup,
> live since 2026-08-31. **`SH-01`'s "withdrawn" was reversed**: the rail
> expansion was rebuilt against the 2026-08-31 ENV prototype as a separate
> sibling column (`dcc_shell.gd::_build_rail`'s header records why the
> newer-canvas rule settles it). **Only `SH-06`'s `→` draft-elevation suffix
> survived as open, and it is (B)** — hence §3's **(A) = 0**.

---

## 11 · Out of scope

- **This register proposes no implementation order and writes no GUI code.** The
  only edits made were the five factual corrections in §4.
- **`DCC_SHELL_SPEC.md`, `DCC_CONTROL_INDEX.md` and `design/` are untouched.**
  They are owner-supplied ground truth; this document cites them.
- **`DCC_SHELL_SPEC.md`'s header corrections (three numbered, #1–#3, plus its other header notices) are respected, not re-litigated.**
  Every one of them (the sculpt commit prose, per-stage run, Brush shape/Stroke &
  grid/Actions, the sculpt global defaults, §12's text-symbol premise, the path
  note) appears here as a (C) or (D) with the correction cited, never as
  something to "fix".
- **No Rust was changed and none is proposed line-by-line.** (B) rows name the
  missing capability; they do not design it.
- ~~**One real defect found and deliberately not fixed here** ... `timeline_bar` is visible
  and empty in CIVIL and INFRA.~~ **CLOSED 2026-09-21 (verified, already done).**
  Re-opened at the symbol: `app.gd`'s timeline strip (`_fill_timeline_strip()`,
  `_build_timeline_expanded()`/`_build_timeline_collapsed()`) is a built control
  — transport/speed pills, scrub track, layer toggles, year readout — not an
  empty 70 px region. Landed in the 2026-08-31→2026-09-05 timeline-strip work.
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

**Nothing here proposes restructuring the shell.** The canvas's top bar shows
the earlier seven menus (Project · World · Generate · Simulate · Map · Assets ·
View); `DCC_SHELL_SPEC.md` §2 replaced them with File · Edit · Assets · Data ·
Preferences · Window · Help plus a domain rail, merged by the owner to three
domains on 2026-08-20 (commit `42547d9`), and §8.7 settles that in the spec's
favour. The canvas is used only as an **exhaustive surface inventory with
disclosure depth**: does every control the original app had exist in this
shell, live or honestly disabled, or is it simply absent?

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

**The honesty rule held for 83 % of the reference's own surface without
anyone auditing for it**: nearly half the inventory is a disabled item, a
disclosed Data-manager route or an in-product "not ported" note naming the
missing Rust. The 17 in (c) are the real finding, and they cluster: eleven are
*whole-network civ operations and generation passes that `generate()`
absorbed* — the gap a one-shot pipeline hides, because no panel ever needed the
button, and no reader could tell that apart from an oversight.

### 13.3 The (c) list in full — every undisclosed omission

Each row names the reference's own `#id` where it has one, and what was done
about it in this pass. **Nine became disabled controls with a real reason;
seven became in-product prose (a stage `gap` string, a route reason, a "Not
built" note); one pair was wired live.**

> **Update, 2026-08-20.** MS-02 (*Infer tectonics from heightmap*) was built,
> not disclosed, in the heightmap-import pass; its row records what closed it.
> The counts above are left as that audit's own result.

| # | Missing surface | Reference `#id` | Where it now lives | Why it could not be wired |
|---|---|---|---|---|
| MS-01 | **Center landmasses** | `#centerBtn` | **done, 2026-08-23** — the button in `app.gd`'s GENERATE · WORLD tool-options bar is live and calls `WorldGen::center_landmasses()`. Engine: `cartalith_terrain::center` (`bestEmptyColumn`/`shiftGridX`/`featherSeamX`, reference HTML 3156-3177) orchestrated by `cartalith_engine::center::center_landmasses` (`centerLandmasses`, 3179-3199). Golden-parity bit-exact (`golden_parity_center.rs`, 6 tests) | Was: "`generate_terrain` places plate seeds from the seed alone; no centring pass and no post-generate offset exist." Premise right, conclusion wrong: the reference does not re-roll seeds, it circular-shifts every grid after the fact, needing no generation hook. The old tooltip's "re-rolls the plate seeds until the land mass lands nearer the middle" misread the reference; corrected in the same pass |
| MS-02 | **Infer tectonics from heightmap** | `#inferTectBtn` | **done, 2026-08-20** — `Data ▸ Import ▸ Heightmaps (PNG)`, and the welcome screen's own *Import a heightmap* tile | Both halves closed in one pass. The reader is `cartalith-assets::raster::decode_png` + `cartalith_terrain::infer::heightmap_to_field`; the inference is `cartalith_terrain::infer` (`buildReliefField`/`pickPlateSeeds`/`classifyPlateCrust`/`reconstructBoundaryStress`/`stampVolcanicArcs`/`inferPlateVelocities`, reference HTML 6641-6752) orchestrated by `cartalith_engine::import::infer_tectonics`. Golden-parity tested bit-exact against the reference (`golden_parity_infer.rs`, 8 tests) |
| MS-03 | **Fold intensity · trench depth · fault blocks** (structured orogeny) | `foldI`/`trenchD`/`faultB` | `world_workspace.gd` — stage 04 Tectonics' `gap` string, which was **empty** | `generate_terrain` hardcodes the reference's own defaults (0.16, 1.0, 0), so behaviour matches; exposing them threads three fields through `OrogenyParams`' call site |
| MS-04 | **Evolve climate ↔ terrain · Evolve cycles** | `#evolveBtn`/`#evoCyc` | `world_workspace.gd` — stage 06 Erosion's `gap`, which named five passes and not these | **DONE, 2026-08-23 — §19 has the account.** `evolveCoupled` is pure orchestration; the one missing piece was **`cartalith_engine::refresh_climate`** (the reference's `computeFlow(true); refreshClimate();` tail, line 5154), run once per cycle so each cycle's rain reflects the last one's orography. Exposed as `passes.evolve_cycles` (`0` is off; the reference's slider starts at 2 because pressing the *button* is its "on"). 4 cycles: 44.0 % of pixels moved |
| MS-05 | **Sediment fill** | `#sedimentBtn` | same stage `gap` | **DONE, 2026-08-23 — §19 has the account.** `depositSediment`'s kernel is `cartalith_erosion::route_sediment` (mass-conserving, golden-parity bit-exact), composed in `generate_terrain`'s pass block: stream-power carve → per-cell eroded-column supply → `compute_flow` on the carved surface → `route_sediment`. Exposed as `passes.sediment_fill` + `passes.sediment_capacity` (reference default `6.0`); 43.7 % of pixels moved. **`#tidalFlatsBtn` closed 2026-08-24** as the seventh pass, `passes.tidal_flats` + `passes.tidal_k` (`0.45`), on `cartalith_climate::tides` — §19.5 |
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
  `_hidden_way_types`) and one `continue` per draw loop. Hidden, not shown,
  sets, so an empty dictionary means "show everything" — the state of an
  untouched shell and a fresh world; a shown-set would need seeding from a
  roster that does not exist until the first generate. The settlement test sits
  **before** any geometry, so a hidden tier never reserves label occupancy.
- `viewport_host.gd` — `set_settlement_kind_visible` / `set_way_type_visible`,
  separate from `set_layer_visible` because they take a *sub*-key: one string
  namespace would collide `"settlements"` with `"settlements/hamlet"`.
- `cartography_workspace.gd` — two L4 groups under Layers: five settlement
  tiers and three land way types (`sea_lane` keeps its existing top-level row
  rather than gaining a second, disagreeing switch).

A hidden class stays hoverable and clickable — hiding a tier is a cartographic
choice, the same independence `_show_settlements` keeps for the whole layer.

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

Every prior pass in this register verified structurally or headlessly ("nothing
graphical verified"). This was the first that looked: a non-headless boot of
`shell/app.tscn`, a 512×512 world, and every major surface driven,
screenshotted and compared against `design/Cartalith DCC Shell.dc.html` and
`design/Journey Planner DCC.dc.html`.

### 14.1 · Method

**Driver**: a temporary harness (`_visual_sweep.gd`/`.tscn`, the uncommitted
dev-tooling convention `_shot.gd`/`_shot_phone.gd` use) that instantiates
`shell/app.tscn`, generates a deterministic world (`seed 483920, 512×512,
sea_level 0.42, villages on`) and saves a PNG per surface after a multi-frame
settle.

**Renderer**: this machine's GLES3/Compatibility path (`gl_compatibility` per
`project.godot`) crashes deterministically on this AMD RX 7800 XT — reproduced
with the pre-existing `_shot.tscn`, so not introduced here — with
`ERROR: Condition "!texture_allocs_cache.has(p_id)" is true` and a segfault, the first
time `open_project_dialog.gd`'s `popup_centered()` runs (the cold-start welcome
prompt). `--rendering-driver opengl3_angle` (ANGLE→D3D11) avoids it with an
identical picture, so this pass captured under ANGLE. Worth a `TOOLCHAIN.md`
note if another AMD/Compatibility machine hits it.

**Surfaces swept**: welcome prompt; shell default (empty + generated, dark);
light theme; Generate World dialog; Generate Sculpt mode + stamp stack; CIVIL
dock default, Timeline category, a selected settlement's right dock and the
territory overlay; CARTO dock default; Layers popover (open, a view picked, the
view rendered with the popover closed); Asset library + slicer with a real
sheet; Data manager; Travel library; Journey Planner takeover with a real
committed route; the map at three zoom levels including deep-zoom LOD tiles.

### 14.2 · Per-surface verdict

| Surface | Verdict | Notes |
|---|---|---|
| Welcome prompt | **PASS** | Matches the "Open project dialog" welcome mode: three tiles (Create/Import/Drop a .zip), search well, Recent/All worlds/Shared tabs. |
| Shell default (dark, generated) | **PASS** | 3-domain rail, menu bar, tool options row, SAMPLE right dock, laid out per `DCC_SHELL_SPEC.md`. |
| Light theme | **PASS** | Consistent full repaint — no leftover dark-token styleboxes anywhere swept. |
| Generate World dialog | **PASS** | Matches the mockup's ten-stage pipeline list + Planet sliders. |
| Generate Sculpt mode | **PASS** | Stamp stack (Undo/Redo/Commit/Discard) appears in the right dock the moment Sculpt is selected. |
| CIVIL dock default | **DEFECT (fixed 2026-08-23)** | CV-VS-01, §14.4 — a thin horizontal seam, CIVIL-only: a deep-zoom LOD tile boundary, visible because the LOD layer painted the reference's *Relief* ramp over the *Biome* map. |
| CIVIL right dock: stuck Sculpt context | **DEFECT (fixed)** | §14.3. |
| CIVIL Timeline category | **PASS** | Years/filters/simulate-collapse rows present, with an honest "not wired to the map" disclosure. |
| CIVIL territory overlay | **PASS** (after correcting the sweep itself) | Committed territory does not show by itself because `cartography_workspace.gd`'s "Political — territory" layer defaults off, as in the design's opt-in layer model. The first sweep pass mistook this for a bug. |
| CARTO dock default | **PASS** | Layers/Layer properties/Annotation match the "Cartography style" screen. |
| Layers popover + z-order | **PASS** | A picked debug view (Elevation) stays on top of the map after the popover closes — the z-order fix `CHANGELOG.md` records, confirmed live. |
| Asset library window | ~~PASS~~ → **FAIL (corrected 2026-08-20; rebuilt)** | The original verdict ("family rail (8 families), slot grid, inspector, empty-library state all honest and correctly laid out") checked function and disclosure, never layout against the canvas — floating OS-titled dialog, stock slabs, no status line or window-bar title, captions outside tiles, a label/value-pair inspector. Owner-reported in those terms. **§14.6**. |
| Asset library slicer, real sheet | **PASS** (arithmetic) / **FAIL (corrected; rebuilt)** (layout) | The overlay lands exactly on a synthetic 6×4 sheet's cell boundaries and the detection readout is right — that half stands. The modal's layout was a vertical stack of stock widgets clipping its own labels, against the canvas's 760 px two-column card. **§14.6**. |
| Asset library: slicer left open on Close | **DEFECT (fixed)** | §14.3. |
| Data manager window | ~~PASS (after fix)~~ → **FAIL (corrected 2026-08-20; rebuilt)** | The original verdict ("Conversion group confirmed gone from the routes rail; the subtitle text still advertising it was the one leftover") was a content check. Layout did not match: a floating 920×600 `AcceptDialog`, a `§`-sigil rail of autowrapping buttons, no window bar, footer or status line, and one grey paragraph where the canvas designs seven columns. Same miss as the Asset library, found by the same owner test. **§14.7**. |
| Travel library window | **PASS** | Animals & mounts tab, 7 stock entries, correct read-only-stock footer. |
| Journey Planner takeover | **PASS** (after correcting the sweep itself) | Spine map, profile/stage selector, stage matrix, party form and right-dock summary (time/load/supply reach/cost/vessels) all render once armed from CIVIL. Armed from CARTOGRAPHY it shows nothing — documented behaviour (`_recompute_visibility()`), but see JP-VS-01. |
| Map, 3 zoom levels + deep-zoom LOD | **PASS** | Pins and labels tier correctly; deep-zoom (z8.0) tiles are visibly pixelated — the known characteristic `tool_overlay.gd`'s header already quotes the owner on ("there is still a certain pixilated quality to the map when we zoom"), not a new finding. |

### 14.3 · Defects found and fixed

**AL-VS-01 — Sprite-sheet slicer modal stranded on top of the whole app.**
`asset_library_window.gd`'s Close called only `hide()` on the parent
`AcceptDialog`; the slicer (`_slicer`, a separate child `Window`) has its own
visibility. Reproduced through the real click path (`close_btn.pressed`): the
slicer stayed on top of everything opened afterwards (Data manager, Travel
library, Journey Planner). Fixed on all three paths (Close, Escape, titlebar ✕)
— `close_btn.pressed`, `close_requested` and `canceled` all call
`_close_slicer()` before/on `hide()`. Re-verified:
`08c_asset_library_closed_slicer_gone_check.png`.

**CV-VS-02 — Stamp stack stuck in the right dock outside WORLD domain.**
`right_dock.gd`'s `show_sculpt_stack()` doc claims "Sample stays the default
everywhere else", but nothing reset `_context` to `CTX_SAMPLE` on a domain
switch, so arming Sculpt in WORLD and switching to CIVIL/CARTOGRAPHY left a
World-only panel with no tool armed. Fixed with `RightDock.leave_sculpt_context()`,
called from `app.gd`'s `_on_workspace_changed()` whenever the new domain isn't
`"world"`; it resets only `CTX_SCULPT`, leaving settlement/route/faction
selections alone (Inspect's selection is domain-independent by design).

**DM-VS-01 — Data manager subtitle still advertised the deleted Conversion
group.** The header hardcoded "import · export · sources · conversion ·
validation" from design §9, though Conversion was deleted 2026-08-20
(`GROUP_ORDER`'s doc comment, `DCC_SHELL_SPEC.md` §2.4's correction note). Now
"import · export · sources · validation," matching `GROUP_ORDER`.

### 14.4 · Defects catalogued, not fixed at the time

*(CV-VS-01 fixed 2026-08-23. **JP-VS-01 too**, 2026-08-24 — §27's `open_journey_planner()` calls `select_domain("civilization")` first, the one-line fix its entry below predicts. This line said "still open" until §38 checked it.)*

**CV-VS-01 — A thin horizontal seam across the map, CIVIL-domain-only.
FIXED 2026-08-23** — a deep-zoom LOD **tile boundary**, reading as a coloured
hairline because the LOD layer painted a different picture from the map under
it. Two causes, both removed:

1. `lod_bridge::synthesize_tile_rgba` passed `tile_bounds(...).to_float()`
   straight to `amplify_region`, whose output index maps to
   `rx + ox/(out-1)*(rw-1)` — endpoints inclusive, a *sample* convention — while
   the base raster is texels (cell `i` covers `[i, i+1)`). A tile stretched
   `TILE_CELLS` cells of screen over `TILE_CELLS - 1` cells of data (1.6%), half
   a cell out of register, leaving a discontinuity down every tile edge. New
   `tile_sample_region` solves `cx + 0.5 == bx + (ox + 0.5) * bw / out`, so
   adjacent tiles sample exactly one texel apart across a shared edge —
   unit-tested at both ends and on an edge-clipped tile.
2. The LOD layer rendered the reference's *Relief* tile
   (`render_height_tile_rgba`'s hypsometric ramp) over the *Biome* base map —
   gold-toned because the hypso ramp's `0.38` stop is `[201,178,74]`. Tiles now
   take colour from `map_view`'s texture through `shell/lod_tile.gdshader` and
   carry only a relief-detail shade ratio.

Measured on the pre-fix build, CIVIL fit view: a row-discontinuity scan read a
**median of 2.26** with spikes of **19.03** at y=599 and **10.30** at y=378 —
both on a `TILE_CELLS` row boundary of that letterbox rect (map top 154, 111 px
per tile row, boundaries at 265/376/487/598/709). CIVIL-only because that
domain's taller dock changes the letterbox rect, moving a tile row into the
middle of the picture. Full account in `cartalith-native/docs/CHANGELOG.md`,
"Deep-zoom LOD: the tiles were the reference's *Relief* view, not its *Biome*
view".

The original investigation (seen in `06_civil_dock_default.png` and
`06b_civil_timeline_category.png` as a dashed, gold-toned full-width hairline
near the letterboxed map's vertical midpoint, absent from every WORLD and
CARTOGRAPHY shot of the same world and camera) established negatives that
all stand:

- **Not a data/logic bug** — temporary instrumentation showed `map_overlay.gd`'s
  `_roads` (48), `_sea_routes` (4), `_settlements` (240),
  `_show_roads`/`_show_sea_routes` (both `true`) and `_border_frac`
  byte-identical either side of the CIVIL switch.
- **Not the sea-route dash styling**, despite matching `SEA_ROUTE_DASH_COLOR` —
  the four sea routes' point ranges (e.g. `y: 282→44`, `y: 330→313`) never span
  the width at near-constant `y`.
- **Not a transient LOD-backlog artifact** — unchanged after an extra 1.5s
  settle (`06z_diag_after_settle.png`, diagnostic run only).
- **Correlated with a letterbox-rect change** — the lead that proved right:
  `ViewportHost`'s map rect is `[P: (18.5, 0.0), S: (935, 935)]` in WORLD vs `[P: (53.5, 0.0), S: (865,
  865)]` in CIVIL, the kind of resize `map_overlay.gd`'s
  `resized.connect(func(): queue_redraw())` reacts to; the seam sat near 50% of
  the new rect's height. `_interior_rect`'s clip/inset math was checked and
  rejected (it insets all four edges symmetrically, not a midline).

**JP-VS-01 — Arming Journey from outside CIVIL gives no visible feedback.**
`_recompute_visibility()` gates the takeover on `app.active_domain() ==
"civilization"` (apparently intentional, given the `JP-13` reference in
`_hide()`'s comment), but `Data ▸ Journey planner… ⇧J` and `app.gd`'s
`open_journey_planner()` arm it from any domain with only a status-bar line
("Journey armed — Esc to release") changing — indistinguishable from a broken
takeover. Left unfixed because it was unclear whether this was a considered
`JP-13` decision; the candidate fix was `open_journey_planner()` calling
`select_domain("civilization")` before `journey_planner_view.open()`.

### 14.5 · Verification

- **Non-headless boot**, real GPU-composited frames via
  `get_viewport().get_texture().get_image()`, not a `SubViewport` fallback.
- **Parse-check**: the four edited files ran inside a full app boot, the sweep
  exercising each fixed path — `_close_slicer()` via the real Close button,
  `leave_sculpt_context()` via a live domain switch away from Sculpt-armed WORLD.
- **Boot-check**: `--headless --path godot-project --quit` clean.
- **Screenshots**: `cartalith-native/godot-project/_visual_sweep.gd`/`.tscn`
  produced 23 PNGs; not committed (screenshots are not source).

### 14.6 · The Asset library window was passed too leniently — corrected and rebuilt (2026-08-20)

The owner, after this sweep shipped: *"The asset manager menu looks nothing
like the DCC work from Claude design."* He was right, and §14.2 had said
**PASS**.

**How a passing check missed it.** The sweep asked whether controls were
present, wired to real engine calls, and honestly disclosed when disabled — all
true, none of them the owner's test (*does it look like the canvas*).
`asset_library_window.gd` was written from `DCC_SHELL_SPEC.md` §8's **prose**
before its bindings existed (the 20+ `#[func]`s came in `8506f13`, the slicer in
`e96a7ae`), and nothing ever laid it against the canvas's `Asset library window 1920`
screen. **A functional check and a visual check are different passes, and
a sweep that only runs the first must say so rather than record a PASS.**

**The 19 deltas**, canvas against the sweep's `08_asset_library_window.png`:

| # | Canvas | What shipped |
|---|---|---|
| 1 | Full-bleed workspace window, 34 px window bar of its own | Floating 1180×760 `AcceptDialog` with an OS title bar |
| 2 | One control vocabulary: `padding:4px 9px; border:1px solid` outline chips, Plex Mono 11 px | Stock Godot filled slabs, stock `OptionButton`, filled search well |
| 3 | `⧉ ASSET LIBRARY` · `map hidden while open` · divider; `☑ Select · 3`; `Sort: slot order ⌄` | No title, subtitle or divider; `Select (0)`; `Slot order ⌄` |
| 4 | Pack metadata is a block in the **inspector** | A NAME/AUTHOR/LICENSE row under the window bar |
| 5 | 26 px status line (`● library edited — apply to map to use it`, counts, keys) | Absent |
| 6 | Rail opens with a 28 px `FAMILIES · N` band | ~90 px of grey disclosure prose |
| 7 | Plain tracked group headers; rows 26 px code · name · `filled/capacity`, accent when incomplete | `§`-sigil `DccWidgets.section()` headers; rows one string at one colour |
| 8 | Selected row: `accent_wash` ground, accent code, brightened name | No selected-row treatment |
| 9 | `Import image…` / `Import pack…` side by side, equal flex | Stacked, full width |
| 10 | `P · PLACES · 10 OF 12 FILLED`; `3 SELECTED` in accent | `… 7 OF 7 SHOWN · 0 FILLED`; `0 selected` in `text_dim` |
| 11 | Batch verbs folded into the grid header band as quiet text | Five filled slabs on their own line |
| 12 | Tile = one bordered box: 76 px art band, hairline, `code · name` caption **inside** | Caption outside the tile |
| 13 | `×N` badge, `☑` selection mark, `empty` on the art band | None; variant count appended to the caption |
| 14 | Visible checkerboard on empties | `SlotCell` used `sunken`/`panel_alt`, one level apart — invisible |
| 15 | Selected tile: 1 px accent border + 35 %-accent outline, offset 1 | A 2 px accent rect |
| 16 | `grid-template-columns:repeat(6,1fr)` | Fixed-width cells, unfilled margin, horizontal scrollbar |
| 17 | `P01 · CAPITAL` band, 150 px preview on a 12 px checkerboard, `name · W × H · …` line, Scale slider, Fit/Reset/Replace…/+Variant, 20 px swatches with a selected marker, anchor segment, tag chips with `＋`, VARIANTS strip, PACK METADATA block, equal-flex Validate/Clear | A stack of `_kv_row` label/value pairs; 18 px unmarked swatches |
| 18 | (implementation) | The inspector rebuilt every child on every selection change — hence the `has_focus()` guard the pack fields needed |
| 19 | Slicer: 760 px card — title bar with `✕`, preview column, 274 px settings column ending in a summary and Cancel / Slice | One vertical stack of stock widgets, clipping its labels (`Trim transparent edg`, `Background → transpa`) |

**Rebuilt** in `shell/asset_library_window.gd` from the canvas: borderless,
sized under the app menu bar; one chip / segment / well / text-button
vocabulary; rail 266, inspector 330, bands 28, tile art 76, variants 56,
swatches 20, slicer 760·274·296 — every number off the canvas, every colour a
`DccTheme` token, no hex. The inspector is built once and refreshed in place.
The slicer keeps its engine-computed overlay (now dashed at 35 % accent — stroke
only, span arithmetic untouched) and the `cd29266` close-with-parent fix.
**Every live binding stayed on the control it was already on.**

**Not regressed — reshaped around.** AS-16 (eight families, not 24) keeps the
rail grammar listing the real eight, disclosure moved to the FAMILIES band's
tooltip. AS-15 (family-level anchor) draws the three-way segment, lights the
real one and disables the others with the reason. AS-14 (render-time weighted
variants) gets the VARIANTS strip, which selects what the *preview* shows and
says so. The then read-only per-item transform got the Scale/Fit/Reset row,
disabled with its reason; Replace… and ＋ Variant became **real**
(`as_import_item` + `as_remove_item`).

**Verified by looking.** Non-headless `opengl3_angle` boot, a real 512×512
world, `reference_pack.zip` loaded, 12 items imported (one slot with three
variants), four screenshot/compare iterations, pixel probes confirming rail =
266 px and the exact `accent_wash` blend, the slicer re-run on a 6×4 sheet
(`24 cells detected · 24 non-empty`), both close paths including Escape via
`Input.parse_input_event`, and `--headless --path . --quit-after 120` clean.
`cartalith-native/docs/CHANGELOG.md` carries the five Godot layout traps this
pass surfaced.

**Updates, 2026-08-23 (AS-07/AS-12/AS-17):** Collections, in-app drag-and-drop
onto a Collections row, the slicer's pan/zoom/click-to-select/Margin handle,
per-item transform editing (`as_set_item_transform`/`as_reset_item_transform`),
"Unassigned imports", per-interior-line dragging (`SliceGrid::with_lines`/
`move_line`) and cell-scoped slicing (`as_slice_apply`'s `only_cell`) all
closed — the full accounts are those rows in §6.3. **Still open then:** dragging
a file from *outside* Godot onto a slot (OS drops reach `Window.files_dropped`,
never a Control's `_can_drop_data`/`_drop_data`, so a slot cannot structurally
be that drop target); moving an assigned item *into* Unassigned imports; and
per-item pan as two SpinBoxes rather than the reference's drag-on-canvas.

### 14.7 · The Data manager window was passed too leniently — corrected and rebuilt (2026-08-20)

The Data manager had §14.6's history exactly — written from
`DCC_SHELL_SPEC.md` §9's **prose** before its export bindings had a caller, and
passed on content rather than layout.

**The 20 deltas**, the canvas's `Data manager window 1920` screen against the
sweep's `09_data_manager_window.png`:

| # | Canvas | What shipped |
|---|---|---|
| 1 | Full-bleed workspace window under the app menu bar, 34 px window bar, 26 px status line | Floating 920×600 `AcceptDialog`, OS title bar, stock **OK** as the only footer |
| 2 | Window bar: `⧉ DATA MANAGER` accent · subtitle · spacer · `Close ✕` outline chip | Title in the OS bar; subtitle a lone mono label in the body; no Close chip |
| 3 | Rail 252 px, opening with a 28 px `ROUTES` band | 260 px, no band |
| 4 | Plain tracked `IMPORT` / `EXPORT` / `SOURCES` / `VALIDATION` headers (`padding:9px 14px 4px`) | `DccWidgets.section()`'s `§ IMPORT` sigil headers — the dock L3 grammar on a window rail |
| 5 | Route rows `padding:5px 14px 5px 24px`, 11.5 px, one line | Flat 26 px `Button`s, no indent, `AUTOWRAP_WORD_SMART`, two lines |
| 6 | Quiet right-hand badge (`tiles`, `→ Assets`, `.zip`) | Qualifier concatenated into the label (`Assets (routes to the Assets menu)`) |
| 7 | Selected row: `rgba(224,163,74,.09)` ground, brightened name, accent `▸` | Font colour only |
| 8 | Import ▸ **Maps** and Import ▸ **GIS / GeoJSON** are two rows | One `Maps (tiles) · GIS / GeoJSON` row |
| 9 | Rail footer: `exports → …` / `last run 14:02 · 62 MB`, two mono lines under a hairline | A `§ EXPORTS ROOT` header, a wrapped path, a third line |
| 10 | 28 px pane header band: breadcrumb left, `web-map ready · XYZ scheme` right | A 12 px-padded breadcrumb, no band, ground or descriptor |
| 11 | Pane body `grid-template-columns:1fr 1fr; gap:0 34px` | Single column |
| 12 | Seven columns — TILES / PROJECTION / LAYERS INCLUDED / OUTPUT / ESTIMATE / MARKDOWN VAULT / RECENT RUNS | One grey paragraph and at most one button — **DM-13** |
| 13 | Row grammar `120px label · control`, `padding:4px 0` | None |
| 14 | Segments (`3px 9px`, one lit), wells (`4px 9px`, mono), `☑`/`☐` rows with a right-hand note | None existed in this file |
| 15 | ESTIMATE: bordered block, four `space-between` rows | Absent |
| 16 | MARKDOWN VAULT: accent-bordered block, `●` status, prose, three checks, three equal-flex buttons | Absent — **DM-14** |
| 17 | RECENT RUNS: three `space-between` mono rows | Absent |
| 18 | Pane footer: `writes to …` · `Save as preset` · `Dry run` · accent `Export 3 619 tiles` | Absent |
| 19 | Status line: `idle · no pass running` · vault state · `Esc close window` | Absent |
| 20 | *(divergence, not a delta)* The canvas still carries a **CONVERSION** group | Correctly absent — deleted by owner decision `17ccc18`; the canvas predates it |

**Rebuilt** in `shell/data_manager_window.gd`: borderless, under the app menu
bar, rail 252, bands 28, status 26, label column 120, pane padding 18, column
gap 34 — canvas numbers, `DccTheme` tokens, no hex. The chip / segment / well /
text-button / band vocabulary **moved from `asset_library_window.gd` into
`dcc_widgets.gd`**, as that file's note asked (*"if a second window needs them,
they move"*); its eight private statics remain as one-line delegators, so none
of its 74 call sites moved.

**Export ▸ Maps wired — DM-02 partially closed, RD-09 closed.**
`region_export_tiles` was bound and golden-tested with **no caller**; this pane
is the caller, exporting the live Region-select marquee as a zipped
`cols × rows` grid. Verified through the real button handler: a 5.17 MB archive
of **33 entries** (16 × `tiles/refined_{r}_{c}_rg16.bin`, 16 × `.png`, plus
`tiles/index.json`). `right_dock.gd`'s *Send to Data ▸ Export* — disabled since
written, tooltip *"the Data Manager panel to call it doesn't exist yet"* — now
opens this pane.

**Drawn and disabled with its reason** rather than omitted or faked: XYZ / TMS /
WMTS addressing (the export is a flat row/col grid plus an index, not a
slippy-map pyramid), every CRS and the world file, `folder` and `MBTiles`
packaging, `leaflet-preview.html` and `style.json`, skip-all-ocean-tiles,
political tint / labels / rivers as export layers, and Save as preset. The
MARKDOWN VAULT block is drawn **quiet rather than accent-bordered** — the
canvas's vault is linked and this one cannot be (DM-14).

**Two canvas inventions measured instead.** ESTIMATE's `~ 214 MB` /
`~ 3 min 40 s` are a size *model* this port lacks, so **Dry run** performs the
whole export without writing and reports real bytes and elapsed time; the block
reads `measured by Dry run` until then. RECENT RUNS and the footer's `last run`
are session-scoped and say so (DM-12).

**Three Godot traps, all of which had shipped:**

1. **`AcceptDialog` enables `wrap_controls` in its constructor**, so the window
   grows to its contents' minimum on every `child_controls_changed()` and never
   shrinks: popped at 997 px, grown to **2032 px inside a 1031 px viewport**,
   footer and status line unreachable below the edge. `wrap_controls = false`
   on both full-bleed windows. This was **a live regression in the shipped Asset
   library too** (status line off the bottom) — fixed in the same commit,
   confirmed by screenshot.
2. **§14.6's autowrap-Label trap, again**: the rail footer's autowrap labels had
   no minimum *width*, so reported an enormous minimum *height* — feeding trap 1.
3. **`theme/dark_theme.tres` gives `ScrollContainer/styles/panel` the
   `SB_FieldDisabled` box** — an input-well stylebox (`content_margin_left/right = 10`,
   a border, **4 px corner radius**) on a container that draws no chrome
   on either canvas screen, insetting every scrolled region in the shell 10 px
   against its header band. Overridden per scroll region here; **the theme
   itself was left untouched, shell-wide** — a global fix belongs in its own
   pass with its own visual check.

**Verified by looking.** Native GL (no ANGLE needed; `6a97911`'s launcher fix
holds), a real 2048×1311 world, a 1024×590-cell marquee via `region_set`, five
screenshot/compare iterations, a real export re-opened with `ZIPReader` (33
entries, `tiles/index.json` present), a dry run of 5.2 MB / 0.58 s, Escape via
`Input.parse_input_event`, the Asset library re-shot for the `wrap_controls`
fix, and `--headless --path . --quit-after 120` clean.

**Still open on this window then:** every Sources and Validation route,
GIS/GeoJSON both ways, the save writer, and DM-02's pyramid half (zoom-level
addressing, CRS, retina variants, ocean-tile skipping) — all disclosed in place.

---

## 15 · The phone overflow menu is wired but inoperable (2026-08-20) — **RESOLVED 2026-08-23**

> **Resolved 2026-08-23**, verified on the OnePlus 6T rather than in an editor
> preview. The awaited design arrived as `design/Cartalith Android
> Phone.dc.html`, implemented as
> `godot-project/shell/phone_menu.gd` plus an L1
> bottom bar in `DccShell._build_phone_menu_bar()`. All four faults are closed
> (**§15.1**; device evidence **§15.2**). The recommendation below held: the
> fix re-presents `menus.gd` and **reimplements none of it** — every row is
> read off the real `PopupMenu` objects and every tap goes back out through
> `id_pressed`/`index_pressed`, so an item added to `menus.gd` still appears on
> the phone with no change.

Classification at the time: **(C)** — a real, connected affordance with no phone
design behind its presentation. Recorded as the brief for the mobile menu design
the owner was having produced; deliberately not fixed then, because a build
would have been discarded when the design landed.

Owner report, from the OnePlus 6T: *"not much from the menus work on android."*

### What is real

`DccShell._build_phone_overflow()` reparents the **actual desktop menu bar**
into the phone sheet, so all seven program menus — File, Edit, Assets, Data,
Preferences, Window, Help — and their ~41 items and 15 submenus are present and
connected, keeping `DCC_SHELL_SPEC.md` §13's promise that `⋯` carries "the full
menu bar". **The routing is worth keeping; only the presentation is missing.**

### What is broken, with device evidence

| # | Fault | Evidence |
|---|---|---|
| 1 | Nothing in the menu path is phone-scaled. `add_menu()` uses `DccTheme.inset(11, 9, 11, 9)` and `FS_MENU` (12 px), raw desktop values, no `_pscale`/`_ptap`. | The row renders ~12 physical px tall — about 1 mm at 314 dpi, against §13's 44 px floor. |
| 2 | Desktop status chrome is reparented with the menus: the `CARTALITH` wordmark (150 px min) and the five `world/res/cpu/gpu/mem` readouts with 22 px gaps. | Pre-generation those labels are empty, so most of the 220-px sheet is blank and the menu row is squeezed into a bottom strip. |
| 3 | The menus do not respond to touch. | Tapping `File` at its centre produced no popup and no pressed state; holding the touch (`adb shell input motionevent DOWN`, captured while held) produced neither. Not conclusively separated from a miss, given (1) — but the observable result is an inert menu. |
| 4 | 15 `add_submenu_*` calls assume hover-to-open, which touch lacks; a nested `PopupMenu` positioned for a pointer has nowhere sane to go at 1080 px wide. | Even with (1) and (3) fixed, ~41 items behind 15 hover-opened submenus is not a phone menu. |

### The brief this implies

Keep the seven menus and ~41 destinations but re-present them as a
**full-screen, touch-sized, drill-down list** — one level per screen, 44 px
minimum rows, back-navigation instead of hover — inheriting `menus.gd`'s wiring
unchanged. The wordmark and readout cluster belong in the app bar or status
sheet, not the menu surface.

The same root cause (desktop contents placed into phone chrome unscaled) was
fixed this pass for the *tool* sheet (`ANDROID_BUILD_SCOPE.md`, fourth device
pass §4), at `set_tool_options()`, deliberately **not** reaching the overflow
sheet so as not to pre-empt this design.

### 15.1 · How each fault was closed (2026-08-23)

| # | Fault | Disposition |
|---|---|---|
| 1 | Nothing phone-scaled | Closed. `phone_menu.gd` sizes everything through `_ps()`/`_pt()` over `DccShell.phone_scale()` — the phone chrome's existing helpers, not a second set of numbers. Rows `_pt(52)`, icon buttons `_pt(44)`, floored at `DccTheme.PHONE_TAP_MIN`. On device: rows **~129 physical px ≈ 66 dp**, clearing the canvas's 44 dp and Android's 48 dp. |
| 2 | Desktop status chrome reparented into the sheet | Closed. The desktop menu and status bars are not drawn on the phone; they are parked in a hidden `PhoneMenuModel` host and used only as the **model** — `menu_bar_row` for the seven `MenuButton`s, `_status_labels` for readouts, which return as 52 dp rows under a `Status` band via the new `DccShell.status_slot_text()`. The 150 px wordmark is gone. |
| 3 | Menus do not respond to touch | Closed. Rows are `PanelContainer`s with their own `gui_input`, handling `InputEventScreenTouch` and `InputEventMouseButton` alike, with a pressed wash. Driven with real `adb shell input tap`/`swipe` at every level — §15.2. |
| 4 | ~41 items behind 15 hover-opened submenus | Closed. No `PopupMenu` is ever popped on the phone. The tree is the canvas's five levels: L1 bottom bar, L2 root list, L3 one menu's items, L4 a 60 %-height sheet, L5 a full screen. Drilling replaces rather than stacks; system back pops one level (`quit_on_go_back = false` plus `DccShell._notification()`). |

**One honest shortfall, unchanged from the design:** the canvas draws titled L3
bands ("§ HYDRAULIC PASSES"), but every `add_separator()` in `menus.gd` is
unlabelled, so a band draws as a hairline-and-gap. Giving a separator text makes
it a caption with no `phone_menu.gd` change; until then it is a rule, not a
heading, and that is stated rather than faked.

### 15.2 · Device verification (OnePlus 6T, 1080×2340, 2026-08-23)

Real `adb shell input tap`/`swipe`, read back with `adb exec-out screencap` —
not an editor preview, since a previous "buttons too small" fix was once wrongly
marked done on editor-only evidence.

- **Portrait composition seen for the first time** — `STATUS.md` had it "still
  unseen" because `"sensor"` orientation defeats `adb`'s rotation override.
- **L1** — `WORLD · CIVIL · CARTO · PANELS · MENU`, active cell in accent.
- **L2** — all seven menus under `Project` / `Content` / `System` bands with
  live counts (File 11, Edit 10, Assets 9, Data 7, Preferences 19, Window 9,
  Help 5) and the `Status` rows.
- **L3** — `Preferences`, breadcrumb `Menu · L3`; disabled rows draw their
  reason as a wrapped second line — *more* legible than desktop's hover-only
  text.
- **L4** — `Devices` as a sheet over dimmed L3 (`Menu · Preferences · L4`),
  enumerating real hardware live: `Adreno (TM) 630 · integrated · vulkan` — so
  `about_to_popup` fires and self-rebuilding submenus are live here too.
- **L5** — `Assets ▸ Asset pack ▸ Edit` as a full screen,
  `Menu · Assets · Asset pack · L5`.
- **Back** — system back popped sheet → screen → root without exiting (pid
  unchanged), matching the canvas's BACK rule.
- **Both palettes** — `Preferences ▸ Theme ▸ Light` from the L4 sheet repainted
  sheet, scrim, the screen behind, toggle and radio; verified on hardware.
- **`Window ▸ Domain rail`** off hides only the three domain cells; `PANELS` and
  `MENU` remain, so the rail can be restored from the same menu (all five
  returned).

### 15.3 · A 29.6-second freeze this work uncovered, and fixed

Making `Preferences ▸ Theme` finger-reachable exposed a pre-existing defect in
`DccShell.rebuild_theme()` no desktop run had shown. **Every
`set_color()`/`set_stylebox()` on a live `Theme` emits `changed`, which
re-propagates `NOTIFICATION_THEME_CHANGED` to every `Control`**, and
`_recolor_project_theme()` wrote once per remapped entry — a whole-tree relayout
per edit. Measured on device:

```
projectTheme=27336ms  windowChrome=1597ms  subtree=670ms  total=29603ms
```

The giveaway is `windowChrome`: **5** writes, 1597 ms — ~320 ms *per write*, so
the cost is per mutation, not per colour examined. Memoising the lookups was
therefore wasted (27336 → 27632 ms) and removed. Batching behind
`set_block_signals(true)` and firing `changed` **once** (`_bulk_theme_edit()`):

```
projectTheme=274ms    windowChrome=417ms   subtree=685ms   total=1376ms
```

— 27.3 s → 274 ms for the dominant phase, **29.6 s → 1.4 s** overall. Method note: the freeze presented as a **dead
tap**, and was twice misdiagnosed as a lost touch ("the tap did nothing"; "the
second tap worked" — the *first* was still finishing). Only log timestamps
either side of the emit showed it. A screenshot 3 s after a tap is not evidence
the tap was lost.

## 16 · The top-left global tool overlay has no drawn presentation in the DCC canvas (2026-08-23) — **RESOLVED 2026-08-24**

Owner-reported, verified by search: the top-left tool set (Measure, Region
select — `global_tools.gd`, `tool_overlay.gd`, `DCC_SHELL_SPEC.md` §4.5.1) is
live in every domain but drawn nowhere in `design/Cartalith DCC Shell.dc.html`
— its only "measure"/"paint" hits are incidental prose ("44 px minimum,
**measured** inside the safe area", "**repaint** 180 ms"); `design/Cartalith
GUI.dc.html` has neither. Distinct from §13.5(d)'s *Measurement tool* line,
which concerned an older superseded canvas (`Cartalith Menu Structure.dc.html`)
under-counting a shipped feature; this was the *current* canvas never drawing
the overlay — a canvas gap, not a shell gap.

**Owner instruction at the time: do not touch
`global_tools.gd`/`tool_overlay.gd` to "fix" this** — the owner was designing
the presentation. Classified **(D)**, a deliberate hold: no agent was to reduce
or reshape Measure/Region select to chase parity with a canvas that had not
drawn them.

### RESOLVED 2026-08-24 — the design landed, and it was two canvases

The hold is lifted. `design/Cartalith Paint Toolbar.dc.html` and
`design/Cartalith Measurement Toolbar.dc.html` (vendored in `e7c10ab`) are one
design: the Paint canvas is the *unifying* bar — **one bar, three mode buttons
(Sculpt · Paint · Measure) on the left, the active mode's tools beside them, an
options bar below** — and the Measurement canvas is its Measure mode in detail,
plus a cross-section strip and a right-dock readout block.

**What landed.** `tool_bar.gd` (the unified bar, a two-row `VBoxContainer`
inside the shell's single `tool_options_row` — no new shell region),
`section_strip.gd` (the cross-section strip, a `viewport_content` overlay in
`resource_overlay.gd`'s mould), `measure_bridge.rs` (engine half), six Measure
modes in `global_tools.gd`, and six presentations of one `CTX_MEASURE` context in
`right_dock.gd`. Sculpt and Paint are **re-presented, not reimplemented** — each
control writes through the same `bridge.sculpt_*` / `bridge.paint_*` call
`world_workspace.gd`'s left-dock panels use.

**Every recorded interaction decision survives** — Measure has no commit; Escape
clears the chain but leaves Measure armed (§4.5.6's exception); Region select is
not that exception and still disarms to Inspect; leaving either tool clears its
draft while Region's rect survives in the engine for *Send to Data ▸ Export*.

**New gaps this design opened, registered rather than silently skipped:**

| Tag | Control | State |
|---|---|---|
| **MEA-01** | Sculpt ▸ **Flatten** and **Noise** | **(B)** — the canvas draws seven sculpt tools; `FreehandMode` has eight and neither of these is among them. The row is built live from `get_sculpt_freehand_modes()`, so it shows the eight that exist. Two new kernels, not a wiring gap. |
| **MEA-02** | Paint ▸ **Water** and **Lithology** layers | **(B)** — `PaintTarget` is Biome/Terrain/Splat; neither has an override array for a dab to write into. A `cartalith-spatial::paint` change plus a staleness edge, not a control. |
| **MEA-03** | Sculpt/Paint ▸ **Mask** | **(C)** — no mask channel in either editor, none designed. Disclosed in the bar's note. |
| **MEA-04** | Distance ▸ **path ▸ great circle** | **(D)** — the map is equirectangular and `cartalith_spatial::measure` is planar with a seam rule; a spherical path would disagree with every route length beside it. |
| **MEA-05** | Distance ▸ **snap ▸ settlements · rivers** | **(C)** — `DCC_SHELL_SPEC.md` §4.5.1 deliberately gives Measure no snap (Way/Route have one; the ruler is raw). The canvas adds it. |
| **MEA-06** | **units ▸ km/mi** | **(A)** — the canvas says this *inherits* the app-wide switch (`#calUnitSeg`); the reference has one (`_setUnits`, line 13722, "switch km/mi and re-render all unit-bearing labels"). The row's original reason — "this shell has no unit preference at all, so every reading in the app is km" — **is false as of 2026-09-07's re-count**: `Preferences ▸ Units` is a three-way radio (`menus.gd`'s `_units_popup`: `DccUnits.label("km"/"mi"/"nmi")`), persisted through `DccSettings.units_mode()`, with `dcc_units.gd` as the display-layer conversion. **CLOSED.** **What is open is narrower and not this row**: `OUTSTANDING_WORK.md` measured 77 raw-km display lines across 14 shell files that never reach `DccUnits`, the journey planner's 24 among them. |
| **MEA-07** | **Saved measurements** list, **Save**, **CSV**, **export PNG**, **save section** | **Built 2026-09-03**, except the two image exports. The store is a caller-owned save **slot** — `annotations/measurements.json` in `cartalith_io::DOCUMENT_SLOTS`, carried by the same `project_save_with_documents` channel as the other five documents (owner ruling, `LARGE_ITEM_RULINGS.md`: *"deliberately not a second persistence mechanism"*) — which answered the row's old objection that "a store is a persistence feature, not a measuring one". `Save measurement`, the list (per-entry recall and drop), `Clear all` and `Copy saved as CSV` are live in `right_dock.gd`; the CSV is canonical km/km²/m/deg whatever the Units preference. An entry carries mode, clicked grid points and reading; the store clears when the world is replaced, and a document from another grid is refused. **export PNG and save section remain (C)** — image writers, unrelated to the store. |
| **MEA-08** | Cross-section ▸ draggable **A/B line-end handles** | **(C)** — a third click starts a new section instead. Same two clicks, no new on-canvas hit test. |
| **MEA-09** | Cross-section ▸ **Custom ▾** field | **(C)** — no user-defined field to bind. The other five channels (Elevation · Terrain · Climate · Hydrology · Geology) are live. |
| **MEA-10** | Area ▸ **rectangle** / **freehand** ring modes, and **⌥ subtract a hole** | **(C)** — polygon only. A rectangle is four clicks; a hole needs a second ring and a signed-area subtraction the readout has no place for. |
| **MEA-11** | Vertical tools **disabled in 2D** | **(D)** — the canvas gates Δ vertical and 3D distance on a 3D relief view; this port reads the same height field in both, so they stay live in 2D and the dock says why. |
| **MEA-12** | Crossings by **river name** | **(D)** — no river entity crosses the GDExtension boundary (the River-context line, `right_dock.gd`). Crossings are described by Strahler order instead of an invented toponym. |

**Ridge crossings needed a definition and had none.** The canvas prints "ridge
crossings 2"; nothing — canvas, spec or reference — says what one is.
`measure_bridge::RIDGE_PROMINENCE_M` is this port's answer, stated in code and
tooltip: a local maximum at least 100 m above the lower of its two flanking
valleys. Without a prominence rule every ripple in a 1 024-sample profile counts.

---

## 17 · Debug-view gaps were never registered here — seven now closed (2026-08-23)

`PARITY_AUDIT.md` §5 item 8 caught the hole: the eleven honestly-unavailable
rows in `sample_bridge.rs`'s `GAP_LAYERS` were disclosed **in code only** (each
`LAYER_GROUPS` row's hint string, shown in the Layers popover), so a reader
walking `reference/FUNCTION_INDEX.md` Part 0's Layers list against this file
found nothing. The audit spotted it via Wildlife; it applied to all eleven.

This section gives them ids. **Seven were closed, not merely registered**, all on
2026-08-23 from `PARITY_AUDIT.md` §3.1 — fjord, landform and windthrow
(DV-01/02/03), then geoid, tides, Köppen and wildlife (DV-06/07/04/11).
`GAP_LAYERS` went from eleven ids to **four**: two lacking a *computation*
(`oro`, `velo`) and two a *composite* (`popdensity`, `siteprofile`).

### 17.1 · The register the debug views never had

| # | Reference view | `LAYER_GROUPS` id | Reference functions | State |
|---|---|---|---|---|
| **DV-01** | **Fjord mask** | `fjord` (Hydrology) | `buildFjordMask`/`carveFjords`/`currentFjordMask`/`carveFjordsOp`, HTML 3208-3249 | **done, 2026-08-23.** `cartalith_terrain::fjord`; view live; `#fjordBtn` live as *Carve fjords* in `world_workspace.gd`'s Glacial group. `golden_parity_fjord.rs`, 6 tests, bit-exact |
| **DV-02** | **Landforms** | `landform` (Surface) | `buildLandformField`/`currentLandform`, HTML 8082-8107 | **done, 2026-08-23.** `cartalith_terrain::landform`; legend is the reference's `LANDFORM_COLS`. `golden_parity_landform.rs`, 6 tests, bit-exact |
| **DV-03** | **Wind-throw** | `windthrow` (Civilization) | `buildWindThrowField`/`currentWindThrowField`, HTML 5602-5636 | **done, 2026-08-23.** `cartalith_climate::windthrow`; gated on the civilisation layer's water bodies (the biome raster is a real input, so a loaded save still reports unavailable). `golden_parity_windthrow.rs`, 4 tests, bit-exact |
| **DV-04** | **Köppen climate** | `koppen` (Climate) | `computeSeasons`/`classifyKoppen`/`buildKoppen`/`koppenColor` (7) | **done, 2026-08-23.** `cartalith_climate::koppen`; five-class legend off the frozen `KOPPEN_KEYS`. `golden_parity_koppen.rs`, 6 tests, classifier bit-exact against the reference's captured seasonal fields. Picking it runs temperature and weather twice more (one solstice each) — the reference's own lazy-build cost, and the slowest view in the popover |
| **DV-05** | **Orogeny** | `oro` (Tectonics) | the signed orogeny preview | open — needs the boundary-polyline structure `generate_terrain` folds into height and never retains |
| **DV-06** | **Geoid** | `geoid` (Tectonics) | `buildGeoid`/`refreshGeoid`/`geoAt`/`currentGeoidPreview` (4) | **done, 2026-08-23.** `cartalith_climate::geoid`; diverging ramp. `golden_parity_geoid.rs`, 7 tests, bit-exact. Previewed at the reference's `0.015` default amplitude, the state the reference previews in (its toggle defaults off). **WW-07's *parameters* stay open** |
| **DV-07** | **Tides** | `tides` (Tectonics) | `tidalForcing`/`computeTideField`/`buildTideField`/`refreshTides`/`currentTideField` (5) | **done, 2026-08-23.** `cartalith_climate::tides`; water only. `golden_parity_tides.rs`, 6 tests, bit-exact — including the Green's-law cap and the geoid-on path. Previewed with the default single moon, the substitution `currentTideField` makes while the toggle is off. **WW-07's *parameters* stay open** |
| **DV-08** | **Velocity** | `velo` (Hydrology) | the Mei virtual-pipe velocity-erosion pass | open — WW-02 |
| **DV-09** | **Pop density** | `popdensity` (Civilization) | the regional persons/km² estimator | open |
| **DV-10** | **Site profile** | `siteprofile` (Civilization) | the flood + slope buildability composite | open — both inputs exist individually |
| **DV-11** | **Wildlife** | `wildlife` (Civilization) | `buildTRI`/`guildTrophic`/`buildEcoregions`/`assignWildlife`/`regionRichness`/`wildRegionColor`/`currentWildlife` | **done, 2026-08-23.** `cartalith_civ::wildlife`; gated on water bodies as `windthrow` is. `golden_parity_wildlife.rs`, 8 tests, bit-exact. `buildNPP` was already ported and is **consumed**, not re-implemented. Its roster click popup is **WL-01** below |

`layer_available` gained two per-world cases rather than unconditional
availabilities: `windthrow` and `wildlife` join `bclass`/`cterrain` on "needs the
civilisation layer" (both read the Cartalith biome grid, which needs water
bodies), so a loaded save reports them unavailable instead of drawing an empty
raster.

### 17.1a · WL-01 — the wildlife roster click popup

`PARITY_AUDIT.md` §5 item 8 listed this as a class-(d) row: a *reference
interaction* with no disclosure anywhere, because the register tracked only the
debug **layer**, never the click behaviour on it.

| # | Reference surface | Reference functions | State |
|---|---|---|---|
| **WL-01** | **Wildlife ecoregion roster popup** | `showWildInfo`/`hideWildInfo`/`wildFmtPop` (HTML 8257-8276), and the map-click branch at 9785-9791 | **done, 2026-08-23** |

Three deliberate choices:

- **The hit test is the reference's own.** `WorldGen::wildlife_region_at` takes
  the nearest region marker within `max(8, GW/40)` cells, skipping regions below
  `markerMin` — line for line with HTML 9787-9789. Outside that it returns an
  empty dictionary and the dock falls back to the sample context, as
  `hideWildInfo()` does.
- **The popup is a RIGHT-dock context, not a floating panel.** The reference
  positions a `#wildInfo` div at the cursor; this shell routes every "you
  clicked something" readout through the dock's context switch. Every field
  `showWildInfo` renders is present, in its order.
- **`wildFmtPop` stays engine-side.** `wild_fmt_pop` is exported from
  `cartalith-civ` and the dock prints its string, so the `~4.5M` wording has one
  implementation.

### 17.2 · Two divergences from the reference, disclosed

1. **No hillshade under the class rasters.** The reference draws `landform` and
   `fjord` over `shadeFactor(x,y)` (HTML 8486-8488). This port's debug rasters
   are flat-coloured across the board (`lith`, `soil`, `btype` …) and
   `sample_bridge.rs` computes no hillshade; the module's convention beat
   matching the reference on two rows and leaving sixteen inconsistent. The
   fjord view's *non-fjord land* tone uses the reference's `v*235*s` with `s`
   fixed at 0.47, the midpoint of `shadeFactor`'s range.
2. **`carve_fjords()` does not re-run flow, rivers or climate.** The reference's
   `carveFjordsOp` follows the carve with `enforceRiverChannels()`,
   `computeFlow(true)` and `refreshClimate()`; this port has no re-runnable path
   for those — the gap `sculpt_commit` documents, for the same reason. Height
   and its direct derivatives are correct after the call; flow, Strahler and
   climate rasters are as they were. Stated in the `#[func]`'s doc comment and
   the button's note.

---

## 18 · The civ-interaction surface: place editing, the context menu, the Delete key and the faction roster (2026-08-23)

`PARITY_AUDIT.md` §5 called items 2 and 3 "the substantive ones" and item 3 "a
live usability hole, not just an inventory gap: a user can add a settlement they
can never fix or undo." Eight of its fourteen rows are civ-interaction rows;
this section closes six, upholds one existing decision, and registers the last
as blocked with the real reason.

### 18.1 · What closed

| # | Reference surface | Where it lives now | State |
|---|---|---|---|
| **CX-01** | **Right-click context menu**, `_civCtxShow` (HTML 25857) and the `contextmenu` handler that fills it (25888) | `map_overlay.gd`'s new `map_right_clicked` signal → `viewport_host.gd` re-emit → `app.gd`'s broadcast → `civilization_workspace.gd`'s `on_map_right_clicked` | **done.** Five of six ops: Edit · Move viewer to · Delete · Drop settlement here · Info here. The sixth, Drop POI — 18.2 |
| **CX-02** | **Delete key deletes the selected place** (block 2 keydown, HTML 26096) | `app.gd`'s `_unhandled_key_input`, broadcast to any workspace implementing `on_delete_key()` | **done.** Guarded while a `LineEdit`/`TextEdit`/`SpinBox` has focus; routed through the editor's own Delete confirmation |
| **ED-03** | **Place edit popup**, `placeEditPopup`/`_civPopulatePlaceEditor` (HTML 16694) | `shell/place_editor_window.gd`, over five new `#[func]`s (`civ_settlement_details`, `civ_edit_settlement`, `civ_settlement_toggle_trait`, `civ_reroll_settlement_name`, `civ_delete_settlement`) | **done.** Name (plus the culture-aware re-roll), class, polity, population, economy, the seven traits, age and walls overrides, history, focus camera, delete |
| **CV-07 / MS-13** | **Add / remove faction, persistent faction identity** | `civ_roster_bridge::FactionRoster` on `CivData`; `civ_add_faction`/`civ_remove_faction`/`civ_set_faction_field`/`civ_faction_count` | **done.** `_civAddFaction`/`_civRemoveFaction` ported including revert-to-Unclaimed. `CIV_FACTION_COUNT` now *seeds* the roster instead of *being* it |
| **FR-01** | **Faction Roster modal**, `_civOpenFactionsModal`/`_civRenderFactionList`/`_civPopulateFactionEditor` (HTML 16177/16247) | `shell/faction_roster_window.gd`, from CIVIL ▸ Politics ▸ *Faction roster…* | **done in part** — 18.3 has the two blocks not built and why |
| **FR-02** | **Procedural faction banners**, `_civFactionBannerCanvas` (HTML 14849) | `shell/faction_banner.gd`, a `Control` with a custom `_draw()` | **done.** The reference's composition — shield outline with two quadratic sweeps, faction colour fill, one of six glyphs by `fid % 6` at 85% white — via `Curve2D` with the exact quadratic-to-cubic control offsets |
| **CV-10** | **"Land sustains ≈ N" readout**, `civPopEstimateOut` / `_civAgrarianRegionalTotal` (HTML 23516) | `cartalith_civ::timeline::civ_agrarian_regional_total` plus `civ_agrarian_regional_total()`; CIVIL ▸ Settlements ▸ Roster and the roster window's overview | **done.** No such function existed in `cartalith-civ`; ported with `golden_parity_roster.rs` (Node `vm` over HTML 23516-23528, two cases pinning `cellKm²`) |
| **CV-11** | **Biome carrying-capacity residual**, `civBiomeKChk` / `_biomeK` (HTML 1406 / 6441) | `CivOptions::biome_k`, `set_biome_k_enabled`/`get_biome_k_enabled`; File ▸ New world ▸ Generation | **done.** `build_carrying_capacity` always took the parameter and nothing could turn it on. Default OFF, matching `_biomeK = 0` ("bit-identical"); the wetland mask is built only when on, as `currentCarryingCapacity` does |

Two more `#[func]` families landed so the pickers use the engine's tables rather
than a GDScript transcription: `civ_trait_vocabulary`,
`civ_specialisation_vocabulary`, `civ_religion_vocabulary`,
`civ_government_vocabulary`, `civ_ag_tech_vocabulary`, `civ_culture_vocabulary`
(from the new `cartalith_civ::roster`, ported verbatim), and
`civ_faction_terrain_fits`, which gives `civ_culture_terrain_fit` — labelled
"**Not wired to any caller yet**" in its own doc — a real caller.

`get_factions()` grew `name`, `religion`, `government`, `ag_tech` and
`population`, and `culture` changed from a recomputed `civ_default_culture(f)`
to a read of roster state. Its doc used to assert "the reference has no faction
*name* registry beyond this"; it does, and the comment records the correction.

### 18.2 · CV-01 (POI) — the decision was checked, and upheld

`civ_tools_bridge.rs`'s module doc says POI "is not a ported concept", and CV-01
records the tool as *omitted rather than built inert*. Re-read before this pass
and confirmed as a real fact of the port: `cartalith-civ/src/tools.rs` ports
Settlement and Territory only, `civ_place_pick_weight`'s doc says the POI branch
"is likewise absent because this port has no POI concept," and no POI record
type exists to attach a drop to. **Nothing here reverses it**: the context menu
ships five ops with "Drop POI here" *absent* (as `_build_tools()` treats the POI
tool); the place editor has no **Category** selector (one option in a
one-category port); `civ_settlement_details`/`civ_edit_settlement` are
settlement-only, as the `#[godot_api]` block's header says. Wanting POI is still
CV-01's estimate — one `civ_drop_poi` mirroring `civ_drop_settlement` plus a real
record type — and an owner call.

**Re-checked live, 2026-08-24** (the manual-authoring audit that closed IN-09):
unchanged. The tool options bar tells the user *"Settlement, Territory, Way and
Route tools are armed from the TOOLS block in the dock. POI has no engine call
(`civ_tools_bridge.rs`) and is not offered."* — omitted and *said*. One nuance,
so it is not mistaken for a POI feature later: the **Icon** tool's vocabulary
includes `"poi"` (`ManualIconFamily::Poi`, a yellow diamond in `map_overlay.gd`),
so a user can place a marker that *looks like* a POI — with no record, name,
faction or inspector.

### 18.3 · What is registered open, with the real reason

| # | Surface | Why not built |
|---|---|---|
| **FR-03** | The Faction Inspector's **Power breakdown** (military / economic / political / cultural / religious) and its **Economy** block (food production and surplus, tax income, trade income, primary exports and imports, strategic resources, craft share) | Both read `_civFactionAggregates`' resource- and density-fed half. `civ_faction_aggregates` **is** ported and called for real (`civ_faction_terrain_fits`) — with `resources: None, density: None`, enough for the terrain-mix half. Filling them means retaining the 15 resource rasters and a population-density field past `compute_civilisation`, which `MEMORY_OPTIMIZATION_SCOPE.md` deliberately paid to avoid. A memory decision plus an `ECONOMY_SCOPE.md` milestone, not a widget. *Stale, re-checked 2026-09-24:* `WorldGen::civ_faction_economy` (`45b368d`) feeds both halves without retaining anything. It rebuilds the resource fields on demand and reads the retained `CivData::dens`. It returns food capacity and surplus, all fifteen resource means, strategic resources, and exports and imports, and CIVIL ▸ Economy draws them (`civilization_workspace.gd::_fill_faction_economy`). The reason above no longer holds. The Faction Inspector's own Power breakdown and Economy block were not re-checked field by field |
| **FR-04** | **Diplomatic relations** | No model in either codebase. The reference's inspector renders "Diplomatic relations — not yet implemented"; so does this |
| **CV-12** | **Placement-diagnostics overlay**, `civDiagnosticsChk` (HTML 1415, `drawCivLayer` §2.6 at 15617) | **Blocked on urban morphology, not UI.** Every line of its fact card is `_um*` data — `_umWallSpec`'s wall ladder, `_umSiteProfile`'s river classification and coast distance, a peek into `_umModelCache` for bridge/ford/harbour validity — inside a `SITE_WM × SITE_HM` box. `cartalith-urban` milestones 8-17 were unported, so there was nothing to draw. Registered as a **disabled control carrying that reason** in CIVIL ▸ Settlements ▸ Not built. *Stale, re-checked 2026-09-24:* since 2026-09-02 (`0bba2f9`), `urban_bridge.rs::settlement_diagnostics` exposes the fact card, and `civilization_workspace.gd::_build_settlement_diagnostics` lists it per settlement in the dock. It is **not drawn on the map**: the reference's on-canvas footprint box is the part still missing |
| **ED-03a** | An edited **specialisation** does not reach `civ_faction_aggregates`' sector output | `FactionPlace::specialisation` is read by that function and every caller passes `None`. Feeding edits in would change already-golden economy numbers on an interactive edit — a decision, not a wiring detail. Stated in `civ_roster_bridge`'s module doc, the editor's Economy tooltip, and here. **SG-02 does not close this** (checked 2026-08-24): `recompute_civilisation` rebuilds `trade_balances` via `civ_resource_trade_balance` over the settlement's catchment; `civ_faction_aggregates` is not on the path |
| **ED-03b** | The **age** and **walls** overrides are stored and consumed by nothing | Only readers are `_umInferAge`/`_umInferWalls`/`_umWallSpec`. Same block as CV-12. *Stale, re-checked 2026-09-24:* both are consumed. `urban_adapter::PlaceOverrides::{walls_override, age_override}` reach `settlement_layout_with` through `urban_bridge.rs`, and `civ_military_bridge.rs` reads them for defences (see `civ_roster_bridge.rs`'s module doc). An edit changes the generated town |
| **ED-03c** | The seven **traits** are stored and never drawn on the map | The reference draws them as glyphs beside the marker; `map_overlay.gd` has no per-trait glyph pass, and that file was deliberately minimal-touch this pass |
| **UM-03** | The layout thumbnail (`peCityPreview`) and its City Viewer launcher (`peCityOpen`) inside the place popup | UM-03 called this "doubly blocked: no place-edit popup exists at all (ED-03) and no city layout to preview even if it did." **Half of that is now false** — the popup exists. The other half stands. *Narrowed, 2026-09-24:* a layout **can** be drawn at icon size. The right dock's settlement context draws an 84 × 84 plan through `urban_layout_draw.gd`'s `draw_layout()`, which is the same renderer as the map and the City Viewer. What is missing is only the thumbnail inside `place_editor_window.gd`, whose own tooltip and footer still give the old reason |
| **ED-03d** | A place edit or delete does **not** recompute provinces, trade balances, roads, territory or `explanations` | **CLOSED 2026-08-24 (SG-02, §21).** `recompute_civilisation()` rebuilds all five; the Settlements ▸ Recompute button calls it. Verified: a hand-dropped capital moved territory, roads, provinces *and* trade balances. An explicit button, not a per-edit cascade — 4.22 s at 2048². The disclosures (`civ_delete_settlement`'s doc, the delete-confirmation dialog, `_settlement_click`'s status hint) now name that button |
| **CV-13** | A faction added after generation owns nothing until something is assigned to it | Not a gap — the reference's `_civAddFaction` behaves identically (appends to `CIV_FACTIONS`, touches nothing placed). `assign_factions` runs inside `generate()` at `CIV_FACTION_COUNT`; the status hint after Add says so |

### 18.4 · Verification

- `cargo test -p cartalith-civ` — green, including the new
  `golden_parity_roster.rs` (3 tests: 13 golden `_civFactionColor` values across
  every hue sector, two `_civAgrarianRegionalTotal` cases, a land-gate negative
  control).
- `cargo test -p cartalith-godot --lib` — 263 passed, including
  `civ_roster_bridge`'s 10 new unit tests (roster seed / add / remove floor,
  revert-to-Unclaimed, vocabulary rejection, trait toggle order, the age/walls
  clamps, delete-by-index).
- Headless boot (`--headless --path godot-project --quit`) clean.
- Interactive verification is recorded in this section's entry in
  `cartalith-native/docs/CHANGELOG.md`.

---

## 19 · The manual erosion passes: kernels ported, wired as generation parameters (2026-08-23)

`PARITY_AUDIT.md` §3.1's row read *"kernels partly absent, no run-button
path"*. Both halves closed in this pass: the kernels are bit-exact ports, and
the run path is **generation-time parameters, every one off by default**, under
`DECISIONS.md` §7d. WW-02, MS-04 and MS-05 point here.

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
`GENERATION_PARAMETERS.md` itemised for these rows (3, 4 and 4);
`hillslope_diffuse` takes its 2 as arguments.

### 19.2 · The decision, and which way it went

**The reference runs none of these inside `generate()`, and says so** —
`evolveCoupled`: *"A new op (never auto-runs) → generate() bit-identical at
defaults"*; `glacialKernel`: *"Manual Glacial erosion button + its worker path
only — not part of default generate()"*. Each is a button that mutates the
finished field and then re-derives flow and climate (`erodeFinish` /
`eroFinish` / `veloFinish`). Two shapes follow, and they are not equivalent:

- **(a) Opt-in run buttons**, the reference's shape — not new architecture
  here: `WorldGen::carve_fjords()` (`#fjordBtn`) and
  `WorldGen::center_landmasses()` (`#centerBtn`) are live post-generation ops,
  and the fjord one already sits in `world_workspace.gd::_build_erosion_passes`
  beside the five disabled placeholders (*"it never runs during generate, so a
  default world is unchanged by this control existing."*). Distinct from
  **WW-11** (per-stage `Run stage n`), which is (D) — in neither engine.
- **(b) Generation-time parameters**, default-off. Permitted by §7d, but it
  requires choosing *where in `generate_terrain`* to insert a pass the
  reference never inserts — a pipeline-order decision with no reference answer
  and so no golden fixture for the composed result.

**(b) was taken**; every toggle is off, so the default reproduces the
reference — verified as an assertion (19.4). The passes run **at the very end
of `generate_terrain`, after `carve_rivers`**, because each reference button
operates on the finished field, in the reference's own panel order —
`velocity → glacial → coastal → hillslope → evolve → sediment_fill`. There is
still **no golden fixture for the composed result**: each kernel is bit-exact
alone, the sequence is this port's choice, and `ErosionPassParams`' doc
comment says which is which.

(a) is **not** foreclosed and is now cheap — a `#[func]` over the same code. It
was not built because UI work was on hold (`CLAUDE.md`) and the parameter path
needs no UI to be real.

### 19.3 · What each row got

- **WW-02** — `cartalith_engine::ErosionPassParams` on `WorldParams`: six
  toggles (`velocity`, `glacial`, `coastal`, `hillslope`, `sediment_fill`, and
  `evolve_cycles` as a count where `0` is off; a seventh, `tidal_flats`,
  joined on 2026-08-24, see 19.5) plus fifteen knobs, each defaulting to the
  reference's `state` literal. **21 rows in `params.rs`**, in the `erosion`
  group; each knob names its reference slider and carries that slider's
  reachable range through its `eparam` mapping. The toggle rows have an empty
  `reference_control`, because the reference's control is a *button* — the
  toggle is the §7d addition. **Droplet stays open**: its kernel exists since
  Phase 1, but its `erodeFinish` tail (thermal + clamp + rebound) is a second
  orchestration outside this pass.
- **MS-05** (`#sedimentBtn`) — `depositSediment`'s orchestration, transcribed:
  stream-power carve → per-cell eroded-column supply → `compute_flow` on the
  carved surface → `route_sediment`. **`#tidalFlatsBtn` closed 2026-08-24** —
  19.5.
- **MS-04** (`#evolveBtn`) — the missing function was written:
  **`cartalith_engine::refresh_climate`**, `pub`, the
  `computeFlow(true); refreshClimate();` tail (reference line 5154). Evolve
  calls it once per cycle — the rain driving the next cycle's incision must
  reflect the orography the last one built — and the pass block once at the end.

**One deliberate deviation, disclosed.** The pass block ends with
`erodeFinish`'s `if(f<0)f=0; else if(f>1)f=1;` clamp (reference line 3894),
which the reference applies only after the *droplet* pass. Found by a test:
`velocity_erode_kernel` has only a ±1e9 finite guard and `route_sediment` adds
without an upper bound, so both can leave a cell outside 0..1 — a transient in
the reference, but baked into a `WorldState` whose 0..1 range the renderer,
downstream stages and `generate_terrain`'s end-to-end test assume here. Applied
once after the last pass, so no pass reads a value the reference would have
left unclamped.

### 19.4 · Verification

- `cargo test -p cartalith-erosion -p cartalith-hydrology -p cartalith-engine`
  green, including `golden_parity_passes.rs` (26 tests, `assert_eq!` on `f32`,
  no tolerance, bit-exact on the first run);
  `cargo test -p cartalith-godot --test params_mapping` passes the 21 new rows
  unmodified.
  `cargo clippy -p cartalith-erosion -p cartalith-engine --all-targets` clean; `cargo build -p cartalith-godot` refreshed
  `target/debug/cartalith_godot.dll`.
- Two `cartalith-engine` tests for the *wiring*, which kernel goldens cannot
  reach. `erosion_passes_off_leave_generation_bit_identical` moves five knobs
  with every toggle off and `assert_eq!`s field, temperature, rainfall **and**
  discharge — a knob alone must do nothing. `each_erosion_pass_changes_the_field_on_its_own`
  runs each of the six alone and asserts the surface moved, stayed finite and
  in 0..1; its glacial case drops the snowline *and* freezes the world (ice
  needs both) and compares against its own climate-only twin.
- **Mutation sweep: 115 literal sites, 98 killed**, after four fixture passes
  shaped to reach what the first missed. The 17 survivors are explained in
  `passes.rs`' module header — reference dead branches, thresholds redundant
  with a pinned constant, razor-edge windows. One is a real finding:
  `applyTidalSedimentation`'s `tr <= 1e-5` floor is **unreachable**, because
  the `sea - 1e-4 - h` headroom cap already excludes any cell it could gate.
- **Non-headless, in the real app** (`_erosion_shot.gd`, untracked, in the
  `_npr_shot.gd` mould): a 512×384 world at seed 483920, through the real
  `EngineBridge` with `reset_params()` → `param_set()` → a full re-generate
  per case. Share of pixels moved by more than 3 levels: velocity 38.2 %,
  glacial 91.3 %, coastal 6.4 %, hillslope 44.5 %, sediment fill 43.7 %,
  evolve 44.0 %. **All-off returns to the base map at 0.0000 %.** Looked at,
  not only measured: hillslope visibly rounded the ridges; velocity reworked
  the drainage and crenulated the coast.
- The control: **`glacial` with the snowline dropped but the world temperate
  moves 0.24 %**, because ice also needs `temp < 0` — a wire-up ignoring the
  temperature gate would have scored like the frozen case.

### 19.5 · The seventh pass: tidal flats (2026-08-24)

`#tidalFlatsBtn` was left open in 19.3 because `apply_tidal_sedimentation`'s
input field had no producer — *a toggle over an always-absent field cannot
work*. `cartalith_climate::tides` (WW-07's engine half, same day) is that
producer. Shapes checked: `compute_tide_field` returns a `Vec<f32>` of `gw*gh`
spring tidal ranges with land exactly `0`, and `apply_tidal_sedimentation`
takes `&[f32]` of `w*h` and skips `tr <= 1e-5`, which land satisfies.

**`passes.tidal_flats` + `passes.tidal_k`** (the reference's `0.45`) are the
seventh toggle and knob, same `erosion` group, same empty `reference_control`.
It runs **last** — the reference's source order (`applyTidalSedimentation`
follows `depositSediment`) and the physical one: mudflats accrete onto the
coastline the passes above finished shaping.

**Unlike the other six**, the reference gates this button on `tideField`, which
exists only while `state.planet.tides.enabled` is on — two switches where this
port has one. This toggle is both: it builds the tide field from the finished
surface just before the kernel reads it, as `refreshTides()` does there.
`PlanetParams` carries no moon roster, so the field uses
`TideParams::default()`'s single Earth–Moon-equivalent companion at this
world's `planet.g` — the substitution the Tides debug view (DV-07) documents. A
partial answer to WW-07's parameter question: **the consumer's toggle is the
enable**. The geoid half has no consumer and stays open.

**Verification.**

- `cargo test -p cartalith-engine -p cartalith-erosion -p cartalith-hydrology
  -p cartalith-godot` green; `each_erosion_pass_changes_the_field_on_its_own`
  runs seven cases and `erosion_passes_off_leave_generation_bit_identical`
  moves `tidal_k` too.
- `the_tidal_flats_pass_only_raises_submerged_cells_toward_sea_level` asserts
  the *shape* — every changed cell was submerged, every change is upward, none
  passes sea level, sea level did not move. A sign error or swapped
  `sea`/`depth` would still pass the "something moved" table.
- **Measured at grid resolution**, 256×192 world, seed 4242: 3,051 cells
  accreted — 6.21 % of the grid, **19.58 % of every water cell** — mean rise
  0.01968 of the 0..1 range, max 0.05129. Water only, upward only.
- **Non-headless**, 19.4's harness: 9.00 % of pixels moved on the 512×384
  world at seed 483920, **all-off 0.0000 %**. The bay shoals (bottom-right bay,
  central sea margins) visibly lightened toward sea level and no coastline
  moved — what accretion capped at `sea - 1e-4` should look like.

## 20 · GeoJSON export: the engine was finished, the boundary was missing (2026-08-24)

DM-03's row estimated *"one `#[func]` plus assembling a `GeoJsonWorld`"*, and
that was exact. The gap looked bigger from outside because
`FUNCTIONAL_CONTRACT.md` read it as **Absent**, when `cartalith_engine::geojson`
had been a complete, golden-verified port of nine reference functions since
milestone E2 — character-for-character, including the hand-written JSON writer
that exists because `serde_json` renders an integral `f64` as `16.0` where
`JSON.stringify` renders `16`. What was absent was a caller — the same shape as
Export ▸ Maps (DM-02): ported, tested, callerless, invisible.

### 20.1 · The binding

`crates/cartalith-godot/src/geojson_bridge.rs` — one `#[godot_api(secondary)]`
block, one `#[func] fn export_geojson(&self) -> GString`, in its own file
because `lib.rs` is a shared hot file. Returns `""` before the first
`generate()`/`load_save()`.

Three reference inputs have no equivalent, each handled by **omission, not
invention**:

- **POIs.** `CIV_POI_KEYS` splits `state.places` into settlements and POIs;
  this port's `SettlementKind` is the six settlement tiers only, so there is no
  `poi` layer — not an empty one, which would claim the world has none.
- **`w.sea`.** The reference keeps land ways and sea lanes in one flat
  `civWays` with a flag; this port has two typed collections
  (`CivData::ways` / `::sea_routes`), so the flag is *derived* from the source
  collection.
- **Rivers.** `_riverNet` is a lazy global cache there; here the receiver tree
  and Strahler orders are on `WorldState`, so polylines are re-traced
  (`trace_river_polylines`, as `urban_bridge` does). The export's `min_order`
  is the reference's **2**, not the carve pipeline's `1`.

**One function had to be ported**, outside the geojson module:
`splitRiverPolylines` (reference 4596-4608), which cuts a chain wherever the
next point is not reachable by a straight stroke — the antimeridian seam, and
optionally an open-water predicate. Without it a wrapped chain exports as one
`LineString` drawn back across the whole map. It went to
`cartalith_hydrology::split_river_polylines`, beside the tracer, with two
goldens whose fixtures were produced by **running the reference's own function
under node**. The export passes no skip predicate, matching the reference's
comment: a lake reach is real hydrology; only the seam jump is cut.

### 20.2 · The route

`data_manager_window.gd`'s Export ▸ GIS / GeoJSON row flips from `gap` to
`live`: a pane, two disclosures (the CRS note — the document's own `note`
property verbatim — and the civ-layer note), and one `Export .geojson…` chip
over a save picker. No options pane, because the binding has none —
`export_geojson` always emits every layer it can. It exports
the **whole world, not the Region-select marquee**, which is Export ▸ Maps'
input; the pane says so. CRS handling still exists nowhere in the workspace,
disclosed in the pane and the document.

### 20.3 · Verification

- `cargo test -p cartalith-engine -p cartalith-erosion -p cartalith-hydrology
  -p cartalith-godot` green, including the two
  `split_river_polylines_matches_the_reference_*` goldens; `cargo build -p cartalith-godot` refreshed the
  cdylib.
- **Non-headless, through the real `EngineBridge`**: a 512×384 world at seed
  483920 exported **305,646 bytes in 21 ms**; valid JSON; **511 features — 239
  settlement, 43 way, 216 river, 6 territory, 7 province**, cross-checked
  against the bridge's getters in the same run (239 settlements, 43 roads, 0
  sea routes — so the `sea:true` branch is the one path unverified in the real
  app). Every coordinate inside the 1200 × 900 km box; numbers render the JS
  way (`"mapWidthKm":1200`, not `1200.0`); river `strahlerOrder` runs 2 · 3 · 4,
  confirming min-order 2.

## 21 · The staleness consumer is wired; its UI deliberately is not (2026-08-24)

`GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` §3.2.4 found the staleness graph
(`cartalith_engine::staleness::pipeline_stage_graph`) correct, tested and
**consumed by nothing**, so every post-generation edit stopped at the height
field. The owner authorised the engine half, now built and wired into the
commit paths:

| Path | Marks | Runs |
|---|---|---|
| `WorldGen::sculpt_commit` | `Height`, at the pass's own tiles | hydrology + climate, one `refresh_climate` |
| `WorldGen::carve_fjords` | `Height`, whole map | hydrology + climate |
| `WorldGen::paint_commit` | `Civ`, at the painted tiles | nothing — a mid-chain edit does not make its own upstreams stale |
| `WorldGen::recompute_stale_stages()` (new `#[func]`) | nothing | whatever is already stale |

All four return `recomputed` and `still_stale` as `PackedStringArray`s.
Measured `--release`: **76.5 ms @512², 97.8 ms @1024², 188.9 ms @2048²** —
18.8× cheaper than the full generation it replaces (`cartalith-native/docs/CHANGELOG.md`,
"The staleness graph gets its consumer").

**No GUI was built, on purpose** (the `CLAUDE.md` UI hold); what the engine
half unblocks was registered so it would be designed, not improvised. *(All
three were then designed and built to owner brief the same day — SG-02, then
SG-01 and SG-03; the notes under the register are those designs.)*

### 21.1 · The register

| Tag | What | Backed by | State | Why it is not built (or, for a closed row, where its design is) |
|---|---|---|---|---|
| **SG-01** | A **staleness indicator** — the DCC mockup's *"downstream update: rivers · deferred"* status line: which stages are stale and why | `WorldGen::stale_stages()` is the read; the shell's reserved `stale` status slot is the surface, plus a per-stage badge in the Civilization dock's Recompute section | **CLOSED 2026-08-24** | Note below — where it lives, and the one source the graph cannot carry |
| **SG-02** | A **"Recompute now"** control for the stages a commit leaves stale — today always `civ` | `recompute_stale_stages()`; `recompute_civilisation()` is the civ half; `civilization_workspace.gd`'s Settlements ▸ **Recompute** section is the control | **CLOSED 2026-08-24** | Note below — what is re-derived, preserved and deliberately not, and the measured cost |
| **SG-03** | **`param_set` marking the graph** — a moved dial invalidating the stage it affects, instead of `engine_bridge.gd`'s blanket *"a moved dial does not recompute a stage, it marks the world stale until the next full generate"* | `params::invalidates()` is the table; `set_params`/`reset_params` mark from it | **CLOSED 2026-08-24** | Note below — the table, its derivation rule, and the finding that the shipped World dock supersedes every mark it makes |

### SG-01, closed 2026-08-24 — where the indicator lives, and the one source the graph cannot carry

`WorldGen::stale_stages()` returns `{stage: {origin, reason, tiles}}`, `{}`
when healthy. A pure query — every `StageGraph` accessor takes `&self` — so it
can be polled without triggering work.

**Two surfaces, neither new chrome.** The shell's `stale` status slot, reserved
since `_build_status_bar()` was written (with a `stale` colour token and an unused `DccWidgets.stale_mark()`) held the
last generation's *duration*, which moved into `pass` ("generated · 3.2s"); it
now reads `stale: climate · civ — sculpt`. The second is a badge above the
Civilization dock's Recompute button ("Stale over 12 tiles — sculpt. Recompute
to catch it up." / "Up to date — nothing has changed under it since the last
recompute."). Both poll on a 1 s `Timer`: staleness comes from half a dozen
`#[func]`s across three workspaces, and six signal couplings for a plain query
is the wrong trade.

**The button still does not grey itself out.** It stays enabled because a
recompute is also how a user re-derives roads and borders after an edit the
engine cannot classify, and the badge already tells them in advance whether it
will do anything.

**What the graph structurally cannot represent.** A hand-dropped, edited or
deleted settlement makes roads, territory, provinces and trade balances stale —
but those are `civ`'s *own* outputs, and `civ` is the leaf, so
`mark_changed(Civ)` marks nothing (`staleness.rs`'s
`a_downstream_only_edit_recomputes_nothing_upstream_of_it`), and marking an
upstream node would be a lie that drags a pointless `refresh_climate` along.
So `WorldGen::civ_dirty` is a plain flag set by exactly those three `#[func]`s,
cleared by `recompute_civilisation` and `absorb`, reported as
`origin: "settlements"`, `reason: "place_edited"`, `tiles: 0` — otherwise the
indicator would read "up to date" right after the edit `ED-03d`'s button exists
for. A flag, not a mark, because this is not a dependency: it is one pass of a
stage behind another pass of the same stage.

### SG-03, closed 2026-08-24 — the per-parameter → stage table

`params::invalidates(key) -> Option<PipelineStage>`, consulted by `set_params`
per applied key and by `reset_params` wholesale. **25 of the 81 parameters mark
something; 56 mark nothing**, and that split is the design.

**The rule is derived, not judged:** a parameter belongs in the table only if
some function *other than* `generate_terrain` reads it — marking a stage stale
promises that recomputing it applies the new value, and nothing re-runs
terrain. Two functions qualify:

| Live consumer | Parameters | Marks | Effect |
|---|---|---|---|
| `refresh_climate` (all of what `recompute_stale` runs) | every `climate.*` row — the **climate** and **weather** groups, 20 keys — plus `peak_m`, `planet.g`, `planet.rotation_hours`, `planet.axial_tilt_deg` (24) | `Hydrology` | climate *and* civ go stale; `recompute_stale`'s gate fires and one `refresh_climate` runs |
| `compute_civilisation` via `recompute_civilisation` | `river_density` (1), through `fresh_river_order` → affordances, roads, territory | `Climate` | **only** civ goes stale; `recompute_stale` runs nothing, leaving `still_stale = ["civ"]` for the dock's button |
| — | the other 56: every tectonic, volcanic, crater, stream, erosion-pass and world-structure knob, plus `carve_rivers`, `use_gpu`, `sea_level` and `world` | nothing | generation-time only; the honest control is Generate |

`sea_level` and `world` *are* read by `climate_params_for`/`weather_params_for`
and still mark nothing: `recompute_stale` is handed `WorldState::sea_level` (a
World-Structure archetype re-anchors it during generation), and
`WorldGen::recompute_params` pins `world` to the value `absorb` snapshotted, so
a moved geometry switch cannot make a recompute describe a different world.

**Why the marked node is one *above* the stage that goes stale.**
`mark_changed(S)` means *S's output changed* — S's consumers go stale, S stays
current; there is no "own inputs moved" state. So the node marked is the one
immediately upstream of the shallowest invalidated stage. For climate that is
`Hydrology`, honest for the weather knobs (`refresh_climate` first recomputes
`flow_discharge` from the new rainfall) and one node coarser than the truth for
the temperature-only dials (`lapse_rate`, `albedo_k`), where discharge does not
move. Exactness would need a fifth, `params` source node — a change to the
pinned four-node graph, not taken.

**The drift guard is mechanical.**
`params_mapping.rs`'s `every_key_that_moves_refresh_climate_is_marked_and_no_other`
moves each of the 81 rows to the far end of its range, re-runs
`refresh_climate` over a fixed height field, and asserts "the output moved" and
"`invalidates()` returns `Hydrology`" agree. Its baseline turns `wind_manual` on
and widens the latitude band, because otherwise `wind_dir_deg` and `albedo_k`
are provably inert — true of the *default world*, false of the parameter.

**Marking only; nothing is recomputed in the setter** — `world_workspace.gd`
writes a slider on every drag tick, which would run `refresh_climate` sixty
times a second.

**Finding that bounds SG-03's value:** no shipped GDScript path leaves one of
these marks standing. Every parameter row in `world_workspace.gd` calls
`_regenerate_live()` on release (the reference's `tparam()` `change`
behaviour, verified in 2026-08-19's Playwright pass), and a full `generate()`
rebuilds the graph in `absorb()`; `reset_params()` has no shell caller. So the
table is a correct boundary contract and a prerequisite, not a user-visible
change. Its consumer would be an "apply climate dials without regenerating"
path, which cannot simply be switched on: a full regenerate with a new
`rain_k` produces different *terrain* (weather runs inside the carve and the
`evolve_cycles` loop). That is a parity decision, not wiring.

### SG-02, closed 2026-08-24 — what "recompute civilisation" was decided to mean

Engine half `WorldGen::recompute_civilisation()`; the control is a
**Recompute** section in the Civilization dock's Settlements category
(`civilization_workspace.gd`), not a menu item, because every readout it fixes
lives in that dock. The question was **how much** to rebuild, and the answer is
deliberately not "all":

- **Re-derived** — everything downstream of the settlement list, against the
  *current* terrain: water bodies, biome, lithology and soil, resource
  potentials, the hierarchical road topology and consolidated ways, sea lanes,
  territory, provinces, per-settlement trade balances, the suitability
  `explanations` (correctly re-indexed) and agrarian density.
- **Preserved** — the settlement list and everything keyed to it:
  hand-dropped places (`civ_drop_settlement`), hand-edited names, tiers,
  populations and factions (`civ_edit_settlement`), the `tid`-keyed
  `place_extras` side table (traits, specialisation, history, age/walls
  overrides), the faction roster, the recorded timeline and year, and
  hand-painted territory, which `CivTools::rebase` re-anchors onto the new
  borders. `CivTools::commit` could not: it is driven by the draft and returns
  early when it is empty, which it always is at recompute time.
- **Not done** — settlement *placement*. Re-running
  `find_settlement_seeds`/`place_settlements` would move every settlement,
  re-roll every name and drop every hand-authored place and `tid`-keyed entry;
  re-placing from terrain already has a control, Generate. Metropolis
  promotion, village seeding and the recovery phase are skipped for the same
  reason — each *authors* settlements.

So: sculpt a mountain under a city and the recompute reroutes its roads and
redraws its borders, but the city stays on the mountain.

**Found by the real-shell run: villages are not road-network nodes.** The
reference seeds them *after* `_civHierarchicalNetwork`, so a village-enabled
world has no roads to its villages. Feeding the whole kept list back into the
topology, as the first implementation did, took a 384 × 288 world from **35
ways to 240 on one press** and tripled the cost (4.3 s → 0.7 s once fixed).
`CivData::village_tids` now records which settlements `civ_seed_villages`
added — keyed by `tid`, since neither an index nor a trailing range survives
`civ_delete_settlement`'s splice or `civ_drop_settlement`'s append — and the
recompute builds the network from the non-village settlements and remaps the
endpoints. After the fix: 35 ways before, 35 after, rerouted around the new
mountain.

**Measured** (release, CPU, square grids at 1200 km): **0.94 s at 512², 1.60 s
at 1024², 4.22 s at 2048²** — about half a full `generate()` of the same world
in the same run (1.28 s / 2.59 s / 8.16 s), and below `UNIFIED_TOOL_PLAN.md`
milestone C's ~7 s/stroke because placement and naming are skipped. No fast
path: a second call on unchanged input costs the same, within a few ms.

**Not registered, because it is not a gap:** the carve-time river network
(`channels`, `stream_order`, `river_mask`) staying as it was after an edit.
`refresh_climate` re-derives drainage, not the vector network, and neither does
the reference's post-edit tail (`computeFlow(true); refreshClimate();`).
Re-extracting rivers after a stroke would be new behaviour with no reference
precedent.

---

## 22 · The phone pass on the civ / urban / render windows (2026-08-24)

Four subsystems landed on desktop and tablet with no phone pass: the
civ-interaction popups, the City Viewer, the unified Sculpt/Paint/Measure bar,
and the NPR Painter block. Driving them on a OnePlus 6T found one structural
fault under all of them and four smaller ones (full reasoning in that date's
`CHANGELOG.md` entry). A second live audit the same day added `PH-07`-`PH-10`,
all invisible to headless checks and to the desktop build: three a phone-only
*scaling* rule that silently did not apply, one a phone-only layout never
written.

### PH-01 · The phone chrome swallowed every tap on the map — **fixed**

`_phone_content_gap` was `MOUSE_FILTER_PASS` and the two containers above it
`STOP` by default, all full-screen. A `PASS` control is still picked and
forwards to its **parent**, not to what is behind it, so `map_overlay.gd`'s
`_gui_input()` had never run on a phone. Dead by touch: tap-to-select a
settlement and every tool click/drag/release handler — Settlement, Territory,
Way, Route, Measure, Sculpt and Paint dabs. Masked because camera pan and pinch
come through `ViewportHost._input()`, which never consults a `mouse_filter`.

The class: **a full-screen layout container in the phone chrome must be
`MOUSE_FILTER_IGNORE`.** Its children are picked independently.

### PH-02 · The map context menu had no touch route — **fixed**

Press-and-hold (500 ms, under 28 px drift) now emits the same
`map_right_clicked`, and `civilization_workspace.gd`'s menu is re-presented as
the phone canvas's L4 sheet rather than a pointer-sized `PopupMenu` that clips
at the screen edge. `map_overlay.gd`'s withheld-press rule stops the hold also
firing the armed tool.

### PH-03 · `wrap_controls` on three more windows — **fixed**

`place_editor_window.gd`, `faction_roster_window.gd` and
`city_viewer_window.gd` shipped with `AcceptDialog`'s default left on — the
third to fifth instances of §14's bug class (the roster even carried a
`max_size` whose comment described the symptom). Now off in the shared
`DccWidgets.phone_window()`, on every platform.

### PH-04 · Desktop-pixel touch targets in every dock and window — **fixed**

`dcc_widgets.gd` authors every shell row in desktop pixels (`_row` 24, `slider`
14, `action` 26, `tool_button` 30x30). `DccShell.phone_fit()` —
`_phone_fit_tool_options()` generalised — floors them at §13's 44 dp across the
dock sheets and the three windows. Four non-obvious cases, detailed in
`CHANGELOG.md`: `OptionButton.fit_to_longest_item`, `clip_text` on expanding
buttons only, `PopupMenu` row height, and `MOUSE_FILTER_PASS` on layout
containers.

### PH-05 · A dock sheet does not scroll from a drag on its content — **fixed 2026-08-24**

Scrollbar and accordion worked, but a flick on the rows — the gesture a phone
user reaches for first — did nothing.

**PH-04's `MOUSE_FILTER_PASS` on the rows was a no-op.** Measured against 4.7.1:
`Container` already *defaults* to `MOUSE_FILTER_PASS`, not `Control`'s `STOP`,
so the `HBoxContainer` rows never blocked anything. **A `Button` did.** Godot
delivers a GUI event to the picked control and up its parents, stopping at the
first `STOP`; a `Button` is `STOP`, so a press starting on one never reaches the
`ScrollContainer`, whose drag-to-scroll runs on exactly those (touch-emulated)
events, gated on `DisplayServer.is_touchscreen_available()`.

`_scrolldrag_probe.gd` (beside `_shot_phone.gd`) flicked twenty points down the
open left sheet and named the control under each. Twelve did not scroll: **nine
`Button`s and three `HSlider`s, nothing else.** From the accordion down the
sheet is all buttons — L2 `category()` headers, L4 `group()` headers, every
`action()` — so the lower two thirds were dead to a flick. It also explains
why the place editor "worked": its form is mostly labels and margins, not a
viewport difference.

**Fix, in `DccShell.phone_fit()`:** a `BaseButton` is set to
`MOUSE_FILTER_PASS`, and the `ScrollContainer` gets a `scroll_deadzone` (new
`PHONE_SCROLL_DEADZONE`, 10 authored px, scaled with the subtree).

- `PASS` is safe on a button: past the deadzone the scroll propagates
  `NOTIFICATION_SCROLL_BEGIN` and the button cancels its pending press.
- **The deadzone is load-bearing.** Godot's default `0` counts a real tap's
  ~2 px wobble as a drag and eats the press. Measured: at 0 a 2 px jitter tap
  scrolls 1 px and fires nothing; at 10 a clean tap, a 2 px and a 6 px wobble
  all fire, and an eight-sample flick scrolls 96 px and fires nothing.
- **`HSlider` is excluded**: a drag starting on a slider means "move it".
- **`OptionButton`, `MenuButton` and `ColorPickerButton` are excluded**: a
  control that opens a `Popup` on *press* pops mid-flick and the popup grabs
  the drag (measured: popup open, scroll 0). Their rows scroll from the label.

`browse_dialog.gd`'s file rows carry the rule inline — they are
`PanelContainer`s with their own `gui_input`, which the shared walk
deliberately does not touch.

**Verified on the OnePlus 6T**: a flick starting on *05 Volcanism & impacts*
scrolls the World sheet to its foot and toggles nothing; a tap on *06 Erosion*
opens it.

### PH-06 · The New world dialog and the file browser are not phone-shaped — **fixed 2026-08-24**

Both opened at desktop size in the middle of a 1080 x 2340 screen with 10 px
type. Both now take `DccWidgets.phone_window()`/`phone_present()` plus
`DccShell.phone_fit(self, 1.0)`, the civ windows' treatment — no second set of
phone constants. Three things it did not already cover:

1. **New world needed a way out.** `phone_window()` goes borderless and this
   dialog's OK is *Create*, so on a phone it gained
   `add_cancel_button("Cancel")` and a `DccWidgets.phone_head()`.
   `phone_window()` is called *before* the OK button is renamed, since it sets
   `ok_button_text` for the read-only windows it was written for. The browser
   already draws its own ✕ head and Cancel foot.
2. **The browser is spawned, not owned.** `_spawn()` gets whatever node the
   caller had — usually `DccApp`, but `open_project_dialog.gd` passes `self`, a `Window` that answers
   neither `is_phone()` nor `phone_scale()`. New `_shell_of()` walks up to the
   nearest `DccShell`. As the first *transient* window with this treatment,
   `phone_window()`'s rotation hook is now guarded and self-disconnecting: the
   lambda is created in a `static` function, so nothing auto-disconnects it,
   and a rotation after close would have touched a freed object.
3. **The breadcrumb widened the window off the screen — on Android only.** A
   `Button` reports its text as its minimum width, and Android's home is
   `/data/data/org.cartalith.walkingskeleton/files`: the crumb row's minimum
   measured **715 px inside a 393 dp window**, dragging the list, foot and
   Open button off the right edge. On Windows `C:/Users/Vincent` fits — **the
   desktop run is not evidence for this class of bug.** The crumb row is now a
   horizontal `ScrollContainer` on phone: no minimum on its scroll axis, every
   segment reachable, drag-scrolling for free off PH-05's `PASS`.

**Verified on the OnePlus 6T.** New world: full screen, phone header, 0
controls under 44 dp, form drag-scrolls to its foot, Cancel and Create
reachable. Browser: fits 1080 px wide with ✕, crumbs, path well, list and foot;
crumbs scroll sideways without navigating; a row tap still selects.

### PH-07 · `phone_fit()`'s font walk cannot see a `RichTextLabel` — **fixed 2026-08-24**

The walk rescaled a font only where `has_theme_font_size_override("font_size")`
was true. **A `RichTextLabel` has no such theme item**: its sizes are
`normal_`, `bold_`, `italic_`, `bold_italic_` and `mono_font_size`, so every
`RichTextLabel` in a dock was skipped silently.

The one affected control is load-bearing: the right dock's **"Why here?" causal
chain** (`right_dock.gd`, `normal_font_size` = `FS_SMALL`). On a 1080 x 2340
handset it drew at a flat 11 *physical* px — about a third the height of every
row above it.

`phone_fit()` now asks for the override list per control class and scales every
name set. A `RichTextLabel` overriding *nothing* is scaled too, off the resolved
theme value — pure text with no minimum-size floor (`app.gd`'s credits body is
the other instance).

**The class: a theme override's *name* is per control type, and a walk that
assumes one name silently skips the types that use another.**

### PH-08 · The dock TOOLS block is unlabelled marks on a touch screen — **fixed 2026-08-24**

§4.5's palette is `dcc_widgets.gd`'s `tool_button`: 30 x 30, a 15 px glyph, an
**empty** `normal` stylebox, the name in a tooltip — complete on a pointer,
neither half of it on a handset:

- **No hover, so the name is unreachable.** CIVIL's block is seven such marks
  (Inspect, Measure, Region select, Settlement, Territory, Way, Route), WORLD's
  four and CARTO's five, indistinguishable on screen.
- **PH-04's floor grew the box, not its contents**: 121 physical px on device,
  glyph still 15 — about a millimetre on a 400 ppi panel, left-aligned, no
  border.

`phone_fit()` now finishes a tool button: the glyph is **re-rasterised from the
SVG** at 0.42 of the box (hence `dcc_widgets.gd` stashes the glyph's *name* —
`DccIcons` caches per `name@px` and a 15 px texture cannot grow without
resampling), `normal` gains a border, and TOOLS buttons gain a caption under the
icon.

**Measured before choosing caption over bounded icon-only**: the widest caption
("Region select") asks 112 dp and CIVIL's four-tool row 338 dp of a 386 dp
sheet — fits, just, at today's vocabulary. So `tools_block()` lays rows out in
an `HFlowContainer`: a `BoxContainer` given more minimum width than it has
**overlaps** rather than clipping. This is not the `HFlowContainer` the note
below reverted — that one sat in the tool sheet's *horizontal*
`ScrollContainer`, with unbounded width; a dock sheet scrolls vertically only.

Desktop is a verified no-op: same 30 x 30 buttons, 32 px pitch, 15 px icons, no
caption, in all three domains.

### PH-09 · PAINT ▸ Class collapses to its own arrow in the tool sheet — **fixed 2026-08-24**

PH-04 turned `OptionButton.fit_to_longest_item` **off** on phone, because a
`fit_to_longest_item` control reports its longest item's width and one
287 px vocabulary label was widening a 393 dp window — right in a dock, where the
row is full width and the control expands. The tool-options row is six
non-expanding controls side by side, so `fit_to_longest_item = false` plus
PH-04's `clip_text` left the Class picker with **no content-derived minimum
width**: 35 px on device, showing no selected class.

`phone_fit()` gained a `wide` flag for the one caller whose subtree scrolls
horizontally (`set_tool_options()` sets it); both shrink measures are skipped
there. The picker now reports 230 dp and reads "Coastal Lowland".

**The class: a "fit the screen" rule is wrong inside a container that scrolls on
that axis, and the two must be told apart explicitly.**

### PH-10 · The welcome / open-project dialog was never phone-adapted — **fixed 2026-08-24**

This file wrote the *precedent* PH-06 generalised (fill the screen, let
`content_scale_factor` map the desktop composition onto the 393 dp reference)
and never took the finished treatment. Now on `DccWidgets.phone_window()` /
`phone_present()` plus `DccShell.phone_fit(self, 1.0)`. Beyond that:

1. **The toolbar had to stack.** The chips' minimum is ~230 dp, and an
   over-constrained `BoxContainer` overlaps: the search well drew over `Recent /
   All worlds / Shared`, and the `LineEdit` got the ~110 dp left — the reported
   "Search wo…". Both device symptoms were one fault. `phone_window()` returns a
   boolean for exactly this.
2. **An `AcceptDialog` sizes its content child on resize and nothing else.**
   Hiding the too-wide subtitle changes a minimum, not a size, so the body kept
   the **497 dp** width measured with the subtitle, inside a 393 dp window; the
   search well ran 82 dp off the right edge with the gallery tiles and *Open
   selected*. Found by dumping `get_combined_minimum_size()` down the tree
   against the visible rect; `new_world_dialog.gd` measured 377 dp beside it,
   which ruled out `phone_present()`. `child_controls_changed()` is the engine's
   re-measure, called last in the phone fit and at the end of every
   `_refresh()`, since the gallery rebuilds per keystroke.
3. **The rotation relay is now the shared, self-disconnecting one**; the file's
   own `phone_insets_changed` connection now carries only screen-specific work
   (which head text fits, how many tiles).

### Not registered, because it is not a gap

The unified tool bar builds through `set_tool_options()`, which already runs the
touch fit, and the phone tool sheet already scrolls horizontally — so its
segments are 44 dp and reachable as built. An `HFlowContainer` was tried and
reverted: inside a horizontal `ScrollContainer` it gets unbounded width and
never wraps.

### PH-11 · A dock sheet remembered its scroll position across close/reopen — **fixed 2026-08-24**

Found on device: a dock sheet scrolled down, closed and reopened, opened still
scrolled. Six earlier attempts that session missed it.

**The cause was absence, not a stray override.** `_build_left_dock()` /
`_build_right_dock()` (`dcc_shell.gd`) built their `ScrollContainer` via
`_scroll()` into a bare local, never kept. `_set_sheet_open()` only toggled
`left_dock.visible` / `right_dock.visible`, and the body is built once, unlike
`phone_menu.gd`'s sheet, which rebuilds and zeroes `scroll_vertical` on every
`_render()` (the precedent mirrored). Nothing ever wrote `scroll_vertical` back
to 0.

Fixed with `_left_dock_scroll` / `_right_dock_scroll`, kept where the return was
discarded, and `_reset_dock_scroll()` from `_set_sheet_open()` on every open
(not close — a mid-close sheet cannot show a reset). It writes
`scroll_vertical = 0` twice, immediately and `call_deferred`, since a
`ScrollContainer` that was `visible = false` a moment ago may not have run its sort/clamp pass.

Verified with `_sheetscroll_probe.gd` (`--resolution 393x852 --force-touch`,
mirroring PH-05's `_scrolldrag_probe.gd`): each sheet given a 4000 px filler,
scrolled to the bottom (4287 / 3764 px), closed, reopened — both
`scroll_vertical = 0`. Also on a windowed 393x852 run without the filler,
against the real WORLD content (0 → 287 → close → reopen → 0).

---

## 23 · RF-01 — the CIVIL dock never rebuilt after a world generated (2026-08-24) — **FIXED**

A new class for this register. Every one of the 215 catalogued entries above is
a **capability** gap; here the engine, surface and data were all complete and
simply never connected on the one signal that matters. §6.11/§6.12 could not
catch it: their question is *"is the disclosed reason accurate?"*, and *"No
settlements — generate a world first"* is true of an empty engine. It was just
displayed over a world with 233 settlements in it.

**`RF` = refresh: a wired, finished surface that never rebuilds when the world
it reads changes.** Not (A)/(B)/(C)/(D) — those classify *missing* things. This
is a bug.

### What was wrong

`app.gd:386-400` constructs every workspace once, at launch, before any world
exists (`ws.setup()` → `Workspace.setup()` → `_build()`), so
`CivilizationWorkspace._build()`'s Settlements/Population/Economy/Politics/Culture/Timeline
and `_infra.setup()`'s Roads/Rivers/Ports/Trade/Logistics all rendered their
empty state, correctly. Nothing re-ran them:

- `app.gd`'s `generation_finished` handler (415-426) writes status-bar text only.
- The other subscribers were `world_workspace`, `cartography_workspace`,
  `right_dock`, `viewport_host` — and, inside CIVIL, **Timeline alone** (the old
  `_build_timeline()` connection at line ~980).
- `_rebuild_readouts()` rebuilt only `_settlements_body`, and only on a
  place/roster **edit** (`_on_civ_edited`), never on a generate.

A fresh generate refreshed one of eleven sections. Verified live against 40
settlements, 6 factions and a full road network: Settlements ▸ ROSTER said *"No
settlements — generate a world first"*, Politics ▸ FACTIONS *"No provinces"*,
Roads ▸ NETWORK *"No roads"* — while the map drew all of it. It survived because
any verification that edited a settlement or faction on the way tripped
`_on_civ_edited` → `_rebuild_readouts()`; it is visible only if you generate and
touch nothing.

### The fix

Both files split each data-backed category into `_build_*` (runs once, claims
the body node) and `_fill_*` (re-runnable) — the shape
`_rebuild_timeline`/`_tl_body` already used — and subscribe to
`generation_finished` **and** `world_loaded` (load, revert and reopen had the
same hole).

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

Culture and Rivers are deliberately absent: each writes one fixed note about a
binding that does not exist (CV-02, IN-01), which no world changes.

Two incidental corrections: `_rebuild_readouts()`'s comment claimed Population
and Economy *"read nothing this touches"* — wrong on both (Population sums
`get_settlements()`; SG-02's **Recompute civilisation** routes through
`_on_civ_edited` and rewrites the trade balances Economy reads and the provinces
Politics reads), so both now refresh after a recompute too. And
`_on_world_changed()` resets `_selected_index`, which otherwise indexed into a
replaced settlement list.

### Cost — measured, not assumed

`8e666ac`'s standing rule — eagerly cascading civ recompute is too expensive to
hang off an edit (~7 s/stroke), which is why SG-02 is a button — is about
**recompute**. This is **presentation**. Checked against `lib.rs`:
`get_settlements`, `get_provinces`, `get_trade_balances`, `get_roads`,
`get_sea_routes`, `get_factions` and `route_count` are all
`civ.<field>.iter().map(…).collect()` over stored `Vec`s; the one O(grid) call,
`civ_agrarian_regional_total`, is one linear pass over stored `civ.dens` /
`ws.field`.

**Measured in the real app** (`_civdock_shot.gd`, 384×288, 233 settlements):

```
CIVDOCK REBUILD COST 13.99 ms for all ten sections (presentation only)
```

against a **1 350 ms** generate — roughly **1 %**, once per generate.

### Verification

`_civdock_shot.gd` / `.tscn` (untracked, run **windowed** — a headless boot
proves only that the extension loads, which is precisely what never caught
this):

1. Assert the empty state IS present before generating.
2. Generate, switch to CIVIL, **make no edit**, read the real `Label`/`Button`
   text from the live tree and assert all ten sections show real numbers.
3. Delete a settlement through `place_editor_window.place_deleted` and assert
   the roster follows 233 → 232 (the edit refresh, not regressed).
4. Generate a **second, different** world and re-run all ten, asserting the
   roster no longer shows the first world's count — a rebuild that runs once is
   the same bug with a longer fuse.

`CIVDOCK RESULT PASS`, plus a screenshot with every accordion forced open. One
check was relaxed, not a defect: seed 771155 genuinely has **0 coastal
settlements**, so Ports ▸ Coastal correctly reads *"No coastal settlements in
this world."* — confirmed by counting the `coastal` flag directly.

### The lesson this register should keep

**A finished surface built before its data exists is not verified by looking at
it.** §14's visual sweep and §6.11/§6.12's line-by-line read both passed,
because both looked *after* doing something. The question that finds this class
is *"what re-runs this, and on which signal?"* — worth asking of every panel
built at launch.

## 24 · SB-01 — every `has_method()` guard failed silently, and a 21-commit-stale `.so` proved why (2026-08-24) — **FIXED**

**Class: not a disconnected control — a register's worth of connected controls
reading as disconnected because the engine behind them was old.** This
register's method ("open the surface, see whether it does anything") returns the
wrong answer whenever the native library is behind the shell, and nothing in the
app distinguished the two.

### The finding

`builds/android/Cartalith.apk`'s `lib/arm64-v8a/libcartalith_godot.so` was
sha256-identical to a **2026-08-23 14:34** build; `git log` over
`cartalith-native/crates/` since then: **25 commits**. Live in the tree, dead on
the handset:

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

**Not one logged anything** — clean `logcat`, no crash, no ANR.

### Root cause of the *silence* (the part this register cares about)

`engine_bridge.gd` guards every wrapper with `world_gen.has_method("…")` and
returns a safe default on a miss — correct, and why a shell survives an older
binary. There were **200 such guards**, all silent, so a stale library was
indistinguishable from a feature that is off. The City Viewer is the sharpest
case: its *"no layout"* is a statement about the **world**, when the truth was
about the **binary** — a well-written empty state that answers the wrong
question confidently.

### Fix

All 200 sites now go through `EngineBridge._has()`, which `push_warning()`s on
the first miss of each name:

> Cartalith: the loaded GDExtension has no WorldGen.`<name>`(). Whatever needed
> it is degraded to a safe default. This almost always means the native library
> is older than the shell (a stale libcartalith_godot.so) — rebuild and
> re-export before treating the missing feature as a bug.

Once per name (several wrappers are polled from a redraw); `missing_bindings()`
exposes the set at runtime. `push_warning` rather than `print` because it rides
`_err_print_error`, which this project's `logcat` greps target. Verified
headlessly both ways: **0** warnings against a current library across every
`_ready()` probe plus NPR, factions, undo, debug layers, urban layouts, paint
and routes; **exactly one** for three calls on a name no binary exports.

### Re-verification after the rebuild

Rebuilt, re-exported (APK sha256-checked to carry the new library), installed.
On the handset: **zero** missing-binding warnings across boot and a full
generation; NPR builds and its styles re-render the map; erosion-pass
parameters and annotation/icon bindings live. The handset then dropped off USB,
so roster, City Viewer, paint visibility, save/undo, debug views, GeoJSON
export, ways and civ-recompute remain **unverified on device** — not upgraded on
the strength of a desktop probe.

### What this register should take from it

**Before recording a control as disconnected, establish that the binary behind
it is current**: one `logcat`/console grep for
`Cartalith: the loaded GDExtension has no`, empty, is the precondition for trusting a gap audit.
Twelve entries above would have been filed as regressions on a screenshot. (The
desktop form of the same trap — a probe loading a `.dll` older than the `.rs` —
is `MISTAKES.md`'s *"Grade a Godot probe as evidence for a Rust change"* row.)

---

## 25 · BK-01 — Android's Back button killed the process, unsaved world and all (2026-08-24) — **FIXED**

**Class: not a disconnected control — one connected to the wrong thing, which
destroyed the user's work.** Found on the handset: Back on a generated,
never-saved world ended the process and the world was gone, with no recovery —
autosave only writes beside a project already saved somewhere
(`DccApp._autosave_tick()`). The highest severity in this register so far:
every other entry is a missing capability; this one is present and *harmful*.

### Root cause — two faults, either of which was sufficient

1. **The back chain's terminal step was a bare `get_tree().quit()`.**
   `DccShell._notification()` already answered `NOTIFICATION_WM_GO_BACK_REQUEST`
   by popping a phone-menu level, then a sheet, then an overlay (built with the
   phone menu, §15's resolution) — and once those ran out, quit immediately.
   `SceneTree.quit()` does **not** raise `NOTIFICATION_WM_CLOSE_REQUEST`, so the
   unsaved-changes prompt File ▸ Close project had gained that session was never
   consulted.
2. **`quit_on_go_back = false` was set only when `_phone` was true.**
   `DccShell._compute_layout_mode()` classifies by *aspect ratio*, so every
   Android device read as a tablet — and a phone whose boot window reports
   landscape — kept the SceneTree default, quitting with no code of ours
   running, not even the sheet/menu popping.

Fault 1 lost the tester's world; fault 2 was a wider door nobody had opened yet.

### The navigation model, and why

Back **leaves exactly one level**, innermost first; only the last can end the
app:

| Press lands on | What back does |
|---|---|
| a dialog or popup window | hides it — found anywhere in the tree, since a dialog is parented to whichever `Control` opened it, not to the root |
| a phone-menu level | `PhoneMenu.go_back()` — L5 → L4 → L3 → L2 → closed |
| a drawer, panel picker or dock sheet | `_close_all_phone_overlays()` |
| an armed tool | Escape's own action, then a real disarm |
| nothing, and a world exists | the **same** save/discard/cancel prompt as File ▸ Close project |
| nothing, and no world exists | quits |

Two decisions with defensible alternatives:

- **Prompt, not "press back again to exit."** That pattern suits an app whose
  back stack is one level deep; here back walks four real levels before the
  exit, so a press arriving there is considered — and the phone composition has
  nowhere to draw the toast, the status bar being parked hidden (§15). The
  prompt guards only when there is something to lose; otherwise back exits at
  once, the platform convention.
- **The tool step disarms unconditionally, unlike Escape.**
  `GlobalTools._measure_escape()` clears the chain and leaves Measure *armed*,
  right for a pointer user. Inherited by Back it made the gesture a **permanent
  no-op**: `armed_tool` never reached `inspect`, and the exit was unreachable
  while Measure was armed. Caught by the probe, not by review.

### The prompt is shared, not duplicated

`DccApp.close_project()` and `DccApp._back_exhausted()` both call one
`confirm_unsaved_world()`, keeping the rule that it appears **whenever a world
exists**, not only when `bridge.world_dirty` is set (see that flag's doc
comment for what it cannot see).

### Two measured phone-presentation traps this uncovered

The prompt is all that stands between a back gesture and a destroyed world, so
it must be tappable on the device. Both traps produced a silent 29 dp button
row:

1. **`DccShell.phone_fit()` structurally cannot reach it.** It walks
   `get_children()`, and `AcceptDialog` parents its button bar as an
   **internal** child, so every stock OK/Cancel row in the shell is outside
   every fit. Usually minor; here the three buttons *are* the dialog.
2. **`Window.popup()` clears `custom_minimum_size` on those buttons.** Isolated
   in a two-node scene: the value survives `content_scale_mode`,
   `content_scale_aspect`, `content_scale_factor`, `min_size` and `max_size`,
   and reads `(0, 0)` the instant the window shows. The floor must be applied
   *after* the popup and on every re-popup — which a rotation is, via
   `phone_window()`'s inset relay.

   A smaller third: `b.custom_minimum_size.y = 44` through an **untyped** loop
   element writes to a temporary copy and is lost. Typed `for b: Button` and a
   whole-`Vector2` assignment fix it.

### Verification

`godot-project/_backnav_probe.gd` (committed) drives the real shell with a
**really generated** world and delivers the actual
`NOTIFICATION_WM_GO_BACK_REQUEST`. All checks pass at `393x852` (the canvas
reference), `540x1170` (half the OnePlus 6T: `content_scale_factor` 1.374, the
44 dp floor holding through it) and `1600x1000` (desktop/tablet: File ▸ Close
project regression and fault 2).

**Not verified on the device**: the handset was `offline` to `adb` throughout
(`kill-server`, `start-server`, `reconnect offline` all failed). A desktop probe
cannot prove Android *delivers* the notification; everything downstream, where
the data loss lived, is proven. Delivery was verified on this handset in §15
("System back popped sheet → screen → root without exiting, pid unchanged") —
evidence for the mechanism, not for this change.

### Two related findings, registered and NOT fixed *(BK-02 fixed since — §26)*

- **BK-02 (A) — the desktop close box had no such gate.** Nothing intercepted
  `NOTIFICATION_WM_CLOSE_REQUEST` and `auto_accept_quit` was default, so the
  title bar's × on Windows destroyed an unsaved world exactly as Back did. Left
  alone here because `auto_accept_quit = false` would make the app unquittable
  if the prompt ever failed to appear, and nobody had reported it;
  `close_project()`'s gate was made shared, taking a continuation. **Fixed the
  same day — §26 answers the un-quittable objection.**
- **BK-03 (D) — `KEYCODE_M` does not reach Godot's shortcut path on Android.**
  A non-finding: **`M` is bound to nothing, on any platform.** Every accelerator
  in `menus.gd` carries Ctrl or Shift (`Ctrl+N/O/S/W/Z`, `Shift+A/J/L/D`,
  `Ctrl+Shift+S/P`), and no bare `KEY_<letter>` exists in the shell. Re-test
  only if a bare-letter accelerator is introduced — at which point the phone
  needs a surface for it anyway.

---

## 26 · BK-02 — the desktop close box did the same thing, and the reason it was left alone was answerable (2026-08-24) — **FIXED**

§25 registered this and declined it for a real objection: `auto_accept_quit = false`
"makes the app unquittable if the prompt ever fails to appear, which is a worse
failure than the one it fixes." True of the naive fix; not of one that carries
its own escape hatch, which is what this is.

### The fault

BK-01, one platform over. Nothing handled `NOTIFICATION_WM_CLOSE_REQUEST` and
`SceneTree.auto_accept_quit` was at its default `true`, so the title bar's ×,
Alt+F4 and the taskbar's Close each ended the process outright, taking an
unsaved generated world with no prompt — and autosave only writes beside a
project already saved somewhere, so nothing recovered it.

### The fix

`DccShell._ready()` sets `auto_accept_quit = false` beside BK-01's
`quit_on_go_back` line; `DccShell._notification()` routes the close request to a
new `_close_requested()` hook, which `DccApp` overrides onto **the same
`confirm_unsaved_world()` gate** File ▸ Close project and the back gesture
already share — a third caller, not a third prompt. The gate's only change: it
returns its dialog, so the caller can check it really went up.

Deliberately *not* routed through the back chain: Back means "leave the
innermost thing" (dialogs, menu levels, overlays, armed tools first); × means
"close the application" and goes straight to the exit gate.

Safe wiring: Godot propagates a window's close request **down its own subtree**
(`Window::_propagate_window_notification` stops at nested `Window`s). Every tool
window and dialog is a *child* of the shell, so closing one cannot reach the
shell's handler — the main window is the only source.

### Why the app cannot be left un-closeable

Invariant: **every close request either quits, or leaves a visible prompt on
screen whose three answers all resolve.** `_close_requested()`'s four branches,
in order:

1. **A visible prompt is already up** → re-raise it. Not quit: a double-click of
   × would otherwise destroy the world the first click asked about.
2. **We already asked and nothing is on screen** → quit unconditionally — the
   escape hatch for exactly the failure the deferral named (a script error
   part-way through building the prompt, a window that never shows).
   `_quit_asked` is set *before* the attempt, so it survives one that dies
   halfway.
3. **Nothing to lose** (`not bridge.has_world`) → quit at once.
4. Otherwise prompt, then **verify the dialog is actually visible** and quit
   immediately if not — so even the first × suffices against a prompt that
   fails on first use.

From the other side: Cancel hides the dialog, which frees it, clearing both
flags and re-arming the gate; Discard quits; Save writes and quits through the
same continuation. A *failed* save neither quits nor prompts —
`_write_project()` does not call its continuation on a failed write (correct:
do not exit on a save that did not happen) — and is not a trap, because the
flags are already cleared and the next × prompts again.

### Verification

`_backnav_probe.gd` (BK-01's harness, extended with `_close_box_pass()`,
`_resolve_pass()`, `_await_real_close()`) drives the real shell at `1600x1000`
with a generated, never-saved world. All pass: the real
`NOTIFICATION_WM_CLOSE_REQUEST` raises the shared "Exit Cartalith" gate
(Discard / Cancel / Save), the object the shell tracks; a second request while
it is up neither stacks nor quits; Cancel dismisses, clears and re-arms (a later
× prompts with a fresh dialog); both escape-hatch branches asserted by branch.

Each answer then **pressed for real**, one process per answer
(`-- --resolve=discard|save|cancel`): Discard exited; Save wrote a 420 KB `.zip`
and *then* exited; Cancel left the app running with nothing on screen.

The OS-delivery link a synthesised notification cannot prove is proven here
(unlike BK-01's Android case): `-- --hold` boots the real shell with an unsaved
world, `WM_CLOSE` is posted to its `HWND` from outside (what the title bar's ×
sends), and the process survives with the gate — "This world has unsaved
changes. / Exit the app?" — drawn over a world whose status bar reads *unsaved
changes*. `cargo build -p cartalith-godot` and a headless boot clean.

---

## 27 · IN-10, IN-11, CA-13 — the owner could not reach the Journey Planner, the Route tool, or region naming (2026-08-24) — **FIXED**

Owner, live, using the app:

> There is no way to plan a Journey or draw a route.
>
> It isn't possible to drop a name for a region on the map as in the HTML
> version.

These arrived the day the Journey Planner reached 66/74 reference functions and
the day after IN-09 made a committed route draw. The finding: **none of the three
capabilities was missing; two of the three paths to them were.** Established by
driving the real windowed shell (`_jpprobe_shot.gd`, `_labelprobe_shot.gd`,
`_fixprobe_shot.gd`) — reading the files says all three work, which is what let
this survive.

### What was actually broken

| id | Symptom the owner hit | Cause | Status |
|---|---|---|---|
| **IN-10** | `Data ▸ Journey planner… ⇧J` did **nothing visible** | the takeover only paints while CIVIL is the active domain; the shell opens on WORLD | **FIXED** |
| **IN-11** | every tool's advertised letter — `W`, `⇧R`, `L`, `B`, `S`, `T`, `V`, `M`, `R`, `I` — did nothing | no key was ever bound to any of them, anywhere | **FIXED** |
| **CA-13** | region naming looked absent | it works; nothing in the dock said the word "region" or named the tool | **FIXED (wording)** |

### IN-10 — a menu item that changed not one pixel

`journey_planner_view.gd`'s `_recompute_visibility()` requires
`app.armed_tool == "journey"` **and** `app.active_domain() == "civilization"` —
correctly, since the view swaps the whole CIVIL region (both docks, map, tool
options bar, timeline band). But `open_journey_planner()` only armed the tool.
Of its three entry points, the INFRA dock's Logistics button is unclickable
outside CIVIL; `Data ▸ Journey planner… ⇧J` and the right dock's "Plan a
journey" are reachable from anywhere, and **the shell opens on WORLD**. So the
owner's most likely first action armed a tool, printed
`Journey armed — Esc to release` in ghost text bottom-right, and changed nothing else
(`jpprobe_02_menu_fired_with_world.png`):

```
AFTER MENU (domain=world): armed=journey jp_active=false center.visible=false viewport.visible=true
```

`open_journey_planner()` now calls `select_domain("civilization")` first. The
ordering is benign either way: the switch emits `workspace_changed` and
recomputes with the tool unarmed (a no-op), then `open()` arms and the
recompute paints.

### IN-11 — ten tooltips, zero bindings

Every `tools_block` entry carries its key in the label (`"Way (W)"`,
`"Route (⇧R)"`, `"Label (L)"`, `"Biome paint (B)"`, `"Region select (R)"`, …). A
search of `shell/` found `_unhandled_key_input` matching Escape, Backspace and
Delete, `layers_popover.gd` matching digits, and **nothing matching a letter**.
The tooltip was the whole feature — the fake control this register exists to
catch, in every domain's TOOLS block since it was built, and the other half of
"there is no way to draw a route": the tooltip says press `⇧R`, and nothing
happens.

Fix: a `Shortcut` per button in `DccWidgets._tools_row`, parsed from the label.
A `Shortcut` rather than a key table on `app.gd` for a functional reason:
`BaseButton::shortcut_input` fires only while the button `is_visible_in_tree()`
and is enabled, and only the active domain's panel is visible — so `W` arms Way
exactly when CIVIL is showing, for free. It also runs *after* GUI input, so a
focused `LineEdit` eats its own letters. `shortcut_in_tooltip` is off: the
tooltip already spells the key in the mockup's notation (`⇧R`) and Godot would
append a differently-spelled copy.

### CA-13 — region naming was never missing

The reference calls these **region labels** (`FUNCTION_INDEX.md`:
`_civPopulateLabelEditor` *"Build the region-label editor"*,
`_civRenderLabelList`, `clearLabels` *"Clear region labels"*), and the port has
had the whole thing since the label milestone: CARTO ▸ TOOLS ▸ Label, click
empty ground, a "New label" prompt with placeholder `Region name`, then
`label_create`, drawn with its three resize/rotate/arc handles. Driven end to
end (`_labelprobe_shot.gd`): "Vale of Ashen" rendered with handles, options bar
`CARTO · LABEL editing #0 Vale of Ashen ✓ Confirm`. So **(c) — already possible,
not found**, for two reasons:

1. The dock section was **"Placed labels"**, empty state **"none placed"** —
   neither says *region* nor names the tool.
2. No menu mentions labels or annotation (every popup scanned for
   `label`/`annot`: nothing), so the only path was an unlabelled icon whose
   tooltip shortcut did not work (IN-11).

Fixed in the reference's vocabulary: section **Region labels**, empty state
*"None yet — arm Label (L) in TOOLS above, then click empty ground and type the
region's name. Drag its handles to size, rotate and arc it."* (the shape of
Logistics' *"No committed routes yet — draw one with the Route tool above"*).
With IN-11 fixed, the `(L)` it names is now real.
**Not done, stated**: no menu route to annotation exists; adding one is a
menu-structure change (§13's territory), not a wording fix.

### What was working the whole time, and is proven to still be

From a fresh launch with real synthesised pointer events: CIVIL ▸ TOOLS carries
Settlement, Territory, **Way** and **Route**, enabled; clicking Route arms it
(`armed_tool = route`); two map clicks reach `_route_click` and the options bar
reads `INFRA · ROUTE · 2 stops · ✓ Commit · Discard`; ✓ Commit takes
`route_count()` 0 → 1, and the Journey Planner opens on
`Route #0 — 506 km (mixed)` with route map, elevation profile, stage bands and stops strip
(`jpprobe_06_journey_planner_open.png`). IN-09's verification holds; "draw a
route" was only unreachable by the two means the owner had reason to try.

### A rule this adds to the register's own method

IN-09 left *"a `#[func]` that returns geometry proves nothing about whether
anything draws it. Check the pixels."* The layer above: **a control that exists,
is enabled, and works when invoked proves nothing about whether a user can find
it.** Both fixes are one line of behaviour, found only by launching the app and
doing the obvious thing. (`MISTAKES.md`'s *Reach a screen inside a probe* row
generalises this: renders / can be operated / can be FOUND.)

### Verification

`_fixprobe_shot.gd`, real windowed shell, generated world, **24 assertions, all
passing**: `Data ▸ Journey planner…` from the launch domain and from CARTO
switches to CIVIL and paints; in CIVIL `W`/`⇧R`/`R`/`S`/`T`/`V`/`M` arm
way/route/region/settlement/territory/inspect/measure, in CARTO `L`/`I` arm
label/icon, in WORLD `B` arms paint; **cross-domain letters are inert** (`L`, `B`
in CIVIL; `W` in CARTO; `⇧R` in WORLD); `W` with a `LineEdit` focused types a
`w` and leaves the tool alone; CARTO reads "Region labels" with the new empty
state (the remaining "none placed" is the Icon panel's own).

Probe artifact that nearly produced a false bug: the `LineEdit` check failed
until the synthesised `InputEventKey` carried a `unicode`. A real keyboard
always sends one, and it is what makes `LineEdit` consume the key before the
shortcut pass — assert against the event the hardware actually sends. Headless
boot of `shell/app.tscn` clean.

---

## 28 · SH-01, IN-12, MT-01 — the rail collapsed, nothing scrolled, and the measure buttons were in the wrong order (2026-08-24) — **FIXED**

Three owner reports from one live session, against `design/Cartalith DCC
Shell.dc.html` and `design/Cartalith Measurement Toolbar.dc.html` as ground
truth:

> 1. the left rail is collapsible and shouldn't be
> 2. rail scrolling doesn't work properly on mouse hover
> 3. the measurement tool quick-buttons aren't in the same position as the design

The middle one was the largest: **no `ScrollContainer` in the application could
be scrolled with the wheel**, since the map camera was built.

### SH-01 — the rail expansion is withdrawn

`_build_rail()` had made the mockup's head chevron a real `Button` (§7.17's
proposal, built 2026-08-19) growing the rail to `W_RAIL_EXPANDED` (200 px) with
a `_phone_list_row()` list of each domain's sub-structure — as
`DCC_SHELL_SPEC.md` §3 asks. **The canvas never draws it**: all eight desktop
artboards across both canvases (dark, light, tablet, three measurement states)
open the rail with the literal `width:40px;flex:none`. The built state borrowed
the phone drawer's type scale into a 200 px column; live, `CARTOGRAPHY` ran
under the left dock.

Withdrawn, with `_rail_subnodes_body`, `_rail_expanded`,
`_rail_expand_button`, `_rail_panel`, `DOMAINS[i].subnodes` and
`DccTheme.W_RAIL_EXPANDED`. **What the canvas draws is kept**: the 29 px head
cell with its dim `›`, now a `Label` with `MOUSE_FILTER_IGNORE`.

**Not changed: `Window ▸ Domain rail`**, which still hides the whole region —
the same reversible region toggle the other four layout regions have, not what
"collapsible" meant here. Recorded rather than removed on inference.

### IN-12 — one `_input` handler swallowed every wheel event in the shell

`viewport_host.gd::_input()` handled `MOUSE_BUTTON_WHEEL_UP`/`_DOWN`
**unconditionally**, with no rect test, then called
`get_viewport().set_input_as_handled()`. `_input` runs on every node for every
event, **before** GUI dispatch — which is why the handler is there (its header
records that `_unhandled_input` never sees the MMB press) — so a notch anywhere
zoomed the map and cancelled GUI dispatch. The left dock (836 px of content in a
774 px window), right dock, every popover and every dialog body were
unscrollable; reproduced at three hover points, all `scroll_vertical == 0` after
five notches.

Fix: the navpad guard the LMB branch already had, generalised — a **press**
belongs to the camera only when it lands on `ViewportHost`'s own rect. Releases
are exempt on purpose: a pan that began on the map and ended over a dock must
still clear `_panning`, or the camera sticks to the cursor.

Third instance of one class (`4e000a3` the phone sheet that would not flick,
`695821f` the right-dock pane that sized itself to its text): **a control that
participates in input dispatch affects nodes it does not own.** The first two
were `mouse_filter`; this is `set_input_as_handled()`, one layer up.

### MT-01 — the measurement quick-buttons were one flat run of six

The canvas draws the row as **three groups separated by rules**, identically in
all three states:

```
[Distance Bearing Area Radius] │ CROSS-SECTION [Elevation … Custom▾] │ [Δ vertical  3D distance]
```

`tool_bar.gd` flattened all six `MEASURE_MODES` into one run (… Radius ·
**Cross-section · Δ vertical**) and hid the channel row behind a `Field`
dropdown in the options bar, shown only once Cross-section was armed. Every
button after Radius sat at the wrong x (`Δ vertical` at x 533 against the
canvas's third group) and five canvas quick-buttons were absent.

Rebuilt to the canvas's grouping, explicit in `tool_bar.gd`
(`MEASURE_GROUP_POINT` / `MEASURE_GROUP_VERTICAL`) rather than derived from
`MEASURE_MODES`' order — that const is the engine's six readings; the grouping
is a presentation fact about this row. **A channel button is how the canvas
arms Cross-section** (its first group has exactly four buttons, no
Cross-section): picking a field arms the section if needed, and picking another
while live only swaps what the strip draws. The `Field` dropdown is gone rather
than duplicated. Two canvas buttons stay undrawn, disclosed in the row's
trailing note: `Custom ▾` (no user-defined field to bind) and `3D distance`
(greyed "3D only" in the canvas itself; Δ vertical already returns it in the
dock).

### Verification — live and windowed, because all three are invisible headless

`_railprobe_shot.gd`, real shell at 1600 × 900:

- **rail**: `has_method("_toggle_rail_expansion") == false`; three buttons, all
  domains; `Rect2(0, 70, 40, 804)` at window widths 1600, 1100, 760 and 640;
  `Window ▸ Domain rail` still hides and re-shows it;
- **wheel over the rail**: camera position and zoom unchanged (before, it
  zoomed); **over the left dock**: `scroll_vertical` 0 → **62** at all three
  points — the entire 836 − 774 range — including with a `Button` under the
  cursor; **over the map**: still zooms 1.00 → 1.15, one `ZOOM_WHEEL_STEP`;
- **measure row**: `Distance 195 · Bearing 265 · Area 329 · Radius 375` │
  `CROSS-SECTION 440 · Elevation 556 · Terrain 632 · Climate 696 · Hydrology
  760 · Geology 836` │ `Δ vertical 907`; pressing `Climate` sets
  `measure_mode()` to `section` and `section_channel()` to `climate` in one
  click, CROSS-SECTION going accent (canvas state 2 against 1 and 3).

Headless boot of `shell/app.tscn` clean.

## 29 · RD-01 — the roads curve, and the renderer was drawing their chords (2026-08-24) — **FIXED**

Owner report:

> settlement roads all render as straight lines — no organic curvature

`PARITY_AUDIT.md` pass 1 lists path smoothing as clean and ported, and it is:
`civ_smooth_path` (`_civSmoothPath`, reference line 21892) faithfully ports
`rdpSimplify(run, 1.5)` → `catmullRomSample(·, 3)` → `Math.round`, runs live
(`civ_consolidate_and_smooth_ways`, from `compute_civilisation`), and
`golden_parity_road_consolidation.rs` asserts its exact point lists. The first
three suspects were all wrong, eliminated in turn.

### What was measured, in order

**The engine's ways really curve.** A probe over a real 384×288 world (seed
483920, `generate_terrain` → `civ_hierarchical_network_topology` →
`civ_consolidate_and_smooth_ways`): **mean sinuosity 1.072, ~11° of turn per
vertex** across 51 visible ways.

**A false alarm that nearly became the answer.** A first fixture on an exact
lattice gave 27 of 47 ways *precisely* zero deviation from their chord — reading
as a broken cost field. It was the fixture: an axis-aligned pair has exactly one
minimum-step 8-connected path. Jittered off the lattice: 27/47 → 8/51 nearly
straight. (`CLAUDE.md`'s "shape fixtures to reach the code", from the other
direction: a fixture can *hide* the code by making the answer degenerate.)

**The renderer draws every point.** `map_overlay.gd`'s `_draw_way_segment` is one
`draw_polyline` per run between `brks` entries — no decimation.

**Curved and fully drawn, yet it looks straight.** Rasterised from the engine at
4 and 24 px per cell: the way is a **polyline of 17-20 points whose chords are 3
grid cells long**; at 24 px/cell those are 72 px straight lines meeting at
visible angles. The curve is real; the screen gets its chords.

### Root cause: a sampling rate calibrated for a canvas that never zoomed

`_civSmoothPath` samples the spline every **3 grid cells** and rounds to whole
cells; `rdpSimplify`'s own comment gives the units away — *"eps in grid units
(caller passes ~1 screen px)"*. The reference draws at roughly one cell per
screen pixel, so a 3-cell chord is 3 px and ±0.5-cell rounding is ±0.5 px;
drawing `lineTo` between those points is indistinguishable from the curve. Here a 384-cell grid fitted to the centre
panel is ~3.6 px per cell and `ViewportHost.ZOOM_MAX` is 8, so one cell can be
**~29 screen px** and a chord ~87 px. Nothing is wrong with the port; the
reference's sampling rate is not a rendering resolution here.

The reference hit the near end of this and recorded it: `_civSmoothPath`'s
**v0.92** note answers another owner report — *"roads nearly miss settlements
when zooming in"* — by un-rounding a way's endpoints, verbatim: *"up to half a
cell of drift that's imperceptible at low zoom but, amplified by LOD zoom (one
grid cell can span many screen pixels), visibly..."*. It keeps interior points
rounded because *"their precision was never load-bearing"*. Under this port's
zoom, it is.

### The fix: the same curve, sampled at render density, in the boundary layer

`get_roads()` (`cartalith-godot/src/lib.rs`) re-samples each way's curve through
its own control points at `WAY_RENDER_STEP_CELLS =
0.25` cells, remapping `brks`, via `cartalith_civ::civ_catmull_rom_sample` —
**the same one definition**, now `pub`, not a second smoothing algorithm.

**Nothing upstream moves.** `Way::pts` is untouched, so `km`,
`_civNetworkMetrics`, `urban_adapter::um_primary_paths` (which reads ways
directly) and every road golden see what they saw. In `cartalith-civ` it would
have changed `_civSmoothPath`'s constants and re-baselined goldens for a
presentational reason; in `map_overlay.gd` it would have put geometry in
GDScript against `ARCHITECTURE.md` (and that file was under concurrent edit).

0.25 cells: a ~7 px chord at `ZOOM_MAX`, sub-pixel below, where finer buys
nothing. Each run between `brks` is re-sampled separately (splining across a
break draws a phantom curve through the seam), and each run's endpoints are
re-asserted afterwards so v0.92's exact-meeting guarantee survives (the spline
lands ~1e-16 off, below `f32`, but the invariant is written, not inferred).

### Verification — measured in the real shell, windowed at 1600 × 900

`_roadcurve_shot.gd`: seed 483920 at 384×288 / 2400 km through the real
`app.tscn`, camera driven to `ZOOM_MAX` over a junction; same build, seed and
view, with and without the re-sample:

| | before | after |
|---|---|---|
| points across 35 ways | 589 | **6,342** |
| mean chord | 2.78 cells | **0.245 cells** |
| max chord | 4.24 cells | **0.328 cells** |
| turn per vertex | 14.47° | **1.70°** |
| longest way `km` | 1243.3 | **1243.3** (unchanged) |
| longest way drawn length | 198.9 cells | 199.6 cells |

Total turning is the same road spread over twelve times the vertices — the
difference between a corner and a curve. Drawn length rises 0.35% (the arc
measured rather than cut); `km` does not move. Screenshots at `ZOOM_MAX` (pins
and labels hidden): hard angles before, continuous curves after.

Headless boot clean. `cargo test -p cartalith-civ` 493 passing (every golden
suite unchanged); `cargo test -p
cartalith-godot --lib` 334 passing, including three new `way_render_tests`
(density, the `brks` remap, empty/single-point/out-of-range inputs).

### Still open

**Sea routes and committed Route-tool routes were deliberately left alone** at
the time (same shape, same chord problem; `map_overlay.gd`'s route rendering was
under concurrent edit). **Closed by §33**, which also found a NaN this
section's fix had been shipping.

**The long straight runs are real, not a defect.** Between corners the route
itself is straight: the routing grid is capped at 384 cells wide and land travel
cost is piecewise-constant within a biome (p10 1.0003 → p90 1.61; the slope term
contributes a p50 of only 0.0014). A road across homogeneous flat ground *is* a
straight line; making it meander is a cost-model change — a `DECISIONS.md`
conversation, not this fix.

## 30 · MR-01, MR-02, MR-03 — the map overlay rasterised in the wrong space, twice, and gated on a moved baseline (2026-08-24) — **FIXED**

Three owner reports against the live map:

> 1. Settlement name text goes blurry quickly and doesn't scale.
> 2. Minor settlements (villages/hamlets) are always visible instead of zoom-gated.
> 3. Routes draw slightly see-through and blurry.

| id | Symptom | Root cause | Fix |
|---|---|---|---|
| MR-01 | Settlement names blur within a notch or two of the default view, and grow with zoom instead of holding still | `draw_string`'s `font_size` is in `map_overlay.gd`'s **local** space, which `ViewportHost` scales by `_camera.scale`; Godot re-scales a `CanvasItem`'s recorded draw commands rather than re-running them, so a 9 px glyph is a 9 px bitmap stretched over 72 screen px at `ZOOM_MAX`. The `maxi(9, …)` floor (the reference's `Math.max(9, sz+lsc)`) also defeated `_civ_zoom_k()`'s compensation: `sc` here is 0.63, `radius + sc` never reaches 9, so the label sat at 9 *local* px and `9 × zoom` on screen | `_crisp_begin()`/`_crisp_end()` — a `draw_set_transform` of `1/zoom` inside which every coordinate and size is a **screen** pixel; glyph and name are measured, rasterised and drawn at final size, crisp and constant across 0.4-8.0 |
| MR-02 | Villages and 209 hamlets drawn full-size, with pins and names, on a map that had never been zoomed | Two causes, one dominant. **(a)** `lib.rs` folds `civ_seed_villages`' output into the roster as plain `Hamlet`s, disclosed as *"a village renders exactly like any other hamlet, which is what the reference's own hamlet-tier tagging for these already implies."* The reference does the opposite: tags them `villageAddon` **so the renderer will not treat them as hamlets**, gates them at `CIV_VILLAGE_ADDON_LOD = 2.4` rather than `CIV_LOD_PLACE.hamlet = 1.4`, and hides them **outright** with no dot fallback (its comment names the complaint: "waay too populated"). Measured: **200 of 209 hamlets are addon villages**, against 24 real settlements; and the shell defaults `villages` to `true` where the reference defaults `false`. **(b)** `_settlement_below_lod` compared `SETTLEMENT_LOD` with raw `_camera_zoom`, whose meaning moved on 2026-08-23 when `reset_view()` became the reference's **cover** scale: `>= 1` by construction and window-shaped, so a world opens at `z = 1.04` in one dock layout and `1.36` in another, and any threshold under 1.4 could be met by the opening view | **(a)** `VILLAGE_ADDON_LOD = 2.4`; below it an addon village draws nothing and is not hit-testable (the reference's `_civPlacePickVisible` excludes it too). **(b)** thresholds compare zoom **normalised by `_lod_zoom_base()`**, re-derived from this control's geometry, so `1.0` means "the view a world opens at" on every window. Measured at the default view: **33 places drawn instead of 233** |
| MR-03 | A committed route reads as a wide, translucent, blurred band rather than the reference's dark-underlay-plus-dashed-amber | MR-01's space error in the other primitive. The reference multiplies **every** way and journey `lineWidth` and dash length by `rsc` (line 15470, `max(1,GW/512)*_civZoomK()*_civWayScale()`); the port dropped the term. A local-space width is scaled **with its antialiasing fringe** — at zoom 8 the 1.5 px amber dash is 12 px wide with ~8 px of fringe each side. Ways and sea lanes had it too; routes' alpha (`.5`/`.85`) makes it obvious | The three linear layers draw inside `_crisp_begin()`, so every width and dash constant is screen pixels with a screen-resolution AA fringe. `ROAD_WIDTH_BY_TYPE`'s 1.6 is 1.6 px at any zoom |

MR-01 and MR-03 are one bug in two primitives (a screen-space quantity computed
in the overlay's local space, magnified with its rasterisation) and share one
fix; MR-02 is independent.

**Verified live, non-headless**, 1600 × 1000 and 2400 × 800 windows, seed
483920 over 384 × 288 with all six tiers and one committed 2,070 km route, at
the reset view and 1.5×, 3× and 5.9×. Before: labels stretched to 54 px of
bitmap mush, 233 places at the default view, the route an amber smear. After:
labels crisp at every zoom, 33 places with addons revealing at 2.4×, the route a
thin dashed line over its underlay. Headless boot and `smoke_test.gd` clean.

**Left open.** `VILLAGE_ADDON_POP` identifies an addon village by its
unconditional `pop: 0` — exact for the default pipeline (the smallest base tier
floors at `round(120 × 0.7 × 0.8) = 67`), documented in `VillageSettlement` and
`lib.rs` — but it is a proxy for a flag the engine keeps:
`CivData::village_tids`, beside the `tid` `get_settlements()` already emits.
Exposing it is one line; not taken because `crates/cartalith-godot/
src/lib.rs` was under concurrent edit. Until then one case degrades: with the
static post-collapse recovery phase on, `civ_apply_recovery` floors every
population at 8, so an addon village draws as an ordinary hamlet — "no
improvement", never a place wrongly hidden.

**Also noted, not changed.** `engine_bridge.gd` defaults `villages` to `true`
(`request.get("villages", true)`) where the reference's `_civVillages` is
`false` (*"OFF by default ⇒ auto-populate output bit-identical"*), so every
world here carries the additive layer. A generation default, not a render
defect — `DECISIONS.md` territory, raised rather than taken.

## 31 · TO-01, TO-02, CV-20, MN-09, SH-15 — a second overlay in the wrong space, and four surfaces that had stopped telling the truth (2026-08-24) — **FIXED**

Owner report: *"plenty of minor discrepancies at the same time"*, alongside §29
and §30. Same method: a `design/*.dc.html` canvas as ground truth, then **drive
the live shell non-headlessly** and measure, across areas §29/§30 had not
touched.

| id | Symptom | Root cause | Fix |
|---|---|---|---|
| TO-01 | Every measure ruler, region marquee, path preview and A/B end label thickens and blurs as the map is zoomed — **§30's defect, in the other overlay** | `ViewportHost` parents `map_overlay` **and** `tool_overlay` under `_camera`, but only `map_overlay` was told the zoom. Every `tool_overlay.gd` constant — a 1.6 px `draw_polyline`, a 3 px `MEASURE_POINT_RADIUS`, an 11 px `draw_string`, a 1.4 px dashed marquee with 6 px dashes and 6 px corner squares — is in *local* pixels, magnified with its rasterisation | `map_overlay.gd`'s `_crisp_begin()`/`_crisp_end()` over the whole `_draw()` (this control emits only tool chrome). The two real map distances — the brush ring, the Radius reading's circle — multiply by `cell_px` so they keep scaling. Zoom is **read off `_camera.scale.x` in `_process`** rather than pushed from `viewport_host.gd` (under concurrent edit); `set_notify_transform(true)` was tried first and does not work — a `Control` ancestor's `scale` change does not propagate `NOTIFICATION_TRANSFORM_CHANGED` to children |
| TO-02 | A selected label's or icon's resize/rotate/arc handle is a much smaller circle than the region that answers a click, and the mismatch grows with zoom | **`HandleCircle.r` is in grid cells, not pixels.** Both producers build it in `x`/`y`'s space (`label_bridge::handle_circles` from `LabelBox.px/py`, `icon_bridge::icon_handle` from `IconBox.px/py`), floor it at `4.0` *cells*, and hit-test at that radius; `tool_overlay.gd` passed it to `draw_circle` as *pixels* | The brush ring's `cell_px` conversion. Measured: `r = 6.4` cells at 2.31 px/cell draws 32 px against the ~30 px the hit test answers (~13 px before) |
| CV-20 | CIVIL ▸ Politics offers *Recalculate territories* and *Generate provinces* under a **Not built** heading, greyed, tooltips asserting no `#[func]` re-runs either — while **Recompute civilisation, eight rows up, does both** | Tooltips predating SG-02, never revisited. `civ_recompute()`'s result reports `provinces` rebuilt, `_recompute_civ()` re-uploads `territory_texture()`, and the Settlement-tool hint has said so since. §30's class of stale copy; the shape of the bake pass's *"No bake/LOD pipeline exists yet"* | Both are live **shortcuts onto `_recompute_civ`** — one owner, two ways in, nothing to drift (the bake pass's pattern). Tooltips name the real limit (it does not re-*place* settlements; only Generate does). *Clear territory* stays disabled — genuinely absent — with its "Same:" premise corrected |
| MN-09 | Assets ▸ Asset pack ▸ Build ▸ *Export pack .zip… ⌘⇧P* prints its shortcut **twice**, once as a modifier neither shipping platform has | Label baked `⌘⇧P` as text *and* `set_item_accelerator` added `Ctrl+Shift+P`, rendering `Export pack .zip… ⌘⇧P    Ctrl+Shift+P`. The canvas draws `⌘⇧P` in the accelerator column, not the label | Label only; the accelerator renders the canvas's layout in the machine's notation. Sibling: Batch ▸ *Delete ⌫* advertised a Backspace binding that **exists nowhere** (`app.gd`'s `_unhandled_key_input` routes Backspace to the armed tool only; `asset_library_window.gd` has no key handling) on a row that opens a window. Glyph dropped |
| SH-15 | §10's timeline strip is **70 px of blank panel across the whole window** whenever CIVIL is active, and `Window ▸ Timeline` toggles the blank band | Timeline controls deliberately live in the CIVIL dock's Timeline category (`TIMELINE_SCOPE.md` §4); leaving the reserved region *on screen and empty* was never part of that decision | The strip carries a pointer: a `TIMELINE` caption, one clipped line saying where the controls are, and **Open Timeline**, which presses the dock category's header (`CivilizationWorkspace.open_timeline_category()`). Re-filled by `toggle_region()` too |

**Checked and found clean** (negative results, so the next pass does not re-walk
them):

- **The Layers popover's field views, all 37 rows.** 33 available rows clicked
  through the real `pressed` path: each set its view (`debug_view()` echoed the
  id) and produced a distinct raster (37 FNV hashes, **no duplicates, no
  nulls**); all four unavailable rows correctly disabled; **hotkeys 1-8 each
  selected exactly their badged row**; opacity live; correct after a regenerate
  to a different grid size.
- **A dead-control sweep of the whole live tree** — every enabled, visible
  `Button`/`CheckBox`/`OptionButton`/`Slider`/`LineEdit`/`SpinBox` with no
  connection on any signal — across shell chrome, the three domain docks, four
  right-dock contexts (settlement, faction, sculpt, region), **all nine
  tool-options bars** (sculpt, paint, measure, icon, label, settlement,
  territory, way, route) and eight windows (Asset library, Data manager, Faction
  roster, Place editor, City viewer, World data, Performance, Travel library).
  **No dead controls.** A cruder first heuristic's four flags were false
  positives: `toggle_mode` tabs connecting `pressed` not `toggled`,
  `ColorPickerButton`s on `color_changed`, search fields filtering on
  `text_changed` not `text_submitted`.
- **Every menu accelerator**, from the live popups: 11, each matching its label,
  none unreachable except `Ctrl+Z` while the undo stack is legitimately empty.
- **Camera-space rasterisation, exhaustively.** `map_overlay` (§30) and
  `tool_overlay` are the only drawing `_camera` children; the rest are
  `TextureRect`s; `wind_fx_layer.gd`'s one-cell stroke is the reference's own
  and *meant* to scale; `journey_planner_view.gd` and `section_strip.gd` draw in
  unscaled dock controls.

**Verified live, non-headless**, 1600 × 1000, seed 483920 over 384 × 288, by
difference against the same frame without the primitive (no terrain-colour
assumption). TO-01 before: the 1.6 px ruler rendered **2 / 6 / 12 / 16 px** at
zoom 1 / 2 / 4 / 6, and the 11 px `A` label's box went **17 × 18 → 69 × 74 px**
between zoom 1 and 4. After: **2 px at every zoom**, **17 × 18 px at both**,
while the 20-cell brush ring still grows 94 → 372 px. TO-02 measured in the same
run. CV-20 pressed for real: *"Recomputed in 0.8 s: 233 settlements kept, 60
ways and 8 provinces rebuilt against the current terrain."* SH-15: 4 children
in CIVIL, still 4 when switched on from WORLD via the Window menu, strip
minimum width 236 px against a 1600 px window and 300 px right dock.

**Left open, reported rather than taken** (two since closed, same day):

- ~~**The map's top-right readout does not carry what the canvas puts there.**~~
  **Closed, 2026-08-24.** `viewport_host.gd`'s `_update_zoom_readout()` draws
  `2D · equirect · z%.1f` over the active style-preset name, matching
  `design/Cartalith DCC Shell.dc.html`'s `2D · equirect · z 5.2` / `relief ·
  atlas preset` structurally: "2D" and "equirect" are honest constants
  (`DCC_SHELL_SPEC.md` §2.4: "this port works in one flat km projection
  throughout"), the second line the real active Map style preset
  (`render_workspace.gd`'s five chips plus "Custom"), pushed via a new
  `ViewportHost.set_style_readout()`. Grid size and extent already show in the
  WORLD dock readout and the Sample panel.
- ~~**The Asset Library has no keyboard delete.**~~ **Closed 2026-08-24**, the
  least clever way: `_unhandled_key_input` on the library `Window` routes Delete
  and Backspace **into `_on_batch_delete`** — same confirmation, count and
  "custom slots are removed entirely, frozen slots are emptied" wording (a
  key-only prompt would be a second place for it to drift). Scope is the grid
  selection; there is no undo and the prompt says *"This cannot be undone."* Two
  guards, as `app.gd`'s handler needed: **a focused text field wins**
  (`LineEdit`/`TextEdit`/`SpinBox`), and **an empty selection says so** in the
  status bar rather than looking dead on the press that teaches the key exists.
  On the window, not `DccApp`, because a `Window` is its own `Viewport`: the key
  arrives only while the library has focus, and the slicer modal does not steal
  it. The menu glyph stays off (`menus.gd` says why: that row opens the window).
  **Verified non-headless** on a 7-slot library: empty selection → 0 dialogs
  and a hint; Delete with 2 selected → 1 dialog *"Delete 2 asset(s)?"*,
  **Cancel keeps both**; Backspace → same dialog, OK runs the batch (frozen
  slots emptied, count stays 7); Backspace with a `LineEdit` focused → **0
  dialogs**.
- **Four `ID_*` constants in `menus.gd` are declared and referenced nowhere
  else**: `ID_REDO`, `ID_HELP_SHORTCUTS`, `ID_PREF_QUALITY`, and
  `ID_PREF_UNITS_KM`/`ID_PREF_UNITS_MI`. Not user-visible (Redo is a real
  `_todo` row; the units switch is the gap `tool_bar.gd`'s Measure options
  disclose), but the residue of four intended surfaces, recorded so the intent
  is not lost.

## 32 · LZ-01 — deep zoom stopped twenty times short of the reference, and the tile it drew had run out of octaves (2026-08-24) — **FIXED**

Owner report, verbatim: *"LOD zooming doesn't seem to go that deep either."*
Measured before changing anything, default 800 km × 512×384 world, 1600×1000
window:

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

The reference caps `viewT.scale` at 8 (line 13381), and the port took that. But
it *hands the camera off* at 2.2×: `enterLodFromView` (13953) pins `viewT.scale`
to 1 and gives zoom to the tiled-LOD viewer, whose `_lodZoom` runs to
`lodMaxZoom()` = `max(64, ceil(mapWidthKm/5))` (10672) — **160 on a default
world**. Its v0.88 comment answers an owner report of this shape: *"highest zoom
stops at 20km, I'd like to drop down to 5km … Scale the cap so a real-world span
of ≤5km is always reachable."* Reachable depth is a property of the map's real
width; `ViewportHost` now computes it per world in `refresh()`.

### 2. The tile had a fixed footprint, so its resolution saturated

`lod_bridge` addressed tiles on a fixed 64-coarse-cell grid and grew the
*output* (256/512/1024 px) with a `detail_level` capped at 2 — a 16 px/cell
ceiling, exactly where `ZOOM_MAX = 8` sat; raising the cap alone would only have
magnified it. It is now a **pyramid** tile: level `z` divides the map into
`2^z × 2^z` fixed-pixel chunks (`cartalith_spatial::pyramid`,
`pyramid_tile_bounds`, `pyramid_level_for_zoom` — already ported and
golden-tested for the bake), so the footprint shrinks with depth at constant
per-tile cost, and tiles per viewful are flat at 24 from z3 to z9 — bounded by
screen area, not zoom.

### 3. The tile carried no progressive detail at all

`synthesize_tile_rgba` called `amplify_region` alone — the failure
`addZoomDetail`'s header names: *"amplifyRegion adds detail at a FIXED
coarse-space frequency, so the fbm runs out of octaves at high zoom and the
surface goes smooth ('details don't get more intricate')."* Synthesis is now
`cartalith_engine::bake::pyramid_tile` verbatim — `refine_tile` **plus**
`add_zoom_detail`'s `min(6, z − zBase)` finer octaves — so a screen tile and a
baked atlas chunk over the same ground are the same numbers by construction.
`z_base()` is shifted by `log2(1024 / TILE_PX)`: the reference's `zBase = 2` is
quoted against its 1024 px `_lodTile`, and an unshifted `2` would add two
octaves past what a 256 px tile resolves — noise, not detail.

### The three things the live driving found that reasoning had not

- **The hillshade faded with depth.** `shade_tile` differences *adjacent pixels*
  with a fixed `exag`, so the same slope shades `1/px_per_cell` as hard once a
  cell spans many pixels. On a dome fixture: 34% of mask pixels shaded at level
  4, 3% at level 7 — a flat mask *even with the octaves in*. Exaggeration is now
  normalised by the tile's pixels-per-cell (scale-invariant); mean
  adjacent-pixel difference across four levels runs 2.44 → 3.36 → 4.31 → 5.49
  instead of 0.30 → 0.03. A free parameter, not a reference constant: the shade
  *ratio* is this port's own construct.
- **`gui/common/snap_controls_to_pixels` destroys a deep-zoom `TextureRect`.** It
  rounds a `Control`'s rect to whole *local* pixels, and `_camera`'s local pixel
  is `_zoom` screen pixels: at z160 the map is 5.5 local px wide, so a 1.74
  local px tile snapped to 1 or 2 — 160 or 320 screen px instead of 278. A
  shown/hidden frame diff found 40 px vertical and 120 px horizontal bands the
  layer changed *not at all*, though the tile arithmetic covered the screen with
  one-pixel overlap. Tiles are `Sprite2D` now (`Node2D` carries a float
  transform, never snapped); the old 64-cell tiles at `ZOOM_MAX = 8` were
  several hundred local px, which hid it.
- **The scale bar read the same at every zoom** — the map's full width, so the
  deepest view still said *"800 km across"*. It is `lodSpanKm()` now (reference
  10675: *"the single source of truth for both the scale bar and any future
  'current view width' readout"*), reading **5.00 km across** at the cap.

### What is genuinely a separate milestone, checked rather than assumed

- **Reading the baked atlas at draw time is not the depth fix**, and as built
  would reintroduce a bug fixed the day before: a baked chunk's PNG is
  `region_export::tile_png_bytes`, the **Relief** coloriser — the hypsometric
  ramp the 2026-08-23 pass removed from this path because it disagrees with the
  biome map at every pixel ("a zoom action exposes the underlying heightmap").
  The reusable half is the chunk's *height* (`rg16`,
  `cartalith_io::decode_chunk`), which has no `#[func]`. And a depth-7 pyramid is
  21 845 tiles, so the atlas can only serve shallow levels, where live synthesis
  is now 12 ms. An optimisation, not the ceiling.
- **Colour at depth is still an interpolation of the coarse raster.** Relief is
  genuinely sub-cell; the *palette* is not, because `renderBiomeTileRGBA` is
  unported (needs temperature, rainfall, lithology and flow at sub-cell
  resolution) — named in `lod_bridge.rs`'s header since 2026-08-23.
- **The frame at extreme zoom costs ~27 ms, and the LOD layer is not why.** At
  z160 with the layer visible, hidden, and at the reset view: 27.17 / 26.74 /
  16.68 ms. The layer is 0.4 ms; the other ~10 ms is the base raster and
  `map_overlay` at 160× — newly reachable, not newly slow. Not chased.
- **`_umLayoutAlpha`'s 24 km → 10 km crossfade is live ground for the first
  time.** `map_overlay.gd`'s urban reveal gate is a pixel rule written against
  the old ~100 km floor, commented that a ported 24 km threshold "would never
  once fire". That premise expired; the comment says so, and the gate was
  deliberately not swapped here — a visible behaviour change belonging with the
  rest of `_umLayoutAlpha`.

## 33 · RD-01b — the sea lanes and committed routes drew their chords too, and the road fix had a live NaN in it (2026-08-24) — **FIXED**

Closes §29's registered leftover, and found something bigger on the way in.

### The chord half, which was the expected part

`get_sea_routes()` (generated `sea_routes` and the manual `sea` ways out of
`InfraTools`) now goes through `way_render_geometry`, §29's boundary helper.
`route_get()` does too, but as a **second key**, the one real design difference:

- `points`/`brks` stay the engine's own list: `jp_compute` plans over
  `CommittedRoute::pts` and returns `plan.stages[i].{i0, i1}` as indices into
  it, which `journey_planner_view.gd` slices for per-stage colour and stop
  fractions. Densifying `points` would silently mis-slice every stage.
- `render_points`/`render_brks` are the drawn polyline; `map_overlay.gd`'s route
  pass reads them, falling back to `points` for an older GDExtension binary.

Measured on §29's world and probe (seed 483920, 384×288, 2400 km,
`_routecurve_shot.gd`):

| | points | chord mean | max | turn/vertex |
|---|---|---|---|---|
| sea lanes, after | 807 over 2 lanes | 0.246 cells | 0.314 | 1.686° |
| committed route, before | 124 | 2.856 cells | 4.243 | 13.607° |
| committed route, after | 1437 | 0.245 cells | 0.330 | 1.665° |

`km` did not move (2195.460 engine-side; drawn length 351.27 → 352.18 cells),
and both endpoints are byte-identical. Roads re-measured in the same run: 6342
points across 35 ways, chord mean 0.2450 — §29's figures to the digit. At 31× on
the committed route: a visible kink before, one continuous sweep after.

### The NaN half, which was not

The sea-lane measurement's first run returned `chord mean -nan`.
`civ_catmull_rom_sample` parameterises each segment by `sqrt(chord)`, and the
Barry-Goldman evaluation divides by **all three** knot intervals while guarding
only the middle one (`t2 - t1`). Two equal consecutive control points make
`t1 - t0` or `t3 - t2` zero in a *neighbouring* window, `lerp` computes
`0 * (x / 0)`, and that whole segment is NaN — one NaN ruins the
`PackedVector2Array` the renderer gets.

Unreachable from `civ_smooth_path`, the reference's only caller, because it
splines `civ_rdp_simplify`'s output and RDP always drops a duplicate. **§29's fix
introduced the first caller that can reach it**: `way_render_polyline`
re-splines `_civSmoothPath`'s *rounded* output, where two samples in one cell
are routine (`golden_parity_sea_routes.rs` records two case-1 routes with
`km: 0` for that reason). So this was live in already-shipped road rendering,
just never measured on a way that stalls.

Fixed **in `civ_catmull_rom_sample` itself**, so roads, sea lanes and routes
share one guard: repeated consecutive control points are collapsed before the
phantom endpoints are built. Parity-neutral, exhaustively: for input with no
repeat `dedup` is the identity, and every input with one previously produced
either NaN (runs of three or more) or an empty result (a two-point run, via the
existing `t2 - t1` skip, which the new `< 2` check reproduces exactly). No
fixture can tell the versions apart. `WAY_RENDER_STEP_CELLS`'s local dedup,
written while chasing this, was removed for the central one.

Mutation-tested: with the guard forced off, three of five new tests fail
(`catmull_rom_survives_coincident_control_points`, walking a duplicate through
every position of a five-point way; `catmull_rom_survives_runs_of_repeats`; the
boundary-level `a_repeated_cell_in_a_rounded_way_does_not_produce_nan`) and two
stay green by design — they pin *reference* behaviour
(`catmull_rom_degenerate_inputs_match_the_reference`, and
`catmull_rom_keeps_a_near_coincident_pair`, which stops anyone "improving" the
exact-equality test into an epsilon and moving the curve).

`cargo test -p cartalith-civ` 372 lib + every golden suite passing,
`cargo test -p cartalith-godot --lib` 337 (five new), `cargo build -p
cartalith-godot` and headless boot clean; a non-headless scan of all three
getters: 0 non-finite of 6342 road, 807 sea and 1437 route points.

## 34 - RN-04, CA-14 - the renderer was already sophisticated; its defaults were conservative (2026-08-24) - **FIXED**

Owner analysis, and a correct one: the reference's renderer runs a full pipeline
(climate → material weights → biome/material colour → texture → relief →
multi-scale hillshade → curvature/AO → atmospheric haze → optional painter
effects → rivers/coast). What makes it look muted is that **most enhancement
sliders default to `0`** and its base palettes are low-chroma — not missing
features. The instruction was explicit: do not rewrite the renderer. Nothing
here is a rewrite.

| id | what | now |
|---|---|---|
| **RN-04** | The four reference render stages the panel's own note listed as unported: **ridge crests**, **surface texture**, **ridged relief**, **curvature shading** | **CLOSED.** Literal ports in the reference's pipeline slots — `build_crest`/`apply_crest` (8005-8023, applied at 8171 and 11971), and three blocks inside `land_color` (7841-7851, 7853-7862, 7870-7876). `cartalith-noise` gained `ridged_oct`, the reference's general `ridgedFbm(x, y, oct, s)`, **beside** the golden-verified fixed-six-octave `ridged`, not by rewriting it. All four are `0.0` in `Default`, so `golden_parity_render.rs` is untouched. |
| **CA-14** | **Colour grading** — the group `render_workspace.gd`'s Still-owed block has listed since the dock was built | **CLOSED (six of ten axes).** `render::apply_color_grade`, presentation-only over the **finished raster**, after `apply_local_contrast` and before the Godot overlays (rivers, labels, icons, territory, scale bar): exposure, contrast, saturation, temperature, shadow tint, highlight tint, one pass in that order. Saturation is exactly luminance-preserving and both hue axes are luminance-compensated, so a graded map keeps the relief pipeline's value structure. **CLOSED (2026-08-24, all ten axes).** Gamma joined as a symmetric power curve (exponent `2^-gamma`) in the lift-gamma-gain slot after exposure, gated at `0` so no `powf` runs at rest. The **four field-influence weights** the design nests under COLOUR (`design/Cartalith Menu Structure v2.dc.html`: "+ Field influence weights - Biome / elevation / moisture / geology"; `TERRAIN_APPEARANCE_RESEARCH.md` SS17 lists the same four) are weights *on the grade*, not axes: `render::build_grade_influence` reduces each field to a `0..1` per-cell signal (relative land elevation, rainfall, `BIOME_VEGETATION_COVER[classify_biome(t,m)]`, the lithology palette's lightness), centres it, and sums to one multiplier per pixel scaling every axis' departure from rest. All four at rest return an **empty** buffer and the grade is byte-identical to six-axis; a weight with no grade under it is the identity, which is why `grade_is_identity()` ignores the four. Both call sites pass it, so screen and export cannot disagree. In the dock as an adjacent **Grade field influence** group (no in-group nesting in this shell). **Still owed**: free colour pickers for the two tints — they remain a blue-to-amber axis. |

### The three controls that are not ports

- **`relief_chroma`** answers the owner's second point. The reference's relief
  blend is `grey = 185 * light`, and a `bio_blend` under 1 lerps toward it —
  costing **value as well as chroma**, dragging every shaded pixel toward one
  neutral, which is why the shipped `0.90` reads faded rather than lit. At
  `relief_chroma = 1` the grey target is a grey of *the pixel's own* luminance
  (the blend is exactly a desaturation) and the light factor cools and slightly
  desaturates shadow while warming and slightly saturating sun. `0.0` is the
  reference byte for byte.
- **`biome_sat`** — chroma of the material mix about its own Rec.709 luma, so it
  never moves one material lighter or darker relative to a neighbour.
- **`haze_strength`** — the reference's `0.18` literal, made adjustable. The
  haze colour `(208, 218, 230)` stays the reference's: it is the sky, not a
  taste.

### Named looks - how the shipped default moved without touching the parity path

`TerrainAppearance::js_reference()` is `Default` with the stage gates zeroed, so
**changing a palette in `Default` changes the JS-parity path**, and
`golden_parity_render.rs` is not re-baselineable (`DECISIONS.md` 7a's carve-out
covers paths where JS parity is *impractical*, and says the CPU rendering port
stays golden-verified).

So the re-pitched palettes, enabled stages and grade live in a **named look**
over the quality tier — `LOOK_PRESETS` / `TerrainAppearance::with_look`, bound
as `list_looks`/`get_look`/`set_look` — and `WorldGen` opens on **Natural
Vibrant**. The tier decides what the renderer *spends*; the look decides what
the picture *is*; a phone answers only the first differently, so a look never
touches a radius, a light count, or a stage a cheap tier switched off.

Three looks: **Quality tier** (the identity), **Natural Vibrant** (the new
default), **Antique Parchment** — section 7 of the owner's brief, the warm
hand-illustrated MapEffects-style plate. It **refines rather than duplicates**:
the existing Antique Map-style chip was `{"sepia": 0.35}`, a toning matrix over
the muted base, and now names this look too, so Antique is a warm aged sheet
*and* the sepia. Every Map-style chip carries a look beside its Painter bundle,
which lets Ink draw pen lines over the vibrant base. `reset_appearance()`
deliberately does **not** clear the look — it has its own picker, and a button
elsewhere silently moving that picker is the desync this register keeps fixing
one control at a time.

### Where the specification and the port's state disagreed

The numbers were written against the reference, where every enhancement slider
is `0`. Three are not zero here; only two moved:

- **Geology 25 %** — this port's `litho_strength`/`litho_exposure` have been
  `0.62`/`0.55` since milestone 5, *more* than asked; lowering them would make
  the vibrant look less geological than the plain tier. **Left at the tier's
  values.**
- **AO 20 %** — **down** from the tier's `0.28`, as specified; coherent because
  crests, curvature and ridged relief now carry local relief the cavity map
  carried alone.
- **Wetness 12 %** — down from the tier's `0.38` (itself set by the same day's
  owner-authorised CA-11 retune), because this instruction is the later one and
  names the number.

### Verified

Non-headless, 2048x1311 (seed 483920). Quality tier → Natural Vibrant: **73.29 %
of pixels moved, mean chroma 48.67 → 63.37 (+30 %), mean luma 139.61 → 138.36
(unchanged), luma sd 42.71 → 48.44 (+13 %)** — "richer, more dimensional, still
physically grounded" as a measurement, nowhere near the 2x of the rainbow biome
map the owner named as the failure mode. Every grade parameter at rest returns
the base at **0.0000 % moved, worst 0 levels**. Through the real dock: the Base
look picker opens on Natural Vibrant with the right chip lit, 35 appearance rows
draw, a real drag on Colour grade > Saturation moves 98.5 % of the raster, the
Antique chip lands look and sepia together, and a saved look round-trips the new
fields at 0.0000 %.

**Disclosed rather than tuned away**: surface texture and ridged relief are
nearly invisible at the specified 18 % and 10 % — the reference's own
arithmetic (`1 + 0.2 * k * (T - 0.5)` at `k = 0.18` is a ±1.8 % modulation,
about 2.5 levels on a 140-luma pixel), not a porting error. Left at the
specified numbers; their sliders reach real strength.

### Verified again, in the export (2026-08-24)

The block above measured the **viewport** only. The export raster, a commit
later, carries the grade in the same slot, but the obvious proof cannot see it:
the grid-resolution byte-for-byte probe passes under `Natural Vibrant`, whose
grade is the identity, so `apply_color_grade` early-returns on both sides.
Re-run under **`Antique Parchment`**, the one shipped look that grades,
non-headless at 2048x1312 (seed 20260824):

| | worst | of 8,060,928 bytes |
|---|---|---|
| Natural Vibrant, export vs viewport | 1 level | ~16 |
| **Antique Parchment, export vs viewport** | **2 levels** | **10** |
| Antique with the grade zeroed, export vs viewport | 1 level | 9 |

The grade isolated (six axes zeroed through `set_appearance`, look otherwise
unchanged): **87.85 % of bytes moved, mean 4.23 levels, worst 16 — identical for
export and screen.** Row two's worst of 2 is row three's worst of 1 amplified by
Antique's `+0.08` contrast (slope 1.064) across a `floor` boundary, not a second
defect; the probe asserts that relationship rather than a loosened number. Two
tests in `bake_raster.rs` hold both halves offline. See `CHANGELOG.md`, *"The
graded export was right, and nothing could have told you"*.

---

## 35 · KV-01, KV-02, KV-03 — the Markdown vault reached the shell, and continents became addressable on the way (2026-08-24) — **NEW SURFACE**

Not a gap this register found: a **new subsystem** the owner scheduled on
2026-08-24, recorded here because it adds three connected surfaces, and because
its opening audit found an entity that did not exist. `MARKDOWN_VAULT_SCOPE.md`
is the scope document; this is the register's view of it.

| id | Surface | Where | Status |
|---|---|---|---|
| **KV-01** | Place editor ▸ **Knowledge** — linked-note count, worst status, and the affordance that opens the vault panel scoped to this settlement | `place_editor_window.gd` | **Connected.** Keyed on `tid`, not on the array index. |
| **KV-02** | CIVIL ▸ Politics ▸ **Linked notes** — every province and every continent, each with its own link count and status | `civilization_workspace.gd` | **Connected.** |
| **KV-03** | **Markdown vault** panel — connect, browse, attach (whole document or one heading), the working copy, the previewed section write-back, the Cartalith block, and author-field population | `vault_window.gd` | **Connected.** |

**The entity audit** (`ROADMAP.md` required it before any code) changed the plan:

- **Settlements** — real, and the strongest: `NamedSettlement::tid` survives a
  rename, a move and a neighbouring deletion.
- **Provinces** — real. `Province::id` is sequential over the seed order and
  re-derived by `civ_recompute()`.
- **Continents** — **did not exist.** The roadmap's "world structure
  archetypes" is `generate_continentality_field`, a per-cell scalar with no
  identity, name or boundary. What did exist is `build_landmass_quality`'s
  golden-verified 8-neighbour flood fill, whose `comp`/`sizes`/`count`
  bookkeeping was "kept for… later milestones" and which `compute_civilisation`
  had computed and discarded on every generate since Phase 2.
  `cartalith_civ::civ_continents` keeps it (rank by area, a name, a bounding
  box, a centroid, a plurality faction); `WorldGen::get_continents()`.
- **POIs** — confirmed absent, and **not built as a side effect**; CV-01's entry
  stands. `EntityKind` has three variants and no `Poi`, and
  `MARKDOWN_VAULT_INTEGRATION.md` §35's criteria 6 and 7 are recorded as
  unsatisfiable in this port rather than faked.

**Two live-run findings**, both invisible to unit tests:

1. Continent 1 and settlement 1 got the **same name** in a real world, because
   `civ_name_rng`'s seed is a fixed reference quirk and both drew its first
   value. Continents now have their own stream (`civ_continent_name_rng`), with a
   test named after the failure.
2. `String(d.get("cells", 0))` in the new Politics rows — GDScript has no
   `String(int)` constructor, so the dock threw on every rebuild. Caught by
   driving the real shell.

**Deliberately not touched**: `DCC_SHELL_SPEC.md` §9's *MARKDOWN VAULT · LINKED*
block in the Data manager. It assumes `obsidian://` links in exported tiles,
note links inside exported GeoJSON, and a **two-way sync toggle** —
`MARKDOWN_VAULT_INTEGRATION.md` §33's explicit V1 non-goal.
`DCC_CONTROL_INDEX.md` records the conflict and the design's own header resolves
it: the block stays deferred, and the vault path / note-count readout is the only
part consistent with V1. Nothing in this pass writes an `obsidian://` link, a
wikilink or a block reference.

**What KV-03 still owed** (each also in the panel's own footer): the map snapshot
(§21 — a crop of the live renderer at three radii, held as its own milestone
rather than shipped as a broken image link); Compare-with-source (§14 — no diff
widget, so a stale source offers Reload or Keep, the two actions that cannot lose
work); project-scoped links (§26 — blocked on the save format carrying a civ
layer at all, which `SAVEFILE_COMPAT.md`'s format does not); and the Android SAF
provider. See `MARKDOWN_VAULT_SCOPE.md` §5 and §8.

---

## 36 · RD-02, CA-15 — every land way was the same colour, and the way-type filter could not see two thirds of the network (2026-08-24) — **FIXED**

§29 and §33 fixed the *geometry* of ways, sea lanes and committed routes. This is
the matching pass over *type and colour*: `drawCivLayer` §2a/§2b (reference lines
15494-15560) against `map_overlay.gd`, driven live and measured in pixels. Two
real defects.

### RD-02 — five land way types, one colour

The reference's §2a is a six-branch ladder, and every branch strokes **twice**: a
dark underlayer, then the type's colour on top — solid for the two trunk tiers,
dashed for the three minor ones. That is what makes a highway, a track and an
ancient way tell apart at a glance. This port drew land ways with **one** stroke
in one flat colour, only the width varying by type. Its class doc admitted half
of it (*"this control strokes every land way in `ROAD_COLOR` regardless of
type"*) as a note on the `ancient` row, so it read as one type's cosmetic
shortfall rather than the whole ladder missing.

**Measured before the fix.** `_waycolor_probe.gd` drives the real
`_draw_way_segment` over pure black and pure white, which makes colour and
effective alpha exactly recoverable (`b = C·a`, `w = C·a + (1−a)` ⇒
`a = 1 − (w − b)`, `C = b/a`), at `set_camera_zoom(0.2)` so `_crisp_begin()`'s
`1/k` transform draws every width 5× thicker and the stroke centre is fully
covered. All five land types returned the identical `C = (91, 75, 40)`,
`a = 0.549` — `ROAD_COLOR` exactly, five times.

`WAY_STYLE` now carries the reference's five branches verbatim. Measured after,
same probe, against the composite the reference's literals predict:

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

Every type within 0.6/255 and 0.002 alpha. Dash *periods* from the same shots
(the on/off split is not measurable — an antialiased cap bleeds ~width/2 past each
end — but the period is): road 15 px vs. 15.5 expected, track 16 vs. 16.5,
ancient 19 vs. 19, sea lane 23 vs. 23, route 40 vs. 40; highway/regional flat
solid (row range 0.000).

**The sea lane, route and selected route were already two-stroke** (§33, IN-09);
only the five land types were flat. One real bug in the sea lane's numbers: its
gap was **2.6, not 2.0**. `_draw_dashed_polyline`'s `gap_len` defaults to
`dash_len`, the sea lane was the one caller relying on that default, and the
reference's pattern is `setLineDash([2.6·rsc, 2·rsc])` — measured period 26 px
against the reference's 23. Every caller now passes the gap explicitly; no pattern
in `drawCivLayer` is actually square.

### Layering, and what it is not

The committed-route layer draws after both network layers (§IN-09 checked one
case). Re-verified by measurement: a route committed **along** the world's longest
highway (552 km solved over a 609 km host, 0 unreachable legs) — at all 13
coincident pixels the rendered colour is explained by the route's composite, not
the host's, with the route's least-squares residual lower at every one. The other
pair orders are correct by construction: sea lanes and land ways never contest a
pixel (different getters, water vs. land), and within `_roads` the reference draws
in array order with no per-type z-order, which this matches.

### CA-15 — the way-type filter listed the manual vocabulary, not the drawn one

`cartography_workspace.gd`'s **Ways · by type** group listed `road`, `track`,
`ancient` — `infra_tools_bridge::parse_way_type`'s *manual* vocabulary (IN-05's
four, minus `sea_lane`). But the switches drive `map_overlay.gd`'s filter on
`get_roads()`' `way_type`, and the generated network classifies by
`cartalith_civ::WayType`, whose two busiest tiers, `highway` and `regional`, were
not listed. Measured on a real 384×288 world: **13 highways and 17 regional roads
against 4 roads and 1 track** — unchecking "Roads" hid 4 of 35 ways and the other
30 could not be hidden. The reference lists all five (`CIV_WAY_TYPES`, line 14743).

`WAY_TYPES` is now that list minus `sea-lane` (which keeps its own top-level layer
row rather than a second, disagreeing switch). Verified live by toggling each type
and counting vanished pixels along a way of that type: highway 284, regional 241,
road 102, track 66.

### Cataloguing — checked, and correct

INFRA's lists were audited against `_civRenderWayList` (reference 17145) and found
honest: a per-tier tally, a "longest" ranking with the type in each label, a
**Sea lanes** group, a hand-drawn-ways group, and the editable **Routes committed
this session** list. The reference marks type with a per-row emoji
(`_wayTypeIcons`) where this uses the type word; same information. Unchanged.

### Files, and how it was verified

- `map_overlay.gd` — `WAY_STYLE` replaces `ROAD_COLOR`/`ROAD_WIDTH_BY_TYPE`;
  `_draw_way_segment` takes a style row and strokes twice; the sea lane passes its
  gap explicitly.
- `cartography_workspace.gd` — `WAY_TYPES` gains `highway` and `regional`.
- No Rust: `get_roads()` has emitted `highway`/`regional`/`road`/`track` since
  Phase 2 milestone 14 and `ancient` since IN-02. Entirely a renderer and a
  filter list.
- `_waycolor_probe.gd` (synthetic, exact) and `_wayreal_probe.gd` (real app:
  generation, a route committed along a road, per-type centred views,
  background-differenced pixel solving, the layering and filter checks). Both
  temporary and untracked.

---

## 37 · The left-rail menu structure v3 pass — fifteen new IDs, and what the three rails became (2026-08-24)

`design/Cartalith Menu Structure v3.dc.html`, vendored at `8cef062`, revises the
left-rail domain menus. The owner scoped implementation to **those menus only**:
the top bar stays, and v3's top-level `Vault` menu goes into the existing **Data**
menu (owner, verbatim: *"the vault menu can be shoved into data"*).
`DCC_SHELL_SPEC.md` carries the supersession disclosure (top-of-file notice plus
inline blocks at §3, §5 and §7); this section is the register's half.

### What the three rails became

| Domain | Before | After (v3's own list and order) |
|---|---|---|
| WORLD | a `GENERATION PIPELINE \| SCULPT` mode switch over a numbered ten-stage accordion | 9 categories: Generate · Terrain · Geology · Hydrology · Climate · Biomes · Ecology · Resources · World data |
| CIVIL | 6 categories + INFRA's 5 appended below a rule | 14 categories: Civilizations · Factions · Territories · Settlements · Points of interest · Routes & ways · Travel · Trade · Economy · Culture · Politics · Military · Relationships · Simulation |
| CARTO | 3 categories + RENDER's flat run of sections appended below a rule | 10 categories: Map style · Terrain appearance · Colours · Layers · Roads & routes · Labels · Assets & landmarks · Political display · Visibility / zoom · Map presets |

**Nothing was rewritten.** Every builder is the one already there, called with a
different parent: `InfrastructureWorkspace` and `RenderWorkspace` gained
`build_*_into()` entry points and a flag that stops them drawing categories of
their own, and `world_workspace.gd`'s `_build_stage()` became
`_build_stage_body()`, drawing the same content into a section. v3's closing rule
(*"every #id keeps its wiring — this is re-parenting, not rewriting"*) held.

Three renames the rest of the shell had to follow, found by grepping the old names:
the timeline strip's hint and its `Open Timeline` tooltip (CIVIL ▸ Timeline →
Politics / Simulation), `layers_popover.gd`'s footer (the political and way-type
switches left Cartography ▸ Layers), and three "World ▸ Generation Pipeline"
pointers in `new_world_dialog.gd` and `tool_bar.gd`.
`DccWidgets.stage_category()` lost its only caller and is marked as such in place
rather than deleted. (Six more stale pointers were missed here; see §38.)

### The fifteen new IDs

Each is a row v3 draws that this port had nothing behind. All fifteen shipped as a
disclosed note or a disabled control carrying its reason — none drawn as working,
none silently omitted. The *reason* column is the record of what was believed on
2026-08-24; where later sections proved it wrong, the class cell says so and
points there.

| # | Row v3 asks for | Where it is disclosed | Why there is nothing behind it | Class |
|---|---|---|---|---|
| **CV-21** | Faction **identity colour** and emblem | CIVIL ▸ Factions ▸ Not built | `FactionRoster` stores no colour field, and `map_overlay.gd` derives a faction's tint from its *index*. v3's own CIVIL-owns-the-colour / CARTO-owns-the-paint split has neither half | **CLOSED 2026-08-25 (§39)** — the reason was wrong: the roster *did* store a colour field, and nothing read it |
| **CV-22** | Faction **history, notes, lore** (v3 marks these `vault`) | CIVIL ▸ Factions ▸ Not built | `cartalith-vault`'s `EntityKind` covers settlement, province and continent. A faction is not addressable there yet | **CLOSED 2026-08-25 (§39)** — the estimate was exact |
| **CV-23** | Borders, claims and **influence** as separate quantities; historical occupation | CIVIL ▸ Territories ▸ Not built | `CivData::territory` is one plurality-owner-per-cell grid: no contested-claim value, no influence field, no per-year ownership record beyond the timeline's settlement snapshots | **NARROWED 2026-08-25 (§41)** — influence and claims built on demand, retained nowhere. Open: **historical occupation over time** (timeline work) |
| **CV-24** | The year scrubber as **program scope** (v3: *"time is not a domain"*) | The timeline strip's own `Open Timeline` tooltip | Agreed in principle, and not moved. `dcc_shell.gd`'s reserved `timeline_bar` is one fixed-height `HBox` with no room for a year-pill list, an add-year field and three filter checkboxes; `TIMELINE_SCOPE.md` §4's standing instruction is to build a dedicated panel rather than guess the region. A shell-frame change, not a menu change | (C) — design first |
| **CV-25** | **Military**: garrisons, defensive strength, fortification network, campaigns | CIVIL ▸ Military ▸ Not built (whole category) | `cartalith-civ` models none of them and neither does the reference. New design, not a port gap. What exists is per-settlement *defensibility*, a terrain heuristic, on the right dock | **NARROWED 2026-08-25 (§40)** — the reason was wrong: the reference has the fortification half and `power.military` was already ported. Open: **garrison headcounts, campaigns, unit movement, combat** |
| **CV-26** | **Relationships**: diplomatic matrix, allies/rivals/subjects, treaties — and v3 Politics' vassalage/alliances/rivalries | CIVIL ▸ Relationships ▸ Not built (whole category), and CIVIL ▸ Politics ▸ Not built | There is no edge between two factions to hold a value, at any year, so a matrix would be a grid of blanks. The reference has none either. Absorbs the one-line gap the old Politics category disclosed | **NARROWED 2026-08-25 (§40)** — the reason was right, and the edge now exists (`cartalith_civ::relations`). Open: **diplomacy actions, treaties, vassalage, and change over time** |
| **IN-13** | **Trade flows** as a routed quantity, imports/exports per settlement, route-cost field, trade-influence raster | CIVIL ▸ Trade ▸ Not built | `civ_resource_trade_balance` produces the hinterland surplus/deficit that *is* shown; nothing ties a trade relationship to the way that would carry it. `ECONOMY_SCOPE.md` holds the aggregation | **CLOSED 2026-08-25 (§42)** — the reference's own supplier match and way-network union-find, run over all fifteen resources (`cartalith_civ::trade`). Open, and disclosed on screen: prices, tariffs, caravans as entities, trade over time |
| **CA-16** | Per-class way **style**: colour, width, casing, dashes, route glow | CARTO ▸ Roads & routes ▸ Not built | `map_overlay.gd` draws every way from one hardcoded width-and-colour pair per type and takes no style argument; the reference's `#civWayScaleR` has no counterpart here. Visibility, which *is* wired, is the whole of what works | **CLOSED 2026-08-25 (§39)** — `#civWayScaleR`/`#wayOpacityR` ported; the reason describes the file as it was *before* §36 |
| **CA-17** | Territory tint opacity, border width/style, claim hatching, influence gradient + legend | CARTO ▸ Political display ▸ Not built | One fixed-alpha fill with a fixed border, and no style record keyed to a faction id for anything to write to. Blocked on CV-21 at the CIVIL end | **CLOSED 2026-08-25 (§39)** for tint opacity (`#territoryOpacityR`) and, via CV-21, identity colour; the rest is CV-23's data gap |
| **CA-18** | **Zoom ladder** (what appears when) and the declutter budget | CARTO ▸ Visibility / zoom ▸ Not built | No per-layer zoom range exists anywhere in the shell; the one zoom-dependent behaviour is the urban-layout reveal band, which `map_overlay.gd` hardcodes. Label/icon collision is not resolved at all — overlapping annotation simply overlaps | **PARTLY CLOSED 2026-08-25 (§39)** — `CIV_LOD_ROAD` ported; the declutter budget and per-layer ranges stay open |
| **CA-19** | **Biome colour table** | CARTO ▸ Colours ▸ Colour grade note | `CART_BIOME_COLS` is a frozen reference table compiled into `cartalith-render` with no `#[func]` to read or rewrite an entry. The four field-influence weights beside it are live | **CLOSED 2026-08-25 (§39)** for *reading* (`debug_layers()` carried all fifteen classes all along). *Rewriting* is a separate, larger item — engine half landed with Ruling P (`cc0f561`); the picker 2026-09-24; the legend / Biomes field / paint preview reading the override is built in `sample_bridge.rs`/`paint_bridge.rs` and needs its four `lib.rs` call sites switched (status in `STATUS.md`) |
| **WW-14** | **Ecological productivity**, flora/fauna distribution | WORLD ▸ Ecology ▸ Not parameterised (whole category) | No crate computes either, here or in the reference. Vegetation density and soil *are* computed — derived off biome/climate/lithology with no dials — and are readable as analysis fields; the note points there | **CLOSED 2026-08-25 (§39)** — the reason was wrong on **both** halves: `build_npp` and `cartalith_civ::wildlife` are real and golden-verified |
| **WW-15** | **Coordinate system · projection** | WORLD ▸ World data ▸ Read the fields | Every field is grid-space, the GeoJSON export writes a plain lon/lat-shaped frame with no CRS declared, and nothing reprojects. Units are km-only (PR-15) | **CLOSED 2026-08-25 (§39)** for the frame — a CRS *is* declared in the document's `note`; `world_crs()` reports it in-app. Reprojection stays open |
| **VA-01** | **Backlinks · unlinked mentions** | CIVIL ▸ Settlements ▸ Not built, and Data ▸ *Missing & orphan notes…* | Both need a reverse index over the whole vault. The provider deliberately opens only the files it is asked for, which is what keeps a large vault cheap to browse and is exactly what an unbounded scan would undo | **CLOSED 2026-08-25 (§42)** — a false pair: a `stat` is not a read, so the index is persisted **and** kept correct per file by `(modified, len)`; mentions filtered by a 64-bit word fingerprint |
| **VA-02** | **Create notes from template**, path convention `Settlements/{name}.md` | CIVIL ▸ Settlements ▸ Not built, and Data ▸ *Create notes from template…* | `cartalith-vault` attaches to notes that already exist and refuses a heading that does not — deliberately (`MARKDOWN_VAULT_SCOPE.md` milestone 1's boundary). There is no note *creator* and no template registry, so the owner's own `design/vault-templates/` cannot be instantiated | **CLOSED 2026-08-25 (§39)** — the boundary quoted is about *editing*, and creating a file cannot destroy one |

`VA-` is a new prefix: the vault's *gaps*, distinct from `KV-` in §35, which
records what the vault subsystem **connected**. Nine of the fifteen closed on
2026-08-25 (§39), four of them because the capability already existed.

### What was wired to real capability, not disclosed

- **CIVIL ▸ Routes & ways / Travel / Trade** — INFRA's Roads, Ports, Logistics
  and Trade content, re-parented under v3's names with every live readout intact
  (per-tier tally, longest-ways ranking, sea lanes, hand-drawn ways, the
  committed-routes list, the journey list and planner). **Rivers** moved to WORLD ▸
  Hydrology, where v3 puts the river network, with its one honest finding (IN-01,
  no `get_rivers()`) — or was meant to; see §38.
- **CIVIL ▸ Territories** — *Recalculate territories* and *Generate provinces*,
  both live shortcuts onto `civ_recompute()` (§31's CV-20 fix), under a category
  named for what they do.
- **CIVIL ▸ Settlements ▸ Linked notes** — the vault entry point, scoped to the
  roster above it, keyed to `tid`.
- **CARTO ▸ Visibility / zoom ▸ Data overlays** — one button onto
  `layers_popover.gd`, with a live count of what `debug_layers()` offers.
  Deliberately *not* a second copy of that list: two pickers over one
  `set_debug_layer()` is the shape this shell keeps having to undo.
- **WORLD ▸ Generate** — Generate world / New seed / Center landmasses as
  shortcuts onto `app.gd`'s own handlers, beside the pipeline-status readout.
- **Data ▸ Markdown vault** — v3's `Vault` menu, folded into Data per the owner.
  One live row onto `vault_window.gd`, whose tooltip states where v3's
  frontmatter-mapping and sync-direction rows actually live (a per-write choice
  in the panel, not a global setting), plus VA-01 and VA-02 as `_todo` rows.

### Verification

`_v3menu_probe.gd` / `.tscn` (temporary, untracked), run **windowed** against a
real world (384×288, seed 483920: 233 settlements, 8 provinces, 35 ways):

1. Each rail's L2 list is **exactly** v3's list in v3's order, and none of the
   nine retired category names survives. The second check caught the first cut
   building INFRA's five old categories *and* the three new ones: `_dock_hosted`
   was set inside `build_ways_into()`, which runs **after** `setup()` (which runs
   `_build()`). Both flags now go in before `setup()`, next to `_nested`.
2. All 33 categories opened and each drew something — a category that throws while
   building leaves an empty body that looks exactly like a closed one.
3. Every disabled control carries a reason (the `_todo()` contract), with the four
   state-gated Sculpt/Paint Commit/Discard buttons named as explicit exemptions.
4. The rows that claim capability driven: the Politics/Simulation split (years
   under one, the simulator under the other, neither under both), Territories'
   recompute pressed for real, the Layers/Political display split, and
   `Data ▸ Markdown vault` pressed through the real popup with the vault window
   asserted on screen.
5. Screenshots of each rail fully open, then each re-parented category alone.

**PASS, 0 failures**, plus a visual pass. `cargo check -p cartalith-godot` clean —
no Rust changed.

The parse check caught one defect first, and it is why this pass was resumed
rather than restarted: `_build_simulation()` assigned an undeclared `_sim_body`,
left mid-refactor by a session-limit kill. The fix is not the declaration —
`_rebuild_timeline()` now refills **both** bodies, each guarded independently, so
the order `_build()` claims them in cannot leave one empty.

## 38 · FR-02, PE-01, SH-11, WW-13 — three of the four were a control that lied about its own state (2026-08-25) — **FIXED**

A conformance sweep over the windows and the shell's cross-references, by driving
the shipped app. Four fixes, two of them defects nothing had recorded, plus a
class of stale pointer left by §37's re-parenting.

### FR-02 — selecting a faction renamed it

**The worst one, and silent.** `faction_roster_window.gd`'s inspector commits the
name field on `focus_exited`. Removing a focused `Control` from the tree releases
focus and fires that signal **synchronously**, so `_clear()` — the first act of
every rebuild — was itself an edit. The list rows are `FOCUS_NONE`, deliberately,
so clicking one does not take focus off the name field, and their handler sets
`_selected = fid` *before* `_rebuild_inspector()`. So the teardown wrote the
**previous** faction's name onto the faction just selected. Measured on a real
6-faction world:

| | roster |
|---|---|
| before | `1:Aurelia, 2:Veldmark, 3:Korrath, 4:Sythe Dominion, 5:Mirelle, 6:Draumr League` |
| after clicking Veldmark with Aurelia's name field focused | `1:Aurelia, `**`2:Aurelia`**`, 3:Korrath, 4:Sythe Dominion, 5:Mirelle, 6:Draumr League` |

No prompt, no status line, no undo. Two factions called Aurelia.

### PE-01 — the place editor's ⟳ never took on its first press

Same mechanism, visible instead of destructive. `DCC_SHELL_SPEC.md` §4.5.3 has
`open_for()` focus the name field, so on desktop it holds focus all session. `⟳`
re-rolls, then rebuilds; the teardown wrote the pre-roll name back before the
rebuilt field read the new one. Isolated three ways on one settlement, separating
the mechanism from the engine:

| path | result |
|---|---|
| A — open (focused), one press | `Yusnashharwell` → `Yusnashharwell` (no-op) |
| B — open, `release_focus()`, one press | `Yusnashharwell` → `Abedomarmarch` |
| C — `civ_reroll_settlement_name()` direct, ×10 | ten distinct names, every one read back correctly |

Only `open_for()` grabs focus, so **presses two onward worked** — which is why a
probe found this and the eye did not. The same file's history `TextEdit` had the
cross-entity form: `open_for()` sets `_index` and only then rebuilds, so
re-opening the editor on another place committed the old form's text onto the
new one.

**The fix is two halves, because a guard alone silently drops real edits.** A
`_rebuilding` flag set across `_clear()` and checked by every `focus_exited`
commit in both files stops the teardown write; `_commit_focused_field()`, called
before the id moves, releases focus so a pending edit lands on the entity it was
typed for. Verified: the roster is unchanged across a selection switch, the first
⟳ renames, a sentinel typed into settlement 6's history does not appear on
settlement 7 — **and settlement 6 keeps it**, so the flush works and the guard is
not swallowing input.

The shell's other five `focus_exited` commits were checked and are safe:
`cartography_workspace.gd`'s two capture `idx` in the closure;
`asset_library_window.gd`'s pack fields are never rebuilt under the caret;
`travel_library_window.gd`'s writes to a local draft; `vault_window.gd`'s reader
is not id-switched.

### SH-11 — the zoom pivot, closed

Open since 2026-08-24, deferred only for file contention. `_zoom_at()`'s maths was
right; its two `_input` callers handed it the wrong space. `InputEvent.position`
is window-relative, `_camera.position` is `ViewportHost`-relative, so the pivot
was out by `global_position * (1/z0 - 1/z1)`. Measured: `global_position`
(412, 70), one wheel notch (zoom 1.6727 → 1.9236), **32.59 px of drift** — the
*same* (32.13, 5.46) at three probe points, the signature of a constant offset and
exactly what that formula predicts. `zoom_step()`, which passes `size * 0.5` and
was always local, measured **0.00 px** and is untouched. After the fix: 0.00 px at
all three points.

### WW-13 — Paint Commit / Discard, closed

Open since 2026-08-24, deferred to keep that commit off a fourth file. Both buttons
gated on `paint_painted_counts()["total"]` — committed *and* pending — which a
commit does not change, so both stayed live over an empty draft. New
`PaintEditor::pending_stamps()` / `paint_draft_count()` counts what `commit_all`
would bake and `discard_all` would throw away, across all three drafts (a layer
switch does not discard the layer left behind). Two Rust tests pin the
divergence. The WORLD dock's and the tool-options bar's Commit draw over one
draft and are on screen together, so each now refreshes the other — otherwise
WW-13 reappears one control over. Measured, driving real dabs:

| state | pending | composite | dock Commit | dock Discard | bar Commit |
|---|---|---|---|---|---|
| before any dab | 0 | 0 | disabled | disabled | disabled |
| after 2 dabs | 2 | 70 | enabled | enabled | enabled |
| after dock commit | **0** | **70** | disabled | disabled | disabled |
| after a further dab, committed from the tool bar | 0 | 74 | disabled | disabled | disabled |

The unchanged composite across a commit is the premise, asserted rather than
assumed.

### Six stale pointers, and one disclosure that vanished

§37 renamed nine categories and caught three follow-up renames; six more were
found here by extracting **every** rendered `A ▸ B` string and checking it against
the shipped structure:

| where | said | is |
|---|---|---|
| `app.gd` atlas-cache tooltip | `WORLD ▸ Finalize` | `WORLD ▸ Generate ▸ Finalize` |
| `menus.gd` ×2 (comments) | same | same |
| `right_dock.gd` sculpt stamp empty state | `World ▸ Sculpt` | `World ▸ Terrain ▸ Sculpt` |
| `place_editor_window.gd` polity tooltip | `Politics ▸ Recalculate territories` | `Civilization ▸ Territories ▸ …` |
| `infrastructure_workspace.gd` way **and** route commit toasts | `Roads ▸ Hand-drawn` | `Civilization ▸ Routes & ways ▸ Hand-drawn` |

The toasts mattered most: they fire at the moment a user goes looking for what
they just drew.

**And `rivers_note()` had no caller at all.** §37 wrote it so IN-01 would travel
with Rivers to `WORLD ▸ Hydrology`, and §37 says it did. It did not: CIVIL stopped
drawing it and WORLD never started, so for a day the finding existed in the source
and nowhere in the app. It is `static` now, with one owner and one caller.

### The class behind them: a jump button that only did half its job

Every `→ Civilization ▸ Territories`-style button switched the rail and stopped.
Survivable at six CIVIL categories; v3 gave it fourteen and CARTO ten, in a
one-open-at-a-time accordion. `Workspace.open_category()` and
`DccShell.select_domain_category()` do both halves and `push_warning` on a title
that no longer exists — so the next rename is loud.

### Two disabled controls with no stated reason

A scan of every window found two greyed controls that did not say why, both
disabled at build time and given their reason only by a refresh that had not run:
the welcome screen's **Open selected** (its primary action), and the asset
library's three anchor chips before any slot is selected. Both now carry the
general form of the fact.

### Verification

Six temporary, untracked probes (`_deadwire_probe`, `_pressall_probe`,
`_conform_probe`, `_jump_probe`, `_reroll_probe`/`_reroll2_probe`,
`_focusbug_probe`), all **windowed** against a real 384 × 288 world, seed 483920
(233 settlements, 6 factions, 35 ways).

- **Unwired-control scan** over 14 windows and the docks, counting alternate
  activation signals (`text_changed` for a live filter, `color_changed`,
  `toggled`, `item_activated`): **0 genuinely unwired** (the two reported are a
  `TabContainer`, which switches its own tabs, and a read-only `TextEdit`);
  **0 disabled-without-a-reason** after the two fixes above.
- **Press-every-enabled-button**, one window at a time, snapshotting every rendered
  string before and after, destructive and blocking labels skipped by name. Of six
  no-change presses, five are false positives (an already-selected tab or nav row,
  and `CityViewer`'s Fit, which moves a canvas the text snapshot cannot see). The
  sixth was PE-01.
- **Menu accelerators** re-enumerated after v3: 11, each matching its label, none
  unreachable except `Ctrl+Z` over a legitimately empty undo stack.
- `cargo check -p cartalith-godot` clean; two new Rust tests pass; headless
  boot-check clean.

### Not fixed, and why

- **CV-24 and the rest of §37's fifteen** — unchanged at this pass. *(Nine closed
  the same day in §39; CV-24 still wants a design.)*
- **ED-02** (an undo *history panel*) stays (C): no design exists. *(§39
  re-checked it; §42 built it.)*
- **The Data manager's five silent nav rows** — `Heightmaps`, `World Data`,
  `Assets` and the two Export rows carry no tooltip, but each opens a pane that
  explains itself. Left alone rather than given filler.
- **§14.4's own status line is stale**, corrected here rather than in place:
  **JP-VS-01 is closed**, by §27's `open_journey_planner()` →
  `select_domain("civilization")`. Its entry still reads "still open".

## 39 · §37's fifteen, worked — nine closed, and four of those nine were already built (2026-08-25)

The owner's instruction was to implement §37's backable items. The session's
standing lesson — a capability the register calls missing keeps turning out to
exist and be unwired — was applied first: **every one of the fifteen was checked
against the crates before anything was written.** That check is the finding.
**Four of the nine closed had working engine capability already**, one more had
it in a form the register described inaccurately, two are new ports and two are
genuinely new code.

| # | Disposition | What was actually true |
|---|---|---|
| **WW-14** | **closed — was already built** | `build_npp` (Miami model) and `cartalith_civ::wildlife`, both golden-verified. §37 said "no crate computes either, here or in the reference" |
| **CV-21** | **closed — was already built** | `FactionEntry::color` existed and **nothing read it**. §37 said the roster "stores no colour field" |
| **CA-19** | **closed — was already readable** | `debug_layers()` always carried all fifteen `CART_BIOME_COLS` classes. §37 said there was "no `#[func]` to read or rewrite an entry"; *rewrite* is the real gap and it is not small |
| **WW-15** | **closed — was already declared** | The GeoJSON writes `geojson::CRS_NOTE` in its own `note` property, verbatim from the reference. §37 said "no CRS declared" |
| **CA-16** | **closed — reference port** | §37's stated reason had been fixed by §36. The real gap was `#civWayScaleR`/`#wayOpacityR` |
| **CA-17** | **closed — reference port** | `#territoryOpacityR`. The rest of the row is CV-23's data gap |
| **CA-18** | **partly closed — reference port** | `CIV_LOD_ROAD`, a real per-type zoom ladder, ported. The declutter budget and per-layer ranges stay open |
| **CV-22** | **closed — new, and exactly the size §37 estimated** | One `EntityKind` variant, two match arms, plus export rows |
| **VA-02** | **closed — new** | §37's reason was a boundary about *editing*, applied to *creating* |
| CV-23 · CV-24 · CV-25 · CV-26 · IN-13 · VA-01 | **still open** | Sharpened below. Three want a design decision, three want real engine work |

### The four that were already built, and why each was missed

One mistake produced all four: **§37 was written during a large UI restructure
and asked "does the dock have a control for this?", not "does the engine have the
quantity?"** (the same trap as `MISTAKES.md`'s *"Report that a feature does not
exist"* row).

**WW-14** is the worst, because the register asserted a negative about the
reference too. `cartalith_civ::build_npp` is the Miami model — the lower of a
temperature and a precipitation ceiling, both capped at 3000 g/m²/yr — golden-
verified since the wildlife port, computed **only inside `wildlife_regions`** as
one of the ecoregion scorer's five inputs, and thrown away. The fauna half is
`cartalith_civ::wildlife`'s connected-component segmentation of the biome grid,
with a guild roster and per-species population estimates, reachable only by
clicking the map while the Wildlife debug view happened to be open. (Measurements
in the Verification table below.)

**CV-21** is the sharpest illustration. `FactionEntry` carried a `color` field
since the roster bridge was written; only a unit test read it. The renderers went
to `lib.rs`'s `FACTION_RGB` by index — *inconsistently*:
`build_territory_texture` used `faction_rgb`'s no-wrap rule while the
Political-control field indexed `FACTION_RGB[(owner-1) % len]` directly, so on a
seven-faction world the field drew faction 7 in faction 1's colour and the map did
not. One `CivData::faction_rgb` is now the only path.

The override is a **second** field rather than a write into `color`, deliberately:
`color` holds the *reference's* `CIV_FACTIONS` table, which this port does not
render in (`FACTION_RGB` is Okabe-Ito, colourblind-safe, a divergence disclosed at
both ends). Writing into `color` would make the reference table the palette for
edited factions only — two rules in one roster. `color_override: None` is exactly
today's behaviour, so a world at rest is bit-identical.

**CA-19**: `debug_layers()`' `bclass` entry carries `(r, g, b, label)` for all
fifteen classes — the biome colour table — and `get_paint_palette` plus
`paint_bridge::swatch_color` read the same constant. CARTO ▸ Colours now points at
that one legend rather than drawing a second copy. A **writable** palette is not
small: `render.rs` is `#[path]`-included standalone by five test targets so it
cannot reach shared mutable state, `CART_BIOME_COLS` is what a painted biome cell
blends toward in `land_color`'s hot loop, and `paint_blend.rs`'s goldens are
written against the frozen values. Scope: a palette field on `RenderCtx` threaded
to `land_color`, a runtime table on `WorldGen`, and re-baselined goldens.

**WW-15**: RFC 7946 **deprecated** the `crs` member, so a `note` is the
declaration a GeoJSON file gets to make, and this port always made it (`CRS_NOTE`,
verbatim from the reference). What was missing was any way to read the frame *in
the app*, and it is **two different frames**:

- **World mode** — the grid wraps in X and rows run 90°N to 90°S: a plate carrée
  graticule over a whole planet; `climate.lat_n`/`lat_s` are ignored, as the
  climate pipeline's own `lat_of(y)` says.
- **Regional mode** — rows run `lat_n` to `lat_s` and X does not wrap, so latitude
  is real and drives the climate model while longitude is not modelled at all.

`world_crs()` reports both, plus cell size, degrees per row and the export's note
(on the verification world: regional, 384 × 288 cells over 2400 × 1800 km). What
stays open is a **projection**: nothing reprojects, so the planar kilometres are
not a projection of the latitudes beside them — which the export's note already
warns.

### The two reference ports, and the one thing §36 had already fixed

**CA-16.** §37's stated reason described the file **before** §36, which had
replaced the flat `ROAD_COLOR` with the reference's five two-stroke styles. What
was genuinely absent is the reference's two per-layer style controls:
`#civWayScaleR` (line 1485), the third term of
`rsc = max(1, GW/512) · _civZoomK() · _civWayScale()`, and `#wayOpacityR` (line
1491), its `globalAlpha`. Both are the identity at their defaults, so the layer at
rest is unchanged. The scale multiplies dash lengths too, because the reference
writes one `rsc` into both (`setLineDash([1.8*rsc, 1.3*rsc])`) — a wider road gets
a proportionally longer dash.

Per-class colour, casing and dash stay unbuilt, for a sharper reason than "no
style argument": the five styles are *ported literals* whose job is to make a
track read as a track, and making one editable means a style record keyed by way
type for the overlay to read instead of its own `WAY_STYLE` constant.

**CA-17.** `#territoryOpacityR` (line 1490), applied at 15440 as
`Math.round(opacity * 255)`; this port had a hardcoded `82/255` in
`build_territory_texture`. The default deliberately stays this port's own, not the
reference's `130/255`: there is a hillshade, a splat and a colour grade under this
wash that the reference's flat biome fill has not, and a heavier tint buries them.

**CA-18, partly.** `CIV_LOD_ROAD` (line 15380, read by `_civWayLodMin` at 15012) is
ported verbatim. Its effect here is narrower and **deliberately not widened**:
`ViewportHost`'s `ZOOM_MIN` is 0.4, so `road`'s 0.35 threshold is unreachable and
only `track` and `ancient` ever drop out, between 0.4× and 0.7×. The two trunk
tiers are `0` — "always", not "missing". A switch the reference does not have was
added, because a per-layer zoom range whose effect you cannot see is
indistinguishable from a bug.

### The two that are new

**CV-22** cost exactly what §37 estimated: one `EntityKind::Faction` variant, one
`as_str` arm, one `parse` arm — what `links.rs`'s module doc reserved (*"adding a
variant here plus a `key()` case is that whole change"*) — plus the export-registry
rows. `ALL` gained a sibling `EVERY`, because a faction has no position and the
three *place* kinds do. §20's rule holds by construction: `entity_values` returns
an empty map for faction 0 (Unclaimed) and for an id past the roster. The three
vocabulary fields (Culture, Government, Religion) are exported deliberately:
`ECONOMY_SCOPE.md` found nothing in either codebase simulates Government or
Religion, which is the argument for writing them into a note, where an author's
own prose carries the meaning.

**VA-02.** §37's boundary is about **editing**: §23 makes the machine block the
only thing Cartalith rewrites unattended, because reshaping an author's prose is
the failure the design is arranged against. Creating a file that was not there is
a different, safe act — an existing path is refused outright — and the body is the
author's template copied verbatim with only the entity's name substituted.

**Templates come from the vault, not from the program.** No registry, no bundled
content: a `.md` with "template" in its path is a template, which is how the
owner's `design/vault-templates/` names its files (`Settlement Template.md`,
`Landmark template.md`, `Region Template/`). Compiling a template set into the
binary would be Cartalith telling an author how to write notes. `discover`
filters the same bounded listing the file picker already walks (no second walk,
still no file opened) and labels a nested template by its folder, because that
corpus has two byte-identical `Landmark template.md` files.

`fill_title` substitutes `{{…Name}}` and the literal `[Name]`, and leaves
`[If applicable]`, `[Optional]` and every other bracketed prompt alone — those are
instructions to the author. Path convention is v3's `Settlements/{name}.md`,
generalised per kind and editable in the field.

### Still open, sharpened

Each was checked against the crates too; what changed is what is known, not the
status. Every one was resolved further within the day, and the later section holds
the full account.

- **CV-23 — borders, claims, influence.** Still (B) large, but the influence field
  was being computed and thrown away: `assign_territory`'s `best_effective`
  (`cartalith-civ/src/lib.rs:6040`), the per-cell cost-distance to the winning
  capital divided by its population weight; a contested value is one more array,
  not one more Dijkstra. Two obstacles named: retaining an `f32` per cell is
  **268 MB at this port's 8192² ceiling** (the objection `civ_continents` already
  records), so it must be an on-demand recompute like `wildlife_regions`; and that
  needs `build_travel_cost`'s `cost` field, which `compute_civilisation` builds as
  a local and frees, keeping only per-settlement samples in `explanations`.
  Historical occupation is separately timeline work. *Every sentence held; built
  in §41, which rebuilds the `cost` field rather than retaining it.*
- **CV-24 — the year scrubber as program scope.** Unchanged and correctly (C):
  `TIMELINE_SCOPE.md` §4 says build a dedicated panel rather than guess the
  region, and `timeline_bar` is one fixed-height `HBox`. A shell-frame change;
  **owner's to specify.**
- **CV-25 — military.** Recorded as unchanged (C): *"neither `cartalith-civ` nor
  the reference models garrisons, fortification networks or campaigns"*. *Wrong —
  §40 found the reference models the fortification half twice over and this port
  had already ported `power.military`; only campaigns were ever absent.*
- **CV-26 — relationships.** Unchanged (C), for a structural reason: no **edge**
  between two factions to hold a value, so a matrix would be a grid of blanks; the
  reference's inspector says "not yet implemented" too. *Right, and it is what
  §40 built; actions, treaties and change over time stayed out.*
- **IN-13 — trade flows.** Still (B) large: `TradeBalance` is `{exports, imports}`,
  a per-settlement verdict on which of fifteen resources a place has too much or
  too little of *against the world mean* — it names **what**, never **who**. A
  flow needs a surplus-to-deficit match and a routing over the way graph, recorded
  here as existing in neither codebase. *Wrong on that last clause — both exist in
  the reference; closed in §42.*
- **VA-01 — backlinks and unlinked mentions.** Still (B) large; the open question
  is the **index, not the scan**: built on demand it stalls a large vault,
  persisted it is a second store to keep in step with a folder edited outside
  Cartalith. *A false pair — closed in §42.*

### Also carried over from §38

- **ED-02 (undo history panel)** — checked, **still (C)**: `undo.rs` keeps a
  `label` on every step (`undo_stats()` reports only the next), so a five-row list
  is data the engine already holds, but §7.1 asks for a *ledger* with
  per-subsystem reversal, a strictly larger thing. Building the five-row list
  would answer the easy half and foreclose the design question. *Built as the
  ledger in §42. Per-subsystem reversal stayed unbuilt; a `Recorded` row already
  knows its subsystem, so turning one on is a kind change rather than a redesign.*
- **The Data manager's five silent nav rows** — left alone again, agreeing with
  §38: each pane names itself in its first line, and a tooltip repeating the row's
  label is filler.

### Verification

`_gap37_probe.gd` / `.tscn` (temporary, untracked), **windowed** against a real
384 × 288 world, seed 483920 (233 settlements, 6 factions, 35 ways). **PASS, 0
failures.**

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

Plus `cargo test`: `cartalith-godot` 343 lib tests and its integration targets
green with one new roster test; `cartalith-vault` **41 → 48**, including one
end-to-end against a real folder proving the created note is attachable by the
ordinary path. `cargo check -p cartalith-godot` clean; headless boot clean.

The parse check caught one defect first: the first cut of CIVIL ▸ Factions added a
second *Faction roster…* button to a category that already had one, which
GDScript rejects as a redeclared local — two openers onto one window, the shape
this shell keeps undoing, refused by the language this time.

**A note on the commits.** WW-15's engine and shell landed in `79396c2` alongside
the other six display IDs; that commit's message does not name it, and this
section is the record.

## 40 · CV-25, CV-26 — the military half was a port nobody had recognised, and the relations half needed an edge that did not exist (2026-08-25) — **NARROWED**

Owner's decision on the two §37 IDs parked pending design: *"build a minimal
version now"*. Both are built; neither is closed, because each leaves a real,
separable feature — and both categories say so on screen, in the same words.
The two were completely different jobs, and that difference is the finding.

### CV-25 was a port, and the register's reason for calling it a design was wrong

§37 recorded *"`cartalith-civ` models none of them and neither does the
reference."* The second half is wrong — the **fifth** §37 entry wrong in exactly
that direction. Grepping the frozen snapshot for `military`, `garrison`, `war` and
`fortif` found three real implementations and one already-ported fourth:

| reference | line | what it is | status before today |
|---|---|---|---|
| `_umWallSpec` | 22109 | the `none · ditch · palisade · stone` ladder, from tier + function + threat + wealth + age + command of ground | **not ported** — `urban_adapter.rs`'s own table said "skipped: the whole fortification pipeline is milestone 10" |
| `_umInferWalls` | 22134 | its boolean view | **not ported**, same note |
| `_civPlaceDefensibility` | 23802 | per-settlement defensive strength `0..1` | **not ported** |
| `_civFactionAggregates` → `power.military` | 23716 | `0.45·normPop + 0.35·fortifiedFraction + 0.20·capitalTierNorm` | **already ported**, golden-verified, and with no reader |

So the "new design" was three small ports and a category to put them in.

**And a live defect the ports exposed.** `FactionPlace::from_settlement`
hard-wires `fortified: false`, because `cartalith-civ` is stateless and the
`umWalls` override lives at the boundary in `place_extras`. Every caller of
`civ_faction_aggregates` had therefore fed the military axis a **constant zero**
for its `0.35 · fortifiedFraction` term — a third of the formula, dead. The new
bridge composes the place rows itself (`um_infer_walls` per settlement, then
`FactionPlace { fortified, ..from_settlement(s) }`), as the reference's aggregate
pass does. Measured on seed 483920: de-walling one faction's five settlements
moves its military power **89.00 → 61.00**.

It also gives `umWalls` and `umAge` their first consumer anywhere —
`civ_roster_bridge.rs`'s module doc had said since ED-03 that an edited value
*"reaches nothing"*.

**What stays open, and only this:** garrison **headcounts**, campaigns, unit
movement, combat. The reference has none of them, and none is derivable from the
above — a headcount would be a fabricated number wearing a real one's clothes.

### CV-26 was genuinely new, and the register's structural objection was the right one

The §37/§39 diagnosis — no edge between two factions to hold a value — was
correct, and the edge is what got built, nothing else. `cartalith_civ::relations`
produces one **symmetric** value per unordered faction pair, **derived and
recomputed** like the aggregates and wildlife regions, stored and saved nowhere.
Four terms, each symmetric by construction, each reported beside the verdict so
the reader can disagree:

| term | weight | source |
|---|---|---|
| shared culture | `+0.30` | `civFactionCulture` |
| shared / opposed faith | `±0.20` | `civFactionReligion`; `none` on either side is silence, not division |
| trade complement | `+0.25` | the aggregate's own `imports`/`exports` |
| border friction | `−0.55` | shared-border cells × `(0.35 + 0.65 · rivalry)` |

**Friction is border × rivalry, not border.** A long border with a weak neighbour
is a frontier, not a rivalry; `rivalry` is high only when *both* sides are strong
**and** evenly matched. The border is measured against **the widest border on this
map**, not an absolute cell count — the relative-not-absolute discipline the
reference's own v1.30/v1.32/v1.37 trade-balance fixes settled on.

**A good nobody supplies is discounted.** The trade term's denominator counts only
imports that *some* faction exports: a deficit nobody can fill is a shared
shortage, not a relationship — the reference's own v1.33 finding that a food
deficit is not automatically an import *"when there is no direct trade that could
sustain [it]"* (line 24500). It matters concretely: this port retains no
`currentPopulationDensity()` equivalent (`CivData::dens` is
`civ_current_agrarian_density`, a **different field**, and substituting it would
silently move `foodProductionCapacity` off the reference's number). With the food
half of the balance absent, `food` lands in every faction's imports and nobody's
exports; without this rule it would dilute every pair's trade term toward zero.

**What stays open:** diplomacy actions, treaties, vassalage, and relations that
change over time — each needs a decision this port should not make alone (who
acts, on what clock, what a treaty does to the map). The value is a reading of the
world as it stands.

### Verification

Two temporary, untracked harnesses against a real world, seed 483920.

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

`_mildock_shot.gd` (shell level, the real `app.tscn`, 233 settlements, 6 factions)
— **PASS, windowed and headless**. Both categories render real rows:

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

Both old "Not built" disclosures are gone while both **narrowed** gaps are still
stated on screen — the probe asserts all four, because closing an entry by
deleting its honest note is the failure this document exists to catch.

Plus `cargo test`: `cartalith-civ` **401** lib tests (11 new in `relations`, 9 in
`military`) and a new `golden_parity_military.rs` (3 tests); `cartalith-godot` 343
lib tests and every integration target green. `cargo check -p cartalith-godot`
clean.

**Two boundary assertions failed on the first extraction, both real range
errors** — `CLAUDE.md`'s "verify the line ranges before slicing" rule, for the
sixth time. The `_umWallSpec` slice started at 22105, inside the v1.17 provenance
comment rather than at the `function` line (22109). And the four-rung assertion
was written as four `return 'x';` statements, which the reference does not
contain: `palisade` reaches its caller only through the ternaries
`pop>=1200?'stone':'palisade'` and `rank>=1?'palisade':'ditch'`. Both failed
loudly instead of emitting a short, plausible golden.

**One equivalent mutant, recorded rather than chased.** `terrainD>0.9` and
`terrainD>=0.9` are the same function: `1 − 4·|r − 0.35|` never evaluates to
exactly `0.9` for any `f64` `r` nearby — the results step from
`0.9000000000000001` straight to `0.8999999999999999`. The *constant* carries the
meaning, and `commanding_village_digs_in` pins it from both sides. Mutating
`0.9 → 0.8` fails; so do `1200 → 1100`, `260 → 250`, `0.6 → 0.65` and
`rank>=3 → rank>=4`.

## 41 · CV-23 — the influence field was being computed and thrown away, and keeping it costs nothing resident (2026-08-25) — **NARROWED**

§39 sharpened this row and got both halves right. This section is what it
prescribed, plus the measurement it asked for.

### What was already true, and what one array bought

`assign_territory` (`cartalith-civ/src/lib.rs`) runs one `road_dijkstra` per
capital and keeps a running per-cell minimum of `dist / territory_weight(pop)` in
a local called `best_effective`. **That local *is* the influence field** — computed
on every `generate()` and dropped on the last line, which returned only the `i32`
owner id.

Contested-ness is the runner-up beside it — the lowest effective distance from a
capital of a *different* faction — and it is a single extra compare in the same
per-cell branch, exact in one pass. The invariant that makes one pass enough:
`rival_effective >= best_effective` always, because the runner-up is only ever
written from the *outgoing* winner at a change of owning faction (the incoming
winner is strictly smaller) or from a candidate that already lost; so the value
discarded is always `>=` the value kept, and the value kept belongs to a faction
that is by construction not the new owner.
`influence_rival_is_the_true_runner_up_faction` checks it against the brute-force
per-faction minimum over a ragged three-faction fixture.

`territory_sweep` is now the one implementation both callers share, so
`assign_territory` and `territory_influence` cannot disagree about ownership —
`influence_owner_matches_assign_territory` pins that over four layouts
(same-faction displacement, cross-faction displacement, a third faction that never
wins, an unreachable half).

### The two obstacles §39 named, and how each was actually paid

**Memory — the owner's decision, followed.** Nothing holds an influence grid.
`sample_bridge::territory_influence` builds one when the layer is opened or the
readout pressed, reads it, and drops it — the `wildlife_regions` shape, named by
the owner. `CivData` gained no field.

**The freed `cost` field** is *rebuilt*, not retained, and honestly so:
`build_travel_cost` is a pure function of the height field and sea level, and
`FieldRefs` — the borrow struct every analysis layer reads through — holds both.
One parallel pass over the grid, **zero** resident bytes. Leaking a `gw × gh` f32
back into `CivData` was the alternative, the exact thing
`MEMORY_OPTIMIZATION_SCOPE.md` exists to prevent.

`territory_sweep` takes a `want_rival` switch for the same reason: `false` makes
the loop body character-for-character what `assign_territory` ran before, with the
two extra runner-up grids left as `Vec::new()`. **Generation pays nothing for a
layer nobody may open.**

### What got built

| | |
|---|---|
| `cartalith_civ::TerritoryInfluence` | `owner` (borders), `rival` (claims), `influence` (the kept `best_effective`), `contested` (`influence / rival_influence`, `0..=1`) |
| `cartalith_civ::territory_influence` | one sweep, one Dijkstra per capital, nothing cached |
| `WorldGen::civ_territory_influence()` | per-faction rows and per-*pair* border rows, plus `transient_bytes` / `resident_bytes` |
| Layers ▸ Civilization ▸ **Contested borders** | `LAYER_GROUPS`' own convention — a row, a hint, a five-swatch legend, a per-world `layer_available` check |
| CIVIL ▸ Territories ▸ **Borders & influence** | the numbers, behind a button, because the computation costs something |

**The raster invents no hue.** Every colour is a faction's own swatch
(`CivData::faction_rgb`, identity colours included), dimmed to `0.26` in a secure
interior and lifted to full strength on a frontier by `0.26 + 0.74·t²`. Past
`t = 0.88` ("the runner-up is within 12 % of the winner") the cell alternates with
the *rival's* swatch on a three-cell diagonal stripe, so a frontier reads as a
two-colour weave naming both claimants — the claim hatching CA-17 asks for, drawn
in the analysis layer rather than the territory wash.

**The contested band is wider far from either capital, and that is the model
talking.** One step in from a border the winner's distance is `d` and the rival's
about `d + 2·step`, so the ratio is near `1` when `d` is large and well under `1`
when the border runs close to a capital. A frontier between two distant centres
genuinely is more evenly balanced than one at a capital's gate.

### What stays open

**Historical occupation over time**, as §39 scoped it: the timeline records
settlement snapshots per year, not a per-year ownership grid — timeline work, not
territory work. The Territories ▸ Not built note now says only that.

### Verification

`_cv23_probe.gd` / `.tscn` (temporary, untracked), **windowed** against §39's
world — seed 483920, 384 × 288, 233 settlements, 6 factions, 35 ways. **PASS, 0
failures.**

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

**Memory, measured.** Process working set (read from Windows, not Godot's
allocator, since every byte in question is Rust's) across **25 consecutive
rebuilds** at 384 × 288: `517.8 MB → 518.0 MB peak → 513.8 MB`, net **−4.0 MB**;
twenty-five calls that each kept their field would have been +140 MB. At
1024 × 768 — 786,432 cells, 510,600 owned, 59,717 on a frontier, **343 ms**, a
**39.8 MB** build — the peak working set does not move **at all**
(`891.5 → 891.5 MB`) and the resident figure comes back 127.5 MB *below* where it
started.

`transient_bytes` reports the honest **peak**: 53 bytes a cell — 4 for the rebuilt
cost field, 24 for the sweep (`owner`/`best_effective`/`rival_effective`/`rival`),
25 for one capital's Dijkstra at a time (`dist`, `prev`, `visited`, and the heap's
own `with_capacity(n)`). **41 of those 53 are what `assign_territory` already
spends inside `generate()`.** Opening the layer costs 12 bytes a cell more,
transiently, and `resident_bytes` is `0`; the 268 MB objection was about
*resident* state, and there is none. Stated plainly: at the 8192² ceiling the same
arithmetic is **3.3 GB transient**, against the **2.6 GB** `assign_territory`
already peaks at inside `generate()` — both enormous, and the ceiling is
theoretical for the whole civ pipeline, not this row.

## 42 · IN-13, VA-01, ED-02 — the last three open items, designed and built (2026-08-25) — **CLOSED**

The owner's instruction was two-part: *"use /ui-ux-pro-max in combination with
your /design skill to create a proper menu for the missing items"*, then build
IN-13, VA-01 and ED-02 against those designs. §37's fifteen were down to those
three plus the narrowed remainders of CV-23, CV-25, CV-26, CA-18, CA-19 and
WW-15, all still disabled controls and bare notes rather than designed surfaces.
The design is a five-artboard canvas in v3's own visual language covering the
three surfaces plus a consistent **anatomy for a "Not built" row**, which the six
narrowed remainders now share.

### IN-13 — the register's reason was wrong, for the sixth time this session

§39 sharpened IN-13 to *"a flow needs a bipartite match plus a network flow,
neither of which exists in either codebase."* The second half is false; the
frozen reference has it in three functions:

| Reference | Line | What it is |
|---|---|---|
| `_civFoodShed` | 24050 | enumerates **every other settlement** as a candidate supplier — the bipartite match |
| `_civFoodConnected` | 24044 | filters candidates through `_civRoadConnected` … |
| `_civRoadComponents` | 24076 | … which is a **union-find over the way network's own endpoints** — the tie to the way that carries it |

Plus `_civFoodMode` (23997), `_civFoodDeliverable` (24004) and `_civGoodReach`
(24442), which decide per pair and per good whether the relationship is possible
and how much survives the distance. The reference runs all of it for **one
good**, `food` — and wrote `_civGoodReach` explicitly to classify *"the reach a
given good can achieve from this settlement"* across a bulk/luxury vocabulary of
twenty-two goods, then used it only for display.

So `cartalith_civ::trade` is **five ports and one new step**: the reference's own
match over the fifteen `CIV_RESOURCE_KEYS` that `TradeBalance` already gives a
verdict on, gated by the reach rule the reference wrote for exactly that. Constants
ported literally: 160/880/8000 km doubling, 220/1600/9000 km reach cliff, the
50 km local supply radius, the 0.6 supplier share.

**The one rule that is not a port**, stated at the function: a settlement's
demand for a good is its **population** — the only per-settlement scale here not
itself derived from trade (`_civPlaceProsperity` reads `tradeVolume`, so using it
would be circular). Demand is split across reachable exporters in proportion to
`_civFoodDeliverable`, and each flow is capped at `SUPPLIER_SHARE × the supplier's own population` — that constant's own sentence, *"one consumer never draws a
supplier's whole surplus"*, applied where it is about. Uncovered demand is not
carried to another supplier.

**Two divergences, both in the module doc.** Road connectivity reads
`Way::a_idx`/`Way::b_idx` instead of re-deriving them with the reference's
`nearest()` endpoint snap at `max(2, GW/50)` — the consolidation tail already
stores which two settlements a way joins. And `_civPlaceNavigability` is ported at
branches (a) and (b) only: branch (c) reads `_umSiteProfile`'s
`coastDistKm`/`riverDistKm`/`riverOrder`, which here are locals inside the layout
builder's water context. It costs almost nothing — (b)'s
`um_site_kind_from_terrain` sweeps the *same* `um_water_reach_km` radius (c)
thresholds against, and the reference's own comments call the site kind
"authoritative" over the traced polylines on both coast and river branches.

**Nothing is stored.** `trade_flows` allocates, answers and drops, like
`territory_influence` and `wildlife`; `CivData` gained no field and the save
format is untouched. The shell holds the one result in `trade_store.gd` so the
dock, the place editor's ledger and the map overlay share one computation;
`app.gd` drops it on every world change, beside the existing
`on_world_changed()` re-runs.

**Three surfaces.** CIVIL ▸ Trade carries v3's four rows in the design's
disclosure ladder — world, good, pair, place. The place editor carries the
per-settlement ledger, because a partner is a *name* that only means something
next to its place. The map draws way **thickness** in CARTO ▸ Roads & routes, not
in the Layers popover: that popover is the one picker for *field rasters*
(`set_debug_layer`), and trade load is a value on a way. Width, not hue, because
every faction swatch is spent on the territory wash and contested borders, and a
way's colour is already its type (RD-02).

### VA-01 — the index is the question, and a stat is not a read

The register posed a choice between two bad options: *"built on demand it stalls
a large vault, and persisted it is a second store to keep in step with a folder
the user edits outside Cartalith."* It is a false pair. `FsVault::meta` already
returns `(modified, len)` **without opening the file** — §14's own
change-detection basis — which makes the index persisted *and* correct:

| | |
|---|---|
| stored | per note: `(modified, len)`, its outgoing link targets, the entity keys of any Cartalith blocks in it, and a 64-bit word fingerprint. **Never the prose.** |
| built | only when a person presses Refresh. The first build reads every note once and says so first. |
| invalidated | per file, by `(modified, len)`. Ten edits in Obsidian cost ten reads. |
| never | a watcher, a background thread, or a scan nobody asked for. |

**Unlinked mentions without storing prose.** `NoteRecord::word_bits` is a Bloom
filter over the note's word tokens — eight bytes a note, from which no word can be
read back (asserted by serialising a record and searching it for its own words).
`mention_candidates` returns the notes that *could* contain every token of a name;
the session opens **only those** and confirms with a real search. False positives
cost one read; **false negatives are impossible**, which is what the tests pin.

**An entity finds its incoming notes by three routes**: its own linked notes'
reverse map, the notes that link to them — and every note carrying
`entity="settlement:42"` **directly**, which finds an entity with no note of its
own (a province's note can describe a place nobody has written a page for). An
index of note-to-note links alone would miss the third.

`broken_links()`/`orphans()` fall out of the same index, which is what
`Data ▸ Missing & orphan notes report…` had been disabled waiting for; that row is
live now. The index is saved in **its own file**
(`user://markdown_vault_index.json`), not inside the link store: §5's store is
portable project data, this is a per-device cache rebuilt in one press.

### ED-02 — a ledger, which is what §7.1 asked for

§39 recorded that a previous pass *"deliberately declined to build a flat
five-row list because that would answer the register's easy half and foreclose
the design question."* This is what that judgement protected.
`undo::HistoryLedger` **records every commit and reverses the ones it can**,
saying per row which it is:

| Kind | Glyph | Meaning |
|---|---|---|
| `HeightSnapshot` | `▲` | a pre-operation height field is held; reverting is real |
| `Recorded` | `·` | it happened; no snapshot exists, and the row carries the **specific** reason |
| `Floor` | `◼` | a generate or a load; history starts here |

**The two structures are deliberately not cross-wired.** The stack evicts on its
own byte budget, so `rows()` takes the live `HeightUndo::depth()` and marks the
newest that many height rows live — one source of truth, asked at read time, so
they cannot drift; an evicted row reports `false` with a reason naming the budget.

Linear only, per §7.1: reverting to a row pops every snapshot above it and the row
leaves with them, because the snapshot **is** the state before that operation.
Recorded rows above it go too — an operation whose height field has been rolled
back from under it is not still in effect.

Call sites: `sculpt_commit` and `carve_fjords` record a snapshot beside their
existing `undo.push`; `paint_commit` and `civ_territory_commit` record with the
reason (*"the pre-commit layer is not retained; Discard reverts an uncommitted
draft only"*); `generate` and `load_save` record a floor, clearing the rows for
the reason `undo.clear()` already runs there.

The panel is a **right-dock context**, per §7.1 proposal 3 — a window over a list
read *against* the map would cover the map. Two tiers, the engine's own
draft/commit seam: the open Sculpt draft above (reversible by its own tool, not a
row), then the commits, newest first.

### The six narrowed remainders, and the anatomy they now share

Not rewritten — they were already honest. The design added a three-part anatomy
(**the noun, named exactly** · **the blocker, named specifically** · **what does
exist instead**) and a state chip separating three different situations:

| Chip | Means | Which |
|---|---|---|
| needs a decision | nothing is missing from the engine; somebody has to say what the feature *is* | CV-25 (garrisons, campaigns, combat — narrowed again by §43, which built the manpower half the owner specified; §44 settled that model's one open question, the era bands' denominator), CV-26 (treaties, vassalage, change over time), WW-15 (reprojection) |
| blocked on | the design is clear and waits on named work elsewhere | CV-23 (historical occupation — timeline work), CA-18 (declutter budget — CA-04's separable layer stack) |
| costs a re-baseline | buildable today, and doing it moves golden expectations `DECISIONS.md` §7a protects | CA-19 (a *writable* biome table) — **the example did not hold** (2026-09-24): Ruling P made the table writable with the default byte-identical to `CART_BIOME_COLS`, so no golden moved; CARTO ▸ Colours ▸ Biome colours now carries the picker |

A fourth state is deliberately absent: *coming soon*. Nothing here has a date.

### Verification

`_in13_probe.gd` / `.tscn` (temporary, untracked), run **windowed** against a real
384 × 288 world, seed 483920 — 233 settlements, 6 factions, 35 ways. **PASS, 0
failures.**

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
tests). `cargo check -p cartalith-godot` clean; headless boot and import clean,
`project.godot` diffed afterwards and unchanged.

**Three defects the windowed run and the screenshot found.** `way_load` was
emitted in `CivData::ways` order while `map_overlay.gd` indexes `get_roads()` order
(which filters hidden ways and appends manual ones), so on this world the shell got
**60 entries for 35 rows**, misaligned from the first hidden way onward. The
Generate-world floor row read `seed 0` against a status bar reading `483920`,
because it recorded before `self.seed` was assigned — visible only with both in
one screenshot. And the Match button was built once from CIVIL's `setup()`, before
any world exists, and nothing re-enabled it — **RF-01** again, found by pressing
the real control.

### What stays open

Nothing new: the six narrowed remainders gained a consistent surface, not a
capability. IN-13's own remainder is stated on screen for the first time:
**prices, tariffs, caravans as entities, and trade that changes over time** —
none derivable from anything the civ layer holds, each needing a decision about
what a currency is here.

## 43 · CV-25's other half — the manpower model, on an owner-supplied specification (2026-08-25) — **STILL NARROWED**

§40 built CV-25 as a minimal military model and found that the reference **does**
model fortification, three times over, and `power.military` was already ported.
The owner then supplied a researched specification for the half §40 could not
derive — **how many people a polity can actually put and keep under arms** —
superseding §40's implicit answer ("none, a headcount would be a fabricated number
wearing a real one's clothes"). That was right about the evidence then and wrong
as a permanent verdict: with five stated variables and two derivation chains, a
headcount is derived, not fabricated.

`MILITARY_MANPOWER_SCOPE.md` is the durable home — the owner's specification
**verbatim**, the derivation of every constant, and the verification. Below is
only what the register needs.

### The split, and what stays

| | disposition |
|---|---|
| `_umWallSpec` / `_umInferWalls` / `_civPlaceDefensibility` | **unchanged.** Fortification is a separate axis — how hard a place is to take, not how many people can be raised. Ports, golden-verified, untouched by this pass. |
| `_civFactionAggregates` → `power.military` | **unchanged, deliberately.** A golden-verified port of the reference's own formula; rewriting it to derive from a model the reference does not have would break parity to gain nothing, and it answers a different question (*this faction against the others on this map*, relative 0-100) from the headcounts (*how many people*, absolute). Reported side by side, each labelled. |
| manpower | **new** — `cartalith_civ::manpower`, four outputs from five variables. |

### The reference really has nothing this time

`manpower`, `mobiliz`, `levy`, `conscript` and `militia` return **two hits in the
whole frozen snapshot**, both `JP_COST_TOLL_PER_BORDER`'s comment using "levy" to
mean a *toll*; `FUNCTION_INDEX.md` returns zero. So there is no golden fixture and
none is fabricated — 14 unit tests and two live probes instead.

### Two more inert tables got their first consumer

The §40 pattern, twice. `roster::AG_TECH_LEVELS`' module doc said
`farmers_per_urbanite` was *"presently as inert as Government/Religion are in the
reference"*; `roster::CIV_GOVERNMENTS`' said *"no simulation reads or writes this,
and nothing in this port does either"*. Both are read now, proved live on a real
world: `traditionalAgrarian → improvedAgrarian` moves a faction's standing army
**1 435 → 2 615**, and `chiefdom → empire` moves it **948 → 1 841**. Their roster
tooltips now say what they do.

### The four outputs, on a real 233-settlement world

Standing 87 … 1 509 · field 3 444 … 9 305 · levy 8 009 … 20 262 across six
factions, with a four-rung force/duration ladder each. **The specification's worked
examples are reproduced**: Kingdom A at 5 846 / 41 221 / 15 870 against a stated
~5 000 / ~40 000 / 15 000-20 000, Kingdom B at 19 067 / 98 889 / 47 368 against
~20 000 / 100 000+ / 40 000-60 000. Every figure in range; the worst is A's
standing army at +17 %, left there rather than tuned.

Three things fall out that the specification does not state and so cannot have
been fitted: Kingdom A's full levy sustains **77 days** (the feudal ~2-month
obligation), Kingdom B's 90- and 180-day rungs are 59 455 and 38 045 (bracketing
its stated field army — a field army *is* a campaign-season force), and the
standing shares land at Imperial Rome's own ratio.

### Four findings, and one is a question for the owner

1. **The specification's era table and its worked example disagree**, and the
   table disagrees with its own Imperial Rome figure — Kingdom A's stated 40 000
   levy is 4 % of population, below the 5-15 % band of every pre-modern era
   listed; Rome's 250 000 over 45-120 million is 0.21-0.56 %, under the classical
   row's 1 % floor by two to five times. Calibrated on the worked example, with the
   band reported as the sanity check the specification asks for. Offered, not
   implemented: **the bands may be shares of a citizen or free population** (the
   specification's Republican Rome citation says "17-29 % of its *citizen*
   population"), in which case the live figures land inside them. **→ Ruled and
   built the same day; see §44.**
2. Standing shares agree with the specification's *example* and not its *table*,
   in the same direction and for the same reason.
3. `ecological_factor` saturated for five of six factions on a real world:
   generated territory sustains at least twice the population the model puts on
   it (the same divergence `civ_agrarian_regional_total`'s readout has always
   shown), and this pass concluded geography therefore discriminates mainly at the
   low end — Draumr League's 87-strong standing army against Veldmark's 1 509 on
   otherwise identical institutions. **Corrected 2026-09-06:** that saturation was
   what owner ruling 11 addressed, moving the ceiling 2.0 → 4.0
   (`ECOLOGICAL_CEILING`, `manpower.rs`). Across 108 faction-samples the old
   ceiling pinned **39 of 108 (36.1%)**, 4.0 pins **21** — so the "mainly at the
   low end" clause is the part the ruling falsified.
4. **The road-density reference was wrong on the first try, and measuring found
   it.** Anchoring on the Roman empire's ~16 km/1 000 km² made roads a dead term;
   this port's way network is inter-settlement trunk roads only, no lanes or
   streets. Recalibrated against what the network produces, roads spread
   0.11-0.91 instead of 0.03-0.23.

### CV-25 stays narrowed, and the note on screen changed

The old note, *"garrisons · campaigns · unit movement · combat"*, now separates
**per-settlement garrisons** (the per-*faction* headcounts are real; which
settlement holds which part of a standing army is a placement rule nothing
implies) from campaigns, unit movement and combat. Still `needs a decision`, still
disclosed, and the shell probe asserts the disclosure is there.


## 44 · CV-25 — the owner answered §43's open question: the era bands are shares of the citizen population (2026-08-25) — **STILL NARROWED**

The owner answered §43's finding 1: **the era table's percentages are shares of
the citizen / free population, not of the total** — grounded in the owner's own
specification, whose Republican Rome figure (*"17-29 % of its **citizen**
population"*) is the one place it names a denominator. `MILITARY_MANPOWER_SCOPE.md`
§1a carries the ruling (as an annotation — that document reproduces the
specification verbatim), §2.6 the derivation, §3.2a the re-measured verdicts.

### What the denominator is, and what grounds it

Nothing in `cartalith-civ` distinguished a citizen, free or full-status subset of
population — grepped before inventing one. `FactionEntry::culture` turned out to
be `CIV_CULTURES`, name-syllable pools with no social content. So it is derived
from what exists: `clamp(CITIZEN_SHARE[government] + 0.68 × urbanisation, 0.20, 0.98)`.

Government is the driver on the merits: the specification's two cases sit on
either side of exactly this distinction — a republic's citizen body is a much
larger share of its polity than a pre-Caracalla empire's, which is what makes
Hopkins' 17-29 % and Rome's 0.21-0.56 % consistent with one table. Shares run
`chiefdom` 0.90 → `monarchy` 0.55 → `republic` 0.50 → `city_state` 0.45 →
`oligarchy` 0.40 → `empire` 0.30, each grounded in §2.6's table (Domesday, Attica
c. 431 BC, Polybius' 225 BC census). The modernisation term is **derived rather
than chosen** — legal servitude is an agrarian institution, and `0.68 =
CITIZEN_CEILING − min(CITIZEN_SHARE)` makes every government converge on universal
civic status at industrial labour ratios; a test pins the identity.

### The calibration did not move, and that is asserted rather than claimed

The four outputs were **not** recalibrated: `the_citizen_ruling_moves_no_headcount`
pins Kingdoms A and B to §43's figures, and the live probe pins that total
population does not move when only the government does and that every levy
restores exactly when the roster is put back. Only the two verdicts changed basis.

### What the verdicts read now

On the 233-settlement world, **five of six factions read `within` on both bands**
where all six previously read `below` on standing. Draumr League is still `below`
on standing at 0.09 % — its `ecological_factor` is 0.428, which is §43's finding
3, not a denominator problem.

On a sparser 33-settlement world with one government per faction (the denominator
has to be *differentiated* to prove anything, and a default roster is
all-`monarchy`), the citizen fraction spreads **0.378 … 0.978**, mobilization
reads `within` for five of six, and standing reads `within` only for the narrowest
citizen body. That residual is §43's finding 2 (standing armies at Imperial Rome's
ratio, which the table's standing column never agreed with), reported rather than
tuned, because correcting it would mean recalibrating outputs validated against
the worked example.

### On screen

The denominator is **surfaced** — a band verdict whose divisor a reader cannot see
is a number they cannot argue with. CIVIL ▸ Military gains a *Who the bands are
measured against* group: per faction the citizen headcount, its share of the
total, both citizen-based shares with their verdicts and the era, and a tooltip
giving the same two figures against total population, so the previous basis stays
legible. The Faction Roster's Military block names the citizen population and the
government that conferred it above its verdict. The category's closing note states
the ruling instead of warning that `below` should be expected. CV-25's row is
otherwise unchanged: still `needs a decision`, still open on per-settlement
garrisons, campaigns, unit movement and combat.

## 45 · MN-10, RL-01, CA-20, RF-02…RF-05, FI-04 — the "is every control wired" sweep (2026-08-25) — **SEVEN FIXED**

The owner asked for confirmation that every GUI control does what it claims,
reaches real capability, and does not silently do nothing. This sweep is as much a
record of what was driven and found **clean** as of what was broken, so the next
pass does not re-walk it. Every finding came out of the live app, none from
reading.

### What was driven, and how much of it

| Probe | What it covers | Never covered before |
|---|---|---|
| `_rf01_probe.gd` | 89 surfaces fingerprinted at **no world → world A → world B** | the whole RF-01 question, systematically |
| `_railpress_probe.gd` | every enabled button in all **33 rail categories**, the right dock in **7 contexts**, **11 tool-options rows**, the section strip, the menu bar | `_pressall_probe.gd` only ever pressed buttons in *windows* |
| `_valuectl_probe.gd` | **11 OptionButtons and 110 Ranges**, each moved to a different value and put back | **nothing had ever changed a value control and asked what happened** — both existing sweeps skip `OptionButton` by name |
| `_menuwire_probe.gd` | 148 menu items across 23 popups, with `about_to_popup` fired first | menu gating computed on popup was read cold, producing four false positives |
| `_winstale_probe.gd` | every window **left open across a generate** | §23 asked "what re-runs this?" of panels built at launch, never of windows built on `open()` |
| `_newsurf_probe.gd` | the newest surfaces end to end, plus all **35 Layers entries measured in pixels** | — |

Three probes were wrong on their first run, each a way this class of sweep
silently covers nothing:

1. **A jump button leaves a different workspace on screen.** After one
   `→ Cartography ▸ Political display` press every later category reads
   `is_visible_in_tree() == false`; the first run "passed" twelve of CIVIL's
   fourteen categories by finding no controls in them.
2. **`emit_signal("pressed")` on a toggle changes no state and fires no
   `toggled`.** All 30-odd `DccWidgets.toggle` checkboxes read as dead until the
   probe drove `button_pressed` instead.
3. **A menu never popped has not been asked its own gating question.** Half this
   shell's menus compute `disabled` in `about_to_popup`.

### MN-10 — a menu item with a written handler and no way to reach it

`Assets ▸ Asset pack ▸ Pack metadata… (name / author / license)` was enabled, had
an id and a handler branch (`_on_assets`' `ID_AP_PACK_META`), and nothing could
reach it: the `AssetPack` popup's **`id_pressed` was never connected**. Its three
child submenus each connect their own (`APEdit`, `APBatch`, `APBuild`), which made
the group look wired. A submenu's `id_pressed` does **not** bubble to its parent in
Godot 4. One line; connections **0 → 1**, and the row now shows the Asset Library
window. `_menuwire_probe.gd` adds the general check: for every popup whose
`id_pressed` has no connection, list its live items.

### RL-01 — a row that named a pair, opened one side, and often did nothing at all

CIVIL ▸ Relationships lists one row per faction pair (`Aurelia ↔ Korrath — wary
(−22)`). Every row called `show_faction(a)` — the **left-hand** faction — so any
run of rows sharing that party was a press with **no visible effect**. Measured on
the 233-settlement six-faction world: **5 of 15 rows dead** — Korrath ↔ Draumr
League, Veldmark ↔ Mirelle, Veldmark ↔ Draumr League, Aurelia ↔ Sythe Dominion,
Aurelia ↔ Korrath. The honest tooltip (*"Opens Aurelia in the right dock"*) kept it
looking deliberate.

Fixed without a new dock context: `show_faction` takes the other party, and the
faction panel gains a **Relations** section built from the same
`civ_faction_relations()` read the list makes — so the two cannot disagree — with
the clicked pair marked `▸`. Verified: **all 15 rows move the dock**, which draws
`▸ Sythe Dominion` among Korrath's relations.

### CA-20 — two Clear-all buttons live over an empty list

`DCC_SHELL_SPEC.md` §4.5.5 asks for both annotation panels *"with counts and
Clear-all"*. The count was never drawn, and the button was enabled at zero with no
tooltip. The count now sits on the button, which is disabled at zero with a stated
reason, its state owned by the `_rebuild_*_panel()` calls that already run on every
place, delete, clear and world change. Verified: empty → `Clear all labels`,
disabled, a 110-character reason; two real labels → `Clear all labels (2)`, live;
press → engine list **0** and dead again.

### RF-02 / RF-03 — §23's question, asked of windows this time

§23 asked *"what re-runs this, and on which signal?"* of every panel built at
launch, never of windows, which are built on `open()` — correct only if nothing
changes while they are up. A world can. Both windows keyed to an identity a
generate renumbers were stale, and both **destructive**, because every editable
control in them writes by that identity:

- **RF-02, the place editor.** Left open across a generate it showed `Sevjuniana`
  pop **19 332** at (142, 14) while the engine's settlement 0 was pop **19 774** at
  (208, 183). Every field writes `civ_edit_settlement(_index, …)`, so a commit
  would have written the previous world's name, kind and traits onto whatever now
  sat at the index — PE-01's failure with a generate as the trigger.
- **RF-03, the faction roster.** Aurelia:27 / Veldmark:49 / Mirelle:57 on screen
  against a live Aurelia:57 / Veldmark:27 / Mirelle:7 — plus two cached per-world
  fields (`_fits`, `_military`) taken at `open()` and never re-taken. FR-02's
  failure, same trigger.

The name RNG is seeded the same way in every world, so settlement 0 and faction 0
get the *same name* on both seeds — the probe's first cut passed on a name
comparison. Population, coordinates and counts discriminate.

Both now subscribe to `generation_finished` and `world_loaded` and rebuild when
visible, like `city_viewer`, `world_data` and `performance`. **Rebuilt, not
closed**: half of `world_loaded`'s emitters (`load_asset_pack`, `as_apply_to_map`)
touch no settlement. And through `_rebuild()` rather than `open_for()`, because
`open_for` commits the focused field first — committing this form against the
world that replaced the one it was typed for is the bug; `_clear()`'s
`_rebuilding` guard drops it.

### RF-04 — the first signal-*ordering* bug this register has had

`infrastructure_workspace.gd`'s comment said the Flows body *"refills from whatever
`TradeStore` holds, which `app.gd` has just cleared on this same world change"*. It
had not. Godot delivers a signal in **connection order**;
`app._register_workspaces()` runs at line 313 and `_wire_status()` at line 333, so
on every generate the INFRA refill read the **previous** world's match (flow count,
timing, settlement names) and only then did `_refresh_world_dependent()` drop the
store, with nothing left to re-run the fill. Measured: after regenerating under a
live **624-flow** match, CIVIL ▸ Trade ▸ Flows still reported 624 while
`TradeStore.last()` was empty, so the dock and its two fellow readers (the place
editor's ledger, the way-load overlay) disagreed about whether a match existed.

Fixed by connecting the clear immediately after the bridge is constructed, before
anything else subscribes; the call in `_refresh_world_dependent()` stays to keep
that function's stated ownership true. Verified: the body returns to *"Not matched
yet"*.

**The lesson, and it is new:** a comment asserting that another handler has already
run is a claim about connection order, which `_ready()`'s call sequence in a
different file decides. Two correct handlers can still be wrong together.

### RF-05 — RF-01 exactly, on a control that worked perfectly

**CARTO ▸ Roads & routes ▸ Trade load** disables itself when
`overlay.has_trade_load()` is false. That category is built once, at launch, with no
world — so the row was born disabled, and the match that makes it valid runs in a
**different workspace**. Measured: after a real 624-flow match, `disabled = true`
while `has_trade_load()` returned `true`; forced on it moved **0.6028 %** of the
map's pixels, off returned **0.0000 %**. The fifth recurrence of RF-01, the second
on a control gated at build time against data that arrives later.

Fixed at the funnel: `map_overlay.set_trade_load()` is the single path both the
match and the world-change clear pass through, so it emits
`trade_load_changed(available)` and CARTO follows it **in both directions** —
switching off, not merely greying, when the reading goes away, because a live
toggle over an empty reading is the same lie the other way round.

### FI-04 — copy that named a place the user cannot look

`File ▸ Recent worlds` remembers a **path**, so every row can outlive its file.
All three rows of a real recent list named deleted saves, and all the user got was
*"load failed — see console"* — true of neither, and naming somewhere an exported
build cannot see. One `file_exists` separates the two owners: a missing file is the
list's problem, a refused one is the save's. Now reads `<name> is no longer on disk`.

### Driven and found clean — the negative results

Recorded so the next pass does not re-walk them.

- **RF-01 across three worlds, 89 surfaces.** All 33 rail categories, the right
  dock, the Layers popover, 11 tool-options rows, the section strip and 23 menu
  popups, fingerprinted with no world, world A, then a different world B. **Not one
  surface kept an empty state across a generate.** §23's eleven sections and §37's
  fifteen all still refresh.
- **Value controls: 11 OptionButtons and 110 Ranges, zero dead.** Every one moved
  something — the class that had never been driven at all.
- **Buttons in the rails and dock: only the findings above.** The one remaining
  no-op is the `SCULPT` mode chip, which rebuilds the row it lives in.
- **The menu bar:** 148 items, 41 pressed. `APBuild ▸ Apply to map` and
  `Assets ▸ Apply library to map` are one action reached two ways and set the same
  status — a confirmation, not a defect. `GpuDevices ▸ Rescan devices…` re-queries
  and refills a submenu read on popup — idempotent on a stable machine, real
  capability, no visible change.
- **The Layers popover, in pixels.** 35 entries; 34 repaint the map and **all 34
  hash differently** — no two layers draw the same frame. RD-02's class is clean
  here.
- **Windows across a generate:** City viewer and World data rebuild. Performance,
  Travel library, Vault, Data manager, Asset library and the Layers popover render
  identically and correctly so — none shows anything keyed to world identity.
- **The newest surfaces:** the trade match runs in 166 ms for 624 flows over 61
  importing and 31 supplied settlements; the ED-02 ledger's floor row names the
  live seed; CIVIL ▸ Territories ▸ *Analyse contested borders* is live and *Clear
  territory* is correctly disabled.

### Not fixed, and why

- **`Window ▸ Open windows ▸ "No windows open"`** is disabled with no tooltip. It is
  an **empty-state caption**, not a capability claim — like the asset-pack
  submenu's three disabled stat rows — so a tooltip would read as disclosing a gap
  that does not exist.
- **A recent-worlds row whose file has vanished stays in the list.** FI-04 makes it
  say so; whether it should then be greyed, removed, or offer to forget itself is a
  product decision, not wiring.


---

## 46 · PH-12 — nine screens that never had a phone pass at all (2026-08-25) — **FIXED**

The owner ran the Android build on a **OnePlus 12** — 1440x3168, ~510 ppi, so
`DccShell._phone_scale` is `1440 / 393 = 3.664`. Every prior phone measurement was
taken at a 1080 short side (scale 2.75), and none on the screens below. His words:

> *"not all screens are optimised for a mobile phone, among others the asset
> manager screen. Plus the layout is impractical and doesn't listen well to
> touch input and isn't intuitive."*

### The fault

The shell has a working four-call phone pattern, written across PH-03 to PH-11 and
documented in `dcc_widgets.gd`'s header: `DccWidgets.phone_window()` at setup,
`DccWidgets.phone_present()` on open, `DccShell.phone_fit(self, 1.0)` after the
body is built, and `DccWidgets.phone_head()` for the header a borderless window
draws in place of its title bar. **Nine `Window`-derived scripts called none of it
— zero occurrences of any of the four.** They were simply the desktop composition
at the device's native resolution, where on a 510 ppi panel a 24 px `dcc_widgets`
row is about 1.2 mm of glass.

Measured windowed at 1440x3168 with the force-touch path (`_ph9_probe.gd`), before
the fix. "Under floor" is §13's 44 dp minimum, 161 physical px at this scale:

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

A parallel sweep on a generated world adds what the empty state hides: the
world-data settlements table is **1 470 individual `Label` nodes** across 240 rows
at font size 9, reachable only through an ~8 px scrollbar; the data manager's body
labels are 39 of 69 under 11 px.

### What a content scale fixes, and what it does not

`phone_present()` maps the desktop-authored composition onto the mockup's 393 dp
reference, which answers density; `phone_fit()` answers tap size. Neither answers
**composition**, which is why `phone_window()` returns `is_phone`:

- **Asset library**: a 266 px family rail + a slot grid + a 330 px inspector — the
  rail and inspector alone are 596 of the 393 dp available. → three panes behind a
  segmented switcher, one at a time (§13's *"docks become full-height sheets, one
  at a time"*), the switcher following the work: pick a family → SLOTS, tap a slot
  → SLOT.
- **Data manager**: 252 px rail + pane → two panes, same switcher.
- **Travel library**: 286 px rail + inspector → two panes, switched by its existing
  tab strip plus a `‹ Entries` chip.
- **Slicer modal**: a flexible preview beside a fixed 274 px settings stack →
  stacked, the settings column scrolling.
- **Journey planner centre panel**: a 196 dp totals column beside the route map,
  and a 642 dp stage matrix beside the stage inspector → both stack.
- **Layers popover**: a **control-type** problem, not composition. See below.
- **World data**: six columns across 393 dp is ~55 dp each. → two-line rows, name
  over the rest, and a 50-row page with a *"showing 50 of 240"* foot.

### Five things only measurement found

1. **`AcceptDialog`'s button bar is an internal child.** `phone_fit()` walks
   `get_children()`, so it has *never* reached the OK button — the only way out of
   `gen_info`, `performance`, `world_data` and the credits sheet. Measured 29 dp.
   Floored in `phone_present()` instead, **after** the `popup()`, because
   `Window.popup()` clears `custom_minimum_size` when it re-lays that bar (the trap
   `app.gd::_floor_prompt_buttons()` recorded for the quit prompt).
2. **`TabContainer`'s tab strip is an internal `TabBar`** — same blind spot. A tab
   has no height property; its height is the font plus the stylebox's vertical
   content margins, so those are the knob. 26 dp stock.
3. **`phone_fit()`'s ellipsis pass reaches only `Button`s.** A `Label` still reports
   its full natural width, and *a `Window` cannot be narrower than its content's
   minimum* (the PH-04 hazard). Three `Label` rows each widened a window past the
   screen: the asset grid's header band with five batch verbs beside it (544 dp),
   its status line's two labels side by side (401 dp), and the data manager rail's
   autowrap foot at **394 dp** — one pixel over, from the panel's own 1 px border.
4. **An embedded subwindow is laid out in its parent viewport's 2D space.** The
   slicer modal was a child of the asset library window, content-scaled by 3.664 on
   a phone — so "fill the screen" from inside it would have been sized in units
   3.66x larger than the screen. Reparented to the shell; the same
   physical-pixels-versus-parent-space confusion `_popup_full()` already records,
   one level further in.
5. **The credits body was empty, on every platform** — not a phone bug. `_ready()`
   fires when a node enters the tree, and `add_child(dlg)` put it there before
   `set_script()` attached `credits.gd`; attaching a script to a node already in the
   tree does not re-run `_ready()`. Measured: **0 characters** in the
   `RichTextLabel`, now 4 420. The attribution `PROVENANCE.md` calls a standing
   obligation had been reaching nobody.

### The Layers popover: checked before it was built

§13 routes several desktop affordances into the ⋯ overflow sheet, so the first
question was whether this one is reachable on a phone at all. **It is, by three
routes**: the map's Layers button (`viewport.layers_button_pressed`), Cartography ▸
*Data overlays…* (`cartography_workspace.gd`), and the Render section's entry
(`render_workspace.gd`).

It became a **full-screen sheet**, not a scaled popover. A popover is a pointer
idiom — anchored to its opener, dismissed by clicking away — and a phone has
neither a stable anchor (the Layers button moves with the safe insets) nor a
reliable "away". Hence an explicit Close (a full-screen sheet has no outside to
tap), and the foot moved *inside* the scroll: the six-line Cartography
cross-reference note, pinned below a 393 dp list, had pushed its last two lines off
the bottom edge beyond any scroll. `DccWidgets.phone_window()` takes an
`AcceptDialog` and this is a `PopupPanel`, so only the applicable halves were used:
`phone_present()` (any `Window`), a `phone_head()`, and `wrap_controls = false` by
hand.

### Verified

`_ph9_probe.gd` / `_ph9_probe.tscn`, driven at **1440x3168** (scale 3.664) and
**1080x2400** (scale 2.748), `--force-touch --nowelcome`, on a generated world. Per
window: `content_scale_factor`, `size` against the screen, every tappable's height
against the floor, every control's combined minimum width against the **window's
own 393 dp column** (comparing dp against the screen's 1440 physical px finds
nothing and misses this whole class), and whether each body scrolls. Screenshots of
all ten surfaces.

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

Identical at 1080x2400 (column 393.0 dp, scale 2.748) — **no regression at the size
every prior pass used**. Desktop unchanged and re-measured: 1440x1002 under the menu
bar for the two full-bleed windows, 1180x780 / 760x620 / 560x480 / 560x420 for the
dialogs, 760x560 for the slicer, 230x600 for the anchored popover, six-column asset
grid, three columns side by side. One back press closes the Layers sheet
(`DccShell::_notification`'s `_topmost_subwindow` reaches it as a subwindow).

### Two probe artifacts, recorded so the next pass does not chase them

- **`--resolution 1440x3168` is silently clamped** to the dev monitor's work area,
  and `_compute_layout_mode()` then decides *tablet* off the boot size, so the run
  measures the wrong composition. Assign `get_window().size` at runtime **before**
  instantiating `app.tscn` (§47's harness went further, to a `SubViewport`).
- **`Window.popup(Rect2i)` clamps to `get_usable_parent_rect()`** — on a desktop
  host `DisplayServer.screen_get_usable_rect()`, 1680x1002 here — so a window that
  correctly fills the 1440x3168 viewport reports 1440x1002; the height is a desktop
  artifact with no Android counterpart. The probe re-asserts `size` after the popup.
  Established as a formula in §47's proven negatives.

### Not fixed, and why

- **The Layers sheet and a phone overlay can be open at the same time.** With the
  sheet up, `_set_drawer_open(true)` leaves both visible:
  `DccShell._close_all_phone_overlays()` does not know about subwindows. The
  one-line fix belongs in `dcc_shell.gd`, which a concurrent agent was editing, so
  it was registered rather than done. Low severity: one back press closes the sheet
  first. (Fixed in §47.)
- **The asset library's phone window bar is four rows tall** (search plus three
  wrapped chip rows), about 24% of the screen. Every action stays reachable and the
  chips wrap by themselves, so this is a proportion question, not a defect — the
  obvious candidate if that screen is revisited.
- **Drag-a-tile-onto-a-Collection** is kept on a phone but its disclosure lives in a
  tooltip, which touch cannot reach. The two pointer-modifier hints beside it
  (`⇧-click ranges · Ctrl-click adds`) are dropped there; the drag itself is left
  alone rather than removed on a guess about touch drag.

## 47 · HD-01…HD-04 — the OnePlus 12 pass: the blur was the font raster, and two of the four leads were negatives (2026-08-25) — **FOUR FIXED, TWO PROVEN NEGATIVE**

The owner reported *blurriness* on the same OnePlus 12 as §46. There
`DccShell._phone_scale` is **3.664** against **2.748** on every earlier 1080-wide
handset, so a defect that scales with it is 33 % worse and crossed from marginal
into visible.

Everything below is measured on the framebuffer. The metric for "is this text
rasterised or resampled" is the **maximum luminance step between horizontally
adjacent pixels**: a natively-rasterised glyph goes ground-to-ink in one pixel,
while a bitmap magnified by *k* cannot step steeper than roughly `1/k` of its own
contrast. A discriminator, not an impression; no adjective was accepted as evidence
in this section.

### HD-01 — Godot 4.7.1 does not oversample fonts for a Window's own content scale · **FIXED**

`DccWidgets.phone_present()` puts every phone modal in a
`CONTENT_SCALE_MODE_CANVAS_ITEMS` sub-Window at `content_scale_factor` 3.664, so a
12 px label is authored in dp and magnified by the compositor. Godot 4.5 introduced
dynamic font oversampling and 4.7.1 has `Viewport.oversampling` **on by default** —
but `Viewport.get_oversampling()` inside such a window returns **1.0**: the automatic
value ignores a Window's own content scale, so the font is rasterised at 12 texels
and the canvas transform smears it. Measured on this build (`_edge_probe.gd`), two
windows drawing the same physical glyph height:

| case | max ΔLum | hard edges (>0.5) |
|---|---|---|
| factor 3.664 / font 12 | **0.2667** | **0** |
| factor 1.000 / font 44 (control) | 0.9843 | 722 |
| factor 3.664 / font 12, `oversampling = false` | 0.2667 | 0 |
| factor 3.664 / font 12, `oversampling_override = 3.664` | **0.9804** | **518** |
| factor 2.750 / font 12 | 0.3569 | 0 |
| factor 2.750 / font 12, override 2.750 | 0.8431 | 375 |

0.2667 is 1/3.75 and 0.3569 is 1/2.80 — the magnification, recovered out of the
pixels. The boolean is **not** the lever (off changed nothing to four decimal
places); `oversampling_override` is. Two traps each made the first cut of the fix
measure as if absent:

1. **The property is inert until the window is in the tree.** Assigned in a
   constructor it reads back on the property and `get_oversampling()` ignores it.
2. **A resize clears it, back to 1.0.** Measured in isolation: on a content-scaled
   Window it survives eleven frames, a `popup()` and a hide/show cycle, then reads
   1.0 the frame after `size` is assigned, and reassigning `content_scale_factor`
   does not restore it — the same trap `phone_present()` carries for the
   `AcceptDialog` button bar. So `DccWidgets.oversample()` sets it *and* re-applies
   it from `size_changed`.

Live on the welcome screen at 1440x3168: **max ΔLum 0.1827 → 0.6126, hard edges 0 →
104**; at 1080x2400: 0.6322 / 74. (0.61, not 0.98, because body text is `#c8cbcd`
on `#17191a`, whose own maximum step is ~0.69 — the native ceiling, not a partial
fix.) One call in one function covers every phone modal with no call-site change.
An embedded `PopupMenu` inherits its parent window's transform but not its font
raster, so `phone_fit()`'s `OptionButton` branch routes its list through the same
call.

### HD-02 — SVG glyphs were rasterised at authored size and then magnified · **FIXED**

`DccIcons.get_icon(name, px)` rasterised at exactly `px`, which inside a
content-scaled window is dp. Measured at 1440x3168: all four glyphs on the welcome
screen at **0.27 texels per physical pixel** — 12 texels drawn at 44 px, 30 at 110, 26 at 95. The brief overstated the reach: there are **8** `DccIcons` raster call
sites, not 66; the other ~60 hits are `DccIcons.SYMBOLS`, which are *text* drawn by
the font — which is why HD-01, not this, is where the owner's complaint lived.

`get_icon()` gains a `magnify` argument: rasterise at `px * magnify`, **present at
`px`** via `ImageTexture.set_size_override`, cache keyed on both. Nothing a caller
lays out moves, and `magnify` defaults to 1 so the main viewport (no content scale,
where a finer raster would be minified back through a 1.2 px hairline) is
byte-identical. Three shapes of the fix were wrong for a real call site:

- a static "device scale" set once by the shell — wrong in the main viewport;
- re-rasterising from `DccShell.phone_fit()`, which knows the number but only
  reaches a subtree that exists when it runs: `open_project_dialog.gd` builds its
  action and import tiles on `navigate()`, after its one `phone_fit(self, 1.0)`.
  **Measured: the search glyph was fixed and the other three were not**;
- re-rasterising on `tree_entered` — fires before `phone_present()` has set
  `content_scale_factor` for every glyph built during `setup()`.

What works is asking the node at draw time, via `get_screen_transform()` — **not**
`get_global_transform_with_canvas()`, which reads like the right answer: measured on
all four glyphs, `gtwc` scale is (1.0, 1.0) and `screen` scale is (3.664122, 3.664122). A `CanvasLayer` transform is not a viewport's *final* transform, where a
content scale lives; with the wrong call the fix is silently inert, which is how it
first measured. After: all four glyphs at **1.00–1.01** texels per pixel at
1440x3168 and 1080x2400, quantised to 1/16 so a float32 `content_scale_factor`
cannot re-rasterise every frame. Desktop unchanged (12/12, 30/30, 26/26, 15/15).

### HD-03 — the viewport's floating chrome is 2.19 mm on a 510 ppi panel · **FIXED**

`viewport_host.gd` lives in the **main** viewport, which has no content scale, so
every constant in it is a real device pixel. `NAVPAD_HIT`'s comment asserted the
opposite — *"the shipped phone's viewport is ~393 px, where that scale is 1.0"* —
and measured, `get_viewport_rect()` reports 1080x2400 and 1440x3168 and
`_phone_scale` 2.748 and 3.664. A raw 44 px pill is **2.83 mm** on a 395 ppi panel
and **2.19 mm** on the OnePlus 12's, against roughly 7 mm for the 44 dp floor every
other target gets. Not blurry, but the same complaint.

The scale rides through the existing `set_safe_insets()` dictionary rather than a
new setter, so `app.gd` (owned by another agent that session) is untouched. Glyphs
are re-rasterised rather than stretched. Measured: layers button and navpad pills
**44 → 161 px (2.19 → 8.02 mm)** at 1440x3168 and **44 → 121 px (2.83 → 7.78 mm)**
at 1080x2400, icons 17 → 62 and 17 → 47 texels.

### HD-04 — `[display]` has no stretch key, and that is required rather than an oversight · **DOCUMENTED**

Checked, both modes side by side on a 1440x3168 window:

| `stretch/mode` | `get_visible_rect` | final transform | `_phone_scale` would be |
|---|---|---|---|
| `disabled` (shipped, unset default) | 1440 x 3168 | 1.00 | 3.6641 |
| `canvas_items` | 1152 x 648 | 1.25 | 1.6489 |

`canvas_items` breaks the shell three ways: the viewport reports the project's
reference size on *every* device, so `_phone_scale` collapses to one constant; the
stretch transform multiplies that a second time, differently per device; and
1152x648 is **landscape**, so `DccShell._landscape` would read true on a phone in
portrait and §13's portrait composition would be unreachable. Written into the
`[display]` comment block, in semicolons, so nobody "fixes" it.

### PH-05 — one last hole in the touch-scroll fix · **FIXED**

Re-run at `_phone_scale` 2.748 and 3.664 rather than the 393 dp reference: **6 of 8**
points down the left sheet scrolled 329 px. One miss is an `HSlider`, deliberate and
documented. The other is a **bare `Control`** — `DccTheme.spacer()` and the
fixed-width gaps beside it — which defaults to `MOUSE_FILTER_STOP` and ends the
event walk on a node that exists only to take up room. `phone_fit()` now passes it
through, matched on the exact class and skipped if anything listens on `gui_input`.
After: **7 of 8** at 1080x2400 and **7 of 7** at 1440x3168, the remainder the
slider.

### §46's one carried-over item — a `Popup` is a `Window`, not a `Control` · **FIXED**

§46 registered the Layers-sheet/drawer overlap. The cause:
`_close_all_phone_overlays()` walks `Control`s only (drawer, panel picker, phone
menu, both dock sheets), and `LayersPopover` is a `PopupPanel`. Matched on `Popup`
and deliberately **not** `Window`: a popover is transient and going elsewhere is
what dismisses it, while an `AcceptDialog` is a modal the user is inside — closing
one under them trades a cosmetic overlap for lost input. `PopupMenu` is caught by
the same test, as it should be. `find_children(..., owned = false)` because these
are built in code with no scene owner; the default `owned = true` returns an empty
list — another silently-inert fix of the kind HD-02 collected three of. Verified:
popover open → `true`, then drawer opened → popover `false` and drawer `true`.

### Proven negatives — two leads that are not defects

- **`phone_present()` fills the screen correctly; the 31 % fill is a dev-box
  artefact.** `Window.popup(rect)` clamps its rect componentwise to
  `DisplayServer.screen_get_usable_rect()`, on this dev monitor **1680 x 1002** —
  and 1002 is exactly the dialog height two independent harnesses reported.
  Established as a formula: asked 1440x900 → 1440x900 (unclamped), asked 1400x1100
  → 1400x**1002**, asked 393x852 → unchanged, asked 1440x3168 → 1440x**1002**.
  `is_embedded()` is `true` in every case, so embedding is not the discriminator.
  On a OnePlus 12 the usable rect *is* 1440x3168 and nothing is clamped; in the real
  app at a phone-aspect viewport that fits the monitor (440x950) the same code fills
  **100.0 %**. A probe simulating a screen taller than the dev monitor must
  re-assert `size` after `popup()` — the window is visible by then, so the
  assignment raises the resize notification and sticks (measured: 1440x3168).
- **The left dock sheet scrolls.** The "scroll by zero" report came from a sweep
  whose `ScrollContainer` had `max_value` 165.0 against a `page` of 2318 —
  `max_value` is the content length, not the overflow, so there was **nothing to
  scroll**. On a container with real overflow (`max` 5409, `page` 2318) the same
  gesture moves 439 px from 6 of 7 points before the PH-05 fix above and 7 of 7
  after.

### Registered, not fixed

- **`RD-03` — the base map raster is `TEXTURE_FILTER_NEAREST` and the reference
  smooths.** (This `RD-03` collides with §6.8's right-dock `RD-03`; not
  renumbered.) `viewport_host.gd:792` sets NEAREST on `map_view`; the LOD tiles use
  LINEAR and say why. The frozen reference sets `imageSmoothingEnabled` in exactly
  four places, **all in the asset library** (the sprite-sheet slicer and the item
  preview); the map canvas never sets it, so it takes the HTML default, `true`. The
  port diverges on the map's own filter, visibly both ways on a phone: a 512-cell
  grid on a 1440-wide screen is *magnified* 2.8x (the blockiness `docs/HANDOFF.md`
  quotes the owner complaining about), and a 2048-cell grid is *minified* 1.42x
  with no mipmap, which aliases under a pinch. Left alone deliberately: the filter
  belongs to `LOD_TILING_INTEGRATION_SCOPE.md` milestone M1, which
  `viewport_host.gd`'s header names as the thing that closes it, and flipping it
  would change every map screenshot in the repository on a pass about UI chrome.

### Harness

`_hidpi_probe.gd` and `_edge_probe.gd` (new, untracked, like every probe in
`godot-project/`). The first drives the real shell inside a **`SubViewport`**
rather than the real window, because Windows clamps a window to the desktop work
area: `--resolution 1440x3168` came back as 1440x1031 and `DccShell` classified it
as a *tablet* (`phone_scale` 2.62, `_phone` false). A `SubViewport` has no such
ceiling, `get_viewport_rect()` inside it reports its own size, and
`gui_embed_subwindows` keeps the shell's dialogs rendering into the same texture.
`--vp WxH` selects the device. It measures fonts, icon texels, tap sizes in
millimetres and the scroll flick, and it is the first probe here that runs at a
phone scale above 1 — **this entire class of defect is arithmetically invisible at
the 393 dp reference size every earlier probe booted at.**

---

## 48 · DS-01…DS-14 — the conformance sweep: what the shell looks like next to the canvases it was built from (2026-08-25) — **TEN FIXED, ONE PART-FIXED, FOUR REGISTERED**

§46 and §47 fixed phone *mechanics* (tap floors, font oversampling, icon
rasterisation, scroll forwarding); nobody had checked whether the result looks
like the design. The owner asked:

> *"the design tool that really all screens are properly in-line with the design
> style and scale and fit properly per device."*

Every screen was screenshotted at desktop 1600x900, tablet 2560x1600, and phone
1440x3168 (OnePlus 12, `_phone_scale` 3.664) and 1080x2400 (2.748), each beside
the canvas region it implements. `design/` was the specification, not
`DCC_SHELL_SPEC.md`: where they disagree the canvas's inline styles win, because
the spec is prose written *about* the canvas.

### The finding under all the others

**`dcc_theme.gd`'s header says every value in it "is read off the design
mockup, not invented", and eleven of them were not.** Three tokens the canvas
uses constantly did not exist; five had wrong values; the light palette had its
ground and floating surface swapped. Everything resolves through
`DccTheme.c()`, so a wrong token is every control that ever asked for it.

| Token | Canvas | Was | Effect |
|---|---|---|---|
| Ink secondary | `#a9adb0` | **absent** | 76 spans in `DCC shell 1920` alone — every menu-bar title and every parameter-row label, all drawn one step too quiet in `text_dim` |
| Border | `rgba(255,255,255,.16)` | **absent** | every chip, well and action button outlined at the *region* hairline `.10` instead |
| Accent hover | `#f0bd72` | **absent** | §11 lists it; nothing had it |
| Divider (dark) | `.07` | `.06` | — |
| Hairline (light) | `.14` | `.12` | — |
| Divider (light) | `.08` | `.07` | — |
| Ink faint (light) | `#8d9088` | `#7c807a` | — |
| Surface (light) | `#f4f2ee` | `#fbfaf7` | light mode's ground was its raised surface |
| Raised (light) | `#fbfaf7` | `#ffffff` | `#ffffff` appears in neither light canvas |

### Ranked divergences, with the numbers

**DS-01 · The whole menu system was set in Plex Mono. It is prose in the
canvas.** — **FIXED.** `add_menu()` and `style_popup()` set `DccTheme.mono(0)`
at `FS_MENU` 12. The canvas's menu bar is `<div style="display:flex;font-size:11.5px;color:#a9adb0"><span
style="padding:9px 11px">File</span>…`, inheriting `'Helvetica Neue'` — **sans,
11.5 px, `#a9adb0`** — as is every dropdown item, only the shortcut column in
Plex. Three compounding errors on the most prominent row. Now prose at
`FS_MENU_ITEM` 11 in `text_secondary`; `FS_MENU` 12 stays for the wordmark,
which really is `font:500 12px 'IBM Plex Mono'`.

**DS-02 · Every action button was a filled amber slab. The canvas has no filled
buttons at all.** — **FIXED.** `DccWidgets.action(primary: true)` drew
`flat(accent, radius 2)` with `bg`-coloured text. Searching the 1920-wide
document for `background:#e0a34a` returns slider fills and **exactly one** other
hit — a *selected layer row* in the layers popover. Every action in every
artboard (`Run stage 04`, `commit pass`, `Apply`, `Open selected`, `Select · 3`)
is `padding:4px 10px;border:1px solid #e0a34a;color:#e0a34a`. One helper, 141
call sites, 20 primary. `modal_button()`'s comment asserting the opposite ("a
filled accent slab, which is the left dock's own run-this-pass affordance") is
corrected in place.

**DS-03 · Tablet is desktop chrome with three bars stretched.** — **FRAME
FIXED, CONTENT REGISTERED.** `_scaled()` was `maxi(44, round(px * 1.53))`, but
§1's tablet column is not a multiplier: 34→52 is x1.53, 26→36 x1.38, 40→48
x1.20, 29→34 x1.17. Against the `DCC shell tablet 2560` artboard the shell drew
a **61 px rail where the canvas draws 48** and a **44 px status bar where it
draws 36** — the 44 px floor firing on non-tappable chrome. Both docks ran the
desktop 372/300 where §1 says 400. All fixed by an exact `DccTheme.TABLET`
table.

Not fixed: everything *inside* those regions. `phone_fit()` (the walk that
rescales fonts, control heights and stylebox padding) begins `if not _phone:
return`, so tablet gets none of it. Measured at 2560x1600 with `--force-touch`:
sliders **14 px**, action buttons **26 px**, dock labels **11 px**, against §13's
"targets 44–52 px" and the tablet canvas's `font:14px/1.4` prose and `13px`
Plex. The canvas's tablet type scale is 14/11.5 ≈ **x1.22**, and its
tool-options track grows 70x2 → 90x3. Left alone deliberately: a scaling layer
that does not exist, not a wrong value, and bolting `phone_fit` onto tablet with
a guessed unit gives a tablet that is neither.

**DS-04 · Dialog chrome was Godot's stock `#404040`.** — **FIXED.**
`AcceptDialog` draws its own `panel` over the `Window` border and nothing had
set it, so Performance, Gen info, World data and every modal footer came up on a
grey **20 steps brighter than anything in either palette** — sampled `#404040`
at (10,10) and (10,1600) of the phone capture. Now `panel` with a `border` edge,
set once on the theme resource.

**DS-05 · The open menu item's highlight was Godot's blue selection bar.** —
**FIXED.** The one saturated colour in a greys-plus-one-amber shell. Canvas:
`background:rgba(224,163,74,.10);color:#e8ebec`, i.e. `accent_wash`, the menu
bar's own open-title wash. The menu panel also gained
`border:1px solid rgba(255,255,255,.14)` (it had the `.10` region hairline) and
`box-shadow:0 14px 34px rgba(0,0,0,.55)` (it had none), and moved from `raised`
`#17191a` — §11's **viewport wash centre stop**, not a menu surface anywhere —
to `panel` `#121314`, which the canvas draws.

**DS-06 · The parameter row — the most repeated component in the shell — was
wrong in three dimensions.** — **FIXED.** Canvas:
`<span style="flex:1;color:#a9adb0;font-family:'Helvetica Neue';font-size:11px">Continentality</span><div style="width:78px;height:2px;…"><div style="width:33%;…background:#e0a34a"></div></div><span style="width:44px;text-align:right;color:#c8cbcd">0.30</span>`.
The shell drew the label in **mono** `text_dim`, gave the track
`SIZE_EXPAND_FILL` (**128 px** at a 372 px dock, growing with it) and reserved
**56 px** for a 44 px value column — which is why "Enable continental shelves"
clipped to "Enable continental s" beside a bar with room to spare.

**DS-07 · Tabs were stock Godot.** — **FIXED.** `TabContainer` draws its strip
from an internal `TabBar` no walk in `dcc_shell.gd` reaches, so World data,
Travel library, the Asset library and the Data manager showed a raised grey pill
with a white top rule. The design's two-way switch is the left dock's
`GENERATION PIPELINE | SCULPT` header —
`background:rgba(224,163,74,.10);border-bottom:1px solid #e0a34a` when on,
nothing when off — which is `DccTheme.active_row()`.

**DS-08 · Both dock headers were 26 px. The canvas gives them 34.** —
**FIXED.** 26 is `H_STATUS`, borrowed by mistake; measured 30 px including its
rule beside a 34 px menu bar and 34 px tool options bar, so the three top rules
did not line up.

**DS-09 · A 74 px hole between the wordmark and File.** — **FIXED.** The
wordmark claimed `custom_minimum_size.x = 150`; the canvas gives it
`margin-right:22px` only. Measured: wordmark ended x=104, `File` began x=178.

**DS-10 · The rail's domain labels were a size down, a tracking up and a weight
up, all at once.** — **FIXED.** `FS_MICRO` 9 at `spacing 2` (≈.22 em) in Medium,
against `font:10px 'IBM Plex Mono';letter-spacing:.12em` regular. Resting ink
also brightened to `text_faint` after the first domain selection, because
`_select_domain()` restored a different token than `_build_rail()` painted
(`#5f6468` is correct).

**DS-11 · The status bar was prose. The canvas sets the whole bar in Plex.** —
**FIXED.** `font:10.5px 'IBM Plex Mono';color:#6f7478`, both themes, desktop and
tablet artboards, including the right-hand modifier hints — which were also a
step quieter at `text_ghost`.

**DS-12 · World data was a six-column spreadsheet folded in half.** —
**FIXED, and designed rather than matched.** §46's PH-12 kept six columns and
broke each record over two lines: it measured clean and read as a folded
spreadsheet — a bare `Class Population Faction Coastal Capital` band under a
column header over no column, and five unlabelled values to count along.

There is **no canvas for this window**, so the replacement derives from the one
phone list the design draws — `design/Cartalith Android Phone.dc.html` screen
`03 Category`, row
`min-height:52px;padding:0 16px;gap:12px;border-top:1px solid rgba(255,255,255,.06)`,
a prose primary line over a `font:9.5px 'IBM Plex Mono';color:#5f6468` summary.
The five columns become that summary, each carrying its own word
(`capital · pop 20 655 · faction 3 · coastal`, not a `yes` in the fifth slot),
and the false column header goes. The chevron is deliberately not copied: these
rows have no drill-down, and an affordance that does nothing is the failure this
pass exists to find.

**DS-13 · The phone's map buttons.** — **ALPHA FIXED, COMPOSITION
REGISTERED.** `_navpad_paint()` filled with `panel` at full alpha; every floating
map control in the phone canvas is `background:rgba(20,22,23,.92)`. Now matched —
a small change: at 92 % over mid-tone terrain the disc lands about four levels
lighter.

Registered, not fixed: the canvas's `01 Viewport` puts **two** controls on the
map — a 48 dp accent FAB (`background:#e0a34a;color:#141617`, the one filled
surface on the phone screen) over a 44 dp secondary pill — plus three layer chips
top-left and a `POINT SAMPLE` readout card at the bottom. The shell draws **four
identical dark pills in a vertical column** (zoom in, zoom out, pan, fit), no
chips, FAB or card. The functions differ, so this is a composition to design in
the canvas's grammar; inventing a FAB for a function the canvas never assigned
one is what "no canvas, derive don't invent" exists to stop.

**DS-14 · Radius 2 on dock and dialog controls.** — **FIXED.** §11: "Radius 0
everywhere." Nine sites across `dcc_widgets`, the faction roster, the new-world
dialog, the place editor and two workspaces. `viewport_host.gd`'s navpad pills
and `resource_overlay.gd` keep their radii on purpose: the phone canvas draws
*floating map* chrome round (`border-radius:14px` chips, `24px` FAB, `12px`
readout card) and only panel chrome square.

### Also found, not a design question

**`performance_window.gd` was shipping the literal string `%.2f`** where the
working-set figure goes — the `%` operator was missing. Caught by screenshot,
not by any test: a `#[func]` returning a figure proves nothing about whether it
reaches a `Label`.

**A second `Close` button paints at the top of every full-bleed phone window**
(`performance_window`, `gen_info_dialog`, `world_data_window`), pre-existing and
seen only in a `SubViewport`-hosted capture; suspected a harness artefact and
left for the next Android pass. §50 confirmed it on the handset as an artefact
(see §50's proven negatives).

### Registered, not fixed

- **The canvas paints no regions; the shell paints all of them.** §11 says "No
  fills on panels: regions are separated by hairlines only", and the canvas
  obeys literally — menu bar, docks, rail and status bar are `<div>`s with a
  `border-*` and **no `background`** on the artboard's `#0d0e0f`. The shell fills
  each with `panel` `#121314`: dock body (18,19,20) where the canvas would be
  (13,14,15). Left to the owner because §11's own token table lists Surface and
  Panel as two tokens and the Android Phone canvas *does* shade its bottom nav
  (`#131516`) against its ground (`#101112`) — the canvases disagree.
  > **RULED 2026-08-25 — keep the fills.** The owner chose the shell's
  > behaviour over the desktop canvas's. The `#121314` regions are **not** a
  > divergence and must not be logged as one or "fixed" by a later pass. §51's
  > menu-by-menu walk was run under this ruling and logs none.
- **The phone's permanently-resident tool options bar costs the map 11.6 points
  of screen.** §13 says "Tool options become a bottom sheet" — summoned, not
  resident. At 1440x3168 the map ends at 698.1 dp of 864.6 (**69.6 %**); the
  canvas's `01 Viewport` gives it 724 of 892 (**81.2 %**). The bar also clips
  its last item ("Center land…") at the right edge with no scroll affordance. A
  sheet is a behaviour change, so registered.
- **§46's asset-library toolbar proportion is worse than that section
  recorded.** At 1440x3168 after §47's fixes the first asset tile begins at
  **342 dp of 864.6 = 39.6 %** of screen height, not ~24 %. Header 33 + search
  63 + two chip rows + tab strip + status strip + batch verb row, all resident,
  none collapsible. A composition fix, not a token.

### Two canvases disagree about the phone, and it is worth saying so

`design/Cartalith DCC Shell.dc.html`'s `DCC shell android phone` is authored at
**393 dp** with a 44 dp safe area, a 52 dp app bar and a **44 dp vertical
domain rail**. `design/Cartalith Android Phone.dc.html` — eight screens, much the
more developed — is authored at **412 dp** ("412 × 892 dp · all five disclosure
levels · 44 dp rows", its own header) with a 28 dp status row, a 56 dp app bar
and a **64 dp five-tab bottom nav** carrying a 14 px glyph over a 9.5 px label.
`DccTheme.PHONE_REF_SHORT` is 393, so everything is scaled against the first;
the shell's 64 dp bottom bar matches the second but carries labels only.
`PHONE_REF_SHORT` is the unit every phone constant is expressed in, so a 4.8 %
error in it is a 4.8 % error in all of them.

> **RULED 2026-08-25 — adopt 412 dp fully.** `design/Cartalith Android Phone
> .dc.html` is the phone authority from here. `DCC shell android phone`
> (393 dp) and `DCC_SHELL_SPEC.md` §13's phone column are superseded for the
> phone device class; desktop and tablet are unaffected and `Cartalith DCC
> Shell.dc.html` remains authoritative for both. Nothing is broken today —
> every shipped phone constant matches the 393 canvas coherently — so this is a
> deliberate redesign, not a defect. **The constant migration is its own pass**
> and was not started by §51; that section carries the measured per-region
> divergence list it needs, and names the one conflict (the 412 canvas's
> bottom-nav tabs are the pre-v3 domain set) that pass must resolve rather than
> copy.

### Harness

`_designconf_shot.gd` / `.tscn` (new, untracked). Hosts the real shell in a
`SubViewport` for §47's reason (`--resolution 1440x3168` is clamped to the
desktop work area, comes back 1440x1002, and `_compute_layout_mode()` calls that
*tablet*), sweeps 20 named screens, dumps every tappable height and font size
with its face, and screenshots each. `--vp WxH --tag NAME [--force-touch]`.

One trap cost a full pass: the probe's teardown called
`app._set_overflow_open(false)` unconditionally, and
`_close_all_phone_overlays()` sets `left_dock.visible = false` with **no phone
guard** — so the first desktop run measured a shell with no docks, and nearly
reported that as the finding.

---

## 49 · KV-04, WW-16 — two things the shipped app says that a real boot disproves (2026-08-25 audit pass 4) — **REGISTERED, NOT FIXED**

Audit pass 4 booted the real shell windowed (`_audit4_probe.gd`, §47's
`SubViewport` idiom) rather than reading about it. Two findings neither the
register nor `STATUS.md` had, both *a document or disclosure asserting something
the code contradicts*. Both are **code** changes, so registered for the owner
and deliberately not made by an audit pass.

### KV-04 · The Markdown Vault's link store has never survived a restart

**Severity: data loss in shipped, milestone-complete functionality.** Every
link a user attaches is silently discarded on the next launch. The boot printed
it unprompted:

> `WARNING: Cartalith: user://markdown_vault.json holds a link store this
> engine could not read; nothing was loaded.` — `vault_store.gd:66`

**The mechanism, isolated to one line.** `VaultStore.save_from`
(`godot-project/shell/vault_store.gd:118-135`), the **only** writer of the
sidecar, re-parses the engine's JSON instead of writing it:

```
var state := bridge.vault_state_json()     # correct JSON, from Rust
var parsed = JSON.parse_string(state)      # every number becomes a float
... f.store_string(JSON.stringify(doc, "  "))
```

Godot's `JSON` types every number as `float`, so `KnowledgeLink::entity_id`
(`crates/cartalith-vault/src/links.rs:175`, `i64`) is written back as `1.0` and
`source_modified` (`:183`, `u64`) as `1787605785.0`. `serde_json` refuses both,
`LinkStore::from_json` errors, `vault_restore_state`
(`crates/cartalith-godot/src/vault_bridge.rs:640`) returns `false`, and
`load_into` warns and returns — correct for a corrupt sidecar, exactly wrong for
one this app wrote itself.

**Proved by bisection** (`_audit4_vault.gd`, four cases against the real
sidecar on disk):

| Case | Restores? |
|---|---|
| A — `vault_state_json()` handed straight back to `vault_restore_state()` | **true** |
| B — the shipped sidecar exactly as `save_from` wrote it | **false** |
| C — B with every `entity_id` coerced back to `int` | **false** |
| D — C with `source_modified` and `version` coerced too | **true** |

The round trip fails on **two** integer fields, and the engine's output is fine.
**The fix is to stop re-parsing**: keep `state` as a string and splice it in, or
restore through the untouched string. Case A proves nothing on the Rust side
needs to change. `vault_restore_state`'s own rule — *"a corrupt sidecar must not
take the links that are in memory with it"* — here guarantees the links are lost
quietly instead of loudly.

### WW-16 · The five erosion Run buttons say the kernels do not exist. Four of them run inside `generate()`

`world_workspace.gd:795-800` builds, for each of *Droplet hydraulic*,
*Hillslope diffuse*, *Velocity (momentum)*, *Glacial* and *Coastal*, a note
reading *"Not ported -- a separate manual pass in the reference with no
cartalith-engine equivalent."* and a disabled button whose tooltip reads *"No
cartalith-engine implementation exists for this pass."*; `:87`'s stage-06 `gap`
string says it a third time.

**All three statements are false, and the file contradicts itself twenty lines
apart** — the loop at `:790-793` builds live parameter rows from the `erosion`
param group, which is the wiring for these very passes:

| Control | Kernel | Called from |
|---|---|---|
| Hillslope diffuse | `cartalith-erosion/src/passes.rs:103` | `cartalith-engine/src/lib.rs:1387` |
| Velocity (momentum) | `passes.rs:245` | `lib.rs:1342` |
| Glacial | `passes.rs:514` | `lib.rs:1346` |
| Coastal | `passes.rs:710` | `lib.rs:1370` |
| Droplet hydraulic | `cartalith-erosion/src/lib.rs:126` | golden-tested; **no engine caller** — "ported, not wired", still not "does not exist" |

All are bit-exact ports with golden fixtures, landed 2026-08-23 — and **this
register recorded it**: §19 (*"kernels ported, wired as generation
parameters"*), whose §19.2 closes:

> *"(a) is **not** foreclosed and is now cheap: the kernels and the run path
> both exist, so a run button would be a `#[func]` over the same code. It was
> not built in this pass because UI work is on hold (`CLAUDE.md`)."*

**That blocker was void when written**: the UI hold was lifted 2026-08-18
(`DCC_SHELL_SCOPE.md`; `CLAUDE.md`'s copy was not corrected until 2026-08-23),
and §19 is dated 2026-08-23. Five controls stayed dead for two days on a hold
lifted for five.

**Class: (A)** — designed, engine-ready, no Rust model missing. One `#[func]`
per pass over `ErosionPassParams`' existing kernel calls plus the reference's
`erodeFinish`/`eroFinish`/`veloFinish` re-derivation, and three strings deleted.
`WorldGen::carve_fjords()` — live, same group, built the same way — is the shape
to copy.

**Minimum honest action, if the buttons stay disabled:** the three strings stop
claiming the implementation does not exist. *"the kernel is ported and runs as a
generation parameter; there is no `#[func]` to run it on its own against a
finished field"* is true and matches §4's convention.

### CV-12's disclosure is stale in both of its clauses

`civilization_workspace.gd:1200` (*Settlement diagnostics overlay*) says:

> *"cartalith-urban milestones 8-17 are unported **and the crate has no
> consumer at all**"*

Neither half survives a grep. **Milestones 1-7, 8a, 12 and 17a are ported**
(`URBAN_MORPHOLOGY_SCOPE.md`; `PARITY_AUDIT.md` pass 3 §17 — 8a and 12 landed
2026-08-24, the day before this text was last touched), and the crate has a real
consumer: `crates/cartalith-civ/Cargo.toml:22` depends on it and
`crates/cartalith-civ/src/urban_adapter.rs:98` imports from it, feeding the
bridge, the deep-zoom map layer and the City Viewer.

**The conclusion still holds** — `_umWallSpec`, `_umSiteProfile` and
`_umModelCache` are among the five unported `_um*` adapters, so the overlay would
have nothing to draw. A **stale reason on a correct verdict**, §4's S1-S5 shape:
fix the sentence, leave the control.

*Superseded, 2026-09-24: the conclusion no longer holds either.* The adapters
were ported (`urban_adapter.rs`'s `um_site_profile` and
`military.rs`'s `um_wall_spec`, among others). Since 2026-09-02, `urban_bridge.rs::settlement_diagnostics` has fed a
working per-settlement Diagnostics section in the CIVIL dock
(`_build_settlement_diagnostics`). Only the reference's on-map footprint box is
still not drawn.

---

## 50 · PH-13…PH-17 — the four passes that had never run on a phone, driven on one (2026-08-25) — **THREE FIXED, SIX REGISTERED, ELEVEN PROVEN**

§46 (nine phone windows), §47 (hi-DPI), §48 (design conformance) and the
`/ponytail` LOD parallelisation all landed in one day and **none had touched real
hardware**; the owner's OnePlus 6T APK was from 09:19 that morning.

Hardware: **OnePlus 6T** (`ONEPLUS_A6013`, LineageOS 22.2 / Android 15),
1080 x 2340, `DccShell._phone_scale` **2.748**, Adreno 630 on OpenGL ES 3.2.
6.41 in diagonal, so **401.6 ppi = 15.81 px/mm**; every millimetre below is a
screenshot measurement divided by that.

**The scale limit, stated first.** The owner reported the blur on a **OnePlus
12** at `_phone_scale` **3.664**. Everything below confirms the §47 fixes *in
kind*, **none at the scale he saw**; a defect that scales with that number is
33 % worse there. Nothing here closes the OnePlus 12 report.

### Ranked: what is actually broken on the device

**PH-16 · The Journey Planner's centre panel is 1 434 px of nothing.** —
**REGISTERED, NOT FIXED.** CIVIL ▸ Data ▸ *Journey planner…* draws a uniform
`panel` `#121314` field from y = 265 to y = 1 699 — **61 % of the screen,
91 mm** — in which **not one pixel exceeds RGB(23, 23, 23)** (every fifth row,
full width). The only journey content is a two-line
`§ ROUTE TOTALS / No committed route selected.` strip *below* it and the
tool-options row's `JOURNEY PLANNER · journey · (no committed route)`. No
route-map frame, no `PROFILE · STAGE SELECTOR` header, no stage matrix, no
inspector — none of what §46 describes as *"both stack"*.

It is not a "no route yet" empty state: `_build()` draws a `PanelContainer` per
row with a bottom rule, and at `_pp(236)` / `_pp(150)` those rules would land at
y = 917 and y = 1 329; neither is on the framebuffer. And `_show()` sets
`app.viewport.visible = false`, so **the map is gone too** — a black rectangle
with no explanation.

Not chased, per this document's rule from the press-and-hold hunt: *"Diagnose
there; confirm on the handset."* Each device iteration is an export, an
`adb install` and a re-navigation, and `print()` from GDScript does not reach
`logcat` on this build. `_ph9_probe.gd` at `--vp 1080x2400 --force-touch`, dumping the rects
of `_center_panel` / `col` / `map_row_pad`, is the two-minute answer.

**PH-13 · The 44 dp floor on the dialog Close button produced a *clipped*
button, not a bigger one.** — **FIXED.** §46's fix five raised
`get_ok_button().custom_minimum_size` after `popup()` (which clears it). It
survives — but `AcceptDialog` had already seated its button bar for the stock
29 dp button, so the taller one grew **downwards out of the window**, where the
subwindow clips it.

| Window | Close/OK visible height | after |
|---|---|---|
| World data | 84 px = **5.31 mm** | 131 px = **8.29 mm** |
| Gen info | 84 px | 131 px |
| Performance | 84 px | 131 px |
| Credits | 84 px | 131 px |
| New World (Cancel + Create) | 78 px | 124 px |

**Proved, not inferred**: the glyph sat at y = 2 245, the centre of the *full*
121 px box, against a window bottom border at 2 266-2 268; a merely-short button
would have centred it at 2 226.

**Three ways of asking the engine to re-lay were tried on the handset and all
measured the same** — the part worth keeping:

| attempt | result |
|---|---|
| `Window.child_controls_changed()` | 82 px — defers to `_update_window_size()`, which with `wrap_controls` off finds the size unchanged and raises nothing |
| `size` assigned immediately after the floor | 84 px — `custom_minimum_size` queues `update_minimum_size()`, so a same-call relay is told the *stock* size |
| the same assignment via `set_deferred()` | 84 px — queue order was the wrong theory too |

The bar is seated **once**, by `popup()`, and nothing `phone_present()` can reach
repeats it. So `DccWidgets._floor_dialog_bar()` does the arithmetic:
`hbox.size.y` still holds the stock height — **the staleness is the input, not
the obstacle** — so the shortfall is `PHONE_TAP_MIN - hbox.size.y`, the bar moves
up by it, and the content child shrinks by the same so they cannot overlap.
Idempotent, and self-healing if a future engine does relay.

**A fourth build was needed**: seating the 44 dp button where the 29 dp one
ended still measured clipped (2 144-2 268 against a border at 2 266-2 268),
because **`AcceptDialog` gives its bar no bottom margin at all** — the button's
border and the window's were the same two pixels. A 12 dp foot (`category()`'s
own `DccTheme.inset(12, 0, 12, 0)`, not a number chosen for a screenshot) makes
it read as a button.

**PH-14 · The Layers button grew its hit rect and kept its old paint.** —
**FIXED.** HD-03 grew it from 44 to 121 px and re-rasterised its glyph; on glass
it still looked like a clipped 2 mm icon. Two independent causes:

1. `Button.icon_alignment` defaults to **`LEFT`**, so the 36 px glyph sat in the
   box's top-left *corner* — x 2-37, y 307-342 — with 84 px empty beside it; and
   `phone_content_insets()` returns `left = 0` in portrait (the map is
   edge-to-edge), so that corner is the panel edge.
2. `flat = true` **suppresses the background stylebox entirely** — the trap
   `_navpad_button()`'s own comment records paying for once (*"the first cut was
   flat, and the pills were invisible over the terrain with only their glyphs
   showing"*); `MISTAKES.md`'s *Override a stylebox on a `Button`* row. So
   `text_dim` grey drew straight onto the biome; over coastal scrub with a
   settlement pin behind it, unreadable.

Now the navpad's 92 %-alpha scrim pill at the same radius, glyph centred, with
`NAVPAD_EDGE` (the canvas's `right:14px`) as a floor on the left inset. Touch
only — desktop's 26 px flat glyph is untouched.

**PH-15 · A scroll flick on a phone menu sheet activates the row it starts
on.** — **REGISTERED, NOT FIXED.** Three identical 250 ms upward flicks from
(540, 1 335) on the MENU ▸ Preferences L3 sheet: the third opened *Working set…*
(the Performance window) instead of scrolling; the same class had opened *Theme*
as an L4 sheet during an earlier two-flick scroll. A 700 ms drag from the same
point scrolls cleanly, so this is a drag-threshold / press-cancel question in the
sheet's row buttons, not PH-05 again. The likeliest single source of the owner's
*"doesn't listen well to touch input"*; wants the desktop harness, not a guess.
(§52 reproduced it at 700 ms too.)

**PH-17 · `DccIcons.SYMBOLS["add"]` drew as a tofu box.** — **FIXED.** It was
`＋` **U+FF0B**, a *fullwidth* plus in the CJK compatibility block, carried by
Noto Sans CJK and by none of the Noto Symbols faces a non-CJK Android build
installs. It rendered as a literal `FF 0B` box in the Travel Library's ENTRIES
header, and would have in `DccWidgets.advanced()`'s L5 sigil and two Asset
Library strings. Now ASCII `+`, which Plex Mono has natively.

Re-parsed from the shipped cmaps: `dcc_theme.gd`'s claim that Plex Mono is
*"missing seven"* of §12's symbols was stale by more than 2x — **19 of the 24
entries in `DccIcons.SYMBOLS` have no glyph** in `IBMPlexMono-Regular.ttf`, and
`FiraSans-Regular.ttf` is worse. Corrected in place.

### Registered, not fixed

- **Labels clip without an ellipsis.** World data ▸ Economy rows end
  `…silver, clay, buildst` — a hard cut, no affordance. §46 records that
  `phone_fit()`'s ellipsis pass reaches only `Button`s; this is the `Label` half,
  now visible in shipped content.
- **DS-12's summary line prints the class twice.** `capital · pop 20 708 ·
  faction 4 · capital` — first token the settlement class, last the capital
  flag. DS-12's worked example (`capital · pop 20 655 · faction 3 · coastal`)
  implies the flag drops when it repeats the class.
- **The navpad's first pill keeps a hover tint after a tap.** (58, 60, 61)
  against (18, 19, 20) on the other three, indefinitely: Android leaves the
  emulated pointer where the finger last was.
- **The pane switcher's focused state is stock Godot.** The Data manager's
  `ROUTES` chip draws a rounded raised grey pill once tapped, against `ROUTE`'s
  correct `active_row()` amber wash — DS-07 restyled selected, not focused, and
  DS-14's "radius 0 everywhere" does not reach it. Invisible on desktop,
  permanent on touch.
- **The MENU sheet's L2 header carries two `×`** — top-left, where the L3 sheet
  correctly draws a `‹` back chevron, and top-right. Both close.
- **The app's own Memory row under-reports by about 4x on Android.** It read
  **0.2 GB** while `dumpsys meminfo` reported **818 MB** TOTAL PSS for the same
  process at the same moment. `performance_window.gd` names its source
  (`OS.get_static_memory_usage()`), but the Rust allocations and the 544 MB of
  `Gfx dev` are outside it. (Fixed as §52's MEM-04.)

### Memory, like for like

Cold boot, one 2048 x 1311 generate from the welcome screen, `TOTAL PSS` sampled
continuously — the metric the three previous device passes used:

| Run | Peak | Steady |
|---|---|---|
| 2026-08-18 | 874 MB | ~510 MB |
| 2026-08-20 | 878 MB | 647 MB |
| **this pass** | **1 033 MB** | **818 MB** |

Read at the time as **+18 % peak and +26 % steady since 2026-08-20** on an 8 GB
handset, undiagnosed. A separate, *dirtier* sample after ~25 minutes of driving
every window plus deep zoom reached **1.82 GB** peak / 1.28 GB steady with
**544 MB in `Gfx dev`**. §52 diagnosed both: the rise is canvas vertex buffers
(MEM-02), the 544 MB is a zoom cost of the dirty sample, and the unfixed-seed
comparison above is not a supportable baseline (MEM-03).

### Proven negatives — eleven things that are genuinely fine

Kept so the next pass does not re-walk them.

- **§48's "second `Close` at the top of every full-bleed phone window" does not
  exist on the handset.** Rows 0-160 of World data, Gen info and the credits
  sheet are **uniform `panel` `#121314`**: maximum channel sum **57** across the
  full width, against **703** where real text is. A harness artefact, as §48
  suspected.
- **Deep-zoom panning is a locked 60 Hz.** 125 SurfaceFlinger present timestamps
  across four continuous pan drags at a 5.7 km span: **median 16.7 ms, p99 16.8,
  max 16.9, zero frames over one vsync** (idle in the same session: max 17.2).
  The `/ponytail` row-parallel `amplify_region` / `add_zoom_detail` /
  `shade_tile` holds up on an Adreno 630.
- **A zoom notch costs at most a 117 ms hitch.** 125 frames across four zoom-in
  taps: median 16.7, p99 **100.1**, max **117.0**, 8 frames over 16.8 ms and 4
  over 33, against `PERFORMANCE_BENCHMARKS.md` §5's pre-parallelisation
  **1.3-1.8 second frozen frame on one wheel notch**. Visible; no longer a stall.
- **HD-03's growth is real on glass.** Navpad pills **117 px = 7.40 mm** at
  401.6 ppi, against 44 px = 2.78 mm before; DS-13's 92 % scrim is live, terrain
  reading through at (102, 104, 104) over bright sand and (24, 29, 30) over dark
  water.
- **Every screen in the sweep is crisply rasterised** at `_phone_scale` 2.748 —
  HD-01's `oversampling_override` works. **Not confirmed at 3.664.**
- **PH-05's touch scroll works everywhere it was checked**: Layers sheet,
  world-data list, credits body, preferences sheet.
- **DS-02 holds on glass.** No filled amber slab in the sweep; every action is
  an outlined amber chip — welcome, New World, Asset Library, Layers, all four
  dialogs. (§52 found one exception outside this sweep.)
- **PH-12 fix five is live**: the credits body renders its full attribution (0
  characters before §46).
- **§48's `performance_window` `%.2f` literal is gone**: *"Working set:
  0.21 GB"*.
- **The desktop-only font fallback list is not the bug it looks like.**
  `DccTheme.mono()` names `Segoe UI Symbol` / `Segoe UI` / `DejaVu Sans`, none on
  Android — and `▸ ✕ ☰ ● ○ ▤` rasterise anyway, because
  `SystemFont.allow_system_fallback` defaults to `true` and Godot's Android
  backend walks `/system/fonts`. Exactly one codepoint had no glyph anywhere
  (PH-17). Do not "fix" the list.
- **`logcat` is clean**: zero `SCRIPT ERROR`, `USER ERROR`, `USER WARNING`, Rust
  panic or `Cartalith: the loaded GDExtension has no …` across a cold boot,
  three full generates and the whole sweep, on a `.so` and APK sha256-verified
  against each other.

### Two things this pass could not do, said plainly

- **Landscape was never observed.** `project.godot` sets `SCREEN_SENSOR`, which
  follows the accelerometer and overrides `settings put system user_rotation`;
  the phone cannot be rotated over `adb`. Every measurement is portrait.
- **The positive control the 2026-08-24 pass asked for is still missing** —
  proof that `push_warning` reaches Android's `logcat` before a silent log is
  trusted as evidence of a matched shell/engine pair. So the clean `logcat` above
  rests on what that pass called *"an argument, not a measurement"*.

### Also worth knowing

- **The map does not pan by a bare drag** — the hand tool must be armed first
  (0 changed pixels before, 26 585 sampled changes after). Not a defect: the
  phone canvas's `01 Viewport` gives pan its own floating control — but it is
  the first thing a finger tries.
- **Generation timing.** 2048 x 1311, read off the app's `Pass` row: **25.1 s**
  cold, **24.8 s** and **25.8 s** warm. `ANDROID_BUILD_SCOPE.md` §3.9's
  2026-08-20 "roughly 16-18 s" is **not** comparable — it was *"inferred from the shape of
  the memory trace rather than an instrumented timer"*. This is the instrumented
  baseline, not a regression against the old.

---

## 51 · MN-11…MN-24 — the menu-by-menu pass: every menu, every submenu, three device classes (2026-08-25) — **TWENTY-ONE FIXED, EIGHTEEN REGISTERED**

§48 walked *screens*; it did not walk the menus one at a time. The owner asked:

> *"I also want you to compare the actual windows/tablet and phone GUI side by
> side, menu by menu with what has been made in Claude design. Every menu should
> match with it."*

Forty-five distinct menus — the seven program menus, thirteen submenus, the
three rail domain accordions, four right-dock contexts, the map context menu,
six tool-options rows, the Layers popover, every `OptionButton` dropdown, and
seven phone surfaces — each opened at four sizes, dumped row by row with its
resolved theme, screenshotted, and set beside its canvas region. **72
menu×device rows, 39 divergences, 21 fixed here.** The full per-menu,
per-device table with measured deltas is the deliverable this section
summarises.

### The two findings that are bigger than any single menu

**MN-11 · Every menu row on a 2560×1600 tablet was 21 px.** — **FIXED.**
`DCC_SHELL_SPEC.md` §13 states a 44 px minimum "measured inside the safe area,
with no exceptions", and §48's `DccTheme.TABLET` table made the tablet *frame*
obey it. Neither `add_menu()` nor `style_popup()` had ever looked at `_touch`.
With `--force-touch` at 2560×1600 the menu bar was 52 px tall carrying seven
**40×51 desktop-sized titles in 11 px type**; `File`'s popup was
**byte-identical to the desktop's**, 467×344 at font size 11 with
`v_separation` 7 — a 21 px row, *less than half the floor on every row of every
menu*. The tablet canvas is unambiguous: `DCC shell tablet 2560` draws its open
Data menu at `font-size:14px`, `padding:9px 18px 9px 30px`, `min-height:44px`,
with `padding:15px 15px` on the bar titles.

Now a measured desktop/tablet table (`DccTheme.MENU`) shaped like
`DccTheme.TABLET`, for the same reason (§1's tablet column is drawn figures, not
a multiplier). Desktop row 21 → **28** (`padding:6px 14px` on an 11.5 px line is
28.7); tablet row 21 → **44**; bar title 11 → 14 px; `File` 40×51 → 53×51.

`PopupMenu` has no per-item height, so the pitch comes through `v_separation` —
dead space *between* rows, while `hover` draws on the row rect alone, giving a
tall menu with a short highlight. The box's `expand_margin` claims the gap back;
together they draw the canvas's full-bleed padded row.

**MN-12 · Every `OptionButton` in the application opened Godot's stock popup.**
— **FIXED.** `dark_theme.tres` defines **no `PopupMenu` type at all** (grepped),
so every popup falls through to Godot's theme unless overridden, and
`style_popup()` was a `DccShell` instance method only the program menus reached.
Every dock picker, dialog select, the paint target, bake depth, asset-library
sort, the journey planner's transport and pace columns and the city viewer's
picker opened a `#0f0f0f` panel with a grey selection bar — DS-05's defect in
fifteen more places.

Fixed at the choke point: the body moved to static `DccWidgets.style_popup()`,
`DccShell.style_popup()` delegates, and `DccWidgets.choice()` calls it on
construction; the six hand-built `OptionButton`s outside that factory are
patched individually. A static factory has no node to reach the shell's
`_touch`, hence `DccTheme.set_touch()` / `is_touch()`, published once from
`DccShell._ready`.

### The one that had never drawn at all

**MN-13 · `flat = true` was suppressing three styleboxes, one of them the menu
bar's own open-menu indicator.** — **FIXED.** A `Button` with `flat = true`
skips its `normal`/`hover`/`pressed` styleboxes outright (`MISTAKES.md`'s
*Override a stylebox on a `Button`* row; `viewport_host.gd` already records
paying for it twice on the Layers button). Three more sites had it, each with an
override that had never appeared on screen:

- `add_menu()` set `pressed` to `active_row()` — the canvas's open title,
  `color:#e0a34a;background:rgba(224,163,74,.08);border-bottom:1px solid
  #e0a34a`. Sampled with File open: background `#121314` (18, 19, 20),
  **identical to the closed `Edit` beside it** — the most prominent state cue in
  the application invisible since the shell was built. Now (34, 30, 24) open
  against (18, 19, 20) closed, with the accent underline.
- `_build_rail()`'s domain buttons had a `hover` box: the rail gave no pointer
  feedback.
- `_phone_list_row()` had one too: the phone's MENU and drawer rows gave no
  press feedback.
- `layers_popover.gd`'s selected row had `active_row()`: the picked layer showed
  only by accent ink and badge opacity. **The first capture of MN-22's fix showed
  a *blank* row where the slab should be**, which is how this older dead override
  was caught.

Rule: **an override on a flat `Button` is a comment, not a style.** Grep before
adding another.

### Ranked, with the numbers

**MN-14 · Every `add_separator()` in `menus.gd` was unlabelled, and the
separator font had never been set.** — **FIXED.** The canvas's group band is
`padding:9px 14px 4px;font:9px 'IBM Plex Mono';letter-spacing:.18em;
color:#5f6468`, appearing nine times across three drawn menus —
`STORAGE LOCATIONS`, `ACTIVE PACK`, `EDIT`, `BATCH · 12 SELECTED`, `BUILD`,
`IMPORT`, `EXPORT`, `SOURCES`, `VALIDATION`. Godot draws that from
`font_separator` / `font_separator_size` / `font_separator_color`, none set, so a
labelled separator would have been **prose at 13 px in `text_faint`** — and there
were no labels anyway. It lands twice: `phone_menu.gd`'s header named it as that
file's one shortfall (*"the moment a separator is given text it becomes a titled
band with no change to this file."*) — it did.

**MN-15 · The Data menu offered four of the canvas's fourteen destinations.** —
**FIXED.** `DCC shell tablet 2560` draws Data as a `⧉ DATA MANAGER` head over
**four labelled bands carrying fourteen indented rows**, each with a trailing
badge — `Heightmaps · PNG · TIFF`, `World Data · .zip · fields`,
`Assets · → Assets`, `Maps · image · tiles`, `Assets · pack .zip`,
`Check Data · 8 warnings`. The shell collapsed each band into one row
(`Import ▸ Maps · Heightmaps · GIS · World data`) opening its group's *first*
route: **ten of fourteen routes had no path from the menu bar**, and no row named
the Data manager window itself. Rows are now generated from
`DataManagerWindow.ROUTES`, which already carries the canvas's label and badge
per route, each opening its exact route through a new `open_route()` /
`open_data_manager_route()` pair.

**MN-16 · `Assets ▸ Asset pack` was three nested submenus where the canvas draws
one panel.** — **FIXED.** The canvas is one 306 px panel with four labelled
bands — `ACTIVE PACK` (five read-only rows plus Pack metadata…), `EDIT` (seven),
`BATCH · 12 SELECTED` (five), `BUILD` (four) — and `Clear library… destructive`
at its foot; the shell had `Edit ▸`, `Batch ▸`, `Build ▸` child popups and no
foot row. Flattened: three popups became zero, removing three places for
**MN-10**'s trap (a submenu's `id_pressed` does not bubble to its parent in
Godot 4). `BATCH` ships without `· 12 SELECTED`: the selection lives in the
window, and a faked count is what this pass exists to find.

**MN-17 · The File menu's order was the canvas's order inverted, and five rows
were missing.** — **FIXED.** The canvas puts `Close project ⌘W` **last**, after a
`STORAGE LOCATIONS` band listing four roots read-only above `Change locations…`
and `Show project on disk`; the shell had Close directly after Revert, one
`Storage locations` row and no band. The four roots are live, elided to their
data-dir-relative tail (`…/Worlds`, `…/Cache/atlas`) with the full path on the
tooltip — this port's roots are `OS.get_user_data_dir()`-relative and six
segments long, and a `PopupMenu` sizes to its widest item. The static note split
into the canvas's two lines for the same reason (as one row it set the panel to
380 px). Measured: 467 → **362 px**, against the canvas's 298.

**MN-18 · The Assets menu had one rule too many and its most important row
last.** — **FIXED.** The canvas runs its four opening rows unbroken and places
`Asset pack ▸` **directly above** Icon families; the shell had a rule between the
slicer and Import image, and `Asset pack ▸` alone at the foot below
`Clear library…` — six positions out of place.

**MN-19 · The map right-click menu was the one `PopupMenu` nothing had ever
styled.** — **FIXED.** 232×54 on **both** desktop and tablet: stock panel, stock
selection bar, 15 px rows — a third of the 44 px floor on the device that has
one. `civilization_workspace.gd` built it with a bare `PopupMenu.new()`. Now
291×149 on the shell's panel.

**MN-20 · The tool options bar was drawing the *dock's* parameter row.** —
**FIXED.** Every artboard with a tool options bar sets the row in
`font:11px 'IBM Plex Mono';color:#8d9296` with inheriting labels —
`DCC shell 1920`'s `hardness`/`intensity`/`noise`, the tablet's same three at
13 px, and `Cartalith Paint Toolbar.dc.html`'s `size`/`strength`/`falloff`/
`spacing`/`max Δ`/`pressure`. The bar used `DccWidgets.slider()`, whose prose
`text_secondary` label is *right for the dock* (DS-06). Repainted at `_narrow()`, the single point
every bar row passes. Track 56 → 70 px (the canvas's figure); label column
56 → 74, because Plex is wider and the first capture clipped `Brush size` to
`Brush si`.

**MN-21 · The armed mode segment is filled, and the armed feature segment is
washed.** — **FIXED.** `Cartalith Paint Toolbar.dc.html` — a **later** artboard
of a component `DCC shell 1920` does not draw — fills exactly one thing:
`padding:5px 12px;border:1px solid #e0a34a;color:#141617;background:#e0a34a` on
the active `SCULPT`/`PAINT`/`MEASURE`. Reversed paper-coloured type on accent is
§11's rule for a filled accent surface, so this is the design's grammar, not an
exception to DS-02 (whose search covered a different artboard). Kept as its own
`set_mode_segment_on()` rather than a flag on `set_segment_on()`, so the fill
cannot spread back to DS-02's 141 call sites. The *feature* segments are
`border:1px solid #e0a34a;color:#e0a34a;background:rgba(224,163,74,.10)` —
border and ink were right, the wash was missing, so "which of these eight is
armed" read as a hairline colour change.

**MN-22 · The Layers popover's selected row.** — **FIXED.** DS-02's search found
"slider fills and **exactly one** other hit — a *selected layer row* in the
layers popover", removed 141 filled buttons, and never applied the one filled row
it had found. Canvas: `padding:4px 8px;background:#e0a34a;color:#1a1206;
font-weight:600`. The reversed ink is `c("bg")` so it follows a theme switch; the
weight is left alone, and said so, because no bold cut of the prose face is
loaded.

**MN-23 · The open-item wash was `.08`; the canvas's is `.10`.** — **FIXED.**
The canvas draws the menu *bar's* open title at `rgba(224,163,74,.08)` and the
dropdown *item* at `rgba(224,163,74,.10)`; DS-05 matched the item to the bar's.
`DccTheme.menu_highlight()` is the item's; `accent_wash` stays the bar's.

**MN-24 · Preferences drew four bands for five groups.** — **FIXED, and
designed rather than matched.** No artboard draws Preferences open. §2.5 names
five groups — Performance, Graphics, Tiles & LOD, Memory, Application — and the
shell had four unlabelled rules, **Tiles & LOD and Memory sharing one**, so
`Tiled LOD` and `Undo history` read as one subject. Band *names* from §2.5; band
*treatment* copied from the three drawn menus.

### Also fixed, not a design question

`font_separator_color` was missing from `_THEME_COLOR_OVERRIDES`, so a
dark/light switch had never repainted it.

### Registered, not fixed

- **A badge is not a right-hand column.** The canvas right-aligns `PNG · TIFF`,
  `pack .zip`, `8 warnings`, `checker ▸` in Plex. Godot's only right column on a
  `PopupMenu` item is the accelerator (a real keystroke or nothing), so every
  badge is appended to the label after a double space. Content matches;
  alignment does not. The same limit makes `Recent worlds`' secondary path text
  a tooltip.
- **`SCULPT · MOUNTAINS` names a feature absent from its own row.** The header
  reads `sculpt_get_feature()`, set from the WORLD dock's feature list; the row
  beside it shows `FreehandMode`'s eight sub-modes, none lit. Each is honest;
  together they read as a bug. Which control owns the armed feature is a
  tool-system question.
- **The tool options bar's height: the two canvases disagree.** `DCC shell 1920`
  draws one 34 px bar; `Cartalith Paint Toolbar.dc.html` draws **two 38 px
  bars**, each ruled. The shell draws one bar with two unruled rows. A third
  owner decision, smaller than the two ruled on today.
- **Tablet dock and bar *contents* are still desktop-sized** (DS-03:
  `phone_fit()` begins `if not _phone: return`). Menus are out of it now; docks
  and the tool options bar are not — accordion rows 30 px, sliders 14 px, dock
  labels 11 px, against §13's 44–52.
- **Dropdown check marks** are Godot's stock radio icons; the canvas uses `●` /
  `○` typographic marks.
- **The Layers popover's band ink** is `#6f7478` where its canvas band is
  `#5f6468`, and the shell prefixes `§ `, the canvas's convention for a dock
  section but not for this popover.
- **Sculpt's feature names** are `FreehandMode`'s real eight (Raise · Lower ·
  Smooth · Cliff · Ridge · Canyon · Mesa · Volcano); the canvas's are Raise ·
  Lower · Smooth · Flatten · Noise · Ridge · Carve · Mask. Flatten, Noise and
  Mask have no engine mode. An engine gap, not a UI one.

### Two canvas-vs-shell conflicts that are deliberate, recorded so they stay shut

- **`Data ▸ Conversion`** is drawn in the tablet artboard and deliberately
  absent from the shell — owner, 2026-08-20, on §7.4's research. The canvas is
  stale on this point.
- **`Vault` as an eighth top-bar menu** is drawn in Menu Structure v3 and is
  deliberately three rows inside Data — owner, 2026-08-24: *"the vault menu can
  be shoved into data."*

### The two open items §48 raised, now ruled on by the owner

**Region fills — KEEP THE FILLS**, and **the phone canvas — ADOPT 412 dp
FULLY**: both rulings are recorded verbatim at the point §48 raised them (its
*Registered, not fixed* and *Two canvases disagree about the phone*). This pass
logged no region-fill divergence under the first. Do not re-open it.

Under the second, `DccTheme.PHONE_REF_SHORT` is `393.0` and every shipped phone
constant matches that canvas coherently, so the migration is a deliberate
redesign and a separate pass, not started here. What this pass produced for it
is the divergence list, measured against the 412 canvas menu by menu:

| Constant / region | Today (393) | 412 canvas | Δ |
|---|---|---|---|
| `PHONE_REF_SHORT` | `393.0` | `412.0` | the unit every figure below is in |
| `H_PHONE_TOP_SAFE` | `44` | **`28`** | 412's status row is a status row, not a keep-clear reserve |
| `H_PHONE_TOP_SCRIM` | `96` | — | 412 draws no gradient scrim, it draws a solid ground |
| `W_PHONE_CUTOUT` | `108` | — | 412 reserves no centre lane |
| `H_PHONE_APP_BAR` | `52` | **`56`** | glyph boxes 40 dp, not 44 |
| `W_PHONE_RAIL` | `44` | **region deleted** | replaced by the bottom nav |
| bottom nav | 64 dp, labels only | 64 dp, **`14px` glyph over `9.5px` label** | five glyphs to draw, to §12's rules |
| `H_PHONE_GESTURE` | `26` | **`20`** | handle 112×4 |
| sheet surface | `panel` | `#151718`, radius `18px 18px 0 0` | a distinct raised tone |
| sheet primary button | square chip | 48 dp pill, `r24`, filled `#e0a34a`/`#141617`, `500 11px Plex/.16em` | secondary is the same pill outlined |
| sheet slider thumb | ~19 dp | 22 dp | track 3 px, row 32 dp |
| ☰ drawer | 300 dp side sheet | **does not exist** — L2 is a full-screen drill with `←` | |
| overflow title | `MENU` + `Cartalith`, **two `✕`** | `MORE` + `ELDRA · 1.6 GB`, one `✕` | |

Measured at 1440×3168 (`_phone_scale` 3.664): status row **43.9 dp** against
28, app bar **52.1** against 56, bottom nav **64.4** against 64, gesture inset
**25.9** against 20, drill rows **52.1** against 52. The 1080×2400 pass
(`_phone_scale` 2.748) is the same composition scaled coherently — **no
arithmetic error at either size**, which is what makes this a redesign rather
than a repair.

**One conflict the migration pass must resolve rather than copy.** The 412
canvas's five bottom-nav tabs are `WORLD · GENERATE · SIMULATE · MAP · MORE` —
the **pre-v3** domain set. `Cartalith Menu Structure v3.dc.html` is newer and the
authority for menu content and naming, with three domains:
`WORLD · CIVIL · CARTO`. The shipped bar is v3's three plus the two phone affordances (`PANELS`,
`MENU`). Probably right; not what the canvas draws, and it needs saying out loud
rather than being silently resolved by whoever gets there first.

### Harness

`_menuconf_probe.gd` / `.tscn` (new, untracked). Hosts the shell in a
`SubViewport` for §47/§48's reason (`--resolution WxH` is clamped to the dev
monitor's 1680×1002 work area and boots *tablet* mode), with
`gui_embed_subwindows = true` — which is what brings a `PopupMenu` (a `Window`,
outside any `Control` walk) into the captured texture at all. It opens every
program menu, recurses into every submenu by `get_item_submenu()`, and dumps per
row the text and disabled/checked/radio/accelerator state, and per popup the
measured size, resolved panel colour, item font size, `v_separation`, separator
font/size/colour and hover colour. `--vp WxH --tag NAME [--force-touch]`.

Two traps: `Window.popup(rect)` clamps to the same usable rect, so a submenu
opened at 3168 px tall comes back 1002 — assign `size` *after* `popup()`. And
`about_to_popup` must be emitted by hand before reading a popup that rebuilds on
open (Recent worlds, GPU devices, Open windows, the Preferences busy-lock), or the
dump reads the previous session's rows.

---

## 52 · MEM-01…MEM-04 — the memory rise was measured, and the hi-DPI pass costs 1.4 MB of it (2026-08-25) — **CAUSE FOUND, ONE ROW FIXED, BASELINE RETIRED**

§50 recorded **1 033 MB peak / 818 MB steady** against 2026-08-20's 878 / 647
(+18 % / +26 %) undiagnosed; the owner asked for a diagnosis before deciding.
The standing hypothesis was §47: both its fixes multiply texture *area* by the
square of `_phone_scale` — 2.748 here (≈7.5×), 3.664 on the OnePlus 12 (≈13.4×).

Hardware: **OnePlus 6T** (`ONEPLUS_A6013`, Android 15), 1080 × 2340,
`_phone_scale` **2.748**, Adreno 630. Metric: `adb shell dumpsys meminfo`
`TOTAL PSS` (grep `TOTAL PSS:` specifically, per §50) plus the per-category
rows, which no previous pass recorded and which made this answerable.

### MEM-01 — the hi-DPI pass costs 1.4 MB, bisected · **PROVEN NEGATIVE**

Measured on **one** build with a runtime switch read from a file on the handset,
so all four variants are the same binary: bit 1 forces `DccIcons.get_icon`'s
`magnify` to 1 (HD-02 off); bit 2 makes `DccWidgets.oversample()` a no-op (HD-01
off). Four cold boots, measured at the welcome screen (the modal §47 measured on)
and after *Continue without a world* with the whole shell built:

| variant | `Gfx dev` welcome | `Gfx dev` shell | Godot textures | glyph raster cache |
|---|---:|---:|---:|---|
| **both on (shipping)** | **9 696 KB** | **9 416 KB** | 26.98 MiB | 38 entries, **245.4 KiB** |
| HD-01 on, HD-02 off | 9 272 KB | 9 028 KB | 26.22 MiB | 34 entries, 167.1 KiB |
| HD-01 off, HD-02 on | 8 544 KB | 8 264 KB | 25.11 MiB | 38 entries, 245.4 KiB |
| both off | 8 268 KB | 8 100 KB | 25.01 MiB | 34 entries, 167.1 KiB |

- **HD-01, font oversampling: 1 152 KB** of `Gfx dev`, identical at both
  points; 1.87 MiB by Godot's texture monitor.
- **HD-02, icon re-rasterisation: 424 KB** of `Gfx dev`, of which **78.3 KiB**
  is the glyph raster cache (245.4 − 167.1) across four extra entries — the same
  glyph at two rasters, as `_cache`'s key promises.
- **Both together: 1 428 KB** — **0.8 %** of a reported rise of 155 MB peak and
  171 MB steady.

**The switch was proved live**: the same welcome-screen crop (search-field
placeholder row) measures max adjacent-pixel |ΔLum| **0.3020 with the fixes on
and 0.1686 off** — §47's discriminator, moving the right way by roughly the
right factor. A silently inert switch would have measured identical, the failure
mode §47 collected three of.

**There is no exchange rate to offer the owner.** Capping oversampling at 2×
would recover a fraction of 1.1 MB; freeing glyph atlases on modal close a
fraction of 389 KiB. Both noise.

### MEM-02 — the memory is in canvas vertex buffers, and they are drawn per dash · **CAUSE**

`performance_window.gd` gained Godot's render monitors (MEM-04), which turned a
partition of `dumpsys` categories into an answer:

| | no world | world up | after 12 zoom-in notches |
|---|---:|---:|---:|
| `TOTAL PSS` | 422 MB | 869–1 029 MB | **1 279 MB** |
| Native Heap | 247 MB | 503–553 MB | — |
| `Gfx dev` | 9.4 MB | 195–338 MB | **556 MB** |
| Godot **textures** | 26.98 MiB | 87.89 MiB | 88.02 MiB |
| Godot **buffers** | 14.33 MiB | **290.8 MiB** | **500.9 MiB** |
| objects in frame | 799 | **311 237** | **560 569** |
| draw calls | 90 | 855 | 932 |
| glyph raster cache | 245.4 KiB | 389.4 KiB | 389.4 KiB |

**Buffers track the object count; textures do not.** Zooming in multiplies drawn
primitives 1.8× and buffers 1.7×, while textures move 0.13 MiB and the glyph
cache not at all. The GPU cost is **canvas geometry**, not pictures; the glyph
cache is **0.08 %** of the buffers it was suspected of dominating.

What draws 311 237 objects: `map_overlay.gd`'s `_draw_dashed_polyline` emits
**one antialiased `draw_line` per dash**, walking a way's length at
`dash + gap`. Before `a13881d` (**2026-08-24**, §36 RD-02) a land way was one
flat `draw_polyline(_stroke_points(...), ROAD_COLOR, width, true)`. That commit
gave every land way the reference's two-stroke treatment — dark underlayer plus
type colour, **dashed for three of the five tiers** — and in the same pass fixed
the way-type filter that "could not see two thirds of the network": the drawn
network grew about threefold and each minor way went from one primitive to one
per dash. `f85c606` (**2026-08-24**) is the second half: town layouts **on by
default**, one `draw_colored_polygon` per lot, revealed inside a km band —
precisely when zoomed in, where 560 569 objects and 500.9 MiB were measured.

Both landed **four days after the 2026-08-20 baseline**, and neither is a defect
— they are the reference's way styling and the owner's own "I don't see the
settlement rendered on the map itself." They were simply not budgeted.

**§50's "544 MB in `Gfx dev`" belongs to its dirty sample, not its clean run**;
the brief that followed read the two as one measurement. Reproduced here: 12
zoom-in notches from a fresh generate reach **556 MB of `Gfx dev` and 1 279 MB
PSS in under a minute**. A zoom cost, available on demand.

### MEM-03 — 878 / 647 was never a comparable baseline · **RETIRED**

**No pass in the chain fixed the seed**, and the New World dialog rerolls it on
every open. Six clean runs today, same APK and procedure (cold boot, one
2048 × 1311 generate from the welcome screen), same metric:

| run | steady `TOTAL PSS` |
|---|---:|
| 1 | 869 MB |
| 2 | 902 MB |
| 3 | 916 MB |
| 4 | 937 MB |
| 5 | 963 MB |
| 6 | 1 029 MB |

**160 MB of spread on one build — the size of the entire reported regression.**
Six *different* seeds in one process (`New seed` ▸ `Generate world`): 916,
1 069, 1 069, 1 073, 1 073, 1 072 MB. More settlements, roads and revealed
layouts genuinely cost more, because MEM-02 is what the memory *is*.

Also: the 2026-08-20 pass sampled **every ~2 s** and §50 continuously (matters
for a transient peak, not a steady level); and §50's 818 MB is below all six
runs, which is what a single sample of a lumpy settling curve looks like — the
trace here still moved between 656 and 963 MB from t+34 s to t+53 s after the
tap before going flat.

**A real level increase since 2026-08-20 is likely — 647 MB is well under the
869 MB floor here — but "+18 % / +26 %" is not a number this measurement can
support; the next pass should fix the seed before quoting one.** State the
fixture, or the result is not a result.

### Leak or level · **LEVEL, four ways**

- **Three same-seed regenerations**: 927.2 / 927.3 / 928.4 MB, and 928.4 MB
  twenty seconds later.
- **Six different-seed generations in one process**: steps once to a plateau at
  gen 2 and holds — 1 069 / 1 069 / 1 073 / 1 073 / 1 072 MB.
- **The clean run's own trace**: 963 MB across roughly 480 consecutive samples
  over 95 s, flat to the kilobyte.
- **Deep zoom, seven consecutive samples 8 s apart**: 1 310 079 → 1 309 808 KB,
  a spread of **271 KB (0.02 %)**, drifting *down*.

Nothing grows without bound. Not a bug; a budget.

### MEM-04 — the Performance window's Memory row, which §50 registered as under-reporting 4× · **FIXED**

`OS.get_static_memory_usage()` is honestly named and excludes both the Rust
allocations and everything in `Gfx dev` (§50: 0.2 GB against 818 MB PSS). It now
sits beside two rows the renderer does know:

- **Video memory: 378.7 MiB — textures 87.89 MiB, buffers 290.8 MiB.**
- **Glyph raster cache: 65 entries, 389.4 KiB. Last frame: 855 draw calls over
  311 237 objects.**

Beside the working-set figure, not instead of it, and labelled as outside it:
no single number on this platform is the one that gets the app killed.
`DccIcons.cache_stats()` is the new binding behind the second row — the one
measurement that could have made HD-02 look expensive, now permanently on
screen.

### Registered, not fixed — the two levers that would actually move the number

Stated as options with their costs and deliberately not taken on a diagnosis
pass. §54 later pulled and measured both (MEM-05, a proven no-op; MEM-06, fixed).

- **The dash loop is one `draw_line` per dash.** `urban_layout_draw.gd` had
  already collapsed the same shape (*"a 6-town sheet took 577 ms a redraw"*, per
  roof `draw_polyline` → one `draw_multiline`); the same collapse on
  `_draw_dashed_polyline` would turn thousands of primitives per way into one
  without touching colour, width or dash period. It cannot use
  `draw_dashed_line()` per vertex pair — that function's comment records why: a
  smoothed route's points are shorter than one dash cycle, so the phase restarts
  and every segment renders solid.
- **Nothing bounds the overlay by zoom.** Town layouts reveal inside a km band,
  but dashed minor ways never drop out, so the object count is unbounded in the
  direction the owner most likes to travel. A screen-length floor per way tier
  would cap it.

### Also worth knowing

- **PH-15 reproduced independently.** A 700 ms drag from (540, 1800) on the
  MENU ▸ Preferences L3 sheet opened *Theme* as an L4 sheet instead of
  scrolling — at a duration §50 recorded as working. A 1 500 ms drag from
  (540, 2250) scrolls cleanly. The threshold lies between; a real touch defect,
  not flick-only.
- **DS-02's "no filled amber slab anywhere in the sweep" has an exception.** The
  Layers ▸ *Data overlays* sheet draws the selected row (`No overlay (base map)`)
  as a **filled** amber slab with dark text. That sheet was outside §50's sweep:
  a gap in the sweep, not a contradiction.
- **Generation is 24.0 s** on this build (the app's `Pass` row), consistent with
  §50's 25.1 / 24.8 / 25.8 s and with §4.1's "16-18 s" never having been
  comparable.
- **`logcat` clean** across the verification boot: zero `SCRIPT ERROR`,
  `USER ERROR`, `USER WARNING`, Rust panic or godot-rust complaint.
- **`/sys/class/kgsl/kgsl-3d0/proc/<pid>/mem` does not exist on this device**, so
  there is no per-allocation GPU dump; `Gfx dev` is the finest driver-side
  granularity and Godot's monitors supply the split.

### Harness

The runtime switch and its flag file are **reverted** — an experiment, not a
proposal; §47 stays as shipped. Kept: `DccIcons.cache_stats()` and the two
Performance-window rows (MEM-04's fix). `project.godot` md5
`ccba27c9280cf8373412e2ba87ed4054` before and after every Godot invocation; the
`;` comment block survived each.

---

## 53 · PH-18…PH-27 — the 412 dp phone migration: the canvas the owner ruled for, built (2026-08-25) — **TEN LANDED, ONE BUG FOUND ON THE DEVICE, FOUR REGISTERED**

§51 measured the whole delta between the shipped 393 dp phone and
`design/Cartalith Android Phone.dc.html` and deliberately did not start the
migration. This is that pass. **Nothing was broken going in** — at 393
(`DccTheme.PHONE_REF_SHORT` = `393.0`) every phone constant matched the 393
canvas at both 1440×3168 and 1080×2400 — so this is a redesign, a regression
would be a real cost, and every figure is given before *and* after at both sizes.

### The arithmetic, first, because it does not cancel

`_phone_scale` is `short_side / PHONE_REF_SHORT`, so moving the reference 393 →
412 **drops** it — 3.664 → **3.495** at 1440, 2.748 → **2.621** at 1080 — and
every `_pscale()` result shrinks 4.6 %. The authored dp move independently and
both ways: app bar *up* (52 → 56), status row and gesture inset *down* hard
(44 → 28, 26 → 20). They do not cancel, so the constants are **re-authored
against the 412 canvas's own literal inline styles**, not converted — §51
records converting-from-prose biting twice.

### Measured, before and after, at both sizes

Harness `_ph412_probe.gd`/`.tscn` (untracked, `SubViewport`-hosted for §47's
reason). Physical pixels, with dp against the reference live at the time.

| Region | 393 canvas @1440 | 412 canvas @1440 | 393 @1080 | 412 @1080 | Canvas asks |
|---|---|---|---|---|---|
| Status row | 161 px · 43.9 dp | **98 px · 28.0 dp** | 121 px · 44.0 dp | **73 px · 27.9 dp** | 28 |
| App bar | 191 px · 52.1 dp | **196 px · 56.1 dp** | 143 px · 52.0 dp | **147 px · 56.1 dp** | 56 |
| Bottom nav | 236 px · 64.4 dp | **225 px · 64.4 dp** | 177 px · 64.4 dp | **169 px · 64.5 dp** | 64 |
| Gesture inset | 95 px · 25.9 dp | **70 px · 20.0 dp** | 71 px · 25.8 dp | **52 px · 19.8 dp** | 20 |
| Sheet action | square chip, 26 px base | **168 px · 48.1 dp pill, r84** | square chip | **126 px · 48.1 dp pill, r63** | 48 dp, `border-radius:24px` |
| Slider thumb | none at all | **77 px · 22.0 dp** | none | **58 px · 22.1 dp** | 22 |
| Top scrim | 352 px gradient | **deleted** | 264 px | **deleted** | solid ground |
| ☰ drawer | 300 dp side sheet | **deleted** | same | **deleted** | no drawer |

Net: the map gains ~88 px of height at 1440 and ~66 px at 1080.

### PH-18 · The constants · **LANDED**

`PHONE_REF_SHORT` 393 → **412**. `H_PHONE_TOP_SAFE` 44 → **28**;
`H_PHONE_APP_BAR` 52 → **56**; `H_PHONE_GESTURE` 26 → **20**.
`H_PHONE_TOP_SCRIM` (96) and `W_PHONE_RAIL` (44, already unused) **deleted**.
New, all read off the canvas: `H_PHONE_BOTTOM_NAV` 64, `W_PHONE_GESTURE_HANDLE`
112, `H_PHONE_ROW` 52, `H_PHONE_PILL` 48, `PHONE_ICON_BOX` 40,
`PHONE_SLIDER_ROW`/`_TRACK`/`_THUMB` 32/3/22. `PHONE_TAP_MIN` stays 44 — the
canvas's own TARGETS card restates it, in 412 units. `W_PHONE_CUTOUT` (108) is
**kept for landscape only**, disclosed: all eight 412 screens are portrait, so
landscape has no canvas and keeps §13's reserve under `DCC_SHELL_SCOPE.md`'s
rule 2.

### PH-19 · The bottom nav is glyph-over-caption, and it carries v3's domains · **LANDED**

The one place the two authorities split, resolved rather than copied. The
canvas's tabs are `WORLD · GENERATE · SIMULATE · MAP · MORE` — the **pre-v3**
domain set; `Cartalith Menu Structure v3.dc.html` is newer, owns domain content
and has three. Rule 1 gives **412's geometry and v3's content**:
`WORLD · CIVIL · CARTO · PANELS · MORE`, in the canvas's treatment — a `14px`
glyph over a `9.5px/.1em` caption, `gap:4px`, active `#e0a34a` over a resting
`#8d9296`. `MENU` is `MORE` now, the canvas's word.

The row was **captions only** before. Three glyphs already existed as drawn SVG
(`DccIcons`' `domain_world` / `domain_civ` / `domain_carto`, authored to §12's
rules); `nav_panels` and `nav_more` are new and **designed rather than
matched**, tracing the canvas's own symbols (▤, ⋯) at §12's 1.2 px stroke.
**This does not re-open the owner's *"those icons don't exist."*** — that ruling
was about the **desktop vertical rail**, whose artboard draws no icon element;
this artboard draws a glyph over every caption, five times.

The active tab has **no fill**: the canvas's lit cell is `color:#e0a34a` only,
where the desktop rail's is `background:rgba(224,163,74,.08)`. The cell
registers `"box": false` in `_domain_marks` so `_select_domain()` skips the slab
for the bar and keeps it for the rail. `PANELS` and `MORE` are not domains, so
never went through `_select_domain()` — the bar said WORLD while MORE sat on top
of it. `_refresh_phone_bar_lit()` lights them off live state instead.

### PH-20 · The ☰ drawer is deleted; ☰ opens the `02 Domain` drill · **LANDED**

The canvas has no drawer at any level: its `02 Domain` screen is a full-screen
drill with a `←`, which the shell's full-height left dock sheet already is, so
`☰` opens that. The 300 dp side sheet listed the same three destinations the
bottom bar's first three cells now carry — two differently-shaped lists of one
destination, which the canvas's "More is a grouped list, not a duplicate menu
bar" note rules out. `_build_phone_drawer()`, `_pick_drawer_domain()`,
`_set_drawer_open()` and `_phone_drawer` are gone; the back chain and
`_close_all_phone_overlays()` lost their branches for it.

### PH-21 · `MORE` has one close button fewer than it had two of · **LANDED**

§51 row 77: the overflow root was titled `MENU` + `Cartalith` with a `✕` on
**each side of its own title**. The canvas's `07 More` header is a `500 12px
Plex/.22em` title and a `10px Plex #6f7478` readout (`ELDRA · 1.6 GB`) — no back,
no close. Now: at L2 the header is title + `<world> · <memory>`, read off the
live `top_world` / `top_mem` slots (the shell's `–` placeholder treated as
"nothing yet"); at L3 and deeper it is `←` and one `✕`. The canvas puts a `⋮`
there at L3, a per-screen overflow with nothing behind it here, so `✕` goes
there instead (`←` leaves a level, `✕` leaves the menu). The **MORE tab is a
toggle**, because that screen no longer has a close: tapping the lit tab again
leaves it.

### PH-22 · The menu covered the bottom bar, so two taps in five did nothing · **FIXED — found on the handset, not in a probe**

The pass's one genuine defect, and why a desktop probe is necessary and not
sufficient: with `MORE` open on the OnePlus 6T, tapping MORE or WORLD did
nothing. `PhoneMenu` is a full-rect `Control` whose children are inset by
`apply_insets()`, and `open()` set the **node's own** `mouse_filter` to `STOP`,
so the whole screen picked — including the strip where the bottom nav lives.

Two halves: the node is `MOUSE_FILTER_IGNORE` always (blocking is `_screen`'s
and `_sheet_scrim`'s job, which cover exactly the menu's rect; `_screen` now
says `STOP` explicitly rather than leaning on `Container`'s `PASS` default); and
`_apply_phone_orientation()` includes the bar's height in the menu's bottom
inset, so the bar is **visible** under the menu as well as reachable — the
canvas's model has the bar as L1 and `07 More` as a tab destination, not a
modal.

Rule, because it generalises: **an overlay that insets its children must not
`STOP` on its own full-rect root.** Same shape as MN-13's `flat = true` — a
property on the wrong node, invisible until driven.

### PH-23 · The status row is a status row, on a solid ground · **LANDED**

`height:28px;padding:0 16px;font:10px 'IBM Plex Mono';color:#8d9296`, both spans
at that ink (the right one was `text_faint`, and both were 11 px). §13's 44 dp
keep-clear reserve, its 96 dp gradient scrim and its 108 dp centre lane are gone:
the canvas paints `background:#101112` under the whole top and the app bar
carries only a `border-bottom`. Deleting the scrim also removed
`rebuild_theme()`'s third pass: its colour lived inside a `GradientTexture2D`, on
no node and in no theme resource, so neither recolour walk saw it (a light-
palette capture once found a charcoal band over a light screen). The status row
is a plain `panel` stylebox now and `_recolor_subtree()` reaches it.

### PH-24 · Sheet buttons are 48 dp pills; sliders have a thumb at all · **LANDED**

The canvas's action button is `height:48px;border-radius:24px`, primary filled
`#e0a34a` with `#141617` type, secondary the same box outlined at
`rgba(255,255,255,.16)`, both `500 11px Plex/.16em` upper case; the shell drew
square outlined chips. `DccTheme.pill()` is **the only rounded surface in this
design system**, deliberately: §11's "radius 0 everywhere" is a desktop-artboard
rule and the phone canvas overrides it on all eight screens. The reversed ink is
`c("bg")`, not the literal, so a theme switch repaints it (as
`set_mode_segment_on()` does for the desktop's one filled accent surface).
Applied from `DccShell.phone_fit()` only, off a new `DccWidgets.ACTION_META`
marker, so desktop and tablet never see a rounded button and the 141 call sites
§48's DS-02 cleared of accent fills **stay cleared**.

The slider: `_style_slider()` removes the grabber outright — §11's "a 2 px rule,
the travelled part in accent, and **no grabber**", right for a pointer. The
phone canvas draws a `22×22` round accent thumb on every slider, because a
finger has no cursor to find the handle with. `DccWidgets.phone_slider()` adds
one as an antialiased disc rather than Godot's stock grabber bitmap (which
cannot be recoloured for the light palette — the same reason `phone_menu.gd`
builds its switch from two rounded styleboxes). Measured 77 px at 1440 and 58 px
at 1080, both 22.0 dp. The **row** stays 44 dp rather than the canvas's 32,
because `phone_fit()`'s `PHONE_TAP_MIN` floor is higher and the canvas's own
TARGETS card set it.

### PH-25 · The L2 drill rows carry a control count · **LANDED**

Canvas: `<span style="font:10px Plex;color:#6f7478">9</span>` at the end of
every category row — "the count is the number of controls inside, so depth is
legible before the tap"; §51 row 44 recorded its absence.
`DccWidgets.category()` grows a trailing count `Label` **on the phone only**,
which needed a `DccTheme.is_phone()` beside `is_touch()` (the factories are
static and cannot reach the shell, and a tablet must not get this). The count is
filled by a `call_deferred` from `category()` itself — the body is empty then and
the caller fills it synchronously, so the frame's idle pass is the first correct
moment, and it works on a node not yet in the tree (a workspace builds its panel
before `register_workspace()` attaches it). It counts **controls, not nodes**: a
`SpinBox` or `OptionButton` is one control of several `BaseButton`s and a
`LineEdit`, so the walk stops there. WORLD ▸ Generate reads `13`.

### PH-26 · The phone menu's band captions were drawing at 9 *physical* pixels · **FIXED**

Found by reading a capture, not the code. `phone_menu.gd::_band()` called
`DccTheme.header()`, a desktop helper whose `FS_HEADER` is a raw 9; the main
viewport has no content scale, so `STATUS` / `PROJECT` / `CONTENT` / `SYSTEM`
were grey smudges about half a millimetre tall on a 510 ppi panel. Now the
canvas's `9.5px Plex/.2em #6f7478` through `_ps()`. Two more corrections in the
same file, read off the canvas: a drill row's second line is `9.5px #5f6468`
(`text_ghost`) where the shell used `text_faint` at 9, and the head's breadcrumb
is `10px` untracked where it was `9px` at spacing 1.

### PH-27 · A fourth dead stylebox, on the phone's most-tapped control · **FIXED**

`_phone_bar_button()` — the app bar's ☰ — was `flat = true` with a `hover`
override under it: MN-13's trap, fourth site. `flat = false` with an empty
`normal`, plus the `pressed` box the other three sites got. The bottom-nav cells
had the same fault and are fixed in the same stroke.

### Two §51 rows that are not phone work, closed here

**Row 61 · the tool options bar** — the owner call §51 left open. `DCC shell
1920` draws one `height:34px` row; `Cartalith Paint Toolbar.dc.html` draws
**two**, each `height:38px;padding:0 14px;border-bottom:1px solid
rgba(255,255,255,.10)`. Rule 1: the Paint Toolbar is the later artboard and
draws a component the other does not. `tool_bar.gd` already built two rows, at
22 and 24-28 px with a 4 px gap and no rule; after: **78 px, two 38 px rows,
ruled**, all three modes. The bar keeps its 34 px minimum for the *idle* row
`app.gd` fills when no tool is armed (what `DCC shell 1920` draws). Disclosed
difference: the rule is inset by the bar's 14 px content margin rather than
full-bleed, because `tool_options_row` — the node `set_tool_options()` refills —
sits inside that margin, and full-bleed would restructure a component four
workspaces and `app.gd` build into.

**Row 70 · dropdown check marks** — the canvas marks a chosen row `●` and an
unchosen one `○`; the shell left Godot's stock radio and check icons, the last
stock artwork in the palette. Godot draws that column from four **theme icons**,
so the typographic mark arrives as a texture: a filled disc and a hairline ring
at the item's type size in palette ink. The four `_disabled` variants are set
too — otherwise they fall back to stock artwork, the trap `style_popup()` exists
to close.

### Designed rather than matched — named, per the standing rule

1. **`nav_panels` and `nav_more`** — the canvas's tab set predates v3 and has no
   glyph for either; each traces its own symbol at §12's stroke.
2. **The L3 header's right-hand slot is a `✕`, not the canvas's `⋮`** — no
   per-screen overflow exists to put behind a `⋮`.
3. **The bottom bar stays visible under the phone menu.** `07 More` draws no
   bar, but the canvas's model says the bar is L1 and that screen a tab
   destination, and with both `✕`s gone it is the only escape that is not
   system back.
4. **`W_PHONE_CUTOUT` survives in landscape** — all eight 412 screens are
   portrait.
5. **The `STATUS` band on the More screen** is an addition (the canvas's `07
   More` bands are PROJECT / ASSETS / VIEW). Kept: it carries the live readouts
   §15 fault 2 rescued from the reparented desktop status bar, and the canvas
   carries `Performance · CPU 38 · GPU 71` as a row — the same information one
   level down.

### Registered, not fixed

- **`⌕` and `⋮` are not built into the app bar.** `⌕` has no destination
  (`menus.gd`'s Edit ▸ Find on map… is a `_todo()` row, "no search index yet"),
  and a magnifier opening a disabled item is worse than none; `⋮` has nothing
  behind it. Both are three-line additions once they have a destination.
- **The bottom nav is `panel` (`#121314`), not the canvas's `#131516`**, and
  the sheets are `raised` (`#17191a`), not `#151718` — 2/255 per channel each.
  Two near-identical literals is what §48 spent a pass removing eleven of.
- **The tool options sheet is still resident, not summoned** (§51 rows 63/83).
- **A phone pill upper-cases its label once**, at `phone_fit()` time; a caller
  that later reassigns `.text` gets sentence case in a pill. No shipped caller
  does; recorded because the failure would be silent.
- **Every pill is 48 dp, including the one the canvas draws at 46** — the
  canvas disagrees with itself by 2 dp:
  `02 Domain`'s `GENERATE WORLD` is `height:46px;border-radius:23px` while
  `04 L4 sheet`'s and `08 Inspector`'s primaries are `48px`/`24px`; its PHONE
  RULES card settles it — "48 dp buttons".
- **The desktop vertical domain rail was already gone**, although §51 rows 39
  and 75 listed "44 dp column" as a live divergence: `W_PHONE_RAIL` had no
  reader and the probe finds no such region at either size. Those rows were
  stale when written — the argument for re-measuring a measured list before
  acting on it.

### Verification

- `_ph412_probe.gd` / `.tscn`, `SubViewport`-hosted, at **1440×3168 and
  1080×2400**, windowed; every region in physical px *and* dp; screenshots of
  the viewport, MORE and the left dock sheet at both sizes.
- `_toolbar412_shot.gd` / `.tscn` at 1600×900 for the two-row bar, all three
  modes. `_menuconf_probe.gd` re-run at 1600×900 and 2560×1600
  (`--force-touch`): no script error or warning; `●`/`○` confirmed by capture.
- **Driven on the owner's OnePlus 6T** (`9608b26b`, 1080×2340, `_phone_scale`
  2.621 after): boot, welcome dismissed, MORE opened and toggled shut, WORLD
  selected, ☰ into the left dock drill, back. PH-22 was found this way only —
  the desktop probe measured the layout correctly and never touched the bar.
  `adb logcat` clean.
- Build: `--export-debug "Android"` against the existing
  `target/aarch64-linux-android/android-dev/libcartalith_godot.so` (no Rust
  changed). `project.godot` md5 `ccba27c9280cf8373412e2ba87ed4054` before and
  after every Godot invocation, `;` comment block intact; `export_presets.cfg`
  and `Cargo.toml` untouched.

## 54 · MEM-02's two levers, both measured — the batching buys nothing and the culling is 370× (2026-08-25) — **ONE FIXED, ONE RETIRED**

§52's brief was diagnosis, so it registered two levers rather than pulling
them. This pass pulled and measured both: one is a no-op, the other the largest
single number this shell has moved. Hardware and metric are §52's: **OnePlus
6T**, Android 15, 1080 × 2340, Adreno 630, `dumpsys meminfo` `TOTAL PSS` and its
per-category rows, plus the render monitors MEM-04 put in
`Preferences ▸ Memory ▸ Working set…`. **Seed 123456, typed into the dialog on every run** (MEM-03's
ruling: a memory figure without its fixture is not a figure); both builds report
`generated · 26.3 s` at 2048 × 1311.

### MEM-05 — `draw_multiline` instead of one `draw_line` per dash · **RETIRED, PROVEN NO-OP**

§52's more attractive lever: `urban_layout_draw.gd` had made this exact collapse
for roof ink and its comment records the payoff (*"a 6-town sheet took 577 ms a
redraw"*). Built, shipped to the handset, measured against a baseline APK
differing in **one file**:

| seed 123456 | per-dash `draw_line` | one `draw_multiline` |
|---|---:|---:|
| objects, world up | 87 198 | 87 177 |
| buffers, world up | 93.04 MiB | 93.16 MiB |
| objects, +12 zoom notches | 857 965 | 857 944 |
| buffers, +12 zoom notches | 751.0 MiB | 751.2 MiB |
| `Gfx dev`, +12 zoom notches | 825.2 MB | 832.2 MB |

Nothing moved. On the desktop, both forms in one process: **336 186 objects and
308 draw calls, both arms, to the digit**; redraw wall time over 200 redraws,
three alternating pairs, 20.69/20.82, 19.09/19.03, 18.39/18.54 ms — noise.

**Why**, since the reasoning that made the lever look good was the wrong part:
308 draw calls for 336 186 objects means **Godot's canvas renderer already
batches** adjacent same-material primitives, so this was a command-count
optimisation against a renderer that had done it; and an antialiased thick line
expands to the same triangles whichever call submits it, so the vertex buffers —
which is what the memory *is* — never depended on call count.
`urban_layout_draw.gd`'s 577 ms was per-`draw_polyline`-**call** GDScript
crossing at a few thousand calls; a dash walk already runs inside one function.
The change was safe — `_dashbatch_probe`, every dashed pattern at
`set_way_scale()` 0.2 / 1.0 / 2.5 and camera zoom 1 and 4: **108 of 108 frames
identical** — but worth nothing, so it was reverted, and the probe is kept as the
reason not to try again.

### MEM-06 — nothing bounded the overlay by zoom, and nothing had to disappear to fix it · **FIXED**

§52 framed this as a design question — *what should drop out at what zoom* — and
it was not one. **The camera is an ancestor transform** (`_crisp_begin()`'s doc
comment), so `map_overlay.gd` could not know how far off screen a way was: every
way was projected, dashed and uploaded every redraw at every zoom, and the
viewport discarded the off-screen ones afterwards. `_run_offscreen()` asks the
question: `get_global_transform_with_canvas().affine_inverse() *
get_viewport_rect()` is the visible slice of this control's local space, and a
run whose bounding box misses it is skipped — ways, sea lanes and committed
routes.

| seed 123456 | before | after | |
|---|---:|---:|---:|
| objects, world up | 87 198 | **43 788** | −50 % |
| buffers, world up | 93.04 MiB | **45.43 MiB** | −51 % |
| draw calls, world up | 787 | 586 | |
| `Gfx dev`, world up | 149.3 MB | **104.9 MB** | −30 % |
| `TOTAL PSS`, world up | 845.3 MB | **807.7 MB** | −4.4 % |
| objects, +12 zoom notches | 857 965 | **2 320** | **−99.7 %** |
| buffers, +12 zoom notches | 751.0 MiB | **42.79 MiB** | −94 % |
| `Gfx dev`, +12 zoom notches | 825.2 MB | **103.2 MB** | **−87.5 %** |
| `TOTAL PSS`, +12 zoom notches | 1 593.8 MB | **806.8 MB** | −49 % |

**The zoom column is the finding**: twelve taps used to cost 676 MB of `Gfx
dev`; now 104.9 MB at the default view and 103.2 MB twelve notches in. §52's
*"nothing bounds it in zoom"* no longer holds, and nothing disappears. Read the
2 320 honestly: at twelve notches this seed's camera is over open sea, and the
old code drew the whole network anyway. Like-for-like over land is the default
view's 87 198 → 43 788, and the desktop harness with a network covering the
canvas: **2 592 846 → 807 765 objects at zoom 8** (−69 %), 1 303 380 → 445 632 at
zoom 4, 658 386 → 337 716 at zoom 2.

**It moves no pixel, checked**: `_cull_probe` renders the shipping script beside
a subclass whose `_visible_local_rect()` returns everything, through a real
`Node2D` camera ancestor at **four zooms × four pans, 16 of 16 frames
byte-identical**. **Not fixed**: one long way crossing the window still has a
bounding box covering it, so its whole run is walked and dashed — why object
count still rises with zoom on the desktop. Per-segment culling would fix it;
registered, not taken.

### Harness

- Two APKs from **one frozen snapshot of `HEAD`**, differing in `map_overlay.gd`
  alone. The working tree could not be used: a concurrent session was mid-edit
  on `shell/dcc_shell.gd` and the first snapshot would not parse
  (`Could not resolve class "DccShell"` on the handset). Snapshot committed `HEAD`, layer the
  file under test on top.
- The chrome looked **not stable across boots** — one run of "the same APK" came
  up with an icon tab bar and docked L3 panels, another with text labels and
  full-screen sheets, worth 799 vs 944 objects and 9.5 vs 15.1 MB of `Gfx dev`
  at the welcome screen; A and C above both came up at 9.5 / 9.7 MB. **Answered
  in §56**: not a boot race but two builds (`c9bb82e` and §53's migration)
  installed over each other; A and C were the same build and the numbers stand.
- `project.godot` md5 `ccba27c9280cf8373412e2ba87ed4054` unchanged across every
  Godot invocation, `;` block intact; `export_presets.cfg` and `Cargo.toml`
  untouched; no Rust changed.


## 55 · FX-01…FX-03 — the flow overlay never saw the camera, so zooming in emptied it (2026-08-25) — **THREE FIXED**

The owner, on the shipped Wind and Ocean-currents views:

> "from the ocean and windcurrent visualisation it doesn't scale with zoom so.
> It doesn't show finer patterns. And also can we make the tip be an arrow head
> instead of a square pixel."

Three faults, one cause: `shell/wind_fx_layer.gd` is a grandchild of
`ViewportHost._camera`, whose `position`/`scale` are pan/zoom, so every
coordinate the layer computes is magnified by a transform it never read, and
nothing compensated in any of the three ways it had to.

### The obvious suspect was innocent

Nearest-neighbour sampling usually produces this complaint (stair-stepped
streaks). Checked first: `_sample()` and `_wet_at()` have been **bilinear**
since the layer landed, and `flowfx_channel_round_trips_the_flow_vectors` keeps
the decode honest against `sample_bridge.rs`'s encode. What was missing was
particles to sample the field *with*.

### FX-01 — the particle set is seeded per grid, not per screen · **FIXED**

`_spawn` picked `rand * _fw`, `rand * _fh` over the **whole grid**, with a
constant count (260 wind / 200 ocean, the reference's own), so zooming in
magnified a fixed scattering. Particles whose head lands in the map area, on a
512×384 world in a 1600×1000 viewport:

| zoom | wind, before | wind, after | ocean, before | ocean, after |
|---|---|---|---|---|
| 1 (fit) | 260 / 260 | 260 | 200 / 200 | 200 |
| 2 | 95 | **204** | 47 | **163** |
| 4 | 18 | **209** | 7 | **144** |
| 8 | **4** | **208** | 2 | **133** |
| 16 | **1** | **195** | 1 | **93** |

"Doesn't show finer patterns" was not a coarse field but an empty one. The
*seeding region* is now the visible slice: `_update_view()` projects the host's
rect back through `get_global_transform().affine_inverse()` into grid cells,
grows it 8 % so streaks drift in and out rather than popping at the edge, and
`_spawn` and `_step`'s retire test use it. The count never changes, so
grid-space density becomes screen-space. **At the fit view the slice is the
whole grid and behaviour is byte-for-byte unchanged** (the reference's constants
untouched; the 260 / 200 columns agree to the particle); zoomed in, the same 260
streaks resolve 1/256th of the area. Also cheaper: nothing advects a particle
sixteen screens away. The ocean column falls at 16× because that seed's centre
is partly land and the reference's 30-try wet-rejection loop correctly declines
to spawn; left as the reference has it (an all-land bay burns 7 800 four-byte
reads a frame and draws nothing — correct and free).

### FX-02 — the hairline is one *cell* wide, and a cell magnifies · **FIXED**

`width = maxf(1.0, sx)` matches the reference, which strokes `lineWidth = 1`
into a `GW × GH` backing canvas that CSS stretches — its hairline is one cell,
and it has no camera. Here the number is multiplied by `_camera.scale`, so one
cell at 8× is a ribbon eight cells thick. Now divided by the camera scale,
pinning the on-screen width to the fit view's at every zoom. A departure from
the literal formula under `DECISIONS.md` §7d, recorded as one: the reference's
*intent* was a hairline, and its arithmetic stopped reproducing that intent once
a camera was in front of it.

### FX-03 — the tip · **FIXED, and batched**

`draw_multiline`'s segments have butt caps, so a trail ended in a flat stub — a
pixel at the fit view, the owner's "square pixel" at 8×. The ocean view at 8×
drew **two fat squares on an otherwise empty map** — FX-01 and FX-03 in one
frame. Arrowheads now, at `k == 0` only (the only depth that *is* a head),
oriented along the last segment and sized in screen pixels divided back out by
the camera scale.

**One call for all of them.** A `draw_colored_polygon` per particle would add
260 canvas commands a frame — §52/§54's finding on `map_overlay.gd`'s dashed
polylines (311 237 objects in a frame, 751 MiB of buffers), being unwound in a
concurrent session as this landed. `RenderingServer.canvas_item_add_triangle_array`
takes the batch as loose triangles (an empty index array is legal if the vertex
count divides by three) with a single-entry colour array, so no per-vertex
`PackedColorArray` either. Measured on-minus-off at a fixed zoom, so the
deep-zoom tiler's own objects cancel:

| | wind, before | wind, after | ocean, before | ocean, after |
|---|---|---|---|---|
| objects the layer adds | +3 113 | **+3 056** | +2 391 | **+2 374** |
| draw calls the layer adds | +2 | **+3** | +2 | **+3** |

**Zero objects and one draw call** for 260 arrowheads. The ~3 100 is
`draw_multiline`'s own segments (260 × 12 trail depths), unchanged — meaning
Godot's objects-in-frame counts primitives, not commands, and the twelve grouped
calls were never twelve objects.

### `water_anim_layer.gd` shares neither fault

Checked: no particles, so no line ends; its noise lattice is already in *cell*
space (`F = 0.22` cycles per cell), so it magnifies with the map exactly as the
reference's `putImageData` does. `flow_tex` is one direction per cell and
`filter_nearest` deliberately, so finer ripples would be invented detail.
Untouched.

### Harness

`_flowzoom_probe.gd` / `.tscn`, non-headless in a 1600×1000 `SubViewport`
(`_hidpi_probe.gd`'s idiom — a real window is clamped to the 1680×1002 work
area, and headless produces no pixels). Seed 483920 at 512×384; for `wind` and
`ocean` at zooms 1 / 2 / 4 / 8 / 16 it reports on-screen particles, the layer's
objects and draw calls and frame-to-frame motion, and writes a frame plus a 3×
centre crop, run against committed `HEAD` and the fix into `before/` and
`after/`. Cost is debug-view-on minus debug-view-off at one fixed zoom, because
the deep-zoom tiler swaps 30 000 objects in and out on its own as the camera
moves, so a single global baseline does not work. `project.godot` md5
`ccba27c9280cf8373412e2ba87ed4054` unchanged across every Godot invocation; no
Rust, `.tscn`, `export_presets.cfg` or `Cargo.toml` touched.

---

## 56 · BI-01 — §54's "the chrome is not stable across boots" is two different builds on one handset, and the boot it blamed is deterministic (2026-08-25) — **NEGATIVE, ONE FIX**

§54's harness note recorded, in passing, that "the same APK" came up in two
chrome variants (icon tab bar vs text labels; 799 vs 944 objects, 9.5 vs 15.1 MB
`Gfx dev` at the welcome screen). Every device measurement here assumes a stable
baseline, and §54 could only say its own two runs "came up in the same variant"
without being able to check. It was handed on with a hypothesis:
`_compute_layout_mode()` decides `_phone` once, at `_ready()`, off a window size
Android may not have finalised, and `short/long < 0.6` would latch *tablet* on a
momentarily square-ish surface.

**The hypothesis is wrong, and the answer is worse than a race**: the two
variants are two different builds of the shell, installed over each other on
one handset by two agents working the same evening.

### The boot, measured — there is nothing provisional about it

A probe build from committed `HEAD` (`51b3230`) logs `get_viewport_rect().size`,
`DisplayServer.window_get_size()`, `screen_get_size()`,
`is_touchscreen_available()`, `OS.has_feature("mobile")` and the computed
`_touch` / `_phone` / `_landscape` / `_phone_scale` at four points in `_ready()`
(entry, after the touch decision, after `_compute_layout_mode()`, after
composition), then on **every** `root.size_changed` and from `_process` on every
frame the viewport size changes, for the first 900 frames. **OnePlus 6T (`9608b26b`), Android 15, 1080 × 2340,
52 instrumented cold starts** (`am force-stop`, `logcat -c`, `am start`, 7–14 s):

| | |
|---|---|
| Viewport size at the first sample inside `_ready()` | **1080 × 2340, all 52** |
| `root.size_changed` emissions, whole corpus | **0** |
| `_phone` / `_landscape` / `_phone_scale` | `true` / `false` / `2.6214`, all 52 |
| Frames on which the viewport size changed | none, in 900 frames a run |

One size, read before anything is decided. `OnGodotSetupCompleted` (which runs
`_ready()`) arrives ≈3.0 s into the process, long after the surface and insets
settle, and `OnGodotMainLoopStarted` follows it.

Three of the 52 ran with `accelerometer_rotation=0` and `user_rotation=1`
(system forced to landscape); the activity still came up `1080 × 2340`, because
`window/handheld/orientation=6` asks for `SCREEN_SENSOR` and the sensor answers.
So **`--force-rotate` over `adb` is not a way to reach `_landscape` on this
device** — the wall the 2026-08-20 pass hit from the other side
(`ANDROID_BUILD_SCOPE.md`, "Portrait was not reachable over `adb`"). And the
aspect test cannot flip even in principle: `_phone` needs `_touch`, fixed for the
process, and `min/max` is rotation-invariant — 1080/2340 = 0.4615 either way.

### The chrome, measured — it is perfectly stable, and then it is not

The probe prints a chrome fingerprint six seconds after the shell is up
(`Performance.OBJECT_COUNT`, `RENDER_TOTAL_OBJECTS_IN_FRAME`,
`RENDER_VIDEO_MEM_USED`, scene-tree node count, phone flags, the L1 bar's child
count and labels, both dock sheets' visibility, `DccIcons.cache_stats()`).
Twenty runs of `HEAD`, `dumpsys meminfo` on ten:

```
objects=11061  render_objs=456  nodes=3454  phone=true  pscale=2.6214
bartext=MORE|PANELS|CARTO|CIVIL|WORLD    Gfx dev 14796–15016 kB
```

**Identical to the digit on every run** — `objects`, `render_objs`, `nodes` and
the bar's contents never moved; `Gfx dev` spans 1.5 % and `TOTAL PSS`
0.5 %, driver noise on a 450 MB process. Then the same probe from `c9bb82e` —
the commit before §53's 412 dp migration (`5600c60`):

| welcome screen, OnePlus 6T | `c9bb82e` (pre-412) | `51b3230` (`HEAD`) | §54 reported |
|---|---:|---:|---|
| `Gfx dev` | **9 452 / 9 636 / 9 500 kB** | **14 796 – 15 016 kB** | **9.5 vs 15.1 MB** |
| `OBJECT_COUNT` | 10 804 | 11 061 | — |
| scene-tree nodes | 3 370 | 3 454 | — |
| L1 bottom bar | **text labels**, `WORLD CIVIL CARTO PANELS MENU` | **icon tab bar**, `MORE` for `MENU`, pill buttons | "icon tab bar" vs "text labels" |

§54's two numbers reproduce to the decimal, on demand, from two commits:
`c9bb82e` and `5600c60`, not two boots. (§54's prose pairs the clauses in
opposite order — the icon bar is the *heavier* variant — the one thing in that
note to correct as written rather than as meant.)

### Why it happened, and it is not really a shell bug

§54's own harness note has the answer without naming it: the concurrent session
mid-edit on `shell/dcc_shell.gd` was **§53's 412 dp migration**, not committed
until `5600c60` at **21:38** — after every APK in `builds/android/` had been
exported: `Cartalith-mem.apk` 20:28, `Cartalith.apk` 20:49,
`Cartalith-dashA.apk` 21:09, `Cartalith-ph412.apk` 21:27. The first three are
§54's, from a still-pre-412 committed `HEAD`; the fourth is the §53 agent's,
carrying the migration eleven minutes before it was a commit. Both agents
installed to the **same package name** (`org.cartalith.walkingskeleton`) on the
**same handset**, and `adb install -r` from either replaces the other. "The same
APK" was two APKs, and nothing in the running app said which.

### BI-01, fixed — `DccShell.build_id()`

The shell now prints one line before it does anything else:

```
Cartalith shell build 756b59e30bc1
```

An MD5 over every file shipped under `res://shell/`, plus `res://map_overlay.gd`
by name, hashed with their paths in sorted order, truncated to 12 hex. No
version constant to forget to bump; it changes when the shipped scripts change.
**The GDScript twin of `EngineBridge._has()`** (§24, added after a `.so` ran 21
commits behind its shell in silence; `ANDROID_BUILD_SCOPE.md`): that guard speaks
for the native half, and nothing spoke for the script half — which cost this
register a startup race that does not exist.

Verified three ways:

- **Determinate**: two consecutive headless runs of one tree → `cc30740ce765`
  both times.
- **Sensitive**: one appended comment line in `shell/right_dock.gd` moved it
  `dcb61a49afd8` → `d1e90bc7c590` — in the editor, where the digest is over
  `.gd` source.
- **Reaches the handset**: `756b59e30bc1` in `logcat` from a cold start of the
  exported APK.

Export and editor digests of one tree differ on purpose (an export ships `.gdc`
+ `.gd.remap`, the editor `.gd` + `.uid`) — genuinely different builds, never to
be conflated in a log. **In an export it is a *behaviour* fingerprint, not a
source one**, measured once the first export refused to move: the same appended
comment leaves `assets/shell/right_dock.gdc` **byte-identical**
(`a8ee35357f4aa4a04cd5f8f39fd323ef`) while a one-line *code* change moves it to
`ed7b699f2580372cc9e26404b2f8f84c` — `script_export_mode=2` compiles to binary
tokens without comments or whitespace. Right granularity for "is this the same
build?", wrong one for "is this the same commit?" — for that, hash the APK.

**What it does not do**: it identifies the build *running*, not that the APK
installed is the one just exported. The check that closes that loop is now the
rule for any device pass comparing two builds:

```
adb shell sha256sum $(adb shell pm path org.cartalith.walkingskeleton | sed 's/package://')
sha256sum builds/android/<the file you just exported>
```

Run against this pass's own build: `36b71ff0…e4ec0dc1`, both sides.

### `_on_window_resized()`'s asymmetry — audited, deliberately unchanged

The referral flagged the early return —

```gdscript
func _on_window_resized() -> void:
	if not _phone:
		return
```

— as possibly the bug: a shell that latched tablet by mistake could never
correct itself. It is not, and hoisting `_compute_layout_mode()` above the guard
is wrong twice: there is nothing to correct (the 52 cold starts; `_touch` fixed,
aspect rotation-invariant), and **recomputing for tablets would break something
real** — the tablet branch assigns `_left_width`/`_right_width`, so every
rotation would reset a tablet user's dragged dock widths (WI-04) to
`W_DOCK_TABLET`. Both reasons are now in the function's doc comment. The desktop
`--resolution WxH` clamping it was compared to is a different fault of the same
shape: the window really is a different size than asked, before `_ready()` runs,
so what is wrong there is the harness's expectation, not the shell's timing.

### What this does to past measurements

**No number in this register is retracted, and one is now positively
defensible.**

- **§54's headline stands.** Its A and C runs came up at 9.5 / 9.7 MB — the
  pre-412 pair — and both APKs were snapshotted from committed `HEAD`, pre-412
  throughout (`c9bb82e` 20:19 → `ba5d351` 20:32 → `d40acb3` 20:57; the migration
  lands at `5600c60` 21:38). Right to say so and right to doubt it.
- **The exposure is real but narrow**: 2026-08-25 evening, the handset, and the
  interval between §53's migration entering the working tree and `5600c60` —
  the only window in which two materially different shells could overwrite each
  other under one package name. §52's memory pass (earlier, single build) and
  §50's and §53's device passes each measured one build against itself, which
  survives this.
- **The real cost was diagnostic, not numeric**: a paragraph recording a phantom,
  and an evening disproving it. `build_id()` makes that unrepeatable.

Registered, not fixed: nothing. BI-01 is closed by the stamp.

### Harness

- **Built from committed `HEAD` (`51b3230`), not the working tree**, per §54's
  rule: `git archive HEAD cartalith-native/godot-project` into a scratch tree, an
  NTFS junction back to the real `cartalith-native/target/` so
  `cartalith.gdextension`'s `res://../target/...` resolves, then `--import` and
  `--export-debug "Android"` there. The fix build layers the one edited file on
  that snapshot; the pre-412 build is the same recipe at `c9bb82e`.
- `project.godot` md5 `ccba27c9280cf8373412e2ba87ed4054` unchanged across every
  Godot invocation in the real tree and both snapshots, `;` block intact;
  `export_presets.cfg` and `Cargo.toml` untouched, no Rust changed,
  `shell/wind_fx_layer.gd` / `shell/water_anim_layer.gd` not opened.
- Device state: `accelerometer_rotation` 0 / `user_rotation` 1 for three runs,
  **both restored** (1 / 0). The handset is left with `Cartalith-buildid.apk` —
  the fix build, `HEAD` plus `dcc_shell.gd` — rather than a probe.
- Noticed in passing, not this pass's to fix: the `android-dev`
  `libcartalith_godot.so` in `target/` is behind the shell, and
  `EngineBridge._has()` says so — `WorldGen.project_save()` missing, once, at
  every boot of every APK built this evening. It wants a
  `cargo ndk -t arm64-v8a build --profile android-dev -p cartalith-godot` before
  the next device pass measures anything that saves.

## 57 · DS-03 and DS-13 re-opened, and a religion-screen design that was premature — a canvas pass whose audit refuted all three of its own designs (2026-08-29) — **THREE REGISTERED, NOTHING BUILT**

The owner asked for the `/ui-ux-pro-max` skill to design the screens the
canvases do not cover — §48's two registered items plus the religion-diffusion
subsystem's surfaces. The pass ran extract → design → **adversarial audit**, two
independent auditors per design, one hunting fabricated provenance and one
buildability. **It produced no code, and that is the result**: all three designs
were refuted — 50 findings, 8 at high severity — and each would have read
correctly in review and shipped broken. What follows is what the audit
established, so a later pass starts from it.

### The finding that re-opens DS-03: the tablet is not a scaled desktop

§48 registered the tablet interior as *"a scaling layer that does not exist
rather than a value that is wrong"*, warning that bolting `phone_fit` onto tablet
with a guessed unit *"is how the next pass gets a tablet that is neither."*
Right instinct, too shallow a diagnosis: **a scaling layer of any unit is the
wrong shape.** Pairing `DCC shell tablet 2560` against `DCC shell 1920` element
by element (~55 interior elements) gives ratios from **×1.00 to ×2.06** with no
centre: hairlines, letter-spacing, the layers FAB and the rail's vertical padding
are *pinned*; type runs ×1.14–×1.30; box metrics ×1.20–×1.57; the action
button's height is **×2.06**. Sans and mono take different multipliers off the
same 11 px rung — sans 11→14, mono 11→13.

Two mechanisms a multiplier cannot express, and which are what the tablet
artboard actually *is*:

1. **A `min-height` touch system introduced from nothing** — `min-height:44px`
   **29 times** and `min-height:34px` 3 times on the tablet artboard, **zero**
   `min-height` declarations on the desktop one. A new constraint layer, not a
   scaled property.
2. **Roughly 30 % of the desktop's content is deleted** — the whole PROPERTIES
   section, the six per-layer opacity minis, the scale bar, the layers popover,
   2 of 6 layer rows and 4 of 13 sample fields. That deletion buys room for
   44 px rows in a band that only grew ×1.498.

**So the tablet is a content decision before a styling one**, and an owner call
rather than a fix: *which controls leave the tablet.* Registered, not taken.

### The architectural blocker under it: `DccTheme.TABLET`'s key space is exhausted

`DccTheme.TABLET` is keyed by the bare desktop integer, and the tablet artboard
maps one desktop figure to two tablet figures in at least five places, each
verified:

| desktop | tablet, in one place | tablet, in another |
|---|---|---|
| `14` | **22** in a bar | **18** in a dock |
| `9` | **15** in the menu bar | **11** in a popup group |
| `6` | **9** in the sample grid | **6**, pinned, in the layers-FAB column |
| `11` | **13** in mono | **14** in sans |
| `70` | **88** as the timeline height | **90** as the tool-options track |
| `40` | **48** as the rail | **40**, pinned |

A value-keyed table cannot hold that; a real fix needs **role-keyed**
resolution, and the audit refuted the obvious placement for it too (below).

### Three high-severity refutations against the proposed tablet fix

1. **The `fit()` dispatcher would have doubled the phone's scale.** The design
   folded `phone_fit(node, unit, wide)` and a new `tablet_fit` behind one
   `DccShell.fit(node, wide)`, dropping `unit`. **22 of the ~25 call sites pass
   `1.0` for a load-bearing reason the design itself quoted** — `dcc_shell.gd:858`:
   *"`1.0` for a `Window` that has already set `content_scale_factor` to
   `_phone_scale` … applying it again here would double it."*
2. **Touch tiers A (44) and B (34) are unreachable as specified.** Both land on
   the same Godot class in the same walked subtree — the mode chips are
   `DccWidgets.segment()` → `chip()` → a plain `Button`, built into
   `tool_options_row`, exactly what the walker is called on. Tier B can never
   fire.
3. **Role-keyed resolution at `_row()`/`label()` re-enters a trap the shell has
   already paid for.** Both are `static` with one predicate available,
   `DccTheme.is_touch()` — **true on phones**, because `_phone` requires `_touch`
   (`dcc_shell.gd:335`). `dcc_shell.gd:249` already records the lesson: *"the
   412 canvas asks for things a tablet must not get."*

### DS-13: four high-severity refutations, and the canvas anchoring was wrong too

- **`tool_pan` is navigation after all.** The design removed it as "a tool
  toggle, not navigation", but `viewport_host.gd:457` is
  `elif mb.button_index == MOUSE_BUTTON_LEFT and _pan_mode:` — **no armed-tool
  condition**. With no tool armed, `_pan_mode` is what makes a one-finger drag
  pan the camera at all.
- **Zoom would have been removed into nothing.** Its stated destination,
  `MORE ▸ VIEW ▸ Zoom`, exists only in the canvas: `phone_menu.gd:82-86`
  projects `menus.gd`'s seven menu-bar menus and **there is no View menu**;
  anything unmatched falls through.
- **There is one navpad and it is not phone-gated.** `_build_navpad()` opens
  `if not _touch: return`, so a touch **tablet** builds the same four pills the
  design rewrites, and `viewport_host.gd` has no phone/tablet discriminator — so
  "the desktop/tablet navpad is untouched" describes nothing.
- **Three colour equalities are arithmetically false.** `rgba(20,22,23,.92)` is
  `#141617`; `DccTheme.DARK.panel` is `#121314` = rgb(18,19,20). `rgba(16,18,19)`
  is `#101213`; `sunken` is `#101112`. The design disproved its own claim two
  decisions later, in its FAB-glyph-ink note.
- Plus: the design anchored the control column bottom-right per the 412 canvas,
  but **both canvases that draw zoom buttons anchor them top-right** (`2b Phone
  map` at `right:12px;top:12px`, `2a Tablet` at `right:14px;top:14px`), and the
  412 canvas's top-right region is drawn **empty**. And `_apply_touch_scale()`
  enforces **one** size for every navpad child, so the canvas's 48 dp FAB over a
  44 dp pill — and its 17 px/15 px glyph split — are erased before they reach
  the screen.

### The religion screens were designed too early, and the audit says so precisely

- **No data path exists, and the design never said one was needed.**
  `get_settlements()` (`cartalith-godot/src/lib.rs:4591`) emits `x, y, name,
  population, kind, faction, capital, coastal, tid` — no religion field, no
  adherent counts; `engine_bridge.gd` carries only `civ_religion_vocabulary()`
  (the 8 labels). Every share, plurality and per-settlement year in the design
  had nothing behind it. `RELIGION_DIFFUSION_SCOPE.md` §2 already names this the
  central gap; the design should have opened with it.
- **"Read-only-ness is carried by the absence of a thumb" fails on phone**:
  `phone_fit()` walks every Control under `right_dock` and unconditionally does
  `if ctl is HSlider: DccWidgets.phone_slider(...)`, re-adding a 22 dp accent
  grabber to a read-only proportion bar.
- **The tablet artboard deletes the proportion bar from dock rows entirely** —
  the design's tablet story contradicts the canvas it claimed to follow.
- One **fabricated provenance**, caught by the auditor grepping the cited
  artboard: a stale-mark style attributed to `DCC shell 1920`, which returns
  **zero** hits for `stale`. The markup is real and lives in
  `DCC Generate World 1920`.

**Sequencing lesson:** the religion surfaces cannot be designed before
`cartalith-civ::belief` and its bridge exist, because every value in the design
is a number the engine does not yet produce. Design against the real bindings,
not the scope document's intent.

### What a later pass should do differently

1. **Take DS-03 to the owner as a content question first.** "Which controls
   leave the tablet" is not answerable from the canvas by a pass; the canvas
   only records an answer somebody already gave.
2. **Do not add a second value-keyed table.** The key space is exhausted, and
   the audit refuted the one obvious place for a role-keyed resolver.
3. **Re-run the same audit shape.** Two adversarial auditors per design — one on
   provenance, told to grep the cited artboard; one on buildability, told to read
   the real shell code — found eight high-severity errors in designs that all
   read as competent. The provenance grep caught the fabricated citation.

## 58 · PH-28 — a 16:9 tablet gets the phone layout, and its controls are clipped off the screen (2026-08-29) — **MEASURED, ONE OWNER DECISION**

The owner asked whether the APK gives each device its respective layout: the
OnePlus 6T for phone, plus a simulated 2K and 4K tablet. Driven on the real
handset (`9608b26b`, ONEPLUS_A6013) against a fresh `--export-debug` build
carrying that day's work, with `adb shell wm size` / `wm density` overrides and
the device reset afterwards. **Three of four geometries are right; the fourth is
a defect of the classifier, not of any layout.**

| Geometry | short/long | Layout drawn | Correct? |
|---|---|---|---|
| 1080×2340 — native 6T | **0.4615** | phone: bottom nav, ☰ bar, navpad | yes |
| 2560×1600 — 2K tablet, 16:10 | **0.625** | tablet: menu bar, rail, both 400 px docks | yes |
| 3840×2160 — 4K, **16:9** | **0.5625** | **phone** | **no** |
| 3840×2400 — 4K, 16:10 | **0.625** | tablet | yes |

### The cause, and the controlled experiment that isolates it

`dcc_shell.gd:335` is the whole classifier:

```gdscript
_phone = _touch and (short_side / long_side) < _PHONE_ASPECT_MAX   # 0.6
```

**Aspect is the only discriminator; resolution and physical size are not
consulted.** A 16:9 panel is `2160/3840 = 0.5625`, under 0.6, so a 4K 16:9
tablet is a phone — and 16:9 is an ordinary tablet, Chromebook and
Android-desktop aspect. The last two rows are the controlled experiment: **the
width is 3840 in both**, only the height changes, and the layout flips.

### It is not a cosmetic misclassification — the shipped result is clipped

Measured on the 3840×2160 run:

- **The welcome gate renders full-screen with its content cut off** — the
  "Create a new world" card truncated mid-card, the footer on top of it. That is
  `phone_present()` doing its job
  (`dlg.min_size = Vector2i.ZERO; dlg.max_size = Vector2i.ZERO`,
  `dcc_widgets.gd:1156-1157`) on a screen it was never meant
  for; at 2560×1600 the same dialog is a correct centred modal.
- **The navpad pills are clipped off the right edge**, the topmost cut through
  horizontally.
- **The action row runs off the screen**: `BAKE AL…`, unreachable.
- No docks on a 3840 px display, and phone-scale type across the whole width.

### The owner decision

The threshold is not an oversight — `_PHONE_ASPECT_MAX`'s comment calls it
*"Midpoint-ish between 19.5:9 (~0.46) phones"* and tablet aspects, correct for
the two classes it was chosen against. The gap is that **aspect alone cannot
separate a 16:9 tablet from a 16:9 phone in landscape**; only size can. A size
condition discriminates in practice: at the tested densities the 4K panel is
**1920×1080 dp** and the 6T **384×832 dp** — no phone is 1920 dp wide — so a
short-side dp floor (from
~600 dp — Android's `sw600dp` tablet breakpoint — upward) separates them without
moving any correct case.

**Not applied here.** Which devices count as phones is a product decision with a
blast radius over every `_phone`-gated path — `phone_fit`, `phone_present`,
`phone_menu.gd`'s whole navigation model — and this pass has a measurement, not
a mandate. The one-line shape of the fix is `_phone = _touch and aspect <
_PHONE_ASPECT_MAX and short_side_dp < PHONE_MAX_SHORT_DP`.

### Related: DS-03 is now visible rather than merely measured

§48 registered the tablet interior as unscaled and §57 found a scaling layer the
wrong shape for it. The 2K and 4K-16:10 runs show the cost on a real panel: at
3840×2400 the dock type is the desktop's own 10.5 px inside a 400 px dock on a
3840 px screen — the larger the tablet, the worse it reads, because nothing in
the interior is a function of the display.

### Build and artefact

`cargo ndk -t arm64-v8a build --profile android-dev -p cartalith-godot` then
`--export-debug "Android"` — the `.gdextension`'s `android.debug.arm64` line
resolves to that `android-dev` output, checked before exporting.
207 699 354 bytes, md5 `f9c9e01984eddee01da82f8497b6b5b0`, copied to
`D:\Users\Vincent\Documents\Vincent\Persoonlijk\Writing\Tools & writing hacks\
Cartalith.apk` and md5-verified there. Device size and density reset to physical
(1080×2340, 450); the override density of 314 the device carried on arrival was
**not** restored, since an earlier session left it rather than it being the
device's own setting.
