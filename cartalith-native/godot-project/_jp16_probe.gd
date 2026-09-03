extends Node
## Focused verification harness for `GUI_GAP_REGISTER.md` PH-16's second half
## (Lane B, phone stage 6): the Journey Planner's phone centre panel.
##
## Scoped to this one screen on purpose. `_ph9_probe.gd`'s nine-window sweep
## reaches the journey-planner section it already has (line ~131-135) only
## after opening Performance / Gen info / World data / Travel library / Data
## manager / Asset library / Asset slicer / Layers, and as of 2026-09-03 it
## SCRIPT ERRORs before any of that on a stale `app._set_drawer_open()` call
## (`dcc_shell.gd`'s own "☰ opens a domain drawer" was superseded by the 412
## canvas's full-screen `02 Domain` drill -- `DCC_SHELL_SPEC.md` §13's own
## superseded box) -- `_set_drawer_open` no longer exists on `DccApp`, only in
## a comment. That is a pre-existing defect in a probe this file does not own
## (`_ph9_probe.gd` is a shared harness, not part of this lane's four files),
## reported rather than fixed here; this probe does not depend on it.
##
##   godot4 --path . --headless --resolution 1080x2400 _jp16_probe.tscn -- --force-touch
##
## What it reports, in order:
##   - phone=/scale() from DccApp itself, so a run that silently landed in
##     desktop mode is caught before anything below is trusted.
##   - `_center_panel`'s own global rect. Expected to span the full phone
##     screen edge to edge -- `journey_planner_view.gd::_build_center_panel()`
##     parents it to `app.viewport_content`, which `dcc_shell.gd::
##     _build_phone_shell()` anchors `PRESET_FULL_RECT` under the SAME
##     floating chrome the live map's own `vp` uses, for the same reason
##     (`DCC_SHELL_SPEC.md` §13: "map draws edge-to-edge behind every inset").
##     That is the designed footprint, not the PH-16 defect -- included here
##     so a future reader has the number rather than having to re-derive it.
##   - `app.viewport.visible` while Journey is armed. Phone must read `true`:
##     the map has to stay live *behind* the panel per §13, never switched
##     off, which was PH-16's second still-true fact.
##   - every `PanelContainer` inside `_center_panel`, with whether its rect's
##     bottom edge falls past the screen -- the direct, mechanical symptom a
##     `phone_scale()` double-application produces (a row inflated to
##     `phone_scale()^2` its authored height pushes everything after it off
##     the bottom, per this file's own history at
##     `journey_planner_view.gd::_build_center_panel()`'s header comment).
##   - a row-by-row luminance scan of `_center_panel`'s own screen rect, the
##     same method `GUI_GAP_REGISTER.md`'s PH-16 finding used ("scanned every
##     fifth row, full width... not one pixel exceeds RGB(23,23,23)"), run
##     here to prove the opposite: that real content -- panel rules, header
##     labels, the ⚠/route-map cutout -- paints somewhere in that rect rather
##     than a uniform near-black fill consuming the whole screen.
##
## No world is generated and no route is committed -- this replicates the
## exact state PH-16's own capture was taken in ("No committed route
## selected"), which is also the state that exercises the STRUCTURAL defect
## (row heights), since every row below is built regardless of route
## selection; only the numbers inside totals/profile/stops/inspector/matrix
## depend on a route existing. Content-richness with a real route committed
## is not covered here -- committing one needs map clicks this harness does
## not attempt.

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _find_all(node: Node, pred: Callable, out: Array) -> Array:
	if pred.call(node):
		out.append(node)
	for c in node.get_children():
		_find_all(c, pred, out)
	return out

func _ready() -> void:
	Input.set_emulate_touch_from_mouse(true)
	## Same clamp-avoidance dance as `_ph9_probe.gd::_ready()`'s own comment
	## explains: `--resolution` is clamped to the monitor's usable rect on
	## Windows, which silently turns a 1080x2400 handset run into a desktop
	## one and makes `_compute_layout_mode()` report `phone=false`. Set
	## explicitly, after boot, where nothing clamps it.
	var want := Vector2i(1080, 2400)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)
	print("ds_window_size=", DisplayServer.window_get_size(), " root.size=", get_tree().root.size)

	var app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	## `app.gd::open_welcome()` shows a cold-start welcome screen whenever no
	## world exists, over everything -- and phone gets a DIFFERENT node from
	## desktop/tablet (`app.gd`'s own doc on `open_welcome()`): desktop uses
	## `open_project_dialog`'s welcome mode, phone uses `phone_project_picker`
	## instead. `_shot_phone.gd`'s committed `--nowelcome` handling hides only
	## the former, so it has this exact same gap on a phone capture -- found
	## here, not fixed there (that probe is not one of this lane's four
	## files). Missing this the first two runs: the screenshot was the
	## picker's own "worlds on this device" list both times, not the journey
	## planner underneath it -- the rect/visible/scan numbers above were still
	## real (they read `_center_panel` directly, never the framebuffer), but
	## the pixel scan was proving the PICKER paints, not this panel. Hiding
	## both covers whichever one a given run actually opened.
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await get_tree().process_frame
	var screen: Vector2 = app.get_viewport_rect().size
	print("=== phone=", app.is_phone(), " scale=", app.phone_scale(), " screen=", screen, " ===")
	if not app.is_phone():
		print("FAIL: --force-touch did not produce phone mode; nothing below is meaningful")
		get_tree().quit(1)
		return

	app.open_journey_planner()
	await get_tree().create_timer(0.8).timeout

	var jpv = app.journey_planner_view
	var cp: Control = jpv._center_panel
	if cp == null or not is_instance_valid(cp):
		print("FAIL: journey_planner_view._center_panel is null/invalid")
		get_tree().quit(1)
		return
	print("[jp] _center_panel visible=", cp.visible, " rect=", cp.get_global_rect())
	print("[jp] app.viewport.visible=", app.viewport.visible,
		" (phone must stay true -- §13: map covered, not switched off)")

	var pcs: Array = []
	_find_all(cp, func(n): return n is PanelContainer, pcs)
	var any_overflow := false
	for p in pcs:
		var c := p as Control
		var r := c.get_global_rect()
		var overflow: bool = (r.position.y + r.size.y) > screen.y + 0.5
		any_overflow = any_overflow or overflow
		print("[jp] PanelContainer rect=", r, " overflow_bottom=", overflow)
	print("[jp] any PanelContainer overflowing the screen bottom = ", any_overflow)

	# One screenshot, then a row-by-row luminance scan across _center_panel's
	# own on-screen rect -- the exact method GUI_GAP_REGISTER.md's PH-16
	# finding used, run here to prove content rather than absence of it.
	await _frames(3)
	var img := get_viewport().get_texture().get_image()
	var out := "user://jp16_center.png"
	img.save_png(out)
	print("shot ", ProjectSettings.globalize_path(out))

	var rect := cp.get_global_rect()
	var x0 := maxi(0, int(rect.position.x))
	var x1 := mini(img.get_width(), int(rect.position.x + rect.size.x))
	var y0 := maxi(0, int(rect.position.y))
	var y1 := mini(img.get_height(), int(rect.position.y + rect.size.y))
	var blank_rows := 0
	var content_rows := 0
	var first_content_y := -1
	var last_content_y := -1
	var y := y0
	while y < y1:
		var row_has_content := false
		var x := x0
		while x < x1:
			var px := img.get_pixel(x, y)
			if px.r8 > 23 or px.g8 > 23 or px.b8 > 23:
				row_has_content = true
				break
			x += 5
		if row_has_content:
			content_rows += 1
			if first_content_y < 0:
				first_content_y = y
			last_content_y = y
		else:
			blank_rows += 1
		y += 5
	print("[jp] scan(every 5th row, x step 5) panel_y=[", y0, ",", y1, "] blank_rows=", blank_rows,
		" content_rows=", content_rows, " first_content_y=", first_content_y,
		" last_content_y=", last_content_y)

	print("=== done ===")
	get_tree().quit()
