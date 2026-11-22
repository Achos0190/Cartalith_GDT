extends RefCounted
## Ruling HF (2026-11-22): the world's timeline range follows the vault until the user overrides it, driven through the REAL
## shell on each form. Run (see `TOOLING_PLAN.md`):
##   python -I cartalith-native/tools/fennara_drive.py GODOT --shell desktop|tablet|phone res://tools/tlrange_drive.gd \
##       --dll DLL --out DIR --root C:/cw/<own scratch> --timeout 420
## The vault is a SYNTHETIC scratch folder (`VAULT`, never the owner's) in the owner's shape: one Chronos block 3000-4769
## (a point at 3000 and one at 4769 so the extent is exact). Steps: (1) the 4200/4700 finding with no vault: set Start 4200 and
## End 4700 as the sim form does, run the simulation over them, and the track and the last recorded year are 4700, not 8900;
## (2) the vault linked: the range is the vault's 3000..4769, the world's founding events outside it keep their years and L2
## lists them "(before the range)"; (3) the user's start/end 3100/4600 win and the table notes events on both sides; (4) save,
## open another world, reopen: the override returns; a world saved with none reopens with none; (5) phone only: Back with the
## transport panel open at L0 closes the panel and does not raise "Exit Cartalith"; (6) phone only: the ladder's count, Back
## and Filter text is the phone's body size and the targets are 44 dp or more. Every check prints `ok` or `FAIL`.

const VAULT := "C:/cw/tlrange_vault"
const SAVE := "C:/cw/tlrange_save.zip"
var _fails := 0
var app: Node
var bridge


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _find(ctx, t: String, cls: String = "") -> Dictionary:
	for e in DriveCtx.dump_ui(ctx, t):
		if cls == "" or e["class"] == cls:
			return e
	return {}


func _wait_for(ctx, t: String, secs: float = 20.0) -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		var e := _find(ctx, t)
		if not e.is_empty():
			return e
		await ctx.wait(0.25)
	return {}


func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))


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


func _ladder() -> Node:
	for n in app.find_children("ChronosItems", "VBoxContainer", true, false):
		if (n as Control).is_visible_in_tree():
			return n
	return null


## The L2 table's Event column, one string per row.
func _rows(lad: Node) -> Array:
	var out: Array = []
	var t: Tree = lad.get("_table")
	var row: TreeItem = t.get_root().get_first_child()
	while row != null:
		out.append(row.get_text(1))
		row = row.get_next()
	return out


func _count(rows: Array, needle: String) -> int:
	return rows.filter(func(r): return String(r).contains(needle)).size()


func run(ctx) -> void:
	app = ctx.get_scene_root()
	if not app.has_method("select_domain"):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	var phone: bool = app.is_phone()
	var form := "phone" if phone else ("tablet" if DccTheme.is_tablet() else "desktop")
	ctx.log("form: %s" % form)

	# (0) past the welcome card
	if phone:
		var nw := await _wait_for(ctx, "+ NEW WORLD")
		await _click_entry(ctx, nw)
		await _click_entry(ctx, await _wait_for(ctx, "CANCEL"))
		await ctx.wait(1.0)
	else:
		var cont := await _wait_for(ctx, "Continue without a world")
		if not cont.is_empty():
			await _click_entry(ctx, cont)
		await ctx.wait(0.5)
	await _generate(ctx, 483920)
	_check(ctx, "a world exists", bool(bridge.has_world))
	_check(ctx, "no override on a fresh world", app.tl_override().is_empty(), str(app.tl_override()))

	# (1) the owner's 4200/4700 finding, with no vault: the sim form's Start and End are the world's own start and end
	app.tl_override_set(4200, 4700)
	_check(ctx, "the override reads back (4200, 4700)", int(app.tl_override().get("start", -1)) == 4200 and int(app.tl_override().get("end", -1)) == 4700, str(app.tl_override()))
	_check(ctx, "the sim form's span is the override", app.tl_sim_span() == [4200, 4700], str(app.tl_sim_span()))
	var r0: Dictionary = app.tl_range()
	_check(ctx, "the track is 4200..4700 before any run", int(r0["min"]) == 4200 and int(r0["max"]) == 4700, str(r0))
	var ws: Object = app.find_child("CivilizationWorkspace", true, false)
	ws.set("_tl_sim_step_years", 50)
	ws.call("_tl_run_simulation", true)  # the form's own run: it reads the span from the world, as the Simulate button does
	await ctx.wait(1.5)
	var out_txt := String(ws.get("_tl_sim_out"))
	ctx.log("sim out: " + out_txt)
	_check(ctx, "the simulation ran over 4200 -> 4700 (not 4200 + 4700 = 8900)", out_txt.begins_with("Simulated") and out_txt.contains("4200") and out_txt.contains("4700") and not out_txt.contains("8900"), out_txt)
	var r1: Dictionary = app.tl_range()
	_check(ctx, "the track still ends on 4700 after the run", int(r1["min"]) == 4200 and int(r1["max"]) == 4700, str(r1))
	_check(ctx, "the first recorded year is 4200 (the run started where the user's Start says)", int(app.tl_recorded_years()[0]) == 4200, str(app.tl_recorded_years()))
	_check(ctx, "the last recorded year is 4700", int(app.tl_recorded_years()[-1]) == 4700, str(app.tl_recorded_years()))
	app.tl_override_set(null, null)
	_check(ctx, "clearing the override empties it", app.tl_override().is_empty())

	# (2) the vault: range = its lowest and highest year; founding events outside keep their years
	DirAccess.make_dir_recursive_absolute(VAULT)
	var note := "---\ntitle: Annals\n---\n```chronos\n* [3000] Dawn\n* [3477] Tarn Bridge raised\n- [3890~3935] Glass Road built\n* [4302] Library burns\n* [4769] Beacon\n```\n"
	_write("Hist.md", note)
	var con: Dictionary = bridge.vault_connect(VAULT, "Range scratch vault")
	_check(ctx, "the scratch vault connects", bool(con.get("ok", false)), str(con))
	var tid0 := int((bridge.settlements()[0] as Dictionary)["tid"])
	var att: Dictionary = bridge.vault_attach("settlement", tid0, "Place", "Hist.md", "")
	_check(ctx, "Hist.md is linked", bool(att.get("ok", false)), str(att))
	ChronosItemsO.invalidate_scan()
	app.select_domain("civilization")
	await ctx.wait(1.0)
	if phone:
		await DriveCtx.click(ctx, DriveCtx.rect_of(app._tl_phone_button).get_center())
	else:
		app._tl_expanded = true
		app._fill_timeline_strip()
	app.emit_signal("timeline_records_changed")
	await ctx.wait(1.5)
	var lad := _ladder()
	_check(ctx, "the ladder is on the strip", lad != null)
	var rv: Dictionary = app.tl_range()
	_check(ctx, "no override: the track is the vault's 3000..4769 (the sim's recorded years do not widen it)", int(rv["min"]) == 3000 and int(rv["max"]) == 4769 and bool(rv["from_world"]), str(rv))
	_check(ctx, "the sim form defaults to the vault range", app.tl_sim_span() == [3000, 4769], str(app.tl_sim_span()))
	lad.call("_set_level", 2)
	await ctx.wait(0.8)
	var rows := _rows(lad)
	var before := _count(rows, "(before the range)")
	_check(ctx, "L2 lists the world's founding events before the range, with the note", before > 0, "before %d of %d" % [before, rows.size()])
	_check(ctx, "the note is the ladder's own off-axis count", before + _count(rows, "(after the range)") == int(lad.get("_off")), "%d vs %d" % [before + _count(rows, "(after the range)"), int(lad.get("_off"))])
	_check(ctx, "no vault event carries a note while the track is the vault's", _count(rows, "Tarn Bridge raised  (") == 0 and _count(rows, "Dawn  (") == 0)
	await ctx.capture("L2_default_" + form)

	# (3) the user's start and end win
	app.tl_override_set(3100, 4600)
	await ctx.wait(1.0)
	var ro: Dictionary = app.tl_range()
	_check(ctx, "the user's 3100..4600 wins over the vault", int(ro["min"]) == 3100 and int(ro["max"]) == 4600, str(ro))
	lad = _ladder()
	if int(lad.level) != 2:
		lad.call("_set_level", 2)
		await ctx.wait(0.8)
	rows = _rows(lad)
	_check(ctx, "'Dawn' (3000) is now before the range", _count(rows, "Dawn  (before the range)") == 1, str(rows.slice(0, 3)))
	_check(ctx, "'Beacon' (4769) is now after the range", _count(rows, "Beacon  (after the range)") == 1)
	_check(ctx, "an event inside is not noted", _count(rows, "Tarn Bridge raised  (") == 0)
	_check(ctx, "every event keeps its year in the table", _count(rows, "Dawn") == 1 and _count(rows, "Beacon") == 1)
	await ctx.capture("L2_override_notes_" + form)
	app.tl_override_set(3100, null)
	await ctx.wait(0.5)
	_check(ctx, "start only: the end follows the vault again (4769)", int(app.tl_range()["max"]) == 4769 and int(app.tl_range()["min"]) == 3100, str(app.tl_range()))
	app.tl_override_set(3100, 4600)
	await ctx.capture("L1_override_" + form)

	# (4) save, another world, reopen: the override returns; a world saved with none reopens with none
	var saved: bool = bridge.save_project(SAVE)
	_check(ctx, "the project saved", saved)
	await _generate(ctx, 771133)
	_check(ctx, "another world carries no override", app.tl_override().is_empty(), str(app.tl_override()))
	_check(ctx, "the project reopens", bridge.load_save(SAVE))
	await ctx.wait(1.0)
	_check(ctx, "the override came back with the world", int(app.tl_override().get("start", -1)) == 3100 and int(app.tl_override().get("end", -1)) == 4600, str(app.tl_override()))
	app.tl_override_set(null, null)
	_check(ctx, "save with none", bridge.save_project(SAVE))
	ctx.log("step A")
	await _generate(ctx, 771133)
	ctx.log("step B")
	app.tl_override_set(1, 2)
	ctx.log("step C")
	_check(ctx, "reopen a world saved with none", bridge.load_save(SAVE))
	await ctx.wait(1.0)
	_check(ctx, "it has no override (a stale one does not survive the open)", app.tl_override().is_empty(), str(app.tl_override()))
	app.tl_override_set(null, null)

	# (5), (6) phone only
	if phone:
		await DriveCtx.click(ctx, DriveCtx.rect_of(app._tl_phone_button).get_center()) if not app.is_phone_sim_strip_open() else null
		await ctx.wait(1.0)
		lad = _ladder()
		# (6) text and targets, measured at L1 where the count line, Back and Filter are all on screen
		lad.call("_set_level", 1)
		await ctx.wait(0.6)
		lad = _ladder()
		var cnt: Label = lad.find_child("LadderCount", true, false)
		var back: Button = lad.find_child("LadderDown", true, false)
		var flt: Button = lad.find_child("FilterToggle", true, false)
		var body: int = DccTheme.FS_BODY
		var f_cnt := cnt.get_theme_font_size("font_size")
		var f_back := back.get_theme_font_size("font_size")
		var f_flt := flt.get_theme_font_size("font_size")
		ctx.log("font px: count %d, Back %d, Filter %d (phone body text %d); phone_scale %.3f" % [f_cnt, f_back, f_flt, body, DccTheme.phone_scale()])
		_check(ctx, "the count line is the phone's body size", f_cnt >= body, str(f_cnt))
		_check(ctx, "Back is the phone's body size", f_back >= body, str(f_back))
		_check(ctx, "Filter is the phone's body size", f_flt >= body, str(f_flt))
		var sc := DccTheme.phone_scale()
		for b in [back, flt, lad.find_child("LadderUp", true, false)]:
			var h: float = DriveCtx.rect_of(b).size.y / sc
			_check(ctx, "%s is 44 dp or more tall" % (b as Button).name, h >= 44.0, "%.1f dp" % h)
		for c in [cnt, back, flt]:
			_check(ctx, "%s is on screen" % (c as Control).name, (c as Control).is_visible_in_tree())
		var vw: float = app.get_viewport().get_visible_rect().size.x
		_check(ctx, "the transport panel stays inside the screen at the larger text (count line included)", DriveCtx.rect_of(app._phone_sim_strip).end.x <= vw + 0.5 and DriveCtx.rect_of(cnt).end.x <= vw + 0.5, "strip right %.1f, count right %.1f, screen %.1f" % [DriveCtx.rect_of(app._phone_sim_strip).end.x, DriveCtx.rect_of(cnt).end.x, vw])
		await ctx.capture("L1_transport_" + form)
		lad.call("_set_level", 0)
		await ctx.wait(0.6)
		_check(ctx, "the transport panel is open at L0", app.is_phone_sim_strip_open() and int(_ladder().level) == 0)
		await ctx.capture("L0_transport_" + form)
		# (5) the back chain
		app.notification(Node.NOTIFICATION_WM_GO_BACK_REQUEST)
		await ctx.wait(0.8)
		_check(ctx, "Back at L0 closes the transport panel", not app.is_phone_sim_strip_open())
		_check(ctx, "and does not raise 'Exit Cartalith'", _find(ctx, "Exit Cartalith").is_empty() and _find(ctx, "Exit the app?").is_empty())
		await ctx.capture("after_back_" + form)
	ctx.log("RESULT %s: %d failed" % [form, _fails])
	if _fails > 0:
		ctx.error("%d checks failed" % _fails)


const ChronosItemsO := preload("res://shell/chronos_items.gd")
