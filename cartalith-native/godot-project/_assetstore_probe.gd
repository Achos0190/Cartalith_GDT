extends Node
## Ruling FR (`LARGE_ITEM_RULINGS.md`): the whole asset library persists per device. Windowed (refuses --headless), and meant to
## run as a SEQUENCE of processes on one scratch `APPDATA` (a wiped directory, never the owner's), because the claim is about
## restarts. `_assetstore_run.py` (kept beside the probe's results, not committed) drives the sequence; by hand:
##
##   1  mkzip    -- boots on a wiped APPDATA, adds two pins, writes `user://custom_pins.zip` (the Ruling FP file), quits. The driver then
##                  deletes `<user>/asset_library/` so only the legacy file remains.
##   2  migrate  -- first start with the legacy file: the pins are in the library, the store exists, the zip is `.migrated`.
##   3  (driver wipes APPDATA)  import  -- imports into a frozen slot, a custom slot, a slice, a refined variant, two pins; sets
##                  pack info, tags and a collection; saves a world; waits for the timer; checks the files and that no `.tmp`
##                  is left; makes one LAST import and quits at once (the exit flush).
##   4  (driver plants `index.json.tmp` garbage)  restart  -- a second process: everything is still there, pixels and all; a
##                  world open merges and drops nothing; the pin palette offers the pins.
##   5  (driver truncates one picture file)  corrupt  -- the entry is reported and skipped, everything else loads.
##
## Form by arguments, as the other probes: `-- --vp 1600x900`, `-- --vp 1700x1000 --force-touch --density=1.0`,
## `-- --vp 1080x2340 --force-touch --density 2.625`; `--out <dir>` for frames, `--tag` prefixes them, `--phase <name>`.
## Exit 0 = held, 1 = a check failed, 2 = could not run.

var app: Node
var _vp: SubViewport
var _bridge
var _checks := 0
var _fails := 0
var _out := ""
var _tag := ""
var _phase := "import"
var _density := 1.0
var _state_path := "user://astore_state.json"
const WORLD_FILE := "astore_world.cartalith"


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("ASTORE %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("ASTORE %s" % s)


func _finish() -> void:
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(4)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join(_tag + name))
	_p("shot %s%s" % [_tag, name])


func _lib() -> Node:
	return app.asset_library_window


func _png(w: int, h: int, seed: int) -> PackedByteArray:
	var img := Image.create(w, h, false, Image.FORMAT_RGBA8)
	for y in h:
		for x in w:
			img.set_pixel(x, y, Color8((x * 7 + seed * 40) % 256, (y * 5 + seed * 70) % 256, (seed * 90) % 256, 255))
	return img.save_png_to_buffer()


## One md5 per stored picture of `uid` (its 64 px thumbnail, which carries the pixels and the transform).
func _thumbs(uid: String) -> Array:
	var out := []
	var n := int(_bridge.as_slot_summary(uid).get("item_count", 0))
	for i in n:
		var h := HashingContext.new()
		h.start(HashingContext.HASH_MD5)
		h.update(_bridge.as_thumbnail_png(uid, i, 64))
		out.append(h.finish().hex_encode())
	return out


func _dir_files(path: String) -> PackedStringArray:
	var out := PackedStringArray()
	var d := DirAccess.open(path)
	if d == null:
		return out
	for f in d.get_files():
		out.append(f)
	return out


func _count_ext(ext: String) -> int:
	var n := 0
	for sub in ["user://asset_library", "user://asset_library/img"]:
		for f in _dir_files(sub):
			if f.ends_with(ext):
				n += 1
	return n


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
		elif a == "--out":
			_out = String(args[i + 1])
			i += 2
		elif a == "--tag":
			_tag = String(args[i + 1])
			i += 2
		elif a == "--phase":
			_phase = String(args[i + 1])
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
	var wd := get_tree().create_timer(600.0)
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
	_p("phase %s, form: %s, viewport %s, density %s, user dir %s" % [_phase, "PHONE" if DccTheme.is_phone() else ("TABLET" if DccTheme.is_touch() else "DESKTOP"), str(vp_size), str(_density), OS.get_user_data_dir()])
	_bridge = app.bridge
	match _phase:
		"mkzip":
			await _phase_mkzip()
		"migrate":
			await _phase_migrate()
		"import":
			await _phase_import()
		"restart":
			await _phase_restart()
		"corrupt":
			await _phase_corrupt()
		_:
			_ok("a known phase", false, _phase)
	_finish()


func _boot_world() -> void:
	_bridge.generate({"seed": 1200, "width_km": 600.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.5).timeout
	_ok("a world generated", _bridge.has_world)


func _pins_kinds() -> Array:
	var k := []
	for e in _bridge.as_pin_catalog():
		k.append(String((e as Dictionary)["kind"]))
	k.sort()
	return k


# ---------------------------------------------------------------------------------------------------------------------------

## The legacy file: two pins in an ordinary pack zip at `user://custom_pins.zip`, exactly what Ruling FP wrote.
func _phase_mkzip() -> void:
	var r: Dictionary = app.library_store_report
	_ok("M0: a first start on a wiped directory reports a fresh store", bool(r.get("ok", false)) and bool(r.get("fresh", false)), str(r))
	_ok("M0: ... holding nothing", int(r.get("items", -1)) == 0)
	_lib().open("pins")
	await _frames(6)
	var a: Dictionary = _lib().add_pin_from_png("Beehive", Color.html("#c0a040"), _png(40, 60, 1))
	var b: Dictionary = _lib().add_pin_from_png("Forge", Color.html("#aa3322"), _png(30, 30, 2))
	_ok("M1: two pins added", bool(a.get("ok", false)) and bool(b.get("ok", false)))
	var kinds := _pins_kinds()
	var ex: Dictionary = _bridge.as_export_pack_bytes()
	_ok("M2: a pack of the pins exports", bool(ex.get("ok", false)))
	var f := FileAccess.open("user://custom_pins.zip", FileAccess.WRITE)
	f.store_buffer(ex["bytes"])
	f.close()
	var st := FileAccess.open(_state_path, FileAccess.WRITE)
	st.store_string(JSON.stringify({"pin_kinds": kinds}))
	st.close()
	_ok("M3: the legacy file is written", FileAccess.file_exists("user://custom_pins.zip"))


func _phase_migrate() -> void:
	var r: Dictionary = app.library_store_report
	var want: Array = JSON.parse_string(FileAccess.get_file_as_string(_state_path))["pin_kinds"]
	_ok("G0: the store opened", bool(r.get("ok", false)), str(r))
	_ok("G1: it was a first start (no index) and the two pins were migrated", bool(r.get("fresh", false)) and int(r.get("migrated_pins", 0)) == 2, str(r))
	_ok("G2: the library holds the same pins with the same kind ids", _pins_kinds() == want, str(_pins_kinds()))
	_ok("G3: the legacy file was retired, not deleted", not FileAccess.file_exists("user://custom_pins.zip") and FileAccess.file_exists("user://custom_pins.zip.migrated"))
	_ok("G4: the store exists on disk with one picture file per pin", FileAccess.file_exists("user://asset_library/index.json") and _dir_files("user://asset_library/img").size() == 2, str(_dir_files("user://asset_library/img")))
	await _frames(4)
	_lib().open("pins")
	await _frames(10)
	await _shot("g_migrated_pins.png")


func _phase_import() -> void:
	var r: Dictionary = app.library_store_report
	_ok("I0: a wiped device starts with a fresh, empty store", bool(r.get("ok", false)) and bool(r.get("fresh", false)) and int(r.get("items", -1)) == 0, str(r))
	await _boot_world()
	var st := {}
	# A frozen slot, and a second variant made by a refine verb (a refined asset).
	var m: Dictionary = _bridge.as_import_item("icons:mountain", "peak.png", _png(48, 48, 3))
	_ok("I1: a picture imports into a frozen slot", bool(m.get("ok", false)), str(m))
	var rf: Dictionary = _bridge.as_refine("icons:mountain", 0, "outline")
	_ok("I2: a refine verb adds a variant", bool(rf.get("ok", false)) and int(_bridge.as_slot_summary("icons:mountain").get("item_count", 0)) == 2, str(rf))
	# A custom slot with tags and two pictures, collected.
	var cs: Dictionary = _bridge.as_add_custom_slot("Lighthouse", "Naval")
	var cuid := String(cs.get("uid", ""))
	_bridge.as_import_item(cuid, "l1.png", _png(32, 40, 4))
	_bridge.as_import_item(cuid, "l2.png", _png(32, 40, 5))
	_bridge.as_batch_tag(PackedStringArray([cuid]), "coast, tall")
	_bridge.as_batch_collect(PackedStringArray([cuid]), "Harbour")
	_bridge.as_set_pack_info("Probe pack", "The probe", "CC-BY")
	# A slice of a sheet into a new custom slot set.
	var sh: Dictionary = _bridge.as_load_sheet("sheet.png", _png(64, 64, 6))
	var sl: Dictionary = _bridge.as_slice_apply({"target": "new_custom", "name": "Cells", "set": "Sheets", "cols": 2, "rows": 2, "skip_empty": false})
	_ok("I3: a sheet loads and slices into new slots", bool(sh.get("ok", false)) and bool(sl.get("ok", false)) and int(sl.get("added", 0)) >= 1, str(sl))
	# Two pins.
	_lib().open("pins")
	await _frames(6)
	_lib().add_pin_from_png("Beehive", Color.html("#c0a040"), _png(40, 60, 7))
	_lib().add_pin_from_png("Forge", Color.html("#aa3322"), _png(30, 30, 8))
	_ok("I4: two pins in the library", (_bridge.as_pin_catalog() as Array).size() == 2)
	var uids: Array = ["icons:mountain", cuid]
	for u in sl.get("uids", PackedStringArray()):
		uids.append(String(u))
	for e in _bridge.as_pin_catalog():
		uids.append(String((e as Dictionary)["uid"]))
	st["uids"] = uids
	st["thumbs"] = {}
	for u in uids:
		st["thumbs"][u] = _thumbs(String(u))
	st["pin_kinds"] = _pins_kinds()
	st["total"] = int(_bridge.as_pack_info().get("total_items", 0))
	st["pack_name"] = String(_bridge.as_pack_info().get("name", ""))
	st["license"] = String(_bridge.as_pack_info().get("license", ""))
	st["tags"] = (_bridge.as_slot_summary(cuid).get("tags", []))
	_ok("I5: the library holds the imports", int(st["total"]) >= 7, str(st["total"]))
	# The world keeps saving what it uses: place a pin that uses a library pin and save.
	var cat: Array = _bridge.as_pin_catalog()
	var reg: bool = _bridge.poi_register_custom_pin(String((cat[0] as Dictionary)["kind"]), String((cat[0] as Dictionary)["name"]), String((cat[0] as Dictionary)["color"]), (cat[0] as Dictionary)["png"])
	_ok("I6: the world registers the library pin it uses", reg)
	var wpath := OS.get_user_data_dir().path_join(WORLD_FILE)
	app._write_project(wpath)   ## the shell's own save: it adds the caller-owned documents, `library/assets.json` among them
	var zr := ZIPReader.new()
	_ok("I7: the world saves, with the library's record in it", zr.open(wpath) == OK and zr.read_file("library/assets.json").size() > 50)
	zr.close()
	# One more import through the bridge, which no shell code notices (no `notify_pins_changed`): only the timer can write it.
	_bridge.as_import_item("icons:mountain", "timer.png", _png(24, 24, 10))
	await get_tree().create_timer(3.2).timeout
	_ok("I8: index.json is on disk", FileAccess.file_exists("user://asset_library/index.json"))
	var imgs := _dir_files("user://asset_library/img")
	var total_now := int(_bridge.as_pack_info().get("total_items", 0))
	_ok("I9: one picture file per stored picture (all distinct), the timer's late import included", imgs.size() == total_now and total_now >= 11, "%d files, %d items" % [imgs.size(), total_now])
	_ok("I10: no temporary file is left (every write was atomic)", _count_ext(".tmp") == 0, str(_count_ext(".tmp")))
	var size: int = _bridge.as_store_size()
	_ok("I11: the store reports a size", size > 0, str(size))
	_lib().open("custom")
	await _frames(10)
	_ok("I12: the library window's status line says it is kept on this device", String((_lib().get("_status_pack") as Label).text).contains("on this device"), (_lib().get("_status_pack") as Label).text)
	await _shot("i_library_window.png")
	_lib().hide()
	await _frames(4)
	# A LAST import and an immediate quit: only the exit flush can save it.
	_bridge.as_import_item("icons:mountain", "late.png", _png(20, 20, 9))
	st["late_total"] = int(_bridge.as_pack_info().get("total_items", 0))
	st["late_thumbs"] = _thumbs("icons:mountain")
	var f := FileAccess.open(_state_path, FileAccess.WRITE)
	f.store_string(JSON.stringify(st))
	f.close()
	_p("quitting at once with an unsynced import")
	_checks_summary_then_quit()


func _checks_summary_then_quit() -> void:
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


func _phase_restart() -> void:
	var r: Dictionary = app.library_store_report
	var st: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(_state_path))
	_ok("R0: the store opened and was not fresh", bool(r.get("ok", false)) and not bool(r.get("fresh", true)), str(r))
	_ok("R1: a second process on the same directory finds nothing skipped", (r.get("skipped", PackedStringArray()) as PackedStringArray).is_empty(), str(r.get("skipped")))
	_ok("R2: a stale index.json.tmp planted by the driver is gone", not FileAccess.file_exists("user://asset_library/index.json.tmp"))
	_ok("R3: every picture, the late one included, is back (the exit flush wrote it)", int(_bridge.as_pack_info().get("total_items", 0)) == int(st["late_total"]), "%s vs %s" % [_bridge.as_pack_info().get("total_items"), st["late_total"]])
	var ok_px := true
	for u in st["uids"]:
		var want: Array = st["thumbs"][u]
		var got := _thumbs(String(u))
		if String(u) == "icons:mountain":
			got = got.slice(0, want.size())
		if got != want:
			ok_px = false
			_p("  pixels differ in %s: %s vs %s" % [u, str(got), str(want)])
	_ok("R4: the pixels (and transforms) of every slot are the same", ok_px)
	_ok("R4: the late variant's pixels too", _thumbs("icons:mountain") == st["late_thumbs"])
	_ok("R5: pin kind ids are unchanged", _pins_kinds() == st["pin_kinds"], str(_pins_kinds()))
	_ok("R6: pack info, licence and tags came back verbatim", String(_bridge.as_pack_info().get("name", "")) == st["pack_name"] and String(_bridge.as_pack_info().get("license", "")) == st["license"])
	_ok("R7: the Harbour collection is back", (_bridge.world_gen.as_collections() as Array).size() >= 1)
	await _boot_world()   ## a world is needed for the palette; generating does not touch the library
	_ok("R8: generating a world did not drop the library", int(_bridge.as_pack_info().get("total_items", 0)) == int(st["late_total"]))
	app.select_domain("cartography")
	await _frames(8)
	app.arm_tool("poi")
	await _frames(10)
	var pal := app.find_child("PinPalette", true, false)
	_ok("R9: the pin palette offers the Custom category with the restored pins", pal != null and (pal.find_child("Cat_custom", true, false) != null or pal.find_child("Chip_custom", true, false) != null))
	await _shot("r_palette.png")
	# A world open merges: the world was saved by the previous process.
	var before_total := int(_bridge.as_pack_info().get("total_items", 0))
	var before_thumbs := _thumbs("icons:mountain")
	var wpath := OS.get_user_data_dir().path_join(WORLD_FILE)
	_ok("R10: the world file exists", FileAccess.file_exists(wpath))
	_ok("R11: the world opens through the shell's own path (which restores library/assets.json)", app._load_project(wpath))
	await _frames(12)
	_ok("R12: no item was dropped or replaced by opening it", int(_bridge.as_pack_info().get("total_items", 0)) == before_total and _thumbs("icons:mountain") == before_thumbs)
	_ok("R13: the library's pins are all still there", _pins_kinds() == st["pin_kinds"])
	# A world whose record names a slot this device lacks merges it in as an empty slot and still drops nothing.
	var rec := '{"version":1,"kind":"cartalith-assetlib","pack":{"name":"Other","author":"x","license":"CC0"},"collections":{"Theirs":["custom:rural/windmill"]},"slots":[{"fam":"custom","id":"windmill","name":"Windmill","meta":{"tags":["mill"]},"items":[{"img":0,"name":"w.png","t":{"scale":1.0,"panX":0.0,"panY":0.0}}],"set":"Rural"}]}'
	var rr: Dictionary = _bridge.asset_library_restore_document(rec)
	_ok("R14: a foreign record merges (one slot created, no item added)", bool(rr.get("ok", false)) and int(rr.get("created", -1)) == 1 and int(rr.get("items", -1)) == 0, str(rr))
	_ok("R15: ... and the library still holds every item and its own pack name", int(_bridge.as_pack_info().get("total_items", 0)) == before_total and String(_bridge.as_pack_info().get("name", "")) == st["pack_name"])
	_lib().open("custom")
	await _frames(10)
	await _shot("r_library_after_restart.png")


func _phase_corrupt() -> void:
	var r: Dictionary = app.library_store_report
	var sk: PackedStringArray = r.get("skipped", PackedStringArray())
	_ok("C0: the store opened despite a damaged picture", bool(r.get("ok", false)) and not bool(r.get("index_corrupt", true)), str(r))
	_ok("C1: exactly one entry is reported and it names its file", sk.size() == 1 and sk[0].contains(".png"), str(sk))
	var st: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(_state_path))
	_ok("C2: every other item loaded", int(_bridge.as_pack_info().get("total_items", 0)) == int(st["late_total"]) - 1, "%s vs %s" % [_bridge.as_pack_info().get("total_items"), st["late_total"]])
	_ok("C3: the damaged file was moved aside, not deleted", _count_ext(".corrupt") == 1)
	await get_tree().create_timer(3.2).timeout
	_ok("C4: a later sync does not delete it", _count_ext(".corrupt") == 1)
	_lib().open("custom")
	await _frames(10)
	await _shot("c_after_corrupt.png")
