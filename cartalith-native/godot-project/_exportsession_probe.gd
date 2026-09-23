extends Node
## Committed verification harness for EXPORT_SCOPE.md §7 E4 batch A -- the
## export overlay session's four `#[func]`s (`export_session_begin`,
## `_submit_tile`, `_finish`, `_abort`), driven end to end with synthetic tiles.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _exportsession_probe.tscn
##
## Headless is sound here: nothing is rendered by Godot. Every tile is bytes
## made in this script, and every comparison is of files on disk.
##
## What it checks, and against what:
##  1. A session fed fully transparent tiles writes a file byte-identical to
##     export_image's for the same options -- PNG at 2048 (one band) and 16384
##     (several bands, the last one short), BigTIFF at 16384.
##  2. A real overlay: one 50%-white premultiplied sample [128,128,128,128]
##     over the whole image lands at 128 + round(t*127/255) for every byte t
##     of the terrain-only export.
##  3. The snapshot: regenerating the world while a session is open does not
##     change what the session writes.
##  4. Refusals: calls with no session, a second begin, a gap at finish, an
##     overlap, a band out of order and an abort all refuse (or clean up) and
##     leave no partial file; export_image still refuses overlays while
##     export_session_begin accepts them.

var bridge: Node
var fails := 0
var dir := ""
var _zeros := {}

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _buf(n: int, fill: int) -> PackedByteArray:
	var key := "%d:%d" % [n, fill]
	if not _zeros.has(key):
		var b := PackedByteArray()
		b.resize(n)
		b.fill(fill)
		_zeros[key] = b
	return _zeros[key]

## Cover every band of the open session, two tiles across and `tile_rows`
## tall, each byte `fill`. Returns the number of tiles, or -1 on a refusal.
func _cover(r: Dictionary, tile_rows: int, fill: int) -> int:
	var wg = bridge.world_gen
	var w := int(r["width"])
	var h := int(r["height"])
	var br := int(r["band_rows"])
	var half := w / 2
	var n := 0
	for b in int(r["bands"]):
		var y0 := b * br
		var rows := mini(br, h - y0)
		var y := y0
		while y < y0 + rows:
			var th := mini(tile_rows, y0 + rows - y)
			for col in [[0, half], [half, w - half]]:
				var t: Dictionary = wg.export_session_submit_tile(b, col[0], y, col[1], th, _buf(col[1] * th * 4, fill))
				if not bool(t.get("ok", false)):
					print("    tile refused: %s" % String(t.get("error", "")))
					return -1
				n += 1
			y += th
	return n

func _bytes(p: String) -> PackedByteArray:
	return FileAccess.get_file_as_bytes(p)

## export_image and a transparent session at the same options; returns
## [image result, session begin result, finish result].
func _pair(name: String, opts: Dictionary, tile_rows: int) -> Array:
	var wg = bridge.world_gen
	var ext := ".tif" if String(opts.get("format", "png")) == "bigtiff" else ".png"
	var a := dir.path_join(name + "_image" + ext)
	var b := dir.path_join(name + "_session" + ext)
	var ri: Dictionary = wg.export_image(a, opts)
	var rb: Dictionary = wg.export_session_begin(b, opts)
	if not bool(rb.get("ok", false)):
		print("    begin FAILED: %s" % String(rb.get("error", "")))
		return [ri, rb, {}]
	var tiles := _cover(rb, tile_rows, 0)
	var rf: Dictionary = wg.export_session_finish()
	print("    %s: %dx%d, %d bands of %d rows, %d tiles, image %d bytes, session %d bytes (%.0f ms)" % [name,
		int(rb["width"]), int(rb["height"]), int(rb["bands"]), int(rb["band_rows"]), tiles,
		int(ri.get("bytes", -1)), int(rf.get("bytes", -1)), float(rf.get("ms", -1.0))])
	var same := _bytes(a) == _bytes(b) and _bytes(a).size() > 0
	_ok(bool(ri.get("ok", false)) and bool(rf.get("ok", false)) and same,
		"%s: transparent session file == export_image file, byte for byte" % name)
	DirAccess.remove_absolute(a)
	DirAccess.remove_absolute(b)
	return [ri, rb, rf]

func _gone(p: String, what: String) -> void:
	_ok(not FileAccess.file_exists(p), "%s leaves no file" % what)

func _ready() -> void:
	get_tree().create_timer(1800.0).timeout.connect(func() -> void:
		push_error("export-session probe watchdog: _ready never finished")
		get_tree().quit(2))
	bridge = load("res://shell/engine_bridge.gd").new()
	add_child(bridge)
	await get_tree().process_frame
	var wg = bridge.world_gen
	dir = ProjectSettings.globalize_path("user://_exportsession_probe")
	DirAccess.make_dir_recursive_absolute(dir)
	print("  scratch: %s" % dir)

	print("\n== 0. a 1024 x 656 world ==")
	wg.generate_sized(20260923, 1200.0, 1024, 656)
	bridge.has_world = true
	_ok(wg.get_width() == 1024 and wg.get_height() == 656, "world is 1024 x 656")
	_ok(wg.export_session_state() == "idle", "no session before begin")

	print("\n== 1. transparent session == export_image ==")
	var r2k := _pair("png2k", {"width": 2048}, 97)
	_ok(int(r2k[1].get("bands", 0)) == 1, "2048 is one band")
	var styled := _pair("png2k_styled", {"width": 2048, "style": {"look": "Antique Parchment"}, "content": {"rivers": false}}, 300)
	_ok(bool(styled[2].get("ok", false)), "a styled, river-less session finishes")
	var r16 := _pair("png16k", {"width": 16384}, 512)
	var bands16 := int(r16[1].get("bands", 0))
	_ok(bands16 > 1, "16384 runs in several bands (%d)" % bands16)
	_ok(int(r16[1].get("height", 0)) % maxi(1, int(r16[1].get("band_rows", 1))) != 0, "and its last band is short")
	_ok(int(r16[2].get("bands", 0)) == bands16, "finish reports the same band count")
	_pair("tif16k", {"width": 16384, "format": "bigtiff"}, 1024)

	print("\n== 2. a [128,128,128,128] overlay composites as 128 + t*127/255 ==")
	var base := dir.path_join("half_base.png")
	var half := dir.path_join("half_session.png")
	wg.export_image(base, {"width": 2048})
	var rb: Dictionary = wg.export_session_begin(half, {"width": 2048})
	_ok(_cover(rb, 256, 128) > 0, "every tile accepted")
	_ok(bool(wg.export_session_finish().get("ok", false)), "finish")
	var ti := Image.new()
	var oi := Image.new()
	ti.load(base)
	oi.load(half)
	ti.convert(Image.FORMAT_RGB8)
	oi.convert(Image.FORMAT_RGB8)
	var t := ti.get_data()
	var o := oi.get_data()
	var wrong := 0
	for i in t.size():
		if o[i] != 128 + (t[i] * 127 + 127) / 255:
			wrong += 1
	_ok(t.size() == int(rb["width"]) * int(rb["height"]) * 3, "decoded %d bytes, the full %dx%d image" % [t.size(), int(rb["width"]), int(rb["height"])])
	_ok(t.size() == o.size() and wrong == 0, "every one of %d bytes is 128 + round(t*127/255) (%d wrong)" % [t.size(), wrong])
	DirAccess.remove_absolute(base)
	DirAccess.remove_absolute(half)

	print("\n== 3. the snapshot ignores edits made while the session is open ==")
	var before := dir.path_join("snap_before.png")
	var during := dir.path_join("snap_session.png")
	wg.export_image(before, {"width": 2048})
	rb = wg.export_session_begin(during, {"width": 2048})
	wg.generate_sized(777, 1200.0, 1024, 656)
	var after := dir.path_join("snap_after.png")
	wg.export_image(after, {"width": 2048})
	_ok(_bytes(after) != _bytes(before), "the regenerate really changed the world's export")
	_ok(_cover(rb, 512, 0) > 0, "tiles accepted after the regenerate")
	_ok(bool(wg.export_session_finish().get("ok", false)), "finish")
	_ok(_bytes(during) == _bytes(before), "the session wrote the world as it was at begin")
	for p in [before, during, after]:
		DirAccess.remove_absolute(p)
	wg.generate_sized(20260923, 1200.0, 1024, 656)

	print("\n== 4. refusals ==")
	_ok(not bool(wg.export_session_submit_tile(0, 0, 0, 1, 1, _buf(4, 0)).get("ok", true)), "a tile with no session is refused")
	_ok(not bool(wg.export_session_finish().get("ok", true)), "finish with no session is refused")
	_ok(not bool(wg.export_session_abort().get("ok", true)), "abort with no session is refused")
	var overlay_opts := {"width": 2048, "content": {"settlements": true, "labels": true}}
	var refused: Dictionary = wg.export_image(dir.path_join("ov.png"), overlay_opts)
	_ok(not bool(refused.get("ok", true)), "export_image still refuses overlays: %s" % String(refused.get("error", "")))
	var p := dir.path_join("abort.png")
	rb = wg.export_session_begin(p, overlay_opts)
	_ok(bool(rb.get("ok", false)), "export_session_begin accepts overlays")
	var second: Dictionary = wg.export_session_begin(dir.path_join("second.png"), {"width": 2048})
	_ok(not bool(second.get("ok", true)) and not FileAccess.file_exists(dir.path_join("second.png")), "a second begin is refused and opens nothing")
	_ok(FileAccess.file_exists(p), "begin opened the file")
	_ok(bool(wg.export_session_abort().get("ok", false)) and wg.export_session_state() == "aborted", "abort")
	_gone(p, "abort")

	p = dir.path_join("gap.png")
	rb = wg.export_session_begin(p, {"width": 16384})
	var w := int(rb["width"])
	var br := int(rb["band_rows"])
	_ok(bool(wg.export_session_submit_tile(0, 0, 0, w, br, _buf(w * br * 4, 0)).get("ok", false)), "band 0 in one tile")
	var fin: Dictionary = wg.export_session_finish()
	_ok(not bool(fin.get("ok", true)), "finish after band 0 of %d is refused: %s" % [int(rb["bands"]), String(fin.get("error", ""))])
	_gone(p, "a gap")

	p = dir.path_join("overlap.png")
	rb = wg.export_session_begin(p, {"width": 2048})
	wg.export_session_submit_tile(0, 0, 0, 100, 10, _buf(100 * 10 * 4, 0))
	var ov: Dictionary = wg.export_session_submit_tile(0, 50, 5, 100, 10, _buf(100 * 10 * 4, 0))
	_ok(not bool(ov.get("ok", true)), "an overlap is refused: %s" % String(ov.get("error", "")))
	_gone(p, "an overlap")

	p = dir.path_join("order.png")
	rb = wg.export_session_begin(p, {"width": 16384})
	var oo: Dictionary = wg.export_session_submit_tile(1, 0, int(rb["band_rows"]), 1, 1, _buf(4, 0))
	_ok(not bool(oo.get("ok", true)), "band 1 before band 0 is refused: %s" % String(oo.get("error", "")))
	_gone(p, "an out-of-order band")
	_ok(wg.export_session_state() == "aborted", "state is aborted")

	print("\n== %s: %d failure(s) ==" % ["PASS" if fails == 0 else "FAIL", fails])
	get_tree().quit(1 if fails > 0 else 0)
