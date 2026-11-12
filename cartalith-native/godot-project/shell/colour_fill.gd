extends RefCounted
## CARTO ▸ Colours ▸ **Fill an area** -- the colour-paint fill helper (`COLOUR_PAINT_SCOPE.md` §13, milestone CP-5).
##
## What it does: arms the tap tool `colour_fill`. A tap inside a crater or basin asks the engine for that
## depression's outline (`bridge.colour_paint_fill_preview`: a closed fill to the pour point, or a capped fill when the
## crater is breached) and shows it as a preview ring on the map. Level, Cap and Grow reshape it (every move asks the
## engine again); Commit paints it with the current brush as ONE `Area` mark plus an optional hard grow stroke along
## its ring, as one step on the paint tool's own undo stack (`colour_paint_tool.gd::commit_marks`). Discard (or Esc, or
## arming another tool, or a world change) drops the preview and writes nothing.
##
## Why it is click-seeded: the scope measured that an automatic crater detector took a river gorge's rimmed pits for a
## crater (§13), so the user says which depression, and the engine only outlines it.
##
## Why it is a tap tool (`DccApp.tool_kind` PLACING): on a touch map a one-finger drag then pans, and only a tap
## acts, so a thumb can move the map and seed a fill without a pan latch (Ruling EC; `MOBILE_SHELL_PROPOSAL.md` §2.5).
##
## What it must never do: write the engine except through `commit_marks` (the preview is read-only), call the engine
## while `bridge.generating`, leave a preview drawn after the tool is disarmed, or invent an outline: when the engine
## says no ring, the preview is cleared and the engine's own sentence is shown.

const TOOL_ID := "colour_fill"

## The level slider's range, metres either side of the automatic level, and the step the strip's -/+ takes.
## Labelled judgements: a crater rim sits within a few tens of metres of the capped fill's `rim - 0.01` level on
## the scope's worlds (§13), so +-200 m spans from the floor to well past the rim.
const LEVEL_MAX_M := 200.0
const LEVEL_STEP_M := 10.0
## The cap slider's range in km (the engine refuses past 60 cells; 40 km is 20 cells at the app's 800 km / 2048 grid),
## the strip's ratio per press, and the grow slider's range.
const CAP_MIN_KM := 0.5
const CAP_MAX_KM := 40.0
const GROW_MAX_KM := 5.0

var cp  ## the owning `ColourPaintTool`
var level_m := 0.0
## `null` = the engine's automatic cap, never a number standing for it.
var cap_km: Variant = null

## The grid cell the last tap fell in, or (-1, -1).
var _cell := Vector2i(-1, -1)
## The engine's last good reply, or `{}`.
var _preview: Dictionary = {}
## The engine's sentence for the last refusal, or "".
var _error := ""
## Whether this object put a ring on the overlay (so `cancel` clears only what it drew).
var _drawn := false

var _status: Label
var _level_s: Dictionary = {}
var _cap_s: Dictionary = {}
var _grow_s: Dictionary = {}
var _commit_btn: Button
var _discard_btn: Button

func _init(owner_tool) -> void:
	cp = owner_tool

# -- The dock block ----------------------------------------------------------------

## Builds the FILL AN AREA block into `body` (the CARTO ▸ Colours section). Without the engine binding it draws one
## note and nothing else.
func build_into(body: Control) -> void:
	body.add_child(DccTheme.label("FILL AN AREA", "text_dim", DccTheme.FS_SMALL))
	if not cp.bridge.colour_paint_fill_api:
		DccWidgets.note(body, "The fill helper is not available in this version of Cartalith.")
		return
	var arm := DccWidgets.action(body, "Fill an area", func(): cp.app.arm_tool(TOOL_ID))
	arm.tooltip_text = "Arms the fill helper: tap inside a crater or a basin to outline it, adjust the outline, then Commit."
	arm.custom_minimum_size = Vector2(maxf(arm.custom_minimum_size.x, _tap()), maxf(arm.custom_minimum_size.y, _tap()))
	DccWidgets.note(body, "Tap inside a crater or basin. The outline is previewed; Level, Cap and Grow reshape it; Commit paints it with the brush above as one fill, one undo step.")
	_level_s = DccWidgets.slider(body, "Level", -LEVEL_MAX_M, LEVEL_MAX_M, 5.0, level_m, "m", func(v: float):
		level_m = v
		_run(),
		"Metres above (or below) the automatic fill level: higher fills more of the bowl, lower less.")
	_cap_s = DccWidgets.slider(body, "Cap", CAP_MIN_KM, CAP_MAX_KM, 0.5, 5.0, "km", func(v: float):
		cap_km = v
		_run(),
		"The widest radius a breached crater's fill may reach: the cap closes the breach with an arc instead of a lobe down the gorge. Automatic until moved.")
	var auto := DccWidgets.action(body, "Automatic cap", func():
		cap_km = null
		_run())
	auto.tooltip_text = "Goes back to the cap the engine chose from the crater's own rim."
	auto.custom_minimum_size = Vector2(maxf(auto.custom_minimum_size.x, _tap()), maxf(auto.custom_minimum_size.y, _tap()))
	_grow_s = DccWidgets.slider(body, "Grow", 0.0, GROW_MAX_KM, 0.1, cp.fill_grow_km, "km", func(v: float): cp.fill_grow_km = v,
		"A hard stroke of this radius along the outline, so the walls up to the rim are painted too. 0 paints the fill alone.")
	DccWidgets.toggle(body, "Land only", cp.fill_land_only, func(v: bool): cp.fill_land_only = v,
		"Off (the default for a fill): a crater lake inside the outline is painted too. On: water keeps its colour.")
	var row := HFlowContainer.new()
	row.add_theme_constant_override("h_separation", 6)
	row.add_theme_constant_override("v_separation", 6)
	body.add_child(row)
	_commit_btn = DccWidgets.action(row, "Commit fill", func(): commit(), true)
	_discard_btn = DccWidgets.action(row, "Discard", func(): discard())
	for b in [_commit_btn, _discard_btn]:
		(b as Button).custom_minimum_size = Vector2(maxf((b as Button).custom_minimum_size.x, _tap()), maxf((b as Button).custom_minimum_size.y, _tap()))
	_status = DccWidgets.note(body, "")
	refresh_ui()

## The tap target on this form factor, both axes (`colour_paint_tool.gd::_tap`'s rule).
static func _tap() -> float:
	return 44.0 if DccTheme.is_touch() else 28.0

## Brings the status line, the Commit / Discard buttons and the sliders up to date with the preview.
func refresh_ui() -> void:
	var has := not _preview.is_empty()
	if _commit_btn != null and is_instance_valid(_commit_btn):
		_commit_btn.disabled = not has
	if _discard_btn != null and is_instance_valid(_discard_btn):
		_discard_btn.disabled = not has and _error == ""
	if has and cap_km == null and not _cap_s.is_empty() and is_instance_valid(_cap_s["slider"]):
		## Automatic cap: the slider shows what the engine chose, without taking it as a choice.
		var a := clampf(float(_preview.get("auto_cap_km", 5.0)), CAP_MIN_KM, CAP_MAX_KM)
		(_cap_s["slider"] as HSlider).set_value_no_signal(a)
		(_cap_s["readout"] as Label).text = "auto"
	if _status == null or not is_instance_valid(_status):
		return
	if has:
		var how := "Closed fill to the pour point" if String(_preview.get("mode", "")) == "pour_point" else "Capped fill (the crater is breached)"
		_status.text = "%s: %s, %d cells. Commit paints it." % [how, DccUnits.format_area(float(_preview.get("area_km2", 0.0))), int(_preview.get("cells", 0))]
	elif _error != "":
		_status.text = "Not outlined: %s" % _error
	else:
		_status.text = "Tap inside a crater or basin on the map."

# -- Click, preview, commit --------------------------------------------------------

## A tap on the map with the fill tool armed: seeds the outline there. The level and the cap start automatic for
## every new tap, so a second crater never inherits the first one's cap.
func on_click(gx: float, gy: float) -> void:
	if cp.bridge == null or cp.bridge.generating:
		return
	if cp.km_per_cell() <= 0.0:
		cp.app.set_status("hint", "Not done: colour paint needs a world with a map width.", "warn")
		return
	_cell = Vector2i(floori(gx), floori(gy))
	level_m = 0.0
	cap_km = null
	if not _level_s.is_empty() and is_instance_valid(_level_s["slider"]):
		(_level_s["slider"] as HSlider).set_value_no_signal(0.0)
		(_level_s["readout"] as Label).text = (_level_s["format"] as Callable).call(0.0)
	_run()

## Asks the engine for the outline at the last tap with the current level and cap and shows it (or the engine's
## reason it cannot).
func _run() -> void:
	if _cell.x < 0 or cp.bridge == null:
		return
	var opts := {"level_m": level_m}
	if cap_km != null:
		opts["cap_km"] = float(cap_km)
	var r: Dictionary = cp.bridge.colour_paint_fill_preview(_cell.x, _cell.y, opts)
	if bool(r.get("ok", false)):
		_preview = r
		_error = ""
	else:
		_preview = {}
		_error = String(r.get("error", "the engine is busy")) if not r.is_empty() else "the world is generating"
	_draw()
	refresh_ui()

## The preview ring on the map: the engine's km ring over `km_per_cell`, closed by repeating the first point.
func _draw() -> void:
	var ov = _overlay()
	if ov == null:
		return
	var kpc: float = cp.km_per_cell()
	var pts := PackedVector2Array()
	if not _preview.is_empty() and kpc > 0.0:
		for p in (_preview["ring_km"] as PackedVector2Array):
			pts.append(p / kpc)
		if pts.size() > 0:
			pts.append(pts[0])
	if pts.size() > 0 or _drawn:
		ov.set_path_preview(pts)
	_drawn = pts.size() > 0

func _overlay():
	var app = cp.app
	if app == null or app.viewport == null or app.viewport.tool_overlay == null:
		return null
	return app.viewport.tool_overlay

## Paints the previewed outline: an `Area` mark with the current brush, plus a hard grow stroke along its ring when
## Grow is above 0, as ONE undo step. Returns whether it painted. Nothing previewed paints nothing.
func commit() -> bool:
	if _preview.is_empty() or cp.bridge == null or cp.bridge.generating:
		return false
	var pts: Array = []
	for p in (_preview["ring_km"] as PackedVector2Array):
		pts.append([snappedf(p.x, 0.001), snappedf(p.y, 0.001)])
	var marks: Array = [cp.brush_mark({"kind": "area", "ring_km": pts, "edge_km": 0.0}, 1.0, cp.fill_land_only)]
	if cp.fill_grow_km > 0.0:
		var closed := pts.duplicate()
		closed.append(pts[0])
		marks.append(cp.brush_mark({"kind": "line", "points_km": closed, "radius_km": snappedf(cp.fill_grow_km, 0.01)}, 1.0, cp.fill_land_only))
	var label := "Fill" if cp.material == "colour" else "%s fill" % String(cp.MATERIALS[cp._material_index()]["label"])
	if not cp.commit_marks(marks, label):
		return false
	cancel()
	refresh_ui()
	return true

## Drops the preview and clears the ring this object drew; writes nothing. Safe at any time (a tool change, a world
## change, Esc, Discard).
func cancel() -> void:
	_preview = {}
	_error = ""
	_cell = Vector2i(-1, -1)
	if _drawn:
		var ov = _overlay()
		if ov != null:
			ov.set_path_preview(PackedVector2Array())
		_drawn = false

## The Discard button: `cancel` plus the status line and buttons redrawn.
func discard() -> void:
	cancel()
	refresh_ui()

## Esc with the fill tool armed: the first press discards a preview, the next puts the tool down.
func on_escape() -> void:
	if not _preview.is_empty() or _error != "":
		discard()
		return
	if cp.app != null:
		cp.app.arm_tool("inspect")

## Whether a preview is held (a probe seam, and what `verbs` reports).
func has_preview() -> bool:
	return not _preview.is_empty()

## The draft verbs the phone's tool strip draws as Commit / Discard (`DccApp`'s `draft_verbs` contract).
func verbs() -> Dictionary:
	return {"commit": commit, "discard": discard, "can": not _preview.is_empty(), "count": 1 if not _preview.is_empty() else 0}

## The strip's `"tool"` rows for the armed fill tool (see `colour_paint_tool.gd::tool_rows`).
func tool_rows() -> Array:
	return [
		{"id": "carto.cf_level", "label": "Level", "section": "tool", "enabled": true, "param": {
			"value": func() -> String: return "%+d m" % int(round(level_m)),
			"step": func(dir: int) -> void: _step_level(dir)}},
		{"id": "carto.cf_cap", "label": "Cap", "section": "tool", "enabled": true, "param": {
			"value": func() -> String: return "auto" if cap_km == null else "%.1f km" % float(cap_km),
			"step": func(dir: int) -> void: _step_cap(dir)}},
		{"id": "carto.cf_grow", "label": "Grow", "section": "tool", "enabled": true, "param": {
			"value": func() -> String: return "none" if cp.fill_grow_km <= 0.0 else "%.1f km" % cp.fill_grow_km,
			"step": func(dir: int) -> void: cp.fill_grow_km = clampf(snappedf(cp.fill_grow_km + 0.5 * dir, 0.1), 0.0, GROW_MAX_KM)}},
		{"id": "carto.cf_land", "label": "Land only", "section": "tool", "enabled": true, "param": {
			"value": func() -> String: return "on" if cp.fill_land_only else "off",
			"step": func(_dir: int) -> void: cp.fill_land_only = not cp.fill_land_only}},
	]

func _step_level(dir: int) -> void:
	level_m = clampf(level_m + LEVEL_STEP_M * dir, -LEVEL_MAX_M, LEVEL_MAX_M)
	if not _level_s.is_empty() and is_instance_valid(_level_s["slider"]):
		(_level_s["slider"] as HSlider).set_value_no_signal(level_m)
		(_level_s["readout"] as Label).text = (_level_s["format"] as Callable).call(level_m)
	_run()

## Steps the cap by x1.25 per press from the engine's automatic cap (or the last chosen one), clamped to the
## slider's range. With no preview to read the automatic cap from, the press does nothing rather than guess one.
func _step_cap(dir: int) -> void:
	var base: Variant = cap_km if cap_km != null else (float(_preview["auto_cap_km"]) if _preview.has("auto_cap_km") else null)
	if base == null:
		return
	cap_km = clampf(snappedf(float(base) * pow(1.25, float(dir)), 0.1), CAP_MIN_KM, CAP_MAX_KM)
	if not _cap_s.is_empty() and is_instance_valid(_cap_s["slider"]):
		(_cap_s["slider"] as HSlider).set_value_no_signal(float(cap_km))
		(_cap_s["readout"] as Label).text = (_cap_s["format"] as Callable).call(float(cap_km))
	_run()
