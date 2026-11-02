# HTML plan after stage 2: v2.74 to stage 3, then the feature ports

*2026-11-01. This is a plan, not a status record. Its inputs are the performance, visual and feature audits of `html/Cartalith v2.72 DCC test.html` (md5 `57e6628a…`, commit `7bd3b49`), all of them read-only. Nothing has been built from them yet. Figures are the audits' own, cited here and not re-measured. "Estimate" marks a gain an audit computed but did not measure; "computed" marks token arithmetic. Line numbers are v2.72's unless marked otherwise.*

## 0. What I re-opened before planning (read-only)

- **Owner-question premises.** I re-checked each premise at its symbol in the committed v2.73 (`git show HEAD:`). That copy was last changed at `dfc38b2` (md5 `e2b42d64…`) and matched the working file. All of the following still hold:
  - `_jpClaimedAt` returns `civTerritory[…]>=0` on a `Uint8Array` (line 26143).
  - `_civBakeKey` reads `state.sun` (23273).
  - `_applyDomainUI` ends in an unconditional `_scheduleViewportRefit()` (18580).
  - `_lodScheduleOverviewRebuild` uses `setTimeout` (14723), and `LOD_OV_STRETCH_STREAK_CAP=4` (14860).
  - `_civSyncCanvas` resizes per zoom (22244).
  - The boot `renderNow()` runs behind the gate (21555).
  - `.busy` has no z-index; `#creditsModal` is at z-index 50.
  - `#scaleBar` is at .55 alpha and `.legend` at .74.
  - `*{border-radius:0 !important}` and `button.on{background:var(--accent)…}` are present.
  - There is no `color-scheme`. The light theme's `--accent` is `#a4650f`; `--faint` is `#6f7478` (dark) and `#6e6f69` (light).
  - `_stUpdateVisible` still sets `row.style.display=''`.
  - The place popup is still `color:#e8dcc4`.
  - 36 `bind()` sliders call `render()` on input in both files (`grep -n "^bind('" FILE | grep -c "render()"`).
  - `bakeSingle`'s only caller is `exportZip` (16685).
  - There are 9 `data-civtool` values; `_setUnits` accepts only km and mi; there is no session mode and no measure tool.
- **Two things moved in v2.73.**
  - **`_dskPlace` is gone.** `_shellPlace` (19978–20001) replaced it, and it still calls `_syncChromeTop()` and then `_scheduleViewportRefit()`.
  - **The shells are keyed differently.** v2.73 keys them on `html[data-shell="phone"]` (201 selectors), `html:not([data-shell="phone"])` (170) and `html[data-shell="tablet"]` (59). v2.72's `@media (max-width:860px)` pair is gone, but the audit's `fixes.css` still uses it.
- **Rulings searched.** I searched `LARGE_ITEM_RULINGS.md` for each question's subject: radius, accent, parchment, emoji, Plex, status bar, preview, blocky, claimed and toll, session mode, measure. Only two rulings bear on the questions:
  - the 2026-09-06 washed on-state (item 7);
  - the port's move off radius 0.
- **Cross-checked documents.** `html/README.md`'s stage plan, `STATUS.md`'s stage-1 entry, and `OUTSTANDING_WORK.md` under "The HTML working copy". Two of this plan's items are already filed there.
- **Preflight rows.** The rows named per version were chosen by reading the whole left column of `MISTAKES.md`'s table (201 rows) against each version's verbs.

## 1. The order

| Version | Carries | Size | Risk | Owner input | Gate in one line |
|---|---|---|---|---|---|
| v2.73 | Stage 2, the phone and tablet shells (in progress, unchanged) | L | shell | none | its own (`html/README.md`) |
| **v2.74** | **Fix pass**: broken controls, contrast and native controls, clipping, boot speed, the Sun slider's cache key | M (31 small items) | CSS, shell JS, one cache key | Q3, Q9, Q10a–b (Q2 and Q11 if answered in time) | `hash_gen1` ALL IDENTICAL; 0 text runs below 4.5:1 in the seven audited views, in both themes |
| **v2.75** | **Generate speed**: routing-surface cache and seeding, stage progress, the civ edge-cost table | M–L | engine | none | `hash_gen1` ALL IDENTICAL plus direct byte-compares; generate at 2048 faster, with disjoint ranges |
| **v2.76** | **Interaction speed**: wheel and pinch, LOD zoom, slider drags, territory brush, render tail; the view kept across domain switches | M–L | render, LOD, civ layer | Q10c–e, Q11 | exact render byte-identical; deterministic counters |
| **v2.77** | **Design alignment**: dock controls, labels, radius, icons, strip chips, headings, type | M | CSS, small markup | Q4–Q8 | contrast and truncation censuses no worse; screenshots looked at |
| **v2.78** | **Stage 3** as `html/README.md` defines it, plus F07; F04 and F05 (and F08) as its first commits if picked | L | shell, derived readouts | Q1, Q12–Q13 | PC-1, PC-4, PC-8 and PC-10 bars, plus §9's carry-overs |
| v2.79 on | Feature ports, in the owner's order | S–M each | features | Q12–Q14 | each port has a probe that fails on the version before; `hash_gen1` identical |

**Why stage 3 sits fifth (Q1).** Stage 3 builds new surfaces: the modules, a non-blocking planner window and the selection history. Three things it stands on are broken or about to change:
- **The tokens and contrast it inherits.** v2.74 fixes them.
- **The control metrics it lays out against.** v2.77's mono uppercase controls are about 1.3× wider (V24).
- **The live map it promises.** Stage 3 promises the map stays live while the planner is open, but one LOD zoom gesture blocks the page for 13–15 s today (P2).

Every stage-3 probe also generates worlds, and a default 2048 generate takes 68 s (P1).

v2.74 to v2.76 need no owner answer to start, so Q4–Q8 can be answered while they are built. v2.77 never blocks: it builds whatever has been answered, and whatever has not been answered stays as it is. Stage 3 builds only from tokens and shared classes (§9), so a later answer restyles it without rework.

If the owner wants stage 3 sooner, the minimum ahead of it is v2.74.

**Version numbers are provisional.**
- Each number is confirmed free in Cartalith_RC when its lane starts, and numbers shift with the owner's picks.
- Each version is one more hand move for the owner (`html/README.md`, "Moving it"). Versions can be moved together.
- v2.75 and v2.76 can be merged if fewer moves matter more. They are kept apart so that:
  - an engine regression is attributable to one file;
  - the engine speed-up does not wait on the owner's look at v2.76's side effects.

## 2. Rules for every version

**Files and lanes**
- Each version is one new file; old versions are never edited (`html/README.md`).
- Every lane edits the same file, so lanes run in series.

**Re-open each finding at its symbol in the then-current file** (MISTAKES "Schedule or act on a backlog row"). Drift from v2.72 to v2.73, measured with `grep -n "function NAME(" FILE` on both committed copies:

| Symbol | v2.72 | v2.73 |
|---|---|---|
| `buildRoutingSurface` | 6720 | 7228 |
| `computeFlow` | 6765 | 7273 |
| `updateReadout` | 12503 | 13011 |
| `_lodScheduleOverviewRebuild` | 14210 | 14718 |
| `_findRun` | 17431 | 17939 |
| `_applyDomainUI` | 17992 | 18507 |
| `_dskToolOptsSync` | 19516 | 20184 |
| `_v3dLoop` | 20067 | 21420 |
| `_civSyncCanvas` | 20891 | 22244 |
| `_civBakeKey` | 21911 | 23264 |
| `_civPaintTerritoryAt` | 21954 | 23307 |
| `_civRenderFactionList` | 22120 | 23473 |
| `_stUpdateVisible` | 22805 | 24158 |
| `_jpClaimedAt` | 24790 | 26143 |
| `_civLandTimeEdgeCost` | 27681 | 29037 |
| `_civHierarchicalNetwork` | 28345 | 29701 |
| `_civOpenPlacePopup` | 33964 | 35344 |
| `_dskPlace` | 19356 | gone (`_shellPlace`, 19978) |

**Probes.**
- Each fix carries a probe assertion that fails on the version before it (MISTAKES "Add a probe in the same commit that changes the behaviour").
- New probes, and any RC test changed on purpose, go in `html/tests/perf/` for the owner to move.

**Gates**, run from a Cartalith_RC clone as the audits ran them:
- `tests/run.sh`: 1280 passed, 0 failed.
- `tests/run_um.sh`: 882 passed, 0 failed.
- `hash_gen1.js <previous> <new>`: ALL IDENTICAL across all five configurations.
- `probe_desktopshell.js`: no failures beyond v2.73's four by-design ones.
- `probe_mobileshell.js`: 0 failed, at the assertion count v2.73 closes with.
- RC's shell probes (shellui, domainrail, cogmodal, assetsbtn, settleparams, geninfo, worldreset, typepreview): every difference classified.
- 0 page errors.

A change to `hash_gen1`'s output needs an owner ruling (MISTAKES "Change generated output").

**Timings**
- Report the median with [min..max], n ≥ 5 per version (n ≥ 4 for the LOD gesture).
- Use a fresh browser per sample, with versions interleaved ABBA.
- **Run alone.** The audits shared the machine (load average 0.6–5.7), so their absolute milliseconds are indicative only.
- An improvement counts only if the ranges are disjoint and an independent re-run's median lands inside the first bracket (MISTAKES "Quote a timing or a benchmark", "Replace a single-sample timing with a median").
- Where a deterministic counter exists, it is the gate and the timing confirms it.

**Visual checks**
- Open the before and after screenshots at 1366, 1600, 1920 and 2560 in both themes, and at 412×915 wherever a phone shares the surface (MISTAKES "Verify a visual change before committing").
- Colour assertions pin literal values, never token against token, and force the theme they were written for.

**Behaviour changes**
- Grep the touched code for the old behaviour's words (MISTAKES "Change behaviour").
- A value that can be absent is dashed with its reason, never shown as 0.

## 3. Step 0, before v2.74 (main loop)

**Keep the evidence.** It is all in this session's scratchpad, which does not outlive the session:
- audit-perf: 8.6 MB;
- audit-visual: 110 MB, including 257 shots;
- audit-features: 0.1 MB.

Copy into the repository, for example under `html/evidence/audit-v2.72-2026-11-01/`, about 26 MB in all:
- the scripts;
- the data and results files;
- the 28 sheets (7.8 MB);
- the 54 shots the findings cite (16.6 MB; all 54 exist).

Leave out the two HTML copies (they are in git at `7bd3b49` and in `reference/`) and the RC harness copy. Then re-point the references below.

**Start the CSS from `fixes.css`, with two corrections.** html/audits/v2.72-2026-11-01/visual/fixes.css holds every proposed rule as injected into v2.72.
1. **Its section comments use an earlier numbering for V19–V25.** Map each rule by its content, not its label:

   | `fixes.css` label | Finding |
   |---|---|
   | "V19" | V25's arrow gap |
   | "V20" | V19 |
   | "V21" | V20 |
   | "V22" | V21 |
   | "V23" | V22 |
   | "V25" | V23 |

2. **Its media queries are v2.72's.** The `@media (max-width:860px)` and `@media not all and (max-width:860px)` blocks must be re-keyed to v2.73's `html[data-shell=…]` selectors.

**Route the audits' report text** (never while a verifier runs):
- **The three `MISTAKES.md` candidates the audits wrote:**
  - read a cache key against the state literal (`_civBakeKey`'s `state.sun`);
  - `setTimeout(0)` is not the background (`_lodScheduleOverviewRebuild`);
  - restyling an active state by recolouring one border side (`.subtab.on`, `.set-nav button.on`).
- **DOC-1's `OUTSTANDING_WORK.md` text:**
  - the `_jpClaimedAt` finding, under "Found in the HTML";
  - a "Godot feature ports" row.
- **One router row per version below.**

`STATUS.md` changes when a version is built, not before. html/audits/v2.72-2026-11-01/features/v272_funcs.txt indexes v2.72's functions for the port lanes.

## 4. v2.73, stage 2: in progress and unchanged

No finding joins stage 2 by default. When stage 2 closes, re-measure the following on v2.73 before applying them:
- V10, V23, V17, the phone half of V03, and V27's phone header.
- V11: check for any new phone control drawn over the map.
- V272-1 and V272-2: they now live in `_shellPlace`. The crossings are desktop↔desktopN at 860, tablet↔tabletN at 1180, and every class change.
- P6: the pinch path is touchmove → `applyView` → `_civSyncCanvas`.

Optional, at the main loop's discretion: V10 and V23 are one CSS rule each, and both are phone-only failures in shared modals. V10 blocks the phone's only route to Credits; V23 draws faction names 0 px wide. Stage 2's lane may take them.

## 5. v2.74 — the fix pass

Every item here is hash-safe by construction: CSS, shell JavaScript and one cache key, with no pixel arithmetic.

### Correctness

- **P7 — the Sun slider does nothing.**
  - Cause: `_civBakeKey` reads `state.sun`, but the slider writes `state.sunAz` (21911–21921; state literal 3427; `bind('sun',…)` 17617).
  - Fix: read `state.sunAz`. Then change each input of the key once and confirm the key moves, checking every key part against the state literal.
  - Bars:
    - `sun_probe.js` passes 3 of 3 and is added to RC's `tests/perf`. It fails 1 of 3 on v2.71 and v2.72.
    - 0 key parts name a field the state literal lacks.
  - Side effect: once fixed, a Sun drag costs a full render per input, like the 36 other sliders, until P3 lands.
  - Evidence:
    - html/audits/v2.72-2026-11-01/perf/scripts/sun_probe.js
    - html/audits/v2.72-2026-11-01/perf/results/sun_v271.txt
    - html/audits/v2.72-2026-11-01/perf/results/sun_v272.txt
    - html/audits/v2.72-2026-11-01/perf/results/slider1024.jsonl
    - html/audits/v2.72-2026-11-01/perf/results/slider2048_v272.json (renderMs 0 for sun)

- **V01 — the rows of CIVIL ▸ Settlements run together into one string.**
  - Cause: `_stEnsurePool` writes `display:flex` into each row's cssText, and `_stUpdateVisible` then deletes it with `row.style.display=''` (v2.71:21734, v2.72:22816; still present in v2.73).
  - Fix: `#stSpacer > div{display:flex}`, or set `'flex'` in the JS.
  - Bars:
    - Every visible row computes `display:flex` (before: 0 of 3 sampled).
    - `#stTableHead`'s columns line up with the data columns within 1 px.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/e_measures.json (E1)
    - html/audits/v2.72-2026-11-01/visual/shots/b_64_dock_civ_settlements_s0.png
    - html/audits/v2.72-2026-11-01/visual/shots/e_E1_table_before.png
    - html/audits/v2.72-2026-11-01/visual/shots/e_E1_table_after.png
    - html/audits/v2.72-2026-11-01/visual/sheets/settle_table.png
    - html/audits/v2.72-2026-11-01/visual/sheets/before_after_1.png

- **V10 — Credits opens behind Settings.**
  - Cause: Credits sits at z-index 50 against Settings' 74, still 50 in v2.73. On a phone, the cog's About page is the only route to Credits.
  - Fix: `#creditsModal{z-index:76}`.
  - Bars:
    - After Settings ▸ About ▸ ⓘ, the element at screen centre is inside `#creditsModal` at 1600×900 and at 412×915, in both themes. The desktop case was measured fixed in both themes.
    - On the phone, a real tap on Credits' × closes it (it timed out before).
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/h_measures.json
    - html/audits/v2.72-2026-11-01/visual/data/j_after.json
    - html/audits/v2.72-2026-11-01/visual/shots/h_credits_from_settings_1600.png
    - html/audits/v2.72-2026-11-01/visual/shots/f_phone_credits.png
    - html/audits/v2.72-2026-11-01/visual/shots/j_l_credits_over_settings_fixed.png

- **V04 — the Assets / ← Map pin is a blank amber block.**
  - Cause: at rest, its accent text sits on an accent fill.
  - Fix: wash plus edge for `#assetsHeaderBtn.on,#settlementsHeaderBtn.on`.
  - Bar: both labels measure ≥ 4.5:1 at rest in both themes (before: 1:1).
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/e_measures.json (G1, G2)
    - html/audits/v2.72-2026-11-01/visual/data/g_v271_vs_v272.json
    - html/audits/v2.72-2026-11-01/visual/shots/e_G1_assets_pin_rest.png
    - html/audits/v2.72-2026-11-01/visual/shots/e_G2_assets_pin_hover.png
    - html/audits/v2.72-2026-11-01/visual/shots/b_51_slicer.png
    - html/audits/v2.72-2026-11-01/visual/shots/g_v271_assets_toggle_rest.png
    - html/audits/v2.72-2026-11-01/visual/shots/j_d_assets_fixed.png

- **V05 — in the light theme, the × of the Settlements screen, the route editor and the City viewer is invisible.**
  - Measured contrast: 1.08, 1.08 and 1.07:1.
  - Fix: the panel tokens; on hover, the accent edge and accent ink.
  - Bar: `.cfm-close`, `.re-close` and `.cv-close` measure ≥ 4.5:1 in light (16.67:1 measured after the fix).
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/l_contrast_factions.json
    - html/audits/v2.72-2026-11-01/visual/data/l_contrast_routeeditor.json
    - html/audits/v2.72-2026-11-01/visual/data/l_contrast_cityviewer.json
    - html/audits/v2.72-2026-11-01/visual/shots/l_30_factions_modal.png
    - html/audits/v2.72-2026-11-01/visual/shots/l_70_route_editor.png
    - html/audits/v2.72-2026-11-01/visual/shots/l_14_cityviewer_1600.png
    - html/audits/v2.72-2026-11-01/visual/shots/j_l_factions_fixed.png
    - html/audits/v2.72-2026-11-01/visual/sheets/light_modals.png

- **V06 — the Settlements screen and the route editor draw their content straight on a 90 % scrim.**
  - Fix: `.cfm-body` on `--panel` with side borders, and `.route-editor-modal` on `--bg`.
  - Bars:
    - The Settlements screen has 0 of 39 text runs below 4.5:1 in both themes (measured after the fix; 22 were below in light before).
    - The route editor's warning and title measure ≥ 4.5:1 in light (before: 2.73 and 3.43).
    - No map label shows through either window.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/j_after.json
    - html/audits/v2.72-2026-11-01/visual/sheets/settlements_tabs.png
    - html/audits/v2.72-2026-11-01/visual/sheets/light_factions_crop.png
    - html/audits/v2.72-2026-11-01/visual/shots/b_70_route_editor.png
    - html/audits/v2.72-2026-11-01/visual/shots/j_d_settlements_global_fixed.png
    - plus V05's two light contrast files

- **V23 — on a phone, the Settlements screen draws every faction name 0 px wide, and its overview reads "Largest by population: Aurelia (0)" before any settlement exists.**
  - Fix:
    - The rows wrap, and each name keeps 120 px.
    - The overview dashes the name and gives its reason: "— (no settlements yet)".
  - Bars:
    - At 412×915, 0 names are narrower than their text (before: 6 of 6 at 0 px, needing 42–99 px).
    - With 0 settlements, no zero-population name is shown.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/i_faction_rows.json
    - html/audits/v2.72-2026-11-01/visual/shots/i_faction_rows_phone412.png
    - html/audits/v2.72-2026-11-01/visual/sheets/phone_modals.png

- **V11 — the loading screen dims the map, but zoom, 3D and Base map stay live above it.**
  - Cause: `.busy` has no z-index, while the map corners sit at 20.
  - Fix: `.busy{z-index:30}` (Q10b).
  - Bars:
    - During a generate, the element at `#zoomIn`, at the 3D switch and at the Base map chip is the overlay: 3 of 3 (before: 0 of 3).
    - Menu panels (45) and the place popup (61) stay above the overlay.
  - V11 is a prerequisite of P1(C) and of stage 3's PC-7.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/h_measures.json
    - html/audits/v2.72-2026-11-01/visual/shots/h_busy_map_controls.png
    - html/audits/v2.72-2026-11-01/visual/shots/f_busy_overlay_1600.png

- **Already filed: Assets mode hides `#assetsPanel`.**
  - The panel holds the pack's name, author, licence and Validate button. It has been hidden since v2.24 by `body.assets-mode .dock,…{display:none}`.
  - It is filed in `OUTSTANDING_WORK.md` under "Found in the HTML by stage 1". C-1 below makes it the first step of any asset-pack tool.
  - Size the fix at the rule itself.
  - Bars:
    - In Assets mode, `#assetsPanel` is displayed inside the window, and Validate runs once on a real click.
    - The docks v2.24 meant to hide stay hidden.
  - Evidence: `OUTSTANDING_WORK.md`; `STATUS.md`'s stage-1 entry ("Found, from before v2.72").

- **V16, also already filed — at 1366×768 with the strip open, the place popup runs 33 px onto the strip.**
  - Cause: its `max-height:min(80vh,660px)` resolves to 614 px against a 587 px map.
  - Fix, desktop and tablet only: either add `calc(100vh - 232px)` to the min (the vh declaration first, then the dvh one), or set `el.style.maxHeight` from `_viewportRect()` in `_civOpenPlacePopup`.
  - Bars:
    - The popup's bottom edge is at or above the strip's top at 1366×768 with the strip expanded. Before, the popup was [593,83,873,697] against a strip top of 664.
    - The phone placement is unchanged.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/e_measures.json (N)
    - html/audits/v2.72-2026-11-01/visual/shots/b_91_civil_selected_1366.png

- **F06 — lands here only if Q2 is answered yes by then.** Otherwise it goes in the first version open after the answer. Its bars are in §10.

### Contrast and native controls

- **V02 — there is no `color-scheme` anywhere.**
  - Effect: on the dark UI, unchecked boxes are white, checked ones are Chrome blue, and slider tracks, scrollbars and select lists stay light. The dock alone holds 125 ranges and 43 checkboxes.
  - Fix: `:root{color-scheme:dark}`, `light` under the light theme, and `accent-color` on checkboxes and radios.
  - Bars:
    - `:root` computes `dark` in the dark theme and `light` in the light one.
    - Checked boxes paint `--accent`.
    - A person opens one select list and one number spinner in each theme; headless runs cannot capture native popups.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/dock_markup.html
    - html/audits/v2.72-2026-11-01/visual/shots/a_03_civil_populated_1600.png
    - html/audits/v2.72-2026-11-01/visual/shots/b_62_world_sculpt.png
    - html/audits/v2.72-2026-11-01/visual/shots/e_F_colorscheme_before.png
    - html/audits/v2.72-2026-11-01/visual/shots/e_F_colorscheme_after.png
    - html/audits/v2.72-2026-11-01/visual/sheets/dock_world_generate.png
    - html/audits/v2.72-2026-11-01/visual/sheets/before_after_colorscheme.png

- **V08 — `--faint` fails WCAG AA in both themes.**
  - In dark, 94 text runs fall below 4.5:1 across seven views: place view 26, Settlements 19, Asset Library 31, route editor 8, City viewer 9, Settings 1, Credits 0.
  - All of them are `--faint` except the route editor's '(land)', which is `#535759` at 2.59:1 and moves to `--dim`.
  - Fix: dark `#7d8285` and light `#676762`. Alternatively, `.hint{color:var(--dim)}`, the design's lowest ink.
  - Bars (computed):

    | Theme | on bg | on panel | on panel2 | on ins |
    |---|---|---|---|---|
    | dark | 4.97 | 4.79 | 4.54 | 4.86 |
    | light | 5.09 | 5.00 | 5.45 | 4.52 |

  - This is a shared-token re-base, so re-check every pair built on the old value (MISTAKES "Re-base a shared token"):
    - the faint–dim step (4.54 against 5.62 on panel2);
    - the default `.pill` text;
    - `option:disabled`;
    - faint on `--wash2`, which computes 3.63, so no text may use that pair.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/token_contrast.txt
    - html/audits/v2.72-2026-11-01/visual/data/b_contrast_place_1600.json
    - html/audits/v2.72-2026-11-01/visual/data/b_contrast_factions.json
    - html/audits/v2.72-2026-11-01/visual/data/b_contrast_assetlib.json
    - html/audits/v2.72-2026-11-01/visual/data/b_contrast_routeeditor.json
    - html/audits/v2.72-2026-11-01/visual/data/b_contrast_cityviewer.json
    - html/audits/v2.72-2026-11-01/visual/data/b_contrast_settings.json
    - html/audits/v2.72-2026-11-01/visual/data/b_contrast_credits.json
    - html/audits/v2.72-2026-11-01/visual/data/d_styles_1366.json
    - html/audits/v2.72-2026-11-01/visual/shots/b_11_place_selected_1600.png
    - html/audits/v2.72-2026-11-01/visual/shots/b_50_asset_library.png

- **V09 — the light accent fails AA as small text, and two hard-coded inks sit on the accent fill.**
  - Today's `#a4650f` measures 3.75–4.15. The two inks are `.pill.on` at 3.73 and `.layers-fab .lp-item.on` at 3.92.
  - Fix: set the token per Q3, and move both inks to `var(--accInk)` either way.
  - Bars, with `#8b550d`:
    - ≥ 4.5:1 on all six grounds (computed 4.52–5.91);
    - `--accInk` on it measures 5.91;
    - both inks ≥ 4.5.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/l_contrast_place_1600.json
    - html/audits/v2.72-2026-11-01/visual/data/l_contrast_routeeditor.json
    - html/audits/v2.72-2026-11-01/visual/shots/l_11_place_selected_1600.png
    - html/audits/v2.72-2026-11-01/visual/shots/j_l_place_selected_fixed.png
    - the token file above
    - the design's `#9a5d0c`, which is `themeVars` in `design/mobile-shell-2026-10-31/Shell.dc.html`

- **V12, first half: the light theme and the Name row.** The dark palette waits for Q6 (v2.77).
  - In light, the popup stays dark with white inputs and fails AA:

    | Text | Contrast |
    |---|---|
    | trait pills | 3.51 |
    | '0–1' and '(auto)' | 2.16 |
    | 'Fortifications (walls)' | 2.64 |
    | 'Trade hub' | 3.73 |

  - In both themes, the 🎲 button is clipped (clientWidth 278 against scrollWidth 289).
  - Fix:
    - the audit's re-token, scoped to `:root[data-theme="light"]`;
    - `.place-edit-popup .row input{min-width:0}`;
    - before relying on CSS alone, grep `_civPopulatePlaceEditor`'s template strings for inline colours.
  - Bars:
    - In light, 0 popup text runs are below 4.5:1.
    - In both themes, clientWidth equals scrollWidth (278/278 after the fix).
    - The roll button's right edge sits inside the inner edge (860 against 861 after; 883 before).
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/e_measures.json (I1)
    - html/audits/v2.72-2026-11-01/visual/data/s2_geom_1920.json
    - html/audits/v2.72-2026-11-01/visual/data/s2_geom_2560.json
    - html/audits/v2.72-2026-11-01/visual/shots/s2_11_place_selected_1600.png
    - html/audits/v2.72-2026-11-01/visual/shots/j_d_place_selected_fixed.png
    - html/audits/v2.72-2026-11-01/visual/sheets/popup_name_row.png
    - plus V09's light place-view files

- **V03 — active sub-tabs and the Settings page list draw a full accent box.**
  - Cause: the theme layer's `button.on` border fills the three sides that each `.on` rule does not recolour.
  - Fix: reset the border shorthand, then draw one 2 px side — the bottom for `.subtab.on`, the left for `.set-nav button.on`, and the bottom on the phone.
  - Bar: `getComputedStyle` on all four sides shows one side at 2 px and three at 0 (before: 1 px on all four).
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/e_measures.json (D1, D2)
    - html/audits/v2.72-2026-11-01/visual/data/g_v271_vs_v272.json
    - html/audits/v2.72-2026-11-01/visual/shots/e_D2_setnav_before.png
    - html/audits/v2.72-2026-11-01/visual/shots/e_D2_setnav_after.png
    - html/audits/v2.72-2026-11-01/visual/sheets/settings_pages.png
    - html/audits/v2.72-2026-11-01/visual/sheets/phone_modals.png

- **V19 — the design's focus ring exists only on the menu bar and rail.**
  - Fix: `:focus-visible{outline:2px solid var(--accent);outline-offset:2px}`. If it reads too loud on text fields, scope it to non-text controls.
  - Bars:
    - The six controls the audit reached by Tab compute a 2 px solid accent outline with a 2 px offset: `#genBtn`, `#reseedBtn`, `#centerBtn`, `#bakeAllDepth`, `#bakeAllBtn` and a summary. Before, they had Chrome's `auto 1px`.
    - Menu rows and the rail keep their inset ring.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/b_focus_seq.json
    - html/audits/v2.72-2026-11-01/visual/data/l_focus_seq.json
    - html/audits/v2.72-2026-11-01/visual/sheets/focus_seq.png

- **V20 — text fields use Arial 13.33 px with almost no padding.**
  - Fix: `font-family:inherit`, mono for numbers and selects, and `padding:5px 7px`.
  - Bars:
    - `input[type=text]` no longer computes Arial.
    - Numbers and selects compute `--mono`.
    - Padding is 5 px by 7 px.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/b_controls_civ.json
    - html/audits/v2.72-2026-11-01/visual/sheets/stats_inputs.png

- **V21 — the scale bar and legend backgrounds are too thin over bright terrain.**
  - Computed contrast today over white snow: 2.91:1 for the scale bar and 2.77:1 for the legend.
  - Fix, per Q9: `.85` and `.86` alpha boxes, with the legend text in `--muted`.
  - Bars (computed over two grounds, #fff and #d9c79c):
    - the scale bar measures ≥ 4.5:1 over both (8.82 over white);
    - the legend measures 6.01 and 6.68;
    - the light-theme overrides keep their values.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/e_measures.json (M)
    - html/audits/v2.72-2026-11-01/visual/sheets/legend_scalebar_1600.png

### Clipping, layout and copy

- **V07 — empty asset slots are 154 px white squares on the dark library.**
  - 18 of them fill the first screen. Cause: `UIState.data.previewBg` defaults to `#ffffff`.
  - Fix: draw empty slots as `:has(> .al-empty)` wells, keeping the user's preview colour for real art.
  - Bars:
    - 0 empty tiles compute white in dark at 1600×900.
    - A tile with art keeps `--preview-al`.
    - Browsers without `:has()` keep today's white, a graceful fallback.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/e_measures.json (G3)
    - html/audits/v2.72-2026-11-01/visual/shots/b_50_asset_library.png
    - html/audits/v2.72-2026-11-01/visual/shots/l_50_asset_library.png
    - html/audits/v2.72-2026-11-01/visual/shots/j_d_assets_fixed.png

- **V13 — in File ▸ Export, 'Export GeoJSON' looks like a line of text.**
  - Cause: v0.87's `.dropdown button` reset strips the button.
  - Bar: `#exportGeoBtn` computes a 1 px solid border on `--panel2` (before: transparent, '0px none').
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/e_measures.json (H)
    - html/audits/v2.72-2026-11-01/visual/shots/e_H_geojson_button.png
    - html/audits/v2.72-2026-11-01/visual/shots/b_21_menu_file_export.png
    - html/audits/v2.72-2026-11-01/visual/shots/j_d_file_export_fixed.png

- **V14 — the status readout cuts a chip mid-text.**
  - At 1600 it is 911 px against 989 needed; at 1366, 677 against 678.
  - Fix: let chips wrap onto the clipped second line, so a chip that does not fit disappears whole.
  - Bar: 0 chips partly visible at 1366, 1600, 1920 and 2560.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/e_measures.json (C)
    - html/audits/v2.72-2026-11-01/visual/sheets/statusbar_1600.png

- **V15 — arming Freehand hides 8 of the 13 sculpt features at 1600 and 10 at 1366, and the bar's hint is stale.**
  - Cause: `_dskToolOptsSync` centres the armed feature, and Freehand is the last one.
  - Fix: give the picker row one (the `order` rules), and reword Freehand's hint to name the modes beside it (see V28).
  - Bars:
    - With Freehand armed, 0 features are hidden at 1600 and at most 1 at 1366. After the fix, 1064 px and 830 px are visible.
    - The bar stays 75 px tall.
    - The idle bar's pins still end at 1590 and 1356.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/k_sculpt.json
    - html/audits/v2.72-2026-11-01/visual/shots/e_J_sculpt_bar.png
    - html/audits/v2.72-2026-11-01/visual/sheets/sculpt_bar_before_after.png
    - e_measures.json (J2)

- **V17 — the New World card clips 'Archipelago' and floats its sticky bar.**
  - 'Archipelago' needs 68 px in a 59 px cell, reads 'Archipela' on the phone, and every shape cell is forced onto two lines.
  - The sticky Back/Generate bar sits 25 px above the card's edge.
  - Fix: three columns for the shapes, and `bottom:-24px` for the bar.
  - Bars:
    - 0 clipped cells at 1366×768 and at 412×915.
    - The sticky gap is ≤ 1 px.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/shots/a_00_newworld.png
    - html/audits/v2.72-2026-11-01/visual/shots/e_A3_worldshape_before.png
    - html/audits/v2.72-2026-11-01/visual/shots/e_A3_worldshape_after.png
    - html/audits/v2.72-2026-11-01/visual/shots/e_A2_sticky_gap_after.png
    - html/audits/v2.72-2026-11-01/visual/sheets/light_start.png
    - e_measures.json (A1, A2)

- **V18 — at 1366, CIVIL's five sub-tabs overflow the dock.**
  - They need 387 px of the 371 available, and STATISTICS is clipped.
  - Bars:
    - The CIVIL dock's scrollWidth ≤ clientWidth at 1366 and 1600.
    - A person re-checks with Windows' Segoe UI, because the audit measured with Inter.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/s2_geom_1366.json
    - html/audits/v2.72-2026-11-01/visual/shots/b_91_civil_selected_1366.png
    - html/audits/v2.72-2026-11-01/visual/shots/s2_91_civil_selected_1366.png
    - e_measures.json (Q)
    - j_after.json (d_geom_1366)

- **V22 — the map chips are misaligned, and the Context panel's empty-state hint is cut at every size.**
  - The Base map chip (32 px) and the Explore filter (36 px) end 4 px apart.
  - The hint needs 397 px but gets 249–273, and it repeats the body's sentence.
  - Fix: one chip height, and either a shorter hint in `_dskCtxSync` or a two-line clamp.
  - Bars:
    - The two chips share a bottom edge.
    - The hint is shown whole at 1366–2560 and does not repeat the body.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/shots/e_L_map_chips.png
    - html/audits/v2.72-2026-11-01/visual/shots/b_11_place_selected_1600.png
    - e_measures.json (L, P)

- **V25, the arrow only.** The label rule is v2.77's.
  - Category summaries read '▸GEOLOGY', because `content:'▸  '` loses its spaces in a flex summary.
  - Fix: `margin-right:8px` on the `::before`.
  - Bar: ≥ 6 px between the arrow and the label on every category.
  - Evidence: html/audits/v2.72-2026-11-01/visual/sheets/settings_pages.png

- **V28 — copy that v2.72's moves made stale.** Three places:
  - the strip's empty line tells the user to open the strip while it is already open;
  - WORLD's tools line says there are no map tools while Sculpt is armed;
  - Freehand's hint says 'pick a sub-mode below'.
  - Bar: each old phrase grepped, with 0 copies left.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/shots/a_04_strip_expanded_noyears_1600.png
    - html/audits/v2.72-2026-11-01/visual/shots/b_62_world_sculpt.png

### Shell speed

- **P4 — every page load renders an empty 2048×1311 ocean behind the 86 %-opaque gate.**
  - The boot `renderNow` takes 1846 [1770..2149] ms in v2.71 and 1960 [1838..1999] ms in v2.72. It sits inside a load task of 2.13 / 2.31 s, so the gate's buttons are dead for about 2 s.
  - Fix (Q10a):
    - On the browser branch, fill `#view` with the ocean colour and render on 'Skip for now'. Generate, Load and Import render anyway.
    - Leave the headless branch untouched.
  - Bars:
    - 0 `renderNow` calls before the user leaves the gate (a counter).
    - The load long task has a median ≤ 0.6 s (estimate: about 0.3 s).
    - After 'Skip for now', the canvas hash equals v2.73's after Skip.
    - `run.sh` 1280/0.
  - Code: 20199–20202 (v2.73: 21555).
  - Evidence:
    - html/audits/v2.72-2026-11-01/perf/scripts/bootrender.js
    - html/audits/v2.72-2026-11-01/perf/results/bootrender.jsonl
    - html/audits/v2.72-2026-11-01/perf/shots/boot_v272.png

- **V272-2 — `_dskInit` adds about 34 ms of boot work, mostly one extra forced layout.**
  - `_syncChromeTop` takes 37 [32..40] ms in v2.72 against v2.71's 6 [6..10]. No load-time difference was established over three runs.
  - Fix: place the controls before the boot's first forced layout (that placement is now `_shellPlace`). Respect v2.72's TDZ rule and the `_dskReady` gate.
  - Bars:
    - Boot LayoutCount ≤ v2.73's − 1.
    - `_syncChromeTop` at boot ≤ 10 ms.
    - `probe_desktopshell.js` and a load at phone width pass.
  - Code: `_dskPlace` 19356–19373; the boot `_syncChromeTop();` at 19124.
  - Evidence:
    - html/audits/v2.72-2026-11-01/perf/scripts/profile_load2.js
    - html/audits/v2.72-2026-11-01/perf/results/profload.jsonl
    - html/audits/v2.72-2026-11-01/perf/results/load.jsonl
    - html/audits/v2.72-2026-11-01/perf/results/load2.jsonl

- **V272-1 — crossing the 860 px line costs about 11 ms more per resize.**
  - TaskDuration rose from 52.8 [46.8..57.6] ms to 64.2 [62.9..72.2] ms, and LayoutCount from 9.3 to 12.8 per resize.
  - Fix: drop `_syncChromeTop()` from the placement function, because the header ResizeObserver republishes it. Alternatively, run it inside the refit's rAF.
  - Bars:
    - On v2.73's mode crossings, LayoutCount per crossing ≤ v2.73's − 1.
    - No established TaskDuration regression.
    - `--canvas-top` still caps the Layers popover: `probe_desktopshell.js` and `probe_shellui.js` pass on both sides of each crossing.
  - Code: `_dskPlace` 19356–19373, `_syncChromeTop` 19092, the ResizeObserver at 19121–19122; in v2.73, `_shellPlace` 19978–20001.
  - Evidence:
    - html/audits/v2.72-2026-11-01/perf/scripts/shell.js
    - html/audits/v2.72-2026-11-01/perf/results/shell.jsonl
    - html/audits/v2.72-2026-11-01/perf/results/interact1024.jsonl

### The v2.74 gate

§2's gates apply, plus a contrast census: the visual audit's in-page WCAG script, run over the seven audited views in both themes at 1600×900 and at 1366×768.
- The bar is 0 enabled text runs below 4.5:1.
- Any failure the audit did not name is either fixed in this pass or listed with its element and ratio.
- If Q3 picks `#9a5d0c`, the accent runs on `--ins` and on the wash are the only permitted failures, listed by element.
- The census script is html/audits/v2.72-2026-11-01/visual/lib.js.

**Preflight rows for the brief:**
- Schedule or act on a backlog row
- Change behaviour
- Add a capability, or write a staleness/cache key
- Fix a stale cache or an invalidation bug
- Re-base a shared token / constant set
- Change a widget's ink
- Assert on pixels
- Assert a drawn colour
- Measure ink or a glyph in a capture
- Report a layout measurement
- Return, marshal or default a value that can be absent
- Dash a field with a reason
- Add a probe in the same commit that changes the behaviour
- Verify a visual change before committing
- Verify anything phone-shaped
- Quote a timing or a benchmark

## 6. v2.75 — generate speed (the engine)

Every item has the same gate:
- `hash_gen1` ALL IDENTICAL, including `--full`;
- `run.sh` 1280/0 and `run_um.sh` 882/0;
- timings run alone.

- **P1(A) — one content-keyed routing-surface cache.**
  - **The problem.** A default 2048 generate takes 68.4 [65.3..70.5] s in v2.71 and 67.7 [65.4..70.4] s in v2.72, with a 60.5 s longest task.
    - `buildRoutingSurface` runs 6 times, for 42.5 s of 69.1 s.
    - 3 of those 6 builds repeat the build before them byte for byte:
      - flow2 repeats flow1;
      - carve's `buildRiverNetwork` repeats carve's `computeFlow`;
      - the render's prologue repeats carve's final `computeFlow`.
  - **The cause.**
    - `computeFlow` bumps `_fieldGen` and then builds directly (6765, 6775).
    - `currentRoutingSurface` (6759–6764) keys on `_fieldGen` alone, so it always misses after a `computeFlow`.
    - Its key also omits sea level and world.
  - **Fix:** one cache, used by both, keyed on GW, GH, seaLevel, world and the field's bytes. The bytes are compared as Uint32Array views: about 3–5 ms and 11 MB at 2048.
  - **Before the fix:**
    - Enumerate every input of `buildRoutingSurface` and every writer of `field`, and change each one once. The writers are: generate, sculpt commit, undo and redo, the erosion and coast buttons (the feature audit counts 11 `pushUndo()` sites in v2.72), load, import and reset.
    - On v2.74, measure whether a render after a sea-slider move differs from a cache-cleared render at the same sea level (today the surface built for the old sea level is reused). If it does, the fix changes what a sea move draws, and the changelog says so. `hash_gen1` never moves the sea slider.
  - **Bars:**
    - Builds per generate go from 6 to 3 (a counter; seeds 12345 and 31337, at 512, 1024 and 2048).
    - Generate at 2048 has a median ≤ 52 s (estimate: about 47 s, −31 %).
    - The render that ends generate takes ≤ 6.5 s (before: 11.4–11.6 s; estimate: 4.7 s).
    - After a sea move, the render equals a cache-cleared render.
  - Evidence:
    - html/audits/v2.72-2026-11-01/perf/scripts/bench.js
    - html/audits/v2.72-2026-11-01/perf/scripts/prof_gen.js
    - html/audits/v2.72-2026-11-01/perf/scripts/routing_dupes.js
    - html/audits/v2.72-2026-11-01/perf/results/gen.jsonl
    - html/audits/v2.72-2026-11-01/perf/results/profgen2048_v272.json
    - html/audits/v2.72-2026-11-01/perf/results/routing_dupes.jsonl (0 mismatches in 12 pairs; the duplicates cost 466–813 ms of 1145–1520 ms per generate)
    - html/audits/v2.72-2026-11-01/perf/results/hash_gen1.txt
    - code: `buildRoutingSurface` 6720–6752 (v2.73: 7228)

- **P1(B) — seed the priority flood only with sub-sea cells that touch land or the map edge.**
  - Why: 95.7–98.1 % of sub-sea cells are interior ocean (41–59 % of all cells) and can close nothing when popped. `MinHeap.pop` alone takes 26.0 s of self time out of 69.1 s.
  - The audit argues the output cannot change, because the change only reorders equal-priority pops. That is an argument, not a proof.
  - Gate (must): `buildRoutingSurface`'s output is byte-identical, old against new, compared directly on:
    - ≥ 4 seeds × {region, world} × 3 sea levels × {512, 1024};
    - plus one 2048 world.
  - Go/no-go: the change lands only if both hold:
    - routing time per build at 2048 falls by ≥ 25 % with disjoint ranges (before: 6.7–7.8 s per build);
    - heap seeds are ≤ 5 % of sub-sea cells.

    Otherwise, revert it and record the measurement.
  - Expected (estimate): most of the ~21 s left after A.
  - Evidence: as for P1(A).

- **P1(C) — show progress instead of a frozen minute.**
  - Fix:
    - On the pool path, yield to a macrotask between the post-pool stages and set the busy label for each stage.
    - Refuse dock and key input while `_busyDepth>0`. V11 already covers the map controls.
  - Bars:
    - During a 2048 generate, the overlay paints ≥ 4 distinct stage labels (1 today).
    - A probe that presses Generate, a slider and a key mid-generate runs 0 handlers.
    - The no-pool headless path stays synchronous, honouring the v0.61/v0.135 contract that the suite calls `generate()` without awaiting it.
  - The longest task then becomes the longest stage: carveRivers, 30.9–32.5 s at 2048 before A and B. Report it; splitting carve is not part of this version.
  - Evidence: gen.jsonl (the stage medians) and bench.js, at the paths above.

- **P5 — auto-populating settlements blocks the page for 15–16 s.**
  - 85 % of that is one Dijkstra search per settlement, evaluating a Tobler edge cost on every relaxation.
  - Measured:
    - `_civIterativeAutoWorld(3)` at 1024 takes 15.3 [14.3..16.8] s in v2.71 and 15.9 [14.7..16.1] s in v2.72, making 43 places and 68 ways.
    - At 512, `_civHierarchicalNetwork` takes 14.3 of 17.3 s. The closure from `_civLandTimeEdgeCost` (27681; closure at 27685) takes 9.8 s, and `toblerKmh` 4.5 s of self time.
  - Pre-check at `_civHierarchicalNetwork` (28345; `res1` and `res2` at 28398 and 28427): the cost, delay and river inputs must not change between the searches of one build.
  - Fix: build an 8-direction edge-cost table once per cost grid (about 6 MB at the 384-px routing grid) and pass it to every `roadDijkstra` of that build.
  - Bars:
    - At 1024 with seed 12345, the median is ≤ 10 s (estimate: 6.5–8 s).
    - The FNV hash of `JSON(state.places, civWays)` is identical on ≥ 3 seeds × {region, world} × {512, 1024}.
    - `probe_roadconnect`, `probe_foodshed` and `probe_trade` pass.
  - Evidence:
    - html/audits/v2.72-2026-11-01/perf/scripts/civtime.js
    - html/audits/v2.72-2026-11-01/perf/results/interact1024.jsonl
    - bench.js

**Preflight rows for the brief:**
- Change generated output
- Add a capability, or write a staleness/cache key
- Fix a stale cache or an invalidation bug
- Assert that two runs produce identical output
- Quote a timing or a benchmark
- Replace a single-sample timing with a median
- Attribute a number to a cause
- Measure "before" with a mutant
- Write a golden-risk item into a brief

## 7. v2.76 — interaction speed

Every item has the same gate:
- the exact render stays byte-identical (`hash_gen1` ALL IDENTICAL);
- deterministic counters are the gate;
- the owner looks at Q10c–e before the version closes.

- **P6 — off-LOD wheel zoom reallocates and redraws the settlement canvas on every tick.**
  - Measured at 1024: 22.2–24.6 ms median and 64–68 ms maximum per tick; 18 reallocations per 24 ticks; a real-input p95 frame interval of 117 ms.
  - With the canvas held at its size, a tick costs 2.0 ms median and 2.8 ms p90.
  - Cause: `applyView` (18378–18394) calls `drawCivLayerAuto`, which calls `_civSyncCanvas` (20891–20900). That sizes the canvas to cssW × dpr × zoom and resets it whenever the size moves more than 2 px.
  - Fix:
    - During a gesture, hold the backing size and let CSS scale it.
    - Resize and redraw once when the zoom settles (about 150 ms), or quantise the sizing zoom to 1, √2, 2, 2√2 and 4.
    - Allow one civ redraw per frame for wheel and touch.
    - Keep direct `drawCivLayerAuto()` calls synchronous: `smoke_gen1.js` reads pixels right after them, near lines 263, 2652 and 2812.
    - Enumerate every writer of the view (wheel, touchmove, zoom buttons, keys, fit, refit, `_viewFill`) and make each end in one settle redraw.
  - Bars:
    - Handler median ≤ 3 ms and p90 ≤ 5 ms at 1024 with settlements.
    - ≤ 2 reallocations per 24-tick gesture when debounced, or ≤ 5 when quantised.
    - After settling, the civ canvas equals a direct draw at the same view.
    - The real-input p95 is reported (target ≤ 50 ms; not yet measured).
  - Evidence:
    - html/audits/v2.72-2026-11-01/perf/scripts/wheel_attr.js
    - html/audits/v2.72-2026-11-01/perf/results/wheelattr1024_v272.json
    - interact1024.jsonl and bench.js, at the paths above

- **P2 — an LOD zoom-in gesture blocks the main thread for 13–15 s.**
  - Measured over 40 ticks at 1024: 31–32 overview rebuilds of 300–650 ms each, and a p95 frame interval of 450 ms.
  - Cause:
    - The `setTimeout(0)` in `_lodScheduleOverviewRebuild` (14210–14232; closure at 14215) runs on the main thread right after every tick. That contradicts its own comment at 14197–14209, which says the rebuild happens "in the background".
    - `LOD_OV_STRETCH_STREAK_CAP=4` (14352) forces a synchronous rebuild on every 5th consecutive stretch.
  - Fix:
    - Rebuild about 150 ms after the last view change, and stretch the last overview in the meantime.
    - Replace the streak cap with a time-or-ratio cap.
    - Correct the comment.
    - Moving the overview into GENPOOL is later work.
  - Bars:
    - On `lod_ctrl.js`'s gesture with settle-refine off: ≤ 3 rebuilds and ≤ 3 s of main-thread time, n ≥ 4 ABBA. Before: 13.0 [12.6..13.4] s in v2.71 and 14.4 [13.9..14.8] s in v2.72.
    - LOD pixels at rest equal v2.75's at the same view.
    - `smoke_gen1.js`'s v0.93 hotfix tests (about lines 2362–2426) are updated on purpose, with the reason written beside them, and shipped in `html/tests/` for the owner to move.
    - The owner looks at the stretch. The owner reported 'blocky' LOD in v0.92 and v0.93.
  - Evidence:
    - html/audits/v2.72-2026-11-01/perf/scripts/prof_lodwheel.js
    - html/audits/v2.72-2026-11-01/perf/scripts/lod_ctrl.js
    - html/audits/v2.72-2026-11-01/perf/results/proflod1024_v272.json
    - html/audits/v2.72-2026-11-01/perf/results/lodctrl.jsonl
    - html/audits/v2.72-2026-11-01/perf/results/lodab.jsonl

- **P3 — dragging any look slider re-renders the whole map on every input.**
  - Measured per input at 2048:

    | Slider | ms per input |
    |---|---|
    | exag | 2971 |
    | bioBlend | 2633 |
    | aoR | 2998 |
    | sea | 4259 (longest task 5.8 s) |

    At 1024 each input takes 681–1043 ms, with no difference between versions.
  - Cause:
    - 36 of the 57 top-level `bind()` sliders call `render()` on input.
    - Each input changes `_civBakeKey` (21911), so the bake cache misses and a full `renderNow` (10730) runs, plus a 16 ms read-back.
    - The peak slider's 'input' handler (17589) also runs `recomputeClimate()` synchronously: 111 ms at 2048.
  - Fix:
    - On 'input', render a stride-2 preview (every second pixel, drawn scaled); on 'change', render exactly.
    - A preview never enters `_civBakeCache`, an export, the atlas or the LOD caches.
    - Move `recomputeClimate` into rAF or 'change'.
    - Splitting the pixel loop across workers is later work.
  - Bars:
    - At 2048, per-input main-thread time for exag, bioBlend and aoR is ≤ 40 % of v2.75's. A stride-2 preview computes 25 % of the pixels.
    - sea and peak are reported.
    - On 'change', the canvas hash equals a cache-cleared `renderNow`'s.
    - A finished drag adds exactly one cache entry.
    - The peak slider's synchronous handler takes ≤ 10 ms.
    - The owner looks at the preview.
  - Evidence:
    - html/audits/v2.72-2026-11-01/perf/scripts/slider.js
    - slider1024.jsonl and slider2048_v272.json, at the paths above

- **P9 — the territory brush rebuilds the whole territory raster on every pointermove.**
  - Measured per step: 6.5–7.7 ms at 1024 and 23 ms at 2048.
  - Code: `_civTerrGen` 21954–21965; `drawCivLayer` 21393–21430.
  - Fix: write only the brush's bounding box (a dirty rectangle).
  - Bars:
    - Pixels written per step ≤ the brush box's area (a counter).
    - After a 50-step stroke, the raster equals a full rebuild byte for byte.
    - Step time at 2048 is reported (target: a few ms; an estimate).
  - Evidence:
    - html/audits/v2.72-2026-11-01/perf/scripts/split_event.js
    - html/audits/v2.72-2026-11-01/perf/results/split1024_v271.json
    - html/audits/v2.72-2026-11-01/perf/results/split1024_v272.json
    - html/audits/v2.72-2026-11-01/perf/results/split2048_v272.json

- **P10 — the render tail, in four parts.** No leaks were found: idle cost was 0.3–0.8 ms in 3 s, and listeners did not grow over repeated use.
  1. **`updateReadout`** scans `tempField` and makes `maxGrade`'s pass on every full render: 4.3–4.8 ms at 1024 and 16.3 ms at 2048.
     - Fix: enumerate the writers of both fields, then cache both values on `_fieldGen`/`_climGen`.
     - Bar: ≤ 2 ms at 2048 when neither generation changed.
  2. **`CIV_BAKE_CACHE_MAX`** (21910) keeps 6 full-map ImageData whatever the grid size. Computed: 64 MB at 2048, about 258 MB at 4096, about 1 GB at 8192.
     - Fix: bound the cache by bytes (128 MB).
     - Bars: ≤ 128 MB with synthetic 4096 and 8192 entries; the hit rate at 2048 unchanged.
  3. **Find rebuilds its control index on every keystroke**: 13.1 / 14.2 ms (`_findRun` 17431).
     - Fix: build the index once per focus of the search box.
     - Bar: ≤ 3 ms per keystroke after the first.
     - If F03 is picked, this part moves with it.
  4. **`_v3dLoop` (20067) requests a frame every frame while 3D is on.**
     - Fix: stop when nothing is dirty; `_v3dKick` restarts it.
     - Bar: 0 frame requests over 3 s of idle (a counter). Real WebGL needs a person.
  - Evidence:
    - html/audits/v2.72-2026-11-01/perf/scripts/trace_idle.js
    - html/audits/v2.72-2026-11-01/perf/scripts/listeners.js
    - html/audits/v2.72-2026-11-01/perf/results/idle_v271.json
    - html/audits/v2.72-2026-11-01/perf/results/idle_v272.json
    - html/audits/v2.72-2026-11-01/perf/results/listeners_v271.json
    - html/audits/v2.72-2026-11-01/perf/results/listeners_v272.json
    - split_event.js and the split results, at the paths above

- **P8, only if Q11 is yes — every domain switch resets zoom and pan, and redraws the settlement layer twice.**
  - Measured: 46–49 ms per click.
  - Cause:
    - `_applyDomainUI` ends with an unconditional `_scheduleViewportRefit()` (18062).
    - That reaches `_refitViewportNow` (19110–19115), which calls `_viewFill()` (18313) even when `.canvas-wrap` did not change.
    - The ResizeObserver (19117–19123) already covers real size changes.
  - Fix: drop that call, or clamp with `_viewClampFill` when the box is unchanged, and correct the call's v2.24 comment.
  - Bars:
    - After zooming to about 2× and visiting every domain, scale and pan are unchanged when the box is unchanged. Before, each switch went back to the cover scale (1.151 in v2.71, 1.105 in v2.72).
    - One `drawCivLayer` per switch (before: 2).
    - A band that changes the box still refits once.
    - `probe_domainrail` and `probe_desktopshell` pass.
  - Evidence:
    - html/audits/v2.72-2026-11-01/perf/scripts/prof_domain.js
    - html/audits/v2.72-2026-11-01/perf/results/profdomain_v271.json
    - html/audits/v2.72-2026-11-01/perf/results/profdomain_v272.json
    - shell.jsonl, at the path above

- **V272-3 — no action.**
  - The LOD gesture measured about 10 % dearer in v2.72 only because the map is 30 px shorter: 684×762 against 684×792 at 1400×900. With v2.71's window shortened to match, the difference was not established.
  - It is not a regression, and P2 removes the per-tick cost.
  - Any version that changes the map's height, stage 3 included, re-runs `lod_ctrl.js`.
  - Evidence:
    - html/audits/v2.72-2026-11-01/perf/scripts/geom.js
    - html/audits/v2.72-2026-11-01/perf/results/lodctrl_geom.jsonl
    - html/audits/v2.72-2026-11-01/perf/results/geom.jsonl
    - lodctrl.jsonl, at the path above

**Preflight rows for the brief:**
- Fix a stale cache or an invalidation bug
- Change behaviour
- Assert on pixels
- Run a pixel probe (with a positive control that must move)
- Measure an effect inside a subset of pixels
- Quote a timing or a benchmark
- Replace a single-sample timing with a median
- Attribute a number to a cause

## 8. v2.77 — design alignment

This version builds from the answers to Q4–Q8; anything still unanswered stays as it is. V25, V29 and V30 follow an existing ruling or the drawing itself, so they need no answer.

- **V24 (Q5) — only 5 of the 96 dock controls match the design.**
  - The design's chip is mono 10 px and 30 px tall. Its on-state is a wash with a 1 px accent edge and accent text.
  - The dock instead has:
    - 26 segmented controls drawing a solid accent fill;
    - 8 action buttons in sans, sentence case;
    - 5 sub-tabs in sans 11 px;
    - a Bake button 224 px wide, where the design draws it full width.
  - Fix:
    - (a) washed segments;
    - (b) mono uppercase action buttons;
    - (c) full-width Bake and Un-finalize.
  - Bars:
    - All 39 (26 + 8 + 5) compute the design's type and on-state.
    - No solid accent fill remains outside whatever Q5 keeps.
    - The truncation census (`__aud.truncs` in html/audits/v2.72-2026-11-01/visual/lib.js) at 1366 and 1600 is no worse than v2.76's, with every row that wraps listed.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/b_controls_world.json
    - html/audits/v2.72-2026-11-01/visual/data/b_controls_civ.json
    - html/audits/v2.72-2026-11-01/visual/data/b_controls_carto.json
    - html/audits/v2.72-2026-11-01/visual/data/b_controls_explore.json
    - html/audits/v2.72-2026-11-01/visual/data/d_inventory_1600.json
    - html/audits/v2.72-2026-11-01/visual/shots/b_60_world_generate.png
    - html/audits/v2.72-2026-11-01/visual/shots/b_64_dock_civ_generation_s0.png
    - html/audits/v2.72-2026-11-01/visual/shots/d3_civil_selected.png
    - d_styles_1366.json and dock_markup.html, at the paths above
    - `LARGE_ITEM_RULINGS.md`, 2026-09-06, item 7 ("Resolved to the WASHED treatment")

- **V25 — the small uppercase section label comes in about eight variants.**
  - Fix:
    - One rule across the nine selectors the audit lists: mono 9.5 px, weight 500, .2em tracking, uppercase, `--dim`.
    - Keep the accent for the location labels (`.dh-path`, `.to-ctx`, `.ctx-k`, `.mb-cap`).
    - Help that runs past three lines may move into `details.adv`; that is a content change, not CSS.
  - Bar: the nine selectors compute one style.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/d_styles_1366.json
    - html/audits/v2.72-2026-11-01/visual/shots/a_03_civil_populated_1600.png
    - settings_pages.png, at the path above

- **V26 (Q4) — the corner radius.**
  - v2.72 keeps v2.23's `*{border-radius:0 !important}`, which overrides 67 radius declarations, and its D-block comment calls the mockup flat. The design draws 10–16 px corners.
  - Bars, if adopted:
    - Buttons, selects, inputs and textareas compute the control token, and pills compute 999 px.
    - `.seg` clips its children.
    - The D-block comment is corrected.
    - `.seg.grid` borders are re-checked.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/shots/d_frame_1600_light.png
    - `cartalith-native/godot-project/shell/dcc_theme.gd`, `ROLE["btn_radius"]` = [8, 12], and its comment

- **V27 (Q7) — colour emoji used as icons.**
  - Counts:
    - 12 of the 130 dock buttons carry emoji.
    - The audit counts 209 emoji code points in v2.72.
    - v2.73 adds phone ones: 198 → 221, counted by `python3 -I` over U+1F300–1FAFF and U+2600–27BF (a narrower set than the audit's).
  - Fix, staged:
    - First the pins: a hidden `.ico` span on the desktop, plus both JS writers (`_carEnterAssetsMode`, `_carExitAssetsMode`).
    - Then the dock's primary buttons.
  - Bars:
    - 0 emoji in the desktop pin row and the dock's primary buttons.
    - Every `textContent` writer of those buttons writes the span (grep).
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/data/a_emoji_civil_1600.json
    - html/audits/v2.72-2026-11-01/visual/shots/a_00_startcard.png
    - e_J_sculpt_bar.png and b_11_place_selected_1600.png, at the paths above

- **V29 — the timeline strip keeps v2.71's controls where the design draws chips and a round play button.**
  - This follows the 2026-09-06 ruling that the timeline's pill on-state matches the WORLD dock's.
  - Bars:
    - The five toggles render as washed chips.
    - Each checkbox still takes Tab, shows the ring and keeps its label.
    - A person checks that a screen reader still announces a checkbox.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/shots/d3_timeline_expanded.png
    - html/audits/v2.72-2026-11-01/visual/shots/a_05_strip_expanded_twoyears_1600.png

- **V30 — window headings come in six styles.**
  - Fix: one rule (mono 12 px, weight 500, .2em tracking, uppercase, accent). The City viewer's place name is exempt, as a proper noun.
  - Records ▸ Factions… opens a window titled 'Settlements'. Put the two names to the owner when v2.77 is reviewed; the default is no change.
  - Bar: the five headings compute one style.
  - Evidence:
    - html/audits/v2.72-2026-11-01/visual/shots/b_30_factions_modal.png
    - html/audits/v2.72-2026-11-01/visual/shots/b_41_credits.png
    - b_70_route_editor.png, at the path above

- **V31 (Q8) — type faces and status-bar density.**
  - The design loads IBM Plex Mono with Helvetica Neue.
  - The file's font stacks resolve to DejaVu Sans Mono and Inter on the audit machine, and to Cascadia or Consolas with Segoe UI on Windows. Mono widths vary about 9 % at .2em tracking.
  - The design's status bar holds 3 items; v2.72's holds about 15 chips.
  - Evidence: statusbar_1600.png, at the path above; `design/mobile-shell-2026-10-31/Desktop.dc.html`.

- **V12, second half: the dark theme (Q6).**
  - Bars:
    - 0 popup text runs below 4.5:1 in both themes.
    - No hard-coded colour left in the `.place-edit-popup` and `.settle-info` rules (grep).
  - Evidence: as for V12 in §5.

- **V21's halo trial (Q9).** Adopt the design's boxless text with a halo only if it measures ≥ 4.5:1 over snow, sand, forest and sea, sampled under the text.

- **The 1100-px desktop window.** It leaves the map 384 px wide in v2.71 and v2.72, a layout point the performance audit recorded. Re-measure it on v2.73, whose docking rule may already answer it.
  - Evidence: geom.js and geom.jsonl, at the paths above.

**The v2.77 gate:**
- §2's gates;
- the contrast census, as in v2.74;
- a truncation census no worse than v2.76's;
- screenshots at the four desktop sizes in both themes, looked at;
- any "matches the design" claim quotes every departure (MISTAKES "Write 'it matches the design'").

**Preflight rows for the brief:**
- Re-base a shared token / constant set
- Change a widget's ink
- Assert a drawn colour
- Write "it matches the design"
- Carry a rule from one variant of a design to another
- Verify a visual change before committing
- Rule on a plurality

## 9. v2.78 — stage 3

Stage 3 is as `html/README.md` defines it:
- the Context panel's modules;
- the selection history (back, forward, pin);
- the journey planner in its own window.

It builds `PC_SHELL_PROPOSAL.md` §3.3 and §3.7 (milestones PC-1, PC-4, PC-8 and PC-10). Explore's journeys are rehomed, and Explore stops being a domain.

**F07 is part of it.**
- 'Plan journey from here' and 'Journey to here' become two rows of `_civCtxItems` (34559 in v2.73, which has no journey verb today).
- The rows call `_civSetTool('route')`, `_civWaypoints`, `_civCommitRoute()` and `_civOpenRouteEditor(idx)`. There is no second route builder.
- A disabled 'Journey to here' says why.
- The journey naming mismatch (PC §3.7, finding M2) is settled in the same change.

**F04, F05 and optionally F08 come first, if picked (Q12).** Built as stage 3's first commits, they let the History, the Faith, culture & relations, and the Military modules re-host builders that already exist: one builder, two hosts (PC §3.3).

**Carry-over bars:**
- **PC-1, PC-4, PC-8 and PC-10's bars, adapted to the HTML:**
  - one selection writer, checked by grep;
  - a place selected three ways is named the same everywhere;
  - with the planner window open, the map draws and takes a real click;
  - the busy overlay mutes the window during a generate. V11 and P1(C) are its prerequisites.
- 0 colour literals in the added CSS outside the token blocks (grep the diff).
- The contrast census over the new surfaces: 0 failures in both themes.
- Every new control is reachable by Tab and shows v2.74's ring.
- `lod_ctrl.js` is re-run if the map's box changes (V272-3).

**Preflight rows for the brief:**
- Re-frame or redesign a window, a form factor's gestures, or a whole shell
- Convert a replacing UI context into an appended one
- Add a second route to a control the user already has one to
- Emit a change signal
- Delete a surface
- Answer "can the user do X?"

## 10. v2.79 onward — feature ports, in the owner's order (Q12–Q13)

Every port follows the same rules:
- it carries a probe that fails on the version before;
- `hash_gen1` stays identical;
- it is a pointer to the existing handler, never a second implementation;
- every labelled setting gets a `_findPanelOwners` owner;
- arming a tool calls `_dskToolOptsSync()`.

Sizes are the feature audit's estimates. All paths below are repository-relative unless shown in full.

| Port | Size | Recommended slot | Bars when built | Evidence |
|---|---|---|---|---|
| F06: `_jpClaimedAt` (a defect; Q2) | one character | v2.74, if answered | false on every sea cell and on unclaimed land: 0 of 12,258 and 10,939 sea cells, and 0 of 152 and 7,235 unclaimed land cells, with `> 0` swapped in; all 6 of seed 777's journeys gain one frontier toll; the planner text about the claimed floor is updated | html/audits/v2.72-2026-11-01/features/probe_audit.js; html/audits/v2.72-2026-11-01/features/probe_v272.json; `crates/cartalith-civ/src/lib.rs::jp_claimed_at`; v2.73 line 26143 |
| F04: Faction relations | S, 150–200 lines | v2.78, first commits | one symmetric value per faction pair, with each term shown; faction 0 is never a party; labelled as the port's judgement; cached on `_civAggGen`/`_civTerrGen` | `crates/cartalith-civ/src/relations.rs::civ_faction_relations` (CV-26); `shell/faction_roster_window.gd::_build_tab_relations`; v2.73's placeholder 'Diplomatic relations — not yet implemented' |
| F05: Political history | S, 80–150 lines | v2.78, first commits | runs of one faction collapse into periods, and a missing year ends a period (tested on a fixture with a gap); every year link goes through `civGotoYear` (grep shows no second year writer); a place without a `tid` is dashed with its reason | `crates/cartalith-civ/src/timeline.rs::civ_settlement_ownership_periods`; `shell/place_editor_window.gd::_build_political`; v2.72's `civSnapshotSave`, `_civAssignTid`, `civGotoYear` |
| F08: Garrisons | M, 120–180 lines | v2.78, first commits, if picked | the split sums to the HTML's own standing army; a place with no reading is dashed with its reason; it is never presented as Godot's totals | `crates/cartalith-civ/src/garrison.rs::civ_garrisons` (Rulings AW, AZ); v2.73 `_civFactionMilitaryHtml`, 'Not modelled: per-settlement garrisons' |
| F07: Map-card journey verbs | S, 30–50 lines | inside v2.78 | as in §9 | STATUS SM-4; `civilization_workspace.gd::_story_ctx_rows`; v2.73 `_civCtxItems` |
| F02, slice 1: Export map image | S, 40–80 lines | v2.79, with F03 | File ▸ Export ▸ Export map image… downloads the PNG `bakeSingle` makes at `#bakeRes`, with the same decoded pixels as `map.png` inside `exportZip` at the same setting; the busy overlay covers the bake; the row says what a finalized world exports (`exportZip` skips the flat bake today) | `shell/export_dialog.gd`; v2.73 `bakeSingle` 16149, whose only caller is `exportZip` at 16685; `CHANGELOG_v2.72.md`, 'Not in this build' |
| F03: Find runs menu commands | S, 40–80 lines | v2.79 | every `.mb-row[data-cmd]` row (42 in v2.72) is found by its own label; before, 0 of the 7 command names the audit tried were found; a hit runs `_dskRun(cmd)`; a disabled row says why; check and radio state is read at render time | `probe_audit.js` and `probe_v272.json`, at the paths above; `shell/command_index.gd` |
| F01: Measure | M, 500–800 lines | v2.80 | six modes (distance, bearing, area, radius, cross-section, Δ vertical); readings saved in the port's `annotations/measurements.json` schema; a reading with no single number omits its value and unit; the world-wrap seam handled; a placing tool in the touch layer (`_tchPlaceTool()`); a `_SHELL_SLOTS` row; the context card's Measure rows; touch on glass checked by a person | `shell/global_tools.gd::MEASURE_MODES`; `crates/cartalith-spatial/src/measure.rs`; `crates/cartalith-godot/src/measure_bridge.rs`; `shell/section_strip.gd`; v2.73 has 9 `data-civtool` values and no measure tool |
| F02, slice 2: overlays and presets | M | after F01 | per-layer toggles over `state.mapFilter`; Player and DM presets; above a browser's canvas limit, the tiles path (`#bakeTiles`) is offered, never a silent failure; the civ zoom tier used at export scale is stated | `shell/export_overlay.gd`; `_v3dRenderCivOffscreen`; `_civSyncCanvas`'s 4096 cap |
| F10: Undo history | M, 120–250 lines | after F02 slice 2 | labelled height rows revert; Recorded rows say why they cannot; `clearUndoHistory()` stays the floor on generate and load | `crates/cartalith-godot/src/undo.rs::HistoryLedger`; v2.73 `MAX_UNDO=5` (12734) |
| F09: Session mode | S–M, 80–150 lines | after F10 | WORLD is hidden on every v2.73 surface (the rail, the domain tab, `#phFan`, the swipe, `#mbWorldWrap`); generate and the resets are refused with a reason; it is refused while `_busyDepth>0` or a sculpt draft is open; it is not persisted | `menus.gd` `ID_WIN_SESSION_MODE`; `dcc_shell.gd::set_session_gate` |
| F12: More style presets | S each | any | Default stays bit-identical (`hash_gen1`); each recipe is re-tuned by eye on two seeds, and the screenshots are looked at | `render_workspace.gd::STYLE_PRESETS`; v2.73 `STYLE_PRESETS` (18225) has 6 entries |
| F14: Keyboard reference | S, 60–120 lines | any | lists only keys the page honours; the list is kept beside the handlers or tested | `shell/shortcuts_dialog.gd`; v2.72 `_DSK_KEYS` (5 rows) and `LAYER_HOTKEYS` |
| F15: Follow system theme; nautical miles | S | any | the third unit value in all six helpers and all three unit segments; expected numbers rebuilt from the converter (MISTAKES "Assert that a converted value CHANGED") | `dcc_settings.gd::THEME_MODES`; `dcc_units.gd`; v2.73 `_setUnits` (19268) accepts km and mi only; no `prefers-color-scheme` |
| F11: Reset generation parameters | M, 80–200 lines | later | defaults are captured at boot, before any load; they are applied through `parseGenerationInfo`'s confirm-and-regenerate path, seed excluded; it respects `[data-genlock]` and Session mode; the menu row says it regenerates | `menus.gd::_build_reset_generation`; v2.72 `#genInfoApplyBtn` |
| F13: Customisable pins | M, 200–300 lines | after F09 | a pin is a `_DSK_CMDS` key that runs `_dskRun`; storage is localStorage behind try/catch; a pin whose command is gone stays greyed with its reason; it has three homes after v2.73 | `shell/quick_bar.gd`; v2.73's four fixed pins |

**REC-1 is the basis.** The feature audit recommends F02, then F03, F04, F05 and F01, with F06 asked now and F07 inside stage 3. This plan moves F02 and F03 after stage 3, because they are uncoupled and the owner's request puts visuals and performance first. If the owner wants them sooner, F02 slice 1 and F03 can be their own small version before stage 3.

## 11. Not planned

- **NP-1: the LLM assistant.**
  - A localStorage value written by a file:// page in one folder was read by a file:// page in another (Chromium 153). A stored key would therefore be readable by every local HTML file in that browser profile. Godot keeps keys in the OS key store.
  - If ever wanted, it is a new design with a session-only key, not a port.
  - Evidence: html/audits/v2.72-2026-11-01/features/ls_probe.js.
- **NP-2: the Markdown vault.**
  - A file:// page has no lasting cross-browser folder access.
  - v2.72 already shows a native save's vault links read-only (`_vaultLinksHtml`, `_vaultSummaryHtml`) and keeps the per-place note `p.history`.
  - A "notes lite" is a design decision for stage 3's Note tab, in a format Godot can import.
- **NP-3: generation and terrain features that move generated output.**
  - Causal landmarks, VA-*, lava flows, GF-*, orogen relief, LOD detail and EF, GPU, multithreading and memory work.
  - Each needs its own ruling, and would ship default-off in the `state.passes` / `state.hydro` convention.
- **NP-4: the manpower recalibration and governments G1.**
  - The recalibration is Rulings AI(b), AI(c), DE and DL; governments G1 is Ruling CZ.
  - Each moves every manpower readout. RC's `probe_manpower.js` pins the specification's examples.
- **NP-5: browser-owned or browser-impossible items.**
  - Recent worlds, storage locations, Save as, Revert and Close project.
  - Rebindable shortcuts, haptics and system Back.
  - Multi-GPU, VRAM, quality tiers, Display P3, the colour grade and per-layer adjustment stacks.
- **NP-6: not ports at all.**
  - Bookmarks, the selection history, the Context modules and the planner window have no Godot implementation yet; they exist only in `cartalith-native/drafts/pc-shell/`.
  - They are stage 3, built from PC §3.1, §3.3 and §3.7.
- **NP-7: pop-out windows, the New World seed preview, and the three render items RC declined in v2.13.**
  - The declined render items are the flow-wetness halo, local contrast and the neatline.
  - Revisit the neatline only inside F02's overlay export.
- **C-1: considered, not shortlisted; revisit after the first ports.**
  - Help ▸ Save diagnostic report (`diagnostic_report.gd`).
  - Coach marks for v2.73's hold and grip (`app.gd::_maybe_show_coach_marks`).
  - Trade flows drawn on the map (IN-13).
  - An editable elevation ramp (`render.rs::ElevationRamp`), keeping `hypso()` by control flow.
  - Colour paint (CP-1…CP-7), reaching all four `landColorCore` call sites.
  - Look history.
  - GeoJSON import.
  - The travel library.
  - Conflicts and campaigns (SP-4 / MM-7), the strongest of these for storytellers, after stage 3.
  - Journey progression by day (SP-2).
  - Asset-pack tools AP-1…AP-10, after v2.74's `#assetsPanel` fix.
  - The reorderable layer stack.
- **DOC-1** is routed in step 0.

## 12. What needs a person

- **v2.74:**
  - one select list and one number spinner in each theme (V02);
  - Segoe UI widths on Windows (V14, V18);
  - Credits on a real phone (V10).
- **v2.76:**
  - the transient stretch, the slider preview and the soft labels (Q10c–e);
  - pinch on glass (P6's path);
  - the 3D idle loop on real WebGL (P10, part 4).
- **v2.77:**
  - a screen reader over the strip's chips (V29);
  - the halo over real terrain (V21).
- **Stage 3 and the ports:**
  - Measure on glass (F01);
  - large exports on phones (F02);
  - Session mode and the pins on v2.73's surfaces (F09, F13).
- **Every version:** none of the audits ran Firefox or Safari, and timings from the shared machine are indicative only.

## OWNER QUESTIONS
- Q1 — Where should stage 3 sit? Stage 3 is the Context panel's modules, the selection history, and the journey planner in its own window.

Recommended: fifth, as v2.78, after four fix versions: v2.74 (the fix pass), v2.75 (generate speed), v2.76 (zoom, slider and brush speed) and v2.77 (design alignment). Three reasons:
- Its new modules and its non-blocking planner window would otherwise be built on tokens that fail WCAG AA (94 dark-theme text runs below 4.5:1 across seven views), and on control sizes that v2.77 changes (the design's mono uppercase controls are about 1.3x wider).
- Its promise that the map stays live while the planner is open does not hold while one LOD zoom gesture blocks the page for 13-15 s.
- Each of its probes waits on a 68 s generate at the default 2048 grid.

v2.74 to v2.76 need no other answer from you, so Q4-Q8 can be answered while they are built.

Alternative: stage 3 next, as v2.74, with these fixes after it; its new surfaces would then be re-measured and restyled when v2.77 lands.

Default if unanswered: v2.74 starts as the fix pass once stage 2 closes.
- Q2 — Journey planner: should the HTML use the port's `> 0` in `_jpClaimedAt`?

Today it returns `civTerritory[...] >= 0` on a Uint8Array, so once territory exists every cell reads as claimed, open sea included. I re-opened this in v2.73 at dfc38b2, line 26143. Godot's `jp_claimed_at` uses `> 0`.

The feature audit measured the effect on v2.72 (256 px, seeds 12345 and 777, after Auto-populate and Auto polity):
- All 12,258 and 10,939 sea cells read as claimed.
- Tolls are 0 on all 7 priced journeys.
- With `> 0`, all 6 of seed 777's journeys gain one frontier toll (6 day-wages).
- Stages move from Sparse Settlements to Ruined Region on 12 of 77 (seed 12345) and 41 of 84 (seed 777).

Please rule on two side effects at the same time:
- The territory raster covers land only, so coastal, bay and river stages over water lose the claimed-land floor.
- Unclaimed wilderness is labelled 'Ruined Region', in Godot too.

Planner day counts change wherever a tier changes; world generation does not change.

Recommended: yes, in the first version open after your answer, with a probe that fails on the version before it.

Default: unchanged until you answer.
- Q3 — Light-theme accent colour: #8b550d, or the design's #9a5d0c?
- #8b550d passes WCAG AA as small text on all six background colours (computed 4.52-5.91:1).
- #9a5d0c (themeVars in design/mobile-shell-2026-10-31/Shell.dc.html) fails on the inset background (4.23) and the wash (4.25).
- Today's #a4650f measures 3.75-4.15, and it fails as the 9-11 px captions, tabs and links it colours.

Recommended: #8b550d.

Default: #8b550d in v2.74, since AA is the floor. The text colour on the 'Trade hub' pill and on the Layers rows moves to --accInk either way.
- Q4 — Corner radius: should the HTML get rounded corners?

The HTML forces `*{border-radius:0 !important}`. That rule dates from v2.23, is still in v2.73, and overrides 67 radius declarations; its comment calls the mockup flat. But:
- The newer canvas (design/mobile-shell-2026-10-31/Desktop.dc.html) draws 10-16 px corners.
- The port's DccTheme already moved off radius 0, under your 2026-08-25 rule that the newer canvas wins: `btn_radius` is 8 px with a pointer and 12 px on touch, and pills are 999 px.

Options:
- The port's values. Recommended, because the two apps then agree.
- The canvas's own per-control radii.
- Keep 0.

Default: keep 0 until you answer, because it reverses a stated v2.23 rule.
- Q5 — Dock controls: apply the washed on-state to the HTML's dock?

Two existing decisions bear on this:
- Your 2026-09-06 ruling for the port, item 7: the timeline's pill on-state matches the WORLD dock's, 'resolved to the WASHED treatment' (a wash, an accent edge and accent text).
- The DS-02 pass removed every filled amber block except the tool bar's SCULPT / PAINT / MEASURE segments.

Both match the design the HTML follows, which draws its chip the same way. In the HTML's dock today, 26 segmented controls still draw a solid accent fill, and only 5 of the 96 dock controls match the design.

The question: apply the washed on-state to those 26 controls, and give the 8 action buttons and 5 sub-tabs the design's mono uppercase type? Should a solid fill stay anywhere?

Recommended: yes, keeping a fill only for tool-mode segments, as the port does.

Default: built that way in v2.77, since it follows the existing ruling.
- Q6 — Place popup in the dark theme: keep its warm 'parchment' palette from v0.90, or give it the theme's colours like every other window?

The parchment palette is #e8dcc4 text on rgba(14,17,23,.97). It is the only parchment surface in the app, and at .97 alpha map labels show faintly through it.

The light theme gets the theme's colours in v2.74 either way: there the card stays dark with white inputs, and it fails AA (2.16-3.73:1).

Recommended: the theme's colours in both themes.

Default: the dark palette stays unchanged until you answer.
- Q7 — Icons: replace colour emoji with monochrome line icons drawn in the text colour, as the design does?

Where emoji are used today:
- 12 of the 130 dock buttons.
- The four pins mix 'SAVE' and 'PLANNER' with '🏘 SETTLEMENTS' and '🎨 ASSETS'.
- The sculpt picker is all emoji.

Recommended: yes, staged in v2.77. The pins and the dock's primary buttons (Bake, Un-finalize) go first; the sculpt picker and the start card follow once icons exist for them.

Default: no change until you answer.
- Q8 — Fonts and the status bar. Two choices:

(a) Fonts. Either embed a subset of IBM Plex Mono (about 30-60 KB as base64; OFL licence, to be confirmed) so text widths and truncation points match on every OS, or keep the system fonts. On Windows those are Cascadia or Consolas and Segoe UI, and mono widths vary about 9% at .2em tracking.

(b) The status bar's ~15 generation-statistics chips. Either keep them (v2.74 makes a chip disappear whole instead of being cut mid-word), or move them into Generation info to match the design's three-item status bar.

Recommended: (a) keep the system fonts for now; (b) keep the chips for now, since v2.74 makes them safe. Moving them is your call on density.

Default: both unchanged.
- Q9 — Scale bar and legend over bright terrain: a darker box, or the design's boxless text with a halo?

The box would use .85 and .86 alpha. Computed contrast over white snow: 8.82:1 for the scale bar and 6.01:1 for the legend, up from 2.91 and 2.77 today.

Recommended: the box in v2.74, then a halo trial in v2.77, adopted only if it measures at least 4.5:1 over snow, sand, forest and sea.

Default: the box.
- Q10 — Do you accept these visible side effects of the speed fixes? Each is shown to you on a build before its version closes.
- (a) v2.74: a flat ocean-coloured backdrop behind the start card, instead of the faint rendered ocean. This saves about 1.9 s on every page load.
- (b) v2.74: the loading overlay takes every input during a generate, zoom, 3D and Base map included, as the Godot shell's does.
- (c) v2.76: during an LOD zoom gesture, the last overview is stretched (a brief blur) until about 150 ms after the gesture stops. This replaces 31-32 rebuilds that block the page for 13-15 s. You reported 'blocky' LOD in v0.92 and v0.93.
- (d) v2.76: while a look slider is being dragged, a half-resolution preview, exact on release.
- (e) v2.76: settlement labels and pins look softer while zooming, until the zoom settles.

Recommended: accept all five.

Default: build them and show you.
- Q11 — Keep the map's zoom and pan when switching between World, Civil, Carto and Explore?

Today every switch returns to the cover view. `_applyDomainUI` ends with an unconditional `_scheduleViewportRefit()`, added in v2.24 for the conditional rail and still there in v2.73, which runs even when the map's box did not change.

Recommended: keep the view. The ResizeObserver still refits when a band changes the box, and each switch saves one redraw of the settlement layer.

Default: unchanged until you answer. It is built in v2.76, or in v2.74 if you answer in time.
- Q12 — Which Godot features should the HTML gain, and when? None of them moves generated output.

Recommended:

(1) With stage 3 (v2.78), as its first work:
- F04 Faction relations and F05 Political history. They fill the placeholder 'Diplomatic relations — not yet implemented' and the missing per-place history, so stage 3's modules re-host real builders.
- F07's two map-card verbs ('Plan journey from here', 'Journey to here'), inside stage 3's planner work.

(2) After stage 3:
- v2.79: F02 Export map image, slice 1 (the terrain PNG the project zip already bakes), together with F03 Find runs menu commands.
- v2.80: F01 Measure, with six modes, saved in the port's annotations/measurements.json format.
- Later: F02 slice 2 (overlays with Player and DM presets).
- Then your pick of: F10 Undo history, F09 Session mode, F12 more style presets, F14 Keyboard reference, F15 Follow-system theme and nautical miles, F11 Reset generation parameters, F13 customisable pins.

F02 slice 1 and F03 are small (about 40-80 lines each); if you want them sooner, they could be their own version before stage 3.

Default: none is built until you pick.
- Q13 — Only if the feature is picked:

(a) F04 would use the port's CV-26 weights: culture +0.30, religion ±0.20, trade complement +0.25, and border friction −0.55 × shared border × (0.35 + 0.65·rivalry), with stances at ±0.15 and ±0.45. These are the port's labelled judgements, not reference values. Port that base, without FH-R2's later religion extension?

(b) F08, per-settlement garrisons, would split the HTML's own standing army using the port's placement rule (Rulings AW and AZ). The HTML's army still uses the flat soldier upkeep of 3.0 that the port replaced, so its totals will not match Godot's. Build it now?

(c) F10: stage 2's long-press card adds a Delete with no confirm and no undo. Give deleting a place a real undo? That goes beyond the port, which records civil edits as not reversible, except ways.

(d) F15: with nautical miles, show altitudes in metres or in feet?

Recommended: (a) yes; (b) not until the manpower model is recalibrated, unless you want per-town numbers now; (c) yes; (d) metres.

Default: as recommended.
- Q14 — Confirm the do-not-port list:
- The LLM assistant. A key stored by a file:// page is readable by every local HTML file in that browser profile; measured in Chromium 153.
- The Markdown vault. A file:// page has no lasting folder access across browsers. A 'notes lite' would be a separate stage-3 design, in a format Godot can import.
- Generation and terrain features that change generated output. Each needs its own ruling and an off-by-default switch.
- The manpower recalibration and governments G1.
- Items the browser owns or makes impossible: recent worlds, Save as, rebindable keys, haptics, multi-GPU, the colour pipeline.
- Pop-out windows.
- The New World seed preview.
- The three render items RC declined in v2.13.

Recommended: confirm.

Default: none of them is planned.