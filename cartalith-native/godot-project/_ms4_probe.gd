extends Node
## MS-4 probe (`MOBILE_SHELL_PROPOSAL.md` section 6, MS-4: "MENU mirrors the eight menus: every one of the 360
## rows reachable in at most three taps from MENU"; Ruling DX Q13: MENU follows the mockup, row for row).
## Two modes, because the desktop menus are the definition and the phone's MENU is checked AGAINST them:
##
##   DUMP    (a DESKTOP shell, no `--force-touch`): walks the live menu bar the way a desktop user opens it
##           (`about_to_popup`, then every item, then every submenu) and writes the structure to a JSON file.
##   WALK    (a PHONE shell, `--force-touch --expect FILE`): boots the real shell as a phone in a `--vp`-sized
##           host `SubViewport`, taps MENU, and drives REAL pointer events (a touch and its emulated mouse button
##           together) through every menu and every submenu, comparing each drawn screen row for row with the
##           dump: the eight titles in the bar's order, each menu's row titles, band captions, row counts, the
##           count on each drill row, and the depth in taps. Then the homes of the rows the fourteen deleted
##           screens drew (`drafts/mobile-shell/README.md`, "Where each phone-only row goes") are tapped for real.
##
## Windowed, with a wiped user dir, under the lock (`MISTAKES.md`, "Run a pixel probe"):
##   godot --path . --resolution 1600x1000 _ms4_probe.tscn -- --vp 1600x1000 --dump C:/cw/ms4_scr/desk.json
##   godot --path . --resolution 1600x1000 _ms4_probe.tscn -- --vp 1080x2340 --density 2.625 --force-touch --expect C:/cw/ms4_scr/desk.json
## (also 2340x1080 @2.625, 1440x3120 @3, 2388x1668 @2 -- a phone under Ruling EG -- and 1080x1920 @3 = 360 dp).
## Arguments read by THIS file: `--vp WxH`, `--dump FILE`, `--expect FILE`. Accepted because the shell reads them:
## `--density D`, `--force-touch`. Anything else aborts with exit 2 (`MISTAKES.md`, "Write a probe's usage header").
##
## **Expected values are literals or come from the OTHER shell.** The eight titles are written out below; every
## row comparison is phone against a dump taken from a separately booted desktop, so a change to `_fill_popup()`
## cannot make its own expectation. The three phone-only differences are a literal table (`PHONE_ONLY`) with the
## reason beside each. Mutants run by hand and listed in the commit: the last menu dropped from `_fill_root()`,
## the leading-separator caption test of `_fill_popup()` reverted, `_build_gesture_reference_row()`'s
## `is_phone()` inverted, `_open_landmark_dock()`'s phone sheet opener removed, the root count digit changed.
##
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or headless.

const MENUS: Array = ["File", "Edit", "Assets", "Records", "Story", "Preferences", "Window", "Help"]
## Two submenus whose `about_to_popup` handlers are not observers (`phone_menu.gd::NO_EXPAND`: AtlasCache evicts
## baked chunks, GpuDevices enumerates wgpu adapters). Their drill rows and counts are compared; they are not entered.
const NO_ENTER: Array = ["Atlas cache", "GPU devices", "Devices"]
## Rows the DESKTOP builds and the phone does not, each a stated, cited decision in `menus.gd` and not a draw fault.
## Keyed by the menu path the rows sit in; a band caption (MODULES) is listed with its rows.
const DESKTOP_ONLY: Dictionary = {
	## Ruling DX PQ10: a stated exception to row for row (`menus.gd::_build_map_surface_rows()`): the phone's menu
	## bar is a hidden model, no accelerator fires there, and the phone opens the card by its own gesture.
	"Edit": ["Context card", "Cartwheel at the pointer"],
	## The pop-out window row (`POPOUT_WINDOWS_SCOPE.md`): sub-windows are a desktop feature.
	"Preferences": ["Open panels in their own window"],
	## Ruling DY: the Context panel's MODULES band and Reset panel are desktop only (`menus.gd`, `is_phone()`).
	"Window > Context panel": ["MODULES", "History", "Economy", "Military", "Faith, culture & relations", "Territory",
		"City layout", "Here", "Routes & journeys", "Links", "Recent & bookmarks", "Reset panel"],
}
## Rows the PHONE adds, as [anchor title prefix, new title]: the new row sits directly after the anchor.
const PHONE_ONLY: Dictionary = {
	"Help": [["Keyboard reference…", "Gesture reference…"]],  ## MS-4, draft 09: the mockup's own row.
	"Preferences": [["Handedness", "UI scale"]],  ## MS-1 S5: touch builds only (MQ-4, Ruling DY).
}
## Ids of the phone-only rows, which the desktop dump cannot hold (menus.gd::ID_HELP_GESTURES).
const PHONE_ONLY_IDS: Dictionary = {"Gesture reference": 919}
## Levels whose rows are the app's live state, not the menu's structure: compared as "draws at least one row".
const DYNAMIC_LEVELS: Array = ["Window > Open windows"]
## Drill rows whose COUNT is live state (the adapters present, the windows open), not menu structure.
const DYNAMIC_COUNTS: Array = ["Devices", "Open windows"]
## Rows whose value is a live readout that differs between two processes; digits are compared as '#'.
const LIVE_VALUE_PREFIXES: Array = ["Working set"]

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node
var _dens := 1.0
var _expect: Dictionary = {}
var _max_taps := 0
var _rows_seen := 0
var _taps_total := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
		print("FAIL: ", what)
	else:
		print("ok:   ", what)

func _quiet(cond: bool, what: String) -> void:
	## Counted and printed only on failure: per-row checks number in the hundreds.
	_checks += 1
	if not cond:
		_fails += 1
		print("FAIL: ", what)

func _frames(n: int) -> void:
	for k in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	if k >= 0 and k + 1 < args.size():
		return String(args[k + 1])
	return dflt

func _clean(text: String) -> String:
	var out := text.strip_edges()
	if out.ends_with("▸"):
		out = out.substr(0, out.length() - 1).strip_edges()
	return out

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("ABORT: run WINDOWED (the dummy driver cannot rasterise or time a frame)")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a in ["--force-touch"]:
			i += 1
		elif a in ["--vp", "--density", "--dump", "--expect"]:
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			get_tree().quit(2)
			return
	var parts := _arg("--vp", "").split("x")
	if parts.size() != 2:
		print("ABORT: needs --vp WxH")
		get_tree().quit(2)
		return
	_dens = float(_arg("--density", "1"))
	await _run(int(parts[0]), int(parts[1]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

# ---- pointer helpers: real events pushed at the host viewport ------------------------------------------------

func _press(p: Vector2, both: bool) -> void:
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	if both:
		var t := InputEventScreenTouch.new()
		t.index = 0
		t.pressed = true
		t.position = p
		_vp.push_input(t)
	_vp.push_input(m)
	await _frames(2)

func _release(p: Vector2, both: bool) -> void:
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = false
	m.position = p
	m.global_position = p
	if both:
		var t := InputEventScreenTouch.new()
		t.index = 0
		t.pressed = false
		t.position = p
		_vp.push_input(t)
	_vp.push_input(m)
	await _frames(3)

func _tap(p: Vector2, both: bool = true) -> void:
	_taps_total += 1
	await _press(p, both)
	await _release(p, both)

func _settle() -> void:
	await get_tree().create_timer(0.25).timeout
	await _frames(3)

func _cell(id: String) -> Button:
	return (_app._phone_tab_cells.get(id, {}) as Dictionary).get("button") as Button

func _centre(id: String) -> Vector2:
	var b := _cell(id)
	return b.get_global_rect().get_center() if b != null else Vector2(-1, -1)

func _texts(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	for c in n.get_children():
		_texts(c, out)

# ---- the DUMP side (desktop) ---------------------------------------------------------------------------------

func _dump_popup(p: PopupMenu, depth: int) -> Array:
	p.about_to_popup.emit()
	var out: Array = []
	for i in p.item_count:
		if p.is_item_separator(i):
			var cap := _clean(p.get_item_text(i))
			if cap != "":
				out.append({"k": "band", "t": cap})
			continue
		var e := {"k": "row", "t": _clean(p.get_item_text(i)), "dis": p.is_item_disabled(i),
			"id": p.get_item_id(i)}
		var sn := p.get_item_submenu(i)
		if sn != "" and not p.is_item_disabled(i):
			var sub := p.get_node_or_null(NodePath(sn)) as PopupMenu
			if sub != null:
				e["k"] = "sub"
				var n := 0
				for j in sub.item_count:
					if not sub.is_item_separator(j):
						n += 1
				e["n"] = n
				if not NO_ENTER.has(e["t"]):
					e["c"] = _dump_popup(sub, depth + 1)
		out.append(e)
	return out

func _dump(path: String) -> void:
	var bar: Node = _app.menu_bar_row
	var all := {}
	var order: Array = []
	for c in bar.get_children():
		if c is MenuButton:
			var t := String((c as MenuButton).text)
			order.append(t)
			var p := (c as MenuButton).get_popup()
			var n := 0
			for j in p.item_count:
				if not p.is_item_separator(j):
					n += 1
			all[t] = {"n": n, "rows": _dump_popup(p, 1)}
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(JSON.stringify({"order": order, "menus": all}))
	f.close()
	_ok(order.size() == 8, "desktop dump: %d menus on the bar (%s)" % [order.size(), str(order)])

# ---- the WALK side (phone) -----------------------------------------------------------------------------------

func _pm() -> Node:
	return _app._phone_menu

func _row_title(row: Control) -> String:
	## PanelContainer > MarginContainer > HBoxContainer > [lead] VBoxContainer > title Label (`_row()`).
	var line := (row.get_child(0) as Node).get_child(0) as Node
	for c in line.get_children():
		if c is VBoxContainer:
			var l := (c as Node).get_child(0) as Label
			return l.text if l != null else ""
	return ""

func _row_trail(row: Control) -> String:
	## The right-hand count of a drill row: the Label after the text column holding only digits.
	var line := (row.get_child(0) as Node).get_child(0) as Node
	var seen_col := false
	for c in line.get_children():
		if c is VBoxContainer:
			seen_col = true
		elif seen_col and c is Label and String((c as Label).text).is_valid_int():
			return (c as Label).text
	return ""

## The screen body's rows as `[{k, t, ctl}]`: a band is a MarginContainer holding its caption Label, a row is a
## PanelContainer; rules and the padded tail are skipped.
func _drawn() -> Array:
	var out: Array = []
	for c in _pm()._screen_body.get_children():
		if c is PanelContainer:
			out.append({"k": "row", "t": _clean(_row_title(c)), "ctl": c})
		elif c is MarginContainer and c.get_child_count() > 0 and c.get_child(0) is Label 				and String((c.get_child(0) as Label).text) != "":
			## A band with no caption is the gap an unlabelled `add_separator()` draws, not a row.
			out.append({"k": "band", "t": String((c.get_child(0) as Label).text), "ctl": c})
	return out

func _tap_ctl(c: Control) -> void:
	var sc: ScrollContainer = _pm()._screen_scroll
	sc.ensure_control_visible(c)
	await _frames(3)
	await _tap(c.get_global_rect().get_center())
	await _settle()

func _back() -> void:
	var b: Button = _pm()._screen_back
	await _tap(b.get_global_rect().get_center())
	await _settle()

## Expected rows for one menu level: the desktop's rows with the cited desktop-only rows removed and the cited
## phone-only rows inserted after their anchors.
func _expected_level(path: String, rows: Array) -> Array:
	var out: Array = []
	var gone: Array = DESKTOP_ONLY.get(path, [])
	var adds: Array = PHONE_ONLY.get(path, [])
	for r in rows:
		if gone.has(String(r["t"])):
			continue
		out.append(r)
		for ad in adds:
			if r["k"] != "band" and String(r["t"]).begins_with(String(ad[0])):
				out.append({"k": "row", "t": String(ad[1]), "dis": false, "phone_only": true})
	return out

func _norm(t: String) -> String:
	for pre in LIVE_VALUE_PREFIXES:
		if t.begins_with(String(pre)):
			var rx := RegEx.new()
			rx.compile("[0-9]+([.][0-9]+)?")
			return rx.sub(t, "#", true)
	return t

func _count_of(path: String, rows: Array) -> int:
	var n := 0
	for r in _expected_level(path, rows):
		if r["k"] != "band":
			n += 1
	return n

## One level: compare the drawn screen with `rows` (row for row, in order), then enter every drill row by a real
## tap and recurse; `taps` is how many taps it took to display this level from MENU.
func _walk_level(menu: String, path: String, rows: Array, taps: int, depth: int) -> void:
	var want := _expected_level(path, rows)
	var got := _drawn()
	var gt: Array = []
	for r in got:
		gt.append(String(r["t"]))
	var wt2: Array = []
	for r in want:
		wt2.append(String(r["t"]).to_upper() if r["k"] == "band" else _norm(String(r["t"])))
	for k in gt.size():
		gt[k] = _norm(String(gt[k]))
	if DYNAMIC_LEVELS.has(path):
		_quiet(not gt.is_empty(), "%s: a live level draws at least one row" % path)
		_rows_seen += gt.size()
		return
	_quiet(gt == wt2, "%s: drawn rows equal the desktop's, in order (%d want, %d got) %s" % [path, wt2.size(), gt.size(),
		"" if gt == wt2 else "\n  want " + str(wt2) + "\n  got  " + str(gt)])
	_rows_seen += gt.size()
	_max_taps = maxi(_max_taps, taps)
	# touch floor: every drawn row is at least 44 dp tall (phone_scale px per dp, one px of rounding)
	var floor_px: float = 44.0 * float(_app.phone_scale()) - 1.0
	for g in got:
		if g["k"] == "row":
			_quiet((g["ctl"] as Control).size.y >= floor_px, "%s: row '%s' is %.1f px tall, floor %.1f" % [
				path, g["t"], (g["ctl"] as Control).size.y, floor_px])
	# drill rows carry the desktop's count
	for k in mini(want.size(), got.size()):
		var w: Dictionary = want[k]
		var dyn := false
		for dc in DYNAMIC_COUNTS:
			dyn = dyn or String(w["t"]).begins_with(String(dc))
		if not dyn and w["k"] == "sub" and k < got.size() and got[k]["k"] == "row" and got[k]["t"] == w["t"]:
			var want_n: int = _count_of(path + " > " + String(w["t"]), w["c"]) if w.has("c") else int(w["n"])
			_quiet(_row_trail(got[k]["ctl"]) == str(want_n), "%s: '%s' shows its count %d (drew '%s')" % [
				path, w["t"], want_n, _row_trail(got[k]["ctl"])])
	# enter every drill row by a real tap. Rows are matched by position, not by title: Window has a check row and a
	# drill row both called "Context panel".
	var j := 0
	var drawn_now := _drawn()
	for r in want:
		var ctl: Control = null
		var jj := j
		while jj < drawn_now.size():
			if drawn_now[jj]["k"] == ("band" if r["k"] == "band" else "row") 					and drawn_now[jj]["t"] == (String(r["t"]).to_upper() if r["k"] == "band" else String(r["t"])):
				ctl = drawn_now[jj]["ctl"]
				j = jj + 1
				break
			jj += 1
		if r["k"] != "sub" or not r.has("c"):
			continue
		if ctl == null:
			_quiet(false, "%s: no drawn row '%s' to tap" % [path, r["t"]])
			continue
		await _tap_ctl(ctl)
		await _walk_level(menu, path + " > " + String(r["t"]), r["c"], taps + 1, depth + 1)
		await _back()
		_quiet(not _drawn().is_empty() and _pm().is_open(), "%s: back from '%s' returns to the menu screen" % [path, r["t"]])
		drawn_now = _drawn()

func _tap_root_row(title: String) -> bool:
	for g in _drawn():
		if g["k"] == "row" and g["t"] == title:
			await _tap_ctl(g["ctl"])
			return true
	return false

func _open_menu() -> void:
	## MENU is the bar's last tab; the probe taps it as a finger does. A second tap while open would close it.
	if not _pm().is_open():
		await _tap(_centre("menu"))
		await _settle()

func _run(vw: int, vh: int) -> void:
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	var dump_path := _arg("--dump", "")
	if dump_path != "":
		_ok(not DccTheme.is_phone(), "dump mode boots a desktop shell")
		_dump(dump_path)
		return
	_ok(DccTheme.is_phone() and _app._phone_root != null, "boots as a phone")
	if not DccTheme.is_phone():
		return
	var ef := FileAccess.open(_arg("--expect", ""), FileAccess.READ)
	if ef == null:
		_ok(false, "--expect FILE is readable")
		return
	_expect = JSON.parse_string(ef.get_as_text())
	ef.close()
	await _walk_root()
	await _homes()

func _walk_root() -> void:
	# 1. MENU opens from its tab and its root is Menus band + eight rows + Status, nothing from the old root.
	await _open_menu()
	_ok(_pm().is_open(), "a real tap on the MENU tab opens MENU")
	var root := _drawn()
	var titles: Array = []
	var bands: Array = []
	for g in root:
		if g["k"] == "row":
			titles.append(g["t"])
		else:
			bands.append(g["t"])
	_ok(_expect["order"] == MENUS, "the desktop bar's menus are the eight titles in order (%s)" % str(_expect["order"]))
	var menu_rows: Array = []
	for g in root:
		if MENUS.has(g["t"]) and g["k"] == "row":
			menu_rows.append(g["t"])
	_ok(menu_rows == MENUS, "MENU's root lists exactly the eight desktop menus in the bar's order (%s)" % str(menu_rows))
	_ok(bands.has("MENUS"), "MENU's root has the MENUS band (%s)" % str(bands))
	for gone in ["Project", "Civilization", "Data manager", "Asset library", "Travel library", "Simulation",
			"Help & about", "Not on the MORE list"]:
		_ok(not titles.has(gone) and not bands.has(gone.to_upper()), "the old root's '%s' is gone" % gone)
	var counts_ok := true
	for t in MENUS:
		for g in root:
			if g["k"] == "row" and g["t"] == t:
				var want_n := _count_of(String(t), (_expect["menus"][t] as Dictionary)["rows"])
				if _row_trail(g["ctl"]) != str(want_n):
					counts_ok = false
					print("  count mismatch on ", t, ": drew ", _row_trail(g["ctl"]), " want ", want_n)
	_ok(counts_ok, "each root row's count is its desktop menu's row count")
	_ok(String(_pm()._screen_head_title.text) == "MENU", "the root's title is MENU (%s)" % _pm()._screen_head_title.text)

	# 2. every menu, row for row, by real taps, to every depth
	var total_rows := 0
	for t in MENUS:
		await _open_menu()
		var opened := await _tap_root_row(t)
		_ok(opened, "tap '%s' on MENU's root" % t)
		if not opened:
			continue
		_ok(String(_pm()._screen_head_title.text) == t.to_upper(), "%s: the screen is titled with the menu's own name (%s)" % [
			t, _pm()._screen_head_title.text])
		await _walk_level(t, t, (_expect["menus"][t] as Dictionary)["rows"], 1, 1)
		await _back()
		_ok(String(_pm()._screen_head_title.text) == "MENU", "%s: back returns to MENU's root" % t)
	print("info: %d rows drawn across the eight menus, deepest level displayed after %d taps, %d taps in all" % [
		_rows_seen, _max_taps, _taps_total])
	_ok(_max_taps <= 3, "every row is displayed within three taps of MENU (deepest %d)" % _max_taps)
	_ok(_rows_seen >= 300, "the walk covered at least 300 drawn rows (%d)" % _rows_seen)
	_pm().close()

# ---- the homes of the rows the fourteen deleted screens drew --------------------------------------------------

func _row_by_title(title: String) -> Control:
	for g in _drawn():
		if g["k"] == "row" and g["t"] == title:
			return g["ctl"]
	return null

## Opens MENU fresh (closing whatever is up first) and taps down `path` by title prefix, one real tap per title.
func _go(path: Array) -> bool:
	_close_windows()
	_app._close_all_phone_overlays()
	_pm().close()
	await _frames(2)
	await _open_menu()
	for t in path:
		var hit: Control = null
		for g in _drawn():
			if g["k"] == "row" and String(g["t"]).begins_with(String(t)):
				hit = g["ctl"]
				break
		if hit == null:
			print("  _go: no row starting '", t, "' on ", str(path))
			return false
		await _tap_ctl(hit)
	return true

## The phone presents every dialog as an embedded Window (DccWidgets.phone_present); a command that opened one
## leaves it over the shell, so the probe hides them before the next real tap.
func _close_windows() -> void:
	for w in _app.find_children("*", "Window", true, false):
		if (w as Window).visible:
			(w as Window).hide()

## The id the desktop dump holds for the row at the end of `path`, or -2.
func _dump_id(path: Array) -> int:
	var rows: Array = (_expect["menus"][String(path[0])] as Dictionary)["rows"]
	var id := -2
	for k in range(1, path.size()):
		var next: Array = []
		for r in rows:
			if r["k"] != "band" and String(r["t"]).begins_with(String(path[k])):
				id = int(r["id"])
				next = r.get("c", [])
				break
		rows = next
	return id

var _last_id := -999

func _record(p: PopupMenu) -> void:
	if not p.id_pressed.is_connected(_on_id):
		p.id_pressed.connect(_on_id)

func _on_id(id: int) -> void:
	_last_id = id

func _popups() -> Array:
	var out: Array = []
	for mb in _pm()._menu_buttons():
		var p := (mb as MenuButton).get_popup()
		out.append(p)
		for c in p.get_children():
			if c is PopupMenu:
				out.append(c)
				for d in (c as PopupMenu).get_children():
					if d is PopupMenu:
						out.append(d)
	return out

## Taps the LAST title of `path` (an action row) and checks it ran the desktop's command: the id the popup emitted
## equals the id the desktop dump has for that row, and the row met the 44 dp floor.
func _leaf(path: Array, what: String) -> void:
	for p in _popups():
		_record(p)
	_last_id = -999
	var reached := await _go(path.slice(0, path.size() - 1))
	_ok(reached, "%s: the menu above it is reached by real taps %s" % [what, str(path)])
	if not reached:
		return
	var want := _dump_id(path)
	if want == -2 and PHONE_ONLY_IDS.has(path[path.size() - 1]):
		want = int(PHONE_ONLY_IDS[path[path.size() - 1]])
	var last := String(path[path.size() - 1])
	var hit: Control = null
	for g in _drawn():
		if g["k"] == "row" and String(g["t"]).begins_with(last):
			hit = g["ctl"]
			break
	_ok(hit != null, "%s: the row is drawn" % what)
	if hit == null:
		return
	_ok((hit as Control).size.y >= 44.0 * float(_app.phone_scale()) - 1.0, "%s: the row meets the 44 dp floor" % what)
	await _tap_ctl(hit)
	await _frames(4)
	_ok(_last_id == want and want >= 0, "%s: the tap ran the desktop's command (id %d, desktop %d)" % [what, _last_id, want])

func _homes() -> void:
	# Help > Gesture reference... is the gesture screen's only door: third row, opens it, back returns to MENU.
	await _go(["Help"])
	var help_rows: Array = []
	for g in _drawn():
		help_rows.append(g["t"])
	_ok(help_rows.size() > 2 and help_rows[2] == "Gesture reference…", "Help lists Gesture reference… third (%s)" % str(help_rows))
	await _leaf(["Help", "Gesture reference"], "Help > Gesture reference…")
	await _frames(6)
	await _settle()
	_ok(_pm().is_open() and String(_pm()._screen_head_title.text) == "GESTURE REFERENCE",
		"the gesture reference opened (%s)" % _pm()._screen_head_title.text)
	await _back()
	_ok(String(_pm()._screen_head_title.text) == "MENU", "system back from the gesture reference returns to MENU (%s)" % _pm()._screen_head_title.text)

	# Assets > Landmark types… (Ruling EH item 2): one row, one tap from Assets, which opens the Landmark types
	# window as a sheet. The window's own buttons are inside an embedded dialog a synthetic tap cannot reach
	# (MISTAKES.md, "Plan a phone probe that must reach a DIALOG"), so `_lmtypes_probe.gd` proves them; this proves
	# the phone's door and that its window opened with the engine's families in it.
	_app.select_domain("world")
	await _frames(4)
	await _leaf(["Assets", "Landmark types"], "Assets > Landmark types…")
	await _frames(10)
	await _settle()
	var lmw = _app.menus.get("_landmark_win")
	_ok(lmw != null and lmw.visible, "the Landmark types row opened its window on the phone")
	if lmw != null:
		var fam_labels: Array = []
		for l in lmw.find_children("LtFamily", "Label", true, false):
			fam_labels.append((l as Label).text)
		_ok(fam_labels.size() == 6 and String(fam_labels[0]).begins_with("Physical"),
			"the sheet lists the engine's six families, Physical first (%s)" % str(fam_labels.slice(0, 2)))
		# The phone's door to the landmark settings is now the window's own "Open <family> in the dock" button.
		# A synthetic tap cannot reach it, so it is invoked through its own signal: the destination handler, not the
		# touch path, and `_open_landmark_dock()`'s phone sheet opener is what is under test.
		for b in lmw.find_children("*", "Button", true, false):
			if (b as Button).get_meta("lt_role", "") == "open" and (b as Button).get_meta("family") == "physical":
				(b as Button).pressed.emit()
				break
		await _frames(8)
		await _settle()
		_ok(_app._active_domain == "civilization", "its Open Physical in the dock button switched the domain to CIVIL (%s)" % _app._active_domain)
		_ok(_app.left_dock.is_visible_in_tree(), "...and opened the dock on the phone (left sheet)")
		lmw.hide()

	# The six civilization tool rows of the deleted Civilization screen: Window > Workspace > Civilization, then
	# Window > Left dock, then each tool's own button in the CIVIL dock's TOOLS block, tapped for real.
	_app._set_sheet_open("left", false)
	_app.select_domain("world")
	await _frames(4)
	await _leaf(["Window", "Workspace", "Civilization"], "Window > Workspace > Civilization")
	await _frames(6)
	_ok(_app._active_domain == "civilization", "Window > Workspace > Civilization selects CIVIL (%s)" % _app._active_domain)
	await _leaf(["Window", "Left dock"], "Window > Left dock")
	await _frames(8)
	await _settle()
	_ok(_app.left_dock.is_visible_in_tree(), "Window > Left dock shows the CIVIL dock")
	var missing: Array = []
	var armed_ok: Array = []
	for tid in ["settlement", "territory", "territory_lasso", "way", "route", "conflict"]:
		var btn: Button = null
		for b in _app.find_children("*", "Button", true, false):
			if String((b as Button).get_meta(DccWidgets.TOOL_ID_META, "")) == tid and (b as Button).is_visible_in_tree():
				btn = b
				break
		if btn == null:
			missing.append(tid)
			continue
		var sc: ScrollContainer = _app._left_dock_scroll
		if sc != null and sc.is_ancestor_of(btn):
			sc.ensure_control_visible(btn)
			await _frames(3)
		_app.arm_tool("inspect")
		await _frames(2)
		await _tap(btn.get_global_rect().get_center())
		await _frames(4)
		if _app.armed_tool == tid:
			armed_ok.append(tid)
	_ok(missing.is_empty() and armed_ok.size() == 6, "all six civilization tools are armable by a real tap in the CIVIL dock (missing %s, armed %s)" % [str(missing), str(armed_ok)])
	_app.arm_tool("inspect")
	_app._set_sheet_open("left", false)

	# Commands whose rows moved: the same id as the desktop's, one tap from their menu.
	for spec in [
			[["Story", "Journey planner"], "Story > Journey planner"],
			[["Story", "Travel library"], "Story > Travel library…"],
			[["Records", "⧉ Data manager"], "Records > Data manager"],
			[["File", "Export", "Maps"], "File > Export > Maps (tiles)"],
			[["Assets", "Icon families", "Feature icons"], "Assets > Icon families > Feature icons"],
			[["Window", "Timeline"], "Window > Timeline"],
			[["Help", "About"], "Help > About"]]:
		await _leaf(spec[0], String(spec[1]))
		await _frames(6)
		_app._close_all_phone_overlays()

	# File and Preferences rows the old Project and Preferences screens drew first.
	await _go(["File"])
	var file_rows: Array = []
	for g in _drawn():
		file_rows.append(g["t"])
	for want_row in ["New world…", "Generate world", "Open project…", "Recent worlds", "Save project", "Save as…", "Autosave"]:
		_ok(file_rows.has(want_row), "File draws '%s'" % want_row)
	_ok(String(file_rows[1]) == "Generate world", "File's second row is Generate world (Ruling DY)")
	await _go(["Preferences"])
	var th := false
	var un := false
	for g in _drawn():
		th = th or String(g["t"]).begins_with("Theme")
		un = un or String(g["t"]).begins_with("Units")
	_ok(th and un, "Preferences draws Theme and Units")
	# Status band: the GPU row the Help screen's Engine row carried.
	_pm().close()
	await _open_menu()
	var root_rows: Array = []
	for g in _drawn():
		root_rows.append(g["t"])
	if String(_pm()._slot("top_gpu")) != "":
		_ok(root_rows.has("GPU"), "MENU's Status band carries the GPU readout (%s)" % str(root_rows))
	_pm().close()
