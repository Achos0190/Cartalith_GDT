extends Node
## The landmark filter in the real shell (owner, 2026-10-19: *"In the layers I'd
## also like an option to turn landmarks off globally or filter per type. The
## landmarks themselves should also scale according to zoom level."*).
##
##   Godot --path . _lmfilter_probe.tscn -- [--vp 1600x1000] [--force-touch] [--out DIR] [--phase set|restart]
##
## Run in a project copy with its own `config/name` (it reads and writes
## `cartalith_settings.cfg`'s `landmark_layer` section; `MISTAKES.md`: Godot
## 4.7 has no `--user-data-dir`). It clears that section on start and restores
## it on every exit path, except `--phase set`, which deliberately LEAVES
## `hidden_kinds = [<the kind it hid>]` and `zoom_law = false` behind so a second
## run with `--phase restart` can assert they came back after a relaunch (and
## then clears them).
##
## Default phase, in order -- the map's Layers popover reached by a click on the
## viewport's own Layers button, never by calling `open()`:
##   U1  the popover holds the filter under the Landmarks row, and expanding the
##       group (a click on its header) shows one row per kind with marks, each
##       label's count equal to `landmark_kind_counts()`, plus the hand-placed row;
##   U2  every tappable control in it is >= 44 px on BOTH axes when touch;
##   U3  Hide all (clicked) hides every ring, every generated POI glyph and the
##       hand-placed POI, persists the set, and a former ring no longer picks;
##   U4  Show all (clicked) restores the frame byte-for-byte;
##   U5  one kind's row (clicked) hides that kind only;
##   U6  the choice survives a regenerate + a new landmark pass;
##   U7  frames at opening x1/x4/x16/x64 centred on a landmark, law on, with the
##       drawn radius each time (screenshots in --out).
##
## Clicks are pushed into the SubViewport the shell lives in. On a phone the
## popover is a content-scaled sheet and `MISTAKES.md` records that synthetic
## taps cannot reach controls inside an embedded scaled window: when a click does
## not land, the probe says so (`TAP-MISS`) and presses the control's signal
## instead -- which tests the handler, not the touch path, and is labelled so.
##
## Windowed only. Result: `LMFILTER ok|FAIL <what>`, `LMFILTER ---- <n> checks,
## PASS|<n> FAILED ----`; exit 0 pass, 1 failure, 2 bad argument/headless, 3 watchdog.

const SEED := 552017

var app: Node
var _vp: SubViewport
var _fails := 0
var _checks := 0
var _out := ""
var _touch := false
var _phase := ""
var _tap_misses := 0


func _p(s: String) -> void:
	print("LMFILTER %s" % s)


func _check(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	_p("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _settled() -> Image:
	await _frames(4)
	var ov: Control = app.viewport.overlay
	var guard := 0
	while ov.pending_builds() > 0 and guard < 300:
		await get_tree().process_frame
		guard += 1
	await _frames(2)
	await RenderingServer.frame_post_draw
	return _vp.get_texture().get_image()


func _shot(name: String) -> void:
	var img: Image = await _settled()
	if _out != "":
		img.save_png(_out.path_join(name))
		_p("shot %s" % name)


## A click at `c`, a Control anywhere in the shell (main viewport or the embedded
## popover window), at its visible centre. Returns whether it landed.
func _click_ctrl(c: Control) -> bool:
	var at := _vp_point(c)
	var fired := [false]
	var cb := func(): fired[0] = true
	var b := c as BaseButton
	if b != null:
		b.pressed.connect(cb)
	for down in [true, false]:
		if _touch:
			var t := InputEventScreenTouch.new()
			t.index = 0
			t.pressed = down
			t.position = at
			_vp.push_input(t)
		var mm := InputEventMouseMotion.new()
		mm.position = at
		_vp.push_input(mm)
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		if _touch:
			ev.device = InputEvent.DEVICE_ID_EMULATION
		_vp.push_input(ev)
		await _frames(2)
	await _frames(3)
	if b != null:
		b.pressed.disconnect(cb)
		if not fired[0]:
			_tap_misses += 1
			_p("TAP-MISS %s at %s -- pressing its signal instead (handler, not the touch path)" % [c.name, at])
			if b is CheckBox or b.toggle_mode:
				b.button_pressed = not b.button_pressed
			else:
				b.pressed.emit()
			await _frames(3)
	return fired[0]


## `c`'s visible centre in the SubViewport's own coordinates, through an
## embedded window's position and content scale where it has one.
func _vp_point(c: Control) -> Vector2:
	var local := c.get_global_rect().get_center()
	var w := c.get_window()
	if w != null and w != _vp and w is Window and w.get_parent() != null:
		var sc := w.content_scale_factor if w.content_scale_factor > 0.0 else 1.0
		return Vector2(w.position) + local * sc
	return local


func _find(n: Node, pred: Callable) -> Node:
	if pred.call(n):
		return n
	for c in n.get_children(true):
		var r := _find(c, pred)
		if r != null:
			return r
	return null


func _all(n: Node, pred: Callable, out: Array) -> Array:
	if pred.call(n):
		out.append(n)
	for c in n.get_children(true):
		_all(c, pred, out)
	return out


func _clear_prefs() -> void:
	DccSettings.set_hidden_landmark_kinds(PackedStringArray())
	DccSettings.set_landmarks_visible(true)
	DccSettings.set_landmark_zoom_law(true)


func _finish() -> void:
	if _phase != "set":
		_clear_prefs()
	_p("tap misses (signal fallback used): %d" % _tap_misses)
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


func _ready() -> void:
	var vp_size := Vector2i(1600, 1000)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a == "--vp" and i + 1 < args.size():
			var wh := args[i + 1].split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
		elif a == "--out" and i + 1 < args.size():
			_out = args[i + 1]
			i += 2
		elif a == "--phase" and i + 1 < args.size():
			_phase = args[i + 1]
			i += 2
		elif a == "--force-touch":
			_touch = true
			i += 1
		else:
			_p("ABORT unknown argument '%s'" % a)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		_p("ABORT run windowed")
		get_tree().quit(2)
		return
	get_tree().create_timer(900.0).timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))
	if _phase != "restart":
		_clear_prefs()

	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.0).timeout

	if _phase == "restart":
		await _restart_leg()
		_finish()
		return

	var bridge = app.bridge
	await _generate()
	if not bridge.has_world:
		_check("a world generated", false)
		_finish()
		return
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_p("FORM %s at %s touch=%s" % [form, _vp.size, _touch])
	await _run(form)
	_finish()


func _generate() -> void:
	var bridge = app.bridge
	bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	await bridge.landmark_run()
	bridge.icon_generate({"family": "poi"})
	app.viewport.refresh_annotations()
	await _frames(6)


## The persisted state came back after a relaunch: the overlay already holds it
## before any world exists.
func _restart_leg() -> void:
	var ov: Control = app.viewport.overlay
	var want := DccSettings.hidden_landmark_kinds()
	_check("R1 the settings file carries what --phase set left", want.size() == 1 and not DccSettings.landmark_zoom_law(),
		"hidden=%s law=%s" % [want, DccSettings.landmark_zoom_law()])
	_check("R2 a fresh launch applies the hidden set", ov.hidden_landmark_kinds() == want, "%s" % [ov.hidden_landmark_kinds()])
	_check("R3 a fresh launch applies the zoom law switch", ov.landmark_zoom_law() == false)


func _run(form: String) -> void:
	var bridge = app.bridge
	var ov: Control = app.viewport.overlay
	var host = app.viewport
	var lms: Array = bridge.landmarks()
	## One hand-placed POI, so the hand-placed row has something to govern.
	var armed: bool = bridge.icon_arm("poi", 0, 1.0, 0.0, 0.0)
	var first: Dictionary = lms[0]
	var placed: int = bridge.icon_place(float(first["x"]) + 6.5, float(first["y"]) + 0.5)
	_p("hand-placed POI: arm=%s place=%d" % [armed, placed])
	host.refresh_annotations()
	await _frames(4)
	var counts: Dictionary = ov.landmark_kind_counts()
	_p("SURVEY landmarks=%d counts=%s" % [lms.size(), counts])
	## `icon_arm` needs a loaded asset pack, which a probe world has none of, so
	## a hand-placed POI usually cannot be stamped here; its row then reads 0.
	## The hand-placed behaviour is pinned pixel-for-pixel by `_lmzoom_probe` F3.
	_check("U0 landmarks exist", lms.size() > 0)
	## The busy overlay of the landmark pass fades out over a second or so and
	## dims the whole frame while it does; wait it out before the reference frame.
	await get_tree().create_timer(2.5).timeout
	await _park_mouse()
	var before: Image = await _settled()
	if _out != "":
		before.save_png(_out.path_join("lmfilter_%s_before.png" % form))

	## -- U1: open the popover the user's way --------------------------------
	var btn: Control = host._layers_btn
	await _click_ctrl(btn)
	await _frames(6)
	var pop: Window = app.layers_popover
	_check("U1 the Layers popover opened from its button", pop.visible)
	var panel: Node = _find(pop, func(n): return n.name == "LandmarkFilter")
	_check("U1 the landmark filter is in the popover", panel != null)
	if panel == null:
		return
	## Expand the collapsed group by clicking its header, as the user would.
	var group_btn: BaseButton = _find(pop, func(n): return n is BaseButton and String(n.get("text")).to_lower().contains("landmarks · by type"))
	if group_btn != null and not panel.is_visible_in_tree():
		await _click_ctrl(group_btn)
	await _frames(4)
	_check("U1 the group header exists and expanding it shows the panel", group_btn != null and panel.is_visible_in_tree())
	var rows: Array = _all(panel, func(n): return n is CheckBox and n.has_meta("landmark_kind"), [])
	var row_keys := {}
	var bad_counts: Array = []
	for cb: CheckBox in rows:
		var k := String(cb.get_meta("landmark_kind"))
		row_keys[k] = cb
		var lbl_text := ""
		for sib in cb.get_parent().get_children():
			if sib is Label:
				lbl_text = (sib as Label).text
				break
		if not lbl_text.ends_with("  %d" % int(counts.get(k, 0))):
			bad_counts.append("%s:'%s'" % [k, lbl_text])
	var missing := counts.keys().filter(func(k): return not row_keys.has(String(k)))
	_check("U1 every kind with marks has a row (and the hand-placed row)", missing.is_empty() and row_keys.has(ov.LANDMARK_MANUAL_POI),
		"rows=%d missing=%s" % [rows.size(), missing])
	_check("U1 every row's printed count equals landmark_kind_counts()", bad_counts.is_empty(), "%s" % [bad_counts])
	await _shot("lmfilter_%s_popover.png" % form)

	## -- U2: targets ----------------------------------------------------------
	var taps: Array = _all(panel, func(n): return n is BaseButton and (n as Control).is_visible_in_tree(), [])
	if group_btn != null:
		taps.append(group_btn)
	var small: Array = []
	for t: Control in taps:
		var s := t.get_global_rect().size
		if s.x < 44.0 or s.y < 44.0:
			small.append("%s %.0fx%.0f" % [t.name, s.x, s.y])
	if _touch:
		_check("U2 every tappable control is >= 44 px on both axes (%d controls)" % taps.size(), small.is_empty(), "%s" % [small])
	else:
		_p("U2 pointer density, sizes not floored: %d controls, under 44: %d" % [taps.size(), small.size()])

	## -- U3: Hide all -----------------------------------------------------------
	var rect: Rect2 = ov._displayed_rect()
	var interior: Rect2 = ov._interior_rect(rect)
	var li := -1
	for k in lms.size():
		var lp: Vector2 = ov._cell_to_screen(Vector2(lms[k]["x"], lms[k]["y"]), rect)
		if interior.has_point(lp) and int(ov._pick_mark(lp, interior, rect)["l"]) == k:
			li = k
			break
	var hide_all: BaseButton = _find(panel, func(n): return n.name == "LmHideAll")
	await _click_ctrl(hide_all)
	await _frames(4)
	var hidden_all: Image = await _settled()
	_check("U3 Hide all: no ring is drawn", ov._ringed_cells().is_empty())
	var vis_poi: Array = bridge.icon_list().filter(func(ic): return String(ic["family"]) == "poi" and not ov._icon_layer_hidden(ic))
	_check("U3 Hide all: no POI glyph (generated or hand-placed) is drawn", vis_poi.is_empty(), "%d still drawn" % vis_poi.size())
	_check("U3 Hide all: the frame changed", hidden_all.get_data() != before.get_data())
	if li >= 0:
		var lp2: Vector2 = ov._cell_to_screen(Vector2(lms[li]["x"], lms[li]["y"]), rect)
		_check("U3 Hide all: a former ring no longer picks", int(ov._pick_mark(lp2, interior, rect)["l"]) == -1)
	_check("U3 Hide all persisted", DccSettings.hidden_landmark_kinds().size() == row_keys.size(),
		"%d stored, %d rows" % [DccSettings.hidden_landmark_kinds().size(), row_keys.size()])
	if _out != "":
		hidden_all.save_png(_out.path_join("lmfilter_%s_hidden_all.png" % form))

	## -- U4: Show all -------------------------------------------------------------
	var show_all: BaseButton = _find(panel, func(n): return n.name == "LmShowAll")
	await _click_ctrl(show_all)
	await _frames(4)
	_check("U4 Show all empties the hidden set", ov.hidden_landmark_kinds().is_empty())
	## Close the popover so the frame compares like with like.
	pop.hide()
	await _park_mouse()
	var after_show: Image = await _settled()
	if _out != "":
		after_show.save_png(_out.path_join("lmfilter_%s_after_show_all.png" % form))
	_check("U4 Show all restores the frame byte-for-byte", after_show.get_data() == before.get_data(),
		_diff(before, after_show))

	## -- U5: one kind --------------------------------------------------------------
	var top := ""
	for k in counts:
		if String(k) != ov.LANDMARK_MANUAL_POI and (top == "" or int(counts[k]) > int(counts[top])):
			top = String(k)
	await _click_ctrl(btn)
	await _frames(6)
	panel = _find(pop, func(n): return n.name == "LandmarkFilter")
	group_btn = _find(pop, func(n): return n is BaseButton and String(n.get("text")).to_lower().contains("landmarks · by type"))
	if group_btn != null and not panel.is_visible_in_tree():
		await _click_ctrl(group_btn)
	var row: CheckBox = _find(panel, func(n): return n is CheckBox and n.has_meta("landmark_kind") and String(n.get_meta("landmark_kind")) == top)
	## The row may sit below the popover's fold; scroll it into view the way a
	## wheel or drag would (this is scrolling, not navigation to a screen).
	var sc: ScrollContainer = _find(pop, func(n): return n is ScrollContainer)
	if sc != null:
		sc.ensure_control_visible(row)
		await _frames(3)
	await _click_ctrl(row)
	await _frames(4)
	var drawn_kinds := {}
	for lm: Dictionary in ov._landmarks:
		if ov._landmark_shown(lm):
			drawn_kinds[String(lm["kind"])] = true
	_check("U5 '%s' row hides that kind and only it" % top,
		not drawn_kinds.has(top) and drawn_kinds.size() == counts.size() - 1 - (1 if counts.has(ov.LANDMARK_MANUAL_POI) else 0),
		"drawn kinds=%d of %d" % [drawn_kinds.size(), counts.size()])
	pop.hide()
	await _shot("lmfilter_%s_hid_%s.png" % [form, top])

	## -- U6: regenerate --------------------------------------------------------------
	await _generate()
	var still: Array = ov._landmarks.filter(func(lm): return String(lm["kind"]) == top and ov._landmark_shown(lm))
	_check("U6 after a regenerate + new pass, '%s' is still hidden" % top,
		ov.hidden_landmark_kinds() == PackedStringArray([top]) and still.is_empty() and ov._landmarks.size() > 0,
		"hidden=%s landmarks=%d" % [ov.hidden_landmark_kinds(), ov._landmarks.size()])

	## -- U7: zoom frames ----------------------------------------------------------------
	await get_tree().create_timer(2.5).timeout   ## the regenerate's busy overlay fades out
	var zr: float = ov._lod_zoom_base()
	var target := {}
	for lm: Dictionary in ov._landmarks:
		if ov._landmark_shown(lm) and String(lm["class"]) == "regional":
			target = lm
			break
	if target.is_empty():
		target = ov._landmarks[0]
	for mult in [1.0, 4.0, 16.0, 64.0]:
		await _centre_at(target, zr * mult)
		## Deep-zoom terrain tiles bake in the background; give them time so
		## the frame judges the marks against the terrain the user would see.
		await get_tree().create_timer(4.0).timeout
		var z: float = host.zoom()
		var r_px: float = ov._landmark_radius(target) * z
		_p("U7 x%-3d zoom %.2f  %s ring radius %.1f px (law on; old law %.1f px)" % [int(mult), z,
			String(target["kind"]), r_px, r_px / ov._landmark_zoom_k()])
		await _shot("lmfilter_%s_zoom_x%d.png" % [form, int(mult)])

	if _phase == "set":
		## Leave state behind for `--phase restart`.
		host.set_landmark_zoom_law(false)


## Moves the pointer to a corner of the frame with no map under it, so no hover
## card differs between two captures that are meant to be compared.
func _park_mouse() -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = Vector2(_vp.size) - Vector2(2, 2)
	_vp.push_input(mm)
	await _frames(3)


## How two frames differ: pixel count and bounding box (empty when equal size
## and identical).
func _diff(a: Image, b: Image) -> String:
	if a.get_size() != b.get_size():
		return "size %s vs %s" % [a.get_size(), b.get_size()]
	var n := 0
	var bb := Rect2i()
	for y in a.get_height():
		for x in a.get_width():
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
				bb = Rect2i(x, y, 1, 1) if n == 1 else bb.expand(Vector2i(x, y))
	return "%d px differ, bbox %s" % [n, bb]


## Zooms the real camera to `z` with `target` at the centre of the map host.
func _centre_at(target: Dictionary, z: float) -> void:
	var host = app.viewport
	var ov: Control = host.overlay
	host.reset_view()
	await _frames(3)
	var rect: Rect2 = ov._displayed_rect()
	var lp: Vector2 = ov._cell_to_screen(Vector2(target["x"], target["y"]), rect)
	var host_pt: Vector2 = (host as Control).get_global_transform_with_canvas().affine_inverse() * (ov.get_global_transform_with_canvas() * lp)
	host._zoom_at(host_pt, z / host.zoom())
	await _frames(2)
	host_pt = (host as Control).get_global_transform_with_canvas().affine_inverse() * (ov.get_global_transform_with_canvas() * lp)
	host._camera.position += (host as Control).size * 0.5 - host_pt
	host._update_lod()
	ov.view_changed()
	await _frames(4)
