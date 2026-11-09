extends Node
## MS-6 probe (`MOBILE_SHELL_PROPOSAL.md` section 6, MS-6: "the planner, timeline, conflicts and notes reachable
## without MENU"). Boots the real shell as a phone in a `--vp`-sized host `SubViewport` and drives REAL pointer events
## (a touch and its emulated mouse together) at the bottom bar's STORY cell, at the STORY sheet's four segments
## `TIME · JOURNEYS · CONFLICTS · NOTES` and at one row of the Journeys band, the planner's.
##
## Windowed, with a wiped user dir (the shell remembers coach marks and layout in the user dir):
##   godot --path . --resolution 1600x1000 _ms6_probe.tscn -- --vp 1080x2340 --density 2.625 --force-touch
##   godot --path . --resolution 1600x1000 _ms6_probe.tscn -- --vp 2340x1080 --density 2.625 --force-touch
##   godot --path . --resolution 1600x1000 _ms6_probe.tscn -- --vp 1440x3120 --density 3
##   godot --path . --resolution 1600x1000 _ms6_probe.tscn -- --vp 2388x1668 --density 2 --force-touch
##   godot --path . --resolution 1600x1000 _ms6_probe.tscn -- --vp 1080x1920 --density 3
## Arguments read by THIS file: `--vp WxH`. Accepted because the shell reads them: `--density D`, `--force-touch`.
## Anything else aborts with exit 2 (`MISTAKES.md`, "Write a probe's usage header").
##
## **Every expected value is a literal written by hand from the proposal and the Story menu** (the four captions, the
## 44 dp floor, the row texts `Timeline`, `Journey planner`, `Conflicts…`, `Notes for this place`); nothing is asserted
## against the constant it came from. Mutants run by hand and listed in the commit.
##
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node
var _dens := 1.0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
		print("FAIL: ", what)
	else:
		print("ok:   ", what)

func _frames(n: int) -> void:
	for k in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	if k >= 0 and k + 1 < args.size():
		return String(args[k + 1])
	return dflt

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a in ["--force-touch"]:
			i += 1
		elif a in ["--vp", "--density"]:
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

## One finger as the project delivers it with mouse emulation on: a touch and its emulated mouse button.
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

func _move(p: Vector2, from: Vector2, both: bool) -> void:
	var m := InputEventMouseMotion.new()
	m.position = p
	m.global_position = p
	m.relative = p - from
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	if both:
		var d := InputEventScreenDrag.new()
		d.index = 0
		d.position = p
		d.relative = p - from
		_vp.push_input(d)
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

func _tap(p: Vector2, both: bool = false) -> void:
	await _press(p, both)
	await _release(p, both)

## Hold on `p` for `ms`, slide in three steps to `to`, and (if `lift`) release there.
func _hold_slide(p: Vector2, to: Vector2, ms: int, lift: bool, both: bool = false) -> void:
	await _press(p, both)
	await get_tree().create_timer(ms / 1000.0).timeout
	var cur := p
	for s in 3:
		var nxt := p.lerp(to, float(s + 1) / 3.0)
		await _move(nxt, cur, both)
		cur = nxt
	if lift:
		await _release(to, both)

func _cell(id: String) -> Button:
	return (_app._phone_tab_cells.get(id, {}) as Dictionary).get("button") as Button

func _centre(id: String) -> Vector2:
	var b := _cell(id)
	return b.get_global_rect().get_center() if b != null else Vector2(-1, -1)

func _caption(id: String) -> String:
	var l := (_app._phone_tab_cells.get(id, {}) as Dictionary).get("label") as Label
	return l.text if l != null else "<none>"

func _texts(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	for c in n.get_children():
		_texts(c, out)

func _menu_has(text: String) -> bool:
	var out: Array = []
	_texts(_app._phone_menu, out)
	for t in out:
		if String(t).contains(text):
			return true
	return false

func _settle() -> void:
	await get_tree().create_timer(0.3).timeout
	await _frames(3)

# ---- the run -------------------------------------------------------------------------------------------------

## The STORY segment button for band key `key` (`StorySeg_<band>`), or null.
func _seg(key: String) -> Button:
	return _app._phone_menu._screen_body.find_child("StorySeg_" + key, true, false) as Button

func _row_centre(text: String) -> Vector2:
	var out := _find_text(_app._phone_menu._screen_body, text)
	return out.get_global_rect().get_center() if out != null else Vector2(-1, -1)

func _find_text(n: Node, text: String) -> Control:
	if (n is Label and (n as Label).text == text) or (n is Button and (n as Button).text == text):
		return n as Control
	for c in n.get_children():
		var f := _find_text(c, text)
		if f != null:
			return f
	return null

## Which segment is lit: the only one whose font colour no other segment shares ("<none or several>" otherwise).
func _lit_key() -> String:
	var keys := ["TIME", "JOURNEYS", "CONFLICTS & CAMPAIGNS", "NOTES"]
	var cols := {}
	for k in keys:
		var b := _seg(k)
		if b == null:
			return "<missing>"
		cols[k] = b.get_theme_color("font_color")
	var odd := []
	for k in keys:
		var same := 0
		for k2 in keys:
			if cols[k2] == cols[k]:
				same += 1
		if same == 1:
			odd.append(k)
	return odd[0] if odd.size() == 1 else "<none or several>"

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
	_ok(DccTheme.is_phone() and _app._phone_root != null, "boots as a phone")
	if not DccTheme.is_phone():
		return
	var pm = _app._phone_menu

	# 1. STORY opens on TIME with four segments in this order, one lit, each a 44 dp target wholly on screen
	await _tap(_centre("story"), true)
	await _settle()
	_ok(pm.is_open() and _app._phone_tab == "story", "the STORY tab opens the sheet, lit")
	_ok(String(pm._screen_head_title.text) == "STORY", "the sheet is titled STORY (got %s)" % pm._screen_head_title.text)
	var keys := ["TIME", "JOURNEYS", "CONFLICTS & CAMPAIGNS", "NOTES"]
	var caps := ["TIME", "JOURNEYS", "CONFLICTS", "NOTES"]
	var last_right := -1.0
	var screen := Rect2(Vector2.ZERO, Vector2(vw, vh))
	for n in 4:
		var b := _seg(keys[n])
		_ok(b != null, "segment %s is drawn" % caps[n])
		if b == null:
			return
		_ok(b.text == caps[n], "segment %d reads '%s' (got '%s')" % [n, caps[n], b.text])
		var r := b.get_global_rect()
		_ok(r.size.y / _dens >= 44.0 and r.size.x / _dens >= 44.0, "segment %s is at least 44 dp (%.1f x %.1f dp)" % [caps[n], r.size.x / _dens, r.size.y / _dens])
		_ok(screen.encloses(r), "segment %s is wholly on screen" % caps[n])
		_ok(r.position.x >= last_right - 0.5, "segment %s sits right of the one before it" % caps[n])
		last_right = r.end.x
	_ok(_lit_key() == "TIME", "TIME is lit on first open (got %s)" % _lit_key())
	_ok(_menu_has("Timeline"), "TIME shows the Timeline row")

	# 2. every segment by a real tap: its rows appear, the lit one follows, the stack does not grow
	var want_row := {"JOURNEYS": "Journey planner", "CONFLICTS & CAMPAIGNS": "Conflicts…", "NOTES": "this place", "TIME": "Timeline"}
	for k in ["JOURNEYS", "CONFLICTS & CAMPAIGNS", "NOTES", "TIME", "NOTES"]:
		await _tap(_seg(k).get_global_rect().get_center(), true)
		await _settle()
		## NOTES' first row reads "Notes for this place", or "Note for the selection" once the Context panel has Note.
		var has_row: bool = _menu_has(String(want_row[k])) or (k == "NOTES" and _menu_has("Note for the selection"))
		var seen: Array = []
		_texts(pm._screen_body, seen)
		_ok(has_row, "%s shows its row '%s' (drawn: %s)" % [k, want_row[k], str(seen)])
		_ok(_lit_key() == k, "%s is lit after its tap (got %s)" % [k, _lit_key()])
		_ok(pm._stack.size() == 1, "tapping a segment replaces the band; the stack stays one deep (%d)" % pm._stack.size())
	_ok(not _menu_has("Journey planner"), "the band left behind draws none of its rows")

	# 3. a band's own row works on a real tap: Journeys > Journey planner shows the planner, without MENU
	await _tap(_seg("JOURNEYS").get_global_rect().get_center(), true)
	await _settle()
	var pc := _row_centre("Journey planner")
	_ok(pc.x >= 0.0 and screen.has_point(pc), "the Journey planner row is on screen")
	await _tap(pc, true)
	await _settle()
	_ok(_app.journey_planner_view != null and is_instance_valid(_app.journey_planner_view) and _app.journey_planner_view.is_visible_in_tree(),
		"tapping the Journey planner row shows the planner")
	_ok(not pm.is_open(), "the sheet is closed behind the planner")
	_app._close_all_phone_overlays()
	await _settle()

	# 4. the tab remembers its band, and system back leaves in one step from any segment
	await _tap(_centre("story"), true)
	await _settle()
	_ok(pm.is_open() and _lit_key() == "JOURNEYS", "STORY reopens on the band last shown (got %s)" % _lit_key())
	await _tap(_seg("CONFLICTS & CAMPAIGNS").get_global_rect().get_center(), true)
	await _settle()
	_ok(pm.go_back() and not pm.is_open(), "system back from a segment closes the sheet in one step")
	_app._close_all_phone_overlays()
	await _settle()

	# 5. a caller can open straight on a band, and an unknown band falls back to the first
	pm.open_story("NOTES")
	await _settle()
	_ok(_lit_key() == "NOTES" and (_menu_has("Notes for this place") or _menu_has("Note for the selection")), "open_story('NOTES') opens on NOTES")
	pm.open_story("NO SUCH BAND")
	await _settle()
	_ok(_lit_key() == "TIME", "an unknown band falls back to TIME (got %s)" % _lit_key())
	pm.close()
	await _settle()

	# 6. MENU still lists Story (the desktop popup, row for row), so nothing the phone reached before is lost
	await _tap(_centre("menu"), true)
	await _settle()
	_ok(pm.is_open() and _menu_has("Story"), "MENU still lists Story")
	await _tap(_centre("menu"), true)
	await _settle()
