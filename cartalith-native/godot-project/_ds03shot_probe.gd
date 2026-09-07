extends Node
## **DS-03's desktop safety property, measured rather than argued.** Dumps the
## laid-out geometry of every visible Control in the left dock, once per
## (domain, mode) pair, to a file. Run it before a reflow change and after, and
## diff the two files: the panels the change does not touch must not produce a
## single differing line.
##
##   Godot_v4.7.1-stable_win64.exe --path . _ds03shot_probe.tscn -- --out=NAME
##   ... [--vp 800x1280 --force-touch --comp tablet]
##
## **This file used to hardcode 1920x1080 and could not reach the tablet at
## all** (found 2026-09-07, lane PROBE-SIGHT). 1080/1920 is 0.5625, under
## `DccShell._PHONE_ASPECT_MAX` (0.60), so `--force-touch` here booted the
## **phone** composition and no flag combination reached the tablet -- which
## made "verified on tablet" claims that cited this probe claims about tree
## state, not about the tablet. `--vp` is the fix; `--comp` is the guard, and
## the guard matters more than the flag. The three readings it classifies on
## are the ones `_sight3_probe.gd` documents; that file is the capture harness
## and this one stays geometry-only.
##
## **`--vp` defaults to the historical 1920x1080, so an existing before/after
## diff is unaffected.** A `--vp` run writes the same filename, so tag it.
##
## **Geometry, not pixels, and that is deliberate.** A framebuffer hash was
## tried first and is not deterministic here -- two consecutive runs of the
## unchanged tree produced ten different hashes -- so it cannot distinguish a
## layout regression from frame-to-frame noise. Position and size can, they are
## exactly what a reflow change moves, and a differing line names the node.
##
## No `--headless` is needed for this one; it reads the layout, not the frame.

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

var _vp: SubViewport

func _boot(w: int, h: int) -> Node:
	_vp = SubViewport.new()
	_vp.size = Vector2i(w, h)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(30)
	## The Welcome gate is modal and, left up, shifts the dock's rect.
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(30)
	return app

## Depth-first in child order, so the walk is stable across runs -- Godot's own
## `get_children()` order is the scene order, and an auto-generated `@Label@412`
## name is NOT stable between runs, which is why the line carries the *path
## index* rather than the node name.
func _dump(root: Control, out: Array, prefix: String) -> void:
	var i := 0
	for c in root.get_children():
		if c is Control:
			var ctl := c as Control
			var here := "%s/%d:%s" % [prefix, i, ctl.get_class()]
			if ctl.is_visible_in_tree():
				var t: Variant = ctl.get("text")
				out.append("%s pos=%s size=%s min=%s %s" % [
					here, str(ctl.position.round()), str(ctl.size.round()),
					str(ctl.get_combined_minimum_size().round()),
					(String(t).left(48) if t != null else "")])
			_dump(ctl, out, here)
		i += 1

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var tag := "x"
	var args := OS.get_cmdline_user_args()
	for a in args:
		if a.begins_with("--out="):
			tag = a.substr(6)
	## Historical default, kept so an old before/after pair still diffs.
	var vw := 1920
	var vh := 1080
	var vi := args.find("--vp")
	if vi >= 0 and vi + 1 < args.size():
		var p: PackedStringArray = String(args[vi + 1]).split("x")
		vw = int(p[0]); vh = int(p[1])
	var app := await _boot(vw, vh)
	var shell: Node = app
	## **Intrinsic, not the number that was passed in.** `DccTheme.is_tablet()`
	## is a published flag; these three are what the shell actually built --
	## the phone bottom bar's cell count, the tablet-only `node_added` hook, and
	## the laid-out rail width (39 pointer / 47 tablet, one px under
	## `_scaled(W_RAIL_COLLAPSED)`'s 40/48 because the rail panel's right border
	## is taken out of the `VBoxContainer` inside it).
	var cells_v: Variant = app.get("_phone_tab_cells")
	var cells: int = (cells_v as Dictionary).size() if cells_v != null else 0
	var hook: bool = get_tree().node_added.is_connected(
		Callable(app, "_on_tablet_node_added"))
	var rail_w: float = (app.get("rail_column") as Control).size.x
	var comp := "?"
	if cells == DccShell.PHONE_TABS.size() and not hook:
		comp = "phone"
	elif cells == 0 and hook and is_equal_approx(rail_w, 47.0):
		comp = "tablet"
	elif cells == 0 and not hook and is_equal_approx(rail_w, 39.0):
		comp = "pc"
	print("[MODE] vp=%dx%d intrinsic=%s (rail=%.0f cells=%d tablet_hook=%s) flags: touch=%s tablet=%s"
		% [vw, vh, comp, rail_w, cells, str(hook),
			str(DccTheme.is_touch()), str(DccTheme.is_tablet())])
	var ci := args.find("--comp")
	if ci >= 0 and ci + 1 < args.size() and String(args[ci + 1]) != comp:
		print("[FATAL] asked for '%s', the shell that booted is '%s'. Nothing written."
			% [String(args[ci + 1]), comp])
		get_tree().quit(3); return
	var ld := shell.get("left_dock") as Control
	var lines: Array = []
	for n in DccShell.RAIL_NODES:
		if String(n.get("kind", "")) != "node":
			continue
		shell.call("_on_rail_node_pressed", String(n["domain"]), String(n["mode"]))
		await _frames(20)
		lines.append("### %s/%s  dock pos=%s size=%s" % [
			String(n["domain"]), String(n["mode"]),
			str(ld.position.round()), str(ld.size.round())])
		## The shell's root column, so a dock that changed HEIGHT can be traced
		## to whichever band above or below it grew.
		var col := ld.get_parent().get_parent()
		if col is Control:
			var bi := 0
			for bc in (col as Control).get_children():
				if bc is Control:
					lines.append("@@@ band %d %s pos=%s size=%s min=%s" % [bi,
						bc.get_class(), str((bc as Control).position.round()),
						str((bc as Control).size.round()),
						str((bc as Control).get_combined_minimum_size().round())])
				bi += 1
		_dump(ld, lines, "")
	var path := "user://ds03_%s.txt" % tag
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string("\n".join(PackedStringArray(lines)))
	f.close()
	print("[OUT] ", ProjectSettings.globalize_path(path), "  lines=", lines.size())
	get_tree().quit(0)
