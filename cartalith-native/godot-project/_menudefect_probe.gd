extends Node
## `menus.gd`'s three conformance defects, measured **through `CommandIndex`**
## rather than read off the source, plus a whole-file sweep for the two shapes
## behind them.
##
## Windowed is unnecessary here -- nothing rasterises and nothing is timed; every
## claim is a `PopupMenu` row's text/tooltip/metadata and the `CommandIndex` row
## built from it. `--headless` is correct for this one.
##
##   godot --headless --path . _menudefect_probe.tscn
##
## **Two `about_to_popup` handlers are fired deliberately, and only two.**
## `command_index.gd`'s header forbids firing them wholesale -- the atlas one
## evicts baked chunks, the GPU one enumerates adapters -- so this fires exactly
## the File popup's (it rewrites the four STORAGE LOCATIONS rows and nothing
## else outside the File menu) and the `UndoBudget` submenu's (it reads
## `undo_stats()` and rewrites its own rows). Those two are where defects 1 and
## 3 live; both are pure reads. The index is built TWICE, before and after, so
## the report shows what a cold search sees and what a search after the menu
## has been opened sees.

var _vp: SubViewport
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(label: String, ok: bool) -> void:
	if not ok:
		_fail += 1
	print("  [%s] %s" % ["PASS" if ok else "FAIL", label])

func _gather(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_gather(c, out)

## Every non-separator row of every popup, flattened, with the three facts the
## index reads: disabled, tooltip, marker.
func _walk(p: PopupMenu, trail: String, out: Array) -> void:
	for i in p.item_count:
		if p.is_item_separator(i):
			continue
		var text := p.get_item_text(i)
		var sub := p.get_item_submenu(i)
		if sub != "":
			var node := p.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				_walk(node as PopupMenu, trail + " > " + text, out)
			continue
		var meta = p.get_item_metadata(i)
		out.append({
			"trail": trail, "text": text,
			"disabled": p.is_item_disabled(i),
			"tip": p.get_item_tooltip(i),
			"marker": String(meta) if typeof(meta) == TYPE_STRING else "",
		})

func _index_rows(app: Node) -> Array:
	var idx := CommandIndex.new()
	idx.build(app, app.get("bridge"))
	return idx.all()

func _dump_titles(rows: Array, needles: Array, tag: String) -> void:
	print("--- %s ---" % tag)
	var hits := 0
	for r in rows:
		var t := String(r["title"])
		for n in needles:
			if t.to_lower().find(String(n).to_lower()) >= 0:
				hits += 1
				print("  title=%s  group=%s  kind=%s  available=%s" % [
					t, String(r["group"]), String(r["kind"]), str(bool(r["available"]))])
				print("     why=%s" % String(r["why"]))
				print("     blurb=%s" % String(r["blurb"]))
				break
	if hits == 0:
		print("  (no matching row in the index)")

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(1)
		return
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(50)
	print("[BOOT] shell up")

	var buttons: Array = []
	_gather(app, buttons)
	var by_name := {}
	for mb in buttons:
		by_name[String((mb as MenuButton).text)] = (mb as MenuButton).get_popup()
	print("[MENUS] %s" % str(by_name.keys()))

	## ---- COLD index (built state only) --------------------------------------
	var cold := _index_rows(app)
	print("[COLD] index rows=%d" % cold.size())
	_dump_titles(cold, ["projects", "worlds", "exports", "packs", "assets", "cache"], "COLD: storage-root-shaped titles")
	_dump_titles(cold, ["Colour management"], "COLD: Colour management")
	_dump_titles(cold, ["Clear undo history"], "COLD: Clear undo history")

	## ---- fire the two handlers ----------------------------------------------
	var file_pop: PopupMenu = by_name.get("File")
	if file_pop != null:
		file_pop.about_to_popup.emit()
		print("[FIRE] File.about_to_popup")
	var pref_pop: PopupMenu = by_name.get("Preferences")
	var undo_pop: PopupMenu = null
	if pref_pop != null:
		undo_pop = pref_pop.get_node_or_null(NodePath("UndoBudget"))
		if undo_pop != null:
			undo_pop.about_to_popup.emit()
			print("[FIRE] UndoBudget.about_to_popup")
	await _frames(2)

	## ---- WARM index ---------------------------------------------------------
	var warm := _index_rows(app)
	print("[WARM] index rows=%d" % warm.size())
	_dump_titles(warm, ["projects", "worlds", "exports", "packs", "assets", "cache"], "WARM: storage-root-shaped titles")
	_dump_titles(warm, ["Colour management"], "WARM: Colour management")
	_dump_titles(warm, ["Clear undo history"], "WARM: Clear undo history")

	## ---- the raw File storage rows -----------------------------------------
	print("--- File popup, every disabled row ---")
	if file_pop != null:
		var frows: Array = []
		_walk(file_pop, "File", frows)
		for r in frows:
			if bool(r["disabled"]):
				print("  text=%s | marker=%s | tip=%s" % [
					String(r["text"]),
					String(r["marker"]) if String(r["marker"]) != "" else "(none)",
					String(r["tip"])])

	## ---- SWEEP: every disabled row in every menu ----------------------------
	print("--- SWEEP: every disabled row, all seven menus (post-fire) ---")
	var all_rows: Array = []
	for k in by_name.keys():
		var p: PopupMenu = by_name[k]
		if p != null:
			_walk(p, String(k), all_rows)
	var n_dis := 0
	var n_silent := 0
	var n_readout := 0
	var n_signpost := 0
	var n_tipped := 0
	for r in all_rows:
		if not bool(r["disabled"]):
			continue
		n_dis += 1
		var m := String(r["marker"])
		if m == "readout":
			n_readout += 1
		elif m == "signpost":
			n_signpost += 1
		elif String(r["tip"]).strip_edges() == "":
			n_silent += 1
		else:
			n_tipped += 1
		print("  [%s] %s > %s" % [m if m != "" else "-", String(r["trail"]), String(r["text"])])
		if String(r["tip"]).strip_edges() != "":
			print("       tip: %s" % String(r["tip"]))
	print("SWEEP TOTALS rows=%d disabled=%d readout=%d signpost=%d unmarked_tipped=%d unmarked_silent=%d"
		% [all_rows.size(), n_dis, n_readout, n_signpost, n_tipped, n_silent])

	## ---- assertions ---------------------------------------------------------
	print("--- assertions ---")
	## Scoped to the File menu's own rows: the Atlas cache readout under
	## Preferences legitimately carries `Root: C:/…` on its tooltip, and a
	## path-shaped-blurb count that swept the whole index would score it as a
	## fifth storage root.
	var storage_bad := 0
	var storage_ro_warm := 0
	var storage_ro_cold := 0
	for r in warm:
		if String(r["group"]) != "File":
			continue
		var b := String(r["blurb"])
		if b.find(":/") < 0 and b.find(":\\") < 0:
			continue
		if String(r["kind"]) == "readout":
			storage_ro_warm += 1
		else:
			storage_bad += 1
			print("  STORAGE-AS-COMMAND: title=%s kind=%s available=%s why=%s"
				% [String(r["title"]), String(r["kind"]), str(bool(r["available"])), String(r["why"])])
	for r in cold:
		if String(r["group"]) == "File" and String(r["kind"]) == "readout":
			var b2 := String(r["blurb"])
			if b2.find(":/") >= 0 or b2.find(":\\") >= 0:
				storage_ro_cold += 1
	_check("no File row carries a filesystem path as an unavailable command's reason (found %d)" % storage_bad, storage_bad == 0)
	_check("the four storage roots arrive as readouts once File has been opened (found %d)" % storage_ro_warm, storage_ro_warm == 4)
	_check("...and in a COLD index, built before File was ever opened (found %d)" % storage_ro_cold, storage_ro_cold == 4)

	var cm_bad := false
	for r in all_rows:
		if String(r["text"]).find("Colour management") >= 0:
			print("  colour-management row: %s" % String(r["text"]))
			if String(r["text"]).find("Render") >= 0:
				cm_bad = true
	_check("the Colour management signpost names no RENDER domain", not cm_bad)

	var undo_seen := false
	var undo_ok := false
	for r in all_rows:
		if String(r["text"]).find("Clear undo history") >= 0:
			undo_seen = true
			var tip := String(r["tip"])
			print("  clear-undo row: disabled=%s tip=%s" % [str(bool(r["disabled"])), tip])
			if bool(r["disabled"]):
				undo_ok = tip.to_lower().find("empty") >= 0
			else:
				undo_ok = true
	_check("the Clear undo history row was found", undo_seen)
	_check("a disabled Clear undo history states its precondition, not the action", undo_ok)

	## **Reported, not asserted.** The sweep's one remaining `disabled and
	## silent` row is `File ▸ Close project`: `_file()`'s `about_to_popup`
	## disables it when there is no world and never writes a tooltip, so
	## `_walk_popup`'s chrome guard drops it from the index **entirely** the
	## moment the File menu has been opened without a world -- not "unavailable
	## with no reason", gone. Same shape as the three defects above and outside
	## this lane's brief, so it is measured here and left for the owner to size.
	var close_cold := 0
	var close_warm := 0
	for r in cold:
		if String(r["title"]) == "Close project":
			close_cold += 1
	for r in warm:
		if String(r["title"]) == "Close project":
			close_warm += 1
	print("  FINDING (not fixed): 'Close project' index rows -- cold=%d warm=%d" % [close_cold, close_warm])

	print("RESULT %s failures=%d" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(0 if _fail == 0 else 1)
