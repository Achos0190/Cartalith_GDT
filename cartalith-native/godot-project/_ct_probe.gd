extends Node
## Chronos timeline probe (Ruling GD/GE; CT-2, CT-4, CT-5, CT-8): the strip's Items row, filters, the passive
## badge, the prompt and an import, inside the real shell, over a scratch vault in `user://`.
##
##   godot --path . --resolution 1600x900 _ct_probe.tscn -- --shots <dir>
##   godot --path . --resolution 1700x1000 --force-touch --density=1.0 _ct_probe.tscn -- --force-touch --shots <dir>
##   godot --path . --resolution 450x975 --force-touch --density 1.094 _ct_probe.tscn -- --force-touch --shots <dir>
##
## Windowed (never `--headless`), from a worktree with its own scratch APPDATA. Exit 0 pass, 1 fail, 2 abort.
## What it protects: the badge counts what the scan found and opens nothing by itself; row count equals the
## Rust model's; a row click lands the year (and day) on the row; a kind filter hides rows and says how
## many; Keep/Link/Import never change a note byte; an import adds rows and Undo removes them; a private
## note's import is flagged and absent from the Chronos export.

var _fails := 0
var _checks := 0
var _shots := ""
var _mode := "desktop"
var _app: Node
var _bridge
const DIR := "user://_ct_vault"
const Items := preload("res://shell/chronos_items.gd")

func _ok(c: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if c else "  FAIL  ") + what)
	if not c:
		_fails += 1

func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame

func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img != null:
		var p := _shots.path_join("ct_%s_%s.png" % [_mode, tag])
		img.save_png(p)
		print("SHOT %s" % p)

func _write(rel: String, text: String) -> void:
	var path := DIR.path_join(rel)
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()

func _read(rel: String) -> String:
	return FileAccess.get_file_as_string(DIR.path_join(rel))

func _click(c: Control) -> void:
	var at := c.get_global_rect().get_center()
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(8)

func _find_btn(root: Node, name_: String) -> Button:
	for b in root.find_children(name_, "Button", true, false):
		if (b as Control).is_visible_in_tree():
			return b
	return null

func _items_node() -> Node:
	return _app.timeline_row.find_child("ChronosItems", true, false)

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		if args[k] == "--force-touch":
			pass
		elif args[k] == "--shots" and k + 1 < args.size():
			k += 1
			_shots = args[k]
		else:
			print("ABORT: unrecognised argument %s" % args[k])
			get_tree().quit(2)
			return
		k += 1
	get_tree().create_timer(400.0).timeout.connect(func() -> void:
		print("WATCHDOG")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s ==" % _mode)
	_bridge = _app.bridge
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _settle(10)
	await _settle(30)
	if not bool(_bridge.has_world):
		print("[FATAL] no world")
		get_tree().quit(2)
		return

	# the scratch vault: a linked chronos note, a year list, a private chronos note, a table
	_write("Kelv.md", "# Kelv\n```chronos\n* [-340] Founded\n- [1204-03-14] Coronation | grand\n```\n")
	_write("Dun.md", "## History\n- 310: Walls\n- 388: Fire\n")
	_write("dm/Plot.md", "```chronos\n* [900] Coup\n```\n")
	_write("Roster.md", "| Year | Event |\n|---|---|\n| 100 | a |\n| 200 | b |\n| 300 | c |\n")
	var before := {}
	for n in ["Kelv.md", "Dun.md", "dm/Plot.md", "Roster.md"]:
		before[n] = _read(n)
	var con: Dictionary = _bridge.vault_connect(ProjectSettings.globalize_path(DIR), "CT probe vault")
	_ok(bool(con.get("ok", false)), "the scratch vault connects")
	var st: Array = _bridge.settlements()
	var tid := int((st[0] as Dictionary)["tid"])
	var att: Dictionary = _bridge.vault_attach("settlement", tid, String((st[0] as Dictionary)["name"]), "Kelv.md", "")
	_ok(bool(att.get("ok", false)), "Kelv.md is linked to settlement %d" % tid)
	Items.invalidate_scan()

	print("== A. the scan, in Rust, then the strip ==")
	var sc: Dictionary = _bridge.world_gen.timelines_scan(0, 200)
	var cands: Array = sc["candidates"]
	_ok(cands.size() == 4, "the scan found 4 timelines (got %d)" % cands.size())
	_ok(int(sc["badge"]) == 4 and int(sc["next"]) == -1, "badge 4 and the whole list covered")
	var plot := cands.filter(func(c): return String(c["rel"]) == "dm/Plot.md")
	_ok(plot.size() == 1 and plot[0].has("private"), "the dm/ note carries the private label")
	var kelv := cands.filter(func(c): return String(c["rel"]) == "Kelv.md")
	_ok(kelv.size() == 1 and bool(kelv[0]["linked"]), "Kelv.md is marked linked")

	_app.select_domain("civilization")
	await _settle(8)
	_app._tl_expanded = true
	_app._fill_timeline_strip()
	if _app.is_phone():
		_app.set_phone_sim_strip_open(true)
	await _settle(30)
	var node := _items_node() if not _app.is_phone() else _app.find_child("ChronosItems", true, false)
	_ok(node != null, "the Items row is on the strip")
	if node == null:
		get_tree().quit(1)
		return
	var badge := _find_btn(node, "TimelinesBadge")
	_ok(badge != null and badge.text.begins_with("! 4 timelines"), "the passive badge reads '%s'" % ("" if badge == null else badge.text))
	_ok(_app.find_child("ChronosPrompt", true, false) == null, "no prompt opened by itself")
	if _app.is_phone():
		await _click(_find_btn(node, "ItemsToggle"))
	await _shot("strip")

	print("== B. rows equal the model; a click lands the year ==")
	var model: Dictionary = _bridge.world_gen.timeline_items({})
	var total := (model["items"] as Array).size()
	var list: Node = node.find_child("ChronosItemsList", true, false)
	var rows := list.get_children().filter(func(c): return c is Button)
	_ok(total >= 2 and rows.size() == mini(total, Items.ROWS_MAX), "row count %d equals the model's %d (cap %d)" % [rows.size(), total, Items.ROWS_MAX])
	# 217 founding rows crowd the 24 nearest the cursor out, so isolate the Note lane for the click
	await _click(_find_btn(node, "Kind_Note"))
	list = node.find_child("ChronosItemsList", true, false)
	rows = list.get_children().filter(func(c): return c is Button)
	_ok(rows.size() == 2, "the Note lane alone is the linked note's 2 events (got %d)" % rows.size())
	var coronation: Button = null
	for r in rows:
		if (r as Button).text.contains("Coronation"):
			coronation = r
	_ok(coronation != null, "the linked note's Coronation is a row")
	var changes := [0]
	var cb := func() -> void: changes[0] += 1
	_app.timeline_changed.connect(cb)
	if coronation != null:
		await _click(coronation)
	_app.timeline_changed.disconnect(cb)
	await _click(_find_btn(node, "Kind_Note"))
	list = node.find_child("ChronosItemsList", true, false)
	rows = list.get_children().filter(func(c): return c is Button)
	_ok(_app.tl_year() == 1204, "the click moved the year to 1204 (got %d)" % _app.tl_year())
	_ok(int(_app.tl_day()) == 72, "...and the day to 72 = 14 March (got %d)" % int(_app.tl_day()))
	print("  info  timeline_changed fired %d time(s) for a dated row" % changes[0])

	print("== C. a kind filter hides rows and says how many ==")
	var chip := _find_btn(node, "Kind_Conflict")
	await _click(chip)
	list = node.find_child("ChronosItemsList", true, false)
	var left := list.get_children().filter(func(c): return c is Button).size()
	var cnt: Label = node.find_child("ChronosItemsCount", true, false)
	_ok(left == 0 and cnt.text.contains("hidden by filters"), "Conflict only: %d rows and the count line says '%s'" % [left, cnt.text])
	await _click(_find_btn(node, "Kind_Conflict"))
	list = node.find_child("ChronosItemsList", true, false)
	_ok(list.get_children().filter(func(c): return c is Button).size() == rows.size(), "clearing the filter restores the rows")
	await _shot("filtered")

	print("== D. the prompt: defaults, and no note ever changes ==")
	await _click(_find_btn(node, "TimelinesBadge"))
	await _settle(10)
	var pr: Node = _app.find_child("ChronosPrompt", true, false)
	_ok(pr != null and pr.visible, "the badge opens the prompt")
	var cards := pr.find_children("Card_*", "VBoxContainer", true, false)
	_ok(cards.size() == 4, "one card per timeline (got %d)" % cards.size())
	for c in cands:
		var want := "link" if (bool(c["linked"]) and not c.has("private")) else "keep"
		_ok(String(pr._choice[pr._key(c)]) == want, "%s defaults to %s" % [String(c["rel"]), want])
	await _shot("prompt")
	# Keep all (bulk), then Kelv's Link via its Apply
	pr.find_child("KeepAll", true, false).pressed.emit()
	await _settle(10)
	var sc2: Dictionary = _bridge.world_gen.timelines_scan(0, 200)
	_ok(int(sc2["badge"]) == 0, "after Apply Keep to all the badge counts 0 (got %d)" % int(sc2["badge"]))
	for n in before:
		_ok(_read(n) == before[n], "%s is byte-identical after the decisions" % n)

	print("== E. import, dedup, private, export, undo ==")
	var dun: Dictionary = {}
	var plotc: Dictionary = {}
	for c in sc2["candidates"]:
		if String(c["rel"]) == "Dun.md":
			dun = c
		if String(c["rel"]) == "dm/Plot.md":
			plotc = c
	var imp: Dictionary = _bridge.world_gen.timeline_decide("Dun.md", String(dun["form"]), int(dun["ordinal"]), "import", "settlement", tid, 0)
	_ok(bool(imp["ok"]) and (imp["added"] as PackedInt64Array).size() == 2, "import added 2 rows")
	var again: Dictionary = _bridge.world_gen.timeline_decide("Dun.md", String(dun["form"]), int(dun["ordinal"]), "import", "settlement", tid, 0)
	_ok((again["added"] as PackedInt64Array).size() == 0 and int(again["duplicates"]) == 2, "a second import adds 0 and reports 2 duplicates")
	var pimp: Dictionary = _bridge.world_gen.timeline_decide("dm/Plot.md", String(plotc["form"]), int(plotc["ordinal"]), "import", "settlement", tid, 0)
	_ok(pimp.has("private_warning"), "a private note's import warns")
	var cnts: Dictionary = _bridge.world_gen.timeline_imported_counts()
	_ok(int(cnts["imported"]) == 3 and int(cnts["private"]) == 1, "3 imported, 1 private")
	var exp: Dictionary = _bridge.world_gen.timeline_export({})
	_ok(String(exp["text"]).contains("Walls") and not String(exp["text"]).contains("Coup"), "the Chronos export holds Dun's events and not the private Coup")
	var pm: Dictionary = _bridge.world_gen.timeline_items({"kinds": PackedStringArray(["Imported"])})
	var priv := (pm["items"] as Array).filter(func(i): return bool(i["private"]))
	_ok(priv.size() == 1, "the private imported row is flagged private in the model")
	_bridge.world_gen.timeline_decide_undo(String(pimp["undo"]))
	_bridge.world_gen.timeline_decide_undo(String(imp["undo"]))
	cnts = _bridge.world_gen.timeline_imported_counts()
	_ok(int(cnts["imported"]) == 0, "undo removes the imported rows")
	for n in before:
		_ok(_read(n) == before[n], "%s is still byte-identical" % n)
	await _shot("done")
	print("== %d checks, %d failed ==" % [_checks, _fails])
	get_tree().quit(1 if _fails > 0 else 0)
