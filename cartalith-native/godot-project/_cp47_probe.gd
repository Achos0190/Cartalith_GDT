extends Node
## Protects: colour paint's CP-4 (the thumb path), CP-5 (the fill helper), CP-6 (export without paint) and the
## map-width notice with Scale with the map (`COLOUR_PAINT_SCOPE.md` §5, §13, §14, §17 Q5; Ruling EB item 7), through
## the real shell on a generated world, with real engine calls and pixel readback:
##
##   A (CP-4). While `colour_paint` is armed the cartography provider offers the brush's `"tool"` rows (size, colour,
##      material, blend, Land only); stepping Size moves the radius by x1.25 and the Radius slider with it and leaves the
##      map's zoom alone; a pinch (two fingers) zooms the map and leaves the brush radius alone; on touch the tool
##      strip draws the rows as chips.
##   B (CP-5). The fill tool is a tap tool; a tap in a real depression of this world previews a closed ring (the engine's
##      own, found by scanning the world, never assumed), Level and Cap reshape it, Commit paints an Area plus a grow
##      stroke as ONE undo step that moves the map raster inside the ring and nowhere far, undo and redo restore and
##      reapply it, Esc discards, arming another tool clears the preview, a locked layer refuses the commit with the
##      document untouched, and a tap on flat or edge ground says why and previews nothing.
##   C (the notice). A regenerate at another map width under existing paint raises the notice with the scope's sentence,
##      the marks unchanged; Show marks outlines every mark and changes none; Scale with the map multiplies every point and
##      radius by new / old as ONE undo step, undo brings the marks and the notice back, redo scales again; Dismiss moves
##      nothing; a regenerate at the same width raises none; a new stroke while it is pending accepts the new width.
##   D (CP-6). `content.colour_paint` false exports the unpainted terrain (equal to an export with the paint cleared),
##      the default exports the paint (different), on the terrain-only path and on the overlay session's path.
##
## Pixel readback, so WINDOWED, never `--headless`:
##   godot --path <copy> res://_cp47_probe.tscn -- --tag desktop --out DIR
##   godot --path <copy> res://_cp47_probe.tscn -- --force-touch --vp 1080x2340 --tag phone --out DIR
##   godot --path <copy> res://_cp47_probe.tscn -- --force-touch --vp 2560x1600 --tag tablet --out DIR
## Flags: `--vp WxH`, `--tag NAME`, `--out DIR` (required), `--force-touch`. Any other `--flag` aborts.
## Prints `[<tag>]   OK|FAIL <what>` and `[<tag>] RESULT <tag> checks=<n> fail=<n>`. Exit 0 = held, 1 = failed, 2 = could
## not run. What it does NOT prove: a phone run proves the rows and the gestures through the app's own dispatch, not
## that a finger on glass reaches them (no device was attached).

const EXPORT_OVERLAY := preload("res://shell/export_overlay.gd")

var app: Node
var bridge: Node
var _vp: SubViewport
var _tag := "cp47"
var _out := ""
var _fail := 0
var _checks := 0
var _tool
var _carto

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])

func _check(ok: bool, what: String) -> void:
	_checks += 1
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _finish() -> void:
	_log("RESULT %s checks=%d fail=%d" % [_tag, _checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

func _cannot(why: String) -> void:
	printerr("PROBE-CANNOT-RUN: " + why)
	_log("CANNOT-RUN " + why)
	get_tree().quit(2)

func _shot(name: String) -> void:
	var img := _vp.get_texture().get_image()
	if img != null:
		img.save_png(_out.path_join("%s_%s.png" % [_tag, name]))

func _all(root: Node, out: Array) -> void:
	for c in root.get_children(true):
		out.append(c)
		_all(c, out)

func _button(root: Node, text: String) -> Button:
	var all: Array = []
	_all(root, all)
	for n in all:
		if n is Button and String((n as Button).text).to_lower() == text.to_lower():
			return n
	return null

func _raster() -> Image:
	var t: Texture2D = bridge.color_texture()
	if t == null:
		return null
	var img := t.get_image()
	if img != null and img.is_compressed():
		img.decompress()
	return img

func _moved(a: Image, b: Image, r: Rect2i) -> int:
	var n := 0
	for y in range(maxi(r.position.y, 0), mini(r.end.y, a.get_height())):
		for x in range(maxi(r.position.x, 0), mini(r.end.x, a.get_width())):
			var p := a.get_pixel(x, y)
			var q := b.get_pixel(x, y)
			if absi(p.r8 - q.r8) > 6 or absi(p.g8 - q.g8) > 6 or absi(p.b8 - q.b8) > 6:
				n += 1
	return n

func _marks() -> int:
	return int(bridge.colour_paint_info().get("marks", -1))

## The cartography provider's `"tool"` rows for the armed tool, by id.
func _rows(armed: String) -> Dictionary:
	var out := {}
	for r in _carto.context_actions({"domain": "cartography", "gx": 0.0, "gy": 0.0, "armed_tool": armed, "hits": []}):
		if String((r as Dictionary).get("section", "")) == "tool":
			out[String(r["id"])] = r
	return out

func _val(rows: Dictionary, id: String) -> String:
	return String(((rows[id] as Dictionary)["param"] as Dictionary)["value"].call())

func _step(rows: Dictionary, id: String, dir: int) -> void:
	((rows[id] as Dictionary)["param"] as Dictionary)["step"].call(dir)

## A world cell the fill helper outlines as a crater or basin of a sensible size, found by asking the ENGINE (stride
## 2 over the interior: the helper refuses a flat or edge cell in microseconds), so the probe assumes no coordinates.
func _find_crater() -> Vector2i:
	var g: Vector2i = bridge.grid_size()
	var best := Vector2i(-1, -1)
	var best_area := 0.0
	for y in range(20, g.y - 20, 2):
		for x in range(20, g.x - 20, 2):
			var r: Dictionary = bridge.colour_paint_fill_preview(x, y, {})
			if bool(r.get("ok", false)):
				var a := float(r.get("area_km2", 0.0))
				if a >= 6.0 and a <= 400.0 and a > best_area:
					best_area = a
					best = Vector2i(x, y)
	return best

func _tap(c: Vector2i) -> void:
	app._on_map_clicked(c.x + 0.5, c.y + 0.5)
	await _frames(3)

func _ready() -> void:
	_tag = _arg("--tag", "cp47")
	_out = _arg("--out", "")
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--force-touch", "--vp", "--tag", "--out"]):
			_log("ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		_cannot("run WINDOWED -- ImageTexture readback is a no-op under --headless")
		return
	if _out == "":
		_cannot("--out DIR is required")
		return
	DirAccess.make_dir_recursive_absolute(_out)
	get_tree().create_timer(1500.0).timeout.connect(func() -> void: _cannot("watchdog"))
	var parts: PackedStringArray = _arg("--vp", "1600x900").split("x")
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
	_log("viewport %dx%d  phone=%s touch=%s" % [_vp.size.x, _vp.size.y, app.is_phone(), DccTheme.is_touch()])
	for flag in ["colour_paint_api", "colour_paint_layers_api", "colour_paint_fill_api", "colour_paint_scale_api"]:
		if not bool(bridge.get(flag)):
			_cannot("the loaded extension lacks %s -- rebuild the DLL (cargo build -p cartalith-godot)" % flag)
			return
	bridge.generate({"seed": 71077345, "width_km": 800.0, "grid_w": 512, "grid_h": 328, "sea_level": 0.42, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	if not bridge.has_world:
		_cannot("no generated world")
		return
	for ws in app._workspaces:
		if ws is CartographyWorkspace:
			_carto = ws
	_tool = _carto.colour_paint if _carto != null else null
	if _tool == null:
		_cannot("no colour-paint tool")
		return
	await _leg_a()
	await _leg_b()
	await _leg_c()
	await _leg_d()
	_finish()

# -- A: CP-4 ------------------------------------------------------------------------

func _leg_a() -> void:
	_log("-- A: the thumb path (CP-4)")
	_check(not _rows("inspect").has("carto.cp_size") and not _rows("sculpt").has("carto.cp_size"), "no other tool is offered the brush rows")
	app.arm_tool("colour_paint")
	await _frames(4)
	var rows := _rows("colour_paint")
	for id in ["carto.cp_size", "carto.cp_colour", "carto.cp_material", "carto.cp_blend", "carto.cp_land"]:
		_check(rows.has(id), "the armed brush offers %s" % id)
	if not rows.has("carto.cp_size"):
		return
	_tool.set_radius(2.0)
	var z0: float = app.viewport.zoom()
	_step(rows, "carto.cp_size", 1)
	_check(is_equal_approx(_tool.radius_km, 2.5) and _val(rows, "carto.cp_size") == "2.50 km", "Size + takes the radius from 2.00 to 2.50 km (%s)" % _val(rows, "carto.cp_size"))
	var sl = _tool._radius_slider
	_check(not sl.is_empty() and is_equal_approx((sl["slider"] as HSlider).value, 2.5), "the Radius slider moved with the stepper")
	_step(rows, "carto.cp_size", -1)
	_check(is_equal_approx(_tool.radius_km, 2.0), "Size - takes it back")
	_check(is_equal_approx(app.viewport.zoom(), z0), "resizing the brush did not zoom the map (%.4f -> %.4f)" % [z0, app.viewport.zoom()])
	var rgb0: Color = _tool.rgb
	_step(rows, "carto.cp_colour", 1)
	_check(_tool.rgb != rgb0 or _val(rows, "carto.cp_colour") != "custom", "Colour + moves to a swatch (%s)" % _val(rows, "carto.cp_colour"))
	var mat0: String = _tool.material
	_step(rows, "carto.cp_material", 1)
	_check(_tool.material != mat0, "Material + moves to the next material (%s -> %s)" % [mat0, _tool.material])
	_step(rows, "carto.cp_material", -1)
	var bl0: String = _tool.blend
	_step(rows, "carto.cp_blend", 1)
	_check(_tool.blend != bl0, "Blend + cycles (%s -> %s)" % [bl0, _tool.blend])
	_step(rows, "carto.cp_blend", -1)
	var lo0: bool = _tool.land_only
	_step(rows, "carto.cp_land", 1)
	_check(_tool.land_only != lo0, "Land only toggles")
	_step(rows, "carto.cp_land", 1)
	## Two fingers belong to the map: a pinch zooms it and never resizes the brush.
	var r0: float = _tool.radius_km
	var z1: float = app.viewport.zoom()
	var ev := InputEventMagnifyGesture.new()
	ev.position = app.viewport.global_position + app.viewport.size * 0.5
	ev.factor = 1.3
	app.viewport._input(ev)
	await _frames(2)
	_check(app.viewport.zoom() > z1 * 1.2 and is_equal_approx(_tool.radius_km, r0), "a pinch zooms the map (%.3f -> %.3f) and leaves the brush radius at %.2f km" % [z1, app.viewport.zoom(), r0])
	if app.is_phone():
		var strip = app._tool_strip
		_check(strip != null and strip.is_visible_in_tree(), "the phone's tool strip is showing while the brush is armed")
		if strip != null:
			strip.refresh()
			await _frames(3)
			var st: Dictionary = strip.debug_state()
			_check(int(st.get("options", 0)) >= 5, "the strip draws the five brush rows as chips (%s)" % str(st))
			_check(app._tool_fan.pill_label() == "COLOUR", "the pill reads COLOUR")
	_shot("a_brush_rows")

# -- B: CP-5 ------------------------------------------------------------------------

func _leg_b() -> void:
	_log("-- B: the fill helper (CP-5)")
	var fill = _tool.fill
	_check(DccApp.tool_kind("colour_fill") == ViewportHost.TOOL_PLACING, "the fill tool is a tap tool (a one-finger drag pans on touch)")
	var c := _find_crater()
	_check(c.x >= 0, "the engine outlines a crater or basin in this world (found at %s)" % str(c))
	if c.x < 0:
		return
	var btn := _button(_carto, "Fill an area")
	_check(btn != null, "CARTO > Colours draws 'Fill an area'")
	if btn != null:
		btn.pressed.emit()
	await _frames(3)
	_check(app.armed_tool == "colour_fill", "pressing it arms colour_fill (%s)" % app.armed_tool)
	var g: Vector2i = bridge.grid_size()
	var before_doc: String = bridge.colour_paint_document()
	var marks0 := _marks()
	var undo0: int = _tool._undo.size()
	var raster0 := _raster()
	_tool.radius_km = 1.0
	_tool.fill_grow_km = 1.5
	_tool.fill_land_only = false
	_tool.material = "colour"
	_tool.blend = "replace"
	_tool.rgb = Color8(200, 30, 18)
	_tool.opacity = 1.0
	await _tap(c)
	_check(fill.has_preview(), "a tap in the depression previews an outline")
	var ov = app.viewport.tool_overlay
	var ring: PackedVector2Array = ov.path_preview
	_check(ring.size() >= 5 and ring[0] == ring[ring.size() - 1], "the preview ring is on the map and closed (%d points)" % ring.size())
	_check(_marks() == marks0 and bridge.colour_paint_document() == before_doc, "previewing writes nothing")
	var area0 := float(fill._preview.get("area_km2", 0.0))
	var mode := String(fill._preview.get("mode", ""))
	_check(mode == "pour_point" or mode == "capped", "the engine says which fill it used (%s)" % mode)
	_check(area0 > 0.0 and ring.size() > 0, "the outline has an area (%.1f km2)" % area0)
	## Level reshapes it (positive control: the preview must change or say why not).
	var rows := _rows("colour_fill")
	for id in ["carto.cf_level", "carto.cf_cap", "carto.cf_grow", "carto.cf_land"]:
		_check(rows.has(id), "the armed fill offers %s" % id)
	_step(rows, "carto.cf_level", -1)
	var area1 := float(fill._preview.get("area_km2", 0.0))
	_check(fill._error != "" or not is_equal_approx(area1, area0), "Level - changes the outline or says why it cannot (%.1f -> %.1f km2, error '%s')" % [area0, area1, fill._error])
	_step(rows, "carto.cf_level", 1)
	_check(is_equal_approx(fill.level_m, 0.0), "Level + returns to the automatic level")
	if mode == "capped":
		_step(rows, "carto.cf_cap", -1)
		_check(fill.cap_km != null, "Cap - takes a cap off the automatic one (%s)" % _val(rows, "carto.cf_cap"))
		fill.cap_km = null
		fill._run()
	_check(fill.has_preview(), "back at the automatic level and cap the outline is previewed again")
	_shot("b_preview")
	## Commit: ONE undo step, an area plus a grow stroke.
	var ring_px := Rect2()
	for p in (fill._preview["ring_km"] as PackedVector2Array):
		var q: Vector2 = p / _tool.km_per_cell()
		ring_px = ring_px.expand(q) if ring_px != Rect2() else Rect2(q, Vector2.ZERO)
	var ok: bool = fill.commit()
	await _frames(6)
	_check(ok and _marks() == marks0 + 2, "Commit paints the area and its grow stroke (%d -> %d marks)" % [marks0, _marks()])
	_check(_tool._undo.size() == undo0 + 1 and String(_tool._undo.back()["label"]) == "Fill", "as ONE undo step labelled Fill")
	_check(not fill.has_preview() and ov.path_preview.size() == 0, "the preview is cleared after Commit")
	var raster1 := _raster()
	var inside := Rect2i(int(ring_px.position.x), int(ring_px.position.y), maxi(int(ring_px.size.x), 1), maxi(int(ring_px.size.y), 1))
	var far := Rect2i(0, 0, 12, 12)
	var m_in := _moved(raster0, raster1, inside)
	_check(m_in > 4 and _moved(raster0, raster1, far) == 0, "the fill moved the raster inside the outline (%d px of %d) and nothing far from it" % [m_in, inside.get_area()])
	_shot("b_filled")
	_tool.undo()
	await _frames(3)
	_check(_marks() == marks0 and bridge.colour_paint_document() == before_doc, "one undo removes the fill: the document is what it was")
	_tool.redo()
	await _frames(3)
	_check(_marks() == marks0 + 2, "one redo puts both marks back")
	_tool.undo()
	await _frames(3)
	## Esc discards, then puts the tool down.
	await _tap(c)
	_check(fill.has_preview(), "premise: a preview to discard")
	app._escape_action()
	await _frames(2)
	_check(not fill.has_preview() and ov.path_preview.size() == 0 and app.armed_tool == "colour_fill", "the first Esc discards the preview and keeps the tool")
	app._escape_action()
	await _frames(2)
	_check(app.armed_tool == "inspect", "the next Esc puts the tool down")
	## Arming another tool clears a preview.
	app.arm_tool("colour_fill")
	await _tap(c)
	_check(fill.has_preview(), "premise: a preview again")
	app.arm_tool("measure")
	await _frames(3)
	_check(not fill.has_preview() and ov.path_preview.size() == 0, "arming another tool clears the preview")
	## A locked layer refuses the commit, whole.
	app.arm_tool("colour_fill")
	await _tap(c)
	var layers: Array = _tool.layers()
	_tool.add_layer("Lockable")
	_tool.set_layer_locked(_tool._clamped_active(), true)
	var doc_locked: String = bridge.colour_paint_document()
	var undo_locked: int = _tool._undo.size()
	_check(not fill.commit() and bridge.colour_paint_document() == doc_locked and _tool._undo.size() == undo_locked, "a locked active layer refuses the commit with the document and the undo stack untouched")
	_tool.set_layer_locked(_tool._clamped_active(), false)
	_tool.undo()
	_tool.undo()
	fill.cancel()
	## Flat or edge ground says why and previews nothing.
	await _tap(Vector2i(2, 2))
	_check(not fill.has_preview() and fill._error != "" and ov.path_preview.size() == 0, "a tap at the map edge previews nothing and says why ('%s')" % fill._error)
	if app.is_phone():
		app.arm_tool("colour_fill")
		await _tap(c)
		var v: Dictionary = app._tool_strip._verbs()
		_check(not v.is_empty() and bool(v.get("can", false)), "the phone strip offers Commit / Discard while an outline is previewed")
		fill.cancel()
	app.arm_tool("inspect")
	_tool.clear()
	await _frames(3)

# -- C: the notice -------------------------------------------------------------------

## The document as parsed JSON.
func _doc() -> Dictionary:
	var d = JSON.parse_string(bridge.colour_paint_document())
	return d if d is Dictionary else {}

func _first_line_marks() -> Array:
	var out: Array = []
	var d := _doc()
	var marks: Array = []
	if d.has("layers"):
		for l in d["layers"]:
			marks.append_array(l["marks"])
	else:
		marks = d.get("marks", [])
	for m in marks:
		out.append(m)
	return out

func _leg_c() -> void:
	_log("-- C: the map-width notice, Scale with the map")
	var notice = _tool.notice
	_tool.clear()
	await _frames(2)
	_tool.undo()
	_tool.clear()
	## Two strokes: one near the centre, one on ground that is SEA at 1200 km (found by the input elevation in that
	## world), so after the regenerate the on-water count has a positive control, and the centre stroke is the negative.
	bridge.generate({"seed": 71077345, "width_km": 1200.0, "grid_w": 512, "grid_h": 328, "sea_level": 0.42, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	var sea_km := Vector2(-1.0, -1.0)
	var kpc12: float = _tool.km_per_cell()
	var gg: Vector2i = bridge.grid_size()
	for y in range(12, gg.y - 12, 8):
		for x in range(12, gg.x - 12, 8):
			var all_sea := true
			for d in [Vector2i(0, 0), Vector2i(3, 0), Vector2i(-3, 0), Vector2i(0, 3), Vector2i(0, -3)]:
				var c: Dictionary = bridge.sample_cell(x + d.x, y + d.y)
				all_sea = all_sea and c.has("elevation_m") and float(c["elevation_m"]) < -50.0
			if all_sea and sea_km.x < 0.0:
				sea_km = Vector2(x * kpc12, y * kpc12)
	_check(sea_km.x >= 0.0, "premise: a patch of open sea at 1200 km (%s km)" % str(sea_km))
	bridge.generate({"seed": 71077345, "width_km": 800.0, "grid_w": 512, "grid_h": 328, "sea_level": 0.42, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	_tool.clear()
	_tool.radius_km = 1.0
	_tool.land_only = true
	_tool.material = "colour"
	var kpc: float = _tool.km_per_cell()
	var g: Vector2i = bridge.grid_size()
	_check(_tool.commit_line([Vector2(g.x * 0.5 * kpc, g.y * 0.5 * kpc), Vector2(g.x * 0.5 * kpc + 40.0, g.y * 0.5 * kpc)]), "a stroke near the centre")
	_check(_tool.commit_line([sea_km, sea_km + Vector2(6.0, 0.0)]), "a land-only stroke over what is sea at 1200 km")
	await _frames(3)
	_check(_marks() == 2 and notice.state().is_empty(), "two marks and no notice at the width they were painted at")
	var wet0 = bridge.colour_paint_marks_on_water()
	var doc_before: String = bridge.colour_paint_document()
	bridge.generate({"seed": 71077345, "width_km": 1200.0, "grid_w": 512, "grid_h": 328, "sea_level": 0.42, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	var s: Dictionary = notice.state()
	_check(not s.is_empty() and is_equal_approx(float(s["old"]), 800.0) and is_equal_approx(float(s["new"]), 1200.0), "the regenerate at 1200 km raises the notice from 800 km (%s)" % str(s))
	_check(bridge.colour_paint_document() == doc_before, "the marks kept their kilometres: the document is unchanged")
	var sent: String = notice.sentence(s) if not s.is_empty() else ""
	## The widths follow Preferences > Units (800 and 1 200 km as literals per mode, not read back from the code under test).
	var widths: Dictionary = {"km": "800 km → 1,200 km", "mi": "497 mi → 746 mi", "nmi": "432 nm → 648 nm"}
	var want_w: String = String(widths.get(DccSettings.units_mode(), ""))
	_check(want_w != "" and sent.begins_with("Map width %s: 2 paint marks kept their real size and position" % want_w), "the sentence is the scope's in the %s unit (%s)" % [DccSettings.units_mode(), sent])
	var wet = bridge.colour_paint_marks_on_water()
	_check(wet is int and int(wet) == 1, "exactly one mark (the sea one, not the centre one) is on water at 1200 km (%s)" % str(wet))
	_check(sent.contains("1 now on water is hidden, not deleted"), "the on-water clause is in the sentence")
	_log("  on-water before regenerate: %s, after: %s" % [str(wet0), str(wet)])
	_shot("c_notice")
	## Show marks changes nothing and clears itself.
	var ov = app.viewport.tool_overlay
	notice.show_marks()
	_check(ov.mark_outlines.size() == 2 and bridge.colour_paint_document() == doc_before, "Show marks outlines both marks and changes none")
	## Scale with the map.
	var undo0: int = _tool._undo.size()
	_check(notice.scale_with_map(), "Scale with the map succeeds")
	await _frames(3)
	var scaled: Array = _first_line_marks()
	var was: Array = (JSON.parse_string(doc_before) as Dictionary)["marks"] if (JSON.parse_string(doc_before) as Dictionary).has("marks") else _first_line_marks()
	_check(_marks() == 2 and _tool._undo.size() == undo0 + 1 and String(_tool._undo.back()["label"]) == "Scale paint with the map", "one undo step, 'Scale paint with the map', no mark added or deleted")
	var all_x15 := scaled.size() == was.size() and not scaled.is_empty()
	for i in mini(scaled.size(), was.size()):
		var a: Dictionary = scaled[i]["shape"]
		var b: Dictionary = was[i]["shape"]
		all_x15 = all_x15 and is_equal_approx(float(a["radius_km"]), float(b["radius_km"]) * 1.5)
		for j in (a["points_km"] as Array).size():
			all_x15 = all_x15 and is_equal_approx(float(a["points_km"][j][0]), float(b["points_km"][j][0]) * 1.5) and is_equal_approx(float(a["points_km"][j][1]), float(b["points_km"][j][1]) * 1.5)
	_check(all_x15, "every point and radius is x1.5 (1200 / 800)")
	_check(notice.state().is_empty(), "the notice is gone once the marks are scaled")
	_tool.undo()
	await _frames(3)
	_check(bridge.colour_paint_document() == doc_before and not notice.state().is_empty(), "one undo puts every mark back at its kilometres and brings the notice back")
	_tool.redo()
	await _frames(3)
	_check(_first_line_marks() == scaled and notice.state().is_empty(), "redo scales them again and removes the notice")
	_tool.undo()
	await _frames(3)
	## Dismiss.
	notice.dismiss()
	_check(notice.state().is_empty() and bridge.colour_paint_document() == doc_before, "Dismiss removes the row and moves no mark")
	## The same width again: no notice (nothing moved relative to the plate).
	bridge.generate({"seed": 71077345, "width_km": 1200.0, "grid_w": 512, "grid_h": 328, "sea_level": 0.42, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	_check(notice.state().is_empty(), "a regenerate at the same width raises no notice again after Dismiss")
	## Painting while a notice is pending accepts the new width.
	bridge.generate({"seed": 71077345, "width_km": 1000.0, "grid_w": 512, "grid_h": 328, "sea_level": 0.42, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	_check(not notice.state().is_empty(), "a regenerate at 1000 km raises a notice")
	_check(_tool.commit_line([Vector2(60.0, 60.0), Vector2(70.0, 60.0)]) and notice.state().is_empty(), "a new stroke while it is pending accepts the new width")
	## Show marks clears itself after OUTLINE_SECONDS.
	notice.show_marks()
	await get_tree().create_timer(notice.OUTLINE_SECONDS + 0.8).timeout
	_check(ov.mark_outlines.is_empty(), "the Show marks outlines clear themselves")
	_tool.clear()

# -- D: CP-6 -------------------------------------------------------------------------

func _file_bytes(path: String) -> PackedByteArray:
	return FileAccess.get_file_as_bytes(path)

func _leg_d() -> void:
	_log("-- D: export without paint (CP-6)")
	bridge.generate({"seed": 71077345, "width_km": 800.0, "grid_w": 512, "grid_h": 328, "sea_level": 0.42, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	var dir := OS.get_user_data_dir().path_join("cp47_export")
	DirAccess.make_dir_recursive_absolute(dir)
	var kpc: float = _tool.km_per_cell()
	var g: Vector2i = bridge.grid_size()
	_tool.land_only = false
	_tool.blend = "replace"
	_tool.rgb = Color8(200, 30, 18)
	_tool.radius_km = 25.0
	_tool.commit_line([Vector2(g.x * 0.3 * kpc, g.y * 0.5 * kpc), Vector2(g.x * 0.7 * kpc, g.y * 0.5 * kpc)])
	_check(_marks() == 1, "premise: one mark to export")
	var wg = bridge.world_gen
	var exp := func(name: String, content: Dictionary) -> PackedByteArray:
		var p := dir.path_join(name)
		var r: Dictionary = wg.export_image(p, {"width": 2048, "content": content})
		_check(bool(r.get("ok", false)), "export %s ok (%s)" % [name, str(r.get("error", ""))])
		return _file_bytes(p)
	var with_paint: PackedByteArray = exp.call("with.png", {})
	var explicit_on: PackedByteArray = exp.call("on.png", {"colour_paint": true})
	var without: PackedByteArray = exp.call("off.png", {"colour_paint": false})
	_tool.clear()
	var none: PackedByteArray = exp.call("none.png", {})
	_check(with_paint.size() > 0 and with_paint == explicit_on, "the default export carries the paint, as colour_paint true does")
	_check(with_paint != without, "colour_paint false changes the picture")
	_check(without == none, "colour_paint false is exactly the unpainted export (%d bytes)" % without.size())
	var bad: Dictionary = wg.export_image(dir.path_join("bad.png"), {"width": 2048, "content": {"colour_paint": "yes"}})
	_check(not bool(bad.get("ok", true)), "a non-boolean colour_paint is refused (%s)" % str(bad.get("error", "")))
	## The overlay session path.
	_tool.commit_line([Vector2(g.x * 0.3 * kpc, g.y * 0.5 * kpc), Vector2(g.x * 0.7 * kpc, g.y * 0.5 * kpc)])
	var ov: Control = null
	var all: Array = []
	_all(app, all)
	for n in all:
		if n.get_script() != null and String(n.get_script().resource_path).ends_with("map_overlay.gd"):
			ov = n
			break
	if ov != null:
		var runner := EXPORT_OVERLAY.new()
		add_child(runner)
		await get_tree().create_timer(1.0).timeout
		var sess := func(name: String, paint: bool) -> PackedByteArray:
			var p := dir.path_join(name)
			var r: Dictionary = await runner.run(bridge, p, {"width": 2048, "content": {"ways": true, "colour_paint": paint}}, ov, [app.viewport.territory_view, app.viewport.province_view])
			_check(bool(r.get("ok", false)), "session export %s ok (%s)" % [name, str(r.get("error", ""))])
			return _file_bytes(p)
		var s_on: PackedByteArray = await sess.call("s_on.png", true)
		var s_off: PackedByteArray = await sess.call("s_off.png", false)
		_tool.clear()
		var s_none: PackedByteArray = await sess.call("s_none.png", true)
		_check(s_on != s_off and s_off == s_none, "the overlay session honours colour_paint too: off equals the unpainted session export")
	else:
		_check(false, "found the map overlay for the session export")
	for f in DirAccess.get_files_at(dir):
		DirAccess.remove_absolute(dir.path_join(f))
	DirAccess.remove_absolute(dir)
