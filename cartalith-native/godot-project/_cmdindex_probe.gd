extends Node
## `CommandIndex` probe — the searchable action list behind the phone
## redesign's Direction A.
##
##   godot --headless --path . --resolution 1600x900 _cmdindex_probe.tscn
##
## The index's whole claim is that it is BUILT from the app's own tables rather
## than written down, so the assertions are about provenance, not about a
## hand-checked count: it must find real parameters with the engine's own
## wording, real menu rows walked off the live MenuBar, and it must report a
## disabled row as unavailable WITH the menu's own reason.
##
## An index that returned an empty list would satisfy every structural check —
## this repository's silently-empty-output trap — so the counts are asserted
## non-zero and specific known entries are named.

var _vp: SubViewport
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _titles(rows: Array) -> String:
	var t := []
	for r in rows:
		t.append(String(r["title"]))
	return "\n".join(t)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(50)
	print("[BOOT] shell up")

	var idx := CommandIndex.new()
	idx.build(app, app.get("bridge"))
	print("\n=== 1: it found things, from both sources ===")
	print("  info total entries: ", idx.size())
	_ok("the index is not empty", idx.size() > 40, true)

	var params := 0
	var menus := 0
	var unavailable := 0
	for r in idx.all():
		match String(r["kind"]):
			"param": params += 1
			"menu": menus += 1
		if not bool(r["available"]):
			unavailable += 1
	print("  info parameters %d · menu commands %d · unavailable %d" % [params, menus, unavailable])
	_ok("real generation parameters were indexed", params > 20, true)
	_ok("real menu commands were indexed", menus > 20, true)

	print("\n=== 2: the wording is the ENGINE's, not this file's ===")
	var all_titles := _titles(idx.all())
	## `params.rs` declares these labels; if the index invented its own
	## wording, or fell back to raw dotted keys, these fail.
	for want in ["Continentality", "Fragmentation", "Sea level"]:
		_ok("carries the engine's label %s" % want, all_titles.find(want) >= 0, true)
	_ok("no raw dotted key leaked through as a title",
		all_titles.find("tect.plates") < 0, true)

	print("\n=== 3: search narrows, and narrows honestly ===")
	var riv := idx.search("riv")
	print("  info search 'riv' -> %d rows" % riv.size())
	_ok("'riv' finds something", riv.size() > 0, true)
	var riv_titles := _titles(riv).to_lower()
	var all_hits_real := true
	for r in riv:
		var hay := (String(r["title"]) + String(r["blurb"]) + String(r["group"])).to_lower()
		if hay.find("riv") < 0:
			all_hits_real = false
	_ok("every 'riv' hit actually contains it somewhere", all_hits_real, true)
	_ok("a nonsense query returns nothing rather than everything",
		idx.search("qqzzxx").size(), 0)
	_ok("an empty query returns everything", idx.search("   ").size(), idx.size())

	print("\n=== 4: an unavailable row says WHY, in place ===")
	var with_reason := 0
	for r in idx.all():
		if not bool(r["available"]) and String(r["why"]).strip_edges() != "":
			with_reason += 1
	print("  info unavailable rows carrying a reason: %d of %d" % [with_reason, unavailable])
	for r in idx.all():
		if not bool(r["available"]) and String(r["why"]).strip_edges() == "":
			print("  SILENT  %s  (%s menu)" % [String(r["title"]), String(r["group"])])
	_ok("there ARE unavailable rows to test", unavailable > 0, true)
	_ok("every unavailable row carries a reason", with_reason, unavailable)

	print("\n=== 5: it is generated, so a menu edit reaches it ===")
	## The Edit menu's Cut row. **These two assertions were inverted on
	## 2026-09-03** and the flip is the point: the row was a `_todo` whose
	## reason mentioned the missing clipboard, and this probe pinned exactly
	## that -- `"Cut is reported unavailable" -> false` and a `why` containing
	## "clipboard". The clipboard landed (`menus.gd`'s `_clipboard`), the row
	## became a `_live` command, and both assertions had to move with it. A
	## test that pins the old behaviour is the same defect as prose that
	## describes it.
	##
	## `available` is the row's BUILT state, which is what `_walk_popup` reads
	## -- Cut goes dark with nothing selected, but only inside
	## `about_to_popup`, exactly as this file's own header records for
	## `Edit > Undo` and `Redo`.
	var cut := idx.search("Cut")
	var found_cut := false
	for r in cut:
		if String(r["title"]) == "Cut":
			found_cut = true
			_ok("Cut is reported available", bool(r["available"]), true)
			_ok("...with no unavailability reason attached", String(r["why"]), "")
			_ok("...indexed as a menu command, not a readout", String(r["kind"]), "menu")
			_ok("...under the Edit menu", String(r["group"]), "Edit")
	_ok("the Edit menu's Cut row reached the index", found_cut, true)

	print("\n=== 6: groups are real and banded ===")
	var gs := idx.groups()
	print("  info groups: ", gs.size(), " -> ", gs.slice(0, 8))
	_ok("more than one group", gs.size() > 3, true)

	print("\n_cmdindex_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
