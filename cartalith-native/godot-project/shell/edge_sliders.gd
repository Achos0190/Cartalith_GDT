extends Control
## The tablet's brush edge sliders (`MAP_CONTEXT_SCOPE.md` s.7.3, fork F5b; owner, 2026-11-17): while a brush is armed
## on a TABLET, one or two thin vertical sliders (radius, strength) sit on the screen edge nearest the NON-dominant hand,
## always visible and within the thumb's reach. The touch counterpart of s.6's Ctrl+RMB HUD (Nomad Sculpt's edge
## sliders, s.2.1). They never replace the tool options bar and own no value of their own.
##
## **One source of truth.** Each rail reads and writes the very state the options bar (or, for colour paint, the dock
## and the phone strip) reads: the engine's sculpt globals (`brush_size`, `intensity`), the world workspace's
## `_paint_brush["radius"]`, `ColourPaintTool.radius_km` / `.opacity`, `LlmRegion.brush`. Nothing is cached as truth: the
## rail's thumb is re-read from that state a few times a second (`TICK_S`) while it is not being dragged, so a value
## moved on the options bar shows here within a tick, and a drag here repaints the bar (throttled) while it moves.
## Paint and the Assistant region brush expose a radius only, so they get one rail; no tool is given a rail for a value
## it does not have (a hardness or a spacing is not "strength").
##
## **Where it may appear.** Only when `DccTheme.is_tablet()` (never the phone, per s.8, never desktop), a brush tool is
## armed (`_tool_descs`), the Preferences switch is on (`DccSettings.edge_sliders_enabled()`, default on) and a world
## exists. Everything else leaves it `visible = false`, so a probe can assert absence by the flag and by the rect.
##
## **Where it sits.** On the edge OPPOSITE `DccSettings.dominant_hand()` (right-handed: left edge), the same side the
## scale bar takes. Vertically it is centred in the band left between the chrome that already lives on that edge
## (`_obstacles`: any other visible, input-taking child of the map host that overlaps the strip's columns: the Layers
## button, the readouts, the scale bar, the zoom row). If that band is shorter than `MIN_BAND` it is hidden rather
## than overlap anything. Each rail's hit area is the 44 dp touch floor wide (`HIT_FLOOR`), its drawn track thin.
##
## **Pen versus finger.** A rail moves only for a press that BEGAN on it (Godot hands later motion to the control that
## took the press), so a pen stroke begun on the map and dragged across the strip never moves a slider. A press on the
## rail is accepted from a finger or from a pen tapping directly on it. The rail is tagged `MAP_CHROME_META`, which is how
## `ViewportHost._on_map` knows a finger there is the slider's and not a pan or a tool press.
##
## No `class_name`: preloaded by `viewport_host.gd`, so adding it needs no `--import` pass (and so no `project.godot`
## rewrite, `MISTAKES.md`).

const LlmRegion := preload("res://shell/llm_region.gd")

## The touch floor in px (`DccToolBar.TOUCH_FLOOR`, 44 dp): each rail's hit area is at least this wide.
const HIT_FLOOR := 44.0
## Gap in px kept between the strip and any other chrome, and from the screen edge.
const GAP := 8.0
## The shortest vertical band (px) the strip will use; below it the strip hides rather than overlap chrome. A labelled
## judgement: a 150 px rail is about three thumb-widths of travel at the 44 dp floor.
const MIN_BAND := 150.0
## The longest a rail is drawn (px): longer than a thumb sweeps from one grip, and the band is centred so it stays near.
const MAX_LEN := 380.0
## Seconds between the polls that re-read values, re-find the armed tool and re-lay the strip out.
const TICK_S := 0.1
## Seconds between options-bar repaints while a rail is being dragged (the bar rebuilds its controls each time).
const BAR_REPAINT_S := 0.12
## Fallback ranges when the engine's own table is not readable (`GLOBAL_RANGES` in `sculpt_bridge.rs`: brush_size
## 6..200 step 1, intensity 0..1.5 step 0.01). Only used until `get_sculpt_globals_info()` answers.
const SCULPT_SIZE_RANGE := [6.0, 200.0, 1.0]
const SCULPT_STRENGTH_RANGE := [0.0, 1.5, 0.01]

## The app (`DccApp`), found by walking up for a node that has `tool_armed`; null until the host is in the tree.
var _app: Node = null
var _acc := 0.0
var _bar_acc := 0.0
var _bar_dirty := false
## The armed tool the rails were built for, and the descriptors (one per rail, outer rail first).
var _tool := ""
var _descs: Array = []
var _rails: Array = []
var _readout: Label
var _backing: Control
var _sculpt_info := {}
var _hand_seen := ""


## One vertical rail: a thin track in a 44 dp wide hit area. Draws itself, reports moves, owns no value but the one it
## last displayed (re-set from the tool's state by the strip).
class Rail extends Control:
	signal moved(v: float)
	signal released()
	var caption := ""
	var vmin := 0.0
	var vmax := 1.0
	var step := 0.01
	## Exponent of the travel curve: value = min + (max - min) * t^gamma. 1 is linear; 2 gives a wide range (a colour
	## brush from 0.05 to 25 km) fine control at the small end.
	var gamma := 1.0
	var value := 0.0
	var dragging := false

	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_STOP
		focus_mode = Control.FOCUS_NONE
		mouse_default_cursor_shape = Control.CURSOR_VSIZE

	## 0 at the bottom, 1 at the top: where `value` sits on the rail.
	func fraction() -> float:
		if vmax <= vmin:
			return 0.0
		return pow(clampf((value - vmin) / (vmax - vmin), 0.0, 1.0), 1.0 / gamma)

	func set_value_quiet(v: float) -> void:
		var c := clampf(v, vmin, vmax)
		if not is_equal_approx(c, value):
			value = c
			queue_redraw()

	## The travel pad (px) at each end, so the thumb centre stays on the rail.
	func _pad() -> float:
		return 16.0

	func _value_at(y: float) -> float:
		var h := maxf(1.0, size.y - 2.0 * _pad())
		var t := clampf(1.0 - (y - _pad()) / h, 0.0, 1.0)
		var v := vmin + (vmax - vmin) * pow(t, gamma)
		if step > 0.0:
			v = snappedf(v, step)
		return clampf(v, vmin, vmax)

	func _gui_input(ev: InputEvent) -> void:
		## Only a press that began on THIS rail starts a drag; motion with no drag open is ignored outright, so no stray
		## pointer (a pen hovering or stroking past the strip) can move it.
		if ev is InputEventMouseButton:
			var mb := ev as InputEventMouseButton
			if mb.button_index != MOUSE_BUTTON_LEFT:
				return
			if mb.pressed:
				dragging = true
				_apply(mb.position.y)
			elif dragging:
				dragging = false
				queue_redraw()
				released.emit()
			accept_event()
		elif ev is InputEventMouseMotion and dragging:
			var mm := ev as InputEventMouseMotion
			if (mm.button_mask & MOUSE_BUTTON_MASK_LEFT) == 0:
				dragging = false
				queue_redraw()
				released.emit()
				return
			_apply(mm.position.y)
			accept_event()

	func _apply(y: float) -> void:
		var v := _value_at(y)
		if not is_equal_approx(v, value):
			value = v
			queue_redraw()
			moved.emit(v)
		else:
			queue_redraw()

	## Thin track + fill + thumb, with a two-letter caption at the top. All colours are theme tokens.
	func _draw() -> void:
		var cx := size.x * 0.5
		var pad := _pad()
		var top := pad
		var bot := size.y - pad
		var tw := 6.0
		var track := Rect2(cx - tw * 0.5, top, tw, bot - top)
		draw_rect(track, DccTheme.c("text_dim").lerp(Color(0, 0, 0, 0), 0.55), true)
		var ty := bot - (bot - top) * fraction()
		draw_rect(Rect2(cx - tw * 0.5, ty, tw, bot - ty), DccTheme.c("accent"), true)
		draw_circle(Vector2(cx, ty), 11.0 if not dragging else 14.0, DccTheme.c("accent"))
		draw_circle(Vector2(cx, ty), 11.0 if not dragging else 14.0, DccTheme.c("text_dim"), false, 1.0)
		var font := DccTheme.mono(0)
		draw_string(font, Vector2(0.0, 12.0), caption, HORIZONTAL_ALIGNMENT_CENTER, size.x, 11, DccTheme.c("text_dim"))


func _init() -> void:
	name = "EdgeSliders"
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	set_anchors_preset(Control.PRESET_FULL_RECT)
	visible = false


func _ready() -> void:
	_backing = Control.new()
	_backing.name = "Backing"
	_backing.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_backing.draw.connect(_draw_backing)
	add_child(_backing)
	_readout = DccTheme.label("", "text", DccTheme.FS_SMALL)
	_readout.name = "Readout"
	_readout.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_readout.add_theme_stylebox_override("normal", _scrim())
	_readout.visible = false
	add_child(_readout)


func _scrim() -> StyleBoxFlat:
	var sb := DccTheme.flat(DccTheme.c("hud_scrim"), 6)
	sb.content_margin_left = 8
	sb.content_margin_right = 8
	sb.content_margin_top = 4
	sb.content_margin_bottom = 4
	return sb


## The translucent plate under the rails, so they read over any terrain colour.
func _draw_backing() -> void:
	if _rails.is_empty():
		return
	var r := Rect2()
	for i in _rails.size():
		var c := _rails[i] as Control
		var cr := Rect2(c.position, c.size)
		r = cr if i == 0 else r.merge(cr)
	_backing.draw_style_box(DccTheme.flat(DccTheme.c("hud_scrim"), 8), r)


func _process(delta: float) -> void:
	_acc += delta
	if _bar_dirty:
		_bar_acc += delta
	if _acc < TICK_S:
		if _bar_dirty and _bar_acc >= BAR_REPAINT_S:
			_repaint_bar()
		return
	_acc = 0.0
	_tick()


## True on the tablet class only (never phone, never desktop).
func _tablet() -> bool:
	return DccTheme.is_tablet() and not DccTheme.is_phone()


func _find_app() -> Node:
	if _app != null and is_instance_valid(_app):
		return _app
	var n: Node = get_parent()
	while n != null:
		if n.has_signal("tool_armed") and "armed_tool" in n:
			_app = n
			return n
		n = n.get_parent()
	return null


func _tick() -> void:
	var app := _find_app()
	var host := get_parent() as Control
	if app == null or host == null or not _tablet() or not DccSettings.edge_sliders_enabled():
		_hide()
		return
	var tool_id := String(app.armed_tool)
	if tool_id != _tool or _descs.is_empty() or _hand_seen != DccSettings.dominant_hand():
		_hand_seen = DccSettings.dominant_hand()
		_tool = tool_id
		_rebuild_rails(app)
	if _descs.is_empty():
		_hide()
		return
	if not _layout(host):
		_hide()
		return
	visible = true
	for i in _rails.size():
		var r := _rails[i] as Rail
		if r.dragging:
			continue
		var v: Variant = (_descs[i]["get"] as Callable).call()
		if v != null:
			r.set_value_quiet(float(v))


func _hide() -> void:
	visible = false
	if _readout != null:
		_readout.visible = false
	for r in _rails:
		(r as Rail).dragging = false


func _bridge(app: Node) -> Node:
	return app.get("bridge")


## The world workspace (the owner of `_paint_brush`), the same lookup `DccToolBar._world_workspace()` makes.
func _world_ws(app: Node) -> Node:
	var panels: Variant = app.get("_workspace_panels")
	return (panels as Dictionary).get("world") if panels is Dictionary else null


func _colour_tool(app: Node) -> Object:
	var wss: Variant = app.get("_workspaces")
	if wss is Array:
		for ws in wss:
			if ws is Object:
				var cp: Variant = (ws as Object).get("colour_paint")
				if cp is Object:
					return cp
	return null


func _sculpt_range(bridge: Node, key: String, fallback: Array) -> Array:
	if _sculpt_info.is_empty() and not bool(bridge.get("generating")) and bool(bridge.get("has_world")):
		for c in bridge.get_sculpt_globals_info():
			_sculpt_info[String((c as Dictionary).get("key", ""))] = c
	if _sculpt_info.has(key):
		var d: Dictionary = _sculpt_info[key]
		return [float(d.get("min", fallback[0])), float(d.get("max", fallback[1])), float(d.get("step", fallback[2]))]
	return fallback


## The rail descriptors for the armed tool, outer rail first: {cap, name, unit, min, max, step, gamma, get, set}.
## `get` returns null when the value is unreadable right now (a generation holds the engine), never a made-up number.
## Empty for any tool that is not a brush with a radius.
func _tool_descs(app: Node, tool_id: String) -> Array:
	var bridge := _bridge(app)
	if bridge == null:
		return []
	match tool_id:
		"sculpt":
			if not bool(bridge.get("has_world")):
				return []
			var sz := _sculpt_range(bridge, "brush_size", SCULPT_SIZE_RANGE)
			var st := _sculpt_range(bridge, "intensity", SCULPT_STRENGTH_RANGE)
			return [
				{"cap": "Size", "name": "Brush size", "unit": " px", "min": sz[0], "max": sz[1], "step": sz[2], "gamma": 1.0,
					"get": func() -> Variant: return _sculpt_get(bridge, "brush_size"),
					"set": func(v: float) -> void: _sculpt_set(bridge, "brush_size", roundf(v))},
				{"cap": "Str", "name": "Strength", "unit": "", "min": st[0], "max": st[1], "step": st[2], "gamma": 1.0,
					"get": func() -> Variant: return _sculpt_get(bridge, "intensity"),
					"set": func(v: float) -> void: _sculpt_set(bridge, "intensity", v)},
			]
		"paint":
			var ws := _world_ws(app)
			if ws == null or not bool(bridge.get("has_world")):
				return []
			return [
				{"cap": "Size", "name": "Brush size", "unit": " cells", "min": 1.0, "max": 40.0, "step": 1.0, "gamma": 1.0,
					"get": func() -> Variant: return float((ws._paint_brush as Dictionary).get("radius", 6.0)),
					"set": func(v: float) -> void:
						if bool(bridge.get("generating")):
							return
						ws._on_paint_radius_changed(v)
						_bar_dirty = true},
			]
		"colour_paint":
			var cp := _colour_tool(app)
			if cp == null:
				return []
			return [
				{"cap": "Size", "name": "Brush radius", "unit": " km", "min": 0.05, "max": 25.0, "step": 0.01, "gamma": 2.0,
					"get": func() -> Variant: return float(cp.get("radius_km")),
					"set": func(v: float) -> void: cp.set_radius(v)},
				{"cap": "Str", "name": "Opacity", "unit": "", "min": 0.0, "max": 1.0, "step": 0.05, "gamma": 1.0,
					"get": func() -> Variant: return float(cp.get("opacity")),
					"set": func(v: float) -> void: cp.set_opacity(v)},
			]
		LlmRegion.TOOL_ID:
			return [
				{"cap": "Size", "name": "Brush size", "unit": " cells", "min": 6.0, "max": 200.0, "step": 1.0, "gamma": 1.0,
					"get": func() -> Variant: return LlmRegion.brush,
					"set": func(v: float) -> void:
						LlmRegion.set_brush(v)
						LlmRegion._refresh_bar()},
			]
	return []


func _sculpt_get(bridge: Node, key: String) -> Variant:
	if bool(bridge.get("generating")) or not bool(bridge.get("has_world")):
		return null
	var g: Dictionary = bridge.sculpt_get_globals()
	return float(g[key]) if g.has(key) else null


func _sculpt_set(bridge: Node, key: String, v: float) -> void:
	if bool(bridge.get("generating")):
		return
	bridge.sculpt_set_globals({key: v})
	_bar_dirty = true


func _rebuild_rails(app: Node) -> void:
	for r in _rails:
		(r as Node).queue_free()
		remove_child(r as Node)
	_rails.clear()
	_descs = _tool_descs(app, _tool)
	for i in _descs.size():
		var d: Dictionary = _descs[i]
		var r := Rail.new()
		r.name = "Rail%d" % i
		r.caption = String(d["cap"])
		r.vmin = float(d["min"])
		r.vmax = float(d["max"])
		r.step = float(d["step"])
		r.gamma = float(d["gamma"])
		r.set_meta("map_chrome", true)   ## `ViewportHost.MAP_CHROME_META`: a finger here is the slider's, not the map's.
		r.tooltip_text = "%s (edge slider; also on the tool options)" % String(d["name"])
		r.moved.connect(_on_moved.bind(i))
		r.released.connect(_on_released.bind(i))
		add_child(r)
		_rails.append(r)
	_readout.visible = false
	move_child(_readout, get_child_count() - 1)


func _on_moved(v: float, i: int) -> void:
	var d: Dictionary = _descs[i]
	(d["set"] as Callable).call(v)
	var r := _rails[i] as Rail
	## The readout follows the thumb on the side facing the map, and shows the value AFTER the tool clamped it.
	var shown := r.value
	var got: Variant = (d["get"] as Callable).call()
	if got != null:
		shown = float(got)
	var digits := 0 if float(d["step"]) >= 1.0 else (1 if float(d["step"]) >= 0.1 else 2)
	_readout.text = "%s  %.*f%s" % [String(d["name"]), digits, shown, String(d["unit"])]
	_readout.visible = true
	var ms := _readout.get_minimum_size()
	var host := get_parent() as Control
	var ty := r.position.y + (r.size.y - 32.0) * (1.0 - r.fraction()) + 16.0
	var x := _strip_rect().end.x + 10.0 if _on_left_edge() else _strip_rect().position.x - ms.x - 10.0
	_readout.position = Vector2(x, clampf(ty - ms.y * 0.5, 0.0, maxf(0.0, host.size.y - ms.y)))


func _on_released(_i: int) -> void:
	_readout.visible = false
	_repaint_bar()
	## The paint tool's dock panel is a static set of widgets, repainted once the drag is over rather than per move.
	var app := _find_app()
	if app != null and _tool == "paint":
		var ws := _world_ws(app)
		if ws != null and ws.has_method("rebuild_paint_panel"):
			ws.rebuild_paint_panel()


## Redraws the options bar from the state a drag just wrote: the sculpt and paint bar's sliders are built from that
## state, so a rebuild is what puts the new value in them. Throttled to `BAR_REPAINT_S` while dragging.
func _repaint_bar() -> void:
	_bar_dirty = false
	_bar_acc = 0.0
	var bar := DccToolBar.instance()
	if bar != null and (_tool == "sculpt" or _tool == "paint"):
		bar.refresh()


func _on_left_edge() -> bool:
	return DccSettings.dominant_hand() == "right"


func _strip_rect() -> Rect2:
	var r := Rect2()
	for i in _rails.size():
		var c := _rails[i] as Control
		var cr := Rect2(c.position, c.size)
		r = cr if i == 0 else r.merge(cr)
	return r


## Other chrome the strip must stay clear of: visible, non-IGNORE children of the host (not full-rect layers, not this
## node), as host-local rects. The host's own labels (`IGNORE` filter but drawn on the corner) are named by property so
## the strip does not sit on the scale bar or readouts either.
func _obstacles(host: Control) -> Array:
	var out: Array = []
	var gp := host.global_position
	for c in host.get_children():
		if c == self or not (c is Control):
			continue
		var ctl := c as Control
		if not ctl.visible:
			continue
		var rect := Rect2(ctl.global_position - gp, ctl.size)
		if rect.size.x >= host.size.x * 0.9 and rect.size.y >= host.size.y * 0.9:
			continue
		var named := false
		for p in ["_layers_btn", "_readout_label", "_coords_label", "_scale_rule", "_scale_label", "_navpad"]:
			if host.get(p) == ctl:
				named = true
		if named or ctl.mouse_filter != Control.MOUSE_FILTER_IGNORE:
			if rect.size.x > 0.0 and rect.size.y > 0.0:
				out.append(rect)
	return out


## Places the rails; false when the band is too short to fit without overlap.
func _layout(host: Control) -> bool:
	var n := _rails.size()
	var hit := maxf(HIT_FLOOR, float(DccTheme.role_px("btn_min_h")))
	var w := hit * n
	var left_edge := _on_left_edge()
	var x0 := GAP if left_edge else host.size.x - GAP - w
	var strip_x := Rect2(x0 - GAP, 0.0, w + 2.0 * GAP, host.size.y)
	var top := GAP
	var bot := host.size.y - GAP
	var mid := host.size.y * 0.5
	for o in _obstacles(host):
		var rr: Rect2 = o
		if not rr.intersects(strip_x):
			continue
		if rr.get_center().y < mid:
			top = maxf(top, rr.end.y + GAP)
		else:
			bot = minf(bot, rr.position.y - GAP)
	var band := bot - top
	if band < MIN_BAND:
		return false
	var len := minf(band, MAX_LEN)
	var y0 := top + (band - len) * 0.5
	for i in n:
		var r := _rails[i] as Rail
		## Outer rail (the first descriptor, radius) is the one nearest the screen edge on either side.
		r.position = Vector2(x0 + hit * (i if left_edge else n - 1 - i), y0)
		r.size = Vector2(hit, len)
	_backing.queue_redraw()
	return true
