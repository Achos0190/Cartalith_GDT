extends Node
## Census only, using ONLY APIs that exist both at HEAD and in the working tree,
## so the same script can be run against either and the two outputs diffed.
## No reference to `apply_mode`, `mode_shows`, `domain_gates` or the mode switch.
##
## Identity is a TEXT signature, not a node path: Godot's auto names
## (`@Button@3476`) carry a per-process instance counter and do not survive a
## second run, which made the first attempt at this diff meaningless.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _vfy_ld32_census.tscn

var app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _is_ctrl(n: Node) -> bool:
	return n is BaseButton or n is LineEdit or n is Range or n is TextEdit \
		or n is ItemList or n is Tree

func _sig(n: Node) -> String:
	var t := n.get_class()
	var s := ""
	if n is Button:
		s = (n as Button).text
	elif n is LineEdit:
		s = (n as LineEdit).placeholder_text
	if s == "" and n is Control:
		s = (n as Control).tooltip_text
	return "%s|%s" % [t, s.replace("\n", " ")]

func _walk(n: Node, out: Array) -> void:
	if n is CanvasItem and not (n as CanvasItem).visible:
		return
	if _is_ctrl(n):
		out.append(_sig(n))
	for c in n.get_children():
		_walk(c, out)

func _modes(id: String) -> Array:
	var out: Array = []
	for n in DccShell.RAIL_NODES:
		if String(n.get("kind", "")) == "node" and String(n["domain"]) == id:
			out.append(String(n["mode"]))
	return out

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 2400:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	print("CEN world=%s" % app.bridge.has_world)

	for d in ["world", "civilization", "cartography"]:
		for m in _modes(d):
			app.select_domain_mode(d, m)
			await _frames(3)
			var p: Control = app.workspace_panel(d) as Control
			var cats: Array = []
			for e in p.categories:
				var body: Control = e.get("body")
				if body == null or not is_instance_valid(body):
					continue
				var wrap: Control = body.get_parent() as Control
				if wrap == null or not wrap.visible:
					continue
				var title := String(e.get("title", ""))
				cats.append(title)
				## Open it, so the controls under it are genuinely reachable.
				if not body.visible:
					(e["button"] as Button).pressed.emit()
					await _frames(2)
				var sigs: Array = []
				if body.visible:
					_walk(body, sigs)
				sigs.sort()
				for s in sigs:
					print("CEN CTRL\t%s\t%s\t%s\t%s" % [d, m, title, s])
				print("CEN CAT\t%s\t%s\t%s\t%d" % [d, m, title, sigs.size()])
			print("CEN MODE\t%s\t%s\t%d\t%s" % [d, m, cats.size(), str(cats)])
	print("CEN DONE")
	get_tree().quit(0)
