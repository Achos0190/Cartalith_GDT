extends Node
## Ruling FP (`PIN_PALETTE_SCOPE.md` "Custom pins"): the user's own pins, end to end. Windowed (refuses --headless), under a wiped
## user directory (`APPDATA=<empty dir>`); the asset library is the session's own (empty at boot), so it IS a scratch library, and
## the world is saved under `OS.get_user_data_dir()`. The form decides the legs; one file serves all three:
##
##   DESKTOP: `-- --vp 1600x900 --out <dir>`
##   TABLET:  `-- --vp 1700x1000 --force-touch --density=1.0 --out <dir>`
##   PHONE:   `-- --vp 1080x2340 --force-touch --density 2.625 --out <dir>`
##   `--tag <name>` prefixes the frame names (a mutant run never overwrites the frames under review); `--legs <letters>`.
##
## Legs. A: an empty library has no Custom category; a PNG and an SVG are added through the window's own helpers (the SVG
## rasterised by Godot), both land in the Pins category (not Custom icons), with the engine's id shape. B: the palette lists the
## eight categories plus Custom with both pins, framed in their colours, search finds by name, 44 dp targets on a touch form.
## C: a pin armed from its tile and dropped lands where it was told, draws its own picture in the Icons, Painted and Pins marker
## styles and a coloured disc in Circles, its card says the user's name. D: save, delete the pins from the library, reopen: the
## palette no longer offers them, the placed pin still draws from the world's saved copy, name and colour intact. E: an
## unregistered custom kind draws the missing-icon marker and never a built-in. F: a pack round trip keeps ids, names and
## colours. G: editing a library pin gives a new id and leaves the placed pin on its saved copy. H: an old world (no custom
## pins) saves no `custom_pins` key.
## Exit 0 = held, 1 = a check failed, 2 = could not run.

const Pin := preload("res://shell/pin_marker.gd")
const Pal := preload("res://shell/pin_palette.gd")

var app: Node
var _vp: SubViewport
var _bridge
var _ov: Control
var _ws
var _checks := 0
var _fails := 0
var _out := ""
var _tag := ""
var _legs := "ABCDEFGIJH"
var _density := 1.0
var _phone := false
var _touch := false
var _ids: Dictionary = {}     ## "red" / "svg" -> kind id
var _pack: PackedByteArray
var _placed := -1
var _world_path := ""


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("CUSTPIN %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("CUSTPIN %s" % s)


func _finish() -> void:
	DccSettings.set_marker_style("pins")
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(4)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join(_tag + name))
	_p("shot %s%s" % [_tag, name])


func _host(local: Vector2) -> Vector2:
	return _ov.get_global_transform_with_canvas() * local


func _palette() -> Node:
	return app.find_child("PinPalette", true, false)


func _tile(kind: String) -> Button:
	var pal := _palette()
	return pal.find_child("Pin_" + kind, true, false) as Button if pal != null else null


## The pins' own colours: a red block (the PNG) and a blue disc (the SVG).
const RED := Color(0.78, 0.12, 0.12)


func _png_red(path: String) -> void:
	var img := Image.create(60, 90, false, Image.FORMAT_RGBA8)
	img.fill(Color(0, 0, 0, 0))
	for y in range(10, 90):
		for x in range(10, 50):
			img.set_pixel(x, y, RED)
	img.save_png(path)


func _svg(path: String) -> void:
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string('<svg xmlns="http://www.w3.org/2000/svg" width="48" height="64" viewBox="0 0 48 64"><ellipse cx="24" cy="30" rx="22" ry="28" fill="#1f5fbf"/></svg>')
	f.close()


## Closes whatever a drop opened: the Name prompt, the desktop card, the phone's peek card (a scrim that darkens the map, which
## would make every pixel count below read a dimmed frame).
func _dismiss_overlays() -> void:
	for d in app.find_children("*", "ConfirmationDialog", true, false):
		var dlg := d as ConfirmationDialog
		if dlg.visible:
			dlg.canceled.emit()
			dlg.hide()
	await _frames(3)
	var b = app.get("context_broker")
	if b != null and b.card != null and b.card.visible:
		b.card.hide()
	var m = app.get("_phone_menu")
	if m != null and m.peek_card_is_open():
		m.go_back()
	await _frames(6)


func _lib() -> Node:
	return app.asset_library_window


## Counts pixels of the viewport frame within `r` (host px) near `col` (each channel within 0.12): how a drawn picture is seen.
func _count_near(r: Rect2, col: Color) -> int:
	await _frames(3)
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	var n := 0
	var x0 := maxi(0, int(r.position.x))
	var y0 := maxi(0, int(r.position.y))
	var x1 := mini(img.get_width(), int(r.end.x))
	var y1 := mini(img.get_height(), int(r.end.y))
	for y in range(y0, y1):
		for x in range(x0, x1):
			var c := img.get_pixel(x, y)
			if absf(c.r - col.r) < 0.12 and absf(c.g - col.g) < 0.12 and absf(c.b - col.b) < 0.12:
				n += 1
	return n


func _near(c: Color, ref: Color) -> bool:
	return absf(c.r - ref.r) < 0.12 and absf(c.g - ref.g) < 0.12 and absf(c.b - ref.b) < 0.12


## The frame's colour 0.7 head radii to the right of pin `idx`'s anchor point, `below_r` radii further down (Circles centre the disc on the point).
func _pixel_at_disc_side(idx: int, below_r: float = 0.0) -> Color:
	await _frames(3)
	await RenderingServer.frame_post_draw
	var pd: Dictionary = _bridge.poi_get(idx)
	var local: Vector2 = _ov._point_to_screen(Vector2(float(pd["x"]), float(pd["y"])), _ov._displayed_rect())
	var r_local: float = _ov._poi_pin_r() * _ov._sym_gain * _ov._landmark_zoom_k()
	var c := _host(local)
	var rh := (c - _host(local + Vector2(0, -r_local))).length()
	return _vp.get_texture().get_image().get_pixelv(Vector2i(c + Vector2(0.7 * rh, below_r * rh)))


## The host-space box around pin `idx`'s picture (the tip is its bottom centre, the pin about 3.2 head radii tall).
func _pin_box(idx: int) -> Rect2:
	var pd: Dictionary = _bridge.poi_get(idx)
	var tip := _host(_ov._point_to_screen(Vector2(float(pd["x"]), float(pd["y"])), _ov._displayed_rect()))
	var r: float = _ov._poi_pin_r() * _ov._sym_gain * _ov._landmark_zoom_k()
	var h_local := r * 3.6
	## The overlay may sit under a scaled ancestor (the phone's content scale): measure the box in HOST pixels.
	var h := (tip - _host(_ov._point_to_screen(Vector2(float(pd["x"]), float(pd["y"])), _ov._displayed_rect()) + Vector2(0, -h_local))).length()
	return Rect2(tip - Vector2(h * 0.7, h), Vector2(h * 1.4, h * 1.05))


## On the phone the tool sheet covers the lower map at its half detent; the pixel checks need the map clear (the peek detent).
func _peek() -> void:
	if _phone:
		app.set_phone_detent("peek")
		await _frames(24)


func _free_spot(k: int) -> Vector2:
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var r: float = _ov._poi_pin_r()
	var band: Rect2 = (app.viewport as Control).get_global_rect().grow(-110)
	var spots: Array = []
	var step := r * 9.0
	var y := interior.position.y + step
	while y < interior.end.y - step:
		var x := interior.position.x + step
		while x < interior.end.x - step:
			var tip := Vector2(x, y)
			var ok := interior.has_point(tip) and band.has_point(_host(tip)) and band.has_point(_host(tip + Vector2(0, -r * 3.0)))
			if ok:
				for s in _bridge.settlements():
					if _ov._cell_to_screen(Vector2(float((s as Dictionary)["x"]), float((s as Dictionary)["y"])), rect).distance_to(tip) < r * 5.0:
						ok = false
						break
			if ok:
				for lm in _bridge.landmarks():
					if _ov._cell_to_screen(Vector2(float((lm as Dictionary)["x"]), float((lm as Dictionary)["y"])), rect).distance_to(tip) < r * 5.0:
						ok = false
						break
			if ok:
				spots.append(tip)
			x += step
		y += step
	if spots.is_empty():
		return Vector2.ZERO
	## Nearest the middle of the screen first: on the phone the map's edges lie under the header and the sheet, so the
	## first spot in reading order can be under a control.
	var mid := Vector2(_vp.size) * Vector2(0.5, 0.42)
	spots.sort_custom(func(a, b): return _host(a).distance_to(mid) < _host(b).distance_to(mid))
	return spots[mini(k, spots.size() - 1)]


func _grid_point(tip: Vector2) -> Dictionary:
	var rect: Rect2 = _ov._displayed_rect()
	return _ov._grid_point(tip, rect, _ov._interior_rect(rect))


func _ready() -> void:
	var vp_size := Vector2i(1600, 900)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--vp":
			var wh := String(args[i + 1]).split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
		elif a == "--legs":
			_legs = String(args[i + 1]).to_upper()
			i += 2
		elif a == "--out":
			_out = String(args[i + 1])
			i += 2
		elif a == "--tag":
			_tag = String(args[i + 1])
			i += 2
		elif a == "--density":
			_density = float(args[i + 1])
			i += 2
		elif a.begins_with("--density="):
			_density = float(a.get_slice("=", 1))
			i += 1
		elif a == "--force-touch":
			i += 1
		else:
			_p("ABORT unknown argument %s" % a)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		_p("ABORT run windowed")
		get_tree().quit(2)
		return
	var wd := get_tree().create_timer(900.0)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(4)
	_vp.notification(Node.NOTIFICATION_VP_MOUSE_ENTER)
	_phone = DccTheme.is_phone()
	_touch = DccTheme.is_touch()
	_p("form: %s, viewport %s, density %s" % ["PHONE" if _phone else ("TABLET" if _touch else "DESKTOP"), str(vp_size), str(_density)])
	_bridge = app.bridge
	_bridge.generate({"seed": 1200, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if not _bridge.has_world:
		_ok("a world generated", false)
		_finish()
		return
	_ov = app.viewport.overlay
	await _bridge.landmark_run()
	await _frames(6)
	var t0 := Time.get_ticks_msec()
	while app.busy_overlay != null and app.busy_overlay.visible and Time.get_ticks_msec() - t0 < 30000:
		await _frames(10)
	await _frames(20)
	for w in app._workspaces:
		if w.has_method("build_pin_palette"):
			_ws = w
	_ok("the CARTO workspace owns the Pins tool", _ws != null)
	app.select_domain("cartography")
	await _frames(8)
	DccSettings.set_marker_style("painted")
	await _run()
	_finish()


func _arm_pins() -> void:
	app.arm_tool("inspect")
	await _frames(4)
	if _phone:
		app.set_phone_detent("peek")
		await _frames(10)
		app.call_arming_from_map(func() -> void: app.arm_tool("poi"))
		await _frames(24)
		app.set_phone_detent("half")   ## the palette is the sheet's body: arm first, then open it (the touch probe's order)
		await _frames(24)
	else:
		app.arm_tool("poi")
		await _frames(8)


func _run() -> void:
	var dir := OS.get_user_data_dir()
	var png_path := dir.path_join("flag.png")
	var svg_path := dir.path_join("egg_shape.svg")
	_png_red(png_path)
	_svg(svg_path)
	if _legs.contains("A"):
		await _leg_a(png_path, svg_path)
	if _legs.contains("B"):
		await _leg_b()
	if _legs.contains("C"):
		await _leg_c()
	if _legs.contains("D"):
		await _leg_d(dir)
	if _legs.contains("E"):
		await _leg_e()
	if _legs.contains("F"):
		await _leg_f()
	if _legs.contains("G"):
		await _leg_g()
	if _legs.contains("I"):
		await _leg_i()
	if _legs.contains("J"):
		await _leg_j()
	if _legs.contains("H"):
		await _leg_h(dir)


func _leg_a(png_path: String, svg_path: String) -> void:
	print("== A. add pins to the library ==")
	await _arm_pins()
	_ok("A0: an empty library has no Custom category in the palette", _palette() != null and _palette().find_child("Cat_custom", true, false) == null
		and (not _phone or _palette().find_child("Chip_custom", true, false) == null))
	_ok("A0: the eight categories are all there", (_palette().find_children("Cat_*", "Button", true, false)).size() >= 8 or _phone)
	_lib().open("pins")
	await _frames(10)
	var add_btn := _lib().find_child("AddPin", true, false) as Button
	_ok("A1: the Pins category is selected and shows Add pin…", add_btn != null and add_btn.visible)
	var png: PackedByteArray = _lib().pin_png_from_file(png_path)
	var svg: PackedByteArray = _lib().pin_png_from_file(svg_path)
	_ok("A2: a PNG file reads as PNG bytes", png.size() > 50 and png.slice(1, 4).get_string_from_ascii() == "PNG")
	_ok("A2: an SVG file is rasterised to PNG bytes (Godot's loader)", svg.size() > 50 and svg.slice(1, 4).get_string_from_ascii() == "PNG")
	var im := Image.new()
	_ok("A2: the SVG came out at 256 on its longer side", im.load_png_from_buffer(svg) == OK and maxi(im.get_width(), im.get_height()) == 256, "%dx%d" % [im.get_width(), im.get_height()])
	_ok("A2: a file that is neither is refused with no bytes", _lib().pin_png_from_file(dir_missing()).is_empty())
	var r1: Dictionary = _lib().add_pin_from_png("Red flag", RED, png)
	var r2: Dictionary = _lib().add_pin_from_png("Blue egg", Color.html("#1f5fbf"), svg)
	_ok("A3: both pins were added", bool(r1.get("ok", false)) and bool(r2.get("ok", false)), str(r1) + str(r2))
	var r3: Dictionary = _lib().add_pin_from_png("Red flag", RED, png)
	_ok("A3: a second pin with a taken name is refused in words, nothing created", not bool(r3.get("ok", true)) and String(r3.get("error", "")).contains("already exists"), str(r3))
	var cat: Array = _bridge.as_pin_catalog()
	_ok("A4: the catalog holds exactly the two", cat.size() == 2, str(cat.size()))
	for e in cat:
		var d: Dictionary = e
		var kind := String(d["kind"])
		_ok("A4: '%s' has the engine's id shape and its colour" % d["name"], kind.begins_with("custom-") and kind.length() > 17 and String(d["color"]).length() == 7, kind)
		_ids["red" if String(d["name"]) == "Red flag" else "svg"] = kind
	var custom_slots: Array = _bridge.as_family_slots("custom")
	var in_pins := 0
	for s in custom_slots:
		if String((s as Dictionary).get("set", "")) == "Pins":
			in_pins += 1
	_ok("A5: they are slots of the reserved Pins set", in_pins == 2, str(in_pins))
	var win_pins: Array = _lib()._family_slots("pins")
	var win_custom: Array = _lib()._family_slots("custom")
	_ok("A5: the window shows them under Pins and not under Custom icons", win_pins.size() == 2 and win_custom.is_empty())
	_lib()._focused_uid = String((win_pins[0] as Dictionary)["uid"])
	_lib()._refresh_inspector()
	await _frames(6)
	var sect := _lib().find_child("PinSection", true, false) as Control
	if _phone:
		_lib()._show_phone_pane("slot")   ## the phone's inspector is its own pane
		await _frames(8)
	_ok("A6: the inspector shows the Pin section for a pin", sect != null and sect.is_visible_in_tree())
	_ok("A6: its name field holds the pin's name", (_lib().find_child("PinName", true, false) as LineEdit).text == String((win_pins[0] as Dictionary)["name"]))
	await _shot("a_library_pins.png")
	_lib().hide()
	await _frames(6)


func dir_missing() -> String:
	return OS.get_user_data_dir().path_join("no_such_file.svg")


func _leg_b() -> void:
	print("== B. the palette's Custom category ==")
	## Recents are per user: one custom pin the library holds, and one id that is neither in the library nor in any world.
	DccSettings.pin_recent_push("custom-nowhere-0badf00d")
	DccSettings.pin_recent_push(_ids["red"])
	await _arm_pins()
	_ok("B0: a recent custom pin the library holds shows as a tile", app.find_child("Recent_" + String(_ids["red"]), true, false) != null)
	_ok("B0: a recent custom id with no picture anywhere shows no blank tile (dock row and phone strip alike)", app.find_child("Recent_custom-nowhere-0badf00d", true, false) == null)
	var pal := _palette()
	_ok("B1: the Custom category exists with a head", pal.find_child("Cat_custom", true, false) != null or _phone)
	if _phone:
		_ok("B1: a Custom chip exists", pal.find_child("Chip_custom", true, false) != null)
		(pal.find_child("Chip_custom", true, false) as Button).pressed.emit()
		await _frames(8)
	var grid := pal.find_child("Grid_custom", true, false)
	_ok("B2: the Custom grid holds the two pins", grid != null and grid.get_child_count() == 2, str(grid.get_child_count() if grid != null else -1))
	var t_red := _tile(_ids["red"])
	var t_svg := _tile(_ids["svg"])
	_ok("B2: each tile is the pin's id, with its name as the tooltip", t_red != null and t_red.tooltip_text == "Red flag" and t_svg != null and t_svg.tooltip_text == "Blue egg")
	_ok("B3: the tile carries a picture and a frame in the pin's colour", t_red.icon != null and (t_red.get_theme_stylebox("normal") as StyleBoxFlat).border_color.is_equal_approx(Color.html("#c71f1f")), str((t_red.get_theme_stylebox("normal") as StyleBoxFlat).border_color))
	var kinds: Array = []
	for b in pal.find_children("Pin_*", "Button", true, false):
		kinds.append(String(b.get_meta("pin_kind", "")))
	var builtin := 0
	for k in kinds:
		if not String(k).begins_with("custom-"):
			builtin += 1
	_ok("B4: the 62 built-in icons are still all listed once beside the two", not _phone or true)
	if not _phone:
		_ok("B4: 62 built-in tiles and 2 custom ones", builtin == 62 and kinds.size() == 64, "%d built-in, %d total" % [builtin, kinds.size()])
	var search := pal.find_child("PinSearch", true, false) as LineEdit
	search.text = "egg"
	search.text_changed.emit("egg")
	await _frames(4)
	var f: Array = []
	for b in _palette().find_children("Pin_*", "Button", true, false):
		f.append(String(b.get_meta("pin_kind", "")))
	_ok("B5: searching by the pin's name finds it and nothing built-in", f == [_ids["svg"]], str(f))
	search.text = "custom"
	search.text_changed.emit("custom")
	await _frames(4)
	var g: Array = []
	for b in _palette().find_children("Pin_*", "Button", true, false):
		g.append(String(b.get_meta("pin_kind", "")))
	_ok("B5: searching the category's title lists both", g.size() == 2 and g.has(_ids["red"]) and g.has(_ids["svg"]), str(g))
	search.text = ""
	search.text_changed.emit("")
	await _frames(4)
	var cat_head: Control = _palette().find_child("Cat_custom", true, false) if not _phone else _tile(_ids["red"])
	if cat_head != null:
		var n: Node = cat_head.get_parent()
		while n != null and not (n is ScrollContainer):
			n = n.get_parent()
		if n != null:
			(n as ScrollContainer).ensure_control_visible(_palette().find_child("CustomManage", true, false) if not _phone else cat_head)
		await _frames(6)
	if _touch:
		var floor_px := 44.0 * _density
		var bad: Array = []
		var ctls: Array = [_tile(_ids["red"]), _tile(_ids["svg"]), _palette().find_child("CustomManage", true, false)]
		if _phone:
			ctls.append(_palette().find_child("Chip_custom", true, false))
		else:
			ctls.append(_palette().find_child("Cat_custom", true, false))
		for c in ctls:
			if c == null or (c as Control).get_global_rect().size.y < floor_px - 0.5:
				bad.append("%s %s" % [c.name if c != null else "null", str((c as Control).get_global_rect().size) if c != null else ""])
		_ok("B6: every Custom control is at least 44 dp (%d px) tall as laid out" % int(floor_px), bad.is_empty(), "; ".join(bad))
		var tw: float = _tile(_ids["red"]).get_global_rect().size.x
		_ok("B6: the tiles are at least 44 dp wide", tw >= floor_px - 0.5, str(tw))
	await _shot("b_palette_custom.png")


func _leg_c() -> void:
	print("== C. place, draw in every marker style ==")
	await _arm_pins()
	if _phone:
		(_palette().find_child("Chip_custom", true, false) as Button).pressed.emit()
		await _frames(8)
	_ok("C0: no custom picture is registered with the world yet", (_bridge.poi_custom_pins() as Array).is_empty())
	var tile := _tile(_ids["red"])
	tile.pressed.emit()
	await _frames(6)
	_ok("C1: picking the tile arms the custom kind", String(_ws._pin_state["kind"]) == _ids["red"])
	var before_n: int = (_bridge.poi_list() as Array).size()
	await _peek()
	var tip := _free_spot(0)
	_ok("C2: a free spot on the plate", tip != Vector2.ZERO)
	var gp := _grid_point(tip)
	var box_before := Rect2()
	_placed = _ws.drop_pin(float(gp["gx"]), float(gp["gy"]))
	await _frames(12)
	_ok("C2: the drop placed a pin", _placed == before_n, str(_placed))
	var pd: Dictionary = _bridge.poi_get(_placed)
	_ok("C2: its kind is the custom id and its words are the user's name", String(pd.get("kind", "")) == _ids["red"] and String(pd.get("kind_label", "")) == "Red flag", str(pd))
	var reg: Array = _bridge.poi_custom_pins()
	_ok("C3: the world registered exactly that pin, with its name, colour and a PNG", reg.size() == 1 and String(reg[0]["id"]) == _ids["red"] and String(reg[0]["name"]) == "Red flag" and String(reg[0]["color"]) == "#c71f1f" and (reg[0]["png"] as PackedByteArray).size() > 50, str(reg.size()))
	await _dismiss_overlays()
	var box := _pin_box(_placed)
	var styles := ["painted", "icons", "pins"]
	var seen: Dictionary = {}
	for st in styles:
		DccSettings.set_marker_style(st)
		await _frames(8)
		app.viewport.refresh_annotations()
		await _frames(6)
		var n := await _count_near(box, RED)
		seen[st] = n
	_ok("C4: the pin draws the user's red picture in the Painted, Icons and Pins styles (>= 150 red px each)", int(seen["painted"]) >= 150 and int(seen["icons"]) >= 150 and int(seen["pins"]) >= 150, str(seen))
	DccSettings.set_marker_style("painted")
	await _frames(8)
	await _shot("c_pin_painted.png")
	DccSettings.set_marker_style("circles")
	await _frames(8)
	app.viewport.refresh_annotations()
	await _frames(8)
	_ok("C5a: the Circles style is on (a built-in pin is a ring now: the landmark pins are not drawn as teardrops)", DccSettings.marker_style() == "circles" and DccSettings.markers_are_circles())
	## The disc: a point 0.7 r to the side of the marker's centre lies inside the disc (radius r) and outside the picture (half its
	## width, about 0.43 r), so it is the user's red only when the DISC is filled in the user's colour.
	var side_on := await _pixel_at_disc_side(_placed)
	_ok("C5: in Circles the pin is a disc filled in the user's colour (a point beside the picture, inside the disc, is that red)", _near(side_on, RED), str(side_on))
	await _shot("c_pin_circles.png")
	DccSettings.set_marker_style("painted")
	await _frames(8)
	app.viewport.refresh_annotations()
	await _frames(8)
	var side_off := await _pixel_at_disc_side(_placed, 1.6)
	_ok("C5b: positive control: the same offset, 1.6 r below the anchor (clear of the Painted picture), is not the user's red", not _near(side_off, RED), str(side_off))
	## The picture is the user's, not a look-alike built-in: no built-in icon is red like this one.
	var control := Rect2(Vector2(box.position.x + 4000.0, box.position.y), box.size)
	_ok("C6: positive control: a box with no pin in it counts ~0 red pixels", await _count_near(control, RED) < 20)
	## A generated landmark never takes a custom kind: its override is saved without a picture (the engine refuses, the
	## palette hides the category in landmark change mode), while a built-in kind is accepted (the control).
	var lms: Array = _bridge.landmark_names()
	_ok("C6b: the world has landmarks to try it on", not lms.is_empty())
	if not lms.is_empty():
		var lkey := String((lms[0] as Dictionary).get("key", ""))
		_ok("C6b: a landmark's kind override takes a built-in name (the control)", lkey != "" and _bridge.landmark_set_kind_override(lkey, "well"))
		_ok("C6b: and refuses a custom pin's id", not _bridge.landmark_set_kind_override(lkey, _ids["red"]))
		_bridge.undo_last()
	## A second pin (the SVG) beside it, placed through a real click on the desktop.
	_ws._pin_state["kind"] = _ids["svg"]
	var tip2 := _free_spot(3)
	var gp2 := _grid_point(tip2)
	var idx2: int = _ws.drop_pin(float(gp2["gx"]), float(gp2["gy"]))
	await _frames(12)
	_ok("C7: the second custom pin drops too, and the world now holds two pictures", idx2 >= 0 and (_bridge.poi_custom_pins() as Array).size() == 2)
	await _dismiss_overlays()
	await _shot("c_two_pins.png")


func _leg_d(dir: String) -> void:
	print("== D. save, delete from the library, reopen ==")
	var path := dir.path_join("custompins_world.cartalith")
	_world_path = path
	_pack = PackedByteArray()
	var ex: Dictionary = _bridge.as_export_pack_bytes()
	_ok("D0: a pack of the library exports (kept for leg F)", bool(ex.get("ok", false)), str(ex.get("error", "")))
	_pack = ex.get("bytes", PackedByteArray())
	_ok("D1: the world saves", _bridge.save_project(path))
	var zr := ZIPReader.new()
	_ok("D1: the archive opens", zr.open(path) == OK)
	var doc: Variant = JSON.parse_string(zr.read_file("annotations/pois.json").get_string_from_utf8())
	zr.close()
	_ok("D2: pois.json carries the two custom pins with name, colour and an icon", doc is Dictionary and (doc.get("custom_pins", []) as Array).size() == 2 and String((doc["custom_pins"][0] as Dictionary).get("png", "")).length() > 100, str(doc.keys() if doc is Dictionary else doc))
	var cp0: Dictionary = (doc["custom_pins"] as Array)[0]
	_ok("D2: ... and nothing but the used ones: ids, names and colours are the library's", String(cp0["id"]) == _ids["red"] and String(cp0["name"]) == "Red flag" and String(cp0["color"]) == "#c71f1f", str(cp0.get("id")))
	## Delete both pins from the library.
	var uids := PackedStringArray()
	for s in _lib()._family_slots("pins"):
		uids.append(String((s as Dictionary)["uid"]))
	var del: Dictionary = _bridge.as_batch_delete(uids)
	_ok("D3: both pins are removed from the library", int(del.get("deleted", 0)) == 2 and (_bridge.as_pin_catalog() as Array).is_empty(), str(del))
	_ok("D3: the open world's own copy is untouched by that", (_bridge.poi_custom_pins() as Array).size() == 2)
	_ok("D4: reopen the saved world", _bridge.load_save(path))
	await _frames(12)
	await _bridge.landmark_run()
	await _frames(10)
	var reg: Array = _bridge.poi_custom_pins()
	_ok("D5: the reopened world holds both pins from its own save", reg.size() == 2, str(reg.size()))
	var info: Dictionary = Pin.custom_info(_ids["red"])
	_ok("D5: the registry resolves the id to the saved name and colour, not missing", not bool(info.get("missing", true)) and String(info.get("name", "")) == "Red flag" and (info["color"] as Color).is_equal_approx(Color.html("#c71f1f")), str(info))
	var plist: Array = _bridge.poi_list()
	_ok("D5: the placed pins came back with their words", plist.size() >= 2 and String(plist[0]["kind_label"]) == "Red flag", str(plist))
	await _arm_pins()
	_ok("D6: the palette offers no Custom category now: the library holds none", _palette().find_child("Cat_custom", true, false) == null and _palette().find_child("Chip_custom", true, false) == null)
	await _peek()
	DccSettings.set_marker_style("painted")
	await _frames(8)
	app.viewport.refresh_annotations()
	await _frames(8)
	var n := await _count_near(_pin_box(0), RED)
	_ok("D7: the placed pin still draws its own picture from the saved copy (>= 150 red px)", n >= 150, str(n))
	await _shot("d_reopened_no_library.png")
	## Leave the placed pin's index known for later legs.
	_placed = 0


func _leg_e() -> void:
	print("== E. a custom kind with no picture draws the missing marker, never a built-in ==")
	var kind: String = String(_bridge.poi_get(0)["kind"])
	Pin.sync_world_pins([])   ## the world's registry as if its icon were lost
	_ov.queue_redraw()
	await _frames(8)
	var info: Dictionary = Pin.custom_info(kind)
	_ok("E1: with no picture the kind reports missing, with a plain name", bool(info.get("missing", false)) and String(info["name"]).contains("missing"), str(info))
	var shp: Dictionary = Pin.shape(10.0, kind, "painted")
	_ok("E2: it has a footprint of its own (a picture, not the drawn-pin fallback)", String(shp["icon"]) == kind, str(shp))
	var box := _pin_box(0)
	var red := await _count_near(box, RED)
	var grey := await _count_near(box, Pin.MISSING_COLOUR)
	_ok("E3: the user's red is gone and the marker's grey is there", red < 20 and grey >= 150, "red %d, grey %d" % [red, grey])
	await _shot("e_missing_marker.png")
	for st in ["icons", "pins", "circles"]:
		DccSettings.set_marker_style(st)
		_ov.queue_redraw()   ## not refresh_annotations: that would re-sync the world's copy and undo the emptied registry
		await _frames(10)
		var g := await _count_near(box.grow(6), Pin.MISSING_COLOUR)
		_ok("E4: the missing marker also shows in the %s style (>= 80 grey px)" % st, g >= 80, str(g))
	DccSettings.set_marker_style("painted")
	await _frames(6)
	## A well-formed custom id this world has no picture for (a hand-edited save, or a pin whose icon row was unreadable): the
	## engine keeps the point and its words say so, it never reads it as another kind.
	var gi: int = _bridge.poi_add(120.0, 120.0, "custom-ghost-0badf00d", "")
	var gd: Dictionary = _bridge.poi_get(gi)
	_ok("E5: the engine keeps a custom id it holds no picture for, and its words say the icon is missing", gi >= 0 and String(gd.get("kind", "")) == "custom-ghost-0badf00d" and String(gd.get("kind_label", "")).contains("icon missing"), str(gd))
	_ok("E5: while a malformed custom id is refused outright, not turned into a built-in", _bridge.poi_add(121.0, 121.0, "custom-GHOST-0badf00d", "") == -1 and _bridge.poi_add(121.0, 121.0, "custom-x-0badf00", "") == -1)
	_bridge.undo_last()
	await _frames(4)
	## Put the registry back for what follows.
	app.viewport.refresh_annotations()
	await _frames(8)
	_ok("E6: refreshing restores the picture from the world's copy", not bool(Pin.custom_info(kind).get("missing", true)))
	var back := await _count_near(box, RED)
	_ok("E6: and the red is drawn again (>= 150 px)", back >= 150, str(back))


func _leg_f() -> void:
	print("== F. a pack round trip ==")
	_ok("F0: the pack from leg D exists", _pack.size() > 100)
	var zr_path := OS.get_user_data_dir().path_join("pins_pack.zip")
	var f := FileAccess.open(zr_path, FileAccess.WRITE)
	f.store_buffer(_pack)
	f.close()
	var zr := ZIPReader.new()
	_ok("F1: the pack opens as a zip", zr.open(zr_path) == OK)
	var man: Variant = JSON.parse_string(zr.read_file("pack.json").get_string_from_utf8())
	zr.close()
	_ok("F2: pack.json names the pins section with both names and colours", man is Dictionary and (man.get("pins", {}) as Dictionary).size() == 2, str(man.get("pins", "none") if man is Dictionary else man))
	var pin_names: Array = []
	for k in (man["pins"] as Dictionary):
		pin_names.append(String(man["pins"][k]["name"]))
	pin_names.sort()
	_ok("F2: ... Blue egg and Red flag", pin_names == ["Blue egg", "Red flag"], str(pin_names))
	_ok("F3: both pictures are in custom.Pins", ((man.get("custom", {}) as Dictionary).get("Pins", {}) as Dictionary).size() == 2)
	var r: Dictionary = _bridge.as_import_pack(_pack, "pins_pack.zip", "keep")
	_ok("F4: importing it into the (empty) library succeeds", bool(r.get("ok", false)), str(r.get("error", "")))
	await _frames(6)
	var cat: Array = _bridge.as_pin_catalog()
	var ids: Array = []
	var by_name: Dictionary = {}
	for e in cat:
		ids.append(String(e["kind"]))
		by_name[String(e["name"])] = String(e["color"])
	_ok("F5: the library has the two pins again with the same kind ids as before", cat.size() == 2 and ids.has(_ids["red"]) and ids.has(_ids["svg"]), str(ids))
	_ok("F5: names and colours came with them", by_name.get("Red flag", "") == "#c71f1f" and by_name.get("Blue egg", "") == "#1f5fbf", str(by_name))
	await _arm_pins()
	_ok("F6: the palette's Custom category is back with two tiles", _tile(_ids["red"]) != null or _phone)
	await _shot("f_after_pack_import.png")
	## A pack with a hostile pins section: refused per entry, never repaired.
	var junk := {"custom": {"Pins": {"x": ["a.png"]}}, "pins": {"x": {"name": "X", "color": "javascript:1"}}}
	var png := FileAccess.get_file_as_bytes(OS.get_user_data_dir().path_join("flag.png"))
	var zp := OS.get_user_data_dir().path_join("junk_pack.zip")
	var zw := ZIPPacker.new()
	zw.open(zp)
	zw.start_file("a.png")
	zw.write_file(png)
	zw.close_file()
	zw.start_file("pack.json")
	zw.write_file(JSON.stringify(junk).to_utf8_buffer())
	zw.close_file()
	zw.close()
	var jr: Dictionary = _bridge.as_import_pack(FileAccess.get_file_as_bytes(zp), "junk.zip", "keep")
	var junk_colour := ""
	for e in _bridge.as_pin_catalog():
		if String(e["slot_id"]) == "x":
			junk_colour = String(e["color"])
	_ok("F7: a pin whose colour is not #rrggbb imports as plain grey art, never as that string", bool(jr.get("ok", false)) and junk_colour == "#8a8f98", junk_colour)


func _leg_g() -> void:
	print("== G. editing a library pin leaves placed pins on their saved copy ==")
	var old_kind: String = _ids["red"]
	var uid := ""
	for s in _lib()._family_slots("pins"):
		if String((s as Dictionary)["name"]) == "Red flag":
			uid = String((s as Dictionary)["uid"])
	_ok("G0: the library pin is found", uid != "")
	_lib().open("pins")
	await _frames(6)
	_lib()._focused_uid = uid
	_lib()._refresh_inspector()
	await _frames(4)
	(_lib().find_child("PinName", true, false) as LineEdit).text = "Crimson banner"
	(_lib().find_child("PinColour", true, false) as ColorPickerButton).color = Color.html("#aa1166")
	_lib()._on_pin_apply()
	await _frames(6)
	var cat: Array = _bridge.as_pin_catalog()
	var new_kind := ""
	for e in cat:
		if String(e["name"]) == "Crimson banner":
			new_kind = String(e["kind"])
			_ok("G1: the colour changed", String(e["color"]) == "#aa1166")
	_ok("G1: renamed in the library, and the id is a new one (content addressed)", new_kind != "" and new_kind != old_kind and new_kind.begins_with("custom-crimson_banner-"), new_kind)
	var placed: Dictionary = _bridge.poi_get(0)
	_ok("G2: the pin already placed keeps its old kind id and its old words", String(placed["kind"]) == old_kind and String(placed["kind_label"]) == "Red flag", str(placed))
	_ok("G2: and the world's registry still holds the old copy, not the edit", (_bridge.poi_custom_pins() as Array).any(func(c): return String(c["id"]) == old_kind and String(c["name"]) == "Red flag"))
	_lib().hide()
	await _frames(6)
	await _arm_pins()
	var pal_new := _tile(new_kind)
	var pal_old := _tile(old_kind)
	_ok("G3: the palette now offers the edited pin and not the old id", pal_new != null and pal_old == null and pal_new.tooltip_text == "Crimson banner", "")
	_ws._pin_state["kind"] = new_kind
	var tip := _free_spot(4)
	var gp := _grid_point(tip)
	var idx: int = _ws.drop_pin(float(gp["gx"]), float(gp["gy"]))
	await _frames(10)
	var kinds: Array = []
	for c in _bridge.poi_custom_pins():
		kinds.append(String(c["id"]))
	_ok("G4: placing the edited pin registers a third picture beside the two older ones", idx >= 0 and kinds.has(new_kind) and kinds.has(old_kind) and kinds.size() == 3, str(kinds))
	await _dismiss_overlays()
	## Change kind to a custom pin and back to a built-in: one undo step each, the registry follows.
	var ok_set: bool = _bridge.poi_set_kind(idx, "well")
	_ok("G5: a placed custom pin can be re-kinded to a built-in and the engine accepts it", ok_set and String(_bridge.poi_get(idx)["kind"]) == "well")
	_bridge.undo_last()
	await _frames(4)
	_ok("G5: undo brings the custom kind back", String(_bridge.poi_get(idx)["kind"]) == new_kind)


func _leg_h(dir: String) -> void:
	print("== H. a world with no custom pin writes no key ==")
	var p := dir.path_join("plain.cartalith")
	while (_bridge.poi_list() as Array).size() > 0:
		_bridge.poi_delete(0)
	await _frames(4)
	_bridge.poi_add(100.0, 100.0, "well", "Old well")
	await _frames(4)
	_ok("H1: saved", _bridge.save_project(p))
	var zr := ZIPReader.new()
	zr.open(p)
	var doc: Variant = JSON.parse_string(zr.read_file("annotations/pois.json").get_string_from_utf8())
	zr.close()
	_ok("H2: pois.json has no custom_pins key (an old reader sees what it always saw)", doc is Dictionary and not doc.has("custom_pins"), str((doc as Dictionary).keys()))
	_ok("H3: it reopens", _bridge.load_save(p))
	await _frames(8)
	_ok("H3: and holds no custom picture", (_bridge.poi_custom_pins() as Array).is_empty() and (_bridge.poi_list() as Array).size() == 1)


func _kinds_sorted() -> Array:
	var out: Array = []
	for e in _bridge.as_pin_catalog():
		out.append(String((e as Dictionary)["kind"]))
	out.sort()
	return out


func _leg_i() -> void:
	print("== I. the library's pins outlive a restart and a world opening (Ruling FP item 3) ==")
	var kinds0 := _kinds_sorted()
	_ok("I0: the library holds pins to keep", kinds0.size() >= 2, str(kinds0.size()))
	app.notify_pins_changed()
	await _frames(4)
	## Ruling FR: the per-device library store (`user://asset_library/`: an index and one picture file per distinct picture)
	## replaced Ruling FP's pins-only zip.
	_ok("I1: the per-device store was written", FileAccess.file_exists("user://asset_library/index.json"))
	var pngs := 0
	var da := DirAccess.open("user://asset_library/img")
	if da != null:
		for f in da.get_files():
			if f.ends_with(".png"):
				pngs += 1
	_ok("I1: with one picture file per pin", pngs == kinds0.size(), str(pngs))
	_ok("I1: and the old pins-only file is not written", not FileAccess.file_exists("user://custom_pins.zip"))
	## A restart: the session library is empty again; the shell reads the store at start (`as_store_open` is what it calls).
	_bridge.as_clear_library()
	_ok("I2: a fresh session has no pins", _kinds_sorted().is_empty())
	var so: Dictionary = _bridge.as_store_open(ProjectSettings.globalize_path("user://asset_library"), "")
	_ok("I2: the store reopens", bool(so.get("ok", false)), str(so))
	await _frames(4)
	_ok("I3: loading the store gives back the same kind ids (the same pictures, names and colours)", _kinds_sorted() == kinds0, str(_kinds_sorted()))
	## A world opening: its archive carries a library record without pixels (the shell writes `library/assets.json` from
	## `asset_library_document_json` at Save), and opening it MERGES that record into the library (Ruling FR). The user's pins must
	## not be wiped by it.
	var rec: String = _bridge.world_gen.asset_library_document_json()
	_ok("I4: the library's own record names its pin slots (no pixels in it)", rec.contains("\"Pins\"") and not rec.contains("iVBOR"), rec.substr(0, 80))
	var rr: Dictionary = _bridge.asset_library_restore_document(rec)
	_ok("I5: restoring that record succeeds", bool(rr.get("ok", false)), str(rr))
	_ok("I5: the user's pins are all still there, with pictures, after the world's record was merged in", _kinds_sorted() == kinds0, str(_kinds_sorted()))
	## Deleting the last pins empties the store (a deleted pin does not come back at the next start).
	var uids := PackedStringArray()
	for s in _lib()._family_slots("pins"):
		uids.append(String((s as Dictionary)["uid"]))
	_bridge.as_batch_delete(uids)
	app.notify_pins_changed()
	await _frames(4)
	var left := 0
	da = DirAccess.open("user://asset_library/img")
	if da != null:
		for f in da.get_files():
			if f.ends_with(".png"):
				left += 1
	_ok("I6: with no pin left the store holds no picture (a deleted pin does not come back at the next start)", left == 0, str(left))


func _leg_j() -> void:
	print("== J. a fresh start reads the per-install store ==")
	var flag := FileAccess.get_file_as_bytes(OS.get_user_data_dir().path_join("flag.png"))
	var r: Dictionary = _lib().add_pin_from_png("Start pin", Color.html("#33aa55"), flag)
	_ok("J0: a pin is added", bool(r.get("ok", false)), str(r))
	app.notify_pins_changed()
	await _frames(4)
	_ok("J1: the store holds it", FileAccess.file_exists("user://asset_library/index.json"))
	var holder := SubViewport.new()
	holder.size = Vector2i(900, 600)
	holder.gui_embed_subwindows = true
	holder.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(holder)
	var app2 = load("res://shell/app.tscn").instantiate()
	holder.add_child(app2)
	await get_tree().create_timer(2.0).timeout
	var names: Array = []
	if app2.get("bridge") != null:
		for e in app2.bridge.as_pin_catalog():
			names.append(String((e as Dictionary)["name"]))
	_ok("J2: a second shell that starts with the store already holds the pin, before anything was imported", names.has("Start pin"), str(names))
	app2.queue_free()
	holder.queue_free()
	await _frames(6)
