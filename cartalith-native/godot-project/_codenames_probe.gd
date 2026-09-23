extends Node
## Windowed census of developer code names in VISIBLE label text on each
## domain's default panels (`OUTSTANDING_WORK.md` §2.10, "Developer code names
## are shown to users"). Counts labels and button text, not tooltips; PASS
## when no default panel shows one.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _codenames_probe.tscn

var _re := RegEx.new()
var _hits: Array = []

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _walk(n: Node, where: String) -> void:
	for c in n.get_children():
		if c is Label and (c as Label).is_visible_in_tree():
			var tx := (c as Label).text
			if _re.search(tx) != null:
				_hits.append("%s | %s" % [where, tx.replace("\n", " ").substr(0, 140)])
		if c is Button and (c as Button).is_visible_in_tree():
			var bt := (c as Button).text
			if _re.search(bt) != null:
				_hits.append("%s | [button] %s" % [where, bt.substr(0, 140)])
		_walk(c, where)

func _ready() -> void:
	## `.gd`/`.rs` file names, `::` paths, and snake_case identifiers followed
	## by `(` -- the shapes a developer name takes in prose.
	_re.compile(r"\.gd\b|\.rs\b|::|\b[a-z]+_[a-z0-9_]+\(")
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge: Node = app.bridge
	bridge.generate({"seed": 9137, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)
	var shell: Node = app  ## app.gd extends DccShell
	for dom in ["world", "civilization", "cartography"]:
		if shell != null:
			shell.select_domain(dom)
		await _frames(10)
		_walk(app, dom)
	var seen := {}
	for h in _hits:
		if not seen.has(h):
			seen[h] = true
			print("CODENAME ", h)
	print("CODENAME total distinct %d" % seen.size())
	## 2026-09-24: 4 before the fix (the world progress note, the CIVIL and
	## CARTO default tool-option lines, the right dock's ramp note), 0 after.
	print("CODENAME %s" % ("PASS" if seen.is_empty() else "FAIL"))
	get_tree().quit(0 if seen.is_empty() else 1)
