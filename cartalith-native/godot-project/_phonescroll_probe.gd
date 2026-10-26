extends Node
## Phone touch-scroll probe (owner finding 2026-10-26, OnePlus 6T: "scrolling doesn't work in all menus").
##
##   godot --path . --resolution 393x852 _phonescroll_probe.tscn -- --force-touch
##   godot --path . --resolution 852x393 _phonescroll_probe.tscn -- --force-touch        (landscape)
##
## Arguments read: `--force-touch`, `--vp WxH` (headless only), `--survey` (print every row, never fail), `--only <substring>` (run the
## surfaces whose name contains it), `--shots <dir>`. Anything else aborts with exit 2. Runs windowed or
## `--headless` (input routing does not need a texture); a wiped user dir is required (the assistant must be off).
##
## ## What it protects
## A touch drag that STARTS ON A CHILD (a button, a panel, a check box, a text field, a list) of a scrollable
## surface must scroll that surface, the same as one that starts on empty space. Godot delivers a touch event to
## the control under the finger and bubbles it to the parent only while the control's `mouse_filter` is not STOP,
## so every STOP leaf under a `ScrollContainer` ends the gesture and the scroller never sees it (the cause the
## previous phone pass found for MORE and search, and fixed at two call sites). This probe walks EVERY surface
## the phone can reach and checks the cause rather than the symptom.
##
## ## How
## For each surface it scans the whole tree (the main window and every embedded `Window`) for scrollers: a
## `ScrollContainer`, an `ItemList`, a `Tree`, a `TextEdit`, a `RichTextLabel` with vertical overflow. For each
## scroller it samples a grid of points inside its visible rectangle, emulates Godot's hit test to learn which
## control a finger would land on at each point, keeps one point per distinct kind of landing control (class,
## `mouse_filter`, script), and injects a REAL `InputEventScreenTouch` / `InputEventScreenDrag` sequence from that
## point (Godot's own bubbling, deadzone and press-cancel code are therefore the thing under test). The
## assertion is that the scroller's own offset moved. `Input.emulate_touch_from_mouse` is switched on for the
## run: Godot only drag-scrolls where `DisplayServer.is_touchscreen_available()` holds, which a desktop does not
## satisfy (the previous probes counted forwarded drag events instead; with the flag the offset itself is real).
##
## ## Honest limits
## A synthetic event sequence is not a finger: it has no multi-touch, no pressure, no palm, no Android
## ViewConfiguration slop and no inertia from real velocity. The device proof is `adb shell input swipe` and a
## finger; see `ANDROID_BUILD_SCOPE.md` 3.13.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

const SEED := 483920
## Drag length in the scroller's own canvas pixels: well past any deadzone (the phone's is 10 dp scaled).
const DRAG_PX := 110.0
## A scroller counts as scrolled when its offset moved by at least this many canvas pixels.
const MIN_MOVE := 20.0

var _fails := 0
var _checks := 0
var _shots := ""
var _survey := false
var _repeat := true
var _debug := false   ## Print the events each scroller sees per drag (diagnosis only).
var _only := ""
var _app: Node
var _tag := "p"
var _rows: Array = []   ## survey rows: {surface, scroller, hit, filter, moved, ok}


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if cond:
		print("  ok    " + what)
	else:
		print("  FAIL  " + what)
		if not _survey:
			_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _wait(sec: float) -> void:
	await get_tree().create_timer(sec).timeout


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


# -- Gesture injection -------------------------------------------------------

## A finger drag in ROOT-window pixels: touch down, 12 drag steps, touch up.
func _touch_drag(from: Vector2, to: Vector2) -> void:
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.position = from
	t.pressed = true
	Input.parse_input_event(t)
	await _settle(2)
	var prev := from
	for k in range(1, 13):
		var p := from.lerp(to, float(k) / 12.0)
		var d := InputEventScreenDrag.new()
		d.index = 0
		d.position = p
		d.relative = p - prev
		d.velocity = (p - prev) * 60.0
		Input.parse_input_event(d)
		prev = p
		await _settle(1)
	var u := InputEventScreenTouch.new()
	u.index = 0
	u.position = to
	u.pressed = false
	Input.parse_input_event(u)
	await _settle(3)


## A finger tap: down and up at the same pixel with no motion.
func _touch_tap(at: Vector2) -> void:
	for down in [true, false]:
		var t := InputEventScreenTouch.new()
		t.index = 0
		t.position = at
		t.pressed = down
		Input.parse_input_event(t)
		await _settle(2)
	await _settle(3)


# -- Scene scan ---------------------------------------------------------------

## Every visible `Window` that can hold UI: the root plus embedded subwindows, topmost last.
func _windows() -> Array:
	var out: Array = [get_tree().root]
	for w in get_tree().root.find_children("*", "Window", true, false):
		if (w as Window).visible:
			out.append(w)
	return out


## Window-local canvas point -> ROOT window pixels (what an injected touch must carry).
func _to_root(w: Window, local: Vector2) -> Vector2:
	if w == get_tree().root:
		return w.get_final_transform() * local
	return Vector2(w.position) + w.get_final_transform() * local


## The topmost control a finger at `local` (canvas coordinates of `w`) would land on, or null. Mirrors Godot's
## `gui_find_control`: reverse tree order, visible, `mouse_filter != IGNORE`, inside every clipping ancestor.
func _hit(w: Window, local: Vector2) -> Control:
	var all := w.find_children("*", "Control", true, false)
	for i in range(all.size() - 1, -1, -1):
		var c := all[i] as Control
		if c.get_viewport() != w or not c.is_visible_in_tree() or c.mouse_filter == Control.MOUSE_FILTER_IGNORE:
			continue
		if not c.get_global_rect().has_point(local):
			continue
		var clipped := false
		var a := c.get_parent()
		while a is Control:
			if (a as Control).clip_contents and not (a as Control).get_global_rect().has_point(local):
				clipped = true
				break
			a = a.get_parent()
		if not clipped:
			return c
	return null


## One scroller found by `_scan`.
class Scroller:
	var node: Control
	var win: Window
	func _init(n: Control, w: Window) -> void:
		node = n
		win = w


## The vertical scroll offset of any scroller kind (canvas pixels).
func _offset(c: Control) -> float:
	if c is ScrollContainer:
		return float((c as ScrollContainer).scroll_vertical)
	if c is ItemList:
		return (c as ItemList).get_v_scroll_bar().value
	if c is Tree:
		return (c as Tree).get_scroll().y
	if c is TextEdit:
		return (c as TextEdit).get_v_scroll_bar().value
	if c is RichTextLabel:
		return (c as RichTextLabel).get_v_scroll_bar().value
	return 0.0


func _set_offset(c: Control, v: float) -> void:
	if c is ScrollContainer:
		(c as ScrollContainer).scroll_vertical = int(v)
	elif c is ItemList:
		(c as ItemList).get_v_scroll_bar().value = v
	elif c is Tree:
		(c as Tree).scroll_to_item((c as Tree).get_root()) if v == 0.0 and (c as Tree).get_root() != null else null
	elif c is TextEdit:
		(c as TextEdit).get_v_scroll_bar().value = v
	elif c is RichTextLabel:
		(c as RichTextLabel).get_v_scroll_bar().value = v


## Whether `c` has vertical overflow worth dragging (room of at least 60 px).
func _overflow(c: Control) -> float:
	if c is ScrollContainer:
		var sc := c as ScrollContainer
		if sc.vertical_scroll_mode == ScrollContainer.SCROLL_MODE_DISABLED:
			return 0.0
		var bar := sc.get_v_scroll_bar()
		return bar.max_value - bar.page
	if c is ItemList:
		var b := (c as ItemList).get_v_scroll_bar()
		return b.max_value - b.page
	if c is Tree:
		var t := c as Tree
		var total := 0.0
		var it := t.get_root()
		if it == null:
			return 0.0
		# Tree exposes no max; compare item count x row height against the control height.
		var n := 0
		var stack := [it]
		while not stack.is_empty():
			var cur: TreeItem = stack.pop_back()
			if cur != t.get_root() or not t.hide_root:
				n += 1
			if not cur.collapsed:
				for ch in cur.get_children():
					stack.append(ch)
		var row := maxf(1.0, t.get_item_area_rect(it).size.y) if it != null else 20.0
		total = float(n) * row - t.size.y
		return total
	if c is TextEdit:
		var te := c as TextEdit
		var b2 := te.get_v_scroll_bar()
		return b2.max_value - b2.page
	if c is RichTextLabel:
		var r := c as RichTextLabel
		var b3 := r.get_v_scroll_bar()
		return b3.max_value - b3.page
	return 0.0


## Every vertically overflowing scroller in every visible window.
func _scan() -> Array:
	var out: Array = []
	for w in _windows():
		for c in (w as Window).find_children("*", "Control", true, false):
			var ctl := c as Control
			if ctl.get_viewport() != w or not ctl.is_visible_in_tree():
				continue
			if not (ctl is ScrollContainer or ctl is ItemList or ctl is Tree or ctl is TextEdit or ctl is RichTextLabel):
				continue
			if ctl.size.y < 40.0 or _overflow(ctl) < 60.0:
				continue
			out.append(Scroller.new(ctl, w))
	return out


func _sig(c: Control) -> String:
	if c == null:
		return "none"
	var scr := c.get_script() as Script
	return "%s|filter=%d|%s" % [c.get_class(), c.mouse_filter, scr.resource_path.get_file() if scr != null else "-"]


## Stroke length for a scroller whose visible part is `vis`: `DRAG_PX`, or less for a short scroller.
func _drag_len(vis: Rect2) -> float:
	return minf(DRAG_PX, vis.size.y * 0.55)


## Sample points (window-local) inside the scroller's visible part, one per distinct landing-control signature.
func _points_for(s: Scroller) -> Array:
	var vis := s.node.get_global_rect().intersection(s.win.get_visible_rect())
	var pts: Array = []
	if vis.size.x < 20.0 or vis.size.y < 40.0:
		return pts
	var seen := {}
	var cols := 6
	var rows := 12
	for ry in rows:
		for cx in cols:
			var p := Vector2(
				vis.position.x + vis.size.x * (float(cx) + 0.5) / float(cols),
				vis.position.y + 6.0 + (vis.size.y - 12.0) * (float(ry) + 0.5) / float(rows))
			# The drag goes UP by `_drag_len(s)`; keep the whole stroke inside the scroller's own visible
			# rectangle: a stroke that ends over a scrim or a neighbour is a different gesture (it can dismiss a
			# sheet on release), not the scroll under test.
			if p.y - _drag_len(vis) < vis.position.y + 3.0:
				continue
			var h := _hit(s.win, p)
			var key := _sig(h)
			if seen.has(key):
				continue
			seen[key] = true
			pts.append({"p": p, "hit": h, "sig": key})
	return pts


## What a gesture must not change: which windows are open, whether MORE and search are open and on which page.
## A drag that changes it ACTIVATED something (a row's tap action ran on release), which is a defect even when
## the list also scrolled.
func _state_sig() -> String:
	var names: Array = []
	for w in get_tree().root.find_children("*", "Window", true, false):
		if (w as Window).visible:
			names.append(String(w.name))
	names.sort()
	return "%s|menu=%s:%s|search=%s|tab=%s" % [names, _app._phone_menu.is_open(), _app._phone_menu.current_page_title(),
		_app._phone_search_overlay.visible, _app.get("_phone_tab")]


## Runs one drag from `pt` on scroller `s`; returns {moved, activated} (canvas pixels, bool).
func _try(s: Scroller, pt: Dictionary) -> Dictionary:
	_set_offset(s.node, 0.0)
	await _settle(2)
	var before := _offset(s.node)
	var sig_before := _state_sig()
	var from := _to_root(s.win, pt["p"] as Vector2)
	var scale := s.win.get_final_transform().get_scale().y
	var vis := s.node.get_global_rect().intersection(s.win.get_visible_rect())
	var to := from + Vector2(0.0, -_drag_len(vis) * scale)
	var seen := {}
	var tap := func(ev: InputEvent) -> void:
		seen[ev.get_class()] = int(seen.get(ev.get_class(), 0)) + 1
	s.node.gui_input.connect(tap)
	await _touch_drag(from, to)
	await _wait(0.25)
	if is_instance_valid(s.node):
		s.node.gui_input.disconnect(tap)
	var activated := _state_sig() != sig_before
	var moved := 0.0
	if is_instance_valid(s.node) and s.node.is_visible_in_tree():
		moved = _offset(s.node) - before
		_set_offset(s.node, 0.0)
	if _debug:
		var chain := ""
		var c: Node = pt["hit"]
		while c is Control and c != s.node:
			chain += "%s(f%d) < " % [c.get_class(), (c as Control).mouse_filter]
			c = c.get_parent()
		print("   DBG from=%s hit=%s chain=%s scroller_saw=%s moved=%.0f activated=%s" % [from, pt["sig"], chain, seen, moved, activated])
	await _settle(2)
	return {"moved": moved, "activated": activated}


## Scan the open surface and drag every distinct landing kind of every scroller, reopening the surface whenever a
## gesture closed or changed it.
func _check_surface(name_: String, opener: Callable) -> void:
	var count := _scan().size()
	var wins: Array = []
	for w in _windows():
		if w != get_tree().root:
			wins.append("%s%s" % [w.name, "" if (w as Window).title == "" else "'" + (w as Window).title + "'"])
	var visible_sc := 0
	var best := 0.0
	for w in _windows():
		for c in (w as Window).find_children("*", "ScrollContainer", true, false):
			if (c as Control).get_viewport() == w and (c as Control).is_visible_in_tree():
				visible_sc += 1
				best = maxf(best, _overflow(c))
	print("== %s: %d overflowing scroller(s); %d visible ScrollContainer(s), max overflow %.0f px; windows %s ==" % [name_, count, visible_sc, best, wins])
	if count == 0:
		_rows.append({"surface": name_, "scroller": "(none overflowing)", "hit": "-", "moved": 0.0, "ok": true})
		return
	for k in count:
		var scrollers := _scan()
		if k >= scrollers.size():
			break
		var s := scrollers[k] as Scroller
		var node_desc := "%s %s in %s" % [s.node.get_class(), s.node.name, "root" if s.win == get_tree().root else s.win.name]
		if _repeat:
			await _repeat_check("%s [%s]" % [name_, node_desc], s)
		var done := {}
		for _guard in 14:
			var pts: Array = []
			for cand in _points_for(s):
				if not done.has(cand["sig"]):
					pts.append(cand)
			if pts.is_empty():
				break
			var pt: Dictionary = pts[0]
			done[pt["sig"]] = true
			var res: Dictionary = await _try(s, pt)
			var moved: float = res["moved"]
			var activated: bool = res["activated"]
			var vis_len := _drag_len(s.node.get_global_rect().intersection(s.win.get_visible_rect())) if is_instance_valid(s.node) else DRAG_PX
			var ok := moved >= minf(MIN_MOVE, vis_len * 0.4) and not activated
			_rows.append({"surface": name_, "scroller": node_desc, "hit": pt["sig"], "moved": moved, "ok": ok, "activated": activated})
			_ok(ok, "%s: a drag from %s scrolls it (%.0f px) and activates nothing%s" % [
				node_desc, pt["sig"], moved, " (ACTIVATED SOMETHING)" if activated else ""])
			if activated or not is_instance_valid(s.node) or not s.node.is_visible_in_tree():
				_close_all()
				await _settle(4)
				opener.call()
				await _wait(0.6)
				var again := _scan()
				if k >= again.size():
					break
				s = again[k] as Scroller


# -- Repeated scrolling (owner 2026-10-26: "Settings only scrolls down once before all scroll input is impossible";
# "scrolling stopped working the more menus were expanded") ---------------------------------------------------

## Whether `s` can still move in direction `dir` (+1 = scroll down, content moves up): not within 6 px of a limit.
func _room(s: Scroller, dir: int) -> bool:
	var off := _offset(s.node)
	var mx := _overflow(s.node)
	return off < mx - 6.0 if dir > 0 else off > 6.0


## A start point near `p` whose landing control is not one that gives a vertical drag a different meaning
## (a slider, a scroll bar, a spin box, a text field arbitrates its own drag).
func _safe_start(s: Scroller, p: Vector2) -> Vector2:
	for dx in [0.0, 60.0, -60.0, 120.0, -120.0, 30.0, -30.0]:
		var q := p + Vector2(dx, 0.0)
		var h := _hit(s.win, q)
		if h == null or not (h is Slider or h is ScrollBar or h is SpinBox or h is LineEdit or h is TextEdit or h.get_class() == "SpinBoxLineEdit"):
			return q
	return p


## One stroke on `s` starting at window-local `p`, WITHOUT resetting the offset first. Returns the offset change.
func _stroke(s: Scroller, p: Vector2, dir: int) -> float:
	var before := _offset(s.node)
	var from := _to_root(s.win, _safe_start(s, p))
	var scale := s.win.get_final_transform().get_scale().y
	var vis := s.node.get_global_rect().intersection(s.win.get_visible_rect())
	var len_ := _drag_len(vis) * scale
	await _touch_drag(from, from + Vector2(0.0, -len_ * float(dir)))
	await _wait(0.45)   # let the fling's inertia end so the next stroke starts from rest
	return _offset(s.node) - before


## Strokes down x3 then up x3 (twice round), cycling start points; every stroke that has room must move the list in
## its direction. A scroller that works once and then ignores input fails here.
func _repeat_check(label: String, s: Scroller) -> void:
	var vis := s.node.get_global_rect().intersection(s.win.get_visible_rect())
	var starts: Array = []
	for fy in [0.8, 0.65, 0.5]:
		for fx in [0.15, 0.5, 0.85]:
			starts.append(Vector2(vis.position.x + vis.size.x * fx, vis.position.y + vis.size.y * fy))
	_set_offset(s.node, 0.0)
	await _settle(3)
	var bad := 0
	var log := ""
	for k in 12:
		var dir := 1 if (k % 6) < 3 else -1
		if not _room(s, dir):
			dir = -dir
			if not _room(s, dir):
				continue
		var moved: float = await _stroke(s, starts[k % starts.size()], dir)
		var good := moved * float(dir) >= 15.0
		log += "%+.0f " % moved
		if not good:
			bad += 1
	_ok(bad <= 1, "%s: 12 repeated strokes (at most 1 ignored: a stroke that lands on a slider is the slider's) in both directions each move the list (%d ignored; offsets %s)" % [label, bad, log])


## The body a section header toggles, or null: the header (or the `HBoxContainer` that holds it on the phone) is
## followed by a `VBoxContainer` (`category()`, `stage_category()`) or a `MarginContainer` around one (`group()`).
func _section_body(btn: Button) -> Control:
	var head: Control = btn
	if btn.get_parent() is HBoxContainer and btn.get_index() == 0:
		head = btn.get_parent() as Control
	var par := head.get_parent()
	if par == null or head.get_index() + 1 >= par.get_child_count():
		return null
	var nxt := par.get_child(head.get_index() + 1)
	if nxt is MarginContainer and nxt.get_child_count() > 0 and nxt.get_child(0) is VBoxContainer:
		return nxt.get_child(0) as Control
	if nxt is VBoxContainer and btn.focus_mode == Control.FOCUS_NONE:
		return nxt as Control
	return null


## Collapsible-section headers under `root`, found by shape (`_section_body`), not by name.
func _section_headers(root: Node) -> Array:
	var out: Array = []
	for b in root.find_children("*", "Button", true, false):
		var btn := b as Button
		if btn.is_visible_in_tree() and not btn.disabled and not btn.get_signal_connection_list("pressed").is_empty() 				and _section_body(btn) != null:
			out.append(btn)
	return out


## The stage index the `GENERATE` sheet's group header toggles (`world_workspace.gd::_pg_group` binds it to
## `_pg_toggle_group`), or -1 when `btn` is not one.
func _stage_index(btn: Button) -> int:
	for c in btn.get_signal_connection_list("pressed"):
		var cb: Callable = c["callable"]
		if cb.get_method() == &"_pg_toggle_group" and not cb.get_bound_arguments().is_empty():
			return int(cb.get_bound_arguments()[0])
	return -1


## Strokes the first scroller 4 times (down, down, up, up) from different start points; returns the log and the
## number of strokes that did not move it (strokes without room are skipped).
func _four_strokes() -> Dictionary:
	var scrollers := _scan()
	if scrollers.is_empty():
		return {"bad": 0, "log": "(nothing overflows)", "overflow": 0.0}
	var s := scrollers[0] as Scroller
	var vis := s.node.get_global_rect().intersection(s.win.get_visible_rect())
	var bad := 0
	var log := ""
	_set_offset(s.node, 0.0)
	await _settle(2)
	for k in 4:
		var dir := 1 if k < 2 else -1
		if not _room(s, dir):
			continue
		var moved: float = await _stroke(s, Vector2(vis.position.x + vis.size.x * (0.2 + 0.2 * k), vis.position.y + vis.size.y * 0.7), dir)
		log += "%+.0f " % moved
		if moved * float(dir) < 15.0:
			bad += 1
	return {"bad": bad, "log": log, "overflow": _overflow(s.node)}


## Opens every collapsible section of the open surface one at a time (the GENERATE sheet's stage groups, and the
## `DccWidgets` groups and categories elsewhere); after each opening it strokes the scroller 4 times and requires
## each stroke to move it. Scrolling that degrades as sections open, or dies after the first rebuild, fails here.
func _expand_check(name_: String) -> void:
	var opened := 0
	var done_stage := {}
	for _round in 16:
		var target: Button = null
		var scrollers := _scan()
		var root_node: Node = (scrollers[0] as Scroller).node if not scrollers.is_empty() else _app
		for b in (root_node as Node).find_children("*", "Button", true, false):
			var btn := b as Button
			if not btn.is_visible_in_tree():
				continue
			var si := _stage_index(btn)
			if si >= 0 and not done_stage.has(si):
				done_stage[si] = true
				target = btn
				break
		if target == null:
			for h in _section_headers(root_node):
				var bd := _section_body(h as Button)
				if bd != null and not bd.visible and not done_stage.has(h.get_instance_id()):
					done_stage[h.get_instance_id()] = true
					target = h
					break
		if target == null:
			break
		var label := target.text.substr(0, 24)
		target.pressed.emit()
		opened += 1
		await _settle(6)
		var r: Dictionary = await _four_strokes()
		_ok(int(r["bad"]) == 0, "%s: after opening section %d ('%s') 4 strokes still move the list (%d ignored; %s; overflow %.0f px)" % [
			name_, opened, label, int(r["bad"]), r["log"], float(r["overflow"])])


# -- Assistant settings: can the owner save the setup? (2026-10-26 "I can't seem to save the LLM on my phone") -------

## With the master switch OFF (the shipped default) the provider, address, model and key controls must be editable,
## and text typed into the address and model fields and then closed over (no Enter, no focus change: what a phone's
## Back key does) must be stored. On the pristine tree every one of those controls was read-only while the master
## switch was off, and a typed address was lost on close.
func _assistant_leg() -> void:
	print("== Assistant settings persistence ==")
	_close_all()
	await _settle(4)
	DccSettings.set_llm_value("enabled", false)
	DccSettings.set_llm_value("base_url", "")
	DccSettings.set_llm_value("model", "")
	DccSettings.set_llm_value("preset_id", "")
	_app.open_assistant_settings()
	await _wait(0.6)
	var win = _app.assistant_settings_window
	var url: LineEdit = win.controls.get("base_url")
	var model: LineEdit = win.controls.get("model")
	var pick: OptionButton = win.controls.get("preset")
	_ok(url != null and url.editable, "the address field is editable while the master switch is off")
	_ok(model != null and model.editable, "the model-name field is editable while the master switch is off")
	_ok(pick != null and not pick.disabled, "the provider picker is enabled while the master switch is off")
	_ok(win.controls.has("key_source") and not (win.controls["key_source"] as OptionButton).disabled,
		"the key-source picker is enabled while the master switch is off")
	url.text = "http://192.168.1.50:1234/v1"
	model.text = "dummy-model"
	win.hide()
	await _settle(4)
	_ok(String(DccSettings.llm_value("base_url")) == "http://192.168.1.50:1234/v1",
		"an address typed and then closed over (no Enter, no focus change) is stored ('%s')" % DccSettings.llm_value("base_url"))
	_ok(String(DccSettings.llm_value("model")) == "dummy-model",
		"a model name typed and then closed over is stored ('%s')" % DccSettings.llm_value("model"))
	_ok(not bool(DccSettings.llm_value("enabled")), "setting the provider up did not switch the assistant on")
	DccSettings.set_llm_value("base_url", "")
	DccSettings.set_llm_value("model", "")


# -- Surfaces -------------------------------------------------------------------

func _close_all() -> void:
	if _app._phone_menu != null and _app._phone_menu.is_open():
		_app._phone_menu.close()
	_app._set_search_open(false)
	for w in get_tree().root.find_children("*", "Window", true, false):
		if (w as Window).visible and w != get_tree().root:
			(w as Window).hide()
	var pp = _app.get("layers_popover")
	if pp != null and pp.visible:
		pp.hide()


func _surfaces() -> Array:
	var a = _app
	var out: Array = []
	out.append(["phone tab MAP half", func(): a._pick_phone_tab("map"); a.set_phone_detent("half")])
	out.append(["phone tab MAP full", func(): a._pick_phone_tab("map"); a.set_phone_detent("full")])
	out.append(["phone tab GENERATE full", func(): a._pick_phone_tab("gen"); a.set_phone_detent("full")])
	for dom in ["world", "civilization", "cartography"]:
		var d: String = dom
		out.append(["left sheet domain " + d + " full", func(): a._pick_bar_domain(d); a.set_phone_detent("full")])
	out.append(["right sheet", func(): a.phone_show_right_sheet()])
	out.append(["search results", func(): a.open_find_on_map(); a._phone_search_field.text = "a"; a._refresh_phone_search()])
	out.append(["MORE root", func(): a._phone_menu.open()])
	for id in a._phone_menu.SCREEN_TITLES.keys():
		var sid: String = String(id)
		out.append(["MORE screen " + sid, func(): a._phone_menu.open(); a._phone_menu._push_screen(sid)])
	out.append(["Assistant settings", func(): a.open_assistant_settings()])
	out.append(["Assistant panel", func(): a.open_assistant_panel()])
	out.append(["Customise / quick bar window", func(): a.open_quick_bar_window()])
	out.append(["Vault overview", func(): a.open_vault_overview()])
	out.append(["Vault browse", func(): a.open_vault_browse()])
	out.append(["Faction roster", func(): a.open_faction_roster()])
	out.append(["Settlement types", func(): a.open_settlement_types()])
	out.append(["New world dialog", func(): a.open_new_world()])
	out.append(["Open project picker", func(): a.open_project_picker()])
	out.append(["Layers popover", func(): a.layers_popover.open()])
	out.append(["Data manager", func(): a.open_data_manager()])
	out.append(["Asset library", func(): a.open_asset_library()])
	out.append(["Travel library", func(): a.open_travel_library()])
	out.append(["Journey planner", func(): a.open_journey_planner()])
	out.append(["Export dialog", func(): a.open_export_dialog()])
	out.append(["Undo history", func(): a.open_undo_history()])
	out.append(["World data", func(): a.open_world_data()])
	out.append(["Generation info", func(): a.open_gen_info()])
	out.append(["Shortcuts", func(): a.open_shortcuts()])
	out.append(["Saved journeys", func(): a.open_saved_journeys()])
	out.append(["Conflicts list", func(): a.open_conflicts_list()])
	out.append(["Credits", func(): a.open_credits()])
	out.append(["About", func(): a.open_about()])
	out.append(["Storage locations", func(): a.open_storage_locations()])
	return out


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := args[i]
		if a == "--force-touch":
			pass
		elif a == "--survey":
			_survey = true
		elif a == "--debug":
			_debug = true
		elif a == "--vp" and i + 1 < args.size():
			## Headless windows have a fixed 100x100 size and ignore `--resolution`; this sets the root window's
			## size before the shell boots so the phone form (decided once, at boot, from the aspect) is reachable.
			i += 1
			var wh := args[i].split("x")
			get_window().size = Vector2i(int(wh[0]), int(wh[1]))
		elif a == "--only" and i + 1 < args.size():
			i += 1
			_only = args[i]
		elif a == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		i += 1
	get_tree().create_timer(1700.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	Input.emulate_touch_from_mouse = true
	_ok(not bool(DccSettings.llm_value("enabled")), "the assistant is off at launch (the user dir was wiped)")
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await _wait(1.2)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	if _app.phone_project_picker != null and _app.phone_project_picker.visible:
		_app.phone_project_picker.hide()
	await _settle(6)
	if not _app.is_phone():
		print("ABORT: this probe is for the phone form (--resolution 393x852 -- --force-touch)")
		get_tree().quit(2)
		return
	var vs := get_viewport().get_visible_rect().size
	_tag = "land" if _app._landscape else "%dx%d" % [int(vs.x), int(vs.y)]
	print("== phone %s, viewport %s, phone_scale %.4f ==" % ["landscape" if _app._landscape else "portrait", vs, _app.phone_scale()])
	var bridge = _app.bridge
	bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await _wait(0.25)
	await _wait(1.0)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	if _app.phone_project_picker != null and _app.phone_project_picker.visible:
		_app.phone_project_picker.hide()
	await _settle(8)
	for entry in _surfaces():
		var nm: String = entry[0]
		if _only != "" and nm.find(_only) < 0:
			continue
		_close_all()
		await _settle(4)
		var opener: Callable = entry[1]
		opener.call()
		await _wait(0.6)
		await _check_surface(nm, opener)
		if nm.find("GENERATE") >= 0 or nm.find("left sheet") >= 0 or nm.find("right sheet") >= 0 or nm.find("Assistant settings") >= 0 or nm.find("MORE screen prefs") >= 0:
			await _expand_check(nm)
	if _only == "" or _only.find("Assistant") >= 0:
		await _assistant_leg()
	_close_all()
	print("---- SURVEY (failing rows) ----")
	for r in _rows:
		if not bool(r["ok"]):
			print("BROKEN | %s | %s | %s | moved %.0f" % [r["surface"], r["scroller"], r["hit"], r["moved"]])
	_finish()
