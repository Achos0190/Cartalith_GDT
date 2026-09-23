extends Node
## UX review capture (2026-09-23, review-only, not committed). WINDOWED.
##   Godot_v4.7.1-stable_win64_console.exe --path . _uxreview_probe.tscn --resolution 1920x1080 -- <outdir> [vault_dir]
## Boots the shell, generates a world, captures WORLD/CIVIL/CARTO, then opens
## the vault browser against `vault_dir`, measures its geometry, and drives a
## REAL border drag through Input.parse_input_event to test resize.
## Never emits `store_changed`, so nothing is persisted to user://.

var _app: Node
var _out := ""

func _find(n: Node, cls) -> Node:
	if is_instance_of(n, cls):
		return n
	for c in n.get_children(true):
		var r := _find(c, cls)
		if r != null:
			return r
	return null

func _find_all(n: Node, cls, out: Array) -> void:
	if is_instance_of(n, cls):
		out.append(n)
	for c in n.get_children(true):
		_find_all(c, cls, out)

func _shot(name: String) -> void:
	await RenderingServer.frame_post_draw
	await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	img.save_png(_out + "/" + name + ".png")
	print("UX shot %s %dx%d" % [name, img.get_width(), img.get_height()])

func _mouse(pos: Vector2, pressed: int) -> void:
	# pressed: 1 down, 0 up, -1 motion (button held)
	if pressed == -1:
		var m := InputEventMouseMotion.new()
		m.position = pos
		m.global_position = pos
		m.button_mask = MOUSE_BUTTON_MASK_LEFT
		Input.parse_input_event(m)
	else:
		var b := InputEventMouseButton.new()
		b.position = pos
		b.global_position = pos
		b.button_index = MOUSE_BUTTON_LEFT
		b.pressed = pressed == 1
		b.button_mask = MOUSE_BUTTON_MASK_LEFT if pressed == 1 else 0
		Input.parse_input_event(b)
	Input.flush_buffered_events()
	await get_tree().process_frame
	await get_tree().process_frame

func _drag(from: Vector2, to: Vector2) -> void:
	await _mouse(from, -1)
	await _mouse(from, 1)
	for i in range(1, 9):
		await _mouse(from.lerp(to, i / 8.0), -1)
	await _mouse(to, 0)

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("UX REFUSED headless")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	_out = args[0] if args.size() > 0 else OS.get_environment("TEMP")
	var vault_dir: String = args[1] if args.size() > 1 else ""
	var wd := Timer.new()
	wd.wait_time = 400.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	print("UX viewport %s" % str(get_viewport().get_visible_rect().size))
	await _shot("00_launch")
	_app.open_project_dialog.hide()
	var bridge = _app.bridge
	bridge.generate({"seed": 883120, "width_km": 2000.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	await _shot("01_world")
	_app.select_domain("civilization")
	await get_tree().create_timer(0.5).timeout
	await _shot("02_civil")
	_app.select_domain("cartography")
	await get_tree().create_timer(0.5).timeout
	await _shot("03_carto")
	_app.select_domain("world")
	await get_tree().create_timer(0.3).timeout

	# -- vault --------------------------------------------------------------
	if vault_dir != "":
		var conn: Dictionary = bridge.vault_connect(vault_dir, "ReviewVault")
		print("UX vault connect %s" % str(conn))
	_app.open_vault_browse()
	await get_tree().create_timer(0.5).timeout
	var vw: Window = _find(_app, VaultWindow)
	print("UX vault pos=%s size=%s min=%s max=%s unresizable=%s wrap=%s borderless=%s" % [
		vw.position, vw.size, vw.min_size, vw.max_size, vw.unresizable, vw.wrap_controls, vw.borderless])
	var _sps: Array = []
	_find_all(vw, HSplitContainer, _sps)
	print("UX DBG browse_split=%s offsets=%s placed=%s rowsize=%s" % [vw._browse_split, (_sps[0] as HSplitContainer).split_offsets if not _sps.is_empty() else "none", (_sps[0] as Node).has_meta("placed") if not _sps.is_empty() else "-", (_sps[0] as Control).size if not _sps.is_empty() else "-"])
	print("UX ok_button visible=%s text=%s" % [vw.get_ok_button().visible, vw.get_ok_button().text])
	var trees: Array = []
	_find_all(vw, Tree, trees)
	for t in trees:
		print("UX tree rect=%s" % str((t as Control).get_global_rect()))
	var scrolls: Array = []
	_find_all(vw, ScrollContainer, scrolls)
	for s in scrolls:
		var sc := s as ScrollContainer
		print("UX scroll rect=%s content_h=%s" % [str(sc.get_global_rect()), str(sc.get_child(0).size.y if sc.get_child_count() > 0 else -1)])
	await _shot("10_vault_browse_default")

	# pick a file with headings, and open the editor
	var files: PackedStringArray = bridge.vault_list_files(2000)
	print("UX vault files=%d" % files.size())
	var pick := ""
	for f in files:
		if bridge.vault_file_headings(f).size() >= 3:
			pick = f
			break
	if pick != "":
		vw._pick_file = pick
		vw._rebuild()
		await get_tree().create_timer(0.3).timeout
		await _shot("11_vault_browse_picked")
		var sc0 := scrolls[0] as ScrollContainer if not scrolls.is_empty() else null
		scrolls.clear()
		_find_all(vw, ScrollContainer, scrolls)
		for s in scrolls:
			var sc := s as ScrollContainer
			print("UX picked scroll rect=%s content_h=%s" % [str(sc.get_global_rect()), str(sc.get_child(0).size.y if sc.get_child_count() > 0 else -1)])

	# -- REAL resize drag on the bottom-right corner --------------------------
	var before := vw.size
	var br := Vector2(vw.position + vw.size) + Vector2(2, 2)
	await _drag(br, br + Vector2(500, 150))
	await get_tree().create_timer(0.3).timeout
	print("UX drag BR +500,+150: size %s -> %s" % [before, vw.size])
	await _shot("12_vault_after_corner_drag")
	# right edge
	var before2 := vw.size
	var re := Vector2(vw.position.x + vw.size.x + 2, vw.position.y + vw.size.y / 2)
	await _drag(re, re + Vector2(400, 0))
	await get_tree().create_timer(0.3).timeout
	print("UX drag R +400: size %s -> %s" % [before2, vw.size])
	# title drag (move)
	var p0 := vw.position
	var tb := Vector2(vw.position.x + 120, vw.position.y - 12)
	await _drag(tb, tb + Vector2(-300, -40))
	await get_tree().create_timer(0.3).timeout
	print("UX drag title: pos %s -> %s" % [p0, vw.position])
	await _shot("13_vault_after_move")

	trees.clear()
	_find_all(vw, Tree, trees)
	for t in trees:
		print("UX after-drag tree rect=%s" % str((t as Control).get_global_rect()))
	var splits: Array = []
	_find_all(vw, HSplitContainer, splits)
	for sp in splits:
		for ch in (sp as Node).get_children():
			print("UX split child %s rect=%s" % [(ch as Node).get_class(), str((ch as Control).get_global_rect())])
	print("UX ok text=%s" % vw.get_ok_button().text)
	# divider drag, then a pick (which rebuilds): does the divider stay?
	if not splits.is_empty():
		var tc := (splits[0] as Node).get_child(0) as Control
		var r := tc.get_global_rect()
		var dv := Vector2(r.end.x + 6, r.position.y + r.size.y / 2) + Vector2(vw.position)
		await _drag(dv, dv + Vector2(150, 0))
		await get_tree().create_timer(0.3).timeout
		print("UX divider drag +150: tree w %s -> %s" % [r.size.x, tc.size.x])
		if files.size() > 1:
			vw._pick_file = files[1]
			vw._rebuild()
			await get_tree().create_timer(0.4).timeout
			var sp2: Array = []
			_find_all(vw, HSplitContainer, sp2)
			print("UX after re-pick tree w %s" % ((sp2[0] as Node).get_child(0) as Control).size.x)
		await _shot("13b_vault_after_divider")

	# -- overview mode (Data > Markdown vault...) -----------------------------
	vw.hide()
	_app.open_vault_overview()
	await get_tree().create_timer(0.5).timeout
	scrolls.clear()
	_find_all(vw, ScrollContainer, scrolls)
	for s in scrolls:
		var sc := s as ScrollContainer
		print("UX overview scroll rect=%s content_h=%s" % [str(sc.get_global_rect()), str(sc.get_child(0).size.y if sc.get_child_count() > 0 else -1)])
	await _shot("15_vault_overview")
	# scoped to a settlement
	var st: Array = bridge.settlements()
	if not st.is_empty():
		vw.hide()
		_app.open_vault("settlement", int(st[0].get("tid", 0)), String(st[0].get("name", "")))
		await get_tree().create_timer(0.5).timeout
		scrolls.clear()
		_find_all(vw, ScrollContainer, scrolls)
		for s in scrolls:
			var sc := s as ScrollContainer
			print("UX scoped scroll rect=%s content_h=%s" % [str(sc.get_global_rect()), str(sc.get_child(0).size.y if sc.get_child_count() > 0 else -1)])
		await _shot("16_vault_scoped")
	vw.hide()

	# -- which other windows are resizable / capped ----------------------------
	var wins: Array = []
	_find_all(_app, AcceptDialog, wins)
	for w in wins:
		var d := w as AcceptDialog
		print("UX win %s title=%s size=%s min=%s max=%s unresizable=%s ok_visible=%s" % [
			d.get_class() if d.get_script() == null else d.get_script().get_global_name(),
			d.title, d.size, d.min_size, d.max_size, d.unresizable, d.get_ok_button() != null and d.get_ok_button().visible])
	# -- title bar over the MAP: is the band/close drawn? ---------------------
	_app.open_vault_browse()
	vw.size = Vector2i(760, 600)
	vw.position = Vector2i(600, 260)
	await get_tree().create_timer(0.4).timeout
	await _shot("40_vault_title_over_map")
	vw.hide()
	_app.open_place_editor(0)
	await get_tree().create_timer(0.5).timeout
	await _shot("41_place_editor")
	for w2 in wins:
		(w2 as Window).hide()
	_app.open_faction_roster()
	await get_tree().create_timer(0.5).timeout
	await _shot("42_faction_roster")
	for w2 in wins:
		(w2 as Window).hide()
	await get_tree().create_timer(0.2).timeout
	# -- Data menu opened by a real click on its title ------------------------
	await _mouse(Vector2(254, 17), -1)
	await _mouse(Vector2(254, 17), 1)
	await _mouse(Vector2(254, 17), 0)
	await get_tree().create_timer(0.4).timeout
	await _shot("20_data_menu")
	var esc := InputEventKey.new()
	esc.keycode = KEY_ESCAPE
	esc.pressed = true
	Input.parse_input_event(esc)
	await get_tree().create_timer(0.3).timeout
	# -- dark palette (not persisted) -----------------------------------------
	_app.menus._apply_theme_mode("dark")
	await get_tree().create_timer(0.6).timeout
	await _shot("30_dark_world")
	_app.select_domain("civilization")
	await get_tree().create_timer(0.5).timeout
	await _shot("31_dark_civil")
	_app.select_domain("world")
	_app.open_vault_browse()
	await get_tree().create_timer(0.5).timeout
	await _shot("32_dark_vault_browse")
	if pick != "":
		vw._pick_file = pick
		vw._rebuild()
		await get_tree().create_timer(0.3).timeout
		await _shot("33_dark_vault_picked")
	vw.hide()
	_app.menus._apply_theme_mode("light")
	await get_tree().create_timer(0.3).timeout
	# -- census: visible text that names code (all three domains built) ------
	var rx := RegEx.new()
	rx.compile("\\.(gd|rs)\\b|`[A-Za-z_.:()]+`|\\b[a-z]+_[a-z_]+\\(")
	var labels: Array = []
	var hits := 0
	var tips := 0
	var tips_all := 0
	var tip_samples: Array = []
	var samples: Array = []
	for dom in ["world", "civilization", "cartography"]:
		_app.select_domain(dom)
		await get_tree().create_timer(0.3).timeout
		labels.clear()
		_find_all(_app, Control, labels)
		for n in labels:
			var t := ""
			if n is Label: t = (n as Label).text
			elif n is Button: t = (n as Button).text
			elif n is RichTextLabel: t = (n as RichTextLabel).text
			if t != "" and rx.search(t) != null and (n as Control).is_visible_in_tree():
				hits += 1
				if samples.size() < 12: samples.append("[%s] %s" % [dom, t.substr(0, 140).replace(char(10), " ")])
			var tt := (n as Control).tooltip_text
			if tt != "": tips_all += 1
			if tt != "" and rx.search(tt) != null:
				tips += 1
				if tip_samples.size() < 6 and dom == "world": tip_samples.append(tt.substr(0, 160).replace(char(10), " "))
	print("UX census visible-text-with-code-identifiers=%d tooltips-with-code=%d of %d (3 domain passes, counted per pass)" % [hits, tips, tips_all])
	for ts in tip_samples:
		print("UX  tip " + ts)
	for sm in samples:
		print("UX   " + sm)
	print("UX DONE")
	get_tree().quit(0)
