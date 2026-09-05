extends Node
## VERIFIER probe for batch round-2, Lane Menus. Nothing here reuses the
## lane's own `_apcmdcheck_probe.gd`: the fifteen dropped rows below were
## re-derived from `git show 1111611 -- shell/menus.gd`'s own removed lines
## (7 EDIT + 5 BATCH + 2 of BUILD + the foot), and every count is recomputed
## from the live MenuBar rather than read off a printed total.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _vfy_r2_menus_probe.tscn

var _app: Node

const DROPPED := [
	"Open library workspace", "Import image into slot", "Sprite sheet slicer",
	"Add variant to slot", "Replace", "Slot transform", "Preview background",
	"Tag", "Collect into set", "Rename", "Duplicate", "Delete",
	"Apply to map", "Import pack .zip", "Clear library",
]

## The six the lane calls "already indexed one entry point up", with the exact
## surviving title it names. Checked as a TITLE substring, not search().
const SURVIVOR_CLAIM := [
	["Open library workspace", "Asset library"],
	["Import image into slot", "Import image"],
	["Sprite sheet slicer", "Sprite sheet slicer"],
	["Import pack .zip", "Import asset pack .zip"],
	["Apply to map", "Apply library to map"],
	["Clear library", "Clear library"],
]

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _titles_matching(rows: Array, needle: String) -> Array:
	var out: Array = []
	var q := needle.to_lower()
	for r in rows:
		if String(r.get("title", "")).to_lower().find(q) >= 0:
			out.append(r)
	return out

## The chrome-skip as it stood at HEAD -- no `is_readout` exemption. Used to
## derive the counterfactual total WITHOUT believing the lane's 363.
func _count_old_rule(popup: PopupMenu, menu_name: String, acc: Dictionary) -> void:
	for i in popup.item_count:
		var text := popup.get_item_text(i)
		if text.strip_edges() == "" or popup.is_item_separator(i):
			continue
		var sub := popup.get_item_submenu(i)
		if sub != "":
			var node := popup.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				_count_old_rule(node as PopupMenu, menu_name, acc)
			continue
		var meta = popup.get_item_metadata(i)
		var marker := String(meta) if typeof(meta) == TYPE_STRING else ""
		if marker == DccMenus.META_SIGNPOST:
			continue
		var disabled := popup.is_item_disabled(i)
		var tip := popup.get_item_tooltip(i)
		if disabled and tip.strip_edges() == "":
			acc["dropped"] = int(acc["dropped"]) + 1
			acc["dropped_readouts"] = int(acc["dropped_readouts"]) + (
				1 if marker == DccMenus.META_READOUT else 0)
			if marker == DccMenus.META_READOUT:
				acc["dropped_readout_titles"].append("%s | %s" % [menu_name, text])
			continue
		acc["kept"] = int(acc["kept"]) + 1

func _gather_mb(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_gather_mb(c, out)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	wd.timeout.connect(func():
		print("VFY  TIMEOUT")
		get_tree().quit(2))
	add_child(wd)
	wd.start()

	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(1)
		return
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await _frames(50)
	print("[BOOT] shell up")

	var idx = load("res://shell/command_index.gd").new()
	idx.build(_app, _app.get("bridge"))
	var rows: Array = idx.all()

	var by_kind := {}
	var unavailable: Array = []
	for r in rows:
		var k := String(r.get("kind", "?"))
		by_kind[k] = int(by_kind.get(k, 0)) + 1
		if not bool(r.get("available", true)):
			unavailable.append(r)
	print("VFY  TOTAL=%d  kinds=%s" % [rows.size(), str(by_kind)])
	var with_reason := 0
	for r in unavailable:
		if String(r.get("why", "")).strip_edges() != "":
			with_reason += 1
	print("VFY  UNAVAILABLE=%d  with_reason=%d" % [unavailable.size(), with_reason])
	for r in unavailable:
		print("VFY  UNAVAIL  %-14s | %-42s | %s" % [
			String(r["group"]), String(r["title"]),
			String(r.get("why", "")).substr(0, 90)])

	print("\nVFY  -- every readout row --")
	for r in rows:
		if String(r.get("kind", "")) == "readout":
			print("VFY  READOUT %-14s | %s" % [String(r["group"]), String(r["title"])])

	## Counterfactual: replay the chrome-skip WITHOUT the is_readout exemption.
	var acc := {"kept": 0, "dropped": 0, "dropped_readouts": 0, "dropped_readout_titles": []}
	var mbs: Array = []
	_gather_mb(_app, mbs)
	for mb in mbs:
		var pm: PopupMenu = (mb as MenuButton).get_popup()
		if pm != null:
			_count_old_rule(pm, String((mb as MenuButton).text), acc)
	print("\nVFY  OLD-RULE replay: kept(menu+readout)=%d dropped_as_chrome=%d of which marked-readout=%d" % [
		acc["kept"], acc["dropped"], acc["dropped_readouts"]])
	for t in acc["dropped_readout_titles"]:
		print("VFY  OLD-RULE would drop readout: %s" % t)
	var params := int(by_kind.get("param", 0))
	print("VFY  => total under OLD rule would be %d ; under LIVE rule it is %d" % [
		params + int(acc["kept"]), rows.size()])

	print("\nVFY  -- 15 dropped Asset-pack rows, TITLE matches only --")
	var zero := 0
	for needle in DROPPED:
		var hits := _titles_matching(rows, needle)
		if hits.is_empty():
			zero += 1
		var desc := ""
		for h in hits:
			desc += " [%s | %s | %s]" % [String(h["group"]), String(h["title"]), String(h["kind"])]
		print("VFY  DROPPED %-24s hits=%d%s" % [needle, hits.size(), desc])
	print("VFY  RESULT  %d of %d return 0 title matches" % [zero, DROPPED.size()])

	print("\nVFY  -- claimed navigation survivors, one entry point up --")
	for pair in SURVIVOR_CLAIM:
		var hits := _titles_matching(rows, String(pair[1]))
		var titles: Array = []
		for h in hits:
			titles.append("%s|%s" % [String(h["group"]), String(h["title"])])
		print("VFY  SURVIVOR %-24s -> '%s' hits=%d %s" % [
			String(pair[0]), String(pair[1]), hits.size(), str(titles)])

	print("\nVFY  -- word-level reachability the lane leans on --")
	for w in ["library", "import", "pack", "apply", "clear", "asset"]:
		var t := _titles_matching(rows, w)
		print("VFY  WORD '%s' title-hits=%d" % [w, t.size()])

	print("\nVFY  -- Cut/Copy/Paste/Select all availability --")
	for w in ["Cut", "Copy", "Paste", "Select all"]:
		for r in rows:
			if String(r["title"]) == w:
				print("VFY  EDITCMD %-11s group=%-10s available=%s why='%s'" % [
					w, String(r["group"]), str(r["available"]), String(r.get("why", ""))])

	print("VFY  DONE")
	get_tree().quit(0)
