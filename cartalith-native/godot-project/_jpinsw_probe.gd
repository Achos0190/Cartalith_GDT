extends Node
## Lane PhoneWidth: WHICH rows of the Journey Planner's phone stage inspector
## demand more than the 1 080 px screen, how wide each one really is, and what
## inside it carries the width.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --headless _jpinsw_probe.tscn -- --force-touch
##
## Layout minimums only -- no pixels are read, so `--headless` is the correct
## driver here (`MISTAKES.md`: "headless for logic and layout; windowed for
## anything that rasterises"). It also avoids the OS clamping a 2 400 px-tall
## window on a 1 080p monitor, which would silently change the viewport every
## number below is measured against.
##
## Run windowed once as well (drop `--headless`) and it agrees to the pixel --
## `root=(1080, 2400)` holds, `_inspector_body` 733 / 770 / 813 on seed 483920's
## three sampled stages either way -- so the headless figures are not a dummy
## -driver artefact. Windowed is the slower of the two and buys nothing here,
## which is why it is not the documented invocation.
##
## Three seeds x three stages each, because a panel width is content-dependent
## and one world is one sample of it exactly as one stage is one sample of the
## per-stage header row (`MISTAKES.md`: "Report a layout measurement").
##
## Density is named beside every number: `phone=true`, `scale=` is
## `DccApp.phone_scale()`, and the screen is 1 080 x 2 400 physical px.

const SEEDS := [483920, 77021, 4242]
## Phone unless `--force-touch` is absent, in which case this is the desktop
## pass: the same rows measured against a 1 684 px session, because the fix
## these numbers grade is unconditional and a claim about desktop has to be
## measured on desktop rather than divided by `phone_scale()`.
const PHONE_SCREEN := Vector2i(1080, 2400)
const DESKTOP_SCREEN := Vector2i(1684, 1050)

var app: Node
var want_phone := false

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _tx(n: Node) -> String:
	if n is OptionButton:
		var ob := n as OptionButton
		var longest := ""
		for i in ob.item_count:
			if ob.get_item_text(i).length() > longest.length():
				longest = ob.get_item_text(i)
		return "OB items=%d fit=%s sel=[%s] longest=[%s]" % [
			ob.item_count, str(ob.fit_to_longest_item), ob.text, longest]
	if n is Button:
		return "[%s]" % (n as Button).text
	if n is Label:
		return "[%s]" % (n as Label).text
	if n is SpinBox:
		return "[%s]" % (n as SpinBox).get_line_edit().text
	return ""

## min.x of every Control ancestor from `leaf` up to the window root, so the
## claim "no ancestor exceeds the screen" is measured rather than reasoned.
func _up(leaf: Control) -> Array:
	var out: Array = []
	var n: Node = leaf
	while n != null:
		if n is Control:
			var c := n as Control
			out.append("%s(%s) min.x=%.0f rect.w=%.0f" % [c.get_class(), c.name,
				c.get_combined_minimum_size().x, c.get_global_rect().size.x])
		n = n.get_parent()
	return out

## Every Control in `root`'s subtree, deepest-first blame for one row.
func _row_dump(prefix: String, row: Control) -> void:
	var parts: Array = []
	for c in row.get_children():
		if c is Control and (c as Control).visible:
			var cr := (c as Control).get_global_rect()
			parts.append("%s min=%.0f laid[%.0f..%.0f]%s" % [(c as Control).get_class(),
				(c as Control).get_combined_minimum_size().x,
				cr.position.x, cr.position.x + cr.size.x, _tx(c)])
	var rr := row.get_global_rect()
	print("%s  %6.0f  laid[%.0f..%.0f]%s  %s :: %s" % [prefix, row.get_combined_minimum_size().x,
		rr.position.x, rr.position.x + rr.size.x,
		("  OFFSCREEN+%.0f" % (rr.position.x + rr.size.x - 1080.0)) if rr.position.x + rr.size.x > 1080.5 else "",
		row.get_class(), " | ".join(parts)])

## `phone_fit()`'s own tappable set for the classes this panel actually builds.
func _taps(root: Node, out: Array) -> void:
	for c in root.get_children():
		if (c is BaseButton or c is SpinBox) and (c as Control).visible:
			out.append(c)
		_taps(c, out)

func _all_rows(root: Node, out: Array) -> void:
	for c in root.get_children():
		if c is BoxContainer or c is GridContainer or c is FlowContainer:
			out.append(c)
		_all_rows(c, out)

func _ready() -> void:
	want_phone = "--force-touch" in OS.get_cmdline_user_args()
	var screen_want := PHONE_SCREEN if want_phone else DESKTOP_SCREEN
	DisplayServer.window_set_size(screen_want)
	get_window().size = screen_want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(3)
	var screen: Vector2 = app.get_viewport_rect().size
	print("INSW === phone=", app.is_phone(), " scale=", app.phone_scale(), " screen=", screen,
		" root=", get_tree().root.size, " ===")
	if want_phone and not app.is_phone():
		print("INSW FAIL: --force-touch did not produce phone mode; nothing below is meaningful")
		get_tree().quit(1)
		return
	if not want_phone and app.is_phone():
		print("INSW FAIL: desktop pass came up in phone mode")
		get_tree().quit(1)
		return

	var bridge = app.bridge
	for seed_v in SEEDS:
		bridge.generate({
			"seed": seed_v, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
			"archetype": "", "villages": true, "sea_level": 0.45,
		})
		var waited := 0
		while bridge.generating and waited < 3000:
			await get_tree().process_frame
			waited += 1
		await _frames(10)
		if not bridge.has_world:
			print("INSW seed %d: generate FAILED" % seed_v)
			continue
		var gs: Vector2i = bridge.grid_size()
		bridge.route_begin("mixed")
		bridge.route_append_stop(gs.x * 0.20, gs.y * 0.30)
		bridge.route_append_stop(gs.x * 0.55, gs.y * 0.50)
		bridge.route_append_stop(gs.x * 0.82, gs.y * 0.72)
		var ridx: int = bridge.route_commit()

		app.open_journey_planner()
		await get_tree().create_timer(0.8).timeout
		var jpv = app.journey_planner_view
		jpv._refresh_route_choice()
		jpv._compute()
		await _frames(12)

		var stages: int = (jpv._last_result.get("plan", {}) as Dictionary).get("stages", []).size()
		## First, last and middle stage, not just stage 0: the header row is the
		## widest content-driven row in the panel and its text is per-stage, so
		## one stage is one sample of it in exactly the way one seed is one
		## sample of the world.
		var picks: Array = []
		for cand in [0, stages / 2, maxi(0, stages - 1)]:
			if not picks.has(cand) and cand < stages:
				picks.append(cand)
		if picks.is_empty():
			picks.append(0)
		for pick in picks:
			await _report_stage(jpv, seed_v, ridx, stages, int(pick), screen)

	print("INSW === done ===")
	get_tree().quit()

func _report_stage(jpv, seed_v: int, ridx: int, stages: int, pick: int, screen: Vector2) -> void:
		jpv._selected_stage = pick
		jpv._compute()
		await _frames(8)
		var body: Control = jpv._inspector_body
		var scroll: Control = body.get_parent()
		print("INSW ##### seed %d route=%d stages=%d selected=%d #####" % [seed_v, ridx, stages, jpv._selected_stage])
		if stages == 0:
			print("INSW   no stages -- inspector shows the empty note only")
		print("INSW   chain from _inspector_body upward:")
		for line in _up(body):
			print("INSW     ^ %s" % line)

		var rows: Array = []
		_all_rows(body, rows)
		rows.append(body)
		var wider := func(a, b): return (a as Control).get_combined_minimum_size().x > (b as Control).get_combined_minimum_size().x
		rows.sort_custom(wider)
		print("INSW   rows by min.x (screen=%d):" % int(screen.x))
		for i in mini(20, rows.size()):
			var r: Control = rows[i]
			if r == body:
				print("INSW   %6.0f  <_inspector_body>" % r.get_combined_minimum_size().x)
				continue
			_row_dump("INSW ", r)
		var over := 0
		for r2 in rows:
			if (r2 as Control).get_combined_minimum_size().x > screen.x:
				over += 1
		print("INSW   rows wider than the screen: %d of %d" % [over, rows.size()])
		var floor_px: float = round(DccTheme.PHONE_TAP_MIN * app.phone_scale()) if app.is_phone() else 0.0
		var taps: Array = []
		_taps(body, taps)
		var short := 0
		for t in taps:
			if (t as Control).get_global_rect().size.y < floor_px - 0.5:
				short += 1
				print("INSW   TAP UNDER FLOOR %.0f: %s %s h=%.0f" % [floor_px, (t as Control).get_class(),
					_tx(t), (t as Control).get_global_rect().size.y])
		print("INSW   tappable in inspector=%d under the %.0f px floor=%d (%s density)"
			% [taps.size(), floor_px, short, "touch/phone" if app.is_phone() else "pointer"])
		print("INSW   scroll min.x=%.0f  h_mode=%d  body min.x=%.0f  center_panel min.x=%.0f rect=%s" % [
			scroll.get_combined_minimum_size().x,
			(scroll as ScrollContainer).horizontal_scroll_mode,
			body.get_combined_minimum_size().x,
			(jpv._center_panel as Control).get_combined_minimum_size().x,
			str((jpv._center_panel as Control).get_global_rect())])
