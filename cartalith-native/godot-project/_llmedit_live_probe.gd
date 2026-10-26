extends Node
## **LLM-7 against a REAL local model** (LM Studio, `qwen3-14b`, thinking off): the whole Edit path through
## the real panel -- region, gate, request, model, validator, card, preview -- and, with `--apply 1`, the
## one-click Apply, the field digests and the one-step undo. Not a regression gate (a real model is not
## deterministic); a dated, labelled observation. The regression gate with a scripted server is
## `_llmedit_probe.gd`.
##
##   godot --path . --resolution 1600x1000 res://_llmedit_live_probe.tscn -- \
##       --prompt "Raise the area I painted into a ridge about 300 m high." --region ridge --runs 1 \
##       --apply 1 --model qwen3-14b --port 1234 --out <dir> [--shots <dir>]
##
## `--region ridge` paints a diagonal stroke over inland ground with headroom; `--region coast` paints a
## strip along a real coastline found on a 4-cell lattice (land beside sea); `--region box` boxes a land
## rectangle. Prints one `LIVE_RESULT <json>` line per run (the validated edits, the rejected ones, the
## preview figures, the digests, the timings) and `RESULT: DONE`. Exit 0 when it ran, 2 on a bad argument.
## **Windowed**, from a worktree copy with its own `config/name` (`MISTAKES.md`).

const SEED := 9137
var _prompt := "Raise the area I painted into a ridge about 300 m high."
var _region := "ridge"
var _runs := 1
var _apply := false
var _model := "qwen3-14b"
var _port := 1234
var _out := ""
var _shots := ""
var _app: Node
var _bridge
var _win
const REGION_SCRIPT := preload("res://shell/llm_region.gd")


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _until(cond: Callable, ms: int) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


func _reset_llm() -> void:
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()


func _text_of(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	for ch in n.get_children(true):
		_text_of(ch, out)


func _land(x: int, y: int) -> float:
	var s: Dictionary = _bridge.sample_cell(x, y)
	return float(s.get("elevation_m", -1.0)) if not s.is_empty() else -1.0


## Paints the requested region through the tool's own handlers; returns a one-line description.
func _paint() -> String:
	REGION_SCRIPT.clear(_app)
	REGION_SCRIPT.install(_app)
	REGION_SCRIPT.painting = true
	REGION_SCRIPT.brush = 16.0
	var drag: Callable = _app._drag_handlers[REGION_SCRIPT.TOOL_ID]
	var rel: Callable = _app._release_handlers[REGION_SCRIPT.TOOL_ID]
	var g: Vector2i = _bridge.grid_size()
	var desc := ""
	if _region == "box":
		var best := Vector2i(-1, -1)
		for y in range(20, g.y - 60, 8):
			for x in range(20, g.x - 70, 8):
				if _land(x, y) > 40.0 and _land(x + 50, y + 40) > 40.0 and _land(x + 25, y + 20) > 40.0:
					best = Vector2i(x, y)
					break
			if best.x >= 0:
				break
		_bridge.region_set(float(best.x), float(best.y), 50.0, 40.0)
		REGION_SCRIPT.painting = false
		REGION_SCRIPT.use_box(_app, _bridge)
		return "box 50 x 40 at (%d, %d)" % [best.x, best.y]
	if _region == "coast":
		var found := Vector2i(-1, -1)
		for y in range(30, g.y - 30, 4):
			for x in range(30, g.x - 30, 4):
				if _land(x, y) >= 0.0 and _land(x + 4, y) < 0.0 and _land(x - 4, y) >= 0.0 and _land(x, y - 12) >= -200.0:
					found = Vector2i(x, y)
					break
			if found.x >= 0:
				break
		if found.x < 0:
			print("ABORT: no coastline found on the 4-cell lattice of this world")
			get_tree().quit(2)
			return "no site"
		for k in range(-3, 4):
			drag.call(float(found.x + 2), float(found.y + k * 9))
		rel.call(float(found.x), float(found.y), true)
		desc = "coast strip around (%d, %d)" % [found.x, found.y]
	else:
		## Inland ground: five samples along the stroke, all land (10 m up to 3 000 m). The first match in scan order.
		var start := Vector2i(-1, -1)
		for y in range(24, g.y - 50, 4):
			for x in range(24, g.x - 80, 4):
				var ok := true
				for k in 5:
					var e := _land(x + k * 14, y + k * 7)
					if e < 10.0 or e > 3000.0:
						ok = false
						break
				if ok:
					start = Vector2i(x, y)
					break
			if start.x >= 0:
				break
		if start.x < 0:
			print("ABORT: no inland stroke site found on this world")
			get_tree().quit(2)
			return "no site"
		for k in 12:
			drag.call(float(start.x + k * 6), float(start.y + k * 3))
		rel.call(float(start.x + 66), float(start.y + 33), true)
		desc = "diagonal stroke from (%d, %d)" % [start.x, start.y]
	REGION_SCRIPT.painting = false
	return desc


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a == "--prompt" and i + 1 < args.size():
			i += 1
			_prompt = args[i]
		elif a == "--region" and i + 1 < args.size():
			i += 1
			_region = args[i]
		elif a == "--runs" and i + 1 < args.size():
			i += 1
			_runs = int(args[i])
		elif a == "--apply" and i + 1 < args.size():
			i += 1
			_apply = args[i] == "1"
		elif a == "--model" and i + 1 < args.size():
			i += 1
			_model = args[i]
		elif a == "--port" and i + 1 < args.size():
			i += 1
			_port = int(args[i])
		elif a == "--out" and i + 1 < args.size():
			i += 1
			_out = args[i]
		elif a == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		i += 1
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed")
		get_tree().quit(2)
		return
	get_tree().create_timer(2400.0).timeout.connect(func() -> void:
		print("WATCHDOG")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	if _out != "":
		DirAccess.make_dir_recursive_absolute(_out)
	_reset_llm()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_bridge = _app.bridge
	_win = _app.assistant_window
	_bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": false})
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	var origin := "http://127.0.0.1:%d" % _port
	for kv in {"enabled": true, "edit": true, "ctx_region": true, "local_allowed": true, "preset_id": "lm_studio",
			"base_url": origin + "/v1", "model": _model, "thinking": "none", "key_source": "env"}.keys():
		pass
	var cfg := {"enabled": true, "edit": true, "ctx_region": true, "local_allowed": true, "preset_id": "lm_studio",
		"base_url": origin + "/v1", "model": _model, "thinking": "none", "key_source": "env"}
	for k in cfg:
		DccSettings.set_llm_value(k, cfg[k])
	DccSettings.llm_add_consent(origin)
	_win.open()
	await _settle(10)
	_win._pick_task("edit")
	var painted := _paint()
	print("REGION %s" % painted)
	_win._edit_key = ""
	for run_i in _runs:
		var d_before: Dictionary = _bridge.edit_state_digest()
		var inp: TextEdit = _win.controls["input"]
		inp.text = _prompt
		inp.text_changed.emit()
		await _settle(3)
		var c0: int = _win.cards.size()
		var t0 := Time.get_ticks_msec()
		_win._on_send()
		await _settle(6)
		var done: bool = await _until(func(): return not _win.run.busy(), 300000)
		await _settle(10)
		var ms := Time.get_ticks_msec() - t0
		var result := {"run": run_i + 1, "model": _model, "prompt": _prompt, "region": painted, "ms": ms, "finished": done,
			"cards": _win.cards.size() - c0, "usage": _win.run.last_usage, "first_token_ms": _win.run.first_token_ms}
		var texts: Array = []
		_text_of(_win._msgs, texts)
		result["panel_tail"] = "\n".join(texts.slice(maxi(0, texts.size() - 24)))
		if _win.cards.size() > c0:
			var card: Dictionary = _win.cards[_win.cards.size() - 1]
			var info: Dictionary = card["info"]
			var valid: Array = []
			for v in info.get("valid", []):
				var vd: Dictionary = v
				valid.append({"text": String(vd["text"]), "operation": vd["operation"], "amount_m": vd.get("amount_m"), "strength": vd.get("strength"), "passes": vd.get("passes"), "falloff": vd["falloff"], "softness": vd["softness"], "defaulted": vd["defaulted"], "reason": vd.get("reason", "")})
			result["valid"] = valid
			result["rejected"] = info.get("rejected", [])
			result["structural"] = info.get("structural", "")
			result["summary"] = info.get("summary", "")
			var p: Dictionary = card.get("preview", {})
			if bool(p.get("ok", false)):
				result["preview"] = {"changed": p["changed"], "max_up_m": p["max_up_m"], "max_down_m": p["max_down_m"], "became_sea": p["became_sea"], "became_land": p["became_land"], "capped": p["capped"], "floored": p["floored"], "before": p["before"], "after": p["after"]}
			result["apply_enabled"] = card["apply"] != null and not (card["apply"] as Button).disabled
			if _shots != "":
				await _settle(4)
				get_viewport().get_texture().get_image().save_png(_shots.path_join("llmedit_live_%s_%d.png" % [_region, run_i + 1]))
			if _apply and card["apply"] != null and not (card["apply"] as Button).disabled:
				var g0: int = _app.undo_groups.cursor
				_win._apply_edit_card(card)
				await _settle(10)
				var d_after: Dictionary = _bridge.edit_state_digest()
				result["applied"] = card["state"] == "applied"
				result["field_changed"] = d_after["field"] != d_before["field"]
				result["undo_groups_added"] = _app.undo_groups.cursor - g0
				_app.undo_last()
				await _settle(10)
				result["undo_restores_field"] = _bridge.edit_state_digest()["field"] == d_before["field"]
				if _shots != "":
					await _settle(2)
				_app.redo_last()
				await _settle(6)
				_app.undo_last()
				await _settle(6)
		else:
			result["answer_or_failure"] = result["panel_tail"]
		if not _win.run.busy():
			print("LIVE_RESULT %s" % JSON.stringify(result))
		if _out != "":
			var f := FileAccess.open(_out.path_join("run_%s_%d.json" % [_region, run_i + 1]), FileAccess.WRITE)
			f.store_string(JSON.stringify(result, "  "))
			f.close()
	LlmWire.shutdown_all()
	print("RESULT: DONE")
	get_tree().quit(0)
