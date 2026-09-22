extends Node
## Committed verification harness for EXPORT_SCOPE.md §7 E3 -- the export
## options dictionary (`export_image` / `export_image_estimate`).
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _exportoptions_probe.tscn
##
## Headless is sound here: every pixel compared is a PNG read back from disk
## with Image.load, never an ImageTexture (MISTAKES.md's headless no-op).
##
## What it checks, and against what:
##  1. A style override leaves the session's appearance untouched -- every
##     appearance getter, AND a session export taken before and after is
##     byte-identical.
##  2. The override means what the session edit means: export_image with
##     style.look / style.preset / style.ramp+tunables equals export_raster_png
##     after the equivalent set_look / load_appearance_preset / load_ramp_preset
##     + set_appearance (the monolithic path, so this is also E1's identity at
##     this width).
##  3. The estimate's band figures are the hand-worked literals for this
##     2048 x 1311 grid (tests/export_options.rs has the arithmetic), and a real
##     16K export runs exactly that many bands and matches the monolithic 16K
##     export pixel for pixel.
##  4. Every refusal refuses.
## BigTIFF files are left in the scratch dir for an external Pillow decode.

var bridge: Node
var fails := 0
var dir := ""

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _load(p: String) -> PackedByteArray:
	var img := Image.new()
	if img.load(p) != OK:
		return PackedByteArray()
	img.convert(Image.FORMAT_RGB8)
	return img.get_data()

func _mono(name: String, width: int) -> PackedByteArray:
	var p := dir.path_join(name + ".png")
	var r: Dictionary = bridge.world_gen.export_raster_png(p, width, false)
	if not bool(r.get("ok", false)):
		print("    export_raster_png %s FAILED: %s" % [name, String(r.get("error", ""))])
		return PackedByteArray()
	return _load(p)

func _img(name: String, opts: Dictionary) -> Dictionary:
	var ext := ".tif" if String(opts.get("format", "png")) == "bigtiff" else ".png"
	var p := dir.path_join(name + ext)
	var r: Dictionary = bridge.world_gen.export_image(p, opts)
	if not bool(r.get("ok", false)):
		print("    export_image %s FAILED: %s" % [name, String(r.get("error", ""))])
	elif ext == ".png":
		r["px"] = _load(p)
	return r

func _session() -> Dictionary:
	var wg = bridge.world_gen
	return {
		"appearance": wg.get_appearance(), "look": wg.get_look(), "ramp": wg.get_color_ramp(),
		"ramp_mode": wg.get_ramp_mode(), "layers": wg.get_layer_stack(), "npr": wg.get_npr(),
		"tier": wg.get_quality_tier(),
	}

func _refused(opts: Dictionary, what: String) -> void:
	var r: Dictionary = bridge.world_gen.export_image(dir.path_join("refused.png"), opts)
	var e: Dictionary = bridge.world_gen.export_image_estimate(opts)
	_ok(not bool(r.get("ok", true)) and String(r.get("error", "")) != "", "export_image refuses %s: %s" % [what, String(r.get("error", ""))])
	if what.begins_with("an overlay") or what.begins_with("an off-ladder"):
		return  # the estimate prices these; it does not refuse them
	_ok(not bool(e.get("ok", true)), "export_image_estimate refuses %s" % what)

func _ready() -> void:
	get_tree().create_timer(1800.0).timeout.connect(func() -> void:
		push_error("export-options probe watchdog: _ready never finished")
		get_tree().quit(2))
	bridge = load("res://shell/engine_bridge.gd").new()
	add_child(bridge)
	await get_tree().process_frame
	var wg = bridge.world_gen
	dir = ProjectSettings.globalize_path("user://_exportoptions_probe")
	DirAccess.make_dir_recursive_absolute(dir)
	print("  scratch: %s" % dir)

	print("\n== 0. a 2048 x 1311 world, the app's own grid ==")
	wg.generate_sized(20260906, 1200.0, 2048, 1311)
	bridge.has_world = true
	_ok(wg.get_width() == 2048 and wg.get_height() == 1311, "world is 2048 x 1311")

	print("\n== 1. a style override does not touch the session ==")
	var s0 := _session()
	var base_before := _mono("session_before", 2048)
	_ok(base_before.size() == 2048 * 1311 * 3, "session export decodes at 2048 x 1311")
	var styled := _img("styled", {"width": 2048, "style": {"look": "Antique Parchment",
		"ramp": String(wg.list_ramp_presets()[2]), "tunables": {"exag": 9.5, "relief_lights": 5}}})
	_ok(bool(styled.get("ok", false)), "a look + ramp + tunables export succeeds")
	var s1 := _session()
	for k in s0:
		_ok(var_to_str(s0[k]) == var_to_str(s1[k]), "session %s unchanged after the styled export" % k)
	var base_after := _mono("session_after", 2048)
	_ok(base_before == base_after, "a session export after the styled one is byte-identical to the one before")
	_ok(styled.get("px", PackedByteArray()) != base_before, "the styled export differs from the session's (the override took effect)")

	print("\n== 2. the override means what the session edit means ==")
	## look: export_image(style.look) == export_raster_png after set_look.
	var by_look := _img("by_look", {"width": 2048, "style": {"look": "Antique Parchment"}})
	wg.set_look("Antique Parchment")
	var set_look_px := _mono("set_look", 2048)
	wg.set_look(String(s0["look"]))
	_ok(by_look.get("px", PackedByteArray()) == set_look_px and set_look_px.size() > 0, "style.look == set_look, byte for byte")
	_ok(set_look_px != base_before, "and Antique Parchment is a different picture from the session look")
	## ramp + tunables: == load_ramp_preset + set_appearance, then reset.
	var ramp_name := String(wg.list_ramp_presets()[4])
	var by_edits := _img("by_edits", {"width": 2048, "style": {"ramp": ramp_name, "tunables": {"exag": 3.25, "local_contrast": 0.2}}})
	wg.load_ramp_preset(ramp_name)
	wg.set_appearance({"exag": 3.25, "local_contrast": 0.2})
	var edits_px := _mono("set_edits", 2048)
	wg.reset_appearance()
	_ok(by_edits.get("px", PackedByteArray()) == edits_px and edits_px.size() > 0, "style.ramp + tunables == load_ramp_preset + set_appearance, byte for byte")
	## preset file: == load_appearance_preset.
	wg.set_look("Antique Parchment")
	wg.set_appearance({"exag": 6.5, "sun_az_deg": 200.0})
	var preset_path := dir.path_join("look.json")
	_ok(wg.save_appearance_preset(preset_path, "probe look"), "saved a preset")
	wg.reset_appearance()
	wg.set_look(String(s0["look"]))
	_ok(var_to_str(_session()) == var_to_str(s0), "session restored before the preset comparison")
	var by_preset := _img("by_preset", {"width": 2048, "style": {"preset": preset_path}})
	_ok(var_to_str(_session()) == var_to_str(s0), "session unchanged by a preset-style export")
	wg.load_appearance_preset(preset_path)
	var loaded_px := _mono("loaded_preset", 2048)
	wg.reset_appearance()
	_ok(by_preset.get("px", PackedByteArray()) == loaded_px and loaded_px.size() > 0, "style.preset == load_appearance_preset, byte for byte")
	_ok(var_to_str(_session()) == var_to_str(s0), "session restored after")
	## rivers off is a real content switch.
	var dry := _img("no_rivers", {"width": 2048, "content": {"rivers": false}})
	var wet := _img("rivers", {"width": 2048})
	_ok(wet.get("px", PackedByteArray()) == base_before, "default options == export_raster_png at 2048 (E1 one band)")
	_ok(dry.get("px", PackedByteArray()) != wet.get("px", PackedByteArray()), "content.rivers = false removes the river ink")

	print("\n== 3. the estimate's bands are the real plan ==")
	var widths: PackedInt32Array = wg.export_raster_widths()
	var want := {2048: [1, 1311, 0], 4096: [1, 2622, 0], 8192: [1, 5244, 0], 16384: [5, 2294, 164], 32768: [33, 655, 328]}
	for w in widths:
		var e: Dictionary = wg.export_raster_estimate(w)
		var got := [int(e.get("bands", -1)), int(e.get("band_rows", -1)), int(e.get("apron_rows", -1))]
		print("    %d: %s  band_peak %d  peak %d  file %d  avail %s" % [w, str(got), int(e.get("band_peak_bytes", -1)),
			int(e.get("peak_bytes", -1)), int(e.get("file_bytes", -1)), str(e.get("memory_available", "absent"))])
		_ok(got == want[w], "%d px: bands/rows/apron %s == hand-worked %s" % [w, str(got), str(want[w])])
		if w > 8192:
			_ok(int(e.get("band_peak_bytes", -1)) == 988053504, "%d px: band peak == the 8K monolithic peak" % w)
	var e_flat: Dictionary = wg.export_image_estimate({"width": 32768, "style": {"tunables": {"local_contrast": 0.0}}})
	_ok(bool(e_flat.get("ok", false)) and int(e_flat.get("bands", -1)) == 16 and int(e_flat.get("apron_rows", -1)) == 0,
		"a style with local contrast off prices 32K at 16 bands, no apron (got %s)" % str(e_flat.get("bands")))
	var e_tif: Dictionary = wg.export_image_estimate({"width": 8192, "format": "bigtiff"})
	_ok(bool(e_tif.get("ok", false)) and not e_tif.has("file_bytes"), "bigtiff estimate omits file_bytes (no model)")
	var e_ov: Dictionary = wg.export_image_estimate({"width": 8192, "content": {"settlements": true, "labels": true}})
	_ok(Array(e_ov.get("overlays_pending", PackedStringArray())) == ["settlements", "labels"], "the estimate names the pending overlays")

	print("\n== 4. a real 16K banded export == the monolithic 16K ==")
	var t0 := Time.get_ticks_msec()
	var big := _img("banded_16k", {"width": 16384})
	print("    banded 16K: %s in %.1f s" % [str(big.get("bands")), (Time.get_ticks_msec() - t0) / 1000.0])
	_ok(int(big.get("bands", -1)) == 5, "export_image ran exactly the estimate's 5 bands")
	_ok(int(big.get("height", -1)) == 10488, "16K is 16384 x 10488")
	t0 = Time.get_ticks_msec()
	var mono16 := _mono("mono_16k", 16384)
	print("    monolithic 16K in %.1f s" % ((Time.get_ticks_msec() - t0) / 1000.0))
	var bpx: PackedByteArray = big.get("px", PackedByteArray())
	_ok(bpx.size() == 16384 * 10488 * 3 and bpx == mono16, "banded 16K PNG decodes to the monolithic 16K's pixels exactly")
	big.erase("px")
	mono16 = PackedByteArray()
	bpx = PackedByteArray()
	var tif := _img("banded_16k", {"width": 16384, "format": "bigtiff"})
	_ok(bool(tif.get("ok", false)) and int(tif.get("bands", -1)) == 5, "16K BigTIFF written in 5 bands (%d bytes)" % int(tif.get("bytes", -1)))
	var tif2 := _img("styled_2k", {"width": 2048, "format": "bigtiff", "style": {"look": "Antique Parchment"}})
	_ok(bool(tif2.get("ok", false)), "2K styled BigTIFF written")

	print("\n== 5. refusals ==")
	_refused({"width": 2048, "fromat": "png"}, "an unknown top-level key")
	_refused({"format": "png"}, "a missing width")
	_refused({"width": 2048, "format": "webp"}, "an unknown format")
	_refused({"width": 2048, "style": {"look": "Antique Parchment", "preset": preset_path}}, "a look and a preset together")
	_refused({"width": 2048, "style": {"look": "Sepia"}}, "an unknown look")
	_refused({"width": 2048, "style": {"tunables": {"exagg": 1.0}}}, "an unknown tunable")
	_refused({"width": 2048, "style": {"preset": dir.path_join("missing.json")}}, "a missing preset file")
	_refused({"width": 2048, "content": {"settlements": false, "settlement_tier": "town"}}, "a tier with settlements off")
	_refused({"width": 2048, "content": {"settlements": true, "settlement_tier": "town"}}, "an overlay (settlements) E4 cannot draw yet")
	_refused({"width": 3000}, "an off-ladder width")
	_ok(var_to_str(_session()) == var_to_str(s0), "session unchanged across every refusal")

	print("\n== %s: %d failure(s) ==" % ["PASS" if fails == 0 else "FAIL", fails])
	get_tree().quit(1 if fails > 0 else 0)
