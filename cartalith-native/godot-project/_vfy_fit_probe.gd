extends Node
## VERIFIER fit probe: re-derive Lane A's overflow claim without its harness.
##
## Lane A reported `performance_window` content at 545 px in a 420 px box on
## desktop and laptop and 1088 px on tablet, and `gen_info_dialog` at 524 px in
## a 480 px box on tablet. **`performance_window` was deleted 2026-09-06**
## (`LARGE_ITEM_RULINGS.md` ruling 19), so only the second half still runs;
## the first is kept here as the measurement, not as a live target. This
## measures the same windows the same way at
## whichever build is on disk, so the SAME file can be run against `HEAD` and
## against the working tree and the two numbers compared.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _vfy_fit_probe.tscn -- --vp 1920x1080
##   ... -- --vp 2560x1600 --force-touch
##
## `--vp WxH` is parsed in `_ready()` below and sizes this probe's own
## `SubViewport`; `--force-touch` is NOT read here -- it is consumed by
## `dcc_shell.gd`/`DccTheme`, and is passed through on the command line only.
## Those are the only two flags, and an unknown one aborts rather than defaults.

var _vp: SubViewport
var app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## The union of every descendant rect, in the dialog's own coordinates, with the
## walk stopping at a `ScrollContainer` (whose overflow is reachable by design).
func _extent(n: Node, origin: Vector2) -> Vector2:
	var m := Vector2.ZERO
	for c in n.get_children():
		if c is Control:
			var ctl := c as Control
			if not ctl.visible:
				continue
			var br := ctl.global_position - origin + ctl.size
			m = Vector2(maxf(m.x, br.x), maxf(m.y, br.y))
			if ctl is ScrollContainer:
				continue
		m = m.max(_extent(c, origin))
	return m

func _report(tag: String, dlg: Window, declared: Vector2) -> void:
	dlg.size = Vector2i(declared)
	await _frames(6)
	## Children of a `Window` carry global coordinates in that window's own
	## space, so the origin is (0,0) -- `dlg.position` is the window's place in
	## the embedding viewport and would shift every rect by it.
	var e := _extent(dlg, Vector2.ZERO)
	print("VFYFIT  %-14s declared=%s  laid-out extent=%s  over_y=%+d px"
		% [tag, declared, e, int(e.y - declared.y)])

func _ready() -> void:
	var size := Vector2i(1920, 1080)
	var argv := OS.get_cmdline_user_args()
	var i := 0
	while i < argv.size():
		var a := String(argv[i])
		if a == "--vp" and i + 1 < argv.size():
			var parts := String(argv[i + 1]).split("x")
			if parts.size() != 2:
				print("VFYFIT ABORT --vp wants WxH, got '%s'" % argv[i + 1])
				get_tree().quit(2)
				return
			size = Vector2i(int(parts[0]), int(parts[1]))
			i += 1
		elif a == "--force-touch":
			pass
		else:
			print("VFYFIT ABORT unknown flag %s -- this probe reads only [--vp WxH, --force-touch]" % a)
			get_tree().quit(2)
			return
		i += 1
	_vp = SubViewport.new()
	_vp.size = size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	app._run_pipeline()
	var w := 0
	while app.bridge.generating and w < 2400:
		await get_tree().process_frame
		w += 1
	await _frames(8)
	print("VFYFIT vp=%s touch=%s tablet=%s fs_prose=%d has_world=%s"
		% [size, DccTheme.is_touch(), DccTheme.is_tablet(),
			DccTheme.role_px("fs_prose"), app.bridge.has_world])

	## The `performance` report that stood here is gone: `LARGE_ITEM_RULINGS.md` ruling 19 (2026-09-06) folded `performance_window.gd`
## away -- no diagnostics window exists in this design language.

	app.open_gen_info()
	await _frames(6)
	await _report("gen info", app.gen_info_dialog, Vector2(560, 480))
	app.gen_info_dialog.hide()

	get_tree().quit(0)
