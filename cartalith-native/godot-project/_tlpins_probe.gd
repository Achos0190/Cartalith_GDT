extends Node
## CV-03 (`GUI_GAP_REGISTER.md`), `TIMELINE_SCOPE.md` success criterion 5: the
## Timeline's *Highlight new* and *Ghost removed* toggles must change what the
## map draws. Drives the real toggles in `civilization_workspace.gd`'s Timeline
## Filters section against a real generated world, and reads the framebuffer.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _tlpins_probe.tscn
##   Godot_v4.7.1-stable_win64_console.exe --path . _tlpins_probe.tscn -- --vp 1080x2340 --force-touch
##
## **Windowed, never `--headless`** (`MISTAKES.md`: a headless pixel probe
## passes vacuously). `--vp WxH` sizes the host SubViewport; `--force-touch` is
## the shell's own flag and is accepted here only so the same line boots the
## phone form. Any other argument aborts (exit 2) rather than being ignored.
##
## Fixture: year 0 records the live world; then one settlement is deleted and a
## new one dropped, and year 10 records that. At year 10 the deleted one is
## `removed` and the dropped one is `added`.
##
## Each check is a **differential** against the same frame with both toggles
## off, measured inside a box around the mark's own screen position:
##   H+  Highlight new ON moves pixels around the new pin, some of them the live
##       `good` token (the halo's ink), and none around the ghost's spot
##   H-  Highlight new OFF again restores both boxes exactly
##   G+  Ghost removed ON moves pixels around the removed settlement's old spot
##   G-  Ghost removed OFF again restores it exactly
## The "moves pixels" half is its own positive control: a mark that did not
## reach the screen reads 0 and fails, whatever the node tree says.
##
## Screenshots land beside this file as `_tlpins_<form>_<state>.png`, plus a 4x
## crop of each box (`..._crop.png`) for looking at the marks themselves.
##
## Exit 0 = green, 1 = a check failed, 2 = the probe could not run.

const SEED := 552017
## Half-size of the measured box, in viewport px. Wide enough to hold a
## capital's halo at the probe's zoom; checked by the crops, not assumed.
const BOX := 40

var app: Node
var _vp: SubViewport
var _fails := 0
var _checks := 0
var _form := "desktop"
var _aborted := false   ## set by every exit-2 path, so the summary line is not printed after it


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("%s  %s%s" % ["PASS" if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _settle() -> void:
	await _frames(3)
	await RenderingServer.frame_post_draw
	await get_tree().process_frame
	await RenderingServer.frame_post_draw


func _shot(tag: String) -> Image:
	## Long enough for deep-zoom tiles and deferred redraws to land, so two
	## shots differ only by what the probe changed between them.
	await get_tree().create_timer(1.5).timeout
	await _settle()
	var img := _vp.get_texture().get_image()
	img.save_png("res://_tlpins_%s_%s.png" % [_form, tag])
	return img


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


## The CheckBox in the toggle row whose label reads `label_text`.
## `DccWidgets.toggle` puts the label and the box in one row container.
func _toggle(ws: Node, label_text: String) -> CheckBox:
	var all: Array = []
	_walk(ws, all)
	for n in all:
		if n is Label and (n as Label).text == label_text:
			for c in n.get_parent().get_children():
				if c is CheckBox:
					return c
	return null


## Viewport-pixel position of grid cell (x, y) as the overlay draws a pin there.
func _cell_px(ov: Control, x: float, y: float) -> Vector2:
	var local: Vector2 = ov._cell_to_screen(Vector2(x, y), ov._displayed_rect())
	return ov.get_global_transform_with_canvas() * local


## [changed pixels, pixels close to `ink`] inside the box around `c`.
func _diff_box(a: Image, b: Image, c: Vector2, ink: Color = Color(0, 0, 0, 0)) -> Array:
	var changed := 0
	var inked := 0
	for y in range(int(c.y) - BOX, int(c.y) + BOX):
		for x in range(int(c.x) - BOX, int(c.x) + BOX):
			if x < 0 or y < 0 or x >= a.get_width() or y >= a.get_height():
				continue
			var pa := a.get_pixel(x, y)
			var pb := b.get_pixel(x, y)
			if absf(pa.r - pb.r) > 0.004 or absf(pa.g - pb.g) > 0.004 or absf(pa.b - pb.b) > 0.004:
				changed += 1
				if ink.a > 0.0 and absf(pb.r - ink.r) < 0.06 and absf(pb.g - ink.g) < 0.06 and absf(pb.b - ink.b) < 0.06:
					inked += 1
	return [changed, inked]


func _crop(img: Image, c: Vector2, tag: String) -> void:
	var r := Rect2i(Vector2i(c) - Vector2i(BOX, BOX), Vector2i(BOX * 2, BOX * 2)).intersection(
		Rect2i(Vector2i.ZERO, img.get_size()))
	if r.size.x <= 0 or r.size.y <= 0:
		return
	var crop := img.get_region(r)
	crop.resize(r.size.x * 4, r.size.y * 4, Image.INTERPOLATE_NEAREST)
	crop.save_png("res://_tlpins_%s_%s_crop.png" % [_form, tag])


func _run() -> void:
	var bridge = app.bridge
	app.select_domain("civilization")
	await _frames(6)
	var ws: Node = null
	for w in app._workspaces:
		if w.has_method("_tl_push_marks"):
			ws = w
			break
	if ws == null:
		print("### TLPINS ABORT: no workspace carries _tl_push_marks -- stale shell copy ###")
		_aborted = true
		get_tree().quit(2)
		return
	if not bridge.world_gen.has_method("civ_year_diff_removed"):
		print("### TLPINS ABORT: this DLL has no civ_year_diff_removed -- stale binary ###")
		_aborted = true
		get_tree().quit(2)
		return
	var ov: Control = app.viewport.overlay

	# -- fixture: pick a settlement to remove, and a spot for a new one -------
	var places: Array = bridge.settlements()
	var g: Vector2i = bridge.grid_size()
	var victim := -1
	var best := INF
	for i in places.size():
		var s: Dictionary = places[i]
		if String(s["kind"]) != "town" or bool(s["capital"]):
			continue
		var d := Vector2(s["x"], s["y"]).distance_to(Vector2(g) * 0.5)
		if d < best:
			best = d
			victim = i
	if victim < 0:
		print("### TLPINS ABORT: no non-capital town to remove ###")
		_aborted = true
		get_tree().quit(2)
		return
	var vs: Dictionary = places[victim]
	var gone_tid := int(vs["tid"])
	var gone_xy := Vector2(vs["x"], vs["y"])
	print("fixture: removing '%s' tid=%d at %s" % [vs["name"], gone_tid, gone_xy])

	bridge.civ_add_year(0)
	## Move the cursor OFF year 0 before editing: an unrecorded year moves only
	## the cursor (Ruling AT), so the edits below cannot reach year 0's record.
	bridge.civ_goto_year(10)
	bridge.civ_delete_settlement(victim)
	## A new town a few cells from the removed one, so both fit one zoomed view.
	## A drop onto a spot an existing place already claims returns THAT place's
	## index and adds nothing (measured on this world's first run), so success is
	## judged by the roster growing, not by the return value.
	var new_idx := -1
	var n_before: int = bridge.settlements().size()
	for off in [Vector2(10, 0), Vector2(-10, 0), Vector2(0, 10), Vector2(0, -10), Vector2(14, 6),
			Vector2(-14, -6), Vector2(6, -14), Vector2(-6, 14), Vector2(18, 0), Vector2(0, 18)]:
		var p: Vector2 = gone_xy + off
		new_idx = bridge.civ_drop_settlement(p.x, p.y, "town", int(vs["faction"]), "Probe New", false)
		if bridge.settlements().size() > n_before:
			break
		new_idx = -1
	if new_idx < 0:
		print("### TLPINS ABORT: civ_drop_settlement refused every spot ###")
		_aborted = true
		get_tree().quit(2)
		return
	## Found by tid, not by the returned index: the index answers a different
	## question (it did not name the dropped town on the first run of this
	## probe), while a tid no settlement held before the drop can only be it.
	var before_tids := {}
	for s0: Dictionary in places:
		before_tids[int(s0["tid"])] = true
	var ns: Dictionary = {}
	for s1: Dictionary in bridge.settlements():
		if not before_tids.has(int(s1["tid"])):
			ns = s1
	if ns.is_empty():
		print("### TLPINS ABORT: no settlement carries a new tid after the drop ###")
		_aborted = true
		get_tree().quit(2)
		return
	var new_tid := int(ns["tid"])
	var new_xy := Vector2(ns["x"], ns["y"])
	print("fixture: dropped '%s' tid=%d at %s" % [ns["name"], new_tid, new_xy])
	bridge.civ_add_year(10)
	ws._tl_goto_year(10)
	ws._rebuild_timeline()

	var diff: Dictionary = bridge.civ_year_diff(10)
	_ok("fixture: year 10's diff names the new tid as added",
		(diff.get("added", PackedInt64Array()) as PackedInt64Array).has(new_tid), "added=%s" % str(diff.get("added")))
	_ok("fixture: year 10's diff names the deleted tid as removed",
		(diff.get("removed", PackedInt64Array()) as PackedInt64Array).has(gone_tid), "removed=%s" % str(diff.get("removed")))
	var ghosts: Array = bridge.civ_year_diff_removed(10)
	var ghost_row: Dictionary = {}
	for r: Dictionary in ghosts:
		if int(r["tid"]) == gone_tid:
			ghost_row = r
	_ok("bridge: civ_year_diff_removed(10) returns the deleted settlement at its year-0 cell",
		not ghost_row.is_empty() and Vector2(ghost_row["x"], ghost_row["y"]) == gone_xy, str(ghosts))
	_ok("bridge: civ_year_diff_removed(0) is empty (no year before the first)",
		(bridge.civ_year_diff_removed(0) as Array).is_empty())

	# -- camera: zoom in and centre between the two marks ---------------------
	app.viewport.zoom_step(3.0)
	var mid := (gone_xy + new_xy) * 0.5
	app.viewport.move_view_to(mid.x, mid.y)
	await _settle()
	var gone_px := _cell_px(ov, gone_xy.x, gone_xy.y)
	var new_px := _cell_px(ov, new_xy.x, new_xy.y)
	var host_rect: Rect2 = app.viewport.get_global_rect()
	print("screen: ghost spot %s, new pin %s, map host %s, zoom %.2f, theme %s" % [
		gone_px, new_px, host_rect, app.viewport.zoom(), "dark" if DccTheme.c("bg").v < 0.5 else "light"])
	_ok("both marks' spots are inside the map host", host_rect.has_point(gone_px) and host_rect.has_point(new_px))

	var hl := _toggle(ws, "Highlight new")
	var gh := _toggle(ws, "Ghost removed")
	_ok("both toggles are found in the Timeline filters", hl != null and gh != null)
	if hl == null or gh == null:
		return
	_ok("Highlight new is enabled", not hl.disabled)
	_ok("Ghost removed is enabled", not gh.disabled)
	hl.button_pressed = false
	gh.button_pressed = false
	var base := await _shot("base")
	_crop(base, new_px, "base_new")
	_crop(base, gone_px, "base_gone")

	# -- H+ / H- ----------------------------------------------------------------
	hl.button_pressed = true
	var h_on := await _shot("highlight")
	_crop(h_on, new_px, "highlight_new")
	var counts: Dictionary = ov.timeline_mark_counts()
	_ok("H+ overlay holds the added tids", int(counts["added"]) > 0 and int(counts["ghosts"]) == 0, str(counts))
	var good: Color = DccTheme.c("good")
	var hn := _diff_box(base, h_on, new_px, good)
	_ok("H+ Highlight new draws a halo at the new pin", hn[0] > 0, "changed=%d" % hn[0])
	_ok("H+ ...in the `good` token's ink", hn[1] > 0, "good-ink px=%d" % hn[1])
	var hg := _diff_box(base, h_on, gone_px)
	_ok("H+ ...and nothing at the removed settlement's spot", hg[0] == 0, "changed=%d" % hg[0])
	hl.button_pressed = false
	var h_off := await _shot("highlight_off")
	var hn0 := _diff_box(base, h_off, new_px)
	_ok("H- toggling Highlight new off removes the halo", hn0[0] == 0, "changed=%d" % hn0[0])

	# -- G+ / G- ----------------------------------------------------------------
	gh.button_pressed = true
	var g_on := await _shot("ghost")
	_crop(g_on, gone_px, "ghost_gone")
	counts = ov.timeline_mark_counts()
	_ok("G+ overlay holds the ghost rows", int(counts["ghosts"]) > 0 and int(counts["added"]) == 0, str(counts))
	var gg := _diff_box(base, g_on, gone_px)
	_ok("G+ Ghost removed draws a faded pin where the removed settlement stood", gg[0] > 0, "changed=%d" % gg[0])
	var gn := _diff_box(base, g_on, new_px)
	_ok("G+ ...and does not touch the new pin", gn[0] == 0, "changed=%d" % gn[0])
	gh.button_pressed = false
	var g_off := await _shot("ghost_off")
	var gg0 := _diff_box(base, g_off, gone_px)
	_ok("G- toggling Ghost removed off removes the ghost", gg0[0] == 0, "changed=%d" % gg0[0])

	# -- both, for the eye ------------------------------------------------------
	hl.button_pressed = true
	gh.button_pressed = true
	var both := await _shot("both")
	_crop(both, (gone_px + new_px) * 0.5, "both_mid")
	hl.button_pressed = false
	gh.button_pressed = false


func _ready() -> void:
	var vp_size := Vector2i(1600, 1000)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a == "--vp" and i + 1 < args.size():
			var wh := args[i + 1].split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
			continue
		if a == "--force-touch":
			i += 1
			continue
		print("### TLPINS ABORT: unknown argument '%s' ###" % a)
		_aborted = true
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("### TLPINS WATCHDOG ###"); get_tree().quit(3))
	wd.start()

	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.0).timeout

	var bridge = app.bridge
	bridge.generate({
		"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	_form = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("FORM booted as %s at %s" % [_form, _vp.size])
	await _run()
	if _aborted:
		return
	print("### TLPINS %s  %d/%d checks passed ###" % ["GREEN" if _fails == 0 else "RED", _checks - _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
