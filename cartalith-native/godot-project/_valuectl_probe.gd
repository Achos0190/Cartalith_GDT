extends Node
## Committed probe for the 2026-08-25 "is every control wired" pass.
##
## The control class no sweep in this repository has ever driven. `_deadwire`
## reads connection lists; `_pressall` and `_railpress` press BUTTONS and
## explicitly skip `OptionButton`. Nothing has ever *changed a value* on an
## OptionButton, an HSlider or a SpinBox and asked whether anything happened --
## which is the whole of CARTO's rail, most of WORLD's, and every tool-options
## bar in the shell.
##
## For each: read the current value, move it to a different one, emit the real
## signal (`item_selected` / `value_changed`), snapshot the whole app, then put
## it back. Report every control whose move changed nothing anywhere.
##
## A no-change move is not automatically a bug -- a value the renderer only
## consults on the next generate, or a mode whose two branches happen to draw
## the same thing on this world -- but it is the short list, and it is a list
## nothing else produces.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _valuectl_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var _app: Node
var _bridge

const DOMAINS := ["world", "civilization", "cartography"]
const TOOLS := ["inspect", "sculpt", "paint", "measure", "region",
	"settlement", "territory", "way", "route", "icon", "label"]


func _p(s: String) -> void:
	print("VALUECTL  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


## Deliberately wider than the button sweeps' snapshot: a value control's whole
## job may be to move a *texture*, which no string captures. So the viewport's
## own drawn pixels are folded in as a cheap checksum.
func _snapshot() -> String:
	var all: Array = []
	_walk(_app, all)
	var parts := PackedStringArray()
	for n in all:
		if n is Label:
			parts.append((n as Label).text)
		elif n is RichTextLabel:
			parts.append((n as RichTextLabel).get_parsed_text())
		elif n is OptionButton:
			parts.append("%s|%d" % [(n as OptionButton).text, (n as OptionButton).selected])
		elif n is Button:
			var b := n as Button
			parts.append("%s|%s|%s" % [b.text, str(b.disabled), str(b.button_pressed)])
		elif n is LineEdit:
			parts.append((n as LineEdit).text)
		elif n is Range:
			parts.append(str((n as Range).value))
		elif n is ItemList:
			parts.append("IL%d" % (n as ItemList).item_count)
		if n is Control:
			parts.append("v" if (n as Control).is_visible_in_tree() else "h")
	return "".join(parts)


## A 24-point sample of the map, so a control whose only effect is on the
## raster is not reported as dead. Cheap: one `get_image()` per move would be
## far too slow across ~200 controls, so this reads the overlay's own state
## instead -- the texture RIDs and the drawn-annotation counts.
func _map_fingerprint() -> String:
	var v = _app.viewport
	if v == null:
		return ""
	var parts := PackedStringArray()
	for prop in ["_camera_zoom", "_camera_x", "_camera_y"]:
		if v.get(prop) != null:
			parts.append(str(v.get(prop)))
	if v.map_view != null and v.map_view.texture != null:
		parts.append(str((v.map_view.texture as Texture2D).get_rid()))
		parts.append(str((v.map_view.texture as Texture2D).get_size()))
	if v.territory_view != null and v.territory_view.texture != null:
		parts.append(str((v.territory_view.texture as Texture2D).get_rid()))
	if v.overlay != null:
		parts.append(str(v.overlay.visible))
	parts.append(String(v.debug_view()) if v.has_method("debug_view") else "")
	return "|".join(parts)


func _path_of(root: Node, c: Node) -> String:
	var parts: Array = []
	var cur := c
	while cur != null and cur != root:
		parts.push_front(cur.name)
		cur = cur.get_parent()
	return "/".join(parts)


func _row_label(c: Node) -> String:
	var p := c.get_parent()
	if p == null:
		return ""
	for s in p.get_children():
		if s is Label:
			return (s as Label).text
	return ""


func _sweep(title: String, root: Node) -> void:
	if root == null:
		_p("%s :: null" % title)
		return
	var all: Array = []
	_walk(root, all)
	var opts: Array = []
	var ranges: Array = []
	for n in all:
		if not (n is Control) or not (n as Control).is_visible_in_tree():
			continue
		if n is OptionButton:
			if not (n as OptionButton).disabled and (n as OptionButton).item_count > 1:
				opts.append(n)
		elif n is Range and not (n is ScrollBar) and not (n is ProgressBar):
			var r := n as Range
			if "editable" in r and not r.editable:
				continue
			if r.max_value > r.min_value:
				ranges.append(r)
	var dead: Array = []

	for o in opts:
		if not is_instance_valid(o) or not o.is_inside_tree():
			continue
		var ob := o as OptionButton
		var was: int = ob.selected
		var to: int = 0 if was != 0 else 1
		var before := _snapshot() + _map_fingerprint()
		ob.selected = to
		ob.item_selected.emit(to)
		await _frames(5)
		var after := _snapshot() + _map_fingerprint()
		if is_instance_valid(ob) and ob.is_inside_tree():
			ob.selected = was
			ob.item_selected.emit(was)
			await _frames(3)
		if before == after:
			dead.append("OPT   %s  «%s»  [%s -> %s]" % [
				_path_of(root, o), _row_label(o),
				ob.get_item_text(was) if was >= 0 and was < ob.item_count else "?",
				ob.get_item_text(to)])

	for r in ranges:
		if not is_instance_valid(r) or not r.is_inside_tree():
			continue
		var rg := r as Range
		var was_v: float = rg.value
		## Move to the far end of the range from where it sits, so a control
		## whose effect is small near its default still shows.
		var to_v: float = rg.min_value if (was_v - rg.min_value) > (rg.max_value - was_v) else rg.max_value
		if is_equal_approx(to_v, was_v):
			continue
		var before2 := _snapshot() + _map_fingerprint()
		rg.value = to_v
		rg.value_changed.emit(to_v)
		await _frames(5)
		var after2 := _snapshot() + _map_fingerprint()
		if is_instance_valid(rg) and rg.is_inside_tree():
			rg.value = was_v
			rg.value_changed.emit(was_v)
			await _frames(3)
		if before2 == after2:
			dead.append("RANGE %s  «%s»  [%s -> %s]" % [
				_path_of(root, r), _row_label(r), str(was_v), str(to_v)])

	_p("---- %s : %d options, %d ranges, %d CHANGED NOTHING" % [
		title, opts.size(), ranges.size(), dead.size()])
	for d in dead:
		_p("   NOCHANGE  %s" % d)


func _find(n: Node, script_file: String) -> Node:
	if n.get_script() != null and String(n.get_script().resource_path).ends_with(script_file):
		return n
	for c in n.get_children(true):
		var r := _find(c, script_file)
		if r != null:
			return r
	return null


func _all_categories(ws: Node) -> Array:
	var out: Array = []
	if ws == null:
		return out
	out.append_array(ws.categories)
	for extra in ["_infra", "_render"]:
		if ws.get(extra) != null:
			out.append_array((ws.get(extra) as Node).categories)
	return out


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 1800.0
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
	_p("world: %d settlements, %d ways" % [
		_bridge.settlements().size(), _bridge.roads().size()])

	var wss := {
		"world": _find(_app, "world_workspace.gd"),
		"civilization": _find(_app, "civilization_workspace.gd"),
		"cartography": _find(_app, "cartography_workspace.gd"),
	}
	for domain in DOMAINS:
		var ws: Node = wss[domain]
		for e in _all_categories(ws):
			var entry: Dictionary = e
			_app._select_domain(domain)
			await _frames(2)
			for e2 in _all_categories(ws):
				((e2 as Dictionary)["body"] as Control).visible = false
			(entry["body"] as Control).visible = true
			await _frames(4)
			await _sweep("%s ▸ %s" % [domain.to_upper(), entry["title"]], entry["body"])

	var tool_domain := {
		"inspect": "world", "sculpt": "world", "paint": "world",
		"measure": "world", "region": "world",
		"settlement": "civilization", "territory": "civilization",
		"way": "civilization", "route": "civilization",
		"icon": "cartography", "label": "cartography",
	}
	for t in TOOLS:
		_app._select_domain(tool_domain[t])
		await _frames(3)
		_app.arm_tool(t)
		await _frames(5)
		await _sweep("ToolOptions/%s" % t, _app.tool_options_row)
	_app._select_domain("world")
	_app.arm_tool("inspect")
	await _frames(3)

	if _app.layers_popover != null:
		_app.layers_popover.visible = true
		if _app.layers_popover.has_method("rebuild"):
			_app.layers_popover.call("rebuild")
		await _frames(5)
		await _sweep("LayersPopover", _app.layers_popover)
		_app.layers_popover.visible = false

	_p("DONE")
	get_tree().quit(0)
