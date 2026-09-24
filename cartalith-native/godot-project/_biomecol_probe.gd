extends Node
## **CA-19 / Ruling P: CARTO ▸ Colours ▸ Biome colours edits the biome table,
## and every reader of that table follows the edit.**
##
## Driven through the real picker (`RenderWorkspace._biome_swatches`, the
## `ColorPickerButton`s CARTO's Colours category draws): set a colour, emit the
## picker's own `color_changed` + `popup_closed`, then read back
##   1. the map raster at a cell PAINTED with that class (the 0.60 paint blend);
##   2. the Biomes field's legend swatch (`debug_layers()` `bclass`);
##   3. the Biomes field raster at a cell OF that class (`debug_texture("bclass")`);
##   4. the paint preview of a pending dab of that class;
## then press the row's Reset and Reset all and assert every one comes back.
##
## **Positive control**: committing the paint dab must move the map pixel under
## it (so the readback is live, not the headless `ImageTexture` no-op) while a
## far, unpainted pixel must not. **Negative control**: the far pixel must not
## move on the colour edit either.
##
## The cell is chosen by its class in the frozen Biomes raster read BEFORE any
## edit (an input), never by what moved.
##
## Pixel readback, so WINDOWED, never `--headless`:
##   godot --path . _biomecol_probe.tscn -- --tag desktop
##   godot --path . _biomecol_probe.tscn -- --force-touch --tag phone
##
## Flags this probe reads, grepped from the body below:
##   `--vp WxH`      SubViewport size in physical px. Default 1600x1000.
##   `--tag NAME`    prefix on every output line. Default `bcol`.
##   `--force-touch` NOT read here -- `dcc_shell.gd` reads it; with a phone-sized
##                   `--vp` (1080x2340) it makes the run a phone.
## Any other `--flag` aborts rather than being silently ignored.
##
## No theme palette is assumed: every pixel assertion compares engine rasters
## against each other or against a colour this probe set, never against a
## threshold written for one palette.

var app: Node
var bridge: EngineBridge
var _vp: SubViewport
var _tag := "bcol"
var _fail := 0

const EDIT := Color8(255, 0, 255)

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

func _reject_unknown_args() -> bool:
	var known := ["--force-touch", "--vp", "--tag"]
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true

func _finish() -> void:
	_log("RESULT %s fail=%d" % [_tag, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

func _px(tex: Texture2D, c: Vector2i) -> Color:
	if tex == null:
		return Color(-1, -1, -1, -1)
	var img := tex.get_image()
	if img == null or img.is_empty():
		return Color(-1, -1, -1, -1)
	if img.is_compressed():
		img.decompress()
	return img.get_pixelv(c)

func _c8(c: Color) -> String:
	return "(%d,%d,%d,%d)" % [c.r8, c.g8, c.b8, c.a8]

func _same8(a: Color, b: Color) -> bool:
	return a.r8 == b.r8 and a.g8 == b.g8 and a.b8 == b.b8

func _legend(k: int) -> Color:
	for g in bridge.debug_layers():
		for it in (g as Dictionary).get("items", []):
			if String((it as Dictionary).get("id", "")) == "bclass":
				var rows: Array = (it as Dictionary).get("legend", [])
				if k - 1 < rows.size():
					var r: Dictionary = rows[k - 1]
					return Color8(int(r["r"]), int(r["g"]), int(r["b"]))
	return Color(-1, -1, -1, -1)

func _all(root: Node, out: Array) -> void:
	for c in root.get_children():
		out.append(c)
		_all(c, out)

func _cartography() -> Node:
	for ws in app._workspaces:
		if ws is CartographyWorkspace:
			return ws
	return null

func _ready() -> void:
	_tag = _arg("--tag", "bcol")
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		_log("ABORT run WINDOWED -- ImageTexture readback is a no-op under --headless")
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	if parts.size() != 2:
		_log("ABORT --vp wants WxH")
		get_tree().quit(2)
		return
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	bridge = app.bridge
	var phone: bool = app.is_phone()
	_log("viewport %dx%d  phone=%s" % [_vp.size.x, _vp.size.y, phone])
	_check(bridge.biome_colors_api, "the engine has the four biome colour bindings")

	var carto = _cartography()
	var rw: RenderWorkspace = carto._render if carto != null else null
	_check(rw != null and rw._biome_swatches.size() == 15,
		"CARTO ▸ Colours drew 15 biome pickers (%d)" % (rw._biome_swatches.size() if rw != null else -1))
	if rw == null or rw._biome_swatches.size() != 15:
		_finish()
		return

	bridge.generate({"seed": 4242, "width_km": 800.0, "grid_w": 256, "grid_h": 164,
		"sea_level": 0.5, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	_check(bridge.has_world, "a world landed")
	## A clean table whatever an earlier session left.
	bridge.reset_biome_colors()
	var g: Vector2i = bridge.grid_size()

	## -- Choose the cell by its class in the FROZEN Biomes raster --------------
	var frozen_legend: Array = []
	for k in range(1, 16):
		frozen_legend.append(_legend(k))
	var b0 := bridge.debug_texture("bclass")
	_check(b0 != null, "the Biomes field draws")
	var img0 := b0.get_image() if b0 != null else null
	var cell := Vector2i(-1, -1)
	var cls := 0
	if img0 != null:
		## Outward from the centre; land classes 1..13 are the ones Biome paint
		## can paint (`PaintTarget::palette()` is `CART_BIOMES[..13]`).
		for r in range(0, mini(g.x, g.y) / 2):
			for dy in range(-r, r + 1):
				for dx in range(-r, r + 1):
					if cls != 0:
						break
					var p := Vector2i(g.x / 2 + dx, g.y / 2 + dy)
					if p.x < 4 or p.y < 4 or p.x >= g.x - 4 or p.y >= g.y - 4:
						continue
					var c := img0.get_pixelv(p)
					for k in range(1, 14):
						if _same8(c, frozen_legend[k - 1]):
							cls = k
							cell = p
							break
			if cls != 0:
				break
	_log("cell %s, class %d, frozen colour %s" % [cell, cls, _c8(frozen_legend[cls - 1]) if cls > 0 else "-"])
	_check(cls > 0, "found a land cell of a paintable class")
	if cls == 0:
		_finish()
		return
	var far := Vector2i((cell.x + g.x / 2) % g.x, (cell.y + g.y / 2) % g.y)
	_log("far control cell %s" % far)

	## -- Positive control: a committed dab moves the map under it ------------
	var m0 := _px(bridge.color_texture(), cell)
	var f0 := _px(bridge.color_texture(), far)
	bridge.paint_set_layer("biome")
	bridge.paint_set_brush(cls, 3.0, 1.0, 0.0, false, false)
	bridge.paint_stroke_at(float(cell.x), float(cell.y))
	bridge.paint_commit()
	await _frames(4)
	var m1 := _px(bridge.color_texture(), cell)
	var f1 := _px(bridge.color_texture(), far)
	_log("map at cell: unpainted %s  painted %s" % [_c8(m0), _c8(m1)])
	_check(m0.a >= 0.0 and not _same8(m0, m1), "positive control: committing the dab moves the map pixel under it")
	_check(_same8(f0, f1), "control: the far, unpainted pixel does not move on the commit")

	## A pending dab for the preview.
	bridge.paint_stroke_at(float(cell.x), float(cell.y))
	var p1 := _px(bridge.build_paint_preview_texture(), cell)
	_log("paint preview before the edit %s" % _c8(p1))
	_check(_same8(p1, frozen_legend[cls - 1]), "the preview starts on the reference colour")

	## -- The edit, through the real picker ----------------------------------
	var pick: ColorPickerButton = rw._biome_swatches[cls - 1]
	_check(_same8(pick.color, frozen_legend[cls - 1]), "the picker opens on the reference colour %s" % _c8(pick.color))
	## The edit marks the project unsaved now that the save carries the table
	## (`appearance.json`'s `biome_cols`, 2026-09-24). Cleared first so the
	## check reads this edit, not the generate before it.
	bridge._set_dirty(false)
	pick.color = EDIT
	pick.color_changed.emit(EDIT)
	pick.popup_closed.emit()
	await _frames(4)
	_check(_same8(bridge.biome_color(cls), EDIT), "the engine holds the edit (%s)" % _c8(bridge.biome_color(cls)))
	_check(bridge.world_dirty, "the edit marks the project unsaved")
	var m2 := _px(bridge.color_texture(), cell)
	var f2 := _px(bridge.color_texture(), far)
	_log("map at cell after the edit %s" % _c8(m2))
	_check(not _same8(m1, m2) and m2.r8 > m1.r8 and m2.g8 < m1.g8,
		"1. the painted cell's map pixel moves toward the edit")
	_check(_same8(f1, f2), "control: the far pixel does not move on the edit")
	var lg := _legend(cls)
	_check(_same8(lg, EDIT), "2. the Biomes legend swatch is the edit (%s)" % _c8(lg))
	var b2 := _px(bridge.debug_texture("bclass"), cell)
	_check(_same8(b2, EDIT), "3. the Biomes field paints the cell in the edit (%s)" % _c8(b2))
	var p2 := _px(bridge.build_paint_preview_texture(), cell)
	_check(_same8(p2, EDIT), "4. the paint preview uses the edit (%s)" % _c8(p2))

	## -- Reset: the row's own button ----------------------------------------
	var row: Node = pick.get_parent()
	var reset_btn: Button = null
	for c in row.get_children():
		if c is Button and not (c is ColorPickerButton) and String((c as Button).text) == "Reset":
			reset_btn = c
	_check(reset_btn != null, "the row has a Reset button")
	bridge._set_dirty(false)
	if reset_btn != null:
		reset_btn.pressed.emit()
	await _frames(4)
	_check(bridge.world_dirty, "the row's Reset marks the project unsaved")
	## A reset with nothing to drop changes nothing a save would write.
	bridge._set_dirty(false)
	if reset_btn != null:
		reset_btn.pressed.emit()
	await _frames(2)
	_check(not bridge.world_dirty, "a second Reset, with nothing left to drop, leaves the project clean")
	var m3 := _px(bridge.color_texture(), cell)
	_check(_same8(m3, m1), "reset restores the map pixel exactly (%s vs %s)" % [_c8(m3), _c8(m1)])
	_check(_same8(_legend(cls), frozen_legend[cls - 1]), "reset restores the legend swatch")
	_check(_same8(pick.color, frozen_legend[cls - 1]), "reset puts the picker back on the reference colour")
	_check(_same8(_px(bridge.build_paint_preview_texture(), cell), frozen_legend[cls - 1]), "reset restores the paint preview")

	## -- Reset all -----------------------------------------------------------
	pick.color = EDIT
	pick.color_changed.emit(EDIT)
	pick.popup_closed.emit()
	await _frames(4)
	## Searched through the whole block, not its direct children: a phone
	## wraps `DccWidgets.action` rows in its own container.
	var all_btn: Button = null
	var in_block: Array = []
	_all(pick.get_parent().get_parent(), in_block)
	for c in in_block:
		## Case-folded: the phone draws `action()` labels in capitals.
		if c is Button and String((c as Button).text).to_lower() == "reset all biome colours":
			all_btn = c
	if all_btn == null:
		for c in in_block:
			if c is Button and not (c is ColorPickerButton) and String((c as Button).text) != "Reset":
				_log("  (button in the block: %s '%s')" % [c.get_class(), (c as Button).text])
	_check(all_btn != null, "the block has a Reset all button")
	if all_btn != null:
		all_btn.pressed.emit()
	await _frames(4)
	_check(_same8(_px(bridge.color_texture(), cell), m1), "Reset all restores the map pixel")
	_check(_same8(bridge.biome_color(cls), frozen_legend[cls - 1]), "Reset all restores the engine table")

	## -- Fit: the block's rows on this device -------------------------------
	## Dock controls are sized in PHYSICAL px on a phone (`phone_fit()` scales
	## the desktop minimum and floors taps at 44 dp x the phone scale), so the
	## bar is the viewport's own width, not a dp figure. And the block's widest
	## pre-existing row is measured beside it, so "fits" also means "does not
	## widen the section beyond what it already asked for" -- `MISTAKES.md`'s
	## ScrollContainer row: an over-wide child would fold into every ancestor.
	var widest := 0.0
	for k in 15:
		var r: Control = (rw._biome_swatches[k] as Control).get_parent()
		widest = maxf(widest, r.get_combined_minimum_size().x)
	var body: Control = (rw._biome_swatches[0] as Control).get_parent().get_parent()
	var widest_other := 0.0
	for c in body.get_children():
		if c is Control and (c as Control).visible:
			var is_row := false
			for k in 15:
				if (rw._biome_swatches[k] as Control).get_parent() == c:
					is_row = true
			if not is_row:
				widest_other = maxf(widest_other, (c as Control).get_combined_minimum_size().x)
	var sw := (rw._biome_swatches[0] as Control).get_combined_minimum_size()
	_log("swatch minimum %.1f x %.1f; widest biome row %.1f; widest other row in the block %.1f; viewport %d px"
		% [sw.x, sw.y, widest, widest_other, _vp.size.x])
	_check(widest <= float(_vp.size.x), "every biome row fits the viewport width")
	_check(widest <= widest_other + 0.5, "no biome row is wider than the block's own widest pre-existing row")
	## The picker's own popup, opened the way a tap opens it.
	var pop: PopupPanel = pick.get_popup()
	pick.get_picker()   ## forces the popup's content to exist before it opens
	pop.popup()
	await _frames(6)
	var pr := Rect2(pop.position, pop.size)
	_log("picker popup at %s size %s (min %s)" % [pop.position, pop.size, pop.get_contents_minimum_size()])
	_check(pop.visible and pop.size.x <= _vp.size.x and pr.position.x >= 0 and pr.end.x <= _vp.size.x + 0.5,
		"the colour popup opens inside the viewport width")
	if phone:
		## Unscaled it measured 298 px on a 1080 px phone (~114 dp): it fit, and
		## was a third of the screen. The desktop popup is 298 px, so at the
		## phone's own scale it must be at least 250 dp.
		_check(pop.size.x >= 250.0 * float(app.phone_scale()),
			"phone: the colour popup is scaled to the phone (%.0f px >= %.0f)" % [pop.size.x, 250.0 * float(app.phone_scale())])
	pop.hide()
	await _frames(4)
	if phone:
		var floor_px := 44.0 * float(app.phone_scale()) - 1.0
		_check(sw.x >= floor_px and sw.y >= floor_px, "phone: the swatch meets the 44 dp tap floor (%.1f px)" % floor_px)

	bridge.paint_discard()
	_finish()
