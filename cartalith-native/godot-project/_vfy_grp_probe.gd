extends Node
## VERIFIER, WINDOWED. Independent re-derivation of Lane B's group()-header
## claims. Boots the real shell, finds group headers STRUCTURALLY (not by
## label), and measures both sides of the ledger by reverting the shipped
## change at runtime -- the same direction Lane B used, because staging a
## candidate on an unchanged tree measures a tree nobody runs.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 \
##       _vfy_grp_probe.tscn

const SEEDS := [483920, 77021, 4242]

## `civilization` carries the L2 accordion that holds one of the two headers
## Lane B named (CIVIL > Military). The rest are visited at their default.
const NODES := [
	["world", "a"], ["world", "b"],
	["civilization", "landmarks"], ["civilization", "factions"],
	["civilization", "infra"], ["civilization", "planner"],
	["cartography", "style"], ["cartography", "labels"],
	["cartography", "icons"], ["cartography", "terrain"],
]

var app: Node
var _fail := 0
var _rel := 0             ## relationship assertions run
var _binders: Dictionary = {}   ## header text -> [min_off, floor, stop_off, stop_on]
var _widths: Dictionary = {}    ## header text -> widest min.x with autowrap OFF
var _files_seen: Dictionary = {}


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _chk(cond: bool, what: String) -> void:
	if not cond:
		_fail += 1
		print("VG FAIL  " + what)

func _grab() -> Image:
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()


## Structural, per Lane B's own correction: a flat unfocusable Button at the
## group-header font size whose NEXT sibling is the MarginContainer holding a
## VBoxContainer body. Deliberately makes NO claim about the label, so a
## runtime text rewrite cannot move it in or out.
func _is_group_header(c: Node) -> bool:
	if not (c is Button):
		return false
	var b := c as Button
	if not b.flat or b.focus_mode != Control.FOCUS_NONE:
		return false
	var want: int = DccTheme.role_px("fs_dock_header") if DccTheme.is_tablet() else DccTheme.FS_HEADER
	if b.get_theme_font_size("font_size") != want:
		return false
	var p := b.get_parent()
	if p == null or b.get_index() + 1 >= p.get_child_count():
		return false
	var pad := p.get_child(b.get_index() + 1)
	if not (pad is MarginContainer) or pad.get_child_count() == 0:
		return false
	return pad.get_child(0) is VBoxContainer


func _shares_width(parent: Node) -> bool:
	return ((parent is BoxContainer) and not (parent as BoxContainer).vertical) \
		or (parent is HFlowContainer) \
		or (parent is GridContainer)


func _headers(root: Node, out: Array) -> void:
	for c in root.get_children():
		if _is_group_header(c):
			out.append(c)
		_headers(c, out)


## Flip every live header's autowrap. `on` = the shipped tree, `off` = exactly
## what `group()` produced before this batch.
func _apply(root: Node, on: bool) -> int:
	var n := 0
	var hs: Array = []
	_headers(root, hs)
	for c in hs:
		if not _shares_width(c.get_parent()):
			(c as Button).autowrap_mode = (TextServer.AUTOWRAP_WORD_SMART if on
				else TextServer.AUTOWRAP_OFF)
			n += 1
	return n


func _drag_to(dock: Control, floor_px: int) -> float:
	var was := dock.custom_minimum_size.x
	dock.custom_minimum_size.x = float(floor_px)
	await _frames(4)
	var got := dock.size.x
	dock.custom_minimum_size.x = was
	await _frames(3)
	return got


## The relationships a shared-widget re-base invalidates, asserted on EVERY
## header found, whichever file built it.
func _relationships(tag: String, hs: Array) -> void:
	for c in hs:
		var b := c as Button
		var p := b.get_parent()
		var pad := p.get_child(b.get_index() + 1) as MarginContainer
		var body := pad.get_child(0) as VBoxContainer
		var txt := String(b.text)
		var h0 := b.size.y
		var vis0 := body.visible
		b.emit_signal("pressed")
		await _frames(3)
		_rel += 1
		_chk(body.visible != vis0, "%s: toggling '%s' did not flip its own body" % [tag, txt])
		_chk(is_equal_approx(b.size.y, h0),
			"%s: '%s' header height moved with body visibility (%.0f -> %.0f)" % [tag, txt, h0, b.size.y])
		b.emit_signal("pressed")
		await _frames(3)
		_chk(body.visible == vis0, "%s: '%s' body did not round-trip" % [tag, txt])
		_chk(String(b.text) == txt, "%s: '%s' header text did not round-trip" % [tag, txt])
		_chk(not (b.clip_text), "%s: '%s' is clipping text (DS-03 says reflow)" % [tag, txt])
		_chk(b.autowrap_mode == TextServer.AUTOWRAP_WORD_SMART,
			"%s: '%s' is not autowrapping in a vertical parent" % [tag, txt])
		_files_seen[txt] = true


func _survey(tag: String, dock: Control, body: Control, floor_px: int, do_rel: bool) -> void:
	await _frames(8)
	var hs: Array = []
	_headers(body, hs)
	if hs.is_empty():
		return
	# Shipped (autowrap ON): what does the dock stop at?
	var stop_on: float = await _drag_to(dock, floor_px)
	# Reverted (autowrap OFF): the pre-batch tree.
	_apply(app, false)
	await _frames(6)
	for c in hs:
		var b := c as Button
		var w: float = b.get_minimum_size().x
		var t := String(b.text)
		if not _widths.has(t) or float(_widths[t]) < w:
			_widths[t] = w
	var stop_off: float = await _drag_to(dock, floor_px)
	_apply(app, true)
	await _frames(6)
	# Which headers, if any, are wide enough to be the binding node?
	for c in hs:
		var b := c as Button
		if not _shares_width(b.get_parent()):
			pass
	if stop_off > float(floor_px) + 0.5:
		# name any header whose OFF minimum alone exceeds the floor
		for c in hs:
			var b := c as Button
			var w: float = float(_widths.get(String(b.text), 0.0))
			if w > float(floor_px) - 35.0:   # dock chrome allowance; reported, not asserted
				_binders[String(b.text)] = [w, floor_px, stop_off, stop_on]
	print("VG %-42s headers=%2d  floor=%3d  stops: shipped=%3.0f reverted=%3.0f  %s"
		% [tag, hs.size(), floor_px, stop_on, stop_off,
			"IMPROVED" if stop_on < stop_off - 0.5 else ""])
	if do_rel:
		await _relationships(tag, hs)


func _categories(body: Node) -> Array:
	var out: Array = []
	for c in body.get_children():
		if c is Button and String((c as Button).text).begins_with(DccIcons.SYMBOLS["caret"]):
			out.append(c)
		out.append_array(_categories(c))
	return out


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 1200.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("VG WATCHDOG")
		get_tree().quit(2))
	wd.start()

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	print("VG density is_tablet=%s is_laptop=%s  left floor=%d right floor=%d  viewport=%s"
		% [DccTheme.is_tablet(), DccTheme.is_laptop(), DccTheme.W_LEFT_DOCK_MIN,
			DccTheme.W_RIGHT_DOCK_MIN, str(get_viewport().size)])

	var bridge = app.bridge
	var rd = app.right_dock_ctrl
	var shot_done := false

	for seed_v in SEEDS:
		bridge.generate({"seed": seed_v, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
			"archetype": "", "villages": true, "sea_level": 0.45})
		var waited := 0
		while bridge.generating and waited < 3000:
			await get_tree().process_frame
			waited += 1
		await _frames(10)
		if not bridge.has_world:
			print("VG seed %d: generate FAILED" % seed_v)
			continue
		print("VG ============ seed %d ============" % seed_v)

		for pair in NODES:
			app.select_domain_mode(String(pair[0]), String(pair[1]))
			await _frames(14)
			var tag0 := "LEFT %s/%s" % [pair[0], pair[1]]
			await _survey(tag0, app.left_dock, app.left_dock_body,
				DccTheme.W_LEFT_DOCK_MIN, seed_v == SEEDS[0])
			if String(pair[0]) == "civilization":
				for cb in _categories(app.left_dock_body):
					if not is_instance_valid(cb):
						continue
					var label := String((cb as Button).text).substr(2).strip_edges()
					(cb as Button).emit_signal("pressed")
					await _frames(10)
					await _survey("%s > %s" % [tag0, label], app.left_dock,
						app.left_dock_body, DccTheme.W_LEFT_DOCK_MIN, false)

		var gs: Vector2i = bridge.grid_size()
		bridge.route_begin("mixed")
		bridge.route_append_stop(gs.x * 0.20, gs.y * 0.30)
		bridge.route_append_stop(gs.x * 0.55, gs.y * 0.50)
		bridge.route_append_stop(gs.x * 0.82, gs.y * 0.72)
		bridge.route_commit()
		app.select_domain("civilization")
		await _frames(4)
		var settlements: Array = bridge.settlements()
		if not settlements.is_empty():
			rd.on_settlement_selected(settlements[0], 0)
		await _frames(8)
		await _survey("RIGHT settlement", app.right_dock, app.right_dock_body,
			DccTheme.W_RIGHT_DOCK_MIN, seed_v == SEEDS[0])
		app.arm_tool("journey")
		await _frames(16)
		await _survey("RIGHT settlement+journey", app.right_dock, app.right_dock_body,
			DccTheme.W_RIGHT_DOCK_MIN, seed_v == SEEDS[0])

		# -- COST: framebuffer, shipped vs reverted, at the shipped widths ----
		if not shot_done:
			shot_done = true
			await _frames(8)
			var shipped := await _grab()
			var n := _apply(app, false)
			await _frames(10)
			var reverted := await _grab()
			_apply(app, true)
			await _frames(8)
			var same: bool = shipped.get_data() == reverted.get_data()
			print("VG COST  headers reverted=%d  full-frame bytes identical=%s  (%dx%d)"
				% [n, same, shipped.get_width(), shipped.get_height()])
			_chk(same, "the change is NOT byte-identical at the shipped dock widths")
			# positive control: toggle one group, must move pixels
			var hs: Array = []
			_headers(app.right_dock_body, hs)
			if hs.is_empty():
				_headers(app.left_dock_body, hs)
			if not hs.is_empty():
				(hs[0] as Button).emit_signal("pressed")
				await _frames(10)
				var toggled := await _grab()
				var moved := 0
				for y in range(0, shipped.get_height(), 2):
					for x in range(0, shipped.get_width(), 2):
						if shipped.get_pixel(x, y) != toggled.get_pixel(x, y):
							moved += 1
				print("VG CONTROL  one group toggled -> %d differing px (every 2nd px sampled)" % moved)
				_chk(moved > 0, "positive control moved no pixels -- the harness proves nothing")
				(hs[0] as Button).emit_signal("pressed")
				await _frames(6)

	print("")
	print("VG ---- widest group headers measured with autowrap OFF (pre-batch minimum) ----")
	var items: Array = []
	for k in _widths:
		items.append([float(_widths[k]), k])
	items.sort_custom(func(a, b): return a[0] > b[0])
	for i in mini(12, items.size()):
		print("VG   %6.1f px   %s" % [items[i][0], items[i][1]])
	print("VG distinct group headers seen: %d" % items.size())
	print("VG headers whose pre-batch minimum is near/over a dock floor:")
	for k in _binders:
		print("VG   %s -> min=%.1f floor=%d  reverted stop=%.0f  shipped stop=%.0f"
			% [k, _binders[k][0], _binders[k][1], _binders[k][2], _binders[k][3]])
	print("VG relationship assertions run: %d" % _rel)
	print("VG RESULT: %d FAIL" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
