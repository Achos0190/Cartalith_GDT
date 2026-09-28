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
##   WH/WG  the same pair for WAYS (2026-09-28): way W is deleted before year 0
##          and restored by undo after it (added at 10); way V is deleted after
##          year 0 (removed at 10). Each leg centres the camera on its way.
##   EO     Exist only re-applies on a STRIP scrub (`DccShell.tl_set_year`),
##          for the dropped pin and for way W: year 0 filtered vs year 0
##          unfiltered must differ at the object, and it must leave the overlay.
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


## [changed px that moved toward `ink`, changed px] inside the box around `c`:
## "toward" means the after-pixel is closer to `ink` than the before-pixel by
## more than one 8-bit step, per channel summed.
func _toward(a: Image, b: Image, c: Vector2, ink: Color) -> Array:
	var toward := 0
	var changed := 0
	for y in range(int(c.y) - BOX, int(c.y) + BOX):
		for x in range(int(c.x) - BOX, int(c.x) + BOX):
			if x < 0 or y < 0 or x >= a.get_width() or y >= a.get_height():
				continue
			var pa := a.get_pixel(x, y)
			var pb := b.get_pixel(x, y)
			if absf(pa.r - pb.r) <= 0.004 and absf(pa.g - pb.g) <= 0.004 and absf(pa.b - pb.b) <= 0.004:
				continue
			changed += 1
			var da := absf(pa.r - ink.r) + absf(pa.g - ink.g) + absf(pa.b - ink.b)
			var db := absf(pb.r - ink.r) + absf(pb.g - ink.g) + absf(pb.b - ink.b)
			if db < da - 0.004:
				toward += 1
	return [toward, changed]


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

	## Two generated ways for the way legs, at least `WAY_CLEAR_CELLS` from the
	## settlement marks so neither leg's box can see the other's marks.
	var picked := _pick_two_ways(bridge, gone_xy)
	if picked.size() < 2:
		print("### TLPINS ABORT: fewer than two drawable generated ways clear of the settlement marks ###")
		_aborted = true
		get_tree().quit(2)
		return
	var iw: int = picked[0]
	var iv: int = picked[1]
	var w_tid := int(bridge.way_get("generated", iw)["tid"])
	var v_tid := int(bridge.way_get("generated", iv)["tid"])
	print("fixture: way W (to be ADDED) index %d tid=%d, way V (to be REMOVED) index %d tid=%d" % [iw, w_tid, iv, v_tid])

	## W is absent from year 0 and present in year 10: delete it, record year 0,
	## then undo the delete (Ruling BA's way undo puts it back at the same index,
	## same tid). There is no other way to give a generated way a year it was
	## not in -- manual ways carry no tid and never reach a snapshot.
	bridge.way_delete("generated", iw)
	bridge.civ_add_year(0)
	## Move the cursor OFF year 0 before editing: an unrecorded year moves only
	## the cursor (Ruling AT), so the edits below cannot reach year 0's record.
	bridge.civ_goto_year(10)
	var undone: String = bridge.undo_last()
	_ok("fixture: undo restores way W with its tid", int(bridge.way_get("generated", iw).get("tid", -1)) == w_tid,
		"undo_last='%s'" % undone)
	## V is present in year 0 and absent from year 10. Its index is unchanged:
	## W's delete and undo cancel.
	bridge.way_delete("generated", iv)
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
	_ok("fixture: year 10's diff names way W as added",
		(diff.get("added", PackedInt64Array()) as PackedInt64Array).has(w_tid))
	_ok("fixture: year 10's diff names way V as removed",
		(diff.get("removed", PackedInt64Array()) as PackedInt64Array).has(v_tid))
	var v_ghost: Dictionary = {}
	for r: Dictionary in bridge.civ_year_diff_removed_ways(10):
		if int(r["tid"]) == v_tid:
			v_ghost = r
	_ok("bridge: civ_year_diff_removed_ways(10) returns way V with its geometry",
		not v_ghost.is_empty() and (v_ghost["points"] as PackedVector2Array).size() >= 2)
	_ok("bridge: ...and not way W, which is present", _row_by_tid(bridge.civ_year_diff_removed_ways(10), w_tid).is_empty())

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

	# -- WH+ / WH-: Highlight new on way W --------------------------------------
	var w_live := _row_by_tid(ov._roads, w_tid)
	_ok("WH fixture: way W is live in the overlay's roads, carrying its tid", not w_live.is_empty())
	if w_live.is_empty():
		return
	var w_pt := _mid_point(w_live)
	var w_px := await _centre_on(ov, w_pt)
	var wbase := await _shot("way_base")
	hl.button_pressed = true
	var wh_on := await _shot("way_highlight")
	_crop(wh_on, w_px, "way_highlight")
	var wh := _diff_box(wbase, wh_on, w_px, good)
	_ok("WH+ Highlight new strokes a halo along way W", wh[0] > 0, "changed=%d" % wh[0])
	## The way halo is the reference's 0.6-alpha stroke, so over terrain it
	## blends and rarely lands within `_diff_box`'s tolerance of pure `good`
	## (phone: 0 of 371 changed px did). What must hold is the direction: the
	## pixels it changes move TOWARD `good`. A halo in any other ink fails.
	var tw := _toward(wbase, wh_on, w_px, good)
	_ok("WH+ ...in the `good` token's ink (changed px move toward it)", tw[0] > 0 and tw[0] * 2 > tw[1],
		"toward=%d of changed=%d (strict good-ink px=%d)" % [tw[0], tw[1], wh[1]])
	hl.button_pressed = false
	var wh_off := await _shot("way_highlight_off")
	_ok("WH- toggling Highlight new off removes it", _diff_box(wbase, wh_off, w_px)[0] == 0)

	# -- WG+ / WG-: Ghost removed on way V ---------------------------------------
	_ok("WG fixture: way V is NOT live (it was deleted)", _row_by_tid(ov._roads, v_tid).is_empty())
	var v_pt := _mid_point(v_ghost)
	var v_px := await _centre_on(ov, v_pt)
	var vbase := await _shot("wayghost_base")
	gh.button_pressed = true
	var wg_on := await _shot("way_ghost")
	_crop(vbase, v_px, "way_ghost_before")
	_crop(wg_on, v_px, "way_ghost")
	counts = ov.timeline_mark_counts()
	_ok("WG+ overlay holds the ghost way", int(counts["ghost_ways"]) > 0, str(counts))
	var wg := _diff_box(vbase, wg_on, v_px)
	_ok("WG+ Ghost removed draws way V's ghost where it ran", wg[0] > 0, "changed=%d" % wg[0])
	gh.button_pressed = false
	var wg_off := await _shot("way_ghost_off")
	_ok("WG- toggling Ghost removed off removes it", _diff_box(vbase, wg_off, v_px)[0] == 0)

	# -- EO: Exist only re-applies on a strip scrub -------------------------------
	## `DccShell.tl_set_year` is the strip's own entry point: it moves the cursor
	## and emits `timeline_changed`, and never calls the workspace directly.
	## Year 0 holds neither the dropped town nor way W, so with Exist only on a
	## scrub from 10 to 0 must drop both from the map. The comparison frame is the
	## SAME year with Exist only off, so terrain and territory are identical and
	## only the filter can differ. Before this fix the scrub left year 10's
	## filtered arrays in place, both frames matched, and these checks read 0.
	await _eo_leg(ws, ov, new_px, (gone_xy + new_xy) * 0.5, "pin", func(): return _row_by_tid(ov._settlements, new_tid).is_empty())
	await _eo_leg(ws, ov, Vector2.ZERO, w_pt, "way", func(): return _row_by_tid(ov._roads, w_tid).is_empty())


## One Exist-only scrub leg, the camera centred on `centre`. `px` is where to
## measure; ZERO means "the centre itself" (the way leg). The pin leg passes the
## pin's own px, measured earlier at the same settlement-camera centre --
## `_centre_on` puts that centre back exactly where `move_view_to(mid)` did,
## to within the half-cell it takes off (a constant, so `px` stays valid).
func _eo_leg(ws: Node, ov: Control, px: Vector2, centre: Vector2, tag: String, gone_from_overlay: Callable) -> void:
	app.tl_set_year(10)
	await _frames(4)
	var p := await _centre_on(ov, centre + Vector2(0.5, 0.5) if px != Vector2.ZERO else centre)
	if px == Vector2.ZERO:
		px = p
	var eo := _toggle(ws, "Exist only")
	eo.button_pressed = true
	await _frames(2)
	_ok("EO(%s) fixture: at year 10 with Exist only on, the object is on the map" % tag, not gone_from_overlay.call())
	app.tl_set_year(0)
	var s1 := await _shot("eo_%s_scrubbed" % tag)
	_ok("EO(%s) a strip scrub to year 0 re-applies Exist only (the object leaves the overlay)" % tag,
		gone_from_overlay.call(), "cursor=%d" % app.bridge.get_civ_year())
	eo = _toggle(ws, "Exist only")
	eo.button_pressed = false
	var s2 := await _shot("eo_%s_unfiltered" % tag)
	_crop(s1, px, "eo_%s_scrubbed" % tag)
	_crop(s2, px, "eo_%s_unfiltered" % tag)
	var d := _diff_box(s1, s2, px)
	_ok("EO(%s) ...and the pixels agree: year 0 filtered differs from year 0 unfiltered there" % tag,
		d[0] > 0, "changed=%d" % d[0])
	app.tl_set_year(10)
	await _frames(4)


## The first row of `rows` whose `tid` is `tid`, or `{}`.
func _row_by_tid(rows: Array, tid: int) -> Dictionary:
	for r: Dictionary in rows:
		if r.has("tid") and int(r["tid"]) == tid:
			return r
	return {}


## The middle render point of a way row -- away from both junction ends, where
## neighbouring ways crowd in.
func _mid_point(row: Dictionary) -> Vector2:
	var pts: PackedVector2Array = row["points"]
	return pts[pts.size() / 2]


## Centres the camera on way-space point `p` and returns its viewport px.
## `move_view_to` centres a CELL (`+0.5`); way points are continuous, so the
## half cell is taken back off.
func _centre_on(ov: Control, p: Vector2) -> Vector2:
	app.viewport.move_view_to(p.x - 0.5, p.y - 0.5)
	await _settle()
	var local: Vector2 = ov._point_to_screen(p, ov._displayed_rect())
	return ov.get_global_transform_with_canvas() * local


## Grid distance every point of a way leg's ways must keep from the settlement
## marks, so the settlement legs' boxes and the way legs' boxes never overlap
## at either form's zoom (desktop ~8 px/cell, phone ~20 px/cell against a
## 40 px box).
const WAY_CLEAR_CELLS := 30.0

## Indices into the generated way store of two drawable ways -- not hidden, a
## tier every zoom this probe uses draws (`road`/`regional`/`highway`), and
## every control point at least `WAY_CLEAR_CELLS` from `avoid` -- nearest first.
func _pick_two_ways(bridge, avoid: Vector2) -> Array:
	var cands: Array = []
	for i in 5000:
		var w: Dictionary = bridge.way_get("generated", i)
		if w.is_empty():
			break
		if bool(w.get("hidden", false)) or not w.has("tid"):
			continue
		if not ["road", "regional", "highway"].has(String(w["way_type"])):
			continue
		var near := INF
		for p in (w["points"] as PackedVector2Array):
			near = minf(near, p.distance_to(avoid))
		if near >= WAY_CLEAR_CELLS:
			cands.append([near, i])
	cands.sort_custom(func(a, b): return a[0] < b[0])
	return [] if cands.size() < 2 else [cands[0][1], cands[1][1]]


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
