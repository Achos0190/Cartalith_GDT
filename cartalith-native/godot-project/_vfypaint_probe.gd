extends Node
## VERIFIER probe (untracked). Fix 4 only -- the tool-options-bar Discard.
##
##   godot --headless --path . _vfypaint_probe.tscn
##
## Discriminating pair: a Discard wired to the wrong engine call, or to the
## sculpt draft, or to a layer wipe, moves one of these and not the other.
##  - the PAINT draft must go to 0
##  - the already-COMMITTED paint total must survive
##  - the sculpt draft must be untouched
## Plus: the right dock's own paint context must be refreshed by the bar's
## Discard (the lane's claimed third refresh), read off the rendered dock.

var app: Node
var _fail := 0
var _n := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _eq(label: String, got: Variant, want: Variant) -> void:
	_n += 1
	var p := str(got) == str(want)
	if not p:
		_fail += 1
	print("P %s %-50s got=%s want=%s" % ["ok  " if p else "FAIL", label, str(got), str(want)])

func _yes(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if not cond:
		_fail += 1
	print("P %s %s%s" % ["ok  " if cond else "FAIL", label, ("  [" + detail + "]") if detail != "" else ""])

func _walk_buttons(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Button:
			out.append(c)
		_walk_buttons(c, out)

func _chip(t: String) -> Button:
	var out: Array = []
	_walk_buttons(app.tool_options_row, out)
	for b in out:
		if String((b as Button).text) == t:
			return b
	return null

## Buttons anywhere in the right dock body, by text.
func _dock_button(t: String) -> Button:
	var out: Array = []
	_walk_buttons(app.right_dock_body, out)
	for b in out:
		if String((b as Button).text).findn(t) >= 0:
			return b
	return null

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)
	var bridge = app.bridge
	bridge.generate({"seed": 4021, "width_km": 2000.0, "grid_w": 192, "grid_h": 144,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var w := 0
	while bridge.generating and w < 8000:
		await get_tree().process_frame
		w += 1
	await _frames(10)
	_yes("world generated", bridge.has_world)
	if not bridge.has_world:
		_finish()
		return

	app.select_domain("world")
	await _frames(6)
	var layers: PackedStringArray = bridge.get_paint_layers()
	bridge.paint_set_layer(String(layers[0]))
	bridge.paint_set_brush(1, 6.0, 1.0, 0.0, false, true)
	app.arm_tool("paint")
	await _frames(10)

	var commit: Button = _chip("Commit")
	var discard: Button = _chip("Discard")
	_yes("Commit chip on the tool options bar", commit != null)
	_yes("Discard chip on the tool options bar", discard != null)
	if discard == null:
		_finish()
		return
	_eq("Discard disabled with an empty draft", discard.disabled, true)
	_eq("Discard tooltip explains why", discard.tooltip_text.length() > 10, true)

	var gs: Vector2i = bridge.grid_size()
	## Leg 1 -- commit some paint so there is a committed layer to protect.
	for k in 5:
		bridge.paint_stroke_at(float(gs.x) * (0.30 + 0.03 * k), float(gs.y) * 0.5)
	bridge.paint_commit()
	await _frames(6)
	var committed: int = int(bridge.paint_painted_counts().get("total", 0))
	_yes("a committed paint layer exists", committed > 0, "%d cells" % committed)
	_eq("draft empty after commit", bridge.paint_draft_count(), 0)

	## Leg 2 -- a fresh draft on top, then Discard from the BAR.
	var sculpt_before: int = bridge.sculpt_stroke_point_count()
	for k in 4:
		bridge.paint_stroke_at(float(gs.x) * (0.62 + 0.03 * k), float(gs.y) * 0.4)
	app.tool_bar.rebuild()
	await _frames(8)
	var pending: int = bridge.paint_draft_count()
	_yes("draft pending after 4 dabs", pending > 0, "%d cells" % pending)
	discard = _chip("Discard")
	_yes("Discard still drawn with a live draft", discard != null)
	_eq("Discard enabled with a pending draft", discard.disabled, false)

	discard.emit_signal("pressed")
	await _frames(10)
	_eq("PAINT draft emptied by Discard", bridge.paint_draft_count(), 0)
	_eq("committed layer SURVIVED the discard",
		int(bridge.paint_painted_counts().get("total", 0)), committed)
	_eq("sculpt draft untouched", bridge.sculpt_stroke_point_count(), sculpt_before)
	discard = _chip("Discard")
	_yes("Discard redrawn after the press", discard != null)
	if discard != null:
		_eq("Discard disabled again", discard.disabled, true)

	## The third refresh the lane claims: the right dock's paint context.
	## Stage a draft, then discard from the bar, and read the right dock's own
	## pending count back out of the rendered tree.
	var ws0 = app._world_workspace()
	if ws0 != null and ws0.has_method("_refresh_right_dock_paint"):
		ws0._refresh_right_dock_paint()
	await _frames(8)
	for k in 3:
		bridge.paint_stroke_at(float(gs.x) * (0.45 + 0.03 * k), float(gs.y) * 0.6)
	app.tool_bar.rebuild()
	var ws = app._world_workspace()
	if ws != null and ws.has_method("_refresh_right_dock_paint"):
		ws._refresh_right_dock_paint()
	await _frames(8)
	var dock_discard_before: Button = _dock_button("Discard")
	_yes("right dock draws a Discard while a draft is live",
		dock_discard_before != null and not dock_discard_before.disabled,
		"null" if dock_discard_before == null else str(dock_discard_before.disabled))
	_chip("Discard").emit_signal("pressed")
	await _frames(10)
	_eq("draft emptied again", bridge.paint_draft_count(), 0)
	var dock_discard_after: Button = _dock_button("Discard")
	## The bar's Discard refreshed the right dock too: its own Discard must not
	## still be live over an emptied draft.
	_yes("right dock Discard is NOT live over an emptied draft",
		dock_discard_after == null or dock_discard_after.disabled,
		"null" if dock_discard_after == null else str(dock_discard_after.disabled))

	_finish()

func _finish() -> void:
	print("P ==== %s  %d/%d assertions, %d failed ====" % [
		"PASS" if _fail == 0 else "FAIL", _n - _fail, _n, _fail])
	get_tree().quit(_fail)
