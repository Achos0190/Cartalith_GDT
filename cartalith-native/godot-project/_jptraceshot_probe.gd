extends Node
## Lane CalcTrace: the Journey Planner's Calculation trace, drawn.
##
## Proves the four things the trace claims about itself, on a real planned
## journey rather than on a fixture:
##
##   1. every row draws, with its source cell filled from a named panel;
##   2. the reconciliation row's residual against the `days` the stage matrix
##      already reports -- measured, not assumed, on every unblocked leg of
##      three routes;
##   3. BOTH branches of `_trace_source()`: a term inherited from the party
##      form, and the SAME term after a stage override is planted, which must
##      re-point at the stage inspector's own row;
##   4. the grid does not widen the dock (it scrolls horizontally instead),
##      and at tablet density no cell falls under the touch floor.
##
##   desktop:  Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _jptraceshot_probe.tscn
##   tablet:   Godot_v4.7.1-stable_win64_console.exe --path . --headless _jptraceshot_probe.tscn -- --force-touch
##
## **The desktop pass is windowed and that is the load-bearing one** -- it
## reads laid-out rects off a dock that has actually been drawn. The tablet
## pass sizes its own 2560 x 1600 window, which a 1 080p monitor clamps when
## windowed (`_jpinsw_probe.gd` records the same trap), and it measures nothing
## but minimum sizes and resolved font sizes, so `--headless` is the correct
## driver for it (`MISTAKES.md`: "headless for logic and layout; windowed for
## anything that rasterises or times a frame"). Neither pass reads a pixel.
##
## The arithmetic half -- the residuals themselves, over every leg and every
## seed, printed to twelve decimals -- is `_jptrace_probe.gd`.
##
## Density is named beside every number: `touch=`/`tablet=` are `DccTheme`'s
## own predicates and `role_px("btn_min_h")` is the floor they select.
##
## Three seeds, because a residual and a panel width are both content-dependent
## and one world is one sample of either (`MISTAKES.md`: "Report a layout
## measurement").

const SEEDS := [483920, 77021, 4242]
## What `_tabletparity_probe.gd` boots its tablet leg at, and the only shape
## that reaches `DccTheme.is_tablet()` here.
const TABLET_SCREEN := Vector2i(2560, 1600)

var app: Node
var want_touch := false

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## The trace's own grid: four columns whose first cell is the literal header
## `step`. Found by shape rather than by node name, so a rename of the
## container cannot make this probe silently measure nothing.
func _find_trace_grid(root: Node) -> GridContainer:
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is GridContainer and (n as GridContainer).columns == 4:
			var g := n as GridContainer
			if g.get_child_count() >= 4 and g.get_child(0) is Label \
					and String((g.get_child(0) as Label).text) == "step":
				return g
		for c in n.get_children():
			stack.append(c)
	return null

func _cell(g: GridContainer, i: int) -> String:
	if i >= g.get_child_count():
		return "<missing>"
	var c: Node = g.get_child(i)
	return String((c as Label).text) if c is Label else "<%s>" % c.get_class()

func _dump_grid(prefix: String, g: GridContainer) -> void:
	var rows: int = g.get_child_count() / 4
	for r in rows:
		var b: int = r * 4
		var rect: Rect2 = (g.get_child(b) as Control).get_global_rect()
		print("%s  %2d | %-46s | %-42s | %14s | %14s |  h=%.0f fs=%d" % [prefix, r,
			_cell(g, b), _cell(g, b + 1), _cell(g, b + 2), _cell(g, b + 3),
			rect.size.y, (g.get_child(b) as Label).get_theme_font_size("font_size")])

## The reconciliation row is the last one; its fourth cell is the residual.
func _resid_cell(g: GridContainer) -> String:
	return _cell(g, g.get_child_count() - 1)

func _ready() -> void:
	want_touch = "--force-touch" in OS.get_cmdline_user_args()
	## Tablet is decided off the boot window size, so it has to be set BEFORE
	## the shell instantiates -- `--resolution 1600x900 -- --force-touch` boots
	## the PHONE composition (measured: `tablet=false phone=true`), which is a
	## different density with a different floor. 2560 x 1600 is the figure
	## `_tabletparity_probe.gd` boots its own tablet leg at.
	if want_touch:
		DisplayServer.window_set_size(TABLET_SCREEN)
		get_window().size = TABLET_SCREEN
		get_tree().root.gui_embed_subwindows = true
		await _frames(4)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(3)

	var floor_px: float = float(DccTheme.role_px("btn_min_h"))
	print("TSHOT === touch=%s tablet=%s phone=%s scale=%.2f screen=%s btn_min_h=%.0f (%s density) ==="
		% [DccTheme.is_touch(), DccTheme.is_tablet(), app.is_phone(), app.phone_scale(),
			str(app.get_viewport_rect().size), floor_px,
			"touch/tablet" if DccTheme.is_tablet() else ("touch/phone" if app.is_phone() else "pointer")])
	if want_touch and not DccTheme.is_tablet():
		print("TSHOT FAIL: --force-touch did not produce TABLET density; nothing below is meaningful")
		get_tree().quit(1)
		return
	if not want_touch and DccTheme.is_touch():
		print("TSHOT FAIL: the pointer pass came up touch")
		get_tree().quit(1)
		return

	var bridge = app.bridge
	var rd = app.right_dock_ctrl
	var body: Control = app.right_dock_body
	var worst_resid := 0.0
	var legs_checked := 0
	var rows_seen := 0
	var sourceless := 0

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
			print("TSHOT seed %d: generate FAILED" % seed_v)
			continue
		var gs: Vector2i = bridge.grid_size()
		bridge.route_begin("mixed")
		bridge.route_append_stop(gs.x * 0.20, gs.y * 0.30)
		bridge.route_append_stop(gs.x * 0.55, gs.y * 0.50)
		bridge.route_append_stop(gs.x * 0.82, gs.y * 0.72)
		var ridx: int = bridge.route_commit()

		app.select_domain("civilization")
		await _frames(3)
		app.arm_tool("journey")
		await _frames(14)
		var jpv = app.journey_planner_view
		jpv._refresh_route_choice()
		jpv._compute()
		rd.refresh_journey()
		await _frames(10)

		var plan: Dictionary = (jpv._last_result.get("plan", {}) as Dictionary)
		var results: Array = plan.get("results", [])
		print("TSHOT ##### seed %d route=%d stages=%d dock_min.x=%.0f body_min.x=%.0f #####"
			% [seed_v, ridx, results.size(),
				(app.right_dock as Control).get_combined_minimum_size().x,
				body.get_combined_minimum_size().x])

		## Every unblocked leg, land and water -- not stage 0. The residual is
		## per-leg and a route's first stage is one sample of it.
		for i in results.size():
			var r: Dictionary = results[i]
			if bool(r.get("blocked", false)):
				continue
			jpv._selected_stage = i
			jpv._compute()
			rd.refresh_journey()
			await _frames(6)
			var g := _find_trace_grid(body)
			if g == null:
				print("TSHOT   %02d NO GRID FOUND -- the trace group did not draw" % (i + 1))
				continue
			legs_checked += 1
			var rows: int = g.get_child_count() / 4
			rows_seen += rows
			var calc: Dictionary = r.get("land", r.get("water", {}))
			var terms: int = (calc.get("trace", []) as Array).size()
			## header + stage length + terms + "= km/day" + reconciliation.
			var want_rows: int = terms + 4
			var resid := _resid_cell(g)
			var ok := rows == want_rows
			print("TSHOT   %02d cat=%-5s terms=%-2d rows=%d (want %d)%s  km=%.3f days=%.6f  RESID=%s"
				% [i + 1, String(r.get("cat", "?")), terms, rows, want_rows,
					"" if ok else "  ROW COUNT MISMATCH",
					float(r.get("km", 0.0)), float(r.get("days", 0.0)), resid])
			var rv := float(resid.replace(" d", ""))
			worst_resid = maxf(worst_resid, absf(rv))
			## Every factor row must carry a source. The only cell allowed to
			## say "no panel shows this" is `column`, whose `col_km`/`col_mod`
			## genuinely reach no readout in this shell.
			for rr in range(2, rows - 2):
				var src := _cell(g, rr * 4 + 1)
				if src.begins_with("—") and not _cell(g, rr * 4).begins_with("column length"):
					sourceless += 1
					print("TSHOT     UNSOURCED ROW: %s -> %s" % [_cell(g, rr * 4), src])
			## One full dump per seed, on the first leg walked.
			if legs_checked % 100 == 1 or i == 0:
				_dump_grid("TSHOT   ", g)
				var sc: Control = g.get_parent()
				print("TSHOT     grid min.x=%.0f  scroll(%s) min.x=%.0f h_mode=%d  dock min.x=%.0f"
					% [g.get_combined_minimum_size().x, sc.get_class(),
						sc.get_combined_minimum_size().x,
						(sc as ScrollContainer).horizontal_scroll_mode if sc is ScrollContainer else -1,
						(app.right_dock as Control).get_combined_minimum_size().x])
				var short := 0
				for ci in g.get_child_count():
					var c: Control = g.get_child(ci)
					if c.get_global_rect().size.y < floor_px - 0.5 and (c is BaseButton or c is LineEdit):
						short += 1
				print("TSHOT     tappable cells under the %.0f px floor: %d (%s density)"
					% [floor_px, short, "touch/tablet" if DccTheme.is_tablet() else "pointer"])

		## -- Both branches of `_trace_source()`, on one leg -----------------
		var land_idx := -1
		for i in results.size():
			if not bool((results[i] as Dictionary).get("blocked", false)) \
					and (results[i] as Dictionary).has("land"):
				land_idx = i
				break
		if land_idx < 0:
			print("TSHOT   no unblocked land leg on this route -- override branch not exercised here")
			continue
		jpv._selected_stage = land_idx
		jpv._stage_overrides.erase(land_idx)
		jpv._compute()
		rd.refresh_journey()
		await _frames(6)
		var g0 := _find_trace_grid(body)
		var before := _source_of(g0, "party pace")
		jpv._set_stage_override(land_idx, "pace", "Forced March")
		await _frames(10)
		rd.refresh_journey()
		await _frames(6)
		var g1 := _find_trace_grid(body)
		var after := _source_of(g1, "party pace")
		print("TSHOT   source branches on stage %02d: inherited=[%s]  overridden=[%s]  %s"
			% [land_idx + 1, before, after,
				"BOTH BRANCHES REACHED" if (before != after and after.begins_with("Stage")) else "BRANCH NOT REACHED"])
		jpv._stage_overrides.erase(land_idx)
		jpv._compute()
		rd.refresh_journey()
		await _frames(4)

		## -- Positive control -----------------------------------------------
		## Every residual above is +0.000000 d, which is exactly the state
		## where a reconciliation cannot be trusted: an assertion that only
		## ever sees the good case proves nothing about the bad one
		## (`MISTAKES.md`: "beware an assertion that passes because the bad
		## state cannot exist yet", and "give the probe a positive control that
		## must move"). So one term is pulled out of the live trace -- the same
		## shape as an engine that applied a factor and did not publish it --
		## and the row must go non-zero by exactly that factor's worth.
		## `_last_result` holds live `Dictionary`/`Array` references, so the
		## pop reaches the same object `_build_trace_group()` reads, and the
		## `_compute()` afterwards throws the mutation away.
		jpv._selected_stage = land_idx
		jpv._compute()
		rd.refresh_journey()
		await _frames(6)
		var live: Dictionary = ((jpv._last_result.get("plan", {}) as Dictionary)
			.get("results", [])[land_idx] as Dictionary)
		var lcalc: Dictionary = live.get("land", live.get("water", {}))
		var ltrace: Array = lcalc.get("trace", [])
		## The LAST term is not a valid control: `load` is exactly x1.000 on
		## every leg these three seeds produce, so dropping it changes the row
		## count and not the number -- measured, and it is why this picks by
		## factor instead. The term furthest from 1.0 is the one whose absence
		## the residual cannot fail to show.
		var pick := -1
		var pick_dev := 0.0
		for ti in ltrace.size():
			var dev: float = absf(float((ltrace[ti] as Dictionary).get("factor", 1.0)) - 1.0)
			if dev > pick_dev:
				pick_dev = dev
				pick = ti
		if pick < 0:
			print("TSHOT   POSITIVE CONTROL skipped: every term on stage %02d is exactly x1.0" % (land_idx + 1))
			continue
		var dropped: Dictionary = ltrace[pick]
		var want_ratio := float(dropped.get("factor", 1.0))
		ltrace.remove_at(pick)
		rd.refresh_journey()
		await _frames(6)
		var gpc := _find_trace_grid(body)
		var pc_resid := _resid_cell(gpc)
		var pc_rows: int = (gpc.get_child_count() / 4) if gpc != null else -1
		var want_resid := float(live.get("days", 0.0)) * (want_ratio - 1.0)
		print("TSHOT   POSITIVE CONTROL stage %02d: dropped `%s` (x%.6f) -> rows=%d resid=%s (expected %+.6f d)  %s"
			% [land_idx + 1, String(dropped.get("key", "?")), want_ratio, pc_rows, pc_resid, want_resid,
				"MISMATCH DRAWN" if not pc_resid.begins_with("+0.000000") else "CONTROL DID NOT MOVE"])
		jpv._compute()
		rd.refresh_journey()
		await _frames(6)
		var grestore := _find_trace_grid(body)
		print("TSHOT   restored: resid=%s rows=%d" % [_resid_cell(grestore),
			(grestore.get_child_count() / 4) if grestore != null else -1])

	print("TSHOT === legs=%d rows=%d unsourced=%d worst |residual|=%.6f d ==="
		% [legs_checked, rows_seen, sourceless, worst_resid])
	get_tree().quit()

## The source cell of the row whose step cell starts with `step_prefix`.
func _source_of(g: GridContainer, step_prefix: String) -> String:
	if g == null:
		return "<no grid>"
	var rows: int = g.get_child_count() / 4
	for r in rows:
		if _cell(g, r * 4).begins_with(step_prefix):
			return _cell(g, r * 4 + 1)
	return "<row not found>"
