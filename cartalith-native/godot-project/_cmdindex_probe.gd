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
##
## Result: `[BOOT] shell up`, six numbered sections (`=== 1: it found things, from both sources ===` to
## `=== 6: groups are real and banded ===`) holding `  ok|FAIL  <name>  got= want=` and `  info ...` lines, a
## `  SILENT  <title>  (<group> menu)` line for any unavailable row with no reason, and a final
## `_cmdindex_probe: PASS` or `<n> FAILURE(S)`; exit 1 on any failure or `[FATAL] extension did not load`.

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

	await _cq0(app, idx)

	print("\n_cmdindex_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)

## CQ-0 (CUSTOM_QUICK_BAR_SCOPE.md s.7): pin identity and the pinnable set.
## Protects: every pinnable row has a unique, stable `pin_key`; every unpinnable
## row says why; the two named examples are pinnable and RUN (their windows
## open); a pressed row reaches the same handler a menu click does.
func _cq0(app: Node, idx: CommandIndex) -> void:
	print("\n=== 7: CQ-0 pin identity and the pinnable set ===")
	var by_reason := {}
	var pinnable := 0
	var keys := {}
	var dup := 0
	var no_reason := 0
	var pins: Array = []
	for r in idx.all():
		if bool(r["pinnable"]):
			pinnable += 1
			pins.append(r)
			var k := String(r["pin_key"])
			if keys.has(k):
				dup += 1
			keys[k] = true
		else:
			var w := String(r["why_not"])
			if w.strip_edges() == "":
				no_reason += 1
			var cat := "other"
			if String(r["kind"]) == "param": cat = "param"
			elif String(r["kind"]) == "readout": cat = "readout"
			elif w.begins_with("Not available"): cat = "unavailable (_todo)"
			elif w.begins_with("A check"): cat = "check/radio"
			elif w.begins_with("A generated"): cat = "dynamic"
			elif w.begins_with("The same command"): cat = "alias of another menu row"
			elif w.begins_with("Its menu runs"): cat = "unsafe about_to_popup"
			elif w.begins_with("Not an app menu"): cat = "embedded ColorPicker popup"
			elif w.begins_with("No registered"): cat = "extra without handler"
			by_reason[cat] = int(by_reason.get(cat, 0)) + 1
	print("  info total rows %d · pinnable %d · unpinnable %d" % [idx.size(), pinnable, idx.size() - pinnable])
	print("  info unpinnable by reason: ", by_reason)
	var refreshing := 0
	for r in pins:
		if bool(r["refreshes_on_open"]):
			refreshing += 1
	print("  info pinnable rows whose popup refreshes state on open (chip must not copy `available`): ", refreshing)
	_ok("some rows are pinnable (not silently empty)", pinnable > 10, true)
	_ok("0 duplicate pin_keys among pinnable rows", dup, 0)
	## Independent of `_why_not_pinnable`: ask the popup itself whether the row is
	## checkable and require the index to have refused it. Goes red if the
	## check/radio filter is weakened (mutation-tested 2026-10-25).
	var checkable_pinned := 0
	var checkable_seen := 0
	for r in idx.all():
		var pp = r.get("_popup")
		if pp == null or String(r["group"]) == "":
			continue
		var ix: int = (pp as PopupMenu).get_item_index(int(r["_id"]))
		if ix >= 0 and (pp as PopupMenu).is_item_checkable(ix):
			checkable_seen += 1
			if bool(r["pinnable"]):
				checkable_pinned += 1
	print("  info real (non-ColorPicker) check/radio rows: ", checkable_seen)
	_ok("there ARE check/radio rows to refuse", checkable_seen > 20, true)
	_ok("no check/radio row is pinnable", checkable_pinned, 0)
	_ok("every unpinnable row has a non-empty why_not", no_reason, 0)
	for r in pins:
		print("  pin  %-34s %-18s %s" % [String(r["pin_key"]), String(r["group"]), String(r["title"])])
	var jp := idx.find_pin("menu:%d" % DccMenus.ID_STORY_JOURNEY_PLANNER)
	var vt := idx.find_pin("menu:%d" % DccMenus.ID_VAULT)
	_ok("Journey planner is pinnable", bool(jp.get("pinnable", false)), true)
	_ok("Markdown vault is pinnable", bool(vt.get("pinnable", false)), true)
	_ok("an unknown pin_key resolves to {} rather than a plausible row", idx.find_pin("menu:999999").is_empty(), true)
	_ok("running an unpinnable row is refused", idx.run_row(idx.search("Sea level")[0]), false)
	if not jp.is_empty():
		_ok("Journey planner chip runs", idx.run_row(jp), true)
		await _frames(4)
		_ok("...and journey_planner_view is visible", app.journey_planner_view != null and app.journey_planner_view.is_visible_in_tree(), true)
	if not vt.is_empty():
		_ok("Markdown vault chip runs", idx.run_row(vt), true)
		await _frames(4)
		_ok("...and vault_window is visible", app.vault_window != null and app.vault_window.visible, true)
