extends Node
## TEMPORARY, untracked probe. Two leads from the press-every-button sweep:
##
##   1. Place editor ⟳ (re-roll name) changed nothing on screen. Call the
##      engine directly ten times and see whether the name actually moves,
##      and whether the editor's own read-back agrees with it.
##   2. Two enabled, visible buttons with EMPTY text -- one in the Data
##      manager, one in the Asset library -- that changed nothing when
##      pressed. Name them.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _reroll_probe.tscn

var _app: Node
var _bridge


func _p(s: String) -> void:
	print("REROLL  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


func _describe_empty_buttons(title: String, root: Node) -> void:
	var all: Array = []
	_walk(root, all)
	for n in all:
		if n is Button and (n as Button).text.strip_edges() == "" \
				and not (n as Button).disabled and (n as Control).is_visible_in_tree():
			var b := n as Button
			var kids := PackedStringArray()
			var sub: Array = []
			_walk(b, sub)
			for k in sub:
				if k is Label:
					kids.append((k as Label).text)
			## and the labels of the button's own siblings, which is where a
			## row like this usually keeps its caption
			var sibs := PackedStringArray()
			var par := b.get_parent()
			if par != null:
				var ps: Array = []
				_walk(par, ps)
				for k in ps:
					if k is Label:
						sibs.append((k as Label).text)
			_p("%s  EMPTY BUTTON  icon=%s tip='%s' size=%s childLabels=%s siblingLabels=%s" % [
				title, str(b.icon != null), b.tooltip_text, str(b.size),
				str(kids), str(sibs)])


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
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

	# ------------------------------------------------------------------ 1
	var st: Array = _bridge.settlements()
	var best := 0
	var best_pop := -1.0
	for i in st.size():
		var p: float = float(st[i].get("population", 0.0))
		if p > best_pop:
			best_pop = p
			best = i
	_p("re-rolling settlement #%d, currently '%s'" % [best, String(st[best].get("name", "?"))])
	for i in 10:
		var n: String = _bridge.civ_reroll_settlement_name(best)
		var back: Dictionary = _bridge.civ_settlement_details(best)
		var listed: Array = _bridge.settlements()
		_p("  roll %2d: returned='%s'  details.name='%s'  settlements()[%d].name='%s'" % [
			i, n, String(back.get("name", "?")), best, String(listed[best].get("name", "?"))])

	## And through the real button, twice, reading the LineEdit each time.
	var pe = _app.place_editor_window
	pe.open_for(best)
	await _frames(8)
	for i in 3:
		var before: String = pe._name_edit.text
		var roll: Button = null
		var all: Array = []
		_walk(pe, all)
		for n in all:
			if n is Button and (n as Button).text == "⟳":
				roll = n
		if roll == null:
			_p("  no reroll button found")
			break
		roll.pressed.emit()
		await _frames(6)
		_p("  button press %d: '%s' -> '%s'" % [i, before, pe._name_edit.text])
	pe.hide()
	await _frames(3)

	# ------------------------------------------------------------------ 2
	_app.data_manager_window.open()
	await _frames(8)
	_describe_empty_buttons("DataManager", _app.data_manager_window)
	_app.data_manager_window.hide()
	await _frames(3)
	_app.asset_library_window.open()
	await _frames(8)
	_describe_empty_buttons("AssetLibrary", _app.asset_library_window)
	_app.asset_library_window.hide()

	_p("DONE")
	get_tree().quit(0)
