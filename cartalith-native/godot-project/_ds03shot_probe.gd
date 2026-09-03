extends Node
## **DS-03's desktop safety property, measured rather than argued.** Dumps the
## laid-out geometry of every visible Control in the left dock, once per
## (domain, mode) pair, to a file. Run it before a reflow change and after, and
## diff the two files: the panels the change does not touch must not produce a
## single differing line.
##
##   Godot_v4.7.1-stable_win64.exe --path . _ds03shot_probe.tscn -- --out=NAME
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
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			tag = a.substr(6)
	var app := await _boot(1920, 1080)
	var shell: Node = app
	print("[MODE] touch=", DccTheme.is_touch(), " tablet=", DccTheme.is_tablet())
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
