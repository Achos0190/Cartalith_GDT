extends Node
## **Ruling AS: the phone's New World warns about memory above Ruling AH's
## ceiling, and generates only once the warning is confirmed.**
##
## Three cases, one per run mode:
##   phone   + 2048 (region aspect, 2048 x 1311)  -> no question; generates at once
##   phone   + 4096 (region aspect)               -> a question quoting ~2.41 GiB;
##                                                  Cancel generates nothing and
##                                                  brings the form back; OK generates
##   desktop + 4096                               -> no question; generates at once
##
## **Generation is recorded, not run.** A 4096 generate is ~2.4 GiB and minutes
## of work, so the dialog's `bridge` is swapped for `RecBridge`, an
## `EngineBridge` subclass whose `generate()` only appends the request. The grid
## is sized BEFORE the swap, because `_derived_grid_h()` asks the real engine
## for the region aspect's row count. Create is pressed by a STAGING CALL
## (`_phone_create.pressed.emit()` on the phone, `confirmed.emit()` on desktop)
## -- `_nwaction_probe.gd`'s header records why synthetic input cannot reach a
## phone-presented `AcceptDialog`.
##
## Logic and layout only, so headless is fine:
##   godot --headless --path . _nwmem_probe.tscn -- --force-touch --tag phone
##   godot --headless --path . _nwmem_probe.tscn -- --tag desktop
##
## Flags this probe actually reads, grepped from the body below:
##   `--vp WxH`      SubViewport size in physical px. Default 1080x2340.
##   `--tag NAME`    prefix on every output line. Default `nwm`.
##   `--force-touch` NOT read here -- `dcc_shell.gd` reads it; it is what makes
##                   the run a phone.
## Any other `--flag` aborts rather than being silently ignored.

class RecBridge extends EngineBridge:
	var calls: Array = []
	func generate(request: Dictionary) -> void:
		calls.append(request)

var app: Node
var _vp: SubViewport
var _tag := "nwm"
var _fail := 0
var rec: RecBridge

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

func _all(root: Node, out: Array) -> void:
	for c in root.get_children():
		out.append(c)
		_all(c, out)

## Sizes the form with the REAL bridge (region aspect), and returns [gw, gh].
func _size_grid(dlg: NewWorldDialog, gw: int) -> Array:
	dlg.aspect_input.selected = NewWorldDialog.ASPECT_REGION_INDEX
	dlg.grid_w_input.value = gw
	dlg._refresh_dimensions()
	return [int(dlg.grid_w_input.value), int(dlg.grid_h_input.value)]

func _press_create(dlg: NewWorldDialog) -> void:
	if app.is_phone():
		dlg._phone_create.pressed.emit()
	else:
		dlg.confirmed.emit()

func _finish() -> void:
	if rec != null:
		rec.free()
		rec = null
	_log("RESULT %s fail=%d" % [_tag, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

func _ready() -> void:
	_tag = _arg("--tag", "nwm")
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "1080x2340").split("x")
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
	var phone: bool = app.is_phone()
	_log("viewport %dx%d  phone=%s" % [_vp.size.x, _vp.size.y, phone])

	var dlg: NewWorldDialog = app.new_world_dialog
	var real_bridge: EngineBridge = dlg.bridge
	## Never added to the tree: `EngineBridge._ready()` boots engine state this
	## stub must not touch. Freed in `_finish()`.
	rec = RecBridge.new()

	# -- Case A: 2048 at the region aspect -- the ceiling itself ---------------
	app.open_new_world()
	await _frames(10)
	var d2: Array = _size_grid(dlg, 2048)
	_log("case A: %d x %d (%d cells)" % [d2[0], d2[1], d2[0] * d2[1]])
	_check(d2[0] * d2[1] <= 2048 * 1311, "the 2K region grid is at or under 2048 x 1311 cells")
	dlg.bridge = rec
	_press_create(dlg)
	await _frames(6)
	_check(not is_instance_valid(dlg._memory_confirm), "2048: no memory question was raised")
	_check(rec.calls.size() == 1, "2048: generated at once (%d generate call(s))" % rec.calls.size())
	dlg.bridge = real_bridge
	rec.calls.clear()
	if dlg.visible:
		dlg.hide()
	await _frames(4)

	# -- Case B: 4096 at the region aspect -------------------------------------
	app.open_new_world()
	await _frames(10)
	var d4: Array = _size_grid(dlg, 4096)
	_log("case B: %d x %d (%d cells)" % [d4[0], d4[1], d4[0] * d4[1]])
	dlg.bridge = rec
	_press_create(dlg)
	await _frames(6)
	if not phone:
		_check(not is_instance_valid(dlg._memory_confirm), "desktop 4096: no memory question")
		_check(rec.calls.size() == 1, "desktop 4096: generated at once (%d call(s))" % rec.calls.size())
		if rec.calls.size() == 1:
			_check(int(rec.calls[0]["grid_w"]) == 4096, "desktop 4096: the request carries grid_w 4096")
		dlg.bridge = real_bridge
		_finish()
		return

	var q: ConfirmationDialog = dlg._memory_confirm
	_check(is_instance_valid(q) and q.visible, "phone 4096: the memory question is up")
	_check(rec.calls.is_empty(), "phone 4096: nothing generated before an answer (%d call(s))" % rec.calls.size())
	_check(not dlg.visible, "phone 4096: the form is hidden under the question")
	if not is_instance_valid(q):
		_finish()
		return
	var text := ""
	var nodes: Array = []
	_all(q, nodes)
	for n in nodes:
		if n is Label or n is RichTextLabel:
			text += String(n.text) + "\n"
	text += q.dialog_text
	_log("  question text: %s" % text.strip_edges().replace("\n", " | "))
	## Literals, not the dialog's constants read back: 4096 x 2621/2622 at the
	## measured 241.3 B/cell is 2.41 GiB -- `MEMORY_OPTIMIZATION_SCOPE.md`'s
	## own table row.
	_check(text.contains("2.41 GiB"), "phone 4096: the question quotes 2.41 GiB")
	_check(text.contains("2048 × 1311"), "phone 4096: the question names the 2048 x 1311 ceiling")
	_check(q.ok_button_text == "Generate anyway", "phone 4096: OK reads `%s`" % q.ok_button_text)
	## Width: `MISTAKES.md`'s ScrollContainer row -- the question must fit the phone.
	var widest := 0.0
	for n in nodes:
		if n is Control and (n as Control).is_visible_in_tree():
			widest = maxf(widest, (n as Control).get_combined_minimum_size().x)
	var screen_dp: float = float(_vp.size.x) / maxf(0.001, float(app.phone_scale()))
	_log("  widest child minimum %.1f dp, screen %.1f dp, window %d px" % [widest, screen_dp, q.size.x])
	_check(widest <= screen_dp + 0.5, "phone 4096: no child of the question is wider than the screen")
	_check(q.size.x <= _vp.size.x, "phone 4096: the question window fits the viewport width")

	## Cancel: nothing generates, the form comes back.
	## Through the button, so `AcceptDialog`'s own deferred hide is in play
	## exactly as a finger would put it there.
	q.get_cancel_button().pressed.emit()
	await _frames(10)
	_check(rec.calls.is_empty(), "phone 4096 Cancel: nothing generated (%d call(s))" % rec.calls.size())
	_check(dlg.visible, "phone 4096 Cancel: the New World form is back")
	_check(int(dlg.grid_w_input.value) == 4096, "phone 4096 Cancel: the form still holds 4096")

	## Create again, then OK: exactly one generate, at 4096.
	_press_create(dlg)
	await _frames(6)
	q = dlg._memory_confirm
	_check(is_instance_valid(q) and q.visible and rec.calls.is_empty(),
		"phone 4096: asked again on the second Create, still nothing generated")
	if is_instance_valid(q):
		q.get_ok_button().pressed.emit()
	await _frames(6)
	_check(rec.calls.size() == 1, "phone 4096 OK: exactly one generate (%d)" % rec.calls.size())
	if rec.calls.size() == 1:
		_check(int(rec.calls[0]["grid_w"]) == 4096, "phone 4096 OK: the request carries grid_w 4096")
	_check(not is_instance_valid(dlg._memory_confirm) or dlg._memory_confirm.is_queued_for_deletion(),
		"phone 4096 OK: the question is dismissed")
	dlg.bridge = real_bridge
	_finish()
