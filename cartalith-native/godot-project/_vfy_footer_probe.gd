extends Node
## VERIFIER probe, 2026-09-07. One question: what were the New World dialog's
## footer buttons, in dp, BEFORE the action row replaced them?
##
## `ANDROID_UI_SPEC.md` §6.6 asserts `59 x 44` and `60 x 44` dp; the lane's own
## report says those three numbers were carried, not measured. Run this with the
## batch's six files `git stash`ed to get HEAD's answer.
##
##   godot --path . _vfy_footer_probe.tscn -- --force-touch --vp 1080x2340

var app: Node
var _vp: SubViewport

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[vfyf] %s" % s)

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _all(root: Node, out: Array) -> void:
	for c in root.get_children():
		out.append(c)
		_all(c, out)

func _dp_in(c: Control, px: float) -> float:
	var f: float = c.get_viewport().get_final_transform().x.x
	return px * maxf(0.001, f) / maxf(0.001, float(app.phone_scale()))

func _ready() -> void:
	var parts: PackedStringArray = _arg("--vp", "1080x2340").split("x")
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	_log("viewport %dx%d phone=%s scale=%.3f" %
		[_vp.size.x, _vp.size.y, app.is_phone(), app.phone_scale()])
	app.open_new_world()
	await _frames(20)
	var dlg: Window = app.new_world_dialog
	if dlg == null or not dlg.visible:
		_log("dialog not up")
		get_tree().quit(1)
		return
	var card: Control = dlg.get("_card")
	if card != null:
		var cr := card.get_global_rect()
		_log("card %.1f x %.1f dp" % [_dp_in(card, cr.size.x), _dp_in(card, cr.size.y)])
	var kids: Array = []
	_all(dlg, kids)
	for n in kids:
		if n is Button and (n as Button).is_visible_in_tree():
			var b := n as Button
			var r := b.get_global_rect()
			if String(b.text) in ["Create", "Cancel", "Close", "CANCEL", "CREATE WORLD"]:
				_log("BUTTON `%s`  x %.1f  %.1f x %.1f dp" %
					[b.text, r.position.x, _dp_in(b, r.size.x), _dp_in(b, r.size.y)])
	get_tree().quit(0)
