extends Node
## VERIFIER probe for the 2026-09-07 "dropdown opens on touch-DOWN" batch.
## Written independently of `_gestclass_probe.gd`: it re-enumerates the tree by
## its own method (INCLUDING internal children, which the lane's walk omits),
## classifies "acts on touch-DOWN" by MEASUREMENT rather than by a class table,
## and reproduces the main loop's on-glass observation with a per-node runtime
## neuter so no file has to be edited to see the defect.
##
##   godot --path . _vfy_gesture_probe.tscn -- --force-touch --vp 1080x2340 --tag V
##
## Flags actually read (grepped from the body below):
##   --vp WxH     SubViewport size. Default 1080x2340.
##   --tag NAME   line prefix. Default vfy.
##   --part A|B|C|D|E|ALL   which sections to run. Default ALL.
##   --force-touch   NOT read here; `dcc_shell.gd` reads it.

const _FILTER := ["STOP", "PASS", "IGNORE"]

var app: Node
var _vp: SubViewport
var _tag := "vfy"
var _fail := 0
var _part := "ALL"

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _want(p: String) -> bool:
	return _part == "ALL" or _part.contains(p)

# -- input ---------------------------------------------------------------------

func _press_at(at: Vector2, down: bool) -> void:
	var e := InputEventMouseButton.new()
	e.button_index = MOUSE_BUTTON_LEFT
	e.pressed = down
	e.position = at
	e.global_position = at
	_vp.push_input(e, true)
	await _frames(1)

func _tap_at(at: Vector2) -> void:
	await _press_at(at, true)
	await _press_at(at, false)
	await _frames(6)

func _swipe(from: Vector2, offsets: Array) -> void:
	await _press_at(from, true)
	var prev := from
	for o in offsets:
		var at: Vector2 = from + (o as Vector2)
		var mm := InputEventMouseMotion.new()
		mm.position = at
		mm.global_position = at
		mm.relative = at - prev
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT
		_vp.push_input(mm, true)
		prev = at
		await _frames(1)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = prev
	up.global_position = prev
	_vp.push_input(up, true)
	await _frames(6)

## A thumb's pivot: sideways travel exceeding downward travel for the first few
## samples, then straight. Restated here rather than imported.
func _jitter(dir: float, reach: float) -> Array:
	var out: Array = []
	var xs := [3.0, 6.0, 5.0, 8.0, 10.0, 6.0, 4.0, 2.0, 0.0, -2.0, -4.0, -4.0, -3.0]
	var ys := [1.0, 2.0, 4.0, 6.0, 10.0, 20.0, 40.0, 80.0, 140.0, 210.0, 290.0, 380.0, 468.0]
	var span: float = ys[ys.size() - 1]
	for i in xs.size():
		out.append(Vector2(float(xs[i]), dir * float(ys[i]) / span * reach))
	return out

# -- tree walking, MY method ---------------------------------------------------

## `get_children(true)` -- internal children INCLUDED. `SpinBox`'s `LineEdit`
## and `TabContainer`'s `TabBar` are added with `INTERNAL_MODE_*`, so a walk
## that leaves the default `false` cannot see them at all.
func _walk(root: Node, out: Array, internal_only: Dictionary) -> void:
	var plain := root.get_children()
	for c in root.get_children(true):
		out.append(c)
		if not (c in plain):
			internal_only[c] = true
		_walk(c, out, internal_only)

func _nearest_any_scroller(n: Node) -> ScrollContainer:
	var p: Node = n.get_parent()
	while p != null:
		if p is ScrollContainer:
			return p as ScrollContainer
		p = p.get_parent()
	return null

func _nearest_v_scroller(n: Node) -> ScrollContainer:
	var p: Node = n.get_parent()
	while p != null:
		if p is ScrollContainer and (p as ScrollContainer).vertical_scroll_mode \
				!= ScrollContainer.SCROLL_MODE_DISABLED:
			return p as ScrollContainer
		p = p.get_parent()
	return null

func _surface(n: Node) -> String:
	var p: Node = n
	while p != null:
		if p is Window and p != get_tree().root and p != _vp:
			var scr: Script = p.get_script()
			return "Win:" + (scr.resource_path.get_file() if scr != null else p.get_class())
		p = p.get_parent()
	return "root"

func _live(c: Control) -> bool:
	if c is BaseButton:
		return not (c as BaseButton).disabled
	if c is LineEdit:
		return (c as LineEdit).editable
	if c is TextEdit:
		return (c as TextEdit).editable
	return true

# -- PART A: inventory ---------------------------------------------------------

func _inventory(where: String) -> Dictionary:
	var internal_only: Dictionary = {}
	var all: Array = []
	_walk(get_tree().root, all, internal_only)
	var by_class: Dictionary = {}
	var internal_classes: Dictionary = {}
	var diverge: Array = []
	var down_press: Dictionary = {}     ## BaseButton with action_mode == PRESS
	var n_controls := 0
	for n in all:
		if not (n is Control):
			continue
		n_controls += 1
		var c := n as Control
		var cls := c.get_class()
		by_class[cls] = int(by_class.get(cls, 0)) + 1
		if internal_only.has(n):
			internal_classes[cls] = int(internal_classes.get(cls, 0)) + 1
		var any := _nearest_any_scroller(c)
		var vs := _nearest_v_scroller(c)
		## The census in `_gestclass_probe.gd` asks `nearest ScrollContainer`
		## and then whether THAT one scrolls vertically; `touch_release_button`
		## asks for the nearest one that scrolls vertically. They disagree
		## exactly when a v-disabled scroller is nested inside a scrolling one.
		if any != vs and any != null:
			diverge.append("%s %s under %s (v-disabled) but %s above"
				% [_surface(c), cls, any.name, "a v-scroller" if vs != null else "none"])
		if c is BaseButton and (c as BaseButton).action_mode \
				== BaseButton.ACTION_MODE_BUTTON_PRESS and _live(c) and vs != null:
			down_press[cls] = int(down_press.get(cls, 0)) + 1
	_log("-- inventory (%s): %d Controls, %d distinct classes" % [where, n_controls, by_class.size()])
	var keys: Array = by_class.keys()
	keys.sort()
	var line := ""
	for k in keys:
		line += " %s=%d" % [k, by_class[k]]
	_log("   classes:%s" % line)
	var ik: Array = internal_classes.keys()
	ik.sort()
	var iline := ""
	for k in ik:
		iline += " %s=%d" % [k, internal_classes[k]]
	_log("   reachable ONLY with get_children(true) (internal):%s"
		% (iline if iline != "" else " none"))
	_log("   BaseButton still ACTION_MODE_BUTTON_PRESS, live, under a v-scroller: %s"
		% (str(down_press) if down_press.size() > 0 else "none"))
	_log("   scroller-lookup divergences (nearest-any vs nearest-vertical): %d" % diverge.size())
	## Every text/item control that acts on a press, under a LIVE vertical
	## scroller -- counted with internal children INCLUDED, which is the whole
	## difference between this number and the lane's 4.
	var txt: Dictionary = {}
	var txt_internal: Dictionary = {}
	for n2 in all:
		if not (n2 is Control):
			continue
		var c2 := n2 as Control
		if not (c2 is LineEdit or c2 is TextEdit or c2 is TabBar or c2 is Tree 				or c2 is ItemList):
			continue
		if _nearest_v_scroller(c2) == null or not _live(c2):
			continue
		var k2 := "%s | %s | vis=%s" % [_surface(c2), c2.get_class(),
			str(c2.is_visible_in_tree())]
		txt[k2] = int(txt.get(k2, 0)) + 1
		if internal_only.has(n2):
			txt_internal[k2] = int(txt_internal.get(k2, 0)) + 1
	var tk: Array = txt.keys()
	tk.sort()
	var tsum := 0
	var tint := 0
	for k2 in tk:
		tsum += int(txt[k2])
		tint += int(txt_internal.get(k2, 0))
		_log("      %3d (%d internal)  %s" % [txt[k2], int(txt_internal.get(k2, 0)), k2])
	_log("   text/item controls acting on press, live, under a v-scroller: %d (%d INTERNAL)"
		% [tsum, tint])
	## Where the dropdowns are, by surface -- the shipped comment in
	## `dcc_shell.gd` claims "10 left sheet, 6 New World card, 2 in the slicer
	## AcceptDialog"; this counts them independently.
	var ob_by: Dictionary = {}
	for n3 in all:
		if not (n3 is OptionButton):
			continue
		var c3 := n3 as OptionButton
		var k3 := "%s | scroller=%s | am=%d | filter=%s" % [_surface(c3),
			("v" if _nearest_v_scroller(c3) != null else "none"),
			c3.action_mode, _FILTER[c3.mouse_filter]]
		ob_by[k3] = int(ob_by.get(k3, 0)) + 1
	var ok3: Array = ob_by.keys()
	ok3.sort()
	for k3 in ok3:
		_log("      OptionButton %3d  %s" % [ob_by[k3], k3])
	for d in diverge:
		_log("      %s" % d)
	return {"classes": by_class, "internal": internal_classes,
		"down_press": down_press, "diverge": diverge.size()}

# -- PART B: measure "acts on touch-DOWN" per class, no class table ------------

## Snapshot every stored property plus focus and popup visibility. The three
## mechanisms that matter -- a value written, focus taken, a popup opened --
## are all visible in one of the three.
func _snapshot(c: Control) -> String:
	var out := ""
	for p in c.get_property_list():
		if int(p["usage"]) & PROPERTY_USAGE_STORAGE:
			out += "%s=%s;" % [p["name"], str(c.get(String(p["name"])))]
	var vp := c.get_viewport()
	out += "focus=%s;" % (str(vp.gui_get_focus_owner() == c) if vp != null else "?")
	if c.has_method("get_popup"):
		var pop = c.call("get_popup")
		if pop != null:
			out += "popup=%s;" % str(pop.visible)
	return out

func _measure_class(cls: String) -> Dictionary:
	if not ClassDB.can_instantiate(cls):
		return {"cls": cls, "note": "not instantiable"}
	var rig := ScrollContainer.new()
	rig.custom_minimum_size = Vector2(400, 300)
	rig.size = Vector2(400, 300)
	rig.position = Vector2(20, 20)
	rig.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	_vp.add_child(rig)
	var box := VBoxContainer.new()
	rig.add_child(box)
	var inst: Object = ClassDB.instantiate(cls)
	if not (inst is Control):
		rig.queue_free()
		return {"cls": cls, "note": "not a Control"}
	var c := inst as Control
	c.custom_minimum_size = Vector2(360, 80)
	## Give the classes that need content something to act on, so "nothing
	## changed" cannot mean "there was nothing to change".
	if c is OptionButton:
		for i in 6:
			(c as OptionButton).add_item("item %d" % i)
	elif c is MenuButton:
		for i in 4:
			(c as MenuButton).get_popup().add_item("m %d" % i)
	elif c is TabBar:
		for i in 4:
			(c as TabBar).add_tab("t %d" % i)
	elif c is ItemList:
		for i in 8:
			(c as ItemList).add_item("row %d" % i)
	elif c is LineEdit:
		(c as LineEdit).text = "abcdefghijklmnop"
	elif c is TextEdit:
		(c as TextEdit).text = "abcdefghij\nklmnop\nqrstuv"
	elif c is Tree:
		var r := (c as Tree).create_item()
		for i in 4:
			(c as Tree).create_item(r).set_text(0, "n %d" % i)
	box.add_child(c)
	var spacer := Control.new()
	spacer.custom_minimum_size = Vector2(360, 2000)
	box.add_child(spacer)
	await _frames(6)
	var at: Vector2 = c.get_global_rect().get_center()
	var before := _snapshot(c)
	## Press ONLY -- no motion, no release. Anything that changes here acted
	## on touch-DOWN, whatever its class.
	await _press_at(at, true)
	await _frames(3)
	var after_down := _snapshot(c)
	await _press_at(at, false)
	await _frames(4)
	var acts_down := after_down != before
	## And: does a jittered vertical swipe that starts on it still scroll?
	rig.scroll_vertical = 0
	await _frames(4)
	## Close anything the press above left standing, so the swipe is not
	## measured through an open popup.
	if c.has_method("get_popup"):
		var pop = c.call("get_popup")
		if pop != null and pop.visible:
			pop.hide()
			await _frames(4)
	var s0 := rig.scroll_vertical
	var v0 := _snapshot(c)
	await _swipe(c.get_global_rect().get_center(), _jitter(-1.0, 200.0))
	var s1 := rig.scroll_vertical
	var v1 := _snapshot(c)
	if c.has_method("get_popup"):
		var pop2 = c.call("get_popup")
		if pop2 != null and pop2.visible:
			pop2.hide()
			await _frames(4)
	var am := -1
	if c is BaseButton:
		am = (c as BaseButton).action_mode
	var res := {"cls": cls, "acts_down": acts_down, "am": am,
		"filter": _FILTER[c.mouse_filter], "scrolled": s1 != s0,
		"state_moved": v1 != v0, "s0": s0, "s1": s1}
	rig.queue_free()
	await _frames(2)
	return res

# -- helpers for the real tree --------------------------------------------------

func _screen_pt(ctl: Control, local: Vector2) -> Vector2:
	var win := ctl.get_window()
	var p: Vector2 = ctl.get_global_transform() * local
	if win != null and win != _vp and win.is_embedded():
		p = win.get_final_transform() * p + Vector2(win.position)
	return p

func _under(ctl: Control, at: Vector2) -> String:
	var hover := InputEventMouseMotion.new()
	hover.position = at
	hover.global_position = at
	_vp.push_input(hover, true)
	await _frames(2)
	var vp: Viewport = ctl.get_viewport()
	var over := vp.gui_get_hovered_control() if vp != null else null
	return "<none>" if over == null else "%s(%s)" % [over.get_class(), over.name]

# -- Boot ----------------------------------------------------------------------

func _ready() -> void:
	_tag = _arg("--tag", "vfy")
	_part = _arg("--part", "ALL")
	var parts: PackedStringArray = _arg("--vp", "1080x2340").split("x")
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if app.phone_project_picker != null and app.phone_project_picker.visible:
		app.phone_project_picker.hide()
	await _frames(6)
	_log("viewport %dx%d phone=%s tablet=%s scale=%.3f"
		% [_vp.size.x, _vp.size.y, app.is_phone(), DccTheme.is_tablet(), app.phone_scale()])
	_check(app.is_phone(), "the shell booted phone")

	if _want("A"):
		_inventory("boot, world-less")

	if _want("B"):
		_log("-- measured, not tabulated: press-only on one instance of each class")
		var seen: Array = []
		var inv: Array = []
		var internal_only: Dictionary = {}
		_walk(get_tree().root, inv, internal_only)
		for n in inv:
			if n is Control and not (n.get_class() in seen):
				seen.append(n.get_class())
		seen.append_array(["TabBar", "Tree", "ItemList", "CheckButton", "TextureButton",
			"LinkButton", "SpinBox", "TextEdit", "RichTextLabel"])
		seen.sort()
		var uniq: Array = []
		for s in seen:
			if not (s in uniq):
				uniq.append(s)
		for cls in uniq:
			var r := await _measure_class(String(cls))
			if r.has("note"):
				_log("   %-22s %s" % [cls, r["note"]])
			else:
				_log("   %-22s acts_on_DOWN=%-5s am=%-2d filter=%-6s swipe_scrolls=%-5s swipe_moved_state=%s (%d->%d)"
					% [cls, str(r["acts_down"]), r["am"], r["filter"],
						str(r["scrolled"]), str(r["state_moved"]), r["s0"], r["s1"]])

	# -- PART C: the on-glass observation, reproduced ---------------------------
	if _want("C"):
		_log("-- PART C: the main loop's observation, with a per-node runtime neuter")
		app.open_new_world()
		await _frames(20)
		var dlg = app.new_world_dialog
		var ob: OptionButton = dlg.archetype_input
		_check(ob != null, "the New World card has an `archetype_input` OptionButton")
		if ob != null:
			var sc := _nearest_v_scroller(ob)
			_check(sc != null, "the Archetype dropdown sits inside a vertical scroller")
			_log("   shipped state: action_mode=%d filter=%s items=%d"
				% [ob.action_mode, _FILTER[ob.mouse_filter], ob.item_count])
			_check(ob.action_mode == BaseButton.ACTION_MODE_BUTTON_RELEASE,
				"shipped: the Archetype dropdown acts on RELEASE")
			_check(ob.mouse_filter == Control.MOUSE_FILTER_PASS,
				"shipped: the Archetype dropdown is MOUSE_FILTER_PASS")
			for neutered in [true, false]:
				if neutered:
					## Exactly the node state HEAD produced: `phone_fit`'s old
					## clause excluded OptionButton from the PASS conversion and
					## nothing touched `action_mode`.
					ob.action_mode = BaseButton.ACTION_MODE_BUTTON_PRESS
					ob.mouse_filter = Control.MOUSE_FILTER_STOP
				else:
					ob.action_mode = BaseButton.ACTION_MODE_BUTTON_RELEASE
					ob.mouse_filter = Control.MOUSE_FILTER_PASS
				var tag := "NEUTERED (as at HEAD)" if neutered else "SHIPPED"
				sc.scroll_vertical = 0
				ob.select(0)
				ob.get_popup().hide()
				await _frames(8)
				sc.ensure_control_visible(ob)
				await _frames(6)
				var s0 := sc.scroll_vertical
				var sel0 := ob.selected
				var at := _screen_pt(ob, ob.size * 0.5)
				var u := await _under(ob, at)
				_check(u.begins_with("OptionButton"),
					"%s: the push point picks the dropdown (under=%s)" % [tag, u])
				await _swipe(at, _jitter(-1.0, 400.0))
				var s1 := sc.scroll_vertical
				var sel1 := ob.selected
				var pop := ob.get_popup().visible
				_log("   %s: swipe ON the dropdown -> sel %d->%d  popup=%s  scroll %d->%d"
					% [tag, sel0, sel1, str(pop), s0, s1])
				if neutered:
					_check(sel1 != sel0 or pop or s1 == s0,
						"NEUTERED: the defect is present (state moved or the card did not scroll)")
				else:
					_check(sel1 == sel0 and not pop,
						"SHIPPED: the dropdown's selection and popup are untouched")
					_check(s1 != s0, "SHIPPED: and the card scrolls (%d -> %d)" % [s0, s1])
				ob.get_popup().hide()
				await _frames(4)
				## The control the main loop quoted: the same swipe from the
				## label column, which scrolled normally even at HEAD.
				sc.scroll_vertical = 0
				await _frames(6)
				var lbl_x: float = _screen_pt(ob, Vector2(0.0, ob.size.y * 0.5)).x - 120.0
				var lbl_pt := Vector2(lbl_x, _screen_pt(ob, ob.size * 0.5).y)
				var ls0 := sc.scroll_vertical
				await _swipe(lbl_pt, _jitter(-1.0, 400.0))
				_log("   %s: the same swipe from the label column at x=%.0f -> scroll %d->%d"
					% [tag, lbl_x, ls0, sc.scroll_vertical])
				_check(sc.scroll_vertical != ls0,
					"%s: the label column always scrolled (%d -> %d)"
						% [tag, ls0, sc.scroll_vertical])
			# -- PART E: the normal interaction, all the way through -----------
			if _want("E"):
				_log("-- PART E: tap-open, then tap an item")
				ob.select(0)
				await _frames(4)
				var pt := _screen_pt(ob, ob.size * 0.5)
				await _tap_at(pt)
				var pm := ob.get_popup()
				_check(pm.visible, "a plain tap opens the popup")
				if pm.visible:
					## `PopupMenu` has no `get_item_rect` on 4.7.1 (checked with
					## `ClassDB.class_has_method`), so item 2's position is FOUND
					## rather than computed: hover down the popup's screen rect until
					## the engine itself reports item 2 as the focused row.
					var ip := Vector2(-1, -1)
					var items: Control = null
					var pmall: Array = []
					var pmint: Dictionary = {}
					_walk(pm, pmall, pmint)
					for ch in pmall:
						if ch is Control and ch.get_class() == "PopupMenuItems":
							items = ch as Control
							break
					_log("   popup pos=%s size=%s items_rect=%s count=%d"
						% [str(pm.position), str(pm.size),
							"<none>" if items == null else str(items.get_global_rect()),
							pm.item_count])
					if items != null:
						var ir := items.get_global_rect()
						var row: float = ir.size.y / float(maxi(1, pm.item_count))
						var loc := Vector2(ir.position.x + ir.size.x * 0.5,
							ir.position.y + row * 2.5)
						ip = Vector2(pm.position) + pm.get_final_transform() * loc
					_check(ip.x >= 0.0, "item 2 of the open popup is reachable by a pointer")
					if ip.x >= 0.0:
						await _tap_at(ip)
						await _frames(6)
						_log("   tapped item 2 at %.0f,%.0f -> selected=%d popup=%s"
							% [ip.x, ip.y, ob.selected, str(pm.visible)])
						_check(ob.selected == 2,
							"and a tap on item 2 selects it (selected=%d)" % ob.selected)
				pm.hide()
				await _frames(4)
		if dlg != null:
			dlg.hide()
			await _frames(6)

	# -- PART F: each of the two writes neutered ON ITS OWN ---------------------
	#
	# The lane's mutation table claims `action_mode` and `mouse_filter` are both
	# load-bearing and neither is sufficient. Driven here per-property on the real
	# Archetype dropdown instead of by editing the file, so each row is a
	# measurement of that one write.
	if _want("F"):
		_log("-- PART F: one write at a time, on the real Archetype dropdown")
		app.open_new_world()
		await _frames(20)
		var d2 = app.new_world_dialog
		var ob2: OptionButton = d2.archetype_input
		var sc2 := _nearest_v_scroller(ob2)
		for row in [["action_mode=PRESS, filter kept PASS", 0, 1],
				["action_mode kept RELEASE, filter=STOP", 1, 0],
				["shipped: RELEASE + PASS", 1, 1]]:
			ob2.action_mode = int(row[1])
			ob2.mouse_filter = int(row[2]) as Control.MouseFilter
			sc2.scroll_vertical = 0
			ob2.select(0)
			ob2.get_popup().hide()
			await _frames(8)
			sc2.ensure_control_visible(ob2)
			await _frames(6)
			var s0 := sc2.scroll_vertical
			var sel0 := ob2.selected
			await _swipe(_screen_pt(ob2, ob2.size * 0.5), _jitter(-1.0, 400.0))
			_log("   %-38s sel %d->%d popup=%s scroll %d->%d"
				% [row[0], sel0, ob2.selected, str(ob2.get_popup().visible),
					s0, sc2.scroll_vertical])
			ob2.get_popup().hide()
			await _frames(4)
		ob2.action_mode = BaseButton.ACTION_MODE_BUTTON_RELEASE
		ob2.mouse_filter = Control.MOUSE_FILTER_PASS
		if d2 != null:
			d2.hide()
			await _frames(6)

	# -- PART G: the SpinBox rows of the same card ------------------------------
	#
	# `SpinBox` is a `Range`, so the lane's census skips it, and its `LineEdit`
	# is an INTERNAL child, so the lane's walk cannot see that either. This asks
	# the on-glass question directly: does a vertical swipe that starts on a
	# SpinBox row of the New World card scroll the card?
	if _want("G"):
		_log("-- PART G: a vertical swipe starting on the card's Seed SpinBox")
		app.open_new_world()
		await _frames(20)
		var d3 = app.new_world_dialog
		var sb: SpinBox = d3.seed_input
		_check(sb != null, "the New World card has a `seed_input` SpinBox")
		if sb != null:
			var sc3 := _nearest_v_scroller(sb)
			_check(sc3 != null, "it sits inside a vertical scroller")
			var le: LineEdit = sb.get_line_edit()
			_log("   SpinBox %.0fx%.0f  its LineEdit %.0fx%.0f filter=%s editable=%s internal=%s"
				% [sb.size.x, sb.size.y, le.size.x, le.size.y, _FILTER[le.mouse_filter],
					str(le.editable), str(not (le in sb.get_children()))])
			sc3.scroll_vertical = 0
			await _frames(6)
			sc3.ensure_control_visible(sb)
			await _frames(6)
			var at3 := _screen_pt(sb, sb.size * 0.5)
			var u3 := await _under(sb, at3)
			var s0 := sc3.scroll_vertical
			var v0 := sb.value
			await _swipe(at3, _jitter(-1.0, 400.0))
			_log("   swipe ON the SpinBox (under=%s): value %s->%s  focus=%s  scroll %d->%d"
				% [u3, str(v0), str(sb.value), str(le.has_focus()), s0, sc3.scroll_vertical])
			## The positive control: the same swipe one row over, on the label.
			sc3.scroll_vertical = 0
			await _frames(6)
			var lx: float = _screen_pt(sb, Vector2(0.0, sb.size.y * 0.5)).x - 120.0
			var ls0 := sc3.scroll_vertical
			await _swipe(Vector2(lx, _screen_pt(sb, sb.size * 0.5).y), _jitter(-1.0, 400.0))
			_log("   the same swipe from the label column at x=%.0f -> scroll %d->%d"
				% [lx, ls0, sc3.scroll_vertical])
			_check(sc3.scroll_vertical != ls0,
				"the card DOES scroll from the label column (%d -> %d)"
					% [ls0, sc3.scroll_vertical])
		if d3 != null:
			d3.hide()
			await _frames(6)

	# -- PART D: the two gates, asserted on the live tree -----------------------
	if _want("D"):
		_log("-- PART D: the gates")
		## Gate 1: no vertical scroller above -> left stock. The seven menu-bar
		## MenuButtons are the population.
		var inv2: Array = []
		var io2: Dictionary = {}
		_walk(get_tree().root, inv2, io2)
		var mb_total := 0
		var mb_stock := 0
		for n in inv2:
			if n is MenuButton:
				mb_total += 1
				if (n as MenuButton).action_mode == BaseButton.ACTION_MODE_BUTTON_PRESS \
						and _nearest_v_scroller(n) == null:
					mb_stock += 1
		_log("   MenuButton in tree: %d, of which stock(am=PRESS) with no v-scroller: %d"
			% [mb_total, mb_stock])
		_check(mb_total > 0 and mb_total == mb_stock,
			"gate 1: every MenuButton has no v-scroller above it and was left stock")
		## Gate 1, the other direction: put a fresh OptionButton somewhere with
		## NO scroller and confirm the helper declines it.
		var loose := OptionButton.new()
		loose.add_item("a")
		_vp.add_child(loose)
		await _frames(2)
		var changed_loose: bool = DccWidgets.touch_release_button(loose)
		_check(not changed_loose and loose.action_mode == BaseButton.ACTION_MODE_BUTTON_PRESS,
			"gate 1: a dropdown with no scroller above is declined and stays PRESS")
		loose.queue_free()
		## Gate 2: already RELEASE -> declined, and nothing else is written.
		var rig := ScrollContainer.new()
		rig.custom_minimum_size = Vector2(300, 200)
		_vp.add_child(rig)
		var cb := CheckBox.new()
		cb.mouse_filter = Control.MOUSE_FILTER_STOP
		rig.add_child(cb)
		await _frames(2)
		var changed_cb: bool = DccWidgets.touch_release_button(cb)
		_check(not changed_cb, "gate 2: a control already on RELEASE is declined")
		_check(cb.mouse_filter == Control.MOUSE_FILTER_STOP,
			"gate 2: and its mouse_filter is NOT rewritten (%s)" % _FILTER[cb.mouse_filter])
		## And a disabled dropdown inside a scroller: no gate claimed for it,
		## so this only records what actually happens.
		var dis := OptionButton.new()
		dis.add_item("a")
		dis.disabled = true
		rig.add_child(dis)
		await _frames(2)
		var changed_dis: bool = DccWidgets.touch_release_button(dis)
		_log("   a DISABLED dropdown inside a scroller: changed=%s am=%d filter=%s"
			% [str(changed_dis), dis.action_mode, _FILTER[dis.mouse_filter]])
		rig.queue_free()
		await _frames(2)
		## One arbiter: the lookup PgSlider uses and the one the button fix uses
		## must be the same function, and must agree on a v-DISABLED scroller.
		var outer := ScrollContainer.new()
		outer.custom_minimum_size = Vector2(300, 200)
		_vp.add_child(outer)
		var inner := ScrollContainer.new()
		inner.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
		outer.add_child(inner)
		var probe_ob := OptionButton.new()
		probe_ob.add_item("a")
		inner.add_child(probe_ob)
		await _frames(2)
		var found: ScrollContainer = DccWidgets.vertical_scroller_above(probe_ob)
		_check(found == outer,
			"vertical_scroller_above skips a v-DISABLED scroller and finds the outer one")
		_check(DccWidgets.touch_release_button(probe_ob),
			"and the fix therefore converts a dropdown nested under a v-disabled scroller")
		_log("   NOTE: `_gestclass_probe.gd::_scroller_of()` returns the nearest scroller of")
		_log("   ANY kind, so it would report that same node scroller=vDISABLED and NOT")
		_log("   count it -- census and fix disagree on exactly this shape.")
		outer.queue_free()
		await _frames(2)

	# -- PART H: is the menu bar even walked by `phone_fit()`? -----------------
	#
	# The shipped comment attributes the seven menu-bar `MenuButton`s staying
	# stock to `touch_release_button`'s no-scroller gate. That is only the reason
	# if `phone_fit()` reaches them at all -- which its own `_phone_fitted` meta
	# answers directly.
	if _want("H"):
		_log("-- PART H: `_phone_fitted` meta, i.e. did phone_fit() walk this node")
		var invh: Array = []
		var ioh: Dictionary = {}
		_walk(get_tree().root, invh, ioh)
		var mb_fit := 0
		var mb_n := 0
		var ob_fit := 0
		var ob_n := 0
		for n in invh:
			if n is MenuButton:
				mb_n += 1
				if (n as Node).has_meta("_phone_fitted"):
					mb_fit += 1
			elif n is OptionButton:
				ob_n += 1
				if (n as Node).has_meta("_phone_fitted"):
					ob_fit += 1
		_log("   MenuButton   %d in tree, %d carry `_phone_fitted`" % [mb_n, mb_fit])
		_log("   OptionButton %d in tree, %d carry `_phone_fitted`" % [ob_n, ob_fit])
		_check(mb_fit == 0,
			"the menu bar is NOT walked by phone_fit(), so the scroller gate is not what protects it")

	_log("RESULT %s fail=%d" % [_tag, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
