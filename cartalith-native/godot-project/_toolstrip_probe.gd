extends Node
## The domain tool palette's move from each left dock's TOOLS block into the
## top tool bar (owner, 2026-09-23). Windowed, not headless -- it captures the
## framebuffer:
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _toolstrip_probe.tscn -- --out <dir> --tag <before|after>
##
## For WORLD (both modes), CIVIL and CARTO it: saves a full-window PNG; lists
## every visible tool-palette button and whether it sits in the top bar
## (`app.tool_options_bar`) or elsewhere (the left dock); then drives each one
## with a REAL mouse click through `push_input` and with its keyboard letter,
## and reads `app.armed_tool` back. Unknown arguments fail loudly.

const EXPECT := {
	"Inspect (V)": "inspect", "Measure (M)": "measure", "Region select (R)": "region",
	"Biome paint (B)": "paint", "Settlement (S)": "settlement", "Territory (T)": "territory",
	"Way (W)": "way", "Route (⇧R)": "route", "Icon (I)": "icon", "Label (L)": "label",
}
const KEYS := {
	"inspect": [KEY_V, false], "measure": [KEY_M, false], "region": [KEY_R, false],
	"paint": [KEY_B, false], "settlement": [KEY_S, false], "territory": [KEY_T, false],
	"way": [KEY_W, false], "route": [KEY_R, true], "icon": [KEY_I, false], "label": [KEY_L, false],
}

var app: Node
var out_dir := ""
var tag := ""
var fails := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	var a := OS.get_cmdline_user_args()
	var i := 0
	while i < a.size():
		match a[i]:
			"--out": out_dir = a[i + 1]; i += 2
			"--tag": tag = a[i + 1]; i += 2
			"--force-touch": i += 1  # read by DccShell itself (tablet run)
			_:
				printerr("TSP unknown argument: ", a[i])
				get_tree().quit(3)
				return
	if out_dir == "" or tag == "":
		printerr("TSP needs --out <dir> --tag <name>")
		get_tree().quit(3)
		return

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	var bridge = app.bridge
	bridge.generate({"seed": 131313, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	await _frames(10)
	if not bridge.has_world:
		printerr("TSP generate FAILED")
		get_tree().quit(2)
		return

	for step in [["world", "a"], ["world", "b"], ["civilization", ""], ["cartography", ""]]:
		var dom: String = step[0]
		if step[1] != "":
			app.select_domain_mode(dom, step[1])
		else:
			app.select_domain(dom)
		app._escape_action(true)
		await _frames(8)
		var name: String = dom + ("_" + String(step[1]) if String(step[1]) != "" else "")
		await _shot(name)
		await _drive(name)

	## Width: every context that claims the bar, with the strip prepended. The
	## bar sits in the shell's top-level VBox, so its minimum is the window's
	## minimum -- over the window width means the right dock is pushed off.
	var win: float = get_viewport().get_visible_rect().size.x
	for ctx in [["world", "b", "sculpt"], ["world", "b", "paint"], ["world", "a", "measure"],
			["civilization", "", "way"], ["civilization", "", "route"], ["civilization", "", "territory"],
			["civilization", "", "settlement"], ["cartography", "", "label"], ["cartography", "", "icon"],
			["cartography", "", "region"]]:
		if ctx[1] != "":
			app.select_domain_mode(ctx[0], ctx[1])
		else:
			app.select_domain(ctx[0])
		app._escape_action(true)
		await _frames(4)
		app.arm_tool(ctx[2])
		await _frames(6)
		var bar_min: float = maxf((app.tool_options_bar as Control).get_combined_minimum_size().x, (app.get("tool_palette_bar") as Control).get_combined_minimum_size().x if app.get("tool_palette_bar") != null else 0.0)
		var strip_min: float = (app.get("_tool_strip_row") as Control).get_combined_minimum_size().x if is_instance_valid(app.get("_tool_strip_row")) else -1.0
		var shell_min: float = (app.get_child(0) as Control).get_combined_minimum_size().x if app.get_child_count() > 0 and app.get_child(0) is Control else -1.0
		var over := bar_min > win
		print("TSP width %s/%s armed=%-10s bar_min=%4.0f strip_min=%4.0f window=%4.0f %s" % [ctx[0], ctx[1], ctx[2], bar_min, strip_min, win, "OVERFLOW" if over else "fits"])
		await _shot("ctx_%s_%s" % [ctx[0], ctx[2]])
		if over:
			fails += 1
	app._escape_action(true)
	## Theme toggle: the palette bar must re-tint with the rest of the chrome.
	app.select_domain("civilization")
	await _frames(4)
	app.toggle_theme()
	await _frames(6)
	await _shot("theme_toggled")
	app.toggle_theme()
	await _frames(4)
	print("TSP DONE fails=%d" % fails)
	get_tree().quit(1 if fails > 0 else 0)

func _shot(name: String) -> void:
	await RenderingServer.frame_post_draw
	var img: Image = get_viewport().get_texture().get_image()
	var path := out_dir.path_join("%s_%s.png" % [tag, name])
	img.save_png(path)
	print("TSP %s shot -> %s" % [name, path])

func _palette() -> Array:
	var found: Array = []
	var stack: Array = [app]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Button and n.has_meta(DccWidgets.TOOL_CAPTION_META) and (n as Button).is_visible_in_tree():
			found.append(n)
		stack.append_array(n.get_children())
	return found

func _drive(name: String) -> void:
	var bar: Control = app.get("tool_palette_bar") if app.get("tool_palette_bar") != null else app.tool_options_bar
	var btns := _palette()
	var tips: Array = []
	for b: Button in btns:
		var where := "TOPBAR" if bar.is_ancestor_of(b) else "DOCK"
		var r := b.get_global_rect()
		tips.append("%s@%s(%d,%d)" % [b.tooltip_text, where, r.position.x, r.position.y])
	print("TSP %s palette[%d]: %s" % [name, btns.size(), ", ".join(tips)])
	var insp := _find("Inspect (V)")
	var insp_lit: bool = insp != null and insp.button_pressed and app.armed_tool == "inspect"
	print("TSP %s Inspect lit after Esc: %s" % [name, "OK" if insp_lit else "FAIL"])
	if not insp_lit:
		fails += 1
	## Iterate tooltips, not buttons: the bar is rebuilt on every arm, so a
	## button captured before the first click is freed by the second.
	var tip_list: Array = []
	for b: Button in btns:
		tip_list.append(b.tooltip_text)
	for tip: String in tip_list:
		if not EXPECT.has(tip):
			continue  # the Pan legend
		var want: String = EXPECT[tip]
		# Real click, through the viewport's input path.
		var bb := _find(tip)
		if bb == null:
			print("TSP %s %s: button vanished before click" % [name, tip]); fails += 1; continue
		var c := bb.get_global_rect().get_center()
		for pressed in [true, false]:
			var ev := InputEventMouseButton.new()
			ev.button_index = MOUSE_BUTTON_LEFT
			ev.pressed = pressed
			ev.position = c
			ev.global_position = c
			get_viewport().push_input(ev)
			await _frames(2)
		var ok_click: bool = app.armed_tool == want
		app._escape_action(true)
		await _frames(3)
		# Keyboard letter.
		var k: Array = KEYS[want]
		var kev := InputEventKey.new()
		kev.keycode = k[0]
		kev.physical_keycode = k[0]
		kev.shift_pressed = k[1]
		kev.pressed = true
		get_viewport().push_input(kev)
		await _frames(2)
		var ok_key: bool = app.armed_tool == want
		var lit := _find(tip)
		var ok_lit: bool = lit != null and lit.button_pressed
		kev = kev.duplicate()
		kev.pressed = false
		get_viewport().push_input(kev)
		app._escape_action(true)
		await _frames(3)
		print("TSP %s %-18s -> %-10s click=%s key=%s lit=%s" % [name, tip, want,
			"OK" if ok_click else "FAIL", "OK" if ok_key else "FAIL", "OK" if ok_lit else "FAIL"])
		if not (ok_click and ok_key and ok_lit):
			fails += 1

func _find(tip: String) -> Button:
	for b: Button in _palette():
		if b.tooltip_text == tip:
			return b
	return null
