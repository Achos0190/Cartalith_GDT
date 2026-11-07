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
- **Notes:** MENU ▸ Records ▸ Markdown vault…, the Vault pin, or ⌕ with `n`. A note opens in **Write** (Ruling DY,
  VQ4), with Preview one tap away (Ctrl/Cmd+E on a keyboard):
  - Write shows the coloured source with a keyboard row above where the phone's keyboard sits. Type `[[` for
    suggestions from notes, headings, and places on the map that have no note yet. Put the caret in a link and
    an **Open ›** chip follows it (touch); on a desktop, Ctrl/Cmd+click the link.
  - In Preview, tap a link to follow it, and hold it to peek. Tick tasks in place.
  - A ` ```chronos ` block draws as a small timeline; tap a row to move the map's timeline to that year.
  - Saving is automatic about a second after you stop typing, and the footer says *editing…* or *saved*. **⟲ Revert to
    opened…** at the end of the toolbar asks first, then puts the note back as you first opened it this session; undo
    brings your edits back while the session lasts (VQ1, VQ6).
  - The tablet adds Split, and a Links · Outline · Properties panel on wide windows.
  - Following a place without a note offers its template; a link to nothing offers to create the note, never
    silently. Rename says how many links in how many notes it will rewrite. "Simulate an edit made outside
    Cartalith" shows the conflict banner.
- **The adaptable quick bar** (`CUSTOM_QUICK_BAR_SCOPE.md`) keeps every home it has today. On a tablet
  the pins sit at the end of the tool bar (more of them on a wider window, the rest under `⋯`), and up to
  four sit under the cartwheel. The mockup approximates the fit with width tiers; the shipped cluster
  fits by measured width (`DccQuickBar._layout()`). MENU starts with a Quick section. On the phone, tap or drag the grip on
  the map's left edge (the right edge when Handedness is Left) for the quick drawer. The Assistant chip
  comes first while Preferences ▸ Assistant is on.
- In the lab: rotate the 6T to landscape, pick TABLET 800 (the panels float), 4K ×1, or drag the grip.
- **The map's control row** (DS-13, `MOBILE_SHELL_PROPOSAL.md` §2.5; Ruling EB answered Q3 to Q5, and Q1 and Q2
  await the owner). It sits at the bottom of the map on the dominant hand's side, with ⌖ recentre at the outer end,
  then − | +, which step by 1.35 (Ruling EB item 2). It moves to the other side with MENU ▸ Preferences ▸ Handedness,
  and the scale bar swaps sides with it. Arm a drawing tool (Region, a brush, Way): ✋ appears at the inner end, and
  while it is on, one finger pans instead of drawing. With Inspect, a drag already pans, so there is no ✋. 2D/3D is
  in MAP ▸ View now. The touch tablet has the same row, without 3D.
- **The double-tap and thumb zoom** (Ruling EB item 3). Double-tap the map to zoom in ×1.7 at the finger. Double-tap,
  keep the finger down and slide: down zooms in, up zooms out, about the finger. With a drawing tool armed both are
  off, labelled "pending DS13-Q1" (MENU ▸ Preferences ▸ Touch & gestures), because that case waits on the owner.
- **A journey saved before a regenerate** (Ruling EA 10): MENU ▸ Story ▸ Saved journeys…, the third journey. The
  planner opens with a notice naming the leg change it dropped, with Show stage 3 and Dismiss.
- **3D height** (Ruling EA 14; Ruling EB items 5 and 6): MAP ▸ View ▸ 3D, then the *Height exaggeration (3D)* row,
  from 1× (true scale, Ruling CT) to 4× in steps of 0.25. On a tablet: Window ▸ 3D view, then the HEIGHT row in the
  3D card at the map's top-left. The value is remembered for the next session as Preferences ▸ 3D viewport
  defaults, here in this browser's storage; reload and it is still there.
- **Colour paint after the map changes width** (Ruling EA 15; Ruling EB item 7): CARTO ▸ Style ▸ Colours ▸
  *Simulate a regenerate at another width*. It draws 17 sample marks where their kilometres fall on the wider map,
  and one sentence heads the section with Show marks, Scale with the map and Dismiss. Scale with the map moves every
  mark back to its share of the map and removes the sentence, as one undo step; undo brings both back. The widths
  follow Preferences ▸ Units.
- **On a tablet narrower than 1 180 dp** the bar is ☰ · File · World · Records · Story (Ruling DY). World there holds
  only the WORLD domain's rows (Generate world, the two resets, Pipeline/Sculpt, Seed & size…); Records is the desktop's
  Records menu. Measured at 600, 768, 1 024 and 1 179 dp: the four menus end 31 dp before Undo at 600 dp.
- **Generate** (File ▸ Generate world, Ctrl+R on a keyboard; on a narrow tablet also World ▸ Generate world) shows the shipped loading screen on every form factor:
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
- **Note** (Ctrl+2) opens the selection's note in Write (Ruling DY, VQ4); Ctrl/Cmd+click a link to select what it
  names, and **REVERT…** puts the note back as you first opened it this session. ⧉ opens the full vault window.
- **The panel is modular** (Ruling DY, PQ1: "Tabs you arrange"). **+** at the end of the tabs lists the modules:
  History, Economy, Military, Faith, culture & relations, Territory, City layout, Here, Routes & journeys, Links and
  Recent & bookmarks. Tick one and it becomes a tab. Drag a tab to move it; right-click it to move it, open it in a
  window that follows the selection, or remove it. A module that does not fit the selection (Territory for a
  settlement, say) hides until a subject it fits is selected, and **+** shows how many are hidden. Window ▸ Context
  panel ▸ MODULES has the same check rows and Reset panel. Ctrl+1/2/3 always reach Inspect, Note and Ask. Each module
  names the shipped builder it reuses; figures the stand-in world does not hold are marked *stand-in*.
- **Ask** (Ctrl+3, or Ctrl+J from anywhere) is the assistant, docked. The chips are what goes with your message:
  they follow the panel, and a click leaves one out. Citations select a place or faction, move the timeline or
  open a note. Cards change nothing until Apply, and each Apply is one undo step. The answers are scripted; no
  model is called.
- **Pin** the panel (the pin in its header) to keep it on one thing while you click others. A banner then offers
  PIN TO IT or FOLLOW.
- **Windows do not block the map.** Records ▸ Factions… (Ctrl+Shift+F) shows the selection's faction and follows it
  until you press FOLLOW to pin it. Drag a window by its title. Story ▸ Conflicts… selects a conflict and moves the
  year to it.
- **The journey planner opens in its own window** beside the live map (Ruling DY, PQ4): select a settlement and the
  planner offers Start here, Add as stop and End here. **⧉ POP OUT** is Ruling DK's per-window button; here it shows
  where the window would go, since a browser canvas cannot open a second screen's window.
- **Ask keeps one conversation per session** (New clears it). The first message asks once whether to save it with the
  world, off unless you say so; the header then says *saved with the world* or *session only* (Ruling DY, PQ5).
- **Menus:** the World menu is now **Records**, and **Generate world** (Ctrl+R) is in File after New world…
  (Ruling DY, PQ6).
- **The top bar holds the armed tool's settings only** (Ruling DZ). With Inspect it shows the context name and one
  hint. New seed, Center landmasses, Bake ALL & finalize and GENERATE are gone from it, and each stays in the WORLD
  dock (Generate world also in File, Ctrl+R). Arm Freehand or Paint and the bar shows that brush's settings with
  Commit and Discard. The quick-bar pins stay at its right end.
- **Ruling EA's three small designs:**
  - Story ▸ Saved journeys… ▸ the third journey opens the planner window with the stale-leg notice (EA 10).
  - Window ▸ 3D view shows the 3D view's card at the map's top-left, with a HEIGHT row between its info line and Back
    to 2D (EA 14), from 1× to 4×, remembered across a reload (Ruling EB items 5 and 6).
  - CARTO ▸ Style ▸ Colours ▸ *Simulate a regenerate at another width* shows colour paint's notice (EA 15), with
    Scale with the map (Ruling EB item 7); the menu bar's undo and redo arrows take it back and forth, one step
    each.
- **Menus** open as side-by-side columns, above any open window, and hovering across the bar switches menus. Right-click the map for the
  card; Q opens the cartwheel at the pointer. Both keys also have Edit rows, Context card and Cartwheel at the pointer,
  so they can be rebound (`PC_SHELL_PROPOSAL.md` PQ10). The menus are checked row by row against the Godot drafts in
  `cartalith-native/docs/evidence/menu-parity-2026-11-01/`.

## Honest limits

- The map is a stand-in made by the mockup: noise continents, a priority-flood fill, D8 rivers and
  placed towns. It is not the engine, and the New World preview here is a stand-in for
  `WorldGen.preview_land_mask`.
- Windows (Data manager, Factions, Vault, Asset library and the rest) show their frame and their main
  sections, not their whole contents. They keep their implementations; only the frame they open in
  changes.
- 3D is a toggle label; the stand-in map stays 2D. On the desktop and the tablet the 3D card stands in for the card
  that `view3d.gd` ships (3D-1), with Ruling EA 14's HEIGHT row added; its tier line is a stand-in.
- The desktop frame still draws a pointer column (3D, ⌖, +, −) at the map's bottom right. The shipped desktop builds
  no such column (`viewport_host.gd::_build_navpad()` is touch-only), and Ruling EA 14 keeps the 3D pill off it. It
  is left for the PC pass; DS-13 covers touch only. Its + and − step by 1.35 like the touch rocker (Ruling EB item
  2); they stepped by 1.6 before.
- The row's 3D meanings (Ruling EB item 4: from 3D-2, − and + move the 3D camera and ⌖ frames the world) are not
  drawn, because 3D here is a label; the row still moves the 2D map.
- The 17 colour-paint marks are a sample the simulation draws; the mockup's Colour paint tool does not store the
  strokes you drag. The stand-in map does not change with the width, so scaled marks land back where they were
  painted. On a real regenerate the ground under them is new.
- The lab's readout applies the proposal's phone line, under 600 dp wide. The owner chose that line under Ruling DY
  (MQ-1 of `MOBILE_SHELL_MS1_SPEC.md`), to land with MS-7; MS-1 itself ships today's rule.
- The modules' figures that the stand-in world does not hold (economy and army sizes, provinces, claims, a layout's
  districts) are labelled stand-in. Bookmarks are kept for the session here; the build saves them inside the world file (Ruling DY, the owner's answer
  "In the world file").
- The desktop's assistant is a script over the stand-in world: it shows the flow (context chips, citations,
  proposal cards, Apply as one undo step), not a model's answers. The tools line under each answer names the kind
  of call the shipped assistant would make; it is illustration, not the shipped tool list.
- Measured in Chromium (Playwright) at 360 × 780, 412 × 892, 780 × 360, 1194 × 834, the lab presets and the
  desktop at 1600 × 900; Ruling DY's changes also at 1366, 1920 and 2560 wide and the narrow tablet bar at 600, 768,
  1 024 and 1 179 dp. DS-13's row was measured at 412 × 892, 892 × 412 and 1194 × 834 in both hands, and Ruling EA's
  three designs on the phone, the tablet and the desktop. Ruling EB's changes were driven there too, the gestures as
  synthetic touch pointer events on the map. Not run on a device.
