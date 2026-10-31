# MOBILE_SHELL_PROPOSAL.md — the phone and tablet shells, re-laid out in dp

**What this is.** A proposal and an interactive mockup, 2026-10-31, for refactoring the phone and tablet shells so
that every menu and option has a home and the layout follows the window's size in dp at any resolution. **What it is
not:** a status record, a build, or a change to the PC menu bar. Nothing in `cartalith-native/` was changed. How far
any of it is built is `cartalith-native/docs/STATUS.md`'s answer. The work is routed from `OUTSTANDING_WORK.md`
("Phone and touch").

**The mockup.** `design/mobile-shell-2026-10-31/` (README there), live as the claude.ai Design artifact
`https://claude.ai/artifact/9VizRJs3LeRZDDpSNji6nP` (private to the owner until shared). `Main.dc.html` is a device
lab: one shell at any size from a 360 dp phone to a 4K panel, with rotation, theme, an Android system-back key and a
free-resize grip. `Live.dc.html` sizes itself to whatever window opens it.

**The owner's words (2026-10-31), verbatim:**

> Check out the current mobile designs for both tablet and mobile phone and stay withing the DCC style design. But
> improve the layout and input methods to account for all menu's and options. Make sure it scales properly accros
> devices and screensizes. Best idea would be to correct the current faulty layout on phone and expand on it make the
> artifact a complete functioning mockup. For example the current menu has a generate menu, when longpressing the
> button it would be nice to be able to switch to civil or cartho modes with all their menus. Leave the pc menu as is.

> Don't forget that I don't expect you to create the app all new. Just the full and extensive GUI as an interactive
> mockup that can be used to completely refractor the current phone and tablet versions in a resolution independent
> way.

> Tablet resolution should go up to 4k

> Also check the project code and branches for the current new world screen with it's rudementary preview, that I'd
> like to keep.

> Maybe add options such as: auto-populate and then things like settlements/routes/names/POI

**Sources read.** The designs: `design/Cartalith-Android-2026-09-07.dc.html`, `design/mcp-2026-09-07/Cartalith
Tablet.dc.html`, `design/dcc-environment-2026-08-31/` (both canvases and `spec/06-phone.md`), each rendered and walked.
The shipped phone, from the OnePlus 6T captures in `cartalith-native/docs/evidence/phone-gui-2026-10-26/`. The code,
in two read-only inventories committed as evidence in `cartalith-native/docs/evidence/mobile-shell-2026-10-31/`:
`MENU_INVENTORY.md` (all eight menus row by row, the tablet bar, phone MORE, search, context card, ring and keys) and
`DOMAIN_INVENTORY.md` (domains, nodes, tools, every dock section and control, the 120 generation parameters, the 43
analysis fields). Both were written by agents reading the code; the symbols this document leans on were opened by
the author at the symbol: `DccShell.RAIL_NODES`, `PHONE_TABS`, `_compute_layout_mode`, `_TABLET_MIN_DP`,
`_PHONE_ASPECT_MAX`, `_phone_scale`, the `PHONE_DETENT_*` constants, `menus.gd`'s id table, and
`new_world_dialog.gd`'s constants and header. All remote branches point at the same commit as `main`
(`git ls-remote --heads origin`, 2026-10-31), so the New World preview checked is the one on `main`.

---

## 1. What is faulty today

### 1.1 The phone shell (code, and the 6T captures)

| Fault | Where | Evidence |
|---|---|---|
| **CIVIL has no tab.** Its six tools, Landmarks and the CIVIL dock are reached through MORE ▸ Civilization | `PHONE_TABS` (`map` · `gen` · `plan` · `more`) | MENU_INVENTORY §4.2, DOMAIN_INVENTORY §4.6 |
| **CARTO's seven categories and most of WORLD's dock have one door:** MORE ▸ "Not on the MORE list" ▸ Window ▸ Left dock. World ▸ Places… and both Timeline rows change the domain but never open the phone's left sheet | `select_domain_category`, `_open_left_sheet` callers | MENU_INVENTORY §8 item 9 |
| **The MAP tab holds the tool-options row and the quick chips only.** The designed Tools / Layers / Style panel was never built | `_build_phone_tool_sheet` | DOMAIN_INVENTORY §5.6; the 6T captures: a horizontally clipped "CARTOGRAPHY · STYLE  Presentation only — …" strip |
| **GENERATE lacks desktop controls:** Center landmasses, Import, Finalize / Bake, Droplet erode, Carve fjords; Sculpt lacks per-feature parameters, the Freehand sub-mode and Grid snap; Falloff draws as a bare 0–3 slider | `build_phone_generate`, `_pg_sculpt` | MENU_INVENTORY §8 item 10, DOMAIN_INVENTORY §3.1, §3.2 |
| **The phone context peek drops rows** with children or a parameter (Copy coordinate, View field, Style preset, Class, brush size) | `PhoneMenu.peek_card` | MENU_INVENTORY §6.1, §8 item 11 |
| **Search runs only pinnable commands;** a check, radio or generated row only toasts a reason | `_run_search_command` | MENU_INVENTORY §5.1, §8 item 7 |
| Floating map controls collided with the armed-tool pill (fixed 2026-10-26); the control column redesign is still open as DS-13 | `viewport_host.gd` | `STATUS.md` 2026-10-26 |
| Drags that start on a card, label or list do not scroll; popup lists select on touch-down | central touch-scroll fix, not built | `STATUS.md` 2026-10-26 |
| **Phone or tablet is decided once at boot from aspect ratio** (short ÷ long side < 0.6) plus a 900 dp shortest-width test on a real device. The phone UI is then scaled by short side ÷ 412 rather than laid out in dp. A foldable that unfolds, or a resized window, keeps the boot-time answer | `_compute_layout_mode`, `_PHONE_ASPECT_MAX`, `_TABLET_MIN_DP`, `_phone_scale` | read at the symbols |

### 1.2 The design canvases

- **Phone (`Cartalith-Android-2026-09-07.dc.html`, `spec/06-phone.md` §9):** targets below the floor (36 dp stage
  chips, 38 dp segment chips and steppers, 48 × 44 icon cells); no safe-area, system-back or keyboard policy; a painted
  status bar and gesture handle; and its `TABLET 800` frame runs the phone layout unchanged.
- **Tablet (`Cartalith Tablet.dc.html`):** both docks are always docked. Rendered at 800 × 1280 the map is a strip
  about 190 dp wide and the dock rows truncate ("Plates & tec…").

---

## 2. The proposal

### 2.1 Phone: four destinations

`MAP · [domain] · STORY · MENU`, bottom bar in portrait (64 dp of targets plus a 16 dp gesture inset), left rail in
landscape.

| Destination | What it holds | Replaces |
|---|---|---|
| **MAP** | Quick pins; view (2D/3D, North up, Fit world, grid, scale bar); the view field, all 43 analysis fields in their six groups, with opacity; quick visibility for settlements, ways, landmarks, rivers, labels, political, hillshade; the six Measure modes | today's MAP (tool-options row only) and the Layers popover |
| **Domain tab** — reads **GENERATE**, **CIVIL** or **CARTO** | **Tap** opens the domain sheet. **Hold** (380 ms) opens a fan of the three domains: slide and lift, or tap. **Swipe** the tab sideways to cycle. The sheet title carries a ▾ that opens the same fan, so the hold is never the only way. Three dots above the icon show which domain is current. Session mode hides WORLD, as it hides GENERATE today, so the fan and the swipe then offer CIVIL and CARTO only | GENERATE, MORE ▸ Civilization, and the Window ▸ Left dock door |
| **STORY** | Segments TIME · JOURNEYS · CONFLICTS · NOTES, the desktop Story menu's bands. JOURNEYS is the planner (verdict card, stages, stage detail, Party, Season, Carriage, Route conditions, Results). TIME is the timeline: transport, year and day, the six simulation layers, recorded years, filters, collapse / recovery | PLAN, MORE ▸ Story, MORE ▸ Simulation |
| **MENU** | The eight desktop menus, **row for row** in the same order and under the same names (File · Edit · Assets · World · Story · Preferences · Window · Help), plus Quick and Status. A submenu of only radio rows is drawn inline as chips, as today's phone does | MORE's bespoke root and its "Not on the MORE list" band |

**The domain sheet** is the desktop dock for the node, not a phone copy: the node's chips (WORLD: Generate · Sculpt;
CIVIL: Landmarks · Factions · Routes & ways · Journeys; CARTO: Style · Layers · Labels · Icons), the domain's tool
palette, then **every** category `RAIL_NODES` gives the node, as accordions. WORLD ▸ Generate gets Generate,
Planet, Geology, Hydrology, Climate, Ecology and World data, with the ten stages inside them and their parameters.
CIVIL ▸ Factions gets its ten categories. CARTO ▸ Style gets Style, Relief & light, Colours and Feature style.
CIVIL ▸ Journeys opens STORY ▸ Journeys rather than a second planner.

**App bar:** the world pill (opens MENU ▸ File), Undo (tap; hold 520 ms for history, with Redo), Search. The `⋮`
overflow goes, because MENU covers it.

**The armed tool** has one home on a phone: a strip above the sheet showing the tool's name, a hint, its main options
as chips, and Commit / Discard where the tool drafts. Arming drops the sheet to peek. Tapping the strip's tool chip
opens the full options at the top of the domain sheet; holding it opens the tool fan.

**Long-press on the map** drops a sample pin and opens a card. The card leads with four to six actions for the
current domain (WORLD: Sculpt here, Cross-section from here, Paint with this biome; CIVIL: Drop settlement, Start
way, Start route, Info here; CARTO: Add label, Stamp icon, View field, Style preset, Export region), then the ground
readouts. Nothing is dropped for being a submenu: View field and Style preset jump to where those live.

**Windows** (Data manager, Factions, Vault, Travel library, Export, Assistant, Shortcuts …) open as a full-height
sheet on a phone and a centred dialog on a tablet. They keep their implementations; only the frame changes.

**Search** finds settlements, factions, labels, routes and landmarks, and **runs** any menu row, dock section, tool or
generation parameter (a parameter opens its stage, expanded). The prefix keys stay: `s`, `f`, `lb`, `r`, `.`, `l`.

### 2.2 Tablet: the desktop shell with touch sizing

The PC menu bar is unchanged. From 1 180 dp it is shown whole (File · Edit · Assets · World · Story · Preferences ·
Window · Help). Below that it is the shipped tablet bar `☰ · File · World · Story`, with ☰ holding the rest as today.
Dropdowns drill in rather than cascade, with 48 dp rows and accelerators shown for a keyboard. The tool-options bar,
the domain rail (with the active domain's nodes as 48 dp buttons), the left dock, the map, the inspector, the timeline
and the status bar keep their desktop order.

**The docking rule (the fix for the 800 dp strip):** a dock stays docked only while the map keeps at least
**520 dp**. Otherwise it floats over the map as a drawer with a shadow and a pin. A floating dock starts closed, and
closes when the map is pressed. The inspector collapses to a 44 dp edge tab. Press and hold on the map opens the
context card beside the finger, on the side away from the palm (Preferences ▸ Handedness).

### 2.3 New World, kept as shipped, plus auto-populate

File ▸ New world… keeps `new_world_dialog.gd` (NWP-3, Ruling CE) part for part. On the left are the grey-on-blue
land / ocean preview (`PREVIEW_LAND #8f9295`, `PREVIEW_OCEAN #2b4d6b`), carrying `PREVIEW_DISCLAIMER` on its frame;
its stats line; the seed row with Roll and Lock; and the four-slot session history (`HISTORY_SLOTS`). On the right are
Extent (with its two notes), Size (the six `SIZE_PRESETS`, plates scaled from width with − / +, the five
`RESOLUTION_PRESETS`), the derived grid, extent and cell-size panel, Archetype (Classic plus the five), and **More
options** (aspect, grid, sea level, village seeding, the placement model). The window is two columns on a tablet (the
shipped 1 000 × 780 window), stacked on a phone, and two columns in phone landscape from 700 dp, as the shipped card
does. Create reads **Create** on a phone and **Generate world** elsewhere.

**New, at the owner's suggestion: AUTO-POPULATE.** One switch, *Auto-populate the world*, then four that run passes
the shell already has, in order, after the terrain is generated:

| Switch | Runs | Note |
|---|---|---|
| Settlements, factions & territories | CIVIL ▸ Populate ▸ Auto-populate world | uses the placement model in More options |
| Routes | CIVIL ▸ Routes & ways ▸ Generate roads | needs settlements; greyed with that reason when they are off |
| Names & labels | CARTO ▸ Labels ▸ Generate labels | place, region, water and landmark names |
| Landmarks & points of interest | CIVIL ▸ Landmarks ▸ Run landmark pass | 49 types in 6 families; caps as set in Landmarks |

---

## 3. Size classes and the dp rules

Every number is in dp and is re-evaluated on every resize: a foldable that unfolds, a rotated tablet or a resized
window gets the right layout without a restart. Nothing is placed in pixels, and nothing is scaled by the ratio of
the screen to a reference phone.

| Window | Layout | Key measures |
|---|---|---|
| width < 600 | **phone** | bottom bar 80 (64 + 16 inset); app bar 52 under a 28 status inset; sheet peek 76, half 50 % of the space above the bar, full to just under the app bar; floating controls follow the sheet edge and thin out below 250 / 120 dp of free map |
| landscape, height < 500 | **phone landscape** | rail 80; the sheet docks right at 42 % (300–400) or wide (up to 600) |
| width 600–839 | **tablet, medium** (fold, 10″ portrait) | rail 64; both docks float |
| width 840–1 199 | **tablet, expanded** | the left dock docks if the map keeps 520; the inspector floats |
| width 1 200–1 599 | **tablet, large** | both dock when the map keeps 520 (left 340/360, right 312/340) |
| width ≥ 1 600 | **extra-large, 4K-class** | from 1 920 the docks widen to 400 / 380, windows to 760, and the menu bar adds the desktop readouts (grid, CPU, GPU, memory) |
| any | **UI scale** 100 / 125 / 150 / 200 % (Preferences ▸ Touch & gestures) | the shell lays out in dp ÷ scale and draws scaled up; for 4K panels at density 1 and for reading comfort |

**4K.** A 3 840 × 2 160 px panel is 1 920 × 1 080 dp at density 2, 2 560 × 1 440 at 1.5 and 3 840 × 2 160 at 1.
All three are lab presets. The map canvas caps its backing store at about 8 MP, so a 4K window on a high-density
screen does not allocate a 33 MP canvas.

**Touch floor:** every tap target is at least 44 × 44 dp on both axes; most are 48–64. Measured with a script that
walks every visible button, input and tab, clipped by its scroll containers. It ran at 360 × 780, 412 × 892,
780 × 360 (MAP, domain sheet, MENU, Preferences, STORY), at 1 194 × 834 and with the File menu open: zero below the
floor. Sliders are 44 dp tall and keep their − / + steppers as the non-drag alternative.

**Contrast**, computed from the token values: secondary text `#8d9296` on `#15171a` is 5.7 : 1; the light theme's
`#5d615c` on `#fbfaf7` is 6.0 : 1; accent text `#e0a34a` on `#15171a` is 8.1 : 1, and the light accent `#9a5d0c` on
`#fbfaf7` is 5.1 : 1. `#6f7478` (3.8 : 1) is kept off text. A row that cannot run is drawn with its reason as a second
line in `#ebc080`, never only greyed.

---

## 4. Input methods

| Input | Does | Where |
|---|---|---|
| Tap | select (Inspect), or the armed tool's action | map |
| Press and hold, 480 ms (350 / 480 / 700 in Touch & gestures) | sample pin and context card | map |
| Drag; pinch; two-finger turn; double-tap | pan; zoom; rotate (a compass appears, tap for north up); zoom ×1.7 | map |
| Two-finger tap / three-finger tap | undo / redo (switchable) | map |
| Swipe in from the right edge (28 dp, 60 dp travel) | inspector | map |
| Hold the domain tab, slide, lift | switch WORLD · CIVIL · CARTO | bottom bar or rail |
| Swipe the domain tab sideways | next / previous domain | bottom bar or rail |
| Swipe up on a tab | open its sheet full | bottom bar |
| Tab re-tap | close the sheet | bottom bar |
| Drag the sheet handle; tap it | peek · half · full; toggle half and full | sheet |
| Hold the tool chip | tool fan for the node's tools | tool strip |
| Hold Undo, 520 ms | edit history, roll back to any step, Redo | app bar |
| System back | closes, in order: window, fan, history, search, menu, card, layers, armed tool, sheet level, sheet | Android |
| Pen | draws and sculpts; with *Pen draws, finger pans* the finger only pans | tablet |
| Keyboard | every desktop accelerator, plus the tablet's Ctrl+R Run pipeline (Help ▸ Keyboard shortcuts…) | tablet |
| Haptic tick | on holds, fans and snaps (switchable) | all |

---

## 5. Mapping onto the shell (for the refactor)

| Today | Proposed |
|---|---|
| `PHONE_TABS` `map · gen · plan · more` | `map · domain · story · menu`; the domain tab's caption follows the current domain |
| `_compute_layout_mode`: boot-time, aspect < 0.6, `_TABLET_MIN_DP` 900 on device, `_phone_scale` = short ÷ 412 | width classes in dp (§3), re-run on every `size_changed`; no proportional scale; a user UI scale |
| `build_phone_generate` and `_pg_*` (a parallel phone copy of the WORLD dock) | the domain sheet hosts the workspaces' own category builders, the same ones the docks use, so a control has one definition |
| `phone_menu.gd` bespoke root plus "Not on the MORE list" | the root lists the eight menus; `_fill_popup` (which already re-presents the hidden `PopupMenu`s) draws every level |
| `tool_fan.gd` pill plus the per-tool phone bars | one tool strip (name, hint, chips, Commit / Discard) and a tool fan on hold |
| `PhoneMenu.peek_card` dropping `children` / `param` rows | the card keeps every row; a submenu row becomes a jump |
| desktop docks at a fixed width on tablet | the 520 dp docking rule, floating docks with pin, a 44 dp inspector tab |
| `PHONE_DETENT_PEEK` 66, `PHONE_DETENT_HALF_FRAC` 0.46, `PHONE_DETENT_FULL_GAP` 96 | 76 (the header fits a tool line), 0.50, to just under the app bar |
| `new_world_dialog.gd` | kept; AUTO-POPULATE rows added under Archetype |
| `menus.gd` (the PC bar) | unchanged |

## 6. Milestones

| # | Milestone | Bar |
|---|---|---|
| MS-1 | Width classes and dp constants in `DccTheme` / `DccShell`, re-evaluated on resize; UI scale | a probe resizing 360 → 900 → 1 920 dp gets phone → tablet → large without a restart |
| MS-2 | Bottom bar and rail with the domain tab (tap, hold-fan, swipe, ▾ title) | all three domains reachable in one gesture; probe drives the hold and the slide |
| MS-3 | Domain sheet from the dock builders | no control reachable only through Window ▸ Left dock (walk every category, count 0) |
| MS-4 | MENU mirrors the eight menus | every one of the 360 rows reachable in at most three taps from MENU |
| MS-5 | Tool strip, tool fan, context card with domain verbs | every tool armed, used and disarmed from the strip |
| MS-6 | STORY tab | the planner, timeline, conflicts and notes reachable without MENU |
| MS-7 | Tablet docking rule, ☰ collapse at 1 180, 44 dp inspector tab, ≥ 1 920 readouts | the map never under 520 dp with a docked panel at 600–3 840 dp |
| MS-8 | New World AUTO-POPULATE | a new world with all four on equals Generate plus the four passes run by hand |
| MS-9 | Search runs every command | a check, radio or parameter result does what its row does |
| MS-10 | Device passes: the OnePlus 6T, a tablet, a 4K window | the touch floor and the docking rule hold on glass |

Each closes with `cargo test --workspace` unaffected and the touched `.gd` files parse-checked (`MISTAKES.md`).

## 7. Owner questions, with defaults

Q1 to Q4 were answered by **Ruling DT** (owner, 2026-10-31: *"Yes to all four defaults"*, `LARGE_ITEM_RULINGS.md`). Q5 to Q9 are open.

| # | Question | Default | Answer |
|---|---|---|---|
| Q1 | Rename PLAN to STORY, holding Time · Journeys · Conflicts · Notes? | Yes | **Yes** (Ruling DT) |
| Q2 | Replace MORE with MENU, the eight desktop menus row for row? | Yes | **Yes** (Ruling DT) |
| Q3 | The WORLD domain tab's caption: GENERATE (today's word) or WORLD (the rail's)? | GENERATE | **GENERATE** (Ruling DT) |
| Q4 | Auto-populate on by default in New World? | Yes, all four | **Yes, all four** (Ruling DT) |
| Q5 | Two- and three-finger tap for undo and redo on by default? | Yes, with a switch | open |
| Q6 | Drop the app bar's `⋮` (Save · Theme · Close world now live in MENU ▸ File and Preferences)? | Yes | open |
| Q7 | Below 1 180 dp, keep the shipped tablet bar `☰ · File · World · Story`? | Yes | open |
| Q8 | Floating docks close when the map is touched? | Yes, unless pinned | open |
| Q9 | Peek detent 76 dp instead of 66? | Yes | open |

## 8. What the mockup does not settle

- The map is the mockup's own stand-in (noise continents, a priority-flood fill, D8 rivers, placed towns), not the
  engine, and its New World preview stands in for `WorldGen.preview_land_mask`.
- Windows show their frame and main sections, not every control inside them.
- 3D is a toggle label.
- Measured in Chromium only (Playwright), at the sizes listed in §3. Glass is MS-10.
