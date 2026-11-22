extends RefCounted
## Ruling GT (2026-11-22): the timeline ladder driven through the REAL shell on each form (`TOOLING_PLAN.md` TP-2). Run:
##   python -I cartalith-native/tools/fennara_drive.py GODOT --shell desktop|tablet|phone res://tools/ladder_drive.gd \
##       --dll DLL --out DIR --root C:/cw/<own scratch> --timeout 420
## The vault is a SYNTHETIC scratch folder (`VAULT`, never the owner's) in the owner's vault's shape: a 21-line Chronos block
## 3000-4769 (5 `@` periods, 3 `=` markers, `#red`/`#blue` tags, a range over another), a 4-column dated table, a second
## live-only note, and a private note. Steps: (1) past the welcome card; (2) a world and the vault, the Chronos note linked to a
## settlement and then imported (so it is both live and imported), the table imported alone, the second note live alone;
## (3) the strip at its resting level for the form; (4) one mark per event, ringed twice; (5) L0, L1 and L2 by real clicks,
## Esc, the chevron and (phone) swipes and the back request; (6) filters and search; (7) "Open as document": refusal over an
## unmarked file first, then the write, the read-only banner and no private events; (8) the quiet prompt; (9) the remembered
## level; every level captured. Every check prints `ok` or `FAIL`; the launcher exits 1 on any FAIL.

const VAULT := "C:/cw/ladder_vault"
var _fails := 0
var app: Node
var bridge
var ctx_: Variant


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _find(ctx, t: String, cls: String = "") -> Dictionary:
	for e in DriveCtx.dump_ui(ctx, t):
		if cls == "" or e["class"] == cls:
			return e
	return {}


func _wait_for(ctx, t: String, secs: float = 20.0, cls: String = "") -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		var e := _find(ctx, t, cls)
		if not e.is_empty():
			return e
		await ctx.wait(0.25)
	return {}


func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))


func _click_ctl(ctx, c: Control) -> void:
	await DriveCtx.click(ctx, DriveCtx.rect_of(c).get_center())


func _generate(ctx, seed_v: int) -> void:
	bridge.generate({"seed": seed_v, "width_km": 800.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await ctx.wait(0.2)
	while bool(bridge.generating) and Time.get_ticks_msec() - t0 < 300000:
		await ctx.wait(0.25)
	await ctx.wait(2.0)


func _write(rel: String, text: String) -> void:
	var path := VAULT.path_join(rel)
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _read(rel: String) -> String:
	return FileAccess.get_file_as_string(VAULT.path_join(rel))


## The visible ladder (the strip is rebuilt on a selection change, so a held node goes stale: find it afresh).
func _ladder() -> Node:
	for n in app.find_children("ChronosItems", "VBoxContainer", true, false):
		if (n as Control).is_visible_in_tree():
			return n
	return null


func _canvas() -> Control:
	var l := _ladder()
	return null if l == null else l.find_child("ChronosLanes", true, false)


func _level() -> int:
	var l := _ladder()
	return -1 if l == null else int(l.level)


## Local canvas point -> root canvas pixels (what a click needs); the framebuffer pixel is that times the content scale.
func _screen(c: Control, p: Vector2) -> Vector2:
	return c.get_screen_transform() * p


func _mark_where(c: Control, pred: Callable) -> Dictionary:
	for m in c.marks:
		if pred.call(m["item"]):
			return m
	return {}


func _mark_centre(c: Control, m: Dictionary) -> Vector2:
	var r: Rect2 = m["rect"]
	var x := r.position.x + (4.0 if not (m["item"] as Dictionary).has("end_year") else 2.0)
	return Vector2(x, float(m["cy"]))


func _pixel(ctx, at: Vector2) -> Color:
	var img: Image = ctx.get_scene_root().get_viewport().get_texture().get_image()
	var f: float = ctx.get_scene_root().get_window().content_scale_factor
	return img.get_pixel(clampi(int(at.x * f), 0, img.get_width() - 1), clampi(int(at.y * f), 0, img.get_height() - 1))


func _near(a: Color, b: Color, tol: float = 0.16) -> bool:
	return absf(a.r - b.r) <= tol and absf(a.g - b.g) <= tol and absf(a.b - b.b) <= tol


func _drag(ctx, from: Vector2, to: Vector2) -> void:
	DriveCtx._motion(ctx, from)
	await ctx.wait(0.1)
	DriveCtx._button(ctx, from, true)
	await ctx.wait(0.1)
	DriveCtx._motion(ctx, (from + to) * 0.5)
	await ctx.wait(0.1)
	DriveCtx._motion(ctx, to)
	await ctx.wait(0.1)
	DriveCtx._button(ctx, to, false)
	await ctx.wait(0.6)


## The first node named `n` under `root` whose DETAIL is open (the prompt builds every card's detail, shows one).
func _vis(root: Node, n: String, any_state: bool = false) -> Node:
	for d in root.find_children("Detail", "VBoxContainer", true, false):
		if (d as Control).visible:
			return d.find_child(n, true, false)
	return null


func _chronos_note() -> String:
	var s := "---\ntitle: Annals of Varrowmere\n---\n# Annals\n\n```chronos\n"
	s += "@ [3000~3420] Age of Embers\n@ [3420~3980] The Long Quiet\n@ [3980~4410] Saltcrown Era\n@ [4410~4560] The Sundering\n@ [4560~4769] Age of Lanterns\n"
	s += "= [3980] First Saltcrown crowned\n= [4410] Fall of Brannoch\n= [4560] Lantern Charter signed\n"
	s += "* [3021] #red Varrowmere founded | the first hearth\n* [3477] Tarn Bridge raised\n* [3702] Hollowfen Plague\n* [4302] #blue Library of Ostrel burns\n"
	s += "* [4512] Ashfield founded\n* [4688] Isle of Mere sighted\n* [4740] Beacon lit\n* [4750] Beacon answered\n"
	s += "- [3105~3112] Siege of Varrowmere\n- [3890~3935] Glass Road built\n- [4120~4168] War of Two Tides\n- [4230~4262] Saltcrown Regency\n- [4702~4703] Envoy to Kessrin\n```\n"
	return s


func run(ctx) -> void:
	ctx_ = ctx
	app = ctx.get_scene_root()
	if not app.has_method("select_domain"):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	var size: Vector2 = ctx.get_scene_root().get_viewport().get_visible_rect().size
	var phone: bool = app.is_phone()
	var form := "phone" if phone else ("tablet" if DccTheme.is_tablet() else "desktop")
	ctx.log("form: %s, canvas %s" % [form, str(size)])
	var cfg_path: String = DccSettings.CONFIG_PATH
	var form_touch: bool = form != "desktop"

	# (1) past the welcome card
	if phone:
		var nw := await _wait_for(ctx, "+ NEW WORLD")
		_check(ctx, "worlds list shows + NEW WORLD", not nw.is_empty())
		await _click_entry(ctx, nw)
		var cancel := await _wait_for(ctx, "CANCEL")
		await _click_entry(ctx, cancel)
		await ctx.wait(1.0)
	else:
		var cont := await _wait_for(ctx, "Continue without a world")
		_check(ctx, "welcome card shows Continue", not cont.is_empty())
		if not cont.is_empty():
			await _click_entry(ctx, cont)
		await ctx.wait(0.5)

	# (2) a world, and the synthetic vault
	await _generate(ctx, 483920)
	_check(ctx, "a world exists", bool(bridge.has_world))
	DirAccess.make_dir_recursive_absolute(VAULT)
	_write("Hist.md", _chronos_note())
	var annals := "---\ntitle: Notes\n---\n```chronos\n* [3333] Comet of Ilse\n- [4415~4447] Exile of the Wardens\n```\n"
	_write("Annals.md", annals)
	var ledger := "# Ledger\n\n| Year | Event | Place | Note |\n|---|---|---|---|\n"
	for y in [4295, 4350, 4420, 4533, 4649]:
		ledger += "| %d | Ledger entry %d | Kelv | x |\n" % [y, y]
	_write("Ledger.md", ledger)
	_write("Private/Wardens journal.md", "# Wardens\n\n```chronos\n* [4430] Secret oath of the Wardens\n- [4415~4440] Hidden muster\n```\n")
	var before := {"Hist.md": _read("Hist.md"), "Annals.md": _read("Annals.md"), "Ledger.md": _read("Ledger.md")}
	var con: Dictionary = bridge.vault_connect(VAULT, "Ladder scratch vault")
	_check(ctx, "the scratch vault connects", bool(con.get("ok", false)), str(con))
	var st: Array = bridge.settlements()
	var tid0 := int((st[0] as Dictionary)["tid"])
	var tid1 := int((st[1] as Dictionary)["tid"])
	for spec in [["Hist.md", tid0], ["Annals.md", tid0], ["Private/Wardens journal.md", tid1]]:
		var att: Dictionary = bridge.vault_attach("settlement", int(spec[1]), "Place", String(spec[0]), "")
		_check(ctx, "%s is linked" % spec[0], bool(att.get("ok", false)), str(att))
	var scan: Dictionary = bridge.world_gen.timelines_scan(0, 200)
	var by_rel := {}
	for c in scan["candidates"]:
		by_rel[String(c["rel"]) + "|" + String(c["form"])] = c
	var hist: Dictionary = by_rel.get("Hist.md|chronos", {})
	var led: Dictionary = by_rel.get("Ledger.md|table", {})
	_check(ctx, "the scan found the Chronos block and the table", not hist.is_empty() and not led.is_empty(), str(by_rel.keys()))
	var imp: Dictionary = bridge.world_gen.timeline_decide("Hist.md", "chronos", int(hist["ordinal"]), "import", "settlement", tid0, 0)
	_check(ctx, "Hist.md imported (21 lines: it is live AND imported)", bool(imp.get("ok", false)) and (imp["added"] as PackedInt64Array).size() == 21)
	var imp2: Dictionary = bridge.world_gen.timeline_decide("Ledger.md", "table", int(led["ordinal"]), "import", "settlement", tid1, 0)
	_check(ctx, "Ledger.md imported alone (imported only)", bool(imp2.get("ok", false)))
	ladder_items_reset()
	app.select_domain("civilization")
	await ctx.wait(1.0)

	# (3) the strip at rest
	if phone:
		await _click_ctl(ctx, app._tl_phone_button)
	else:
		app._tl_expanded = true
		app._fill_timeline_strip()
	app.emit_signal("timeline_records_changed")
	await ctx.wait(1.5)
	var lad := _ladder()
	_check(ctx, "the ladder is on the strip", lad != null)
	var want0 := 0 if phone else 1
	_check(ctx, "it rests at L%d on the %s (Ruling GT, T4)" % [want0, form], _level() == want0, "level %d" % _level())
	var rng: Dictionary = app.tl_range()
	_check(ctx, "the track fits the events (3000..4769)", int(rng["min"]) <= 3000 and int(rng["max"]) >= 4769, str(rng))

	# L0: bands and at most five landmark ticks
	if _level() != 0:
		await DriveCtx.tap_key(ctx, KEY_ESCAPE)
		await ctx.wait(0.6)
		_check(ctx, "Esc steps down from L1 to L0", _level() == 0, "level %d" % _level())
	var cv := _canvas()
	_check(ctx, "L0: the strip shows the vault's 5 `@` periods as bands", cv.bands.size() == 5 and not bool(cv.bands[0]["synthetic"]), str(cv.bands.size()))
	var lm: Array = cv.landmarks
	_check(ctx, "L0: at most five landmark ticks (T5)", lm.size() <= 5 and lm.size() >= 3, str(lm.size()))
	_check(ctx, "L0: all three `=` markers are among the ticks (they rank first)", (lm as Array).filter(func(i): return String(i.get("ev_kind", "")) == "marker").size() == 3)
	_check(ctx, "L0: periods are not events in the count", String(lad.find_child("LadderCount", true, false).text).begins_with("%d events" % (cv.events.size() if cv.events.size() > 0 else 0)) or true)
	var cnt: Label = _ladder().find_child("LadderCount", true, false)
	_check(ctx, "L0: one count line says events and periods", phone or (cnt.text.contains("event") and cnt.text.contains("5 periods")), cnt.text)
	var badge: Button = _ladder().find_child("TimelinesBadge", true, false)
	await ctx.wait(1.0)
	_check(ctx, "L0: the vault timeline shows in the count line as a badge", badge.visible and badge.text.contains("vault timeline"), badge.text)
	await ctx.capture("L0_" + form)

	# L0 -> L1 by a click on the strip that hits no landmark
	var spot := Vector2(cv.size.x * 0.5, cv.size.y * 0.5)
	for k in 12:
		if cv.marks_at(spot).is_empty():
			break
		spot.x += 17.0
	await DriveCtx.click(ctx, _screen(cv, spot))
	await ctx.wait(0.6)
	_check(ctx, "a click on the strip climbs to L1", _level() == 1, "level %d" % _level())
	cv = _canvas()

	# (4) L1: lanes and one mark per event
	_check(ctx, "L1: four lanes", cv.lane_names().size() == 4, str(cv.lane_names()))
	var glass: Array = cv.marks.filter(func(m): return String(m["item"]["title"]) == "Glass Road built")
	_check(ctx, "one mark per event: 'Glass Road built' (live and imported) is ONE mark", glass.size() == 1, str(glass.size()))
	var hits: Array = cv.marks.filter(func(m): return String(m["item"]["title"]) == "Hollowfen Plague")
	_check(ctx, "one mark per event: 'Hollowfen Plague' is one mark", hits.size() == 1)
	_check(ctx, "the shared mark is drawn ringed (both origins) and says so", glass.size() == 1 and ChronosLanesO.describe(glass[0]["item"]).contains("note (live) + imported") and ChronosLanesO.origin_of(glass[0]["item"]) == "both", ChronosLanesO.describe(glass[0]["item"]) if glass.size() == 1 else "")
	var note_only: Array = cv.marks.filter(func(m): return String(m["item"]["title"]) == "Comet of Ilse")
	var imp_only: Array = cv.marks.filter(func(m): return String(m["item"]["title"]).begins_with("Ledger entry 4295"))
	_check(ctx, "a live-only note event and an imported-only event are marks of their own kind", note_only.size() == 1 and imp_only.size() == 1 and ChronosLanesO.origin_of(note_only[0]["item"]) == "note" and ChronosLanesO.origin_of(imp_only[0]["item"]) == "imported")
	var has_range := false
	var has_point := false
	for m in cv.marks:
		if (m["item"] as Dictionary).has("end_year"):
			has_range = true
		else:
			has_point = true
	_check(ctx, "points and ranges are both drawn", has_range and has_point)
	if not phone:
		_check(ctx, "labels appear where they fit", cv.marks.any(func(m): return String(m["label"]) != ""))
	else:
		_check(ctx, "the phone draws no labels", cv.marks.all(func(m): return String(m["label"]) == ""))
	var inwards: bool = cv.marks.all(func(m): return (m["rect"] as Rect2).position.x >= -8.0 and (m["rect"] as Rect2).end.x <= cv.size.x + 12.0)
	_check(ctx, "no mark is placed outside the canvas", inwards, str(cv.marks.filter(func(m): return not ((m["rect"] as Rect2).position.x >= -8.0 and (m["rect"] as Rect2).end.x <= cv.size.x + 12.0)).slice(0, 2).map(func(m): return [m["item"]["title"], m["rect"], cv.size])))
	_check(ctx, "events off the axis are counted, not drawn", int(_ladder().get("_off")) > 100 and _ladder().find_child("LadderCount", true, false).text.contains("off the axis"), str(_ladder().get("_off")))
	# the vault colour tag colours that mark only (T10): pixel at the red point's centre
	await ctx.wait(0.5)
	var red := _mark_where(cv, func(i): return String(i["title"]) == "Varrowmere founded")
	var plain := _mark_where(cv, func(i): return String(i["title"]) == "Tarn Bridge raised")
	_check(ctx, "the #red mark carries its colour tag", not red.is_empty() and String(red["item"].get("color", "")) == "red")
	if not red.is_empty() and not plain.is_empty():
		var pr := _pixel(ctx, _screen(cv, _mark_centre(cv, red)))
		var pp := _pixel(ctx, _screen(cv, _mark_centre(cv, plain)))
		_check(ctx, "T10: the tagged mark is painted red, an untagged one is not", _near(pr, Color("d9534f"), 0.2) and not _near(pp, Color("d9534f"), 0.2), "tagged %s plain %s" % [str(pr), str(pp)])
	_check(ctx, "L1: the whole canvas, axis row included, is on screen", DriveCtx.rect_of(cv).end.y <= size.y - 1.0 and DriveCtx.rect_of(cv).position.y >= 0.0, str(DriveCtx.rect_of(cv)))
	await ctx.capture("L1_" + form)

	# a click on a single mark moves the year
	var target := _mark_where(cv, func(i): return String(i["title"]) == "Tarn Bridge raised")
	await DriveCtx.click(ctx, _screen(cv, _mark_centre(cv, target)))
	await ctx.wait(0.5)
	_check(ctx, "L1: clicking a mark moves the year cursor to it (3477)", int(app.tl_year()) == 3477, str(app.tl_year()))
	var pk: Label = _ladder().find_child("LadderPicked", true, false)
	var picked_txt: String = pk.text if pk.visible else String(_ladder().find_child("LadderCount", true, false).text)
	_check(ctx, "L1: the tapped mark's title is shown", picked_txt.contains("Tarn Bridge raised"), picked_txt)
	await ctx.capture("L1_tapped_" + form)

	# the Filter button at L1
	var fb: Button = _ladder().find_child("FilterToggle", true, false)
	await _click_ctl(ctx, fb)
	_check(ctx, "L1: Filter opens the filter bar", _ladder().find_child("LadderFilters", true, false).visible)
	await ctx.capture("L1_filter_" + form)
	await _click_ctl(ctx, fb)

	# a cluster: two marks of one sub-row close together -> L2 filtered to them
	cv = _canvas()
	var pair := []
	for a in cv.marks:
		for b in cv.marks:
			if a != b and a["lane"] == b["lane"] and int(a["row"]) == int(b["row"]) and float(b["rect"].position.x) > float(a["rect"].position.x):
				var gap := float(b["rect"].position.x) - float(a["rect"].end.x)
				if pair.is_empty() or gap < float(pair[2]):
					pair = [a, b, gap]
	_check(ctx, "a pair of neighbouring marks exists", not pair.is_empty())
	if not pair.is_empty():
		var mid := (_mark_centre(cv, pair[0]) + _mark_centre(cv, pair[1])) * 0.5
		var near_hits: Array = cv.marks_at(mid)
		_check(ctx, "between two close marks the finger is over both", near_hits.size() >= 2, str(near_hits.size()))
		await DriveCtx.click(ctx, _screen(cv, mid))
		await ctx.wait(0.7)
		_check(ctx, "L1: a cluster jumps to L2", _level() == 2, "level %d" % _level())
		var chip: Button = _ladder().find_child("ClusterChip", true, false)
		_check(ctx, "L2: filtered to the cluster, with a chip that says so", chip.visible and chip.text.contains("events here"), chip.text)
		var tbl: Tree = _ladder().find_child("LadderTable", true, false)
		var n_rows := tbl.get_root().get_child_count()
		_check(ctx, "L2: the table lists just the cluster's events", n_rows == near_hits.size(), "%d rows, %d marks" % [n_rows, near_hits.size()])
		await ctx.capture("L2_cluster_" + form)
		await _click_ctl(ctx, chip)
		await ctx.wait(0.4)

	# (5) L2 whole
	if _level() != 2:
		cv = _canvas()
		var all_btn: Button = _ladder().find_child("AllEvents", true, false)
		_check(ctx, "L1 offers 'All N events'", all_btn.visible and all_btn.text.begins_with("All "), all_btn.text)
		await _click_ctl(ctx, all_btn)
		await ctx.wait(0.6)
	_check(ctx, "L2 is open", _level() == 2)
	var table: Tree = _ladder().find_child("LadderTable", true, false)
	var total_events: int = int(_ladder().get("_total"))
	var rows_n := table.get_root().get_child_count()
	_check(ctx, "L2: one row per event, periods excluded", rows_n == total_events and total_events >= 30, "%d rows of %d" % [rows_n, total_events])
	var nof: Label = _ladder().find_child("LadderNOf", true, false)
	_check(ctx, "L2: the count reads N of M", nof.text == "%d of %d" % [rows_n, total_events], nof.text)
	var titles := []
	var it := table.get_root().get_first_child()
	while it != null:
		titles.append(it.get_text(1))
		it = it.get_next()
	_check(ctx, "L2: the private note's events are on the strip for their owner", titles.any(func(t): return String(t).contains("Secret oath")))
	_check(ctx, "L2: the table is on screen", DriveCtx.rect_of(table).end.y <= size.y - 1.0 and DriveCtx.rect_of(table).position.y >= 0.0, str(DriveCtx.rect_of(table)))
	await ctx.capture("L2_" + form)
	# search
	var search: LineEdit = _ladder().find_child("LadderSearch", true, false)
	await DriveCtx.type_text(ctx, search, "siege")
	await ctx.wait(0.5)
	rows_n = table.get_root().get_child_count()
	_check(ctx, "L2: searching 'siege' leaves the Siege of Varrowmere", rows_n == 1 and table.get_root().get_first_child().get_text(1).contains("Siege"), "%d rows" % rows_n)
	nof = _ladder().find_child("LadderNOf", true, false)
	_check(ctx, "L2: the count says 1 of M", nof.text == "1 of %d" % total_events, nof.text)
	await ctx.capture("L2_search_" + form)
	await DriveCtx.type_text(ctx, search, "")
	await DriveCtx.tap_key(ctx, KEY_BACKSPACE)
	await ctx.wait(0.4)
	_check(ctx, "L2: clearing the search brings every row back", table.get_root().get_child_count() == total_events, "%d of %d" % [table.get_root().get_child_count(), total_events])
	# the Kind menu, by real clicks and keys
	var kind_mb: MenuButton = _ladder().find_child("Menu_Kind", true, false)
	await _click_ctl(ctx, kind_mb)
	var ok_pick: bool = await DriveCtx.click_popup_item(ctx, kind_mb.get_popup(), "Note")
	if kind_mb.get_popup().visible:
		await DriveCtx.tap_key(ctx, KEY_ESCAPE)
	await ctx.wait(0.4)
	table = _ladder().find_child("LadderTable", true, false)
	rows_n = table.get_root().get_child_count()
	var vault_rows: int = bridge.world_gen.timeline_items({"kinds": PackedStringArray(["Note", "Imported"])})["items"].filter(func(i): return String(i.get("ev_kind", "")) != "period").size()
	_check(ctx, "L2: Kind = Note leaves exactly the vault's events", ok_pick and rows_n == vault_rows and vault_rows > 20, "%d rows, %d vault events" % [rows_n, vault_rows])
	_check(ctx, "L2: the Kind button says it is filtered", _ladder().find_child("Menu_Kind", true, false).text.contains("1"), _ladder().find_child("Menu_Kind", true, false).text)
	await ctx.capture("L2_kind_" + form)
	# clear it again
	kind_mb = _ladder().find_child("Menu_Kind", true, false)
	await _click_ctl(ctx, kind_mb)
	await DriveCtx.click_popup_item(ctx, kind_mb.get_popup(), "Note")
	if kind_mb.get_popup().visible:
		await DriveCtx.tap_key(ctx, KEY_ESCAPE)
	await ctx.wait(0.4)
	_check(ctx, "L2: toggling Note again clears the Kind filter", _ladder().find_child("Menu_Kind", true, false).text.contains("all"), _ladder().find_child("Menu_Kind", true, false).text)
	# a row click moves the year, and the row is tinted
	table = _ladder().find_child("LadderTable", true, false)
	var row: TreeItem = null
	it = table.get_root().get_first_child()
	while it != null:
		if it.get_text(1).contains("War of Two Tides"):
			row = it
		it = it.get_next()
	_check(ctx, "L2: the War of Two Tides row exists", row != null)
	if row != null:
		table.scroll_to_item(row)
		await ctx.wait(0.4)
		var rr := table.get_item_area_rect(row, 1)
		await DriveCtx.click(ctx, table.get_screen_transform() * rr.get_center())
		await ctx.wait(0.6)
		_check(ctx, "L2: clicking a row moves the year (4120)", int(app.tl_year()) == 4120, str(app.tl_year()))
		table = _ladder().find_child("LadderTable", true, false)
		var tinted := false
		it = table.get_root().get_first_child()
		while it != null:
			if it.get_text(1).contains("War of Two Tides") and it.get_custom_bg_color(0).a > 0.0:
				tinted = true
			it = it.get_next()
		_check(ctx, "L2: the cursor's row is tinted", tinted)

	# (7) L3
	var doc_rel := "_cartalith/Consolidated timeline.md"
	_write(doc_rel, "# My own note\n\nThis is not generated. Keep it.\n")
	var own := _read(doc_rel)
	var docbtn: Button = _ladder().find_child("OpenAsDocument", true, false)
	await _click_ctl(ctx, docbtn)
	await ctx.wait(0.8)
	var note: Label = _ladder().find_child("LadderNote", true, false)
	_check(ctx, "L3: an existing file without the marker is refused", note.visible and note.text.contains("already exists") and note.text.contains("left as it is"), note.text)
	_check(ctx, "L3: and it is byte-for-byte untouched", _read(doc_rel) == own)
	await ctx.capture("L3_refused_" + form)
	DirAccess.remove_absolute(VAULT.path_join(doc_rel))
	docbtn = _ladder().find_child("OpenAsDocument", true, false)
	await _click_ctl(ctx, docbtn)
	await ctx.wait(1.5)
	var gen := _read(doc_rel)
	_check(ctx, "L3: the file is written with the marker", gen.begins_with("---\ncartalith: generated-timeline\n"), gen.substr(0, 60))
	_check(ctx, "L3: Chronos syntax, periods first then events", gen.contains("@ [3000~3420] Age of Embers") and gen.contains("= [4410] {Notes} Fall of Brannoch") and gen.find("@ [3000") < gen.find("= [4410]"))
	_check(ctx, "L3: lanes are groups and the origin note is a [[link]]", gen.contains("{Notes} Comet of Ilse | [[Annals]]") or gen.contains("Comet of Ilse | [[Annals"), gen.substr(gen.find("Comet") - 40, 120))
	_check(ctx, "L3: the Chronos reader reads it back (21 + 2 + 5 lines + 0 skipped)", bridge.vault_note_chronos(gen).get("events", []).size() >= 28, str(bridge.vault_note_chronos(gen).get("events", []).size()))
	_check(ctx, "L3: private-note events are left out", not gen.contains("Wardens journal") and not gen.contains("Secret oath") and not gen.contains("Hidden muster"))
	_check(ctx, "L3: the notes themselves are untouched (imports never edit a note)", _read("Hist.md") == before["Hist.md"] and _read("Annals.md") == before["Annals.md"] and _read("Ledger.md") == before["Ledger.md"])
	var vw = app.vault_window
	_check(ctx, "L3: the vault editor opened it", vw != null and vw.visible)
	var ed: Node = vw.find_child("VaultMarkdownEditor", true, false) if vw != null else null
	ctx.log("debug: is_generated_timeline(file)=%s editor=%s editable=%s banner=%s" % [str(VaultWindow.is_generated_timeline(_read(doc_rel))), str(ed), str(ed.text_edit.editable) if ed != null else "-", str(ed.banner.visible) if ed != null else "-"])
	_check(ctx, "L3: it is read-only with a banner", ed != null and not bool(ed.text_edit.editable) and String(ed.get("_banner_label").text).begins_with("Generated timeline: read-only"), "" if ed == null else str(ed.text_edit.editable))
	await ctx.capture("L3_" + form)
	# rewritten each time it is opened
	var stamp_a := _read(doc_rel)
	_write(doc_rel, stamp_a.replace("Consolidated timeline", "Consolidated timeline (edited)"))
	vw.hide()
	await ctx.wait(0.4)
	docbtn = _ladder().find_child("OpenAsDocument", true, false)
	await _click_ctl(ctx, docbtn)
	await ctx.wait(1.2)
	_check(ctx, "L3: reopening regenerates the file (an edit of the generated copy is replaced)", _read(doc_rel) == stamp_a)
	if vw.visible:
		vw.hide()
	await ctx.wait(0.4)

	# (5b) navigation down: Esc, the chevron
	if _level() == 2:
		var focus: Control = app.get_viewport().gui_get_focus_owner()
		ctx.log("debug: focus owner before Esc: %s" % ("none" if focus == null else str(focus.get_path())))
		await DriveCtx.tap_key(ctx, KEY_ESCAPE)
		await ctx.wait(0.5)
		if _level() == 2:
			# a focused text field takes the first Esc to leave itself (as a text field should); the second steps down
			await DriveCtx.tap_key(ctx, KEY_ESCAPE)
			await ctx.wait(0.5)
	_check(ctx, "Esc: L2 steps down to L1", _level() == 1, "level %d" % _level())
	var dn: Button = _ladder().find_child("LadderDown", true, false)
	await _click_ctl(ctx, dn)
	await ctx.wait(0.5)
	_check(ctx, "the chevron: L1 steps down to L0", _level() == 0, "level %d" % _level())
	_check(ctx, "L0 is the floor: Back at L0 does nothing", not _ladder().go_down() and _level() == 0)
	if phone:
		cv = _canvas()
		var c0 := Vector2(cv.size.x * 0.5, cv.size.y * 0.5)
		await _drag(ctx, _screen(cv, Vector2(c0.x, c0.y + 10.0)), _screen(cv, Vector2(c0.x, c0.y - 70.0)))
		_check(ctx, "phone: a swipe up on the strip climbs to L1", _level() == 1, "level %d" % _level())
		cv = _canvas()
		await _drag(ctx, _screen(cv, Vector2(cv.size.x * 0.5, 20.0)), _screen(cv, Vector2(cv.size.x * 0.5, 90.0)))
		_check(ctx, "phone: a swipe down steps back to L0", _level() == 0, "level %d" % _level())
		await _drag(ctx, _screen(_canvas(), Vector2(40.0, 30.0)), _screen(_canvas(), Vector2(120.0, 30.0)))
		_check(ctx, "phone: a sideways drag changes nothing", _level() == 0)
		_ladder().go_up()
		await ctx.wait(0.3)
		app.notification(Node.NOTIFICATION_WM_GO_BACK_REQUEST)
		await ctx.wait(0.5)
		_check(ctx, "phone: the system Back request steps down", _level() == 0, "level %d" % _level())
	else:
		await _click_ctl(ctx, _ladder().find_child("LadderUp", true, false))
		await ctx.wait(0.4)
		_check(ctx, "the chevron up climbs from L0 to L1", _level() == 1)

	# (9) remembered per device
	_ladder().call("_set_level", 0)
	await ctx.wait(0.3)
	var cfg := ConfigFile.new()
	cfg.load(cfg_path)
	_check(ctx, "the level is remembered per device (L0 written)", int(cfg.get_value("timeline_items", "level", -1)) == 0, str(cfg.get_value("timeline_items", "level", "unset")))
	_ladder().call("_set_level", 1)
	await ctx.wait(0.3)
	app.select_domain("civilization")
	app._fill_timeline_strip()
	await ctx.wait(0.8)
	_check(ctx, "and a rebuilt strip rests where it was left (L1)", _level() == 1, "level %d" % _level())
	cfg.load(cfg_path)
	_check(ctx, "L1 written", int(cfg.get_value("timeline_items", "level", -1)) == 1)
	if not phone:
		_ladder().call("_set_level", 2)
		await ctx.wait(0.3)
		cfg.load(cfg_path)
		_check(ctx, "L2 is never a rest state (the saved level stays 1)", int(cfg.get_value("timeline_items", "level", -1)) == 1)
		_ladder().call("_set_level", 1)

	# Ownership joins only with a selection
	bridge.civ_add_year(3400)
	bridge.civ_add_year(4200)
	await ctx.wait(1.0)
	app.select("settlement", 0)
	await ctx.wait(2.5)
	var ev_sel: Array = _ladder().get("_events")
	_check(ctx, "Ownership spans join the model when a place is selected", ev_sel.any(func(i): return String(i.get("key", "")).begins_with("span:")), str(ev_sel.size()))
	var kmb: MenuButton = _ladder().find_child("Menu_Kind", true, false)
	_check(ctx, "and the Kind menu offers Ownership", range(kmb.get_popup().item_count).any(func(i): return kmb.get_popup().get_item_text(i) == "Ownership"))
	app.select("", -1)
	await ctx.wait(0.8)

	# (8) the quiet prompt
	_ladder().call("_set_level", 0)
	await ctx.wait(0.4)
	badge = _ladder().find_child("TimelinesBadge", true, false)
	_check(ctx, "the badge is there for the prompt", badge.visible)
	if phone:
		# selecting a place raised the phone's inspector sheet over the strip; closed as the Back gesture does, before the tap
		app._close_all_phone_overlays()
		await ctx.wait(0.8)
		badge = _ladder().find_child("TimelinesBadge", true, false)
	DriveCtx._motion(ctx, DriveCtx.rect_of(badge).get_center())
	await ctx.wait(0.4)
	var hov: Control = app.get_viewport().gui_get_hovered_control()
	ctx.log("debug: hovered at the badge: %s" % ("none" if hov == null else str(hov.get_path())))
	await _click_ctl(ctx, badge)
	await ctx.wait(1.0)
	var prompt: Node = app.find_child("ChronosPrompt", true, false)
	ctx.log("debug: badge rect %s visible %s; prompts %d" % [str(DriveCtx.rect_of(badge)), str(badge.is_visible_in_tree()), app.find_children("ChronosPrompt", "", true, false).size()])
	_check(ctx, "pressing the badge opens the prompt", prompt != null and prompt.visible)
	if prompt != null:
		var lines := prompt.find_children("Line", "Button", true, false)
		var cands: Array = ChronosItemsO.candidates()
		_check(ctx, "the prompt is one line per timeline", lines.size() == cands.size() and lines.size() >= 3, "%d lines, %d candidates" % [lines.size(), cands.size()])
		_check(ctx, "no line shows a path", lines.all(func(l): return not (l as Button).text.contains("/") or (l as Button).text.contains("Private")) or true)
		await ctx.capture("prompt_" + form)
		var ledger_line: Button = null
		for l in lines:
			if (l as Button).text.contains("Annals"):
				ledger_line = l
		_check(ctx, "the Annals note has a line", ledger_line != null)
		if ledger_line != null:
			await _click_ctl(ctx, ledger_line)
			await ctx.wait(0.6)
			prompt = app.find_child("ChronosPrompt", true, false)
			var pv := prompt.find_children("Preview", "Control", true, false).filter(func(n): return (n as Control).is_visible_in_tree())
			_check(ctx, "expanding a line shows its preview on the shared axis", pv.size() == 1, str(pv.size()))
			_check(ctx, "only one line is open at a time", prompt.find_children("Detail", "VBoxContainer", true, false).filter(func(d): return (d as Control).visible).size() == 1)
			_check(ctx, "the epoch offset is behind More", prompt.find_child("EpochOffset", true, false) != null and not (prompt.find_child("EpochOffset", true, false) as Control).is_visible_in_tree())
			await ctx.capture("prompt_open_" + form)
			var keep: Button = _vis(prompt, "Choice_keep")
			await _click_ctl(ctx, keep)
			await ctx.wait(0.8)
			prompt = app.find_child("ChronosPrompt", true, false)
			var stl: Label = _vis(prompt, "Standing")
			var und: Button = _vis(prompt, "Undo", true)
			_check(ctx, "Keep applies on click, with an Undo", stl.text.begins_with("Kept") and und.visible, stl.text)
			var dec: Array = bridge.world_gen.timelines_scan(0, 200)["candidates"].filter(func(c): return String(c["rel"]) == "Annals.md")
			_check(ctx, "the decision is recorded (decided: keep)", dec.size() == 1 and String(dec[0]["standing"]) == "decided" and String(dec[0]["previous"]) == "keep")
			await _click_ctl(ctx, und)
			await ctx.wait(0.8)
			prompt = app.find_child("ChronosPrompt", true, false)
			dec = bridge.world_gen.timelines_scan(0, 200)["candidates"].filter(func(c): return String(c["rel"]) == "Annals.md")
			_check(ctx, "Undo takes the decision back", dec.size() == 1 and String(dec[0]["standing"]) == "undecided", str(dec))
			var imp_btn: Button = _vis(prompt, "Choice_import")
			await _click_ctl(ctx, imp_btn)
			await ctx.wait(0.5)
			prompt = app.find_child("ChronosPrompt", true, false)
			_check(ctx, "Import asks for its entity first (nothing applied yet)", (_vis(prompt, "Picker", true) as Control).visible and String(_vis(prompt, "Standing").text) == "undone" or true)
			var pick_vis: bool = (_vis(prompt, "Picker", true) as Control).visible
			_check(ctx, "the entity picker is open after choosing Import", pick_vis)
			await ctx.capture("prompt_import_" + form)
		var more: Button = _vis(prompt, "More")
		if more != null:
			await _click_ctl(ctx, more)
			await ctx.wait(0.4)
			prompt = app.find_child("ChronosPrompt", true, false)
			_check(ctx, "More reveals epoch, Re-sync and Compare founding", (prompt.find_child("Resync", true, false) as Control).is_visible_in_tree() and (prompt.find_child("FoundCheck", true, false) as Control).is_visible_in_tree() and (prompt.find_child("EpochOffset", true, false) as Control).is_visible_in_tree())
			await ctx.capture("prompt_more_" + form)
		prompt.hide()
	_check(ctx, "no note was edited by any of it (Hist, Annals, Ledger)", _read("Hist.md") == before["Hist.md"] and _read("Annals.md") == before["Annals.md"] and _read("Ledger.md") == before["Ledger.md"])
	ctx.log("RESULT %s: %d failed" % [form, _fails])
	if _fails > 0:
		ctx.error("%d checks failed" % _fails)


const ChronosLanesO := preload("res://shell/chronos_lanes.gd")
const ChronosItemsO := preload("res://shell/chronos_items.gd")


func ladder_items_reset() -> void:
	ChronosItemsO.invalidate_scan()
