extends Node
## TEMPORARY verification harness for the nine phone-untreated windows
## (`GUI_GAP_REGISTER.md` PH-12). Opens each one at handset size and measures
## it rather than eyeballing a screenshot.
##
##   godot4 --path . --resolution 1440x3168 _ph9_probe.tscn -- --force-touch --nowelcome
##   godot4 --path . --resolution 1080x2400 _ph9_probe.tscn -- --force-touch --nowelcome
##
## `--force-touch` is `dcc_shell.gd`'s own testing override; without it the
## phone composition is unreachable on a dev box with no touch hardware.
##
## What it asserts, per window:
##   - `content_scale_factor` (1.0 = still desktop pixels at native density),
##   - `size` against the screen (anything wider overflows),
##   - the tallest/widest combined minimum in the tree, innermost first,
##   - how many tappable Controls fall under §13's 44 dp floor,
##   - whether the body actually scrolls (content > viewport in a
##     ScrollContainer).

var app: Node
var _screen := Vector2.ZERO

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

func _find_all(node: Node, pred: Callable, out: Array) -> Array:
	if pred.call(node):
		out.append(node)
	for c in node.get_children():
		_find_all(c, pred, out)
	return out

func _ready() -> void:
	Input.set_emulate_touch_from_mouse(true)
	## `--resolution` is clamped to the monitor's usable rect on Windows, which
	## silently turns a 1440x3168 handset run into a 1440x1031 desktop one --
	## and `_compute_layout_mode()` then reports `phone=false`, so the whole
	## probe measures the wrong composition. Set explicitly, after boot, where
	## nothing clamps it; `--position 0,0` keeps the off-screen part off the
	## bottom rather than off the top.
	var want := Vector2i(1440, 3168)
	if OS.get_cmdline_user_args().has("--h1080"):
		want = Vector2i(1080, 2400)
	DisplayServer.window_set_size(want)
	get_window().size = want
	## Android has exactly one OS window, so Godot embeds every subwindow into
	## the root viewport there. On Windows they are real OS windows, and the
	## compositor clamps one to the monitor -- which silently caps a 1440x3168
	## full-screen dialog at 1440x1002 and makes the probe report a fill that
	## did not happen. Embedded is both the faithful model of the device and
	## the only one whose numbers mean anything here.
	print("embed_subwindows was ", get_tree().root.gui_embed_subwindows)
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)
	print("ds_window_size=", DisplayServer.window_get_size(),
		" root.size=", get_tree().root.size,
		" root.visible_rect=", get_tree().root.get_visible_rect(),
		" screen_usable=", DisplayServer.screen_get_usable_rect())
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	_screen = app.get_viewport_rect().size
	print("=== phone=", app.is_phone(), " scale=", app.phone_scale(),
		" screen=", _screen, " ===")

	## A world, so the tables, the journey planner and the asset library are
	## measured with real content in them rather than an empty state -- 240
	## settlement rows behave very differently from "No world generated".
	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	print("=== has_world=", app.bridge.has_world, " ===")
	await _frames(6)

	await _win("performance", func(): app.performance_window.open(), app.performance_window)
	await _win("gen_info", func(): app.gen_info_dialog.open(), app.gen_info_dialog)
	await _win("world_data", func(): app.world_data_window.open(), app.world_data_window)
	await _win("travel_library", func(): app.travel_library_window.open(), app.travel_library_window)
	await _win("data_manager", func(): app.data_manager_window.open(), app.data_manager_window)
	await _win("asset_library", func(): app.asset_library_window.open(), app.asset_library_window)
	await _win("asset_slicer", func():
		app.asset_library_window.open("", true), app.asset_library_window._slicer)
	await _win("layers", func(): app.layers_popover.open(), app.layers_popover)
	## Layers is a `PopupPanel`, not an `AcceptDialog`; §13 asks whether a
	## second phone overlay can sit on top of it, so open the drawer while it is
	## up and report what happens, then check the back chain hides the popover
	## first (`DccShell::_notification`'s `_topmost_subwindow`).
	app.layers_popover.open()
	await get_tree().create_timer(0.5).timeout
	if app.is_phone():
		app._set_drawer_open(true)
		await _frames(4)
		print("[layers] with drawer open: popover visible=", app.layers_popover.visible,
			" drawer visible=", app._phone_drawer.visible)
		app._set_drawer_open(false)
		await _frames(2)
	app.propagate_notification(NOTIFICATION_WM_GO_BACK_REQUEST)
	await _frames(3)
	print("[layers] after one back: popover visible=", app.layers_popover.visible)
	app.layers_popover.hide()
	await _frames(2)

	app.open_credits()
	await _frames(6)
	var creds: Window = app._credits_dialog
	if creds != null:
		await _report("credits", creds)
		var rt := _find(creds, func(n): return n is RichTextLabel) as RichTextLabel
		print("[credits] body chars=", (rt.get_parsed_text().length() if rt != null else -1),
			" (0 or -1 = the empty-body bug)")
		await _shot("credits")
		creds.hide()

	# Journey planner is not a Window: it swaps `app.viewport_content`.
	app.open_journey_planner()
	await get_tree().create_timer(0.8).timeout
	await _report_ctl("journey_center", app.journey_planner_view._center_panel)
	await _shot("journey_center")

	print("=== done ===")
	get_tree().quit()

func _win(tag: String, opener: Callable, w: Window) -> void:
	opener.call()
	await get_tree().create_timer(0.7).timeout
	await _report(tag, w)
	await _shot(tag)
	if is_instance_valid(w):
		w.hide()
	if tag.begins_with("asset"):
		app.asset_library_window.hide()
	await _frames(3)

func _report(tag: String, w: Window) -> void:
	if w == null or not is_instance_valid(w):
		print("[", tag, "] MISSING")
		return
	## **`Window.popup(Rect2i)` clamps to the parent's usable rect**, and on a
	## non-embedded desktop subwindow that rect is the MONITOR's usable rect --
	## 1680x1002 here -- not the 1440x3168 root viewport this probe set up. So a
	## phone window that correctly asks to fill 1440x3168 is reported 1440x1002
	## on this box: the width (the axis the 393 dp column problem lives on) is
	## real, the height is a desktop artifact with no counterpart on Android,
	## where there is one OS window and every subwindow is embedded in it.
	## Judge the fill by `size.x` and `position`, and the rest by the content
	## measurements below, which are unaffected.
	## The correction that makes the numbers below the device's rather than this
	## monitor's. The window is already visible, so assigning `size` DOES raise
	## the resize notification `AcceptDialog` lays its content child out from
	## (`dcc_widgets.gd`'s own `phone_present()` header records the hidden-window
	## case, which is the one that does not) -- so this re-lays the real
	## composition at the real handset height instead of measuring one squeezed
	## into 1002 px by a desktop artifact. Only for windows that asked to fill.
	if w.position == Vector2i.ZERO and w.size.x == int(_screen.x) and w.size.y < int(_screen.y):
		w.size = Vector2i(_screen)
		await _frames(3)
	print("[", tag, "] embedded=", w.is_embedded(), " visible=", w.visible,
		" size=", w.size, " pos=", w.position,
		" scale=", w.content_scale_factor, " wrap=", ("wrap_controls" in w and w.wrap_controls),
		" borderless=", w.borderless, " min_size=", w.min_size)
	var over_w: int = int(w.size.x) - int(_screen.x)
	var over_h: int = int(w.size.y) - int(_screen.y)
	print("[", tag, "] overflow px  w=", over_w, " h=", over_h)
	await _report_ctl(tag, w)

func _report_ctl(tag: String, root: Node) -> void:
	if root == null or not is_instance_valid(root):
		print("[", tag, "] MISSING control")
		return
	var unit: float = 1.0
	var owner_win: Node = root
	while owner_win != null and not (owner_win is Window):
		owner_win = owner_win.get_parent()
	if owner_win != null and owner_win != get_tree().root:
		unit = (owner_win as Window).content_scale_factor
	## Everything laid out in the MAIN viewport has no content scale, so an
	## authored pixel there is one physical pixel: the floor to compare against
	## is 44 * phone_scale, not 44.
	var floor_px: float = 44.0 if unit != 1.0 else 44.0 * app.phone_scale()
	var taps: Array = []
	_find_all(root, func(n): return n is BaseButton or n is LineEdit or n is Range \
		or n is TextEdit or n is SpinBox, taps)
	var small := 0
	var smallest := 1e9
	for t in taps:
		var c := t as Control
		if not c.visible:
			continue
		var h: float = maxf(c.size.y, c.get_combined_minimum_size().y)
		if h < floor_px:
			small += 1
			smallest = minf(smallest, h)
	print("[", tag, "] tappables=", taps.size(), " under ", floor_px, "px = ", small,
		" smallest=", (smallest if small > 0 else 0.0))
	## **The column to compare against is the window's own space, not the
	## screen's.** A content-scaled window is 1440 physical px wide and 393 dp
	## wide, and every `custom_minimum_size` inside it is in dp. Comparing dp
	## against 1440 finds nothing and misses exactly the fault this pass is
	## about -- "a `Button` reports its own text as its minimum width, and a
	## `Window` cannot be narrower than its content's minimum".
	var col_w: float = _screen.x / maxf(1.0, unit if unit != 1.0 else 1.0)
	if unit == 1.0 and owner_win != null and owner_win != get_tree().root:
		col_w = _screen.x   ## an unscaled window really is that many px wide
	var wide: Array = []
	_find_all(root, func(n): return n is Control \
		and (n as Control).get_combined_minimum_size().x > col_w, wide)
	print("[", tag, "] column=", col_w, " dp · controls whose min.x exceeds it: ", wide.size())
	## Innermost first -- `_find_all` walks pre-order, so the tail of the list is
	## the deepest, and the deepest offender is the one that actually owns the
	## width. Every ancestor above it merely reports what it was handed.
	wide.reverse()
	for i in mini(8, wide.size()):
		var c := wide[i] as Control
		var extra := ""
		if c is Button:
			extra = " '" + (c as Button).text.substr(0, 30) + "'"
		elif c is Label:
			extra = " '" + (c as Label).text.substr(0, 30) + "'"
		print("[", tag, "]    ", c.get_class(), " min=", c.get_combined_minimum_size(),
			extra, "  @", String(root.get_path_to(c)))
	var scrolls: Array = []
	_find_all(root, func(n): return n is ScrollContainer, scrolls)
	for s in scrolls:
		var sc := s as ScrollContainer
		var inner := Vector2.ZERO
		for c in sc.get_children():
			if c is Control:
				inner = inner.max((c as Control).size)
		print("[", tag, "] scroll ", sc.size, " content=", inner,
			" scrolls=", inner.y > sc.size.y + 1.0, " deadzone=", sc.scroll_deadzone)

func _shot(tag: String) -> void:
	await _frames(3)
	var img := get_viewport().get_texture().get_image()
	var out := "user://ph9_%s.png" % tag
	img.save_png(out)
	print("shot ", ProjectSettings.globalize_path(out))
