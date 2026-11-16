extends Node
## CQ-5, the tablet form of the quick bar (`CUSTOM_QUICK_BAR_SCOPE.md` section 7, CQ-5's row): the touch scale and
## the touch gestures of the quick cluster, the ring dock and the Customise window, measured in dp at the OnePlus Pad
## 3's class and driven by REAL touch events.
##
##   godot --path . --resolution 1600x1000 _quickbar_tablet_probe.tscn -- --vp 3392x2400 --density 2.225 --force-touch
##   godot --path . --resolution 1600x1000 _quickbar_tablet_probe.tscn -- --vp 2400x3392 --density 2.225 --force-touch  (portrait)
##
## Windowed, never headless; wiped user dir (a scratch APPDATA); under `C:/cw/.win_lock` when the machine is shared.
## Arguments read here: `--vp WxH` (required: the host SubViewport the shell boots in, so the Pad's pixel size is
## reproduced on any monitor), `--shots <dir>`. Accepted because the shell reads them: `--density D`, `--force-touch`.
## Anything else aborts with exit 2. Mutation runs edit the code under test (literal replacements, restored after each
## run), never this file.
##
## ## The finger
##
## A finger is an `InputEventScreenTouch` / `InputEventScreenDrag` AND the emulated mouse event Godot derives from it
## (`device = InputEvent.DEVICE_ID_EMULATION`, the marker `map_overlay.gd` and `quick_bar.gd` gate their touch paths
## on), pushed into the host viewport in its own screen pixels (the host's stretch applied: about 2.2x at the Pad's
## density). Nothing calls `ring_touch_open()` or `open_chip_menu()` directly: the hold, the slide, the lift and the
## long-press all arrive the way a thumb's do.
##
## ## What it protects (every expectation is a literal written here or an independent measurement)
##
## 1. DP. A control's size in dp is its size on screen in physical pixels over the density: the control's own size,
##    times its window's content scale, times the host's stretch, over `--density`. Measured per control, never read
##    from a constant (`DccTheme.role_px`, `DOCK_*`).
## 2. THE CLUSTER. With pins, the Assistant chip, `＋` and (when the bar overflows) `⋯` on the bar, every chip is at
##    least 44 dp on both axes. A short tap on a pin runs it; a LONG press (0.4 s, still) opens the chip's menu and does
##    NOT run the pin when the finger lifts; a long press that wanders past the slop is a drag, not a menu; the
##    Assistant chip's long press opens its own menu. A mouse (a press that is not an emulated one) never gets the
##    long-press.
## 3. THE DOCK. Every dock chip is at least 44 dp on both axes; a thumb hold (0.62 s, still), a slide DOWN through the disc
##    into a dock chip and a lift runs that chip's command; a swipe that never rests (no hold) opens no ring and runs
##    nothing; a lift on a disabled chip runs nothing; a lift in the dead zone leaves the ring open and a TAP on a dock
##    chip then runs it; the dock is not clipped at any of the four screen edges.
## 4. HANDEDNESS. With either `DccSettings.dominant_hand()`, the context card that opens with the ring overlaps no dock
##    chip and no slot, at three places on the map.
## 5. CUSTOMISE. The window opens from a tap on `＋`; on every tab every visible button, check box, field and list row is
##    at least 44 dp on both axes; and a tap on a row then `Add` pins it (the whole window works with a finger).
##
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

const SEED := 483920
## A hold longer than `map_overlay.gd`'s 500 ms tablet hold, with margin for a slow frame.
const HOLD_S := 0.62
## The quick bar's long-press is 0.4 s; wait past it.
const LONG_S := 0.55

var _fails := 0
var _checks := 0
var _shots := ""
var _vp: SubViewport
var _app: Node
var _bridge
var _dens := 1.0
var _scale := 1.0
var _keys: Array = []
var _vault_key := "menu:%d" % DccMenus.ID_VAULT
var _planner_key := "menu:%d" % DccMenus.ID_STORY_JOURNEY_PLANNER
var _quickbar_key := ""
var _win: QuickBarWindow
## `--debug` prints the ring state at every slide step.
var _dbg := false
var _gen_seen := false


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if _bridge != null and _bridge.generating and not _gen_seen:
		_gen_seen = true
		print("  !! a generation is running at this check (nothing in this probe starts one)")
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _frames(n: int = 3) -> void:
	for _i in n:
		await get_tree().process_frame


func _wait(s: float) -> void:
	await get_tree().create_timer(s).timeout


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	if img != null:
		var p := _shots.path_join("cq5_%s.png" % tag)
		img.save_png(p)
		print("SHOT %s" % p)


func _arg(name: String, dflt: String) -> String:
	var a := OS.get_cmdline_user_args()
	for i in a.size():
		if a[i] == name and i + 1 < a.size():
			return a[i + 1]
	return dflt


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			i += 1
		elif a == "--debug":
			_dbg = true
			i += 1
		elif a in ["--vp", "--density", "--shots"]:
			i += 2
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
	var parts := _arg("--vp", "").split("x")
	if parts.size() != 2:
		print("ABORT: needs --vp WxH")
		get_tree().quit(2)
		return
	_dens = float(_arg("--density", "1"))
	_shots = _arg("--shots", "")
	get_tree().create_timer(1500.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	Input.emulate_touch_from_mouse = true
	Engine.max_fps = 60
	await _run(int(parts[0]), int(parts[1]))
	if _checks == 0:
		_fails += 1
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


# -- the finger ---------------------------------------------------------------------------

## Host viewport canvas coordinates -> the pixels an event pushed into the host carries.
func _px(p: Vector2) -> Vector2:
	return _vp.get_final_transform() * p


## A point of a control, in the host's canvas coordinates: an embedded window's content is placed by the window's own
## position and content scale first.
func _host_point(c: Control, local: Vector2) -> Vector2:
	var p := c.get_global_transform_with_canvas() * local
	var w := c.get_window()
	if w != null and w != get_tree().root and w != _vp and w.is_embedded():
		return Vector2(w.position) + w.get_final_transform() * p
	return p


func _centre(c: Control) -> Vector2:
	return _host_point(c, c.size * 0.5)


## A control's size in dp: its size over the window's content scale and the host's stretch, over the density.
func _dp(c: Control) -> Vector2:
	var s := 1.0
	var w := c.get_window()
	if w != null and w != get_tree().root and w != _vp and w.is_embedded():
		s = w.get_final_transform().get_scale().x
	return c.size * c.get_global_transform_with_canvas().get_scale().x * s * _vp.get_final_transform().get_scale().x / _dens


func _dp_units(canvas_units: float) -> float:
	return canvas_units * _vp.get_final_transform().get_scale().x / _dens


func _push(ev: InputEvent) -> void:
	_vp.push_input(ev, false)


func _down(p_canvas: Vector2) -> void:
	var p := _px(p_canvas)
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = true
	t.position = p
	_push(t)
	var m := InputEventMouseButton.new()
	m.device = InputEvent.DEVICE_ID_EMULATION
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_push(m)
	await _frames(2)


func _drag(p_canvas: Vector2, prev_canvas: Vector2) -> void:
	var p := _px(p_canvas)
	var prev := _px(prev_canvas)
	var d := InputEventScreenDrag.new()
	d.index = 0
	d.position = p
	d.relative = p - prev
	_push(d)
	var m := InputEventMouseMotion.new()
	m.device = InputEvent.DEVICE_ID_EMULATION
	m.position = p
	m.global_position = p
	m.relative = p - prev
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_push(m)


func _up(p_canvas: Vector2) -> void:
	var p := _px(p_canvas)
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = false
	t.position = p
	_push(t)
	var m := InputEventMouseButton.new()
	m.device = InputEvent.DEVICE_ID_EMULATION
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = false
	m.position = p
	m.global_position = p
	_push(m)
	await _frames(3)


## A stroke from `a` to `b` in `steps` drag samples, one frame apart. Does not lift.
func _slide(a: Vector2, b: Vector2, steps: int) -> void:
	var prev := a
	for k in range(1, steps + 1):
		var p := a.lerp(b, float(k) / float(steps))
		_drag(p, prev)
		prev = p
		await _frames(2)
		if _dbg:
			var st := _ring_state()
			print("    [slide %d] at %s ring active=%s centre=%s hover='%s' dock_hover=%s ov_last=%s tra=%s focus=%s" % [k, str(p), str(st.get("active")), str(st.get("centre")), str(st.get("hover")), str(st.get("dock_hover")), str(_app.viewport.overlay._last_mouse_local), str(_app.viewport.overlay._touch_ring_active), str(_vp.gui_get_focus_owner())])


func _tap(p: Vector2) -> void:
	await _down(p)
	await _wait(0.08)
	await _up(p)
	await _wait(0.25)


## A mouse press, which is NOT an emulated finger (device 0): what a desktop mouse sends.
func _mouse_press_hold(p_canvas: Vector2, secs: float) -> void:
	var p := _px(p_canvas)
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	_push(m)
	await _wait(secs)
	var u := m.duplicate() as InputEventMouseButton
	u.pressed = false
	u.button_mask = 0
	_push(u)
	await _frames(3)


# -- helpers --------------------------------------------------------------------------------

func _broker():
	return _app.context_broker


func _bar() -> DccQuickBar:
	return _app._quick_bar


func _ring_state() -> Dictionary:
	var r = _broker().ring
	return r.debug_state() if r != null else {}


func _ring_reset() -> void:
	if _broker().ring != null:
		_broker().ring_close()
	if _broker().card != null and _broker().card.visible:
		_broker().card.hide()
	await _frames(3)


func _vault_open() -> bool:
	return _app.vault_window != null and _app.vault_window.visible


func _hide_windows() -> void:
	if _win and _win.visible:
		_win.hide()
	if _app.vault_window:
		_app.vault_window.hide()
	var nodes: Array = _vp.find_children("*", "Window", true, false)
	for n in nodes:
		if n is AcceptDialog and (n as Window).visible and n != _win:
			(n as Window).hide()
	await _frames(3)


func _idx() -> CommandIndex:
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	return idx


func _store(list: String, keys: Array) -> void:
	var idx := _idx()
	var pins: Array = []
	for k in keys:
		var row := idx.find_pin(String(k))
		pins.append({"k": String(k), "t": String(row.get("title", "ghost"))})
	_ok(DccSettings.set_quick_bar_value(list, pins), "stored %d %s pins" % [pins.size(), list])


func _map_centre() -> Vector2:
	return _app.viewport.overlay.get_global_rect().get_center()


## The map pane as the user sees it: the viewport host's rect. The overlay's own rect is wider and taller than the pane
## (the docks and the bars draw over its edges on a tablet), so a hold at "the overlay's edge" lands on a dock or a bar.
func _map_rect() -> Rect2:
	return _app.viewport.get_global_rect()


## The viewport's visible rect in the host's canvas coordinates.
func _view_rect() -> Rect2:
	return Rect2(Vector2.ZERO, Vector2(_vp.size) / _vp.get_final_transform().get_scale().x)


func _enable_assistant() -> void:
	DccSettings.set_llm_value("enabled", true)
	DccSettings.set_llm_value("local_allowed", true)
	DccSettings.set_llm_value("prose", true)
	DccSettings.set_llm_value("preset_id", "ollama")
	DccSettings.set_llm_value("base_url", "http://127.0.0.1:9/v1")
	DccSettings.set_llm_value("model", "mock-model")


# -- the run --------------------------------------------------------------------------------

func _run(vw: int, vh: int) -> void:
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await _wait(1.6)
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	await _frames(6)
	_scale = _vp.get_final_transform().get_scale().x
	print("== %dx%d density %.4f: %s, host stretch %.4f ==" % [vw, vh, _dens, "tablet" if DccTheme.is_tablet() else ("phone" if DccTheme.is_phone() else "desktop"), _scale])
	_ok(DccTheme.is_tablet() and not DccTheme.is_phone(), "the shell booted as a tablet (this probe is the tablet form)")
	if not DccTheme.is_tablet():
		return
	_ok(absf(_scale - _dens) < 0.01, "the host stretch (%.4f) is the density (%.4f): one canvas unit is one dp" % [_scale, _dens])
	_bridge = _app.bridge
	_win = _app.quick_bar_window
	_bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384, "archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await _wait(0.25)
	await _wait(1.0)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(8)
	for r in _idx().all():
		if bool(r["pinnable"]):
			_keys.append(String(r["pin_key"]))
			if String(r["title"]) == "Quick bar":
				_quickbar_key = String(r["pin_key"])
	_ok(_quickbar_key != "" and _keys.has(_vault_key) and _keys.has(_planner_key), "the three named pins are pinnable (vault, planner, quick bar)")
	if _dbg:
		await _experiment()
	print("  map pane %s, overlay %s" % [str(_map_rect()), str(_app.viewport.overlay.get_global_rect())])
	await _cluster()
	await _long_press()
	await _dock_size()
	await _dock_gestures()
	await _handedness()
	await _edges()
	await _customise()
	await _planner_last()


# -- 1 + 2: the cluster -------------------------------------------------------------------

func _visible_chips() -> Array:
	var out: Array = []
	for c in _bar().get_children():
		if c is Button and (c as Button).visible and c.is_visible_in_tree():
			out.append(c)
	return out


func _cluster() -> void:
	print("== 1. the quick cluster: every chip is at least 44 dp ==")
	_enable_assistant()
	_store("bar", [_vault_key, _planner_key, _quickbar_key])
	_bar().refresh()
	await _frames(6)
	var chips := _visible_chips()
	var names: PackedStringArray = []
	var worst := Vector2(1.0e9, 1.0e9)
	var all_ok := true
	for c in chips:
		var d := _dp(c)
		names.append("%s %.0fx%.0f" % [(c as Button).name, d.x, d.y])
		worst = Vector2(minf(worst.x, d.x), minf(worst.y, d.y))
		if d.x < 43.5 or d.y < 43.5:
			all_ok = false
	print("  chips: " + ", ".join(names))
	_ok(chips.size() >= 5, "the cluster shows the Assistant chip, three pins and ＋ (%d chips)" % chips.size())
	_ok(all_ok, "every shown chip is at least 44 x 44 dp (smallest %.1f x %.1f)" % [worst.x, worst.y])
	_ok(_bar().assistant_chip().visible, "the Assistant chip is shown")
	## the overflow chip: 24 pins at this width
	var fill: Array = []
	for k in _keys:
		if fill.size() < 24 and not String(k).begins_with("extra:"):
			fill.append(k)
	_store("bar", fill)
	_bar().refresh()
	await _frames(6)
	var more: Button = _bar()._more
	_ok(more.visible, "with %d pins the bar overflows and the ⋯ chip is shown" % fill.size())
	if more.visible:
		var md := _dp(more)
		_ok(md.x >= 43.5 and md.y >= 43.5, "the ⋯ chip is at least 44 x 44 dp (%.1f x %.1f)" % [md.x, md.y])
	await _shot("01_cluster")
	_store("bar", [_vault_key, _planner_key, _quickbar_key])
	_bar().refresh()
	await _frames(6)


func _long_press() -> void:
	print("== 2. a pin: tap runs it, long-press opens its menu and runs nothing ==")
	await _hide_windows()
	var chip: Button = null
	for c in _visible_chips():
		if String(c.name).begins_with("Pin_") and String(c.get_meta("pin_key", "")) == _vault_key:
			chip = c
	_ok(chip != null, "the vault pin has a chip")
	if chip == null:
		return
	var menu: PopupMenu = _bar()._chip_menu
	## tap
	await _tap(_centre(chip))
	_ok(_vault_open(), "a short tap on the pin runs it (the vault opens)")
	await _hide_windows()
	## long-press, still, then lift
	await _down(_centre(chip))
	await _wait(LONG_S)
	## CQ-6 (2026-11-16): the rest LIFTS the pin; the menu now opens on the lift when the finger never left its place.
	_ok(not menu.visible and _bar()._reorder != null and _bar()._reorder.active, "a finger resting on the pin for 0.55 s lifts it (its menu waits for the lift)")
	_ok(not _vault_open(), "the pin has not run while the finger is down")
	await _up(_centre(chip))
	await _wait(0.3)
	_ok(not _vault_open(), "lifting after the long-press does NOT run the pin")
	_ok(menu.visible, "the menu opens on the lift of a rest that never left its place")
	await _shot("02_chip_menu")
	var rows := PackedStringArray()
	for r in menu.item_count:
		rows.append(menu.get_item_text(r))
	_ok(rows.has("Remove from quick bar") and rows.has("Move left") and rows.has("Move right"), "the menu offers Remove and Move: %s" % str(rows))
	## row pitch of the menu: the font's height plus its v_separation, as the popup draws it
	var fs := menu.get_theme_font_size("font_size")
	var pitch := menu.get_theme_font("font").get_height(fs) + float(menu.get_theme_constant("v_separation"))
	var pitch_dp := _dp_units(pitch) * (menu.get_final_transform().get_scale().x if menu.is_embedded() else 1.0)
	_ok(pitch_dp >= 43.5, "the menu rows are at least 44 dp tall (pitch %.1f dp)" % pitch_dp)
	menu.hide()
	await _frames(3)
	## remove through a finger: long-press, then tap the Remove row
	await _down(_centre(chip))
	await _wait(LONG_S)
	await _up(_centre(chip))
	await _wait(0.3)
	var idx_remove := -1
	for r in menu.item_count:
		if menu.get_item_text(r) == "Remove from quick bar":
			idx_remove = r
	_ok(idx_remove >= 0, "the menu is open again")
	## a PopupMenu row's centre: top margin + index * pitch + half a pitch (separators are taller, but Remove is first)
	var panel_top := float(menu.get_theme_stylebox("panel").get_content_margin(SIDE_TOP))
	var row_local := Vector2(float(menu.size.x) * 0.5, panel_top + (float(idx_remove) + 0.5) * pitch)
	var row_host := Vector2(menu.position) + menu.get_final_transform() * row_local if menu.is_embedded() else Vector2(menu.position) + row_local
	await _tap(row_host)
	var left: Array = []
	for p in DccSettings.quick_bar_value("bar"):
		left.append(String(p["k"]))
	_ok(not left.has(_vault_key) and left.size() == 2, "tapping Remove with a finger removed the pin: %s" % str(left))
	_store("bar", [_vault_key, _planner_key, _quickbar_key])
	_bar().refresh()
	await _frames(6)
	await _hide_windows()
	## a long-press that wanders is a drag: no menu
	for c in _visible_chips():
		if String(c.get_meta("pin_key", "")) == _vault_key:
			chip = c
	var c0 := _centre(chip)
	await _down(c0)
	await _wait(0.15)
	await _slide(c0, c0 + Vector2(40, 0), 3)
	await _wait(LONG_S)
	_ok(not menu.visible, "a finger that wanders 40 dp opens no menu")
	await _up(c0 + Vector2(40, 0))
	await _wait(0.2)
	_ok(not menu.visible, "and the lift opens none")
	await _hide_windows()
	## a mouse (not an emulated finger) that rests on the chip gets no long-press menu
	await _mouse_press_hold(_centre(chip), LONG_S)
	_ok(not menu.visible, "a mouse press held 0.55 s opens no menu (touch only; a right-click still does)")
	await _hide_windows()
	## the Assistant chip's long-press
	var ach: Button = _bar().assistant_chip()
	var amenu: PopupMenu = _bar()._assistant_menu
	_ok(ach.visible, "the Assistant chip is shown")
	if ach.visible:
		await _down(_centre(ach))
		await _wait(LONG_S)
		_ok(amenu.visible, "a long press on the Assistant chip opens its menu")
		var arows := PackedStringArray()
		for r in amenu.item_count:
			arows.append(amenu.get_item_text(r))
		_ok(arows.has("Hide the Assistant chip") and arows.has("Customise…"), "its menu offers Hide and Customise: %s" % str(arows))
		await _up(_centre(ach))
		await _wait(0.3)
		_ok(not (_app.assistant_window != null and _app.assistant_window.visible), "and lifting did not open the assistant panel")
		amenu.hide()
		await _frames(3)
	await _hide_windows()


# -- 3: the dock ----------------------------------------------------------------------------

func _dock_size() -> void:
	print("== 3. the ring dock: every chip is at least 44 dp; the hold opens it ==")
	_store("dock", [_vault_key, _planner_key, _quickbar_key, "menu:999999"])
	await _ring_reset()
	await _hide_windows()
	var mid := _map_centre()
	## With the assistant on, its chip is the dock's first (CQ-7): five chips, all of them touch sized.
	await _down(mid)
	await _wait(HOLD_S)
	var st5 := _ring_state()
	var r5: Array = st5.get("dock_rects", [])
	var worst5 := Vector2(1.0e9, 1.0e9)
	for r in r5:
		worst5 = Vector2(minf(worst5.x, _dp_units((r as Rect2).size.x)), minf(worst5.y, _dp_units((r as Rect2).size.y)))
	_ok(r5.size() == 5 and worst5.x >= 43.5 and worst5.y >= 43.5, "with the Assistant on the dock holds five chips, each at least 44 x 44 dp (%d, smallest %.1f x %.1f)" % [r5.size(), worst5.x, worst5.y])
	await _up(mid)
	await _ring_reset()
	DccSettings.set_llm_value("enabled", false)
	await _frames(4)
	await _down(mid)
	await _wait(HOLD_S)
	var st := _ring_state()
	_ok(bool(st.get("active", false)) and bool(st.get("visible", false)), "a still hold of 0.62 s opens the ring, drawn")
	var rects: Array = st.get("dock_rects", [])
	_ok(rects.size() == 4, "with the Assistant off the ring shows the four dock pins (%d)" % rects.size())
	var worst := Vector2(1.0e9, 1.0e9)
	for r in rects:
		var rr: Rect2 = r
		worst = Vector2(minf(worst.x, _dp_units(rr.size.x)), minf(worst.y, _dp_units(rr.size.y)))
	_ok(worst.x >= 43.5 and worst.y >= 43.5, "every dock chip is at least 44 x 44 dp (smallest %.1f x %.1f)" % [worst.x, worst.y])
	var disc := _dp_units(float(st.get("disc_radius", 0.0))) * 2.0
	print("  dock chip %.1f x %.1f dp; disc %.0f dp across; first chip %.0f dp below the finger" % [worst.x, worst.y, disc, _dp_units((rects[0] as Rect2).position.y - (st["centre"] as Vector2).y)])
	await _shot("03_hold_ring_dock")
	await _up(mid)
	await _ring_reset()


## One thumb gesture: down at `start`, rest `hold`, slide to `target` in `steps`, read the ring, lift. Returns what it saw.
func _gesture(start: Vector2, hold: float, target_of: Callable, steps: int = 10) -> Dictionary:
	await _ring_reset()
	await _hide_windows()
	await _down(start)
	await _wait(hold)
	var st0 := _ring_state()
	var target: Vector2 = target_of.call(st0)
	if hold > 0.0:
		await _slide(start, target, steps)
	var st1 := _ring_state()
	await _up(target)
	await _wait(0.3)
	return {"open": bool(st0.get("active", false)), "drawn": bool(st0.get("visible", false)), "hover": int(st1.get("dock_hover", -1)),
		"caption": String(st1.get("caption", "")), "after": _ring_state(), "target": target, "rects": st0.get("dock_rects", [])}


func _dock_gestures() -> void:
	print("== 3b. hold, slide down into a dock chip, lift ==")
	var mid := _map_centre()
	var chip_i := func(i: int) -> Callable:
		return func(st: Dictionary) -> Vector2:
			var rs: Array = st.get("dock_rects", [])
			return (rs[i] as Rect2).get_center() if i < rs.size() else mid + Vector2(0, 300)
	var g: Dictionary = await _gesture(mid, HOLD_S, chip_i.call(0))
	_ok(bool(g["open"]), "the hold opened the ring")
	_ok(int(g["hover"]) == 0, "sliding down hovers dock chip 0 (hover %d, caption '%s')" % [int(g["hover"]), String(g["caption"])])
	_ok(_vault_open(), "lifting on dock chip 0 ran it: the vault is open")
	await _shot("04_after_dock_run")
	_ok(not bool((g["after"] as Dictionary).get("active", false)), "and the ring closed")
	## (dock chip 1, the journey planner, swaps the main view: it runs last in the probe, `_planner_last`)
	g = await _gesture(mid, HOLD_S, chip_i.call(2))
	_ok(int(g["hover"]) == 2 and _win.visible, "lifting on dock chip 2 ran the Customise window (hover %d, open %s)" % [int(g["hover"]), str(_win.visible)])
	await _hide_windows()
	## a disabled chip
	g = await _gesture(mid, HOLD_S, chip_i.call(3))
	_ok(int(g["hover"]) == 3 and not _vault_open() and not _win.visible, "lifting on the dock chip of a removed command runs nothing (hover %d, caption '%s')" % [int(g["hover"]), String(g["caption"])])
	_ok(not bool((g["after"] as Dictionary).get("active", false)), "and the ring closes")
	await _hide_windows()
	## the same hold-slide-lift on a compass slot: W is Region select, which arms the Region tool
	g = await _gesture(mid, HOLD_S, func(st: Dictionary) -> Vector2:
		return (st["centre"] as Vector2) - Vector2(float(st["radius"]), 0.0))
	_ok(String(_app.armed_tool) == "region", "hold, slide left onto W and lift arms Region select (armed '%s')" % String(_app.armed_tool))
	_app.arm_tool("inspect")
	await _frames(3)
	## no hold: a swipe never opens the ring and never runs a chip
	g = await _gesture(mid, 0.0, chip_i.call(0))
	## (with no hold `_gesture` does not slide; do the swipe itself, fast, with no rest)
	await _ring_reset()
	await _down(mid)
	var swipe_to := mid + Vector2(0, 300)
	await _slide(mid, swipe_to, 4)
	var ring_up := bool(_ring_state().get("active", false))
	await _up(swipe_to)
	await _wait(0.3)
	_ok(not ring_up and not _vault_open() and not _win.visible, "a swipe down with no rest opens no ring and runs no chip")
	await _hide_windows()
	## dead-zone lift leaves it open (sticky); a tap on a chip then runs it
	await _ring_reset()
	await _down(mid)
	await _wait(HOLD_S)
	await _up(mid)
	await _wait(0.3)
	var sticky := _ring_state()
	_ok(bool(sticky.get("active", false)) and bool(sticky.get("sticky", false)), "a lift in the dead zone leaves the ring open and sticky")
	var rs: Array = sticky.get("dock_rects", [])
	if rs.size() > 0:
		await _tap((rs[0] as Rect2).get_center())
		_ok(_vault_open(), "a finger tap on a dock chip of the open ring runs it")
	await _hide_windows()
	await _ring_reset()
	await _store_dock_empty()


## Puts the dock back as the later sections expect it: three real commands and a removed one (never a command that
## does anything on its own, such as a generate: a lift that lands on one would run it).
func _store_dock_empty() -> void:
	_store("dock", [_vault_key, _planner_key, _quickbar_key, "menu:999999"])


# -- 4: handedness --------------------------------------------------------------------------

func _handedness() -> void:
	print("== 4. handedness: the card never lands on the dock ==")
	var was := DccSettings.dominant_hand()
	var r := _map_rect()
	var spots := [r.get_center(), r.position + r.size * Vector2(0.22, 0.5), r.position + r.size * Vector2(0.78, 0.55)]
	for hand in ["right", "left"]:
		DccSettings.set_dominant_hand(hand)
		for spot in spots:
			await _ring_reset()
			await _hide_windows()
			await _down(spot)
			await _wait(HOLD_S)
			var st := _ring_state()
			var card = _broker().card
			var both: bool = card != null and card.visible and bool(st.get("active", false))
			var hit := false
			if both:
				var cr: Rect2 = card.panel_rect()
				for dr in st["dock_rects"]:
					if cr.intersects(dr):
						hit = true
				for k in ["N", "NE", "E", "SE", "S", "SW", "W", "NW"]:
					var deg := {"N": -90.0, "NE": -45.0, "E": 0.0, "SE": 45.0, "S": 90.0, "SW": 135.0, "W": 180.0, "NW": -135.0}
					var a := deg_to_rad(float(deg[k]))
					var c := (st["centre"] as Vector2) + Vector2(cos(a), sin(a)) * float(st["radius"])
					if cr.intersects(Rect2(c - Vector2.ONE * float(st["slot"]) * 0.5, Vector2.ONE * float(st["slot"]))):
						hit = true
			var detail := ""
			if both:
				detail = " (card %s, ring centre %s, dock %s, pane %s)" % [str(card.panel_rect()), str(st["centre"]), str(st["dock_rects"]), str(_map_rect())]
			_ok(both and not hit, "%s hand, hold at (%.0f, %.0f): ring and card open and the card overlaps no dock chip and no slot%s" % [hand, spot.x, spot.y, detail])
			if hand == "left" and spot == spots[0]:
				await _shot("05_left_hand_card")
			await _up(spot)
	DccSettings.set_dominant_hand(was)
	await _ring_reset()


# -- 5: edges -------------------------------------------------------------------------------

func _edges() -> void:
	print("== 5. the dock is not clipped at the screen edges (a thumb hold within 20 dp of each) ==")
	var view := _view_rect()
	var r := _map_rect().intersection(view)
	var pts := {"left": Vector2(r.position.x + 20.0, r.get_center().y), "right": Vector2(r.end.x - 20.0, r.get_center().y),
		"top": Vector2(r.get_center().x, r.position.y + 20.0), "bottom": Vector2(r.get_center().x, r.end.y - 20.0)}
	for name in pts:
		await _ring_reset()
		await _hide_windows()
		await _down(pts[name])
		await _wait(HOLD_S)
		var st := _ring_state()
		var rects: Array = st.get("dock_rects", [])
		var inside := rects.size() == 4
		for dr in rects:
			if not view.encloses(dr):
				inside = false
		var c: Vector2 = st.get("centre", Vector2.ZERO)
		var dr_ := float(st.get("disc_radius", 0.0))
		var disc_in := view.encloses(Rect2(c - Vector2.ONE * dr_, Vector2.ONE * 2.0 * dr_))
		_ok(inside and disc_in, "%s edge: the four dock chips and the disc are inside the screen (hold %s, centre %s, view %s, %d chips)" % [name, str(pts[name]), str(c), str(view), rects.size()])
		if name == "bottom":
			await _shot("06_bottom_edge")
		await _up(pts[name])
	await _ring_reset()


# -- 6: Customise on touch ------------------------------------------------------------------

func _interactive(root: Node, out: Array) -> void:
	for c in root.get_children():
		if c is Window:
			continue
		if c is Control and (c as Control).is_visible_in_tree():
			if c is BaseButton or c is LineEdit or c is OptionButton or c is SpinBox:
				out.append(c)
		_interactive(c, out)


func _customise() -> void:
	print("== 6. Customise on touch: every control is at least 44 dp; the window works with a finger ==")
	await _ring_reset()
	await _hide_windows()
	var plus: Button = _bar()._plus
	await _tap(_centre(plus))
	await _wait(0.4)
	_ok(_win.visible, "a tap on ＋ opens the Customise window (chip centre %s, visible %s, view %s)" % [str(_centre(plus)), str(plus.is_visible_in_tree()), str(_view_rect())])
	await _wait(0.4)
	await _shot("07_customise")
	var tabs: TabBar = _win._tabs
	var ntabs: int = tabs.tab_count
	_ok(ntabs >= 3, "the window has %d tabs" % ntabs)
	var tabs_ok := true
	for t in ntabs:
		var tr := tabs.get_tab_rect(t)
		var d := tr.size * tabs.get_global_transform_with_canvas().get_scale().x * _win.get_final_transform().get_scale().x * _vp.get_final_transform().get_scale().x / _dens
		if d.x < 43.5 or d.y < 43.5:
			tabs_ok = false
			print("  tab %d is %.1f x %.1f dp" % [t, d.x, d.y])
	_ok(tabs_ok, "every tab is at least 44 x 44 dp")
	for t in ntabs:
		tabs.current_tab = t
		tabs.tab_selected.emit(t)
		await _frames(6)
		var ctl: Array = []
		_interactive(_win, ctl)
		var small: Array = []
		for c in ctl:
			var d := _dp(c)
			if d.x < 43.5 or d.y < 43.5:
				small.append("%s %s %.0fx%.0f" % [c.get_class(), String(c.name), d.x, d.y])
		_ok(ctl.size() > 0 and small.is_empty(), "tab %d '%s': %d controls, all at least 44 x 44 dp%s" % [t, tabs.get_tab_title(t), ctl.size(), "" if small.is_empty() else " -- too small: " + "; ".join(small)])
		for lst in _win.find_children("*", "ItemList", true, false):
			var il := lst as ItemList
			if il.is_visible_in_tree() and il.item_count > 0:
				var rh := il.get_item_rect(0, true).size.y * il.get_global_transform_with_canvas().get_scale().x * _win.get_final_transform().get_scale().x * _vp.get_final_transform().get_scale().x / _dens
				_ok(rh >= 43.5, "tab %d list '%s': a row is %.1f dp tall (at least 44)" % [t, String(il.name), rh])
	_win.hide()
	await _frames(3)


## `--debug` only: which of the card's behaviours swallows a finger's motion after the hold.
func _experiment() -> void:
	print("== X. experiment: does a motion reach the map after a hold? ==")
	await _ring_reset()
	await _hide_windows()
	var mid := _map_centre()
	await _down(mid)
	await _wait(HOLD_S)
	var card = _broker().card
	print("  card visible=%s popup_window=%s embedded=%s focused_sub=%s" % [str(card.visible), str(card.popup_window), str(card.is_embedded()), str(card.has_focus())])
	_drag(mid + Vector2(0, 40), mid)
	await _frames(3)
	print("  after one drag with the card up: ov_last=%s ring hover='%s'" % [str(_app.viewport.overlay._last_mouse_local), str(_ring_state().get("hover"))])
	card.popup_window = false
	_drag(mid + Vector2(0, 80), mid + Vector2(0, 40))
	await _frames(3)
	print("  popup_window=false: ov_last=%s ring hover='%s'" % [str(_app.viewport.overlay._last_mouse_local), str(_ring_state().get("hover"))])
	card.popup_window = true
	card.hide()
	await _frames(3)
	_drag(mid + Vector2(0, 120), mid + Vector2(0, 80))
	await _frames(3)
	print("  card hidden: ov_last=%s ring active=%s hover='%s'" % [str(_app.viewport.overlay._last_mouse_local), str(_ring_state().get("active")), str(_ring_state().get("hover"))])
	await _up(mid + Vector2(0, 120))
	await _ring_reset()


## Dock chip 1 is the journey planner, which swaps the whole main view, so it is the probe's last gesture.
func _planner_last() -> void:
	print("== 7. lifting on a dock chip that opens the journey planner ==")
	await _ring_reset()
	await _hide_windows()
	var mid := _map_centre()
	var g: Dictionary = await _gesture(mid, HOLD_S, func(st: Dictionary) -> Vector2:
		var rs: Array = st.get("dock_rects", [])
		return (rs[1] as Rect2).get_center() if rs.size() > 1 else mid + Vector2(0, 300))
	var planner_open: bool = _app.journey_planner_view != null and _app.journey_planner_view.visible
	_ok(int(g["hover"]) == 1 and planner_open, "lifting on dock chip 1 ran the journey planner (hover %d, open %s)" % [int(g["hover"]), str(planner_open)])
