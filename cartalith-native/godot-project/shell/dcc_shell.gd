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

## Emitted by `_select_domain()` when the active domain changes;
## `app.gd::_refresh_rail_foot()` listens.
signal workspace_changed(id: String)
## Declared for the active-tool id changing, but no emit or connection to it exists
## anywhere in this project (grep, 2026-10-12) -- intent unverified; do not rely on it
## firing.
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
## RENDER's one subject (the terrain appearance tunables) now lives under
## CARTO the same
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
#     is another's absence. That is the gate, and since Ruling L it is **both**
#     WORLD nodes' rather than `world/b`'s alone -- see the next paragraph.
#
# `shows` therefore means: *while this mode is active, the dock renders exactly
# these category headers.* Absent (eight nodes of ten) means no gate at all.
# `world/b`'s `["Terrain", "Biomes"]` is §3 row 2 -- the sculpt block, which
# Ruling L (2026-09-21) makes two categories rather than one: height molding and
# biome painting, *"everything done by hand on the map surface"*.
#
# **`world/a` now carries a `shows` too, and before Ruling L it deliberately did
# not.** The old table left `a` ungated on purpose: `Terrain` carried stage 5's
# erosion parameters as well as the sculpt block, so the pipeline view had to
# render a category the sculpt node owned, and "absent means render everything"
# was exactly the behaviour `a` wanted. Ruling L moves those parameters out (to
# `Hydrology` and `Geology`) and states the complement outright -- *"the mode
# pill is the only gate: SCULPT shows Terrain and Biomes; PIPELINE shows the
# rest"* -- so `a` renders nine categories where the specification says seven
# until it lists its own. `shows` and `owns` are identical strings here today
# and are still two keys, because they answer two questions (`owns` is *which
# node lights*, `shows` is *what the dock draws*) and CIVIL/CARTO keep them
# apart. `Workspace.apply_mode()` is what reads this, and `_select_domain()` is
# the one place that calls it.
#
# **A gate is only allowed where a route exists.** `world/b` is reachable from
# its rail node, from the dock's own mode switch (`_build_mode_switch()`), and
# from arming Sculpt; `world/a` from all three of the same. Nothing else in the
# three docks is gated, so nothing else can be stranded by one -- which is the
# property `_leftdock12_probe.gd` §2 asserts by name over all thirty-two
# categories rather than by counting them.
#
# The rest of `mode` is a rail-and-dock *selector*, not a gate. Each domain's
# dock is ONE accordion of every category that domain owns, so gating CIVIL by
# mode would make the nine CIVIL categories no node opens -- Populate,
# Settlements, Territories, Relationships, Military, Culture, Religion,
# Economy, Timeline -- reachable only by a rail trip. That is the
# failure this stage's own rule forbids ("every category reachable before must
# be reachable after"), and §3 does not ask for it. So a node click *opens* its
# category and lights the rail; outside WORLD's two it never hides a sibling.
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
	{"kind": "node", "domain": "world", "mode": "a", "label": "Generate",
		"category": "Generate",
		"owns": ["Generate", "Planet", "Geology", "Hydrology", "Climate",
			"Ecology", "World data"],
		"shows": ["Generate", "Planet", "Geology", "Hydrology", "Climate",
			"Ecology", "World data"]},
	{"kind": "node", "domain": "world", "mode": "b", "label": "Sculpt",
		"category": "Terrain", "owns": ["Terrain", "Biomes"],
		"shows": ["Terrain", "Biomes"]},

	{"kind": "head", "domain": "civilization", "label": "CIVIL"},
	{"kind": "node", "domain": "civilization", "mode": "landmarks", "label": "Landmarks",
		"category": "Landmarks", "owns": ["Landmarks"]},
	{"kind": "node", "domain": "civilization", "mode": "factions",
		"label": "Factions", "category": "Factions",
		"owns": ["Populate", "Settlements", "Factions", "Territories", "Relationships",
			"Military", "Culture", "Religion", "Economy", "Timeline"]},
	{"kind": "node", "domain": "civilization", "mode": "infra", "label": "Routes & ways",
		"category": "Routes & ways", "owns": ["Routes & ways"]},
	{"kind": "node", "domain": "civilization", "mode": "planner", "label": "Journey planner",
		"category": "Travel", "owns": ["Travel"]},

	{"kind": "head", "domain": "cartography", "label": "CARTO"},
	{"kind": "node", "domain": "cartography", "mode": "style", "label": "Style",
		"category": "Style",
		"owns": ["Style", "Relief & light", "Colours", "Feature style"]},
	{"kind": "node", "domain": "cartography", "mode": "layers", "label": "Layers",
		"category": "Layers", "owns": ["Layers"]},
	{"kind": "node", "domain": "cartography", "mode": "labels", "label": "Labels",
		"category": "Labels", "owns": ["Labels"]},
	{"kind": "node", "domain": "cartography", "mode": "icons", "label": "Icons",
		"category": "Icons", "owns": ["Icons"]},
]

# `RAIL_NODES` -- where this port had to decide, because the prototype's ten
# nodes do not cover every category the three docks build. Written down rather than invented in
# silence, per the house rule; each line is a claim a reader can disagree with.
#
# - **WORLD `b` owns `Terrain` and `Biomes`.** They are where
#   `world_workspace.gd:_build_categories()` parents `_sculpt_body` and
#   `_paint_body`, so they are the two categories that contain the by-hand UI
#   the prototype's `ldSculpt` block draws. The seven remaining WORLD categories
#   are pipeline stages and go to `a`, which is what
#   `ldPipe:s.domain==='WORLD'&&wm==='a'` (`ENV:1945`) means.
#   **`a`'s label is `Generate`, not `Generation pipeline`** -- Ruling L's node
#   list, whose own parenthetical is *"names now match their targets"*, and this
#   node's target is `WORLD ▸ Generate`. The mode pill still reads `PIPELINE`
#   (`_MODE_SWITCH_LABELS`), which is the canvas's own word for the mode rather
#   than for the node.
# - **CIVIL `factions` is the catch-all.** Ruling L
#   (`design/owner-references-2026-09-12/left_rail_tree_resorted.md` L22-27,
#   2026-09-13) names five CIVIL nodes -- Settlements, Landmarks, Factions,
#   Routes & ways, Journey planner -- and the labels, targets and order here are
#   its, less the first: **the Settlements node is not built**. A node is keyed
#   by its `mode`, `rail_node()` returns the first match and the rail rows are
#   keyed `domain/mode`, so a Settlements node would need a fifth CIVIL mode id,
#   which is not this table's to add. Four nodes open four of CIVIL's thirteen
#   categories; the other nine (Populate, Settlements, Territories,
#   Relationships, Military, Culture, Religion, Economy, Timeline) are the
#   roster and its consequences, which is what the `factions` node's own dock
#   block draws (`FACTIONS` list + `civPlaces`, `04-left-dock.md` §4 row 7), so
#   they go there. `Economy` with them: it absorbs `Trade`, which went to
#   `infra` only while `_infra.build_trade_into()` drew it as a category of its
#   own. **`landmarks` is still CIVIL's first node, so still what
#   `_domain_mode` is seeded with, and so what a fresh CIVIL entry lights --
#   over an open Populate, which `factions` owns.** Left standing on purpose:
#   the one write that closed it (a boot-time `apply_domain_mode()` from
#   `CivilizationWorkspace._build()`, 2026-09-13, removed the same week) also
#   moved `app.gd::_on_workspace_changed()`'s re-entry baseline, and a Way
#   draft armed before the first in-CIVIL navigation then kept or lost its
#   Commit/Discard row differently from before Ruling L. Reordering this table
#   contradicts the tree's node order and a Settlements node needs a fifth mode
#   id; both are the owner's call (plan C1).
# - **CARTO `style` is the catch-all**, and since Ruling L its own node list
#   says so. The tree (`left_rail_tree_resorted.md` L28-32) names four CARTO
#   shortcuts -- Style, Layers, Labels, Icons -- with the note that "names now
#   match their targets", so each node's label IS its category and the three
#   remaining categories (Relief & light, Colours, Feature style) hang off
#   `style`: they are the look of the map, which is what that node opens.
#   `style` is also CARTO's *first* node, so it is what `_domain_mode` seeds the
#   rail with, and it owns Style, which is the tab's default-open category --
#   entry lights the node that owns the open body, which is the property CIVIL
#   could not have (plan C1).
#
#   **One instruction in Ruling L's retarget list is NOT satisfied here, and it
#   is left standing rather than guessed at.** That list also says *"Rail footer
#   TERRAIN → RELIEF"*. The foot prints the mode id upper-cased
#   (`app.gd::_refresh_rail_foot()`), and `TERRAIN` was this table's fourth
#   CARTO mode, whose node opened `Terrain appearance` -- the category Ruling L
#   renames to `Relief & light`. But the tree's node list puts **Layers** in
#   that fourth slot, not Relief & light, and its own rule says a node's name
#   matches its target. The two cannot both hold with four nodes: the fourth
#   mode is either `relief` (footer RELIEF, no Layers shortcut) or `layers`
#   (footer LAYERS, no Relief shortcut). The tree is the specification and
#   Ruling L's own tie-break is "tree is leading", so this table takes `layers`
#   -- the word TERRAIN does leave the footer, which is what the instruction was
#   for, but it becomes LAYERS rather than RELIEF. **Whether the owner wants a
#   fifth CARTO node, or Relief & light in the fourth slot instead of Layers, is
#   the owner's call.**

# -- Region handles -----------------------------------------------------------
#
# Everything a workspace module needs is reachable from here. Workspaces never
# reach past these into the frame's own containers.

## The HBox the menu-bar buttons are added to; built by `_build_menu_bar()`.
var menu_bar_row: HBoxContainer
## The HBox the tool-options controls are added to (inside the band stored as
## `tool_options_bar`).
var tool_options_row: HBoxContainer
var tool_options_bar: Control          ## The band `tool_options_row` sits in --
	## `_build_tool_options_bar()`'s own return value, stored so a caller (a
	## probe included) can find the band BY MEMBER. Tablet wraps `tool_options_row`
	## in a `ScrollContainer`, so `tool_options_row.get_parent().get_parent()`
	## no longer reaches this band on that composition -- see the header there.
var rail_column: VBoxContainer
## The left dock panel; its minimum width follows `_left_width` unless collapsed.
var left_dock: PanelContainer
## Header label of the left dock; set to the active domain's upper-cased label by the
## domain switch.
var left_dock_title: Label
var left_dock_body: VBoxContainer      ## Workspace panels attach here.
## The map host region that `viewport_content` lives in; assigned by the viewport
## builder.
var viewport_area: Control
var viewport_content: Control          ## The map surface; overlays are children.
## The right dock panel (its title and contents follow the selection, see
## `right_dock_title`).
var right_dock: PanelContainer
var right_dock_title: Label          ## §6: contents follow the selection, not a fixed
	## "Layers" chrome label -- kept live by `right_dock.gd`'s own `set_right_dock_title()` call at the end of every `_rebuild()`.
var right_dock_body: VBoxContainer
## The timeline region built by `_build_timeline()`; callers null-check it.
var timeline_bar: Control
## The HBox inside the timeline region that holds its controls; callers null-check it.
var timeline_row: HBoxContainer
## The HBox of status-bar slots; no label may raise its minimum width (see
## `_build_status_bar()`).
var status_row: HBoxContainer

## Rotated label holding the rail's whole footer string; written by `set_rail_foot()`.
## The stacked form `_rail_foot_run`/`_rail_foot_total` draws the same string.
var rail_foot: Label
## The foot's second form -- `StatusBar.dc.html` panel 5's upright `00` / rule /
## `10`. `rail_foot` above still holds the whole string; this draws it.
var _rail_foot_stack: MarginContainer
## Upper half of the stacked rail footer (`StatusBar.dc.html` panel 5's upright pair).
var _rail_foot_run: Label
## Lower half of the stacked rail footer, under the rule.
var _rail_foot_total: Label
const RAIL_FOOT_GAP := 6      ## `gap:6px` between the two halves and the rule.
const RAIL_FOOT_PAD_B := 8    ## `padding-bottom:8px` under the lower half.
const RAIL_FOOT_RULE_W := 14  ## `width:14px` on the `--div` hairline.
var _domain_buttons: Dictionary = {}   ## id -> Button
var _domain_marks: Dictionary = {}     ## id -> {icon, label}
## The 14 px hairline drawn ABOVE each rail mark but the first, keyed by that
## mark's domain id (`_build_rail()`). Kept only so the Session-mode gate
## (SM-7, `_apply_session_gate()`) can hide a separator that hiding a mark
## would otherwise leave leading the column.
var _rail_seps: Dictionary = {}
## Each rail-expansion header's row and its subtitle row, keyed by domain id
## (`_build_rail_expansion()`). Held for the same gate: the node rows are
## already in `_rail_node_rows`, the headers above them were not keyed at all.
var _rail_exp_heads: Dictionary = {}

# -- Session mode (SM-7, `STORY_MENUS_PROPOSAL.md` §3.4, Ruling CK) -----------
#
# `Window ▸ Session mode` is a table-side switch: it hides the generator and
# keeps the story tools. The **menu half** (the check row, the seeded
# `Session` layout, the disabled rows) is `menus.gd::set_session_mode()`; this
# half is the gate on the shell's own navigation, because every route into a
# domain -- the rail, the node list, the phone tabs, `Window ▸ Workspace`, the
# command index, every in-dock "→ WORLD ▸ …" jump -- lands on `_select_domain()`,
# `select_domain_mode()` or `select_domain_category()`, and one guard there
# covers them all.
#
# **What is hidden is a domain, not a list of widgets.** §3.4 asks to hide
# "WORLD generation and sculpt controls and the pipeline readouts", and every
# one of those is already scoped to the WORLD domain by code that predates
# this: the WORLD dock is that domain's `_workspace_panels` entry, the tool
# palette is rebuilt from `_domain_tools[active_domain()]` (`app.gd::
# _strip_entries()`), and the right dock's pipeline/sculpt sections answer
# `app.active_domain() == "world"` (`right_dock.gd::_tool_section()`,
# `_draft_stack_live()`). So keeping WORLD from being the active domain hides
# exactly those, and the only extra chrome to hide is the WORLD mark itself
# (desktop/tablet rail, rail expansion, the phone domain tab while it reads GENERATE).
#
# Hidden is never destroyed: no panel is freed, no mode is rewritten and no
# engine call is made by the gate. Turning it off makes WORLD reachable again
# exactly as it was.

## Domains Session mode keeps out of reach. One entry, WORLD, for the reason in
## the block above; an array so the separator rule in `_apply_session_gate()`
## is written for the general case rather than for "the first mark".
const SESSION_HIDDEN_DOMAINS: Array = ["world"]
## What a refused route into a hidden domain prints on the status bar. Names
## the way back, because the route the user just took is one that works again
## once the mode is off.
const SESSION_REFUSED := "Session mode is on, so WORLD (generation and sculpt) is hidden. Window ▸ Session mode, Ctrl+Shift+M, turns it off."
## Whether the gate is up. Written only by `set_session_gate()`; boots `false`
## and is never persisted (SM-7: a toggle for one table session, not a setting).
var _session_mode := false
## Emitted after the gate moves, with its new state.
signal session_mode_changed(on: bool)

## Whether Session mode's gate is up.
func is_session_mode() -> bool:
	return _session_mode

## Whether Session mode currently keeps domain `id` out of reach.
func session_hides_domain(id: String) -> bool:
	return _session_mode and SESSION_HIDDEN_DOMAINS.has(id)

## Raise or lower the gate. **Navigation only**: it must never switch domain
## itself (the caller, `menus.gd::set_session_mode()`, moves the shell off a
## hidden domain through the layout mechanism FIRST, then raises this), and it
## must never call the engine -- it can run while a generation owns it.
func set_session_gate(on: bool) -> void:
	_session_mode = on
	_apply_session_gate()
	session_mode_changed.emit(on)

## Shows or hides the chrome that names a hidden domain: its rail mark (on the
## phone the same dictionary holds the WORLD domain tab's button, `_build_phone_
## bar()`), the hairline above a mark that would otherwise lead the column, and
## the rail expansion's header, subtitle and node rows for it. Idempotent.
func _apply_session_gate() -> void:
	for id in _domain_buttons:
		var b = _domain_buttons[id]
		if b is Control and is_instance_valid(b):
			(b as Control).visible = not session_hides_domain(String(id))
	## A separator sits above every mark but the first. It is drawn only while
	## its own mark is drawn AND some mark above it is, so hiding WORLD (the
	## first) does not leave CIVIL's hairline floating at the top.
	var any_above := false
	for d in DOMAINS:
		var did := String(d["id"])
		var shown := not session_hides_domain(did)
		var sep = _rail_seps.get(did)
		if sep is Control and is_instance_valid(sep):
			(sep as Control).visible = shown and any_above
		any_above = any_above or shown
	for did in _rail_exp_heads:
		for row in _rail_exp_heads[did]:
			if row is Control and is_instance_valid(row):
				(row as Control).visible = not session_hides_domain(String(did))
	for key in _rail_node_rows:
		var r = _rail_node_rows[key]
		if r is Control and is_instance_valid(r):
			(r as Control).visible = not session_hides_domain(String(key).split("/")[0])

## The refusal every navigation guard shares: prints `SESSION_REFUSED` and
## answers true when `id` is hidden, so a caller can `return` on it.
func _session_refuses(id: String) -> bool:
	if not session_hides_domain(id):
		return false
	set_status("hint", SESSION_REFUSED, "warn")
	return true
## Id of the active domain; initial `world` is the boot default, applied by
## `_select_domain()`.
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

## Builds the initial `_domain_mode` from `RAIL_NODES`: each domain's first node is its
## default mode.
static func _default_modes() -> Dictionary:
	var out := {}
	for n in RAIL_NODES:
		if String(n.get("kind", "")) == "node" and not out.has(String(n["domain"])):
			out[String(n["domain"])] = String(n["mode"])
	return out
## True while the left dock is collapsed to its chevron strip.
var _left_collapsed := false
## True while the right dock is collapsed to its chevron strip.
var _right_collapsed := false
## Current left-dock width in px; seeded from `DccTheme.W_LEFT_DOCK`, reassigned by
## `_compute_layout_mode()` from the `w_left_dock` role.
var _left_width := float(DccTheme.W_LEFT_DOCK)
## Current right-dock width in px; same scheme as `_left_width`.
var _right_width := float(DccTheme.W_RIGHT_DOCK)
var _status_labels: Dictionary = {}    ## slot -> Label
## §2's three trailing squares (`03-menu-bar.md` §2, children 4/5/8). Held
## because both undo and redo carry live engine state -- enabled-ness and the
## operation name in the tooltip -- that does not exist yet when
## `_build_menu_bar()` runs. `_wire_menu_squares()` is what keeps them true.
var _menu_undo_btn: Button
## The redo square at the menu bar's right end; its enabled state and tooltip are kept by
## `_wire_menu_squares()`.
var _menu_redo_btn: Button
var _collapse_buttons: Dictionary = {} ## "left"/"right" -> Button, so the chevron can flip
var _dock_readouts: Dictionary = {}    ## "left"/"right" -> the collapsed-state Label
var _workspace_panels: Dictionary = {} ## domain id -> Control
## True on a real touch device (`is_touchscreen_available()` with the `mobile` feature)
## or under `--force-touch`; mirrored into `DccTheme.set_touch`.
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
# The classifier is `DccTheme.classify()` (MS-1 stage S1, `MOBILE_SHELL_MS1_SPEC.md`
# §3.1), which owns the line and its argument: with a known density, 900 dp of shortest
# width at every aspect ratio (`DccTheme.TABLET_MIN_DP`, owner answer `BUILD_ANSWERS.md` §2.3,
# made Ruling EG on 2026-11-07); with the legacy figure only, the aspect line
# (`DccTheme.PHONE_ASPECT_MAX`). Both are order-independent, so a rotation cannot flip either.
# Both constants and their worked table moved there from this file; `_compute_layout_mode()`
# is the shell's one caller.
## True when the composition is the phone one, mirrored into `DccTheme.set_phone`. Written by the boot
## classification only (`_booted`, MS-1 S3): a resize that crosses the line requests a class swap
## (`_request_class_swap()`) instead of flipping this flag under a composition built for the other class.
## Until MS-1 S4 builds the swap, the request is logged and the boot class is kept.
var _phone := false
## True while the viewport is wider than tall; unlike `_phone` it is re-decided on every
## resize.
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
## Portrait top safe-area spacer (status-bar inset); built only when `_phone`.
var _phone_top_safe: Control
## Landscape side safe-area spacer (the rotated pocket); built only when `_phone`.
var _phone_side_safe: Control
var _phone_chrome_margin: MarginContainer  ## Shifts right in landscape so the
	## rail and app bar clear `_phone_side_safe` -- "the domain rail shifts
	## inward" (Phone inset rules, LANDSCAPE).
var _phone_content_gap: Control            ## Hosts the floating rail; its own
	## rect is the visible gap between the app bar and the tool sheet.
var _phone_tool_sheet: PanelContainer
## The two things the tool sheet can hold, and which one is up. The tool-options
## scroller is the desktop bar (§13); the domain host (`_phone_dom_host`) holds the dock's own categories
## for WORLD, CIVIL and CARTO. At most one is visible -- see `_refresh_phone_sheet_body()`. (Ruling EH item 1
## retired the third body, the phone's own GENERATE column.)
var _phone_tool_scroll: ScrollContainer
## The phone tool sheet's content column.
var _phone_sheet_col: VBoxContainer
## MS-11: the phone's quick drawer lives over the map, mounted by `set_phone_quick_drawer()`; the shell keeps no
## handle to it (`QuickBarPhone` owns it, and a class swap lets it go through `represent_capture()`).
## MS-3: the domain sheet's host inside the phone tool sheet, and the three pieces of the LEFT dock that
## move into it while a WORLD, CIVIL or CARTO domain sheet is up (WORLD since Ruling EH item 1) (`_mount_dock_in_sheet()`). The dock is the
## builder (`_build_left_dock()`, `register_workspace()`, `apply_mode()`), so nothing is rebuilt for the
## phone: the mode switch and the category scroller are the same nodes, reparented, and returned to the
## dock's own column when the sheet leaves (so Window > Left dock keeps working).
var _phone_dom_host: VBoxContainer
var _left_dock_mode_switch: Control
var _left_dock_col: VBoxContainer
var _dock_in_sheet := false
## MS-3: the rows the domain sheet carries above the dock's own pieces, built lazily on first need (so the
## `_on_phone_node_added()` hook that fits them is the real path, not a build-time accident):
## CIVIL's door to STORY > Journeys, and the MAP sheet's row. (WORLD's "back to the three columns" row went
## with the columns, Ruling EH item 1.)
var _phone_dom_journeys_row: Control
var _phone_map_row: Control
## MS-3: the MAP sheet's scrolling body (`_refresh_phone_map_row()`), and its switches by layer id.
var _phone_map_body: ScrollContainer
var _phone_map_toggles: Dictionary = {}
## 3D-2's phone HEIGHT row (`_rebuild_phone_map_body()`): the "3D height" section's box and its `DccWidgets.slider()`
## result, or empty. The section shows only while 3D does (`ViewportHost.view_mode_changed`).
const View3DScript := preload("res://shell/view3d.gd")   ## no `class_name` there; the HEIGHT range constants only
var _phone_height_box: Control
var _phone_height: Dictionary = {}
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
var _phone_sheet_grab: Control         ## The WHOLE header block's hit area, 2026-09-13
	## on -- pill row, title/subtitle and the close circle -- not only the pill,
	## which is all it was before this pass. See `_build_phone_tool_sheet()`'s
	## own header for why it grabs the header rather than sitting beside it.
var _phone_sheet_drag := {}            ## `{"y0","h0","h"}` while a drag is live; empty otherwise.
var _phone_sheet_tween: Tween          ## Held so a new snap kills the running one.
var _phone_sheet_title: Label          ## `sheetTitle` -- refreshed per tab, `_refresh_phone_sheet_header()`.
var _phone_sheet_subtitle: Label       ## `sheetSub` -- same refresh.

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
## The bottom bar's cells box (one of the four boxes flipped to `vertical` in landscape).
var _phone_bar_cells: BoxContainer
var _phone_bar_dests: BoxContainer     ## The three destination tabs, MORE excluded
	## -- `_rail_region` on phone. See `_build_phone_menu_bar()`'s header.
const W_PHONE_LAND_RAIL := 72.0   ## Prototype, landscape branch: `width:72px;
	## background:var(--pan2);border-right:1px solid var(--hair2)`, and the map
	## host moves to `left:72px`.
const W_PHONE_LAND_SHEET_MAX := 440.0  ## `Math.min(440,Math.round(fw*0.46))`.
## Landscape sheet width as a fraction of the frame width; the prototype's `Math.round(fw*0.46)`, capped by `W_PHONE_LAND_SHEET_MAX`.
const PHONE_LAND_SHEET_FRAC := 0.46

## Clock readout in the phone status pocket.
var _phone_clock_label: Label
## Battery readout in the phone status pocket.
var _phone_battery_label: Label
var _phone_side_clock_label: Label     ## Landscape's rotated-pocket twins of
var _phone_side_battery_label: Label   ## the two above -- see `_build_phone_side_safe()`.
var _phone_menu_bar: Control    ## L1 of the phone disclosure tree -- the bottom
	## bar. Named handle because `_phone_bottom_reserve()` has to measure it.
var _phone_menu: PhoneMenu      ## L2-L5. Replaces the old `_phone_overflow`
	## sheet (`GUI_GAP_REGISTER.md` §15).
## CM-6 (`MAP_CONTEXT_SCOPE.md` §8.2, §11; Ruling AX F2). `preload`, not
## `class_name` -- `tool_fan.gd`'s own header explains why -- so this is typed
## against the const below rather than a global class.
const ToolFan := preload("res://shell/tool_fan.gd")
## The Find `n ` scope's notes search (VE-9); a script file with no `class_name`.
const NotesFind := preload("res://shell/notes_find.gd")
## The CM-6 tool fan (a `ToolFan`): the gesture grammar behind the tool strip's chip (the chip is mounted in
## `_tool_strip`, since MS-5's second step).
var _tool_fan: ToolFan
## MS-5: the phone's tool strip (`tool_strip.gd`), mounted in `_phone_content_gap`. Preloaded, not a `class_name`
## (the project's rule for a script that would need an import pass to register one).
const ToolStrip := preload("res://shell/tool_strip.gd")
var _tool_strip: ToolStrip
## True while the left dock is up as a phone sheet; written by `_set_sheet_open()`.
var _left_sheet_open := false
## True while the right dock is up as a phone sheet.
var _right_sheet_open := false
## The two dock `ScrollContainer`s, held only so `_set_sheet_open()` can zero
## their scroll on open -- neither dock exposes its `_scroll()` return value
## anywhere else, and a sheet's body (`left_dock_body`/`right_dock_body`) is
## never torn down between opens, so whatever scroll position was left from
## the previous open is still sitting on the node when it reopens.
var _left_dock_scroll: ScrollContainer
## The right dock's `ScrollContainer`, held so `_set_sheet_open()` can zero its scroll on
## open.
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
## The phone app-bar search overlay's input.
var _phone_search_field: LineEdit
## The phone search overlay's results column.
var _phone_search_results: VBoxContainer
## The phone search overlay's scope chips.
var _phone_search_chips: HFlowContainer
## The phone search overlay's result-count readout.
var _phone_search_count: Label
var _place_search_index  ## `PlaceSearch` -- untyped, see `_has_place_search()`.
## `CommandIndex`, built lazily the first time a `.` query is run and dropped by
## `_reset_place_search_cache()` alongside the place index. Statically typed,
## unlike `_place_search_index` beside it: `command_index.gd` is a shipped file
## with eight probes already naming the class, not this pass's concurrent
## sibling, so a bare `CommandIndex` token here cannot fail to parse.
var _command_index: CommandIndex

## The floating undo chip (tap undoes, hold opens history); built only on phone.
var _phone_undo_chip: Button
## `06-phone.md` §6.2's edit-history popover and sim strip, and the app bar's
## `⋮` overflow. All three float in the phone composition and are hidden until
## something opens them.
var _phone_undo_pop: Control
## The floating simulation strip (year, slider, transport); hidden until opened.
var _phone_sim_strip: Control
## The strip's year readout (`YEAR n`, or `NO WORLD`).
var _phone_sim_year: Label
## The strip's year slider, ranged `TL_YEAR_MIN`..`TL_YEAR_MAX`.
var _phone_sim_slider: HSlider
## The strip's play/pause button.
var _phone_sim_play: Button
var _phone_sim_speeds: Dictionary = {}   ## multiplier -> Button
var _phone_sim_transport: Array[Control] = []  ## Everything `tl_available()` gates.
## The app bar's `⋮` overflow popover; hidden until opened.
var _phone_overflow_pop: Control
## The overflow row's right-hand text for `Save project`: `_phone_saved_at`, or an em
## dash when never saved.
var _phone_overflow_saved: Label
## The overflow row's right-hand text for `Theme`: `dark` or `light`.
var _phone_overflow_theme: Label
## `savedAt` (§4.3's own state key). Filled from `EngineBridge.project_saved`,
## which is the signal `app.gd`'s own save bookkeeping already fires -- so this
## is the same event, not a second notion of "saved".
var _phone_saved_at := ""
## True between press and release of the undo chip; lets release tell a tap from a
## completed hold.
var _undo_chip_down := false
## True once the hold timer opened history for the current press, so release does not
## also undo.
var _undo_chip_hold_fired := false
## Counts chip presses. Each press's hold timer carries its own number and fires only if it is still the latest, so the
## timer of an EARLIER tap cannot mistake a second tap, started within `PHONE_UNDO_HOLD_SEC` of it, for a hold (it
## did: two quick taps undid once and opened the popover on the second, found by `_announdo_probe.gd`).
var _undo_chip_press_no := 0
const PHONE_UNDO_HOLD_SEC := 0.45  ## Standard mobile long-press threshold
	## (Android's own `ViewConfiguration.getLongPressTimeout()` default).

## Desktop's `Edit ▸ Find on map…` presentation -- built lazily, once, on
## first `open_find_on_map()` call. See that function for why this is a plain
## `AcceptDialog` rather than the phone's hand-built overlay.
var _desktop_search_dialog: AcceptDialog
const WindowRepresent := preload("res://shell/window_represent.gd")
## Find-on-map dialog input (desktop).
var _desktop_search_field: LineEdit
## Find-on-map dialog results column (desktop).
var _desktop_search_results: VBoxContainer
## Find-on-map dialog scope chips (desktop).
var _desktop_search_chips: HFlowContainer
## Find-on-map dialog result-count readout (desktop).
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

## Boot sequence for the shell. Prints the build id first (so it is the first Cartalith line in logcat), decides touch (real touchscreen on a mobile build, or the testing-only `--force-touch` user arg) and publishes it, plus the phone flags and scale, to `DccTheme` for the static widget factories; computes the layout mode once; takes over the Android back gesture and the window-close request (so `_notification` and `_close_requested` MUST always resolve, or nothing could quit the app); builds the phone shell or the desktop shell; fits the docks; selects the active domain. It must run before any other method assumes the regions exist.
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

	## Phone-vs-tablet is decided here, off the boot window size. After boot a resize
	## re-classifies (MS-1 S3, `_on_resize_settled()`); a changed class goes to the swap,
	## never to this flag. Orientation (`_landscape`) is re-decided on every resize.
	_compute_layout_mode()
	## The narrower half of the same publication -- the 412 canvas asks for
	## things a tablet must not get, and a static factory has no way to tell the
	## two apart from `is_touch()` alone. See `DccTheme.is_phone()`.
	DccTheme.set_phone(_phone)
	DccTheme.set_phone_scale(_phone_scale)
	## MS-1 S2: the tablet's dp unit, before either composition is built so every figure is laid out in
	## it. Needs `set_phone()` above (`_unit_factor()` reads `DccTheme.is_tablet()`).
	_apply_unit(_unit_factor())
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
	## MS-1 S4.2: the composition is built by `_mount_composition()`, the one path a cold boot and (from
	## S4.4) a class swap both take, so a swap cannot diverge from a boot. Everything class-independent
	## (the classification above, the resize connection below, `_booted`) stays here.
	_mount_composition()
	get_tree().root.size_changed.connect(_on_window_resized)
	## MS-1 S3: from here on only a class swap may change `_phone` (see the field).
	_booted = true

## MS-1 S4.2 (`MOBILE_SHELL_MS1_SPEC.md` section 3.5, `S4_INVENTORY.md` sections 2.1, 3 and 11): builds the
## composition for the class `_phone` and `DccTheme` name, hooks it, and selects the active domain. This is the
## block `_ready()` held between the classification and the resize connection; moving it changes no node and no
## connection of the shell's own, and one order: the root's `size_changed` is now connected after
## `_select_domain()` instead of before it, and nothing in the build or in `_select_domain()` connects to or emits
## that signal (`S4_INVENTORY.md` section 15.2, O8). The application's half moved too, and what that changed is
## measured there: `DccApp._mount_app_composition()`.
##
## Everything it connects to a long-lived emitter is guarded by `is_connected()`, so a second mount (the
## round trip of `_ms1swap_probe.gd --leg roundtrip`, and from S4.4 a swap) adds no second connection. The
## connections that belong to one class (`_on_tablet_node_added`, `_on_phone_node_added`) are made by the
## builders and undone by `_unmount_composition()`.
##
## Must be called only with no composition present (boot, or after `_unmount_composition()`), and only after
## `DccTheme` has been told the class: the builders read `is_tablet()` and the phone scale at build time.
## It must never free or rebuild a survivor (the bridge, the viewport and its overlays, the timers).
func _mount_composition() -> void:
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
		if not get_tree().node_added.is_connected(_on_dock_node_added):
			get_tree().node_added.connect(_on_dock_node_added)
	## **Windows are walked too.** Only docks were: `phone_fit()` / `tablet_arbitrate()` reach a
	## window only where the window's own file calls them, and `quick_bar_window.gd` (an
	## `ItemList` catalogue), `gen_info_dialog.gd` (a `TextEdit`) and the credits and roster
	## windows never did, so none of the central touch-scroll fixes reached them (measured:
	## `_phonescroll_probe.gd --only Customise`, 0 px). One hook, every touch composition.
	if _phone or DccTheme.is_tablet():
		if not get_tree().node_added.is_connected(_on_window_node_added):
			get_tree().node_added.connect(_on_window_node_added)
	elif get_tree().node_added.is_connected(_on_window_node_added):
		## MS-1 S4.3: a swap onto the desktop leaves the touch composition; the hook is touch-only (S4.2 note,
		## `S4_INVENTORY.md` section 15.1).
		get_tree().node_added.disconnect(_on_window_node_added)

	_select_domain(_active_domain)
	## MS-7: which menu bar this composition was built with (the swap compares it, `_swap_needed()`), and the
	## docking rule for the width it was built at. The deferred call repeats the rule once the row has its real
	## size: the first one ran on a row that had not been laid out, from the viewport's width.
	_bar_whole_built = not _phone and DccTheme.is_tablet() 		and DccTheme.tablet_bar_whole(float(DccTheme.layout().get("dp", Vector2.ZERO).x))
	_apply_dock_plan()
	_apply_dock_plan.call_deferred()
	## Deferred, and it has to be: this function runs as `super._ready()` from
	## `DccApp._ready()`, which only creates `EngineBridge` and `ViewportHost` on
	## the lines *after* it returns. A deferred call lands after every `_ready()`
	## in the tree, so both exist by then -- and `_refresh_viewport_context()`
	## still no-ops rather than erroring on a bare `DccShell` (every phone-chrome
	## probe in this project), which has neither.
	_refresh_viewport_context.call_deferred()

## MS-1 S4.2 (`S4_INVENTORY.md` sections 2.1, 3, 4 and 11): the inverse of `_mount_composition()`. Frees the
## composition (`_phone_root` or `_desktop_root`, which hold every node the builders made, plus the phone's
## `ToolFan`, a plain `Node` child of the app), disconnects the connections the builders made for one class,
## kills what outlives a freed node (the sheet's `Tween`; a `Tween` is not a `Node`), nulls every field that
## pointed into the composition and clears every dictionary and array that held its nodes, and drops the
## composition's entries from `DccTheme`'s registries.
##
## What it does not touch, because it survives: the bridge, the timers, `_active_domain` and `_domain_mode`, the
## dock widths and collapse flags, `_phone`, `_landscape` and `_phone_scale` (a swap republishes those before the
## mount), the timeline's playback state, every `is_connected()`-guarded connection of `_mount_composition()`,
## the debounce timers and the lazily built desktop search dialog.
##
## The caller detaches the survivors that live inside the composition (the viewport and its overlays, under
## `viewport_content`) BEFORE this runs: `DccApp._unmount_app_composition()` does, then calls this. Freed with
## `free()`, not `queue_free()`, so a mount straight after finds no half-dead node of the same class; call it
## from an idle point, never from a signal the composition itself is emitting. Called by the swap (S4.4) and by
## the probes (`_ms1swap_probe.gd --leg roundtrip` proves it leaves no orphan and no extra connection).
func _unmount_composition() -> void:
	var tree := get_tree()
	## Class-specific connections (inventory section 3.1 rows 3 and 4, section 3.2), and the debounce flags a
	## disconnected hook would otherwise leave true for the next class's pass.
	if tree.node_added.is_connected(_on_tablet_node_added):
		tree.node_added.disconnect(_on_tablet_node_added)
	if tree.node_added.is_connected(_on_phone_node_added):
		tree.node_added.disconnect(_on_phone_node_added)
	if timeline_changed.is_connected(_refresh_phone_sim_strip):
		timeline_changed.disconnect(_refresh_phone_sim_strip)
	_tablet_arb_pending = false
	_phone_fit_pending = false
	_dock_fit_pending = false
	## MS-1 S4.3: the three build-time wirings to survivors end with the composition they serve, and their
	## once-guards reopen for the next mount (a swap would otherwise leave the old class's lambdas on the bridge).
	for links in [_menu_squares_links, _phone_undo_links, _phone_overflow_links]:
		for l in links:
			if is_instance_valid(l[0]) and (l[0] as Object).is_connected(String(l[1]), l[2]):
				(l[0] as Object).disconnect(String(l[1]), l[2])
		(links as Array).clear()
	_menu_squares_wired = false
	_phone_undo_wired = false
	_phone_overflow_wired = false
	## A `Tween` is not a `Node`: freeing the sheet would leave it running.
	if _phone_sheet_tween != null and _phone_sheet_tween.is_valid():
		_phone_sheet_tween.kill()
	_phone_sheet_tween = null
	_phone_sheet_drag = {}
	_dom_gesture = {}
	if _tool_fan != null and is_instance_valid(_tool_fan):
		_tool_fan.teardown()
		_tool_fan.free()
	_tool_fan = null
	## MS-5: the strip (and the chip inside it) is a child of the phone root, freed with it; only the reference is cleared.
	_tool_strip = null
	for root in [_phone_root, _desktop_root]:
		if root != null and is_instance_valid(root):
			(root as Node).free()
	_phone_root = null
	_desktop_root = null
	## The nodes the builders made are gone; every handle to one is now null or empty.
	menu_bar_row = null
	tool_options_row = null
	tool_options_bar = null
	rail_column = null
	left_dock = null
	_tablet_row = null
	_left_slot = null
	_right_slot = null
	_left_tab = null
	_right_tab = null
	_left_floating = false
	_right_floating = false
	last_dock_plan = {}
	left_dock_title = null
	left_dock_body = null
	viewport_area = null
	viewport_content = null
	right_dock = null
	right_dock_title = null
	right_dock_body = null
	timeline_bar = null
	timeline_row = null
	status_row = null
	rail_foot = null
	_rail_foot_stack = null
	_rail_foot_run = null
	_rail_foot_total = null
	_rail_region = null
	_rail_exp_column = null
	_rail_chevron = null
	_menu_undo_btn = null
	_menu_redo_btn = null
	_domain_buttons.clear()
	_domain_marks.clear()
	_rail_seps.clear()
	_rail_exp_heads.clear()
	_rail_node_rows.clear()
	_status_labels.clear()
	_status_cells.clear()
	_status_rules.clear()
	_status_dot = null
	_status_tail = null
	_collapse_buttons.clear()
	_dock_readouts.clear()
	_workspace_panels.clear()
	_left_dock_scroll = null
	_right_dock_scroll = null
	_left_dock_mode_switch = null
	_left_dock_col = null
	_dock_in_sheet = false
	_mode_switch_row = null
	_mode_switch_pill = null
	_mode_switch_buttons.clear()
	_mode_switch_domain = ""
	_command_index = null
	## The phone's composition.
	_phone_top_safe = null
	_phone_side_safe = null
	_phone_chrome_margin = null
	_phone_content_gap = null
	_phone_tool_sheet = null
	_phone_tool_scroll = null
	_phone_sheet_col = null
	_phone_dom_host = null
	_phone_dom_journeys_row = null
	_phone_map_row = null
	_phone_map_body = null
	_phone_map_toggles.clear()
	_phone_app_bar = null
	_phone_gesture_inset = null
	_phone_sheet_grab = null
	_phone_sheet_title = null
	_phone_sheet_subtitle = null
	_phone_chrome_col = null
	_phone_bar_row = null
	_phone_bar_domains = null
	_phone_bar_cells = null
	_phone_bar_dests = null
	_phone_clock_label = null
	_phone_battery_label = null
	_phone_side_clock_label = null
	_phone_side_battery_label = null
	_phone_menu_bar = null
	_phone_menu = null
	_phone_search_overlay = null
	_phone_search_field = null
	_phone_search_results = null
	_phone_search_chips = null
	_phone_search_count = null
	_phone_undo_chip = null
	_phone_undo_pop = null
	_phone_sim_strip = null
	_phone_sim_year = null
	_phone_sim_slider = null
	_phone_sim_play = null
	_phone_sim_speeds.clear()
	_phone_sim_transport.clear()
	_phone_overflow_pop = null
	_phone_overflow_saved = null
	_phone_overflow_theme = null
	_phone_tab_cells.clear()
	_domain_fan = null
	_phone_detent = "peek"
	_left_sheet_open = false
	_right_sheet_open = false
	_undo_chip_down = false
	_undo_chip_hold_fired = false
	_coach_marks_wait_for_picker = false
	## MS-1 S4.3: the next composition is built expanded, and `_toggle_dock()` flips from these flags, so one left
	## true would read a collapsed dock that is not there. The user's collapse is carried by
	## `_capture_shell_state()` and put back by `_restore_dock_state()`, per class.
	_left_collapsed = false
	_right_collapsed = false
	DccTheme.clear_composition_registries()

## The pointer-first / tablet composition: one continuous vertical stack of
## fixed-height bars around a horizontal row of rail/docks/viewport. This is
## the shell as it existed before phone support -- `_scaled()` already makes
## it tablet-safe, so nothing here is phone-aware.
# ===================== MS-1 S4.3: the class memory and the cross-class remount (spec §3.5) =====================
#
# A swap deletes one composition and builds another, so what the USER set in the old one has to be carried by
# something that outlives it. That something is `_class_memory`, kept by the shell (never by a composition),
# written by `_capture_shell_state()` just before the unmount and read by `_restore_dock_state()` (before the
# mount: the build reads the widths) and `_restore_view_state()` (after it). It is **per class** wherever the
# class decides what the value means: WI-04's dragged widths and the dock collapse flags (a 232 px portrait
# dock width is not a 400 px landscape one), the scroll offsets (the phone's sheet is not a dock) and the phone
# sheet's detent and tab (a desktop has neither). Capture writes only the keys the OLD class can express and
# leaves the others untouched, so desktop -> phone -> desktop finds the desktop's widths where the user left
# them. Nothing here is saved to disk: a restart is a cold boot (`DccSettings` holds what persists).

## What each class last held, `{class_key: {...}}`; see the block above. `{}` until the first capture.
var _class_memory: Dictionary = {}

## The class the shell is laid out for now: "phone", "tablet" or "desktop". The key of `_class_memory`, and
## what a probe asks instead of reading three flags.
func class_key() -> String:
	if _phone:
		return "phone"
	return "tablet" if DccTheme.is_tablet() else "desktop"

## Captures what the composition of the CURRENT class holds that the user set and that the unmount would lose:
## the dock widths and collapse flags, the scroll offsets of the docks (tablet, desktop) or of the phone's sheet
## (the dock's scroller is the one the sheet shows, so `scroll_left` carries it), and on the phone the sheet's detent and tab. Called before `_unmount_composition()`.
## Reads only; never changes a node.
func _capture_shell_state() -> void:
	var key := class_key()
	_remember_dock_widths()
	## **A floating dock's collapse flag is the docking rule's, not the user's** (MS-7: `_set_dock_floating()` closes a dock
	## that starts floating by writing `_*_collapsed`, and `_toggle_dock()` is the only writer). Remembered as it stands, a
	## tablet passed through 920 dp (both docks float, closed) would hand "collapsed" to the next 1 800 dp tablet, which would
	## then differ from a cold boot of the same size by a dock the user never closed (found by `tools/ms1_b5.sh`'s dump
	## comparison, 232 differing lines on origin/main). So a floating dock keeps the value last remembered for this class.
	var prev: Dictionary = _class_memory.get(key, {})
	var lc: bool = bool(prev.get("left_collapsed", false)) if _left_floating else _left_collapsed
	var rc: bool = bool(prev.get("right_collapsed", false)) if _right_floating else _right_collapsed
	var m: Dictionary = {"left_collapsed": lc, "right_collapsed": rc}
	if _phone:
		m["detent"] = _phone_detent
		m["tab"] = _phone_tab
		m["scroll_tool"] = _phone_tool_scroll.scroll_vertical if _phone_tool_scroll != null and is_instance_valid(_phone_tool_scroll) else 0
	m["scroll_left"] = _left_dock_scroll.scroll_vertical if _left_dock_scroll != null and is_instance_valid(_left_dock_scroll) else 0
	m["scroll_right"] = _right_dock_scroll.scroll_vertical if _right_dock_scroll != null and is_instance_valid(_right_dock_scroll) else 0
	_class_memory[key] = m

## Puts back, BEFORE the mount, what the build reads: the dock widths. A class never visited keeps what
## `_compute_layout_mode()` just published for it (the role widths), which is a real default and not a guess.
## The collapse flags are applied by `_restore_view_state()` once the docks exist.
func _restore_dock_state() -> void:
	var w: Dictionary = _dock_widths.get(_width_key(), {})
	if w.has("left"):
		_left_width = float(w["left"])
	if w.has("right"):
		_right_width = float(w["right"])

## The dock widths the user has dragged, `{"<class>:<orientation>": {"left": px, "right": px}}`. Orientation is part
## of the key because a width is a figure for one shape of window (the portrait tablet's 232 is not the landscape
## tablet's 400), and a resize can flip the orientation before a swap runs (`_on_tablet_resized()`).
var _dock_widths: Dictionary = {}

## The key of `_dock_widths` for the class and orientation the shell is laid out for now.
func _width_key() -> String:
	return "%s:%s" % [class_key(), "land" if _landscape else "port"]

## Puts the current widths away under the current key. Reads only the two width fields.
func _remember_dock_widths() -> void:
	_dock_widths[_width_key()] = {"left": _left_width, "right": _right_width}

## Puts back, AFTER the mount, what needs the composition to exist: the collapse flags (through
## `_toggle_dock()`, so the strip, readout and glyph agree), the phone's tab and detent, and the scroll offsets.
## A coroutine: a scroll offset has no meaning until the content has been laid out, so the scrolls are written
## two frames later, and the caller awaits it. The tab is written after `_select_domain()` (the mount ran it),
## which derives a tab from the domain and would otherwise win.
func _restore_view_state() -> void:
	var m: Dictionary = _class_memory.get(class_key(), {})
	if m.is_empty():
		return
	if left_dock != null and right_dock != null:
		## MS-7: a floating dock is left as the docking rule made it (closed): the memory's flag is about a docked
		## dock, and `_toggle_dock()` flips rather than sets, so a closed floating dock would open here.
		if bool(m.get("left_collapsed", false)) != _left_collapsed and not _left_floating:
			_toggle_dock(true)
		if bool(m.get("right_collapsed", false)) != _right_collapsed and not _right_floating:
			_toggle_dock(false)
	if _phone:
		if m.has("tab"):
			_phone_tab = String(m["tab"])
			_refresh_phone_tabs()
			_refresh_phone_sheet_body()
		if m.has("detent"):
			_set_phone_detent(String(m["detent"]), false)
	await get_tree().process_frame
	await get_tree().process_frame
	for pair in [[_left_dock_scroll, "scroll_left"], [_right_dock_scroll, "scroll_right"],
			[_phone_tool_scroll, "scroll_tool"]]:
		var sc = pair[0]
		if sc != null and is_instance_valid(sc) and m.has(pair[1]):
			(sc as ScrollContainer).scroll_vertical = int(m[pair[1]])

## Republishes the class for a swap to the one the host's size and `touch` name, exactly as a cold boot decides
## it: `_layout` is cleared (a boot has no previous classification to be hysteretic about), the classification
## runs with `_booted` false so it writes `_phone`, and everything `_ready()` publishes after it is published
## again (the touch flag, the phone flags and scale, the tablet's unit, the window chrome, the per-frame process
## that only the phone needs). Must be called with NO composition mounted: the builders read what this sets.
## Called by `_remount_composition()` (S4.4).
func _republish_class(touch: bool) -> void:
	_touch = touch
	DccTheme.set_touch(touch)
	## The statics a cold boot starts from. `_compute_layout_mode()` reads `role_px()` for the dock widths, and
	## `role_px()` reads the phone flag and scale and the layout: left at the OLD class's values they gave a
	## phone-first shell swapped to a tablet a 372 px left dock where a cold tablet draws 400 (measured,
	## `_ms1swap_probe.gd --leg crossclass`; the per-class width memory hid it whenever the tablet had been visited).
	DccTheme.set_phone(false)
	DccTheme.set_phone_scale(1.0)
	if not _swap_keeps_class:
		DccTheme.set_layout({})
		_layout = {}
	_booted = false
	_compute_layout_mode()
	_booted = true
	DccTheme.set_phone(_phone)
	DccTheme.set_phone_scale(_phone_scale)
	_apply_unit(_unit_factor())
	_style_window_chrome()
	## `_process()` switches itself off on a desktop or a tablet and on a phone without a virtual keyboard; a
	## swap onto a phone has to switch it back on (its own comment, written for this).
	set_process(_phone)

func _build_desktop_shell() -> void:
	var shell := VBoxContainer.new()
	shell.set_anchors_preset(Control.PRESET_FULL_RECT)
	shell.add_theme_constant_override("separation", 0)
	add_child(shell)
	_desktop_root = shell

	shell.add_child(_build_menu_bar())
	shell.add_child(_build_tool_options_bar())

	var main_row := HBoxContainer.new()
	main_row.size_flags_vertical = Control.SIZE_EXPAND_FILL
	main_row.add_theme_constant_override("separation", 0)
	shell.add_child(main_row)

	main_row.add_child(_build_rail())
	## MS-7: on the tablet each dock sits in a slot, so that a dock the docking rule floats can open over the map
	## while its slot stays an edge tab in the row (`_make_dock_slot()`). The desktop is built exactly as before.
	if DccTheme.is_tablet():
		_tablet_row = main_row
		main_row.add_child(_make_dock_slot(_build_left_dock(), true))
		main_row.add_child(_build_viewport())
		main_row.add_child(_make_dock_slot(_build_right_dock(), false))
	else:
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

## MS-1 S4.2: the root of the pointer-first / tablet composition (`_build_desktop_shell()`'s vertical stack), null
## on the phone, where `_phone_root` is the root. `_unmount_composition()` frees whichever one exists.
var _desktop_root: Control

# ===================== MS-7: the tablet's docking rule (`MOBILE_SHELL_PROPOSAL.md` section 2.2) =====================
#
# A dock stays docked only while the map keeps `DccTheme.TABLET_MAP_MIN_DP` (520 dp); otherwise it floats: its slot
# in the row shrinks to a 44 dp edge tab and the dock opens over the map as a drawer, closed to start with, and
# closes when the map is pressed (Ruling DU Q8). The decision is `DccTheme.tablet_dock_plan()`, a pure function a
# probe asserts with literals; everything below is its effect on nodes. TABLET ONLY, inside the tablet shell (Ruling
# EG puts the phone/tablet line at 900 dp of shortest width; Ruling DY MQ-1 (c) makes the width classes layout
# classes inside it): `_make_dock_slot()` is called only when `DccTheme.is_tablet()`, so the desktop's nodes are the
# nodes it always had. **Not built here:** the pin (a floating dock that stays open on a map press); the proposal
# draws it, `OUTSTANDING_WORK.md` lists it.
#
# The unit: the tablet's root content scale makes `get_viewport_rect()` dp (MS-1 S2), so every width here is dp.

## The tablet's rail/docks/map row (`_build_desktop_shell()`); null off the tablet.
var _tablet_row: Control
## The slot each tablet dock sits in; plain `Control`s, so a dock wider than its slot overflows over the map instead
## of widening the row (a container would fold the dock's minimum size into the row). Null off the tablet.
var _left_slot: Control
var _right_slot: Control
## True while the docking rule floats that dock.
var _left_floating := false
var _right_floating := false
## The edge tab of each floating dock: the 44 dp target that opens it, and while it is open the handle that closes it.
var _left_tab: Button
var _right_tab: Button
## The last `DccTheme.tablet_dock_plan()` result applied, `{}` before the first (probes read it).
var last_dock_plan: Dictionary = {}

## Wraps a freshly built tablet dock in its slot, with the floating state's edge tab. Called once per dock by
## `_build_desktop_shell()`. The slot's width is the dock's, so a docked dock lays out exactly as it did in the row.
func _make_dock_slot(dock: Control, is_left: bool) -> Control:
	var slot := Control.new()
	slot.mouse_filter = Control.MOUSE_FILTER_IGNORE
	slot.size_flags_vertical = Control.SIZE_EXPAND_FILL
	slot.custom_minimum_size.x = dock.custom_minimum_size.x
	slot.add_child(dock)
	dock.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	var tab := Button.new()
	tab.focus_mode = Control.FOCUS_NONE
	tab.visible = false
	var side := "left" if is_left else "right"
	tab.tooltip_text = "Open or close the %s dock" % side
	tab.accessibility_name = tab.tooltip_text
	tab.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	for state in ["normal", "hover", "pressed", "focus"]:
		tab.add_theme_stylebox_override(state, DccTheme.flat(DccTheme.c("panel")))
	## 56 tall and `TABLET_DOCK_TAB_DP` wide: both at least the touch floor (`PHONE_TAP_MIN`, 44).
	tab.custom_minimum_size = Vector2(DccTheme.TABLET_DOCK_TAB_DP, 56.0)
	tab.pressed.connect(_toggle_dock.bind(is_left))
	slot.add_child(tab)
	if is_left:
		_left_slot = slot
		_left_tab = tab
	else:
		_right_slot = slot
		_right_tab = tab
	return slot

## The width of the row's fixed parts, in dp: everything in the tablet row that is neither a dock slot nor the map
## (the rail). Read from the nodes, so a rail that grows is counted.
func _tablet_fixed_w() -> float:
	var w := 0.0
	for c in _tablet_row.get_children():
		if c == _left_slot or c == _right_slot or c == viewport_area:
			continue
		if c is Control and (c as Control).visible:
			w += (c as Control).get_combined_minimum_size().x
	return w

## Decides which docks float for the current width and applies it: a dock that starts floating is closed, a dock
## that stops floating opens (a judgement: a docked dock is the tablet's default state, and a floating one is the
## exception that starts closed, as the proposal says). Idempotent, so every caller may call it freely: a mount,
## an orientation flip, a settled resize (a window, a UI scale), a restore. No-op off the tablet.
func _apply_dock_plan() -> void:
	if _left_slot == null or not is_instance_valid(_left_slot) or _tablet_row == null or not is_instance_valid(_tablet_row):
		return
	var row_w := _tablet_row.size.x
	if row_w <= 0.0:
		row_w = get_viewport_rect().size.x
	var plan := DccTheme.tablet_dock_plan(row_w, _tablet_fixed_w(), _left_width, _right_width)
	last_dock_plan = plan
	_set_dock_floating(true, not bool(plan["left"]))
	_set_dock_floating(false, not bool(plan["right"]))

## Moves one dock into or out of the floating state, applying the open/closed default of the new state.
func _set_dock_floating(is_left: bool, floating: bool) -> void:
	var was := _left_floating if is_left else _right_floating
	if is_left:
		_left_floating = floating
	else:
		_right_floating = floating
	if floating != was:
		## Floating starts closed; docked starts open. `_toggle_dock()` flips, so only flip a dock that differs.
		var collapsed := is_dock_collapsed("left" if is_left else "right")
		if floating != collapsed:
			_toggle_dock(is_left)
	_sync_dock_slot(is_left)

## Lays the slot, the dock and the tab out for the dock's state: docked (the slot is the dock's width, or the tab's
## when the user collapsed it), floating closed (the slot is the tab), floating open (the slot is the tab, the dock
## overlays the map above it, the tab moves to the dock's open edge to close it). Reads the three flags, writes
## only layout; safe to call at any time.
func _sync_dock_slot(is_left: bool) -> void:
	var slot := _left_slot if is_left else _right_slot
	var dock := left_dock if is_left else right_dock
	var tab := _left_tab if is_left else _right_tab
	if slot == null or not is_instance_valid(slot) or dock == null or tab == null:
		return
	var floating := _left_floating if is_left else _right_floating
	var collapsed := is_dock_collapsed("left" if is_left else "right")
	var width := _left_width if is_left else _right_width
	var overlay := floating and not collapsed
	## A closed floating dock is not drawn at all: its collapsed content has a minimum width of its own (127 dp on a
	## 1 000 dp tablet, measured by `_ms7_probe.gd` before this was hidden) that would spill out of a 44 dp slot over
	## the map. The tab, which fills the strip, is what is seen. A docked dock the user collapsed keeps its own strip
	## and the slot is at least that wide.
	dock.visible = not (floating and collapsed)
	if not floating and not collapsed:
		slot.custom_minimum_size.x = width
	elif floating:
		slot.custom_minimum_size.x = DccTheme.TABLET_DOCK_TAB_DP
	else:
		slot.custom_minimum_size.x = maxf(DccTheme.TABLET_DOCK_TAB_DP, dock.get_combined_minimum_size().x)
	slot.z_index = 30 if overlay else 0
	var t := DccTheme.TABLET_DOCK_TAB_DP
	if overlay:
		dock.anchor_top = 0.0
		dock.anchor_bottom = 1.0
		dock.offset_top = 0.0
		dock.offset_bottom = 0.0
		if is_left:
			dock.anchor_left = 0.0
			dock.anchor_right = 0.0
			dock.offset_left = 0.0
			dock.offset_right = width
		else:
			dock.anchor_left = 1.0
			dock.anchor_right = 1.0
			dock.offset_left = -width
			dock.offset_right = 0.0
	else:
		dock.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	## The tab: shown while floating. Closed, it covers the strip's head; open, it sits on the dock's open edge.
	tab.visible = floating
	tab.set_anchors_preset(Control.PRESET_TOP_LEFT)
	var x := 0.0
	if overlay:
		x = width if is_left else -width - t
	tab.anchor_left = 0.0 if is_left else 1.0
	tab.anchor_right = tab.anchor_left
	if not overlay and not is_left:
		x = -t
	tab.offset_left = x
	tab.offset_right = x + t
	## Closed, the tab is the whole strip (full height: the dock behind it is hidden, and a 44 x row-height target is
	## the biggest, easiest thing to hit); open, a 56 dp handle at the dock's top edge.
	tab.anchor_top = 0.0
	tab.anchor_bottom = 0.0 if overlay else 1.0
	tab.offset_top = 0.0
	tab.offset_bottom = 56.0 if overlay else 0.0
	var open_glyph: String = DccIcons.SYMBOLS["collapse"] if is_left else DccIcons.SYMBOLS["expand"]
	var shut_glyph: String = DccIcons.SYMBOLS["expand"] if is_left else DccIcons.SYMBOLS["collapse"]
	tab.text = open_glyph if overlay else shut_glyph

## A press on the map while a floating dock is open closes it (Ruling DU Q8). Never consumes the event: the press
## still reaches the map. A press inside the open dock, or outside the map (the menu bar, the rail), is left alone.
func _input(ev: InputEvent) -> void:
	if _left_slot == null and _right_slot == null:
		return
	var pressed := (ev is InputEventMouseButton and (ev as InputEventMouseButton).pressed) \
		or (ev is InputEventScreenTouch and (ev as InputEventScreenTouch).pressed)
	if not pressed or viewport_area == null or not is_instance_valid(viewport_area):
		return
	var pos: Vector2 = (ev as InputEventMouse).position if ev is InputEventMouse else (ev as InputEventScreenTouch).position
	if not viewport_area.get_global_rect().has_point(pos):
		return
	for is_left in [true, false]:
		var floating: bool = _left_floating if is_left else _right_floating
		var dock := left_dock if is_left else right_dock
		var tab := _left_tab if is_left else _right_tab
		## The tab is excluded: its own press closes the dock, and closing here as well would reopen it on release.
		if floating and not is_dock_collapsed("left" if is_left else "right") \
				and dock != null and not dock.get_global_rect().has_point(pos) \
				and not tab.get_global_rect().has_point(pos):
			_toggle_dock(is_left)

## Scales a fixed desktop pixel height for the current form factor: raw on a pointer
## build, the `DccTheme.TABLET` table entry (or a 44 px-floored `TOUCH_SCALE` multiple) on
## touch. Must never be applied to phone-native chrome, which uses `_pscale()`/`_ptap()`.
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

# -- MS-1: density, probe arguments and the classifier's inputs ------------------
#
# `MOBILE_SHELL_MS1_SPEC.md` §3.2. `_is_tablet_sized()` lived here until MS-1 stage S1; its question (is
# the short side at least 900 dp, on a real device with a DPI above zero) is now `DccTheme.classify()`'s
# size half, fed by `_density()` below, which answers "unknown" in exactly the cases it answered false.

## The last `DccTheme.classify()` result, `{}` before the first. From stage S3 it is fed back as the
## classifier's `current` argument, for hysteresis; in S1 it is only published.
var _layout: Dictionary = {}

## The value of a user argument, `-- --density 2.625` or `-- --density=2.625`, or "" when it is absent.
## `OS.get_cmdline_user_args()` is what `--force-touch` already reads.
func _user_arg_value(name: String) -> String:
	var args := OS.get_cmdline_user_args()
	for i in args.size():
		var a := String(args[i])
		if a == name and i + 1 < args.size():
			return String(args[i + 1])
		if a.begins_with(name + "="):
			return a.substr(name.length() + 1)
	return ""

## The hosting viewport's size in PHYSICAL px: `Window.size` for the root, `SubViewport.size` inside a
## `--vp` probe host. Never `get_viewport_rect()`, which is in canvas units once a content scale is set
## (stage S2). With no content scale the two are equal, which is why stage S1 changes no answer.
func _host_px() -> Vector2:
	var vp := get_viewport()
	if vp is Window:
		return Vector2((vp as Window).size)
	if vp is SubViewport:
		return Vector2((vp as SubViewport).size)
	return get_viewport_rect().size

## The legacy figure (spec §3.2): what each composition draws at today when no real density is known,
## short side ÷ 412 below aspect 0.6 (the phone's `_phone_scale`) and 1.0 otherwise (the tablet draws
## unscaled). It does not measure dp, so `classify()` is told the density is unknown and skips the dp half
## of the line, exactly as the off-device rule always has. This keeps every existing probe's result.
func _legacy_density() -> float:
	var px := _host_px()
	var s := minf(px.x, px.y)
	var l := maxf(px.x, px.y)
	if l <= 0.0:
		return 1.0
	if _touch and s / l < DccTheme.PHONE_ASPECT_MAX:
		return maxf(1.0, s / DccTheme.PHONE_REF_SHORT)
	return 1.0

## The density the shell classifies with, as `{value: float, known: bool}`. `known` is false only for the
## legacy figure.
##   - On a device (`OS.has_feature("mobile")`): `screen_get_dpi() ÷ 160`, the figure `_is_tablet_sized()`
##     used. Stage S0 is to measure on the OnePlus 6T whether the platform's own density can be read
##     instead; until then this is MQ-2's accepted fallback (Ruling DY, 2026-11-01). A DPI of zero or less
##     (some Android builds report it for a secondary display) is unknown, as `_is_tablet_sized()` treated it.
##   - Off a device: `--density D` (D > 0) when given, else the legacy figure.
## It must never report `known` for a figure that is not a real density: the dp half of the line would
## then classify a synthetic `--force-touch` viewport against the desktop monitor's DPI.
func _density() -> Dictionary:
	if OS.has_feature("mobile"):
		var dpi := float(DisplayServer.screen_get_dpi(DisplayServer.window_get_current_screen()))
		if dpi > 0.0:
			return {"value": dpi / 160.0, "known": true}
		return {"value": _legacy_density(), "known": false}
	var arg := _user_arg_value("--density")
	if arg != "" and arg.to_float() > 0.0:
		return {"value": arg.to_float(), "known": true}
	return {"value": _legacy_density(), "known": false}

## The UI scale (spec §3.6). `--ui-scale S` (S > 0) overrides for a probe run without writing anything.
## Otherwise, on a touch build, `DccSettings.ui_scale()` (MS-1 S5); on the PC always 1.0 (MQ-4, Ruling DY:
## the PC keeps its own density sets). On the phone it still enters the classification, because a scale
## can cross the line, but the phone's unit ignores it (`_phone_scale_for()`, MQ-3 (a)).
func _ui_scale() -> float:
	var arg := _user_arg_value("--ui-scale")
	if arg != "" and arg.to_float() > 0.0:
		return arg.to_float()
	if _touch:
		return DccSettings.ui_scale()
	return 1.0

## MS-1 S5: re-runs the class question and the unit after the UI scale changed (`Preferences ▸ UI scale`).
## The same path as a settled resize, so a scale that crosses the line requests a class swap instead of
## flipping the composition in place.
##
## **Deferred to idle, never run inside the caller.** The caller is `menus.gd::_on_ui_scale_choice`, the
## `id_pressed` of a `PopupMenu` that is part of the composition, and a swap `free()`s that composition
## (`_unmount_composition()`: "never from a signal the composition itself is emitting"). Run inline it freed
## the popup in the middle of its own emission, which the Pad logged as "Object ... was freed or unreferenced
## while a signal is being emitted from it" at UI scale 200 % the moment the tablet bar crossed
## `TABLET_WHOLE_BAR_DP` (logcat 2026-11-09, `size class change requested` one line before). Deferring lets
## the emission end first; the result is the same swap one idle step later.
func ui_scale_changed() -> void:
	if not _touch:
		return
	_on_resize_settled.call_deferred()

## S2 (spec §3.3, MQ-3 (a), Ruling DY's answer): the phone's unit from density. It is capped at today's
## short/412, so a phone narrower than 412 dp keeps its 412-unit layout until MS-2 to MS-6 rebuild those
## surfaces. It is 1.0 off the phone, which makes `DccTheme.phone_scale()`'s own comment true. On the
## OnePlus 6T at a density of 2.625 this returns today's 1080/412 = 2.6214 exactly (B7). UI scale is not
## applied on the phone under MQ-3 (a). With the legacy figure (no `--density`, off a device) it is
## exactly today's `max(1, short/412)`, so every existing phone probe is unchanged.
func _phone_scale_for(px: Vector2, density: float) -> float:
	if not _phone:
		return 1.0
	return maxf(1.0, minf(density, minf(px.x, px.y) / DccTheme.PHONE_REF_SHORT))

## S2 (spec §3.3): the tablet's unit. The window the shell lives in gets a content scale, so every tablet
## figure (the `TABLET` table, `role_px()`'s touch column, the static factories) reads as dp with no
## call-site edit. The phone and the PC keep a factor of 1.
##
## S0 measured the mode on this build (`cartalith-native/docs/evidence/ms1-2026-11-07/`): the root keeps
## mode DISABLED (the project's default, which `project.godot` requires), where the factor scales the
## canvas exactly and Godot raises the root's font oversampling to the factor by itself (edge test 0.925
## at 2, against 1.000 native), so no `oversampling_override` is set. CANVAS_ITEMS would scale against the
## project's 1152 x 648 base size instead (2.083 at a factor of 2), so it is not used. Embedded dialogs,
## popups and colour pickers inherit the factor, and input maps back to canvas units correctly.
##
## A `--vp` probe host is a SubViewport: `size_2d_override` with stretch, which S0 measured drawing the
## same pixels as the root factor (0 of 120 000 differ). **At a factor of 1 the override is cleared to
## (0, 0)**, not set to the full size as draft 02 wrote it: that is the host's own default, so a legacy run
## is the pre-MS-1 viewport by construction rather than by arithmetic (B9, B10).
func _apply_unit(factor: float) -> void:
	var vp := get_viewport()
	if vp is Window:
		(vp as Window).content_scale_factor = factor
	elif vp is SubViewport:
		var sv := vp as SubViewport
		if is_equal_approx(factor, 1.0):
			sv.size_2d_override = Vector2i.ZERO
		else:
			sv.size_2d_override = Vector2i(roundi(sv.size.x / factor), roundi(sv.size.y / factor))
		sv.size_2d_override_stretch = true

## S2: what `_ready()` calls after the layout is published, and what S3 calls after a resize: the tablet
## takes density x UI scale; everything else takes 1. Off a device without `--density` the tablet's
## density is the legacy 1.0, so no existing probe moves.
func _unit_factor() -> float:
	if DccTheme.is_tablet():
		var d := _density()
		return float(d["value"]) * _ui_scale()
	return 1.0

## Classifies the viewport as phone, tablet or desktop and sets `_phone`, `_landscape`
## and `_phone_scale` (plus the narrow-laptop density set, explained below). Needs a
## non-empty viewport and silently keeps the previous answer otherwise; must never be
## called with `_touch` unset expecting a phone result.
##
## The phone/tablet decision is `DccTheme.classify()` over the host's physical size (MS-1 S1), fed the
## previous result for hysteresis (S3): at boot `_layout` is `{}` and the lines sit at their constants.
func _compute_layout_mode() -> void:
	var px := _host_px()
	if px.x <= 0.0 or px.y <= 0.0:
		return
	var size: Vector2 = get_viewport_rect().size
	var d := _density()
	_layout = DccTheme.classify(px, float(d["value"]), _ui_scale(), _touch, bool(d["known"]), _layout)
	DccTheme.set_layout(_layout)
	## Only the boot call writes `_phone` (MS-1 S3, draft 02): after it, a changed class goes to the swap
	## (spec §3.5), never a flipped flag under a composition built for the other class.
	if not _booted:
		_phone = bool(_layout["phone"])
		## Switches that no `phone_fit()` walk reaches (a window built without one, or a row added after its walk:
		## Quick bar, New World's auto-populate rows) are caught as they enter the tree. Owner leftover 2026-11-13.
		if _phone and not get_tree().node_added.is_connected(_on_switch_node_added):
			get_tree().node_added.connect(_on_switch_node_added)
	_landscape = bool(_layout["landscape"])
	## MS-1 S2: the phone's unit from the density, 1.0 off the phone (draft 02's `_phone_scale_for()`).
	_phone_scale = _phone_scale_for(px, float(d["value"]))
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
	## `ROLE`'s touch column -- **still 400/400**, §1's "so two-column readouts
	## survive the larger type" (`UI_SHELL_DESIGN.md`) and what landscape tablet
	## keeps today. The canvas's own landscape figure is 320/320
	## (`Cartalith Tablet.dc.html`'s `valsShell()`), and the owner ruled canvas
	## adoption 2026-09-07 ("tablet still is the pc layout instead of the
	## design I gave") -- but moving `ROLE` to 320 was tried 2026-09-12 and
	## **reverted** 2026-09-13: 8 of 10 rail nodes' dock content still drew
	## 331-382 px at 320 (`_ds03fit_probe.gd --force-touch` at 2560x1600,
	## measured 2026-09-12), because the content itself has not reflowed to
	## the narrower budget -- a declared 320 is not a drawn 320. `ROLE` moves
	## to 320 in the same change that reflows that content, not before. Every
	## width is therefore stated once, in `DccTheme`, and none is duplicated
	## here.
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
	## `_on_tablet_resized()` below, where its WI-04 constraint is argued.
	if (_touch and not _phone) or DccTheme.is_laptop():
		_left_width = float(DccTheme.role_px("w_left_dock"))
		_right_width = float(DccTheme.role_px("w_right_dock"))

# ===================== MS-1 S3: live re-evaluation within a class (spec §3.4) =====================

## True once `_ready()` has built the first composition. From then on only the swap (S4) changes `_phone`.
var _booted := false
## The class question's debounce: 150 ms, a labelled judgement to be revisited on glass. A child of the
## shell itself, never of a composition, so a later swap (S4) cannot free it.
const RESIZE_DEBOUNCE_S := 0.15
var _resize_timer: Timer

## Connected to `root.size_changed` in `_ready()` (draft 02, MS-1 S3). Pointer builds return at once, so
## the PC is untouched (B9). On touch, today's in-class handling runs at once, so a rotation re-lays out
## with no added delay; only the class question waits for the size to settle.
##
## The phone half is the old handler's: `_compute_layout_mode()` re-runs for its orientation. The tablet
## half is still `_on_tablet_resized()`, which acts only on an orientation flip, so WI-04's dragged dock
## widths survive a same-orientation resize (B4). The old comment here argued that re-classifying was
## needless because the class could never change: true on the OnePlus 6T, where over 52 cold starts
## `root.size_changed` never fired (`GUI_GAP_REGISTER.md` §56), but not for a resizable window, split
## screen, a foldable or a UI-scale change, which is what MS-1 exists for.
func _on_window_resized() -> void:
	if not _touch:
		return
	if _phone:
		var was_landscape := _landscape
		_compute_layout_mode()
		if _landscape != was_landscape:
			_apply_phone_orientation()
	else:
		_on_tablet_resized()
	if _resize_timer == null:
		_resize_timer = Timer.new()
		_resize_timer.one_shot = true
		_resize_timer.wait_time = RESIZE_DEBOUNCE_S
		_resize_timer.timeout.connect(_on_resize_settled)
		add_child(_resize_timer)
	_resize_timer.start()

## The size has settled. Classify against the current class (hysteresis): a changed class goes to the
## swap; otherwise the classification is published (its width class and dp, spec §3.4: an addition to
## draft 02, which dropped it) and the unit is re-applied in case the density or the UI scale moved.
func _on_resize_settled() -> void:
	var px := _host_px()
	if px.x <= 0.0 or px.y <= 0.0:
		return
	var d := _density()
	var next := DccTheme.classify(px, float(d["value"]), _ui_scale(), _touch, bool(d["known"]), _layout)
	if _swap_needed(next):
		_request_class_swap(next)
		return
	## The size came back to the current class while a swap was waiting (a split-screen divider dragged
	## across the line and back): the wait is over without a swap, and its status line goes with it.
	if not _pending_swap.is_empty():
		_cancel_swap_wait()
	_layout = next
	DccTheme.set_layout(_layout)
	_apply_unit(_unit_factor())
	## MS-7: a settled resize or a changed UI scale moves the row's width, so the docking rule is decided again.
	_apply_dock_plan()

## MS-7: whether the tablet's menu bar was built whole (the desktop's eight menus, from `TABLET_WHOLE_BAR_DP`).
## Written by `_mount_composition()`; compared by `_swap_needed()`.
var _bar_whole_built := false

## Whether the classification `next` asks for a swap (a remount): the phone/tablet line crossed (MS-1), or, on a
## tablet, the width crossed `TABLET_WHOLE_BAR_DP` (MS-7: the bar's menus are built once, so a new bar is a new
## mount, spec section 3.5). The bar's line has the classifier's 16 dp hysteresis band, so a window dragged along
## 1 180 dp does not rebuild the shell on every frame.
func _swap_needed(next: Dictionary) -> bool:
	if bool(next["phone"]) != _phone:
		return true
	if _phone or not DccTheme.is_tablet():
		return false
	var want := DccTheme.tablet_bar_whole(float((next["dp"] as Vector2).x), 1 if _bar_whole_built else 0)
	return want != _bar_whole_built

# ===================== MS-1 S4.1: quiescence (spec §3.5, `S4_INVENTORY.md` §8) =====================
#
# A class swap must never discard input the user has not confirmed (spec §3.5, "Nothing is discarded to make
# room for a layout change"). So before a swap runs, every source of held input is asked, and while any
# answers, the swap waits with its reason on the status line and runs when the last one ends. The owner's
# defaults stand for what waits (`S4_INVENTORY.md` Q-S4-2 and Q-S4-3): a window holding typed changes and a
# transient confirmation both make the swap wait; other open windows do not (S4.5 re-presents them).
# S4.4 made the swap real (`_run_class_swap()`). A long-lived window that is open makes it wait, with its name as the
# reason, until it has a re-present path (S4.5, `window_represent.gd`; the windows that have one are
# `DccApp.REPRESENTED_WINDOWS`), which `_collect_window_blockers()` recognises by `represent_capture()`.

## How often a waiting swap asks again whether the blockers have ended, in seconds. A labelled judgement:
## fast enough that a swap follows a closed dialog within about a breath, slow enough that the poll costs
## nothing measurable (it is a handful of property reads). Not a debounce: `RESIZE_DEBOUNCE_S` is that.
const SWAP_QUIET_POLL_S := 0.5
## The classification a swap is waiting to apply, or `{}` when no swap is waiting. Written only by
## `_request_class_swap()` and cleared by `_cancel_swap_wait()` or by the swap running.
var _pending_swap: Dictionary = {}
## The poll for a waiting swap; a child of the shell itself, never of a composition, so a swap cannot free it.
var _swap_wait_timer: Timer
## The status sentence this file last wrote for a waiting swap ("" when none), so it is cleared only while
## it is still the sentence on screen and never wipes a hint some other writer put there since.
var _swap_wait_text := ""
## The hint the wait's sentence replaced, `{text, token}`, put back by `_clear_swap_wait_text()`.
var _swap_prev_hint: Dictionary = {}
## How many swap requests have got past the quiescence gate. A probe's counter; nothing in the product reads it.
var class_swap_runs := 0
## True from the moment a swap starts unmounting until its restore has finished (it spans the two frames a scroll
## offset needs). A request made meanwhile waits like any other (`quiescent_blockers()` reports it), so two swaps
## never overlap and a resize that arrives mid-swap is classified once the first has landed.
var _swap_running := false
## How long the last swap took, in ms: `last_swap_sync_ms` is the synchronous part (capture, unmount, republish,
## mount, restore up to the two frames a scroll offset waits), `last_swap_ms` the whole of it. A probe's figures,
## measured on whatever machine runs it; nothing in the product reads them.
var last_swap_ms := 0.0
var last_swap_sync_ms := 0.0

## Every source of unconfirmed input that is live now, each as a short phrase a user can act on ("a dock is
## being resized"), in a stable order; empty when the shell is quiet. Asks, in turn: the shell's own drags
## and holds; `_app_quiescent_blockers()`, which `DccApp` overrides for the engine, the map and the tools
## (`S4_INVENTORY.md` §8); every visible window under the shell that declares `quiescent_blockers()` (its
## held changes); and every visible transient confirmation. A pure query: it must never close, cancel or
## change anything it reports, because the user decides what happens to held input, not the layout.
func quiescent_blockers() -> Array[String]:
	var out: Array[String] = []
	if _dragging_dock != "":
		out.append("a dock is being resized")
	if not _phone_sheet_drag.is_empty():
		out.append("the tool sheet is being dragged")
	if _undo_chip_down:
		out.append("the undo chip is held")
	if _swap_running:
		out.append("a layout change is running")
	out.append_array(_app_quiescent_blockers())
	_collect_window_blockers(self, out)
	return out

## The app-level half of `quiescent_blockers()`. A bare `DccShell` (the phone-chrome probes) has no engine,
## map or tools, so it reports none; `DccApp` overrides it.
func _app_quiescent_blockers() -> Array[String]:
	return []

## MS-1 S4.5 batch 5: a window that has no `represent_capture()` of its own because its OWNER carries it across a class swap
## (the asset slicer: `AssetLibraryWindow.represent_capture()` holds its sheet and parameters and rebuilds it). Such a window is
## not "an open window that holds the layout"; what it holds that is unconfirmed, it reports through `quiescent_blockers()`.
const REPRESENTED_BY_OWNER_META := "represented_by_owner"

## Walks `node` for visible windows. A window that declares `quiescent_blockers()` answers for itself (its
## held changes, which only it can read). Any other visible `ConfirmationDialog` is a pending decision, the
## shape `DccWidgets.confirm()` and `prompt()` build (`S4_INVENTORY.md` §5.3, "Transient dialogs"), as is a
## `modal_card()` that `DccWidgets.modal_present()` marked with `TRANSIENT_CARD_META`, and each
## waits by Q-S4-3's default. **S4.4: any other visible window also holds the swap** ("the X window is open"): a
## long-lived window is laid out for the class it was opened in, so a swap would leave it in the wrong layout, or (a
## window with typed text) lose it. **S4.5: a window that declares `represent_capture()` is the exception** (it is built
## again for the new class and given its state back, `S4_INVENTORY.md` section 18); what it holds unconfirmed it still
## reports through `quiescent_blockers()` or `held_changes()` and still waits (Q-S4-2). The reason names the window so the
## user can close it. An open menu (a `Popup`) holds it too, for a moment.
func _collect_window_blockers(node: Node, out: Array[String]) -> void:
	for c in node.get_children():
		if c is Window:
			var w := c as Window
			if w.visible:
				var before := out.size()
				if w.has_method("quiescent_blockers"):
					for b in w.call("quiescent_blockers"):
						out.append(String(b))
				## PC-7's contract (`context_window.gd`): a context window answers `held_changes()` with the reason it
				## holds something that has no commit path yet (the Place editor's unsubmitted event, the roster's
				## open religion editor), or "". The same held input makes a swap wait.
				elif w.has_method("held_changes"):
					var held := String(w.call("held_changes"))
					if held != "":
						out.append(held)
				elif w is ConfirmationDialog or w.has_meta(DccWidgets.TRANSIENT_CARD_META):
					var t := w.title.strip_edges()
					out.append("a confirmation is open" + ("" if t == "" else " (" + t + ")"))
				## S4.4: an open window that reported nothing of its own still holds the swap until S4.5.
				## S4.5: a window that re-presents itself (`window_represent.gd`) is not a reason to wait; what it holds
				## unconfirmed it has already said above.
				## The desktop Find dialog is a plain AcceptDialog the shell re-presents itself (`search_represent_*`).
				if out.size() == before and not w.has_method("represent_capture") and not w.has_meta(REPRESENTED_BY_OWNER_META) and w != _desktop_search_dialog:
					if w is Popup:
						out.append("a menu is open")
					else:
						var wt := w.title.strip_edges()
						out.append("the %s window is open (close it to change the layout)" % (String(w.name) if wt == "" else wt))
			## A window's own sub-dialogs are not walked: they are its business, reported through its
			## `quiescent_blockers()`, and a hidden window cannot hold a decision on screen.
			continue
		_collect_window_blockers(c, out)

## A classification that crossed the line (from `_on_resize_settled()` or a waiting swap's poll). If
## anything holds unconfirmed input, the swap waits: the reason goes on the status line (and in a toast on
## the phone, whose status bar is not drawn), and the poll asks again every `SWAP_QUIET_POLL_S`. Otherwise
## it runs. It must never flip `_phone` itself, and never discard what a blocker holds.
func _request_class_swap(next: Dictionary) -> void:
	_pending_swap = next
	var why := quiescent_blockers()
	if not why.is_empty():
		_show_swap_wait(why)
		if _swap_wait_timer == null:
			_swap_wait_timer = Timer.new()
			_swap_wait_timer.one_shot = true
			_swap_wait_timer.wait_time = SWAP_QUIET_POLL_S
			_swap_wait_timer.timeout.connect(_on_swap_wait_poll)
			add_child(_swap_wait_timer)
		_swap_wait_timer.start()
		return
	_pending_swap = {}
	_clear_swap_wait_text()
	_run_class_swap(next)

## True while a swap that keeps the class runs (MS-7: the tablet's menu bar crossed `TABLET_WHOLE_BAR_DP`). Read by
## `_republish_class()`, which then classifies with the old classification as `current`, so the hysteresis holds.
var _swap_keeps_class := false

## The swap itself (MS-1 S4.4): `_perform_class_swap()` runs on the mount path, and the log line lets a probe see
## the request (the prefix `size class change requested` is what `_ms1class_probe.gd`'s runner greps). A coroutine
## (the restore waits two frames for the scroll offsets); `_request_class_swap()` does not await it, and
## `_swap_running` is what keeps a second request from starting while it runs.
func _run_class_swap(next: Dictionary) -> void:
	class_swap_runs += 1
	print("Cartalith: size class change requested (phone=%s, %s dp)" % [str(next["phone"]), str(next["dp"])])
	_swap_running = true
	## MS-7: a swap that keeps the class (the tablet's menu bar changed, `_swap_needed()`) must not forget the
	## classification it came from: `_republish_class()` would classify it from scratch, with no hysteresis, and a
	## tablet 10 dp inside the 16 dp band would come back a phone.
	_swap_keeps_class = bool(next["phone"]) == _phone
	var t0 := Time.get_ticks_usec()
	await _perform_class_swap()
	last_swap_ms = (Time.get_ticks_usec() - t0) / 1000.0
	_swap_keeps_class = false
	_swap_running = false

## What a swap does. The base shell (the phone-chrome probes' bare `DccShell`) has no application and no composition
## to carry state across, so it keeps its boot class, which is the pre-MS-1 behaviour; `DccApp` overrides this
## with `_remount_composition()`.
func _perform_class_swap() -> void:
	print("Cartalith: a bare shell keeps its boot class")

## The waiting swap's poll. The size is classified again, because the window may have moved while the swap
## waited: back in the current class, the wait simply ends; still across the line, the request is made
## again with the newest classification, so a swap applies where the window is now, not where it was.
func _on_swap_wait_poll() -> void:
	if _pending_swap.is_empty():
		return
	var px := _host_px()
	if px.x <= 0.0 or px.y <= 0.0:
		_swap_wait_timer.start()
		return
	var d := _density()
	var next := DccTheme.classify(px, float(d["value"]), _ui_scale(), _touch, bool(d["known"]), _layout)
	if not _swap_needed(next):
		_cancel_swap_wait()
		return
	_request_class_swap(next)

## Ends a wait without a swap: forgets the pending classification, stops the poll, clears the status line.
func _cancel_swap_wait() -> void:
	_pending_swap = {}
	if _swap_wait_timer != null:
		_swap_wait_timer.stop()
	_clear_swap_wait_text()

## The classification a swap is waiting for, or `{}` when none is waiting (probes read it).
func pending_class_swap() -> Dictionary:
	return _pending_swap

## Puts the wait's reason on the status line: the first blocker, and how many more. Rewritten only when the
## sentence changes, so a poll does not restart the phone's toast every half second.
func _show_swap_wait(why: Array[String]) -> void:
	var text := "Layout change waiting: %s" % why[0]
	if why.size() > 1:
		text += " (and %d more)" % (why.size() - 1)
	if text == _swap_wait_text:
		return
	## The hint that was there is put back when the wait ends (the boot's "File > New world... to begin" is one),
	## unless the wait's sentence is the one being replaced by a newer sentence of the same wait.
	if _swap_wait_text == "" and _status_labels.has("hint"):
		_swap_prev_hint = {"text": status_slot_text("hint"), "token": String(_status_tokens.get("hint", "text_ghost"))}
	_swap_wait_text = text
	if _status_labels.has("hint"):
		set_status("hint", text, "text_dim")
	phone_toast(text)

## Clears the wait's sentence, but only while it is still the hint on screen.
func _clear_swap_wait_text() -> void:
	if _swap_wait_text == "":
		return
	if _status_labels.has("hint") and status_slot_text("hint") == _swap_wait_text:
		set_status("hint", String(_swap_prev_hint.get("text", "")), String(_swap_prev_hint.get("token", "text_ghost")))
	_swap_prev_hint = {}
	_swap_wait_text = ""

## The tablet's share of the resize handler, and it acts on exactly one event:
## an orientation flip. Added 2026-09-07 with the portrait dock fix.
##
## **WI-04's dragged widths survive a same-orientation resize**, and MS-1 S3 keeps
## that (B4): the class re-check runs from `_on_resize_settled()`, which re-applies
## the unit but never the dock widths. Here: the flip test below reads the raw
## viewport size and returns *before* `_compute_layout_mode()` -- which is the
## call that would overwrite `_left_width`/`_right_width` -- so a same-orientation
## resize mutates nothing at all.
##
## A flip is the one case where the dragged width cannot be kept, and that is
## not a preference: portrait and landscape are different budgets (232 against
## 400, `DccTheme.TABLET_PORTRAIT` and `ROLE.w_left_dock`/`w_right_dock`
## respectively -- landscape's canvas figure is 320, but that move is
## reverted pending a content reflow, see `W_DOCK_TABLET`'s own header), and a
## landscape width carried into portrait is the +285 px overflow the original
## batch fixed. Without this function the fix would hold only for a shell that
## launched already in portrait.
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
	## MS-1 S4.4: the dragged widths belong to the orientation being left. They are put away under its key before
	## the flip's re-layout overwrites them, so a swap that follows the same resize (a tablet that is rotated, or
	## dragged across the 900 dp line, flips orientation first and swaps after the debounce) restores the widths of
	## the orientation it comes back to and never the other's (a portrait 232 in a landscape tablet).
	_remember_dock_widths()
	_compute_layout_mode()
	_apply_dock_widths()
	## MS-7: a different orientation is a different width, so the docking rule is decided again.
	_apply_dock_plan()

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
	## MS-7: on the tablet the dock sits in a slot whose width follows it (and a floating dock's overlay width).
	_sync_dock_slot(true)
	_sync_dock_slot(false)

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
## trusts: `_density()` above reads `screen_get_dpi() / 160.0` as the
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

## The Android system font-scale ratio (screen scale over density), cached after the first
## call; always `1.0` off Android. The 0.85-1.6 clamp is a labelled judgement bounding a
## hostile or broken DPI report, not a measured range. Phone type sizes multiply by this.
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
## `[20, 60, 20]`. All six are defined even though only four have call sites in
## this shell today (back, detent, sample, tool_arm: `grep -rno '_haptic("[a-z_]*")'`,
## 2026-11-02), because the table is the specification and a
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
## On Android it needs `permissions/vibrate=true` in `export_presets.cfg`'s Android
## preset. Until 2026-11-02 that key was missing, and Godot documents that the call
## then has no effect, so every haptic here was silent on a device (Ruling ED;
## `STYLUS_HAPTICS_RESEARCH.md`). Must never be the only cue: a device may have no
## vibrator at all (research §1.4).
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
	## after boot (MS-1 S3: only the S4 swap may change it, and S4 is not built;
	## when it is, the swap must re-enable this), and a display server either has an IME or never will --
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

## Builds the 'program scope only' menu bar region: the amber wordmark, the menu buttons' host row (`menu_bar_row`, where `add_menu` puts the `MenuButton`s), then the undo and redo squares (disabled on a bare `DccShell` with no app) and the readout cluster (world, pass state, res/cpu/gpu/mem). On a phone the bar is built the same way but parked in the hidden `PhoneMenuModel` host as a model, not a view (see `_build_phone_shell`). Returns the bar; the caller adds it.
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

## Handler for the menu bar's redo square: calls `redo_last()` when the DccApp subclass
## defines it (looked up reflectively because this base class does not), then refreshes the
## undo/redo squares. Must never assume the method exists.
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
var _menu_squares_wired := false
## MS-1 S4.3: the connections each of the three build-time wirings made to a survivor, as `[emitter, signal,
## callable]`, so `_unmount_composition()` can drop exactly them. They are class-specific (the squares exist on
## the desktop and the tablet, the chip and the overflow's saved time on the phone), so a swap must end them with
## the composition they serve or a cold boot of the target class and a remounted one differ by their receivers.
var _menu_squares_links: Array = []
var _phone_undo_links: Array = []
var _phone_overflow_links: Array = []
## Same once-guards for the phone's two build-time wirings (`_wire_phone_undo_chip()`, `_wire_phone_overflow()`).
var _phone_undo_wired := false
var _phone_overflow_wired := false

func _wire_menu_squares() -> void:
	(func() -> void:
		var bridge := _find_engine_bridge()
		## S4.3: the squares are gone if a swap left the desktop and the tablet before this deferred call ran.
		if bridge == null or _menu_undo_btn == null:
			return
		## MS-1 S4.2: connected once per composition. The lambdas read the squares through
		## `_refresh_menu_squares()` at call time, and a second mount does not double them (`S4_INVENTORY.md`
		## section 15.1); S4.3 records each link so the unmount drops them with the bar they serve.
		if not _menu_squares_wired:
			_menu_squares_wired = true
			for sig in ["generation_finished", "params_applied", "world_loaded", "dirty_changed", "annotations_edited"]:
				var cb := func(_a = null): _refresh_menu_squares()
				bridge.connect(sig, cb)
				_menu_squares_links.append([bridge, sig, cb])
			## The shared look history (Ruling BY item 2, `DccApp.look_history`) emits
			## `changed` on every commit, undo, redo and clear, so a look-only edit
			## lights the squares without polling. A bare `DccShell` has no such
			## property and skips this.
			var lh = get("look_history")
			if lh != null and lh.has_signal("changed"):
				lh.changed.connect(_refresh_menu_squares)
				_menu_squares_links.append([lh, "changed", Callable(self, "_refresh_menu_squares")])
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

	## The subclass's routing (`DccApp.can_undo_any()` / `undo_target_label()`),
	## the same one `Edit > Undo` reads, so a look-only history lights the square;
	## a bare `DccShell` has neither and falls back to the height stack alone.
	var routed: bool = has_method("can_undo_any")
	var can_undo: bool = bool(call("can_undo_any")) if routed else bridge.can_undo()
	_menu_undo_btn.disabled = not can_undo
	var undo_name: String = String(call("undo_target_label")) if routed else bridge.undo_label()
	_menu_undo_btn.tooltip_text = ("Undo %s (Ctrl+Z)" % undo_name) if can_undo \
		else ("Nothing to undo. This is the global height undo -- a Sculpt or Paint "
			+ "commit, a carve, a generate -- not the Sculpt draft's own stamp history, "
			+ "which has its own Undo in the right dock.")
	_paint_menu_square(_menu_undo_btn, can_undo)

	var bound: bool = bridge.world_gen != null \
		and bridge.world_gen.has_method("redo_available")
	var can_redo: bool = (bound or routed) \
		and (bool(call("can_redo_any")) if routed else bridge.redo_available())
	_menu_redo_btn.disabled = not can_redo
	## The disabled-state messages name the missing/false `WorldGen.redo_available()` binding.
	if not bound and not can_redo:
		_menu_redo_btn.tooltip_text = ("Redo. This build of the app is missing the redo "
			+ "feature -- almost always because the native library is older than this shell. "
			+ "Rebuild it.")
	elif can_redo:
		var redo_name: String = String(call("redo_target_label")) if routed else bridge.redo_label()
		_menu_redo_btn.tooltip_text = "Redo %s (Ctrl+Shift+Z or Ctrl+Y)" % redo_name
	else:
		_menu_redo_btn.tooltip_text = ("Nothing to redo. "
			+ "Edit > Undo history... shows what the ledger is holding.")
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
	## PC-9 (`PC_SHELL_PROPOSAL.md` §3.8): with one menu open, moving the pointer across the bar
	## opens the menu under it, as every desktop menu bar does. Off on touch: there is no hover.
	mb.switch_on_hover = not _touch
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
	## MS-7 (the 44 dp bar): the short tablet bar's `☰` measured 43 dp wide (one glyph and the padding), under the
	## floor on that axis. Touch only; the pointer bar is untouched.
	if _touch:
		mb.custom_minimum_size.x = maxf(mb.custom_minimum_size.x, 44.0)
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
## Stylebox override names the palette-switch recolour walks (see `_recolor_subtree()`): the ones the shell's widgets author. Add a name when a new widget sets another.
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

## True when `old_pal` is the dark palette, i.e. the switch just made is dark to light. Used
## by the recolour pass to decide which direction `DccTheme.remap()` should read.
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
	## **The border must reach up over the title bar.** Godot draws an embedded
	## window's title and its close button in the `title_height` strip ABOVE
	## the content rect, and paints that strip only with this box's
	## `expand_margin_top` -- the stock theme expands it by ~32 px for exactly
	## this. With no expansion the strip was bare, so every dialog's title and
	## its ✕ floated over whatever lay behind (map, menu bar), outside the
	## panel they close (owner, 2026-09-23: "buttons to exit menus ... fall
	## outside of the menus themselves").
	var title_h := th.get_constant("title_height", "Window") \
		if th.has_constant("title_height", "Window") \
		else ThemeDB.get_default_theme().get_constant("title_height", "Window")
	for box in ["embedded_border", "embedded_unfocused_border"]:
		var frame := DccTheme.panel("raised",
			{"left": 1, "right": 1, "top": 1, "bottom": 1})
		frame.expand_margin_top = title_h
		th.set_stylebox(box, "Window", frame)
	## The stock ✕ is a near-white bitmap with no modulate constant, so on the
	## light palette it vanished into the strip above. Rasterised in the
	## palette's own ink instead -- re-run by `rebuild_theme()` on a switch.
	for pair in [["close", "text_dim"], ["close_pressed", "accent"]]:
		var img := Image.new()
		img.load_svg_from_string(('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" ' +
			'width="16" height="16"><path d="M4 4 L12 12 M12 4 L4 12" fill="none" stroke="#%s" ' +
			'stroke-width="1.4" stroke-linecap="round"/></svg>') % DccTheme.c(pair[1]).to_html(false))
		th.set_icon(pair[0], "Window", ImageTexture.create_from_image(img))
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

## Recursively remaps every colour and `StyleBoxFlat`/`StyleBoxLine` theme override under
## `node` from `old_pal` to the live palette, so a palette switch reaches already-built
## controls. Only touches the override names listed in `_THEME_COLOR_OVERRIDES` and
## `_THEME_STYLEBOX_OVERRIDES`; colours not found in the old palette are left alone.
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

## Remaps one `StyleBoxFlat`'s background and border colours in place. A colour that is
## not a token of `old_pal` (`remap()` returns null) is deliberately left unchanged.
func _recolor_stylebox(sb: StyleBoxFlat, old_pal: Dictionary) -> void:
	var new_bg = DccTheme.remap(sb.bg_color, old_pal)
	if new_bg != null:
		sb.bg_color = new_bg
	var new_border = DccTheme.remap(sb.border_color, old_pal)
	if new_border != null:
		sb.border_color = new_border

# -- §4 Tool options bar ------------------------------------------------------

## Builds the tool options bar region and stores its row in `tool_options_row` (the node the active workspace or `app.set_tool_options` refills). Its height comes from `DccTheme.role_px("h_tool_options")`, not `_scaled()`, because the tablet scale table is keyed by bare desktop integers and 40 collides with the rail's width. On a tablet the row sits in a horizontal scroller so a long bar scrolls instead of clipping or dropping controls. Returns the bar.
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
	## **Tablet only, and the fix for the rail's own frame overflow.** The
	## canvas draws this zone `flex:1;min-width:0;overflow-x:auto`
	## (`TABLET_UI_SPEC.md` §2.2) -- FILL the band when the content's minimum
	## is smaller than it (`flex:1`), scroll rather than clip or wrap when it
	## is not (`overflow-x:auto`), and DS-03's ruling ("keep everything,
	## reflow only") forbids dropping a control to make it fit. At portrait
	## 800 px wide, six touch-sized text buttons alone gave `tool_options_row`
	## a combined minimum of 1 057 px (`_compfit_probe.gd`) -- the single
	## largest contributor to the shell's own +285 px overflow, bigger than
	## both docks' share combined. A bare `HBoxContainer` folds that minimum
	## straight up through `pad` and `bar` into the shell's top-level
	## `VBoxContainer`; a `ScrollContainer` whose horizontal axis is anything
	## but DISABLED does not (`MISTAKES.md`, "Read a layout that overflows the
	## screen") -- the same idiom `_build_phone_tool_sheet()` already uses for
	## this identical row on phone (`horizontal_scroll_mode = SCROLL_MODE_AUTO`).
	## Vertical stays DISABLED (no scrollBAR on that axis, i.e. no second
	## scroll direction) -- the bar's own height is `role_px("h_tool_options")`
	## above, not this row's business; `pad` still needs to FILL that height,
	## which is a size-flag question on `pad`, not a `ScrollContainer` axis one
	## -- see the flags immediately below.
	##
	## **`pad` needs `SIZE_EXPAND_FILL` on BOTH axes, and shipping with neither
	## was the bug** (verifier findings, 2026-09-13; the vertical half found by
	## this pass measuring against HEAD rather than trusting the horizontal
	## fix alone -- see the mutation/HEAD-diff evidence below). A
	## `ScrollContainer`'s single child is sized to `max(container_size,
	## child_minimum_size)`, PER AXIS, only on the axis(es) that carry the
	## EXPAND flag; without it that axis holds the child at its OWN minimum,
	## anchored to the start, same as `PanelContainer` would do for a
	## `SIZE_SHRINK` child. `pad` had no size flags set on either axis (Godot's
	## plain default is FILL, not EXPAND_FILL), so at 2560x1600 the WORLD row
	## measured 1057x44 (spacer collapsed to 0 px, Bake/Refine at x 704-1071)
	## against HEAD's non-scroll 2532x55 (spacer 690-2165, Bake/Refine at
	## 2179-2546); INFRA·WAY's Commit/Discard moved 2329-2546 -> 518-735.
	## Setting only the horizontal flag reproduced HEAD's x-positions and
	## widths exactly (`_toolbargeom_probe.gd`, run against this file and
	## against a `git show HEAD:` checkout of it, both at 2560x1600 and
	## 1280x800) but left height at 44 against HEAD's 55 -- buttons drawing
	## top-aligned in the band with the same rule applying to Y independently.
	## Setting both flags makes the `ScrollContainer` give `pad` the FULL band
	## size on whichever axis is >= `pad`'s own minimum on that axis --
	## `tool_options_row`'s own internal spacer (a `SIZE_EXPAND_FILL` child of
	## its own) then has real horizontal slack to grow into, and the row fills
	## the band's height, reproducing HEAD's layout exactly on both. When the
	## band is narrower than `pad`'s minimum (portrait 800 px, which never
	## affects Y since the band's height is fixed), `max()` above still yields
	## the minimum on that axis, so the flag changes nothing there and the row
	## still scrolls horizontally -- `_ds03fit_probe.gd`/`_compfit_probe.gd`
	## assert both cases.
	##
	## Desktop and laptop get no `ScrollContainer` at all -- `pad` parents
	## `tool_options_row` directly, exactly as before this existed, so neither
	## composition's tree or minimum size moves by a pixel.
	if DccTheme.is_tablet():
		pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		pad.size_flags_vertical = Control.SIZE_EXPAND_FILL
		var scroll := ScrollContainer.new()
		scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
		scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
		scroll.add_theme_stylebox_override("panel", DccTheme.empty())
		scroll.add_child(pad)
		bar.add_child(scroll)
	else:
		bar.add_child(pad)
	tool_options_bar = bar
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

## One switch's glyph boost and 44 dp real-density floor. `unit` is the phone unit on the main viewport, 1.0 in a
## window, whose own content scale is `win_csf`. 44 dp of the device's REAL density as well as of the phone unit: the
## unit is the density capped at short side / 412, so 44 units is 41 dp on the 6T (2.625 x 44 against 2.8125 x 44).
## Only a switch is raised, by a few px.
func _fit_switch(cb: BaseButton, unit: float, win_csf: float) -> void:
	DccWidgets.phone_switch(cb, unit, win_csf > 1.0)
	var dp_floor := ceilf(44.0 * float(_density()["value"]) / maxf(win_csf, 1.0))
	cb.custom_minimum_size = Vector2(maxf(cb.custom_minimum_size.x, dp_floor),
		maxf(cb.custom_minimum_size.y, dp_floor))


## `SceneTree.node_added` hook (phone only, see `_compute_layout_mode()`'s connect): every `CheckBox` / `CheckButton` is
## sized by the same `_fit_switch()` one idle frame later, once its parents and its window's content scale exist.
## Idempotent with `phone_fit()`'s own pass (`phone_switch()` stores what it painted), so a switch both reach is
## sized once. Never touches anything else.
func _on_switch_node_added(n: Node) -> void:
	if n is CheckBox or n is CheckButton:
		_fit_switch_late.call_deferred(n as BaseButton)

## Deferred half of `_on_switch_node_added()`: a window's factor wins; on the main viewport the phone unit does.
func _fit_switch_late(cb: BaseButton, tries: int = 0) -> void:
	if not is_instance_valid(cb) or not cb.is_inside_tree():
		return
	var csf := _switch_window_scale(cb)
	if csf > 1.0:
		_fit_switch(cb, 1.0, csf)
		return
	var w := cb.get_window()
	if w == get_window():
		if _phone_scale > 1.0:
			_fit_switch(cb, _phone_scale, 1.0)
		return
	## A dialog may be built (and its rows added) long before `phone_present()` scales it: New World's auto-populate
	## rows, measured 2026-11-13 (their window's `visibility_changed` never reached us), and any window prebuilt
	## hidden at boot. So wait for the switch's own first `draw` (it only fires once the control is visible in the
	## tree, a signal and not a per-frame poll over hundreds of switches), then look again; a window that is visible
	## but still unscaled is polled for up to 300 frames (5 s at 60 fps).
	if not cb.is_visible_in_tree():
		if not cb.draw.is_connected(_fit_switch_after_draw.bind(cb)):
			cb.draw.connect(_fit_switch_after_draw.bind(cb), CONNECT_ONE_SHOT)
	elif tries < 300:
		await get_tree().process_frame
		_fit_switch_late(cb, tries + 1)

## `_fit_switch_late()`'s retry, once the switch has first been drawn.
func _fit_switch_after_draw(cb: BaseButton) -> void:
	await get_tree().process_frame
	_fit_switch_late(cb)

## The content scale of the nearest ancestor `Window` of `ctl` when that is above 1.0 (a phone window's own unit,
## `DccWidgets.phone_window()`), else 1.0. `phone_fit(d, 1.0)` is the window call shape, so this is how a switch
## there learns it is on a phone at all. Not-in-tree controls walk their parents just the same.
func _switch_window_scale(ctl: Node) -> float:
	var n: Node = ctl.get_parent()
	while n != null:
		if n is Window:
			return maxf((n as Window).content_scale_factor, 1.0) if _phone else 1.0
		n = n.get_parent()
	return 1.0

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
## The five per-style font-size theme items a `RichTextLabel` carries; `font_size` alone does nothing there, so the phone font pass must write each.
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
			## Owner, 2026-11-13: *"selection switches are ridiculously small"*. The tap box above is floored at 44 dp,
			## but the switch glyph was a texture drawn in physical px; `phone_switch()` scales it by this walk's
			## own unit, so every `CheckBox` and `CheckButton` on a fitted surface follows the phone and no control
			## sets its own size.
			if ctl is CheckBox or ctl is CheckButton:
				## A switch in a phone WINDOW arrives with `unit` 1.0 and a content-scaled ancestor `Window`; the
				## same glyph boost and 44 dp real-density floor apply there, in that window's local pixels.
				if unit > 1.0:
					_fit_switch(ctl as BaseButton, unit, 1.0)
				else:
					## A window is fitted at `unit` 1.0 BEFORE `phone_present()` gives it its content scale (measured
					## 2026-11-13: Assistant settings, Quick bar, New World and Export all arrived unscaled), so the
					## window case is decided one idle frame later, when the ancestor `Window`'s factor is final.
					_fit_switch_late.call_deferred(ctl as BaseButton)
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
			## The same hole one class over: a `ColorPickerButton`'s popup is a
			## `Window` this walk cannot reach either. Scaled by the phone's own
			## factor, **not** by `unit` -- `unit` is 1.0 inside the roster and
			## the slicer, but their pickers' popups embed in the main viewport
			## and inherit none of those windows' scale. See the helper.
			if ctl is ColorPickerButton:
				DccWidgets.phone_color_popup(ctl as ColorPickerButton)
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
## tablet=true` (aspect .625, over `DccTheme.PHONE_ASPECT_MAX`; that is the legacy
## off-device rule, which still holds without `--density`, while on a device Ruling EG
## makes 800 x 1280 dp a phone):
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
		## The text field above is only HALF of a SpinBox. Its own arrow
		## strip steps the value on touch-DOWN and a vertical swipe that
		## starts there drags it (measured 2026-10-05, 4.0 -> 3.0 on a
		## tap and -> 1.0 on a swipe), so the strip is arbitrated by
		## `DccWidgets.PgSpin`. Same slop as the field and slider; stock
		## wheel/mouse stay untouched (see `PgSpin`'s gates).
		DccWidgets.touch_spinbox(ctl as SpinBox, 8.0 * unit)
	## **Superseded 2026-11-08 for the READ-ONLY `TextEdit` (and `ItemList`, passive cards, popups), and
	## 2026-11-12 for the EDITABLE `TextEdit`:** `_touch_scroll_arbitrate()` below gives them the central
	## touch scroll. The next paragraph is the reasoning that held until then.
	_touch_scroll_arbitrate(ctl, unit)
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

## **Central touch scroll (Ruling BK; `OUTSTANDING_WORK.md` "Central touch-scroll fix")**: the three
## fixes that stop a drag reaching the scroller above it, split out of `_touch_arbitrate()` so the
## window hook (`_on_window_node_added()`) can apply THEM ALONE. The rest of `_touch_arbitrate()`
## (slider, button and field arbitration) stays opt-in per window through the window's own
## `phone_fit()` call, deliberately: a field parked at `FOCUS_NONE` refuses a programmatic
## `grab_focus()`, and the first version of the window hook applied all of it to the vault editor and
## broke `_ve5_probe.gd` on the phone (66 of 90 checks and a script error at the property form; the
## origin/main tree passes 90). Idempotent: each helper guards itself.
##
## `unit` scales the 8 dp slop (Android's `ViewConfiguration.getScaledTouchSlop()`) like the slider's:
##
## * a button in a scroller fires on release and lets the drag through (`DccWidgets.touch_release_button()`).
## * a passive STOP card, colour rect, label or non-scrolling text block becomes PASS
##   (`DccWidgets.touch_passive()`). The `PanelContainer` the arbitration used to leave alone "because
##   several carry their own `gui_input`" is handled by that function's own gate (no listener, no
##   handler script) instead of by skipping the class. Measured at 393x852 on the CARTO domain sheet:
##   every one of 12 repeated strokes that started on a theme chip was swallowed by the STOP
##   `PanelContainer` holding the chip, the `Button` inside it being PASS already.
## * an `ItemList` or `TextEdit` (editable ones keep their caret) gets a touch-scroll proxy
##   (`DccWidgets.touch_list()`): the engine scrolls neither by touch. So does a control a window marked
##   `DccWidgets.TAP_META` (a drawn tile that handles taps itself).
## * an `OptionButton`/`MenuButton` popup gets the same proxy (`DccWidgets.touch_popup()`):
##   `PopupMenu` picks an item on touch and cannot be dragged. The popup is an internal child, which
##   `get_children()` (the walks' iterator) never visits, so it is asked for by name here.
func _touch_scroll_arbitrate(ctl: Control, unit: float) -> void:
	DccWidgets.touch_passive(ctl)
	## A button inside a scroller fires on release and lets the drag through (`touch_release_button()`, the same call
	## `_touch_arbitrate()` makes). Added here 2026-11-12 because a window on the tablet never goes through
	## `_touch_arbitrate()`, and the Asset library's inspector Button swallowed every drag that began on it (0 px).
	if ctl is BaseButton:
		DccWidgets.touch_release_button(ctl as BaseButton)
	if ctl is ItemList or ctl is TextEdit or ctl.has_meta(DccWidgets.TAP_META):
		DccWidgets.touch_list(ctl, 8.0 * unit)
		## A control a long press can lift gets the shell's haptic pulse at the moment it lifts (`DccWidgets.LIFT_META`).
		if ctl.has_meta(DccWidgets.LIFT_META) and ctl.has_meta("_touch_scroll"):
			var lift_ts = ctl.get_meta("_touch_scroll")
			if lift_ts is DccWidgets.TouchScroll:
				(lift_ts as DccWidgets.TouchScroll).on_lift = func() -> void: _haptic("tool_arm")
	if ctl is OptionButton:
		DccWidgets.touch_popup((ctl as OptionButton).get_popup(), 8.0 * unit)
	elif ctl is MenuButton:
		DccWidgets.touch_popup((ctl as MenuButton).get_popup(), 8.0 * unit)

## Stamp for `_touch_scroll_walk()`; its own key, so a window already walked by `phone_fit()` (which
## stamps nothing) and a node stamped by `_touch_arbitrate_walk()` are both still visited once here.
const _TOUCH_SCR_META := "_touch_scroll_walked"

## Recursive worker for the window hook: `_touch_scroll_arbitrate()` on every `Control` under `node`
## exactly once (guarded by `_TOUCH_SCR_META`).
func _touch_scroll_walk(node: Node, unit: float) -> void:
	for child in node.get_children():
		if child is Control and not child.has_meta(_TOUCH_SCR_META):
			child.set_meta(_TOUCH_SCR_META, true)
			_touch_scroll_arbitrate(child as Control, unit)
		_touch_scroll_walk(child, unit)

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

## Recursive worker for `tablet_arbitrate()`: arbitrates each `Control` under `node`
## exactly once (guarded by `_TOUCH_ARB_META`). Must never revisit a stamped node.
func _touch_arbitrate_walk(node: Node, unit: float) -> void:
	for child in node.get_children():
		if child is Control and not child.has_meta(_TOUCH_ARB_META):
			child.set_meta(_TOUCH_ARB_META, true)
			_touch_arbitrate(child as Control, unit)
		_touch_arbitrate_walk(child, unit)

## Windows (other than popups) with an arbitration pass already queued, by instance id, so
## a window that adds three hundred controls queues one pass, not three hundred.
var _win_arb_pending := {}

## `node_added` hook (phone and tablet): when a `Control` arrives inside a `Window` that is
## not the shell's own viewport and not a `Popup` (dropdown lists are handled by their
## owner's `touch_popup()`), queues ONE deferred `_touch_scroll_walk()` of that window.
## `unit` is 1.0: a window is content-scaled, and the 8 dp slop is in the window's own
## pixels (the rule `_touch_arbitrate()`'s header gives for windows).
func _on_window_node_added(node: Node) -> void:
	if not (node is Control):
		return
	var vp := node.get_viewport()
	if not (vp is Window) or vp is Popup or vp == get_viewport():
		return
	var id := vp.get_instance_id()
	if _win_arb_pending.has(id):
		return
	_win_arb_pending[id] = true
	_run_window_arbitrate.call_deferred(vp as Window)

## Deferred body of `_on_window_node_added()`; tolerates a window freed in between.
func _run_window_arbitrate(w: Window) -> void:
	_win_arb_pending.erase(w.get_instance_id() if is_instance_valid(w) else 0)
	if is_instance_valid(w):
		_touch_scroll_walk(w, 1.0)

## The tablet half of `_on_phone_node_added()`, and the reason the census
## number was 224 rather than 0: every workspace panel is attached to a dock
## long after `_build_desktop_shell()` returns, so a one-shot walk at boot
## would arbitrate the chrome and none of the content. Same coalescing as the
## phone's -- one deferred pass per frame however many nodes arrive.
var _tablet_arb_pending := false

## `node_added` hook (tablet only): when a `Control` arrives anywhere inside either dock,
## schedules ONE deferred arbitration pass (`_tablet_arb_pending` coalesces a burst of
## additions). Ignores nodes outside the docks.
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

## Deferred body of `_on_tablet_node_added()`: clears the pending flag, then arbitrates
## (and, per the note below, fits) both docks.
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

## Recursive worker for `tablet_fit()`: raises buttons and text fields to the role minimum
## height and fonts labels to the tablet size pair, once per node (`_TABLET_FIT_META`).
## Only ever grows a size, never shrinks one.
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
	## **Centred vertically (owner decision 2026-10-26; it was left high on purpose until then).** `Button`
	## stacks icon over caption from the top of its content box and the cell's height is content-driven: the
	## caption's line box reserves about 7 px under its baseline, so the 44 px cell held a 35 px group with 1 px
	## above it and 8 px below (`_alignment_probe.gd`, 393x852; centre offsets -3.5 / -4.5 / -3.5 / -5.0 px for
	## Inspect / Measure / Region select / Pan, the spread being the captions' own descenders). A top margin of
	## `m` moves the group down by `m` AND grows the cell by `m` (its centre moves `m / 2`), so the group gains
	## `m / 2` on the centre: `m = 9 dp` leaves -0.5 .. +0.5 px on those four (8 dp left -1.0 on two), inside the probe's 1 px bar.
	## The cell therefore grows 44 -> 51-52 px at 393 wide (the owner expected about 49; the figure that centres
	## it is measured, not chosen). The cells live in the full-height left sheet, not in the detent sheet whose
	## height budget `_phone_map_sheet_content_height()` derives from content, so no detent changed. Same
	## margin on all three states, or the content jumps under the finger on press (see the side margins above).
	var top_pad: float = round(9.0 * unit)
	for state2 in ["normal", "hover", "pressed", "hover_pressed", "disabled"]:
		## `hover_pressed` (an armed tool under the finger or cursor, e.g. Pan) and `disabled` are NOT overridden
		## above and resolve to the shared theme's boxes: copy those before touching their margins, and give the
		## copy the same side margins so the content does not move sideways in that state either.
		if not b.has_theme_stylebox_override(state2):
			var own: StyleBox = b.get_theme_stylebox(state2).duplicate()
			own.content_margin_left = pad
			own.content_margin_right = pad
			b.add_theme_stylebox_override(state2, own)
		b.get_theme_stylebox(state2).content_margin_top = top_pad

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

## `node_added` hook (phone only): schedules ONE deferred `phone_fit()` of both docks when a
## `Control` is added inside either of them. The ancestor walk is needed because
## `child_entered_tree` fires for direct children only.
func _on_phone_node_added(node: Node) -> void:
	if _phone_fit_pending or not (node is Control):
		return
	## Only a dock descendant is our business. Walked rather than connected
	## per-panel because `child_entered_tree` fires for direct children only,
	## and every panel is several levels down.
	var p: Node = node.get_parent()
	while p != null:
		if p == left_dock or p == right_dock or p == _phone_dom_host:
			_phone_fit_pending = true
			_run_phone_dock_fit.call_deferred()
			return
		p = p.get_parent()

## Deferred body of `_on_phone_node_added()`: clears `_phone_fit_pending` and re-runs
## `phone_fit()` over both docks at the current `_phone_scale`.
func _run_phone_dock_fit() -> void:
	_phone_fit_pending = false
	for dock in [left_dock, right_dock, _phone_dom_host]:
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
## A 2026-09-12 change moved `W_DOCK_TABLET` to 320 without touching
## `civilization/planner` (out of that batch's scope) and was reverted
## 2026-09-13 for unrelated reasons (other dock content had not reflowed to
## 320) -- `W_DOCK_TABLET` is 400 throughout, so this 455 figure is unaffected
## either way. It still needs a fresh `_ldwidth_probe.gd --force-touch` before
## being cited again as current, regardless.
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

## `node_added` hook: schedules ONE deferred `dock_fit()` of both docks when a `Control`
## arrives inside either, so panels attached after boot are width-fitted too.
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

## Deferred body of `_on_dock_node_added()`: clears `_dock_fit_pending` and runs
## `dock_fit()` over both docks.
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

## Read-only accessor for the phone-px to physical-px factor (`_phone_scale`), for callers
## outside this file; they must not compute their own.
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
## MS-4 replaced that screen: the phone's route is now MENU ▸ File ▸ New world…, the File popup's first row
## (`phone_menu.gd::_fill_popup()`), and `new_world_route()` says so.
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
	return "MENU ▸ File ▸ New world…" if DccTheme.is_phone() \
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

## Whether the phone's status region is currently shown (`_phone_status_shown`).
func is_status_region_shown() -> bool:
	return _phone_status_shown

## Shows or hides the phone's status region and re-applies the orientation pass so the
## visibility rule lives in one place.
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
			_rail_seps[String(d.id)] = sep

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
			var head_rows: Array = [hp]
			var sub := _domain_subtitle(String(n["domain"]))
			if sub != "":
				var sub_row := _rail_header_subtitle(sub)
				col.add_child(sub_row)
				head_rows.append(sub_row)
			_rail_exp_heads[String(n["domain"])] = head_rows
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

## Whether the desktop rail is in its expanded (labelled) state (`_rail_expanded`).
func is_rail_expanded() -> bool:
	return _rail_expanded

## Rotates the rail's single chevron to 180 degrees when expanded and 0 when collapsed,
## about its own centre. Safe to call before the chevron exists or after it is freed.
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

## Positions one rail button's rotated label inside the rail and sizes the button to fit it.
## `text_size.y` is the horizontal extent after the -90 degree rotation. Needs the label in the
## tree so its minimum size is measurable; see `_relayout_rail_labels()`.
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

## Makes `id` the active domain: restyles every rail/bar cell (active fill, ink, glyph) and
## switches the visible workspace. Does not itself handle the re-click-expands-the-rail
## case; that is the press handler's job.
func _select_domain(id: String) -> void:
	## SM-7: Session mode's one choke point (see its block near the top of this
	## file). Refused before anything is written, so a refused route leaves the
	## domain, its mode and the rail exactly as they were.
	if _session_refuses(id):
		return
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
		_refresh_phone_sheet_body()
	## §5.2's chip reads the domain and its mode, and `select_domain_mode()`
	## writes the mode before calling here -- so this one call site covers both.
	## No-op until the viewport exists (`DccShell._ready()` runs as
	## `super._ready()`, before `app.gd` builds it); the deferred call at the foot
	## of `_ready()` is what draws it the first time.
	_refresh_viewport_context()
	## MS-1 S4.2: not while a composition is being mounted. `_mount_composition()` calls `_select_domain()` before
	## the app's workspaces and right dock exist, and a boot has no listener yet; a remount has them all (the app's
	## `_on_workspace_changed()` reads `right_dock_ctrl`), so the mount holds the signal back and the caller makes
	## the one call the listeners need once everything is built (`DccApp._remount_composition()`).
	if not _mounting_composition:
		workspace_changed.emit(id)

## True from after `_unmount_composition()` to the end of the mount: `_select_domain()` then does not emit
## `workspace_changed`. Written only by a remount (boot never sets it: nothing listens that early).
var _mounting_composition := false

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
	## SM-7: refused here as well as in `_select_domain()`, because this writes
	## `_domain_mode[id]` before it gets there -- a hidden domain's mode must
	## come back untouched when Session mode is turned off.
	if _session_refuses(id):
		return
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
## eight of the ten nodes carry no `shows` at all, and a gate that defaulted to
## an empty allow-list would blank eight docks. Absent means ungated, which is
## why this returns the key's own value untouched rather than something derived
## from `owns` -- `owns` answers *which node lights*, and every category is in
## exactly one `owns` list, so deriving a gate from it would leave CIVIL showing
## only the categories one node happens to own -- one header in `landmarks`, two
## in `infra`, one in `planner`. WORLD's two nodes are where the two keys happen
## to coincide since Ruling L, and that coincidence is not the rule.
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

## The `mode` of the first node of domain `id` whose `owns` list contains `category`; the
## empty string when none does, so callers must check.
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
	## `_refresh_phone_sheet_body()` correctly declines. The result on a real
	## handset was the app booting to the one-line tool-options strip with the
	## GENERATE tab lit over it: *"one line that barely scrolls properly and a
	## lot of white space"* (the sheet body is the dock's own categories now, Ruling EH item 1, but the
	## boot-order trap is the same). Measured on the device (`9608b26b`,
	## ONEPLUS_A6013), not on a desktop probe -- the probe taps the tab, which
	## is exactly the path that hid this.
	##
	## Deferred, for `register_workspace()`'s own reason two lines below: the
	## panel arriving here is a bare `WorldWorkspace.new()` whose `setup()` has
	## not run, so filling the sheet from it now would read an empty parameter
	## table.
	_refresh_phone_sheet_body.call_deferred()
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

## The id of the currently active domain (`_active_domain`).
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
	## SM-7: same reason as `select_domain_mode()` -- this writes the mode first.
	if _session_refuses(id):
		return
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
## Touch value of `--ctl` (36 px), the same figure on both touch canvases; `CTL_PX` is the pointer value.
const CTL_PX_TOUCH := 36
## The two literal paddings in the dock-header blocks. Literal `px` in the
## canvas, not tokens -- only the horizontal `var(--pad)` moves with density --
## so they do not scale.
const DOCK_HEAD_PAD_TOP := 8
## Owner, 2026-10-26: the left and right header dividers must sit at one height, so the two docks share ONE
## header height. The canvas's `ENV:305` gives the left `padding:8px var(--pad) 0` (32 px, divider 6 px
## higher than the right's); the owner's measured finding outranks it (`DCC_SHELL_SCOPE.md`: an owner
## decision is newer than any canvas). 6, the right dock's own figure, so neither strip loses height.
## Pinned by `_alignment_probe.gd` (divider y equal within 0.5 px at desktop and tablet).
const DOCK_HEAD_PAD_BOTTOM_LEFT := 6
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
	_left_dock_mode_switch = _build_mode_switch()
	col.add_child(_left_dock_mode_switch)
	_left_dock_col = col

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
		## MS-7: the tablet's slot (and a floating dock's overlay) follows the dragged width.
		_sync_dock_slot(is_left)

## Builds one dock's collapse/expand square (glyph-only, so it carries a tooltip and an
## accessibility name) and registers it in `_collapse_buttons` under `left`/`right`.
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
# derived text is `LANDMARKS` / `FACTIONS` / `ROUTES & WAYS` /
# `JOURNEY PLANNER` for CIVIL (9/8/13/15 characters over four halves) and
# `STYLE` / `LAYERS` / `LABELS` / `ICONS` for CARTO (5/6/6/5, shortened by
# Ruling L from `LAYERS & STYLE`/`LABELS`/`ICONS`/`TERRAIN APPEARANCE`, and not
# re-measured since). Those are legible strings, not placeholders, and whether four of
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
# Bold overflows. CIVIL's column was measured with the labels before Ruling L
# (2026-09-13) shortened `FACTIONS & SETTLEMENTS` to `FACTIONS`, and has not been
# re-measured since. WORLD's shipping pill clears every density with room; CARTO's
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
## Per-node pill text overrides keyed `domain/mode`, for nodes whose rail label differs from the pill text.
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

## Builds the per-domain mode pill (band 2 of the left dock, hidden until a domain with two
## modes is active) and records its row and pill in `_mode_switch_row`/`_mode_switch_pill`.
## Only the empty container is built here; segments are filled per domain later.
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

## Builds the label shown in place of a collapsed dock's body and stores the inner `Label` in
## `_dock_readouts[side]`; returns the hidden wrapping container. Text arrives via
## `set_dock_readout()`.
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

## Whether the dock on `side` (`left`, or anything else meaning right) is collapsed.
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
	## MS-7: the tablet's slot, tab and overlay follow the flag just written.
	_sync_dock_slot(is_left)

# -- §9 Viewport --------------------------------------------------------------

## Builds the map region: a panel in `bg` that expands horizontally, holding a clipping `viewport_content` Control, and records both in `viewport_area` and `viewport_content` for the viewport host to fill. Shared by the desktop shell and the phone shell (where it fills the whole screen under the chrome). Returns the panel.
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
# `valsCore()`), a four-arm fall-through (this port adds one variant, PAINT, inside
# arm 2; the design has no Paint chip):
#
#   1  a run is active            GENERATING — STAGE NN
#   2  WORLD, sculpt mode         SCULPT · DRAFT   (PAINT · DRAFT while Paint is armed; a port addition)
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
## True once the generation-stage signals of the engine bridge are connected, so a later
## re-bind does not double-connect.
var _vp_wired := false

## Composes and pushes §5.2's chip. Cheap enough to call from every signal that
## can change it; the one non-trivial read is `stale_stages()`, and
## `EngineBridge.mark_dirty()` early-returns once already dirty, so
## `params_changed` fires on the clean->dirty transition and not per drag frame.
func _refresh_viewport_context() -> void:
	## `_refresh_phone_sheet_header()`'s own WORLD case reads the same
	## seed/mode state this function does (world mode, `bridge.world_gen`'s
	## seed) and needs the same trigger set -- a dice roll or a world landing
	## while GENERATE is already the lit tab, not only a tab press. Riding this
	## function's existing wiring rather than a second `generation_started` /
	## `world_loaded` connection to the same bridge; unconditional and ahead of
	## the `host == null` return below, since the header needs no viewport.
	##
	## A direct, synchronous call -- deliberately, after two deferred variants
	## (`call_deferred`, then a full `process_frame` boundary) both still
	## panicked reaching into `world_gen` from a `generation_stage` tick, which
	## fires on the MAIN thread WHILE the worker thread still owns `world_gen`
	## for its entire run (`engine_bridge.gd::_worker()`'s own doc comment).
	## Neither timing fix could have worked: the unsafe window is the whole
	## generation, not one reentrant instant. `_refresh_phone_sheet_header()`
	## itself now carries the real guard (`bridge.generating`): the engine's `world_gen` belongs to the worker
	## for the whole run (the retired phone column's `_pg_rebuild()` had the same guard for the same object).
	_refresh_phone_sheet_header()
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
		## Arming or disarming Paint changes arm 2's wording (above), and no
		## domain/mode write accompanies it on the phone's PAINT segment
		## (the retired phone column's mode segment selected the mode BEFORE it armed), so the chip has to
		## follow the tool itself. `tool_armed` is `DccApp`'s signal; this class
		## is also instantiated bare by older probes, hence `has_signal`.
		if has_signal("tool_armed"):
			connect("tool_armed", func(_id: String): _refresh_viewport_context())

	var text := ""
	var tip := ""
	if bridge != null and bridge.generating:
		## Arm 1. The index is absent for the stretch between `generation_started`
		## and the first stage tick, so the chip says the state without inventing
		## a stage number for it.
		text = ("GENERATING — STAGE %02d" % _vp_stage) if _vp_stage > 0 else "GENERATING"
		tip = "A generation run is in flight. Stage numbers come from the engine's own generation_stage tick."
	elif _active_domain == "world" and active_mode("world") == "b" and String(get("armed_tool")) == "paint":
		## Arm 2's PAINT form -- a deliberate divergence from the design's string
		## (DECISIONS.md 7p). WORLD's mode `b` holds BOTH Sculpt and Paint
		## (`mode_for_category()`), and the design, which has no Paint tool in that
		## mode, reads `SCULPT · DRAFT` for all of it. With Paint armed that is a
		## false sentence: the draft is paint cells, not stamps. Keyed on the armed
		## tool, not the mode, so Sculpt (and the idle mode-`b` dock) keeps arm 2
		## verbatim below. `get()` because `armed_tool` lives on `DccApp`, which
		## extends this class. Refreshed by the `tool_armed` hook wired below.
		text = "PAINT · DRAFT"
		tip = "Paint is armed. Painted cells are a draft until they are committed; the painted layers under them are unchanged."
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

## Builds the timeline strip region and stores its row as `timeline_row` (workspaces fill it). It deliberately sets no fixed height: the collapsed and expanded forms differ and a single height was wrong for both, so the content drives it; the caller assigns the result to `timeline_bar`. Returns the bar.
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
# track is continuous and honest -- the cursor really does land where the
# playhead is.
#
# **The territory under it changes only at the years CIVIL ▸ Timeline has
# recorded** (Ruling AT, 2026-09-24). A recorded year loads its snapshot and
# makes it the Territory tool's base, dropping paint and any pending stroke, so
# the next stroke, subtract or recompute edits that year. An UNRECORDED year
# moves the cursor and nothing else: territory, base and paint stay as they
# were. That departs from the reference's `terr.fill(0)` deliberately, so
# scrubbing or playing across the track cannot erase unsaved territory. The
# claims shown between marks are therefore those of whichever recorded year
# last loaded them -- not always the mark below the cursor, since stepping back
# from 705 to 450 keeps 705's -- and `tl_territory_year()` names it.

## `05-right-dock-and-bars.md` §4.2: "the scrub range is therefore fixed at
## year -400 ... year 1200 (1600 years)", and `06-phone.md` §6.2's slider is
## `min=-400 max=1200 step=1`. The same two numbers, in both canvases.
##
## **Since PC-3 these two are the track's FALLBACK, not its range.** The strip's range is taken from the
## world (`tl_range()`, finding T3 of `PC_SHELL_PROPOSAL.md`), and these answer only for a world with no
## recorded year and no conflict, where there is nothing to take a range from. `tl_range()` says which it
## is (`from_world`), so a caller never mistakes this axis for the world's own.
const TL_YEAR_MIN := -400
## Top of the fallback track (see the note above). Playback stops at `tl_range_max()`, which is this
## only when the world gave no range.
const TL_YEAR_MAX := 1200
## The narrowest track `tl_range()` draws from world data, in years. A world whose only recorded year is 340
## has a span of zero, and a zero-width track cannot place a playhead. A labelled judgement (PC-3); 100 is
## about the width at which two marks one decade apart are still a few pixels apart on a 700 px track.
const TL_MIN_SPAN := 100
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
## Playback tick interval in seconds, a labelled judgement; each tick advances the cursor by `tl_speed` years.
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
## `workspaces/civilization_workspace.gd` connects (`_tl_on_cursor_moved`), as do the Place editor
## and the right dock (`on_timeline_changed`) -- PC-2. A reader that shows the year must connect
## here, and a writer of the year must call `tl_set_year()`, which emits it.
signal timeline_changed()

## PC-3. Emitted when what the strip's expanded rows list changed WITHOUT the year moving: a year recorded
## or removed, a Timeline filter flipped (CIVIL > Timeline and the strip share the flags, which live in
## `CivilizationWorkspace`), or the Timeline body rebuilt. The strip rebuilds its rows on it, deferred and
## coalesced, so it never frees a node mid-click. A move of the year is `timeline_changed`, not this.
signal timeline_records_changed()

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

## Names the underlying no-op: `civ_goto_year` before any generate.
const TL_UNAVAILABLE := "Generate a world first. The year cursor is civilisation state, so moving it now would silently do nothing."

## The cursor. `0` with no engine and before any generate, which is
## `CivData::year`'s own init value rather than a stand-in for it.
func tl_year() -> int:
	var bridge := _find_engine_bridge()
	return 0 if bridge == null else bridge.get_civ_year()

## **The one writer of the year** (`PC_SHELL_PROPOSAL.md` PC-2, finding T1). Every move of
## the cursor -- the strip's scrub, step and playback, CIVIL > Timeline's pills, scrub, Step and
## Animate, the Place editor's year chips and Go to year -- comes through here, and this is the
## only function in `shell/` that calls `bridge.civ_goto_year()` (`grep -rn "civ_goto_year(" shell/`
## finds the wrapper's definition in `engine_bridge.gd` and this one call). It must never be
## bypassed: the engine write alone moves no reader, and a surface that wrote the year itself
## left the strip, the dock's garrison row and the journey markers on the old year.
##
## Clamped to the track by default, because both canvases draw a fixed -400..1200 axis and a
## playhead outside it has nowhere to be. **`exact`** skips the clamp for a year that is a
## *value* rather than a pointer on the track -- a recorded year (the CIVIL dock lets one be
## added anywhere in -9999..9999) or a year typed in the Place editor (+-100000). Clamping those
## would load the snapshot of the wrong year, which is what the pre-PC-2 direct calls never did.
## The strip draws its playhead from `tl_year()` unclamped; PC-3 takes the range from the world
## (finding T3).
func tl_set_year(year: int, exact := false) -> void:
	if not tl_available():
		return
	var bridge := _find_engine_bridge()
	if bridge == null:
		return
	var r := tl_range()
	bridge.civ_goto_year(year if exact else clampi(year, int(r["min"]), int(r["max"])))
	timeline_changed.emit()

## **The strip's range, taken from the world** (`PC_SHELL_PROPOSAL.md` section 3.2 "One range", finding T3;
## PC-3). `{"min": int, "max": int, "from_world": bool}`: the earliest to the latest of the world's own dated
## things -- every recorded year (`tl_recorded_years()`) and every conflict's start and, when it has one,
## end (`bridge.conflict_list()`) -- widened to `TL_MIN_SPAN` when they sit closer than that.
##
## **`from_world` false means "the world gave no range", and then `min`/`max` are the fallback axis
## `TL_YEAR_MIN`..`TL_YEAR_MAX`, not a range**: a fresh world has no recorded year, and a track drawn from
## nothing must say so rather than pass an arbitrary span off as the world's (`MISTAKES.md`, never encode
## "no value" as a plausible value). The strip keeps the fallback track so a fresh world can still be
## scrubbed, labels its range literals `default axis`, and no caller may treat the pair as data.
##
## The cursor is deliberately NOT folded in: a range that moved when the playhead did would re-map every mark
## while it was dragged. A cursor outside the range (an exact write such as the Place editor's Go to year)
## pins the playhead to the nearest end and the readout still prints the true year.
## No settlement founding year is a source: the engine records none (the settlement dictionary carries no
## founding field), so the proposal's "earliest founding" is not built. The selection's chronos entries are
## drawn as ticks (`tl_selection_events()`) and are not a range source either, so selecting a place never
## moves the axis under the user.
func tl_range() -> Dictionary:
	var years: Array = []
	for y in tl_recorded_years():
		years.append(int(y))
	var bridge := _find_engine_bridge()
	if bridge != null and bridge.has_world:
		for c in bridge.conflict_list():
			var cd: Dictionary = c
			if cd.has("start_year"):
				years.append(int(cd["start_year"]))
			if cd.has("end_year"):
				years.append(int(cd["end_year"]))
	if years.is_empty():
		return {"min": TL_YEAR_MIN, "max": TL_YEAR_MAX, "from_world": false}
	var lo: int = years[0]
	var hi: int = years[0]
	for y in years:
		lo = mini(lo, int(y))
		hi = maxi(hi, int(y))
	if hi - lo < TL_MIN_SPAN:
		## Widened by the shortfall, split as evenly as integers allow, so the data sits mid-track.
		var short := TL_MIN_SPAN - (hi - lo)
		lo -= short / 2
		hi += short - short / 2
	return {"min": lo, "max": hi, "from_world": true}

## The track's first year (`tl_range()["min"]`).
func tl_range_min() -> int:
	return int(tl_range()["min"])

## The track's last year (`tl_range()["max"]`); playback stops here.
func tl_range_max() -> int:
	return int(tl_range()["max"])

## **The selection's own dated entries, for the strip's ticks and event list** (PC-3, finding T5): the
## chronos events of the notes attached to the selected settlement or faction
## (`bridge.vault_entity_chronos`, Ruling AM). `[{"start": int, "end"?: int, "name": String, "rel": String}]`,
## ascending by start; `[]` for no selection, a kind with no notes, a vault that is not reachable, or an
## unreadable note (an empty list is "nothing to draw", never a year). `end` is absent for a single-year
## entry, as the engine omits it. Reads `selection` through `get()`, because the shell base does not declare it.
func tl_selection_events() -> Array:
	var sel: Variant = get("selection")
	if not (sel is Dictionary) or (sel as Dictionary).is_empty():
		return []
	var bridge := _find_engine_bridge()
	if bridge == null or not bridge.has_world:
		return []
	var kind := String((sel as Dictionary).get("kind", ""))
	var id := int((sel as Dictionary).get("id", -1))
	if kind == "settlement":
		var all: Array = bridge.settlements()
		if id < 0 or id >= all.size():
			return []
		id = int((all[id] as Dictionary).get("tid", 0))
		if id == 0:
			return []
	elif kind != "faction":
		return []
	var got: Dictionary = bridge.vault_entity_chronos(kind, id)
	if not bool(got.get("ok", false)):
		return []
	var out: Array = []
	for e in got.get("events", []):
		var ed: Dictionary = e
		if not ed.has("start"):
			continue
		var item := {"start": int(ed["start"]), "name": String(ed.get("name", "")), "rel": String(ed.get("rel", ""))}
		if ed.has("end"):
			item["end"] = int(ed["end"])
		out.append(item)
	return out

## Every conflict that has a start year, as `[{"id": int, "name": String, "start": int, "end"?: int}]`
## (`bridge.conflict_list()`); `end` is absent for an open-ended conflict. The strip draws each as a span
## and lists it. `[]` before a world.
func tl_conflict_spans() -> Array:
	var bridge := _find_engine_bridge()
	var out: Array = []
	if bridge == null or not bridge.has_world:
		return out
	for c in bridge.conflict_list():
		var cd: Dictionary = c
		if not cd.has("start_year"):
			continue
		var item := {"id": int(cd.get("id", -1)), "name": String(cd.get("name", "")), "start": int(cd["start_year"])}
		if cd.has("end_year"):
			item["end"] = int(cd["end_year"])
		out.append(item)
	return out

## Moves the year cursor one step in `direction` (-1 or +1) of `tl_speed` years; `tl_set_year()`
## clamps and no-ops before a world exists.
func tl_step(direction: int) -> void:
	tl_set_year(tl_year() + direction * tl_speed)

## The recorded year the claims on screen last loaded from, `{"year": int}`,
## or `{}` when they did not come from one (see §10a above). Omitted rather
## than defaulted, for the reason `tl_year_neighbours()` gives.
func tl_territory_year() -> Dictionary:
	var bridge := _find_engine_bridge()
	return {} if bridge == null else bridge.get_civ_territory_year()

## The cursor's day within its year (`STORY_PLANNING_SCOPE.md` SP-2, Ruling
## AO): 0-based in `cartalith_vault::chronos`'s 365-day, no-leap calendar,
## held by the engine beside `CivData::year` (`WorldGen::civ_day`) so this
## file still keeps no date of its own. Only saved journeys read it -- a year
## is too coarse to place a party on a weeks-long route. The year controls
## above leave it alone.
func tl_day() -> int:
	var bridge := _find_engine_bridge()
	return 0 if bridge == null else bridge.get_civ_day_of_year()

## Sets the cursor's day within its year, clamped to the engine's 0-364 (no-leap) calendar,
## and emits `timeline_changed`. Does nothing before a world is generated.
func tl_set_day(day_of_year: int) -> void:
	if not tl_available():
		return
	var bridge := _find_engine_bridge()
	if bridge == null:
		return
	bridge.civ_set_day_of_year(clampi(day_of_year, 0, 364))
	timeline_changed.emit()

## The cursor as `YYYY-MM-DD`; `""` before any world.
func tl_date_text() -> String:
	var bridge := _find_engine_bridge()
	return "" if bridge == null else String(bridge.get_civ_date().get("text", ""))

## Every year CIVIL ▸ Timeline has recorded a snapshot for, ascending.
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
## `has()`. Board D's readout renders `next` from this dictionary; its
## *territory holds at 412 AD* comes from `tl_territory_year()` instead (Ruling
## AT), which equals `prev` when the cursor moved forward from that mark and
## not when it stepped back from a later one. Board F's cursor at 1200 has no
## `next` and that part is dropped rather than printed empty.
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

## Selects the playback ladder step; ignores any value not in `TL_SPEEDS`. Updates the running
## timer's interval and emits `timeline_changed`.
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
	if tl_playing and tl_year() >= tl_range_max():
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

## Playback timer callback: advances the cursor by `tl_speed` years, or on reaching
## `TL_YEAR_MAX` snaps to it, stops playback and the timer (no wrap-around).
##
## **Holds while `generating`** (mid-generation sweep, 2026-10-12): the timer keeps firing under
## the busy overlay, which blocks input only, and `tl_year()` / `tl_set_year()` reach
## `get_civ_year` / `civ_goto_year`, `#[func]`s the worker thread holds (the `bind() failed`
## panic), and `timeline_changed` fans out to listeners that read the engine too. The tick is
## skipped, not playback stopped: the next tick after the run advances from where it held.
func _tl_tick() -> void:
	var gen_bridge := _find_engine_bridge()
	if gen_bridge != null and gen_bridge.generating:
		return
	var next := tl_year() + tl_speed
	var top := tl_range_max()
	if next >= top:
		tl_set_year(top)
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
	if tl_year() >= tl_range_max():
		return "end of track"
	return ("playing ×%d" % tl_speed) if tl_playing else "paused"

## Flips one timeline layer toggle, persists all of them and emits `timeline_changed`.
## Unknown ids are ignored.
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

## Writes every timeline layer toggle into the `timeline` section of the user config file.
## Loads the file first so unrelated sections are preserved; must never replace the whole file.
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
## True once the status bar's progress signals are connected; `_wire_status_progress()`
## returns early when set.
var _status_wired := false
var _status_tail: Control           ## the clipped, minimum-0 box holding `mid` + `hint`

## Give the `hint` label an explicit width: its natural text width, capped to the
## room the tail has left after `mid` and the gap between them. Called whenever
## either slot's text or visibility changes and whenever the tail is resized.
##
## Why a stated width and not the label's own minimum: `clip_text` /
## ellipsis collapses a `Label`'s minimum width to 1 (`MISTAKES.md`, "Add
## `clip_text` / ellipsis to a Label", measured), and a non-expanding label of
## minimum 1 draws 1 px wide, i.e. not at all. `custom_minimum_size.x` here is
## that missing figure. It can never raise the shell's minimum width: it is
## capped by the tail's own width, and the tail is a plain `Control` whose
## minimum is 0, so the cap does not depend on the figure it caps (no feedback
## loop). With room to spare the figure is the natural width, so the full
## sentence draws exactly as before (END-aligned in the tail); without it the
## sentence is ellipsised and the tail's tooltip carries all of it.
##
## Must never: read the hint's own `get_minimum_size()` (that is the collapsed 1),
## or size from the window (the tail's width is the only honest budget).
func _fit_status_tail() -> void:
	if _status_tail == null or not _status_labels.has("hint"):
		return
	var hint: Label = _status_labels["hint"]
	var mid: Label = _status_labels["mid"]
	_status_tail.tooltip_text = hint.text
	var natural := 0.0
	if hint.text != "":
		var font: Font = hint.get_theme_font("font")
		var fs: int = hint.get_theme_font_size("font_size")
		natural = ceilf(font.get_string_size(hint.text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x) + 1.0
	var room := _status_tail.size.x
	if mid.visible:
		room -= mid.get_combined_minimum_size().x + float(STATUS_SLOT_GAP)
	hint.custom_minimum_size.x = clampf(minf(natural, room), 0.0, natural)

## Build the 26 px status bar: one cell per `STATUS_SLOTS` entry, then the
## `_status_tail` holding `mid` and `hint`. Returns the bar's `PanelContainer`.
## Must never let any label's text raise `status_row`'s minimum width through
## the tail -- see `_fit_status_tail()`.
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
	## The tail replaces what used to be a bare `DccTheme.spacer()` followed by
	## the `mid` and `hint` labels as three more children of `status_row`. That
	## layout let a long `hint` sentence (the catchment one, 1 703 px bar minimum
	## at a 1600 px window, `OUTSTANDING_WORK.md`) become part of the HBox's
	## minimum width and widen the whole shell. The tail is a plain `Control`
	## (its minimum width is 0 whatever its children want, because a `Control`
	## does not fold its children's minimums into its own the way a container
	## does), expanding to the same space the spacer took, clipped, and holds an
	## END-aligned HBox of `[mid][hint]`. `_fit_status_tail()` then gives `hint`
	## the one minimum width that cannot go wrong: its natural width, capped to
	## what is left in the tail. That is the fix `MISTAKES.md`'s `clip_text` row
	## demands -- `clip_text` alone collapses the label's minimum to 1, and a
	## non-expanding label with a minimum of 1 is drawn 1 px wide (it vanishes),
	## so the label's width is stated rather than left to its own minimum.
	## `status_row` still has exactly one child per slot plus this one, so
	## `app.gd`'s `move_child(_stale_recompute, 2)` arithmetic is unchanged.
	_status_tail = Control.new()
	_status_tail.name = "StatusTail"
	_status_tail.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_status_tail.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_status_tail.clip_contents = true
	_status_tail.resized.connect(_fit_status_tail)
	status_row.add_child(_status_tail)
	var tail_row := HBoxContainer.new()
	tail_row.name = "StatusTailRow"
	tail_row.alignment = BoxContainer.ALIGNMENT_END
	tail_row.add_theme_constant_override("separation", STATUS_SLOT_GAP)
	tail_row.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_status_tail.add_child(tail_row)
	## `statusMid` (`05-right-dock-and-bars.md` §3.3, `BUILD_ANSWERS.md` §2.2):
	## `var(--dis)`, END-aligned in the tail just before the key hints. Composed in
	## `app.gd::_refresh_status_mid()` -- this file only reserves the slot, the
	## same as every other one here.
	var mid := DccTheme.mono_label("", "text_ghost", DccTheme.role_px("fs_status"), 0)
	_status_labels["mid"] = mid
	mid.visible = false
	tail_row.add_child(mid)
	var hint := DccTheme.mono_label("", "text_faint", DccTheme.role_px("fs_status"), 0)
	_status_labels["hint"] = hint
	hint.visible = false
	## Ellipsis when the sentence does not fit; the full sentence is the tail's
	## tooltip (`_fit_status_tail()`). Safe only because that function states
	## the label's width -- see the comment above on the `clip_text` trap.
	hint.clip_text = true
	hint.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	tail_row.add_child(hint)
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

## The default ink token for a status slot: the menu-bar strip's `text_dim` for `top_`
## slots, else the status bar's `text_faint` (see the canvas rows cited above).
static func _status_default_token(slot: String) -> String:
	return "text_dim" if slot.begins_with(_STATUS_TOP_PREFIX) else "text_faint"

## Writes `text` into a status or menu-bar slot and inks it with `token` (the slot's strip
## default when empty). An unknown slot logs an error and returns. May re-run the status-bar
## omission and tail-fit passes; must never be given a colour literal in place of a token name.
func set_status(slot: String, text: String, token: String = "") -> void:
	if not _status_labels.has(slot):
		push_error("DccShell: no status slot '%s'" % slot)
		return
	var ink: String = token if token != "" else _status_default_token(slot)
	var l: Label = _status_labels[slot]
	_status_tokens[slot] = token
	l.text = text
	l.add_theme_color_override("font_color", DccTheme.c(ink))
	if slot == "autosave":
		## `ink`, not `token` -- an omitted argument is now the empty string,
		## and `DccTheme.c("")` is not a colour. The dot must follow whatever
		## the label was actually painted with, which is what `ink` is.
		_paint_status_dot(ink)
	if _status_cells.has(slot) or STATUS_TAIL_SLOTS.has(slot):
		_apply_status_omission()
	if STATUS_TAIL_SLOTS.has(slot):
		_fit_status_tail()

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
## MS-1 S4.2: the token each slot was last written with (`""` means the slot's default ink), so a rebuilt status
## bar can be refilled exactly: `status_snapshot()` and `restore_status()`. Survives a composition, like the text it
## accompanies is carried across it (`S4_INVENTORY.md` section 15.5).
var _status_tokens: Dictionary = {}

## Every status slot's text and the token it was last written with, as `{slot: {"text", "token"}}`. Read before
## `_unmount_composition()`, because the labels hold the only copy of the text.
func status_snapshot() -> Dictionary:
	var out := {}
	for slot in _status_labels:
		out[slot] = {"text": (_status_labels[slot] as Label).text, "token": String(_status_tokens.get(slot, ""))}
	return out

## Writes a `status_snapshot()` back through `set_status()` (so the omission rule, the autosave dot and the tail fit
## all run), in the slot order the snapshot has. A slot the new bar does not have is skipped, never an error.
func restore_status(snap: Dictionary) -> void:
	for slot in snap:
		if _status_labels.has(slot):
			set_status(String(slot), String(snap[slot]["text"]), String(snap[slot]["token"]))

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

## Builds the phone composition under `_phone_root`: the map full-bleed first, then the menu bar and status bar parked in a hidden `PhoneMenuModel` host (the model `phone_menu.gd` reads, not a view; built first on purpose, because the app bar's 'top_world' status label must be registered after it and win), then the chrome column (safe-area strip, app bar, content gap, tool sheet, timeline, phone menu bar, gesture inset), the side safe strip and the search overlay, and the remaining sheets. Many members are assigned here and read later by the phone paths, so the ordering inside it is load-bearing.
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
	## after the user dismisses that. On the default WORLD domain tab the empty shell
	## already carries `Generate world` as an accent-filled primary in the dock's Run section (the
	## sheet body since Ruling EH item 1), so an empty-state signpost drawn into this gap would be a second
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
	_phone_menu.closed.connect(_on_phone_menu_closed)

	## CM-6: the armed-tool pill lives in `_phone_content_gap` (clear of the
	## tool sheet AND the bottom nav, the same bound `_build_phone_undo_chip()`
	## chose for the same reason); the fan itself is a top-level overlay
	## `ToolFan.setup()` parents onto `self` (`app`), drawn above everything
	## already built by tree order, same as `radial_ring.gd`'s own ring.
	_tool_fan = ToolFan.new()
	_tool_fan.setup(self)
	## MS-5 second step: the pill is the chip of the tool strip, which now sits in `_phone_content_gap`.
	_tool_strip = ToolStrip.new()
	_tool_fan.strip = _tool_strip
	_tool_strip.setup(self, _tool_fan, _phone_content_gap)

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
## which is what the ruling says adopting this canvas scopes. **A real world
## name now exists** (`OUTSTANDING_WORK.md`'s Recent-worlds row, name half:
## `cartalith_civ::naming::world_name`, `WorldGen::get_world_name()`, `app.gd`
## `_world_pill_text()` — no more `"ELDRA"` literal), but that resolves only
## the *content* question, not the *layout* one this paragraph is actually
## about: the pill still has nowhere to put a name as its own line, because
## splitting it needs the same stage-3 bar rebuild the paragraph above
## describes. So the pill keeps drawing `CARTALITH` over that slot rather
## than reusing half the one `top_world` line it does have for a name the
## other half already carries the seed for.
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
	## `set_status("top_world", _world_pill_text())`, `"<name> · <seed>"` off
	## the real generated name now, not a literal) -- no phone-aware
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
		## `PRESET_CENTER` moves only the ANCHORS to the cell's centre; the offsets stay 0, so the icon's
		## top-left corner sat on the centre and the 16 px glyph drew 8 px right and 8 px low
		## (measured 2026-10-26, `_alignment_probe.gd`, 393x852). Half the icon's own size either side
		## of the anchor puts its centre on the cell's.
		var half := ic.custom_minimum_size * 0.5
		ic.offset_left = -half.x
		ic.offset_right = half.x
		ic.offset_top = -half.y
		ic.offset_bottom = half.y
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
	## MS-2 (draft 03): `_build_phone_tab_cell()` builds each cell, and no tab registers in `_domain_buttons` /
	## `_domain_marks` any more: the old GENERATE and MAP cells were the only phone entries, and the one domain
	## cell stands for three domains, so `_refresh_phone_tabs()` lights and captions every cell itself.
	## MENU is `cells`' own second child (the place MORE had, which `Window ▸ Domain rail` never hides);
	## every other tab goes in the box that row hides.
	for t in PHONE_TABS:
		var cell := _build_phone_tab_cell(t)
		var key := String(t.id)
		_phone_tab_cells[key] = cell
		(cells if key == "menu" else dests).add_child(cell["button"] as Control)
	return bar

## The phone's four tabs (`MOBILE_SHELL_PROPOSAL.md` section 2.1; Rulings DT and DU): `MAP · <domain> · STORY ·
## MENU`, replacing `MAP · GENERATE · PLAN · MORE`. Built from `drafts/mobile-shell/03` (MS-2).
##
## The domain tab stands for WORLD, CIVIL and CARTO: its caption and glyph follow the current domain
## (`DOMAIN_TAB`), a tap opens that domain's sheet, a hold opens the domain fan, a sideways swipe cycles
## (`_on_domain_cell_input()`). STORY opens `PhoneMenu` on its Story screen (the four bands; JOURNEYS holds
## the journey planner the PLAN tab opened). MENU is the old MORE overlay (`_toggle_overflow()`).
## MAP still selects CARTOGRAPHY until MS-3 builds its own sheet.
##
## **Where the old tabs' targets went (MISTAKES.md, "Delete a surface")**: GENERATE is the domain tab while
## the domain is WORLD; PLAN's planner is STORY ▸ Journeys ▸ Journey planner; CIVIL, which had no tab, is the
## domain tab's second stop; MORE is MENU. Nothing is dropped.
##
## `glyph` is literal text, not an icon key (2026-09-13): the canvas draws each tab's glyph as plain mono text
## (`Cartalith Android.dc.html`), rendered by `_phone_bar_cell()`. The older notes in this file that say
## "four task tabs MAP · GENERATE · PLAN · MORE" describe the bar before MS-2.
##
## `tip` is the sheet header's subtitle for MAP (byte-for-byte the canvas's `titles.map[1]`) and the tab's
## tooltip; the domain tab's tip is its tooltip only, the sheet header reading `DOMAIN_TAB`.
## The four tabs (Ruling DT Q1 and Q2). The domain tab's caption and glyph follow the current domain
## (`DOMAIN_TAB`), so its row carries neither.
const PHONE_TABS: Array = [
	## `tip` stays the canvas's own `titles.map[1]` until MS-3 builds MAP's sheet (draft 03 had "View · fields ·
	## layers", the proposal's wording for that future sheet; `_detent_probe` pins the canvas string).
	{"id": "map", "caption": "MAP", "glyph": "▤", "tip": "layers · style · annotation"},
	{"id": "domain", "caption": "", "glyph": "",
		"tip": "The current domain. Hold, or swipe sideways, to switch WORLD · CIVIL · CARTO"},
	{"id": "story", "caption": "STORY", "glyph": "➔", "tip": "Time · Journeys · Conflicts · Notes"},
	## `tip` says what the root holds today (the MORE root, retitled); MS-4 replaces it with the eight desktop menus
	## and this line with "The eight desktop menus, Quick and Status" (draft 03's text).
	{"id": "menu", "caption": "MENU", "glyph": "☰", "tip": "program · data · preferences"},
]
## What the domain tab says for each domain (Ruling DT Q3: GENERATE while WORLD is the domain). `fan` is
## the caption in the domain fan and the sheet title. The glyphs are ones the shell already draws (`⌗` the
## old GENERATE tab, `◍` the old Civilization row, `▦` the asset library row), so none is new to the font.
const DOMAIN_TAB: Dictionary = {
	"world": {"caption": "GENERATE", "glyph": "⌗", "fan": "WORLD · GENERATE",
		"sub": "The generation pipeline, and Sculpt"},
	"civilization": {"caption": "CIVIL", "glyph": "◍", "fan": "CIVIL",
		"sub": "settlements · territory · ways · conflicts"},
	"cartography": {"caption": "CARTO", "glyph": "▦", "fan": "CARTO",
		"sub": "layers · style · annotation"},
}
## Swipe order: left goes to the next, right to the previous. The ids are `DOMAINS`' own.
const DOMAIN_ORDER: Array = ["world", "civilization", "cartography"]
## The hold that opens the domain fan. It is the mockup's `NAV_HOLD_MS` (`design/mobile-shell-2026-10-31`), a
## design figure, revisited on glass in MS-10. The tool fan's own hold is 260 ms (`ToolFan.FAN_HOLD_MS`).
const DOMAIN_HOLD_MS := 380
## Sideways travel that counts as a swipe, in phone units (scaled by `_pscale`). A labelled judgement: past
## the 12-unit tap slop the tool fan uses, short of anything a sheet drag would need.
const DOMAIN_SWIPE_UNITS := 48.0

## Draft 04, loaded by path so no new `class_name` (and no import pass) is needed.
const _PhoneDomainFan := preload("res://shell/phone_domain_fan.gd")
var _domain_fan: Control
## The domain tab's gesture: `{g0, token, fan, used}` while a press is down, `{}` otherwise. `used` means the
## press became a swipe or a fan pick, so the Button's own `pressed` (a release inside it) must be skipped.
var _dom_gesture: Dictionary = {}
var _dom_token := 0

## Builds one bar cell. The bar builder's loop calls this instead of `_phone_bar_cell()` directly:
##     for t in PHONE_TABS:
##         var cell := _build_phone_tab_cell(t)
##         var key := String(t.id)
##         _phone_tab_cells[key] = cell
##         (cells if key == "menu" else dests).add_child(cell["button"] as Control)
## MENU takes the place MORE had (`cells`, which `Window ▸ Domain rail` never hides). The domain cell is
## no longer registered in `_domain_buttons` / `_domain_marks`: one cell stands for three domains, so it is
## lit and captioned by `_refresh_phone_tabs()` instead (see the addition below).
func _build_phone_tab_cell(t: Dictionary) -> Dictionary:
	var id := String(t.id)
	if id != "domain":
		return _phone_bar_cell(String(t.caption), String(t.glyph), String(t.tip),
			_pick_phone_tab.bind(id))
	var dt: Dictionary = DOMAIN_TAB.get(_active_domain, DOMAIN_TAB["world"])
	var cell := _phone_bar_cell(String(dt["caption"]), String(dt["glyph"]), String(t.tip),
		_on_domain_cell_pressed)
	var b := cell["button"] as Button
	b.gui_input.connect(_on_domain_cell_input.bind(b))
	_add_domain_dots(cell)
	return cell

## The diameter of one domain dot and the gap between dots, in phone units. `MOBILE_SHELL_PROPOSAL.md` section
## 2.1 ("Three dots above the icon show which domain is current") and `design/mobile-shell-2026-10-31/Shell.dc.html`
## (`dots`: three 5 px circles 4 px apart, accent for the current domain, `--bord` for the others). The mockup's
## figures are kept; `_pscale()` makes a dot 13 physical px at the 6T, which a finger-sized bar needs to read it.
const DOMAIN_DOT_UNITS := 5
const DOMAIN_DOT_GAP_UNITS := 4

## Owner, 2026-11-13, on the OnePlus 6T: the domain tab "misses the indication balls". Builds one dot per
## `DOMAIN_ORDER` entry above the domain tab's glyph, in the cell's own column, so the row follows the bar wherever
## the bar goes: along the bottom in portrait, down the left rail in landscape (`_apply_phone_nav_orientation()`
## moves the cells, not their insides). The dots are `Panel`s with a round stylebox so a probe reads each one's
## colour and rect without pixels. Never a tap target: `mouse_filter` is ignore on all of them, so the cell's
## tap, hold and swipe are unchanged. Stored in the cell as `dots` and `dots_row`.
func _add_domain_dots(cell: Dictionary) -> void:
	var col := cell.get("col") as VBoxContainer
	if col == null:
		return
	var row := HBoxContainer.new()
	row.name = "DomainDots"
	row.alignment = BoxContainer.ALIGNMENT_CENTER
	row.add_theme_constant_override("separation", _pscale(DOMAIN_DOT_GAP_UNITS))
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	var dots: Array = []
	for i in DOMAIN_ORDER.size():
		var d := Panel.new()
		d.name = "Dot%d" % i
		d.custom_minimum_size = Vector2(_pscale(DOMAIN_DOT_UNITS), _pscale(DOMAIN_DOT_UNITS))
		d.mouse_filter = Control.MOUSE_FILTER_IGNORE
		row.add_child(d)
		dots.append(d)
	col.add_child(row)
	col.move_child(row, 0)
	cell["dots"] = dots
	cell["dots_row"] = row
	_paint_domain_dots(cell)

## Colours the dots for the active domain: accent on the current one, `text_ghost` on the others. A fresh
## stylebox per dot on every call (it is called from `_refresh_domain_tab_caption()`, which every tab and domain
## change and every palette flip re-runs), so no dot can keep a stale colour.
func _paint_domain_dots(cell: Dictionary) -> void:
	var dots: Array = cell.get("dots", [])
	var cur := DOMAIN_ORDER.find(_active_domain)
	for i in dots.size():
		var d := dots[i] as Panel
		if d == null or not is_instance_valid(d):
			continue
		var box := DccTheme.flat(DccTheme.c("accent") if i == cur else DccTheme.c("text_ghost"))
		box.set_corner_radius_all(_pscale(DOMAIN_DOT_UNITS))
		d.add_theme_stylebox_override("panel", box)


## Bottom-bar tab cells keyed by `PHONE_TABS` id; restyled by the tab-highlight refresh.
var _phone_tab_cells: Dictionary = {}
## Which of `PHONE_TABS` is lit. Starts on the domain tab: WORLD is the first domain, and was GENERATE.
var _phone_tab := "domain"
## The tab that was lit when MENU or STORY took the screen, so closing the overlay can give the lit state back.
## Empty when no overlay tab is lit: never a stand-in tab id (`MISTAKES.md`, a value for "none").
var _phone_tab_under_overlay := ""

## Every tab. `toggle` is false when a domain switch re-picks the domain tab: the sheet then stays open
## (or opens to half from peek) instead of folding to peek as a second tap on the same tab does.
##
## Only MAP and the domain tab drive the sheet's detent ("tab tap opens half", `docs/ANDROID_UI_SPEC.md`;
## re-tapping the lit one folds it to peek, for the reason `_on_phone_sheet_grab_input()` sets out: this sheet
## is the tool options bar and has no "gone" state). STORY and MENU open `PhoneMenu` over it, a full overlay,
## so lifting a peeking sheet behind it would only surface after the overlay closed.
func _pick_phone_tab(id: String, toggle: bool = true) -> void:
	## SM-7: Session mode refuses a domain, and a tab can still be picked by code; refuse before `_phone_tab`
	## is written, or the bar would light a tab whose domain `_select_domain()` then refuses. MAP selects
	## CARTOGRAPHY (MS-3 builds its own sheet), so it is refused with it. Departure from draft 03, which
	## guarded only the domain tab: the shipped loop guarded MAP too.
	if id == "domain" and _session_refuses(_active_domain):
		return
	if id == "map" and _session_refuses("cartography"):
		return
	## MS-11: a tab press closes an open quick drawer (it is a surface over the map; MENU or STORY would otherwise
	## draw over it and the world would stay paused under it).
	_close_quick_drawer_for_back()
	var was_tab := _phone_tab
	if (id == "menu" or id == "story") and was_tab != "menu" and was_tab != "story":
		_phone_tab_under_overlay = was_tab
	_phone_tab = id
	match id:
		"menu":
			_toggle_overflow()
		"story":
			## A second tap on the lit STORY tab closes it, as MENU's does (a departure from draft 03, which
			## only opened). `_close_all_phone_overlays()` first, as `_toggle_overflow()` does, so no dock
			## sheet is left behind it.
			var story_was_open: bool = _phone_menu != null and _phone_menu.is_open() and was_tab == "story"
			_close_all_phone_overlays()
			if _phone_menu != null and not story_was_open:
				_phone_menu.open_story()
		"map":
			if _phone_menu != null and _phone_menu.is_open():
				_phone_menu.close()
			## MS-3 builds the MAP sheet: view, fields and layers (proposal section 5). Until it lands, MAP
			## keeps the shipped behaviour of selecting CARTOGRAPHY, which the old `map` row did.
			_pick_bar_domain("cartography")
		"domain":
			if _phone_menu != null and _phone_menu.is_open():
				_phone_menu.close()
			_pick_bar_domain(_active_domain)
	if _phone_tab_drives_sheet(id):
		if toggle and was_tab == id and _phone_detent != "peek":
			_set_phone_detent("peek")
		elif _phone_detent == "peek":
			_set_phone_detent("half")
	_refresh_phone_tabs()
	_refresh_phone_sheet_header()
	## After the detent, not before: `_refresh_phone_sheet_body()` mounts the dock's scroller, whose height the
	## detent has just set, and a mount against the old height leaves it measured for the wrong box on its
	## first frame.
	_refresh_phone_sheet_body()

## `sheetTitle`/`sheetSub` (`AND:181-182`) for whichever tab is lit -- the
## canvas's own per-tab `titles` table, quoted (`AND:1466`):
## `titles={map:['MAP','layers · style · annotation'],
## gen:['GENERATE',s.genMode==='pipe'?'pipeline · seed '+s.world.seed:
## 'sculpt · draft stamps'],plan:[...,'journey · Vhal Serai → Port Amre'],
## more:mt}`, `mt` from `_moreTitle()` (`AND:1208`).
##
## Three of the four are static (`PHONE_TABS`' own `caption`/`tip`, now
## byte-for-byte the canvas's MAP and MORE-root strings -- see that constant's
## own header comment). GENERATE is live: `active_mode("world")` and
## `_find_engine_bridge()` are both existing reads this file already has for
## other reasons (the viewport context chip below, `open_journey_planner`'s
## neighbourhood) -- not a new cross-lane dependency into `world_workspace.gd`,
## which is what the 2026-09-13 header comment here used to say could not be
## done without one.
##
## **PLAN's live equivalent is wired, 2026-09-13** -- the gap this comment
## used to name (`journey_planner_view.gd` had no public accessor for the
## selected journey's endpoint names) is closed by that file's own
## `phone_header_info()`, added this pass. `journey · <from> → <to>` reads the
## committed route's actual first/last stop names, and `PLAN · STAGE n` fires
## exactly when a stage is isolated (`_isolated_stage >= 0`) -- both from the
## engine's own state, never the canvas's demo string. **Falls all the way
## through to `PHONE_TABS`' static "PLAN"/"Journey planner" below** when
## nothing is planned yet (`phone_header_info()`'s own "journey"/"journey"
## sentinel, checked here rather than printed) or when `journey_planner_view`
## is absent (`DccShell` instantiated bare, as probes do) -- so an unplanned
## PLAN tab still reads as unplanned, not as a broken live string.
##
## **MORE's per-page equivalent stays NOT wired, and now for a different
## reason than the old one.** `phone_menu.gd` no longer lacks the accessor --
## `current_page_title()`, added the same pass -- but that menu draws an
## OPAQUE full-screen page over this exact header whenever it `is_open()`
## (`current_page_title()`'s own doc comment has the node-order proof), so the
## live string would update a `Label` nothing can see. MORE keeps its root
## string regardless of navigation depth for that reason, not for a missing
## getter.
##
## Called from `_pick_phone_tab()` (a tab press) and from
## `_refresh_viewport_context()` (every signal that can change the world's
## seed, generation state or pipe/sculpt mode) -- not tab-press-only, since
## GENERATE's subtitle can go stale while GENERATE is already the lit tab (a
## dice-rolled seed, or the PIPELINE/SCULPT segment inside the sheet itself).
##
## **While `bridge.generating` is true, this never touches `world_gen`.**
## Found by actually generating a world under `_detent_probe.gd`, not argued
## from the signature: `engine_bridge.gd::_worker()`'s own doc says it
## plainly -- "the worker owns `world_gen`" -- for the Rust worker thread's
## entire run, and `_process()` polls a SEPARATE `RefCounted` (`_progress`)
## and re-emits `generation_stage` on the MAIN thread precisely so callers can
## react WITHOUT reaching into the borrowed object. A first version of this
## function did reach into it (`get_seed()`, from a `generation_stage` /
## `generation_finished` listener) and hit "Gd<T>::bind() failed, already
## bound; T = cartalith_godot::WorldGen" -- repeatably, and **no amount of
## deferring the call helped** (`call_deferred`, then a full `process_frame`
## boundary, both still panicked): the object is unsafe to touch for the
## worker's whole run, not just for one reentrant instant, so the fix is the
## same branch the retired phone column's `_pg_rebuild()` took for the same reason
## (`if bridge.generating: progress card else: idle block`), not a timing change.
func _refresh_phone_sheet_header() -> void:
	if _phone_sheet_title == null or not is_instance_valid(_phone_sheet_title):
		return
	if _phone_world_sheet_on():
		_phone_sheet_title.text = "GENERATE"
		## Mode `b` holds BOTH Sculpt and Paint, so `active_mode()` cannot tell them apart; the armed tool is
		## what says which one the user is working on. (The retired phone column kept its own `_pg_mode` for
		## this; Ruling EH item 1.)
		var bridge := _find_engine_bridge()
		if bridge != null and bridge.generating:
			## The progress home the retired column's progress card moved to, besides the dock's Pipeline
			## status rows and the map's GENERATING chip: the header is on screen at every detent and for
			## every category. `_vp_stage` is the 1-based stage the engine last reported (0 before the
			## first tick), kept by `_refresh_viewport_context()`, which calls this function.
			var stage_part := (", stage %02d" % _vp_stage) if _vp_stage > 0 else "…"
			_phone_sheet_subtitle.text = "pipeline · generating" + stage_part
		elif active_mode("world") == "b" and String(get("armed_tool")) == "paint":
			_phone_sheet_subtitle.text = "paint · draft cells"
		elif active_mode("world") == "b":
			_phone_sheet_subtitle.text = "sculpt · draft stamps"
		else:
			var seed_str := _phone_seed_text(bridge)
			_phone_sheet_subtitle.text = ("pipeline · seed " + seed_str) if seed_str != "" else "pipeline · no world yet"
		return
	## MS-2: the PLAN tab is gone and its planner is STORY ▸ Journeys, so the PLAN header (title "PLAN" /
	## "PLAN · STAGE n", subtitle "journey · A → B" or "Journey planner") belongs to the STORY tab while the
	## planner is what is on screen (`armed_tool == "journey"`: the planner is a tool and swaps the main view
	## on the phone). With the planner closed the STORY tab reads "STORY". A departure from draft 03, which
	## did not mention this branch; dropping it would have orphaned `JourneyPlannerView.phone_header_info()`
	## and the PLAN header probes (`_detent_probe`, `_stagetitle_probe`, `_plansubtitle_probe`).
	if _phone_tab == "story" and String(get("armed_tool")) == "journey":
		## `get("journey_planner_view")`, not `.journey_planner_view` --
		## that property lives on `DccApp` (`app.gd`), which EXTENDS this
		## class rather than the reverse, so `self` here has no static
		## member of that name to read. The same dynamic reach `phone_menu.gd`
		## already uses for the identical cross-file shape
		## (the deleted `_load_preset_into_planner()`'s own `_shell.get(...)`, MS-4).
		var jp = get("journey_planner_view")
		if jp != null and is_instance_valid(jp) and jp.has_method("phone_header_info"):
			var info: Dictionary = jp.phone_header_info()
			var from_name := String(info.get("from", "journey"))
			var to_name := String(info.get("to", "journey"))
			## The sentinel `phone_header_info()` documents: both halves read
			## "journey" only when there are not yet two resolved stops to
			## name. Falls through to the static PHONE_TABS entry below
			## rather than print "journey · journey → journey".
			if from_name != "journey" or to_name != "journey":
				var stage := int(info.get("stage", -1))
				_phone_sheet_title.text = ("PLAN · STAGE %d" % stage) if stage > 0 else "PLAN"
				_phone_sheet_subtitle.text = "journey · %s → %s" % [from_name, to_name]
				return
		## Else: no live planner to read, or nothing planned yet -- the planner's own static header.
		_phone_sheet_title.text = "PLAN"
		_phone_sheet_subtitle.text = "Journey planner"
		return
	## The domain tab on CIVIL or CARTO: the domain's own caption and one-line subtitle (`DOMAIN_TAB`). WORLD
	## returned above. The subtitles are labelled judgements (the Android canvas has no CIVIL or CARTO sheet).
	if _phone_tab == "domain":
		var dt: Dictionary = DOMAIN_TAB.get(_active_domain, DOMAIN_TAB["world"])
		_phone_sheet_title.text = String(dt["fan"])
		_phone_sheet_subtitle.text = String(dt["sub"])
		return
	for t in PHONE_TABS:
		if String(t.id) == _phone_tab:
			_phone_sheet_title.text = String(t.caption)
			_phone_sheet_subtitle.text = String(t.tip)
			return

## `bridge.world_gen.get_seed()`, guarded exactly as `GenInfoDialog.seed_text()`
## guards the same read (`gen_info_dialog.gd`) -- `""` when the binding is
## absent, never a fake `0`, which is a legal seed and would misreport "no
## world" as "seed zero". Takes `bridge` rather than re-finding it -- the
## CALLER already knows it needs one to check `bridge.generating` first (see
## this function's own caller), and finding it twice would invite a second
## copy of that guard drifting out of sync with this one.
func _phone_seed_text(bridge: EngineBridge) -> String:
	if bridge == null or bridge.world_gen == null or not bridge.world_gen.has_method("get_seed"):
		return ""
	## No world yet: the binding exists and `get_seed()` answers 0, which is not a
	## seed anyone chose -- "" sends the caller to "pipeline · no world yet".
	if not bridge.has_world:
		return ""
	return str(int(bridge.world_gen.get_seed()))

## Which of `PHONE_TABS` a domain lights. Every domain lives on the one domain tab, except that the MAP tab
## keeps itself lit while it is what selected CARTOGRAPHY, and a tab that opens an overlay (STORY, MENU)
## stays lit when the domain changes under it (the planner selects CIVIL from STORY ▸ Journeys; MENU ▸ Window ▸
## Workspace picks any): re-pointing the bar then would name a screen the user is not on. Departure from
## draft 03, which returned "domain" unconditionally and would have un-lit MAP on its own press
## (`_pick_phone_tab("map")` calls `_select_domain()`, which calls this) and STORY on the planner's.
func _phone_tab_for_domain(id: String) -> String:
	if _phone_tab == "map" and id == "cartography":
		return "map"
	if _phone_tab == "story" or _phone_tab == "menu":
		return _phone_tab
	return "domain"

## 2026-11-12 (`OUTSTANDING_WORK.md`, "After a MORE window closes the tab stays lit"): `PhoneMenu` closed, by back,
## by a row that opens a window (`_activate()` closes it so the window is not hidden behind it) or by another
## overlay. The MENU or STORY tab that opened it must stop reading as the screen the user is on, or it stays lit
## over the map after the window it launched is dismissed. Deferred one frame: `_pick_phone_tab("menu")` closes the
## menu in `_close_all_phone_overlays()` and reopens it in the same call, and by the next frame it is open again
## and this does nothing. The tab given back is the one lit before the overlay (MAP stays MAP only while the
## domain is still CARTOGRAPHY, as `_phone_tab_for_domain()` has it), else the domain tab.
func _on_phone_menu_closed() -> void:
	_restore_phone_tab_after_overlay.call_deferred()

func _restore_phone_tab_after_overlay() -> void:
	if _phone_menu == null or _phone_menu.is_open():
		return
	if _phone_tab != "menu" and _phone_tab != "story":
		return
	var back := _phone_tab_under_overlay
	if back == "" or back == "menu" or back == "story" or (back == "map" and _active_domain != "cartography"):
		back = "domain"
	_phone_tab = back
	_phone_tab_under_overlay = ""
	_refresh_phone_tabs()
	_refresh_phone_sheet_header()
	_refresh_phone_sheet_body()

## MAP and the domain tab drive the bottom sheet; STORY and MENU open PhoneMenu over it.
func _phone_tab_drives_sheet(id: String) -> bool:
	return id == "map" or id == "domain"

## True when the sheet is on WORLD: the domain tab is lit and the domain is WORLD (what `_phone_tab == "gen"`
## meant before the domain tab). The body is the dock's own categories since Ruling EH item 1.
func _phone_world_sheet_on() -> bool:
	return _phone_tab == "domain" and _active_domain == "world"

## Called at the end of `_refresh_phone_tabs()` (MS-2): the domain cell's caption and glyph follow `_active_domain`.
## Its lit state comes from the existing loop, because its key is "domain" and `_phone_tab` is "domain"
## whenever a domain sheet is up.
func _refresh_domain_tab_caption() -> void:
	var dcell: Dictionary = _phone_tab_cells.get("domain", {})
	if dcell.is_empty():
		return
	var dt: Dictionary = DOMAIN_TAB.get(_active_domain, DOMAIN_TAB["world"])
	var lbl := dcell.get("label") as Label
	var ic := dcell.get("icon") as Label
	if lbl != null and is_instance_valid(lbl):
		lbl.text = String(dt["caption"])
	if ic != null and is_instance_valid(ic):
		ic.text = String(dt["glyph"])
	_paint_domain_dots(dcell)

## A plain tap on the domain tab. Skipped when the same press already became a swipe or a fan pick: the
## Button emits `pressed` on a release inside it, after `gui_input` has seen that release.
func _on_domain_cell_pressed() -> void:
	var skip := bool(_dom_gesture.get("used", false))
	_dom_gesture.clear()
	if skip:
		return
	_pick_phone_tab("domain")

## The domain tab's gestures, from the same mouse and touch events the tool fan's pill reads
## (`tool_fan.gd`), converted to canvas coordinates the same way. With mouse emulation on (the project's
## default), one finger arrives twice, so a second press while a gesture is live is ignored.
func _on_domain_cell_input(event: InputEvent, b: Button) -> void:
	var pos := Vector2.ZERO
	var kind := ""
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index != MOUSE_BUTTON_LEFT:
			return
		pos = mb.position
		kind = "press" if mb.pressed else "release"
	elif event is InputEventScreenTouch:
		var st := event as InputEventScreenTouch
		pos = st.position
		kind = "press" if st.pressed else "release"
	elif event is InputEventMouseMotion:
		pos = (event as InputEventMouseMotion).position
		kind = "move"
	elif event is InputEventScreenDrag:
		pos = (event as InputEventScreenDrag).position
		kind = "move"
	else:
		return
	var g: Vector2 = b.get_global_transform_with_canvas() * pos
	match kind:
		"press":
			if not _dom_gesture.is_empty():
				return
			_dom_token += 1
			_dom_gesture = {"g0": g, "token": _dom_token, "fan": false, "used": false}
			get_tree().create_timer(DOMAIN_HOLD_MS / 1000.0).timeout.connect(
				_on_domain_hold.bind(_dom_token, b))
		"move":
			if _dom_gesture.is_empty():
				return
			if bool(_dom_gesture["fan"]):
				_domain_fan.call("pointer", g)
			elif not bool(_dom_gesture["used"]):
				var dx: float = g.x - (_dom_gesture["g0"] as Vector2).x
				if absf(dx) >= float(_pscale(DOMAIN_SWIPE_UNITS)):
					_dom_gesture["used"] = true
					_cycle_domain(1 if dx < 0.0 else -1)
		"release":
			if _dom_gesture.is_empty():
				return
			if bool(_dom_gesture["fan"]):
				_dom_gesture["used"] = true
				var picked := String(_domain_fan.call("release_at", g))
				if picked != "":
					_switch_domain_from_tab(picked)
			if bool(_dom_gesture["used"]):
				## The Button may or may not emit `pressed` (only when the release is inside it), so the
				## gesture is cleared after this frame's input either way.
				_clear_dom_gesture.call_deferred(int(_dom_gesture["token"]))
			else:
				_dom_gesture.clear()

## Clears the gesture left by a swipe or a fan pick, unless a newer press has started since.
func _clear_dom_gesture(token: int) -> void:
	if not _dom_gesture.is_empty() and int(_dom_gesture["token"]) == token:
		_dom_gesture.clear()

## The hold timer. It only acts on the press that started it, and only if that press is still down and has
## not become a swipe.
func _on_domain_hold(token: int, b: Button) -> void:
	if _dom_gesture.is_empty() or int(_dom_gesture["token"]) != token or bool(_dom_gesture["used"]):
		return
	if _domain_fan == null:
		_domain_fan = _PhoneDomainFan.new()
		_domain_fan.call("setup", self)
		_phone_root.add_child(_domain_fan)
	_dom_gesture["fan"] = true
	_haptic("sample")
	var entries: Array = []
	for id in DOMAIN_ORDER:
		var dt: Dictionary = DOMAIN_TAB[id]
		entries.append({"id": id, "caption": String(dt["fan"]), "glyph": String(dt["glyph"]),
			"refused": _session_refuses(String(id))})
	## In landscape the bar is a left rail (`_apply_phone_nav_orientation()`), so the fan opens to the right into
	## the map instead of upward over the other cells (departure from draft 03; see `phone_domain_fan.gd`).
	_domain_fan.call("open_above", b.get_global_rect(), _active_domain, entries,
		Vector2.RIGHT if _landscape else Vector2.UP)

## Swipe: the next or previous domain in `DOMAIN_ORDER`.
func _cycle_domain(step: int) -> void:
	var i := DOMAIN_ORDER.find(_active_domain)
	if i < 0:
		i = 0
	_switch_domain_from_tab(String(DOMAIN_ORDER[posmod(i + step, DOMAIN_ORDER.size())]))

## The one way the domain tab changes the domain: through `_pick_bar_domain()`, as the old GENERATE and MAP
## tabs did, then re-picking the domain tab without the second-tap fold. Session mode's refusal is honoured.
func _switch_domain_from_tab(id: String) -> void:
	if _session_refuses(id):
		return
	_haptic("tool_arm")
	_pick_bar_domain(id)
	_pick_phone_tab("domain", false)

## Called first in the phone's system-back chain, before the sheet levels: an open domain fan closes first
## (proposal §4's order: window, fan, quick drawer, cartwheel, …).
func _close_domain_fan_for_back() -> bool:
	if _domain_fan != null and bool(_domain_fan.call("is_open")):
		_domain_fan.call("close")
		return true
	return false

## The active tab wears the candidate's own pill -- `padding:5px 16px;
## border-radius:14px; background:rgba(224,163,74,.16)` behind the glyph
## (`candidates/Android Chrome B.dc.html`). Lighting only the caption, which is
## what the old bar did, left the row reading as four labels of equal weight.
func _refresh_phone_tabs() -> void:
	_refresh_domain_tab_caption()
	for key in _phone_tab_cells.keys():
		var cell: Dictionary = _phone_tab_cells[key]
		var on: bool = (String(key) == _phone_tab)
		var lbl: Label = cell.get("label")
		var pill: PanelContainer = cell.get("pill")
		if lbl != null and is_instance_valid(lbl):
			(lbl as Label).add_theme_color_override("font_color",
				DccTheme.c("accent" if on else "text_dim"))
		## The glyph's ink moved here with the domain cells' registration in `_domain_marks` (MS-2): the
		## old `_select_domain()` loop tinted it only for cells registered there.
		var gly := cell.get("icon") as CanvasItem
		if gly != null and is_instance_valid(gly):
			gly.modulate = DccTheme.c("accent" if on else "text_dim")
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
	## panel.
	##
	## **A mono `Label`, not `DccIcons.rect()` -- 2026-09-13.** `glyph` is now
	## the canvas's own literal character (`▤`/`⌗`/`➔`/`⋯`, `PHONE_TABS`'s own
	## header), and "glyph text in the mono font is the canvas's own mechanism"
	## -- there is no SVG to rasterise. Font drawn in plain white and left
	## uncoloured here, matching `DccIcons.rect()`'s own "one asset, two states
	## via `modulate`" contract exactly (its header comment, and `_select_domain()`'s
	## `(marks["icon"] as CanvasItem).modulate = ...`, which reads generically
	## off `CanvasItem` and needs no change for this to keep working on a
	## `Label` in place of a `TextureRect`) -- setting `font_color` to the
	## initial token directly, the way `DccTheme.mono_label()` would, then ALSO
	## letting `modulate` multiply over it would double-apply the tint.
	var ic := Label.new()
	ic.text = glyph
	ic.add_theme_font_override("font", DccTheme.mono())
	ic.add_theme_font_size_override("font_size", _pscale(14))
	ic.add_theme_color_override("font_color", Color(1, 1, 1))
	ic.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	ic.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	ic.custom_minimum_size = Vector2(_pscale(14), _pscale(14))
	ic.modulate = DccTheme.c("text_dim")
	ic.mouse_filter = Control.MOUSE_FILTER_IGNORE
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
	return {"button": b, "label": l, "icon": ic, "pill": pill, "col": col}

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
	## Kept as a local, not just handed to the override -- the header-block
	## floor further down needs this exact instance's own OWN top/bottom
	## content margin, not a second `_phone_sheet_box(false)` call (harmless
	## today since the function is pure, but this way there is only ever one
	## stylebox in play).
	var sheet_style := _phone_sheet_box(false)
	sheet.add_theme_stylebox_override("panel", sheet_style)

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	sheet.add_child(col)
	_phone_sheet_col = col

	## The grab handle -- as of the detents, the drag target for them, and as
	## of 2026-09-13 the WHOLE header block (pill, title/subtitle, close),
	## not the pill alone -- `_detent_probe.gd`'s own SS5.3 comment already
	## called it that before this pass built it: "grabs the whole header
	## block". `VBoxContainer`, not a bare `Control`, so the pill row and the
	## title row below it stack without manual positioning -- `gui_input` and
	## `mouse_filter` both work identically on a `Container`, so nothing about
	## the drag wiring below changes shape, only what it wraps.
	## `MOUSE_FILTER_STOP` (set explicitly because it is now load-bearing
	## rather than incidental) so the row picks the press; everything in it
	## that is *not* meant to pick is `IGNORE`, same convention
	## `_phone_content_gap` documents at length -- except the close circle,
	## which is `STOP` on purpose so its own tap does not also register as a
	## zero-distance drag.
	_phone_sheet_grab = VBoxContainer.new()
	_phone_sheet_grab.add_theme_constant_override("separation", 0)
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
	## **The row used to stay 24 dp deliberately -- built into a real header
	## block instead, 2026-09-13.** §5.3's grab region was never the pill's row
	## alone -- it is *"the whole header block above the scroller"*, pill row
	## plus a title row (`padding:0 14px 10px` around 38 x 38 back/close
	## buttons) in the prototype, `AND:176-186` here. **"This sheet has no
	## header block" was true when that sentence was written and is corrected
	## by this pass, not merely restated**: `sheetTitle`/`sheetSub`/
	## `hSheetClose` (`AND:181/182/184`) are built below, closing the symptom
	## `OUTSTANDING_WORK.md` filed -- "half a body row shows at peek,
	## PIPELINE/SCULPT chips cut at 52 of 115 px". The canvas's own peek budget
	## is spent on `handle + header`, not on a sliver of the first scrollable
	## row; this sheet had never built the header half of that budget.
	##
	## **Sized to fit `peek` deliberately, not organically -- the refuted
	## attempt's own lesson.** A prior pass pinned a comparably-sized
	## (`_ptap`-floored, ~44 dp) control above this same scroll clip and the
	## sheet grew to 98 dp at what the code still called "peek" against a 66 dp
	## formula (`.claude/resume-2026-09-12/peek_pin_2026-09-12.patch`, reverted):
	## no `phone_insets_changed` emitted, and at cold boot it covered 29 px of
	## the bottom nav and hid the coordinate/scale labels. The mechanism is
	## `Control.get_combined_minimum_size()`: a non-expanding child is sized to
	## `max(custom_minimum_size, its own children's genuine minimum)`, so
	## stacking a second full 44 dp accessibility-floored row ON TOP of the
	## pill's old 44 dp (`_ptap(20)`) pushes `col`'s own reported minimum to
	## ~88 dp regardless of what `peek` says, and `_phone_chrome_col`
	## (`_phone_tool_sheet`'s parent) has no way to tell that apart from a
	## deliberately taller `half`/`full`.
	##
	## Avoided here two ways, not one:
	## - **The pill's own row shrinks from `_ptap(20)` to a plain `_pscale(14)`,
	##   below.** It no longer needs its own 44 dp accessibility floor -- that
	##   requirement now belongs to the close circle in the title row, and the
	##   WHOLE header (pill row plus title row) is one `gui_input` target
	##   regardless of which sub-row is pressed, so the pill sub-row's own
	##   height is purely visual.
	## - **`_phone_sheet_grab.custom_minimum_size.y` is set to
	##   `_phone_detent_height("peek")` directly, not a literal 66,
	##   immediately below.** The pill row and the title row (`_ptap(38)` plus
	##   a few px of padding) are kept modest enough to fit inside that figure
	##   with room to spare, so this explicit floor -- not the organic sum --
	##   is what `max()` picks: `col`'s reported minimum stays exactly `peek`,
	##   at every detent, the same invariant `peek < half < full` already
	##   guarantees holds without a special case. If a future change pushed the
	##   two rows' organic sum past `peek`, `max()` would pick THAT instead and
	##   the sheet would grow -- exactly the failure mode above -- so keep the
	##   rows inside this figure rather than widen it.
	##
	## Measured against this reasoning, not only argued from it -- see this
	## batch's report for the before/after numbers windowed at 1080x2340, and
	## `_detent_probe.gd`'s own SS5.2/5.3 blocks for the standing assertions.
	##
	## **`- sheet_vpad`, added 2026-09-13: the reasoning above stops one layer
	## too early.** It keeps `col`'s own reported minimum at exactly `peek` --
	## true, and not enough, because `col` is not what gets measured. `sheet`
	## (the `PanelContainer` `col` sits inside) has its OWN panel stylebox
	## (`_phone_sheet_box()`: a 1 physical-px top border, no explicit content
	## margin, so `StyleBoxFlat` reports that border width back as the margin),
	## and a `PanelContainer`'s combined minimum is its child's minimum PLUS
	## that stylebox's margins on the relevant axis -- invisible at HEAD, where
	## the bare handle's minimum (`_ptap(20)`, ~115 px @1080x2340) sat ~57 px
	## under `peek` and the outer sheet's own detent target won outright with
	## room to spare either way. Setting the header's floor to the FULL `peek`
	## figure (just above, pre-fix) closed that slack to exactly zero, which is
	## exactly where the stylebox's 1 px stopped being free: measured (this
	## batch, `_detent_probe.gd`'s cold-boot header/sheet log, 1080x2340)
	## header=173 sheet=174 -- the "grew 1 px" a prior pass called
	## byte-identical without a HEAD comparison behind it. Subtracting the
	## stylebox's own margin here restores `sheet.size.y == 173` exactly, at
	## the layer HEAD's own 173 was always measured at, rather than shrinking
	## anything the user can see.
	var sheet_vpad := sheet_style.get_margin(SIDE_TOP) + sheet_style.get_margin(SIDE_BOTTOM)
	_phone_sheet_grab.custom_minimum_size.y = _phone_detent_height("peek") - sheet_vpad
	_phone_sheet_grab.mouse_filter = Control.MOUSE_FILTER_STOP
	_phone_sheet_grab.gui_input.connect(_on_phone_sheet_grab_input)

	var pill_row := Control.new()
	## `_pscale`, not `_ptap` -- see the header comment above for why the pill
	## row no longer carries its own accessibility floor.
	pill_row.custom_minimum_size.y = _pscale(14)
	pill_row.mouse_filter = Control.MOUSE_FILTER_IGNORE
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
	pill_row.add_child(handle)
	_phone_sheet_grab.add_child(pill_row)

	## The title row -- `AND:178-186`: `sheetTitle`/`sheetSub` stacked at
	## `flex:1`, then a 38 x 38 close circle, in a `padding:0 14px 10px` row.
	## `_ptap(38)`, not the canvas's bare 38, per every other tappable thing in
	## this shell -- see the big header comment above for why THIS row, not
	## the pill row, is where the 44 dp accessibility floor now lives.
	var title_row := HBoxContainer.new()
	title_row.add_theme_constant_override("separation", _pscale(10))
	title_row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var title_pad := MarginContainer.new()
	title_pad.add_theme_constant_override("margin_left", _pscale(14))
	title_pad.add_theme_constant_override("margin_right", _pscale(14))
	## Trimmed from the canvas's own 10 px bottom padding -- part of the modest
	## budget the big header comment above describes, so `pill_row` + this row
	## together still fit inside `peek` rather than widen it.
	title_pad.add_theme_constant_override("margin_bottom", _pscale(4))
	title_pad.mouse_filter = Control.MOUSE_FILTER_IGNORE
	title_pad.add_child(title_row)

	var title_stack := VBoxContainer.new()
	title_stack.add_theme_constant_override("separation", 0)
	title_stack.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## Centred on the row's centre line, which the 44 dp close button sets. Left at the HBox default
	## (top-aligned) the title and subtitle block sat 7 px above the button's centre
	## (`_alignment_probe.gd`, 2026-10-26); the row's height is the button's, so nothing grows.
	title_stack.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	title_stack.mouse_filter = Control.MOUSE_FILTER_IGNORE
	## No `clip_text` -- `MISTAKES.md`'s own row: it collapses
	## `get_minimum_size().x` to 1, and beside `close_btn` (a fixed-size
	## sibling, not `SIZE_EXPAND_FILL`, so this specific pairing would not hit
	## the "vanishes entirely" case) it is still simplest not to depend on
	## that distinction holding forever. `PHONE_TABS`' own captions are short.
	## `_pfont(11)`/`_pfont(9.5)`, not the raw `DccTheme.FS_SMALL`/`FS_MICRO` --
	## every OTHER phone type size in this file goes through `_pfont()` (see
	## its own doc comment), and this pair did not: measured drawing 16/13 px
	## at this phone's scale (2.62) where the canvas's `font:500 11px`/
	## `font:9.5px` (`AND:181-182`) means about 29/25 px. `9.5`, a literal, not
	## `FS_MICRO` (9) -- the canvas's own subtitle size is 9.5 and `_pfont()`
	## takes a float, so there is no reason to round to the nearest shared
	## constant a half-pixel category away.
	##
	## **The title's tracking gets the same treatment, 2026-09-13 -- it did not
	## have it.** `DccTheme.mono_label(..., 2, true)` passed `mono()`'s
	## `spacing` a bare `2` *physical* px against the canvas's own
	## `letter-spacing:.2em` (`AND:181`), which at an 11 px font is 2.2 *design*
	## px and, at this phone's 2.62 scale, ≈5.8 physical -- roughly a third of
	## what shipped. Every other phone caller of a scaled quantity in this file
	## pre-scales before handing it to a `DccTheme` factory (`_pfont(11)` for
	## the size right beside this); this call alone did not for its `spacing`
	## argument. Computed as a fraction of the ALREADY-scaled title size below,
	## not as a second independent `_pfont(2.2)` call, so the ratio to the
	## rendered glyph stays exactly `.2em` however `_pfont()`'s own formula
	## changes later -- including its dynamic-type term: tracking is a font
	## metric and should grow with the user's text-size preference the same
	## way the glyphs it separates do.
	var title_fs := _pfont(11)
	_phone_sheet_title = DccTheme.mono_label("", "accent", title_fs,
		maxi(1, roundi(title_fs * 0.2)), true)
	_phone_sheet_title.mouse_filter = Control.MOUSE_FILTER_IGNORE
	title_stack.add_child(_phone_sheet_title)
	_phone_sheet_subtitle = DccTheme.mono_label("", "text_dim", _pfont(9.5))
	_phone_sheet_subtitle.mouse_filter = Control.MOUSE_FILTER_IGNORE
	title_stack.add_child(_phone_sheet_subtitle)
	title_row.add_child(title_stack)

	## `hSheetClose` -- collapses the sheet to `peek` rather than the canvas's
	## `tab:null`: this app's phone sheet has no "gone" state on top of the
	## three detents (`_pick_phone_tab()`'s own comment says so directly), so
	## `peek` is the nearest real equivalent to "dismissed".
	##
	## **Re-verified rather than re-argued, 2026-09-13**: `_pick_phone_tab()`'s
	## own `match` only ever sets `_phone_tab` to one of `PHONE_TABS`' four ids
	## or leaves it where it was, `_refresh_phone_tabs()` has no branch for an
	## unmatched one beyond drawing every cell "off", and nothing in this file,
	## `phone_menu.gd` or `journey_planner_view.gd` treats an empty `_phone_tab`
	## as "the sheet is gone" -- it would just be every nav cell unlit over
	## whatever domain content was already showing. There is still no state to
	## match `tab:null` onto; `peek` stands.
	var close_btn := Button.new()
	close_btn.text = "✕"
	close_btn.focus_mode = Control.FOCUS_NONE
	## The HIT area stays the 44 dp accessibility floor, unchanged -- only the
	## ink drawn inside it shrinks, below.
	close_btn.custom_minimum_size = Vector2(_ptap(38), _ptap(38))
	close_btn.mouse_filter = Control.MOUSE_FILTER_STOP  ## Catches its own tap
		## before it can reach `_phone_sheet_grab`'s drag handler above.
	var close_sb := StyleBoxFlat.new()
	## `line_soft` stands in for the canvas's own `--chip` (`rgba(#,.05)`
	## dark / light) -- close in weight, and an existing token rather than a
	## new palette entry for one circle (`MISTAKES.md`: re-basing a shared
	## token set is its own, larger pass). **Re-checked, not re-guessed,
	## 2026-09-13**: `dcc_theme.gd`'s own canvas-token table (its header
	## comment, off `ENV:25`) carries no `--chip`/`--chipOn` row at all --
	## those are `AND`-only properties `ENV` never declares, unlike `warn`,
	## which the same table imports from `AND` for exactly that reason. There
	## is no dedicated token to switch to; `line_soft` is still the nearest one.
	close_sb.bg_color = DccTheme.c("line_soft")
	## **The ink drawn is now the canvas's own 38 dp circle, not the hit box's
	## 44 dp floor, 2026-09-13** -- `AND:184`: `width:38px;height:38px;
	## border-radius:19px`. A `StyleBoxFlat` cannot be smaller than the
	## `Control` drawing it, so `close_btn` stays `_ptap(38)` (the
	## accessibility floor always wins there, ~115 px at this phone's scale)
	## and the VISUAL box is pulled in from that with a NEGATIVE
	## `expand_margin` -- the same property `dcc_widgets.gd`'s hover box uses
	## *positively* to draw a background bigger than its control; here it
	## draws one smaller, centred (all four margins equal), while
	## `close_btn`'s own rect -- what `gui_input` actually hit-tests -- never
	## moves.
	var close_hit := float(_ptap(38))
	var close_visual := float(_pscale(38))
	var close_inset := (close_hit - close_visual) * 0.5
	close_sb.expand_margin_left = -close_inset
	close_sb.expand_margin_top = -close_inset
	close_sb.expand_margin_right = -close_inset
	close_sb.expand_margin_bottom = -close_inset
	var close_r := _pscale(19)
	close_sb.corner_radius_top_left = close_r
	close_sb.corner_radius_top_right = close_r
	close_sb.corner_radius_bottom_left = close_r
	close_sb.corner_radius_bottom_right = close_r
	for state in ["normal", "hover", "pressed", "focus"]:
		close_btn.add_theme_stylebox_override(state, close_sb)
	close_btn.add_theme_color_override("font_color", DccTheme.c("text_secondary"))
	## `_pfont(13)`, matching the canvas's `font:13px` for `hSheetClose`
	## (`AND:184`) -- the same unscaled-constant defect as the title/subtitle
	## above, not a separate one.
	close_btn.add_theme_font_size_override("font_size", _pfont(13))
	close_btn.pressed.connect(func(): set_phone_detent("peek"))
	title_row.add_child(close_btn)

	_phone_sheet_grab.add_child(title_pad)
	col.add_child(_phone_sheet_grab)
	_refresh_phone_sheet_header()

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

	## **There is no GENERATE column here any more (Ruling EH item 1).** It was a second, phone-only parameter
	## sheet (`WorldWorkspace.build_phone_generate()`) built after the owner's 2026-09-07 report that the
	## sheet held no generation parameters (*"I don't have sliders or fields to change parameters on how the
	## world is generated"*). MS-3 then mounted the dock's own categories in this sheet for CIVIL and CARTO
	## and added an ALL segment for WORLD; Ruling EH retired the column, so WORLD's sheet body is the dock too
	## (`_phone_dom_host`, below) and the generation parameters have exactly one builder, the workspace's
	## `_build_categories()`. The 2026-09-07 defect stays closed by that mount, and `_eh1_probe` taps it.
	## MS-3: the domain sheet's host (see `_phone_dom_host`). Empty and hidden until a CIVIL or CARTO domain
	## sheet is lit; `separation 0` and a full-height expand so the dock's own scroller fills the sheet.
	_phone_dom_host = VBoxContainer.new()
	_phone_dom_host.add_theme_constant_override("separation", 0)
	_phone_dom_host.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_phone_dom_host.visible = false
	col.add_child(_phone_dom_host)
	return sheet

## Chooses the phone tool sheet's body, and (re)mounts it: the dock's own categories (WORLD, CIVIL, CARTO on
## the domain tab) or the tool-options row (the MAP tab, and anywhere the dock cannot be mounted).
##
## Called from `_pick_phone_tab()` and from `_select_domain()`, which are the two places the answer can change
## -- a domain picked from MORE > Workspace or from an in-shell jump lights the domain tab through
## `_phone_tab_for_domain()` without ever passing through a tab press.
##
## Ruling EH item 1: WORLD no longer has a body of its own. Before it, WORLD's body was the phone GENERATE
## column (`WorldWorkspace.build_phone_generate()`, a second parameter builder) and the dock showed only
## behind an ALL segment; now WORLD mounts the dock exactly as CIVIL and CARTO do.
##
## WORLD additionally waits for its workspace panel (`register_workspace()`), because `DccShell` is
## instantiated bare by probes with no workspaces registered at all, and the boot-order trap
## (`register_workspace()`'s own comment) means the first call, from `_select_domain()` during the shell's
## build, finds none and the deferred one finds it.
func _refresh_phone_sheet_body() -> void:
	var ws: Node = _workspace_panels.get("world")
	var dom: bool = _phone and _phone_tab == "domain" and not _left_sheet_open and _phone_dom_host != null and left_dock_body != null and _left_dock_scroll != null
	if dom and _active_domain == "world" and ws == null:
		dom = false
	_mount_dock_in_sheet(dom)
	if _dock_in_sheet and dom:
		_sync_dom_rows(true)
	_refresh_phone_map_row()
	if _phone_tool_scroll != null and is_instance_valid(_phone_tool_scroll):
		## On the MAP tab with the idle Inspect tool the tool-options row holds only a caption, and as an expanding
		## sibling it took the room the MAP body needs; it comes back when a drawing tool is armed (`armed_tool`).
		var map_body_on: bool = _phone and _phone_tab == "map" and _phone_detent != "peek" and String(get("armed_tool")) == "inspect"
		_phone_tool_scroll.visible = not dom and not map_body_on
	## The phone seed readout in the Run section shows the dialog's pending seed, which a roll anywhere (the
	## tool bar, File > New world) can change while the dock is out of sight; refresh it when the sheet shows.
	if dom and ws != null and ws.has_method("refresh_phone_seed_readout"):
		ws.call("refresh_phone_seed_readout")

## MS-3: moves the dock's mode switch and category scroller into the phone tool sheet (`on`) or back into the
## left dock's own column (`not on`). Reparenting, not rebuilding: `register_workspace()` attached every
## workspace panel to `left_dock_body` once, and `apply_mode()` / `_refresh_mode_switch()` keep working on the
## same nodes wherever they sit. Order is kept (mode switch, then scroller) because `reparent()` appends.
## Must never run with the left sheet showing: `_set_sheet_open("left", true)` calls this with `false`
## first, or Window > Left dock would open an empty sheet.
func _mount_dock_in_sheet(on: bool) -> void:
	if on == _dock_in_sheet or _phone_dom_host == null or _left_dock_col == null or _left_dock_scroll == null or _left_dock_mode_switch == null:
		return
	_dock_in_sheet = on
	var target: Control = _phone_dom_host if on else _left_dock_col
	_left_dock_mode_switch.reparent(target, false)
	_left_dock_scroll.reparent(target, false)
	_sync_dom_host_visible()
	_sync_dom_rows(on)

## Shows the domain host while the dock is mounted in it, except at the `peek` detent in portrait. The sheet's
## minimum height is the sum of its children's, and the dock's mode switch alone is a 44 dp row, so a visible
## host lifts the peek detent above its 66 dp (`_ms5b_probe`: the strip sat 131 px higher and the map's control
## row overlapped it, found when WORLD began mounting the dock at boot, Ruling EH item 1; `_refresh_phone_map_row()`
## hides its row at peek for the same reason). Peek shows the header only, as it always did.
func _sync_dom_host_visible() -> void:
	if _phone_dom_host == null:
		return
	_phone_dom_host.visible = _dock_in_sheet and (_landscape or _phone_detent != "peek")

## Builds (once) and shows or hides the door row above the mounted dock for the CURRENT domain. Split out of
## `_mount_dock_in_sheet()` by Ruling EH item 1: WORLD now mounts the dock too, so a domain change from WORLD to
## CIVIL happens with the dock already mounted, `_mount_dock_in_sheet()` returns early on `on == _dock_in_sheet`,
## and the Journeys row was never built (found by `_ms3b_probe`). Called by `_refresh_phone_sheet_body()` on every
## refresh, mounted or not.
func _sync_dom_rows(on: bool) -> void:
	if on:
		_ensure_dom_rows()
	if _phone_dom_journeys_row != null:
		_phone_dom_journeys_row.visible = on and _active_domain == "civilization"

## MS-3: builds, once, the row that sits above the dock's pieces inside `_phone_dom_host` and moves it to the
## front (`reparent()` appended the dock's pieces first). A plain `DccWidgets.action()` button, so
## `phone_fit()` (through `_on_phone_node_added()`'s `_phone_dom_host` clause) sizes it to the 44 dp floor like
## every dock control. It is a door, not a copy: CIVIL's journeys row opens STORY > Journeys
## (`PhoneMenu.open_story("JOURNEYS")`), the proposal's "CIVIL > Journeys opens STORY > Journeys rather than a
## second planner". WORLD's "Back to Pipeline - Sculpt - Paint" row is gone with the columns it returned to
## (Ruling EH item 1).
func _ensure_dom_rows() -> void:
	if _phone_dom_journeys_row == null:
		_phone_dom_journeys_row = DccWidgets.action(_phone_dom_host, "Journeys ▸ planner, saved journeys (STORY)", func():
			_pick_phone_tab("story")
			if _phone_menu != null:
				_phone_menu.open_story("JOURNEYS"))
		_phone_dom_journeys_row.name = "DomJourneysRow"
		_phone_dom_host.move_child(_phone_dom_journeys_row, 0)

## MS-3 (rewritten 2026-11-13, owner on the OnePlus 6T: *"The drawer for map on mobile is empty"*): the MAP sheet's
## own body, `MOBILE_SHELL_PROPOSAL.md` section 2.1's MAP row: the view (Fit world, 2D / 3D), the view field with
## its opacity, the six Measure modes, and quick visibility for every layer `CartographyWorkspace.LIVE_LAYERS`
## names (settlements, ways, landmarks, rivers, the two political rows and the rest), then a door to the full
## Layers popover (the 43 analysis fields with their legend, digit hotkeys and the landmark filter, which the
## popover keeps as its own builder: it is a `Popup` whose rows are fitted at a window scale of 1.0, so mounting
## it in this sheet, fitted at the phone unit, would double-scale it the first time it had been opened).
##
## **Why it was empty.** The first MS-3 step put ONE 44 dp row here (three buttons that opened the popover or
## reset the view) above the tool-options row, and the tool-options row is empty on the MAP tab until a tool is
## armed (`_refresh_phone_sheet_body()` hides it only for a domain sheet). So the half-height sheet showed a title
## and a clipped row of three buttons: the design's view, fields and layers were three taps away, not in it.
##
## Every control is an existing factory (`DccWidgets.action()` / `choice()` / `slider()` / `toggle()`) writing
## through the same host calls the popover and the CARTO dock use (`ViewportHost.set_layer_visible()`,
## `set_debug_layer()`, `set_debug_opacity()`, `GlobalTools.set_measure_mode()`), so there is one state and the
## two surfaces cannot drift: `layer_visibility_changed` re-seeds the switches here when the dock or the popover
## flips one. Hidden at the peek detent (a visible body would lift the peek above its 66 dp, `_quickbar_phone_probe`)
## and off the MAP tab. Rebuilt each time the sheet shows and when a world loads, so the field list is the world's.
## `_phone_map_row` stays the first row (the view buttons) because `_ms3b_probe` and the 44 dp checks read it.
func _refresh_phone_map_row() -> void:
	var show: bool = _phone and _phone_tab == "map" and _phone_detent != "peek"
	var holder: Node = _phone_dom_host.get_parent() if _phone_dom_host != null else null
	if show and _phone_map_body == null and holder != null:
		_phone_map_body = ScrollContainer.new()
		_phone_map_body.name = "MapBody"
		_phone_map_body.visible = false   ## the first `show` below rebuilds it (only a hidden to shown change does)
		_phone_map_body.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
		_phone_map_body.size_flags_vertical = Control.SIZE_EXPAND_FILL
		_phone_map_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		holder.add_child(_phone_map_body)
		## Before the tool-options scroller, not after it: that scroller is an expanding sibling and a body placed
		## behind it was squeezed to a few rows (first probe screenshot, 2026-11-13).
		holder.move_child(_phone_map_body, _phone_tool_scroll.get_index() if _phone_tool_scroll != null and _phone_tool_scroll.get_parent() == holder else _phone_dom_host.get_index())
		var br := _find_engine_bridge()
		if br != null:
			br.generation_finished.connect(func(_ok: bool): _rebuild_phone_map_body())
			br.world_loaded.connect(func(): _rebuild_phone_map_body())
		var vp0 = get("viewport")
		if vp0 != null and vp0.has_signal("layer_visibility_changed"):
			vp0.layer_visibility_changed.connect(_sync_phone_map_toggles)
	if _phone_map_body != null:
		if show and not _phone_map_body.visible:
			_phone_map_body.visible = true
			_rebuild_phone_map_body()
		elif not show:
			_phone_map_body.visible = false
	if _phone_map_row != null:
		_phone_map_row.visible = show

## Empties and refills the MAP body from the host and the engine (see `_refresh_phone_map_row()`). Safe while the
## body is hidden. The view field is a dropdown of every available view in `debug_layers()` order, "Base map" first,
## each labelled `group · view` because a dropdown has no group headings; picking one calls the popover's own
## `set_debug_layer()`.
func _rebuild_phone_map_body() -> void:
	if _phone_map_body == null or not is_instance_valid(_phone_map_body) or not _phone_map_body.visible:
		return
	for c in _phone_map_body.get_children():
		_phone_map_body.remove_child(c)
		c.queue_free()
	_phone_map_toggles.clear()
	_phone_height_box = null
	_phone_height = {}
	_phone_map_row = null
	var vp = get("viewport")
	var br := _find_engine_bridge()
	if vp == null or br == null:
		return
	var body := VBoxContainer.new()
	body.name = "MapBodyColumn"
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_theme_constant_override("separation", _pscale(6))
	_phone_map_body.add_child(body)

	var row := HBoxContainer.new()
	row.name = "MapRow"
	row.add_theme_constant_override("separation", _pscale(8))
	body.add_child(row)
	var b1 := DccWidgets.action(row, "All layers", func():
		var lp = get("layers_popover")
		if lp != null:
			lp.open())
	b1.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var b2 := DccWidgets.action(row, "Fit world", func(): vp.reset_view())
	b2.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var b3 := DccWidgets.action(row, "2D / 3D", func(): vp.set_3d(not vp.is_3d()))
	b3.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_phone_map_row = row

	## The view field and its opacity.
	var fsec := DccWidgets.section(body, "View field")
	var ids: Array = ["off"]
	var labels: Array = ["Base map"]
	for g in br.debug_layers():
		for it in (g as Dictionary)["items"]:
			var item: Dictionary = it
			if bool(item["available"]):
				ids.append(String(item["id"]))
				labels.append("%s · %s" % [String((g as Dictionary)["group"]), String(item["label"])])
	var cur := ids.find(String(vp.debug_view()))
	DccWidgets.choice(fsec, "Showing", labels, maxi(cur, 0), func(i: int): vp.set_debug_layer(String(ids[i])))
	DccWidgets.slider(fsec, "Opacity", 0.0, 100.0, 1.0, float(vp.debug_opacity()) * 100.0, "%",
		func(v: float): vp.set_debug_opacity(v / 100.0))

	## The six Measure modes, from `GlobalTools.MEASURE_MODES` (one list, so a seventh mode appears here too).
	var msec := DccWidgets.section(body, "Measure")
	var grid := GridContainer.new()
	grid.columns = 3
	grid.add_theme_constant_override("h_separation", _pscale(8))
	grid.add_theme_constant_override("v_separation", _pscale(8))
	msec.add_child(grid)
	for m in GlobalTools.MEASURE_MODES:
		var mid := String((m as Dictionary)["id"])
		var mb := DccWidgets.action(grid, String((m as Dictionary)["label"]), func():
			GlobalTools.set_measure_mode(self, mid)
			call("arm_tool", "measure"))   ## `arm_tool()` is `app.gd`'s (this class is its base).
		mb.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	## Quick visibility: the Visible layers band's own rows, the same switch as CARTO > Layers.
	var lsec := DccWidgets.section(body, "Visible layers")
	for l in CartographyWorkspace.LIVE_LAYERS:
		var lid := String((l as Dictionary)["id"])
		var write := (func(on: bool): vp.set_landmarks_shown_by_user(on)) if lid == "landmarks" \
			else (func(on: bool): vp.set_layer_visible(lid, on))
		_phone_map_toggles[lid] = DccWidgets.toggle(lsec, String((l as Dictionary)["label"]),
			vp.layer_visible(lid), write)
	## The proposal's North up / grid / scale bar toggles (`MOBILE_SHELL_PROPOSAL.md` §2, MAP), read at the symbols
	## 2026-11-13: the desktop has NO North-up control (the map cannot rotate yet, so it is always north up), NO scale-bar
	## toggle (`_update_scale_bar()` always draws it), and its only grid is Preferences > Tiles & LOD > Chunk debug
	## overlay > Grid, i.e. `set_lod_debug("grid")` (`menus.gd::_on_lod_debug()`). So only that one is built here, through
	## the same host call and the same state, labelled as the chunk grid it is; the other two have no host call to write.
	if vp.has_method("set_lod_debug"):
		var vsec := DccWidgets.section(body, "Map view")
		_phone_map_toggles["lod_grid"] = DccWidgets.toggle(vsec, "Chunk grid (tiled view)",
			vp.lod_debug_enabled("grid"), func(on: bool): vp.set_lod_debug("grid", on))
	## 3D-2's HEIGHT row for the phone (`View3D._build_height_row()` says its home here is MAP ▸ View): the same
	## 1x..4x range and 0.25 step, written through the SAME call, `View3D.set_height()`, so the PC/tablet card row,
	## the 3D mesh and `Preferences ▸ 3D viewport defaults` share one state. Shown only while 3D shows (a height for a
	## view that is not on screen is a control that does nothing visible); re-seeded from the view on every entry.
	## Never the world or its save. `phone_fit()` below gives the track its touch floor (`touch_slider()`).
	_phone_height_box = DccWidgets.section(body, "3D height · 1× is true scale")
	var cur_h: float = DccSettings.view3d_exaggeration(View3DScript.VERTICAL_EXAGGERATION_DEFAULT)
	if vp.view3d() != null and vp.view3d().has_method("current_height"):
		cur_h = vp.view3d().current_height()
	_phone_height = DccWidgets.slider(_phone_height_box, "Height", View3DScript.HEIGHT_MIN, View3DScript.HEIGHT_MAX,
		View3DScript.HEIGHT_STEP, cur_h, "×", func(v: float): vp.ensure_view3d().set_height(v))
	_phone_height_box.visible = vp.is_3d()
	## 44 dp of REAL density, as the task asks: `phone_fit()`'s floor is 44 phone units (115 px at the 6T's scale,
	## 40.9 dp at density 2.8125), so the track is raised to the real-dp floor when the density is known.
	var dv: Dictionary = _density()
	if bool(dv["known"]):
		(_phone_height["slider"] as HSlider).set_meta("dp_floor_px", 44.0 * float(dv["value"]))
	if vp.has_signal("view_mode_changed") and not vp.view_mode_changed.is_connected(_sync_phone_height):
		vp.view_mode_changed.connect(_sync_phone_height)
	phone_fit(_phone_map_body, _phone_scale)
	if _phone_height.has("slider") and (_phone_height["slider"] as HSlider).has_meta("dp_floor_px"):
		var hs := _phone_height["slider"] as HSlider
		hs.custom_minimum_size.y = maxf(hs.custom_minimum_size.y, float(hs.get_meta("dp_floor_px")))

## Shows the phone HEIGHT section while 3D does and re-seeds its slider from the view's own value
## (`set_value_no_signal()`, so the seed does not echo back into `set_height()`).
func _sync_phone_height(is_3d: bool) -> void:
	if _phone_height_box == null or not is_instance_valid(_phone_height_box):
		return
	_phone_height_box.visible = is_3d
	var vp = get("viewport")
	## Re-seeds whether or not 3D is showing: Preferences ▸ 3D height can move the value while 2D is up, and the slider
	## must read it the next time the section shows.
	if vp != null and vp.view3d() != null and not _phone_height.is_empty():
		var s := _phone_height["slider"] as HSlider
		s.set_value_no_signal(vp.view3d().current_height())
		(_phone_height["readout"] as Label).text = (_phone_height["format"] as Callable).call(s.value)

## Re-seeds the MAP body's switches after a layer flips anywhere else (`ViewportHost.layer_visibility_changed`).
## `set_pressed_no_signal()`, so the write does not echo back into `set_layer_visible()`.
func _sync_phone_map_toggles(layer: String, shown: bool) -> void:
	var cb = _phone_map_toggles.get(layer)
	if cb != null and is_instance_valid(cb):
		(cb as CheckBox).set_pressed_no_signal(shown)

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

## MS-11: mounts the phone's quick drawer (`quick_drawer.gd`, built by `QuickBarPhone.refresh()`) over the map: above
## the map surface and its floating controls, below the tool sheet, the bar and every sheet added after the map, so
## the drawer covers the map's controls and never the sheet's grab handle or the bottom bar. A null drawer or a
## non-phone shell does nothing; the caller frees the drawer itself. Replaced `set_phone_quick_row` (CQ-6).
func set_phone_quick_drawer(d: Control) -> void:
	if d == null or not _phone or _phone_root == null:
		return
	if d.get_parent() == null:
		_phone_root.add_child(d)
	## Directly after the top-level child of `_phone_root` that holds the map: everything built after it stays on top.
	var map_child: Node = viewport_content
	while map_child != null and map_child.get_parent() != _phone_root:
		map_child = map_child.get_parent()
	if map_child != null:
		_phone_root.move_child(d, map_child.get_index() + 1)

## The first step of the phone's system-back chain after windows and the domain fan (proposal section 4's order:
## window, fan, quick drawer, ...): an open quick drawer closes. True when it closed one.
func _close_quick_drawer_for_back() -> bool:
	var qb = get("phone_quick")   ## `DccApp`'s QuickBarPhone; a bare `DccShell` has none
	if qb == null:
		return false
	var d: Control = qb.call("drawer")
	return d != null and is_instance_valid(d) and bool(d.call("close_if_open"))

## Public phone-geometry accessors for the quick bar's own nodes (`quick_drawer.gd`), which are not
## part of this file but must size in the same units: `phone_px` is `_pscale`, `phone_tap` is `_ptap`
## (raised to the 44 dp floor), `phone_font` is `_pfont`. They exist so nothing computes its own scale
## (see `phone_scale()`'s comment).
func phone_px(dp: float) -> int:
	return _pscale(dp)

func phone_tap(dp: float) -> int:
	return _ptap(dp)

func phone_font(dp: float) -> int:
	return _pfont(dp)

## Phone only: the band, in device px, the armed-tool strip takes above the bottom of the map area (0 off phone
## or while the strip is hidden). `ViewportHost` lifts the zoom row by it; see `ToolStrip.reserve()`. The name
## says "pill" because `viewport_host.gd` has called it that since the pill was all there was (MS-5's second
## step made the pill the strip's chip, and renaming the shell method for a cosmetic is a separate change).
func phone_tool_pill_reserve() -> float:
	if not _phone or _tool_strip == null:
		return 0.0
	return _tool_strip.reserve()

## Ruling EJ item 2 (MS-5): called by `arm_tool` when a tool was armed from the map. Drops the phone tool sheet
## to peek (portrait only: landscape has no detents) unless it is already there. Changes nothing else.
func phone_drop_sheet_to_peek() -> void:
	if not _phone or _landscape or _phone_detent == "peek":
		return
	set_phone_detent("peek")

## MS-5: the tool strip's chip tap. Raises the phone tool sheet to half when it is at peek, and lowers it to peek
## when it is above, so one tap opens "the full options" the sheet draws for the armed tool and the next closes
## them (`MOBILE_SHELL_PROPOSAL.md` section 2.1). It changes neither the tab nor the domain: the sheet already shows
## the armed tool's own options on whichever tab is lit (MAP's options row, WORLD's phone column, a CIVIL or CARTO
## dock), and switching tab for it would leave the domain the tool belongs to. Portrait only: in landscape the
## sheet has no detents (`_snap_phone_sheet`), so the tap is a no-op there and the options are already beside the
## map. Never arms or disarms anything.
func phone_toggle_tool_options() -> void:
	if not _phone or _landscape:
		return
	set_phone_detent("half" if _phone_detent == "peek" else "peek")

## A short self-dismissing sentence over `near` (the shell's phone toast); nothing off the phone.
func phone_toast(text: String, near: Control = null) -> void:
	_show_phone_toast(text, near)

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
##
## **`half` is content-sized for the MAP tab specifically, Ruling Z
## (`LARGE_ITEM_RULINGS.md`, 2026-09-21).** `PHONE_DETENT_HALF_FRAC` is
## transcribed from the prototype's own four-tab sheet, where every tab fills
## the space it opens. MAP's sheet body is `tool_options_row` alone --
## `_tool_options_cartography_default()`'s one label row -- so the flat
## fraction opened a sheet 92% blank regardless of world state
## (`OUTSTANDING_WORK.md`'s own measured row, 933/1003 px blank with no world,
## 912/1003 with one). The domain tab keeps the untouched flat fraction: its sheet
## body is the dock's own multi-row category column (WORLD included since Ruling EH item 1, which retired
## the phone's own GENERATE column), which the fraction was sized for, and every other tab (`story`/`menu`)
## doesn't drive the sheet's height at all (`_phone_tab_drives_sheet()`). Gated on
## `_phone_tab == "map"` rather than on which body is visible --
## `_pick_phone_tab()` sets `_phone_tab` before it calls
## `_set_phone_detent("half")`, but only calls `_refresh_phone_sheet_body()`
## (which flips that visibility) AFTER, so the visibility flag would still
## name the PREVIOUS tab's content on the very transition that matters most.
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
			var half := maxf(peek, round(fh * PHONE_DETENT_HALF_FRAC))
			if _phone_tab == "map":
				## Never grow past what the flat fraction already gives (the
				## ceiling every other tab still gets) and never shrink below
				## `peek` -- `clampf`'s own bounds enforce both without a second
				## `maxf`/`minf` pair.
				half = clampf(_phone_map_sheet_content_height(), peek, half)
			return half

## MAP's real content height: the header block (`_phone_sheet_grab` -- pill row
## plus title row, whatever it is actually laid out to, not the constant it
## happens to be floored at) plus `tool_options_row`'s own minimum (a single
## line for MAP), plus that row's own bottom margin and the sheet panel's own
## stylebox margins -- every piece `_build_phone_tool_sheet()` actually stacks
## above and around the scroller, read back rather than re-derived by formula.
## `_phone_sheet_box(false)` is called again rather than cached: its own doc
## comment already calls the function pure and a second call harmless.
##
## Runs only from `_phone_detent_height("half")`, which `_pick_phone_tab()`
## reaches only after it has already set `_phone_tab` and repopulated
## `tool_options_row` for the new domain (`_pick_bar_domain()` runs before the
## detent call in that function) -- so this never measures stale or empty
## content from the tab being left.
func _phone_map_sheet_content_height() -> float:
	var header_h := 0.0
	if _phone_sheet_grab != null and is_instance_valid(_phone_sheet_grab):
		header_h = _phone_sheet_grab.get_combined_minimum_size().y
	var row_h := 0.0
	if tool_options_row != null and is_instance_valid(tool_options_row):
		row_h = tool_options_row.get_combined_minimum_size().y
	## The MAP body (`_refresh_phone_map_row()`, MS-3) is the sheet's real content now, an expanding ScrollContainer
	## whose own minimum is ~12 px, so counting only the 40 px tool-options row sized `half` at header + 40 and left
	## the body 66 px of its 239 px sheet (Ruling Z's figure predates MS-3; found by `_p3dheight_probe` 2026-11-13,
	## the "first MAP tap" row in OUTSTANDING_WORK.md). Its column's minimum is what the body wants; the caller's
	## `clampf` against the flat fraction still caps it, and the body scrolls past that. `_set_phone_detent()` runs
	## `_refresh_phone_map_row()` (which fills the body) before `_snap_phone_sheet()` asks for this, so the column is
	## already built on the very first tap. Peek never reaches here.
	var body_h := 0.0
	if _phone_map_body != null and is_instance_valid(_phone_map_body) and _phone_map_body.visible and _phone_map_body.get_child_count() > 0:
		var col := _phone_map_body.get_child(0) as Control
		if col != null:
			body_h = col.get_combined_minimum_size().y
	var sheet_style := _phone_sheet_box(false)
	var vpad := sheet_style.get_margin(SIDE_TOP) + sheet_style.get_margin(SIDE_BOTTOM)
	return header_h + maxf(row_h, body_h) + float(_pscale(10)) + vpad

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

## Public setter for the phone tool sheet's detent; accepts only `peek`, `half` or `full` and
## moves there via `_set_phone_detent()`. Anything else is ignored.
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
	_refresh_phone_map_row()
	_sync_dom_host_visible()
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

## Builds a full-rect dimming scrim (black at 0.55 alpha, a labelled judgement) that blocks
## input to everything beneath and calls `on_tap` when tapped.
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

## Builds the sheet header's close square (a 44 dp target) wired to `on_press`; named and
## tooltipped for touch accessibility.
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
	## PASS, not the Button default STOP (owner finding 2026-10-26): STOP ends Godot's bubbling of a touch
	## drag, so a swipe that began on a result row never reached the results list's `ScrollContainer`.
	## The row still gets every press and release; `pressed` fires as before.
	row.mouse_filter = Control.MOUSE_FILTER_PASS
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
## the two sheet close buttons here, `PhoneMenu._open_left_sheet() (deleted by MS-4)`, and four
## in `DccApp.toggle_region()` (its `ID_WIN_RESET` branch closes both sheets,
## and its two phone rows toggle one each).

# -- Phone overlay state ---------------------------------------------------
#
# Every phone overlay is mutually exclusive -- opening any one closes all the
# others, including a dock sheet -- so there is exactly one state variable per
# overlay and one shared teardown rather than a general stack. `PhoneMenu` keeps
# its own drill stack inside itself; from out here it is one more overlay.

## Closes every transient phone overlay in one go (search, overflow and undo popups, the phone menu, both dock sheets and their open flags) and then hides any visible `Popup` descendant, which is how the Layers popover is reached (a Control walk never reaches a Window). Called before another overlay opens so they stay mutually exclusive. Safe with any of them unbuilt: each is null-checked.
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
	## Both absent-reason strings name the missing `DccApp.save_project`/`close_project` methods.
	_phone_overflow_saved = _phone_overflow_row(col, "Save project", "save_project",
		"This build of the app can't save.")
	_phone_overflow_theme = _phone_overflow_row(col, "Theme", "toggle_theme", "")
	_phone_overflow_row(col, "Close world", "close_project",
		"This build of the app can't close the world.")

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
		if bridge == null or _phone_overflow_wired or not _phone:
			return
		_phone_overflow_wired = true
		var cb := func(_path: String):
			_phone_saved_at = Time.get_time_string_from_system().substr(0, 5)
		bridge.project_saved.connect(cb)
		_phone_overflow_links.append([bridge, "project_saved", cb])
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
const SEARCH_HINT := "Start with a scope and a space to narrow: s settlements, f factions, c cultures, lb labels, r routes, l landmarks, n notes, . commands."

## The search prefix table, in display order; each row has `prefix`, `label` and exactly one of `entity`, `commands` or `why` (see the note above).
const SEARCH_SCOPES: Array = [
	{"prefix": "s", "label": "settlements", "entity": "settlement"},
	{"prefix": "f", "label": "factions", "entity": "faction"},
	{"prefix": "c", "label": "cultures", "entity": "culture"},
	{"prefix": "lb", "label": "labels", "entity": "label"},
	{"prefix": "r", "label": "routes", "entity": "route"},
	## VE-9: the vault's notes, searched by `vault_search` (names always, text once indexed) through `NotesFind`.
	{"prefix": "n", "label": "notes", "notes": true},
	{"prefix": ".", "label": "commands", "commands": true},

	## Ruling EI gave a placed landmark a derived name (type plus nearest place) and Ruling EL a user-given one that
	## is saved; `PlaceSearch._add_landmarks` indexes them, so this scope is live. A hit pans the map to the cell.
	{"prefix": "l", "label": "landmarks", "entity": "landmark"},
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

## How many result rows `_fill_search_results()` builds at most, on every surface that shows search results.
## 2026-11-12, OUTSTANDING_WORK.md "Search field lag of 10 to 40 s on the 24601 world over adb". Measured on this
## machine (`_phonesearchperf_probe.gd`, seed 24601 at 2048 x 1311, 240 settlements, 225 roads, phone form): every
## keystroke built one `Button` row per hit, about ten nodes each, and the cost was the rows, not the match: the
## one-letter query "a" (306 hits) took 68 ms in the handler and a further 71 ms of layout against 1.2 ms for the
## match itself, and a query with no hits took 1.5 ms. The cost is linear in hits, so a world with more places, and a
## handset that is an order of magnitude slower than this desktop, turns the broad first letters of a name (and the
## empty query that opens the overlay, which lists everything) into seconds per keystroke. 60 is ten screens of the
## phone sheet's six rows and well past what a person scrolls before typing another letter; the ranking puts the
## STARTS WITH band first, so the cap keeps the best hits. A labelled judgement, not a measured optimum.
const SEARCH_ROW_CAP := 60
## Band headers for `CommandIndex` results, in ranker order: title, group, blurb.
const COMMAND_BANDS: Array = ["MATCHED THE NAME", "MATCHED THE MENU", "MATCHED THE DESCRIPTION"]

## Which seam band a place-search row belongs to: 0 name starts with the needle, 1
## contains it, 2 neither (matched on kind or subtitle). Must mirror the ranker that
## produced the order (see the comment above).
static func _place_band(row: Dictionary, needle: String) -> int:
	var at := String(row.get("name", "")).to_lower().find(needle)
	if at == 0:
		return 0
	return 1 if at > 0 else 2

## Which seam band a command row belongs to: 0 title contains the needle, 1 group (menu)
## contains it, 2 otherwise (description). Mirrors the command ranker; same warning as
## `_place_band`.
static func _command_band(row: Dictionary, needle: String) -> int:
	if String(row.get("title", "")).to_lower().find(needle) >= 0:
		return 0
	return 1 if String(row.get("group", "")).to_lower().find(needle) >= 0 else 2

## Splits a raw field value into `{scope?, query}`. `scope` is ABSENT unless the
## token before the first space is exactly one of `SEARCH_SCOPES`' prefixes --
## so `lb ald` scopes to labels, `l ald` scopes to landmarks,
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
		## MS-9: `true` runs each menu's own open refreshers BEFORE its rows are read, so a check or radio shows
		## what the menu would show if it were opened now, and the title it was indexed under is the title the
		## item still has (a refresher may rewrite item text).
		idx.build(self, _find_engine_bridge(), true)
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

	if scope.has("notes"):
		var nf := NotesFind.rows(self, _find_engine_bridge(), query)
		nf["scope"] = scope
		return nf

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
## **A `.` command row runs** (phone since 2026-10-26, desktop since PC-9's finding S4, every kind since MS-9):
## a press goes to `_run_search_command()`, which runs the row through `CommandIndex.run_search()` and otherwise
## says why it cannot. A check or radio shows its state in the second line. The second line says where the
## command lives (its `group`, the menu it is under) or, when unavailable, `why`.
func _fill_search_results(container: VBoxContainer, res: Dictionary, close_fn: Callable) -> void:
	for child in container.get_children():
		child.queue_free()
	if res.has("note"):
		container.add_child(_search_notice(String(res["note"])))
		return
	var rows: Array = res.get("rows", [])
	if rows.is_empty():
		container.add_child(_search_notice("No matches"))
		if res.has("footer"):
			container.add_child(_search_notice(String(res["footer"])))
		return
	## The rows are capped (`SEARCH_ROW_CAP`); the footer below says so, so a cut list is never read as the whole answer.
	var matched := rows.size()
	if matched > SEARCH_ROW_CAP:
		rows = rows.slice(0, SEARCH_ROW_CAP)
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
			## MS-9: a check or radio row says which way it is set ("On", "Off", "Selected", "Not selected") so a
			## press that toggles it can be seen to have; a row with no such state adds nothing (`state_word()` is
			## "" then). Only an available row: an unavailable one shows its reason alone.
			var state_text := _ensure_command_index().state_word(d) if bool(d.get("available", false)) else ""
			if state_text != "":
				line2 = "%s · %s" % [state_text, line2]
			## 2026-10-26 (owner finding, OnePlus 6T): the row used to be a DISABLED readout, so a tap on a command
			## did nothing at all and read as a dead list. A tap is never silent (MS-9 widened it from "pinnable rows
			## only"): a command runs through `CommandIndex.run_search()` (the same `id_pressed` the menu emits) and
			## the overlay closes, except a check or radio, which stays open and shows its new state; a row that
			## cannot run (disabled, a readout, a pointer) says why from `search_block()` and stays open.
			var cmd := _phone_list_row(String(d.get("title", "")), line2, _run_search_command.bind(d, close_fn))
			(cmd as Button).accessibility_description = 				"%s. Lives under the %s menu. Tap to run it." % [line2, String(d.get("group", ""))]
			## The result row carries the index row it draws (a meta, not a node name: sibling rows share a title and
			## Godot renames duplicate names), so a probe (`_ms9_probe.gd`) presses the row the person would press and
			## compares what happened with that row's own handler.
			(cmd as Button).set_meta("search_row", d)
			## PC-9 (`PC_SHELL_PROPOSAL.md` s.3.8, finding S4): the desktop dialog's rows run too, through
			## the same `_run_search_command()`. Until 2026-11-08 it drew them as a disabled readout
			## because it had no toast to say why a row would not run; it says it in its count line now.
			## CQ-6 (`CUSTOM_QUICK_BAR_SCOPE.md` s.3.2): a pinnable command gets a star beside its row, the phone's
			## equivalent of Blender's right-click ▸ Add to Quick Favorites. Only on a row the index calls
			## pinnable, so a reason-bearing row keeps its reason and gains no control.
			if String(d.get("pin_key", "")) != "" and bool(d.get("pinnable", false)):
				cmd.size_flags_horizontal = Control.SIZE_EXPAND_FILL
				var line := HBoxContainer.new()
				line.add_theme_constant_override("separation", 0)
				## The column is `SIZE_EXPAND_FILL` inside its scroller now (see `_build_phone_search_overlay`),
				## so the line fills it and the star lands at the right edge with no width hack.
				line.size_flags_horizontal = Control.SIZE_EXPAND_FILL
				line.add_child(cmd)
				line.add_child(_command_star(d))
				container.add_child(line)
			else:
				container.add_child(cmd)
			continue
		var kind := String(d.get("kind", ""))
		var sub := String(d.get("subtitle", ""))
		var line2 := "%s — %s" % [kind, sub] if kind != "" and sub != "" else kind + sub
		container.add_child(_phone_list_row(String(d.get("name", "")), line2,
			_select_search_hit.bind(d, close_fn)))
	if matched > SEARCH_ROW_CAP:
		container.add_child(_search_notice("Showing the first %d of %d matches. Type more to narrow the list." % [SEARCH_ROW_CAP, matched]))
	## A qualifier the list must not hide (the notes scope's names-only / capped answers).
	if res.has("footer"):
		container.add_child(_search_notice(String(res["footer"])))

## A tap on a `.` command row (phone and desktop dialog alike). MS-9: EVERY row kind, not only the pinnable ones.
## **One path:** the run is `CommandIndex.run_search()`, whose tail `_fire()` is also `run_row()`'s (the owning
## popup's own `id_pressed`, or `arm_tool`), the code a quick-bar chip and a menu click share; this never emits a
## menu id itself. Until MS-9 a check, a radio or a parameter only toasted a reason; now a check or radio runs and
## redraws with its new state, a parameter opens its control (`reveal_param()`), and a row that cannot run (disabled,
## a readout, a pointer) says why and changes nothing. The reason is a toast on the phone and the dialog's count
## line on the desktop, which has no toast and keeps the dialog's own text field.
func _run_search_command(d: Dictionary, close_fn: Callable) -> void:
	var idx := _ensure_command_index()
	## MS-9: every row goes through `CommandIndex.run_search()`, which gates it (`search_block()`) and then ends in the
	## same `_fire()` a quick-bar chip ends in. The three outcomes differ only in what the surface does after:
	##   - a check or radio STAYS OPEN and redraws, so the row shows its new state ("On" to "Off");
	##   - a parameter closes the surface first and then opens its control, so the focus lands after the hide;
	##   - any other command closes the surface and runs, as it always has.
	## A row that cannot run says why and changes nothing.
	var kind := String(d.get("kind", ""))
	var blocked := idx.search_block(d)
	if blocked != "":
		_say_search_reason(blocked)
		return
	if bool(d.get("checkable", false)):
		var why_toggle := idx.run_search(d)
		if why_toggle != "":
			_say_search_reason(why_toggle)
			return
		## Rebuild, not just redraw: the press may have changed what else is available (Undo after an edit), and the
		## index is rebuilt on every open for exactly that reason.
		_command_index = null
		if _phone:
			_refresh_phone_search()
		else:
			_refresh_desktop_search()
		return
	if kind == "param":
		var why_param := idx.run_search(d)
		if why_param != "":
			_say_search_reason(why_param)
			return
		close_fn.call()
		return
	close_fn.call()
	var why := idx.run_search(d)
	if why != "":
		_say_search_reason(why)

## MS-9: shows the generation parameter `key` to the person who searched for it. Returns "" on success, else the reason.
## Makes the WORLD dock visible on every form, then asks the world workspace to open the parameter's category and
## focus its control (`WorldWorkspace.focus_param()`). Desktop: the left dock is re-expanded if collapsed. Tablet:
## the same `_toggle_dock()` (MS-7's slot follows the flag). Phone: the domain tab's sheet, at least at its half
## detent (`_pick_phone_tab()`, which leaves a higher detent alone). Refused with the domain's own sentence when
## Session mode hides WORLD.
func reveal_param(key: String) -> String:
	var panel: Control = _workspace_panels.get("world")
	if panel == null or not panel.has_method("param_category"):
		return "This build has no parameter dock to open."
	var cat := String(panel.call("param_category", key))
	if cat == "":
		return "This parameter has no control in the dock."
	if session_hides_domain("world"):
		return SESSION_REFUSED
	select_domain_category("world", cat)
	if _phone:
		_pick_phone_tab("domain", false)
	elif is_dock_collapsed("left"):
		_toggle_dock(true)
	return String(panel.call("focus_param", key))

## Where `_run_search_command()` says why a row did not run: the phone's toast, or the desktop
## Find dialog's count line (which the next keystroke rewrites, so it never goes stale).
func _say_search_reason(text: String) -> void:
	if _phone:
		phone_toast(text)
	elif _desktop_search_count != null and is_instance_valid(_desktop_search_count):
		_desktop_search_count.text = text

## The pin / unpin star beside a pinnable command in the phone's `.` search results (CQ-6). 48 dp square (the
## quick bar's chip size, floor 44), `★` when the command is pinned and `☆` when not; a tap toggles it through
## the store's own `quick_bar_add` / `quick_bar_remove` (so the cap and the refusal sentence exist once, in
## `DccSettings`), toasts a refusal, and refreshes the phone's chip row.
func _command_star(row: Dictionary) -> Button:
	var key := String(row["pin_key"])
	var star := Button.new()
	star.name = "PinStar"
	star.flat = true
	star.focus_mode = Control.FOCUS_NONE
	star.custom_minimum_size = Vector2(_ptap(QuickBarPhone.CHIP_DP), _ptap(QuickBarPhone.CHIP_DP))
	star.add_theme_font_size_override("font_size", _pfont(16))
	star.add_theme_color_override("font_color", DccTheme.c("accent"))
	star.add_theme_color_override("font_hover_color", DccTheme.c("accent"))
	star.add_theme_color_override("font_pressed_color", DccTheme.c("accent"))
	_paint_star(star, key)
	star.pressed.connect(_toggle_star.bind(star, row))
	return star

## Sets the star's glyph, tooltip and accessibility name from the store: `★ Unpin` when `key` is pinned.
func _paint_star(star: Button, key: String) -> void:
	var pinned := false
	for p in DccSettings.quick_bar_value("bar"):
		if String(p["k"]) == key:
			pinned = true
	star.text = "★" if pinned else "☆"
	star.tooltip_text = "Remove from the quick bar" if pinned else "Pin to the quick bar"
	star.accessibility_name = star.tooltip_text

## A tap on a command's star: pin it, or unpin it when it already is.
func _toggle_star(star: Button, row: Dictionary) -> void:
	var key := String(row["pin_key"])
	var pinned := false
	for p in DccSettings.quick_bar_value("bar"):
		if String(p["k"]) == key:
			pinned = true
	if pinned:
		DccSettings.quick_bar_remove("bar", key)
	else:
		var why := DccSettings.quick_bar_add("bar", key, String(row.get("title", "")))
		if why != "":
			_show_phone_toast(why, star)
	_paint_star(star, key)
	var q = get("phone_quick")
	if q != null:
		q.refresh()

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
		## **A declined chip is floored too, and the reason is not symmetry.** No scope is declined today (`l`
		## was, until landmarks were indexed), but `why` is a state a row can be in again, and an exemption keyed
		## on `disabled` would put a live chip back under the floor the day one is, silently.
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

## Re-runs the phone overlay's search against its field text and refills chips, count and
## result rows. No-op until the overlay's widgets exist.
func _refresh_phone_search() -> void:
	if _phone_search_results == null or not is_instance_valid(_phone_search_results):
		return
	var res := _search_result_set(_phone_search_field.text)
	_fill_scope_chips(_phone_search_chips, res)
	_phone_search_count.text = _search_count_text(res)
	_fill_search_results(_phone_search_results, res, func(): _set_search_open(false))

## Desktop twin of `_refresh_phone_search()`: refills the dialog's chips, count and rows and
## hides the dialog on pick. No-op until the dialog's widgets exist.
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
##
## **PC-1 (`PC_SHELL_PROPOSAL.md` §3.1, finding S4): a hit also selects** what it names, through
## `DccApp.select()`, for the two kinds whose `id` is the selection's own handle: a settlement (its index
## in `bridge.settlements()`, `place_search.gd::_add_settlements()`) and a faction (its id). Other kinds
## (labels, ways, routes) only move the camera, as before. Asked through `has_method`, because this
## base class has no selection of its own.
func _select_search_hit(d: Dictionary, close_fn: Callable) -> void:
	var vh := _find_viewport_host()
	## A row with no position (a culture) pans nowhere; `(0, 0)` would be a corner that is not its place.
	if vh != null and d.has("x") and d.has("y"):
		vh.move_view_to(float(int(d.get("x", 0))), float(int(d.get("y", 0))))
	var entity := String(d.get("entity", ""))
	## VE-9: a notes hit opens that note in the vault window (the app owns it; a bare shell has none).
	if entity == "note" and d.has("rel"):
		var vw = get("vault_window")
		if vw != null and vw.has_method("open_note"):
			vw.call("open_note", String(d["rel"]))
		close_fn.call()
		return
	if (entity == "settlement" or entity == "faction" or entity == "culture") and d.has("id") and has_method("select"):
		call("select", entity, int(d["id"]))
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
	## Phone fix 2026-10-26 (owner findings on the OnePlus 6T): the results column is the scroller's only
	## child and a `ScrollContainer` gives a child its MINIMUM width unless that child carries
	## `SIZE_EXPAND` -- so the column, and with it every row's hit area, was only as wide as the widest
	## row (a tap on the right half of a short row fell on nothing), and an autowrap `No matches` notice
	## had a minimum width of about one letter and drew one letter per line. Vertical only: a results list
	## has no sideways content, so sideways scrolling is off and the column is exactly the panel's width.
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	col.add_child(scroll)
	_phone_search_results = VBoxContainer.new()
	_phone_search_results.add_theme_constant_override("separation", 0)
	_phone_search_results.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_phone_search_results)

	overlay.add_child(panel)
	overlay.visible = false
	return overlay

## Opens or closes the phone search overlay, closing every other overlay first. On open it
## clears the field, drops the place-search cache and focuses the field next frame.
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

## MS-1 S4.5 (`window_represent.gd`, `S4_INVENTORY.md` section 18): the desktop `Find on map` dialog across a class swap. It is a plain
## `AcceptDialog` the shell builds on first use, so it has no `represent_capture()` of its own; `DccApp._capture_windows()` /
## `_represent_windows()` / `_restore_windows()` call these three beside the windows `_make_window()` builds. Returns
## `{"built": false}` when the dialog was never built (a swap leaves it unbuilt, as a first open then builds it for the class it is in).
## What is carried: whether it is open, the query text (which holds the scope prefix, `SEARCH_SCOPES`, so the active chip follows
## it) and the results' scroll. The results themselves are re-derived from the world by `_refresh_desktop_search()`.
func search_represent_capture() -> Dictionary:
	if _desktop_search_dialog == null or not is_instance_valid(_desktop_search_dialog):
		return {"built": false}
	return {"built": true, "open": _desktop_search_dialog.visible, "text": _desktop_search_field.text,
		"scroll": WindowRepresent.scroll_value(_desktop_search_results)}

## Frees the old dialog and clears the four handles the builder set. Before the registries are pruned, like the windows.
func search_represent_free() -> void:
	if _desktop_search_dialog != null and is_instance_valid(_desktop_search_dialog):
		_desktop_search_dialog.free()
	_desktop_search_dialog = null
	_desktop_search_field = null
	_desktop_search_chips = null
	_desktop_search_count = null
	_desktop_search_results = null

## Builds the dialog again for the new class (when it existed) and, if it was open, opens it with its text. A coroutine: the scroll
## waits for the new layout.
func search_represent_restore(bag: Dictionary) -> void:
	if not bool(bag.get("built", false)):
		return
	_ensure_desktop_search_dialog()
	## The phone searches in its own overlay (`open_find_on_map()`): a 440 x 480 desktop dialog over a 393 dp column is not its form,
	## so the dialog is rebuilt closed there and the user's query on the other class is not carried onto it.
	if not bool(bag["open"]) or _phone:
		return
	_open_desktop_find_on_map()
	var q := String(bag["text"])
	if q != "":
		_desktop_search_field.text = q
		_refresh_desktop_search()
	await WindowRepresent.restore_scroll(_desktop_search_results, int(bag["scroll"]))

## Builds the desktop `Find on map` dialog (field, chips, count, results) on first use only;
## later calls return immediately. The 400 x 420 minimum size is a labelled judgement.
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

## Returns this shell's direct child that is an `EngineBridge`, or null (a bare `DccShell` in a screenshot probe has none), so every caller must null-check. Only searches direct children, not the whole tree.
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
		## S4.3: no chip, no wiring (a swap left the phone before this deferred call ran).
		if bridge == null or _phone_undo_chip == null:
			return
		## The signals a commit to the height field can arrive through, plus `annotations_edited` (a label, icon or
		## landmark-name edit is an undo step with no height snapshot, and emits nothing else that reaches here).
		## NOT `generation_started`/`params_changed` -- a dial moving or a
		## generate merely beginning commits nothing, so `can_undo()` cannot
		## have changed yet; NOT `project_saved` -- writing a `.zip` reads the
		## height field, it does not touch the undo stack.
		## MS-1 S4.2: connected once (`_phone_undo_wired`); the lambdas read the chip at call time.
		if not _phone_undo_wired:
			_phone_undo_wired = true
			for sig in ["generation_finished", "params_applied", "world_loaded", "dirty_changed", "annotations_edited"]:
				var cb := func(_a = null): _refresh_phone_undo_chip()
				bridge.connect(sig, cb)
				_phone_undo_links.append([bridge, sig, cb])
			## The shared look history emits `changed` on every commit, undo, redo and clear (the
			## same hook `_wire_menu_squares()` uses), so a look-only edit shows the chip too
			## (OUTSTANDING_WORK "phone undo chip answers only bridge.can_undo()", 2026-10-12).
			var lh = get("look_history")
			if lh != null and lh.has_signal("changed"):
				lh.changed.connect(_refresh_phone_undo_chip)
				_phone_undo_links.append([lh, "changed", Callable(self, "_refresh_phone_undo_chip")])
		_refresh_phone_undo_chip()
	).call_deferred()

## Shows the phone undo chip only while there is a step to undo: the look history OR the height
## stack, the same `DccApp.can_undo_any()` the desktop squares and `Edit > Undo` read. A bare
## `DccShell` has no such method and falls back to the height stack alone. Must never answer
## from `bridge.can_undo()` alone: a look-only history would then have no chip (it was reachable
## only through MORE > Edit > Undo).
func _refresh_phone_undo_chip() -> void:
	if _phone_undo_chip == null:
		return
	var bridge := _find_engine_bridge()
	if bridge == null:
		_phone_undo_chip.visible = false
		return
	_phone_undo_chip.visible = bool(call("can_undo_any")) if has_method("can_undo_any") 		else bridge.can_undo()

## Chip press start: arms the hold detector with a one-shot timer of `PHONE_UNDO_HOLD_SEC`,
## after which `_check_phone_undo_chip_hold()` decides tap versus hold.
func _on_phone_undo_chip_down() -> void:
	_undo_chip_down = true
	_undo_chip_hold_fired = false
	_undo_chip_press_no += 1
	get_tree().create_timer(PHONE_UNDO_HOLD_SEC).timeout.connect(
		_check_phone_undo_chip_hold.bind(_undo_chip_press_no))

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
func _check_phone_undo_chip_hold(press_no: int) -> void:
	if not _undo_chip_down or press_no != _undo_chip_press_no:
		return  ## Already released -- a tap, not a hold; `_on_phone_undo_chip_up()` handled it.
	_undo_chip_hold_fired = true
	_haptic("detent")
	_open_phone_undo_popover()

## Chip release: after a hold does nothing (the popover is already open); a short tap closes
## an open popover, else undoes one step.
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

## Builds one row of the edit-history popover from a ledger entry (`label`, `seq`, `steps`,
## `reversible`); irreversible entries are disabled. Rows roll back via `undo_revert_to()`.
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
		row.tooltip_text = ("%s · %s. Reverts to the state before this "
			+ "operation, discarding the %d step%s after it as well -- history here is "
			+ "linear, so there is no branch to come back to.") % [
				String(d.get("subsystem", "")), String(d.get("detail", "")),
				steps - 1, "" if steps == 2 else "s"]
		row.pressed.connect(func(): _phone_revert_history(seq))
	else:
		row.disabled = true
		var why := String(d.get("reason", ""))
		row.tooltip_text = ("%s · %s. Nothing is still held to revert this step%s" % [
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
	## LLM-5: the desktop panel's `_do_revert` refusal, same reason
	## (`undo_group.gd::blocks_revert_to`); `undo_groups` lives on `DccApp`.
	var groups = get("undo_groups")
	if groups != null:
		var split: String = groups.blocks_revert_to(seq)
		if split != "":
			if _phone_undo_pop != null:
				_phone_undo_pop.visible = false
			_show_phone_toast(split, _phone_undo_chip, 3.2)
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
		## Same third line the desktop revert path (`right_dock.gd::_do_revert`)
		## uses, same reason: a live LOD tile keeps its own captured
		## `base_tex` from the OLD texture (`OUTSTANDING_WORK.md`, "the
		## in-session tile cache is not invalidated by a sculpt").
		host.invalidate_lod_tiles()
		host.set_preview_texture(null)
	_refresh_phone_undo_chip()
	_show_phone_toast("Reverted %d step%s" % [done, "" if done == 1 else "s"],
		_phone_undo_chip, 2.4)

## Tap. "Undo: ... map edits only" -- `can_undo()`/`undo_last()` are the
## GLOBAL undo (`engine_bridge.gd`'s "Global heightmap undo" block: "Deliberately
## NOT the same thing as `sculpt_undo`/`sculpt_redo`... those pop a stamp off an
## uncommitted draft, these pop a whole committed height field"). Since Ruling BA
## (2026-09-28) the one civilisation edit it also reverts is a way deleted from
## the map context card -- still a map edit -- and the overlay is told when that
## is what came back.
## **Routed through the wrapped `undo_last()`, not the raw `bridge.undo_last()`
## this chip used to call directly.** That call reverted the height field but
## never repainted it -- `DccApp.undo_last()`'s own doc comment names the two
## lines that do (`viewport.map_view.texture = bridge.color_texture()`,
## `invalidate_lod_tiles()`), and this chip skipped both, along with the
## status line and the History dock refresh `_menu_bar_undo()` already gets.
## The map only caught up once some other signal happened to repaint it (a
## dial move, a fresh generate), which read as "the undo did nothing" on a
## phone that has no menu-bar Undo to fall back on.
##
## The label for the toast is read *before* the call (`undo_label()`, a pure
## preview query -- `next_undo_step()`'s own doc comment), since
## `DccApp.undo_last()` returns nothing to build "Undid: %s" from and a
## second read afterwards would already be one step stale.
##
## Degrades to doing nothing on a bare `DccShell` (the screenshot probes),
## which has neither `undo_last()` nor `viewport` -- the same guard
## `_menu_bar_undo()` uses, and for the same reason.
func _do_phone_undo() -> void:
	var bridge := _find_engine_bridge()
	if bridge == null:
		return
	## Routed like the chip's visibility: a look-only history undoes through `undo_last()` too.
	var routed := has_method("can_undo_any")
	if not (bool(call("can_undo_any")) if routed else bridge.can_undo()):
		return
	var label: String = String(call("undo_target_label")) if routed else bridge.undo_label()
	if has_method("undo_last"):
		call("undo_last")
	_refresh_phone_undo_chip()
	if label != "":
		_show_phone_toast("Undid: %s" % label, _phone_undo_chip, 2.4)

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
# there was *no* entry point there; opened 2026-09-06, the deleted `_fill_sim()`'s first
# row is a `Transport strip on map` switch whose handler `phone_menu.gd::_toggle_sim_strip()` (deleted by MS-4)
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
	## The fallback axis at build; `_refresh_phone_sim_strip()` follows the world's range from then on (PC-3).
	## A fresh world has no range, so the phone's slider is exactly the shipped -400..1200 there.
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

## Whether the phone simulation strip exists and is visible.
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
	## The slider follows the world's range (PC-3); the bounds change only when they differ, with signals
	## blocked, because a bound that clamps the value would otherwise re-enter `value_changed` and write the
	## year back to the engine.
	var rng := tl_range()
	if int(_phone_sim_slider.min_value) != int(rng["min"]) or int(_phone_sim_slider.max_value) != int(rng["max"]):
		_phone_sim_slider.set_block_signals(true)
		_phone_sim_slider.min_value = int(rng["min"])
		_phone_sim_slider.max_value = int(rng["max"])
		_phone_sim_slider.set_block_signals(false)
	_phone_sim_slider.set_value_no_signal(float(year))
	_phone_sim_play.text = DccIcons.SYMBOLS["pause"] if tl_playing \
		else DccIcons.SYMBOLS["play"]
	_phone_sim_play.tooltip_text = ("Pause" if tl_playing else "Play") \
		+ " -- %s. The cursor is the civilisation timeline's own year; a year CIVIL ▸ Timeline has recorded loads its territory, and any other year keeps the territory already shown." % tl_state_text()
	## `civ_goto_year`. "CIVIL > Politics" was a stale category name (audit B15).
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
	## Investigated (not assumed) after an owner report that a toast read
	## "faint, with a map label drawn on top": `wrap` is `_phone_root.add_child
	## (wrap)`'d below, and `_phone_root`'s FIRST child is the map viewport
	## (`_build_phone_shell()`, `_phone_root.add_child(vp)` before anything
	## else) -- every toast is already a LATER sibling than the map, which
	## Godot draws in tree order at equal `z_index`, so the map cannot already
	## be drawing over it by ordinary sibling order. The report's real cause
	## was a screenshot taken mid-fade-in (`modulate.a` between 0 and 1, which
	## fades the whole toast -- background AND text -- letting the map show
	## through), fixed at the capture site (`_pandraghint_probe.gd`), not here.
	## `z_index` is still set, defensively: nothing in this codebase sets a
	## `z_index` above `2` (`_export_grid_layer.gd`, `viewport_host.gd`) and
	## `_navpad`/journey markers/etc. all rely on sibling order the same way
	## this file's own toasts did -- so `50` costs nothing and removes "does a
	## future map layer's z_index change put it back on top of a toast" as a
	## question anyone has to re-derive from source again.
	wrap.z_index = 50
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
	## MS-1 S4.4: the tween belongs to the TOAST (`wrap.create_tween()`), not to the shell. A class swap frees the
	## composition and with it a toast still fading; a tween owned by the shell kept running and, when it finished,
	## called this lambda with its captured `wrap` already freed ("Lambda capture at index 0 was freed", seconds after
	## the swap). A tween bound to a node stops when the node is freed, so the lambda is never called on a dead toast.
	var tw := wrap.create_tween()
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

## Coach marks (§13 chrome: "two subtle toasts, persisted" -- now three, see
## `pan_drag_hint` below). First-run hints into what THIS file itself builds
## and can point a toast at with a real node: the bottom bar's disclosure
## model (`_phone_menu_bar`) and the tool sheet's drag handle
## (`_phone_sheet_grab`).
## Long-press-to-sample -- the third candidate the ORIGINAL assigning brief
## named -- is deliberately not one of these: it is `map_overlay.gd`'s
## gesture, a file this pass does not own and has no node handle on, so a
## toast pointed at it would be a guess rather than an anchored hint.
##
## **`pan_drag_hint`** -- Ruling BO / `OUTSTANDING_WORK.md` ("a drag on the
## map does nothing until the hand tool is armed, with no on-screen cue").
## `viewport_host.gd`'s navpad ✋ already paints its OWN armed state (accent
## fill when `pan_mode()` is true, `_navpad_paint()`), which answers "is it
## armed right now" for someone already looking at the button -- what was
## missing is anything that tells a first-time user the button exists and
## that a bare drag needs it. That gap is exactly this coach mark's job, and
## it is deliberately NOT anchored to the navpad: `_navpad`/`_pan_btn` are
## private locals of `viewport_host.gd` (a file this task may not edit, per
## brief), so this points at nothing (`near = null`, which
## `_position_phone_toast()` already centres) rather than reaching into that
## file for a rect this pass has no right to add a getter for. Text and glyph
## (✋) are the SAME vocabulary the navpad button and `tool_fan.gd`'s pill
## already use (`pill_label()`), not a new coined term. Must never assume
## `viewport_host`/`_navpad` exists here -- if a future pass wants this
## anchored, that is a `viewport_host.gd` change (a public rect getter),
## made by whoever owns that file, not a reach-around from this one.
const _COACH_MARKS := [
	## Rewritten for MS-2's bar (MAP · domain · STORY · MENU), against `PHONE_TABS`, which is what the bar draws
	## since the tab migration. The old text named WORLD/CIVIL/CARTO and PANELS
	## -- three captions the bar no longer carries and one that was never a tab
	## at all -- and so pointed a first-run hint at a bar that does not exist.
	{"id": "bottombar_tabs",
		"text": "MAP · domain · STORY switch tasks here — hold the domain tab to change domain, MENU reaches everything else."},
	{"id": "sheet_handle", "text": "Drag this handle to expand tool options."},
	## Shown last (after the two that already existed) rather than first: the
	## nav-bar and sheet-handle hints are about controls that are on screen the
	## instant the app opens, while this one is about a gesture a user might
	## not attempt for several minutes -- ordering it after the other two shows
	## it in the same first-run session without competing with them for the
	## very first thing painted.
	{"id": "pan_drag_hint",
		"text": "A one-finger drag only moves the map while Pan (✋) is armed."},
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

## True from the moment a cold boot decides the coach marks must wait for
## `phone_project_picker.gd`'s own entry screen to close, until that report
## arrives and starts them. See `_start_or_defer_coach_marks()`'s own doc
## comment for the bug this holds off and why a frame-count guess cannot be
## the fix. Must only ever be flipped false by `_on_phone_entry_screen_
## visibility()` below, or left alone -- never read as "coach marks have
## started", which `_show_next_coach_mark()` running is the actual signal of.
var _coach_marks_wait_for_picker := false

## Starts the first-run coach marks, phone only, deferred one frame so the layout has settled.
## The deferred callee decides whether to wait for the project picker.
func _maybe_show_coach_marks() -> void:
	if not _phone:
		return
	(func(): _start_or_defer_coach_marks()).call_deferred()

## Ruling BK doc comment. Decides, on the SAME deferred frame this used to
## unconditionally start the sequence on, whether `phone_project_picker.gd`'s
## full-screen entry screen is about to cover it -- see that file's own
## `_on_phone_entry_screen_visibility()`-forwarding comment for the timing this
## exists to fix (`OUTSTANDING_WORK.md`, "the first coach mark fires under the
## phone project picker at cold boot", found 2026-09-28).
##
## Reads `bridge.has_world` (reflectively -- `get("bridge")`) rather than the
## picker's own `visible` flag, deliberately: `visible` is not yet meaningful
## at this point (`phone_project_picker.gd`'s `open()` has not run -- see that
## file's comment for the exact wait it is still behind), so checking it here
## would just repeat the original bug one line later. `bridge.has_world` is
## instead the REAL precondition `app.gd::_ready()` itself branches on to
## decide whether to open the picker at all (`if not bridge.has_world:
## _open_welcome_when_drawn()`), and it is already final by this deferred
## frame: `app.gd::_ready()` assigns `bridge` synchronously, immediately after
## the `super._ready()` call this whole sequence started inside of, and that
## whole function (including, on phone, constructing `phone_project_picker`
## itself) finishes before this file's own `call_deferred()` callback can run.
##
## `get("bridge")` rather than a typed reference: `DccShell` (this file) is
## the base class and does not declare `bridge` -- `DccApp` (`app.gd`) does --
## so a probe that drives a bare `DccShell` with no `App` layer has neither a
## `bridge` nor a picker, and treating that as "nothing to wait for" is the
## correct fallback there too, not a guess (matches the existing reflection
## idiom this file already uses for `_COACH_MARKS`, see that const's own
## comment).
##
## If a world already exists, the picker never opens (`app.gd`'s own branch
## above), so there is nothing to wait for and the marks start on this same
## frame -- unchanged from before this fix. Otherwise the picker WILL open,
## and `_on_phone_entry_screen_visibility(false)` is what starts the sequence
## once it closes.
func _start_or_defer_coach_marks() -> void:
	var bridge_obj: Object = get("bridge")
	var picker_will_open := bridge_obj != null and not bool(bridge_obj.has_world)
	if picker_will_open:
		_coach_marks_wait_for_picker = true
		return
	_show_next_coach_mark(0)

## Called by `phone_project_picker.gd::setup()`, which forwards that dialog's
## own `Window.visibility_changed` here -- see that file's comment on the
## connection for the full reasoning. Must only ever be called for that
## reason, and must never itself decide to SHOW the picker or touch it in any
## other way -- this is a report, not a control.
func _on_phone_entry_screen_visibility(now_visible: bool) -> void:
	if _coach_marks_wait_for_picker and not now_visible:
		_coach_marks_wait_for_picker = false
		_show_next_coach_mark(0)

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
	## `pan_drag_hint` has no anchor by design (see `_COACH_MARKS`'s own doc
	## comment above it) -- `near` stays `null`, which `_position_phone_toast()`
	## already treats as "centre on screen" for exactly this reason.
	var near: Control = null
	if id == "bottombar_tabs":
		near = _phone_menu_bar
	elif id == "sheet_handle":
		near = _phone_sheet_grab
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

## Persists that coach mark `id` has been shown (`coach_marks` section of the user config), so it
## is never shown again. Preserves other config sections.
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
## Built for the map context menu (`context_broker.gd` since CM-1), which on a phone
## is opened by a press-and-hold (`map_overlay.gd`) and cannot use a stock
## popup: pointer-sized rows, and clipping rather than nudging when a finger
## lands near the screen edge.
func phone_present_popup(popup: PopupMenu, title: String, trail: String) -> bool:
	if not _phone or _phone_menu == null:
		return false
	_close_all_phone_overlays()
	_phone_menu.open_sheet(popup, title, trail)
	return true

## Opens the phone's MENU with screen `id` pushed over its root (`PhoneMenu.open_screen()`), after closing every
## other phone overlay, as every phone overlay opens. For a program-menu row whose content is a phone screen: Help ▸
## Gesture reference… (`menus.gd`, MS-4). Does nothing off the phone, where no such row is built.
func phone_open_menu_screen(id: String) -> void:
	if not _phone or _phone_menu == null:
		return
	_close_all_phone_overlays()
	_phone_menu.open_screen(id)

## CM-5 (`MAP_CONTEXT_SCOPE.md` §8.1): the map context menu's OWN phone
## presentation since CM-5 -- peek (chip row) / half (the full sectioned
## card), replacing the plain `PopupMenu` sheet `phone_present_popup()` above
## still serves every other transient popup. Same false-on-desktop-and-
## tablet contract, for the same reason.
##
## `_close_all_phone_overlays()` runs only on a FRESH drop, not on a
## re-resolve (the pin dragged to a new spot): `PhoneMenu.peek_card_is_open()`
## tells the two apart, so a drag's re-resolve updates the same sheet in
## place, at whichever detent it already had, instead of that call tearing it
## down and `peek_card()` reopening it at `peek` under the finger's drag.
func phone_present_peek_card(req: Dictionary, actions: Array, reselect: Callable,
		on_dismiss: Callable = Callable()) -> bool:
	if not _phone or _phone_menu == null:
		return false
	if not _phone_menu.peek_card_is_open():
		_close_all_phone_overlays()
	_phone_menu.peek_card(req, actions, reselect, on_dismiss)
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
	## **Refused while `generating`** (mid-generation sweep, 2026-10-12). Back is a window
	## notification, not an `InputEvent`, so the busy overlay's `_input` never sees it, and every
	## step of the chain below can reach the engine the worker thread holds: hiding a dialog runs
	## its cancel path, `PhoneMenu.go_back()` re-renders a screen that reads counts,
	## `_back_exhausted()` runs the armed tool's escape handler or builds the exit card (which reads
	## the seed). The same stance as the overlay for every other input: nothing moves mid-run, and
	## the status line says why. Nothing to replay: the gesture is repeated after the run.
	var gen_bridge := _find_engine_bridge()
	if gen_bridge != null and gen_bridge.generating:
		set_status("hint", "Not done: Back -- the world is generating. Try again when it finishes.", "warn")
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
	## MS-2 (draft 03): an open domain fan closes first (proposal section 4's order: window, fan, ...).
	if _close_domain_fan_for_back():
		_haptic("back")
		return
	## MS-11: an open quick drawer closes next (proposal section 4's order: window, fan, quick drawer, ...).
	if _close_quick_drawer_for_back():
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

## Opens the phone's right sheet -- its right dock -- from outside the shell.
## `right_dock.gd` calls it when a map tap selects a mark (a landmark ring, an
## icon glyph), whose detail lands in that sheet and would otherwise stay
## closed. A no-op off the phone.
func phone_show_right_sheet() -> void:
	if DccTheme.is_phone():
		_set_sheet_open("right", true)

## Closes the phone's right sheet if it is open; a no-op otherwise and off the phone. The twin of
## `phone_show_right_sheet()`: `DccApp._on_element_tapped` calls it once a tap's action chips are up, because the
## selection that tap made opened this sheet over the map and the chips (children of the map host) sit under it
## (Ruling EO: a tap shows the chips; the sheet is a swipe away). Never touches the left sheet or the menu.
func phone_hide_right_sheet() -> void:
	if DccTheme.is_phone() and _right_sheet_open:
		_set_sheet_open("right", false)

## Shows or hides the left or right dock as a phone sheet. Opening first closes all other phone
## overlays, then resets that dock's scroll to the top.
func _set_sheet_open(side: String, open: bool) -> void:
	if open:
		_close_all_phone_overlays()
	if side == "left":
		## MS-3: hand the dock's body back before the left sheet shows (see `_mount_dock_in_sheet()`).
		if open:
			_mount_dock_in_sheet(false)
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

	## **The menu takes the same rect as a dock sheet**: over the app bar (its
	## own header replaces it, per the canvas's L2/L3 artboards), never over
	## the status safe area, the landscape side safe area, the gesture inset --
	## or the bottom bar. Computed once, here, and shared by both, because a
	## sheet that disagrees with the menu about where the bar's reserve ends is
	## exactly how one of them draws under it.
	##
	## **Never over the bottom bar**, as of the 412 migration. That bar is L1
	## of the disclosure tree, and the canvas's `07 More` screen -- which is
	## what the menu's root is -- carries no close button of its own precisely
	## because it is a *tab destination*: you leave it by tapping another tab, or
	## the lit one. Covering the bar with the screen it opens would make MORE the
	## one tab in the bar that cannot be undone by pressing it, and (with the
	## canvas's two-`✕`-becomes-none change) would leave system back as the only
	## way out at all. **A dock sheet reaches the bar through the identical MORE
	## ▸ Window route and closes the identical way** (its own header's close
	## button, never the bar), so the same argument holds for it -- and until
	## this line, did not: `left_dock`/`right_dock` stopped only at
	## `_safe_bottom()`, the SYSTEM gesture inset, and never reserved the APP's
	## own bar on top of it -- two different numbers from two different
	## sources, and only one of them was here. The sheet's bottom
	## `H_PHONE_BOTTOM_NAV` dp then sat drawn *underneath* the bar rather than
	## stopping above it: measured on a OnePlus handset at 1080x2340, the bar's
	## own opaque band starts at y=2116; this shell's desktop composition (same
	## resolution, `--force-touch`) puts the bar at y=2119 and, before this
	## fix, the open dock sheet's own bottom edge at y=2288 -- 169 px of the
	## sheet sitting behind the bar it never reserved, confirmed by sampling
	## the bar's own "GENERATE" caption pixel before and after opening the
	## sheet: unchanged, so the bar wins the overlap and the sheet's bottom
	## band is the part that goes missing, not the other way round.
	##
	## `_ptap()` rather than the bar's measured `size.y`: this runs before the
	## first layout pass, where that is still zero, and it is the same expression
	## the bar sets its own minimum from.
	##
	## In landscape that same bar is the left rail, so its reserve moves from
	## the bottom inset (both the menu's and, by the same rule, each dock
	## sheet's) to the left one -- `rail_w`, folded into the side reserve below.
	## Getting this wrong would have been invisible in portrait and would have
	## covered the rail with the screen the rail opens.
	##
	## `_phone_menu_bar.size.y` in preference to the constant, once the bar has
	## actually been laid out -- `_phone_bottom_reserve()`'s own established
	## pattern a few hundred lines down, for the same reason: `_ptap()`'s
	## rounding and the VBoxContainer's real accumulated child heights do not
	## always land on the same integer px (measured here: 168 vs the bar's own
	## rendered 169), and the bar's actual rect, not the nominal dp figure, is
	## what a sheet is trying to stop above. Before the first layout pass
	## `size.y` is still zero, so the constant is the correct fallback then.
	var bar_reserve := 0
	if _phone_menu_bar != null and _phone_menu_bar.visible and not _landscape:
		bar_reserve = int(_phone_menu_bar.size.y) if _phone_menu_bar.size.y > 0.0 \
			else _ptap(DccTheme.H_PHONE_BOTTOM_NAV)

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
		## `+ bar_reserve`: see the comment above `bar_reserve` -- the same
		## reserve `apply_insets()` gives the menu a few lines down, given to
		## the dock sheets as well now that the two are meant to agree.
		sheet.offset_bottom = -(_safe_bottom() + bar_reserve)

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
