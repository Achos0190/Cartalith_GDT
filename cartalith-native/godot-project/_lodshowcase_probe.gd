extends Node
## Owner request 2026-09-20: "show me a map rendered at 2k with LOD tiles
## going down to LOD 8". Not a pass/fail probe -- a screenshot capture.
## Must run WINDOWED (headless has no real framebuffer to capture):
##
##   Godot_v4.7.1 --path . --resolution 2048x1152 --rendering-driver opengl3 _lodshowcase_probe.tscn -- --out <path.png>
##
## TILE_PX (cartalith-godot/src/lod_bridge.rs) is 256, not 512 -- reported
## honestly rather than silently matched to the request.

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	var out_path := "user://lod_showcase.png"
	var full_out_path := "user://lod_showcase_full.png"
	var args := OS.get_cmdline_user_args()
	for i in args.size():
		if args[i] == "--out" and i + 1 < args.size():
			out_path = args[i + 1]
		if args[i] == "--full-out" and i + 1 < args.size():
			full_out_path = args[i + 1]

	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(10)

	# Dismiss any welcome / project-picker dialog left open by boot.
	for w in get_tree().root.get_children():
		if w is Window and w != get_tree().root and w.visible and w.has_method("hide"):
			w.hide()
	await _frames(3)

	print("[gen] generating a 2048-wide world…")
	var bridge = app.bridge
	# Plain default archetype/sea_level -- the app's own defaults, matching
	# the owner's original request with no thumb on the scale. Earlier
	# attempts wrongly suspected this seed/archetype of generating 100%
	# ocean; that was this probe's own land-scan bug (fixed below), not a
	# generation problem -- see the land-scan comment.
	bridge.generate({
		"seed": 20260920, "width_km": 800.0, "grid_w": 2048, "grid_h": 1311,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	var waited := 0.0
	while bridge.generating and waited < 120.0:
		await get_tree().create_timer(0.25).timeout
		waited += 0.25
	if bridge.generating or not bridge.has_world:
		print("[FATAL] generation did not finish (has_world=", bridge.has_world, ")")
		get_tree().quit(1); return
	print("[gen] done, has_world=", bridge.has_world)
	await _frames(10)

	var vp: Node = app.viewport
	var win := get_window()
	var size := win.size
	print("[boot] window size = ", size)

	# Owner asked to see the whole map first -- capture the default "cover"
	# fit view (reset_view(), viewport_host.gd's own reset/reset-nav-pad
	# behaviour) before zooming into any one LOD-8 tile below. Dismiss
	# dialogs first so this capture is clean too.
	var opd0 = app.get("open_project_dialog")
	if opd0 != null and opd0 is Window and (opd0 as Window).visible:
		(opd0 as Window).hide()
	await _frames(3)
	if vp.has_method("reset_view"):
		vp.call("reset_view")
		await _frames(6)
	var full_img := get_viewport().get_texture().get_image()
	if full_img != null:
		var full_err := full_img.save_png(full_out_path)
		print("[shot-full] save_png(", full_out_path, ") -> ", full_err, " size=", full_img.get_size())
		print("[shot-full] real path -> ", ProjectSettings.globalize_path(full_out_path))
	else:
		print("[shot-full] FATAL: get_texture().get_image() returned null")

	# _do_center_landmasses() is a Whole-world-only op (it re-wraps in
	# longitude) and this run is Region mode -- confirmed a no-op by an
	# earlier attempt (status bar said so explicitly). Region mode has no
	# wrap to exploit, so find a real land cell directly instead: scan a
	# grid with the live sample_cell(gx,gy) bridge call (backs the SAMPLE
	# panel) and centre on whichever is most clearly inland -- highest
	# elevation among sampled land cells -- then move_view_to() pans the
	# camera onto it at the CURRENT zoom, no zoom change, so it composes
	# cleanly with the zoom loop below.
	#
	# CORRECTED 2026-09-20: `d.has("water")` is NOT "is this cell wet" --
	# sample_cell's own wrapper (cartalith-godot/src/lib.rs, the
	# `s.water_body` match) sets the "water" key to "land"/"ocean"/"lake"
	# for EVERY classified cell, land included, because `water_body:
	# Option<u8>` is `Some(0)` for land (an in-bounds classification entry),
	# not `None`. An earlier version of this probe scanned 26000 points,
	# found zero non-"water"-keyed cells, and concluded the terrain
	# generator was producing 100% ocean -- checked directly against
	# `absorb()`'s own civ.water_bodies counts for this exact run
	# (land=1,485,988 ocean=881,766 lake=317,174 out of 2,684,928, a normal
	# ~55% land world) and that was wrong: the generator was fine the whole
	# time, this scan's `is_water` check was reading the wrong condition.
	var g := Vector2i(2048, 1311)
	var best_gx := g.x / 2
	var best_gy := g.y / 2
	var best_elev := -INF
	var best_dict: Dictionary = {}
	var found_land := false
	var n_land := 0
	var n_water := 0
	var n_empty := 0
	var cols := 60
	var rows := 40
	for cy in rows:
		for cx in cols:
			var gx: int = int((float(cx) + 0.5) / float(cols) * g.x)
			var gy: int = int((float(cy) + 0.5) / float(rows) * g.y)
			var d: Dictionary = bridge.sample_cell(gx, gy)
			if d.is_empty():
				n_empty += 1
				continue
			var is_water: bool = d.get("water", "land") != "land"
			if is_water:
				n_water += 1
			else:
				n_land += 1
			if is_water:
				continue
			var elev: float = float(d.get("elevation", -INF))
			if elev > best_elev:
				best_elev = elev
				best_gx = gx
				best_gy = gy
				best_dict = d
				found_land = true
	print("[land] scan found_land=", found_land, " best=(", best_gx, ",", best_gy, ") elev=", best_elev,
		" n_land=", n_land, " n_water=", n_water, " n_empty=", n_empty)
	if not best_dict.is_empty():
		print("[land] best cell dict = ", best_dict)

	if vp.has_method("move_view_to"):
		vp.call("move_view_to", float(best_gx), float(best_gy))
		await _frames(4)
		print("[centre] camera moved to land cell (", best_gx, ",", best_gy, ")")

	# Zoom in step by step (the real, user-facing path -- _zoom_at also
	# triggers _update_lod internally) until lod_level_for_zoom reports 8,
	# recomputing the same screen_px_per_cell viewport_host.gd itself uses.
	# `_zoom_at` re-anchors on whatever screen point it is given each call;
	# passing screen centre every step keeps the land cell move_view_to()
	# just centred there, since the camera was already panned onto it.
	var target_level := 8
	var reached := -1
	for step in range(40):
		var native_scale: float = minf(float(size.x) / float(g.x), float(size.y) / float(g.y))
		var zoom_now: float = vp.get("_zoom")
		var screen_px_per_cell: float = native_scale * zoom_now
		var level: int = bridge.lod_level_for_zoom(screen_px_per_cell)
		if step % 5 == 0 or level >= target_level:
			print("  step ", step, " zoom=", zoom_now, " px/cell=", screen_px_per_cell, " level=", level)
		if level >= target_level:
			reached = level
			break
		vp.call("_zoom_at", Vector2(size) * 0.5, 1.6)
		await _frames(2)

	print("[zoom] reached LOD level ", reached, " (wanted ", target_level, ")")
	await _frames(20)  # let the deepest tiles finish synthesizing/drawing

	# The welcome dialog (app.open_project_dialog, an AcceptDialog) opens on a
	# deferred call after boot (app.gd::_open_welcome_when_drawn) and is not a
	# direct child of get_tree().root, so the tree-walk dismiss never found
	# it. Reference it directly instead.
	var opd = app.get("open_project_dialog")
	if opd != null and opd is Window and (opd as Window).visible:
		print("  [dismiss] closing open_project_dialog")
		(opd as Window).hide()
	var ppp = app.get("phone_project_picker")
	if ppp != null and ppp.has_method("close"):
		ppp.call("close")
	await _frames(10)

	var img := get_viewport().get_texture().get_image()
	if img == null:
		print("[FATAL] get_texture().get_image() returned null -- this run is not actually windowed/rendering")
		get_tree().quit(1); return
	var err := img.save_png(out_path)
	print("[shot] save_png(", out_path, ") -> ", err, " size=", img.get_size())
	print("[shot] real path -> ", ProjectSettings.globalize_path(out_path))

	print("_lodshowcase_probe: DONE reached_level=", reached, " img=", img.get_size())
	get_tree().quit(0 if reached >= target_level else 2)
