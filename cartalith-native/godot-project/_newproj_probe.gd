extends Node
## The desktop new-project flow, end to end: boot `app.tscn`, wait for the
## welcome dialog `_open_welcome_when_drawn()` presents, press its
## `+ New world...` button, press the New World dialog's OK (Create), wait for
## `generation_finished`, then run the shell for a while and screenshot it.
##
## Protects the owner's 2026-09-28 report "starting a new project causes the
## app to crash/stall". Every stage is timestamped so a stall names the stage
## it stopped in; a crash shows as a missing `RESULT` line and a non-zero exit.
##
## Buttons are pressed with `pressed.emit()` -- this is a wiring and stability
## check of the route, not a claim about hit-testing (MISTAKES.md: `emit()` is
## not a finger).
##
##   godot --path . _newproj_probe.tscn -- --out DIR [flags]
##
## Flags read below (grepped from the body, not assumed):
##   `--out DIR`          screenshot folder (default `user://`)
##   `--settle-frames N`  frames run after the last generate (default 600)
##   `--res N`            grid columns to request (default: the dialog's own)
##   `--repeat N`         new projects in a row; round 2+ opens the dialog
##                        over the open world, as `File ▸ New world` does
##   `--hover 1`          mouse motion over the map during each generate
##   `--click 1`          left clicks on the map during each generate
##   `--zoom 1`           wheel notches over the map during each generate
##   `--real-click 1`     click `New world` and `Create` as a mouse does
##                        (`Input` events at their drawn centres) instead of
##                        `pressed.emit()`; fails if the dialog does not open or
##                        Create starts no generate. This is the leg that
##                        caught the 1 x 1 New World dialog (2026-09-28)
##   `--trace-size 1`     print a stack on every New World dialog resize
##   `--touch-engine 1`   NEGATIVE CONTROL: calls a `WorldGen` `#[func]`
##                        mid-generate on purpose; it panics and aborts this
##                        probe's coroutine, so the run then hangs by design
## **The pass bar is not only `RESULT PASS`.** A `#[func]` reached during a
## generate panics (`Gd<T>::bind() failed, already bound`) without failing
## anything GDScript can see, so the run's stderr must also carry zero
## `in WorldGen::` lines. Measured 2026-09-28 with `--hover 1 --click 1
## --repeat 2`: 168 before `viewport_host.gd::_generating()`, 0 after.
## Run WINDOWED: `frame_post_draw` never fires headless, so the welcome
## dialog would never be presented.

var app: Node
var _t0 := 0
var _finished := -1

func _log(s: String) -> void:
	print("[np %7.2fs] %s" % [float(Time.get_ticks_msec() - _t0) / 1000.0, s])

func _arg(name: String, dflt: String) -> String:
	var a := OS.get_cmdline_user_args()
	var i := a.find(name)
	return String(a[i + 1]) if i >= 0 and i + 1 < a.size() else dflt

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _find_button(root: Node, needle: String) -> Button:
	for c in root.get_children():
		if c is Button and String((c as Button).text).contains(needle):
			return c
		var r := _find_button(c, needle)
		if r != null:
			return r
	return null

## A real left click at `b`'s on-screen centre, through `Input` so it takes
## the same route a mouse does (hit-test, the embedded dialog window, focus).
## The dialogs are embedded subwindows (the project keeps Godot's default), so
## the root-space point is the window's position plus the control's
## window-local centre scaled by the window's final transform.
func _real_click(b: Control, what: String) -> void:
	## Let the popup finish laying out and placing itself: a rect read in the
	## frame it opened is the pre-layout one (measured: `92,22` against the
	## drawn `421,417`).
	await _frames(20)
	var vp := b.get_viewport()
	var at: Vector2 = vp.get_final_transform() * (b.get_global_rect().get_center())
	if vp is Window and vp != get_tree().root:
		at += Vector2((vp as Window).position)
	_log("real click on %s at %s" % [what, str(at)])
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	Input.parse_input_event(mm)
	await _frames(2)
	for down in [true, false]:
		var e := InputEventMouseButton.new()
		e.button_index = MOUSE_BUTTON_LEFT
		e.pressed = down
		e.position = at
		e.global_position = at
		Input.parse_input_event(e)
		await _frames(2)
	await _frames(3)

func _ready() -> void:
	_t0 = Time.get_ticks_msec()
	if DisplayServer.get_name() == "headless":
		_log("ABORT run windowed")
		get_tree().quit(2)
		return
	var out := _arg("--out", OS.get_user_data_dir())
	var settle := int(_arg("--settle-frames", "600"))
	_log("boot app.tscn")
	app = load("res://shell/app.tscn").instantiate()
	if _arg("--trace-size", "") == "1":
		get_tree().node_added.connect(func(n: Node):
			if n is NewWorldDialog:
				(n as Window).size_changed.connect(func():
					print("[trace] NewWorldDialog size -> %s" % (n as Window).size)
					print_stack()))
	add_child(app)
	var dlg: Node = null
	for i in 600:
		await get_tree().process_frame
		if app.open_project_dialog != null and app.open_project_dialog.visible:
			dlg = app.open_project_dialog
			break
	if dlg == null:
		_log("FAIL welcome dialog never appeared")
		get_tree().quit(1)
		return
	_log("welcome dialog up; new-world dialog size before any click %s" % app.new_world_dialog.size)
	app.new_world_dialog.size_changed.connect(func():
		_log("  new-world dialog size -> %s" % app.new_world_dialog.size)
		if app.new_world_dialog.size.x < 50:
			print_stack())
	var nb := _find_button(dlg, "New world")
	if nb == null:
		_log("FAIL no `New world` button in the welcome dialog")
		get_tree().quit(1)
		return
	var real := _arg("--real-click", "") == "1"
	if real:
		await _real_click(nb, "welcome `New world`")
	else:
		nb.pressed.emit()
	await _frames(10)
	var nw: AcceptDialog = app.new_world_dialog
	_log("after New world: welcome visible=%s  new-world dialog visible=%s" % [dlg.visible, nw.visible])
	_log("  root %s  dialog pos %s size %s  ok rect %s" % [get_viewport().get_visible_rect().size,
		nw.position, nw.size, nw.get_ok_button().get_global_rect()])
	_log("  contents min %s  min_size %s  wrap %s  max_size %s  children %d" % [
		nw.get_contents_minimum_size(), nw.min_size, nw.wrap_controls, nw.max_size, nw.get_child_count()])
	for ch in nw.get_children(true):
		if ch is Control:
			_log("    child %s %s vis=%s min=%s size=%s" % [ch.get_class(), ch.name,
				(ch as Control).visible, (ch as Control).get_combined_minimum_size(), (ch as Control).size])
	if real:
		await RenderingServer.frame_post_draw
		get_viewport().get_texture().get_image().save_png(_arg("--out", OS.get_user_data_dir()).path_join("dialog.png"))
	if real and not nw.visible:
		_log("FAIL the New World dialog did not open from a real click")
		_log("RESULT FAIL")
		get_tree().quit(1)
		return
	app.bridge.generation_stage.connect(func(i: int, n: String, _t: int):
		_log("stage %d %s" % [i, n]))
	app.bridge.generation_finished.connect(func(ok: bool):
		_finished = 1 if ok else 0
		_log("generation_finished ok=%s" % ok))
	## Time every `generation_finished` handler: each connection is swapped for
	## a wrapper that calls it and logs how long it took.
	for sig_name in ["generation_finished", "params_applied"]:
		var sig: Signal = app.bridge.get(sig_name)
		for c: Dictionary in sig.get_connections():
			var cb: Callable = c["callable"]
			if cb.get_object() == self:
				continue
			sig.disconnect(cb)
			var label := "%s -> %s.%s" % [sig_name, cb.get_object().get_class() if cb.get_object() != null else "?", cb.get_method()]
			if cb.get_object() is Node:
				label += " (%s)" % (cb.get_object() as Node).name
			if sig_name == "generation_finished":
				sig.connect(func(ok: bool):
					var t := Time.get_ticks_usec()
					cb.call(ok)
					var ms := float(Time.get_ticks_usec() - t) / 1000.0
					if ms > 20.0:
						_log("  handler %.0f ms  %s" % [ms, label]), c["flags"])
			else:
				sig.connect(func():
					var t := Time.get_ticks_usec()
					cb.call()
					var ms := float(Time.get_ticks_usec() - t) / 1000.0
					if ms > 20.0:
						_log("  handler %.0f ms  %s" % [ms, label]), c["flags"])
	var res := int(_arg("--res", "0"))
	var repeat := int(_arg("--repeat", "1"))
	for round_i in repeat:
		if round_i > 0:
			## A second new project over the open one, through the menu's own call.
			_log("round %d: open_new_world() over the open world" % round_i)
			app.open_new_world()
			await _frames(10)
		if res > 0:
			nw.grid_w_input.value = res
			nw.grid_w_input.value_changed.emit(float(res))
			await _frames(2)
		_log("new world dialog visible=%s  request=%s" % [nw.visible, str(nw.request())])
		_finished = -1
		_log("press Create")
		var t_create := Time.get_ticks_msec()
		if real:
			await _real_click(nw.get_ok_button(), "New World `%s`" % nw.get_ok_button().text)
		else:
			nw.get_ok_button().pressed.emit()
		var fmax := 0.0
		if _arg("--touch-engine", "") == "1":
			## What a hover or any main-thread reader does mid-generation: one
			## plain `#[func]` read on the object the worker has borrowed.
			await get_tree().create_timer(3.0).timeout
			_log("touch: calling world_gen.get_width() mid-generation")
			var tt := Time.get_ticks_usec()
			var w = app.bridge.world_gen.get_width()
			_log("touch: returned %s after %.0f ms" % [str(w), float(Time.get_ticks_usec() - tt) / 1000.0])
		if _arg("--hover", "") == "1":
			await get_tree().create_timer(3.0).timeout
			var vr := get_viewport().get_visible_rect()
			for k in 60:
				var mm := InputEventMouseMotion.new()
				mm.position = vr.size * Vector2(0.5 + 0.002 * k, 0.5)
				mm.global_position = mm.position
				var th := Time.get_ticks_usec()
				Input.parse_input_event(mm)
				await get_tree().process_frame
				var dt := float(Time.get_ticks_usec() - th) / 1000.0
				if dt > 250.0:
					_log("hover: frame after motion %d took %.0f ms" % [k, dt])
			_log("hover: done")
		if _arg("--zoom", "") == "1":
			## Wheel notches and a drag over the map mid-generation: what a person
			## does while waiting. Real input through the viewport, not a call.
			await get_tree().create_timer(2.0).timeout
			var vr2 := get_viewport().get_visible_rect()
			var at := vr2.size * Vector2(0.5, 0.5)
			for k in 12:
				for down in [true, false]:
					var wb := InputEventMouseButton.new()
					wb.button_index = MOUSE_BUTTON_WHEEL_UP if k < 6 else MOUSE_BUTTON_WHEEL_DOWN
					wb.pressed = down
					wb.factor = 1.0
					wb.position = at
					wb.global_position = at
					Input.parse_input_event(wb)
				var tz := Time.get_ticks_usec()
				await _frames(3)
				var dz := float(Time.get_ticks_usec() - tz) / 1000.0
				if dz > 250.0:
					_log("zoom: notch %d frames took %.0f ms" % [k, dz])
			_log("zoom: done")
		if _arg("--click", "") == "1":
			## Left clicks on the map mid-generation (select, pick, sample).
			var vr3 := get_viewport().get_visible_rect()
			for k in 8:
				var cp := vr3.size * Vector2(0.4 + 0.03 * k, 0.45 + 0.02 * k)
				for down in [true, false]:
					var cb := InputEventMouseButton.new()
					cb.button_index = MOUSE_BUTTON_LEFT
					cb.pressed = down
					cb.position = cp
					cb.global_position = cp
					Input.parse_input_event(cb)
					await _frames(2)
			_log("click: done")
		var last_g := Time.get_ticks_usec()
		while _finished < 0:
			await get_tree().process_frame
			var nowg := Time.get_ticks_usec()
			fmax = maxf(fmax, float(nowg - last_g) / 1000.0)
			if nowg - last_g > 250000:
				_log("  main thread blocked %.0f ms during generation" % (float(nowg - last_g) / 1000.0))
			last_g = nowg
			if _finished < 0 and not app.bridge.generating \
					and Time.get_ticks_msec() - t_create > 5000:
				_log("FAIL Create started no generate in 5 s (dialog visible=%s)" % nw.visible)
				_log("RESULT FAIL")
				get_tree().quit(1)
				return
			if Time.get_ticks_msec() - _t0 > 1200000:
				_log("FAIL generation did not finish in 1200 s")
				get_tree().quit(1)
				return
		_log("round %d: longest main-thread frame during generation %.0f ms" % [round_i, fmax])
		if round_i + 1 < repeat:
			await _frames(120)
	_log("settling %d frames" % settle)
	var worst := 0.0
	var worst_at := -1
	var over := 0
	var last := Time.get_ticks_usec()
	for f in settle:
		await get_tree().process_frame
		var now := Time.get_ticks_usec()
		var ms := float(now - last) / 1000.0
		last = now
		if ms > worst:
			worst = ms
			worst_at = f
		if ms > 100.0:
			over += 1
			_log("  frame %d took %.0f ms" % [f, ms])
	_log("settled: worst frame %.0f ms at %d; %d frames over 100 ms" % [worst, worst_at, over])
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	var path := out.path_join("newproj.png")
	img.save_png(path)
	_log("has_world=%s  screenshot %s" % [app.bridge.has_world, path])
	var ok: bool = _finished == 1 and app.bridge.has_world
	_log("RESULT %s" % ("PASS" if ok else "FAIL"))
	get_tree().quit(0 if ok else 1)
