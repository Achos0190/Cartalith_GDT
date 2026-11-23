extends RefCounted
## Ruling HM (owner, 2026-11-23): a river town is laid round the drawn river's edge with a river as wide as the map
## draws it, and its marker stands on its market's bank. Driven through the REAL shell on one form; run once per form:
##   python -I cartalith-native/tools/fennara_drive.py GODOT <project copy> res://tools/town_bank_drive.gd \
##       --scene res://shell/app.tscn -- <the form's window and user args>
## (take `C:/cw/.win_lock` yourself, with an owner file, for the run). Inputs are environment variables, since the
## shell reads its own user arguments:
##   HM_OUT     the directory the crops are saved to (required);
##   HM_SEED, HM_GW, HM_GH (default 483920, 384, 288), HM_VILLAGES (default 1), HM_PARAMS (a `key: value` file of an
##   owner's world, applied before generating);
##   HM_TOWNS   "i,j,k": the settlements to frame by index, or HM_CELLS "x:y,x:y" by cell, or HM_AUTO=n to have
##              `_choose` pick n (two whose market's bank flipped, then the largest moved river towns by class);
##   HM_MAP_ZOOMS (default "1,4"), HM_MUTATE ("off": the after state is left off, so the checks below must FAIL).
## Per town and per switch state (off = before the ruling, on = after) it saves a crop of the map round the town at each
## zoom and of the City Viewer's plan at its opening frame and at x4:
##   <HM_OUT>/<form>_t<index>_<off|on>_map_x<zoom>.png, ..._city_x<1|4>.png
## Checks (each prints `ok` or `FAIL`; the launcher exits 1 on any FAIL), all with the switch on:
##   1. the ruling reaches at least one framed town (laid round a point that is not its cell, or its river now as wide
##      as the map draws it: the positive control), and moves no framed town with the switch off;
##   2. THE MARKER: the overlay draws each framed marker at its cell plus the engine's offset for the density on screen,
##      and a press at that point finds that settlement;
##   3. ONE BANK: the engine's census says the marker and the market of every framed town are on one bank at x1 and x4;
##   4. NOTHING OVER THE RIVER: no building of a framed town lies over a river the map draws.
##   6. THE EXPORT COPY: an export overlay built from the live one asks for bank points at the export's density and
##      draws its markers there.
##   7. A FROZEN TOWN STAYS FROZEN: a town frozen with the switch off is served from its save, unchanged, with the
##      switch on, and its check says \"civ bank origin is now on\".
##   5. THE SWITCH REACHES THE TOWNS DRAWN: each write of `civ.bank_sites` emits `EngineBridge.urban_params_changed`.

var _fails := 0
var app: Node
var bridge
var _form := "desktop"
var _out := ""


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _wait_for(ctx, t: String, secs: float = 20.0) -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		for e in DriveCtx.dump_ui(ctx, t):
			return e
		await ctx.wait(0.25)
	return {}


func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))


## Past the boot card: the phone's worlds list, or the desktop and tablet welcome (`drive_proof.gd`).
func _past_welcome(ctx) -> void:
	if app.get_viewport().get_visible_rect().size.x < 700.0:
		var nw := await _wait_for(ctx, "+ NEW WORLD")
		if not nw.is_empty():
			await _click_entry(ctx, nw)
			var cancel := await _wait_for(ctx, "CANCEL")
			if not cancel.is_empty():
				await _click_entry(ctx, cancel)
		await ctx.wait(1.0)
		return
	var cont := await _wait_for(ctx, "Continue without a world")
	if not cont.is_empty():
		await _click_entry(ctx, cont)
		await ctx.wait(0.6)


func _env(k: String, d: String) -> String:
	var v := OS.get_environment(k)
	return v if v != "" else d


## The window's pixels round `centre` (window coordinates), `half` each way, as an image.
func _crop(centre: Vector2, half: int) -> Image:
	var img: Image = app.get_viewport().get_texture().get_image()
	var k: float = float(img.get_width()) / app.get_viewport().get_visible_rect().size.x
	var r := Rect2i(Vector2i(centre * k) - Vector2i(half, half), Vector2i(half, half) * 2).intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	return img.get_region(r)


func _save(img: Image, town: int, state: String, what: String) -> void:
	img.save_png(_out.path_join("%s_t%d_%s_%s.png" % [_form, town, state, what]))


## The marker point of settlement `k` in window pixels, as the overlay draws it now.
func _marker_px(k: int) -> Vector2:
	var ov = app.viewport.overlay
	return ov.get_global_transform_with_canvas() * ov._settle_pos(ov._settlements[k], ov.displayed_rect())


## `n` river towns of this world for the frames: first up to two whose market stood across a drawn river from their
## marker before the ruling (switch off) and no longer does, then the most populous towns the ruling moves. Lays every
## settlement out twice on the main thread (a minute or two at 384 cells); the choice is logged as an `HM_TOWNS` value so
## the other forms frame the same towns.
func _choose(ctx, places: Array, n: int) -> Array:
	var all := PackedInt32Array()
	for k in places.size():
		all.append(k)
	var host = app.viewport
	bridge.param_set("civ.bank_sites", false)
	await ctx.wait(0.5)
	var off: Array = bridge.world_gen.town_bank_census(all, host.size.x, host.size.y)
	bridge.param_set("civ.bank_sites", true)
	await ctx.wait(0.5)
	var on: Array = bridge.world_gen.town_bank_census(all, host.size.x, host.size.y)
	var flipped: Array = []
	var moved: Array = []
	for k in mini(off.size(), on.size()):
		var a: Dictionary = off[k]
		var b: Dictionary = on[k]
		if not (a.get("river", false) and b.get("laid", false) and b.has("origin_m")):
			continue
		## Not at the map's edge: the crop round it must be map, and the frame's own margin takes no press.
		var g: Vector2i = bridge.grid_size()
		if mini(mini(int(places[k]["x"]), int(places[k]["y"])), mini(g.x - int(places[k]["x"]), g.y - int(places[k]["y"]))) < 14:
			continue
		if (int(a.get("side_x1", 1)) < 0 or int(a.get("side_x4", 1)) < 0) and int(b.get("side_x1", 1)) > 0 and int(b.get("side_x4", 1)) > 0:
			flipped.append(k)
		else:
			moved.append(k)
	## Largest class first (the roster carries no population here); the roster's own order within a class.
	var rank := {"capital": 0, "metropolis": 1, "city": 2, "town": 3, "village": 4, "hamlet": 5}
	moved.sort_custom(func(x, y):
		var rx := int(rank.get(String(places[x]["kind"]), 9))
		var ry := int(rank.get(String(places[y]["kind"]), 9))
		return rx < ry or (rx == ry and x < y))
	var out: Array = flipped.slice(0, 2) + moved
	out = out.slice(0, n)
	ctx.log("chosen: %d flipped of %d, HM_TOWNS=%s" % [mini(2, flipped.size()), flipped.size(), ",".join(out.map(func(k): return str(k)))])
	return out


func run(ctx) -> void:
	app = ctx.get_scene_root()
	if not ("bridge" in app):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	_out = _env("HM_OUT", "")
	if _out == "":
		ctx.error("HM_OUT is required")
		return
	DirAccess.make_dir_recursive_absolute(_out)
	var size: Vector2 = app.get_viewport().get_visible_rect().size
	_form = "phone" if size.x < 700.0 else ("tablet" if bool(DccTheme.is_tablet()) else "desktop")
	ctx.log("form %s, window %s" % [_form, str(size)])
	await _past_welcome(ctx)

	var req := {"seed": int(_env("HM_SEED", "483920")), "width_km": 800.0, "grid_w": int(_env("HM_GW", "384")), "grid_h": int(_env("HM_GH", "288")),
		"archetype": "", "villages": _env("HM_VILLAGES", "1") == "1"}
	var pfile := _env("HM_PARAMS", "")
	if pfile != "":
		var f := FileAccess.open(pfile, FileAccess.READ)
		var known: Dictionary = bridge.world_gen.get_params()
		var set := {}
		while f != null and not f.eof_reached():
			var line := f.get_line()
			var key := line.get_slice(": ", 0)
			if not known.has(key) or line.find(": ") < 0:
				continue
			var v := line.get_slice(": ", 1).strip_edges()
			match typeof(known[key]):
				TYPE_BOOL: set[key] = v == "true"
				TYPE_INT: set[key] = int(v)
				TYPE_FLOAT: set[key] = float(v)
				_: set[key] = v
		ctx.log("params from %s: %d keys" % [pfile, set.size()])
		## An owner's world is made by the engine call the shell's own worker makes (`EngineBridge._worker`), on this
		## thread, and handed to the shell as `EngineBridge._finish` hands it over. Not through `EngineBridge.generate`:
		## that restores the current rules for a new seed and sets the sea level and four experimental flags from its
		## request's defaults, so the world it made from this file was not the owner's (no settlement at his cell).
		bridge.world_gen.set_params(set)
		bridge.world_gen.set_villages_enabled(bool(req["villages"]))
		bridge.world_gen.generate_sized(int(req["seed"]), float(req["width_km"]), int(req["grid_w"]), int(req["grid_h"]))
		bridge.last_width_km = bridge.world_gen.get_map_width_km()
		bridge.last_height_km = bridge.world_gen.get_map_height_km()
		bridge.has_world = true
		bridge.params_dirty = false
		bridge.generation_finished.emit(true)
		bridge.params_applied.emit()
	else:
		bridge.generate(req)
		await ctx.wait(0.2)
		var t0 := Time.get_ticks_msec()
		while bool(bridge.generating) and Time.get_ticks_msec() - t0 < 900000:
			await ctx.wait(0.25)
	await ctx.wait(2.0)
	if app.get("open_project_dialog") != null and app.open_project_dialog:
		app.open_project_dialog.hide()
	_check(ctx, "a world exists", bool(bridge.has_world))
	var places: Array = bridge.settlements()

	var towns: Array = []
	for t in _env("HM_TOWNS", "").split(",", false):
		towns.append(int(t))
	for c in _env("HM_CELLS", "").split(",", false):
		for k in places.size():
			if int(places[k]["x"]) == int(c.get_slice(":", 0)) and int(places[k]["y"]) == int(c.get_slice(":", 1)):
				towns.append(k)
	if towns.is_empty() and _env("HM_AUTO", "") != "":
		towns = await _choose(ctx, places, int(_env("HM_AUTO", "6")))
	_check(ctx, "the towns to frame exist", not towns.is_empty() and towns.all(func(k): return k >= 0 and k < places.size()), str(towns))
	if towns.is_empty():
		return
	var zooms: Array = []
	for z in _env("HM_MAP_ZOOMS", "1,4").split(",", false):
		zooms.append(float(z))
	var host = app.viewport
	var ov = host.overlay
	var idx := PackedInt32Array(towns)
	var moved := {false: 0, true: 0}
	var rows_on: Array = []
	## 5. the towns drawn are forgotten when the switch flips: the shell's own signal, counted.
	var pressed := {}
	var reached := 0
	var width_off := {}
	var forgot := [0]
	bridge.urban_params_changed.connect(func() -> void: forgot[0] += 1)
	for on in [false, true]:
		var state := "on" if on else "off"
		bridge.param_set("civ.bank_sites", on and _env("HM_MUTATE", "") != "off")
		await ctx.wait(1.0)
		var rows: Array = bridge.world_gen.town_bank_census(idx, host.size.x, host.size.y)
		for r in rows:
			moved[on] += int(r.has("origin_m"))
			## A town the ruling reaches: laid round another point, or (a town already clear of the drawn river, like the
			## owner's on seed 532187) its river now as wide as the map draws it.
			if on:
				reached += int(r.has("origin_m") or absf(float(r.get("river_w", 0.0)) - float(width_off.get(int(r["i"]), 0.0))) > 1.0)
			else:
				width_off[int(r["i"])] = float(r.get("river_w", 0.0))
		if on:
			rows_on = rows
		for k in towns:
			var s: Dictionary = places[k]
			for z in zooms:
				host.reset_view()
				await ctx.wait(0.2)
				if z > 1.0:
					host._zoom_at(host.size * 0.5, z)
					host.move_view_to(float(s["x"]), float(s["y"]))
				## Deep-zoom tiles and the town layer arrive off the main thread.
				await ctx.wait(3.0 if z >= 16.0 else 1.5)
				_save(_crop(_marker_px(k), 170), k, state, "map_x%d" % int(z))
				if on:
					## 2. the marker: a press at the point the overlay draws it at finds this settlement. Tried at every
					## zoom framed and required at one: a small place's marker is decluttered away at the opening view and
					## a town's gives way to its plan at the deepest.
					var p: Vector2 = _marker_px(k)
					var local: Vector2 = ov.get_global_transform_with_canvas().affine_inverse() * p
					if ov._hit_test_settlement(local, ov._interior_rect(ov.displayed_rect()), ov.displayed_rect()) == k:
						pressed[k] = true
			if on:
				_check(ctx, "a press at #%d's drawn point finds it (at one of the zooms framed)" % k, pressed.has(k))
			host.reset_view()
			app.open_city_viewer(k)
			await ctx.wait(2.5)
			var cv = app.city_viewer_window
			for z in [1.0, 4.0]:
				cv._zoom = z
				cv._pan = Vector2.ZERO
				cv._canvas.queue_redraw()
				await ctx.wait(0.8)
				var r: Rect2 = DriveCtx.rect_of(cv._canvas)
				var img: Image = app.get_viewport().get_texture().get_image()
				var kk: float = float(img.get_width()) / app.get_viewport().get_visible_rect().size.x
				_save(img.get_region(Rect2i(Vector2i(r.position * kk), Vector2i(r.size * kk)).intersection(Rect2i(Vector2i.ZERO, img.get_size()))), k, state, "city_x%d" % int(z))
			cv._zoom = 1.0
			cv.hide()
			await ctx.wait(0.4)
	_check(ctx, "each write of civ.bank_sites made the shell forget the towns it drew (urban_params_changed)", forgot[0] >= 2, "%d" % forgot[0])
	_check(ctx, "with the switch off no framed town is laid off its cell", moved[false] == 0, "%d" % moved[false])
	_check(ctx, "with the switch on at least one framed town is laid round the drawn edge or has its river at the map's width (the positive control)", reached > 0, "%d of %d reached, %d moved" % [reached, towns.size(), moved[true]])
	for r in rows_on:
		var k := int(r["i"])
		_check(ctx, "#%d: the marker and the market are on one bank at x1 and x4" % k, int(r.get("side_x1", 1)) == 1 and int(r.get("side_x4", 1)) == 1)
		_check(ctx, "#%d: no building over a river the map draws" % k, bool(r.get("laid", false)) and int(r.get("over_b", 1)) == 0, "%s of %s" % [r.get("over_b", "-"), r.get("of_b", "-")])
		_check(ctx, "#%d: the town has its river and its market is out of it" % k, bool(r.get("river", false)) and float(r.get("market_river_m", 0.0)) >= float(r.get("river_w", 0.0)) * 0.5)
	## 6. the export copy (`shell/export_overlay.gd`, the path Ruling HG left unrun): a copy of the live overlay sized for a
	## 4096 x 3072 image asks the engine for its markers' bank points at ITS density, not the screen's.
	var eo = load("res://shell/export_overlay.gd").new()
	app.add_child(eo)
	var err: String = eo.setup(ov, 4096, 3072, {"settlements": true, "settlement_tier": "all"})
	_check(ctx, "the export copy builds", err == "", err)
	if err == "":
		var xo = eo.overlay
		xo._settle_pos(xo._settlements[towns[0]], xo.displayed_rect())
		var want: PackedVector2Array = bridge.settlement_bank_offsets(xo._bank_ppc, -1.0)
		var same: bool = want.size() == xo._settlements.size()
		var nonzero := 0
		for i in mini(want.size(), xo._settlements.size()):
			same = same and (xo._settlements[i] as Dictionary).get("bank", Vector2.ZERO) == want[i]
			nonzero += int(want[i].length() > 0.0)
		_check(ctx, "the export copy asks at its own density (%.2f px per cell against the screen's %.2f)" % [xo._bank_ppc, ov._bank_ppc], xo._bank_ppc > 0.0 and not is_equal_approx(xo._bank_ppc, ov._bank_ppc))
		_check(ctx, "the export copy's markers stand at the engine's bank points for that density", same and nonzero > 0, "%d of %d moved" % [nonzero, want.size()])
	eo.queue_free()
	## 7. a frozen town stays frozen (Rulings GZ and HA): the last framed town is frozen with the switch off; with the
	## switch back on it is served from its save, building for building, and the cheap check names what changed.
	var fk: int = towns[-1]
	bridge.param_set("civ.bank_sites", false)
	await ctx.wait(0.5)
	var froze: bool = bridge.town_freeze(fk)
	var before: Array = bridge.urban_layouts(PackedInt32Array([fk]))
	bridge.param_set("civ.bank_sites", true)
	await ctx.wait(0.5)
	var after: Array = bridge.urban_layouts(PackedInt32Array([fk]))
	var st: Dictionary = bridge.town_state(fk, true)
	_check(ctx, "#%d froze with the switch off" % fk, froze and before.size() == 1 and bool(before[0].get("frozen", false)))
	_check(ctx, "#%d is served unchanged with the switch on (pin, market and every building)" % fk, after.size() == 1 and bool(after[0].get("frozen", false))
		and after[0]["pin"] == before[0]["pin"] and after[0]["market"] == before[0]["market"] and str(after[0]["buildings"]) == str(before[0]["buildings"]))
	## A town the ruling moves is told by name; one it only widens the river of is told its ground changed.
	var why := str(st.get("reasons", ""))
	_check(ctx, "#%d's check says what changed (\"civ bank origin is now on\", or for a town not moved that the ground under it changed)" % fk,
		why.find("civ bank origin is now on") >= 0 or (not rows_on.is_empty() and not (rows_on[-1] as Dictionary).has("origin_m") and why.find("the ground or roads under this town changed") >= 0), why.left(200))
	## The positive control: the same town, not frozen, is laid out elsewhere with the switch on (check 1 counted it moved).
	ctx.log("RESULT %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
