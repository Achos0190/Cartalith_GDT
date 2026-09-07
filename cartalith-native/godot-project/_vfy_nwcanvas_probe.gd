extends Node
## VERIFIER probe (2026-09-07). Two questions the lane's own `_nwsize_probe.gd`
## does not answer, kept apart because they have different owners:
##
## 1. **A hard 1-to-1 element diff against the canvas.** Every visible control
##    inside the phone New World dialog is dumped in tree order with its dp
##    geometry, and the canvas's own list (`design/
##    Cartalith-Android-2026-09-07.dc.html`, the `modalOpen` block: NAME, SEED
##    plus dice, EXTENT chips, CANCEL, CREATE WORLD) is then checked item by
##    item. A count is not a comparison, so each canvas row gets its own line.
## 2. **Does the km field COMMIT?** A field that displays and does not reach the
##    engine is the seed defect already on file. This types a value no preset
##    holds, presses Create, waits for `generation_finished`, and reads the
##    world's extent back out of `WorldGen` -- not out of the field.
##
##   godot --path . _vfy_nwcanvas_probe.tscn -- --force-touch --vp 1080x2340
##
## Flags actually read below: `--vp WxH` (default 1080x2340), `--tag NAME`
## (default vfy). `--force-touch` is read by `dcc_shell.gd`, not here, and
## without it every line is void. Run WINDOWED: nothing samples a pixel, but
## the GUI hit-test wants a frame loop.

var app: Node
var _vp: SubViewport
var _tag := "vfy"
var _fail := 0
var _gen_ok := -1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	return String(args[i + 1]) if (i >= 0 and i + 1 < args.size()) else dflt

func _dp(c: Control, px: float) -> float:
	var f: float = c.get_viewport().get_final_transform().x.x
	return px * maxf(0.001, f) / maxf(0.001, float(app.phone_scale()))

func _walk(root: Node, out: Array, depth: int) -> void:
	## `include_internal = true`: `AcceptDialog` adds its buttons hbox with
	## `INTERNAL_MODE_FRONT`, so a default `get_children()` walk cannot see
	## CANCEL or Create at all and reports them missing. Cost one run.
	for c in root.get_children(true):
		if c is Control and not (c as Control).is_visible_in_tree():
			continue
		if c is Control:
			out.append({"c": c, "d": depth})
		_walk(c, out, depth + 1)

func _caption(c: Control) -> String:
	if c is OptionButton:
		return "[%s]" % (c as OptionButton).text
	if c is Button:
		return (c as Button).text
	if c is Label:
		return (c as Label).text
	if c is LineEdit:
		return "<LineEdit %s ph=%s>" % [(c as LineEdit).text, (c as LineEdit).placeholder_text]
	if c is SpinBox:
		return "<SpinBox %s>" % str((c as SpinBox).value)
	return ""

func _ready() -> void:
	_tag = _arg("--tag", "vfy")
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--force-touch", "--vp", "--tag"]):
			_log("ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	var parts: PackedStringArray = _arg("--vp", "1080x2340").split("x")
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.8).timeout
	await _frames(4)
	_log("viewport %dx%d phone=%s scale=%.3f" % [_vp.size.x, _vp.size.y, app.is_phone(), app.phone_scale()])
	if not app.is_phone():
		_check(false, "not a phone at this viewport")
		_log("RESULT %s fail=%d" % [_tag, _fail])
		get_tree().quit(1)
		return

	## Boot state, before ANY input reaches the shell.
	var dlg: Window = app.new_world_dialog
	_check(not dlg.visible, "boot: New World dialog closed before any input")
	_check(app.phone_project_picker != null and app.phone_project_picker.visible,
		"boot: the project picker is what the app comes up in")
	_check(float(app.bridge.last_width_km) == 0.0,
		"boot: no world yet (last_width_km %s)" % str(app.bridge.last_width_km))

	await _leg_inventory(dlg)
	await _leg_commit(dlg)
	await _leg_tap_to_set()
	_log("RESULT %s fail=%d" % [_tag, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

# -- Leg 1: the canvas diff ----------------------------------------------------

func _leg_inventory(dlg: Window) -> void:
	_log("-- ordered inventory of the phone New World dialog")
	## Staging call, and labelled as one: the lane established that a synthetic
	## press cannot be routed into an embedded AcceptDialog sub-window, and this
	## leg is about CONTENT and GEOMETRY, which a synthetic open does not distort.
	app.open_new_world()
	await _frames(20)
	_check(dlg.visible, "the dialog is up")
	if not dlg.visible:
		return
	var out: Array = []
	_walk(dlg, out, 0)
	for e in out:
		var c: Control = e["c"]
		var cap := _caption(c)
		if cap.is_empty():
			continue
		var r := c.get_global_rect()
		_log("    %s%-14s @x%-4.0f %5.0f x %-5.0f dp  %s" % ["  ".repeat(int(e["d"])),
			c.get_class(), _dp(c, r.position.x), _dp(c, r.size.x), _dp(c, r.size.y), cap])

	## The canvas's own list, row by row.
	var caps: Array = []
	var lines: Array = []
	for e in out:
		var c: Control = e["c"]
		var cap := _caption(c)
		if not cap.is_empty():
			lines.append(cap)
		if c is Button:
			caps.append(String((c as Button).text))
	_check(lines.has("New world") or lines.has("NEW WORLD"),
		"canvas NEW WORLD title -> a readable title exists")
	## A free-text input, i.e. one that is NOT a `SpinBox`'s own internal line
	## edit. Including internal children makes every spin box look like a text
	## field otherwise, which is the opposite answer.
	var free_text: Array = []
	for e in out:
		var c: Control = e["c"]
		if c is LineEdit and not (c.get_parent() is SpinBox):
			free_text.append(String(c.name))
	_check(free_text.is_empty(),
		"canvas NAME + text input -> NO free-text field on the card %s (declared departure, spec 6.3)"
		% str(free_text))
	_check(lines.has("Seed"), "canvas SEED -> Seed")
	var dice: Button = null
	for e in out:
		var c: Control = e["c"]
		if c is Button and String((c as Button).accessibility_name) == "New seed":
			dice = c
	_check(dice != null, "canvas dice 44x42 -> a New seed button exists")
	if dice != null:
		var dr := dice.get_global_rect()
		_log("    dice %.0f x %.0f dp (canvas 44 x 42)" % [_dp(dice, dr.size.x), _dp(dice, dr.size.y)])
	var region := false
	var world := false
	for cap in caps:
		if cap == "REGION":
			region = true
		if cap == "WORLD":
			world = true
	_check(region and world, "canvas EXTENT REGION|WORLD chips -> both drawn")
	_check(caps.has("Cancel") or caps.has("CANCEL"), "canvas CANCEL -> a cancel button exists")
	_check(caps.has("Create") or caps.has("CREATE WORLD"), "canvas CREATE WORLD -> a create button exists")
	_log("  button captions: %s" % str(caps))

# -- Leg 2: does the km field reach the engine? --------------------------------

func _leg_commit(dlg: Window) -> void:
	_log("-- does Width (km) COMMIT to the engine?")
	var width: SpinBox = dlg.get("width_input")
	var res: OptionButton = dlg.get("resolution_input")
	var preset: OptionButton = dlg.get("size_preset_input")
	## A value NO preset holds, so a pass cannot come from a default.
	res.selected = 0
	res.item_selected.emit(0)
	await _frames(2)
	width.value = 1234.0
	width.value_changed.emit(1234.0)
	await _frames(3)
	_log("  field now: Width %s km, preset %s, resolution %s, grid %s x %s" %
		[str(width.value), preset.text, res.text,
		str((dlg.get("grid_w_input") as SpinBox).value), str((dlg.get("grid_h_input") as SpinBox).value)])
	var req: Dictionary = dlg.request()
	_check(is_equal_approx(float(req["width_km"]), 1234.0),
		"request() carries width_km = %s" % str(req["width_km"]))

	app.bridge.generation_finished.connect(func(ok: bool): _gen_ok = 1 if ok else 0)
	## The press. A tap is attempted first; the fallback is labelled.
	var accept: AcceptDialog = dlg as AcceptDialog
	var ok_btn: Button = accept.get_ok_button()
	var vp: Viewport = ok_btn.get_viewport()
	var rect: Rect2 = ok_btn.get_global_rect()
	var at: Vector2 = vp.get_final_transform() * (rect.position + rect.size * 0.5)
	for down in [true, false]:
		var e := InputEventMouseButton.new()
		e.button_index = MOUSE_BUTTON_LEFT
		e.pressed = down
		e.position = at
		e.global_position = at
		vp.push_input(e, false)
		await _frames(1)
	await _frames(6)
	var by_tap := bool(app.bridge.generating) or _gen_ok >= 0
	if not by_tap:
		_log("  (the synthetic tap did not reach the sub-window button -- pressing it by call)")
		ok_btn.pressed.emit()
		await _frames(6)
	_log("  create route: %s" % ("synthetic tap" if by_tap else "labelled call"))
	var waited := 0
	while _gen_ok < 0 and waited < 3000:
		await _frames(1)
		waited += 1
	_check(_gen_ok == 1, "generation finished ok (%d, after %d frames)" % [_gen_ok, waited])
	var wk: float = float(app.bridge.last_width_km)
	var hk: float = float(app.bridge.last_height_km)
	_log("  engine reports %.3f km x %.3f km" % [wk, hk])
	_check(is_equal_approx(wk, 1234.0),
		"the engine's OWN map width is the typed 1234 km, not the 800 km default (%.3f)" % wk)

# -- Leg 3: the one behaviour the withholding could have broken -----------------

## **Tap-to-set.** `PgSlider` swallows the touch-DOWN, which is what Godot's own
## `Slider` used to set the value from -- so a plain tap on the track is the
## regression risk of the fix, and neither the lane's probe nor the on-glass log
## exercised one. It should still land, applied at RELEASE instead of at press.
var _ends := 0

func _count_end(_c: bool) -> void:
	_ends += 1

func _first_pg(root: Node) -> HSlider:
	for c in root.get_children(true):
		if c is HSlider and (c as HSlider).get_script() != null:
			return c as HSlider
		var r := _first_pg(c)
		if r != null:
			return r
	return null

func _leg_tap_to_set() -> void:
	_log("-- a plain TAP on the track still sets the value")
	if app.phone_project_picker != null and app.phone_project_picker.visible:
		app.phone_project_picker.hide()
		await _frames(4)
	var tab: Label = null
	for e in _tree_controls():
		var c: Control = e
		if c is Label and String((c as Label).text) == "GENERATE":
			tab = c
	if tab == null:
		_check(false, "the GENERATE tab is findable")
		return
	var tvp: Viewport = tab.get_viewport()
	var trect: Rect2 = tab.get_global_rect()
	var tat: Vector2 = tvp.get_final_transform() * (trect.position + trect.size * 0.5)
	for down in [true, false]:
		var e2 := InputEventMouseButton.new()
		e2.button_index = MOUSE_BUTTON_LEFT
		e2.pressed = down
		e2.position = tat
		e2.global_position = tat
		tvp.push_input(e2, false)
		await _frames(1)
	await _frames(8)
	var scroll: ScrollContainer = app._phone_gen_scroll
	_check(scroll != null and scroll.visible, "the GENERATE column is up")
	if scroll == null or not scroll.visible:
		return
	var sl := _first_pg(scroll)
	_check(sl != null, "a PgSlider is present")
	if sl == null:
		return
	scroll.ensure_control_visible(sl)
	await _frames(4)
	sl.drag_ended.connect(_count_end)
	var before: float = sl.value
	var r: Rect2 = sl.get_global_rect()
	var at: Vector2 = Vector2(r.position.x + r.size.x * 0.85, r.position.y + r.size.y * 0.5)
	_ends = 0
	for down in [true, false]:
		var e3 := InputEventMouseButton.new()
		e3.button_index = MOUSE_BUTTON_LEFT
		e3.pressed = down
		e3.position = at
		e3.global_position = at
		_vp.push_input(e3, true)
		await _frames(2)
	await _frames(6)
	var after: float = sl.value if is_instance_valid(sl) else NAN
	_log("  tap at 85%% of the track: %s -> %s, drag_ended x%d (alive=%s)"
		% [str(before), str(after), _ends, is_instance_valid(sl)])
	_check(_ends == 1, "a tap emits exactly one drag_ended, so it still commits (%d)" % _ends)
	_check(not is_instance_valid(sl) or after != before,
		"and the value moved (%s -> %s)" % [str(before), str(after)])

func _tree_controls() -> Array:
	var out: Array = []
	var raw: Array = []
	_walk(_vp, raw, 0)
	for e in raw:
		out.append(e["c"])
	return out
