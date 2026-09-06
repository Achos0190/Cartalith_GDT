extends Node
## Empty-phone blank-row census, **attributed to a surface**.
##
## `_ph16band_probe.gd` counts blank rows and finds the longest run; it does not
## say *whose* rows they are, and the register was misled twice by exactly that
## (first blaming the Journey Planner, then the app-with-no-world). This probe
## keeps the same pixel definition -- a row no pixel of which exceeds
## RGB(23,23,23), the register's own dark-palette floor -- and adds the missing
## column: for every blank row, which phone surface is drawn there.
##
## Run **windowed**, never `--headless`: `ImageTexture.update()` is a no-op under
## the dummy driver and `get_texture().get_image()` comes back null or stale
## (MISTAKES.md, "Run a pixel probe").
##
##   godot4 --path . _emptyphone_probe.tscn -- --force-touch [flags]
##
## **Every flag below is read by the body of this file** -- grepped, not
## remembered, because a header documenting an argument the probe never reads
## measures the same state three times and calls it three (MISTAKES.md, "Write
## a probe's usage header"). `_unknown_args()` refuses to run on anything else
## rather than silently defaulting.
##
##   State (pick one; default is a cold boot with the entry screen as shipped):
##     (none)       cold boot, entry screen as shipped  -- the FIRST SCREEN
##     --dismiss    entry screen dismissed, still no world -- the register's state
##     --world      a world generated (the picker closes itself on load)
##
##   Then optionally one of:
##     --tab ID     press a bottom-nav tab first: map · gen · plan · more
##     --screen ID  open MORE and push one `PhoneMenu.ROOT_ROWS` screen:
##                  project · civ · data · assets · travel · sim · prefs · help
##     --pressgen   press the tool sheet's `GENERATE WORLD` and assert a world
##                  appears; asserts instead of censusing, then exits
##
##   Always available:
##     --vp WxH     window size (default 1080x2400, the density every figure in
##                  the register was taken at)
##     --desktop    assert the empty-session hint only, on a shell booted
##                  WITHOUT `--force-touch`; no phone census follows
##
## The density is printed beside every number because a touch floor and a drawn
## height are different questions at different densities (MISTAKES.md, "Judge a
## measurement against a floor").

const THRESH := 23
const SAMPLES := 5   ## x positions per row used to name the owning surface.

var _vp := Vector2i(1080, 2400)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, fallback: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return args[i + 1]
	return fallback

func _has(name: String) -> bool:
	return OS.get_cmdline_user_args().has(name)

const FLAGS_BARE: Array = ["--force-touch", "--dismiss", "--world", "--pressgen",
	"--desktop"]
const FLAGS_VALUE: Array = ["--vp", "--tab", "--screen"]

## Fail loudly on an argument this probe does not read, rather than defaulting
## and reporting a state nobody asked for. `--force-touch` is the shell's, not
## this file's, and is listed so passing it is not an error.
func _unknown_args() -> Array:
	var out: Array = []
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if FLAGS_VALUE.has(a):
			i += 2
			continue
		if not FLAGS_BARE.has(a):
			out.append(a)
		i += 1
	return out

## The floor is palette-bound and this machine boots `mode="light"`, where every
## background pixel is 251 and the scan cannot fail. Force dark, then refuse to
## run if it did not take.
func _force_dark(app: Node) -> void:
	if DccTheme.is_dark():
		return
	DccTheme.apply_theme(true)
	app.rebuild_theme(false)

## Per-row blankness, plus the palette-agnostic companion (`uniform`: every pixel
## on the row equals the row's leftmost pixel) so a future palette change cannot
## silently make this vacuous.
func _scan(img: Image) -> Dictionary:
	img.convert(Image.FORMAT_RGB8)
	var w := img.get_width()
	var h := img.get_height()
	var data := img.get_data()
	var stride := w * 3
	var blank: Array = []
	blank.resize(h)
	var uniform := 0
	for y in h:
		var base := y * stride
		var is_blank := true
		var is_uniform := true
		var r0 := data[base]
		var g0 := data[base + 1]
		var b0 := data[base + 2]
		var x := 0
		while x < w:
			var i := base + x * 3
			if is_blank and (data[i] > THRESH or data[i + 1] > THRESH or data[i + 2] > THRESH):
				is_blank = false
			if is_uniform and (data[i] != r0 or data[i + 1] != g0 or data[i + 2] != b0):
				is_uniform = false
			if not is_blank and not is_uniform:
				break
			x += 1
		blank[y] = is_blank
		if is_uniform:
			uniform += 1
	return {"blank": blank, "uniform": uniform, "w": w, "h": h}

func _find_button(node: Node, want: String) -> Button:
	if node is Button and (node as Button).text.strip_edges().to_lower() == want:
		return node as Button
	for c in node.get_children():
		var hit := _find_button(c, want)
		if hit != null:
			return hit
	return null

func _find_wide(node: Node, limit: float, out: Array, depth: int = 0) -> void:
	if node is Control and (node as Control).is_visible_in_tree():
		var mw: float = (node as Control).get_combined_minimum_size().x
		if mw > limit:
			out.append("%s%s (%s) min_w=%.0f text=%s" % [
				"  ".repeat(depth), node.name, node.get_class(), mw,
				(node.text.substr(0, 44) if ("text" in node) else "-")])
	for c in node.get_children():
		_find_wide(c, limit, out, depth + 1)

func _visible_rect(n: Node) -> Rect2:
	if n is Window:
		var win := n as Window
		if not win.visible:
			return Rect2()
		return Rect2(Vector2(win.position), Vector2(win.size))
	if n is Control:
		var c := n as Control
		if not c.is_visible_in_tree():
			return Rect2()
		return c.get_global_rect()
	return Rect2()

## Ordered bottom-to-top. The LAST entry whose rect covers a row's sample point
## owns that row -- the same order Godot draws them in.
func _regions(app: Node) -> Array:
	var out: Array = []
	var add := func(label: String, n):
		if n == null:
			return
		var r: Rect2 = _visible_rect(n)
		if r.size.x <= 0.0 or r.size.y <= 0.0:
			return
		out.append({"name": label, "rect": r})
	add.call("map viewport (behind all chrome)", app._phone_root)
	add.call("status safe row", app._phone_top_safe)
	add.call("app bar", app._phone_app_bar)
	add.call("content gap -- MAP", app._phone_content_gap)
	add.call("tool sheet", app._phone_tool_sheet)
	add.call("timeline strip", app.timeline_bar)
	add.call("bottom nav bar", app._phone_menu_bar)
	add.call("gesture inset", app._phone_gesture_inset)
	add.call("left dock sheet", app.left_dock)
	add.call("right dock sheet", app.right_dock)
	if app._phone_menu != null:
		add.call("MORE menu screen", app._phone_menu._screen)
		add.call("MORE menu sheet", app._phone_menu._sheet)
	add.call("entry screen (PhoneProjectPicker)", app.phone_project_picker)
	add.call("open project dialog", app.open_project_dialog)
	if app.journey_planner_view != null:
		add.call("journey planner", app.journey_planner_view)
	return out

func _owner_of(y: int, w: int, regions: Array) -> String:
	var best := "(nothing drawn)"
	for i in range(regions.size()):
		var r: Rect2 = regions[i]["rect"]
		if y < r.position.y or y >= r.position.y + r.size.y:
			continue
		## The row is blank across its whole width, so any region that covers the
		## row's centre band is a candidate; take the topmost.
		var covers := false
		for s in SAMPLES:
			var x := int((float(s) + 0.5) / float(SAMPLES) * float(w))
			if x >= r.position.x and x < r.position.x + r.size.x:
				covers = true
				break
		if covers:
			best = regions[i]["name"]
	return best

func _census(app: Node, img: Image, tag: String) -> void:
	var s := _scan(img)
	var blank: Array = s["blank"]
	var h: int = s["h"]
	var w: int = s["w"]
	var regions := _regions(app)

	var per: Dictionary = {}       ## surface -> blank rows
	var total_rows: Dictionary = {}  ## surface -> rows it covers at all
	var order: Array = []
	for y in h:
		var who := _owner_of(y, w, regions)
		if not total_rows.has(who):
			total_rows[who] = 0
			per[who] = 0
			order.append(who)
		total_rows[who] += 1
		if blank[y]:
			per[who] += 1

	var blank_total := 0
	for y in h:
		if blank[y]:
			blank_total += 1

	## Longest contiguous blank run and who owns its midpoint.
	var run := 0
	var run_start := -1
	var longest := 0
	var longest_start := -1
	for y in h:
		if blank[y]:
			if run == 0:
				run_start = y
			run += 1
			if run > longest:
				longest = run
				longest_start = run_start
		else:
			run = 0

	print("")
	print("=== CENSUS ", tag, "  ", w, "x", h, " (phone density, windowed) ===")
	print("blank_rows=", blank_total, " of ", h, "   uniform_rows=", s["uniform"],
		"   longest_run=", longest, " at y=", longest_start, "..",
		(longest_start + longest), " owned by ", _owner_of(longest_start + longest / 2, w, regions))
	print("%-42s %6s %6s %6s %5s" % ["SURFACE", "y0", "y1", "rows", "blank"])
	for name in order:
		var y0 := -1
		var y1 := -1
		for y in h:
			if _owner_of(y, w, regions) == name:
				if y0 < 0:
					y0 = y
				y1 = y
		print("%-42s %6d %6d %6d %5d" % [name, y0, y1 + 1, total_rows[name], per[name]])

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var bad := _unknown_args()
	if not bad.is_empty():
		print("UNKNOWN ARGUMENT(S) ", bad, " -- see this file's header; refusing to run")
		get_tree().quit(2)
		return

	var spec := _arg("--vp", "1080x2400").split("x")
	if spec.size() == 2:
		_vp = Vector2i(int(spec[0]), int(spec[1]))
	DisplayServer.window_set_size(_vp)
	get_window().size = _vp
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.6).timeout

	_force_dark(app)
	await _frames(4)
	if not DccTheme.is_dark():
		print("NOT DARK -- RGB(23,23,23) is a dark-theme floor; nothing measured")
		get_tree().quit(1)
		return

	## The empty session's one instruction, asserted in BOTH directions so the
	## check can actually fail. `--desktop` boots the same shell without
	## `--force-touch`, where the desktop wording is the correct answer -- a
	## test that only ever sees the phone could not tell a working branch from a
	## hardcoded phone string.
	var hint: String = app.status_slot_text("hint")
	var route: String = DccShell.new_world_route()
	print("hint=", hint, "   new_world_route=", route, "   is_phone=", app.is_phone())
	var want := "MORE ▸ Project ▸ New world…" if app.is_phone() else "File ▸ New world…"
	if hint != want + " to begin" or route != want:
		print("FAIL: empty-session hint should be '", want, " to begin'")
		get_tree().quit(1)
		return
	print("OK hint route matches the composition")

	if _has("--desktop"):
		print("=== desktop leg only asserts the hint; no phone census follows ===")
		get_tree().quit()
		return
	if not app.is_phone():
		print("NOT PHONE -- window did not register as a handset; nothing measured")
		get_tree().quit(1)
		return
	print("density=", _vp, " phone=", app.is_phone(), " scale=", app.phone_scale(),
		" dark=", DccTheme.is_dark())

	var tag := "cold boot (entry screen as shipped)"
	if _has("--world"):
		var bridge = app.bridge
		bridge.generate({
			"seed": 77021, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
			"archetype": "", "villages": true, "sea_level": 0.45,
		})
		while bridge.generating:
			await get_tree().create_timer(0.25).timeout
		await get_tree().create_timer(1.5).timeout
		tag = "world generated"
	elif _has("--dismiss"):
		if app.phone_project_picker != null:
			app.phone_project_picker.hide()
		if app.open_project_dialog != null:
			app.open_project_dialog.hide()
		tag = "no world, entry screen dismissed"
	await _frames(6)

	## **The premise behind "no signpost was added".** The claim recorded at
	## `dcc_shell.gd::_build_phone_shell()`'s `_phone_content_gap` is that the
	## empty shell already offers a route to a world without one, because the
	## default GENERATE tab's tool sheet carries `Generate world` as an
	## accent-filled primary. Drawn is not wired (MISTAKES.md), so the button is
	## pressed here and `bridge.has_world` read back -- not inspected in a
	## screenshot and not reasoned from `world_workspace.gd`.
	if _has("--pressgen"):
		var btn := _find_button(app._phone_tool_sheet, "generate world")
		if btn == null:
			print("FAIL: no `Generate world` button in the empty phone tool sheet")
			get_tree().quit(1)
			return
		print("found `", btn.text, "` disabled=", btn.disabled,
			" rect=", btn.get_global_rect(), " has_world_before=", app.bridge.has_world)
		btn.emit_signal("pressed")
		while app.bridge.generating:
			await get_tree().create_timer(0.25).timeout
		await get_tree().create_timer(1.0).timeout
		print("has_world_after=", app.bridge.has_world)
		if not app.bridge.has_world:
			print("FAIL: pressing it from the empty shell produced no world")
			get_tree().quit(1)
			return
		print("OK the empty shell's own primary makes a world")
		get_tree().quit()
		return

	## Per-tab census. The bottom bar is L1 of the phone disclosure tree, so
	## "which surface owns the blank rows" is only answered once each of its four
	## destinations has been opened -- a panel that opens onto nothing is a
	## different defect from a map with nothing in it.
	var tab := _arg("--tab", "")
	if tab != "":
		app._pick_phone_tab(tab)
		await get_tree().create_timer(1.0).timeout
		await _frames(6)
		tag += " · tab=" + tab

	## One of `PhoneMenu.ROOT_ROWS`' eight ids, pushed on top of MORE. This is
	## the brief's second category -- "a whole panel that is meaningless without
	## a world" -- and it can only be answered one screen at a time.
	var screen := _arg("--screen", "")
	if screen != "":
		app._phone_menu.open()
		await _frames(2)
		app._phone_menu._push_screen(screen)
		await get_tree().create_timer(0.8).timeout
		await _frames(6)
		tag += " · MORE/" + screen

	print("picker_visible=", (app.phone_project_picker != null and app.phone_project_picker.visible),
		" has_world=", app.bridge.has_world)

	## Horizontal overflow, for the hazard `phone_menu.gd::_note_row()` names: a
	## `ScrollContainer` with horizontal scrolling DISABLED folds its child's
	## minimum width into its own, so one unwrapped `Label` drags every ancestor
	## past the screen with no scrollbar to reveal it (MISTAKES.md, "Read a
	## layout that overflows the screen"). Reported for whichever overlay is up.
	for host in [app._phone_menu._screen if app._phone_menu != null else null,
			app._phone_chrome_margin]:
		if host == null or not (host as Control).is_visible_in_tree():
			continue
		var over: Array = []
		_find_wide(host, _vp.x, over)
		print("min-width > ", _vp.x, " under ", (host as Control).name, ": ", over.size(), " nodes")
		for e in over:
			print("   ", e)

	var img := get_viewport().get_texture().get_image()
	img.save_png("user://emptyphone.png")
	_census(app, img, tag)
	print("=== done ===")
	get_tree().quit()
