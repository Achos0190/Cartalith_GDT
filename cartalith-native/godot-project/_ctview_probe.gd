extends Node
## CT view probe (2026-11-22): the strip's range, once-per-entry rows, and the count over a synthetic vault.
## Same launch forms and exit codes as `_ct_probe.gd`.

var _fails := 0
var _checks := 0
var _shots := ""
var _mode := "desktop"
var _density := 0.0
var _expect := ""
var _touch := false
var _app: Node
var _bridge
const DIR := "user://_ctview_vault"
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
		var p := _shots.path_join("ctview_%s_%s.png" % [_mode, tag])
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


	# CT view issues 1-4 on a synthetic vault shaped like the owner's: a Chronos block of 21 events from
	# 3000 to 4769 (17 points, 4 ranges), a 4-column table 4295-4649, the note linked to a settlement.
	var lines := "```chronos\n"
	var pts := [3000, 3100, 3250, 3400, 3550, 3700, 3850, 4000, 4150, 4300, 4450, 4600, 4700, 4720, 4740, 4755, 4769]
	for i in pts.size():
		lines += "* [%d] Point %d\n" % [pts[i], i]
	lines += "- [3000~4000] Long reign\n- [3300~3600] Plague\n- [4100~4400] Siege\n@ [4500~4769] Late age\n```\n"
	lines += "\n## Ledger\n| Year | Event | Place | Note |\n|---|---|---|---|\n"
	for y in [4295, 4350, 4420, 4533, 4649]:
		lines += "| %d | Ledger %d | Kelv | x |\n" % [y, y]
	_write("Hist.md", lines)
	var before := _read("Hist.md")
	var con: Dictionary = _bridge.vault_connect(ProjectSettings.globalize_path(DIR), "CT view vault")
	_ok(bool(con.get("ok", false)), "the scratch vault connects")
	var st: Array = _bridge.settlements()
	var tid := int((st[0] as Dictionary)["tid"])
	var att: Dictionary = _bridge.vault_attach("settlement", tid, String((st[0] as Dictionary)["name"]), "Hist.md", "")
	_ok(bool(att.get("ok", false)), "Hist.md is linked to settlement %d" % tid)
	Items.invalidate_scan()
	_app.select_domain("civilization")
	await _settle(8)
	_app._tl_expanded = true
	_app._fill_timeline_strip()
	if _app.is_phone():
		_app.set_phone_sim_strip_open(true)
	await _settle(30)
	var node := _cur_node()
	_ok(node != null, "the Items row is on the strip")
	if _app.is_phone():
		await _click(_find_btn(_cur_node(), "ItemsToggle"))
	_app.selection = {"kind": "settlement", "id": 0}
	await _settle(10)
	_cur_node()._refresh()
	await _settle(8)

	print("== 1. the track covers the note's events ==")
	var rng: Dictionary = _app.tl_range()
	_ok(int(rng["min"]) <= 3000 and int(rng["min"]) > 0 and int(rng["max"]) >= 4769, "tl_range covers 3000..4769 and is not dragged back by the 200+ founding years (got %d..%d)" % [int(rng["min"]), int(rng["max"])])
	_app.tl_set_year(4769, true)
	await _settle(6)
	_ok(is_equal_approx(_app._tl_frac(4769), 1.0) and _app._tl_frac(4600) < 0.999 and _app._tl_frac(3000) > 0.0 - 0.0001 and _app._tl_frac(3850) > 0.0 and _app._tl_frac(3850) < 0.999, "the playhead is at the end only for the last year; 3850 sits mid-track (%.3f)" % _app._tl_frac(3850))
	_app.tl_set_year(3000, true)
	await _settle(4)
	var pk := _find_row(_cur_node(), "Point 8")
	_ok(pk != null, "the Point 8 row (4150) is listed")
	if pk != null:
		await _click(pk)
		_ok(_app.tl_year() == 4150 and _app._tl_frac(4150) > 0.0 and _app._tl_frac(4150) < 0.999, "clicking it moves the year to 4150 and the playhead is mid-track (%.3f)" % _app._tl_frac(4150))
	await _shot("track_linked")

	print("== 2. each entry once, in its real form ==")
	var evn: Node = _app.timeline_row.find_child("TimelineEvents", true, false)  # the phone strip has no EVENTS row
	_ok(evn == null or evn.find_children("*", "Button", true, false).filter(func(b): return (b as Button).text.contains("Long reign")).is_empty(), "the EVENTS row does not repeat the note's entries")
	for nm in ["Long reign", "Plague", "Siege", "Late age", "Point 3"]:
		var hits := _rows_with(_cur_node(), nm)
		_ok(hits.size() == 1, "'%s' is one row (got %d)" % [nm, hits.size()])
	var lr := _rows_with(_cur_node(), "Long reign")
	_ok(lr.size() == 1 and (lr[0] as Button).text.begins_with("3000~4000"), "Long reign reads 3000~4000 ('%s')" % ("" if lr.is_empty() else (lr[0] as Button).text))

	print("== 3/4. linked, then imported: one count, one row each ==")
	var cnt_note: Dictionary = _bridge.world_gen.timeline_items({"kinds": PackedStringArray(["Note", "Imported"])})
	_ok((cnt_note["items"] as Array).size() == 21, "linked only: 21 note rows (got %d)" % (cnt_note["items"] as Array).size())
	var sc: Dictionary = _bridge.world_gen.timelines_scan(0, 200)
	var chr: Dictionary = {}
	for c in sc["candidates"]:
		if String(c["rel"]) == "Hist.md" and String(c["form"]) == "chronos":
			chr = c
	_ok(not chr.is_empty(), "the scan found the Chronos block of Hist.md")
	var imp: Dictionary = _bridge.world_gen.timeline_decide("Hist.md", String(chr["form"]), int(chr["ordinal"]), "import", "settlement", tid, 0)
	_ok(bool(imp["ok"]) and (imp["added"] as PackedInt64Array).size() == 21, "imported 21 rows")
	var both: Dictionary = _bridge.world_gen.timeline_items({"kinds": PackedStringArray(["Note", "Imported"])})
	_ok((both["items"] as Array).size() == 21, "linked AND imported: still 21 (was 42), got %d" % (both["items"] as Array).size())
	_ok((both["items"] as Array).all(func(i): return bool(i["also_linked"])), "every survivor says it is also linked")
	_app.emit_signal("timeline_records_changed")
	await _settle(10)
	_cur_node()._refresh()
	await _settle(8)
	var btn := _find_btn(_cur_node(), "ItemsToggle")
	var total := int(_bridge.world_gen.timeline_items({})["total"])
	_ok(btn.text.begins_with("Items (") and btn.text.ends_with(" of %d)" % total), "the Items button names shown of total ('%s', total %d)" % [btn.text, total])
	var cl := _cur_node().find_child("ChronosItemsCount", true, false) as Label
	_ok(cl.text.contains("hidden by filters (the selection (settlement"), "the count line names the filter that hid the rest ('%s')" % cl.text)
	var lr2 := _rows_with(_cur_node(), "Long reign")
	_ok(lr2.size() == 1 and (lr2[0] as Button).text.contains("[note + imported]"), "Long reign is one row carrying both badges ('%s')" % ("" if lr2.is_empty() else (lr2[0] as Button).text))
	await _click(_find_btn(_cur_node(), "Scope"))
	btn = _find_btn(_cur_node(), "ItemsToggle")
	_ok(btn.text == "Items (%d)" % total, "'All items' shows every row and the button says so ('%s')" % btn.text)
	await _shot("both")
	_ok(_read("Hist.md") == before, "the note is byte-identical after the import (imports never edit a note)")
	print("== %d checks, %d failed ==" % [_checks, _fails])
	get_tree().quit(1 if _fails > 0 else 0)

func _rows_with(root: Node, text: String) -> Array:
	var out: Array = []
	for b in root.find_children("Item_*", "Button", true, false):
		if (b as Button).text.contains(text):
			out.append(b)
	return out

func _find_row(root: Node, text: String) -> Button:
	var r := _rows_with(root, text)
	return null if r.is_empty() else r[0]
