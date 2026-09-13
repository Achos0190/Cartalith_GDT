extends Node
## Lane GRID, 2026-09-13. Measures `world_workspace.gd`'s tablet-portrait
## slider reflow (`_make_compact_slider_cell`, `_build_param_row` /
## `_build_droplet_erosion`) against the WORLD dock's real click unit --
## `WorldWorkspace::CATEGORIES` (Generate, Terrain, Geology, Hydrology,
## Climate, Biomes, Ecology, Resources, World data), each of which embeds one
## or more `STAGES` entries' rows. `STAGES` itself has no independent open/
## close state; `CATEGORIES`' own accordion button is what a tap actually
## opens, so that is the unit this probe sweeps, walked from
## `WorkspacePanel.categories` (`workspaces/workspace.gd`) rather than the
## STAGES table a first reading of the brief suggested.
##
## `--force-touch` is read once, at process start (`DccShell._ready()`), so it
## cannot be toggled per boot inside one run -- three separate invocations:
##
##   Godot..console.exe --path . _worldportraitgrid_probe.tscn
##       -> desktop 1920x1080, laptop 1366x768 (G2 control frames, non-touch)
##   Godot..console.exe --path . _worldportraitgrid_probe.tscn -- --force-touch
##       -> portrait 800x1280 / 900x1440, tablet landscape 1280x800,
##          phone 1080x2340
##   Godot..console.exe --path . _worldportraitgrid_probe.tscn -- --force-touch --rotate
##       -> one instance, live: portrait -> landscape -> portrait
##
## Windowed (no `--headless`): every figure here is a Control minimum/drawn
## size, which needs no rasterised frame, but the brief's own instruction was
## windowed and it costs nothing to honour it exactly as given.

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

## `OUTSTANDING_WORK.md` "TABLET PORTRAIT: five Planet sliders..." row's own
## disclosed, still-open residue: Climate's *Lapse rate* readout `6.5°C/km`
## (63px, never clipped -- it's a number) beside the 132px label floor reads
## 244 in EVERY portrait leg (800x1280, 900x1440, and a generated world), 12px
## over the 232 role. Folding it into the generic "within 232 +-1" check made
## this probe's own portrait baseline 2 (then 3, with `--with-world`) silent
## FAILs a real regression could hide behind (wf53). Pinned as a NAMED literal
## instead: baseline is 0, and per MISTAKES.md's "write a test that pins a
## constant" rule, a DIFFERENT number -- worse OR better -- still goes red,
## which is deliberate: closing 244 needs a shorter unit string from
## `params.rs` or a narrower portrait label floor, not a silent probe pass.
const CLIMATE_KNOWN_RESIDUE_PX := 244.0

func _assert_dock_width(tag: String, title: String, dock_w: float) -> void:
	if title == "Climate":
		_ok("%s Climate dock_w == %.0f (KNOWN residue, OUTSTANDING_WORK.md Lapse-rate unit string)" % [
			tag, CLIMATE_KNOWN_RESIDUE_PX], dock_w, CLIMATE_KNOWN_RESIDUE_PX)
	else:
		_ok("%s %s dock_w within 232 +-1" % [tag, title], absf(dock_w - 232.0) <= 1.0, true)

func _boot_vp(w: int, h: int) -> Dictionary:
	var vp := SubViewport.new()
	vp.size = Vector2i(w, h)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(30)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.get("phone_project_picker") != null:
		app.phone_project_picker.hide()
	await _frames(10)
	return {"vp": vp, "app": app}

## The single widest visible-leaf minimum under `root` -- what would be
## driving an oversized dock, named the way `_tabspec_probe.gd::_drivers_walk`
## already names a driver so a report reads the same way that probe's does.
func _worst_row_min(root: Node) -> Dictionary:
	var best_w := 0.0
	var best_txt := ""
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Control and not (n as Control).is_visible_in_tree():
			continue
		for c in n.get_children(true):
			stack.append(c)
		if n is Control:
			var m: float = (n as Control).get_combined_minimum_size().x
			## `>=`, not `>`: a wrapper `Container` reports the same minimum as
			## its one wide child, and DFS visits the wrapper first, so `>`
			## would freeze on the ancestor and never name the actual leaf.
			if m >= best_w and m > 0.0:
				best_w = m
				var t := ""
				if "text" in n:
					t = str(n.get("text")).strip_edges()
				elif n is OptionButton:
					var items: Array = []
					for i in (n as OptionButton).item_count:
						items.append((n as OptionButton).get_item_text(i))
					t = "items=" + ",".join(items)
				best_txt = "%s %s" % [n.get_class(), t.left(40)]
	return {"w": best_w, "what": best_txt}

## Every category in `panel.get("categories")`, opened one at a time exactly
## the way a tap does (`btn.pressed.emit()`, the signal `DccWidgets.category`
## itself connects) -- not a second, parallel way of finding "open".
func _sweep_categories(app: Node) -> Array:
	var shell := app as DccShell
	shell.select_domain_mode("world", "a")
	await _frames(6)
	var panel: Control = shell.workspace_panel("world")
	var ld := app.get("left_dock") as Control
	var out: Array = []
	for e in (panel.get("categories") as Array):
		var btn := e["button"] as Button
		var body := e["body"] as Control
		if not body.visible:
			btn.pressed.emit()
			await _frames(8)
		out.append({
			"title": String(e["title"]),
			"dock_w": ld.size.x,
			"dock_min": ld.get_combined_minimum_size().x,
			"worst_row": _worst_row_min(body),
		})
		## Named chain for anything still over the 232 portrait role -- printed
		## at the point of discovery rather than re-derived from the summary
		## table, so the driver is attributed to the category it was measured
		## in, not guessed from a width alone.
		if DccTheme.is_tablet_portrait() and ld.size.x > 232.5:
			_drivers(String(e["title"]), body, 216.0)
	return out

## `_tabspec_probe.gd::_drivers_walk`'s own technique, reused rather than
## re-invented: print every visible node whose own combined minimum meets or
## exceeds `floor_w`, which is how a pass-through `Container` (reporting
## exactly its one child's minimum) is told apart from the leaf actually
## driving the number -- both print, in order, so the chain is legible.
func _drivers(tag: String, root: Node, floor_w: float) -> void:
	print("  [drivers over %.0f] %s" % [floor_w, tag])
	_drivers_walk(root, floor_w, 0)

func _drivers_walk(node: Node, floor_w: float, depth: int) -> void:
	for child in node.get_children(true):
		if not (child is Control):
			_drivers_walk(child, floor_w, depth)
			continue
		var ctl := child as Control
		if not ctl.is_visible_in_tree():
			continue
		var m: float = ctl.get_combined_minimum_size().x
		if m > floor_w:
			var txt := ""
			if "text" in ctl:
				txt = str(ctl.get("text")).strip_edges()
			if txt == "" and ctl is OptionButton:
				var items: Array = []
				for i in (ctl as OptionButton).item_count:
					items.append((ctl as OptionButton).get_item_text(i))
				txt = "items=" + ",".join(items)
			print("    %s%s  min=%.1f  own=%.1f  %s" % [
				"  ".repeat(depth), ctl.get_class(), m, ctl.custom_minimum_size.x,
				("\"" + txt.left(48) + "\"") if txt != "" else ctl.name])
		_drivers_walk(ctl, floor_w, depth + 1)

func _print_sweep(rows: Array) -> void:
	for r in rows:
		var wr: Dictionary = r["worst_row"]
		print("  %-14s dock_w=%6.1f dock_min=%6.1f  worst_row_min=%6.1f  %s" % [
			String(r["title"]), float(r["dock_w"]), float(r["dock_min"]),
			float(wr["w"]), String(wr["what"])])

## The bug this whole lane's revert was about, made directly assertable:
## `left_dock_title`/`right_dock_title` (`dcc_shell.gd`) draw beside a
## `DccTheme.spacer()` (`SIZE_EXPAND_FILL`) with no floor of their own, so an
## unconditional `header()` clip collapses them to ~1 px -- measured, not
## guessed, the day this was found (WORLD/CIVILIZATION/CARTOGRAPHY/SAMPLE all
## at 1.0 px). 20 px is comfortably below any real render of a short
## all-caps word at `fs_dock_header`/`FS_HEADER` size and comfortably above
## the ~1 px collapse signature, so it separates the two without pinning an
## exact figure this probe does not own. Checked on every leg, not just the
## four the bug report named: a dock head must never elide, full stop.
##
## Visibility asserted everywhere EXCEPT phone -- confirmed pre-existing on a
## pure HEAD copy, not a consequence of anything this lane touched: phone
## builds both docks `as_sheet` (`_build_left_dock`'s own doc, `dcc_shell.gd`)
## and §13's sheets are closed at boot ("docks become full-height sheets, ONE
## AT A TIME"), so both title Labels are legitimately `is_visible_in_tree()
## == false` until a sheet opens -- a precondition this probe does not drive
## (never navigate inside a probe). The WIDTH check does not depend on tree
## visibility (`get_combined_minimum_size` reads it regardless) and still
## catches the collapse this assertion exists for on every geometry, phone
## included.
func _assert_dock_titles(tag: String, app: Node) -> void:
	var lt := app.get("left_dock_title") as Label
	var rt := app.get("right_dock_title") as Label
	if not DccTheme.is_phone():
		_ok("%s left_dock_title visible" % tag, lt != null and lt.is_visible_in_tree(), true)
	if lt != null:
		_ok("%s left_dock_title width > 20 (not collapsed)" % tag,
			lt.get_combined_minimum_size().x > 20.0, true)
	if not DccTheme.is_phone():
		_ok("%s right_dock_title visible" % tag, rt != null and rt.is_visible_in_tree(), true)
	if rt != null:
		_ok("%s right_dock_title width > 20 (not collapsed)" % tag,
			rt.get_combined_minimum_size().x > 20.0, true)

func _run_sweep_frame(tag: String, w: int, h: int) -> Array:
	var boot := await _boot_vp(w, h)
	print("\n--- %s  vp=%dx%d  touch=%s tablet=%s phone=%s laptop=%s portrait=%s ---" % [
		tag, w, h, str(DccTheme.is_touch()), str(DccTheme.is_tablet()),
		str(DccTheme.is_phone()), str(DccTheme.is_laptop()),
		str(DccTheme.is_tablet_portrait())])
	_assert_dock_titles(tag, boot["app"])
	var rows := await _sweep_categories(boot["app"])
	_print_sweep(rows)
	(boot["app"] as Node).queue_free()
	(boot["vp"] as Node).queue_free()
	await _frames(2)
	return rows

## Lane PROBES, 2026-09-13, closing the probe-honesty gap `OUTSTANDING_WORK.md`
## ("Discipline debts") named: this file's rotate leg asserted dock width and
## the mounted slider shape but never the one thing `_on_portrait_changed()`
## added on 2026-09-13 for the SAME bug class -- the GENERATE stage-name
## labels' own `clip_text`/`text_overrun_behavior` re-derivation
## (`world_workspace.gd:1711-1717`). Confirmed by mutation: commenting out
## that loop left every existing rotate assertion below green. Read via
## `panel.get("_stage_name_labels")`, the same private-array pattern this file
## already uses for `_slider_reflow_entries` -- not a new access technique.
func _assert_stage_clip(tag: String, panel: Control) -> void:
	var want_compact := DccTheme.is_tablet_portrait()
	var want_overrun := TextServer.OVERRUN_TRIM_ELLIPSIS if want_compact \
		else TextServer.OVERRUN_NO_TRIMMING
	var labels: Array = panel.get("_stage_name_labels")
	_ok("%s stage-name labels array non-empty" % tag, labels.size() > 0, true)
	var mismatched := 0
	for nl in labels:
		var lbl := nl as Label
		if lbl == null:
			continue
		if lbl.clip_text != want_compact or lbl.text_overrun_behavior != want_overrun:
			mismatched += 1
	_ok("%s stage-name labels re-clipped (%d checked, want compact=%s)" % [
		tag, labels.size(), want_compact], mismatched, 0)

## `--rotate`: one instance, real window (not a SubViewport) -- `DccShell`'s
## resize handling listens on `get_tree().root.size_changed`
## (`dcc_shell.gd::_ready()`), which only fires from the actual OS/root
## viewport resizing. A SubViewport's own `.size` write does not reach it, so
## the sweep's SubViewport-per-frame boot cannot exercise this path at all --
## this is the one part of the probe that has to run in a real window.
## `boot_landscape`: the refutation named BOTH directions broken separately
## ("a landscape boot rotated to portrait reads 331; a portrait boot rotated
## to landscape keeps two-line cells") -- one fixed reflow function serves
## both (`_mount_wide`/`_mount_compact` are symmetric, called by the same
## `_on_portrait_changed`), but that is a code-review claim until each BOOT
## order is actually exercised once, not inferred from the other.
func _run_rotate(boot_landscape: bool) -> void:
	print("\n=== ROTATE, one instance, real window, --force-touch, boot=%s ===" %
		("landscape" if boot_landscape else "portrait"))
	get_window().size = Vector2i(1280, 800) if boot_landscape else Vector2i(800, 1280)
	await _frames(4)
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(30)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(10)
	var shell := app as DccShell
	shell.select_domain_mode("world", "a")
	await _frames(6)
	var ld := app.get("left_dock") as Control
	var panel: Control = shell.workspace_panel("world")
	# Open "Generate" (World structure's 5 steering sliders -- the row's own
	# cited example) so the dock's drawn width reflects real slider content,
	# not an empty accordion. **Generate boots OPEN already** (`i == 0` in
	# `_build_categories()`), which is exactly the bug this fixes: the
	# reverted batch's own rotate probe pressed unconditionally here, which
	# TOGGLES an already-open category CLOSED (`DccWidgets._toggle_category`)
	# and so measured an empty body the whole run. Same "only press if
	# closed" guard `_sweep_categories` below already uses.
	for e in (panel.get("categories") as Array):
		if String(e["title"]) == "Generate":
			if not (e["body"] as Control).visible:
				(e["button"] as Button).pressed.emit()
			break
	await _frames(10)
	## Assert the body actually holds rows before trusting any width measured
	## against it -- `MISTAKES.md`'s "a probe must fail when the feature is
	## absent": a probe that reads an empty accordion's width and calls it
	## clean is the exact failure this whole function exists to catch.
	var gen_body: Control = null
	for e in (panel.get("categories") as Array):
		if String(e["title"]) == "Generate":
			gen_body = e["body"] as Control
			break
	_ok("rotate: Generate body visible", gen_body != null and gen_body.is_visible_in_tree(), true)
	_ok("rotate: Generate body non-empty",
		gen_body != null and _worst_row_min(gen_body)["w"] > 0.0, true)

	## dock_w alone is NOT a discriminating test of which shape is mounted:
	## `_apply_dock_widths()` (dcc_shell.gd) sets the LANDSCAPE dock to a fixed
	## 400 regardless of content, and a compact row that stayed compact after
	## a rotate-to-landscape still fits inside 400 with slack, so dock_w==400
	## alone would still read PASS. Found by this lane's own mutation test:
	## disabling `_build()`'s `DccTheme.watch_portrait` registration survived
	## every width-only assertion below. Read the actual mounted shape off a
	## named row instead, the same way `_gridgesture_probe.gd` does.
	var reflow: Array = panel.get("_slider_reflow_entries")
	var continentality: Dictionary = {}
	for e in reflow:
		if (e["label"] as Label).text == "Continentality":
			continentality = e
			break
	_ok("rotate: found a reflow entry to track", not continentality.is_empty(), true)

	var tag1 := "landscape 1" if boot_landscape else "portrait 1"
	print("  [%s] is_tablet_portrait=%s dock_w=%.1f dock_min=%.1f" % [
		tag1, str(DccTheme.is_tablet_portrait()), ld.size.x, ld.get_combined_minimum_size().x])
	_ok("%s classified" % tag1, DccTheme.is_tablet_portrait(), not boot_landscape)
	if boot_landscape:
		_ok("%s dock_w == 400 (W_DOCK_TABLET, unmoved by this lane)" % tag1, ld.size.x, 400.0)
	else:
		_ok("%s dock_w within 232 +-1" % tag1, absf(ld.size.x - 232.0) <= 1.0, true)
	_ok("%s Continentality mounted shape" % tag1, continentality.get("mounted", "?"),
		"compact" if not boot_landscape else "wide")
	_assert_stage_clip(tag1, panel)

	get_window().size = Vector2i(800, 1280) if boot_landscape else Vector2i(1280, 800)
	await _frames(12)
	var tag2 := "portrait" if boot_landscape else "landscape"
	print("  [%s] is_tablet_portrait=%s dock_w=%.1f dock_min=%.1f" % [
		tag2, str(DccTheme.is_tablet_portrait()), ld.size.x, ld.get_combined_minimum_size().x])
	_ok("%s classified" % tag2, DccTheme.is_tablet_portrait(), boot_landscape)
	if boot_landscape:
		_ok("%s dock_w within 232 +-1" % tag2, absf(ld.size.x - 232.0) <= 1.0, true)
	else:
		_ok("%s dock_w == 400 (W_DOCK_TABLET, unmoved by this lane)" % tag2, ld.size.x, 400.0)
	_ok("%s Continentality mounted shape" % tag2, continentality.get("mounted", "?"),
		"compact" if boot_landscape else "wide")
	_assert_stage_clip(tag2, panel)

	get_window().size = Vector2i(1280, 800) if boot_landscape else Vector2i(800, 1280)
	await _frames(12)
	var tag3 := "landscape 2" if boot_landscape else "portrait 2"
	print("  [%s] is_tablet_portrait=%s dock_w=%.1f dock_min=%.1f" % [
		tag3, str(DccTheme.is_tablet_portrait()), ld.size.x, ld.get_combined_minimum_size().x])
	_ok("%s classified" % tag3, DccTheme.is_tablet_portrait(), not boot_landscape)
	if boot_landscape:
		_ok("%s dock_w == 400 (W_DOCK_TABLET, unmoved by this lane)" % tag3, ld.size.x, 400.0)
	else:
		_ok("%s dock_w within 232 +-1" % tag3, absf(ld.size.x - 232.0) <= 1.0, true)
	_ok("%s Continentality mounted shape" % tag3, continentality.get("mounted", "?"),
		"compact" if not boot_landscape else "wide")
	_assert_stage_clip(tag3, panel)

	app.queue_free()
	await _frames(2)

## `--with-world`: every sweep above opens categories against a fresh,
## ungenerated panel ("no world" in the stage-status column) -- proof asked
## for "world generated" explicitly, so this runs one real (small, fast)
## `bridge.generate()` first, the same poll-`bridge.generating` shape
## `_genstage_probe.gd` uses, THEN sweeps all nine categories the normal way.
## A generate does not change any dial's own text (dials are inputs, not
## outputs), so the only real question is whether a populated Ecology
## readout or a "done N.NNs" stage-status column newly overflows -- state_label
## is already autowrap+expand-fill (near-0 min regardless of content) and
## `note()` is a fixed 190 px regardless of text length, so neither should,
## but "should not" is a claim and this is the measurement.
func _run_world_sweep() -> Array:
	var boot := await _boot_vp(800, 1280)
	var app: Node = boot["app"]
	var bridge = app.get("bridge")
	print("\n--- PORTRAIT 800x1280, WORLD GENERATED ---")
	bridge.generate({
		"seed": 424242, "width_km": 800.0, "grid_w": 256, "grid_h": 192,
		"sea_level": 0.5, "villages": false,
	})
	var waited := 0
	while bridge.generating and waited < 1200:
		await _frames(1)
		waited += 1
	_ok("generation finished before the poll gave up", bridge.generating, false)
	_ok("the bridge produced a world", bridge.has_world, true)
	await _frames(6)
	var rows := await _sweep_categories(app)
	_print_sweep(rows)
	app.queue_free()
	await _frames(2)
	return rows

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(1)
		return
	var args := OS.get_cmdline_user_args()
	var touch := "--force-touch" in args

	if touch and "--rotate" in args:
		await _run_rotate("--boot-landscape" in args)
		print("\n_worldportraitgrid_probe (rotate): ", _fail, " FAILURE(S)")
		get_tree().quit(1 if _fail > 0 else 0)
		return

	if touch and "--with-world" in args:
		var wrows := await _run_world_sweep()
		print("\n[assertions, world generated]")
		for r in wrows:
			_ok("world-generated %s body non-empty" % String(r["title"]),
				float((r["worst_row"] as Dictionary)["w"]) > 0.0, true)
			_assert_dock_width("world-generated", String(r["title"]), float(r["dock_w"]))
		print("\n_worldportraitgrid_probe (with-world): ", _fail, " FAILURE(S)")
		get_tree().quit(1 if _fail > 0 else 0)
		return

	if touch:
		var port_800 := await _run_sweep_frame("PORTRAIT 800x1280", 800, 1280)
		var port_900 := await _run_sweep_frame("PORTRAIT 900x1440", 900, 1440)
		var tland := await _run_sweep_frame("TABLET LANDSCAPE 1280x800", 1280, 800)
		var phone := await _run_sweep_frame("PHONE 1080x2340", 1080, 2340)
		print("\n[assertions]")
		## `MISTAKES.md`: "a probe must fail when the feature is absent" -- a
		## width measured against an empty accordion is not a measurement of
		## anything, so every row's own content is asserted non-empty ALONGSIDE
		## its width, not just printed beside it.
		for r in port_800:
			_ok("800x1280 %s body non-empty" % String(r["title"]),
				float((r["worst_row"] as Dictionary)["w"]) > 0.0, true)
			_assert_dock_width("800x1280", String(r["title"]), float(r["dock_w"]))
		for r in port_900:
			_ok("900x1440 %s body non-empty" % String(r["title"]),
				float((r["worst_row"] as Dictionary)["w"]) > 0.0, true)
			_assert_dock_width("900x1440", String(r["title"]), float(r["dock_w"]))
		for r in tland:
			_ok("1280x800(landscape) %s body non-empty" % String(r["title"]),
				float((r["worst_row"] as Dictionary)["w"]) > 0.0, true)
			_ok("1280x800(landscape) %s dock_w == 400" % String(r["title"]), r["dock_w"], 400.0)
		# Phone never consumes ROLE/TABLET_PORTRAIT (`DccTheme.role_px()`'s own
		# header) -- recorded, not asserted to a literal, since the phone
		# composition is a separate PHONE_* system this lane does not touch.
	else:
		await _run_sweep_frame("DESKTOP 1920x1080", 1920, 1080)
		await _run_sweep_frame("LAPTOP 1366x768", 1366, 768)

	print("\n_worldportraitgrid_probe: ", _fail, " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
