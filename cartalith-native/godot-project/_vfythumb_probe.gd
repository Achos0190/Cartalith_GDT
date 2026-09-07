extends Node
## VERIFIER-owned re-measurement of the THUMBNAILS lane. Nothing here reads a
## framebuffer, so headless is valid: every leg is a decoded image, a cache
## file or a wall-clock timing.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --headless \
##     _vfythumb_probe.tscn -- --out <dir>

var app: Node
var _bridge: Node
var _out := ""
var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond: print("  PASS  ", what)
	else:
		fails += 1
		print("  FAIL  ", what)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _generate(sd: int) -> void:
	_bridge.generate({
		"seed": sd, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.6).timeout

func _wipe() -> void:
	var d := DirAccess.open("user://thumbs")
	if d == null: return
	for f in d.get_files():
		d.remove(f)

func _cached() -> PackedStringArray:
	var d := DirAccess.open("user://thumbs")
	return PackedStringArray() if d == null else d.get_files()

func _ready() -> void:
	var a := OS.get_cmdline_user_args()
	for i in a.size():
		if a[i] == "--out" and i + 1 < a.size(): _out = a[i + 1]
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(2.0).timeout
	if app.open_project_dialog != null: app.open_project_dialog.hide()
	_bridge = app.bridge
	var root: String = DccSettings.storage_root("projects")
	DirAccess.make_dir_recursive_absolute(root)
	var pa := root.path_join("_vfyA.zip")
	var pb := root.path_join("_vfyB.zip")

	print("\n[1] Two worlds, two seeds, saved by the real engine")
	await _generate(11111)
	_ok(_bridge.save_project(pa), "save_project(_vfyA.zip) seed 11111")
	await _generate(22222)
	_ok(_bridge.save_project(pb), "save_project(_vfyB.zip) seed 22222")

	for pair in [[pa, "A"], [pb, "B"]]:
		var z := ZIPReader.new()
		if z.open(String(pair[0])) == OK:
			var h = JSON.parse_string(z.read_file("project.json").get_string_from_utf8())
			var w: Dictionary = (h as Dictionary)["world"]
			print("  [", pair[1], "] grid ", w["grid_width"], "x", w["grid_height"],
				"  sea_level ", w["sea_level"])
			if _out != "":
				var f := FileAccess.open(_out.path_join("hm" + String(pair[1]) + ".f32"), FileAccess.WRITE)
				f.store_buffer(z.read_file("rasters/heightmap.f32")); f.close()
				var m := FileAccess.open(_out.path_join("meta" + String(pair[1]) + ".json"), FileAccess.WRITE)
				m.store_string(JSON.stringify({"gw": w["grid_width"], "gh": w["grid_height"],
					"sea": w["sea_level"]})); m.close()
			z.close()

	print("\n[2] The tiles are the two worlds, and they differ as coastlines do")
	OpenProjectDialog._thumb_cache.clear()
	_wipe()
	var ia: Image = OpenProjectDialog._render_thumbnail(pa)
	var ib: Image = OpenProjectDialog._render_thumbnail(pb)
	_ok(ia != null and ib != null, "both conforming saves rendered")
	if ia != null and ib != null:
		if _out != "":
			ia.save_png(_out.path_join("tileA.png")); ib.save_png(_out.path_join("tileB.png"))
		var diff := 0
		var sea_a := 0
		var sea_b := 0
		var maskdiff := 0
		var hues := {}
		for y in ia.get_height():
			for x in ia.get_width():
				var ca := ia.get_pixel(x, y)
				var cb := ib.get_pixel(x, y)
				if ca != cb: diff += 1
				var sa: bool = ca.b > ca.r and ca.b > ca.g
				var sb: bool = cb.b > cb.r and cb.b > cb.g
				if sa: sea_a += 1
				if sb: sea_b += 1
				if sa != sb: maskdiff += 1
				hues[ca] = true
		print("  differing px %d/%d   sea px A=%d B=%d   mask disagreement=%d   distinct colours in A=%d"
			% [diff, 96 * 72, sea_a, sea_b, maskdiff, hues.size()])
		_ok(maskdiff > 500,
			"the LAND/SEA masks disagree in %d of 6912 px -- a different coastline, not a different hue" % maskdiff)
		_ok(hues.size() > 200,
			"tile A carries %d distinct colours (an identicon wash is one smooth ramp)" % hues.size())
		_ok(sea_a > 200 and sea_a < 6700, "tile A has both sea and land (%d sea px)" % sea_a)

	print("\n[3] Deterministic: the same archive renders the same tile")
	var ia2: Image = OpenProjectDialog._render_thumbnail(pa)
	var same := true
	for y in ia.get_height():
		for x in ia.get_width():
			if ia.get_pixel(x, y) != ia2.get_pixel(x, y): same = false
	_ok(same, "two independent renders of _vfyA.zip are pixel-identical")

	print("\n[4] Cache, and its invalidation on mtime")
	OpenProjectDialog._thumb_cache.clear()
	_wipe()
	var t1: Texture2D = OpenProjectDialog.thumbnail(pa)
	var t2: Texture2D = OpenProjectDialog.thumbnail(pa)
	_ok(t1 == t2, "same path twice returns the SAME texture object (session cache hit)")
	_ok(t1 is ImageTexture, "a conforming save gets an ImageTexture, not a gradient (%s)" % t1.get_class())
	var files1 := _cached()
	_ok(files1.size() == 1, "one PNG written to user://thumbs (%d)" % files1.size())
	var mt_before := FileAccess.get_modified_time(pa)
	var raw := FileAccess.get_file_as_bytes(pa)
	await get_tree().create_timer(1.4).timeout
	var w2 := FileAccess.open(pa, FileAccess.WRITE); w2.store_buffer(raw); w2.close()
	var mt_after := FileAccess.get_modified_time(pa)
	_ok(mt_after != mt_before, "touching the archive moved its mtime (%d -> %d)" % [mt_before, mt_after])
	var t3: Texture2D = OpenProjectDialog.thumbnail(pa)
	_ok(t3 != t1, "the same path with a NEW mtime re-rendered (different texture object)")
	var files2 := _cached()
	_ok(files2.size() == 1, "the older PNG for that world was pruned -- still %d file(s)" % files2.size())
	if files1.size() == 1 and files2.size() == 1:
		_ok(String(files2[0]) != String(files1[0]),
			"and it is a different key (%s -> %s)" % [files1[0], files2[0]])
	var tb: Texture2D = OpenProjectDialog.thumbnail(pb)
	_ok(_cached().size() == 2, "a second world caches alongside rather than replacing")
	_ok(tb != t3, "two worlds are two textures")

	print("\n[5] Damage falls back to identicon(), it does not throw")
	var pdmg := root.path_join("_vfyDMG.zip")
	var cut := raw.slice(0, int(raw.size() * 0.6))
	var wd := FileAccess.open(pdmg, FileAccess.WRITE); wd.store_buffer(cut); wd.close()
	_ok(OpenProjectDialog._render_thumbnail(pdmg) == null, "a truncated archive renders null")
	var td: Texture2D = OpenProjectDialog.thumbnail(pdmg)
	_ok(td is GradientTexture2D, "and thumbnail() hands back the identicon (%s)" % td.get_class())
	var pfor := root.path_join("_vfyFOREIGN.zip")
	var wf := FileAccess.open(pfor, FileAccess.WRITE)
	wf.store_string("not a zip at all"); wf.close()
	_ok(OpenProjectDialog._render_thumbnail(pfor) == null, "a non-zip renders null")
	_ok(OpenProjectDialog.thumbnail(pfor) is GradientTexture2D, "and gets the identicon")
	_ok(OpenProjectDialog._render_thumbnail(root.path_join("_vfyNOPE.zip")) == null,
		"an absent path renders null")

	print("\n[6] Gallery-open timing, re-measured on the real dialog")
	var dlg = app.open_project_dialog
	dlg.set("_welcome", false)
	dlg.set("_scope", "all")
	var n: int = dlg.call("_paths").size()
	OpenProjectDialog._thumb_cache.clear()
	_wipe()
	var t0 := Time.get_ticks_usec()
	dlg.call("_refresh")
	var cold := (Time.get_ticks_usec() - t0) / 1000.0
	await _frames(2)
	t0 = Time.get_ticks_usec()
	dlg.call("_refresh")
	var warm_mem := (Time.get_ticks_usec() - t0) / 1000.0
	await _frames(2)
	OpenProjectDialog._thumb_cache.clear()
	t0 = Time.get_ticks_usec()
	dlg.call("_refresh")
	var warm_disk := (Time.get_ticks_usec() - t0) / 1000.0
	print("  corpus=%d saves   COLD(no cache)=%.1f ms   WARM(memory)=%.1f ms   WARM(disk)=%.1f ms"
		% [n, cold, warm_mem, warm_disk])
	_ok(n >= 3, "the corpus is a real set of saves (%d)" % n)
	_ok(warm_mem < cold, "the memory-warm open is cheaper than the cold one")
	_ok(warm_disk < cold, "the disk-warm open is cheaper than the cold one")

	for f in ["_vfyA.zip", "_vfyB.zip", "_vfyDMG.zip", "_vfyFOREIGN.zip"]:
		DirAccess.remove_absolute(root.path_join(f))
	print("\n[RESULT] vfythumb fails=", fails)
	get_tree().quit(1 if fails > 0 else 0)
