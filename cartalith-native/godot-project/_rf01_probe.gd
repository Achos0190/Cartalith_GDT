extends Node
## Committed probe for the 2026-08-25 "is every control wired" pass.
##
## Hunts **RF-01** systematically: a surface built at launch, before a world
## exists, with nothing re-running it on `generation_finished` / `world_loaded`.
## It has recurred at least four times (§23 the CIVIL dock, `_refresh_finalize`'s
## bake shortcut, the timeline's two bodies, §42's trade Match button).
##
## Method -- three phases, and the app is TOUCHED IN NO OTHER WAY between them:
##   A. boot, no world at all. Fingerprint every category body on all three
##      rails, the right dock, the layers popover, every tool-options row, and
##      every menu item.
##   B. generate world 1. Re-fingerprint. Anything IDENTICAL is a candidate.
##   C. generate world 2, different seed AND different size. Re-fingerprint.
##      Anything identical to B refreshed exactly once and then stopped -- the
##      same bug with a longer fuse (§23's own step 4).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _rf01_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var _app: Node
var _bridge
var _fail := 0

## Phrases that mean "this surface is showing its empty state". A container
## holding one of these AND unchanged across a generate is the smoking gun.
const EMPTY_MARKERS := [
	"generate a world", "no world", "No settlements", "No roads", "No routes",
	"No provinces", "No factions", "No ways", "No sea", "No coastal",
	"No trade", "No journeys", "No data", "not generated", "Generate a world",
	"no settlements", "No labels", "No icons", "No regions", "No notes",
	"No territor", "No military", "No relations", "No flows", "No history",
]


func _p(s: String) -> void:
	print("RF01  %s" % s)


func _bad(s: String) -> void:
	_fail += 1
	print("RF01  FAIL  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


## A fingerprint sensitive to any real content or state change, and to nothing
## else. Deliberately includes `disabled` and item counts: the Match-button bug
## changed no text at all.
func _fp(root: Node) -> String:
	if root == null:
		return "<null>"
	var all: Array = []
	_walk(root, all)
	var parts := PackedStringArray()
	for n in all:
		if n is OptionButton:
			var o := n as OptionButton
			var items := PackedStringArray()
			for i in o.item_count:
				items.append(o.get_item_text(i))
			parts.append("O:%s|%s|%s" % [o.text, "/".join(items), str(o.disabled)])
		elif n is Button:
			var b := n as Button
			parts.append("B:%s|%s|%s" % [b.text, str(b.disabled), str(b.button_pressed)])
		elif n is RichTextLabel:
			parts.append("R:%s" % (n as RichTextLabel).get_parsed_text())
		elif n is Label:
			parts.append("L:%s" % (n as Label).text)
		elif n is LineEdit:
			parts.append("E:%s|%s" % [(n as LineEdit).text, (n as LineEdit).editable])
		elif n is TextEdit:
			parts.append("T:%d" % (n as TextEdit).text.length())
		elif n is ItemList:
			var il := n as ItemList
			var items2 := PackedStringArray()
			for i in il.item_count:
				items2.append(il.get_item_text(i))
			parts.append("I:%s" % "/".join(items2))
		elif n is Tree:
			parts.append("W:%d" % _tree_rows((n as Tree).get_root()))
		elif n is Range:
			parts.append("V:%s" % str((n as Range).value))
	return "\n".join(parts)


func _tree_rows(item: TreeItem) -> int:
	if item == null:
		return 0
	var n := 1
	var c := item.get_first_child()
	while c != null:
		n += _tree_rows(c)
		c = c.get_next()
	return n


func _has_marker(s: String) -> bool:
	for m in EMPTY_MARKERS:
		if s.find(m) >= 0:
			return true
	return false


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


func _collect_popups(n: Node, out: Array) -> void:
	if n is PopupMenu:
		out.append(n)
	for c in n.get_children(true):
		_collect_popups(c, out)


func _menu_fp() -> Dictionary:
	var pops: Array = []
	_collect_popups(_app, pops)
	var out := {}
	for p in pops:
		var pm := p as PopupMenu
		var lines := PackedStringArray()
		for i in pm.item_count:
			lines.append("%s|%s" % [pm.get_item_text(i), str(pm.is_item_disabled(i))])
		var key: String = pm.get_path()
		out[key] = "\n".join(lines)
	return out


const TOOLS := ["inspect", "sculpt", "paint", "measure", "region",
	"settlement", "territory", "way", "route", "icon", "label"]

const DOMAINS := ["world", "civilization", "cartography"]


## One full sweep. Returns {label: fingerprint}.
func _sweep(phase: String) -> Dictionary:
	var out := {}
	var wss := {
		"world": _find(_app, "world_workspace.gd"),
		"civilization": _find(_app, "civilization_workspace.gd"),
		"cartography": _find(_app, "cartography_workspace.gd"),
	}
	for domain in DOMAINS:
		_app._select_domain(domain)
		await _frames(3)
		var ws: Node = wss[domain]
		for e in _all_categories(ws):
			var entry: Dictionary = e
			(entry["body"] as Control).visible = true
		await _frames(3)
		for e in _all_categories(ws):
			var entry2: Dictionary = e
			out["%s ▸ %s" % [domain.to_upper(), entry2["title"]]] = _fp(entry2["body"])
	# right dock, in its default context
	out["RightDock/sample"] = _fp(_app.right_dock_ctrl)
	# layers popover
	if _app.layers_popover != null:
		if _app.layers_popover.has_method("rebuild"):
			_app.layers_popover.visible = true
			_app.layers_popover.call("rebuild")
		await _frames(3)
		out["LayersPopover"] = _fp(_app.layers_popover)
		_app.layers_popover.visible = false
		await _frames(2)
	# every tool-options row (the tool bar builds into this same row)
	_app._select_domain("world")
	await _frames(2)
	for t in TOOLS:
		_app.arm_tool(t)
		await _frames(4)
		out["ToolOptions/%s" % t] = _fp(_app.tool_options_row)
	_app.arm_tool("inspect")
	await _frames(2)
	out["SectionStrip"] = _fp(_app.section_strip)
	# menus
	var mf := _menu_fp()
	for k in mf:
		out["Menu%s" % k] = mf[k]
	_p("sweep %s captured %d surfaces" % [phase, out.size()])
	return out


func _generate(seed: int, gw: int, gh: int, km: float) -> void:
	_bridge.generate({
		"seed": seed, "width_km": km, "grid_w": gw, "grid_h": gh,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.6).timeout


func _diff(a: Dictionary, b: Dictionary, tag: String) -> void:
	var same: Array = []
	var changed := 0
	var keys := a.keys()
	keys.sort()
	for k in keys:
		if not b.has(k):
			_p("  %s  %s DISAPPEARED" % [tag, k])
			continue
		if a[k] == b[k]:
			same.append(k)
		else:
			changed += 1
	_p("== %s : %d changed, %d identical ==" % [tag, changed, same.size()])
	for k in same:
		var txt: String = String(a[k])
		if _has_marker(txt):
			_bad("%s  %s  UNCHANGED and still showing an EMPTY STATE" % [tag, k])
			for line in txt.split("\n"):
				if _has_marker(String(line)):
					_p("        > %s" % line)
		else:
			_p("  %s  same: %s" % [tag, k])


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 900.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.5).timeout
	_bridge = _app.bridge
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)

	_p("=== PHASE A : no world ===")
	_p("settlements=%d factions=%d" % [_bridge.settlements().size(), _bridge.get_factions().size()])
	var a: Dictionary = await _sweep("A")

	_p("=== PHASE B : world 1 (seed 483920, 384x288, 2400 km) ===")
	await _generate(483920, 384, 288, 2400.0)
	_p("settlements=%d factions=%d ways=%d" % [
		_bridge.settlements().size(), _bridge.get_factions().size(), _bridge.roads().size()])
	var b: Dictionary = await _sweep("B")
	_diff(a, b, "A→B")

	_p("=== PHASE C : world 2 (seed 771155, 256x192, 900 km) ===")
	await _generate(771155, 256, 192, 900.0)
	_p("settlements=%d factions=%d ways=%d" % [
		_bridge.settlements().size(), _bridge.get_factions().size(), _bridge.roads().size()])
	var c: Dictionary = await _sweep("C")
	_diff(b, c, "B→C")

	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
