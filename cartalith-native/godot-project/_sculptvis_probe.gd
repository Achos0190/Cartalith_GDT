extends Node

## **With no tool armed, is the sculpt body on screen, and which block is it in?**
##
## The owner reported the sculpt tools showing "under the geology tab in world"
## on PC. `world_workspace.gd:549` adds `_sculpt_body` to the **Terrain**
## category, and `:550` gives **Geology** its own foot with no sculpt controls
## in it — so the report and the code name different blocks. The dock is an
## accordion rendering several blocks into one scroll region, so "under
## geology" may be where it APPEARS rather than where it is parented.
##
## Those are two different fixes, so this measures rather than assumes:
##   * is `_sculpt_body` visible with nothing armed? (the defect itself)
##   * which category body is it parented into? (which fix applies)
##   * what renders directly above it? (what the owner would have called it)
##
## `_paint_body` is measured beside it as the control: it carries
## `visible = false` at construction and `_sculpt_body` does not, so if the
## asymmetry is real they differ here.
##
##   godot --headless _sculptvis_probe.tscn -- --vp 1920x1080
##
## Exit 0 measured · 2 could not reach the workspace.

var _vp: SubViewport

func _log(s: String) -> void:
	print("[sculptvis] ", s)

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var a := OS.get_cmdline_user_args()
	for i in a.size():
		if a[i] == name and i + 1 < a.size():
			return a[i + 1]
	return dflt

## The named ancestor a node sits under — the block a reader would call it.
func _block_of(node: Node, app: Node) -> String:
	var p: Node = node.get_parent()
	var trail: Array[String] = []
	while p != null and p != app:
		if p.name != "" and not str(p.name).begins_with("@"):
			trail.append(str(p.name))
		p = p.get_parent()
	return " < ".join(trail) if not trail.is_empty() else "(all anonymous)"

func _ready() -> void:
	var vp_s := _arg("--vp", "1920x1080").split("x")
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(vp_s[0]), int(vp_s[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(30)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(30)

	var ws: Node = null
	for n in app.find_children("*", "", true, false):
		if n.get_class() == "VBoxContainer" and str(n.name) == "WorldWorkspace":
			ws = n
			break
	if ws == null:
		_log("ABORT no WorldWorkspace in the tree")
		get_tree().quit(2)
		return

	_log("armed tool = %s" % str(app.get("armed_tool")))
	for pair in [["_sculpt_body", ws.get("_sculpt_body")], ["_paint_body", ws.get("_paint_body")]]:
		var body: Control = pair[1]
		if body == null:
			_log("  %-13s (null)" % pair[0])
			continue
		_log("  %-13s own_vis=%-5s in_tree=%-5s  children=%d  block: %s" % [
			pair[0], str(body.visible), str(body.is_visible_in_tree()),
			body.get_child_count(), _block_of(body, app)])

	## **Both edges, because hiding it forever would pass a one-sided check.**
	## A gate is two claims -- it hides when the tool is not armed AND it shows
	## when it is -- and only the pair distinguishes a fix from a deletion.
	var sculpt_body: Control = ws.get("_sculpt_body")
	var off_vis: bool = sculpt_body != null and sculpt_body.visible
	app.arm_tool("sculpt")
	await _frames(6)
	var on_vis: bool = sculpt_body != null and sculpt_body.visible
	_log("  armed sculpt  -> own_vis=%s" % str(on_vis))
	app.arm_tool("inspect")
	await _frames(6)
	var back_vis: bool = sculpt_body != null and sculpt_body.visible
	_log("  back to inspect -> own_vis=%s" % str(back_vis))

	_log("=== the answer ===")
	if not off_vis and on_vis and not back_vis:
		_log("  GATED CORRECTLY: hidden unarmed, shown on sculpt, hidden again.")
	elif not off_vis and not on_vis:
		_log("  BROKEN: hidden even WITH sculpt armed -- this is a deletion, not a gate.")
	elif off_vis:
		_log("  DEFECT PRESENT: visible with nothing armed (the owner report).")
	else:
		_log("  shown on arm but not hidden on disarm -- the arm-another-tool path.")
	get_tree().quit(0)
