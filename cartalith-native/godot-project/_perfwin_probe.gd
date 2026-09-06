extends Node
## Committed probe for `LARGE_ITEM_RULINGS.md` ruling 19 -- "Diagnostics
## window? -> NO WINDOW ... `performance_window.gd` folds away ... the *rows*
## move."
##
## Two jobs, and the second is the one that makes the ruling safe:
##   1. the window is gone -- no `DccApp.performance_window`, no
##      `open_performance()`, no `res://shell/performance_window.gd`;
##   2. the rows that carried its content still DRAW, read off the live
##      `Preferences` popup rather than off `menus.gd`'s source.
##
## Run BEFORE the deletion too: every check in part 1 and the three "moved
## row" checks in part 2 fail at HEAD, which is what makes them evidence
## rather than tautologies.
##
## Palette: nothing here asserts a pixel, so the light/dark boot this machine
## does cannot change a verdict. Row TEXT and TOOLTIP only.
##
## `godot --headless --path . _perfwin_probe.tscn`

var _fail := 0
var _vp: SubViewport

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good = (got == want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, "  want=", want)

## Every `MenuButton`/`MenuBar`-fronted popup in the shell, found the same way
## `command_index.gd::_gather_menu_buttons` finds them -- by capability, not by
## node path, because the bar is rebuilt from `menus.gd` and has no stable name.
func _gather(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children():
		_gather(c, out)

func _popup_named(app: Node, want: String) -> PopupMenu:
	var buttons: Array = []
	_gather(app, buttons)
	for mb in buttons:
		if String((mb as MenuButton).text).to_lower().begins_with(want.to_lower()):
			return (mb as MenuButton).get_popup()
	return null

## Row lookup by leading text. The Working-set readout is rewritten wholesale on
## every `about_to_popup` (`_refresh_working_set_row`), so it is matched on its
## stable prefix and its VALUE is read off the same string.
func _row(p: PopupMenu, prefix: String) -> int:
	for i in p.item_count:
		if p.get_item_text(i).begins_with(prefix):
			return i
	return -1

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
	var bridge = app.get("bridge")

	print("\n=== 1: the window is gone ===")
	var props := []
	for pr in app.get_property_list():
		props.append(String(pr["name"]))
	_ok("DccApp has no `performance_window` field", props.has("performance_window"), false)
	_ok("DccApp has no `open_performance()`", app.has_method("open_performance"), false)
	_ok("shell/performance_window.gd is deleted",
		FileAccess.file_exists("res://shell/performance_window.gd"), false)

	print("\n=== 2: the rows it carried still draw ===")
	var pref := _popup_named(app, "Preferences")
	if pref == null:
		print("  FAIL could not find the Preferences popup"); _fail += 1
		print("\n[VERDICT] ", "PASS" if _fail == 0 else "FAIL (%d)" % _fail)
		get_tree().quit(1 if _fail > 0 else 0); return
	## Forces `_refresh_working_set_row()` / `_refresh_gpu_retry_row()` etc.,
	## which is where the live values are written. Without this the row still
	## reads the build-time placeholder text.
	pref.about_to_popup.emit()
	await _frames(2)

	var ws := _row(pref, "Working set")
	_ok("a `Working set` row exists", ws >= 0, true)
	var ws_text := pref.get_item_text(ws) if ws >= 0 else ""
	var ws_tip := pref.get_item_tooltip(ws) if ws >= 0 else ""
	print("    row text: ", ws_text)
	print("    tooltip : ", ws_tip)
	_ok("it carries a real figure (GB or MB)", ws_text.contains("GB") or ws_text.contains("MB"), true)
	_ok("it is a readout, not a command",
		ws >= 0 and String(pref.get_item_metadata(ws)) == DccMenus.META_READOUT, true)
	## The moved row: `performance_window.gd`'s Memory section carried the three
	## `Performance.RENDER_*_MEM_USED` monitors, and nothing else in the shell
	## reads them. Literal unit strings, not a constant asserted against itself.
	##
	## **Both branches, and which one is expected is decided here, not there.**
	## The monitors read a flat zero under the headless dummy driver and tens of
	## MiB in a real window, so `_video_mem_line()` dashes the zero rather than
	## printing `0 B`. Running this probe headless exercises the dash and running
	## it windowed exercises the figures -- a probe that only walked the live
	## path could not see an inversion (`MISTAKES.md`'s two-branch-dash row).
	_ok("the tooltip speaks about video memory at all", ws_tip.contains("Video memory"), true)
	var headless := DisplayServer.get_name() == "headless"
	print("    display server: ", DisplayServer.get_name(), "  -> expecting the ",
		"DASH branch" if headless else "FIGURES branch")
	if headless:
		_ok("...dashed, with its reason, rather than reading `0 B`",
			ws_tip.contains("not being reported") and not ws_tip.contains("0 B"), true)
	else:
		_ok("...with textures and buffers broken out",
			ws_tip.contains("textures") and ws_tip.contains("buffers"), true)
		_ok("...and the total is a real non-zero figure",
			int(Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED)) > 0, true)
	## The window-opening row `Working set…` is not in the spec (SS2.5 draws
	## `Working set | Read-only`) and is what the ruling removes.
	var opener := -1
	for i in pref.item_count:
		var t := pref.get_item_text(i)
		if t.begins_with("Working set") and (t.ends_with("…") or t.ends_with("...")):
			opener = i
	_ok("no `Working set…` dialog row survives", opener, -1)

	var gpu := _row(pref, "GPU acceleration")
	_ok("SS2.5's `GPU acceleration` toggle still draws", gpu >= 0, true)
	print("    row text: ", pref.get_item_text(gpu) if gpu >= 0 else "")
	_ok("...and is a checkbox", gpu >= 0 and pref.is_item_checkable(gpu), true)

	var rq := _row(pref, "Render quality")
	_ok("SS2.5's `Render quality` submenu still draws", rq >= 0, true)
	var rq_tip := pref.get_item_tooltip(rq) if rq >= 0 else ""
	print("    tooltip : ", rq_tip)
	var rec := String(bridge.recommended_quality_tier())
	var tiers: PackedStringArray = bridge.quality_tiers()
	print("    engine says recommended=", rec, "  tiers=", tiers)
	_ok("the recommended tier is a real tier", rec in tiers, true)
	## The other moved row: `recommended_quality_tier()` had exactly one
	## consumer in the whole project and it was the deleted window.
	_ok("the tooltip names the recommendation", rq_tip.to_lower().contains("recommend"), true)
	_ok("...and names the tier the engine actually recommends", rq_tip.contains(rec), true)

	print("\n=== 3: the searchable index ===")
	var idx := CommandIndex.new()
	idx.build(app, bridge)
	var titles := []
	for r in idx.all():
		titles.append(String(r["title"]))
	print("  index size: ", idx.size())
	for r in idx.all():
		if String(r["title"]).contains("orking set"):
			print("    [idx] title=", r["title"], " kind=", r["kind"], " avail=", r["available"])
	_ok("`Working set` is still findable", titles.has(ws_text.strip_edges()) or ws_text.strip_edges() in "\n".join(titles), true)
	## `command_index.gd` strips a row's trailing ellipsis, so the deleted
	## `Working set…` opener is indexed as a bare `Working set` of
	## `kind: "menu"` -- measured at HEAD, where it collides by title with the
	## readout row two lines above it in the same popup. Counted by KIND, not
	## by title, because the title alone cannot tell the two apart.
	var dead := 0
	var readout := 0
	for r in idx.all():
		var t := String(r["title"]).strip_edges()
		if String(r["kind"]) == "menu" and t == "Working set":
			dead += 1
		if String(r["kind"]) == "readout" and t.begins_with("Working set"):
			readout += 1
	_ok("the removed opener is no longer an indexed command", dead, 0)
	_ok("the surviving readout IS indexed -- the row is the search result", readout >= 1, true)

	print("\n[VERDICT] ", "PASS" if _fail == 0 else "FAIL (%d checks)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
