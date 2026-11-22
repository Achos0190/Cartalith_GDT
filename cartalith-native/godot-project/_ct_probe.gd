extends Node
## Chronos timeline probe (Ruling GD/GE; CT-2, CT-4, CT-5, CT-8): the strip's Items row, filters, the passive
## badge, the prompt and an import, inside the real shell, over a scratch vault in `user://`.
##
##   godot --path . --resolution 1600x900 _ct_probe.tscn -- --shots <dir>
##   godot --path . --resolution 1700x1000 _ct_probe.tscn -- --force-touch --density=1.0 --shots <dir>
##   godot --path . --resolution 450x975 _ct_probe.tscn -- --force-touch --density 1.094 --shots <dir>
##
## **`--density` is a USER argument (after `--`)**, read by `DccShell._density()`. The first tablet run put
## `--density=1.0` before the scene, where the engine swallows it: the shell then classified a 1700x1000
## touch window by the legacy aspect rule (1000/1700 = 0.588 < 0.6) and booted the PHONE shell. The phone
## run's `--density 1.094` before the scene was worse: the engine took `1.094` for the scene to run, so the
## app booted instead of the probe and nothing printed (the "hang"). The probe now takes the flag after
## `--`, and asserts the form it booted is the form the arguments ask for, so a mis-launched run fails
## instead of passing on the wrong shell.
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
var _density := 0.0
var _expect := ""
var _touch := false
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
	var sc := c.get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(c)
		await _settle(3)
	var at := c.get_global_rect().get_center()
	# a control inside an embedded sub-window (the prompt) reports window-local coordinates
	var w := c.get_window()
	if w != get_tree().root and w.is_embedded():
		at += Vector2(w.position)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(8)

func _click_at(c: Control, local: Vector2) -> void:
	var at := c.get_global_transform() * local
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(6)

func _find_btn(root: Node, name_: String) -> Button:
	for b in root.find_children(name_, "Button", true, false):
		if (b as Control).is_visible_in_tree():
			return b
	return null

## The strip is rebuilt when the selection changes, so a node held across a selection is freed.
func _cur_node() -> Node:
	return _items_node() if not _app.is_phone() else _app.find_child("ChronosItems", true, false)

func _items_node() -> Node:
	return _app.timeline_row.find_child("ChronosItems", true, false)

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		if args[k] == "--force-touch":
			_touch = true
		elif args[k] == "--density" and k + 1 < args.size():
			k += 1
			_density = float(args[k])
		elif String(args[k]).begins_with("--density="):
			_density = float(String(args[k]).substr(10))
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
	var win := get_window().size
	# EG: with a known density a touch window under 900 dp is a phone; without one the legacy aspect rule
	# (`DccTheme.PHONE_ASPECT_MAX` 0.6) decides, which is how an engine-arg `--density` ran as a phone
	var short_dp := minf(win.x, win.y) / _density if _density > 0.0 else 0.0
	var narrow := float(minf(win.x, win.y)) / float(maxf(win.x, win.y)) < 0.6
	_expect = "desktop" if not _touch else ("phone" if ((_density > 0.0 and short_dp < 900.0) or (_density <= 0.0 and narrow)) else "tablet")
	_ok(_mode == _expect, "booted the %s shell as asked (got %s at %dx%d density %s)" % [_expect, _mode, win.x, win.y, str(_density)])
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
	var st0: Array = _bridge.settlements()
	var f0 := int((st0[0] as Dictionary).get("founded", 0))
	_ok((st0[0] as Dictionary).has("founded"), "settlement 0 has a founding year (%d)" % f0)
	_write("Orsa.md", "---
founded: %d
---
# Orsa
" % (f0 + 7))
	_write("Roster.md", "| Year | Event |\n|---|---|\n| 100 | a |\n| 200 | b |\n| 300 | c |\n")
	var before := {}
	for n in ["Kelv.md", "Dun.md", "dm/Plot.md", "Roster.md", "Orsa.md"]:
		before[n] = _read(n)
	var con: Dictionary = _bridge.vault_connect(ProjectSettings.globalize_path(DIR), "CT probe vault")
	_ok(bool(con.get("ok", false)), "the scratch vault connects")
	var st: Array = _bridge.settlements()
	var tid := int((st[0] as Dictionary)["tid"])
	var att: Dictionary = _bridge.vault_attach("settlement", tid, String((st[0] as Dictionary)["name"]), "Kelv.md", "")
	_ok(bool(att.get("ok", false)), "Kelv.md is linked to settlement %d" % tid)
	Items.invalidate_scan()

	# a saved journey on a real route (the spans lane's journey bar) and two recorded years (ownership spans)
	var towns: Array = _bridge.settlements()
	var gs: Vector2 = _bridge.grid_size()
	var presets: Array = _bridge.world_gen.tl_list("preset")
	var preset_id := String((presets[0] as Dictionary).get("id", "")) if not presets.is_empty() else ""
	var pairs: Array = []
	for i in towns.size():
		for j in range(i + 1, towns.size()):
			var dd := Vector2(towns[i]["x"], towns[i]["y"]).distance_to(Vector2(towns[j]["x"], towns[j]["y"]))
			if dd < gs.x * 0.45:
				pairs.append([dd, i, j])
	pairs.sort_custom(func(u, v): return u[0] > v[0])
	var jid := -1
	for kk in mini(8, pairs.size()):
		var ta: Dictionary = towns[pairs[kk][1]]
		var tb: Dictionary = towns[pairs[kk][2]]
		_bridge.route_begin("mixed")
		_bridge.route_append_stop(float(ta["x"]), float(ta["y"]))
		_bridge.route_append_stop(float(tb["x"]), float(tb["y"]))
		var ridx: int = _bridge.route_commit()
		jid = _bridge.journey_save("Probe caravan", preset_id, ridx, 40)
		var planned := false
		for jd in _bridge.journey_positions():
			if int(jd.get("id", -1)) == jid and jd.has("total_days") and float(jd["total_days"]) >= 3.0:
				planned = true
		if planned:
			break
		_bridge.journey_delete(jid)
		jid = -1
	_ok(jid >= 0, "a journey with a planned length is saved")
	_bridge.civ_add_year(0)
	_bridge.civ_add_year(50)

	print("== A. the scan, in Rust, then the strip ==")
	var sc: Dictionary = _bridge.world_gen.timelines_scan(0, 200)
	var cands: Array = sc["candidates"]
	_ok(cands.size() == 5, "the scan found 5 timelines (got %d)" % cands.size())
	_ok(int(sc["badge"]) == 5 and int(sc["next"]) == -1, "badge 5 and the whole list covered")
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
	# the phone strip is built before any world, so its scan is the 5 s poll for a vault (`_start_scan`)
	var waited := 0
	while _find_btn(_cur_node(), "TimelinesBadge") == null and waited < 12:
		await get_tree().create_timer(1.0).timeout
		waited += 1
	var badge := _find_btn(_cur_node(), "TimelinesBadge")
	_ok(badge != null and badge.text.begins_with("! 5 timelines"), "the passive badge reads '%s'" % ("" if badge == null else badge.text))
	_ok(_app.find_child("ChronosPrompt", true, false) == null, "no prompt opened by itself")
	if _app.is_phone():
		await _click(_find_btn(_cur_node(), "ItemsToggle"))
	await _shot("strip")

	print("== B. rows equal the model; a click lands the year ==")
	var model: Dictionary = _bridge.world_gen.timeline_items({})
	var mitems: Array = model["items"]
	var total := mitems.size()
	var n_found := mitems.filter(func(i): return String(i["source"]) == "Founding").size()
	var list: Node = _cur_node().find_child("ChronosItemsList", true, false)
	var rows := list.get_children().filter(func(c): return c is Button)
	_ok(n_found > Items.FOUNDING_GROUP_MAX, "the world has %d Founding rows (above the group threshold %d)" % [n_found, Items.FOUNDING_GROUP_MAX])
	var cluster: Button = _find_btn(_cur_node(), "FoundingCluster")
	_ok(cluster != null and cluster.text.begins_with("Founding x%d " % n_found), "one cluster chip stands for all %d founding rows ('%s')" % [n_found, "" if cluster == null else cluster.text])
	_ok(rows.filter(func(c): return String(c.name).begins_with("Item_founded")).is_empty(), "no Founding row is drawn while the cluster is shut")
	var others := total - n_found
	_ok(rows.size() == mini(others, Items.ROWS_MAX) + 1, "row count %d = the %d other rows (cap %d) + the cluster" % [rows.size(), others, Items.ROWS_MAX])
	await _shot("strip_clustered")
	await _click(cluster)
	list = _cur_node().find_child("ChronosItemsList", true, false)
	rows = list.get_children().filter(func(c): return c is Button)
	var f_rows := rows.filter(func(c): return String(c.name).begins_with("Item_founded"))
	_ok(f_rows.size() > 0 and f_rows.size() <= Items.ROWS_MAX, "pressing the cluster lists founding rows (%d, capped at %d)" % [f_rows.size(), Items.ROWS_MAX])
	await _click(_find_btn(_cur_node(), "FoundingCluster"))
	list = _cur_node().find_child("ChronosItemsList", true, false)
	_ok(list.get_children().filter(func(c): return String(c.name).begins_with("Item_founded")).is_empty(), "pressing it again shuts them")
	# selection follows: a selected settlement's own founding row is a row, with no cluster
	_app.selection = {"kind": "settlement", "id": 0}
	await _settle(10)
	node = _cur_node()
	node._refresh()
	await _settle(6)
	list = _cur_node().find_child("ChronosItemsList", true, false)
	var own := list.get_children().filter(func(c): return String(c.name) == "Item_founded_settlement_%d" % tid)
	_ok(own.size() == 1 and _find_btn(_cur_node(), "FoundingCluster") == null, "selected settlement %d: its own founding row is shown and there is no cluster" % tid)
	_app.selection = {}
	await _settle(10)
	node = _cur_node()
	node._refresh()
	await _settle(4)

	print("== B2. the spans lane: ownership, conflict and journey bars ==")
	var jm: Dictionary = _bridge.world_gen.timeline_items({"kinds": PackedStringArray(["Journey"])})
	var jitems: Array = jm["items"]
	_ok(jitems.size() == 1 and jitems[0].has("end_year") and String(jitems[0]["precision"]) == "range", "the saved journey is a range in the model (planner calendar_days), not a point: %s" % str(jitems))
	var spans: Control = _cur_node().find_child("ChronosSpans", true, false)
	_ok(spans != null and spans.visible and spans.bars.size() >= 1, "the lane is drawn with the journey bar (%d bars)" % (0 if spans == null else spans.bars.size()))
	var jb: Array = [] if spans == null else spans.bars.filter(func(b): return String(b["item"]["source"]) == "Journey")
	_ok(jb.size() == 1 and (jb[0]["rect"] as Rect2).size.x >= 4.0, "the journey bar is at least 4 px wide even though it lasts days on a years-wide axis (w=%s)" % ("" if jb.is_empty() else str((jb[0]["rect"] as Rect2).size.x)))
	var note_lbl: Label = _cur_node().find_child("ChronosSpansNote", true, false)
	var own_all: Dictionary = _bridge.world_gen.timeline_items({"kinds": PackedStringArray(["Ownership"])})
	var n_own := (own_all["items"] as Array).size()
	_ok(n_own > Items.SPANS_MAX and note_lbl.visible and note_lbl.text.contains("ownership"), "with no selection the %d ownership spans are held back and the lane says so ('%s')" % [n_own, note_lbl.text])
	_ok(spans.bars.filter(func(b): return String(b["item"]["source"]) == "Ownership").is_empty(), "...and none of them is drawn")
	_app.selection = {"kind": "settlement", "id": 0}
	await _settle(10)
	node = _cur_node()
	node._refresh()
	await _settle(6)
	spans = _cur_node().find_child("ChronosSpans", true, false)
	var ob: Array = spans.bars.filter(func(b): return String(b["item"]["source"]) == "Ownership")
	_ok(ob.size() >= 1 and String(ob[0]["item"]["key"]).begins_with("span:settlement:%d:" % tid), "selecting settlement %d draws its ownership span (%d)" % [tid, ob.size()])
	await _shot("spans")
	var jbar: Dictionary = {}
	for bb in spans.bars:
		if String(bb["item"]["source"]) == "Journey":
			jbar = bb
	if not jbar.is_empty():
		await _click_at(spans, (jbar["rect"] as Rect2).get_center())
		_ok(_app.tl_year() == int(jbar["item"]["start_year"]), "pressing the journey bar moves the year to its start (%d)" % _app.tl_year())
	_app.selection = {}
	await _settle(10)
	node = _cur_node()
	node._refresh()
	await _settle(4)
	await _click(_find_btn(_cur_node(), "Scope"))
	note_lbl = _cur_node().find_child("ChronosSpansNote", true, false)
	_ok(note_lbl.visible and note_lbl.text.begins_with("%d ownership spans - narrow" % n_own), "All items: %d ownership spans exceed the cap and the lane says so ('%s')" % [n_own, note_lbl.text])
	await _click(_find_btn(_cur_node(), "Scope"))

	# 217 founding rows crowd the 24 nearest the cursor out, so isolate the Note lane for the click
	await _click(_find_btn(_cur_node(), "Kind_Note"))
	list = _cur_node().find_child("ChronosItemsList", true, false)
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
	await _click(_find_btn(_cur_node(), "Kind_Note"))
	list = _cur_node().find_child("ChronosItemsList", true, false)
	rows = list.get_children().filter(func(c): return c is Button)
	_ok(_app.tl_year() == 1204, "the click moved the year to 1204 (got %d)" % _app.tl_year())
	_ok(int(_app.tl_day()) == 72, "...and the day to 72 = 14 March (got %d)" % int(_app.tl_day()))
	_ok(changes[0] == 1, "a dated row click fires timeline_changed exactly once (got %d)" % changes[0])

	print("== C. a kind filter hides rows and says how many ==")
	var chip := _find_btn(_cur_node(), "Kind_Conflict")
	await _click(chip)
	list = _cur_node().find_child("ChronosItemsList", true, false)
	var left := list.get_children().filter(func(c): return c is Button).size()
	var cnt: Label = _cur_node().find_child("ChronosItemsCount", true, false)
	_ok(left == 0 and cnt.text.contains("hidden by filters"), "Conflict only: %d rows and the count line says '%s'" % [left, cnt.text])
	await _click(_find_btn(_cur_node(), "Kind_Conflict"))
	list = _cur_node().find_child("ChronosItemsList", true, false)
	_ok(list.get_children().filter(func(c): return c is Button).size() == rows.size(), "clearing the filter restores the rows")
	await _shot("filtered")
	# every control of the strip's Items body is at least 44 dp tall in a touch form (phone: 44 x the density)
	if _touch:
		var floor_px := floorf(44.0 * (_density if _density > 0.0 else 1.0)) if _mode == "phone" else 32.0
		var small: Array = []
		var nbtn := 0
		for bt in _cur_node().find_children("*", "Button", true, false):
			if (bt as Control).is_visible_in_tree():
				nbtn += 1
				if (bt as Control).size.y < floor_px - 0.5:
					small.append("%s=%.0f" % [bt.name, (bt as Control).size.y])
		_ok(nbtn >= 12 and small.is_empty(), "all %d visible chips are >= %.0f px tall on %s (small: %s)" % [nbtn, floor_px, _mode, str(small)])

	print("== D. the prompt: defaults, and no note ever changes ==")
	await _click(_find_btn(_cur_node(), "TimelinesBadge"))
	await _settle(10)
	var pr: Node = _app.find_child("ChronosPrompt", true, false)
	_ok(pr != null and pr.visible, "the badge opens the prompt")
	var cards := pr.find_children("Card_*", "VBoxContainer", true, false)
	_ok(cards.size() == 5, "one card per timeline (got %d)" % cards.size())
	for c in cands:
		var want := "link" if (bool(c["linked"]) and not c.has("private")) else "keep"
		_ok(String(pr._choice[pr._key(c)]) == want, "%s defaults to %s" % [String(c["rel"]), want])
	var vp := get_viewport().get_visible_rect()
	var done_rect: Rect2 = (pr.find_child("Done", true, false) as Control).get_global_rect()
	if pr.is_embedded():
		done_rect.position += Vector2(pr.position)
	_ok(vp.encloses(done_rect) and vp.encloses(Rect2(Vector2(pr.position), Vector2(pr.size))), "the prompt and its Done button are inside the window (Done %s in %s, prompt %s)" % [str(done_rect), str(vp), str(Rect2(Vector2(pr.position), Vector2(pr.size)))])
	if _touch:
		var pfloor := floorf(44.0 * _density) if _mode == "phone" else 32.0
		var psmall: Array = []
		var pn := 0
		for bt in pr.find_children("*", "Button", true, false):
			if (bt as Control).is_visible_in_tree() and not (bt is OptionButton):
				pn += 1
				if (bt as Control).size.y < pfloor - 0.5:
					psmall.append("%s %s in %s text=%s =%.0f" % [bt.name, bt.get_class(), bt.get_parent().name, (bt as Button).text, (bt as Control).size.y])
		_ok(pn >= 10 and psmall.is_empty(), "all %d visible prompt buttons (Done included) are >= %.0f px tall (small: %s)" % [pn, pfloor, str(psmall)])
	await _shot("prompt")
	print("== D2. CT-7: the founding comparison and a re-sync, by real clicks ==")
	var orsa_c: Dictionary = {}
	var dun_c: Dictionary = {}
	for c in cands:
		if String(c["rel"]) == "Orsa.md":
			orsa_c = c
		if String(c["rel"]) == "Dun.md":
			dun_c = c
	var orsa_card: Node = pr.find_child("Card_" + pr._key(orsa_c).md5_text().substr(0, 8), true, false)
	_ok(orsa_card != null, "Orsa.md has a card")
	await _click(orsa_card.find_child("FoundCheck", true, false))
	var otext: Label = orsa_card.find_child("ExtrasText", true, false)
	_ok(otext.text.contains("the note says %d, the settlement was founded %d" % [f0 + 7, f0]), "the comparison shows both years ('%s')" % otext.text)
	_ok(int((_bridge.settlements()[0] as Dictionary)["founded"]) == f0, "...and changed nothing yet")
	await _click(orsa_card.find_child("SetFounded", true, false))
	_ok(int((_bridge.settlements()[0] as Dictionary)["founded"]) == f0 + 7, "Set the founding year: the settlement is now founded %d" % (f0 + 7))
	await _click(orsa_card.find_child("UndoFounded", true, false))
	_ok(int((_bridge.settlements()[0] as Dictionary)["founded"]) == f0, "Undo restores %d" % f0)
	await _shot("prompt_founding")
	var dun_card: Node = pr.find_child("Card_" + pr._key(dun_c).md5_text().substr(0, 8), true, false)
	await _click(dun_card.find_child("Choice_import", true, false))
	await _click(dun_card.find_child("Apply", true, false))
	_ok(int(_bridge.world_gen.timeline_imported_counts()["imported"]) == 2, "Dun.md imported by the card: 2 rows")
	_write("Dun.md", "## History
- 310: Walls
- 388: Fire
- 400: Plague
")
	before["Dun.md"] = _read("Dun.md")
	await _click(dun_card.find_child("Resync", true, false))
	var dtext: Label = dun_card.find_child("ExtrasText", true, false)
	_ok(dtext.text.contains("1 added, 0 removed") and dtext.text.contains("0 changed"), "re-sync plan: 1 added, 0 removed, 0 changed ('%s')" % dtext.text)
	_ok(int(_bridge.world_gen.timeline_imported_counts()["imported"]) == 2, "...and applied nothing yet")
	await _click(dun_card.find_child("ApplyResync", true, false))
	_ok(int(_bridge.world_gen.timeline_imported_counts()["imported"]) == 3, "Apply re-sync: the new line is imported (3 rows)")
	_ok(_read("Dun.md") == before["Dun.md"], "the re-sync did not touch the note")
	await _click(dun_card.find_child("UndoResync", true, false))
	_ok(int(_bridge.world_gen.timeline_imported_counts()["imported"]) == 2, "Undo re-sync: back to 2 rows")
	await _click(dun_card.find_child("Undo", true, false))
	_ok(int(_bridge.world_gen.timeline_imported_counts()["imported"]) == 0, "the card's Undo removes the import (0 rows)")
	_write("Dun.md", "## History
- 310: Walls
- 388: Fire
")
	before["Dun.md"] = _read("Dun.md")
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
	_write("Html.md", "<span class='ob-timelines' data-start-date='-50-03-00-00' data-title='Ash fall'>x</span>
<div class='ob-timelines' data-start-date='10-00-00-00' data-end-date='20-00-00-00'>Drought</div>
")
	var hb := _read("Html.md")
	var hs: Dictionary = _bridge.world_gen.timelines_scan(0, 200)
	var hc := (hs["candidates"] as Array).filter(func(c): return String(c["rel"]) == "Html.md")
	_ok(hc.size() == 1 and String(hc[0]["form"]) == "html_spans" and int(hc[0]["count"]) == 2, "CT-9: the Timelines plugin's spans are found as one html_spans timeline of 2 events (%s)" % str(hc))
	_ok(_read("Html.md") == hb, "...and the note is untouched")
	DirAccess.remove_absolute(ProjectSettings.globalize_path(DIR.path_join("Html.md")))
	await _shot("done")
	print("== %d checks, %d failed ==" % [_checks, _fails])
	get_tree().quit(1 if _fails > 0 else 0)
