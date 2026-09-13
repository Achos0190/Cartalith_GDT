extends Node
## Geometry witness for the TILECACHE lane's fix (`invalidate_lod_tiles()` in
## `viewport_host.gd`, called from `world_workspace.gd::_on_sculpt_commit()`).
## Neither touched file builds or resizes a Control, so this dumps the WORLD
## dock's own control tree -- position, size, class, in stable tree order --
## AFTER driving a full stroke+commit cycle through the real sculpt-commit
## path, so the comparison covers the exact call this lane changed. Run once
## against this tree and once against a HEAD-content scratch copy of the same
## two files; the two dumps must be byte-identical.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _tcworlddock_probe.tscn
##
## Headless is fine here -- this reads Control rects from the scene tree, not
## the framebuffer (`MISTAKES.md`'s pixel-probe rule is about the latter).

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## Anonymous nodes' auto-names (`@MarginContainer@1493`) embed a counter that
## is global across the WHOLE scene tree, not just this dock -- so its value
## drifts between two separate process runs for reasons that have nothing to
## do with WORLD dock structure (how many other anonymous nodes booted
## earlier elsewhere in the app). Comparing paths raw would make this probe
## noisy regardless of the fix under test; blanking the counter keeps the
## STRUCTURAL shape (which class, how deep) while dropping the run-to-run
## noise. A node with an explicit `.name` (no "@") is left untouched.
func _norm_segment(seg: String) -> String:
	if seg.begins_with("@"):
		var last_at := seg.rfind("@")
		if last_at > 0:
			return seg.substr(0, last_at + 1) + "N"
	return seg

func _norm_path(path: String) -> String:
	var parts := path.split("/")
	for i in parts.size():
		parts[i] = _norm_segment(parts[i])
	return "/".join(parts)

func _dump(n: Node, root: Node, out: Array) -> void:
	if n is Control:
		var c := n as Control
		var path := _norm_path(str(root.get_path_to(n)))
		out.append("%s | %s | pos=%s | size=%s | vis=%s" % [
			path, c.get_class(), c.position, c.size, c.visible])
	for ch in n.get_children():
		_dump(ch, root, out)

## A fixed frame count is a guess, and a fresh scratch copy (first-ever boot
## under a never-used `config/name`, its own `.godot` import cache) settles
## slower than a warm one -- `Container` layout sort is itself deferred a
## frame, so a dump taken too early reads every un-sorted child's position as
## its construction-time (0,0). Waits until two dumps 3 frames apart agree,
## rather than asserting a frame count that only held for one run of it.
func _settled_dump(ws: Node, max_tries: int = 60) -> Array:
	var prev: Array = []
	var first := true
	for i in max_tries:
		await _frames(3)
		var cur: Array = []
		_dump(ws, ws, cur)
		if not first and cur == prev:
			return cur
		prev = cur
		first = false
	print("TCWD WARN never settled after %d tries -- dumping anyway" % max_tries)
	return prev

func _ready() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] the .gdextension did not load.")
		get_tree().quit(2)
		return

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(30)
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(10)

	var bridge = app.bridge
	bridge.generate({
		"seed": 24601, "width_km": 640.0, "grid_w": 96, "grid_h": 64,
		"sea_level": 0.5, "villages": true,
	})
	await bridge.generation_finished
	await _frames(10)
	if not bridge.has_world:
		print("[ABORT] no world -- nothing else here can run")
		get_tree().quit(2)
		return

	app.select_domain("world")
	app.arm_tool("sculpt")
	await _frames(6)

	var ws = app._world_workspace()
	if ws == null:
		print("[ABORT] no WorldWorkspace")
		get_tree().quit(2)
		return

	## Drive the exact path this lane changed, so the dump covers the dock
	## state that path actually leaves behind, not just its state at rest.
	bridge.sculpt_begin_stroke()
	bridge.sculpt_add_point(40.0, 26.0)
	bridge.sculpt_add_point(44.0, 30.0)
	bridge.sculpt_end_stroke()
	await _frames(3)
	if bridge.sculpt_stamp_count() > 0:
		ws._on_sculpt_commit()

	var out: Array = await _settled_dump(ws)
	print("TCWD control_count=%d" % out.size())
	for line in out:
		print("TCWD ", line)
	print("TCWD DONE")
	get_tree().quit(0)
