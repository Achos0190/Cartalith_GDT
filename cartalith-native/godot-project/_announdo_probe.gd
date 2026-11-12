extends "res://_edfc_probe.gd"
## Undo and redo for label, icon and landmark-name edits (`EDIT_FROM_CONTEXT_AUDIT.md`, "Undo for label, icon and name
## edits"; `annotation_undo.rs`), through the real shell with real input. It reuses `_edfc_probe.gd`'s boot, pointer events,
## prompt answering and element finders (`extends`), and overrides only `_run`.
##
##   godot --path . --resolution 1600x1000 _announdo_probe.tscn -- --vp 1600x1000                        (desktop)
##   godot --path . --resolution 1600x1000 _announdo_probe.tscn -- --vp 1600x1000 --force-touch          (the touch tablet)
##   ... -- --vp 1080x2340 --density 2.8125 --force-touch                                                (the phone)
## Arguments are the parent's (`--vp`, `--shots`, `--only`, `--density`, `--force-touch`). Windowed, wiped user dir, one lock.
##
## Legs (every assertion is a literal written by hand, never the constant it came from):
##   UB  the stores: add, rename, move, delete of a label; place, move, resize (one drag), delete of an icon; a landmark
##       rename. Each is ONE step: the real undo key puts the store back byte for byte, the redo key re-applies it, and
##       the row names the edit. Deleting the first of two keeps the second at its index after undo.
##   UI  the controls of each form: the undo and redo keys, the Edit menu row (its text names the edit), the toolbar
##       squares (desktop, tablet) and the phone's undo chip, each taking the step and showing the state.
##   UT  interleaving: annotation, height, way and look steps undo newest first; the colour-paint stack is independent;
##       a new edit truncates redo; the History ledger row reverts to it.
##   UD  real gestures: a label dragged with the Label tool is ONE step (many motion samples); a label created through
##       the tool's prompt (a create, a reclass and a style set) is ONE step.
##   UG  the assistant's grouped undo: a group holding label, icon, landmark-name and height members undoes and redoes in
##       ONE press, and the engine digest (now carrying the three stores) equals the pre-group one.
##   US  save and load: the history is session-only, like the height and way stacks (nothing to undo after a reopen); the
##       edited stores themselves survive it.
##   UH  selection hazards: deleting or undoing renumbers a store, so a selection and the Alt+Left history of that kind are
##       dropped, never left naming a neighbour.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

const LookHistory := preload("res://shell/look_history.gd")

var _bridge


## The key `code` pressed and released through the host viewport (the shell's accelerators run from there).
func _key(code: Key, ctrl := true, shift := false) -> void:
	# A closed dialog's field can still hold the viewport's keyboard focus (a Ctrl+Z then undoes ITS text); a user's click on
	# the map takes it back, so the probe releases it.
	_vp.gui_release_focus()
	# One frame first: a user's key cannot arrive in the frame of the edit before it, and the Edit menu's Undo row (a
	# disabled row swallows its accelerator) is refreshed by a signal the bridge emits at the end of that frame.
	await _frames(2)
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		ev.ctrl_pressed = ctrl
		ev.shift_pressed = shift
		_vp.push_input(ev)
		await _frames(2)
	await _frames(4)


func _carto():
	for ws in _app._workspaces:
		if ws is CartographyWorkspace:
			return ws
	return null


## The three stores as one comparable value.
func _stores() -> Dictionary:
	return {"labels": _bridge.label_list(), "icons": _bridge.icon_list(), "names": _bridge.landmark_names()}


## Undo through the control this form has: the real key on desktop and the touch tablet, the chip on the phone.
func _undo_ctl() -> void:
	if DccTheme.is_phone():
		# The chip is shown or hidden by a signal the bridge emits at the end of the frame of the edit, so a frame passes first
		# (a finger cannot arrive sooner). The phone has no menu bar, so there is no Ctrl+Z to fall back on: a hidden chip
		# is a failure, not a reason to press the key.
		# A selection opens the phone's right sheet over the map's lower part, and the chip sits under it; a user closes the
		# sheet first, so the probe does (the chip itself is what is under test, not the sheet's z-order).
		_app.phone_hide_right_sheet()
		_app.set_phone_detent("peek")
		await _frames(4)
		var chip: Button = _app.get("_phone_undo_chip")
		_ok(chip != null and chip.visible, "the phone's undo chip is shown while there is a step to undo")
		if chip != null and chip.visible:
			await _tap(chip.get_global_rect().get_center())
		return
	await _key(KEY_Z, true)


## Redo through the key (the phone has no redo control but Edit > Redo, which `redo_last` is).
func _redo_ctl() -> void:
	if DccTheme.is_phone():
		_app.redo_last()
		await _frames(4)
		return
	await _key(KEY_Y, true)


func _run(vw: int, vh: int) -> void:
	_vw = vw
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	_bridge = _app.bridge
	_bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42})
	await _wait_gen(_bridge)
	await get_tree().create_timer(1.0).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	if DccTheme.is_phone():
		_app._pick_phone_tab("map")
		_app.set_phone_detent("peek")
	_ov = _app.viewport.overlay
	_app.viewport.reset_view()
	await _frames(6)
	_ok(_bridge.has_world, "a world is up")
	_ok(_touch == _app.viewport._touch, "the form matches the flags (touch %s)" % str(_touch))
	await _bridge.landmark_run()
	await _frames(4)
	var pack: String = ProjectSettings.globalize_path("res://") + "../crates/cartalith-assets/tests/fixtures/reference_pack.zip"
	_ok(_bridge.world_gen.load_asset_pack(pack), "the fixture asset pack loaded (icons need one)")
	await _frames(4)
	_bridge.clear_undo()
	await _frames(2)
	_ok(not _app.can_undo_any(), "a fresh world has nothing to undo (positive control for every later check)")

	if _runs("UB"):
		await _leg_stores()
	if _runs("UI"):
		await _leg_controls()
	if _runs("UT"):
		await _leg_interleave()
	if _runs("UD"):
		await _leg_gestures()
	if _runs("UG"):
		await _leg_group()
	if _runs("UH"):
		await _leg_selection()
	if _runs("US"):
		await _leg_save_load()


## The visible one-field prompt (an `AcceptDialog`, which the Label tool's New-label dialog is, not a ConfirmationDialog),
## typed into and confirmed. False when there is none.
func _answer_accept(text: String) -> bool:
	var found: AcceptDialog = null
	var stack: Array = [_app]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is AcceptDialog and (n as AcceptDialog).visible and not n.is_queued_for_deletion():
			found = n
		for c in n.get_children():
			stack.append(c)
	if found == null:
		return false
	var le := _find_line_edit(found)
	if le == null:
		return false
	le.text = text
	found.confirmed.emit()
	found.hide()
	await _frames(4)
	return true


# ---- UB ------------------------------------------------------------------------------------------------------------

func _leg_stores() -> void:
	print("== UB the stores ==")
	var b = _bridge
	var base := _stores()
	# -- labels
	var ws = _carto()
	var ia: int = ws._create_label(40.0, 40.0, "Alpha")
	var ib: int = ws._create_label(60.0, 50.0, "Bravo")
	_ok(ia == 0 and ib == 1, "UB two labels exist at indices 0 and 1 (%d, %d)" % [ia, ib])
	var a0: Dictionary = b.label_get(0)
	_ok(String(b.undo_label()) == "Add label “Bravo”", "UB the row for a new label (a create, a reclass and a style set) reads Add label “Bravo” (%s)" % b.undo_label())
	await _undo_ctl()
	_ok(b.label_list().size() == 1, "UB ONE undo removes the whole new label, not just its last engine call (%d left)" % b.label_list().size())
	_ok(String(b.undo_label()) == "Add label “Alpha”", "UB and the next row is the first label's add (%s)" % b.undo_label())
	await _redo_ctl()
	_ok(b.label_list().size() == 2 and String(b.label_get(1).get("text", "")) == "Bravo", "UB the redo key brings Bravo back at index 1")
	# rename (the context card's one-shot path)
	var r: Dictionary = b.label_set(0, {"text": "Alpha Two"})
	_ok(bool(r.get("ok", false)) and String(b.label_get(0)["text"]) == "Alpha Two", "UB renamed")
	_ok(String(b.undo_label()) == "Rename label “Alpha” to “Alpha Two”", "UB the rename row names both texts (%s)" % b.undo_label())
	await _undo_ctl()
	_ok(b.label_get(0) == a0, "UB undo restores the label's every field (%s vs %s)" % [str(b.label_get(0)), str(a0)])
	await _redo_ctl()
	_ok(String(b.label_get(0)["text"]) == "Alpha Two", "UB redo renames again")
	await _undo_ctl()
	# move (one shot)
	var before_move: Dictionary = b.label_get(0)
	_ok(b.label_move(0, 90.0, 70.0), "UB moved")
	_ok(is_equal_approx(float(b.label_get(0)["x"]), 90.0), "UB the label is at x 90")
	await _undo_ctl()
	_ok(b.label_get(0) == before_move, "UB undo puts the label back where it was")
	await _redo_ctl()
	_ok(is_equal_approx(float(b.label_get(0)["x"]), 90.0) and is_equal_approx(float(b.label_get(0)["y"]), 70.0), "UB redo moves it again")
	await _undo_ctl()
	# delete the FIRST of two: the second must come back at index 1
	var both: Array = b.label_list()
	_ok(b.label_delete(0), "UB deleted label 0")
	_ok(b.label_list().size() == 1 and String(b.label_get(0)["text"]) == "Bravo", "UB Bravo moved up to index 0")
	await _undo_ctl()
	_ok(b.label_list() == both, "UB undo puts Alpha back at index 0 and Bravo at 1 (indices are the store's order)")
	await _redo_ctl()
	_ok(b.label_list().size() == 1, "UB redo deletes it again")
	await _undo_ctl()
	# clear all labels is one step
	b.label_clear_all()
	_ok(b.label_list().is_empty(), "UB cleared")
	await _undo_ctl()
	_ok(b.label_list() == both, "UB ONE undo brings back every cleared label")
	# -- icons
	_ok(b.icon_arm("feature", 0, 1.0, 0.0, 0.0), "UB an icon armed")
	var i0: int = b.icon_place(100.0, 60.0)
	var i1: int = b.icon_place(120.0, 60.0)
	b.icon_disarm()
	_ok(i0 == 0 and i1 == 1, "UB two icons placed (%d, %d)" % [i0, i1])
	_ok(String(b.undo_label()).begins_with("Place icon"), "UB the placement row reads Place icon… (%s)" % b.undo_label())
	await _undo_ctl()
	_ok(b.icon_list().size() == 1, "UB undo removes the second icon")
	await _redo_ctl()
	var icons2: Array = b.icon_list()
	_ok(icons2.size() == 2, "UB redo puts it back")
	_ok(b.icon_select_set(PackedInt64Array([1])), "UB icon 1 selected (a resize needs the selection)")
	var s_before := float(b.icon_get(1)["scale"])
	b.annotation_stream_begin()
	for k in 6:
		b.icon_resize(1, 120.5, 60.5, 122.0 + float(k), 60.5, 2.0)
	b.annotation_stream_end()
	var s_after := float(b.icon_get(1)["scale"])
	_ok(not is_equal_approx(s_after, s_before), "UB the resize drag changed the scale (%s -> %s)" % [str(s_before), str(s_after)])
	_ok(String(b.undo_label()).begins_with("Resize icon"), "UB the row reads Resize icon… (%s)" % b.undo_label())
	await _undo_ctl()
	_ok(is_equal_approx(float(b.icon_get(1)["scale"]), s_before), "UB ONE undo reverses six resize samples")
	await _redo_ctl()
	_ok(is_equal_approx(float(b.icon_get(1)["scale"]), s_after), "UB redo restores the final scale")
	await _undo_ctl()
	var moved_from: Dictionary = b.icon_get(0)
	_ok(b.icon_move(0, 30.0, 30.0), "UB icon moved")
	await _undo_ctl()
	_ok(b.icon_get(0) == moved_from, "UB undo puts the icon back")
	var all_icons: Array = b.icon_list()
	_ok(b.icon_delete(0), "UB icon 0 deleted")
	await _undo_ctl()
	_ok(b.icon_list() == all_icons, "UB undo restores both icons in order")
	b.icon_clear_all()
	await _undo_ctl()
	_ok(b.icon_list() == all_icons, "UB ONE undo restores the cleared icons")
	# -- landmark name
	var lms: Array = b.landmark_names()
	_ok(not lms.is_empty(), "UB landmarks exist (%d)" % lms.size())
	var key := String((lms[0] as Dictionary)["key"])
	var gen_name := String((lms[0] as Dictionary)["name"])
	_ok(b.landmark_set_user_name(key, "The Weeping Stair"), "UB landmark renamed")
	_ok(String(b.undo_label()) == "Rename landmark to “The Weeping Stair”", "UB the row names the new name (%s)" % b.undo_label())
	await _undo_ctl()
	var back: Dictionary = (b.landmark_names()[0] as Dictionary)
	_ok(String(back["name"]) == gen_name and String(back["source"]) != "user", "UB undo returns the generated name (%s)" % str(back))
	await _redo_ctl()
	_ok(String((b.landmark_names()[0] as Dictionary)["name"]) == "The Weeping Stair", "UB redo names it again")
	await _undo_ctl()
	# a refused edit records nothing
	var depth0 := int(b.undo_stats().get("annotation_depth", -1))
	_ok(not b.label_move(99, 1.0, 1.0) and not b.label_delete(99) and not b.icon_delete(99), "UB edits of a missing index are refused")
	_ok(int(b.undo_stats().get("annotation_depth", -2)) == depth0, "UB a refused edit adds no step (depth %d)" % depth0)
	# the notes follow a move and the undo returns them (Ruling EQ item 2)
	var root := ProjectSettings.globalize_path("user://") + "un_vault"
	DirAccess.make_dir_recursive_absolute(root + "/Labels")
	var wf := FileAccess.open(root + "/Labels/Alpha.md", FileAccess.WRITE)
	wf.store_string("# Alpha\n")
	wf.close()
	_ok(bool(b.vault_connect(root, "UN").get("ok", false)), "UB a vault is connected")
	var e1 := int(b.vault_label_entity_id(0).get("entity_id", 0))
	_ok(bool(b.vault_attach("label", e1, "Alpha", "Labels/Alpha.md", "").get("ok", false)), "UB a note is attached to label 0")
	_ok(b.label_move(0, 33.0, 33.0), "UB label 0 moved")
	var e2 := int(b.vault_label_entity_id(0).get("entity_id", 0))
	_ok(e2 != e1 and b.vault_links_for("label", e2).size() == 1 and b.vault_links_for("label", e1).is_empty(), "UB the note followed the label")
	await _undo_ctl()
	_ok(b.vault_links_for("label", e1).size() == 1 and b.vault_links_for("label", e2).is_empty(), "UB undo moved the note back with the label")
	await _redo_ctl()
	_ok(b.vault_links_for("label", e2).size() == 1, "UB redo moves it forward again")
	await _undo_ctl()
	# back to a clean slate for the legs after
	b.label_clear_all()
	b.icon_clear_all()
	b.landmark_set_user_name(key, "")
	b.clear_undo()
	await _frames(3)
	_ok(base["labels"] == b.label_list() and base["icons"] == b.icon_list(), "UB the stores are back to their starting content")


# ---- UI ------------------------------------------------------------------------------------------------------------

func _leg_controls() -> void:
	print("== UI the controls of this form ==")
	var b = _bridge
	await _domain("world")
	var ws = _carto()
	var i0: int = ws._create_label(50.0, 50.0, "Control")
	_ok(i0 == 0, "UI a label to undo")
	await _frames(4)
	# the toolbar squares and the phone chip show state
	if DccTheme.is_phone():
		var chip: Button = _app.get("_phone_undo_chip")
		_ok(chip != null and chip.visible, "UI phone: the undo chip is shown for a label edit (no height step exists)")
		await _undo_ctl()
		_ok(b.label_list().is_empty(), "UI phone: a real tap on the chip undid the label")
		_ok(not chip.visible, "UI phone: the chip hides once nothing is left to undo")
		await _redo_ctl()
		_ok(b.label_list().size() == 1, "UI phone: Edit > Redo (redo_last) brings it back")
		_ok(chip.visible, "UI phone: the chip is back")
		# two taps well inside the chip's 0.52 s hold time: both are taps (a stale hold timer used to turn the second into a hold)
		ws._create_label(52.0, 52.0, "Second")
		await _frames(4)
		_ok(b.label_list().size() == 2, "UI phone: two labels, two steps to undo")
		var cc := chip.get_global_rect().get_center()
		await _tap(cc)
		# The second press starts about 0.25 s after the first and is held 0.32 s: the FIRST press's hold timer (0.52 s after
		# its own press) fires while this finger is still down, and must not call it a hold. Still shorter than its own hold.
		_finger(cc, true)
		await get_tree().create_timer(0.32).timeout
		_finger(cc, false)
		await _frames(6)
		_ok(b.label_list().is_empty(), "UI phone: two quick taps undo two steps (a stale hold timer used to eat the second)")
		_ok(_app.get("_phone_undo_pop") == null or not (_app.get("_phone_undo_pop") as Control).visible, "UI phone: and the history popover did not open")
		_app.redo_last()
		_app.redo_last()
		await _frames(4)
		_ok(b.label_list().size() == 2, "UI phone: two redos bring both back")
	else:
		var sq: Button = _app.get("_menu_undo_btn")
		_ok(sq != null and not sq.disabled, "UI the toolbar Undo square is enabled for a label edit")
		var rsq: Button = _app.get("_menu_redo_btn")
		await _click_at(sq.get_global_rect().get_center())
		_ok(b.label_list().is_empty(), "UI a real click on the Undo square undid the label")
		_ok(sq.disabled, "UI the Undo square disables once nothing is left")
		_ok(rsq != null and not rsq.disabled, "UI the Redo square lights")
		await _click_at(rsq.get_global_rect().get_center())
		_ok(b.label_list().size() == 1, "UI a real click on the Redo square brought it back")
		# the Edit menu row names the edit
		var edit: PopupMenu = _menu_named("Edit")
		_ok(edit != null, "UI the Edit menu is found")
		if edit != null:
			edit.about_to_popup.emit()
			var undo_row := _row_text(edit, "Undo")
			_ok(undo_row == "Undo Add label “Control”", "UI Edit > Undo reads 'Undo Add label “Control”' (%s)" % undo_row)
			edit.id_pressed.emit(DccMenus.ID_UNDO)
			await _frames(4)
			_ok(b.label_list().is_empty(), "UI Edit > Undo took the step")
			edit.about_to_popup.emit()
			_ok(_row_text(edit, "Redo") == "Redo Add label “Control”", "UI Edit > Redo reads 'Redo Add label “Control”' (%s)" % _row_text(edit, "Redo"))
			edit.id_pressed.emit(DccMenus.ID_REDO)
			await _frames(4)
			_ok(b.label_list().size() == 1, "UI Edit > Redo took it back")
		# Ctrl+Shift+Z is redo too
		await _key(KEY_Z, true)
		_ok(b.label_list().is_empty(), "UI Ctrl+Z undid it")
		await _key(KEY_Z, true, true)
		_ok(b.label_list().size() == 1, "UI Ctrl+Shift+Z redid it")
	# the radial ring's Undo row (a context ring on touch) offers the step too
	var ring := GlobalTools.ring_cardinals(_app, {"finalized": false, "armed_tool": "inspect"})
	var s_row: Dictionary = ring.get("S", {})
	var ring_want := "Undo add label “second”" if DccTheme.is_phone() else "Undo add label “control”"
	_ok(bool(s_row.get("enabled", false)) and String(s_row.get("label", "")) == ring_want, "UI the ring's Undo row is enabled and names the edit (%s)" % str(s_row))
	b.label_clear_all()
	b.clear_undo()
	await _frames(3)


func _menu_named(title: String) -> PopupMenu:
	var burger: PopupMenu = null
	for ch in _app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == title:
			return (ch as MenuButton).get_popup()
		if ch is MenuButton and (ch as MenuButton).text == "☰":
			burger = (ch as MenuButton).get_popup()
	if burger != null:
		for i in burger.item_count:
			if not burger.is_item_separator(i) and burger.get_item_text(i) == title and burger.get_item_submenu(i) != "":
				return burger.get_node_or_null(NodePath(burger.get_item_submenu(i))) as PopupMenu
	return null


## The text of the first item of `p` that starts with `prefix`.
func _row_text(p: PopupMenu, prefix: String) -> String:
	for i in p.item_count:
		if p.get_item_text(i).begins_with(prefix):
			return p.get_item_text(i)
	return ""


# ---- UT ------------------------------------------------------------------------------------------------------------

## Two land cells `span` apart on one row, for a road drag; an empty list when none.
func _land_pair(span: int) -> Array:
	for y in range(12, 192 - 12, 4):
		for x in range(12, 256 - 12 - span, 4):
			var ok := true
			for k in [0, span / 2, span]:
				if String(_bridge.world_gen.sample_cell(x + k, y).get("water", "")) != "land":
					ok = false
			if ok:
				return [Vector2(x, y), Vector2(x + span, y)]
	return []


func _leg_interleave() -> void:
	print("== UT interleaving ==")
	var b = _bridge
	await _domain("cartography")
	var ws = _carto()
	var look_before: String = str(LookHistory.snapshot(b, ["biomes"]))
	# annotation, height, annotation, way, annotation
	var ia: int = ws._create_label(70.0, 70.0, "First")
	var d_field0 := String(b.edit_state_digest().get("field", ""))
	_ok(bool(b.carve_fjords().get("ok", false)), "UT a height step (carve fjords)")
	await _frames(2)
	_ok(b.label_move(0, 80.0, 80.0), "UT label moved after the carve")
	var way_ok := false
	var pr := _land_pair(10)
	if not pr.is_empty():
		way_ok = b.way_stroke_begin({"way_type": "road"})
		for k in 11:
			var p: Vector2 = (pr[0] as Vector2).lerp(pr[1], k / 10.0)
			b.way_stroke_sample(p.x, p.y, k * 16)
		way_ok = bool(b.way_stroke_end().get("committed", false)) and way_ok
	_ok(way_ok, "UT a way drag committed")
	_ok(b.icon_arm("feature", 0, 1.0, 0.0, 0.0), "UT an icon armed")
	var ii: int = b.icon_place(100.0, 100.0)
	b.icon_disarm()
	_ok(ii == 0, "UT an icon placed after the way")
	var ledger_kinds: Array = []
	for r in b.undo_ledger():
		if bool((r as Dictionary).get("reversible", false)):
			ledger_kinds.append(String((r as Dictionary)["kind"]))
	_ok(ledger_kinds == ["annotation", "height", "annotation", "way", "annotation"], "UT the ledger holds the five steps in commit order (%s)" % str(ledger_kinds))
	_ok(b.undo_next_subsystem() == "annotation", "UT the next step is the icon placement")
	await _undo_ctl()
	_ok(b.icon_list().is_empty(), "UT 1: the icon placement is undone first")
	_ok(b.undo_next_subsystem() == "civ", "UT then the next step is the way drag (%s)" % b.undo_next_subsystem())
	await _undo_ctl()
	_ok(b.undo_next_subsystem() == "annotation", "UT 2: the way drag is undone; the label move is next")
	await _undo_ctl()
	_ok(is_equal_approx(float(b.label_get(0)["x"]), 70.0), "UT 3: the label move is undone (x %s)" % str(b.label_get(0)["x"]))
	_ok(b.undo_next_subsystem() == "height", "UT the carve is next")
	await _undo_ctl()
	_ok(String(b.edit_state_digest().get("field", "")) == d_field0, "UT 4: the carve is undone, the field is as before")
	await _undo_ctl()
	_ok(b.label_list().is_empty(), "UT 5: the first label add is undone last")
	# redo walks back in the same order
	await _redo_ctl()
	_ok(b.label_list().size() == 1, "UT redo 1: the label add")
	await _redo_ctl()
	_ok(String(b.edit_state_digest().get("field", "")) != d_field0, "UT redo 2: the carve")
	await _redo_ctl()
	_ok(is_equal_approx(float(b.label_get(0)["x"]), 80.0), "UT redo 3: the label move")
	await _redo_ctl()
	await _redo_ctl()
	_ok(b.icon_list().size() == 1 and not b.redo_available(), "UT redo 4 and 5: the way and the icon; nothing is left to redo")
	# a new edit truncates the redo tail
	await _undo_ctl()
	_ok(b.redo_available() and b.redo_label().begins_with("Place icon"), "UT after one undo, redo offers the placement (%s)" % b.redo_label())
	_ok(b.label_move(0, 85.0, 85.0), "UT a new edit")
	_ok(not b.redo_available(), "UT a new edit drops the redo")
	# the colour-paint stack is independent
	var t = ws.colour_paint
	var kpc := float(b.colour_paint_info().get("km_per_cell", 0.0))
	var p0: String = String(b.edit_state_digest().get("colour_paint", ""))
	_ok(t.commit_line([Vector2(256 * 0.3, 192 * 0.5) * kpc, Vector2(256 * 0.6, 192 * 0.5) * kpc]), "UT a colour-paint stroke")
	var lbl_now: Dictionary = b.label_get(0)
	await _undo_ctl()
	_ok(String(b.edit_state_digest().get("colour_paint", "")) != p0, "UT the plain undo does not touch the paint stroke")
	_ok(is_equal_approx(float(b.label_get(0)["x"]), 80.0), "UT it took the label edit (x %s)" % str(b.label_get(0)["x"]))
	t.undo()
	await _frames(2)
	_ok(String(b.edit_state_digest().get("colour_paint", "")) == p0, "UT the paint tool's own undo takes the stroke")
	_ok(b.label_get(0) != lbl_now, "UT and leaves the annotation stack alone")
	# a look edit and an annotation edit interleave newest first
	var before: Dictionary = LookHistory.snapshot(b, ["biomes"])
	b.set_biome_color(3, Color8(200, 40, 160))
	_app.look_history.commit("Biome colour", LookHistory.snapshot(b, ["biomes"]), b, before)
	_ok(b.label_move(0, 90.0, 90.0), "UT a label move after the look edit")
	await _undo_ctl()
	_ok(is_equal_approx(float(b.label_get(0)["x"]), 80.0) and _app.look_history.can_undo(), "UT the label move is undone first; the look edit is still undoable")
	await _undo_ctl()
	_ok(not _app.look_history.can_undo(), "UT then the look edit")
	# the History ledger row reverts to it
	_ok(b.label_move(0, 11.0, 11.0), "UT a label step for the ledger")
	_ok(b.label_move(0, 12.0, 12.0), "UT another")
	var rows: Array = b.undo_ledger()
	var target := -1
	for r in rows:
		var d: Dictionary = r
		if String(d.get("kind", "")) == "annotation" and bool(d.get("reversible", false)) and String(d.get("label", "")).begins_with("Move label"):
			target = int(d["seq"])
			break
	_ok(target > 0, "UT the ledger offers annotation rows to revert to (seq %d)" % target)
	var steps_before: int = b.undo_ledger().filter(func(r): return bool((r as Dictionary).get("reversible", false))).size()
	var done: int = b.undo_revert_to(target)
	_ok(done >= 2, "UT reverting to an annotation row takes every step above it too (%d)" % done)
	_ok(b.undo_ledger().filter(func(r): return bool((r as Dictionary).get("reversible", false))).size() == steps_before - done, "UT and the rows go with them")
	b.label_clear_all()
	b.icon_clear_all()
	b.clear_undo()
	await _frames(3)


# ---- UD ------------------------------------------------------------------------------------------------------------

## A pointer drag from `a` to `b` in `n` samples: real mouse events on desktop, a real touch on a touch form.
func _drag(a: Vector2, b: Vector2, n: int) -> void:
	a -= _off
	b -= _off
	if _touch:
		_finger(a, true)
		await _frames(3)
		for k in range(1, n + 1):
			var p := a.lerp(b, float(k) / float(n))
			var t := InputEventScreenDrag.new()
			t.index = 0
			t.position = p
			_vp.push_input(t)
			var m := InputEventMouseMotion.new()
			m.device = InputEvent.DEVICE_ID_EMULATION
			m.position = p
			m.global_position = p
			m.button_mask = MOUSE_BUTTON_MASK_LEFT
			_vp.push_input(m)
			await _frames(1)
		_finger(b, false)
		await _frames(6)
	else:
		await _move_to(a)
		_mouse(a, MOUSE_BUTTON_LEFT, true)
		await _frames(2)
		for k in range(1, n + 1):
			var p := a.lerp(b, float(k) / float(n))
			var mv := InputEventMouseMotion.new()
			mv.position = p
			mv.global_position = p
			mv.button_mask = MOUSE_BUTTON_MASK_LEFT
			_vp.push_input(mv)
			await _frames(1)
		_mouse(b, MOUSE_BUTTON_LEFT, false)
		await _frames(6)


## A host position where the Label tool's own hit test (`label_pick_all`, the engine's box around the label's anchor with
## the live px-per-cell and zoom) names label `li`, scanned in half-cell steps around its anchor; (-1, -1) when none.
func _tool_hit_pos(li: int) -> Vector2:
	var lb: Dictionary = _bridge.label_get(li)
	var ppc: float = _app.viewport.label_px_per_cell()
	var z: float = _app.viewport.zoom()
	for ring in range(0, 13):
		for dy in range(-ring, ring + 1):
			for dx in range(-ring, ring + 1):
				if maxi(absi(dx), absi(dy)) != ring:
					continue
				var cell := Vector2(float(lb["x"]) + 0.5 * dx, float(lb["y"]) + 0.5 * dy)
				var hits: PackedInt64Array = _bridge.label_pick_all(cell.x, cell.y, ppc, z)
				if hits.size() > 0 and int(hits[0]) == li:
					var hp := _pos_of_cell(cell)
					if hp.x >= 0.0:
						return hp
	return Vector2(-1, -1)


func _leg_gestures() -> void:
	print("== UD real gestures ==")
	var b = _bridge
	await _domain("cartography")
	await _centre(Vector2(128, 96))
	# No calibration: `_calibrate` reads a motion event the overlay never receives while a tool is not yet armed and answers
	# (-150, -150), which shifts every press off the label; the host fills the viewport, so the offset is zero.
	_off = Vector2.ZERO
	var ws = _carto()
	_app.arm_tool("label")
	await _frames(4)
	# a label created through the tool's prompt: ONE step
	var spot := _empty_spot(Vector2(-1, -1), 0.0)
	_ok(spot.x >= 0.0, "UD an empty spot for a label")
	if spot.x < 0.0:
		return
	var n0: int = b.label_list().size()
	await _map_click(spot)
	_ok(await _answer_accept("Gamma"), "UD the Label tool's prompt appeared and took a name")
	_ok(b.label_list().size() == n0 + 1, "UD the label was created")
	var depth: int = int(b.undo_stats().get("annotation_depth", 0))
	_ok(depth == 1, "UD creating a label through the tool is ONE undo step (not a create, a reclass and a style set): depth %d" % depth)
	_ok(String(b.undo_label()) == "Add label “Gamma”", "UD the row reads Add label “Gamma” (%s)" % b.undo_label())
	# drag it: many samples, one step
	var li: int = b.label_list().size() - 1
	# A selected label shows its three handles, and a press on one grabs it (resize, rotate or arc) instead of moving the
	# label; deselected, the press selects the label and starts a MOVE drag.
	b.label_select(-1)
	await _frames(3)
	await _centre(Vector2(float(b.label_get(li)["x"]), float(b.label_get(li)["y"])))
	# The Label tool's own hit test (the engine's box around the label's anchor), not the overlay's pick: press on the anchor.
	var pa: Vector2 = _tool_hit_pos(li)
	_ok(pa.x >= 0.0, "UD a point the Label tool's own hit test resolves to the new label (the tool misses what the overlay can pick)")
	if pa.x < 0.0:
		return
	var x0 := float(b.label_get(li)["x"])
	var y0 := float(b.label_get(li)["y"])
	var dest := _empty_spot(pa, 90.0)
	_ok(dest.x >= 0.0, "UD a drag destination exists")
	var depth_before := int(b.undo_stats().get("annotation_depth", 0))
	var x1 := x0
	if _touch:
		# On a touch form a one-finger drag with a tap-or-place tool is a camera pan (MS-5, `viewport_host.gd::_touch_motion`),
		# so the label is moved by the card's Move: arm it, tap the destination. A drag must record NO step.
		await _drag(pa, dest, 10)
		_ok(int(b.undo_stats().get("annotation_depth", 0)) == depth_before and is_equal_approx(float(b.label_get(li)["x"]), x0), "UD touch: a one-finger drag pans the camera and records no label step (depth %d)" % int(b.undo_stats().get("annotation_depth", 0)))
		_app.arm_tool("inspect")
		GlobalTools._begin_move(_app, "label", li, "Gamma")
		await _frames(3)
		await _centre(Vector2(float(b.label_get(li)["x"]), float(b.label_get(li)["y"])))
		dest = _empty_spot(Vector2(-1, -1), 0.0)
		var cd := _cell_of(dest)
		await _map_click(dest)
		x1 = float(b.label_get(li)["x"])
		_ok(absf(x1 - cd.x) < 1.0 and absf(x1 - x0) > 1.0, "UD touch: the Move destination tap moved the label (x %s -> %s)" % [str(x0), str(x1)])
	else:
		await _drag(pa, dest, 10)
		x1 = float(b.label_get(li)["x"])
		_ok(absf(x1 - x0) > 1.0, "UD the real drag moved the label (x %s -> %s)" % [str(x0), str(x1)])
	_ok(int(b.undo_stats().get("annotation_depth", 0)) == depth_before + 1, "UD the move is ONE undo step however many samples the drag had (depth %d -> %d)" % [depth_before, int(b.undo_stats().get("annotation_depth", 0))])
	_ok(String(b.undo_label()) == "Move label “Gamma”", "UD the row reads Move label “Gamma” (%s)" % b.undo_label())
	await _undo_ctl()
	_ok(is_equal_approx(float(b.label_get(li)["x"]), x0) and is_equal_approx(float(b.label_get(li)["y"]), y0), "UD ONE undo puts the label back at its start")
	await _redo_ctl()
	_ok(is_equal_approx(float(b.label_get(li)["x"]), x1), "UD ONE redo puts it at the move's end")
	if not _touch:
		# a second drag is a second step (the stream is closed by the release)
		await _centre(Vector2(float(b.label_get(li)["x"]), float(b.label_get(li)["y"])))
		var pb: Vector2 = _tool_hit_pos(li)
		var depth2 := int(b.undo_stats().get("annotation_depth", 0))
		_ok(pb.x >= 0.0, "UD the moved label can be grabbed again")
		if pb.x >= 0.0:
			await _drag(pb, pb + Vector2(0.0, 40.0), 6)
			_ok(int(b.undo_stats().get("annotation_depth", 0)) == depth2 + 1, "UD a second drag is its own step, not merged into the first (depth %d -> %d)" % [depth2, int(b.undo_stats().get("annotation_depth", 0))])
	# typing into the label's text field: a burst of keystrokes is ONE step, a pause starts the next
	var depth3 := int(b.undo_stats().get("annotation_depth", 0))
	for t in ["Ga", "Gam", "Gamm", "Gamma 2"]:
		ws._apply_label_field(li, "text", t)
		await _frames(2)
	_ok(int(b.undo_stats().get("annotation_depth", 0)) == depth3 + 1, "UD four keystrokes inside the idle window are ONE step (depth %d -> %d)" % [depth3, int(b.undo_stats().get("annotation_depth", 0))])
	await get_tree().create_timer(0.9).timeout
	ws._apply_label_field(li, "text", "Gamma 3")
	await _frames(2)
	_ok(int(b.undo_stats().get("annotation_depth", 0)) == depth3 + 2, "UD a keystroke after a 0.9 s pause is a new step (depth %d)" % int(b.undo_stats().get("annotation_depth", 0)))
	_ok(String(b.undo_label()).begins_with("Rename label"), "UD the row reads Rename label… (%s)" % b.undo_label())
	await _undo_ctl()
	_ok(String(b.label_get(li)["text"]) == "Gamma 2", "UD undoing the second burst leaves the first burst's text (%s)" % str(b.label_get(li)["text"]))
	await _undo_ctl()
	_ok(String(b.label_get(li)["text"]) != "Gamma 2" and String(b.label_get(li)["text"]) != "Gamma 3", "UD undoing the first burst returns the text from before it (%s)" % str(b.label_get(li)["text"]))
	_app.arm_tool("inspect")
	b.label_clear_all()
	b.clear_undo()
	await _frames(3)


# ---- UG ------------------------------------------------------------------------------------------------------------

func _leg_group() -> void:
	print("== UG the assistant's grouped undo ==")
	var b = _bridge
	var g = _app.undo_groups
	# `clear_undo()` (every earlier leg) records a new floor row; the group then sees "a new world" at the first
	# `world_loaded` (a carve emits one) and dissolves itself, so it is told the world is the one it will run on.
	g.on_world_changed()
	var d0: Dictionary = b.edit_state_digest().duplicate()
	for k in ["labels", "icons", "landmark_names"]:
		_ok(d0.has(k), "UG the digest carries '%s'" % k)
	_ok(g.begin("Assistant: tidy the map") == "", "UG a group opened")
	_ok(bool(b.carve_fjords().get("ok", false)), "UG height member")
	var ws = _carto()
	_ok(ws._create_label(61.0, 61.0, "Grouped") >= 0, "UG label member (one compound step)")
	_ok(b.icon_arm("feature", 0, 1.0, 0.0, 0.0), "UG icon armed")
	_ok(b.icon_place(70.0, 70.0) >= 0, "UG icon member")
	b.icon_disarm()
	var key := String((b.landmark_names()[0] as Dictionary)["key"])
	_ok(b.landmark_set_user_name(key, "Grouped Name"), "UG landmark-name member")
	var c: Dictionary = g.commit()
	_ok(bool(c.get("ok", false)) and int(c.get("members", 0)) == 4, "UG commit: ok with four members (%s)" % str(c))
	var d1: Dictionary = b.edit_state_digest().duplicate()
	for part in ["field", "labels", "icons", "landmark_names"]:
		_ok(String(d0[part]) != String(d1[part]), "UG positive control: the group moved '%s'" % part)
	_ok(_app.undo_target_label() == "Assistant: tidy the map", "UG Edit > Undo names the group (%s)" % _app.undo_target_label())
	await _undo_ctl()
	var du: Dictionary = b.edit_state_digest().duplicate()
	_ok(du == d0, "UG ONE undo restores the whole digest, annotation stores included")
	_ok(b.label_list().is_empty() and b.icon_list().is_empty(), "UG no label or icon is left")
	_ok(_app.redo_target_label() == "Assistant: tidy the map", "UG Edit > Redo names the group (%s)" % _app.redo_target_label())
	await _redo_ctl()
	_ok(b.edit_state_digest().duplicate() == d1, "UG ONE redo restores the post-group digest")
	# an annotation edit on top blocks a member being taken alone
	_ok(b.label_move(0, 62.0, 62.0), "UG an ungrouped label move on top of the group")
	await _undo_ctl()
	_ok(float(b.label_get(0)["x"]) == 61.0 and b.edit_state_digest().duplicate() == d1, "UG the plain undo took the ungrouped move alone")
	await _undo_ctl()
	_ok(b.edit_state_digest().duplicate() == d0, "UG then the group, whole")
	b.clear_undo()
	await _frames(3)


# ---- UH ------------------------------------------------------------------------------------------------------------

func _leg_selection() -> void:
	print("== UH selection hazards ==")
	var b = _bridge
	var ws = _carto()
	for k in 3:
		ws._create_label(40.0 + 10.0 * k, 40.0, "S%d" % k)
	b.clear_undo()
	_app.select("label", 2)
	_app.select("label", 1)
	_ok(String(_app.selection.get("kind", "")) == "label" and int(_app.selection["id"]) == 1, "UH label 1 is selected (the back history holds label 2)")
	_ok(b.label_delete(0), "UH label 0 deleted: every later index shifts")
	_ok(_app.selection.is_empty(), "UH the selection of that kind is dropped, not left on the neighbour")
	var named: Array = _app._selection_back.filter(func(e): return String((e as Dictionary).get("kind", "")) == "label")
	_ok(named.is_empty(), "UH and no Alt+Left entry still names a label by its old index (%d left)" % named.size())
	_ok(not _app.select_back() or _app.selection.is_empty() or String(_app.selection.get("kind", "")) != "label", "UH Alt+Left does not step back to a label")
	# undoing the delete shifts again
	_app.select("label", 1)
	await _undo_ctl()
	_ok(b.label_list().size() == 3, "UH the delete is undone")
	_ok(_app.selection.is_empty(), "UH the undo of a delete (rows shift) drops the selection of that kind")
	# an undo that keeps every index keeps the selection
	_ok(b.label_move(1, 50.0, 60.0), "UH a move")
	_app.select("label", 1)
	await _undo_ctl()
	_ok(String(_app.selection.get("kind", "")) == "label" and int(_app.selection["id"]) == 1, "UH undoing a move keeps the selection (no index moved)")
	# icons: placing after a pass appends, never renumbers; clearing and undoing does
	_ok(b.icon_arm("feature", 0, 1.0, 0.0, 0.0), "UH an icon armed")
	b.icon_place(10.0, 10.0)
	b.icon_place(20.0, 10.0)
	b.icon_disarm()
	_app.select("icon", 1)
	var res: Dictionary = b.icon_generate({"family": "PLACES", "scale": 1.0})
	_ok(String(_app.selection.get("kind", "")) == "icon" and int(_app.selection["id"]) == 1 and String(b.icon_get(1).get("slot", "")) != "", "UH a placement pass appends: the selected icon still names itself (placed %s)" % str(res.get("placed", "?")))
	b.label_clear_all()
	b.icon_clear_all()
	b.clear_undo()
	await _frames(3)


# ---- US ------------------------------------------------------------------------------------------------------------

func _leg_save_load() -> void:
	print("== US save and load ==")
	var b = _bridge
	var ws = _carto()
	var key := String((b.landmark_names()[0] as Dictionary)["key"])
	ws._create_label(55.0, 55.0, "Saved")
	b.landmark_set_user_name(key, "Saved Name")
	_ok(b.can_undo(), "US there is history before the save")
	var path := OS.get_user_data_dir().path_join("announdo.cartalith")
	_ok(b.save_project(path), "US saved")
	_ok(b.load_save(path), "US reopened")
	await _frames(8)
	_ok(b.label_list().size() == 1 and String(b.label_get(0)["text"]) == "Saved", "US the label survived the reopen")
	var found := false
	for n in b.landmark_names():
		if String((n as Dictionary)["key"]) == key:
			found = String((n as Dictionary)["name"]) == "Saved Name"
	_ok(found, "US the landmark name survived the reopen")
	_ok(not b.can_undo(), "US the undo history did not: it is session-only, like the height and way stacks")
	_ok(not _app.can_undo_any(), "US and no control offers an undo after the reopen")
	_ok(int(b.undo_stats().get("annotation_depth", -1)) == 0, "US the held annotation steps are gone (depth 0)")
	# an edit after the reopen undoes normally
	_ok(b.label_move(0, 20.0, 20.0), "US a move after the reopen")
	await _undo_ctl()
	_ok(is_equal_approx(float(b.label_get(0)["x"]), 55.0), "US it undoes (x %s)" % str(b.label_get(0)["x"]))
