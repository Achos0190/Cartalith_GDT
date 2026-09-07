extends Node
## **The right dock, measured against the PC canvas** -- `design/mcp-2026-09-07/
## Cartalith DCC Environment.dc.html`, cited below as `ENV:<line>`.
##
##   Godot_v4.7.1 --headless --path . _rdconform_probe.tscn -- --vp 1920x1080
##   Godot_v4.7.1 --headless --path . _rdconform_probe.tscn -- --vp 1366x768
##   Godot_v4.7.1 --headless --path . _rdconform_probe.tscn -- --vp 2560x1600 --force-touch
##
## **Why this exists at all.** `_ds03fit_probe.gd` sweeps the *rail*, and the
## rail does not drive this dock -- §6 makes its contents follow the
## *selection*. So every one of its rail steps measured the same Sample panel,
## and eight contexts plus six appended tool sections had never been
## width-measured at all when this was written. They are, below, and the
## verdict is that the dock holds `--rdW` at all three bands: the widest body
## minimum found anywhere is 264 (ANNOTATION, at the 280 px LAPTOP band).
##
## Every figure asserted here is typed from the canvas markup, never read back
## off the constant it is checking -- `_dockfit_probe.gd`'s own note on why
## `_ds03fit_probe` could not pin `--ldW` applies to `--rdW` identically.
##
## **The divider gaps are asserted as DRAWN distances**, measured between
## global rects, not as the constants that produce them: `RightDock`'s margins
## are the canvas figures minus `DccWidgets.section()`'s body separation, so a
## probe that read the constants would pass while the pixels were wrong.

# -- Canvas literals, typed from the markup ----------------------------------
const C_RDW := 304        ## `ENV:25`  `--rdW:304px`
const C_RDW_LAPTOP := 280 ## `ENV:1819` `w1366` -> `--rdW:280px`
const C_RDW_TOUCH := 400  ## `ENV:1819` touch -> `--rdW:400px`
const C_RAILW := 40       ## `ENV:25`  `--railW:40px`, the collapsed dock
const C_ROW := 28         ## `ENV:25`  `--row:28px`
const C_FIELD_ROW_H := 22 ## `ENV:961` `min-height:calc(var(--row) - 6px)`
const C_ROW_GAP := 10     ## `ENV:961` `gap:10px`
const C_PAINT_ROW_GAP := 9 ## `ENV:1044` `gap:9px`
const C_HERO_GAP := 2     ## `ENV:955` `gap:2px`
const C_RULE_ABOVE := 10  ## `ENV:955` `padding-bottom:10px`
const C_RULE_BELOW := 9   ## `ENV:955` `margin-bottom:9px`
const C_HUD_PAD_X := 9    ## `ENV:913` `padding:4px 9px`
const C_HUD_PAD_Y := 4
const C_HUD_RADIUS := 6   ## `ENV:913` `border-radius:6px`
const C_SEG_TRACK_PAD := 2 ## `ENV:838` `padding:2px`
const C_SEG_RADIUS := 14   ## `ENV:838` `border-radius:14px`

var _fail := 0
var _vp: SubViewport
var app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _le(name: String, got: float, cap: float) -> void:
	var good := got <= cap + 0.5
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=%.1f cap=%.1f" % [got, cap])

## Every leaf whose own minimum exceeds `cap`, hidden ones included -- a
## collapsed body contributes nothing until a reader opens it.
func _latent(root: Node, cap: float) -> Array:
	var out: Array = []
	if root == null:
		return out
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n is Control and n.get_child_count() == 0:
			var mn := (n as Control).get_combined_minimum_size().x
			if mn > cap:
				var t: Variant = n.get("text")
				out.append("%s min_x=%.0f  %s" % [n.get_class(), mn,
					(String(t).left(50) if t != null else "")])
	return out

## The dock, in one named state. Asserts the invariant `_ds03fit_probe` states
## for the rail and could not reach here: no panel forces its dock open, and no
## latent leaf is waiting to.
func _state(tag: String) -> void:
	await _frames(10)
	var rd := app.get("right_dock") as Control
	var body := app.get("right_dock_body") as Control
	var rw := float(app.get("_right_width"))
	_le("%s: the right dock is not forced open" % tag, rd.size.x, rw)
	_le("%s: nor is its own minimum" % tag, rd.get_combined_minimum_size().x, rw)
	var lat := _latent(body, rw)
	_ok("%s: no latent over-wide leaf" % tag, lat.size(), 0)
	for s in lat:
		print("        ", s)
	print("        body_min=%.0f  budget=%.0f" % [body.get_combined_minimum_size().x, rw])

func _find(root: Node, cls: String, txt: String = "") -> Control:
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n.get_class() == cls and n is Control:
			if txt == "" or String(n.get("text")) == txt:
				return n as Control
	return null

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(1)
		return
	var w := 1920
	var h := 1080
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--vp" and i + 1 < args.size():
			var parts := String(args[i + 1]).split("x")
			w = int(parts[0])
			h = int(parts[1])
			i += 2
		elif args[i] == "--force-touch":
			i += 1
		else:
			print("[FATAL] unrecognised argument: ", args[i])
			get_tree().quit(2)
			return

	_vp = SubViewport.new()
	_vp.size = Vector2i(w, h)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(30)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(20)

	var touch := DccTheme.is_touch()
	var rw := float(app.get("_right_width"))
	print("[VP] %dx%d  touch=%s  _right_width=%.0f" % [w, h, touch, rw])

	print("")
	print("-- `--rdW` itself, as a canvas LITERAL --")
	## The same shape `_dockfit_probe.gd` had to add for `--ldW`: `_right_width`
	## comes from `role_px("w_right_dock")`, so a cap of `_right_width` moves
	## with the constant and stays green at any value. These do not.
	var want_rdw := C_RDW_TOUCH if touch else (C_RDW if w >= 1920 else C_RDW_LAPTOP)
	_ok("W_RIGHT_DOCK is the canvas's --rdW (ENV:25)", DccTheme.W_RIGHT_DOCK, C_RDW)
	_ok("and the shipped dock is that number at this band (ENV:1819)", rw, float(want_rdw))
	_ok("the collapsed dock is --railW (ENV:25)", DccTheme.W_RAIL_COLLAPSED, C_RAILW)

	print("")
	print("-- the HUD chips (ENV:913 `padding:4px 9px;border-radius:6px`) --")
	_ok("HUD_PAD_X", ViewportHost.HUD_PAD_X, C_HUD_PAD_X)
	_ok("HUD_PAD_Y", ViewportHost.HUD_PAD_Y, C_HUD_PAD_Y)
	_ok("HUD_RADIUS", ViewportHost.HUD_RADIUS, C_HUD_RADIUS)

	print("")
	print("-- CONTEXT SWEEP, no world --")
	var rdc = app.get("right_dock_ctrl")
	rdc.call("_rebuild")
	await _state("sample/no-world")

	app.call("_run_pipeline")
	var waited := 0
	while app.bridge.generating and waited < 2400:
		await get_tree().process_frame
		waited += 1
	print("[GEN] has_world=%s after %d frames" % [app.bridge.has_world, waited])
	await _frames(10)
	if not app.bridge.has_world:
		print("[FATAL] generate failed")
		get_tree().quit(1)
		return

	print("")
	print("-- CONTEXT SWEEP, world live --")
	rdc.set("_context", "sample")
	rdc.call("_rebuild")
	await _state("sample")
	var setts: Array = app.bridge.settlements()
	if not setts.is_empty():
		rdc.call("on_settlement_selected", setts[0], 0)
		await _state("settlement")
	var facs: Array = app.bridge.get_factions()
	if not facs.is_empty():
		rdc.call("show_faction", int((facs[0] as Dictionary).get("id", 0)))
		await _state("faction")
	rdc.call("show_history")
	await _state("history")
	rdc.set("_context", "region")
	rdc.set("_region_result", {"x0": 10, "y0": 10, "x1": 200, "y1": 160})
	rdc.call("_rebuild")
	await _state("region")
	for m in ["distance", "bearing", "area", "radius", "vertical", "section"]:
		rdc.set("_context", "measure")
		rdc.set("_measure_result", {"total_km": 1234.5, "projected_km2": 91000.0,
			"radius_km": 210.0, "delta_m": -430.0, "length_km": 780.0,
			"segments": [{"km": 400.0, "bearing": 43.0}, {"km": 834.5, "bearing": 191.0}]})
		rdc.set("_measure_mode", m)
		rdc.call("_rebuild")
		await _state("measure/" + m)

	print("")
	print("-- APPENDED TOOL SECTIONS --")
	rdc.set("_context", "sample")
	for t in ["paint", "sculpt", "label", "icon", "territory", "way", "route"]:
		app.call("arm_tool", t)
		await _frames(6)
		await _state("tool:%s -> %s" % [t, rdc.call("_tool_section")])
	app.call("arm_tool", "inspect")
	await _frames(6)

	print("")
	print("-- the key/value row, DRAWN (ENV:961) --")
	rdc.set("_context", "sample")
	rdc.call("_rebuild")
	await _frames(10)
	var elev: Label = rdc.get("_sample_elev")
	_ok("the sample hero readout was found", elev != null, true)
	var pos: Label = rdc.get("_sample_pos")
	if pos != null:
		var row := pos.get_parent() as HBoxContainer
		_ok("row separation is the canvas gap", row.get_theme_constant("separation"), C_ROW_GAP)
		## The drawn gap, not the constant: the label ends here and the value
		## starts there, and the distance between them is what a reader sees.
		var key := row.get_child(0) as Control
		var val := row.get_child(1) as Control
		_ok("and the DRAWN key-to-value gap is that number",
			"%.0f" % (val.get_global_rect().position.x - key.get_global_rect().end.x),
			str(C_ROW_GAP))
		if not touch:
			_ok("row min height is --row - 6", int(row.custom_minimum_size.y), C_FIELD_ROW_H)

	print("")
	print("-- the hero block and its closing hairline, DRAWN (ENV:955) --")
	if elev != null:
		var wrap := elev.get_parent() as VBoxContainer
		var caption := wrap.get_child(0) as Control
		_ok("caption-to-number gap is the canvas gap",
			"%.0f" % (elev.get_global_rect().position.y - caption.get_global_rect().end.y),
			str(C_HERO_GAP))
		var col := wrap.get_parent() as Control
		var idx := wrap.get_index()
		var div: Control = null
		if idx + 1 < col.get_child_count():
			div = col.get_child(idx + 1) as Control
		_ok("a divider follows the hero block", div is MarginContainer, true)
		if div is MarginContainer:
			var line := div.get_child(0) as ColorRect
			_ok("and it is drawn --div, not --hair (ENV:955)",
				line.color, DccTheme.c("line_soft"))
			_ok("the DRAWN gap above the rule",
				"%.0f" % (line.get_global_rect().position.y - wrap.get_global_rect().end.y),
				str(C_RULE_ABOVE))
			if idx + 2 < col.get_child_count():
				var after := col.get_child(idx + 2) as Control
				_ok("the DRAWN gap below the rule",
					"%.0f" % (after.get_global_rect().position.y - line.get_global_rect().end.y),
					str(C_RULE_BELOW))

	## The NEGATIVE control: the readouts the canvas does NOT close with a rule
	## must not have grown one. Way is `ENV:1166` -- a key/value row with a
	## `500 20px` accent value and nothing under it.
	## The tool has to be ARMED for the section to append at all --
	## `_tool_section()` rule 7 reads `app.armed_tool` and the draft, and
	## calling `show_way()` alone leaves the first half false.
	app.call("arm_tool", "way")
	rdc.call("show_way", "way", PackedVector2Array([Vector2(4, 4), Vector2(9, 9)]), "Way")
	await _frames(12)
	var way := _find(app.get("right_dock_body"), "Label", "Waypoints")
	_ok("the Way readout was found", way != null, true)
	if way != null:
		var wwrap := way.get_parent() as Control
		var wcol := wwrap.get_parent() as Control
		var widx := wwrap.get_index()
		var nxt: Control = null
		if widx + 1 < wcol.get_child_count():
			nxt = wcol.get_child(widx + 1) as Control
		var grew: bool = nxt is MarginContainer and nxt.get_child_count() > 0 \
			and nxt.get_child(0) is ColorRect
		_ok("and it has NO divider (ENV:1166 draws none)", grew, false)
	app.call("arm_tool", "inspect")
	await _frames(4)

	print("")
	print("-- the paint legend row is NOT the key/value row (ENV:1044) --")
	rdc.set("_context", "sample")
	app.call("arm_tool", "paint")
	await _frames(12)
	var swatch: Control = null
	var st2: Array = [app.get("right_dock_body")]
	while not st2.is_empty():
		var n: Node = st2.pop_back()
		for c in n.get_children(true):
			st2.append(c)
		if n is ColorRect and (n as Control).custom_minimum_size == Vector2(10, 10):
			swatch = n as Control
	_ok("a paint legend swatch was found", swatch != null, true)
	if swatch != null:
		var prow := swatch.get_parent() as HBoxContainer
		_ok("paint row separation", prow.get_theme_constant("separation"), C_PAINT_ROW_GAP)
		if not touch:
			_ok("paint row min height is a full --row", int(prow.custom_minimum_size.y), C_ROW)
	app.call("arm_tool", "inspect")
	await _frames(4)

	print("")
	print("-- the CIVIL segment track (ENV:838) --")
	## Built into a host this probe owns rather than found in the panel column,
	## for `_ds03fit_probe.gd`'s own reason: the religion section lives inside a
	## collapsed `group()`, and a control that has never been laid out reports
	## stale rects -- the first version of this block measured a chip gap of
	## **-70** off exactly that. An injected control is laid out, so the DRAWN
	## gap below is a real distance.
	var civ: Node = null
	for wnode in app.get("_workspaces"):
		if String(wnode.name) == "CivilizationWorkspace":
			civ = wnode
	_ok("CivilizationWorkspace was found", civ != null, true)
	if civ != null:
		var host := VBoxContainer.new()
		host.custom_minimum_size = Vector2(C_RDW, 0)
		host.size = Vector2(C_RDW, 60)
		add_child(host)
		civ.call("_religion_segments", host)
		await _frames(12)
		var track := _find(host, "PanelContainer") as PanelContainer
		_ok("the religion segment track was built", track != null, true)
		if track != null:
			var sb2 := track.get_theme_stylebox("panel") as StyleBoxFlat
			_ok("track radius (ENV:838)", sb2.corner_radius_top_left, C_SEG_RADIUS)
			_ok("track padding (ENV:838)", int(sb2.content_margin_left), C_SEG_TRACK_PAD)
			## The DRAWN padding, not just the stylebox field: the first chip's
			## left edge minus the track's own.
			var chips := track.get_child(0) as HBoxContainer
			_ok("the chips abut", chips.get_theme_constant("separation"), 0)
			if chips.get_child_count() >= 2:
				var a := chips.get_child(0) as Control
				var b := chips.get_child(1) as Control
				_ok("the DRAWN padding left of the first chip",
					"%.0f" % (a.get_global_rect().position.x - track.get_global_rect().position.x),
					str(C_SEG_TRACK_PAD))
				_ok("and the DRAWN gap between two chips is zero",
					"%.0f" % (b.get_global_rect().position.x - a.get_global_rect().end.x), "0")
		host.queue_free()

	print("")
	print("_rdconform_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
