extends RefCounted
## CARTO ▸ Colours ▸ **Paint colour** -- Ruling CN's non-destructive colour
## paint (`COLOUR_PAINT_SCOPE.md` CP-2) and its lava material (CP-1M, §18).
##
## What it does: arms the `colour_paint` map tool; a press-drag-release on the
## map is one stroke, sampled into a polyline in **map kilometres** (the
## engine's convention: origin at the map's top-left edge, `km = g * km_per_cell`
## for the overlay's edge-origin grid coordinates), simplified, and appended to
## the engine's layer as one mark. Material (flat colour or a glow material),
## colour, blend, radius in km, hardness, opacity and land-only are this
## object's brush state; hide and clear act on the layer.
##
## Why it is its own object: the layer is presentation, not world data
## (`DCC_SHELL_SPEC.md` §4.5.2 keeps WORLD paint in WORLD), so it has its own
## undo stack -- undoing paint never undoes a look change or a sculpt, and the
## reverse (`COLOUR_PAINT_SCOPE.md` §8). A step is the whole document before
## the change (marks are small; §9 measures 5 000 x 40-point marks at 3.98 MB),
## and the stack drops its oldest steps first when it would outgrow what the
## app's undo budget has left (`undo_stats()`'s `budget_bytes - bytes`).
##
## What it must never do: call the engine while `bridge.generating` (every
## bridge call here is gated there and returns "nothing"); invent a scale (with
## no `km_per_cell` the tool says so and paints nothing); or leave a stroke open
## past a release, a tool change or a world change.

## The tool id `app.arm_tool` takes and the click/drag/release handlers key on.
const TOOL_ID := "colour_paint"

## The materials offered, in picker order. `colour` is the flat colour CP-1
## shipped; the others are `EmissiveMaterial` presets -- the same mechanism with
## a different ramp or heat. `heat`, `crack` (density) and `glow` (halo radius,
## km) are the preset's defaults; the sliders below move them per stroke.
## Labelled judgements, chosen on the 2026-10-18 lava frames
## (`cartalith-native/docs/evidence/colour-paint-lava-2026-10-18/`).
const MATERIALS := [
	{"key": "colour", "label": "Colour"},
	{"key": "lava", "label": "Lava", "ramp": "fire", "heat": 0.75, "crack": 0.5, "glow": 1.5, "crust": Color8(34, 33, 38)},
	{"key": "ember", "label": "Embers", "ramp": "fire", "heat": 0.35, "crack": 0.8, "glow": 0.8, "crust": Color8(40, 36, 34)},
	{"key": "ice", "label": "Ice glow", "ramp": "ice", "heat": 0.75, "crack": 0.5, "glow": 1.5, "crust": Color8(30, 36, 44)},
	{"key": "toxic", "label": "Toxic glow", "ramp": "toxic", "heat": 0.75, "crack": 0.5, "glow": 1.5, "crust": Color8(30, 36, 28)},
]

## Blend modes, in the engine's keys (`ColourBlend::key`). Replace first: the
## scope's measured default (§3) -- the only mode in which a picked red reads
## as that red on every ground; the owner has not ruled on the default.
const BLENDS := [
	{"key": "replace", "label": "Replace"},
	{"key": "colour", "label": "Tint (colour)"},
	{"key": "multiply", "label": "Multiply"},
]

## Swatches: `COLOUR_PAINT_SCOPE.md` §15's list, verbatim.
const SWATCHES := [
	["Lava red", Color8(200, 30, 18)], ["Ash grey", Color8(120, 118, 112)],
	["Sulphur", Color8(220, 200, 60)], ["Sea blue", Color8(40, 90, 150)],
	["Grass", Color8(86, 140, 58)], ["Sand", Color8(210, 180, 120)],
	["Ochre", Color8(225, 160, 110)], ["Snow", Color8(240, 240, 236)],
]

## A stroke sample is kept once it is this fraction of the brush radius from
## the last kept one. Labelled judgement: dense enough that a fast curve stays
## round, sparse enough that a long stroke stays far under the engine's 4 096
## points per mark.
const SPACING := 0.25
## The simplification tolerance (Ramer-Douglas-Peucker) as a fraction of the
## radius: a point closer than this to the chord is dropped. Labelled judgement:
## a tenth of the radius moves the stroke's edge by less than that.
const SIMPLIFY := 0.1
## The engine's per-mark point cap (`colour_paint::MAX_POINTS_PER_MARK`),
## restated: a stroke is resampled coarser rather than refused at this length.
const MAX_POINTS := 4096

var app: Node
var bridge: Node

## Brush state. `rgb` is the flat colour, or the crust for a material.
var material := "colour"
var rgb := Color8(200, 30, 18)
var blend := "replace"
var radius_km := 1.0
var hardness := 0.8
var opacity := 1.0
var land_only := true
var heat := 0.75
var crack := 0.5
var glow_km := 1.5
var heat_along := 0.0

## The stroke being drawn, in km, and its grid-cell copy for the preview line.
var _pts_km: Array = []
var _pts_grid := PackedVector2Array()
var _open := false

## Undo/redo: whole documents ("" = no layer), newest last, with labels.
var _undo: Array = []
var _redo: Array = []

## The section's live widgets, for `_refresh_ui`.
var _status: Label
var _undo_btn: Button
var _redo_btn: Button
var _hide_btn: CheckBox
var _mat_rows: Array = []
var _colour_row: Control
var _pick: ColorPickerButton

## Wires the three map handlers once. `app` must be the DccApp (it owns the tool
## dispatch: `register_tool_*_handler`).
func setup(app_: Node, bridge_: Node) -> void:
	app = app_
	bridge = bridge_
	if app != null and app.has_method("register_tool_click_handler"):
		app.register_tool_click_handler(TOOL_ID, _on_press)
		app.register_tool_drag_handler(TOOL_ID, _on_drag)
		app.register_tool_release_handler(TOOL_ID, _on_release)
	if app != null and app.has_signal("tool_armed"):
		app.tool_armed.connect(func(id: String):
			if id != TOOL_ID:
				_cancel_stroke())

## Kilometres per grid cell, or a negative number when the world has none (no
## binding, generating, no map width). Never a guessed scale.
func km_per_cell() -> float:
	var info: Dictionary = bridge.colour_paint_info() if bridge != null else {}
	return float(info.get("km_per_cell", -1.0)) if info.has("km_per_cell") else -1.0

# -- The section -----------------------------------------------------------------

## Builds the Paint colour section into `parent` (the CARTO ▸ Colours body).
## Without the binding it says so in one note and builds nothing else.
func build_into(parent: Control) -> void:
	var body := DccWidgets.section(parent, "Paint colour")
	if bridge == null or not bridge.colour_paint_api:
		DccWidgets.note(body, "This build's engine has no colour-paint binding (colour_paint_info), so the tool cannot paint.")
		return
	var arm := DccWidgets.action(body, "Paint on the map", func(): app.arm_tool(TOOL_ID), true)
	arm.tooltip_text = "Arms the colour brush: press and drag on the map to paint one stroke. Paint is presentation only -- it never changes the world."
	## Material first and prominent: the owner's lava case is a material, not a colour.
	var mats: Array = []
	for m in MATERIALS:
		mats.append(String(m["label"]))
	DccWidgets.choice(body, "Material", mats, _material_index(), func(i: int): _set_material(i),
		"Colour paints a flat colour. Lava, Embers, Ice glow and Toxic glow paint a self-lit surface: a dark crust broken by glowing cracks, hottest inside, with a soft halo; the light and shadows never darken the glow.")
	## A flow, not a box: nine swatches at the touch floor do not fit a dock's
	## width in one row (`MISTAKES.md`: a row that cannot shrink widens every
	## ancestor), so they wrap. Touch takes the 44 px target on both axes.
	var touch := DccTheme.is_touch()
	var sw := 44.0 if touch else 22.0
	_colour_row = HFlowContainer.new()
	(_colour_row as HFlowContainer).add_theme_constant_override("h_separation", 6)
	(_colour_row as HFlowContainer).add_theme_constant_override("v_separation", 6)
	body.add_child(_colour_row)
	_pick = ColorPickerButton.new()
	_pick.edit_alpha = false
	_pick.color = rgb
	_pick.custom_minimum_size = Vector2(44, 44 if touch else 28)
	_pick.tooltip_text = "The colour painted (for a glow material: the cooled crust's colour)."
	_pick.color_changed.connect(func(c: Color): rgb = c)
	DccWidgets.confirmable_color(_pick, func(c: Color): rgb = c)
	_colour_row.add_child(_pick)
	for s in SWATCHES:
		var b := Button.new()
		b.custom_minimum_size = Vector2(sw, sw)
		b.tooltip_text = String(s[0])
		var sb := StyleBoxFlat.new()
		sb.bg_color = s[1]
		for st in ["normal", "hover", "pressed", "focus"]:
			b.add_theme_stylebox_override(st, sb)
		var c: Color = s[1]
		b.pressed.connect(func():
			rgb = c
			_pick.color = c)
		_colour_row.add_child(b)
	var bl: Array = []
	for b2 in BLENDS:
		bl.append(String(b2["label"]))
	DccWidgets.choice(body, "Blend", bl, _blend_index(), func(i: int): blend = String(BLENDS[i]["key"]),
		"Replace: the colour itself, shaded by the relief. Tint: the colour's hue at the ground's own lightness (keeps texture; cannot make snow deep red). Multiply: a stain that only darkens.")
	DccWidgets.slider(body, "Radius", 0.05, 25.0, 0.05, radius_km, "km", func(v: float):
		radius_km = v
		_refresh_ui(),
		"Brush radius in map kilometres: the same ground at every zoom and in the export.")
	DccWidgets.slider(body, "Hardness", 0.0, 1.0, 0.05, hardness, "", func(v: float): hardness = v,
		"1 is a crisp edge; 0 fades from the centre line out.")
	DccWidgets.slider(body, "Opacity", 0.0, 1.0, 0.05, opacity, "", func(v: float): opacity = v)
	DccWidgets.toggle(body, "Land only", land_only, func(v: bool): land_only = v,
		"On: sea and lakes under the stroke keep their colour. Off: water is painted too (a crater lake inside a painted crater).")
	_mat_rows = []
	_mat_rows.append(DccWidgets.slider(body, "Heat", 0.0, 1.0, 0.05, heat, "", func(v: float): heat = v,
		"How much of the surface is open and hot: 0 a cooled crust with faint seams, 1 a molten, white-hot lake."))
	_mat_rows.append(DccWidgets.slider(body, "Crack density", 0.0, 1.0, 0.05, crack, "", func(v: float): crack = v,
		"Plate size: coarse plates (2 km) to fine (0.25 km). Finer cracks keep appearing as you zoom in."))
	_mat_rows.append(DccWidgets.slider(body, "Glow radius", 0.0, 10.0, 0.1, glow_km, "km", func(v: float): glow_km = v,
		"How far the glow spills onto the ground around the stroke."))
	_mat_rows.append(DccWidgets.slider(body, "Cool along stroke", 0.0, 1.0, 0.05, heat_along, "", func(v: float): heat_along = v,
		"How much a stroke cools from where it starts to where it ends: a flow cooling away from its vent."))
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	body.add_child(row)
	_undo_btn = DccWidgets.action(row, "Undo", undo)
	_redo_btn = DccWidgets.action(row, "Redo", redo)
	_hide_btn = DccWidgets.toggle(body, "Hide colour paint", false, func(v: bool): set_hidden(v),
		"Hides every mark without deleting any. Undoable.")
	var clr := DccWidgets.action(body, "Clear colour paint", clear)
	clr.tooltip_text = "Removes every mark. Undoable."
	_status = DccWidgets.note(body, "")
	_refresh_ui()

## The brush ring at the cursor while the tool is armed: the radius in grid
## cells, from the same `km_per_cell` the stroke uses. Called by the CARTO
## workspace's `on_cursor_sampled`.
func on_cursor_sampled(gx: float, gy: float, valid: bool) -> void:
	if app == null or app.viewport == null or app.viewport.tool_overlay == null or app.armed_tool != TOOL_ID:
		return
	var kpc := km_per_cell()
	app.viewport.tool_overlay.set_brush_cursor(valid and kpc > 0.0, gx, gy, radius_km / kpc if kpc > 0.0 else 0.0)

func _material_index() -> int:
	for i in MATERIALS.size():
		if String(MATERIALS[i]["key"]) == material:
			return i
	return 0

func _blend_index() -> int:
	for i in BLENDS.size():
		if String(BLENDS[i]["key"]) == blend:
			return i
	return 0

## Picks material `i`: a glow preset loads its heat, density, halo and crust.
func _set_material(i: int) -> void:
	var m: Dictionary = MATERIALS[clampi(i, 0, MATERIALS.size() - 1)]
	material = String(m["key"])
	if m.has("heat"):
		heat = float(m["heat"])
		crack = float(m["crack"])
		glow_km = float(m["glow"])
		rgb = m["crust"]
		if _pick != null and is_instance_valid(_pick):
			_pick.color = rgb
	_refresh_ui()

## Re-reads the layer and brings the status line, the material rows and the
## undo/redo buttons up to date.
func _refresh_ui() -> void:
	var glow := material != "colour"
	for r in _mat_rows:
		var row = (r as Dictionary).get("row")
		if row is Control and is_instance_valid(row):
			(row as Control).visible = glow
	if _undo_btn != null and is_instance_valid(_undo_btn):
		_undo_btn.disabled = _undo.is_empty()
	if _redo_btn != null and is_instance_valid(_redo_btn):
		_redo_btn.disabled = _redo.is_empty()
	if _status == null or not is_instance_valid(_status):
		return
	var info: Dictionary = bridge.colour_paint_info()
	if info.is_empty():
		_status.text = "Colour paint: not available while the world is generating." if bridge.generating else "Colour paint: no binding."
		return
	if not info.has("km_per_cell"):
		_status.text = "This world has no map width, so paint (stored in kilometres) cannot be placed."
		return
	var n := int(info.get("marks", 0))
	## The brush's size on this map: kilometres are fixed, so the readout says
	## how many grid cells (and, at the current zoom, screen pixels) that is.
	var kpc := float(info["km_per_cell"])
	var cells := radius_km / kpc
	var px_txt := ""
	## Screen px per cell = the overlay's unzoomed px per cell x the camera zoom.
	if app != null and app.viewport != null and app.viewport.has_method("label_px_per_cell") and app.viewport.has_method("zoom"):
		var ppc: float = float(app.viewport.label_px_per_cell()) * float(app.viewport.zoom())
		if ppc > 0.0:
			px_txt = ", %d px at this zoom" % int(round(cells * ppc))
	_status.text = "%d mark%s%s. Brush %.2f km = %.1f cells%s." % [n, "" if n == 1 else "s", " (hidden)" if not bool(info.get("visible", true)) and n > 0 else "", radius_km, cells, px_txt]
	if _hide_btn != null and is_instance_valid(_hide_btn):
		_hide_btn.set_pressed_no_signal(not bool(info.get("visible", true)))

# -- Stroke capture ----------------------------------------------------------------

func _on_press(gx: float, gy: float) -> void:
	_cancel_stroke()
	var kpc := km_per_cell()
	if kpc <= 0.0:
		if app != null and app.has_method("set_status"):
			app.set_status("hint", "Not done: colour paint needs a world with a map width.", "warn")
		return
	_open = true
	_pts_km = [Vector2(gx * kpc, gy * kpc)]
	_pts_grid = PackedVector2Array([Vector2(gx, gy)])
	_preview()

func _on_drag(gx: float, gy: float) -> void:
	if not _open:
		return
	var kpc := km_per_cell()
	if kpc <= 0.0:
		return
	var p := Vector2(gx * kpc, gy * kpc)
	if p.distance_to(_pts_km[_pts_km.size() - 1]) < radius_km * SPACING:
		return
	_pts_km.append(p)
	_pts_grid.append(Vector2(gx, gy))
	_preview()

func _on_release(gx: float, gy: float, valid: bool) -> void:
	if not _open:
		return
	if valid:
		_on_drag(gx, gy)
	var pts := _simplify(_pts_km, radius_km * SIMPLIFY)
	_cancel_stroke()
	commit_line(pts)

## Appends one stroke (km points) with the current brush as one undo step.
## Public so a probe can paint without synthesising pointer events.
func commit_line(points_km: Array) -> bool:
	if points_km.is_empty() or bridge == null or bridge.generating:
		return false
	while points_km.size() > MAX_POINTS:
		var thin: Array = []
		for i in range(0, points_km.size(), 2):
			thin.append(points_km[i])
		points_km = thin
	var pts: Array = []
	for p in points_km:
		var v: Vector2 = p
		pts.append([snappedf(v.x, 0.001), snappedf(v.y, 0.001)])
	var mark := {
		"shape": {"kind": "line", "points_km": pts, "radius_km": radius_km},
		"rgb": [rgb.r8, rgb.g8, rgb.b8],
		"opacity": opacity,
		"hardness": hardness,
		"blend": blend,
		"land_only": land_only,
	}
	if material != "colour":
		var m: Dictionary = MATERIALS[_material_index()]
		mark["emissive"] = {"ramp": String(m["ramp"]), "heat": heat, "crack_density": crack, "glow_radius_km": glow_km, "heat_along": heat_along}
	var before: String = bridge.colour_paint_document()
	var r: Dictionary = bridge.colour_paint_add_mark(JSON.stringify(mark))
	if not bool(r.get("ok", false)):
		if app != null and app.has_method("set_status"):
			app.set_status("hint", "Not painted: %s" % String(r.get("error", "the engine refused the stroke")), "warn")
		return false
	_push_undo(before, "Paint stroke" if material == "colour" else "%s stroke" % String(MATERIALS[_material_index()]["label"]))
	_after_edit()
	return true

func _preview() -> void:
	if app == null or app.viewport == null or app.viewport.tool_overlay == null:
		return
	app.viewport.tool_overlay.set_path_preview(_pts_grid)

## Ends any open stroke without painting it (tool change, world change).
func _cancel_stroke() -> void:
	_open = false
	_pts_km = []
	_pts_grid = PackedVector2Array()
	if app != null and app.viewport != null and app.viewport.tool_overlay != null:
		app.viewport.tool_overlay.set_path_preview(PackedVector2Array())

## Ramer-Douglas-Peucker on km points; keeps the ends.
static func _simplify(pts: Array, tol: float) -> Array:
	if pts.size() < 3:
		return pts.duplicate()
	var keep := []
	keep.resize(pts.size())
	keep.fill(false)
	keep[0] = true
	keep[pts.size() - 1] = true
	var stack := [[0, pts.size() - 1]]
	while not stack.is_empty():
		var seg: Array = stack.pop_back()
		var a: int = seg[0]
		var b: int = seg[1]
		var pa: Vector2 = pts[a]
		var pb: Vector2 = pts[b]
		var best := -1.0
		var bi := -1
		for i in range(a + 1, b):
			var d := Geometry2D.get_closest_point_to_segment(pts[i], pa, pb).distance_to(pts[i])
			if d > best:
				best = d
				bi = i
		if bi >= 0 and best > tol:
			keep[bi] = true
			stack.append([a, bi])
			stack.append([bi, b])
	var out := []
	for i in pts.size():
		if keep[i]:
			out.append(pts[i])
	return out

# -- Layer actions and undo --------------------------------------------------------

func set_hidden(hidden: bool) -> void:
	var before: String = bridge.colour_paint_document()
	if bridge.colour_paint_set_visible(not hidden):
		_push_undo(before, "Colour paint hidden" if hidden else "Colour paint shown")
		_after_edit()

func clear() -> void:
	var before: String = bridge.colour_paint_document()
	if before == "":
		return
	var r: Dictionary = bridge.colour_paint_set_document("")
	if bool(r.get("ok", false)):
		_push_undo(before, "Clear colour paint")
		_after_edit()

func undo() -> void:
	_step(_undo, _redo)

func redo() -> void:
	_step(_redo, _undo)

## Moves one step from `from` to `to`: the current document goes onto `to`, the
## stored one is restored.
func _step(from: Array, to: Array) -> void:
	if from.is_empty() or bridge == null or bridge.generating:
		return
	var entry: Dictionary = from.pop_back()
	var now: String = bridge.colour_paint_document()
	var r: Dictionary = bridge.colour_paint_set_document(String(entry["doc"]))
	if not bool(r.get("ok", false)):
		from.append(entry)
		return
	to.append({"doc": now, "label": entry["label"]})
	_after_edit()

func can_undo() -> bool:
	return not _undo.is_empty()

func can_redo() -> bool:
	return not _redo.is_empty()

func _push_undo(doc: String, label: String) -> void:
	_undo.append({"doc": doc, "label": label})
	_redo.clear()
	## The app's undo budget, shared: the engine's stack uses `bytes` of
	## `budget_bytes`; this stack drops its oldest steps to stay inside what is
	## left (never below one step), and never holds more steps than the engine's.
	var st: Dictionary = bridge.undo_stats() if bridge.has_method("undo_stats") else {}
	var room := int(st.get("budget_bytes", 64 << 20)) - int(st.get("bytes", 0))
	var max_steps := int(st.get("max_steps", 100))
	var total := 0
	for e in _undo:
		total += String((e as Dictionary)["doc"]).length()
	while _undo.size() > 1 and (total > room or _undo.size() > max_steps):
		total -= String((_undo[0] as Dictionary)["doc"]).length()
		_undo.pop_front()

## World changed (a regenerate or a load): the engine's layer may be another
## one, so this stack no longer describes it.
func on_world_changed() -> void:
	_cancel_stroke()
	_undo.clear()
	_redo.clear()
	_refresh_ui()

func _after_edit() -> void:
	if app != null and app.viewport != null:
		app.viewport.refresh()
	_refresh_ui()
