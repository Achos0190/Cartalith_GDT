extends Node
## **`wantCounts` -- the reference's fixed per-tier settlement counts -- end to
## end, through the New World dialog's own widgets.**
##
## 1. Generates three default worlds and hashes each one's its settlement list (x, y, kind,
##    name, population, faction, seat, port) and road count, then re-runs
##    CIVIL ▸ Auto-populate (`civ_populate`) and hashes again. Those two hashes
##    are the flag-OFF output; run this probe against the pre-change DLL too and
##    they must match (it prints them whether or not the new rows exist).
## 2. If the engine publishes `civ.fixed_counts`: opens the dialog, switches
##    the toggle on and types 2/3/5/8/6 into the five count boxes BY SETTING THE
##    SPINBOXES (so the dialog's own `value_changed` handlers do the
##    `param_set`), asserts the two dials went inert, screenshots, then
##    Auto-populates and asserts the tier histogram is exactly the request.
## 3. Switches the toggle off through the widget and Auto-populates again: the
##    hash must equal step 1's populate hash, byte for byte.
##
## Run WINDOWED (pixels are read for the screenshots):
##   godot --path . _wantcounts_probe.tscn
## Reads no command-line flags.

const WANT := [2, 3, 5, 8, 6]
const KINDS := ["capital", "city", "town", "village", "hamlet"]
const SHOT_DIR := "user://wantcounts_probe"

var _app: Node
var _fail := 0

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	print("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _hash(bridge) -> String:
	var parts: PackedStringArray = []
	for s in bridge.settlements():
		parts.append("%d,%d,%s,%s,%d,%d,%s,%s" % [s.x, s.y, s.kind, s.name, s.population,
			s.faction, str(s.capital), str(s.coastal)])
	parts.append("roads=%d" % bridge.roads().size())
	return "\n".join(parts).md5_text()

func _hist(bridge) -> Array:
	var h := [0, 0, 0, 0, 0, 0]  ## + "other" (metropolis)
	for s in bridge.settlements():
		var i := KINDS.find(String(s.kind))
		h[i if i >= 0 else 5] += 1
	return h

func _shot(name: String) -> void:
	await RenderingServer.frame_post_draw
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(SHOT_DIR))
	var path := ProjectSettings.globalize_path("%s/%s.png" % [SHOT_DIR, name])
	get_viewport().get_texture().get_image().save_png(path)
	print("  SHOT %s" % path)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	## Tall enough that the dialog's Generation section and its note fit.
	DisplayServer.window_set_size(Vector2i(1280, 1000))
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge

	## Three worlds for the flag-off hashes (one world is one sample); the
	## counts steps below run on the last of them.
	var r: Dictionary
	var off_hash := ""
	for seed in [12345, 4242, 77021]:
		bridge.generate({
			"seed": seed, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
			"archetype": "", "villages": false, "sea_level": 0.45,
		})
		while bridge.generating:
			await get_tree().create_timer(0.25).timeout
		await get_tree().create_timer(0.5).timeout
		print("GENERATE_HASH seed %d %s  hist C/Ci/T/V/H/other %s  n=%d" % [seed, _hash(bridge), str(_hist(bridge)), bridge.settlements().size()])
		r = bridge.civ_populate()
		_check(bool(r.get("ok", false)), "default Auto-populate ran (%s)" % str(r.get("reason", "")))
		off_hash = _hash(bridge)
		print("POPULATE_HASH seed %d %s  hist %s" % [seed, off_hash, str(_hist(bridge))])

	if bridge.param_info("civ.fixed_counts").is_empty():
		print("NO civ.fixed_counts ROW -- pre-change engine; hashes above are the baseline")
		get_tree().quit(0)
		return

	var dlg = _app.new_world_dialog
	dlg.popup_centered()
	await get_tree().create_timer(0.5).timeout
	_check(dlg._count_toggle != null and dlg._count_boxes.size() == 5, "dialog built the toggle and five count boxes")
	_check(not dlg._count_boxes[0].editable, "counts are read-only while the toggle is off")
	var thr_slider: HSlider = dlg._count_dials[0]["slider"]
	_check(thr_slider.editable, "suitability floor live while counts are off")
	dlg._count_toggle.button_pressed = true
	for i in 5:
		dlg._count_boxes[i].value = WANT[i]
	await get_tree().process_frame
	_check(bridge.param_get("civ.fixed_counts") == true, "toggle wrote civ.fixed_counts")
	var stored := []
	for k in ["civ.n_capital", "civ.n_city", "civ.n_town", "civ.n_village", "civ.n_hamlet"]:
		stored.append(int(bridge.param_get(k)))
	_check(stored == WANT, "count boxes wrote the five rows: %s" % str(stored))
	_check(not thr_slider.editable and not dlg._count_dials[1]["slider"].editable, "both dials inert while counts are active")
	print("  NOTE '%s'" % dlg._count_note.text)
	## Bring the section into view in the dialog's own scroll.
	var n: Node = dlg._count_toggle
	while n != null and not (n is ScrollContainer):
		n = n.get_parent()
	if n != null:
		var sc := n as ScrollContainer
		sc.scroll_vertical = int(sc.get_v_scroll_bar().max_value)
	await get_tree().create_timer(0.3).timeout
	await _shot("dialog_counts_on")
	dlg.hide()

	r = bridge.civ_populate()
	_check(bool(r.get("ok", false)), "fixed-count Auto-populate ran")
	var h := _hist(bridge)
	print("COUNTS_HIST C/Ci/T/V/H/other %s  requested %s" % [str(h), str(WANT)])
	_check(h.slice(0, 5) == WANT and h[5] == 0, "tier histogram is exactly the request")
	## The start screen is an exclusive window left over from launch; it
	## covers the map, so close every window the app has open.
	for w in _app.find_children("*", "Window", true, false):
		(w as Window).hide()
	await get_tree().create_timer(0.5).timeout
	await _shot("map_counts_on")

	## Off again, through the widget, and the default output must come back.
	dlg._count_toggle.button_pressed = false
	await get_tree().process_frame
	_check(thr_slider.editable, "dials live again once counts are off")
	r = bridge.civ_populate()
	var back := _hash(bridge)
	print("POPULATE_HASH_AFTER_OFF %s" % back)
	_check(back == off_hash, "counts off reproduces the default Auto-populate byte for byte")

	## Leave nothing behind for the next probe.
	for i in 5:
		dlg._count_boxes[i].value = 0
	print("RESULT %s (%d failures)" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(0 if _fail == 0 else 1)
