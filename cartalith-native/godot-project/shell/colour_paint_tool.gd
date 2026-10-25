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
## Paint layers (Ruling CS, owner 2026-10-19: default blend **Colour**, and
## *"use paint layers"*): the engine's document is a stack of named layers,
## composited bottom first before the light. The section lists them **top
## first** (as a paint program does), one row each -- show/hide, the name
## (press to make it the active layer, the one strokes go into), lock, move
## up, move down -- with New layer, Rename, Clear layer and Delete layer
## acting on the active one. Every layer edit is one step on this tool's undo
## stack (a whole document before the change), exactly like a stroke.
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
##
## `blend` is the material's own default blend, loaded when it is picked (the
## Blend row follows). The flat colour takes Ruling CS's **Colour**. The glow
## materials keep **Replace**, as shipped and as their frames were judged: the
## blend applies to the crust before the light, and a Colour-blended crust
## takes the ground's own lightness -- near-white crust on a snow cone, which
## is not lava. The ruling keeps lava's defaults "as shipped".
const MATERIALS := [
	{"key": "colour", "label": "Colour", "blend": "colour"},
	{"key": "lava", "label": "Lava", "blend": "replace", "ramp": "fire", "heat": 0.75, "crack": 0.5, "glow": 1.5, "crust": Color8(34, 33, 38)},
	{"key": "ember", "label": "Embers", "blend": "replace", "ramp": "fire", "heat": 0.35, "crack": 0.8, "glow": 0.8, "crust": Color8(40, 36, 34)},
	{"key": "ice", "label": "Ice glow", "blend": "replace", "ramp": "ice", "heat": 0.75, "crack": 0.5, "glow": 1.5, "crust": Color8(30, 36, 44)},
	{"key": "toxic", "label": "Toxic glow", "blend": "replace", "ramp": "toxic", "heat": 0.75, "crack": 0.5, "glow": 1.5, "crust": Color8(30, 36, 28)},
]

## Blend modes, in the engine's keys (`ColourBlend::key`). Colour first: the
## owner's default (Ruling CS, 2026-10-19, overriding the scope's measured
## Replace of §3). Replace stays one choice away for a red that must read red
## on snow (Colour there is pink).
const BLENDS := [
	{"key": "colour", "label": "Colour (tint)"},
	{"key": "replace", "label": "Replace"},
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
## Ruling CS: Colour is the default blend (the flat material's own default).
var blend := "colour"
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
## LLM-5: how many steps `_push_undo` has ever pushed; each entry carries its
## own value as `"n"` and keeps it as it moves between `_undo` and `_redo`, so
## an undo group (`shell/undo_group.gd`) can name its own steps after any
## number of undos and redos. Never reset, so a serial is never reused.
var serial := 0

## The section's live widgets, for `_refresh_ui`.
var _status: Label
var _undo_btn: Button
var _redo_btn: Button
var _hide_btn: CheckBox
var _mat_rows: Array = []
var _colour_row: Control
var _pick: ColorPickerButton
var _blend_ob: OptionButton

## The active paint layer, as the engine's index (0 = bottom): where strokes
## go and what Rename / Clear layer / Delete layer act on. Clamped on every
## refresh, so an undo that removes a layer never leaves it pointing past the
## end. With no layers it is 0, which starts the default "Paint 1".
var active_layer := 0
## The layers list's rows box (rebuilt by `_rebuild_layers`) and the
## active-layer action buttons (disabled with a reason when they cannot act).
var _layers_box: VBoxContainer
var _layer_actions: Dictionary = {}

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
	_blend_ob = DccWidgets.choice(body, "Blend", bl, _blend_index(), func(i: int): blend = String(BLENDS[i]["key"]),
		"Colour (the default): the colour's hue at the ground's own lightness -- keeps texture and relief; cannot make snow deep red. Replace: the colour itself, shaded by the relief. Multiply: a stain that only darkens. Picking a material loads its own default (glow materials use Replace).")
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
	if bridge.colour_paint_layers_api:
		_build_layers(body)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	body.add_child(row)
	_undo_btn = DccWidgets.action(row, "Undo", undo)
	_redo_btn = DccWidgets.action(row, "Redo", redo)
	_hide_btn = DccWidgets.toggle(body, "Hide colour paint", false, func(v: bool): set_hidden(v),
		"Hides every mark without deleting any. Undoable.")
	var clr := DccWidgets.action(body, "Clear colour paint", clear)
	clr.tooltip_text = "Removes every mark in every layer, and the layers. Undoable."
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

## Picks material `i`: it loads its own default blend (the Blend row follows,
## so the two views of `blend` never disagree), and a glow preset its heat,
## density, halo and crust.
func _set_material(i: int) -> void:
	var m: Dictionary = MATERIALS[clampi(i, 0, MATERIALS.size() - 1)]
	material = String(m["key"])
	blend = String(m.get("blend", blend))
	if _blend_ob != null and is_instance_valid(_blend_ob):
		_blend_ob.select(_blend_index())
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
	var nl := int(info.get("layers", 0))
	var in_layers := " in %d layers" % nl if nl > 1 else ""
	_status.text = "%d mark%s%s%s. Brush %.2f km = %.1f cells%s." % [n, "" if n == 1 else "s", in_layers, " (hidden)" if not bool(info.get("visible", true)) and n > 0 else "", radius_km, cells, px_txt]
	if _hide_btn != null and is_instance_valid(_hide_btn):
		_hide_btn.set_pressed_no_signal(not bool(info.get("visible", true)))
	_rebuild_layers()

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
	## Into the active paint layer (Ruling CS); a binary without layers takes
	## the stroke on top as CP-2 did.
	var r: Dictionary = bridge.colour_paint_layer_edit("colour_paint_add_mark_to", [_clamped_active(), JSON.stringify(mark)]) \
		if bridge.colour_paint_layers_api else bridge.colour_paint_add_mark(JSON.stringify(mark))
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

# -- Paint layers (Ruling CS) ------------------------------------------------------

## The tap target on this form factor, both axes: 44 px on touch (`MISTAKES.md`:
## a control a rebuild creates escapes the fitters, so it is floored here, at
## the call site, on width AND height), 28 px with a pointer.
static func _tap() -> float:
	return 44.0 if DccTheme.is_touch() else 28.0

## The layers, bottom first, as the engine lists them (`[]` with none).
func layers() -> Array:
	return bridge.colour_paint_layers() if bridge != null else []

## `active_layer` clamped into the current layers (0 with none).
func _clamped_active() -> int:
	return clampi(active_layer, 0, maxi(layers().size() - 1, 0))

## Builds the PAINT LAYERS block into `body`: a heading, the rows box (filled by
## `_rebuild_layers`) and the actions on the active layer, which wrap rather than
## widen the dock (`MISTAKES.md`: a row that cannot shrink widens every ancestor).
func _build_layers(body: Control) -> void:
	body.add_child(DccTheme.label("PAINT LAYERS", "text_dim", DccTheme.FS_SMALL))
	_layers_box = VBoxContainer.new()
	_layers_box.add_theme_constant_override("separation", 4)
	body.add_child(_layers_box)
	var acts := HFlowContainer.new()
	acts.add_theme_constant_override("h_separation", 6)
	acts.add_theme_constant_override("v_separation", 6)
	body.add_child(acts)
	_layer_actions = {}
	for spec in [
		["new", "New layer", func(): add_layer(""), "Adds an empty layer above the others and makes it the active one. Undoable."],
		["rename", "Rename…", func(): _prompt_rename(), "Renames the active layer."],
		["clear", "Clear layer", func(): clear_layer(_clamped_active()), "Removes every mark on the active layer and keeps the layer. Undoable."],
		["delete", "Delete layer", func(): delete_layer(_clamped_active()), "Removes the active layer and its marks. Undoable."],
	]:
		var b := DccWidgets.action(acts, String(spec[1]), spec[2])
		b.tooltip_text = String(spec[3])
		b.custom_minimum_size = Vector2(maxf(b.custom_minimum_size.x, _tap()), maxf(b.custom_minimum_size.y, _tap()))
		_layer_actions[spec[0]] = b
	_rebuild_layers()

## Re-reads the layers and redraws the rows, **top layer first**: show/hide,
## the name (pressed = active; press to make it active), lock, up, down. Each
## button is floored to `_tap()` on both axes. Also brings the action buttons'
## disabled state and its reason up to date.
func _rebuild_layers() -> void:
	if _layers_box == null or not is_instance_valid(_layers_box):
		return
	for c in _layers_box.get_children():
		_layers_box.remove_child(c)
		c.queue_free()
	var ls := layers()
	var n := ls.size()
	active_layer = _clamped_active()
	var t := _tap()
	if n == 0:
		DccWidgets.note(_layers_box, "No layers yet. The first stroke starts “Paint 1”; New layer adds one above it.")
	for k in n:
		var i := n - 1 - k
		var l: Dictionary = ls[i]
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 4)
		row.set_meta("paint_layer", i)
		_layers_box.add_child(row)
		var vis := bool(l.get("visible", true))
		var locked := bool(l.get("locked", false))
		var eye := _layer_button(row, "●" if vis else "○", t,
			"Shown: press to hide this layer (its marks stay)." if vis else "Hidden: press to show this layer.")
		eye.pressed.connect(func(): set_layer_visible(i, not vis))
		var name_b := _layer_button(row, "%s  · %d" % [String(l.get("name", "")), int(l.get("marks", 0))], t,
			"The active layer: new strokes go here." if i == active_layer else "Press to make this the active layer (where strokes go).")
		name_b.toggle_mode = true
		name_b.button_pressed = i == active_layer
		name_b.clip_text = true
		name_b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		name_b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		name_b.custom_minimum_size.x = maxf(60.0, t)
		if not vis:
			name_b.modulate.a = 0.55
		name_b.pressed.connect(func(): select_layer(i))
		## LOCK / LOCKED in Plex Mono rather than a glyph, as the seed lock does
		## (`new_world_dialog.gd::_build_seed_lock`). The word carries the state
		## for a reader with no sight of the dimming. (Corrected 2026-10-19: this
		## said `DccIcons.SYMBOLS["locked"]` was an emoji the fonts do not draw,
		## "measured here: it rendered as ǵ12". The ǵ12 was the entry's own
		## escape, `"\U0001F512"`, which GDScript 4 reads as U+01F5 + "12";
		## fixed in `dcc_icons.gd`. The glyph is not in Plex Mono and draws only
		## through the system-font fallback, so the word stays.)
		var lock := _layer_button(row, "LOCKED" if locked else "LOCK", t,
			"Locked: its marks cannot be painted on, cleared or deleted. Press to unlock." if locked else "Unlocked. Press to lock this layer's marks.")
		lock.add_theme_font_override("font", DccTheme.mono(1, true))
		lock.add_theme_font_size_override("font_size", DccTheme.FS_MICRO)
		lock.custom_minimum_size.x = maxf(t, 52.0)
		lock.modulate.a = 1.0 if locked else 0.5
		lock.pressed.connect(func(): set_layer_locked(i, not locked))
		var up := _layer_button(row, "▲", t, "Move this layer up (drawn over the layer above it).")
		up.disabled = i == n - 1
		up.pressed.connect(func(): move_layer(i, i + 1))
		var dn := _layer_button(row, "▼", t, "Move this layer down (drawn under the layer below it).")
		dn.disabled = i == 0
		dn.pressed.connect(func(): move_layer(i, i - 1))
	var act: Dictionary = ls[active_layer] if n > 0 else {}
	var act_locked := bool(act.get("locked", false))
	if _layer_actions.has("rename"):
		(_layer_actions["rename"] as Button).disabled = n == 0
		(_layer_actions["rename"] as Button).tooltip_text = "Renames the active layer." if n > 0 else "No layer to rename yet."
	for key in ["clear", "delete"]:
		if not _layer_actions.has(key):
			continue
		var b: Button = _layer_actions[key]
		b.disabled = n == 0 or act_locked or (key == "clear" and int(act.get("marks", 0)) == 0)
		if n == 0:
			b.tooltip_text = "No layer yet."
		elif act_locked:
			b.tooltip_text = "The active layer “%s” is locked." % String(act.get("name", ""))
		elif key == "clear" and int(act.get("marks", 0)) == 0:
			b.tooltip_text = "The active layer has no marks."
		else:
			b.tooltip_text = "Removes every mark on the active layer and keeps the layer. Undoable." if key == "clear" else "Removes the active layer and its marks. Undoable."

## One square layer-row button, floored to `t` on both axes.
func _layer_button(row: Control, text: String, t: float, tip: String) -> Button:
	var b := Button.new()
	b.text = text
	b.focus_mode = Control.FOCUS_NONE
	b.tooltip_text = tip
	b.custom_minimum_size = Vector2(t, t)
	## The density's prose size, as `DccWidgets.choice()` sets it: a bare
	## `Button` keeps the theme's pointer size, which the phone frame of
	## 2026-10-19 showed as unreadably small names and glyphs in 44 px rows.
	b.add_theme_font_size_override("font_size", DccTheme.role_px("fs_prose") if DccTheme.is_tablet() else DccTheme.FS_SMALL)
	row.add_child(b)
	return b

func _prompt_rename() -> void:
	var ls := layers()
	if ls.is_empty() or app == null:
		return
	var i := _clamped_active()
	DccWidgets.prompt(app, "Rename paint layer", "Name:", String((ls[i] as Dictionary).get("name", "")), "Rename",
		func(typed: String): rename_layer(i, typed))

## Makes layer `i` the active one (no engine edit, so no undo step).
func select_layer(i: int) -> void:
	active_layer = i
	_rebuild_layers()

## One layer edit as one undo step: the document before it is pushed only when
## the engine accepted the edit; a refusal changes nothing and says why.
## Returns the engine's reply (`{}` without the binding or while generating).
func _layer_op(op: String, args: Array, label: String) -> Dictionary:
	if bridge == null or bridge.generating:
		return {}
	var before: String = bridge.colour_paint_document()
	var r: Dictionary = bridge.colour_paint_layer_edit(op, args)
	if not bool(r.get("ok", false)):
		if app != null and app.has_method("set_status") and r.has("error"):
			app.set_status("hint", "Not done: %s" % String(r["error"]), "warn")
		return r
	_push_undo(before, label)
	_after_edit()
	return r

## Adds a layer on top (`""` = the next free "Paint N") and makes it active.
func add_layer(name_: String) -> bool:
	var r := _layer_op("colour_paint_layer_add", [name_], "New layer")
	if bool(r.get("ok", false)):
		active_layer = int(r.get("layer", active_layer))
		_rebuild_layers()
		return true
	return false

func rename_layer(i: int, name_: String) -> bool:
	return bool(_layer_op("colour_paint_layer_rename", [i, name_], "Rename layer").get("ok", false))

## Moves layer `from` to `to`; the active layer stays the same layer.
func move_layer(from: int, to: int) -> bool:
	var a := _clamped_active()
	if not bool(_layer_op("colour_paint_layer_move", [from, to], "Move layer").get("ok", false)):
		return false
	if a == from:
		active_layer = to
	elif from < a and a <= to:
		active_layer = a - 1
	elif to <= a and a < from:
		active_layer = a + 1
	_rebuild_layers()
	return true

func set_layer_visible(i: int, v: bool) -> bool:
	return bool(_layer_op("colour_paint_layer_set_visible", [i, v], "Layer shown" if v else "Layer hidden").get("ok", false))

func set_layer_locked(i: int, v: bool) -> bool:
	return bool(_layer_op("colour_paint_layer_set_locked", [i, v], "Layer locked" if v else "Layer unlocked").get("ok", false))

## Deletes layer `i`; the active layer stays the same layer where it survives.
func delete_layer(i: int) -> bool:
	var a := _clamped_active()
	if not bool(_layer_op("colour_paint_layer_delete", [i], "Delete layer").get("ok", false)):
		return false
	if a > i:
		active_layer = a - 1
	_rebuild_layers()
	return true

func clear_layer(i: int) -> bool:
	return bool(_layer_op("colour_paint_layer_clear", [i], "Clear layer").get("ok", false))

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

## This tool's own Undo button. **Refuses a step that belongs to an LLM-5 undo
## group** (`"group"` on the entry): taking it alone would half-undo a change
## the user applied as one, so the status line points at Edit > Undo, which
## undoes the whole group. An ungrouped step is undone exactly as before.
func undo() -> void:
	if _refuse_grouped(_undo, "Undo"):
		return
	_step(_undo, _redo)

## This tool's own Redo button; the mirror of `undo()`, same group refusal.
func redo() -> void:
	if _refuse_grouped(_redo, "Redo"):
		return
	_step(_redo, _undo)

## LLM-5: `true` (and a status line) when the step `from` would take next is
## part of an undo group. The group's own label names it, read from the app's
## `undo_groups`; with no app (a bare probe) the refusal still holds.
func _refuse_grouped(from: Array, what: String) -> bool:
	if from.is_empty() or not (from.back() as Dictionary).has("group"):
		return false
	if app != null and app.has_method("set_status"):
		var label := "a grouped change"
		if "undo_groups" in app and app.undo_groups != null:
			label = app.undo_groups.label_of(int((from.back() as Dictionary)["group"]))
		app.set_status("hint", "Not done: this paint step is part of \"%s\". Use Edit > %s, which takes the whole change." % [label, what], "warn")
	return true

## LLM-5: one step for an undo group (`shell/undo_group.gd`), bypassing the
## refusal above because the group is the caller taking its own step. Returns
## whether the document was restored.
func step_for_group(undo_side: bool) -> bool:
	return _step(_undo, _redo) if undo_side else _step(_redo, _undo)

## Moves one step from `from` to `to`: the current document goes onto `to`, the
## stored one is restored. The entry's serial (`"n"`) and group tag (`"group"`)
## travel with it (LLM-5). Returns whether anything moved: `false` for an empty
## stack, a running generate, or a document the engine refused (the entry is
## then put back where it was).
func _step(from: Array, to: Array) -> bool:
	if from.is_empty() or bridge == null or bridge.generating:
		return false
	var entry: Dictionary = from.pop_back()
	var now: String = bridge.colour_paint_document()
	var r: Dictionary = bridge.colour_paint_set_document(String(entry["doc"]))
	if not bool(r.get("ok", false)):
		from.append(entry)
		return false
	var moved := {"doc": now, "label": entry["label"]}
	for k in ["n", "group"]:
		if entry.has(k):
			moved[k] = entry[k]
	to.append(moved)
	_after_edit()
	return true

func can_undo() -> bool:
	return not _undo.is_empty()

func can_redo() -> bool:
	return not _redo.is_empty()

func _push_undo(doc: String, label: String) -> void:
	serial += 1
	_undo.append({"doc": doc, "label": label, "n": serial})
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
	## A reopened project's strokes go onto its top layer, as a new one's do.
	active_layer = maxi(layers().size() - 1, 0)
	_refresh_ui()

func _after_edit() -> void:
	if app != null and app.viewport != null:
		app.viewport.refresh()
	_refresh_ui()
