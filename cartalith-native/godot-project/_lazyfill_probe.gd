extends Node
## `OUTSTANDING_WORK.md`, "Every generate freezes the main thread after it
## finishes": CIVIL's Economy / Military / Relationships and WORLD's Ecology
## docks no longer refill on `generation_finished` while their category is
## closed; the refill (`DccWidgets.fill_when_visible`) runs the moment the
## category is opened. This probe protects three things:
##   1. after a generate, those four bodies hold a PENDING fill (the O(grid)
##      Rust call was skipped), not a finished one;
##   2. opening each category draws the real content (a faction name, a pair
##      row, the NPP line) and clears the pending fill -- the output is the
##      same as an eager fill, only later;
##   3. a second generate while a category is open refills at once, and one
##      while it is closed again leaves a pending fill rather than stale text
##      being drawn.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _lazyfill_probe.tscn

var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("LAZYFILL %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1
		print("PROBE-FAIL ", what)

func _texts(node: Node, out: PackedStringArray) -> void:
	if node is Label:
		out.append((node as Label).text)
	elif node is RichTextLabel:
		out.append((node as RichTextLabel).get_parsed_text())
	elif node is Button:
		out.append((node as Button).text)
	for c in node.get_children():
		_texts(c, out)

func _text(body: Control) -> String:
	var out := PackedStringArray()
	_texts(body, out)
	return "\n".join(out)

func _generate(app: Node, seed_v: int) -> void:
	app.bridge.generate({"seed": seed_v, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while app.bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(6)

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("LAZYFILL REFUSED: headless")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 250.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("LAZYFILL WATCHDOG"); get_tree().quit(3))
	wd.start()

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	await _generate(app, 9137)
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(4)

	var civ: Node = app.workspace_panel("civilization")
	var world: Node = app.workspace_panel("world")
	_check("both workspaces are registered", civ != null and world != null)

	# 1. closed categories hold a pending fill after the generate.
	app.select_domain("civilization")
	await _frames(4)
	var cases := [
		[civ, "Economy", "_economy_body"],
		[civ, "Military", "_military_body"],
		[civ, "Relationships", "_relations_body"],
		[world, "Ecology", "_ecology_body"],
	]
	await _generate(app, 4242)
	for c in cases:
		var body: Control = c[0].get(c[2])
		_check("%s: pending after a generate while closed" % c[1], DccWidgets.has_pending_fill(body))

	# 2. opening the category draws the content and runs the fill.
	for c in cases:
		var domain := "civilization" if c[0] == civ else "world"
		app.select_domain_category(domain, c[1])
		await _frames(3)
		var body: Control = c[0].get(c[2])
		_check("%s: nothing pending once opened" % c[1], not DccWidgets.has_pending_fill(body))
		var t := _text(body)
		_check("%s: the body has content" % c[1], t.length() > 40, t.left(80))
		if c[1] == "Economy":
			var fb: Control = civ.get("_economy_faction_body")
			_check("Economy: the By-faction body is filled too", _text(fb).length() > 20)
		if c[1] == "Ecology":
			_check("Ecology: names net primary productivity", t.contains("Net primary productivity"), t.left(120))

	# 3. a generate while a category is open refills at once.
	app.select_domain_category("civilization", "Relationships")
	await _frames(3)
	await _generate(app, 777)
	_check("open Relationships refilled at once (nothing pending)",
		not DccWidgets.has_pending_fill(civ.get("_relations_body")))
	_check("and its Economy sibling category, closed, is pending again",
		DccWidgets.has_pending_fill(civ.get("_economy_body")))

	print("LAZYFILL RESULT %s" % ("PASS" if _fails == 0 else "FAIL (%d)" % _fails))
	get_tree().quit(0 if _fails == 0 else 1)
