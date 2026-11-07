# `html/` — a working copy of the DCC-line HTML, for the owner to move into Cartalith_RC

This folder is **not** part of the native port. It holds the next DCC-line versions of the single-file HTML app,
built here at the owner's request — *"Can we upgrade the layout in the html with the new design?"*, then *"A working
copy in this repo"* and *"Everything, staged"* — so that the owner can move each into its home repository,
**Cartalith_RC**, by hand. Nothing here is read by the Rust workspace or the Godot project, and Cartalith_RC's own root
`CLAUDE.md` governs a file once it lands there.

`Cartalith v2.72 DCC test.html` started as a byte copy of Cartalith_RC's `Cartalith v2.71 DCC test.html` (commit
`aca222c` here; the same bytes are frozen in `reference/`), so `git diff aca222c -- html/` shows what v2.72 changed.
`Cartalith v2.73 DCC test.html` started as a byte copy of v2.72 (commit `61ccab5`), so `git diff 61ccab5 -- html/`
shows what v2.73 changed. `Cartalith v2.74 DCC test.html` started as a byte copy of v2.73 (commit `dfa97d2`), so
`git diff dfa97d2 -- html/` shows what v2.74 changed. Old versions stay as they are: they are never edited.

## What is here

| Path | What it is | Where it goes in Cartalith_RC |
|---|---|---|
| `Cartalith v2.74 DCC test.html` | the app, v2.74 (DCC line) — the fix pass and the design alignment of the 2026-11-01 audits (`audits/v2.72-2026-11-01/PLAN.md` §5 and §8), the planner's toll fix (Ruling EC item 6) and rounded corners (item 7), ahead of the ports and stage 3 (Ruling EF) | the repository root, beside v2.73 |
| `tests/perf/probe_fixpass.js` | v2.74's Playwright probe: every fix-pass and design-alignment finding at its bar, by real clicks, taps and keys, with measured geometry and computed style; give v2.73 second and it also takes the truncation census on both (usage in its header) | `tests/perf/probe_fixpass.js` |
| `tests/perf/census_v274.js` | v2.74's contrast gate: the visual audit's WCAG census over its seven views, both themes, 1600×900 and 1366×768 | `tests/perf/census_v274.js` |
| `tests/perf/shots_v274.js` | the sixteen captures in `evidence/v2.74/`, by the probe's own clicks | `tests/perf/shots_v274.js` (optional) |
| `CHANGELOG_v2.74.md`, `CLAUDE_v2.74.md` | v2.74's changelog entry and its two `CLAUDE.md` pastes | as v2.73's, after them |
| `evidence/v2.74/*.png` | v2.74's sixteen captures, final bytes: the desktop at 1366 (WORLD › Generate) and 1600 (CIVIL › Generation), dark and light, the strip open; 1920 (CARTO, its sections open) and 2560 (EXPLORE, a settlement selected); the Settlements window, the route editor, File ▸ Export and the asset library at 1600 dark; the place popup at 1600 light; the sculpt bar with Freehand armed at 1366; New World at 1366 (over the flat backdrop, Q10a); the phone at 412×915 (the map, the Settlements window's faction rows) and the tablet at 1194×834 | review only |
| `Cartalith v2.73 DCC test.html` | the app, v2.73 (DCC line) — stage 2 of the shell upgrade: the phone and tablet shells | the repository root, beside v2.72 |
| `Cartalith v2.72 DCC test.html` | the app, v2.72 (DCC line) — stage 1: the desktop shell | the repository root, beside `Cartalith v2.71 DCC test.html` (keep v2.71) |
| `tests/perf/probe_mobileshell.js` | v2.73's Playwright probe: the three classes, the phone shell, DS-13, the gestures, the tablet, the docking rule (usage in its header) | `tests/perf/probe_mobileshell.js` |
| `tests/perf/probe_desktopshell.js` | v2.72's Playwright probe (usage in its header) | `tests/perf/probe_desktopshell.js` |
| `CHANGELOG_v2.73.md`, `CHANGELOG_v2.72.md` | each version's `## v2.7x (DCC line) — …` entry, in `CHANGELOG.md`'s own format | paste at the top of `CHANGELOG.md`, v2.72's first, then v2.73's above it |
| `CLAUDE_v2.73.md`, `CLAUDE_v2.72.md` | each version's two pastes for the root `CLAUDE.md`: the file-table row (and the phrase to change in the previous row) and the `### … (v2.7x, DCC-line file only)` section | as each file says, v2.72's first |
| `evidence/v2.73/*.png` | 26 screenshots: the phone at 412×915 (the map with the pull-out tab, the quick drawer open with its grip riding the drawer's edge, the domain sheet, the domain fan, MENU, MENU › Edit with each disabled row's reason on a second amber line, MENU over a settlement's popup, MAP with one View caption and the Scale bar switch, the sculpt strip — feature picker and Commit/Discard only, its brush/hardness/intensity now on the WORLD › Sculpt page shown beside it — the strip with Settlement armed, the long-press card, left-handed), at 360×780 (STORY › Time, the asset library with ← Map in the app bar, MENU › File › Export, the Settlements window's Global tab with each label stacked above a full-width slider), the landscape phone at 844×390 (the domain sheet; Sculpt armed; left-handed, the grip at the sheet's edge), the tablet at 1194×834 (the Context panel floating; the Sculpt tool-options bar carrying Discard and Commit) and 768×1024 (the narrow bar, ☰ › Preferences drilled in), the desktop at 600×900 (the narrow bar, File › Export drilled in), 700×900 (the left dock floating, open) and 1366×768. Fix round 2 (2026-11-02) re-took fifteen and added five (the Settlements Global window, MENU › Edit's reason lines, the WORLD › Sculpt page, landscape Sculpt, the tablet Sculpt bar); the other six re-take pixel-identical on the final bytes | review only; Cartalith_RC needs none of them |
| `evidence/v2.72/*.png` | v2.72's screenshots (the desktop at four sizes, light and dark, File ▸ Export, the sculpt bar; the phone at 412×915; v2.71; the design board) | review only |

## The stage plan

1. **Stage 1 — v2.72: the desktop shell** (then: windows wider than 860px). The menu bar with the eight menus, the
   tool-options bar holding the armed tool's settings only (Ruling DZ), Run in the WORLD dock, the rail on the left,
   the Context panel (Inspect), the timeline strip in every domain, the restyled status bar and map controls.
2. **Stage 2 — v2.73: the phone and tablet shells**, from `MOBILE_SHELL_PROPOSAL.md` §2.1, §2.2, §2.4, §2.5, §3, §4 and Rulings DT,
   DU, DX, DY, EB and EC (items 1–4). One decider for the class (phone under 600px wide or under 500px tall in
   landscape; tablet for any other coarse primary pointer; desktop for a fine one), CSS keyed on `html[data-shell]`,
   the slot table generalised to every shell; the phone's bottom bar, app bar, sheet, MENU, STORY, tool strip, quick
   drawer on a grip (the owner's "pull-out tab") and long-press card; DS-13 and Handedness; the touch gestures; the
   tablet's touch sizing, drill-in menus and narrow bar; the docking rule on the desktop and the tablet.
3. **Stage 3 — the Context panel's modules and the journey planner in its own window** (`PC_SHELL_PROPOSAL.md`
   §3.3, §3.7), with the selection history (back, forward, pin) and Explore's journeys rehomed — after which EXPLORE
   stops being a domain (the phone's domain tab and fan list it until then).

**Before stage 3, the owner's order** (Ruling EF, 2026-11-02: *"Fixes and feature ports first then stage 3"*), each
one new file, its number confirmed free in Cartalith_RC when its lane starts:
1. **v2.74 — the fix pass and the design alignment** (built in two lanes; its verification rounds are unfinished — `cartalith-native/docs/STATUS.md` says how far it has got, and `build-notes/v2.74/` holds the open review findings): the plan's §5, the toll fix (Ruling EC item 6), the
   rounded corners (item 7) and §8's V24 (the washed on-state and the dock's type), V25, V29 and V30.
2. **v2.75 — the speed pass**: §6 (generate) and §7 (zoom, LOD, sliders, brush, the render tail), with the
   half-resolution slider preview (Ruling EC item 8). `hash_gen1` stays ALL IDENTICAL.
3. **v2.76 — the first ports** (Ruling EC item 5): Export map image, Find runs commands, faction relations and a
   settlement's political history.
4. **v2.77 — the Measure tool** (Ruling EC item 5), on v2.73's touch layer.
5. **Then stage 3** (above), with the map card's two journey verbs (F07).

## Moving it

1. Copy `Cartalith v2.72 DCC test.html`, `Cartalith v2.73 DCC test.html` and `Cartalith v2.74 DCC test.html` to
   Cartalith_RC's root, and `tests/perf/probe_desktopshell.js`, `probe_mobileshell.js`, `probe_fixpass.js` and
   `census_v274.js` to its `tests/perf/`.
2. Paste `CHANGELOG_v2.72.md`, then `CHANGELOG_v2.73.md`, then `CHANGELOG_v2.74.md` (each from its `## v2.7x` heading
   down) at the top of `CHANGELOG.md`, so v2.74's entry ends up first.
3. Make the `CLAUDE.md` edits `CLAUDE_v2.72.md` describes, then `CLAUDE_v2.73.md`'s, then `CLAUDE_v2.74.md`'s.
4. Run the checks below from Cartalith_RC's root.

## Verifying it (from Cartalith_RC's root, after moving)

The headless suites and the render A/B (none of them can see the shell; they prove the engine did not move):

```bash
tests/run.sh "Cartalith v2.73 DCC test.html"        # expect 1280 passed, 0 failed (re-run once if the bathymetry-variance check flakes — CHANGELOG v2.73)
tests/run_um.sh "Cartalith v2.73 DCC test.html"     # expect 882 passed, 0 failed
node tests/perf/hash_gen1.js "Cartalith v2.72 DCC test.html" "Cartalith v2.73 DCC test.html"   # expect ALL IDENTICAL
```

The shells themselves, by real taps, CDP touches, clicks and measured geometry:

```bash
node tests/perf/probe_mobileshell.js "Cartalith v2.73 DCC test.html"    # expect 279 passed, 0 failed
node tests/perf/probe_mobileshell.js "Cartalith v2.72 DCC test.html"    # the before build: 133 fail, by design
node tests/perf/probe_desktopshell.js "Cartalith v2.73 DCC test.html" "Cartalith v2.71 DCC test.html"
```

v2.74, the fix pass and the design alignment (its CHANGELOG entry gives each figure):

```bash
tests/run.sh "Cartalith v2.74 DCC test.html"        # expect 1280 passed, 0 failed (two random-world assertions can flake — CHANGELOG v2.74; re-run once)
tests/run_um.sh "Cartalith v2.74 DCC test.html"     # expect 882 passed, 0 failed
node tests/perf/hash_gen1.js "Cartalith v2.73 DCC test.html" "Cartalith v2.74 DCC test.html"   # expect ALL IDENTICAL
node tests/perf/probe_fixpass.js "Cartalith v2.74 DCC test.html" "Cartalith v2.73 DCC test.html"   # expect 220 passed, 0 failed
node tests/perf/probe_fixpass.js "Cartalith v2.73 DCC test.html"   # the before build: 70 passed, 146 failed, by design
node tests/perf/census_v274.js "Cartalith v2.74 DCC test.html"     # expect 0 enabled text runs below AA (v2.73: 561)
node tests/perf/probe_mobileshell.js "Cartalith v2.74 DCC test.html"    # expect 279 passed, 0 failed
node tests/perf/probe_desktopshell.js "Cartalith v2.74 DCC test.html"   # expect 154 passed, 4 failed: v2.73's four, by design (below)
```

`probe_desktopshell.js` on v2.73 fails four, all by design (CHANGELOG v2.73 classifies them): the two version checks
(v2.72 → v2.73) and its two phone sections, which pin v2.71's phone (the ☰ drawer, the document bar) that stage 2
replaced. Every desktop assertion passes unchanged. v2.74 fails the same four and no other. Run it alone on the machine: under load its *"NEW SEED is
clickable in the dock and generates"* failed once in the 2026-11-02 verifiers' runs and passed in isolation every time;
it now prints its three parts (clicked, in the dock, generates run), so a red run says which — re-run before believing it.

Every Playwright script reads `PLAYWRIGHT_DIR` and `CHROME_BIN` as the other probes in that folder do; unset
`CHROME_BIN` if its default path does not exist on the machine.

RC's own shell probes were run on v2.72, v2.73 and v2.74; the CHANGELOG v2.73 entry classifies every difference, and
v2.74 prints v2.73's FAIL lines, line for line (CHANGELOG v2.74 gives the counts). To repeat them:

```bash
for p in shellui domainrail cogmodal assetsbtn settleparams geninfo worldreset typepreview; do
  NODE_OPTIONS=--max-old-space-size=8192 node tests/perf/probe_$p.js "Cartalith v2.74 DCC test.html"
done
```

## What still needs a person at a real device

- **Touch on glass** (MS-10): the hold (480ms phone, 500ms tablet), the tap's 240ms wait for a double-tap, the
  double-tap's ×1.7, thumb zoom's 120px per doubling, the domain tab's 380ms hold and its swipe, the grip's drag, and
  the sheet's drag between detents — on the OnePlus 6T and a tablet, in both hands.
- **The safe-area insets**: a phone with a notch and a gesture bar (headless reads every `env(safe-area-inset-*)` as 0).
- **An on-screen keyboard on a landscape tablet**: typing in Find must not flip the tablet into the phone shell (the
  decider's keyboard guard).
- **Android's own long-press menu, and the click after a fling**: both are handled in code and only provable on a
  device.
- **Rotating a real phone** between portrait and landscape with the sheet open.
- **Ctrl+R, Ctrl+S, Ctrl+O** on the desktop (v2.72's list), **the 3D view** with real WebGL, and **a screen reader**
  over the menu bar, the phone's bar, sheet and fan.
- **The phone frame's gesture figures that are labelled judgements** (the domain tab's swipe, the grip's drag, the
  drawer's swipe, the sheet's dead band, the click-swallow windows, the card's gap — each named in the file) are
  set on glass in MS-10. (DS13-Q1, DS13-Q2 and thumb zoom with a drawing or placing tool are no longer open: Ruling
  EC items 1–4 answered them on 2026-11-01 at exactly the defaults v2.73 builds.)
- **A sideways swipe on the open quick drawer, and the asset library on a real 360dp phone** (a mobile browser may
  zoom out rather than widen its layout viewport, as Chrome's emulation does).
- **Fix round 2's three device checks**: a pinch on the sheet or the app bar must not zoom the page (Android's *Force
  enable zoom* may override `touch-action`); a field focused in the sheet must stay above the keyboard (proved
  headless only for a keyboard that resizes the layout viewport, which Chrome on Android does not do by default); and
  a hold on the domain tab lifted in place must leave the fan open, its lift's click spent on the new scrim.
- **The 1px seam in the two 844×390 captures** (`v273_phone844x390_landscape`, `…_left_hand_sheet`): a darker row
  across the map at the side sheet's head line, present in every headless full-page capture but not in a clipped one
  and not in the map's own pixels (see CHANGELOG v2.73) — confirm on a device it is absent, or it is a capture
  artifact to leave as noted.
- **v2.74's fix pass and design alignment:** one native select list and one number spinner in each theme (the new
  `color-scheme`; headless cannot capture native popups); Segoe UI and Consolas on Windows for the widths that were
  measured here with Inter and DejaVu Sans Mono — the status chips, CIVIL's mono sub-tabs (they fit 1366 here), the
  sculpt picker with Freehand armed (one feature out of sight at 1366), and the dock's mono segments and action
  buttons with the rows they wrap; Credits from Settings › About on a real phone; a screen reader over the timeline
  strip's three filter chips (each should still be announced as a checkbox); a finger on DS-13's round buttons and the
  12px touch controls on a real tablet and phone; and, for the owner to look at, the flat backdrop behind the start
  card (Q10a), the busy overlay taking the map's input mid-generate (Q10b), and the corners and washed on-states.
