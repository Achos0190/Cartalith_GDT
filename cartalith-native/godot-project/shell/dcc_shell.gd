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
## The foot's second form -- `StatusBar.dc.html` panel 5's upright `00` / rule /
## `10`. `rail_foot` above still holds the whole string; this draws it.
var _rail_foot_stack: MarginContainer
var _rail_foot_run: Label
var _rail_foot_total: Label
const RAIL_FOOT_GAP := 6      ## `gap:6px` between the two halves and the rule.
const RAIL_FOOT_PAD_B := 8    ## `padding-bottom:8px` under the lower half.
const RAIL_FOOT_RULE_W := 14  ## `width:14px` on the `--div` hairline.
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
## The two things the tool sheet can hold, and which one is up. The tool-options
## scroller is the desktop bar (§13); the GENERATE column is the canvas's
## `tabIsGen` sheet. Exactly one is visible -- see `_refresh_phone_gen_panel()`.
var _phone_tool_scroll: ScrollContainer
var _phone_gen_scroll: ScrollContainer
var _phone_gen_col: VBoxContainer
var _phone_app_bar: PanelContainer  ## Held so a probe can measure it and so
	## `phone_content_insets()` reads its real height rather than recomputing it.
var _phone_gesture_inset: Control

# -- Phone bottom-sheet detents ------------------------------------------------
#
# **Derived from** `design/dcc-environment-2026-08-31/spec/06-phone.md` §5.1
# (geometry), §5.2 (`_detH(det)`) and §5.3 (drag handle) -- the in-repo,
# greppable authority for everything in this block, added 2026-09-05 alongside
# the citation below rather than in place of it.
#
# The older citation, `docs/ANDROID_UI_SPEC.md`, Locked decisions: *"Sheets:
# peek → half → full detents, drag handle; tab tap opens half"* and *"bar stays
# visible at full sheet"*. **That file is not in this repository** and never has
# been (`git log --all -- '*ANDROID_UI_SPEC*'` is empty): per
# `design/android-2026-08-30/README.md` it lives in the owner's Claude Design
# project and is reachable only through DesignSync. It is kept here because it
# is real and it is what the detents were built from; §5 is added because a
# reader who greps this repo for it finds nothing.
#
# The three heights are the interactive prototype's own arithmetic, not derived
# ones -- `design/android-2026-08-30/Cartalith Android.dc.html`, `_detH()`, and
# **byte-for-byte the same expression** in the newer
# `design/dcc-environment-2026-08-31/Cartalith Android.dc.html` that §5 is a
# transcription of (the two imports differ -- 157 021 vs 168 836 bytes -- and
# `_detH` is not one of the places they differ):
#
#     fh   = frameH - 84            (portrait; the whole frame in landscape)
#     peek = 66                     a constant, not a fraction of anything
#     half = round(fh * 0.46)
#     full = fh - 96                so 96 dp of map is never covered
#
# That `84` is the prototype's `navH` (`navH = land ? 0 : 84`), and it is this
# shell's bottom bar (`H_PHONE_BOTTOM_NAV` 64) plus its gesture inset
# (`H_PHONE_GESTURE` 20) -- the same figure reached from two constants instead
# of one literal. **The two split it differently**: §3 gives the same 84 as a
# 66 dp tab row over an 18 dp gesture inset, so the total agrees and the seam
# inside it does not. Nothing here depends on the seam; `H_PHONE_BOTTOM_NAV`
# and `H_PHONE_GESTURE` are `dcc_theme.gd`'s, and the discrepancy is recorded
# rather than resolved from this file.
#
# `_phone_nav_reserve()` reads it live rather than hard-coding 84, because this
# shell parks the timeline between the sheet and the bar and the prototype has
# no equivalent of that row -- **and because its gesture term is
# `_safe_bottom()`, which is `max(_pscale(H_PHONE_GESTURE), the real display
# inset)`, not `H_PHONE_GESTURE` unconditionally.** On a handset whose bottom
# cutout inset exceeds 20 dp the reserve grows and `fh` shrinks with it, which
# is the intended behaviour and is why the sum is computed rather than written
# down.
#
# Measured 2026-09-05 at three densities by `_detent_probe.gd`, which drives
# the gesture through `SubViewport.push_input()` -- the viewport's own hit-test
# and `gui_input` dispatch, not a direct call on the handler. `dp` is physical
# px over `phone_scale()`; the three scales are distinct (1.7476 / 2.6214 /
# 3.4951) so this is three densities and not one box measured three times:
#
#     720x1600   scale 1.7476  reserve  147 px = 84.12 dp  fh 1453 px = 831.44 dp
#     1080x2340  scale 2.6214  reserve  220 px = 83.93 dp  fh 2120 px = 808.74 dp
#     1440x3200  scale 3.4951  reserve  294 px = 84.12 dp  fh 2906 px = 831.44 dp
#
#     peek  115 /  173 /  231 px  = 65.81 / 66.00 / 66.09 dp   (§5.2: 66)
#     half  668 /  975 / 1337 px  = 382.24 / 371.94 / 382.53   (§5.2: 382 / 372 / 382)
#     full 1285 / 1868 / 2570 px  = 735.31 / 712.61 / 735.31   (§5.2: 735 / 713 / 735)
#
# All three land inside 1 dp of §5.2 at all three densities; the residue is
# `_pscale()` rounding a dp constant into whole physical px, not a divergence.
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
var _phone_search_chips: HFlowContainer
var _phone_search_count: Label
var _place_search_index  ## `PlaceSearch` -- untyped, see `_has_place_search()`.
## `CommandIndex`, built lazily the first time a `.` query is run and dropped by
## `_reset_place_search_cache()` alongside the place index. Statically typed,
## unlike `_place_search_index` beside it: `command_index.gd` is a shipped file
## with eight probes already naming the class, not this pass's concurrent
## sibling, so a bare `CommandIndex` token here cannot fail to parse.
var _command_index: CommandIndex

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
var _desktop_search_chips: HFlowContainer
var _desktop_search_count: Label

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
	## DS-03's width invariant, at the one point where both docks exist in
	## either composition -- see `dock_fit()` for what it does, why it is
	## density-independent unlike the two walks beside it, and the measurement.
	## The immediate call covers the dock chrome `_build_left_dock()` already
	## attached, which the hook cannot revisit; the hook covers every workspace
	## panel, all of which arrive later.
	if left_dock != null and right_dock != null:
		_run_dock_fit()
		get_tree().node_added.connect(_on_dock_node_added)

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

	## **A tablet is a touch device and this composition never treated it as
	## one.** `_build_phone_shell()` connects `node_added` so every panel a
	## workspace attaches later gets `phone_fit()`; the tablet ran the same
	## desktop shell with no equivalent, so `DccWidgets.touch_slider()` and
	## `touch_release_button()` -- both attached inside `phone_fit()` -- never
	## reached it. Measured at `--force-touch --vp 800x1280` (which
	## `_compute_layout_mode()` reports `phone=false tablet=true`), at boot,
	## before this call existed: **224** unarbitrated writable `Range` inside a
	## live vertical scroller (`_rangeswipe_probe.gd --census-only`) and **27**
	## unarbitrated touch-DOWN controls (`_gestclass_probe.gd --census-only`).
	##
	## Connected here, at the end of `_build_desktop_shell()`, for the reason
	## the phone's own connect states: both docks have to exist first, or the
	## handler walks toward a null.
	##
	## **The hook does all of the measured work and the immediate call does
	## none of it**, and that is stated rather than implied because the obvious
	## reading is the other way round. Mutated both ways, same probe and frame:
	## hook alone **10**, boot walk alone **224** -- every panel a workspace
	## attaches arrives after this function returns, so a one-shot walk here
	## sees only chrome. The boot call is kept for that chrome (the menu bar,
	## the tool-options row, the rail, the timeline and the status bar are
	## outside both docks and the hook never revisits them) and it contributes
	## **0** to this census, which counts only what sits inside a live vertical
	## scroller.
	if DccTheme.is_tablet():
		tablet_arbitrate(self)
		get_tree().node_added.connect(_on_tablet_node_added)

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
	## The **fifth density set's** publication, and the whole of the orientation
	## wiring on this side. `_landscape` is computed thirteen lines up and has
	## been since the phone composition needed it; this hands the same fact down
	## to `DccTheme`, where `role_px()` can reach it. Nothing here learns a new
	## question, and no second orientation source is introduced -- which was the
	## brief: reuse the phone's machinery rather than build a tablet copy of it.
	DccTheme.set_portrait(not _landscape)
	## **Both non-desktop bands resolve their dock pair identically, so they are
	## one branch.** `role_px()` consults `LAPTOP` for the narrow-pointer band
	## and `TABLET_PORTRAIT` for portrait touch, and otherwise falls through to
	## `ROLE`'s touch column -- 400/400, which is §1's "so two-column readouts
	## survive the larger type" (`UI_SHELL_DESIGN.md`) and is what landscape
	## tablet keeps. Every width is therefore stated once, in `DccTheme`, and
	## none is duplicated here.
	##
	## **The touch half read `float(DccTheme.W_DOCK_TABLET)` for both docks,
	## unconditionally, and that is the measured defect.** At 800 x 1280
	## `--force-touch` the shell laid out rail 47 + left 400 + viewport 237 +
	## right 400 = 1084 px inside a frame of 800, putting the right dock's edge
	## at x = 1085: +285 px of overflow, the menu bar cut mid-word, no map
	## visible. `role_px()` returns the same 400 in landscape, so the landscape
	## frames do not move; see `DccTheme.TABLET_PORTRAIT` for why only the
	## portrait half of the canvas's 232/320 pair is taken.
	##
	## Runs from `_ready()`, and on a tablet once more per orientation flip --
	## `_on_tablet_resized()` below, where that exception to the early return
	## and its WI-04 constraint are argued.
	if (_touch and not _phone) or DccTheme.is_laptop():
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
		_on_tablet_resized()
		return  ## Desktop windows resizing is not this shell's concern.
	var was_landscape := _landscape
	_compute_layout_mode()
	if _landscape != was_landscape:
		_apply_phone_orientation()

## The tablet's share of the resize handler, and it acts on exactly one event:
## an orientation flip. Added 2026-09-07 with the portrait dock fix.
##
## **The early return above is not weakened.** Its purpose (WI-04, argued in the
## comment on `_on_window_resized()`) is that a tablet user's dragged dock
## widths survive a resize, and they still do: the flip test below reads the raw
## viewport size and returns *before* `_compute_layout_mode()` -- which is the
## call that would overwrite `_left_width`/`_right_width` -- so a same-orientation
## resize mutates nothing at all.
##
## A flip is the one case where the dragged width cannot be kept, and that is
## not a preference: portrait and landscape are different budgets (232 against
## 400, `DccTheme.TABLET_PORTRAIT`), and a landscape width carried into portrait
## is the +285 px overflow this batch fixed. Without this function the fix would
## hold only for a shell that launched already in portrait.
##
## Desktop pays one predicate and leaves: `DccTheme.is_tablet()` is `touch and
## not phone`.
func _on_tablet_resized() -> void:
	if not DccTheme.is_tablet():
		return
	var size: Vector2 = get_viewport_rect().size
	if size.x <= 0.0 or size.y <= 0.0:
		return
	if (size.x > size.y) == _landscape:
		return  ## Same orientation. WI-04's dragged widths stand.
	_compute_layout_mode()
	_apply_dock_widths()

## Pushes `_left_width`/`_right_width` onto the two docks. A collapsed dock is
## skipped rather than re-expanded -- its `custom_minimum_size.x` is
## `W_RAIL_COLLAPSED` while collapsed, and `_toggle_dock()` reads the fields
## this function does not touch when it expands again, so the new width is
## picked up there.
func _apply_dock_widths() -> void:
	if left_dock != null and not _left_collapsed:
		left_dock.custom_minimum_size.x = _left_width
	if right_dock != null and not _right_collapsed:
		right_dock.custom_minimum_size.x = _right_width

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
## `pscale(px) >= px >= 44` already). At scale 2.621 the app bar's glyph cells
## grow from 105 to `_pscale(44)` = 115 physical px (+10, ~9%) -- still well
## inside the app bar's own `_ptap(H_PHONE_APP_BAR)` = 147 px height, and the
## row still fits its widest measured target (720 px short side) with room to
## spare for "CARTALITH" + the seed subtitle. **That measurement was taken
## against the four-cell row `☰ ⌕ ▤ ⋮`; ruling 20 has since cut it to `⌕ ⋮`
## beside the world pill (`_build_phone_app_bar()`), so the headroom is larger
## now, not smaller, and the figure above is left as the worst case it was.**
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

	## **Re-anchored 2026-09-07 against the live canvas, which says none of what
	## the four figures here used to say.** `Cartalith DCC Environment.dc.html`
	## line 57, quoted whole because every term of it was wrong in this file:
	##
	##     font:var(--m1) 'IBM Plex Mono',monospace;letter-spacing:.18em;
	##     color:var(--acc);margin:0 12px 0 4px
	##
	## - `var(--m1)` is **10 px** pointer and **12 px** touch (`ENV:25` /
	##   `ENV:1819`). This drew `FS_MENU` = 12 at both, so the desktop wordmark
	##   was 20 % over and the tablet one was right by coincidence.
	## - the declaration carries **no weight**, so it is regular 400. This drew
	##   `medium = true`.
	## - `.18em` is 1.8 px at 10 and 2.16 px at 12; Godot spacing is an integer,
	##   so **2** at both densities. This drew 3.
	## - `var(--acc)` is the amber `#e0a34a`. This drew `text_bright`, a
	##   near-white -- the single most visible of the four.
	##
	## `DccTheme.ROLE`'s own `fs_wordmark` is `[12, 15]` and documents itself as
	## *"`font:500 12px` .26em → `500 15px`"*, which is the canvas retired on
	## 2026-08-23; it has no consumer and is not read here. Naming it would have
	## replaced one stale figure with another.
	var wordmark := DccTheme.mono_label("CARTALITH", "accent",
		DccTheme.FS_MENU if DccTheme.is_tablet() else DccTheme.FS_TINY, 2)
	row.add_child(wordmark)
	## `margin:0 12px 0 4px`. The 4 px left margin is already paid by the pad
	## above: the canvas bar is `padding:0 10px` (`ENV:56`) and 10 + 4 = the 14
	## this `MarginContainer` sets. The right margin is **12**, and was 22 here
	## behind a comment that asserted the canvas said 22 -- it does not, and the
	## error was invisible because a comment naming a canvas reads as a
	## measurement. The 22 is still correct one clause down, where the readout
	## loop uses it to stand in for `ENV`'s own `margin-left:8px` plus gap.
	var wordmark_gap := Control.new()
	wordmark_gap.custom_minimum_size.x = 12
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
	##
	## **The canvas draws ONE cell here, and it is a departure that is now
	## stated rather than silent.** `ENV:103-104` is `<span style="flex:1">`
	## then a single `{{ worldLabel }}`; `res`, `cpu`, `gpu` and `mem` are this
	## shell's own. They are **kept**, because deleting them is a capability
	## loss with no other desktop home: `grep -rn top_cpu --include=*.gd` puts
	## their only other reader in `phone_menu.gd`'s MORE ▸ STATUS list (rows
	## `top_res`/`top_cpu`/`top_gpu`/`top_mem`, `phone_menu.gd:305-312`), which
	## is a phone surface. `app.gd:773-774` writes `top_res` and `top_mem` on
	## every generate, and `menus.gd:3287` reads `top_mem` back so the two
	## cannot disagree. Removing them needs an owner ruling and a home for the
	## four figures, not a lane's edit.
	##
	## What IS corrected here is the type, which had no such justification:
	## `ENV:104` is `font:var(--m1)` -- 10 px pointer, 12 px touch -- in
	## `var(--dim)` `#8d9296`. This drew `FS_READOUT` (a flat 11 at both
	## densities) in `text_faint` `#6f7478`, so every cell was a rung too big
	## on the desktop and a rung too dark everywhere.
	var readout_fs: int = DccTheme.FS_MENU if DccTheme.is_tablet() else DccTheme.FS_TINY
	for slot in ["world", "res", "cpu", "gpu", "mem"]:
		var l := DccTheme.mono_label("", "text_dim", readout_fs, 1)
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
## `undoCol`/`redoCol` **are readable, and they say exactly this.** Prototype
## line 1907: `undoCol:s.undoStack.length?'var(--sec)':'var(--dis)'`, and
## `redoCol` the same over `redoStack`. So `--sec` (`text_secondary`) live and
## `--dis` (`text_ghost`) dead is a quotation, not a stand-in, and
## `_paint_menu_square()`'s swap between them is the canvas's own rule —
## "can I press this" carried by ink as well as by the `disabled` flag.
##
## Corrected 2026-09-05. This comment previously said the pair "were inside the
## prototype's truncated tail and do not exist to read", citing
## `03-menu-bar.md`'s file-integrity header. That header, like
## `04-left-dock.md` §0/§9.1, describes a 262 144-byte truncated copy; the
## frozen file here is 239 712 bytes and complete. The choice this comment
## defends was right — it is the *reason* that was wrong, which is the more
## expensive half to leave standing.
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
			## **Both axes unconditionally.** The width floor used to be nested
			## under `if min_size.x > 0.0`, which made it reach only controls
			## that had already declared a width -- so an icon-only button, a
			## bare `CheckBox`, anything sized by its own content, was never
			## floored horizontally however many times this walk ran over it.
			## The route-map layer button is the instance that surfaced it
			## (35 x 115 physical px, 13 dp wide against a 44 dp floor, *after*
			## `phone_fit()` had run; fixed in `journey_planner_view.gd` by
			## declaring a width, which is to say by satisfying the condition
			## rather than by removing it). It was never one button:
			## `_widthfloor_probe.gd` counts **174** laid-out tappable controls
			## under the 115 px floor at a 1080 x 2340 / 412 dp handset, and
			## **all 174** of them have `custom_minimum_size.x == 0` -- the
			## guard was the whole cause, not a contributing one.
			##
			## **What the condition protected, measured rather than assumed:**
			## nothing. It is not a protection, it is the shape of the two
			## `round(min_size.? * unit)` lines above -- where `> 0.0` really is
			## a no-op, since `round(0.0 * unit)` is `0.0` -- carried down two
			## lines to a `maxf()` where it is not. Both were written in one
			## edit, in `_phone_fit_tool_options()`, this walk's first version
			## (`c33ccb6`), over a single tool-options row. Removing it and
			## re-censusing the whole 4 315-control surface: 439 controls grew,
			## **2** shrank (the "Enable continental steering" row label
			## 497 -> 457 px, whose text wants 364, and the spacer beside it),
			## **0** controls changed height, **0** controls' text stopped
			## fitting, 77 controls that had never resolved a width got one, and
			## of 17 `Window`s exactly one content minimum moved -- 497 -> 507
			## against a 1080 px screen. Nothing overflowed that was not
			## already inside a horizontal scroller by design.
			##
			## The one consequence worth stating rather than burying: `Range`
			## is in this list, so a `VSlider` or `ProgressBar` would now get a
			## 115 px *width* minimum, which for a `VSlider` would be wrong.
			## Neither exists anywhere in the phone tree today: the censused
			## tappable population is `Button` 276, `HSlider` 216, `CheckBox`
			## 53, `SpinBox` 12, `LineEdit` 2, and opening the roster, the data
			## manager and the credits dialog adds only a single `TextEdit` to
			## that list of classes. 208 of the 216 `HSlider`s already carry a
			## non-zero `custom_minimum_size.x` and so were floored before this
			## change too; `DccWidgets.slider()` is one writer of that
			## (`custom_minimum_size = Vector2(track_w, 14)`) and no attempt was
			## made to attribute all 208 to it. So no carve-out is written for a
			## control that is not here. `tablet_fit()` below
			## does exclude `Range`, and its reason is explicitly about
			## *height* growing a fixed-height bar; it is not a width precedent
			## and is not cited as one.
			if ctl is BaseButton or ctl is LineEdit or ctl is Range or ctl is TextEdit:
				min_size.y = maxf(min_size.y, tap)
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
			## Everything from here to the `ScrollContainer` deadzone -- and
			## the `touch_slider()` call that used to sit in the `HSlider`
			## branch above -- moved into `_touch_arbitrate()` below, verbatim,
			## so the tablet can reach the same body. Nothing between the two
			## old positions was executable (lines 1960-1978 were comment), so
			## the order this runs in is unchanged.
			_touch_arbitrate(ctl, unit)
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

## **The gesture arbitration, extracted so the tablet can have it too.**
##
## Every clause below answers one question -- *what does a touch that begins
## on this control mean?* -- and none of them sizes anything. That is the whole
## reason this is a separate body from `phone_fit()`'s sizing work and from
## `tablet_fit()`'s: `GUI_GAP_REGISTER.md` §57 refuted `is_touch()` as a
## predicate for **sizing** (it is true on a phone, "and the 412 canvas asks
## for things a tablet must not get"), and every one of its three refutations
## is about a figure or a `unit`. None of them touches gesture classification,
## which is a property of the *input device* and identical on both.
##
## **The gap this closes, measured before the change** -- `--force-touch --vp
## 800x1280`, which `_compute_layout_mode()` classifies `phone=false
## tablet=true` (aspect .625, over `_PHONE_ASPECT_MAX`):
##
## | probe | at boot | what it means on glass |
## |---|---|---|
## | `_rangeswipe_probe.gd --census-only` | **224** unarbitrated writable `Range` in a live vertical scroller (214 of them `left_dock sheet`) | a vertical flick that starts on a slider rewrites the parameter, silently |
## | `_gestclass_probe.gd --census-only` | **27** unarbitrated touch-DOWN (16 `OptionButton`, 7 `SpinBoxLineEdit`, 4 `LineEdit`) | a flick that starts on a dropdown opens its popup and then picks from it |
##
## Those are the same two defects `phone_fit()` was extended to close on
## 2026-09-07, on a composition that is just as touch-only -- a tablet has no
## wheel and no hover either. The tablet simply never ran the walk that carries
## them: `tablet_fit()` had **one** call site (`tool_options_row`), and the
## `node_added` hook that keeps the phone's docks fitted is connected inside
## `_build_phone_shell()` and nowhere else.
##
## `unit` is a multiplier on *travelled distance*, not on a hit area, and it is
## **1.0 on a tablet** for the same reason `_scaled()` reads a table rather than
## a scale factor: the tablet composition lays out in physical pixels with no
## `content_scale_factor`, so 8 authored px is 8 real px there. The phone passes
## `_phone_scale` down a dock and `1.0` inside a content-scaled `Window`, and
## both routes still reach this function unchanged.
func _touch_arbitrate(ctl: Control, unit: float) -> void:
	if ctl is HSlider:
		## **And the gesture arbitration, which is the other half of the
		## same sentence.** `DccWidgets.PgSlider` withholds the press
		## until the gesture has travelled 8 dp and then gives it to the
		## axis it travelled furthest along, because Godot's own
		## `Slider::gui_input` sets the value on touch-DOWN -- so a
		## vertical swipe that happens to begin on a slider rewrites the
		## parameter instead of scrolling the sheet. Measured on glass:
		## Ocean depth `0.60` -> `0.14` in one gesture, silently.
		##
		## Attached HERE rather than at each factory because a live
		## census of the phone tree at 1080x2340
		## (`_rangeswipe_probe.gd --census-only`) counted 247 `Range`
		## nodes, **245 writable and inside a live vertical scroller, of
		## which 3 arbitrated** -- and the other 242 are built by six
		## different files. This walk already reaches all but two of
		## them (`phone_menu.gd` and `world_workspace.gd` attach their
		## own; neither surface is one `phone_fit()` walks).
		## `8.0 * unit` is a distance TRAVELLED, so it takes the same
		## `unit` the tap floor above takes and is deliberately not
		## floored at 44: a slop is not a hit area.
		DccWidgets.touch_slider(ctl as HSlider, 8.0 * unit)
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
	## An `HSlider` is deliberately **not** included, and the reason
	## written here until 2026-09-07 -- *"a drag that starts on a slider
	## means 'move this slider', on every touch platform there is"* --
	## was refuted on glass. It is true of the HORIZONTAL drag and false
	## of the vertical one, and Godot makes it worse than a
	## misclassification: `Slider::gui_input` calls `set_as_ratio()` from
	## the PRESS position, so the value has already jumped before there
	## is any motion to classify. `MOUSE_FILTER_PASS` would not have
	## fixed that either -- a `PASS` control is still picked and still
	## runs its own `gui_input`. The arbitration happens inside the
	## control instead (`DccWidgets.touch_slider()`, attached a few lines
	## above), which needs the `STOP` this clause leaves in place.
	##
	## **A `BaseButton` that opens its popup on *press* is converted
	## rather than excluded, and that is a change from what stood here
	## until 2026-09-07.** The old reason -- *"such a control pops
	## mid-flick, the popup grabs the drag, and the gesture then
	## neither scrolls nor is undone (measured on `OptionButton`: popup
	## open, scroll 0)"* -- was a correct measurement of `PASS` alone,
	## and `PASS` alone is not the fix. It leaves `action_mode` at
	## `ACTION_MODE_BUTTON_PRESS`, so the popup still opens under the
	## finger and the scroll it now forwards is a scroll of the popup.
	##
	## `DccWidgets.touch_release_button()`, called just below, moves
	## `action_mode` to RELEASE **and** takes the control to `PASS` in
	## the same step -- so by the time this clause runs, a converted
	## dropdown is already `PASS` and this `if` is a no-op for it. The
	## exclusion list stays for the ones it did NOT convert: a dropdown
	## with no vertical scroller above it (the census's `scroller=none`
	## rows -- one in `asset_library_window.gd`, one in
	## `city_viewer_window.gd`, and the seven `MenuButton`s of the menu
	## bar) keeps opening on press, because there is nothing there for a
	## vertical gesture to mean instead.
	##
	## `ColorPickerButton` is excluded on its own footing and not by
	## association: it measures `action_mode == 1` (RELEASE) on 4.7.1,
	## so it is not the touch-DOWN defect at all. It stays `STOP`
	## because whether that blocks a scroll under it is a separate
	## question, and no probe in this pass could stage a visible one.
	if ctl is BaseButton and ctl.mouse_filter == Control.MOUSE_FILTER_STOP \
			and not (ctl is OptionButton or ctl is MenuButton \
				or ctl is ColorPickerButton):
		ctl.mouse_filter = Control.MOUSE_FILTER_PASS
	## **The other half of the same sentence, and the half that makes
	## the clause above safe to widen.** Godot's `OptionButton` and
	## `MenuButton` ship `action_mode == ACTION_MODE_BUTTON_PRESS`
	## (measured on 4.7.1, against `CheckBox` 1, `ColorPickerButton` 1
	## and `Button` 1), so their popup opens on touch-DOWN, before there
	## is any motion to classify -- the §1.14 slider defect one class
	## over. `_gestclass_probe.gd` at 1080x2340, before this call
	## existed: a jittered vertical swipe on a left-dock
	## `DccWidgets.choice()` row took `sel=7 -> sel=3` with the sheet
	## not moving, and the same swipe on the New World card left its
	## popup standing open.
	##
	## **The census, with its state named, because the state is most of
	## the number.** `_gestclass_probe.gd --census-only` at 1080x2340,
	## with this call neutered, counts unarbitrated touch-DOWN controls
	## inside a live vertical scroller as:
	##
	##   world-less boot          **22** = 18 `OptionButton` + 4 `LineEdit`
	##   after tapping PLAN       **38** = 34 + 4
	##   after a generate         **44** = 40 + 4
	##
	## With it in place, **4** in every one of those states, all
	## `LineEdit`. So this converts **40** dropdowns, not the 18 a
	## world-less boot can see -- the same understatement §1.15's
	## slider census recorded about itself and which cost that pass
	## four uncovered sites. The 18 break down 10 left sheet, 6 New
	## World card, 2 in `asset_library_window.gd`'s slicer modal (an
	## unscripted `AcceptDialog`, fitted by its own `phone_fit` call).
	##
	## The 4 `LineEdit`s are left alone: they write no value, they take
	## FOCUS from the press (and on Android raise the soft keyboard
	## over the sheet the swipe was scrolling), and no state this probe
	## could stage put a visible one inside a scroller -- so the change
	## would have been unmeasurable. Reported, not changed.
	##
	## Placed in `phone_fit()` for the reason `touch_slider()` above is:
	## the population is built by five different files and this walk
	## already reaches every one of them.
	if ctl is BaseButton:
		DccWidgets.touch_release_button(ctl as BaseButton)
	## **The third member of the same family, and the one the paragraph
	## above deferred.** That note said the 4 `LineEdit`s were "left
	## alone … no state this probe could stage put a visible one inside
	## a scroller -- so the change would have been unmeasurable". A
	## verifier then staged one: a jittered vertical swipe on the New
	## World card's **Seed** field gives scroll `0 -> 0` with its
	## internal `SpinBoxLineEdit` focused, while the label column at the
	## same `y` scrolls `0 -> 62`. On Android that focus raises the soft
	## keyboard over the sheet the swipe was trying to scroll.
	##
	## Neither switch above is the switch here -- a `LineEdit` has no
	## `action_mode`, and the `SpinBoxLineEdit` rows are **already
	## `MOUSE_FILTER_PASS` and still eat the swipe**, because
	## `LineEdit::gui_input` accepts every left press. The lever is
	## `focus_mode`, measured against three alternatives in
	## `DccWidgets.PgField`'s own table.
	##
	## **Two call sites, not one**, and the second is the reason the
	## first is not enough: a `SpinBox`'s field is an INTERNAL child,
	## and this walk iterates `get_children()`, which excludes internal
	## children -- so **12 of the 16 hazardous fields at boot** are
	## unreachable from the `ctl is LineEdit` branch and have to be asked
	## for by name. (20 after PLAN or MORE, 24 with a world loaded; the
	## ratio at maximum is 20 of 24. This said "12 of 34", and **no state
	## produces 34** -- it was 16 plus the 18 hidden `PopupMenu` search
	## fields, which the same change taught the census to exclude.)
	if ctl is LineEdit:
		DccWidgets.touch_focus_field(ctl as LineEdit, 8.0 * unit)
	elif ctl is SpinBox:
		DccWidgets.touch_focus_field(
			(ctl as SpinBox).get_line_edit(), 8.0 * unit)
	## `TextEdit` is deliberately not in that list. It is the other
	## touch-DOWN text class, but it carries its OWN vertical scroll, so
	## "give the vertical to the ancestor" is the wrong answer for it
	## and no state this pass could stage put one inside a live
	## scroller. **Stated precisely, because the first version of this
	## line overstated it:** `--census-only` reports exactly ONE
	## `TextEdit` in every state -- `gen_info_dialog.gd`'s, with
	## `scroller=none` and `live=false` -- not zero. The claim the
	## exclusion rests on is the narrower one and it holds: no
	## `TextEdit` sits inside a live vertical scroller. Reported, not
	## changed.
	## That deadzone is not a default -- Godot's is **0**, at which the ~2 px
	## of wobble in a real thumb tap already counts as a drag and silently
	## eats the press. Without this, the fix above would trade "the sheet does
	## not scroll" for "the buttons do not press". Scaled with the rest of the
	## subtree, so it is the same physical distance in a dock (unscaled,
	## `unit` = `_phone_scale`) as in a content-scaled window (`unit` = 1.0).
	if ctl is ScrollContainer:
		(ctl as ScrollContainer).scroll_deadzone = maxi(
			PHONE_SCROLL_DEADZONE, int(round(PHONE_SCROLL_DEADZONE * unit)))

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

## **The tablet's own reach to `_touch_arbitrate()`, kept separate from the
## sizing walk above on purpose.**
##
## `tablet_fit()` floors heights and font sizes; this classifies gestures. They
## are wired apart because their blast radius is not the same: the sizing walk
## has one call site (`tool_options_row`) and widening *it* to both docks would
## re-float ~700 nodes' minimum heights and label sizes inside a fixed 400 px
## dock -- a layout change nothing in this pass measured. The arbitration
## changes `mouse_filter`, `action_mode`, `focus_mode`, `scroll_deadzone` and a
## slider's script, and moves no pixel at rest.
##
## Gated on `is_tablet()` rather than `is_touch()` for the reason `tablet_fit()`
## gives one screen up, and belt-and-braces: the phone reaches the identical
## body through `phone_fit()` with its own `unit`, and must never be walked
## twice with a different one.
##
## `unit` is 1.0 and is passed explicitly rather than defaulted, so a caller
## cannot acquire the phone's scale by omission.
func tablet_arbitrate(node: Node) -> void:
	if not DccTheme.is_tablet():
		return
	_touch_arbitrate_walk(node, 1.0)

## Idempotent by the same meta pattern `phone_fit()` and `tablet_fit()` use.
## Its own key, not theirs: a node the sizing walk has already visited still
## needs arbitrating, and vice versa.
const _TOUCH_ARB_META := "_touch_arbitrated"

func _touch_arbitrate_walk(node: Node, unit: float) -> void:
	for child in node.get_children():
		if child is Control and not child.has_meta(_TOUCH_ARB_META):
			child.set_meta(_TOUCH_ARB_META, true)
			_touch_arbitrate(child as Control, unit)
		_touch_arbitrate_walk(child, unit)

## The tablet half of `_on_phone_node_added()`, and the reason the census
## number was 224 rather than 0: every workspace panel is attached to a dock
## long after `_build_desktop_shell()` returns, so a one-shot walk at boot
## would arbitrate the chrome and none of the content. Same coalescing as the
## phone's -- one deferred pass per frame however many nodes arrive.
var _tablet_arb_pending := false

func _on_tablet_node_added(node: Node) -> void:
	if _tablet_arb_pending or not (node is Control):
		return
	var p: Node = node.get_parent()
	while p != null:
		if p == left_dock or p == right_dock:
			_tablet_arb_pending = true
			_run_tablet_dock_arbitrate.call_deferred()
			return
		p = p.get_parent()

func _run_tablet_dock_arbitrate() -> void:
	_tablet_arb_pending = false
	for dock in [left_dock, right_dock]:
		if dock != null:
			tablet_arbitrate(dock)
			## **And FIT it, which this hook did not do until 2026-09-07.**
			##
			## `tablet_fit()` ran once, from the deferred pass in
			## `register_workspace()`, so anything a REBUILD created afterwards
			## was never reached -- `_rebuild_label_panel()` and
			## `_rebuild_label_edit_form()` being the ones that kept surfacing.
			## Three consecutive batches found the same shape and fixed the
			## instances: an `edit` button at 44x29, its `x` at 27x29, a colour
			## well at 60x24, all against a 44 dp floor. **Fixing instances
			## does not stop a fitter that only runs at boot from missing the
			## next rebuild.**
			##
			## Free to hang here: the arbitration hook is already debounced
			## through `_tablet_arb_pending` and already scoped to the two
			## docks, and `_tablet_fit_walk()` marks every Control it visits
			## with `_TABLET_FIT_META` and skips it thereafter -- so this is
			## idempotent by construction and costs a walk, not a re-fit.
			##
			## Still height-only. Flooring width here would grow whatever
			## fixed-width bar contains a control, which is the self-inflicted
			## overflow `tablet_fit()`'s own header says to report rather than
			## cause -- so the `x` at 27 px WIDE stays a per-site fix.
			tablet_fit(dock)

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

## **DS-03's width half, and the only one of the three dock walks in this file
## that is not a density rule.** The owner's ruling is "keep everything, reflow
## only"; `_ds03fit_probe.gd` asserts its consequence, that no dock panel may
## force its dock wider than the dock is. `civilization/planner` did, at both
## pointer densities, and had been red in the tree unread.
##
## **The mechanism, at its symbol.** `DccWidgets.choice()` leaves
## `OptionButton.fit_to_longest_item` at Godot's default `true`, so a dropdown
## reports the width of the longest item in its **list**, not of the item it is
## **showing**. One of the planner's, `journey_planner_view.gd::_choice_field()`
## for "Desert water", shows "Auto" (59 px) and reserves its list entry
## "Established Caravan Route" (185 px) -- sampled, not established as the
## widest; the number that matters is the whole panel's, below. Measured with
## `_ldwidth_probe.gd`, 1920 x 1080, one boot:
##
##   left dock                                          404   `--ldW` = 372
##   left_dock_body                                     397   budget 365
##   the same body, `fit_to_longest_item` off on its 24
##   expanding dropdowns                                364   dock -> 372
##
## 365 is 372 minus the dock's own furniture: `_build_left_dock()` carves a
## 6 px `_dock_drag_handle()` and a 1 px right border OUT of the reserved
## width rather than adding to it. The other nine rail nodes measure 216..323
## against that 365, so the token is not tight and the planner is the outlier.
## Same probe `--force-touch` at 2560 x 1600: 455 against `W_DOCK_TABLET` 400.
##
## **Why a dock walk and not the factory.** `fit_to_longest_item` is right
## wherever a dropdown competes for width with siblings and the row may grow --
## `phone_fit()`'s `wide` branch keeps it for exactly that case, the phone tool
## sheet, where turning it off once collapsed PAINT ▸ Class onto its drop-down
## arrow (see that function's header). A dock is the opposite: the canvas draws
## it `flex:none` at a fixed `--ldW` (`ENV:25`, `ENV:304`), so a control
## reserving width it is not showing does not avoid a reflow, it *forces* one.
## The rule is therefore scoped to the two docks rather than written into
## `choice()`, which serves both shapes. `tool_options_row` is outside both
## docks and this walk never reaches it.
##
## **And only a control that expands** -- the same guard `phone_fit()` applies
## to `Button` and `Label`: one sized by its own text and not expanding has
## nothing else to take a width from, so removing its content minimum
## collapses it. All 24 of the planner's are `SIZE_EXPAND_FILL` (`choice()`
## sets it), so the guard costs nothing there and protects an ad hoc dropdown
## elsewhere.
##
## Density-independent: 372, `LAPTOP`'s 330 and `W_DOCK_TABLET`'s 400 are all
## fixed columns, and the narrow ones need this more, not less.
##
## No meta flag, unlike `phone_fit()`: that pass *multiplies* and must not run
## twice, this one writes a constant `false` and is idempotent by nature.
func dock_fit(node: Node) -> void:
	for child in node.get_children():
		if child is OptionButton \
				and ((child as OptionButton).size_flags_horizontal & Control.SIZE_EXPAND) != 0:
			var ob := child as OptionButton
			ob.fit_to_longest_item = false
			## **`fit_to_longest_item = false` on its own is not enough, and
			## the miss is a transition rather than a state.** Without
			## `fit_to_longest_item` a `Button`'s minimum tracks the text it is
			## *currently* showing, so the dock fits until the reader picks the
			## long option and then re-opens. Driven rather than reasoned
			## about: `_dockfit_probe.gd` puts every dock dropdown on its widest
			## item at once and measured the left dock at **389** against 372
			## with only the line above. Both properties below take the text
			## out of the minimum, and **each is individually sufficient** --
			## mutated separately, both survived; mutated together, the probe
			## goes red at 389 again. Both are set anyway, because that is the
			## pair `phone_fit()` already applies to every expanding `Button`,
			## of which this is one, and one vocabulary for one shape is worth
			## more than the line saved. Safe here for `phone_fit()`'s own
			## reason: the control expands, so it still draws at the row's full
			## leftover and only the overflow is trimmed.
			ob.clip_text = true
			ob.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
		dock_fit(child)

## Same shape as `_on_phone_node_added()` above and for the same reason: every
## workspace panel is rebuilt from a signal at some point after boot, so a
## one-shot pass over a dock would be correct for about a second, and
## `node_added` is the only hook that sees all of them without this file
## knowing which panels exist. Coalesced onto one deferred pass per frame.
##
## **The `Control` breadth is not load-bearing, and that is recorded so it is
## not re-derived.** Mutated to `node is OptionButton`, both density legs of
## `_ds03fit_probe` still PASS, so the narrower filter is a live option; it is
## kept wide only to match the two sibling hooks. The three parts that ARE
## load-bearing were each mutated alone and each returned the original 404
## against 372: clearing `fit_to_longest_item`, `dock_fit()`'s `SIZE_EXPAND`
## guard, and this connect.
##
## **What the fix does not reach**, so it is not mistaken for headroom:
## `_ldwidth_probe.gd` measures the planner at 364 against a 365 budget, +1
## where its nine siblings have +42..+149. 28 px of that is a second
## `ScrollContainer` nested inside the dock's own
## (`journey_planner_view.gd::_build_left_panel()`), whose scrollbar duplicates
## the dock's. Filed separately rather than bundled into a width fix.
var _dock_fit_pending := false

func _on_dock_node_added(node: Node) -> void:
	if _dock_fit_pending or not (node is Control):
		return
	var p: Node = node.get_parent()
	while p != null:
		if p == left_dock or p == right_dock:
			_dock_fit_pending = true
			_run_dock_fit.call_deferred()
			return
		p = p.get_parent()

func _run_dock_fit() -> void:
	_dock_fit_pending = false
	for dock in [left_dock, right_dock]:
		if dock != null:
			dock_fit(dock)

## Read by dialogs that have to present themselves differently on a phone
## (`open_project_dialog.gd`); `_phone`/`_phone_scale` stay private because
## nothing outside should be *setting* them.
func is_phone() -> bool:
	return _phone

func phone_scale() -> float:
	return _phone_scale

## **The route to a new world, named in the vocabulary of the composition the
## user is actually looking at.**
##
## `File ▸ New world…` is right on desktop and tablet and is drawn nowhere at
## all on a phone: `_build_phone_shell()` parks the entire `MenuBar` inside the
## permanently hidden `PhoneMenuModel` host, and `phone_menu.gd`'s `ROOT_ROWS`
## re-titles that menu **`Project`**. So every empty-state sentence carrying the
## desktop path pointed a handset user at a surface that does not exist -- and
## on a phone with no world those sentences are the *only* instruction the app
## gives, since the status bar they normally live in is parked with the menu
## bar and reaches the screen only as `phone_menu.gd`'s MORE ▸ STATUS ▸ `Next`
## row.
##
## Measured 2026-09-06, 1080x2400 windowed, dark forced
## (`_emptyphone_probe.tscn -- --force-touch --dismiss --tab more`): that row
## read `Next / File ▸ New world… to begin` on a screen whose own list has no
## File in it. The replacement is checked at the screen rather than derived --
## `--screen project` draws `New world…` as the third row of PROJECT, from
## `phone_menu.gd::_fill_project()`'s `_act(body, p, DccMenus.ID_NEW_WORLD, …)`.
##
## `static`, and off `DccTheme.is_phone()` rather than this instance's
## `_phone`, for the reason `DccTheme.set_phone()`'s own comment gives: the
## callers are windows and static widget factories that have no route to the
## shell node. `DccShell._ready()` publishes it before anything can ask.
##
## **Six call sites, counted 2026-09-06 with**
## `grep -rn 'New world… to begin' shell/ | grep -v '^[^:]*:[0-9]*:[[:space:]]*#'`
## -- two in `app.gd` (boot and `_close_world()`), three in
## `world_data_window.gd`, one in `faction_roster_window.gd`. All six phone-
## reachable: both windows call `DccWidgets.phone_present()`. Kept in one place
## so a seventh cannot drift; callers append their own trailing clause.
static func new_world_route() -> String:
	return "MORE ▸ Project ▸ New world…" if DccTheme.is_phone() \
		else "File ▸ New world…"

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
	## The counter form, upright and stacked -- `StatusBar.dc.html` panel 5,
	## which is the only drawing of this foot in the approved set. `00` over a
	## `14x1` `--div` rule over `10`, all `--m2` in `--dis`, bottom-aligned with
	## `padding-bottom:8px` and `gap:6px` in a 40 px column (`W_RAIL_COLLAPSED`).
	##
	## Built alongside the rotated label rather than replacing it: the foot also
	## carries the mode WORDS (`SCULPT`, `LANDMARKS`, `STYLE`, and
	## `journey_planner_view.gd`'s `JOURNEY`), which the artboard does not draw
	## and which the older canvas sets `writing-mode:vertical-rl`. Both forms
	## exist for the same reason -- `00 / 10` and `SCULPT` are each wider than
	## 40 px laid out flat -- and `set_rail_foot()` picks between them on the
	## shape of the string.
	_rail_foot_stack = MarginContainer.new()
	_rail_foot_stack.name = "RailFootCounter"
	_rail_foot_stack.set_anchors_preset(Control.PRESET_FULL_RECT)
	_rail_foot_stack.add_theme_constant_override("margin_bottom", RAIL_FOOT_PAD_B)
	_rail_foot_stack.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_rail_foot_stack.visible = false
	var foot_col := VBoxContainer.new()
	foot_col.alignment = BoxContainer.ALIGNMENT_END
	foot_col.add_theme_constant_override("separation", RAIL_FOOT_GAP)
	_rail_foot_stack.add_child(foot_col)
	_rail_foot_run = DccTheme.mono_label("", "text_ghost", DccTheme.FS_MICRO, 0)
	_rail_foot_run.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	foot_col.add_child(_rail_foot_run)
	var foot_rule := ColorRect.new()
	foot_rule.color = DccTheme.c("line_soft")
	foot_rule.custom_minimum_size = Vector2(RAIL_FOOT_RULE_W, DccTheme.role_px("hairline"))
	foot_rule.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	foot_col.add_child(foot_rule)
	_rail_foot_total = DccTheme.mono_label("", "text_ghost", DccTheme.FS_MICRO, 0)
	_rail_foot_total.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	foot_col.add_child(_rail_foot_total)
	foot_holder.add_child(_rail_foot_stack)
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
			## `3` before the subtitle line below existed; `1` now, so the
			## header and the sentence explaining it read as one block rather
			## than two (`design/round3-corrected/Rail.dc.html` A2, which draws
			## `10px 14px 1px` on the header and `0 14px 4px` on the subtitle).
			hp.add_theme_constant_override("margin_bottom", 1)
			hp.add_child(h)
			col.add_child(hp)
			var sub := _domain_subtitle(String(n["domain"]))
			if sub != "":
				col.add_child(_rail_header_subtitle(sub))
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

## `DOMAINS[i].subtitle` for a domain id, or `""` for an id that has none.
##
## `""` rather than a placeholder sentence, and `_build_rail_expansion()` skips
## the row entirely on it -- an empty label under a header would draw a blank
## band that reads as a subtitle that failed to load. There is no `DOMAINS` row
## without a subtitle today; the branch exists so that adding one does not
## silently ship an empty line.
func _domain_subtitle(domain_id: String) -> String:
	for entry in DOMAINS:
		var d: Dictionary = entry
		if String(d.get("id", "")) == domain_id:
			return String(d.get("subtitle", ""))
	return ""

## The one added line under a rail-expansion header
## (`design/round3-corrected/Rail.dc.html` A2/A3).
##
## **Wraps, does not clip.** `AUTOWRAP_WORD_SMART`, no `clip_text`, no
## `text_overrun_behavior`: CIVIL's subtitle is 71 characters (measured
## `Label.text.length()` by `_railfind_probe.gd` §C, 2026-09-06; A3's own
## caption says 70) and takes three lines in the 200 px desktop column, two in
## the tablet's 264 -- and the subtitle is the entire payload of
## expanding the rail -- clipping would leave the widest domain the least
## explained (A3 states exactly that). Headers therefore become unequal height.
## Node rows are untouched and keep their own uniform `_scaled(28)`.
##
## Two things this deliberately does NOT do:
##
##   - It does not set a `custom_minimum_size.x`. An autowrapping `Label`
##     reports minimum width 1 in Godot 4, so it cannot widen the enclosing
##     `ScrollContainer` -- whose horizontal axis is DISABLED, and a disabled
##     axis folds the child's minimum into the parent's own (see `MISTAKES.md`,
##     "Read a layout that overflows the screen"). Pinning a width here would
##     push the whole rail wider at the tablet's 264 as well as at 200.
##   - It does not touch the node rows' typography. Headers stay tracked mono,
##     nodes stay sans, selection stays ink alone.
##
## **Ink, measured rather than asserted.** `text_ghost` is the nearest shipped
## token to the artboard's `#565c60` (dark `#5f6468`, light `#9a9d95`) and is
## deliberately dimmer than the `text_faint` header above it, which is what A2
## draws. Against the `panel` ground this column sits on that is **3.15:1 dark /
## 2.68:1 light**, under the header's own **3.97:1 / 3.11:1** -- so neither the
## added line nor the header it explains reaches 4.5:1. The header's shortfall
## shipped long before this line; the new line is dimmer still, and that is the
## design's stated intent, not an oversight. Recorded here so the next reader
## does not have to re-derive it to know it was looked at.
##
## Size is `FS_TINY` (10), one step under the node rows' `FS_SMALL` (11), not
## the artboard's literal 11: the artboard's node row is 12 px and its subtitle
## 11, so the drawn relationship is "one step under the node label", and the
## shipped node label is 11 rather than 12.
func _rail_header_subtitle(text: String) -> Control:
	var l := DccTheme.label(text, "text_ghost", DccTheme.FS_TINY)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_theme_constant_override("margin_top", 0)
	pad.add_theme_constant_override("margin_bottom", 4)
	pad.add_child(l)
	return pad

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
##
## **Two forms, chosen on the shape of the string.** `NN / NN` -- the stage
## counter, and the only form `StatusBar.dc.html` panel 5 draws -- goes to the
## upright stack built in `_build_rail()`, its `/` becoming the artboard's 14 px
## hairline. Everything else (`SCULPT`, `LANDMARKS`, `STYLE`, `JOURNEY`) stays
## the rotated label the older canvas specifies, re-centred on every set because
## its width changes with the text.
##
## `rail_foot.text` is written in **both** cases even when that label is the
## hidden one, so anything reading the foot back reads the whole string rather
## than having to reassemble it from two halves.
##
## The counter is never blanked and never dashed. With no world it reads
## `00 / 10` and both halves are facts -- nothing has run, ten stages exist --
## which is the artboard's own binding note on panel 5. Verified at boot in
## `_statusbar4_probe.tscn`; `app.gd::_refresh_rail_foot()` is what composes it.
func set_rail_foot(text: String) -> void:
	if rail_foot == null:
		return
	rail_foot.text = text
	var halves := text.split(" / ")
	var stacked: bool = halves.size() == 2 \
		and halves[0].is_valid_int() and halves[1].is_valid_int()
	if _rail_foot_stack != null:
		_rail_foot_stack.visible = stacked
		if stacked:
			_rail_foot_run.text = halves[0]
			_rail_foot_total.text = halves[1]
	rail_foot.visible = not stacked
	if stacked:
		return
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
		## The tab can change here without any tab having been pressed -- MORE ▸
		## Workspace, `select_domain()` from a menu row, or any in-shell jump --
		## so the sheet's contents have to follow the same write, not only the
		## press. Second of the two call sites; see the function's own doc.
		_refresh_phone_gen_panel()
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
	## **The boot case, and it is the owner's own defect.** `_phone_tab` starts
	## at `"gen"` (see its declaration), so on a phone the app comes up with
	## GENERATE already lit and `_pick_phone_tab()` never called -- and
	## `_select_domain()`'s own refresh runs during shell build, before any
	## workspace is registered, so `_workspace_panels` is empty and
	## `_refresh_phone_gen_panel()` correctly declines. The result on a real
	## handset was the app booting to the one-line tool-options strip with the
	## GENERATE tab lit over it: *"one line that barely scrolls properly and a
	## lot of white space"*. Measured on the device (`9608b26b`,
	## ONEPLUS_A6013), not on a desktop probe -- the probe taps the tab, which
	## is exactly the path that hid this.
	##
	## Deferred, for `register_workspace()`'s own reason two lines below: the
	## panel arriving here is a bare `WorldWorkspace.new()` whose `setup()` has
	## not run, so filling the sheet from it now would read an empty parameter
	## table.
	_refresh_phone_gen_panel.call_deferred()
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

## **`--ctl`, the canvas's control square.** `24px` on the pointer densities
## (`ENV:25`) and `36px` on the touch one (`ENV:1819`'s `densStr`); the tablet
## canvas's own `valsShell()` writes `--ctl:36px` too, so both touch canvases
## agree and this is one of the few tokens the 2026-09-07 re-base did not split.
## Read here rather than through `role_px()` because it has no `ROLE` row --
## `dcc_theme.gd`'s token table lists `--ctl` among the four `densStr` tokens
## with none, and adding one is an owner ruling on which canvas governs the
## interior, not a lane's edit.
const CTL_PX := 24
const CTL_PX_TOUCH := 36
## The two literal paddings in the dock-header blocks. Literal `px` in the
## canvas, not tokens -- only the horizontal `var(--pad)` moves with density --
## so they do not scale.
const DOCK_HEAD_PAD_TOP := 8
const DOCK_HEAD_PAD_BOTTOM_LEFT := 0    ## `ENV:305`: `padding:8px var(--pad) 0`
const DOCK_HEAD_PAD_BOTTOM_RIGHT := 6   ## `ENV:948`: `padding:8px var(--pad) 6px`

## **A dock header's band, derived rather than pinned -- and the two docks are
## not the same height.**
##
## Both headers were built to `_scaled(34)` under a comment claiming *"the
## canvas gives both dock headers `height:34px`"*. **`height:34px` does not
## occur in the canvas at all** -- `grep -c 'height:34px'` over
## `mcp-2026-09-07/Cartalith DCC Environment.dc.html` returns **0**, and over
## its Tablet sibling too.
##
## **Where the 34 actually came from, since it was not invented.** The
## superseded `design/Cartalith DCC Shell.dc.html` draws its **menu bar** at
## `height:34px` with a `border-bottom` (its lines 222 and 440), and the comment
## said so in its own second sentence: *"the same band the menu bar and the tool
## options bar get, so the three horizontal rules across the top of the shell
## line up as one rhythm."* The menu bar was then re-based to **36** on the
## current canvas -- `dcc_theme.gd`'s `H_MENU_BAR` records `--menuH` as
## *"34 -> 36 (`ENV:25`)"* -- and the two headers that had copied it were left
## behind. `MISTAKES.md`'s re-base row is exactly this: every *relationship*
## built on the old value goes unverified, and no test sees it. What made it
## unrecoverable by reading was the citation: the figure kept pointing at "the
## canvas" after "the canvas" had become a different file.
##
## Fixing it also uncovers the half the single pasted figure hid -- the current
## canvas authors no height for either header, it authors *padding around a
## `var(--ctl)` square*, and the two paddings differ.
##
##   `ENV:305`  left  `gap:8px;padding:8px var(--pad) 0`    -> 8 + 24 + 0 = **32**
##   `ENV:948`  right `gap:8px;padding:8px var(--pad) 6px`  -> 8 + 24 + 6 = **38**
##
## and on touch, where `--ctl` is 36, **44** and **50**. `dcc_theme.gd`'s
## `TABLET` table already carried the correct reading of `ENV:305` in prose
## ("the prototype authors no height at all") while this file asserted the
## opposite four lines from the value; its `34: 52` row is now unreachable from
## here and is left for whoever owns that table to retire.
##
## Neither block carries a `border-bottom`: `ENV:305`'s is on the TOOLS block
## two siblings below (`ENV:317`) and `ENV:948`'s next sibling is the scroll
## body. The `DccTheme.rule()` both docks draw under the header is therefore a
## second inheritance from the same superseded canvas -- where every 34 px band
## *did* carry one -- measured in this pass and deliberately **not** changed:
## it is the same re-base, it wants the same ruling, and removing a rule moves
## the whole dock's rhythm.
func _dock_head_h(pad_bottom: int) -> int:
	return DOCK_HEAD_PAD_TOP + (CTL_PX_TOUCH if _touch else CTL_PX) + pad_bottom

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
	## `ENV:305` -- see `_dock_head_h()` for why this is 32 and not the 34 the
	## comment here used to claim. The sheet figure is `PHONE_TAP_MIN` and is
	## **not** an `ENV` number: the phone canvas draws no dock header at all,
	## only a bottom sheet with a 20 px grab band over a 38 px control row
	## (`Cartalith Android.dc.html:177-184`), which is a different chrome.
	head.custom_minimum_size.y = _ptap(44) if as_sheet \
		else _dock_head_h(DOCK_HEAD_PAD_BOTTOM_LEFT)
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
	## `ENV:948` -- 38, six px taller than the left dock's 32, because this
	## block's padding is `8px var(--pad) 6px` where the left's is
	## `8px var(--pad) 0`. See `_dock_head_h()`; the two were built to one wrong
	## number and the difference between them is the half that stayed hidden.
	head.custom_minimum_size.y = _ptap(44) if as_sheet \
		else _dock_head_h(DOCK_HEAD_PAD_BOTTOM_RIGHT)
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
# it as the WORLD **a / b** switch and says so — and the literal condition is in
# the prototype after all: `ldSwitch:s.domain==='WORLD'` (line 1939), so the
# canvas gates the pill on the domain being WORLD, full stop. §9.1 files that
# binding, and both segment labels, under *Lost to truncation*; **that table is
# stale, not the file.** See `_MODE_SWITCH_LABELS`' own block below for the
# measurement and the three other bindings §9.1 gives up on that are present.
#
# The code below gates on `domain_gates()` instead, which is the same answer by
# a route that survives a second domain gaining a `shows` — see the four
# derived/not-derived bullets further down. Where the canvas says *"WORLD"* and
# this file says *"any domain that gates"*, the two agree today because WORLD is
# the only gating domain; the derived form is kept because a literal `"world"`
# is exactly what a verifier caught here on 2026-09-05.
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
# **What is derived and what is not, part by part.** A verifier read this
# function on 2026-09-05 and found only the *visibility* derived while the nodes,
# the labels, the refresh's tooltip and the press all named `"world"` — so a
# future CIVIL or CARTO gate would have drawn an empty pill carrying WORLD's two
# labels. Generalised the same day. Stated per part, because "derived, not
# hardcoded" is the claim this shell has already got wrong once here:
#
# - **Derived** — *whether* the pill shows: `domain_gates()`, which asks whether
#   any of the domain's nodes carries a `shows`.
# - **Derived** — *which segments* it has: `domain_nodes(_active_domain)`, so the
#   pill is as many halves as that domain has modes, in `RAIL_NODES` order.
# - **Derived** — the lit segment, the tooltips and the press target: all read
#   `_active_domain` and `rail_node(_active_domain, mode)`.
# - **NOT derived, and deliberately — because they are QUOTED** — two of the ten
#   labels. `_MODE_SWITCH_LABELS` overrides `world/a` and `world/b` with the
#   canvas's own two strings; `mode_switch_label()` falls back to the node's own
#   `label` upper-cased for the other eight, which *is* a derivation. The
#   distinction matters and this file had it backwards until 2026-09-05: the two
#   overridden strings are the **more** faithful of the ten, not the less.
#
# **Both strings are QUOTED FROM THE CANVAS. Neither is derived, and neither is
# this port's invention.** Corrected 2026-09-05 (batch 35), because this block
# previously said the opposite and cited a spec section that is stale:
#
# | string | canvas binding | where |
# |---|---|---|
# | `SCULPT` | `ldSwB` — the pill's own segment-b caption | prototype line 1940 |
# | `PIPELINE` | `ldCollapsedLabel`'s own word for mode a, `(wm==='a'?'PIPELINE':'SCULPT')` | prototype line 1937 |
#
# `design/dcc-environment-2026-08-31/Cartalith DCC Environment.dc.html:1940`
# reads, verbatim: `ldSwA:'GENERATION PIPELINE',ldSwB:'SCULPT',`. The two lines
# under it carry `ldSwACol`/`ldSwABg` and `ldSwBCol`/`ldSwBBg`, and the line
# above carries `ldSwitch:s.domain==='WORLD'`.
#
# **What this block used to say, and why it was wrong.** It quoted
# `04-left-dock.md` §9.1 — *"the switch's own two labels are not recoverable"* —
# and concluded that `PIPELINE`/`SCULPT` was "this port's decision". §9.1's
# sixteen-row *Lost to truncation* table, and §0's *"the source file is
# truncated ... exactly 262 144 bytes (256 KiB) and ends mid-token"*, describe a
# copy of the prototype this repository no longer holds. **The frozen file is
# 239 712 bytes and 1 994 lines and ends `</script></body></html>`**; §0's own
# truncation point, `measRows.push({i:('0'+i).slice(-2),len:this.fmtKm(km),be`,
# is a complete statement at line 1863. §0's last instruction was *"Get an
# untruncated copy of this file before building"* and commit `660cbef` ("Design
# answered: the files are whole") is someone doing exactly that. **The spec
# section was never updated, and this file went on citing it.** The spec is
# frozen under `design/` and is not this file's to fix; what is fixed here is
# the claim made in this tree's own source.
#
# **`GENERATION PIPELINE` — `ldSwA` itself — is not used, and NOT because it
# does not fit.** That was the old block's reasoning ("would silently put a
# 19-character label in one half of a 372 px dock's pill") and it is false.
# Measured 2026-09-05, `_worlddockb_probe.gd` §2, headless, desktop 1920x1080,
# light palette, both pairs built through this file's own `_mode_switch_segment`
# → `DccWidgets.segment()`:
#
#   dock width                                372 px
#   shipped `PIPELINE` | `SCULPT`, minimum x  119 px
#   canvas `GENERATION PIPELINE` | `SCULPT`   185 px   — clears by 187 px
#
# It fits with room. The short form stays for a different and better reason:
# `design/proposed-2026-09-05-round2/WorldDockB.dc.html` is the canvas the owner
# approved on 2026-09-05 and it draws `PIPELINE`, and the standing rule is that
# where two canvases disagree the **newer** one wins (`CLAUDE.md`, owner ruling
# 2026-08-25). `PIPELINE` is not a compromise against `ldSwA` — it is
# `ldCollapsedLabel`'s own abbreviation of the same mode, so both halves of the
# pill are canvas words either way.
#
# The override is keyed **`domain/mode`** rather than by bare mode — `a` and `b`
# are not reserved words, and a future domain naming a mode `a` must not inherit
# WORLD's noun.
#
# **The fallback is the node's own label, and it is not free.** No domain but
# WORLD has a gate, so no other label is on screen today; if one gains one, the
# derived text is `LANDMARKS` / `FACTIONS & SETTLEMENTS` / `WAYS & ROUTES` /
# `JOURNEY PLANNER` for CIVIL (9/22/13/15 characters over four halves) and
# `LAYERS & STYLE` / `LABELS` / `ICONS` / `TERRAIN APPEARANCE` for CARTO
# (14/6/5/19). Those are legible strings, not placeholders, and whether four of
# them fit is a measurement rather than an opinion -- the pill sits in the header
# band *above* the scroll, so an overflow there has no scrollbar anywhere to
# reveal it, which is this tree's recurring class. `_leftdock12_probe.gd` §7(f)
# builds the real segments through `_mode_switch_segment()` and prints each
# band's combined minimum against the dock it would sit in, at all three
# densities. Measured 2026-09-05, band-2 minimum x against the live dock width:
#
# | density                | dock | WORLD (2) | CIVIL (4) | CARTO (4) |
# |------------------------|------|-----------|-----------|-----------|
# | desktop 1920x1080      |  372 |     153.0 | **461.0** |     365.0 |
# | tablet 2560x1600       |  400 |     211.0 | **633.0** | **508.0** |
# | phone 1080x2340, sheet | 1080 |     153.0 |     461.0 |     365.0 |
#
# Bold overflows. WORLD's shipping pill clears every density with room; CARTO's
# four derived labels clear the desktop dock by 7 px and overflow the tablet's;
# CIVIL's overflow both. (The phone row is the *authored* figure -- §7(f)'s
# scratch segments sit outside both docks, so `_on_phone_node_added()` never
# fits them; the real fitted pill is §8's.) So the answer for a domain that
# gains a gate is a shorter string in this table, which is why the table is the
# extension point rather than something to delete once the fallback exists.
var _mode_switch_row: Control
var _mode_switch_pill: HBoxContainer        ## the segment row; its children are per-domain
var _mode_switch_buttons: Dictionary = {}   ## mode -> Button, for `_mode_switch_domain` only
var _mode_switch_domain := ""               ## which domain those buttons were built for
const _MODE_SWITCH_LABELS: Dictionary = {"world/a": "PIPELINE", "world/b": "SCULPT"}

## The pill text for one rail node: the `_MODE_SWITCH_LABELS` override where one
## exists, and the node's own `label` upper-cased where none does. See the block
## above for which of the ten are which and why.
##
## Public because `_leftdock12_probe.gd` asserts the derivation for every node of
## every domain, including the eight with no pill today — a fallback nothing
## exercises is a fallback nobody has read.
static func mode_switch_label(id: String, mode: String) -> String:
	var key := "%s/%s" % [id, mode]
	if _MODE_SWITCH_LABELS.has(key):
		return String(_MODE_SWITCH_LABELS[key])
	return String(rail_node(id, mode).get("label", mode)).to_upper()

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
	_mode_switch_pill = row
	## The segments themselves are **not** built here. They belong to whichever
	## domain is active, and the active domain changes; `_refresh_mode_switch()`
	## builds them the first time it runs (from the line below, still inside this
	## function, so the pill is populated before this returns exactly as it was
	## when the loop lived here) and rebuilds them whenever the domain changes.
	_refresh_mode_switch()
	return pad

## One `flex:1` half of the pill, for one rail node of one domain.
##
## Separate from the rebuild loop so `_leftdock12_probe.gd` §7(f) can measure the
## **shipped** builder for a domain that has no pill today, rather than a replica
## of it that could drift from this one.
func _mode_switch_segment(row: Control, id: String, mode: String) -> Button:
	var b := DccWidgets.segment(row, mode_switch_label(id, mode),
		_on_mode_switch_pressed.bind(id, mode))
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
	## scale is what every other control in the dock does. A segment rebuilt
	## *after* boot is fitted too: `_on_phone_node_added()` watches `node_added`
	## for anything parented under either dock and defers one `phone_fit()` pass
	## per frame, which is the same hook every rebuilt panel row already uses.
	b.custom_minimum_size.y = 44 if (_touch and not _phone) else 24
	return b

## Replace the pill's halves with one domain's modes.
##
## Called only when the domain the buttons were built for is not the domain now
## active, so a switch WORLD → CIVIL → WORLD (CIVIL being ungated, so the pill is
## hidden and never rebuilt) leaves the same two `Button`s in place. That matters
## beyond cost: `_mode_switch_buttons` is mutated rather than reassigned, so the
## probes that take a reference to it once and press segments later keep pressing
## live buttons.
func _rebuild_mode_switch(id: String) -> void:
	_mode_switch_domain = id
	_mode_switch_buttons.clear()
	for c in _mode_switch_pill.get_children():
		## Removed *and* queued, in that order, and neither half is optional.
		## `queue_free()` alone leaves the child in the tree until idle, so the
		## pill would measure both domains' halves for a frame; a plain `free()`
		## is worse, because this function is reachable from a segment's own
		## `pressed` handler -- press → `select_domain_mode()` → `_select_domain()`
		## → `_refresh_mode_switch()` → here -- and freeing the button that is
		## mid-emit takes the process with it.
		_mode_switch_pill.remove_child(c)
		c.queue_free()
	for n in domain_nodes(id):
		var mode := String(n["mode"])
		## One half lit and the rest quiet: the state has to be legible without
		## colour alone, so the tooltip names it in words. `set_segment_on()`
		## supplies the accent ink and wash; `_refresh_mode_switch()` owns both,
		## and owns them for every half this loop makes.
		_mode_switch_buttons[mode] = _mode_switch_segment(_mode_switch_pill, id, mode)

## Show the pill for a gating domain, hide it for the rest, build its halves for
## whichever domain that is, and light whichever segment is the active mode.
## Four call sites, and each is why one of the reads below is here:
## `_select_domain()` -- the one choke point every domain and every mode change
## passes through, so the pill cannot disagree with the body beneath it;
## `apply_domain_mode()`, which writes a mode without switching domain;
## `_toggle_dock()`, which is why `_left_collapsed` is read here rather than only
## at the collapse; and `_build_mode_switch()`, which is how the pill gets its
## first set of halves.
##
## **A one-mode domain gets no pill even if it gates.** `domain_gates()` is
## satisfied by a single node carrying a `shows`, and a pill of one half is a
## control that cannot change anything -- a permanently-lit segment offering the
## state it is already in. The gate would still be honoured by the rail; what is
## suppressed is only the switch. Written as a separate clause rather than folded
## into `domain_gates()` because that function answers *"can a mode change remove
## a header here"*, which is a true and useful thing to know about such a domain
## and is what `Workspace` asks it.
func _refresh_mode_switch() -> void:
	if _mode_switch_row == null or not is_instance_valid(_mode_switch_row):
		return
	var on := domain_gates(_active_domain) and domain_nodes(_active_domain).size() > 1 \
		and not _left_collapsed
	_mode_switch_row.visible = on
	if not on:
		## Deliberately before the rebuild. A hidden pill showing another
		## domain's halves is not a defect -- nothing renders it, and
		## `_mode_switch_domain` still says whose they are, so the next refresh
		## that turns the pill back on rebuilds if and only if it has to. The
		## collapse path depends on this: `_toggle_dock()` refreshes twice around
		## a state the user never sees.
		return
	if _mode_switch_domain != _active_domain:
		_rebuild_mode_switch(_active_domain)
	var active := String(_domain_mode.get(_active_domain, ""))
	for mode in _mode_switch_buttons:
		var b: Button = _mode_switch_buttons[mode]
		if not is_instance_valid(b):
			continue
		var lit: bool = String(mode) == active
		DccWidgets.set_segment_on(b, lit)
		b.tooltip_text = "%s — %s" % [
			String(rail_node(_active_domain, String(mode)).get("label", mode)),
			"showing" if lit else "click to show"]

## Pressing a segment is exactly a rail-node press on the same node, minus the
## expansion column: same mode write, same category open, same gate. Routed
## through `select_domain_mode()` rather than writing `_domain_mode` here, so a
## future node behaviour (the Journey takeover is one already) cannot arrive for
## the rail and not for the pill.
##
## The domain is bound at build time beside the mode, not re-read from
## `_active_domain` here: the two agree for a press on a visible pill, and a
## bound pair is the version that stays right if a queued press ever arrives
## after a domain change.
func _on_mode_switch_pressed(id: String, mode: String) -> void:
	select_domain_mode(id, mode)

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
		## Its `SIZE_EXPAND_FILL` halves -- two, `PIPELINE` and `SCULPT`, in the
		## one domain that gates today, and however many the next one has --
		## carry a combined minimum width far past `W_RAIL_COLLAPSED`, and a
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
# **The step size is a DELIBERATE DIVERGENCE, not a recovery.** Corrected
# 2026-09-05: this block used to say `hTlStep`'s own step "is truncated out of
# the delivered file", so `tl_speed` was recorded as a forced choice. It is not
# truncated. Prototype line 1979:
#
#     hTlStep:e=>this.setState(x=>({tlYear:Math.min(1200,Math.max(-400,
#       x.tlYear+ +e.currentTarget.dataset.d))}))
#
# and the two buttons that call it carry `data-d="-1"` and `data-d="1"`
# (markup lines 1198-1199). **The canvas steps by exactly one year, and the
# speed pill does not touch the step buttons at all** — `tlSpeed` is consumed by
# playback alone there.
#
# `tl_step()` below steps by `tl_speed` instead, and that is kept rather than
# reverted, because a ±1 step on a 1 600-year track is 233 clicks to cross the
# span the pill claims to control, and because §4.2 places the two step squares
# immediately beside the pill and offers no other reading of what the pill is
# for. But it is now a divergence this port has chosen with the canvas in front
# of it, which is a different thing from a gap it had to fill — and if the ±1
# behaviour is ever wanted, `data-d` is where it is written down.
#   - **playback** advances the cursor by `tl_speed` years every `TL_TICK_SEC`,
#     which is §6.2's rule for the phone strip stated literally ("playing
#     advances the year by `speed` every 600 ms") and matches the desktop
#     canvas's own `tlState:s.tlRun?'RUNNING '+s.tlSpeed:'PAUSED'` (line 1978),
#     where the speed is likewise the *playback* rate. Applied to both views.
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
## `tlSpeeds` is **in** the prototype, at line 1980, and it is this ladder
## exactly: `tlSpeeds:['×1','×10','×100'].map(v=>({v,col:s.tlSpeed===v?
## 'var(--acc)':'var(--dim)',bg:s.tlSpeed===v?'var(--wash2)':'transparent'}))`.
## Corrected 2026-09-05 — this said "truncated out of the prototype ... recovered
## from the phone canvas rather than guessed". The recovery was right and the
## reason for needing one was not; `06-phone.md` §6.2 and the desktop canvas
## agree, which is a corroboration rather than a substitution.
##
## That line also settles the lit segment's colours **for the timeline pill**
## the same way `ldSwABg` settles them for the WORLD dock's: `var(--wash2)` fill
## under `var(--acc)` ink when selected, `transparent` when not — which is
## `DccWidgets.set_segment_on()`, the washed form, and not the filled amber slab
## `set_mode_segment_on()` keeps for the tool bar. The owner ruled the same way
## on 2026-09-05 (`LARGE_ITEM_RULINGS.md` §7) and recorded it as an
## interpretation open to reversal; it is not an interpretation.
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

## Every year CIVIL ▸ Politics has recorded a snapshot for, ascending.
##
## `Timeline.dc.html`'s row 2 draws one mark per entry of this array, and the
## board's own note is the reason it could be drawn at all: the marks are real
## or they are not drawn, and a tick row that can never populate is worse than
## a bare rail. `engine_bridge.gd::get_civ_timeline_years()` is a real binding
## over `WorldGen::get_civ_timeline_years`, guarded by `_has()` so a binary
## that predates the timeline milestone answers an empty array rather than
## erroring.
##
## **Empty is a state, not an error, and it is the state a fresh world is in.**
## Measured (`_tlscrub_probe.gd` §3): a completed `generate()` leaves this at
## **0** entries -- generation records no timeline year -- so board G is what
## every newly generated world draws until `civ_add_year` or a collapse
## simulation runs.
func tl_recorded_years() -> PackedInt64Array:
	var bridge := _find_engine_bridge()
	return PackedInt64Array() if bridge == null else bridge.get_civ_timeline_years()

## Where one year sits among the recorded ones: `at` when the cursor is
## standing on a recorded year, `prev` for the nearest recorded year below it,
## `next` for the nearest above.
##
## **Every key is omitted rather than defaulted.** There is no year value that
## means "none" on a -400..1200 track -- `0` and `-400` are both legal cursor
## positions and `-400` is the first frame of the axis -- so callers ask with
## `has()`. Board D's readout is exactly this dictionary rendered: `prev`
## present and `next` present prints *territory holds at 412 AD · next 500 AD*;
## board F's cursor at 1200 has no `next` and the clause is dropped rather than
## printed empty.
##
## `prev` is also the year `civ_year_diff()` diffs against: `engine_bridge.gd`
## documents that binding as diffing "against the chronologically-previous
## recorded year", so board C's *since 340 AD* is this key and not a second
## source that could disagree with the counts beside it.
func tl_year_neighbours(year: int) -> Dictionary:
	var out: Dictionary = {}
	## Ascending, so the last write below `year` is the greatest such year and
	## the first write above it is the least -- no sort and no min/max pass.
	for y in tl_recorded_years():
		var yi := int(y)
		if yi == year:
			out["at"] = yi
		elif yi < year:
			out["prev"] = yi
		elif not out.has("next"):
			out["next"] = yi
	return out

## The recorded year a shift-drag magnets to, as `{"year": int}` -- or `{}`
## when the world has recorded none, which is the state every freshly generated
## world is in.
##
## Ties go to `prev`, deliberately: dragging forward through the midpoint
## between two marks should not jump ahead of the pointer.
func tl_nearest_recorded(year: int) -> Dictionary:
	var n := tl_year_neighbours(year)
	if n.has("at"):
		return {"year": int(n["at"])}
	if n.has("prev") and n.has("next"):
		var p := int(n["prev"])
		var q := int(n["next"])
		return {"year": p if (year - p) <= (q - year) else q}
	if n.has("prev"):
		return {"year": int(n["prev"])}
	if n.has("next"):
		return {"year": int(n["next"])}
	return {}

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

## The string §4.2 binds as `{{ tlState }}`. It says which state is live and,
## when playing, at what rate -- the speed pill is a set of three and only one
## of them is what is actually happening.
##
## **The canvas spells it in upper case and this shell does not.** Corrected
## 2026-09-05: this comment said §4.2 leaves `tlState` `UNSPECIFIED`. It does
## not -- prototype line 1978 is `tlState:s.tlRun?'RUNNING '+s.tlSpeed:'PAUSED'`,
## so the older canvas's two words are `RUNNING ×10` and `PAUSED`. The lower-case
## forms below are `design/proposed-2026-09-05-round2/Timeline.dc.html`'s, which
## draws `paused` (board C), `scrubbing` (E) and `end of track` (F) in the same
## `--m1` accent slot; the newer approved canvas wins, and three states in one
## slot want a sentence voice rather than three shouted words.
##
## **A third word, added 2026-09-05 for `Timeline.dc.html` board F.** At the top
## of the track `tl_toggle_play()` refuses to start and `_tl_tick()` has already
## stopped, so `playing` is unreachable there and `paused` -- beside a play
## square this repaint draws dead -- reads as a fault rather than as the end of
## the range. `end of track` is checked first because it is the stronger claim:
## it is true whatever the run state, and the run state at 1200 is always
## stopped anyway.
##
## This is the string all **three** views of the cursor print, not just the
## desktop strip: `_refresh_phone_sim_strip()` and `phone_menu.gd`'s
## Simulation rows read the same function, and the end of the track is as true
## on the phone as it is here.
func tl_state_text() -> String:
	if tl_year() >= TL_YEAR_MAX:
		return "end of track"
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
#
# **Four states, one bar.** `design/proposed-2026-09-05/StatusBar.dc.html`,
# approved by the owner 2026-09-05 (*"I like the layouts as proposed, implement
# those"*). The spec this bar was built to gives it ONE priority message; that
# artboard draws the four states one message cannot express, and everything it
# asks for structurally is in this function:
#
#   1. one priority message                       the `pass` slot, unchanged
#   2. multi-slot, 1 px rules between neighbours  every slot below at once
#   3. autosave as a STATE, with a dot            the `autosave` cell, which
#                                                 this file had registered and
#                                                 never drawn until this change
#   4. `loaded -- no generation this session`     composed in `app.gd`
#
# and one rule across all four: **a slot with nothing to say is omitted -- never
# drawn empty, never placeholdered.** That was measured broken before this
# change. Booted with no world (`_statusbar4_probe.tscn`, first run):
#
#     [1] Label vis=true text=[] minx=1.0      <- stale
#     [4] Label vis=true text=[] minx=1.0      <- atlas
#     [6] Label vis=true text=[] minx=1.0      <- mid
#
# three empty labels drawn at a 1 px minimum, each still collecting the row's
# 18 px separation on both sides.
#
# **Where the artboard's content has no source it is named here, not minted.**
# Checked against the engine rather than assumed:
#
# - `atlas 62 % · 148 MB` -- the MB half is real (`atlas_status().bytes_text`,
#   formatted by `bake_bridge.rs::human_bytes`). **The percentage is not.**
#   `AtlasStatus` carries `chunks / bytes / deepest_level / text / finalized /
#   tile_size / world_key / root` and no cap, budget or quota anywhere, so
#   there is nothing for a percent to be a percent OF. It would need a
#   per-world atlas budget on `BakeState` first.
# - `saved 47 s ago` -- the shell holds the *clock*, not the elapsed, so the
#   slot draws `· saved HH:MM` (`app.gd::_autosave_state_text()`, from
#   `Time.get_time_string_from_system()`). A relative form needs the
#   `Time.get_ticks_msec()` of the write kept and a repaint tick to age it;
#   neither exists.
# - `timings read from the save` -- **no source at all.**
#   `EngineBridge.last_generate_ms` is assigned in exactly three places and all
#   three are `Time.get_ticks_msec() - _gen_start_msec` from a live run; nothing
#   reads a timing back out of a project archive. So state 4 draws its first
#   half -- `app.gd::_refresh_status_mid()` composes `loaded -- no generation
#   this session` verbatim -- and not its second.
# - `autosave every 4 min` -- real, **not 4, and not any one number**: the
#   interval is `DccSettings.autosave_minutes()`, `maxi(1, …)` over
#   `user://cartalith_settings.cfg`'s `autosave/minutes`, absent-key default
#   **5**, and `File ▸ Autosave interval` sets it from
#   `DccMenus.AUTOSAVE_MINUTES` = `[1, 5, 15]` plus `Off`. 4 is not on that
#   ladder and appears nowhere in the shell. `app.gd::_autosave_interval_text()`
#   reads the setting on every write of the slot, so the sentence follows the
#   armed `_autosave_timer.wait_time` rather than restating a constant.
# - `3 stages stale` -- real (`EngineBridge.stale_stages()`), though `app.gd`
#   spends the slot on the stage NAMES plus the upstream reason rather than on
#   a count.
# - `pass 4 / 6` -- real, and the one slot this change adds: `index + 1` and
#   `total` straight off `EngineBridge.generation_stage`, whose total is
#   `cartalith-engine/src/progress.rs::STAGE_COUNT` (`STAGE_NAMES: [&str; 10]`,
#   so 10 on this build). Read off the signal, never a second copy of the table.
#
# **Tokens, through `dcc_theme.gd`'s own prototype→here table** (the one above
# `DARK`), never as hex: `--acc`→`accent`, `--block`→`block`, `--dim`→
# `text_dim`, `--faint`→`text_faint`, `--dis`→`text_ghost`, `--good`→`good`,
# `--div`→`line_soft`, `--hair`→`line` (`DccTheme.panel()`'s border colour).
# Metrics: `--sbH:26px`→`H_STATUS`, `--pad:14px`→`role_px("bar_pad_x")` (14/22,
# and hardcoded 14 here until now, so the bar did not widen on a tablet), the
# rules' `1px`→`role_px("hairline")`.
#
# **The bar's own font now goes through `role_px("fs_status")`** -- 10 pointer,
# 12 tablet -- a role written for exactly this bar and, until now, with no
# *shipping* consumer: it drew at the bare `FS_SMALL` (11) in both densities
# (`git show HEAD:...dcc_shell.gd`). This comment said "zero consumers", which a
# verifier refuted on 2026-09-05: `git grep fs_status HEAD -- '*.gd'` also
# returns `_roleresolve_probe.gd`, which asserts on the role. `MISTAKES.md`'s
# "call a constant dead" row exists because probes are committed files that read
# theme tables -- grep them too. The
# artboard sets the row `font-size:var(--m2)`, i.e. **9**, which is one value in
# `dcc_theme.gd`'s `ROLE` table away and is not this file's to change.
#
# **The background is still `panel_alt`, deliberately.** The artboard draws the
# bar on `--pan`, and `dcc_theme.gd`'s own header already records that the
# prototype gives all three bars no background at all and that re-assigning
# region→token belongs to a later structural stage. Moving one of the three
# would split the set.

## The left group's fixed order, read off the four states together. Each entry
## is a slot key `set_status()` addresses by name; its cell is *omitted* -- not
## blanked -- the instant its text goes empty.
##
## `autosave` moved behind `atlas` here: the artboard's state 3 draws it
## immediately after the priority message precisely *because* stale, atlas and
## progress are all empty in that state, which is the omission rule doing the
## positioning rather than a slot reserving a place near the front.
const STATUS_SLOTS := ["pass", "stale", "atlas", "progress", "autosave"]

## Drawn past the spacer, where the artboard shows no separators: these get the
## omission rule and no rule of their own.
const STATUS_TAIL_SLOTS := ["mid", "hint"]

## `padding:0 12px` either side of each `1px` rule, so 24 px between two texts
## and 12 px of clear on each side of the hairline between them.
const STATUS_SLOT_GAP := 12

## `height:12px` on the rules, against the bar's 26. Centred, not stretched --
## `DccTheme.rule(true)` is the wrong tool twice over here: it paints `line`
## (`--hair`) where the artboard paints `--div`, and it expands to fill.
const STATUS_RULE_H := 12

var _status_cells: Dictionary = {}  ## slot -> the HBox holding its rule + label
var _status_rules: Dictionary = {}  ## slot -> that cell's leading 1 px rule
var _status_dot: Label              ## the autosave state marker
var _status_wired := false

func _build_status_bar() -> Control:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _scaled(DccTheme.H_STATUS)
	bar.add_theme_stylebox_override("panel", DccTheme.panel("panel_alt", {"top": 1}))
	status_row = HBoxContainer.new()
	status_row.add_theme_constant_override("separation", STATUS_SLOT_GAP)
	var pad := MarginContainer.new()
	var bar_pad := DccTheme.role_px("bar_pad_x")
	pad.add_theme_constant_override("margin_left", bar_pad)
	pad.add_theme_constant_override("margin_right", bar_pad)
	pad.add_child(status_row)
	bar.add_child(pad)

	## One cell per slot at the top level of `status_row`, so the row still has
	## exactly one child per slot: `app.gd::_setup_staleness()` appends its
	## Recompute button and then `move_child(_stale_recompute, 2)` to sit it
	## immediately after the `stale` readout, and that arithmetic has to keep
	## meaning what it says. It does -- 0 `pass`, 1 `stale`, so 2 is still the
	## first position after it -- and the reorder past that point (`autosave`
	## moved behind `atlas` and `progress`) is now spelled out at that call site
	## too, so neither comment can be read as describing the old order.
	for slot in STATUS_SLOTS:
		var cell := HBoxContainer.new()
		cell.name = "Slot_" + slot
		cell.add_theme_constant_override("separation", STATUS_SLOT_GAP)
		var r := ColorRect.new()
		r.color = DccTheme.c("line_soft")
		r.custom_minimum_size = Vector2(DccTheme.role_px("hairline"), STATUS_RULE_H)
		r.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		cell.add_child(r)
		_status_rules[slot] = r
		## The autosave dot rides 6 px ahead of its own text and nothing else's,
		## so it needs an inner box with its own separation rather than the
		## cell's 12.
		var host: Control = cell
		if slot == "autosave":
			var inner := HBoxContainer.new()
			inner.add_theme_constant_override("separation", 6)
			cell.add_child(inner)
			_status_dot = DccTheme.mono_label("●", "good", DccTheme.role_px("fs_status"), 0)
			_status_dot.size_flags_vertical = Control.SIZE_SHRINK_CENTER
			inner.add_child(_status_dot)
			host = inner
		var l := DccTheme.mono_label("", "text_faint", DccTheme.role_px("fs_status"), 0)
		_status_labels[slot] = l
		host.add_child(l)
		status_row.add_child(cell)
		_status_cells[slot] = cell
		cell.visible = false
	status_row.add_child(DccTheme.spacer())
	## `statusMid` (`05-right-dock-and-bars.md` §3.3, `BUILD_ANSWERS.md` §2.2):
	## `var(--dis)`, between the spacer and the key hints. Composed in
	## `app.gd::_refresh_status_mid()` -- this file only reserves the slot, the
	## same as every other one here.
	var mid := DccTheme.mono_label("", "text_ghost", DccTheme.role_px("fs_status"), 0)
	_status_labels["mid"] = mid
	mid.visible = false
	status_row.add_child(mid)
	var hint := DccTheme.mono_label("", "text_faint", DccTheme.role_px("fs_status"), 0)
	_status_labels["hint"] = hint
	hint.visible = false
	status_row.add_child(hint)
	## The bridge is added by `app.gd::_ready()` and does not exist while the
	## frame is being built, so the `progress` slot's wiring waits a frame --
	## the same deferral, and for the same reason, as `_relayout_rail_labels()`.
	call_deferred("_wire_status_progress")
	return bar

## `pass N / M` while a run is in flight (state 2's fourth slot). Its own three
## connections rather than a read off `_vp_stage`: that field is maintained
## inside `_refresh_viewport_context()`, which returns early when there is no
## `ViewportHost` -- a condition that has nothing to do with whether the status
## bar should be counting.
##
## Cleared on `generation_started` rather than set to `pass 0 / N`: there is no
## index between the start and the first tick, and the artboard's rule is to
## omit a slot with nothing to say, not to placeholder it.
func _wire_status_progress() -> void:
	if _status_wired:
		return
	var bridge := _find_engine_bridge()
	if bridge == null:
		return
	_status_wired = true
	bridge.generation_started.connect(func():
		set_status("progress", "", "text_dim"))
	bridge.generation_stage.connect(func(index: int, _name: String, total: int):
		set_status("progress", "pass %d / %d" % [index + 1, total], "text_dim"))
	bridge.generation_finished.connect(func(_ok: bool):
		set_status("progress", "", "text_dim"))

## Set one status slot. Slots: pass, stale, atlas, progress, autosave, mid,
## hint, and the menu bar's top_world / top_pass / top_cpu / top_gpu / top_mem.
##
## The `top_*` slots are menu-bar labels registered into the same dictionary and
## are deliberately left out of the omission pass: they are a different strip
## with a different layout, and hiding one would reflow the menu bar.
## **The default ink is per-strip, and until 2026-09-07 it was not.** Every
## write here stamps a `font_color` override, so a build-time token on one of
## these labels survives exactly until the first `set_status()` call -- and
## `app.gd:611` writes `set_status("top_world", "—")` during boot. The menu
## bar's readouts were therefore drawn in the status bar's ink no matter what
## `_build_menu_bar()` asked for, which is why correcting the construction site
## alone moved nothing: caught by `_canvasfig_probe.gd`, which asserts the ink
## on the LIVE label rather than at construction, four cells passing and
## `top_world` -- the one slot boot writes -- failing.
##
## The two strips are two different canvas rows and the canvas gives them
## different inks: `ENV:104` (menu bar) is `color:var(--dim)`, `ENV:1222`
## (status bar, `statusKeys`) is `color:var(--faint)`. A single default cannot
## be right for both, so the default is resolved from the slot's own strip.
## An explicit `token` argument still wins, which is what the status bar's
## severity writers (`accent` for `autosave failed`) depend on.
const _STATUS_TOP_PREFIX := "top_"

static func _status_default_token(slot: String) -> String:
	return "text_dim" if slot.begins_with(_STATUS_TOP_PREFIX) else "text_faint"

func set_status(slot: String, text: String, token: String = "") -> void:
	if not _status_labels.has(slot):
		push_error("DccShell: no status slot '%s'" % slot)
		return
	var ink: String = token if token != "" else _status_default_token(slot)
	var l: Label = _status_labels[slot]
	l.text = text
	l.add_theme_color_override("font_color", DccTheme.c(ink))
	if slot == "autosave":
		## `ink`, not `token` -- an omitted argument is now the empty string,
		## and `DccTheme.c("")` is not a colour. The dot must follow whatever
		## the label was actually painted with, which is what `ink` is.
		_paint_status_dot(ink)
	if _status_cells.has(slot) or STATUS_TAIL_SLOTS.has(slot):
		_apply_status_omission()

## The artboard's one structural rule, applied to the whole left group at once:
## a slot with no text is not drawn, and a rule is drawn only *between* two
## visible slots -- so the leading one never carries one and a gap in the middle
## never leaves a hanging hairline.
##
## Walked in `STATUS_SLOTS` order every time rather than patched per slot: the
## first-visible slot changes as neighbours fill and empty, and the state that
## proves it is state 3, where `pass` and `autosave` are the only two live and
## the rule has to land between them across three empty cells.
func _apply_status_omission() -> void:
	var seen := false
	for slot in STATUS_SLOTS:
		var cell: Control = _status_cells.get(slot)
		if cell == null:
			continue
		var on: bool = (_status_labels[slot] as Label).text != ""
		cell.visible = on
		var r: Control = _status_rules.get(slot)
		if r != null:
			r.visible = on and seen
		if on:
			seen = true
	for slot in STATUS_TAIL_SLOTS:
		if _status_labels.has(slot):
			var tl: Label = _status_labels[slot]
			tl.visible = tl.text != ""

## The autosave marker's ink follows its own sentence's severity rather than
## sitting on `--good` forever. `app.gd`'s five autosave writers -- three
## branches of `_refresh_save_status()` and two of `_autosave_tick()`, measured
## 2026-09-05 with `grep -n 'set_status("autosave"' shell/app.gd |
## grep -vc '^[0-9]*:\s*#'` -- use a
## quiet token while the autosave is healthy and `accent` when it is not
## (`autosave failed`, `unsaved changes`), and a green dot in front of
## "autosave failed" would be the readout contradicting itself.
func _paint_status_dot(token: String) -> void:
	if _status_dot == null:
		return
	var quiet := ["text_faint", "text_dim", "text_ghost", "text_secondary"]
	_status_dot.add_theme_color_override("font_color",
		DccTheme.c("good" if quiet.has(token) else token))

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
	##
	## **This node owns the phone's blank-row band, and the band is a map with
	## no world in it.** Censused 2026-09-06 by `_emptyphone_probe.tscn` at
	## 1080x2400 windowed, dark forced, blank = no pixel on the row above
	## RGB(23,23,23) -- the same rule `_ph16band_probe.gd` counts by, and the two
	## agree exactly. Per surface, no world and entry screen dismissed:
	##
	##   content gap (this node)  1 786 rows   1 556 blank   ← 87% of the screen's
	##   bottom nav bar             169 rows      88 blank
	##   status safe row             73 rows      48 blank
	##   gesture inset               52 rows      42 blank
	##   tool sheet                 173 rows      27 blank
	##   app bar                    147 rows      26 blank
	##                                        1 787 total
	##
	## Generate a world and this node goes to **0 blank**. **Three of the five
	## surfaces are unchanged** -- nav 88, status 48, app bar 26 -- and the other
	## two shrink with it: the tool sheet 27 -> 6 and the gesture inset 42 -> 11.
	## The screen totals 179, which is the arithmetic of exactly those five.
	## (An earlier draft of this comment said "every other row above is
	## unchanged", which was false for two of the five it had just tabulated.)
	## So the whole band is one surface with nothing to draw, and the residual
	## 179 is the leading inside bars that are working correctly. **It is not a
	## spacer, not a list built empty and not a container failing to collapse**
	## -- the three causes worth hunting -- so there is no row here to remove.
	##
	## **And it is not the first screen either.** A cold boot with no world runs
	## `app.gd::open_welcome()` → `phone_project_picker.gd`, which covers y
	## 0..2348 and measures **212** blank rows; the 1 787 state is reachable only
	## after the user dismisses that. On the default GENERATE tab the empty shell
	## already carries `GENERATE WORLD` as an accent-filled primary in the tool
	## sheet, so an empty-state signpost drawn into this gap would be a second
	## route to a button already on screen. Considered and not built, 2026-09-06.
	##
	## Drawn is not wired, so that last sentence is a **pressed** button, not a
	## read screenshot: `--pressgen` finds it in `_phone_tool_sheet` at
	## 392 x 126 px (48 dp tall at this scale, over the 44 dp touch floor),
	## `disabled=false`, and driving it with no world takes `bridge.has_world`
	## false → true. The one thing the empty shell was missing was a *route in
	## words* -- `set_status("hint", …)` named `File ▸ …`, a menu a handset does
	## not draw. See `new_world_route()` above.
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

## The app bar: **`[world pill] · ⌕ · ⋮`**, three cells, owner ruling 20
## (2026-09-06).
##
## `design/dcc-environment-2026-08-31/Cartalith Android.dc.html:79-88` is the
## drawing, and it is the newer of the two Android canvases, so `CLAUDE.md`'s
## "the newer canvas wins" settles it against `design/Cartalith Android
## Phone.dc.html` screen 01, which drew `☰ / title over seed / ⌕ / ⋮`:
##
##   `gap:8px;padding:4px 10px 0` over a row of
##   - the **world pill**: `flex:1;padding:8px 14px;border-radius:20px;
##     background:{{pillBg}};border:1px solid var(--hair)`, carrying
##     `{{worldName}}` (`500 11.5px Plex, letter-spacing:.2em, --ink`) over
##     `{{worldMeta}}` (`9.5px Plex, --dim`),
##   - `⌕` and `⋮`: `width:44px;height:44px;border-radius:22px` on the same
##     `pillBg` fill and `--hair` border.
##
## ## The two cells that were here and are not
##
## **`☰` (domain drawer) and `▤` (panels) are gone, and the ruling's binding
## condition was that nothing they reached goes with them.** Measured before
## deleting either, not reasoned about, by `_appbar20_probe.gd` at 1080x2400
## `--force-touch`:
##
## | pressed | reached |
## |---|---|
## | `☰` Domain panel | `left=true right=false picker=false` |
## | `▤` Panels | the panel-picker overlay (`picker=true`) |
## | picker ▸ Left panel | `left=true` |
## | picker ▸ Right panel | `right=true` |
##
## So the destination set behind the pair is exactly **{left dock sheet, right
## dock sheet}** — `▤` opened a *router*, not a destination of its own. Both
## survive, on a route the same probe drove end to end through the drawn rows:
## MORE ▸ `Window` ▸ `Left dock` flips `_left_sheet_open` false→true, and
## `Right dock` flips `_right_sheet_open`. Ground truth off the live `MenuBar`
## in the same run: `Left dock[on] · Right dock[on] · Timeline[on] · Status
## bar[on] · Domain rail[on] · …` — every row enabled, so neither is a drawn
## row with a disabled destination. `DccApp.toggle_region()` is what those two
## rows reach, and its `is_phone()` branch routes both through
## `_set_sheet_open()` rather than writing `visible` (its own comment says
## why).
##
## The panel picker itself went with `▤` in the same pass: it was a two-row
## chooser between those two sheets and `▤` was its only entry, so keeping it
## would have left the built-and-unwired surface this file's own history keeps
## finding. `MORE ▸ Window` carries both rows and one more (`Timeline`,
## `Status bar`, `Domain rail`) than the picker ever did.
##
## ## What is still owed, so it is not read as built
##
## The canvas draws this row **floating over the map** (`position:absolute;
## z-index:8`, no bar, no bottom rule) and splits the pill into a world *name*
## over `seed · status`. This keeps the `PanelContainer` bar, its
## `H_PHONE_APP_BAR` height and its bottom hairline, because the chrome column,
## `phone_content_insets()` and `_apply_phone_orientation()` are all measured
## against a bar that occupies height — that is stage 3 of the shell rebuild,
## which is what the ruling says adopting this canvas scopes. And the shell has
## no world *name* to put on the pill's first line: `ELDRA` is a literal in
## `app.gd`'s `set_status("top_world", "ELDRA · %d" % seed)`, so the pill keeps
## drawing `CARTALITH` over that slot rather than splitting a hardcoded string
## in two and presenting half of it as a world's name.
##
## ## The two cells that stayed
##
##   - **`⋮`.** `design/dcc-environment-2026-08-31/Cartalith Android.dc.html`'s
##     `hMenu` (`:89-95`, `:897`) opens a 230 dp popover carrying `Save
##     project` + `savedAt`, `Theme` + `themeLabel`, and `Close world`. All
##     three destinations already exist in this shell. It is *not* a duplicate
##     of the MORE tab: MORE is the program-menu tree, this is three
##     document-level actions, which is the split the canvas itself draws by
##     giving the phone both.
##   - **`⌕`.** `open_find_on_map()` below is the destination, on both the
##     phone (a full-width overlay, built here) and the desktop (`menus.gd`'s
##     row calls `_host.open_find_on_map()`, which this file answers with an
##     `AcceptDialog`). Drawn, not text: `⌕` (U+2315) is the one glyph
##     `dcc_icons.gd`'s own header names as missing from Plex Mono's fallback
##     chain, the same reason that file drew a `PATHS["search"]` icon instead
##     of listing it in `SYMBOLS` -- so this cell is the one
##     `_phone_bar_button()` call in this bar passing `icon_name` instead of a
##     `SYMBOLS` glyph string. **Guarded**, the same way the cell disappears if
##     `place_search.gd` is missing: no affordance with nothing behind it, in
##     either direction.
func _build_phone_app_bar() -> PanelContainer:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _ptap(DccTheme.H_PHONE_APP_BAR)
	bar.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"bottom": 1}))
	var row := HBoxContainer.new()
	## `gap:8px` and `padding:0 10px`, both the newer canvas's. They were 14 and
	## 12, which is the 2026-08-30 canvas's four-cell row -- one cell fewer has
	## more room, not less, so the numbers move with the drawing rather than
	## being left where a wider row put them.
	row.add_theme_constant_override("separation", _pscale(8))
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _pscale(10))
	pad.add_theme_constant_override("margin_right", _pscale(10))
	pad.add_child(row)
	bar.add_child(pad)

	## The world pill. A `PanelContainer` and not a bare `VBoxContainer` as
	## before: the canvas gives this cell a fill, a hairline and a 20 dp radius,
	## which is what separates it from the two glyph cells beside it now that
	## there is no `☰` opening the row.
	var pill := PanelContainer.new()
	pill.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pill.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	pill.add_theme_stylebox_override("panel",
		_phone_bar_pill(_pscale(20), _pscale(14), _pscale(8)))
	var title_col := VBoxContainer.new()
	title_col.add_theme_constant_override("separation", 0)
	title_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## `font:500 11.5px 'IBM Plex Mono';letter-spacing:.2em;color:var(--ink)`.
	## .2em of 11.5 px is 2.3 px and `spacing_glyph` is whole pixels, so 2.
	title_col.add_child(DccTheme.mono_label("CARTALITH", "text_bright", _pfont(12), 2, true))
	## Reuses the same "top_world" status slot the desktop menu bar's readout
	## cluster fills (`_wire_status()` in `app.gd` calls
	## `set_status("top_world", "ELDRA · %d" % seed)`) -- no phone-aware
	## branch needed in `app.gd` for this to stay live. The canvas's
	## `{{worldMeta}}` is `9.5px 'IBM Plex Mono';color:var(--dim)`.
	var subtitle := DccTheme.mono_label("", "text_faint", _pfont(10), 0)
	_status_labels["top_world"] = subtitle
	title_col.add_child(subtitle)
	pill.add_child(title_col)
	row.add_child(pill)

	## `⌕`. Only if the index it opens actually exists -- see this function's
	## own header and `_has_place_search()`. `place_search.gd` is a parallel,
	## concurrently-landing file; a build that races ahead of it must not draw
	## a magnifier over nothing, so this checks fresh on every call rather than
	## caching the answer from boot.
	if _has_place_search():
		row.add_child(_phone_bar_button("", "Search", func(): open_find_on_map(),
			"text", "search"))

	## `⋮`. See this function's own header. `overflow` is `⋯` in
	## `DccIcons.SYMBOLS` -- the horizontal ellipsis the *bottom bar's* MORE
	## cell traces -- so the vertical one the app bar draws is a literal, the
	## same way `GLYPH_THEME` is.
	row.add_child(_phone_bar_button(GLYPH_OVERFLOW, "More actions",
		func(): _set_phone_overflow_open(true)))
	return bar

## The canvas's `{{pillBg}}` box: `background:rgba(18,20,21,.92)` on dark and
## `rgba(251,250,247,.92)` on light, `border:1px solid var(--hair)`, rounded.
##
## `raised` is the token both of those resolve to -- it is the surface one step
## above `panel`, which is what the prototype's two literals are relative to
## their own `--pan`. Taken as a token rather than as the literal so the
## 2026-08-31 re-base's lesson holds: a baked colour cannot be remapped and
## `DccTheme.remap()` matches it to nothing.
##
## The alpha in the prototype is the map showing through a *floating* bar. This
## bar is not floating yet (see `_build_phone_app_bar()`'s "what is still
## owed"), so there is nothing behind it to show through and the fill is opaque
## -- a 0.92 alpha over an already-opaque bar would only mute the pill against
## its own parent.
func _phone_bar_pill(radius: int, pad_x: int, pad_y: int) -> StyleBoxFlat:
	var sb := DccTheme.outline("line", "raised")
	sb.set_corner_radius_all(radius)
	sb.content_margin_left = pad_x
	sb.content_margin_right = pad_x
	sb.content_margin_top = pad_y
	sb.content_margin_bottom = pad_y
	return sb

## `⋮` U+22EE. Not in `DccIcons.SYMBOLS`, which carries `⋯` (`overflow`) for
## the bottom bar's MORE cell; the two are different marks on the same canvas
## and the table is not this file's to extend. Resolves through `mono()`'s
## `SystemFont` fallback like every other entry that Plex Mono has no glyph for.
const GLYPH_OVERFLOW := "\u22ee"

## One app-bar glyph cell. `design/dcc-environment-2026-08-31/Cartalith
## Android.dc.html:86-87` draws `width:44px;height:44px;border-radius:22px` on
## the pill fill and hairline, `font:15px 'IBM Plex Mono';color:var(--body)`.
##
## **The box has a background now, and it did not before.** The 2026-08-30
## canvas drew these as bare `40x40` *layout* cells with no fill, which is what
## the previous version of this comment described; the newer canvas gives them
## the same `{{pillBg}}` box it gives the world pill, and that is the whole
## reason the three cells read as one row rather than as a title with two
## glyphs floating beside it. `PHONE_ICON_BOX` stays 40 and `_ptap()` floors it
## at the TARGETS card's 44 dp, so the drawn box is the canvas's 44 either way.
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
	## cells (`⌕` and `⋮`; `☰` and `▤` too, until ruling 20) are the app's whole
	## top bar beside the world pill, `tip` reached
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
	## `border-radius:22px` on all three drawn states, not just `normal`: a
	## square hover or press wash on a round cell is the same defect as no
	## feedback at all, one visual step milder.
	var r := _pscale(22)
	b.add_theme_stylebox_override("normal", _phone_bar_pill(r, 0, 0))
	b.add_theme_stylebox_override("focus", DccTheme.empty())
	b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft"), r))
	b.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("accent_wash"), r))
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
## resolves it: this bar takes **412's geometry and v3's content**.
##
## It had five slots when that was written -- v3's three domains plus PANELS
## (both docks) and MORE -- and has **four** now: `PHONE_TABS` is the authority
## a few lines below, and it reads MAP / GENERATE / PLAN / MORE. PANELS went
## when the four-tab bar landed and its app-bar replacement `▤` went with
## ruling 20; `MORE ▸ Window` carries both docks. `MENU` was the fifth caption
## and is `MORE` now, which is the canvas's word for that exact destination.
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
	## After the detent, not before: `_refresh_phone_gen_panel()` fills a
	## scroller whose height the detent has just set, and a fill against the
	## old height leaves the column measured for the wrong box on its first
	## frame.
	_refresh_phone_gen_panel()

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
			## **`.14` and radius `13`, from `AND:600` + `AND:1460` + `AND:31`,
			## not the `.16` / `14` this took from a candidate.** NOT the
			## `accent_wash` token either: that is 9% (`ENV:25`), the desktop's
			## active-menu wash, and it is effectively invisible behind a 14 px
			## glyph on `#121314` -- checked on the handset, the pill did not
			## read at all at that weight. `--wash` in the ANDROID palette is a
			## different figure from `--wash` in the PC one (`.14` against
			## `.09`), which is why this cannot simply read the shared token --
			## and the measured objection does not apply to `.14`.
			var box := DccTheme.flat(Color(DccTheme.c("accent"), 0.14))
			box.set_corner_radius_all(_pscale(13))
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
	## `AND:599`: `gap:3px` between the pill and the caption, not 4. The 4 came
	## from `candidates/Android Chrome B.dc.html`, which this cell is anchored
	## to in three more places below -- all three re-checked in the same pass
	## and all three moved.
	col.add_theme_constant_override("separation", _pscale(3))
	col.alignment = BoxContainer.ALIGNMENT_CENTER
	col.mouse_filter = Control.MOUSE_FILTER_IGNORE

	## `_pscale`d, not the raw 14: the main viewport has no content scale, so a
	## 14 px glyph would be 14 *physical* px -- under a millimetre on a 510 ppi
	## panel. `DccIcons.rect()` reads its own magnification off the canvas
	## transform, which here is 1, so this is the real raster size too.
	var ic := DccIcons.rect(glyph, _pscale(14), "text_dim")
	ic.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	## The glyph sits in a pill that only the ACTIVE tab fills. Lighting only
	## the caption (what the bar did before) left four labels of equal weight
	## and no sense of where you are. Empty stylebox until
	## `_refresh_phone_tabs()` fills it, so an inactive tab is byte-identical to
	## what it drew before.
	##
	## **Re-anchored 2026-09-07 from the candidate to the shipped canvas.** This
	## cited `candidates/Android Chrome B.dc.html`: `padding:5px 16px;
	## border-radius:14px; background:rgba(224,163,74,.16)`. The live
	## `Cartalith Android.dc.html` line 600 is `padding:4px 16px;
	## border-radius:13px; background:{{ t.bg }}`, and `t.bg` for the active tab
	## resolves at line 1460 to `var(--wash)` = `rgba(224,163,74,.14)` (line 31).
	## So all three of the candidate's numbers moved by one rung, and only the
	## 16 px horizontal padding survived. A candidate is not the canvas.
	var pill := PanelContainer.new()
	pill.add_theme_stylebox_override("panel", DccTheme.empty())
	pill.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	pill.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var pill_pad := MarginContainer.new()
	pill_pad.add_theme_constant_override("margin_left", _pscale(16))
	pill_pad.add_theme_constant_override("margin_right", _pscale(16))
	pill_pad.add_theme_constant_override("margin_top", _pscale(4))
	pill_pad.add_theme_constant_override("margin_bottom", _pscale(4))
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
	## Two quantities from two canvases, and they are not the same question.
	##
	## **The pill is 42 x 4**, from the newest canvas -- verified at the source
	## rather than through its transcription:
	## `design/dcc-environment-2026-08-31/Cartalith Android.dc.html` carries
	## `width:42px;height:4px;border-radius:2px;background:var(--bord)`, and
	## `spec/06-phone.md` §5.3 transcribes exactly that. It was `40x4` from
	## `candidates/Android Chrome B.dc.html` (a file that is **not in this
	## repository** -- see the detent constant block for where those two
	## citations live), and `34x4` before that from
	## `design/Cartalith Android Phone.dc.html`. The comment here used to give
	## the candidate the disagreement as "the newer canvas"; the 2026-08-31
	## import is newer than it, so that ordering is stale and the newest canvas
	## wins under `CLAUDE.md`'s first working rule. The `border-radius:2px` is
	## the one value not matched: this is a `ColorRect`, which has no corner
	## radius, and 2 px of it does not justify a `Panel` and a `StyleBoxFlat`.
	##
	## **The row stays 24 dp, and that is left alone deliberately.** §5.3's
	## grab region is not the pill's row -- it is *"the whole header block above
	## the scroller"*, which in the prototype is the 20 dp handle row plus a
	## title row (`padding:0 14px 10px` around 38 x 38 back/close buttons), so
	## roughly 68 dp of grabbable header. **This sheet has no header block**:
	## it is the desktop tool-options bar (§13), with no title, no back arrow
	## and no close ✕, so the handle row IS the grab region. Following §5.3's
	## `height:20px` literally would therefore shrink the only gesture target
	## this sheet has, which is the opposite of what §5.3 is doing.
	##
	## What no canvas settles is how tall a *headerless* sheet's grab region
	## should be, so it is not invented here. Measured 2026-09-05 by
	## `_detent_probe.gd` at 720x1600, 1080x2340 and 1440x3200: 42 / 63 / 84
	## physical px, **24.03 dp at all three** -- below the 44 dp floor, and
	## below the 48 dp Android minimum §9's own accessibility item invokes. That
	## item lists the prototype's sub-minimum targets (36 dp stage-override
	## chips, 38 dp segment chips and sculpt presets, 38 x 38 steppers, 48 x 44
	## icon cells) and does **not** name any sheet handle, because in the
	## prototype the grab region is the header block and is nowhere near the
	## limit. Filed as an open question for `DESIGN_HANDOFF.md` rather than
	## answered from this file.
	## **20, from `AND:177`** -- `<div style="height:20px;display:flex;
	## align-items:center;justify-content:center">` around the 42x4 pill. The
	## paragraph above says "what no canvas settles is how tall a *headerless*
	## sheet's grab region is"; the shipped Android canvas settles it, and the
	## 24 here was the reasoned estimate that sentence licensed.
	##
	## **`_ptap()`, not `_pscale()`, and that is the owner-reported defect,
	## 2026-09-07.** Owner, on the device: *"it seems an issue with dragging the
	## drawer up in the sculpt menu."* `_sheetgrab_probe.gd` at 1080x2340 drove
	## a ten-rung ladder of whole drags through the SubViewport's own hit-test
	## and measured the live band -- the offsets from the handle's centre that
	## actually raise the sheet -- as **exactly the grab row and nothing more**:
	## raised at -26/-12/0/+12 px, dead at +-26 px and beyond, a target
	## **19.84 dp** tall. `_detent_probe.gd` reports PASS on the same build
	## because it presses the handle's exact CENTRE; a centre press can never
	## see the width of the target it hits.
	##
	## 19.84 dp is under half the **44 dp** floor `phone_fit()` applies to every
	## other tappable thing in this shell (`DccTheme.PHONE_TAP_MIN`, the
	## canvas's own TARGETS card) and well under Android's 48 dp. The paragraph
	## further up already flagged 24.03 dp as below both and filed it as an open
	## question for `DESIGN_HANDOFF.md`; the `AND:177` read then took it to 20.
	##
	## `_ptap()` is this file's own answer to exactly this question --
	## `_pscale(maxf(DccTheme.PHONE_TAP_MIN, px))` -- so the canvas's authored
	## **20 stays the figure in the source** and the shell's tap floor is what
	## reaches the screen, the same way every other phone target here is sized.
	## The pill is a `PRESET_CENTER` `ColorRect`, so it stays 42 x 4 dp and
	## stays centred; nothing drawn moves except the invisible hit row.
	##
	## **The cost, stated rather than discovered later:** `peek` is 66 dp and is
	## unchanged, so the sheet body at `peek` goes 45.78 dp -> ~22 dp. Nothing
	## fitted there today anyway -- `tool_options_row` measures 48 dp against a
	## 45.78 dp viewport, i.e. it was already clipped and already relying on the
	## body's `SCROLL_MODE_AUTO`. `peek` is a sliver by design ("still there,
	## out of the way"), and at a sliver the handle is the part that has to
	## work.
	_phone_sheet_grab.custom_minimum_size.y = _ptap(20)
	_phone_sheet_grab.mouse_filter = Control.MOUSE_FILTER_STOP
	_phone_sheet_grab.gui_input.connect(_on_phone_sheet_grab_input)
	var handle := ColorRect.new()
	## Token-derived, not a literal white: `DccTheme.remap()` can only repaint a
	## colour it can trace back to a token, so a flat `Color(1,1,1,0.25)` here
	## would stay white when the palette goes light and vanish into the panel.
	## The candidate's `rgba(255,255,255,.25)` is therefore matched in *weight*
	## rather than in value -- `text_ghost` at the alpha this handle already
	## carried, unchanged.
	##
	## **The objection above is answered rather than overruled, 2026-09-07.** It
	## read: the newest canvas paints this pill `var(--bord)`, *"but an alpha on
	## a shared token is a relationship, not a value … re-weighting it is a
	## contrast change that has to be computed for light and dark both. That is
	## a token pass, not a detent pass."* True of a hand-written
	## `Color(1,1,1,0.16)`, which is what was being weighed. It is not true of
	## the **token**: `DccTheme.DARK["border"]` is already `Color(1,1,1,0.16)`
	## and `LIGHT["border"]` is `Color(0,0,0,0.20)` -- character for character
	## the two values `Cartalith Android.dc.html` gives `--bord` (line 31 dark,
	## line 1469 light). So both palettes are computed, by the canvas, and
	## reading the token keeps `remap()` working, which was the whole reason a
	## literal was refused.
	##
	## `AND:177` is the row this pill sits in and it is unambiguous:
	## `height:20px … width:42px;height:4px;border-radius:2px;
	## background:var(--bord)`.
	handle.color = DccTheme.c("border")
	## 42 x 4, the newest canvas's own figure -- see the grab row above for the
	## three-way provenance and why the row's height did not move with it.
	var hw := _pscale(42)
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
	_phone_tool_scroll = scroll

	## **The GENERATE sheet, and the defect it closes.**
	##
	## Owner, 2026-09-07, on a OnePlus 6T: *"I don't have sliders or fields to
	## change parameters on how the world is generated … I can pull them up but
	## there is one line that barely scrolls properly and a lot of white
	## space."* That line is `tool_options_row` above -- one `HBoxContainer` --
	## and the sliders were never in this sheet at all. `_pick_phone_tab("gen")`
	## calls `_pick_bar_domain("world")`, which is `_select_domain("world")`,
	## which decides what the LEFT DOCK would show; on a phone `left_dock` is a
	## sheet with `visible = false` and the only two openers are
	## `DccApp.toggle_region(ID_WIN_LEFT)` (MORE ▸ Window) and
	## `PhoneMenu._open_left_sheet()`. The bottom bar reaches neither, so the
	## generation parameters were unreachable by tapping.
	##
	## Fixed here rather than by teaching GENERATE to open the left sheet,
	## because the canvas puts this content **in the sheet**
	## (`design/Cartalith-Android-2026-09-07.dc.html`, `tabIsGen`): a column of
	## collapsible parameter groups, not a full-screen dock. A second column in
	## the same sheet, swapped for the tool-options row, is what that draws.
	##
	## Empty and hidden until `_refresh_phone_gen_panel()` fills it, which it
	## does from `WorldWorkspace.build_phone_generate()` -- the workspace owns
	## the content because it is the one object that already reads
	## `bridge.param_*`, and duplicating that read here is the "second
	## parameter table" defect this project keeps finding.
	_phone_gen_scroll = ScrollContainer.new()
	_phone_gen_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_phone_gen_scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	_phone_gen_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_phone_gen_scroll.add_theme_stylebox_override("panel", DccTheme.empty())
	_phone_gen_scroll.visible = false
	var gen_pad := MarginContainer.new()
	## `padding:2px 14px 24px` -- the prototype's own sheet-body padding.
	gen_pad.add_theme_constant_override("margin_left", _pscale(14))
	gen_pad.add_theme_constant_override("margin_right", _pscale(14))
	gen_pad.add_theme_constant_override("margin_top", _pscale(2))
	gen_pad.add_theme_constant_override("margin_bottom", _pscale(24))
	gen_pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## Inert padding, and the one node of the GENERATE column that
	## `WorldWorkspace._pg_open_gestures()` cannot reach -- it walks down from
	## the column, and this is the column's parent. `IGNORE` for the same reason
	## everything inert in there is `IGNORE`: a touch drag has to reach this
	## `ScrollContainer` or the sheet only scrolls from its outer gutter.
	gen_pad.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_phone_gen_scroll.add_child(gen_pad)
	_phone_gen_col = VBoxContainer.new()
	_phone_gen_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_phone_gen_col.add_theme_constant_override("separation", _pscale(12))
	gen_pad.add_child(_phone_gen_col)
	col.add_child(_phone_gen_scroll)
	return sheet

## Show the GENERATE column instead of the tool-options row, or the other way
## round, and (re)fill it.
##
## Called from `_pick_phone_tab()` and from `_select_domain()`, which are the
## two places the answer can change -- a domain picked from MORE ▸ Workspace or
## from an in-shell jump lights the GENERATE tab through
## `_phone_tab_for_domain()` without ever passing through a tab press.
##
## `has_method` rather than a cast: `DccShell` is instantiated bare by probes
## with no workspaces registered at all, and `_workspace_panels` is empty then.
func _refresh_phone_gen_panel() -> void:
	if _phone_gen_scroll == null or not is_instance_valid(_phone_gen_scroll):
		return
	var on: bool = _phone and _phone_tab == "gen"
	var ws: Node = _workspace_panels.get("world")
	if on and (ws == null or not ws.has_method("build_phone_generate")):
		on = false
	_phone_gen_scroll.visible = on
	if _phone_tool_scroll != null and is_instance_valid(_phone_tool_scroll):
		_phone_tool_scroll.visible = not on
	if not on:
		return
	if _phone_gen_col.get_child_count() == 0:
		ws.call("build_phone_generate", _phone_gen_col)
	else:
		ws.call("refresh_phone_generate")

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
##
## **Every way in, enumerated from the call sites rather than from memory** --
## the sheet is a gesture surface, and the failure that matters on one is not a
## wrong height but a height with no way back out of it. Walked 2026-09-05 and
## exercised end to end by `_detent_probe.gd`:
##
## | Route | Reaches | Leaves by |
## |---|---|---|
## | boot / rotation into portrait -- `_apply_phone_orientation()` → `_snap_phone_sheet(false)` | whatever `_phone_detent` already held; `peek` on a cold start | any row below |
## | `_pick_phone_tab()`, any workspace tab tapped while at `peek` -- the lit one included, since the `elif` catches it | `half` | re-tap, or drag |
## | `_pick_phone_tab()`, the LIT workspace tab re-tapped while ABOVE `peek` | `peek` | tab tap, or drag |
## | `_on_phone_sheet_grab_input()` release | nearest of the three | tab tap, or drag |
## | `_on_phone_sheet_grab_input()` release under `PHONE_DETENT_DISMISS` | `peek` | tab tap, or drag |
## | `set_phone_detent()` -- `menus.gd`'s `_apply_layout()`, `Window ▸ Layouts` | any of the three | tab tap, or drag |
##
## Two things fall out of that table and neither is a defect, so both are
## written down instead of fixed:
##
## 1. **`full` has exactly one entrance a finger can take** -- the drag. No tab
##    tap reaches it (§5.2's `hTab` only ever lifts `peek` to `half`, and this
##    shell follows it), so if the drag ever regresses, `full` becomes
##    unreachable *silently*: every other detent still works and nothing
##    fails. That is what `_detent_probe.gd` asserts through real hit-tested
##    routing rather than by calling the handler.
## 2. **No detent can strand anyone.** Every row's exit column is a control
##    that is on screen at that detent: the bottom bar is a *sibling above*
##    which no height can cover (see the constant block), so the tab route is
##    live at `full` as much as at `peek`.
##
## Landscape has no rows at all: `_on_phone_sheet_grab_input()` returns
## immediately, `_snap_phone_sheet()` returns immediately, and the grab handle
## is hidden -- `_apply_phone_orientation()` sets the drawer's width instead.
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
##
## **What else a detent change moves: nothing.** That is the claim the sentence
## above makes structurally, and it was measured rather than reasoned from the
## node tree -- `_detent_probe.gd` records the global rect of the app bar, the
## bottom nav, the timeline, both dock sheets and the phone menu, drives
## `peek → full`, and re-reads all six. At 720x1600, 1080x2340 and 1440x3200,
## 2026-09-05, every one came back `moved=false`. The map gap absorbs the whole
## delta, which is why `phone_insets_changed` only has to reach
## `ViewportHost`'s floating chrome and not the chrome column.
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
	## `AND:605`: `width:112px;height:4px;border-radius:2px;background:var(
	## --bord)`. `border` is that token in both palettes (`DccTheme.DARK` /
	## `LIGHT`, `#ffffff29` / `#00000033`), so this is the canvas value and is
	## still token-derived -- see the tool sheet's own handle for the argument
	## that a literal white would break `remap()`, and for why reading the
	## token is not the same thing as writing the literal.
	handle.color = DccTheme.c("border")
	var hw := _pscale(DccTheme.W_PHONE_GESTURE_HANDLE)
	var hh := _pscale(4)
	handle.set_anchors_preset(Control.PRESET_CENTER)
	handle.size = Vector2(hw, hh)
	handle.position = Vector2(-hw / 2.0, -hh / 2.0)
	handle.mouse_filter = Control.MOUSE_FILTER_IGNORE
	wrap.add_child(handle)
	return wrap

# -- Phone overlays: search, overflow, dock sheets ------------------------
#
# None of these states are pictured in the mockup -- it ships exactly
# one static screen, chrome closed. Their *triggers* (`⌕`/`⋮`, and `MORE ▸
# Window ▸ Left dock`/`Right dock` for the two sheets) and their *destination*
# (the reused dock/menu-bar/status-bar content) are spec'd;
# the overlay presentation itself is this file's own construction, built to
# the same visual language (colour tokens, hairlines, Plex Mono) as
# everything else in `DccTheme`/`DccWidgets` rather than invented from
# scratch. Said plainly because the rest of this file can cite a mockup line
# for nearly every choice, and these can't.
#
# This heading and that trigger list both read `panel picker` and `☰/▤/⋯`
# until ruling 20 removed the picker and its glyph -- see the tombstone above
# `_close_all_phone_overlays()`.

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
	## "hover" two lines down is a real `line_soft` fill; flat drew neither it
	## nor the (already-empty, so harmless either way) "normal" state.
	b.flat = false
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

## Two phone surfaces were here and are **deleted**, in two passes a fortnight
## apart, and both for the same reason: a second, differently-shaped list of
## destinations the phone already reaches.
##
##   - **The `☰` domain drawer** -- a 300 dp side sheet listing the three
##     `DOMAINS` with their subtitles, plus `_pick_drawer_domain()` and
##     `_set_drawer_open()` -- went on 2026-08-25 with the 412 dp migration.
##     `design/Cartalith Android Phone.dc.html` draws no drawer at any level;
##     its `02 Domain` screen is a full-screen drill with a `←`, which is what
##     this shell's full-height left dock sheet already is. The three domains
##     the drawer listed are the bottom bar's own first three cells.
##   - **The `▤` panel picker** -- a bottom sheet with two rows, `Left panel`
##     and `Right panel`, each opening the matching dock sheet -- went on
##     2026-09-06 with owner ruling 20, which took `☰` and `▤` off the app bar
##     (see `_build_phone_app_bar()`'s header for the drawing and for the
##     reachability measurement that had to pass first). The picker was a
##     router, `▤` was its only entry, and `MORE ▸ Window` already carries
##     `Left dock` and `Right dock` as drawn, enabled rows -- driven end to end
##     by `_appbar20_probe.gd`, which reads `_left_sheet_open` /
##     `_right_sheet_open` back off the shell after tapping each one.
##
## `_set_sheet_open()` is untouched by either deletion and still has **seven**
## call sites in three files -- `grep -rn "_set_sheet_open(" --include=*.gd
## shell/ | grep -v "func _set_sheet_open" | grep -v ":[0-9]*:##"`, 2026-09-06:
## the two sheet close buttons here, `PhoneMenu._open_left_sheet()`, and four
## in `DccApp.toggle_region()` (its `ID_WIN_RESET` branch closes both sheets,
## and its two phone rows toggle one each).

# -- Phone overlay state ---------------------------------------------------
#
# Every phone overlay is mutually exclusive -- opening any one closes all the
# others, including a dock sheet -- so there is exactly one state variable per
# overlay and one shared teardown rather than a general stack. `PhoneMenu` keeps
# its own drill stack inside itself; from out here it is one more overlay.

func _close_all_phone_overlays() -> void:
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
	## with the Layers sheet up, opening the then-`☰` left-dock overlay left
	## **both** visible. (`☰` went with ruling 20; the same overlap reaches the
	## same sheet from `MORE ▸ Window ▸ Left dock` now, so the fix below is not
	## a fix for a route that no longer exists.)
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

## Shows the `⋮` popover built by `_build_phone_overflow()`. **Not to be
## confused with `_set_overflow_open()`**, which opens `PhoneMenu`'s L2 root
## and keeps that name for `_shot_phone.gd --overflow`; see its own comment for
## what mistaking the two already cost. This one is the only thing that makes
## `_phone_overflow_pop` visible, so it is also the only thing that gives its
## three rows a layout pass -- a probe that wants to measure them has to call
## it by name.
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
#
# **`CommandIndex` is on the permanent side of that line, not this one.**
# `shell/command_index.gd` shipped long before this section and is already
# named as a static type by `_vfy_batch0905_probe.gd`, so `SEARCH_SCOPES`' `.`
# scope reaches it by class name with no `ResourceLoader` dance. What it did
# NOT have until this pass was a consumer: every construction of one was a
# probe. See `_ensure_command_index()`.
#
# The four additions the round-3 artboard asks for
# (`design/round3-corrected/FindOnMap.dc.html` B2) all live in this section:
# scope prefixes (`SEARCH_SCOPES`, `_split_search_scope()`), the `.` scope
# (`_ensure_command_index()`), band headers (`PLACE_BANDS`/`COMMAND_BANDS`,
# drawn by `_fill_search_results()`) and the count (`_search_count_text()`).
# The dialog itself is unchanged: B3's recommendation is to keep it, and a
# status-bar locator is a separate change to a different surface.

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
##
## **`query` is the residual, never the raw field.** Since the scope prefixes
## landed this is called only from `_search_result_set()`, which has already
## split `lb ald` into a scope and `ald`; handing it the raw text would search
## for the prefix token as if it were part of the name. Scope filtering happens
## in the caller, on `entity`, so this function still returns the whole matching
## set and `size()` still means the whole index.
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

## Drops both cached indexes -- called whenever an overlay/dialog opens, so a
## world that regenerated since the last search is never searched stale. Cheap
## to over-call: an unopened search never rebuilds anything.
##
## **Both**, not just the place index the name still says: the `.` scope's
## `CommandIndex` goes stale the same way and worse. A place row goes wrong when
## a settlement is added; a command row's `available`/`why` change with the
## world state every `about_to_popup` handler in `menus.gd` reads, so a cached
## command index would report a command as unavailable after the thing it waits
## for arrived. The name is kept because `_set_search_open()` and
## `_open_desktop_find_on_map()` both cite it by it.
func _reset_place_search_cache() -> void:
	_place_search_index = null
	_command_index = null

# -- Scopes ---------------------------------------------------------------------
#
# `design/round3-corrected/FindOnMap.dc.html` B2, addition 1.
#
# **The token before the first space, matched WHOLE.** `_split_search_scope()`
# compares with `==` against a whole `prefix`, never `begins_with`, which is the
# entire reason `l` and `lb` can coexist -- a prefix test would make every `lb`
# query a landmarks query. There is no ambiguity to resolve and no ordering
# dependency in this array.
#
# A row carries `entity` (filter `PlaceSearch` rows on that field), or
# `commands` (search `CommandIndex` instead), or `why` (declined -- drawn, and
# explained, but returning nothing). **Exactly one of the three**, and the key
# is ABSENT rather than empty on the other two, so `has()` decides and no caller
# has to know what an empty `entity` would have meant.
## The prefix grammar, in one sentence, on both fields. Written out rather than
## generated from `SEARCH_SCOPES` on purpose: a generated list would read
## "s settlements, f factions, ..." and say nothing about the space, which is
## the part that decides whether a token is a scope at all.
const SEARCH_HINT := "Start with a scope and a space to narrow: s settlements, f factions, lb labels, r routes, . commands."

const SEARCH_SCOPES: Array = [
	{"prefix": "s", "label": "settlements", "entity": "settlement"},
	{"prefix": "f", "label": "factions", "entity": "faction"},
	{"prefix": "lb", "label": "labels", "entity": "label"},
	{"prefix": "r", "label": "routes", "entity": "route"},
	{"prefix": ".", "label": "commands", "commands": true},
	## **Declined, in the code's own words.** `place_search.gd`'s header
	## declines icons because `icon_dict` (lib.rs:5939-5948) carries
	## `family`/`slot`/`set`/`scale` -- four closed vocabularies and no
	## identifying name -- so a row reading "pine / forest" is not a place
	## anyone typed a name to find. Drawn disabled rather than omitted because
	## a searcher who expects landmarks deserves to be told why there are none,
	## and typing `l ` says the same thing where the result list would be.
	{"prefix": "l", "label": "landmarks",
		"why": "Placed landmarks carry family / slot / set / scale — four closed vocabularies, no name. Nothing to type to find one."},
]

## The band a returned row fell in, recomputed here rather than re-ranked.
##
## `PlaceSearch.search()` returns `prefix + name_hit + other`, and
## `CommandIndex.search()` returns `title + group + blurb`; both concatenate
## three already-sorted lists and neither tells the caller where the seams are.
## These two functions restate each ranker's own test so the seams can be drawn.
##
## **The rows never move.** `_fill_search_results()` walks the returned order and
## emits a header only where this index changes, so if one of these ever drifts
## from the ranker it wrote itself against, the visible failure is a band header
## appearing twice -- not a list in a different order from the one `search()`
## chose. That is the whole reason the grouping is computed instead of re-sorted.
const PLACE_BANDS: Array = ["STARTS WITH", "CONTAINS", "MATCHED ON KIND OR SUBTITLE"]
const COMMAND_BANDS: Array = ["MATCHED THE NAME", "MATCHED THE MENU", "MATCHED THE DESCRIPTION"]

static func _place_band(row: Dictionary, needle: String) -> int:
	var at := String(row.get("name", "")).to_lower().find(needle)
	if at == 0:
		return 0
	return 1 if at > 0 else 2

static func _command_band(row: Dictionary, needle: String) -> int:
	if String(row.get("title", "")).to_lower().find(needle) >= 0:
		return 0
	return 1 if String(row.get("group", "")).to_lower().find(needle) >= 0 else 2

## Splits a raw field value into `{scope?, query}`. `scope` is ABSENT unless the
## token before the first space is exactly one of `SEARCH_SCOPES`' prefixes --
## so `lb ald` scopes to labels, `l ald` scopes to the declined landmarks row,
## and `ald` (no space at all, or an unrecognised token) searches everything for
## the whole string, `s` and `lb` included.
func _split_search_scope(raw: String) -> Dictionary:
	var at := raw.find(" ")
	if at < 0:
		return {"query": raw.strip_edges()}
	var token := raw.substr(0, at)
	for entry in SEARCH_SCOPES:
		var sc: Dictionary = entry
		if String(sc["prefix"]) == token:
			return {"scope": sc, "query": raw.substr(at + 1).strip_edges()}
	return {"query": raw.strip_edges()}

## `CommandIndex` for the `.` scope, built lazily. This dialog is its FIRST
## shipping consumer: before this pass the only things that ever constructed one
## were probes (`_cmdindex_probe.gd`, `_cmdunavail_probe.gd`, `_perfwin_probe.gd`,
## `_apcmdcheck_probe.gd`, `_idxfind_probe.gd`, `_vfy_batch0905_probe.gd`) --
## `grep -rn 'CommandIndex' godot-project --include=*.gd`, 2026-09-06, no
## `shell/` hit outside comments. Its API, read off `command_index.gd` rather
## than assumed: `CommandIndex.new()`, `build(app, bridge)`, `size()`, `all()`,
## `search(q)`, `groups()`, each row `{title, blurb, group, kind, available,
## why, key}`.
##
## `self` as the `app` argument because `DccApp extends DccShell`: `build()`
## walks `_gather_menu_buttons(app, ...)` for `MenuButton`s, and `add_menu()` in
## this very file is what parents them. A bare `DccShell` has no menus and no
## bridge and yields an empty index rather than failing -- the same degradation
## every other engine touch in this file already takes.
func _ensure_command_index() -> CommandIndex:
	if _command_index == null:
		var idx := CommandIndex.new()
		idx.build(self, _find_engine_bridge())
		_command_index = idx
	return _command_index

## Everything one keystroke produces, in one dictionary, so the two surfaces
## render from the same computation instead of two.
##
## Keys: `rows` (what to draw) and `total` (the size of the pool they were drawn
## from, for the count) always; `query` (the residual after a scope token);
## `scope` only when one was recognised; `commands` only when the rows are
## `CommandIndex` rows rather than `PlaceSearch` rows; `note` only when a
## declined scope was typed and the reason is what should be shown instead of a
## result list. Absent, never empty -- callers use `has()`.
func _search_result_set(raw: String) -> Dictionary:
	var split := _split_search_scope(raw)
	var query := String(split["query"])
	var out := {"query": query}
	if split.has("scope"):
		out["scope"] = split["scope"]
	var scope: Dictionary = split.get("scope", {})

	if scope.has("why"):
		out["rows"] = []
		out["total"] = 0
		out["note"] = String(scope["why"])
		return out

	if scope.has("commands"):
		var idx := _ensure_command_index()
		out["rows"] = idx.all() if query == "" else idx.search(query)
		out["total"] = idx.size()
		out["commands"] = true
		return out

	var rows: Array = _run_place_search(query)
	if scope.has("entity"):
		var want := String(scope["entity"])
		var kept: Array = []
		for r in rows:
			if String((r as Dictionary).get("entity", "")) == want:
				kept.append(r)
		rows = kept
	out["rows"] = rows
	## The whole index, not the filtered pool: "7 of 342" says seven of the
	## world's three hundred and forty-two indexed places, which is the sentence
	## a scope chip makes interesting. `size()` is `PlaceSearch`'s own, so an
	## absent index (no `place_search.gd`, no bridge) reports 0 rather than a
	## guess.
	out["total"] = int(_place_search_index.call("size")) if _place_search_index != null else 0
	return out

## Renders a result set into `container` as `_phone_list_row()` rows (used on
## BOTH surfaces -- see `_open_desktop_find_on_map()`'s own comment for why
## reusing the phone row on desktop is deliberate here, not an oversight), with
## a band header wherever the ranking's own three bands change over.
##
## **A `.` command row is drawn inert, and that is the code's state, not a
## styling choice.** A `CommandIndex` row is `{title, blurb, group, kind,
## available, why, key}` -- no popup, no item id, no `Callable`, and no `x`/`y`
## -- so nothing in this shell can dispatch a menu command from one, the way
## `_select_search_hit()` can pan to a place. The row therefore says where the
## command lives (its `group`, i.e. the menu it is under) and, when it is
## unavailable, `why`; it does not offer a press that would do nothing.
func _fill_search_results(container: VBoxContainer, res: Dictionary, close_fn: Callable) -> void:
	for child in container.get_children():
		child.queue_free()
	if res.has("note"):
		container.add_child(_search_notice(String(res["note"])))
		return
	var rows: Array = res.get("rows", [])
	if rows.is_empty():
		container.add_child(_search_notice("No matches"))
		return
	var commands: bool = res.has("commands")
	var needle := String(res.get("query", "")).to_lower()
	var bands: Array = COMMAND_BANDS if commands else PLACE_BANDS
	var last_band := -1
	for row in rows:
		var d: Dictionary = row
		## Bands only describe a match, so an empty query -- which returns the
		## index in build order, unranked -- has none to draw.
		if needle != "":
			var band := _command_band(d, needle) if commands else _place_band(d, needle)
			if band != last_band:
				container.add_child(_search_band_header(String(bands[band])))
				last_band = band
		if commands:
			var why := String(d.get("why", ""))
			var line2: String = why if why != "" else "%s menu" % String(d.get("group", ""))
			var cmd := _phone_list_row(String(d.get("title", "")), line2, func(): pass)
			(cmd as Button).disabled = true
			(cmd as Button).accessibility_description = \
				"%s. Lives under the %s menu." % [line2, String(d.get("group", ""))]
			container.add_child(cmd)
			continue
		var kind := String(d.get("kind", ""))
		var sub := String(d.get("subtitle", ""))
		var line2 := "%s — %s" % [kind, sub] if kind != "" and sub != "" else kind + sub
		container.add_child(_phone_list_row(String(d.get("name", "")), line2,
			_select_search_hit.bind(d, close_fn)))

## One band header inside the result list. Tracked mono caps in `text_faint`,
## the same vocabulary `DccWidgets.section()` uses for a dock section and the
## rail expansion uses for a domain header -- a divider, not a row.
func _search_band_header(text: String) -> Control:
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _pscale(14))
	pad.add_theme_constant_override("margin_right", _pscale(14))
	pad.add_theme_constant_override("margin_top", _pscale(9))
	pad.add_theme_constant_override("margin_bottom", _pscale(3))
	pad.add_child(DccTheme.mono_label(text, "text_faint", _pfont(9), 2, true))
	return pad

## "No matches", or a declined scope's reason, in the space the rows would have
## taken. Wraps: a reason is a sentence, not a label.
func _search_notice(text: String) -> Control:
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _pscale(14))
	pad.add_theme_constant_override("margin_right", _pscale(14))
	pad.add_theme_constant_override("margin_top", _pscale(12))
	var l := DccTheme.label(text, "text_faint", _pfont(10))
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	pad.add_child(l)
	return pad

## The scope chips (`FindOnMap.dc.html` B2). Rebuilt wholesale on every render
## rather than repainted, because `DccWidgets.chip()` bakes `accent` into four
## styleboxes at construction and there is no setter for it afterwards -- six
## buttons is cheaper than the result list this is rebuilt alongside.
##
## Pressing the active chip clears the scope instead of re-applying it, so a
## chip is a toggle and there is no state a pointer cannot get out of.
func _fill_scope_chips(row: HFlowContainer, res: Dictionary) -> void:
	for child in row.get_children():
		child.queue_free()
	var scope: Dictionary = res.get("scope", {})
	var active := String(scope.get("prefix", "")) if not scope.is_empty() else ""
	var query := String(res.get("query", ""))
	for entry in SEARCH_SCOPES:
		var sc: Dictionary = entry
		var p := String(sc["prefix"])
		var on := p == active
		var declined := sc.has("why")
		var press := Callable() if declined else _set_search_query.bind(
			query if on else "%s %s" % [p, query])
		var b := DccWidgets.chip(row, "%s %s" % [p, String(sc["label"])], press,
			on and not declined, _pscale(6), _pscale(2))
		b.add_theme_font_size_override("font_size", _pfont(9))
		## **§13's tap floor, phone only -- and it was missing for a day.**
		## These chips shipped 2026-09-06 at **43 x (147..219) physical px** on
		## a 1080 x 2340 / 412 dp handset (`_phone_scale` 2.6214), and
		## `_phonechrome_probe.gd`'s tap-floor walk was printing
		## `no tap-floor violations got=6` the whole time. They escape
		## `phone_fit()`'s own floor because the search overlay is not a dock
		## subtree -- it is hand-sized in `_ptap()`/`_pscale()` the way
		## `_phone_list_row()`'s `_ptap(52)` is -- so the floor has to be
		## written here, at the one place this shell builds them.
		##
		## **Both axes, and the width half is a no-op today rather than a
		## guess.** Measured at both densities: the narrowest chip (`r routes`)
		## is 147 px against the 115 px floor at 1080 px, and 56 dp against 44
		## at 412 dp, so `custom_minimum_size.x` never binds and the wrap is
		## unchanged -- 5 chips then 1, two lines, at both. Stated as a mutation
		## rather than as an intention: replacing this line with
		## `Vector2(0.0, tap)` leaves `_phonechrome_probe.gd` §4a byte-identical
		## (all six chips still `*x115`, still two lines, still six result
		## rows), while `Vector2(tap, 0.0)` turns two of its assertions red.
		## It is set anyway because that is the shape `phone_fit()` settled on
		## for every other tappable control in this shell (see its "Both axes
		## unconditionally" block: the `if min_size.x > 0.0` guard there was the
		## whole cause of 174 under-floor controls, not a contributing one), and
		## a scope added with a shorter label would otherwise be small again in
		## silence. **So no probe covers the width half, and cannot** -- there
		## is no chip narrow enough to make it bind.
		##
		## **The declined `l` chip is floored too, and the reason is not
		## symmetry.** Exempting it measures 65 dp for the row instead of 92 --
		## a real saving, and still rejected: `FindOnMap.dc.html`'s own note
		## says that scope "ships when a placed landmark has one identifying
		## name", so `why` is a state this row loses, and an exemption keyed on
		## `disabled` would put a live chip back under the floor the day it
		## does, silently. It is also the only chip on line two, so at 17 dp it
		## draws as a runt under a 44 dp line.
		if _phone:
			var tap := float(_ptap(DccTheme.PHONE_TAP_MIN))
			b.custom_minimum_size = Vector2(tap, tap)
		if declined:
			b.disabled = true
			b.tooltip_text = String(sc["why"])
			b.accessibility_description = String(sc["why"])
		else:
			b.tooltip_text = "Type \"%s \" to search only %s" % [p, String(sc["label"])]

## The count line. Never "0 of 0" for a scope that has no index to count: a
## declined scope names itself and says so instead, because a zero there would
## read as "the world has none", which is a different and false claim.
func _search_count_text(res: Dictionary) -> String:
	if res.has("note"):
		var scope: Dictionary = res.get("scope", {})
		return "%s · not indexed" % String(scope.get("label", "this scope"))
	var shown: int = (res.get("rows", []) as Array).size()
	var noun := "commands" if res.has("commands") else "places"
	return "%d of %d %s · rebuilt on open" % [shown, int(res.get("total", 0)), noun]

## Writes the field on whichever surface is up and re-renders. Used by the scope
## chips, which are the only thing that sets the query from outside the field --
## `LineEdit.text_changed` does NOT fire on a programmatic `text` assignment, so
## the re-render here is the whole update, not a duplicate of one.
func _set_search_query(text: String) -> void:
	var field: LineEdit = _phone_search_field if _phone else _desktop_search_field
	if field != null and is_instance_valid(field):
		field.text = text
		field.caret_column = text.length()
		field.grab_focus()
	if _phone:
		_refresh_phone_search()
	else:
		_refresh_desktop_search()

func _refresh_phone_search() -> void:
	if _phone_search_results == null or not is_instance_valid(_phone_search_results):
		return
	var res := _search_result_set(_phone_search_field.text)
	_fill_scope_chips(_phone_search_chips, res)
	_phone_search_count.text = _search_count_text(res)
	_fill_search_results(_phone_search_results, res, func(): _set_search_open(false))

func _refresh_desktop_search() -> void:
	if _desktop_search_results == null or not is_instance_valid(_desktop_search_results):
		return
	var res := _search_result_set(_desktop_search_field.text)
	_fill_scope_chips(_desktop_search_chips, res)
	_desktop_search_count.text = _search_count_text(res)
	_fill_search_results(_desktop_search_results, res,
		func(): _desktop_search_dialog.hide())

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

## A full-width overlay on the shell's own phone-overlay pattern:
## `_phone_overlay_scrim()` for outside-tap dismissal, `_close_all_phone_
## overlays()`/back-gesture participation, visible only while open. (The
## sentence this replaces cited `_build_phone_panel_picker()` as the pattern's
## other user; that function was deleted with `▤` under ruling 20.
## `_phone_overlay_scrim()` now has exactly two callers, this one and
## `_build_phone_overflow()` -- `grep -n "_phone_overlay_scrim(" shell/*.gd`,
## 2026-09-06, two call sites plus the definition.)
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
	## **360 -> 460 -> 514, twice for the same reason: this pass put chrome
	## inside a fixed box.** Measured by `_railfind_probe.gd` §K at 412x915,
	## windowed, 2026-09-06. The chips row was 38 px and the count line 13, and
	## with the separation they cost the result scroll 51 px, so 360 -> 460. The
	## chips then took §13's tap floor (`_fill_scope_chips()`, whose own block
	## carries the 43 px measurement that forced it) and the row went **38 -> 92
	## px** -- still two lines of chips, now 44 dp tall instead of 17.
	##
	## At 460 that leaves the scroll 286 px, which is **5** rows of `_ptap(52)`,
	## not 6. At 514 it measures 340 px and holds 6 again -- the same figure the
	## previous raise bought, and the same rule: adding chrome to a fixed-height
	## sheet takes the space from whatever was below it, and here that is the
	## entire result of searching, so the sheet grows by the chrome rather than
	## the list shrinking. §K's own `fits > 4` would have stayed green through
	## the drop to 5 -- so the capacity is asserted **by number** in
	## `_phonechrome_probe.gd` §4a, which is the probe that owns this defect;
	## §K still prints it and is left as the second density's reading.
	##
	## It fits, with room, and at three densities rather than one -- panel
	## widths and heights here are content-independent but the reserve above
	## them (`_ptap(H_PHONE_APP_BAR)` + `_safe_top()`) is not. Read off the
	## probes rather than divided out of one reading: `_phonechrome_probe.gd`
	## §4a prints **top 220, bottom 1567 of 2340** at 1080x2340, and
	## `_railfind_probe.gd` §K's `panel.size.y + panel.offset_top <=
	## viewport.y` passes windowed at 412x915 (sheet 514), 1080x2340 (1347) and
	## 1440x3168 (1797).
	panel.offset_bottom = panel.offset_top + _pscale(514)

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
	## `commands` is in the list because the `.` scope really does reach them
	## now. The old wording named the four place families and nothing else, which
	## was exactly true before `SEARCH_SCOPES` and understates the field today.
	_phone_search_field.placeholder_text = "Search places, factions, routes, commands…"
	_phone_search_field.tooltip_text = SEARCH_HINT
	_phone_search_field.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_phone_search_field.custom_minimum_size.y = _ptap(40)
	DccWidgets.well(_phone_search_field, _pscale(12), _pscale(8))
	_phone_search_field.add_theme_font_size_override("font_size", _pfont(12))
	_phone_search_field.text_changed.connect(func(_q: String): _refresh_phone_search())
	head.add_child(_phone_search_field)
	head.add_child(_sheet_close_button(func(): _set_search_open(false)))

	## Chips and count, between the field and the list -- B2's own order.
	## `HFlowContainer` because six chips do not fit one phone line and the
	## sixth wrapping is the correct answer, not a horizontal scroll.
	_phone_search_chips = HFlowContainer.new()
	_phone_search_chips.add_theme_constant_override("h_separation", _pscale(5))
	_phone_search_chips.add_theme_constant_override("v_separation", _pscale(4))
	var chip_pad := MarginContainer.new()
	chip_pad.add_theme_constant_override("margin_left", _pscale(16))
	chip_pad.add_theme_constant_override("margin_right", _pscale(16))
	chip_pad.add_child(_phone_search_chips)
	col.add_child(chip_pad)

	_phone_search_count = DccTheme.mono_label("", "text_faint", _pfont(9), 0)
	_phone_search_count.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	var count_pad := MarginContainer.new()
	count_pad.add_theme_constant_override("margin_left", _pscale(16))
	count_pad.add_theme_constant_override("margin_right", _pscale(16))
	count_pad.add_theme_constant_override("margin_top", _pscale(6))
	count_pad.add_theme_constant_override("margin_bottom", _pscale(2))
	count_pad.add_child(_phone_search_count)
	col.add_child(count_pad)

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
		_refresh_phone_search()
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
	_refresh_desktop_search()
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
	_desktop_search_field.placeholder_text = "Search places, factions, routes, commands…"
	_desktop_search_field.tooltip_text = SEARCH_HINT
	DccWidgets.well(_desktop_search_field)
	_desktop_search_field.text_changed.connect(func(_q: String): _refresh_desktop_search())
	col.add_child(_desktop_search_field)

	## Chips then count, the same order and the same two controls the phone
	## overlay builds -- one design, two surfaces, not two designs.
	_desktop_search_chips = HFlowContainer.new()
	_desktop_search_chips.add_theme_constant_override("h_separation", 5)
	_desktop_search_chips.add_theme_constant_override("v_separation", 4)
	col.add_child(_desktop_search_chips)

	_desktop_search_count = DccTheme.mono_label("", "text_faint", DccTheme.FS_MICRO, 0)
	_desktop_search_count.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	col.add_child(_desktop_search_count)

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
# a `min=-400 max=1200 step=1` slider; the three speed labels at `9.5px` mono,
# lit `var(--acc)` and quiet `var(--faint)`; and `✕` at `11px` mono
# `var(--sec)`, which stops playback and hides the strip.
#
# **One deviation, and the arithmetic is what forced it: the scrub gets its own
# row.** §6.2 draws all six on one line and gives the slider `flex:1` -- "the
# rest of the width". That works in the canvas because four of the six are not
# boxes at all: the speed labels are `9.5px` type with `padding:6px 3px` and
# `✕` is an `11px` glyph with `padding:6px 2px`, so each is about as wide as
# its own glyphs. (Play is the one the canvas does declare, at `38x38`, and it
# is also the one the floor barely moves -- 38 dp to 44.) This port floors every
# tappable control at §13's 44 dp (`_ptap()`), and that floor is what ate the
# track: five of the six siblings became 44 dp boxes and the slider, the one
# `SIZE_EXPAND_FILL` child, was left the remainder.
#
# Measured on the shipped nodes, `_simscrub_probe.gd`, 2026-09-06:
#
# | at | inner width | five `_ptap` boxes | 6 gaps | `YEAR -400` | left for the scrub |
# |---|---|---|---|---|---|
# | 1080 x 2340, scale 2.621, floor 115 px | 966 px | 575 | 156 | 157 | **78 px** (96 with the shorter `NO WORLD`) |
# | 1440 x 3168, scale 3.495, floor 154 px | 1286 px | 770 | 210 | 205 | **101 px** (123 with `NO WORLD`) |
# | 2340 x 1080 landscape, scale 2.621 | 888 px | 575 | 156 | 157 | **0 px** (18 with `NO WORLD`) |
#
# **Every phone composition, so this was never a small-screen edge case**:
# 36.6 dp at 1080, 35.2 dp at 1440 and **6.9 dp in landscape**, against the
# same 44 dp floor. The floor is scale-invariant and so is the overrun -- the
# row asks for more dp than 412 has, and landscape has fewer still: 338.8 dp of
# inner width against portrait's 368.5. Landscape is also the tightest fit in
# the strip on the other axis: with `YEAR -400` the row's minimum is 950 px and
# it is given exactly 950. (The landscape figures come from the probe's replica
# rather than a pre-fix build; the same replica reproduces the 1080 portrait
# scrub width to the pixel, which is what licenses reading them that way.)
#
# The two other ways out were measured and lost; the numbers live in
# `_simscrub_probe.gd::_losing_options()`, which builds a replica from these
# controls' own minimum widths so they stay true as those minimums move.
# **Wrapping the row in an `HFlowContainer`** (the shell has the precedent, in
# the search chips) lays **one line** and changes nothing: a `FlowContainer`
# breaks on its children's *minimum* widths and the scrub is the expander,
# whose minimum is 0. Declaring the floor on it does make it wrap -- and then
# the break lands after the sixth control, stranding `✕` alone on a second row
# and still leaving the track sharing the first (237 px, 90.4 dp at 1080).
# **Shrinking the three speed pills** to close the 19 px deficit at 1080 puts
# each at 108.7 px = 41.5 dp: three violations bought with one, and the scrub
# arrives at exactly 115 px, which is a target and not a length.
#
# A track is dragged along its length. 96 px of length over `TL_YEAR_MIN`..
# `TL_YEAR_MAX` is 16.7 years per pixel; its own row is 966 px and 1.66. The
# strip pays 157 -> 288 px of height for that at 1080 (385 at 1440), which is
# 18% of `_phone_content_gap` in both portrait legs and 38% in landscape, whose
# gap is 749 px -- it grows upward over the map (`GROW_DIRECTION_BEGIN`), and
# the gap is the bound that already clears the tool sheet and the bottom bar.
# All three are asserted inside it, not argued: `_simscrub_probe.gd`'s A4.
#
# **`Timeline.dc.html` board C is cited for what it settles and no more.** That
# board is the desktop/tablet timeline and does not govern this strip; what it
# shows is that when a transport competes with a scrub for one width, this
# project's own newest drawing of *this cursor* answers with
# `flex-direction:column;gap:6px` -- "Row 1, transport" and "Row 2, the scrub
# track" full width beneath it. The phone has less width than the desktop, not
# more. `_pscale(6)` below is that board's gap rather than a fourth number.
#
# **This is the desktop timeline strip's cursor, not a second one.** Every
# control here goes through the §10a block -- `tl_year()`, `tl_set_year()`,
# `tl_toggle_play()`, `tl_set_speed()` -- which reads and writes the engine's
# `CivData::year` directly. `timeline_changed` is what keeps this view and
# `app.gd`'s desktop strip agreeing; neither holds a year of its own.
#
# **Two ways in, and `phone_menu.gd` is one of them.** This block used to say
# there was *no* entry point there; opened 2026-09-06, `_fill_sim()`'s first
# row is a `Transport strip on map` switch whose handler `_toggle_sim_strip()`
# calls `set_phone_sim_strip_open()` and then closes the sheet -- because the
# strip draws over the map that sheet covers, which is §6.6's own toast ("close
# this sheet to scrub") carried out rather than printed. The other way in is
# the phone's own timeline row: `app.gd::_fill_timeline_strip()` dispatches to
# `_fill_phone_timeline_row()`, whose whole-width button toggles the strip, and
# that row is the surface the desktop expands in place and the phone has no
# room to.
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

	## Two rows, not §6.2's one -- see this block's header for the measured
	## arithmetic that forced it and for the two alternatives that lost. `6` is
	## the gap `Timeline.dc.html` board C puts between its own transport row and
	## its own scrub row (`flex-direction:column;gap:6px`), rather than a fourth
	## number invented for this strip.
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", _pscale(6))
	wrap.add_child(col)

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", _pscale(10))
	col.add_child(row)

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

	## Where the slider used to sit. With the scrub moved to its own row the
	## transport row has 122 px of slack at 1080 (measured, no-world string),
	## and without an expander it would all pool at the right-hand end past the
	## `✕`. This puts it between the readout and the speed ladder, which is
	## board C's row 1 exactly (`<div style="flex:1"></div>` between `tlState`
	## and the trailing controls) and keeps §6.2's own left-to-right grouping:
	## transport and readout at one end, rate and dismiss at the other.
	##
	## `IGNORE`, not the `Control` default of `STOP`: it exists only to take up
	## room, and `phone_fit()`'s own spacer clause is the recorded reason a bare
	## `Control` with no `gui_input` must not be the node a tap lands on. That
	## walk never runs over this strip -- it is built in phone units directly --
	## so the rule is applied here by hand rather than inherited.
	var gap := DccTheme.spacer()
	gap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(gap)

	_phone_sim_slider = HSlider.new()
	_phone_sim_slider.min_value = TL_YEAR_MIN
	_phone_sim_slider.max_value = TL_YEAR_MAX
	_phone_sim_slider.step = 1
	_phone_sim_slider.focus_mode = Control.FOCUS_NONE
	_phone_sim_slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_phone_sim_slider.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	## §13's tap floor, on **both** axes and declared here rather than left to
	## the row to supply. `phone_fit()` -- this shell's other 44 dp floor, and
	## the one that reaches `Range` -- never walks this strip, so nothing else
	## was flooring the scrub at all; it was sized entirely by what its five
	## floored siblings left over. The same `_ptap(0)` the speed pills below
	## use, which is `_pscale(maxf(44, 0))`.
	##
	## The height is the term that actually bites: `DccWidgets.phone_slider()`
	## takes `maxf()` against whatever is already declared, and its own row
	## figure is `PHONE_SLIDER_ROW` (32 dp) -- 12 dp under the floor at every
	## scale. Declaring 44 first is what makes that `maxf()` resolve to 44.
	_phone_sim_slider.custom_minimum_size = Vector2(_ptap(0), _ptap(0))
	DccWidgets.phone_slider(_phone_sim_slider, _phone_scale)
	## `value_changed` and not `drag_ended`: the readout beside it has to follow
	## the finger, and `tl_set_year()` is a cursor write plus a snapshot load
	## keyed on an exact year -- cheap at every year the timeline never
	## recorded, which is almost all of them.
	_phone_sim_slider.value_changed.connect(func(v: float):
		if int(v) != tl_year():
			tl_set_year(int(v)))
	## `col`, not `row`: the second line of the strip, full width. Built here so
	## the construction still reads in §6.2's own left-to-right order; only the
	## parent differs, and `col` already holds `row` so this lands beneath it.
	col.add_child(_phone_sim_slider)
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
##
## **This is NOT the `⋮` popover.** That is `_phone_overflow_pop`, and
## `_set_phone_overflow_open()` -- eleven characters longer, same file -- is
## what shows it. The names are close enough that every probe warm-up in the
## tree calls this one believing it opens the popover, which is how the
## popover's three rows (`Save project` / `Theme` / `Close world`) reached
## 2026-09-06 as the only phone subtree nothing had ever laid out:
## `_phonechrome_probe.gd` reported them at `size=(0.0, 115.0)` and blamed a
## deleted warm-up that had never covered them. Measured with
## `_sheetback_probe.gd --sub 1080x2340`: calling the other function first
## takes them to `(601.0, 115.0)` at `_phone_scale` 2.6214. If you want the
## popover, you want the other function.
func _set_overflow_open(open: bool) -> void:
	_close_all_phone_overlays()
	if open:
		_phone_menu.open()

## The MORE tab is a toggle, because the canvas's `07 More` screen carries no
## close button of its own -- tapping the lit tab again is how you leave it, the
## way a bottom-nav tab behaves everywhere else. Without this, MORE would be the
## one tab in the bar that cannot be undone by pressing it.
##
## **Where this parts from `06-phone.md`, stated rather than left to be
## rediscovered.** §2's tab table gives MORE a *sheet title* and *sheet
## subtitle* like the other three, and §5.4 puts its back arrow in the sheet
## header (`Visible when (tab==='more' && moreStack.length>1)`): in the
## prototype MORE is **content inside the detented sheet**, sharing its peek /
## half / full geometry. Here it is `PhoneMenu`, a full-rect overlay with its
## own header, and the detents are the tool-options sheet's alone.
##
## Converting it is a real piece of work and it is **not** a `dcc_shell.gd`
## change: the overlay's rect is written by `PhoneMenu.apply_insets()` in
## `phone_menu.gd`, its five levels carry their own header and back stack, and
## `phone_present_popup()` reuses the same node for transient `PopupMenu`s. The
## cost is enumerable and worth writing down before anyone attempts it -- L2-L5
## would have to survive being resized to 66 dp at `peek`, and §5.4's header
## would have to take over from `PhoneMenu`'s. Not attempted from this file.
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
	if (_phone_search_overlay != null and _phone_search_overlay.visible) \
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
	## `apply_insets()` is **`PhoneMenu`'s**, not this class's, and it is not
	## the shared pass -- it writes three offsets on `_screen`, `_sheet` and
	## `_sheet_scrim`, all of them inside `phone_menu.gd`, and reaches nothing
	## else. *This* function is the shared pass: the chrome column's margins
	## (and with them the app bar), `_apply_phone_nav_orientation()`, both dock
	## sheets, the menu's insets here, and `_snap_phone_sheet()` below. Written
	## down 2026-09-05 because a brief reached this line calling it
	## `DccShell::apply_insets()` and describing it as serving the app bar, the
	## nav bar and the timeline; it serves none of the three.
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
	## disclosure: a 42x4 bar was still drawn over a 24 dp row that could not be
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
