extends Node
## **New World, owner report 2026-11-13: four items, measured.** ("Generating roads, labels, the toggle in new world seems to
## have no effect. Also the option for the amount of plates or the world presets such as how much villages and how the
## population of villages behaves seems to have gone missing. Additionally: in the new world I want to be able to set the base
## tech level for the whole world also, next to the phase the world is in. And the preview, instead of grey on blue a
## rudimentary heightmap would be preferred.")
##
## Legs (each prints OK/FAIL rows; the last line is `RESULT <tag> fail=<n> checks=<n>`):
##   toggles  Create (seed 4242, small grid) with the Auto-populate switches set each way; counts of ways, generated labels,
##            settlements and landmarks after it. A switch that is OFF must leave its product at ZERO, and an ON control run
##            must leave it above zero (the positive control: without it "zero" could mean the pass never works here).
##   layout   the controls the owner named (Plates, Factions, Village seeding, Fixed counts and the five counts, Recovery phase,
##            Base technology) are visible ON THE FIRST SCREEN of the dialog, scrolled to if the card scrolls, and the engine
##            reads what each one is set to after Create.
##   tech     the base technology control sits beside Recovery phase, lists the engine's six levels, and Create makes every
##            faction's ag_tech that level (read through `civ_roster()`); the default level is the reference's, so a default
##            Create is unchanged.
##   preview  the preview is a heightmap: more than two distinct colours, land and ocean separated, and brightness on land
##            rising with the engine's own height field (Pearson r against `preview_height_field`).
##
## Run WINDOWED, under the lock (`ImageTexture.update()` is a no-op headless):
##   godot --path . res://_newworld_items_probe.tscn -- --vp 1280x900 --tag desk --shots DIR
##   godot --path . res://_newworld_items_probe.tscn -- --force-touch --vp 1080x2340 --density 2.8125 --tag phone --shots DIR
## Flags: `--vp WxH`, `--tag NAME`, `--force-touch`, `--grid N` (default 192), `--legs a,b`, `--shots DIR`, `--density F`
## (accepted for the record; the harness sets the viewport, the shell reads the real screen). Anything else aborts (exit 2).

const SEED := 4242
## The tiers the owner's presets name; literals, typed here, not read from the code under test.
const COUNT_LABELS := ["Capitals", "Cities", "Towns", "Villages", "Hamlets"]

var app: Node
var bridge: Node
var dlg: Node
var _vp: SubViewport
var _tag := "nwi"
var _fail := 0
var _checks := 0
var _grid := 192
var _shots := ""


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


func _ready() -> void:
	_tag = _arg("--tag", "nwi")
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--force-touch", "--vp", "--tag", "--grid", "--legs", "--shots", "--density"]):
			_log("ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	_grid = int(_arg("--grid", "192"))
	_shots = _arg("--shots", "")
	var parts: PackedStringArray = _arg("--vp", "1280x900").split("x")
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
	dlg = app.new_world_dialog
	if DisplayServer.get_name() == "headless":
		_log("ABORT headless; nothing was measured")
		get_tree().quit(2)
		return
	_log("viewport %dx%d phone=%s tablet=%s touch=%s" % [_vp.size.x, _vp.size.y, app.is_phone(), DccTheme.is_tablet(), DccTheme.is_touch()])
	var legs: PackedStringArray = _arg("--legs", "layout,preview,tech,toggles").split(",")
	app.open_new_world()
	await _frames(30)
	if not dlg.visible:
		_check(false, "the New World dialog is up")
		_finish()
		return
	await _shot("00_open")
	for leg in legs:
		match leg:
			"layout": await _leg_layout()
			"preview": await _leg_preview()
			"presets": await _leg_presets()
			"tech": await _leg_tech()
			"toggles": await _leg_toggles()
			_: _check(false, "unknown leg %s" % leg)
	_finish()


func _finish() -> void:
	_log("RESULT %s fail=%d checks=%d" % [_tag, _fail, _checks])
	get_tree().quit(1 if _fail > 0 else 0)


## Saves the viewport as `<shots>/<tag>_<name>.png` when `--shots` was given.
func _shot(name: String) -> void:
	if _shots.is_empty():
		return
	await _frames(3)
	DirAccess.make_dir_recursive_absolute(_shots)
	var img := _vp.get_texture().get_image()
	img.save_png("%s/%s_%s.png" % [_shots, _tag, name])


# -- helpers ------------------------------------------------------------------------------------

## The first Label under the dialog whose text is exactly `text`, or null.
func _find_label(text: String, root: Node = null) -> Control:
	var n: Node = root if root != null else dlg
	if n is Label and (n as Label).text == text:
		return n as Label
	## A switch carries its own caption (`CheckBox.text`): the row IS the button, so it answers for its own text.
	if n is CheckBox and (n as CheckBox).text == text:
		return n as Control
	for c in n.get_children():
		var r := _find_label(text, c)
		if r != null:
			return r
	return null


## Whether `c` can be seen by someone using the dialog: shown in the tree AND, once its scroller is asked to bring it into view,
## inside that scroller's rectangle. A control in a collapsed More options body or in a hidden group fails the first half.
func _reachable(c: Control) -> bool:
	if c == null or not c.is_visible_in_tree():
		return false
	var q: Node = c.get_parent()
	while q != null:
		if q is ScrollContainer:
			(q as ScrollContainer).ensure_control_visible(c)
			await _frames(2)
			var sr := (q as ScrollContainer).get_global_rect()
			var r := c.get_global_rect()
			return r.size.x > 0.0 and r.size.y > 0.0 and sr.encloses(r.grow(-1.0))
		q = q.get_parent()
	return true


func _row_control(label_text: String) -> Control:
	var l := _find_label(label_text)
	if l == null:
		return null
	return l if l is CheckBox else l.get_parent() as Control


## Create closes the dialog, and a closed dialog fails every visibility test: reopen it (a no-op when it is up).
func _ensure_open() -> void:
	if not dlg.visible:
		app.open_new_world()
		await _frames(30)


func _create_and_wait() -> void:
	var finished := {"v": false}
	dlg.auto_populate_finished.connect(func(_r): finished["v"] = true, CONNECT_ONE_SHOT)
	dlg.get_ok_button().pressed.emit()
	await bridge.generation_finished
	var guard := 0
	while not finished["v"] and guard < 4000:
		await get_tree().process_frame
		guard += 1
	await _frames(6)
	_check(finished["v"], "Create finished (and its passes, if any)")


func _set_grid_seed(seed_v: int = SEED) -> void:
	## Value AND display text, and no `apply()`: `apply()` re-parses the text the field shows, which a value written from code does not
	## update on an embedded dialog (`new_world_dialog.gd::request()` records the hazard), so it put the previous seed straight
	## back and the first version of the preview leg judged one world three times.
	dlg.seed_input.value = seed_v
	dlg.seed_input.get_line_edit().text = str(seed_v)
	dlg.grid_w_input.value = _grid
	dlg.grid_w_input.get_line_edit().text = str(_grid)


func _cb(key: String) -> CheckBox:
	return dlg.get("_ap_checks").get(key) as CheckBox


# -- toggles ------------------------------------------------------------------------------------

## The counts the owner looks at: generated ways, generated labels, settlements, landmarks.
func _counts() -> Dictionary:
	var gen_labels := 0
	for row in bridge.labels_render_list():
		if bool((row as Dictionary).get("generated", false)):
			gen_labels += 1
	return {"ways": bridge.roads().size(), "labels": gen_labels, "settlements": bridge.settlements().size(),
		"landmarks": bridge.landmarks().size()}


func _leg_toggles() -> void:
	_log("leg toggles (seed %d, grid %d)" % [SEED, _grid])
	var civ_now := {}
	for k in bridge.param_snapshot():
		if String(k).begins_with("civ.") or String(k) == "tect.plates":
			civ_now[k] = bridge.param_snapshot()[k]
	_log("  civ parameters on entry: %s" % str(civ_now))
	var rq: Dictionary = dlg.request()
	_log("  the form would post: villages %s, recovery_phase %s, metropolis %s, biome_k %s; dialog open %s" % [str(rq["villages"]), str(rq["recovery_phase"]), str(rq["metropolis"]), str(rq["biome_k"]), str(dlg.visible)])
	var cases := [
		{"name": "all on", "master": true, "off": []},
		{"name": "routes off", "master": true, "off": ["routes"]},
		{"name": "names off", "master": true, "off": ["names"]},
		{"name": "settlements off", "master": true, "off": ["settlements"]},
		{"name": "master off", "master": false, "off": []},
	]
	var got := {}
	for c in cases:
		_cb("master").button_pressed = bool(c["master"])
		for k in ["settlements", "routes", "landmarks", "names"]:
			_cb(k).button_pressed = not (k in (c["off"] as Array))
		await _frames(2)
		_set_grid_seed()
		await _frames(2)
		await _create_and_wait()
		got[c["name"]] = _counts()
		if c["name"] == "all on":
			var gs = bridge.grid_size()
			_log("  all-on world: grid %s, width %s km, sea level %s, plates %s, kinds %s, recovery %s, factions %s" % [str(gs), str(bridge.last_width_km), str(bridge.param_get("sea_level")), str(bridge.param_get("tect.plates")), str(_by_kind()), str(bridge.get_recovery_phase()), str(bridge.param_get("civ.factions"))])
		_log("  %-16s %s" % [c["name"], str(got[c["name"]])])
		await _shot("toggles_" + String(c["name"]).replace(" ", "_"))
	var on: Dictionary = got["all on"]
	_check(int(on["ways"]) > 0 and int(on["labels"]) > 0 and int(on["settlements"]) > 0,
		"positive control: with everything on there are ways, generated labels and settlements (%s)" % str(on))
	_check(int(got["routes off"]["ways"]) == 0, "Routes off: no ways (%d)" % int(got["routes off"]["ways"]))
	_check(int(got["routes off"]["settlements"]) > 0, "Routes off: settlements are still placed (%d)" % int(got["routes off"]["settlements"]))
	_check(int(got["names off"]["labels"]) == 0, "Names off: no generated labels (%d)" % int(got["names off"]["labels"]))
	_check(int(got["names off"]["ways"]) > 0, "Names off: roads are still generated (%d)" % int(got["names off"]["ways"]))
	_check(int(got["settlements off"]["settlements"]) == 0 and int(got["settlements off"]["ways"]) == 0,
		"Settlements off: no settlements and no ways (%s)" % str(got["settlements off"]))
	var m: Dictionary = got["master off"]
	_check(int(m["settlements"]) == 0 and int(m["ways"]) == 0 and int(m["landmarks"]) == 0 and int(m["labels"]) == 0,
		"Master off: terrain only, nothing generated by any pass (%s)" % str(m))
	## Back to the defaults for any later leg.
	_cb("master").button_pressed = true
	for k in ["settlements", "routes", "landmarks", "names"]:
		_cb(k).button_pressed = true


# -- layout -------------------------------------------------------------------------------------

func _leg_layout() -> void:
	await _ensure_open()
	_log("leg layout")
	var want := ["Plates", "Factions", "Village seeding (additive hamlets)", "Fixed settlement counts", "Recovery phase"]
	want.append_array(COUNT_LABELS)
	for t in want:
		var row := _row_control(String(t))
		var ok: bool = row != null and await _reachable(row)
		_check(ok, "'%s' is on the first screen of the form%s" % [t, "" if row != null else " (not built)"])
	await _shot("layout")


# -- presets ------------------------------------------------------------------------------------

func _ws(cls: String) -> Node:
	for w in app.get("_workspaces"):
		if (w.get_script() as Script).get_global_name() == cls:
			return w
	return null


## Settlements by kind, from the engine.
func _by_kind() -> Dictionary:
	var out := {}
	for st in bridge.settlements():
		var k := String((st as Dictionary).get("kind", "?"))
		out[k] = int(out.get(k, 0)) + 1
	return out


## The presets and the plate count REACH the engine: set through the form's own controls, Create, and read the engine back (the
## engine, never the control the value was typed into) and the WORLD dock (`_param_ctrls`, read from its own widgets).
func _leg_presets() -> void:
	await _ensure_open()
	_log("leg presets")
	_set_grid_seed()
	_cb("master").button_pressed = true
	for k in ["settlements", "routes", "landmarks", "names"]:
		_cb(k).button_pressed = (k == "settlements")
	dlg._plates_manual = true
	dlg._plates_value = 23
	dlg._refresh_plates()
	(dlg.get("_factions_spin") as SpinBox).value = 9
	dlg.recovery_input.select(2)
	dlg.recovery_input.item_selected.emit(2)
	dlg.get("_count_toggle").button_pressed = true
	var want := [2, 3, 4, 5, 6]
	var boxes: Array = dlg.get("_count_boxes")
	for i in 5:
		(boxes[i] as SpinBox).value = want[i]
	await _frames(2)
	await _create_and_wait()
	var snap: Dictionary = bridge.param_snapshot()
	_check(int(snap["tect.plates"]) == 23, "plates 23 reached the engine (%s)" % str(snap["tect.plates"]))
	var ws: Node = _ws("WorldWorkspace")
	var shown = null
	if ws != null and (ws.get("_param_ctrls") as Dictionary).has("tect.plates"):
		shown = ((ws.get("_param_ctrls") as Dictionary)["tect.plates"]["slider"] as HSlider).value
	_check(shown != null and absf(float(shown) - 23.0) < 0.5, "the WORLD dock's Plates control shows 23 (%s)" % str(shown))
	_check(int(snap["civ.factions"]) == 9 and bridge.get_factions().size() == 9, "9 factions reached the engine and the roster (%s, %d)" % [str(snap["civ.factions"]), bridge.get_factions().size()])
	_check(bool(snap["civ.fixed_counts"]) and int(snap["civ.n_capital"]) == 2 and int(snap["civ.n_city"]) == 3 and int(snap["civ.n_town"]) == 4
		and int(snap["civ.n_village"]) == 5 and int(snap["civ.n_hamlet"]) == 6, "the fixed counts 2/3/4/5/6 reached the engine")
	_check(bridge.get_recovery_phase() == 2, "recovery phase 2 reached the engine (%d)" % bridge.get_recovery_phase())
	var k := _by_kind()
	_log("  settlements by kind with fixed counts 2/3/4/5/6: %s" % str(k))
	var total := 0
	for v in k.values():
		total += int(v)
	_check(total > 0 and total <= 20, "the fixed counts bound the world (%d settlements, at most 20 asked)" % total)
	## Village seeding: the switch reaches the engine. (An effect contrast -- extra hamlets or addon tracks on against off -- was
	## tried and dropped: at this probe's 192-wide grid the pass adds none that a count or a way tally can tell from the
	## variation between two Creates, so it would have asserted nothing.)
	await _ensure_open()
	dlg.get("_count_toggle").button_pressed = false
	dlg.villages_check.button_pressed = true
	_cb("routes").button_pressed = true
	await _frames(2)
	await _create_and_wait()
	_check(bool(bridge.param_snapshot()["civ.villages"]), "village seeding reached the engine")
	await _ensure_open()
	dlg.villages_check.button_pressed = false
	(dlg.get("_factions_spin") as SpinBox).value = 6
	dlg.recovery_input.select(0)
	dlg.recovery_input.item_selected.emit(0)
	dlg._plates_manual = false
	dlg._refresh_plates()
	await _shot("presets")


# -- tech ---------------------------------------------------------------------------------------

func _leg_tech() -> void:
	await _ensure_open()
	_log("leg tech")
	var row := _row_control("Base technology")
	_check(row != null, "a 'Base technology' row is built")
	if row == null:
		return
	_check(await _reachable(row), "Base technology is on the first screen of the form")
	var phase := _row_control("Recovery phase")
	_check(phase != null and phase.get_parent() == row.get_parent(), "Base technology sits in the same section as Recovery phase")
	var ob: OptionButton = dlg.get("tech_input")
	_check(ob != null and ob.item_count == 6, "it lists the engine's six levels (%d)" % (ob.item_count if ob != null else -1))
	_check(ob != null and ob.selected == 1, "and starts on the reference's own default, Traditional Agrarian (index %d)" % (ob.selected if ob != null else -1))
	var want := {"subsistence": 0, "industrial": 5}
	for key in want:
		ob.select(int(want[key]))
		ob.item_selected.emit(int(want[key]))
		_set_grid_seed()
		_cb("master").button_pressed = true
		await _frames(2)
		await _create_and_wait()
		var snap: Dictionary = bridge.param_snapshot()
		_check(int(snap["civ.base_ag_tech"]) == int(want[key]), "%s: the engine holds civ.base_ag_tech %d (got %s)" % [key, int(want[key]), str(snap["civ.base_ag_tech"])])
		var techs := {}
		for f in bridge.get_factions():
			techs[String((f as Dictionary).get("ag_tech", "?"))] = true
		_check(techs.size() == 1 and techs.has(key), "%s: every faction in the roster has ag_tech '%s' (%s)" % [key, key, str(techs.keys())])
	await _shot("tech")
	ob.select(1)
	ob.item_selected.emit(1)


# -- preview ------------------------------------------------------------------------------------

## Renders `tex` through `mat` into a `w` x `h` offscreen viewport, one texel to one pixel, and returns the image: the shader's
## own output for every cell, with no on-screen scaling, anti-aliasing or other widget over it. (The on-screen frame is the
## screenshot; this is the measurement.)
func _render_cells(tex: Texture2D, mat: Material, w: int, h: int) -> Image:
	var sv := SubViewport.new()
	sv.size = Vector2i(w, h)
	sv.transparent_bg = false
	sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	var tr := TextureRect.new()
	tr.texture = tex
	tr.material = mat
	tr.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	tr.stretch_mode = TextureRect.STRETCH_SCALE
	tr.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	tr.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	sv.add_child(tr)
	add_child(sv)
	await _frames(4)
	var img := sv.get_texture().get_image()
	sv.queue_free()
	return img


func _luma(c: Color) -> float:
	return 0.299 * c.r + 0.587 * c.g + 0.114 * c.b


## One sample is one sample: three worlds, each judged by the same bars.
func _leg_preview() -> void:
	for sd in [4242, 51127, 776010]:
		await _preview_one(sd)


func _preview_one(seed_v: int) -> void:
	await _ensure_open()
	_log("leg preview, seed %d" % seed_v)
	_set_grid_seed(seed_v)
	await _frames(40)
	await _shot("preview_%d" % seed_v)
	var canvas := dlg.get("_preview_canvas") as TextureRect
	_check(canvas != null and canvas.visible and String(dlg.get("preview_state")) == "drawn", "a preview is drawn (state %s)" % String(dlg.get("preview_state")))
	var mask: PackedByteArray = dlg.get("preview_mask")
	var heights: PackedByteArray = dlg.get("preview_heights") if dlg.get("preview_heights") != null else PackedByteArray()
	_check(heights.size() == mask.size() and not heights.is_empty(),
		"the dialog holds a height byte for every preview cell (%d heights, %d mask cells)" % [heights.size(), mask.size()])
	if heights.size() != mask.size() or heights.is_empty():
		return
	var agree := 0
	for i in mask.size():
		if (heights[i] >= 128) == (mask[i] == 1):
			agree += 1
	_check(agree == mask.size(), "land is the same cells in both encodings (%d of %d agree)" % [agree, mask.size()])
	## The engine's own heightmap for the very same request, so the dialog is not judged against itself.
	var params: Dictionary = bridge.new_world_params_snapshot()
	params["seed"] = int(dlg.seed_input.value)
	params["width_km"] = float(dlg.width_input.value)
	params["grid_h"] = int(dlg.grid_h_input.value)
	params["archetype"] = String(dlg.get("_archetype"))
	params["world"] = dlg.extent_input.selected == 1
	params["tect.plates"] = dlg._classic_plates()
	var engine: PackedByteArray = bridge.preview_height_field(params, 320, int(dlg.grid_w_input.value))
	_check(engine == heights, "the dialog's heights equal the engine's preview_height_field for the same request (%d bytes)" % engine.size())
	## Time per preview, five runs each, median with min..max, run on an otherwise idle app: the engine call, and the dialog's own
	## work on its answer (`_draw_preview`: the mask pass and the texture upload). The shader's cost is on the GPU and is not here.
	var t_engine: Array = []
	var t_draw: Array = []
	for i in 5:
		var t0 := Time.get_ticks_usec()
		var b: PackedByteArray = bridge.preview_height_field(params, 320, int(dlg.grid_w_input.value))
		t_engine.append(float(Time.get_ticks_usec() - t0) / 1000.0)
		var t1 := Time.get_ticks_usec()
		dlg._draw_preview(b, true, 0.0)
		t_draw.append(float(Time.get_ticks_usec() - t1) / 1000.0)
	t_engine.sort()
	t_draw.sort()
	_log("  TIMING engine call %.0f ms (min %.0f, max %.0f); dialog draw %.1f ms (min %.1f, max %.1f); five runs each" % [t_engine[2], t_engine[0], t_engine[4], t_draw[2], t_draw[0], t_draw[4]])
	## What the shader draws, cell by cell.
	var w := 320
	var h := heights.size() / w
	var img: Image = await _render_cells(dlg.get("_preview_tex"), dlg.get("_preview_material"), w, h)
	var bins := {}
	var land_l := PackedFloat64Array()
	var land_h := PackedFloat64Array()
	var sea_l := 0.0
	var sea_n := 0
	var land_sum := 0.0
	var land_n := 0
	for y in h:
		for x in w:
			var i := y * w + x
			var l := _luma(img.get_pixel(x, y))
			var b := int(l * 32.0)
			bins[b] = int(bins.get(b, 0)) + 1
			if heights[i] >= 128:
				land_l.append(l)
				land_h.append(float(heights[i]))
				land_sum += l
				land_n += 1
			else:
				sea_l += l
				sea_n += 1
	## The positive control that the SHADER ran: the raw L8 texture is grey (R = G = B everywhere), which has plenty of luminance
	## levels and a perfect correlation with height, and is exactly what a failed shader compile shows (the first run of this
	## probe passed those two checks on a shader that had not compiled). The shaded picture is tinted: land is green to sand,
	## the sea is bluer than it is red.
	var tinted_land := 0
	var blue_sea := 0
	for y in h:
		for x in w:
			var c := img.get_pixel(x, y)
			if heights[y * w + x] >= 128:
				if maxf(c.r, maxf(c.g, c.b)) - minf(c.r, minf(c.g, c.b)) > 0.05:
					tinted_land += 1
			elif c.b > c.r + 0.05:
				blue_sea += 1
	_check(land_n > 0 and tinted_land * 10 >= land_n * 9, "the shader ran: %d of %d land cells are tinted, not grey" % [tinted_land, land_n])
	_check(sea_n > 0 and blue_sea * 10 >= sea_n * 9, "the shader ran: %d of %d sea cells are bluer than red" % [blue_sea, sea_n])
	var total := w * h
	var busy := 0
	for b in bins:
		if int(bins[b]) * 100 >= total:
			busy += 1
	_check(busy >= 6, "more than two colour levels: %d of 32 luminance bins each hold at least 1%% of the cells" % busy)
	## Pearson r between the land cells' brightness and their height byte.
	var mean_l := land_sum / maxf(1.0, float(land_n))
	var mean_h := 0.0
	for v in land_h:
		mean_h += v
	mean_h /= maxf(1.0, float(land_h.size()))
	var sxy := 0.0
	var sxx := 0.0
	var syy := 0.0
	for k in land_l.size():
		var dx := land_l[k] - mean_l
		var dy := land_h[k] - mean_h
		sxy += dx * dy
		sxx += dx * dx
		syy += dy * dy
	var r := sxy / sqrt(maxf(1e-12, sxx * syy))
	_log("  land brightness against height: r = %.3f over %d land cells; mean luminance land %.3f, sea %.3f" % [r, land_n, mean_l, sea_l / maxf(1.0, float(sea_n))])
	_check(absf(r) >= 0.5, "land brightness tracks height (|r| = %.3f, bar 0.5)" % absf(r))
	_check(land_n > 0 and sea_n > 0, "both land and sea are present in the sample world (%d land, %d sea)" % [land_n, sea_n])
	## The ocean side is shaded too: deeper is darker, by the same kind of measure.
	var sl := PackedFloat64Array()
	var sh := PackedFloat64Array()
	for y in h:
		for x in w:
			var i := y * w + x
			if heights[i] < 128:
				sl.append(_luma(img.get_pixel(x, y)))
				sh.append(float(heights[i]))
	var ml := 0.0
	var mh := 0.0
	for k in sl.size():
		ml += sl[k]
		mh += sh[k]
	ml /= maxf(1.0, float(sl.size()))
	mh /= maxf(1.0, float(sh.size()))
	var qxy := 0.0
	var qxx := 0.0
	var qyy := 0.0
	for k in sl.size():
		qxy += (sl[k] - ml) * (sh[k] - mh)
		qxx += (sl[k] - ml) * (sl[k] - ml)
		qyy += (sh[k] - mh) * (sh[k] - mh)
	var rs := qxy / sqrt(maxf(1e-12, qxx * qyy))
	_log("  sea brightness against depth byte: r = %.3f" % rs)
	_check(rs >= 0.5, "the sea is darker where it is deeper (r = %.3f, bar 0.5)" % rs)
