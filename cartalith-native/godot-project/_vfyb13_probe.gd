extends Node
## VERIFIER probe, 2026-09-05. Written independently of `_phonemore_reach_probe.gd`
## to REFUTE Lane A's reachability claim, not to reproduce it.
##
## Three differences from the lane's own probe, each chosen because it is where
## a coverage gap would hide:
##
##   1. **All seven program menus**, not the three the lane walks. The lane
##      argues Assets/Help/Edit/Window are whole-popup drills and therefore
##      safe; that is an argument, so it is checked by pushing each drill and
##      reading back what it drew.
##   2. **One level deeper.** Every top-level submenu of every menu is pushed
##      and its OWN items are checked. A submenu turned into 6.6 `seg` chips
##      drops separators and routes disabled/non-radio items to `rest`; if that
##      path lost one, top-level-only checking cannot see it.
##   3. **The union is built from the whole surface**, so a row that moved to a
##      different screen still counts -- the question is "reachable anywhere",
##      which is the actual regression test.
##
##   Godot..._console.exe --path . _vfyb13_probe.tscn -- --force-touch --nowelcome
## Run WINDOWED for the layout figures; the banner prints which it was.

var app: Node
var pm
var _fail := 0
const DEFAULT_SIZE := Vector2i(1080, 2400)

func _arg_size() -> Vector2i:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--size="):
			var wh := s.substr(7).split("x")
			if wh.size() == 2 and wh[0].is_valid_int() and wh[1].is_valid_int():
				return Vector2i(int(wh[0]), int(wh[1]))
	return DEFAULT_SIZE

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("VFY %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _labels(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append(String((c as Label).text))
		elif c is Button:
			out.append(String((c as Button).text))
		_labels(c, out)

func _clean(t: String) -> String:
	var s := t.strip_edges()
	if s.ends_with("▸"):
		s = s.substr(0, s.length() - 1).strip_edges()
	return s

## `menus.gd::_readout()` writes "label   live value"; identity is the label half.
func _key(t: String) -> String:
	var s := _clean(t)
	var cut := s.find("   ")
	return s.substr(0, cut).strip_edges() if cut > 0 else s

func _has(texts: Array, want: String) -> bool:
	var k := _key(want)
	if k == "":
		return true   ## an empty-texted item is not a command
	for t in texts:
		if _key(String(t)) == k:
			return true
	return false

func _menu_popup(title: String) -> PopupMenu:
	for child in app.menu_bar_row.get_children():
		if child is MenuButton and String(child.text) == title:
			var p := (child as MenuButton).get_popup()
			p.about_to_popup.emit()
			return p
	return null

const MENUS: Array = ["File", "Edit", "Assets", "Data", "Preferences", "Window", "Help"]

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var want := _arg_size()
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	pm = app.get("_phone_menu")
	if pm == null:
		print("VFY  !! no PhoneMenu -- run with -- --force-touch")
		get_tree().quit(1)
		return
	var vp: Vector2 = app.get_viewport_rect().size
	print("VFY banner: display=%s  window=%dx%d  viewport=%.0fx%.0f  phone_scale=%.3f"
		% [DisplayServer.get_name(), want.x, want.y, vp.x, vp.y, app.phone_scale()])

	# ---- 1. the union of every string the phone MORE surface can draw --------
	var union: Array = []
	var body_min: Dictionary = {}
	for s in ["more", "project", "civ", "data", "sim", "prefs"]:
		pm.open()
		if s != "more":
			pm._push_screen(String(s))
		await _frames(3)
		var t: Array = []
		_labels(pm._screen_body, t)
		union.append_array(t)
		body_min[String(s)] = pm._screen_body.get_combined_minimum_size()

	## Every menu drill **the root actually routes to**, pushed for real.
	##
	## Derived from the live `ROOT_ROWS` / `ROOT_REST`, NOT from a written list
	## of the seven menus. The first version of this probe pushed all seven
	## unconditionally, which made the union include a menu the root no longer
	## linked to -- a check that could not fail. Caught by mutating `ROOT_REST`
	## to drop `Window`: the reachability assertion stayed green with Window
	## unreachable. Same shape as MISTAKES.md's non-discriminating-check row.
	var routed: Array = []
	for spec in pm.ROOT_ROWS:
		if String(spec["t"]) == "menu":
			routed.append(String(spec["id"]))
	for m in pm.ROOT_REST:
		routed.append(String(m))
	print("VFY root routes to these menus: [%s]" % ", ".join(routed))
	for m in routed:
		var p := _menu_popup(String(m))
		if p == null:
			_check("root-routed menu '%s' exists on the bar" % m, false)
			continue
		pm.open()
		pm._push(p, String(m), 3)
		await _frames(3)
		var t: Array = []
		_labels(pm._screen_body, t)
		union.append_array(t)
		body_min["drill:" + String(m)] = pm._screen_body.get_combined_minimum_size()

	# ---- 2. exhaustive: every top-level item of every menu -------------------
	var missing_top: Array = []
	var total_top := 0
	var reachable_menus: Array = []
	for m in MENUS:
		var p := _menu_popup(String(m))
		if p == null:
			continue
		var before := missing_top.size()
		for i in p.item_count:
			if p.is_item_separator(i):
				continue
			total_top += 1
			var txt := _clean(p.get_item_text(i))
			if _has(union, txt):
				continue
			## A submenu turned into chips is covered by a CHILD's text.
			var node := p.get_item_submenu(i)
			var covered := false
			if node != "":
				var sp := p.get_node_or_null(NodePath(node)) as PopupMenu
				if sp != null:
					for j in sp.item_count:
						if not sp.is_item_separator(j) \
								and _has(union, _clean(sp.get_item_text(j))):
							covered = true
							break
			if not covered:
				missing_top.append("%s > %s" % [m, txt])
		if missing_top.size() == before:
			reachable_menus.append(String(m))
	_check("every top-level row of all SEVEN menus is drawn somewhere (%d rows)" % total_top,
		missing_top.is_empty(), ", ".join(missing_top))
	print("VFY menus fully covered: [%s]" % ", ".join(reachable_menus))

	# ---- 3. one level DEEPER: every child of every top-level submenu ---------
	## Only over menus step 2 proved reachable: drilling a submenu of a menu the
	## root cannot reach would count rows nobody can get to.
	var missing_deep: Array = []
	var total_deep := 0
	for m in reachable_menus:
		var p := _menu_popup(String(m))
		if p == null:
			continue
		for i in p.item_count:
			var node := p.get_item_submenu(i)
			if node == "":
				continue
			var sp := p.get_node_or_null(NodePath(node)) as PopupMenu
			if sp == null:
				continue
			pm.open()
			pm._push(sp, _clean(p.get_item_text(i)), 3)
			await _frames(3)
			var st: Array = []
			_labels(pm._screen_body, st)
			for j in sp.item_count:
				if sp.is_item_separator(j):
					continue
				total_deep += 1
				var ct := _clean(sp.get_item_text(j))
				if _has(st, ct) or _has(union, ct):
					continue
				missing_deep.append("%s > %s > %s" % [m, _clean(p.get_item_text(i)), ct])
	_check("every child of every top-level submenu is drawn (%d rows)" % total_deep,
		missing_deep.is_empty(), ", ".join(missing_deep))

	# ---- 4. the destinations HEAD's root action rows reached -----------------
	for label in ["Civilization dock", "Open journey planner", "Travel library"]:
		_check("HEAD-era destination still on the surface: '%s'" % label,
			_has(union, String(label)))

	# ---- 5. LAYOUT: does any screen overflow the phone? ----------------------
	print("VFY --- layout, window %dx%d, phone_scale %.3f ---"
		% [want.x, want.y, app.phone_scale()])
	var over := 0
	for k in body_min:
		var mn: Vector2 = body_min[k]
		var bad: bool = mn.x > float(want.x)
		if bad:
			over += 1
		print("VFY    %-18s body combined_min = %7.1f x %8.1f   %s"
			% [k, mn.x, mn.y, "OVERFLOWS" if bad else "fits"])
	_check("no MORE screen or drill overflows the phone width", over == 0, "%d over" % over)

	print("VFY DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
