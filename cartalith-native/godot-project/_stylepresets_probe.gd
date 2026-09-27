extends Node
## Committed probe for **Ruling BI** (`LARGE_ITEM_RULINGS.md`, 2026-09-27):
## the eight researched map style presets (`MAP_STYLE_RESEARCH.md` §4), built
## into `render_workspace.gd`'s `STYLE_PRESETS` beside the six shipped tiles
## and "Village".
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _stylepresets_probe.tscn -- --out DIR [--tag T]
##
## **Run WINDOWED** -- this probe screenshots the viewport, which a headless
## driver never draws (`MISTAKES.md`'s "cargo test cannot see a broken shell"
## row, the GDScript-side version of it).
##
## For every `STYLE_PRESETS` entry, in order:
##   1. applies it the same way a real click does -- `tiles[i]["button"]`'s own
##      `pressed` signal, not a direct `_apply_preset()` call, so this probe
##      exercises the exact path a user's finger does;
##   2. reads the live look/NPR/appearance back from the bridge and asserts
##      each equals what the preset's own bundle says it should be;
##   3. screenshots the viewport and asserts the frame differs from the
##      previous preset's by more than a stated pixel-fraction threshold;
##   4. saves the screenshot.
## Then builds one contact-sheet PNG of every preset in `STYLE_PRESETS` order.

var app: Node
var bridge
var rw: Node
var _fail := 0
var _out := ""
var _tag := "run"

const KNOWN_FLAGS := ["--out", "--tag"]
## Fraction of RGB bytes that must differ by more than 2 levels between two
## consecutive presets' screenshots for them to count as visibly distinct --
## the same threshold shape `render_serial`'s Rust-side `moved()` uses in
## `ruling_bi_style_presets.rs`, chosen loosely (a whole-viewport screenshot
## carries UI chrome the render tests don't) rather than tuned to this image.
const DIFF_THRESHOLD := 0.01
const DIFF_TOL := 2


func _p(s: String) -> void:
	print("STYLEPRESETS  %s" % s)


func _ok(cond: bool, s: String) -> void:
	if cond:
		print("STYLEPRESETS  ok    %s" % s)
	else:
		_fail += 1
		print("STYLEPRESETS  FAIL  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _find(n: Node, script_file: String) -> Node:
	if n.get_script() != null and String(n.get_script().resource_path).ends_with(script_file):
		return n
	for c in n.get_children(true):
		var r := _find(c, script_file)
		if r != null:
			return r
	return null


func _parse_args() -> bool:
	var args := OS.get_cmdline_user_args()
	for i in args.size():
		var a := String(args[i])
		if not a.begins_with("--"):
			continue
		var known := false
		for f in KNOWN_FLAGS:
			if a == f or a.begins_with(f + "="):
				known = true
		if not known:
			print("STYLEPRESETS  UNKNOWN ARGUMENT %s -- known: %s" % [a, str(KNOWN_FLAGS)])
			return false
	for i in args.size():
		if String(args[i]) == "--out" and i + 1 < args.size():
			_out = String(args[i + 1])
		if String(args[i]) == "--tag" and i + 1 < args.size():
			_tag = String(args[i + 1])
	if _out == "":
		print("STYLEPRESETS  NO --out DIR given.")
		return false
	DirAccess.make_dir_recursive_absolute(_out)
	return true


func _generate() -> void:
	bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout


func _shot() -> Image:
	await get_tree().create_timer(0.5).timeout
	await _frames(4)
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()


## Fraction of RGB bytes differing by more than `DIFF_TOL` levels. `a`/`b`
## must be the same format and size; a size mismatch (a resized window
## between shots) is reported as "moved everything" rather than crashing.
func _pixel_diff_frac(a: Image, b: Image) -> float:
	if a.get_size() != b.get_size() or a.get_format() != b.get_format():
		return 1.0
	var da := a.get_data()
	var db := b.get_data()
	var n := mini(da.size(), db.size())
	if n == 0:
		return 1.0
	var d := 0
	for i in n:
		if absi(int(da[i]) - int(db[i])) > DIFF_TOL:
			d += 1
	return float(d) / float(n)


## Every key `values` (the full `STYLE_MANAGED` bundle merged with the
## preset's own overrides) names, checked against `live` (the bridge
## readback) with a small float tolerance -- `set_npr`/`set_appearance` clamp
## to their own declared ranges, and every value used here is already inside
## them, so an exact-ish match (not bitwise) is the right bar.
func _assert_dict_matches(live: Dictionary, want: Dictionary, ctx: String) -> void:
	for key in want:
		if not live.has(key):
			_ok(false, "%s: live dict has no '%s' key" % [ctx, key])
			continue
		var lv = live[key]
		var wv = want[key]
		if typeof(wv) == TYPE_BOOL or typeof(lv) == TYPE_BOOL:
			_ok(bool(lv) == bool(wv), "%s: %s live=%s want=%s" % [ctx, key, str(lv), str(wv)])
		else:
			_ok(absf(float(lv) - float(wv)) < 0.001, "%s: %s live=%f want=%f" % [ctx, key, float(lv), float(wv)])


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 900.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG TIMEOUT"); get_tree().quit(3))
	wd.start()

	if not _parse_args():
		get_tree().quit(2)
		return

	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	bridge = app.bridge

	if DisplayServer.get_name() == "headless":
		_ok(false, "run WINDOWED -- see this probe's header")
		get_tree().quit(1)
		return

	await _generate()
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(8)

	app.select_domain_category("cartography", "Style")
	await _frames(8)
	rw = _find(app, "render_workspace.gd")
	_ok(rw != null, "the RenderWorkspace node is reachable")
	if rw == null:
		get_tree().quit(1)
		return

	var tiles: Array = rw.get("_preset_tiles")
	_ok(tiles.size() == RenderWorkspace.STYLE_PRESETS.size(), "one tile per STYLE_PRESETS entry")

	var prev_img: Image = null
	var prev_name := ""
	var shots: Array = []  # [[name, Image]]

	for i in RenderWorkspace.STYLE_PRESETS.size():
		var entry: Array = RenderWorkspace.STYLE_PRESETS[i]
		var name := String(entry[0])
		var btn: Button = tiles[i]["button"]
		## §1: through the real UI path -- the tile's own `pressed` signal,
		## which is exactly what `_preset_tile`'s `btn.pressed.connect(...)`
		## wires to a click.
		btn.emit_signal("pressed")
		await _frames(4)
		await get_tree().create_timer(0.3).timeout

		## §2: live knobs vs. the preset's own definition.
		_ok(bridge.look() == String(entry[1]), "%s: live look == '%s'" % [name, String(entry[1])])
		var want_npr: Dictionary = RenderWorkspace.STYLE_MANAGED.duplicate()
		for key in Dictionary(entry[2]):
			want_npr[key] = entry[2][key]
		_assert_dict_matches(bridge.npr_settings(), want_npr, "%s NPR" % name)
		if entry.size() > 3:
			_assert_dict_matches(bridge.appearance(), Dictionary(entry[3]), "%s appearance" % name)
		if entry.size() > 4:
			var stops: Array = bridge.color_ramp()
			_ok(not stops.is_empty(), "%s: ramp '%s' loaded with stops" % [name, String(entry[4])])

		## §3: the frame differs from the previous preset's.
		var img := await _shot()
		if prev_img != null:
			var frac := _pixel_diff_frac(prev_img, img)
			_ok(frac > DIFF_THRESHOLD, "%s vs %s: pixel diff %.4f > %.4f" % [name, prev_name, frac, DIFF_THRESHOLD])
		else:
			_p("%s: first preset, no predecessor to diff against" % name)

		## §4: one screenshot per preset.
		var path := "%s/%s_preset_%02d_%s.png" % [_out, _tag, i, name.to_snake_case()]
		img.save_png(path)
		_p("saved %s" % path)
		shots.append([name, img])

		prev_img = img
		prev_name = name

	## Contact sheet: every preset's screenshot, scaled down, in one grid, in
	## `STYLE_PRESETS` order (reading order left-to-right, top-to-bottom).
	if not shots.is_empty():
		var cols := 4
		var rows := int(ceil(float(shots.size()) / float(cols)))
		var cell_w := 480
		var cell_h := 300
		var sheet := Image.create(cell_w * cols, cell_h * rows, false, Image.FORMAT_RGB8)
		sheet.fill(Color(0.08, 0.08, 0.09))
		for i in shots.size():
			var nm: String = shots[i][0]
			var im: Image = (shots[i][1] as Image).duplicate()
			im.resize(cell_w, cell_h - 24, Image.INTERPOLATE_BILINEAR)
			## The viewport screenshot format (typically RGBA8) does not
			## match the sheet's RGB8 -- `blit_rect` refuses a format
			## mismatch outright rather than converting, so this must.
			im.convert(sheet.get_format())
			var cx := (i % cols) * cell_w
			var cy := (i / cols) * cell_h
			sheet.blit_rect(im, Rect2i(Vector2i.ZERO, im.get_size()), Vector2i(cx, cy + 24))
			_p("contact sheet cell %d = %s" % [i, nm])
		var sheet_path := "%s/%s_contact_sheet.png" % [_out, _tag]
		## No burned-in labels: rendering glyph textures into `Image` cells
		## without a `Viewport` pass is not worth the machinery here. The
		## per-preset filenames already carry the name, and the "contact
		## sheet cell N = name" lines above give the same index -> name
		## mapping to read the sheet against.
		sheet.save_png(sheet_path)
		_p("saved contact sheet %s (%dx%d, %d cells)" % [sheet_path, sheet.get_width(), sheet.get_height(), shots.size()])

	_p("=== SUMMARY: %d failures ===" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
