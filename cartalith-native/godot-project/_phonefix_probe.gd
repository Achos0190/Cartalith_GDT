extends Node
## Committed verification harness for the four phone-only findings fixed in
## this pass. Drives the *real shell* at handset size and measures the four
## controls rather than eyeballing a screenshot.
##
##   godot4 --path . --resolution 393x852 _phonefix_probe.tscn -- --force-touch --nowelcome
##
## `--force-touch` is `dcc_shell.gd`'s own testing override; without it the
## phone composition is unreachable on a dev box with no touch hardware.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var app: Node

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
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout

	print("=== phone=", app.is_phone(), " scale=", app.phone_scale(), " ===")

	# --- 4. Welcome / open-project dialog -----------------------------------
	var dlg: Window = app.open_project_dialog
	print("[4] dialog visible=", dlg.visible, " size=", dlg.size,
		" scale=", dlg.content_scale_factor, " wrap=", dlg.wrap_controls,
		" borderless=", dlg.borderless)
	var le := _find(dlg, func(n): return n is LineEdit) as LineEdit
	if le != null:
		print("[4] search LineEdit size=", le.size, " min=", le.get_combined_minimum_size(),
			" placeholder='", le.placeholder_text, "'")
		var chips: Array = []
		_find_all(dlg, func(n): return n is Button and (n as Button).text in ["Recent", "All worlds", "Shared"], chips)
		for c in chips:
			print("[4]   chip '", (c as Button).text, "' rect=", (c as Control).get_global_rect())
		print("[4] well rect=", le.get_parent().get_parent().get_parent().get_global_rect())
	_dump_min(dlg, 0, 300.0)
	print("[4] visible_rect=", dlg.get_visible_rect(), " content_scale_size=", dlg.content_scale_size)
	for c in dlg.get_children():
		if c is Control:
			print("[4] direct child ", c.get_class(), " rect=", (c as Control).get_rect(),
				" min=", (c as Control).get_combined_minimum_size())
	await _shot("welcome")
	dlg.hide()
	await _frames(2)

	# --- 1. RichTextLabel font walk -----------------------------------------
	# Planted in the left dock so `_on_phone_node_added` picks it up exactly as
	# it picks up the right dock's real "Why here?" block, which needs a world
	# and a selected settlement to exist.
	var rt := RichTextLabel.new()
	rt.bbcode_enabled = true
	rt.fit_content = true
	rt.add_theme_font_size_override("normal_font_size", 11)
	app.left_dock.add_child(rt)
	await _frames(4)
	print("[1] RichTextLabel normal_font_size after walk=",
		rt.get_theme_font_size("normal_font_size"),
		" (authored 11, expect round(11*", app.phone_scale(), "))")
	rt.queue_free()

	# --- 2. TOOLS block glyphs ----------------------------------------------
	app._set_sheet_open("left", true)
	await get_tree().create_timer(0.6).timeout
	for domain in ["world", "civilization", "cartography"]:
		app._select_domain(domain)
		await get_tree().create_timer(0.5).timeout
		var tools: Array = []
		_find_all(app.left_dock, func(n): return n is Button and n.has_meta("dcc_tool_caption"), tools)
		print("[2] domain=", domain, " tool buttons=", tools.size())
		for t in tools:
			var b := t as Button
			print("[2]   '", b.text, "' rect=", b.get_global_rect(),
				" icon=", (b.icon.get_size() if b.icon != null else Vector2.ZERO),
				" fs=", b.get_theme_font_size("font_size"),
				" bordered=", b.get_theme_stylebox("normal") is StyleBoxFlat)
		await _shot("tools_" + domain)

	# --- 3. PAINT class OptionButton ----------------------------------------
	app._set_sheet_open("left", false)
	await _frames(3)
	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 900:
		await get_tree().process_frame
		waited += 1
	print("[3] has_world=", app.bridge.has_world)
	await _frames(6)
	var bar = DccToolBar.instance()
	if bar != null:
		bar.mode = "paint"
	app.arm_tool("paint")
	if bar != null:
		bar.rebuild()
	await get_tree().create_timer(0.6).timeout
	var obs: Array = []
	_find_all(app.tool_options_row, func(n): return n is OptionButton, obs)
	print("[3] tool_options_row OptionButtons=", obs.size())
	for o in obs:
		var ob := o as OptionButton
		print("[3]   text='", ob.text, "' size=", ob.size,
			" min=", ob.get_combined_minimum_size(),
			" fit_longest=", ob.fit_to_longest_item, " clip=", ob.clip_text)
	await _shot("paintopts")

	print("=== done ===")
	get_tree().quit()

## Every node whose combined minimum width exceeds the column, innermost first
## -- the technique `dcc_widgets.gd`'s own header records for the roster.
func _dump_min(node: Node, depth: int, limit: float) -> void:
	if node is Control:
		var w: float = (node as Control).get_combined_minimum_size().x
		if w > limit:
			var extra := ""
			if node is Label:
				extra = " '" + (node as Label).text.substr(0, 40) + "'"
			elif node is Button:
				extra = " '" + (node as Button).text.substr(0, 40) + "'"
			print("[min] ", " ".repeat(depth), node.get_class(), " min.x=", w, extra)
	for c in node.get_children():
		_dump_min(c, depth + 1, limit)

func _shot(name: String) -> void:
	await _frames(3)
	var img := get_viewport().get_texture().get_image()
	var out := "user://phonefix_%s.png" % name
	img.save_png(out)
	print("shot ", ProjectSettings.globalize_path(out))
