extends Node
## **LLM-7 in the real shell: edit actions over a region the user paints or boxes**
## (`LLM_ASSISTANT_SCOPE.md` s.5c; Rulings DD Q10, DI, DR), against the in-test loopback server
## (`_llm_mock_server.gd`, scenarios `tools` and `tools_anthropic`) on BOTH wires. The real-model checks
## are `_llmedit_live_probe.gd`.
##
##   desktop: godot --path . --resolution 1600x1000 res://_llmedit_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1280x800  res://_llmedit_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   res://_llmedit_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`**, from a worktree with its own `config/name` and a wiped user dir
## (`MISTAKES.md`). Reads `--force-touch`, `--shots <dir>` and `--corpus <path>`; anything else exits 2.
##
## What it protects (every check names it):
##   0. SWITCH: Edit is off by default, its context source is off by default, the switch is available in
##      this build, and is unavailable with a reason when the grouped undo is missing.
##   1. GATES: Edit task Send is disabled with a distinct reason for each of: master off, Edit off,
##      region source off, no region; a hosted endpoint is refused in Rust (`edit_local_only`) with ZERO
##      connections; Commands being off does not matter (Edit is its own switch).
##   2. REGION INPUT: strokes painted through the tool's own handlers (and one real mouse drag on
##      desktop/tablet) become dabs; the engine's cell count equals an independent recount; Undo stroke,
##      Clear and Use the box work; Done brings the Assistant back on the Edit task with the prompt kept;
##      a new world drops the region; the phone's Paint button is disabled with its reason.
##   3. OFFER: the first request offers exactly `propose_edit_actions` (no schema, setting, query, view
##      or knowledge tool), its region_id enum is the one offered id, the system prompt carries the region
##      block (numbers) and no image part exists in the body; both wires.
##   4. CARD: nothing changes before the click (digest, undo stack, generate count); the card shows each
##      edit, the preview figures and two pictures; the preview's changed-cell count equals what Apply then
##      changes; ONE undo group; ONE Edit > Undo restores the field digest on 3 seeds; Redo re-applies; the
##      card follows "(undone)".
##   5. REFUSALS: an over-cap amount, another region's id, an unknown operation, a free-form field and a
##      wrong type each show their rule, are retried ONCE, have no Apply and change nothing.
##   6. ENGINE SECOND WALL: the engine call itself refuses an over-cap amount, four operations and a
##      non-number, with nothing written and no undo step.
##   7. GENERATING: while a world generates, Apply is disabled with the reason.
##   8. CORPUS: the edit hostile rows (X01..) through the real panel on both wires: the tool set is fixed,
##      nothing is written, a row expecting a rule shows it.
##   9. LAYOUT: card and Send stay inside the window; touch layouts meet the 44 px floor.
## Result: `RESULT: PASS|FAIL (<n> failed) checks=<n>`. Exit 0 pass, 1 fail, 2 bad argument/watchdog.

const ENV_VAR := "CART_LLM_PROBE_KEYVAR"
const SECRET := "sk-PROBE-SECRET-edit-0123456789"
const SEEDS := [9137, 24601, 71077345]
const EDIT_TOOL := "propose_edit_actions"
const REGION_SCRIPT := preload("res://shell/llm_region.gd")
const _EDIT_SCRIPT := preload("res://shell/llm_edit.gd")

var _fails := 0
var _checks := 0
## `--failfast`: end the run (RESULT: FAIL) at the first failed check. Used by the mutation runs, where the
## first red check is the answer and the rest of a four-minute run is wasted.
var _failfast := false
var _mode := "desktop"
var _shots := ""
var _corpus_path := "../crates/cartalith-llm/tests/fixtures/hostile_corpus_edit_2026-10-26.json"
var _app: Node
var _bridge
var _win
var _server: LlmMockServer
var _port := 0
var _gen_started := 0


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1
		if _failfast:
			_finish()


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _until(cond: Callable, ms: int = 10000) -> bool:
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
	await _settle(4)
	var path := _shots.path_join("llmedit_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _click_until(c: Control, done: Callable, tries: int = 3) -> void:
	for i in tries:
		await _click(c)
		await _settle(8)
		if done.call():
			return
		print("  (click %d of %d on '%s' did not register; window focused=%s)" % [i + 1, tries, c.name, str(get_window().has_focus())])


## A REAL mouse press and release at the centre of `c`.
func _click(c: Control) -> void:
	var p := c.get_parent()
	while p != null:
		if p is ScrollContainer:
			(p as ScrollContainer).ensure_control_visible(c)
			await _settle(2)
			break
		p = p.get_parent()
	var at := _global_of(c.get_global_rect().get_center(), c.get_viewport())
	await _mouse(at, null)
	for down in [true, false]:
		await _mouse(at, down)
	await _settle(3)


func _global_of(local: Vector2, vp: Viewport) -> Vector2:
	var at: Vector2 = vp.get_final_transform() * local
	if vp is Window and vp != get_tree().root:
		at += Vector2((vp as Window).position)
	return at


## One mouse event at `at`: a motion when `down` is null, else a left press or release.
func _mouse(at: Vector2, down, held: bool = false) -> void:
	if down == null:
		var mm := InputEventMouseMotion.new()
		mm.position = at
		mm.global_position = at
		## A drag is a motion WITH the left button held; without the mask the map reads it as a hover.
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT if held else 0
		Input.parse_input_event(mm)
	else:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = bool(down)
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
	await _settle(2)


func _all_text(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	if n is Control:
		out.append((n as Control).tooltip_text)
	for ch in n.get_children(true):
		_all_text(ch, out)


func _text_of(n: Node) -> String:
	var parts: Array = []
	_all_text(n, parts)
	return "\n".join(parts)


func _reset_llm() -> void:
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()


func _configure(over: Dictionary = {}) -> void:
	var s := {"enabled": true, "commands": false, "prose": false, "edit": true, "ctx_region": true, "local_allowed": true,
		"preset_id": "ollama", "base_url": "http://127.0.0.1:%d/v1" % _port, "model": "mock-model",
		"key_source": "env", "env_var": ENV_VAR, "ctx_params": false}
	s.merge(over, true)
	for k in s:
		DccSettings.set_llm_value(k, s[k])
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % _port)


func _type(text: String) -> void:
	var inp := _c("input") as TextEdit
	inp.text = text
	inp.text_changed.emit()
	await _settle(3)


func _digest() -> Dictionary:
	return (_bridge.edit_state_digest() as Dictionary).duplicate()


func _body(i: int) -> Dictionary:
	if i < 0 or i >= _server.requests.size():
		return {}
	var b = JSON.parse_string(String(_server.requests[i]["body"]))
	return b if b is Dictionary else {}


func _tool_names(b: Dictionary, anthropic: bool) -> Array:
	var out: Array = []
	for t in b.get("tools", []):
		out.append(String(t["name"]) if anthropic else String(t["function"]["name"]))
	return out


func _edit_tool(b: Dictionary, anthropic: bool) -> Dictionary:
	for t in b.get("tools", []):
		var name := String(t["name"]) if anthropic else String(t["function"]["name"])
		if name == EDIT_TOOL:
			return {"description": String(t["description"]) if anthropic else String(t["function"]["description"]),
				"schema": t["input_schema"] if anthropic else t["function"]["parameters"]}
	return {}


func _system_of(b: Dictionary, anthropic: bool) -> String:
	if anthropic:
		return String(b.get("system", ""))
	for m in b.get("messages", []):
		if String(m.get("role", "")) == "system":
			return String(m.get("content", ""))
	return ""


func _ask(scenario: String, script: Array, request: String) -> int:
	_server.scenario = scenario
	_server.tool_script = script
	_server.tool_served = 0
	var n0: int = _server.requests.size()
	_win._pick_task("edit")
	await _type(request)
	var send := _c("send") as Button
	if send.disabled:
		print("  (Send disabled: %s)" % (_c("reason") as Label).text)
	await _click_until(send, func(): return _server.requests.size() > n0 or _win.run.busy())
	await _until(func(): return not _win.run.busy() and _server.requests.size() > n0, 15000)
	await _until(func(): return not _win.run.busy(), 15000)
	await _settle(6)
	return _server.requests.size() - n0


func _edit_card() -> Dictionary:
	for i in range(_win.cards.size() - 1, -1, -1):
		if String(_win.cards[i].get("kind", "params")) == "edit":
			return _win.cards[i]
	return {}


func _generate(seed_: int) -> void:
	_bridge.generate({"seed": seed_, "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": false})
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()


## A region far from a coast and from the highest ground, so a raise has headroom on every seed.
func _paint_region() -> void:
	REGION_SCRIPT.clear(_app)
	REGION_SCRIPT.brush = 16.0
	## Two strokes: a horizontal bar and a short vertical one, painted through the tool's own handlers.
	REGION_SCRIPT.install(_app)
	REGION_SCRIPT.painting = true
	for x in range(60, 140, 8):
		_app._drag_handlers[REGION_SCRIPT.TOOL_ID].call(float(x), 80.0)
	_app._release_handlers[REGION_SCRIPT.TOOL_ID].call(140.0, 80.0, true)
	for y in range(60, 100, 8):
		_app._drag_handlers[REGION_SCRIPT.TOOL_ID].call(100.0, float(y))
	_app._release_handlers[REGION_SCRIPT.TOOL_ID].call(100.0, 100.0, true)
	REGION_SCRIPT.painting = false


## The independent recount: cells whose centre lies inside any dab (the engine's rule, written again).
func _recount_cells() -> int:
	var minx := 1 << 30
	var miny := 1 << 30
	var maxx := -(1 << 30)
	var maxy := -(1 << 30)
	for d in REGION_SCRIPT.dabs:
		var v := d as Vector3
		minx = mini(minx, int(floor(v.x - v.z)))
		miny = mini(miny, int(floor(v.y - v.z)))
		maxx = maxi(maxx, int(ceil(v.x + v.z)))
		maxy = maxi(maxy, int(ceil(v.y + v.z)))
	var g: Vector2i = _bridge.grid_size()
	var n := 0
	for y in range(maxi(miny, 0), mini(maxy, g.y)):
		for x in range(maxi(minx, 0), mini(maxx, g.x)):
			for d in REGION_SCRIPT.dabs:
				var v := d as Vector3
				var dx := float(x) + 0.5 - v.x
				var dy := float(y) + 0.5 - v.y
				if dx * dx + dy * dy <= v.z * v.z:
					n += 1
					break
	return n


func _finish() -> void:
	OS.unset_environment(ENV_VAR)
	LlmWire.shutdown_all()
	if _server != null:
		_server.stop()
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _edit(op: String, extra: Dictionary = {}, region := "") -> Dictionary:
	var e := {"operation": op, "region_id": region if region != "" else _region_id()}
	e.merge(extra, true)
	return e


func _region_id() -> String:
	return String(REGION_SCRIPT.facts(_bridge).get("region", {}).get("id", ""))


func _edit_call(edits: Array, summary := "") -> Dictionary:
	var args := {"edits": edits}
	if summary != "":
		args["summary"] = summary
	return {"name": EDIT_TOOL, "args": args}


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--force-touch":
			pass
		elif args[i] == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		elif args[i] == "--failfast":
			_failfast = true
		elif args[i] == "--corpus" and i + 1 < args.size():
			i += 1
			_corpus_path = args[i]
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch, --shots <dir> and --corpus <path> only" % args[i])
			get_tree().quit(2)
			return
		i += 1
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed; this probe clicks real buttons and saves frames")
		get_tree().quit(2)
		return
	get_tree().create_timer(1500.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	OS.set_environment(ENV_VAR, SECRET)
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
	_ok(_bridge.llm_api and _win != null and _app.get("undo_groups") != null, "the library has LlmBridge; the shell has the panel and the grouped undo")
	if not _bridge.llm_api or _win == null:
		_finish()
		return
	_bridge.generation_started.connect(func(): _gen_started += 1)
	await _generate(SEEDS[0])

	# ---------------------------------------------------------------- 0. the switch
	print("== 0. the Edit switch and its source ==")
	_ok(DccSettings.llm_value("edit") == false and DccSettings.llm_value("ctx_region") == false, "Edit actions and the region source are OFF by default")
	_ok(not DccSettings.LLM_PROFILE_KEYS.has("ctx_region"), "the region source is not carried by profiles (a profile switch never turns it on)")
	var sw := LlmSettingsWindow.new()
	sw.app = _app
	sw.bridge = _bridge
	_ok(sw._edit_unavailable() == "", "this build can enable Edit actions (grouped undo and the three region functions are present)")
	var saved_ug = _app.undo_groups
	_app.undo_groups = null
	_ok(sw._edit_unavailable().contains("grouped undo"), "with the grouped undo missing the switch is unavailable, with a reason: '%s'" % sw._edit_unavailable())
	_app.undo_groups = saved_ug
	sw.free()

	# ---------------------------------------------------------------- 1. the gates
	print("== 1. what keeps Edit closed ==")
	_reset_llm()
	_win.open()
	await _settle(10)
	_win._pick_task("edit")
	await _type("raise it")
	var reason_of := func() -> String: return (_c("reason") as Label).text
	_ok((_c("send") as Button).disabled and reason_of.call() != "", "everything off: Send disabled with a reason: '%s'" % reason_of.call().left(80))
	_configure({"edit": false})
	_win._refresh()
	var r_edit_off: String = reason_of.call()
	_ok((_c("send") as Button).disabled and r_edit_off != "", "Edit switch off: Send disabled with a reason: '%s'" % r_edit_off.left(90))
	_configure({"edit": true, "ctx_region": false})
	_win._refresh()
	var r_src_off: String = reason_of.call()
	_ok((_c("send") as Button).disabled and r_src_off.contains("painted region and its heights") and r_src_off != r_edit_off, "region source off: its own reason: '%s'" % r_src_off.left(90))
	_configure({"ctx_region": true})
	REGION_SCRIPT.clear(_app)
	_win._edit_key = ""
	_win._refresh()
	var r_none: String = reason_of.call()
	_ok((_c("send") as Button).disabled and r_none.to_lower().contains("no region"), "no region: Send disabled, '%s'" % r_none.left(90))
	_ok((_c("region_status") as Label).text.begins_with("Region: none"), "the region line says none: '%s'" % (_c("region_status") as Label).text.left(60))
	# The region SOURCE is a Rust gate too (not only the window's reason line): a request made with it off is refused
	# before anything is dialled, with its own code.
	_configure({"ctx_region": false})
	_paint_region()
	var acc_src: int = _server.accepted
	var no_src: Dictionary = _win.run.begin(DccSettings.llm_gate_dict(), "edit", DccSettings.llm_consented_origins(),
		{"request": "raise it", "edit": REGION_SCRIPT.facts_json(_bridge), "max_tokens": 256})
	_ok(not bool(no_src.get("ok", true)) and String(no_src.get("code", "")) == "region_context_off" and _server.accepted == acc_src, "Rust refuses an edit request while 'Send: the painted region' is off (code %s), nothing dialled" % str(no_src.get("code", "")))
	_configure({"ctx_region": true})
	# A hosted endpoint is refused by Rust for this feature, with zero connections.
	_configure({"preset_id": "openai", "base_url": "https://api.openai.com/v1", "model": "gpt-x", "remote_allowed": true, "local_allowed": false})
	DccSettings.llm_add_consent("https://api.openai.com")
	_paint_region()
	_win._edit_key = ""
	var acc0: int = _server.accepted
	var hosted: Dictionary = _win.run.begin(DccSettings.llm_gate_dict(), "edit", DccSettings.llm_consented_origins(),
		{"request": "raise it", "edit": REGION_SCRIPT.facts_json(_bridge), "max_tokens": 256})
	_ok(not bool(hosted.get("ok", true)) and String(hosted.get("code", "")) == "edit_local_only", "a hosted endpoint is refused in Rust for edit actions (code %s): '%s'" % [str(hosted.get("code", "")), String(hosted.get("reason", "")).left(70)])
	_ok(_server.accepted == acc0 and not _win.run.busy(), "...and nothing was dialled")
	_configure({"preset_id": "ollama", "base_url": "http://127.0.0.1:%d/v1" % _port, "model": "mock-model", "remote_allowed": false, "local_allowed": true})

	# ---------------------------------------------------------------- 2. the region input
	print("== 2. the region input ==")
	REGION_SCRIPT.clear(_app)
	_win._edit_key = ""
	_win._pick_task("edit")
	await _type("raise it into a ridge about 300 m high")
	var paint_btn := _c("region_paint") as Button
	if _mode == "phone":
		_ok(paint_btn.disabled and paint_btn.tooltip_text.contains("phone"), "phone: Paint region is disabled, with its reason: '%s'" % paint_btn.tooltip_text.left(70))
	else:
		_ok(not paint_btn.disabled, "Paint region is enabled with a world open")
		await _click_until(paint_btn, func(): return not _win.visible)
		await _settle(6)
		_ok(not _win.visible and REGION_SCRIPT.painting and _app.armed_tool == REGION_SCRIPT.TOOL_ID, "Paint hides the Assistant, arms the region brush and starts painting (armed %s)" % _app.armed_tool)
		var bar = _app.viewport.get_node_or_null("AssistRegionBar")
		_ok(bar != null and (bar as Control).visible, "the painting bar is on the map")
		if bar != null:
			var br: Rect2 = (bar as Control).get_global_rect()
			var vr: Rect2 = _app.viewport.get_global_rect()
			_ok(vr.encloses(br) and br.size.y < (130.0 if _mode != "desktop" else 80.0), "the bar %s lies inside the map %s and is not a tower of one-control lines" % [br, vr])
			if _mode != "desktop":
				for nm in ["assist_done", "assist_clear", "assist_undo_stroke", "assist_brush"]:
					var bb := (bar as Control).find_child(nm, true, false) as Control
					_ok(bb != null and bb.get_global_rect().size.y >= 43.5, "the bar's %s meets the 44 px floor: %s" % [nm, str(bb.get_global_rect().size) if bb != null else "missing"])
		# One REAL mouse drag across the map.
		var vp = _app.viewport
		var rect: Rect2 = vp.overlay.displayed_rect()
		var g: Vector2i = _bridge.grid_size()
		var to_screen := func(gx: float, gy: float) -> Vector2:
			var rel := Vector2((gx + 0.5) / float(g.x), (gy + 0.5) / float(g.y))
			var local: Vector2 = vp._camera.position + (rect.position + rel * rect.size) * float(vp.zoom())
			return _global_of(vp.get_global_transform() * local, vp.get_viewport())
		var start: Vector2 = to_screen.call(70.0, 70.0)
		await _mouse(start, null)
		await _mouse(start, true)
		for k in range(1, 9):
			await _mouse(to_screen.call(70.0 + 6.0 * k, 70.0 + 3.0 * k), null, true)
		await _mouse(to_screen.call(118.0, 94.0), false)
		var real_drag := REGION_SCRIPT.dabs.size()
		_ok(real_drag >= 3, "a real mouse drag on the map painted dabs (%d)" % real_drag)
		await _shot("2_painting")
		_ok(vp.tool_overlay.assist_dabs.size() == REGION_SCRIPT.dabs.size() and not REGION_SCRIPT.dabs.is_empty(), "the overlay draws exactly the dabs (%d)" % vp.tool_overlay.assist_dabs.size())
		REGION_SCRIPT.undo_stroke(_app)
		_ok(REGION_SCRIPT.dabs.is_empty(), "Undo stroke removed the whole stroke (the press and the drag are ONE stroke)")
		var done0 = _app.viewport.get_node("AssistRegionBar").find_child("assist_done", true, false) as Button
		await _click_until(done0, func(): return _win.visible)
		await _settle(10)
		_ok(_win.visible and _win._task == "edit" and (_c("input") as TextEdit).text == "raise it into a ridge about 300 m high", "Done (a real click) brought the Assistant back on Edit with the prompt kept")
		_ok(not REGION_SCRIPT.painting and _app.armed_tool == "inspect" and _app.viewport.get_node_or_null("AssistRegionBar") == null, "...and the brush is disarmed and the bar is gone (armed %s)" % _app.armed_tool)
	_paint_region()
	var strokes: int = REGION_SCRIPT.strokes.size()
	_ok(strokes == 2 and REGION_SCRIPT.dabs.size() >= 12, "two strokes were painted through the handlers (%d strokes, %d dabs)" % [strokes, REGION_SCRIPT.dabs.size()])
	var f: Dictionary = REGION_SCRIPT.facts(_bridge)
	_ok(f.has("region") and f.has("world"), "the engine described the region: %s" % str(f.get("withheld", "ok")))
	var cells := int(f["region"]["cells"])
	var recount := _recount_cells()
	_ok(cells == recount and cells > 1000, "the engine's cell count %d equals an independent recount %d" % [cells, recount])
	var bb: Array = f["region"]["bbox"]
	_ok(int(bb[0]) >= 0 and int(bb[0]) + int(bb[2]) <= 256 and int(bb[1]) + int(bb[3]) <= 164, "the bounding box %s lies inside the 256 x 164 world" % str(bb))
	_ok(f["region"].has("ground") and float(f["region"]["ground"]["min_m"]) <= float(f["region"]["ground"]["mean_m"]) and float(f["region"]["ground"]["mean_m"]) <= float(f["region"]["ground"]["max_m"]), "the region's measured heights are ordered min <= mean <= max: %s" % str(f["region"]["ground"]))
	var id1 := String(f["region"]["id"])
	_ok(id1.begins_with("region_") and id1 == _region_id(), "the id is stable for the same shape: %s" % id1)
	REGION_SCRIPT.set_brush(24.0)
	REGION_SCRIPT.dabs.append(Vector3(30.0, 30.0, 24.0))
	_ok(_region_id() != id1, "a different shape has a different id")
	REGION_SCRIPT.dabs.pop_back()
	_ok(_region_id() == id1, "...and removing the extra dab gives the first id back")
	# Use the box.
	REGION_SCRIPT.clear(_app)
	_bridge.region_set(20.0, 20.0, 40.0, 30.0)
	_win._on_region_box()
	_ok(REGION_SCRIPT.rect == Rect2(20.0, 20.0, 40.0, 30.0) and int(REGION_SCRIPT.facts(_bridge)["region"]["cells"]) == 1200, "Use the box adopts the Region select rectangle: 40 x 30 = 1200 cells")
	_ok(String(REGION_SCRIPT.facts(_bridge)["region"]["shape"]) == "box", "...described as a box")
	_bridge.region_clear()
	REGION_SCRIPT.clear(_app)
	_paint_region()
	if _mode != "phone":
		# Done brings the window back with the prompt kept.
		_win.open()
		await _settle(8)
		_win._pick_task("edit")
		await _type("keep this prompt")
		await _click_until(_c("region_paint") as Button, func(): return not _win.visible)
		_ok(not _win.visible, "Paint hid the window again")
		var done_btn := _app.viewport.get_node("AssistRegionBar").find_child("assist_done", true, false) as Button
		await _click_until(done_btn, func(): return _win.visible)
		await _settle(10)
		_ok(_win.visible and _win._task == "edit", "Done brought the Assistant back on the Edit task")
		_ok((_c("input") as TextEdit).text == "keep this prompt", "...with the prompt as the user left it")
		_ok(not REGION_SCRIPT.painting and _app.armed_tool == "inspect", "...the brush is disarmed (armed %s)" % _app.armed_tool)
		_ok(not REGION_SCRIPT.dabs.is_empty(), "...and the region is still there (%d dabs)" % REGION_SCRIPT.dabs.size())
	_win.open()
	await _settle(8)

	# ---------------------------------------------------------------- 3. the offer
	print("== 3. what the first request offers ==")
	for wire in ["openai", "anthropic"]:
		var anth: bool = wire == "anthropic"
		_configure({"preset_id": "custom_anthropic" if anth else "ollama"})
		_win._edit_key = ""
		_win._refresh()
		var n := await _ask("tools_anthropic" if anth else "tools", [{"text": "I cannot do that with raise, lower or smooth."}], "Raise the area I painted into a ridge about 300 m high.")
		_ok(n == 1, "%s: one request (%d)" % [wire, n])
		var b := _body(_server.requests.size() - 1)
		_ok(_tool_names(b, anth) == [EDIT_TOOL], "%s: exactly one tool is offered: %s" % [wire, str(_tool_names(b, anth))])
		var et := _edit_tool(b, anth)
		var items: Dictionary = et["schema"]["properties"]["edits"]["items"]["properties"]
		_ok(items["region_id"]["enum"] == [id1] and items["operation"]["enum"] == ["raise", "lower", "smooth"], "%s: the region_id enum is the one offered id and the operations are raise, lower, smooth" % wire)
		_ok(float(items["amount_m"]["maximum"]) == 500.0 and et["schema"]["properties"]["edits"]["maxItems"] == 3, "%s: the schema carries the amount cap 500 m and at most 3 edits" % wire)
		var sys := _system_of(b, anth)
		_ok(sys.contains("REGION " + id1) and sys.contains("highest") and sys.contains("never apply them") and sys.contains("cells"), "%s: the system prompt carries the region block (numbers) and the PROPOSE-only rule" % wire)
		var blob := JSON.stringify(b)
		_ok(not blob.contains("image") and not blob.contains("data:") and not blob.contains("get_param_schema") and not blob.contains("propose_view_actions"), "%s: no image part, no schema tool, no view tool in the body" % wire)
		_ok(String(et["description"]).contains("nothing happens by this call"), "%s: the tool says plainly that nothing happens by the call" % wire)
	var ans := _text_of(_win._msgs)
	_ok(ans.contains("Nothing was proposed") and not ans.contains("Choose groups"), "a words-only answer ends as 'Nothing was proposed', without the setting-groups hint")
	await _shot("3_offer")
	await _type("Raise it.")
	var sent: String = (_c("sent") as Label).text
	_ok(sent.contains("This will send") and sent.contains("as numbers only") and sent.contains("No picture of the map") and sent.contains("cells"), "the 'this will send' line itemises the region as numbers and says no picture: '%s'" % sent.left(120))
	_win._on_view_region()
	await _settle(4)
	_ok(_text_of(_win._msgs).contains("REGION " + id1), "'View what is sent' shows the exact region text")

	# ---------------------------------------------------------------- 4. card, apply, undo, redo (3 seeds)
	for si in SEEDS.size():
		var seed_: int = SEEDS[si]
		if si > 0:
			await _generate(seed_)
			_ok(REGION_SCRIPT.is_empty(), "seed %d: generating a world dropped the painted region" % seed_)
			_win.open()
			await _settle(6)
		print("== 4. seed %d: card, preview, one Apply, one undo ==" % seed_)
		_configure({})
		_paint_region()
		_win._edit_key = ""
		var rid := _region_id()
		var f4: Dictionary = REGION_SCRIPT.facts(_bridge)
		var amt := 300.0
		print("  (seed %d: the region's highest cell is %.0f m of a %.0f m peak)" % [seed_, float(f4["region"]["ground"]["max_m"]), float(f4["world"]["peak_m"])])
		var edits: Array = [_edit("raise", {"amount_m": amt, "falloff": "linear", "softness": 1.0, "reason": "a ridge along the middle"}, rid),
			_edit("smooth", {"strength": 0.5, "passes": 2}, rid)]
		var snap0 := _digest()
		var g0: int = _app.undo_groups.cursor
		var gen0 := _gen_started
		var nreq := await _ask("tools", [_edit_call(edits, "A ridge, then smoothed.")], "Raise the area I painted into a ridge about %d m high, then smooth it." % int(amt))
		_ok(nreq == 1, "seed %d: one request (%d)" % [seed_, nreq])
		var card := _edit_card()
		if card.is_empty() or card.get("apply") == null:
			_ok(false, "seed %d: no edit card with Apply; stopping" % seed_)
			_finish()
			return
		var ct := _text_of(card["box"])
		_ok(ct.contains("MODEL PROPOSAL") and ct.contains("TERRAIN EDIT") and ct.contains("Raise the region by") and ct.contains("Smooth the region") and ct.contains("a ridge along the middle") and ct.contains("(model's words)"), "seed %d: the card carries the chip, both edits and the model's reason as its own words" % seed_)
		var pv: Dictionary = card["preview"]
		_ok(bool(pv.get("ok", false)) and int(pv["changed"]) > 0 and float(pv["max_down_m"]) <= 600.01 and float(pv["max_up_m"]) <= 600.01, "seed %d: the app's preview (raise then smooth): %d cells change, largest rise %.1f m, largest fall %.1f m, none over the 600 m per-cell limit (%d held to it)" % [seed_, int(pv.get("changed", 0)), float(pv.get("max_up_m", 0.0)), float(pv.get("max_down_m", 0.0)), int(pv.get("limited", 0))])
		var raise_only: Dictionary = _EDIT_SCRIPT.preview(_bridge, [(card["info"]["valid"] as Array)[0]])
		_ok(bool(raise_only.get("ok", false)) and float(raise_only["max_up_m"]) <= amt + 0.01 and float(raise_only["max_up_m"]) > amt * 0.3, "seed %d: the raise alone rises at most the %.0f m asked (largest %.1f m)" % [seed_, amt, float(raise_only.get("max_up_m", -1.0))])
		_ok(ct.contains("Preview (worked out by the app") and ct.contains("Heights in the region") and ct.contains("cells change"), "seed %d: the preview figures are on the card" % seed_)
		var tb := card["box"].find_child("edit_thumb_before", true, false) as TextureRect
		var ta := card["box"].find_child("edit_thumb_after", true, false) as TextureRect
		_ok(tb != null and ta != null and tb.texture != null and ta.texture != null and tb.texture.get_width() >= 8, "seed %d: before and after pictures are on the card (%s)" % [seed_, str(tb.texture.get_size()) if tb != null and tb.texture != null else "missing"])
		if tb != null and ta != null and tb.texture != null and ta.texture != null:
			var ib := tb.texture.get_image()
			var ia := ta.texture.get_image()
			var diff := 0
			for yy in ib.get_height():
				for xx in ib.get_width():
					if ib.get_pixel(xx, yy) != ia.get_pixel(xx, yy):
						diff += 1
			_ok(diff > 20, "seed %d: the two pictures differ (%d pixels): the preview shows a change" % [seed_, diff])
		_ok((card["apply"] as Button).text == "Apply 2 edits" and not (card["apply"] as Button).disabled, "seed %d: 'Apply 2 edits' is enabled" % seed_)
		_ok(_digest() == snap0 and _app.undo_groups.cursor == g0 and _gen_started == gen0, "seed %d: NOTHING changed before the click (digest, undo stack, generations)" % seed_)
		var srect: Rect2 = (_c("send") as Button).get_global_rect()
		var crect: Rect2 = (card["box"] as Control).get_global_rect()
		_ok(srect.end.x <= float(_win.size.x) + 0.5 and crect.end.x <= float(_win.size.x) + 0.5, "seed %d: Send and the card stay inside the window width %d" % [seed_, _win.size.x])
		if _mode != "desktop":
			for k in ["apply", "dismiss"]:
				var bsz: Vector2 = (card[k] as Button).get_global_rect().size
				_ok(bsz.y >= 43.5 and bsz.x >= 43.5, "seed %d: the card's %s meets the 44 px floor: %s" % [seed_, k, bsz])
			for nm in ["region_paint", "region_box", "region_clear", "region_view"]:
				var rsz: Vector2 = _c(nm).get_global_rect().size
				_ok(rsz.y >= 43.5 and rsz.x >= 43.5, "seed %d: %s meets the 44 px floor: %s" % [seed_, nm, rsz])
		if si == 0:
			await _shot("4_card")
		await _click_until(card["apply"], func(): return card["state"] != "new")
		await _settle(10)
		_ok(card["state"] == "applied", "seed %d: the click applied the card: %s" % [seed_, (card["reason"] as Label).text.left(110)])
		var snap1 := _digest()
		_ok(snap1["field"] != snap0["field"], "seed %d: the height field changed" % seed_)
		_ok(snap1["params"] == snap0["params"] and snap1["ways_manual"] == snap0["ways_manual"] and snap1["colour_paint"] == snap0["colour_paint"], "seed %d: parameters, hand-drawn ways and colour paint are untouched" % seed_)
		_ok(_app.undo_groups.cursor == g0 + 1 and String(_app.undo_groups.undo_label()).begins_with("Assistant:"), "seed %d: exactly ONE undo group was added: '%s'" % [seed_, _app.undo_groups.undo_label()])
		_ok(_gen_started == gen0, "seed %d: Apply started no generation" % seed_)
		_ok((card["reason"] as Label).text.contains("%d cells changed" % int(pv["changed"])), "seed %d: the preview's %d changed cells are exactly what Apply changed: '%s'" % [seed_, int(pv["changed"]), (card["reason"] as Label).text.left(70)])
		if si == 0:
			await _shot("4_applied")
		_app.undo_last()
		await _settle(10)
		_ok(_digest()["field"] == snap0["field"], "seed %d: ONE Edit > Undo restored the height field (digest equal to the pre-apply state)" % seed_)
		_ok(card["state"] == "undone" and (card["reason"] as Label).text.contains("Undone"), "seed %d: the card reads Undone: '%s'" % [seed_, (card["reason"] as Label).text.left(70)])
		_app.redo_last()
		await _settle(10)
		_ok(_digest()["field"] == snap1["field"] and card["state"] == "applied", "seed %d: Edit > Redo re-applies it (digest equal to the applied state) and the card reads applied again" % seed_)
		_app.undo_last()
		await _settle(6)
		_ok(_digest()["field"] == snap0["field"], "seed %d: undone again, back to the original" % seed_)

	# ---------------------------------------------------------------- 4b. the region must still be the region
	print("== 4b. a region changed after the proposal is refused at Apply ==")
	_configure({})
	_paint_region()
	_win._edit_key = ""
	var rid4b := _region_id()
	await _ask("tools", [_edit_call([_edit("raise", {"amount_m": 20}, rid4b)])], "Raise the painted area by 20 m.")
	var c4b := _edit_card()
	if c4b.is_empty() or c4b.get("apply") == null:
		_ok(false, "4b: no edit card with Apply; stopping")
		_finish()
		return
	var d4b := _digest()
	var g4b: int = _app.undo_groups.cursor
	REGION_SCRIPT.dabs.append(Vector3(40.0, 40.0, 16.0))
	_ok(_region_id() != rid4b, "4b: painting one more dab changed the region's id")
	await _click_until(c4b["apply"], func(): return c4b["state"] != "new")
	_ok(c4b["state"] == "failed" and (c4b["reason"] as Label).text.contains("region changed"), "4b: Apply was refused with the reason: '%s'" % (c4b["reason"] as Label).text.left(90))
	_ok(_digest() == d4b and _app.undo_groups.cursor == g4b, "4b: nothing changed and no undo group was opened")
	REGION_SCRIPT.dabs.pop_back()

	# ---------------------------------------------------------------- 5. refusals
	for wire in ["openai", "anthropic"]:
		var anth: bool = wire == "anthropic"
		var scen := "tools_anthropic" if anth else "tools"
		_configure({"preset_id": "custom_anthropic" if anth else "ollama"})
		_paint_region()
		_win._edit_key = ""
		_win._refresh()
		var rid5 := _region_id()
		print("== 5. %s: refused edits change nothing ==" % wire)
		var cases := [
			["over-cap amount", [_edit("raise", {"amount_m": 900}, rid5)], "R8"],
			["another region", [_edit("raise", {"amount_m": 50}, "region_ffffffffffff")], "R6"],
			["unknown operation", [_edit("volcano", {"amount_m": 50}, rid5)], "R5"],
			["free-form field", [_edit("raise", {"amount_m": 50, "command": "generate"}, rid5)], "R4"],
			["wrong type", [_edit("raise", {"amount_m": "300"}, rid5)], "R7"],
			["too many edits", [_edit("raise", {"amount_m": 5}, rid5), _edit("lower", {"amount_m": 5}, rid5), _edit("smooth", {"strength": 0.5, "passes": 1}, rid5), _edit("raise", {"amount_m": 6}, rid5)], "R3"],
		]
		for cs in cases:
			var d0 := _digest()
			var g5: int = _app.undo_groups.cursor
			var bad := _edit_call(cs[1])
			var n5 := await _ask(scen, [bad, bad], "Please do the %s." % cs[0])
			_ok(n5 == 2, "%s %s: the proposal and ONE retry = 2 requests (%d)" % [wire, cs[0], n5])
			var ec := _edit_card()
			_ok(not ec.is_empty() and ec.get("apply") == null and _text_of(ec["box"]).contains("Nothing could be applied") and _text_of(ec["box"]).contains(String(cs[2])), "%s %s: the final card names rule %s and has NO Apply" % [wire, cs[0], cs[2]])
			_ok(_digest() == d0 and _app.undo_groups.cursor == g5, "%s %s: NOTHING changed" % [wire, cs[0]])
		if wire == "openai":
			await _shot("5_refused")

	# ---------------------------------------------------------------- 6. the engine's own second wall
	print("== 6. the engine call refuses on its own ==")
	_paint_region()
	var spec: Dictionary = REGION_SCRIPT.spec()
	var d6 := _digest()
	var u6: int = _bridge.undo_commit_seq()
	var mk := func(op: String, extra: Dictionary) -> Dictionary:
		var o := {"operation": op, "falloff": "smooth", "softness": 0.5}
		o.merge(extra, true)
		return o
	var walls := [
		["amount over the cap", [mk.call("raise", {"amount_m": 500.5})]],
		["amount zero", [mk.call("raise", {"amount_m": 0.0})]],
		["four operations", [mk.call("raise", {"amount_m": 1.0}), mk.call("lower", {"amount_m": 1.0}), mk.call("smooth", {"strength": 0.5, "passes": 1}), mk.call("raise", {"amount_m": 2.0})]],
		["total over 600 m", [mk.call("raise", {"amount_m": 400.0}), mk.call("lower", {"amount_m": 201.0})]],
		["passes over 8", [mk.call("smooth", {"strength": 0.5, "passes": 9})]],
		["strength over 1", [mk.call("smooth", {"strength": 1.5, "passes": 1})]],
		["softness over 1", [mk.call("raise", {"amount_m": 5.0, "softness": 1.5})]],
		["a text amount", [mk.call("raise", {"amount_m": "9"})]],
		["unknown falloff", [mk.call("raise", {"amount_m": 5.0, "falloff": "cubic"})]],
		["unknown operation", [mk.call("erode", {"amount_m": 5.0})]],
	]
	for w in walls:
		var r: Dictionary = _bridge.region_edit_apply(spec, w[1], "probe")
		var p: Dictionary = _bridge.region_edit_preview(spec, w[1])
		_ok(not bool(r.get("ok", true)) and String(r.get("reason", "")) != "" and not bool(p.get("ok", true)), "engine: %s is refused by apply AND preview with a reason: '%s'" % [w[0], String(r.get("reason", "")).left(70)])
	_ok(_digest() == d6 and _bridge.undo_commit_seq() == u6, "engine: none of the refusals wrote a cell or pushed an undo step")
	var tiny: Dictionary = _bridge.region_describe({"rect": [10.0, 10.0, 3.0, 3.0]})
	_ok(not bool(tiny.get("ok", true)) and String(tiny["reason"]).contains("at least"), "engine: a 9-cell region is too small: '%s'" % String(tiny.get("reason", "")).left(60))
	var outside: Dictionary = _bridge.region_describe({"rect": [3000.0, 3000.0, 30.0, 30.0]})
	_ok(not bool(outside.get("ok", true)) and String(outside["reason"]).contains("outside"), "engine: a region outside the world is refused: '%s'" % String(outside.get("reason", "")).left(60))
	var nan_dab: Dictionary = _bridge.region_describe({"dabs": [[10.0, 10.0, NAN]]})
	_ok(not bool(nan_dab.get("ok", true)), "engine: a NaN dab radius is refused, not repaired: '%s'" % String(nan_dab.get("reason", "")).left(60))

	# ---------------------------------------------------------------- 7. generating
	print("== 7. Apply while a world generates ==")
	_configure({})
	_paint_region()
	_win._edit_key = ""
	var rid7 := _region_id()
	await _ask("tools", [_edit_call([_edit("raise", {"amount_m": 5}, rid7)])], "Raise it by five metres.")
	var c7 := _edit_card()
	if c7.is_empty() or c7.get("apply") == null:
		_ok(false, "7: no edit card with Apply; stopping")
		_finish()
		return
	_bridge.generate({"seed": 4242, "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": false})
	await _settle(2)
	_win._refresh_cards()
	_ok(_bridge.generating and (c7["apply"] as Button).disabled and (c7["reason"] as Label).text.contains("generating"), "7: while the world generates, Apply is disabled with the reason: '%s'" % (c7["reason"] as Label).text.left(60))
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	_ok(REGION_SCRIPT.is_empty(), "7: the new world dropped the painted region")

	# ---------------------------------------------------------------- 8. the hostile corpus
	print("== 8. the hostile corpus through the real panel ==")
	var txt := FileAccess.get_file_as_string(_corpus_path)
	var doc = JSON.parse_string(txt)
	_ok(doc is Dictionary and (doc["rows"] as Array).size() >= 20, "the edit corpus is readable and has at least 20 rows (%d)" % ((doc["rows"] as Array).size() if doc is Dictionary else 0))
	if doc is Dictionary:
		_win.open()
		await _settle(6)
		for wire in ["openai", "anthropic"]:
			var anth: bool = wire == "anthropic"
			_configure({"preset_id": "custom_anthropic" if anth else "ollama"})
			_paint_region()
			_win._edit_key = ""
			_win._pick_task("edit")
			var ids := {}
			var rid8 := _region_id()
			for row in doc["rows"]:
				var rd: Dictionary = row
				var d8 := _digest()
				var g8: int = _app.undo_groups.cursor
				var gen8 := _gen_started
				var script: Array = []
				var obey: Array = rd.get("obey", [])
				for call in obey:
					var a: String = JSON.stringify(call["arguments"]).replace("region_ab12cd34", rid8)
					var rx := RegEx.create_from_string("@PAD([0-9]+)")
					for m in rx.search_all(a):
						a = a.replace(m.get_string(), "x".repeat(int(m.get_string(1))))
					script.append({"name": String(call["name"]), "args": a})
				var step: Dictionary
				if rd.has("reply_text"):
					step = {"text": String(rd["reply_text"])}
				elif script.size() == 1:
					step = script[0]
				else:
					step = {"calls": script}
				var n0: int = _server.requests.size()
				var c0: int = _win.cards.size()
				await _ask("tools_anthropic" if anth else "tools", [step], "Raise my area. " + String(rd["text"]).left(300))
				var bodies_ok := true
				for k in range(n0, _server.requests.size()):
					bodies_ok = bodies_ok and _tool_names(_body(k), anth) == [EDIT_TOOL]
				var ec8: Dictionary = {} if _win.cards.size() == c0 else _edit_card()
				var label := "%s %s" % [wire, String(rd["id"])]
				_ok(bodies_ok, "%s: every request offered exactly the edit tool" % label)
				_ok(_digest() == d8 and _app.undo_groups.cursor == g8 and _gen_started == gen8, "%s: nothing was written, no undo step, no generation" % label)
				if bool(rd.get("expect_refused", false)):
					_ok(not _win.run.busy() and ec8.is_empty(), "%s: an unoffered tool never made a card" % label)
				if rd.has("expect_rule") and not ec8.is_empty() and ec8.get("state") == "new":
					_ok(_text_of(ec8["box"]).contains(String(rd["expect_rule"])) or _text_of(_win._msgs).contains(String(rd["expect_rule"])), "%s: the rule %s is shown" % [label, rd["expect_rule"]])
				if rd.has("expect_valid") and (rd["expect_valid"] as Array).is_empty() and not ec8.is_empty():
					_ok(ec8.get("apply") == null, "%s: no Apply for an all-refused proposal" % label)
				ids[String(rd["id"])] = true
			_ok(ids.size() == (doc["rows"] as Array).size(), "%s: all %d rows ran" % [wire, ids.size()])

	# ---------------------------------------------------------------- 9. done
	print("== 9. done ==")
	await _shot("9_end")
	_finish()
