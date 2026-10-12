extends Node
## **Does the New World dialog fit and stay reachable at TABLET size?**
##
## Protects: the desktop/tablet form of the New World card (`new_world_dialog.gd`, `DESKTOP_SIZE`
## 1000 x 780, `DESKTOP_MIN_SIZE` 960 x 720, two columns with the seed preview left). The four
## existing `_nw*` probes (`_nwclip`, `_nwsize`, `_nwaction`, `_vfy_nwcanvas`) are phone-only and
## abort on a tablet frame, so before this probe nothing drew the dialog at 1600 x 2560 or
## 2560 x 1600 (`OUTSTANDING_WORK.md`, tablet-density row; `STATUS.md` 2026-10-12).
##
## It asserts, per frame:
##   1. the touch composition is the TABLET one (not phone, touch on) -- anything else aborts,
##      because a green run that built the wrong card measured nothing (`MISTAKES.md`, a probe
##      passing vacuously);
##   2. the dialog window lies wholly inside the viewport it is embedded in;
##   3. the Generate (OK) and Cancel buttons are visible and DRAWN WHOLE -- the control's rect
##      narrowed by every clipping ancestor, not `get_global_rect()`, which reports the unclipped
##      rect whatever a scroller does with it (the blind spot `_nwclip_probe.gd` documents);
##   4. the seed preview is present: its canvas is visible with a texture, drawn whole inside the
##      window and larger than a token (a preview a few pixels tall is not "present");
##   5. a positive control: a button parked in a 1 px clipping band is judged NOT drawn whole by
##      the same helper, so check 3 can fail.
##
## It does not press anything (synthetic input cannot reach a control inside an embedded dialog;
## `_nwaction_probe.gd`'s header carries the measurement); it stages the dialog with
## `app.open_new_world()` and everything after is geometry.
##
##   godot --path . res://_nwtablet_probe.tscn -- --force-touch --vp 1600x2560 --tag t1600
##   godot --path . res://_nwtablet_probe.tscn -- --force-touch --vp 2560x1600 --tag t2560
##
## Flags: `--vp WxH`, `--tag NAME`, `--force-touch` (required). Any other `--flag` aborts.
## Exit codes: 0 all legs passed, 1 a leg failed, 2 could not run and asserted nothing.

var app: Node
var _vp: SubViewport
var _tag := "nwtablet"
var _fail := 0


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


## The control's own rect narrowed by every ancestor that clips (`clip_contents`), stopping at the
## Window. Zero size when nothing survives. Same rule as `_nwclip_probe.gd::_painted_rect`.
func _painted_rect(c: Control) -> Rect2:
	var r := c.get_global_rect()
	var q: Node = c.get_parent()
	while q != null and not (q is Window):
		if q is Control and (q as Control).clip_contents:
			r = r.intersection((q as Control).get_global_rect())
			if r.size.x <= 0.0 or r.size.y <= 0.0:
				return Rect2(r.position, Vector2.ZERO)
		q = q.get_parent()
	return r


## True when nothing of `c` is clipped away: painted rect equals its own rect, which is non-empty.
func _whole(c: Control) -> bool:
	var full := c.get_global_rect()
	var seen := _painted_rect(c)
	return full.size.x > 0.0 and full.size.y > 0.0 \
		and absf(seen.size.x - full.size.x) < 1.0 and absf(seen.size.y - full.size.y) < 1.0


func _ready() -> void:
	_tag = _arg("--tag", "nwtablet")
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--force-touch", "--vp", "--tag"]):
			_log("ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	var parts: PackedStringArray = _arg("--vp", "1600x2560").split("x")
	if parts.size() != 2 or not ("--force-touch" in OS.get_cmdline_user_args()):
		_log("ABORT needs --vp WxH and --force-touch (without it the shell is the desktop one)")
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
	_log("viewport %dx%d phone=%s tablet=%s touch=%s" % [_vp.size.x, _vp.size.y, app.is_phone(), DccTheme.is_tablet(), DccTheme.is_touch()])
	if app.is_phone() or not DccTheme.is_tablet() or not DccTheme.is_touch():
		_log("ABORT this run is not the touch TABLET composition; nothing was measured")
		get_tree().quit(2)
		return

	app.open_new_world()
	await _frames(30)
	var dlg: AcceptDialog = app.new_world_dialog
	_check(dlg != null and dlg.visible, "the New World dialog is up")
	if dlg == null or not dlg.visible:
		get_tree().quit(1)
		return

	# 2. the window inside the viewport it is embedded in.
	var win := Rect2(Vector2(dlg.position), Vector2(dlg.size))
	var vpr := Rect2(Vector2.ZERO, Vector2(_vp.size))
	_log("  window %s inside viewport %s" % [str(win), str(vpr)])
	_check(win.size.x >= 900.0 and win.size.y >= 700.0, "the window kept a usable size (%s)" % str(win.size))
	_check(vpr.encloses(win), "the window lies wholly inside the viewport")

	# 3. Generate and Cancel reachable.
	var ok_btn := dlg.get_ok_button()
	var cancel_btn: Button = dlg.get("_desktop_cancel")
	var win_local := Rect2(Vector2.ZERO, Vector2(dlg.size))
	for pair in [[ok_btn, "Generate"], [cancel_btn, "Cancel"]]:
		var b := pair[0] as Button
		_check(b != null and b.is_visible_in_tree(), "%s button is visible" % pair[1])
		if b != null:
			var r := b.get_global_rect()
			_log("    %s rect %s painted %s" % [pair[1], str(r), str(_painted_rect(b))])
			_check(_whole(b), "%s is drawn WHOLE without scrolling" % pair[1])
			_check(win_local.encloses(r), "%s lies inside the dialog window" % pair[1])
			_check(r.size.y >= 24.0, "%s has a real height (%.0f)" % [pair[1], r.size.y])

	# 4. the preview is present (drawn by the engine's early stages; allow it a moment).
	var canvas: TextureRect = dlg.get("_preview_canvas")
	var spins := 0
	while canvas != null and not (canvas.visible and canvas.texture != null) and spins < 60:
		await _frames(5)
		spins += 1
	_check(canvas != null, "the preview canvas node exists")
	if canvas != null:
		_check(canvas.visible and canvas.texture != null, "the preview is drawn (visible with a texture, after %d frames)" % (spins * 5))
		var cr := canvas.get_global_rect()
		_log("    preview rect %s painted %s" % [str(cr), str(_painted_rect(canvas))])
		_check(cr.size.x >= 200.0 and cr.size.y >= 100.0, "the preview is a usable size (%.0f x %.0f)" % [cr.size.x, cr.size.y])
		_check(_whole(canvas), "the preview is drawn WHOLE")
		_check(win_local.encloses(cr), "the preview lies inside the dialog window")

	# 5. positive control: the helper must be able to say no.
	var band := ScrollContainer.new()
	band.custom_minimum_size = Vector2(40, 1)
	band.size = Vector2(40, 1)
	dlg.add_child(band)
	var parked := Button.new()
	parked.custom_minimum_size = Vector2(40, 40)
	band.add_child(parked)
	await _frames(4)
	_check(not _whole(parked), "control: a button parked in a 1 px clipping band is NOT drawn whole")
	band.queue_free()

	_log("RESULT %s fail=%d" % [_tag, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
