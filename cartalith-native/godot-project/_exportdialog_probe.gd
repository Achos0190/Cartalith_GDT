extends Node
## Probe for `shell/export_dialog.gd` (`EXPORT_SCOPE.md` §7 E5, first slice)
## and its menu row `Data ▸ Export ▸ Export map image…` (`menus.gd`,
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
## Writes a screenshot `user://_exportdialog_<mode>.png` and prints its path.

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
	_ok(not row.is_empty(), "the Data menu carries 'Export map image…'")
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

	print("RESULT: %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
