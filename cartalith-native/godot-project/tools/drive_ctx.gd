class_name DriveCtx
extends RefCounted
## TOOLING_PLAN.md M2, TP-3 (Ruling GP): helpers a Fennara runtime script uses to DRIVE THE REAL SHELL with
## real input events and READ its state back. Ours, not an edit of the MIT addon (`addons/fennara`), which
## stays upgradable. Every function is a `static` coroutine taking the script's `ctx` (a
## `runtime_script_context.gd`) first; call it `await DriveCtx.menu_open(ctx, "File", "New World")`.
##
## What it must never do: call a handler directly. A click is a mouse-move + press + release pushed into the
## root viewport (hit-testing, focus and popups run as for a hand); text is one `InputEventKey` pair per
## character into whatever holds focus. `dump_ui` only reads. Coordinates are canvas space, which is what
## `rect_of()` returns (a control's rect in the root's canvas; `Control.get_global_rect()` is NOT that inside an embedded
## dialog) and what `Viewport.push_input` expects.
##
## A PopupMenu row is not a Control, so `ctx.click_button` cannot reach it; `click_popup_item` does.

const _RING := "res://shell/radial_ring.gd"


## One frame, or `n`.
static func _frames(ctx, n: int = 1) -> void:
	var tree: SceneTree = ctx.get_scene_root().get_tree()
	for _i in n:
		await tree.process_frame


static func _vp(ctx) -> Viewport:
	return ctx.get_scene_root().get_viewport()


static func _motion(ctx, at: Vector2) -> void:
	var m := InputEventMouseMotion.new()
	m.position = at
	m.global_position = at
	_vp(ctx).push_input(m)


static func _button(ctx, at: Vector2, down: bool, btn: int = MOUSE_BUTTON_LEFT) -> void:
	var e := InputEventMouseButton.new()
	e.button_index = btn
	e.pressed = down
	e.position = at
	e.global_position = at
	e.button_mask = (1 << (btn - 1)) if down else 0
	_vp(ctx).push_input(e)


## Mouse move, press, release at `at` (canvas pixels). The real click `dump_ui` rects are meant for.
static func click(ctx, at: Vector2) -> void:
	_motion(ctx, at)
	await _frames(ctx, 2)
	_button(ctx, at, true)
	await _frames(ctx, 2)
	_button(ctx, at, false)
	await _frames(ctx, 4)


static func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


static func _all(ctx) -> Array:
	var out: Array = []
	_walk(ctx.get_scene_root().get_tree().root, out)
	return out


## A control's rect on the ROOT canvas, i.e. where a click must land. `get_global_rect()` is in the control's own
## window, and the shell's dialogs are embedded `Window`s with an offset AND a content scale (the phone's 1.09), so a
## rect read inside one is not a click position. `get_screen_transform()` folds both in. Found 2026-11-22: the welcome
## card's "Continue" read (525, 438) and sat at (735, 507); on the phone the seed field read (117, 478) and sat at (185, 546).
static func rect_of(c: Control) -> Rect2:
	return c.get_screen_transform() * Rect2(Vector2.ZERO, c.size)


static func _text_of(c: Control) -> String:
	if c is MenuBar:
		var t: PackedStringArray = []
		for i in (c as MenuBar).get_menu_count():
			t.append((c as MenuBar).get_menu_title(i))
		return " | ".join(t)
	if "text" in c:
		return str(c.get("text"))
	return ""


static func _enabled_of(c: Control) -> bool:
	if c is BaseButton:
		return not (c as BaseButton).disabled
	if c is LineEdit:
		return (c as LineEdit).editable
	if c is TextEdit:
		return (c as TextEdit).editable
	if c is Range:
		return (c as Range).editable if "editable" in c else true
	return true


## Every VISIBLE Control (and, for each visible PopupMenu, its rows) as
## `{path, class, text, rect:[x,y,w,h], enabled, tooltip}`; popup rows are `{path:"<popup>#<i>", class:"PopupMenu.item", ...}`.
## "Visible" is `is_visible_in_tree()` and a rect with area that meets the screen, so a hidden, parked or scrolled-away control is not listed.
## `contains` (optional) keeps only entries whose text or path contains it, case-insensitively.
static func dump_ui(ctx, contains: String = "") -> Array:
	var out: Array = []
	var q := contains.to_lower()
	var screen: Rect2 = ctx.get_scene_root().get_viewport().get_visible_rect()
	for n in _all(ctx):
		if n is Control:
			var c := n as Control
			if not c.is_visible_in_tree():
				continue
			var r := rect_of(c)
			if r.size.x <= 0.0 or r.size.y <= 0.0 or not screen.intersects(r):
				continue
			out.append({"path": str(c.get_path()), "class": c.get_class(), "text": _text_of(c),
				"rect": [r.position.x, r.position.y, r.size.x, r.size.y], "enabled": _enabled_of(c),
				"tooltip": c.tooltip_text})
		elif n is PopupMenu and (n as PopupMenu).visible:
			var p := n as PopupMenu
			for i in p.item_count:
				if p.is_item_separator(i):
					continue
				out.append({"path": "%s#%d" % [p.get_path(), i], "class": "PopupMenu.item", "text": p.get_item_text(i),
					"rect": [p.position.x, p.position.y, p.size.x, p.size.y], "enabled": not p.is_item_disabled(i),
					"tooltip": p.get_item_tooltip(i)})
	if q != "":
		out = out.filter(func(e): return str(e["text"]).to_lower().contains(q) or str(e["path"]).to_lower().contains(q))
	return out


## Focus `node` with a real click on its centre (skipped if it already has focus), then send one key
## press/release per character of `s` (`\n` is Enter). Returns the control's `text` afterwards so the caller can
## assert, but assert from `dump_ui` too: that is the independent read-back.
static func type_text(ctx, node: Variant, s: String, replace: bool = true) -> String:
	var c: Control = ctx.node(node)
	if c.get_viewport().gui_get_focus_owner() != c:
		await click(ctx, rect_of(c).get_center())
		if c.get_viewport().gui_get_focus_owner() != c:
			ctx.error("type_text: %s did not take focus from a click" % c.get_path())
			return ""
	if replace:
		for down in [true, false]:
			var a := InputEventKey.new()
			a.pressed = down
			a.keycode = KEY_A
			a.physical_keycode = KEY_A
			a.ctrl_pressed = true
			_vp(ctx).push_input(a)
		await _frames(ctx, 1)
	for ch in s:
		var code := ch.unicode_at(0)
		for down in [true, false]:
			var k := InputEventKey.new()
			k.pressed = down
			k.keycode = KEY_ENTER if ch == "\n" else (OS.find_keycode_from_string(ch.to_upper()) if ch.to_upper() != ch.to_lower() or ch.is_valid_int() else code)
			k.physical_keycode = k.keycode
			k.unicode = 0 if ch == "\n" else code
			_vp(ctx).push_input(k)
		await _frames(ctx, 1)
	await _frames(ctx, 2)
	return str(c.get("text")) if "text" in c else ""


static func _visible_popup(ctx, below: Node = null) -> PopupMenu:
	for n in _all(ctx):
		if n is PopupMenu and (n as PopupMenu).visible and (below == null or below.is_ancestor_of(n)):
			return n
	return null


## The visible bar entry titled `name`: a `MenuButton` (the shell's desktop and tablet bars) -> its node, else null.
static func _menu_button(ctx, name: String) -> MenuButton:
	for n in _all(ctx):
		if n is MenuButton and (n as MenuButton).is_visible_in_tree() and (n as MenuButton).text.to_lower() == name.to_lower():
			return n
	return null


## Press the real key `code` once (down and up) into the root viewport.
static func tap_key(ctx, code: Key) -> void:
	for down in [true, false]:
		var k := InputEventKey.new()
		k.pressed = down
		k.keycode = code
		k.physical_keycode = code
		_vp(ctx).push_input(k)
		await _frames(ctx, 1)


## Activate the row `label` (case-insensitive, `contains` fallback) of an open popup with real KEY events: Down until
## `get_focused_item()` is that row (separators and disabled rows are skipped by the popup itself), then Enter.
## Why keys and not a click: a PopupMenu has no row-rect API, and its window sits in a scaled embedded space whose
## `position`/`size` do not match root canvas pixels (measured 2026-11-22: size 370 drawn at about 300 canvas px), so a
## hover scan over `position` never focused a row. The menu's own keyboard path is the same code a hand reaches.
static func click_popup_item(ctx, p: PopupMenu, label: String) -> bool:
	var want := -1
	for pass_exact in [true, false]:
		for i in p.item_count:
			var t := p.get_item_text(i).to_lower().trim_suffix("…")
			var l := label.to_lower().trim_suffix("…")
			if (t == l) if pass_exact else t.contains(l):
				want = i
				break
		if want >= 0:
			break
	if want < 0 or p.is_item_disabled(want) or p.is_item_separator(want):
		ctx.error("click_popup_item: no enabled row '%s' in %s (rows: %s)" % [label, p.name, ", ".join(range(p.item_count).map(func(i): return p.get_item_text(i)))])
		return false
	var guard := p.item_count + 2
	while p.get_focused_item() != want and guard > 0:
		await tap_key(ctx, KEY_DOWN)
		guard -= 1
	if p.get_focused_item() != want:
		ctx.error("click_popup_item: Down never reached row '%s'" % label)
		return false
	await tap_key(ctx, KEY_ENTER)
	await _frames(ctx, 3)
	return true


## Open a menu by `name` and, if `item` is given, click that row by label.
##  * a bar `MenuButton` (File, Edit, Preferences ...): a real click on it, then the popup row;
##  * the phone (no bar): the bottom MENU tab, then the menu's row, then the item's row, all by label;
##  * `"ring"`: the cartwheel, opened as the Q key does over the map; `item` is a slot label (or a sub-ring row after);
##  * otherwise a PopupMenu already open whose node name or first row matches (a context menu you opened by hand).
## Returns true when the menu opened and (if asked) the row was clicked. Reads, never calls a handler.
static func menu_open(ctx, name: String, item: String = "") -> bool:
	if name.to_lower() == "ring":
		return await _ring(ctx, item)
	var mb := _menu_button(ctx, name)
	if mb != null:
		await click(ctx, rect_of(mb).get_center())
		var popup := mb.get_popup()
		await _frames(ctx, 3)
		if not popup.visible:
			ctx.error("menu_open: clicking the '%s' bar entry did not open its popup" % name)
			return false
		return item == "" or await click_popup_item(ctx, popup, item)
	if name.to_lower() != "ring" and _menu_button(ctx, name) == null and await _phone_menu(ctx, name, item):
		return true
	for n in _all(ctx):
		if n is PopupMenu and (n as PopupMenu).visible and (n.name.to_lower() == name.to_lower() or ((n as PopupMenu).item_count > 0 and (n as PopupMenu).get_item_text(0).to_lower() == name.to_lower())):
			return item == "" or await click_popup_item(ctx, n, item)
	ctx.error("menu_open: no bar menu, ring or open popup named '%s'" % name)
	return false


static func _find_ring(ctx) -> Node:
	for n in _all(ctx):
		if n.get_script() != null and (n.get_script() as Script).resource_path == _RING:
			return n
	return null


static func _ring(ctx, item: String) -> bool:
	var ring := _find_ring(ctx)
	if ring == null:
		ctx.error("menu_open ring: no radial_ring.gd node in the tree")
		return false
	var vp := _vp(ctx)
	var centre := vp.get_visible_rect().get_center()
	_motion(ctx, centre)
	await _frames(ctx, 2)
	for down in [true, false]:
		var k := InputEventKey.new()
		k.pressed = down
		k.keycode = KEY_Q
		k.physical_keycode = KEY_Q
		vp.push_input(k)
	await _frames(ctx, 4)
	if not ring.is_open():
		ctx.error("menu_open ring: Q over the map did not open the ring (is the map under the pointer?)")
		return false
	if item == "":
		return true
	var st: Dictionary = ring.debug_state()
	var c: Vector2 = st["centre"]
	for dir in ring.DIRS:
		var slot: Variant = ring._slots.get(dir)
		if slot is Dictionary and str(slot.get("label", "")).to_lower() == item.to_lower():
			await click(ctx, c + Vector2.from_angle(deg_to_rad(float(ring.ANGLES[dir]))) * float(st["radius"]))
			return true
	ctx.error("menu_open ring: no slot labelled '%s' (slots: %s)" % [item, str(ring._slots.keys())])
	return false


## The phone has no bar: the bottom MENU tab opens a list of the eight menus (rows drawn as Labels over a button), and a
## row opens that menu's own rows the same way. `true` when `name` (and `item`, if given) were clicked; `false` when this
## is not the phone shell (no MENU tab) or a row is missing (then it reports why through ctx.error).
static func _phone_menu(ctx, name: String, item: String) -> bool:
	var tabs := dump_ui(ctx, "MENU").filter(func(e): return str(e["text"]) == "MENU")
	if tabs.is_empty():
		return false
	tabs.sort_custom(func(a, b): return a["rect"][1] > b["rect"][1])   # the bottom tab, not the screen title
	await _click_text(ctx, tabs[0])
	await _frames(ctx, 20)
	var row := _row(ctx, name)
	if row.is_empty():
		ctx.error("menu_open (phone): no MENU row '%s'" % name)
		return false
	await _click_text(ctx, row)
	await _frames(ctx, 20)
	if item == "":
		return true
	var it := _row(ctx, item)
	if it.is_empty():
		ctx.error("menu_open (phone): no row '%s' under '%s'" % [item, name])
		return false
	await _click_text(ctx, it)
	await _frames(ctx, 10)
	return true


static func _row(ctx, label: String) -> Dictionary:
	var l := label.to_lower().trim_suffix("…")
	for pass_exact in [true, false]:
		for e in dump_ui(ctx):
			var t := str(e["text"]).to_lower().trim_suffix("…")
			if t != "" and ((t == l) if pass_exact else t.begins_with(l)) and e["rect"][1] > 100.0:
				return e
	return {}


static func _click_text(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))
