extends Node
## PC-4 probe: the Context panel (`PC_SHELL_PROPOSAL.md` §3.3, milestone PC-4), inside the real shell,
## with real mouse clicks and real key events through `Input.parse_input_event()`.
##
##   desktop: godot --path . --resolution 1600x1000 _pc4context_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1600x1000 _pc4context_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   _pc4context_probe.tscn -- --force-touch --phone --shots <dir>
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped
## `app_userdata/<name>` (`MISTAKES.md`). Reads `--force-touch`, `--phone` and `--shots <dir>`;
## anything else exits 2.
##
## What it protects (PC-4's bar: "Every name in Inspect selects; the pin holds while selecting
## elsewhere; an armed tool's section is below the subject"). A small world is generated (seed 4242,
## 256 x 192, as `_pc1select_probe.gd`) and one conflict is drawn between two factions, attached to a
## settlement. Then:
##   A. PANEL: the header (back, forward, name, FOLLOWING) and a tab strip of exactly one tab,
##      INSPECT (Note and Ask are PC-5's and PC-6's and must be absent).
##   B. SETTLEMENT by a real map click: the selection, the header name and the panel's subject agree.
##   C. LINKS: on the settlement page, then the faction page, then the conflict page, every link
##      Inspect drew that names a subject is scrolled into view and clicked (a real click, its
##      centre inside the dock's visible rect): the selection and the panel's subject become that
##      subject; Alt+← (a real key) returns. Every year link moves the engine's year to its year.
##      Each page is reached through a link on the page before it, never by a function call.
##   D. LANDMARK by a real map click on a landmark marker away from every settlement, when one is
##      drawn (reported as "not drawn", never a pass, when none is).
##   E. FIND a faction (Ctrl+F, its name, a real click on its row): Inspect shows the faction.
##   F. PIN: the header's FOLLOWING pressed (real click) pins; a real map click on another place moves
##      the selection (CIVIL's index follows it) while the panel keeps its subject; the banner names
##      the new selection; PIN TO IT re-pins to it; Alt+← moves the selection, the pin holds; FOLLOW
##      shows the selection again. The menu's Pin and Follow rows do the same.
##   G. TOOL: with a settlement shown, arming Territory appends its section BELOW the subject.
##   H. MENUS: Window's region row reads "Context panel"; its submenu's Inspect, Pin, Follow and Reset
##      panel are live with tooltips and Note and Ask are disabled with reasons; Ctrl+1 (a real key)
##      shows a hidden panel; Reset panel shows a hidden and collapsed panel on Inspect; Story ▸
##      Political history of the selection opens Inspect at its history section.
##   P. PHONE (`--phone`): no header and no links in the selection sheet; two captures (launch, and
##      the sheet showing a selected place) for pixel comparison against the pre-PC-4 tree.
## Exit 0 pass, 1 fail, 2 bad argument, watchdog or no world, 3 a leg could not run.

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _phone := false
var _app: Node
var _bridge


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img != null:
		var path := _shots.path_join("pc4_%s_%s.png" % [_mode, tag])
		img.save_png(path)
		print("SHOT %s" % path)


func _click_at(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)
	await _settle(2)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(4)


func _key(code: Key, ctrl := false, alt := false) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		ev.ctrl_pressed = ctrl
		ev.alt_pressed = alt
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(4)


func _dock():
	return _app.right_dock_ctrl


func _body() -> Control:
	return _app.right_dock_body


## The named header control, or null.
func _hdr(node_name: String) -> Control:
	return _body().find_child(node_name, true, false) as Control


## Scrolls `c` into the dock's view and clicks its centre for real. Fails the check (and returns
## false) when the centre is not inside the dock scroll's visible rect, so a clipped link cannot pass.
func _press(c: Control, what: String) -> bool:
	if c == null:
		_ok(false, "%s exists to be pressed" % what)
		return false
	var sc: ScrollContainer = _app._right_dock_scroll
	if sc != null and sc.is_ancestor_of(c):
		sc.ensure_control_visible(c)
		await _settle(4)
	var at := c.get_global_rect().get_center()
	var vis := sc.get_global_rect() if (sc != null and sc.is_ancestor_of(c)) else Rect2(Vector2.ZERO, get_viewport().get_visible_rect().size)
	var inside := vis.has_point(at) and c.is_visible_in_tree()
	_ok(inside, "%s is drawn inside the dock's visible rect (centre %s in %s)" % [what, str(at), str(vis)])
	if not inside:
		return false
	await _click_at(at)
	await _settle(4)
	return true


## Every link Inspect drew, `[{node, target|year, text}]`, in tree order.
func _links() -> Array:
	var out: Array = []
	for b in _body().find_children("*", "Button", true, false):
		if not (b as Node).has_meta("ctx_link") or not (b as Control).is_visible_in_tree():
			continue
		var e := {"node": b, "text": (b as Button).text}
		if (b as Node).has_meta("ctx_target"):
			e["target"] = (b as Node).get_meta("ctx_target")
		if (b as Node).has_meta("ctx_year"):
			e["year"] = int((b as Node).get_meta("ctx_year"))
		out.append(e)
	return out


## The `§` section heading a link sits under: the last header Label above it in the dock body.
func _section_of(c: Control) -> String:
	var y := c.get_global_rect().position.y
	var best := ""
	var best_y := -INF
	for l in _body().find_children("*", "Label", true, false):
		var t := (l as Label).text
		if not t.begins_with("§ "):
			continue
		var ly := (l as Control).get_global_rect().position.y
		if ly <= y and ly > best_y:
			best_y = ly
			best = t
	return best


func _subject() -> Dictionary:
	return _dock().shown_subject()


## Clicks every subject link on the page now shown (up to `cap`), checking each selects its target
## and Alt+← returns to `page`. Then one year link, if any. Returns the links seen.
func _walk_links(page: Dictionary, label: String, cap: int = 10) -> Array:
	var links := _links()
	var subj_links := links.filter(func(e): return e.has("target"))
	_ok(not subj_links.is_empty(), "%s: Inspect draws at least one name link (got %d links)" % [label, links.size()])
	## Order: the first link of each section (the `§` heading above it) first, then the rest, so the
	## cap can never leave a whole section unpressed (a mutant that broke the Relations links once
	## survived because twelve settlement links used up the cap before them).
	var order: Array = []
	var seen := {}
	for i in subj_links.size():
		var sk := _section_of(subj_links[i]["node"])
		if not seen.has(sk):
			seen[sk] = true
			order.append(i)
	for i in subj_links.size():
		if not order.has(i):
			order.append(i)
	_ok(seen.size() >= 1, "%s: links found in %d sections (%s)" % [label, seen.size(), str(seen.keys())])
	var n := 0
	for i in order:
		if n >= cap:
			break
		## Re-read after every press: the page was rebuilt by the select and the back.
		var now := _links().filter(func(e): return e.has("target"))
		if i >= now.size():
			break
		var e: Dictionary = now[i]
		var tgt: Dictionary = e["target"]
		if not await _press(e["node"], "%s link '%s'" % [label, e["text"]]):
			continue
		n += 1
		_ok(_app.selection == tgt and _subject() == tgt,
			"%s: link '%s' selects %s and Inspect shows it (selection %s, panel %s)" % [label, e["text"], str(tgt), str(_app.selection), str(_subject())])
		await _key(KEY_LEFT, false, true)
		_ok(_app.selection == page and _subject() == page,
			"%s: Alt+← returns to %s (got %s / %s)" % [label, str(page), str(_app.selection), str(_subject())])
	var years := _links().filter(func(e): return e.has("year"))
	if not years.is_empty():
		var y: Dictionary = years[0]
		if await _press(y["node"], "%s year link %d" % [label, int(y["year"])]):
			_ok(int(_bridge.get_civ_year()) == int(y["year"]),
				"%s: year link %d moves the year (engine %d)" % [label, int(y["year"]), int(_bridge.get_civ_year())])
	return links


func _screen_of(cell: Vector2) -> Vector2:
	var ov = _app.viewport.overlay
	return (ov as Control).get_global_transform() * ov._cell_to_screen(cell, ov._displayed_rect())


## Named settlements whose marker is inside the map view, as indices.
func _visible_settlements() -> Array:
	var ov = _app.viewport.overlay
	var r: Rect2 = (ov as Control).get_global_rect().grow(-40.0)
	var all: Array = _bridge.settlements()
	var out: Array = []
	for i in all.size():
		var s: Dictionary = all[i]
		if String(s.get("name", "")).strip_edges() == "":
			continue
		if r.has_point(_screen_of(Vector2(float(s.get("x", 0)), float(s.get("y", 0))))):
			out.append(i)
	return out


func _click_settlement(i: int) -> void:
	var s: Dictionary = _bridge.settlements()[i]
	await _click_at(_screen_of(Vector2(float(s.get("x", 0)), float(s.get("y", 0)))))
	await _settle(4)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		var a := args[k]
		if a == "--force-touch":
			pass
		elif a == "--phone":
			_phone = true
		elif a == "--shots" and k + 1 < args.size():
			k += 1
			_shots = args[k]
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		k += 1
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s ==" % _mode)
	if _phone and _mode != "phone":
		print("[FATAL] --phone asked for and the shell is not the phone (%s)" % _mode)
		get_tree().quit(3)
		return
	if _phone:
		await _shot("launch")
	_bridge = _app.bridge
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _settle(10)
	await _settle(30)
	if not bool(_bridge.has_world) or _bridge.settlements().size() < 3 or _bridge.get_factions().size() < 2:
		print("[FATAL] no world with three settlements and two factions")
		get_tree().quit(2)
		return
	if _phone:
		await _leg_phone()
	else:
		await _leg_desktop()
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _leg_phone() -> void:
	print("== P. phone ==")
	## Seeding the capture: the selection is the model's (PC-1), which the phone keeps.
	_app.select("settlement", 0)
	await _settle(10)
	_ok(_hdr("ContextHeader") == null, "the phone's sheet draws no Context panel header")
	_ok(_links().is_empty(), "the phone's sheet draws no Inspect links (got %d)" % _links().size())
	await _shot("sheet")


func _leg_desktop() -> void:
	## The conflict the C leg walks: two factions as its sides, attached to a settlement of the first.
	var facs: Array = _bridge.get_factions()
	var f1 := int((facs[0] as Dictionary).get("id", 1))
	var f2 := int((facs[1] as Dictionary).get("id", 2))
	var all: Array = _bridge.settlements()
	var anchor := -1
	for i in all.size():
		if int((all[i] as Dictionary).get("faction", 0)) == f1:
			anchor = i
			break
	var a_s: Dictionary = all[maxi(anchor, 0)]
	var r: Dictionary = _bridge.conflict_add({"kind": "battle",
		"points": PackedVector2Array([Vector2(float(a_s.get("x", 0)), float(a_s.get("y", 0)))]),
		"start_year": 250})
	var cid := int(r.get("id", -1))
	_bridge.conflict_update(cid, {"name": "Probe War", "sides": PackedInt32Array([f1, f2]), "end_year": 260,
		"anchor_kind": "settlement", "anchor_tid": int(a_s.get("tid", 0))})
	_ok(bool(r.get("ok", false)) and cid >= 0, "a conflict was drawn for the walk (id %d)" % cid)
	_app.select_domain("civilization")
	_app.arm_tool("inspect")
	await _settle(8)

	print("== A. the panel ==")
	_ok(_app.context_tabs() == PackedStringArray(["inspect"]), "context_tabs() is [inspect] (got %s)" % str(_app.context_tabs()))
	for nm in ["ContextHeader", "CtxBack", "CtxForward", "CtxName", "CtxPin", "CtxTabs"]:
		_ok(_hdr(nm) != null, "the header has %s" % nm)
	var tabs := _hdr("CtxTabs")
	var tab_texts: Array = []
	if tabs != null:
		for b in tabs.find_children("*", "Button", true, false):
			tab_texts.append((b as Button).text)
	_ok(tab_texts == ["INSPECT"], "the tab strip holds exactly INSPECT (got %s)" % str(tab_texts))
	_ok(_hdr("CtxName") != null and (_hdr("CtxName") as Label).text == "Nothing selected", "with nothing selected the header says so")
	_ok(_hdr("CtxPin") != null and (_hdr("CtxPin") as Button).disabled, "with nothing shown the pin is disabled")

	print("== B. a settlement by a map click ==")
	var vis := _visible_settlements()
	_ok(vis.size() >= 2, "two named settlements are drawn inside the map view (got %d)" % vis.size())
	if vis.size() < 2:
		return
	var a := int(vis[0])
	var b := int(vis[vis.size() - 1])
	await _click_settlement(a)
	var page_a := {"kind": "settlement", "id": a}
	_ok(_app.selection == page_a and _subject() == page_a, "the map click selects settlement %d and Inspect shows it (%s / %s)" % [a, str(_app.selection), str(_subject())])
	var a_name := String((_bridge.settlements()[a] as Dictionary).get("name", ""))
	_ok(_hdr("CtxName") != null and (_hdr("CtxName") as Label).text == a_name, "the header names '%s' (got '%s')" % [a_name, (_hdr("CtxName") as Label).text if _hdr("CtxName") else "?"])
	_ok(_hdr("CtxPin") != null and (_hdr("CtxPin") as Button).text == "FOLLOWING", "the header says FOLLOWING")
	await _shot("settlement")

	print("== C. every name selects ==")
	## The anchor settlement's page has the conflict and its holder; reach it by a real click first.
	if anchor >= 0 and anchor != a:
		_app.viewport.move_view_to(float(a_s.get("x", 0)), float(a_s.get("y", 0)))
		await _settle(8)
		await _click_settlement(anchor)
	var page_s: Dictionary = _app.selection.duplicate()
	_ok(String(page_s.get("kind", "")) == "settlement", "a settlement page is shown for the walk (%s)" % str(page_s))
	await _walk_links(page_s, "settlement")
	## The faction page, through the settlement's Faction link (a real click).
	var flink: Array = _links().filter(func(e): return e.has("target") and String(e["target"]["kind"]) == "faction")
	_ok(not flink.is_empty(), "the settlement page links its holder")
	if not flink.is_empty():
		await _press(flink[0]["node"], "the holder link")
		var page_f: Dictionary = _app.selection.duplicate()
		_ok(String(page_f.get("kind", "")) == "faction", "the holder link opens a faction page (%s)" % str(page_f))
		await _shot("faction")
		var flinks := await _walk_links(page_f, "faction", 12)
		var kinds := {}
		for e in flinks:
			if e.has("target"):
				kinds[String(e["target"]["kind"])] = true
		_ok(kinds.has("settlement") and kinds.has("faction") and kinds.has("conflict"),
			"the faction page links settlements, other factions and its conflict (kinds %s)" % str(kinds.keys()))
		## The conflict page, through the faction page's conflict link.
		var clink: Array = _links().filter(func(e): return e.has("target") and String(e["target"]["kind"]) == "conflict")
		if not clink.is_empty():
			await _press(clink[0]["node"], "the conflict link")
			var page_c: Dictionary = _app.selection.duplicate()
			_ok(page_c == {"kind": "conflict", "id": cid}, "the conflict link opens conflict %d (%s)" % [cid, str(page_c)])
			await _shot("conflict")
			var clinks := await _walk_links(page_c, "conflict")
			var ck := {}
			for e in clinks:
				if e.has("target"):
					ck[String(e["target"]["kind"])] = int(ck.get(String(e["target"]["kind"]), 0)) + 1
			_ok(int(ck.get("faction", 0)) >= 2 and int(ck.get("settlement", 0)) >= 1,
				"the conflict page links both sides and its place (kinds %s)" % str(ck))

	print("== D. a landmark by a map click ==")
	await _leg_landmark()

	print("== E. Find a faction ==")
	await _leg_find_faction(f2)

	print("== E2. a conflict by its CIVIL ▸ Military ▸ Conflicts row ==")
	await _leg_conflict_row(cid)

	print("== F. the pin ==")
	## Find moved the camera (`move_view_to`); let any glide finish before reading marker positions.
	await get_tree().create_timer(1.5).timeout
	await _settle(10)
	## The earlier legs moved the view: take two places drawn in it now, far apart on screen.
	## Which of them a click really selects is the overlay's own hit test's answer (`_first_clickable()`):
	## the landmark leg ran a landmark pass, and a place under a landmark marker picks the landmark.
	## (A distance-from-landmarks filter here dropped every drawn place on the touch tablet.)
	var now := _visible_settlements()
	## Largest first: a small place's marker can be decluttered away at this zoom, a city's is drawn.
	var all_s: Array = _bridge.settlements()
	now.sort_custom(func(x, y): return int((all_s[x] as Dictionary).get("population", 0)) > int((all_s[y] as Dictionary).get("population", 0)))
	_ok(now.size() >= 2, "two named settlements are drawn in the view for the pin leg (got %d)" % now.size())
	if now.size() < 2:
		return
	## Two places a real click actually selects (a marker can be decluttered away at this zoom):
	## each candidate is clicked and kept only if the click selected it. The second sits 60 px or
	## more from the first, so the two presses cannot hit one marker.
	a = await _first_clickable(now, Vector2.INF)
	b = await _first_clickable(now, _screen_of(_cell_of(a)) if a >= 0 else Vector2.INF) if a >= 0 else -1
	_ok(a >= 0 and b >= 0, "two places in view select on a real click (%d, %d)" % [a, b])
	if a < 0 or b < 0:
		return
	await _leg_pin(a, b)

	print("== G. an armed tool appends below the subject ==")
	await _leg_tool(a)

	print("== H. menus ==")
	await _leg_menus(a)


## The first of `cands` (at most eight tried) that a real click selects, at least 60 px from `away`
## (`Vector2.INF` for no constraint), or -1.
func _first_clickable(cands: Array, away: Vector2) -> int:
	var tried := 0
	for i in cands:
		if tried >= 4:
			break
		var p := _screen_of(_cell_of(int(i)))
		if away != Vector2.INF and p.distance_to(away) < 60.0:
			continue
		## Ask the overlay's own hit test first (`map_overlay.gd::_pick_mark`, what a press runs): a
		## marker decluttered away or under another mark is not this place's to click.
		var ov = _app.viewport.overlay
		var rect: Rect2 = ov._displayed_rect()
		var local: Vector2 = (ov as Control).get_global_transform().affine_inverse() * p
		var pk: Dictionary = ov._pick_mark(local, ov._interior_rect(rect), rect)
		if int(pk.get("s", -1)) != int(i) or int(pk.get("l", -1)) != -1:
			continue
		tried += 1
		await _click_settlement(int(i))
		print("  info tried place %d at %s: selection %s" % [int(i), str(p), str(_app.selection)])
		if _app.selection == {"kind": "settlement", "id": int(i)}:
			return int(i)
	return -1


func _cell_of(i: int) -> Vector2:
	var s: Dictionary = _bridge.settlements()[i]
	return Vector2(float(s.get("x", 0)), float(s.get("y", 0)))


func _leg_landmark() -> void:
	var ov = _app.viewport.overlay
	if _bridge.landmarks().is_empty():
		## A fresh generate places no landmarks; the landmark pass is its own run (CIVIL ▸ Landmarks).
		var lr: Dictionary = await _bridge.landmark_run()
		print("  info landmark pass: %s" % str(lr.get("placed", lr.get("error", "?"))))
		await _settle(20)
	var lms: Array = ov._landmarks
	print("  info landmarks: overlay %d, bridge %d" % [lms.size(), _bridge.landmarks().size()])
	var r: Rect2 = (ov as Control).get_global_rect().grow(-40.0)
	var sets: Array = _bridge.settlements()
	var tried := 0
	for lm in lms:
		if tried >= 6:
			break
		var at := _screen_of(Vector2(float((lm as Dictionary).get("x", 0)), float((lm as Dictionary).get("y", 0))))
		if not r.has_point(at):
			continue
		var clear := true
		for s in sets:
			if _screen_of(Vector2(float((s as Dictionary).get("x", 0)), float((s as Dictionary).get("y", 0)))).distance_to(at) < 40.0:
				clear = false
				break
		if not clear:
			continue
		## Only a point the overlay's own hit test (`_pick_mark`) calls this landmark and no settlement:
		## then the click MUST select it, and anything else is a failure, never a skip.
		var rect: Rect2 = ov._displayed_rect()
		var local: Vector2 = (ov as Control).get_global_transform().affine_inverse() * at
		var pk: Dictionary = ov._pick_mark(local, ov._interior_rect(rect), rect)
		if int(pk.get("s", -1)) != -1 or int(pk.get("l", -1)) < 0:
			continue
		tried += 1
		await _click_at(at)
		await _settle(4)
		var want: int = _dock().landmark_index_of(ov._landmarks[int(pk["l"])])
		_ok(want >= 0 and _app.selection == {"kind": "landmark", "id": want} and _subject() == _app.selection,
			"a landmark click selects landmark %d and Inspect shows it (%s / %s)" % [want, str(_app.selection), str(_subject())])
		await _shot("landmark")
		return
	print("  info no landmark marker could be hit away from the settlements (tried %d): leg not run, not passed" % tried)


## CIVIL ▸ Military (a real click on the category head), then the conflict's row (a real click):
## the row is `select_conflict()`'s, which must go through the one selection path.
func _leg_conflict_row(cid: int) -> void:
	var head: Control = null
	for bt in _app.left_dock.find_children("*", "BaseButton", true, false):
		var txt := (bt as Button).text if bt is Button else ""
		for l in (bt as Node).find_children("*", "Label", true, false):
			txt += (l as Label).text
		if txt.strip_edges().ends_with("Military") and (bt as Control).is_visible_in_tree():
			head = bt
			break
	_ok(head != null, "CIVIL draws the Military category")
	if head == null:
		return
	## The category may already be open from an earlier leg, when a click on its head shuts it:
	## a second click then opens it. Two presses at most, each a real click.
	var row: Button = null
	var hsc := head.get_parent()
	while hsc != null and not (hsc is ScrollContainer):
		hsc = hsc.get_parent()
	for attempt in 2:
		if hsc != null:
			(hsc as ScrollContainer).ensure_control_visible(head)
			await _settle(4)
		await _click_at(head.get_global_rect().get_center())
		await _settle(12)
		for bt in _app.left_dock.find_children("*", "Button", true, false):
			if (bt as Button).text.contains("Probe War") and (bt as Control).is_visible_in_tree():
				row = bt
				break
		if row != null:
			break
		var seen: Array = []
		for bt in _app.left_dock.find_children("*", "Button", true, false):
			if (bt as Button).text.contains("·") and (bt as Control).is_visible_in_tree():
				seen.append((bt as Button).text.substr(0, 40))
		print("  info Military press %d (head at %s) found no Probe War row; rows %s" % [attempt + 1, str(head.get_global_rect()), str(seen.slice(0, 8))])
	_ok(row != null, "Military ▸ Conflicts lists Probe War")
	if row == null:
		return
	var sc := row.get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(row)
		await _settle(4)
	await _click_at(row.get_global_rect().get_center())
	await _settle(6)
	var want := {"kind": "conflict", "id": cid}
	_ok(_app.selection == want and _subject() == want,
		"the Conflicts row selects conflict %d and Inspect shows it (%s / %s)" % [cid, str(_app.selection), str(_subject())])


func _leg_find_faction(fid: int) -> void:
	var fname := ""
	for f in _bridge.get_factions():
		if int((f as Dictionary).get("id", -1)) == fid:
			fname = String((f as Dictionary).get("name", ""))
	await _key(KEY_F, true)
	await _settle(6)
	var field: LineEdit = _app._desktop_search_field
	_ok(field != null and field.is_visible_in_tree(), "Ctrl+F opens Find on map")
	if field == null or fname == "":
		return
	field.grab_focus()
	field.text = fname
	field.text_changed.emit(fname)
	await _settle(8)
	var hit: Control = null
	for bt in _app._desktop_search_results.find_children("*", "BaseButton", true, false):
		var texts: Array = []
		if bt is Button:
			texts.append((bt as Button).text)
		for l in (bt as Node).find_children("*", "Label", true, false):
			texts.append((l as Label).text)
		if texts.any(func(t): return String(t).to_lower().contains(fname.to_lower())) \
				and texts.any(func(t): return String(t).to_lower().contains("faction")):
			hit = bt
			break
	if hit == null:
		for bt in _app._desktop_search_results.find_children("*", "BaseButton", true, false):
			var texts: Array = []
			for l in (bt as Node).find_children("*", "Label", true, false):
				texts.append((l as Label).text)
			if (bt is Button and (bt as Button).text.to_lower().contains(fname.to_lower())) \
					or texts.any(func(t): return String(t).to_lower() == fname.to_lower()):
				hit = bt
				break
	_ok(hit != null, "Find lists faction '%s'" % fname)
	if hit == null:
		return
	var w := hit.get_window()
	var at := hit.get_global_rect().get_center() + (Vector2(w.position) if w != null and w != get_tree().root else Vector2.ZERO)
	await _click_at(at)
	await _settle(6)
	var want := {"kind": "faction", "id": fid}
	_ok(_app.selection == want and _subject() == want, "Find selects faction %d and Inspect shows it (%s / %s)" % [fid, str(_app.selection), str(_subject())])
	_ok(_hdr("CtxName") != null and (_hdr("CtxName") as Label).text == fname, "the header names the faction")


func _civ_index() -> int:
	var civ = _app._workspaces.filter(func(ww): return ww is CivilizationWorkspace)
	return int(civ[0]._selected_index) if not civ.is_empty() else -2


func _leg_pin(a: int, b: int) -> void:
	var sa := {"kind": "settlement", "id": a}
	var sb := {"kind": "settlement", "id": b}
	await _click_settlement(a)
	_ok(_subject() == sa, "settlement %d is shown before pinning (%s)" % [a, str(_subject())])
	await _press(_hdr("CtxPin"), "the pin button")
	_ok(_dock().is_pinned() and (_hdr("CtxPin") as Button).text == "PINNED", "pressing FOLLOWING pins the panel and it says PINNED")
	var b_s: Dictionary = _bridge.settlements()[b]
	await _click_settlement(b)
	_ok(_app.selection == sb, "a map click while pinned still moves the selection to %d (%s)" % [b, str(_app.selection)])
	_ok(_civ_index() == b, "CIVIL's index follows the selection while the panel is pinned (%d)" % _civ_index())
	_ok(_subject() == sa, "the pin holds: the panel still shows settlement %d (%s)" % [a, str(_subject())])
	var b_name := String(b_s.get("name", ""))
	var banner := _hdr("CtxBanner")
	var btext := ""
	if banner != null:
		for l in banner.find_children("*", "Label", true, false):
			btext += (l as Label).text
	_ok(banner != null and btext.contains(b_name), "the banner says the selection is now '%s' (got '%s')" % [b_name, btext])
	await _shot("pinned_banner")
	await _press(_hdr("CtxPinToIt"), "PIN TO IT")
	_ok(_dock().is_pinned() and _subject() == sb and _hdr("CtxBanner") == null, "PIN TO IT re-pins to %d and the banner goes" % b)
	await _key(KEY_LEFT, false, true)
	_ok(_app.selection != sb and _subject() == sb, "Alt+← moves the selection and the pin holds (%s / %s)" % [str(_app.selection), str(_subject())])
	await _press(_hdr("CtxFollow"), "FOLLOW")
	_ok(not _dock().is_pinned() and _subject() == _app.selection, "FOLLOW releases the pin and shows the selection (%s / %s)" % [str(_app.selection), str(_subject())])
	## The menu's rows, through the submenu's own id_pressed (ids 917 Pin, 918 Follow).
	var sub := _ctx_menu()
	_ok(sub != null, "Window ▸ Context panel exists")
	if sub == null:
		return
	await _click_settlement(a)
	sub.id_pressed.emit(DccMenus.ID_WIN_CTX_PIN)
	await _settle(6)
	_ok(_dock().is_pinned() and _subject() == sa, "Window ▸ Context panel ▸ Pin pins to the subject shown")
	_app.select("", -1)
	await _settle(4)
	_ok(_subject() == sa, "a deselect while pinned keeps the subject")
	sub.id_pressed.emit(DccMenus.ID_WIN_CTX_FOLLOW)
	await _settle(6)
	_ok(not _dock().is_pinned() and _subject().is_empty(), "Window ▸ Context panel ▸ Follow releases it onto the (empty) selection")
	sub.id_pressed.emit(DccMenus.ID_WIN_CTX_PIN)
	await _settle(4)
	_ok(not _dock().is_pinned(), "Pin with nothing shown pins nothing")


## Window ▸ Context panel's submenu: under the bar's Window menu, or ☰ ▸ Window on a bar that folds it.
func _ctx_menu() -> PopupMenu:
	var wp := _window_popup()
	return wp.find_child("ContextPanelMenu", true, false) as PopupMenu if wp != null else null


func _sub(p: PopupMenu, title: String) -> PopupMenu:
	for i in p.item_count:
		if not p.is_item_separator(i) and p.get_item_text(i) == title and p.get_item_submenu(i) != "":
			return p.get_node_or_null(NodePath(p.get_item_submenu(i))) as PopupMenu
	return null


## The popup holding Window's rows: the bar's Window, else ☰ ▸ Window.
func _window_popup() -> PopupMenu:
	var burger: PopupMenu = null
	for ch in _app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == "Window":
			return (ch as MenuButton).get_popup()
		if ch is MenuButton and (ch as MenuButton).text == "☰":
			burger = (ch as MenuButton).get_popup()
	return _sub(burger, "Window") if burger != null else null


func _leg_tool(a: int) -> void:
	await _click_settlement(a)
	_app.arm_tool("territory")
	await _settle(8)
	var head := _hdr("ContextHeader")
	var subj_sec: Control = null
	var tool_sec: Control = null
	for l in _body().find_children("*", "Label", true, false):
		var t := (l as Label).text.to_upper()
		if subj_sec == null and t == "§ SETTLEMENT":
			subj_sec = l
		if tool_sec == null and t.begins_with("§ TERRITORY"):
			tool_sec = l
	_ok(subj_sec != null and tool_sec != null, "the settlement section and the Territory section are both drawn")
	if subj_sec != null and tool_sec != null and head != null:
		_ok(head.get_global_rect().position.y < subj_sec.get_global_rect().position.y
			and subj_sec.get_global_rect().position.y < tool_sec.get_global_rect().position.y,
			"header above subject above the tool's section (y %d < %d < %d)" % [int(head.get_global_rect().position.y),
				int(subj_sec.get_global_rect().position.y), int(tool_sec.get_global_rect().position.y)])
		_ok(_subject() == {"kind": "settlement", "id": a}, "arming the tool keeps the subject")
	await _shot("tool")
	_app.arm_tool("inspect")
	await _settle(4)


func _leg_menus(a: int) -> void:
	var wp := _window_popup()
	_ok(wp != null, "the Window rows exist")
	if wp == null:
		return
	var texts: Array = []
	for i in wp.item_count:
		if not wp.is_item_separator(i):
			texts.append(wp.get_item_text(i))
	_ok(texts.count("Context panel") == 2 and not texts.has("Right dock"),
		"Window's region row reads Context panel, beside its submenu (rows %s)" % str(texts.slice(0, 8)))
	var sub := _ctx_menu()
	if sub == null:
		_ok(false, "Window ▸ Context panel exists")
		return
	var want_live := ["Inspect", "Pin to the current subject", "Follow the selection", "Reset panel"]
	var want_off := ["Note", "Ask (assistant)"]
	for i in sub.item_count:
		if sub.is_item_separator(i):
			continue
		var t := sub.get_item_text(i)
		if want_live.has(t):
			_ok(not sub.is_item_disabled(i) and sub.get_item_tooltip(i).strip_edges() != "", "Context panel ▸ %s is live with a tooltip" % t)
		elif want_off.has(t) or sub.is_item_disabled(i):
			_ok(sub.is_item_disabled(i) and sub.get_item_tooltip(i).strip_edges() != "", "Context panel ▸ %s is disabled with a reason" % t)
	## Ctrl+1 (a real key) shows a hidden panel.
	var rnode: CanvasItem = _app._region_nodes[DccMenus.ID_WIN_RIGHT]
	wp.id_pressed.emit(DccMenus.ID_WIN_RIGHT)
	await _settle(4)
	_ok(not rnode.visible, "Window ▸ Context panel's check hides the panel")
	await _key(KEY_1, true)
	_ok(rnode.visible, "Ctrl+1 shows the hidden panel on Inspect")
	## Reset panel: hidden and collapsed, then reset.
	wp.id_pressed.emit(DccMenus.ID_WIN_RIGHT)
	await _settle(4)
	_app._toggle_dock(false)
	await _settle(4)
	_ok(not rnode.visible and _app.is_dock_collapsed("right"), "the panel is hidden and collapsed before Reset")
	sub.id_pressed.emit(DccMenus.ID_WIN_CTX_RESET)
	await _settle(6)
	_ok(rnode.visible and not _app.is_dock_collapsed("right") and _hdr("CtxTabInspect") != null,
		"Reset panel shows the panel, uncollapsed, on Inspect")
	_ok(_app.context_panel_modules() == PackedStringArray(["inspect"]), "the arrangement is Inspect (got %s)" % str(_app.context_panel_modules()))
	## Story ▸ Political history of the selection: Inspect at its history section.
	var story: PopupMenu = null
	for ch in _app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == "Story":
			story = (ch as MenuButton).get_popup()
	_ok(story != null and story.get_item_index(DccMenus.ID_STORY_POLITICAL) >= 0
		and story.get_item_text(story.get_item_index(DccMenus.ID_STORY_POLITICAL)) == "Political history of the selection",
		"Story's row reads Political history of the selection")
	if story != null:
		## On a FACTION page (reached by a real click on a place's holder link), whose history section,
		## its conflicts, is the last on the page: from the top of the page it is out of view until the
		## row scrolls to it. (A settlement's history sits mid-page, in view from either end.)
		await _click_settlement(a)
		var fl: Array = _links().filter(func(e): return e.has("target") and String(e["target"]["kind"]) == "faction")
		if not fl.is_empty():
			await _press(fl[0]["node"], "the holder link, for the history leg")
		var sc: ScrollContainer = _app._right_dock_scroll
		await _settle(4)
		sc.scroll_vertical = 0
		await _settle(2)
		var hist0 := _body().find_child("InspectHistory", true, false) as Control
		_ok(String(_app.selection.get("kind", "")) == "faction" and hist0 != null
			and not sc.get_global_rect().intersects(hist0.get_global_rect()),
			"control: on a faction page at the top, its history section is out of view")
		story.id_pressed.emit(DccMenus.ID_STORY_POLITICAL)
		await _settle(8)
		var hist := _body().find_child("InspectHistory", true, false) as Control
		_ok(hist != null and sc.get_global_rect().intersects(hist.get_global_rect()),
			"Political history of the selection scrolls Inspect to its history section")
		await _shot("history")
