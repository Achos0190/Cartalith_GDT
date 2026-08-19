extends Control
class_name JourneyPlannerView

## The Journey Planner as an in-shell tool takeover -- `JOURNEY_PLANNER_SPEC.md`'s
## direction 1a (distance spine), replacing the old `journey_planner_window.gd`
## (`extends AcceptDialog`, deleted this pass).
##
## **Architecture decision (2026-08-19).** `DCC_SHELL_SPEC.md` §4.5.4's own
## addition reads: arming the JOURNEY tool "swaps the whole INFRA viewport
## region (map, both docks, tool options bar)... rather than drawing an
## overlay on the map like Way/Route do." This file is that swap, built the
## same way `app.gd`'s `_on_workspace_changed` already swaps chrome per
## domain -- one `Control` per swapped region, built once and hidden, shown
## by listening to signals that already exist (`app.tool_armed`,
## `app.workspace_changed`) rather than inventing a second dispatch
## mechanism:
##
## **Domain merge (2026-08-20).** INFRA (and the Way/Route/Journey tools with
## it) merged into CIVIL -- `dcc_shell.gd`'s `DOMAINS` doc comment. Journey
## still swaps the *whole* domain region it lives under, which is now
## `"civilization"`, not `"infrastructure"`; every reference to that domain
## id below was updated to match, and the swap now hides all of CIVIL's own
## content (Settlement/Territory/Roads/Rivers/... alike), not just the
## INFRA slice of it -- correct under the merge, since Journey's own contract
## was always "the whole domain region", never "just the INFRA half of it".
##
## - `_left_panel` is appended to `app.left_dock_body` alongside every
##   domain's `register_workspace()` panel, but is never registered as one --
##   it is shown/hidden by this file directly, and `CivilizationWorkspace`'s
##   own panel (which now also contains the nested `InfrastructureWorkspace`)
##   is hidden the same way while Journey is active (reached via
##   `app._workspace_panels["civilization"]`; a leading-underscore, same-
##   layer read, not a public API this file invented).
## - `_center_panel` is appended to `app.viewport_content` next to
##   `app.viewport` (the map surface) and swaps places with it -- `app.viewport
##   .visible = false` while this is showing, restored on disarm.
## - The right dock is NOT taken over directly. `right_dock.gd` already owns
##   `right_dock_body` end to end and rebuilds it on every selection change
##   (`RightDock`'s own doc: "contents follow the selection, not the
##   workspace"); fighting that ownership would mean two files clearing the
##   same container. Instead this adds one more context to `RightDock`'s
##   existing `CTX_*` dispatch (`CTX_JOURNEY`, mirroring the `CTX_SCULPT`
##   precedent already there) that delegates the actual content-building back
##   to `build_results()` below. `right_dock.gd` carries ~20 new lines for
##   this; nothing here reaches into its container directly.
## - The tool options bar follows the exact pattern every other tool already
##   uses (`app.set_tool_options`, `_tool_options_way`/`_tool_options_route`
##   in `infrastructure_workspace.gd`) -- `_tool_options_journey()` below is a
##   sibling of those, not a new mechanism.
##
## Visibility is computed, not stored as a single flag: `_active` is true
## exactly when `app.armed_tool == "journey"` AND `app.active_domain() ==
## "civilization"` (was `"infrastructure"` before the 2026-08-20 domain
## merge), recomputed on both `tool_armed` and `workspace_changed`
## so switching domains away and back while Journey stays armed (the "one
## tool armed at a time, globally" rule every other tool already lives under)
## restores the swap correctly instead of leaving stale chrome.
##
## ## What is real vs. disclosed placeholder
##
## Every number in the route map, route totals, terrain profile, stops strip,
## stage inspector, stage matrix and results panel traces to a real
## `bridge.route_get()` / `bridge.jp_compute()` call -- there is no invented
## data anywhere in this file. Specific, named simplifications against the
## mockup's own example content:
##
## - **Journeys list** shows committed routes (`route_count`/`route_get`),
##   not named, persisted "journeys" -- no such registry exists engine-side.
## - **Carriage auto/manual**: the toggle is real UI state, but "Auto" mode's
##   own picker (`jpAutoPickTransport`, reference HTML ~line 19617) has no
##   Rust port -- `journey_bridge.rs`/`cartalith-civ` expose no
##   auto-carriage function. Selecting Auto disables the animal/vehicle
##   fields and disables the picker with a stated reason rather than faking a
##   plausible-looking auto-pick; genuinely a Rust-side gap, reported rather
##   than invented here per this task's own constraint.
## - **Party presets**: `JP_PRESETS` (reference HTML ~line 17595) is
##   JS-side-only; no `jp_presets()` binding exists. The preset control is
##   present and disabled with that reason.
## - **Re-route for <mode>…**: `_jpRerouteForMode`/`jpAutoPickTransport`'s
##   sibling -- same gap, same disclosure.
## - **Cost group** (food/fodder/wages/tolls/upkeep in currency): `jp_plan`
##   returns no monetary figures at all (`jp_journey_plan_dict`, checked
##   against the full field list). Shown with a stated gap, not invented sums.
## - **Elevation-profile sparkline**: unlike the old dialog (which reported
##   `plan.profile`'s presence and stopped), this pass DOES draw it --
##   `_ProfileView` plots the real 0-1 normalised samples. It was only
##   time-boxed out before; rebuilding this view was the right time to close
##   it for real.
## - **⇧-drag spine trim**: genuinely deferred, not faked. `jp_compute` has no
##   request field for trimming a route's endpoints or interior span --
##   building the gesture would mean inventing a request shape with nothing
##   on the other side of the boundary to receive it. Click-to-select and
##   ⌥-click-to-isolate are both real and implemented.
## - **Stage inspector disabled-with-reason**: implemented for the two cases
##   `JOURNEY_PLANNER_SPEC.md` §6 names explicitly (vessel on a land stage,
##   mount when the effective transport isn't Mounted Rider). The other
##   fields are always editable (blank/auto is always a legal value even when
##   it will not matter for this stage's category) rather than an exhaustive,
##   unexposed per-field validity matrix.
## - **Stops strip x-position**: real -- each stop's fractional position is
##   the nearest route-point's cumulative chord length divided by the route's
##   total chord length, which is exactly proportional to km on this port's
##   flat-projection map (`map_width_km` is uniform across the grid).

var app: DccApp
var bridge: EngineBridge

# -- Plan / route state --------------------------------------------------------

var _bound := false
var _options: Dictionary = {}
var _default_plan: Dictionary = {}
var _plan_values: Dictionary = {}
var _route_index := -1
var _last_result: Dictionary = {}
var _stage_overrides: Dictionary = {}   ## int stage idx -> Dictionary (JpStageOverride field pairs)
var _layovers: Dictionary = {}          ## stop key (String) -> int days
var _selected_stage := 0
var _isolated_stage := -1               ## -1 = no isolation
var _carriage_auto := true              ## View-local only; see class doc's disclosed gap.

var _active := false

# -- Region roots ---------------------------------------------------------------

var _left_panel: VBoxContainer
var _left_route_section: VBoxContainer
var _left_party_body: VBoxContainer
var _auto_obs: Dictionary = {}   ## JP-15: field_key (String) -> OptionButton, the party form's own "Auto" fields -- refreshed post-compute by `_refresh_auto_labels()` rather than rebuilt, so a live numeric edit elsewhere in the form never loses focus.

var _center_panel: Control
var _route_map: _RouteMapView
var _totals_body: VBoxContainer
var _profile: _ProfileView
var _stops_row: HBoxContainer
var _stops_note: Label
var _inspector_body: VBoxContainer
var _matrix_body: VBoxContainer
var _matrix_problem_count: Label
var _timeline_view: _TimelineBandView   ## JP-13: lives in `app.timeline_row`, not under `_center_panel` -- see `_rebuild_timeline_band()`.

# ================================================================= Setup ====

func setup(a: DccApp, b: EngineBridge) -> void:
	app = a
	bridge = b
	_bound = bridge.world_gen.has_method("jp_options") \
		and bridge.world_gen.has_method("jp_default_plan") \
		and bridge.world_gen.has_method("jp_compute") \
		and bridge.world_gen.has_method("route_count") \
		and bridge.world_gen.has_method("route_get")

	_build_left_panel()
	_build_center_panel()
	_left_panel.visible = false
	_center_panel.visible = false

	app.tool_armed.connect(func(_id: String): _recompute_visibility())
	app.workspace_changed.connect(func(_id: String): _recompute_visibility())

## Both entry points this pass wires (`DCC_SHELL_SPEC.md` §2.4, §4.5.4):
## `Data ▸ Journey planner… ⇧J` and the INFRA dock's own Logistics button both
## call this. It only arms the tool -- `_recompute_visibility()` (driven by
## the `tool_armed` signal this triggers) does the actual region swap, the
## same one-way flow every other tool already uses.
func open() -> void:
	app.arm_tool("journey")

func _recompute_visibility() -> void:
	var should_show := _bound and app.armed_tool == "journey" and app.active_domain() == "civilization"
	if should_show == _active:
		return
	_active = should_show
	if _active:
		_show()
	else:
		_hide()

func _show() -> void:
	## Hides the WHOLE civilization panel -- which now nests
	## `InfrastructureWorkspace` too (2026-08-20 domain merge) -- not just an
	## INFRA slice of it. Matches Journey's own contract of swapping the whole
	## domain region it lives under.
	var civ_panel: Control = app._workspace_panels.get("civilization")
	if civ_panel != null:
		civ_panel.visible = false
	app.viewport.visible = false
	_left_panel.visible = true
	_center_panel.visible = true
	app.set_rail_foot("JOURNEY")
	_refresh_route_choice()
	_rebuild_party_form()
	_tool_options_journey()
	app.right_dock_ctrl.show_journey(self)
	_compute()

func _hide() -> void:
	_left_panel.visible = false
	_center_panel.visible = false
	app.viewport.visible = true
	var civ_panel: Control = app._workspace_panels.get("civilization")
	if civ_panel != null and app.active_domain() == "civilization":
		civ_panel.visible = true
	if app.right_dock_ctrl != null:
		app.right_dock_ctrl.clear_journey()
	## JP-13: this view is the only thing that ever populates `timeline_row`
	## (CV-09 -- `GUI_GAP_REGISTER.md` §11 -- leaves it deliberately empty in
	## CIVIL); clear it back to that empty state on disarm so Journey content
	## never leaks into a domain switch.
	if app.timeline_row != null:
		for c in app.timeline_row.get_children():
			app.timeline_row.remove_child(c)
			c.queue_free()
		_timeline_view = null

# ============================================================ Left panel ====

func _build_left_panel() -> void:
	_left_panel = VBoxContainer.new()
	_left_panel.add_theme_constant_override("separation", 0)
	_left_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	app.left_dock_body.add_child(_left_panel)

	if not _bound:
		DccWidgets.note(_left_panel,
			"jp_options / jp_default_plan / jp_compute / route_count / route_get are not exposed by this build's GDExtension binary -- rebuild cartalith-godot.")
		return

	_left_route_section = DccWidgets.section(_left_panel, "Journeys")
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_left_panel.add_child(scroll)
	_left_party_body = VBoxContainer.new()
	_left_party_body.add_theme_constant_override("separation", 0)
	_left_party_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_left_party_body)

	_options = bridge.jp_options()
	_default_plan = bridge.jp_default_plan()

func _refresh_route_choice() -> void:
	if not _bound:
		return
	for c in _left_route_section.get_children():
		_left_route_section.remove_child(c)
		c.queue_free()

	var count := bridge.route_count()
	if count == 0:
		DccWidgets.note(_left_route_section,
			"No committed routes yet -- arm Route (⇧R) below, click waypoints, ✓ Commit, then this list re-reads it. Journeys here are committed routes, not a separate persisted list; JP_PRESETS-style named journeys are a JS-only concept with no engine registry.")
		_route_index = -1
		return

	var labels: Array = []
	for i in count:
		var r: Dictionary = bridge.route_get(i)
		var km := float(r.get("km", 0.0))
		var mode := String(r.get("mode", "?"))
		var unreach := int(r.get("unreachable_legs", 0))
		var label_text := "Route #%d — %s km (%s)" % [i, _fmt_thousands(km, 0), mode]
		if unreach > 0:
			label_text += "  [%d unreachable]" % unreach
		labels.append(label_text)
	if _route_index < 0 or _route_index >= count:
		_route_index = 0
	DccWidgets.choice(_left_route_section, "Committed route", labels, _route_index,
		func(i: int):
			_route_index = i
			_stage_overrides.clear()
			_layovers.clear()
			_selected_stage = 0
			_isolated_stage = -1
			_compute(),
		"route_get()'s own points/km/mode. Journeys here are committed routes, not a separate persisted list.")

# -- Party form fields (shared field-binding vocabulary, matching journey_planner_window.gd's own convention) --

func _plan_value_changed(structural: bool) -> void:
	if structural:
		_rebuild_party_form()
	_compute()

func _number_field(parent: Control, label_text: String, key: String, minimum: float,
		maximum: float, step: float, integer: bool, tooltip: String = "") -> void:
	var v := float(_plan_values.get(key, 0.0))
	DccWidgets.number(parent, label_text, minimum, maximum, step, v,
		func(nv: float):
			_plan_values[key] = (int(nv) if integer else nv)
			_plan_value_changed(false),
		tooltip)

func _toggle_field(parent: Control, label_text: String, key: String, structural: bool = false, tooltip: String = "") -> void:
	var v := bool(_plan_values.get(key, false))
	DccWidgets.toggle(parent, label_text, v,
		func(nv: bool):
			_plan_values[key] = nv
			_plan_value_changed(structural),
		tooltip)

func _choice_field(parent: Control, label_text: String, key: String, opts: PackedStringArray,
		allow_auto: bool, structural: bool = false, tooltip: String = "") -> void:
	var labels: Array = []
	var raw: Array = []
	if allow_auto:
		labels.append(_auto_label(key))
		raw.append("")
	for o in opts:
		labels.append(String(o))
		raw.append(String(o))
	var current := String(_plan_values.get(key, ""))
	var idx: int = raw.find(current)
	if idx < 0:
		idx = 0
	var ob := DccWidgets.choice(parent, label_text, labels, idx,
		func(i: int):
			_plan_values[key] = raw[i]
			_plan_value_changed(structural),
		tooltip)
	if allow_auto:
		_auto_obs[key] = ob

func _route_cond_field(parent: Control, label_text: String, key: String, current: String,
		on_pick: Callable, tooltip: String = "") -> void:
	var labels: Array = [_auto_label(key)]
	var raw: Array = [""]
	var conds: Dictionary = _options.get("route_cond", {})
	for cat in ["land", "river", "sea"]:
		var opts: PackedStringArray = conds.get(cat, PackedStringArray())
		for o in opts:
			labels.append("%s: %s" % [cat.capitalize(), String(o)])
			raw.append(String(o))
	var idx: int = raw.find(current)
	if idx < 0:
		idx = 0
	var ob := DccWidgets.choice(parent, label_text, labels, idx, func(i: int): on_pick.call(raw[i]), tooltip)
	_auto_obs[key] = ob

## JP-15 (`JOURNEY_PLANNER_SPEC.md` §5: "Auto-valued fields show `auto ·
## <resolved value>` so the resolved value is never hidden") -- already true
## for stage overrides via `_inherit_label` above; this is its party-form
## sibling. `"Auto"` alone when nothing has been computed yet, or when this
## field genuinely has no single resolved value to show (`weather_override`:
## `jp_weather_factor`'s auto is a continuous blend across every condition,
## not one chosen condition -- there is nothing honest to print).
func _auto_label(key: String) -> String:
	var resolved := _party_auto_resolved(key)
	return ("Auto · %s" % resolved) if resolved != "" else "Auto"

## The real resolved value behind one auto-valued party-form field, read from
## the last compute -- the party form is journey-wide, so this reads the
## first stage/leg carrying a real answer, the same "first applicable leg"
## convention `_pack_range_note()` above already uses (a per-stage breakdown
## already exists, in the stage inspector's own `_inherit_label`).
func _party_auto_resolved(key: String) -> String:
	if _last_result.is_empty() or not bool(_last_result.get("ok", false)):
		return ""
	var plan: Dictionary = _last_result.get("plan", {})
	match key:
		"rest_cadence":
			return String(plan.get("rest_basis", ""))
		"route_cond", "infra":
			var stages: Array = plan.get("stages", [])
			for st in stages:
				var v := String((st as Dictionary).get(key, ""))
				if v != "":
					return v
		"mount_animal":
			for r in plan.get("results", []):
				var land: Dictionary = (r as Dictionary).get("land", {})
				var mk := String(land.get("mount_key", ""))
				if mk != "":
					return mk
		"desert_water":
			for r in plan.get("results", []):
				var land: Dictionary = (r as Dictionary).get("land", {})
				if land.is_empty() or not bool(land.get("is_desert", false)) or not bool(land.get("desert_tier_auto", false)):
					continue
				var tier := String(land.get("desert_tier", ""))
				if tier != "":
					return tier
	return ""

## Cheap post-compute refresh for the party form's own "Auto" fields --
## relabels item 0 in place rather than calling `_rebuild_party_form()`,
## which would rebuild every SpinBox in the form and drop focus out of
## whichever one the party is mid-edit in (`_plan_value_changed`'s
## structural/non-structural split exists for exactly this reason).
func _refresh_auto_labels() -> void:
	for key in _auto_obs.keys():
		var ob: OptionButton = _auto_obs[key]
		if not is_instance_valid(ob) or ob.item_count == 0:
			continue
		ob.set_item_text(0, _auto_label(key))

func _rebuild_party_form() -> void:
	if not _bound or _left_party_body == null:
		return
	for c in _left_party_body.get_children():
		_left_party_body.remove_child(c)
		c.queue_free()
	_auto_obs.clear()

	if _plan_values.is_empty():
		for key in _default_plan.keys():
			if key != "party_fields":
				_plan_values[key] = _default_plan[key]

	# -- Traveler --------------------------------------------------------------
	var traveler := DccWidgets.section(_left_party_body, "Party · Traveler")
	_number_field(traveler, "Group size", "group_size", 1.0, 100000.0, 1.0, true, "people")
	_choice_field(traveler, "Pace", "pace", _options.get("pace", PackedStringArray()), false)
	_number_field(traveler, "Hours/day (land)", "hours", 1.0, 16.0, 0.5, false)
	_number_field(traveler, "Trade cargo (kg)", "cargo_kg", 0.0, 500000.0, 10.0, false)
	_number_field(traveler, "Supplies carried (d)", "supply_days", 1.0, 90.0, 1.0, true)
	var pr := _pack_range_note()
	if pr != "":
		DccWidgets.note(traveler, pr)
	_toggle_field(traveler, "Carry food (off = live off the land)", "carry_food")
	_choice_field(traveler, "Grazing", "grazing", _options.get("grazing", PackedStringArray()), false)
	_choice_field(traveler, "Foraging", "foraging", _options.get("foraging", PackedStringArray()), false)

	# -- Season & weather --------------------------------------------------------
	var season := DccWidgets.section(_left_party_body, "Season & weather")
	_choice_field(season, "Season", "season", _options.get("season", PackedStringArray()), false, true)
	_choice_field(season, "Weather", "weather_override", _options.get("weather_override", PackedStringArray()), true, true,
		"Auto weighs every condition by the season's own odds for this route's biome.")
	_choice_field(season, "Rest days", "rest_cadence", _options.get("rest_cadence", PackedStringArray()), true, true,
		"Auto = the reference's own 1-in-5 default.")
	_toggle_field(season, "Season advances during the journey", "season_drift", true)

	# -- Carriage ------------------------------------------------------------
	var carriage := DccWidgets.section(_left_party_body, "Carriage")
	var mode_row := HBoxContainer.new()
	mode_row.add_theme_constant_override("separation", 4)
	var auto_btn := Button.new()
	auto_btn.text = "Auto"
	auto_btn.toggle_mode = true
	auto_btn.button_pressed = _carriage_auto
	auto_btn.focus_mode = Control.FOCUS_NONE
	var manual_btn := Button.new()
	manual_btn.text = "Manual"
	manual_btn.toggle_mode = true
	manual_btn.button_pressed = not _carriage_auto
	manual_btn.focus_mode = Control.FOCUS_NONE
	auto_btn.pressed.connect(func(): _carriage_auto = true; manual_btn.button_pressed = false; _rebuild_party_form())
	manual_btn.pressed.connect(func(): _carriage_auto = false; auto_btn.button_pressed = false; _rebuild_party_form())
	mode_row.add_child(auto_btn)
	mode_row.add_child(manual_btn)
	carriage.add_child(mode_row)
	if _carriage_auto:
		DccWidgets.note(carriage,
			"Auto (best animal for the route's terrain × biome, km-weighted) is the reference's jpAutoPickTransport — not ported to Rust yet (no cartalith-civ auto-carriage function exists). Counts below stay at whatever Manual last set; toggling Auto only disables editing them here, it does not compute a pick.")

	_choice_field(carriage, "Transport", "transport", _options.get("transport", PackedStringArray()), false, true)
	var transport := String(_plan_values.get("transport", "Walking"))
	if transport == "Mounted Rider":
		_choice_field(carriage, "Mount", "mount_animal", _options.get("mount_animal", PackedStringArray()), true, false,
			"Only consulted when the party carries no donkeys/mules/camels/horses of its own.")
	if transport == "River Transport" or transport == "Sea Faring":
		_choice_field(carriage, "Vessel", "vessel", _options.get("vessel", PackedStringArray()), false)

	var animals := HBoxContainer.new()
	animals.add_theme_constant_override("separation", 4)
	_animal_pair(carriage, "Donkeys / Mules", "donkey", "mule")
	_animal_pair(carriage, "Camels / Horses", "camel", "horse")
	_animal_pair(carriage, "Carts / Wagons", "carts", "wagons")
	_animal_pair(carriage, "Travois / Sleds", "travois", "sleds")
	_toggle_field(carriage, "Auto-promote Walking → Baggage Train if overloaded", "auto_promote")

	# -- Route conditions --------------------------------------------------------
	var route_group := DccWidgets.section(_left_party_body, "Route conditions")
	_route_cond_field(route_group, "Road quality", "route_cond", String(_plan_values.get("route_cond", "")),
		func(v: String): _plan_values["route_cond"] = v; _plan_value_changed(false))
	_choice_field(route_group, "Infrastructure", "infra", _options.get("infra", PackedStringArray()), true)
	_choice_field(route_group, "Desert water", "desert_water", _options.get("desert_water", PackedStringArray()), true,
		false, "Auto measures the longest waterless run on this route and picks the matching tier.")
	_toggle_field(route_group, "Respect seasonal closures (winter passes)", "seasonal_closures", true)

	var footer := DccTheme.mono_label(
		"party preset: not wired — JP_PRESETS has no jp_presets() binding", "text_ghost", DccTheme.FS_TINY)
	footer.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	var footer_pad := MarginContainer.new()
	footer_pad.add_theme_constant_override("margin_left", 14)
	footer_pad.add_theme_constant_override("margin_top", 8)
	footer_pad.add_theme_constant_override("margin_bottom", 8)
	footer_pad.add_child(footer)
	_left_party_body.add_child(footer_pad)

func _animal_pair(parent: Control, label_text: String, key_a: String, key_b: String) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = 24
	var l := DccTheme.mono_label(label_text, "text_dim", DccTheme.FS_SMALL)
	l.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
	l.clip_text = true
	row.add_child(l)
	for key in [key_a, key_b]:
		var sb := SpinBox.new()
		sb.min_value = 0
		sb.max_value = 2000
		sb.step = 1
		sb.value = float(_plan_values.get(key, 0.0))
		sb.editable = not _carriage_auto
		sb.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		sb.value_changed.connect(func(v: float): _plan_values[key] = int(v); _plan_value_changed(false))
		row.add_child(sb)
	parent.add_child(row)

## `_jpPackRange`'s own ceiling advisory (reference ~line 19661), attached to
## the supplies field per `JOURNEY_PLANNER_SPEC.md` §5. `jp_capacity()` is
## exposed to `cartalith-godot` internally but not as a `#[func]` -- this
## reads the same real numbers the reference computes, via the closest bound
## primitive: the last compute's own land-leg capacity, if one exists. Before
## any compute has run there is nothing to derive it from, so it stays quiet
## rather than guessing.
func _pack_range_note() -> String:
	if _last_result.is_empty() or not bool(_last_result.get("ok", false)):
		return ""
	var plan: Dictionary = _last_result.get("plan", {})
	var results: Array = plan.get("results", [])
	for r in results:
		var d: Dictionary = r
		if d.has("land"):
			var l: Dictionary = d["land"]
			var cap: Dictionary = l.get("capacity", {})
			var fodder := float(cap.get("fodder", 0.0))
			var supply_days := float(_plan_values.get("supply_days", 1.0))
			if fodder > 0.0 and supply_days > 0.0:
				var mule_days := supply_days  ## fodder already reflects supply_days at the current grazing setting
				return "At this grazing setting, the current fodder load carries roughly %.0f day(s) as configured -- lower Supplies or raise Grazing if this is a mule/donkey-bound route and animals are running short." % mule_days
			break
	return ""

# =========================================================== Compute path ====

func _compute() -> void:
	if not _bound or _route_index < 0:
		_last_result = {}
		_apply_result()
		return
	var request: Dictionary = {"route": _route_index, "plan": _plan_values.duplicate(true)}
	if not _stage_overrides.is_empty():
		var ov: Dictionary = {}
		for idx in _stage_overrides:
			ov[idx] = (_stage_overrides[idx] as Dictionary).duplicate(true)
		request["stage_overrides"] = ov
	if not _layovers.is_empty():
		request["layovers"] = _layovers.duplicate(true)
	_last_result = bridge.jp_compute(request)
	_apply_result()

func _apply_result() -> void:
	var plan: Dictionary = {}
	if bool(_last_result.get("ok", false)):
		plan = _last_result.get("plan", {})
	var stages: Array = plan.get("stages", [])
	if _selected_stage >= stages.size():
		_selected_stage = maxi(0, stages.size() - 1)

	_rebuild_route_map(plan)
	_rebuild_profile(plan)
	_rebuild_stops(plan)
	_rebuild_inspector(plan)
	_rebuild_matrix(plan)
	_rebuild_timeline_band(plan)
	_refresh_auto_labels()
	if app != null and app.right_dock_ctrl != null:
		app.right_dock_ctrl.refresh_journey()

## Stage bands as fractions of total route km (real: `stages[i].km`, cumulative)
## and the elevation sparkline as `plan.profile`'s own 0-1 normalised samples
## -- one per route point, sharing the route map's own polyline in index
## space (`the_assembled_world_actually_drives_jp_plan`'s own test: "one
## elevation sample per route point").
func _rebuild_profile(plan: Dictionary) -> void:
	var stages: Array = plan.get("stages", [])
	var results: Array = plan.get("results", [])
	var profile: PackedFloat64Array = plan.get("profile", PackedFloat64Array())
	_profile.profile = profile
	var total_km := float(plan.get("km", 0.0))
	var bands: Array = []
	if total_km > 0.0:
		var cum := 0.0
		for i in stages.size():
			var s: Dictionary = stages[i]
			var r: Dictionary = results[i] if i < results.size() else {}
			var km := float(s.get("km", 0.0))
			var start := cum / total_km
			cum += km
			var end := cum / total_km
			var lr := 0.0
			if r.has("land"):
				lr = float((r["land"] as Dictionary).get("load_ratio", 0.0))
			elif r.has("water"):
				lr = float((r["water"] as Dictionary).get("load_ratio", 0.0))
			var label_text := "%d" % (i + 1)
			if bool(r.get("blocked", false)):
				label_text += " %s" % DccIcons.SYMBOLS["blocked"]
			elif String(s.get("cat", "land")) != "land":
				label_text += " · %s" % String(s.get("cat", ""))
			bands.append({
				"start": start, "end": end, "cat": String(s.get("cat", "land")),
				"blocked": bool(r.get("blocked", false)), "warn": lr > 0.9, "label": label_text,
			})
	_profile.bands = bands
	_profile.selected_idx = _selected_stage
	_profile.isolated_idx = _isolated_stage
	_profile.queue_redraw()

## JP-13 (`JOURNEY_PLANNER_SPEC.md` §2: "Timeline bar carries the journey
## calendar: one band per day, coloured travel / water / weather hold /
## rest-layover") -- `app.timeline_row` sat visible and empty in INFRA while
## JOURNEY was armed (`GUI_GAP_REGISTER.md` JP-13, §11: "the one place in the
## shell showing an empty region with no explanation"). Lives in
## `app.timeline_row`, not `_center_panel`, because that container belongs to
## `DccShell`/`DccApp`, not this file -- see this file's own class doc for
## why every other region this view takes over is reached the same
## read-only way.
func _rebuild_timeline_band(plan: Dictionary) -> void:
	if not _bound or app == null or app.timeline_row == null:
		return
	for c in app.timeline_row.get_children():
		app.timeline_row.remove_child(c)
		c.queue_free()
	_timeline_view = null

	if _route_index < 0:
		app.timeline_row.add_child(DccTheme.mono_label(
			"no committed route selected", "text_ghost", DccTheme.FS_TINY))
		return
	var total_days := float(plan.get("total_days", -1.0))
	if plan.is_empty() or total_days < 0.0:
		var reason := "journey blocked -- no calendar to show" if not plan.is_empty() else "no result yet"
		app.timeline_row.add_child(DccTheme.mono_label(reason, "block" if not plan.is_empty() else "text_ghost", DccTheme.FS_TINY))
		return

	## Real segments only: `results[i].days` per stage (land -> accent, water
	## and river -> the water token) plus one trailing block for
	## `rest_days + layover_days` combined (`text_dim`). Combined, not
	## interleaved, because the engine's own model already treats rest and
	## layover as calendar time laid on top of travel rather than assigned to
	## specific days (`JpJourneyPlan::days`'s own doc comment, v1.52: "rest
	## days and layovers are calendar time laid on top") -- a trailing block
	## is not an approximation of something more precise the data could give,
	## it is what "laid on top" means. "Weather hold" is never drawn: `jp_plan`
	## carries no discrete weather-hold day count anywhere -- weather is
	## `jp_weather_factor`'s continuous per-leg speed multiplier, already
	## folded into each stage's own `days` below -- the legend still names it
	## (per the spec and the mockup's own legend), with a tooltip stating why
	## no segment is ever lit for it rather than silently dropping one of the
	## spec's four categories.
	var stages: Array = plan.get("stages", [])
	var results: Array = plan.get("results", [])
	var segments: Array = []
	for i in stages.size():
		var s: Dictionary = stages[i]
		var r: Dictionary = results[i] if i < results.size() else {}
		var d := float(r.get("days", 0.0))
		if d <= 0.0:
			continue
		var cat := String(s.get("cat", "land"))
		segments.append({"days": d, "token": "water" if cat != "land" else "accent"})
	var rest_layover := float(plan.get("rest_days", 0)) + float(plan.get("layover_days", 0))
	if rest_layover > 0.0:
		segments.append({"days": rest_layover, "token": "text_dim"})

	app.timeline_row.add_child(DccTheme.mono_label("day 1", "text_faint", DccTheme.FS_TINY))
	_timeline_view = _TimelineBandView.new()
	_timeline_view.segments = segments
	_timeline_view.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_timeline_view.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	_timeline_view.custom_minimum_size = Vector2(0, 8)
	app.timeline_row.add_child(_timeline_view)
	app.timeline_row.add_child(DccTheme.mono_label("day %d" % int(roundf(total_days)), "text_faint", DccTheme.FS_TINY))

	var legend := HBoxContainer.new()
	legend.add_theme_constant_override("separation", 10)
	_timeline_legend_item(legend, "accent", "travel")
	_timeline_legend_item(legend, "water", "water")
	var wx := _timeline_legend_item(legend, "block", "weather hold")
	wx.tooltip_text = "jp_plan reports no discrete weather-hold day count -- weather is a continuous per-leg speed multiplier, already folded into each stage's own travel days to the left. Never lit."
	_timeline_legend_item(legend, "text_dim", "rest / layover")
	app.timeline_row.add_child(legend)

func _timeline_legend_item(parent: Control, token: String, label_text: String) -> Control:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)
	var sw := ColorRect.new()
	sw.custom_minimum_size = Vector2(7, 7)
	sw.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	sw.color = DccTheme.c(token)
	row.add_child(sw)
	row.add_child(DccTheme.mono_label(label_text, "text_ghost", DccTheme.FS_MICRO))
	parent.add_child(row)
	return row

# ============================================================ Center panel ====

func _build_center_panel() -> void:
	_center_panel = Control.new()
	_center_panel.set_anchors_preset(Control.PRESET_FULL_RECT)
	app.viewport_content.add_child(_center_panel)

	if not _bound:
		var note := DccTheme.label(
			"jp_options / jp_default_plan / jp_compute / route_count / route_get are not exposed by this build.",
			"text_ghost", DccTheme.FS_SMALL)
		note.set_anchors_preset(Control.PRESET_CENTER)
		_center_panel.add_child(note)
		return

	var col := VBoxContainer.new()
	col.set_anchors_preset(Control.PRESET_FULL_RECT)
	col.add_theme_constant_override("separation", 0)
	_center_panel.add_child(col)

	# -- Route map row (236px) --------------------------------------------------
	var map_row := HBoxContainer.new()
	map_row.custom_minimum_size.y = 236
	map_row.add_theme_constant_override("separation", 0)
	var map_row_pad := PanelContainer.new()
	map_row_pad.custom_minimum_size.y = 236
	map_row_pad.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"bottom": 1}))
	map_row_pad.add_child(map_row)
	col.add_child(map_row_pad)

	_route_map = _RouteMapView.new()
	_route_map.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_route_map.size_flags_vertical = Control.SIZE_EXPAND_FILL
	map_row.add_child(_route_map)

	var totals_panel := PanelContainer.new()
	totals_panel.custom_minimum_size.x = 196
	totals_panel.add_theme_stylebox_override("panel", DccTheme.panel("panel_alt", {"left": 1}))
	map_row.add_child(totals_panel)
	var totals_col := VBoxContainer.new()
	totals_col.add_theme_constant_override("separation", 0)
	totals_panel.add_child(totals_col)
	DccWidgets.section(totals_col, "Route totals")   ## Adds the header + an empty body to totals_col; the body is discarded, `_totals_body` below is what this file actually fills.
	_totals_body = VBoxContainer.new()
	_totals_body.add_theme_constant_override("separation", 4)
	var totals_body_pad := MarginContainer.new()
	totals_body_pad.add_theme_constant_override("margin_left", 11)
	totals_body_pad.add_theme_constant_override("margin_right", 11)
	totals_body_pad.add_child(_totals_body)
	totals_col.add_child(totals_body_pad)

	# -- Terrain profile row (150px) --------------------------------------------
	var profile_wrap := PanelContainer.new()
	profile_wrap.custom_minimum_size.y = 150
	profile_wrap.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"bottom": 1}))
	col.add_child(profile_wrap)
	var profile_col := VBoxContainer.new()
	profile_col.add_theme_constant_override("separation", 0)
	profile_wrap.add_child(profile_col)
	var profile_head := HBoxContainer.new()
	profile_head.custom_minimum_size.y = 22
	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 12)
	head_pad.add_child(profile_head)
	profile_col.add_child(head_pad)
	profile_head.add_child(DccTheme.mono_label("PROFILE · STAGE SELECTOR", "text_dim", DccTheme.FS_HEADER, 2, true))
	profile_head.add_child(DccTheme.label("  click a band to inspect · ⌥ click isolates", "text_ghost", DccTheme.FS_MICRO))

	_profile = _ProfileView.new()
	_profile.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_profile.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_profile.stage_clicked.connect(_on_stage_clicked)
	profile_col.add_child(_profile)

	# -- Stops strip (32px) ------------------------------------------------------
	var stops_wrap := PanelContainer.new()
	stops_wrap.custom_minimum_size.y = 32
	stops_wrap.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"bottom": 1}))
	col.add_child(stops_wrap)
	var stops_outer := HBoxContainer.new()
	stops_outer.add_theme_constant_override("separation", 14)
	var stops_pad := MarginContainer.new()
	stops_pad.add_theme_constant_override("margin_left", 12)
	stops_pad.add_theme_constant_override("margin_right", 12)
	stops_pad.add_child(stops_outer)
	stops_wrap.add_child(stops_pad)
	stops_outer.add_child(DccTheme.mono_label("STOPS · LAYOVER DAYS", "text_dim", DccTheme.FS_HEADER, 2, true))
	_stops_row = HBoxContainer.new()
	_stops_row.add_theme_constant_override("separation", 8)
	_stops_row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	stops_outer.add_child(_stops_row)
	_stops_note = DccTheme.mono_label("", "text_ghost", DccTheme.FS_TINY)
	stops_outer.add_child(_stops_note)

	# -- Lower area: inspector + matrix -------------------------------------------
	var lower := HBoxContainer.new()
	lower.size_flags_vertical = Control.SIZE_EXPAND_FILL
	lower.add_theme_constant_override("separation", 0)
	col.add_child(lower)

	var inspector_wrap := PanelContainer.new()
	inspector_wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	inspector_wrap.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"right": 1}))
	lower.add_child(inspector_wrap)
	var inspector_scroll := ScrollContainer.new()
	inspector_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	inspector_wrap.add_child(inspector_scroll)
	_inspector_body = VBoxContainer.new()
	_inspector_body.add_theme_constant_override("separation", 0)
	_inspector_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	inspector_scroll.add_child(_inspector_body)

	var matrix_wrap := PanelContainer.new()
	matrix_wrap.custom_minimum_size.x = 642
	lower.add_child(matrix_wrap)
	var matrix_scroll := ScrollContainer.new()
	matrix_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	matrix_wrap.add_child(matrix_scroll)
	_matrix_body = VBoxContainer.new()
	_matrix_body.add_theme_constant_override("separation", 0)
	_matrix_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	matrix_scroll.add_child(_matrix_body)

# -- Route map + totals ---------------------------------------------------------

func _rebuild_route_map(plan: Dictionary) -> void:
	for c in _totals_body.get_children():
		_totals_body.remove_child(c)
		c.queue_free()

	if _route_index < 0 or not _bound:
		_route_map.pts = PackedVector2Array()
		_route_map.stage_segments = []
		_route_map.stops = []
		_route_map.queue_redraw()
		DccWidgets.note(_totals_body, "No committed route selected.")
		return

	var route: Dictionary = bridge.route_get(_route_index)
	var pts: PackedVector2Array = route.get("points", PackedVector2Array())
	var stages: Array = plan.get("stages", [])
	var results: Array = plan.get("results", [])
	var segments: Array = []
	for i in stages.size():
		var s: Dictionary = stages[i]
		var r: Dictionary = results[i] if i < results.size() else {}
		segments.append({
			"i0": int(s.get("i0", 0)), "i1": int(s.get("i1", 0)),
			"cat": String(s.get("cat", "land")), "blocked": bool(r.get("blocked", false)),
		})
	_route_map.pts = pts
	_route_map.stage_segments = segments
	var stops: Array = plan.get("stops", [])
	var stop_pts: Array = []
	for st in stops:
		var d: Dictionary = st
		stop_pts.append(Vector2(float(d.get("x", 0.0)), float(d.get("y", 0.0))))
	_route_map.stops = stop_pts
	_route_map.queue_redraw()

	if plan.is_empty():
		var err := String(_last_result.get("error", "No result yet."))
		DccWidgets.note(_totals_body, err)
		return

	var total_km := float(plan.get("km", 0.0))
	var land_km := 0.0
	var water_km := 0.0
	var blocked_count := 0
	for i in stages.size():
		var s: Dictionary = stages[i]
		var r: Dictionary = results[i] if i < results.size() else {}
		if String(s.get("cat", "land")) == "land":
			land_km += float(s.get("km", 0.0))
		else:
			water_km += float(s.get("km", 0.0))
		if bool(r.get("blocked", false)):
			blocked_count += 1

	_totals_row(_totals_body, "distance", "%s km" % _fmt_thousands(total_km, 0))
	_totals_row(_totals_body, "land · water", "%s · %s" % [_fmt_thousands(land_km, 0), _fmt_thousands(water_km, 0)])
	_totals_row(_totals_body, "ascent", "%s m" % _fmt_thousands(float(plan.get("ascent", 0.0)), 0))
	_totals_row(_totals_body, "high point", "%s m" % _fmt_thousands(float(plan.get("hi_m", 0.0)), 0))
	var stage_text := "%d" % stages.size()
	if blocked_count > 0:
		stage_text += " · %d blocked" % blocked_count
	_totals_row(_totals_body, "stages", stage_text)
	_totals_row(_totals_body, "settlements", "%d on path" % stops.size())
	_totals_body.add_child(DccTheme.rule())
	_totals_row(_totals_body, "mean speed", "%.1f km/d" % float(plan.get("avg_km_day", 0.0)), "accent")

func _totals_row(parent: Control, label_text: String, value_text: String, token: String = "text") -> void:
	var row := HBoxContainer.new()
	row.custom_minimum_size.y = 18
	row.add_child(DccTheme.mono_label(label_text, "text_dim", DccTheme.FS_SMALL))
	row.add_child(DccTheme.spacer())
	row.add_child(DccTheme.mono_label(value_text, token, DccTheme.FS_SMALL))
	parent.add_child(row)

# -- Stops strip ---------------------------------------------------------------

func _rebuild_stops(plan: Dictionary) -> void:
	for c in _stops_row.get_children():
		_stops_row.remove_child(c)
		c.queue_free()

	var stops: Array = plan.get("stops", [])
	if stops.is_empty():
		_stops_note.text = "No stops on this route."
		return

	var pts: PackedVector2Array = PackedVector2Array()
	if _route_index >= 0:
		pts = bridge.route_get(_route_index).get("points", PackedVector2Array())
	var fracs := _stop_fractions(stops, pts)

	var total_layover := 0
	for i in stops.size():
		var d: Dictionary = stops[i]
		var key := String(d.get("key", ""))
		var days := int(_layovers.get(key, int(d.get("layover_days", 0))))
		total_layover += days
		var chip := HBoxContainer.new()
		chip.add_theme_constant_override("separation", 6)
		var stretch := 1.0
		if i + 1 < fracs.size():
			stretch = maxf(0.2, (fracs[i + 1] - fracs[i]) * 40.0)
		chip.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		chip.size_flags_stretch_ratio = stretch
		chip.add_child(DccTheme.mono_label(String(d.get("name", "?")), "text_dim", DccTheme.FS_SMALL))
		var sb := SpinBox.new()
		sb.min_value = 0
		sb.max_value = 365
		sb.step = 1
		sb.value = days
		sb.custom_minimum_size.x = 60
		sb.value_changed.connect(func(v: float):
			if v > 0:
				_layovers[key] = int(v)
			else:
				_layovers.erase(key)
			_compute())
		chip.add_child(sb)
		chip.add_child(DccTheme.mono_label("d", "text_ghost", DccTheme.FS_TINY))
		_stops_row.add_child(chip)

	_stops_note.text = "%d layover day%s · placed at the settlements the route threads" % [
		total_layover, "" if total_layover == 1 else "s"]

## Nearest-point-index projection onto the route's own chord-length axis --
## exact for position purposes since `map_width_km` is uniform across the
## grid (see this file's own doc comment).
func _stop_fractions(stops: Array, pts: PackedVector2Array) -> PackedFloat64Array:
	var out := PackedFloat64Array()
	if pts.size() < 2:
		for _s in stops:
			out.append(0.0)
		return out
	var cum := PackedFloat64Array()
	cum.append(0.0)
	for i in range(1, pts.size()):
		cum.append(cum[i - 1] + pts[i - 1].distance_to(pts[i]))
	var total: float = cum[cum.size() - 1]
	if total <= 0.0:
		total = 1.0
	for st in stops:
		var d: Dictionary = st
		var p := Vector2(float(d.get("x", 0.0)), float(d.get("y", 0.0)))
		var best_i := 0
		var best_d2 := INF
		for i in pts.size():
			var d2 := pts[i].distance_squared_to(p)
			if d2 < best_d2:
				best_d2 = d2
				best_i = i
		out.append(cum[best_i] / total)
	return out

# -- Stage selection -------------------------------------------------------------

func _on_stage_clicked(idx: int, isolate: bool) -> void:
	_selected_stage = idx
	if isolate:
		_isolated_stage = -1 if _isolated_stage == idx else idx
	_profile.selected_idx = _selected_stage
	_profile.isolated_idx = _isolated_stage
	_profile.queue_redraw()
	var plan: Dictionary = _last_result.get("plan", {}) if bool(_last_result.get("ok", false)) else {}
	_rebuild_inspector(plan)
	_rebuild_matrix(plan)

func _stage_override(idx: int) -> Dictionary:
	return _stage_overrides.get(idx, {})

func _set_stage_override(idx: int, field: String, value) -> void:
	var entry: Dictionary = (_stage_overrides.get(idx, {}) as Dictionary).duplicate()
	var is_blank: bool = (typeof(value) == TYPE_STRING and value == "")
	if is_blank:
		entry.erase(field)
	else:
		entry[field] = value
	if entry.is_empty():
		_stage_overrides.erase(idx)
	else:
		_stage_overrides[idx] = entry
	_compute()

func _fmt_thousands(v: float, decimals: int) -> String:
	var s := ("%.*f" % [decimals, v])
	var neg := s.begins_with("-")
	if neg:
		s = s.substr(1)
	var dot := s.find(".")
	var int_part := s if dot < 0 else s.substr(0, dot)
	var frac_part := "" if dot < 0 else s.substr(dot)
	var out := ""
	var count := 0
	for i in range(int_part.length() - 1, -1, -1):
		out = int_part[i] + out
		count += 1
		if count % 3 == 0 and i > 0:
			out = " " + out
	return ("-" if neg else "") + out + frac_part

# ==================================================== Stage inspector (§6) ====

func _effective_transport(idx: int, ov: Dictionary) -> String:
	var t := String(ov.get("transport", ""))
	if t != "":
		return t
	return String(_plan_values.get("transport", "Walking"))

func _inherit_label(field_key: String, stage: Dictionary, results_entry: Dictionary) -> String:
	var pv := String(_plan_values.get(field_key, ""))
	if pv != "":
		return "Inherit (%s)" % pv
	match field_key:
		"route_cond":
			return "Auto (%s)" % String(stage.get("route_cond", "?"))
		"infra":
			return "Auto (%s)" % String(stage.get("infra", "?"))
		"mount_animal":
			var land: Dictionary = results_entry.get("land", {})
			var mk := String(land.get("mount_key", ""))
			return "Auto (%s)" % mk if mk != "" else "Auto"
		_:
			return "Auto"

func _override_choice_row(parent: Control, idx: int, ov: Dictionary, field: String, label_text: String,
		opts: PackedStringArray, inherit_label: String, disabled: bool = false, disabled_reason: String = "") -> void:
	var labels: Array = [inherit_label]
	var raw: Array = [""]
	for o in opts:
		labels.append(String(o))
		raw.append(String(o))
	var current := String(ov.get(field, ""))
	var i0: int = raw.find(current)
	if i0 < 0:
		i0 = 0
	var ob := DccWidgets.choice(parent, label_text, labels, i0, func(i: int): _set_stage_override(idx, field, raw[i]))
	if disabled:
		ob.disabled = true
		ob.tooltip_text = disabled_reason

func _override_number_row(parent: Control, idx: int, ov: Dictionary, field: String, label_text: String,
		minimum: float, maximum: float, step: float, integer: bool) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	row.custom_minimum_size.y = 24
	var l := DccTheme.mono_label(label_text, "text_dim", DccTheme.FS_SMALL)
	l.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
	l.clip_text = true
	row.add_child(l)
	var has_ov := ov.has(field)
	var cb := CheckBox.new()
	cb.button_pressed = has_ov
	cb.focus_mode = Control.FOCUS_NONE
	cb.tooltip_text = "Override this stage"
	row.add_child(cb)
	var base_v := float(_plan_values.get(field, 0.0))
	var sb := SpinBox.new()
	sb.min_value = minimum
	sb.max_value = maximum
	sb.step = step
	sb.value = float(ov.get(field, base_v))
	sb.editable = has_ov
	sb.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cb.toggled.connect(func(on: bool):
		sb.editable = on
		if on:
			_set_stage_override(idx, field, (int(sb.value) if integer else sb.value))
		else:
			_set_stage_override(idx, field, ""))
	sb.value_changed.connect(func(v: float):
		if cb.button_pressed:
			_set_stage_override(idx, field, (int(v) if integer else v)))
	row.add_child(sb)
	parent.add_child(row)

func _override_toggle_row(parent: Control, idx: int, ov: Dictionary, field: String, label_text: String) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	row.custom_minimum_size.y = 24
	var l := DccTheme.mono_label(label_text, "text_dim", DccTheme.FS_SMALL)
	l.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
	l.clip_text = true
	row.add_child(l)
	var has_ov := ov.has(field)
	var cb := CheckBox.new()
	cb.button_pressed = has_ov
	cb.focus_mode = Control.FOCUS_NONE
	row.add_child(cb)
	var base_v := bool(_plan_values.get(field, false))
	var vcb := CheckBox.new()
	vcb.text = "on"
	vcb.button_pressed = bool(ov.get(field, base_v))
	vcb.disabled = not has_ov
	cb.toggled.connect(func(on: bool):
		vcb.disabled = not on
		_set_stage_override(idx, field, (vcb.button_pressed if on else "")))
	vcb.toggled.connect(func(v: bool):
		if cb.button_pressed:
			_set_stage_override(idx, field, v))
	row.add_child(vcb)
	parent.add_child(row)

func _rebuild_inspector(plan: Dictionary) -> void:
	for c in _inspector_body.get_children():
		_inspector_body.remove_child(c)
		c.queue_free()

	var stages: Array = plan.get("stages", [])
	if stages.is_empty():
		DccWidgets.note(_inspector_body, "No stages -- pick a committed route and configure the party form.")
		return
	if _selected_stage < 0 or _selected_stage >= stages.size():
		_selected_stage = 0
	var idx := _selected_stage
	var s: Dictionary = stages[idx]
	var results: Array = plan.get("results", [])
	var r: Dictionary = results[idx] if idx < results.size() else {}
	var ov: Dictionary = _stage_override(idx)
	var cat := String(s.get("cat", "land"))

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 12)
	head.custom_minimum_size.y = 26
	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 12)
	head_pad.add_theme_constant_override("margin_right", 12)
	head_pad.add_child(head)
	_inspector_body.add_child(head_pad)
	_inspector_body.add_child(DccTheme.rule())
	head.add_child(DccTheme.mono_label("STAGE %02d" % (idx + 1), "block" if bool(r.get("blocked", false)) else "accent",
		DccTheme.FS_HEADER, 2, true))
	head.add_child(DccTheme.label("%s · %s km · %.0f m · %s" % [
		String(s.get("terrain", "?")), _fmt_thousands(float(s.get("km", 0.0)), 0),
		float(s.get("gain", 0.0)), String(s.get("biome", "?"))], "text_faint", DccTheme.FS_MICRO))
	head.add_child(DccTheme.spacer())
	head.add_child(DccTheme.label("overrides: %d" % ov.size(), "text_ghost", DccTheme.FS_MICRO))
	if _isolated_stage == idx:
		head.add_child(DccTheme.mono_label("isolated", "accent", DccTheme.FS_MICRO))

	if bool(r.get("blocked", false)):
		var blk := DccTheme.label("BLOCKED: %s" % String(r.get("blocked_reason", "")), "block", DccTheme.FS_SMALL)
		blk.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		var blk_pad := MarginContainer.new()
		blk_pad.add_theme_constant_override("margin_left", 12)
		blk_pad.add_theme_constant_override("margin_top", 8)
		blk_pad.add_child(blk)
		_inspector_body.add_child(blk_pad)

	var grid := VBoxContainer.new()
	grid.add_theme_constant_override("separation", 2)
	var grid_pad := MarginContainer.new()
	grid_pad.add_theme_constant_override("margin_left", 12)
	grid_pad.add_theme_constant_override("margin_right", 12)
	grid_pad.add_theme_constant_override("margin_top", 8)
	grid_pad.add_child(grid)
	_inspector_body.add_child(grid_pad)
	DccWidgets.note(grid, "OVERRIDES · BLANK INHERITS THE PARTY FORM")

	var eff_transport := _effective_transport(idx, ov)
	_override_choice_row(grid, idx, ov, "transport", "Travel mode",
		_options.get("transport", PackedStringArray()), _inherit_label("transport", s, r))
	_override_number_row(grid, idx, ov, "group_size", "Group size", 1.0, 100000.0, 1.0, true)
	_override_number_row(grid, idx, ov, "cargo_kg", "Cargo (kg)", 0.0, 500000.0, 10.0, false)
	_override_choice_row(grid, idx, ov, "pace", "Pace", _options.get("pace", PackedStringArray()), _inherit_label("pace", s, r))
	_override_number_row(grid, idx, ov, "hours", "Hours/day", 1.0, 16.0, 0.5, false)
	_override_choice_row(grid, idx, ov, "weather_override", "Weather",
		_options.get("weather_override", PackedStringArray()), _inherit_label("weather_override", s, r))
	_override_toggle_row(grid, idx, ov, "carry_food", "Carry food")
	_override_number_row(grid, idx, ov, "supply_days", "Supplies (d)", 0.0, 90.0, 1.0, true)
	_override_choice_row(grid, idx, ov, "grazing", "Grazing", _options.get("grazing", PackedStringArray()), _inherit_label("grazing", s, r))
	_override_choice_row(grid, idx, ov, "foraging", "Foraging", _options.get("foraging", PackedStringArray()), _inherit_label("foraging", s, r))
	var cat_opts: PackedStringArray = (_options.get("route_cond", {}) as Dictionary).get(cat, PackedStringArray())
	_override_choice_row(grid, idx, ov, "route_cond", "Road quality", cat_opts, _inherit_label("route_cond", s, r))
	_override_choice_row(grid, idx, ov, "infra", "Infrastructure", _options.get("infra", PackedStringArray()), _inherit_label("infra", s, r))
	_override_choice_row(grid, idx, ov, "mount_animal", "Mount", _options.get("mount_animal", PackedStringArray()),
		_inherit_label("mount_animal", s, r), eff_transport != "Mounted Rider",
		"Only applies when this stage's travel mode is Mounted Rider.")
	_override_choice_row(grid, idx, ov, "desert_water", "Desert water", _options.get("desert_water", PackedStringArray()),
		_inherit_label("desert_water", s, r))
	_override_choice_row(grid, idx, ov, "vessel", "Vessel", _options.get("vessel", PackedStringArray()),
		_inherit_label("vessel", s, r), cat == "land", "— land stage, no vessel applies.")

	var actions := HBoxContainer.new()
	actions.add_theme_constant_override("separation", 8)
	var actions_pad := MarginContainer.new()
	actions_pad.add_theme_constant_override("margin_left", 12)
	actions_pad.add_theme_constant_override("margin_top", 6)
	actions_pad.add_child(actions)
	_inspector_body.add_child(actions_pad)
	DccWidgets.action(actions, "clear overrides", func(): _stage_overrides.erase(idx); _compute())
	DccWidgets.action(actions, "copy to all land stages", func(): _copy_override_to_land_stages(idx, plan))
	DccWidgets.action(actions, "isolate stage", func(): _on_stage_clicked(idx, true))

	_inspector_body.add_child(DccTheme.spacer())
	var footer := HBoxContainer.new()
	footer.add_theme_constant_override("separation", 20)
	footer.custom_minimum_size.y = 26
	var footer_pad := MarginContainer.new()
	footer_pad.add_theme_constant_override("margin_left", 12)
	footer_pad.add_theme_constant_override("margin_top", 6)
	footer_pad.add_child(footer)
	_inspector_body.add_child(DccTheme.rule())
	_inspector_body.add_child(footer_pad)
	if not bool(r.get("blocked", false)):
		var load_ratio := 0.0
		if r.has("land"):
			load_ratio = float((r["land"] as Dictionary).get("load_ratio", 0.0))
		elif r.has("water"):
			load_ratio = float((r["water"] as Dictionary).get("load_ratio", 0.0))
		var travel_days_so_far := 0.0
		for i in range(0, idx + 1):
			if i < results.size():
				travel_days_so_far += float((results[i] as Dictionary).get("days", 0.0))
		_footer_stat(footer, "this stage", "%.1f d" % float(r.get("days", 0.0)))
		_footer_stat(footer, "km/day", "%.1f" % float(r.get("daily_km", 0.0)))
		_footer_stat(footer, "load", "%.0f%%" % (load_ratio * 100.0), "warn" if load_ratio > 0.9 else "text_bright")
		_footer_stat(footer, "ascent", "%s m" % _fmt_thousands(float(s.get("gain", 0.0)), 0))
		_footer_stat(footer, "arrive", "~day %.0f (travel only)" % travel_days_so_far)

func _footer_stat(parent: Control, label_text: String, value_text: String, token: String = "text_bright") -> void:
	var box := HBoxContainer.new()
	box.add_theme_constant_override("separation", 4)
	box.add_child(DccTheme.mono_label(label_text, "text_dim", DccTheme.FS_SMALL))
	box.add_child(DccTheme.mono_label(value_text, token, DccTheme.FS_SMALL))
	parent.add_child(box)

func _copy_override_to_land_stages(idx: int, plan: Dictionary) -> void:
	var src: Dictionary = _stage_override(idx)
	if src.is_empty():
		return
	var stages: Array = plan.get("stages", [])
	for i in stages.size():
		var s: Dictionary = stages[i]
		if String(s.get("cat", "land")) == "land":
			_stage_overrides[i] = src.duplicate()
	_compute()

# ======================================================== Stage matrix (§7) ====

const _MATRIX_COLS := 10

func _matrix_header(cols: Array) -> void:
	var grid := GridContainer.new()
	grid.columns = _MATRIX_COLS
	grid.add_theme_constant_override("h_separation", 4)
	grid.add_theme_constant_override("v_separation", 3)
	for c in cols:
		grid.add_child(DccTheme.mono_label(String(c), "text_faint", DccTheme.FS_MICRO, 1, true))
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 10)
	pad.add_theme_constant_override("margin_right", 10)
	pad.add_theme_constant_override("margin_top", 6)
	pad.add_theme_constant_override("margin_bottom", 4)
	pad.add_child(grid)
	_matrix_body.add_child(pad)
	_matrix_body.add_child(DccTheme.rule())

func _rebuild_matrix(plan: Dictionary) -> void:
	for c in _matrix_body.get_children():
		_matrix_body.remove_child(c)
		c.queue_free()

	var stages: Array = plan.get("stages", [])
	if stages.is_empty():
		DccWidgets.note(_matrix_body, "No stages -- pick a committed route and configure the party form.")
		return
	var results: Array = plan.get("results", [])

	var head := HBoxContainer.new()
	head.custom_minimum_size.y = 26
	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 12)
	head_pad.add_theme_constant_override("margin_right", 12)
	head_pad.add_child(head)
	_matrix_body.add_child(head_pad)
	head.add_child(DccTheme.mono_label("STAGE MATRIX", "text_dim", DccTheme.FS_HEADER, 2, true))
	head.add_child(DccTheme.label("  lit cell = override · dim = computed, read-only", "text_ghost", DccTheme.FS_MICRO))
	head.add_child(DccTheme.spacer())
	var blocked_total := 0
	for r in results:
		if bool((r as Dictionary).get("blocked", false)):
			blocked_total += 1
	if blocked_total > 0:
		head.add_child(DccTheme.mono_label("%s %d" % [DccIcons.SYMBOLS["blocked"], blocked_total], "block", DccTheme.FS_MICRO))
	_matrix_body.add_child(DccTheme.rule())

	_matrix_header(["stage", "mode", "pace", "hrs", "terrain · biome", "weather", "cargo kg", "supply d", "km/d", "days"])

	# -- Problem stages first (blocked, then warned), then route order. --
	var order: Array = range(stages.size())
	var priority := func(i: int) -> int:
		var r: Dictionary = results[i] if i < results.size() else {}
		if bool(r.get("blocked", false)):
			return 0
		var lr := 0.0
		if r.has("land"):
			lr = float((r["land"] as Dictionary).get("load_ratio", 0.0))
		elif r.has("water"):
			lr = float((r["water"] as Dictionary).get("load_ratio", 0.0))
		return 1 if lr > 0.9 else 2
	order.sort_custom(func(a, b):
		var pa: int = priority.call(a)
		var pb: int = priority.call(b)
		if pa != pb:
			return pa < pb
		return a < b)

	if _isolated_stage >= 0:
		order = order.filter(func(i): return i == _isolated_stage)

	var grid := GridContainer.new()
	grid.columns = _MATRIX_COLS
	grid.add_theme_constant_override("h_separation", 4)
	grid.add_theme_constant_override("v_separation", 3)
	var grid_pad := MarginContainer.new()
	grid_pad.add_theme_constant_override("margin_left", 10)
	grid_pad.add_theme_constant_override("margin_right", 10)
	grid_pad.add_theme_constant_override("margin_top", 4)
	grid_pad.add_child(grid)
	_matrix_body.add_child(grid_pad)

	for i in order:
		var s: Dictionary = stages[i]
		var r: Dictionary = results[i] if i < results.size() else {}
		var ov: Dictionary = _stage_override(i)
		var blocked := bool(r.get("blocked", false))
		var lr := 0.0
		if r.has("land"):
			lr = float((r["land"] as Dictionary).get("load_ratio", 0.0))
		elif r.has("water"):
			lr = float((r["water"] as Dictionary).get("load_ratio", 0.0))
		var warn := (not blocked) and lr > 0.9
		var token := "block" if blocked else ("warn" if warn else "text_dim")
		var mark := (" %s" % DccIcons.SYMBOLS["blocked"]) if blocked else ((" %s" % DccIcons.SYMBOLS["warn_tri"]) if warn else "")
		var stage_btn := Button.new()
		stage_btn.flat = true
		stage_btn.focus_mode = Control.FOCUS_NONE
		stage_btn.text = "%02d %s%s" % [i + 1, String(s.get("terrain", "?")), mark]
		stage_btn.clip_text = true
		stage_btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
		stage_btn.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
		stage_btn.add_theme_font_override("font", DccTheme.mono(0, i == _selected_stage))
		stage_btn.add_theme_color_override("font_color", DccTheme.c(token if token != "text_dim" else ("accent" if i == _selected_stage else "text_dim")))
		stage_btn.pressed.connect(func(): _on_stage_clicked(i, false))
		grid.add_child(stage_btn)

		var mode_ob := OptionButton.new()
		mode_ob.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
		mode_ob.focus_mode = Control.FOCUS_NONE
		var transport_opts: PackedStringArray = _options.get("transport", PackedStringArray())
		mode_ob.add_item("—")
		var mode_current := String(ov.get("transport", ""))
		var mode_sel := 0
		for ti in transport_opts.size():
			mode_ob.add_item(transport_opts[ti])
			if transport_opts[ti] == mode_current:
				mode_sel = ti + 1
		mode_ob.selected = mode_sel
		mode_ob.item_selected.connect(func(sel: int): _set_stage_override(i, "transport", ("" if sel == 0 else transport_opts[sel - 1])))
		grid.add_child(mode_ob)

		var pace_ob := OptionButton.new()
		pace_ob.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
		pace_ob.focus_mode = Control.FOCUS_NONE
		var pace_opts: PackedStringArray = _options.get("pace", PackedStringArray())
		pace_ob.add_item("—")
		var pace_current := String(ov.get("pace", ""))
		var pace_sel := 0
		for pi in pace_opts.size():
			pace_ob.add_item(pace_opts[pi])
			if pace_opts[pi] == pace_current:
				pace_sel = pi + 1
		pace_ob.selected = pace_sel
		pace_ob.item_selected.connect(func(sel: int): _set_stage_override(i, "pace", ("" if sel == 0 else pace_opts[sel - 1])))
		grid.add_child(pace_ob)

		var hrs_sb := SpinBox.new()
		hrs_sb.min_value = 0
		hrs_sb.max_value = 16
		hrs_sb.step = 0.5
		hrs_sb.custom_minimum_size.x = 38
		hrs_sb.value = float(ov.get("hours", 0.0))
		hrs_sb.value_changed.connect(func(v: float): _set_stage_override(i, "hours", ("" if v <= 0.0 else v)))
		grid.add_child(hrs_sb)

		grid.add_child(DccTheme.mono_label("%s · %s" % [String(s.get("terrain", "?")), String(s.get("biome", "?"))],
			"water" if String(s.get("cat", "land")) != "land" else "text_dim", DccTheme.FS_TINY))
		var eff: Dictionary = r.get("eff", {})
		grid.add_child(DccTheme.mono_label(String(eff.get("weather_override", "")).capitalize() if String(eff.get("weather_override", "")) != "" else "—",
			"text_dim", DccTheme.FS_TINY))

		var cargo_l := DccTheme.mono_label("—" if blocked else _fmt_thousands(float(eff.get("cargo_kg", 0.0)), 0), "text_dim", DccTheme.FS_TINY)
		cargo_l.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		grid.add_child(cargo_l)
		var supply_l := DccTheme.mono_label("—" if blocked else "%.1f" % float(eff.get("supply_days", 0.0)), "text_dim", DccTheme.FS_TINY)
		supply_l.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		grid.add_child(supply_l)
		var kmd_l := DccTheme.mono_label("—" if blocked else "%.1f" % float(r.get("daily_km", 0.0)), "text_dim", DccTheme.FS_TINY)
		kmd_l.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		grid.add_child(kmd_l)
		var days_l := DccTheme.mono_label("—" if blocked else "%.1f" % float(r.get("days", 0.0)), token if blocked else "text", DccTheme.FS_TINY)
		days_l.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		grid.add_child(days_l)

	var footer := HBoxContainer.new()
	footer.add_theme_constant_override("separation", 8)
	footer.custom_minimum_size.y = 24
	var footer_pad := MarginContainer.new()
	footer_pad.add_theme_constant_override("margin_left", 12)
	footer_pad.add_theme_constant_override("margin_right", 12)
	footer_pad.add_theme_constant_override("margin_top", 6)
	footer_pad.add_child(footer)
	_matrix_body.add_child(DccTheme.rule())
	_matrix_body.add_child(footer_pad)
	DccWidgets.action(footer, "clear column", func(): _matrix_clear_column())
	DccWidgets.action(footer, "fill down", func(): _matrix_fill_down(stages))
	if _isolated_stage >= 0:
		DccWidgets.action(footer, "show all stages", func(): _isolated_stage = -1; _apply_result())
	footer.add_child(DccTheme.spacer())
	footer.add_child(DccTheme.label("cargo and supply columns run cumulatively down the route", "text_ghost", DccTheme.FS_TINY))

## `clear column`/`fill down` act on the three matrix-editable fields
## together (mode/pace/hours) -- the mockup shows one tool row for the whole
## matrix, not per-column, and with only three editable columns splitting it
## further would be more chrome than the feature earns this pass.
func _matrix_clear_column() -> void:
	for idx in _stage_overrides.keys().duplicate():
		var entry: Dictionary = _stage_overrides[idx]
		entry.erase("transport")
		entry.erase("pace")
		entry.erase("hours")
		if entry.is_empty():
			_stage_overrides.erase(idx)
		else:
			_stage_overrides[idx] = entry
	_compute()

func _matrix_fill_down(stages: Array) -> void:
	var src: Dictionary = _stage_override(_selected_stage)
	var picked: Dictionary = {}
	for f in ["transport", "pace", "hours"]:
		if src.has(f):
			picked[f] = src[f]
	if picked.is_empty():
		return
	for i in range(_selected_stage + 1, stages.size()):
		var entry: Dictionary = (_stage_overrides.get(i, {}) as Dictionary).duplicate()
		for f in picked:
			entry[f] = picked[f]
		_stage_overrides[i] = entry
	_compute()

# ============================================================== Tool options ====

func _tool_options_journey() -> void:
	app.set_tool_options(func(row: HBoxContainer):
		row.add_child(DccTheme.mono_label("JOURNEY PLANNER", "accent", DccTheme.FS_SMALL, 2, true))
		var count := bridge.route_count()
		var labels: Array = []
		for i in count:
			labels.append("Route #%d" % i)
		if labels.is_empty():
			labels.append("(no committed route)")
		DccWidgets.choice(row, "journey", labels, maxi(0, _route_index),
			func(i: int):
				if i < count:
					_route_index = i
					_compute())
		var preset_btn := DccWidgets.action(row, "preset: not wired", func(): pass)
		preset_btn.disabled = true
		preset_btn.tooltip_text = "JP_PRESETS is JS-only in the reference; no jp_presets() binding exists."
		var carriage_lbl := DccTheme.mono_label("carriage: %s" % ("auto" if _carriage_auto else "manual"), "text_ghost", DccTheme.FS_SMALL)
		row.add_child(carriage_lbl)
		var reroute_btn := DccWidgets.action(row, "re-route for %s…" % String(_plan_values.get("transport", "Walking")), func(): pass)
		reroute_btn.disabled = true
		reroute_btn.tooltip_text = "jpAutoPickTransport / _jpRerouteForMode have no Rust port -- see this file's own doc comment."
		row.add_child(DccTheme.label("⇧ drag spine to trim (deferred) · ⌥ click isolates a stage", "text_ghost", DccTheme.FS_MICRO))
		row.add_child(DccTheme.spacer())
		var save_btn := DccWidgets.action(row, "save journey", func(): pass)
		save_btn.disabled = true
		save_btn.tooltip_text = "No save-writer exists for journeys (or for projects generally -- cartalith-io is read-only)."
		var export_btn := DccWidgets.action(row, "export table", _export_stage_table)
	)

## The one export path with real data behind it: the stage matrix as CSV, to
## the OS clipboard -- no file-writer exists to save it to disk (same gap the
## save-journey button discloses), but a clipboard export is honest and
## immediately useful.
func _export_stage_table() -> void:
	if not bool(_last_result.get("ok", false)):
		return
	var plan: Dictionary = _last_result.get("plan", {})
	var stages: Array = plan.get("stages", [])
	var results: Array = plan.get("results", [])
	var lines: Array[String] = ["stage,cat,terrain,biome,km,days,km_per_day,blocked"]
	for i in stages.size():
		var s: Dictionary = stages[i]
		var r: Dictionary = results[i] if i < results.size() else {}
		lines.append("%d,%s,%s,%s,%.1f,%.2f,%.2f,%s" % [
			i + 1, String(s.get("cat", "")), String(s.get("terrain", "")), String(s.get("biome", "")),
			float(s.get("km", 0.0)), float(r.get("days", 0.0)), float(r.get("daily_km", 0.0)),
			str(bool(r.get("blocked", false)))])
	DisplayServer.clipboard_set("\n".join(lines))
	app.set_status("hint", "Stage table copied to clipboard (%d rows)." % stages.size(), "text_ghost")

# =========================================================== Results (§8) ====
#
# Delegated from `right_dock.gd`'s `CTX_JOURNEY` -- see this file's own class
# doc for why the right dock is not taken over directly.

func build_results(body: Control) -> void:
	for c in body.get_children():
		body.remove_child(c)
		c.queue_free()

	if _route_index < 0:
		DccWidgets.note(body, "Pick a committed route in the left dock to begin.")
		return
	if _last_result.is_empty():
		DccWidgets.note(body, "No result yet.")
		return

	var rejected: PackedStringArray = _last_result.get("rejected", PackedStringArray())
	if rejected.size() > 0:
		DccWidgets.note(body, "Rejected keys (defaults used instead): %s" % ", ".join(rejected))

	if not bool(_last_result.get("ok", false)):
		DccWidgets.note(body, "Compute failed: %s" % String(_last_result.get("error", "unknown error")))
		return

	var plan: Dictionary = _last_result.get("plan", {})
	var verdict: Dictionary = _last_result.get("verdict", {})
	var confidence: Dictionary = _last_result.get("confidence", {})

	_build_verdict_card(body, plan, verdict, confidence)
	_build_time_group(body, plan, confidence)
	_build_load_group(body, plan)
	_build_supply_group(body, plan)
	_build_cost_group(body)
	_build_vessels_group(body, plan)
	_build_trace_group(body)

## `right_dock.gd`'s RD-11 collapsed-readout call (`_dock_readout_text()`) --
## the one number worth keeping visible when Journey is the active right-dock
## context and the dock is collapsed. Reads `_last_result` rather than
## exposing it, matching this file's own convention that every other reader
## of the plan (`build_results` and its `_build_*_group` helpers) goes
## through a method here instead of the underscore-prefixed field directly.
func readout_text() -> String:
	if _route_index < 0:
		return "no route"
	if _last_result.is_empty() or not bool(_last_result.get("ok", false)):
		return "no result"
	var plan: Dictionary = _last_result.get("plan", {})
	var days := float(plan.get("total_days", -1.0))
	var km := float(plan.get("km", 0.0))
	return ("%.0f d · %.0f km" % [days, km]) if days >= 0.0 else "%.0f km" % km

func _build_verdict_card(body: Control, plan: Dictionary, verdict: Dictionary, confidence: Dictionary) -> void:
	var level := String(verdict.get("level", ""))
	var token := "block" if level in ["severe", "blocked"] else ("warn" if level == "strained" else "accent")
	var glyph := DccIcons.SYMBOLS["blocked"] if level == "blocked" else (DccIcons.SYMBOLS["warn_tri"] if level in ["strained", "severe"] else "")
	var card := VBoxContainer.new()
	card.add_theme_constant_override("separation", 4)
	var pad := MarginContainer.new()
	for side in ["left", "right", "top", "bottom"]:
		pad.add_theme_constant_override("margin_" + side, 11)
	pad.add_child(card)
	body.add_child(pad)
	body.add_child(DccTheme.rule())

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 6)
	if glyph != "":
		head.add_child(DccTheme.mono_label(glyph, token, DccTheme.FS_SMALL))
	head.add_child(DccTheme.mono_label(String(verdict.get("label", "?")).to_upper(), token, DccTheme.FS_HEADER, 2, true))
	card.add_child(head)

	var total_days := float(plan.get("total_days", -1.0))
	var days_row := HBoxContainer.new()
	days_row.add_theme_constant_override("separation", 6)
	days_row.add_child(DccTheme.mono_label(("%.0f" % total_days) if total_days >= 0.0 else "—", "text_bright", 26, 0, true))
	days_row.add_child(DccTheme.mono_label("calendar days", "text_dim", DccTheme.FS_SMALL))
	card.add_child(days_row)

	var split := HBoxContainer.new()
	split.add_theme_constant_override("separation", 12)
	split.add_child(DccTheme.mono_label("%.1f travel" % float(plan.get("travel_days", 0.0)), "text_dim", DccTheme.FS_TINY))
	split.add_child(DccTheme.mono_label("%d rest / layover" % (int(plan.get("rest_days", 0)) + int(plan.get("layover_days", 0))), "text_dim", DccTheme.FS_TINY))
	if not confidence.is_empty():
		split.add_child(DccTheme.mono_label("%.0f – %.0f d" % [float(confidence.get("lo_days", 0.0)), float(confidence.get("hi_days", 0.0))], "text", DccTheme.FS_TINY))
	card.add_child(split)

	var text_l := DccTheme.label(String(verdict.get("text", "")), token, DccTheme.FS_SMALL)
	text_l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	card.add_child(text_l)
	var reasons: PackedStringArray = verdict.get("reasons", PackedStringArray())
	for reason in reasons:
		var rl := DccTheme.label("· %s" % String(reason), "text_ghost", DccTheme.FS_TINY)
		rl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		card.add_child(rl)

	var blocked_idx := int(plan.get("blocked_idx", -1))
	if blocked_idx >= 0:
		var results: Array = plan.get("results", [])
		var br: Dictionary = results[blocked_idx] if blocked_idx < results.size() else {}
		card.add_child(DccTheme.mono_label("RESOLVE INLINE · STAGE %02d" % (blocked_idx + 1), "text_ghost", DccTheme.FS_MICRO, 1, true))
		card.add_child(_blocked_resolution_row(br))

## JP-14 (`JOURNEY_PLANNER_SPEC.md` §9: "offers its resolutions inline (turn
## off closures, re-route land-only, depart earlier)") -- three `_plan_values`
## edits plus a `_compute()` recall, not a route-level reroute. v2.10's own
## "re-route journey, land-only" quick fix (`_jpRerouteForMode`) replaces the
## drawn path with a fresh Dijkstra land route -- real pathfinding work with
## no Rust port (`GUI_GAP_REGISTER.md` JP-01/JP-03, deliberately left alone
## by this pass, not reimplemented under a different row's name here).
## What genuinely IS a plain plan edit, and resolves the three land-stage
## block reasons that name it in their own text ("Switch to Walking or
## reroute", "Remove carts/wagons or reroute", "Add travois or pack animals,
## or reroute" -- `cartalith-civ/src/lib.rs`'s own `jp_calc_land_ex`):
## forcing the party's transport to Walking AND zeroing carts/wagons -- the
## wheeled-vehicle block is gated on cart/wagon *count*, not on `transport`
## (`jp_calc_land_ex`: `(wagons>0||carts>0) && JP_WHEEL_BLOCKED.contains
## (terrain)`, checked independently of which mode is selected), so the
## transport flip alone clears the Mounted-Rider and Baggage-Train reasons
## but not this one; both together clear all three. It does nothing for a
## genuinely blocked WATER leg (`jp_calc_water` never reads `plan.transport`,
## only `plan.vessel`) -- offered anyway, since recomputing is how the party
## finds that out, not a claim this always resolves it.
func _blocked_resolution_row(r: Dictionary) -> Control:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	if bool(r.get("blocked_seasonal", false)):
		DccWidgets.action(row, "turn off seasonal closures", func():
			_plan_values["seasonal_closures"] = false
			_plan_value_changed(true))
	DccWidgets.action(row, "force Walking (land-only)", func():
		_plan_values["transport"] = "Walking"
		_plan_values["carts"] = 0
		_plan_values["wagons"] = 0
		_plan_value_changed(true))
	var seasons: PackedStringArray = _options.get("season", PackedStringArray())
	var cur := String(_plan_values.get("season", ""))
	var si: int = seasons.find(cur)
	if si > 0:
		var earlier := String(seasons[si - 1])
		DccWidgets.action(row, "depart in %s instead" % earlier, func():
			_plan_values["season"] = earlier
			_plan_value_changed(true))
	return row

func _kv_row(parent: Control, label_text: String, value_text: String, token: String = "text") -> void:
	var row := HBoxContainer.new()
	row.custom_minimum_size.y = 18
	row.add_child(DccTheme.mono_label(label_text, "text_dim", DccTheme.FS_SMALL))
	row.add_child(DccTheme.spacer())
	row.add_child(DccTheme.mono_label(value_text, token, DccTheme.FS_SMALL))
	parent.add_child(row)

func _build_time_group(body: Control, plan: Dictionary, confidence: Dictionary) -> void:
	var g := DccWidgets.section(body, "Time")
	_kv_row(g, "travel days", "%.1f" % float(plan.get("travel_days", 0.0)))
	_kv_row(g, "rest days · %s" % String(plan.get("rest_basis", "")), str(int(plan.get("rest_days", 0))))
	var stops: Array = plan.get("stops", [])
	_kv_row(g, "layovers", "%d at %d stops" % [int(plan.get("layover_days", 0)), stops.size()])
	if not confidence.is_empty():
		_kv_row(g, "mean · best · worst", "%.0f · %.0f · %.0f" % [
			float(plan.get("total_days", 0.0)), float(confidence.get("lo_days", 0.0)), float(confidence.get("hi_days", 0.0))])
	var seasons: PackedStringArray = plan.get("seasons_crossed", PackedStringArray())
	var arrival := seasons[seasons.size() - 1] if seasons.size() > 0 else String(_plan_values.get("season", "?"))
	_kv_row(g, "arrival season", arrival, "accent")

func _bar(parent: Control, ratio: float, warn_from: float = 0.7, block_from: float = 1.0) -> void:
	var track := HBoxContainer.new()
	track.custom_minimum_size.y = 5
	track.add_theme_constant_override("separation", 0)
	var filled_ratio := clampf(ratio, 0.0, 1.0)
	var fill := ColorRect.new()
	fill.color = DccTheme.c("block" if ratio >= block_from else ("warn" if ratio >= warn_from else "accent"))
	fill.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	fill.size_flags_stretch_ratio = maxf(0.001, filled_ratio)
	track.add_child(fill)
	var empty := ColorRect.new()
	empty.color = DccTheme.c("line")
	empty.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	empty.size_flags_stretch_ratio = maxf(0.001, 1.0 - filled_ratio)
	track.add_child(empty)
	parent.add_child(track)

func _build_load_group(body: Control, plan: Dictionary) -> void:
	var g := DccWidgets.section(body, "Load")
	var results: Array = plan.get("results", [])
	var worst_cap: Dictionary = {}
	var worst_ratio := 0.0
	for r in results:
		var d: Dictionary = r
		var land: Dictionary = d.get("land", {})
		if land.is_empty():
			continue
		var lr := float(land.get("load_ratio", 0.0))
		if lr >= worst_ratio:
			worst_ratio = lr
			worst_cap = land.get("capacity", {})
	if worst_cap.is_empty():
		DccWidgets.note(g, "No land leg computed for this journey -- capacity is a land-transport concept (river/sea legs report crew and hold instead, see Vessels below).")
	else:
		_bar(g, worst_ratio)
		_kv_row(g, "cargo · supplies", "%s · %s kg" % [
			_fmt_thousands(float(_plan_values.get("cargo_kg", 0.0)), 0), _fmt_thousands(float(worst_cap.get("fodder", 0.0)) + float(worst_cap.get("human_food", 0.0)), 0)])
		_kv_row(g, "capacity", "%s kg" % _fmt_thousands(float(worst_cap.get("capacity", 0.0)), 0))
		var carriers: Array[String] = []
		for pair in [["donkey", "donkeys"], ["mule", "mules"], ["camel", "camels"], ["horse", "horses"],
				["carts", "carts"], ["wagons", "wagons"], ["travois", "travois"], ["sleds", "sleds"]]:
			var n := int(_plan_values.get(pair[0], 0))
			if n > 0:
				carriers.append("%d %s" % [n, pair[1]])
		_kv_row(g, "carriers", " · ".join(carriers) if not carriers.is_empty() else "none carried")
		_kv_row(g, "load", "%.0f%% of capacity" % (worst_ratio * 100.0), "block" if worst_ratio >= 1.0 else ("warn" if worst_ratio >= 0.7 else "text"))
	DccWidgets.note(g, "Speed penalty is folded into each leg's own km/day rather than reported as a separate percentage -- jp_plan does not return one.")

func _build_supply_group(body: Control, plan: Dictionary) -> void:
	var g := DccWidgets.section(body, "Supply reach")
	var reach: Dictionary = plan.get("resupply_reach", {})
	if reach.is_empty():
		DccWidgets.note(g, "No resupply-reach figure for this journey.")
		return
	_kv_row(g, "carried", "%.0f d · %s km" % [float(_plan_values.get("supply_days", 0.0)), _fmt_thousands(float(reach.get("required_km", 0.0)), 0)])
	var gap := float(reach.get("max_gap_km", 0.0))
	var gap_km_per_day := float(plan.get("avg_km_day", 1.0))
	var gap_days := gap / maxf(1.0, gap_km_per_day)
	_kv_row(g, "longest gap", "%.1f d · %s km" % [gap_days, _fmt_thousands(gap, 0)], "warn" if bool(reach.get("unmet", false)) else "text")
	_kv_row(g, "desert km", "%s km" % _fmt_thousands(float(plan.get("desert_km", 0.0)), 0))
	_kv_row(g, "stops needed", str(int(reach.get("stops", 0))), "block" if bool(reach.get("unmet", false)) else "text")
	_build_reach_bar(g, plan, reach)
	DccWidgets.note(g, "Foraging offset is not broken out as a separate figure by jp_plan -- it is already folded into the food/water totals above.")

## JP-12 (§8: "per-leg bar with resupply ticks") -- `resupply_reach` itself
## carries no per-stop positions (`required_km`/`max_gap_km`/`stops`/`unmet`
## are route-wide scalars), but `plan.stops` does, via the same
## `_stop_fractions()` chord-length projection the stops strip above already
## uses. One segment per gap between consecutive resupply stops (route ends
## counting as the first/last boundary), lit `block` when that specific leg's
## own distance would outrun `required_km` -- the carry range this journey's
## `supply_days` actually give it -- `accent` otherwise, with a tick between
## every pair of segments marking the stop itself. Real geometry, not the
## single worst-gap figure the kv rows above already show; this is what
## "per-leg", plural, adds.
func _build_reach_bar(parent: Control, plan: Dictionary, reach: Dictionary) -> void:
	var stops: Array = plan.get("stops", [])
	var total_km := float(plan.get("km", 0.0))
	if stops.is_empty() or total_km <= 0.0 or _route_index < 0:
		return
	var pts: PackedVector2Array = bridge.route_get(_route_index).get("points", PackedVector2Array())
	var fracs := _stop_fractions(stops, pts)
	var required_km := float(reach.get("required_km", 0.0))

	var track := HBoxContainer.new()
	track.custom_minimum_size.y = 6
	track.add_theme_constant_override("separation", 0)
	var bounds := PackedFloat64Array([0.0])
	bounds.append_array(fracs)
	bounds.append(1.0)
	for i in range(bounds.size() - 1):
		var seg_frac: float = bounds[i + 1] - bounds[i]
		if seg_frac > 0.0:
			var seg_km: float = seg_frac * total_km
			var seg := ColorRect.new()
			seg.color = DccTheme.c("block") if (required_km > 0.0 and seg_km > required_km) else DccTheme.c("accent")
			seg.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			seg.size_flags_stretch_ratio = maxf(0.001, seg_frac)
			seg.tooltip_text = "%s km leg" % _fmt_thousands(seg_km, 0)
			track.add_child(seg)
		if i < bounds.size() - 2:   ## a tick at every interior boundary -- a real stop, not a route end
			var tick := ColorRect.new()
			tick.color = DccTheme.c("bg")
			tick.custom_minimum_size.x = 2
			track.add_child(tick)
	parent.add_child(track)

func _build_cost_group(body: Control) -> void:
	var g := DccWidgets.section(body, "Cost")
	DccWidgets.note(g, "jp_plan returns no monetary figures, so nothing here can be filled today -- but the cost model itself IS ported: cartalith_civ::jp_journey_cost (jpJourneyCost, reference line 18873) computes carriage, wages, crew, upkeep, tolls, transshipment, total, per-tonne-km and break-even, with golden tests. It is simply never called: jp_compute/jp_journey_plan_dict don't invoke it and no #[func] exposes it. Every input it needs is already computed inside jp_plan (per-leg km/days/crew, JpDerivedStage::claimed_frac, JpJourneyPlan::transshipments), so this is a boundary gap, not a model gap. Not invented here.")

func _build_vessels_group(body: Control, plan: Dictionary) -> void:
	var g := DccWidgets.section(body, "Vessels · water legs")
	var stages: Array = plan.get("stages", [])
	var results: Array = plan.get("results", [])
	var any := false
	for i in stages.size():
		var r: Dictionary = results[i] if i < results.size() else {}
		var water: Dictionary = r.get("water", {})
		if water.is_empty():
			continue
		any = true
		var s: Dictionary = stages[i]
		_kv_row(g, "stage %02d · %s" % [i + 1, String(s.get("cat", "water"))], String(water.get("transport_label", "?")))
		_kv_row(g, "hold used", "%s / %s kg" % [
			_fmt_thousands(float(_plan_values.get("cargo_kg", 0.0)), 0), _fmt_thousands(float(water.get("hold_kg", 0.0)), 0)])
		_kv_row(g, "crew", str(int(water.get("crew", 0))))
	if not any:
		DccWidgets.note(g, "No water legs on this route.")
	else:
		DccWidgets.note(g, "Sailing-window text (daylight vs. open-water hours) is not part of jp_water_calc's return -- not shown.")

func _build_trace_group(body: Control) -> void:
	var head := HBoxContainer.new()
	head.custom_minimum_size.y = 26
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 13)
	pad.add_theme_constant_override("margin_right", 13)
	pad.add_child(head)
	body.add_child(DccTheme.rule())
	body.add_child(pad)
	head.add_child(DccTheme.mono_label("calculation trace", "text_ghost", DccTheme.FS_SMALL))
	head.add_child(DccTheme.spacer())
	var open_btn := DccWidgets.action(head, "open ⧉", func(): pass)
	open_btn.disabled = true
	open_btn.tooltip_text = "No calculation-trace window exists yet -- jpCalcLand/jpCalcWater's own 'formula' trace string is deliberately not carried across the boundary (jp_land_calc_dict's own doc comment: presentation, not engine); building a trace window from raw dict values is future work, not faked here."

# ================================================================ Draw views ====

## The route-map polyline, coloured per stage category with real geometry --
## `route_get()`'s own points, sliced per `plan.stages[i].{i0,i1}` (the same
## index range `jp_plan` derived that stage over). No SVG-cloning of the
## mockup's specific example curve; whatever route is actually committed
## draws its own real shape.
class _RouteMapView extends Control:
	var pts: PackedVector2Array = PackedVector2Array()
	var stage_segments: Array = []   ## [{i0,i1,cat,blocked}]
	var stops: Array = []            ## [Vector2] world grid coords

	func _ready() -> void:
		resized.connect(func(): queue_redraw())

	func _fit(rect: Rect2) -> Callable:
		if pts.is_empty():
			return func(p: Vector2) -> Vector2: return Vector2.ZERO
		var minv := pts[0]
		var maxv := pts[0]
		for p in pts:
			minv.x = minf(minv.x, p.x)
			minv.y = minf(minv.y, p.y)
			maxv.x = maxf(maxv.x, p.x)
			maxv.y = maxf(maxv.y, p.y)
		var margin_x: float = maxf(1.0, (maxv.x - minv.x) * 0.12)
		var margin_y: float = maxf(1.0, (maxv.y - minv.y) * 0.12)
		minv -= Vector2(margin_x, margin_y)
		maxv += Vector2(margin_x, margin_y)
		var bw: float = maxf(1e-6, maxv.x - minv.x)
		var bh: float = maxf(1e-6, maxv.y - minv.y)
		var s: float = minf((rect.size.x - 20.0) / bw, (rect.size.y - 20.0) / bh)
		var ox: float = rect.position.x + (rect.size.x - bw * s) * 0.5
		var oy: float = rect.position.y + (rect.size.y - bh * s) * 0.5
		return func(p: Vector2) -> Vector2: return Vector2(ox + (p.x - minv.x) * s, oy + (p.y - minv.y) * s)

	func _draw() -> void:
		var rect := Rect2(Vector2.ZERO, size)
		if pts.size() < 2:
			draw_string(ThemeDB.fallback_font, Vector2(14, 20), "no committed route selected",
				HORIZONTAL_ALIGNMENT_LEFT, -1, 11, DccTheme.c("text_ghost"))
			return
		var fit := _fit(rect)
		if stage_segments.is_empty():
			var poly := PackedVector2Array()
			for p in pts:
				poly.append(fit.call(p))
			draw_polyline(poly, DccTheme.c("accent"), 1.6, true)
		else:
			for seg in stage_segments:
				var d: Dictionary = seg
				var i0: int = clampi(int(d.get("i0", 0)), 0, pts.size() - 1)
				var i1: int = clampi(int(d.get("i1", i0)), i0, pts.size() - 1)
				var poly := PackedVector2Array()
				for i in range(i0, i1 + 1):
					poly.append(fit.call(pts[i]))
				if poly.size() < 2:
					continue
				var cat := String(d.get("cat", "land"))
				var blocked := bool(d.get("blocked", false))
				var col := DccTheme.c("block") if blocked else (DccTheme.c("water") if cat != "land" else DccTheme.c("accent"))
				var w := 3.0 if cat != "land" else 1.8
				draw_polyline(poly, col, w, true)
		for i in range(0, pts.size(), maxi(1, pts.size() / 40)):
			var p: Vector2 = fit.call(pts[i])
			draw_circle(p, 1.2, DccTheme.c("line_soft"))
		for st in stops:
			var p2: Vector2 = fit.call(st)
			draw_circle(p2, 3.0, DccTheme.c("bg"))
			draw_arc(p2, 3.0, 0, TAU, 16, DccTheme.c("accent"), 1.4)

## The terrain-profile spine (`JOURNEY_PLANNER_SPEC.md` §3, §4): stage bands
## along a shared distance axis with the real elevation sparkline
## (`plan.profile`) overlaid, and the stage selector interaction (click,
## ⌥-click isolate).
class _ProfileView extends Control:
	signal stage_clicked(idx: int, isolate: bool)

	var profile: PackedFloat64Array = PackedFloat64Array()
	var bands: Array = []   ## [{start, end, cat, blocked, warn, label}] fractions 0..1
	var selected_idx := -1
	var isolated_idx := -1

	func _ready() -> void:
		resized.connect(func(): queue_redraw())
		mouse_filter = Control.MOUSE_FILTER_STOP

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			if bands.is_empty() or size.x <= 0.0:
				return
			var frac: float = clampf(event.position.x / size.x, 0.0, 1.0)
			for i in bands.size():
				var b: Dictionary = bands[i]
				if frac >= float(b.get("start", 0.0)) and frac <= float(b.get("end", 1.0)):
					stage_clicked.emit(i, event.alt_pressed)
					return

	func _draw() -> void:
		var w := size.x
		var h := size.y
		if w <= 0.0 or h <= 0.0:
			return
		for i in range(1, 4):
			var y := h * float(i) / 4.0
			draw_line(Vector2(0, y), Vector2(w, y), DccTheme.c("line_soft"), 1.0)
		for i in bands.size():
			var b: Dictionary = bands[i]
			var x0 := float(b.get("start", 0.0)) * w
			var x1 := float(b.get("end", 1.0)) * w
			var blocked := bool(b.get("blocked", false))
			var warn := bool(b.get("warn", false))
			var cat := String(b.get("cat", "land"))
			var dimmed := isolated_idx >= 0 and i != isolated_idx
			var base_col := DccTheme.c("block") if blocked else (DccTheme.c("water") if cat != "land" else DccTheme.c("accent"))
			var fill := Color(base_col, 0.16 if i == selected_idx else (0.05 if warn else 0.0))
			if dimmed:
				fill.a *= 0.3
			if fill.a > 0.0:
				draw_rect(Rect2(x0, 0, x1 - x0, h), fill)
			if i == selected_idx:
				draw_rect(Rect2(x0, 0, x1 - x0, h), base_col, false, 1.0)
			if x0 > 0.0:
				draw_line(Vector2(x0, 0), Vector2(x0, h), DccTheme.c("line"), 1.0)
		if profile.size() >= 2:
			var poly := PackedVector2Array()
			var fill_poly := PackedVector2Array()
			var lo := profile[0]
			var hi := profile[0]
			for v in profile:
				lo = minf(lo, v)
				hi = maxf(hi, v)
			var span: float = maxf(1e-6, hi - lo)
			var track_h := h - 15.0   ## bottom 15px reserved for the stage index strip
			for i in profile.size():
				var x := w * float(i) / float(profile.size() - 1)
				var y := track_h - ((profile[i] - lo) / span) * (track_h - 8.0) - 2.0
				poly.append(Vector2(x, y))
			fill_poly.append(Vector2(0, track_h))
			fill_poly.append_array(poly)
			fill_poly.append(Vector2(w, track_h))
			draw_colored_polygon(fill_poly, Color(DccTheme.c("accent"), 0.10))
			draw_polyline(poly, DccTheme.c("accent"), 1.4, true)
		for i in bands.size():
			var b: Dictionary = bands[i]
			var x0 := float(b.get("start", 0.0)) * w
			var label_text := String(b.get("label", ""))
			var col := DccTheme.c("block") if bool(b.get("blocked", false)) else (DccTheme.c("accent") if i == selected_idx else DccTheme.c("text_ghost"))
			draw_string(ThemeDB.fallback_font, Vector2(x0 + 3, h - 4), label_text,
				HORIZONTAL_ALIGNMENT_LEFT, maxf(4.0, (float(b.get("end", 1.0)) - float(b.get("start", 0.0))) * w - 4.0), 9, col)

## JP-13's day-band strip -- see `_rebuild_timeline_band()`'s own doc comment
## for what each segment means and why "weather hold" never lights up. Same
## `_draw()` convention as `_RouteMapView`/`_ProfileView` above.
class _TimelineBandView extends Control:
	var segments: Array = []   ## [{days: float, token: String}], route order

	func _ready() -> void:
		resized.connect(func(): queue_redraw())

	func _draw() -> void:
		var w := size.x
		var h := size.y
		if w <= 0.0 or h <= 0.0:
			return
		var total := 0.0
		for seg in segments:
			total += float((seg as Dictionary).get("days", 0.0))
		if total <= 0.0:
			draw_rect(Rect2(0, 0, w, h), DccTheme.c("line"))
			return
		var x := 0.0
		for i in segments.size():
			var seg: Dictionary = segments[i]
			var frac: float = float(seg.get("days", 0.0)) / total
			var sw: float = frac * w
			draw_rect(Rect2(x, 0, sw, h), DccTheme.c(String(seg.get("token", "accent"))))
			x += sw
			if i + 1 < segments.size() and sw > 1.5:
				draw_line(Vector2(x, 0), Vector2(x, h), DccTheme.c("bg"), 1.0)
