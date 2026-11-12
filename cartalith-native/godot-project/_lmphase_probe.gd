extends Node
## The landmark Inspect page's **Phase** row (M9's temporal state, 2026-11-12), in the real shell: select a
## placed landmark through the one selection path and read the dock's labels. Positive control: a landmark
## in a populated world shows "Phase" with a reason in its tooltip. Negative control: the dock for the same
## landmark in a world with no settlements shows no Phase row at all (absence, not "Natural") -- driven by
## clearing the settlements through `civ_delete_settlement` until none remain.
##
##   godot --path . --resolution 1600x1000 _lmphase_probe.tscn          (windowed, under C:/cw/.win_lock)
## Exit 0 on PASS, 1 on FAIL.

var _fail := 0
var _app: Node
var _vp: SubViewport


func _ok(cond: bool, what: String) -> void:
	if not cond:
		_fail += 1
	print("%s  %s" % ["ok  " if cond else "FAIL", what])


func _frames(n: int) -> void:
	for _k in n:
		await get_tree().process_frame


## Every Label under `n` whose text equals `text`, as a flat list.
func _labels(n: Node, text: String, out: Array) -> void:
	if n is Label and (n as Label).text == text:
		out.append(n)
	for c in n.get_children():
		_labels(c, text, out)


func _ready() -> void:
	DisplayServer.window_set_size(Vector2i(1600, 1000))
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 1000)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	var bridge = _app.bridge
	bridge.generate({"seed": 24601, "width_km": 1200.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	await _frames(6)
	await bridge.landmark_run()
	await _frames(4)
	var lms: Array = bridge.landmarks()
	_ok(lms.size() > 0, "landmarks placed: %d" % lms.size())
	if lms.is_empty():
		get_tree().quit(1)
		return

	_app.select("landmark", 0)
	await _frames(8)
	var found: Array = []
	_labels(_app, "Phase", found)
	_ok(found.size() >= 1, "the Inspect page shows a Phase row (%d label(s))" % found.size())
	if found.size() >= 1:
		var tip := String(((found[0] as Label).get_parent() as Control).tooltip_text) + String((found[0] as Label).tooltip_text)
		_ok("lifecycle" in tip or "Research" in tip or tip != "", "the row carries its reason as a tooltip")
	var want := String(bridge.landmark_phase("%s@%d,%d" % [String(lms[0]["kind"]), int(lms[0]["x"]), int(lms[0]["y"])]).get("phase", "")).capitalize()
	var vals: Array = []
	_labels(_app, want, vals)
	_ok(want != "" and vals.size() >= 1, "the row's value is the bridge's phase (%s)" % want)

	# Negative control: no settlements, no phase row.
	var n: int = bridge.settlements().size()
	for i in range(n - 1, -1, -1):
		bridge.civ_delete_settlement(i)
	await bridge.landmark_run()
	await _frames(4)
	_app.select("landmark", 0)
	await _frames(8)
	var none: Array = []
	_labels(_app, "Phase", none)
	_ok(bridge.settlements().is_empty(), "every settlement is gone")
	_ok(none.is_empty(), "with no settlement the Inspect page shows no Phase row (%d)" % none.size())
	print("LMPHASE  %s  (%d failure(s))" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(0 if _fail == 0 else 1)
