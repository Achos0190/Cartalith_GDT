extends Workspace
class_name CivilizationWorkspace

## CIVIL domain (§3): settlements, population, economy, politics, culture,
## and (§4.5.3) the Settlement and Territory tools.
##
## The engine backs four of the five browsing categories as *readable* data
## (`get_settlements`, `get_provinces`, `get_trade_balances`). Culture has no
## GDExtension binding at all -- `cartalith-civ` computes culture profiles
## internally (`STATUS.md`'s "generation rules and culture profiles"
## milestone) but nothing exports them.
##
## `civ_tools_bridge.rs` (`UNIFIED_TOOL_PLAN.md` milestone F) now backs
## placing a settlement and painting territory too -- `STRANDED_TOOLS.md`
## rows 10 and 12, closed by the TOOLS block below. POI is the one third of
## §4.5.3's own table that stays unbuilt: `_civDropPOI` has no Rust
## counterpart anywhere in this workspace (`civ_tools_bridge.rs`'s own doc
## comment), so there is no `civ_drop_poi` to arm a button against -- see
## `_build_tools()`'s own comment for where that button would have gone.
##
## The five browsing categories below (Settlements/Population/Economy/
## Politics/Culture) stay read-only -- clicking a settlement or faction row
## pins it into the right dock (`right_dock.gd`'s Settlement/Faction
## contexts) exactly like clicking it on the map does. Only the TOOLS block
## writes to `bridge`.
##
## **Domain merge (2026-08-20, owner instruction: "Infra can be dropped as a
## name and can be absorbed by civil").** This dock now also carries the
## former INFRA domain's five subjects and its Way/Route tools, via `_infra`
## below -- a real `InfrastructureWorkspace` instance, unmodified in what it
## does, appended as a nested `VBoxContainer` after this file's own six
## categories. `_build_tools()` draws ONE combined TOOLS block (Settlement ·
## Territory · Way · Route) rather than two stacked ones; `_infra`'s own
## `_build_tools()` (called from its own `setup()`, `_nested = true`) skips
## drawing its half of that row but still registers the Way/Route click/drag/
## escape handlers, since those don't care which file drew the button. See
## `InfrastructureWorkspace`'s own class doc for the mechanism, and
## `DCC_SHELL_SPEC.md`'s correction notice for the disclosure.

## The engine's six `SettlementKind` tiers, highest first -- the same order
## and vocabulary `journey_bridge::settlement_kind_key` emits and
## `civ_tools_bridge::kind_from_str` accepts. `metropolis` was added
## 2026-08-20 with the port of `_civSelectMetropolises`.
const KIND_ORDER := ["metropolis", "capital", "city", "town", "village", "hamlet"]
const KIND_PLURAL := {
	"metropolis": "metropolises",
	"capital": "capitals", "city": "cities", "town": "towns",
	"village": "villages", "hamlet": "hamlets",
}

## Which of our two tools (if either) is armed -- tracked locally for the
## same reason `infrastructure_workspace.gd`'s own `_active_infra_tool` is:
## `app.tool_armed` fires with the new id already written into `app.armed_
## tool`, so there is no other way to learn what is being armed away FROM.
var _active_civ_tool := ""

## -- Settlement tool state (§4.5.3's own options row). Persists across
## re-arms/rebuilds of the tool options bar so the row always reflects the
## shell's last choice, the same reasoning `cartography_workspace.gd`'s own
## `_icon_*` state vars follow. --
var _settlement_kind := "town"
var _settlement_faction := 1
## Cleared back to "" after every successful placement (`_settlement_click`)
## -- a name left in this field would otherwise get stamped onto every
## settlement dropped afterward, verbatim (`manual_settlement_name` only
## generates one when the given name is blank).
var _settlement_name := ""
var _settlement_snap_water := false

## -- Territory tool state. `_territory_radius`'s default is the reference's
## own `_civTerRadius` initial value (`TERRITORY_BRUSH_RADIUS`, `tools.rs`
## line 102), not an arbitrary number. --
var _territory_faction := 1
var _territory_radius := 5.0
var _territory_subtract := false

## -- Timeline state (`TIMELINE_SCOPE.md` milestone 6). See the Timeline
## section's own header comment, below `_build_culture()`, for why this is a
## sixth `DccWidgets.category()` here rather than a new `right_dock.gd` CTX_*
## context. --
var _tl_body: VBoxContainer
var _tl_add_year := 100                 ## Reference default (`#civTlYear` value="100").
var _tl_playing := false
var _tl_play_timer: Timer
var _tl_sim_mode := "collapse"          ## "collapse" | "recovery"
var _tl_sim_character := "mixed"        ## "mixed" | "trade" | "disease" | "conflict"
var _tl_sim_severity := 0.5             ## fraction [0,1] -- reference slider/100
var _tl_sim_rate := 0.01                ## fraction/yr -- reference slider(tenths-%)/1000
var _tl_sim_start_year := 0
var _tl_sim_duration := 100
var _tl_sim_step_years := 10
var _tl_filter_exist_only := false
var _tl_filter_ghost := false
var _tl_filter_highlight := false
var _tl_sim_out := ""

## The former INFRA domain, nested into this dock -- see this file's own
## class doc and `InfrastructureWorkspace`'s own class doc for the mechanism.
var _infra: InfrastructureWorkspace

## -- Selection, context menu and Delete key (`PARITY_AUDIT.md` §5 items 2,
## 3, 4). The reference keeps one `_civSelectedPlace`; this is the same
## thing as an index into `get_settlements()`, which is the identity every
## `#[func]` in this area keys on. `-1` for nothing selected. --
var _selected_index := -1
## Rebuilt per right click rather than kept: its item list depends on
## whether a place was hit and what it is called, so a cached menu would be
## stale on every open but the first.
var _ctx_menu: PopupMenu
var _ctx_gx := 0.0
var _ctx_gy := 0.0
var _ctx_hit := -1
## `civPopEstimateOut` ("Land sustains ≈ N") and the Settlements roster
## body, both refreshed after any place/roster edit.
var _settlements_body: Control
## The three other data-backed categories' own bodies, held for the same
## reason and cleared/refilled by the same `_rebuild_readouts()`. Culture has
## no body field because it has no data behind it: `_build_culture()` writes
## one fixed note about the missing binding, which a world does not change.
## `_tl_body` (Timeline) is the fifth, declared with the rest of the Timeline
## state above because it also carries playback/simulation state to reset.
var _population_body: Control
var _economy_body: Control
var _politics_body: Control
## SG-02's Recompute row. Held as fields because the handler is a coroutine
## that has to find both again after the blocking engine call, and because a
## GDScript lambda captures locals by value at creation time -- a
## `func(): _recompute_civ(b)` closing over its own button would capture null.
var _recompute_btn: Button
var _recompute_note: Label
## SG-01's badge: what the engine's own stage graph says about `civ` right
## now, above the button that clears it. Polled on the same 1 s clock
## `app.gd`'s status-slot indicator uses, because the states it reports are
## produced in three other workspaces (a sculpt, a carve, a dropped place).
var _recompute_badge: Label
var _stale_timer: Timer


func _build() -> void:
	_infra = InfrastructureWorkspace.new()
	_infra._nested = true

	_build_tools()
	_build_settlements()
	_build_population()
	_build_economy()
	_build_politics()
	_build_culture()
	_build_timeline()

	## Appended last, after CIVIL's own six categories -- `_infra.setup()`
	## calls its own `_build()`, which adds its five categories (Roads/
	## Rivers/Ports/Trade/Logistics) as children of `_infra` itself, and
	## registers the Way/Route handlers `_build_tools()` above already drew
	## buttons for. One rule marks the seam so the merge reads as two grouped
	## subjects, not one undifferentiated list.
	add_child(DccTheme.rule())
	add_child(_infra)
	_infra.setup(app, bridge)

	## Both windows are owned by `app` (long-lived, opened from four places
	## between them); this workspace is the one that knows how to put the
	## map back in sync afterwards, so it owns the connections rather than
	## either window reaching into `viewport` itself.
	app.place_editor_window.place_changed.connect(_on_civ_edited)
	app.place_editor_window.place_deleted.connect(_on_civ_edited)
	app.faction_roster_window.roster_changed.connect(_on_roster_changed)

	## `GUI_GAP_REGISTER.md` RF-01. Everything above ran ONCE, at launch, from
	## `app.gd`'s `_register_workspaces` -- before any world exists -- so every
	## category drew its "generate a world first" empty state against an empty
	## engine. Nothing then re-ran it: `app.gd`'s own `generation_finished`
	## handler only writes status-bar text, and the only subscriber this file
	## had was Timeline's (below, now folded into `_on_world_changed`). The
	## result was a dock that stayed permanently empty over a world the map
	## was already drawing -- found live against 40 settlements, 6 factions and
	## a full road network. `_infra` connects its own five categories to the
	## same two signals, in its own `_build()`.
	bridge.generation_finished.connect(func(ok: bool): if ok: _on_world_changed())
	bridge.world_loaded.connect(_on_world_changed)

## A fresh generate or a loaded save replaces every settlement, province,
## trade balance and recorded year this dock reads -- and invalidates the
## selection, which is an index into a settlement list that no longer exists.
func _on_world_changed() -> void:
	_selected_index = -1
	_rebuild_readouts()
	_tl_on_world_changed()

## A place edit or delete moved map data: repaint the pins, refresh the
## selection state that may now point at a different (or no) settlement, and
## rebuild the dock's own rosters/readouts.
func _on_civ_edited() -> void:
	if _selected_index >= bridge.settlements().size():
		_selected_index = -1
	_refresh_civ_data()
	_rebuild_readouts()

func _on_roster_changed() -> void:
	## Removing a faction reverts its settlements AND its territory cells to
	## Unclaimed, so the territory raster is stale too -- same direct write
	## `_commit_territory` uses, without `refresh()`'s camera reset.
	app.viewport.territory_view.texture = bridge.territory_texture()
	_on_civ_edited()

## Rebuilds every category whose content depends on world data, scoped the way
## `_rebuild_timeline` already scopes its own -- one held body node per
## category, cleared and refilled, so the accordion around them (`categories`,
## which holds these same body nodes) is untouched and whichever L2 the user
## has open stays open.
##
## Called from two places, and it took a live 40-settlement world to notice
## only one of them existed: a place/roster edit (`_on_civ_edited`, always
## did) and a fresh generate or loaded save (`_on_world_changed`, RF-01 --
## never did). The old scoping comment here claimed Population and Economy
## "read nothing this touches", which was wrong on both counts: Population
## sums `get_settlements()`, and a Recompute (SG-02, which routes through
## `_on_civ_edited` too) rewrites the trade balances Economy reads and the
## provinces Politics reads.
##
## Cheap on purpose, and checked rather than assumed: every call these four
## fills make is a pure read of already-computed state -- `get_settlements`/
## `get_provinces`/`get_trade_balances`/`get_factions` copy stored `Vec`s of a
## few dozen entries, and the one O(grid) call in the set
## (`civ_agrarian_regional_total`) is a single linear pass over the stored
## `civ.dens`/`ws.field` that recomputes neither. This is a *presentation*
## rebuild, not the civ *recompute* the staleness work deliberately refused to
## cascade after every stroke -- that one is seconds per press and stays
## behind its own button.
func _rebuild_readouts() -> void:
	if _settlements_body != null and is_instance_valid(_settlements_body):
		_clear_body(_settlements_body)
		_fill_settlements(_settlements_body)
	if _population_body != null and is_instance_valid(_population_body):
		_clear_body(_population_body)
		_fill_population(_population_body)
	if _economy_body != null and is_instance_valid(_economy_body):
		_clear_body(_economy_body)
		_fill_economy(_economy_body)
	if _politics_body != null and is_instance_valid(_politics_body):
		_clear_body(_politics_body)
		_fill_politics(_politics_body)

## `remove_child` before `queue_free` on purpose: `queue_free` defers to the
## end of the frame, so a child left parented is still in `get_children()`
## while the refill runs and would draw twice for one frame. Same teardown
## `_rebuild_timeline` performs inline, factored out here because four
## categories now need it.
static func _clear_body(node: Control) -> void:
	for c in node.get_children():
		node.remove_child(c)
		c.queue_free()

# -- Selection, right-click menu, Delete key ---------------------------------

## `app.gd`'s `_wire_selection` forwards every map selection here (this
## workspace is in `_workspaces`). The reference's `_civSelectedPlace` is
## what Delete and the place editor both act on; this is that.
func on_settlement_selected(_data: Variant, index: int) -> void:
	_selected_index = index

## `_civCtxShow` (reference 25857) and the `contextmenu` handler that builds
## its item list (25888). Five of the reference's six operations; the sixth
## ("Drop POI here") is absent because POI is not a ported concept
## (`GUI_GAP_REGISTER.md` CV-01, `civ_tools_bridge.rs`'s own module doc) --
## omitted rather than shown disabled, matching how this file's own
## `_build_tools()` already treats the POI tool.
##
## The reference gates its menu on "a civ-capable tab is open"; the
## equivalent gate here is this workspace being the active domain, which
## `app.gd`'s broadcast does not check -- so it is checked here.
func on_map_right_clicked(gx: float, gy: float, hit: int, screen_pos: Vector2) -> void:
	if app.active_domain() != "civilization":
		return
	if not bridge.has_world:
		return
	_ctx_gx = gx
	_ctx_gy = gy
	_ctx_hit = hit
	if _ctx_menu == null:
		_ctx_menu = PopupMenu.new()
		_ctx_menu.id_pressed.connect(_on_ctx_id)
		add_child(_ctx_menu)
	_ctx_menu.clear()
	if hit >= 0:
		var s: Dictionary = bridge.settlements()[hit]
		var nm := String(s.get("name", "(unnamed)"))
		_ctx_menu.add_item("Edit %s" % nm, 0)
		_ctx_menu.add_item("Move viewer to %s" % nm, 1)
		_ctx_menu.add_item("Delete %s" % nm, 2)
		_ctx_menu.add_separator()
	_ctx_menu.add_item("Drop settlement here", 3)
	_ctx_menu.add_separator()
	_ctx_menu.add_item("Info here (settlement & ecology)", 4)
	## Phone: the very same menu, re-presented as the canvas's L4 sheet
	## (`phone_menu.gd`'s `open_sheet`), because a finger has no second button
	## and a pointer-sized popup at a fingertip is both unreadable and clipped
	## at a screen edge. `map_overlay.gd` turns the press-and-hold into the
	## `map_right_clicked` that got us here, so nothing above this line differs
	## between the two pointers -- one menu definition, two presentations.
	## Returns false on desktop and tablet, where the stock popup below runs
	## exactly as it always has.
	var ctx_title := "Here"
	if hit >= 0:
		var picked: Dictionary = bridge.settlements()[hit]
		ctx_title = String(picked.get("name", "Place"))
	if app.phone_present_popup(_ctx_menu, ctx_title,
			"Map · cell %d, %d" % [int(gx), int(gy)]):
		return
	## `screen_pos` is `map_overlay`'s own local space; a `PopupMenu` pops in
	## screen space, which is what this conversion is for.
	_ctx_menu.position = Vector2i(app.viewport.overlay.get_screen_position() + screen_pos)
	_ctx_menu.reset_size()
	_ctx_menu.popup()

func _on_ctx_id(id: int) -> void:
	match id:
		0:
			_selected_index = _ctx_hit
			app.open_place_editor(_ctx_hit)
		1:
			var s: Dictionary = bridge.settlements()[_ctx_hit]
			app.viewport.move_view_to(float(int(s.get("x", 0))), float(int(s.get("y", 0))))
		2:
			app.place_editor_window.confirm_delete(_ctx_hit)
		3:
			## The reference's own "⌂ Drop settlement here" calls
			## `_civDropPlace` with the menu's own cell, using whatever class
			## and faction the Settlement tool is currently set to -- exactly
			## what `_settlement_click` already does, so it is reused rather
			## than duplicated.
			_settlement_click(_ctx_gx, _ctx_gy)
		4:
			## `_civInfoAt`: the reference opens its own info readout. This
			## shell's equivalent is the Sample dock, which is already fed by
			## `cursor_sampled` -- so the honest action is to select whatever
			## is here and say where to read it, not to build a second panel.
			if _ctx_hit >= 0:
				app.right_dock_ctrl.on_settlement_selected(bridge.settlements()[_ctx_hit], _ctx_hit)
				app.set_status("hint", "Pinned in the right dock — the reference's _civInfoAt readout.", "text_ghost")
			else:
				app.set_status("hint",
					"Nothing here. _civInfoAt's ecology half (biome/wildlife at a cell) has no binding — see the Layers popover's own debug views.",
					"text_ghost")

## `PARITY_AUDIT.md` §5 item 4 / reference block 2's keydown at line 26096:
## Delete removes the selected place. Returns `true` when it handled the key,
## so `app.gd`'s broadcast stops at the first workspace that did.
func on_delete_key() -> bool:
	if app.active_domain() != "civilization":
		return false
	if _selected_index < 0 or _selected_index >= bridge.settlements().size():
		return false
	app.place_editor_window.confirm_delete(_selected_index)
	return true

# -- Tools (§4.5.3: Settlement, Territory -- and, since the 2026-08-20 merge,
# §4.5.4: Way, Route) -----------------------------------------------------

## §4.5's TOOLS block: the three global tools (`GlobalTools.install`) plus
## this domain's own four, built first (§4.5: "every left dock opens with a
## TOOLS block"). One combined row rather than two stacked ones -- Settlement
## and Territory are this file's own; Way and Route belong to `_infra`
## (`InfrastructureWorkspace`, composed in via `_build()` above since the
## 2026-08-20 domain merge) and are registered by its own `_build_tools()`
## when `_infra.setup()` runs, `_nested = true` so it draws no second row.
##
## POI is not a fifth button here: §4.5.3's own table lists it ("Click drops
## a point of interest, `_civDropPOI`"), but `civ_tools_bridge.rs`'s own
## module doc says outright that POI "is not a ported concept" -- no Rust
## function anywhere in this workspace drops one, so there is nothing an
## armed POI tool could call. Arming a button with no engine behind it would
## be the fake control this port's own discipline (`DECISIONS.md`) exists to
## avoid, so it is omitted rather than built disabled or wired to a stub.
func _build_tools() -> void:
	DccWidgets.tools_block(self, app, app.tool_group, [
		{"id": "settlement", "glyph": "tool_settlement", "label": "Settlement (S)"},
		{"id": "territory", "glyph": "tool_territory", "label": "Territory (T)"},
		{"id": "way", "glyph": "tool_way", "label": "Way (W)"},
		{"id": "route", "glyph": "tool_route", "label": "Route (⇧R)"},
	])
	app.register_tool_click_handler("settlement", func(gx, gy): _settlement_click(gx, gy))
	app.register_tool_drag_handler("territory", func(gx, gy): _territory_drag(gx, gy))
	app.tool_armed.connect(_on_civ_tool_armed)

## Reacts to ANY tool arming anywhere in the app (`app.tool_armed` is one
## shared signal across every domain), not just ours -- see `_active_civ_
## tool`'s own doc for why a local flag, not `app.armed_tool`, is what tells
## us whether *we* were the one just disarmed.
##
## Deliberately does NOT auto-commit an in-progress Territory stroke on
## switching away, unlike `infrastructure_workspace.gd`'s Way/Route (whose
## own `_on_infra_tool_armed` commits on switch). Territory's engine state
## (`civ_tools_bridge::CivTools`) is modelled on Paint's `PassBuffer`/
## `PaintLayer` draft, not Way/Route's click-chain (that module's own doc
## comment: "Milestone C's own precedent -- territory paint is PaintStamp/
## PaintLayer, unchanged"), and Paint's own precedent
## (`world_workspace.gd`'s `_on_tool_armed`) leaves its draft pending across
## a tool switch too, trusting the explicit Commit/Discard buttons rather
## than silently baking in whatever was mid-stroke when the user clicked
## away. A stray auto-commit of a half-finished territory claim would be a
## worse surprise than a draft that just waits.
func _on_civ_tool_armed(id: String) -> void:
	match id:
		"settlement":
			_active_civ_tool = "settlement"
			_tool_options_settlement()
		"territory":
			_active_civ_tool = "territory"
			_tool_options_territory()
		_:
			if _active_civ_tool != "":
				_active_civ_tool = ""
				## Matches `infrastructure_workspace.gd`'s own reasoning:
				## only reclaim the options bar for the plain "back to
				## Inspect" case -- Measure/Region set no options-bar content
				## of their own, so guessing at something to show while one
				## of those is armed would just be a different flavour of
				## stale.
				if id == "inspect":
					_tool_options_civ_idle()

## §10's brush ring, wired from `on_cursor_sampled` per the tool-arming
## substrate's own instructions (`app.gd`'s `_wire_selection` forwards every
## viewport cursor sample to any workspace that implements this method).
##
## Deliberately has no `else: hide` branch. `world_workspace.gd`'s own
## `on_cursor_sampled` (registered first -- `app.gd`'s `_register_
## workspaces` builds `["world", "civilization", ...]` in that order, and
## `_wire_selection`'s forwarding loop runs in the same order) already hides
## the brush ring for every tool that isn't sculpt/paint, Territory included
## -- so this only ever needs to say when to SHOW it; the "otherwise hide"
## half is already handled upstream. Adding a redundant `else` here would
## instead race it: this file's handler runs AFTER world's in that same
## loop, so an unconditional hide here would win the frame and blank the
## sculpt/paint ring the instant this file is present at all.
func on_cursor_sampled(gx: float, gy: float, valid: bool) -> void:
	if app.armed_tool == "territory":
		app.viewport.tool_overlay.set_brush_cursor(valid, gx, gy, _territory_radius)

func _tool_options_label(row: HBoxContainer, text: String) -> void:
	row.add_child(DccTheme.mono_label(text, "accent", DccTheme.FS_SMALL, 2, true))

## Reclaims the options bar once Settlement/Territory hands it back to plain
## Inspect. Same in-session-only caveat `infrastructure_workspace.gd`'s own
## twin carries: `app.gd`'s domain-switch default is stale the moment this
## file ships, but `app.gd` isn't ours to touch this pass.
func _tool_options_civ_idle() -> void:
	app.set_tool_options(func(row: HBoxContainer):
		_tool_options_label(row, "CIVIL · INSPECT")
		row.add_child(DccTheme.label("Settlement and Territory tools are armed from the TOOLS block above.", "text_ghost", DccTheme.FS_MICRO))
		row.add_child(DccTheme.spacer())
	)

## Shared by Settlement's and Territory's own options rows -- both need "pick
## a faction id", and `bridge.get_factions()` (`CIV_FACTION_COUNT` real ids,
## 1-based) is the one live source for what a valid faction actually is; a
## bare number spinbox would let either tool arm against an id the engine
## has never heard of. Renders a plain note instead of the dropdown before
## any world exists (`get_factions()` is empty then -- `lib.rs`'s own guard,
## no `civ` to read faction ids from yet).
func _faction_choice(row: HBoxContainer, current: int, on_change: Callable) -> void:
	var factions := bridge.get_factions()
	if factions.is_empty():
		row.add_child(DccTheme.label("no world generated -- factions unknown", "text_ghost", DccTheme.FS_MICRO))
		return
	var ids: Array = []
	var labels: Array = []
	for f in factions:
		var d: Dictionary = f
		var fid := int(d.get("id", 1))
		ids.append(fid)
		labels.append("%d · %s" % [fid, String(d.get("culture", "?")).capitalize()])
	DccWidgets.choice(row, "Faction", labels, maxi(0, ids.find(current)),
		func(i: int): on_change.call(ids[i]))

## `DccWidgets` has no bare-text-field row builder (`choice`/`slider`/
## `toggle`/`number`/`action` cover every other row shape this shell uses,
## per that file's own L4 disclosure-grammar comment) -- built by hand here
## rather than adding a sixth for this one call site, matching its private
## `_row()`'s own label+control layout (`DccWidgets.ROW_LABEL_W`) so it reads
## as one more row in the bar, not a visual outlier. `text_changed` updates
## the state var in place with no row rebuild, unlike a family/mode change
## elsewhere in this file -- rebuilding on every keystroke would steal focus
## out of the field mid-word.
func _settlement_name_field(row: HBoxContainer) -> void:
	var mini := HBoxContainer.new()
	mini.add_theme_constant_override("separation", 8)
	mini.custom_minimum_size.y = 24
	var l := DccTheme.mono_label("Name", "text_dim", DccTheme.FS_SMALL, 0)
	l.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
	mini.add_child(l)
	var edit := LineEdit.new()
	edit.text = _settlement_name
	edit.placeholder_text = "(blank = generated)"
	edit.custom_minimum_size.x = 130
	edit.text_changed.connect(func(t: String): _settlement_name = t)
	mini.add_child(edit)
	row.add_child(mini)

## §4.5.3's Settlement options row: `CIVIL · SETTLEMENT` · class · faction ·
## name · snap to water. §4.5.3's own class list ("metropolis / city / town /
## village / hamlet") used to be one tier wider than the engine modelled;
## since `_civSelectMetropolises` was ported (2026-08-20) the two agree
## exactly, and `civ_tools_bridge::kind_from_str` accepts all six tiers.
## This dropdown reuses `KIND_ORDER` (this file's own highest-first tier
## order, `_build_settlements()` above) and so offers every one of them.
##
## No pick-radius control: `civ_drop_settlement` (`lib.rs`) computes its own
## pick radius internally (`cartalith_civ::tools::civ_place_pick_radius(gw)`)
## and takes no radius argument at all, so a slider here would be decoration
## with nothing behind it -- the same kind of stale-spec-vs-engine gap this
## task's own brief names for Sculpt, treated the same way.
func _tool_options_settlement() -> void:
	app.set_tool_options(func(row: HBoxContainer):
		_tool_options_label(row, "CIVIL · SETTLEMENT")
		DccWidgets.choice(row, "Class", KIND_ORDER.map(func(k): return String(k).capitalize()),
			KIND_ORDER.find(_settlement_kind), func(i: int): _settlement_kind = KIND_ORDER[i])
		_faction_choice(row, _settlement_faction, func(fid: int): _settlement_faction = fid)
		_settlement_name_field(row)
		DccWidgets.toggle(row, "Snap to water", _settlement_snap_water,
			func(v: bool): _settlement_snap_water = v)
		row.add_child(DccTheme.spacer())
	)

## One click, one drop (`DCC_SHELL_SPEC.md` §4.5.3: "Click drops a place").
func _settlement_click(gx: float, gy: float) -> void:
	var idx := bridge.civ_drop_settlement(gx, gy, _settlement_kind, _settlement_faction, _settlement_name, _settlement_snap_water)
	if idx < 0:
		app.set_status("hint", "Settlement refused -- out of bounds, or water without Snap to water.", "accent")
		return
	_settlement_name = ""
	_refresh_civ_data()
	## §4.5.3's own right-dock column: "The new settlement's inspector, live,
	## focused on the name field." `right_dock.gd`'s Settlement context
	## (`_build_settlement`) renders every field, name included, as a plain
	## read-only Label, not a LineEdit -- there is no name field to focus,
	## and `right_dock.gd` is explicitly not this pass's to change. The
	## closest honest equivalent is selecting the new settlement the same
	## way clicking one on the map already does (`map_overlay.gd`'s own
	## `settlement_selected` -> `right_dock.gd`'s `on_settlement_selected`).
	var settlements := bridge.settlements()
	if idx < settlements.size():
		## Also the selection Delete and the place editor act on -- this path
		## pins the right dock directly rather than through `map_overlay.gd`'s
		## `settlement_selected`, so `on_settlement_selected` above never fires
		## for it and `_selected_index` would otherwise stay stale.
		_selected_index = idx
		app.right_dock_ctrl.on_settlement_selected(settlements[idx], idx)
	## §4.5.6: "A tool that writes world data ... reports its staleness
	## consequence in the status bar the moment it commits." Not `bridge.
	## mark_dirty()` -- that flags the GENERATION PARAMETERS stale, prompting
	## a full regenerate, which would rebuild `civ.settlements` from scratch
	## and silently discard this manual drop. The real consequence is
	## narrower: provinces/trade balances/roads were computed before this
	## edit and won't reflect it until the civ layer is rebuilt -- which as
	## of SG-02 is a button in this dock (Settlements ▸ Recompute), not a
	## full regenerate, so the hint names it.
	app.set_status("hint",
		"Settlement placed -- provinces/trade/roads still predate it. Settlements ▸ Recompute civilisation catches them up.",
		"text_ghost")
	if _active_civ_tool == "settlement":
		_tool_options_settlement()

## Repopulates `map_overlay.gd`'s settlement layer without the full
## `ViewportHost.refresh()` this same data normally goes through after a
## generate -- that call also runs `reset_view()` (recentres/rezooms the
## camera), which would be actively hostile here and worse for Territory
## (`_commit_territory` below calls this too, and a camera snap on every
## paint commit would be disorienting mid-session). `viewport.overlay` is a
## public field (`ViewportHost.overlay: Control`) for exactly this -- the
## same call `refresh()` itself makes for civ data, just without the camera
## reset wrapped around it.
func _refresh_civ_data() -> void:
	var g := bridge.grid_size()
	app.viewport.overlay.set_civ_data(_tl_apply_filters(bridge.settlements()), bridge.roads(),
		bridge.sea_routes(), g.x, g.y, bridge.border_inset_frac())

## Timeline "Exist only" filter (`_build_timeline_filters` below): keeps only
## settlements whose `tid` (`lib.rs`'s `get_settlements()`, now real -- see
## this file's own top-of-file Timeline comment, corrected below) is in the
## active year's `civ_year_diff().present` set, so unchecking the box
## actually removes pins from `map_overlay.gd`'s draw call -- filtering
## upstream of `set_civ_data`, not inside that file (out of scope for this
## pass, `CLAUDE.md`'s territory note). Gated on a non-empty timeline: with
## no recorded years, `civ_year_diff()` has nothing to diff and reports an
## empty `present` set, which would hide every settlement rather than leave
## the (undefined, timeline-less) view alone. "Ghost removed"/"Highlight
## new" stay display-only -- both need per-pin visual state (fade / halo)
## that only `map_overlay.gd`'s own `_draw()` can render, and "removed"
## additionally needs the OLD snapshot's settlement data (position/name),
## which no `#[func]` exposes (`civ_year_diff()` returns tid sets only) --
## real, disclosed remaining gaps, not silently faked.
func _tl_apply_filters(settlements: Array) -> Array:
	if not _tl_filter_exist_only:
		return settlements
	if _tl_years().is_empty():
		return settlements
	var diff: Dictionary = bridge.civ_year_diff(bridge.get_civ_year())
	var present: PackedInt64Array = diff.get("present", PackedInt64Array())
	if present.is_empty():
		return settlements
	var present_set := {}
	for t in present:
		present_set[int(t)] = true
	return settlements.filter(func(s): return present_set.has(int(s.get("tid", 0))))

## §4.5.3's Territory options row: `CIVIL · TERRITORY` · faction · radius ·
## add/subtract · a live stats readout · Commit/Discard. No "respect
## coastlines" toggle: `civ_territory_paint_at` (`civ_tools_bridge::CivTools
## ::paint_at`) always pushes an ungated circular dab (`PaintStamp::
## ungated`) with no coastline mask behind it, so there is nothing to wire a
## toggle to -- the same disclosed-gap treatment this file already gives
## Settlement's missing pick-radius control above.
##
## The stats readout (`civ_faction_territory_stats`) reads the COMMITTED
## `civ.territory`, not the in-progress draft (`CivTools::paint_at` only
## touches `territory_draft`, baked in on `commit()` -- that module's own
## doc comment) -- so like Paint's own painted-cell counts (`world_workspace
## .gd`'s `_build_paint`), this is only "live" per commit, not per dab. Same
## reason there is no live preview texture here either: nothing in `civ_
## tools_bridge.rs`/`lib.rs` builds one for Territory (checked against every
## `civ_*` `#[func]` in `lib.rs` -- `build_sculpt_preview_texture`/`build_
## paint_preview_texture` exist, no Territory equivalent does).
func _tool_options_territory() -> void:
	app.set_tool_options(func(row: HBoxContainer):
		_tool_options_label(row, "CIVIL · TERRITORY")
		_faction_choice(row, _territory_faction, func(fid: int): _territory_faction = fid)
		DccWidgets.slider(row, "Radius", 1.0, 20.0, 1.0, _territory_radius, " c",
			func(v: float): _territory_radius = v)
		DccWidgets.choice(row, "Mode", ["Add", "Subtract (⇧)"], 1 if _territory_subtract else 0,
			func(i: int): _territory_subtract = (i == 1))
		var stats := bridge.civ_faction_territory_stats(_territory_faction)
		if not stats.is_empty():
			row.add_child(DccTheme.mono_label(
				"%d cells · %.0f km² · %d contested" % [
					int(stats.get("claimed_cells", 0)), float(stats.get("area_km2", 0.0)),
					int(stats.get("contested_cells", 0))],
				"text_dim", DccTheme.FS_MICRO))
		row.add_child(DccTheme.spacer())
		DccWidgets.action(row, "✓ Commit", _commit_territory, true)
		DccWidgets.action(row, "Discard", _discard_territory)
	)

## Paints on every drag sample (§4.5.3: "Drag paints the armed faction's
## claim"). ⇧ is a momentary modifier ORed with the Mode choice's own
## Subtract state, not a replacement for it -- matching Paint's own "⇧
## erases" precedent (`world_workspace.gd`'s `_paint_apply_dab`).
func _territory_drag(gx: float, gy: float) -> void:
	var subtract := _territory_subtract or Input.is_key_pressed(KEY_SHIFT)
	bridge.civ_territory_paint_at(gx, gy, _territory_faction, _territory_radius, subtract)

func _commit_territory() -> void:
	bridge.civ_territory_commit()
	## Same reasoning as `_refresh_civ_data()`'s own doc comment: the direct
	## field write `ViewportHost.refresh()` itself would make for the
	## territory texture, without that call's camera-resetting side effects.
	app.viewport.territory_view.texture = bridge.territory_texture()
	## §4.5.3's own right-dock column: "Faction inspector with live area,
	## claimed-cell count, and contested-cell warning." `right_dock.gd`'s
	## Faction context (`_build_faction`) predates `civ_faction_territory_
	## stats` and still says outright "no per-faction cell count or area
	## query exists" -- true when that sentence was written, no longer true,
	## but `right_dock.gd` is explicitly not this pass's to change. Pinning
	## the faction is still the closest honest equivalent (same "select what
	## was just edited" pattern `_settlement_click` above uses); the live
	## cells/area/contested numbers §4.5.3 actually wants live in THIS file's
	## own tool options row instead (above), which can read the real binding.
	app.right_dock_ctrl.show_faction(_territory_faction)
	app.set_status("hint",
		"Territory committed -- provinces/trade were computed before this edit.", "text_ghost")
	if _active_civ_tool == "territory":
		_tool_options_territory()

func _discard_territory() -> void:
	bridge.civ_territory_discard()
	if _active_civ_tool == "territory":
		_tool_options_territory()


# -- Settlements --------------------------------------------------------

func _build_settlements() -> void:
	var cat := DccWidgets.category(self, "Settlements", categories, true)
	## Outside `_settlements_body` on purpose: this section is not a readout
	## of the roster, and `_rebuild_readouts()` -- which the recompute itself
	## triggers -- would otherwise free the label the result was just written
	## to. Being outside also means it exists before a world does, which the
	## roster inside does not (this dock is built once at startup and only
	## refills on a place edit).
	_build_recompute(cat)
	## Everything a place edit or delete invalidates lives under this one
	## node, so `_rebuild_readouts()` can tear down exactly that much --
	## same scoping discipline `_rebuild_timeline` already uses for `_tl_body`.
	_settlements_body = VBoxContainer.new()
	_settlements_body.add_theme_constant_override("separation", 4)
	_settlements_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cat.add_child(_settlements_body)
	_fill_settlements(_settlements_body)

func _fill_settlements(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Roster")
	var settlements := bridge.settlements()
	if settlements.is_empty():
		DccWidgets.note(sec, "No settlements -- generate a world first (World ▸ Generation Pipeline).")
		_build_settlement_gaps(parent)
		return

	var counts := {}
	for s in settlements:
		var kind := String((s as Dictionary).get("kind", "?"))
		counts[kind] = int(counts.get(kind, 0)) + 1
	var parts: Array[String] = []
	for kind in KIND_ORDER:
		if counts.has(kind):
			parts.append("%d %s" % [counts[kind], kind if int(counts[kind]) == 1 else KIND_PLURAL[kind]])
	DccWidgets.note(sec, "%d settlements -- %s." % [settlements.size(), ", ".join(parts)])
	_build_pop_estimate(sec)

	var by_pop := DccWidgets.group(sec, "Largest, by population")
	var ranked: Array = []
	for i in range(settlements.size()):
		ranked.append({"index": i, "data": settlements[i]})
	ranked.sort_custom(func(a, b): return int(a.data.population) > int(b.data.population))
	for i in range(mini(8, ranked.size())):
		_settlement_row(by_pop, ranked[i].data, ranked[i].index)

	_build_settlement_gaps(parent)

## `GUI_GAP_REGISTER.md` SG-02's "Recompute now", and the recompute ED-03d
## says a place edit never triggered.
##
## Lives here rather than in a menu because this is the dock that shows what
## goes stale: the roster above it, the Economy and Politics categories below
## it, and the territory/roads the map draws are all products of the one call
## this button makes. A menu item would have been further from every readout
## it fixes.
##
## Still deliberately always enabled, and now for a *narrower* reason than
## before. The old note here said a greyed-out button would be reporting a
## state the user cannot see, and pointed at SG-01. SG-01 is built (the badge
## below, and the shell's `stale` status slot), so that objection is gone --
## but the button must stay pressable anyway, because "stale" is not the only
## thing it is for: a recompute is also how a user re-derives roads and
## borders after an edit the engine cannot classify, and pressing it with
## nothing stale is a real recompute of the same answer, not an error. What
## the badge changes is that the user now knows *in advance* whether it will
## do anything, which is what greying it out was only a proxy for.
func _build_recompute(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Recompute")
	_recompute_badge = DccWidgets.note(sec, "")
	_refresh_staleness()
	_recompute_btn = DccWidgets.action(sec, "Recompute civilisation", _recompute_civ)
	var b := _recompute_btn
	b.tooltip_text = ("Re-derives everything downstream of the settlement list against the current "
		+ "terrain: roads, sea lanes, territory, provinces, trade balances and the suitability "
		+ "explanations. The settlements themselves are kept exactly as they are -- including "
		+ "hand-dropped and hand-edited ones, their traits and history, the faction roster and "
		+ "hand-painted territory. It does NOT re-place settlements; sculpt a mountain under a "
		+ "city and the city stays put. Re-placing from terrain is what Generate does.\n\n"
		+ "Seconds, not milliseconds, and it runs on the main thread -- the window will hold "
		+ "still. Measured in a release build: about 1.0 s at 512², 1.6 s at 1024² and 4.2 s at "
		+ "2048², roughly half the cost of a full Generate of the same world. That is why it is "
		+ "a button and not an automatic cascade after every brush stroke.")
	_recompute_note = DccWidgets.note(sec,
		"A terrain or place edit leaves the civ layer stale on purpose -- press this when you "
		+ "want roads, territory, provinces and economy to catch up with it.")
	_stale_timer = Timer.new()
	_stale_timer.name = "CivStalenessPoll"
	_stale_timer.wait_time = 1.0
	_stale_timer.timeout.connect(_refresh_staleness)
	sec.add_child(_stale_timer)
	_stale_timer.start()

## SG-01, per-stage rather than per-shell: `civ`'s own entry out of
## `stale_stages()`, said in the vocabulary of the button below it.
##
## Three states, all read from the engine and none of them inferred here: no
## world at all; `civ` absent from the reply, which is genuinely up to date;
## or present, with the graph's own most-upstream reason ("sculpt",
## "carve_fjords", "param:climate.rain_k") or the settlements flag
## ("place_edited") that the graph structurally cannot carry.
func _refresh_staleness() -> void:
	if _recompute_badge == null or not is_instance_valid(_recompute_badge):
		return
	if not bridge.has_world:
		_recompute_badge.text = "No world yet."
		return
	var civ: Dictionary = bridge.stale_stages().get("civ", {})
	if civ.is_empty():
		_recompute_badge.text = "Up to date -- nothing has changed under it since the last recompute."
		return
	var reason := String(civ.get("reason", ""))
	if reason.is_empty():
		reason = String(civ.get("origin", "an edit"))
	var scope := ""
	if int(civ.get("tiles", 0)) > 0:
		scope = " over %d tiles" % int(civ.get("tiles", 0))
	_recompute_badge.text = "Stale%s -- %s. Recompute to catch it up." % [scope, reason]

## The button's own progress affordance. `recompute_civilisation` is a
## synchronous engine call with no progress signal to subscribe to, so the
## honest minimum is to say so before blocking: relabel, disable, let two
## frames actually paint that, then run. Two frames rather than one because a
## single `process_frame` await returns before the redraw has reached the
## screen on the frame the label changed.
func _recompute_civ() -> void:
	var b := _recompute_btn
	if b != null and is_instance_valid(b):
		b.text = "Recomputing…"
		b.disabled = true
		await get_tree().process_frame
		await get_tree().process_frame
	var r: Dictionary = bridge.civ_recompute()
	if b != null and is_instance_valid(b):
		b.disabled = false
		b.text = "Recompute civilisation"
	if _recompute_note != null and is_instance_valid(_recompute_note):
		if not bool(r.get("ok", false)):
			_recompute_note.text = "Not recomputed. %s" % String(r.get("reason", "Unknown reason."))
		else:
			_recompute_note.text = ("Recomputed in %.1f s: %d settlements kept, %d ways and %d "
				+ "provinces rebuilt against the current terrain.") % [
				float(r.get("ms", 0.0)) / 1000.0, int(r.get("settlements", 0)),
				int(r.get("ways", 0)), int(r.get("provinces", 0))]
	## Territory and the pins/roads overlay both moved. Same direct writes
	## `_commit_territory` and `_refresh_civ_data` use, for the same reason
	## (no camera reset), and `_on_civ_edited` rebuilds the roster readouts --
	## which is also what puts `_recompute_note` on screen.
	app.viewport.territory_view.texture = bridge.territory_texture()
	_on_civ_edited()
	## Both SG-01 readouts, immediately rather than up to a second later --
	## the button that just cleared the state is the one place a lagging
	## badge would read as "it didn't work".
	_refresh_staleness()
	if app.has_method("refresh_staleness"):
		app.refresh_staleness()

## `civPopEstimateOut` / `_civAgrarianRegionalTotal` (reference 23516) --
## `PARITY_AUDIT.md` §5 item 7, "the only world-level population sanity
## figure the reference shows", which had no Rust function at all until this
## pass ported one.
##
## Shown beside the settled total on purpose: the number is only useful as a
## comparison, which is exactly how the reference's own readout row uses it.
func _build_pop_estimate(parent: Control) -> void:
	var agr := bridge.civ_agrarian_regional_total()
	if agr.is_empty():
		return
	var sustains := int(agr.get("sustains", 0))
	var settled := int(agr.get("settled", 0))
	var land := int(agr.get("land_km2", 0))
	var pct := (100.0 * float(settled) / float(sustains)) if sustains > 0 else 0.0
	DccWidgets.note(parent,
		"Land sustains ≈ %s people over %s km² of land; %s actually live in settlements (%.1f%%). "
		% [FactionRosterWindow._thousands(sustains), FactionRosterWindow._thousands(land),
			FactionRosterWindow._thousands(settled), pct]
		+ "Σ agrarian density × cell area over land -- the ceiling settlement nuclei are sized "
		+ "against, not a target.")


## The reference's own settlement-population operations, which this shell has
## no equivalent of because the split is different, not because they were
## forgotten: `generate()` *places* the world as part of the one-shot chain
## (`compute_civilisation`), so there is no separate "populate now" step to
## press and nothing that clears just the civ layer.
##
## Corrected 2026-08-24 (SG-02): the half of that sentence which used to read
## "without re-running the whole pipeline" is no longer true. `Recompute
## civilisation` above re-derives the whole civ layer downstream of the
## settlement list without re-running terrain. What still has no control is
## re-*placing* settlements, which is what both rows below are about.
func _build_settlement_gaps(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Not built")
	var pop := DccWidgets.action(sec, "Auto-populate world", func(): pass)
	pop.disabled = true
	pop.tooltip_text = "The reference's #civAutoPopulateBtn, plus its capitals / towns / hamlets count sliders. In this port settlement placement is not a separate pass: compute_civilisation runs inside generate() and there is no civ_populate #[func] to call on its own, nor any parameter for the three counts (params.rs has 58 entries, none of them civ). Re-generate from World ▸ Generation pipeline to re-place everything."
	var clear := DccWidgets.action(sec, "Clear places & routes", func(): pass)
	clear.disabled = true
	clear.tooltip_text = "The reference's #civClearPlacesBtn. Same shape: no civ_clear_places #[func] exists, and CivData is rebuilt wholesale by generate() rather than mutated in place, so there is no partial teardown to expose. Individual manual drops can still be undone by re-generating."
	var diag := DccWidgets.action(sec, "Settlement diagnostics overlay", func(): pass)
	diag.disabled = true
	diag.tooltip_text = "The reference's #civDiagnosticsChk (drawCivLayer §2.6). Every line of the card it draws is urban-morphology data: _umWallSpec's wall ladder, _umSiteProfile's river classification, and a peek into _umModelCache for bridge/ford/harbour validity, inside a SITE_WM×SITE_HM footprint box. None of those exist in this port -- cartalith-urban milestones 8-17 are unported and the crate has no consumer at all -- so the overlay would have nothing to draw. PARITY_AUDIT.md §5 item 13."
	DccWidgets.note(sec,
		"The biome carrying-capacity residual IS exposed now (File ▸ New world ▸ Generation -- "
		+ "the reference's own #civBiomeKChk, default off). Urban morphology layouts remain a "
		+ "separate unported subsystem (URBAN_MORPHOLOGY_SCOPE.md, Phase 5, in progress). "
		+ "Village seeding, the imperial-seat (metropolis) tier and the post-collapse recovery "
		+ "phase are exposed in the same place.")

func _settlement_row(parent: Control, data: Dictionary, index: int) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)
	var text := "%s -- %s, pop %d" % [data.get("name", "?"), String(data.get("kind", "?")).capitalize(), int(data.get("population", 0))]
	var b := DccWidgets.action(row, text, func():
		_selected_index = index
		app.right_dock_ctrl.on_settlement_selected(data, index))
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.tooltip_text = "Pin this settlement in the right dock (same as clicking it on the map)."
	var edit := DccWidgets.action(row, "✎", func():
		_selected_index = index
		app.open_place_editor(index))
	edit.tooltip_text = "Open the place editor (name, class, polity, population, economy, traits, history, delete)."
	parent.add_child(row)

# -- Population -----------------------------------------------------------

## Split build/fill, like Settlements above: `_build_*` runs once and claims
## the category body, `_fill_*` is what `_rebuild_readouts()` can re-run.
func _build_population() -> void:
	_population_body = DccWidgets.category(self, "Population", categories)
	_fill_population(_population_body)

func _fill_population(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Totals")
	var settlements := bridge.settlements()
	if settlements.is_empty():
		DccWidgets.note(sec, "No settlements -- generate a world first.")
		return

	var total := 0
	var largest_name := ""
	var largest_pop := -1
	for s in settlements:
		var d: Dictionary = s
		var pop := int(d.get("population", 0))
		total += pop
		if pop > largest_pop:
			largest_pop = pop
			largest_name = String(d.get("name", "?"))
	DccWidgets.note(sec, "Total population %d across %d settlements. Largest: %s (%d)." % [
		total, settlements.size(), largest_name, largest_pop])
	DccWidgets.note(sec,
		"Per-settlement population is real (get_settlements()'s own field). Faction- or " +
		"province-level population aggregation has no binding -- see Politics below for " +
		"what get_provinces() does carry.")

# -- Economy ----------------------------------------------------------------

func _build_economy() -> void:
	_economy_body = DccWidgets.category(self, "Economy", categories)
	_fill_economy(_economy_body)

func _fill_economy(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Trade balance")
	var settlements := bridge.settlements()
	var balances := bridge.trade_balances()
	if balances.is_empty():
		DccWidgets.note(sec, "No trade balances -- generate a world first.")
		return

	var exports := {}
	var imports := {}
	var trading := 0
	for i in range(balances.size()):
		var t: Dictionary = balances[i]
		var ex: PackedStringArray = t.get("exports", PackedStringArray())
		var im: PackedStringArray = t.get("imports", PackedStringArray())
		if ex.size() > 0 or im.size() > 0:
			trading += 1
		for r in ex:
			exports[r] = int(exports.get(r, 0)) + 1
		for r in im:
			imports[r] = int(imports.get(r, 0)) + 1
	DccWidgets.note(sec, "%d of %d settlements carry a trade relationship (surplus or deficit)." % [
		trading, settlements.size()])
	DccWidgets.note(sec, "Most-exported: %s. Most-imported: %s." % [
		_top_key(exports), _top_key(imports)])
	DccWidgets.note(sec,
		"This is the hinterland term only (civ_resource_trade_balance) -- the full " +
		"faction-level aggregation (population, tax, the five-axis power heuristic) is " +
		"real future scope per ECONOMY_SCOPE.md, not yet computed.")

func _top_key(counts: Dictionary) -> String:
	if counts.is_empty():
		return "none"
	var best_key := ""
	var best_n := -1
	for k in counts:
		if int(counts[k]) > best_n:
			best_n = int(counts[k])
			best_key = String(k)
	return "%s (%d settlements)" % [best_key, best_n]

# -- Politics -----------------------------------------------------------

func _build_politics() -> void:
	_politics_body = DccWidgets.category(self, "Politics", categories)
	_fill_politics(_politics_body)

func _fill_politics(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Factions")
	var provinces := bridge.provinces()
	var settlements := bridge.settlements()
	if provinces.is_empty():
		DccWidgets.note(sec, "No provinces -- generate a world first.")
	else:
		var by_faction := {}
		for p in provinces:
			var d: Dictionary = p
			var f := int(d.get("faction", 0))
			if not by_faction.has(f):
				by_faction[f] = []
			by_faction[f].append(d)
		var factions: Array = by_faction.keys()
		factions.sort()
		var roster := DccWidgets.group(sec, "Roster, by province count")
		for f in factions:
			var provs: Array = by_faction[f]
			var cap_name := "—"
			if not provs.is_empty():
				var cap_idx := int((provs[0] as Dictionary).get("capital_settlement_index", -1))
				if cap_idx >= 0 and cap_idx < settlements.size():
					cap_name = String((settlements[cap_idx] as Dictionary).get("name", "—"))
			var text := "Faction %d -- %d provinces, capital %s" % [f, provs.size(), cap_name]
			var b := DccWidgets.action(roster, text, func(): app.right_dock_ctrl.show_faction(f))
			b.alignment = HORIZONTAL_ALIGNMENT_LEFT
			b.tooltip_text = "Open this faction in the right dock."

	## `civOpenFactionsBtn` (`GUI_GAP_REGISTER.md` CV-07 / MS-13). The
	## register's reason -- "CIV_FACTION_COUNT is a compile-time constant …
	## get_factions() enumerates a fixed set, it does not own one" -- was
	## true when written and is not any more: `CivData::faction_roster` owns
	## a real, growable roster.
	var roster_btn := DccWidgets.action(sec, "Faction roster…", func(): app.open_faction_roster(), true)
	roster_btn.tooltip_text = "The reference's Faction Roster modal: world overview, per-faction cards, and the inspector (name / culture / religion / government / ag-tech, procedural banner, Territory fit, settlement sublist), plus add and remove faction."

	DccWidgets.note(sec,
		"Territory paint and settlement placement (STRANDED_TOOLS.md rows 10, 12) are wired " +
		"now -- see the TOOLS block at the top of this dock (§4.5.3's Settlement and " +
		"Territory tools).")

	var gaps := DccWidgets.section(parent, "Not built")
	var recalc := DccWidgets.action(gaps, "Recalculate territories", func(): pass)
	recalc.disabled = true
	recalc.tooltip_text = "The reference's territory recompute. assign_territory() runs inside compute_civilisation as part of generate(); no #[func] re-runs it against edited settlements, so a manual drop does not redraw the claim map until the next full re-generate (which is what the Settlement tool's own status hint already says). Painting a claim by hand is the wired alternative -- the Territory tool above."
	var clear_ter := DccWidgets.action(gaps, "Clear territory", func(): pass)
	clear_ter.disabled = true
	clear_ter.tooltip_text = "Same: CivData::territory is rebuilt wholesale by generate() and there is no civ_clear_territory #[func]. The Territory tool's own Discard reverts an uncommitted draft only, not the committed claim map."
	var gen_prov := DccWidgets.action(gaps, "Generate provinces", func(): pass)
	gen_prov.disabled = true
	gen_prov.tooltip_text = "The reference's province generator. Provinces are produced inside generate() and only read out (get_provinces()); no #[func] regenerates them. Their map tint IS live -- Cartography ▸ Layers ▸ Political — provinces."
	DccWidgets.note(gaps,
		"Diplomatic relations (the design's own per-faction sub-list) is new work with no " +
		"reference behaviour behind it either -- cartalith-civ models no inter-faction " +
		"relation of any kind.")

# -- Culture ----------------------------------------------------------------

func _build_culture() -> void:
	var cat := DccWidgets.category(self, "Culture", categories)
	var sec := DccWidgets.section(cat, "Profiles")
	DccWidgets.note(sec,
		"cartalith-civ generates culture profiles internally (generation rules + culture " +
		"profiles milestone, STATUS.md), but no GDExtension method exports them -- " +
		"get_settlements()/get_provinces() carry no culture field, and there is no " +
		"get_cultures() to read one from. Nothing here can be honest until that binding " +
		"lands.")

# -- Timeline (`TIMELINE_SCOPE.md` milestone 6) --------------------------------
#
# Built as a sixth L2 category in THIS workspace's left dock, alongside
# Settlements/Population/Economy/Politics/Culture above, rather than a new
# `right_dock.gd` CTX_* context. `right_dock.gd`'s own CTX_SCULPT/CTX_JOURNEY
# (the precedent this milestone's own brief pointed at) are both driven by an
# actual map TOOL arming (`app.tool_armed`) tied to viewport interaction --
# Sculpt shows the live stamp stack while the Sculpt tool is armed or a
# stroke just ended; Journey swaps the whole INFRA region while the JOURNEY
# tool is armed. Timeline has no map click of its own: add year / goto year /
# run simulation are all pure state edits with no tool to arm, exactly like
# THIS FILE's own Settlements/Population/Politics categories already are
# (click a row, pin something, done). Given that, `DccWidgets.category()` --
# this file's own established vocabulary, used five times already above -- is
# the correctly-scoped precedent, not `right_dock.gd`'s tool-armed dispatch.
#
# Also deliberately NOT wired into `dcc_shell.gd`'s own `timeline_bar`/
# `timeline_row` (`_build_timeline()` there -- the empty bottom strip
# `DCC_CONTROL_INDEX.md` §10 reserves, shown for civilization (was
# civilization/infrastructure before the 2026-08-20 domain merge -- one
# surviving id already covers both), `app.gd`'s `_on_workspace_changed`).
# `TIMELINE_SCOPE.md` §4's own
# words: "if you're unsure whether a given shell region is the discrete
# scrub mechanism vs the continuous six-toggle feature, stop and default to
# building your own dedicated panel rather than risking the wrong one." That
# bar is one fixed-height HBox row -- no room for a year-pill list, an add-
# year field, three filter checkboxes AND the whole collapse-sim form -- and
# §10 never disambiguates whether ITS OWN scrub track is meant for this
# discrete `civTimeline` or the still-open six-toggle continuous simulation.
# Left untouched rather than risk building into the wrong one; this category
# is the "dedicated panel" that note calls for instead.
#
# `get_settlements()` (`lib.rs`) now carries a real `tid` field (previously a
# disclosed gap here -- `NamedSettlement` had one at the Rust level but
# nothing exported it). `_tl_apply_filters` (below `_refresh_civ_data`) uses
# it: "Exist only" filters the array handed to `map_overlay.gd`'s
# `set_civ_data` down to the active year's `civ_year_diff().present` tids,
# upstream of that file rather than inside it. "Ghost removed"/"Highlight
# new" still cannot style individual pins -- that needs per-pin fade/halo
# drawing (`map_overlay.gd`'s own `_draw()`, out of scope this pass) and, for
# removed pins specifically, the OLD snapshot's settlement data (position/
# name), which no `#[func]` exposes yet (`civ_year_diff()` returns tid sets
# only). Real, disclosed remaining gap -- see `_build_timeline_filters`'s own
# note, not silently faked.

func _build_timeline() -> void:
	var cat := DccWidgets.category(self, "Timeline", categories)
	_tl_body = cat
	## The two `bridge.generation_finished`/`bridge.world_loaded` connections
	## that used to live here are now `_build()`'s single pair, calling
	## `_on_world_changed()` -- which calls `_tl_on_world_changed()` below
	## alongside the other five categories. For a long time this was the ONLY
	## subscriber in the file, which is exactly why the rest of the dock never
	## refreshed (RF-01); one connection point makes that hard to repeat.
	_rebuild_timeline()

## A fresh generate/loaded save invalidates any in-flight playback and the
## last simulation's own readout -- same reasoning `right_dock.gd`'s
## `_rebuild()` already applies on the same two signals.
func _tl_on_world_changed() -> void:
	_tl_playing = false
	if _tl_play_timer != null:
		_tl_play_timer.stop()
	_tl_sim_out = ""
	_rebuild_timeline()

## Tears down and rebuilds just this category's body -- the same "whole-
## section rebuild on every action" discipline `right_dock.gd`'s own
## `_rebuild()`/`show_sculpt_stack()` pair already establishes, scoped to
## `_tl_body` rather than the whole workspace so Settlements/Population/etc.
## above are untouched.
func _rebuild_timeline() -> void:
	if _tl_body == null:
		return
	for c in _tl_body.get_children():
		_tl_body.remove_child(c)
		c.queue_free()
	if not bridge.has_world:
		DccWidgets.note(_tl_body, "Generate a world first.")
		return
	_build_timeline_years(_tl_body)
	_build_timeline_scrub(_tl_body)
	_build_timeline_playback(_tl_body)
	_build_timeline_filters(_tl_body)
	_build_timeline_sim(_tl_body)

## `_civFormatYear` (reference line 20644), ported verbatim: negative years
## are BC.
static func _tl_format_year(year: int) -> String:
	return ("%d BC" % -year) if year < 0 else ("%d AD" % year)

## `get_civ_timeline_years()` is already ascending (`lib.rs`'s own doc
## comment) -- re-sorted defensively rather than trusted blindly, since
## nothing here is on a hot path.
func _tl_years() -> Array:
	var out: Array = Array(bridge.get_civ_timeline_years())
	out.sort()
	return out

func _tl_nearest_year(years: Array, target: int) -> int:
	var nearest: int = years[0]
	for y in years:
		if absi(int(y) - target) < absi(nearest - target):
			nearest = int(y)
	return nearest

## Same reasoning as `_refresh_civ_data()`/`_commit_territory()` above: every
## call that moves the timeline cursor (`civGotoYear`) reloads `territory`
## engine-side but does not itself touch the rendered texture -- this is the
## direct field write that does, without `ViewportHost.refresh()`'s camera-
## reset side effect.
func _tl_refresh_territory_view() -> void:
	if app != null and app.viewport != null and app.viewport.territory_view != null:
		app.viewport.territory_view.texture = bridge.territory_texture()

func _tl_goto_year(year: int) -> void:
	bridge.civ_goto_year(year)
	_tl_refresh_territory_view()
	_refresh_civ_data()   ## "Exist only" is keyed to the active year -- see `_tl_apply_filters`.

func _tl_add_year_action() -> void:
	bridge.civ_add_year(_tl_add_year)
	_tl_refresh_territory_view()
	_rebuild_timeline()

func _tl_remove_year_action(year: int) -> void:
	bridge.civ_remove_year(year)
	_tl_refresh_territory_view()
	_rebuild_timeline()

# -- Years pill row + Add year (Cluster A: civAddYear/civRemoveYear/civGotoYear) --

func _build_timeline_years(body: Control) -> void:
	var sec := DccWidgets.section(body, "Years")
	var add_row := HBoxContainer.new()
	add_row.add_theme_constant_override("separation", 6)
	var yi := SpinBox.new()
	yi.min_value = -9999
	yi.max_value = 9999
	yi.step = 1
	yi.value = _tl_add_year
	yi.custom_minimum_size.x = 90
	yi.value_changed.connect(func(v: float): _tl_add_year = int(v))
	add_row.add_child(yi)
	DccWidgets.action(add_row, "Add year", _tl_add_year_action)
	sec.add_child(add_row)

	var years := _tl_years()
	if years.is_empty():
		DccWidgets.note(sec, "No years added yet.")
	else:
		var flow := HFlowContainer.new()
		flow.add_theme_constant_override("h_separation", 4)
		flow.add_theme_constant_override("v_separation", 4)
		var cur := bridge.get_civ_year()
		for y in years:
			flow.add_child(_tl_year_pill(int(y), int(y) == cur))
		sec.add_child(flow)
		DccWidgets.note(sec, "Active year: %s" % _tl_format_year(cur))
	DccWidgets.note(sec,
		"Each year stores a snapshot of the territory paint plus which settlements/roads " +
		"existed then. Negative years are BC. Tap a pill to jump to that era, or %s to remove it." %
			DccIcons.SYMBOLS["cross"])

func _tl_year_pill(year: int, active: bool) -> Control:
	var pill := HBoxContainer.new()
	pill.add_theme_constant_override("separation", 1)
	var go := Button.new()
	go.text = _tl_format_year(year)
	go.flat = true
	go.focus_mode = Control.FOCUS_NONE
	go.custom_minimum_size.y = 22
	go.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	go.add_theme_font_override("font", DccTheme.mono(1))
	go.add_theme_color_override("font_color", DccTheme.c("bg") if active else DccTheme.c("text"))
	go.add_theme_color_override("font_hover_color", DccTheme.c("bg") if active else DccTheme.c("text_bright"))
	go.add_theme_stylebox_override("normal", DccTheme.flat(DccTheme.c("accent") if active else DccTheme.c("sunken"), 2))
	go.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("accent").lightened(0.1) if active else DccTheme.c("raised"), 2))
	go.tooltip_text = "Jump to %s (civ_goto_year)." % _tl_format_year(year)
	go.pressed.connect(func(): _tl_goto_year(year); _rebuild_timeline())
	pill.add_child(go)
	var rm := Button.new()
	rm.text = DccIcons.SYMBOLS["cross"]
	rm.flat = true
	rm.focus_mode = Control.FOCUS_NONE
	rm.custom_minimum_size = Vector2(18, 22)
	rm.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	rm.add_theme_color_override("font_color", DccTheme.c("text_ghost"))
	rm.add_theme_color_override("font_hover_color", DccTheme.c("accent"))
	rm.tooltip_text = "Remove %s (civ_remove_year)." % _tl_format_year(year)
	rm.pressed.connect(func(): _tl_remove_year_action(year))
	pill.add_child(rm)
	return pill

# -- Scrub track (Cluster C: _civWireYearSlider, v0.91 real time-scale) --------

func _build_timeline_scrub(body: Control) -> void:
	var years := _tl_years()
	if years.size() < 2:
		return   ## Gated exactly like the reference's #explTimelineSliderRow (>1 recorded year).
	var sec := DccWidgets.section(body, "Scrub")
	var lo: int = years[0]
	var hi: int = years[years.size() - 1]
	var cur := bridge.get_civ_year()
	var caps := HBoxContainer.new()
	caps.add_child(DccTheme.mono_label(_tl_format_year(lo), "text_ghost", DccTheme.FS_TINY))
	caps.add_child(DccTheme.spacer())
	caps.add_child(DccTheme.mono_label(_tl_format_year(hi), "text_ghost", DccTheme.FS_TINY))
	sec.add_child(caps)
	DccWidgets.slider(sec, "Year", float(lo), float(hi), 1.0, float(cur), "",
		func(v: float): _tl_goto_year(_tl_nearest_year(years, int(round(v)))),
		"Real time-scale -- min/max/value are the actual recorded years, not a snapshot-count " +
			"index (reference v0.91). Dragging snaps to the nearest recorded year.",
		func(): _rebuild_timeline())
	DccWidgets.note(sec, "Active year: %s" % _tl_format_year(cur))

# -- Playback transport (Cluster C: _civTlStartPlay/_civTlStopPlay, 1200ms) ----

func _build_timeline_playback(body: Control) -> void:
	var years := _tl_years()
	if years.size() < 2:
		return   ## Same gate as the scrub row -- reference shares one markup section for both.
	var sec := DccWidgets.section(body, "Playback")
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	var label_text := ("%s Stop" % DccIcons.SYMBOLS["pause"]) if _tl_playing else ("%s Animate" % DccIcons.SYMBOLS["play"])
	DccWidgets.action(row, label_text, func(): _tl_stop_play() if _tl_playing else _tl_start_play())
	DccWidgets.action(row, "Step", _tl_step)
	sec.add_child(row)

func _tl_ensure_timer() -> void:
	if _tl_play_timer == null:
		_tl_play_timer = Timer.new()
		_tl_play_timer.wait_time = 1.2   ## Reference's own 1200ms interval, verbatim.
		_tl_play_timer.one_shot = false
		_tl_play_timer.timeout.connect(_tl_on_play_tick)
		add_child(_tl_play_timer)

func _tl_start_play() -> void:
	var years := _tl_years()
	if years.size() < 2:
		return
	_tl_ensure_timer()
	_tl_playing = true
	_tl_play_timer.start()
	_rebuild_timeline()

func _tl_stop_play() -> void:
	_tl_playing = false
	if _tl_play_timer != null:
		_tl_play_timer.stop()
	_rebuild_timeline()

func _tl_on_play_tick() -> void:
	var years := _tl_years()
	if years.size() < 2:
		_tl_stop_play()
		return
	var cur := bridge.get_civ_year()
	var idx := years.find(cur)
	var next: int = (idx + 1) if idx >= 0 else 0
	if next >= years.size():
		_tl_stop_play()
		return
	_tl_goto_year(int(years[next]))
	if next == years.size() - 1:
		_tl_stop_play()
	else:
		_rebuild_timeline()

func _tl_step() -> void:
	var years := _tl_years()
	if years.size() < 2:
		return
	var cur := bridge.get_civ_year()
	var idx := years.find(cur)
	var next: int = (idx + 1) if idx >= 0 else 0
	if next >= years.size():
		return
	_tl_goto_year(int(years[next]))
	_rebuild_timeline()

# -- Filters (Cluster C: explTlExistOnly/Ghost/Highlight, _civYearDiff) --------

func _build_timeline_filters(body: Control) -> void:
	var sec := DccWidgets.section(body, "Filters")
	DccWidgets.toggle(sec, "Exist only", _tl_filter_exist_only,
		func(v: bool): _tl_filter_exist_only = v; _refresh_civ_data(),
		"Reference: hide anything not present in the selected year (civ_year_diff().present). " +
			"Real here -- unchecking removes non-present settlement pins from the map.")
	DccWidgets.toggle(sec, "Ghost removed", _tl_filter_ghost,
		func(v: bool): _tl_filter_ghost = v,
		"Reference: fade objects removed since the previous recorded year (civ_year_diff().removed).")
	DccWidgets.toggle(sec, "Highlight new", _tl_filter_highlight,
		func(v: bool): _tl_filter_highlight = v,
		"Reference: halo objects added since the previous recorded year (civ_year_diff().added).")
	var years := _tl_years()
	if not years.is_empty():
		var diff: Dictionary = bridge.civ_year_diff(bridge.get_civ_year())
		var present: int = (diff.get("present", PackedInt64Array()) as PackedInt64Array).size()
		var removed: int = (diff.get("removed", PackedInt64Array()) as PackedInt64Array).size()
		var added: int = (diff.get("added", PackedInt64Array()) as PackedInt64Array).size()
		DccWidgets.note(sec,
			"%d present, %d removed, %d added vs. the previous recorded year -- real, live civ_year_diff() output." %
				[present, removed, added])
	DccWidgets.note(sec,
		"get_settlements() now carries tid (lib.rs) -- 'Exist only' is wired for real and " +
		"filters map pins by the active year's civ_year_diff().present. 'Ghost removed'/" +
		"'Highlight new' still cannot style individual pins: that needs per-pin fade/halo " +
		"drawing (map_overlay.gd's own _draw(), out of scope this pass) and, for removed " +
		"pins specifically, the OLD snapshot's settlement data, which no #[func] exposes yet " +
		"(civ_year_diff() returns tid sets only, not positions/names). Disclosed, not faked.")

# -- Collapse / recovery simulator form (Cluster B/impure wiring) -------------

func _build_timeline_sim(body: Control) -> void:
	var grp := DccWidgets.group(body, "Simulate collapse / recovery", false)
	DccWidgets.choice(grp, "Mode", ["Collapse (decline + migration)", "Recovery (regrowth)"],
		0 if _tl_sim_mode == "collapse" else 1,
		func(i: int): _tl_sim_mode = ("collapse" if i == 0 else "recovery"); _rebuild_timeline())
	if _tl_sim_mode == "collapse":
		var chars := ["mixed", "trade", "disease", "conflict"]
		DccWidgets.choice(grp, "Character",
			["Mixed (default)", "Trade -- hubs fall hardest", "Disease -- dense/connected fall first",
				"Conflict -- undefended fall first"],
			maxi(0, chars.find(_tl_sim_character)),
			func(i: int): _tl_sim_character = chars[i])
		DccWidgets.slider(grp, "Severity", 0.0, 100.0, 1.0, _tl_sim_severity * 100.0, "%",
			func(v: float): _tl_sim_severity = v / 100.0)
	else:
		DccWidgets.slider(grp, "Regrowth rate", 0.1, 3.0, 0.1, _tl_sim_rate * 100.0, "%/yr",
			func(v: float): _tl_sim_rate = v / 100.0)
	DccWidgets.number(grp, "Start year", -9999.0, 9999.0, 1.0, float(_tl_sim_start_year),
		func(v: float): _tl_sim_start_year = int(v))
	DccWidgets.number(grp, "Duration (yr)", 1.0, 1000000.0, 1.0, float(_tl_sim_duration),
		func(v: float): _tl_sim_duration = int(v))
	DccWidgets.number(grp, "Step years", 1.0, 1000000.0, 1.0, float(_tl_sim_step_years),
		func(v: float): _tl_sim_step_years = int(v))
	DccWidgets.action(grp, "Simulate", func(): _tl_run_simulation(false), true)
	DccWidgets.note(grp,
		"Runs a year-by-year simulation from the CURRENT settlements and writes one " +
		"timeline entry per step. Collapse: each settlement's stress (trade/density/ " +
		"undefended exposure, weighted by Character) drives mortality and gravity-model " +
		"out-migration each step; a nucleus below its tier's floor demotes or is " +
		"abandoned. Recovery: logistic regrowth toward each settlement's catchment " +
		"ceiling. See docs/research/collapse-timeline-dynamics.md.")
	if _tl_sim_out != "":
		DccWidgets.note(grp, _tl_sim_out)

func _tl_run_simulation(confirm_overwrite: bool) -> void:
	var request := {
		"mode": _tl_sim_mode,
		"character": _tl_sim_character,
		"severity": _tl_sim_severity,
		"rate": _tl_sim_rate,
		"start_year": _tl_sim_start_year,
		"duration": _tl_sim_duration,
		"step_years": _tl_sim_step_years,
		"confirm_overwrite": confirm_overwrite,
	}
	var result: Dictionary = bridge.civ_run_collapse_simulation(request)
	if not bool(result.get("ok", false)):
		if bool(result.get("needs_confirm", false)):
			_tl_show_confirm(result)
			return
		_tl_sim_out = String(result.get("error", "Simulation failed."))
		_rebuild_timeline()
		return
	_tl_sim_out = _tl_format_sim_result(result)
	_tl_refresh_territory_view()
	_rebuild_timeline()

## `_civRunCollapseSimulation`'s own blocking `confirm()` (reference line
## 24911), reimplemented as a real Yes/No dialog -- this shell's own
## precedent is `AcceptDialog` built by hand and `add_child`ed onto `app`
## (`app.gd`'s `open_storage_locations`/`open_credits`, `cartography_
## workspace.gd`'s `_prompt_label_name`), none of which needed a Cancel path.
## `ConfirmationDialog` (a built-in `AcceptDialog` subclass adding exactly
## that Cancel/close-as-No button) is the closest match to that convention
## for a real two-way choice.
func _tl_show_confirm(result: Dictionary) -> void:
	var years: PackedInt64Array = result.get("clobber_years", PackedInt64Array())
	var parts: Array[String] = []
	for y in years:
		parts.append(_tl_format_year(int(y)))
	var dlg := ConfirmationDialog.new()
	dlg.title = "Overwrite recorded years?"
	dlg.dialog_text = "Simulation will overwrite %d existing timeline year%s (%s).\n\nContinue?" % [
		years.size(), "" if years.size() == 1 else "s", ", ".join(parts)]
	dlg.get_ok_button().text = "Overwrite"
	dlg.confirmed.connect(func(): _tl_run_simulation(true); dlg.queue_free())
	dlg.canceled.connect(dlg.queue_free)
	app.add_child(dlg)
	dlg.popup_centered()

## Reference's own `civSimOut` innerHTML (lines 24940-24949), ported field-
## for-field -- `fmt()`'s k/M abbreviation is skipped (this port's numbers
## are small enough in practice, and DccTheme has no established large-number
## abbreviation convention elsewhere to match).
func _tl_format_sim_result(r: Dictionary) -> String:
	var steps := int(r.get("steps", 0))
	var end_year := int(r.get("end_year", 0))
	var final_n := int(r.get("final_settlements", 0))
	var head := "Simulated %d step%s (%d yr each), %s -> %s. %d settlements remain." % [
		steps, "" if steps == 1 else "s", _tl_sim_step_years,
		_tl_format_year(_tl_sim_start_year), _tl_format_year(end_year), final_n]
	if _tl_sim_mode == "recovery":
		return head
	var died := int(r.get("died", 0))
	var migrated := int(r.get("migrated", 0))
	var unplaced := int(r.get("unplaced", 0))
	var failed := int(r.get("failed", 0))
	return "%s %d died, %d migrated (%d lost in transit/diaspora), %d settlement%s failed/abandoned." % [
		head, died, migrated, unplaced, failed, "" if failed == 1 else "s"]
