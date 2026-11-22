extends RefCounted
## Ruling GW / GX (2026-11-22): a format-3 save carries the art its world uses, driven through the REAL shell. Run it twice, each from
## its own `--root` (so its own empty APPDATA and its own device library), device A first:
##   python -I cartalith-native/tools/fennara_drive.py GODOT --shell desktop res://tools/save_art_drive.gd --dll DLL --out DIR --root C:/cw/<A> --timeout 300
##   python -I ... --root C:/cw/<B> ...                                    (the same script: it finds A's save and becomes device B)
## **Device A** (no `WORLD` yet): a generated world, a custom pin added to the library (`as_pin_add`), registered with the world and
## placed, the world saved to `WORLD`; the zip must hold `library/art/<key>.png` and say `format_version` 3.
## **Device B** (`WORLD` exists, the library is EMPTY: asserted first as the positive control): File open on that save; the pin
## must now be in B's library (`as_pin_catalog`) and on disk after a store sync, and the world must draw it (`PinMarker.custom_info`
## not `missing`). Then a real FORMAT-2 save (`OLD`, the committed 2026-09-24 fixture, copied so nothing of the repo is touched) is
## opened and must load, with the library unchanged. Every check prints `ok` or `FAIL`; the launcher exits 1 on any FAIL.

const SCRATCH := "C:/cw/fmt3_scratch"
const WORLD := "C:/cw/fmt3_scratch/world_a.zip"
const OLD := "C:/cw/fmt3_scratch/old_format2.zip"
var _fails := 0
var app: Node
var bridge


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _find(ctx, t: String) -> Dictionary:
	for e in DriveCtx.dump_ui(ctx, t):
		return e
	return {}


func _wait_for(ctx, t: String, secs: float = 20.0) -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		var e := _find(ctx, t)
		if not e.is_empty():
			return e
		await ctx.wait(0.25)
	return {}


func _generate(ctx) -> void:
	bridge.generate({"seed": 4242, "width_km": 800.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.45})
	await ctx.wait(0.2)
	var t0 := Time.get_ticks_msec()
	while bool(bridge.generating) and Time.get_ticks_msec() - t0 < 300000:
		await ctx.wait(0.25)
	await ctx.wait(2.0)


## A distinctive 64x64 pin picture: a magenta disc with a white core and a cyan stem, so a blank or grey draw cannot pass for it.
func _pin_png() -> PackedByteArray:
	var img := Image.create(64, 64, false, Image.FORMAT_RGBA8)
	for y in 64:
		for x in 64:
			var d := Vector2(x - 32, y - 24).length()
			if d <= 20.0:
				img.set_pixel(x, y, Color(1, 1, 1) if d <= 7.0 else Color(0.9, 0.1, 0.8))
			elif absi(x - 32) <= 3 and y > 44:
				img.set_pixel(x, y, Color(0.0, 0.8, 0.9))
	return img.save_png_to_buffer()


func _past_welcome(ctx) -> void:
	var cont := await _wait_for(ctx, "Continue without a world")
	if not cont.is_empty():
		var r: Array = cont["rect"]
		await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))
		await ctx.wait(0.6)


## Entry names of a zip, by the central directory (works for method-93 entries too: only the names are read).
func _zip_names(path: String) -> PackedStringArray:
	var z := ZIPReader.new()
	if z.open(path) != OK:
		return PackedStringArray()
	var names := z.get_files()
	z.close()
	return names


func _format_version(path: String) -> int:
	var z := ZIPReader.new()
	if z.open(path) != OK:
		return -1
	var j: Variant = JSON.parse_string(z.read_file("project.json").get_string_from_utf8())
	z.close()
	return int((j as Dictionary).get("format_version", -1)) if j is Dictionary else -1


## The open-project gallery's thumbnail of `path`, drawn from the archive's heightmap by the engine (`project_height_samples`):
## format 3's Zstandard rasters cannot be read by Godot's `ZIPReader`, so a thumbnail that is a real map (many distinct colours, not
## the one-colour placeholder) proves the engine path works for that archive.
func _thumb_check(ctx, path: String, label: String) -> void:
	var probe: Object = ClassDB.instantiate("WorldGen")
	var pr: Dictionary = probe.project_height_samples(path, 96, 72)
	ctx.log("engine sample of %s: ok=%s error=%s gw=%s heights=%d" % [path.get_file(), str(pr.get("ok")), str(pr.get("error")), str(pr.get("gw")), (pr.get("heights", PackedFloat32Array()) as PackedFloat32Array).size()])
	var img: Image = OpenProjectDialog._render_thumbnail(path)
	var colours := {}
	if img != null:
		for y in img.get_height():
			for x in img.get_width():
				colours[img.get_pixel(x, y).to_html(false)] = true
	_check(ctx, "%s: the gallery thumbnail is a rendered map, not a placeholder" % label, img != null and colours.size() >= 20,
		"distinct colours=%d" % colours.size())


func run(ctx) -> void:
	app = ctx.get_scene_root()
	if not ("bridge" in app):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	DirAccess.make_dir_recursive_absolute(SCRATCH)
	await _past_welcome(ctx)
	if not FileAccess.file_exists(WORLD):
		await _device_a(ctx)
	else:
		await _device_b(ctx)
	ctx.log("RESULT %s" % ("FAIL: %d check(s)" % _fails if _fails > 0 else "PASS"))


func _device_a(ctx) -> void:
	ctx.log("== device A")
	_check(ctx, "A starts with an empty library", (bridge.as_pin_catalog() as Array).is_empty())
	await _generate(ctx)
	var added: Dictionary = bridge.as_pin_add("Drive Beacon", "#e61acc", _pin_png())
	_check(ctx, "the pin is added to A's library", bool(added.get("ok", false)), str(added))
	var cat: Array = bridge.as_pin_catalog()
	_check(ctx, "A's pin catalog holds it", cat.size() == 1)
	var e: Dictionary = cat[0]
	_check(ctx, "the world registers the pin", bridge.poi_register_custom_pin(String(e["kind"]), String(e["name"]), String(e["color"]), e["png"]))
	var idx: int = bridge.poi_add(128.0, 96.0, String(e["kind"]), "Beacon One")
	_check(ctx, "the pin is placed", idx >= 0)
	app.viewport.refresh_annotations()
	await ctx.wait(1.5)
	ctx.capture("A_pin_placed")
	var r: Dictionary = bridge.project_save_with_documents(WORLD, app._project_documents())
	_check(ctx, "the world saves", bool(r.get("ok", false)), str(r))
	var names := _zip_names(WORLD)
	var art: Array = []
	for n in names:
		if n.begins_with("library/art/"):
			art.append(n)
	_check(ctx, "the zip embeds exactly the one picture the world uses", art.size() == 1, str(art))
	_check(ctx, "project.json says format_version 3", _format_version(WORLD) == 3)
	_thumb_check(ctx, WORLD, "a format-3 save")
	_check(ctx, "the rasters are delta-shuffled entries", names.has("rasters/heightmap.delta-shuffled.f32"), str(names))
	# the old format-2 save for device B is copied here beforehand (the committed 2026-09-24 fixture; this script never touches the repo)
	_check(ctx, "an old format-2 save is in place for device B", FileAccess.file_exists(OLD))


func _device_b(ctx) -> void:
	ctx.log("== device B")
	_check(ctx, "positive control: B's library starts EMPTY", (bridge.as_pin_catalog() as Array).is_empty())
	_check(ctx, "B holds no pin of its own before opening", (bridge.poi_custom_pins() as Array).is_empty())
	_check(ctx, "B's library folder holds no picture yet", DirAccess.get_files_at(ProjectSettings.globalize_path("user://asset_library/img")).is_empty())
	var ok: bool = await app.open_project_busy(WORLD)
	_thumb_check(ctx, OLD, "a format-2 save")
	_check(ctx, "B opens A's format-3 save", ok)
	await ctx.wait(2.0)
	var t_end := Time.get_ticks_msec() + 40000
	while not _find(ctx, "OPENING PROJECT").is_empty() and Time.get_ticks_msec() < t_end:
		await ctx.wait(0.5)
	_check(ctx, "the opening overlay is gone before the frame is taken", _find(ctx, "OPENING PROJECT").is_empty())
	app.viewport.refresh_annotations()
	await ctx.wait(1.0)
	var cat: Array = bridge.as_pin_catalog()
	_check(ctx, "the pin is now in B's library", cat.size() == 1 and String((cat[0] as Dictionary).get("name", "")) == "Drive Beacon", str(cat.size()))
	var pins: Array = bridge.poi_custom_pins()
	_check(ctx, "the world holds its pin", pins.size() == 1)
	if cat.size() == 1 and pins.size() == 1:
		_check(ctx, "library pin and world pin are the same kind id", String((cat[0] as Dictionary)["kind"]) == String((pins[0] as Dictionary)["id"]))
		var info: Dictionary = (load("res://shell/pin_marker.gd") as GDScript).custom_info(String((pins[0] as Dictionary)["id"]))
		_check(ctx, "the pin draws its picture, not the missing marker", not info.is_empty() and not bool(info.get("missing", true)), str(info))
		var img := Image.new()
		img.load_png_from_buffer((cat[0] as Dictionary)["png"])
		var magenta := 0
		for y in img.get_height():
			for x in img.get_width():
				var c := img.get_pixel(x, y)
				if c.a > 0.5 and c.r > 0.8 and c.g < 0.3 and c.b > 0.7:
					magenta += 1
		_check(ctx, "B's library picture is A's magenta disc", magenta > 200, "magenta px=%d" % magenta)
	var sync: Dictionary = bridge.as_store_sync(0)
	## the shell's own store tick may already have written it (then `unchanged`): either way the store is in step
	_check(ctx, "B's library store is in step with its library", bool(sync.get("ok", false)) and not bool(sync.get("pending", true)), str(sync))
	var imgs := DirAccess.get_files_at(ProjectSettings.globalize_path("user://asset_library/img"))
	_check(ctx, "an img/<key>.png file is on B's disk", imgs.size() >= 1, str(imgs))
	await ctx.wait(1.0)
	ctx.capture("B_pin_after_open")
	var live: Image = await ctx.frame(1600)
	if live != null:
		live.save_png(SCRATCH.path_join("B_live_frame.png"))
	# a format-2 save opens as before and leaves the library alone
	var before: int = (bridge.as_pin_catalog() as Array).size()
	var ok2: bool = await app.open_project_busy(OLD)
	await ctx.wait(4.0)
	_check(ctx, "an old format-2 save opens", ok2)
	_check(ctx, "the file really is format 2 (its own project.json says so)", _format_version(OLD) == 2)
	_check(ctx, "and has a world", int(bridge.grid_size().x) > 0)
	_check(ctx, "the device library is unchanged by it", (bridge.as_pin_catalog() as Array).size() == before)
	ctx.capture("B_old_format2_open")
