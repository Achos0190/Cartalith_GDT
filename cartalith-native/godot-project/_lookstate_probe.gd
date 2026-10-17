extends Node
## Probe for two shell backlog rows found by ET-F3 (STATUS.md 2026-10-12):
##   1. loading a saved look never restored the map texture it was saved from -- the look
##      file held the Painter (NPR) block but `appearance_rebased` always took the SESSION's
##      npr, so a look saved on Ink and reloaded over Default rendered with Default's npr;
##   2. a project open kept the session's lit style-preset tile (and readout) even though
##      the opened project's own look / Painter block described another preset or none.
##
## Protects: (1) save look on a base preset, switch away, load it => the map texture is
## byte-identical to the one saved; (2) after `_load_project` the lit tile is the preset the
## project's engine state matches (look + Painter block), or Custom when none does.
##
## MUST run WINDOWED from a SCRATCH COPY of the project (own `config/name`, its own user
## dir, so the live cartalith_settings.cfg is never written):
##   Godot_v4.7.x_win64_console.exe --path <copy> res://_lookstate_probe.tscn -- --out DIR
## Exit 0 = every check held; 1 = a check failed; 2 = could not run.
## Result: an assertion, windowed only. Command per the header: `res://_lookstate_probe.tscn -- --out DIR` (`--out`
## required). Prints `LKST  ok    <what>`, `PROBE-FAIL LKST  <what>` per miss and `LKST  done  <n> checks, <n> failed`;
## stderr carries `PROBE-CANNOT-RUN: ...` (headless, missing `--out`, no world, no RenderWorkspace, 900 s watchdog). Exit
## 0 = every check held, 1 = a check failed, 2 = could not run.

const RW := preload("res://shell/workspaces/render_workspace.gd")

var GRID := Vector2i(512, 328)
var _out := ""
var _fail := 0
var _checks := 0
var _app: Node
var _vh: Control
var _br: Node
var _rw: Node
var _tiles: Array = []


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if cond:
		print("LKST  ok    " + what)
	else:
		_fail += 1
		print("PROBE-FAIL LKST  " + what)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _find(n: Node, script_file: String) -> Node:
	if n.get_script() != null and String(n.get_script().resource_path).ends_with(script_file):
		return n
	for c in n.get_children(true):
		var r := _find(c, script_file)
		if r != null:
			return r
	return null


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless -- run windowed.")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	for i in args.size():
		if args[i] == "--out" and i + 1 < args.size():
			_out = args[i + 1]
	if _out == "":
		printerr("PROBE-CANNOT-RUN: --out DIR is required")
		get_tree().quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		printerr("PROBE-CANNOT-RUN: watchdog"); get_tree().quit(2))
	DisplayServer.window_set_size(Vector2i(1600, 1000))
	await _frames(2)
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	_br.generate({"seed": 483920, "width_km": 1200.0, "grid_w": GRID.x, "grid_h": GRID.y,
		"archetype": "", "villages": true, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout
	if _br.generating or _br.settlements().is_empty():
		printerr("PROBE-CANNOT-RUN: no generated world")
		get_tree().quit(2)
		return
	_app.select_domain_category("cartography", "Style")
	await _frames(8)
	_rw = _find(_app, "render_workspace.gd")
	if _rw == null:
		printerr("PROBE-CANNOT-RUN: no RenderWorkspace")
		get_tree().quit(2)
		return
	_tiles = _rw.get("_preset_tiles")
	_vh.reset_view()
	await _settle()
	await _leg_look_texture()
	await _leg_open_tile()
	print("LKST  done  %d checks, %d failed" % [_checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)


func _settle() -> void:
	await _frames(8)
	var t0 := Time.get_ticks_msec()
	while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.3).timeout
	await _frames(4)


func _press_base(i: int) -> void:
	(_tiles[i]["button"] as Button).emit_signal("pressed")
	await _settle()


func _idx(n: String) -> int:
	for i in RW.STYLE_PRESETS.size():
		if String(RW.STYLE_PRESETS[i][0]) == n:
			return i
	return -1


## The engine's map texture as RGB8 bytes (`_etf3_probe.gd`'s measure).
func _tex_bytes() -> PackedByteArray:
	var t: Texture2D = _vh.map_view.texture
	if t == null:
		return PackedByteArray()
	var im := t.get_image()
	im.convert(Image.FORMAT_RGB8)
	return im.get_data()


func _ndiff(a: PackedByteArray, b: PackedByteArray) -> int:
	if a.size() != b.size():
		return maxi(a.size(), b.size())
	var n := 0
	for i in a.size():
		if a[i] != b[i]:
			n += 1
	return n


func _look_file(name: String) -> String:
	return "user://appearance_presets/%s.json" % _br.call("_preset_slug", name)


func _load_look(name: String) -> bool:
	var pick: OptionButton = _rw.get("_preset_pick")
	for i in pick.item_count:
		if pick.get_item_text(i) == name:
			pick.select(i)
			_rw.call("_on_load_look")
			await _settle()
			return true
	return false


func _save_look(name: String) -> void:
	_rw.get("_preset_name").text = name
	_rw.call("_on_save_look")


## Row 1: a base preset's look, saved, switched away from and loaded, must repaint the same map.
func _leg_look_texture() -> void:
	var made: Array = []
	for pname in ["Ink", "Antique", "Village"]:
		var i := _idx(pname)
		_ok(i >= 0, "1: the gallery has a %s tile" % pname)
		await _press_base(i)
		var t_saved := _tex_bytes()
		var lname: String = "lkst " + pname
		_save_look(lname)
		made.append(lname)
		await _press_base(_idx("Default"))
		var t_default := _tex_bytes()
		_ok(not t_saved.is_empty() and _ndiff(t_saved, t_default) > 0,
			"1: positive control -- %s and Default render differently" % pname)
		_rw.call("_refresh_preset_list")
		_ok(await _load_look(lname), "1: the %s look loaded" % pname)
		var nd := _ndiff(_tex_bytes(), t_saved)
		_ok(nd == 0, "1: %s look reload reproduces the saved map texture (%d of %d bytes differ)" % [pname, nd, t_saved.size()])
	for n in made:
		DirAccess.remove_absolute(ProjectSettings.globalize_path(_look_file(n)))
	_rw.call("_refresh_preset_list")


func _save(path: String) -> bool:
	if FileAccess.file_exists(path):
		DirAccess.remove_absolute(path)
	_app._write_project(path)
	await _frames(2)
	return FileAccess.file_exists(path)


func _open(path: String) -> bool:
	var ok: bool = _app._load_project(path)
	await get_tree().create_timer(0.5).timeout
	await _settle()
	return ok


## Row 2: what a project open lights must follow the OPENED project, not the session.
func _leg_open_tile() -> void:
	var d := _idx("Default")
	var ink := _idx("Ink")
	var p_def := _out.path_join("p_default.zip")
	var p_ink := _out.path_join("p_ink.zip")
	var p_cus := _out.path_join("p_custom.zip")
	await _press_base(d)
	_ok(await _save(p_def), "2: the Default-state project saved")
	await _press_base(ink)
	_ok(await _save(p_ink), "2: the Ink-state project saved")
	## Custom: Ink plus a Painter value no tile carries.
	_br.set_npr({"hachure": 0.37})
	_rw.call("_mark_custom")
	await _settle()
	_ok(await _save(p_cus), "2: the Custom-state project saved")

	await _press_base(ink)
	_ok(await _open(p_def), "2: the Default project opened over a session lit on Ink")
	_ok(int(_rw.get("_active_preset")) == d, "2: ... lights Default, not the session's Ink (active=%d)" % int(_rw.get("_active_preset")))
	_ok(String(_vh._style_readout) == "Default", "2: ... and the readout says Default (%s)" % String(_vh._style_readout))

	await _press_base(d)
	_ok(await _open(p_ink), "2: the Ink project opened over a session lit on Default")
	_ok(int(_rw.get("_active_preset")) == ink, "2: ... lights Ink, not the session's Default (active=%d)" % int(_rw.get("_active_preset")))

	await _press_base(ink)
	_ok(await _open(p_cus), "2: the Custom project opened over a session lit on Ink")
	_ok(int(_rw.get("_active_preset")) == -1, "2: ... lights no tile (Custom), not the session's Ink (active=%d)" % int(_rw.get("_active_preset")))
	_ok(String(_vh._style_readout) == "Custom", "2: ... and the readout says Custom (%s)" % String(_vh._style_readout))
