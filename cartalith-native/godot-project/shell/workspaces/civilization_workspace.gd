extends Workspace
class_name CivilizationWorkspace

## CIVIL domain (§3): settlements, population, economy, politics, culture,
## and (§4.5.3) the Settlement and Territory tools.
##
## The engine backs all five browsing categories as *readable* data
## (`get_settlements`, `get_provinces`, `get_trade_balances`,
## `get_factions`, `get_cultures`).
##
## **Culture was the exception until 2026-09-01, and had stopped being one
## on 2026-08-25.** This doc, and a note on screen in the panel itself, both
## said "there is no `get_cultures()` to read one from". There is: `lib.rs`'s
## `#[func] fn get_cultures()` closed `GUI_GAP_REGISTER.md` CV-02 and returns
## `{id, key, name, terrain_affinity, faction_count, factions,
## settlement_count, population}` per culture. Nothing called it -- not this
## file, and not `engine_bridge.gd`, which still has no wrapper for it (see
## `_cultures()` below for how this file reaches it in the meantime).
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

## -- Timeline state (`TIMELINE_SCOPE.md` milestone 6). See the Politics
## section's own header comment, below `_build_culture()`, for why this is a
## `DccWidgets.category()` here rather than a new `right_dock.gd` CTX_*
## context. --
##
## v3 splits what used to be one Timeline category in two, so this state now
## backs two bodies: `_tl_body` is **Politics** (the recorded years, the
## scrubber, playback and the existence filters -- political change over
## time) and `_sim_body` is **Simulation** (the collapse/recovery model that
## writes into those years). They share every `_tl_sim_*` field below because
## they are two views of one subject, and `_rebuild_timeline()` refills both
## from one place -- a simulation run has to re-draw the year list it just
## appended to, and the year list has to re-draw the result note.
var _tl_body: VBoxContainer
var _sim_body: VBoxContainer
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
## The other data-backed categories' own bodies, held for the same
## reason and cleared/refilled by the same `_rebuild_readouts()`. Culture
## joined them on 2026-09-01: its seven rows exist without a world (they are
## `CIV_CULTURES`, compile-time constants) but their faction, settlement and
## population counts do not, so the category has to refill like the rest.
## `_tl_body` (Politics) and `_sim_body` (Simulation) are declared with the
## rest of the timeline state above, because they also carry playback and
## simulation state to reset.
##
## `_politics_body` became `_factions_body` + `_territories_body` on
## 2026-08-24 with v3, which splits the old Politics category in two -- who
## the polities *are*, and what ground they hold. Politics is now the
## time-varying half (v3: "political change over time").
var _population_body: Control
var _economy_body: Control
var _culture_body: Control
var _factions_body: Control
var _territories_body: Control
## CV-25's and CV-26's category bodies. Both refill on `_rebuild_readouts()`
## like the four above: their inputs are the settlement roster, the place
## editor's overrides and the territory raster, and all three move under a
## place edit or a recompute. Both calls are one O(cells) aggregate pass and
## one O(cells) border pass -- the same cost class as `civ_faction_terrain_fits`,
## which the roster window already pays on open, and unlike `_influence_body`
## above they run no Dijkstra.
var _military_body: Control
var _relations_body: Control
## CV-23's on-demand influence readout, inside `_territories_body`. Refilled
## by `_analyse_influence()` only -- never by `_rebuild_readouts()`, which
## would run one Dijkstra per capital on every place rename.
var _influence_body: VBoxContainer
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
## The civ-authoring ruling's Populate row (`LARGE_ITEM_RULINGS.md`,
## 2026-08-31). Held as fields for exactly the reason `_recompute_btn` above
## is: `_populate_world` is a coroutine that has to find its own button again
## after the blocking engine call.
var _populate_btn: Button
var _populate_note: Label
var _clear_places_btn: Button
## `PARITY_AUDIT.md` §23 F13's `civ_regional_population` readout -- held so
## `_on_compute_regional_population` can find the label it already built
## instead of rebuilding the section, the same reasoning `_recompute_note`
## documents above.
var _regional_pop_note: Label

## -- CIVIL ▸ Landmarks (`design/landmark-generation/LANDMARK_UI_DESIGN.md`).
## See the `# -- CIVIL ▸ Landmarks` block near the foot of this file for what
## each of these backs; they are declared together here with the rest of the
## dock's category state rather than beside their own code, so the "what does
## this workspace retain" question has one answer. --
var _landmarks_body: Control
## One record per type row, keyed by the ENGINE's own kind key. Holds both the
## nodes (`row`/`under`/`bar`/`line`/`count`/`token`/`slider`/`readout`) and the
## row's model (`cap`/`armed`/`rung`/`retained`/`default_cap`/`funnel`), because
## §2.2's second line is a function of all of them at once and recomputing it
## from a walk of the tree would be reading the display to redraw the display.
var _lm_rows := {}
## family key -> {button, body, title}. The `button` is `DccWidgets.group()`'s
## header, which that factory does not return -- §3.2's counts live on it.
var _lm_groups := {}
## class key ("" for `all`) -> the lit-set chip. §3.1's filter.
var _lm_chips := {}
var _lm_filter := ""
var _lm_head_note: Label       ## §4.4's headroom line.
var _lm_crowd_note: Label      ## §4.1's "a regional landmark keeps 34 km clear".
var _lm_crowd_readout: Label   ## The `× 1.00` the slider factory cannot format.
var _lm_run_btn: Button
var _lm_run_note: Label
var _lm_stale_note: Label
## §5's funnel, built lazily on the first click and reused: one popover, refilled
## per type, rather than 49 that mostly never open.
var _lm_funnel: PopupPanel
var _lm_crowding := 1.0
var _lm_radii: Array = []


func _build() -> void:
	_infra = InfrastructureWorkspace.new()
	_infra._nested = true
	## Both flags have to be set before `setup()`, because `setup()` runs
	## `_infra._build()` and `_build()` reads them: `_nested` suppresses the
	## duplicate TOOLS row, `_dock_hosted` suppresses the categories this dock
	## now draws itself (v3): FOUR of `InfrastructureWorkspace`'s old five are
	## redrawn here as Routes & ways / Travel / Trade, through its own
	## `build_*_into()` entry points, and the fifth -- Rivers -- left this dock
	## entirely for WORLD ▸ Hydrology, which draws its one disclosure from the
	## same `InfrastructureWorkspace.rivers_note()`. Setting `_dock_hosted` inside
	## `build_ways_into()` alone was not enough -- that call happens *after*
	## `setup()`, so the old Roads/Rivers/Ports/Trade/Logistics categories
	## still got built once, under the wrong parent, before it ran.
	_infra._dock_hosted = true

	_build_tools()

	## Added and set up *before* the categories, because those categories are
	## what it draws into now. `_infra.setup()` runs its own `_build()`, which
	## draws no categories of its own while `_dock_hosted` (set above) but
	## still registers the Way/Route click, drag and escape handlers
	## `_build_tools()` above already drew buttons for. See
	## `InfrastructureWorkspace`'s own `_dock_hosted` doc.
	add_child(_infra)
	_infra.setup(app, bridge)

	## v3's fourteen CIVIL categories, in v3's own order.
	_build_civilizations()                                                ## 1
	_build_factions()                                                     ## 2
	_build_territories()                                                  ## 3
	_build_settlements()                                                  ## 4
	_build_landmarks()                                                    ## 5
	_infra.build_ways_into(
		DccWidgets.category(self, "Routes & ways", categories))           ## 6
	_infra.build_travel_into(
		DccWidgets.category(self, "Travel", categories))                  ## 7
	_infra.build_trade_into(
		DccWidgets.category(self, "Trade", categories))                   ## 8
	_build_economy()                                                      ## 9
	_build_culture()                                                      ## 10
	_build_timeline()                                                     ## 11 Politics
	_build_military()                                                     ## 12
	_build_relationships()                                                ## 13
	_build_simulation()                                                   ## 14

	## `04-left-dock.md` §6: "Landmarks is the floor." All fourteen categories
	## above (`_infra`'s three included -- they share this same `categories`
	## array, per its own header comment) now exist, so this is the one place
	## to attach the floor to every one of their headers at once. See
	## `_lm_enforce_floor()`'s own doc comment for the mechanism.
	for cat_entry: Dictionary in categories:
		var cat_btn: Button = cat_entry.get("button")
		if cat_btn != null and is_instance_valid(cat_btn):
			cat_btn.pressed.connect(_lm_enforce_floor)

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
	## Landmarks rebuild here and NOT in `_rebuild_readouts()`: a new world
	## replaces every placed landmark and every funnel, but a place rename does
	## not, and rebuilding on a rename would shut whichever family group the user
	## had open under them.
	_lm_rebuild()

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
	## The Political-control analysis field draws in the same swatches
	## (`GUI_GAP_REGISTER.md` CV-21), so an identity-colour edit leaves it
	## stale too. Re-asking for whatever view is up is free for every other
	## one, and this signal fires at the rate a roster is edited.
	if app.viewport.debug_view() == "control":
		app.viewport.set_debug_layer("control")
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
	if _culture_body != null and is_instance_valid(_culture_body):
		_clear_body(_culture_body)
		_fill_culture(_culture_body)
	if _factions_body != null and is_instance_valid(_factions_body):
		_clear_body(_factions_body)
		_fill_factions(_factions_body)
	if _territories_body != null and is_instance_valid(_territories_body):
		_clear_body(_territories_body)
		_fill_territories(_territories_body)
	if _military_body != null and is_instance_valid(_military_body):
		_clear_body(_military_body)
		_fill_military(_military_body)
	if _relations_body != null and is_instance_valid(_relations_body):
		_clear_body(_relations_body)
		_fill_relationships(_relations_body)

## `remove_child` before `queue_free` on purpose: `queue_free` defers to the
## end of the frame, so a child left parented is still in `get_children()`
## while the refill runs and would draw twice for one frame. Same teardown
## `_rebuild_timeline` performs inline, factored out here because four
## categories now need it.
## Biggest population first, for CIVIL ▸ Population's per-faction breakdown.
## A named function rather than an inline lambda: a lambda body cannot be
## wrapped across lines inside a call's argument list, and this comparison
## does not fit one.
static func _by_population_desc(a: Variant, b: Variant) -> bool:
	return int((a as Dictionary).get("population", 0)) > int((b as Dictionary).get("population", 0))

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

## `Edit > Deselect` (§2.2), the sibling of `on_delete_key` above and reached
## the same way: `DccApp.clear_selection()` walks the workspaces and calls
## whichever ones answer.
##
## Domain-guarded exactly as `on_delete_key` is -- deselecting a settlement
## while the user is in Cartography would clear a selection they cannot see,
## and the register's own rule is that a command acts on what is in front of
## the person issuing it.
##
## Returns whether there was anything to clear, so the caller can tell "there
## was no selection" from "it happened".
func on_deselect() -> bool:
	if app.active_domain() != "civilization":
		return false
	if _selected_index < 0:
		return false
	_selected_index = -1
	return true

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
		## The one `PopupMenu` in the shell that never went through the shell's
		## own styling: measured 232x54 on both desktop and tablet in the
		## 2026-08-25 menu sweep -- Godot's stock panel, stock selection bar and
		## a 15 px row on a device whose floor is 44. Every program menu beside
		## it is `#121314` with the accent wash, and this one was not.
		DccWidgets.style_popup(_ctx_menu)
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
				## **Both halves ARE bound, and this message said they were not**
				## (corrected 2026-09-01). Biome: `sample_cell()` sets a `biome`
				## key and `right_dock.gd`'s Sample panel has drawn a Biome row
				## off it for as long as the panel has existed. Wildlife:
				## `wildlife_region_at()` is bound, wrapped, and already called
				## by `app.gd`'s cursor handler whenever the Wildlife view is the
				## drawn layer -- the reference's own `state.debug === 'wildlife'`
				## gate. The real limitation is narrower and is what this now
				## says: the biome raster needs the civilisation layer's water
				## bodies, which a loaded `.zip` save does not carry.
				app.set_status("hint",
					"Nothing here. This cell's readings are in the Sample panel (right dock) — biome included, on a generated world; a loaded save carries no civilisation layer, so Biome reads — there. Wildlife appears in the same dock while Layers ▸ Wildlife is the drawn view.",
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
			## `05-right-dock-and-bars.md` §1.12, GUI replacement stage 5:
			## `rdMode4()` rule 4 -- unconditional on the tool. `leave_
			## territory_context()` below is the disarm half.
			if app.right_dock_ctrl.has_method("show_territory"):
				app.right_dock_ctrl.show_territory(_territory_faction)
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
			if app.right_dock_ctrl.has_method("leave_territory_context"):
				app.right_dock_ctrl.leave_territory_context()

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
		_faction_choice(row, _territory_faction, func(fid: int):
			_territory_faction = fid
			## Unlike Radius/Mode just below, a faction re-pick changes
			## WHICH faction's stats this dock's own right-dock companion
			## is showing -- `right_dock.gd`'s CTX_TERR keeps no state of
			## its own, so it has to be told.
			if app.right_dock_ctrl.has_method("show_territory"):
				app.right_dock_ctrl.show_territory(fid))
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
	## claimed-cell count, and contested-cell warning." **This used to pin
	## `show_faction()` instead**, with a comment explaining `right_dock.gd`'s
	## real Territory context "is explicitly not this pass's to change" --
	## true when that sentence was written; `right_dock.gd`'s CTX_TERR
	## (`05-right-dock-and-bars.md` §1.12) now exists and is the exact live
	## cells/area/contested reading that comment was waiting for, so this
	## points at it instead. `civ_faction_territory_stats` is unchanged --
	## this dock's own tool-options row above still reads it too.
	app.right_dock_ctrl.show_territory(_territory_faction)
	app.set_status("hint",
		"Territory committed -- provinces/trade were computed before this edit.", "text_ghost")
	if _active_civ_tool == "territory":
		_tool_options_territory()

func _discard_territory() -> void:
	bridge.civ_territory_discard()
	if _active_civ_tool == "territory":
		_tool_options_territory()
	## Discard only touches the draft (`civ_territory_discard`'s own doc);
	## the committed stats CTX_TERR reads are unaffected, but re-announcing
	## costs nothing and keeps this symmetric with commit and arm above.
	if app.right_dock_ctrl.has_method("show_territory"):
		app.right_dock_ctrl.show_territory(_territory_faction)


# -- Settlements --------------------------------------------------------

## v3 CIVIL ▸ CIVILIZATIONS: *"Auto-populate world · Clear places & routes ·
## + Placement model"* -- the world-scale act of putting people on a map,
## which v3 lifts out of Settlements (a browser over the result) and gives its
## own category, first on the rail.
##
## Every control it names is `_build_settlement_gaps`'s content: none of it is
## callable in this port, for one reason stated once rather than three times --
## `compute_civilisation` runs *inside* `generate()` and no `#[func]` runs it
## alone. The placement model's own dials do exist, in File ▸ New world ▸
## Generation, and the note says where.
##
## **No longer built open.** It was, before `_build_landmarks()` existed --
## the only category CIVIL had an opinion about opening was whichever one
## happened to be first. `04-left-dock.md` §6 states a default now
## ("`Default civCat = 'landmarks'`") and treats Landmarks as CIVIL's floor
## (`_lm_enforce_floor()`); leaving Civilizations open on first paint while
## every later fall-back lands on Landmarks would make the category CIVIL
## opens with different from the one it always returns to, for no reason
## either default is right. `_build_landmarks()` now carries the `true`.
func _build_civilizations() -> void:
	var cat := DccWidgets.category(self, "Civilizations", categories)
	DccWidgets.note(DccWidgets.section(cat, "How people get placed"),
		"Settlement placement is not a separate pass in this port: "
		+ "compute_civilisation runs inside generate(), reading the finished "
		+ "terrain, climate and biome fields. So the act v3 draws here is "
		+ "World ▸ Generate, and what is tunable is the placement model -- "
		+ "biome carrying-capacity, the imperial-seat tier, village seeding, "
		+ "urban layouts and the recovery phase -- which lives in File ▸ New "
		+ "world ▸ Generation because it is a creation-time choice.")
	var newworld := DccWidgets.action(cat, "Placement model → File ▸ New world…",
		func(): app.open_new_world())
	newworld.alignment = HORIZONTAL_ALIGNMENT_LEFT
	_build_settlement_gaps(cat)

## v3 CIVIL ▸ FACTIONS. The roster is the writable half (add/remove/edit,
## CV-07) and the province tally below it is the derived half.
##
## v3 also puts *identity colour* here, and marks it authoritative: "how it
## paints -- tint, opacity, border width -- is CARTO". That split is designed
## and not built on either side; the note says so rather than offering a
## colour picker nothing reads.
func _build_factions() -> void:
	_factions_body = DccWidgets.category(self, "Factions", categories)
	_fill_factions(_factions_body)

func _fill_factions(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Roster")
	var roster_btn := DccWidgets.action(sec, "Faction roster…", func(): app.open_faction_roster(), true)
	roster_btn.tooltip_text = "The reference's Faction Roster modal: world overview, per-faction cards, and the inspector (name / culture / religion / government / ag-tech, procedural banner, Territory fit, settlement sublist), plus add and remove faction."

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
		var roster := DccWidgets.group(sec, "By province count")
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

	## `GUI_GAP_REGISTER.md` **CV-21**, built 2026-08-25. The register's
	## reason ("FactionRoster stores no colour field") was wrong -- it stored
	## one and nothing read it. It is the render palette now.
	var identity := DccWidgets.section(parent, "Identity colour")
	DccWidgets.note(identity,
		"Each faction's own colour, set in the roster window and drawn by "
		+ "everything that draws a faction: the territory wash, the Political "
		+ "control analysis field, and its banner. Unset, it takes the "
		+ "colourblind-safe palette's colour for that index. The picker is the "
		+ "first row of the roster window's Identity block, beside the banner it "
		+ "repaints live.")
	## No second *Faction roster…* button: this category already has one
	## above, and two openers onto one window is the shape this shell keeps
	## having to undo.
	var paint_btn := DccWidgets.action(identity, "How heavily it paints → Cartography ▸ Political display",
		func(): app.select_domain_category("cartography", "Political display"))
	paint_btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
	paint_btn.tooltip_text = "v3's own split: which colour a faction *is* belongs here, how heavily the wash is laid on belongs to CARTO."

	## `GUI_GAP_REGISTER.md` **CV-22**, built 2026-08-25. The register's own
	## estimate was right to the line: one `EntityKind` variant, one `as_str`
	## arm, one `parse` arm, plus the export registry rows a faction can fill.
	var notes := DccWidgets.section(parent, "Linked notes")
	DccWidgets.note(notes,
		"A faction's history, notes and lore live in an external Markdown vault "
		+ "(any folder of .md files), the same as a settlement's, a province's "
		+ "and a continent's. Cartalith reads on demand and writes only on an "
		+ "explicit, previewed action.")
	## The roster, not the province index above: a faction with no province
	## yet is still a faction, and still has a history worth writing down.
	var roster_rows := bridge.get_factions()
	if roster_rows.is_empty():
		DccWidgets.note(notes, "No factions — generate a world first.")
	else:
		for f in roster_rows:
			var fd: Dictionary = f
			_knowledge_row(notes, "faction", int(fd.get("id", 0)), String(fd.get("name", "?")),
				"%d settlements · %s" % [int(fd.get("settlement_count", 0)),
					String(fd.get("culture", "")).capitalize()])
		DccWidgets.note(notes,
			"Cartalith can fill Name, Culture, Government, Religion, its capital's "
			+ "coordinates, member settlements, total population and claimed area "
			+ "into its own block in that note. The three vocabulary fields drive "
			+ "nothing in the engine (ECONOMY_SCOPE.md) — which is exactly why "
			+ "they are worth writing where an author's prose about them is.")

	var gaps := DccWidgets.section(parent, "Not built")
	DccWidgets.note(gaps,
		"A faction **emblem** (GUI_GAP_REGISTER.md CV-21). The banner is "
		+ "procedural -- a port of _civFactionBannerCanvas' own composition, "
		+ "driven by the faction id and its colour -- and there is no image slot, "
		+ "no charge vocabulary and no asset-library binding for an authored one.")

## v3 CIVIL ▸ TERRITORIES: recompute, provinces, the territory brush, and the
## linked notes for the two entity kinds a territory is made of.
func _build_territories() -> void:
	_territories_body = DccWidgets.category(self, "Territories", categories)
	_fill_territories(_territories_body)

func _fill_territories(parent: Control) -> void:
	## **Two of these rows stopped being gaps when SG-02 shipped and nobody
	## moved them** (found 2026-08-24 by driving the dock rather than reading
	## it). Both tooltips asserted, in the shipped build, that no `#[func]`
	## re-runs territory or provinces -- and `civ_recompute()` re-derives
	## *both*. They are shortcuts onto that one call now, exactly the shape the
	## bake pass used for the tool-options bar's dead copy of "Bake ALL levels":
	## one owner of the action, two ways in, no second implementation to drift.
	var pol := DccWidgets.section(parent, "Recompute")
	var recalc := DccWidgets.action(pol, "Recalculate territories", _recompute_civ)
	recalc.disabled = not bridge.has_world
	recalc.tooltip_text = ("The reference's territory recompute. Runs Settlements ▸ Recompute "
		+ "civilisation, which re-derives the whole civ layer downstream of the settlement "
		+ "list — territory included — against the current terrain and the current settlements, "
		+ "hand-dropped and hand-edited ones kept. It does NOT re-place settlements; only "
		+ "Generate does that. Painting a claim by hand stays available too — the Territory "
		+ "tool in the TOOLS block above.")
	var gen_prov := DccWidgets.action(pol, "Generate provinces", _recompute_civ)
	gen_prov.disabled = not bridge.has_world
	gen_prov.tooltip_text = ("The reference's province generator. Same one call: Recompute "
		+ "civilisation rebuilds the province partition and reports how many it produced. "
		+ "Their map tint is a separate switch — Cartography ▸ Political display.")
	DccWidgets.note(pol,
		"The Territory brush and its radius are in the TOOLS block at the top of "
		+ "this dock; arming it puts the radius in the tool options bar.")

	_build_influence(parent)

	_fill_knowledge(parent, bridge.provinces())

	## Wired 2026-09-02 (`LARGE_ITEM_RULINGS.md`'s civ-authoring ruling, stage
	## 5 of 5). It sat disabled in "Not built" saying "there is no
	## civ_clear_territory #[func], so there is no way to leave the claim map
	## empty"; there is one now, so the control moves into the Recompute
	## section beside the two buttons that rebuild what it empties, rather than
	## staying in a section named for what it no longer is.
	##
	## Destructive and irreversible, so it confirms first
	## (`_confirm_destructive`) -- the same shape the timeline's own overwrite
	## guard uses, and the reference's own handler does exactly this
	## (`if(civTerritory && civTerritory.some(...) && !confirm(...)) return;`,
	## reference 26665), skipping the prompt when there is nothing to lose.
	var clear_ter := DccWidgets.action(pol, "Clear territory", _clear_territory)
	clear_ter.disabled = not bridge.has_world
	clear_ter.tooltip_text = ("Empties the claim map: both the computed borders assign_territory "
		+ "derived from the capitals and every dab of hand-painted territory, plus the provinces "
		+ "cut out of them. Settlements, roads and the timeline are untouched.\n\n"
		+ "Not undoable. Recalculate territories re-derives the computed borders from the "
		+ "capitals; erased paint is gone for good.")

	var gaps := DccWidgets.section(parent, "Not built")
	## `GUI_GAP_REGISTER.md` §42's Not-built anatomy: the noun, the blocker
	## named specifically, and what does exist instead.
	DccWidgets.note(gaps,
		"Historical occupation over time  ·  blocked on the timeline\n"
		+ "The timeline records a settlement snapshot per year, not a per-year "
		+ "ownership grid, so there is nothing to scrub a border against. That is "
		+ "timeline work rather than territory work (GUI_GAP_REGISTER.md CV-23's "
		+ "third quantity).\n"
		+ "Borders, claims and influence at the present year are built: "
		+ "Borders & influence, above.")

## `GUI_GAP_REGISTER.md` **CV-23**: borders, claims and influence as three
## separate quantities rather than one plurality-owner-per-cell grid.
##
## **Behind a button, and that is the design, not a shortcut.** The influence
## field is not stored anywhere: `CivData` keeps only `assign_territory`'s
## `i32` owner grid, because a per-cell influence field is 16 bytes a cell --
## 1.07 GB at this port's 8192² ceiling, the same objection `civ_continents`
## already records. So the engine rebuilds it on demand, reads it, and drops
## it, exactly the way `wildlife_regions` works. Refilling this section on
## every dock rebuild would run one Dijkstra per capital each time a place is
## renamed; asking for it is the honest surface for a computation that costs
## something.
func _build_influence(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Borders & influence")
	var run := DccWidgets.action(sec, "Analyse contested borders", _analyse_influence)
	run.disabled = not bridge.has_world
	run.tooltip_text = ("Rebuilds the cost-distance influence field the territory pass computes "
		+ "and discards, and reports it three ways: how far each faction's capitals actually "
		+ "reach (influence), which rival comes closest to every cell (claims), and which pairs "
		+ "of factions genuinely meet and how evenly (borders). Nothing is retained -- the field "
		+ "is dropped before the numbers come back, and the reading reports what it cost. "
		+ "The map view of the same field is Layers ▸ Civilization ▸ Contested borders.")
	_influence_body = VBoxContainer.new()
	_influence_body.add_theme_constant_override("separation", 4)
	_influence_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	sec.add_child(_influence_body)
	DccWidgets.note(_influence_body,
		"Not run yet. The same field draws as a map layer under "
		+ "Layers ▸ Civilization ▸ Contested borders.")

func _analyse_influence() -> void:
	if _influence_body == null or not is_instance_valid(_influence_body):
		return
	_clear_body(_influence_body)
	var d := bridge.civ_territory_influence()
	if d.is_empty():
		DccWidgets.note(_influence_body,
			"No territory to analyse: territory is projected from capitals, and this world "
			+ "has none (or carries no civilisation layer at all, which is every loaded save).")
		return
	var owned := int(d.get("owned_cells", 0))
	var frontier := int(d.get("contested_cells", 0))
	var pct := (100.0 * float(frontier) / float(owned)) if owned > 0 else 0.0
	var margin := int(round(100.0 * (1.0 - float(d.get("frontier_threshold", 0.88)))))
	var head := "%s owned land cells; %s of them (%.1f%%) sit on a frontier -- a rival faction reaches them within %d%% of the owner's own effective cost-distance. Mean contest %.3f (0 = uncontested, 1 = evenly split)."
	DccWidgets.note(_influence_body, head % [
		FactionRosterWindow._thousands(owned), FactionRosterWindow._thousands(frontier), pct,
		margin, float(d.get("mean_contested", 0.0))])

	var by_f := DccWidgets.group(_influence_body, "Per faction")
	for row in d.get("factions", []):
		var r: Dictionary = row
		DccWidgets.note(by_f, "%s -- %s cells, %s on a frontier; mean reach %.1f, mean contest %.3f"
			% [String(r.get("name", "?")), FactionRosterWindow._thousands(int(r.get("cells", 0))),
				FactionRosterWindow._thousands(int(r.get("frontier_cells", 0))),
				float(r.get("mean_influence", 0.0)), float(r.get("mean_contested", 0.0))])

	var borders: Array = d.get("borders", [])
	var by_b := DccWidgets.group(_influence_body, "Contested borders")
	if borders.is_empty():
		DccWidgets.note(by_b,
			"No two factions meet on this world: every frontier cell's nearest rival is too "
			+ "far to count, or there is only one faction holding ground.")
	else:
		for row in borders:
			var r: Dictionary = row
			DccWidgets.note(by_b, "%s ↔ %s -- %s frontier cells, mean contest %.3f"
				% [String(r.get("a_name", "?")), String(r.get("b_name", "?")),
					FactionRosterWindow._thousands(int(r.get("cells", 0))),
					float(r.get("mean_contested", 0.0))])

	var foot := "Built on demand and dropped: %.1f MB of per-cell working set for this world, at its peak, held for the length of the call and retained nowhere. Four fifths of that is what generating this world's territory already spent."
	DccWidgets.note(_influence_body, foot % (float(d.get("transient_bytes", 0)) / 1048576.0))

func _build_settlements() -> void:
	var cat := DccWidgets.category(self, "Settlements", categories)
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
	## v3 folds Population into Settlements ▸ § Properties ▸ Population rather
	## than leaving it a category of its own -- the totals are a fact *about*
	## the roster above, and a category whose whole content is two summary
	## lines was the emptiest row on the rail.
	_build_population(cat)
	_build_settlement_vault(cat)

func _fill_settlements(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Roster")
	var settlements := bridge.settlements()
	if settlements.is_empty():
		DccWidgets.note(sec, "No settlements -- generate a world first (World ▸ Generate).")
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
	## **The outcome now also reaches the status bar** (2026-09-01).
	## `_recompute_note` lives in CIVIL ▸ Settlements ▸ Recompute, but this
	## same handler is what CIVIL ▸ Territories' own "Recalculate territories"
	## and "Generate provinces" buttons call -- and these are accordion
	## categories, one open at a time, so pressing either of those wrote the
	## only feedback into a panel that was necessarily closed. That included
	## the failure branch: a refusal reported nowhere the presser could see.
	## One line onto a surface that is visible from every category, rather
	## than a second note node per button to keep in sync.
	var outcome := ""
	if not bool(r.get("ok", false)):
		outcome = "Not recomputed. %s" % String(r.get("reason", "Unknown reason."))
	else:
		outcome = ("Recomputed in %.1f s: %d settlements kept, %d ways and %d "
			+ "provinces rebuilt against the current terrain.") % [
			float(r.get("ms", 0.0)) / 1000.0, int(r.get("settlements", 0)),
			int(r.get("ways", 0)), int(r.get("provinces", 0))]
	if _recompute_note != null and is_instance_valid(_recompute_note):
		_recompute_note.text = outcome
	app.set_status("hint", outcome, "text" if bool(r.get("ok", false)) else "accent")
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

## One Yes/No gate for the three destructive civ operations, so "confirm
## before an irreversible action" is decided once rather than three times.
##
## `ConfirmationDialog` built by hand and `add_child`ed onto `app`, which is
## this shell's own established pattern (`_tl_show_confirm` above, `app.gd`'s
## `open_storage_locations`, `cartography_workspace.gd`'s
## `_prompt_label_name`).
##
## `skip_when_empty` is the reference's own behaviour, not a shortcut: all
## three of its Clear handlers check whether there is anything to lose first
## and only then call `confirm()`, with the comment "skipped when there's
## nothing to lose, so an empty map's Clear buttons stay instant" (reference
## 26662). A prompt asking permission to delete nothing trains people to
## dismiss prompts.
func _confirm_destructive(title: String, body: String, ok_text: String, skip_when_empty: bool, on_confirm: Callable) -> void:
	if skip_when_empty:
		on_confirm.call()
		return
	var dlg := ConfirmationDialog.new()
	dlg.title = title
	dlg.dialog_text = body
	dlg.get_ok_button().text = ok_text
	dlg.confirmed.connect(func(): on_confirm.call(); dlg.queue_free())
	dlg.canceled.connect(dlg.queue_free)
	app.add_child(dlg)
	dlg.popup_centered()

## The reference's Auto-populate world (`#civAutoPopulateBtn`), stage 1 of the
## civ-authoring ruling's five.
##
## Same progress affordance as `_recompute_civ`, and for the same reason:
## `civ_populate` is a synchronous engine call with no progress signal, so the
## honest minimum is to relabel, disable, let two frames actually paint that,
## then block. Two frames rather than one because a single `process_frame`
## await returns before the redraw has reached the screen.
func _populate_world() -> void:
	_confirm_destructive(
		"Re-place every settlement?",
		"Auto-populate replaces all %d settlement%s, their names, tiers, populations and "
			% [bridge.settlements().size(), "" if bridge.settlements().size() == 1 else "s"]
			+ "per-place notes, and drops the recorded timeline with them.\n\n"
			+ "Faction names and colours, and hand-painted territory, are kept. This cannot be undone.",
		"Re-place",
		bridge.settlements().is_empty(),
		func(): _populate_world_now())

func _populate_world_now() -> void:
	var b := _populate_btn
	if b != null and is_instance_valid(b):
		b.text = "Populating…"
		b.disabled = true
		await get_tree().process_frame
		await get_tree().process_frame
	var r: Dictionary = bridge.civ_populate()
	if b != null and is_instance_valid(b):
		b.disabled = false
		b.text = "Auto-populate world"
	var outcome := ""
	if not bool(r.get("ok", false)):
		outcome = "Not populated. %s" % String(r.get("reason", "Unknown reason."))
	else:
		outcome = ("Populated in %.1f s: %d settlements placed, %d ways and %d provinces "
			+ "built around them.") % [
			float(r.get("ms", 0.0)) / 1000.0, int(r.get("settlements", 0)),
			int(r.get("ways", 0)), int(r.get("provinces", 0))]
	if _populate_note != null and is_instance_valid(_populate_note):
		_populate_note.text = outcome
	app.set_status("hint", outcome, "text" if bool(r.get("ok", false)) else "accent")
	_after_civ_layer_replaced()

## The reference's Clear places & routes (`#civClearPlacesBtn`), stage 4.
func _clear_places() -> void:
	var n := bridge.settlements().size()
	_confirm_destructive(
		"Clear all settlements?",
		"Removes %d settlement%s and every way, sea lane and journey with them, along with "
			% [n, "" if n == 1 else "s"]
			+ "their territory, provinces and the recorded timeline.\n\nThis cannot be undone.",
		"Clear",
		n == 0 and bridge.roads().is_empty(),
		func(): _clear_places_now())

func _clear_places_now() -> void:
	var r: Dictionary = bridge.civ_clear_places()
	var outcome := "Cleared %d settlement(s), %d way(s), %d sea lane(s) and %d journey(s)." % [
		int(r.get("settlements", 0)), int(r.get("ways", 0)),
		int(r.get("sea_routes", 0)), int(r.get("journeys", 0))]
	if _populate_note != null and is_instance_valid(_populate_note):
		_populate_note.text = outcome + " Auto-populate builds a new world's worth."
	app.set_status("hint", outcome, "text")
	_after_civ_layer_replaced()

## The reference's Clear territory (`#civClearTerrBtn`), stage 5.
func _clear_territory() -> void:
	_confirm_destructive(
		"Clear all territory?",
		"Empties the claim map: the computed borders and every hand-painted dab, plus the "
			+ "provinces cut out of them. Settlements and roads are untouched.\n\n"
			+ "Recalculate territories re-derives the computed borders; erased paint cannot be "
			+ "recovered.",
		"Clear",
		bridge.provinces().is_empty() and not bridge.has_world,
		func(): _clear_territory_now())

func _clear_territory_now() -> void:
	var r: Dictionary = bridge.civ_clear_territory()
	var outcome := "Cleared %d claimed cell(s) -- computed borders and paint both." % int(r.get("cleared_cells", 0))
	app.set_status("hint", outcome, "text")
	_after_civ_layer_replaced()

## What every stage that replaces or empties the civ layer has to do
## afterwards, in one place rather than four copies: the territory wash and
## the pins/roads overlay both moved, the roster readouts have to rebuild, and
## both SG-01 staleness readouts have to update immediately rather than up to
## a second later -- a button that just changed the world is the one place a
## lagging badge reads as "it didn't work". Lifted verbatim out of
## `_recompute_civ`'s tail, which is where all four lines came from.
func _after_civ_layer_replaced() -> void:
	app.viewport.territory_view.texture = bridge.territory_texture()
	_on_civ_edited()
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
	_build_regional_population(parent)

## `PARITY_AUDIT.md` §23 F13: `civ_regional_population` (`estimateRegionalDensityKm2`
## via the reference's own `_civRegionalPopulation`, HTML line 23297) --
## ported, tested, and until now called by nothing. A DIFFERENT figure from
## the agrarian total above (`cartalith_civ::estimate_regional_density_km2`'s
## own doc comment: "additive to carrying capacity k, never feeds back into
## it"), so it gets its own row rather than replacing that one -- the
## reference draws both as separate concepts (the agrarian ceiling
## settlements are sized against vs. the conservative regional-average `Pop
## density` layer), even though only the agrarian total had a live readout
## in this port before this pass.
##
## A button, not an automatic readout: the engine call recomputes water
## access, lithology, soil and carrying capacity fresh every time (none of
## them are retained on `CivData` the way agrarian density's own input is --
## see `civ_regional_population`'s own doc comment), so it costs a real
## fraction of what `Recompute civilisation` costs. Auto-running it on every
## roster refresh (a place edit) would put that cost on a path that today
## costs nothing.
## **Reaches `bridge`, not `bridge.world_gen`** (2026-09-01). Both halves of
## this pair used to test `bridge.world_gen.has_method(...)` and then call
## `bridge.world_gen.civ_regional_population()` directly, past the wrapper
## `engine_bridge.gd` already carries -- which meant the shell's one
## missing-binding warning never fired for this call, and the wrapper read
## as dead code. The wrapper performs the same guard and answers `{}`, so
## the local test is not merely redundant, it is the deviation.
##
## The button is drawn unconditionally now: `_build_pop_estimate` above only
## reaches here when `civ_agrarian_regional_total()` came back non-empty,
## which needs a world, so there is no state in which this row appears over
## nothing. A binary too old to answer says so when pressed, in the handler.
func _build_regional_population(parent: Control) -> void:
	var btn := DccWidgets.action(parent, "Compute regional population (persons/km²)…",
		_on_compute_regional_population)
	btn.tooltip_text = ("The reference's OTHER modeled-population figure (currentPopulationDensity's "
		+ "persons/km² field, integrated over land) -- a conservative regional average that includes "
		+ "waste ground, as opposed to the settled-core productivity the total above is grounded in. "
		+ "Recomputed fresh on every press; not automatic.")
	_regional_pop_note = DccWidgets.note(parent, "")

func _on_compute_regional_population() -> void:
	if _regional_pop_note == null or not is_instance_valid(_regional_pop_note):
		return
	var r: Dictionary = bridge.civ_regional_population()
	if r.is_empty():
		## Two different causes, and the old text asserted the wrong one
		## whenever the binding was the missing half rather than the world.
		_regional_pop_note.text = ("No world yet." if not bridge.has_world
			else "This build's engine has no civ_regional_population binding -- the "
				+ "native library is older than this shell.")
		return
	var total := int(r.get("total", 0))
	var land := int(r.get("land_km2", 0))
	var claimed := int(r.get("claimed", 0))
	var text := "Regional average model ≈ %s people over the same %s km²." \
		% [FactionRosterWindow._thousands(total), FactionRosterWindow._thousands(land)]
	if claimed > 0:
		text += " %s of that falls inside painted faction territory." % FactionRosterWindow._thousands(claimed)
	_regional_pop_note.text = text


## The reference's own settlement-population operations.
##
## **Both were wired 2026-09-02** (`LARGE_ITEM_RULINGS.md`'s civ-authoring
## ruling, stages 1 and 4 of 5), so they live in their own **Populate**
## section now rather than under "Not built", which keeps only the settlement
## diagnostics overlay it still honestly describes.
##
## The two notes this replaces are worth keeping as history, because both were
## true when written and both stopped being true in the same direction: first
## "`generate()` places the world as part of the one-shot chain, so there is
## no separate populate step" (SG-02 made the *downstream* half re-entrant in
## 2026-08-24, and this ruling made placement itself re-entrant), then "nor
## any parameter for the three counts — `params.rs` exposes no civ parameters
## at all" (the civ `PARAMS` group is seven rows, and three of them are
## placement dials Auto-populate reads).
func _build_settlement_gaps(parent: Control) -> void:
	var pop_sec := DccWidgets.section(parent, "Populate")
	_populate_btn = DccWidgets.action(pop_sec, "Auto-populate world", _populate_world)
	_populate_btn.disabled = not bridge.has_world
	_populate_btn.tooltip_text = ("The reference's #civAutoPopulateBtn. Re-places every settlement "
		+ "from the current suitability field and rebuilds everything under it — roads, sea lanes, "
		+ "territory, provinces, economy — without re-rolling the terrain. This is how the seven "
		+ "civilisation parameters in File ▸ New world ▸ Generation become adjustable: move one, "
		+ "press this, see the world it makes.\n\n"
		+ "Replaces every settlement, so names, tiers, populations and per-place notes are all "
		+ "new; the recorded timeline is dropped with them. Faction names, cultures and colours "
		+ "survive, and so does hand-painted territory. Not undoable.\n\n"
		+ "Seconds, not milliseconds, on the main thread — the same cost as Recompute "
		+ "civilisation, which measured about 1.0 s at 512², 1.6 s at 1024² and 4.2 s at 2048².")
	_populate_note = DccWidgets.note(pop_sec,
		"Keeps the terrain and re-derives the people on it. To re-roll the land as well, "
		+ "use World ▸ Generate.")
	_clear_places_btn = DccWidgets.action(pop_sec, "Clear places & routes", _clear_places)
	_clear_places_btn.disabled = not bridge.has_world
	_clear_places_btn.tooltip_text = ("The reference's #civClearPlacesBtn. Empties the settlement "
		+ "list and everything indexed by it: per-place notes, trade balances, provinces and the "
		+ "territory derived from the capitals. Ways and journeys go too — the reference's own "
		+ "rule, since a route network with no places to connect is meaningless.\n\n"
		+ "The recorded timeline is dropped: every snapshot refers to settlements that no longer "
		+ "exist. Not undoable — Auto-populate derives a new set rather than restoring these.")

	var sec := DccWidgets.section(parent, "Not built")
	var diag := DccWidgets.action(sec, "Settlement diagnostics overlay", func(): pass)
	diag.disabled = true
	## Rewritten 2026-08-31. The previous wording ended "the crate has no
	## consumer at all", which was false and had been for a week:
	## `cartalith-civ` depends on `cartalith-urban` (its Cargo.toml) and
	## `urban_adapter.rs` uses it, `cartalith-godot/src/urban_bridge.rs`
	## consumes that adapter, and milestones 1-7, 8a, 12 and 17a are done.
	## The real blocker is narrower and per-line, so the reason now names the
	## card's four lines and which milestone owes each -- checked against
	## `URBAN_MORPHOLOGY_SCOPE.md`'s milestone headings, not remembered.
	diag.tooltip_text = "The reference's #civDiagnosticsChk (drawCivLayer §2.6). Per settlement it draws a SITE_WM×SITE_HM footprint box and a fact card of at most three lines: specialisation + _umWallSpec's wall rung on the first, _umSiteProfile's river classification on the second, and -- only when a layout is already in _umModelCache -- bridge/ford/harbour validity on the third. Two of those five values this port can produce: SITE_WM/SITE_HM and um_wall_spec are both ported (cartalith-civ's urban_adapter and military), and cartalith-urban IS consumed, by that adapter and through it by cartalith-godot's urban_bridge. The other three have nothing behind them: settlements carry no specialisation, _umSiteProfile is unported because its own consumers are unbuilt, harbours/bridges/fords are URBAN_MORPHOLOGY_SCOPE.md milestone 9, the wall builder is milestone 10 and districts are 13 -- and _umModelCache is out of scope for every milestone (this port keys layouts GDScript-side instead). So the overlay would draw a box, a rung and three blanks, which is worse than not drawing it. Blocked on urban milestones 9, 10 and 13. PARITY_AUDIT.md §5 item 13."
	DccWidgets.note(sec,
		"Urban morphology layouts remain a separate unported subsystem "
		+ "(URBAN_MORPHOLOGY_SCOPE.md, Phase 5, in progress) -- that is what this one row is "
		+ "waiting on. Every other civilisation dial IS exposed: the biome carrying-capacity "
		+ "residual, village seeding, the imperial-seat (metropolis) tier, the post-collapse "
		+ "recovery phase, the faction count and the two settlement-placement dials, all in "
		+ "File ▸ New world ▸ Generation, and all re-runnable from Auto-populate above.")

## `04-left-dock.md` §6b's `hCivPlaceSel`: "selects the place, arms inspect,
## and opens the right dock." `arm_tool()` no-ops when Inspect is already
## armed (`app.gd`'s own early return), so this costs nothing on the common
## path and only matters when the row is clicked with Territory, Way or
## Route still armed from a previous action.
func _settlement_row(parent: Control, data: Dictionary, index: int) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)
	var text := "%s -- %s, pop %d" % [data.get("name", "?"), String(data.get("kind", "?")).capitalize(), int(data.get("population", 0))]
	var b := DccWidgets.action(row, text, func():
		_selected_index = index
		app.arm_tool("inspect")
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
func _build_population(parent: Control) -> void:
	_population_body = VBoxContainer.new()
	_population_body.add_theme_constant_override("separation", 0)
	_population_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(_population_body)
	_fill_population(_population_body)

## v3 CIVIL ▸ SETTLEMENTS ▸ `§ Vault note`, and it is the one place in v3's
## civic half where the Markdown vault is fully backed: a settlement is an
## `EntityKind` in `cartalith-vault`, keyed by its `tid`, which survives a
## rename and a `civ_recompute()`.
##
## v3's own rule for this band -- *"numbers the model reads stay app-native,
## everything descriptive is a heading in the settlement's note"* -- is
## already how this port is built: the place editor writes name/class/polity/
## population, and the vault holds the prose.
##
## Three of v3's five rows exist and three do not, and this says which is
## which rather than drawing five and letting three of them do nothing.
func _build_settlement_vault(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Linked notes")
	var info := bridge.vault_info()
	var root := String(info.get("root", ""))
	if root.is_empty():
		DccWidgets.note(sec,
			"No vault connected. A vault is any folder of .md files -- Obsidian is "
			+ "one, and nothing here requires it. Cartalith reads on demand and "
			+ "writes only on an explicit, previewed action; it never rewrites a "
			+ "note's body.")
	else:
		DccWidgets.note(sec, "Vault: %s" % root)
	var open := DccWidgets.action(sec, "Markdown vault…", func(): app.open_vault_overview(), true)
	open.tooltip_text = "Connect or re-link a vault folder, browse it, and see every knowledge link in this world -- settlements, provinces and continents together."
	DccWidgets.note(sec,
		"A settlement's own notes are on its place editor, under Knowledge -- "
		+ "click a settlement above, then ✎. The link is keyed to the "
		+ "settlement's tid, so it survives a rename and a recompute.")

	## `GUI_GAP_REGISTER.md` **VA-02**, built 2026-08-25.
	var n_templates := bridge.vault_templates().size()
	DccWidgets.note(sec,
		("A settlement with no note yet can be created from one of your own "
		+ "templates, in the vault panel: %s. Cartalith copies the template "
		+ "verbatim with the settlement's name substituted, at "
		+ "Settlements/{name}.md, and refuses if that path already exists -- it "
		+ "never overwrites a note. Author-field population is separate and "
		+ "previewed: OnlyIfEmpty by default, reporting what it skipped.")
		% ("%d found in this vault" % n_templates if n_templates > 0
			else "none found yet -- a template is any .md with \"template\" in its path"))

	## `GUI_GAP_REGISTER.md` **VA-01**, built 2026-08-25. What was a "Not
	## built" note is now a live readout: the index answers both halves, and
	## the per-entity rows are on the place editor beside the link that
	## produced them.
	var idx := bridge.vault_backlink_stats()
	if bool(idx.get("built", false)):
		DccWidgets.note(sec,
			("Backlinks are indexed: %d notes, %d links, %d Cartalith blocks. A "
			+ "settlement's own incoming references and unlinked mentions are on its "
			+ "place editor, under Knowledge.") % [
				int(idx.get("notes", 0)), int(idx.get("links", 0)),
				int(idx.get("entities", 0))])
	else:
		DccWidgets.note(sec,
			"Backlinks and unlinked mentions are not indexed for this vault yet "
			+ "(GUI_GAP_REGISTER.md VA-01). Build the index once from the vault "
			+ "panel above and a settlement's place editor shows what points at it; "
			+ "after that a refresh only re-opens the notes that changed.")

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
	## **The faction half of the sentence that used to be here was false, and
	## had been since the roster shipped** (found 2026-09-01). It said
	## "faction- or province-level population aggregation has no binding".
	## `get_factions()` sums `s.pop` over each faction's own settlements and
	## emits it as `population` (`lib.rs`), and the roster window has been
	## drawing that number all along. The PROVINCE half is true and is kept:
	## `get_provinces()` carries `{id, faction, name,
	## capital_settlement_index}` and nothing more, and a settlement's `tid` is
	## its timeline stable id, not a province id -- so there is no membership
	## to group by from this side either.
	var factions := bridge.get_factions()
	if not factions.is_empty():
		var by_faction := DccWidgets.group(sec, "By faction")
		var rows: Array = factions.duplicate()
		rows.sort_custom(_by_population_desc)
		for f in rows:
			var fd: Dictionary = f
			var fpop := int(fd.get("population", 0))
			var share := (100.0 * float(fpop) / float(total)) if total > 0 else 0.0
			var sc := int(fd.get("settlement_count", 0))
			var b := DccWidgets.action(by_faction,
				"%s -- %s over %d settlement%s (%.1f%%)" % [
					String(fd.get("name", "?")), FactionRosterWindow._thousands(fpop),
					sc, "" if sc == 1 else "s", share],
				func(): app.right_dock_ctrl.show_faction(int(fd.get("id", 0))))
			b.alignment = HORIZONTAL_ALIGNMENT_LEFT
			b.tooltip_text = "get_factions()'s own population field -- the sum of this faction's settlements' pop. Opens the faction in the right dock."
	DccWidgets.note(sec,
		"Per-settlement population is real (get_settlements()'s own field) and "
		+ "faction-level aggregation is get_factions()'s. Province-level is the one "
		+ "that has no binding: get_provinces() carries a name, a faction and a "
		+ "capital index, and no settlement belongs to a province in anything that "
		+ "crosses the boundary, so there is nothing here to sum.")

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
		"This is the per-settlement hinterland term (civ_resource_trade_balance). The " +
		"faction-level aggregation is below.")
	_fill_faction_economy(parent)

## `OUTSTANDING_WORK.md` §2.3: `_civFactionAggregates`' resource- and
## density-fed half, "as a *surfaced* readout" -- the row's own note was "the
## aggregate is ported and has three callers; nothing shows this half", and
## this is the surface.
##
## Collapsed by default (`DccWidgets.group(..., false)`), and that is the
## design rather than tidiness: `civ_faction_economy` rebuilds the lithology
## and resource rasters on every call, so opening Economy must not pay for
## them until somebody asks for this specific answer.
##
## The note this replaces claimed the faction-level aggregation was "real
## future scope per ECONOMY_SCOPE.md, not yet computed". Half of that was
## already false when written -- `civ_faction_aggregates` has been ported and
## called since the military bridge landed -- and what was genuinely missing
## was a caller that fed it `pots`/`dens` and showed the result. Tax and the
## five-axis power heuristic really are still unsurfaced, so the note below
## names those two and nothing else.
func _fill_faction_economy(parent: Control) -> void:
	var rows := bridge.civ_faction_economy()
	if rows.is_empty():
		return
	var sec := DccWidgets.section(parent, "By faction")
	var grp := DccWidgets.group(sec, "Territory, food and resources", false)
	var names := bridge.get_factions()
	for r in rows:
		var d: Dictionary = r
		var f := int(d.get("faction", 0))
		var label := "Faction %d" % f
		if f < names.size():
			label = String((names[f] as Dictionary).get("name", label))
		var surplus := float(d.get("food_surplus", 0.0))
		## The sign is the whole point of the pair, so it is said in words
		## rather than left as a leading minus in a run of numbers.
		var verdict := "feeds itself with %s to spare" % FactionRosterWindow._thousands(int(surplus))
		if surplus < 0.0:
			verdict = "short by %s" % FactionRosterWindow._thousands(int(-surplus))
		DccWidgets.note(grp, "%s -- %s km², %s people, %s." % [
			label, FactionRosterWindow._thousands(int(float(d.get("territory_km2", 0.0)))),
			FactionRosterWindow._thousands(int(float(d.get("pop", 0.0)))), verdict])
		var strat: PackedStringArray = d.get("strategic", PackedStringArray())
		var ex: PackedStringArray = d.get("exports", PackedStringArray())
		var im: PackedStringArray = d.get("imports", PackedStringArray())
		DccWidgets.note(grp, "    Strategic: %s.  Exports: %s.  Imports: %s." % [
			"none" if strat.is_empty() else ", ".join(strat),
			"none" if ex.is_empty() else ", ".join(ex),
			"none" if im.is_empty() else ", ".join(im)])
	DccWidgets.note(grp,
		"Food capacity is the agrarian carrying capacity of the faction's own cells; the " +
		"surplus is that against the population actually living on them, so a shortfall means " +
		"a polity that has to import. Strategic resources are catchment means above the " +
		"reference's own 0.4 bar; exports and imports compare that catchment against the world " +
		"mean, with food added by the surplus sign.")
	DccWidgets.note(grp,
		"Tax income and the five-axis power heuristic come out of the same pass and are not " +
		"drawn anywhere yet -- ECONOMY_SCOPE.md owns them.")

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

# -- Knowledge: provinces and continents (Markdown vault) --------------------

## `MARKDOWN_VAULT_INTEGRATION.md` §28 for the two entity kinds that have no
## floating editor of their own. A settlement gets its KNOWLEDGE block inside
## `place_editor_window.gd`; a province and a continent are only ever listed,
## so their affordance is a row in the list.
##
## **Continents live here, in Politics, and that is a judgement worth stating.**
## A landmass is geography, not politics — but this port's continents are
## `civ_continents`' output, they carry the faction holding the most of them,
## and this is the one dock that already reads `CivData`. Splitting them into
## the World workspace would mean a second file reading the same civ layer for
## one list. If a real geography dock appears, they belong there.
func _fill_knowledge(parent: Control, provinces: Array) -> void:
	var sec := DccWidgets.section(parent, "Linked notes")
	DccWidgets.note(sec,
		"Provinces and continents can each be linked to a note in an external Markdown vault " +
		"(any folder of .md files — Obsidian is one, and nothing here requires it). Cartalith " +
		"reads on demand and writes only on an explicit, previewed action.")

	if provinces.is_empty():
		DccWidgets.note(sec, "No provinces — generate a world first.")
	else:
		var pg := DccWidgets.group(sec, "Provinces", false)
		for p in provinces:
			var d: Dictionary = p
			var pid := int(d.get("id", 0))
			var pname := String(d.get("name", "?"))
			_knowledge_row(pg, "province", pid, pname)

	## `id` here is a **rank by area**, not a persistent identity — the
	## engine's `get_continents()` doc comment carries the whole reasoning.
	## The row says so rather than leaving a user to find out when a sculpted
	## land bridge renumbers two landmasses into one.
	var continents := bridge.continents()
	if continents.is_empty():
		DccWidgets.note(sec, "No continents listed. Either no world has been generated, or every " +
			"landmass in this one is below the listing floor — an all-islands world legitimately " +
			"has none, which is a real outcome rather than missing data.")
	else:
		var cg := DccWidgets.group(sec, "Continents", false)
		for c in continents:
			var d: Dictionary = c
			var cid := int(d.get("id", 0))
			var cname := String(d.get("name", "?"))
			_knowledge_row(cg, "continent", cid, cname,
				"%d cells · centre %d, %d" % [int(d.get("cells", 0)), int(d.get("cx", 0)), int(d.get("cy", 0))])
		DccWidgets.note(cg,
			"A continent's id is its rank by area, largest first — it is derived from the height " +
			"field on every generate, not stored. Editing terrain so two landmasses merge will " +
			"renumber them, so every link also remembers the name it was made against.")


func _knowledge_row(parent: Control, kind: String, entity_id: int, label: String, detail: String = "") -> void:
	var summary := bridge.vault_entity_summary(kind, entity_id)
	var n := int(summary.get("link_count", 0))
	var mark := ""
	if n > 0:
		mark = " · %d note%s %s" % [n, "" if n == 1 else "s",
			String(VaultWindow.STATUS_TEXT.get(String(summary.get("status", "")), ""))]
	var text := label + (" — " + detail if detail != "" else "") + mark
	var b := DccWidgets.action(parent, text, func(): app.open_vault(kind, entity_id, label))
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.tooltip_text = "Open the Markdown vault panel scoped to %s." % label


# -- Culture ----------------------------------------------------------------

func _build_culture() -> void:
	_culture_body = DccWidgets.category(self, "Culture", categories)
	_fill_culture(_culture_body)

## `get_cultures()` (`GUI_GAP_REGISTER.md` CV-02, closed 2026-08-25), reached
## without an `engine_bridge.gd` wrapper because this pass does not own that
## file.
##
## Prefers a wrapper if one lands, so the day it does this function keeps
## working and the fallback becomes dead weight to delete rather than a
## second path to reconcile. The direct `world_gen` call underneath is the
## deviation, not the shape to copy: every other reader in this dock goes
## through `bridge`, and this one should too as soon as it can.
func _cultures() -> Array:
	if bridge.has_method("get_cultures"):
		return bridge.get_cultures()
	if bridge.world_gen != null and bridge.world_gen.has_method("get_cultures"):
		return bridge.world_gen.get_cultures()
	return []

## The seven naming cultures as rows, replacing the note that told the user
## the binding did not exist.
##
## **No "generate a world first" empty state, deliberately.** Unlike every
## other category in this dock, these seven rows are real before any
## `generate()`: they are `cartalith_civ::CIV_CULTURES`, compile-time
## constants, and `get_cultures()`'s own doc calls that "the honest answer
## rather than an inconsistency". Only the aggregates need a world, and they
## come back zero -- so the counts line says which world it is counting over
## rather than the panel pretending to be empty.
##
## `terrain_affinity` is `""` for `common` and `imperial` by design (they are
## identity-flavoured, not terrain-themed, and `civ_culture_terrain_fit`
## refuses to invent a verdict for them), so those two rows say so instead of
## showing a blank column.
func _fill_culture(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Profiles")
	var cultures := _cultures()
	if cultures.is_empty():
		DccWidgets.note(sec,
			"This build's engine has no get_cultures() binding -- the native library "
			+ "is older than this shell. Rebuild and re-export before treating the "
			+ "empty list as a missing feature.")
		return

	var placed := 0
	for c in cultures:
		placed += int((c as Dictionary).get("settlement_count", 0))
	if placed > 0:
		DccWidgets.note(sec,
			"%d cultures; %d settlements carry one through the faction that holds them."
			% [cultures.size(), placed])
	else:
		DccWidgets.note(sec,
			"%d cultures. They exist without a world -- CIV_CULTURES is a compile-time "
			% cultures.size()
			+ "table and a culture id survives both a regenerate and a save/load, which "
			+ "is why a linked note below stays attached to the right one. The faction, "
			+ "settlement and population counts are zero until a world is generated.")

	for c in cultures:
		var d: Dictionary = c
		var affinity := String(d.get("terrain_affinity", ""))
		var terrain := affinity.capitalize() if not affinity.is_empty() else "no terrain theme"
		var fc := int(d.get("faction_count", 0))
		var detail := "%s · %d faction%s" % [terrain, fc, "" if fc == 1 else "s"]
		if fc > 0:
			detail += " (%s) · %d settlements · %s people" % [
				String(d.get("factions", "")), int(d.get("settlement_count", 0)),
				FactionRosterWindow._thousands(int(d.get("population", 0)))]
		_knowledge_row(sec, "culture", int(d.get("id", 0)), String(d.get("name", "?")), detail)

	DccWidgets.note(sec,
		"A culture is set per FACTION, in the roster window's own Culture picker; "
		+ "a settlement takes its faction's and has no override of its own, which "
		+ "is why the counts above are counts of factions and of what they hold. "
		+ "Three things read it: the settlement name pool (_civSettleName), the "
		+ "roster's Territory fit verdict, and faction relations -- two factions "
		+ "sharing a culture score one point of affinity toward each other "
		+ "(relations.rs), which is what puts a culture in Relationships below.")
	var roster := DccWidgets.action(sec, "Which faction has which culture → Faction roster…",
		func(): app.open_faction_roster())
	roster.alignment = HORIZONTAL_ALIGNMENT_LEFT
	roster.tooltip_text = "The roster window's inspector carries the per-faction Culture picker this category counts over."

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

## `open_timeline_category()` stood here until 2026-09-01 and had no caller.
## Its own doc named one twice -- `app.gd`'s timeline strip, which pressed
## this dock's Politics accordion from a button the 2026-08-31 strip rewrite
## removed. `app.gd`'s §10 header records that removal and why: the strip that
## replaced it is a transport, a speed pill group, six layer toggles and a
## scrub track, not a pointer at this panel. Deleted rather than kept as a
## named entry point, because the argument for keeping it was ownership of the
## string "Politics", and its own body already conceded that
## `Workspace.open_category("Politics")` -- which every cross-domain jump in
## this shell uses -- is the shared version of the same search.

## v3 CIVIL ▸ POLITICS: *"Political change over time · #civTlAddYearBtn"*.
##
## The recorded years, the scrubber, playback and the existence filters -- the
## timeline as a *record of political change*, which is what it actually is
## here: `civ_add_year` snapshots which settlements exist, and going to a year
## reloads that snapshot's territory.
##
## **v3 calls the scrubber program scope, and it stays here anyway.** Its
## reasoning is sound ("time is not a domain; every domain reads the current
## year") and the shell already reserves a bottom strip for it
## (`dcc_shell.gd`'s `timeline_bar`). But that strip is one fixed-height HBox
## row with no space for a year-pill list, an add-year field and three filter
## checkboxes, and `TIMELINE_SCOPE.md` §4's standing instruction is to build a
## dedicated panel rather than risk the wrong region. Moving it is a shell-
## frame change, not a menu change, so it is out of this pass's scope --
## recorded in `GUI_GAP_REGISTER.md` CV-24 rather than half-done.
func _build_timeline() -> void:
	var cat := DccWidgets.category(self, "Politics", categories)
	_tl_body = cat
	## The two `bridge.generation_finished`/`bridge.world_loaded` connections
	## that used to live here are now `_build()`'s single pair, calling
	## `_on_world_changed()` -- which calls `_tl_on_world_changed()` below
	## alongside the other categories. For a long time this was the ONLY
	## subscriber in the file, which is exactly why the rest of the dock never
	## refreshed (RF-01); one connection point makes that hard to repeat.
	_rebuild_timeline()

## v3 CIVIL ▸ SIMULATION: collapse and recovery. Its own category because it
## is a *model*, not a record -- v3's own footnote: "writes one timeline entry
## per step: history, never the live editable world."
func _build_simulation() -> void:
	_sim_body = DccWidgets.category(self, "Simulation", categories)
	_rebuild_timeline()

# -- CIVIL ▸ Landmarks --------------------------------------------------------
#
# `design/landmark-generation/LANDMARK_UI_DESIGN.md`, built. This replaces v3's
# `Points of interest` category and the "Not built" stub that stood in it
# (`_build_poi`, deleted with this block) -- §1.1: *"the existing v3 category
# Points of interest, renamed, with its 'Not built' stub replaced."*
#
# **Why CIVIL and not WORLD**, restated here because it is measured rather than
# tasteful and the next reader will be tempted the same way §1.2 was: a WORLD
# parameter row calls `_mark_stale_from` then `_regenerate_live`
# (`world_workspace.gd:1113`, `:1140`), which calls `bridge.generate()` -- the
# whole world from stage 01, every time, because `generate()` is monolithic. A
# landmark cap slider on that path would re-run tectonics, erosion, hydrology
# and climate to move Waterfall from 40 to 41. CIVIL's recompute is a *button*
# (`_build_recompute` below), and a button-driven pass is also what makes the
# cap-vs-placed readout possible at all: "11 placed" is a fact about the last
# run, and a panel that silently re-ran on every drag would have no last run to
# report.
#
# **Every value here comes from the bridge and nothing is hardcoded.** The type
# vocabulary, the families, the classes, the caps and the funnels are all read
# from `landmark_*` on `EngineBridge`. If those methods are absent the category
# draws a disclosed empty state (`_lm_not_built`) rather than a list this file
# invented -- the same rule `menus.gd:233`'s `_todo` follows, one level up.
#
# **Cross-checked against GUI replacement stage 4's own authority,
# `design/dcc-environment-2026-08-31/spec/04-left-dock.md` §6a, 2026-09-01.**
# The two documents describe the same feature independently -- different
# section numbers, same ladder (`0,1,2,3,5,8,12,20,30,50,80,120,200`), same
# crowding range (0.25-2.00), same class set (CON/REG/LOC/CUL), same "types
# compete" toggle, same family bulk arm-all/off, same five-integer funnel --
# and every place they could conflict on a NUMBER, this file already reads
# the number from the bridge rather than copying either document's, which is
# what let one specific check settle clean: §6a.1 prints example class radii
# of 120/34/12/8 km, this port's real default is 200/34/10/6 km
# (`landmark.rs::DEFAULT_CLASS_RADIUS_KM`), and the two agree only on
# Regional -- because §6a's own table is illustrative UI-mockup arithmetic,
# not this engine's ground truth, and `_lm_class_radius()` below was already
# built to read the live value instead of either spec's number. Nothing in
# this pass changed a value for that reason. Two header-level rules this
# check found needed a decision of their own:
#
# - §6's "Landmarks is the floor" (closing whichever CIVIL category is open
#   falls back to Landmarks, never to nothing) is real accordion behaviour,
#   not styling, and this port's 14-category CIVIL accordion needs a floor
#   even more than the design's own four-wide one did -- an all-collapsed
#   CIVIL dock is a worse empty state than the prototype ever had to defend
#   against. Built: `_lm_enforce_floor()`, wired once at the foot of
#   `_build()` after every category in the shared `categories` group exists.
# - §6's `{{ lmCatCount }}` (an "N armed · N on the map" readout ON the
#   category header, visible before the accordion opens) is NOT built.
#   `DccWidgets.category()` returns only the body VBox and has no parameter
#   for one (its own doc comment names a `header_extra` the function it sits
#   above does not actually take), and the entry it appends to `categories`
#   reuses one `title` field as both the button's redraw source AND the
#   exact string `Workspace.open_category()` matches to jump here from
#   outside this file -- so writing a live count into it would silently
#   break every cross-domain "open Landmarks" jump the moment a sibling
#   category's own toggle next repainted this button from that same
#   (now-wrong) title. That factory is not this pass's to edit; disclosed
#   here rather than shipped as a readout that goes stale the instant a
#   neighbour is clicked.
#
# Places the design and what is buildable disagreed are marked **DIVERGENCE**
# in place, with the reason, rather than left for a reader to find by holding
# the artboard next to the running dock.

## §2.4's cap ladder, verbatim: `off · 1 · 2 · 3 · 5 · 8 · 12 · 20 · 30 · 50 ·
## 80 · 120 · 200`. Thirteen detents on a rounded 1-2-3-5 ladder, because "a
## linear 0-200 slider spends most of its travel in a range nobody wants" and
## the step from 1 to 2 deserves as much travel as the step from 120 to 200 --
## for a Continental-class type one versus two is the design of the world and
## 120 versus 200 means nothing.
##
## The slider therefore carries the ladder INDEX (0..12), never the cap, and
## `_lm_refresh_row()` rewrites the readout from this table -- the `fmt` closure
## `DccWidgets.slider()` installs (`dcc_widgets.gd:387`) would otherwise print
## the index. Index 0 is the detented zero stop, which reads `off`.
const LM_LADDER: Array[int] = [0, 1, 2, 3, 5, 8, 12, 20, 30, 50, 80, 120, 200]

## §23's four classes, in §23's own order.
##
## This is the ONE place the panel assumes an order the bridge states
## positionally: `landmark_settings()` returns `class_radius_km` as a
## four-element `Array`, so something has to say which element is which.
## `landmark_set_class_radius()` takes the key, so only the READ is positional
## and a mis-ordered write is not possible.
const LM_CLASSES: Array[String] = ["continental", "regional", "local", "cultural"]
const LM_CLASS_LABEL := {
	"continental": "Continental", "regional": "Regional",
	"local": "Local", "cultural": "Cultural",
}
## Which element of `class_radius_km` the Crowding readout quotes. §4.1's own
## example sentence is about a *regional* landmark, and regional is the
## middle-of-the-road class most of §29's types fall in.
const LM_QUOTED_CLASS := 1

## §2.2's limiting-reason vocabulary: the engine's token -> the word the row
## prints. `at_cap` is the only one drawn in accent, which is what makes §2.2's
## "a panel where nothing is at cap has no accent on any second line" true by
## construction rather than by a rule someone has to remember.
const LM_LIMIT_WORD := {
	"at_cap": "at cap", "spacing": "spacing", "no_terrain": "no terrain",
	"candidates": "candidates", "disarmed": "off", "not_buildable": "not buildable",
	## `not_generated` had a row here until 2026-09-01. `LandmarkLimit` has six
	## variants and no seventh: nothing in `landmark.rs` ever emitted that token,
	## and the case it was written for -- a declared type with no generator --
	## reports `not_buildable`. Dropped rather than kept defensively, because
	## `_lm_limit_word`/`_lm_limit_why` below already answer an unknown token
	## honestly, by echoing it and saying this panel has no wording for it.
}
## The same four rows of §2.2's table, as the tooltip each token carries. Every
## reason other than `at cap` is the panel saying *the cap is not what is
## limiting you*, which is the sentence the owner's brief is entirely about, so
## each one names what to touch instead.
const LM_LIMIT_WHY := {
	"at_cap": "The cap was the binding constraint. Dragging this slider right WILL place more.",
	"spacing": "The exclusion radius rejected the rest. Raising the cap changes nothing -- lower Crowding, or this class's radius.",
	"no_terrain": "Every remaining candidate failed this type's own constraints. The cap is not what is limiting you.",
	"candidates": "The candidate pool ran out before the cap or the spacing did. The world is too small or too coarse for more of these.",
	"disarmed": "Disarmed. The cap is retained and the row says what it was.",
	"not_buildable": "The engine reports no placement rule for this type yet, so it is listed and disabled rather than quietly omitted -- a reader can tell 'unimplemented' from 'hidden'.",
}

## The engine's `limit` token, normalised before it is looked up.
##
## **The deviation this was written for is gone**, and this comment had not
## caught up (re-checked 2026-09-01). It landed because the engine shipped its
## reasons as display words -- `"at cap"`, `"no terrain"`, `"off"` -- against a
## contract that says machine keys, and a bare dictionary lookup missed every
## one in the worst available way: the row still *read* correctly, because the
## fallback echoes an unrecognised token, while §2.2's accent and the whole
## tooltip were silently dead. Found by `_landmark_probe.gd` half E asserting
## the styling rather than the text, against a real 384x288 world.
##
## `LandmarkLimit::as_str()` emits `at_cap` / `spacing` / `no_terrain` /
## `candidates` / `disarmed` / `not_buildable` now, and `landmark.rs`'s
## `limit_tokens_are_the_wire_format_the_shell_keys_off` test pins all six --
## a rename there fails there rather than three files away, under a probe that
## thinks to assert on styling. So this is kept only as a widened door for a
## binary older than that fix, not as a patch over live behaviour.
static func _lm_limit_key(raw: String) -> String:
	return raw.strip_edges().to_lower().replace(" ", "_").replace("-", "_")

## The word the row prints for a reason token, raw or normalised. An unknown
## token is printed verbatim rather than swallowed -- a reason this panel has no
## wording for is still a reason, and dropping it would make the row look as
## though nothing limited it.
static func _lm_limit_word(raw: String) -> String:
	return String(LM_LIMIT_WORD.get(_lm_limit_key(raw), raw.strip_edges()))

## ...and its explanation, which is never blank: a token with no wording says
## so, in place, instead of hovering to nothing.
static func _lm_limit_why(raw: String) -> String:
	var k := _lm_limit_key(raw)
	if LM_LIMIT_WHY.has(k):
		return String(LM_LIMIT_WHY[k])
	if raw.strip_edges().is_empty():
		return "The engine gave no limiting reason for this type."
	return ("This build's engine gave \"%s\" as the limiting reason and this "
		+ "panel has no wording for it; the funnel is the arithmetic it came "
		+ "from.") % raw.strip_edges()

## v3 CIVIL ▸ LANDMARKS (`design/landmark-generation/LANDMARK_UI_DESIGN.md`
## §1.1). Three L3 sections -- PLACEMENT, TYPES, LAST RUN -- and five
## disclosure levels, never six (§3: domain / category / section / family /
## the row's own fold), which is why the class is a badge on the row and not a
## second tree over the family one.
##
## Built open (`04-left-dock.md` §6: `Default civCat = 'landmarks'`) -- see
## `_build_civilizations()`'s own comment for why this moved here, and
## `_lm_enforce_floor()` for the rest of §6's rule.
func _build_landmarks() -> void:
	_landmarks_body = DccWidgets.category(self, "Landmarks", categories, true)
	_fill_landmarks(_landmarks_body)

## `Assets ▸ Landmark types ▸ <family> ▸ Open … in the dock` lands here
## (`menus.gd`'s `_open_landmark_dock`). `select_domain_category()` has already
## switched the domain and opened the category; this opens the family group
## *inside* it, which is the half §9.1 row 20 calls owed for the category jump.
##
## Presses the group's own header rather than flipping `visible`, for the same
## reason `open_category()` presses a category header: the caret text and the
## count line are written by that handler and by ours behind it, and setting
## `visible` directly would leave both saying the opposite of what the panel
## shows. Scrolling the dock down to the group is still owed -- the scroller
## belongs to `dcc_shell.gd`, which this pass does not own.
func open_landmark_family(family: String) -> void:
	open_category("Landmarks")
	var g: Dictionary = _lm_groups.get(family, {})
	if g.is_empty():
		return
	var body: Control = g.get("body")
	var btn: Button = g.get("button")
	if body == null or btn == null or not is_instance_valid(body) or not is_instance_valid(btn):
		return
	if not body.visible:
		btn.pressed.emit()

## `04-left-dock.md` §6, verbatim: *"clicking the open Landmarks header does
## nothing… clicking any other open header collapses back to Landmarks…
## Landmarks is the floor — one category is always open, and it is never
## zero."* `DccWidgets.category()`'s own accordion has no floor: re-clicking
## whichever header is open always leaves the whole group closed, CIVIL
## included, and that all-collapsed state is what this corrects, once, right
## after it happens.
##
## Connected to every CIVIL category button's `pressed` (the loop at the foot
## of `_build()`) rather than only to Landmarks' own: the spec's rule fires
## on re-clicking ANY open header, not just Landmarks', and a listener added
## after `category()`'s own `pressed.connect()` runs SECOND on every press
## (Godot fires a signal's connections in connection order) -- so by the time
## this runs, the built-in toggle has already produced whatever the click
## caused, and this only ever has to detect and correct the one outcome the
## spec forbids, never race it.
##
## Calls `DccWidgets._toggle_category()` directly rather than re-emitting a
## button press, for two reasons: it cannot recurse into this same handler
## (`_toggle_category` sets properties; it never re-fires `pressed`), and it
## never touches `entry["title"]` -- seen live-mutated by
## `Workspace.open_category()`'s own string match, one more reason that field
## has to stay exactly `"Landmarks"` (see the header comment above this
## section for the readout this same constraint already ruled out).
func _lm_enforce_floor() -> void:
	var landmarks_entry: Dictionary = {}
	var any_open := false
	for e: Dictionary in categories:
		var body: Control = e.get("body")
		if body == null or not is_instance_valid(body):
			continue
		if String(e.get("title", "")) == "Landmarks":
			landmarks_entry = e
		if body.visible:
			any_open = true
	if not any_open and not landmarks_entry.is_empty():
		DccWidgets._toggle_category(landmarks_entry, categories)

## Rebuild the category from the bridge. Called on a new world and after a
## settings reset -- NOT from `_rebuild_readouts()`, because a place edit does
## not change a landmark cap and rebuilding here would close whichever family
## group the user had open.
func _lm_rebuild() -> void:
	if _landmarks_body == null or not is_instance_valid(_landmarks_body):
		return
	_clear_body(_landmarks_body)
	_fill_landmarks(_landmarks_body)

func _fill_landmarks(parent: Control) -> void:
	_lm_rows.clear()
	_lm_groups.clear()
	_lm_chips.clear()
	_lm_filter = ""
	var kinds := _lm_kinds()
	if kinds.is_empty():
		_lm_not_built(parent)
		return
	var st := _lm_settings()
	_lm_crowding = float(st.get("crowding", 1.0))
	_lm_radii = (st.get("class_radius_km", []) as Array).duplicate()
	_lm_placement(parent, st)
	_lm_types(parent, kinds, st, _lm_funnel_map())
	_lm_last_run(parent)

## The disclosed empty state. Drawn when the landmark bridge is absent (an
## older cdylib, or a build where the pass has not landed) or when it returns
## no vocabulary at all -- this repository's own silently-empty-output trap,
## which four subsystems have already been bitten by, so the two cases say
## different things rather than sharing one vague sentence.
func _lm_not_built(parent: Control) -> void:
	var has_api: bool = bridge != null and bridge.has_method("landmark_kinds")
	var sec := DccWidgets.section(parent, "Not wired")
	if has_api:
		DccWidgets.note(sec,
			"The landmark bridge is here and it reports no types at all. That is "
			+ "an empty vocabulary, not an empty world: landmark_kinds() is the "
			+ "type table and it does not depend on a world existing, so an empty "
			+ "one means the engine side is not finished rather than that there is "
			+ "nothing to place.")
	else:
		DccWidgets.note(sec,
			"The panel is built; the engine binding is not. EngineBridge carries no "
			+ "landmark_kinds(), so there is no type vocabulary to draw rows from, "
			+ "no caps to write and no pass to run. Every control this category "
			+ "would hold is withheld rather than drawn inert.")
	DccWidgets.note(sec,
		"What it expects, and nothing more: landmark_kinds, landmark_settings, "
		+ "landmark_run, landmark_last_run, landmark_funnels, landmark_headroom, "
		+ "and the five "
		+ "setters (cap, armed, crowding, class radius, cross-type competition). "
		+ "The moment they exist this category fills itself -- no type list, "
		+ "family or class name is written down in this file.")
	DccWidgets.note(sec,
		"Hand-stamped icons are unaffected and still work. They are annotation, "
		+ "not entities, and they live where annotation lives -- a hand-placed "
		+ "mark has no causal chain and no emergent importance, which is what a "
		+ "generated landmark is, so the two lists never merge into one count.")
	var go := DccWidgets.action(parent, "Place an icon → Cartography ▸ Assets & landmarks",
		func(): app.select_domain_category("cartography", "Assets & landmarks"))
	go.alignment = HORIZONTAL_ALIGNMENT_LEFT

# -- § PLACEMENT --------------------------------------------------------------

## §4. One dial named after its effect, one toggle that changes the meaning of
## the whole panel, and the headroom line above both.
func _lm_placement(parent: Control, st: Dictionary) -> void:
	var sec := DccWidgets.section(parent, "Placement")

	## §4.4, the panel-scale answer: "caps total 640 · room for about 210 at
	## this spacing · last run placed 187". The arithmetic that explains
	## everything below it, on screen *before* the user goes hunting for a
	## broken slider. The word `about` is doing real work and stays.
	_lm_head_note = DccWidgets.note(sec, "")
	_lm_refresh_headroom()

	## §4.1. `× 1.00` is arithmetic; `34 km` is a fact about the map the user is
	## looking at. Nobody needs to know what the multiplier multiplies, so the
	## word "Poisson" and the letter `r` appear nowhere in this control.
	##
	## The `on_change`/`on_release` split is `dcc_widgets.gd:349`'s own, and the
	## reference's `tparam()` split before it: every tick updates the readout and
	## the km sentence (cheap, local), and the write to the engine happens once,
	## on release.
	var crowd := DccWidgets.slider(sec, "Crowding", 0.25, 2.00, 0.05, _lm_crowding,
		"", func(v: float): _lm_on_crowding(v),
		"Scales every class's exclusion radius at once, 0.25x (sparse) to 2.00x "
		+ "(dense). This is the control the reason token `spacing` is pointing "
		+ "at: when a type places fewer than its cap because the radius rejected "
		+ "the rest, this moves the number and the cap does not.",
		func(): _lm_write("landmark_set_crowding", [_lm_crowding]))
	_lm_crowd_readout = crowd["readout"]
	_lm_crowd_note = DccWidgets.note(sec, "")
	_lm_refresh_crowding()

	## §4.3, the one toggle that changes the meaning of the whole panel, in
	## nineteen words with no jargon in them.
	DccWidgets.toggle(sec, "Types compete with each other",
		bool(st.get("cross_type_competition", true)),
		func(v: bool): _lm_write("landmark_set_cross_competition", [v]),
		"On: one exclusion field over every type at once, so a dense Physical "
		+ "family genuinely crowds out the Religious one. Off: one field per "
		+ "type, and the families stop interacting. Both are legitimate worlds "
		+ "and the difference is enormous.")
	DccWidgets.note(sec,
		"Off lets a shrine sit beside a waterfall. On keeps every landmark clear "
		+ "of every other one.")

	## §4.2's L5: the four class radii the Crowding dial scales. "One dial for
	## everyone; four for the person who wants a world where regional landmarks
	## crowd and continental ones do not."
	var adv := DccWidgets.advanced(sec, "advanced")
	DccWidgets.note(adv,
		"Exclusion radius per class, before Crowding. Continental landmarks keep "
		+ "the most ground clear and cultural ones the least, which is what makes "
		+ "one world landmark per continent and a shrine every few valleys come "
		+ "out of the same rule.")
	for i in LM_CLASSES.size():
		var ck: String = LM_CLASSES[i]
		var km := _lm_class_radius(i)
		var cname := String(LM_CLASS_LABEL.get(ck, ck))
		DccWidgets.slider(adv, cname, 1.0, 400.0, 1.0, km, " km",
			func(v: float): _lm_on_radius(i, v),
			"The clear ground a %s-class landmark keeps around itself before " % cname.to_lower()
			+ "Crowding scales it. Every type of this class inherits it.",
			func(): _lm_write("landmark_set_class_radius", [ck, _lm_class_radius(i)]))

	## **DIVERGENCE.** No artboard draws a reset, and §9.1's control table does
	## not list one. It is here because the locked bridge contract offers
	## `landmark_reset_settings()` and a panel that never calls it leaves that
	## call dead -- and because `UI_SHELL_DESIGN.md`'s own rule for an L5 fold is
	## "expert dials only, defaults already correct", which is only true if there
	## is a way back to those defaults. One row, in the fold, beside the dials it
	## undoes.
	DccWidgets.action(adv, "Reset every cap and radius to its default", _lm_reset)

# -- § TYPES ------------------------------------------------------------------

## §3. Family is the grouping, class is a badge on the row, and the four class
## chips filter rather than nest -- §29's six families and §23's four classes
## are orthogonal, and nesting both would produce a six-level tree that
## `UI_SHELL_DESIGN.md` forbids outright ("A sixth level means the L2 category
## is wrong and should be split").
func _lm_types(parent: Control, kinds: Array, st: Dictionary, funnels: Dictionary) -> void:
	var sec := DccWidgets.section(parent, "Types")

	## Family order is the ENGINE's first-seen order, not a list written here.
	## `landmark_kinds()` is the vocabulary and this file must not carry a second
	## copy of it that can drift -- the same argument `params.rs`'s own header
	## makes about 58 names, 58 ranges and 58 labels.
	var fams: Array[String] = []
	var by_fam := {}
	var by_class := {}
	var viewshed := 0
	var unbuildable := 0
	for k in kinds:
		var kd: Dictionary = k
		var f := String(kd.get("family", "other"))
		if not by_fam.has(f):
			by_fam[f] = []
			fams.append(f)
		(by_fam[f] as Array).append(kd)
		var c := String(kd.get("class", ""))
		by_class[c] = int(by_class.get(c, 0)) + 1
		if bool(kd.get("needs_viewshed", false)):
			viewshed += 1
		if not bool(kd.get("buildable", true)):
			unbuildable += 1

	DccWidgets.note(sec,
		"%d types in %d families. The four classes are a badge on the row and a "
		% [kinds.size(), fams.size()]
		+ "filter, not a second tree -- a waterfall is Physical by family and "
		+ "regional by class, and those are different questions.")

	## §9.3, and the rule this panel is held to: the viewshed gap shows on the
	## ROW, not in a footnote. The engine computes no visibility term at all --
	## no line of sight, no horizon march, no sky-view factor -- so the types that
	## lean on it carry `[no viewshed]` beside their name and this line says how
	## many there are and what it costs them.
	##
	## The weight quoted here was the research's 0.20 until 2026-08-31, when
	## `BUILD_ANSWERS.md` §3 replaced it with the owner's own formula. Text only:
	## the design asks for the gap to be *stated on the panel* and this port
	## already puts it on every affected row too, which is the stricter of the
	## two, so the placement stands and only the number moved.
	if viewshed > 0:
		DccWidgets.note(sec,
			"%d of them carry [no viewshed]: this engine computes no visibility " % viewshed
			+ "term. Once visibility analysis lands the owner's formula is "
			+ "score = 0.6 × prominence + 0.4 × visible land area inside 30 km, "
			+ "caps unchanged; until then the second half of that score has nothing "
			+ "behind it, and the panel says so on the row rather than presenting a "
			+ "score it cannot honestly compute.")
	if unbuildable > 0:
		DccWidgets.note(sec,
			"%d are listed and disabled: the engine reports no placement rule for " % unbuildable
			+ "them yet. Omitting them would be worse -- a reader who finds no row "
			+ "cannot tell whether the type is unimplemented or simply hidden.")

	## **DIVERGENCE.** §3.1 gives every type row an L5 `+ advanced` fold holding
	## its own constraints (§7's four hydraulic tests for a waterfall, §8's
	## topological one for a pass, a per-type Minimum separation, an importance
	## floor). The locked bridge contract exposes none of them: there is no
	## per-type setter of any kind, only the four per-class radii above. Rather
	## than draw ~50 folds of permanently dead sliders, the gap is stated once,
	## here, where a reader looking for the fold will meet it.
	DccWidgets.note(sec,
		"No row carries a + advanced fold. A type's own thresholds -- minimum "
		+ "drop and flow for a waterfall, corridor strength for a pass, its own "
		+ "minimum separation, its importance floor -- are not exposed by the "
		+ "landmark bridge, so the four class radii under Placement are the only "
		+ "spacing controls that exist.")

	## §3.1's four filter chips, plus `all`. `segment()` is the shell's own
	## "one of a lit set" control (`dcc_widgets.gd:959`) and `set_segment_on`
	## carries the accent wash, so the lit chip reads as lit rather than as a
	## hairline colour change.
	var chips := HBoxContainer.new()
	chips.add_theme_constant_override("separation", 3)
	sec.add_child(chips)
	_lm_chips[""] = DccWidgets.segment(chips, "all", func(): _lm_set_filter(""))
	_lm_chips[""].tooltip_text = "Show every class."
	for ck in LM_CLASSES:
		if not by_class.has(ck):
			continue
		var n := int(by_class[ck])
		var b := DccWidgets.segment(chips, "%s %d" % [_lm_badge(ck), n],
			func(): _lm_set_filter(ck))
		b.tooltip_text = "%d %s-class types. Dims every row that is not one." % [
			n, String(LM_CLASS_LABEL.get(ck, ck)).to_lower()]
		_lm_chips[ck] = b

	## §3.1: the six families, one open at a time by default.
	for fi in fams.size():
		var f: String = fams[fi]
		var title := _lm_pretty(f)
		var body := DccWidgets.group(sec, title, fi == 0)
		var btn := _lm_last_button(sec)
		_lm_groups[f] = {"button": btn, "body": body, "title": title}
		## §3.2: a collapsed group is not silent. `group()`'s own toggle handler
		## rewrites the header text from the title it captured, so this
		## connection is made AFTER it -- signal callbacks fire in connection
		## order, so ours re-appends the counts every time the caret is clicked.
		if btn != null:
			btn.pressed.connect(func(): _lm_refresh_group(f))

		## §3.2's bulk gesture. **DIVERGENCE:** the artboard puts `arm all · off`
		## on the group header row. `DccWidgets.group()` builds that header as a
		## single `Button` with no room for children and this pass may not edit
		## that factory, so the pair is the first row *inside* the group instead.
		## It is a bulk operation, not a second copy of the row control -- the
		## same distinction `Assets ▸ Asset pack ▸ Batch` already draws.
		var bulk := HBoxContainer.new()
		bulk.add_theme_constant_override("separation", 4)
		body.add_child(bulk)
		DccWidgets.chip(bulk, "arm all", func(): _lm_bulk(f, true)).tooltip_text = (
			"Arms every buildable type in this family at its retained cap, or at "
			+ "its default where it has never been set. Thirteen detents six times "
			+ "is not the way to turn a family on.")
		DccWidgets.chip(bulk, "off", func(): _lm_bulk(f, false)).tooltip_text = (
			"Disarms every type in this family. Each row keeps its number and "
			+ "says so, so this is reversible without retyping anything.")

		for k in (by_fam[f] as Array):
			_lm_type_row(body, k, st, funnels)
		_lm_refresh_group(f)

	_lm_apply_filter()

## §2. One slider, two jobs: it arms the type and it sets the cap. The zero stop
## is detented and reads `off`, because "zero waterfalls" and "waterfalls
## disabled" are the same outcome and there is no reason to make the user say it
## twice.
##
## The STORE is still two fields (`{armed, cap}`), which is not a contradiction
## -- it is what stops a papercut this codebase has already met: `ScatterRule`
## (`cartalith-assets/src/scatter.rs:133`) keeps `enabled` and `density` apart
## for the identical reason, that a user who switches something off briefly
## should get their number back. So disarming writes `landmark_set_armed(false)`
## and never `landmark_set_cap(0)`, and the row prints `was 40`.
func _lm_type_row(parent: Control, kind: Dictionary, st: Dictionary,
		funnels: Dictionary) -> void:
	var key := String(kind.get("key", ""))
	if key.is_empty():
		return
	var label := String(kind.get("label", key))
	var fam := String(kind.get("family", "other"))
	var cls := String(kind.get("class", ""))
	var buildable := bool(kind.get("buildable", true))
	var needs_vs := bool(kind.get("needs_viewshed", false))
	var default_cap := int(kind.get("default_cap", 0))
	var caps: Dictionary = st.get("caps", {})
	var armed_map: Dictionary = st.get("armed", {})
	var cap := int(caps.get(key, default_cap))
	var armed: bool = buildable and bool(armed_map.get(key, false))
	var rung := _lm_rung(cap) if armed else 0

	var tip := "%s -- %s class." % [label, String(LM_CLASS_LABEL.get(cls, cls))]
	if needs_vs:
		tip += (" [no viewshed]: this engine computes no visibility term. Once "
			+ "visibility analysis lands the owner's formula is score = 0.6 × "
			+ "prominence + 0.4 × visible land area inside 30 km, caps unchanged; "
			+ "until then this type scores without that second half.")
	if not buildable:
		tip += " " + String(LM_LIMIT_WHY["not_buildable"])
	else:
		tip += (" Drag to the zero stop to disarm; the cap is kept and the row "
			+ "says what it was. The track is a 1-2-3-5 ladder, not a linear "
			+ "0-200 count.")

	var parts := DccWidgets.slider(parent, label, 0.0, float(LM_LADDER.size() - 1),
		1.0, float(rung), "", func(v: float): _lm_on_cap(key, v), tip,
		func(): _lm_commit_cap(key))
	var row: HBoxContainer = parts["row"]
	var slider: HSlider = parts["slider"]
	var name_label := row.get_child(0) as Label
	## `slider()` puts a `DccTheme.spacer()` between the label and the track,
	## which expands. Captured BEFORE the badge is inserted, and identified by
	## its exact class rather than by a surviving index, because a bare `Control`
	## is the only thing in that row that is one.
	var slack: Control = null
	if row.get_child_count() > 1 and row.get_child(1).get_class() == "Control":
		slack = row.get_child(1)

	## §3.1's class badge, in the row's left gutter: a three-letter mono mark,
	## not a nested level. `_row()` builds its label as child 0, so the badge is
	## inserted ahead of it and the name label gives up its fixed width to take
	## the slack instead -- 49 names of very different lengths through one
	## `clip_text` column would clip the long ones on every row that has a tag.
	## §12's drawn glyph for this type, ahead of the class badge.
	##
	## `DccIcons.landmark_glyph()` resolves the engine's own key, including the
	## three that reuse a shipped sculpt-feature glyph rather than carrying a
	## near-duplicate (`cliff`, `lake`, `volcanic_feature`). It returns `""` for
	## a key this build has no glyph for, and then **no icon is drawn at all** --
	## the engine's type list is data and can grow ahead of the glyph table, and
	## a wrong icon is worse than none.
	##
	## `modulate` rather than a second asset: the glyph is drawn in white and
	## takes its colour from the row, so it dims with a `not buildable` row and
	## inverts with the light theme without a light-mode copy existing. That is
	## `currentColor` by the route `dcc_icons.gd`'s own header describes.
	var glyph := DccIcons.landmark_glyph(key)
	if glyph != "":
		var ic := TextureRect.new()
		ic.texture = DccIcons.get_icon(glyph, 13)
		ic.custom_minimum_size = Vector2(13, 13)
		ic.stretch_mode = TextureRect.STRETCH_KEEP_CENTERED
		ic.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		ic.modulate = DccTheme.c("text_dim" if buildable else "text_ghost")
		ic.tooltip_text = label
		row.add_child(ic)
		row.move_child(ic, 0)

	var badge := DccTheme.mono_label(_lm_badge(cls), "text_ghost", DccTheme.FS_MICRO, 0)
	badge.custom_minimum_size.x = 22
	badge.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	row.add_child(badge)
	row.move_child(badge, 1 if glyph != "" else 0)
	## The canvas gives the *name* the slack (`flex:1` on the row's name span),
	## so the expansion moves off `_row()`'s spacer and onto the label, and the
	## label's fixed `ROW_LABEL_W` goes to zero. Both halves matter: with the
	## width left at 132 a row carrying the `[no viewshed]` tag needs ~334 px of
	## a dock the user can drag down to `W_LEFT_DOCK_MIN` (300), and the row
	## would overflow rather than clip. At zero it clips, which is what
	## `_row()`'s own `clip_text` is already there for.
	if name_label != null:
		name_label.custom_minimum_size.x = 0
		name_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if slack != null:
		slack.size_flags_horizontal = Control.SIZE_FILL

	## §9.3 / `Dock.dc.html`: the bracketed tag beside the type's name, on the
	## row, never in a footnote.
	if needs_vs:
		var tag := DccTheme.mono_label("[no viewshed]", "text_faint", DccTheme.FS_MICRO, 0)
		tag.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		row.add_child(tag)
		row.move_child(tag, 2)

	## §2.2 part 1 -- **the crux**. A second 2 px rule directly under the slider
	## track, its length the placed count as a fraction of the cap. Two bars,
	## same origin, different lengths; the gap between them IS the
	## cap-versus-quota distinction, rendered on every row with no reading
	## required, and they sit flush when a type tops out.
	##
	## The columns are `_row()`'s own: `TRACK_W` wide, ending `ROW_VALUE_W` plus
	## one row separation from the right edge, so the two bars share an origin
	## exactly rather than approximately.
	var under := HBoxContainer.new()
	under.add_theme_constant_override("separation", 0)
	under.custom_minimum_size.y = 2
	under.add_child(DccTheme.spacer())
	var track := HBoxContainer.new()
	track.add_theme_constant_override("separation", 0)
	track.custom_minimum_size = Vector2(_lm_track_w(), 2)
	var bar := ColorRect.new()
	bar.color = DccTheme.c("text_dim")
	bar.custom_minimum_size = Vector2(0, 2)
	track.add_child(bar)
	under.add_child(track)
	var gutter := Control.new()
	gutter.custom_minimum_size.x = DccWidgets.ROW_VALUE_W + 8
	under.add_child(gutter)
	parent.add_child(under)

	## §2.2 parts 2 and 3: the count, and the one word that names what actually
	## stopped the generator. The word is a `text_button` because clicking it
	## opens §5's funnel -- the design's three depths are the under-bar at a
	## glance, the token in one word, and the popover in five numbers.
	var line := HBoxContainer.new()
	line.add_theme_constant_override("separation", 4)
	var indent := Control.new()
	indent.custom_minimum_size.x = 30
	line.add_child(indent)
	var count := DccTheme.mono_label("", "text_ghost", DccTheme.FS_MICRO, 0)
	line.add_child(count)
	parent.add_child(line)
	var token := DccWidgets.text_button(line, "", func(): _lm_open_funnel(key))

	_lm_rows[key] = {
		"family": fam, "class": cls, "label": label, "buildable": buildable,
		"needs_viewshed": needs_vs, "row": row, "under": under, "bar": bar,
		"line": line, "count": count, "token": token, "slider": slider,
		"readout": parts["readout"], "cap": cap, "armed": armed, "rung": rung,
		"retained": cap, "default_cap": default_cap,
		"funnel": funnels.get(key, {}), "drag_from_off": false,
	}
	if not buildable:
		slider.editable = false
	slider.drag_started.connect(func(): _lm_drag_start(key))
	_lm_refresh_row(key)

# -- § LAST RUN ---------------------------------------------------------------

## §9.1 rows 15-17. The run button relabels and disables itself for the length
## of the pass. It used to do that and then *block the main thread* for the
## whole run, which is what the owner reported on 2026-09-01 as a freeze;
## `EngineBridge.landmark_run()` is threaded now, so the relabel is a live
## busy state rather than the last thing the window painted before dying.
func _lm_last_run(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Last run")
	_lm_stale_note = DccWidgets.note(sec, "")
	_lm_refresh_stale()
	_lm_run_btn = DccWidgets.action(sec, "Run landmark pass", _lm_run, true)
	_lm_run_btn.tooltip_text = ("Generates candidates for every armed type, scores "
		+ "them, and spaces them under the exclusion radii above. Seconds, not "
		+ "milliseconds -- but it runs on a worker thread, so the window stays "
		+ "live and the map updates itself when it lands.\n\nDeliberately a "
		+ "button and not a cascade after every slider: a "
		+ "panel that silently re-ran on every drag would have no *last run* to "
		+ "report, and '11 placed' is a fact about a run.")
	## Always pressable, for `_build_recompute`'s own reason: pressing it with
	## nothing stale is a real re-run of the same answer, not an error, and a
	## greyed button reports a state the badge above it already reports better.
	_lm_run_note = DccWidgets.note(sec,
		"Not run yet in this session. Every row's second line will say what the "
		+ "pass actually did and, in one word, what stopped it.")

	## §1.4: hand-stamped icons keep their own group at the foot and are never
	## mixed into a generated family's counts. A hand-placed mark has no causal
	## chain and no emergent importance -- the two fields a landmark *is* -- so a
	## generated 11 and a hand-placed 3 are different claims and this panel never
	## adds them together.
	var hand := DccWidgets.group(sec, "placed by hand", false)
	var icons: Array = bridge.icon_list() if bridge != null and bridge.has_method("icon_list") else []
	DccWidgets.note(hand,
		"%d icon(s) stamped on the map by hand. They are annotation, not " % icons.size()
		+ "entities: no causal chain, no emergent importance, and never counted "
		+ "into a family above.")
	var go := DccWidgets.action(hand, "Place an icon → Cartography ▸ Assets & landmarks",
		func(): app.select_domain_category("cartography", "Assets & landmarks"))
	go.alignment = HORIZONTAL_ALIGNMENT_LEFT

## `_recompute_civ`'s pattern, in this panel's own subject.
##
## The `await` on `landmark_run()` is not optional and not cosmetic: the bridge
## runs the pass on a `Thread` and hands the reply back through
## `landmark_finished`, so a bare call would return the coroutine rather than
## the result and every row below would read an empty funnel. Nothing here
## pushes the placements at the map -- `ViewportHost` connects itself to
## `landmark_finished` for that, so a second caller of `landmark_run()` cannot
## forget the way this one did.
func _lm_run() -> void:
	var b := _lm_run_btn
	if b != null and is_instance_valid(b):
		b.text = "Running…"
		b.disabled = true
	var r: Dictionary = {}
	if bridge != null and bridge.has_method("landmark_run"):
		r = await bridge.landmark_run()
	if b != null and is_instance_valid(b):
		b.disabled = false
		b.text = "Run landmark pass"
	if _lm_run_note != null and is_instance_valid(_lm_run_note):
		if r.is_empty():
			_lm_run_note.text = ("No pass ran: this build's EngineBridge has no "
				+ "landmark_run(). Nothing was changed.")
		elif not bool(r.get("ok", false)):
			_lm_run_note.text = "Not run. %s" % String(r.get("error", "Unknown reason."))
		else:
			_lm_run_note.text = ("Placed %d landmarks in %.1f s. Every armed row's "
				+ "second line now says what stopped it; click that word for the "
				+ "arithmetic.") % [int(r.get("placed", 0)), float(r.get("seconds", 0.0))]
	## The funnels the run just produced, from the run's own reply where it
	## carries them and from `landmark_funnels()` otherwise -- the contract
	## returns them in both places and this must not depend on which.
	var funnels := {}
	for f in (r.get("funnels", []) as Array):
		funnels[String((f as Dictionary).get("kind", ""))] = f
	if funnels.is_empty():
		funnels = _lm_funnel_map()
	for key in _lm_rows:
		var rec: Dictionary = _lm_rows[key]
		rec["funnel"] = funnels.get(key, {})
		_lm_refresh_row(String(key))
	for f in _lm_groups:
		_lm_refresh_group(String(f))
	_lm_refresh_headroom()
	_lm_refresh_stale()

# -- §5 the "why fewer" funnel ------------------------------------------------

## §5. Clicking a row's reason token opens the funnel for that type from the
## last run: one arithmetic, no prose. Its shape and its division of labour are
## `explain_settlement`'s (`cartalith-godot/src/lib.rs:4920`), whose own doc
## comment states the rule this popover keeps -- *"All wording is left to the
## caller: this returns facts, not prose."* The engine returns the integers;
## this file writes the labels.
##
## A `PopupPanel` rather than a `PopupMenu`: these are numbers, not actions, and
## `layers_popover.gd` is the shell's own precedent for a panel-shaped popover.
## Built here rather than in `dcc_widgets.gd`, which this pass may not edit --
## the same way this file already builds `_ctx_menu` inline.
func _lm_open_funnel(key: String) -> void:
	var r: Dictionary = _lm_rows.get(key, {})
	if r.is_empty():
		return
	var f: Dictionary = r.get("funnel", {})
	if f.is_empty():
		return
	if _lm_funnel == null or not is_instance_valid(_lm_funnel):
		_lm_funnel = PopupPanel.new()
		_lm_funnel.add_theme_stylebox_override("panel",
			DccTheme.panel("panel", {"left": 1, "right": 1, "top": 1, "bottom": 1}))
		add_child(_lm_funnel)
	for c in _lm_funnel.get_children():
		_lm_funnel.remove_child(c)
		c.queue_free()
	_lm_funnel.add_child(_lm_funnel_body(r, f))
	var tok: Control = r["token"]
	var at := Vector2i(tok.get_screen_position()) + Vector2i(0, int(tok.size.y) + 4)
	_lm_funnel.popup(Rect2i(at, Vector2i(342, 0)))

func _lm_funnel_body(r: Dictionary, f: Dictionary) -> Control:
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 12)
	pad.add_theme_constant_override("margin_right", 12)
	pad.add_theme_constant_override("margin_top", 10)
	pad.add_theme_constant_override("margin_bottom", 10)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 3)
	pad.add_child(box)

	var raw_limit := String(f.get("limit", ""))
	var limit := _lm_limit_key(raw_limit)
	box.add_child(DccTheme.header("%s · last pass" % String(r["label"]), ""))
	var cand := int(f.get("candidates", 0))
	var c_con := int(f.get("rejected_constraint", 0))
	var c_sc := int(f.get("rejected_score", 0))
	var c_sp := int(f.get("rejected_spacing", 0))
	## The fifth bucket (engine, 2026-08-30). Before it existed these were
	## folded into `rejected_score`, and this popover reported them as "below
	## the importance floor" -- a quality judgement the generator never made.
	## Measured on the fixture the moment the bucket landed: `ford` had **320**
	## candidates blamed on score, and every one of them had passed every test
	## and simply lost to the number 24.
	var c_cap := int(f.get("rejected_cap", 0))
	var cap := int(f.get("cap", int(r["cap"])))
	var placed := int(f.get("placed", 0))
	var left := cand
	_lm_funnel_row(box, "candidates evaluated", "", str(cand), limit == "candidates")
	left -= c_con
	_lm_funnel_row(box, "failed this type's constraints", "− %d" % c_con,
		"%d left" % left, limit == "no_terrain")
	left -= c_sc
	## Drawn even at zero, and it IS zero on every kind today. That is a real
	## fact about the pipeline rather than a gap: §30 has no suitability-
	## rejection step (step 6 rejects on constraints, step 8 on spacing, step 7
	## only ranks), so the engine's score floor is 0.0 and nothing falls under
	## it. Showing the row keeps the funnel's five terms visible and lets a
	## future floor appear here without the popover changing shape.
	_lm_funnel_row(box, "below the importance floor", "− %d" % c_sc,
		"%d left" % left, false)
	left -= c_sp
	_lm_funnel_row(box, "rejected by spacing", "− %d" % c_sp,
		"%d left" % left, limit == "spacing")
	left -= c_cap
	## **The row the owner's brief is about.** These passed everything and were
	## turned away by the number alone -- so the wording says exactly that, and
	## deliberately does not use the verb "rejected", which the four rows above
	## have earned and this one has not.
	_lm_funnel_row(box, "over the cap — would have fit", "− %d" % c_cap,
		"%d left" % left, limit == "at_cap")
	## §5's own annotation: this row subtracts nothing and it is the most
	## important line here. It is the panel stating, in its own funnel, whether
	## the number the user set was what limited them -- which is the whole of the
	## owner's brief. Omit it and the funnel silently drops the one term the user
	## came to check.
	_lm_funnel_row(box, "cap %d" % cap, "",
		"reached" if limit == "at_cap" else "not reached", limit == "at_cap")
	box.add_child(DccTheme.rule())
	var unused: int = maxi(cap - placed, 0)
	_lm_funnel_row(box, "%d placed" % placed, "",
		"%d of the cap unused" % unused, false)

	DccWidgets.note(box, _lm_limit_why(raw_limit))

	## §5's two actions, drawn and disabled with the reason attached -- the same
	## treatment `menus.gd:233`'s `_todo` gives an unbacked menu row, and the one
	## `TypeRow.dc.html` argues for in as many words ("a dimmed row with a dash
	## is not a bug"). Neither is computable from the bridge contract: the
	## crowding figure at which the rejected candidates would have fit is not
	## returned by anything, and the rejected candidates themselves have no
	## coordinates in `landmark_funnels()` and no renderer path of their own.
	var acts := HBoxContainer.new()
	acts.add_theme_constant_override("separation", 5)
	box.add_child(acts)
	var lower := DccWidgets.chip(acts, "Lower crowding to fit", Callable())
	lower.disabled = true
	lower.tooltip_text = ("Owed. §5 wants the exact multiplier at which the "
		+ "rejected candidates would have fit under the remaining cap, and "
		+ "landmark_funnels() returns counts only -- a generic 'adjust spacing' "
		+ "would send you back to guessing, which is what the design rejects.")
	var show := DccWidgets.chip(acts, "Show rejected", Callable())
	show.disabled = true
	show.tooltip_text = ("Owed. Drawing the rejected candidates inside the placed "
		+ "ones' exclusion rings needs their positions and a map layer for them; "
		+ "landmark_funnels() carries neither.")
	return pad

func _lm_funnel_row(box: Control, label_text: String, minus: String, right: String,
		binding: bool) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	var token := "accent" if binding else "text_faint"
	var l := DccTheme.mono_label(label_text, token, DccTheme.FS_TINY, 0)
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(l)
	if minus != "":
		var m := DccTheme.mono_label(minus, token, DccTheme.FS_TINY, 0)
		m.custom_minimum_size.x = 52
		m.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		row.add_child(m)
	var rr := DccTheme.mono_label(right, "text_bright" if binding else "text_ghost",
		DccTheme.FS_TINY, 0)
	rr.custom_minimum_size.x = 88
	rr.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	row.add_child(rr)
	box.add_child(row)

# -- landmark refreshes and handlers ------------------------------------------

func _lm_refresh_headroom() -> void:
	if _lm_head_note == null or not is_instance_valid(_lm_head_note):
		return
	var h := _lm_headroom()
	if h.is_empty():
		_lm_head_note.text = ("Caps total %d. The packing estimate and the last " % _lm_caps_total()
			+ "run's total are not reported by this build's bridge, so this line "
			+ "carries the one figure it can stand behind.")
		return
	_lm_head_note.text = ("caps total %d · room for about %d at this spacing · "
		+ "last run placed %d") % [int(h.get("caps_total", _lm_caps_total())),
		int(h.get("room_estimate", 0)), int(h.get("last_placed", 0))]

func _lm_caps_total() -> int:
	var n := 0
	for key in _lm_rows:
		var r: Dictionary = _lm_rows[key]
		if bool(r["armed"]):
			n += int(r["cap"])
	return n

func _lm_refresh_crowding() -> void:
	if _lm_crowd_readout != null and is_instance_valid(_lm_crowd_readout):
		_lm_crowd_readout.text = "× %.2f" % _lm_crowding
	if _lm_crowd_note == null or not is_instance_valid(_lm_crowd_note):
		return
	## §4.1: the second line is the whole point. `× 1.00` is arithmetic; `34 km`
	## is a fact about the map the user is looking at.
	##
	## Kilometres unconditionally: `Preferences ▸ Units` has two ids reserved
	## (`menus.gd`'s `ID_PREF_UNITS_KM` / `ID_PREF_UNITS_MI`) and nothing in the
	## shell reads them, so there is no setting to honour yet and inventing one
	## here would be a second copy of a preference.
	var km := _lm_class_radius(LM_QUOTED_CLASS) * _lm_crowding
	var cname := String(LM_CLASS_LABEL.get(LM_CLASSES[LM_QUOTED_CLASS], "")).to_lower()
	_lm_crowd_note.text = "a %s landmark keeps %.0f km clear · sparse → dense" % [cname, km]

func _lm_class_radius(i: int) -> float:
	return float(_lm_radii[i]) if i >= 0 and i < _lm_radii.size() else 0.0

func _lm_on_crowding(v: float) -> void:
	_lm_crowding = v
	_lm_refresh_crowding()

func _lm_on_radius(i: int, v: float) -> void:
	while _lm_radii.size() <= i:
		_lm_radii.append(0.0)
	_lm_radii[i] = v
	if i == LM_QUOTED_CLASS:
		_lm_refresh_crowding()

func _lm_reset() -> void:
	_lm_write("landmark_reset_settings", [])
	_lm_rebuild()

## §2.1's zero stop, live: every tick rewrites the readout and the second line
## so the row's own state is honest mid-drag, and nothing is written to the
## engine until the drag ends.
func _lm_on_cap(key: String, v: float) -> void:
	var r: Dictionary = _lm_rows.get(key, {})
	if r.is_empty():
		return
	var rung: int = clampi(int(round(v)), 0, LM_LADDER.size() - 1)
	r["rung"] = rung
	r["armed"] = rung > 0
	if rung > 0:
		r["cap"] = LM_LADDER[rung]
	_lm_refresh_row(key)

func _lm_drag_start(key: String) -> void:
	var r: Dictionary = _lm_rows.get(key, {})
	if not r.is_empty():
		r["drag_from_off"] = not bool(r["armed"])

## §2.1's *"Drag up from `off` and the slider resumes at 40."*
##
## Implemented as a snap on release rather than mid-drag: leaving the zero stop
## re-arms the type at the number the row was already promising it had kept, and
## the row's own `was 40` is what made that promise, so the value the user lands
## on is the one they were shown. Changing it from there is an ordinary drag
## from an armed state.
##
## **A resumed cap is restored exactly, not re-quantised.** The ladder has no
## 40 -- `off · 1 · 2 · 3 · 5 · 8 · 12 · 20 · 30 · 50 · 80 · 120 · 200` -- so
## running the retained number back through `_lm_rung()` and taking the rung's
## value would silently turn the 40 the row promised into 30, which is a
## different world and a broken promise. The rung is where the *handle* goes;
## `cap` stays the number the store actually holds. Caught by
## `_landmark_probe.gd` §B9, which is why the fixture's default is 40.
func _lm_commit_cap(key: String) -> void:
	var r: Dictionary = _lm_rows.get(key, {})
	if r.is_empty():
		return
	var rung := int(r.get("rung", 0))
	var resumed := 0
	if rung > 0 and bool(r.get("drag_from_off", false)):
		resumed = int(r["retained"])
		if resumed <= 0:
			resumed = int(r["default_cap"])
		if resumed > 0:
			rung = _lm_rung(resumed)
			r["rung"] = rung
			(r["slider"] as HSlider).set_value_no_signal(float(rung))
	r["drag_from_off"] = false
	r["armed"] = rung > 0
	if rung > 0:
		r["cap"] = resumed if resumed > 0 else LM_LADDER[rung]
		r["retained"] = int(r["cap"])
		_lm_write("landmark_set_cap", [key, int(r["cap"])])
	## Disarming writes `armed` and NOT a zero cap -- see `_lm_type_row`'s own
	## header for why the store keeps two fields.
	_lm_write("landmark_set_armed", [key, bool(r["armed"])])
	_lm_refresh_row(key)
	_lm_refresh_group(String(r["family"]))
	_lm_refresh_headroom()

func _lm_bulk(family: String, on: bool) -> void:
	for key in _lm_rows:
		var r: Dictionary = _lm_rows[key]
		if String(r["family"]) != family or not bool(r["buildable"]):
			continue
		if on:
			var cap := int(r["retained"])
			if cap <= 0:
				cap = int(r["default_cap"])
			if cap <= 0:
				continue
			## The exact number, not the rung's -- see `_lm_commit_cap()` for
			## why re-quantising a retained cap is a broken promise.
			r["rung"] = _lm_rung(cap)
			r["cap"] = cap
			r["retained"] = cap
			r["armed"] = true
			(r["slider"] as HSlider).set_value_no_signal(float(r["rung"]))
			_lm_write("landmark_set_cap", [String(key), int(r["cap"])])
			_lm_write("landmark_set_armed", [String(key), true])
		else:
			r["rung"] = 0
			r["armed"] = false
			(r["slider"] as HSlider).set_value_no_signal(0.0)
			_lm_write("landmark_set_armed", [String(key), false])
		_lm_refresh_row(String(key))
	_lm_refresh_group(family)
	_lm_refresh_headroom()

## §2.2, all three parts at once: the readout, the placed under-bar and the
## resolved line with its one-word reason.
func _lm_refresh_row(key: String) -> void:
	var r: Dictionary = _lm_rows.get(key, {})
	if r.is_empty():
		return
	var readout: Label = r["readout"]
	var count: Label = r["count"]
	var token: Button = r["token"]
	var under: Control = r["under"]
	var bar: ColorRect = r["bar"]
	var armed := bool(r["armed"])
	var cap := int(r["cap"])
	if not is_instance_valid(readout):
		return
	readout.text = ("%d max" % cap) if armed else "off"

	if not bool(r["buildable"]):
		readout.text = "—"
		count.text = "not buildable"
		token.visible = false
		under.visible = false
		(r["row"] as Control).modulate.a = 0.55
		return
	if not armed:
		var retained := int(r["retained"])
		count.text = ("was %d" % retained) if retained > 0 else "never set"
		token.visible = false
		under.visible = false
		return

	var f: Dictionary = r.get("funnel", {})
	if f.is_empty():
		count.text = "not run yet"
		token.visible = false
		under.visible = false
		return

	var placed := int(f.get("placed", 0))
	var raw := String(f.get("limit", ""))
	var limit := _lm_limit_key(raw)
	count.text = "%d placed ·" % placed
	token.visible = true
	token.text = _lm_limit_word(raw)
	token.tooltip_text = "%s Click for the funnel." % _lm_limit_why(raw)
	## §2.2: `at cap` in accent, every other reason in ink-dim. A panel where
	## nothing is at cap therefore has no accent on any second line, and a user
	## who has genuinely maxed something sees it immediately.
	token.add_theme_color_override("font_color",
		DccTheme.c("accent") if limit == "at_cap" else DccTheme.c("text_ghost"))
	under.visible = true
	bar.custom_minimum_size.x = _lm_bar_px(placed, cap, int(r["rung"]))

## The placed bar's length, in pixels of the same track the slider fills.
##
## §2.2 fixes the invariant rather than the arithmetic: "two bars, same origin,
## different lengths", flush when placed equals cap. The artboard draws every
## armed row's cap bar at full width because it is not drawing a real slider; a
## real one fills to where the cap sits on the ladder, so the placed bar is
## scaled to *that* fill. Do it any other way and a topped-out row stops reading
## as topped out, which is the one thing the pair of bars exists to show.
func _lm_bar_px(placed: int, cap: int, rung: int) -> float:
	if cap <= 0 or rung <= 0:
		return 0.0
	var frac := clampf(float(placed) / float(cap), 0.0, 1.0)
	var fill := float(rung) / float(LM_LADDER.size() - 1)
	return round(frac * fill * float(_lm_track_w()))

## §3.2: `› PHYSICAL   6 of 15 armed · 74 placed`. A count on a closed container
## is worth more than the container -- the same principle `Assets ▸ Icon
## families` already draws, and the reason a user who opens this panel with
## everything collapsed can still see where the map's markers came from.
func _lm_refresh_group(family: String) -> void:
	var g: Dictionary = _lm_groups.get(family, {})
	if g.is_empty():
		return
	var btn: Button = g.get("button")
	if btn == null or not is_instance_valid(btn):
		return
	var armed := 0
	var total := 0
	var placed := 0
	for key in _lm_rows:
		var r: Dictionary = _lm_rows[key]
		if String(r["family"]) != family:
			continue
		total += 1
		if bool(r["armed"]):
			armed += 1
		placed += int((r.get("funnel", {}) as Dictionary).get("placed", 0))
	btn.text = "%s %s   %d of %d armed · %d placed" % [
		DccIcons.SYMBOLS["expand"], String(g["title"]).to_upper(), armed, total, placed]

func _lm_set_filter(cls: String) -> void:
	_lm_filter = cls
	_lm_apply_filter()

## §3.1: the chips DIM the non-matching rows rather than hiding them. A row that
## vanishes takes its own count with it, and a user cannot tell a filtered panel
## from a short one.
func _lm_apply_filter() -> void:
	for k in _lm_chips:
		var b: Button = _lm_chips[k]
		if b != null and is_instance_valid(b):
			DccWidgets.set_segment_on(b, String(k) == _lm_filter)
	for key in _lm_rows:
		var r: Dictionary = _lm_rows[key]
		var hit: bool = _lm_filter == "" or String(r["class"]) == _lm_filter
		var a := 1.0 if hit else 0.28
		if not bool(r["buildable"]):
			a = minf(a, 0.55)
		(r["row"] as Control).modulate.a = a
		(r["line"] as Control).modulate.a = a
		(r["under"] as Control).modulate.a = a

## §9.1 row 17. `stale_stages()` has no `landmarks` key -- it is a graph over
## the ten generation stages plus `civ` -- so this note reports the layer the
## pass reads *from* and says outright that it cannot report the pass itself.
## Inferring "landmarks stale" from "civ stale" would be this file inventing a
## fact the engine did not state.
func _lm_refresh_stale() -> void:
	if _lm_stale_note == null or not is_instance_valid(_lm_stale_note):
		return
	if bridge == null or not bridge.has_world:
		_lm_stale_note.text = "No world yet -- generate one before running a landmark pass."
		return
	var civ: Dictionary = bridge.stale_stages().get("civ", {})
	if civ.is_empty():
		_lm_stale_note.text = ("The civ layer this pass reads is up to date. The "
			+ "engine's stage graph carries no landmarks entry, so nothing can say "
			+ "whether the last landmark run itself is still current.")
		return
	var why := String(civ.get("reason", ""))
	if why.is_empty():
		why = String(civ.get("origin", "an edit"))
	_lm_stale_note.text = ("The civ layer under this pass is stale (%s), so a run " % why
		+ "now would place against settlements and roads that have not caught up. "
		+ "Recompute civilisation first.")

# -- landmark bridge access ---------------------------------------------------
#
# Every call is guarded. A concurrent pass is writing these wrappers and the
# panel must degrade to a disclosed empty state against a build that does not
# have them yet, not crash the dock -- the same `_has()` shape
# `engine_bridge.gd` uses for `sized_api`/`import_api`/`save_api`.

func _lm_kinds() -> Array:
	if bridge == null or not bridge.has_method("landmark_kinds"):
		return []
	return bridge.landmark_kinds()

func _lm_settings() -> Dictionary:
	if bridge == null or not bridge.has_method("landmark_settings"):
		return {}
	return bridge.landmark_settings()

func _lm_headroom() -> Dictionary:
	if bridge == null or not bridge.has_method("landmark_headroom"):
		return {}
	return bridge.landmark_headroom()

func _lm_funnel_map() -> Dictionary:
	var out := {}
	if bridge == null or not bridge.has_method("landmark_funnels"):
		return out
	for f in bridge.landmark_funnels():
		out[String((f as Dictionary).get("kind", ""))] = f
	return out

func _lm_write(method: String, args: Array) -> void:
	if bridge != null and bridge.has_method(method):
		bridge.callv(method, args)

# -- landmark helpers ---------------------------------------------------------

## The nearest rung to a cap, never equality: a cap stored before this ladder
## changed (or handed over as the engine's own default) must still land on a
## detent rather than on nothing. Same rule `menus.gd`'s
## `_refresh_lighting_menu()` applies to its four value ladders.
static func _lm_rung(cap: int) -> int:
	if cap <= 0:
		return 0
	var best := 1
	for i in range(1, LM_LADDER.size()):
		if absi(LM_LADDER[i] - cap) < absi(LM_LADDER[best] - cap):
			best = i
	return best

## `CON` / `REG` / `LOC` / `CUL` -- §3.1's three-letter mono badge. Derived from
## the engine's own class key rather than tabulated, so a fifth class would get
## a badge instead of a blank gutter.
static func _lm_badge(cls: String) -> String:
	return cls.substr(0, 3).to_upper() if cls.length() >= 3 else cls.to_upper()

## A family key as a group title: `religious_cultural` -> `RELIGIOUS CULTURAL`
## once `group()` upper-cases it.
static func _lm_pretty(key: String) -> String:
	return key.replace("_", " ")

## The dock's own track width, which is a tablet figure on tablet -- the placed
## bar has to share the slider's column exactly, so it reads the same expression
## `DccWidgets.slider()` does rather than the desktop constant.
static func _lm_track_w() -> int:
	return DccTheme.role_px("slider_track_w") if DccTheme.is_tablet() else DccWidgets.TRACK_W

## The header `Button` `DccWidgets.group()` just appended -- it returns the body
## and not the header, and §3.2's counts have to live on the header. Searched
## backwards for the last `Button` child rather than indexed off the end, so a
## change to what `group()` appends after its header does not silently return
## the wrong node.
static func _lm_last_button(parent: Control) -> Button:
	for i in range(parent.get_child_count() - 1, -1, -1):
		var c := parent.get_child(i)
		if c is Button:
			return c
	return null

## v3 CIVIL ▸ MILITARY (`GUI_GAP_REGISTER.md` **CV-25**, built 2026-08-25).
##
## **The register's own reason for calling this new design was wrong**, in the
## way §37 has now been wrong four times: it said the reference models no
## garrisons or fortifications. It models both halves of what this category
## shows. `_umWallSpec`/`_umInferWalls` (reference 22109-22136) are a real
## four-rung fortification ladder, `_civPlaceDefensibility` (23802) is a real
## per-settlement defensive strength, and `_civFactionAggregates`'
## `power.military` -- already ported -- is a real per-faction readout. All
## three are ports now (`cartalith_civ::military`, golden-verified).
##
## The port had also been feeding that military axis a constant zero for its
## `0.35 * fortifiedFraction` term, because `FactionPlace::fortified` had no
## producer. It has one now, so these numbers are *closer* to the reference
## than the ones any other caller has been getting.
##
## What is still genuinely absent, and stays absent: garrison headcounts,
## campaigns, unit movement, combat. None is derivable from anything here and
## the reference has none either -- see the section's own note.
func _build_military() -> void:
	_military_body = DccWidgets.category(self, "Military", categories)
	_fill_military(_military_body)

## The three `sort_custom` comparators CV-25/CV-26 need, as named statics
## rather than inline lambdas: a GDScript lambda body cannot wrap onto a
## second line, and one written as a long single line is unreadable.
static func _by_military(x, y) -> bool:
	return float((x as Dictionary).get("military", 0.0)) > float((y as Dictionary).get("military", 0.0))

static func _by_defensibility(x, y) -> bool:
	return float((x as Dictionary).get("defensibility", 0.0)) > float((y as Dictionary).get("defensibility", 0.0))

static func _by_relation_value(x, y) -> bool:
	return float((x as Dictionary).get("value", 0.0)) > float((y as Dictionary).get("value", 0.0))

func _fill_military(parent: Control) -> void:
	var data: Dictionary = bridge.civ_military_summary()
	var factions: Array = data.get("factions", [])
	var places: Array = data.get("settlements", [])

	var strength := DccWidgets.section(parent, "Faction strength")
	if factions.is_empty():
		DccWidgets.note(strength, "No factions -- generate a world first.")
	else:
		DccWidgets.note(strength,
			"_civFactionAggregates' military axis: 45% relative population, 35% "
			+ "the share of this faction's settlements that are fortified, 20% its "
			+ "capital's tier. The reference's own words for the whole power "
			+ "breakdown are \"explicitly derived/heuristic, never presented as "
			+ "simulated\", and that holds here.")
		## Descending by military power: the readout's whole job is to let
		## two factions be compared, and a roster-index order buries that.
		var rows := factions.duplicate()
		rows.sort_custom(_by_military)
		var list := DccWidgets.group(strength, "By military power")
		for r in rows:
			var d: Dictionary = r
			var f := int(d.get("faction", 0))
			var b := DccWidgets.action(list, "%s -- %d/100 · %d of %d fortified" % [
				String(d.get("name", "?")), int(round(float(d.get("military", 0.0)))),
				int(d.get("fortified_count", 0)), int(d.get("settlement_count", 0))],
				func(): app.right_dock_ctrl.show_faction(f))
			b.alignment = HORIZONTAL_ALIGNMENT_LEFT
			b.tooltip_text = ("Capital %s · overall power %d/100 · walls: %d stone, "
				+ "%d palisade, %d ditch. Open this faction in the right dock.") % [
				String(d.get("capital", "—")), int(round(float(d.get("overall", 0.0)))),
				int(d.get("walled_stone", 0)), int(d.get("walled_palisade", 0)),
				int(d.get("walled_ditch", 0))]

	_fill_manpower(parent, factions)

	var forts := DccWidgets.section(parent, "Fortifications")
	if places.is_empty():
		DccWidgets.note(forts, "No settlements -- generate a world first.")
	else:
		DccWidgets.note(forts,
			"_umWallSpec's ladder -- none · ditch · palisade · stone -- from tier, "
			+ "function, threat (the fortified trait), wealth, age and command of "
			+ "the ground. Defensive strength blends the terrain's own ruggedness "
			+ "with whether the place is walled. The place editor's Walls, Age, "
			+ "Traits and Specialisation overrides all feed this; before today "
			+ "Walls and Age reached nothing at all.")
		var walled: Array = []
		for p in places:
			if bool((p as Dictionary).get("walled", false)):
				walled.append(p)
		walled.sort_custom(_by_defensibility)
		DccWidgets.note(forts, "%d of %d settlements are fortified." % [walled.size(), places.size()])
		var list := DccWidgets.group(forts, "Strongest places", walled.size() <= 12)
		## `civ_military_summary`'s `index` is into `bridge.settlements()`, so
		## a row pins the settlement through the same call `_settlement_row`
		## uses rather than a second selection path.
		var roster := bridge.settlements()
		for p in walled:
			var d: Dictionary = p
			var idx := int(d.get("index", -1))
			var b := DccWidgets.action(list, "%s -- %s wall · defence %d%%" % [
				String(d.get("name", "?")), String(d.get("wall_spec", "none")),
				int(round(100.0 * float(d.get("defensibility", 0.0))))],
				func():
					if idx >= 0 and idx < roster.size():
						_selected_index = idx
						app.right_dock_ctrl.on_settlement_selected(roster[idx], idx))
			b.alignment = HORIZONTAL_ALIGNMENT_LEFT
			b.tooltip_text = "%s · population %d · faction %d. Pin it in the right dock." % [
				String(d.get("kind", "")).capitalize(), int(d.get("pop", 0)), int(d.get("faction", 0))]

	var gaps := DccWidgets.section(parent, "Not built")
	DccWidgets.note(gaps,
		"Per-settlement garrisons · campaigns · unit movement · combat  ·  needs a decision\n"
		+ "The per-FACTION headcounts above are real and derived (Manpower). What "
		+ "is still absent is allocating them: which settlement holds which part "
		+ "of a standing army is a placement rule nothing here implies, and a "
		+ "campaign needs a clock, a map objective and an opposed force — none of "
		+ "which exists. The reference has none of it either. A feature to "
		+ "specify, not a gap to wire (GUI_GAP_REGISTER.md CV-25).\n"
		+ "Also absent by design: change over time. Every number in this category "
		+ "is a reading of the world as it stands.")

## The manpower half of CIVIL ▸ MILITARY (`MILITARY_MANPOWER_SCOPE.md`, built
## 2026-08-25 on the owner's own supplied specification).
##
## **Four outputs, not one "military size" statistic**, because they diverge
## radically: Imperial Rome kept ~250 000 regulars over perhaps 45-120 million
## people, while Republican Rome temporarily mobilised 17-29 % of its citizen
## body in the Second Punic War. A single number cannot say both.
##
## The five variables are shown beside the four outputs on purpose. This model
## has **no reference implementation to check against** -- the frozen snapshot
## has no army-size code at any line -- so the only defensible presentation is
## one that shows its working and lets the reader disagree with a driver
## rather than with a headcount.
##
## The era row is a **sanity band, never a driver**: it is derived from the
## five variables and reported with "within"/"above"/"below" rather than
## clamping anything. The owner's own words: *"these are modelling ranges, not
## historical laws."*
static func _by_standing(x, y) -> bool:
	return float(((x as Dictionary).get("manpower", {}) as Dictionary).get("standing_army", 0.0)) > float(((y as Dictionary).get("manpower", {}) as Dictionary).get("standing_army", 0.0))

## `1 240` rather than `1240.0` everywhere below -- these are headcounts, and
## a decimal point on a headcount reads as precision the model does not have.
static func _head(v: float) -> String:
	return FactionRosterWindow._thousands(int(round(v)))

## Everything one faction's row cannot fit: the populations the four outputs
## are drawn from, the two durations, and the era band with its verdict.
##
## The band is quoted with the verdict rather than instead of it, so a faction
## outside its era's range reads as a finding about that faction and not as a
## bug in the table.
##
## The era percentages are shares of the CITIZEN / FREE population and are
## labelled as such, so the citizen figure is quoted on its own line first
## (owner ruling, 2026-08-25). A band verdict whose denominator is invisible
## is a number a reader cannot argue with, which is the one thing this whole
## section is built not to be.
static func _manpower_tooltip(m: Dictionary) -> String:
	return ("Total population %s (%s in farming, %s of military age).\n"
		+ "Citizen / free population %s — %.0f%% of the total, and the "
		+ "denominator the era bands are read against.\n"
		+ "Professional core %s of the standing army.\n"
		+ "A field army stays out %d days; a full levy %d.\n"
		+ "Era: %s — %s. Standing %.2f%% of citizens (band %.1f-%.1f%%, %s); "
		+ "mobilization %.1f%% (band %.0f-%.0f%%, %s).\n"
		+ "Open this faction in the right dock.") % [
		_head(float(m.get("total_population", 0.0))),
		_head(float(m.get("farming_population", 0.0))),
		_head(float(m.get("mobilization_pool", 0.0))),
		_head(float(m.get("citizen_population", 0.0))),
		100.0 * float(m.get("citizen_fraction", 0.0)),
		_head(float(m.get("professional_core", 0.0))),
		int(round(float(m.get("field_duration_days", 0.0)))),
		int(round(float(m.get("emergency_duration_days", 0.0)))),
		String(m.get("era", "?")), String(m.get("era_constraint", "")),
		100.0 * float(m.get("standing_citizen_share", 0.0)),
		100.0 * float(m.get("era_standing_lo", 0.0)),
		100.0 * float(m.get("era_standing_hi", 0.0)),
		String(m.get("era_standing_verdict", "?")),
		100.0 * float(m.get("emergency_citizen_share", 0.0)),
		100.0 * float(m.get("era_mobilization_lo", 0.0)),
		100.0 * float(m.get("era_mobilization_hi", 0.0)),
		String(m.get("era_mobilization_verdict", "?"))]

func _fill_manpower(parent: Control, factions: Array) -> void:
	var sec := DccWidgets.section(parent, "Manpower")
	if factions.is_empty():
		DccWidgets.note(sec, "No factions -- generate a world first.")
		return

	DccWidgets.note(sec,
		"Agricultural technology does not set army size. It sets surplus, "
		+ "labour requirements, transport, the taxation base and administrative "
		+ "capacity, and manpower is supported out of those -- so this reports "
		+ "four separate figures that can differ radically for one population, "
		+ "and the five variables behind them.")

	var rows := factions.duplicate()
	rows.sort_custom(_by_standing)

	var list := DccWidgets.group(sec, "Standing · field · emergency")
	for r in rows:
		var d: Dictionary = r
		var m: Dictionary = d.get("manpower", {})
		if m.is_empty():
			continue
		var f := int(d.get("faction", 0))
		var b := DccWidgets.action(list, "%s -- standing %s · field %s · levy %s" % [
			String(d.get("name", "?")), _head(float(m.get("standing_army", 0.0))),
			_head(float(m.get("field_army", 0.0))),
			_head(float(m.get("emergency_mobilization", 0.0)))],
			func(): app.right_dock_ctrl.show_faction(f))
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.tooltip_text = _manpower_tooltip(m)

	## Closed by default: it is the model's most informative output and also
	## its densest, and the three headcounts above are what a first read wants.
	var dur := DccWidgets.group(sec, "How long each can stay out", false)
	DccWidgets.note(dur,
		"The largest force sustainable for a given campaign length. 30 days is "
		+ "feasible, 90 difficult, 180 severe disruption, and a year needs a "
		+ "substantially different fiscal system -- a feudal obligation typically "
		+ "expired at about two months. A figure marked ⌈pool⌉ is capped by how "
		+ "many can be raised at all rather than by how long they can be fed.")
	for r in rows:
		var d: Dictionary = r
		var m: Dictionary = d.get("manpower", {})
		var ladder: Array = m.get("force_ladder", [])
		if ladder.size() < 4:
			continue
		var parts := PackedStringArray()
		for e in ladder:
			var l: Dictionary = e
			parts.append("%dd %s%s" % [int(l.get("days", 0)), _head(float(l.get("force", 0.0))),
				" ⌈pool⌉" if bool(l.get("capped_by_pool", false)) else ""])
		DccWidgets.note(dur, "%s -- %s" % [String(d.get("name", "?")), " · ".join(parts)])

	var drv := DccWidgets.group(sec, "What drives it", false)
	DccWidgets.note(drv,
		"Five interacting variables. Technology is deliberately not one of "
		+ "them: it enters only through the agricultural labour ratio, which is "
		+ "why two factions on the same ag-tech row with different governments, "
		+ "roads and land get different answers.")
	for r in rows:
		var d: Dictionary = r
		var m: Dictionary = d.get("manpower", {})
		if m.is_empty():
			continue
		var n := DccWidgets.note(drv, "%s -- farming %.0f%% · surplus/farmer %.2f · extraction %.1f%% · professional %.0f%% · logistics %.2f" % [
			String(d.get("name", "?")),
			100.0 * float(m.get("agricultural_labour_ratio", 0.0)),
			float(m.get("food_surplus_per_farmer", 0.0)),
			100.0 * float(m.get("fiscal_extraction_efficiency", 0.0)),
			100.0 * float(m.get("professionalization", 0.0)),
			float(m.get("logistics_capacity", 0.0))])
		n.tooltip_text = ("Ag-tech %s · government %s. State capacity %.2f, the "
			+ "term both extraction and professionalisation scale from. Ecological "
			+ "factor %.2f -- how well this faction's own territory feeds the "
			+ "people on it, and the reason geography moves the answer at all.") % [
			String(m.get("ag_tech", "?")), String(m.get("government", "?")),
			float(m.get("state_capacity", 0.0)), float(m.get("ecological_factor", 0.0))]
		n.mouse_filter = Control.MOUSE_FILTER_PASS

	## The era band's denominator, on screen rather than buried in a tooltip
	## (owner ruling, 2026-08-25). Before this, a "below" verdict was a
	## percentage of an invisible divisor; now the divisor is a headcount a
	## reader can disagree with, next to the government that produced it.
	var civ := DccWidgets.group(sec, "Who the bands are measured against", false)
	DccWidgets.note(civ,
		"The era percentages are shares of the CITIZEN / FREE population, not "
		+ "of the total -- the specification's own Republican Rome figure is "
		+ "quoted as \"17-29 % of its citizen population\". That body is a "
		+ "government's own share (a kin-based chiefdom counts nearly everyone; "
		+ "a slave-holding empire counts a minority), widened as a society "
		+ "leaves agriculture behind, since serfdom and slavery are agrarian "
		+ "institutions. It sets no headcount above -- only what they are a "
		+ "percentage of.")
	for r in rows:
		var d: Dictionary = r
		var m: Dictionary = d.get("manpower", {})
		if m.is_empty():
			continue
		var n := DccWidgets.note(civ, "%s -- citizens %s of %s (%.0f%%) · standing %.2f%% %s · mobilization %.1f%% %s · %s" % [
			String(d.get("name", "?")),
			_head(float(m.get("citizen_population", 0.0))),
			_head(float(m.get("total_population", 0.0))),
			100.0 * float(m.get("citizen_fraction", 0.0)),
			100.0 * float(m.get("standing_citizen_share", 0.0)),
			String(m.get("era_standing_verdict", "?")),
			100.0 * float(m.get("emergency_citizen_share", 0.0)),
			String(m.get("era_mobilization_verdict", "?")),
			String(m.get("era", "?"))])
		n.tooltip_text = ("Government %s. Bands: standing %.1f-%.1f%%, "
			+ "mobilization %.0f-%.0f%%. Against TOTAL population the same two "
			+ "figures would read %.2f%% and %.1f%%, which is the reading the "
			+ "first build of this model used.") % [
			String(m.get("government", "?")).replace("_", " "),
			100.0 * float(m.get("era_standing_lo", 0.0)),
			100.0 * float(m.get("era_standing_hi", 0.0)),
			100.0 * float(m.get("era_mobilization_lo", 0.0)),
			100.0 * float(m.get("era_mobilization_hi", 0.0)),
			100.0 * float(m.get("standing_share", 0.0)),
			100.0 * float(m.get("emergency_share", 0.0))]
		n.mouse_filter = Control.MOUSE_FILTER_PASS

	## The owner's own modelling caution, and the reason it is a number rather
	## than a warning string: ancient army figures are massively exaggerated
	## (Xerxes' invasion is described in millions and reconstructs to ~70 000
	## infantry and 9 000 cavalry), so the honest check is what could have been
	## fed in one place, not what a chronicle claims.
	var worst := 1.0
	for r in rows:
		var m: Dictionary = (r as Dictionary).get("manpower", {})
		var c := float(m.get("concentration_ratio", 1.0))
		if c > 0.0 and c < worst:
			worst = c
	DccWidgets.note(sec,
		"Plausibility: no faction here can concentrate more than %.0f%% of what "
		% [100.0 * worst]
		+ "it can raise. A host reported above its own field-army figure could "
		+ "not have been supplied in one place, whatever the source says.")

	## Said on screen rather than only in the scope document, because the
	## denominator the bands are read against is a modelling decision and not
	## an implementation detail -- it moves every verdict on this screen.
	DccWidgets.note(sec,
		"Era bands are a sanity check and never a driver — the era is derived "
		+ "from the five variables, not chosen, and nothing is clamped into "
		+ "range. They are shares of the citizen / free population (owner "
		+ "ruling, 2026-08-25), which is the reading the specification's own "
		+ "Republican Rome citation states outright and the one that stops its "
		+ "own Imperial Rome figure — ~250 000 regulars over 45–120 million — "
		+ "sitting under the classical band's 1 % floor. The four headcounts "
		+ "above are calibrated on the specification's worked examples and are "
		+ "unaffected by it.")


## v3 CIVIL ▸ RELATIONSHIPS (`GUI_GAP_REGISTER.md` **CV-26**, built 2026-08-25).
##
## The register's structural objection was the right one and it is what this
## builds: *there was no edge between two factions to hold a value*. There is
## one now (`cartalith_civ::relations`) -- **derived and recomputed**, the same
## shape as the aggregates and the wildlife regions, never stored, never
## saved, never changing on its own.
##
## Unlike Military above, this one has no reference implementation: the frozen
## snapshot's only hits for diplomacy, alliance, vassal or treaty are prose.
## So it is deliberately the smallest defensible thing -- four symmetric terms
## over quantities the civ layer already computes, each reported beside the
## verdict so the reader can disagree with it.
##
## Diplomacy actions, treaties, vassalage and change over time are **out of
## scope by design**, not by omission; the section's own note says so.
func _build_relationships() -> void:
	_relations_body = DccWidgets.category(self, "Relationships", categories)
	_fill_relationships(_relations_body)

func _fill_relationships(parent: Control) -> void:
	var pairs: Array = bridge.civ_faction_relations()

	var sec := DccWidgets.section(parent, "Standing")
	if pairs.is_empty():
		DccWidgets.note(sec,
			"Fewer than two factions -- generate a world, or add a faction in the "
			+ "roster window. A relation needs two parties.")
	else:
		DccWidgets.note(sec,
			"Derived, not simulated: shared culture (+30), shared or opposed faith "
			+ "(±20), how much of what each side lacks the other exports (+25), and "
			+ "friction along a shared border, weighted by how evenly matched the "
			+ "two are (−55). Recomputed on every open; nothing here is stored.")
		var rows := pairs.duplicate()
		rows.sort_custom(_by_relation_value)
		var list := DccWidgets.group(sec, "Every pair")
		for r in rows:
			var d: Dictionary = r
			var a := int(d.get("a", 0))
			## `GUI_GAP_REGISTER.md` **RL-01**: the second id. This row names a
			## pair, so the dock is told both parties -- without it the callback
			## was `show_faction(a)` alone, which meant a row claiming a pair
			## opened one side of it and two consecutive rows sharing that side
			## were a press with no visible effect anywhere (5 of 15 rows on a
			## real six-faction world). `_build_faction_relations` draws the
			## marked pair.
			var other := int(d.get("b", 0))
			var b := DccWidgets.action(list, "%s ↔ %s -- %s (%+d)" % [
				String(d.get("a_name", "?")), String(d.get("b_name", "?")),
				String(d.get("stance", "neutral")),
				int(round(100.0 * float(d.get("value", 0.0))))],
				func(): app.right_dock_ctrl.show_faction(a, other))
			b.alignment = HORIZONTAL_ALIGNMENT_LEFT
			b.tooltip_text = ("Border %d cells (%d%% of the widest on this map) · "
				+ "culture %+d · faith %+d · trade %+d · rivalry %d%%. "
				## RL-01: this used to read "Opens %s in the right dock" with
				## `a_name`, which was true and was the defect -- the row names
				## a pair. It opens both now, and says so.
				+ "Opens %s in the right dock, with %s marked among its relations.") % [
				int(d.get("border_cells", 0)),
				int(round(100.0 * float(d.get("border_fraction", 0.0)))),
				int(round(30.0 * float(d.get("culture_term", 0.0)))),
				int(round(20.0 * float(d.get("religion_term", 0.0)))),
				int(round(25.0 * float(d.get("trade_term", 0.0)))),
				int(round(100.0 * float(d.get("rivalry_term", 0.0)))),
				String(d.get("a_name", "?")), String(d.get("b_name", "?"))]

	var gaps := DccWidgets.section(parent, "Not built")
	DccWidgets.note(gaps,
		"Treaties · vassalage · diplomacy actions · change over time  ·  needs a decision\n"
		+ "Three unanswered questions rather than one gap: who acts, on what clock, "
		+ "and what a treaty does to the map (GUI_GAP_REGISTER.md CV-26, narrowed "
		+ "to exactly these). Vassalage and alliances under v3's Politics are the "
		+ "same open question.\n"
		+ "The standing between every pair is derived and live above -- a reading "
		+ "of the world as it is, which stops short of anything anyone did.")

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
## Both bodies are refilled together, and each is guarded on its own so the
## order `_build()` claims them in cannot leave one empty: Politics is
## category 11 and Simulation is category 14, so the first call runs with
## `_sim_body` still null and the second catches both.
func _rebuild_timeline() -> void:
	if _tl_body != null and is_instance_valid(_tl_body):
		_clear_body(_tl_body)
		if not bridge.has_world:
			DccWidgets.note(_tl_body, "Generate a world first.")
		else:
			_build_timeline_years(_tl_body)
			_build_timeline_scrub(_tl_body)
			_build_timeline_playback(_tl_body)
			_build_timeline_filters(_tl_body)
			_build_politics_gaps(_tl_body)
	if _sim_body != null and is_instance_valid(_sim_body):
		_clear_body(_sim_body)
		if not bridge.has_world:
			DccWidgets.note(_sim_body, "Generate a world first.")
		else:
			_build_timeline_sim(_sim_body)

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
	## `accent_ink` on the filled half since the 2026-08-31 re-base -- see the
	## token's own comment in `dcc_theme.gd`; `c("bg")` was near-black on amber.
	go.add_theme_color_override("font_color",
		DccTheme.c("accent_ink") if active else DccTheme.c("text"))
	go.add_theme_color_override("font_hover_color",
		DccTheme.c("accent_ink") if active else DccTheme.c("text_bright"))
	go.add_theme_stylebox_override("normal", DccTheme.flat(DccTheme.c("accent") if active else DccTheme.c("sunken")))
	go.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("accent").lightened(0.1) if active else DccTheme.c("raised")))
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

## v3 POLITICS' second row -- *"Vassalage · alliances · rivalries"* -- has the
## same missing model as RELATIONSHIPS below, so it says so here and points at
## the one category that owns the finding rather than repeating it.
func _build_politics_gaps(body: Control) -> void:
	var sec := DccWidgets.section(body, "Not built")
	DccWidgets.note(sec,
		"Vassalage, alliances and rivalries over time (GUI_GAP_REGISTER.md "
		+ "CV-26). A recorded year snapshots which settlements exist and who "
		+ "holds which cell; it records no relation between two factions, "
		+ "because cartalith-civ has no such relation to record at any year. "
		+ "The whole finding is under Relationships below.")


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
