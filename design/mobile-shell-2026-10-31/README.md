# Shell mockup (phone, tablet and desktop), 2026-10-31

An interactive mockup of the phone and tablet shells, built at the owner's request as the reference
for refactoring the current phone and tablet layouts in dp, so they work at any resolution. The same
component also draws the **desktop (PC) shell** at mouse density, built for the PC refactor the owner
asked for later the same day. It is a GUI mockup only: nothing in `cartalith-native/` was changed.

The proposals it illustrates, with the reasons, the rules and the owner questions, are at the repository
root:

- `MOBILE_SHELL_PROPOSAL.md` for the phone and the tablet;
- `PC_SHELL_PROPOSAL.md` for the desktop: one selection, one year, the Context panel, windows that do not
  block the map, the docked assistant and the menu changes;
- `VAULT_EDITOR_PROPOSAL.md` for the note editor.

What has been built is `cartalith-native/docs/STATUS.md`'s answer, never this folder's. The folder keeps
its first name, because the documents above cite it.

The live canvas is the claude.ai Design artifact `https://claude.ai/artifact/9VizRJs3LeRZDDpSNji6nP`.
It is private to the owner until it is shared from its Share menu. Press **Play** on a frame.

| File | Frame | What it is |
|---|---|---|
| `Main.dc.html` | 1480 × 1200 | **Device lab.** One shell at any size: presets from a 360 dp phone to 4K (×2, ×1.5 and ×1 density), rotation, theme, an Android **system back** key, and a corner grip that resizes the window freely up to 3840 dp |
| `Shell.dc.html` | 412 × 892 | **The shell itself** (props `w`, `h` in dp). It chooses the phone, phone-landscape or tablet layout from those two numbers alone |
| `Tablet.dc.html` | 1194 × 834 | The shell at an 11-inch tablet size |
| `Live.dc.html` | fills the window | The shell at the size of whatever window opens it. Open it on a phone, a foldable or a tablet to see that device's own layout |
| `Desktop.dc.html` | 1600 × 1000 | **The desktop shell**: the same `Shell` with `pointer` on, at 1366 × 768, 1600 × 900, 1920 × 1080 or 2560 × 1440 (shown scaled when larger than the board), with a theme switch. The assistant starts on here so Ask can be tried; the product ships it off (Ruling DC) |
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
- **On a tablet, hold the map for 500 ms.** The cartwheel and the context card open together, as
  shipped (`MAP_CONTEXT_SCOPE.md` §7.1). Slide to a slot and lift, or lift in place and tap. A `▸` slot
  opens its sub-ring. The four cardinals are the same in every domain; the diagonals are the domain's.
- **Notes:** MENU ▸ World ▸ Markdown vault…, the Vault pin, or ⌕ with `n`. A note opens in Preview:
  - Tap a link to follow it, and hold it to peek. Tick tasks in place.
  - A ` ```chronos ` block draws as a small timeline; tap a row to move the map's timeline to that year.
  - Write shows the coloured source with a keyboard row above where the phone's keyboard sits. Type `[[` for
    suggestions from notes, headings, and places on the map that have no note yet.
  - The tablet adds Split, and a Links · Outline · Properties panel on wide windows.
  - Following a place without a note offers its template. Rename updates the links. "Simulate an edit made
    outside Cartalith" shows the conflict banner.
- **The adaptable quick bar** (`CUSTOM_QUICK_BAR_SCOPE.md`) keeps every home it has today. On a tablet
  the pins sit at the end of the tool bar (more of them on a wider window, the rest under `⋯`), and up to
  four sit under the cartwheel. The mockup approximates the fit with width tiers; the shipped cluster
  fits by measured width (`DccQuickBar._layout()`). MENU starts with a Quick section. On the phone, tap or drag the grip on
  the map's left edge (the right edge when Handedness is Left) for the quick drawer. The Assistant chip
  comes first while Preferences ▸ Assistant is on.
- In the lab: rotate the 6T to landscape, pick TABLET 800 (the panels float), 4K ×1, or drag the grip.
- **Generate** (World ▸ Generate world, Ctrl+R on a keyboard) shows the shipped loading screen on every form factor:
  the busy overlay's relief trace, the stage line and one of the shipped loading lines per stage. It holds every
  input until the run ends, and has no cancel, as shipped.

## Things to try on the desktop

- **Select anything** by clicking the map or with Ctrl+F. Inspect, Note and Ask in the Context panel on the right
  follow it, and so does any window set to FOLLOW. Every name in Inspect is a link. Alt+← and Alt+→ walk back
  and forward through what you have selected.
- **Drag the timeline.** The map recolours each settlement by who held it that year and draws places not yet founded
  as outlines. Inspect says who held the place, and the chronos row of the note for that year lights up. The ticks
  under the slider are the world's events, its conflicts and the selection's own chronos entries. Open the strip (**TIMELINE**) for the controls CIVIL ▸ Timeline has today, now in every domain: the
  recorded years, **+ RECORD** for the shown year, the three filters and the speeds, then the event list.
- **Note** (Ctrl+2) reads or writes the selection's note in place; a link selects what it names. ⧉ opens the full
  vault window.
- **Ask** (Ctrl+3, or Ctrl+J from anywhere) is the assistant, docked. The chips are what goes with your message:
  they follow the panel, and a click leaves one out. Citations select a place or faction, move the timeline or
  open a note. Cards change nothing until Apply, and each Apply is one undo step. The answers are scripted; no
  model is called.
- **Pin** the panel (the pin in its header) to keep it on one thing while you click others. A banner then offers
  PIN TO IT or FOLLOW.
- **Windows do not block the map.** World ▸ Factions… (Ctrl+Shift+F) shows the selection's faction and follows it
  until you press FOLLOW to pin it. Drag a window by its title. Story ▸ Conflicts… selects a conflict and moves the
  year to it.
- **Menus** open as side-by-side columns, and hovering across the bar switches menus. Right-click the map for the
  card; Q opens the cartwheel at the pointer.

## Honest limits

- The map is a stand-in made by the mockup: noise continents, a priority-flood fill, D8 rivers and
  placed towns. It is not the engine, and the New World preview here is a stand-in for
  `WorldGen.preview_land_mask`.
- Windows (Data manager, Factions, Vault, Asset library and the rest) show their frame and their main
  sections, not their whole contents. They keep their implementations; only the frame they open in
  changes.
- 3D is a toggle label; the stand-in map stays 2D.
- The lab's readout applies the proposal's phone line, under 600 dp wide. The owner's standing answer is 900 dp of
  shortest width (`design/dcc-environment-2026-08-31/BUILD_ANSWERS.md` §2.3), and where the line ends up is MQ-1 of
  `MOBILE_SHELL_MS1_SPEC.md`. The mockup shows the proposal's design, not a decision.
- The desktop's assistant is a script over the stand-in world: it shows the flow (context chips, citations,
  proposal cards, Apply as one undo step), not a model's answers. The tools line under each answer names the kind
  of call the shipped assistant would make; it is illustration, not the shipped tool list.
- Measured in Chromium (Playwright) at 360 × 780, 412 × 892, 780 × 360, 1194 × 834, the lab presets and the
  desktop at 1600 × 900. Not run on a device.
