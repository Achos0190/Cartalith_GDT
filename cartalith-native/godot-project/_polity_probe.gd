extends Node
## FH-5 (`FACTION_HUB_DESIGN.md` §5.2): the Place editor's Polity picker moves
## the claim grid. Drives the real picker on a real generated world.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _polity_probe.tscn [-- --shots DIR] [--vp WxH --force-touch]
##
## Windowed only (it reads the territory texture's pixels, and
## `ImageTexture.update()` is a no-op under `--headless`). Without `--vp` the
## shell boots in the window (desktop); with `--vp WxH` it boots inside a
## SubViewport of that size, which with `--force-touch` is the phone form --
## the `_ctxphone_probe.gd` convention. `--shots DIR` saves the confirm dialog
## and the territory raster before/after as PNGs there. An unknown argument
## aborts with exit 2.
##
## Legs, each opening with what it protects:
##   1. tooltip -- the old "does not repaint the borders" prose is gone;
##   2. cancel -- a province move asks first, and Cancel changes nothing;
##   3. confirm -- the dialog states the preview's cell count, and exactly
##      those cells move from A to B in the claim grid and the drawn raster;
##   4. recompute -- the moved cells survive Recompute civilisation;
##   5. one cell -- a non-seed place moves its own cell with no dialog;
##   6. no grid -- a project reopened without its claim grid relabels only,
##      with no dialog, and says the borders did not move.
##
## Prints `POLITY ok|FAIL ...` per check and a RESULT line. Exit 0 all green,
## 1 a failure, 2 could not run (headless, no candidate, missing binding).

var app: Node
var bridge
var _fails := 0
var _checks := 0
var _shots := ""
var _vp: SubViewport


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _check(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("POLITY %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1
		print("PROBE-FAIL %s" % what)


func _abort(why: String) -> void:
	print("### POLITY ABORT: %s ###" % why)
	get_tree().quit(2)


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


## The Polity picker: the `OptionButton` in the row whose tooltip is the
## Polity tooltip (`DccWidgets._row` puts the tooltip on the row).
func _polity_row() -> HBoxContainer:
	var all: Array = []
	_walk(app.place_editor_window, all)
	for n in all:
		if n is HBoxContainer and (n as HBoxContainer).tooltip_text.begins_with("Which polity"):
			return n
	return null


func _polity_picker() -> OptionButton:
	var row := _polity_row()
	if row == null:
		return null
	for c in row.get_children(true):
		if c is OptionButton:
			return c
	return null


## Every live "Move polity?" confirm under the shell.
func _dialogs() -> Array:
	var all: Array = []
	_walk(app, all)
	var out: Array = []
	for n in all:
		if n is ConfirmationDialog and (n as ConfirmationDialog).title == "Move polity?" \
				and not n.is_queued_for_deletion():
			out.append(n)
	return out


## A dialog's text, desktop (`dialog_text`) or phone (labels in its body).
func _dialog_text(d: ConfirmationDialog) -> String:
	var parts := PackedStringArray([d.dialog_text])
	var all: Array = []
	_walk(d, all)
	for n in all:
		if n is Label:
			parts.append((n as Label).text)
		elif n is RichTextLabel:
			parts.append((n as RichTextLabel).get_parsed_text())
	return "\n".join(parts)


## The whole claim grid as `sample_cell().control` (0 = unowned, -1 = no
## grid) -- the engine's own per-cell owner, independent of the picker.
func _grid() -> PackedInt32Array:
	var g: Vector2i = bridge.grid_size()
	var out := PackedInt32Array()
	out.resize(g.x * g.y)
	for y in g.y:
		for x in g.x:
			out[y * g.x + x] = int((bridge.sample_cell(x, y) as Dictionary).get("control", -1))
	return out


func _diff(a: PackedInt32Array, b: PackedInt32Array) -> Array:
	var out: Array = []
	for i in a.size():
		if a[i] != b[i]:
			out.append(i)
	return out


func _terr_image() -> Image:
	var t: Texture2D = bridge.territory_texture()
	return t.get_image() if t != null else null


func _pixels_differ(a: Image, b: Image) -> int:
	if a == null or b == null or a.get_size() != b.get_size():
		return -1
	var n := 0
	for y in a.get_height():
		for x in a.get_width():
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
	return n


func _shot(name: String) -> void:
	if _shots == "":
		return
	await RenderingServer.frame_post_draw
	var img: Image = (_vp.get_texture() if _vp != null else get_viewport().get_texture()).get_image()
	img.save_png(_shots.path_join("polity_%s.png" % name))


func _save_terr(name: String) -> void:
	if _shots == "":
		return
	var img := _terr_image()
	if img != null:
		img.save_png(_shots.path_join("polity_terr_%s.png" % name))


func _status() -> String:
	return String((app._status_labels["hint"] as Label).text)


## Selects `to` in the picker the way a user's choice does: `select()` plus the
## `item_selected` signal `DccWidgets.choice` listens to.
func _pick(to: int) -> bool:
	var ob := _polity_picker()
	if ob == null:
		return false
	for i in ob.item_count:
		if ob.get_item_text(i).begins_with("%d · " % to):
			ob.select(i)
			ob.item_selected.emit(i)
			return true
	return false


func _picker_shows(f: int) -> bool:
	var ob := _polity_picker()
	return ob != null and ob.selected >= 0 and ob.get_item_text(ob.selected).begins_with("%d · " % f)


## First settlement passing `want(row, preview)` for some other faction.
func _candidate(want: Callable) -> Dictionary:
	var rows: Array = bridge.settlements()
	var factions: Array = bridge.get_factions()
	for i in rows.size():
		var s: Dictionary = rows[i]
		for f in factions:
			var b := int((f as Dictionary)["id"])
			if b == int(s["faction"]):
				continue
			var pv: Dictionary = bridge.civ_polity_reassign_preview(i, b)
			if bool(pv.get("ok", false)) and want.call(s, pv):
				return {"index": i, "to": b, "from": int(s["faction"]), "pv": pv, "s": s}
	return {}


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		_abort("headless -- run windowed")
		return
	var vp_size := Vector2i.ZERO
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a == "--vp" and i + 1 < args.size():
			var wh := args[i + 1].split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
		elif a == "--shots" and i + 1 < args.size():
			_shots = args[i + 1]
			i += 2
		elif a == "--force-touch":
			i += 1
		else:
			_abort("unknown argument '%s'" % a)
			return
	var wd := Timer.new()
	wd.wait_time = 270.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("### POLITY WATCHDOG ###"); get_tree().quit(3))
	wd.start()

	app = load("res://shell/app.tscn").instantiate()
	if vp_size != Vector2i.ZERO:
		_vp = SubViewport.new()
		_vp.size = vp_size
		_vp.transparent_bg = false
		_vp.gui_embed_subwindows = true
		_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		add_child(_vp)
		_vp.add_child(app)
	else:
		add_child(app)
	await get_tree().create_timer(1.0).timeout
	bridge = app.bridge
	if not bridge.world_gen.has_method("civ_polity_reassign_preview"):
		_abort("this DLL has no civ_polity_reassign_preview")
		return
	bridge.generate({"seed": 9137, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("FORM booted as %s" % form)
	if not await _run():
		return
	print("### POLITY RESULT %s  %d/%d checks passed (form %s) ###" % [
		"GREEN" if _fails == 0 else "RED", _checks - _fails, _checks, form])
	get_tree().quit(0 if _fails == 0 else 1)


## Returns false when the probe could not run (it has already quit with 2),
## so no RESULT line claims a pass it never measured.
func _run() -> bool:
	var seed_c := _candidate(func(s: Dictionary, pv: Dictionary):
		return int(pv.get("cells", 0)) >= 2 and not bool(pv.get("one_cell_only", true)) \
			and not bool(pv.get("loses_capital", false)))
	if seed_c.is_empty():
		## A world whose only province seeds are capitals (seed 9137 at 256x192
		## is one): take a capital, whose move also reports `loses_capital`.
		seed_c = _candidate(func(s: Dictionary, pv: Dictionary):
			return int(pv.get("cells", 0)) >= 2 and not bool(pv.get("one_cell_only", true)))
	if seed_c.is_empty():
		_abort("no province-seed candidate on this world")
		return false
	var idx := int(seed_c["index"])
	var a := int(seed_c["from"])
	var b := int(seed_c["to"])
	var pv: Dictionary = seed_c["pv"]
	var n := int(pv["cells"])
	print("candidate: settlement %d (%s, %s) %d -> %d, %d cells" % [idx, seed_c["s"]["name"], seed_c["s"]["kind"], a, b, n])

	# 1. tooltip
	app.place_editor_window.open_for(idx)
	await _frames(6)
	var row := _polity_row()
	_check("tooltip: the Polity row exists (Protects: the picker is found by its tooltip)", row != null)
	if row == null:
		return true
	_check("tooltip: describes the land move (Protects: §5.2 item 10, prose follows behaviour)",
		row.tooltip_text.contains("moves the settlement's land") and row.tooltip_text.contains("not undoable"))
	_check("tooltip: the old 'does not repaint' prose is gone", not row.tooltip_text.contains("does not repaint"))

	# 2. cancel
	var g0 := _grid()
	_check("premise: the place stands on its own polity's land", g0[int(seed_c["s"]["y"]) * bridge.grid_size().x + int(seed_c["s"]["x"])] == a)
	_check("cancel: picker selects B", _pick(b))
	await _frames(6)
	var ds := _dialogs()
	_check("cancel: a province move asks first (Protects: §5.2 item 7)", ds.size() == 1, str(ds.size()))
	if ds.size() == 1:
		var d: ConfirmationDialog = ds[0]
		var t := _dialog_text(d)
		_check("cancel: the dialog states the preview's cell count", t.contains("%d cells" % n), t.left(160))
		_check("cancel: the dialog says not undoable (Protects: §5.2 item 8)", t.contains("Not undoable; use the Territory tool to repaint"))
		_check("cancel: the dialog names both polities", t.contains(String(pv["from_name"])) and t.contains(String(pv["to_name"])))
		if bool(pv.get("loses_capital", false)):
			_check("cancel: a capital move names the consequence (Protects: §5.2 item 5)", t.contains("only capital"), t)
		await _shot("confirm_dialog")
		d.canceled.emit()
		await _frames(6)
	_check("cancel: nothing moved (Protects: a dialog is a question, not an apply)", _diff(g0, _grid()).is_empty())
	_check("cancel: the settlement keeps A", int(bridge.settlements()[idx]["faction"]) == a)
	_check("cancel: the picker shows A again", _picker_shows(a))

	# 3. confirm
	app.open_faction_roster(a, "territory")
	await _frames(6)
	var t0 := _terr_image()
	_save_terr("before")
	await _shot("before")
	_check("confirm: picker selects B", _pick(b))
	await _frames(6)
	ds = _dialogs()
	_check("confirm: the dialog is up (positive control for the no-dialog legs)", ds.size() == 1, str(ds.size()))
	if ds.size() == 1:
		(ds[0] as ConfirmationDialog).confirmed.emit()
	await _frames(10)
	var g1 := _grid()
	var moved := _diff(g0, g1)
	_check("confirm: exactly the previewed number of cells moved (Protects: §5.2 item 1)", moved.size() == n, "%d vs %d" % [moved.size(), n])
	var all_ab := not moved.is_empty()
	for c in moved:
		if g0[c] != a or g1[c] != b:
			all_ab = false
	_check("confirm: every moved cell went from A to B", all_ab)
	_check("confirm: the settlement is B's", int(bridge.settlements()[idx]["faction"]) == b)
	var t1 := _terr_image()
	var px := _pixels_differ(t0, t1)
	_check("confirm: the drawn territory raster changed (Protects: the map refresh after the move)", px > 0, str(px))
	_check("confirm: the map shows the new raster", app.viewport.territory_view.texture != null)
	_check("confirm: the status line reports the move", _status().contains("%d cells moved" % n), _status())
	_check("confirm: the picker shows B", _picker_shows(b))
	_check("confirm: the Factions hub stayed open and rebuilt", app.faction_roster_window.visible)
	_save_terr("after")
	await _shot("after")
	app.faction_roster_window.hide()

	# 4. recompute
	var r: Dictionary = bridge.civ_recompute()
	await _frames(6)
	_check("recompute: ran", bool(r.get("ok", false)), str(r.get("reason", "")))
	var g2 := _grid()
	var kept := true
	for c in moved:
		if g2[c] != b:
			kept = false
	_check("recompute: every moved cell is still B's (Protects: the move is paint, it survives Recompute)", kept)
	_check("recompute: the settlement is still B's", int(bridge.settlements()[idx]["faction"]) == b)

	# 5. one cell, no dialog
	var one := _candidate(func(s: Dictionary, p: Dictionary):
		return int(p.get("cells", 0)) == 1 and bool(p.get("one_cell_only", false)) and not bool(p.get("loses_capital", false)))
	_check("one cell: a non-seed candidate exists", not one.is_empty())
	if not one.is_empty():
		var oi := int(one["index"])
		var ob := int(one["to"])
		app.place_editor_window.open_for(oi)
		await _frames(6)
		var h0 := _grid()
		_check("one cell: picker selects B", _pick(ob))
		await _frames(8)
		_check("one cell: no dialog (Protects: §5.2 item 7, a one-cell move applies directly)", _dialogs().is_empty())
		var dd := _diff(h0, _grid())
		var own: int = int(one["s"]["y"]) * bridge.grid_size().x + int(one["s"]["x"])
		_check("one cell: exactly its own cell moved", dd.size() == 1 and int(dd[0]) == own, str(dd))
		_check("one cell: the settlement is B's", int(bridge.settlements()[oi]["faction"]) == ob)
		_check("one cell: the status line says so", _status().contains("one cell moved"), _status())

	# 6. no claim grid
	app.place_editor_window.hide()
	var path := OS.get_user_data_dir().path_join("_polity_probe.zip")
	var stripped := OS.get_user_data_dir().path_join("_polity_probe_noterr.zip")
	var done := [false]
	app._write_project(path, func(): done[0] = true)
	for _i in 200:
		if done[0]:
			break
		await _frames(2)
	_check("no grid: the project was written", FileAccess.file_exists(path))
	_check("no grid: the claim grid was taken out", _strip(path, stripped, "rasters/territory.") == 1)
	_check("no grid: it reopens", bool(app._load_project(stripped)))
	await _frames(10)
	bridge = app.bridge
	var rows: Array = bridge.settlements()
	var ni := 0
	var na := int(rows[0]["faction"]) if not rows.is_empty() else 1
	var nb := 1 if na != 1 else 2
	var npv: Dictionary = bridge.civ_polity_reassign_preview(ni, nb)
	_check("no grid: the preview says no_claim_grid", String(npv.get("reason", "")) == "no_claim_grid", str(npv))
	app.place_editor_window.open_for(ni)
	await _frames(6)
	_check("no grid: picker selects B", _pick(nb))
	await _frames(6)
	_check("no grid: no dialog", _dialogs().is_empty())
	_check("no grid: the label still changes (Protects: the §5.3 fallback)", int(bridge.settlements()[ni]["faction"]) == nb)
	_check("no grid: the status line says the borders did not move", _status().contains("borders not moved"), _status())
	app.place_editor_window.hide()
	return true


## Copy `src` to `dst` without every entry whose name starts with `prefix`
## (`_claimsabsent_probe.gd`'s helper); returns how many were dropped.
func _strip(src: String, dst: String, prefix: String) -> int:
	var reader := ZIPReader.new()
	if reader.open(src) != OK:
		return -1
	var packer := ZIPPacker.new()
	if packer.open(dst) != OK:
		return -1
	var dropped := 0
	for name in reader.get_files():
		if name.begins_with(prefix):
			dropped += 1
			continue
		packer.start_file(name)
		packer.write_file(reader.read_file(name))
		packer.close_file()
	packer.close()
	reader.close()
	return dropped
