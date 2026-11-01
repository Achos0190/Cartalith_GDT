# `html/` — a working copy of the DCC-line HTML, for the owner to move into Cartalith_RC

This folder is **not** part of the native port. It holds the next DCC-line version of the single-file HTML app
(`Cartalith v2.72 DCC test.html`), built here at the owner's request — *"Can we upgrade the layout in the html
with the new design?"*, then *"A working copy in this repo"* and *"Everything, staged"* — so that the owner can
move it into its home repository, **Cartalith_RC**, by hand. Nothing here is read by the Rust workspace or the
Godot project, and Cartalith_RC's own root `CLAUDE.md` governs the file once it lands there.

The file started as a byte copy of Cartalith_RC's `Cartalith v2.71 DCC test.html` (commit `aca222c` here; the
same bytes are frozen in `reference/`), so `git diff aca222c -- html/` shows exactly what v2.72 changed.

## What is here

| Path | What it is | Where it goes in Cartalith_RC |
|---|---|---|
| `Cartalith v2.72 DCC test.html` | the app, v2.72 (DCC line) — stage 1 of the shell upgrade: the desktop shell | the repository root, beside `Cartalith v2.71 DCC test.html` (keep v2.71: old versions are never edited or removed) |
| `tests/perf/probe_desktopshell.js` | the Playwright probe for this version (usage in its header) | `tests/perf/probe_desktopshell.js` |
| `CHANGELOG_v2.72.md` | the `## v2.72 (DCC line) — …` entry, in `CHANGELOG.md`'s own format | paste at the top of `CHANGELOG.md`, above `## v2.71 (DCC line) — …` |
| `CLAUDE_v2.72.md` | two pastes for the root `CLAUDE.md`: the new file-table row (and the one phrase to change in v2.71's row), and the `### … (v2.72, DCC-line file only)` section | as that file says: the row above v2.71's, the section above v2.71's section |
| `evidence/v2.72/*.png` | screenshots: v2.72 at 1366×768, 1600×900, 1920×1080 and 2560×1440 (dark), 1600×900 (light), File ▸ Export and the sculpt bar at 1600×900, the phone at 412×915, v2.71 at 1600×900, and the design board's 1600×900 frame (shot in a 1600×1000 window, since the board draws its own toolbar above the frame) | review only; Cartalith_RC needs none of them |

## The stage plan

1. **Stage 1 — this version: the desktop shell** (windows wider than 860px). The menu bar with the eight menus,
   the tool-options bar holding the armed tool's settings only (Ruling DZ), Run in the WORLD dock, the rail on
   the left, the Context panel (Inspect), the timeline strip in every domain, the restyled status bar and map
   controls. A phone (860px and narrower) keeps v2.71's layout exactly.
2. **Stage 2 — the phone and tablet shells**, from `MOBILE_SHELL_PROPOSAL.md` and `MOBILE_SHELL_MS1_SPEC.md`.
   The `_DSK_SLOTS` table is where a control's phone home is decided, so stage 2 starts there.
3. **Stage 3 — the Context panel's modules and the journey planner in its own window** (`PC_SHELL_PROPOSAL.md`
   §3.3, §3.7), with the selection history (back, forward, pin) and Explore's journeys rehomed.

## Moving it

1. Copy `Cartalith v2.72 DCC test.html` to Cartalith_RC's root and `tests/perf/probe_desktopshell.js` to its
   `tests/perf/`.
2. Paste `CHANGELOG_v2.72.md` (from its `## v2.72` heading down) at the top of `CHANGELOG.md`.
3. Make the two `CLAUDE.md` edits `CLAUDE_v2.72.md` describes.
4. Run the checks below from Cartalith_RC's root.

## Verifying it (from Cartalith_RC's root, after moving)

The headless suites and the render A/B (none of them can see the shell; they prove the engine did not move):

```bash
tests/run.sh "Cartalith v2.72 DCC test.html"        # expect 1280 passed, 0 failed (the same as v2.71)
tests/run_um.sh "Cartalith v2.72 DCC test.html"     # expect 882 passed, 0 failed
node tests/perf/hash_gen1.js "Cartalith v2.71 DCC test.html" "Cartalith v2.72 DCC test.html"   # expect ALL IDENTICAL
```

The shell itself, by real clicks and measured geometry (the second file makes the phone half compare against
v2.71's own geometry):

```bash
node tests/perf/probe_desktopshell.js "Cartalith v2.72 DCC test.html" "Cartalith v2.71 DCC test.html"
```

Every Playwright script reads `PLAYWRIGHT_DIR` and `CHROME_BIN` as the other probes in that folder do; unset
`CHROME_BIN` if its default path does not exist on the machine.

RC's own probes that query a header, tool-options, rail, dock, timeline, status or modal id were run on v2.71
and v2.72 before this was handed over; the CHANGELOG entry and the lane report classify every difference. To
repeat them:

```bash
for p in shellui domainrail cogmodal assetsbtn geninfo worldreset settleparams typepreview savetree civbusy \
         manpower trade landmass landsurface roadconnect foodshed placement water_gap; do
  NODE_OPTIONS=--max-old-space-size=8192 node tests/perf/probe_$p.js "Cartalith v2.72 DCC test.html"
done
```

Expected on v2.72: four assertions differ from v2.71, all intended (the CHANGELOG entry names each);
`probe_savetree` needs the larger heap on v2.71 too (it ran out at node's default 2 GB here on both files);
`probe_civbusy` opens its file under `/home/user/Cartalith_RC/`, so run it from a clone at that path.

## What still needs a person at a real browser

- **Ctrl+R, Ctrl+S, Ctrl+O** (Cmd on macOS): the page calls `preventDefault`, and headless Chromium has no
  reload or save accelerator to prove it wins against. Press each once in Chrome, Firefox and Safari with a
  world open: Ctrl+R must generate and must not reload, Ctrl+S must download the project .zip, Ctrl+O must open
  the file picker.
- **The 3D view from Window ▸ 3D view** with real WebGL (headless runs the software path).
- **A touch screen wider than 860px**: ✋ shows in the map's bottom-right column only for a coarse pointer, and
  the pan joystick sits just above that column.
- **A screen reader over the menu bar** (menu buttons carry `aria-haspopup`/`aria-expanded`; rows are buttons).
- **Resizing a window across 860px** by hand: the probe crosses it headless, and a real browser's resize is
  worth one look.
