extends Node
## TEMPORARY, untracked probe -- the cross-domain "→ Civilization ▸ Territories"
## jump buttons, which until 2026-08-25 switched the rail and stopped there.
##
## For each one: press it for real, then assert BOTH halves -- the domain is the
## one the label names, and the category the label names is the open one.
## Also re-asserts IN-01's river note and sweeps every rendered string for the
## nine category names v3 retired.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _jump_probe.tscn

var _app: Node
var _bridge
var _fail := 0

## Retired by the v3 pass. A rendered string naming one of these as a
## destination is a stale pointer. `Roads`/`Trade`/`Rivers`/`Politics` are
## excluded from the raw-word scan -- they are ordinary English here.
const RETIRED_PATHS := [
	"Roads ▸", "Ports ▸", "Logistics ▸", "Rivers ▸", "Population ▸",
	"Timeline ▸ ", "World ▸ Sculpt", "WORLD ▸ Finalize",
	"Generation Pipeline", "Politics ▸ Recalculate",
]


func _p(s: String) -> void:
	print("JUMP  %s" % s)


func _bad(s: String) -> void:
	_fail += 1
	print("JUMP  FAIL  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


func _find_button(root: Node, exact_text: String) -> Button:
	var all: Array = []
	_walk(root, all)
	for n in all:
		if n is Button and (n as Button).text == exact_text:
			return n
	return null


func _open_category(ws) -> String:
	for e in ws.categories:
		if (e["body"] as Control).visible:
			return String(e["title"])
	return "<none>"


func _all_text() -> Array:
	var all: Array = []
	_walk(_app, all)
	var out: Array = []
	for n in all:
		if n is Label:
			out.append((n as Label).text)
		elif n is RichTextLabel:
			out.append((n as RichTextLabel).get_parsed_text())
		elif n is Button:
			out.append((n as Button).text)
		if n is Control and (n as Control).tooltip_text != "":
			out.append((n as Control).tooltip_text)
	return out


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	_bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)

	var panels: Dictionary = _app._workspace_panels
	var carto = panels["cartography"]
	var civ = panels["civilization"]

	var cases := [
		["cartography", carto, "Draw and edit ways → Civilization ▸ Routes & ways", "civilization", "Routes & ways"],
		["cartography", carto, "Edit territories → Civilization ▸ Territories", "civilization", "Territories"],
		["civilization", civ, "Place an icon → Cartography ▸ Assets & landmarks", "cartography", "Assets & landmarks"],
	]
	for c in cases:
		_app.select_domain(String(c[0]))
		await _frames(4)
		var b := _find_button(c[1], String(c[2]))
		if b == null:
			_bad("button not found: '%s'" % c[2])
			continue
		b.pressed.emit()
		await _frames(6)
		var dom: String = _app.active_domain()
		var cat := _open_category(panels[String(c[3])])
		var ok := dom == String(c[3]) and cat == String(c[4])
		_p("%s  '%s' -> domain=%s open=%s (want %s / %s)" % [
			"PASS " if ok else "FAIL ", c[2], dom, cat, c[3], c[4]])
		if not ok:
			_fail += 1

	## The timeline strip's own Open Timeline, which routes through the same
	## lookup now.
	_app.select_domain("civilization")
	await _frames(4)
	var tl := _find_button(_app.timeline_row, "Open Timeline")
	if tl == null:
		_bad("Open Timeline button not found in the timeline strip")
	else:
		tl.pressed.emit()
		await _frames(6)
		var cat2 := _open_category(civ)
		if cat2 == "Politics":
			_p("PASS  Open Timeline -> CIVIL ▸ Politics")
		else:
			_bad("Open Timeline opened '%s', not Politics" % cat2)

	# -------------------------------------------------------------- IN-01
	_app.select_domain("world")
	await _frames(4)
	var found := false
	for t in _all_text():
		if String(t).find("No hydrological river entity is exposed to Godot") >= 0:
			found = true
	if found:
		_p("PASS  IN-01 river note present in WORLD")
	else:
		_bad("IN-01 river note missing from WORLD")

	# ------------------------------------------------- retired-name sweep
	## Open every category in all three rails first -- a closed accordion body
	## still has its Labels in the tree, but tooltips on lazily-built rows do
	## not exist until the row does.
	for dom in ["world", "civilization", "cartography"]:
		_app.select_domain(dom)
		await _frames(2)
		var ws = panels[dom]
		for e in ws.categories:
			(e["button"] as Button).pressed.emit()
			await _frames(1)
	await _frames(4)

	var hits := {}
	for t in _all_text():
		var s := String(t)
		for r in RETIRED_PATHS:
			if s.find(r) >= 0:
				hits[r] = s.substr(maxi(0, s.find(r) - 40), 130)
	if hits.is_empty():
		_p("PASS  no retired category name is named as a destination anywhere")
	else:
		for k in hits:
			_bad("retired pointer '%s' still rendered: …%s…" % [k, hits[k]])

	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
