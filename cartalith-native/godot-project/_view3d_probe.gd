extends Node
## **3D-1 acceptance probe** (`3D_VIEWPORT_SCOPE.md` §5, "3D-1 · The gate and
## an empty 3D mode"). What it protects, one leg per acceptance line:
##
## - **A. Lazy and inert until used.** Boot with a world: no `View3D` node
##   exists, the 2D frame is captured; `Window ▸ 3D view` builds and enters 3D;
##   leaving 3D gives back a 2D frame **byte-identical** to the one before.
## - **B. The probe runs and its positive controls fire** on this machine (VTF
##   and array controls move pixels; Basic/Standard/Full timed; layers read
##   back), and the result is stored with the device identity.
## - **C. The positive controls FAIL when VTF is stubbed out**
##   (`View3DGate.debug_stub_vtf`): both controls move 0 pixels, tier Off,
##   cause `vtf`, and the card says so.
## - **D. Each tier's disclosure renders**: every `classify()` cause, the
##   untested state and a forced state, written into the card and READ BACK
##   from screen pixels (ink inside the status label's drawn rect), each with
##   its measured number in the text.
## - **E. Forcing a tier then failing drops one tier without a crash**, through
##   the real `apply_force()` path with `debug_fail_from_tier`, plus the
##   warning dialog `request_force()` raises above the measured tier.
## - **F. `CommandIndex` counts the 3D rows with reasons**: every indexed row
##   whose title names 3D, and every unavailable one carries its `why`.
## - **G. Staleness**: a changed adapter / driver / Godot / probe version
##   re-measures; a matching identity does not.
## - **H. (touch only, `--force-touch`) the navpad's 3D pill** exists, clears
##   the touch floor on both axes, and switches the view both ways. Pressing by
##   `pressed.emit()` tests the handler, not a finger (`MISTAKES.md` row 155);
##   this leg is regression, not reachability.
##
## MUST run windowed (headless has no rasteriser and never fires
## `frame_post_draw`); refuses headless with exit 2. **Declares and clears**
## the `view3d` settings it asserts on at start and end (`MISTAKES.md`, "Trust
## a probe that reads `user://`").
##
##   godot --path . --resolution 1600x900 _view3d_probe.tscn -- --out DIR
##   godot --path . --resolution 1080x2340 _view3d_probe.tscn -- --force-touch --out DIR
##
## Flags (an unknown one aborts with exit 2): `--out DIR` (required, captures
## go there), `--force-touch` (read by `DccShell`; enables leg H).
## Result line: `RESULT checks=N fail=M`. Exit 0 all held, 1 a check failed,
## 2 could not run.

const ARGS: Array[String] = ["--out", "--force-touch"]
## The 3D-1 scripts carry no `class_name` (each user preloads them).
const View3DGate := preload("res://shell/view3d_gate.gd")
const View3D := preload("res://shell/view3d.gd")

var app: Node
var vp: ViewportHost
var _out := ""
var _checks := 0
var _fail := 0
var _touch := false

func _ok(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fail += 1
	print("%s  %s  %s" % ["PASS" if cond else "FAIL", name, detail])

func _parse() -> bool:
	var a := OS.get_cmdline_user_args()
	var i := 0
	while i < a.size():
		match a[i]:
			"--out":
				_out = a[i + 1] if i + 1 < a.size() else ""
				i += 1
			"--force-touch":
				_touch = true
			_:
				print("RESULT could not run: unknown argument %s -- accepted: %s" % [a[i], ", ".join(ARGS)])
				return false
		i += 1
	return _out != ""

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("RESULT could not run: headless has no rasteriser")
		get_tree().quit(2)
		return
	if not _parse():
		print("RESULT could not run: need --out DIR")
		get_tree().quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	DccSettings.set_view3d_result({})
	DccSettings.set_view3d_forced_tier(-1)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(0.8).timeout
	vp = app.viewport
	await _generate()
	app.open_project_dialog.hide()
	await _frames(10)
	await _leg_a_and_b()
	await _leg_c()
	await _leg_d()
	await _leg_e()
	_leg_f()
	_leg_g()
	if _touch:
		await _leg_h()
	DccSettings.set_view3d_result({})
	DccSettings.set_view3d_forced_tier(-1)
	print("RESULT checks=%d fail=%d" % [_checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

func _generate() -> void:
	var bridge = app.bridge
	bridge.generate({"seed": 483920, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.6).timeout

func _frames(n: int) -> void:
	for i in n:
		await RenderingServer.frame_post_draw

func _shot(name: String) -> Image:
	await _frames(3)
	var img := get_viewport().get_texture().get_image()
	img.save_png(_out.path_join(name))
	return img

func _wait_idle(v: View3D, limit_s := 120.0) -> void:
	var t0 := Time.get_ticks_msec()
	await _frames(2)
	while v.busy() and (Time.get_ticks_msec() - t0) < limit_s * 1000.0:
		await get_tree().process_frame

func _menu_popup(title: String) -> PopupMenu:
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == title:
			return (ch as MenuButton).get_popup()
	return null

func _bar_titles() -> String:
	var t := []
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton:
			t.append((ch as MenuButton).text)
	return ", ".join(t)

## The popup holding `Window ▸ 3D view`: the Window menu on PC and phone, or
## the Window submenu under ☰ on tablet. Found by the row's id, so the probe
## follows the row wherever the composition puts it.
func _window_popup() -> PopupMenu:
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton:
			var found := _find_id((ch as MenuButton).get_popup(), DccMenus.ID_WIN_VIEW_3D)
			if found != null:
				return found
	return null

func _find_id(p: PopupMenu, id: int) -> PopupMenu:
	if p.get_item_index(id) >= 0:
		return p
	for c in p.get_children(true):
		if c is PopupMenu:
			var f := _find_id(c as PopupMenu, id)
			if f != null:
				return f
	return null

func _diff_px(a: Image, b: Image) -> int:
	if a.get_size() != b.get_size():
		return -1
	var n := 0
	var da := a.get_data()
	var db := b.get_data()
	for i in range(0, da.size(), 4):
		if da[i] != db[i] or da[i + 1] != db[i + 1] or da[i + 2] != db[i + 2]:
			n += 1
	return n

## Ink inside a label's drawn rect: pixels whose luminance differs from the
## rect's median by more than 40 levels. Palette-agnostic (`MISTAKES.md`,
## "Assert on pixels"): it measures contrast against the label's own ground.
func _ink(img: Image, r: Rect2) -> int:
	var lums := PackedFloat32Array()
	var x0 := int(clampf(r.position.x, 0, img.get_width() - 1))
	var y0 := int(clampf(r.position.y, 0, img.get_height() - 1))
	var x1 := int(clampf(r.end.x, 0, img.get_width()))
	var y1 := int(clampf(r.end.y, 0, img.get_height()))
	for y in range(y0, y1):
		for x in range(x0, x1):
			lums.append(img.get_pixel(x, y).get_luminance() * 255.0)
	if lums.is_empty():
		return 0
	var s := lums.duplicate()
	s.sort()
	var med := s[s.size() / 2]
	var n := 0
	for v in lums:
		if absf(v - med) > 40.0:
			n += 1
	return n

# -- A + B ------------------------------------------------------------------------

func _leg_a_and_b() -> void:
	_ok("A: no View3D node before first entry", vp.view3d() == null and vp.get_node_or_null("View3D") == null)
	var before := await _shot("00_2d_before.png")
	var win := _window_popup()
	var idx := win.get_item_index(DccMenus.ID_WIN_VIEW_3D) if win != null else -1
	_ok("A: Window ▸ 3D view row exists", idx >= 0, "index %d · bar: %s" % [idx, _bar_titles()])
	if idx < 0:
		return
	_ok("A: its tooltip says untested before any run", win.get_item_tooltip(idx).find("not been tested") >= 0, win.get_item_tooltip(idx))
	var t0 := Time.get_ticks_msec()
	win.id_pressed.emit(DccMenus.ID_WIN_VIEW_3D)
	var v := vp.view3d()
	_ok("A: the row built and showed the 3D view", v != null and vp.is_3d())
	if v == null:
		return
	await _wait_idle(v)
	var ms := Time.get_ticks_msec() - t0
	var r := DccSettings.view3d_result()
	_ok("B: a result was stored", not r.is_empty(), "probe took %d ms wall" % ms)
	print("GATE ", JSON.stringify(r))
	_ok("B: VTF positive control moved pixels", bool(r.get("vtf", {}).get("pass", false)), "moved %.4f" % float(r.get("vtf", {}).get("moved", 0.0)))
	_ok("B: array positive control moved pixels", bool(r.get("array", {}).get("pass", false)), "moved %.4f" % float(r.get("array", {}).get("moved", 0.0)))
	var w: Dictionary = r.get("workloads", {})
	for k in w:
		var m: Dictionary = w[k]
		print("WORKLOAD %s frames=%d median=%.3f p95=%.3f min=%.3f max=%.3f bar=%.1f triangles=%d drawn=%d draws=%d" % [k, m["frames"], m["median"], m["p95"], m["min"], m["max"], m["bar"], m["triangles"], m.get("drawn_primitives", -1), m.get("draw_calls", -1)])
	for k in r.get("memory", {}):
		print("MEMORY %s %s" % [k, JSON.stringify(r["memory"][k])])
	_ok("B: tier at least Basic on this desktop", int(r.get("tier", 0)) >= View3DGate.TIER_BASIC, "tier %s cause '%s'" % [View3DGate.TIER_NAMES[int(r.get("tier", 0))], r.get("cause", "")])
	_ok("B: stored identity matches this device", View3DGate.stale_reason(r, View3DGate.device_identity()) == "")
	## Step 4's own controls: the magenta layer drawn is seen; magenta stored
	## on another layer while the last is drawn is not.
	var pos := await v.gate().memory_check(v.subviewport(), 64)
	var neg := await v.gate().memory_check(v.subviewport(), 64, 0)
	_ok("B: layer read-back sees the marked layer", bool(pos["ok"]) and bool(pos["read_back"]), JSON.stringify(pos))
	_ok("B: layer read-back refuses an unmarked layer", not bool(neg["ok"]) and not bool(neg["read_back"]) and bool(neg["allocated"]), JSON.stringify(neg))
	var img := await _shot("01_3d_measured.png")
	_ok("B: the card shows the disclosure", v._status.text == v.disclosure() and v._status.is_visible_in_tree(), v._status.text)
	_ok("B: the card's text is on screen", _ink(img, v._status.get_global_rect()) > 50, "ink %d" % _ink(img, v._status.get_global_rect()))
	_ok("A: the row is checked while 3D shows", win.is_item_checked(idx))
	vp.set_3d(false)
	var after := await _shot("02_2d_after.png")
	## Compared over the MAP's own rect (the viewport host), which is what
	## "2D is unchanged" means. The whole frame is printed as information
	## only: an OS tooltip raised by wherever the real cursor rests can land
	## on the docks between the two captures (seen once at 1080 x 2340, a
	## CIVIL rail tooltip over the left dock, 7 777 px, nothing in the map).
	var mr := Rect2i(Vector2i(vp.global_position), Vector2i(vp.size))
	var d := _diff_px(before.get_region(mr), after.get_region(mr))
	_ok("A: 2D map byte-identical after a 3D round trip", d == 0, "%d map pixels differ in %s; whole frame: %d" % [d, mr, _diff_px(before, after)])

# -- C ------------------------------------------------------------------------------

func _leg_c() -> void:
	var real := DccSettings.view3d_result()
	vp.set_3d(true)
	var v := vp.view3d()
	v.gate().debug_stub_vtf = true
	await v.retest()
	var r := DccSettings.view3d_result()
	_ok("C: stubbed VTF fails the VTF control", not bool(r["vtf"]["pass"]), "moved %.4f" % float(r["vtf"]["moved"]))
	_ok("C: stubbed VTF fails the array control", not bool(r["array"]["pass"]), "moved %.4f" % float(r["array"]["moved"]))
	_ok("C: tier Off, cause vtf", int(r["tier"]) == View3DGate.TIER_OFF and String(r["cause"]) == "vtf")
	_ok("C: no workload was timed after a failed control", (r["workloads"] as Dictionary).is_empty())
	var img := await _shot("03_3d_off_vtf_stubbed.png")
	_ok("C: the card says why", v._status.text.find("height textures") >= 0 and _ink(img, v._status.get_global_rect()) > 50, v._status.text)
	v.gate().debug_stub_vtf = false
	DccSettings.set_view3d_result(real)
	v._refresh()

# -- D ------------------------------------------------------------------------------

## A synthetic result of the given cause, built on this device's identity so
## the card treats it as current.
func _synthetic(cause: String) -> Dictionary:
	var r := View3DGate.device_identity()
	r["phone"] = false
	r["method"] = "gl_compatibility"
	r["api"] = "TestVendor 4.6"
	var wl := func(key: String, p95: float, aborted := false) -> Dictionary:
		return {"frames": 180, "median": p95 * 0.9, "p95": p95, "min": p95 * 0.5, "max": p95 * 1.2,
			"bar": View3DGate.bar_for(key), "aborted": aborted, "worst": 312.0 if aborted else p95 * 1.2,
			"triangles": View3DGate.triangles(key)}
	r["vtf"] = {"moved": 0.02, "pass": cause != "vtf"}
	r["array"] = {"moved": 0.02, "pass": cause != "array"}
	r["workloads"] = {
		"basic": wl.call("basic", 48.0 if cause == "basic_slow" else 0.9),
		"standard_desktop": wl.call("standard_desktop", 41.0 if cause == "standard_slow" else 1.4),
		"full": wl.call("full", 22.0 if cause == "full_slow" else 3.1),
	}
	r["memory"] = {"standard_desktop": {"ok": cause != "standard_memory"}, "full": {"ok": cause != "full_memory"}}
	return r

func _leg_d() -> void:
	var real := DccSettings.view3d_result()
	var v := vp.view3d()
	## cause -> [expected tier, a fragment the sentence must carry]
	var cases := {
		"vtf": [0, "height textures"], "basic_slow": [0, "48 ms"], "array": [1, "texture array"],
		"standard_slow": [1, "41 ms"], "standard_memory": [1, "256 terrain layers"],
		"full_slow": [2, "22 ms"], "full_memory": [2, "512 terrain layers"], "": [3, "3.1 ms"],
	}
	var i := 0
	for cause in cases:
		var r := _synthetic(cause)
		var c := View3DGate.classify(r)
		_ok("D: classify(%s) -> %s" % [cause if cause != "" else "full", View3DGate.TIER_NAMES[cases[cause][0]]],
			int(c["tier"]) == int(cases[cause][0]) and String(c["cause"]) == cause, str(c))
		DccSettings.set_view3d_result(r)
		v._refresh()
		var img := await _shot("04_disclosure_%02d_%s.png" % [i, cause if cause != "" else "full"])
		var ink := _ink(img, v._status.get_global_rect())
		_ok("D: disclosure renders (%s)" % cause, v._status.text.find(String(cases[cause][1])) >= 0 and ink > 50, "ink %d · %s" % [ink, v._status.text])
		i += 1
	## Untested and forced.
	DccSettings.set_view3d_result({})
	v._refresh()
	_ok("D: untested disclosure", v._status.text.find("not been tested") >= 0, v._status.text)
	DccSettings.set_view3d_result(_synthetic("standard_slow"))
	DccSettings.set_view3d_forced_tier(View3DGate.TIER_STANDARD)
	v._refresh()
	var img2 := await _shot("04_disclosure_%02d_forced.png" % i)
	_ok("D: forced disclosure names both tiers", v._status.text.begins_with("3D Standard, forced. The test chose Basic") and _ink(img2, v._status.get_global_rect()) > 50, v._status.text)
	var cnr := View3DGate.disclosure({"could_not_run": true, "why": "headless has no rasteriser"})
	_ok("D: could-not-run is Off with its reason", cnr.find("could not run") >= 0 and View3DGate.classify({"could_not_run": true})["tier"] == 0, cnr)
	DccSettings.set_view3d_forced_tier(-1)
	DccSettings.set_view3d_result(real)
	v._refresh()

# -- E ------------------------------------------------------------------------------

func _leg_e() -> void:
	var real := DccSettings.view3d_result()
	var v := vp.view3d()
	## Measured Basic (Standard too slow), so Standard and Full are both forces.
	var basic := _synthetic("standard_slow")
	DccSettings.set_view3d_result(basic)
	v._refresh()
	v.request_force(View3DGate.TIER_FULL)
	await _frames(3)
	var warn: ConfirmationDialog = v._warn
	## The text lives in `dialog_text` on desktop and in a body label on the
	## phone's own dialog form (`DccWidgets.confirm()`), so read both.
	var warn_text := ""
	if warn != null:
		warn_text = warn.dialog_text
		for l in warn.find_children("", "Label", true, false):
			warn_text += " " + (l as Label).text
		for l in warn.find_children("", "RichTextLabel", true, false):
			warn_text += " " + (l as RichTextLabel).get_parsed_text()
	_ok("E: forcing above the measured tier asks first", warn != null and warn.is_inside_tree() and warn_text.find("reset the graphics driver") >= 0,
		warn_text.strip_edges() if warn != null else "no dialog")
	var wimg := await _shot("05_force_warning.png")
	if warn != null and not warn.borderless:
		## The desktop body must actually draw its prose (a first cut drew the
		## title bar only). Sampled in the window's own rect of the capture.
		var wr := Rect2(Vector2(warn.position), Vector2(warn.size))
		var lbl := warn.get_label()
		_ok("E: the desktop warning draws its whole text", lbl.get_line_count() >= 3 and lbl.get_visible_line_count() >= lbl.get_line_count() and _ink(wimg, wr) > 300,
			"lines %d visible %d ink %d" % [lbl.get_line_count(), lbl.get_visible_line_count(), _ink(wimg, wr)])
	if warn != null:
		warn.hide()
		warn.queue_free()
	_ok("E: asking changed nothing yet", DccSettings.view3d_forced_tier() == -1)
	## Full fails, Standard holds.
	v.gate().debug_fail_from_tier = View3DGate.TIER_FULL
	await v.apply_force(View3DGate.TIER_FULL)
	_ok("E: forced Full that fails drops one tier to Standard", DccSettings.view3d_forced_tier() == View3DGate.TIER_STANDARD and v.tier() == View3DGate.TIER_STANDARD, v.disclosure())
	_ok("E: the drop is named on the card, with its measured frame and the limit", v._status.text.begins_with("Forced Full did not hold: one frame took 300 ms, over the 250 ms limit.") and v._status.text.find("dropped to Standard") >= 0, v._status.text)
	await _shot("06_forced_full_dropped.png")
	## Full and Standard both fail: two drops, back to the measured Basic, and
	## the force is cleared rather than stored as a "force" of the measured tier.
	DccSettings.set_view3d_forced_tier(-1)
	v.gate().debug_fail_from_tier = View3DGate.TIER_STANDARD
	await v.apply_force(View3DGate.TIER_FULL)
	_ok("E: two failures drop to the measured tier and clear the force", DccSettings.view3d_forced_tier() == -1 and v.tier() == View3DGate.TIER_BASIC, v.disclosure())
	_ok("E: both drops are named", v._status.text.count("did not hold") == 2, v._status.text)
	v.gate().debug_fail_from_tier = -1
	## A force at or below the measured tier applies at once, with no check.
	v.request_force(View3DGate.TIER_OFF)
	_ok("E: forcing Off applies at once", DccSettings.view3d_forced_tier() == View3DGate.TIER_OFF and v.tier() == View3DGate.TIER_OFF)
	v.request_force(-1)
	_ok("E: Automatic clears the force", DccSettings.view3d_forced_tier() == -1)
	DccSettings.set_view3d_result(real)
	v._refresh()
	vp.set_3d(false)
	_ok("E: still alive and back in 2D", not vp.is_3d())

# -- F ------------------------------------------------------------------------------

func _leg_f() -> void:
	var ci := CommandIndex.new()
	ci.build(app, app.bridge)
	var rows := []
	for r in ci.all():
		## A 3D row is one that names 3D in its title, or an unavailable row
		## whose reason is the 3D view (Anti-aliasing, Alternate frames).
		if String(r["title"]).find("3D") >= 0 or String(r["why"]).find("3D view") >= 0:
			rows.append(r)
	var unavailable := 0
	var with_reason := 0
	for r in rows:
		print("INDEX3D  %-28s kind=%-7s available=%s why=%s blurb=%s" % [r["title"], r["kind"], r["available"], String(r["why"]).left(80), String(r["blurb"]).left(80)])
		if not bool(r["available"]):
			unavailable += 1
			if String(r["why"]).strip_edges() != "":
				with_reason += 1
	var titles := rows.map(func(r): return String(r["title"]))
	_ok("F: 3D rows indexed", rows.size() >= 6, "%d rows, %d unavailable, %d with a reason" % [rows.size(), unavailable, with_reason])
	_ok("F: every unavailable 3D row carries a reason", unavailable == with_reason and unavailable >= 3)
	_ok("F: 3D view, Re-test 3D and the tier readout are indexed", titles.has("3D view") and titles.has("Re-test 3D") and titles.any(func(t): return String(t).begins_with("3D tier:")))
	var tier_row: Array = rows.filter(func(r): return String(r["title"]).begins_with("3D tier:"))
	_ok("F: the tier readout's blurb is the disclosure", tier_row.size() == 1 and String(tier_row[0]["blurb"]) == View3DGate.disclosure(DccSettings.view3d_result(), DccSettings.view3d_forced_tier()),
		String(tier_row[0]["blurb"]) if tier_row.size() == 1 else "missing")

# -- G ------------------------------------------------------------------------------

func _leg_g() -> void:
	var r := DccSettings.view3d_result()
	var id := View3DGate.device_identity()
	_ok("G: matching identity is current", View3DGate.stale_reason(r, id) == "")
	for k in ["adapter", "driver", "godot", "probe_version"]:
		var other := id.duplicate()
		other[k] = "something else"
		_ok("G: %s change re-measures" % k, View3DGate.stale_reason(r, other) == "%s changed" % k)
	_ok("G: never tested re-measures", View3DGate.stale_reason({}, id) == "never tested")

# -- H ------------------------------------------------------------------------------

func _leg_h() -> void:
	var b: Button = vp._view3d_btn
	if not DccTheme.is_phone():
		## The pill is phone-only; a tablet composition must not grow the
		## navpad past the map between its docks (the reason in `_build_navpad`).
		_ok("H: no 3D pill on the tablet composition", b == null)
		var row: Rect2 = vp._navpad.get_global_rect()
		_ok("H: the navpad stays inside the map", row.position.x >= vp.global_position.x, "row x %.0f, map x %.0f" % [row.position.x, vp.global_position.x])
		return
	_ok("H: the navpad has a 3D pill", b != null and b.is_visible_in_tree(), "")
	if b == null:
		return
	## The floor at this composition's own density: `_touch_scale` is what the
	## navpad itself was scaled by (phone: `phone_scale()`; tablet: 1.0).
	var floor_px := maxi(DccTheme.PHONE_TAP_MIN, int(round(44.0 * vp._touch_scale)))
	print("COMPOSITION phone=%s tablet=%s touch_scale=%.3f floor=%d" % [DccTheme.is_phone(), DccTheme.is_tablet(), vp._touch_scale, floor_px])
	await _frames(3)
	for c in vp._navpad.find_children("", "Button", true, false):
		print("NAVPAD %-12s size=%s min=%s" % [(c as Button).tooltip_text, (c as Button).size, (c as Button).custom_minimum_size])
	_ok("H: the pill clears the touch floor on both axes", b.size.x >= floor_px and b.size.y >= floor_px, "%s vs %d" % [b.size, floor_px])
	_ok("H: it reads 3D in 2D", b.text == "3D")
	b.pressed.emit()
	await _wait_idle(vp.view3d())
	_ok("H: pressing it opens 3D and relabels it 2D", vp.is_3d() and b.text == "2D")
	var img := await _shot("07_touch_3d.png")
	_ok("H: the 3D card is on screen on touch", _ink(img, vp.view3d()._status.get_global_rect()) > 50)
	## Not covered by the app bar: the card's top is inside the host's safe
	## top inset, which is where the phone's app bar ends.
	var top_ok := vp.view3d()._title.get_global_rect().position.y >= vp.global_position.y + float(vp._safe_insets.get("top", 0.0))
	_ok("H: the card starts below the app bar (inside the safe insets)", top_ok,
		"title y %.0f, inset top %.0f" % [vp.view3d()._title.get_global_rect().position.y, vp.global_position.y + float(vp._safe_insets.get("top", 0.0))])
	var back: Button = vp.view3d()._back_btn
	_ok("H: card buttons clear the touch floor", back.size.x >= floor_px and back.size.y >= floor_px, "%s" % back.size)
	b.pressed.emit()
	await _frames(3)
	_ok("H: pressing it again returns to 2D", not vp.is_3d() and b.text == "3D")
