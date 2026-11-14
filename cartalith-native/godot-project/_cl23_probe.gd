extends Node
## CL-2 and CL-3 (`CITY_MAP_LOOK_SCOPE.md` sections 4.2 and 4.3, Ruling EW item 3): the teardrop pin's hit test, and a
## click on a marker opening the information card with its Note section. Windowed (refuses --headless), under a wiped
## user directory (`APPDATA=<empty dir>`), a throw-away vault in TEMP.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _cl23_probe.tscn -- --vp 1600x1000
##   ... -- --vp 1600x1000 --force-touch                       (the touch tablet)
##   ... -- --vp 1080x2340 --density 2.8125 --force-touch      (the phone)
##
## Legs. A: the footprint (head, stem, a pixel outside, the 44 dp touch floor) and that `_hit_test_landmark`, `hits_at`
## and `_pick_mark` give one answer on 200 random points. B (desktop): a real left click on a pin opens the card for that
## one landmark with a NOTE section, first with no vault (a disabled row that says why), then with one (Create note...),
## the Create dialog makes the file on disk in the chosen folder and binds it, the pin turns from white to orange
## (framebuffer pixels), and a second click shows Open note, which shows the Note tab. C: an armed tool does not open
## the card and still receives its click. D (touch): a tap selects and shows the chips (Ruling EO), a hold opens the card
## or peek sheet with the Note section, whose Create makes the note. Exit 0 = held, 1 = a check failed, 2 = could not run.

const Pin := preload("res://shell/pin_marker.gd")
const NoteScript := preload("res://shell/context_note.gd")

var app: Node
var _vp: SubViewport
var _bridge
var _ov: Control
var _touch := false
var _checks := 0
var _fails := 0
var _root := ""
var _out := ""


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("CL23 %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("CL23 %s" % s)


func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _finish() -> void:
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


# ---- events ------------------------------------------------------------------------------------------------------

func _host(local: Vector2) -> Vector2:
	return _ov.get_global_transform_with_canvas() * local


func _press_at(at: Vector2, down: bool) -> void:
	if _touch:
		var t := InputEventScreenTouch.new()
		t.index = 0
		t.pressed = down
		t.position = at
		_vp.push_input(t)
		var em := InputEventMouseButton.new()
		em.button_index = MOUSE_BUTTON_LEFT
		em.pressed = down
		em.position = at
		em.global_position = at
		em.device = InputEvent.DEVICE_ID_EMULATION
		em.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
		_vp.push_input(em)
	else:
		var mm := InputEventMouseMotion.new()
		mm.position = at
		mm.global_position = at
		_vp.push_input(mm)
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		ev.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
		_vp.push_input(ev)


## A real click (or tap) at host point `at`.
func _click_host(at: Vector2) -> void:
	_press_at(at, true)
	await _frames(2)
	_press_at(at, false)
	await _frames(6)


## A real click (or tap) at `local` in the overlay's own space.
func _click(local: Vector2) -> void:
	await _click_host(_host(local))


## A held finger at `local`: down, past the hold (500 ms tablet, 480 ms phone), up.
func _hold(local: Vector2) -> void:
	var at := _host(local)
	_press_at(at, true)
	await get_tree().create_timer(0.8).timeout
	await _frames(3)
	_press_at(at, false)
	await _frames(8)


# ---- readouts ----------------------------------------------------------------------------------------------------

func _card():
	var b = app.get("context_broker")
	return b.card if b != null else null


func _card_open() -> bool:
	var c = _card()
	return c != null and c.visible


func _close_card() -> void:
	var c = _card()
	if c != null and c.visible:
		c.hide()
	var br = app.get("context_broker")
	if br != null and br.ring_is_open():
		br.ring_close()   ## the tablet's hold opens the cartwheel beside the card
	var pm = app.get("_phone_menu")
	if pm != null and pm.peek_card_is_open():
		pm._dismiss_peek_card()
	if app.get("element_chips") != null:
		app.element_chips.hide_chips()
	if DccTheme.is_phone():
		app.phone_hide_right_sheet()
		app._close_all_phone_overlays()
	_ov._touch_ring_active = false
	_ov._touch_swallow_up = false
	_ov._sample_pin = {}
	await _frames(3)


func _card_texts() -> Array:
	var out: Array = []
	var c = _card()
	if c != null:
		for r in c.drawn_rows():
			out.append("%s|%s" % [String(r.get("kind", "")), String(r.get("text", ""))])
	return out


func _find_row(kind: String, prefix: String) -> Variant:
	var c = _card()
	if c == null:
		return null
	for r in c.drawn_rows():
		if String(r.get("kind", "")) == kind and String(r.get("text", "")).begins_with(prefix):
			return r
	return null


func _detail_text() -> String:
	var c = _card()
	if c == null:
		return ""
	var d: Node = c.find_child("Detail", true, false)
	return (d as Label).text if d is Label else ""


func _vault_rows() -> Array:
	var out: Array = []
	for r in app.context_broker.last_actions:
		if String((r as Dictionary).get("section", "")) == "note":
			out.append(r)
	return out


## The colour of the framebuffer at overlay-local point `local` (a settled frame).
func _pixel(local: Vector2) -> Color:
	await _frames(4)
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	var at := _host(local)
	return img.get_pixelv(Vector2i(clampi(int(at.x), 0, img.get_width() - 1), clampi(int(at.y), 0, img.get_height() - 1)))


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join(name))
	_p("shot %s" % name)


# ---- run ---------------------------------------------------------------------------------------------------------

func _ready() -> void:
	var vp_size := Vector2i(1600, 1000)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--vp":
			var wh := String(args[i + 1]).split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
		elif a == "--density":
			i += 2
		elif a == "--out":
			_out = String(args[i + 1])
			i += 2
		elif a == "--force-touch":
			_touch = true
			i += 1
		else:
			_p("ABORT unknown argument %s" % a)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		_p("ABORT run windowed")
		get_tree().quit(2)
		return
	var wd := get_tree().create_timer(600.0)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(4)
	_vp.notification(Node.NOTIFICATION_VP_MOUSE_ENTER)
	_bridge = app.bridge
	_bridge.generate({"seed": 1200, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if not _bridge.has_world:
		_ok("a world generated", false)
		_finish()
		return
	_ov = app.viewport.overlay
	await _bridge.landmark_run()
	await _frames(6)
	## The busy card dims the map and swallows input while a run is on: wait for it to go (30 s cap), then settle.
	var t0 := Time.get_ticks_msec()
	while app.busy_overlay != null and app.busy_overlay.visible and Time.get_ticks_msec() - t0 < 30000:
		await _frames(10)
	await _frames(20)
	if DccTheme.is_phone():
		app.element_chips.hide_chips()
		app._close_all_phone_overlays()   ## the world sheet covers the lower half of the map
		await _frames(10)
	await _run()
	_finish()


func _run() -> void:
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_p("FORM %s at %s touch=%s" % [form, _vp.size, _touch])
	app.arm_tool("inspect")
	var lms: Array = _bridge.landmarks()
	_ok("L0 landmarks are placed and on the overlay", lms.size() > 0 and _ov._landmarks.size() == lms.size(), str(lms.size()))
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var host_rect: Rect2 = (app.viewport as Control).get_global_rect().grow(-80)
	if DccTheme.is_phone():
		## The phone keeps its GENERATE sheet over the lower half of the map and the app bar over the top: use the band between.
		host_rect = Rect2(80.0, 330.0, float(_vp.size.x) - 160.0, 700.0)
	## The MOST ISOLATED landmark: its tip and head inside the interior and on screen, chosen by the largest distance
	## to any other landmark tip or settlement, so a sweep around it is answered by it and not by a neighbour. It must
	## clear 2.5 head radii (a head and a neighbour's stem) or the leg refuses to run.
	var li := -1
	var r := 0.0
	var best_clear := 0.0
	for k in lms.size():
		var lm: Dictionary = lms[k]
		if not _ov._landmark_shown(lm):
			continue
		var tip: Vector2 = _ov._cell_to_screen(Vector2(float(lm["x"]), float(lm["y"])), rect)
		var rr: float = _ov._landmark_pin_r(lm)
		var head := tip + Vector2(0.0, -Pin.STEM_RATIO * rr)
		if not (interior.has_point(tip) and interior.has_point(head) and host_rect.has_point(_host(tip)) and host_rect.has_point(_host(head))):
			continue
		var clear := 100000.0
		for o in lms.size():
			if o != k:
				var op: Vector2 = _ov._cell_to_screen(Vector2(float(lms[o]["x"]), float(lms[o]["y"])), rect)
				clear = minf(clear, op.distance_to(tip))
		for s in _bridge.settlements():
			var sp: Vector2 = _ov._cell_to_screen(Vector2(float((s as Dictionary)["x"]), float((s as Dictionary)["y"])), rect)
			clear = minf(clear, sp.distance_to(tip) - 10.0)
		if clear > best_clear and clear >= 2.5 * rr + 4.0:
			best_clear = clear
			li = k
			r = rr
	_ok("L1 an isolated, on-screen landmark exists", li >= 0)
	if li < 0:
		return
	var lm: Dictionary = lms[li]
	var tip: Vector2 = _ov._cell_to_screen(Vector2(float(lm["x"]), float(lm["y"])), rect)
	var pad: float = _ov._landmark_pin_pad(r)
	var head := tip + Vector2(0.0, -Pin.STEM_RATIO * r)
	_p("target #%d kind=%s class=%s tip %s head r=%.2f pad=%.2f zoom=%.3f" % [li, lm["kind"], lm["class"], tip, r, pad, _ov._camera_zoom])

	await _leg_a(li, tip, r, pad, interior, rect)
	if _touch:
		await _leg_d(li, tip, r, form)
	else:
		await _leg_b(li, tip, r)
		await _leg_c(li, tip, r)


func _picks(p: Vector2, li: int, interior: Rect2, rect: Rect2) -> Array:
	var a: bool = int(_ov._hit_test_landmark(p, interior, rect)[0]) == li
	var b := false
	for h in _ov.hits_at(p):
		if String((h as Dictionary)["kind"]) == "landmark" and int((h as Dictionary)["id"]) == li:
			b = true
	var c: bool = int(_ov._pick_mark(p, interior, rect)["l"]) == li
	return [a, b, c]


func _leg_a(li: int, tip: Vector2, r: float, pad: float, interior: Rect2, rect: Rect2) -> void:
	print("== A. the footprint ==")
	var head := tip + Vector2(0.0, -Pin.STEM_RATIO * r)
	for pair in [["the head centre", head], ["the middle of the stem (0.75 r above the tip: outside the head even with its pad)", tip + Vector2(0.0, -0.75 * r)], ["the tip", tip],
			["1 px outside the head", head + Vector2(r + 1.0, 0.0)]]:
		var got := _picks(pair[1], li, interior, rect)
		_ok("A: %s picks in all three tests" % pair[0], got[0] and got[1] and got[2], str(got))
	var k: float = _ov._landmark_zoom_k()
	if not _touch:
		_ok("A: the pad is the pointer's 2 px (zoom law idle)", is_equal_approx(pad, Pin.HIT_PAD * k), "%.3f vs %.3f" % [pad, Pin.HIT_PAD * k])
		var out4 := _picks(head + Vector2(r + 4.0, 0.0), li, interior, rect)
		_ok("A: 4 px outside the head picks in none of the three", not out4[0] and not out4[1] and not out4[2], str(out4))
		var up4 := _picks(head + Vector2(0.0, -(r + 4.0)), li, interior, rect)
		_ok("A: 4 px above the head picks in none", not up4[0] and not up4[1] and not up4[2], str(up4))
	else:
		var unit := DccTheme.phone_scale() if DccTheme.is_phone() else 1.0
		var across: float = 2.0 * (r + pad) * _ov._camera_zoom
		_ok("A: the touch target is at least 44 dp across (%.1f px vs %.1f)" % [across, 44.0 * unit], across >= 44.0 * unit - 0.5)
		var edge := _picks(head + Vector2(r + pad - 0.5, 0.0), li, interior, rect)
		_ok("A: just inside the touch floor picks", edge[0] and edge[1], str(edge))
		var beyond := _picks(head + Vector2(r + pad + 2.0, 0.0), li, interior, rect)
		_ok("A: just beyond the touch floor picks in none", not beyond[0] and not beyond[1], str(beyond))
	## 200 random points in a box around the pin: the three tests agree, and the sweep is not vacuous.
	var rng := RandomNumberGenerator.new()
	rng.seed = 23
	var disagree := 0
	var yes := 0
	var no := 0
	for _n in 200:
		var p := head + Vector2(rng.randf_range(-3.0, 3.0), rng.randf_range(-2.0, 4.0)) * r
		var g := _picks(p, li, interior, rect)
		## A point a neighbouring mark also contains is not this pin's alone: `_pick_mark` may rightly give it away.
		var foreign := false
		for h in _ov.hits_at(p):
			if not (String((h as Dictionary)["kind"]) == "landmark" and int((h as Dictionary)["id"]) == li):
				foreign = true
		if foreign:
			continue
		if g[0] != g[1] or (g[2] and not g[0]) or (g[0] and not g[2]):
			disagree += 1
		if g[0]:
			yes += 1
		else:
			no += 1
	_ok("A: 200 random points, the three tests agree (disagreements %d)" % disagree, disagree == 0)
	_ok("A: the sweep hit and missed (hit %d, missed %d)" % [yes, no], yes > 10 and no > 10)
	await _shot("cl23_pins_%s.png" % ("touch" if _touch else "desk"))


func _leg_b(li: int, tip: Vector2, r: float) -> void:
	print("== B. a left click on a pin ==")
	var head := tip + Vector2(0.0, -Pin.STEM_RATIO * r)
	_vp.warp_mouse(Vector2.ZERO)
	var back0: int = app._selection_back.size()
	var n_actions := 0
	## No vault yet: the card opens and the Note section says why there is no note.
	await _click(head)
	_ok("B1: a left click on the pin head opens the card", _card_open())
	var lr: Dictionary = app.context_broker.last_request
	_ok("B1: the request is about that one landmark, source click", (lr.get("hits", []) as Array).size() == 1
		and String(lr["hits"][0]["kind"]) == "landmark" and int(lr["hits"][0]["id"]) == li and String(lr.get("source", "")) == "click", str(lr.get("hits", [])))
	_ok("B1: the click selected it, exactly one history step", app.selection == {"kind": "landmark", "id": li}
		and app._selection_back.size() == back0 + 1, "%s back %d->%d" % [app.selection, back0, app._selection_back.size()])
	_ok("B1: the card has a NOTE band", _find_row("band", "NOTE") != null, str(_card_texts()))
	var vr := _vault_rows()
	_ok("B1: with no vault the note row is disabled and says why", vr.size() == 1 and not bool(vr[0].get("enabled", true))
		and String(vr[0].get("reason", "")).contains("No vault folder"), str(vr))
	await _close_card()

	## A vault: the card offers Create note.
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-cl23-%s-%d" % ["desk", Time.get_ticks_msec()]
	_write(_root + "/Lore/Old.md", "# Old\n")
	_write(_root + "/Places/Other.md", "# Other\n")
	_ok("B2: the throw-away vault binds", bool(_bridge.vault_connect(_root, "Chronicle").get("ok", false)))
	_ov.invalidate_pin_notes()
	var white := await _pixel(head + Vector2(r * 0.85, 0.0))
	_ok("B2: the pin is white with no linked note (%s)" % white, white.r > 0.92 and white.g > 0.9 and white.b > 0.86)
	await _click(head)
	_ok("B2: the card opens again", _card_open())
	var cr = _find_row("action", "Create note")
	_ok("B2: the Note section offers Create note...", cr != null, str(_card_texts()))
	_ok("B2: its detail says the landmark has no note yet", _detail_text().contains("no note yet"), _detail_text())
	await _shot("cl23_card_%s.png" % ("touch" if _touch else "desk"))
	if cr == null:
		return

	## Create: the dialog, the folder chooser, the file on disk, the binding, the pin turning orange.
	var c = _card()
	await _click_host((cr["node"] as Control).get_global_rect().get_center() + Vector2(c.position))
	await _frames(6)
	var note = app.context_broker._note
	var d: Dictionary = note._last_dialog
	_ok("B3: pressing it opens the Create note dialog", not d.is_empty() and (d["dialog"] as Window).visible)
	if d.is_empty():
		return
	var fp: Dictionary = d["chooser"]
	_ok("B3: the dialog has the folder chooser (Lore is offered)", (fp["tree"] as Tree).get_root() != null)
	(fp["select"] as Callable).call("Lore")
	await _frames(3)
	var edit := d["field"] as LineEdit
	_ok("B3: choosing Lore rewrites the path into Lore/", edit.text.begins_with("Lore/"), edit.text)
	var rel := edit.text
	var eid := int((_bridge.landmark_names()[li] as Dictionary)["id"])
	((d["dialog"] as ConfirmationDialog).get_ok_button()).pressed.emit()
	await _frames(8)
	_ok("B3: the note is on disk in Lore", FileAccess.file_exists(_root + "/" + rel), _root + "/" + rel)
	var links: Array = _bridge.vault_links_for("landmark", eid)
	_ok("B3: it is bound to the landmark", links.size() == 1 and String((links[0] as Dictionary).get("path", "")) == rel, str(links))
	_ok("B3: the overlay now says the landmark has a note", _ov.landmark_has_note(li))
	await _close_card()
	var orange := await _pixel(head + Vector2(r * 0.85, 0.0))
	_ok("B3: the pin is orange (%s)" % orange, orange.r > 0.85 and orange.g > 0.42 and orange.g < 0.7 and orange.b < 0.35)
	await _shot("cl23_orange_desk.png")

	## Open: the card shows the note, and Open shows the Note tab.
	await _click(head)
	var orow = _find_row("action", "Open note")
	_ok("B4: the card now offers Open note with the note's title", orow != null and String(orow["text"]).contains(String((_bridge.landmark_names()[li] as Dictionary)["name"])), str(_card_texts()))
	_ok("B4: the snippet line is present", _detail_text() != "", _detail_text())
	if orow != null:
		c = _card()
		await _click_host((orow["node"] as Control).get_global_rect().get_center() + Vector2(c.position))
		await _frames(8)
		_ok("B4: Open shows the Context panel's Note tab", String(app.right_dock_ctrl._tab) == "note", str(app.right_dock_ctrl._tab))
		_ok("B4: the note is the one created", String(app.vault_window.docked_path()) == rel or app.vault_window.visible, str(app.vault_window.docked_path()))
	await _close_card()


## The head centre of landmark li in the overlay's CURRENT layout (the map can move when a tool changes the bars around it).
func _head_now(li: int) -> Vector2:
	var lm: Dictionary = _bridge.landmarks()[li]
	var tip: Vector2 = _ov._cell_to_screen(Vector2(float(lm["x"]), float(lm["y"])), _ov._displayed_rect())
	return tip + Vector2(0.0, -Pin.STEM_RATIO * _ov._landmark_pin_r(lm))


func _leg_c(li: int, tip: Vector2, r: float) -> void:
	print("== C. an armed tool does not open the card ==")
	var head := tip + Vector2(0.0, -Pin.STEM_RATIO * r)
	app.arm_tool("settlement")
	await _frames(6)
	## Arming a tool changes the tool-options bar, which can move the map: the pin is found again in the CURRENT layout, and a
	## control says a marker really is under the click (a click on bare ground would pass this leg for any gate).
	head = _head_now(li)
	_ok("C1 control: a marker is under the click point with Settlement armed", _ov.marker_at(head).get("kind", "") == "landmark")
	var got := {"n": 0}
	var had: bool = app._click_handlers.has("settlement")
	var orig: Callable = app._click_handlers["settlement"] if had else Callable()
	if had:
		app._click_handlers["settlement"] = func(_gx: float, _gy: float) -> void: got["n"] += 1
	var before: Dictionary = app.context_broker.last_request
	await _click(head)
	_ok("C1: with Settlement armed the click does not open the card", not _card_open())
	_ok("C1: and issued no card request", app.context_broker.last_request == before)
	_ok("C1: the armed tool still received the click", had and int(got["n"]) == 1, str(got))
	if had:
		app._click_handlers["settlement"] = orig
	app.arm_tool("inspect")
	await _frames(6)
	head = _head_now(li)
	await _click(head)
	_ok("C2: control, pointer armed again, the same click opens the card", _card_open())
	await _close_card()
	## A plain click on bare ground opens nothing.
	var bare: Vector2 = tip + Vector2(r * 12.0, r * 6.0)
	if _ov._interior_rect(_ov._displayed_rect()).has_point(bare) and _ov.marker_at(bare).is_empty():
		await _click(bare)
		_ok("C3: a click on bare ground opens no card", not _card_open())
		await _close_card()


func _leg_d(li: int, tip: Vector2, r: float, form: String) -> void:
	print("== D. touch: a tap and a hold ==")
	var head := tip + Vector2(0.0, -Pin.STEM_RATIO * r)
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-cl23-touch-%d" % Time.get_ticks_msec()
	_write(_root + "/Lore/Old.md", "# Old\n")
	_ok("D0: the throw-away vault binds", bool(_bridge.vault_connect(_root, "Chronicle").get("ok", false)))
	_ov.invalidate_pin_notes()
	await _frames(4)
	var white := await _pixel(head + Vector2(r * 0.85, 0.0))
	_ok("D1: the pin is white with no note (%s)" % white, white.r > 0.92 and white.g > 0.9 and white.b > 0.86)
	await _tap_it(head)
	_ok("D1: a tap selects the landmark", app.selection == {"kind": "landmark", "id": li}, str(app.selection))
	_ok("D1: a tap shows the chips (Ruling EO), not the card", app.element_chips.visible and not _card_open()
		and not (app.get("_phone_menu") != null and app._phone_menu.peek_card_is_open()))
	var chip = app.element_chips.find_child("ChipNote", true, false)
	_ok("D1: the chips include Note", chip != null)
	await _shot("cl23_tap_%s.png" % form)
	await _close_card()
	await get_tree().create_timer(1.2).timeout   ## past the double-tap window, or the tap and the hold read as one double tap
	await _hold(head)
	var rows := _vault_rows()
	if rows.is_empty():
		_ok("D2: a hold builds the Note section", false, "no note rows: card open %s, hits %s, ids %s" % [_card_open(), str(app.context_broker.last_request.get("hits")), str(app.context_broker.last_actions.map(func(a): return a.get("id")))])
		return
	_ok("D2: a hold builds the card's Note section (Create note...)", rows.size() == 1 and String(rows[0]["label"]).begins_with("Create note"), str(rows))
	if DccTheme.is_phone():
		var pm = app._phone_menu
		_ok("D2: the peek sheet is open", pm.peek_card_is_open())
		pm._set_peek_detent("half", false)
		await _frames(4)
		var labels = pm.peek_full_row_labels()
		_ok("D2: the sheet's list has Create note...", "Create note…" in labels, str(labels))
		await _shot("cl23_peek_%s.png" % form)
		await pm._dismiss_peek_card()
	else:
		_ok("D2: the card is open with a NOTE band", _card_open() and _find_row("band", "NOTE") != null, str(_card_texts()))
		await _shot("cl23_card_%s.png" % form)
	## Create through the row's own callable (the sheet is a separate widget set; the tablet's card row is the same callable).
	(rows[0]["callable"] as Callable).call()
	await _frames(8)
	var d: Dictionary = app.context_broker._note._last_dialog
	_ok("D3: Create note opens the dialog", not d.is_empty() and (d["dialog"] as Window).visible)
	if d.is_empty():
		return
	var fp: Dictionary = d["chooser"]
	(fp["select"] as Callable).call("Lore")
	var rel := String((d["field"] as LineEdit).text)
	var eid := int((_bridge.landmark_names()[li] as Dictionary)["id"])
	_ok("D3: the touch dialog's controls are at least 44 px", (fp["new_button"] as Button).custom_minimum_size.y >= 44.0 and (d["field"] as LineEdit).custom_minimum_size.y >= 44.0)
	((d["dialog"] as ConfirmationDialog).get_ok_button()).pressed.emit()
	await _frames(8)
	_ok("D3: the note is on disk in Lore", FileAccess.file_exists(_root + "/" + rel), rel)
	_ok("D3: and bound to the landmark", _bridge.vault_links_for("landmark", eid).size() == 1)
	await _close_card()
	var orange := await _pixel(head + Vector2(r * 0.85, 0.0))
	_ok("D3: the pin is orange (%s)" % orange, orange.r > 0.85 and orange.g > 0.42 and orange.g < 0.7 and orange.b < 0.35)
	await _shot("cl23_orange_%s.png" % form)
	await get_tree().create_timer(1.2).timeout
	await _hold(head)
	rows = _vault_rows()
	_ok("D4: after the create a hold shows Open note · title", rows.size() == 1 and String(rows[0]["label"]).begins_with("Open note"), str(rows))
	await _close_card()


## A tap: touch then lift, quickly (under the hold).
func _tap_it(local: Vector2) -> void:
	var at := _host(local)
	_press_at(at, true)
	await _frames(3)
	_press_at(at, false)
	await _frames(10)
