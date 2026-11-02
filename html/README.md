# `html/` — a working copy of the DCC-line HTML, for the owner to move into Cartalith_RC

This folder is **not** part of the native port. It holds the next DCC-line versions of the single-file HTML app,
built here at the owner's request — *"Can we upgrade the layout in the html with the new design?"*, then *"A working
copy in this repo"* and *"Everything, staged"* — so that the owner can move each into its home repository,
**Cartalith_RC**, by hand. Nothing here is read by the Rust workspace or the Godot project, and Cartalith_RC's own root
`CLAUDE.md` governs a file once it lands there.

`Cartalith v2.72 DCC test.html` started as a byte copy of Cartalith_RC's `Cartalith v2.71 DCC test.html` (commit
`aca222c` here; the same bytes are frozen in `reference/`), so `git diff aca222c -- html/` shows what v2.72 changed.
`Cartalith v2.73 DCC test.html` started as a byte copy of v2.72 (commit `61ccab5`), so `git diff 61ccab5 -- html/`
shows what v2.73 changed. v2.72 stays as it is: old versions are never edited.

## What is here

| Path | What it is | Where it goes in Cartalith_RC |
|---|---|---|
| `Cartalith v2.73 DCC test.html` | the app, v2.73 (DCC line) — stage 2 of the shell upgrade: the phone and tablet shells | the repository root, beside v2.72 |
| `Cartalith v2.72 DCC test.html` | the app, v2.72 (DCC line) — stage 1: the desktop shell | the repository root, beside `Cartalith v2.71 DCC test.html` (keep v2.71) |
| `tests/perf/probe_mobileshell.js` | v2.73's Playwright probe: the three classes, the phone shell, DS-13, the gestures, the tablet, the docking rule (usage in its header) | `tests/perf/probe_mobileshell.js` |
| `tests/perf/probe_desktopshell.js` | v2.72's Playwright probe (usage in its header) | `tests/perf/probe_desktopshell.js` |
| `CHANGELOG_v2.73.md`, `CHANGELOG_v2.72.md` | each version's `## v2.7x (DCC line) — …` entry, in `CHANGELOG.md`'s own format | paste at the top of `CHANGELOG.md`, v2.72's first, then v2.73's above it |
| `CLAUDE_v2.73.md`, `CLAUDE_v2.72.md` | each version's two pastes for the root `CLAUDE.md`: the file-table row (and the phrase to change in the previous row) and the `### … (v2.7x, DCC-line file only)` section | as each file says, v2.72's first |
| `evidence/v2.73/*.png` | 21 screenshots: the phone at 412×915 (the map with the pull-out tab, the quick drawer open with its grip riding the drawer's edge, the domain sheet, the domain fan, MENU, MENU drawn over a settlement's popup, MAP with one View caption and the Scale bar switch, the sculpt strip with ✋, the strip with Settlement armed — no Sculpt rows, the long-press card, left-handed), at 360×780 (STORY › Time, the asset library with ← Map and the bar on screen, MENU › File › Export), the landscape phone at 844×390 (the domain sheet; left-handed, the grip at the sheet's edge), the tablet at 1194×834 (the Context panel floating, open, its pin beside the chevron) and 768×1024 (the narrow bar, ☰ › Preferences drilled in), the desktop at 600×900 (the narrow bar, File › Export drilled in), 700×900 (the left dock floating, open) and 1366×768. The fix pass (2026-11-02) re-took two and added seven | review only; Cartalith_RC needs none of them |
| `evidence/v2.72/*.png` | v2.72's screenshots (the desktop at four sizes, light and dark, File ▸ Export, the sculpt bar; the phone at 412×915; v2.71; the design board) | review only |

## The stage plan

1. **Stage 1 — v2.72: the desktop shell** (then: windows wider than 860px). The menu bar with the eight menus, the
   tool-options bar holding the armed tool's settings only (Ruling DZ), Run in the WORLD dock, the rail on the left,
   the Context panel (Inspect), the timeline strip in every domain, the restyled status bar and map controls.
2. **Stage 2 — v2.73: the phone and tablet shells**, from `MOBILE_SHELL_PROPOSAL.md` §2.1–§2.5, §3, §4 and Rulings DT,
   DU, DX, DY, EB. One decider for the class (phone under 600px wide or under 500px tall in landscape; tablet for any
   other coarse primary pointer; desktop for a fine one), CSS keyed on `html[data-shell]`, the slot table generalised
   to every shell; the phone's bottom bar, app bar, sheet, MENU, STORY, tool strip, quick drawer on a grip (the
   owner's "pull-out tab") and long-press card; DS-13 and Handedness; the touch gestures; the tablet's touch sizing,
   drill-in menus and narrow bar; the docking rule on the desktop and the tablet.
3. **Stage 3 — the Context panel's modules and the journey planner in its own window** (`PC_SHELL_PROPOSAL.md`
   §3.3, §3.7), with the selection history (back, forward, pin) and Explore's journeys rehomed — after which EXPLORE
   stops being a domain (the phone's domain tab and fan list it until then).

## Moving it

1. Copy `Cartalith v2.72 DCC test.html` and `Cartalith v2.73 DCC test.html` to Cartalith_RC's root, and
   `tests/perf/probe_desktopshell.js` and `tests/perf/probe_mobileshell.js` to its `tests/perf/`.
2. Paste `CHANGELOG_v2.72.md`, then `CHANGELOG_v2.73.md` (each from its `## v2.7x` heading down) at the top of
   `CHANGELOG.md`, so v2.73's entry ends up first.
3. Make the `CLAUDE.md` edits `CLAUDE_v2.72.md` describes, then those `CLAUDE_v2.73.md` describes.
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
node tests/perf/probe_mobileshell.js "Cartalith v2.73 DCC test.html"    # expect @@N@@ passed, 0 failed
node tests/perf/probe_mobileshell.js "Cartalith v2.72 DCC test.html"    # the before build: @@F72@@ fail, by design
node tests/perf/probe_desktopshell.js "Cartalith v2.73 DCC test.html" "Cartalith v2.71 DCC test.html"
```

`probe_desktopshell.js` on v2.73 fails four, all by design (CHANGELOG v2.73 classifies them): the two version checks
(v2.72 → v2.73) and its two phone sections, which pin v2.71's phone (the ☰ drawer, the document bar) that stage 2
replaced. Every desktop assertion passes unchanged.

Every Playwright script reads `PLAYWRIGHT_DIR` and `CHROME_BIN` as the other probes in that folder do; unset
`CHROME_BIN` if its default path does not exist on the machine.

RC's own shell probes were run on v2.72 and v2.73; the CHANGELOG v2.73 entry classifies every difference. To repeat
them:

```bash
for p in shellui domainrail cogmodal assetsbtn settleparams geninfo worldreset typepreview; do
  NODE_OPTIONS=--max-old-space-size=8192 node tests/perf/probe_$p.js "Cartalith v2.73 DCC test.html"
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
