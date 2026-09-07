extends Node
## VERIFIER probe, 2026-09-07. Three questions the batch's own probes do not
## answer, asked independently:
##
## 1. **Does any `phone_menu.gd::_slider_row()` slider exist in the live tree?**
##    `_rangeswipe_probe.gd` labels one census "MORE > Simulation (the Year
##    slider's screen)" and that census counts ZERO `PhoneMenu` Ranges. This
##    opens MORE > Simulation and prints what the body actually holds.
## 2. **What does the `editable == false` gate cost?** The gate hands a disabled
##    slider back to `Slider::gui_input`, which is `MOUSE_FILTER_STOP` -- so the
##    vertical drag may dead-end there and scroll nothing.
## 3. **A control: the same swipe on the same slider while it is editable.**
##
##   godot --path . _vfy_gate_probe.tscn -- --force-touch --vp 1080x2340

var app: Node
var _vp: SubViewport
var _tag := "vfy"

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

func _all(root: Node, out: Array) -> void:
	for c in root.get_children():
		out.append(c)
		_all(c, out)

func _walk_all() -> Array:
	var out: Array = []
	_all(get_tree().root, out)
	return out

func _tap_at(at: Vector2) -> void:
	for down in [true, false]:
		var e := InputEventMouseButton.new()
		e.button_index = MOUSE_BUTTON_LEFT
		e.pressed = down
		e.position = at
		e.global_position = at
		_vp.push_input(e, true)
		await _frames(1)
	await _frames(4)

func _push_path(from: Vector2, offsets: Array) -> void:
	var down := InputEventMouseButton.new()
	down.button_index = MOUSE_BUTTON_LEFT
	down.pressed = true
	down.position = from
	down.global_position = from
	_vp.push_input(down, true)
	await _frames(1)
	var prev := from
	for o in offsets:
		var at: Vector2 = from + (o as Vector2)
		var mm := InputEventMouseMotion.new()
		mm.position = at
		mm.global_position = at
		mm.relative = at - prev
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT
		_vp.push_input(mm, true)
		prev = at
		await _frames(1)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = prev
	up.global_position = prev
	_vp.push_input(up, true)
	await _frames(4)

## The same path `_rangeswipe_probe._jitter()` builds -- restated here rather
## than imported, so this probe does not depend on the file under review.
func _jitter(dir: float, reach: float) -> Array:
	var out: Array = []
	var xs := [3.0, 6.0, 5.0, 8.0, 10.0, 6.0, 4.0, 2.0, 0.0, -2.0, -4.0, -4.0, -3.0]
	var ys := [1.0, 2.0, 4.0, 6.0, 10.0, 20.0, 40.0, 80.0, 140.0, 210.0, 290.0, 380.0, 468.0]
	var span: float = ys[ys.size() - 1]
	for i in xs.size():
		out.append(Vector2(float(xs[i]), dir * float(ys[i]) / span * reach))
	return out

func _scroller_of(n: Node) -> ScrollContainer:
	var p: Node = n.get_parent()
	while p != null:
		if p is ScrollContainer:
			return p as ScrollContainer
		p = p.get_parent()
	return null

func _find_caption(text: String) -> Control:
	for n in _walk_all():
		if n is Label and (n as Label).is_visible_in_tree() \
				and String((n as Label).text) == text:
			return n as Control
	for n in _walk_all():
		if n is Button and (n as Button).is_visible_in_tree() \
				and String((n as Button).text) == text:
			return n as Control
	return null

func _pressable_ancestor(n: Node) -> Button:
	var p := n
	while p != null:
		if p is Button:
			return p as Button
		p = p.get_parent()
	return null

func _tap_caption(text: String) -> bool:
	var t := _find_caption(text)
	if t == null:
		_log("   (no visible caption `%s`)" % text)
		return false
	var cell := _pressable_ancestor(t)
	var at: Vector2 = (cell if cell != null else t).get_global_rect().get_center()
	await _tap_at(at)
	await _frames(8)
	return true

func _phone_menu() -> Node:
	for n in _walk_all():
		if n is PhoneMenu:
			return n
	return null

func _ready() -> void:
	var parts: PackedStringArray = _arg("--vp", "1080x2340").split("x")
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if app.phone_project_picker != null and app.phone_project_picker.visible:
		app.phone_project_picker.hide()
	await _frames(6)
	_log("viewport %dx%d phone=%s scale=%.3f" %
		[_vp.size.x, _vp.size.y, app.is_phone(), app.phone_scale()])

	# -- Q1: is any PhoneMenu Range ever in the tree? --------------------------
	var pm := _phone_menu()
	_log("Q1 PhoneMenu instance in tree at boot: %s" % str(pm != null))
	if await _tap_caption("MORE"):
		pm = _phone_menu()
		_log("Q1 after MORE: PhoneMenu=%s visible=%s" %
			[str(pm != null), str(pm != null and (pm as Control).is_visible_in_tree())])
		if await _tap_caption("Simulation"):
			pm = _phone_menu()
			var ranges := 0
			var vis := 0
			if pm != null:
				var kids: Array = []
				_all(pm, kids)
				for n in kids:
					if n is Range:
						ranges += 1
						if (n as Control).is_visible_in_tree():
							vis += 1
							_log("   PhoneMenu Range %s script=%s slop=%s" %
								[n.get_class(),
								("yes" if n.get_script() != null else "NO"),
								str(n.get("slop"))])
			_log("Q1 MORE > Simulation: PhoneMenu holds %d Range (%d visible in tree)"
				% [ranges, vis])
			_log("Q1 the shell says tl_available()=%s -- the `Year` slider's own gate"
				% str(app.tl_available()) if app.has_method("tl_available") else
				"Q1 (no tl_available on app)")
			var missing := _find_caption("Year")
			_log("Q1 a `Year` caption is on screen: %s" % str(missing != null))
	## Whatever screen MORE left up, get out of it before Q2.
	if _phone_menu() != null and (_phone_menu() as Control).is_visible_in_tree():
		(_phone_menu() as Object).call("close")
		await _frames(6)

	# -- Q2/Q3: the editable gate's cost --------------------------------------
	app._set_sheet_open("left", true)
	await _frames(10)
	var sl: HSlider = null
	for n in _walk_all():
		if n is HSlider and (n as HSlider).is_visible_in_tree():
			var sc0 := _scroller_of(n)
			if sc0 != null and sc0.vertical_scroll_mode \
					!= ScrollContainer.SCROLL_MODE_DISABLED:
				sl = n as HSlider
				break
	if sl == null:
		_log("Q2 no left-dock slider found")
		get_tree().quit(0)
		return
	var sc := _scroller_of(sl)
	for editable in [true, false]:
		sc.ensure_control_visible(sl)
		await _frames(4)
		sl.editable = editable
		await _frames(2)
		var r := sl.get_global_rect()
		var s0 := sc.scroll_vertical
		var v0 := sl.value
		var bar := sc.get_v_scroll_bar()
		var room_up: float = bar.max_value - bar.page - float(s0)
		var dir := -1.0 if room_up > 200.0 else 1.0
		var reach: float = minf(float(_vp.size.y) * 0.20, maxf(120.0, absf(room_up)))
		var start := Vector2(r.position.x + r.size.x * 0.25,
			r.position.y + r.size.y * 0.5)
		await _push_path(start, _jitter(dir, reach))
		_log("Q%d editable=%s script=%s : value %s -> %s   scroll %d -> %d"
			% [2 if editable else 3, str(editable),
				("yes" if sl.get_script() != null else "NO"),
				str(v0), str(sl.value), s0, sc.scroll_vertical])
	sl.editable = true
	get_tree().quit(0)
