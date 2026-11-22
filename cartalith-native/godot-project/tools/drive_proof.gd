extends RefCounted
## TOOLING_PLAN.md M2 (Ruling GP item 2): the scripted session that proves `DriveCtx` drives the REAL shell on each form.
## Run: `python -I cartalith-native/tools/fennara_drive.py GODOT --shell desktop|tablet|phone res://tools/drive_proof.gd --dll DLL --out DIR`.
## Steps: (1) get past the welcome card / worlds list with real clicks, (2) open a menu and click a row
## (File > New world..., on the phone through the MENU tab after "+ NEW WORLD" then CANCEL), (3) type a seed into the New World
## screen's seed field, (4) READ IT BACK from `dump_ui` (an independent read of the live tree, not the typed string), (5) capture.
## Every check prints `ok` or `FAIL`; the launcher exits 1 on any FAIL. Mutants that must turn it red: `drive_mutant.py`.
## It never writes a world: the session ends on the New World screen without pressing Create.

const SEED := "424242"
var _fails := 0


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


## First dump_ui entry whose text contains `t` (and class `cls` if given), or {}.
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


func run(ctx) -> void:
	var size: Vector2 = ctx.get_scene_root().get_viewport().get_visible_rect().size
	var phone := size.x < 700.0
	ctx.log("form: window canvas %s, %s" % [str(size), "phone" if phone else "desktop/tablet"])
	if phone:
		# the worlds list covers the shell: + NEW WORLD then CANCEL is the way past it, then the MENU tab does the rest
		var nw := await _wait_for(ctx, "+ NEW WORLD")
		_check(ctx, "worlds list shows + NEW WORLD", not nw.is_empty())
		await _click_entry(ctx, nw)
		var cancel := await _wait_for(ctx, "CANCEL")
		_check(ctx, "New World screen offers CANCEL", not cancel.is_empty())
		await _click_entry(ctx, cancel)
		await ctx.wait(1.0)
		_check(ctx, "CANCEL left the New World screen (no seed field)", _find(ctx, "", "SpinBoxLineEdit").is_empty())
		var opened_p: bool = await DriveCtx.menu_open(ctx, "File", "New world")
		_check(ctx, "menu_open File > New world (phone MENU tab)", opened_p)
	else:
		var cont := await _wait_for(ctx, "Continue without a world")
		_check(ctx, "welcome card shows Continue", not cont.is_empty())
		await _click_entry(ctx, cont)
		await ctx.wait(0.5)
		_check(ctx, "welcome card gone after the click", _find(ctx, "Continue without a world").is_empty())
		var opened: bool = await DriveCtx.menu_open(ctx, "File", "New world")
		_check(ctx, "menu_open File > New world", opened)
	# the New World screen: its seed box is the first integer-valued LineEdit
	var seed_path := ""
	var before := ""
	var t_end := Time.get_ticks_msec() + 15000
	while seed_path == "" and Time.get_ticks_msec() < t_end:
		for e in DriveCtx.dump_ui(ctx):
			if e["class"] == "SpinBoxLineEdit" and str(e["text"]).is_valid_int() and e["enabled"]:
				seed_path = e["path"]
				before = e["text"]
				break
		await ctx.wait(0.25)
	_check(ctx, "New World screen shows a seed field", seed_path != "")
	if seed_path != "":
		var got: String = await DriveCtx.type_text(ctx, seed_path, SEED)
		await ctx.wait(0.3)
		var back := ""
		for e in DriveCtx.dump_ui(ctx):
			if e["path"] == seed_path:
				back = str(e["text"])
		_check(ctx, "dump_ui reads the typed seed back", back == "424242", "before=%s typed-return=%s dump=%s" % [before, got, back])
		_check(ctx, "positive control: the field really changed", before != back)
	var n_tip := 0
	var n_off := 0
	for e in DriveCtx.dump_ui(ctx):
		n_tip += 1 if str(e["tooltip"]) != "" else 0
		n_off += 0 if e["enabled"] else 1
	ctx.log("dump_ui: %d visible controls, %d with tooltips, %d disabled" % [DriveCtx.dump_ui(ctx).size(), n_tip, n_off])
	ctx.capture("seed_typed")
	ctx.log("PROOF %s" % ("FAIL: %d check(s)" % _fails if _fails > 0 else "PASS"))
