extends Node
## VERIFIER, 2026-09-05. Lane A landed `"bridges"`/`"ford"` on
## `urban_layouts()`. This probe asks the question the ×48 preflight row asks:
## does any prose still describe the OLD behaviour -- and in particular, is
## there a dashed field whose stated reason the same run disproves?
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _vfy_bridgedash_probe.tscn
##
## Nothing here is palette-bound: every assertion is a dictionary key, an
## integer, or a substring of a Label/tooltip.

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ck(name: String, cond: bool, detail: String = "") -> void:
	print("  %s %s%s" % ["ok  " if cond else "FAIL", name,
		("   -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _labels(n: Node, out: Array = []) -> Array:
	for c in n.get_children():
		if c is Label:
			out.append(c)
		_labels(c, out)
	return out

func _ready() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	DccTheme.apply_theme(false)   ## force LIGHT

	app.bridge.generate({"seed": 40417, "width_km": 2000.0, "grid_w": 256,
		"grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while app.bridge.generating and waited < 6000:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	var n_set: int = app.bridge.settlements().size()
	_ck("world generated", app.bridge.has_world, "%d settlements" % n_set)

	print("[1] what urban_layouts() now answers")
	var idx := PackedInt32Array()
	for i in n_set:
		idx.append(i)
	var got: Array = app.bridge.urban_layouts(idx)
	_ck("layouts returned", got.size() > 0, "%d layouts for %d settlements" % [got.size(), n_set])
	var missing_key := 0
	var with_bridges := 0
	var total_bridges := 0
	var with_ford := 0
	var best := -1
	var best_n := 0
	for l in got:
		var d := l as Dictionary
		if not d.has("bridges"):
			missing_key += 1
			continue
		var b: PackedVector2Array = d["bridges"]
		var dirs: PackedVector2Array = d.get("bridge_dirs", PackedVector2Array())
		if b.size() != dirs.size():
			_ck("bridges/bridge_dirs are parallel", false, "%d vs %d" % [b.size(), dirs.size()])
		if b.size() > 0:
			with_bridges += 1
			total_bridges += b.size()
			if b.size() > best_n:
				best_n = b.size()
				best = int(d.get("index", -1))
		if d.has("ford"):
			with_ford += 1
	_ck("`bridges` is present on EVERY layout (always-present rule)", missing_key == 0,
		"%d of %d missing the key" % [missing_key, got.size()])
	_ck("some settlement really has a road crossing", with_bridges > 0,
		"%d towns with bridges, %d spans total, %d with a ford; busiest = index %d with %d"
			% [with_bridges, total_bridges, with_ford, best, best_n])

	print("[2] right_dock.gd's `Bridges` field -- same dictionary, still dashed")
	if best < 0:
		_ck("a settlement with bridges was found to inspect", false)
	else:
		var settle: Dictionary = app.bridge.settlements()[best]
		app.right_dock_ctrl.on_settlement_selected(settle, best)
		await _frames(6)
		var found := {}
		for l in _labels(app.right_dock_ctrl):
			var t := String((l as Label).text)
			if t == "Bridges":
				var par := (l as Label).get_parent() as Control
				var vals := []
				for s in _labels(par):
					vals.append(String((s as Label).text))
				found = {"vals": vals, "tip": par.tooltip_text}
				break
		_ck("the `Bridges` field is drawn in the right dock", not found.is_empty(),
			str(found.get("vals", [])))
		var vals: Array = found.get("vals", [])
		var joined := " | ".join(PackedStringArray(vals))
		var tip := String(found.get("tip", ""))
		_ck("it still reads as a dash, though the dictionary beside it holds %d span(s)" % best_n,
			joined.find("—") != -1, joined)
		_ck("its tooltip still says the adapter does not carry the field", 
			tip.find("does not carry that field through") != -1,
			tip.substr(0, 260))

	print("[3] civilization_workspace.gd's third line")
	var cw = app.get_node_or_null("%s" % "") # placeholder, resolved below
	_ck("the shipped source still draws `not surfaced by any binding yet`",
		FileAccess.get_file_as_string("res://shell/workspaces/civilization_workspace.gd")
			.find("bridge/ford: — not surfaced by any binding yet") != -1)

	print("=== %d FAILED (a FAIL here is the verifier's finding, not a broken probe) ===" % _fail)
	get_tree().quit(0)
