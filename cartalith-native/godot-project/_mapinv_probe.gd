extends Node

## `dcc_shell.gd::PHONE_DETENT_ANIM` is 0.28 s; this is that plus a margin.
## Pinned to the shell's own literal rather than to the constant, so a change
## there fails this probe instead of silently racing it again.
const PHONE_DETENT_SETTLE := 0.40
## Lane MAP, 2026-09-07. **A measurement, not a feature.** Inventories what the
## phone MAP tab actually puts on glass, against `tabIsMap` in
## `design/Cartalith-Android-2026-09-07.dc.html` -- for `ANDROID_UI_SPEC.md` 2.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##       _mapinv_probe.tscn -- --force-touch --vp 1080x2340 --tag map
##
## Flags this probe reads, grepped from the body below:
##   `--vp WxH`      SubViewport size in physical px. Default 1080x2340.
##   `--tag NAME`    prefix on every output line. Default `map`.
##   `--force-touch` NOT read here -- `dcc_shell.gd` reads it from
##                   `OS.get_cmdline_user_args()`, and without it the shell
##                   boots desktop and every line below is void.
##
## Headless is sound: nothing here reads a pixel. Every quantity is node state
## (`Control.size`, `.visible`, `Button.disabled`, `Label.text`) which the
## layout pass computes with no renderer.
##
## **Reachability is measured by hit-testing, not by reading `.visible`.** A
## control can be `visible` and still sit under an opaque sibling, off-screen,
## or be `MOUSE_FILTER_IGNORE`. `_reachable()` asks the real GUI tree instead.

var app: Node
var _vp: SubViewport
var _tag := "map"

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _reject_unknown_args() -> bool:
	var known := ["--force-touch", "--vp", "--tag"]
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true

func _press(at: Vector2, down: bool) -> void:
	var e := InputEventMouseButton.new()
	e.button_index = MOUSE_BUTTON_LEFT
	e.pressed = down
	e.position = at
	e.global_position = at
	_vp.push_input(e, true)

func _tap(at: Vector2) -> void:
	_press(at, true)
	await _frames(2)
	_press(at, false)
	await _frames(4)

## Every `Control` under `root`, depth first.
func _controls(root: Node, out: Array) -> Array:
	for c in root.get_children():
		if c is Control:
			out.append(c as Control)
		_controls(c, out)
	return out

## Is this control the topmost thing the viewport would hand a press at its own
## centre to? The real test for "can a finger get there", as against `.visible`.
func _reachable(c: Control) -> bool:
	if not c.is_visible_in_tree() or c.size.x < 1.0 or c.size.y < 1.0:
		return false
	var p := c.get_global_rect().get_center()
	if p.x < 0 or p.y < 0 or p.x >= float(_vp.size.x) or p.y >= float(_vp.size.y):
		return false
	var m := InputEventMouseMotion.new()
	m.position = p
	m.global_position = p
	_vp.push_input(m, true)
	var hit: Control = _vp.gui_get_hovered_control()
	if hit == null:
		return false
	var n: Node = hit
	while n != null:
		if n == c:
			return true
		n = n.get_parent()
	return false

func _dp(px: float) -> float:
	return px / float(app.phone_scale())

## Every pressable a finger could actually hit right now.
func _pressables() -> Array:
	var out: Array = []
	for c in _controls(app, []):
		if c is BaseButton and (c as BaseButton).is_visible_in_tree():
			var b := c as BaseButton
			var txt := ""
			if b is Button:
				txt = (b as Button).text
			if txt == "":
				txt = b.tooltip_text
			out.append({"t": txt, "dis": b.disabled, "hit": _reachable(b),
				"sz": b.size})
	return out

func _sheet_text() -> Array:
	var out: Array = []
	var row: Control = app.tool_options_row
	if row == null:
		return out
	for c in _controls(row, [row]):
		if c is Label:
			out.append("LABEL  " + (c as Label).text)
		elif c is Button:
			out.append("BUTTON " + (c as Button).text)
		elif c is LineEdit:
			out.append("EDIT   " + (c as LineEdit).text)
	return out

func _ready() -> void:
	_tag = _arg("--tag", "map")
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "1080x2340").split("x")
	if parts.size() != 2:
		_log("ABORT --vp wants WxH")
		get_tree().quit(2)
		return
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	if not app.is_phone():
		_log("ABORT not phone mode -- pass --force-touch")
		get_tree().quit(2)
		return
	_log("=== vp=%s scale=%.4f ===" % [str(_vp.size), app.phone_scale()])

	# ---------------------------------------------------------------- 1
	_log("--- 1: launch state, before any tap ---")
	_log("  lit tab              = %s" % str(app._phone_tab))
	_log("  detent               = %s" % str(app.phone_detent()))
	_log("  armed tool           = %s" % str(app.armed_tool))
	_log("  domain               = %s" % str(app.active_domain()))
	var pre := _pressables()
	var pre_hit := 0
	for p in pre:
		if p["hit"] and not p["dis"]:
			pre_hit += 1
	_log("  pressables total=%d  finger-reachable+enabled=%d" % [pre.size(), pre_hit])
	for p in pre:
		if p["hit"]:
			_log("    HIT  %-28s dis=%s  %s" % [p["t"], str(p["dis"]), str(p["sz"])])

	# ---------------------------------------------------------------- 2
	_log("--- 2: tap MAP through the real GUI tree ---")
	var cell: Dictionary = app._phone_tab_cells.get("map", {})
	var pill: Control = cell.get("pill")
	if pill == null:
		_log("  ABORT no MAP cell")
		get_tree().quit(2)
		return
	await _tap(pill.get_global_rect().get_center())
	## **Wait for the detent tween, then ASSERT the tap landed.** Two defects
	## lived in the four lines below, and both published as measured fact.
	##
	## 1. The sheet height was read DURING `dcc_shell.gd`'s 0.28 s detent tween
	##    (`PHONE_DETENT_ANIM`, on `custom_minimum_size:y`). Six runs at this
	##    probe's own command line gave 592, 589, 575, 574, 506 and 510 px --
	##    an 86 px / 33 dp spread, and never the 609 px that `ANDROID_UI_SPEC.md`
	##    quotes in two places. The dp arithmetic was self-consistent
	##    (609 / 2.6214 = 232.3), which is what made a mid-tween sample
	##    convincing.
	##
	## 2. Under a real display driver the tap does not land at all: the lit tab
	##    stays `gen` and section 3 then reports GENERATE's text nodes as if
	##    they were the MAP sheet's. **The probe asserted nothing about its own
	##    precondition**, so it reported a confident wrong screen.
	##
	## The wait is 0.28 s plus a margin, then frames -- a tween finishing is a
	## time fact, not a frame-count fact, and `_frames()` alone raced it.
	await get_tree().create_timer(PHONE_DETENT_SETTLE).timeout
	await _frames(3)
	if str(app._phone_tab) != "map":
		_log("  ABORT the MAP tap did not land -- lit tab is %s, so every figure"
			% str(app._phone_tab))
		_log("  below would describe the wrong sheet. Section 3's text nodes are")
		_log("  the giveaway: GENERATE's, not MAP's.")
		get_tree().quit(2)
		return
	_log("  lit tab              = %s" % str(app._phone_tab))
	_log("  detent               = %s" % str(app.phone_detent()))
	_log("  domain               = %s" % str(app.active_domain()))
	_log("  armed tool           = %s" % str(app.armed_tool))
	var sheet: Control = app._phone_tool_sheet
	_log("  sheet h              = %.1f px  %.2f dp" % [sheet.size.y, _dp(sheet.size.y)])
	_log("  tool scroll visible  = %s" % str(app._phone_tool_scroll.visible))
	_log("  gen  scroll visible  = %s" % str(app._phone_gen_scroll.visible))

	# ---------------------------------------------------------------- 3
	_log("--- 3: EVERY text node in the MAP sheet ---")
	var body := _sheet_text()
	_log("  %d text nodes in tool_options_row" % body.size())
	for s in body:
		_log("    " + s)
	_log("  %d Controls under tool_options_row" % _controls(app.tool_options_row, []).size())

	# ---------------------------------------------------------------- 4
	_log("--- 4: what a finger can press with MAP lit ---")
	var post := _pressables()
	var buried := 0
	for p in post:
		if p["hit"]:
			_log("    HIT  %-28s dis=%s" % [p["t"], str(p["dis"])])
		else:
			buried += 1
	_log("  %d pressables exist but are NOT hit-testable here" % buried)

	# ---------------------------------------------------------------- 5
	_log("--- 5: the canvas's four mapTools, against arm_tool ---")
	for id in ["inspect", "measure", "label", "icon"]:
		var found := ""
		for p in post:
			if p["hit"] and String(p["t"]).to_lower().contains(id):
				found = String(p["t"])
		_log("    %-9s reachable-by-tap=%s  %s" % [id, str(found != ""), found])

	# ---------------------------------------------------------------- 6
	_log("--- 6: LayersPopover -- the layer rows the canvas draws in the sheet ---")
	var host: ViewportHost = app.get("viewport")
	var lb: Control = host.get("_layers_btn")
	_log("  layers button visible=%s hit=%s" % [
		str(lb != null and lb.is_visible_in_tree()),
		str(lb != null and _reachable(lb))])
	var pop = app.get("layers_popover")
	if pop != null:
		pop.call("open")
		await _frames(8)
		var rows: Dictionary = pop.get("_rows")
		_log("  popover visible=%s rows=%d" % [str(pop.visible), rows.size()])
		for want in ["off", "bclass", "control", "elevation", "slope", "flow", "temp", "rain"]:
			_log("    canvas row -> engine id %-10s present=%s" % [want, str(rows.has(want))])
		pop.hide()
		await _frames(3)

	# ---------------------------------------------------------------- 7
	_log("--- 7: STYLE -- ramps and looks, and whether the phone reaches them ---")
	var b: EngineBridge = app.get("bridge")
	_log("  ramp_api=%s  presets=%s" % [str(b.ramp_api), str(b.ramp_presets())])
	_log("  look_api=%s  looks=%s" % [str(b.look_api), str(b.looks())])
	_log("  preset_api=%s" % str(b.preset_api))

	# ---------------------------------------------------------------- 8
	## Does the SCULPT/PAINT/MEASURE segment (`tool_bar.gd::_build_mode_buttons`)
	## ever become hit-testable? It is the only control that arms Measure, and
	## `_on_tool_armed` draws it only while one of the three is ALREADY armed --
	## so this asks whether that circle can be broken from a phone surface.
	_log("--- 8: the one control that arms MEASURE ---")
	for ctx in [["map lit, nothing else", ""], ["after arm_tool(paint)", "paint"],
			["after arm_tool(paint) then re-tap MAP", "retap"]]:
		if String(ctx[1]) == "paint":
			app.arm_tool("paint")
			await _frames(6)
		elif String(ctx[1]) == "retap":
			await _tap(pill.get_global_rect().get_center())
		var seg_hit := false
		var seg_vis := false
		for p in _pressables():
			if String(p["t"]) == "MEASURE":
				seg_vis = true
				if p["hit"]:
					seg_hit = true
		_log("    %-38s MEASURE chip exists=%s hit=%s  (tool=%s, toolscroll=%s)" % [
			String(ctx[0]), str(seg_vis), str(seg_hit), str(app.armed_tool),
			str(app._phone_tool_scroll.visible)])

	get_tree().quit(0)
