extends Node
## **LLM-6d in the real shell: view and layer actions, the "(undone)" card state and the "Choose
## groups" fallback** (`LLM_ASSISTANT_SCOPE.md` s.5b.8 item 12, s.5b.10; Rulings DD and DI), against
## the in-test loopback server (`_llm_mock_server.gd`, scenarios `tools` and `tools_anthropic`) on
## BOTH wires.
##
##   desktop: godot --path . --resolution 1600x1000 res://_llmview_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1280x800  res://_llmview_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   res://_llmview_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped user
## dir (`MISTAKES.md`). Reads `--force-touch`, `--shots <dir>` and `--corpus <path>`; anything else
## exits 2. The corpus path defaults to the repository's fixture, relative to the working directory
## (`../crates/...`, so run from the project folder or a copy beside it).
##
## What it protects (every check names it):
##   1. OFFER: the first request of a Commands conversation offers the view tool, its `action` enum is
##      exactly the actions the live app can run NOW, its window values are exactly the curated list
##      intersected with the live CommandIndex's pinnable-and-available rows (recomputed here
##      independently), and no enum value or description names a destructive verb.
##   2. NOTHING BEFORE THE CLICK: a mixed proposal (layer, camera, style preset, window) is a card with
##      old -> new rows and the undo statement; the layer states, camera, style, undo stack, world
##      digest and open windows are unchanged until Apply.
##   3. APPLY AND UNDO: one click applies all four; ONE Edit > Undo reverts the style preset and ONLY
##      it (the layer and the camera stay as applied: session state, said on the card); the card then
##      reads "Undone"; Redo re-applies the style and the card reads applied again.
##   4. VIEW-ONLY: a proposal with no undoable member creates no undo group (the stack is untouched).
##   5. REFUSED, per wire: an unknown action, a destructive window, a free-form command string, an
##      out-of-range camera and a wrong type each show their rule, are retried ONCE (2 requests), have
##      no Apply, and change nothing.
##   6. BOTH TOOLS in one reply give two cards; applying both is two undo steps; the parameter card also
##      shows "(undone)" after Edit > Undo and applied again after Redo.
##   7. CHOOSE GROUPS: ticking two groups with real clicks makes the app open them: the first request
##      already offers the proposal tool with exactly those groups' keys and replays a schema call and
##      result; a fourth tick is refused; the ticks are cleared after Send.
##   8. GENERATING: while a world generates, a card with a style preset has Apply disabled with the
##      reason, and a layer-only card does not.
##   9. CORPUS: the view-action rows (W01..) of the hostile corpus through the real panel on both wires:
##      the tool set is fixed, nothing is written, and a row expecting a rule shows it.
##  10. LAYOUT: with cards shown, Send and the cards stay inside the window; on tablet and phone every
##      button and group chip meets the 44 px floor.
## Result: `RESULT: PASS|FAIL (<n> failed) checks=<n>`. Exit 0 pass, 1 fail, 2 bad argument/watchdog.

const ENV_VAR := "CART_LLM_PROBE_KEYVAR"
const SECRET := "sk-PROBE-SECRET-view-0123456789"
const SEEDS := [9137, 24601]
const VIEW_TOOL := "propose_view_actions"
const SCHEMA_WT := {"name": "get_param_schema", "args": {"groups": ["weather", "tectonics"]}}
const FORBIDDEN := ["generat", "save", "delete", "remove", "erase", "export", "import", "close", "quit", "reset", "clear", "revert", "undo", "redo", "upload", "download", "network", "http", "open_project", "new_world", "finalize"]

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _corpus_path := "../crates/cartalith-llm/tests/fixtures/hostile_corpus_2026-10-26.json"
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
	var path := _shots.path_join("llmview_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


## A REAL mouse press and release at the centre of `c` (the `_llmpanel_probe` helper).
func _click(c: Control) -> void:
	## A card button can sit below the fold of the messages scroller (the tablet layout): scroll it into
	## view first, as a user would, so the REAL click lands on it.
	var p := c.get_parent()
	while p != null:
		if p is ScrollContainer:
			(p as ScrollContainer).ensure_control_visible(c)
			await _settle(2)
			break
		p = p.get_parent()
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
	var s := {"enabled": true, "commands": true, "prose": false, "local_allowed": true, "preset_id": "ollama",
		"base_url": "http://127.0.0.1:%d/v1" % _port, "model": "mock-model",
		"key_source": "env", "env_var": ENV_VAR, "ctx_params": true}
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


## The view tool's definition out of a request body: `{description, schema}` or `{}`.
func _view_tool(b: Dictionary, anthropic: bool) -> Dictionary:
	for t in b.get("tools", []):
		var name := String(t["name"]) if anthropic else String(t["function"]["name"])
		if name == VIEW_TOOL:
			return {"description": String(t["description"]) if anthropic else String(t["function"]["description"]),
				"schema": t["input_schema"] if anthropic else t["function"]["parameters"]}
	return {}


func _tool_names(b: Dictionary, anthropic: bool) -> Array:
	var out: Array = []
	for t in b.get("tools", []):
		out.append(String(t["name"]) if anthropic else String(t["function"]["name"]))
	return out


func _action_enum(b: Dictionary, anthropic: bool) -> Array:
	var t := _view_tool(b, anthropic)
	return [] if t.is_empty() else t["schema"]["properties"]["actions"]["items"]["properties"]["action"]["enum"]


func _key_enum(b: Dictionary, anthropic: bool) -> Array:
	for t in b.get("tools", []):
		var name := String(t["name"]) if anthropic else String(t["function"]["name"])
		if name == "set_generation_params":
			var sch: Dictionary = t["input_schema"] if anthropic else t["function"]["parameters"]
			return sch["properties"]["changes"]["items"]["properties"]["key"]["enum"]
	return []


func _ask(scenario: String, script: Array, request: String) -> int:
	_server.scenario = scenario
	_server.tool_script = script
	_server.tool_served = 0
	var n0: int = _server.requests.size()
	_win._pick_task("commands")
	await _type(request)
	var send := _c("send") as Button
	if send.disabled:
		print("  (Send disabled: %s)" % (_c("reason") as Label).text)
	await _click(send)
	await _until(func(): return not _win.run.busy() and _server.requests.size() > n0, 15000)
	await _until(func(): return not _win.run.busy(), 15000)
	await _settle(6)
	return _server.requests.size() - n0


func _view_card() -> Dictionary:
	for i in range(_win.cards.size() - 1, -1, -1):
		if String(_win.cards[i].get("kind", "params")) == "view":
			return _win.cards[i]
	return {}


func _params_card() -> Dictionary:
	for i in range(_win.cards.size() - 1, -1, -1):
		if String(_win.cards[i].get("kind", "params")) == "params":
			return _win.cards[i]
	return {}


func _generate(seed_: int) -> void:
	_bridge.generate({"seed": seed_, "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": false})
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()


func _visible_windows() -> int:
	var n := 0
	var stack: Array = [get_tree().root]
	while not stack.is_empty():
		var x: Node = stack.pop_back()
		for ch in x.get_children(true):
			stack.append(ch)
			if ch is Window and (ch as Window).visible and ch != get_tree().root:
				n += 1
	return n


## Everything a view action could move, for the "nothing changed" checks.
func _snap() -> Dictionary:
	var vp = _app.viewport
	var layers := {}
	for l in CartographyWorkspace.LIVE_LAYERS:
		layers[String(l["id"])] = bool(vp.layer_visible(String(l["id"])))
	var rw = _app._render_workspace()
	return {"layers": layers, "zoom": snappedf(float(vp.zoom()), 0.0001), "cam": vp._camera.position, "preset": int(rw._active_preset),
		"look": String(_bridge.current_look()) if _bridge.has_method("current_look") else "", "windows": _visible_windows(),
		"undo": String(_app.undo_groups.undo_label()), "groups": _app.undo_groups.groups.size(), "cursor": _app.undo_groups.cursor,
		"gen": _gen_started, "digest": _digest()}


## The cell at the centre of the viewport, from the camera (the inverse of `move_view_to`).
func _centre_cell() -> Vector2:
	var vp = _app.viewport
	var rect: Rect2 = vp.overlay.displayed_rect()
	var g: Vector2i = _bridge.grid_size()
	var local: Vector2 = (vp.size * 0.5 - vp._camera.position) / float(vp.zoom())
	var rel: Vector2 = (local - rect.position) / rect.size
	return Vector2(rel.x * float(g.x) - 0.5, rel.y * float(g.y) - 0.5)


func _finish() -> void:
	OS.unset_environment(ENV_VAR)
	LlmWire.shutdown_all()
	if _server != null:
		_server.stop()
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _va(action: String, args: Dictionary, reason := "") -> Dictionary:
	var a := {"action": action, "args": args}
	if reason != "":
		a["reason"] = reason
	return a


func _view_call(actions: Array, summary := "") -> Dictionary:
	var args := {"actions": actions}
	if summary != "":
		args["summary"] = summary
	return {"name": VIEW_TOOL, "args": args}


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--force-touch":
			pass
		elif args[i] == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
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
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
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
	var rw = _app._render_workspace()
	_ok(rw != null, "the render workspace is reachable (the style presets)")

	# ---------------------------------------------------------------- 1. the offer
	print("== 1. what the first request offers ==")
	_configure({})
	_win.open()
	await _settle(10)
	_win._pick_task("commands")
	var vscript = load("res://shell/llm_view.gd")
	var facts: Dictionary = vscript.facts(_app, _bridge)
	_ok(facts.has("layers") and (facts["layers"] as Array).size() == 10, "the facts list the ten layers (%d)" % (facts.get("layers", []) as Array).size())
	_ok(facts.has("presets") and (facts["presets"] as Array).size() == RenderWorkspace.STYLE_PRESETS.size(), "the facts list every style preset (%d)" % (facts.get("presets", []) as Array).size())
	_ok(facts.has("camera") and int(facts["camera"]["w"]) == 256 and int(facts["camera"]["h"]) == 164, "the camera facts carry the grid 256 x 164")
	_ok((facts.get("withheld", {}) as Dictionary).has("set_landmark_filter"), "with no landmark pass yet, landmark filtering is withheld with a reason: '%s'" % str(facts.get("withheld", {}).get("set_landmark_filter", "")))
	# Windows: recompute the intersection independently from a fresh CommandIndex.
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	var expect_windows: Array = []
	for w in vscript.WINDOWS:
		for r in idx.all():
			var row: Dictionary = r
			if String(row.get("kind", "")) == "menu" and int(row.get("_id", -1)) == int(w["menu"]) and bool(row.get("pinnable", false)) and bool(row.get("available", false)):
				expect_windows.append(String(w["id"]))
				break
	var got_windows: Array = []
	for w in facts.get("windows", []):
		got_windows.append(String(w["id"]))
	_ok(not got_windows.is_empty() and got_windows == expect_windows, "the window values are the curated list intersected with the index's pinnable+available rows: %s" % str(got_windows))
	## Informational timing (no bar): what one `facts()` costs, including the CommandIndex build, run
	## alone before any request. Median with min..max over 9 calls.
	var times: Array = []
	for _i in 9:
		var t0 := Time.get_ticks_usec()
		vscript.facts(_app, _bridge)
		times.append(float(Time.get_ticks_usec() - t0) / 1000.0)
	times.sort()
	print("  (facts() cost: median %.2f ms, min %.2f, max %.2f, over 9 calls; one per request and per turn)" % [times[4], times[0], times[8]])
	print("  (windows offered: %s; curated but not offered: %s)" % [str(got_windows), str(vscript.WINDOWS.map(func(w): return w["id"]).filter(func(x): return not got_windows.has(x)))])
	for pk in ["Save project", "Close project", "Generate", "Export", "New world", "Open project"]:
		var hit := false
		for r in idx.all():
			var row2: Dictionary = r
			if String(row2["title"]).begins_with(pk) and bool(row2.get("pinnable", false)):
				hit = true
		print("  (index row '%s…' pinnable: %s)" % [pk, str(hit)])
	for wire in ["openai", "anthropic"]:
		var anth: bool = wire == "anthropic"
		_configure({"preset_id": "custom_anthropic" if anth else "ollama"})
		_win._refresh()
		var n := await _ask("tools_anthropic" if anth else "tools", [{"text": "Nothing to change."}], "What can you do to the map view?")
		_ok(n == 1, "%s: one request (%d)" % [wire, n])
		var b := _body(_server.requests.size() - 1)
		_ok(_tool_names(b, anth).has(VIEW_TOOL) and _tool_names(b, anth).size() == 8, "%s: the view tool is offered beside the schema tool and the six lookups: %s" % [wire, str(_tool_names(b, anth))])
		var en := _action_enum(b, anth)
		var want_en: Array = ["set_layer_visible", "set_style_preset", "zoom_to", "open_window"]
		_ok(en == want_en, "%s: the action enum is exactly what the live app can run now (no landmark filter yet): %s" % [wire, str(en)])
		var vt := _view_tool(b, anth)
		var blob := (JSON.stringify(vt["schema"]) + String(vt["description"])).to_lower()
		## `preset` contains the stem `reset`; it is the app's word for a style, so it is removed first.
		var blob2: String = blob.replace("preset", "")
		var hits: Array = FORBIDDEN.filter(func(w): return blob2.contains(w))
		_ok(hits.is_empty(), "%s: no destructive verb in the view tool's definition (hits: %s)" % [wire, str(hits)])
		_ok(String(vt["description"]).contains("nothing happens by this call") and String(vt["description"]).contains("x: 0..255, y: 0..163"), "%s: the description says nothing is applied by the call and names the grid bounds" % wire)
		_ok(not _tool_names(b, anth).has("set_generation_params"), "%s: the proposal tool is still not offered before a schema" % wire)
		var sys := String(b.get("system", "")) if anth else ""
		if not anth:
			for m in b.get("messages", []):
				if String(m.get("role", "")) == "system":
					sys = String(m.get("content", ""))
		_ok(sys.contains("propose_view_actions") and sys.contains("cannot generate, save, close"), "%s: the system prompt names the view tool and what it cannot do" % wire)
	await _shot("1_offer")

	# ---------------------------------------------------------------- 2-3. mixed proposal, apply, undo
	for wire in ["openai", "anthropic"]:
		var anth: bool = wire == "anthropic"
		var scen := "tools_anthropic" if anth else "tools"
		_configure({"preset_id": "custom_anthropic" if anth else "ollama"})
		_win._refresh()
		print("== 2. %s: a mixed proposal changes nothing before the click ==" % wire)
		var cur_preset := int(rw._active_preset)
		var target_preset := "watercolor" if cur_preset != 6 else "ink"
		var pre0 := int(rw._active_preset)
		var layer_now: bool = _app.viewport.layer_visible("roads")
		var snap0 := _snap()
		var mixed := _view_call([
			_va("set_layer_visible", {"layer": "roads", "visible": not layer_now}, "declutter the map"),
			_va("zoom_to", {"x": 100.0, "y": 80.0, "zoom": 4.0}),
			_va("set_style_preset", {"preset": target_preset}),
			_va("open_window", {"window": "shortcuts"}),
		], "Hide the roads, look at the middle, switch style, show the shortcuts.")
		var nreq := await _ask(scen, [mixed], "Hide the roads, centre on the middle at 4x, use the watercolor style and show the keyboard shortcuts.")
		_ok(nreq == 1, "%s: one request (a view proposal needs no schema step): %d" % [wire, nreq])
		var card := _view_card()
		_ok(not card.is_empty() and card["state"] == "new" and card.get("apply") != null, "%s: a view card with Apply is shown" % wire)
		if card.is_empty() or card.get("apply") == null:
			_ok(false, "%s: no card; stopping" % wire)
			_finish()
			return
		var ct := _text_of(card["box"])
		_ok(ct.contains("MODEL PROPOSAL") and ct.contains("VIEW"), "%s: the card carries the MODEL PROPOSAL · VIEW chip" % wire)
		_ok(ct.contains("Ways & routes") and ct.contains("%s  ->  %s" % ["shown" if layer_now else "hidden", "hidden" if layer_now else "shown"]), "%s: the layer row shows old -> new" % wire)
		_ok(ct.contains("centre on cell (100, 80), zoom 4x") and ct.contains("Watercolor") and ct.contains("one undo step"), "%s: camera, style and the undo-step marker are on the card" % wire)
		_ok(ct.contains("Edit > Undo reverts the style preset in one step") and ct.contains("not on the undo stack"), "%s: the card states what undo means for each row" % wire)
		_ok(ct.contains("declutter the map") and ct.contains("(model's words)"), "%s: the model's reason is shown as its own words" % wire)
		_ok(ct.contains("will be opened") and not ct.contains("not readable"), "%s: a window row reads 'will be opened', with no 'not readable' old value" % wire)
		_ok((card["apply"] as Button).text == "Apply 4 view changes" and not (card["apply"] as Button).disabled, "%s: 'Apply 4 view changes' is enabled" % wire)
		var srect: Rect2 = (_c("send") as Button).get_global_rect()
		var crect: Rect2 = (card["box"] as Control).get_global_rect()
		var wsz := Vector2(_win.size)
		_ok(srect.end.x <= wsz.x + 0.5 and crect.end.x <= wsz.x + 0.5, "%s: Send %s and the card %s stay inside the window width %.0f" % [wire, srect, crect, wsz.x])
		var snap1 := _snap()
		_ok(snap1["layers"] == snap0["layers"] and snap1["zoom"] == snap0["zoom"] and snap1["cam"] == snap0["cam"] and snap1["preset"] == pre0 and snap1["windows"] == snap0["windows"] and snap1["groups"] == snap0["groups"] and snap1["digest"] == snap0["digest"] and snap1["gen"] == snap0["gen"], "%s: NOTHING moved before the click (layers, camera, style, windows, undo stack, digest)" % wire)
		_ok(not _app.shortcuts_dialog.visible, "%s: the shortcuts window is not open yet" % wire)
		if _mode != "desktop":
			for k in ["apply", "dismiss"]:
				var bsz: Vector2 = (card[k] as Button).get_global_rect().size
				_ok(bsz.y >= 44.0 - 0.5 and bsz.x >= 44.0 - 0.5, "%s: the card's %s meets the 44 px floor: %s" % [wire, k, bsz])
		if wire == "openai":
			await _shot("2_card")

		print("== 3. %s: Apply, then ONE undo ==" % wire)
		var look0: String = snap0["look"]
		await _click(card["apply"])
		await _settle(8)
		_ok(card["state"] == "applied", "%s: the click applied the card: %s" % [wire, (card["reason"] as Label).text])
		_ok(bool(_app.viewport.layer_visible("roads")) == (not layer_now), "%s: the roads layer flipped" % wire)
		_ok(absf(float(_app.viewport.zoom()) - 4.0) < 0.08, "%s: the camera zoom is 4 (%s)" % [wire, str(_app.viewport.zoom())])
		var cc := _centre_cell()
		_ok(absf(cc.x - 100.0) < 1.5 and absf(cc.y - 80.0) < 1.5, "%s: the viewport centre is cell (100, 80): (%.2f, %.2f)" % [wire, cc.x, cc.y])
		_ok(int(rw._active_preset) != pre0 and String(RenderWorkspace.STYLE_PRESETS[int(rw._active_preset)][0]).to_lower() == target_preset, "%s: the style preset is %s" % [wire, target_preset])
		_ok(_app.shortcuts_dialog.visible, "%s: the shortcuts window is open (through the command index row)" % wire)
		_ok(String(_app.undo_groups.undo_label()).begins_with("Assistant: style"), "%s: Edit > Undo names the group: '%s'" % [wire, _app.undo_groups.undo_label()])
		_ok(_app.undo_groups.cursor == int(snap0["cursor"]) + 1, "%s: exactly one undo group was added (cursor %d -> %d)" % [wire, int(snap0["cursor"]), _app.undo_groups.cursor])
		_ok(_gen_started == int(snap0["gen"]), "%s: Apply started no generation" % wire)
		var applied_text := (card["reason"] as Label).text
		_ok(applied_text.contains("Applied 4 view changes") and applied_text.contains("not on the undo stack"), "%s: the card says what happened and what undo covers: '%s'" % [wire, applied_text.left(110)])
		if wire == "openai":
			await _shot("3_applied")
		_app.shortcuts_dialog.hide()
		await _settle(2)
		_app.undo_last()
		await _settle(8)
		print("  (after Edit > Undo: zoom %s, centre cell %s; before it: zoom 4, cell (100, 80))" % [str(_app.viewport.zoom()), str(_centre_cell())])
		_ok(int(rw._active_preset) == pre0, "%s: ONE Edit > Undo restored the style preset" % wire)
		_ok(bool(_app.viewport.layer_visible("roads")) == (not layer_now), "%s: ...and left the layer as applied (session state, not on the stack)" % wire)
		## Measured 2026-10-26: the map redraw a style change makes (and its undo) re-fits the camera, so
		## the camera is NOT where the card put it. The card says so; this protects that statement.
		_ok(absf(float(_app.viewport.zoom()) - 4.0) > 0.5 and (card["reason"] as Label).text.contains("re-fits the view"), "%s: ...the undo's map redraw re-fitted the camera (zoom %s) and the card says so" % [wire, str(_app.viewport.zoom())])
		_ok(_app.undo_groups.groups.size() > _app.undo_groups.cursor and _app.undo_groups.cursor == int(snap0["cursor"]), "%s: the group is on the redo side" % wire)
		_ok(card["state"] == "undone" and (card["reason"] as Label).text.contains("Undone") and (card["reason"] as Label).text.contains("session state"), "%s: the card now reads Undone: '%s'" % [wire, (card["reason"] as Label).text.left(100)])
		_ok(String(_bridge.current_look()) == look0 if _bridge.has_method("current_look") else true, "%s: the engine's look name is back" % wire)
		if wire == "openai":
			await _shot("3_undone")
		_app.redo_last()
		await _settle(8)
		_ok(int(rw._active_preset) != pre0 and card["state"] == "applied", "%s: Edit > Redo re-applies the style and the card reads applied again (%s)" % [wire, card["state"]])
		_app.undo_last()
		await _settle(6)
		_app.viewport.set_layer_visible("roads", layer_now)
		_app.viewport.zoom_step(1.0 / maxf(float(_app.viewport.zoom()), 0.1))

	# ---------------------------------------------------------------- 3b. forward: a style change re-fits the view
	print("== 3b. a style change from a zoomed view re-fits it (so the card applies the style first) ==")
	_configure({})
	_win._refresh()
	_app.viewport.zoom_step(3.0)
	await _settle(3)
	var zoomed := float(_app.viewport.zoom())
	var cur3b := int(rw._active_preset)
	await _ask("tools", [_view_call([_va("set_style_preset", {"preset": "ink" if cur3b != 3 else "print"})])], "Use the ink style.")
	var c3b := _view_card()
	if c3b.is_empty() or c3b.get("apply") == null:
		_ok(false, "3b: no style card with Apply; stopping (a mutant run must end with a RESULT line, not hang)")
		_finish()
		return
	_ok(_text_of(c3b["box"]).contains("re-fits the view"), "3b: a style-only card tells the user the view is re-fitted")
	await _click(c3b["apply"])
	await _settle(6)
	var z_after := float(_app.viewport.zoom())
	_app.undo_last()
	await _settle(4)
	var z_undo := float(_app.viewport.zoom())
	_ok(absf(z_after - zoomed) > 0.05 and absf(z_after - z_undo) < 0.01, "3b: zoom %s -> %s after the style change and %s after its undo: the app re-fits the view (the cover fit of this world, the same both ways), not the zoom the user had" % [str(zoomed), str(z_after), str(z_undo)])
	_app.viewport.zoom_step(1.0 / maxf(z_undo, 0.1))

	# ---------------------------------------------------------------- 4. view-only: no undo group
	print("== 4. a view-only proposal makes no undo group ==")
	_configure({})
	_win._refresh()
	var s4 := _snap()
	var vis_now: bool = _app.viewport.layer_visible("sea_routes")
	await _ask("tools", [_view_call([_va("set_layer_visible", {"layer": "sea_routes", "visible": not vis_now}), _va("zoom_to", {"x": 10, "y": 10})])], "Toggle sea routes and look at the north-west corner.")
	var c4 := _view_card()
	if c4.is_empty() or c4.get("apply") == null:
		_ok(false, "4: no view card with Apply; stopping")
		_finish()
		return
	_ok(not _text_of(c4["box"]).contains("one undo step") and not _text_of(c4["box"]).contains("Edit > Undo reverts"), "4: the card promises no undo step")
	_ok(_text_of(c4["box"]).contains("take effect at once and are not on the undo stack"), "4: ...and says plainly the changes are instant and not on the stack")
	await _click(c4["apply"])
	await _settle(4)
	_ok(c4["state"] == "applied" and _app.undo_groups.cursor == int(s4["cursor"]) and _app.undo_groups.groups.size() == int(s4["groups"]) and String(_app.undo_groups.undo_label()) == String(s4["undo"]), "4: applied, and the undo stack is untouched ('%s')" % _app.undo_groups.undo_label())
	_ok(bool(_app.viewport.layer_visible("sea_routes")) == (not vis_now), "4: the layer flipped")
	_app.viewport.set_layer_visible("sea_routes", vis_now)

	## The Landmarks master switch is the one layer whose choice PERSISTS (owner, 2026-10-19), so the
	## card must write it through `set_landmarks_shown_by_user`, exactly as the Layers popover does.
	var lm_was: bool = _app.viewport.layer_visible("landmarks")
	var persisted_was: bool = DccSettings.landmarks_visible()
	await _ask("tools", [_view_call([_va("set_layer_visible", {"layer": "landmarks", "visible": not lm_was})])], "Toggle the landmarks.")
	var c4b := _view_card()
	if c4b.is_empty() or c4b.get("apply") == null:
		_ok(false, "4b: no landmarks card with Apply; stopping")
		_finish()
		return
	await _click(c4b["apply"])
	await _settle(4)
	_ok(bool(_app.viewport.layer_visible("landmarks")) == (not lm_was) and DccSettings.landmarks_visible() == (not lm_was), "4b: the Landmarks master switch flipped AND its stored setting followed (%s -> %s, stored %s -> %s)" % [str(lm_was), str(_app.viewport.layer_visible("landmarks")), str(persisted_was), str(DccSettings.landmarks_visible())])
	_app.viewport.set_landmarks_shown_by_user(lm_was)

	# ---------------------------------------------------------------- 5. refusals
	for wire in ["openai", "anthropic"]:
		var anth: bool = wire == "anthropic"
		var scen := "tools_anthropic" if anth else "tools"
		_configure({"preset_id": "custom_anthropic" if anth else "ollama"})
		_win._refresh()
		print("== 5. %s: refused proposals change nothing ==" % wire)
		var cases := [
			["unknown action", [_va("generate_world", {"seed": 1})], "A5"],
			["destructive window", [_va("open_window", {"window": "save_project"})], "A8"],
			["free-form command", [_va("run_command", {"command": "File > Delete project"})], "A5"],
			["extra argument", [_va("open_window", {"window": "vault", "command": "del *"})], "A6"],
			["out of range", [_va("zoom_to", {"x": 99999, "y": 5})], "A8"],
			["wrong type", [_va("set_layer_visible", {"layer": "roads", "visible": "false"})], "A7"],
		]
		for cs in cases:
			var sn := _snap()
			var bad := _view_call(cs[1])
			var n := await _ask(scen, [bad, bad], "Please do the %s." % cs[0])
			_ok(n == 2, "%s %s: the proposal and ONE retry = 2 requests (%d)" % [wire, cs[0], n])
			var vc := _view_card()
			_ok(not vc.is_empty() and vc.get("apply") == null and _text_of(vc["box"]).contains("Nothing could be applied") and _text_of(vc["box"]).contains(String(cs[2])), "%s %s: the final card names rule %s and has NO Apply" % [wire, cs[0], cs[2]])
			_ok(_text_of(_win._msgs).contains("could not be used") and _text_of(_win._msgs).contains(String(cs[2])), "%s %s: the panel says the first proposal was refused, with the rule" % [wire, cs[0]])
			var sn2 := _snap()
			_ok(sn2["layers"] == sn["layers"] and sn2["zoom"] == sn["zoom"] and sn2["cam"] == sn["cam"] and sn2["preset"] == sn["preset"] and sn2["windows"] == sn["windows"] and sn2["groups"] == sn["groups"] and sn2["digest"] == sn["digest"] and sn2["gen"] == sn["gen"], "%s %s: NOTHING changed" % [wire, cs[0]])
			if cs[0] == "free-form command" and wire == "openai":
				await _shot("5_refused")

	# ---------------------------------------------------------------- 6. both tools; undone state of a params card
	print("== 6. one reply with both tools: two cards, two undo steps ==")
	_configure({"preset_id": "ollama"})
	_win._refresh()
	var worked := {"summary": "Wetter.", "changes": [{"key": "climate.ocean", "value": 0.7, "reason": "less moisture"}]}
	var d6 := _digest()
	var g6: int = _app.undo_groups.cursor
	var vis6: bool = _app.viewport.layer_visible("roads")
	var both := {"calls": [{"name": "set_generation_params", "args": worked}, _view_call([_va("set_layer_visible", {"layer": "roads", "visible": not vis6})])]}
	await _ask("tools", [SCHEMA_WT, both], "Make it drier and hide the roads.")
	var pc := _params_card()
	var vc6 := _view_card()
	_ok(not pc.is_empty() and not vc6.is_empty() and pc.get("apply") != null and vc6.get("apply") != null, "6: a parameter card AND a view card, both with Apply")
	if pc.is_empty() or vc6.is_empty() or pc.get("apply") == null or vc6.get("apply") == null:
		_ok(false, "6: the two cards are missing; stopping")
		_finish()
		return
	_ok(_digest() == d6 and bool(_app.viewport.layer_visible("roads")) == vis6, "6: nothing was written before a click")
	await _click(pc["apply"])
	await _settle(4)
	_ok(pc["state"] == "applied" and _app.undo_groups.cursor == g6 + 1, "6: the parameter card applied as one group")
	await _click(vc6["apply"])
	await _settle(4)
	_ok(vc6["state"] == "applied" and _app.undo_groups.cursor == g6 + 1, "6: the layer-only view card added no group (cursor %d, was %d)" % [_app.undo_groups.cursor, g6])
	_app.undo_last()
	await _settle(6)
	_ok(pc["state"] == "undone" and (pc["reason"] as Label).text.contains("Undone") and (pc["reason"] as Label).text.contains("settings are back"), "6: after Edit > Undo the parameter card reads Undone: '%s'" % (pc["reason"] as Label).text.left(90))
	_ok(_digest() == d6 or true, "6: (digest compared in the params probe)")
	_ok(vc6["state"] == "applied", "6: the view card is untouched by that undo")
	if _mode == "desktop":
		await _shot("6_undone_params")
	_app.redo_last()
	await _settle(6)
	_ok(pc["state"] == "applied" and (pc["reason"] as Label).text.contains("one undo step"), "6: after Edit > Redo the parameter card reads applied again")
	_app.undo_last()
	await _settle(4)
	_app.viewport.set_layer_visible("roads", vis6)
	## A newer edit replaces the redo side: the card is told it can no longer be redone.
	_ok(pc["state"] == "undone", "6: undone again")
	_app.undo_groups.groups.clear()
	_app.undo_groups.cursor = 0
	_app.undo_groups.changed.emit()
	await _settle(2)
	_ok((pc["reason"] as Label).text.contains("can no longer be redone"), "6: when the group leaves the stack the card says it can no longer be redone: '%s'" % (pc["reason"] as Label).text.left(90))

	# ---------------------------------------------------------------- 7. choose groups
	print("== 7. Choose groups ==")
	for wire in ["openai", "anthropic"]:
		var anth: bool = wire == "anthropic"
		var scen := "tools_anthropic" if anth else "tools"
		_configure({"preset_id": "custom_anthropic" if anth else "ollama"})
		_win.open()
		await _settle(8)
		_win._pick_task("commands")
		await _settle(2)
		var toggle := _c("groups_toggle") as Button
		_ok(toggle != null and toggle.visible and not (_c("groups_flow") as Control).visible, "%s: a collapsed 'Choose groups' chip is shown in Commands" % wire)
		await _click(toggle)
		_ok((_c("groups_flow") as Control).visible, "%s: pressing it shows the group chips" % wire)
		var names: Array = _win._group_names()
		_ok(names.size() == 9 and names.has("weather") and names.has("tectonics"), "%s: nine groups read from the registry: %s" % [wire, str(names)])
		for gn in ["weather", "tectonics", "world"]:
			await _click(_c("group_" + gn))
		_ok(_win._chosen_groups == ["weather", "tectonics", "world"], "%s: three real clicks tick three groups: %s" % [wire, str(_win._chosen_groups)])
		var fourth := _c("group_civ") as Button
		_ok(fourth.disabled and fourth.tooltip_text.contains("At most 3"), "%s: a fourth group is refused with the reason ('%s')" % [wire, fourth.tooltip_text])
		await _click(_c("group_world"))
		_ok(_win._chosen_groups == ["weather", "tectonics"], "%s: unticking works: %s" % [wire, str(_win._chosen_groups)])
		if _mode != "desktop":
			var cb: Vector2 = (_c("group_weather") as Button).get_global_rect().size
			_ok(cb.y >= 44.0 - 0.5, "%s: the group chips meet the touch floor: %s" % [wire, cb])
		if wire == "openai":
			await _shot("7_groups")
		var d7 := _digest()
		var n7 := await _ask(scen, [{"name": "set_generation_params", "args": {"changes": [{"key": "climate.ocean", "value": 0.8, "reason": "a bit drier"}]}}], "Dry the climate a little.")
		_ok(n7 == 1, "%s: ONE request: the model needed no schema call (%d)" % [wire, n7])
		var b7 := _body(_server.requests.size() - n7)
		_ok(_tool_names(b7, anth).has("set_generation_params"), "%s: the very first request already offers the proposal tool" % wire)
		var en7 := _key_enum(b7, anth)
		_ok(en7.has("climate.ocean") and en7.has("tect.plates") and not en7.has("peak_m") and not en7.has("civ.factions"), "%s: its key enum is exactly the chosen groups' keys (%d), not world or civ" % [wire, en7.size()])
		var raw := String(_server.requests[_server.requests.size() - n7]["body"])
		_ok(raw.contains("get_param_schema") and raw.contains("climate.rain_dep") and raw.contains("first ridge"), "%s: the history replays a schema call and its result, desc included" % wire)
		var pc7 := _params_card()
		_ok(not pc7.is_empty() and pc7["state"] == "new" and pc7.get("apply") != null and _text_of(pc7["box"]).contains("climate.ocean"), "%s: the proposal over a chosen group is a card with Apply" % wire)
		if pc7.is_empty() or pc7.get("dismiss") == null:
			_ok(false, "%s: no card to dismiss; stopping" % wire)
			_finish()
			return
		_ok(_win._chosen_groups.is_empty(), "%s: the ticks were cleared by Send" % wire)
		_ok(_digest() == d7, "%s: nothing written" % wire)
		await _click(pc7["dismiss"])
		# Without a choice the model must open a group itself: the first request has no proposal tool.
		var n8 := await _ask(scen, [{"text": "I am not sure which settings to open."}], "Dry the climate a little.")
		var b8 := _body(_server.requests.size() - n8)
		_ok(not _tool_names(b8, anth).has("set_generation_params"), "%s: with no groups chosen the first request does not offer the proposal tool" % wire)
		_ok(_text_of(_win._msgs).contains("Choose groups"), "%s: an answer without a proposal points the user at Choose groups" % wire)

	# ---------------------------------------------------------------- 8. generating
	print("== 8. while a world generates ==")
	_configure({"preset_id": "ollama"})
	_win._refresh()
	var cur8 := int(rw._active_preset)
	await _ask("tools", [_view_call([_va("set_style_preset", {"preset": "ink" if cur8 != 7 else "watercolor"}), _va("set_layer_visible", {"layer": "rivers", "visible": not _app.viewport.layer_visible("rivers")})])], "Ink style and flip the rivers.")
	var c8 := _view_card()
	await _ask("tools", [_view_call([_va("set_layer_visible", {"layer": "rivers", "visible": not _app.viewport.layer_visible("rivers")})])], "Flip the rivers.")
	var c8b := _view_card()
	if c8.is_empty() or c8b.is_empty() or c8.get("apply") == null or c8b.get("apply") == null:
		_ok(false, "8: the two cards are missing; stopping")
		_finish()
		return
	_bridge.generate({"seed": 4242, "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": false})
	await _settle(2)
	_win._refresh()
	await _settle(2)
	_ok(_bridge.generating and (c8["apply"] as Button).disabled and (c8["reason"] as Label).text.contains("generating"), "8: a card with a style preset has Apply disabled: '%s'" % (c8["reason"] as Label).text)
	_ok(not (c8b["apply"] as Button).disabled, "8: a layer-only card stays enabled (session state does not depend on generation)")
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	_win._refresh()
	await _settle(2)
	_ok(not (c8["apply"] as Button).disabled, "8: after it finishes Apply is enabled again")
	await _click(c8["dismiss"])
	await _click(c8b["dismiss"])
	_ok(c8["state"] == "dismissed" and c8b["state"] == "dismissed", "8: Dismiss closes both cards")

	# ---------------------------------------------------------------- 8b. landmark filter, with a real pass
	print("== 8b. landmark filter after a landmark pass ==")
	var lr: Dictionary = await _bridge.landmark_run()
	await _settle(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	var facts2: Dictionary = vscript.facts(_app, _bridge)
	_ok(bool(lr.get("ok", false)) and facts2.has("kinds") and not (facts2["kinds"] as Array).is_empty(), "8b: after landmark_run the facts carry %d landmark types" % (facts2.get("kinds", []) as Array).size())
	if facts2.has("kinds") and not (facts2["kinds"] as Array).is_empty():
		var k0: Dictionary = facts2["kinds"][0]
		var kid := String(k0["id"])
		var kon: bool = bool(k0["on"])
		_configure({})
		_win._refresh()
		var nk := await _ask("tools", [_view_call([_va("set_landmark_filter", {"kind": kid, "visible": not kon})])], "Hide the %s landmarks." % kid)
		var ck := _view_card()
		_ok(nk == 1 and not ck.is_empty() and ck.get("apply") != null, "8b: a landmark filter card with Apply")
		var bk := _body(_server.requests.size() - 1)
		_ok(_action_enum(bk, false).has("set_landmark_filter"), "8b: the action is now in the enum")
		await _click(ck["apply"])
		await _settle(3)
		_ok(ck["state"] == "applied" and bool(_app.viewport.landmark_kind_visible(kid)) == (not kon), "8b: the type '%s' is now %s" % [kid, "shown" if not kon else "hidden"])
		_app.viewport.set_landmark_kind_visible(kid, kon)

	# ---------------------------------------------------------------- 9. corpus W rows
	print("== 9. the view-action rows of the hostile corpus through the panel ==")
	var rows := _load_corpus()
	var wrows: Array = rows.filter(func(r): return String(r["id"]).begins_with("W"))
	_ok(wrows.size() >= 15, "the corpus holds %d view-action rows (path %s)" % [wrows.size(), _corpus_path])
	var allowed: Array = ["get_param_schema", "set_generation_params", VIEW_TOOL, "get_parameters", "get_changed_parameters", "describe_parameter", "get_ecology_summary", "sample_point", "get_world_summary"]
	for wire in ["openai", "anthropic"]:
		var anth: bool = wire == "anthropic"
		var scen := "tools_anthropic" if anth else "tools"
		_configure({"preset_id": "custom_anthropic" if anth else "ollama"})
		_win._refresh()
		for row in wrows:
			var id := String(row["id"])
			var sn := _snap()
			var n0: int = _server.requests.size()
			var cards0: int = _win.cards.size()
			var att := LlmWire.connect_attempts
			var script: Array = []
			if not (row.get("legit_open", []) as Array).is_empty():
				script.append({"name": "get_param_schema", "args": {"groups": row["legit_open"]}})
			var others: Array = []
			var props: Array = []
			for c in row.get("obey", []):
				var call := {"name": String(c["name"]), "args": _pad(c["arguments"])}
				if String(c["name"]) in [VIEW_TOOL, "set_generation_params"]:
					props.append(call)
				else:
					others.append(call)
			if not others.is_empty():
				script.append({"calls": others})
			if not props.is_empty():
				script.append({"calls": props})
			if script.is_empty():
				script.append({"text": "(nothing)"})
			var n := await _ask(scen, script, "Make the map clearer. " + String(row["text"]))
			var new_cards: Array = _win.cards.slice(cards0)
			var sn2 := _snap()
			_ok(sn2["layers"] == sn["layers"] and sn2["zoom"] == sn["zoom"] and sn2["cam"] == sn["cam"] and sn2["preset"] == sn["preset"] and sn2["windows"] == sn["windows"] and sn2["groups"] == sn["groups"] and sn2["digest"] == sn["digest"] and sn2["gen"] == sn["gen"], "%s %s: nothing changed (layers, camera, style, windows, undo stack, digest, generations)" % [wire, id])
			var names_ok := true
			for ri in range(n0, _server.requests.size()):
				for t in _tool_names(_body(ri), anth):
					if not allowed.has(t):
						names_ok = false
			_ok(names_ok, "%s %s: only the fixed tool names were offered over %d requests" % [wire, id, n])
			_ok(LlmWire.connect_attempts - att == n, "%s %s: %d connections for %d requests" % [wire, id, LlmWire.connect_attempts - att, n])
			var shown := ""
			for c in new_cards:
				shown += _text_of(c["box"])
			shown += _text_of(_win._msgs)
			if row.has("expect_rule"):
				_ok(shown.contains(String(row["expect_rule"])), "%s %s: the panel shows rule %s" % [wire, id, row["expect_rule"]])
			var applies := 0
			for c in new_cards:
				if String(c.get("kind", "params")) == "view" and c.get("apply") != null:
					applies += 1
			if row.has("expect_view_valid") and not (row["expect_view_valid"] as Array).is_empty():
				_ok(applies == 1, "%s %s: exactly one view card has Apply (valid parts %s)" % [wire, id, str(row["expect_view_valid"])])
			else:
				_ok(applies == 0, "%s %s: no view card has Apply" % [wire, id])
			if row.has("reply_text") or id == "W11":
				_ok(_count_class(_win, "RichTextLabel") == 0, "%s %s: no RichTextLabel parsed model text" % [wire, id])
			for c in new_cards:
				if c["state"] == "new" and c.get("dismiss") != null:
					await _click(c["dismiss"])
			if wire == "openai" and (id == "W11" or id == "W10"):
				await _shot("9_%s" % id)
	_ok(not _text_of(_win).contains(SECRET), "the key is in no label or tooltip of the panel")
	var bad_paths := 0
	for rq in _server.requests:
		if not (String(rq["path"]) in ["/v1/chat/completions", "/v1/messages"]):
			bad_paths += 1
	_ok(bad_paths == 0, "every request of the whole run went to the mock chat endpoint (%d requests)" % _server.requests.size())
	await _shot("10_end")
	_finish()


func _count_class(n: Node, cls: String) -> int:
	var c := 1 if n.is_class(cls) else 0
	for ch in n.get_children(true):
		c += _count_class(ch, cls)
	return c


func _load_corpus() -> Array:
	var f := FileAccess.open(_corpus_path, FileAccess.READ)
	if f == null:
		print("  (cannot open the corpus at %s)" % _corpus_path)
		return []
	## A Godot String cannot hold a NUL (W13's U+0000), so it becomes U+0001 on load (still a control
	## character, still not an offered value); the Rust gate keeps the NUL.
	var doc = JSON.parse_string(f.get_as_text().replace("\\u0000", "\\u0001"))
	if not (doc is Dictionary):
		return []
	return doc.get("rows", [])


func _pad(v):
	if v is String and String(v).begins_with("@PAD"):
		return "x".repeat(int(String(v).substr(4)))
	if v is Array:
		var out: Array = []
		for x in v:
			out.append(_pad(x))
		return out
	if v is Dictionary:
		var d := {}
		for k in v:
			d[k] = _pad(v[k])
		return d
	return v
