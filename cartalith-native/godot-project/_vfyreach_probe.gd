extends Node
## VERIFIER spot-check for PC-REACH's "STILL NOT REACHED" list. That report says
## the six tool sections `rdStamps/rdPaint/rdStops/rdAnno/rdTerr/rdPlan` cannot
## be drawn because "arm_tool() takes only inspect/journey/measure/paint/sculpt
## by grep". `right_dock.gd::_tool_section()` is the derivation, and it keys on
## `app.armed_tool` plus (for paint/stops) the domain. This probe arms each id
## the shell actually calls `arm_tool()` with and reports which section appears.
##
##   Godot_v4.7.1 --headless --path . _vfyreach_probe.tscn -- --vp 1920x1080
##
## Reads only; arms tools through the public `app.arm_tool()` and reads
## `right_dock`'s own `_tool_section()` plus the drawn section headers.

var _vp: SubViewport

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## Every `§ `-prefixed header currently drawn in the right dock body.
func _headers(root: Node) -> Array:
	var out: Array = []
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n is Label and (n as Label).is_visible_in_tree():
			var t := String((n as Label).text)
			if t.begins_with("§"):
				out.append(t)
	out.sort()
	return out

func _watchdog() -> void:
	await get_tree().create_timer(240.0).timeout
	print("[RESULT] WATCHDOG -- never reached the end")
	get_tree().quit(2)

func _ready() -> void:
	_watchdog()
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(1)
		return
	var w := 1920
	var h := 1080
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--vp" and i + 1 < args.size():
			var parts := String(args[i + 1]).split("x")
			w = int(parts[0])
			h = int(parts[1])
			i += 2
		else:
			print("[FATAL] unrecognised argument: ", args[i])
			get_tree().quit(2)
			return

	_vp = SubViewport.new()
	_vp.size = Vector2i(w, h)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(30)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(30)

	var rd: Node = app.get("right_dock_ctrl")
	var rdbody: Node = app.get("right_dock")
	print("[VP] ", w, "x", h, "  right_dock_ctrl=", rd != null)

	## The full set of ids `arm_tool()` is actually called with anywhere in the
	## project, taken from `grep -roE 'arm_tool\("[a-z_]*"' --include=*.gd`.
	var ids := ["inspect", "journey", "measure", "paint", "sculpt",
		"territory", "label", "icon", "way", "route", "select"]
	for domain in ["world", "civilization", "cartography"]:
		app.select_domain(domain)
		await _frames(10)
		for id in ids:
			## Re-arm through a different id first: `arm_tool()` early-returns
			## when the id is already armed, so a straight loop would silently
			## skip every second case.
			app.arm_tool("select")
			await _frames(4)
			app.arm_tool(id)
			await _frames(10)
			var sec := String(rd.call("_tool_section"))
			print("  domain=%-13s armed=%-10s _tool_section=%-10s headers=%s" % [
				domain, id, ("(none)" if sec == "" else sec), str(_headers(rdbody))])
	get_tree().quit(0)
