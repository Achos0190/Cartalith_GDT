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
| **MENU** | The eight desktop menus, **row for row** in the same order and under the same names (File · Edit · Assets · Records · Story · Preferences · Window · Help; Ruling DY renamed World to Records and moved Generate world to File, after New world…), plus Quick and Status. A submenu of only radio rows is drawn inline as chips, as today's phone does | MORE's bespoke root and its "Not on the MORE list" band |

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

The PC menu bar keeps its eight menus and their order. Ruling DY renamed World to Records. From 1 180 dp the bar is
shown whole: File · Edit · Assets · Records · Story · Preferences · Window · Help.

Below 1 180 dp the bar is **`☰ · File · World · Records · Story`** (Ruling DY, PQ6). That amends Ruling DU's Q7,
which kept the shipped `☰ · File · World · Story`.
- **World** keeps only the WORLD domain's rows: Generate world, Reset generation parameters, Reset one stage,
  Pipeline / Sculpt and Seed & size….
- **Records** is the desktop's menu.
- **File** leaves out Generate world on this bar, because World holds it.
- **☰** holds the rest, as today.
- **At 600 dp** the bar's width is a build bar: nothing may overlap.
Dropdowns drill in rather than cascade, with 48 dp rows and accelerators shown for a keyboard. The tool-options bar,
the domain rail (with the active domain's nodes as 48 dp buttons), the left dock, the map, the inspector, the timeline
and the status bar keep their desktop order.

**The tool-options bar holds the armed tool's settings only** (Ruling DZ, 2026-11-01; `PC_SHELL_PROPOSAL.md`
§3.12). With no tool armed, it shows the context name and one hint (*Tap selects · hold opens the cartwheel and
the card*). Generate world, New seed, Center landmasses and Bake ALL & finalize stay in the WORLD dock and the
menus. The quick bar's pins at its right end stay.

**The right dock and the strip, proposed later the same day.** `PC_SHELL_PROPOSAL.md` proposes the desktop's right dock
as a **Context panel** (Inspect · Note · Ask, following the selection or pinned), and the timeline strip in every domain
with CIVIL ▸ Timeline's controls in its expanded row. Because the tablet is the desktop shell, both reach the tablet at
touch size; that is its PQ1 and PQ2, open. The mockup's tablet frame draws them. The tablet's windows stay modal sheets.

**The docking rule (the fix for the 800 dp strip):** a dock stays docked only while the map keeps at least
**520 dp**. Otherwise it floats over the map as a drawer with a shadow and a pin. A floating dock starts closed, and
closes when the map is pressed. The inspector collapses to a 44 dp edge tab.

**The cartwheel stays.** Press and hold on the map for 500 ms opens the cartwheel and the context card together, as
shipped (CM-3/CM-4, `MAP_CONTEXT_SCOPE.md` §7.1). Slide to a slot and lift, or lift in place and tap. The card sits
beside the ring, on the side away from the dominant hand (Preferences ▸ Handedness). §2.4 has the rest, with the quick
bar. *(Corrected 2026-10-31: this paragraph first said the hold opened only the card, which would have dropped the
shipped tablet ring.)*

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

### 2.4 The adaptable quick bar and the cartwheel

Both are shipped, and the refactor keeps both. The quick bar (`CUSTOM_QUICK_BAR_SCOPE.md`, CQ-0 to CQ-4, CQ-6 and
CQ-7 built) is one list of pinned commands drawn in several places, stored per device in
`user://cartalith_settings.cfg` and never in a world. A pin runs the very handler its menu row runs. The cartwheel is
the eight-slot radial ring (`radial_ring.gd`, CM-3 and CM-4 built): its four cardinals are the same in every domain
(Inspect, Measure ▸, Undo, Region) and its four diagonals are the domain's tool families (`MAP_CONTEXT_SCOPE.md`
§5.1). The phone has no cartwheel; it has the thumb fan.

| | Tablet | Phone |
|---|---|---|
| **Quick cluster** | the end of the tool bar, after a rule, as shipped (CQ-2). `DccQuickBar._layout()` already keeps every pin that fits the bar's measured width and moves the rest under `⋯`, so a wider tablet shows more pins with no change; `＋` opens the Quick bar window | **a side drawer** (new): a 44 × 88 dp grip at the middle of the map's edge, on the side away from the dominant hand (left by default), opens a drawer of the pins. Tap the grip or drag it inward to open; tap the scrim, drag the grip back or press system back to close |
| **Under the cartwheel** | the ring dock: up to four chips beneath the ring's caption, as shipped (CQ-4) | no cartwheel; the thumb fan is unchanged |
| **In MENU** | — (the PC menu bar is unchanged) | a Quick section first in MENU, as shipped in MORE (CQ-6, renamed by Ruling DT) |
| **Assistant chip** | first in the cluster and in the dock, only while the assistant is on and a provider is configured (CQ-7) | first in the drawer and in MENU's Quick section, under the same rule |
| **Empty bar** | the `＋` chip alone, as shipped | nothing is drawn, as shipped: no grip until something is pinned or the Assistant chip is live |

The drawer replaces CQ-6's chip row in the MAP sheet: the new MAP sheet is view, fields and layers only. The grip is a
visible tab, not an edge swipe, because Android's gesture navigation takes swipes that start at a screen edge as
Back. A tap on the grip is never taken. In phone landscape the grip sits beside the 80 dp rail.

The cartwheel's touch geometry is the shipped one (`radial_ring.gd::_apply_touch_scale()`): ring radius about 141 dp,
slots about 92 dp, a dead zone of about 34 dp and a disc of about 205 dp radius, which slides inward near a screen edge
rather than clipping. The dock's chips are about 86 × 61 dp. The mockup draws all of this at those figures.

### 2.5 The phone map's controls (DS-13)

**What this is.** The design for `GUI_GAP_REGISTER.md` §57's DS-13, the row of map controls on a touch screen, written
2026-11-01 for the owner's approval. The owner answered three of its five questions the same day (Ruling EB items 2
to 4: DS13-Q3, Q4 and Q5, below). DS13-Q1 and Q2 await the owner; the research for them is done
(`MAP_INTERACTION_RESEARCH.md`, Ruling EB item 1). It is drawn and working in the mockup (`Live.dc.html` at phone or
tablet size) and checked by the mockup test `ds13.js`. That test covers phone portrait, phone landscape and tablet,
each right- and left-handed, then the desktop's zoom step and the double-tap and thumb zoom on fresh pages: 204 checks
(`grep -c '^ok '` on its output, 2026-11-01). Captures are `ds13-*.png` in
`cartalith-native/docs/evidence/mobile-shell-2026-10-31/`. Nothing in `cartalith-native/` was changed.

**What ships today, read at the symbols.**
- **The row.** `viewport_host.gd::_build_navpad()` builds it on every touch build (`if not _touch: return`), phone
  and tablet alike. It is one bottom row, from left to right:
  - the zoom rocker, − and +, each a call to `zoom_step()` by `ZOOM_BUTTON_STEP` 1.35 about the centre;
  - ✋, `_pan_btn`, a latch (`set_pan_mode()`);
  - ⟳, `view_fill`, which calls `reset_view()`: fit the world and clear the latch;
  - on the phone only, the `3D`/`2D` text pill, `_view3d_btn`.

  This is the owner's 2026-09-05 board, `design/proposed-2026-09-05-round2/PhoneViewportControls.dc.html`, and it is
  built; the file's own comments say "a bottom row since DS-13".
- **Placement** (`_apply_safe_insets()`). The row always sits on the right, `NAVPAD_EDGE` 12 dp in. It sits
  `NAVPAD_GAP` 10 dp above the sheet, and above the larger of the coordinate readout and the tool pill
  (`_pill_reserve()`). It hides when that band is too small. Handedness is not read.
- **Size.** Every target is `NAVPAD_HIT` 44 dp. `_apply_touch_scale()` sizes every button it finds to one size.
- **Gestures.**
  - One finger pans only while ✋ is on: `_input`'s left-button branch is `MOUSE_BUTTON_LEFT and _pan_mode`. The
    magnify branch's own comment says single-finger pan is deliberately not wired on touch: the reference gives the
    finger to the tool ("one finger keeps painting/drawing", HTML line 13988).
  - Two fingers pan (`InputEventPanGesture`) and pinch-zoom (`InputEventMagnifyGesture`).
  - There is no double-tap and no map rotation. `phone_menu.gd::GESTURES_ABSENT` says so for double-tap, and
    `GESTURES` says "This build has no map rotation".
- **Left open by the 2026-09-05 board** (its NOT SETTLED list):
  - reach for a left-handed user;
  - an 18 dp overlap with the undo chip at 360 dp;
  - `_apply_touch_scale()` sizing each child.

**What each control is for**, once §4's approved gestures exist (drag pans, pinch zooms, two fingers turn, double-tap
zooms ×1.7) and thumb zoom with them (a double-tap held and slid zooms continuously, Ruling EB item 3):

| Control | Does | Why it stays beside the gestures |
|---|---|---|
| ⌖ Recentre (the shipped `view_fill` glyph; the mockup draws ⌖) | `reset_view()`: fits the whole world, north up once rotation exists, and clears the latch. In 3D, from 3D-2, it frames the world (Ruling EB item 4) | No gesture gets you back from a lost view in one tap |
| − Zoom out | `zoom_step(1 / 1.35)` about the centre; Ruling EB item 2 keeps the step. In 3D, from 3D-2, it moves the camera out (item 4) | The one zoom-out that works in one tap with every tool armed. Thumb zoom also zooms out one-handed (slide up), but what it does while a drawing tool is armed waits on DS13-Q1. Pinch needs two fingers, and a two-finger tap is undo (Ruling DU Q5) |
| + Zoom in | `zoom_step(1.35)` about the centre. In 3D, from 3D-2, it moves the camera in (Ruling EB item 4) | It pairs with −. It is one-handed, and it serves anyone who cannot pinch; the double-tap and thumb zoom act at the finger instead |
| ✋ Pan latch | While a **drawing** tool is armed, one finger pans instead of drawing. Tap again to give the finger back | Drawing tools own the one-finger drag, as the reference and `_input` keep it. With a tap-only tool a drag already pans, so the latch would do nothing there and is not drawn |
| 2D / 3D | **leaves the row**, for MAP ▸ View (§2.1) | Ruling EA 14 calls the phone pill interim, with MAP's view row as its home. `Window ▸ 3D view` stays on every form, and the tablet never had the pill |

**Behaviour.**
- **Tap-only tools** (Inspect, Measure, Settlement, Label, Icon, Conflict, Route): a one-finger drag that drifts past
  `map_overlay.gd::_TOUCH_SLOP` (28 px, about 10 dp) pans the map. A tap still selects or acts, and a hold still
  opens the card. The overlay already withholds a touch press until the gesture says what it is. A drift with a
  tap-only tool becomes a camera pan, rather than a press released to the tool.
- **Drawing tools** (the sculpt brushes, WORLD paint, colour paint, Region, Territory, Way, Lasso): one finger
  draws, as it does today.
  - ✋ appears at the inner end of the row; while it is on, one finger pans.
  - It clears when a tool is armed or disarmed (new), and on ⌖ (as `reset_view()` already does).
  - With *Pen draws, finger pans* on, ✋ is not drawn: the finger already pans.
- **Two fingers** always pan, pinch and turn; a two-finger tap undoes.
- **Thumb zoom** (Ruling EB item 3; DS13-Q3, *"Add it"*):
  - **The double-tap comes with it.** §4's double-tap is designed but not built, so the two are built together. A
    second tap within 300 ms and 34 px of the first zooms ×1.7 at the finger over 240 ms
    (`design/dcc-environment-2026-08-31/spec/06-phone.md` §7). That spec also holds a single tap back 240 ms, so its
    two figures overlap: a second tap between 240 and 300 ms would arrive after the first had acted. The mockup
    therefore counts a second press only while the first tap is still waiting, so one pair of taps never both acts and
    zooms. The build sets both figures on glass (MS-10).
  - **Double-tap, hold, slide.** The second tap's finger stays down and slides past the touch slop, and the map zooms
    continuously about the point it landed on. **Sliding down zooms in and sliding up zooms out**, "Google Maps style"
    in the owner's words. `MAP_INTERACTION_RESEARCH.md` §2 finds that direction in the Google Maps SDK for Android, in
    Android's own quick scale (`ScaleGestureDetector`) and in most of the map apps it read. Apple Maps documents the
    reverse: drag up to zoom in. Lifting ends the zoom. Lifting without sliding is the plain double-tap.
  - **The second press's hold belongs to thumb zoom.** However long it is held, it opens no card. A press with no
    tap before it still opens the card at the shipped hold (480 ms phone, 500 ms tablet; §4).
  - **Undo is untouched.** Thumb zoom is one finger. A second finger landing hands the view to pinch, and a
    two-finger tap stays undo (Ruling DU Q5), even straight after a tap.
  - **With a drawing tool armed: not settled.** Ruling EB item 3 requires that thumb zoom collide neither with undo
    nor with a drawing tool's strokes, and settles the drawing tool's case with item 1's answer (DS13-Q1 and Q2),
    which awaits the owner. `MAP_INTERACTION_RESEARCH.md` §6 puts two findings to the owner; they are research, not
    answers:
    - Krita's guard: once two fingers have been down, a drawing tool ignores the rest of that touch;
    - thumb zoom kept off while a placing or drawing tool is armed, except while ✋ is on.

    §6 raises the same question for tools whose tap places something (Settlement, Label, Icon, Conflict, a Route
    waypoint, a Measure point): a double-tap would place before it could zoom. Android's own answer is to hold the
    tap back for up to 300 ms.
  - **The mockup's placeholder, which decides nothing.** While a drawing tool is armed, both gestures are off and
    labelled "pending DS13-Q1": in the map's accessible name, and in MENU ▸ Preferences ▸ Touch & gestures. A
    double-tap is then two taps for the tool, and a slide draws. With a tap-only tool, a tap waits 240 ms for a second
    tap before it acts (`06-phone.md` §7), so a double-tap never places.
  - **Where.** Every touch build: phone and tablet. The desktop with a mouse is unchanged, because the hold-slide is
    a touch gesture.
- **The latch's toast stays** (`_announce_pan_mode()`, 4.5 s), and no toast may cover the row. On the tablet, toasts
  stack 20 dp above it; the phone's are at the top.

**Anchoring.**
- **Where.** The bottom of the map, on the dominant hand's side (`Preferences ▸ Handedness`, `DccSettings` `hand`,
  default right). It sits `NAVPAD_EDGE` 12 dp in from that edge and 12 dp above whatever ends the map below it: the
  bottom bar, the sheet at its current detent, or the tool strip. These are the same insets
  (`phone_content_insets()`, `_pill_reserve()`) that place it today.
- **Order.** From the outer edge: ⌖, then − | +, then ✋. ✋ is the only member that comes and goes. At the inner end
  its arrival moves nothing else: ⌖ stays exactly where the thumb learned it (the test checks this to 2 px). The row
  mirrors with Handedness.
- **The scale bar** takes the other side. The coordinate readout stays under the row, as its floor today.
- **The quick-drawer grip** (§2.4) is already on the side away from the dominant hand, at the middle of the edge, so
  the two never meet.
- **Landscape.** The same row, at the bottom of the map. On the right it clears the side sheet; on the left it
  clears the 80 dp rail, each by 12 dp.
- **When it hides.** It hides when the band above the sheet is shorter than the row and its gaps, which is today's
  `_navpad.visible` test. That includes the full detent.
- **Why the bottom.** The canvases draw zoom top-right (§57's anchoring note). The owner's ruling of 2026-09-05 is
  newer and moved it to the bottom for thumb reach, so the bottom stands.

**Sizes.** Every target is 44 × 44 dp (`NAVPAD_HIT`); the rocker is 88 × 44, two 44 dp halves with a 1 px hairline
inset 11 dp (`NAVPAD_RULE_INSET`). Gaps are 10 dp (`NAVPAD_GAP`) and there is one glyph size. `_apply_touch_scale()`
as shipped gives every child one size, and the design asks for nothing else.

**Colours.** These are `_navpad_paint()`'s tokens as shipped, quoted from `dcc_theme.gd`; nothing is new.

| Part | Token | Dark | Light |
|---|---|---|---|
| Pill fill | `panel` at `NAVPAD_ALPHA` 0.92 | `#121314` at 0.92 | `#fbfaf7` at 0.92 |
| Pill edge, rocker hairline | `line` | `rgba(255,255,255,0.10)` | `rgba(0,0,0,0.14)` |
| Glyph | `text` | `#c8cbcd` | `#23241f` |
| Latch on: fill | `panel` at 0.92 blended with `accent_wash_2` | wash `rgba(224,163,74,0.16)` | wash `rgba(164,101,15,0.16)` |
| Latch on: edge and glyph | `accent` | `#e0a34a` | `#a4650f` |

The mockup carries these as `--navpill`, `--navarm` and `--navacc`. The mockup's own base tokens drift from
`DccTheme` in three places: dark `--pan` is `#15171a`, light `--dim` is `#5d615c` and light `--acc` is `#9a5d0c`.
The row does not use them.

**The tablet.** `_build_navpad()` is gated on touch, not on the phone, so a touch tablet builds the same row, and
DS-13 is one design for both.
- **Where.** The row sits at the bottom of the map, between the docks, on the dominant hand's side. It is 12 dp in
  from the map's edge and 14 dp above the map's bottom edge.
- **What differs.** There is no 3D in the row (`Window ▸ 3D view`, as 3D-1 ships), and toasts stack above the row.
- **The desktop** is unchanged: with a mouse no row is built, and the wheel, middle-drag and Space-drag remain. The
  mockup's desktop frame still draws a pointer column (3D, ⌖, +, −) that the shipped desktop does not build. That is
  outside DS-13.

**What it replaces in `viewport_host.gd`.**
1. **`_build_navpad()`.**
   - The order becomes ⌖, rocker, ✋, from the outer edge, with the direction taken from Handedness.
   - `_view3d_btn` goes, with its `DccTheme.is_phone()` branch, `_sync_view3d_btn()` and its two callers; 3D is
     reached from MAP ▸ View (MS-5).
   - After this, no phone-only branch is left in the row.
2. **`_apply_safe_insets()`.**
   - For a left hand, the row's x is `maxf(l, NAVPAD_EDGE)`.
   - The scale label and rule (`_scale_label`, `_scale_rule`, bottom-left today) move to the right, and the
     coordinate readout moves with the row.
3. **✋'s visibility.** It follows the armed tool: drawn only while that tool draws and *Pen draws, finger pans* is
   off. The tool's kind is asked of the shell by capability, as `_pill_reserve()` asks for the pill.
4. **`set_pan_mode(false)` on arm and disarm**, beside the call `reset_view()` already makes.
5. **The one-finger pan for tap-only tools.**
   - It goes in `_input` and the overlay's withheld press: past `_TOUCH_SLOP`, a tap-only tool's drag pans.
   - It ships **in the same change** as item 3. No build may hide ✋ before the drag pans, or the phone loses its
     only one-finger pan.
6. **The double-tap and thumb zoom** (§4's double-tap, approved and not built; thumb zoom, Ruling EB item 3) are new
   branches beside the magnify branch, built together.
   - Their behaviour with a drawing tool armed lands with DS13-Q1's answer.
   - `phone_menu.gd`'s `GESTURES_ABSENT` double-tap row (*"No double-tap handler exists on the map"*) becomes a
     `GESTURES` row in the same change, with thumb zoom beside it.
7. **The 3D camera** (Ruling EB item 4; DS13-Q4, *"Yes, same buttons"*).
   - From 3D-2, `zoom_step()` and ⌖'s `reset_view()` ask `is_3d()` first. While 3D shows, − and + move the 3D camera
     out and in, and ⌖ frames the world (`3D_VIEWPORT_SCOPE.md` §3.9). 3D-2 sets the camera's step.
   - Today the row already sits over the 3D view: `ensure_view3d()` puts the view under `_navpad`. But neither
     function asks `is_3d()`, so the buttons move the hidden 2D camera.

**Rejected, and why.** Each of §57's refutations is answered, then the board's open items:
- **"✋ is a tool toggle, not navigation; remove it"** (§57, 1st). It is navigation. It stays for drawing tools, and
  for tap tools its job passes to the drag itself, in the same change (item 5).
- **"Zoom moves to MORE ▸ VIEW ▸ Zoom"** (§57, 2nd). No such place exists. Zoom stays in the row; MAP ▸ View takes
  only 2D/3D, which the approved MAP sheet already lists.
- **"Phone only; the desktop and tablet row is untouched"** (§57, 3rd). There is one row for every touch build. The
  tablet is designed here, and the desktop has no row.
- **The three colour equalities** (§57, 4th). Every colour above is a `DccTheme` token read at `_navpad_paint()`,
  with its value quoted from `dcc_theme.gd`.
- **Top-right, as the canvases draw it** (§57, anchoring). The 2026-09-05 ruling is newer and chose the bottom.
- **A 48 dp FAB over 44 dp pills, with two glyph sizes** (§57, sizing). There is one size and one glyph size, which is
  what `_apply_touch_scale()` keeps.
- **The board's open items:**
  - Left-handed reach: the row now mirrors.
  - The 360 dp overlap with the undo chip: gone, because Undo moved to the app bar (§2.1).
  - Per-child sizing: the rocker is two 44 dp buttons, and nothing needs a second size.
- **Also rejected:**
  - a column up the edge, which is out of thumb reach (the 2026-09-05 ruling);
  - a zoom slider, for which there is no room; pinch and thumb zoom already zoom continuously, and the double-tap
    zooms in steps;
  - ✋ always drawn, because with a tap tool it would be a control that changes nothing;
  - a hold on ✋ for a momentary pan, because a hold already opens the card, and a second meaning on one 44 dp target
    is a collision.

**Milestone fit.**
- **MS-5** (the tool strip, the fan and the card). The latch follows the armed tool and the strip sits under the row,
  so items 1 to 6 land there.
  - Items 3 and 5 land together, and wait on DS13-Q1 and Q2.
  - Item 6, the double-tap and thumb zoom, lands there too. Its behaviour while a drawing tool is armed waits on
    DS13-Q1.
- **3D-2** (`3D_VIEWPORT_SCOPE.md`) builds item 7 with the 3D camera. It needs nothing from MS-5, because the row
  ships today.
- **MS-7** checks the tablet's anchoring against the docking rule.
- **MS-10** passes it on the 6T and a tablet, in both hands, the double-tap and thumb zoom included.
- **Bars:**
  - every target at least 44 × 44 dp at every density;
  - no overlap with the grip, scale bar, sheet, strip, bottom bar or a toast, at 360 and 412 dp, in portrait and
    landscape, in both hands;
  - ⌖ not moving when ✋ comes and goes;
  - − and + step by exactly 1.35 (Ruling EB item 2);
  - with Inspect, a drag past the slop pans while a tap still selects and a hold still opens the card;
  - with a drawing tool, the drag draws, and with ✋ on it pans;
  - a double-tap zooms ×1.7 at the finger;
  - double-tap, hold, slide zooms continuously about the finger, down in and up out, and its held press opens no
    card;
  - a tap followed by a two-finger tap still undoes, and a pinch still zooms;
  - with a drawing tool armed, thumb zoom's bar is set with DS13-Q1's answer. The mockup's placeholder bar: neither
    gesture changes the zoom;
  - from 3D-2, − and + move the 3D camera and ⌖ frames the world.

  `ds13.js` holds all of these in the mockup today except the last, because the mockup draws no 3D camera (§8).

**Owner questions, with defaults.**
1. **DS13-Q1. Is ✋ drawn only while a drawing tool is armed?** This changes the 2026-09-05 board, which always draws
   it. Default: yes.
   **Awaiting the owner; the research is done (Ruling EB item 1).** `MAP_INTERACTION_RESEARCH.md` §6 supports the
   default. It adds two findings the default does not cover: Krita's guard, and how thumb zoom behaves with a placing
   or drawing tool (see Behaviour).
2. **DS13-Q2. Does one finger pan with a tap-only tool?** This is §4's approved "Drag: pan". It is asked by name
   because the shipped `_input` deliberately does not do it. Default: yes, with a drawing tool keeping the finger.
   **Awaiting the owner; the research is done (Ruling EB item 1).** `MAP_INTERACTION_RESEARCH.md` §6 supports the
   default: every navigation map it read pans with one finger.
3. **DS13-Q3. One-finger continuous zoom** (double-tap, then drag). Default: not in v1.
   **Answered (Ruling EB item 3), against the default: "Add it".** Double-tap, hold and slide, Google Maps style, so
   sliding down zooms in. It is built together with the plain double-tap. How it behaves while a drawing tool is armed
   is settled with DS13-Q1's answer (Behaviour, *Thumb zoom*).
4. **DS13-Q4. Once 3D-2 draws terrain, does the row drive the 3D camera?** That would be − and + to zoom and ⌖ back
   to the top-down start. Default: yes, with the same row and meanings; 3D-2 sets the camera's steps.
   **Answered (Ruling EB item 4): "Yes, same buttons".** From 3D-2, − and + move the camera in and out and ⌖ frames
   the world, as the ruling records it (item 7 above).
5. **DS13-Q5. The zoom step.** Keep `ZOOM_BUTTON_STEP` 1.35 (reference 13464), or match double-tap's ×1.7. Default:
   keep 1.35.
   **Answered (Ruling EB item 2): "Keep 1.35×".** `viewport_host.gd::ZOOM_BUTTON_STEP` stays 1.35. The mockup's
   1.6 goes. It was the desktop frame's pointer column (`hZoomIn` / `hZoomOut`); the touch rocker already stepped
   1.35 (`MC_ZOOM_STEP`). Every button zoom in the mockup now steps 1.35.

---

## 3. Size classes and the dp rules

Every number is in dp and is re-evaluated on every resize: a foldable that unfolds, a rotated tablet or a resized
window gets the right layout without a restart. Nothing is placed in pixels, and nothing is scaled by the ratio of
the screen to a reference phone.

> **Correction, 2026-10-31, found while writing `MOBILE_SHELL_MS1_SPEC.md`.** The table's first row puts the phone line
> at 600 dp of width. That overturns an owner answer this proposal did not cite:
> `design/dcc-environment-2026-08-31/BUILD_ANSWERS.md` §2.3 (2026-08-31) puts the line at **900 dp of shortest
> width**, because at 800 dp the desktop-parity shell's 48 dp rail and 400 dp dock leave a 352 dp map, narrower than
> the dock beside it.
> None of Q1 to Q9 asked to move it, so Rulings DT and DU do not cover it, and the 900 dp answer stands. The table is
> this proposal's design, not a decision. Where the line goes is MQ-1 of `MOBILE_SHELL_MS1_SPEC.md`, whose §2.2 also
> records that the shipped code applies §2.3 only to screens below aspect 0.6.

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
| Press and hold: 480 ms on a phone and 500 ms on a tablet, the shipped `_PHONE_TOUCH_HOLD_MS` and `_TOUCH_HOLD_MS` (`map_overlay.gd`); Touch & gestures offers Short (350) and Long (700) beside them | phone: sample pin and context card · tablet: the cartwheel and the card together | map |
| Hold, slide to a slot, lift | tablet: picks a cartwheel slot; lifting on a `▸` slot opens its sub-ring | map |
| Tap or drag the quick grip | phone: the quick drawer of pinned commands | map edge |
| Drag; pinch; two-finger turn; double-tap | pan; zoom; rotate (a compass appears, tap for north up); zoom ×1.7 at the finger, built with thumb zoom (Ruling EB item 3) | map |
| Double-tap, hold, slide (thumb zoom) | zoom continuously about the finger: slide down to zoom in, up to zoom out (Ruling EB item 3, "Google Maps style"; `MAP_INTERACTION_RESEARCH.md` §2). The held press opens no card. With a drawing tool armed: settled with DS13-Q1 (§2.5) | map |
| Two-finger tap / three-finger tap | undo / redo (switchable) | map |
| Swipe in from the right edge (28 dp, 60 dp travel) | inspector, where the platform leaves the edge to the app (with Android's gesture navigation it is Back) | map |
| Hold the domain tab, slide, lift | switch WORLD · CIVIL · CARTO | bottom bar or rail |
| Swipe the domain tab sideways | next / previous domain | bottom bar or rail |
| Swipe up on a tab | open its sheet full | bottom bar |
| Tab re-tap | close the sheet | bottom bar |
| Drag the sheet handle; tap it | peek · half · full; toggle half and full | sheet |
| Hold the tool chip | tool fan for the node's tools | tool strip |
| Hold Undo, 520 ms | edit history, roll back to any step, Redo | app bar |
| System back | closes, in order: window, fan, quick drawer, cartwheel, history, search, menu, card, layers, armed tool, sheet level, sheet | Android |
| Pen | draws and sculpts; with *Pen draws, finger pans* the finger only pans | tablet |
| Keyboard | every desktop accelerator, Ctrl+R Generate world among them (one label since Ruling DX; on the PC's File menu since Ruling DY), listed in Help ▸ Keyboard reference… | tablet |
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
| `QuickBarPhoneRow` mounted in the tool sheet by `set_phone_quick_row()` (CQ-6) | a side drawer on a grip (§2.4), fed by the same pin store and `QuickBarPhone`; MENU's Quick section stays |
| `radial_ring.gd`, `context_broker.gd`, `quick_bar_ring.gd` (the tablet cartwheel and its dock), and the CQ-2 cluster on the tablet tool bar | unchanged (§2.4) |
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
| MS-5 | Tool strip, tool fan, context card with domain verbs; the map's control row, DS-13, with the double-tap and thumb zoom (§2.5) | every tool armed, used and disarmed from the strip; §2.5's bars for the row |
| MS-6 | STORY tab | the planner, timeline, conflicts and notes reachable without MENU |
| MS-7 | Tablet docking rule, ☰ collapse at 1 180, 44 dp inspector tab, ≥ 1 920 readouts | the map never under 520 dp with a docked panel at 600–3 840 dp; the cartwheel, its dock and the cluster still work at every tablet size |
| MS-8 | New World AUTO-POPULATE | a new world with all four on equals Generate plus the four passes run by hand |
| MS-9 | Search runs every command | a check, radio or parameter result does what its row does |
| MS-10 | Device passes: the OnePlus 6T, a tablet, a 4K window | the touch floor and the docking rule hold on glass |
| MS-11 | The phone's quick drawer (§2.4), fed by the CQ pin store; CQ-6's sheet row retired | every pin runs from the drawer; the grip is at least 44 × 88 dp and draws nothing while the bar is empty; tap and drag open it; scrim, drag and system back close it |

Each closes with `cargo test --workspace` unaffected and the touched `.gd` files parse-checked (`MISTAKES.md`).

The owner scheduled MS-1 first on 2026-10-31, as documentation for now (Ruling DV). Its build specification, with
stages, numeric bars and its own owner questions, is `MOBILE_SHELL_MS1_SPEC.md`.

## 7. Owner questions, with defaults

**Every question is answered.** The rulings are in `LARGE_ITEM_RULINGS.md`:
- **Q1 to Q4**, by Ruling DT (owner, 2026-10-31: *"Yes to all four defaults"*).
- **Q5 to Q9**, by Ruling DU (owner, 2026-10-31: *"Yes to Q5 through Q9 too"*).
- **Q13**, by Ruling DX (2026-11-01). It was added by that day's menu parity check.
- **Q10 to Q12**, by Ruling DY (2026-11-01). They were added on 2026-10-31, after the owner asked whether the
  adaptable quick bar and the cartwheel were accounted for (§2.4).

Every default stood except Q13's. Ruling DY's PQ6 (`PC_SHELL_PROPOSAL.md`) later amended Q7's answer: below 1 180 dp the
bar is `☰ · File · World · Records · Story` (§2.2).

| # | Question | Default | Answer |
|---|---|---|---|
| Q1 | Rename PLAN to STORY, holding Time · Journeys · Conflicts · Notes? | Yes | **Yes** (Ruling DT) |
| Q2 | Replace MORE with MENU, the eight desktop menus row for row? | Yes | **Yes** (Ruling DT) |
| Q3 | The WORLD domain tab's caption: GENERATE (today's word) or WORLD (the rail's)? | GENERATE | **GENERATE** (Ruling DT) |
| Q4 | Auto-populate on by default in New World? | Yes, all four | **Yes, all four** (Ruling DT) |
| Q5 | Two- and three-finger tap for undo and redo on by default? | Yes, with a switch | **Yes, with a switch** (Ruling DU) |
| Q6 | Drop the app bar's `⋮` (Save · Theme · Close world now live in MENU ▸ File and Preferences)? | Yes | **Yes** (Ruling DU) |
| Q7 | Below 1 180 dp, keep the shipped tablet bar `☰ · File · World · Story`? | Yes | **Yes** (Ruling DU); amended by Ruling DY PQ6 to `☰ · File · World · Records · Story` |
| Q8 | Floating docks close when the map is touched? | Yes, unless pinned | **Yes, unless pinned** (Ruling DU) |
| Q9 | Peek detent 76 dp instead of 66? | Yes | **Yes** (Ruling DU) |
| Q10 | On the phone, move the quick pins from the MAP sheet's row (CQ-6) to a side drawer on a grip, keeping MENU's Quick section? | Yes | **Yes: a side drawer on a grip** (Ruling DY) |
| Q11 | Put the drawer's grip on the side away from the dominant hand, so the holding hand's thumb reaches it (left by default, following Preferences ▸ Handedness)? | Yes | **Yes: away from the dominant hand** (Ruling DY) |
| Q12 | On a tablet, keep the quick bar and the cartwheel exactly as shipped: the tool bar's cluster (as many pins as fit, the rest under `⋯`, so a wider tablet shows more), the cartwheel on a 500 ms hold, and its dock of four? | Yes | **Yes, as shipped** (Ruling DY) |
| Q13 | **Asked by name: it reopens Ruling DT's answer to Q2.** DT took MENU as the desktop menus *"row for row, same order, same names"*, and the mockup does that. Draft `05_phone_menu_root.gd` instead opens the shipped phone screens for File, Assets, Preferences and Help. Those reorder rows, add some (recent worlds inline, the asset families, Runtime and Engine), and rename others (Interval; screen titles Project, Asset library, Help & about). World also gains an "On the phone" band. Found by the menu parity check of 2026-11-01 (64 rows; `cartalith-native/docs/evidence/menu-parity-2026-11-01/REPORT.md`, P1). Keep DT as written? | Yes, once every phone-only row has a home in a desktop menu; Simulation has none yet | **Follow the mockup** (Ruling DX): MENU repeats the desktop menus row for row; each phone-only row gets a home or a stated drop |

## 8. What the mockup does not settle

- The map is the mockup's own stand-in (noise continents, a priority-flood fill, D8 rivers, placed towns), not the
  engine, and its New World preview stands in for `WorldGen.preview_land_mask`.
- Windows show their frame and main sections, not every control inside them.
- 3D is a toggle label, so the row's 3D meanings (Ruling EB item 4) are not drawn; the row still moves the 2D map.
- Thumb zoom with a drawing tool armed is a placeholder, off and labelled "pending DS13-Q1" (§2.5).
- The Assistant chip appears while Preferences ▸ Assistant is on. The shipped chip also needs a configured provider
  (CQ-7).
- Measured in Chromium only (Playwright), at the sizes listed in §3. Glass is MS-10.
