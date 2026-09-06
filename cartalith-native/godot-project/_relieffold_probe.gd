extends Node
## The Colour relief fold, read back off both real docks in a booted app.
##
## `TerrainAppearance::ramp_strength` ships at `0.0` and `LayerStack::composite`
## skips Colour relief while its ramp contributes nothing, so at the shipped
## default that row's dot, opacity, blend and order were four live controls over
## a layer measured (`_rampstrength_probe.gd`, windowed, three seeds) to move
## **zero** bytes. The judgement taken 2026-09-06 was to fold the row to its name
## and its state until the ramp has a strength, in `right_dock.gd`'s appended
## Layers section and in `render_workspace.gd`'s Terrain raster stack **both** --
## so this probe asserts both, and asserts them in both directions.
##
## Reasoning from `_relief_is_dark` would prove nothing: the question is what is
## on screen, and whether raising the slider puts the controls back **without a
## second gesture**. That last part is the one that could silently rot --
## `set_appearance` carries no signal of its own, so the crossing is routed
## through `layer_stack_changed`, and if that emit is ever dropped the fold goes
## stale in whichever dock the user is not looking at.
##
## The strength is moved through the **real slider handle** and its own
## `drag_ended`, not by calling `bridge.set_appearance` -- otherwise this would
## test the setter and not the wiring.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _relieffold_probe.tscn

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("FOLD %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

# -- Reading the docks back ---------------------------------------------------

func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children():
		_walk(c, out)

func _nodes(root: Node) -> Array:
	var out: Array = []
	if root != null:
		_walk(root, out)
	return out

func _labels(root: Node) -> Array:
	var out: Array = []
	for n in _nodes(root):
		if n is Label:
			out.append((n as Label).text)
	return out

## Buttons whose text is exactly one of `wanted`. The reorder pair is the
## cheapest proof that a row is a full row: `_reorder`/`_reorder_button` are the
## only things in either dock that draw a bare "Up"/"Down".
func _button_count(root: Node, wanted: Array) -> int:
	var n := 0
	for x in _nodes(root):
		if x is Button and wanted.has(String((x as Button).text)):
			n += 1
	return n

func _count_text(root: Node, needle: String) -> int:
	var n := 0
	for t in _labels(root):
		if String(t) == needle:
			n += 1
	return n

func _sliders(root: Node) -> int:
	var n := 0
	for x in _nodes(root):
		if x is HSlider:
			n += 1
	return n

# -- One state, asserted on both docks ----------------------------------------

## `folded` says which state the shell should be in. Every number below is
## stated for both states rather than for the one being tested, because a
## check that only knows the failing side cannot tell a fold from an empty dock.
func _assert_state(tag: String, folded: bool, rw: Node) -> void:
	var rd_body: Node = app.right_dock_body
	var stack: Node = rw._layer_host

	_check("%s: the left dock's stack host exists" % tag, stack != null)
	if stack == null:
		return

	# -- Right dock (`right_dock.gd::_append_layers`) -------------------------
	var rd_reorder := _button_count(rd_body, ["Up", "Down"])
	var rd_marks := _count_text(rd_body, "not drawing")
	_check("%s: right dock draws %d reorder buttons" % [tag, 4 if folded else 6],
		rd_reorder == (4 if folded else 6), "got %d" % rd_reorder)
	_check("%s: right dock shows the folded state %s" % [tag, "once" if folded else "not at all"],
		rd_marks == (1 if folded else 0), "got %d" % rd_marks)
	_check("%s: right dock still names Colour relief either way" % tag,
		_labels(rd_body).has("Colour relief"), "labels=%s" % [_labels(rd_body).size()])

	# -- Left dock (`render_workspace.gd::_build_layer_stack`) ----------------
	var lw_reorder := _button_count(stack, ["Up", "Down"])
	var lw_marks := _count_text(stack, "not drawing")
	var lw_sliders := _sliders(stack)
	_check("%s: left dock draws %d reorder buttons" % [tag, 4 if folded else 6],
		lw_reorder == (4 if folded else 6), "got %d" % lw_reorder)
	_check("%s: left dock shows the folded state %s" % [tag, "once" if folded else "not at all"],
		lw_marks == (1 if folded else 0), "got %d" % lw_marks)
	_check("%s: left dock draws %d opacity sliders" % [tag, 2 if folded else 3],
		lw_sliders == (2 if folded else 3), "got %d" % lw_sliders)
	_check("%s: left dock still names Colour relief either way" % tag,
		_labels(stack).has("Colour relief"), "labels=%s" % [_labels(stack)])

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 1800:
		await get_tree().process_frame
		waited += 1
	print("FOLD world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("FOLD  !! generate failed -- nothing else here can run")
		get_tree().quit(1)
		return

	## CARTO, because `_append_layers` is domain-gated on it and the left dock's
	## stack lives in CARTO's own Layers category.
	app.select_domain("cartography")
	await _frames(6)

	## Named beside the numbers rather than assumed: both docks size their rows
	## off `DccTheme.is_tablet()`, and a count taken at one density is a claim
	## about that density only.
	print("FOLD density: %s, viewport %s" % [
		"touch/tablet" if DccTheme.is_tablet() else "pointer",
		str(get_viewport().get_visible_rect().size)])

	var carto: Node = null
	for ws in app._workspaces:
		if ws is CartographyWorkspace:
			carto = ws
			break
	_check("the CARTO workspace is registered", carto != null)
	if carto == null:
		get_tree().quit(1)
		return
	var rw: Node = carto._render
	_check("its RenderWorkspace child exists", rw != null)
	if rw == null:
		get_tree().quit(1)
		return

	_check("the engine ships ramp_strength at 0.0",
		float(app.bridge.appearance().get("ramp_strength", -1.0)) == 0.0,
		"got %s" % str(app.bridge.appearance().get("ramp_strength", "<absent>")))
	await _assert_state("default", true, rw)

	# -- Raise it through the real slider, not through the setter --------------
	var row: Dictionary = rw._app_rows.get("ramp_strength", {})
	_check("the ramp strength slider is built and remembered", not row.is_empty())
	if row.is_empty():
		get_tree().quit(1)
		return
	var handle: Dictionary = row["handle"]
	var s: HSlider = handle["slider"]
	var scale := float(row["scale"])
	s.value = 0.35 * scale
	s.drag_ended.emit(true)
	await _frames(6)
	_check("the slider reached the engine",
		absf(float(app.bridge.appearance().get("ramp_strength", -1.0)) - 0.35) < 0.005,
		"engine now %s" % str(app.bridge.appearance().get("ramp_strength", "<absent>")))
	## The point of the whole crossing-emit: no second gesture, no dock switch.
	await _assert_state("strength 0.35", false, rw)

	# -- And back. The transition INTO the folded state, not just out of it ----
	var row2: Dictionary = rw._app_rows.get("ramp_strength", {})
	var s2: HSlider = (row2["handle"] as Dictionary)["slider"]
	s2.value = 0.0
	s2.drag_ended.emit(true)
	await _frames(6)
	_check("the slider took it back to zero",
		float(app.bridge.appearance().get("ramp_strength", -1.0)) == 0.0,
		"engine now %s" % str(app.bridge.appearance().get("ramp_strength", "<absent>")))
	await _assert_state("back to 0.00", true, rw)

	print("")
	print("FOLD %s -- %d failure(s)" % ["ALL PASS" if _fail == 0 else "FAILURES", _fail])
	get_tree().quit(1 if _fail > 0 else 0)
