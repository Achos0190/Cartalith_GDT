extends Node
## VERIFIER probe, 2026-09-05, Lane C. Two questions the lane's own probe
## answers a slightly easier version of.
##
## 1. **A generalised mechanism must be tested against a domain that does not
##    exist yet.** `_leftdock12_probe.gd` §7 calls `_rebuild_mode_switch()`
##    DIRECTLY, which is the rebuild path but not the gate path -- it never
##    passes through `_refresh_mode_switch()`'s own decision to show the pill
##    for this domain. This probe drives the NORMAL user route
##    (`select_domain_mode`) and reads the pill back, so it works either way:
##    run once against the shipping table (CARTO ungated -> no pill) and once
##    with a `shows` added to a CARTO node (CARTO gated -> CARTO's own four
##    halves, and never WORLD's two). The second run is the real test.
##
## 2. **The floor.** An ungated dock must now be leavable with everything
##    closed; a gated one must not. Both directions, and the ungated case is
##    the one that used to fail.
##
##   Godot..._console.exe --path . _vfyb13c_probe.tscn -- --vp 1920x1080

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(label: String, got, want) -> void:
	var pass_ := str(got) == str(want)
	print("VC %s  %s  got=%s want=%s" % ["ok  " if pass_ else "FAIL", label, got, want])
	if not pass_:
		_fail += 1

func _body(panel: Control, title: String) -> Control:
	for e in (panel.get("categories") as Array):
		if String(e["title"]) == title:
			return e["body"]
	return null

func _rendered(panel: Control) -> Array:
	var out: Array = []
	for e in (panel.get("categories") as Array):
		var w := (e["body"] as Control).get_parent() as Control
		if w != null and w.visible:
			out.append(String(e["title"]))
	return out

func _open_count(panel: Control) -> int:
	var n := 0
	for e in (panel.get("categories") as Array):
		if (e["body"] as Control).visible:
			n += 1
	return n

func _pill_texts() -> String:
	var pill: HBoxContainer = app.get("_mode_switch_pill")
	var t: Array = []
	for c in pill.get_children():
		t.append((c as Button).text)
	return ", ".join(t)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	var ua := OS.get_cmdline_user_args()
	for i in ua.size():
		if String(ua[i]) == "--vp" and i + 1 < ua.size():
			var wh := String(ua[i + 1]).split("x")
			DisplayServer.window_set_size(Vector2i(int(wh[0]), int(wh[1])))
			await _frames(6)
			app._compute_layout_mode()
			DccTheme.set_phone(app._phone)
			await _frames(2)

	## The live gate table, printed so the two runs are distinguishable in the
	## log rather than by which command produced them.
	var gates: Array = []
	for n in app.get("RAIL_NODES"):
		if String(n.get("kind", "")) != "node":
			continue
		if not (n.get("shows", []) as Array).is_empty():
			gates.append("%s/%s" % [String(n["domain"]), String(n["mode"])])
	print("VC banner: gated nodes = [%s]  carto_gates=%s"
		% [", ".join(gates), app.call("domain_gates", "cartography")])

	var world: Control = app.call("workspace_panel", "world")
	var carto: Control = app.call("workspace_panel", "cartography")
	var row: Control = app.get("_mode_switch_row")

	# ================= 1. the pill, by the NORMAL user route ==================
	## Labels first, derived independently of the pill: these are RAIL_NODES'
	## own `label` values upper-cased, except WORLD's two, which are the
	## recorded override.
	_ok("label world/a", DccShell.mode_switch_label("world", "a"), "PIPELINE")
	_ok("label world/b", DccShell.mode_switch_label("world", "b"), "SCULPT")
	_ok("label carto/style", DccShell.mode_switch_label("cartography", "style"),
		"LAYERS & STYLE")
	_ok("label civ/factions", DccShell.mode_switch_label("civilization", "factions"),
		"FACTIONS & SETTLEMENTS")

	app.call("select_domain_mode", "world", "a")
	await _frames(3)
	_ok("WORLD: pill visible", row.visible, true)
	_ok("WORLD: its own two halves", _pill_texts(), "PIPELINE, SCULPT")
	_ok("WORLD: recorded domain", app.get("_mode_switch_domain"), "world")

	## CARTO by the ordinary route. Ungated today -> no pill. Gated (the mutated
	## run) -> CARTO's OWN four halves. The refutation this exists to catch is
	## a visible pill still wearing "PIPELINE, SCULPT".
	app.call("select_domain_mode", "cartography", "style")
	await _frames(3)
	var carto_gates: bool = app.call("domain_gates", "cartography")
	print("VC   CARTO selected: pill.visible=%s texts=[%s] domain=%s"
		% [row.visible, _pill_texts(), app.get("_mode_switch_domain")])
	_ok("CARTO: pill visible iff CARTO gates", row.visible, carto_gates)
	if carto_gates:
		_ok("CARTO gated: the pill is CARTO's, not WORLD's", _pill_texts(),
			"LAYERS & STYLE, LABELS, ICONS, TERRAIN APPEARANCE")
		_ok("CARTO gated: recorded domain", app.get("_mode_switch_domain"), "cartography")
		## And a half presses to CARTO, not to WORLD.
		var btns: Dictionary = app.get("_mode_switch_buttons")
		(btns["labels"] as Button).pressed.emit()
		await _frames(3)
		_ok("CARTO gated: a half presses its own domain",
			app.call("active_domain"), "cartography")
		_ok("CARTO gated: ...in its own mode", app.call("active_mode", "cartography"),
			"labels")

	# ================= 2. the floor ==========================================
	app.call("select_domain_mode", "world", "a")
	await _frames(3)
	_ok("floor: WORLD a is UNGATED (precondition)",
		(app.call("mode_shows", "world", "a") as Array).is_empty(), true)
	var open_btn: Button = null
	var open_title := "Generate"
	for e in (world.get("categories") as Array):
		if (e["body"] as Control).visible:
			open_btn = e["button"]
			open_title = String(e["title"])
	_ok("floor: exactly one header open in WORLD a", _open_count(world), 1)
	if open_btn != null:
		open_btn.pressed.emit()
		await _frames(3)
	_ok("floor: an UNGATED dock can be left with everything closed",
		_open_count(world), 0)

	## And the gated dock still cannot. Driven through `apply_domain_mode`, not
	## `select_domain_mode`, which opens the node's own category and would
	## report green with no floor at all.
	app.call("apply_domain_mode", "world", "b")
	await _frames(3)
	_ok("floor: entering the GATED mode still floors the dock",
		_body(world, "Terrain").visible, true)
	_ok("floor: ...on the gated block only", ", ".join(_rendered(world)), "Terrain")

	## CARTO: ungated in the shipping table, so it must close to zero. In the
	## mutated run CARTO's `style` mode gates, so the floor legitimately applies
	## and this expectation flips -- asserted against the live gate, not a
	## constant, so the same probe is correct in both runs.
	app.call("select_domain_mode", "cartography", "style")
	await _frames(3)
	var copen: Button = null
	for e in (carto.get("categories") as Array):
		if (e["body"] as Control).visible:
			copen = e["button"]
	if copen != null:
		copen.pressed.emit()
		await _frames(3)
	var carto_mode_gates: bool = \
		not (app.call("mode_shows", "cartography", app.call("active_mode", "cartography")) as Array).is_empty()
	_ok("floor: CARTO closes to zero iff its active mode does not gate",
		_open_count(carto) == 0, not carto_mode_gates)

	print("VC DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
