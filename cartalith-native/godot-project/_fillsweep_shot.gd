extends Node
## Real-app pixel evidence for Lane FILLS, windowed (headless renders nothing
## to diff -- `_phonechrome_probe.gd`'s own note, `MISTAKES.md` line 61).
##
## Targets, one section each: Faction Roster's selected row, the Travel
## Library kind tabs + selected rail row, Place Editor's trait chips (on and
## off), an L2 category header's hover wash, and CARTO's ramp "Stop editor"
## selected-stop row. Each section samples a pixel just inside a button's own
## rect (away from any glyph) and asserts the SELECTED/ACTIVE one differs from
## an UNSELECTED sibling sampled the same way -- the differential proof the
## fix actually draws, not just that "some pixel changed somewhere".
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 \
##       _fillsweep_shot.tscn

var fails := 0
var checks := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _grab() -> Image:
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()

## Finds the first Button under `root` whose text satisfies `pred`.
func _find_btn(root: Node, pred: Callable) -> Button:
	if root is Button and pred.call((root as Button).text):
		return root
	for c in root.get_children():
		var found := _find_btn(c, pred)
		if found != null:
			return found
	return null

## `AcceptDialog extends Window`, and `Control.get_global_rect()` on a control
## inside one is relative to THAT window's own local viewport, not the main
## one the capture reads -- confirmed by a first pass where Travel Library's
## window (`position=(210,110)`) sampled two clearly-different, correctly
## styled buttons as pixel-identical, both landing on whatever the main
## viewport shows at literal (3,56). Add the control's own window's position
## (zero for controls that already live in the main window, e.g. `left_dock`)
## to land in the captured image's actual coordinate space.
func _screen_rect(b: Control) -> Rect2:
	var r := b.get_global_rect()
	var w := b.get_window()
	if w != null and w != get_window():
		r.position += Vector2(w.position)
	return r

func _sample_left(b: Button, img: Image) -> Color:
	var r := _screen_rect(b)
	var x := int(r.position.x) + 3
	var y := int(r.position.y + r.size.y * 0.5)
	x = clampi(x, 0, img.get_width() - 1)
	y = clampi(y, 0, img.get_height() - 1)
	return img.get_pixel(x, y)

func _check(label: String, cond: bool, extra: String = "") -> void:
	checks += 1
	if not cond:
		fails += 1
	print("FILLSWEEP %-7s %s  %s" % ["PASS" if cond else "FAIL", label, extra])

func _ready() -> void:
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	app.bridge.generate({
		"seed": 483920, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while app.bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _frames(15)

	# -- A: Faction Roster selected row -----------------------------------
	var factions: Array = app.bridge.get_factions()
	if factions.size() >= 2:
		app.open_faction_roster()
		await _frames(4)
		var frw = app.faction_roster_window
		var f0: Dictionary = factions[0]
		var f1: Dictionary = factions[1]
		frw._selected = int(f0.get("id", 1))
		frw._rebuild_list()
		await _frames(3)
		var img := await _grab()
		var name0 := String(f0.get("name", "?"))
		var name1 := String(f1.get("name", "?"))
		var btn_sel := _find_btn(frw, func(t: String): return t.begins_with(name0))
		var btn_unsel := _find_btn(frw, func(t: String): return t.begins_with(name1))
		if btn_sel != null and btn_unsel != null:
			var c_sel := _sample_left(btn_sel, img)
			var c_unsel := _sample_left(btn_unsel, img)
			_check("A faction-roster selected-vs-unselected row differ",
				c_sel != c_unsel, "sel=%s unsel=%s" % [c_sel, c_unsel])
		else:
			_check("A faction-roster rows found", false, "sel_btn=%s unsel_btn=%s" % [btn_sel, btn_unsel])
		frw.hide()
		await _frames(2)
	else:
		print("FILLSWEEP SKIP    A faction-roster -- only %d faction(s)" % factions.size())

	# -- B: Travel Library kind tabs + selected rail row --------------------
	app.open_travel_library("")
	await _frames(4)
	var tlw = app.travel_library_window
	var tab_buttons: Dictionary = tlw._tab_buttons
	if tab_buttons.size() >= 2:
		var img_tabs := await _grab()
		var keys: Array = tab_buttons.keys()
		var active_key := String(tlw._current_kind)
		var other_key := ""
		for k in keys:
			if String(k) != active_key:
				other_key = String(k)
				break
		var active_btn: Button = tab_buttons[active_key]
		var other_btn: Button = tab_buttons[other_key]
		var a_sb: StyleBox = active_btn.get_theme_stylebox("normal")
		var o_sb: StyleBox = other_btn.get_theme_stylebox("normal")
		print("FILLSWEEP DEBUG   active_btn rect=%s flat=%s normal_sb=%s bg=%s" \
			% [active_btn.get_global_rect(), active_btn.flat, a_sb,
				(a_sb as StyleBoxFlat).bg_color if a_sb is StyleBoxFlat else "n/a"])
		print("FILLSWEEP DEBUG   other_btn  rect=%s flat=%s normal_sb=%s bg=%s" \
			% [other_btn.get_global_rect(), other_btn.flat, o_sb,
				(o_sb as StyleBoxFlat).bg_color if o_sb is StyleBoxFlat else "n/a"])
		print("FILLSWEEP DEBUG   window visible=%s size=%s position=%s" \
			% [tlw.visible, tlw.size, tlw.position])
		var c_active := _sample_left(active_btn, img_tabs)
		var c_other := _sample_left(other_btn, img_tabs)
		_check("B travel-library active-vs-inactive tab differ",
			c_active != c_other, "active(%s)=%s other(%s)=%s" % [active_key, c_active, other_key, c_other])
	else:
		_check("B travel-library tab buttons found", false, "count=%d" % tab_buttons.size())

	var entries: Array = tlw._bridge.tl_list(String(tlw._current_kind))
	if entries.size() >= 2:
		var id0 := String((entries[0] as Dictionary).get("id", ""))
		var id1 := String((entries[1] as Dictionary).get("id", ""))
		tlw._select_entry(id0)
		await _frames(3)
		var img_rail := await _grab()
		var name_sel := String((entries[0] as Dictionary).get("name", ""))
		var name_unsel := String((entries[1] as Dictionary).get("name", ""))
		var rail_sel := _find_btn(tlw._rail_body, func(t: String): return t.begins_with(name_sel))
		var rail_unsel := _find_btn(tlw._rail_body, func(t: String): return t.begins_with(name_unsel))
		if rail_sel != null and rail_unsel != null:
			print("FILLSWEEP DEBUG   rail_sel rect=%s flat=%s  rail_unsel rect=%s flat=%s" \
				% [rail_sel.get_global_rect(), rail_sel.flat, rail_unsel.get_global_rect(), rail_unsel.flat])
			var rc_sel := _sample_left(rail_sel, img_rail)
			var rc_unsel := _sample_left(rail_unsel, img_rail)
			_check("B travel-library selected-vs-unselected rail row differ",
				rc_sel != rc_unsel, "sel=%s unsel=%s" % [rc_sel, rc_unsel])
		else:
			_check("B travel-library rail rows found", false, "sel_btn=%s unsel_btn=%s" % [rail_sel, rail_unsel])
	else:
		print("FILLSWEEP SKIP    B travel-library rail -- only %d entries in %s" % [entries.size(), tlw._current_kind])
	tlw.hide()
	await _frames(2)

	# -- C: Place Editor trait chips, on vs off ------------------------------
	var settlements: Array = app.bridge.settlements()
	var vocab0: Array = app.bridge.civ_trait_vocabulary() if app.bridge.has_method("civ_trait_vocabulary") else []
	## Traits are a user EDIT (`civ_settlement_toggle_trait`), not generated
	## data -- a fresh world has none set, which is why the natural search this
	## replaced found zero of 240 settlements with a mixed on/off set. Toggle
	## one on directly, the same call the chip's own `pressed` handler makes.
	if settlements.size() > 0 and vocab0.size() > 0 and app.bridge.has_method("civ_settlement_toggle_trait"):
		app.bridge.civ_settlement_toggle_trait(0, String((vocab0[0] as Dictionary).get("key", "")))
	var traits_ok := false
	for si in settlements.size():
		var details: Dictionary = app.bridge.civ_settlement_details(si) if app.bridge.has_method("civ_settlement_details") else {}
		var on: PackedStringArray = details.get("traits", PackedStringArray())
		var vocab: Array = app.bridge.civ_trait_vocabulary() if app.bridge.has_method("civ_trait_vocabulary") else []
		if on.size() > 0 and on.size() < vocab.size():
			app.open_place_editor(si)
			await _frames(4)
			var pew = app.place_editor_window
			var on_key := String(on[0])
			var on_label := ""
			var off_label := ""
			for e in vocab:
				var d: Dictionary = e
				var key := String(d.get("key", ""))
				var lbl := "%s %s" % [String(d.get("glyph", "")), String(d.get("label", key))]
				if key == on_key and on_label == "":
					on_label = lbl
				elif not on.has(key) and off_label == "":
					off_label = lbl
			await _frames(2)
			var img_traits := await _grab()
			var btn_on := _find_btn(pew, func(t: String): return t == on_label)
			var btn_off := _find_btn(pew, func(t: String): return t == off_label)
			if btn_on != null and btn_off != null:
				var c_on := _sample_left(btn_on, img_traits)
				var c_off := _sample_left(btn_off, img_traits)
				_check("C place-editor trait chip on-vs-off differ",
					c_on != c_off, "on(%s)=%s off(%s)=%s" % [on_label, c_on, off_label, c_off])
				traits_ok = true
			else:
				_check("C place-editor trait chips found", false, "on_btn=%s off_btn=%s (on_label=%s off_label=%s)" % [btn_on, btn_off, on_label, off_label])
				traits_ok = true
			pew.hide()
			await _frames(2)
			break
	if not traits_ok:
		print("FILLSWEEP SKIP    C place-editor traits -- no settlement had a mixed on/off set")

	# -- D: an L2 category header's hover wash -------------------------------
	## Excludes the dock's own collapse chevron ("‹"/"›", found first in the
	## tree since it decorates the dock frame itself): a real `category()`
	## header's text is always a glyph plus an upper-cased title, length > 2.
	var cat_btn := _find_btn(app.left_dock, func(t: String): return t.length() > 2)
	if cat_btn != null:
		var img_before := await _grab()
		var before := _sample_left(cat_btn, img_before)
		var mm := InputEventMouseMotion.new()
		var r := _screen_rect(cat_btn)
		mm.position = Vector2(r.position.x + r.size.x * 0.5, r.position.y + r.size.y * 0.5)
		mm.global_position = mm.position
		Input.parse_input_event(mm)
		await _frames(3)
		var img_after := await _grab()
		var after := _sample_left(cat_btn, img_after)
		_check("D category-header hover wash differs from idle",
			before != after, "idle=%s hover=%s text=%s" % [before, after, cat_btn.text])
	else:
		_check("D category header found in left dock", false)

	# -- F: CARTO ramp Stop editor, selected-stop row ------------------------
	app.select_domain("cartography")
	app.arm_tool("inspect")
	await _frames(4)
	var rd = app.right_dock_ctrl
	print("FILLSWEEP INFO    right dock title after CARTO+inspect = %s  ramp_api=%s" % [rd._current_title(), app.bridge.ramp_api])
	var stops: Array = app.bridge.color_ramp()
	if app.bridge.ramp_api and stops.size() >= 2:
		rd._stops_selected = 0
		rd._rebuild()
		await _frames(3)
		var img_stops := await _grab()
		var sel_hex := String((stops[0][1] as Color).to_html(false)).to_upper()
		var unsel_hex := String((stops[1][1] as Color).to_html(false)).to_upper()
		## `rd` (`RightDock`) is a plain `Node` controller, not itself part of
		## the visible Control tree -- the rows it builds live under
		## `app.right_dock_body`, which is where the search has to look.
		var btn_sel := _find_btn(app.right_dock_body, func(t: String): return t == "selected")
		var btn_unsel := _find_btn(app.right_dock_body, func(t: String): return t == "select")
		if btn_sel != null and btn_unsel != null:
			var cs := _sample_left(btn_sel, img_stops)
			var cu := _sample_left(btn_unsel, img_stops)
			_check("F carto ramp-stop selected-vs-unselected row differ",
				cs != cu, "sel=%s unsel=%s (hex %s vs %s)" % [cs, cu, sel_hex, unsel_hex])
		else:
			_check("F carto ramp-stop rows found", false, "sel_btn=%s unsel_btn=%s" % [btn_sel, btn_unsel])
	else:
		print("FILLSWEEP SKIP    F carto ramp-stops -- ramp_api=%s stops=%d" % [app.bridge.ramp_api, stops.size()])

	print("FILLSWEEP TOTAL   %d checks, %d failed" % [checks, fails])
	get_tree().quit(0 if fails == 0 else 1)
