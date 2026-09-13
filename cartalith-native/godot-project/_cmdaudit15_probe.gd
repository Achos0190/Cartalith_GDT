extends Node
## **The authoritative re-open of "15 menu commands unavailable, each carrying
## a true reason"** (`OUTSTANDING_WORK.md`, owned by `STATUS.md`, re-cut
## 2026-09-03).
##
## Rather than re-deriving each `_todo()` guard's truth value by reading
## `menus.gd` (`_cmdblock15_probe.gd` did that, symbol by symbol, and found 11
## of 15 guards now satisfied against the loaded `.dll`), this probe asks the
## SAME question the app itself asks: it boots `app.tscn`, builds a
## `CommandIndex` off the live shell exactly as `dcc_shell.gd::
## _ensure_command_index()` does in production (`idx.build(self,
## _find_engine_bridge())`, called here through the shell's own method rather
## than re-implemented), and counts `available == false` rows -- the same
## count the 2026-09-03 figure ("374 total, 15 unavailable, 15 of 15 with a
## reason") was itself produced from. `CommandIndex._walk_popup()` reads each
## row exactly as `menus.gd` left it after every `_refresh_*` that runs at
## build time, so this is not a second opinion, it is the first one asked
## again today.
##
## Every unavailable row is printed with its group, title and reason, so a
## reader can compare this run's list against the row's own text line by line
## rather than trusting a bare count.
##
##   godot --headless --resolution 1920x1080 _cmdaudit15_probe.tscn

func _log(s: String) -> void:
	print("[cmdaudit15] %s" % s)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	var vp := SubViewport.new()
	vp.size = Vector2i(1920, 1080)
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(6)

	var shell: Node = app.get("shell")
	if shell == null:
		shell = app
	_log("is_phone=%s (this walk is form-factor-independent: all three menu"
		% shell.call("is_phone")
		+ " layouts share the same PopupMenu content, only the MenuButton count differs)")

	var idx: CommandIndex = shell.call("_ensure_command_index")
	var rows: Array = idx.all()
	var unavailable: Array = []
	for r in rows:
		if not bool(r["available"]):
			unavailable.append(r)
	unavailable.sort_custom(func(a, b): return String(a["group"] + a["title"]) \
		< String(b["group"] + b["title"]))

	_log("total rows: %d" % rows.size())
	var with_reason := 0
	for r in unavailable:
		var why := String(r["why"])
		if why.strip_edges() != "":
			with_reason += 1
		_log("  UNAVAILABLE [%s] %s -- %s" % [r["group"], r["title"],
			why if why != "" else "(NO REASON -- this itself would be a defect)"])
	_log("RESULT unavailable=%d of %d total, %d of %d carry a reason"
		% [unavailable.size(), rows.size(), with_reason, unavailable.size()])
	get_tree().quit(0)
