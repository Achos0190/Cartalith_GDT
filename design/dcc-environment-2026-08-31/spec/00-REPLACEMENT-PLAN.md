# Replacing the GUI — the plan

Owner instruction, 2026-08-31: **"Replace the current GUI, do not upgrade. Fully
replace."**

Six specification passes ran over the two prototypes and are in this directory.
This document is what has to happen to the code, and it opens with the condition
the prototypes arrived in, because that shaped everything under it.

> **This document defines the replacement — the structural delta, the
> file-by-file actions and the seven stages. It does not track them.** Which
> stages have run, and what is in the tree today, lives in
> **`cartalith-native/docs/STATUS.md`** and nowhere else. Read a stage below for
> *what it is*, never for whether it is done.

---

## 0 · How the prototypes were delivered, and the 84 holes that left

**`Cartalith DCC Environment.dc.html` first arrived as exactly 262 144 bytes —
256.0 KiB on the nose.** That is the design MCP's `get_file` cap, not a
coincidence, and the file ends mid-word inside the logic class:

```js
measRows.push({i:('0'+i).slice(-2),len:this.fmtKm(km),be
```

— cut in the middle of `bear`, inside `valsCore()`.

`Cartalith Android.dc.html` is **166 424 bytes and complete**, ending properly
with `</script></body></html>`.

### What that did and did not cost

The **markup** survived, and the markup is where the layout, the labels, the
structure and the token sets live. So the frame, the rail tree, the seven menus,
the dock contents and the geometry were all specified and buildable from the
first delivery.

What was missing was the **tail of `valsCore()`'s return object** — the bindings
that say which string or colour fills each hole. That is why the six passes
raised **84 `UNSPECIFIED:` items**, and why so many of them read "was in the
truncated return".

Concretely, the following could not be built without guessing, and this
repository does not guess at a design:

- `ldPipe` — the gate that selects the Generation-pipeline dock (its complement
  `ldSculpt` survives, so the shape is knowable but the file does not say it)
- every string and colour in the pipeline run block — `runStageLabel`,
  `runChainLabel`, `pipeNote`, `progTitle`, `progPct`, `finLabel`, `bakeLabel`
- `rdTitle` for Sample, Measure and Region, and `rdCollapsedLabel` everywhere
- `sampleRows` — the row labels, order and formatting of the sample readout
- `regionRows`, `regionReadout`, `measSegCol`/`measPathCol`, `hMeasMode`
- `tbLabel` for most tool-options contexts
- `statusMid` — **no candidate exists anywhere in the delivered bytes**
- `vpContext`, `vpField`, `scrimBg`, `mapCursor`, `layersBtnBg/Col`
- `tlShow` / `tlCollapsed` / `tlExpanded` — the timeline's gating

**The ask that followed**, and its answer: re-export the desktop prototype under
256 KiB — either split it into two `.dc.html` files (frame + docks, say), or
strip the embedded map-drawing code, which is a large block that the GUI port
does not need. The phone file needed nothing. **Answered 2026-08-31**: the file
in this directory is 239 712 bytes and ends properly with
`</script></body></html>`, its heavy method bodies split into
`cartalith-dcc-parts.js` behind `window.CDCC`. The bindings listed above are
readable from the prototype rather than guessed at; the list stays as the record
of what the six specification passes had to leave `UNSPECIFIED:`.

---

## 1 · The structural delta, most consequential first

### 1.1 The rail goes from five domains to three

> **Owner ruling, 2026-08-31:** *"Infra gets absorbed by civil and render by
> Carto."* The fold is deliberate and is not up for re-litigation. INFRA and
> RENDER stop being top-level domains; their content survives as nodes under
> CIVIL and CARTO.

Before the fold: `WORLD · CIVIL · INFRA · CARTO · RENDER`, five flat domains.

After: **three domains, each a header over a node tree.**

| Domain | Node | mode |
|---|---|---|
| WORLD | Generation pipeline | `a` |
| | Sculpt | `b` |
| CIVIL | Landmarks | `landmarks` |
| | Factions & settlements | `factions` |
| | Ways & routes | `infra` |
| | Journey planner | `planner` |
| CARTO | Layers & style | `''` |
| | Labels | `''` |
| | Icons | `''` |
| | Terrain appearance | `''` |

So **INFRA folds into CIVIL** and **RENDER folds into CARTO**. This is the one
part that genuinely cannot be an upgrade.

**Every consequence, enumerated:**

| Site | What changes |
|---|---|
| `dcc_shell.gd` `DOMAINS` | five entries → three, plus a node tree per domain |
| `dcc_shell.gd` rail build | flat cells → header + node rows, with the `inset -2px 0 var(--acc)` active marker |
| `dcc_shell.gd` `select_domain_category(id, category)` | signature must carry a *mode*, not just a category |
| `app.gd` workspace registry | five workspaces → three hosts, with `infrastructure_workspace` and `render_workspace` becoming *modes inside* CIVIL and CARTO rather than domains |
| `workspaces/infrastructure_workspace.gd` (1 103 lines) | already composed into CIVIL via `_nested`; becomes a mode |
| `workspaces/render_workspace.gd` (1 047 lines) | already composed into CARTO via `build_*_into`; becomes a mode |
| `phone_menu.gd`, `dcc_shell.gd` phone bar | the phone's domain cells and MORE list |
| `command_index.gd` | groups by domain; the group set changes |
| `faction_roster_window.gd:682`, `civilization_workspace.gd:399`, and every other `select_domain_category` caller | re-point |

**What makes the fold cheap**, as the shell stood when this plan was written:
INFRA and RENDER were *already* composed into CIVIL and CARTO rather than being
independent — `infrastructure_workspace` runs with
`_nested = true` inside civ, and `render_workspace` exposes
`build_presets_into` / `build_colours_into` for cartography to call. The fold is
closer to a promotion of what exists than to a rewrite.

### 1.2 Token and density changes

| Token | Before | New | Note |
|---|---|---|---|
| `--ins` (inset/cell) | `#101112` (`sunken`) | **`#191c1e`** | the new value is §11's own; the one it replaces had drifted |
| `--wash` | `.08` | **`.09`** | §11's value |
| `--accInk` | *absent* | **`#141005`** | reversed ink on filled accent — new token |
| `--wash2` | *absent* | **`.16`** | armed-tool wash — new token |
| `--menuH` | 34 | **36** | |
| `--tbH` (tool options) | 34 | **40** | tablet 52 → **56** |
| `--rdW` (right dock) | 300 | **304** | |
| `--fs` (base) | 11/12 | **11.5** | |
| semantic | `#e0a840 / #b55950 / #7d9dae` | **`#6fae7d / #c96a5a / #6a9bc4`** | good / block / water |

### 1.3 A fourth breakpoint

`LAPTOP 1366` — `--ldW:330 --rdW:280 --pop:280`. The shell had desktop, tablet
and phone; this adds a narrow-desktop density between desktop and tablet.

---

## 2 · File-by-file

| File | Lines when written | Action | Why |
|---|---|---|---|
| `dcc_shell.gd` | 4 339 | **REWRITE-INTERIOR** | rail 5→3 + node tree, new tokens, the 1366 breakpoint; the phone half is a separate rewrite from the complete phone spec |
| `dcc_theme.gd` | 739 | **REWRITE-INTERIOR** | new token values, two new tokens, four density sets |
| `dcc_widgets.gd` | 1 551 | **RETARGET** | the factories stay; their metrics come from the new tokens |
| `app.gd` | 1 859 | **REWRITE-INTERIOR** | workspace registry 5→3, rail wiring |
| `menus.gd` | 2 758 | **RETARGET** | seven menus unchanged; only styling and any new rows |
| `workspaces/world_workspace.gd` | 1 843 | **RETARGET** | becomes WORLD's two modes |
| `workspaces/civilization_workspace.gd` | 3 608 | **RETARGET** | becomes CIVIL's four modes |
| `workspaces/cartography_workspace.gd` | 1 192 | **RETARGET** | becomes CARTO's four nodes |
| `workspaces/infrastructure_workspace.gd` | 1 103 | **RETARGET** | domain → CIVIL mode `infra` |
| `workspaces/render_workspace.gd` | 1 047 | **RETARGET** | domain → CARTO node |
| `right_dock.gd` | 1 759 | **REWRITE-INTERIOR** | new contexts; needs the `rdTitle`/`sampleRows` bindings §0 lists |
| `tool_bar.gd` | 606 | **REWRITE-INTERIOR** | needs the `tbLabel` bindings §0 lists |
| `phone_menu.gd` | 1 100 | **REWRITE-INTERIOR** | from the complete phone spec |
| `viewport_host.gd` | 2 060 | **RETARGET** | furniture restyled; needs `vpContext`/`vpField` |
| `layers_popover.gd` | 482 | **REWRITE-INTERIOR** | new popover; needs `l.bg`/`l.col` |
| `dcc_icons.gd` | 365 | **KEEP** | the prototype imports this repo's own glyph set |
| `engine_bridge.gd` | 3 207 | **KEEP** | not GUI |
| the nine windows | 6 363 | **KEEP, RESTYLE** | the prototypes do not specify them; artboards being ported separately |
| `asset_library_window.gd`, `data_manager_window.gd`, `journey_planner_view.gd`, `travel_library_window.gd` | 9 391 | **KEEP, RESTYLE** | designs exist in the older canvases |

**Nothing is deleted outright.** The fold of INFRA and RENDER moves code; it
does not discard it.

*The line counts above are a snapshot taken while this plan was written, and are
a sizing input rather than a measurement of the tree. They drift; `STATUS.md`
records by how much. Re-measure before leaning on one.*

---

## 3 · Build order — every stage leaves the app runnable

A big-bang rewrite of 48 609 lines is not acceptable here; every stage below is
independently probe-verifiable.

1. **Tokens.** `dcc_theme.gd` to the new values, plus `--accInk` and `--wash2`
   and the 1366 density. Nothing structural. Verified by the existing probes
   still passing and by a screenshot diff.
2. **The rail, 5 → 3.** The node tree, `select_domain_category` gaining a mode,
   the two workspace folds, every caller re-pointed. **This is the risky stage**
   — new probe needed: assert all three domains reach every node, and that every
   pre-existing category is still reachable under its new home.
3. **Menus.** Restyle only; `_cmdindex_probe` guards the row count.
4. **Left dock**, mode by mode, against `04-left-dock.md`.
5. **Right dock, tool options, status, timeline, viewport furniture.** Needs the
   `rdTitle` / `sampleRows` / `tbLabel` / `statusMid` bindings §0 lists, which
   the re-exported prototype supplies.
6. **Phone**, from the complete `06-phone.md`.
7. **The nine windows**, restyled to the new tokens.

---

## 4 · Risks, and what catches them

| Risk | Caught by |
|---|---|
| A category becomes unreachable in the 5→3 fold | **no existing probe** — needs a new one, per stage 2 |
| Menu rows lost | `_cmdindex_probe` (330 entries / 247 commands today) |
| Tablet parity broken | `_tabletparity_probe` (223 rows both legs) |
| Phone tap floor regressions | `_phonechrome_probe` |
| Landmark panel broken by the CIVIL fold | `_landmark_probe` |
| Token drift between shell and design | **no probe** — needs one asserting `DccTheme` against this spec |

---

## 5 · For the owner

1. ~~**Re-export the desktop prototype under 256 KiB**~~ (split it, or strip the
   embedded map-drawing code). **Answered 2026-08-31** — see §0. The split file
   carries the bindings stages 2 and 5 were written against.
2. **CARTO's four nodes all carry an empty `mode`.** Do `Layers & style`,
   `Labels`, `Icons` and `Terrain appearance` select four distinct dock panels,
   or collapse to one? The design gives four rows and one destination.
3. ~~**`statusMid` has no candidate anywhere in the delivered file** — what does
   the middle of the status bar show?~~ **Answered 2026-08-31** by the re-export:
   `statusMid` reads
   `last pass <NN stage-name> · <n> ms · repaint 84 ms · autosave <time|off>`,
   taken from the last stage whose `stageState` is `resolved`.
4. ~~Does the fold lose anything you meant to keep?~~ **Answered 2026-08-31:**
   *"Infra gets absorbed by civil and render by Carto."* Deliberate. See §1.1.
5. The full 84-item list is at the foot of each spec file under
   `UNSPECIFIED:`.
