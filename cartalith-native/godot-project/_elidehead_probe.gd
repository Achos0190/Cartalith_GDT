extends Node
## **The real-shell half of the `_elide_labels` fix**
## (`dcc_theme.gd::header()`, `OUTSTANDING_WORK.md` row on `_elide_labels`
## growing without limit on desktop).
##
## `_elidechurn_probe.gd` proves the mechanism in isolation; this probe boots
## the actual `app.tscn`, walks the live WORLD dock's real header Labels (the
## default `_active_domain`, so no domain switch is needed to reach it), and
## dumps each one's text, clip_text, text_overrun_behavior and drawn global
## rect -- the exact quantities the row's own gate names: "any label's drawn
## rect or clip state changes at any form factor".
##
## Run once against the live tree and once against a HEAD-copy scratch build
## (`shell/dcc_theme.gd` swapped back to its pre-fix content, everything else
## identical) at the SAME resolution, and diff the two RESULT blocks. Every
## `label ...` line must be byte-identical; only the `registered-in-array`
## count may differ (0 on the fixed tree, equal to the label count on HEAD --
## the whole defect this row is about).
##
##   godot --headless --resolution 1920x1080 _elidehead_probe.tscn
##
## Deliberately desktop-only (no `--force-touch`): the row's own defect is
## desktop-specific, and `_elidechurn_probe.gd` already covers the tablet
## path directly.

func _log(s: String) -> void:
	print("[elidehead] %s" % s)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## Every Label under `root` tagged as a `header()` build (`dcc_role` meta ==
## `"fs_dock_header"`), found by the SAME tag `DccTheme.header()` itself
## writes -- not by node path, which a layout change could move.
func _collect_headers(root: Node, out: Array) -> void:
	if root is Label and root.has_meta("dcc_role") \
			and root.get_meta("dcc_role") == "fs_dock_header":
		out.append(root)
	for c in root.get_children():
		_collect_headers(c, out)

func _ready() -> void:
	var vp := SubViewport.new()
	vp.size = Vector2i(1920, 1080)
	vp.transparent_bg = false
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(6)

	var shell: Node = app.get("shell")
	if shell == null:
		shell = app
	_log("is_tablet=%s is_phone=%s active_domain=%s"
		% [DccTheme.is_tablet(), shell.call("is_phone"), shell.get("_active_domain")])

	var left_body: Node = shell.get("left_dock_body")
	if left_body == null:
		_log("RESULT no left_dock_body -- cannot reach the WORLD dock")
		get_tree().quit(2)
		return

	var headers: Array = []
	_collect_headers(left_body, headers)
	## Stable order across the two runs regardless of any incidental
	## Dictionary/Array build-order difference elsewhere in the shell.
	headers.sort_custom(func(a, b): return a.text < b.text)

	_log("header count: %d" % headers.size())
	var registered := 0
	for entry in headers:
		var l: Label = entry
		if DccTheme._elide_labels.has(l):
			registered += 1
		var r: Rect2 = l.get_global_rect()
		_log("label text=%-40s clip=%s overrun=%d rect=(%d,%d,%d,%d)" % [
			l.text, l.clip_text, l.text_overrun_behavior,
			int(round(r.position.x)), int(round(r.position.y)),
			int(round(r.size.x)), int(round(r.size.y))])
	_log("registered-in-array: %d of %d" % [registered, headers.size()])
	_log("RESULT done")
	get_tree().quit(0)
