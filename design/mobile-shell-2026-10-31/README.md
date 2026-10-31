# Mobile shell mockup, 2026-10-31

An interactive mockup of the phone and tablet shells, built at the owner's request as the reference
for refactoring the current phone and tablet layouts in dp, so they work at any resolution. It is a
GUI mockup only: nothing in `cartalith-native/` was changed. The proposal it illustrates, with
the reasons, the rules and the owner questions, is `MOBILE_SHELL_PROPOSAL.md` at the repository
root. What has been built is `cartalith-native/docs/STATUS.md`'s answer, never this folder's.

The live canvas is the claude.ai Design artifact `https://claude.ai/artifact/9VizRJs3LeRZDDpSNji6nP`.
It is private to the owner until it is shared from its Share menu. Press **Play** on a frame.

| File | Frame | What it is |
|---|---|---|
| `Main.dc.html` | 1480 × 1200 | **Device lab.** One shell at any size: presets from a 360 dp phone to 4K (×2, ×1.5 and ×1 density), rotation, theme, an Android **system back** key, and a corner grip that resizes the window freely up to 3840 dp |
| `Shell.dc.html` | 412 × 892 | **The shell itself** (props `w`, `h` in dp). It chooses the phone, phone-landscape or tablet layout from those two numbers alone |
| `Tablet.dc.html` | 1194 × 834 | The shell at an 11-inch tablet size |
| `Live.dc.html` | fills the window | The shell at the size of whatever window opens it. Open it on a phone, a foldable or a tablet to see that device's own layout |
| `Rows.dc.html` | 412 × 1400 | The row renderer every list uses (menus, docks, windows), drawn standalone as a vocabulary sheet |
| `canvas.json` | | The canvas index: frame positions and the notes |

Each file loads `./support.js`, the Design-component runtime. As with the other `design/` imports it
is not duplicated here: copy `design/support.js` beside the files to open them from disk (checked
2026-10-31, `Main.dc.html` renders with it).

## Things to try

- **Hold GENERATE** in the bottom bar. A fan offers WORLD, CIVIL and CARTO: slide to one and lift, or
  tap. Swiping the tab sideways cycles the domains. The sheet title's ▾ opens the same fan.
- Open **CIVIL**: its nodes are chips (Landmarks · Factions · Routes & ways · Journeys), then its tools,
  then every dock category the desktop shows (Populate … Timeline) as accordions.
- **MENU** is the desktop menu bar, row for row: File ▸ Export, Preferences, Window ▸ Session mode.
- **Long-press the map** to drop a sample pin. A card shows actions for the current domain and the
  ground readouts.
- **Arm a tool** (Settle, Way, Measure, a sculpt feature). The sheet drops to peek and the tool strip
  shows the tool's name, its key options and Commit / Discard. Tap the strip's tool chip for the full
  options, or hold it for the tool fan.
- **⌕** finds places, factions, labels and routes, and runs any menu row, dock section or generation
  parameter, for example `vault`, `sea level` or `. export`.
- **File ▸ New world…** keeps the shipped window: the seed preview, Roll and Lock, a four-seed
  history and the AUTO-POPULATE options (new).
- In the lab: rotate the 6T to landscape, pick TABLET 800 (the panels float), 4K ×1, or drag the grip.

## Honest limits

- The map is a stand-in made by the mockup: noise continents, a priority-flood fill, D8 rivers and
  placed towns. It is not the engine, and the New World preview here is a stand-in for
  `WorldGen.preview_land_mask`.
- Windows (Data manager, Factions, Vault, Asset library and the rest) show their frame and their main
  sections, not their whole contents. They keep their implementations; only the frame they open in
  changes.
- 3D is a toggle label; the stand-in map stays 2D.
- Measured in Chromium (Playwright) at 360 × 780, 412 × 892, 780 × 360, 1194 × 834 and the lab presets.
  Not run on a device.
