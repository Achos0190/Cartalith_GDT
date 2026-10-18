extends Node
## Probe for `shell/export_dialog.gd` (`EXPORT_SCOPE.md` §7 E5, first slice)
## and its menu row `File ▸ Export ▸ Export map image…` (`menus.gd`,
## `ID_EXPORT_MAP_IMAGE`), inside the real shell.
##
##   desktop: Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _exportdialog_probe.tscn
##   phone:   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 393x852 _exportdialog_probe.tscn -- --force-touch
##
## `--force-touch` is `dcc_shell.gd`'s own override and the only user argument
## read; anything else aborts with exit 2. **Windowed, never `--headless`**: the
## export's overlay tiles are `SubViewport` readbacks and the screenshot is the
## framebuffer, and the dummy display driver draws neither (`MISTAKES.md`).
##
## What it protects:
##   1. the menu row is disabled with the no-world reason before a world, and
##      with the busy reason while a generation runs; enabled after;
##   2. the row's id opens the dialog (the real `id_pressed` dispatch), and the
##      dialog fits the viewport;
##   3. sizes: every ladder width listed, 2048 enabled on this world, a refused
##      size carries its reason (`size_refusal` over a forced unaffordable
##      estimate; nothing refused when the platform reports no memory);
##   4. defaults: settlements on at towns (Ruling AZ), sRGB; Export disabled
##      until a destination is chosen, and with the busy reason while
##      `bridge.generating`;
##   5. a SMALL real export (2048 px) end to end through `ExportOverlay.run`,
##      producing a PNG that decodes at 2048 px wide; and a cancel that reports
##      `cancelled` and leaves no file. **Never 32K.**
##   6. an existing destination asks before replacing: declined leaves the file
##      byte-identical and starts nothing; confirmed replaces it;
##   7. the STYLE picker: Current look (default) sends no `style` key; a named
##      engine look sends `style.look`, changes the exported pixels, and leaves
##      the live map's look unchanged;
##   8. JPEG (E5 remainder, 2026-10-18): PNG is the default and sends no
##      `quality`; choosing JPEG shows the QUALITY row, sends `format: jpeg` and
##      `quality: 90`, re-prices the sizes with an "(estimate)" file size, and
##      turns the chosen `.png` destination into `.jpg`; a 2048 px JPEG export
##      end to end is a baseline JFIF (SOI, SOF0) that decodes at 2048 px wide
##      and carries no ICC profile in sRGB; in Display P3 it carries an APP2
##      `ICC_PROFILE` whose profile is a real ICC (`acsp` at byte 36); and the
##      file grows with quality (40 < 90 < 95). **Never 32K.**
## Writes a screenshot `user://_exportdialog_<mode>.png` and prints its path.
## Result: an assertion with screenshots, windowed only (CRLF file). Commands per the header: `--path . --resolution
## 1600x1000 _exportdialog_probe.tscn` (desktop) and `--resolution 393x852 _exportdialog_probe.tscn -- --force-touch`
## (phone); `--force-touch` is the only argument read and anything else prints `ABORT: unrecognised argument <a> -- this
## probe reads --force-touch only` and exits 2. Prints `  ok    |  FAIL  <what>` rows, `== mode <mode>, viewport <size>
## ==`, `== 1. menu row ==` to `== 8. jpeg ==` banners, `  size  <item>` rows, `SHOT <path>` per saved
## screenshot and `RESULT: PASS|FAIL (<n> failed)`. It writes a real 2048 export to disk (section 5). Exit 0 when no
## check failed, 1 otherwise or when the menu row or the dialog is missing, 2 from the watchdog (`WATCHDOG: the probe did
## not finish`).

var _fails := 0

func _ok(cond: bool, what: String) -> void:
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1

func _find_row(n: Node, text: String) -> Array:
	if n is PopupMenu:
		var p := n as PopupMenu
		for i in p.item_count:
			if p.get_item_text(i) == text:
				return [p, i]
	for c in n.get_children(true):
		var r := _find_row(c, text)
		if not r.is_empty():
			return r
	return []

## The JPEG header facts section 8 asserts, read by walking the segments from SOI
## to SOS (ITU-T T.81 B.1: `FF xx`, a big-endian length counting itself):
## `{soi, sof0, sof2, icc_segments, icc_acsp}`. `icc_acsp` is whether the first
## `ICC_PROFILE` chunk's profile has the ICC signature `acsp` at its byte 36.
func _jpeg_facts(b: PackedByteArray) -> Dictionary:
	var out := {"soi": b.size() > 2 and b[0] == 0xFF and b[1] == 0xD8, "sof0": false, "sof2": false,
		"icc_segments": 0, "icc_acsp": false}
	var p := 2
	while p + 4 <= b.size() and b[p] == 0xFF:
		var m: int = b[p + 1]
		var n: int = (b[p + 2] << 8) | b[p + 3]
		if m == 0xC0:
			out["sof0"] = true
		elif m == 0xC2:
			out["sof2"] = true
		elif m == 0xE2 and n >= 16 and b.slice(p + 4, p + 16).get_string_from_ascii() == "ICC_PROFILE":
			if int(out["icc_segments"]) == 0:
				var prof := p + 4 + 14
				out["icc_acsp"] = b.slice(prof + 36, prof + 40).get_string_from_ascii() == "acsp"
			out["icc_segments"] = int(out["icc_segments"]) + 1
		if m == 0xDA:
			break
		p += 2 + n
	return out

func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		if a != "--force-touch":
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch only" % a)
			get_tree().quit(2)
			return
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var mode := "phone" if app.is_phone() else "desktop"
	print("== mode %s, viewport %s ==" % [mode, get_viewport().get_visible_rect().size])
	var bridge: Node = app.bridge

	print("== 1. menu row ==")
	var row := _find_row(app, "Export map image…")
	_ok(not row.is_empty(), "File ▸ Export carries 'Export map image…' (SM-3)")
	if row.is_empty():
		get_tree().quit(1)
		return
	var pm: PopupMenu = row[0]
	var idx: int = row[1]
	pm.about_to_popup.emit()
	_ok(pm.is_item_disabled(idx) and pm.get_item_tooltip(idx).begins_with("No world"),
		"before a world: disabled, reason '%s'" % pm.get_item_tooltip(idx))
	bridge.generate({"seed": 9137, "width_km": 1200.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true})
	if bridge.generating:
		pm.about_to_popup.emit()
		_ok(pm.is_item_disabled(idx) and pm.get_item_tooltip(idx).contains("generation"),
			"while generating: disabled, reason '%s'" % pm.get_item_tooltip(idx))
	else:
		print("  note  generation finished synchronously; busy reason not observed")
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	for _f in 6:
		await get_tree().process_frame
	pm.about_to_popup.emit()
	_ok(not pm.is_item_disabled(idx), "with a world: enabled")

	print("== 2. opens and fits ==")
	pm.id_pressed.emit(pm.get_item_id(idx))
	for _f in 4:
		await get_tree().process_frame
	var d: AcceptDialog = app.export_dialog
	_ok(d != null and d.visible, "the row's id opens the dialog")
	if d == null:
		get_tree().quit(1)
		return
	var vp := get_viewport().get_visible_rect().size
	var rect := Rect2(Vector2(d.position), Vector2(d.size))
	_ok(rect.position.x >= 0 and rect.position.y >= 0 and rect.end.x <= vp.x + 0.5 and rect.end.y <= vp.y + 0.5,
		"dialog %s inside the viewport %s" % [rect, vp])

	print("== 3. sizes ==")
	var sizes: OptionButton = d.get("_sizes")
	var widths: Array = d.get("_widths")
	_ok(sizes.item_count == widths.size() and widths.size() >= 5, "%d sizes listed: %s" % [sizes.item_count, widths])
	var i2048 := widths.find(2048)
	_ok(i2048 >= 0 and not sizes.is_item_disabled(i2048), "2048 is offered and enabled")
	for i in sizes.item_count:
		if sizes.is_item_disabled(i):
			_ok(sizes.get_item_tooltip(i) != "", "disabled %d carries a reason: %s" % [widths[i], sizes.get_item_tooltip(i)])
		print("  size  %s%s" % [sizes.get_item_text(i), "  [disabled]" if sizes.is_item_disabled(i) else ""])
	var DLG = d.get_script()
	_ok(DLG.size_refusal({"ok": true, "band_affordable": false, "band_peak_bytes": 2.0e9, "memory_available": 1.0e9}).contains("2.00 GB"),
		"an unaffordable band is refused with its figures")
	_ok(DLG.size_refusal({"ok": true, "bands": 5, "band_peak_bytes": 2.0e9}) == "",
		"no reported memory refuses nothing")

	print("== 4. defaults and refusals ==")
	var o: Dictionary = d.options()
	_ok(o["content"].get("settlements") == true and o["content"].get("settlement_tier") == "town",
		"settlements on at towns: %s" % o["content"])
	_ok(o["color_space"] == "sRGB", "colour space defaults to sRGB")
	_ok(d.export_refusal() == "Choose where to save the image.", "no destination: '%s'" % d.export_refusal())
	bridge.generating = true
	_ok(d.export_refusal().contains("generation"), "while generating: '%s'" % d.export_refusal())
	bridge.generating = false

	var dir := ProjectSettings.globalize_path("user://_exportdialog_probe")
	DirAccess.make_dir_recursive_absolute(dir)
	var path := dir.path_join("dlg_%s.png" % mode)
	if FileAccess.file_exists(path):
		DirAccess.remove_absolute(path)
	sizes.select(i2048)
	d.set_destination(path)
	_ok(d.export_refusal() == "", "2048 with a destination: Export enabled")
	await get_tree().create_timer(0.5).timeout
	var shot := "user://_exportdialog_%s.png" % mode
	get_viewport().get_texture().get_image().save_png(shot)
	print("SHOT %s" % ProjectSettings.globalize_path(shot))

	print("== 5. a real 2048 export, then a cancel ==")
	var seen := [0.0]
	d.get("_bar").value_changed.connect(func(v: float): seen[0] = maxf(seen[0], v))
	d.start_export()
	var res: Dictionary = await d.export_finished
	_ok(bool(res.get("ok", false)), "export ok: %s" % res)
	var f := FileAccess.open(path, FileAccess.READ)
	var sig := f.get_buffer(8) if f != null else PackedByteArray()
	_ok(sig == PackedByteArray([137, 80, 78, 71, 13, 10, 26, 10]), "the file is a PNG")
	var img := Image.load_from_file(path)
	_ok(img != null and img.get_width() == 2048, "decodes at 2048 px wide (%s)" % (img.get_size() if img else "null"))
	_ok(seen[0] >= 0.99, "the progress bar reached the end (%.2f)" % seen[0])
	_ok(d.get("_status").text.begins_with("Wrote"), "status: '%s'" % d.get("_status").text)

	var cpath := dir.path_join("dlg_%s_cancel.png" % mode)
	d.set_destination(cpath)
	d.start_export()
	for _f in 3:
		await get_tree().process_frame
	d.cancel_export()
	var cr: Dictionary = await d.export_finished
	_ok(bool(cr.get("cancelled", false)), "cancel reports cancelled: %s" % cr)
	_ok(not FileAccess.file_exists(cpath), "a cancelled export leaves no file")

	## The engine refuses a new raster export while a cancelled one is still
	## stopping its terrain render (`export_session.rs::STOPPING`); give it time.
	await get_tree().create_timer(2.5).timeout

	print("== 6. overwrite confirmation ==")
	var sentinel := PackedByteArray([1, 2, 3, 4, 5])
	var wf := FileAccess.open(path, FileAccess.WRITE)
	wf.store_buffer(sentinel)
	wf.close()
	d.set_destination(path)
	d.start_export()
	for _f in 3:
		await get_tree().process_frame
	var ow: ConfirmationDialog = d.get("_overwrite_dlg")
	_ok(ow != null and is_instance_valid(ow) and ow.visible, "an existing destination opens the replace prompt")
	_ok(not bool(d.get("_running")), "nothing runs while the prompt is open")
	var oshot := "user://_exportdialog_overwrite_%s.png" % mode
	get_viewport().get_texture().get_image().save_png(oshot)
	print("SHOT %s" % ProjectSettings.globalize_path(oshot))
	if ow != null and is_instance_valid(ow):
		ow.canceled.emit()
	for _f in 3:
		await get_tree().process_frame
	_ok(FileAccess.get_file_as_bytes(path) == sentinel, "declined: the existing file is untouched")
	_ok(not bool(d.get("_running")) and d.get("_overwrite_dlg") == null, "declined: no export started")
	d.start_export()
	for _f in 3:
		await get_tree().process_frame
	ow = d.get("_overwrite_dlg")
	_ok(ow != null and is_instance_valid(ow), "asked again on a second Export")
	if ow != null and is_instance_valid(ow):
		ow.confirmed.emit()
	var rw: Dictionary = await d.export_finished
	_ok(bool(rw.get("ok", false)) and FileAccess.get_file_as_bytes(path) != sentinel, "confirmed: the file is replaced by the export")

	print("== 7. look override ==")
	var lk: OptionButton = d.get("_look")
	var names: Array = bridge.looks()
	_ok(lk.item_count == names.size() + 1 and lk.get_item_text(0) == "Current look" and lk.selected == 0,
		"picker: Current look + the engine's %d looks, default Current look" % names.size())
	_ok(not d.options().has("style"), "Current look sends no style key")
	var look_before: String = bridge.world_gen.get_look()
	var pick := 1
	for i in names.size():
		if String(names[i]) != look_before:
			pick = i + 1
			break
	lk.select(pick)
	_ok(d.options().get("style", {}).get("look") == String(names[pick - 1]), "a named look sends style.look: %s" % d.options().get("style"))
	var spath := dir.path_join("dlg_%s_look.png" % mode)
	if FileAccess.file_exists(spath):
		DirAccess.remove_absolute(spath)
	d.set_destination(spath)
	d.start_export()
	var rs: Dictionary = await d.export_finished
	_ok(bool(rs.get("ok", false)), "export with a named look ok: %s" % rs)
	var simg := Image.load_from_file(spath)
	var cimg := Image.load_from_file(path)
	_ok(simg != null and cimg != null and simg.get_data() != cimg.get_data(), "the named look changed the exported pixels")
	_ok(bridge.world_gen.get_look() == look_before, "the live map's look is unchanged (%s)" % look_before)
	await get_tree().create_timer(0.4).timeout
	var lshot := "user://_exportdialog_look_%s.png" % mode
	get_viewport().get_texture().get_image().save_png(lshot)
	print("SHOT %s" % ProjectSettings.globalize_path(lshot))

	print("== 8. jpeg ==")
	lk.select(0)
	var fmt: OptionButton = d.get("_format")
	var qrow: Control = d.get("_quality_row")
	var qs: HSlider = d.get("_quality")
	_ok(fmt != null and fmt.item_count == 2 and fmt.get_item_text(0) == "PNG" and fmt.get_item_text(1) == "JPEG"
		and fmt.selected == 0, "FORMAT: PNG (default) and JPEG")
	_ok(not qrow.visible and d.options().get("format") == "png" and not d.options().has("quality"),
		"PNG: quality hidden, sends format png and no quality: %s" % d.options())
	var jpath := dir.path_join("dlg_%s_j.png" % mode)
	d.set_destination(jpath)
	fmt.select(1)
	fmt.item_selected.emit(1)
	for _f in 3:
		await get_tree().process_frame
	var o8: Dictionary = d.options()
	_ok(qrow.visible and o8.get("format") == "jpeg" and int(o8.get("quality", -1)) == 90,
		"JPEG: quality row shown, sends format jpeg quality 90: %s / %s" % [o8.get("format"), o8.get("quality")])
	jpath = String(d.get("_path"))
	_ok(jpath.ends_with("dlg_%s_j.jpg" % mode), "the destination became .jpg: %s" % jpath)
	_ok(sizes.get_item_text(i2048).contains("(estimate)"), "2048 is priced as a JPEG: '%s'" % sizes.get_item_text(i2048))
	sizes.select(i2048)
	await get_tree().create_timer(0.4).timeout
	var jshot := "user://_exportdialog_jpeg_%s.png" % mode
	get_viewport().get_texture().get_image().save_png(jshot)
	print("SHOT %s" % ProjectSettings.globalize_path(jshot))
	var jsizes := {}
	for q in [90, 40, 95]:
		qs.value = q
		var qpath := dir.path_join("dlg_%s_q%d.jpg" % [mode, q])
		if FileAccess.file_exists(qpath):
			DirAccess.remove_absolute(qpath)
		d.set_destination(qpath)
		d.start_export()
		var rj: Dictionary = await d.export_finished
		var jb := FileAccess.get_file_as_bytes(qpath)
		jsizes[q] = jb.size()
		_ok(bool(rj.get("ok", false)) and rj.get("format") == "jpeg" and int(rj.get("quality", -1)) == q,
			"q%d export ok, reports jpeg q%d: %s" % [q, q, rj])
		if q == 90:
			var facts := _jpeg_facts(jb)
			_ok(bool(facts["soi"]) and bool(facts["sof0"]) and not bool(facts["sof2"]), "baseline JFIF: %s" % facts)
			_ok(int(facts["icc_segments"]) == 0, "sRGB JPEG carries no ICC profile")
			var jimg := Image.new()
			var err := jimg.load_jpg_from_buffer(jb)
			_ok(err == OK and jimg.get_width() == 2048, "decodes at 2048 px wide (%s, %s)" % [err, jimg.get_size()])
			if err == OK:
				var crop := jimg.get_region(Rect2i(jimg.get_width() / 2 - 256, jimg.get_height() / 2 - 160, 512, 320))
				var cshot := "user://_exportdialog_jpegcrop_%s.png" % mode
				crop.save_png(cshot)
				print("SHOT %s" % ProjectSettings.globalize_path(cshot))
		DirAccess.remove_absolute(qpath)
	print("  jpeg sizes at 2048: %s" % jsizes)
	_ok(int(jsizes[40]) < int(jsizes[90]) and int(jsizes[90]) < int(jsizes[95]), "file grows with quality: %s" % jsizes)
	qs.value = 90
	var p3 := -1
	var sp: OptionButton = d.get("_space")
	for k in sp.item_count:
		if sp.get_item_text(k) == "Display P3":
			p3 = k
	_ok(p3 >= 0, "Display P3 is offered")
	if p3 >= 0:
		sp.select(p3)
		var ppath := dir.path_join("dlg_%s_p3.jpg" % mode)
		if FileAccess.file_exists(ppath):
			DirAccess.remove_absolute(ppath)
		d.set_destination(ppath)
		d.start_export()
		var rp: Dictionary = await d.export_finished
		var pb := FileAccess.get_file_as_bytes(ppath)
		var pf := _jpeg_facts(pb)
		_ok(bool(rp.get("ok", false)) and rp.get("color_space") == "Display P3", "P3 JPEG export ok: %s" % rp)
		_ok(int(pf["icc_segments"]) >= 1 and bool(pf["icc_acsp"]), "P3 JPEG embeds an ICC profile (APP2): %s" % pf)
		var pimg := Image.new()
		_ok(pimg.load_jpg_from_buffer(pb) == OK and pimg.get_width() == 2048, "P3 JPEG decodes at 2048 px")
		DirAccess.remove_absolute(ppath)
		sp.select(0)
	fmt.select(0)
	fmt.item_selected.emit(0)
	_ok(not qrow.visible and not d.options().has("quality"), "back to PNG: quality hidden and not sent")

	for p2 in [path, cpath, spath]:
		if FileAccess.file_exists(p2):
			DirAccess.remove_absolute(p2)

	print("RESULT: %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
