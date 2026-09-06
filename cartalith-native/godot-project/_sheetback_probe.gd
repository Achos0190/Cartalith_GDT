extends Node
## Splits the two claims in the `ANDROID_BUILD_SCOPE.md` device row
## *"the left panel sheet retains its scroll offset across close/reopen and
## would not scroll back up"* (2026-08-24 09:54, OnePlus 6T) and identifies
## the three `size=(0.0, 115.0)` buttons `_phonechrome_probe.gd` reports.
##
## Run WINDOWED, not `--headless`: `ScrollContainer`'s drag-to-scroll is gated
## on `DisplayServer.is_touchscreen_available()` and this probe prints that
## flag beside every measurement rather than assuming it.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 393x852
##       _sheetback_probe.tscn -- --force-touch
##   Godot_v4.7.1-stable_win64_console.exe --path .
##       _sheetback_probe.tscn -- --force-touch --sub 1080x2340
##
## Two arguments are read HERE, and nothing else is accepted -- an unknown one
## aborts rather than being silently ignored, so a stale flag in a brief
## cannot quietly measure the wrong thing:
##   `--force-touch`  read by `DccShell._ready()`, not by this file; without it
##                    `_phone` can never be true in this dev environment.
##   `--sub WxH`      host the shell in a `SubViewport` of that size. Required
##                    for the handset density: a windowed run asked for
##                    1080x2340 is clamped to the desktop it opens on
##                    (measured on this box -- the shell came up NOT phone),
##                    and `--resolution` is the only other lever there is.
##
## Section A  -- claim 1, "retains its scroll offset across close/reopen".
## Section A2 -- the same point flicked four times in a row. **The first
##   synthesised flick after a (re)open always reads delta 0 with no hovered
##   control and the next three scroll**, so a one-shot flick probe here
##   measures its own warm-up. Separated deliberately: without it the dead
##   reading looks positional and gets written up as a defect.
## Section B  -- claim 2, "would not scroll back up", flicked in BOTH
##   directions at three x positions, because the existing PH-05 probe
##   (`_scrolldrag_probe.gd`) only ever flicks the finger upward.
## Section B2 -- twelve x positions across one row, to prove a dead flick is
##   not a dead band.
## Section C  -- the three unlaid buttons: which subtree they are in, named
##   against `DccShell`'s own `_phone_root` child fields rather than by the
##   anonymous `@Button@214` path the failing probe can print, then measured
##   again with that subtree actually shown.

var _scroll: ScrollContainer
var _app: Node
var _vp: Viewport  ## Where input is pushed and hover is read: the root window,
                   ## or the `--sub WxH` SubViewport when one is asked for.
var _local := false

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _find_scroll(n: Node) -> ScrollContainer:
	if n is ScrollContainer:
		return n
	for c in n.get_children():
		var s := _find_scroll(c)
		if s != null:
			return s
	return null

func _under(at: Vector2) -> String:
	var hover := InputEventMouseMotion.new()
	hover.position = at
	_vp.push_input(hover, _local)
	await get_tree().process_frame
	var over := _vp.gui_get_hovered_control()
	if over == null:
		return "<none>"
	return "%s filter=%d text=%s" % [over.get_class(), over.mouse_filter,
		str(over.get("text")).strip_edges().left(22)]

## `step` is the per-sample finger delta in PHYSICAL px: negative moves the
## finger up (content scrolls DOWN, `scroll_vertical` rises), positive moves
## it down (content scrolls UP, `scroll_vertical` falls). Eight samples, the
## same shape `_scrolldrag_probe.gd` uses, so the two are comparable.
func _flick(at: Vector2, step: float, start: int) -> Array:
	_scroll.scroll_vertical = start
	await _frames(1)
	var before := _scroll.scroll_vertical
	var who := await _under(at)
	var mb := InputEventMouseButton.new()
	mb.button_index = MOUSE_BUTTON_LEFT
	mb.pressed = true
	mb.position = at
	_vp.push_input(mb, _local)
	var p := at
	for i in 8:
		p += Vector2(0, step)
		var mm := InputEventMouseMotion.new()
		mm.position = p
		mm.relative = Vector2(0, step)
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT
		_vp.push_input(mm, _local)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = p
	_vp.push_input(up, _local)
	await _frames(1)
	return [before, _scroll.scroll_vertical, who]

func _ready() -> void:
	Input.set_emulate_touch_from_mouse(true)
	## `--sub WxH`: host the shell in a `SubViewport` of that size instead of
	## the root window. Needed for the handset density -- a windowed run asked
	## for 1080x2340 is CLAMPED to the desktop it opens on (measured on this
	## box: the shell came up non-phone at that request), and
	## `_compute_layout_mode()` reads `get_viewport_rect().size`, which for a
	## node inside a `SubViewport` is that viewport own size. Same device the
	## `_phonechrome_probe.gd` uses, and the reason it uses it.
	var args := OS.get_cmdline_user_args()
	var sub := Vector2i.ZERO
	var i := args.find("--sub")
	if i >= 0 and i + 1 < args.size():
		var wh: PackedStringArray = args[i + 1].split("x")
		if wh.size() == 2:
			sub = Vector2i(int(wh[0]), int(wh[1]))
	## Every argument is either recognised or fatal. `--sub`'s own value is
	## consumed with it, so `1080x2340` is not itself an unknown flag.
	var skip := -1
	for j in args.size():
		if j == skip:
			continue
		if args[j] == "--sub":
			skip = j + 1
			continue
		if args[j] != "--force-touch":
			print("[ABORT] unrecognised argument '", args[j],
				"' -- this probe reads --force-touch and --sub WxH")
			get_tree().quit(1)
			return
	_app = load("res://shell/app.tscn").instantiate()
	if sub != Vector2i.ZERO:
		var v := SubViewport.new()
		v.size = sub
		v.gui_embed_subwindows = true
		v.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		add_child(v)
		v.add_child(_app)
		_vp = v
		_local = true
	else:
		add_child(_app)
		_vp = get_viewport()
	await get_tree().create_timer(1.2).timeout
	if _app.get("open_project_dialog") != null:
		_app.open_project_dialog.hide()
	await _frames(2)

	if not bool(_app.is_phone()):
		print("[ABORT] not the phone composition -- pass `-- --force-touch`")
		get_tree().quit(1)
		return
	var scale: float = float(_app.phone_scale())
	print("[DENSITY] viewport=%s  _phone_scale=%.4f  _pscale(44)=%s physical px  touchscreen=%s" % [
		_vp.get_visible_rect().size, scale,
		_app.call("_pscale", 44), DisplayServer.is_touchscreen_available()])

	_app._set_sheet_open("left", true)
	await get_tree().create_timer(0.5).timeout
	_scroll = _find_scroll(_app.left_dock)
	if _scroll == null:
		print("[ABORT] no ScrollContainer under left_dock")
		get_tree().quit(1)
		return
	var rect := _scroll.get_global_rect()
	var max_v := int(_scroll.get_v_scroll_bar().max_value - _scroll.size.y)
	print("[SHEET] scroll rect=%s  deadzone=%d  reachable max scroll=%d" % [
		rect, _scroll.scroll_deadzone, max_v])

	print("\n=== A: claim 1 -- does the offset survive close/reopen? ===")
	_scroll.scroll_vertical = max_v
	await _frames(2)
	var at_bottom := _scroll.scroll_vertical
	_app._set_sheet_open("left", false)
	await _frames(3)
	_app._set_sheet_open("left", true)
	await _frames(5)
	print("  scrolled to %d, closed, reopened -> scroll_vertical=%d %s" % [
		at_bottom, _scroll.scroll_vertical,
		"(claim 1 NOT reproduced)" if _scroll.scroll_vertical == 0 else "(claim 1 REPRODUCED)"])

	print("\n=== A2: is the dead flick the POINT or the FIRST-of-run? ===")
	## Every run so far has exactly one flick reading 0, and it is always
	## iteration 1 of section B -- which is also the first synthesised drag
	## after section A reopened the sheet. Same point, four times in a row,
	## settles which of the two it is without another guess.
	for k in 4:
		var f: Array = await _flick(Vector2(rect.position.x + rect.size.x * 0.2,
			rect.position.y + 24.0), -12.0 * scale, 0)
		print("     attempt %d  delta=%+5d  under=%s" % [k + 1, int(f[1]) - int(f[0]), f[2]])

	print("\n=== B: claim 2 -- flick both directions, three x positions ===")
	## Three x positions across the sheet, matching the device pass's own
	## "three x positions", plus a y sweep -- because what a drag lands on is
	## position-dependent and the whole question is what eats it.
	var xs := [rect.position.x + rect.size.x * 0.2,
		rect.get_center().x, rect.position.x + rect.size.x * 0.8]
	var stepf := 12.0 * scale  ## the same 12 authored px per sample, at this density
	for tag in [["UP-flick (finger up, scroll_vertical must RISE)", -stepf, 0],
			["DOWN-flick (finger down, scroll_vertical must FALL)", stepf, max_v]]:
		print("  -- %s, start=%d, step=%+.0f px x 8" % [tag[0], tag[2], tag[1]])
		var y := rect.position.y + 24.0
		while y < rect.end.y - 24.0:
			for x in xs:
				var r: Array = await _flick(Vector2(x, y), float(tag[1]), int(tag[2]))
				var moved: int = int(r[1]) - int(r[0])
				print("     x=%4d y=%4d  %5d -> %5d  delta=%+5d  under=%s" % [
					int(x), int(y), int(r[0]), int(r[1]), moved, r[2]])
			y += 260.0

	print("\n=== B2: the single flick point that scrolled 0 -- how wide is it? ===")
	print("  scroll mouse_filter=%d  v_scrollbar visible=%s rect=%s" % [
		_scroll.mouse_filter, _scroll.get_v_scroll_bar().visible,
		_scroll.get_v_scroll_bar().get_global_rect()])
	var y0 := rect.position.y + 24.0
	var xr := rect.position.x + 4.0
	while xr < rect.end.x:
		var rr: Array = await _flick(Vector2(xr, y0), -stepf, 0)
		print("     x=%4d y=%4d  delta=%+5d  under=%s" % [
			int(xr), int(y0), int(rr[1]) - int(rr[0]), rr[2]])
		xr += rect.size.x / 12.0

	print("\n=== C: the three size=(0.0, 115.0) buttons ===")
	var root: Control = _app.get("_phone_root")
	## Named against the fields `dcc_shell.gd` itself holds, so the answer is
	## a subtree name and not another anonymous path.
	var named := {}
	for f in ["_phone_chrome_margin", "_phone_side_safe", "_phone_search_overlay",
			"_phone_menu", "_phone_overflow_pop", "_phone_tool_sheet",
			"_phone_menu_bar", "_phone_undo_chip", "left_dock", "right_dock"]:
		var v = _app.get(f)
		if v is Node:
			named[(v as Node).get_instance_id()] = f
	var idx := 0
	for child in root.get_children():
		var who: String = named.get(child.get_instance_id(), "<unnamed>")
		print("  _phone_root[%d] %-22s name=%-20s %-16s visible=%s vis_in_tree=%s" % [
			idx, who, child.name, child.get_class(), child.visible,
			(child as Control).is_visible_in_tree() if child is Control else "-"])
		idx += 1
	print("  -- the offenders, popover CLOSED (the state the failing probe walks) --")
	_hunt(root, [])
	## The half that decides whether this is a measurement gap or a real
	## width-floor violation hiding behind one. `_set_phone_overflow_open()` is
	## the function that shows this subtree; every probe warm-up in the tree
	## calls `_set_overflow_open()`, which is a DIFFERENT function -- it opens
	## `PhoneMenu`'s L2 root and never touches `_phone_overflow_pop`.
	print("  -- the same three, after `_set_phone_overflow_open(true)` --")
	_app.call("_set_phone_overflow_open", true)
	await _frames(6)
	var floor_px: float = float(_app.call("_pscale", 44))
	var rows: Array[Button] = []
	var pop = _app.get("_phone_overflow_pop")
	if pop == null:
		print("     <_phone_overflow_pop is null>")
	else:
		_collect_buttons(pop, rows)
	for b in rows:
		print("     size=%s cms=%s  floor=%s  width_ok=%s height_ok=%s  a11y=%s" % [
			b.size, b.custom_minimum_size, floor_px,
			b.size.x >= floor_px - 0.5, b.size.y >= floor_px - 0.5,
			b.accessibility_name])
	_app.call("_close_all_phone_overlays")
	await _frames(2)
	get_tree().quit()

func _collect_buttons(n: Node, out: Array[Button]) -> void:
	if n is Button:
		out.append(n)
	for c in n.get_children():
		_collect_buttons(c, out)

func _hunt(n: Node, chain: Array) -> void:
	var here: Array = chain.duplicate()
	here.append(n)
	if n is Button and (n as Button).size.x <= 0.5 and (n as Button).size.y > 0.5 \
			and (n as BaseButton).mouse_filter == Control.MOUSE_FILTER_STOP:
		var parts := PackedStringArray()
		for a in here:
			parts.append("%s:%s%s" % [a.name, a.get_class(),
				"" if (a as Node).get_script() == null else "<" + (a as Node).get_script().resource_path.get_file() + ">"])
		print("     size=%s cms=%s icon=%s vis_in_tree=%s owner_visible=%s" % [
			(n as Button).size, (n as Button).custom_minimum_size,
			"null" if (n as Button).icon == null else str((n as Button).icon.get_size()),
			(n as Button).is_visible_in_tree(),
			_first_hidden(n)])
		print("     chain: ", " > ".join(parts))
	for c in n.get_children():
		_hunt(c, here)

## Which ancestor is the one that is `visible = false` -- the thing that keeps
## the subtree from ever getting a container sort. Names the class and, where
## the node carries one, its script, so the answer routes to a file.
func _first_hidden(n: Node) -> String:
	var p: Node = n
	while p != null:
		if p is Control and not (p as Control).visible:
			return "%s(%s)" % [p.get_class(), p.name]
		p = p.get_parent()
	return "<none hidden>"
