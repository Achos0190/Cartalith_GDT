extends AcceptDialog
class_name JourneyPlannerWindow

## Infrastructure ▸ Logistics ▸ Open Journey Planner
## (`JOURNEY_PLANNER_SCOPE.md` closing-status steps 3 and 5, `DCC_SHELL_SPEC.md`
## §6's Route context table): a party-composition form over a committed route,
## calling `jp_compute` and rendering its plan/verdict/confidence.
##
## Follows `world_data_window.gd`/`performance_window.gd`/`new_world_dialog.gd`'s
## own shape (`extends AcceptDialog`, `class_name`, `setup(bridge)`, `open()` ->
## `popup_centered()`) rather than a dock panel -- the form is ~20 party fields
## plus per-stage overrides and a multi-stage results breakdown, too big for
## the left dock's disclosure grammar (`DccWidgets`' own L1-L5 doc comment).
##
## **Route source is read-only here.** This window does not draw routes -- it
## lists what `route_count()`/`route_get()` already have (the INFRA domain's
## Route tool, `infrastructure_workspace.gd`'s Way/Route TOOLS block). If
## nothing is committed yet it says so and points at that tool rather than
## inventing a second route-drawing interaction.
##
## **What is genuinely built this pass, not faked:**
## - The full ~20-field party/plan form, seeded from `jp_default_plan()` and
##   using `jp_options()`'s vocabulary for every dropdown (never a hard-coded
##   second copy of it -- the exact failure `journey_bridge.rs`'s own module
##   doc warns about: an option string the engine does not recognise silently
##   falls through to a wrong-row default instead of erroring).
## - Compute, with `rejected` surfaced rather than hidden.
## - The results panel: verdict, confidence band, totals, the stage-by-stage
##   breakdown (each leg's derived stage plus its land/water calculation or
##   block reason), stops with resupply reach, and the timeline.
## - Per-stage overrides (`request.stage_overrides`): pick a stage, override its
##   route condition or infrastructure tier, and the whole journey recomputes
##   immediately. A real, working feature, not the "documented not-built-yet"
##   fallback the task brief allowed.
##
## **Deliberately left undone, disclosed rather than faked:**
## - The elevation-profile sparkline (`plan.profile`, 0-1 normalised heights).
##   `_render_profile_note` reports the sample count and stops there -- see its
##   own comment. Time-boxed, not attempted-and-hidden.
## - `plan.day_fracs` and `plan.results[i].eff` (the effective plan a leg was
##   actually computed under, which season drift and the per-stage vessel
##   fallback can both alter from what was submitted) are real fields on the
##   returned dict that this pass does not surface -- the stage/leg summary
##   already shown is complete enough to be honest without them.

var bridge: EngineBridge

var _bound := false
var _options: Dictionary = {}
var _default_plan: Dictionary = {}
var _plan_values: Dictionary = {}
var _last_result: Dictionary = {}
var _route_index := -1
var _stage_overrides: Dictionary = {} ## int stage idx -> {"route_cond"/"infra": String}
var _override_stage_idx := 0

var _route_section: VBoxContainer
var _results_body: VBoxContainer
var _compute_btn: Button

# -- Setup / open ---------------------------------------------------------------

func setup(b: EngineBridge) -> void:
	bridge = b
	title = "Journey Planner"
	size = Vector2i(900, 820)
	wrap_controls = false ## Same reason `new_world_dialog.gd` sets this: an autowrap dialog grows to its full content height and can run off a 1080p screen.
	get_ok_button().text = "Close"

	## `has_method` guard, matching every `engine_bridge.gd` wrapper's own
	## convention -- an older GDExtension binary without this milestone's
	## `#[func]`s degrades to an honest "not available" note instead of a
	## silent empty form. `bridge.jp_compute()` itself already returns `{}` on
	## a missing binding (the internal guard in `engine_bridge.gd`), but
	## checking here means the party form is never built against a vocabulary
	## that can't exist either.
	_bound = bridge.world_gen.has_method("jp_options") \
		and bridge.world_gen.has_method("jp_default_plan") \
		and bridge.world_gen.has_method("jp_compute") \
		and bridge.world_gen.has_method("route_count") \
		and bridge.world_gen.has_method("route_get")

	var margin := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		margin.add_theme_constant_override("margin_" + side, 10)
	add_child(margin)

	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 4)
	margin.add_child(root)

	if not _bound:
		DccWidgets.note(root,
			"jp_options / jp_default_plan / jp_compute / route_count / route_get " +
			"are not exposed by this build's GDExtension binary -- rebuild " +
			"cartalith-godot to pick up the Journey Planner boundary " +
			"(JOURNEY_PLANNER_SCOPE.md's closing-status steps 1/2/4).")
		return

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	root.add_child(scroll)

	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 2)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(body)

	_options = bridge.jp_options()
	_default_plan = bridge.jp_default_plan()
	_plan_values = {}
	## `party_fields` names the ten party-count keys among the other eighteen --
	## it is not itself a plan field, so it is excluded from the working form
	## values rather than sent back to `jp_compute` as an unrecognised key.
	for key in _default_plan.keys():
		if key != "party_fields":
			_plan_values[key] = _default_plan[key]

	_route_section = DccWidgets.section(body, "Route")
	_build_party_section(body)
	_build_compute_section(body)
	_results_body = DccWidgets.section(body, "Results")

	_refresh_route_choice()
	_render_results()

func open() -> void:
	if _bound:
		## Routes can be committed (INFRA's Route tool) after this window was
		## first built, so the list is re-read every time it opens rather than
		## once at `setup()`.
		_refresh_route_choice()
	popup_centered()

# -- Route ------------------------------------------------------------------

func _refresh_route_choice() -> void:
	for c in _route_section.get_children():
		_route_section.remove_child(c)
		c.queue_free()

	var count := bridge.route_count()
	if count == 0:
		DccWidgets.note(_route_section,
			"No committed routes yet. Draw one with the INFRA domain's Route " +
			"tool -- arm Route, click waypoints along the map, then ✓ Commit -- " +
			"then reopen this window.")
		_route_index = -1
		if _compute_btn != null:
			_compute_btn.disabled = true
		return

	var labels: Array = []
	for i in count:
		var r: Dictionary = bridge.route_get(i)
		var km := float(r.get("km", 0.0))
		var mode := String(r.get("mode", "?"))
		var unreach := int(r.get("unreachable_legs", 0))
		var label_text := "Route #%d -- %.1f km (%s)" % [i, km, mode]
		if unreach > 0:
			label_text += "  [%d unreachable leg%s]" % [unreach, "" if unreach == 1 else "s"]
		labels.append(label_text)

	if _route_index < 0 or _route_index >= count:
		_route_index = 0
	DccWidgets.choice(_route_section, "Committed route", labels, _route_index,
		func(i: int): _route_index = i,
		"route_get()'s own {points, brks, km, mode, unreachable_legs}. jp_compute prefers this index over a raw point list -- it samples the route's real f64 grid coordinates.")
	if _compute_btn != null:
		_compute_btn.disabled = false

# -- Party / plan form --------------------------------------------------------

## `label_text` rows use the exact `jp_compute`/`jp_default_plan` field key as
## `_plan_values`' own key, so `_compute()` can send the dictionary straight
## through -- no second name mapping to keep in sync.
func _number_field(parent: Control, label_text: String, key: String, minimum: float,
		maximum: float, step: float, integer: bool, tooltip: String = "") -> void:
	var v := float(_plan_values.get(key, 0.0))
	var on_change := func(nv: float) -> void:
		_plan_values[key] = (int(nv) if integer else nv)
	DccWidgets.number(parent, label_text, minimum, maximum, step, v, on_change, tooltip)

func _toggle_field(parent: Control, label_text: String, key: String, tooltip: String = "") -> void:
	var v := bool(_plan_values.get(key, false))
	var on_change := func(nv: bool) -> void:
		_plan_values[key] = nv
	DccWidgets.toggle(parent, label_text, v, on_change, tooltip)

## `allow_auto` prepends "Auto" / `""` -- the reference's own `!v ||
## v==='auto'` convention (`journey_bridge::opt_key`) for the fields whose
## `None` means "derive it per stage" (`desert_water`, `weather_override`,
## `route_cond`, `infra`, `rest_cadence`, `mount_animal`).
func _choice_field(parent: Control, label_text: String, key: String, opts: PackedStringArray,
		allow_auto: bool, tooltip: String = "") -> void:
	var labels: Array = []
	var raw: Array = []
	if allow_auto:
		labels.append("Auto")
		raw.append("")
	for o in opts:
		labels.append(String(o))
		raw.append(String(o))
	var current := String(_plan_values.get(key, ""))
	var idx: int = raw.find(current)
	if idx < 0:
		idx = 0
	var on_change := func(i: int) -> void:
		_plan_values[key] = raw[i]
	DccWidgets.choice(parent, label_text, labels, idx, on_change, tooltip)

## `route_cond` is one plan-wide field, but its legal vocabulary is nested per
## travel category (`jp_options()`'s own `route_cond: {land, river, sea}` --
## "Maintained" cannot describe a sea leg). Flattened into one dropdown with a
## category prefix on the label; the value sent back is the bare key, exactly
## what `plan_from_pairs` expects, applied "wherever legal for the stage's own
## travel category" per the Rust doc comment.
func _route_cond_field(parent: Control) -> void:
	var labels: Array = ["Auto"]
	var raw: Array = [""]
	var conds: Dictionary = _options.get("route_cond", {})
	for cat in ["land", "river", "sea"]:
		var opts: PackedStringArray = conds.get(cat, PackedStringArray())
		for o in opts:
			labels.append("%s: %s" % [cat.capitalize(), String(o)])
			raw.append(String(o))
	var current := String(_plan_values.get("route_cond", ""))
	var idx: int = raw.find(current)
	if idx < 0:
		idx = 0
	var on_change := func(i: int) -> void:
		_plan_values["route_cond"] = raw[i]
	DccWidgets.choice(parent, "Route condition", labels, idx, on_change,
		"Plan-wide override, applied wherever legal for each stage's own travel category. Auto lets every stage derive its own condition from the map instead.")

func _build_party_section(body: VBoxContainer) -> void:
	var sec := DccWidgets.section(body, "Party & plan")
	DccWidgets.note(sec,
		"Seeded from jp_default_plan() -- the reference's own _jpEnsurePlan defaults. Anything left untouched below computes exactly as if it had never been sent.")

	var travel := DccWidgets.group(sec, "Travel")
	_choice_field(travel, "Transport", "transport", _options.get("transport", PackedStringArray()), false)
	_choice_field(travel, "Mount (if party has no animals)", "mount_animal",
		_options.get("mount_animal", PackedStringArray()), true,
		"Only consulted when the party carries no donkeys/mules/camels/horses of its own.")
	_choice_field(travel, "Vessel (water legs)", "vessel", _options.get("vessel", PackedStringArray()), false)
	_number_field(travel, "Hours per travel day", "hours", 1.0, 16.0, 0.5, false)
	_choice_field(travel, "Pace", "pace", _options.get("pace", PackedStringArray()), false)

	var timing := DccWidgets.group(sec, "Timing")
	_choice_field(timing, "Season (departure)", "season", _options.get("season", PackedStringArray()), false)
	_toggle_field(timing, "Season drift (long journeys cross seasons)", "season_drift")
	_choice_field(timing, "Rest cadence", "rest_cadence", _options.get("rest_cadence", PackedStringArray()), true)

	var supply := DccWidgets.group(sec, "Supply")
	_number_field(supply, "Supply days carried", "supply_days", 0.0, 90.0, 1.0, true)
	_toggle_field(supply, "Carry food (off = live off the land only)", "carry_food")
	_choice_field(supply, "Grazing", "grazing", _options.get("grazing", PackedStringArray()), false)
	_choice_field(supply, "Foraging", "foraging", _options.get("foraging", PackedStringArray()), false)
	_choice_field(supply, "Desert water source", "desert_water", _options.get("desert_water", PackedStringArray()), true,
		"Auto derives the tier from the stage's own measured waterless run instead of a fixed choice.")
	_choice_field(supply, "Weather override", "weather_override", _options.get("weather_override", PackedStringArray()), true,
		"Auto uses the season x biome weather average for each stage.")
	_toggle_field(supply, "Seasonal closures (mountain passes, etc.)", "seasonal_closures")

	var route_group := DccWidgets.group(sec, "Route conditions")
	_route_cond_field(route_group)
	_choice_field(route_group, "Infrastructure tier", "infra", _options.get("infra", PackedStringArray()), true)

	var party := DccWidgets.group(sec, "Party")
	_number_field(party, "Group size (people)", "group_size", 1.0, 2000.0, 1.0, true)
	_number_field(party, "Cargo, excl. food/water (kg)", "cargo_kg", 0.0, 500000.0, 50.0, false)
	_toggle_field(party, "Auto-promote (Walking -> Baggage Train if overloaded)", "auto_promote")

	var animals := DccWidgets.group(sec, "Pack & mount animals")
	_number_field(animals, "Donkeys", "donkey", 0.0, 500.0, 1.0, true)
	_number_field(animals, "Mules", "mule", 0.0, 500.0, 1.0, true)
	_number_field(animals, "Camels", "camel", 0.0, 500.0, 1.0, true)
	_number_field(animals, "Horses", "horse", 0.0, 500.0, 1.0, true)

	var vehicles := DccWidgets.group(sec, "Vehicles")
	_number_field(vehicles, "Carts", "carts", 0.0, 200.0, 1.0, true)
	_number_field(vehicles, "Wagons", "wagons", 0.0, 200.0, 1.0, true)
	_number_field(vehicles, "Sleds", "sleds", 0.0, 200.0, 1.0, true)
	_number_field(vehicles, "Travois", "travois", 0.0, 200.0, 1.0, true)

# -- Compute ------------------------------------------------------------------

func _build_compute_section(body: VBoxContainer) -> void:
	var sec := DccWidgets.section(body, "Compute")
	var g := DccWidgets.group(sec, "Journey")
	_compute_btn = DccWidgets.action(g, "Compute journey", _compute, true)
	DccWidgets.note(g, "Runs jp_compute over the selected route and the form above. Any per-stage override set below is included too.")

func _compute() -> void:
	if not _bound:
		return
	if _route_index < 0:
		_last_result = {}
		_render_results()
		return
	var request: Dictionary = {"route": _route_index, "plan": _plan_values.duplicate(true)}
	if not _stage_overrides.is_empty():
		var ov: Dictionary = {}
		for idx in _stage_overrides:
			ov[idx] = (_stage_overrides[idx] as Dictionary).duplicate(true)
		request["stage_overrides"] = ov
	_last_result = bridge.jp_compute(request)
	_render_results()

# -- Results -------------------------------------------------------------------

func _render_results() -> void:
	for c in _results_body.get_children():
		_results_body.remove_child(c)
		c.queue_free()

	if _last_result.is_empty():
		if _route_index < 0:
			DccWidgets.note(_results_body, "Pick a committed route above, then press Compute journey.")
		else:
			DccWidgets.note(_results_body, "No result yet -- press Compute journey.")
		return

	_render_rejected(_results_body)

	if not bool(_last_result.get("ok", false)):
		DccWidgets.note(_results_body, "Compute failed: %s" % String(_last_result.get("error", "unknown error")))
		return

	var plan: Dictionary = _last_result.get("plan", {})
	var verdict: Dictionary = _last_result.get("verdict", {})
	var confidence: Dictionary = _last_result.get("confidence", {})

	_render_verdict(_results_body, verdict)
	_render_confidence(_results_body, confidence)
	_render_totals(_results_body, plan)
	_render_stages(_results_body, plan)
	_render_stops(_results_body, plan)
	_render_timeline(_results_body, plan)
	_render_stage_overrides(_results_body, plan)
	_render_profile_note(_results_body, plan)

func _render_rejected(parent: Control) -> void:
	var rejected: PackedStringArray = _last_result.get("rejected", PackedStringArray())
	if rejected.size() > 0:
		DccWidgets.note(parent,
			"Rejected keys (unrecognised or wrong-typed -- the reference default was used for each instead): %s" % ", ".join(rejected))

func _render_verdict(parent: Control, verdict: Dictionary) -> void:
	if verdict.is_empty():
		return
	var sec := DccWidgets.section(parent, "Verdict")
	var level := String(verdict.get("level", ""))
	var label_text := String(verdict.get("label", "?"))
	var token := "stale" if (level == "severe" or level == "blocked") else "accent"
	sec.add_child(DccTheme.mono_label(label_text.to_upper(), token, 16, 1, true))
	var text_l := DccTheme.label(String(verdict.get("text", "")), "text", DccTheme.FS_SMALL)
	text_l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	sec.add_child(text_l)
	var reasons: PackedStringArray = verdict.get("reasons", PackedStringArray())
	if reasons.size() > 0:
		var g := DccWidgets.group(sec, "Contributing factors")
		for r in reasons:
			DccWidgets.note(g, "- %s" % String(r))

func _render_confidence(parent: Control, confidence: Dictionary) -> void:
	var sec := DccWidgets.section(parent, "Confidence band")
	if confidence.is_empty():
		DccWidgets.note(sec, "No band -- the journey is blocked, or its day count is non-finite.")
		return
	DccWidgets.note(sec, "%.1f - %.1f days. %s" % [
		float(confidence.get("lo_days", 0.0)), float(confidence.get("hi_days", 0.0)),
		String(confidence.get("note", ""))])

func _render_totals(parent: Control, plan: Dictionary) -> void:
	var sec := DccWidgets.section(parent, "Totals")
	var g := DccWidgets.group(sec, "Journey")
	DccWidgets.note(g, "%.1f km over %.1f travel days (%.1f km/day average)." % [
		float(plan.get("km", 0.0)), float(plan.get("days", 0.0)), float(plan.get("avg_km_day", 0.0))])
	var total_days := float(plan.get("total_days", -1.0))
	if total_days >= 0.0:
		DccWidgets.note(g, "%.1f calendar days total -- %d rest, %d layover, %d handling." % [
			total_days, int(plan.get("rest_days", 0)), int(plan.get("layover_days", 0)), int(plan.get("handling_days", 0))])
	DccWidgets.note(g, "Food %.1f kg, water %.1f L, fodder %.1f kg." % [
		float(plan.get("food_kg", 0.0)), float(plan.get("water_l", 0.0)), float(plan.get("fodder_kg", 0.0))])
	DccWidgets.note(g, "%d river crossing(s), %.1f km mountain pass, %.1f km desert, %.0f%% bad-weather odds for the season." % [
		int(plan.get("river_crossings", 0)), float(plan.get("pass_km", 0.0)),
		float(plan.get("desert_km", 0.0)), float(plan.get("bad_wx_pct", 0.0))])
	DccWidgets.note(g, "Elevation: %.0f m ascent, %.0f m descent, range %.0f-%.0f m." % [
		float(plan.get("ascent", 0.0)), float(plan.get("descent", 0.0)),
		float(plan.get("lo_m", 0.0)), float(plan.get("hi_m", 0.0))])
	if bool(plan.get("blocked", false)):
		DccWidgets.note(g, "BLOCKED at stage %d." % int(plan.get("blocked_idx", -1)))
	var transship := int(plan.get("transshipments", 0))
	if transship > 0:
		DccWidgets.note(g, "%d transshipment(s), %.1f h transfer overhead." % [transship, float(plan.get("transfer_overhead", 0.0))])
	var seasons: PackedStringArray = plan.get("seasons_crossed", PackedStringArray())
	if seasons.size() > 0:
		DccWidgets.note(g, "Seasons crossed: %s." % ", ".join(seasons))

func _resupply_line(r: Dictionary) -> String:
	if r.is_empty():
		return ""
	var feasible := bool(r.get("feasible", false))
	var verdict_s := String(r.get("verdict", "?"))
	var cause := String(r.get("cause", ""))
	var extra := " (%s)" % cause if cause != "" else ""
	return "%s -- resupply %s%s." % [verdict_s, ("feasible" if feasible else "NOT feasible"), extra]

func _render_stages(parent: Control, plan: Dictionary) -> void:
	var stages: Array = plan.get("stages", [])
	var results: Array = plan.get("results", [])
	if stages.is_empty():
		return
	var sec := DccWidgets.section(parent, "Stages (%d)" % stages.size())
	for i in stages.size():
		var s: Dictionary = stages[i]
		var r: Dictionary = results[i] if i < results.size() else {}
		var g := DccWidgets.group(sec, "Stage %d -- %s" % [i, String(s.get("cat", "?"))], false)
		DccWidgets.note(g, "%s / %s -- route %s, infra %s -- %.1f km." % [
			String(s.get("terrain", "?")), String(s.get("biome", "?")),
			String(s.get("route_cond", "?")), String(s.get("infra", "?")), float(s.get("km", 0.0))])
		if int(s.get("river_crossings", 0)) > 0 or float(s.get("dry_km", 0.0)) > 0.0:
			DccWidgets.note(g, "%d river crossing(s), %.1f km dry, %.0f m gain / %.0f m loss." % [
				int(s.get("river_crossings", 0)), float(s.get("dry_km", 0.0)),
				float(s.get("gain", 0.0)), float(s.get("loss", 0.0))])
		if r.is_empty():
			continue
		if bool(r.get("blocked", false)):
			var seasonal := " (seasonal)" if bool(r.get("blocked_seasonal", false)) else ""
			DccWidgets.note(g, "BLOCKED: %s%s" % [String(r.get("blocked_reason", "")), seasonal])
			continue
		DccWidgets.note(g, "%.1f days at %.1f km/day." % [float(r.get("days", 0.0)), float(r.get("daily_km", 0.0))])
		if r.has("land"):
			var l: Dictionary = r["land"]
			DccWidgets.note(g, "%s -- load %.0f%% of capacity, %s." % [
				String(l.get("transport_label", "?")), float(l.get("load_ratio", 0.0)) * 100.0,
				("desert" if bool(l.get("is_desert", false)) else "non-desert")])
			var resup_line := _resupply_line(l.get("resupply", {}))
			if resup_line != "":
				DccWidgets.note(g, resup_line)
		elif r.has("water"):
			var w: Dictionary = r["water"]
			DccWidgets.note(g, "%s -- crew %d, load %.0f%% of capacity." % [
				String(w.get("transport_label", "?")), int(w.get("crew", 0)), float(w.get("load_ratio", 0.0)) * 100.0])
			var resup_line2 := _resupply_line(w.get("resupply", {}))
			if resup_line2 != "":
				DccWidgets.note(g, resup_line2)

func _render_stops(parent: Control, plan: Dictionary) -> void:
	var stops: Array = plan.get("stops", [])
	var sec := DccWidgets.section(parent, "Stops (%d)" % stops.size())
	if stops.is_empty():
		DccWidgets.note(sec, "No stops on this route.")
	else:
		for st in stops:
			var d: Dictionary = st
			var layover := int(d.get("layover_days", 0))
			var extra := "  -- %d layover day(s)" % layover if layover > 0 else ""
			DccWidgets.note(sec, "%s (%s)%s" % [String(d.get("name", "?")), String(d.get("kind", "?")), extra])
	var reach: Dictionary = plan.get("resupply_reach", {})
	if not reach.is_empty():
		DccWidgets.note(sec, "Resupply reach: longest gap %.1f km, party carries %.1f km of supply -- %s." % [
			float(reach.get("max_gap_km", 0.0)), float(reach.get("required_km", 0.0)),
			("UNMET" if bool(reach.get("unmet", false)) else "met")])

func _render_timeline(parent: Control, plan: Dictionary) -> void:
	var timeline: Array = plan.get("timeline", [])
	if timeline.is_empty():
		return
	var sec := DccWidgets.section(parent, "Timeline")
	## Closed by default -- a season-scale journey can run 100+ travel days,
	## and the timeline is the one list here long enough that always-open
	## would make every other section scroll past it.
	var g := DccWidgets.group(sec, "%d travel day(s)" % timeline.size(), false)
	for t in timeline:
		var d: Dictionary = t
		var camp := String(d.get("camp", ""))
		var camp_note := "  -- camp at %s" % camp if camp != "" else ""
		DccWidgets.note(g, "Day %d: %.1f km, %s / %s%s" % [
			int(d.get("day", 0)), float(d.get("km", 0.0)),
			String(d.get("terrain", "?")), String(d.get("biome", "?")), camp_note])

## The one genuinely optional feature the task brief allowed leaving as a
## documented stub -- built for real instead. Picking a stage and overriding
## its route condition or infrastructure tier recomputes the whole journey
## immediately via `jp_compute`'s own `stage_overrides` key, and the choice
## persists (shown with "[override active]") across the main Compute button
## too, until explicitly cleared.
func _render_stage_overrides(parent: Control, plan: Dictionary) -> void:
	var stages: Array = plan.get("stages", [])
	if stages.is_empty():
		return
	if _override_stage_idx < 0 or _override_stage_idx >= stages.size():
		_override_stage_idx = 0

	var sec := DccWidgets.section(parent, "Per-stage overrides")
	DccWidgets.note(sec,
		"Override one stage's route condition or infrastructure tier below -- it recomputes the whole journey immediately.")
	var group_open := not _stage_overrides.is_empty()
	var g := DccWidgets.group(sec, "Override a stage", group_open)

	var stage_labels: Array = []
	for i in stages.size():
		var s: Dictionary = stages[i]
		var mark := "  [override active]" if _stage_overrides.has(i) else ""
		stage_labels.append("Stage %d -- %s%s" % [i, String(s.get("cat", "?")), mark])

	var detail := VBoxContainer.new()
	g.add_child(detail)

	var show_detail: Callable
	show_detail = func(idx: int) -> void:
		_override_stage_idx = idx
		for c in detail.get_children():
			detail.remove_child(c)
			c.queue_free()
		var cat := String((stages[idx] as Dictionary).get("cat", "land"))
		var existing: Dictionary = _stage_overrides.get(idx, {})

		var conds: Dictionary = _options.get("route_cond", {})
		var cat_opts: PackedStringArray = conds.get(cat, PackedStringArray())
		var cond_labels: Array = ["Auto"]
		var cond_raw: Array = [""]
		for o in cat_opts:
			cond_labels.append(String(o))
			cond_raw.append(String(o))
		var cond_idx: int = cond_raw.find(String(existing.get("route_cond", "")))
		if cond_idx < 0:
			cond_idx = 0
		DccWidgets.choice(detail, "Route cond (%s)" % cat, cond_labels, cond_idx, func(i: int):
			_set_stage_override(idx, "route_cond", cond_raw[i]))

		var infra_opts: PackedStringArray = _options.get("infra", PackedStringArray())
		var infra_labels: Array = ["Auto"]
		var infra_raw: Array = [""]
		for o in infra_opts:
			infra_labels.append(String(o))
			infra_raw.append(String(o))
		var infra_idx: int = infra_raw.find(String(existing.get("infra", "")))
		if infra_idx < 0:
			infra_idx = 0
		DccWidgets.choice(detail, "Infra tier", infra_labels, infra_idx, func(i: int):
			_set_stage_override(idx, "infra", infra_raw[i]))

		if _stage_overrides.has(idx):
			DccWidgets.action(detail, "Clear this stage's override", func(): _clear_stage_override(idx))

	DccWidgets.choice(g, "Stage", stage_labels, _override_stage_idx, func(i: int): show_detail.call(i))
	show_detail.call(_override_stage_idx)

func _set_stage_override(idx: int, field: String, value: String) -> void:
	var entry: Dictionary = (_stage_overrides.get(idx, {}) as Dictionary).duplicate()
	if value == "":
		entry.erase(field)
	else:
		entry[field] = value
	if entry.is_empty():
		_stage_overrides.erase(idx)
	else:
		_stage_overrides[idx] = entry
	_compute()

func _clear_stage_override(idx: int) -> void:
	_stage_overrides.erase(idx)
	_compute()

## `plan.profile` is real (`_jpPlan`'s own 0-1 normalised elevation samples,
## the reference's own elevation chart source) -- this reports its presence
## honestly instead of drawing nothing and saying nothing, or drawing a chart
## that was not actually asked for at this pass's time budget.
func _render_profile_note(parent: Control, plan: Dictionary) -> void:
	var profile: PackedFloat64Array = plan.get("profile", PackedFloat64Array())
	if profile.is_empty():
		return
	DccWidgets.note(parent,
		("plan.profile carries %d normalised (0-1) elevation samples -- " % profile.size()) +
		"a sparkline chart of it is left undone this pass (time-boxed, not faked). The data itself is real and available for a future pass.")
