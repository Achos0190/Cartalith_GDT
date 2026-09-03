extends Node
## PH-16 remaining-band verifier -- a REAL render, not headless (a headless
## dummy render returns a null texture, so a pixel-level probe needs a real
## GL context; MISTAKES.md's own preflight table).
##
## Band-scans the framebuffer for the longest contiguous run of rows where no
## pixel anywhere in that row (full width, every row -- exhaustive, since this
## is a one-shot verification rather than the coarse every-fifth-row sweep the
## original register entry describes) exceeds RGB(23,23,23), the register's own
## floor. Dark palette is forced and asserted: that floor is palette-bound and
## this machine boots light (see `_force_dark`).
##
## **Four states, because one cannot discriminate.** The register measured only
## "planner open, no world" and read the whole band as this panel's. Run all
## four before believing any of them:
##
##   godot --path . _ph16band_probe.tscn -- --force-touch --nowelcome [flag]
##
##   (none)        planner open, no world      -- the register's own state
##   --nojp        planner never opened        -- the control it never had
##   --withworld   world generated, no route   -- the state the panel owns
##   --withroute   world + a committed route   -- the state a user works in
##
## Measured 2026-09-03 at 1080x2400 -- blank rows / longest band:
##   --nojp 1 494 / 1 078 · none 1 047 / 253 · --withworld 694 / 98
##   · --withroute 291 / 66.
## Opening the planner *removes* 447 blank rows from the no-world screen, so
## the no-world band is the app's, not this panel's.
##
## Also reports every descendant of `_center_panel` whose combined minimum
## width exceeds the screen -- the defect the register never caught, and the
## one that put 357 px of the panel off the right edge.
##
## Run NOT --headless: a headless dummy render returns a null texture.

const THRESH := 23

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## MISTAKES.md, "Assert on pixels": `THRESH` is **palette-bound**. This machine's
## `cartalith_settings.cfg` carries `[theme] mode="light"`, so `menus.gd::build()`
## boots the shell light and every background pixel comes back (251, 250, 247) --
## above the floor by construction, which is exactly how this probe's first run
## reported `blank_rows=0 of 2400` and proved nothing. The scan below therefore
## refuses to run unless the *drawn* palette is dark, and prints a second,
## palette-agnostic figure (uniform rows) beside it.
func _force_dark(app: Node) -> void:
	if DccTheme.is_dark():
		return
	DccTheme.apply_theme(true)
	app.rebuild_theme(false)

func _find_wide(node: Node, limit: float, out: Array, depth: int = 0) -> void:
	if node is Control:
		var mw: float = (node as Control).get_combined_minimum_size().x
		if mw > limit:
			out.append("%s%s (%s) min_w=%.0f text=%s" % [
				"  ".repeat(depth), node.name, node.get_class(), mw,
				(node.text.substr(0, 44) if ("text" in node) else "-")])
	for c in node.get_children():
		_find_wide(c, limit, out, depth + 1)

func _find_all_windows(node: Node, out: Array) -> Array:
	if node is Window:
		out.append(node)
	for c in node.get_children():
		_find_all_windows(c, out)
	return out

func _band_scan(img: Image) -> void:
	img.convert(Image.FORMAT_RGB8)
	var w := img.get_width()
	var h := img.get_height()
	var data := img.get_data()   ## 3 bytes/pixel, row-major, post-convert
	var stride := w * 3
	var blank_rows := 0
	var uniform_rows := 0
	var longest_run := 0
	var longest_start := -1
	var run := 0
	var run_start := -1
	for y in h:
		var base := y * stride
		var row_blank := true
		var row_uniform := true
		var r0 := data[base]
		var g0 := data[base + 1]
		var b0 := data[base + 2]
		var x := 0
		while x < w:
			var i := base + x * 3
			if row_blank and (data[i] > THRESH or data[i + 1] > THRESH or data[i + 2] > THRESH):
				row_blank = false
			if row_uniform and (data[i] != r0 or data[i + 1] != g0 or data[i + 2] != b0):
				row_uniform = false
			if not row_blank and not row_uniform:
				break
			x += 1
		if row_uniform:
			uniform_rows += 1
		if row_blank:
			blank_rows += 1
			if run == 0:
				run_start = y
			run += 1
			if run > longest_run:
				longest_run = run
				longest_start = run_start
		else:
			run = 0
	print("blank_rows=", blank_rows, " of ", h,
		"  longest_run=", longest_run, " at y=", longest_start, "..", (longest_start + longest_run))
	## Palette-agnostic companion: a row every pixel of which equals its own
	## leftmost pixel. Cannot be defeated by a light palette the way `THRESH` can.
	print("uniform_rows=", uniform_rows, " of ", h)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 120.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var want := Vector2i(1080, 2400)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout

	_force_dark(app)
	await _frames(4)
	print("dark=", DccTheme.is_dark(), " panel=", DccTheme.c("panel"), " bg=", DccTheme.c("bg"))
	if not DccTheme.is_dark():
		print("NOT DARK -- RGB(23,23,23) is a dark-theme floor; nothing measured")
		get_tree().quit(1)
		return

	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
		await get_tree().process_frame

	var screen: Vector2 = app.get_viewport_rect().size
	print("=== phone=", app.is_phone(), " scale=", app.phone_scale(), " screen=", screen, " ===")
	if not app.is_phone():
		print("NOT PHONE -- window size did not register as a handset; nothing measured")
		get_tree().quit(1)
		return

	## `--withroute` generates a world and commits a real route first, which
	## discriminates the two candidate causes the band could have: a map view
	## that never receives a texture on phone (band survives), versus an EMPTY
	## STATE that reserves full-height rows it has nothing to put in (band
	## collapses). `_routecutout_probe.gd`'s own recipe, unchanged.
	## `--withworld` stops one step short: a real world, no committed route. The
	## state that separates "the app has nothing" from "the planner withholds
	## what the app already has".
	if OS.get_cmdline_user_args().has("--withroute") or OS.get_cmdline_user_args().has("--withworld"):
		var bridge = app.bridge
		bridge.generate({
			"seed": 77021, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
			"archetype": "", "villages": true, "sea_level": 0.45,
		})
		while bridge.generating:
			await get_tree().create_timer(0.25).timeout
		await get_tree().create_timer(1.0).timeout
		var wg = bridge.world_gen
		if OS.get_cmdline_user_args().has("--withroute"):
			var places: Array = wg.get_settlements()
			if places.size() < 2:
				print("FAIL: need two settlements, got ", places.size())
				get_tree().quit(1)
				return
			wg.route_begin("mixed")
			wg.route_append_stop(float(places[0]["x"]), float(places[0]["y"]))
			wg.route_append_stop(float(places[1]["x"]), float(places[1]["y"]))
			print("committed route #", wg.route_commit(), " of ", places.size(), " settlements")
		app.select_domain("civilization")
		await _frames(4)

	## `--nojp` measures the SAME screen with the planner never opened -- the
	## control the register's own figure never had. Without it there is no way to
	## tell a band this panel causes from a band the phone shell has anyway.
	if not OS.get_cmdline_user_args().has("--nojp"):
		app.open_journey_planner()
	await get_tree().create_timer(0.8).timeout
	await _frames(4)

	var jp = app.journey_planner_view
	print("bound=", jp._bound, " active=", jp._active, " route_index=", jp._route_index,
		" pts=", jp._route_map.pts.size() if jp._route_map != null else -1,
		" scale_factor=", jp._route_map.scale_factor if jp._route_map != null else -1.0)
	if jp._center_panel != null:
		print("center visible=", jp._center_panel.visible, " size=", jp._center_panel.size)
	if jp._route_map_wrap != null:
		print("route_map_wrap global_rect=", jp._route_map_wrap.get_global_rect())
	## Horizontal overflow: any descendant whose COMBINED MINIMUM width exceeds
	## the screen drags every ancestor container out with it (Godot clamps a
	## Control's size up to its combined minimum even under PRESET_FULL_RECT), so
	## the deepest such node is the cause and the rest are symptoms.
	if jp._center_panel != null:
		var over: Array = []
		_find_wide(jp._center_panel, screen.x, over)
		print("min-width > screen (", int(screen.x), "): ", over.size(), " nodes")
		for e in over:
			print("   ", e)

	for w in _find_all_windows(app, []):
		(w as Window).hide()
	await _frames(4)

	var img := get_viewport().get_texture().get_image()
	var out := "user://ph16band.png"
	img.save_png(out)
	print("saved ", ProjectSettings.globalize_path(out))

	_band_scan(img)
	print("=== done ===")
	get_tree().quit()
