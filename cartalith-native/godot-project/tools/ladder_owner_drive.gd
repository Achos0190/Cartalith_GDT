extends RefCounted
## Item 4 (2026-11-22): the timeline ladder (Ruling GT) over a COPY of the owner's real vault (`VAULT`, read-only original
## never opened). Prints counts and fixed-vocabulary words only, never note text. Run:
##   python -I tools/fennara_drive.py GODOT --shell desktop res://tools/ladder_owner_drive.gd --dll DLL --out DIR --root C:/cw/<scratch>
## Steps: welcome card, a world, connect the copy, scan (counts), import every Chronos candidate onto settlement 0 (the copy's
## own state, not the vault's notes), L0 -> L1 -> L2 -> L3 with frames, an unmarked file at the L3 path refused byte-identically,
## the generated write, the quiet prompt. `ctx.log` lines `ok`/`FAIL`.

const VAULT := "C:/cw/item4_vault"
var _fails := 0
var app: Node
var bridge


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _find(ctx, t: String) -> Dictionary:
	for e in DriveCtx.dump_ui(ctx, t):
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


func _click_ctl(ctx, c: Control) -> void:
	await DriveCtx.click(ctx, DriveCtx.rect_of(c).get_center())


func _ladder() -> Node:
	for n in app.find_children("ChronosItems", "VBoxContainer", true, false):
		if (n as Control).is_visible_in_tree():
			return n
	return null


func run(ctx) -> void:
	app = ctx.get_scene_root()
	if not app.has_method("select_domain"):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	var cont := await _wait_for(ctx, "Continue without a world")
	_check(ctx, "welcome card", not cont.is_empty())
	var r: Array = cont["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))
	await ctx.wait(0.5)
	bridge.generate({"seed": 483920, "width_km": 800.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.45})
	await ctx.wait(0.2)
	var t0 := Time.get_ticks_msec()
	while bool(bridge.generating) and Time.get_ticks_msec() - t0 < 300000:
		await ctx.wait(0.25)
	await ctx.wait(2.0)
	_check(ctx, "a world exists", bool(bridge.has_world))
	var con: Dictionary = bridge.vault_connect(VAULT, "Owner vault copy")
	_check(ctx, "the copy connects", bool(con.get("ok", false)), str(con.keys()))
	var cands := []
	var off := 0
	var scanned := 0
	for i in 20:
		var s: Dictionary = bridge.world_gen.timelines_scan(off, 200)
		if not bool(s.get("ok", true)):
			_check(ctx, "scan ok", false, str(s.get("error", "")))
			break
		cands.append_array(s["candidates"])
		scanned += int(s.get("scanned", 0))
		off = int(s.get("next", -1))
		if off < 0:
			break
	var by_form := {}
	var priv := 0
	var skipped := 0
	var events := 0
	for c in cands:
		by_form[String(c["form"])] = int(by_form.get(String(c["form"]), 0)) + 1
		if c.has("private"):
			priv += 1
		skipped += int(c["skipped"])
		events += int(c["count"])
	ctx.log("COUNT notes scanned %d, candidates %d, by form %s, private %d, events %d, skipped lines %d" % [scanned, cands.size(), str(by_form), priv, events, skipped])
	app.select_domain("civilization")
	await ctx.wait(1.0)
	app._tl_expanded = true
	app._fill_timeline_strip()
	app.emit_signal("timeline_records_changed")
	await ctx.wait(2.5)
	# the quiet prompt BEFORE any decision (importing decides every candidate and the badge goes)
	var lad0 := _ladder()
	_check(ctx, "the ladder is on the strip", lad0 != null)
	if lad0 == null:
		ctx.error("no ladder")
		return
	var badge: Button = lad0.find_child("TimelinesBadge", true, false)
	_check(ctx, "the badge is visible before any decision", badge != null and badge.visible)
	if badge != null and badge.visible:
		ctx.log("COUNT badge text: %s" % badge.text)
		await _click_ctl(ctx, badge)
		await ctx.wait(1.0)
		var prompt: Node = app.find_child("ChronosPrompt", true, false)
		_check(ctx, "the quiet prompt opens", prompt != null and prompt.visible)
		if prompt != null:
			var lines := prompt.find_children("Line", "Button", true, false)
			var ncand: int = preload("res://shell/chronos_items.gd").candidates().size()
			ctx.log("COUNT prompt lines %d, candidates %d" % [lines.size(), ncand])
			_check(ctx, "one prompt line per timeline", lines.size() == ncand and lines.size() > 0)
			_check(ctx, "no line shows a path", lines.all(func(l): return not (l as Button).text.contains("/") and not (l as Button).text.contains("\\") and not (l as Button).text.to_lower().contains(".md")))
			await ctx.capture("owner_prompt")
			prompt.hide()
	await ctx.wait(0.5)
	var st: Array = bridge.settlements()
	var tid0 := int((st[0] as Dictionary)["tid"])
	var imported := 0
	var refused := 0
	for c in cands:
		if c.has("private"):
			continue
		var imp: Dictionary = bridge.world_gen.timeline_decide(String(c["rel"]), String(c["form"]), int(c["ordinal"]), "import", "settlement", tid0, 0)
		if bool(imp.get("ok", false)):
			imported += 1
		else:
			refused += 1
	ctx.log("COUNT imported %d candidates, %d not imported" % [imported, refused])
	ChronosItemsInv.invalidate_scan()
	app.emit_signal("timeline_records_changed")
	await ctx.wait(2.0)
	var lad := _ladder()
	_check(ctx, "the ladder is on the strip", lad != null)
	if lad == null:
		ctx.error("no ladder")
		return
	var ev: Array = lad.get("_events")
	ctx.log("COUNT ladder events %d, off-axis %d, level %d" % [ev.size(), int(lad.get("_off")), int(lad.level)])
	var rng: Dictionary = app.tl_range()
	ctx.log("COUNT track range %s" % str(rng))
	lad.call("_set_level", 0)
	await ctx.wait(0.8)
	await ctx.capture("owner_L0")
	var cv: Control = lad.find_child("ChronosLanes", true, false)
	await DriveCtx.click(ctx, cv.get_screen_transform() * Vector2(cv.size.x * 0.5, cv.size.y * 0.5))
	await ctx.wait(0.7)
	_check(ctx, "a tap climbs to L1", int(_ladder().level) >= 1, "level %d" % int(_ladder().level))
	await ctx.capture("owner_L1")
	var allb: Button = null
	for bt in _ladder().find_children("*", "Button", true, false):
		if (bt as Button).visible and (bt as Button).text.begins_with("All "):
			allb = bt
	_check(ctx, "the All N events button is there", allb != null)
	if allb != null:
		await _click_ctl(ctx, allb)
	await ctx.wait(1.0)
	_check(ctx, "L2 opens", int(_ladder().level) == 2)
	var table = _ladder().find_child("LadderTable", true, false)
	var rows := 0
	if table != null and table.get_root() != null:
		var it = table.get_root().get_first_child()
		while it != null:
			rows += 1
			it = it.get_next()
	ctx.log("COUNT L2 table rows %d" % rows)
	await ctx.capture("owner_L2")
	var doc_rel := "_cartalith/Consolidated timeline.md"
	var path := VAULT.path_join(doc_rel)
	_check(ctx, "premise: the copy had no file at the L3 path", not FileAccess.file_exists(path))
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string("# unmarked\n\nnot generated\n")
	f.close()
	var own := FileAccess.get_file_as_string(path)
	await _click_ctl(ctx, _ladder().find_child("OpenAsDocument", true, false))
	await ctx.wait(1.0)
	var note: Label = _ladder().find_child("LadderNote", true, false)
	_check(ctx, "L3 refuses an existing unmarked file", note.visible and note.text.contains("left as it is"), note.text.substr(0, 30))
	_check(ctx, "and leaves it byte-identical", FileAccess.get_file_as_string(path) == own)
	await ctx.capture("owner_L3_refused")
	DirAccess.remove_absolute(path)
	await _click_ctl(ctx, _ladder().find_child("OpenAsDocument", true, false))
	await ctx.wait(2.0)
	var gen := FileAccess.get_file_as_string(path)
	_check(ctx, "L3 writes the marked document", gen.begins_with("---\ncartalith: generated-timeline\n"))
	var back: Dictionary = bridge.vault_note_chronos(gen)
	ctx.log("COUNT L3 document bytes %d, events read back %d, skipped %d" % [gen.length(), (back.get("events", []) as Array).size(), (back.get("skipped", []) as Array).size() if back.has("skipped") else -1])
	_check(ctx, "the Chronos reader reads the document back", (back.get("events", []) as Array).size() > 0)
	await ctx.capture("owner_L3")
	if app.vault_window != null and app.vault_window.visible:
		app.vault_window.hide()
	await ctx.wait(0.5)
	_ladder().call("_set_level", 0)
	await ctx.wait(0.5)
	ctx.log("RESULT owner: %d failed" % _fails)
	if _fails > 0:
		ctx.error("%d checks failed" % _fails)

const ChronosItemsInv := preload("res://shell/chronos_items.gd")
