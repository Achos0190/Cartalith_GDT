extends Node
## Probe for GROUNDED PLACE PROSE (`LLM_ASSISTANT_SCOPE.md` "Grounded place prose", LLM-11): the
## assistant panel's "Describe place" entry is written from the world's own data for the place and
## everything within 25 km, plus the Vault notes that bear on it, and a grounding check is shown after
## the reply. Runs the real panel in the real shell against the in-test loopback mock server
## (`_llm_mock_server.gd`); no model is involved.
##
##   desktop: godot --path . --resolution 1600x1000 _llmplace_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1280x800  _llmplace_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   _llmplace_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`**, from a scratch copy with its own `config/name` and a wiped user dir
## (`MISTAKES.md`), after `cargo build` so the DLL is newer than every `.rs` (state both times).
## Arguments read: `--force-touch` and `--shots <dir>`; anything else aborts with exit 2.
##
## What it protects (each line is an owner bar):
##   1. RECOUNT: for three seeds the facts equal an independent recount from the engine's own getters
##      (population, faction, trade lists, walls, elevation, the neighbours within 25 km and which twelve
##      are kept), the facts are deterministic, the lattice is bounded, and every figure is read-only.
##   2. RADIUS: a settlement dropped 24.95 km away is listed, one 25.12 km away is not.
##   3. CAPS: more than twelve neighbours lists twelve and NAMES the rest; the truncation is in the text.
##   4. VAULT: a linked note and the place's own note are sent; an unrelated note is not; a private-marked
##      note is withheld and counted; the order is deterministic; the notes are capped and the cut listed.
##   5. SEND: the "this will send" line itemises the sections, counts and note titles; the request body
##      carries two fences (world_data, vault_notes), the grounded system prompt, `reasoning_effort: none`
##      and no image; hosted providers still need consent and a refused gate opens no socket.
##   6. GROUNDING: after a reply with an invented number and a made-up town name the card lists both as
##      unverified and the real population and neighbour as facts used; nothing is blocked.
##   7. WRITE: the Vault is untouched until the Save click; the click appends one marked block.
##   8. LAYOUT: the new controls fit at desktop, tablet and phone.
## Result: `RESULT: PASS|FAIL (<n> failed)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

const SECRET := "sk-PROBE-SECRET-0123456789wxyz"
const ENV_VAR := "CART_LLM_PROBE_KEYVAR"
const VAULT_DIR := "user://llm_place_probe_vault"
const SEEDS := [9137, 71077345, 24601]
const WIDTH_KM := 160.0
const GRID_W := 400
const GRID_H := 300

var _fails := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _win
var _server: LlmMockServer
var _port := 0
var _vault := ""


func _ok(cond: bool, what: String) -> void:
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _until(cond: Callable, ms: int = 8000) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


func _c(name: String) -> Control:
	return _win.controls.get(name) as Control


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	var path := _shots.path_join("llm11_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _click(c: Control) -> void:
	var local := c.get_global_rect().get_center()
	var vp := c.get_viewport()
	var at: Vector2 = vp.get_final_transform() * local
	if vp is Window and vp != get_tree().root:
		at += Vector2((vp as Window).position)
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	Input.parse_input_event(mm)
	await _settle(2)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(3)


func _find_row(n: Node, text: String) -> Array:
	if n is PopupMenu:
		var p := n as PopupMenu
		for i in p.item_count:
			if p.get_item_text(i) == text:
				return [p, i]
	for c in n.get_children(true):
		var r := _find_row(c, text)
		if not r.is_empty():
			return r
	return []


func _all_text(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
		out.append((n as Button).tooltip_text)
	elif n is TextEdit:
		out.append((n as TextEdit).text)
	if n is Control:
		out.append((n as Control).tooltip_text)
	for ch in n.get_children(true):
		_all_text(ch, out)


func _window_text() -> String:
	var parts: Array = []
	_all_text(_win, parts)
	return "\n".join(parts)


func _read(path: String) -> String:
	var f := FileAccess.open(path, FileAccess.READ)
	return f.get_as_text() if f != null else ""


func _write(rel: String, text: String) -> void:
	var path := _vault.path_join(rel)
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _configure(over: Dictionary = {}) -> void:
	var s := {"enabled": true, "prose": true, "local_allowed": true, "preset_id": "ollama",
		"base_url": "http://127.0.0.1:%d/v1" % _port, "model": "mock-model",
		"key_source": "env", "env_var": ENV_VAR, "ctx_entity": false, "ctx_notes": true}
	s.merge(over, true)
	for k in s:
		DccSettings.set_llm_value(k, s[k])
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % _port)


func _reset_llm() -> void:
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()


func _type(text: String) -> void:
	var inp := _c("input") as TextEdit
	inp.text = text
	inp.text_changed.emit()
	await _settle(3)


func _last_body() -> Dictionary:
	var b = JSON.parse_string(String(_server.requests[_server.requests.size() - 1]["body"]))
	return b if b is Dictionary else {}


func _msg(body: Dictionary, role: String) -> String:
	for m in body.get("messages", []):
		if m["role"] == role:
			return String(m["content"])
	return ""


func _finish() -> void:
	OS.unset_environment(ENV_VAR)
	LlmWire.shutdown_all()
	if _server != null:
		_server.stop()
	print("RESULT: %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)


## Generates a world and runs the belief model, so religion data exists the way it does once the
## user has opened the Religion layer.
func _world(seed_value: int) -> void:
	_bridge.generate({"seed": seed_value, "width_km": WIDTH_KM, "grid_w": GRID_W, "grid_h": GRID_H, "archetype": "", "villages": true})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _settle(6)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	_bridge.civ_belief_run(60)


## The most populous settlement's index.
func _largest() -> int:
	var ss: Array = _bridge.settlements()
	var best := 0
	for i in ss.size():
		if int(ss[i].population) > int(ss[best].population):
			best = i
	return best


## The independent recount: the neighbours within 25 km by plain geometry over `settlements()`.
func _recount(idx: int) -> Dictionary:
	var ss: Array = _bridge.settlements()
	var ck := WIDTH_KM / float(GRID_W)
	var p: Dictionary = ss[idx]
	var within := []
	for j in ss.size():
		if j == idx:
			continue
		var s: Dictionary = ss[j]
		var d := _km(float(s.x) - float(p.x), float(s.y) - float(p.y), ck)
		if d <= 25.0:
			within.append({"j": j, "d": d, "pop": int(s.population), "tid": int(s.tid), "name": String(s.name)})
	within.sort_custom(func(a, b):
		if a["pop"] != b["pop"]:
			return a["pop"] > b["pop"]
		if absf(a["d"] - b["d"]) > 1e-12:
			return a["d"] < b["d"]
		return a["tid"] < b["tid"])
	var kept: Array = within.slice(0, 12)
	kept.sort_custom(func(a, b):
		if absf(a["d"] - b["d"]) > 1e-12:
			return a["d"] < b["d"]
		return a["tid"] < b["tid"])
	return {"within": within.size(), "kept": kept, "omitted": within.size() - kept.size()}


## Distance in km in float64 (a `Vector2` is float32 and would differ from Rust's f64 in the 7th digit).
func _km(dx: float, dy: float, ck: float) -> float:
	## On a map that wraps east-west the shorter way round the seam (the engine's "world" parameter).
	if bool(_bridge.param_get("world")):
		var gw := float(GRID_W)
		if dx > gw / 2.0:
			dx -= gw
		elif dx < -gw / 2.0:
			dx += gw
	return sqrt(dx * dx + dy * dy) * ck


## True when `(x, y)` is a land cell with no settlement within 10 cells of it: the engine returns an existing
## settlement instead of placing a new one when a drop lands too close to one (measured: a drop 3 cells from a
## settlement answered that settlement's index), so a probe that wants NEW settlements must choose clear ground.
func _free_land(x: int, y: int) -> bool:
	if x < 0 or y < 0 or x >= GRID_W or y >= GRID_H:
		return false
	if String(_bridge.sample_cell(x, y).get("water", "")) != "land":
		return false
	for s in _bridge.settlements():
		if absf(float(s.x) - x) < 10.0 and absf(float(s.y) - y) < 10.0:
			return false
	return true


func _facts(idx: int) -> Dictionary:
	var g: Dictionary = LlmPlaceFacts.gather(_bridge, _win.run.stream, idx)
	if not bool(g.get("ok", false)):
		return {"ok": false, "g": g}
	var b: Dictionary = _win.run.stream.place_bundle(JSON.stringify(g["raw"]), "[]", "", false)
	return {"ok": bool(b.get("ok", false)), "g": g, "b": b, "json": JSON.parse_string(String(b.get("facts_json", "{}")))}


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--force-touch":
			pass
		elif args[i] == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch and --shots <dir> only" % args[i])
			get_tree().quit(2)
			return
		i += 1
	get_tree().create_timer(1500.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	OS.unset_environment(ENV_VAR)
	_server = LlmMockServer.new()
	add_child(_server)
	_ok(_server.start(), "the mock server listens")
	_port = _server.port
	_reset_llm()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, viewport %s ==" % [_mode, get_viewport().get_visible_rect().size])
	_bridge = _app.bridge
	_win = _app.assistant_window
	_ok(_bridge.llm_api and _win != null, "the library has LlmBridge and the shell has the panel")
	if not _bridge.llm_api or _win == null:
		_finish()
		return
	var stream: Object = _win.run.stream
	_ok(stream != null and stream.has_method("place_bundle") and stream.has_method("grounding_check"), "the library has the place-facts functions")
	var lim: Dictionary = stream.place_limits()
	_ok(is_equal_approx(float(lim.get("radius_km", 0)), 25.0) and int(lim.get("max_neighbours", 0)) == 12 and int(lim.get("max_notes", 0)) == 6, "the limits are the owner's: 25 km, 12 neighbours, 6 notes")

	# ------------------------------------------------------------------ 1. recount, three seeds
	print("== 1. facts equal an independent recount (3 seeds) ==")
	var first_idx := 0
	for sd in SEEDS:
		await _world(sd)
		var idx := _largest()
		var ss: Array = _bridge.settlements()
		var p: Dictionary = ss[idx]
		var t0 := Time.get_ticks_msec()
		var f := _facts(idx)
		var ms := Time.get_ticks_msec() - t0
		_ok(bool(f["ok"]), "seed %d: facts were gathered for %s (%d ms, %d characters)" % [sd, p.name, ms, int(f["b"].get("chars", 0))])
		if not bool(f["ok"]):
			continue
		print("GATHER seed=%d place=%s settlements=%d gather_ms=%d chars=%d sections=%d" % [sd, p.name, ss.size(), ms, int(f["b"]["chars"]), (f["b"]["sections"] as Array).size()])
		var j: Dictionary = f["json"]
		var wd: Dictionary = j["world_data"]
		_ok(String(j["place"]) == String(p.name) and int(wd["place"]["population"]) == int(p.population) and int(wd["place"]["id"]) == int(p.tid), "seed %d: name, id and population equal the engine's (%s, %d)" % [sd, p.name, int(p.population)])
		_ok(String(wd["place"]["size_class"]) == String(p.kind) and bool(wd["place"]["capital"]) == bool(p.capital) and bool(wd["place"]["coastal"]) == bool(p.coastal), "seed %d: size class, capital and coastal equal the engine's" % sd)
		var fac := {}
		for fr in _bridge.get_factions():
			if int(fr.id) == int(p.faction):
				fac = fr
		_ok(not fac.is_empty() and String(wd["faction"]["name"]) == String(fac.name) and String(wd["faction"]["government"]) == String(fac.government), "seed %d: the faction and its government equal the roster's (%s)" % [sd, String(fac.get("name", "?"))])
		var bal: Array = _bridge.trade_balances()
		var ex := []
		for e in bal[idx].exports:
			ex.append(String(e))
		_ok(wd["economy"]["exports"] == ex, "seed %d: this settlement's exports equal the trade balance (%s)" % [sd, str(ex)])
		var wall := ""
		for r in _bridge.civ_military_summary().settlements:
			if int(r.index) == idx:
				wall = String(r.wall_spec)
		_ok(wall != "" and String(wd["defence"]["wall_spec"]) == wall, "seed %d: walls equal the military summary's (%s)" % [sd, wall])
		var cell: Dictionary = _bridge.sample_cell(int(p.x), int(p.y))
		_ok(absf(float(wd["terrain"]["elevation_m"]) - float(cell.elevation_m)) < 1e-6 and String(wd["terrain"]["biome"]) == String(cell.biome), "seed %d: elevation and biome equal sample_cell's (%.1f m, %s)" % [sd, float(cell.elevation_m), String(cell.biome)])
		if p.has("religion"):
			_ok(wd["religion"].has("plurality") and String(wd["religion"]["plurality"]) == String(p.religion), "seed %d: the faith equals the belief layer's (%s)" % [sd, String(p.religion)])
		var rc := _recount(idx)
		var nb: Dictionary = wd["neighbours"]
		_ok(int(nb["within_radius"]) == int(rc["within"]), "seed %d: %d settlements within 25 km, same as the recount" % [sd, int(rc["within"])])
		var listed: Array = nb["listed"]
		var names_f := []
		for n in listed:
			names_f.append(String(n["name"]))
		var names_r := []
		for n in rc["kept"]:
			names_r.append(String(n["name"]))
		_ok(names_f == names_r, "seed %d: the %d listed neighbours are the recount's largest, nearest first" % [sd, names_r.size()])
		var dist_ok := true
		for k in listed.size():
			if absf(float(listed[k]["distance_km"]) - float(rc["kept"][k]["d"])) > 1e-9:
				dist_ok = false
		_ok(dist_ok, "seed %d: every listed distance equals the recount's" % sd)
		_ok(int(f["b"]["neighbours_listed"]) <= 12 and (rc["omitted"] == 0 or String(f["b"]["text"]).contains("are not listed")), "seed %d: at most 12 neighbours; omitted ones (%d) are named in the text" % [sd, int(rc["omitted"])])
		# Deterministic: a second gather is byte-identical.
		var f2 := _facts(idx)
		_ok(String(f2["b"]["text"]) == String(f["b"]["text"]) and String(f2["b"]["facts_json"]) == String(f["b"]["facts_json"]), "seed %d: a second gather is byte-identical (deterministic)" % sd)
		# Absent means absent: the landmark pass has not run, so the text says so.
		_ok(String(f["b"]["text"]).contains("landmarks: not available"), "seed %d: landmarks (no pass ran) read 'not available', not 'none'" % sd)
		var lat: PackedInt32Array = stream.place_sample_points(int(p.x), int(p.y), WIDTH_KM / GRID_W, GRID_W, GRID_H, bool(_bridge.param_get("world")))
		_ok(lat.size() % 2 == 0 and lat.size() / 2 <= 600 and lat.size() > 100, "seed %d: the lattice has %d points (<= 600)" % [sd, lat.size() / 2])
		if sd == SEEDS[0]:
			first_idx = idx
	# From here the first seed is the working world.
	await _world(SEEDS[0])
	first_idx = _largest()

	# ------------------------------------------------------------------ 2. radius, in the engine
	print("== 2. the 25 km radius, in the engine ==")
	var ss0: Array = _bridge.settlements()
	var ck := WIDTH_KM / float(GRID_W)
	## Offsets in cells at 0.4 km per cell: INSIDE has 62.25 < d <= 62.5 cells (24.9 to 25.0 km), OUTSIDE has
	## 62.75 < d < 63.0 cells (25.1 to 25.2 km). Found by enumeration, not typed in.
	var inside_offs: Array[Vector2i] = []
	var outside_offs: Array[Vector2i] = []
	for ox in range(0, 64):
		for oy in range(0, 64):
			var dc := sqrt(float(ox * ox + oy * oy))
			if dc > 62.25 and dc <= 62.5:
				inside_offs.append(Vector2i(ox, oy))
			elif dc > 62.75 and dc < 63.0:
				outside_offs.append(Vector2i(ox, oy))
	var order: Array = range(ss0.size())
	order.sort_custom(func(a, b): return int(ss0[a].population) > int(ss0[b].population))
	var pair := {}
	var pp: Dictionary = ss0[first_idx]
	for cand in order.slice(0, 25):
		if not pair.is_empty():
			break
		var q: Dictionary = ss0[cand]
		for io in inside_offs:
			for oo in outside_offs:
				for sx: int in [1, -1]:
					for sy: int in [1, -1]:
						if not pair.is_empty():
							break
						var ax: int = int(q.x) + io.x * sx
						var ay: int = int(q.y) + io.y * sy
						var bx: int = int(q.x) - oo.x * sx
						var by: int = int(q.y) - oo.y * sy
						if ax < 0 or ay < 0 or bx < 0 or by < 0 or ax >= GRID_W or bx >= GRID_W or ay >= GRID_H or by >= GRID_H:
							continue
						if _free_land(ax, ay) and _free_land(bx, by):
							pair = {"a": Vector2i(ax, ay), "b": Vector2i(bx, by), "place": cand}
	if not pair.is_empty():
		first_idx = int(pair["place"])
		pp = ss0[first_idx]
	_ok(not pair.is_empty(), "two land cells at 24.95 km and 25.12 km exist around %s" % String(pp.name))
	if not pair.is_empty():
		var n_before: int = _bridge.settlements().size()
		var ia: int = _bridge.civ_drop_settlement(float(pair["a"].x), float(pair["a"].y), "hamlet", int(pp.faction), "Probe Inside", false)
		var ib: int = _bridge.civ_drop_settlement(float(pair["b"].x), float(pair["b"].y), "hamlet", int(pp.faction), "Probe Outside", false)
		_ok(ia == n_before and ib == n_before + 1 and _bridge.settlements().size() == n_before + 2, "the engine placed two NEW settlements (indices %d, %d of %d)" % [ia, ib, _bridge.settlements().size()])
		var sd2: Array = _bridge.settlements()
		var pi := first_idx
		for k in sd2.size():
			if String(sd2[k].name) == String(pp.name):
				pi = k
		var da := _km(float(sd2[ia].x) - float(sd2[pi].x), float(sd2[ia].y) - float(sd2[pi].y), ck)
		var db := _km(float(sd2[ib].x) - float(sd2[pi].x), float(sd2[ib].y) - float(sd2[pi].y), ck)
		_ok(da > 24.9 and da <= 25.0 and db > 25.1 and db < 25.2, "the distances are %.3f km and %.3f km" % [da, db])
		var f := _facts(pi)
		var t: String = String(f["b"]["text"])
		_ok(t.contains("- Probe Inside ("), "the settlement %.2f km away IS in the facts" % da)
		_ok(not t.contains("Probe Outside"), "the settlement %.2f km away is NOT in the facts" % db)
		var ins: String = t.substr(t.find("- Probe Inside ("), 120)
		_ok(ins.contains("%.1f km" % da), "...with its distance (%.1f km): '%s'" % [da, ins.left(90)])
		first_idx = pi

		# ---------------------------------------------------------------- 3. caps
		print("== 3. caps ==")
		var dropped := 0
		var rx := int(sd2[pi].x)
		var ry := int(sd2[pi].y)
		## A ring of candidate cells 12 to 50 cells out (4.8 to 20 km), clear ground only.
		for k in 400:
			if dropped >= 14:
				break
			var ang := float(k) * 2.399963
			var rad := 12.0 + float(k % 38)
			var ox := rx + int(round(cos(ang) * rad))
			var oy := ry + int(round(sin(ang) * rad))
			if not _free_land(ox, oy):
				continue
			var nb4: int = _bridge.settlements().size()
			if _bridge.civ_drop_settlement(float(ox), float(oy), "hamlet", int(pp.faction), "Probe Crowd %02d" % dropped, false) == nb4:
				dropped += 1
		_ok(dropped == 14, "dropped %d new hamlets near the place" % dropped)
		var rc2 := _recount(pi)
		var f3 := _facts(pi)
		var t3: String = String(f3["b"]["text"])
		print("CAP within=%d dropped=%d listed=%d" % [int(rc2["within"]), dropped, int(f3["b"]["neighbours_listed"])])
		_ok(int(rc2["within"]) > 12, "now more than 12 settlements lie within 25 km (%d)" % int(rc2["within"]))
		_ok(int(f3["b"]["neighbours_listed"]) == 12, "exactly 12 are listed (the cap)")
		var om := int(rc2["within"]) - 12
		_ok(t3.contains("%d of %d within 25 km are not listed" % [om, int(rc2["within"])]), "the text says %d of %d are not listed" % [om, int(rc2["within"])])
		var trunc_has := false
		for s in f3["b"]["truncated"]:
			if String(s).begins_with("neighbours:"):
				trunc_has = true
		_ok(trunc_has, "...and the truncation list carries it")
		# The omitted names in the text are exactly the recount's omitted ones (the smallest, nearest first).
		var all_within: Array = []
		var ssx: Array = _bridge.settlements()
		for j in ssx.size():
			if j == pi:
				continue
			var dd := _km(float(ssx[j].x) - float(ssx[pi].x), float(ssx[j].y) - float(ssx[pi].y), ck)
			if dd <= 25.0:
				all_within.append(String(ssx[j].name))
		var kept_names := []
		for n in rc2["kept"]:
			kept_names.append(String(n["name"]))
		var named_ok := true
		for nm in all_within:
			if not kept_names.has(nm) and not t3.contains(String(nm)):
				named_ok = false
		_ok(named_ok, "every omitted neighbour is named in the text")
	else:
		_fails += 3

	# ------------------------------------------------------------------ 4. Vault selection
	print("== 4. Vault notes ==")
	var place: Dictionary = _bridge.settlements()[first_idx]
	var pname := String(place.name)
	var pfac := ""
	for fr in _bridge.get_factions():
		if int(fr.id) == int(place.faction):
			pfac = String(fr.name)
	var rel_label := ""
	for rr in _bridge.civ_religion_vocabulary():
		if p_religion(place) == String(rr.key):
			rel_label = String(rr.label)
	var nb_name := ""
	var nbs: Array = _recount(first_idx)["kept"]
	for n in nbs:
		if not String(n["name"]).begins_with("Probe"):
			nb_name = String(n["name"])
			break
	_vault = ProjectSettings.globalize_path(VAULT_DIR)
	DirAccess.make_dir_recursive_absolute(_vault)
	var place_rel := "Places/%s.md" % pname
	_write(place_rel, "# %s\nA salt-marsh town. The harbourmaster is [[Mara Venn]], who answers to [[%s]]. Its founding story is *my own canon*: it was founded by refugees from the drowned coast, in the year 14.\n#harbour\n" % [pname, pfac])
	_write("People/Mara Venn.md", "# Mara Venn\nHarbourmaster of %s. Her brother Teodor sailed east and never returned.\n" % pname)
	_write("Factions/%s.md" % pfac, "# %s\nThe ruling line is the House of Ash.\n" % pfac)
	if rel_label != "":
		_write("Lore/%s.md" % rel_label, "# %s\nRites are held at dawn on the quay.\n" % rel_label)
	if nb_name != "":
		_write("Places/%s.md" % nb_name, "# %s\nA mill town upriver that feeds %s.\n" % [nb_name, pname])
	_write("Places/Backlink note.md", "# Backlink note\nSee [[%s]] for the harbour. The quay bell rings at six.\n" % pname)
	_write("Places/Secret plot.md", "---\nprivate: true\n---\n# Secret plot\nIn [[%s]] the harbourmaster is a spy. SECRETPLOTWORD.\n" % pname)
	_write("Random/Grocery list.md", "# Grocery list\neggs, flour, UNRELATEDWORD\n")
	var vinfo: Dictionary = _bridge.vault_connect(_vault, "LLM place probe")
	_ok(bool(vinfo.get("ok", false)) and bool(_bridge.vault_info().get("bound", false)), "a probe Vault is connected: %s" % str(vinfo.get("error", "")))
	var att: Dictionary = _bridge.vault_attach("settlement", int(place.tid), pname, place_rel, "")
	_ok(bool(att.get("ok", false)), "the place's note is attached in the Vault: %s" % str(att.get("error", "")))
	_bridge.vault_refresh_backlinks()
	var vg: Dictionary = LlmPlaceFacts.gather_vault(_bridge, stream, LlmPlaceFacts.gather(_bridge, stream, first_idx), {})
	_ok(bool(vg.get("ok", false)), "the Vault half gathered: %s" % str(vg.get("notes", [])))
	var rels := []
	for r in vg.get("read", []):
		rels.append(String(r["rel"]))
	print("VAULT plan: ", rels)
	_ok(rels.has(place_rel), "the place's own (attached) note is read, first: %s" % str(rels.slice(0, 1)))
	_ok(not rels.is_empty() and rels[0] == place_rel, "...and ranked first")
	_ok(rels.has("People/Mara Venn.md"), "a note linked from the place's note ([[Mara Venn]]) is read")
	_ok(rels.has("Places/Backlink note.md"), "a note that links TO the place (a backlink) is read")
	_ok(rels.has("Factions/%s.md" % pfac), "a note titled like the place's faction is read")
	if nb_name != "":
		_ok(rels.has("Places/%s.md" % nb_name), "a note titled like a listed neighbour is read")
	_ok(not rels.has("Random/Grocery list.md"), "an unrelated note is NOT read")
	var rels2 := []
	for r in LlmPlaceFacts.gather_vault(_bridge, stream, LlmPlaceFacts.gather(_bridge, stream, first_idx), {}).get("read", []):
		rels2.append(String(r["rel"]))
	_ok(rels == rels2, "the order is deterministic across two gathers")
	var g1 := LlmPlaceFacts.gather(_bridge, stream, first_idx)
	var read_all: Array = vg["read"]
	var b1: Dictionary = stream.place_bundle(JSON.stringify(g1["raw"]), JSON.stringify(read_all), String(vg["own_note_text"]), true)
	var bj: Dictionary = JSON.parse_string(String(b1["bundle"]))
	var vtext := String(bj.get("vault_text", ""))
	_ok(vtext.contains("Teodor sailed east") and vtext.contains("The quay bell rings at six"), "the linked notes' text is in the VAULT NOTES block")
	_ok(not vtext.contains("UNRELATEDWORD"), "...and the unrelated note's text is not")
	_ok(not vtext.contains("SECRETPLOTWORD") and not String(bj["corpus"]).contains("SECRETPLOTWORD"), "a private-marked note ([private: true]) is NOT sent, and not in the grounding corpus")
	_ok(int(b1["vault_withheld"]) == 1, "...and is counted as withheld (%d)" % int(b1["vault_withheld"]))
	var no_notes: Dictionary = stream.place_bundle(JSON.stringify(g1["raw"]), JSON.stringify(read_all), String(vg["own_note_text"]), false)
	var nj: Dictionary = JSON.parse_string(String(no_notes["bundle"]))
	_ok(String(nj.get("vault_text", "x")) == "" and (nj["vault_notes"] as Array).is_empty(), "with the Vault setting off no note text is sent")
	_ok(String(nj["corpus"]).contains("founded by refugees"), "...but the place's own note still grounds the check (a name the user wrote counts)")

	# ------------------------------------------------------------------ 5. the panel: send, fences, summary
	print("== 5. the panel ==")
	_app.right_dock_ctrl.on_settlement_selected(_bridge.settlements()[first_idx], first_idx)
	await _settle(4)
	_configure({"ctx_entity": false, "ctx_notes": true})
	var row := _find_row(_app, "Assistant panel…")
	_ok(not row.is_empty(), "Story ▸ NOTES carries 'Assistant panel…'")
	if row.is_empty():
		_finish()
		return
	var pm: PopupMenu = row[0]
	pm.about_to_popup.emit()
	pm.id_pressed.emit(pm.get_item_id(row[1]))
	await _settle(10)
	var picked := _c("note") as OptionButton
	var idx_v := 0
	for k in picked.item_count:
		if picked.get_item_text(k) == place_rel:
			idx_v = k
	_ok(idx_v > 0, "the picker lists the place's note")
	picked.select(idx_v)
	picked.item_selected.emit(idx_v)
	await _settle(3)
	var note_before := _read(_vault.path_join(place_rel))
	OS.set_environment(ENV_VAR, SECRET)
	await _type("Write the entry for this settlement for the world Vault: history, daily life, trade, faith, neighbours.")
	await _settle(4)
	var sent_t := (_c("sent") as Label).text
	print("SENT: ", sent_t)
	_ok(sent_t.begins_with("This will send: your request ("), "the sent line starts with the request")
	_ok(sent_t.contains("the generated world data for %s (" % pname), "...names the place's generated data")
	for sec in ["The place (", "Settlements within 25 km (", "The land within 25 km ("]:
		_ok(sent_t.contains(sec), "...lists the section '%s...' with its line count" % sec)
	_ok(sent_t.contains("settlements within 25 km listed"), "...and the neighbour counts")
	_ok(sent_t.contains("Vault note") and sent_t.contains("\"%s\"" % pname) and sent_t.contains("\"Mara Venn\"") and sent_t.contains("\"Backlink note\""), "...and the Vault note titles that will be sent")
	_ok(not sent_t.contains("Grocery") and not sent_t.contains("Secret plot"), "...and not the unrelated or the private note")
	_ok(sent_t.contains("withheld by a private mark"), "...and the withheld count")
	_ok(not (_c("send") as Button).disabled, "Send is enabled with the 'current entity' switch OFF (the place is the subject)")
	var facts_status := (_c("facts_status") as Label).text
	_ok(facts_status.begins_with("Place facts: ") and facts_status.contains("sections"), "the facts line: '%s'" % facts_status.left(90))
	await _shot("1_ready")

	# A reply containing real facts, an invented number and a made-up town.
	var real_pop := str(int(place.population))
	var reply := "%s is a city of %s souls. It stands %s. *Locals swear the tide-bell rings by itself.* To the east lies Korvash, a market town of 5300 people, and %s lies upriver." % [pname, real_pop, "on the coast" if bool(place.coastal) else "inland", nb_name]
	var toks := []
	for w in reply.split(" "):
		toks.append(String(w) + " ")
	_server.scenario = "stream"
	_server.token_delay_ms = 5
	_server.tokens = toks
	var n0 := _server.requests.size()
	var send_btn := _c("send") as Button
	print("SEND button rect %s disabled %s window %s visible %s" % [send_btn.get_global_rect(), send_btn.disabled, _win.size, send_btn.is_visible_in_tree()])
	await _click(send_btn)
	await _until(func(): return _server.requests.size() > n0 and not _win.run.busy(), 15000)
	await _settle(8)
	if _server.requests.size() <= n0:
		print("NO REQUEST: window text tail: %s" % _window_text().right(700))
		_ok(false, "the Send click produced a request")
		_finish()
		return
	var body := _last_body()
	var sys := _msg(body, "system")
	var usr := _msg(body, "user")
	_ok(sys.contains("Use ONLY these two blocks") and sys.contains("Discrepancy:") and sys.contains("single asterisks"), "the request carries the grounded system prompt")
	_ok(usr.count("<context kind=\"world_data\"") == 1 and usr.count("<context kind=\"vault_notes\"") == 1 and usr.count("</context>") == 2, "...two fences: world_data and vault_notes")
	_ok(usr.contains("population: %s" % real_pop) and usr.contains("[Settlements within 25 km]"), "...the world data holds the engine's population and the neighbours section")
	_ok(usr.contains("Teodor sailed east") and not usr.contains("UNRELATEDWORD") and not usr.contains("SECRETPLOTWORD"), "...the Vault notes hold the linked text, never the unrelated or the private")
	_ok(usr.contains("Write the entry for this settlement"), "...and the user's own request")
	_ok(String(body.get("reasoning_effort", "")) == "none" and int(body.get("max_tokens", 0)) == 2048, "...with reasoning_effort none and max_tokens 2048: %s / %s" % [str(body.get("reasoning_effort", "absent")), str(body.get("max_tokens", "absent"))])
	var raw_body := String(_server.requests[_server.requests.size() - 1]["body"])
	_ok(not raw_body.contains("image_url") and not raw_body.contains("data:image") and not raw_body.contains("base64"), "...and no image of any kind (Ruling DI P5)")
	_ok(not usr.contains(SECRET) and not raw_body.contains(SECRET), "...and the key is not in the body")
	_ok(usr.length() < 20000, "...the whole user message is %d characters" % usr.length())
	print("BODY user chars=%d" % usr.length())
	# Grounding card.
	_ok(_c("grounding") != null, "a Grounding check card appeared after the reply")
	var gt := (_c("grounding") as Label).text if _c("grounding") != null else ""
	print("GROUNDING: ", gt)
	_ok(gt.contains("Unverified claims (") and gt.contains("Korvash (name)") and gt.contains("5300 (number)"), "...it lists the made-up town and the invented number as unverified")
	var gl: Dictionary = _win.last_grounding
	_ok((gl["numbers_matched"] as PackedStringArray).has(real_pop), "...and the real population %s as matched" % real_pop)
	if nb_name != "":
		_ok((gl["names_matched"] as PackedStringArray).has(pname.split(" ")[0]) and (gl["names_matched"] as PackedStringArray).has(nb_name.split(" ")[0]), "...and the place and a real neighbour among the facts used")
	_ok(not (gl["names_unverified"] as PackedStringArray).has("Locals"), "...and the italic span is reported as flavour, not flagged: %s" % str(gl["flavour"]))
	_ok(not (_c("save") as Button).disabled, "nothing was blocked: Save is enabled")
	_ok(_read(_vault.path_join(place_rel)) == note_before, "the Vault note is UNCHANGED after the reply and the check (no write without a click)")
	await _shot("2_reply_and_check")
	# View facts: a local card, no request.
	var req_n := _server.requests.size()
	await _click(_c("facts_view"))
	await _settle(4)
	_ok(_server.requests.size() == req_n and _window_text().contains("WORLD DATA (generated, authoritative for figures)") and _window_text().contains("VAULT NOTES (your writing"), "View facts shows the two sections locally and sends nothing")
	await _shot("3_view_facts")
	# Save: the click appends one marked block and keeps the user's text.
	await _click(_c("save"))
	await _settle(6)
	var after := _read(_vault.path_join(place_rel))
	_ok(after.begins_with(note_before) and after.count("CARTALITH:MODEL-GENERATED-END") == 1, "after the Save click the note keeps its text as a prefix and gains one marked block")
	_ok(after.contains("Korvash"), "...the reply is saved as written (the check never edits it)")
	# The next entry does NOT treat the first model block as canon.
	_server.tokens = ["Ok. "]
	n0 = _server.requests.size()
	await _type("Write it again.")
	await _click(_c("send"))
	await _until(func(): return _server.requests.size() > n0 and not _win.run.busy(), 10000)
	var u2 := _msg(_last_body(), "user")
	_ok(not u2.contains("Korvash") and not u2.contains("MODEL-GENERATED"), "a model-generated block already in the note is NOT fed back as canon")
	_ok(u2.contains("founded by refugees"), "...while the user's own words in the note are")

	# ---- consent and the key rule
	print("== consent ==")
	_configure({"preset_id": "openai", "base_url": "https://api.openai.com/v1", "model": "gpt-x", "remote_allowed": true})
	_win._refresh()
	await _settle(3)
	var acc0 := _server.accepted
	var att0 := LlmWire.connect_attempts
	var gate := DccSettings.llm_gate_dict()
	var res: Dictionary = _win.run.stream.prepare(gate, "prose", PackedStringArray(), _win._args("Write it."))
	_ok(not bool(res.get("ok", true)) and String(res.get("code", "")) == "consent_needed", "a hosted provider without per-origin consent is refused at the gate with consent_needed (%s): the facts need the same consent" % str(res.get("code", "?")))
	_ok(LlmWire.connect_attempts == att0 and _server.accepted == acc0, "...and opened no connection")
	_configure({})

	# ---- 6. caps on notes (8 more linked notes, big)
	print("== 6. note caps ==")
	for k in 8:
		_write("Places/Extra %d.md" % k, "# Extra %d\nSee [[%s]]. %s\n" % [k, pname, "lorem ".repeat(400)])
	_bridge.vault_refresh_backlinks()
	var g2 := LlmPlaceFacts.gather(_bridge, stream, first_idx)
	var vg2: Dictionary = LlmPlaceFacts.gather_vault(_bridge, stream, g2, {})
	_ok((vg2["read"] as Array).size() <= int(lim["max_reads"]), "at most %d notes are read (%d)" % [int(lim["max_reads"]), (vg2["read"] as Array).size()])
	var b2: Dictionary = stream.place_bundle(JSON.stringify(g2["raw"]), JSON.stringify(vg2["read"]), String(vg2["own_note_text"]), true)
	var bj2: Dictionary = JSON.parse_string(String(b2["bundle"]))
	var sent_notes: Array = bj2["vault_notes"]
	var total := 0
	for n in sent_notes:
		total += int(n["chars"])
		_ok(int(n["chars"]) <= int(lim["max_note_chars"]), "note '%s' is %d characters (cap %d)" % [String(n["title"]), int(n["chars"]), int(lim["max_note_chars"])])
	_ok(sent_notes.size() <= int(lim["max_notes"]) and total <= 6000, "at most 6 notes and 6000 characters are sent (%d notes, %d characters)" % [sent_notes.size(), total])
	var cut_listed := false
	for s in bj2["truncated"]:
		if String(s).contains("Vault note") and (String(s).contains("characters sent") or String(s).contains("not sent")):
			cut_listed = true
	_ok(cut_listed, "...and the cuts are listed: %s" % str(bj2["truncated"]).left(160))

	# ---- 7. layout
	print("== 7. layout ==")
	_win.open()
	await _settle(8)
	var floor_px := 0.0
	if _mode == "phone":
		floor_px = float(DccTheme.PHONE_TAP_MIN)
	elif _mode == "tablet":
		floor_px = float(DccTheme.role_px("btn_min_h"))
	var win_rect := Rect2(Vector2(_win.position), Vector2(_win.size))
	var vp := get_viewport().get_visible_rect().size
	_ok(win_rect.position.x >= -0.5 and win_rect.position.y >= -0.5 and win_rect.end.x <= vp.x + 0.5 and win_rect.end.y <= vp.y + 0.5, "the window %s is inside the viewport %s" % [win_rect, vp])
	for name in ["facts_view", "facts_refresh", "send", "save", "note"]:
		var ctl := _c(name)
		if ctl == null or not ctl.is_visible_in_tree():
			continue
		if floor_px > 0.0:
			_ok(ctl.size.y + 0.5 >= floor_px, "'%s' is at least %d px tall: %.0f" % [name, int(floor_px), ctl.size.y])
		var rr := ctl.get_global_rect()
		_ok(rr.position.x >= -1.0 and rr.end.x <= float(_win.size.x) + 1.0 or _mode == "desktop" and rr.end.x <= win_rect.end.x + 1.0, "'%s' sits inside the window width" % name)
	await _shot("4_layout")
	_ok(LlmWire.live_threads == 0, "no worker thread is live at the end")
	_finish()


## The settlement dictionary's religion key, or "".
func p_religion(s: Dictionary) -> String:
	return String(s.get("religion", ""))
