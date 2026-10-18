extends Node
## Story list windows probe -- milestone SM-6 of `STORY_MENUS_PROPOSAL.md`
## (§3.3, §6, Ruling CK, 2026-10-18): `saved_journeys_window.gd` and
## `conflicts_window.gd`, opened through their Story menu rows on the live
## shell.
##
##   no world      both windows say "No world" and draw no row
##   empty         a world with nothing saved: the Saved journeys empty state
##                 offers the planner; Conflicts offers Draw a conflict
##   journeys      a real route committed, saved through the planner's own
##                 `_commit_saved_journey` (the Save journey prompt's action):
##                 one row per `journey_list()` entry, its stored name, length
##                 and mode; a row press opens the planner on THAT journey
##                 (`_active_journey`), and `open_saved` refuses an unknown id
##   conflicts     three conflicts (one ended before the cursor year, one
##                 ongoing with a named side, one anchored): one row each, in
##                 start-year order; the readout counts the active ones; the
##                 ACTIVE chip keeps only those; the side's faction is named;
##                 "ongoing" for no end year, never 0; a row press selects the
##                 conflict in the right dock's Conflict context
##
## Screenshots (`--out DIR`): each window with rows, `lists_<form>_<name>.png`.
##
## Run WINDOWED from `godot-project/`:
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _storylists_probe.tscn -- --vp 1366x768 --out C:/abs/dir
##   ... -- --vp 1920x1080 --out C:/abs/dir
##   ... -- --force-touch --vp 2560x1600 --out C:/abs/dir   (tablet)
##   ... -- --force-touch --vp 393x852 --out C:/abs/dir     (phone)
##
## `--vp WxH` sizes the host SubViewport; `--force-touch` is `dcc_shell.gd`'s
## override; `--out DIR` is where PNGs go (omitted: no screenshots).
##
## Result: `LISTS ok  |FAIL  <check>` rows and `LISTS RESULT <n> checks, <f>
## failed (<form>)`. Exit 0 when nothing failed, 1 when a check failed, 2 when
## the shell could not be measured, 3 on the watchdog.

var app: Node
var _vp: SubViewport
var _checks := 0
var _fails := 0
var _out := ""
var _form := "desktop"

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _check(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("LISTS %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _shot(name: String) -> void:
	if _out == "":
		return
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	if img == null:
		_check("capture %s returned an image" % name, false)
		return
	var path := _out.path_join("lists_%s_%s.png" % [_form, name])
	img.save_png(path)
	print("LISTS SHOT %s" % path)

## The row containers a window drew, found by the meta each row carries (names
## are auto-renamed between siblings, so only the first keeps its own).
func _rows(w: Node, meta: String) -> Array:
	if w == null:
		return []
	return w.find_children("*", "BoxContainer", true, false).filter(func(n): return n.has_meta(meta))

## Every Label / Button text under `n`.
func _texts(n: Node) -> String:
	var out: PackedStringArray = []
	for c in n.find_children("*", "Control", true, false):
		if c is Label:
			out.append((c as Label).text)
		elif c is Button:
			out.append((c as Button).text)
	return " | ".join(out)

func _readout(w: Node, name: String) -> String:
	var r: Node = w.find_child(name, true, false) if w != null else null
	return (r as Label).text if r != null else ""

func _story() -> PopupMenu:
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == "Story":
			return (ch as MenuButton).get_popup()
	return null

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("LISTS WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("LISTS FATAL extension did not load")
		get_tree().quit(2)
		return
	_out = _arg("--out", "")
	if _out != "":
		DirAccess.make_dir_recursive_absolute(_out)
	var parts: PackedStringArray = _arg("--vp", "1366x768").split("x")
	var vw := int(parts[0])
	var vh := int(parts[1])
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(6)
	var story := _story()
	if story == null:
		print("LISTS FATAL no Story menu")
		get_tree().quit(2)
		return
	_form = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_form += "_%dx%d" % [vw, vh]
	print("LISTS form=%s" % _form)
	var bridge = app.bridge

	## -- no world.
	story.id_pressed.emit(DccMenus.ID_SAVED_JOURNEYS)
	await _frames(4)
	var sj = app.saved_journeys_window
	_check("no world: Saved journeys opens and says No world",
		sj != null and sj.visible and _readout(sj, "SjReadout") == "No world" and _rows(sj, "journey_id").is_empty(),
		_readout(sj, "SjReadout"))
	sj.hide()
	story.id_pressed.emit(DccMenus.ID_CONFLICTS_LIST)
	await _frames(4)
	var cw = app.conflicts_window
	_check("no world: Conflicts opens and says No world",
		cw != null and cw.visible and _readout(cw, "CfReadout") == "No world" and _rows(cw, "conflict_id").is_empty(),
		_readout(cw, "CfReadout"))
	cw.hide()

	bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bool(bridge.generating):
		await get_tree().create_timer(0.25).timeout
	await _frames(4)
	var setts: Array = bridge.settlements()
	_check("the probe world has settlements", setts.size() >= 2, str(setts.size()))

	## -- empty states.
	story.id_pressed.emit(DccMenus.ID_SAVED_JOURNEYS)
	await _frames(4)
	_check("empty: 0 saved journeys, and the planner is offered",
		_readout(sj, "SjReadout") == "0 saved journeys" and sj.find_child("SjOpenPlanner", true, false) != null,
		_readout(sj, "SjReadout"))
	sj.hide()
	story.id_pressed.emit(DccMenus.ID_CONFLICTS_LIST)
	await _frames(4)
	_check("empty: 0 conflicts, and Draw a conflict is offered",
		_readout(cw, "CfReadout").begins_with("0 conflicts") and cw.find_child("CfDraw", true, false) != null,
		_readout(cw, "CfReadout"))
	cw.hide()

	## -- a real journey: route between the first settlement and its nearest
	## neighbour, saved through the planner's own save action.
	var s0: Dictionary = setts[0]
	var best := 1
	var best_d := INF
	for i in range(1, setts.size()):
		var d := Vector2(float(setts[i]["x"]), float(setts[i]["y"])).distance_to(Vector2(float(s0["x"]), float(s0["y"])))
		if d < best_d:
			best_d = d
			best = i
	bridge.route_begin("mixed")
	bridge.route_append_stop(float(s0["x"]) + 0.5, float(s0["y"]) + 0.5)
	bridge.route_append_stop(float(setts[best]["x"]) + 0.5, float(setts[best]["y"]) + 0.5)
	var ri: int = bridge.route_commit()
	_check("a route commits", ri >= 0, str(ri))
	var jpv = app.journey_planner_view
	app.open_journey_planner_with_route(ri)
	await _frames(6)
	jpv._commit_saved_journey("Probe journey A")
	jpv._commit_saved_journey("Probe journey B")
	await _frames(3)
	app.select_domain("world")
	await _frames(3)
	var jl: Array = bridge.journey_list()
	_check("two journeys are in the engine store", jl.size() == 2, str(jl.size()))
	story.id_pressed.emit(DccMenus.ID_SAVED_JOURNEYS)
	await _frames(6)
	var jrows := _rows(sj, "journey_id")
	_check("Saved journeys draws one row per stored journey", jrows.size() == jl.size(), str(jrows.size()))
	_check("the readout counts them", _readout(sj, "SjReadout") == "2 saved journeys", _readout(sj, "SjReadout"))
	var txt := _texts(sj)
	_check("rows carry the stored name, length and mode",
		txt.find("Probe journey A") >= 0 and txt.find(DccUnits.format(float(jl[0].get("km", 0.0)))) >= 0
			and txt.find(String(jl[0].get("mode", "?"))) >= 0, txt.substr(0, 300))
	await _shot("saved_journeys")
	## Press the SECOND row: the planner must load that journey, not the first.
	var want_id := int(jl[1]["id"])
	var row_b: Node = null
	for r in jrows:
		if int(r.get_meta("journey_id")) == want_id:
			row_b = r
	var openb: Button = row_b.find_child("SjOpen", true, false) if row_b != null else null
	if openb != null:
		openb.pressed.emit()
		await _frames(6)
		var civ_panel: Control = app._workspace_panels.get("civilization")
		var active := int(jpv.get("_active_journey"))
		_check("a row press closes the window and opens the planner on that journey",
			not sj.visible and app.active_domain() == "civilization" and civ_panel != null and not civ_panel.visible
				and active >= 0 and int((jpv._journeys[active] as Dictionary).get("engine_id", -1)) == want_id,
			"active=%d sj.visible=%s domain=%s civ_visible=%s" % [active, sj.visible, app.active_domain(),
				civ_panel.visible if civ_panel != null else null])
	else:
		_check("the second journey's row has its open button", false)
	_check("open_saved refuses an id the planner does not hold", jpv.open_saved(987654) == false)
	app.select_domain("world")
	await _frames(3)

	## -- conflicts.
	var year := int(bridge.get_civ_year())
	var factions: Array = bridge.get_factions().filter(func(f): return int(f.get("id", 0)) > 0)
	var fid := int(factions[0]["id"]) if not factions.is_empty() else 0
	var fname := String(factions[0]["name"]) if not factions.is_empty() else ""
	var pt := PackedVector2Array([Vector2(float(s0["x"]), float(s0["y"]))])
	var c_old: Dictionary = bridge.conflict_add({"name": "Old war", "kind": "battle",
		"start_year": year - 50, "end_year": year - 40, "points": pt})
	var c_now: Dictionary = bridge.conflict_add({"name": "Current front", "kind": "front", "start_year": year - 5,
		"ongoing": true, "sides": PackedInt32Array([fid]),
		"points": PackedVector2Array([pt[0], pt[0] + Vector2(6, 3)])})
	var c_anch: Dictionary = bridge.conflict_add({"name": "Siege of a place", "kind": "siege",
		"start_year": year - 1, "end_year": year + 1, "points": pt,
		"anchor_kind": "settlement", "anchor_tid": int(s0["tid"])})
	_check("three conflicts are recorded", bool(c_old.get("ok")) and bool(c_now.get("ok")) and bool(c_anch.get("ok")),
		"%s %s %s" % [c_old, c_now, c_anch])
	var all: Array = bridge.conflict_list()
	var n_active := all.filter(func(c): return bool(c.get("active", false))).size()
	story.id_pressed.emit(DccMenus.ID_CONFLICTS_LIST)
	await _frames(6)
	var crows := _rows(cw, "conflict_id")
	_check("Conflicts draws one row per conflict (%d)" % all.size(), crows.size() == all.size(), str(crows.size()))
	_check("readout: N conflicts · M active in year Y",
		_readout(cw, "CfReadout") == "%d conflicts · %d active in year %d" % [all.size(), n_active, year],
		_readout(cw, "CfReadout"))
	var order: Array = crows.map(func(r): return int(r.get_meta("conflict_id")))
	_check("rows are in start-year order (Old war first)", not order.is_empty() and order[0] == int(c_old["id"]), str(order))
	var ctxt := _texts(cw)
	_check("the ended conflict reads %d–%d, the ongoing one 'ongoing'" % [year - 50, year - 40],
		ctxt.find("%d–%d" % [year - 50, year - 40]) >= 0 and ctxt.find("%d–ongoing" % (year - 5)) >= 0, "")
	_check("the side is named by its faction", fname != "" and ctxt.find(fname) >= 0, fname)
	_check("the anchored conflict names its place",
		ctxt.find(String(s0.get("name", "?"))) >= 0, String(s0.get("name", "")))
	await _shot("conflicts")
	var chip: Button = cw.find_child("CfActiveChip", true, false)
	if chip != null:
		chip.pressed.emit()
		await _frames(4)
		_check("ACTIVE keeps only the %d active conflicts" % n_active,
			_rows(cw, "conflict_id").size() == n_active, str(_rows(cw, "conflict_id").size()))
		cw._set_filter("all")
		await _frames(3)
	var target := int(c_now["id"])
	var trow: Node = null
	for r in _rows(cw, "conflict_id"):
		if int(r.get_meta("conflict_id")) == target:
			trow = r
	var sel: Button = trow.find_child("CfSelect", true, false) if trow != null else null
	if sel != null:
		sel.pressed.emit()
		await _frames(6)
		var rd = app.right_dock_ctrl
		_check("a row press closes the window and selects the conflict in the right dock",
			not cw.visible and String(rd.get("_context")) == "conflict" and int(rd.get("_conflict_id")) == target,
			"%s %s" % [rd.get("_context"), rd.get("_conflict_id")])
		await _shot("conflict_selected")
	else:
		_check("the ongoing conflict's row has its select button", false)

	print("LISTS RESULT %d checks, %d failed (%s)" % [_checks, _fails, _form])
	get_tree().quit(1 if _fails > 0 else 0)
