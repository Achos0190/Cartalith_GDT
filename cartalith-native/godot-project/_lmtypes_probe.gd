extends Node
## Ruling EH item 2 probe: `Assets ▸ Landmark types…` is one row that opens a browsable window
## (`shell/landmark_types_window.gd`), and nothing the old three-level cascade showed was lost.
##
##   desktop: godot --path . --resolution 1600x1000 _lmtypes_probe.tscn
##   touch:   godot --path . --resolution 1600x1000 _lmtypes_probe.tscn -- --force-touch
##   phone:   godot --path . --resolution 1080x2340 _lmtypes_probe.tscn -- --force-touch --density 2.8125
##
## **Windowed, never `--headless`; wiped user dir; under the lock** (`MISTAKES.md`, "Run a pixel probe"). Arguments read
## here: `--force-touch`, `--density D` (the shell reads both) and `--shots DIR`; anything else aborts with exit 2.
##
## What it protects. Every expectation is a LITERAL taken from the OLD cascade on origin/main
## (`a2200e5b`, `_build_landmark_types_menu`, dumped with `StubBridge` and with the live bridge before the
## change; the four `*_L2`/`*_L3` consts below are those dumps, signposts and separators removed), never read
## back from the code under test:
##   A. THE ROW (a real click on the menu): one `Landmark types…` row with no submenu, no child popup named
##      `LandmarkTypes`, and none of the cascade's fixed rows left on the menu.
##   B. THE WINDOW (a real click on that row): opens, titled `Landmark types`; non-blocking on the desktop
##      (not exclusive) and modal on touch.
##   C. NOTHING LOST: with the live bridge and with the fixture bridge, the window's family headers, type rows,
##      `Open <Family> in the dock` rows, hand count and the two destinations equal the baseline, in order;
##      the signpost is present.
##   D. DESTINATIONS (real clicks inside the window): `Open Physical in the dock` opens CIVIL ▸ Landmarks;
##      Landmark icons… opens the Asset library; Landmark label style… opens Cartography.
##   E. SEARCH, by TITLE (`MISTAKES.md`, "Move a command off the menu bar"): `Landmark types`, `Landmark icons`
##      and `Landmark label style` each appear exactly once; the window follows the bridge on the next open.
##   F. PHONE (touch at phone density): the window is a sheet, every button clears 44 dp and is no wider than
##      the scroller. Opened through `menus.gd`'s own entry, not by a finger (a synthetic tap cannot reach an
##      embedded dialog: `MISTAKES.md`); `_ms4_probe.gd` taps the real MENU row.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or no extension.

const LP := preload("res://_landmark_probe.gd")
const SIGN := "Rows here read state; they do not arm a type."

const STUB_L2: Array = [
	"Physical   3 of 5 armed · 41 placed",
	"Transportation   1 of 2 armed · 3 placed",
	"Placed by hand   0",
	"Landmark icons…",
	"Landmark label style…"
]
const STUB_L3: Array = [
	"physical|Peak   24 max · 24 placed · at cap",
	"physical|Waterfall   40 max · 11 placed · spacing",
	"physical|Cliff   30 max · 6 placed · no terrain",
	"physical|Ancient forest   off · was 2",
	"physical|Ice shelf   not buildable",
	"physical|Open Physical in the dock",
	"transportation|Mountain pass   16 max · 3 placed · candidates",
	"transportation|Pilgrim way   off · was 12",
	"transportation|Open Transportation in the dock"
]
const LIVE_L2: Array = [
	"Physical   11 of 15 armed · 0 placed",
	"Transport   7 of 8 armed · 0 placed",
	"Economic   5 of 6 armed · 0 placed",
	"Military   6 of 6 armed · 0 placed",
	"Cultural   0 of 8 armed · 0 placed",
	"Historical   2 of 6 armed · 0 placed",
	"Placed by hand   0",
	"Landmark icons…",
	"Landmark label style…"
]
const LIVE_L3: Array = [
	"physical|Peak   24 max",
	"physical|Ridge   20 max",
	"physical|Saddle   not buildable",
	"physical|Cliff   30 max",
	"physical|Gorge   8 max",
	"physical|Cave   12 max",
	"physical|Waterfall   40 max",
	"physical|Spring   30 max",
	"physical|Lake   16 max",
	"physical|Delta   not buildable",
	"physical|River confluence   20 max",
	"physical|Volcanic feature   10 max",
	"physical|Rock formation   20 max",
	"physical|Glacial feature   not buildable",
	"physical|Ancient forest   not buildable",
	"physical|Open Physical in the dock",
	"transport|Mountain pass   12 max",
	"transport|River crossing   not buildable",
	"transport|Ford   24 max",
	"transport|Bridge site   12 max",
	"transport|Road junction   20 max",
	"transport|Caravan station   10 max",
	"transport|Portage   8 max",
	"transport|Harbour   16 max",
	"transport|Open Transport in the dock",
	"economic|Mine   24 max",
	"economic|Quarry   20 max",
	"economic|Salt works   not buildable",
	"economic|Resource extraction site   16 max",
	"economic|Market site   12 max",
	"economic|Trade depot   10 max",
	"economic|Open Economic in the dock",
	"military|Fort   16 max",
	"military|Watchtower   20 max",
	"military|Fortified pass   8 max",
	"military|Fortified crossing   8 max",
	"military|Battlefield   12 max",
	"military|Border marker   16 max",
	"military|Open Military in the dock",
	"cultural|Shrine   not buildable",
	"cultural|Temple   not buildable",
	"cultural|Sacred grove   not buildable",
	"cultural|Sacred mountain   not buildable",
	"cultural|Pilgrimage site   not buildable",
	"cultural|Tomb   not buildable",
	"cultural|Monument   not buildable",
	"cultural|Ceremonial site   not buildable",
	"cultural|Open Cultural in the dock",
	"historical|Ruin   not buildable",
	"historical|Abandoned settlement   12 max",
	"historical|Ancient road   8 max",
	"historical|Historic battlefield   not buildable",
	"historical|Destroyed fortress   not buildable",
	"historical|Historic crossing   not buildable",
	"historical|Open Historical in the dock"
]


var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _win: AcceptDialog


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _move(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)
	await _settle(2)


## A REAL motion, press and release at `at` through the input system (touch builds emulate the mouse).
func _click_at(at: Vector2) -> void:
	await _move(at)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(3)


func _menu_button(title: String) -> MenuButton:
	for ch in _app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == title:
			return ch
	return null


func _index_of(p: PopupMenu, text: String) -> int:
	for i in p.item_count:
		if not p.is_item_separator(i) and p.get_item_text(i) == text:
			return i
	return -1


func _texts(p: PopupMenu) -> Array:
	var out: Array = []
	for i in p.item_count:
		if not p.is_item_separator(i):
			out.append(p.get_item_text(i))
	return out


## Presses the popup row whose text is `text` with REAL keys: Down until the popup's focused item is the row,
## then Enter. A synthetic pointer motion does not move a popup's hover focus here (measured: the focused item
## stayed -1 at every y of the popup), so the keys are the real-input route that reaches a row, as
## `_pc8planner_probe.gd` does for Story ▸ Journey planner. Returns false when the row was never focused.
func _click_row(p: PopupMenu, text: String) -> bool:
	var idx := _index_of(p, text)
	if idx < 0 or not p.visible:
		return false
	var guard := 0
	while p.get_focused_item() != idx and guard < 80:
		await _key(KEY_DOWN)
		guard += 1
	if p.get_focused_item() != idx:
		return false
	await _key(KEY_ENTER)
	return true


func _key(code: Key) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(2)


## What the window DREW, reassembled in the baseline's two-level order: L2 = family headers, hand, icons,
## labels; L3 = per family `family|type row` and `family|Open X in the dock`.
func _drawn(w: AcceptDialog) -> Dictionary:
	var l2: Array = []
	var l3: Array = []
	var hand := ""
	var icons := ""
	var labels := ""
	for n in w.find_children("*", "Control", true, false):
		if n is Label and n.name == "LtFamily":
			l2.append((n as Label).text)
		elif n is Button:
			var b := n as Button
			match String(b.get_meta("lt_role", b.name)):
				"type", "open":
					l3.append("%s|%s" % [String(b.get_meta("family")), b.text])
				"LtHand":
					hand = b.text
				"LtIcons":
					icons = b.text
				"LtLabels":
					labels = b.text
	l2.append_array([hand, icons, labels])
	return {"l2": l2, "l3": l3}


func _compare(w: AcceptDialog, want_l2: Array, want_l3: Array, tag: String) -> void:
	var d := _drawn(w)
	_ok(d["l2"] == want_l2, "%s: family headers, hand count and destinations equal the cascade's (got %s)" % [tag, str(d["l2"])])
	var got: Array = d["l3"]
	_ok(got.size() > 0 and got == want_l3, "%s: every type row and every Open-in-the-dock row equals the cascade's, in order (%d rows; first diff %s)" % [
		tag, got.size(), _first_diff(got, want_l3)])
	var sign := w.find_child("LtSignpost", true, false) as Label
	_ok(sign != null and sign.text == SIGN, "%s: the read-only signpost is on the window" % tag)


func _first_diff(a: Array, b: Array) -> String:
	for i in maxi(a.size(), b.size()):
		var x := str(a[i]) if i < a.size() else "<none>"
		var y := str(b[i]) if i < b.size() else "<none>"
		if x != y:
			return "#%d %s != %s" % [i, x, y]
	return "none"


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := args[i]
		if a == "--force-touch":
			pass
		elif a == "--density" and i + 1 < args.size():
			i += 1
		elif a == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		else:
			print("ABORT: unrecognised argument %s -- reads --force-touch, --density D and --shots <dir> only" % a)
			get_tree().quit(2)
			return
		i += 1
	get_tree().create_timer(540.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	var picker := _app.find_child("PhoneProjectPicker", true, false)
	if picker != null and picker is Window:
		(picker as Window).hide()
	await _settle(6)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, viewport %s ==" % [_mode, get_viewport().get_visible_rect().size])
	if _mode == "phone":
		await _phone()
	else:
		await _desktop()
	_finish()


func _desktop() -> void:
	print("== A. the row ==")
	## The desktop's Assets menu holds the row; the tablet's ☰ overflow holds it at its top level.
	var mb := _menu_button("Assets" if _mode == "desktop" else "☰")
	_ok(mb != null, "the menu button exists")
	await _click_at(mb.get_global_rect().get_center())
	await _settle(3)
	var ap := mb.get_popup()
	_ok(ap.visible, "a real click opens the menu that holds the row")
	var li := _index_of(ap, "Landmark types…")
	_ok(li >= 0, "the menu has a row titled Landmark types…")
	_ok(li >= 0 and ap.get_item_submenu(li) == "", "...and it owns no submenu")
	_ok(ap.get_node_or_null(NodePath("LandmarkTypes")) == null, "no child popup named LandmarkTypes survives")
	for gone in ["Landmark icons…", "Landmark label style…", "Placed by hand"]:
		var any := false
		for r in _texts(ap):
			any = any or String(r).begins_with(gone)
		_ok(not any, "the cascade's fixed row '%s' is not left on the menu" % gone)
	_ok(li >= 0 and ap.get_item_id(li) == 503, "the row's id is the fresh 503, not a retired one")
	print("== B. the window, by a real click on the row ==")
	_ok(await _click_row(ap, "Landmark types…"), "a real pointer found and pressed the row")
	await _settle(8)
	_win = _app.menus.get("_landmark_win")
	_ok(_win != null and _win.visible, "the Landmark types window opened")
	if _win == null:
		return
	_ok(_win.title == "Landmark types", "it is titled Landmark types (%s)" % _win.title)
	_ok(_win.exclusive == (_mode != "desktop"), "exclusive only off the desktop (exclusive=%s, mode %s)" % [str(_win.exclusive), _mode])
	print("== C. nothing lost (live bridge) ==")
	_compare(_win, LIVE_L2, LIVE_L3, "live")
	var ro := _win.find_child("LtReadout", true, false) as Label
	_ok(ro != null and ro.text == "6 families · 49 types · 31 armed · 0 placed", "the live readout totals the cascade's six family counts (%s)" % (ro.text if ro else "-"))
	await _shot("window")
	print("== C2. nothing lost (fixture bridge, with a landmark pass run) ==")
	var stub = LP.StubBridge.new()
	stub.has_world = true
	stub.ran = true
	var w2 := AcceptDialog.new()
	w2.set_script(load("res://shell/landmark_types_window.gd"))
	_app.add_child(w2)
	w2.setup(_app, stub, {})
	w2.exclusive = false  ## a touch form would make it modal beside the live window; only its rows are read
	w2.open()
	await _settle(6)
	_compare(w2, STUB_L2, STUB_L3, "fixture")
	print("== E1. the window follows the bridge on the next open ==")
	stub.armed["ancient_forest"] = true
	stub.caps["ancient_forest"] = 9
	w2.open()
	await _settle(4)
	var got := ""
	for b in w2.find_children("*", "Button", true, false):
		if (b as Button).get_meta("type_key", "") == "ancient_forest":
			got = (b as Button).text
	_ok(got == "Ancient forest   9 max · 0 placed" or got == "Ancient forest   9 max", "arming Ancient forest at 9 changes its row (%s)" % got)
	var hdr := ""
	for l in w2.find_children("LtFamily", "Label", true, false):
		if (l as Label).get_meta("family") == "physical":
			hdr = (l as Label).text
	_ok(hdr == "Physical   4 of 5 armed · 41 placed", "...and its family header recounts (%s)" % hdr)
	w2.hide()
	w2.queue_free()
	print("== D. destinations, by real clicks inside the live window ==")
	_app.select_domain("world")
	await _settle(4)
	var sc := _win.find_child("LtScroll", true, false) as ScrollContainer
	var open_b: Button = null
	for b in _win.find_children("*", "Button", true, false):
		if (b as Button).get_meta("lt_role", "") == "open" and (b as Button).get_meta("family") == "physical":
			open_b = b
	_ok(open_b != null, "the Open Physical in the dock button exists")
	if open_b != null:
		sc.ensure_control_visible(open_b)
		await _settle(3)
		await _click_at(_centre_in(_win, open_b))
		await _settle(6)
		_ok(_app._active_domain == "civilization", "it switched to CIVIL (%s)" % _app._active_domain)
		_ok(_win.visible == (_mode == "desktop"), "the window stays open on the desktop and closes when modal (visible=%s)" % str(_win.visible))
		var civ := _app.find_child("CivilizationWorkspace", true, false)
		var titles: Array = []
		for e in (civ.get("categories") as Array):
			if (e["body"] as Control).visible:
				titles.append(String(e["title"]))
		_ok(titles.has("Landmarks"), "CIVIL ▸ Landmarks is the open category (%s)" % str(titles))
	await _reopen()
	var ic := _win.find_child("LtIcons", true, false) as Button
	sc.ensure_control_visible(ic)
	await _settle(3)
	await _click_at(_centre_in(_win, ic))
	await _settle(10)
	var alw: Window = _app.asset_library_window
	_ok(alw != null and alw.visible, "Landmark icons… opened the Asset library")
	if alw != null:
		alw.hide()
	await _settle(3)
	await _reopen()
	var lb := _win.find_child("LtLabels", true, false) as Button
	sc.ensure_control_visible(lb)
	await _settle(3)
	_app.select_domain("world")
	await _settle(3)
	await _click_at(_centre_in(_win, lb))
	await _settle(8)
	_ok(_app._active_domain == "cartography", "Landmark label style… switched to CARTO (%s)" % _app._active_domain)
	print("== E. search by title ==")
	var counts := {"Landmark types": 0, "Landmark icons": 0, "Landmark label style": 0}
	var idx := CommandIndex.new()
	idx.build(_app, _app.bridge)
	for r in idx.all():
		var t := String((r as Dictionary).get("title", "")).replace("…", "").strip_edges()
		if counts.has(t):
			counts[t] = int(counts[t]) + 1
	_ok(counts == {"Landmark types": 1, "Landmark icons": 1, "Landmark label style": 1}, "each of the three titles is in the index exactly once (%s)" % str(counts))


## The centre of `ctl` in MAIN-viewport coordinates: a control's global rect is in its own embedded window's
## space, which starts at the window's `position` (`context_window.gd::_on_window_input` converts the same way).
func _centre_in(w: Window, ctl: Control) -> Vector2:
	return ctl.get_global_rect().get_center() + Vector2(w.position)


## Re-opens the window when a destination closed it (modal forms), through the menu's own entry.
func _reopen() -> void:
	if not _win.visible:
		_app.menus.call("_open_landmark_types")
		await _settle(6)


func _phone() -> void:
	print("== F. phone sheet ==")
	_app.menus.call("_open_landmark_types")
	await _settle(10)
	_win = _app.menus.get("_landmark_win")
	_ok(_win != null and _win.visible, "the window opened")
	if _win == null:
		return
	_ok(_win.exclusive, "it is modal on the phone")
	var vis := get_viewport().get_visible_rect()
	var sc := _win.find_child("LtScroll", true, false) as ScrollContainer
	## The window carries `content_scale_factor = phone_scale()` (`DccWidgets.phone_present`), so its own
	## coordinates are dp: 44 in them is 44 dp on glass.
	var floor_px := 44.0 - 1.0
	_ok(is_equal_approx(_win.content_scale_factor, float(_app.phone_scale())), "the sheet carries the phone's content scale (%s)" % str(_win.content_scale_factor))
	var small: Array = []
	var n := 0
	for b in _win.find_children("*", "Button", true, false):
		var bb := b as Button
		if not bb.is_visible_in_tree() or not bb.has_meta("lt_role") and not String(bb.name).begins_with("Lt"):
			continue
		n += 1
		if bb.size.y < floor_px:
			small.append("%s %.0f" % [bb.name, bb.size.y])
	_ok(n >= 55 and small.is_empty(), "%d buttons, every one clears the 44 dp floor (%.0f dp); under: %s" % [n, floor_px, str(small.slice(0, 4))])
	_ok(sc != null and sc.get_global_rect().size.x > 0.0 and sc.get_global_rect().size.x <= vis.size.x + 1.0,
		"the scroller fits the screen width (%s of %s)" % [str(sc.get_global_rect().size.x), str(vis.size.x)])
	var first: Button = null
	for b in _win.find_children("*", "Button", true, false):
		if (b as Button).get_meta("lt_role", "") == "type":
			first = b
			break
	_ok(first != null and first.get_global_rect().size.x <= sc.get_global_rect().size.x + 1.0,
		"a type row is no wider than the scroller")
	await _shot("phone")


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img != null:
		DirAccess.make_dir_recursive_absolute(_shots)
		img.save_png(_shots.path_join("lmtypes_%s_%s.png" % [_mode, tag]))
