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
## - **Journeys list**: real as of 2026-08-23 (JP-06/JP-08) — "save journey"
##   names the selected route *plus* the whole party form and adds it to the
##   list, which reloads it in one click. **Persistent as of 2026-08-26**: the
##   list is written into the project archive as `entities/journeys.json` (the
##   §9.6 slot `SAVEFILE_COMPAT.md` reserved) and restored on open — see the
##   "Journeys, on disk (F10)" section at the foot of this file. It stays in
##   GDScript rather than in `cartalith-civ` because a saved journey is exactly
##   the request `jp_compute` already takes, so the engine would own nothing
##   the shell does not already hold; the archive channel is
##   `project_save_with_documents`, which takes caller-owned slots as text.
## - **Carriage auto/manual**: real as of 2026-08-23 (JP-01).
##   `jpAutoPickTransport` was already ported (`cartalith_civ::
##   jp_auto_pick_transport`); what was missing was the call. Auto now sends
##   `jp_compute`'s own `auto_carriage` key, the picker mutates the plan
##   before it is computed (the reference's `_jpRunAuto`, line 19614, one
##   call site per refresh), and `_sync_auto_carriage()` writes the picked
##   counts back into the form — its `_jpSyncAssetInputs` (line 19632),
##   including the `promoted` -> structural-rebuild rule.
## - **Party set-ups**: real as of 2026-08-20 (JP-02). Not the reference's
##   JS-only `JP_PRESETS` (~line 17595) — the Travel Library's own stored
##   rows, stock and captured alike (`tl_list`/`tl_get("preset")`,
##   `tl_capture_preset_from_plan`), which is the strictly larger thing.
##   Applying one fills the party form; "capture party…" writes the current
##   form back as a new row (`TRAVEL_LIBRARY_SPEC.md` §3.4).
## - **Travel Library definitions in the party form**: real as of
##   2026-08-20 (TL-01) for animals — see the "Travel Library wiring"
##   section below for exactly what is offerable and what is not, and why.
## - **Re-route for <mode>…**: real as of 2026-08-23 (JP-03).
##   `_jpRerouteForMode` (reference line 20391) ported into `cartalith-civ`
##   and bound as `jp_reroute`; it re-paths the committed route's two
##   endpoints under the domain the journey's transport implies and rewrites
##   the route in place. An unreachable answer is refused with the
##   reference's own message rather than drawn as the straight-line fallback
##   `route_commit` tolerates.
## - **Cost group**: real as of 2026-08-23 (JP-04). The model
##   (`jp_journey_cost`) had been ported and golden-tested since milestone 3
##   and was called by nothing; `jp_plan_cost` is the reference's own call
##   site (line 19854) and `jp_compute` now returns its `cost` dict. Priced
##   in **day-wages**, never a currency — that is the model's own unit, not
##   a simplification here.
## - **Calculation trace**: real as of 2026-08-23 (JP-05), built as
##   `GUI_GAP_REGISTER.md` §7.12 proposed — an inline group over the
##   selected stage rather than the spec's `⧉` window, one row per
##   multiplicative term with its running value. The reference's `formula`
##   *string* still does not cross the boundary; what crosses is the
##   structured `trace` (`JpTerm`), whose product is asserted engine-side to
##   equal the leg's own `daily_km`.
## - **Elevation-profile sparkline**: unlike the old dialog (which reported
##   `plan.profile`'s presence and stopped), this pass DOES draw it --
##   `_ProfileView` plots the real 0-1 normalised samples. It was only
##   time-boxed out before; rebuilding this view was the right time to close
##   it for real.
## - **⇧-drag spine trim**: real as of 2026-08-23 (JP-07). `jp_compute` gained
##   a `trim` request key (two fractions of the route's arc length) which
##   cuts the polyline through `cartalith_civ::jp_trim_points` *before*
##   anything reads it — so every stage index, stop key and per-stage
##   override that comes back belongs to the trimmed route, and a trim is
##   indistinguishable from having drawn the shorter route by hand. ⇧ click
##   without dragging clears it. Click-to-select and ⌥-click-to-isolate are
##   unchanged.
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
## - **Wildlife forage modifier**: real as of 2026-08-26 (F12).
##   `jp_compute` used to pass `&|_, _| 1.0` as the forage modifier over a
##   comment calling that *"the reference's own answer on a world whose
##   wildlife layer was never built"* -- a description of the port as of
##   Journey Planner milestone 4, stale ever since `cartalith_civ::wildlife`
##   landed (`PARITY_AUDIT.md` §23 F12). Foraging now reads this world's own
##   ecoregion species richness at each stage midpoint, exactly as the
##   reference's `_jpWildlifeForageMod(mx,my)` does, bounded to [0.5, 1.8]
##   against the world's own mean. **It only moves anything when Foraging is
##   not "None"** -- `jp_foraging` returns before it reads the modifier in
##   that mode, which is the default. The cost that made this a cache
##   problem rather than a one-liner, and the fingerprint that keeps it from
##   going stale, are documented on `sample_bridge::WildlifeCache`.
## - **Fodder ceiling under "Supplies carried"**: real as of 2026-08-26
##   (JP-16, `PARITY_AUDIT.md` §23 F13). This line used to restate the
##   current supply setting from the last compute's own `capacity.fodder`;
##   it now states the **ceiling**, from `jp_pack_range`, before the user
##   configures past it -- which is the whole content of the reference's
##   v1.49 fix. See `_refresh_pack_range_note()` for what it replaced.
## - **Campaign-duration advisory**: real as of 2026-08-26 (JP-17).
##   `jp_risk` rides on `jp_compute`'s `plan.risk`, the reference's own field
##   on `_jpPlan`'s return, and is drawn where the reference draws it --
##   after the cost group, before the stage table.
## - **Vessel matrix**: real as of 2026-08-26 (JP-17). `jp_vessel_matrix`
##   in both of the reference's own two views (route-scored and general
##   reference). One disclosed divergence, in `jp_vessel_matrix`'s own
##   `#[func]` doc comment: the water **column order** is `(cat, terrain)`
##   alphabetical rather than the reference's physical order, because that
##   order lives in two private `const`s inside `cartalith-civ` and copying
##   them here would be a second table.
## - **`jp_auto_stage_picks` still runs at a flat forage modifier of 1.0**,
##   disclosed at its call site in `lib.rs`: it takes one scalar for the
##   whole journey rather than the per-stage closure `jp_plan_full` takes,
##   so there is no honest per-stage value to give it. It only *ranks*
##   candidate per-stage packages; whatever it picks is recomputed under the
##   real per-stage modifier before anything is reported.

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
var _carriage_auto := true              ## Sent as `jp_compute`'s own `auto_carriage` (JP-01).
## Sent as `jp_compute`'s `auto_stage` (`DECISIONS.md` §7j). Off by default:
## it rewrites per-stage overrides, and a planner that silently re-tacks the
## train the first time a route is opened would be doing it behind the user.
var _stage_auto := false

## JP-07. The ⇧-drag spine trim, as two fractions of the route's arc length.
## `Vector2(0, 1)` is the whole route and is not sent at all, so an untrimmed
## journey's request is byte-identical to what it was before this existed.
var _trim := Vector2(0.0, 1.0)

## JP-06 / JP-08. The journeys list: a route index plus the whole party form,
## named. **Persisted** into the project's `entities/journeys.json` slot by
## `journeys_document()` and read back by `restore_journeys_document()` — both
## at the foot of this file. Kept in GDScript rather than pushed into
## `cartalith-civ` because a saved journey is exactly the request `jp_compute`
## already takes, so the engine would own nothing the shell does not; the
## archive channel is `project_save_with_documents`, which carries
## caller-owned slots as text. `route` is an index into the routes saved
## beside it, which is why `setup()` clears this list on a world change.
## Entries: `{name: String, route: int, plan: Dictionary, stage_overrides:
## Dictionary, layovers: Dictionary, animal_entries: Dictionary, trim: Vector2}`.
var _journeys: Array = []
var _active_journey := -1

## The `EngineBridge.last_documents` dictionary the last restore read -- held
## for `is_same()` identity only, and never indexed. `world_loaded` re-announces
## the *same* documents on every in-place field op, and re-restoring them threw
## away every journey planned since the file was opened; see
## `restore_journeys_document()` for the whole reasoning.
var _restored_documents: Dictionary = {}

## TL-01: which Travel Library animal definition occupies each of the four
## built-in party-form species slots -- species key (String) -> entry id
## (String). Sent as `jp_compute`'s own `animal_entries` request key, NOT as
## a `plan` field: it is a library reference, not one of `JpPlan`'s values,
## and `plan_from_pairs` would rightly reject it. Defaults to each species'
## own stock entry (which the engine reads as "no override"), so an untouched
## form computes exactly what it did before this existed.
var _animal_entries: Dictionary = {}
var _library_animals: Array = []   ## last `tl_list("animal")`, refreshed per form rebuild
var _library_vessels: Array = []   ## last `tl_list("vessel")`

var _active := false

# -- Region roots ---------------------------------------------------------------

var _left_panel: VBoxContainer
var _left_route_section: VBoxContainer
var _left_party_body: VBoxContainer
var _auto_obs: Dictionary = {}   ## JP-15: field_key (String) -> OptionButton, the party form's own "Auto" fields -- refreshed post-compute by `_refresh_auto_labels()` rather than rebuilt, so a live numeric edit elsewhere in the form never loses focus.
## JP-16: the fodder-ceiling advisory under "Supplies carried", refreshed in
## place by `_refresh_pack_range_note()` for exactly the reason `_auto_obs`
## above is -- it changes on every party-form keystroke and rebuilding the
## form to update one line would drop focus out of the SpinBox being typed in.
var _pack_range_label: Label
## JP-17: `jp_vessel_matrix()`'s output, fetched once. A **static table** --
## eleven hulls against nine water types, no world state anywhere in it -- so
## re-fetching it per compute would be re-reading a constant.
var _vessel_matrix: Dictionary = {}

var _center_panel: Control
var _route_map_wrap: Control
var _route_map: _RouteMapView
var _route_line: _RouteLineLayer
var _route_map_layer_btn: Button
var _route_map_layer_popup: PopupMenu
## Which `EngineBridge.debug_texture()` id the route-map backdrop shows --
## `"off"` (the pre-existing plain background) by default, so this feature is
## purely additive until picked.
var _route_map_layer_id := "map"
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

	## **The journeys list has a world lifecycle, and until now had none.**
	##
	## A journey is a *route index* plus a party form (see `_journeys`), and a
	## route index only means anything against the world that produced it. A
	## list carried across a world change indexes routes that no longer exist,
	## and `journeys_document()` then writes those dangling indices into the
	## NEXT project's archive on the next save. Two connections close that:
	##
	## - `generation_finished`: a generate replaces `WorldGen.infra` wholesale,
	##   so every committed route is gone and `route_count()` answers 0.
	##   Heightmap import arrives here too (`EngineBridge.import_heightmap()`
	##   emits it), which is right -- an imported heightmap is a new world.
	## - `world_loaded`, but **only when no world remains**. That signal fires
	##   for seven different reasons (a project open, an import, `close_world`,
	##   an asset pack, `center_landmasses`, `carve_fjords`, `as_apply_to_map`)
	##   and only `close_world()` leaves `has_world == false`. The four in-place
	##   ops keep the same routes and must not touch the list; a project *open*
	##   is `restore_journeys_document()`'s job, which `app.gd` calls from
	##   `_restore_project_documents()` (itself called only from `_load_project`,
	##   the one place a new set of documents actually arrives).
	##
	## A close is therefore the one case both ends could touch: `close_world()`
	## emits `world_loaded` and leaves `last_documents` holding the previous
	## archive's text. The clear below handles it, and
	## `restore_journeys_document()`'s own identity guard means a later stray
	## call with those same bytes cannot undo it.
	bridge.generation_finished.connect(func(ok: bool): if ok: clear_journeys())
	bridge.world_loaded.connect(func(): if not bridge.has_world: clear_journeys())

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
	_phone_refit()

func _hide() -> void:
	_left_panel.visible = false
	_center_panel.visible = false
	app.viewport.visible = true
	var civ_panel: Control = app._workspace_panels.get("civilization")
	if civ_panel != null and app.active_domain() == "civilization":
		civ_panel.visible = true
	if app.right_dock_ctrl != null:
		app.right_dock_ctrl.clear_journey()
	## JP-13, **corrected**: hand `timeline_row` back to `app.gd` rather than
	## leaving it empty.
	##
	## The original premise -- "this view is the only thing that ever populates
	## `timeline_row`; CV-09 leaves it deliberately empty in CIVIL" -- stopped
	## being true when `app.gd` grew `_fill_timeline_strip()` and the desktop
	## timeline strip that lives in that row. Clearing the row to empty on disarm
	## therefore blanked the strip *permanently*: `_repaint_timeline()`'s own
	## rebuild fallback tests `_tl_year_labels.is_empty()`, and those labels were
	## still held (freed, but held), so the guard written for exactly this case
	## could never fire.
	##
	## `_fill_timeline_strip()` clears the row itself before rebuilding and resets
	## every held label/button reference in the same breath, so nothing of
	## Journey's band leaks into the domain switch either.
	##
	## `app.gd::arm_tool()` also refills after its `tool_armed` emit, gated on the
	## row being empty. That gate is now a no-op for this path rather than a
	## second fill — which is the intended relationship, not an oversight: the
	## borrower gives the row back here, and that gate stays as the backstop for
	## any disarm route that never reaches this function.
	_timeline_view = null
	if app.timeline_row != null:
		app._fill_timeline_strip()

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
			"No committed routes yet -- arm Route (⇧R) below, click waypoints, ✓ Commit, then this list re-reads it.")
		_route_index = -1
		return

	# JP-08's journeys list: the named, saved journeys first (a journey is a
	# route *plus* a party form), then the raw committed routes underneath.
	for i in _journeys.size():
		var j: Dictionary = _journeys[i]
		var jrow := HBoxContainer.new()
		jrow.add_theme_constant_override("separation", 4)
		var pad := MarginContainer.new()
		pad.add_theme_constant_override("margin_left", 13)
		pad.add_theme_constant_override("margin_right", 13)
		pad.add_child(jrow)
		_left_route_section.add_child(pad)
		var open_btn := Button.new()
		open_btn.text = "%s%s" % ["● " if i == _active_journey else "", String(j.get("name", "journey"))]
		open_btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
		open_btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		open_btn.tooltip_text = "Route #%d + this party form. Saved into the project's entities/journeys.json and restored on File ▸ Open project. The route is stored as an INDEX, so a journey only means what it meant against the routes saved beside it." % int(j.get("route", 0))
		open_btn.pressed.connect(func(): _load_journey(i))
		jrow.add_child(open_btn)
		var del_btn := Button.new()
		del_btn.text = "×"
		del_btn.tooltip_text = "Forget this journey."
		del_btn.pressed.connect(func(): _delete_journey(i))
		jrow.add_child(del_btn)
	if not _journeys.is_empty():
		_left_route_section.add_child(DccTheme.rule())

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
			_trim = Vector2(0.0, 1.0)
			_compute(),
		"route_get()'s own points/km/mode. \"save journey\" in the tool options bar names the route + party form together and adds it to the list above; that list travels in the project archive and comes back when the project is reopened.")

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
## first stage/leg carrying a real answer (a per-stage breakdown already
## exists, in the stage inspector's own `_inherit_label`). The fodder-ceiling
## note this once cited as the same convention no longer works that way --
## `_refresh_pack_range_note()` reads the party plan itself now, not a leg.
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
	_refresh_pack_range_note()

func _rebuild_party_form() -> void:
	if not _bound or _left_party_body == null:
		return
	for c in _left_party_body.get_children():
		_left_party_body.remove_child(c)
		c.queue_free()
	_auto_obs.clear()

	if _plan_values.is_empty():
		# `_jpEnsurePlan`, and only on a genuinely new plan -- which is the
		# reference's own `isNewPlan` gate, so re-entering the form never
		# overwrites a party the user has since edited. The route-aware
		# defaults (Sea Faring for a route the `mixed` cost grid took mostly
		# across open water, plus `jpAutoPickVessel`'s correction from the
		# route's real stages) when a route is selected; the route-blind
		# `jp_default_plan()` when none is, or on a binary without it.
		var seed_plan: Dictionary = _default_plan
		if _route_index >= 0:
			var rp: Dictionary = bridge.jp_plan_for_route(_route_index)
			if not rp.is_empty():
				seed_plan = rp
		for key in seed_plan.keys():
			if key != "party_fields" and key != "sea_journey":
				_plan_values[key] = seed_plan[key]

	# The Travel Library is edited in its own window (⇧L), which can be open
	# alongside this form, so the entry lists are re-read on every rebuild
	# rather than cached at bind time.
	_refresh_library()

	# -- Traveler --------------------------------------------------------------
	var traveler := DccWidgets.section(_left_party_body, "Party · Traveler")
	_number_field(traveler, "Group size", "group_size", 1.0, 100000.0, 1.0, true, "people")
	_choice_field(traveler, "Pace", "pace", _options.get("pace", PackedStringArray()), false)
	_number_field(traveler, "Hours/day (land)", "hours", 1.0, 16.0, 0.5, false)
	_number_field(traveler, "Trade cargo (kg)", "cargo_kg", 0.0, 500000.0, 10.0, false)
	_number_field(traveler, "Supplies carried (d)", "supply_days", 1.0, 90.0, 1.0, true)
	## JP-16. The wagon-equation ceiling, immediately under the control that
	## crosses it -- the reference's own placement (line 19657) and its own
	## reason: the threshold is knowable in advance and belongs beside the
	## control, not in a warning after the fact.
	_pack_range_label = DccWidgets.note(traveler, "")
	_refresh_pack_range_note()
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
	auto_btn.pressed.connect(func(): _carriage_auto = true; manual_btn.button_pressed = false; _rebuild_party_form(); _compute())
	manual_btn.pressed.connect(func(): _carriage_auto = false; auto_btn.button_pressed = false; _rebuild_party_form(); _compute())
	mode_row.add_child(auto_btn)
	mode_row.add_child(manual_btn)
	carriage.add_child(mode_row)
	if _carriage_auto:
		DccWidgets.note(carriage, _auto_carriage_note())

	# JP-10 / DECISIONS.md 7j: per-stage auto-pick. Separate from the
	# Auto/Manual pair above because it answers a different question -- that
	# one sizes ONE train for the whole route, this one re-tacks it stage by
	# stage where the ground rewards it.
	DccWidgets.toggle(carriage, "Re-pack per stage where it pays", _stage_auto,
		func(nv: bool):
			_stage_auto = nv
			_compute(),
		"jp_auto_stage_picks: measures each land stage's own best species, vehicle and land mode against that stage's terrain, and applies the swap when it beats the current setup by more than 10% -- or when it turns an impassable stage passable, where no percentage applies. Scales with group size and cargo, because every candidate is measured through the same jp_calc_land the stage itself uses. Never picks a mode the party lacks the animals for, and never overrules a per-stage field you set by hand.")
	if _stage_auto:
		DccWidgets.note(carriage, _stage_picks_note())

	_choice_field(carriage, "Transport", "transport", _options.get("transport", PackedStringArray()), false, true)
	var transport := String(_plan_values.get("transport", "Walking"))
	if transport == "Mounted Rider":
		_mount_field(carriage)
	if transport == "River Transport" or transport == "Sea Faring":
		_vessel_field(carriage)

	var animals := HBoxContainer.new()
	animals.add_theme_constant_override("separation", 4)
	_animal_pair(carriage, "Donkeys / Mules", "donkey", "mule")
	_animal_pair(carriage, "Camels / Horses", "camel", "horse")
	_animal_pair(carriage, "Carts / Wagons", "carts", "wagons")
	_animal_pair(carriage, "Travois / Sleds", "travois", "sleds")
	_toggle_field(carriage, "Auto-promote Walking → Baggage Train if overloaded", "auto_promote")

	_build_animal_definitions(carriage)

	# -- Route conditions --------------------------------------------------------
	var route_group := DccWidgets.section(_left_party_body, "Route conditions")
	_route_cond_field(route_group, "Road quality", "route_cond", String(_plan_values.get("route_cond", "")),
		func(v: String): _plan_values["route_cond"] = v; _plan_value_changed(false))
	_choice_field(route_group, "Infrastructure", "infra", _options.get("infra", PackedStringArray()), true)
	_choice_field(route_group, "Desert water", "desert_water", _options.get("desert_water", PackedStringArray()), true,
		false, "Auto measures the longest waterless run on this route and picks the matching tier.")
	_toggle_field(route_group, "Respect seasonal closures (winter passes)", "seasonal_closures", true)

	var footer := DccTheme.mono_label(
		"party set-ups live in Data ▸ Travel library (⇧L); apply or capture one from the tool options bar above",
		"text_ghost", DccTheme.FS_TINY)
	footer.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	var footer_pad := MarginContainer.new()
	footer_pad.add_theme_constant_override("margin_left", 14)
	footer_pad.add_theme_constant_override("margin_top", 8)
	footer_pad.add_theme_constant_override("margin_bottom", 8)
	footer_pad.add_child(footer)
	_left_party_body.add_child(footer_pad)

## JP-01. `jpAutoPickTransport`'s own outcome, in prose -- the reference's
## `auto.hint`. Every number here is the picker's own return
## (`jp_auto_transport_dict`), not a re-derivation in GDScript.
## What the per-stage picker actually changed, in the party form's own words.
## Every number here is `jp_compute`'s `stage_picks` -- nothing is recomputed
## on this side.
func _stage_picks_note() -> String:
	if not bool(_last_result.get("ok", false)):
		return "Compute a route to see what each stage would rather be carried by."
	var picks: Array = _last_result.get("stage_picks", [])
	if picks.is_empty():
		return "No stage is better off re-packed: every land stage is already within 10% of the best species, vehicle and mode available to this party on its own ground."
	var lines: Array[String] = []
	for p in picks:
		var d: Dictionary = p
		var changed: Array[String] = []
		if String(d.get("species", "")) != "":
			changed.append(String(d["species"]))
		if String(d.get("vehicle", "")) != "":
			changed.append(String(d["vehicle"]))
		if String(d.get("transport", "")) != "":
			changed.append(String(d["transport"]))
		var head := "Stage %d (%s): %s" % [int(d.get("stage", 0)) + 1, d.get("terrain", "?"), " + ".join(changed)]
		if bool(d.get("unblocks", false)):
			lines.append("%s -- was impassable, now %.0f km/day. %s" % [head, float(d.get("daily_km_after", 0.0)), d.get("reason", "")])
		else:
			lines.append("%s -- %.0f -> %.0f km/day (+%.0f%%). %s" % [
				head, float(d.get("daily_km_before", 0.0)), float(d.get("daily_km_after", 0.0)),
				float(d.get("gain_pct", 0.0)), d.get("reason", "")])
	return "\n".join(lines)

func _auto_carriage_note() -> String:
	var auto: Dictionary = _last_result.get("auto", {})
	if auto.is_empty():
		return "Auto picks the transport, animal species and vehicle count for this route (jpAutoPickTransport: best animal for the route's terrain × biome, km-weighted). Compute a route to see the pick."
	var reason := String(auto.get("reason", ""))
	match reason:
		"no_land_stages":
			return "Auto has nothing to pick: this route has no land stage. A water leg picks its own vessel (jp_auto_stage_vessel)."
		"not_a_land_mode":
			return "Auto applies to Walking / Mounted Rider / Baggage Train. \"%s\" is a water mode, so the carriage counts below are left alone." % String(_plan_values.get("transport", ""))
		"walking":
			return "Walking: the party carries its own load — %s kg needed against %s kg of porter capacity, so no animals or vehicles are assigned." % [
				_fmt_thousands(float(auto.get("total_need", 0.0)), 0), _fmt_thousands(float(auto.get("porter_cap", 0.0)), 0)]
		"walking_overloaded":
			return "⚠ Walking, over capacity: %s kg needed against %s kg of porter capacity, and auto-promote is off — so animals and vehicles stay cleared rather than a pack train being invented. Turn on \"Auto-promote\" below, or reduce the load." % [
				_fmt_thousands(float(auto.get("total_need", 0.0)), 0), _fmt_thousands(float(auto.get("porter_cap", 0.0)), 0)]
		"mount":
			return "Mounted Rider on %s — %s" % [String(auto.get("species", "")), String(auto.get("why", ""))]
		"baggage_train":
			var head := "Baggage Train"
			if bool(auto.get("promoted", false)):
				head = "Promoted Walking → Baggage Train"
			var txt := "%s: %d × %s, %d cart(s), %d wagon(s) — %s" % [
				head, int(auto.get("count", 0)), String(auto.get("species", "")),
				int(auto.get("carts", 0)), int(auto.get("wagons", 0)), String(auto.get("why", ""))]
			if bool(auto.get("fodder_infeasible", false)):
				txt += "  ⚠ No animal count solves this trip: at this length one animal can no longer carry its own fodder, so every animal added makes the shortfall worse. The count shown is an honest floor (cargo + supplies), not an answer — shorten the trip, raise grazing, or resupply."
			return txt
	return "Auto ran but reported no outcome this build understands (\"%s\")." % reason

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

# =============================================== Travel Library wiring (TL-01) ====
#
# `TRAVEL_LIBRARY_SPEC.md` §1's own promise -- "everything defined here becomes
# a selectable option in the planner's party form" -- and gap-register rows
# JP-02/IN-06. Three controls read the live library rather than `jp_options()`'s
# built-in vocabulary: the per-species animal-definition pickers, the Mount
# picker, and the Vessel picker.
#
# The boundary, and where it is stated: only the four built-in party-form
# species (donkey/mule/camel/horse) have a `JpParty` slot, so an animal entry
# is offerable only if it *resolves* to one of them -- its own `species_key`
# (every stock species entry, and any custom duplicate of one) or the one its
# "Substitutes for" chain reaches. `tl_list("animal")`'s `species_slot` key is
# that resolution, computed engine-side by
# `travel_bridge::TravelLibrary::animal_species_slot`; nothing here re-derives
# it. Entries that resolve to no slot -- the stock Ox/Yak/Reindeer, and every
# from-blank custom animal until its owner fills "Substitutes for" in -- are
# named in `_unslotted_note()` with the fix, rather than silently omitted.
#
# Vessels and vehicles remain data-only engine-side (no resolver equivalent to
# the animal one exists for `jp_ship_stats`/`jp_capacity`'s constants), so the
# Vessel picker lists every library vessel but leaves the ones with no engine
# counterpart *disabled with the reason on the item*, which is where a user
# actually hits the limit.

const _SPECIES := ["donkey", "mule", "camel", "horse"]
const _SPECIES_LABEL := {"donkey": "Donkey", "mule": "Mule", "camel": "Camel", "horse": "Horse"}

func _refresh_library() -> void:
	_library_animals = bridge.tl_list("animal") if bridge != null else []
	_library_vessels = bridge.tl_list("vessel") if bridge != null else []
	# Seed (and repair) the per-species selection: each species defaults to
	# its own stock entry, which `animal_overrides_selected` reads as "no
	# override" -- so an untouched form computes exactly what it always did.
	for species in _SPECIES:
		var current := String(_animal_entries.get(species, ""))
		if current != "" and _library_row(_library_animals, current).size() > 0:
			continue
		_animal_entries[species] = _stock_entry_id(species)

func _library_row(rows: Array, id: String) -> Dictionary:
	for r in rows:
		if String((r as Dictionary).get("id", "")) == id:
			return r
	return {}

func _stock_entry_id(species: String) -> String:
	for r in _library_animals:
		var row: Dictionary = r
		if String(row.get("origin", "")) == "stock" and String(row.get("species_slot", "")) == species:
			return String(row.get("id", ""))
	return ""

## Every animal entry that may occupy `species`, in the library's own list
## order (stock first, then custom in add order).
func _slot_candidates(species: String) -> Array:
	var out: Array = []
	for r in _library_animals:
		var row: Dictionary = r
		if String(row.get("species_slot", "")) == species:
			out.append(row)
	return out

## One entry's dropdown label. The `· custom` tag is `2b`'s own mono
## `custom · edited …` treatment, cut to the part that is true here (no edit
## timestamps exist in this port); `⚠` carries §4's validation state through.
func _entry_label(row: Dictionary) -> String:
	var text := String(row.get("name", ""))
	if String(row.get("origin", "")) == "custom":
		text += "  · custom"
	match String(row.get("validation_state", "ok")):
		"incomplete":
			text += "  ⚠"
		"conflicting":
			text += "  ⚠⚠"
	return text

## The four per-species definition pickers. Rendered whether or not any
## custom entry exists: the row is where a user learns the choice is theirs.
func _build_animal_definitions(parent: Control) -> void:
	parent.add_child(DccTheme.rule())
	var head := DccTheme.mono_label("ANIMAL DEFINITIONS · TRAVEL LIBRARY", "text_faint", DccTheme.FS_MICRO, 2)
	parent.add_child(head)
	for species in _SPECIES:
		_animal_entry_field(parent, species)
	var note := _unslotted_note()
	if note != "":
		DccWidgets.note(parent, note)

func _animal_entry_field(parent: Control, species: String) -> void:
	var candidates := _slot_candidates(species)
	var labels: Array = []
	var ids: Array = []
	for row in candidates:
		labels.append(_entry_label(row))
		ids.append(String((row as Dictionary).get("id", "")))
	if labels.is_empty():
		# Only reachable if the stock entry for this species was somehow
		# absent; say so rather than drawing an empty control.
		DccWidgets.note(parent, "%s: no Travel Library definition available." % _SPECIES_LABEL[species])
		return
	var current := String(_animal_entries.get(species, ""))
	var idx: int = ids.find(current)
	if idx < 0:
		idx = 0
	var ob := DccWidgets.choice(parent, _SPECIES_LABEL[species], labels, idx,
		func(i: int):
			_animal_entries[species] = ids[i]
			_plan_value_changed(true),
		"Which Travel Library definition this species' capacity, speed, fodder, water and terrain table come from. A stock entry means the built-in figures; a custom one re-plans the journey from its own.")
	ob.custom_minimum_size.y = 22
	if String(_library_row(_library_animals, current).get("origin", "")) == "custom":
		ob.add_theme_color_override("font_color", DccTheme.c("accent"))

## The honest boundary, stated where a user meets it: every custom animal
## that cannot be offered at all, by name, with the one edit that fixes it.
func _unslotted_note() -> String:
	var names: Array[String] = []
	for r in _library_animals:
		var row: Dictionary = r
		if String(row.get("species_slot", "")) != "":
			continue
		if String(row.get("origin", "")) != "custom":
			continue
		names.append(String(row.get("name", "")))
	if names.is_empty():
		return ""
	return ("Not offerable: %s. JpParty has four fixed species slots (donkey/mule/camel/horse) and no generic animal-count map, so a wholly new species has nowhere to sit. Set \"Substitutes for\" on it (Data ▸ Travel library ⇧L) to the id of the built-in species it stands in for — it then occupies that slot with its own capacity, speed, fodder, water and terrain table. Seasonal physiology and desert food/water multipliers still come from the substituted species: TRAVEL_LIBRARY_SPEC.md §3.1 carries no fields for either." % ", ".join(names))

## Mount picker, library-backed. Selecting a custom entry sets both the
## engine's `mount_animal` species key AND that species' definition slot --
## the same two facts one choice implies.
func _mount_field(parent: Control) -> void:
	var labels: Array = [_auto_label("mount_animal")]
	var species_of: Array = [""]
	var ids: Array = [""]
	for species in _SPECIES:
		for r in _slot_candidates(species):
			var row: Dictionary = r
			if not bool(row.get("usable_as_mount", false)):
				continue
			labels.append("%s  ›  %s" % [_SPECIES_LABEL[species], _entry_label(row)])
			species_of.append(species)
			ids.append(String(row.get("id", "")))
	var current_species := String(_plan_values.get("mount_animal", ""))
	var current_id := String(_animal_entries.get(current_species, ""))
	var idx := 0
	for i in ids.size():
		if String(species_of[i]) == current_species and String(ids[i]) == current_id:
			idx = i
			break
	var ob := DccWidgets.choice(parent, "Mount", labels, idx,
		func(i: int):
			_plan_values["mount_animal"] = species_of[i]
			if String(species_of[i]) != "":
				_animal_entries[species_of[i]] = ids[i]
			_plan_value_changed(true),
		"Only consulted when the party carries no donkeys/mules/camels/horses of its own. Entries are the Travel Library's own mount-capable definitions (§3.1 'usable as a mount').")
	_auto_obs["mount_animal"] = ob

## Every vessel `jp_compute` will accept for `plan.vessel`: the built-in
## roster plus every Travel Library definition the vessel resolver can turn
## into `ShipStats`. Used by this picker and by the stage inspector's own
## per-stage Vessel override, so both offer the same set.
##
## IN-06's remainder, closed: a library vessel reaches the engine through
## `TravelLibrary::vessel_overrides` -> `travel_library::vessel_resolver_fn`
## -> `JpVesselResolver`, the exact sibling of the animal resolver. Only an
## entry still missing one of its four numeric fields stays out, because the
## resolver declines an incomplete definition rather than shipping a hull
## with a zero hold.
func _vessel_names() -> Array:
	var out: Array = []
	for n in (_options.get("vessel", PackedStringArray()) as PackedStringArray):
		out.append(String(n))
	for r in _library_vessels:
		var row: Dictionary = r
		var vessel_name := String(row.get("name", ""))
		if vessel_name != "" and not out.has(vessel_name) and String(row.get("validation_state", "")) == "ok":
			out.append(vessel_name)
	return out

func _vessel_field(parent: Control) -> void:
	var engine_names: PackedStringArray = _options.get("vessel", PackedStringArray())
	var labels: Array = []
	var names: Array = []
	var live: Array = []
	for r in _library_vessels:
		var row: Dictionary = r
		var vessel_name := String(row.get("name", ""))
		var complete := String(row.get("validation_state", "")) == "ok"
		var hooked := engine_names.has(vessel_name) or complete
		labels.append(_entry_label(row) if hooked else "%s  — incomplete" % _entry_label(row))
		names.append(vessel_name)
		live.append(hooked)
	# Any engine vessel with no library row at all still has to be reachable.
	for n in engine_names:
		if not names.has(String(n)):
			labels.append(String(n))
			names.append(String(n))
			live.append(true)
	var current := String(_plan_values.get("vessel", ""))
	var idx: int = names.find(current)
	if idx < 0:
		idx = 0
	var ob := DccWidgets.choice(parent, "Vessel", labels, idx,
		func(i: int):
			if not bool(live[i]):
				return
			_plan_values["vessel"] = String(names[i])
			_plan_value_changed(false),
		"Every vessel here drives the real water calculation: the eleven built-in hulls through jp_ship_stats, and any Travel Library definition through the vessel resolver (its speed, hold, crew and water rating). An entry still missing one of those four fields is disabled — the resolver declines an incomplete definition rather than sailing a hull with a zero hold. One limit worth knowing: §3.3 has no per-water-type blacklist field, so a custom vessel is constrained by its mode and water rating only, never by a named water type the way \"River Barge cannot navigate River with Rapids\" is.")
	for i in live.size():
		if not bool(live[i]):
			ob.set_item_disabled(i, true)

## JP-16. `_jpPackRange`'s own ceiling advisory (reference line 19657, the
## v1.49 fix), attached to the supplies field per `JOURNEY_PLANNER_SPEC.md`
## §5 -- now computed by `cartalith_civ::jp_pack_range` through the
## `jp_pack_range` binding rather than approximated here.
##
## **What this replaces, and why the replacement is not cosmetic.** Until
## 2026-08-26 this function read the last compute's own land-leg `capacity
## .fodder` and reported "roughly N day(s) as configured" -- which restated
## the *current* supply setting and never stated the **ceiling**. That is
## precisely the pre-v1.48 behaviour the reference wrote `_jpPackRange` to
## end (`PARITY_AUDIT.md` §23 F13, the owner's own report: *"250kg of cargo
## now necessitates roughly 213 mules"*). The engine function had been
## ported and golden-tested since milestone 6 and was called by nothing.
##
## Three states, all the reference's own: full grazing (no ceiling exists),
## under the ceiling, and past it -- the last coloured `warn`, because
## beyond it no pack-train size works at all.
func _refresh_pack_range_note() -> void:
	if _pack_range_label == null or not is_instance_valid(_pack_range_label):
		return
	var pr := _pack_range()
	if pr.is_empty() or not bool(pr.get("ok", false)):
		## No pack animal in the party -- the reference's own `return null`.
		## There is no fodder ceiling without an animal carrying its own fodder.
		_pack_range_label.text = ""
		_pack_range_label.visible = false
		return
	_pack_range_label.visible = true
	var species := String(pr.get("label", "animal")).to_lower()
	if bool(pr.get("unlimited", false)):
		_pack_range_label.text = "Full grazing — the %ss feed themselves on route, so no carry-duration ceiling applies." % species
		_pack_range_label.add_theme_color_override("font_color", DccTheme.c("text_ghost"))
		return
	var ratio := float(pr.get("ratio", 0.0))
	var max_days := float(pr.get("max_days", 0.0))
	var text := "A %s can carry at most ~%.0f days of its own fodder at this grazing setting — past that its whole load is its own food." % [species, max_days]
	var token := "text_ghost"
	if ratio >= 1.0:
		token = "warn"
		text = "%s %s Beyond this no pack-train size works: shorten the carry, graze more, or resupply at a stop." % [DccIcons.SYMBOLS["warn_tri"], text]
	elif ratio >= 0.7:
		token = "stale"
		text += " You are close to that ceiling."
	_pack_range_label.text = text
	_pack_range_label.add_theme_color_override("font_color", DccTheme.c(token))

## `jp_pack_range(plan, has_desert)`. Pure and world-free, so it answers
## before a route is committed and before `generate()` -- which is the whole
## point of the v1.49 fix. `has_desert` comes off the last computed journey
## when there is one (the reference reads it off the finished plan for the
## same reason: a desert crossing changes what an animal eats); `false`
## otherwise, which is the reference's own value when `_jpPlan` throws.
func _pack_range() -> Dictionary:
	if not _bound:
		return {}
	var plan: Dictionary = _last_result.get("plan", {}) if bool(_last_result.get("ok", false)) else {}
	## Through the bridge, not around it. `EngineBridge.jp_pack_range()` performs
	## the same `has_method` probe this used to do inline -- and additionally
	## records the miss in `missing_bindings()`, which is the shell's staleness
	## fingerprint. A call site that reaches `world_gen` directly is invisible to
	## it, so a stale binary looks healthy from the one place that reports on it.
	return bridge.jp_pack_range(_plan_values, bool(plan.get("has_desert", false)))

# =========================================================== Compute path ====

func _compute() -> void:
	if not _bound or _route_index < 0:
		_last_result = {}
		_apply_result()
		return
	var request: Dictionary = {"route": _route_index, "plan": _plan_values.duplicate(true)}
	if _carriage_auto:
		request["auto_carriage"] = true
	if _stage_auto:
		request["auto_stage"] = true
	if _trim.x > 0.0 or _trim.y < 1.0:
		request["trim"] = _trim
	if not _animal_entries.is_empty():
		request["animal_entries"] = _animal_entries.duplicate()
	if not _stage_overrides.is_empty():
		var ov: Dictionary = {}
		for idx in _stage_overrides:
			ov[idx] = (_stage_overrides[idx] as Dictionary).duplicate(true)
		request["stage_overrides"] = ov
	if not _layovers.is_empty():
		request["layovers"] = _layovers.duplicate(true)
	_last_result = bridge.jp_compute(request)
	_apply_result()

## The carriage keys `jpAutoPickTransport` mutates on the plan -- the exact
## set the reference's `_jpSyncAssetInputs` (line 19632) writes back into the
## form's own disabled inputs.
const _AUTO_CARRIAGE_KEYS := ["donkey", "mule", "camel", "horse",
	"carts", "wagons", "travois", "sleds", "transport", "mount_animal"]

## `_jpRunAuto` mutates the plan; the form has to show what it picked, or the
## Auto counts stay at whatever Manual last set (which is exactly the gap the
## old "toggling Auto only disables editing them" note disclosed). Writes the
## picked values back into `_plan_values` and rebuilds the form when the
## picker *promoted* the transport, matching the reference's own
## `if(auto&&auto.promoted) structural=true`.
func _sync_auto_carriage() -> bool:
	if not _carriage_auto:
		return false
	var auto: Dictionary = _last_result.get("auto", {})
	if auto.is_empty() or not bool(auto.get("ok", false)):
		return false
	var picked: Dictionary = auto.get("plan", {})
	for key in _AUTO_CARRIAGE_KEYS:
		if picked.has(key):
			_plan_values[key] = picked[key]
	return bool(auto.get("promoted", false))

func _apply_result() -> void:
	var plan: Dictionary = {}
	if bool(_last_result.get("ok", false)):
		plan = _last_result.get("plan", {})
	if _sync_auto_carriage():
		# Walking -> Baggage Train changes which rows the form shows, so it
		# has to be a rebuild rather than a relabel. `_rebuild_party_form()`
		# does not recompute, so this cannot recurse.
		_rebuild_party_form()
	var stages: Array = plan.get("stages", [])
	if _selected_stage >= stages.size():
		_selected_stage = maxi(0, stages.size() - 1)

	_rebuild_route_map(plan)
	_refresh_route_map_layer_texture()
	_rebuild_profile(plan)
	_rebuild_stops(plan)
	_rebuild_inspector(plan)
	_rebuild_matrix(plan)
	_rebuild_timeline_band(plan)
	_refresh_auto_labels()
	_phone_refit()   ## PH-12: five of those six rebuilt from fresh nodes.
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
	_profile.trim = _trim
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

## Phone (§13) -- PH-12.
##
## Journey is the one screen in this pass that is **not** a `Window`: it swaps
## the whole domain region in place (this file's own header). Half of it is
## therefore already covered -- `_left_panel` hangs off `app.left_dock_body`,
## and `DccShell::_on_phone_node_added()` phone-fits every dock descendant; the
## results panel goes through `right_dock.gd`, same thing. What is NOT covered
## is this centre panel: it is parented to `app.viewport_content`, which the
## dock walker deliberately does not touch, so every one of its constants was
## drawn at native device resolution.
##
## And unlike a `Window`, it has no `content_scale_factor` to lean on -- there
## is no sub-viewport here, so an authored pixel in the main viewport really is
## one physical pixel. That is why `phone_fit()` is called with
## `app.phone_scale()` rather than the `1.0` every window in this pass uses:
## the compositor has applied nothing, so the walk has to apply everything.
##
## Two compositions do not survive the scale and are answered here instead:
## the route map's 196 dp totals column beside it, and the 642 dp stage matrix
## beside the stage inspector. Both stack.
##
## **PH-16, found and fixed 2026-09-03: every fixed-height row below was
## scaled twice.** This function used to be `_pp(px)`, a local
## "authored px -> physical px" helper (`px * app.phone_scale()`) that
## `_build_center_panel()` called when setting `map_row_pad` / `map_row` /
## `_route_map_wrap` / `profile_wrap` / `profile_head` / `stops_wrap`'s
## `custom_minimum_size.y` -- pre-scaling them before they ever joined the
## tree. `_do_phone_refit()` below then calls `phone_fit(_center_panel,
## app.phone_scale())`, whose own walk (`DccShell.phone_fit()`) multiplies
## every `Control.custom_minimum_size` it finds by that same `unit` -- with no
## way to tell a value it is seeing for the first time from one this file had
## already scaled. Every row this file pre-scaled therefore left the panel at
## `phone_scale()^2` its intended height rather than `phone_scale()` -- on the
## `GUI_GAP_REGISTER.md` §50 handset that is 236 dp rendering at ~1 623
## physical px instead of ~619, which is where the
## measured "1 434 px of nothing, the map hidden behind it" came from: the
## inflated map row alone consumed most of the screen, pushing the profile
## spine, the stops strip, the stage inspector and the stage matrix below the
## bottom of the framebuffer.
##
## The rest of this file's own phone pattern (`tool_options_row`,
## `dcc_widgets.gd`'s row/slider/action factories) never pre-scales: every
## caller authors desktop-pixel constants and leaves ALL of the scaling to the
## one `phone_fit()` walk. `_build_center_panel()` now follows the same rule
## -- its six fixed heights are the bare authored ints (236, 150, 22, 32) --
## so `phone_fit()`'s single pass is the only multiplication that happens, and
## the composition above (the map needing "a height of its own... so the
## numbers under it still read") renders at the size it was designed for.
var _phone := false

## PH-12: `app.viewport_content` is outside `DccShell._on_phone_node_added()`'s
## dock walk, so nothing fits this subtree unless this file asks. Every one of
## the centre panel's regions -- totals, profile, stops, inspector, matrix -- is
## cleared and rebuilt on each `_compute()`, so the ask is deferred and repeated
## rather than one-shot; `phone_fit()` is idempotent by meta-flag, so a repeat
## costs one visit per already-sized control.
##
## `app.phone_scale()`, not `1.0`: there is no content-scaled sub-viewport here.
## See `_build_center_panel()`'s own header.
func _phone_refit() -> void:
	if _phone and _center_panel != null:
		_do_phone_refit.call_deferred()

func _do_phone_refit() -> void:
	if _phone and is_instance_valid(_center_panel):
		app.phone_fit(_center_panel, app.phone_scale())

func _build_center_panel() -> void:
	_phone = app != null and app.has_method("is_phone") and app.is_phone()
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
	## PH-12/PH-16: every fixed height in this panel is *authored* desktop
	## pixels, left un-scaled here -- exactly like `tool_options_row`'s own
	## constants (`dcc_shell.gd`'s `set_tool_options()` header). `phone_fit()`
	## is what multiplies it by `phone_scale()`, once, from `_do_phone_refit()`
	## below; pre-scaling it here too was PH-16 (this function's own header).
	## 236 authored px becomes ~619 physical px at that handset's measured
	## `phone_scale` of **2.621**, about 11 mm of a 165 mm screen, for the
	## panel's principal view.
	##
	## *2.748 was this comment's figure until 2026-09-03. It predates
	## `dcc_theme.gd`'s 393 -> 412 `PHONE_REF_SHORT` rebase, which dropped the
	## scale at 1080 to 2.621 — the value this file's own probe prints. A
	## verifier caught the two disagreeing.*
	var map_row: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	map_row.custom_minimum_size.y = 236
	map_row.add_theme_constant_override("separation", 0)
	var map_row_pad := PanelContainer.new()
	map_row_pad.custom_minimum_size.y = 236
	map_row_pad.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"bottom": 1}))
	map_row_pad.add_child(map_row)
	col.add_child(map_row_pad)

	## Three stacked layers, tree order not z_index -- `ViewportHost` itself
	## never needs z_index for exactly this kind of stack (`map_view` then
	## `_lod_layer` then `overlay`, three siblings added in draw order): a
	## CHILD always draws after its own PARENT's `_draw()`, but a negative
	## `z_index` to push behind a PARENT bleeds into comparisons against
	## ancestors too and can land behind an opaque panel background several
	## levels up -- found live, not guessed, when the LOD tiles this wrap
	## exists for went fully invisible under exactly that.
	_route_map_wrap = Control.new()
	_route_map_wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_route_map_wrap.size_flags_vertical = Control.SIZE_EXPAND_FILL
	if _phone:
		## Stacked, the map needs a height of its own or the totals column takes
		## the row; 60% of the band, so the numbers under it still read. Bare
		## authored px -- see the "Route map row" comment above.
		_route_map_wrap.custom_minimum_size.y = 150
	map_row.add_child(_route_map_wrap)

	_route_map = _RouteMapView.new()
	_route_map.set_anchors_preset(Control.PRESET_FULL_RECT)
	_route_map_wrap.add_child(_route_map)

	_route_line = _RouteLineLayer.new()
	_route_line.backdrop = _route_map
	_route_line.set_anchors_preset(Control.PRESET_FULL_RECT)
	_route_line.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_route_map_wrap.add_child(_route_line)

	_build_route_map_layer_button()

	var totals_panel := PanelContainer.new()
	if not _phone:
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
	_profile.trim_dragged.connect(_on_trim_dragged)
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
	# The chip row goes inside a plain `Control`, not straight into the
	# HBox. Measured 2026-08-23 on a real 1684 px-wide session: a 34-stop
	# route's chips (a settlement name at natural width plus a 60 px SpinBox
	# each) report a combined minimum width of ~7 400 px, and because a
	# Container propagates its children's minimum size upward with nothing
	# clipping it, **the whole centre column was being stretched to 7 417 px
	# inside its 748 px parent** — pushing the route map, the profile spine,
	# the inspector and the matrix mostly off-screen. A physical mouse could
	# reach only the first two of fourteen stage bands, which silently capped
	# the spine's own click-to-select and ⌥-isolate (both long shipped) as
	# well as this pass's ⇧-drag trim.
	#
	# A plain `Control` reports only its OWN `custom_minimum_size`, so the
	# propagation stops here while the row still receives the real width and
	# lays its chips across the distance axis exactly as designed; anything
	# that genuinely does not fit is clipped rather than shoving the layout.
	var stops_clip := Control.new()
	stops_clip.clip_contents = true
	stops_clip.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	stops_clip.size_flags_vertical = Control.SIZE_EXPAND_FILL
	stops_outer.add_child(stops_clip)
	_stops_row = HBoxContainer.new()
	_stops_row.add_theme_constant_override("separation", 8)
	_stops_row.set_anchors_preset(Control.PRESET_FULL_RECT)
	stops_clip.add_child(_stops_row)
	_stops_note = DccTheme.mono_label("", "text_ghost", DccTheme.FS_TINY)
	stops_outer.add_child(_stops_note)

	# -- Lower area: inspector + matrix -------------------------------------------
	## PH-12: the stage inspector beside a 642 dp stage matrix is 642 physical px
	## of a 1440 px screen for one of two panes, at authored type. Stacked, the
	## matrix keeps its own horizontal scroll (it is genuinely wide) and the
	## inspector gets the full width.
	var lower: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
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
	if _phone:
		matrix_wrap.size_flags_vertical = Control.SIZE_EXPAND_FILL
	else:
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

## The route-map backdrop's own layer picker. Default is `"map"` -- the real
## rendered terrain (`EngineBridge.color_texture()`, the same colour +
## hillshade texture `ViewportHost.refresh()` puts in `map_view`), which is
## what "a cutout of the map" meant in the first place. The other options are
## `debug_texture()`'s field rasters -- the same ones the main map's Layers
## popover (`layers_popover.gd`) offers, reduced to the four that actually
## bear on whether a route is viable (what the ground is, what water is near,
## what lives there, the shape of the land) -- plus None, the plain
## background this view had before any of this existed. Not a second copy of
## `LayersPopover`: six rows fit a plain `PopupMenu`, styled the same way this
## file already styles `mode_ob`/`pace_ob`'s option-button popups
## (`DccWidgets.style_popup`), with no need for a whole second `PopupPanel`
## class.
##
## `"map"` alone gets `ViewportHost`'s own deep-zoom LOD tiles
## (`_RouteMapView.set_backdrop`'s own doc explains why the other five
## can't) -- the same shader that composites them onto `color_texture()`.
const LOD_TILE_SHADER := preload("res://shell/lod_tile.gdshader")
## The LOD tile fetch targets THIS resolution on the crop's SHORTER world
## axis -- `_sync_lod()` takes `maxf()` of the two px-per-cell ratios, so it
## is the shorter axis that lands on exactly this figure and the longer one
## that gets proportionally more. Not the panel's own on-screen size:
## `_RouteMapView._sync_lod()`'s
## own doc explains why: the panel is ~230 px tall, but the engine can
## synthesize far more than that, and capping the fetch at display size
## would throw away real detail the world already has just because the
## widget showing it is small. Sprites still land on-screen through the
## panel's own `_fit()`, unchanged -- this only changes how much source
## detail feeds that downscale.
const ROUTE_MAP_CAPTURE_PX := 2048.0
const _ROUTE_MAP_LAYER_IDS := ["map", "water", "bclass", "cterrain", "wildlife", "off"]
const _ROUTE_MAP_LAYER_LABELS := {
	"map": "Map", "off": "None", "water": "Water", "bclass": "Biome",
	"cterrain": "Terrain", "wildlife": "Wildlife",
}

func _build_route_map_layer_button() -> void:
	_route_map_layer_btn = Button.new()
	_route_map_layer_btn.flat = true
	_route_map_layer_btn.focus_mode = Control.FOCUS_NONE
	_route_map_layer_btn.icon = DccIcons.get_icon("layers", 15)
	_route_map_layer_btn.tooltip_text = "Route map background — what the cutout under the line shows."
	## Icon-only, so `icon_alignment`'s `LEFT` default hangs the glyph off the
	## left edge of its own hit box -- `ViewportHost`'s own `_layers_btn` (the
	## same widget, same icon) carries the OnePlus 6T history that found this.
	_route_map_layer_btn.icon_alignment = HORIZONTAL_ALIGNMENT_CENTER
	## Anchored to the panel's top-right, growing LEFT and DOWN off that
	## corner. `position = Vector2(-26, 6)` + `size = Vector2(20, 20)` (what
	## this was) does NOT survive: a `Control` is clamped up to its combined
	## minimum size, and this themed icon button's is 35 x 27 -- so the rect
	## grew rightwards from a 20-wide assumption and hung 9 px off the panel's
	## right edge. Measured live, not guessed. `grow_horizontal` is the same
	## lever `dcc_shell.gd` uses for exactly this ("picks which edge stays put
	## while it grows"), which makes the inset exact whatever the theme says
	## the button's minimum is.
	_route_map_layer_btn.set_anchors_and_offsets_preset(Control.PRESET_TOP_RIGHT)
	_route_map_layer_btn.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	_route_map_layer_btn.grow_vertical = Control.GROW_DIRECTION_END
	_route_map_layer_btn.offset_right = -6
	_route_map_layer_btn.offset_top = 6
	## On the wrap, not `_route_map` -- `_route_map`/`_route_line` are two
	## siblings under it now (see `_build_center_panel()`'s own comment), and
	## the button has to sit above both regardless of which one currently
	## draws a cutout under it.
	_route_map_wrap.add_child(_route_map_layer_btn)

	## A child of `app`, not of the button -- `app.gd`'s own `layers_popover`
	## does the same (`add_child(layers_popover)` on the app root), and a
	## `Popup`/`Window` node's placement in the tree is about lifetime, not
	## visual parenting, so it belongs beside every other top-level popup
	## rather than nested inside this panel's own layout.
	_route_map_layer_popup = PopupMenu.new()
	DccWidgets.style_popup(_route_map_layer_popup)
	_route_map_layer_popup.id_pressed.connect(_on_route_map_layer_picked)
	app.add_child(_route_map_layer_popup)
	_route_map_layer_btn.pressed.connect(func():
		_rebuild_route_map_layer_popup()
		var pos := _route_map_layer_btn.global_position + Vector2(0, _route_map_layer_btn.size.y)
		_route_map_layer_popup.position = Vector2i(pos)
		_route_map_layer_popup.popup())

## Rebuilt on every open rather than held, matching `LayersPopover.rebuild()`:
## `available` can change between opens (a route drawn, then a save loaded
## over it) and re-reading `debug_layers()` is the only honest way to know.
func _rebuild_route_map_layer_popup() -> void:
	_route_map_layer_popup.clear()
	var flat := {}
	for g in bridge.debug_layers():
		for it in (g as Dictionary).get("items", []):
			var item: Dictionary = it
			flat[String(item.get("id", ""))] = item
	for i in _ROUTE_MAP_LAYER_IDS.size():
		var id: String = _ROUTE_MAP_LAYER_IDS[i]
		var label: String = String(_ROUTE_MAP_LAYER_LABELS[id])
		_route_map_layer_popup.add_radio_check_item(label, i)
		var idx := i   ## Items are added in `_ROUTE_MAP_LAYER_IDS` order with id == i, so index == id.
		_route_map_layer_popup.set_item_checked(idx, id == _route_map_layer_id)
		if id == "off":
			_route_map_layer_popup.set_item_tooltip(idx, "The plain background this view always had.")
			continue
		if id == "map":
			_route_map_layer_popup.set_item_tooltip(idx, "The rendered terrain — the same colour and hillshade texture the main map shows.")
			_route_map_layer_popup.set_item_disabled(idx, not bridge.has_world)
			continue
		var item: Dictionary = flat.get(id, {})
		var available: bool = bool(item.get("available", true))
		_route_map_layer_popup.set_item_disabled(idx, not available)
		_route_map_layer_popup.set_item_tooltip(idx, String(item.get("hint", "")))

func _on_route_map_layer_picked(id_index: int) -> void:
	if id_index < 0 or id_index >= _ROUTE_MAP_LAYER_IDS.size():
		return
	_route_map_layer_id = _ROUTE_MAP_LAYER_IDS[id_index]
	_refresh_route_map_layer_texture()

## Re-fetches the current layer id's texture and redraws. Called on a manual
## pick, and also from `_apply_result()` -- a plan recompute can follow a
## regenerate, and `color_texture()`/`debug_texture()` build fresh off
## whatever world is live, so a texture fetched before that regenerate would
## otherwise sit stale (wrong, not crashing) until the next manual pick.
func _refresh_route_map_layer_texture() -> void:
	var id := _route_map_layer_id
	var tex: Texture2D = null
	if id == "off":
		pass
	elif id == "map":
		tex = bridge.color_texture()
	else:
		## `debug_texture()`'s own contract: null for an unknown id or a view
		## this world has no input for -- exactly "no cutout", so no branch is
		## needed here for the disabled case.
		tex = bridge.debug_texture(id)
	## LOD tiling only for "map" -- `set_backdrop`'s own doc says why the
	## other four fields (and the flat base raster all six fall back to)
	## can't use it.
	_route_map.set_backdrop(tex, id == "map", bridge)
	## The halo pass `_route_line` draws is gated on `backdrop.map_texture !=
	## null`, which just changed.
	_route_line.queue_redraw()

func _rebuild_route_map(plan: Dictionary) -> void:
	for c in _totals_body.get_children():
		_totals_body.remove_child(c)
		c.queue_free()

	if _route_index < 0 or not _bound:
		_route_map.pts = PackedVector2Array()
		_route_map.stage_segments = []
		_route_map.stops = []
		_route_map.queue_redraw()
		_route_line.queue_redraw()
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
	_route_line.queue_redraw()

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
	# The trace group traces the SELECTED stage, so it follows the spine.
	if app != null and app.right_dock_ctrl != null:
		app.right_dock_ctrl.refresh_journey()

## JP-07. The trim cuts the route polyline inside `jp_compute` (its own
## `trim` request key -> `cartalith_civ::jp_trim_points`), so every stage
## index, stop key and per-stage override that comes back belongs to the
## trimmed route -- which is why the per-stage state is reset here, exactly
## as picking a different route already does.
func _on_trim_dragged(from_frac: float, to_frac: float) -> void:
	var next := Vector2(from_frac, to_frac)
	if next.is_equal_approx(_trim):
		return
	_trim = next
	_stage_overrides.clear()
	_layovers.clear()
	_selected_stage = 0
	_isolated_stage = -1
	_tool_options_journey()
	_compute()

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
	_override_choice_row(grid, idx, ov, "vessel", "Vessel", PackedStringArray(_vessel_names()),
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
		DccWidgets.style_popup(mode_ob.get_popup())
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
		DccWidgets.style_popup(pace_ob.get_popup())
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
		_preset_controls(row)
		var carriage_lbl := DccTheme.mono_label("carriage: %s" % ("auto" if _carriage_auto else "manual"), "text_ghost", DccTheme.FS_SMALL)
		row.add_child(carriage_lbl)
		var transport := String(_plan_values.get("transport", "Walking"))
		var reroute_btn := DccWidgets.action(row, "re-route for %s…" % transport, _reroute_journey)
		reroute_btn.disabled = count == 0
		reroute_btn.tooltip_text = "_jpRerouteForMode: re-paths this committed route's two endpoints under the domain %s implies (sea / river / land), and refuses an unreachable answer rather than drawing the straight-line fallback route_commit tolerates." % transport
		var trim_text := "⇧ drag spine to trim · ⌥ click isolates a stage"
		if _trim.x > 0.0 or _trim.y < 1.0:
			trim_text = "trimmed %d–%d%% · click the spine outside the range to clear" % [roundi(_trim.x * 100.0), roundi(_trim.y * 100.0)]
		row.add_child(DccTheme.label(trim_text, "accent" if (_trim.x > 0.0 or _trim.y < 1.0) else "text_ghost", DccTheme.FS_MICRO))
		row.add_child(DccTheme.spacer())
		var save_btn := DccWidgets.action(row, "save journey", _save_journey)
		save_btn.disabled = count == 0
		save_btn.tooltip_text = "Names this route + party form and adds it to the Journeys list in the left dock. File ▸ Save project writes that list into the archive as entities/journeys.json and reopening the project restores it -- it is not lost when the app closes. One real limit: a journey stores a route INDEX, so generating a new world discards the list rather than pointing it at routes that no longer exist."
		var export_btn := DccWidgets.action(row, "export table", _export_stage_table)
	)

# ================================== Journeys list / save (JP-06, JP-08) ====

func _save_journey() -> void:
	var d := ConfirmationDialog.new()
	d.title = "Save journey"
	d.min_size = Vector2i(380, 0)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 6)
	body.add_child(DccTheme.label("Name for this journey:", "text_dim", DccTheme.FS_SMALL))
	var le := LineEdit.new()
	var km := 0.0
	if bool(_last_result.get("ok", false)):
		km = float((_last_result.get("plan", {}) as Dictionary).get("km", 0.0))
	le.text = "Journey %d — %s km" % [_journeys.size() + 1, _fmt_thousands(km, 0)]
	le.select_all_on_focus = true
	body.add_child(le)
	body.add_child(DccTheme.label(
		"Stored in this project — written by File ▸ Save project, restored on open.",
		"text_ghost", DccTheme.FS_MICRO))
	d.add_child(body)
	d.confirmed.connect(func():
		var jname := le.text.strip_edges()
		if jname != "":
			_journeys.append({
				"name": jname,
				"route": _route_index,
				"plan": _plan_values.duplicate(true),
				"stage_overrides": _stage_overrides.duplicate(true),
				"layovers": _layovers.duplicate(true),
				"animal_entries": _animal_entries.duplicate(),
				"trim": _trim,
			})
			_active_journey = _journeys.size() - 1
			_refresh_route_choice()
			app.set_status("hint", "Saved journey \"%s\" — save the project to keep it." % jname, "accent")
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered()
	le.grab_focus.call_deferred()

func _load_journey(i: int) -> void:
	if i < 0 or i >= _journeys.size():
		return
	var j: Dictionary = _journeys[i]
	_active_journey = i
	_route_index = int(j.get("route", 0))
	_plan_values = (j.get("plan", {}) as Dictionary).duplicate(true)
	_stage_overrides = (j.get("stage_overrides", {}) as Dictionary).duplicate(true)
	_layovers = (j.get("layovers", {}) as Dictionary).duplicate(true)
	_animal_entries = (j.get("animal_entries", {}) as Dictionary).duplicate()
	_trim = j.get("trim", Vector2(0.0, 1.0))
	_selected_stage = 0
	_isolated_stage = -1
	_rebuild_party_form()
	_tool_options_journey()
	_compute()
	app.set_status("hint", "Loaded journey \"%s\"." % String(j.get("name", "")), "accent")

func _delete_journey(i: int) -> void:
	if i < 0 or i >= _journeys.size():
		return
	_journeys.remove_at(i)
	if _active_journey == i:
		_active_journey = -1
	elif _active_journey > i:
		_active_journey -= 1
	_refresh_route_choice()

## JP-03. `_jpRerouteForMode` over the selected committed route: re-paths its
## endpoints under the domain this journey's transport implies, rewrites the
## route in place (so `route_get`/`jp_compute`'s `route` index still names
## it), and recomputes. An unreachable answer is reported, not drawn -- the
## reference refuses `reachable:false` outright.
func _reroute_journey() -> void:
	if _route_index < 0 or not bridge.world_gen.has_method("jp_reroute"):
		app.set_status("hint", "jp_reroute is not exposed by this build's GDExtension binary -- rebuild cartalith-godot.", "warn")
		return
	var transport := String(_plan_values.get("transport", "Walking"))
	var r: Dictionary = bridge.world_gen.jp_reroute(_route_index, transport, "")
	if not bool(r.get("ok", false)):
		app.set_status("hint", String(r.get("error", "re-route failed")), "warn")
		return
	# The route's geometry changed under every stage index, so per-stage
	# overrides and layovers no longer name the same ground -- the same reset
	# picking a different route already does.
	_stage_overrides.clear()
	_layovers.clear()
	_selected_stage = 0
	_isolated_stage = -1
	_trim = Vector2(0.0, 1.0)
	_refresh_route_choice()
	_tool_options_journey()
	_compute()
	app.set_status("hint", "Re-routed for %s — %s km." % [transport, _fmt_thousands(float(r.get("km", 0.0)), 0)], "accent")

# ================================================ Party set-ups (JP-02) ====
#
# `TRAVEL_LIBRARY_SPEC.md` §3.4: one row = one preset of *party-form values
# only*, no route, and applying one leaves per-stage overrides untouched.
# `tl_get("preset", id)` returns exactly `PRESET_FIELD_KEYS` -- the same
# twenty keys `_plan_values` already speaks (`travel_bridge::preset_to_pairs`
# is `PartyPreset::apply_to`'s own inverse) -- so applying is an assignment,
# not a translation table that could drift.
#
# The reference's `JP_PRESETS` (JS-only, ~line 17595) is *not* what this
# reads: this port's presets are the Travel Library's own stored rows, stock
# and captured alike, which is the strictly larger thing.

func _preset_controls(row: HBoxContainer) -> void:
	var presets: Array = bridge.tl_list("preset") if bridge != null else []
	var labels: Array = ["party set-up…"]
	var ids: Array = [""]
	for p in presets:
		var pr: Dictionary = p
		labels.append(_entry_label(pr))
		ids.append(String(pr.get("id", "")))
	DccWidgets.choice(row, "set-up", labels, 0,
		func(i: int):
			if i > 0:
				_apply_preset(String(ids[i])),
		"Applies a Travel Library party set-up to the form below (§3.4: party-form values only — no route, and per-stage overrides are left untouched).")
	DccWidgets.action(row, "capture party…", _capture_preset).tooltip_text = \
		"Writes the current party form into a new Travel Library party set-up (TRAVEL_LIBRARY_SPEC.md §3.4)."

func _apply_preset(id: String) -> void:
	if id == "":
		return
	var entry: Dictionary = bridge.tl_get("preset", id)
	if not bool(entry.get("ok", false)):
		app.set_status("hint", "That party set-up no longer exists.", "warn")
		return
	# Only the keys the form itself owns: `tl_get` also returns the entry's
	# own metadata (`id`/`origin`/`validation_*`/…), which is not plan data.
	var applied := 0
	for key in _default_plan.keys():
		if key == "party_fields" or not entry.has(key):
			continue
		_plan_values[key] = entry[key]
		applied += 1
	_rebuild_party_form()
	_compute()
	app.set_status("hint", "Applied party set-up \"%s\" (%d fields)." % [String(entry.get("name", "")), applied], "accent")

func _capture_preset() -> void:
	var d := ConfirmationDialog.new()
	d.title = "Capture party from planner"
	d.min_size = Vector2i(360, 0)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 6)
	body.add_child(DccTheme.label("Name for the new party set-up:", "text_dim", DccTheme.FS_SMALL))
	var le := LineEdit.new()
	le.text = "Captured party"
	le.select_all_on_focus = true
	body.add_child(le)
	d.add_child(body)
	d.confirmed.connect(func():
		var preset_name := le.text.strip_edges()
		if preset_name != "":
			var result: Dictionary = bridge.tl_capture_preset_from_plan(preset_name, _plan_values.duplicate(true))
			if bool(result.get("ok", false)):
				app.set_status("hint", "Captured party set-up \"%s\"." % preset_name, "accent")
				_tool_options_journey()
			else:
				app.set_status("hint", "Capture failed: %s" % String(result.get("error", "")), "warn")
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered()
	le.grab_focus.call_deferred()

## The one export path with real data behind it: the stage matrix as CSV, to
## the OS clipboard. No CSV file-writer exists to save it to disk -- unrelated
## to the Journeys list, which does persist (`journeys_document()`); this is
## a missing *text file* writer, not a missing archive. A clipboard export is
## honest and immediately useful meanwhile.
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
	_build_risk_note(body, plan)
	_build_vessels_group(body, plan)
	_build_trace_group(body)

## JP-17. `jp_risk` (reference line 19385, a port of V1.915's
## `assessCampaignRisk` tiers) -- the campaign-duration advisory.
## `PARITY_AUDIT.md` §23 F13: ported with milestone 6, called by nothing.
##
## Placed exactly where the reference places it (line 19872: after the cost
## group, before the stage table) and for its reason: it is not a verdict
## about whether the journey *works* -- `_build_verdict_card` above owns
## that -- it is an advisory about what a journey of this *length* costs in
## attrition, fatigue and supply lines regardless of how feasible it is.
## Silent under ten travel days, which is the reference's own `null`.
func _build_risk_note(body: Control, plan: Dictionary) -> void:
	var risk := String(plan.get("risk", ""))
	if risk == "":
		return
	var l := DccWidgets.note(body, "%s %s" % [DccIcons.SYMBOLS["warn_tri"], risk])
	l.add_theme_color_override("font_color", DccTheme.c("warn"))

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

## The reference's own cost formatter (line 19855): `1.2k` past a thousand,
## whole numbers past a hundred, one decimal below.
func _fmt_wages(v: float) -> String:
	if v >= 1000.0:
		return "%.1fk" % (v / 1000.0)
	return ("%.0f" % v) if v >= 100.0 else ("%.1f" % v)

## JP-04. `jp_compute`'s own `cost` key -- `jp_plan_cost` -> the milestone-3
## `jp_journey_cost` that nothing used to call. Empty on a blocked journey,
## which is the reference's own `null` (it bails on `plan.blocked` before
## pricing anything), not a missing binding.
func _build_cost_group(body: Control) -> void:
	var g := DccWidgets.section(body, "Cost")
	var cost: Dictionary = _last_result.get("cost", {})
	if cost.is_empty():
		DccWidgets.note(g, "No cost for a blocked journey -- jpJourneyCost returns null when any stage is impassable, because there is no trip to price. Clear the block above and this fills in.")
		return
	var plan: Dictionary = _last_result.get("plan", {})
	var total := float(cost.get("total", 0.0))
	var cargo_t := float(cost.get("cargo_t", 0.0))
	_kv_row(g, "carriage", "%s  (%.2f t over %s km)" % [
		_fmt_wages(float(cost.get("carriage", 0.0))), cargo_t, _fmt_thousands(float(plan.get("km", 0.0)), 0)])
	_kv_row(g, "wages", "%s  (%d × %.0f d)" % [
		_fmt_wages(float(cost.get("wages", 0.0))), int(_plan_values.get("group_size", 1)), float(cost.get("days", 0.0))])
	if float(cost.get("crew", 0.0)) > 0.0:
		_kv_row(g, "crew", _fmt_wages(float(cost.get("crew", 0.0))))
	if float(cost.get("upkeep", 0.0)) > 0.0:
		_kv_row(g, "animals & vehicles", _fmt_wages(float(cost.get("upkeep", 0.0))))
	if float(cost.get("tolls", 0.0)) > 0.0:
		var borders := int(cost.get("borders", 0))
		_kv_row(g, "tolls", "%s  (%d frontier%s)" % [_fmt_wages(float(cost.get("tolls", 0.0))), borders, "" if borders == 1 else "s"])
	if float(cost.get("transship", 0.0)) > 0.0:
		_kv_row(g, "transshipment", "%s  (%d)" % [_fmt_wages(float(cost.get("transship", 0.0))), int(plan.get("transshipments", 0))])
	_kv_row(g, "total", "%s day-wages" % _fmt_wages(total), "text_bright")
	var ptk := float(cost.get("per_tonne_km", -1.0))
	if ptk >= 0.0:
		_kv_row(g, "per tonne-km", "%.3f" % ptk, "text_dim")
	var be := float(cost.get("break_even_per_tonne", -1.0))
	if be >= 0.0:
		DccWidgets.note(g, "The cargo must fetch at least %s day-wages per tonne more at the destination than it cost at the origin, simply to cover this journey." % _fmt_wages(be))
	else:
		DccWidgets.note(g, "No trade cargo -- this is the cost of moving the party itself.")
	DccWidgets.note(g, "Priced in day-wages (one day of unskilled labour), never a currency: the land/river/sea carriage ratios follow Diocletian's Price Edict, which this engine already uses for food logistics, while the absolute level of money in a world is the owner's to set. Tolls are approximated from territory changes along the route (JpDerivedStage.claimed_frac crossing 0.5), and the reference labels that an approximation itself.")

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
		# JP-09. `jp_water_window(cat, terrain)` -- hours actually under way
		# per day on this water type, which is a factor of the leg's own
		# daily_km rather than a label bolted beside it.
		_kv_row(g, "sailing window", "%.0f h/day · %s" % [
			float(water.get("sailing_window_h", 0.0)), String(s.get("terrain", ""))], "water")
	if not any:
		DccWidgets.note(g, "No water legs on this route.")
	else:
		DccWidgets.note(g, "The sailing window is the engine's own jp_water_window for each water type (a sheltered bay is worked in daylight; open sea is stood through the night), not the vessel's Travel Library \"sailing window\" field -- nothing in the engine couples the two, and pretending otherwise would invent a model.")
	_build_vessel_matrix_groups(body, plan, any)

## JP-17. `jp_vessel_matrix` (reference line 17984) -- `PARITY_AUDIT.md` §23
## F13: ported with milestone 2, called by nothing until 2026-08-26.
##
## The reference draws two views of this one pure table (line 19883) and both
## are built here, for its own stated reason: *"when the route HAS water, the
## vessels are scored on this route's own water types and the ones that
## cannot make it say why; with no water stages it falls back to the general
## reference, so the information is reachable from any route."*
##
## - **On this route** — every hull's km/day and days over *these* legs,
##   ranked, with the ones that cannot make it named and dimmed. Only when
##   the route has water legs.
## - **By water type** — the full hull x water grid, collapsed by default.
##   The fastest hull per column is lit in accent, which is the whole point:
##   an open-sea passage sails through the night (22 h) while a sheltered bay
##   is daylight-limited (9 h), so hull speed and hull rating pull against
##   each other and the fastest vessel is **not** the same everywhere.
##
## The grid scrolls horizontally inside the dock rather than widening it --
## same discipline the stops strip already had to learn (see its own note).
func _build_vessel_matrix_groups(body: Control, plan: Dictionary, has_water: bool) -> void:
	var vm := _vessel_matrix_data()
	if vm.is_empty():
		return
	var waters: Array = vm.get("waters", [])
	var vessels: Array = vm.get("vessels", [])
	if waters.is_empty() or vessels.is_empty():
		return
	var current := String(_plan_values.get("vessel", ""))

	if has_water:
		_build_vessels_on_route(body, plan, waters, vessels, current)

	var g := DccWidgets.group(body, "vessel reference · speed by water", false)
	DccWidgets.note(g, "km/day per water type: cruise x that water's sailing window x the fraction of cruise the hull realises. A dash is a hull not rated for that water at all -- a different statement from slow. Lit = fastest hull for that water.")
	var grid := GridContainer.new()
	grid.columns = waters.size() + 1
	grid.add_theme_constant_override("h_separation", 4)
	grid.add_theme_constant_override("v_separation", 3)
	grid.add_child(DccTheme.mono_label("vessel", "text_dim", DccTheme.FS_MICRO, 1, true))
	for w in waters:
		grid.add_child(DccTheme.mono_label(_water_column_label(w), "text_dim", DccTheme.FS_MICRO, 1, true))
	for v in vessels:
		var vd: Dictionary = v
		var vname := String(vd.get("name", ""))
		var is_current := vname == current
		grid.add_child(DccTheme.mono_label(("%s%s" % ["> " if is_current else "", vname]), "accent" if is_current else "text", DccTheme.FS_MICRO))
		var cells: PackedFloat64Array = vd.get("cells", PackedFloat64Array())
		for ci in waters.size():
			var km: float = cells[ci] if ci < cells.size() else -1.0
			if km < 0.0:
				grid.add_child(DccTheme.mono_label("—", "text_ghost", DccTheme.FS_MICRO))
				continue
			var best := String((waters[ci] as Dictionary).get("best_vessel", ""))
			grid.add_child(DccTheme.mono_label("%.0f" % km, "accent" if best == vname else "text_dim", DccTheme.FS_MICRO))
	var scroll := ScrollContainer.new()
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.custom_minimum_size.y = grid.get_combined_minimum_size().y
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(grid)
	g.add_child(scroll)

## The route-aware half: each hull totalled over *this* journey's water legs.
func _build_vessels_on_route(body: Control, plan: Dictionary, waters: Array, vessels: Array, current: String) -> void:
	var stages: Array = plan.get("stages", [])
	## (cat, terrain) -> column index in `waters`, so a leg finds its own
	## column rather than this file restating the water vocabulary.
	var col: Dictionary = {}
	for i in waters.size():
		var w: Dictionary = waters[i]
		col["%s|%s" % [String(w.get("cat", "")), String(w.get("terrain", ""))]] = i
	var legs: Array = []
	var water_km := 0.0
	for s in stages:
		var sd: Dictionary = s
		if String(sd.get("cat", "")) == "land":
			continue
		var key := "%s|%s" % [String(sd.get("cat", "")), String(sd.get("terrain", ""))]
		if not col.has(key):
			continue
		legs.append({"col": int(col[key]), "km": float(sd.get("km", 0.0)), "terrain": String(sd.get("terrain", ""))})
		water_km += float(sd.get("km", 0.0))
	if legs.is_empty() or water_km <= 0.0:
		return

	var scored: Array = []
	for v in vessels:
		var vd: Dictionary = v
		var cells: PackedFloat64Array = vd.get("cells", PackedFloat64Array())
		var days := 0.0
		var blocked_on := ""
		for leg in legs:
			var l: Dictionary = leg
			var ci: int = l["col"]
			var km: float = cells[ci] if ci < cells.size() else -1.0
			if km <= 0.0:
				blocked_on = String(l["terrain"])
				break
			days += float(l["km"]) / km
		scored.append({
			"name": String(vd.get("name", "")),
			"ok": blocked_on == "" and days > 0.0,
			"why": blocked_on,
			"days": days,
			"kmday": (water_km / days) if (blocked_on == "" and days > 0.0) else 0.0,
			"hold": float(vd.get("cargo_kg", 0.0)),
			"crew": int(vd.get("crew", 0)),
		})
	scored.sort_custom(func(a, b):
		if bool(a["ok"]) != bool(b["ok"]):
			return bool(a["ok"])
		return float(a["kmday"]) > float(b["kmday"]))

	var g := DccWidgets.group(body, "vessels on this route", false)
	DccWidgets.note(g, "%s km of water across %d leg(s). Speed is cruise x that water's sailing window x the fraction of cruise it realises -- cargo and weather aside, this is the ranking that matters." % [_fmt_thousands(water_km, 0), legs.size()])
	for s in scored:
		var sd: Dictionary = s
		var vname := String(sd["name"])
		var mark := "> " if vname == current else ""
		if bool(sd["ok"]):
			_kv_row(g, "%s%s" % [mark, vname], "%.0f km/d · %.1f d" % [float(sd["kmday"]), float(sd["days"])],
				"accent" if vname == current else "text")
		else:
			_kv_row(g, "%s%s" % [mark, vname], "%s cannot enter %s" % [DccIcons.SYMBOLS["blocked"], String(sd["why"])], "text_ghost")

## The reference's own column abbreviation (line 19921): a water-type name
## with its "River with "/"River " prefix stripped, so nine columns fit.
func _water_column_label(w: Variant) -> String:
	var d: Dictionary = w
	var t := String(d.get("terrain", ""))
	if t.begins_with("River with "):
		return t.substr(11)
	if t.begins_with("River "):
		return t.substr(6)
	return t

## `jp_vessel_matrix()`, fetched once per session. Empty on a binary without
## the binding, which is the same "rebuild cartalith-godot" state every other
## `has_method` guard in this file reports by simply drawing nothing.
func _vessel_matrix_data() -> Dictionary:
	if not _vessel_matrix.is_empty():
		return _vessel_matrix
	if not _bound:
		return {}
	## Through the bridge -- see `_pack_range()` for why the inline `has_method`
	## probe this replaces was the wrong shape.
	_vessel_matrix = bridge.jp_vessel_matrix()
	return _vessel_matrix

## JP-05. `GUI_GAP_REGISTER.md` §7.12's own proposal, built as an inline
## group rather than the spec's `⧉` window: one row per multiplicative term
## in the engine's application order, with the running value beside it. The
## rows come from `land.trace`/`water.trace` -- structured factors, not the
## reference's `formula` prose string, which stays out of the engine on
## purpose. The trace's own invariant (`∏ factor == daily_km`) is asserted in
## `cartalith-civ`, so the last running value here always equals the km/day
## the results panel reports above.
func _build_trace_group(body: Control) -> void:
	var g := DccWidgets.section(body, "Calculation trace")
	var plan: Dictionary = _last_result.get("plan", {})
	var results: Array = plan.get("results", [])
	if results.is_empty():
		DccWidgets.note(g, "No computed stage to trace.")
		return
	var idx: int = clampi(_selected_stage, 0, results.size() - 1)
	var r: Dictionary = results[idx]
	if bool(r.get("blocked", false)):
		DccWidgets.note(g, "Stage %02d is blocked, so no speed chain was computed for it: %s" % [idx + 1, String(r.get("blocked_reason", ""))])
		return
	var calc: Dictionary = r.get("land", r.get("water", {}))
	var trace: Array = calc.get("trace", [])
	if trace.is_empty():
		DccWidgets.note(g, "This build's GDExtension binary returns no trace -- rebuild cartalith-godot.")
		return
	DccWidgets.note(g, "Stage %02d · %s — click a band on the spine to trace another stage." % [idx + 1, String(r.get("cat", ""))])
	var running := 1.0
	for t in trace:
		var term: Dictionary = t
		var factor := float(term.get("factor", 1.0))
		running *= factor
		var key := String(term.get("key", ""))
		var detail := String(term.get("detail", ""))
		var lhs := key if detail == "" else "%s · %s" % [key, detail]
		# The first term is the base speed, not a multiplier: printing it as
		# "×4.0" would read as a factor applied to something.
		var rhs := ("%.2f" % factor) if key == "base" else "×%.3f    %.2f" % [factor, running]
		var token := "text" if factor >= 0.999 else ("warn" if factor < 0.7 else "text_dim")
		_kv_row(g, lhs, rhs, "text_bright" if key == "base" else token)
	_kv_row(g, "= km/day", "%.2f" % float(calc.get("daily_km", 0.0)), "accent")

# ================================================================ Draw views ====

## The route-map polyline, coloured per stage category with real geometry --
## `route_get()`'s own points, sliced per `plan.stages[i].{i0,i1}` (the same
## index range `jp_plan` derived that stage over). No SVG-cloning of the
## mockup's specific example curve; whatever route is actually committed
## draws its own real shape.
##
## **The backdrop is a real cutout, not a mockup.** `map_texture`, when set,
## is one of `EngineBridge.debug_texture()`'s grid-sized rasters -- the same
## bitmap the Layers popover paints over the main map -- cropped to exactly
## the world-space rect `_fit()` already computes for the line (same 12%
## margin, same bounding box), so the terrain under the line and the line
## itself are always in registration. The outer script's layer-picker button
## owns which id (`off`/`water`/`bclass`/`cterrain`/`wildlife`) is live; this
## class only draws whatever texture it is handed.
class _RouteMapView extends Control:
	## `ViewportHost.MAX_LOD_TILES_PER_UPDATE`'s own figure -- what that
	## budgets per single input event during interactive pan/zoom, this
	## panel needs once per route pick, layer switch or resize. `_sync_lod()`
	## backs off to a shallower zoom level rather than truncating the fetch
	## when a crop would need more than this -- see that call site's own
	## comment for why a truncated fetch is a real, found-live bug, not a
	## theoretical one.
	const _LOD_TILE_BUDGET := 48
	var pts: PackedVector2Array = PackedVector2Array()
	var stage_segments: Array = []   ## [{i0,i1,cat,blocked}]
	var stops: Array = []            ## [Vector2] world grid coords
	var map_texture: Texture2D = null
	## True only for the "map" layer -- see `set_backdrop`.
	var _use_lod := false
	var _lod_bridge: EngineBridge = null
	var _lod_sprites: Array = []   ## [Sprite2D] children -- draw order is tree order, see `_build_center_panel()`.
	## The grid-aligned union of every synthesized tile's own world footprint
	## -- NOT the route's `[minv,maxv]` (`_bounds()`), which `_sync_lod()` only
	## uses to pick which tiles to fetch. A tile is grid-square; the route's
	## own bbox+margin rarely lines up with that grid, so the tiles almost
	## always cover a bit more ground than the route asked for. `_route_line`
	## dims exactly this rect, not `[minv,maxv]` -- dimming the tighter box
	## left an undimmed sliver of full-brightness map wherever a tile stuck
	## out past it, a real seam, not a rounding nicety.
	var lod_world_bounds := Rect2()

	func _ready() -> void:
		resized.connect(func(): _sync_lod(); queue_redraw())
		## `map_texture` is a `gw x gh` per-cell raster -- one texel per world
		## grid cell, cropped to a small local region and magnified well past
		## 1:1 -- so it needs to lean on smoothing between texels rather than
		## show them as hard squares. Explicit rather than trusting the
		## project default: this node has no ancestor that overrides it today,
		## but `ViewportHost`'s own base map (`_raster()`) deliberately sets
		## `TEXTURE_FILTER_NEAREST`, and this class living beside that code is
		## exactly the situation where an inherited default is worth pinning.
		texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR

	## The one entry point the outer script uses to change what the backdrop
	## shows -- replaces a plain `map_texture = ...` assignment because the
	## "map" layer needs more than a texture swap: it also owns a set of
	## LOD-tile `Sprite2D` children (`_sync_lod`), which every other layer
	## must NOT carry.
	##
	## **Why only "map" gets LOD tiles.** `lod_bridge.rs`'s own "What a tile
	## actually contains" section is explicit: a tile is a relief-detail
	## *shade ratio*, not a picture -- `lod_tile.gdshader` multiplies it into
	## `color_texture()`'s own colour, sampled at the tile's footprint. There
	## is no equivalent shade-ratio synthesis for `debug_texture()`'s four
	## field views: `bclass`/`cterrain` are categorical class ids (no
	## sub-cell value to refine), and `water`/`wildlife` are derived scores
	## with no relief-detail model behind them either. Feeding any of those
	## into `LOD_TILE_SHADER`'s `base_tex` would multiply a real detail ratio
	## into a field it was never computed against -- wrong, not just blurry.
	## Those four (and "off") keep the flat, bilinear-smoothed crop `_draw()`
	## already draws; only "map" gets the sharper composited version.
	func set_backdrop(tex: Texture2D, use_lod: bool, bridge: EngineBridge) -> void:
		map_texture = tex
		_use_lod = use_lod
		_lod_bridge = bridge
		_sync_lod()
		queue_redraw()

	func _clear_lod_sprites() -> void:
		for s in _lod_sprites:
			if is_instance_valid(s):
				s.queue_free()
		_lod_sprites.clear()
		lod_world_bounds = Rect2()

	## Rebuilds `_lod_sprites` from scratch against the current route bounds
	## and world (fetch) and the panel's own size (placement only -- see
	## `ROUTE_MAP_CAPTURE_PX`). Cheap to just discard and refetch rather than
	## diff against the previous set: a route-preview panel redraws on a
	## route pick or a panel resize, neither of which is a per-frame event
	## the way `ViewportHost`'s camera pan/zoom is -- the incremental
	## reconciliation `_apply_lod_tiles` needs for THAT doesn't earn its
	## complexity here.
	func _sync_lod() -> void:
		_clear_lod_sprites()
		if not _use_lod or _lod_bridge == null or pts.size() < 2 or map_texture == null:
			return
		var rect := Rect2(Vector2.ZERO, size)
		var b := _bounds()
		var minv: Vector2 = b[0]
		var maxv: Vector2 = b[1]
		var bw: float = maxf(1e-6, maxv.x - minv.x)
		var bh: float = maxf(1e-6, maxv.y - minv.y)
		## Targets `ROUTE_MAP_CAPTURE_PX` on the crop's SHORTER world axis (the
		## longer one gets proportionally more, since `maxf` picks the bigger
		## of the two px-per-cell ratios) -- NOT `rect.size`, the panel's own
		## physical pixel size. Over-asking is safe: the backoff loop below
		## walks `z` back down until the tile count fits the budget, so this
		## only sets the ceiling that loop starts from. `_fit()`
		## still places the resulting tiles on-screen at the panel's real
		## size below; this only decides how much source detail is fetched
		## before that downscale, so a small panel still gets a genuinely
		## sharp capture rather than one throttled to its own display size.
		var s: float = maxf(ROUTE_MAP_CAPTURE_PX / bw, ROUTE_MAP_CAPTURE_PX / bh)
		var z: int = _lod_bridge.lod_level_for_zoom(s)
		var n: int = _lod_bridge.lod_tiles_per_axis(z)
		if n <= 0:
			return   ## No LOD binding on this build -- degrade to the flat crop.
		var g: Vector2i = _lod_bridge.grid_size()
		if g.x < 2 or g.y < 2:
			return
		var fit := _fit(rect, minv, maxv)
		## `tile_bounds`'s own definition (`lod_bridge.rs`): `[0,gw-1]x[0,gh-1]`
		## split `n` ways per axis, tiles sharing their edge sample. No half-
		## texel inset (`_lod_tile_rect`'s own `half`/`tw/(tw-1)` refinement,
		## for a continuously zooming camera keeping every texel centred on
		## its sample at any depth) -- this panel draws one static crop, not a
		## live zoom, and that difference really is sub-texel. `ponytail:` a
		## deliberate simplification, not an oversight.
		##
		## **`_lod_tile_rect`'s OTHER term is not optional, and is kept.**
		## `TILE_OFFSET` below is its `+ Vector2(0.5, 0.5)`, which is half a
		## *cell*, not half a texel: the pyramid indexes *samples* (cell
		## indices, `[0, gw-1]`), while `pts` -- and therefore `_fit`, and
		## `map_texture`'s own pixels -- are in the continuous cell-span
		## `[0, gw]` this shell draws route points in (`place_search.gd`'s
		## coordinate-space note: `_point_to_screen` divides by `gw` with no
		## `+0.5`, unlike `_cell_to_screen`'s cell-indexed markers). Sample
		## `i` therefore lives at cell-span `i + 0.5`. Dropping the conversion
		## slides the relief detail half a cell off the colour it multiplies,
		## which is the exact registration `lod_tile.gdshader`'s own header
		## says the two agree by construction on. It is ~1.6 px on a
		## world-crossing route and tens of px on a short local one, since the
		## panel's fit scale grows as the crop shrinks.
		const TILE_OFFSET := Vector2(0.5, 0.5)
		var step := Vector2(float(g.x - 1) / float(n), float(g.y - 1) / float(n))
		## Cell-span crop -> sample space, the same shift `_update_lod()`
		## writes inline as `(gx0 - 0.5) / step.x`.
		var smin := minv - TILE_OFFSET
		var smax := maxv - TILE_OFFSET
		var col0: int = clampi(int(floor(smin.x / step.x)), 0, n - 1)
		var col1: int = clampi(int(floor(smax.x / step.x)), 0, n - 1)
		var row0: int = clampi(int(floor(smin.y / step.y)), 0, n - 1)
		var row1: int = clampi(int(floor(smax.y / step.y)), 0, n - 1)
		## Back off to a shallower level -- fewer, BIGGER tiles, still real
		## LOD detail, just less of it -- until the full intersecting grid
		## fits `_LOD_TILE_BUDGET`. Capping the fetch LOOP instead (tried
		## first) truncates mid-grid and leaves the untouched remainder of
		## the panel solid black -- found live, not guessed, not a corner
		## case: it reproduced at a perfectly ordinary 1024x768 world. Every
		## tile in the FINAL [col0,col1]x[row0,row1] range always gets
		## fetched, so there is no resolution at which this can leave a gap.
		while (col1 - col0 + 1) * (row1 - row0 + 1) > _LOD_TILE_BUDGET and z > 0:
			z -= 1
			n = _lod_bridge.lod_tiles_per_axis(z)
			step = Vector2(float(g.x - 1) / float(n), float(g.y - 1) / float(n))
			col0 = clampi(int(floor(smin.x / step.x)), 0, n - 1)
			col1 = clampi(int(floor(smax.x / step.x)), 0, n - 1)
			row0 = clampi(int(floor(smin.y / step.y)), 0, n - 1)
			row1 = clampi(int(floor(smax.y / step.y)), 0, n - 1)
		## The grid-aligned union `lod_world_bounds`'s own doc explains --
		## the selected tile RANGE, not which fetches happen to succeed
		## below; a tile that fails to synthesize still occupies its square of
		## "this is the intended dim area" as far as the line layer is
		## concerned.
		var union_min := Vector2(col0 * step.x, row0 * step.y) + TILE_OFFSET
		var union_max := Vector2((col1 + 1) * step.x, (row1 + 1) * step.y) + TILE_OFFSET
		lod_world_bounds = Rect2(union_min, union_max - union_min)
		for row in range(row0, row1 + 1):
			for col in range(col0, col1 + 1):
				var tex: Texture2D = _lod_bridge.lod_synthesize_tile(z, col, row)
				if tex == null:
					continue
				## Sample bounds -> cell-span, so the sprite's screen rect AND
				## its `base_uv*` both land where `map_texture`'s own pixels
				## are -- see `TILE_OFFSET` above. Shifting both together
				## keeps the base colour identical to the flat-crop path's
				## `draw_texture_rect_region` while moving the relief detail
				## onto the ground it was computed for.
				var twmin := Vector2(col * step.x, row * step.y) + TILE_OFFSET
				var twmax := Vector2((col + 1) * step.x, (row + 1) * step.y) + TILE_OFFSET
				var p0: Vector2 = fit.call(twmin)
				var p1: Vector2 = fit.call(twmax)
				var sprite := Sprite2D.new()
				sprite.texture = tex
				sprite.centered = false
				## `LINEAR`, matching `_build_lod_tile`'s own reasoning: a tile
				## drawn at its own texel size must not reintroduce the hard
				## single-cell squares this whole feature exists to remove.
				sprite.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
				sprite.position = p0
				var tsz := Vector2(maxf(float(tex.get_width()), 1.0), maxf(float(tex.get_height()), 1.0))
				sprite.scale = (p1 - p0) / tsz
				## No explicit z_index: default 0, which already draws after
				## this control's own `_draw()` (nothing to be "behind" for
				## the LOD case -- `_draw()` skips the flat-crop path
				## whenever `_use_lod` did produce tiles, see below), and `_route_line`
				## (the line/markers/dim) is a SIBLING added after this whole
				## control in `_build_center_panel()`, so it draws after
				## these sprites without needing z_index at all. A negative
				## z_index was tried first and made every sprite invisible --
				## `z_as_relative` compares the accumulated z against
				## ancestors too, and it landed behind this panel's own
				## opaque background several levels up, not just behind this
				## control's `_draw()` as intended.
				var mat := ShaderMaterial.new()
				mat.shader = JourneyPlannerView.LOD_TILE_SHADER
				mat.set_shader_parameter("base_tex", map_texture)
				mat.set_shader_parameter("base_uv0", twmin / Vector2(g.x, g.y))
				mat.set_shader_parameter("base_uv1", twmax / Vector2(g.x, g.y))
				sprite.material = mat
				add_child(sprite)
				_lod_sprites.append(sprite)

	## The route's own world-space bounding box, 12% margin included -- the
	## exact rect `_fit()` used to normalise into before this existed, now
	## also the crop window for `map_texture`.
	func _bounds() -> Array:
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
		return [minv, maxv]

	func _fit(rect: Rect2, minv: Vector2, maxv: Vector2) -> Callable:
		var bw: float = maxf(1e-6, maxv.x - minv.x)
		var bh: float = maxf(1e-6, maxv.y - minv.y)
		var s: float = minf((rect.size.x - 20.0) / bw, (rect.size.y - 20.0) / bh)
		var ox: float = rect.position.x + (rect.size.x - bw * s) * 0.5
		var oy: float = rect.position.y + (rect.size.y - bh * s) * 0.5
		return func(p: Vector2) -> Vector2: return Vector2(ox + (p.x - minv.x) * s, oy + (p.y - minv.y) * s)

	## Only the backdrop: the "no route" placeholder, and the flat crop path
	## for every layer except "map" (which draws nothing here -- `_sync_lod`'s
	## sprite children are this control's own children, and they need to
	## render AFTER this call, which is exactly what a plain, non-negative
	## z_index child already does. The dim wash and the line/markers moved to
	## `_RouteLineLayer`, a SIBLING added after this whole control -- see
	## `_build_center_panel()`'s comment for why tree order replaces z_index
	## here, and `_RouteLineLayer._draw()` for where the rest of this went.
	func _draw() -> void:
		if pts.size() < 2:
			draw_string(ThemeDB.fallback_font, Vector2(14, 20), "no committed route selected",
				HORIZONTAL_ALIGNMENT_LEFT, -1, 11, DccTheme.c("text_ghost"))
			return
		## Only when tiles were actually built. `_sync_lod()`'s three early
		## returns (no LOD binding on this build, a degenerate grid, a null
		## `color_texture()`) each say "degrade to the flat crop" -- and a
		## bare `if _use_lod` here made that comment false, leaving the "Map"
		## layer blank in exactly the cases it promised a fallback for.
		if _use_lod and not _lod_sprites.is_empty():
			return
		var rect := Rect2(Vector2.ZERO, size)
		var b := _bounds()
		var minv: Vector2 = b[0]
		var maxv: Vector2 = b[1]
		var fit := _fit(rect, minv, maxv)
		if map_texture != null:
			var tsz := Vector2(map_texture.get_size())
			var src := Rect2(minv, maxv - minv).intersection(Rect2(Vector2.ZERO, tsz))
			if src.size.x > 0.0 and src.size.y > 0.0:
				var dest := Rect2(fit.call(src.position), fit.call(src.position + src.size) - fit.call(src.position))
				draw_texture_rect_region(map_texture, dest, src)
				## Dimmed so the route line and stop markers stay legible
				## over real map detail instead of competing with it.
				draw_rect(dest, Color(0, 0, 0, 0.32))

## The line/markers/dim-wash layer -- a SIBLING of `_RouteMapView`, added
## after it (`_build_center_panel()`), so it always draws on top of both the
## flat-crop backdrop AND the LOD sprite children by tree order, with no
## z_index needed anywhere. Reads its geometry from `backdrop` rather than
## owning a second copy: `pts`/`stage_segments`/`stops`/`map_texture`/
## `_use_lod` all still live on `_RouteMapView`, which is what
## `_rebuild_route_map`/`set_backdrop` already write to.
class _RouteLineLayer extends Control:
	var backdrop: _RouteMapView = null

	func _ready() -> void:
		resized.connect(func(): queue_redraw())

	func _draw() -> void:
		if backdrop == null or backdrop.pts.size() < 2:
			return   ## `_RouteMapView._draw()` already shows the placeholder text.
		var pts := backdrop.pts
		var stage_segments := backdrop.stage_segments
		var stops := backdrop.stops
		var map_texture := backdrop.map_texture
		var rect := Rect2(Vector2.ZERO, size)
		var b := backdrop._bounds()
		var minv: Vector2 = b[0]
		var maxv: Vector2 = b[1]
		var fit := backdrop._fit(rect, minv, maxv)
		if backdrop._use_lod and map_texture != null:
			## The LOD sprites are `backdrop`'s children, drawn before this
			## whole sibling control -- the dim wash belongs here, not there,
			## so it lands on top of them instead of under.
			##
			## Dims `lod_world_bounds` (what the tiles actually cover), NOT
			## `[minv,maxv]` (what the route asked for) -- see that field's
			## own doc. Using `[minv,maxv]` here left an undimmed sliver of
			## full-brightness map wherever a grid-aligned tile stuck out
			## past the route's own tighter bbox, a real visible seam.
			var lb := backdrop.lod_world_bounds
			if lb.size.x > 0.0 and lb.size.y > 0.0:
				var dest := Rect2(fit.call(lb.position), fit.call(lb.end) - fit.call(lb.position))
				draw_rect(dest, Color(0, 0, 0, 0.32))
		## A halo pass under the line, cutout only. `DccTheme.c("water")`'s
		## stage colour is exactly the biome/water cutout's own hue family --
		## plain background never had this problem, so the halo is scoped to
		## the one condition that creates it rather than changed unconditionally.
		var halo := map_texture != null
		if stage_segments.is_empty():
			var poly := PackedVector2Array()
			for p in pts:
				poly.append(fit.call(p))
			if halo:
				draw_polyline(poly, Color(0, 0, 0, 0.6), 3.6, true)
			draw_polyline(poly, DccTheme.c("accent"), 1.6, true)
		else:
			for seg in stage_segments:
				var d: Dictionary = seg
				var i0: int = clampi(int(d.get("i0", 0)), 0, pts.size() - 1)
				var i1: int = clampi(int(d.get("i1", i0)), i0, pts.size() - 1)
				## Extended one point past `i1` (when one exists): a single-
				## point stage (`i0 == i1`, real -- e.g. a waypoint with no
				## length of its own) would otherwise build a 1-point `poly`
				## and hit the `size() < 2` guard below, leaving that point
				## joined to NEITHER neighbour and a real gap in the line.
				## Every segment reaching one point into the next makes
				## consecutive segments always share a point, at the cost of
				## redrawing one point-length of the junction in each of the
				## two colours it touches -- imperceptible, and cheaper than
				## carrying `i1`'s successor across iterations by hand.
				var i1_ext: int = mini(i1 + 1, pts.size() - 1)
				var poly := PackedVector2Array()
				for i in range(i0, i1_ext + 1):
					poly.append(fit.call(pts[i]))
				if poly.size() < 2:
					continue
				var cat := String(d.get("cat", "land"))
				var blocked := bool(d.get("blocked", false))
				var col := DccTheme.c("block") if blocked else (DccTheme.c("water") if cat != "land" else DccTheme.c("accent"))
				var w := 3.0 if cat != "land" else 1.8
				if halo:
					draw_polyline(poly, Color(0, 0, 0, 0.6), w + 2.0, true)
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
	## JP-07: `JOURNEY_PLANNER_SPEC.md` §3's "⇧ drag trims". Two fractions of
	## the distance axis, always ordered low-high; `(0, 1)` clears the trim.
	signal trim_dragged(from_frac: float, to_frac: float)

	var profile: PackedFloat64Array = PackedFloat64Array()
	var bands: Array = []   ## [{start, end, cat, blocked, warn, label}] fractions 0..1
	var selected_idx := -1
	var isolated_idx := -1
	var trim := Vector2(0.0, 1.0)   ## the committed trim, drawn as dimmed margins

	var _drag_from := -1.0   ## >= 0 while a ⇧ drag is live
	var _drag_to := -1.0

	func _ready() -> void:
		resized.connect(func(): queue_redraw())
		mouse_filter = Control.MOUSE_FILTER_STOP

	func _gui_input(event: InputEvent) -> void:
		if size.x <= 0.0:
			return
		if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
			var frac: float = clampf(event.position.x / size.x, 0.0, 1.0)
			if event.pressed and event.shift_pressed:
				_drag_from = frac
				_drag_to = frac
				queue_redraw()
				return
			if not event.pressed and _drag_from >= 0.0:
				var a: float = minf(_drag_from, _drag_to)
				var b: float = maxf(_drag_from, _drag_to)
				_drag_from = -1.0
				_drag_to = -1.0
				queue_redraw()
				# A ⇧ click (rather than a drag) clears the trim: a zero-width
				# range is not a journey, and "click to clear" is the cheapest
				# undo the gesture can have.
				if b - a < 0.01:
					trim_dragged.emit(0.0, 1.0)
				else:
					trim_dragged.emit(a, b)
				return
			if event.pressed and not bands.is_empty():
				for i in bands.size():
					var band: Dictionary = bands[i]
					if frac >= float(band.get("start", 0.0)) and frac <= float(band.get("end", 1.0)):
						stage_clicked.emit(i, event.alt_pressed)
						return
		elif event is InputEventMouseMotion and _drag_from >= 0.0:
			_drag_to = clampf(event.position.x / size.x, 0.0, 1.0)
			queue_redraw()

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
		# JP-07's trim, drawn last so it reads as a mask over the whole spine
		# rather than as another band: the trimmed-away margins are veiled,
		# and a live ⇧ drag previews its own range the same way.
		var live: bool = _drag_from >= 0.0
		var lo: float = (minf(_drag_from, _drag_to) if live else trim.x)
		var hi: float = (maxf(_drag_from, _drag_to) if live else trim.y)
		if lo > 0.0 or hi < 1.0:
			var veil := Color(DccTheme.c("bg"), 0.62)
			if lo > 0.0:
				draw_rect(Rect2(0, 0, lo * w, h), veil)
			if hi < 1.0:
				draw_rect(Rect2(hi * w, 0, (1.0 - hi) * w, h), veil)
			var edge := DccTheme.c("accent")
			draw_line(Vector2(lo * w, 0), Vector2(lo * w, h), edge, 1.5)
			draw_line(Vector2(hi * w, 0), Vector2(hi * w, h), edge, 1.5)

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


# ================================================ Journeys, on disk (F10) ====
#
# `entities/journeys.json`, the slot `SAVEFILE_COMPAT.md` §9.6 reserved and
# nothing wrote. The list was in-session only, and this file's own header said
# so — the reason given was that no save-writer existed, which stopped being
# true on 2026-08-23, and then that GDScript state had no channel to the
# archive, which stopped being true when `project_save_with_documents` landed.
# What was actually missing by 2026-08-26 was a reader that returned the bytes
# that were stored rather than a re-serialisation of them; that is `afc2d57`,
# and this is the consumer it was built for.

## This view's half of the project file, as JSON **text**.
##
## `Vector2` is written as a two-element array because JSON has no vector, and
## `_trim` is the only field in a journey that is not already a JSON-native
## type. Everything else is exactly what `_save_journey` stored.
func journeys_document() -> String:
	var out: Array = []
	for j in _journeys:
		var d: Dictionary = (j as Dictionary).duplicate(true)
		var t: Vector2 = d.get("trim", Vector2(0.0, 1.0))
		d["trim"] = [t.x, t.y]
		out.append(d)
	return JSON.stringify({"journeys": out})

## The inverse. `app.gd::_restore_project_documents()` calls this with whatever
## the archive's `entities/journeys.json` slot held, once per project open.
##
## **Two guards, each of which was data loss before it existed.**
##
## 1. *Restore only when the documents are new.* This used to hang off `app.gd`'s
##    `world_loaded` handler, and that signal is emitted for seven different
##    reasons while only `EngineBridge.load_save()` ever assigns
##    `last_documents` — so centring the landmasses, carving fjords, applying an
##    asset pack or closing the world all replayed the *previous* archive's
##    journeys over everything planned since the file was opened. `app.gd` has
##    since moved the call to `_load_project()`, which is the right end of the
##    fix; this is the view's own half of it, and it is what makes the function
##    safe to call from any handler rather than from exactly one. `is_same()`
##    on the dictionary is the test: `load_save` assigns a **fresh**
##    `Dictionary` on every open (a re-open of the same path included), and
##    nothing else assigns it at all, so object identity means "new documents
##    arrived" and nothing else. Value equality would not do — two opens of the
##    same file carry equal text and must both restore.
## 2. *New documents with no journeys slot clear the list.* Keeping it was how
##    project A's journeys followed the user into project B, to be written into
##    B's archive by `journeys_document()` on the next save — carrying route
##    indices that index B's routes, which is the same corruption seen from the
##    other end. A flat legacy archive lands here too: `load_save` leaves
##    `documents` empty for it, which is a new (empty) dictionary and therefore
##    a genuine "this project has no journeys".
##
## A slot that is present but unparseable still leaves the list alone: that is
## a corrupt document, not an empty one, and it must not silently delete work.
func restore_journeys_document(text: String) -> void:
	if bridge != null:
		if is_same(bridge.last_documents, _restored_documents):
			return
		_restored_documents = bridge.last_documents
	if text.strip_edges() == "":
		clear_journeys()
		return
	var parsed = JSON.parse_string(text)
	if not (parsed is Dictionary):
		push_warning("Cartalith: entities/journeys.json is not an object; the journeys list is left alone")
		return
	var arr = (parsed as Dictionary).get("journeys", [])
	if not (arr is Array):
		return
	var loaded: Array = []
	for e in arr:
		if not (e is Dictionary):
			continue
		var d: Dictionary = (e as Dictionary).duplicate(true)
		var t = d.get("trim", [0.0, 1.0])
		## `route` is an index into the committed routes and MUST stay an int:
		## `JSON.parse_string` floats every number, and `jp_compute` rejects a
		## float where it wants an index. This is the shell's half of §14.1 —
		## the engine guarantees the bytes, not what GDScript does after
		## parsing them.
		d["route"] = int(d.get("route", 0))
		d["trim"] = Vector2(float(t[0]), float(t[1])) if (t is Array and (t as Array).size() == 2) else Vector2(0.0, 1.0)
		loaded.append(d)
	_journeys = loaded
	_active_journey = -1
	if _bound:
		_refresh_route_choice()

## Empties the list because the world its route indices pointed into is gone.
##
## Public: `setup()` connects it to `generation_finished` and to the
## world-less half of `world_loaded`, and `restore_journeys_document()` calls
## it when a newly opened project carries no journeys slot of its own.
## `_refresh_route_choice()` is what redraws the left dock's list, and it is
## only safe once `_build_left_panel()` has run — which `_bound` implies, since
## that is the only branch which creates `_left_route_section`.
func clear_journeys() -> void:
	if _journeys.is_empty() and _active_journey < 0:
		return
	_journeys = []
	_active_journey = -1
	if _bound:
		_refresh_route_choice()

## How many saved journeys reference one Travel Library entry —
## `TRAVEL_LIBRARY_SPEC.md` §4's "how many saved journeys reference it", which
## the inspector printed as a hard-coded `0`.
##
## It is answered here rather than in `travel_bridge.rs` because the journeys
## list is the shell's own state by design (see `_journeys`): the engine holds
## no journey to count. `travel_bridge.rs::animal_usage_in_journeys` was
## written when that was true of every journey anywhere; it is still true of
## the *engine*, and this is the reader that closes the gap on the shell side.
##
## `kind` is the Travel Library kind. An `animal` is matched by entry **id**
## through `animal_entries` (`_save_journey` stores that map verbatim); a
## `vessel` by **name**, because `JpPlan.vessel` is a name and the resolver
## keys on it (`travel_bridge.rs::vessel_overrides`). `vehicle` is always 0 and
## honestly so — no vehicle reaches a computed journey at all.
func journey_usage(kind: String, entry_id: String, entry_name: String) -> int:
	var n := 0
	for j in _journeys:
		var journey: Dictionary = j
		if kind == "animal":
			var entries: Dictionary = journey.get("animal_entries", {})
			if entry_id != "" and entries.values().has(entry_id):
				n += 1
		elif kind == "vessel":
			var plan: Dictionary = journey.get("plan", {})
			if entry_name != "" and String(plan.get("vessel", "")) == entry_name:
				n += 1
	return n
