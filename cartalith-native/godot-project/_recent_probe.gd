extends Node
## TEMPORARY, untracked probe. One question: does a `File ▸ Recent worlds` row
## actually load the world it names, and does it say anything when it cannot?
## `_menuwire_probe.gd` reported both rows as "changed nothing anywhere", which
## is either a dead menu row or a probe artifact, and the two are worth
## separating.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _recent_probe.tscn

var _app: Node
var _bridge


func _p(s: String) -> void:
	print("RECENT  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


func _status() -> String:
	var all: Array = []
	_walk(_app, all)
	var out := PackedStringArray()
	for n in all:
		if n is Label and (n as Label).text.strip_edges() != "":
			out.append((n as Label).text)
	return " | ".join(out)


func _popup(named: String) -> PopupMenu:
	var all: Array = []
	_walk(_app, all)
	for n in all:
		if n is PopupMenu and (n as Node).name == named:
			return n
	return null


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 400.0
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

	var rw := _popup("RecentWorlds")
	if rw == null:
		_p("no RecentWorlds popup")
		get_tree().quit(1)
		return
	## The recent list is refilled from the FILE menu's own `about_to_popup`,
	## not the submenu's -- firing it on `RecentWorlds` alone leaves the list
	## empty and the probe reports "0 recent rows" over a real history.
	var pops: Array = []
	_walk(_app, pops)
	for n in pops:
		if n is PopupMenu:
			(n as PopupMenu).about_to_popup.emit()
	await _frames(4)
	_p("%d recent rows" % rw.item_count)
	for i in rw.item_count:
		if rw.is_item_separator(i):
			continue
		_p("  row %d '%s' disabled=%s tip='%s'" % [
			i, rw.get_item_text(i), str(rw.is_item_disabled(i)),
			rw.get_item_tooltip(i).substr(0, 60)])

	for i in rw.item_count:
		if rw.is_item_separator(i) or rw.is_item_disabled(i):
			continue
		var label := rw.get_item_text(i)
		## Put a known, different world up first, so "nothing changed" cannot be
		## "it loaded the world that was already there".
		_bridge.generate({
			"seed": 24601, "width_km": 700.0, "grid_w": 192, "grid_h": 144,
			"archetype": "", "villages": true, "sea_level": 0.45,
		})
		while _bridge.generating:
			await get_tree().create_timer(0.25).timeout
		await get_tree().create_timer(0.6).timeout
		var g0 = _bridge.grid_size()
		var n0: int = _bridge.settlements().size()
		var st0 := _status()
		rw.id_pressed.emit(rw.get_item_id(i))
		await get_tree().create_timer(2.0).timeout
		await _frames(6)
		var g1 = _bridge.grid_size()
		var n1: int = _bridge.settlements().size()
		var st1 := _status()
		_p("'%s': grid %s -> %s, settlements %d -> %d, project='%s'" % [
			label, str(g0), str(g1), n0, n1, _app.current_project_path])
		if g0 == g1 and n0 == n1:
			_p("   >> loaded nothing. status changed: %s" % str(st0 != st1))
			if st0 != st1:
				for part in st1.split(" | "):
					if st0.find(part) < 0:
						_p("   new status text: %s" % part)
		else:
			_p("   >> loaded a different world, as claimed")

	_p("DONE")
	get_tree().quit(0)
