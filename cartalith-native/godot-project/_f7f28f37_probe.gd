extends Node
## Probe for owner findings F7, F28 and F37 (2026-11-21). Windowed (never --headless), a scratch APPDATA, under the lock:
##   godot --path . --resolution 1600x900 _f7f28f37_probe.tscn -- [--force-touch --density 1.0] --shots DIR
## Every expectation is a literal. Real mouse and key events drive the menus, the toggle and the map clicks.
##   F7:  Preferences ▸ GPU acceleration is ticked on a PC, its readout row names the adapter and the two GPU kernels
##        after a Generate, and the toggle (real key press) turns the readout to "GPU off".
##   F28: under a World-Structure archetype Plates / Drift stay editable, hold the archetype's derived 26 / 1.6, and a
##        hand edit (real click on the Plates slider) survives a Generate.
##   F37: with Snap to water ticked, a real click on dry land 3-5 cells from the sea lands the settlement on a cell
##        touching the sea; with it off the same click stays where it was clicked.
var _fails := 0
var _checks := 0
var _app: Node
var _b
var _shots := ""
var _mode := "desktop"

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1

func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame

func _move(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)
	await _settle(2)

func _click_at(at: Vector2) -> void:
	await _move(at)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(3)

func _key(code: Key) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(2)

func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img != null:
		DirAccess.make_dir_recursive_absolute(_shots)
		img.save_png(_shots.path_join("f7f28f37_%s_%s.png" % [_mode, tag]))

func _gen(seed_: int, w_km: float, gw: int, gh: int) -> void:
	_b.generate({"seed": seed_, "width_km": w_km, "grid_w": gw, "grid_h": gh, "sea_level": 0.5})
	var n := 0
	while _b.generating and n < 4000:
		await get_tree().process_frame
		n += 1
	await _settle(30)

func _menu_button(title: String) -> MenuButton:
	for ch in _app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == title:
			return ch
	return null

func _index_of(p: PopupMenu, prefix: String) -> int:
	for i in p.item_count:
		if not p.is_item_separator(i) and p.get_item_text(i).strip_edges().begins_with(prefix):
			return i
	return -1

func _prefs_popup() -> PopupMenu:
	return _menu_button("Preferences").get_popup()

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := args[i]
		if a == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		elif a == "--force-touch" or a.begins_with("--density") or (i > 0 and args[i - 1] == "--density"):
			pass
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		i += 1
	get_tree().create_timer(500.0).timeout.connect(func() -> void:
		print("WATCHDOG")
		get_tree().quit(2))
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_b = _app.get("bridge")
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, viewport %s ==" % [_mode, get_viewport().get_visible_rect().size])
	await _gen(133742, 700.0, 384, 288)
	await _part_f7()
	await _part_f37()
	await _part_f28()
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

# -- F7 ---------------------------------------------------------------------------------------------

func _part_f7() -> void:
	print("== F7: GPU row and readout ==")
	_ok(bool(_b.param_get("use_gpu")) == (_mode == "desktop"), "use_gpu boots on for desktop, off on touch (Ruling EY)")
	var stages: PackedStringArray = _b.gpu_stages_used()
	if _mode != "desktop":
		_ok(stages.is_empty(), "touch: no GPU stage ran")
		return
	_ok(stages == PackedStringArray(["plate_assignment", "base_field_blur"]), "the last generate ran exactly the two ruled kernels (%s)" % str(stages))
	_ok(_b.gpu_last_adapter().length() > 3, "the engine names the adapter that ran (%s)" % _b.gpu_last_adapter())
	await _click_at(_menu_button("Preferences").get_global_rect().get_center())
	var p := _prefs_popup()
	_ok(p.visible, "Preferences opened by a real click")
	var gi := _index_of(p, "GPU acceleration")
	_ok(gi >= 0 and p.is_item_checked(gi), "the GPU acceleration row is ticked")
	var txt := p.get_item_text(gi + 1)
	print("  info readout: ", txt)
	_ok(txt.contains(_b.gpu_last_adapter()) and txt.contains("plate assignment") and txt.contains("plate-height blur"), "the readout row names the adapter and both kernels")
	await _shot("prefs_gpu_on")
	var guard := 0
	while p.get_focused_item() != gi and guard < 80:
		await _key(KEY_DOWN)
		guard += 1
	await _key(KEY_ENTER)
	_ok(not bool(_b.param_get("use_gpu")), "a real Enter on the row turns use_gpu off")
	_ok(p.get_item_text(gi + 1).contains("GPU off"), "the readout follows: " + p.get_item_text(gi + 1))
	await _shot("prefs_gpu_off")
	if not p.visible:
		await _click_at(_menu_button("Preferences").get_global_rect().get_center())
	guard = 0
	while p.get_focused_item() != gi and guard < 80:
		await _key(KEY_DOWN)
		guard += 1
	await _key(KEY_ENTER)
	_ok(bool(_b.param_get("use_gpu")) and not p.get_item_text(gi + 1).contains("GPU off"), "and back on")
	await _key(KEY_ESCAPE)
	if p.visible:
		p.hide()
	await _settle(4)

# -- F37 --------------------------------------------------------------------------------------------

var _sea := -1.0

func _is_sea(gx: int, gy: int) -> bool:
	if _sea < 0.0:
		_sea = float(_b.world_gen.world_summary().get("sea_level", 0.5))
	var s: Dictionary = _b.sample_cell(gx, gy)
	return not s.is_empty() and float(s.get("elevation", 1.0)) < _sea

func _is_river(gx: int, gy: int) -> bool:
	## The engine's river test is flow above `river_flow_thresh` (0.0004 x cells = 44.2 at 384x288 with a unit detail factor),
	## which also holds undrawn headwater cells; the drawn rivers (river_order) are a subset.
	var q: Dictionary = _b.sample_cell(gx, gy)
	return int(q.get("river_order", 0)) > 0 or float(q.get("drainage", 0.0)) >= 44.2

## A dry cell whose nearest sea cell is 3..4 cells away and which does not touch the sea itself.
func _find_inland_site(gw: int, gh: int, avoid: Vector2i = Vector2i(-100, -100)) -> Vector2i:
	var best := Vector2i(-1, -1)
	for gy in range(20, gh - 20, 3):
		for gx in range(20, gw - 20, 3):
			if _is_sea(gx, gy):
				continue
			var near := 99
			for dy in range(-4, 5):
				for dx in range(-4, 5):
					if (dx != 0 or dy != 0) and _is_sea(gx + dx, gy + dy):
						near = mini(near, maxi(absi(dx), absi(dy)))
			if near >= 3 and near <= 4 and (absi(gx - avoid.x) > 20 or absi(gy - avoid.y) > 20):
				var settled := false
				for s in _b.settlements():
					if absf(float(s["x"]) - gx) < 12.0 and absf(float(s["y"]) - gy) < 12.0:
						settled = true
				if not settled:
					return Vector2i(gx, gy)
	return best

func _touches_sea(gx: int, gy: int, rivers: bool = false) -> bool:
	for dy in range(-1, 2):
		for dx in range(-1, 2):
			if (dx != 0 or dy != 0) and (_is_sea(gx + dx, gy + dy) or (rivers and _is_river(gx + dx, gy + dy))):
				return true
	return false

func _snap_toggle() -> CheckBox:
	for n in _app.find_children("*", "CheckBox", true, false):
		var row := (n as CheckBox).get_parent()
		if row != null:
			for ch in row.get_children():
				if ch is Label and (ch as Label).text == "Snap to water":
					return n
	return null

func _click_site(site: Vector2i) -> void:
	var host: Control = _app.viewport
	var pt: Vector2 = host.grid_to_view(float(site.x) + 0.5, float(site.y) + 0.5)
	await _click_at(host.get_global_transform() * pt)

func _part_f37() -> void:
	print("== F37: Snap to water ==")
	var site := _find_inland_site(384, 288)
	var site2 := _find_inland_site(384, 288, site)
	_ok(site.x >= 0 and site2.x >= 0, "found two dry cells 3-4 cells inland, 20+ apart (%s, %s)" % [str(site), str(site2)])
	if site.x < 0 or site2.x < 0:
		return
	_app.select_domain("civilization")
	await _settle(8)
	_app.arm_tool("settlement")
	await _settle(8)
	var cb := _snap_toggle()
	_ok(cb != null, "the Snap to water toggle is on the options bar")
	if cb == null:
		return
	_ok(not cb.button_pressed, "...and defaults off")
	var n0: int = _b.settlements().size()
	await _click_site(site)
	var st: Array = _b.settlements()
	_ok(st.size() == n0 + 1, "off: a real map click dropped one settlement")
	var off_cell := Vector2i(int(round(float(st[st.size() - 1]["x"]))), int(round(float(st[st.size() - 1]["y"]))))
	_ok(absi(off_cell.x - site.x) <= 1 and absi(off_cell.y - site.y) <= 1 and not _touches_sea(off_cell.x, off_cell.y), "off: it stays where clicked %s -> %s, not on the shore" % [str(site), str(off_cell)])
	cb = _snap_toggle()  ## the drop rebuilt the options bar
	await _click_at(cb.get_global_rect().get_center())
	cb = _snap_toggle()
	_ok(cb != null and cb.button_pressed, "a real click ticks Snap to water")
	await _click_site(site2)
	st = _b.settlements()
	_ok(st.size() == n0 + 2 or st.size() == n0 + 1, "on: the click placed or selected a settlement")
	var on_cell := Vector2i(int(round(float(st[st.size() - 1]["x"]))), int(round(float(st[st.size() - 1]["y"]))))
	print("  info on-cell ", on_cell, " from ", site2)
	for dy in range(-1, 2):
		var row := ""
		for dx in range(-1, 2):
			var q: Dictionary = _b.sample_cell(on_cell.x + dx, on_cell.y + dy)
			row += "%.3f/%s/%s " % [float(q.get("elevation", -1)), str(q.get("river_order", "-")), str(q.get("drainage", "-"))]
		print("  info nb ", row)
	print("  info sea ", _sea)
	_ok(st.size() == n0 + 2 and _touches_sea(on_cell.x, on_cell.y, true) and not _is_sea(on_cell.x, on_cell.y), "on: the settlement sits on a dry cell touching the sea")
	await _shot("snap_on")

# -- F28 --------------------------------------------------------------------------------------------

func _part_f28() -> void:
	print("== F28: Plates and Drift under a World-Structure archetype ==")
	_app.select_domain("world")
	await _settle(8)
	_ok(bool(_b.param_get("ws_tect_editable")), "the shipped app has ws_tect_editable on")
	_b.world_gen.apply_archetype("archipelago")
	_b.param_set("world_structure.enabled", true)
	_ok(int(_b.param_get("tect.plates")) == 26 and absf(float(_b.param_get("tect.vel")) - 1.6) < 1e-9, "the archipelago archetype derives plates 26 and drift 1.6 (%s, %s)" % [str(_b.param_get("tect.plates")), str(_b.param_get("tect.vel"))])
	await _gen(133742, 700.0, 384, 288)
	var ws: Control = null
	for w in _app._workspaces:
		if w.name == "WorldWorkspace":
			ws = w
	var sl: HSlider = ws._ws_override_sliders.get("tect.plates", null)
	_ok(sl != null and sl.editable, "the Plates slider is editable under World Structure")
	_ok(sl != null and int(sl.value) == 26, "...and shows the derived 26 (%s)" % [str(sl.value if sl != null else "none")])
	if sl == null:
		return
	var why: String = ws.focus_param("tect.plates")  ## the settings search's own door: opens the category and unfolds the row
	_ok(why == "", "the Plates row can be revealed (%s)" % why)
	await _settle(12)
	if sl.is_visible_in_tree():
		var r := sl.get_global_rect()
		await _click_at(Vector2(r.position.x + r.size.x * 0.2, r.get_center().y))
		var v := int(_b.param_get("tect.plates"))
		_ok(v != 26 and v >= 4 and v <= 12, "a real click at 20% of the Plates slider set a hand value (%d)" % v)
		await _shot("plates_hand")
		await _gen(133742, 700.0, 384, 288)
		_ok(int(_b.param_get("tect.plates")) == v, "the hand value survives a Generate (%d)" % int(_b.param_get("tect.plates")))
	else:
		var chain := ""
		var n: Node = sl
		while n != null and n != _app:
			chain += "%s(%s) " % [n.name, str((n as CanvasItem).visible) if n is CanvasItem else "-"]
			n = n.get_parent()
		print("  info chain: ", chain)
		_ok(false, "the Plates slider was not visible to click")
