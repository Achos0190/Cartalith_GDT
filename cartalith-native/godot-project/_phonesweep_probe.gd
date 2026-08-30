extends Node
## THROWAWAY evidence-gathering probe for the OnePlus 12 phone-UI complaint.
## Not tracked, not to be committed. Boots the real shell at handset size(s)
## with --force-touch, generates a small world, walks every named screen,
## screenshots it, and measures every tappable control and label font size.
##
## Run (windowed, NOT headless -- this is about pixels):
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1440x3168 _phonesweep_probe.tscn -- --force-touch
##
## Sweeps 1440x3168 first, then resizes the live window to 1080x2400 in place
## (same trick _shot_phone.gd's --rotate uses) so both resolutions come out of
## one run. Screenshots and the raw measurement log land in
## user://phonesweep/*.png and user://phonesweep/log.txt.

const SEED := 483920
const TAP_FLOOR := 44.0   ## dcc_theme.gd DccTheme.PHONE_TAP_MIN, dp/units as authored.
const FONT_FLOOR := 11.0

var app: Node
var _log_lines: Array = []
var _shot_dir := ""
var _last_window: Window = null   ## most recently opened standalone Window, hidden before the next.

func _log(s: String) -> void:
	print(s)
	_log_lines.append(s)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _find(node: Node, pred: Callable) -> Node:
	if pred.call(node):
		return node
	for c in node.get_children():
		var hit := _find(c, pred)
		if hit != null:
			return hit
	return null

# ---------------------------------------------------------------------------
# Measurement
# ---------------------------------------------------------------------------

func _win_scale(n: Node) -> float:
	var w := n.get_window()
	if w == null:
		return 1.0
	return w.content_scale_factor if w.content_scale_factor > 0.0 else 1.0

## Walks the whole live tree from `root` (usually get_tree().root, so it
## crosses into every open sub-Window too) collecting physical-px heights of
## every visible BaseButton / LineEdit / Range, and physical-px font sizes of
## every visible Label / RichTextLabel.
func _measure(root: Node) -> Dictionary:
	var tappable: Array = []   # [{path, cls, text, h, floor_violation}]
	var fonts: Array = []      # [{path, cls, text, fs}]

	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children():
			stack.append(c)
		if n is Control and not (n as Control).is_visible_in_tree():
			continue
		if n is BaseButton or n is LineEdit or n is Range:
			var ctl := n as Control
			var scale := _win_scale(n)
			var h: float = ctl.size.y * scale
			var text := ""
			if n is BaseButton:
				text = (n as BaseButton).text
			elif n is LineEdit:
				text = (n as LineEdit).placeholder_text
			tappable.append({"path": str(n.get_path()), "cls": n.get_class(),
				"text": String(text).substr(0, 34), "h": h})
		if n is Label or n is RichTextLabel:
			var scale2 := _win_scale(n)
			var key := "normal_font_size" if n is RichTextLabel else "font_size"
			var fs: int = (n as Control).get_theme_font_size(key)
			var text2 := ""
			if n is Label:
				text2 = (n as Label).text
			fonts.append({"path": str(n.get_path()), "cls": n.get_class(),
				"text": String(text2).substr(0, 34), "fs": fs * scale2})

	return {"tappable": tappable, "fonts": fonts}

func _report_measure(tag: String, m: Dictionary) -> void:
	var tappable: Array = m["tappable"]
	var under: Array = tappable.filter(func(r): return r["h"] < TAP_FLOOR and r["h"] > 0.0)
	under.sort_custom(func(a, b): return a["h"] < b["h"])
	_log("  [measure %s] tappable controls=%d  under-%.0fpx-floor=%d" %
		[tag, tappable.size(), TAP_FLOOR, under.size()])
	for r in under.slice(0, 12):
		_log("    UNDER-FLOOR h=%.1f  %s '%s'  %s" % [r["h"], r["cls"], r["text"], r["path"]])

	var fonts: Array = m["fonts"]
	var funder: Array = fonts.filter(func(r): return r["fs"] < FONT_FLOOR and r["fs"] > 0.0)
	funder.sort_custom(func(a, b): return a["fs"] < b["fs"])
	_log("  [measure %s] labels=%d  under-%.0fpx-font=%d" %
		[tag, fonts.size(), FONT_FLOOR, funder.size()])
	for r in funder.slice(0, 12):
		_log("    SMALL-FONT fs=%.1f  %s '%s'  %s" % [r["fs"], r["cls"], r["text"], r["path"]])

func _min_size_report(tag: String) -> void:
	var w := get_window()
	var content_root: Control = app if app is Control else null
	var min_sz := Vector2.ZERO
	if content_root != null:
		min_sz = content_root.get_combined_minimum_size()
	_log("  [win %s] window.size=%s content_scale_factor=%s content_min=%s screen=%s" %
		[tag, w.size, w.content_scale_factor, min_sz, get_viewport().get_visible_rect().size])
	if min_sz.x > w.size.x:
		_log("    OVERFLOW: content min width %.0f > window width %d" % [min_sz.x, w.size.x])

# ---------------------------------------------------------------------------
# Screenshot + per-screen driver
# ---------------------------------------------------------------------------

func _shot(name: String, res_tag: String) -> void:
	await _frames(4)
	var img := get_viewport().get_texture().get_image()
	var out := "%s/%s__%s.png" % [_shot_dir, res_tag, name]
	img.save_png(out)
	_log("  shot -> %s  (%dx%d)" % [out, img.get_width(), img.get_height()])

func _open_window(win: Window) -> void:
	if _last_window != null and is_instance_valid(_last_window) and _last_window != win:
		_last_window.hide()
	_last_window = win

func _close_overlays_and_windows() -> void:
	app._set_overflow_open(false)   ## closes drawer/picker/menu/dock-sheets too
	if _last_window != null and is_instance_valid(_last_window):
		_last_window.hide()
	_last_window = null

func _screen(name: String, res_tag: String, setup: Callable) -> void:
	_log("--- %s / %s ---" % [res_tag, name])
	await setup.call()
	await _frames(6)
	_min_size_report(name)
	_report_measure(name, _measure(get_tree().root))
	await _shot(name, res_tag)

# ---------------------------------------------------------------------------
# Flick / scroll test
# ---------------------------------------------------------------------------

func _find_scroll(n: Node) -> ScrollContainer:
	if n is ScrollContainer:
		return n
	for c in n.get_children():
		var s := _find_scroll(c)
		if s != null:
			return s
	return null

func _flick(scroll: ScrollContainer, at: Vector2) -> int:
	scroll.scroll_vertical = 0
	await get_tree().process_frame
	var vp := get_viewport()
	var hover := InputEventMouseMotion.new()
	hover.position = at
	vp.push_input(hover)
	await get_tree().process_frame
	var mb := InputEventMouseButton.new()
	mb.button_index = MOUSE_BUTTON_LEFT
	mb.pressed = true
	mb.position = at
	vp.push_input(mb)
	var p := at
	for i in 10:
		p += Vector2(0, -14)
		var mm := InputEventMouseMotion.new()
		mm.position = p
		mm.relative = Vector2(0, -14)
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT
		vp.push_input(mm)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = p
	vp.push_input(up)
	await get_tree().process_frame
	return scroll.scroll_vertical

func _flick_test(res_tag: String) -> void:
	_log("--- %s / flick-test-left-sheet ---" % res_tag)
	_close_overlays_and_windows()
	app._set_sheet_open("left", true)
	await _frames(6)
	var scroll := _find_scroll(app.left_dock)
	if scroll == null:
		_log("  no ScrollContainer under left_dock")
	else:
		var r := scroll.get_global_rect()
		var mv: float = scroll.get_v_scroll_bar().max_value
		var got := await _flick(scroll, r.get_center())
		_log("  scroll rect=%s content max_value=%s after-flick scroll_vertical=%d" % [r, mv, got])
	app._set_sheet_open("left", false)
	await _frames(2)

# ---------------------------------------------------------------------------
# Main sweep
# ---------------------------------------------------------------------------

func _sweep(res_tag: String) -> void:
	_log("=== SWEEP %s  phone=%s scale=%s landscape=%s ===" %
		[res_tag, app.is_phone(), app.phone_scale(), get_window().size.x > get_window().size.y])

	await _screen("shell_at_rest", res_tag, func():
		_close_overlays_and_windows())

	await _screen("menu_drawer", res_tag, func():
		_close_overlays_and_windows()
		app._set_drawer_open(true))

	await _screen("panel_picker", res_tag, func():
		_close_overlays_and_windows()
		app._set_panel_picker_open(true))

	await _screen("overflow_sheet", res_tag, func():
		_close_overlays_and_windows()
		app._set_overflow_open(true))

	for dom in ["world", "civilization", "cartography"]:
		await _screen("domain_%s_leftsheet" % dom, res_tag, func():
			_close_overlays_and_windows()
			app.select_domain(dom)
			app._set_sheet_open("left", true))

	await _screen("phone_tool_sheet_paint", res_tag, func():
		_close_overlays_and_windows()
		app.select_domain("world")
		app.arm_tool("paint")
		var bar = DccToolBar.instance()
		if bar != null:
			bar.mode = "paint"
			bar.rebuild())

	await _screen("asset_library", res_tag, func():
		_close_overlays_and_windows()
		app.open_asset_library()
		_open_window(app.asset_library_window))

	await _screen("journey_planner", res_tag, func():
		_close_overlays_and_windows()
		app.open_journey_planner())

	await _screen("data_manager", res_tag, func():
		_close_overlays_and_windows()
		app.open_data_manager()
		_open_window(app.data_manager_window))

	await _screen("travel_library", res_tag, func():
		_close_overlays_and_windows()
		app.open_travel_library()
		_open_window(app.travel_library_window))

	await _screen("world_data", res_tag, func():
		_close_overlays_and_windows()
		app.open_world_data()
		_open_window(app.world_data_window))

	await _screen("performance", res_tag, func():
		_close_overlays_and_windows()
		app.open_performance()
		_open_window(app.performance_window))

	await _screen("gen_info", res_tag, func():
		_close_overlays_and_windows()
		app.open_gen_info()
		_open_window(app.gen_info_dialog))

	await _screen("layers_popover", res_tag, func():
		_close_overlays_and_windows()
		app.layers_popover.open())

	await _screen("credits", res_tag, func():
		_close_overlays_and_windows()
		app.open_credits()
		var dlg := _find(app, func(n): return n is AcceptDialog and (n as AcceptDialog).title.begins_with("Credits"))
		if dlg != null:
			_open_window(dlg as Window))

	# -- control group: screens already given a phone treatment ------------
	var settlements: Array = app.bridge.settlements()
	var idx := 0
	if settlements.size() > 0:
		var best := 0
		for i in settlements.size():
			if float(settlements[i].get("population", 0)) > float(settlements[best].get("population", 0)):
				best = i
		idx = best

	await _screen("ctrl_city_viewer", res_tag, func():
		_close_overlays_and_windows()
		app.open_city_viewer(idx)
		_open_window(app.city_viewer_window))

	await _screen("ctrl_place_editor", res_tag, func():
		_close_overlays_and_windows()
		app.open_place_editor(idx)
		_open_window(app.place_editor_window))

	await _screen("ctrl_faction_roster", res_tag, func():
		_close_overlays_and_windows()
		app.open_faction_roster()
		_open_window(app.faction_roster_window))

	await _screen("ctrl_vault", res_tag, func():
		_close_overlays_and_windows()
		app.open_vault_overview()
		_open_window(app.vault_window))

	await _screen("ctrl_new_world", res_tag, func():
		_close_overlays_and_windows()
		app.open_new_world()
		_open_window(app.new_world_dialog))

	await _screen("ctrl_open_project", res_tag, func():
		_close_overlays_and_windows()
		app.open_project_picker()
		_open_window(app.open_project_dialog))

	await _screen("ctrl_browse", res_tag, func():
		_close_overlays_and_windows()
		var dlg = DccBrowseDialog.choose_folder(app, "Browse test — pick a folder", "",
			"", func(_p): pass)
		_open_window(dlg))

	_close_overlays_and_windows()
	await _flick_test(res_tag)

## `DccShell._ready()`'s own comment: "Phone-vs-tablet is decided once, here,
## off the boot window size" -- `_compute_layout_mode()` runs synchronously
## inside `add_child(app)` below and the whole tree is built from its result,
## so the window MUST already be phone-shaped before that call, not resized
## afterwards. `--resolution WxH` at launch is clamped to the monitor's work
## area by Godot's own startup window creation (1680x1050 here can't hold a
## 3168-tall window) -- confirmed the hard way: a first attempt at
## `--resolution 1440x3168` booted a 1440x1031 window and the shell measured
## itself as *tablet*, not phone, for the entire run. Reassigning
## `Window.size` at runtime, after boot, does NOT suffer that clamp (also
## confirmed empirically), so this boots small and immediately resizes before
## `app` exists.
func _boot_size(target: Vector2i) -> void:
	var w := get_window()
	w.size = target
	for i in 10:
		await get_tree().process_frame
		if w.size == target:
			break
	_log("[boot] requested %s got window.size=%s" % [target, w.size])

func _ready() -> void:
	Input.set_emulate_touch_from_mouse(true)
	_shot_dir = "user://phonesweep"
	DirAccess.make_dir_recursive_absolute(_shot_dir)

	await _boot_size(Vector2i(1440, 3168))

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(0.8).timeout

	var bridge = app.bridge
	bridge.generate({
		"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.6).timeout

	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	await _sweep("1440x3168")

	## In-place resolution swap: resize the real window, same technique
	## _shot_phone.gd's --rotate uses to fire root.size_changed for real. This
	## is safe post-boot (unlike the initial size) *because* `_phone` is
	## already true from the first pass -- `_on_window_resized()` only
	## re-runs `_compute_layout_mode()` when already in phone mode; see
	## `_boot_size()`'s header comment for why the very first size can't go
	## through this same path.
	_close_overlays_and_windows()
	await _boot_size(Vector2i(1080, 2400))
	await _frames(3)
	await get_tree().create_timer(0.4).timeout
	_log("[boot] after resize: phone=%s scale=%s" % [app.is_phone(), app.phone_scale()])

	await _sweep("1080x2400")

	var f := FileAccess.open(_shot_dir + "/log.txt", FileAccess.WRITE)
	f.store_string("\n".join(_log_lines))
	f.close()
	_log("=== DONE. log at %s ===" % ProjectSettings.globalize_path(_shot_dir + "/log.txt"))
	_log("=== shots at %s ===" % ProjectSettings.globalize_path(_shot_dir))
	get_tree().quit()
