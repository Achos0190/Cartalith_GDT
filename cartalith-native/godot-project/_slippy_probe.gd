extends Node
## Committed verification harness for the slippy-map pyramid export
## (`OUTSTANDING_WORK.md` §2.5, "Slippy-map tile addressing"): the
## `slippy_export_tiles` binding, and the Data manager's Export ▸ Maps scheme
## row that drives it.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _slippy_probe.tscn
##
## Two halves, and neither is readable from the source:
##
##   A. **The binding**, through EngineBridge on the app's own generated world:
##      every scheme's archive is opened with ZIPReader, its entry count checked
##      against a literal, and the SAME ground is found at each scheme's own
##      address -- XYZ 2/1/0, TMS 2/1/3, WMTS cartalith/2/0/1 -- with a positive
##      control (XYZ 2/1/0 must differ from XYZ 2/1/3, or the flip proves
##      nothing). PNGs are decoded and their sizes checked, @2x at twice.
##   B. **The shell**: pressing the XYZ segment must swap the tile grid row for
##      the zoom range, retina must double the footer's count, `0–8` must be
##      refused with a reason, and a real Export must write a zip whose entries
##      are z/x/y paths.

var app: Node
var dm: Node
var _fail := 0
var _tmp := ""

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("SL %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  (%s)" % detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _find_button(root: Node, text: String) -> Button:
	for c in root.get_children():
		if c is Button and (c as Button).text == text:
			return c
		var r := _find_button(c, text)
		if r != null:
			return r
	return null

## A `_check()` row: a Button whose text is a child Label, not `Button.text`.
func _find_check(root: Node, text: String) -> Button:
	for c in root.get_children():
		if c is Button and _find_label(c, text):
			return c
		var r := _find_check(c, text)
		if r != null:
			return r
	return null

func _find_label(root: Node, text: String) -> bool:
	for c in root.get_children():
		if c is Label and (c as Label).text == text:
			return true
		if _find_label(c, text):
			return true
	return false

## Every entry of a zip held in memory, name -> bytes.
func _unzip(bytes: PackedByteArray, name: String) -> Dictionary:
	var path := _tmp.path_join(name)
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_buffer(bytes)
	f.close()
	var z := ZIPReader.new()
	var out := {}
	if z.open(path) != OK:
		return out
	for n in z.get_files():
		out[n] = z.read_file(n)
	z.close()
	return out

func _png_size(b: PackedByteArray) -> Vector2i:
	var img := Image.new()
	if img.load_png_from_buffer(b) != OK:
		return Vector2i(-1, -1)
	return Vector2i(img.get_width(), img.get_height())

func _ready() -> void:
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("SL ==== WATCHDOG: the probe did not finish ====")
		get_tree().quit(2))
	_tmp = OS.get_user_data_dir().path_join("slippyprobe")
	DirAccess.make_dir_recursive_absolute(_tmp)
	var old := DirAccess.open(_tmp)
	for f in old.get_files():
		old.remove(f)

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 1800:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	if not app.bridge.has_world:
		print("SL  !! generate failed -- nothing else here can run")
		get_tree().quit(1)
		return
	var gw: int = app.bridge.world_gen.get_width()
	var gh: int = app.bridge.world_gen.get_height()
	print("SL world %d x %d" % [gw, gh])

	_binding()
	await _shell()

	print("SL ---- %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

func _binding() -> void:
	var b: Node = app.bridge
	var opts := {"max_z": 2, "tile_size": 64, "retina": true}
	var got := {}
	for s in ["xyz", "tms", "wmts"]:
		var o := opts.duplicate()
		o["scheme"] = s
		var bytes: PackedByteArray = b.slippy_export_tiles(o)
		_check("%s: archive is non-empty" % s, not bytes.is_empty(), "%d bytes" % bytes.size())
		got[s] = _unzip(bytes, "%s.zip" % s)
		## (1 + 4 + 16) tiles x 2 densities + leaflet-preview.html + tiles.json.
		_check("%s: 44 entries" % s, (got[s] as Dictionary).size() == 44, "%d" % (got[s] as Dictionary).size())
		var man = JSON.parse_string((got[s] as Dictionary).get("tiles.json", PackedByteArray()).get_string_from_utf8())
		_check("%s: tiles.json names its scheme" % s, man is Dictionary and man.get("scheme", "") == s, str(man.get("scheme", "?")) if man is Dictionary else "unparsed")
		if man is Dictionary:
			_check("%s: maxzoom 2" % s, int(man.get("maxzoom", -1)) == 2)
			_check("%s: no invented bounds" % s, not man.has("bounds"))

	var x: Dictionary = got["xyz"]
	var t: Dictionary = got["tms"]
	var w: Dictionary = got["wmts"]
	var a: PackedByteArray = x.get("2/1/0.png", PackedByteArray())
	_check("xyz 2/1/0 exists", not a.is_empty())
	_check("tms 2/1/3 is xyz 2/1/0", a == t.get("2/1/3.png", PackedByteArray()))
	_check("wmts cartalith/2/0/1 is xyz 2/1/0", a == w.get("cartalith/2/0/1.png", PackedByteArray()))
	_check("control: xyz 2/1/0 differs from xyz 2/1/3", a != x.get("2/1/3.png", PackedByteArray()))
	var s1 := _png_size(a)
	var s2 := _png_size(x.get("2/1/0@2x.png", PackedByteArray()))
	print("SL   2/1/0 is %s, @2x is %s" % [s1, s2])
	_check("tile's long edge is 64", maxi(s1.x, s1.y) == 64, str(s1))
	## The long edge is exactly doubled; the short edge is `round(2·ts/aspect)`,
	## which can land one pixel off `2·round(ts/aspect)` -- tile_dims rounds
	## each density independently.
	_check("@2x long edge is exactly twice", maxi(s2.x, s2.y) == 128, "%s vs %s" % [s2, s1])
	_check("@2x short edge is twice within a pixel",
		absi(mini(s2.x, s2.y) - 2 * mini(s1.x, s1.y)) <= 1, "%s vs %s" % [s2, s1])
	_check("wmts @2x lives in its own TileMatrixSet", w.has("cartalith@2x/2/0/1.png"))
	_check("an unknown scheme writes nothing",
		(b.slippy_export_tiles({"scheme": "grid", "max_z": 0}) as PackedByteArray).is_empty())
	_check("a missing scheme writes nothing",
		(b.slippy_export_tiles({"max_z": 0}) as PackedByteArray).is_empty())

func _shell() -> void:
	dm = app.data_manager_window
	app.open_data_manager_route("export_maps")
	await _frames(4)
	_check("grid mode first: Tile grid row present", _find_label(dm, "Tile grid"))
	var xyz := _find_button(dm, "XYZ")
	_check("XYZ segment exists and is enabled", xyz != null and not xyz.disabled)
	if xyz == null:
		return
	xyz.pressed.emit()
	await _frames(4)
	_check("XYZ press set the scheme", dm._tx_scheme == 1, str(dm._tx_scheme))
	_check("pyramid mode: Zoom range row present", _find_label(dm, "Zoom range"))
	_check("pyramid mode: Tile grid row gone", not _find_label(dm, "Tile grid"))
	var z8 := _find_button(dm, "0–8")
	_check("0–8 refused, with a reason", z8 != null and z8.disabled and z8.tooltip_text != "")
	## Default z0–4: 1+4+16+64+256.
	_check("footer counts 341 tiles", _find_button(dm, "Export 341 tiles") != null)
	var ret := _find_check(dm, "Retina @2x variants")
	_check("retina toggle exists", ret != null)
	if ret != null:
		ret.pressed.emit()
		await _frames(4)
		_check("retina doubles the footer count", _find_button(dm, "Export 682 tiles") != null)
		ret = _find_check(dm, "Retina @2x variants")
		ret.pressed.emit()
		await _frames(4)

	## A real write through the pane's own Export path, small enough to be quick.
	var z4 := _find_button(dm, "0–4")
	_check("0–4 enabled", z4 != null and not z4.disabled)
	dm._tx_tile = 256
	dm._tx_dest = _tmp.path_join("pane.zip")
	var t0 := Time.get_ticks_msec()
	dm._run_export(false)
	await _frames(3)
	print("SL   pane export took %d ms" % (Time.get_ticks_msec() - t0))
	_check("the run was recorded ok", not dm._runs.is_empty() and bool(dm._runs[0].get("ok", false)),
		str(dm._runs[0]) if not dm._runs.is_empty() else "no run")
	var z := ZIPReader.new()
	_check("pane.zip exists and opens", z.open(dm._tx_dest) == OK)
	var names := z.get_files()
	z.close()
	_check("pane.zip: 341 tiles + leaflet-preview.html + tiles.json", names.size() == 343, "%d" % names.size())
	_check("pane.zip: leaflet-preview.html", names.has("leaflet-preview.html"))
	_check("pane.zip: deepest corner 4/15/15.png", names.has("4/15/15.png"))
	_check("pane.zip: root 0/0/0.png", names.has("0/0/0.png"))
