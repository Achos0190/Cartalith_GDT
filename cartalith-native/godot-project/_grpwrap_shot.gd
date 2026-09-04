extends Node
## Lane B / batch 25, part 3 -- what the `group()` header change costs and buys.
##
## Parts 1 and 2 measured the tree before it. This one measures both sides of
## the ledger on one run by taking the SHIPPED tree and reverting the change at
## runtime -- every live group header back to `AUTOWRAP_OFF`, which is exactly
## what `DccWidgets.group()` produced before this batch:
##
##   COST    the desktop framebuffer at the SHIPPED dock widths, shipped
##           against reverted, differing-pixel count. The bar is byte-identical.
##   BENEFIT the width each dock actually stops at when dragged to its floor,
##           shipped against reverted.
##
## Run in this direction on purpose. Staging the *candidate* on top of an
## unchanged tree measures a tree nobody will ever run; reverting the shipped
## one measures the tree that ships, which is the only one a regression can
## appear in.
##
## Windowed, because `--headless` renders nothing to diff (`_rdappend_shot.gd`'s
## own note, and MISTAKES.md's `ImageTexture.update()` row). Differing/total is
## a palette-agnostic measure, not a brightness threshold, so it is not the
## palette-bound trap -- the palette is printed anyway so the run is identifiable.
##
## The positive control is a group TOGGLE, which must move pixels: a harness
## that reports 0 for the candidate is only evidence if it can report non-zero
## for something.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 \
##       _grpwrap_shot.tscn

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _grab() -> Image:
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()

func _diff(a: Image, b: Image) -> Dictionary:
	if a.get_width() != b.get_width() or a.get_height() != b.get_height():
		return {"n": -1, "total": 0}
	var n := 0
	for y in a.get_height():
		for x in a.get_width():
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
	return {"n": n, "total": a.get_width() * a.get_height()}

## A `DccWidgets.group()` header, and NOT merely a button whose label starts
## with the same glyph. This predicate took two corrections, both caught by
## widening the sweep rather than by reading it:
##
##   1. `begins_with(sigil + " ")` alone counted `+ Add faction` -- a chip in
##      an `HBoxContainer` -- as a group header 16 times per seed, which would
##      have shipped a false claim about no call site handing this factory a
##      distributing parent.
##   2. Adding "the text after the sigil is upper-case" fixed that and broke
##      the headers that matter most: `civilization_workspace.gd`'s
##      `_lm_refresh_group()` appends `"   %d of %d armed · %d placed"` in
##      LOWER case, so the widest header in the shell (280 px) was silently
##      excluded from every count.
##
## So the test is STRUCTURAL, off what `group()` actually builds: a flat,
## unfocusable `Button` at the group header's own font size, followed
## immediately by the `MarginContainer` holding its body `VBoxContainer`.
## Nothing else in the shell has that shape, and no amount of runtime text
## rewriting can move it.
func _is_group_header(c: Node) -> bool:
	if not (c is Button):
		return false
	var b := c as Button
	if not b.flat or b.focus_mode != Control.FOCUS_NONE:
		return false
	var t := String(b.text)
	if not (t.begins_with(DccIcons.SYMBOLS["expand"] + " ") or t.begins_with("+ ")):
		return false
	var want := DccTheme.role_px("fs_dock_header") if DccTheme.is_tablet() else DccTheme.FS_HEADER
	if b.get_theme_font_size("font_size") != want:
		return false
	var p := b.get_parent()
	if p == null or b.get_index() + 1 >= p.get_child_count():
		return false
	var pad := p.get_child(b.get_index() + 1)
	if not (pad is MarginContainer) or pad.get_child_count() == 0:
		return false
	return pad.get_child(0) is VBoxContainer

## `action()`'s guard, verbatim in intent: the question is "does a sibling
## compete for my width", not "which class is my parent". `BoxContainer`
## non-vertical, `HFlowContainer` and `GridContainer` all answer yes.
func _shares_width(parent: Node) -> bool:
	return ((parent is BoxContainer) and not (parent as BoxContainer).vertical) 		or (parent is HFlowContainer) or (parent is GridContainer)

func _apply(root: Node, on: bool, out: Array) -> void:
	for c in root.get_children():
		if _is_group_header(c) and not _shares_width(c.get_parent()):
			(c as Button).autowrap_mode = (TextServer.AUTOWRAP_WORD_SMART if on
				else TextServer.AUTOWRAP_OFF)
			out.append(c)
		_apply(c, on, out)

## Width the dock settles at when dragged to `floor_px`, then put back.
func _drag_to(dock: Control, floor_px: int) -> float:
	var was := dock.custom_minimum_size.x
	dock.custom_minimum_size.x = float(floor_px)
	await _frames(4)
	var got := dock.size.x
	dock.custom_minimum_size.x = was
	await _frames(3)
	return got

func _surface(tag: String, dock: Control, floor_px: int) -> void:
	await _frames(6)
	var shipped := await _grab()
	var stops_shipped: float = await _drag_to(dock, floor_px)
	var touched: Array = []
	_apply(app, false, touched)          ## revert to the pre-batch header
	await _frames(8)
	var reverted := await _grab()
	var stops_reverted: float = await _drag_to(dock, floor_px)
	var d := _diff(shipped, reverted)
	print("GW %-32s headers=%3d  px_differ=%6d / %d   floor=%d  stops shipped=%.0f reverted=%.0f"
		% [tag, touched.size(), int(d["n"]), int(d["total"]), floor_px,
			stops_shipped, stops_reverted])
	if int(d["n"]) != 0:
		_fail += 1
		print("GW    !! NOT byte-identical at the shipped dock width")
	var back: Array = []
	_apply(app, true, back)              ## put the shipped behaviour back
	await _frames(6)

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	print("GW palette=%s  is_laptop=%s  is_tablet=%s  left_w=%.0f right_w=%.0f"
		% ["dark" if DccTheme.is_dark() else "light",
			DccTheme.is_laptop(), DccTheme.is_tablet(),
			app.left_dock.custom_minimum_size.x, app.right_dock.custom_minimum_size.x])

	app.bridge.generate({"seed": 483920, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while app.bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _frames(14)
	if not app.bridge.has_world:
		print("GW  !! generate failed")
		get_tree().quit(1)
		return

	# -- positive control: a toggle MUST move pixels ---------------------------
	app.select_domain_mode("civilization", "landmarks")
	await _frames(14)
	var found: Array = []
	_collect_headers(app.left_dock_body, found)
	if found.is_empty():
		print("GW  !! no group header on CIVIL/landmarks -- the control cannot run")
		get_tree().quit(1)
		return
	var ctl_before := await _grab()
	(found[0] as Button).emit_signal("pressed")
	await _frames(8)
	var ctl_after := await _grab()
	var cd := _diff(ctl_before, ctl_after)
	print("GW CONTROL toggle '%s': px_differ=%d / %d" % [
		String((found[0] as Button).text), int(cd["n"]), int(cd["total"])])
	if int(cd["n"]) <= 0:
		_fail += 1
		print("GW    !! the control moved nothing -- every 0 below is vacuous")
	(found[0] as Button).emit_signal("pressed")
	await _frames(8)

	# -- cost + benefit, per surface ------------------------------------------
	await _surface("CIVIL/landmarks (left)", app.left_dock, DccTheme.W_LEFT_DOCK_MIN)
	app.select_domain_category("civilization", "Military")
	await _frames(16)
	await _surface("CIVIL/Military (left)", app.left_dock, DccTheme.W_LEFT_DOCK_MIN)
	app.select_domain_mode("cartography", "style")
	await _frames(16)
	await _surface("CARTO/style (left)", app.left_dock, DccTheme.W_LEFT_DOCK_MIN)

	var gs: Vector2i = app.bridge.grid_size()
	app.bridge.route_begin("mixed")
	app.bridge.route_append_stop(gs.x * 0.20, gs.y * 0.30)
	app.bridge.route_append_stop(gs.x * 0.55, gs.y * 0.50)
	app.bridge.route_append_stop(gs.x * 0.82, gs.y * 0.72)
	app.bridge.route_commit()
	app.select_domain("civilization")
	await _frames(4)
	var settlements: Array = app.bridge.settlements()
	if not settlements.is_empty():
		app.right_dock_ctrl.on_settlement_selected(settlements[0], 0)
	app.arm_tool("journey")
	await _frames(18)
	await _surface("RIGHT journey", app.right_dock, DccTheme.W_RIGHT_DOCK_MIN)

	## **The desktop density, forced.** This machine's usable client area tops
	## out around 1904 px, so `_compute_layout_mode()` never leaves the LAPTOP
	## band from `--resolution` alone (measured: 2100x1150 requested, viewport
	## still 1736204 px and `is_laptop=true`). `DccTheme.LAPTOP` overrides
	## exactly three roles -- `w_left_dock`, `w_right_dock`, `w_menu_popup` --
	## and `DccWidgets.group()` reads none of them, so for THIS question the
	## band is the two dock widths and nothing else. Writing them is therefore
	## the desktop geometry, not an approximation of it.
	if not DccTheme.is_tablet():
		app.left_dock.custom_minimum_size.x = float(DccTheme.W_LEFT_DOCK)
		app.right_dock.custom_minimum_size.x = float(DccTheme.W_RIGHT_DOCK)
		await _frames(10)
		print("GW -- forced desktop widths: left=%d right=%d --"
			% [DccTheme.W_LEFT_DOCK, DccTheme.W_RIGHT_DOCK])
		await _surface("DESKTOP RIGHT journey", app.right_dock, DccTheme.W_RIGHT_DOCK_MIN)
		app.select_domain_category("civilization", "Military")
		await _frames(16)
		await _surface("DESKTOP CIVIL/Military", app.left_dock, DccTheme.W_LEFT_DOCK_MIN)

	print("GW === %s ===" % ("all surfaces byte-identical" if _fail == 0 else "%d problems" % _fail))
	get_tree().quit(0)

func _collect_headers(root: Node, out: Array) -> void:
	for c in root.get_children():
		if c is Control and not (c as Control).visible:
			continue
		if _is_group_header(c):
			out.append(c)
		_collect_headers(c, out)
