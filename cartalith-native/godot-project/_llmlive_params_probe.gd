extends Node
## **LLM-6c stage 2, live: one model arm through the REAL assistant panel, on a real world.**
## (`LLM_ASSISTANT_SCOPE.md` s.5b.7 "Direction quality of real models"; Rulings DD, DF, DI.)
## Probe-only and NOT part of any suite: it needs a model server (or a hosted key) that is
## already running, and it never starts, downloads or stops anything itself.
##
##   godot --path . --resolution 1600x1000 res://_llmlive_params_probe.tscn -- \
##       --arm A --preset ollama --base-url http://127.0.0.1:11434/v1 --model qwen3:14b \
##       --key-source env --seed 71077345 --extra-seed 24601 --shots <dir> --out <file.jsonl>
##
## **Windowed, never `--headless`** (it clicks real buttons, regenerates and saves frames), from a
## project copy with its own `config/name` and a wiped user dir (`MISTAKES.md`: Godot 4.7 has no
## `--user-data-dir`). Reads exactly the flags below; anything else exits 2.
##
## WHAT IT DOES, per arm (the same prompts, seed and order for every arm):
##   1. Configures the assistant through the app's OWN settings code (`DccSettings.set_llm_value`,
##      `llm_add_consent`) -- master switch, Commands, "Send: generation parameters" ticked, the
##      preset, base URL and model -- then runs `LlmGpuRule.sync` exactly as the settings window
##      does on a change, and records the plan (the "detected as" line, whether it switched) and
##      the compute adapter selection before and after.
##   2. Generates the seed at the New World dialog's DEFAULT size through
##      `NewWorldDialog.generate_checked()` (the path every user Generate takes), records the
##      before-map (fit view and a x4 zoom about the map centre) and world statistics.
##   3. For each prompt: types it into the panel, presses the real Send button, waits for the run
##      to end, screenshots the card, records the verdicts and changes, the per-turn timing and
##      the token usage the server reported; when there is an Apply button it CLICKS it, hides the
##      panel, regenerates (user path), records the after-map and statistics, presses Undo ONCE
##      (`app.undo_last()`) and checks the parameter digest is back to the pre-Apply value; then
##      regenerates the base world and checks the WHOLE edit-state digest equals the pre-Apply one.
##   4. With `--extra-seed`, repeats ONE prompt (`EXTRA_PROMPT`) on that seed.
##
## WHAT IT MUST NEVER DO: read, print or write a key (the key, when one is used, is resolved by
## the app's own keystore path from `key_source`); write anything but its JSON lines and images;
## send a map image to any model (Ruling DI P5 -- the images are for the report only).
##
## Statistics come from `WorldGen.sample_cell` on a regular lattice (every `STRIDE`-th cell in
## both axes), because no binding returns the height field or the volcano table whole: land
## fraction (drawn water classification), mean land elevation in metres, and DISTINCT volcano
## edifices hit by the lattice (a `(kind, age_years)` pair identifies one edifice). The volcano
## figure is therefore a lower bound: an edifice smaller than the lattice spacing can be missed.
##
## Output: one `LIVE <json>` line per record and the same JSON as one line in `--out`;
## `RESULT: DONE arm=<a> records=<n> auth_stop=<bool>`. Exit 0 finished, 1 could not run,
## 2 bad argument or watchdog.

## The requests, verbatim from the owner's brief, in this order for every arm.
const PROMPTS := [
	["no_sea", "I don't want any sea, change the map"],
	["more_volcanoes", "I want more volcanoes"],
	["drier_lower", "drier climate, lower mountains"],
	["islands", "make it look like a chain of islands"],
	["imp_river", "add a river here"],
]
## The one request repeated on `--extra-seed` (labelled judgement: the open-ended request whose
## outcome depends most on the world, so a second seed says the most).
const EXTRA_PROMPT := ["islands", "make it look like a chain of islands"]
## Lattice spacing for the statistics, in cells. Labelled judgement: at the default 2048-column
## grid this is about 168 000 samples (a few seconds), and 4 cells is about 2.3 km at 1200 km.
const STRIDE := 4
## Longest wait for one model request, ms (a thinking 27B model on 1024 tokens at about 26 tok/s
## needs well under this; it is a hang guard, not an expectation).
const ASK_TIMEOUT_MS := 600000

var _o := {"arm": "", "preset": "", "base-url": "", "model": "", "key-source": "env", "seed": "71077345",
	"extra-seed": "", "shots": "", "out": "", "watchdog": "5400", "prompts": ""}
var _app: Node
var _bridge
var _win
var _records := 0
var _auth_stop := false
var _gen_started := 0
## Per-request event log, filled by the run's signals.
var _ev: Array = []
var _t_send := 0


func _log(rec: Dictionary) -> void:
	rec["arm"] = String(_o["arm"])
	var text := JSON.stringify(rec)
	print("LIVE " + text)
	_records += 1
	var path := String(_o["out"])
	if path != "":
		var f := FileAccess.open(path, FileAccess.READ_WRITE if FileAccess.file_exists(path) else FileAccess.WRITE)
		if f != null:
			f.seek_end()
			f.store_line(text)
			f.close()


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


## Waits for the map to be drawn: frames, a pause, then the overlay's background builds
## (the `_ap5_probe` settle), and one drawn frame.
func _settle_map() -> void:
	await _frames(8)
	await get_tree().create_timer(1.5).timeout
	var ov: Control = _app.viewport.overlay
	var guard := 0
	while ov != null and ov.has_method("pending_builds") and int(ov.call("pending_builds")) > 0 and guard < 400:
		await _frames(2)
		guard += 1
	await _frames(4)
	await RenderingServer.frame_post_draw


func _until(cond: Callable, ms: int) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


## A REAL mouse press and release at the centre of `c` (the `_llmparams_probe` helper).
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
	await _frames(2)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _frames(2)
	await _frames(3)


func _c(name: String) -> Control:
	return _win.controls.get(name) as Control


func _all_text(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	for ch in n.get_children(true):
		_all_text(ch, out)


func _text_of(n: Node) -> String:
	var parts: Array = []
	_all_text(n, parts)
	return "\n".join(parts)


func _jpg(img: Image, name: String) -> String:
	if String(_o["shots"]) == "":
		return ""
	var path := String(_o["shots"]).path_join("%s_%s.jpg" % [String(_o["arm"]), name])
	img.save_jpg(path, 0.85)
	return path


## The whole window as drawn (the embedded assistant window included).
func _frame() -> Image:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	img.convert(Image.FORMAT_RGB8)
	return img


## The map at fit view and at x4 about the map centre, each cropped to the map's rect.
func _map_shots(tag: String) -> Dictionary:
	var host: Control = _app.viewport
	host.reset_view()
	await _settle_map()
	var rect := Rect2i(host.get_global_rect())
	var full := await _frame()
	rect = rect.intersection(Rect2i(Vector2i.ZERO, full.get_size()))
	var out := {"x1": _jpg(full.get_region(rect), tag + "_x1")}
	host._zoom_at(host.size * 0.5, 4.0)
	await _settle_map()
	var z := await _frame()
	out["x4"] = _jpg(z.get_region(rect), tag + "_x4")
	host.reset_view()
	await _settle_map()
	return out


## World statistics from the lattice (see the header for what each figure means).
func _stats() -> Dictionary:
	var w: int = _bridge.world_gen.get_width() if _bridge.world_gen.has_method("get_width") else 0
	var h: int = _bridge.world_gen.get_height() if _bridge.world_gen.has_method("get_height") else 0
	if w <= 0 or h <= 0:
		return {"error": "no grid size readable"}
	var t0 := Time.get_ticks_msec()
	var n := 0
	var land := 0
	var lake := 0
	var elev_sum := 0.0
	var elev_max := -1.0e9
	var volc := {}
	var volc_cells := 0
	for y in range(STRIDE / 2, h, STRIDE):
		for x in range(STRIDE / 2, w, STRIDE):
			var d: Dictionary = _bridge.world_gen.sample_cell(x, y)
			if d.is_empty():
				continue
			n += 1
			var water := String(d.get("water", "land"))
			if water == "land":
				land += 1
				var e := float(d.get("elevation_m", 0.0))
				elev_sum += e
				elev_max = maxf(elev_max, e)
			elif water == "lake":
				lake += 1
			if d.has("volcano_age_years"):
				volc_cells += 1
				volc["%s|%s" % [String(d.get("volcano_kind", "?")), str(d["volcano_age_years"])]] = true
	var s := {"grid": [w, h], "samples": n, "stride": STRIDE, "ms": Time.get_ticks_msec() - t0}
	if n > 0:
		s["land_fraction"] = float(land) / float(n)
		s["lake_fraction"] = float(lake) / float(n)
		s["volcano_cell_fraction"] = float(volc_cells) / float(n)
		s["volcanoes_hit"] = volc.size()
	if land > 0:
		s["mean_land_elev_m"] = elev_sum / float(land)
		s["max_land_elev_m"] = elev_max
	return s


func _digest() -> Dictionary:
	return (_bridge.edit_state_digest() as Dictionary).duplicate()


## Generates at the New World dialog's current form (the user's Generate path) and waits.
func _generate() -> Dictionary:
	var g0 := _gen_started
	var t0 := Time.get_ticks_msec()
	_app.new_world_dialog.generate_checked()
	await _until(func(): return _gen_started > g0, 10000)
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	var ms := Time.get_ticks_msec() - t0
	await _frames(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	return {"generate_ms": ms, "backend": str(_bridge.gpu_last_backend()), "device_usage": _bridge.gpu_last_device_usage(),
		"stages_on_gpu": _bridge.gpu_stages_used()}


func _set_seed(s: int) -> void:
	var nwd = _app.new_world_dialog
	nwd.seed_input.set_value_no_signal(s)
	nwd._seed_dirty = false


func _configure() -> Dictionary:
	var preset := String(_o["preset"])
	var base := String(_o["base-url"])
	var ep: Dictionary = _bridge.llm_endpoint_info(preset, base)
	var local := bool(ep.get("is_local", false))
	var s := {"enabled": true, "commands": true, "prose": false, "edit": false,
		"local_allowed": local, "remote_allowed": not local, "preset_id": preset, "base_url": base,
		"model": String(_o["model"]), "key_source": String(_o["key-source"]), "ctx_params": true,
		"gpu_override": "automatic"}
	for k in s:
		if not DccSettings.set_llm_value(k, s[k]):
			return {"error": "set_llm_value refused %s" % k}
	DccSettings.llm_add_consent(String(ep.get("origin", "")))
	var before: PackedStringArray = _bridge.gpu_selected_devices()
	var plan: Dictionary = LlmGpuRule.sync(_bridge)
	return {"endpoint": ep, "gpu_plan": plan, "gpu_rule_applied": LlmGpuRule.is_applied(),
		"selected_before": before, "selected_after": _bridge.gpu_selected_devices(), "devices": _bridge.gpu_devices()}


func _hook_run() -> void:
	var run: LlmRun = _win.run
	run.token.connect(func(_t: String): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": "token"}))
	run.tool_called.connect(func(i: Dictionary): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": "tool_call", "name": String(i.get("name", "")), "arguments": String(i.get("arguments", ""))}))
	run.params_step.connect(func(i: Dictionary): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": String(i.get("type", "step")), "groups": Array(i.get("groups", PackedStringArray()))}))
	run.usage.connect(func(i: Dictionary): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": "usage", "u": i}))
	run.proposal.connect(func(_i: Dictionary): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": "proposal"}))
	run.answered.connect(func(t: String): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": "answered", "text": t}))
	run.failed.connect(func(m: String): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": "failed", "message": m}))
	run.retrying.connect(func(m: String): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": "retrying", "message": m}))
	run.idle.connect(func(): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": "idle"}))


## A failure message that means the credential was refused: the brief says stop the arm, no retry.
static func _is_auth_failure(m: String) -> bool:
	var l := m.to_lower()
	return l.contains("401") or l.contains("403") or l.contains("authentic") or l.contains("api key") \
		or l.contains("invalid x-api-key") or l.contains("permission")


func _ask(id: String, text: String, seed_: int) -> void:
	_win.open()
	await _frames(10)
	_win._pick_task("commands")
	var inp := _c("input") as TextEdit
	inp.text = text
	inp.text_changed.emit()
	await _frames(3)
	var send := _c("send") as Button
	var rec := {"kind": "request", "id": id, "request": text, "seed": seed_, "send_disabled": send.disabled,
		"send_reason": (_c("reason") as Label).text if _c("reason") != null else ""}
	var cards0: int = _win.cards.size()
	var params_before: Dictionary = _bridge.param_snapshot()
	var d_before := _digest()
	_ev = []
	_t_send = Time.get_ticks_msec()
	await _click(send)
	await _until(func(): return _win.run.busy(), 5000)
	var finished := await _until(func(): return not _win.run.busy(), ASK_TIMEOUT_MS)
	await _frames(6)
	rec["finished"] = finished
	rec["events"] = _ev.duplicate(true)
	rec["model_text"] = _win.run.text_so_far
	## Timing: first output event (a streamed word or a decoded tool call) and the end.
	var first := -1
	var end := -1
	var prompt_tok := 0
	var compl_tok := 0
	var usage_n := 0
	for e in _ev:
		if first < 0 and String(e["e"]) in ["token", "tool_call"]:
			first = int(e["t"])
		if String(e["e"]) in ["proposal", "answered", "failed"]:
			end = int(e["t"])
		if String(e["e"]) == "usage":
			usage_n += 1
			prompt_tok += int((e["u"] as Dictionary).get("prompt", 0))
			compl_tok += int((e["u"] as Dictionary).get("completion", 0))
		if String(e["e"]) == "failed" and _is_auth_failure(String(e.get("message", ""))):
			_auth_stop = true
	if first >= 0:
		rec["first_output_ms"] = first
	if end >= 0:
		rec["total_ms"] = end
	if usage_n > 0:
		rec["usage_turns"] = usage_n
		rec["prompt_tokens"] = prompt_tok
		rec["completion_tokens"] = compl_tok
	rec["panel_text"] = _text_of(_win._msgs).right(1500)
	var card: Dictionary = _win.cards[_win.cards.size() - 1] if _win.cards.size() > cards0 else {}
	if not card.is_empty():
		rec["card"] = card["info"]
		rec["card_text"] = _text_of(card["box"])
	rec["card_shot"] = _jpg(await _frame(), "%s_s%d_card" % [id, seed_])
	if card.is_empty() or card.get("apply") == null:
		rec["outcome"] = "no_apply"
		_log(rec)
		return
	## Apply (a real click), then the user's own regenerate.
	await _click(card["apply"])
	await _frames(6)
	rec["apply_state"] = String(card["state"])
	var landed := {}
	for c in (card["info"] as Dictionary).get("valid", []):
		landed[String(c["key"])] = _bridge.param_get(String(c["key"]))
	rec["landed"] = landed
	rec["undo_label"] = String(_app.undo_groups.undo_label())
	_win.hide()
	await _frames(4)
	rec["regen"] = await _generate()
	rec["stats_after"] = _stats()
	rec["map_after"] = await _map_shots("%s_s%d_after" % [id, seed_])
	## ONE undo, then the settings must be the pre-Apply ones.
	_app.undo_last()
	await _frames(6)
	var d_undo := _digest()
	rec["undo_params_equal"] = String(d_undo.get("params", "")) == String(d_before.get("params", ""))
	var snap_after: Dictionary = _bridge.param_snapshot()
	var diff := []
	for k in params_before:
		if str(params_before[k]) != str(snap_after.get(k)):
			diff.append(k)
	rec["undo_param_keys_differing"] = diff
	## Back to the base world: the whole digest must match (undo plus a deterministic generate).
	rec["regen_base"] = await _generate()
	var d_base := _digest()
	rec["base_digest_equal"] = d_base == d_before
	if d_base != d_before:
		var keys := []
		for k in d_before:
			if str(d_before[k]) != str(d_base.get(k)):
				keys.append(k)
		rec["base_digest_keys_differing"] = keys
	rec["outcome"] = "applied"
	_log(rec)


func _base(seed_: int) -> void:
	_set_seed(seed_)
	var g := await _generate()
	var rec := {"kind": "base", "seed": seed_, "regen": g, "grid_request": _app.new_world_dialog.request(),
		"digest": _digest(), "stats": _stats()}
	rec["map"] = await _map_shots("base_s%d" % seed_)
	## `request()` holds Variant floats and ints only; drop nothing.
	_log(rec)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if not a.begins_with("--") or not _o.has(a.substr(2)) or i + 1 >= args.size():
			print("ABORT: unrecognised argument %s; this probe reads --%s" % [a, ", --".join(_o.keys())])
			get_tree().quit(2)
			return
		_o[a.substr(2)] = String(args[i + 1])
		i += 2
	for need in ["arm", "preset", "base-url", "model"]:
		if String(_o[need]) == "":
			print("ABORT: --%s is required" % need)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed; this probe clicks real buttons and saves frames")
		get_tree().quit(2)
		return
	get_tree().create_timer(float(_o["watchdog"])).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		LlmWire.shutdown_all()
		get_tree().quit(2))
	if String(_o["shots"]) != "":
		DirAccess.make_dir_recursive_absolute(String(_o["shots"]))
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.5).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)
	_bridge = _app.bridge
	_win = _app.assistant_window
	if not _bridge.llm_api or _win == null:
		print("RESULT: CANNOT RUN (no LlmBridge or no assistant panel in this build)")
		get_tree().quit(1)
		return
	_bridge.generation_started.connect(func(): _gen_started += 1)
	var cfg := _configure()
	cfg["kind"] = "config"
	cfg["model"] = String(_o["model"])
	cfg["preset"] = String(_o["preset"])
	cfg["dll_note"] = "see the run log for the dll mtime"
	_log(cfg)
	if cfg.has("error"):
		get_tree().quit(1)
		return
	_hook_run()
	var only := String(_o["prompts"]).split(",", false)
	var seed_main := int(_o["seed"])
	await _base(seed_main)
	for p in PROMPTS:
		if _auth_stop:
			break
		if not only.is_empty() and not only.has(String(p[0])):
			continue
		await _ask(String(p[0]), String(p[1]), seed_main)
	if String(_o["extra-seed"]) != "" and not _auth_stop:
		var s2 := int(_o["extra-seed"])
		await _base(s2)
		await _ask(String(EXTRA_PROMPT[0]), String(EXTRA_PROMPT[1]), s2)
	LlmWire.shutdown_all()
	## Put the user's own compute selection back, as deselecting the provider would.
	DccSettings.set_llm_value("enabled", false)
	LlmGpuRule.sync(_bridge)
	print("RESULT: DONE arm=%s records=%d auth_stop=%s" % [String(_o["arm"]), _records, str(_auth_stop)])
	get_tree().quit(0)
