extends Node
## WINDOWED, IN THE REAL SHELL: AP-3 (Ruling CQ Q2) as the app shows it. A
## generated world, a pack loaded through the bridge, one icon of each AP-3
## family (plus a feature icon, which keeps its glyph) placed through the
## Icon tool's own engine calls, and screenshots at fit, x4 and x16 about the
## icons -- for a human to look at, desktop and phone.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _markerart_app_probe.tscn
##   ... _markerart_app_probe.tscn -- --vp 1080x2400 --force-touch   (phone composition)
##
## `--vp WxH` sizes the host SubViewport (default 1600x900); any other argument
## aborts (exit 2). Not the real device (`MISTAKES.md`: `--force-touch` is not
## the phone).
##
## Checks, besides the frames:
##   - the app turned the switch on (`overlay.marker_art_enabled`) and
##     installed a resolver;
##   - with the pack loaded, turning the switch off on the live overlay moves
##     pixels at the icons, and turning it back on restores the frame byte for
##     byte (the switch is what changes the map, nothing else);
##   - the feature icon's neighbourhood does not move with the switch.
## Uses `_markerart_probe.gd`'s pack builder (`user://markerart/pack.zip`).
## Prints `MARKERAPP ok|FAIL <what>`, `MARKERAPP saved <png>` and
## `MARKERAPP checks=<n> failures=<n>`. Exit 0 / 1 / 2.

const SEED := 483920
var app: Node
var _vp: SubViewport
var _fails := 0
var _checks := 0


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print("MARKERAPP %s  %s" % ["ok  " if cond else "FAIL", what])
	if not cond:
		_fails += 1


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


## Long enough for the deep-zoom tile pass to land after a zoom: a frame
## taken while tiles are still arriving differs everywhere (measured: every
## pixel of an 80x80 window moved between two shots 8 frames apart).
func _settle() -> void:
	await _frames(8)
	await get_tree().create_timer(1.5).timeout
	await _frames(4)
	await RenderingServer.frame_post_draw


func _shot(tag: String) -> Image:
	var img := _vp.get_texture().get_image()
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	var path := "user://markerart/app_%s_%s.png" % [form, tag]
	img.save_png(path)
	print("MARKERAPP saved ", ProjectSettings.globalize_path(path))
	return img


## The SubViewport pixel a grid point draws at, through the live overlay's own
## transform (camera included).
func _grid_to_vp(gx: float, gy: float) -> Vector2:
	var ov: Control = app.viewport.overlay
	var local: Vector2 = ov.call("_point_to_screen", Vector2(gx, gy), ov.call("_displayed_rect"))
	return ov.get_global_transform_with_canvas() * local


static func _diff_at(a: Image, b: Image, c: Vector2, half: int) -> int:
	var n := 0
	for y in range(maxi(0, int(c.y) - half), mini(a.get_height(), int(c.y) + half)):
		for x in range(maxi(0, int(c.x) - half), mini(a.get_width(), int(c.x) + half)):
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
	return n


func _ready() -> void:
	var vp_size := Vector2i(1600, 900)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a == "--vp" and i + 1 < args.size():
			var wh := args[i + 1].split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
			continue
		if a == "--force-touch":
			i += 1
			continue
		print("MARKERAPP ABORT: unknown argument '%s'" % a)
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		print("MARKERAPP ABORT: windowed only (it saves screenshots)")
		get_tree().quit(2)
		return
	DirAccess.make_dir_recursive_absolute("user://markerart")
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge = app.bridge
	bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _settle()
	print("MARKERAPP form=%s view=%s palette=%s" % [
		"phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop"),
		_vp.size, "dark" if DccTheme.is_dark() else "light"])

	## No feature-icon art in this pack: the scattered `icons` family would
	## cover the terrain in grey discs and hide what these frames are for.
	var pack_path: String = load("res://_markerart_probe.gd")._build_pack(false)
	_ok(pack_path != "" and bridge.load_asset_pack(pack_path), "the probe pack loads through the bridge")
	await _settle()
	var ov: Control = app.viewport.overlay
	_ok(bool(ov.get("marker_art_enabled")), "the app turned AP-3's switch ON (viewport_host PACK_MARKER_ART)")

	# One icon per family around the map centre. Sea marks snap to a coast or
	# are refused; that is the Icon tool's rule, reported rather than forced.
	var cx := 256.0
	var cy := 192.0
	var placed := {}
	## 12 cells apart, so at x4 (about 7 px a cell here) each icon's 40 px
	## comparison window holds that icon alone.
	for spec in [["settlement", 3, cx - 24.0, cy], ["poi", 0, cx - 12.0, cy], ["feature", 0, cx, cy],
			["seamarks", 2, cx + 12.0, cy]]:
		if bridge.icon_arm(spec[0], spec[1], 1.0, 0.0, 0.0):
			var at: int = bridge.icon_place(spec[2], spec[3])
			if at >= 0:
				placed[spec[0]] = [spec[2], spec[3]]
	if bridge.icon_arm_custom("lighthouse", "Naval", 1.0, 0.0, 0.0):
		if bridge.icon_place(cx + 24.0, cy) >= 0:
			placed["custom"] = [cx + 24.0, cy]
	print("MARKERAPP placed: ", placed.keys())
	for k in ["settlement", "poi", "custom", "feature"]:
		_ok(placed.has(k), "a %s icon was placed through icon_arm/icon_place" % k)
	app.viewport.refresh_annotations()
	await _settle()

	# Find the placed rows as the engine stores them (sea marks may have snapped).
	var rows: Array = bridge.icon_list()
	_shot("fit")
	## `_zoom_at` takes a point in the ViewportHost's own local space (its doc:
	## the `_input` call sites convert), so the SubViewport point converts.
	var host: Control = app.viewport
	var cluster: Vector2 = host.get_global_transform_with_canvas().affine_inverse() * _grid_to_vp(cx, cy)
	host._zoom_at(cluster, 4.0)
	await _settle()
	var on4 := _shot("x4")
	# The switch, live: off moves the icons' pixels, on restores the frame.
	var cb: Callable = ov.get("_marker_art_resolver")
	ov.set_marker_art_resolver(cb, false)
	await _settle()
	var off4 := _shot("x4_switch_off")
	for r in rows:
		var d: Dictionary = r
		var fam := String(d["family"])
		var p := _grid_to_vp(float(d["x"]), float(d["y"]))
		## A narrow (phone) view leaves the outer icons off screen at x4; that
		## is not measured, and says so, rather than reading as "0 px moved".
		if not Rect2(Vector2(20, 20), Vector2(_vp.size) - Vector2(40, 40)).has_point(p):
			print("MARKERAPP note  %s icon is off screen at x4 (%s); not measured" % [fam, p])
			continue
		var moved := _diff_at(on4, off4, p, 20)
		if fam == "feature":
			_ok(moved == 0, "feature icon unchanged by the switch (%d px)" % moved)
		elif ["settlement", "poi", "custom", "seamarks"].has(fam):
			_ok(moved > 0, "%s icon: switch off restores the glyph (%d px moved)" % [fam, moved])
	ov.set_marker_art_resolver(cb, true)
	await _settle()
	var back4 := _vp.get_texture().get_image()
	var drift := 0
	for r in rows:
		var d2: Dictionary = r
		drift += _diff_at(on4, back4, _grid_to_vp(float(d2["x"]), float(d2["y"])), 20)
	_ok(drift == 0, "switch back ON restores every icon's pixels byte for byte (%d px differ)" % drift)
	cluster = host.get_global_transform_with_canvas().affine_inverse() * _grid_to_vp(cx, cy)
	host._zoom_at(cluster, 4.0)
	await _settle()
	_shot("x16")
	print("MARKERAPP checks=%d failures=%d" % [_checks, _fails])
	get_tree().quit(1 if _fails > 0 else 0)
