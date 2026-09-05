extends Control
class_name DccShell

## The DCC editor frame (`DCC_SHELL_SPEC.md` §1-§3, §6, §7, §9, §11).
##
## Six regions in DOM order plus the two bars that bracket them, built in code
## rather than in a `.tscn` so that the geometry table in §1 is readable as a
## table here, and so five workspace modules can attach without five people
## editing one scene file.
##
## This script owns the *frame* only: region sizes, dock collapse, which
## workspace is active, and the status bar. It owns no world state and calls no
## engine method -- `EngineBridge` does that, and the workspaces read it. The
## load-bearing rule from `UI_SHELL_DESIGN.md`: the top bar is about the
## program, the map is about the world.

signal workspace_changed(id: String)
signal tool_changed(tool_id: String)
signal phone_insets_changed()  ## §13: fires whenever a rotation changes where
	## the phone chrome's edges sit, so `ViewportHost`'s own corner chrome
	## (built and owned by `app.gd`, not this file) can re-read `phone_content_insets()`.

# -- Workspaces (§3) ----------------------------------------------------------
#
# Five domains on the rail. Generate / Simulate / Render / View are *not*
# menus in this shell -- that is the structural change this revision makes, and
# the reason the menu bar below is seven program menus and nothing else.

## A fourth key, `subnodes`, carried each domain's dock sub-structure for
## SH-01's expanded rail. **Removed 2026-08-24 with the expansion itself** --
## see `_build_rail()`'s header for why the rail no longer has two states, and
## therefore nothing left to list.
##
## **Domain merge (2026-08-20, owner instruction: "Infra can be dropped as a
## name and can be absorbed by civil... And render into carto.")** Five
## domains become three. INFRA's five subjects (Roads/Rivers/Ports/Trade/
## Logistics) and its Way/Route tools now live under CIVIL, via
## `civilization_workspace.gd` composing an `InfrastructureWorkspace` instance
## into its own dock rather than that class getting its own rail button.
## RENDER's one subject (Terrain appearance) now lives under CARTO the same
## way, via `cartography_workspace.gd` composing a `RenderWorkspace` instance.
## Nothing was deleted -- both classes still exist, still own their own
## category builders and tool click handlers, they are just reached through a
## different rail button now. See `DCC_SHELL_SPEC.md`'s own correction notice
## for the full disclosure and `GUI_GAP_REGISTER.md` §6.11-§6.14.
const DOMAINS: Array = [
	{"id": "world", "label": "World", "rail": "WORLD", "icon": "domain_world",
		"subtitle": "Terrain, hydrology, climate and ecology"},
	{"id": "civilization", "label": "Civilization", "rail": "CIVIL", "icon": "domain_civ",
		"subtitle": "Settlements, factions, provinces, trade, roads, sea routes and journeys"},
	{"id": "cartography", "label": "Cartography", "rail": "CARTO", "icon": "domain_carto",
		"subtitle": "Layers, styles, labels, annotation and terrain appearance"},
]

# -- The node tree behind the rail (stage 2, 2026-08-31) ----------------------
#
# `design/dcc-environment-2026-08-31/Cartalith DCC Environment.dc.html:1823-1824`
# builds this list literally; the labels, their order and their `mode` strings
# below are that array transcribed, ampersands and sentence case included. The
# prototype renders it flat -- three `{t:'h'}` headers interleaved with ten
# `{t:'n'}` nodes -- and so does `_build_rail_expansion()`, which is why this is
# one flat array and not a nested one.
#
# **`mode` selects; `shows` gates. Most nodes have no `shows`.**
#
# Owner ruling, `LARGE_ITEM_RULINGS.md` 2026-09-05 item 2: restructure the left
# dock to `04-left-dock.md` §3's blocks. Read in full, §3 gates far less than a
# one-line summary of it suggests, and this table is that reading written down:
#
#   - **§3 point 3** -- *"CIVIL always shows all four category headers, with
#     exactly one body expanded between them"*. The headers are interleaved with
#     the bodies; the shape is an accordion, not a tab strip. So no CIVIL node
#     carries `shows`, and every CIVIL category header stays on screen in every
#     CIVIL mode, exactly as it does today.
#   - **§3 point 2** -- the four CARTO nodes render *one* dock (`ldCarto` and
#     `ldRender` are both plain `domain==='CARTO'`). BUILD_ANSWERS §2.1 then
#     gave those four nodes four real destinations *"accordion headers in the
#     left dock ... same grammar as the CIVIL categories"*, which is the newer
#     document and the one this shell follows -- but neither statement hides a
#     header, so no CARTO node carries `shows` either.
#   - **§3 rows 1 and 2** -- `ldPipe` and `ldSculpt` are genuine complements,
#     and WORLD is the only domain in the whole table where one block's presence
#     is another's absence. That is the one gate, and it is `world/b`'s.
#
# `shows` therefore means: *while this mode is active, the dock renders exactly
# these category headers.* Absent (nine nodes of ten) means no gate at all.
# `world/b`'s `["Terrain"]` is §3 row 2 -- the sculpt block alone -- and
# `world/a` deliberately has none, so the Generation pipeline block keeps all
# nine WORLD categories including `Terrain`'s stage-5 erosion parameters, which
# are pipeline parameters and belong in the pipeline view. `Workspace.apply_mode()`
# is what reads this, and `_select_domain()` is the one place that calls it.
#
# **A gate is only allowed where a route exists.** `world/b` is reachable from
# its rail node, from the dock's own mode switch (`_build_mode_switch()`), and
# from arming Sculpt; `world/a` from all three of the same. Nothing else in the
# three docks is gated, so nothing else can be stranded by one -- which is the
# property `_leftdock12_probe.gd` §2 asserts by name over all thirty-four
# categories rather than by counting them.
#
# The rest of `mode` is a rail-and-dock *selector*, not a gate. Each domain's
# dock is ONE accordion of every category that domain owns, so gating CIVIL by
# mode would make the nine CIVIL categories the prototype has no node for --
# Civilizations, Territories, Economy, Culture, Religion, Politics, Military,
# Relationships, Simulation -- reachable only by a rail trip. That is the
# failure this stage's own rule forbids ("every category reachable before must
# be reachable after"), and §3 does not ask for it. So a node click *opens* its
# category and lights the rail; outside `world/b` it never hides a sibling.
#
# **One node does more than that, and the design is why.** CIVIL ▸ `planner`
# opens the `Travel` category exactly like its siblings *and* arms the Journey
# takeover, because the owner moved that command here from `Data ▸ Journey
# planner… ⇧J` on 2026-09-05 -- it is the one item in the menu bar whose
# destination is the viewport rather than a window or a dock body. The extra
# behaviour is on the node **press** (`_on_rail_node_pressed()`), never on
# `select_domain_mode()`, so nothing that merely wants the Travel accordion
# acquires a viewport swap. It still hides no sibling category.
#
# `category` is the accordion header the node opens -- `Workspace.open_category()`
# matches these strings verbatim against the titles the workspaces pass to
# `DccWidgets.category()`, so a typo here is a silent no-op and is what
# `_railfold_probe.gd` §2 exists to catch.
#
# `owns` is the reverse map: which categories make this node read as the active
# one. Every category of every domain appears in exactly one node's `owns`,
# which is asserted rather than assumed (`_railfold_probe.gd` §3). Where the
# design gives no node for a category, the assignment is this port's judgement
# and is called out in `_MODE_ASSIGNMENT_NOTES` below rather than passed off as
# the design's.
const RAIL_NODES: Array = [
	{"kind": "head", "domain": "world", "label": "WORLD"},
	{"kind": "node", "domain": "world", "mode": "a", "label": "Generation pipeline",
		"category": "Generate",
		"owns": ["Generate", "Geology", "Hydrology", "Climate", "Biomes",
			"Ecology", "Resources", "World data"]},
	{"kind": "node", "domain": "world", "mode": "b", "label": "Sculpt",
		"category": "Terrain", "owns": ["Terrain"], "shows": ["Terrain"]},

	{"kind": "head", "domain": "civilization", "label": "CIVIL"},
	{"kind": "node", "domain": "civilization", "mode": "landmarks", "label": "Landmarks",
		"category": "Landmarks", "owns": ["Landmarks"]},
	{"kind": "node", "domain": "civilization", "mode": "factions",
		"label": "Factions & settlements", "category": "Factions",
		"owns": ["Civilizations", "Factions", "Territories", "Settlements",
			"Economy", "Culture", "Religion", "Politics", "Military",
			"Relationships", "Simulation"]},
	{"kind": "node", "domain": "civilization", "mode": "infra", "label": "Ways & routes",
		"category": "Routes & ways", "owns": ["Routes & ways", "Trade"]},
	{"kind": "node", "domain": "civilization", "mode": "planner", "label": "Journey planner",
		"category": "Travel", "owns": ["Travel"]},

	{"kind": "head", "domain": "cartography", "label": "CARTO"},
	{"kind": "node", "domain": "cartography", "mode": "style", "label": "Layers & style",
		"category": "Layers",
		"owns": ["Layers", "Map style", "Colours", "Roads & routes",
			"Political display", "Visibility / zoom", "Map presets"]},
	{"kind": "node", "domain": "cartography", "mode": "labels", "label": "Labels",
		"category": "Labels", "owns": ["Labels"]},
	{"kind": "node", "domain": "cartography", "mode": "icons", "label": "Icons",
		"category": "Assets & landmarks", "owns": ["Assets & landmarks"]},
	{"kind": "node", "domain": "cartography", "mode": "terrain", "label": "Terrain appearance",
		"category": "Terrain appearance", "owns": ["Terrain appearance"]},
]

# `RAIL_NODES` -- where this port had to decide, because the prototype's ten
# nodes do not cover this shell's thirty-three categories. Written down rather than invented in
# silence, per the house rule; each line is a claim a reader can disagree with.
#
# - **WORLD `b` owns `Terrain` and nothing else.** `Terrain` is where
#   `world_workspace.gd:_build_categories()` parents `_sculpt_body`, so it is
#   the only category that contains the sculpt UI the prototype's `ldSculpt`
#   block draws. The eight remaining WORLD categories are pipeline stages and
#   go to `a`, which is what `ldPipe:s.domain==='WORLD'&&wm==='a'` (`ENV:1945`)
#   means. `Terrain` therefore does NOT appear under `a` even though it carries
#   stage 5's parameters -- a node owns a category exactly once, and the
#   accordion shows all nine regardless, so nothing is lost.
# - **CIVIL `factions` is the catch-all.** The prototype's four CIVIL nodes map
#   cleanly onto four of this shell's fourteen categories. The other ten have no
#   node. Eight of them (Civilizations, Territories, Settlements, Economy,
#   Culture, Politics, Military, Relationships, Simulation) are the roster and
#   its consequences, which is what the `factions` node's own dock block draws
#   (`FACTIONS` list + `civPlaces`, `04-left-dock.md` §4 row 7), so they go
#   there. `Trade` goes to `infra` because `civilization_workspace.gd:238`
#   builds it from `_infra.build_trade_into()` -- INFRA's own subject, and the
#   `infra` node is INFRA's surviving name.
# - **CARTO `style` is the catch-all**, for the same reason: `Layers & style` is
#   the node the prototype gives the layer tree, the ramp editor and
#   `caDomains`/`caLight` (`ENV:496`), and this shell's Map style, Colours,
#   Political display, Visibility / zoom, Map presets and Roads & routes are all
#   layer-and-style subjects with no node of their own.

# -- Region handles -----------------------------------------------------------
#
# Everything a workspace module needs is reachable from here. Workspaces never
# reach past these into the frame's own containers.

var menu_bar_row: HBoxContainer
var tool_options_row: HBoxContainer
var rail_column: VBoxContainer
var left_dock: PanelContainer
var left_dock_title: Label
var left_dock_body: VBoxContainer      ## Workspace panels attach here.
var viewport_area: Control
var viewport_content: Control          ## The map surface; overlays are children.
var right_dock: PanelContainer
var right_dock_title: Label          ## §6: contents follow the selection, not a fixed
	## "Layers" chrome label -- kept live by `right_dock.gd`'s own `set_right_dock_title()` call at the end of every `_rebuild()`.
var right_dock_body: VBoxContainer
var timeline_bar: Control
var timeline_row: HBoxContainer
var status_row: HBoxContainer

var rail_foot: Label
var _domain_buttons: Dictionary = {}   ## id -> Button
var _domain_marks: Dictionary = {}     ## id -> {icon, label}
var _active_domain := "world"

## The per-domain mode, one live selection each, exactly as the prototype keeps
## three independent fields rather than one: `worldMode` (`ENV:1199`, initial
## `'a'`), `civCat` (`cc()`, `ENV:1211`, absent from the initial state and so
## defaulting to `landmarks`) and `cartoCat` (`ct()`, `ENV:1289`, defaulting to
## `style`). Three fields and not one is what lets a user leave CIVIL on
## `planner`, visit CARTO, and come back to `planner` -- the same persistence
## rule `register_workspace()`'s comment states for L2 accordion state.
##
## Seeded from `RAIL_NODES` rather than written out, so the defaults cannot
## drift from the tree: the first node of each domain is that domain's default,
## which matches all three of the prototype's own defaults (`a`, `landmarks`,
## `style` are each their domain's first node).
var _domain_mode: Dictionary = _default_modes()

static func _default_modes() -> Dictionary:
	var out := {}
	for n in RAIL_NODES:
		if String(n.get("kind", "")) == "node" and not out.has(String(n["domain"])):
			out[String(n["domain"])] = String(n["mode"])
	return out
var _left_collapsed := false
var _right_collapsed := false
var _left_width := float(DccTheme.W_LEFT_DOCK)
var _right_width := float(DccTheme.W_RIGHT_DOCK)
var _status_labels: Dictionary = {}    ## slot -> Label
## §2's three trailing squares (`03-menu-bar.md` §2, children 4/5/8). Held
## because both undo and redo carry live engine state -- enabled-ness and the
## operation name in the tooltip -- that does not exist yet when
## `_build_menu_bar()` runs. `_wire_menu_squares()` is what keeps them true.
var _menu_undo_btn: Button
var _menu_redo_btn: Button
var _collapse_buttons: Dictionary = {} ## "left"/"right" -> Button, so the chevron can flip
var _dock_readouts: Dictionary = {}    ## "left"/"right" -> the collapsed-state Label
var _workspace_panels: Dictionary = {} ## domain id -> Control
var _touch := false

# -- Domain rail ---------------------------------------------------------------
## What `Window ▸ Domain rail` hides. Not the same node in both compositions,
## which is the whole reason it is a named field rather than a walk up from
## `rail_column`: on desktop it is the rail panel itself (exactly what
## `rail_column.get_parent().get_parent()` used to reach), but on the phone the
## domains are three cells of the L1 bottom bar, and hiding the bar would take
## the MENU cell with it -- the only route back to the row that un-hides it.
## See `_build_phone_menu_bar()`.
##
## **Desktop: this is now the rail *pair*, not the 40 px strip.** `ENV:282`
## wraps both the strip and the expansion column in the one
## `<sc-if value="{{ showRail }}">`, so hiding the rail hides the expansion with
## it -- which is also the only correct behaviour, since the expansion has no
## affordance of its own to reopen from once its chevron is gone.
var _rail_region: Control

# -- The expansion column (`railExp`, `ENV:293`-`303`) -------------------------
## Collapsed at rest, matching `railExp:false` (`ENV:1199`). The prototype makes
## it a genuine `<sc-if>` -- the column is absent from the DOM, not a width
## transition -- so this is `visible`, not an animated width.
var _rail_expanded := false
var _rail_exp_column: Control          ## The 200/264 px column itself.
var _rail_chevron: Control             ## The one `▸` that rotates 0°/180°.
var _rail_node_rows: Dictionary = {}   ## "domain/mode" -> the row's Label.

# -- WI-04 dock width dragging --------------------------------------------------
var _dragging_dock := ""  ## "", "left" or "right" -- which handle (if any) owns the current drag.

# -- Phone layout (§13) --------------------------------------------------------
#
# `_touch` alone can't tell a tablet from a phone -- both are touch devices.
# The discriminator is the screen's own aspect: `min(w,h)/max(w,h)` is
# order-independent, so it survives rotation without flip-flopping between the
# two compositions (a phone rotated to landscape is still ~0.46, a tablet
# rotated to portrait is still ~0.625). `_phone` is decided once, at boot,
# because a device's form factor never changes at runtime; `_landscape` is
# re-decided on every resize, because rotation genuinely does, and §13 asks
# for a distinct landscape treatment.
const _PHONE_ASPECT_MAX := 0.6  ## Midpoint-ish between 19.5:9 (~0.46) phones
	## and 16:10 (~0.625) tablets -- every common handset aspect sits under it.

## **Aspect alone gets a 16:9 tablet wrong, and that is not hypothetical.**
##
## 1920 x 1080 is 0.5625, under `_PHONE_ASPECT_MAX`, so every 16:9 Android
## tablet was classified as a PHONE and given the phone composition. Found on
## 2026-08-30 by `_tabletparity_probe.gd`, whose "desktop" leg boots 1920x1080
## under `--force-touch` and came up running `phone_project_picker.gd` --
## visible in the run as a `_set_transient_exclusive_child` warning from
## `phone_present()` in a leg that had no business being a phone at all.
##
## It also runs straight against the owner's standing directive to "keep the
## tablet version as close as possible to the windows gui": a 16:9 tablet was
## getting the opposite.
##
## The fix is a SIZE test, which is the thing aspect was standing in for.
## dp is `px / (dpi / 160)`.
##
## **The threshold is 900 dp, and it is deliberately NOT Android's own 600.**
## Owner ruling, 2026-08-31, with the arithmetic that settles it: the
## desktop-parity shell has a hard chrome floor of 48 dp rail + 400 dp dock =
## **448 dp before any map at all**. At 800 dp that leaves a 352 dp map --
## narrower than the dock beside it, which is not the Windows GUI in any useful
## sense. At 900 dp it leaves 452 dp and the map is the larger pane again.
##
## So the line is where the map stops being the smaller half. `sw600dp` is the
## right breakpoint for a phone/tablet LAYOUT question in general; it is the
## wrong one for this shell, whose chrome is unusually wide.
##
## Worked through, on the devices this port is measured against:
##
##   OnePlus 6T   1080 short / (402/160) = 430 dp  -> phone
##   OnePlus 12   1440 short / (525/160) = 439 dp  -> phone
##   TABLET 800    800 dp                          -> phone   (deliberate; the
##                                                   design frames it as one)
##   16:9 tablet  1080 short / (200/160) = 864 dp  -> phone   (under 900)
##   2560x1600    1600 short / (288/160) = 889 dp  -> phone   (!! see below)
##   TABLET PORT. 1600 dp shortest width           -> tablet
##
## **The 2560x1600 case is worth stating plainly**: at 288 dpi it measures
## 889 dp, four short of the line, so a physically large tablet with a very
## dense panel lands on the phone side. That is what the ruling says and the
## arithmetic is the ruling's own -- the shell needs 448 dp of chrome and 889
## does not comfortably carry it. `_tabletparity_probe` forces the tablet
## composition directly and so is unaffected.
##
## **Applied only on a real mobile device.** `screen_get_dpi()` reports the
## desktop monitor under `--force-touch`, where the viewport size is synthetic
## and the two have nothing to do with each other -- a probe forcing 1080x2340
## on a 96-dpi monitor computes 1800 dp and would classify the phone leg as a
## tablet. So the dp test runs when `OS.has_feature("mobile")` is genuinely
## true, and `--force-touch` runs keep pure aspect, which is what every existing
## phone probe was written against.
const _TABLET_MIN_DP := 900.0
var _phone := false
var _landscape := false
## The on-screen keyboard's height in physical px, 0 when it is down. Written
## only by `_process()`, read by every bottom inset -- see that function for why
## this is polled rather than signalled.
var _phone_kb_height := 0
var _phone_scale := 1.0  ## Maps `DccTheme.PHONE_REF_SHORT` phone-px onto the
	## real device's short side. Clamped to >= 1.0: the mockup's own numbers
	## already clear the 44 px floor at scale 1, so this only ever scales up,
	## never shrinks a target below spec.

# Phone region handles, built only when `_phone` is true.
var _phone_root: Control
var _phone_top_safe: Control
var _phone_side_safe: Control
var _phone_chrome_margin: MarginContainer  ## Shifts right in landscape so the
	## rail and app bar clear `_phone_side_safe` -- "the domain rail shifts
	## inward" (Phone inset rules, LANDSCAPE).
var _phone_content_gap: Control            ## Hosts the floating rail; its own
	## rect is the visible gap between the app bar and the tool sheet.
var _phone_tool_sheet: PanelContainer
var _phone_app_bar: PanelContainer  ## Held so a probe can measure it and so
	## `phone_content_insets()` reads its real height rather than recomputing it.
var _phone_gesture_inset: Control

# -- Phone bottom-sheet detents ------------------------------------------------
#
# `docs/ANDROID_UI_SPEC.md`, Locked decisions: *"Sheets: peek → half → full
# detents, drag handle; tab tap opens half"* and *"bar stays visible at full
# sheet"*.
#
# The three heights are the interactive prototype's own arithmetic, not derived
# ones -- `design/android-2026-08-30/Cartalith Android.dc.html`, `_detH()`:
#
#     fh   = frameH - 84            (portrait; the whole frame in landscape)
#     peek = 66                     a constant, not a fraction of anything
#     half = round(fh * 0.46)
#     full = fh - 96                so 96 dp of map is never covered
#
# That `84` is the prototype's `navH`, which is exactly this shell's bottom bar
# (`H_PHONE_BOTTOM_NAV` 64) plus its gesture inset (`H_PHONE_GESTURE` 20) -- the
# same figure reached from two constants instead of one literal.
# `_phone_nav_reserve()` reads it live rather than hard-coding 84, because this
# shell parks the timeline between the sheet and the bar and the prototype has
# no equivalent of that row.
#
# **"Bar stays visible at full sheet" is structural here, not arithmetic.** The
# sheet is a *sibling above* the bottom bar in the chrome column, so no detent
# height can put it over the bar; the prototype has to position the sheet at
# `bottom:${navH}px` to get the same result out of absolute positioning.
const PHONE_DETENT_PEEK := 66.0        ## `_detH`: `det==='peek'?66`.
const PHONE_DETENT_HALF_FRAC := 0.46   ## `Math.round(fh*0.46)`.
const PHONE_DETENT_FULL_GAP := 96.0    ## `full=fh-96`.
const PHONE_DETENT_ANIM := 0.28        ## `transition:height .28s cubic-bezier(.3,.9,.3,1)`.
	## Godot has no cubic-bezier easing; `TRANS_CUBIC`/`EASE_OUT` is the curve
	## that control point set approximates -- named as an approximation rather
	## than passed off as the mockup value.
const PHONE_DETENT_MIN_DRAG := 40.0    ## `_sm`: `Math.max(40, ...)`.
const PHONE_DETENT_DISMISS := 44.0     ## `_su`: `if(h<44)` closes the sheet.
## The prototype boots at `detent:'half'` because ITS sheet carries the tab's
## whole panel. This one carries the tool options row and nothing else (§13:
## "tool options become a bottom sheet"), so half opens a sheet that is mostly
## empty -- measured on the handset: one row of chips above roughly 700 px of
## nothing. Boot at `peek`, which is that content's own height, and leave half
## and full reachable by dragging the handle.
##
## This is a deliberate divergence from the prototype and it is the CONTENT
## that differs, not the geometry: the detent sizes below are the prototype's
## own numbers, unchanged.
var _phone_detent := "peek"
var _phone_sheet_grab: Control         ## The drag handle's own hit area.
var _phone_sheet_drag := {}            ## `{"y0","h0","h"}` while a drag is live; empty otherwise.
var _phone_sheet_tween: Tween          ## Held so a new snap kills the running one.

# -- Phone landscape: left rail + right-docked sheet ---------------------------
#
# `docs/ANDROID_UI_SPEC.md`: *"Landscape: nav becomes left rail, sheet docks
# right, map stays wide."* Both are the *same nodes* rotated into new positions
# rather than a second set built alongside -- `_apply_phone_orientation()`
# reparents them between the chrome column and `_phone_root`, and flips the
# bar's own box containers with `BoxContainer.vertical`.
var _phone_chrome_col: VBoxContainer   ## The portrait stack the two nodes above return to.
var _phone_bar_row: BoxContainer       ## The bottom bar's four nested boxes, held
var _phone_bar_domains: BoxContainer   ## only so `vertical` can be flipped on them.
var _phone_bar_cells: BoxContainer
var _phone_bar_dests: BoxContainer     ## The three destination tabs, MORE excluded
	## -- `_rail_region` on phone. See `_build_phone_menu_bar()`'s header.
const W_PHONE_LAND_RAIL := 72.0   ## Prototype, landscape branch: `width:72px;
	## background:var(--pan2);border-right:1px solid var(--hair2)`, and the map
	## host moves to `left:72px`.
const W_PHONE_LAND_SHEET_MAX := 440.0  ## `Math.min(440,Math.round(fw*0.46))`.
const PHONE_LAND_SHEET_FRAC := 0.46

var _phone_clock_label: Label
var _phone_battery_label: Label
var _phone_side_clock_label: Label     ## Landscape's rotated-pocket twins of
var _phone_side_battery_label: Label   ## the two above -- see `_build_phone_side_safe()`.
var _phone_panel_picker: Control
var _phone_menu_bar: Control    ## L1 of the phone disclosure tree -- the bottom
	## bar. Named handle because `_phone_bottom_reserve()` has to measure it.
var _phone_menu: PhoneMenu      ## L2-L5. Replaces the old `_phone_overflow`
	## sheet (`GUI_GAP_REGISTER.md` §15).
var _left_sheet_open := false
var _right_sheet_open := false
## The two dock `ScrollContainer`s, held only so `_set_sheet_open()` can zero
## their scroll on open -- neither dock exposes its `_scroll()` return value
## anywhere else, and a sheet's body (`left_dock_body`/`right_dock_body`) is
## never torn down between opens, so whatever scroll position was left from
## the previous open is still sitting on the node when it reopens.
var _left_dock_scroll: ScrollContainer
var _right_dock_scroll: ScrollContainer

# -- Phone: app-bar search, floating undo chip, coach marks --------------------
#
# The three items the Android phone-chrome spec still owed
# (`design/Cartalith Android Phone.dc.html`'s own TARGETS/chip vocabulary):
# "Search: app bar, pans map to place", "Undo: floating ↶ chip (tap undo, hold
# history), map edits only", "Coach marks: two subtle toasts, persisted".
#
# All three read live engine state (`EngineBridge.can_undo()`,
# `ViewportHost.move_view_to()`, `PlaceSearch`), which `bridge`/`viewport` are
# -- but those are `DccApp` fields (`app.gd`), not this base class's. This
# file's own header says it "calls no engine method", and every existing
# reach past that line already does it the same way: a typed child lookup
# guarded to return null on a bare `DccShell` (`_find_engine_bridge()`,
# `_find_viewport_host()` below), the same shape `has_method("open_journey_
# planner")` at `_pick_phone_tab()` already uses for the one subclass METHOD
# this file calls. A probe that instantiates `DccShell` alone degrades to "the
# chip never shows, the search button never draws, the toasts never fire" --
# never a crash.
var _phone_search_overlay: Control
var _phone_search_field: LineEdit
var _phone_search_results: VBoxContainer
var _place_search_index  ## `PlaceSearch` -- untyped, see `_has_place_search()`.

var _phone_undo_chip: Button
## `06-phone.md` §6.2's edit-history popover and sim strip, and the app bar's
## `⋮` overflow. All three float in the phone composition and are hidden until
## something opens them.
var _phone_undo_pop: Control
var _phone_sim_strip: Control
var _phone_sim_year: Label
var _phone_sim_slider: HSlider
var _phone_sim_play: Button
var _phone_sim_speeds: Dictionary = {}   ## multiplier -> Button
var _phone_sim_transport: Array[Control] = []  ## Everything `tl_available()` gates.
var _phone_overflow_pop: Control
var _phone_overflow_saved: Label
var _phone_overflow_theme: Label
## `savedAt` (§4.3's own state key). Filled from `EngineBridge.project_saved`,
## which is the signal `app.gd`'s own save bookkeeping already fires -- so this
## is the same event, not a second notion of "saved".
var _phone_saved_at := ""
var _undo_chip_down := false
var _undo_chip_hold_fired := false
const PHONE_UNDO_HOLD_SEC := 0.45  ## Standard mobile long-press threshold
	## (Android's own `ViewConfiguration.getLongPressTimeout()` default).

## Desktop's `Edit ▸ Find on map…` presentation -- built lazily, once, on
## first `open_find_on_map()` call. See that function for why this is a plain
## `AcceptDialog` rather than the phone's hand-built overlay.
var _desktop_search_dialog: AcceptDialog
var _desktop_search_field: LineEdit
var _desktop_search_results: VBoxContainer

# -- Build identity ------------------------------------------------------------

## One line in the boot log saying **which build this is**, and it exists
## because a device pass could not answer that question and drew the wrong
## conclusion from not being able to (`GUI_GAP_REGISTER.md` §56).
##
## The shell-side twin of `EngineBridge._has()`, which the 2026-08-24 pass added
## after a `.so` ran 21 commits behind its own shell in silence
## (`ANDROID_BUILD_SCOPE.md`). That guard speaks for the *native* half of the
## pair; nothing spoke for the GDScript half, so two APKs built forty minutes
## apart across a UI migration were indistinguishable on the handset except by
## looking at them -- and §54 recorded that difference as "the shell's chrome is
## not stable across boots", a startup race that does not exist.
##
## Hashes every file the project ships under `res://shell/`, name and content,
## in sorted order. Deliberately the whole directory rather than this one file:
## the two APKs §54 compared differed in `map_overlay.gd` alone in one pair and
## in `dcc_shell.gd` in the other, and a digest that only covers its own source
## would have missed the first. `map_overlay.gd` itself sits at the project root
## and is folded in by name for the same reason.
##
## Costs a few hundred KB of MD5 at boot, once, and cannot rot: there is no
## version constant for anyone to forget to bump. It changes when the shipped
## scripts change, which is exactly the question "is this the same build?"
##
## The digest is **not** comparable between an editor run and an export -- an
## export ships `.gdc` + `.gd.remap` where the editor has `.gd` + `.uid`. That is
## a feature: those genuinely are different builds of the same tree, and the two
## must never be mistaken for each other in a measurement log.
##
## And in an export it is a *behaviour* fingerprint rather than a source one,
## because what it hashes there is the compiled token stream. Measured, not
## assumed: appending one comment line to `shell/right_dock.gd` and re-exporting
## leaves `assets/shell/right_dock.gdc` **byte-identical** (`a8ee3535...`), while
## a one-line code change moves it (`ed7b699f...`). So two APKs differing only in
## comments carry the same id -- which is the right granularity for "is this the
## same build?" asked of a measurement, and the wrong one for "is this the same
## commit?". For the latter, hash the APK.
##
## It does not, and cannot, prove the *installed APK* is the one just built --
## only `sha256` against `adb shell pm path` does that. See §56's harness note.
static func build_id() -> String:
	var files: Array[String] = ["res://map_overlay.gd", "res://map_overlay.gdc"]
	var stack: Array[String] = ["res://shell"]
	while not stack.is_empty():
		var dir: String = stack.pop_back()
		for d in DirAccess.get_directories_at(dir):
			stack.append(dir.path_join(d))
		for f in DirAccess.get_files_at(dir):
			files.append(dir.path_join(f))
	files.sort()
	var ctx := HashingContext.new()
	ctx.start(HashingContext.HASH_MD5)
	for f in files:
		## `get_md5()` returns "" for a path that is not there, which is the
		## normal case for two of the three root entries above -- folding the
		## empty string in under its own name keeps the digest defined either way.
		ctx.update((f + ":" + FileAccess.get_md5(f)).to_utf8_buffer())
	return ctx.finish().hex_encode().substr(0, 12)

# -- Build --------------------------------------------------------------------

func _ready() -> void:
	## Before anything else, so it is the first Cartalith line in `logcat` and
	## survives a boot that fails after it. See `build_id()` above for why a
	## device pass needs it.
	print("Cartalith shell build ", build_id())
	## `--force-touch`: a testing-only override, same pattern as `_shot.gd`'s
	## own `--generate` flag. Real touch hardware is never present in this
	## dev/CI environment, so without it the phone/tablet composition below is
	## simply unreachable from `--resolution WxH` alone -- there is no device-
	## preview loop, and `--resolution` is otherwise the only lever the
	## verification harness has to exercise §13 at all.
	_touch = (DisplayServer.is_touchscreen_available() and OS.has_feature("mobile")) \
		or "--force-touch" in OS.get_cmdline_user_args()
	## Published for the static widget factories -- `DccWidgets.style_popup()`
	## sizes a menu off it and has no node to reach this one through.
	DccTheme.set_touch(_touch)
	set_anchors_preset(Control.PRESET_FULL_RECT)
	var ground := ColorRect.new()
	ground.color = DccTheme.c("bg")
	ground.set_anchors_preset(Control.PRESET_FULL_RECT)
	ground.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(ground)

	## Phone-vs-tablet is decided once, here, off the boot window size -- a
	## device's own form factor is not something that changes at runtime.
	## Orientation (`_landscape`) *is* re-decided on every resize below, which
	## is the half of §13 that genuinely needs to react live.
	_compute_layout_mode()
	## The narrower half of the same publication -- the 412 canvas asks for
	## things a tablet must not get, and a static factory has no way to tell the
	## two apart from `is_touch()` alone. See `DccTheme.is_phone()`.
	DccTheme.set_phone(_phone)
	DccTheme.set_phone_scale(_phone_scale)
	_style_window_chrome()

	## Hand the Android back gesture to `_notification()` below instead of
	## letting the SceneTree quit on it -- there are levels to pop first
	## (`design/Cartalith Android Phone.dc.html`, PHONE RULES / BACK), and past
	## the last of them an unsaved world to protect.
	##
	## Set for EVERY layout mode, not only the phone. It was phone-only when
	## first written, which left every Android device the aspect test classifies
	## as a *tablet* taking the SceneTree default -- back quits, at once, with
	## no prompt. `quit_on_go_back` is inert on desktop, where no windowing
	## system ever sends the request, so there is nothing to guard it with.
	get_tree().quit_on_go_back = false
	## The desktop half of the same guard: the title bar's ×, Alt+F4 and the
	## taskbar's Close all arrive as `NOTIFICATION_WM_CLOSE_REQUEST`, and with
	## `auto_accept_quit` at its default the SceneTree ends the process on them
	## before any of this shell's code runs -- an unsaved world destroyed by one
	## click, the exact fault the back gesture had on Android.
	##
	## Turning it off means NOTHING quits the app unless our code asks, so
	## `_close_requested()` below carries the obligation to always resolve. See
	## `DccApp._close_requested()` for the proof that it cannot trap the user.
	get_tree().auto_accept_quit = false
	## The six timeline layer toggles, before either composition builds a view
	## of them -- see the §10a block.
	_tl_load_layers()
	if _phone:
		_build_phone_shell()
	else:
		_build_desktop_shell()

	get_tree().root.size_changed.connect(_on_window_resized)
	_select_domain(_active_domain)
	## Deferred, and it has to be: this function runs as `super._ready()` from
	## `DccApp._ready()`, which only creates `EngineBridge` and `ViewportHost` on
	## the lines *after* it returns. A deferred call lands after every `_ready()`
	## in the tree, so both exist by then -- and `_refresh_viewport_context()`
	## still no-ops rather than erroring on a bare `DccShell` (every phone-chrome
	## probe in this project), which has neither.
	_refresh_viewport_context.call_deferred()

## The pointer-first / tablet composition: one continuous vertical stack of
## fixed-height bars around a horizontal row of rail/docks/viewport. This is
## the shell as it existed before phone support -- `_scaled()` already makes
## it tablet-safe, so nothing here is phone-aware.
func _build_desktop_shell() -> void:
	var shell := VBoxContainer.new()
	shell.set_anchors_preset(Control.PRESET_FULL_RECT)
	shell.add_theme_constant_override("separation", 0)
	add_child(shell)

	shell.add_child(_build_menu_bar())
	shell.add_child(_build_tool_options_bar())

	var main_row := HBoxContainer.new()
	main_row.size_flags_vertical = Control.SIZE_EXPAND_FILL
	main_row.add_theme_constant_override("separation", 0)
	shell.add_child(main_row)

	main_row.add_child(_build_rail())
	main_row.add_child(_build_left_dock())
	main_row.add_child(_build_viewport())
	main_row.add_child(_build_right_dock())

	timeline_bar = _build_timeline()
	shell.add_child(timeline_bar)
	shell.add_child(_build_status_bar())

func _scaled(px: int) -> int:
	## §13: tablet scales every fixed height, with a 44 px floor on anything
	## tappable. Windows is pointer-first and takes the raw value. Phone does
	## not use this at all for its own chrome -- see `_pscale()`/`_ptap()` --
	## but still calls it for the desktop-shaped bars (menu bar, status bar)
	## that phone relocates into the ⋯ overflow sheet unmodified.
	if not _touch:
		return px
	## `DccTheme.TABLET` first: §1's tablet column is a table of exact figures,
	## not a multiplier, and the multiplier-plus-floor this used to be got two
	## of the five wrong in opposite directions. The 44 px floor still applies
	## to anything the table does not name, because an unnamed figure is a
	## control rather than a region.
	if DccTheme.TABLET.has(px):
		return int(DccTheme.TABLET[px])
	return maxi(44, int(round(px * DccTheme.TOUCH_SCALE)))

# -- §13 Phone layout mode ------------------------------------------------

## Order-independent aspect: see the field comment on `_phone` above for why.
## Android's `sw600dp`: is the short side at least 600 density-independent
## pixels? See `_TABLET_MIN_DP`'s own comment for why this exists and why it is
## gated on a real device.
##
## Returns `false` -- "not tablet-sized, judge by aspect alone" -- whenever the
## question cannot be answered honestly: off a mobile device, or when the
## platform reports a DPI of zero or less, which some Android builds do for a
## secondary display. A wrong `true` would hand a phone the desktop
## composition, which is far worse than the aspect rule this falls back to.
func _is_tablet_sized(short_side_px: float) -> bool:
	if not OS.has_feature("mobile"):
		return false
	var dpi := float(DisplayServer.screen_get_dpi(DisplayServer.window_get_current_screen()))
	if dpi <= 0.0:
		return false
	return short_side_px / (dpi / 160.0) >= _TABLET_MIN_DP

func _compute_layout_mode() -> void:
	var size: Vector2 = get_viewport_rect().size
	if size.x <= 0.0 or size.y <= 0.0:
		return
	var short_side: float = minf(size.x, size.y)
	var long_side: float = maxf(size.x, size.y)
	_phone = _touch and (short_side / long_side) < _PHONE_ASPECT_MAX \
		and not _is_tablet_sized(short_side)
	_landscape = size.x > size.y
	_phone_scale = maxf(1.0, short_side / DccTheme.PHONE_REF_SHORT)
	## The **fourth density set**, new with the 2026-08-31 token re-base.
	## `DccTheme.LAPTOP`'s header carries the whole argument; what belongs here
	## is why the test is on `size.x` and not on `short_side`. The prototype's
	## own gate is `frame === 'w1366'` against a 1366 x 768 artboard (`ENV:1675`)
	## and what the narrow set gives back is horizontal: two dock widths and a
	## menu popup. A tall pointer window -- a 1200 x 1600 portrait monitor -- has
	## a short side of 1200 and a width of 1200, so both readings agree there;
	## they part on a 2560 x 1080 ultrawide, where `short_side` would call it
	## narrow and the width correctly does not. Width is the question the
	## override answers.
	DccTheme.set_narrow(size.x < float(DccTheme.W_LAPTOP_MAX))
	## §1's tablet column widens BOTH docks to 400 px, "so two-column readouts
	## survive the larger type" (`UI_SHELL_DESIGN.md`). Neither dock had ever
	## been told that: tablet ran the desktop pair, which was 372/300 when this
	## was written and is 372/304 after the 2026-08-31 token re-base -- the
	## point is unaffected, since what tablet needed was 400/400 either way.
	if _touch and not _phone:
		_left_width = float(DccTheme.W_DOCK_TABLET)
		_right_width = float(DccTheme.W_DOCK_TABLET)
	## The LAPTOP band's half of the same assignment, and the reason it is an
	## `elif` in spirit even though `is_laptop()` already excludes touch: both
	## branches write the same two fields, and reading them as one either/or is
	## how the next person will expect it. `role_px()` resolves the override, so
	## the widths are stated once, in `DccTheme.LAPTOP`, and not duplicated here.
	##
	## Runs once, from `_ready()`, exactly like the tablet branch above and for
	## the same recorded reason: `_on_window_resized()` early-returns for
	## anything that is not the phone, so that a tablet user's dragged dock
	## widths (WI-04) are not reset on rotation. The cost is that dragging a
	## desktop window across 1920 px does not re-band it until the next launch.
	## That is a real limit and it is the existing one, not a new one -- making
	## the band live would first require the drag-width preservation this early
	## return protects.
	elif DccTheme.is_laptop():
		_left_width = float(DccTheme.role_px("w_left_dock"))
		_right_width = float(DccTheme.role_px("w_right_dock"))

## **The early return is deliberate, and it was audited rather than assumed**
## (`GUI_GAP_REGISTER.md` §56). It reads as an asymmetry -- a shell that latched
## tablet by mistake can never correct itself -- and the obvious fix, hoisting
## `_compute_layout_mode()` above the guard, is the wrong one twice over:
##
## - There is nothing to correct. `_phone` needs `_touch`, which is fixed for
##   the life of the process, and the aspect it tests is `min/max`, which a
##   rotation cannot change. Measured on the OnePlus 6T over 52 cold starts:
##   `get_viewport_rect()` reports the real 1080 x 2340 at the first sample
##   inside `_ready()` and `root.size_changed` never fires at all. There is no
##   provisional size for the decision to race against on this handset.
## - Recomputing for tablets would *break* something real. The tablet branch of
##   `_compute_layout_mode()` assigns `_left_width`/`_right_width`, so running
##   it on every resize would reset a tablet user's dragged dock widths (WI-04)
##   to `W_DOCK_TABLET` on every rotation.
func _on_window_resized() -> void:
	if not _phone:
		return  ## Tablet/desktop windows resizing is not this shell's concern.
	var was_landscape := _landscape
	_compute_layout_mode()
	if _landscape != was_landscape:
		_apply_phone_orientation()

## Phone-only geometry: scaled off the real device's short side, no 44 px
## floor -- for chrome that is deliberately *not* tappable (the top safe area,
## the gesture inset) a floor would be wrong.
func _pscale(px: float) -> int:
	return maxi(1, int(round(px * _phone_scale)))

## Phone-only geometry for anything tappable: §13's floor, no exceptions.
##
## **Was `maxi(DccTheme.PHONE_TAP_MIN, _pscale(px))` and the floor never fired
## on a real handset** -- caught by a coordinator review of this exact file,
## 2026-08-30, not by any probe. `PHONE_TAP_MIN` (44) is a REFERENCE-px figure,
## the same unit every other constant `_pscale()` takes is authored in, but the
## old body compared it directly against `_pscale(px)`, which is already
## PHYSICAL px. On the OnePlus 6T this build is tested on (`_phone_scale` =
## 1080/412 = 2.621), `_ptap(40)` -- the app bar's own icon cell,
## `PHONE_ICON_BOX` -- computed `maxi(44, round(40*2.621))` = `maxi(44, 105)` =
## 105 physical px, which is **40 dp**, not 44: the "floor" of 44 was being
## measured in the wrong unit and so sat *below* every scaled value it was
## meant to raise. The bug was invisible near `_phone_scale` 1.0 (where 44 and
## `_pscale(44)` coincide) and silent everywhere else, because a control that
## is merely a bit small draws exactly like one that is correctly sized.
##
## `phone_fit()` above (`tap := maxf(1.0, round(DccTheme.PHONE_TAP_MIN * unit))`,
## this file's OTHER 44 dp floor) already did this the right way round --
## floor in reference units, multiply once -- which is what confirms the fix
## rather than just asserting it: `_pscale(maxf(PHONE_TAP_MIN, px))` is
## `phone_fit()`'s own `tap` expression with `unit` renamed to `_phone_scale`.
##
## The one call site this moves is `_phone_bar_button()`'s `PHONE_ICON_BOX`
## (40 < 44); every other `_ptap()` call in this file already passes >= 44 and
## is bit-for-bit unchanged (verified: for px >= 44 and scale >= 1,
## `maxi(44, pscale(px))` and `pscale(maxf(44,px))` are the same value, since
## `pscale(px) >= px >= 44` already). At scale 2.621 the app bar's ☰/▤/⌕ cells
## grow from 105 to `_pscale(44)` = 115 physical px (+10, ~9%) -- still well
## inside the app bar's own `_ptap(H_PHONE_APP_BAR)` = 147 px height, and the
## three-cell-plus-wordmark row still fits its widest measured target (720 px
## short side) with over 200 px to spare for "CARTALITH" + the seed subtitle.
## No collision found; see `_phonechrome_probe.gd`'s tap-floor walk for the
## general assertion this fix needed and the old code would have failed.
func _ptap(px: float) -> int:
	return _pscale(maxf(DccTheme.PHONE_TAP_MIN, px))

## -- Dynamic type -------------------------------------------------------------
##
## `UNWIRED_FUNCTIONS.md` "No content descriptions, no dynamic type": every
## phone type size in this file was `_pscale(px)`, which is the *screen's* short
## side over 412 and says nothing about how large the person using the phone has
## asked text to be. A user who has turned Android's font size up to its largest
## step got exactly the same 10 px readouts as one who has never opened that
## setting.
##
## **What this build actually exposes, checked rather than assumed.** A full
## `ClassDB` sweep of every class in this Godot 4.7.1 build for a method whose
## name contains `font_scale`, `text_scale` or `oversampl` returns only
## `Viewport`/`Window` oversampling and `TextServerExtension`'s virtuals --
## there is **no** font-scale accessor anywhere, on `OS`, on `DisplayServer` or
## on the `AccessibilityServer` singleton (whose whole method list is
## `update_set_*` node properties). So the OS setting has to be *derived*.
##
## The derivation uses the two `DisplayServer` screen metrics this file already
## trusts: `_is_tablet_sized()` above reads `screen_get_dpi() / 160.0` as the
## platform's display **density**, which is Android's own definition. Android's
## other metric, `DisplayMetrics.scaledDensity`, is density multiplied by the
## user's font scale, and is what `screen_get_scale()` reports there -- so their
## ratio is the font scale on its own.
##
## **Android only, and disclosed as such.** On a pointer build the two numbers
## are unrelated (`screen_get_scale()` is the window-manager UI scale and
## `screen_get_dpi()` is 96 on the desktop this was measured on, giving a
## meaningless 1.67), and desktop has `DccTheme`'s own density sets for the same
## job. Everything that is not Android reads 1.0 and this function changes
## nothing. Clamped to 0.85..1.6 because the shell's phone chrome is a fixed
## 64/56/28 dp column stack: a 2.0 the platform is entitled to return would
## overflow rows that have nowhere to grow, and half-honouring a setting beats
## breaking the layout that carries it.
##
## Cached: the value is a system setting, not a per-frame quantity, and this is
## called from every label built on the phone.
var _os_text_scale_cache := -1.0

func _os_text_scale() -> float:
	if _os_text_scale_cache >= 0.0:
		return _os_text_scale_cache
	_os_text_scale_cache = 1.0
	if OS.has_feature("android"):
		var screen := DisplayServer.window_get_current_screen()
		var density := float(DisplayServer.screen_get_dpi(screen)) / 160.0
		var scaled := float(DisplayServer.screen_get_scale(screen))
		if density > 0.0 and scaled > 0.0:
			_os_text_scale_cache = clampf(scaled / density, 0.85, 1.6)
	return _os_text_scale_cache

## Phone-only **type** size: `_pscale()` for the device, times the OS font
## scale for the person. Every phone font size in this file goes through this
## rather than `_pscale()`; every phone *box* still goes through `_pscale()` /
## `_ptap()`, because a tap target is a finger measurement and does not grow
## when type does.
func _pfont(px: float) -> int:
	return maxi(1, int(round(px * _phone_scale * _os_text_scale())))

## -- Real device safe areas ---------------------------------------------------
##
## `DccTheme.H_PHONE_TOP_SAFE` (28) and `H_PHONE_GESTURE` (20) are the 412
## canvas's own figures and nothing in `shell/` ever asked the DEVICE what its
## insets are, so on a handset whose status cutout is deeper than 28 dp the
## clock row -- and the app bar under it -- drew beneath the cutout.
## `BUILD_ANSWERS.md` §4 rules it: *"the mock value is the floor, the real inset
## wins when it is larger"*, which is the prototype's own
## `max(env(safe-area-inset-top), 30px)`.
##
## `get_display_safe_area()` reports PHYSICAL px in screen coordinates, and
## `_pscale()` produces physical px too, so the two are directly comparable: the
## mock is scaled into device px first and the real inset is not scaled at all.
## On every platform without cutouts the safe area IS the screen, so all four
## insets come out 0 and only the mock survives -- desktop is untouched.
func _display_safe_inset(edge: String) -> int:
	var screen := DisplayServer.screen_get_size()
	var safe := DisplayServer.get_display_safe_area()
	## A platform with no notion of a safe area reports an empty rect rather
	## than the whole screen; either way there is nothing to add to the mock.
	if screen.x <= 0 or screen.y <= 0 or safe.size.x <= 0 or safe.size.y <= 0:
		return 0
	match edge:
		"top":
			return maxi(0, safe.position.y)
		"bottom":
			return maxi(0, screen.y - safe.end.y)
		"left":
			return maxi(0, safe.position.x)
		_:
			return maxi(0, screen.x - safe.end.x)

## The portrait status band, and the top inset every phone rect measures from.
func _safe_top() -> int:
	return maxi(_pscale(DccTheme.H_PHONE_TOP_SAFE), _display_safe_inset("top"))

## The gesture inset, and with it every "stop above the bottom edge" offset.
func _safe_bottom() -> int:
	return maxi(_pscale(DccTheme.H_PHONE_GESTURE), _display_safe_inset("bottom"))

## Landscape's side band. It is drawn from `H_PHONE_TOP_SAFE` because it is the
## portrait status row rotated (see `_build_phone_side_safe()`), and the device
## agrees -- the cutout that was at the top is now at one side. WHICH side
## depends on which way the handset was turned and this band is only ever drawn
## on the left, so the wider of the two real insets is the honest reserve.
func _safe_side() -> int:
	return maxi(_pscale(DccTheme.H_PHONE_TOP_SAFE),
		maxi(_display_safe_inset("left"), _display_safe_inset("right")))

## -- Haptics ------------------------------------------------------------------
##
## `BUILD_ANSWERS.md` §4's one table, transcribed whole: sample 12 ms · detent
## 8 ms · tool arm 10 ms · verdict `[14, 40, 14]` · back 6 ms · blocked
## `[20, 60, 20]`. All six are defined even though only three have call sites in
## this shell today, because the table is the specification and a
## half-transcribed table is exactly the thing the next person re-derives
## wrongly. Odd-indexed entries are the gaps BETWEEN buzzes -- the Android
## `vibrate(long[])` convention minus its leading delay, which is always 0 here.
const _HAPTIC_MS := {
	"sample": [12], "detent": [8], "tool_arm": [10],
	"verdict": [14, 40, 14], "back": [6], "blocked": [20, 60, 20],
}

## `Input.vibrate_handheld()` is already a no-op off Android/iOS, but the
## feature test is made here rather than trusted: without it the multi-pulse
## kinds would still spend a `SceneTreeTimer` per gap on every desktop press.
func _haptic(kind: String) -> void:
	if not OS.has_feature("mobile"):
		return
	var pattern: Array = _HAPTIC_MS.get(kind, [])
	if pattern.is_empty():
		push_warning("Cartalith: no haptic '%s'." % kind)
		return
	var at := 0.0
	for i in pattern.size():
		var ms := int(pattern[i])
		if i % 2 == 0:
			if at <= 0.0:
				Input.vibrate_handheld(ms)
			else:
				get_tree().create_timer(at).timeout.connect(
					func() -> void: Input.vibrate_handheld(ms))
		at += float(ms) / 1000.0

## The on-screen keyboard, polled -- Godot raises no notification when the IME
## opens or closes. The Android window is not resized either (the soft-input
## mode is `adjustNothing` unless `project.godot` says otherwise, and this file
## may not edit it), so the keyboard simply draws over the bottom of the frame
## and everything docked there goes under it: the tool sheet, the timeline and
## the bottom bar. `BUILD_ANSWERS.md` §4 wires the prototype's `visualViewport`
## resize into `state.kb` and has `dockBottom` add it;
## `DisplayServer.virtual_keyboard_get_height()` is that number and a per-frame
## integer read is the only route to it.
##
## Phone-only and change-gated: everywhere else this returns 0 for the life of
## the process and only the first two lines ever run.
func _process(_delta: float) -> void:
	## A one-time self-disable rather than a per-frame branch. `_phone` is fixed
	## for the life of the process (`_on_window_resized()`'s early-return states
	## the argument), and a display server either has an IME or never will --
	## and polling one that does not pushes a "Virtual keyboard not supported"
	## warning PER CALL, which at 60 fps buries every real message in the log.
	## Found on the headless `_phonechrome_probe.gd` run, not reasoned about.
	if not _phone or not DisplayServer.has_feature(
			DisplayServer.FEATURE_VIRTUAL_KEYBOARD):
		set_process(false)
		return
	var kb := DisplayServer.virtual_keyboard_get_height()
	if kb == _phone_kb_height:
		return
	_phone_kb_height = kb
	## `_apply_phone_orientation()` owns every phone inset there is, keyboard
	## included, so the height lands in one place instead of two.
	_apply_phone_orientation()

# -- §2 Menu bar: program scope only ------------------------------------------

func _build_menu_bar() -> Control:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _scaled(DccTheme.H_MENU_BAR)
	bar.add_theme_stylebox_override("panel",
		DccTheme.panel("panel", {"bottom": 1}))

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 0)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_child(row)
	bar.add_child(pad)

	var wordmark := DccTheme.mono_label("CARTALITH", "text_bright", DccTheme.FS_MENU, 3, true)
	row.add_child(wordmark)
	## `margin-right:22px` on the canvas's own wordmark, and nothing else --
	## the reserved 150 px this used to claim opened a 74 px hole between
	## CARTALITH and File where the canvas has 22 px, which is the first thing
	## the eye lands on and the first thing that read as "not the design".
	var wordmark_gap := Control.new()
	wordmark_gap.custom_minimum_size.x = 22
	row.add_child(wordmark_gap)

	menu_bar_row = HBoxContainer.new()
	menu_bar_row.add_theme_constant_override("separation", 0)
	row.add_child(menu_bar_row)

	## Design children 3-5 (`03-menu-bar.md` §2): a `1x16` `var(--div)` rule
	## with `margin:0 6px`, then the undo and redo squares. Until this pass undo
	## was reachable only from `Edit ▸ Undo` and Ctrl+Z, so the bar's own
	## most-used control was one the design draws and this shell did not.
	var div_pad := MarginContainer.new()
	div_pad.add_theme_constant_override("margin_left", 6)
	div_pad.add_theme_constant_override("margin_right", 6)
	var div := DccTheme.rule(true)
	div.custom_minimum_size = Vector2(1, 16)
	div.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	div_pad.add_child(div)
	row.add_child(div_pad)

	_menu_undo_btn = _menu_square(DccIcons.SYMBOLS["undo"], _menu_bar_undo)
	## `↶`/`↷` are the whole control. Their *tooltips* are rewritten every
	## refresh (`_refresh_menu_squares()`) because they carry the step's name and
	## the disabled reason; the accessible NAME is the fixed verb, and the
	## changing half rides `accessibility_description` there.
	_menu_undo_btn.accessibility_name = "Undo"
	row.add_child(_menu_undo_btn)
	_menu_redo_btn = _menu_square(DccIcons.SYMBOLS["redo"], _menu_bar_redo)
	_menu_redo_btn.accessibility_name = "Redo"
	row.add_child(_menu_redo_btn)
	## Both start disabled, and stay that way on a bare `DccShell` -- the
	## screenshot probes build one with no `DccApp` and no engine under it.
	## That is this shell's own rule rather than a special case here: a square
	## that can do nothing is drawn dead, with the reason in its tooltip.
	_refresh_menu_squares()
	_wire_menu_squares()

	row.add_child(DccTheme.spacer())

	## The readout cluster: world, pass state, and the three cost meters. §11
	## keeps these in the menu bar because they describe the *program's* load,
	## not the world's content.
	for slot in ["world", "res", "cpu", "gpu", "mem"]:
		var l := DccTheme.mono_label("", "text_faint", DccTheme.FS_READOUT, 1)
		_status_labels["top_" + slot] = l
		row.add_child(l)
		var gap := Control.new()
		gap.custom_minimum_size.x = 22
		row.add_child(gap)

	## Design child 8: `◐`, the same `var(--ctl)` square, `color:var(--sec)`,
	## `margin-left:8px`. The 22 px gap the readout loop above emits after its
	## last cell stands in for that margin, so nothing extra is added here.
	##
	## Not a third source of truth for the theme: `toggle_theme()` is the same
	## `DccTheme.apply_theme()` + `rebuild_theme()` pair `Preferences ▸ Theme`
	## drives, so the square and the menu row move one piece of state.
	var theme_sq := _menu_square(GLYPH_THEME, toggle_theme)
	theme_sq.accessibility_name = "Switch theme"
	theme_sq.tooltip_text = ("Switch between the dark and light palettes. The same two "
		+ "palettes Preferences > Theme picks from; this square is the one-tap form of "
		+ "it and does not change the dark/light/follow-system mode stored there.")
	row.add_child(theme_sq)
	return bar

## `◐` U+25D0. Not added to `DccIcons.SYMBOLS`, which is the shared table and
## not this file's to extend; it resolves through the same `SystemFont` fallback
## chain `DccTheme.mono()` installs for the 19 entries of that table Plex Mono
## has no glyph for, so it draws exactly like the `↶` and `↷` beside it.
const GLYPH_THEME := "\u25d0"

## `--ctl`, the square-button box: 24 px at `ENV:25`, 36 px in `densStr`
## (`ENV:1819`). A literal pair rather than `_scaled()`, because
## `DccTheme.TABLET` does not name 24 and the 44 px floor its fallback applies
## would draw a 44 px square where the prototype draws 36.
const MENU_CTL := [24, 36]

## One `var(--ctl)` square: `width/height:var(--ctl); border-radius:8px;
## background:var(--ins); display:grid; place-items:center`.
##
## `undoCol`/`redoCol` were inside the prototype's truncated tail and do not
## exist to read (`03-menu-bar.md`'s own file-integrity header). The `ROLE`
## equivalents stand in, and they are the pair every other quiet control in this
## bar already uses: `--sec` (`text_secondary`) live, `--dis` (`text_ghost`)
## dead. `_paint_menu_square()` swaps between them, so "can I press this" is
## carried by ink as well as by the `disabled` flag.
func _menu_square(glyph: String, on_press: Callable) -> Button:
	var b := Button.new()
	b.text = glyph
	b.focus_mode = Control.FOCUS_NONE
	## Not `flat` -- `add_menu()`, `_phone_bar_button()` and `_phone_list_row()`
	## all carry the same note: a flat `Button` skips its `normal`/`hover`/
	## `pressed` styleboxes outright, so the `--ins` ground and the press
	## feedback set below would never once draw.
	b.flat = false
	var d: int = MENU_CTL[1] if _touch else MENU_CTL[0]
	b.custom_minimum_size = Vector2(d, d)
	b.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	b.add_theme_font_override("font", DccTheme.mono())
	b.add_theme_font_size_override("font_size", DccTheme.menu("fs_bar", _touch))
	b.add_theme_color_override("font_color", DccTheme.c("text_secondary"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	var ground := DccTheme.flat(DccTheme.c("sunken"), 8)
	b.add_theme_stylebox_override("normal", ground)
	b.add_theme_stylebox_override("disabled", ground)
	b.add_theme_stylebox_override("focus", DccTheme.empty())
	b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("raised"), 8))
	b.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("line_soft"), 8))
	b.pressed.connect(on_press)
	return b

## The dark/light flip behind design child 8, and behind the phone overflow's
## `Theme` row. Exactly what `DccMenus._on_theme_choice()` does for its own two
## explicit rows: repoint the palette, then walk the tree and re-derive every
## colour already baked off the old one.
##
## **It does not write `DccSettings.set_theme_mode()`, and neither does the
## Preferences row.** Nothing in this shell reads `theme_mode()` back -- checked
## this session, `grep -rn "theme_mode()" shell/` finds only its own definition
## in `dcc_settings.gd` -- so the theme is a session choice in both places.
## Persisting it from here alone would make the square survive a restart and the
## menu not, which is worse than either.
##
## `DccMenus._theme_mode` (its radio marks) is not updated by this call and goes
## stale after it -- that submenu builds its checks once and never re-reads them.
## Stated rather than silently left: the fix belongs in that file, which this
## pass does not own.
func toggle_theme() -> void:
	var was_dark := DccTheme.is_dark()
	DccTheme.apply_theme(not was_dark)
	rebuild_theme(was_dark)

## `DccApp.undo_last()`/`redo_last()` are this shell's single undo and redo
## paths -- each repaints the map without resetting the camera, writes the
## status line and refreshes the History dock -- and both live one class down,
## in the subclass this file is the base of. Reached by name rather than
## reimplemented here: a bare `DccShell` (the screenshot probes build one) has
## neither, and must degrade to doing nothing rather than reach a null bridge.
func _menu_bar_undo() -> void:
	if has_method("undo_last"):
		call("undo_last")
	_refresh_menu_squares()

func _menu_bar_redo() -> void:
	if has_method("redo_last"):
		call("redo_last")
	_refresh_menu_squares()

## Same deferred wiring, and for the same reason, as `_wire_phone_undo_chip()`:
## `bridge` is built by `DccApp._ready()` in the lines *after* the
## `super._ready()` that runs this file's builders, so there is nothing to ask
## until the next idle frame. The four signals are that function's four, chosen
## the same way -- a commit to the height field can arrive through any of them,
## and none of the others moves the undo stack.
func _wire_menu_squares() -> void:
	(func() -> void:
		var bridge := _find_engine_bridge()
		if bridge == null:
			return
		for sig in ["generation_finished", "params_applied", "world_loaded", "dirty_changed"]:
			bridge.connect(sig, func(_a = null): _refresh_menu_squares())
		_refresh_menu_squares()
	).call_deferred()

## Enabled-ness and the reason, for both squares.
##
## Redo is the one that can be *missing* rather than merely empty, so it is
## asked in two steps, the way `menus.gd`'s own `Redo` row asks it:
## `world_gen.has_method()` first (is the binding there at all), then
## `redo_available()` (is there a step to take). A shell newer than the
## `libcartalith_godot` beside it gets the first reason; a shell sitting at the
## top of its ledger gets the second. Neither is invented -- each names the call
## that was actually made.
func _refresh_menu_squares() -> void:
	if _menu_undo_btn == null or _menu_redo_btn == null:
		return
	var bridge := _find_engine_bridge()
	if bridge == null:
		for b in [_menu_undo_btn, _menu_redo_btn]:
			b.disabled = true
			b.tooltip_text = "No engine is loaded in this window."
			b.accessibility_description = b.tooltip_text
			_paint_menu_square(b, false)
		return

	var can_undo: bool = bridge.can_undo()
	_menu_undo_btn.disabled = not can_undo
	_menu_undo_btn.tooltip_text = ("Undo %s (Ctrl+Z)" % bridge.undo_label()) if can_undo \
		else ("Nothing to undo. This is the global height undo -- a Sculpt or Paint "
			+ "commit, a carve, a generate -- not the Sculpt draft's own stamp history, "
			+ "which has its own Undo in the right dock.")
	_paint_menu_square(_menu_undo_btn, can_undo)

	var bound: bool = bridge.world_gen != null \
		and bridge.world_gen.has_method("redo_available")
	var can_redo: bool = bound and bridge.redo_available()
	_menu_redo_btn.disabled = not can_redo
	if not bound:
		_menu_redo_btn.tooltip_text = ("Redo. This GDExtension build predates the global "
			+ "redo binding (WorldGen.redo_available is missing) -- almost always a native "
			+ "library older than this shell. Rebuild it.")
	elif can_redo:
		_menu_redo_btn.tooltip_text = "Redo %s (Ctrl+Shift+Z)" % bridge.redo_label()
	else:
		_menu_redo_btn.tooltip_text = ("Nothing to redo (WorldGen.redo_available() is "
			+ "false). Edit > Undo history... shows what the ledger is holding.")
	_paint_menu_square(_menu_redo_btn, can_redo)
	## The reason each square is live or dead, carried to a screen reader as
	## well as to a pointer. Written from the tooltips just set above rather
	## than restated, so the two cannot say different things.
	for b in [_menu_undo_btn, _menu_redo_btn]:
		b.accessibility_description = b.tooltip_text

## `--sec` live, `--dis` dead. `font_disabled_color` already resolves to
## `text_ghost`, so this exists for the enabled half; writing both keeps the
## square right for the frame between the two states.
func _paint_menu_square(b: Button, live: bool) -> void:
	b.add_theme_color_override("font_color",
		DccTheme.c("text_secondary" if live else "text_ghost"))

## Register a program menu. The caller fills the PopupMenu through `on_built`,
## so this file never has to know what File contains.
func add_menu(title: String, on_built: Callable) -> MenuButton:
	var mb := MenuButton.new()
	mb.text = title
	## **`flat` off, and this is not cosmetic.** `MenuButton` constructs itself
	## flat, and a flat `Button` skips its `normal`/`hover`/`pressed` styleboxes
	## outright -- the trap `viewport_host.gd` records paying for twice on the
	## Layers button. So the `pressed` override three lines down, which is the
	## canvas's own open-menu indicator
	## (`color:#e0a34a;background:rgba(224,163,74,.08);border-bottom:1px solid
	## #e0a34a`), had **never drawn**: sampled off the framebuffer with the File
	## menu open, the title's background was `#121314` (18,19,20) -- the panel
	## behind it -- identical to the closed `Edit` beside it. The most prominent
	## state cue in the application was invisible. `normal`/`hover` are
	## `StyleBoxEmpty` with the canvas's padding, so nothing else changes.
	mb.flat = false
	mb.focus_mode = Control.FOCUS_NONE
	## Prose face, not Plex -- `<span style="padding:9px 11px">File</span>` sits
	## inside a `font-size:11.5px;color:#a9adb0` run in the canvas, with no
	## font-family of its own, so it inherits `'Helvetica Neue'` from the
	## artboard root. No font override here at all is how a Control keeps
	## `dark_theme.tres`'s `default_font` (Fira Sans).
	##
	## The tablet column is `font-size:14px;padding:15px 15px` in
	## `DCC shell tablet 2560` -- measured, not scaled. Until 2026-08-25 this
	## row ignored `_touch` entirely, so a 2560x1600 tablet drew a 52 px menu
	## bar (§48 fixed the *bar*) carrying seven 40x51 desktop-sized titles in
	## 11 px type, none of which is a 44 px target in either dimension.
	var mfs := DccTheme.menu("fs_bar", _touch)
	var mpx := DccTheme.menu("bar_pad_x", _touch)
	var mpy := DccTheme.menu("bar_pad_y", _touch)
	mb.add_theme_font_size_override("font_size", mfs)
	mb.add_theme_color_override("font_color", DccTheme.c("text_secondary"))
	mb.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	mb.add_theme_stylebox_override("normal", DccTheme.inset(mpx, mpy, mpx, mpy))
	mb.add_theme_stylebox_override("hover", DccTheme.inset(mpx, mpy, mpx, mpy))
	mb.add_theme_stylebox_override("focus", DccTheme.inset(mpx, mpy, mpx, mpy))
	mb.add_theme_stylebox_override("disabled", DccTheme.inset(mpx, mpy, mpx, mpy))
	## `active_row()` is `accent_wash` (.08) plus a 1 px accent underline, which
	## is exactly what the canvas draws on an open title -- and the .08 here is
	## right, unlike the *item* inside the dropdown, which is .10. Two literals,
	## a few lines apart in the same artboard.
	var open_box := DccTheme.active_row()
	open_box.content_margin_left = mpx
	open_box.content_margin_right = mpx
	open_box.content_margin_top = mpy
	open_box.content_margin_bottom = mpy
	mb.add_theme_stylebox_override("pressed", open_box)
	mb.add_theme_color_override("font_pressed_color", DccTheme.c("accent"))
	menu_bar_row.add_child(mb)
	var popup := mb.get_popup()
	style_popup(popup)
	on_built.call(popup)
	return mb

## The canvas's own menu panel. **The body moved to `DccWidgets.style_popup()`
## 2026-08-25** and this is now a delegate, kept because `menus.gd` and
## `phone_menu.gd` both call it by this name.
##
## Why it moved: `PopupMenu` is not only the seven program menus. Every
## `OptionButton` in the shell owns one too, and none of them had ever been
## styled -- the whole dropdown vocabulary of the application (every dock
## picker, every dialog select, the paint target, the bake depth) was opening
## Godot's stock dark theme, a `#0f0f0f` panel with a grey selection bar, in a
## shell whose palette is `#121314` plus one amber. `dropdown()` is a static
## factory with no shell to reach, so the styling had to become static too --
## see `DccTheme.is_touch()`, which exists for exactly this call.
func style_popup(popup: PopupMenu) -> void:
	DccWidgets.style_popup(popup)

# -- PR-13/PR-14 Theme rebuild --------------------------------------------------
#
# `DccTheme.apply_theme()` only re-points which palette `c()` resolves
# against; it repaints nothing, because every node that already called `c()`
# baked a plain `Color` value into its own `add_theme_*_override`, not a live
# reference. This is the other half: walk the whole tree and re-derive every
# one of those baked values from the token that produced it.
#
# Godot exposes no "list every override this node has" call, so the two
# arrays below are the exhaustive set of override *names* this codebase
# actually uses -- grepped, not guessed:
#   grep -rhoE 'add_theme_(color|stylebox)_override\("[^"]+"' godot-project/shell
# The *values* need no such list: `DccTheme.remap()` reverse-looks-up
# whichever token in the *old* palette produced the colour already sitting on
# a node, and repaints it with that same token's new value. A colour that
# matches no token (a literal, e.g. a phone overlay's plain black scrim) is
# left alone -- there is nothing to remap it to.
#
# This walks every node under the shell root, so it reaches workspace panels,
# popups and dialogs too, not just the frame chrome `DccShell` itself builds
# -- but only nodes that already exist. A dialog that has never been opened
# yet builds itself fresh from `DccTheme.c()` the first time it opens, which
# already picks up the new palette; nothing extra is needed for those.

## Re-grepped 2026-08-20 (owner: "make sure the lightmode version is available
## everywhere"). Six names had accumulated since the lists were first written
## and were therefore never repainted by a theme switch -- exactly the drift
## the comment above predicts, introduced by the windows built *after* the
## theme pass. `caret_color`/`font_placeholder_color`/`font_uneditable_color`
## and the `read_only`/`disabled`/`focus` styleboxes all come from
## `dcc_widgets.gd`'s text fields, which is why every dialog with a text well
## (the browse dialogs, the asset library, the data manager, this file's own
## search) kept dark input wells under the light palette.
const _THEME_COLOR_OVERRIDES := [
	"caret_color", "default_color", "font_accelerator_color", "font_color",
	"font_disabled_color", "font_hover_color", "font_placeholder_color",
	"font_pressed_color", "font_separator_color", "font_uneditable_color",
	"icon_hover_color", "icon_normal_color", "icon_pressed_color",
]
const _THEME_STYLEBOX_OVERRIDES := [
	"disabled", "focus", "grabber_area", "grabber_area_highlight", "hover", "normal",
	"panel", "pressed", "read_only", "separator", "slider",
]

## Called by `menus.gd` immediately after `DccTheme.apply_theme()`, passing
## whichever palette was active a moment ago (`was_dark`) so the walk knows
## what it's reversing.
func rebuild_theme(was_dark: bool) -> void:
	var old_pal: Dictionary = DccTheme.DARK if was_dark else DccTheme.LIGHT
	_recolor_project_theme(old_pal)
	_style_window_chrome()
	_recolor_subtree(self, old_pal)
	## The phone top scrim used to need a third pass here: its colour lived
	## inside a `GradientTexture2D`, on no node and in no theme resource, so
	## neither walk above could see it, and a light-palette capture found a
	## charcoal band over a light screen. The 412 canvas draws no scrim -- the
	## status row is a plain `panel` stylebox now, which `_recolor_subtree()`
	## reaches like every other region.

## The other half of "everywhere", found 2026-08-20 by capturing every window
## under the light palette instead of trusting the walk.
##
## `project.godot` sets `gui/theme/custom` to a real, hand-authored dark
## `Theme` resource, and that resource is the fallback for every control state
## nothing overrides explicitly -- disabled buttons, focus rings, scrollbars,
## tooltips, popup separators, `SpinBox`/`OptionButton`/`CheckBox` chrome. None
## of it is a per-node `add_theme_*_override`, so `_recolor_subtree()` below
## could never have reached any of it: the colours live inside a `Resource`,
## not on the nodes. That is why a disabled `DccWidgets.action()` button (which
## sets `normal`/`hover` but no `disabled` stylebox) stayed a dark slab on a
## light shell -- "Bake ALL & finalize" and the world workspace's own
## "Finalize · LOD 0-3", both visibly wrong in the light capture.
##
## Remapping works because that resource was authored from these exact tokens:
## its header lists surface `#0d0e0f`, text `#c8cbcd`, accent `#e0a34a` and the
## rest, which are `DccTheme.DARK`'s values verbatim. So the same reverse
## lookup the node walk uses converts the whole resource.
##
## Mutating it is in-memory only -- `load()` returns the cached instance the
## whole tree is already resolving against, so this repaints every fallback at
## once and nothing is written back to disk. Switching back re-runs it with the
## palettes swapped. A `StyleBoxFlat` shared by several entries is visited more
## than once, which is harmless: after the first visit its colour no longer
## matches anything in `old_pal`, so `remap()` returns null and leaves it be.
func _recolor_project_theme(old_pal: Dictionary) -> void:
	var path := String(ProjectSettings.get_setting("gui/theme/custom", ""))
	if path == "":
		return
	var th := load(path) as Theme
	if th == null:
		return
	var extras := _theme_extras(was_dark_to_light(old_pal))
	## See `_bulk_theme_edit()`: without this the walk costs 27 s on a phone.
	th.set_block_signals(true)
	for type_name in th.get_color_type_list():
		for color_name in th.get_color_list(type_name):
			var nc = _remap_theme_color(th.get_color(color_name, type_name), old_pal, extras)
			if nc != null:
				th.set_color(color_name, type_name, nc)
	for type_name in th.get_stylebox_type_list():
		for box_name in th.get_stylebox_list(type_name):
			var sb := th.get_stylebox(box_name, type_name)
			if sb is StyleBoxFlat:
				var f := sb as StyleBoxFlat
				var nb = _remap_theme_color(f.bg_color, old_pal, extras)
				if nb != null:
					f.bg_color = nb
				var nr = _remap_theme_color(f.border_color, old_pal, extras)
				if nr != null:
					f.border_color = nr
	_bulk_theme_edit(th)

## Ends a batch of `Theme` mutations: unblocks the resource's signals and fires
## `changed` exactly once.
##
## **Every `set_color()`/`set_stylebox()` on a live `Theme` emits `changed`, and
## that re-propagates `NOTIFICATION_THEME_CHANGED` to every `Control` in the
## tree.** Each edit therefore costs a whole-tree relayout, and this shell's
## tree is large -- the phone chrome, the parked desktop menu/status model, the
## dock sheets and every runtime-built window at once.
##
## Measured on the real OnePlus 6T before this batching, switching
## `Preferences ▸ Theme` to Light froze the main thread for **29.6 s**
## (`projectTheme=27336ms windowChrome=1597ms subtree=670ms`). The giveaway is
## `windowChrome`: it performs just **5** theme writes and still cost 1.6 s, or
## ~320 ms *per write* -- the cost is per mutation, not per colour examined. A
## first attempt at memoising the colour lookups was therefore wasted (27336 ->
## 27632 ms, i.e. no change) and was removed rather than kept as decoration.
##
## Found only because the phone menu made `Theme` reachable by finger for the
## first time. On desktop the same freeze exists but reads as a hitch on a
## machine that is ~50x faster per node; on the phone it looked like a **dead
## tap**, and was twice mistaken for a lost touch event before the log
## timestamps showed a 29.6 s round trip between press and repaint.
func _bulk_theme_edit(th: Theme) -> void:
	th.set_block_signals(false)
	th.emit_changed()

func was_dark_to_light(old_pal: Dictionary) -> bool:
	return old_pal == DccTheme.DARK

## Embedded `Window` chrome -- the title bar and its close button that every
## `AcceptDialog` in this shell draws above its own branded header. The project
## theme resource defines no `Window` entries at all, so that chrome came from
## Godot's stock built-in theme, which is dark and fixed: under the light
## palette every dialog wore a charcoal title bar over light content, and no
## amount of remapping could reach it because there was nothing to remap.
##
## Written from tokens rather than remapped, so it is correct on a cold boot in
## either palette as well as after a switch -- hence the call from `_ready()`
## as well as from `rebuild_theme()`.
func _style_window_chrome() -> void:
	var path := String(ProjectSettings.get_setting("gui/theme/custom", ""))
	if path == "":
		return
	var th := load(path) as Theme
	if th == null:
		return
	## Five writes, and each one used to cost ~320 ms on the phone -- see
	## `_bulk_theme_edit()`, which is where that number was measured.
	th.set_block_signals(true)
	th.set_color("title_color", "Window", DccTheme.c("text_bright"))
	th.set_color("title_outline_modulate", "Window", DccTheme.c("raised"))
	for box in ["embedded_border", "embedded_unfocused_border"]:
		th.set_stylebox(box, "Window", DccTheme.panel("raised",
			{"left": 1, "right": 1, "top": 1, "bottom": 1}))
	## `AcceptDialog` draws its *own* `panel` on top of the Window border, and
	## nothing here had ever set it -- so Performance, Gen info, World data and
	## the footer band of every modal came up on Godot's stock `#404040` grey,
	## sampled off the framebuffer 2026-08-25. `#404040` is not a token in
	## either palette and is 20 steps brighter than anything the canvas draws.
	## `AcceptDialog` inherits `PanelContainer`'s type only for *some* boxes,
	## which is why the shell's own `PanelContainer` styling never reached it.
	var dlg := DccTheme.panel("panel",
		{"left": 1, "right": 1, "top": 1, "bottom": 1})
	dlg.border_color = DccTheme.c("border")
	dlg.content_margin_left = 0
	dlg.content_margin_right = 0
	dlg.content_margin_top = 0
	dlg.content_margin_bottom = 0
	th.set_stylebox("panel", "AcceptDialog", dlg)
	th.set_stylebox("panel", "PopupPanel", dlg)

	## Tabs. `TabContainer` draws its strip from an internal `TabBar` that no
	## walk in this file reaches, so World data, Travel library, the Asset
	## library and the Data manager were all showing Godot's stock tab chrome:
	## a raised grey pill with a white top rule on the selected tab. The
	## design's own two-way switch -- the left dock's `GENERATION PIPELINE |
	## SCULPT` header -- is `background:rgba(224,163,74,.10)` with
	## `border-bottom:1px solid #e0a34a` when on and nothing at all when off,
	## which is `active_row()`, the same shape the menu bar and every active
	## dock row already use. Set on both type names because a bare `TabBar`
	## does not inherit `TabContainer`'s.
	##
	## **This does not reach the Data manager's *phone* pane switcher.**
	## `data_manager_window.gd::_build_phone_switcher()` is a bespoke `Button`
	## row, by design (its own comment: "See [asset_library_window.gd] for why
	## this is a segmented row and not a `TabContainer`") -- so it is a
	## `Button`, not a `TabContainer`/`TabBar`, and this override cannot touch
	## it. Its buttons never got an explicit "hover" override either, so an
	## Android touch tap -- which leaves the emulated pointer parked where the
	## finger last was, the same mechanism `viewport_host.gd`'s navpad pill
	## measured stuck -- can leave one showing Godot's own default `Button`
	## hover chrome permanently (`GUI_GAP_REGISTER.md`'s phone residue: "the
	## pane switcher's focused state is stock Godot... permanent on touch").
	## `data_manager_window.gd` is not a file this pass owns; recorded here
	## rather than fixed, since the claim two lines up that this override
	## already covered "the Data manager" is what sent this lane looking.
	for type_name in ["TabContainer", "TabBar"]:
		var on := DccTheme.active_row(true)
		on.content_margin_left = 14
		on.content_margin_right = 14
		on.content_margin_top = 7
		on.content_margin_bottom = 7
		th.set_stylebox("tab_selected", type_name, on)
		var off := DccTheme.flat(Color(0, 0, 0, 0))
		off.content_margin_left = 14
		off.content_margin_right = 14
		off.content_margin_top = 7
		off.content_margin_bottom = 7
		th.set_stylebox("tab_unselected", type_name, off)
		th.set_stylebox("tab_hovered", type_name,
			DccTheme.flat(DccTheme.c("line_soft")))
		th.set_stylebox("tab_focus", type_name, DccTheme.empty())
		th.set_color("font_selected_color", type_name, DccTheme.c("accent"))
		th.set_color("font_unselected_color", type_name, DccTheme.c("text_dim"))
		th.set_color("font_hovered_color", type_name, DccTheme.c("text_bright"))
	th.set_stylebox("panel", "TabContainer", DccTheme.panel("panel", {"top": 1}))
	th.set_stylebox("tabbar_background", "TabContainer", DccTheme.empty())
	_bulk_theme_edit(th)

## `DccTheme.remap()` first, then the supplementary table below for the
## handful of colours the theme resource uses that are not tokens at all.
func _remap_theme_color(value: Color, old_pal: Dictionary, extras: Dictionary) -> Variant:
	var key := value.to_html(false)
	if extras.has(key):
		return Color(extras[key] as Color, value.a)
	return DccTheme.remap(value, old_pal)

## The theme resource predates `DccTheme` and its header claims to use the same
## values; six of them do not, so the token reverse-lookup cannot see them and
## they stayed dark under the light palette. Measured by dumping every distinct
## `Color(...)` in the `.tres` and diffing against both palettes, not guessed.
##
## Each entry is a derivation, not a new colour invented here: two are plain
## surfaces, one is a token with a one-digit typo, and two are the accent with
## the same lighten/darken the widgets already apply to it
## (`DccWidgets.action()` uses `c("accent").lightened(0.1)`).
##
## Deliberately absent, because they are correct in both palettes and are not
## misses: `#1a1206`, the near-black used for text sitting *on* the amber slab,
## which must stay dark on a light ground too, and `#e66b6b`, the error red --
## the same reasoning `DccTheme` already applies to `warn`/`block`/`water`.
func _theme_extras(to_light: bool) -> Dictionary:
	var accent: Color = DccTheme.LIGHT["accent"] if to_light else DccTheme.DARK["accent"]
	var out := {
		## Intended as `text_dim`; the resource has 0x96 where the token has
		## 0x92 in blue, which is enough to defeat an exact-match lookup.
		"8d9396": "text_dim",
		"131416": "panel",    ## The panel surface behind field chrome.
		"1a1b1d": "sunken",   ## Input well, resting.
		"252729": "raised",   ## Input well, hover.
	}
	var pal: Dictionary = DccTheme.LIGHT if to_light else DccTheme.DARK
	var map := {}
	for hex in out:
		map[hex] = pal[out[hex]] as Color
	map["c48c38"] = accent.darkened(0.125)   ## Accent, pressed.
	map["edb45f"] = accent.lightened(0.09)   ## Accent, hover.
	if to_light:
		return map
	## Reversing: the light run wrote the light values, so key the table by
	## those instead. Same four tokens, same two derivations.
	var rev := {}
	for hex in out:
		rev[(DccTheme.LIGHT[out[hex]] as Color).to_html(false)] = pal[out[hex]] as Color
	rev[(DccTheme.LIGHT["accent"] as Color).darkened(0.125).to_html(false)] = accent.darkened(0.125)
	rev[(DccTheme.LIGHT["accent"] as Color).lightened(0.09).to_html(false)] = accent.lightened(0.09)
	return rev

func _recolor_subtree(node: Node, old_pal: Dictionary) -> void:
	if node is Control or node is Window:
		for name in _THEME_COLOR_OVERRIDES:
			if node.has_theme_color_override(name):
				var nc = DccTheme.remap(node.get_theme_color(name), old_pal)
				if nc != null:
					node.add_theme_color_override(name, nc)
		for name in _THEME_STYLEBOX_OVERRIDES:
			if node.has_theme_stylebox_override(name):
				var sb: StyleBox = node.get_theme_stylebox(name)
				if sb is StyleBoxFlat:
					_recolor_stylebox(sb, old_pal)
				elif sb is StyleBoxLine:
					## The menu separator (`style_popup()`), the one non-Flat
					## box the shell authors. Silently skipped before it
					## existed, which would have left a dark hairline across a
					## light menu.
					var lc = DccTheme.remap((sb as StyleBoxLine).color, old_pal)
					if lc != null:
						(sb as StyleBoxLine).color = lc
	if node is ColorRect:
		var nc = DccTheme.remap((node as ColorRect).color, old_pal)
		if nc != null:
			(node as ColorRect).color = nc
	for child in node.get_children():
		_recolor_subtree(child, old_pal)

func _recolor_stylebox(sb: StyleBoxFlat, old_pal: Dictionary) -> void:
	var new_bg = DccTheme.remap(sb.bg_color, old_pal)
	if new_bg != null:
		sb.bg_color = new_bg
	var new_border = DccTheme.remap(sb.border_color, old_pal)
	if new_border != null:
		sb.border_color = new_border

# -- §4 Tool options bar ------------------------------------------------------

func _build_tool_options_bar() -> Control:
	var bar := PanelContainer.new()
	## `role_px()`, **not** `_scaled()`, and this is the one region in the shell
	## that cannot use `_scaled()`. The 2026-08-31 token re-base put `--tbH` at
	## 40 px, which is also `--railW`, and `DccTheme.TABLET` is keyed by the
	## bare desktop integer -- so `_scaled(40)` has to mean one thing and the
	## rail already owns it (40 -> 48; this bar needs 40 -> 56). `ROLE` exists
	## for exactly this collision; see `DccTheme.TABLET`'s header for the full
	## account, and note that `_scaled()` is still correct for the other three
	## bands, whose keys stayed unique.
	bar.custom_minimum_size.y = DccTheme.role_px("h_tool_options")
	bar.add_theme_stylebox_override("panel",
		DccTheme.panel("panel_alt", {"bottom": 1}))
	tool_options_row = HBoxContainer.new()
	tool_options_row.add_theme_constant_override("separation", 14)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_child(tool_options_row)
	bar.add_child(pad)
	return bar

## Replace the bar's contents. §4: it holds the active tool's frequently-changed
## values and its commit/discard, and never a control belonging to another tool
## -- so switching tools clears it rather than appending to it.
func set_tool_options(build: Callable) -> void:
	for child in tool_options_row.get_children():
		tool_options_row.remove_child(child)
		child.queue_free()
	build.call(tool_options_row)
	## Phone only: the tool sheet's height tracks its content
	## (`_build_phone_tool_sheet()` sets no fixed height), so a domain switch
	## can change how far `ViewportHost`'s corner chrome needs to clear it.
	## Deferred one frame so `_phone_bottom_reserve()` reads the sheet's real
	## post-layout size rather than its size from before this rebuild.
	if _phone:
		## `wide`: this row lives inside a horizontally scrolling sheet, so it
		## is the one subtree that must NOT be squeezed to fit the screen --
		## see `phone_fit()`'s own header.
		phone_fit(tool_options_row, _phone_scale, true)
		(func(): phone_insets_changed.emit()).call_deferred()
	elif DccTheme.is_tablet():
		## `tool_options_row` is rebuilt continuously (every tool-mode switch),
		## outside `register_workspace()`'s one-time walk -- and it is the one
		## place a caller this pass does not own (`tool_bar.gd`'s
		## `_tool_segment()`) sets a raw `custom_minimum_size.y` on a
		## `DccWidgets.segment()` button *after* the factory already sized it,
		## which would otherwise silently undo that fix on every rebuild. Run
		## after `build.call()` above, this floors it back up rather than
		## reporting a fault this choke point can trivially close.
		tablet_fit(tool_options_row)

## Owner, 2026-08-20: "the bottom menu butons on phone are near too small to
## use". They were, and the earlier "44 px lands at ~121 physical px" arithmetic
## was measuring the wrong thing -- it described the *chrome* (`_build_phone_app
## _bar()`, the domain rail of the day), which does route every size through
## `_ptap()`. The sheet's *contents* never touched `_ptap()` at all: they are
## built by the workspaces' own `_build_*_tool_options_row()` callbacks against
## desktop pixel constants (`cartography_workspace.gd` sets buttons to a literal
## `Vector2(34, 20)`), and Godot's default stretch mode is disabled, so 20
## virtual px is 20 *physical* px -- about 1.6 mm on this 314 dpi panel.
##
## Fixed here rather than in the workspaces because `set_tool_options()` is the
## single choke point all of them already pass through, so one pass over the
## finished row phone-sizes every current and future tool row without making a
## dozen workspace files phone-aware (and without touching files another agent
## may be mid-flight in). Applied after `build.call()` so it sees the real
## nodes, and re-applied on every rebuild because each one makes fresh ones.
##
## Generalised 2026-08-24 from the tool row to any subtree, because the dock
## *sheets* and the three civ windows had exactly the same disease: every row
## in them comes from `dcc_widgets.gd`, which is authored in desktop pixels
## (`_row` is 24 px tall, `slider` 14, `action` 26, `tool_button` 30x30) and
## knows nothing about a phone. One walker fixes all of them; the alternative
## was making a dozen panel files phone-aware, several of which other agents
## are mid-flight in.
##
## `unit` is what one authored pixel is worth in this subtree's own space:
##   - `_phone_scale` for anything laid out in the main viewport (the docks,
##     the tool row) -- there is no content scale there, so a 24 px row really
##     is 24 physical px, about 2 mm.
##   - `1.0` for a `Window` that has already set `content_scale_factor` to
##     `_phone_scale` (the three civ windows, `open_project_dialog.gd`'s own
##     treatment): the scale is applied once by the compositor, and applying it
##     again here would double it.
##
## Idempotent by meta-flag, because the dock pass below re-runs on every
## rebuild and a second multiplication would grow every row without bound.
const _PHONE_FIT_META := "_phone_fitted"

## §13's touch-scroll deadzone, in authored pixels. See `phone_fit()`'s own
## `ScrollContainer` branch for why leaving it at Godot's 0 is not an option
## once a button forwards its drag.
const PHONE_SCROLL_DEADZONE := 10

## **The font-size override is not always called `font_size`.** Every control
## in this shell but one carries the generic name, so the walk below checked
## only that -- and a `RichTextLabel` does not have it at all. Its own sizes
## are five separate theme items, one per style, and setting `font_size` on
## one does nothing whatsoever. The right dock's "Why here?" causal chain
## (`right_dock.gd`, `normal_font_size` = `FS_SMALL`) was therefore skipped in
## silence and drew at a flat 11 *physical* px on a 1080-wide handset, about a
## third the height of every row above it. Measured on the device; there is no
## warning and no visible failure anywhere else.
const _FONT_SIZE_KEYS: PackedStringArray = ["font_size"]
const _RICH_FONT_SIZE_KEYS: PackedStringArray = [
	"normal_font_size", "bold_font_size", "italic_font_size",
	"bold_italic_font_size", "mono_font_size"]

## `wide` says the subtree scrolls horizontally, so nothing in it has to be
## made to fit a 393 dp column. Exactly one caller sets it -- the phone tool
## sheet, which `_build_phone_tool_sheet()` wraps in a `SCROLL_MODE_AUTO`
## `ScrollContainer` -- and it turns off the two width-shrinking measures
## below, both of which are wrong there and one of which was actively
## breaking it. PAINT ▸ Class is the case that found it: `fit_to_longest_item
## = false` plus `clip_text` leaves an `OptionButton` with **no** content-
## derived minimum width at all. Down a dock that is invisible, because the
## row is full width and the control expands into it; in the tool sheet the
## row is one of six side by side and none of them expands, so the control
## collapsed onto its own drop-down arrow -- 35 px, showing which class is
## selected nowhere. Sizing it from its longest item instead just makes the
## sheet a little wider, and the sheet already scrolls.
func phone_fit(node: Node, unit: float, wide: bool = false) -> void:
	for child in node.get_children():
		if child is Control and not child.has_meta(_PHONE_FIT_META):
			var ctl := child as Control
			ctl.set_meta(_PHONE_FIT_META, true)
			## Explicit font-size overrides beat any theme we could hang on the
			## sheet, so they have to be re-written rather than inherited.
			## See `_RICH_FONT_SIZE_KEYS` for why the name is asked for per
			## control class rather than assumed to be `font_size`.
			if unit != 1.0:
				var rich := ctl is RichTextLabel
				var scaled_any := false
				for key in (_RICH_FONT_SIZE_KEYS if rich else _FONT_SIZE_KEYS):
					if ctl.has_theme_font_size_override(key):
						ctl.add_theme_font_size_override(key,
							maxi(1, int(round(ctl.get_theme_font_size(key) * unit))))
						scaled_any = true
				## A `RichTextLabel` that overrides *nothing* still needs the
				## pass: it is pure text with no minimum-size floor to catch
				## it, so left alone it renders at the stock theme size, which
				## on a phone is the same unscaled physical pixel the override
				## case was. `app.gd`'s credits body is the other one in this
				## shell. Resolved off the theme rather than hard-coded, so a
				## re-themed default still lands right.
				if rich and not scaled_any:
					ctl.add_theme_font_size_override("normal_font_size",
						maxi(1, int(round(ctl.get_theme_font_size("normal_font_size") * unit))))
			## Scale whatever the desktop row asked for, then floor anything
			## tappable at §13's 44 px -- the floor is the half the owner felt.
			var tap := maxf(1.0, round(DccTheme.PHONE_TAP_MIN * unit))
			var min_size := ctl.custom_minimum_size
			if min_size.x > 0.0:
				min_size.x = round(min_size.x * unit)
			if min_size.y > 0.0:
				min_size.y = round(min_size.y * unit)
			if ctl is BaseButton or ctl is LineEdit or ctl is Range or ctl is TextEdit:
				min_size.y = maxf(min_size.y, tap)
				if min_size.x > 0.0:
					min_size.x = maxf(min_size.x, tap)
			ctl.custom_minimum_size = min_size
			## §4.5's TOOLS block. The floor above grew each tool's *box* to
			## 44 dp and left everything inside it exactly as authored, which is
			## the whole fault: `dcc_widgets.gd`'s `tool_button` is a 15 px
			## glyph, an **empty** `normal` stylebox, and the tool's name in a
			## tooltip. On a pointer that is a complete control -- hover names
			## it, and 30 px is a comfortable target. On a handset it is
			## neither. There is no hover, so the name is unreachable by any
			## route at all; and 15 px stays 15 *physical* px, about a
			## millimetre on a 400 ppi panel, sitting left-aligned in a 121 px
			## cell with no border to say where the button even is. CIVIL's
			## block is seven such marks (Inspect, Measure, Region select,
			## Settlement, Territory, Way, Route) with nothing to tell any of
			## them apart. Measured on the device, and exactly the class of
			## fault no headless check can see.
			if ctl is Button and ctl.has_meta(DccWidgets.TOOL_GLYPH_META):
				_phone_fit_tool_button(ctl as Button, unit)
			## The 412 canvas's action button: a 48 dp pill, filled for the
			## primary and outlined for the secondary. Only reached from here, so
			## the desktop chip is untouched everywhere else -- see
			## `DccWidgets.phone_pill()`.
			elif ctl is Button and ctl.has_meta(DccWidgets.ACTION_META):
				DccWidgets.phone_pill(ctl as Button, unit)
			## 3 px track, 22 dp round thumb, 32 dp row. The dock's slider has no
			## grabber at all by §11; the phone canvas draws one on every slider
			## it has, because a finger has no cursor to find the handle with.
			if ctl is HSlider:
				DccWidgets.phone_slider(ctl as HSlider, unit)
			## **A drag that starts on a row has to reach the scroll above it.**
			## `dcc_widgets.gd` builds every row as an `HBoxContainer`, and a
			## `Control` picks by default (`MOUSE_FILTER_STOP`), which ends the
			## event walk right there -- Godot delivers a GUI event to the picked
			## control and then up its parents, stopping at the first `STOP`. On a
			## pointer that costs nothing, because scrolling is the wheel. On a
			## phone it is the whole gesture: the left dock sheet could only be
			## scrolled by catching its 4 px scrollbar, which on a 400 ppi panel
			## is about a millimetre, so the NPR Painter block below the fold was
			## effectively unreachable. Found by driving the real handset -- a
			## flick on the rows did nothing, the same flick on the scrollbar
			## worked.
			##
			## `PASS`, not `IGNORE`: a `PASS` control is still picked, so the
			## row keeps its own tooltip and hover, and only *forwards* what it
			## does not handle. Layout containers only -- a `PanelContainer` is
			## excluded because several in this shell (`phone_menu.gd`'s rows,
			## the roster's folded bar) carry their own `gui_input` and must
			## keep stopping the event they consume.
			if (ctl is BoxContainer or ctl is MarginContainer) \
					and ctl.mouse_filter == Control.MOUSE_FILTER_STOP:
				ctl.mouse_filter = Control.MOUSE_FILTER_PASS
			## PH-05's last hole, found by re-running the flick sweep at
			## `_phone_scale` 2.748 rather than at the 393 dp reference every earlier
			## probe used: 6 of 8 points down the left sheet scrolled 329 px and two
			## did not. One is an `HSlider`, which is deliberate and is explained
			## below. The other is a **bare `Control`** -- `DccTheme.spacer()` and the
			## fixed-width gaps beside it -- which defaults to `MOUSE_FILTER_STOP` and
			## so ends the event walk on a node that exists only to take up room.
			##
			## Matched on the exact class rather than `is Control`, because every
			## control in this shell is one; and skipped if anything is listening on
			## `gui_input`, since a plain `Control` with a handler (a scrim, a drag
			## handle) is picking on purpose. A spacer with neither has nothing to
			## consume the event it is currently swallowing.
			if ctl.get_class() == "Control" and ctl.get_script() == null \
					and ctl.mouse_filter == Control.MOUSE_FILTER_STOP \
					and ctl.get_signal_connection_list("gui_input").is_empty():
				ctl.mouse_filter = Control.MOUSE_FILTER_PASS
			## PH-05, the other half of the same sentence -- and the half that was
			## actually load-bearing. A `Container` already defaults to `PASS`
			## (measured, 4.7.1: `MOUSE_FILTER_PASS`, not the `Control` default the
			## comment above assumed), so the rows were never the blocker. **A
			## `Button` is.** `_scrolldrag_probe.gd` flicked twenty points down the
			## left sheet: every point that failed to scroll was a `Button` or an
			## `HSlider`, and from the accordion down the sheet is nothing *but*
			## buttons -- the L2 `category()` headers, the L4 `group()` headers, every
			## `action()`. That is the "a flick on the content does nothing" the
			## handset found, and it is why the scrollbar still worked: only the
			## content was covered.
			##
			## `PASS` is safe on a button *because* `ScrollContainer` and `BaseButton`
			## already cooperate: past the deadzone the scroll propagates
			## `NOTIFICATION_SCROLL_BEGIN`, which cancels the button's pending press.
			## Measured, all four cases: a clean tap fires, a 2 px and a 6 px wobble
			## still fire, an eight-sample flick scrolls 96 px and fires nothing.
			##
			## An `HSlider` is deliberately **not** included: a drag that starts on a
			## slider means "move this slider", on every touch platform there is.
			##
			## Neither are the three `BaseButton`s that open a `Popup` on *press* --
			## and not for symmetry: such a control pops mid-flick, the popup grabs
			## the drag, and the gesture then neither scrolls nor is undone (measured
			## on `OptionButton`: popup open, scroll 0). Their rows still scroll from
			## the label beside them.
			if ctl is BaseButton and ctl.mouse_filter == Control.MOUSE_FILTER_STOP 					and not (ctl is OptionButton or ctl is MenuButton 						or ctl is ColorPickerButton):
				ctl.mouse_filter = Control.MOUSE_FILTER_PASS
			## That deadzone is not a default -- Godot's is **0**, at which the ~2 px
			## of wobble in a real thumb tap already counts as a drag and silently
			## eats the press. Without this, the fix above would trade "the sheet does
			## not scroll" for "the buttons do not press". Scaled with the rest of the
			## subtree, so it is the same physical distance in a dock (unscaled,
			## `unit` = `_phone_scale`) as in a content-scaled window (`unit` = 1.0).
			if ctl is ScrollContainer:
				(ctl as ScrollContainer).scroll_deadzone = maxi(
					PHONE_SCROLL_DEADZONE, int(round(PHONE_SCROLL_DEADZONE * unit)))
			## An `OptionButton`'s list is a `PopupMenu`, which is a `Window` and
			## not a `Control` -- so it is not in this walk and inherits none of
			## the above. Left alone its rows came out at ~21 dp inside a
			## content-scaled window (measured on the handset, City Viewer's
			## settlement picker: 40 names at half the tap floor), and at ~8 dp
			## in a dock, where nothing scales the stock font either.
			##
			## A `PopupMenu` has no row-height property; a row is its font plus
			## `v_separation`, so those are the two knobs. 22 is what brings a
			## default row to the 44 dp floor rather than an arbitrary bump.
			## **A `Button` reports its own text as its minimum width**, and a
			## `Window` cannot be narrower than its content's minimum -- so one
			## long label anywhere inside a dialog widens the whole window past
			## the screen, and everything laid out after the scrolling pane goes
			## with it. The faction roster is the case that found this: its
			## settlement sublist ("Draumr League — 13 settlements, 39 210") and
			## its vocabulary pickers put the content minimum at 473 px on a
			## 393 dp screen, the window grew to fit, and the Add/Remove row
			## ended up 1 750 px below the bottom of the phone. Measured with
			## `get_combined_minimum_size()` down the dialog's own tree in a
			## `--force-touch` run, not guessed.
			##
			## Trimming takes the text out of that calculation while still
			## drawing it in full wherever it fits -- which at 393 dp is nearly
			## everywhere -- and marks the rest with an ellipsis rather than
			## cutting mid-glyph.
			##
			## Only for a control that already stretches. A `Button` sized by
			## its own text and *not* expanding has nothing else to get a width
			## from, so trimming it collapses it to nothing -- the roster's
			## Add/Remove pair went to zero width the first time this was
			## applied to every button alike.
			if not wide and ctl is Button and (ctl.size_flags_horizontal & Control.SIZE_EXPAND) != 0:
				(ctl as Button).clip_text = true
				(ctl as Button).text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
			## The `Label` half of the hole just closed for `Button`.
			## `GUI_GAP_REGISTER.md`'s phone residue: World data ▸ Economy rows
			## ending `…silver, clay, buildst` -- a hard cut with no ellipsis,
			## because this walk had only ever reached `Button.clip_text`.
			## `dcc_widgets.gd`'s own `_project_picker` header hit the identical
			## shape once already (a path label running off the screen) and the
			## fix there is the same pair of properties, `Label.clip_text` +
			## `text_overrun_behavior` -- just never generalised to this walk.
			##
			## Same guard as the `Button` branch above, for the same reason: a
			## `Label` sized by its own text and not expanding has no other
			## width to trim *from*. Skipped when it already wraps
			## (`autowrap_mode != AUTOWRAP_OFF`) -- a wrapping label (this
			## file's own disabled-reason second line, `phone_menu.gd`'s row
			## subtitles) is deliberately multi-line, and trimming it to one
			## line would be a regression, not the fix this is.
			if not wide and ctl is Label and (ctl.size_flags_horizontal & Control.SIZE_EXPAND) != 0 \
					and (ctl as Label).autowrap_mode == TextServer.AUTOWRAP_OFF:
				(ctl as Label).clip_text = true
				(ctl as Label).text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
			if ctl is OptionButton:
				## `clip_text` alone does not shrink an `OptionButton`:
				## `fit_to_longest_item` is on by default, so it reports the
				## width of the **longest item in the list**, not of the
				## selection, specifically so the control does not resize when
				## you pick a different one. On a 393 dp screen that is the
				## expensive guarantee -- Ag. technology's "Traditional Agrarian
				## (ard plow, …)" alone asked for 287 px of a 393 dp row. A
				## phone row is full width and never sits beside anything, so
				## there is no reflow to protect against.
				if not wide:
					(ctl as OptionButton).fit_to_longest_item = false
				var pop := (ctl as OptionButton).get_popup()
				pop.add_theme_constant_override("v_separation", int(round(22.0 * unit)))
				if unit != 1.0:
					pop.add_theme_font_size_override("font_size",
						maxi(1, int(round(pop.get_theme_font_size("font_size") * unit))))
				## HD-01's other half. An embedded sub-window is drawn inside its
				## parent's canvas, so this list inherits the parent window's
				## content scale -- which is the whole reason the `unit != 1.0`
				## branch above exists -- but not the parent's font raster, which
				## is per-`Viewport`. So in the content-scaled case (`unit` 1.0,
				## magnify 3.664) the list's own rows smear exactly as the window
				## behind them did. In a dock the magnify is 1 and the branch
				## above already rasterised the font at its real size, so nothing
				## is set and nothing changes.
				DccWidgets.oversample(pop, _phone_magnify(unit))
		phone_fit(child, unit, wide)

# -- §13 Tablet interior ------------------------------------------------------
#
# `UNWIRED_FUNCTIONS.md`'s "the tablet interior walk -- nothing reads `ROLE`",
# `GUI_GAP_REGISTER.md` §57. `phone_fit()` above multiplies every authored
# figure by a `unit`, because there is one (`_phone_scale`). Tablet has none:
# §57 measured the artboard's own interior ratios at x1.00-x2.06 with no
# centre, so `DccTheme.ROLE` is a table of drawn `[desktop, tablet]` pairs, not
# a multiplier, and `tablet_fit()` is kept as a SECOND function rather than
# folded into `phone_fit()` behind a shared dispatcher for exactly the reason
# §57's own refutation #1 gives: 22 of ~25 `phone_fit()` call sites pass
# `unit = 1.0` for the reason at that function's own header (a `Window` that
# already applied `content_scale_factor` once) -- a dispatcher that dropped
# `unit` would double the phone's scale.
#
# **Most of the fix is not this function.** `DccWidgets`' own factories
# (`_row()`, `slider()`, `action()`, `segment()`, `category()`,
# `stage_category()`, `group()`, `tool_button()`, `toggle()`, `choice()`,
# `number()`) and `right_dock.gd`'s/`layers_popover.gd`'s own row builders now
# resolve their `ROLE` figure at the point of construction -- the answer §57's
# refutation #2 itself gives: a walk dispatched by Godot class cannot tell a
# tier-A action from a tier-B mode chip when both are a plain `Button` in the
# same subtree (`DccWidgets.segment()` -> `chip()`), but the factory that built
# one always knows which it made. `tablet_fit()` below is the fallback for
# whatever a raw `Button.new()`/`Label.new()` elsewhere in the tree -- a
# workspace panel this pass does not own -- built without going through one.
# It floors, never shrinks, and is a no-op everywhere but a tablet.

## Idempotent, matching `phone_fit()`'s own `_PHONE_FIT_META` pattern -- safe
## to call more than once over the same subtree.
const _TABLET_FIT_META := "_tablet_fitted"

## Walks `node`'s descendants and floors whatever a `DccWidgets`/`right_dock.gd`
## /`layers_popover.gd` factory did not already size: any `BaseButton`,
## `LineEdit` or `TextEdit` under `role_px("btn_min_h")` (tier A -- the safe
## default for an ad hoc control, since a tier-B mode chip only exists behind a
## factory this pass already fixed at its source), and any visible `Label`
## under its own `ROLE` figure.
##
## `HSlider` is deliberately excluded: it is a `Range`, not a discrete tap
## target, and `DccWidgets.slider()`/`_style_slider()` already resolve its
## `slider_track_w`/`slider_track_h` at construction. Flooring its control
## height here too would grow it well past the 2-3 px line the design draws
## and, worse, would grow whatever fixed-height bar contains it -- exactly the
## kind of self-inflicted overflow this pass was told to report, not cause.
##
## **Guarded on `DccTheme.is_tablet()`, not `is_touch()`.** `is_touch()` is
## true on a phone too (`_phone` requires `_touch`); `is_tablet()` exists
## precisely so a tablet-only pass cannot silently re-size the phone --
## `GUI_GAP_REGISTER.md` §57 refuted an earlier proposal on exactly that
## ground, and this file's own `_phone`/`is_phone()` note records the mirror
## case ("the 412 canvas asks for things a tablet must not get").
func tablet_fit(node: Node) -> void:
	if not DccTheme.is_tablet():
		return
	_tablet_fit_walk(node)

func _tablet_fit_walk(node: Node) -> void:
	for child in node.get_children():
		if child is Control and not child.has_meta(_TABLET_FIT_META):
			var ctl := child as Control
			ctl.set_meta(_TABLET_FIT_META, true)
			if ctl is BaseButton or ctl is LineEdit or ctl is TextEdit:
				var floor_h := float(DccTheme.role_px("btn_min_h"))
				if ctl.custom_minimum_size.y < floor_h:
					ctl.custom_minimum_size.y = floor_h
			elif ctl is Label:
				## `DccTheme.header()` stamps `ROLE_META` with the exact role it
				## built the label for (`fs_dock_header`, a smaller pair than a
				## bare Plex guess would give it). Everything else falls back to
				## the same mono/prose test `DccTheme.mono_label()` vs `label()`
				## already makes real: a `font` theme override means Plex
				## (`fs_readout`), its absence means the project theme's prose
				## default (`fs_prose`).
				var role: String = ctl.get_meta(DccTheme.ROLE_META) \
					if ctl.has_meta(DccTheme.ROLE_META) \
					else ("fs_readout" if ctl.has_theme_font_override("font") else "fs_prose")
				var floor_fs := DccTheme.role_px(role)
				if ctl.get_theme_font_size("font_size") < floor_fs:
					ctl.add_theme_font_size_override("font_size", floor_fs)
		_tablet_fit_walk(child)

## What one unit of `phone_fit()`'s own space is worth in physical pixels.
##
## The two spaces `phone_fit()` serves reach the same place by different routes:
## a dock lays out in real pixels and `unit` is `_phone_scale`, a content-scaled
## window lays out in dp and `unit` is 1.0 while the compositor supplies the
## rest. Their product is `_phone_scale` in both, which is why one expression
## covers both and why a caller never has to know which one it is in. Exactly
## 1.0 in a dock (`_phone_scale / _phone_scale` is exact in IEEE-754), so the
## dock path is byte-identical to what it did before this existed.
##
## Gated on `_phone` and not only on the callers being phone-only, because
## `_compute_layout_mode()` computes `_phone_scale` for **every** composition --
## a 1920 x 1080 desktop reads 2.75 -- and a `phone_fit()` call that ever
## reached a pointer build would otherwise quietly rasterise every glyph at
## nearly three times its size and minify it back down, which is worse than the
## fault this closes rather than merely wasteful.
func _phone_magnify(unit: float) -> float:
	if not _phone:
		return 1.0
	return _phone_scale / maxf(0.0001, unit)

## The phone half of a §4.5 tool button: a real glyph, a visible box, and the
## name the tooltip can no longer deliver. See the call site in `phone_fit()`
## for the fault this closes.
func _phone_fit_tool_button(b: Button, unit: float) -> void:
	## Re-rasterised from the SVG at the size it will actually be drawn at,
	## which is why `dcc_widgets.gd` stashes the glyph's *name*: `DccIcons`
	## caches per (name, drawn, raster), so the 15 px texture already in hand
	## cannot be grown without resampling it. 0.42 of the box leaves the caption
	## room and keeps the icon off the border.
	##
	## `_phone_magnify(unit)` is HD-02's half: `box` is in this subtree's own
	## space, so in a dock it is already physical and the magnify is exactly 1
	## (this call is unchanged there), while in a content-scaled window it is dp
	## and the raster has to be 3.664x finer than the number beside it.
	var box := maxf(b.custom_minimum_size.x, b.custom_minimum_size.y)
	b.icon = DccIcons.get_icon(String(b.get_meta(DccWidgets.TOOL_GLYPH_META)),
		maxi(1, int(round(box * 0.42))), _phone_magnify(unit))
	## The desktop button is invisible at rest on purpose -- a palette of eight
	## empty squares reads as one strip, and hover picks one out. Touch has no
	## hover, so at rest is the only state there is, and an unbounded mark is
	## not identifiable as a target. `pressed` keeps its accent wash, so armed
	## still reads differently from merely present.
	b.add_theme_stylebox_override("normal",
		DccTheme.outline("line_soft", "panel", maxi(1, int(round(unit)))))
	## `outline()` and `flat()` both carry a zero content margin, so without
	## this the caption is drawn hard against the border it just gained -- and
	## the *three* states have to agree, or `Button` re-lays its content out on
	## press and the label jumps sideways under the finger holding it.
	var pad: float = round(4.0 * unit)
	for state in ["normal", "hover", "pressed"]:
		var sb: StyleBox = b.get_theme_stylebox(state)
		sb.content_margin_left = pad
		sb.content_margin_right = pad
	if not b.has_meta(DccWidgets.TOOL_CAPTION_META):
		return
	b.text = String(b.get_meta(DccWidgets.TOOL_CAPTION_META))
	## Icon *above* the caption, not beside it: `Button` stacks the two
	## whenever `vertical_icon_alignment` is anything but CENTER, and stacking
	## is what keeps a tool close to square instead of turning the row into
	## four wide pills. The width this asks for is why `tools_block()` lays its
	## rows out in an `HFlowContainer`.
	b.vertical_icon_alignment = VERTICAL_ALIGNMENT_TOP
	b.alignment = HORIZONTAL_ALIGNMENT_CENTER
	b.add_theme_font_size_override("font_size",
		maxi(1, int(round(DccTheme.FS_MICRO * unit))))
	b.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	b.add_theme_color_override("font_pressed_color", DccTheme.c("accent"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))

## The dock sheets carry every workspace panel -- the NPR Painter block, the
## CIVIL dock's Faction roster button, the right dock's Settlement ▸ City
## layout -- and every one of them is rebuilt from a signal at some point after
## boot, so a one-shot pass over the dock at build time would be correct for
## about a second. `node_added` is the only hook that sees all of them without
## this file knowing which panels exist; the work is coalesced onto one
## deferred pass per frame, so a rebuild that adds 2000 nodes still fits once.
##
## Cheap because the fit itself is meta-flagged: the pass walks the sheet, but
## only *touches* the nodes it has not already sized.
var _phone_fit_pending := false

func _on_phone_node_added(node: Node) -> void:
	if _phone_fit_pending or not (node is Control):
		return
	## Only a dock descendant is our business. Walked rather than connected
	## per-panel because `child_entered_tree` fires for direct children only,
	## and every panel is several levels down.
	var p: Node = node.get_parent()
	while p != null:
		if p == left_dock or p == right_dock:
			_phone_fit_pending = true
			_run_phone_dock_fit.call_deferred()
			return
		p = p.get_parent()

func _run_phone_dock_fit() -> void:
	_phone_fit_pending = false
	for dock in [left_dock, right_dock]:
		if dock != null:
			phone_fit(dock, _phone_scale)

## Read by dialogs that have to present themselves differently on a phone
## (`open_project_dialog.gd`); `_phone`/`_phone_scale` stay private because
## nothing outside should be *setting* them.
func is_phone() -> bool:
	return _phone

func phone_scale() -> float:
	return _phone_scale

## The node `Window ▸ Domain rail` shows and hides, for `DccApp`'s region map.
## Falls back to `rail_column` rather than returning null, so a composition that
## somehow never set it hides *something* real instead of crashing the menu.
func rail_region() -> Control:
	return _rail_region if _rail_region != null else rail_column

## `Window ▸ Status bar` on the phone. **Not** a node in `DccApp`'s region map,
## which is why this is a setter pair rather than a `status_region()` twin of
## `rail_region()` above.
##
## Two reasons it cannot be a node. The first is the defect
## `UNWIRED_FUNCTIONS.md` registered: on a phone the desktop status bar is built
## into the hidden `PhoneMenuModel` host (`_build_phone_shell()`), where it is
## the *data model* `phone_menu.gd` reads and not a drawn surface at all, so the
## menu row moved a check mark over a node that was already permanently
## invisible. The second is that the surface it should act on is **two** nodes,
## not one: `_phone_top_safe` in portrait and `_phone_side_safe` in landscape,
## swapped by `_apply_phone_orientation()` on every rotation. A single captured
## node would be wrong in one orientation and would be overwritten by the next
## rotation in the other -- the same write-the-node-behind-the-flag's-back fault
## that made `Window ▸ Left dock` desync the phone sheets.
##
## **What it hides.** On the phone composition the shell's status *readouts*
## (pass, hint, stale, autosave) are not drawn as a strip at all -- `top_world`
## is the app bar's subtitle and the rest are rows inside MORE -- so there is no
## app status strip for this row to act on. What is drawn, and is what "status
## bar" names on this platform, is the 28 dp clock/battery row the 412 canvas
## puts across the top (`_build_phone_top_safe()`) and its landscape column.
## Unchecking the row hides that and gives the map the band back; nothing the
## user can read anywhere else disappears with it.
var _phone_status_shown := true

func is_status_region_shown() -> bool:
	return _phone_status_shown

func set_status_region_shown(shown: bool) -> void:
	_phone_status_shown = shown
	## Routed through the orientation pass rather than written here, so the
	## portrait/landscape half of each node's visibility stays stated in exactly
	## one place.
	_apply_phone_orientation()

# -- §3 Domain rail -----------------------------------------------------------

## **The rail has two states again, and this time the canvas draws both.**
##
## Between 2026-08-19 and 2026-08-24 it had a collapsed/expanded pair: SH-01
## turned the mockup's head chevron into a `Button` that grew the rail to
## `W_RAIL_EXPANDED` (200 px) and swapped the domain column for a
## `_phone_list_row()` list of each domain's sub-structure. That was deleted on
## 2026-08-24 with the reasoning "the canvas never draws it" -- every one of the
## eight desktop artboards across `design/Cartalith DCC Shell.dc.html` and
## `design/Cartalith Measurement Toolbar.dc.html` opened the rail at the same
## literal `width:40px;flex:none`, and the state that had been built instead
## borrowed the *phone* drawer's type scale into a 200 px column: screenshotted
## live, "CARTOGRAPHY" ran straight under the left dock. The owner reported it
## as "the left rail is collapsible and shouldn't be".
##
## **That reasoning was true of the old canvas and is false of this one.** The
## 2026-08-31 ENV prototype draws the expansion as a *separate sibling column*
## (`ENV:293`-`303`), not as the strip growing: the 40 px strip keeps its width
## in both states and a `var(--railExpW)` column appears to its right, pushing
## the left dock over. That is a different composition from the one the owner
## rejected, and `CLAUDE.md`'s "when two design canvases disagree, the newer one
## wins" settles which applies. The type scale that broke last time cannot break
## this time either: node rows are sans at `--fs` (11.5 px) and headers mono at
## `--m2` (9 px) -- `02-rail-and-domains.md` §5d -- neither of which is the
## phone drawer's scale.
##
## Composition, top to bottom (`02-rail-and-domains.md` §5b):
##
##   chevron cell   `height:var(--tool)`, `--faint`, one `▸` rotated 0°/180°
##   domain ×3      `flex:1; max-height:112px`, vertical mono label
##   spacer         `flex:1`
##   footer         vertical mono `--m2`, `--faint`
##
## What this function returns is therefore the **pair**, not the strip:
## `ENV:282` wraps both in one `<sc-if value="{{ showRail }}">`, so
## `Window ▸ Domain rail` must take both or leave the expansion column stranded
## with no chevron to close it. `_rail_region` is the pair for the same reason.
func _build_rail() -> Control:
	## The pair. An `HBoxContainer` rather than the rail panel itself, so the
	## expansion column is a sibling at the same depth the prototype puts it --
	## see this function's header for why it is not the strip widening.
	var pair := HBoxContainer.new()
	pair.add_theme_constant_override("separation", 0)
	_rail_region = pair

	var rail := PanelContainer.new()
	rail.custom_minimum_size.x = _scaled(DccTheme.W_RAIL_COLLAPSED)
	rail.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"right": 1}))
	rail_column = VBoxContainer.new()
	rail_column.add_theme_constant_override("separation", 14)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_top", 12)
	pad.add_child(rail_column)

	## The head cell. **A `Button` again, and this is a reversal that has to be
	## argued rather than just done.** It was demoted to a `Label` on 2026-08-24
	## under the rule "chrome the mockup specifies, not an affordance nothing
	## behind it can honour" -- correct at the time, because the expansion it
	## toggled had just been deleted. There is something behind it now:
	## `ENV:1929` binds `hRailExp:()=>this.setState(x=>({railExp:!x.railExp}))`
	## to exactly this cell, and BUILD_ANSWERS §2.5 rules on what it opens. The
	## rule did not change; the fact it was applied to did.
	##
	## `_scaled(30)`, not the old `_scaled(29)`: `ENV:284` draws the cell at
	## `height:var(--tool)`, which is 30 px pointer and 44 px touch.
	## `DccTheme.TABLET`'s stale `29: 34` row was flagged by stage 1 as
	## "left alone because changing it moves the rail head's box, and the rail
	## is stage 2's rebuild" -- this is that rebuild, so the row moved to
	## `30: 44` in the same pass.
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	var chev_btn := Button.new()
	chev_btn.flat = false
	chev_btn.focus_mode = Control.FOCUS_NONE
	chev_btn.custom_minimum_size.y = _scaled(30)
	chev_btn.tooltip_text = "Show or hide the node list"
	chev_btn.accessibility_name = "Show or hide the node list"
	chev_btn.add_theme_stylebox_override("normal", DccTheme.empty())
	chev_btn.add_theme_stylebox_override("focus", DccTheme.empty())
	chev_btn.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("line_soft")))
	chev_btn.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	chev_btn.pressed.connect(_toggle_rail_expansion)
	## **One glyph rotated, not two glyphs swapped** -- BUILD_ANSWERS §2.5
	## retires the `›`/`‹` pair the old head cell used. So this is
	## `SYMBOLS["submenu"]` (`▸`, the same U+25B8 the dock accordions draw) with
	## `rotation` flipped between `0` and `PI`, and `_paint_rail_chevron()` owns
	## the flip. A `Control` rotates about `pivot_offset`, which defaults to the
	## top-left, so a 180° turn about it would throw the glyph clean out of its
	## own cell -- the pivot is therefore re-centred on every `resized`, which is
	## the only moment the label's size is known (the same in-tree measurement
	## problem `_layout_rail_label()`'s header documents, in its smaller form).
	var chev := DccTheme.mono_label(DccIcons.SYMBOLS["submenu"], "text_dim", DccTheme.FS_SMALL)
	chev.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	chev.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	chev.set_anchors_preset(Control.PRESET_FULL_RECT)
	chev.mouse_filter = Control.MOUSE_FILTER_IGNORE
	chev.resized.connect(_paint_rail_chevron)
	chev_btn.add_child(chev)
	_rail_chevron = chev
	col.add_child(chev_btn)
	col.add_child(DccTheme.rule())

	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 0)
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body.add_child(pad)
	col.add_child(body)
	rail.add_child(col)

	var w := float(_scaled(DccTheme.W_RAIL_COLLAPSED))
	for i in DOMAINS.size():
		var d: Dictionary = DOMAINS[i]
		if i > 0:
			## A 14 px hairline between each pair, exactly as the mockup draws
			## it -- the rail's only ornament.
			var sep := ColorRect.new()
			sep.color = DccTheme.c("line")
			sep.custom_minimum_size = Vector2(14, 1)
			sep.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
			rail_column.add_child(sep)

		var b := Button.new()
		b.tooltip_text = "%s -- %s" % [d.label, d.subtitle]
		## Not flat: a flat `Button` skips its styleboxes, so the `hover` box on
		## the next line had never drawn and the rail gave no pointer feedback
		## at all. `normal` is empty, so nothing else changes. (Same trap as the
		## menu bar's open title, found in the same 2026-08-25 menu sweep.)
		b.flat = false
		b.focus_mode = Control.FOCUS_NONE
		b.add_theme_stylebox_override("normal", DccTheme.empty())
		b.add_theme_stylebox_override("focus", DccTheme.empty())
		b.add_theme_stylebox_override("disabled", DccTheme.empty())
		b.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("line_soft")))
		b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
		## **Not `_select_domain` any more.** BUILD_ANSWERS §2.5: clicking the
		## already-active domain toggles the expansion column, because with the
		## dock closed it is the only affordance in reach. `_on_domain_pressed()`
		## is that branch; a click on an inactive domain still lands on
		## `_select_domain()` unchanged.
		b.pressed.connect(_on_domain_pressed.bind(d.id))

		## The reference rail is text only -- verified twice: once by reading
		## `design/Cartalith DCC Shell.dc.html`'s own markup (`writing-mode:
		## vertical-rl` labels, no icon element anywhere in the rail), and once
		## by the owner directly, after an earlier revision added icons anyway:
		## "those icons don't exist." Removed rather than hidden behind a flag --
		## an addition the design does not specify does not get to linger. The
		## 2026-08-31 prototype agrees: `ENV:287`'s cell holds one
		## `writing-mode:vertical-rl` span and nothing else.
		## `font:10px 'IBM Plex Mono'; letter-spacing:.12em; color:#5f6468`,
		## regular weight -- the rail block's own inline style, verbatim. This
		## was 9 px at `spacing 2` (≈.22em) in Medium, which is a size down, a
		## tracking up and a weight up all at once: three small errors
		## compounding on the one piece of chrome that is always on screen.
		## `spacing` is whole pixels, so .12em at 10 px is 1.
		var vlabel := DccTheme.mono_label(String(d.rail).to_upper(),
			"text_ghost", DccTheme.FS_TINY, 1, false)
		vlabel.rotation = -PI / 2.0
		b.add_child(vlabel)
		## Provisional geometry only. The real numbers come from
		## `_layout_rail_labels()` below, once this rail is in the tree --
		## measuring here cannot work, and that is not a style preference:
		## nothing built by this function is in the `SceneTree` yet (the rail
		## is *returned* and added by the caller), so a `Label` asked for its
		## minimum size here has no theme and therefore no font to measure.
		## Measured before the fix: the three five-character labels WORLD /
		## CIVIL / CARTO produced button heights of **67, 53 and 62 px** from
		## this early read, against the identical `(34, 14)` all three report
		## once in-tree. Three different wrong answers to the same question.
		_layout_rail_label(b, vlabel, w)

		_domain_buttons[d.id] = b
		_domain_marks[d.id] = {"label": vlabel}
		rail_column.add_child(b)

	## Re-measure every rail label once the rail is actually in the tree. See
	## `_layout_rail_label()` for why this cannot be done above.
	call_deferred("_relayout_rail_labels")

	body.add_child(DccTheme.spacer())
	rail_foot = DccTheme.mono_label("", "text_ghost", DccTheme.FS_MICRO, 2)
	rail_foot.rotation = -PI / 2.0
	var foot_holder := Control.new()
	foot_holder.custom_minimum_size.y = 84
	foot_holder.add_child(rail_foot)
	body.add_child(foot_holder)

	pair.add_child(rail)
	pair.add_child(_build_rail_expansion())
	return pair

## The `--railExpW` node column (`ENV:293`-`303`), collapsed at rest.
##
## Thirteen rows in one flat list -- three headers interleaved with ten nodes --
## because that is literally what `ENV:1824` builds and `ENV:295` iterates. The
## header/node distinction is drawn, not structural: a header is mono `--m2` at
## `.2em` in `--faint` and is inert; a node is sans `--fs`, clickable, hovers to
## `--wash`, and states its selection **in ink alone** (`ENV:1826`'s `bg` is the
## literal string `'transparent'` for every node, always). There is no selected
## row fill, no indicator bar and no bold weight, which is why the only thing
## `_paint_rail_nodes()` writes is `font_color`.
##
## The column is `visible = false` at rest rather than absent, where the
## prototype uses a real `<sc-if>` that keeps it out of the DOM. A hidden
## `Control` in Godot contributes no minimum size to its `HBoxContainer` parent,
## so the laid-out result is identical -- and building it once means the node
## rows exist for `_paint_rail_nodes()` and for `_railfold_probe.gd` to assert
## on before the user has ever opened it.
func _build_rail_expansion() -> Control:
	var panel := PanelContainer.new()
	panel.visible = false
	panel.custom_minimum_size.x = DccTheme.role_px("w_rail_expanded")
	panel.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"right": 1}))
	_rail_exp_column = panel

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	panel.add_child(scroll)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(col)

	for entry in RAIL_NODES:
		var n: Dictionary = entry
		if String(n["kind"]) == "head":
			## `padding:10px 14px 3px`, mono `--m2`, `.2em`, `--faint`, and
			## **`--faint` unconditionally**: `ENV:1826` computes a `col` for
			## header rows too and `ENV:296` then hard-codes the colour and
			## throws it away, so a header never changes with selection. Copied
			## rather than corrected -- `02-rail-and-domains.md` §8 item 10 flags
			## the discard as unresolvable from the file, and inventing the
			## other reading would be inventing a design value.
			var h := DccTheme.mono_label(String(n["label"]), "text_faint",
				DccTheme.FS_MICRO, 2)
			var hp := MarginContainer.new()
			hp.add_theme_constant_override("margin_left", 14)
			hp.add_theme_constant_override("margin_right", 14)
			hp.add_theme_constant_override("margin_top", 10)
			hp.add_theme_constant_override("margin_bottom", 3)
			hp.add_child(h)
			col.add_child(hp)
			continue

		var b := Button.new()
		b.text = String(n["label"])
		b.flat = false
		b.focus_mode = Control.FOCUS_NONE
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.clip_text = true
		b.custom_minimum_size.y = _scaled(28)
		b.tooltip_text = "%s -- %s" % [String(n["domain"]).capitalize(), String(n["label"])]
		## Sans, not mono: `ENV:297` declares no `font` on the node row, so it
		## inherits the shell body face -- the one element in the whole rail that
		## is NOT IBM Plex (`02-rail-and-domains.md` §5d). Left at the theme's
		## default font for exactly that reason: no `DccTheme.mono()` here.
		b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
		b.add_theme_stylebox_override("normal", DccTheme.inset(14, 2, 14, 2))
		b.add_theme_stylebox_override("focus", DccTheme.empty())
		b.add_theme_stylebox_override("pressed", DccTheme.inset(14, 2, 14, 2))
		b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("accent_wash")))
		b.pressed.connect(_on_rail_node_pressed.bind(String(n["domain"]), String(n["mode"])))
		col.add_child(b)
		_rail_node_rows["%s/%s" % [String(n["domain"]), String(n["mode"])]] = b

	_paint_rail_nodes()
	return panel

## `hRailExp` (`ENV:1929`). Nothing but the flag and the two things that read it.
func _toggle_rail_expansion() -> void:
	set_rail_expanded(not _rail_expanded)

## Public because `Window ▸ Reset layout` writes `railExp:false` (`ENV:2052`)
## and because `_railfold_probe.gd` drives the toggle without a synthetic click.
func set_rail_expanded(on: bool) -> void:
	_rail_expanded = on
	if _rail_exp_column != null and is_instance_valid(_rail_exp_column):
		_rail_exp_column.visible = on
	_paint_rail_chevron()

func is_rail_expanded() -> bool:
	return _rail_expanded

func _paint_rail_chevron() -> void:
	if _rail_chevron == null or not is_instance_valid(_rail_chevron):
		return
	_rail_chevron.pivot_offset = _rail_chevron.size * 0.5
	_rail_chevron.rotation = PI if _rail_expanded else 0.0

## `hDomain` (`ENV:1930`-`1931`), verbatim in behaviour:
##
##     if(id===s.domain) this.setState(x=>({railExp:!x.railExp}));
##     else this.setDomain(id)
##
## BUILD_ANSWERS §2.5 gives the reason the two branches differ -- the expansion
## toggle "is the only affordance in reach when the panel is closed" -- and that
## is also why re-clicking the active domain must not be the no-op it was before
## this stage.
func _on_domain_pressed(id: String) -> void:
	if id == _active_domain:
		_toggle_rail_expansion()
		return
	_select_domain(id)

## `hRailNode` (`ENV:1934`): sets the domain, sets *that domain's* mode from the
## row's own `data-mode`, and closes the expansion -- all three, in that order.
## The close is what makes the column a transient drill-down rather than a second
## permanent navigation surface, and it matches `setDomain`'s own
## `railExp:false` (`ENV:2054`).
##
## ## CIVIL ▸ `planner` also opens the Journey planner (2026-09-05)
##
## Owner ruling, `LARGE_ITEM_RULINGS.md` 2026-09-05 item 4: the Journey planner
## *"becomes a CIVIL rail node rather than a Data menu row"*. `RAIL_NODES` has
## carried the node since stage 2, but it only opened CIVIL's `Travel` accordion
## category -- the takeover itself was reached from `Data ▸ Journey planner… ⇧J`
## and three in-dock buttons, and that menu row is gone as of the same pass
## (`menus.gd::_data()`). This branch is the row's replacement.
##
## **Hung on the click, not on `select_domain_mode()`.** That function is also
## reached by `select_domain_category()`, and `mode_for_category("civilization",
## "Travel")` resolves to `planner` -- so putting the takeover there would make
## every jump that merely wants the Travel accordion (a cross-reference button,
## a probe, `_railfold_probe.gd` §2's own driver) swap the viewport as a side
## effect. A node *press* is the deliberate act; a mode write is not.
##
## ## And the three sibling CIVIL nodes release it
##
## `journey_planner_view.gd::_recompute_visibility()` shows the takeover while
## `armed_tool == "journey"` **and** the domain is CIVIL. Switching domain
## therefore hides it on its own; switching *node inside CIVIL* did not, and
## `_hide()` is what restores `_workspace_panels["civilization"]`. Clicking
## `Landmarks` with the planner up would otherwise have re-shown the civ dock
## underneath a still-visible planner panel -- two left docks at once. That was
## reachable before this pass and is the main path after it, which is why it is
## fixed here rather than left as a corner.
##
## Disarmed *before* the mode write, so `_hide()` runs while the domain is still
## CIVIL and hands `timeline_row` back (JP-13) before the new node's category
## opens. `has_method("arm_tool")` because `DccShell` is instantiated bare by the
## capture probes -- the same guard `_pick_phone_tab()` uses for
## `open_journey_planner()`, and for the same reason.
func _on_rail_node_pressed(domain: String, mode: String) -> void:
	if domain == "civilization" and mode != "planner" and has_method("arm_tool") \
			and String(get("armed_tool")) == "journey":
		call("arm_tool", "inspect")
	select_domain_mode(domain, mode)
	if domain == "civilization" and mode == "planner" and has_method("open_journey_planner"):
		## Re-selects the same mode on the way through, which is why the call
		## above is not skipped: `select_domain_mode()` is idempotent (the mode
		## is already written, `_select_domain()` repaints from it, and
		## `Workspace.open_category()` emits only when the body is hidden), and
		## routing the rail through the one opener every other entry point uses
		## is worth more than saving the repaint.
		call("open_journey_planner")
	set_rail_expanded(false)

## Node ink (`ENV:1826`). Accent when the node's domain is the active one **and**
## its mode is that domain's current mode; `text` otherwise.
##
## The prototype's predicate is
## `n.dom===s.domain && (!n.mode || n.mode===modeOf(n.dom))`, whose `!n.mode`
## half existed only because the four CARTO nodes carried an empty mode in the
## truncated build and therefore all lit at once (`02-rail-and-domains.md` §3a).
## BUILD_ANSWERS §2.1 gave them real modes -- `ENV:1824` now reads
## `nd('Layers & style','CARTO','style')` and so on -- so the `!n.mode` branch is
## dead in the complete file and is not reproduced here: every node in
## `RAIL_NODES` has a mode, and exactly one node per domain is accent.
## `_railfold_probe.gd` §5 asserts that for CARTO specifically, since CARTO is
## where the defect was.
func _paint_rail_nodes() -> void:
	for key in _rail_node_rows:
		var parts := String(key).split("/")
		var on: bool = parts[0] == _active_domain \
			and String(_domain_mode.get(parts[0], "")) == parts[1]
		var b: Button = _rail_node_rows[key]
		if is_instance_valid(b):
			b.add_theme_color_override("font_color",
				DccTheme.c("accent") if on else DccTheme.c("text"))

## Centre one rotated rail label inside its own button, and size the button to
## it.
##
## **The rotation is why this needs stating.** A `Control` rotates about its
## `pivot_offset`, which defaults to its top-left, and `rotation = -PI/2` maps
## local `(x, y)` to parent `(y, -x)`. So a label of length `L` positioned at
## `y` occupies the parent's vertical span **`[y - L, y]`** -- it grows
## *upward* from its own position, not downward. The pre-2026-08-30 code set
## that position to a flat `12.0`, which put the text `L - 12` px **above the
## top of its own button**: measured live, all three labels overflowed by 22 px
## and left 41-55 px of empty band beneath them, which is the misalignment the
## owner reported.
##
## Centred means the span `[y - L, y]` is centred in a button of height
## `L + 2·PAD`, which solves to `y = L + PAD`.
const RAIL_LABEL_PAD := 12.0

func _layout_rail_label(b: Button, vlabel: Control, rail_w: float) -> void:
	var text_size := vlabel.get_minimum_size()
	## `text_size.y` is the glyph height, which *after* the -90° rotation is
	## the label's horizontal extent -- hence `.y` against the rail's width.
	vlabel.position = Vector2(
		round(rail_w * 0.5 - text_size.y * 0.5),
		text_size.x + RAIL_LABEL_PAD)
	b.custom_minimum_size.y = text_size.x + RAIL_LABEL_PAD * 2.0

## Deferred from `_build_rail()`, because nothing that function builds is in the
## `SceneTree` while it runs, and an orphaned `Label` has no theme to measure a
## font with. Idempotent, so it is safe to call again after a theme or scale
## change.
func _relayout_rail_labels() -> void:
	if _rail_region == null or not is_instance_valid(_rail_region):
		return
	var w := float(_scaled(DccTheme.W_RAIL_COLLAPSED))
	for id in _domain_marks.keys():
		var b: Button = _domain_buttons.get(id)
		var lbl: Control = _domain_marks[id].get("label")
		if b != null and lbl != null and is_instance_valid(b) and is_instance_valid(lbl):
			_layout_rail_label(b, lbl, w)

## The rail foot carries the active context and, in World, the stage counter.
## Re-centred on every set because its width changes with the text.
func set_rail_foot(text: String) -> void:
	if rail_foot == null:
		return
	rail_foot.text = text
	var w := float(_scaled(DccTheme.W_RAIL_COLLAPSED))
	var m := rail_foot.get_minimum_size()
	rail_foot.position = Vector2(round(w * 0.5 - m.y * 0.5), 12.0 + m.x)

func _select_domain(id: String) -> void:
	_active_domain = id
	for key in _domain_buttons:
		var b: Button = _domain_buttons[key]
		var on: bool = key == id
		var marks_pre: Dictionary = _domain_marks.get(key, {})
		## The desktop rail's active cell is `background:rgba(224,163,74,.08)`
		## in its own artboard. The 412 phone canvas's active *tab* has no fill
		## at all -- `<div style="...;color:#e0a34a">` and nothing else -- so the
		## bar cell registers `"box": false` and states itself in ink, glyph and
		## caption together, the way a bottom-nav tab does everywhere.
		if bool(marks_pre.get("box", true)):
			b.add_theme_stylebox_override("normal",
				DccTheme.active_row(false) if on else DccTheme.empty())
		var marks: Dictionary = _domain_marks.get(key, {})
		## `text_ghost` (`#5f6468`) is the *desktop rail's* resting ink -- see
		## `_build_rail()`. The 412 phone canvas rests its bottom-nav tabs one
		## step brighter, at `#8d9296` (`text_dim`), so the cell that registers
		## the mark says which. Restoring to `text_faint` here used to mean the
		## rail brightened by one step the first time a domain was ever selected
		## and never went back.
		var off: String = String(marks.get("off", "text_ghost"))
		if marks.has("label"):
			(marks["label"] as Label).add_theme_color_override("font_color",
				DccTheme.c("accent") if on else DccTheme.c(off))
		## The phone bar's glyph. A `DccIcons` texture is drawn in white and
		## tinted by `modulate`, so this is the same one-asset/two-states
		## contract `dcc_icons.gd`'s header describes -- not a second texture.
		if marks.has("icon"):
			(marks["icon"] as CanvasItem).modulate = \
				DccTheme.c("accent") if on else DccTheme.c(off)
	for key in _workspace_panels:
		(_workspace_panels[key] as Control).visible = key == id
	## **The §3 gate, applied at the one choke point every route reaches.**
	## `select_domain_mode()`, `select_domain_category()`, `select_domain()`,
	## `Window ▸ Workspace`, the phone tabs and every in-shell "→ Cartography ▸ …"
	## jump all land here, and all of them can arrive with a *different* mode
	## than the panel was last painted for -- `_domain_mode` persists per domain,
	## so returning to WORLD after leaving it in Sculpt has to restore Sculpt's
	## body, not the pipeline's. Applied to every registered panel and not only
	## the active one, so a panel that was gated while hidden is correct the
	## instant it is shown rather than one frame later.
	##
	## Ordered after the visibility loop above for a reason: `apply_mode()` ends
	## by making sure one *visible* category is open, and "visible" is read off
	## the wraps it has just set, not off the panel -- so it is safe either way,
	## but a reader looking for the gate should find it beside the thing it gates.
	for key in _workspace_panels:
		var p: Control = _workspace_panels[key]
		if p.has_method("apply_mode"):
			p.call("apply_mode", String(_domain_mode.get(key, "")))
	_refresh_mode_switch()
	for d in DOMAINS:
		if d.id == id:
			left_dock_title.text = String(d.label).to_upper()
			break
	## The expansion column's ink follows the domain, and `setDomain` closes the
	## column (`ENV:2054`: `{domain:d, railExp:false}`) -- both are part of the
	## same state write in the prototype, so both happen here rather than at the
	## two call sites that reach `_select_domain()`.
	_paint_rail_nodes()
	set_rail_expanded(false)
	## The phone's bottom bar names four TASKS, not the three domains, so a
	## domain change that did not come from a tab press used to leave it lit on
	## wherever the user last WAS -- most visibly MORE, which stayed lit after
	## Civilization was picked out of it, since `PHONE_TABS` has no
	## `civilization` entry at all to light instead. Done here, at the one choke
	## point every route reaches (`select_domain()`, `select_domain_mode()`,
	## `select_domain_category()`, `Window ▸ Workspace`, and every in-shell
	## "→ Cartography ▸ ..." jump), rather than at each of those callers.
	if _phone:
		_phone_tab = _phone_tab_for_domain(id)
		_refresh_phone_tabs()
	## §5.2's chip reads the domain and its mode, and `select_domain_mode()`
	## writes the mode before calling here -- so this one call site covers both.
	## No-op until the viewport exists (`DccShell._ready()` runs as
	## `super._ready()`, before `app.gd` builds it); the deferred call at the foot
	## of `_ready()` is what draws it the first time.
	_refresh_viewport_context()
	workspace_changed.emit(id)

## The active mode of one domain -- `worldMode` / `cc()` / `ct()` behind one
## accessor, since this shell has no reason to keep three names for one question.
## Falls back to that domain's first node, which is what `cc()`'s and `ct()`'s
## own `|| 'landmarks'` / `|| 'style'` defaults do.
func active_mode(id: String = "") -> String:
	var d := id if not id.is_empty() else _active_domain
	return String(_domain_mode.get(d, ""))

## Switch domain **and** mode, then open whichever category that mode's node
## names. The rail's own node click (`_on_rail_node_pressed`) is one caller;
## `select_domain_category()` below is the other.
##
## The category open is not optional garnish: this shell's left dock is a single
## accordion per domain (see `RAIL_NODES`' header for why it cannot be
## mode-gated the way the prototype's is), so without it a node click would
## light the rail and change nothing the user can see.
func select_domain_mode(id: String, mode: String) -> void:
	var node := rail_node(id, mode)
	if node.is_empty():
		push_warning("Cartalith: no rail node '%s/%s'." % [id, mode])
		_select_domain(id)
		return
	_domain_mode[id] = mode
	## Order matters. `_select_domain()` repaints the rail from `_domain_mode`,
	## so the mode has to be written first or the click would light the previous
	## node; and it emits `workspace_changed`, which `app.gd::_refresh_rail_foot()`
	## answers by reading `active_mode()` -- also written above.
	_select_domain(id)
	var panel: Control = _workspace_panels.get(id)
	if panel != null and panel.has_method("open_category"):
		if not panel.call("open_category", String(node["category"])):
			push_warning("Cartalith: rail node '%s/%s' names category '%s', which the %s dock does not have."
				% [id, mode, String(node["category"]), id])

## Write one domain's mode and re-run everything that reads it -- the rail ink,
## the dock's mode switch, and the §3 gate on that domain's panel -- **without**
## opening a category.
##
## That omission is the whole reason this is separate from `select_domain_mode()`
## rather than a flag on it. `Workspace.open_category()` calls this when the
## category it was asked for is behind a mode switch; if this opened a category
## too, the two would recurse through each other. `select_domain_mode()` is the
## composition of this and the open, and stays the entry point for anything that
## wants both.
##
## Does not switch domain: a mode is per-domain state that survives leaving it
## (`_domain_mode`'s own header), so writing CIVIL's mode from a CARTO dock is
## meaningful and must not drag the user across the rail.
func apply_domain_mode(id: String, mode: String) -> void:
	if rail_node(id, mode).is_empty():
		push_warning("Cartalith: no rail node '%s/%s'." % [id, mode])
		return
	_domain_mode[id] = mode
	_paint_rail_nodes()
	_refresh_mode_switch()
	var panel: Control = _workspace_panels.get(id)
	if panel != null and panel.has_method("apply_mode"):
		panel.call("apply_mode", mode)

## The `RAIL_NODES` row for one (domain, mode) pair, or `{}`.
static func rail_node(id: String, mode: String) -> Dictionary:
	for n in RAIL_NODES:
		if String(n.get("kind", "")) == "node" \
				and String(n["domain"]) == id and String(n["mode"]) == mode:
			return n
	return {}

## Which mode owns an accordion category -- the reverse of `RAIL_NODES`' `owns`
## lists, and the reason `select_domain_category()` does not need every caller to
## learn a third argument.
##
## **This derivation is the single source of truth, and that is deliberate.**
## The obvious alternative was to make the mode a required parameter and update
## the eight existing `select_domain_category()` call sites to pass one. Every
## such call would then either agree with this table (and be redundant) or
## disagree with it (and be a bug that lights the wrong rail node while opening
## the right category). A caller knows which category it wants; only
## `RAIL_NODES` knows which node owns it. Returning `""` for an unowned category
## is the honest answer -- `select_domain_category()` treats it as "leave the
## mode alone" rather than guessing, and `_railfold_probe.gd` §3 asserts that no
## category in any of the three docks is unowned, so the `""` branch is
## unreachable in a correct build and is there to make an incorrect one visible.
## The categories one mode renders, or `[]` for *"all of them"* -- the read side
## of `RAIL_NODES`' `shows` key, whose header block above is the reasoning.
##
## `[]` and "this mode hides everything" are deliberately not the same value:
## nine of the ten nodes carry no `shows` at all, and a gate that defaulted to
## an empty allow-list would blank nine docks. Absent means ungated, which is
## why this returns the key's own value untouched rather than something derived
## from `owns` -- `owns` answers *which node lights*, and every category is in
## exactly one `owns` list, so deriving a gate from it would take `Terrain`'s
## erosion parameters out of the pipeline in WORLD `a`, and leave CIVIL showing
## only the categories one node happens to own -- one header in `landmarks`, two
## in `infra`, one in `planner`.
static func mode_shows(id: String, mode: String) -> Array:
	var n := rail_node(id, mode)
	return (n.get("shows", []) as Array) if not n.is_empty() else []

## Whether a domain gates its dock at all -- true when any of its nodes carries
## a `shows`. Drives the dock's own mode switch (`_build_mode_switch()`), which
## is shown for exactly the domains where a mode change is capable of removing a
## header from the body, and hidden where it cannot.
static func domain_gates(id: String) -> bool:
	for n in RAIL_NODES:
		if String(n.get("kind", "")) == "node" and String(n["domain"]) == id \
				and not (n.get("shows", []) as Array).is_empty():
			return true
	return false

## Every node of one domain, in `RAIL_NODES` order.
static func domain_nodes(id: String) -> Array:
	var out: Array = []
	for n in RAIL_NODES:
		if String(n.get("kind", "")) == "node" and String(n["domain"]) == id:
			out.append(n)
	return out

static func mode_for_category(id: String, category: String) -> String:
	for n in RAIL_NODES:
		if String(n.get("kind", "")) != "node" or String(n["domain"]) != id:
			continue
		if (n.get("owns", []) as Array).has(category):
			return String(n["mode"])
	return ""

## A workspace module calls this once, from `_ready`, with the panel it wants in
## the left dock. Panels are built up front and hidden, not rebuilt on every
## switch -- §3 requires each domain's L2 open/closed state to persist.
func register_workspace(id: String, panel: Control) -> void:
	panel.visible = id == _active_domain
	panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## Which domain's dock this panel *is*. A `Workspace` could not answer that
	## before: `app.active_domain()` is the domain currently on screen, which is
	## the wrong question for a panel that is not on screen -- and every one of
	## the three is off screen two thirds of the time. `apply_mode()` needs it to
	## look its own `shows` list up in `RAIL_NODES`, so it is written here, at the
	## one call that knows the pairing, rather than duplicated as a constant in
	## each of the three subclasses where it could drift from this dictionary.
	if panel.has_method("bind_domain"):
		panel.call("bind_domain", id)
	_workspace_panels[id] = panel
	left_dock_body.add_child(panel)
	## **Deferred, not called here directly -- measured, not assumed.**
	## `dcc_widgets.gd`'s own header claims a workspace builds its whole panel
	## before this call ever runs; `app.gd::_register_workspaces()` disproves
	## it for the panel *itself*: `register_workspace(entry[0], ws)` runs
	## against a bare `WorldWorkspace.new()` with only its `name` set, and
	## `ws.setup(self, bridge)` -- which is what actually fills it with every
	## category, section and row -- runs on the very next line, after this
	## function has already returned. A synchronous walk here would floor an
	## empty tree and fix nothing; `category()`'s own `_fill_category_count()`
	## already reaches for `call_deferred()` for exactly this reason (a body
	## the caller has not filled yet), and the same fix applies here one level
	## up. By the end of the current frame every workspace registered this
	## frame has also been `setup()` -- `_register_workspaces()`'s loop calls
	## both synchronously, with no `await` between them. No-op on desktop and
	## phone.
	call_deferred("tablet_fit", panel)

func active_domain() -> String:
	return _active_domain

## One domain's left-dock panel. Public for `_railfold_probe.gd`, which has to
## read each workspace's own `categories` array to prove the fold stranded
## nothing -- and which must not do that by reaching into `_workspace_panels`,
## since a probe that knows a private field is a probe that breaks on a rename
## instead of on a regression.
func workspace_panel(id: String) -> Control:
	return _workspace_panels.get(id)

## WI-02's public entry point: `menus.gd`'s Window ▸ Workspace submenu jumps
## here rather than calling the underscore-prefixed `_select_domain()`
## directly across a file boundary.
func select_domain(id: String) -> void:
	_select_domain(id)

## Switch domain **and** open the category the caller named -- what every
## "→ Cartography ▸ Political display"-style jump button in the shell actually
## promises. See `Workspace.open_category()` for why half of it was not enough
## once v3 gave CIVIL fourteen categories and CARTO ten.
##
## **The signature carries a mode now** (stage 2, 2026-08-31). Before the rail
## grew its node tree, "switch domain and open a category" was the whole of
## navigation. It no longer is: the rail also has a lit node per domain, and a
## jump that moved the dock without moving the node would leave the two
## disagreeing about where the user is -- the shell claiming CIVIL ▸ Landmarks in
## the rail while the dock sits open on Military.
##
## `mode` is **optional and defaults to derived**, not required. See
## `mode_for_category()` for the argument: the mapping category → mode is a fact
## about `RAIL_NODES`, not about the call site, so a required third argument at
## eight call sites would be eight chances to disagree with the one table that
## knows. Pass it explicitly only to override -- i.e. when the caller wants a
## node lit that does not own the category it is opening, which nothing in this
## shell wants today.
##
## Silent about a miss on purpose at the call site, loud in the log: a stale
## pointer must not swallow the domain switch the user asked for, and a
## `push_warning` is what a probe can assert on.
func select_domain_category(id: String, category: String, mode: String = "") -> void:
	var m := mode if not mode.is_empty() else mode_for_category(id, category)
	## `""` means no node owns this category, which `_railfold_probe.gd` §3
	## proves cannot happen for any category the three docks actually build. If
	## it somehow does, the domain switch and the category open still happen --
	## losing the rail highlight is a far smaller failure than swallowing the
	## navigation, which is the same judgement the `push_warning` below encodes.
	if not m.is_empty():
		_domain_mode[id] = m
	_select_domain(id)
	var panel: Control = _workspace_panels.get(id)
	if panel != null and panel.has_method("open_category"):
		if not panel.call("open_category", category):
			push_warning("Cartalith: no category '%s' in the %s dock -- stale cross-domain pointer." % [category, id])

## §6's right-dock header -- see `_build_right_dock()`'s own comment on why
## this exists instead of a fixed "LAYERS" label. `text` is already the
## upper-cased section name; `DccTheme.header()` built the sigil-free label
## once with an initial value, this just updates its text in place.
func set_right_dock_title(text: String) -> void:
	if right_dock_title != null:
		right_dock_title.text = text.to_upper()

# -- §6 Docks -----------------------------------------------------------------

## `as_sheet`: §13's phone treatment -- "docks become full-height sheets, one
## at a time". The header swaps its collapse chevron for a close button and
## the dock stops claiming a fixed desktop width, but the body underneath
## (`left_dock_body`, where every workspace attaches its panel) is built
## exactly as it is for desktop/tablet, unchanged -- this is the "minimal or
## no change" reuse the phone chrome depends on.
func _build_left_dock(as_sheet: bool = false) -> Control:
	left_dock = PanelContainer.new()
	if not as_sheet:
		left_dock.custom_minimum_size.x = _left_width
	left_dock.add_theme_stylebox_override("panel",
		DccTheme.panel("panel") if as_sheet else DccTheme.panel("panel", {"right": 1}))

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if as_sheet:
		left_dock.add_child(col)
	else:
		## WI-04 (§1: "user-draggable within min/max"): a 6 px grip at the
		## dock's inner edge, facing the viewport -- carved out of the dock's
		## own reserved width rather than added to it, so `_left_width` still
		## means what it always has. `_dock_drag_handle()` owns the drag math;
		## the right dock mirrors this with the handle on its other side.
		var body_row := HBoxContainer.new()
		body_row.add_theme_constant_override("separation", 0)
		body_row.add_child(col)
		body_row.add_child(_dock_drag_handle(true))
		left_dock.add_child(body_row)

	var head := HBoxContainer.new()
	## 34 px, not 26: the canvas gives both dock headers `height:34px` -- the
	## same band the menu bar and the tool options bar get, so the three
	## horizontal rules across the top of the shell line up as one rhythm.
	## 26 is `H_STATUS`, borrowed here by mistake, and it left the dock title
	## sitting 4 px proud of the tool options bar beside it.
	head.custom_minimum_size.y = _ptap(44) if as_sheet else _scaled(34)
	left_dock_title = DccTheme.header("WORLD", "")
	head.add_child(left_dock_title)
	head.add_child(DccTheme.spacer())
	if as_sheet:
		head.add_child(_sheet_close_button(func(): _set_sheet_open("left", false)))
	else:
		head.add_child(_collapse_button(true))
	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 12)
	head_pad.add_theme_constant_override("margin_right", 6)
	head_pad.add_child(head)
	col.add_child(head_pad)
	col.add_child(DccTheme.rule())
	## Always built, even in sheet mode, so `set_dock_readout("left", …)` never
	## hits the "no dock readout" error -- it just stays permanently hidden,
	## since a sheet has no collapsed state to surface it in. Its writers are
	## `Workspace.push_dock_readout()` and `world_workspace.gd`'s override of
	## it, reached on every domain switch from `DccApp._on_workspace_changed()`;
	## until 2026-09-05 the WORLD one was the only writer in the shell.
	col.add_child(_dock_readout("left"))
	## `04-left-dock.md` §2.1's band 2 -- pinned above the scroll body, exactly
	## where the canvas puts it, so the one control that can put a hidden block
	## back never scrolls away from the body it is hiding.
	col.add_child(_build_mode_switch())

	var scroll := _scroll()
	_left_dock_scroll = scroll
	left_dock_body = VBoxContainer.new()
	left_dock_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left_dock_body.add_theme_constant_override("separation", 0)
	scroll.add_child(left_dock_body)
	col.add_child(scroll)
	return left_dock

## `as_sheet`: see `_build_left_dock()` -- identical treatment, mirrored.
func _build_right_dock(as_sheet: bool = false) -> Control:
	right_dock = PanelContainer.new()
	if not as_sheet:
		right_dock.custom_minimum_size.x = _right_width
	right_dock.add_theme_stylebox_override("panel",
		DccTheme.panel("panel") if as_sheet else DccTheme.panel("panel", {"left": 1}))

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if as_sheet:
		right_dock.add_child(col)
	else:
		## `as_sheet`: see `_build_left_dock()`'s own drag-handle comment --
		## identical treatment, mirrored so the grip faces the viewport here too.
		var body_row := HBoxContainer.new()
		body_row.add_theme_constant_override("separation", 0)
		body_row.add_child(_dock_drag_handle(false))
		body_row.add_child(col)
		right_dock.add_child(body_row)

	## §6's own header carries whatever context is showing (Sample, Settlement,
	## Route...), not a fixed label -- "Layers" was this dock's mockup-pictured
	## *default* state, not its permanent chrome title; a bare "LAYERS" left
	## painted here regardless of context was misleading once every other
	## context (Sample, Settlement, Route, River, Faction, Measure, Region
	## select, Stamp stack, Journey) had its own real section header one
	## scroll-step below it saying something else. `right_dock.gd`'s
	## `_rebuild()` keeps this in sync via `set_right_dock_title()`, the same
	## pattern `left_dock_title` already follows for the domain name.
	right_dock_title = DccTheme.header("SAMPLE", "")
	var head := HBoxContainer.new()
	## 34 px, not 26: the canvas gives both dock headers `height:34px` -- the
	## same band the menu bar and the tool options bar get, so the three
	## horizontal rules across the top of the shell line up as one rhythm.
	## 26 is `H_STATUS`, borrowed here by mistake, and it left the dock title
	## sitting 4 px proud of the tool options bar beside it.
	head.custom_minimum_size.y = _ptap(44) if as_sheet else _scaled(34)
	if as_sheet:
		head.add_child(right_dock_title)
		head.add_child(DccTheme.spacer())
		head.add_child(_sheet_close_button(func(): _set_sheet_open("right", false)))
	else:
		head.add_child(_collapse_button(false))
		head.add_child(right_dock_title)
		head.add_child(DccTheme.spacer())
	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 6)
	head_pad.add_theme_constant_override("margin_right", 12)
	head_pad.add_child(head)
	col.add_child(head_pad)
	col.add_child(DccTheme.rule())
	col.add_child(_dock_readout("right"))

	var scroll := _scroll()
	_right_dock_scroll = scroll
	right_dock_body = VBoxContainer.new()
	right_dock_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right_dock_body.add_theme_constant_override("separation", 0)
	scroll.add_child(right_dock_body)
	col.add_child(scroll)
	return right_dock

## Godot's default theme draws a rounded, outlined panel behind every
## ScrollContainer. §11 is explicit that regions are separated by hairlines
## only, with radius 0 everywhere, so the panel is removed rather than
## restyled -- the dock around it already draws the one border there should be.
func _scroll() -> ScrollContainer:
	var s := ScrollContainer.new()
	s.size_flags_vertical = Control.SIZE_EXPAND_FILL
	s.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	s.add_theme_stylebox_override("panel", DccTheme.empty())
	return s

## WI-04: the drag grip itself, a bare 6 px strip that lights up on hover so
## it reads as an affordance without adding chrome the spec doesn't draw.
## `is_left` picks which dock it belongs to -- purely to route the drag delta
## and to know which of `_left_width`/`_right_width` and which min/max pair
## (§1's geometry table) apply, since dragging right *widens* the left dock
## but *narrows* the right one.
func _dock_drag_handle(is_left: bool) -> Control:
	var handle := PanelContainer.new()
	handle.custom_minimum_size.x = 6
	handle.mouse_default_cursor_shape = Control.CURSOR_HSPLIT
	handle.mouse_filter = Control.MOUSE_FILTER_STOP
	handle.add_theme_stylebox_override("panel", DccTheme.empty())
	handle.mouse_entered.connect(func(): handle.add_theme_stylebox_override(
		"panel", DccTheme.flat(DccTheme.c("line_soft"))))
	handle.mouse_exited.connect(func():
		if _dragging_dock == "":
			handle.add_theme_stylebox_override("panel", DccTheme.empty()))
	handle.gui_input.connect(_on_dock_drag_input.bind(is_left))
	return handle

## Godot routes mouse motion to whichever Control had the initial press even
## once the cursor drifts off its rect (the same mechanism `SplitContainer`'s
## own internal dragger relies on), so this needs no separate `_input`
## override to track a drag past the handle's own 6 px width.
func _on_dock_drag_input(ev: InputEvent, is_left: bool) -> void:
	var side := "left" if is_left else "right"
	if is_dock_collapsed(side):
		return
	if ev is InputEventMouseButton and ev.button_index == MOUSE_BUTTON_LEFT:
		_dragging_dock = side if ev.pressed else ""
	elif ev is InputEventMouseMotion and _dragging_dock == side:
		## Dragging right widens the left dock (its handle sits on its right
		## edge) but narrows the right dock (its handle sits on its left edge).
		var delta: float = ev.relative.x if is_left else -ev.relative.x
		if is_left:
			_left_width = clampf(_left_width + delta,
				float(DccTheme.W_LEFT_DOCK_MIN), float(DccTheme.W_LEFT_DOCK_MAX))
			left_dock.custom_minimum_size.x = _left_width
		else:
			_right_width = clampf(_right_width + delta,
				float(DccTheme.W_RIGHT_DOCK_MIN), float(DccTheme.W_RIGHT_DOCK_MAX))
			right_dock.custom_minimum_size.x = _right_width

func _collapse_button(is_left: bool) -> Button:
	var b := Button.new()
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.text = DccIcons.SYMBOLS["collapse"] if is_left else DccIcons.SYMBOLS["expand"]
	## Glyph-only, and it had no tooltip either -- so this square was unnamed to
	## a pointer as well as to a screen reader. Both, now.
	b.tooltip_text = "Collapse or expand the %s dock" % ("left" if is_left else "right")
	b.accessibility_name = b.tooltip_text
	b.add_theme_color_override("font_color", DccTheme.c("text_faint"))
	b.custom_minimum_size = Vector2(_scaled(20), _scaled(20))
	b.pressed.connect(_toggle_dock.bind(is_left))
	_collapse_buttons["left" if is_left else "right"] = b
	return b

## §6's last line: "collapsed, the dock keeps its primary readout visible --
## elevation for Sample, layer dots for Layers, stamp count for the stack." So a
## collapsed dock is not an empty 40 px strip; it is a strip that still says the
## one thing you collapsed it in order to keep watching.
##
## The label lives outside the ScrollContainer precisely because collapsing
## hides that container -- putting the readout inside it would hide the thing
## the rule exists to preserve.
# -- §2.3 The left dock's mode switch (`ldSwitch`) -----------------------------
#
# `04-left-dock.md` §2.1 band 2: a two-segment pill, pinned between the dock
# header and the TOOLS block, *"shown only when `ldSwitch` is true"*. §2.3 reads
# it as the WORLD **a / b** switch and says so; §9.1 records that the literal
# condition, and both segment labels, went with the prototype's truncated tail.
#
# **Why it exists here rather than being left to the rail.** `RAIL_NODES`'
# `shows` key gives `world/b` a real gate -- in Sculpt the dock renders one
# category and hides eight. The rail can put them back, but the rail's node list
# is `railExp:false` at rest (`ENV:1199`), so from inside a gated dock the way
# out is two clicks through a column that is not on screen. The canvas draws the
# pill precisely so that it is one click, in the dock, beside what it hides.
# That is the whole reason a gate is permitted at all here (see `RAIL_NODES`'
# header), so the switch and the gate ship together or neither ships.
#
# **Only the VISIBILITY is derived. Corrected 2026-09-05 after a verifier read
# the rest of the function.** `domain_gates()` asks whether any of a domain's
# nodes carries a `shows`, so the pill appears for a gated domain without being
# told which one. Everything else here is hardcoded to WORLD:
# `_build_mode_switch()` iterates `domain_nodes("world")`, `_MODE_SWITCH_LABELS`
# is `{a: PIPELINE, b: SCULPT}`, and `_refresh_mode_switch()` reads WORLD's mode.
#
# So a future CIVIL or CARTO gate would show an **empty pill with WORLD's two
# labels**, not its own affordance — the opposite of what the first version of
# this comment claimed. Generalising it means sourcing the nodes and the labels
# from the active domain too; that is a real change, not a rename, and it is not
# made here because nothing needs it yet.
#
# **The two labels are a decision, not a quotation.** §9.1: *"the switch's own
# two labels are not recoverable ... `Generation pipeline` / `Sculpt` are the
# closest evidence but are 19 and 6 characters -- almost certainly not the pill
# text."* `PIPELINE` / `SCULPT` is this port's choice: both are words the rail's
# own node labels already use, both fit the `--m2` uppercase mono the dock sets
# every heading in, and they balance at 8 and 6 characters across two `flex:1`
# halves. Recorded the same way `Workspace.push_dock_readout()` records its own
# unrecoverable string, rather than passed off as the design's.
var _mode_switch_row: Control
var _mode_switch_buttons: Dictionary = {}   ## mode -> Button
const _MODE_SWITCH_LABELS: Dictionary = {"a": "PIPELINE", "b": "SCULPT"}

func _build_mode_switch() -> Control:
	var pad := MarginContainer.new()
	## §2.1 band 2's own box: `padding: 8px var(--pad) 2px`.
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_theme_constant_override("margin_top", 8)
	pad.add_theme_constant_override("margin_bottom", 2)
	pad.visible = false
	_mode_switch_row = pad

	## §2.3's container: `background: var(--ins); border-radius:999px; padding:3px`.
	## `--ins` is the `sunken` token. **The radius is not reproduced**: §11's
	## "radius 0 everywhere" governs every desktop artboard, `DccTheme.pill()`'s
	## own header records that the phone canvas is the single exception, and the
	## shell's 22 other `set_segment_on()` call sites are all square. A rounded switch
	## among square siblings would read as a different kind of control.
	var shellbox := PanelContainer.new()
	var pill_box := DccTheme.flat(DccTheme.c("sunken"))
	pill_box.content_margin_left = 3
	pill_box.content_margin_right = 3
	pill_box.content_margin_top = 3
	pill_box.content_margin_bottom = 3
	shellbox.add_theme_stylebox_override("panel", pill_box)
	pad.add_child(shellbox)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 3)
	shellbox.add_child(row)

	for n in domain_nodes("world"):
		var mode := String(n["mode"])
		var b := DccWidgets.segment(row, String(_MODE_SWITCH_LABELS.get(mode, mode)),
			_on_mode_switch_pressed.bind(mode))
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		## §2.3's `flex:1` halves are `--ctl` tall (24 desktop / 36 tablet).
		## **The tablet figure is raised to the 44 px tap floor rather than taking
		## the canvas's 36**, which is the same call `_scaled()` makes for every
		## height its own table does not name; `DccWidgets.segment()`'s
		## `role_px("chip_min_h")` of 34 is below the floor and is a standing
		## shell-wide issue, so matching it would not be conformance.
		##
		## **The phone takes the authored 24 and nothing else.** `phone_fit()`
		## multiplies `custom_minimum_size.y` by its own unit and *then* floors
		## every `BaseButton` at `PHONE_TAP_MIN * unit`, so writing a
		## pre-scaled `_ptap(24)` here scaled it twice: measured at **301 px**
		## for one segment on a 1080x2340 sheet, against the 115 px the floor
		## actually asks for. Authoring in desktop pixels and letting the fitter
		## scale is what every other control in the dock does.
		b.custom_minimum_size.y = 44 if (_touch and not _phone) else 24
		## Two segments, one lit: the state has to be legible without colour
		## alone, so the tooltip names it in words. `set_segment_on()` supplies
		## the accent ink and wash; `_refresh_mode_switch()` owns both.
		_mode_switch_buttons[mode] = b
	_refresh_mode_switch()
	return pad

## Show the pill for a gating domain, hide it for the rest, and light whichever
## segment is the active mode. Called from `_select_domain()` -- the one choke
## point every domain and every mode change passes through -- so the pill cannot
## disagree with the body beneath it.
func _refresh_mode_switch() -> void:
	if _mode_switch_row == null or not is_instance_valid(_mode_switch_row):
		return
	var on := domain_gates(_active_domain) and not _left_collapsed
	_mode_switch_row.visible = on
	if not on:
		return
	var active := String(_domain_mode.get(_active_domain, ""))
	for mode in _mode_switch_buttons:
		var b: Button = _mode_switch_buttons[mode]
		if not is_instance_valid(b):
			continue
		var lit: bool = String(mode) == active
		DccWidgets.set_segment_on(b, lit)
		b.tooltip_text = "%s — %s" % [
			String(rail_node("world", String(mode)).get("label", mode)),
			"showing" if lit else "click to show"]

## Pressing a segment is exactly a rail-node press on the same node, minus the
## expansion column: same mode write, same category open, same gate. Routed
## through `select_domain_mode()` rather than writing `_domain_mode` here, so a
## future node behaviour (the Journey takeover is one already) cannot arrive for
## the rail and not for the pill.
func _on_mode_switch_pressed(mode: String) -> void:
	select_domain_mode("world", mode)

func _dock_readout(side: String) -> Control:
	var l := DccTheme.label("", "text_dim", DccTheme.FS_TINY)
	l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 4)
	pad.add_theme_constant_override("margin_right", 4)
	pad.add_theme_constant_override("margin_top", 8)
	pad.add_child(l)
	pad.visible = false
	_dock_readouts[side] = l
	return pad

## Whatever the dock's current context considers its one essential number. Kept
## up to date whether or not the dock is collapsed, so collapsing never reveals
## a stale value.
func set_dock_readout(side: String, text: String) -> void:
	if not _dock_readouts.has(side):
		push_error("DccShell: no dock readout for side '%s'" % side)
		return
	(_dock_readouts[side] as Label).text = text

func is_dock_collapsed(side: String) -> bool:
	return _left_collapsed if side == "left" else _right_collapsed

## A collapsed dock shrinks to the rail width rather than disappearing, and
## swaps its body for the readout above.
func _toggle_dock(is_left: bool) -> void:
	var dock := left_dock if is_left else right_dock
	var side := "left" if is_left else "right"
	var collapsed := not (_left_collapsed if is_left else _right_collapsed)
	dock.custom_minimum_size.x = float(DccTheme.W_RAIL_COLLAPSED) if collapsed else (_left_width if is_left else _right_width)
	## `dock.get_child(0)` is the drag-handle `HBoxContainer` (`_dock_drag_handle()`
	## wraps `col` and the grip together), not the `ScrollContainer` -- which sits
	## one level deeper, inside `col`. Hiding the wrong child left the real
	## `ScrollContainer`'s content visible and its minimum size still forcing the
	## dock wider than `W_RAIL_COLLAPSED`. Go straight to the stored reference
	## instead of walking the tree.
	var scroll := _left_dock_scroll if is_left else _right_dock_scroll
	if scroll != null:
		scroll.visible = not collapsed
	(_dock_readouts[side] as Label).get_parent().visible = collapsed
	if is_left:
		## The title has no room at 40 px; the chevron is all that fits, and it
		## is the only affordance for getting the dock back.
		left_dock_title.visible = not collapsed
		_left_collapsed = collapsed
		## **And neither has the mode switch** (`04-left-dock.md` §2.1 band 2).
		## Two `SIZE_EXPAND_FILL` segments carrying `PIPELINE` and `SCULPT` have
		## a combined minimum width far past `W_RAIL_COLLAPSED`, and a
		## `MarginContainer` propagates its child's minimum to the `VBoxContainer`
		## and on to the dock -- so leaving it up would hold the collapsed strip
		## open at the switch's width with no scrollbar anywhere to show why.
		## The same reason the title is hidden two lines above, and the same
		## failure class as the disabled-axis `ScrollContainer`s this tree has
		## repeatedly grown. `_refresh_mode_switch()` re-reads `_left_collapsed`, so
		## re-opening the dock restores it in the right state.
		_refresh_mode_switch()
	else:
		_right_collapsed = collapsed
	var btn: Button = _collapse_buttons.get(side)
	if btn != null:
		var open_glyph: String = DccIcons.SYMBOLS["collapse"] if is_left else DccIcons.SYMBOLS["expand"]
		var shut_glyph: String = DccIcons.SYMBOLS["expand"] if is_left else DccIcons.SYMBOLS["collapse"]
		btn.text = shut_glyph if collapsed else open_glyph

# -- §9 Viewport --------------------------------------------------------------

func _build_viewport() -> Control:
	var area := PanelContainer.new()
	area.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	area.add_theme_stylebox_override("panel", DccTheme.flat(DccTheme.c("bg")))
	viewport_area = area
	viewport_content = Control.new()
	viewport_content.clip_contents = true
	area.add_child(viewport_content)
	return area

# -- §9a The viewport context chip (`vpContext`) ------------------------------
#
# `05-right-dock-and-bars.md` §5.2 draws a chip beside the Layers button and
# leaves `vpContext` `UNSPECIFIED:` -- "its string in every context", with the
# one surviving hook `vpCtxExtra()` returning `''`. The 2026-08-31 re-export
# supplies it (`Cartalith DCC Environment.dc.html`, `const vpCtx = ...` in
# `valsCore()`), a four-arm fall-through:
#
#   1  a run is active            GENERATING — STAGE NN
#   2  WORLD, sculpt mode         SCULPT · DRAFT
#   3  WORLD, pipeline mode       STAGE NN · EDITED   /  STAGE NN · RESOLVED
#   4  otherwise                  the domain's own name
#
# Arms 1, 2 and 4 port exactly. **Arm 3's number does not, and is dropped rather
# than guessed.** The prototype's `NN` is `staleFrom || openStage`: `staleFrom`
# is an index into its own ten-stage pipeline, and this port's staleness is
# `stale_stages()`, keyed by the stage-graph's *names* (`height`, `hydrology`,
# `climate`, `civ`) with no index anywhere; `openStage` is the left dock's open
# accordion, which this class has no accessor for and should not grow one for a
# chip. Both halves of `staleFrom || openStage` are therefore unavailable, while
# the `EDITED` / `RESOLVED` verdict beside them is exactly answerable. So the
# chip says the verdict over the domain name -- both words the design's own,
# arm 4's noun under arm 3's adjective -- and the tooltip says what is missing.
# `WORLD · EDITED` is a true sentence; `STAGE 07 · EDITED` would be a guess.
#
# Written from here rather than from `viewport_host.gd` because the string is
# composed of domain, mode and generation state and that node knows none of the
# three -- the same push-not-poll split `set_style_readout()` already uses.

## The 1-based index of the stage a run is currently in, read off the engine's
## own `generation_stage` tick. `0` between runs.
##
## **Not a second copy of `progress.rs::STAGE_NAMES`** -- the index only, off the
## signal, which is the discipline `app.gd::_wire_status()`'s own comment sets
## out for `statusMid` after a duplicated stage table drifted once already.
var _vp_stage := 0
var _vp_wired := false

## Composes and pushes §5.2's chip. Cheap enough to call from every signal that
## can change it; the one non-trivial read is `stale_stages()`, and
## `EngineBridge.mark_dirty()` early-returns once already dirty, so
## `params_changed` fires on the clean->dirty transition and not per drag frame.
func _refresh_viewport_context() -> void:
	var host := _find_viewport_host()
	if host == null:
		return
	var bridge := _find_engine_bridge()
	if not _vp_wired and bridge != null:
		_vp_wired = true
		bridge.generation_started.connect(func():
			_vp_stage = 0
			_refresh_viewport_context())
		bridge.generation_stage.connect(func(index: int, _name: String, _total: int):
			_vp_stage = index + 1
			_refresh_viewport_context())
		bridge.generation_finished.connect(func(_ok: bool):
			_vp_stage = 0
			_refresh_viewport_context())
		bridge.world_loaded.connect(_refresh_viewport_context)
		## The stale/settled pair: `params_changed` is "a dial moved; downstream
		## is stale", `params_applied` is "a generate landed; nothing is stale".
		bridge.params_changed.connect(_refresh_viewport_context)
		bridge.params_applied.connect(_refresh_viewport_context)

	var text := ""
	var tip := ""
	if bridge != null and bridge.generating:
		## Arm 1. The index is absent for the stretch between `generation_started`
		## and the first stage tick, so the chip says the state without inventing
		## a stage number for it.
		text = ("GENERATING — STAGE %02d" % _vp_stage) if _vp_stage > 0 else "GENERATING"
		tip = "A generation run is in flight. Stage numbers come from the engine's own generation_stage tick."
	elif _active_domain == "world" and active_mode("world") == "b":
		## Arm 2, verbatim.
		text = "SCULPT · DRAFT"
		tip = "Sculpt is armed. Stamps are a draft until they are committed; the height field under them is unchanged."
	else:
		## Arms 3 and 4. `stale_stages()` refuses mid-generation and answers `{}`
		## for a world-less session, which is also the honest "nothing is stale".
		var rail := ""
		for d in DOMAINS:
			if String(d.id) == _active_domain:
				rail = String(d.rail)
				break
		if rail == "":
			return
		if bridge == null or not bridge.has_world:
			text = rail
			tip = "No world yet. Generate one and this chip reports whether the map still rests on the last full pass."
		elif _active_domain != "world":
			## **The verdict is WORLD's alone.** `ENV:1889` applies
			## `EDITED`/`RESOLVED` only under `s.domain === 'WORLD'`; every other
			## domain gets the bare rail. This branch appended it for *every*
			## domain until 2026-09-03, so a generated world read
			## `CIVIL · RESOLVED` where the design says `CIVIL`.
			##
			## Found by a verifier, and invisible to the probe that covered this
			## function: that probe is world-less by construction, and the
			## world-less arm above already returns the bare rail — so the two
			## agreed for the wrong reason.
			text = rail
			tip = ("Staleness is reported on WORLD, where the generation graph lives. "
				+ "This domain shows its rail alone, as the design does.")
		else:
			var stale: Dictionary = bridge.stale_stages()
			text = "%s · %s" % [rail, "EDITED" if not stale.is_empty() else "RESOLVED"]
			if stale.is_empty():
				tip = ("Every stage the graph tracks has re-run since the last edit, so the map "
					+ "is what the current parameters produce.")
			else:
				var names: Array = stale.keys()
				names.sort()
				tip = ("Stale: %s. The design's own chip names the stage NUMBER here; this port's "
					+ "stage graph is keyed by name and has no index, so the verdict is reported "
					+ "and the number is not invented. Recompute from the status bar to settle it.") \
					% ", ".join(names)
	host.set_viewport_context(text, tip)

# -- §10 Timeline bar ---------------------------------------------------------

func _build_timeline() -> Control:
	var bar := PanelContainer.new()
	## **No fixed height any more.** `01-frame-and-tokens.md` §3.7 gives the strip
	## two forms with two different heights -- collapsed is
	## `calc(var(--sbH) - 2px)` and expanded is explicitly *auto* -- so a single
	## `H_TIMELINE` could only ever be right for one of them. It was right for
	## neither: the region drew 70 px of blank panel in CIVIL
	## (`app.gd::_fill_timeline_strip`'s own measurement), and the journey
	## planner's day band, which is 20 px of labels, got the same 70.
	## Content-driven, both forms and the band land on their own height.
	bar.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"top": 1}))
	timeline_row = HBoxContainer.new()
	timeline_row.add_theme_constant_override("separation", 14)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_theme_constant_override("margin_top", 8)
	pad.add_theme_constant_override("margin_bottom", 8)
	pad.add_child(timeline_row)
	bar.add_child(pad)
	return bar

## The two boxes `_build_timeline()` deliberately does not fix, set by whoever
## fills `timeline_row` -- because §3.7 gives the strip's two forms two
## different ones and the row is shared by four fillers.
##
## `pad_y` is the vertical half of the CSS `padding`: `8px var(--pad)` expanded,
## **`0 var(--pad)` collapsed**. `fixed_h` is the collapsed form's
## `height:calc(var(--sbH) - 2px)`, or `0` for "auto", which is what §3.7 says
## the expanded form is.
##
## Set on the `PanelContainer`, border included, matching `_build_status_bar()`
## -- which takes `--sbH` as the whole bar's height with the same 1 px top rule
## inside it, and matching `02-rail-and-domains.md`'s own frame arithmetic
## (`1080 − 36 − 40 − 26`), which subtracts the token and not the token plus a
## border.
##
## **The horizontal margins are not touched**: `var(--pad)` is 14 in both forms
## and was already right.
func set_timeline_metrics(pad_y: int, fixed_h: int) -> void:
	if timeline_row == null or timeline_bar == null:
		return
	var pad := timeline_row.get_parent() as MarginContainer
	if pad == null:
		return
	pad.add_theme_constant_override("margin_top", pad_y)
	pad.add_theme_constant_override("margin_bottom", pad_y)
	timeline_bar.custom_minimum_size.y = float(fixed_h)

# -- §10a The timeline model ------------------------------------------------
#
# **One cursor, two views.** The year cursor already existed before this strip
# did: it is `CivData::year`, reached through `civ_goto_year()`/`get_civ_year()`,
# and the CIVIL dock's Politics category has drawn it as a row of recorded-year
# pills since the timeline milestone landed. `01-frame-and-tokens.md` §3.7 and
# `05-right-dock-and-bars.md` §4.2 add a second *view* of it -- the desktop
# strip -- and `06-phone.md` §6.2 a third, the phone sim strip. All three read
# and write the engine's own cursor through the four functions below and keep no
# year of their own, which is the only arrangement in which they cannot drift.
#
# What is *not* the engine's, and therefore lives here: whether playback is
# running, the speed multiplier, and the six layer toggles. None of the three
# has an engine counterpart to defer to.
#
# The two figures the prototype leaves `UNSPECIFIED` and this file has to
# choose, both chosen so the speed pill means one thing rather than two:
#   - **step size** is `tl_speed` years. `hTlStep`'s own step is truncated out
#     of the delivered file.
#   - **playback** advances the cursor by `tl_speed` years every `TL_TICK_SEC`,
#     which is §6.2's rule for the phone strip stated literally ("playing
#     advances the year by `speed` every 600 ms") and is applied to both views.
#
# `civ_goto_year(y)` accepts any year: it writes `CivData::year` unconditionally
# and then loads a snapshot only if one was recorded for exactly that year
# (`cartalith-godot/src/lib.rs:400 CivData::civ_goto_year` -- note there are
# TWO `civ_goto_year` in that file; this is the inner one, on `CivData`, not
# the `#[func]` wrapper on `WorldGen` cited further down). So the -400..1200
# track is continuous and
# honest -- the cursor really does land where the playhead is -- and the
# territory under it changes only at the years the dock has recorded.

## `05-right-dock-and-bars.md` §4.2: "the scrub range is therefore fixed at
## year -400 ... year 1200 (1600 years)", and `06-phone.md` §6.2's slider is
## `min=-400 max=1200 step=1`. The same two numbers, in both canvases.
const TL_YEAR_MIN := -400
const TL_YEAR_MAX := 1200
## `tlSpeeds` is truncated out of the prototype; §4.3 records that its markup
## says three options and that `×10` is the surviving default. `06-phone.md`
## §6.2 has the other two verbatim -- `×1 ×10 ×100` -- so the ladder is
## recovered from the phone canvas rather than guessed.
const TL_SPEEDS := [1, 10, 100]
const TL_TICK_SEC := 0.6
## §4.3's `tlTog`, in its order, with its defaults. The markup renders the id
## as the pill's text, so these strings are the visible labels verbatim.
const TL_LAYERS := [
	["Climate", true], ["Population", true], ["Economy", false],
	["Politics", true], ["Infrastructure", false], ["Warfare", false],
]
## `BUILD_ANSWERS.md` §3, verbatim and quoted rather than paraphrased: the six
## toggles are *intended* rather than an oversight, and the owner fixed both the
## note and where it has to sit ("note on the timeline"). Drawn on the strip by
## `app.gd::_fill_timeline_strip()` and repeated as each pill's tooltip.
const TL_LAYER_NOTE := "they record which layer you want; no layer renders yet"

## Emitted whenever the cursor, the speed, the run state or a layer toggle
## moves -- the one notification both views rebuild on, so neither has to know
## the other exists.
##
## **`workspaces/civilization_workspace.gd` should connect to this too**: its
## year pills and its `_refresh_civ_data()` are the third view of the same
## cursor, and a scrub from the strip leaves them a frame behind until it does.
## Named here rather than left silent; that file is not this pass's to edit.
signal timeline_changed()

var tl_speed := 10            ## `tlSpeed:'×10'`.
var tl_playing := false       ## `tlRun:false`.
var tl_layers: Dictionary = {}  ## label -> bool. Loaded in `_ready()`.
var _tl_timer: Timer          ## Playback. Built on first play, never before.

## Whether the cursor can be moved at all -- **not merely whether moving it is
## interesting**. `WorldGen::civ_goto_year` is `if let Some(civ) = self.civ`
## (`cartalith-godot/src/lib.rs:11062 WorldGen::civ_goto_year` -- the
## `#[func]` wrapper, not the `CivData` method of the same name at `:391`)
## and `self.civ` is `None` until a `generate()` has run, so
## before one every transport control here is a control with nothing behind it.
## Measured, not assumed: a probe run of this strip against a world-less shell
## set the cursor to 412, to 99999 and to -99999 and read back `0` all three
## times. Every view draws its transport disabled, carrying `TL_UNAVAILABLE`,
## when this is false.
func tl_available() -> bool:
	var bridge := _find_engine_bridge()
	return bridge != null and bridge.has_world

const TL_UNAVAILABLE := "Generate a world first. The year cursor is civilisation state, and civ_goto_year is a no-op before any generate -- moving it now would silently do nothing."

## The cursor. `0` with no engine and before any generate, which is
## `CivData::year`'s own init value rather than a stand-in for it.
func tl_year() -> int:
	var bridge := _find_engine_bridge()
	return 0 if bridge == null else bridge.get_civ_year()

## Move the cursor. Clamped to the track, because both canvases draw a fixed
## -400..1200 axis and a playhead outside it has nowhere to be.
func tl_set_year(year: int) -> void:
	if not tl_available():
		return
	var bridge := _find_engine_bridge()
	if bridge == null:
		return
	bridge.civ_goto_year(clampi(year, TL_YEAR_MIN, TL_YEAR_MAX))
	timeline_changed.emit()

func tl_step(direction: int) -> void:
	tl_set_year(tl_year() + direction * tl_speed)

func tl_set_speed(mult: int) -> void:
	if not TL_SPEEDS.has(mult):
		return
	tl_speed = mult
	if _tl_timer != null:
		_tl_timer.wait_time = TL_TICK_SEC
	timeline_changed.emit()

## §6.2: playback stops at the top of the track rather than wrapping -- "capped
## at 1200". The desktop canvas states no wrap either.
func tl_toggle_play() -> void:
	if not tl_available():
		return
	tl_playing = not tl_playing
	if tl_playing and tl_year() >= TL_YEAR_MAX:
		tl_playing = false
	if _tl_timer == null:
		_tl_timer = Timer.new()
		_tl_timer.name = "TimelinePlayback"
		_tl_timer.wait_time = TL_TICK_SEC
		_tl_timer.timeout.connect(_tl_tick)
		add_child(_tl_timer)
	if tl_playing:
		_tl_timer.start()
	else:
		_tl_timer.stop()
	timeline_changed.emit()

func _tl_tick() -> void:
	var next := tl_year() + tl_speed
	if next >= TL_YEAR_MAX:
		tl_set_year(TL_YEAR_MAX)
		tl_playing = false
		_tl_timer.stop()
		timeline_changed.emit()
		return
	tl_set_year(next)

## The running/paused string §4.2 binds as `{{ tlState }}` and leaves
## `UNSPECIFIED`. It says which of the two states is live and, when playing, at
## what rate -- the speed pill is a set of three and only one of them is what is
## actually happening.
func tl_state_text() -> String:
	return ("playing ×%d" % tl_speed) if tl_playing else "paused"

func tl_toggle_layer(id: String) -> void:
	if not tl_layers.has(id):
		return
	tl_layers[id] = not bool(tl_layers[id])
	_tl_save_layers()
	timeline_changed.emit()

## The six toggles persist, which is the half of `BUILD_ANSWERS.md` §3 that is
## real: they record a choice, and a choice that forgot itself on restart would
## record nothing. Written straight through `ConfigFile` on
## `DccSettings.CONFIG_PATH` rather than through a `DccSettings` accessor, the
## same way `_set_coach_mark_seen()` a few hundred lines below does -- and for
## the same reason, that `dcc_settings.gd` is not this pass's file to extend.
func _tl_load_layers() -> void:
	var cfg := ConfigFile.new()
	cfg.load(DccSettings.CONFIG_PATH)
	tl_layers = {}
	for row in TL_LAYERS:
		tl_layers[row[0]] = bool(cfg.get_value("timeline", String(row[0]), row[1]))

func _tl_save_layers() -> void:
	var cfg := ConfigFile.new()
	cfg.load(DccSettings.CONFIG_PATH)
	for id in tl_layers:
		cfg.set_value("timeline", String(id), bool(tl_layers[id]))
	cfg.save(DccSettings.CONFIG_PATH)

# -- §11 Status bar -----------------------------------------------------------

func _build_status_bar() -> Control:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _scaled(DccTheme.H_STATUS)
	bar.add_theme_stylebox_override("panel", DccTheme.panel("panel_alt", {"top": 1}))
	status_row = HBoxContainer.new()
	status_row.add_theme_constant_override("separation", 18)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_child(status_row)
	bar.add_child(pad)

	## `font:10.5px 'IBM Plex Mono'; color:#6f7478` on the canvas's status bar,
	## in both themes and on both the desktop and tablet artboards -- the whole
	## bar is Plex, including the modifier hints on the right, which were the
	## one thing here already drawn a shade quieter than everything else. Set
	## in the prose face at 11 until 2026-08-25.
	for slot in ["pass", "stale", "autosave", "atlas"]:
		var l := DccTheme.mono_label("", "text_faint", DccTheme.FS_SMALL, 0)
		_status_labels[slot] = l
		status_row.add_child(l)
		## **`autosave` is registered and not drawn.** `BUILD_ANSWERS.md` §2.2
		## folds the autosave field into `statusMid`, and a bar carrying the same
		## clock twice is exactly the "four independent slots" this pass was told
		## to compose. The slot itself stays: four writers in `app.gd` address it
		## by name, and `phone_menu.gd` re-presents it as a row on a handset,
		## where there is no status bar to fold anything into.
		l.visible = slot != "autosave"
	status_row.add_child(DccTheme.spacer())
	## `statusMid` (`05-right-dock-and-bars.md` §3.3, `BUILD_ANSWERS.md` §2.2):
	## `var(--dis)`, between the spacer and the key hints. Composed in
	## `app.gd::_refresh_status_mid()` -- this file only reserves the slot, the
	## same as every other one here.
	var mid := DccTheme.mono_label("", "text_ghost", DccTheme.FS_SMALL, 0)
	_status_labels["mid"] = mid
	status_row.add_child(mid)
	var hint := DccTheme.mono_label("", "text_faint", DccTheme.FS_SMALL, 0)
	_status_labels["hint"] = hint
	status_row.add_child(hint)
	return bar

## Set one status slot. Slots: pass, stale, autosave, atlas, mid, hint, and the
## menu bar's top_world / top_pass / top_cpu / top_gpu / top_mem.
func set_status(slot: String, text: String, token: String = "text_faint") -> void:
	if not _status_labels.has(slot):
		push_error("DccShell: no status slot '%s'" % slot)
		return
	var l: Label = _status_labels[slot]
	l.text = text
	l.add_theme_color_override("font_color", DccTheme.c(token))

## Read one status slot back. `phone_menu.gd` re-presents the readout cluster
## as list rows on its root screen, because on the phone the desktop status bar
## and menu-bar readouts are parked in a hidden host rather than drawn
## (`GUI_GAP_REGISTER.md` §15 fault 2). Returns "" for a slot that has never
## been set, and for one that does not exist -- callers here are building a
## list, and an unset readout is a row that should simply not appear, not an
## error to push.
func status_slot_text(slot: String) -> String:
	if not _status_labels.has(slot):
		return ""
	return (_status_labels[slot] as Label).text

# -- §13 Phone chrome -----------------------------------------------------
#
# Phone reorganises rather than truncates: this is a distinct composition,
# not `_build_desktop_shell()` with `_scaled()` turned up further. It targets
# the same contract every workspace already depends on --
# `left_dock_body`/`right_dock_body` (workspaces attach here via
# `register_workspace()` and `right_dock.gd`), `tool_options_row`
# (`set_tool_options()`), `timeline_row`, `rail_column`/`_domain_buttons`/
# `_domain_marks` (`_select_domain()`) and `menu_bar_row`/`status_row`
# (`add_menu()`/`set_status()`) -- so nothing downstream of the frame needs to
# know which composition it is standing in. That contract is also this
# section's limit: the *content* those containers hold is reused verbatim,
# unchanged, from whatever `app.gd` and the workspaces already build for
# desktop/tablet. Only the frame around it differs.
#
# Z-order (back to front, matching draw order in `design/Cartalith DCC
# Shell.dc.html`'s "DCC shell android phone" screen):
#   1. The map, edge-to-edge, full rect -- underneath everything (inset rule
#      "DRAW EDGE-TO-EDGE, PAD BY INSET").
#   2. A gradient scrim over the top band (inset rule "SCRIM, NOT A BAR").
#   3. The chrome column: top safe area → app bar → [map gap] → tool sheet →
#      timeline → **L1 bottom bar** → bottom gesture inset. Wrapped in
#      `_phone_chrome_margin`, whose left margin is what shifts in landscape
#      to clear the side safe area. The bottom bar is the one part of this
#      column that comes from `design/Cartalith Android Phone.dc.html` rather
#      than the DCC shell canvas; it took the floating domain rail's place.
#   4. Overlays, all full-rect, all hidden until opened: the side safe area
#      (landscape only), the panel picker, the phone
#      menu (`phone_menu.gd`, L2-L5), and the left/right dock sheets.
#
# What this section does NOT build, named rather than silently skipped
# (`menus.gd`'s own discipline for what it can't honour):
#   - Any slide/drag animation. The mockup pictures exactly one static sheet
#     state; nothing here answers a drag gesture on the tool-sheet handle or
#     the gesture-inset handle -- both are decorative, matching what's shown.
#   - Touch-pan-while-drawing (v2.10's `#sculptNavpad` precedent, §13 alludes
#     to it via the sculpt tool options the phone tool sheet would host).
#     `main.gd` carries no such handling to port forward -- grepped for
#     sculpt/navpad/joystick/pan and found nothing -- so this is a genuine
#     gap for whoever wires sculpt-tool touch input, not a chrome omission.
#   - The mockup's decorative notch/punch-hole graphic (dashed box + dot,
#     lines 1452-1455 of the mockup). That reads as a mockup-authoring aid
#     showing where a *real* device's hardware cutout sits, not something a
#     shipped app should paint a fake copy of over an arbitrary point on an
#     arbitrary screen. The 108 px keep-clear reserve is honoured -- nothing
#     is ever placed there -- just not decorated.

func _build_phone_shell() -> void:
	_phone_root = Control.new()
	_phone_root.set_anchors_preset(Control.PRESET_FULL_RECT)
	_phone_root.mouse_filter = Control.MOUSE_FILTER_PASS
	add_child(_phone_root)

	var vp := _build_viewport()
	vp.set_anchors_preset(Control.PRESET_FULL_RECT)
	_phone_root.add_child(vp)

	## No gradient scrim any more: the 412 canvas paints a **solid ground** above
	## the map (`background:#101112` on the screen, the app bar carrying only a
	## `border-bottom`), not a fade. The status row builds its own opaque panel;
	## see `_build_phone_top_safe()`.

	## Menu bar and status bar keep their exact desktop construction --
	## `add_menu()`/`set_status()` stay phone-unaware -- and are parked in a
	## hidden host. They are the **model**, not a view: `menu_bar_row` is where
	## `add_menu()` puts the seven real `MenuButton`s and `_status_labels` is
	## where `set_status()` writes, and `phone_menu.gd` reads both. Nothing here
	## is drawn.
	##
	## §15 fault 2 was that these two bars were previously *reparented into the
	## phone sheet whole* -- a 150 px desktop wordmark and five readouts that are
	## empty before a generation, squeezing the menu row into a bottom strip.
	## Parking them is the fix; the readouts come back as real rows on the
	## menu's own root screen.
	##
	## Still built *first*, for the reason it always was: both this and the app
	## bar register a Label under the "top_world" status slot, `_status_labels`
	## keeps only the most recently registered one, and the app bar's subtitle
	## is the one that must win. That ordering is load-bearing.
	var model_host := Control.new()
	model_host.name = "PhoneMenuModel"
	model_host.visible = false
	model_host.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_phone_root.add_child(model_host)
	model_host.add_child(_build_menu_bar())
	model_host.add_child(_build_status_bar())

	## Both of these cover the whole screen, and a `Control` picks by default
	## (`MOUSE_FILTER_STOP`) -- so as pure layout scaffolding they were each,
	## on their own, enough to keep every tap off the map underneath. See
	## `_phone_content_gap`'s own comment below for the full diagnosis; these
	## two are the same bug one and two levels up, and all three had to go for
	## `map_overlay.gd` to see a finger.
	##
	## Their children (the app bar, the tool sheet, the bottom menu bar) are
	## picked independently of their parent's filter, so nothing tappable is
	## lost by taking the containers themselves out of picking.
	_phone_chrome_margin = MarginContainer.new()
	_phone_chrome_margin.set_anchors_preset(Control.PRESET_FULL_RECT)
	_phone_chrome_margin.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_phone_root.add_child(_phone_chrome_margin)

	var chrome := VBoxContainer.new()
	chrome.add_theme_constant_override("separation", 0)
	chrome.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_phone_chrome_margin.add_child(chrome)
	## Held because landscape lifts two of this column's children out of it --
	## the bottom bar becomes a left rail and the tool sheet docks right -- and
	## portrait has to put them back in the right slots. See
	## `_apply_phone_orientation()`.
	_phone_chrome_col = chrome

	_phone_top_safe = _build_phone_top_safe()
	chrome.add_child(_phone_top_safe)

	_phone_app_bar = _build_phone_app_bar()
	chrome.add_child(_phone_app_bar)

	## The gap between the app bar and the tool sheet: nothing but map. The
	## floating domain rail used to sit in it; the canvas moved the domains to
	## the bottom bar, so the map now has the whole width back.
	_phone_content_gap = Control.new()
	_phone_content_gap.size_flags_vertical = Control.SIZE_EXPAND_FILL
	## **`IGNORE`, not `PASS`.** This was `PASS`, and `PASS` does not mean what
	## it reads like: a `PASS` control is still *picked* -- it receives the
	## event and then forwards it to its own **parent**, never to the nodes
	## behind it. So this spacer, which expands to fill the entire region
	## between the app bar and the tool sheet, was the control under every tap
	## on the map, and `map_overlay.gd`'s `_gui_input()` had never once run on a
	## phone. Only `IGNORE` takes a control out of picking so what is behind it
	## is reached.
	##
	## Everything on `map_overlay` that a finger should be able to do was dead
	## because of this one enum: tap-to-select a settlement, every registered
	## tool click/drag/release handler (Settlement, Territory, Way, Route,
	## Measure, and Sculpt/Paint dabs), and the press-and-hold that opens the
	## civ context sheet. It looked like the map worked because *camera* pan and
	## pinch come through `ViewportHost._input()`, which is a raw input hook and
	## never consults a `mouse_filter` at all -- so the half that was broken was
	## exactly the half nobody had driven on the device.
	##
	## Found with `gui_get_hovered_control()` over the map centre in a
	## `--force-touch` run: it named this node, not the overlay.
	##
	## Children are picked on their own, so the floating rail this hosts is
	## unaffected -- `IGNORE` on a parent does not disable its children.
	_phone_content_gap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	chrome.add_child(_phone_content_gap)

	_phone_tool_sheet = _build_phone_tool_sheet()
	chrome.add_child(_phone_tool_sheet)

	timeline_bar = _build_timeline()
	chrome.add_child(timeline_bar)

	_phone_menu_bar = _build_phone_menu_bar()
	chrome.add_child(_phone_menu_bar)

	_phone_gesture_inset = _build_phone_gesture_inset()
	chrome.add_child(_phone_gesture_inset)

	_phone_side_safe = _build_phone_side_safe()
	_phone_root.add_child(_phone_side_safe)

	_phone_panel_picker = _build_phone_panel_picker()
	_phone_root.add_child(_phone_panel_picker)

	## `⌕`'s destination. Built and added unconditionally -- unlike the app
	## bar's cell, which only draws when `_has_place_search()` is true, this
	## costs nothing sitting hidden and `open_find_on_map()` (called from
	## `menus.gd` on desktop, and reachable here if a future build ever wires
	## a phone route to it that isn't the app-bar cell) needs somewhere to
	## open even if the bar button that normally reaches it is absent.
	_phone_search_overlay = _build_phone_search_overlay()
	_phone_root.add_child(_phone_search_overlay)

	## L2-L5. Added after the panel picker so it draws over it, and
	## before the dock sheets so a dock sheet still wins -- the same
	## mutually-exclusive rule `_close_all_phone_overlays()` enforces anyway.
	_phone_menu = PhoneMenu.new()
	_phone_root.add_child(_phone_menu)
	_phone_menu.setup(self)

	## Full-height sheets (§13), built by the exact same functions the
	## desktop/tablet dock uses -- `as_sheet = true` only swaps the header's
	## collapse chevron for a close button and drops the fixed desktop width.
	## `left_dock_body`/`right_dock_body` (what every workspace and
	## `right_dock.gd` actually attach content to) are unaffected either way.
	_phone_root.add_child(_build_left_dock(true))
	left_dock.set_anchors_preset(Control.PRESET_FULL_RECT)
	left_dock.visible = false
	_phone_root.add_child(_build_right_dock(true))
	right_dock.set_anchors_preset(Control.PRESET_FULL_RECT)
	right_dock.visible = false

	## Connected only once both docks exist, so `_on_phone_node_added()` never
	## walks toward a null. Every workspace panel is attached after this point,
	## which is exactly what it is here to catch.
	get_tree().node_added.connect(_on_phone_node_added)

	_apply_phone_orientation()

	## The floating undo chip. Built into `_phone_content_gap` -- "the visible
	## gap between the app bar and the tool sheet" (that field's own comment,
	## a few hundred lines up), which already excludes the bottom nav bar AND
	## the tool sheet by construction: it is the container `_phone_nav_reserve()`
	## and `_phone_bottom_reserve()` exist to carve out, not a rect this code
	## has to carve out a second time. Checked and rejected: anchoring the chip
	## straight to `_phone_root` at `size.y - _phone_nav_reserve() - margin`
	## looked like the more literal reading of "clear of the bottom nav
	## (`_phone_nav_reserve()`)", but `_phone_nav_reserve()` only accounts for
	## the gesture inset and the bottom bar -- NOT the tool sheet, which sits
	## between them and the map and is taller than both at every detent past
	## `peek`. A chip placed by that arithmetic would sit *under* the sheet,
	## not above it. `_phone_content_gap` is the strictly safer bound, and
	## bottom-LEFT keeps it clear of `ViewportHost`'s own navpad, which floats
	## in the same vertical band on the right (`viewport_host.gd`
	## `_apply_safe_insets()`: the navpad also stacks up from just above the
	## tool sheet, at `right: NAVPAD_EDGE`).
	_phone_undo_chip = _build_phone_undo_chip()
	_phone_content_gap.add_child(_phone_undo_chip)
	## §6.2's edit-history popover, in the same container and anchored off the
	## same corner, so the two move together whatever the tool sheet is doing.
	_phone_undo_pop = _build_phone_undo_popover()
	_phone_content_gap.add_child(_phone_undo_pop)
	_wire_phone_undo_chip()

	## §6.2's sim strip. In `_phone_content_gap` for the undo chip's reason --
	## it is the container that already excludes the bottom nav and the tool
	## sheet by construction -- and hidden until the timeline strip's own row
	## opens it.
	_phone_sim_strip = _build_phone_sim_strip()
	_phone_content_gap.add_child(_phone_sim_strip)
	timeline_changed.connect(_refresh_phone_sim_strip)

	## §4.3's `⋮` popover. On `_phone_root` rather than in the chrome column:
	## it is anchored to the screen (`right:10px; top:86px`), and it carries a
	## scrim that has to cover the map.
	_phone_overflow_pop = _build_phone_overflow()
	_phone_root.add_child(_phone_overflow_pop)
	_wire_phone_overflow()

	_maybe_show_coach_marks()

## The 412 canvas's status row, verbatim: `height:28px;padding:0 16px;
## font:10px 'IBM Plex Mono';color:#8d9296`, clock left, `LTE ▮▮ 84%` right at
## `letter-spacing:.14em`, and a **solid** ground rather than §13's gradient
## scrim over the map.
##
## What this used to be was §13's 44 dp *keep-clear reserve* with a 96 dp
## gradient behind it and a 108 dp centre lane nothing was allowed into. The
## newer canvas draws none of the three: 28 dp, edge to edge, opaque. The lane
## survives only in landscape, where no canvas exists -- see `W_PHONE_CUTOUT`.
func _build_phone_top_safe() -> Control:
	var ground := PanelContainer.new()
	ground.add_theme_stylebox_override("panel", DccTheme.panel("panel"))
	ground.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _pscale(16))
	pad.add_theme_constant_override("margin_right", _pscale(16))
	pad.mouse_filter = Control.MOUSE_FILTER_IGNORE
	ground.add_child(pad)
	var row := HBoxContainer.new()
	row.custom_minimum_size.y = _safe_top()
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	pad.add_child(row)

	## Both spans are `#8d9296` in the canvas -- the right one was `text_faint`
	## here, one ink step quiet, and both were 11 px against the canvas's 10.
	_phone_clock_label = DccTheme.mono_label("", "text_dim", _pfont(10))
	_phone_clock_label.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(_phone_clock_label)
	row.add_child(DccTheme.spacer())
	_phone_battery_label = DccTheme.mono_label("", "text_dim", _pfont(10), 1)
	_phone_battery_label.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(_phone_battery_label)

	var timer := Timer.new()
	timer.wait_time = 30.0
	timer.autostart = true
	timer.timeout.connect(_refresh_phone_status_glyphs)
	ground.add_child(timer)
	_refresh_phone_status_glyphs()
	return ground

## The clock is the real system time (`Time`, not the mockup's static "9:41").
## Battery/signal/Wi-Fi stay the mockup's own decorative placeholder glyphs --
## checked against this Godot build's own `OS` class (`ClassDB.class_get_method_list`)
## rather than assumed: there is no `power`/`battery` method on it at all, so
## there is nothing real to back these three with cross-platform. Only the
## clock gets the honest-data treatment.
func _refresh_phone_status_glyphs() -> void:
	var t := Time.get_time_dict_from_system()
	var clock_text := "%02d:%02d" % [int(t["hour"]), int(t["minute"])]
	var battery_text := "▲ ▮▮ --"
	if _phone_clock_label != null:
		_phone_clock_label.text = clock_text
	if _phone_battery_label != null:
		_phone_battery_label.text = battery_text
	if _phone_side_clock_label != null:
		_phone_side_clock_label.text = clock_text
	if _phone_side_battery_label != null:
		_phone_side_battery_label.text = battery_text

## Landscape's side safe area (inset rule "LANDSCAPE": "the cutout moves to a
## side edge: apply the same reserve horizontally"). Judgment call, undocumented
## by the mockup (it only pictures the portrait screen): the cutout is placed
## on the *left* edge, and the portrait top row's "left/right pockets" become
## this column's "top/bottom pockets" -- the same pocket structure, rotated,
## rather than a different rule invented for landscape. The rail "shifts
## inward" (same inset rule) for free: it floats inside `_phone_chrome_margin`,
## whose left margin grows by this column's width in `_apply_phone_orientation()`.
func _build_phone_side_safe() -> Control:
	var wrap := Control.new()
	wrap.set_anchors_preset(Control.PRESET_LEFT_WIDE)
	wrap.offset_left = 0
	wrap.offset_right = _safe_side()
	wrap.offset_top = 0
	wrap.offset_bottom = 0
	wrap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	wrap.visible = false

	var bg := ColorRect.new()
	bg.color = Color(DccTheme.c("bg"), 0.9)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
	wrap.add_child(bg)

	var col := VBoxContainer.new()
	col.set_anchors_preset(Control.PRESET_FULL_RECT)
	col.add_theme_constant_override("separation", 0)
	wrap.add_child(col)

	var top_pad := MarginContainer.new()
	top_pad.add_theme_constant_override("margin_top", _pscale(10))
	_phone_side_clock_label = DccTheme.mono_label("", "text_dim", _pfont(10))
	_phone_side_clock_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	top_pad.add_child(_phone_side_clock_label)
	col.add_child(top_pad)
	col.add_child(DccTheme.spacer())
	## The keep-clear reserve -- see the "notch graphic" note in the section
	## header comment above: nothing is placed here, deliberately undecorated.
	var dead := Control.new()
	dead.custom_minimum_size.y = _pscale(DccTheme.W_PHONE_CUTOUT)
	col.add_child(dead)
	col.add_child(DccTheme.spacer())
	var bot_pad := MarginContainer.new()
	bot_pad.add_theme_constant_override("margin_bottom", _pscale(10))
	_phone_side_battery_label = DccTheme.mono_label("", "text_faint", _pfont(9))
	_phone_side_battery_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	bot_pad.add_child(_phone_side_battery_label)
	col.add_child(bot_pad)
	return wrap

## The app bar. `design/Cartalith Android Phone.dc.html` screen 01:
## `height:56px;display:flex;align-items:center;gap:14px;padding:0 12px;
## border-bottom:1px solid rgba(255,255,255,.09)`, carrying `☰` (16 px) / title
## over seed / `⌕` / `⋮` in 40 dp cells.
##
## One of the canvas's four cells is still not built, for a stated reason, and
## a second is now live where this comment used to say it could not be:
##
##   - **`⋮` is built now.** It used to be declined here, on the reasoning that
##     the 2026-08-30 canvas drew it as a *contextual* overflow with nothing
##     per-screen to put behind it. **That reason expired on 2026-08-31**, when
##     `design/dcc-environment-2026-08-31/Cartalith Android.dc.html` defined it
##     exactly: `hMenu` (`:89-95`, `:897`) opens a 230 dp popover carrying
##     `Save project` + `savedAt`, `Theme` + `themeLabel`, and `Close world`.
##     All three destinations already exist in this shell, and `CLAUDE.md`'s
##     "the newer canvas wins" settles which drawing to build. It is *not* a
##     duplicate of the MORE tab: MORE is the program-menu tree, this is three
##     document-level actions, which is the split the canvas itself draws by
##     giving the phone both.
##   - **`⌕` is built now.** This comment used to say it had "no destination"
##     because `menus.gd`'s Edit ▸ Find on map… was a disabled `_todo()` row
##     reasoning "no search index yet" -- true when it was written and false
##     the moment `shell/place_search.gd` landed, which left the sentence
##     exactly as stale as the ones this session has been correcting all week
##     elsewhere in this file. `open_find_on_map()` below is the real
##     destination now, on both the phone (a full-width overlay, built here)
##     and the desktop (`menus.gd`'s row calls `_host.open_find_on_map()`,
##     which this file answers with an `AcceptDialog`). Drawn, not text: `⌕`
##     (U+2315) is the one glyph `dcc_icons.gd`'s own header names as missing
##     from Plex Mono's fallback chain, the same reason that file drew a
##     `PATHS["search"]` icon instead of listing it in `SYMBOLS` -- so this
##     cell is the one `_phone_bar_button()` call in this bar passing
##     `icon_name` instead of a `SYMBOLS` glyph string. **Guarded**, the same
##     way the cell disappears if `place_search.gd` is missing: no affordance
##     with nothing behind it, in either direction.
##   - **`▤`** was here and is the bottom bar's PANELS tab now.
func _build_phone_app_bar() -> PanelContainer:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _ptap(DccTheme.H_PHONE_APP_BAR)
	bar.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"bottom": 1}))
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", _pscale(14))
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _pscale(12))
	pad.add_theme_constant_override("margin_right", _pscale(12))
	pad.add_child(row)
	bar.add_child(pad)

	## The 412 canvas has **no side drawer**: its `02 Domain` screen is a
	## full-screen drill with a `←`, and the shell's own full-height left dock
	## sheet is that screen. So `☰` opens the sheet directly, and the 300 dp side
	## sheet that used to list the three domains with their subtitles is gone --
	## the bottom bar's three domain cells are the same three destinations, now
	## with a glyph each, and carrying them twice was the duplication the canvas
	## rules out.
	row.add_child(_phone_bar_button(DccIcons.SYMBOLS["drawer"], "Domain panel",
		func(): _set_sheet_open("left", true)))

	var title_col := VBoxContainer.new()
	title_col.add_theme_constant_override("separation", 0)
	title_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	title_col.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	## `font:500 12px 'IBM Plex Mono';letter-spacing:.2em;color:#e8ebec` -- .2em
	## of 12 px is 2.4 px, and `spacing_glyph` is whole pixels, so 2. This was 3.
	title_col.add_child(DccTheme.mono_label("CARTALITH", "text_bright", _pfont(12), 2, true))
	## Reuses the same "top_world" status slot the desktop menu bar's readout
	## cluster fills (`_wire_status()` in `app.gd` calls
	## `set_status("top_world", "ELDRA · %d" % seed)`) -- no phone-aware
	## branch needed in `app.gd` for this to stay live. `font:10px 'IBM Plex
	## Mono';color:#6f7478`, untracked in the canvas.
	var subtitle := DccTheme.mono_label("", "text_faint", _pfont(10), 0)
	_status_labels["top_world"] = subtitle
	title_col.add_child(subtitle)
	row.add_child(title_col)

	## `⌕`. Only if the index it opens actually exists -- see this function's
	## own header and `_has_place_search()`. `place_search.gd` is a parallel,
	## concurrently-landing file; a build that races ahead of it must not draw
	## a magnifier over nothing, so this checks fresh on every call rather than
	## caching the answer from boot.
	if _has_place_search():
		row.add_child(_phone_bar_button("", "Search", func(): open_find_on_map(),
			"text", "search"))

	## **`▤` panels, restoring access the four-tab bar took away.**
	## `_phone_panel_picker` is how the phone reaches the left and right dock
	## content, and its ONLY entry used to be the PANELS cell in the bottom
	## bar. Replacing that bar with MAP/GENERATE/PLAN/MORE left the picker
	## built, alive and unreachable -- built-and-unwired, the defect class this
	## repository keeps finding, introduced by me this session and caught on the
	## device rather than by reading.
	##
	## `DCC_SHELL_SPEC.md` §13 puts it here anyway: the phone app bar is
	## "☰ (domain drawer), title + seed, ▤ (panels), ⋯ (overflow menu)". So the
	## fix and the spec agree.
	row.add_child(_phone_bar_button(DccIcons.SYMBOLS["panels"], "Panels",
		func(): _set_panel_picker_open(true)))

	## `⋮`. See this function's own header for why it is here now and was not
	## before. `overflow` is `⋯` in `DccIcons.SYMBOLS` -- the horizontal
	## ellipsis the *bottom bar's* MORE cell traces -- so the vertical one the
	## app bar draws is a literal, the same way `GLYPH_THEME` is.
	row.add_child(_phone_bar_button(GLYPH_OVERFLOW, "More actions",
		func(): _set_phone_overflow_open(true)))
	return bar

## `⋮` U+22EE. Not in `DccIcons.SYMBOLS`, which carries `⋯` (`overflow`) for
## the bottom bar's MORE cell; the two are different marks on the same canvas
## and the table is not this file's to extend. Resolves through `mono()`'s
## `SystemFont` fallback like every other entry that Plex Mono has no glyph for.
const GLYPH_OVERFLOW := "\u22ee"

## One app-bar glyph cell. The canvas draws a `40x40` box at `color:#c8cbcd`
## (`text`, not the `text_dim` this used) with `font:16px 'IBM Plex Mono'`; the
## box is a *layout* cell with no background, so the hit target still floors at
## the TARGETS card's 44 dp rather than shrinking to the drawn 40.
##
## **Not `flat`.** A `Button` with `flat = true` skips its `normal`/`hover`/
## `pressed` styleboxes outright, so the press feedback on the last two lines
## had never once appeared -- the fourth site of the trap `GUI_GAP_REGISTER.md`
## MN-13 found in three others, and the one on the phone's most-tapped control.
##
## `icon_name`, added alongside `⌕`: `glyph` is drawn as `b.text`, which only
## works for a `SYMBOLS` entry -- a real character some font in the chain can
## shape. `⌕` (U+2315) is not one (`dcc_icons.gd`'s own header, the `search`/
## `import` note), so it is drawn instead, the same as the bottom nav's own
## glyphs (`DccIcons.rect()`, a few hundred lines below this one). A child
## `TextureRect` rather than `Button.icon` + `icon_*_color` theme overrides:
## `DccIcons.rect()` already tints to a token and centres itself via anchors
## the way `_phone_list_row()`'s `rpad` does against a non-`Container` parent
## (that function's own comment), so this reuses exactly that rather than
## adding a second glyph-tinting mechanism next to it. `glyph` is ignored
## when `icon_name` is set; callers pass `""` for it, same as the `⌕` call
## site above does.
func _phone_bar_button(glyph: String, tip: String, on_press: Callable,
		token: String = "text", icon_name: String = "") -> Button:
	var b := Button.new()
	b.flat = false
	b.focus_mode = Control.FOCUS_NONE
	## **`accessibility_name`, not just a tooltip.** Godot raises a tooltip on
	## hover, and a handset has no hover -- so on the one composition where these
	## four cells (`☰`, `⌕`, `▤`, `⋮`) are the app's whole top bar, `tip` reached
	## nobody at all, and to a screen reader the button was a bare glyph or, for
	## `⌕`, a `TextureRect` child with no text of any kind. `Control` carries
	## `accessibility_name` in this Godot 4.7.1 build (checked against
	## `ClassDB.class_get_property_list("Control")`, which lists it beside
	## `accessibility_description` and the five `*_nodes` relations), and it is
	## the one channel that does reach a touch user. Same string: the tooltip is
	## already the control's name and duplicating it here would let the two drift.
	b.tooltip_text = tip
	b.accessibility_name = tip
	b.custom_minimum_size = Vector2(_ptap(DccTheme.PHONE_ICON_BOX),
		_ptap(DccTheme.PHONE_ICON_BOX))
	b.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	if icon_name != "":
		var ic := DccIcons.rect(icon_name, _pscale(16), token)
		ic.set_anchors_preset(Control.PRESET_CENTER)
		b.add_child(ic)
	else:
		b.text = glyph
	b.add_theme_font_size_override("font_size", _pfont(16))
	b.add_theme_font_override("font", DccTheme.mono())
	b.add_theme_color_override("font_color", DccTheme.c(token))
	b.add_theme_stylebox_override("normal", DccTheme.empty())
	b.add_theme_stylebox_override("focus", DccTheme.empty())
	b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	b.add_theme_stylebox_override("pressed", DccTheme.active_row(false))
	b.pressed.connect(on_press)
	return b

## L1: the bottom bar (`design/Cartalith Android Phone.dc.html`, artboard
## "01 · VIEWPORT" -- five equal cells, 64 dp, glyph over a 9.5 px tracked
## caption, active cell in accent).
##
## **This replaces the floating left rail.** The canvas draws no rail: it moves
## the domains to the bottom, where a thumb reaches them, and its PHONE RULES
## make the bar level 1 of the disclosure tree outright ("L1 is the bottom
## bar"). Keeping both would have put the same three domains on screen twice.
## ## The one place the two authorities split
##
## The canvas's five tabs are `WORLD · GENERATE · SIMULATE · MAP · MORE` -- the
## **pre-v3** domain set. `design/Cartalith Menu Structure v3.dc.html` is newer
## and is the authority for domain content and naming, and it has three:
## `WORLD · CIVIL · CARTO` (INFRA merged into CIVIL, RENDER into CARTO, owner
## 2026-08-20). `DCC_SHELL_SCOPE.md`'s rule 1 -- "the newer canvas wins" --
## resolves it: this bar takes **412's geometry and v3's content**. Five slots,
## v3's three domains plus the two phone affordances the canvas's own fifth tab
## and app bar establish: PANELS (both docks) and MORE (the program menu tree).
##
## `MENU` was the fifth caption and is `MORE` now, which is the canvas's word
## for that exact destination.
##
## ## The glyph row
##
## `<span style="font:14px Plex">◈</span>` over `<span style="font:9.5px Plex;
## letter-spacing:.1em">WORLD</span>`, `gap:4px`, active `#e0a34a` and resting
## `#8d9296`. The row was captions only until this pass.
##
## The five marks are **drawn, not typed**, and three of them already existed:
## `DccIcons`' `domain_world`/`domain_civ`/`domain_carto` are this design
## system's own glyphs for these exact three subjects, authored to §12's rules.
## `nav_panels` and `nav_more` are new and are *designed rather than matched*
## under `DCC_SHELL_SCOPE.md`'s rule 2 -- each traces the canvas's own chosen
## symbol (▤, ⋯) at §12's stroke.
##
## This does **not** re-open the owner's *"those icons don't exist"* ruling. That
## was about the **desktop vertical rail**, whose artboard draws `writing-mode:
## vertical-rl` captions and no icon element at all (see `_build_rail()`). This
## artboard draws a glyph over every caption, explicitly, five times.
##
## `rail_column` stays the container the domain cells sit in, so
## `set_rail_foot()`/`_select_domain()` and anything else that already knows
## that name keeps working. What `Window ▸ Domain rail` hides is `_rail_region`,
## which here is **only the three destination cells** -- MAP, GENERATE and PLAN.
## MORE stays, because the menu is the only place the row that un-hides the rail
## lives: hide MORE with the rest and the row becomes a one-way door.
##
## That was the rule all along, and the bar broke it silently. This paragraph
## used to read "PANELS and MENU stay", naming the pre-`PHONE_TABS` five-cell
## bar, while the loop below added every cell it had -- MORE included -- under
## `_rail_region`. Unchecking the row therefore built exactly the door the rule
## forbids, and left `_phone_menu_bar.visible` true behind it, so
## `_phone_nav_reserve()` went on reserving 64 dp for an empty strip. Fixed by
## giving the destination tabs a box of their own (`_phone_bar_dests`) and
## leaving MORE outside it -- so the bar the reserve pays for is never empty.
func _build_phone_menu_bar() -> Control:
	var bar := PanelContainer.new()
	bar.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"top": 1}))

	## **`BoxContainer`, not `HBoxContainer`** -- for all four of the boxes this
	## bar nests, and for no other reason than landscape. `HBoxContainer` is not
	## merely a `BoxContainer` with `vertical = false` preset: its `set_vertical`
	## *refuses* the write outright (`Can't change orientation of
	## HBoxContainer`, `scene/gui/box_container.cpp`). Assuming otherwise is
	## exactly what the first run of `_apply_phone_nav_orientation()` did, three
	## errors per rotation and a rail that stayed horizontal. A bare
	## `BoxContainer` starts horizontal and lets the flip through.
	var row := BoxContainer.new()
	row.vertical = false
	row.add_theme_constant_override("separation", 0)
	bar.add_child(row)

	var domains := BoxContainer.new()
	domains.vertical = false
	domains.add_theme_constant_override("separation", 0)
	domains.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	domains.size_flags_stretch_ratio = float(PHONE_TABS.size())
	row.add_child(domains)

	rail_column = VBoxContainer.new()
	rail_column.add_theme_constant_override("separation", 0)
	rail_column.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	domains.add_child(rail_column)

	var cells := BoxContainer.new()   ## See `row` above for why not `HBox`.
	cells.vertical = false
	cells.add_theme_constant_override("separation", 0)
	rail_column.add_child(cells)

	## The three destination tabs and only those -- see this function's header
	## for why MORE may not be in here. Stretch ratios keep all four cells the
	## same size: three shares for this box against MORE's own one, so nothing
	## about the bar's geometry changes, only what `Window ▸ Domain rail` reaches.
	var dests := BoxContainer.new()   ## See `row` above for why not `HBox`.
	dests.vertical = false
	dests.add_theme_constant_override("separation", 0)
	dests.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	dests.size_flags_stretch_ratio = float(PHONE_TABS.size() - 1)
	cells.add_child(dests)
	_rail_region = dests

	## Landscape turns this bar on its side (`docs/ANDROID_UI_SPEC.md`: "nav
	## becomes left rail"). Every one of these four is an `HBoxContainer`,
	## which in Godot 4 is a `BoxContainer` with `vertical = false` -- so the
	## rotation is a property flip on the *same nodes*, not a second bar built
	## alongside this one. Held here because `_apply_phone_orientation()` has no
	## other route to them: `rail_column` is a documented public-ish name that
	## `set_rail_foot()`/`_select_domain()` already rely on, and the other four
	## were locals.
	_phone_bar_row = row
	_phone_bar_domains = domains
	_phone_bar_cells = cells
	_phone_bar_dests = dests

	## **`PHONE_TABS`, not `DOMAINS`.** The bar used to mirror the desktop's
	## three workspaces plus PANELS and MORE, which is how a phone ended up
	## with two tabs -- "PANELS" and "MORE" -- that name a container rather
	## than a destination. `docs/ANDROID_UI_SPEC.md` replaces them with four
	## task tabs, and CIVIL moves under MORE, which that spec states
	## explicitly ("MORE: Project, Civilization ..., Data manager, ...").
	for t in PHONE_TABS:
		var cell := _phone_bar_cell(String(t.caption), String(t.icon), String(t.tip),
			_pick_phone_tab.bind(String(t.id)))
		var key := String(t.id)
		_phone_tab_cells[key] = cell
		if String(t.domain) != "":
			_domain_buttons[String(t.domain)] = cell["button"]
			_domain_marks[String(t.domain)] = {"label": cell["label"], "icon": cell["icon"],
				"off": "text_dim", "box": false}
		## MORE is `cells`' own second child; every other tab goes in the box
		## `Window ▸ Domain rail` hides.
		(cells if key == "more" else dests).add_child(cell["button"] as Control)
	return bar

## The phone's four task tabs (`docs/ANDROID_UI_SPEC.md`: "bottom bar, task tabs
## MAP · GENERATE · PLAN · MORE").
##
## `domain` is the desktop workspace a tab selects, or `""` for a tab that is
## not a workspace at all. Two of the four map straight onto existing domains;
## PLAN opens the journey planner, and MORE is the overflow screen.
##
## `civilization` is deliberately absent as a *tab* and still fully reachable --
## the planner selects it, and MORE lists it. The spec moved it there rather
## than dropping it.
const PHONE_TABS: Array = [
	{"id": "map", "caption": "MAP", "icon": "domain_carto", "domain": "cartography",
		"tip": "Layers, style and annotation"},
	{"id": "gen", "caption": "GENERATE", "icon": "domain_world", "domain": "world",
		"tip": "The generation pipeline, and Sculpt"},
	{"id": "plan", "caption": "PLAN", "icon": "tool_route", "domain": "",
		"tip": "Journey planner"},
	{"id": "more", "caption": "MORE", "icon": "nav_more", "domain": "",
		"tip": "Project, Civilization, Data, Assets, Preferences, Help"},
]

var _phone_tab_cells: Dictionary = {}
var _phone_tab := "gen"   ## Which of `PHONE_TABS` is lit.

## One tab press. A workspace tab selects its domain; the two that are not
## workspaces do their own thing.
func _pick_phone_tab(id: String) -> void:
	var was_tab := _phone_tab
	_phone_tab = id
	match id:
		"more":
			_toggle_overflow()
		"plan":
			if _phone_menu != null and _phone_menu.is_open():
				_phone_menu.close()
			## `DccApp extends DccShell`, so `self` is the app; the planner
			## opener lives on the subclass. Guarded rather than assumed,
			## because `DccShell` is also instantiated bare by probes.
			if has_method("open_journey_planner"):
				call("open_journey_planner")
		_:
			if _phone_menu != null and _phone_menu.is_open():
				_phone_menu.close()
			for t in PHONE_TABS:
				if String(t.id) == id and String(t.domain) != "":
					_pick_bar_domain(String(t.domain))
	## "tab tap opens half" (`docs/ANDROID_UI_SPEC.md`). The prototype's `hTab`
	## is `{tab:t, detent: s.detent==='peek' ? 'half' : s.detent}` -- a tap
	## *lifts* a peeking sheet to half and leaves half and full where the user
	## put them, so switching tabs never shrinks a sheet someone has just pulled
	## open. Re-tapping the lit tab closes it there; here it collapses to peek
	## instead, for the reason `_on_phone_sheet_grab_input()` sets out at
	## length: this sheet is the tool options bar, and it has no "gone" state on
	## the other two form factors.
	##
	## Only the two *workspace* tabs drive the detent. In the prototype all four
	## tabs fill the one sheet, so all four move it; here PLAN opens the journey
	## planner and MORE opens `PhoneMenu`, both full overlays over the sheet
	## rather than content inside it. Lifting a peeking sheet behind an overlay
	## the user cannot see through would only surface after they closed it
	## again, as a sheet that had grown while they were elsewhere.
	if _phone_tab_drives_sheet(id):
		if was_tab == id and _phone_detent != "peek":
			_set_phone_detent("peek")
		elif _phone_detent == "peek":
			_set_phone_detent("half")
	_refresh_phone_tabs()

## Which of `PHONE_TABS` a domain lights. A domain with no tab of its own lives
## under MORE and lights MORE -- except when the tab already lit is itself not a
## workspace, which means THAT tab is what opened this domain (PLAN selects
## Civilization) and re-pointing the bar at MORE would name a screen the user is
## not on.
func _phone_tab_for_domain(id: String) -> String:
	for t in PHONE_TABS:
		if String(t.domain) == id:
			return String(t.id)
	return "more" if _phone_tab_drives_sheet(_phone_tab) else _phone_tab

## Whether a tab's destination is the tool sheet itself. `domain` is `""` for
## exactly the two tabs that open an overlay instead -- see `PHONE_TABS`.
func _phone_tab_drives_sheet(id: String) -> bool:
	for t in PHONE_TABS:
		if String(t.id) == id:
			return String(t.domain) != ""
	return false

## The active tab wears the candidate's own pill -- `padding:5px 16px;
## border-radius:14px; background:rgba(224,163,74,.16)` behind the glyph
## (`candidates/Android Chrome B.dc.html`). Lighting only the caption, which is
## what the old bar did, left the row reading as four labels of equal weight.
func _refresh_phone_tabs() -> void:
	for key in _phone_tab_cells.keys():
		var cell: Dictionary = _phone_tab_cells[key]
		var on: bool = (String(key) == _phone_tab)
		var lbl: Label = cell.get("label")
		var pill: PanelContainer = cell.get("pill")
		if lbl != null and is_instance_valid(lbl):
			(lbl as Label).add_theme_color_override("font_color",
				DccTheme.c("accent" if on else "text_dim"))
		if pill != null and is_instance_valid(pill):
			## `rgba(224,163,74,.16)` verbatim from the candidate, NOT the
			## `accent_wash` token. That token is 8% (`#e0a34a14`), which is
			## the desktop's active-menu wash and is effectively invisible
			## behind a 14 px glyph on `#121314` -- checked on the handset, the
			## pill did not read at all at 8%. The candidate chose twice that
			## for this surface and it is right for it.
			var box := DccTheme.flat(Color(DccTheme.c("accent"), 0.16))
			box.set_corner_radius_all(_pscale(14))
			(pill as PanelContainer).add_theme_stylebox_override("panel",
				box if on else DccTheme.empty())



## One bar cell: a `14px` glyph over a `9.5px/.1em` caption with `gap:4px`,
## centred in a 64 dp cell. Returns all three nodes because `_select_domain()`
## recolours the caption *and* the glyph, and only the domain cells register
## there.
func _phone_bar_cell(caption: String, glyph: String, tip: String,
		on_press: Callable) -> Dictionary:
	var b := Button.new()
	b.tooltip_text = tip
	## The caption below is a child `Label`, not `b.text`, so this `Button`
	## carries no accessible name of its own -- see `_phone_bar_button()`'s note.
	## `tip` rather than `caption`: the caption is a five-letter tracked word
	## ("WORLD"), the tip is the sentence that says what tapping it does.
	b.accessibility_name = tip
	## Not flat: a flat `Button` draws no stylebox, so both boxes below would be
	## comments. Same trap as `add_menu()`, `_build_rail()` and
	## `_phone_list_row()` -- see `GUI_GAP_REGISTER.md` MN-13.
	b.flat = false
	b.focus_mode = Control.FOCUS_NONE
	## Canvas: a 64 dp bar. `_ptap` floors it at the 44 px minimum and scales it
	## with everything else, so this is the same target arithmetic the app bar's
	## own buttons use -- not a second set of numbers.
	b.custom_minimum_size.y = _ptap(DccTheme.H_PHONE_BOTTOM_NAV)
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.add_theme_stylebox_override("normal", DccTheme.empty())
	b.add_theme_stylebox_override("focus", DccTheme.empty())
	b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	b.add_theme_stylebox_override("pressed", DccTheme.active_row(false))
	b.pressed.connect(on_press)

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", _pscale(4))
	col.alignment = BoxContainer.ALIGNMENT_CENTER
	col.mouse_filter = Control.MOUSE_FILTER_IGNORE

	## `_pscale`d, not the raw 14: the main viewport has no content scale, so a
	## 14 px glyph would be 14 *physical* px -- under a millimetre on a 510 ppi
	## panel. `DccIcons.rect()` reads its own magnification off the canvas
	## transform, which here is 1, so this is the real raster size too.
	var ic := DccIcons.rect(glyph, _pscale(14), "text_dim")
	ic.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	## The glyph sits in a pill that only the ACTIVE tab fills --
	## `candidates/Android Chrome B.dc.html`: `padding:5px 16px;
	## border-radius:14px; background:rgba(224,163,74,.16)`. Lighting only the
	## caption (what the bar did before) left four labels of equal weight and no
	## sense of where you are. Empty stylebox until `_refresh_phone_tabs()`
	## fills it, so an inactive tab is byte-identical to what it drew before.
	var pill := PanelContainer.new()
	pill.add_theme_stylebox_override("panel", DccTheme.empty())
	pill.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	pill.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var pill_pad := MarginContainer.new()
	pill_pad.add_theme_constant_override("margin_left", _pscale(16))
	pill_pad.add_theme_constant_override("margin_right", _pscale(16))
	pill_pad.add_theme_constant_override("margin_top", _pscale(5))
	pill_pad.add_theme_constant_override("margin_bottom", _pscale(5))
	pill_pad.mouse_filter = Control.MOUSE_FILTER_IGNORE
	pill_pad.add_child(ic)
	pill.add_child(pill_pad)
	col.add_child(pill)

	## `9.5px` with `.1em` tracking -- just under 1 px at that size, so `spacing`
	## is 1. This was 9 px at 2 (≈.22em) in Medium: a size down, tracking up and
	## weight up all at once, the same three-error compound `_build_rail()`
	## records for the desktop rail's own caption.
	var l := DccTheme.mono_label(caption.to_upper(), "text_dim", _pfont(9.5), 1, false)
	l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	col.add_child(l)

	b.add_child(col)
	col.set_anchors_preset(Control.PRESET_FULL_RECT)
	return {"button": b, "label": l, "icon": ic, "pill": pill}

## A bar domain was tapped: switch domain and drop any menu that was over it,
## so the result is visible immediately -- the canvas's "the map never leaves
## the screen" rule.
func _pick_bar_domain(id: String) -> void:
	_close_all_phone_overlays()
	_select_domain(id)

## §13: "tool options become a bottom sheet", now with
## `docs/ANDROID_UI_SPEC.md`'s three detents behind it.
##
## The handle used to be decorative, with a comment saying so: the 412 canvas
## pictured one static sheet state and answering a drag would have been invented
## behaviour. `docs/ANDROID_UI_SPEC.md` and the interactive prototype it ships
## with now specify the gesture exactly, so the handle is live -- see
## `_on_phone_sheet_grab_input()` for the drag and `_phone_detent_height()` for
## the three heights.
##
## `tool_options_row` is the same `HBoxContainer` `set_tool_options()` already
## rebuilds from `app.gd` -- wrapped in a `ScrollContainer` here because its
## desktop-tuned content (a run-pipeline row with several buttons and spacers)
## is wider than 412 dp and would otherwise clip.
func _build_phone_tool_sheet() -> PanelContainer:
	var sheet := PanelContainer.new()
	sheet.add_theme_stylebox_override("panel", _phone_sheet_box(false))

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	sheet.add_child(col)

	## The grab handle -- and, as of the detents, the drag target for them.
	## `MOUSE_FILTER_STOP` (a bare `Control`'s default, set explicitly because
	## it is now load-bearing rather than incidental) so the row picks the
	## press; everything else in the phone chrome that is *not* meant to pick
	## is `IGNORE` for the reason `_phone_content_gap` documents at length.
	_phone_sheet_grab = Control.new()
	## `height:24px` with a `40x4` radius-2 bar
	## (`candidates/Android Chrome B.dc.html`, the sheet's own first row). This
	## was `20`/`34x4` from `design/Cartalith Android Phone.dc.html`; the
	## candidate is the newer canvas and `CLAUDE.md`'s first working rule gives
	## it the disagreement.
	_phone_sheet_grab.custom_minimum_size.y = _pscale(24)
	_phone_sheet_grab.mouse_filter = Control.MOUSE_FILTER_STOP
	_phone_sheet_grab.gui_input.connect(_on_phone_sheet_grab_input)
	var handle := ColorRect.new()
	## Token-derived, not a literal white: `DccTheme.remap()` can only repaint a
	## colour it can trace back to a token, so a flat `Color(1,1,1,0.25)` here
	## would stay white when the palette goes light and vanish into the panel.
	## The candidate's `rgba(255,255,255,.25)` is therefore matched in *weight*
	## rather than in value -- `text_ghost` at the alpha this handle already
	## carried, unchanged; only its geometry moved to the candidate's.
	handle.color = Color(DccTheme.c("text_ghost"), 0.55)
	var hw := _pscale(40)
	var hh := _pscale(4)
	handle.set_anchors_preset(Control.PRESET_CENTER)
	handle.size = Vector2(hw, hh)
	handle.position = Vector2(-hw / 2.0, -hh / 2.0)
	handle.mouse_filter = Control.MOUSE_FILTER_IGNORE  ## The bar must not eat
		## the press its own row is there to receive.
	_phone_sheet_grab.add_child(handle)
	col.add_child(_phone_sheet_grab)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	## Was `DISABLED`, on the premise that the sheet always hugs its one content
	## row. A detent sets the height instead, so at `peek` (66 dp, of which the
	## handle row takes 24) that row no longer fits and needs somewhere to go.
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	## And it must take the height the detent hands the sheet, rather than its
	## own minimum -- otherwise `half` and `full` draw a tall empty panel with
	## the content still crushed against the handle.
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.add_theme_stylebox_override("panel", DccTheme.empty())
	tool_options_row = HBoxContainer.new()
	## Scaled like everything else in the sheet: left unscaled these read as a
	## hairline against `_phone_fit_tool_options()`-sized controls, which is
	## what put the first control flush against the screen edge on the device.
	tool_options_row.add_theme_constant_override("separation", _pscale(14))
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _pscale(14))
	pad.add_theme_constant_override("margin_right", _pscale(14))
	pad.add_theme_constant_override("margin_bottom", _pscale(10))
	pad.add_child(tool_options_row)
	scroll.add_child(pad)
	col.add_child(scroll)
	return sheet

## The sheet surface. Portrait: `background:#15171a; border-radius:22px 22px 0 0`
## (`candidates/Android Chrome B.dc.html`, and the prototype's own
## `sheetStyle`). Landscape: `border-radius:0` with `border-left:1px solid
## var(--hair)`, because a sheet docked to the right edge is a dock, not a
## sheet, and the prototype drops the radius there explicitly.
##
## `#15171a` is drawn as the `raised` token (`#17191a`) rather than a twelfth
## near-identical literal -- the accumulation `GUI_GAP_REGISTER.md` §48 exists
## because of -- and `raised` is already this palette's "anything floating",
## which is exactly what a sheet over the map is.
##
## The shadow is the candidate's `box-shadow:0 -8px 24px rgba(0,0,0,.35)`, all
## three values verbatim. It is an **approximation, not a match**:
## `StyleBoxFlat`'s shadow is an expanded, anti-aliased copy of the box, not a
## gaussian blur, so 24 px of it reads as a firmer edge than CSS's would. Both
## are only there to lift the sheet off the map, which it does; said plainly
## because everything else on this surface is exact.
func _phone_sheet_box(docked: bool) -> StyleBoxFlat:
	var box := DccTheme.panel("raised", {"left": 1} if docked else {"top": 1})
	if not docked:
		box.corner_radius_top_left = _pscale(22)
		box.corner_radius_top_right = _pscale(22)
	box.shadow_color = Color(0, 0, 0, 0.35)
	box.shadow_size = _pscale(24)
	box.shadow_offset = Vector2(0, -_pscale(8))
	return box

## What the bottom of the screen owes to chrome that is *not* the sheet: the
## gesture inset, the bottom bar, and (this shell's own addition, which the
## prototype has no row for) the timeline when it is up. The prototype folds the
## first two into one `navH = 84` literal.
func _phone_nav_reserve() -> float:
	var r := float(_safe_bottom())
	if _phone_menu_bar != null and _phone_menu_bar.visible:
		## `_ptap()` rather than the bar's measured `size.y`, for the same
		## reason `_apply_phone_orientation()` gives: this can run before the
		## first layout pass, where that is still zero.
		r += float(_ptap(DccTheme.H_PHONE_BOTTOM_NAV))
	if timeline_bar != null and timeline_bar.visible:
		r += timeline_bar.size.y
	return r

## The three detent heights, from the prototype's `_detH()` -- see the constant
## block above for the transcription. Landscape has no detents (the sheet is
## width-driven there), so this is portrait-only and its callers all guard.
func _phone_detent_height(det: String) -> float:
	var fh: float = get_viewport_rect().size.y - _phone_nav_reserve()
	var peek := float(_pscale(PHONE_DETENT_PEEK))
	match det:
		"peek":
			return peek
		"full":
			## `maxf` guards the degenerate small-window case (a `--force-touch`
			## desktop probe at a few hundred px) where `fh - 96` would come out
			## under the peek height and the sheet would *shrink* on "full".
			return maxf(peek, fh - float(_pscale(PHONE_DETENT_FULL_GAP)))
		_:
			return maxf(peek, round(fh * PHONE_DETENT_HALF_FRAC))

## The phone tool sheet's current detent, and the way back to one.
##
## `menus.gd`'s `_capture_layout()`/`_apply_layout()` have guarded on
## `has_method("phone_detent")` and `has_method("set_phone_detent")` since
## saved layouts existed, and both guards were dead: neither name was ever
## declared here, so `Window ▸ Layouts ▸ Save layout as…` silently stored no
## detent and restoring one silently left the sheet where it was -- while the
## submenu's own tooltip promised "plus the tool sheet's detent on the phone".
## Two accessors, no new state (2026-09-01).
##
## The setter clamps to the three detents `_phone_detent_height()` actually
## knows, so a hand-edited or older config cannot leave `_phone_detent`
## naming a height that does not exist -- the match there would silently
## treat it as "half" while every string comparison elsewhere
## (`_pick_phone_tab`'s `!= "peek"`) read it as something else again.
func phone_detent() -> String:
	return _phone_detent

func set_phone_detent(det: String) -> void:
	if det != "peek" and det != "half" and det != "full":
		return
	_set_phone_detent(det)

## Move to a detent. `animate` is false for the two cases where a transition
## would be wrong: the initial layout, and a rotation (where the whole chrome
## re-lays out in one frame anyway).
func _set_phone_detent(det: String, animate: bool = true) -> void:
	_phone_detent = det
	## `BUILD_ANSWERS.md` §4's "detent snap", 8 ms -- and only on the animated
	## calls. `animate` is false for exactly the two cases that are not a snap
	## the user performed: the initial layout and a rotation. Buzzing on those
	## would fire a detent haptic at launch, before a finger has touched
	## anything.
	if animate:
		_haptic("detent")
	_snap_phone_sheet(animate)

## The prototype's `_snapSheet()`: writes the current detent's height onto the
## sheet, and does nothing at all in landscape.
##
## `custom_minimum_size.y` is the height here because the sheet is a child of a
## `VBoxContainer` -- the container owns the rect, and a minimum is the only
## thing a child gets to say about it. `_phone_content_gap` above it carries
## `SIZE_EXPAND_FILL`, so every pixel the sheet claims comes out of the map gap
## and none of it out of the bar below.
func _snap_phone_sheet(animate: bool) -> void:
	if _phone_tool_sheet == null or _landscape:
		return
	var target := _phone_detent_height(_phone_detent)
	if _phone_sheet_tween != null and _phone_sheet_tween.is_valid():
		_phone_sheet_tween.kill()
	if not animate:
		_phone_tool_sheet.custom_minimum_size.y = target
		phone_insets_changed.emit()
		return
	_phone_sheet_tween = create_tween()
	_phone_sheet_tween.set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	_phone_sheet_tween.tween_property(_phone_tool_sheet,
		"custom_minimum_size:y", target, PHONE_DETENT_ANIM)
	## Fired once, at rest, rather than per-frame: `phone_insets_changed` makes
	## `ViewportHost` re-read `phone_content_insets()` and re-place its floating
	## chrome, and doing that on every tween frame would animate the scale bar
	## and the coordinate readout along with the sheet for no stated reason.
	_phone_sheet_tween.finished.connect(func() -> void: phone_insets_changed.emit())

## One drag on the sheet's grab handle -- the prototype's `_sd`/`_sm`/`_su`
## triple:
##
## - press: record the finger's y and the height it started from, and kill the
##   height transition so the sheet tracks the finger exactly;
## - move: `h = h0 - (y - y0)`, clamped to `[40, frameH - 90]`;
## - release: snap to whichever of the three detent heights is nearest.
##
## Two departures from the prototype, both stated rather than quietly taken:
##
## 1. **Under 44 px this collapses to `peek`; the prototype closes the sheet.**
##    Its sheet is a tab panel with nothing in it when no tab is lit. This one
##    is the desktop's *tool options bar* (§13: "tool options become a bottom
##    sheet") -- the one row that is on screen at all times on desktop and
##    tablet -- so a state where it is gone entirely has no counterpart on the
##    other two form factors to keep parity with. `peek` is the closest honest
##    reading of "dismissed": still there, out of the way.
## 2. **Mouse and touch events are both handled.** Godot only synthesises mouse
##    events from touch while `emulate_mouse_from_touch` is on; it is on by
##    default, but this file may not edit `project.godot` to guarantee it, so
##    the screen events are read directly too. Handling both is safe rather
##    than double-counted: the maths is absolute (recomputed from the finger's
##    current position every event, never accumulated), so a duplicated move is
##    idempotent, a duplicated press re-records the same origin, and the second
##    of a duplicated release finds `_phone_sheet_drag` already empty.
##
## Positions are converted to *global* y. A local y would drift: this control is
## inside the sheet, and the sheet's origin moves upward as it grows, so local
## coordinates shift under a finger that has not moved.
func _on_phone_sheet_grab_input(event: InputEvent) -> void:
	if _landscape or _phone_tool_sheet == null:
		return  ## `_sd()` returns immediately in landscape too: a docked side
			## sheet has a width, not a detent.
	var press_state := 0  ## -1 release, +1 press, 0 not a press event at all.
	var moved := false
	var local_y := 0.0
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index != MOUSE_BUTTON_LEFT:
			return
		press_state = 1 if mb.pressed else -1
		local_y = mb.position.y
	elif event is InputEventScreenTouch:
		var st := event as InputEventScreenTouch
		press_state = 1 if st.pressed else -1
		local_y = st.position.y
	elif event is InputEventMouseMotion:
		moved = true
		local_y = (event as InputEventMouseMotion).position.y
	elif event is InputEventScreenDrag:
		moved = true
		local_y = (event as InputEventScreenDrag).position.y
	else:
		return

	var gy: float = _phone_sheet_grab.global_position.y + local_y
	if press_state == 1:
		if _phone_sheet_tween != null and _phone_sheet_tween.is_valid():
			_phone_sheet_tween.kill()
		_phone_sheet_drag = {"y0": gy, "h0": _phone_tool_sheet.size.y}
		_phone_sheet_grab.accept_event()
		return
	if _phone_sheet_drag.is_empty():
		return
	if moved:
		var lo := float(_pscale(PHONE_DETENT_MIN_DRAG))
		## The prototype's ceiling is `frameH - 90` on an absolutely-positioned
		## sheet, which cannot push anything: here the sheet is a row in a
		## `VBoxContainer`, and a minimum height that large would shove the
		## timeline, the bottom bar and the gesture inset off the bottom of the
		## screen mid-drag. The `full` detent's own height is the ceiling
		## instead -- 90 dp under the prototype's `frameH - 90` on a 412 x 892
		## frame (802 against 712), and it
		## makes "bar stays visible at full sheet" hold *during* the gesture and
		## not merely at rest.
		var hi: float = _phone_detent_height("full")
		var h: float = clampf(float(_phone_sheet_drag["h0"]) \
			- (gy - float(_phone_sheet_drag["y0"])), lo, maxf(lo, hi))
		_phone_sheet_drag["h"] = h
		_phone_tool_sheet.custom_minimum_size.y = h
		_phone_sheet_grab.accept_event()
		return
	## Release. `_su()`: nearest detent by absolute height difference, with the
	## prototype's own `best='half'` as the seed.
	var final_h: float = float(_phone_sheet_drag.get("h", _phone_sheet_drag["h0"]))
	_phone_sheet_drag = {}
	if final_h < float(_pscale(PHONE_DETENT_DISMISS)):
		_set_phone_detent("peek")
		return
	var best := "half"
	var best_d := INF
	for det in ["peek", "half", "full"]:
		var d: float = absf(_phone_detent_height(det) - final_h)
		if d < best_d:
			best_d = d
			best = det
	_set_phone_detent(best)

## The gesture inset: `height:20px` with a `112x4` radius-2 handle at
## `rgba(255,255,255,.22)`, on every one of the 412 canvas's eight screens.
## §13 reserved 26 dp with a 110 px handle; both figures moved.
##
## "No tappable target inside it" still holds, and `MOUSE_FILTER_IGNORE` all the
## way down enforces it structurally rather than visually -- there is nothing
## here a tap could hit even by accident.
func _build_phone_gesture_inset() -> Control:
	var wrap := Control.new()
	wrap.custom_minimum_size.y = _safe_bottom()
	wrap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var bg := ColorRect.new()
	bg.color = Color(DccTheme.c("bg"), 0.9)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
	wrap.add_child(bg)
	var handle := ColorRect.new()
	handle.color = Color(DccTheme.c("text_ghost"), 0.6)  ## Token-derived: see the
		## tool sheet's own handle for why a literal white is wrong here.
	var hw := _pscale(DccTheme.W_PHONE_GESTURE_HANDLE)
	var hh := _pscale(4)
	handle.set_anchors_preset(Control.PRESET_CENTER)
	handle.size = Vector2(hw, hh)
	handle.position = Vector2(-hw / 2.0, -hh / 2.0)
	handle.mouse_filter = Control.MOUSE_FILTER_IGNORE
	wrap.add_child(handle)
	return wrap

# -- Phone overlays: panel picker, overflow, dock sheets ------------------
#
# None of these four states are pictured in the mockup -- it ships exactly
# one static screen, chrome closed. Their *triggers* (☰/▤/⋯) and their
# *destination* (the reused dock/menu-bar/status-bar content) are spec'd;
# the overlay presentation itself is this file's own construction, built to
# the same visual language (colour tokens, hairlines, Plex Mono) as
# everything else in `DccTheme`/`DccWidgets` rather than invented from
# scratch. Said plainly because the rest of this file can cite a mockup line
# for nearly every choice, and these four can't.

## A dimmed full-rect scrim that closes its overlay when tapped outside the
## panel placed on top of it. Named handler rather than an inline lambda --
## `gui_input`'s own event argument plus a multi-statement body closed by the
## outer `connect(...)`'s `)` on the same line is the exact shape the
## match-in-a-lambda gotcha warns about, just with `if` instead of `match`.
func _phone_scrim_tap(ev: InputEvent, on_tap: Callable) -> void:
	var tapped: bool = (ev is InputEventMouseButton and ev.pressed) \
		or (ev is InputEventScreenTouch and ev.pressed)
	if tapped:
		on_tap.call()

func _phone_overlay_scrim(on_tap: Callable) -> Control:
	var wrap := Control.new()
	wrap.set_anchors_preset(Control.PRESET_FULL_RECT)
	var dim := ColorRect.new()
	dim.color = Color(0, 0, 0, 0.55)
	dim.set_anchors_preset(Control.PRESET_FULL_RECT)
	dim.mouse_filter = Control.MOUSE_FILTER_STOP
	dim.gui_input.connect(_phone_scrim_tap.bind(on_tap))
	wrap.add_child(dim)
	return wrap

func _sheet_close_button(on_press: Callable) -> Button:
	var b := Button.new()
	b.text = DccIcons.SYMBOLS["cross"]
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.tooltip_text = "Close"
	## `✕` is the whole control; see `_phone_bar_button()` for why a tooltip is
	## not a name on a touch build.
	b.accessibility_name = "Close"
	b.custom_minimum_size = Vector2(_ptap(44), _ptap(44))
	b.add_theme_color_override("font_color", DccTheme.c("text_faint"))
	b.add_theme_stylebox_override("normal", DccTheme.empty())
	b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	b.pressed.connect(on_press)
	return b

## A tappable title/subtitle row, shared by the drawer's domain list and the
## panel picker's two entries. `rpad`'s own full-rect anchors are what make it
## fill `row` -- `Button` isn't a `Container`, so a child's anchors resolve
## against its rect like any other Control parent, they just aren't
## auto-assigned the way a container's children would be.
func _phone_list_row(title: String, subtitle: String, on_press: Callable) -> Control:
	var row := Button.new()
	## Not flat -- see `add_menu()` and `_build_rail()`: a flat `Button` draws
	## no stylebox, so the press feedback on the next lines had never appeared
	## on a phone list row either.
	row.flat = false
	row.focus_mode = Control.FOCUS_NONE
	row.alignment = HORIZONTAL_ALIGNMENT_LEFT
	row.custom_minimum_size.y = _ptap(52)
	row.add_theme_stylebox_override("normal", DccTheme.empty())
	row.add_theme_stylebox_override("focus", DccTheme.empty())
	row.add_theme_stylebox_override("disabled", DccTheme.empty())
	row.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("line_soft")))
	row.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))

	var rc := VBoxContainer.new()
	rc.add_theme_constant_override("separation", 1)
	rc.mouse_filter = Control.MOUSE_FILTER_IGNORE
	rc.add_child(DccTheme.mono_label(title.to_upper(), "text_bright", _pfont(11), 1, true))
	rc.add_child(DccTheme.label(subtitle, "text_faint", _pfont(9)))
	## The row's two lines are child `Label`s of a `Button` carrying no `text`
	## of its own, so the control a screen reader lands on is nameless -- the
	## same defect `_phone_bar_button()` fixes for a glyph cell, arrived at
	## from the other side: there the name was drawn and unreadable, here it
	## is readable and unreachable. Measured, not assumed -- a walk of the
	## built phone tree listed both panel-picker rows and all three drawer
	## rows with an empty `accessibility_name`.
	##
	## `title` un-cased, not the drawn `to_upper()`: the capitals are
	## typography (`mono_label`'s tracked caps) and a reader that spells out
	## an all-caps string would be reading the styling, not the name.
	row.accessibility_name = title
	row.accessibility_description = subtitle
	var rpad := MarginContainer.new()
	rpad.add_theme_constant_override("margin_left", _pscale(14))
	rpad.add_child(rc)
	row.add_child(rpad)
	rpad.set_anchors_preset(Control.PRESET_FULL_RECT)

	row.pressed.connect(on_press)
	return row

## The ☰ domain drawer -- a 300 dp side sheet listing the three `DOMAINS`
## with their subtitles, plus `_pick_drawer_domain()` and `_set_drawer_open()`
## -- was here and is **deleted** (2026-08-25, the 412 dp migration).
##
## `design/Cartalith Android Phone.dc.html` draws no drawer at any level. Its
## `02 Domain` screen is a full-screen drill with a `←`, which is exactly what
## this shell's full-height left dock sheet already is, so `☰` opens that
## instead. The three domains the drawer listed are the bottom bar's own first
## three cells -- with a glyph each as of this pass -- and a second,
## differently-shaped list of the same three destinations was the duplication
## that canvas's "More is a grouped list, not a duplicate menu bar" rule exists
## to prevent.

## ▤ panel picker: which dock to open as a full-height sheet. Anchored to the
## bottom rather than the drawer's side-panel treatment, so ☰ and ▤ read as
## two distinct affordances rather than the same drawer twice.
func _build_phone_panel_picker() -> Control:
	var overlay := _phone_overlay_scrim(func(): _set_panel_picker_open(false))

	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DccTheme.panel("raised", {"top": 1}))
	panel.set_anchors_preset(Control.PRESET_BOTTOM_WIDE)
	panel.offset_left = 0
	panel.offset_right = 0
	panel.offset_top = -_pscale(160)
	panel.offset_bottom = -_safe_bottom()

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	panel.add_child(col)
	col.add_child(_phone_list_row("Left panel", "The active domain's workspace tools",
		func(): _set_sheet_open("left", true)))
	col.add_child(DccTheme.rule())
	col.add_child(_phone_list_row("Right panel", "Layers and selection detail",
		func(): _set_sheet_open("right", true)))

	overlay.add_child(panel)
	overlay.visible = false
	return overlay

# -- Phone overlay state ---------------------------------------------------
#
# Every phone overlay is mutually exclusive -- opening any one closes all the
# others, including a dock sheet -- so there is exactly one state variable per
# overlay and one shared teardown rather than a general stack. `PhoneMenu` keeps
# its own drill stack inside itself; from out here it is one more overlay.

func _close_all_phone_overlays() -> void:
	if _phone_panel_picker != null:
		_phone_panel_picker.visible = false
	if _phone_search_overlay != null:
		_phone_search_overlay.visible = false
	if _phone_overflow_pop != null:
		_phone_overflow_pop.visible = false
	if _phone_undo_pop != null:
		_phone_undo_pop.visible = false
	if _phone_menu != null:
		_phone_menu.close()
	if left_dock != null:
		left_dock.visible = false
	if right_dock != null:
		right_dock.visible = false
	_left_sheet_open = false
	_right_sheet_open = false
	## `GUI_GAP_REGISTER.md` §46, raised by the concurrent phone pass and picked
	## up here because this is the function that owns the answer. Every entry
	## above is a `Control`; the Layers popover is a `PopupPanel`, which is a
	## `Window`, and no Control walk has ever reached it. Measured by that pass:
	## with the Layers sheet up, opening the ☰ overlay left **both** visible.
	##
	## `Popup` and deliberately not `Window`. A popover is transient -- going
	## somewhere else is what dismisses it -- while an `AcceptDialog` is a modal
	## the user is currently inside, and closing one out from under them would
	## trade a cosmetic overlap for lost input. `PopupMenu` is caught by the same
	## test, which is right for the same reason.
	##
	## `owned = false`: these are built in code and have no scene owner, so the
	## default `owned = true` would return an empty list and this would be
	## another silently-inert fix.
	for node in find_children("", "Popup", true, false):
		var pop := node as Popup
		if pop != null and pop.visible:
			pop.hide()

func _set_panel_picker_open(open: bool) -> void:
	_close_all_phone_overlays()
	_phone_panel_picker.visible = open

# -- ⋮ App-bar overflow (`06-phone.md` §4.3) --------------------------------
#
# `position:absolute; right:10px; top:86px; width:230px; border-radius:18px;
# background:var(--pan); border:1px solid var(--bord); box-shadow:0 14px 34px
# rgba(0,0,0,.45); padding:6px 0`, three rows at `min-height:44px; padding:0
# 16px`, each a label on the left and a `9.5px` mono value on the right.
#
# Three rows, three destinations that already existed:
#   - `Save project` -> `DccApp.save_project()`, which falls through to Save
#     as... on a world that has never been written. The right-hand value is
#     `savedAt`, filled from `EngineBridge.project_saved`.
#   - `Theme` -> `toggle_theme()`, the same palette flip design child 8 in the
#     desktop menu bar drives. The value is the palette that is live now, which
#     is what `themeLabel` binds.
#   - `Close world` -> `DccApp.close_project()`, which is already the shell's
#     one unsaved-work gate (`confirm_unsaved_world()`).
#
# All three are reached by name, and each row is drawn disabled with its reason
# when the method behind it is not there -- a bare `DccShell` probe has none of
# them, and this file's rule is that a drawn row can always be pressed or says
# why not.
func _build_phone_overflow() -> Control:
	var overlay := _phone_overlay_scrim(func(): _set_phone_overflow_open(false))

	var panel := PanelContainer.new()
	var box := DccTheme.panel("raised")
	box.set_corner_radius_all(_pscale(18))
	box.border_color = DccTheme.c("border")
	box.set_border_width_all(1)
	box.content_margin_top = _pscale(6)
	box.content_margin_bottom = _pscale(6)
	panel.add_theme_stylebox_override("panel", box)
	panel.set_anchors_preset(Control.PRESET_TOP_RIGHT)
	## Grows downward off a zero-height rect: a `Control` outside a container is
	## still clamped up to its own combined minimum size, and `grow_vertical`
	## picks which edge stays put while it grows. That is what keeps `top:86px`
	## exact while the row heights decide the rest.
	panel.grow_vertical = Control.GROW_DIRECTION_END
	panel.offset_top = _pscale(86)
	panel.offset_bottom = _pscale(86)
	panel.offset_right = -_pscale(10)
	panel.offset_left = -_pscale(10) - _pscale(230)

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	panel.add_child(col)
	_phone_overflow_saved = _phone_overflow_row(col, "Save project", "save_project",
		"There is no save path on this build (DccApp.save_project is missing).")
	_phone_overflow_theme = _phone_overflow_row(col, "Theme", "toggle_theme", "")
	_phone_overflow_row(col, "Close world", "close_project",
		"There is no close path on this build (DccApp.close_project is missing).")

	overlay.add_child(panel)
	overlay.visible = false
	return overlay

## One row. `method` is called on `self` -- `DccApp` is the subclass this file
## is the base of, so `save_project`/`close_project` resolve there and
## `toggle_theme` here. Returns the right-hand value `Label` so
## `_set_phone_overflow_open()` can refresh it; the caller ignores it for the
## row that has no value.
func _phone_overflow_row(parent: Control, text: String, method: String,
		absent_reason: String) -> Label:
	var row := Button.new()
	row.flat = false
	row.focus_mode = Control.FOCUS_NONE
	row.custom_minimum_size.y = _ptap(44)
	row.add_theme_stylebox_override("normal", DccTheme.empty())
	row.add_theme_stylebox_override("focus", DccTheme.empty())
	row.add_theme_stylebox_override("disabled", DccTheme.empty())
	row.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	row.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("line_soft")))

	var line := HBoxContainer.new()
	line.mouse_filter = Control.MOUSE_FILTER_IGNORE
	line.add_theme_constant_override("separation", _pscale(10))
	line.add_child(DccTheme.label(text, "text", _pfont(12)))
	line.add_child(DccTheme.spacer())
	## `9.5px 'IBM Plex Mono'` -- `var(--faint)` on `savedAt`, `var(--acc)` on
	## `themeLabel`. Theme is the one that reports a live choice rather than a
	## timestamp, which is why the canvas gives it the accent.
	var value := DccTheme.mono_label("", "accent" if method == "toggle_theme" else "text_faint",
		_pscale(10), 0)
	line.add_child(value)
	var lpad := MarginContainer.new()
	lpad.add_theme_constant_override("margin_left", _pscale(16))
	lpad.add_theme_constant_override("margin_right", _pscale(16))
	lpad.mouse_filter = Control.MOUSE_FILTER_IGNORE
	lpad.add_child(line)
	row.add_child(lpad)
	lpad.set_anchors_preset(Control.PRESET_FULL_RECT)

	## Nameless for the same reason `_phone_list_row()` was: the label is a
	## child of a text-less `Button`.
	row.accessibility_name = text
	if has_method(method):
		row.pressed.connect(func():
			_set_phone_overflow_open(false)
			call(method))
	else:
		row.disabled = true
		## A dead control carries its reason -- and on a handset `tooltip_text`
		## carries it nowhere, because there is no hover to raise it. Both, so
		## the 412-wide desktop window this composition is developed in keeps
		## the tooltip it does show.
		row.tooltip_text = absent_reason
		row.accessibility_description = absent_reason
	parent.add_child(row)
	return value

func _set_phone_overflow_open(open: bool) -> void:
	_close_all_phone_overlays()
	if _phone_overflow_pop == null:
		return
	if open:
		## `savedAt` reads `—` until this session has actually written a
		## project, rather than the canvas's mock `14:02`: a time nothing
		## produced is a fabricated record, and the row beside it still works.
		_phone_overflow_saved.text = _phone_saved_at if _phone_saved_at != "" else "—"
		_phone_overflow_theme.text = "dark" if DccTheme.is_dark() else "light"
	_phone_overflow_pop.visible = open

## `project_saved` carries the path; the canvas's `hMenuSave` stamps `HH:MM`.
## Deferred for `_wire_phone_undo_chip()`'s reason exactly -- `bridge` is built
## after the frame this runs in.
func _wire_phone_overflow() -> void:
	(func() -> void:
		var bridge := _find_engine_bridge()
		if bridge == null:
			return
		bridge.project_saved.connect(func(_path: String):
			_phone_saved_at = Time.get_time_string_from_system().substr(0, 5))
	).call_deferred()

# -- ⌕ Find on map --------------------------------------------------------------
#
# `shell/place_search.gd` -- `PlaceSearch.new(); .build(bridge); .size();
# .all(); .search(q)`, each row `{name, kind, subtitle, x, y, entity, id}` --
# is a PARALLEL file landing alongside this one. Never referenced by its class
# name: a bare `PlaceSearch` token in this script would fail to PARSE (not
# just fail at runtime) on any checkout where that file has not landed yet,
# since GDScript resolves a global `class_name` at compile time. Every touch
# below goes through `ResourceLoader.exists()` + `load()` + duck-typed calls
# instead, and `_place_search_index` is deliberately untyped for the same
# reason. `EngineBridge`/`ViewportHost` do not need this treatment -- both are
# permanent, already-shipped files, not this pass's concurrent sibling.

## Whether the index this whole feature depends on exists yet. Checked fresh
## on every call (button build, each open) rather than once at boot and
## cached, because a dev session can have this file's build running before
## `place_search.gd` lands and after, and a cached `false` would leave the
## button undrawable for the rest of that session even once the file showed
## up. `ResourceLoader.exists()` over `ClassDB.class_exists("PlaceSearch")`:
## `PlaceSearch` is a GDScript `class_name`, not an engine-registered class,
## so `ClassDB` never lists it regardless of whether the file exists.
func _has_place_search() -> bool:
	return ResourceLoader.exists("res://shell/place_search.gd")

## Runs (or re-runs) a query against a lazily-built, cached index. Rebuilt
## once per phone-overlay-open / desktop-dialog-open rather than on every
## keystroke -- `text_changed` fires per character, and re-scanning the whole
## world on each one would make the field feel laggy for no benefit `.search()`
## alone doesn't already give it. See `_set_search_open()` and
## `_open_desktop_find_on_map()` for the two places that reset the cache.
func _run_place_search(query: String):
	if not _has_place_search():
		return []
	if _place_search_index == null:
		var script := load("res://shell/place_search.gd")
		if script == null:
			return []
		var bridge := _find_engine_bridge()
		if bridge == null:
			return []  ## Bare `DccShell` -- no bridge, nothing to index.
		var idx = script.new()
		idx.call("build", bridge)
		_place_search_index = idx
	var q := query.strip_edges()
	return _place_search_index.call("all") if q == "" else _place_search_index.call("search", q)

## Drops and rebuilds the cached index -- called whenever an overlay/dialog
## opens, so a world that regenerated since the last search is never searched
## stale. Cheap to over-call: an unopened search never rebuilds anything.
func _reset_place_search_cache() -> void:
	_place_search_index = null

## Renders a result set into `container` as `_phone_list_row()` rows (used on
## BOTH surfaces -- see `_open_desktop_find_on_map()`'s own comment for why
## reusing the phone row on desktop is deliberate here, not an oversight).
func _fill_search_results(container: VBoxContainer, rows, close_fn: Callable) -> void:
	for child in container.get_children():
		child.queue_free()
	if rows == null or (rows as Array).is_empty():
		var pad := MarginContainer.new()
		pad.add_theme_constant_override("margin_left", _pscale(14))
		pad.add_theme_constant_override("margin_top", _pscale(12))
		pad.add_child(DccTheme.label("No matches", "text_faint", _pfont(10)))
		container.add_child(pad)
		return
	for row in rows:
		var d: Dictionary = row
		var title := String(d.get("name", ""))
		var kind := String(d.get("kind", ""))
		var sub := String(d.get("subtitle", ""))
		var line2 := "%s — %s" % [kind, sub] if kind != "" and sub != "" else kind + sub
		container.add_child(_phone_list_row(title, line2,
			_select_search_hit.bind(d, close_fn)))

## A result row was picked: pan the map to it (`x`/`y` are grid cells, the
## same coordinate handling every OTHER `move_view_to()` call site in this
## project already uses -- `faction_roster_window.gd`, `place_editor_window.gd`,
## `civilization_workspace.gd`: `move_view_to(float(int(row.get("x",0))),
## float(int(row.get("y",0))))`, matched verbatim rather than invented here),
## then close whichever surface opened it.
func _select_search_hit(d: Dictionary, close_fn: Callable) -> void:
	var vh := _find_viewport_host()
	if vh != null:
		vh.move_view_to(float(int(d.get("x", 0))), float(int(d.get("y", 0))))
	close_fn.call()

## Public: `menus.gd`'s Edit ▸ Find on map… row calls `_host.open_find_on_map()`
## (that row's own comment names `ViewportHost.move_view_to()` as "the one
## half that already exists" and a place index as the other half -- this
## closes both out). The phone app bar's `⌕` cell calls it too, so there is
## exactly one function deciding what "search" means on this shell, not two
## routes that could drift apart.
func open_find_on_map() -> void:
	if not _has_place_search():
		return  ## Same guard the app-bar cell draws itself behind.
	if _phone:
		_set_search_open(true)
	else:
		_open_desktop_find_on_map()

## ▤-style full-width overlay, `_build_phone_panel_picker()`'s own pattern:
## `_phone_overlay_scrim()` for outside-tap dismissal, `_close_all_phone_
## overlays()`/back-gesture participation, visible only while open.
## Anchored under the app bar (`phone_content_insets().top`, the same figure
## `ViewportHost` reads to keep its own corner chrome clear of it) rather than
## the screen's true top edge, so the app bar -- and the `⌕` cell that opened
## this -- stays visible and tappable-to-close-again while the sheet is up.
func _build_phone_search_overlay() -> Control:
	var overlay := _phone_overlay_scrim(func(): _set_search_open(false))

	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DccTheme.panel("raised", {"bottom": 1}))
	panel.set_anchors_preset(Control.PRESET_TOP_WIDE)
	panel.offset_left = 0
	panel.offset_right = 0
	panel.offset_top = phone_content_insets().get("top", 0.0)
	panel.offset_bottom = panel.offset_top + _pscale(360)

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	panel.add_child(col)

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", _pscale(8))
	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", _pscale(16))
	head_pad.add_theme_constant_override("margin_right", _pscale(8))
	head_pad.add_theme_constant_override("margin_top", _pscale(10))
	head_pad.add_theme_constant_override("margin_bottom", _pscale(6))
	head_pad.add_child(head)
	col.add_child(head_pad)

	_phone_search_field = LineEdit.new()
	_phone_search_field.placeholder_text = "Search places, factions, routes…"
	_phone_search_field.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_phone_search_field.custom_minimum_size.y = _ptap(40)
	DccWidgets.well(_phone_search_field, _pscale(12), _pscale(8))
	_phone_search_field.add_theme_font_size_override("font_size", _pfont(12))
	_phone_search_field.text_changed.connect(func(q: String):
		_fill_search_results(_phone_search_results, _run_place_search(q),
			func(): _set_search_open(false)))
	head.add_child(_phone_search_field)
	head.add_child(_sheet_close_button(func(): _set_search_open(false)))

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.add_theme_stylebox_override("panel", DccTheme.empty())
	col.add_child(scroll)
	_phone_search_results = VBoxContainer.new()
	_phone_search_results.add_theme_constant_override("separation", 0)
	scroll.add_child(_phone_search_results)

	overlay.add_child(panel)
	overlay.visible = false
	return overlay

func _set_search_open(open: bool) -> void:
	_close_all_phone_overlays()
	if _phone_search_overlay == null:
		return
	_phone_search_overlay.visible = open
	if open:
		_reset_place_search_cache()
		_phone_search_field.text = ""
		_fill_search_results(_phone_search_results, _run_place_search(""),
			func(): _set_search_open(false))
		_phone_search_field.grab_focus.call_deferred()

## Desktop's presentation. `AcceptDialog`/`PopupPanel` are retheme'd from the
## project's own theme resource by `_style_window_chrome()` (this file, the
## `panel` stylebox on both type names -- set at boot and again on every
## palette switch), which is why this needs no bespoke scrim/sheet chrome of
## its own the way the phone overlay does: there is a window-manager frame
## under it already, styled the same way every other modal in this shell
## (`World data`, `New world…`, …) already relies on without building its own
## copy of that styling. Built lazily -- once, on first open -- rather than at
## boot, so a desktop session that never searches never pays for it; reused on
## every later call the way `new_world_dialog`/`world_data_window` are reused
## in `app.gd`.
##
## `_phone_list_row()` for its rows even though this is a desktop surface: the
## row's visual language -- rounded press feedback, IBM Plex Mono, a dim
## subtitle line -- is the chrome vocabulary this whole pass's brief names
## ("rounded sheets, pill chips, tonal fills... IBM Plex Mono labels") for the
## shell generally, not a phone-only rule, and `_pscale()`/`_ptap()` are
## identity on desktop (`_phone_scale` stays 1.0, never touched outside
## `_compute_layout_mode()`'s phone branch) -- so the row renders at its
## authored size here, not scaled up. Building a second, near-identical row
## factory for one dialog would be the kind of drift this file's own "one
## visual language" comments elsewhere argue against.
func _open_desktop_find_on_map() -> void:
	_ensure_desktop_search_dialog()
	_reset_place_search_cache()
	_desktop_search_field.text = ""
	_fill_search_results(_desktop_search_results, _run_place_search(""),
		func(): _desktop_search_dialog.hide())
	_desktop_search_dialog.popup_centered(Vector2i(440, 480))
	_desktop_search_field.grab_focus.call_deferred()

func _ensure_desktop_search_dialog() -> void:
	if _desktop_search_dialog != null:
		return
	var dlg := AcceptDialog.new()
	dlg.title = "Find on map"
	dlg.ok_button_text = "Close"
	dlg.min_size = Vector2i(400, 420)
	add_child(dlg)
	_desktop_search_dialog = dlg

	var pad := MarginContainer.new()
	for side in ["left", "right", "top", "bottom"]:
		pad.add_theme_constant_override("margin_%s" % side, 14)
	dlg.add_child(pad)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 8)
	pad.add_child(col)

	_desktop_search_field = LineEdit.new()
	_desktop_search_field.placeholder_text = "Search places, factions, routes…"
	DccWidgets.well(_desktop_search_field)
	_desktop_search_field.text_changed.connect(func(q: String):
		_fill_search_results(_desktop_search_results, _run_place_search(q),
			func(): _desktop_search_dialog.hide()))
	col.add_child(_desktop_search_field)

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(scroll)
	_desktop_search_results = VBoxContainer.new()
	_desktop_search_results.add_theme_constant_override("separation", 0)
	scroll.add_child(_desktop_search_results)

# -- ↶ Floating undo chip --------------------------------------------------------
#
# `bridge` is a `DccApp` field (`app.gd`), not this base class's -- this
# file's own header says it "calls no engine method". `_find_engine_bridge()`/
# `_find_viewport_host()` below are the one, explicitly-guarded exception this
# pass adds, reached by a typed child walk rather than a stored reference so a
# bare `DccShell` (every phone-chrome probe in this project, including this
# pass's own `_phonechrome_probe.gd`) gets null and degrades -- chip stays
# hidden, search button never draws -- instead of failing to compile or crash.
# `EngineBridge`/`ViewportHost` are real, always-shipped classes (unlike
# `PlaceSearch` above), so a static return type is safe here.

func _find_engine_bridge() -> EngineBridge:
	for child in get_children():
		if child is EngineBridge:
			return child
	return null

## `viewport` (`app.gd`'s field) is a child of `viewport_content` -- a field
## THIS class DOES declare and build (`_build_viewport()`): laying out where
## the map surface goes is squarely "the frame", the one thing `app.gd` needs
## handed to it before `viewport.setup(bridge)` can run.
func _find_viewport_host() -> ViewportHost:
	if viewport_content == null:
		return null
	for child in viewport_content.get_children():
		if child is ViewportHost:
			return child
	return null

## The chip itself. `DccTheme.pill()` -- "the ONLY rounded surface in this
## design system" per its own header, the 412 canvas's action-button factory
## -- rather than a hand-rolled `StyleBoxFlat`: this chip IS that button, just
## floating over the map instead of stretched into the tool sheet's action
## row, so it wants the same primary-filled, reversed-ink, 48 dp pill
## `DccWidgets.phone_pill()` builds for the sheet's Commit/Discard buttons.
## Not called through `phone_pill()` itself -- that function reads
## `ACTION_META` + `unit` off a `Button` already sized by `phone_fit()`'s
## walk, which only ever reaches a dock or the tool sheet, and this chip is a
## child of neither -- so this reproduces its recipe (`DccTheme.pill()` twice,
## `accent_hover` on the lit pair, reversed `c("accent_ink")` ink) directly
## instead.
func _build_phone_undo_chip() -> Button:
	var b := Button.new()
	b.name = "PhoneUndoChip"
	b.focus_mode = Control.FOCUS_NONE
	b.text = DccIcons.SYMBOLS["undo"]  ## "↶" -- the spec's own "↶ chip".
	b.tooltip_text = "Undo (hold to see what it would undo)"
	b.accessibility_name = "Undo"
	b.accessibility_description = "Hold to see what the next undo would revert."
	b.add_theme_font_override("font", DccTheme.mono())
	b.add_theme_font_size_override("font_size", _pfont(18))
	var d := _ptap(DccTheme.H_PHONE_PILL)
	var r := d / 2
	var rest := DccTheme.pill(true, r, _pscale(4), _pscale(4))
	var lit := DccTheme.pill(true, r, _pscale(4), _pscale(4))
	lit.bg_color = DccTheme.c("accent_hover")
	b.add_theme_stylebox_override("normal", rest)
	b.add_theme_stylebox_override("disabled", rest)
	b.add_theme_stylebox_override("hover", lit)
	b.add_theme_stylebox_override("pressed", lit)
	b.add_theme_stylebox_override("focus", DccTheme.empty())
	## `accent_ink` since the 2026-08-31 re-base -- see the token's comment in
	## `dcc_theme.gd`. This is a filled amber pill, so it is exactly the case
	## the token exists for.
	var fg := DccTheme.c("accent_ink")
	b.add_theme_color_override("font_color", fg)
	b.add_theme_color_override("font_hover_color", fg)
	b.add_theme_color_override("font_pressed_color", fg)
	b.custom_minimum_size = Vector2(d, d)
	## Bottom-left within `_phone_content_gap` -- see `_build_phone_shell()`'s
	## own comment at this chip's construction site for why that container
	## (not `_phone_nav_reserve()` arithmetic against `_phone_root`) is the
	## bound that is actually clear of both the bottom nav AND the tool sheet,
	## and why bottom-LEFT keeps it off `ViewportHost`'s navpad on the right.
	b.set_anchors_preset(Control.PRESET_BOTTOM_LEFT)
	b.offset_left = _pscale(16)
	b.offset_top = -d - _pscale(16)
	b.offset_right = _pscale(16) + d
	b.offset_bottom = -_pscale(16)
	b.visible = false  ## `_refresh_phone_undo_chip()` shows it once `can_undo()` is true.
	## `pressed` deliberately unused: tap and hold both start from `button_
	## down` so this can tell them apart before either fires (see the pair
	## below) -- `pressed` alone only ever fires on a clean tap and has
	## nothing to race against a hold with.
	b.button_down.connect(_on_phone_undo_chip_down)
	b.button_up.connect(_on_phone_undo_chip_up)
	return b

## `bridge` does not exist yet when `_build_phone_shell()` (and this call,
## made at the end of it) run: `app.gd`'s `_ready()` calls `super._ready()` --
## which is what runs THIS file's `_ready()`, which is what calls
## `_build_phone_shell()` -- and only builds `bridge` in the lines AFTER that
## call returns. Deferred one frame, the same wait `_on_phone_node_added()`'s
## own `call_deferred` relies on elsewhere in this file: by the next idle
## frame `DccApp._ready()` has finished in full and `bridge` is live. A bare
## `DccShell` (a probe) still has none a frame later either, and
## `_find_engine_bridge()` returning null here is exactly the "chip stays
## hidden" degrade this section's header promises.
func _wire_phone_undo_chip() -> void:
	(func() -> void:
		var bridge := _find_engine_bridge()
		if bridge == null:
			return
		## The four signals a commit to the height field can arrive through.
		## NOT `generation_started`/`params_changed` -- a dial moving or a
		## generate merely beginning commits nothing, so `can_undo()` cannot
		## have changed yet; NOT `project_saved` -- writing a `.zip` reads the
		## height field, it does not touch the undo stack.
		for sig in ["generation_finished", "params_applied", "world_loaded", "dirty_changed"]:
			bridge.connect(sig, func(_a = null): _refresh_phone_undo_chip())
		_refresh_phone_undo_chip()
	).call_deferred()

func _refresh_phone_undo_chip() -> void:
	if _phone_undo_chip == null:
		return
	var bridge := _find_engine_bridge()
	_phone_undo_chip.visible = bridge != null and bridge.can_undo()

func _on_phone_undo_chip_down() -> void:
	_undo_chip_down = true
	_undo_chip_hold_fired = false
	get_tree().create_timer(PHONE_UNDO_HOLD_SEC).timeout.connect(
		_check_phone_undo_chip_hold)

## Fires once, `PHONE_UNDO_HOLD_SEC` after a press starts. If the finger is
## still down, this IS the hold -- §6.2's "a 520 ms hold opens a popover".
##
## **This used to show a one-line toast, on a reason that was true and narrower
## than it read.** `EngineBridge` really does expose no per-step history among
## `undo_label()`/`undo_stats()` -- but it exposes `undo_ledger()` and
## `undo_revert_to()` a thousand lines further down, which is exactly the array
## and the roll-back this popover needs, and which the desktop right dock has
## drawn as a multi-step history (`right_dock.gd::_build_history`) since ED-02.
## So no new binding was needed and none is added; this is the same path, in the
## phone's shape.
func _check_phone_undo_chip_hold() -> void:
	if not _undo_chip_down:
		return  ## Already released -- a tap, not a hold; `_on_phone_undo_chip_up()` handled it.
	_undo_chip_hold_fired = true
	_haptic("detent")
	_open_phone_undo_popover()

func _on_phone_undo_chip_up() -> void:
	var was_hold := _undo_chip_hold_fired
	_undo_chip_down = false
	_undo_chip_hold_fired = false
	if was_hold:
		return
	## §6.2: "a short tap (no hold) on the chip undoes one step; if the popover
	## is open, a short tap closes it."
	if _phone_undo_pop != null and _phone_undo_pop.visible:
		_phone_undo_pop.visible = false
		return
	_do_phone_undo()

# -- ↶ Edit-history popover (`06-phone.md` §6.2) ------------------------------
#
# `position:absolute; left:0; bottom:52px; width:220px`, radius 16,
# `background:var(--pan)`, `border:1px solid var(--bord)`, `padding:4px 0`.
# Header `EDIT HISTORY · TAP TO ROLL BACK` at `9px` mono `.18em` `var(--dim)`;
# rows `min-height:40px; padding:0 14px; font:10.5px mono; var(--body)`, label
# `{index+1} · {action}`, newest first, at most six.
#
# The ledger is read fresh on every open, never cached, for `right_dock.gd`'s
# own stated reason: `reversible` is a property of the live undo stack, which
# evicts on its own byte budget, so a cached row would go stale in silence.
#
# **A row that cannot be reverted is drawn dead and says why.** `undo_ledger()`
# reports one row per commit whether or not a height snapshot is still held for
# it, and each such row carries the engine's own `reason`. §6.2 draws every row
# as tappable because its mock stack is uniform; this one is not, and inventing
# a tap that would silently do nothing is the fault this shell's rules exist to
# prevent.
func _build_phone_undo_popover() -> Control:
	var panel := PanelContainer.new()
	panel.name = "PhoneUndoHistory"
	var box := DccTheme.panel("raised")
	box.set_corner_radius_all(_pscale(16))
	box.border_color = DccTheme.c("border")
	box.set_border_width_all(1)
	box.content_margin_top = _pscale(4)
	box.content_margin_bottom = _pscale(4)
	panel.add_theme_stylebox_override("panel", box)
	panel.set_anchors_preset(Control.PRESET_BOTTOM_LEFT)
	## Grows upward off a zero-height rect -- see `_build_phone_overflow()`'s
	## own note on `grow_vertical`; here the bottom edge is the fixed one, so
	## the list rises off the chip instead of sinking behind the tool sheet.
	panel.grow_vertical = Control.GROW_DIRECTION_BEGIN
	panel.offset_left = _pscale(16)
	panel.offset_right = _pscale(16) + _pscale(220)
	## The chip's own box plus §6.2's `bottom:52px` measured from the chip, not
	## from the screen: the chip is `_ptap(H_PHONE_PILL)` tall and sits
	## `_pscale(16)` off the bottom of this container.
	var lift := _pscale(16) + _ptap(DccTheme.H_PHONE_PILL) + _pscale(8)
	panel.offset_top = -lift
	panel.offset_bottom = -lift
	panel.visible = false
	return panel

## Rebuilt on every open rather than kept in sync: six rows off a ledger read is
## cheaper than a subscription, and it is the only moment the list is looked at.
func _open_phone_undo_popover() -> void:
	if _phone_undo_pop == null:
		return
	for c in _phone_undo_pop.get_children():
		_phone_undo_pop.remove_child(c)
		c.queue_free()

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	_phone_undo_pop.add_child(col)

	var head := DccTheme.mono_label("EDIT HISTORY · TAP TO ROLL BACK", "text_dim",
		_pscale(9), 2, false)
	var hpad := MarginContainer.new()
	hpad.add_theme_constant_override("margin_left", _pscale(14))
	hpad.add_theme_constant_override("margin_right", _pscale(14))
	hpad.add_theme_constant_override("margin_top", _pscale(8))
	hpad.add_theme_constant_override("margin_bottom", _pscale(8))
	hpad.add_child(head)
	col.add_child(hpad)

	var bridge := _find_engine_bridge()
	var rows: Array = bridge.undo_ledger() if bridge != null else []
	if rows.is_empty():
		var empty := DccTheme.label(
			"Nothing committed this session. A generate, a load, a Sculpt or Paint commit, "
			+ "a carve or a territory commit all enter here.", "text_ghost", _pscale(10))
		empty.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		var epad := MarginContainer.new()
		epad.add_theme_constant_override("margin_left", _pscale(14))
		epad.add_theme_constant_override("margin_right", _pscale(14))
		epad.add_theme_constant_override("margin_bottom", _pscale(8))
		epad.add_child(empty)
		col.add_child(epad)
	else:
		## Newest first and capped at six, both §6.2's. `i` is the position in
		## the drawn list, so the label numbers what the reader sees rather than
		## the engine's own oldest-first sequence.
		var shown := 0
		for k in range(rows.size() - 1, -1, -1):
			if shown >= 6:
				break
			col.add_child(_phone_undo_row(shown, rows[k]))
			shown += 1

	_close_all_phone_overlays()
	_phone_undo_pop.visible = true

func _phone_undo_row(index: int, entry: Variant) -> Control:
	var d: Dictionary = entry
	var reversible := bool(d.get("reversible", false))
	var seq := int(d.get("seq", 0))
	var steps := int(d.get("steps", 0))
	var row := Button.new()
	row.flat = false
	row.focus_mode = Control.FOCUS_NONE
	row.alignment = HORIZONTAL_ALIGNMENT_LEFT
	row.custom_minimum_size.y = _pscale(40)
	row.text = "%d · %s" % [index + 1, String(d.get("label", "?"))]
	row.clip_text = true
	row.add_theme_font_override("font", DccTheme.mono())
	row.add_theme_font_size_override("font_size", _pfont(10))
	row.add_theme_color_override("font_color", DccTheme.c("text"))
	row.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	row.add_theme_stylebox_override("normal", DccTheme.inset(_pscale(14), 0, _pscale(14), 0))
	row.add_theme_stylebox_override("disabled", DccTheme.inset(_pscale(14), 0, _pscale(14), 0))
	row.add_theme_stylebox_override("focus", DccTheme.empty())
	row.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	row.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("line_soft")))
	if reversible:
		row.tooltip_text = ("%s · %s. Reverts the height field to the state before this "
			+ "operation, discarding the %d step%s after it as well -- history here is "
			+ "linear, so there is no branch to come back to.") % [
				String(d.get("subsystem", "")), String(d.get("detail", "")),
				steps - 1, "" if steps == 2 else "s"]
		row.pressed.connect(func(): _phone_revert_history(seq))
	else:
		row.disabled = true
		var why := String(d.get("reason", ""))
		row.tooltip_text = ("%s · %s. No height snapshot is still held for this step%s" % [
			String(d.get("subsystem", "")), String(d.get("detail", "")),
			(" -- %s." % why) if why != "" else "."])
	return row

## The phone half of `right_dock.gd::_do_revert()`, and the same two lines of
## repaint for the same reason: write `map_view.texture` directly rather than
## calling `ViewportHost.refresh()`, which would also reset the camera. Rolling
## back should leave you looking at exactly where you were looking.
##
## No confirmation dialog in front of it, unlike the desktop's: §6.2 states the
## interaction as a single tap that "reverts every entry above index i", and a
## modal over a 220 dp popover on a handset is a different design, not a
## translation of this one. The row's own label is what says how far back it
## goes, and the toast below reports what actually happened.
func _phone_revert_history(seq: int) -> void:
	var bridge := _find_engine_bridge()
	if bridge == null:
		return
	var done: int = bridge.undo_revert_to(seq)
	if _phone_undo_pop != null:
		_phone_undo_pop.visible = false
	if done <= 0:
		_show_phone_toast(
			"That step is no longer available -- its snapshot was dropped to stay inside "
			+ "the undo budget.", _phone_undo_chip, 3.2)
		return
	var host := _find_viewport_host()
	if host != null:
		host.map_view.texture = bridge.color_texture()
		host.set_preview_texture(null)
	_refresh_phone_undo_chip()
	_show_phone_toast("Reverted %d step%s" % [done, "" if done == 1 else "s"],
		_phone_undo_chip, 2.4)

## Tap. "Undo: ... map edits only" -- `can_undo()`/`undo_last()` are the
## GLOBAL heightmap undo (`engine_bridge.gd`'s "Global heightmap undo" block:
## "Deliberately NOT the same thing as `sculpt_undo`/`sculpt_redo`... those
## pop a stamp off an uncommitted draft, these pop a whole committed height
## field"), never a civilisation/settlement/route edit -- so "map edits only"
## already holds without this file narrowing anything further. Checked
## against that block's own comment rather than assumed true.
func _do_phone_undo() -> void:
	var bridge := _find_engine_bridge()
	if bridge == null or not bridge.can_undo():
		return
	var reverted: String = bridge.undo_last()
	_refresh_phone_undo_chip()
	if reverted != "":
		_show_phone_toast("Undid: %s" % reverted, _phone_undo_chip, 2.4)

# -- ▶ Sim strip (`06-phone.md` §6.2) ------------------------------------------
#
# `bottom:98px` portrait / `bottom:14px` landscape, `z:10`, `padding:0 10px`;
# inner `padding:8px 12px`, radius 18, `background:pillBg`, `border:1px solid
# var(--hair)`. Left to right: play/pause in a `38x38` radius-19 `var(--wash)`
# circle at `13px` mono `var(--acc)`; `YEAR {n}` at `11px` mono `var(--ink)`;
# a `min=-400 max=1200 step=1` slider taking the rest of the width; the three
# speed labels at `9.5px` mono, lit `var(--acc)` and quiet `var(--faint)`; and
# `✕` at `11px` mono `var(--sec)`, which stops playback and hides the strip.
#
# **This is the desktop timeline strip's cursor, not a second one.** Every
# control here goes through the §10a block -- `tl_year()`, `tl_set_year()`,
# `tl_toggle_play()`, `tl_set_speed()` -- which reads and writes the engine's
# `CivData::year` directly. `timeline_changed` is what keeps this view and
# `app.gd`'s desktop strip agreeing; neither holds a year of its own.
#
# There is no entry point in `phone_menu.gd` for it (that file routes
# MORE ▸ Simulation to the CIVIL Simulation category, which is a different
# destination and stays as it is). The phone's own timeline strip row opens it
# -- `app.gd::_fill_timeline_strip()`'s collapsed form -- which is the surface
# the desktop expands in place and the phone has no room to.
func _build_phone_sim_strip() -> Control:
	var wrap := PanelContainer.new()
	wrap.name = "PhoneSimStrip"
	var box := DccTheme.panel("raised")
	box.set_corner_radius_all(_pscale(18))
	box.border_color = DccTheme.c("line")
	box.set_border_width_all(1)
	box.content_margin_left = _pscale(12)
	box.content_margin_right = _pscale(12)
	box.content_margin_top = _pscale(8)
	box.content_margin_bottom = _pscale(8)
	wrap.add_theme_stylebox_override("panel", box)
	wrap.set_anchors_preset(Control.PRESET_BOTTOM_WIDE)
	wrap.grow_vertical = Control.GROW_DIRECTION_BEGIN
	wrap.offset_left = _pscale(10)
	wrap.offset_right = -_pscale(10)
	## Measured off `_phone_content_gap`'s own bottom edge, which already ends
	## above the tool sheet and the bottom bar -- the same bound the undo chip
	## takes, and the reason neither has to redo `_phone_nav_reserve()`'s
	## arithmetic. §6.2's `bottom:98px` is measured from the screen, where the
	## bar and the sheet are still below it.
	wrap.offset_top = -_pscale(14)
	wrap.offset_bottom = -_pscale(14)
	wrap.visible = false

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", _pscale(10))
	wrap.add_child(row)

	## The `38x38` accent circle. `▶`/`⏸` swap in `_refresh_phone_sim_strip()`.
	_phone_sim_play = Button.new()
	_phone_sim_play.focus_mode = Control.FOCUS_NONE
	_phone_sim_play.flat = false
	## Kept in step with the glyph by `_refresh_phone_sim_strip()`; this is the
	## resting state it is built in.
	_phone_sim_play.accessibility_name = "Play"
	var d := _ptap(38)
	_phone_sim_play.custom_minimum_size = Vector2(d, d)
	_phone_sim_play.add_theme_font_override("font", DccTheme.mono())
	_phone_sim_play.add_theme_font_size_override("font_size", _pfont(13))
	_phone_sim_play.add_theme_color_override("font_color", DccTheme.c("accent"))
	_phone_sim_play.add_theme_color_override("font_hover_color", DccTheme.c("accent"))
	_phone_sim_play.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	## `var(--wash)`, not a filled amber slab: the canvas draws the circle at
	## `background:var(--wash)` with accent *ink*, which is the quiet form.
	var circle := DccTheme.flat(DccTheme.c("accent_wash"), d / 2)
	_phone_sim_play.add_theme_stylebox_override("normal", circle)
	_phone_sim_play.add_theme_stylebox_override("disabled", circle)
	_phone_sim_play.add_theme_stylebox_override("focus", DccTheme.empty())
	_phone_sim_play.add_theme_stylebox_override("hover",
		DccTheme.flat(DccTheme.c("accent_wash_2"), d / 2))
	_phone_sim_play.add_theme_stylebox_override("pressed",
		DccTheme.flat(DccTheme.c("accent_wash_2"), d / 2))
	_phone_sim_play.pressed.connect(tl_toggle_play)
	row.add_child(_phone_sim_play)
	_phone_sim_transport.append(_phone_sim_play)

	_phone_sim_year = DccTheme.mono_label("", "text_bright", _pfont(11), 0)
	row.add_child(_phone_sim_year)

	_phone_sim_slider = HSlider.new()
	_phone_sim_slider.min_value = TL_YEAR_MIN
	_phone_sim_slider.max_value = TL_YEAR_MAX
	_phone_sim_slider.step = 1
	_phone_sim_slider.focus_mode = Control.FOCUS_NONE
	_phone_sim_slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_phone_sim_slider.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	DccWidgets.phone_slider(_phone_sim_slider, _phone_scale)
	## `value_changed` and not `drag_ended`: the readout beside it has to follow
	## the finger, and `tl_set_year()` is a cursor write plus a snapshot load
	## keyed on an exact year -- cheap at every year the timeline never
	## recorded, which is almost all of them.
	_phone_sim_slider.value_changed.connect(func(v: float):
		if int(v) != tl_year():
			tl_set_year(int(v)))
	row.add_child(_phone_sim_slider)
	_phone_sim_transport.append(_phone_sim_slider)

	for mult in TL_SPEEDS:
		var b := Button.new()
		b.text = "×%d" % mult
		b.flat = false
		b.focus_mode = Control.FOCUS_NONE
		b.custom_minimum_size = Vector2(_ptap(0), _ptap(0))
		b.add_theme_font_override("font", DccTheme.mono())
		b.add_theme_font_size_override("font_size", _pfont(10))
		b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
		b.add_theme_stylebox_override("normal", DccTheme.inset(_pscale(3), 0, _pscale(3), 0))
		b.add_theme_stylebox_override("disabled", DccTheme.inset(_pscale(3), 0, _pscale(3), 0))
		b.add_theme_stylebox_override("focus", DccTheme.empty())
		b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
		b.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("line_soft")))
		b.tooltip_text = ("How far the year cursor moves per step, and per 600 ms of "
			+ "playback: %d year%s." % [mult, "" if mult == 1 else "s"])
		b.pressed.connect(tl_set_speed.bind(mult))
		_phone_sim_speeds[mult] = b
		_phone_sim_transport.append(b)
		row.add_child(b)

	row.add_child(_sheet_close_button(func():
		if tl_playing:
			tl_toggle_play()
		_phone_sim_strip.visible = false))
	return wrap

## Open or close the strip. Called by the phone's own timeline row; `✕` closes
## it from the inside.
func set_phone_sim_strip_open(open: bool) -> void:
	if _phone_sim_strip == null:
		return
	if not open and tl_playing:
		tl_toggle_play()
	_phone_sim_strip.visible = open
	if open:
		_refresh_phone_sim_strip()

func is_phone_sim_strip_open() -> bool:
	return _phone_sim_strip != null and _phone_sim_strip.visible

## Repaints the strip from the one model. Wired to `timeline_changed`, so a
## scrub on the desktop strip -- or a year jump from the CIVIL dock's pills once
## that file connects too -- moves this slider without either knowing about the
## other.
func _refresh_phone_sim_strip() -> void:
	if _phone_sim_strip == null or not _phone_sim_strip.visible:
		return
	## Nothing here can move the cursor before a generate -- see
	## `tl_available()` -- so the whole transport goes dead and carries the
	## reason rather than answering a tap with silence.
	var live := tl_available()
	for c in _phone_sim_transport:
		if c is Button:
			(c as Button).disabled = not live
		elif c is HSlider:
			(c as HSlider).editable = live
		if not live:
			c.tooltip_text = TL_UNAVAILABLE
	var year := tl_year()
	_phone_sim_year.text = ("YEAR %d" % year) if live else "NO WORLD"
	if not live:
		return
	## `set_value_no_signal`: this is the *echo* of a cursor that has already
	## moved, and letting it re-enter `value_changed` would write the year back
	## to the engine on every refresh.
	_phone_sim_slider.set_value_no_signal(float(year))
	_phone_sim_play.text = DccIcons.SYMBOLS["pause"] if tl_playing \
		else DccIcons.SYMBOLS["play"]
	_phone_sim_play.tooltip_text = ("Pause" if tl_playing else "Play") \
		+ " -- %s. The cursor is the CIVIL timeline's own year (civ_goto_year); the map's territory changes only at the years CIVIL > Politics has recorded." % tl_state_text()
	## The glyph swaps between `▶` and `⏸`, so the name has to swap with it --
	## a fixed "Play" would be wrong for half the button's life.
	_phone_sim_play.accessibility_name = "Pause" if tl_playing else "Play"
	for mult in _phone_sim_speeds:
		var b: Button = _phone_sim_speeds[mult]
		b.add_theme_color_override("font_color",
			DccTheme.c("accent" if int(mult) == tl_speed else "text_faint"))

# -- Toasts: the undo chip's own feedback, and the two coach marks ------------

## A small, subtle, self-dismissing pill -- phone only. One primitive for two
## callers (the undo chip's tap/hold feedback above, and the two first-run
## coach marks below), so "a toast" is one visual language rather than two
## near-duplicates built a few functions apart.
##
## `mouse_filter = IGNORE` throughout: the spec's "never block a tap" is not
## merely "keep it brief" -- it is a requirement that a tap land on whatever
## is under the toast at any point in its lifetime, so this carries no button
## and cannot be the control a `gui_input` walk picks.
func _show_phone_toast(text: String, near: Control, seconds: float = 2.8) -> void:
	if not _phone or _phone_root == null:
		return
	var wrap := PanelContainer.new()
	wrap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var box := DccTheme.panel("raised")
	box.set_corner_radius_all(_pscale(14))
	box.content_margin_left = _pscale(14)
	box.content_margin_right = _pscale(14)
	box.content_margin_top = _pscale(9)
	box.content_margin_bottom = _pscale(9)
	wrap.add_theme_stylebox_override("panel", box)
	var l := DccTheme.mono_label(text, "text_bright", _pfont(10.5))
	l.autowrap_mode = TextServer.AUTOWRAP_WORD
	l.custom_minimum_size.x = _pscale(220)
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	wrap.add_child(l)
	wrap.modulate.a = 0.0
	_phone_root.add_child(wrap)
	_position_phone_toast.call_deferred(wrap, near)
	var tw := create_tween()
	tw.tween_property(wrap, "modulate:a", 1.0, 0.18)
	tw.tween_interval(seconds)
	tw.tween_property(wrap, "modulate:a", 0.0, 0.35)
	tw.finished.connect(func(): if is_instance_valid(wrap): wrap.queue_free())

## Deferred one frame off `_show_phone_toast()` so `wrap.get_combined_minimum_
## size()` reflects its actual label text rather than whatever a zero-frame-old
## node reports. Centres over `near` when it is given and still on screen,
## clamped so the toast itself never runs off any edge.
func _position_phone_toast(wrap: Control, near: Control) -> void:
	if not is_instance_valid(wrap):
		return
	var screen: Vector2 = get_viewport_rect().size
	var size: Vector2 = wrap.get_combined_minimum_size()
	var cx := screen.x * 0.5
	var cy := screen.y * 0.5
	if near != null and is_instance_valid(near) and near.is_inside_tree():
		var r := near.get_global_rect()
		cx = r.position.x + r.size.x * 0.5
		cy = r.position.y - _pscale(12) - size.y * 0.5
	wrap.position = Vector2(
		clampf(cx - size.x * 0.5, _pscale(12), maxf(_pscale(12), screen.x - size.x - _pscale(12))),
		clampf(cy - size.y * 0.5, _pscale(12), maxf(_pscale(12), screen.y - size.y - _pscale(12))))
	wrap.size = size

## Coach marks (§13 chrome: "two subtle toasts, persisted"). The two
## highest-value first-run hints into what THIS file itself builds and can
## point a toast at with a real node: the bottom bar's disclosure model
## (`_phone_menu_bar`) and the tool sheet's drag handle (`_phone_sheet_grab`).
## Long-press-to-sample -- the third candidate the assigning brief named -- is
## deliberately not one of the two: it is `map_overlay.gd`'s gesture, a file
## this pass does not own and has no node handle on, so a toast pointed at it
## would be a guess rather than an anchored hint.
const _COACH_MARKS := [
	## Rewritten against `PHONE_TABS`, which is what the bar has actually drawn
	## since the tab migration. The old text named WORLD/CIVIL/CARTO and PANELS
	## -- three captions the bar no longer carries and one that was never a tab
	## at all -- and so pointed a first-run hint at a bar that does not exist.
	{"id": "bottombar_tabs",
		"text": "MAP · GENERATE · PLAN switch tasks here — MORE reaches everything else."},
	{"id": "sheet_handle", "text": "Drag this handle to expand tool options."},
]

## Probe seam: a GDScript `const` is not an instance property, so
## `Object.get("_COACH_MARKS")` -- the reflection idiom this file's `_find_*`
## helpers rely on for a real `var` -- returns nothing for it. A method is
## reflectable regardless of the underscore convention (`.call()` works on any
## method), so this is what `_phonechrome_probe.gd` actually calls.
func _coach_mark_ids() -> Array:
	var out: Array = []
	for m in _COACH_MARKS:
		out.append(String(m.get("id", "")))
	return out

func _maybe_show_coach_marks() -> void:
	if not _phone:
		return
	(func(): _show_next_coach_mark(0)).call_deferred()

## Shows the first mark in `_COACH_MARKS` not yet seen, waits for it to finish
## (its own display time plus a beat), then recurses to the next -- so the two
## are sequential, never stacked, and a mark already seen on a prior run is
## skipped silently rather than leaving a gap in the sequence.
func _show_next_coach_mark(i: int) -> void:
	if i >= _COACH_MARKS.size():
		return
	var mark: Dictionary = _COACH_MARKS[i]
	var id := String(mark.get("id", ""))
	if _coach_mark_seen(id):
		_show_next_coach_mark(i + 1)
		return
	## The sheet-handle mark points at a control that is hidden and inert in
	## landscape -- `_on_phone_sheet_grab_input()` returns immediately there and
	## `_apply_phone_nav_orientation()` now hides the bar it names. Skipped
	## WITHOUT being marked seen, so a handset that boots landscape still gets
	## the hint the first time it is turned upright, rather than silently
	## burning it against a control that was never on screen.
	if id == "sheet_handle" and _landscape:
		_show_next_coach_mark(i + 1)
		return
	var near: Control = _phone_menu_bar if id == "bottombar_tabs" else _phone_sheet_grab
	_show_phone_toast(String(mark.get("text", "")), near, 3.2)
	_set_coach_mark_seen(id)
	get_tree().create_timer(3.6).timeout.connect(_show_next_coach_mark.bind(i + 1))

## `DccSettings` (`shell/dcc_settings.gd`) exposes only named sections --
## storage roots, recent projects, GPU, autosave -- no generic flag store, and
## this task's file ownership is `dcc_shell.gd` alone, so adding one there is
## out of scope for this pass. This reads/writes the SAME store
## (`DccSettings.CONFIG_PATH`, its own public constant) in a section of its
## own ("coach_marks") instead of inventing a second file, which is what the
## brief actually rules out.
##
## Known rough edge, stated rather than hidden: `DccSettings` caches its own
## `ConfigFile` in memory for the process's whole lifetime (`_ensure_loaded()`'s
## `_loaded` guard) and never re-reads disk, so if it calls its own `_save()`
## AFTER this section is written -- `remember_project()` on the next `.zip`
## load, a GPU or autosave change -- that save serialises `DccSettings`' own
## in-memory copy, which never learned this section exists, and this section
## is lost from disk until this code writes it again. Worst case: a coach
## mark reappears once, in a session that also touches one of those settings
## after dismissing it; never data loss, and self-correcting the next time
## either mark is shown. The clean fix is a real flag API on `DccSettings`
## itself -- a change to a file this task's ownership boundary does not
## permit making here.
func _coach_mark_seen(id: String) -> bool:
	var cfg := ConfigFile.new()
	cfg.load(DccSettings.CONFIG_PATH)
	return bool(cfg.get_value("coach_marks", id, false))

func _set_coach_mark_seen(id: String) -> void:
	var cfg := ConfigFile.new()
	cfg.load(DccSettings.CONFIG_PATH)
	cfg.set_value("coach_marks", id, true)
	cfg.save(DccSettings.CONFIG_PATH)

## Kept under its old name so `_shot_phone.gd --overflow` and anything else
## already driving it keeps working; what it opens is now `PhoneMenu`'s L2 root
## rather than the reparented desktop bar.
func _set_overflow_open(open: bool) -> void:
	_close_all_phone_overlays()
	if open:
		_phone_menu.open()

## The MORE tab is a toggle, because the canvas's `07 More` screen carries no
## close button of its own -- tapping the lit tab again is how you leave it, the
## way a bottom-nav tab behaves everywhere else. Without this, MORE would be the
## one tab in the bar that cannot be undone by pressing it.
func _toggle_overflow() -> void:
	var was_open: bool = _phone_menu != null and _phone_menu.is_open()
	_close_all_phone_overlays()
	if not was_open:
		_phone_menu.open()

## Offer a transient `PopupMenu` the phone's own sheet presentation. Returns
## **false** on desktop and tablet, where the caller should go on and call
## `PopupMenu.popup()` as it always has -- so a call site reads as one line
## with no `is_phone()` branch of its own, and a build with no phone chrome
## behaves identically to one that never heard of this function.
##
## Built for `civilization_workspace.gd`'s map context menu, which on a phone
## is opened by a press-and-hold (`map_overlay.gd`) and cannot use a stock
## popup: pointer-sized rows, and clipping rather than nudging when a finger
## lands near the screen edge.
func phone_present_popup(popup: PopupMenu, title: String, trail: String) -> bool:
	if not _phone or _phone_menu == null:
		return false
	_close_all_phone_overlays()
	_phone_menu.open_sheet(popup, title, trail)
	return true

## Android's back gesture -- the hardware `KEYCODE_BACK` and the edge swipe that
## replaced it -- arriving as `NOTIFICATION_WM_GO_BACK_REQUEST` because
## `_ready()` turned `quit_on_go_back` off. The canvas's BACK rule is "leaves a
## sheet, then the L2 screen, then the viewport", and one press leaves exactly
## ONE level, innermost first:
##
##   1. a dialog or popup window, wherever in the tree it is parented,
##   2. a phone-menu level (L5 → L4 → L3 → L2 → closed),
##   3. any other phone overlay -- panel picker, either dock sheet,
##   4. `_back_exhausted()`, which `DccApp` overrides to disarm a live tool and,
##      failing that, to put the SAME save/discard/cancel prompt File ▸ Close
##      project uses in front of the exit.
##
## Step 4 is why this was reopened. The first version ended in a bare
## `get_tree().quit()`, so a back gesture at the viewport with an unsaved
## generated world in memory destroyed it with no prompt at all. That is not a
## hypothetical: it happened to a tester on an OnePlus 6T, and the world was
## gone. Nothing in this shell may end the process without going through the
## same gate File ▸ Close project goes through.
func _notification(what: int) -> void:
	## The desktop system close, handled here for the one reason the whole gate
	## exists: it must not be possible to end this process with unsaved work in
	## it without being asked. Deliberately NOT routed through the back chain
	## above -- back means "leave the innermost thing", and the × means "close
	## the application", so it skips straight past dialogs, menu levels and
	## armed tools to the exit gate itself.
	##
	## Reaches this node only for the MAIN window: Godot propagates a window's
	## close request DOWN its own subtree (`Window::_propagate_window_notification`
	## stops at nested `Window`s), so closing a tool window or a dialog -- all of
	## which are children of this shell, not parents of it -- never lands here.
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		_close_requested()
		return
	if what != NOTIFICATION_WM_GO_BACK_REQUEST:
		return
	## Innermost first. An embedded dialog draws OVER the phone menu, so the
	## menu must not eat the gesture while one is open. Hidden rather than
	## freed: every dialog in this shell already frees itself from
	## `visibility_changed`, and hiding is precisely what its Cancel does.
	## `BUILD_ANSWERS.md` §4's "back", 6 ms, on each of the three levels a press
	## actually LEAVES -- and deliberately not on `_back_exhausted()`, which is
	## not a level left but the exit gate, and which `DccApp` answers with a
	## dialog of its own.
	var top := _topmost_subwindow(get_tree().root)
	if top != null:
		top.hide()
		_haptic("back")
		return
	if _phone_menu != null and _phone_menu.go_back():
		_haptic("back")
		return
	if (_phone_panel_picker != null and _phone_panel_picker.visible) \
			or (_phone_search_overlay != null and _phone_search_overlay.visible) \
			or _left_sheet_open or _right_sheet_open:
		_close_all_phone_overlays()
		_haptic("back")
		return
	_back_exhausted()

## The deepest visible `Window` under `root`, `root` itself excluded.
##
## Walked rather than read off a list, because a dialog is parented to whichever
## `Control` opened it and not to the root -- `DccApp`'s own prompts are children
## of `DccApp`, and `Viewport` exposes no subwindow list to GDScript. One walk
## per back press is not a cost worth optimising.
func _topmost_subwindow(node: Node) -> Window:
	var found: Window = null
	for child in node.get_children():
		if child is Window and not (child as Window).visible:
			continue  ## A hidden window's own children are unreachable too.
		var deeper := _topmost_subwindow(child)
		if deeper != null:
			found = deeper
		elif child is Window:
			found = child
	return found

## What back does once there is nothing left to leave. `DccShell` on its own
## holds no document, so quitting is correct here; `DccApp` overrides it to
## guard unsaved work first.
func _back_exhausted() -> void:
	get_tree().quit()

## The window manager asked for the app to close. `auto_accept_quit` is off, so
## this function OWNS the exit: if it neither quits nor puts a resolvable prompt
## on screen, the window cannot be closed at all. `DccShell` holds no document,
## so quitting outright is correct here; `DccApp` overrides it to guard unsaved
## work first, and carries the argument for why its version is still always
## escapable.
func _close_requested() -> void:
	get_tree().quit()

func _set_sheet_open(side: String, open: bool) -> void:
	if open:
		_close_all_phone_overlays()
	if side == "left":
		_left_sheet_open = open
		left_dock.visible = open
		if open:
			_reset_dock_scroll(_left_dock_scroll)
	else:
		_right_sheet_open = open
		right_dock.visible = open
		if open:
			_reset_dock_scroll(_right_dock_scroll)

## `phone_menu.gd::_render()` zeroes its own scroll the same way on every
## fill; a dock sheet's body, unlike the menu's, is never torn down and
## rebuilt between opens (`_build_left_dock`/`_build_right_dock` run once, at
## shell build time, `as_sheet = true` only swapping the header) -- so
## nothing else ever touches `scroll_vertical` back to 0, and whatever the
## sheet was scrolled to when it last closed is still sitting on the
## `ScrollContainer` when it reopens. Set once immediately (the common case)
## and once more deferred: a `ScrollContainer` that was `visible = false` a
## moment ago has not necessarily run its own sort/clamp pass yet, and a bare
## synchronous write here can still be re-clamped against a stale scrollbar
## range on the same frame the sheet becomes visible.
func _reset_dock_scroll(scroll: ScrollContainer) -> void:
	if scroll == null:
		return
	scroll.scroll_vertical = 0
	scroll.call_deferred("set", "scroll_vertical", 0)

## Re-applied on every resize while `_phone` is true (`_on_window_resized()`),
## which is the one part of the phone/tablet decision that genuinely must be
## live -- a device rotates at runtime even though its form factor never
## does. Only safe-area visibility, the chrome column's left margin, and the
## two dock sheets' rects change between orientations; the panel-
## picker/menu overlays and the tool sheet/timeline/bottom bar are unaffected,
## so this never touches anything a workspace has attached content to.
func _apply_phone_orientation() -> void:
	if _phone_top_safe == null:
		return  ## Not built yet -- called once more at the end of `_build_phone_shell()`.
	## `and _phone_status_shown`: `Window ▸ Status bar` is the other input to
	## these two, and it has to survive a rotation -- see
	## `set_status_region_shown()` for what the row means on this composition.
	_phone_top_safe.visible = (not _landscape) and _phone_status_shown
	_phone_side_safe.visible = _landscape and _phone_status_shown

	## "The chrome shifts inward" (inset rule, LANDSCAPE): everything below the
	## safe area lives inside this margin, so growing it by the side safe area's
	## own width is the entire mechanism.
	var side_reserve := _safe_side() if _landscape else 0
	## ...and in landscape two more regions now live at the edges: the nav rail
	## on the left and the docked sheet on the right
	## (`docs/ANDROID_UI_SPEC.md`: "nav becomes left rail, sheet docks right,
	## map stays wide"). The chrome column runs *between* them.
	##
	## The prototype instead lets the sheet overlay the app bar (its map host is
	## `left:72px;right:0` and the sheet sits on top at `z-index:12`). Insetting
	## the column is the deliberate departure: this app bar carries the search
	## and overflow buttons, which the prototype's does not, and a docked sheet
	## covering them would put two of the phone's five always-available controls
	## behind a sheet the user has to close to reach them. The *map* is still
	## edge-to-edge behind everything, so "map stays wide" is unaffected.
	var rail_w := _pscale(W_PHONE_LAND_RAIL) if _landscape else 0
	var sheet_w := _phone_land_sheet_width() if _landscape else 0
	_phone_chrome_margin.add_theme_constant_override("margin_left",
		side_reserve + rail_w)
	_phone_chrome_margin.add_theme_constant_override("margin_right", sheet_w)
	## ...and up, by whatever the on-screen keyboard is currently covering. In
	## portrait this column IS the bottom dock -- the tool sheet, the timeline
	## and the bottom bar are its last three children -- so one margin lifts all
	## of them clear of the IME, which is `dockBottom`'s whole job in the
	## prototype (`BUILD_ANSWERS.md` §4). In landscape those two are reparented
	## out to `_phone_root` and are handled by `gesture` in
	## `_apply_phone_nav_orientation()` instead; the margin still applies here,
	## harmlessly, because the app bar it does still contain is anchored to the
	## top.
	_phone_chrome_margin.add_theme_constant_override("margin_bottom",
		_phone_kb_height)

	_apply_phone_nav_orientation(side_reserve, rail_w, sheet_w)

	var safe_top := 0 if _landscape else _safe_top()
	for sheet in [left_dock, right_dock]:
		if sheet == null:
			continue
		## `+ rail_w`: a full-height dock sheet must clear the nav rail for the
		## same reason it clears the side safe area -- the rail is L1 of the
		## disclosure tree in landscape exactly as the bottom bar is in
		## portrait, and covering it would strand the user in the sheet.
		sheet.offset_left = side_reserve + rail_w
		sheet.offset_right = 0
		sheet.offset_top = safe_top
		sheet.offset_bottom = -_safe_bottom()

	## The menu takes the same rect as a dock sheet: over the app bar (its own
	## header replaces it, per the canvas's L2/L3 artboards), never over the
	## status safe area, the landscape side safe area or the gesture inset.
	##
	## **And never over the bottom bar**, as of the 412 migration. That bar is
	## L1 of the disclosure tree, and the canvas's `07 More` screen -- which is
	## what the menu's root is -- carries no close button of its own precisely
	## because it is a *tab destination*: you leave it by tapping another tab, or
	## the lit one. Covering the bar with the screen it opens would make MORE the
	## one tab in the bar that cannot be undone by pressing it, and (with the
	## canvas's two-`✕`-becomes-none change) would leave system back as the only
	## way out at all.
	##
	## `_ptap()` rather than the bar's measured `size.y`: this runs before the
	## first layout pass, where that is still zero, and it is the same expression
	## the bar sets its own minimum from.
	##
	## In landscape that same bar is the left rail, so its reserve moves from
	## the menu's bottom inset to its left one -- `rail_w`, folded into the
	## side reserve below. Getting this wrong would have been invisible in
	## portrait and would have covered the rail with the screen the rail opens.
	var bar_reserve := 0
	if _phone_menu_bar != null and _phone_menu_bar.visible and not _landscape:
		bar_reserve = _ptap(DccTheme.H_PHONE_BOTTOM_NAV)
	if _phone_menu != null:
		_phone_menu.apply_insets(float(safe_top), float(side_reserve + rail_w),
			float(_safe_bottom() + bar_reserve))

	phone_insets_changed.emit()

## `Math.min(440, Math.round(fw * 0.46))` -- the prototype's landscape sheet
## width, in its own dp, mapped onto real pixels. `fw` there is the frame's
## *long* side, which in landscape is the viewport width.
func _phone_land_sheet_width() -> int:
	return int(minf(float(_pscale(W_PHONE_LAND_SHEET_MAX)),
		round(get_viewport_rect().size.x * PHONE_LAND_SHEET_FRAC)))

## The landscape half of "nav becomes left rail, sheet docks right" -- and the
## portrait half that undoes it.
##
## Both regions are the **same nodes** in both orientations: rotating the bar is
## a `BoxContainer.vertical` flip (an `HBoxContainer` in Godot 4 is a
## `BoxContainer` with `vertical = false`, so this is a property, not a class),
## and relocating either one is a reparent between the chrome column and
## `_phone_root`. A second bar and a second sheet built alongside would have
## meant `set_tool_options()`, `_select_domain()`, `_domain_buttons`,
## `_phone_tab_cells` and `rail_column` each carrying two targets to keep in
## step -- five places for the two to drift apart.
func _apply_phone_nav_orientation(side_reserve: int, rail_w: int,
		sheet_w: int) -> void:
	if _phone_menu_bar == null or _phone_tool_sheet == null \
			or _phone_chrome_col == null:
		return
	## The two nodes this function places in landscape sit on `_phone_root`,
	## OUTSIDE `_phone_chrome_margin` and so outside the keyboard margin it
	## carries -- which is why the IME height is folded in here as well. In
	## portrait `gesture` is unused (both readers below are inside `if
	## _landscape`), so this cannot double-count against that margin.
	var gesture := _safe_bottom() + _phone_kb_height

	## `_phone_root` is a plain `Control`, so a child of it keeps whatever
	## anchors it is given -- the two dock sheets already rely on exactly that.
	var host: Node = _phone_root if _landscape else _phone_chrome_col
	for node in [_phone_tool_sheet, _phone_menu_bar]:
		if node.get_parent() == host:
			continue
		node.get_parent().remove_child(node)
		host.add_child(node)
	if _landscape:
		## Index 3 is directly above `_phone_chrome_margin` and below every
		## overlay (`_phone_side_safe`, the panel picker, `PhoneMenu`, the two
		## dock sheets) -- the same stacking order these two have in portrait,
		## where the overlays are added to `_phone_root` after the chrome.
		_phone_root.move_child(_phone_tool_sheet, 3)
		_phone_root.move_child(_phone_menu_bar, 3)
	else:
		## Back into their portrait slots. The order is re-established from the
		## top down, each target derived from the node just placed above it,
		## rather than by naming a neighbour's index directly.
		##
		## **`move_child()` inserts at the index it is given**, so the intuitive
		## "put the bar where the gesture inset currently is" lands it one slot
		## *below* the inset whenever the bar is already above it -- which is
		## the state the build-time call runs in. Caught on the device-shaped
		## probe: the bottom bar drew at y=827 in an 892 px frame, i.e. flush to
		## the bottom edge with the 20 px gesture inset stranded above it,
		## every launch, before any rotation.
		var order: Array[Node] = [_phone_content_gap, _phone_tool_sheet]
		if timeline_bar != null:
			order.append(timeline_bar)
		order.append(_phone_menu_bar)
		order.append(_phone_gesture_inset)
		for i in range(1, order.size()):
			_phone_chrome_col.move_child(order[i], order[i - 1].get_index() + 1)

	## -- the bar --------------------------------------------------------
	for box in [_phone_bar_row, _phone_bar_domains, _phone_bar_cells,
			_phone_bar_dests]:
		if box != null:
			box.vertical = _landscape
	## Both stretch boxes, not just the outer one: `_phone_bar_dests` carries
	## the three destination cells' share of exactly the same stretch, so a flip
	## that skipped it would leave them sharing the bar's OLD axis.
	for box in [_phone_bar_domains, _phone_bar_dests]:
		if box == null:
			continue
		## The stretch that makes the four cells share the bar has to change
		## axis with it; `size_flags_stretch_ratio` is axis-agnostic and stays.
		box.size_flags_horizontal = \
			Control.SIZE_FILL if _landscape else Control.SIZE_EXPAND_FILL
		box.size_flags_vertical = \
			Control.SIZE_EXPAND_FILL if _landscape else Control.SIZE_FILL
	if _phone_bar_cells != null:
		## Prototype rail: `gap:6` between cells. Its `padding-top:40` is not
		## carried across -- that clears the prototype's status row, which sits
		## along the top edge in both orientations; this shell rotates its
		## status row onto the *left* edge in landscape (`_phone_side_safe`),
		## and the rail already starts to the right of it.
		_phone_bar_cells.add_theme_constant_override("separation",
			_pscale(6) if _landscape else 0)
		## And the same gap inside the destination box, or the three cells it
		## holds would sit flush against one another while MORE alone stood off.
		if _phone_bar_dests != null:
			_phone_bar_dests.add_theme_constant_override("separation",
				_pscale(6) if _landscape else 0)
	## `border-right:1px solid var(--hair2)` on `var(--pan2)` -- and `--pan2` is
	## `#121314`, which is the `panel` token the bar already draws in, so only
	## the edge moves.
	_phone_menu_bar.add_theme_stylebox_override("panel",
		DccTheme.panel("panel", {"right": 1} if _landscape else {"top": 1}))
	if _landscape:
		_phone_menu_bar.set_anchors_preset(Control.PRESET_LEFT_WIDE)
		_phone_menu_bar.offset_left = side_reserve
		_phone_menu_bar.offset_right = side_reserve + rail_w
		_phone_menu_bar.offset_top = 0
		_phone_menu_bar.offset_bottom = -gesture
		_phone_menu_bar.custom_minimum_size = Vector2(float(rail_w), 0.0)
	else:
		_phone_menu_bar.custom_minimum_size = Vector2.ZERO

	## -- the sheet ------------------------------------------------------
	## The grab handle is a PORTRAIT affordance only.
	## `_on_phone_sheet_grab_input()` returns immediately in landscape, because
	## `BUILD_ANSWERS.md` §4 rules that the landscape drawer has no detents to
	## re-snap to -- correct, and correctly implemented. What was missing is the
	## disclosure: a 40x4 bar was still drawn over a 24 dp row that could not be
	## dragged, and coach mark #2 told the user to drag it. Hidden rather than
	## dimmed, because there is no disabled state for a drag target and no hover
	## surface on a phone to carry the reason in a tooltip -- the coach mark is
	## skipped in landscape instead (`_show_next_coach_mark()`).
	if _phone_sheet_grab != null:
		_phone_sheet_grab.visible = not _landscape
	if _landscape:
		_phone_tool_sheet.set_anchors_preset(Control.PRESET_RIGHT_WIDE)
		_phone_tool_sheet.offset_left = -sheet_w
		_phone_tool_sheet.offset_right = 0
		_phone_tool_sheet.offset_top = 0
		## §13: "Timeline and sheets stop above it" -- the gesture inset is the
		## one region a docked sheet still may not reach into.
		_phone_tool_sheet.offset_bottom = -gesture
		## The detent height is a portrait concept; in landscape the anchors own
		## the rect and a leftover minimum would fight them.
		_phone_tool_sheet.custom_minimum_size.y = 0.0
		_phone_tool_sheet.add_theme_stylebox_override("panel",
			_phone_sheet_box(true))
	else:
		_phone_tool_sheet.add_theme_stylebox_override("panel",
			_phone_sheet_box(false))
		## Un-animated: a rotation re-lays the whole chrome out in one frame,
		## and a 0.28 s height tween across that reads as a glitch, not a
		## transition.
		_snap_phone_sheet(false)

## What `ViewportHost`'s own corner chrome (layers button, coord readout,
## scale bar -- all built by `viewport_host.gd`, unchanged) should treat as
## "inside the safe area" now that the phone chrome sits on top of an
## edge-to-edge map instead of a flow container sizing the viewport to the
## gap between docks. `app.gd` calls this once, deferred, after building
## `ViewportHost` (deferred so the tool sheet has had one layout pass -- see
## `_phone_bottom_reserve()`), and again on every `phone_insets_changed`
## (rotation, and any tool-sheet content change -- `set_tool_options()` fires
## it too).
func phone_content_insets() -> Dictionary:
	if not _phone:
		return {"left": 10.0, "top": 10.0, "right": 10.0, "bottom": 10.0, "scale": 1.0}
	## No left reserve in portrait any more: the domain rail that used to float
	## there is the bottom bar now, so the map has the full width back and only
	## the landscape safe area still eats into it.
	var left := 0.0
	var right := 0.0
	if _landscape:
		left += float(_safe_side())
		## Landscape's two new edge regions: the nav rail on the left, the
		## docked sheet on the right (`docs/ANDROID_UI_SPEC.md`). Both are
		## opaque, so `ViewportHost`'s floating chrome has to clear them the
		## same way it clears the bottom bar in portrait -- and `right` had
		## never been anything but 0 before there was something over there.
		left += float(_pscale(W_PHONE_LAND_RAIL))
		right = float(_phone_land_sheet_width())
	var top := float(_ptap(DccTheme.H_PHONE_APP_BAR))
	if not _landscape:
		top += float(_safe_top())
	## `scale` rides along because it is the one number `ViewportHost` needs and
	## has no other route to (`GUI_GAP_REGISTER.md` HD-03). Its floating chrome
	## -- the Layers button and the four navpad pills -- is authored at a raw
	## 44 px, on a premise `NAVPAD_HIT`'s own comment states outright and which
	## is false: the main viewport is NOT content-scaled, so the shipped
	## handset's viewport is its full pixel width, not 393. Measured, 44 px
	## comes to 2.83 mm on a 1080/395 ppi panel and **2.19 mm** on the
	## OnePlus 12's 1440/510, against the ~7 mm a 44 dp floor is asking for.
	## Passing it through the existing dictionary rather than adding a setter
	## keeps `app.gd` -- which owns the one call site -- out of it.
	## The clamp is the detents' doing. At `full` the sheet covers everything
	## but 96 dp of map, so the *honest* bottom reserve exceeds the screen and
	## `ViewportHost` would place its coordinate readout and scale bar above the
	## top edge -- off screen, which is worse than occluded. One tap target of
	## band is left for them; they overlap the sheet's top edge at that detent,
	## which is the same trade `_phone_bottom_reserve()` already documents in
	## the other direction.
	var band: float = get_viewport_rect().size.y - top - float(_ptap(DccTheme.PHONE_TAP_MIN))
	## `+ _phone_kb_height`: `ViewportHost`'s floating chrome has to clear the
	## IME for the same reason the docked chrome does -- the keyboard draws over
	## the frame rather than resizing it, so nothing else moves out of its way.
	var bottom: float = clampf(_phone_bottom_reserve() + float(_phone_kb_height),
		0.0, maxf(0.0, band))
	return {"left": left, "top": top, "right": right,
		"bottom": bottom, "scale": _phone_scale}

## The tool sheet's real height depends on whatever `tool_options_row`
## currently holds -- domain content this frame doesn't own -- so this reads
## the sheet's own actual size once it has one (non-zero after a layout pass)
## rather than guessing it. Before that first pass (the un-deferred instant
## `_build_phone_shell()` finishes in), falls back to a fixed estimate biased
## generous on purpose: an *under*-estimate leaves the coordinate readout and
## scale bar hidden behind the opaque sheet (found by screenshot -- the
## original flat 44 px guess did exactly this), while an *over*-estimate only
## leaves them floating a bit higher than strictly necessary. Wrong in the
## safe direction, in other words.
func _phone_bottom_reserve() -> float:
	if _landscape:
		## Neither the bar nor the sheet is at the bottom any more -- they are
		## the left rail and the right dock. Only the gesture inset and the
		## timeline still are, and both are reported to the caller as `left`
		## and `right` instead (see `phone_content_insets()`).
		var b := float(_safe_bottom())
		if timeline_bar != null and timeline_bar.visible:
			b += timeline_bar.size.y
		return b
	var bar := 0.0
	if _phone_menu_bar != null and _phone_menu_bar.visible:
		bar = _phone_menu_bar.size.y if _phone_menu_bar.size.y > 0.0 \
			else float(_ptap(64))
	if _phone_tool_sheet != null and _phone_tool_sheet.size.y > 0.0:
		var h := _phone_tool_sheet.size.y + float(_safe_bottom())
		if timeline_bar != null and timeline_bar.visible:
			h += timeline_bar.size.y
		return h + bar + float(_pscale(8))
	return float(_pscale(20 + 90) + _safe_bottom()) + bar
