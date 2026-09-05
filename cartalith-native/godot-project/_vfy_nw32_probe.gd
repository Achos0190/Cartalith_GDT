extends Node
## VERIFIER probe for batch 32 lane B (the New world dialog). Two questions the
## lane's own report cannot answer for itself: is the phone form a DIFFERENT
## form or the desktop one in a smaller box, and does Create still create.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _vfy_nw32_probe.tscn
##   ... -- --force-touch                      (tablet leg)

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ck(tag: String, name: String, cond: bool, detail: String = "") -> void:
	print("VNW %s  [%s] %s%s" % ["ok  " if cond else "FAIL", tag, name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _shown(n: Node) -> bool:
	var c: Node = n
	while c != null:
		if c is Window:
			return true
		if c is CanvasItem and not (c as CanvasItem).visible:
			return false
		c = c.get_parent()
	return true

func _boot(w: int, h: int) -> Node:
	var vp := SubViewport.new()
	vp.size = Vector2i(w, h)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(60)
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(5)
	return app

## Every Label text reachable (visible chain) inside the dialog.
func _labels(root: Node, out: Array) -> void:
	if root is CanvasItem and not (root as CanvasItem).visible:
		return
	if root is Label:
		out.append((root as Label).text)
	for c in root.get_children():
		_labels(c, out)

func _run(app: Node, tag: String) -> void:
	var d: NewWorldDialog = app.new_world_dialog
	var phone: bool = DccTheme.is_phone()
	print("VNW -- %s  is_phone=%s is_touch=%s" % [tag, phone, DccTheme.is_touch()])
	app.open_new_world()
	await _frames(20)

	var card: PanelContainer = d.get("_card")
	var chips: Array = d.get("_extent_chips")
	_ck(tag, "card exists iff phone", (card != null) == phone, "card=%s" % (card != null))
	_ck(tag, "extent chips exist iff phone", (chips.size() == 2) == phone,
		"chips=%d" % chips.size())

	## The load-bearing question: on a phone the *desktop* controls must be off
	## the screen, and on desktop they must all be on it.
	var desktop_only := {
		"extent dropdown": d.extent_input,
		"map-width preset": d.size_preset_input,
		"grid columns": d.grid_w_input,
		"grid rows": d.grid_h_input,
		"archetype": d.archetype_input,
		"villages toggle": d.villages_check,
	}
	for k in desktop_only:
		var c: Control = desktop_only[k]
		_ck(tag, "%s reachable == not phone" % k, _shown(c) == (not phone),
			"shown=%s" % _shown(c))
	## ...while the three the card DOES draw stay reachable in both.
	_ck(tag, "seed reachable", _shown(d.seed_input))
	_ck(tag, "resolution reachable", _shown(d.resolution_input))
	## It hides itself when there is nothing to warn about (`visible = not
	## warnings.is_empty()`), so the assertion is on its CONTAINER, and the
	## label's own reachability is asserted below with a warning switched on.
	_ck(tag, "the dimension-warning row is on the visible form",
		_shown(d.dimension_warning_label.get_parent()))
	## 8K, in BOTH compositions: on a phone this is the check that the memory
	## cost is disclosed on the card rather than in the hidden half.
	var was_r := d.resolution_input.selected
	d.resolution_input.selected = 4
	d.resolution_input.item_selected.emit(4)
	await _frames(3)
	_ck(tag, "the 8K warning is reachable where the choice is made",
		d.dimension_warning_label.text != "" and _shown(d.dimension_warning_label),
		"text='%s'" % d.dimension_warning_label.text.substr(0, 44))
	d.resolution_input.selected = was_r
	d.resolution_input.item_selected.emit(was_r)
	await _frames(3)

	if phone:
		_ck(tag, "card width is 360 at 412 dp",
			is_equal_approx(card.custom_minimum_size.x, 360.0),
			"got=%.1f phone_scale=%.3f" % [card.custom_minimum_size.x, DccTheme._phone_scale])
		var dice: Button = null
		for c in (d.seed_input.get_parent() as Node).get_children():
			if c is Button and (c as Button).accessibility_name == "New seed":
				dice = c
		_ck(tag, "the dice exists, is named and is 44x44", dice != null \
			and dice.custom_minimum_size == Vector2(44, 44), "dice=%s" % dice)
		var before := int(d.seed_input.value)
		if dice != null:
			dice.pressed.emit()
			await _frames(3)
		_ck(tag, "the dice actually rerolls the seed", int(d.seed_input.value) != before,
			"%d -> %d" % [before, int(d.seed_input.value)])
		## Chips drive the hidden dropdown, and the dropdown is the one source.
		(chips[1] as Button).pressed.emit()
		await _frames(3)
		_ck(tag, "WORLD chip -> dropdown selected=1", d.extent_input.selected == 1,
			"selected=%d name='%s'" % [d.extent_input.selected, (chips[1] as Button).accessibility_name])
		(chips[0] as Button).pressed.emit()
		await _frames(3)
		_ck(tag, "REGION chip -> dropdown selected=0", d.extent_input.selected == 0,
			"selected=%d name='%s'" % [d.extent_input.selected, (chips[0] as Button).accessibility_name])

	# ---- Create still creates -------------------------------------------------
	## Smallest resolution, so the run is short. `request()` reads three hidden
	## controls on a phone; this is the assertion that they are still there.
	d.resolution_input.selected = 0
	d.resolution_input.item_selected.emit(0)
	await _frames(4)
	var req: Dictionary = d.request()
	print("VNW   [%s] request()=%s" % [tag, str(req)])
	_ck(tag, "request() carries a usable grid",
		int(req.get("grid_w", 0)) > 0 and int(req.get("grid_h", 0)) > 0,
		"gw=%s gh=%s" % [req.get("grid_w"), req.get("grid_h")])
	_ck(tag, "request() carries no 'name' (nothing stores one)", not req.has("name"))
	var want_w := int(req["grid_w"])
	var want_h := int(req["grid_h"])
	d.confirmed.emit()
	await _frames(6)
	var waited := 0
	while app.bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	await _frames(6)
	var gs: Vector2i = Vector2i(app.bridge.grid_size())
	_ck(tag, "Create created, at the requested grid",
		app.bridge.has_world and gs == Vector2i(want_w, want_h),
		"has_world=%s grid=%s want=%s (%d frames)" % [app.bridge.has_world, str(gs),
			str(Vector2i(want_w, want_h)), waited])

func _ready() -> void:
	var a := await _boot(1920, 1080)
	await _run(a, "DESKTOP 1920x1080")
	a.queue_free()
	await _frames(10)
	if DccTheme.is_touch():
		var p := await _boot(1080, 2340)
		await _run(p, "PHONE 1080x2340")
	print("VNW DONE  failures=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
