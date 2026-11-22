extends Node
## **Live assistant runs against the owner's LM Studio, through the real panel** (`LLM_ASSISTANT_SCOPE.md` s.9,
## 2026-11-17; owner permission recorded in `STATUS.md`). One scenario per launch, so a hang or a crash costs one scenario:
##
##   godot --path . --resolution 1700x1000 res://_llmlive_run_probe.tscn -- --scenario qa --model qwen3.8-27b \
##       --port 1235 --vault <scratch dir> --out <dir> [--shots <dir>]
##
## `--port` is the **recording proxy** (`asst_proxy.py`, which forwards to LM Studio on 1234 and logs every request body and
## response), so the bodies inspected are the bytes the app really sent. Scenarios: `qa` (Questions with the two search
## boxes ticked; citations; a private-note canary), `params` (a Commands proposal, Apply with the rebuild, Undo and
## regenerate), `edit_ridge`, `edit_mesa`, `edit_paint` (Ruling FW, a painted region), `region_tick` (the profile's
## region tick on, then off). Everything goes through the panel's own controls (input text, the Send and card buttons'
## `pressed`), never through the engine directly, except where a world must exist (the probe generates it, as the other
## probes do). The probe writes only into `--out`, `--vault` and the scratch user dir; it never reads a key (LM Studio takes
## none), never clicks Save, and sends no image. **Windowed**, from a worktree with a wiped user dir.
## Result: `RESULT: DONE scenario=<s>` and `result_<scenario>.json` in `--out`. Exit 0 ran, 2 bad argument.

const REGION_SCRIPT := preload("res://shell/llm_region.gd")
const SEED := 9137
const CANARY := "ZEBRAPRIVATE7731"

var _o := {"scenario": "qa", "model": "qwen3.8-27b", "port": "1235", "vault": "", "out": "", "shots": "", "ctx": "8192", "tag": "", "revise": "0"}
var _app: Node
var _bridge
var _win
var _rec := {"events": []}
var _t0 := 0


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _until(cond: Callable, ms: int) -> bool:
	var t := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


func _log(what: String, extra: Dictionary = {}) -> void:
	var e := {"t_ms": Time.get_ticks_msec() - _t0, "what": what}
	e.merge(extra)
	_rec["events"].append(e)
	print("EVENT ", JSON.stringify(e).left(600))


func _texts(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append("[" + (n as Button).text + "]")
	for ch in n.get_children(true):
		_texts(ch, out)


func _transcript() -> String:
	var a: Array = []
	_texts(_win._msgs, a)
	return "\n".join(a.filter(func(s): return String(s).strip_edges() != ""))


func _shot(tag: String) -> void:
	if _o["shots"] == "":
		return
	await _settle(4)
	get_viewport().get_texture().get_image().save_png(String(_o["shots"]).path_join("live_%s_%s.png" % [_o["scenario"], tag]))


func _put(rel: String, text: String) -> void:
	var p := String(_o["vault"]).path_join(rel)
	DirAccess.make_dir_recursive_absolute(p.get_base_dir())
	var f := FileAccess.open(p, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _generate(seed_: int, villages: bool) -> void:
	_bridge.generate({"seed": seed_, "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": villages})
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()


func _configure(over: Dictionary) -> void:
	var origin := "http://127.0.0.1:%s" % _o["port"]
	var s := {"enabled": true, "local_allowed": true, "preset_id": "lm_studio", "base_url": origin + "/v1", "model": _o["model"],
		"thinking": "none", "key_source": "env", "context_tokens": int(_o["ctx"]), "regen_on_apply": true, "conv_save_asked": true}
	s.merge(over, true)
	for k in s:
		DccSettings.set_llm_value(k, s[k])
	DccSettings.llm_add_consent(origin)


## Types `text` into the panel's own input and presses its Send button; waits for the run to end. Returns ms.
func _send(text: String, task: String) -> int:
	_win._pick_task(task)
	await _settle(4)
	var inp: TextEdit = _win.controls["input"]
	inp.text = text
	inp.text_changed.emit()
	await _settle(4)
	var send: Button = _win.controls["send"]
	if send.disabled:
		_log("send_disabled", {"reason": String((_win.controls["reason"] as Label).text)})
		return -1
	var t := Time.get_ticks_msec()
	send.pressed.emit()
	await _settle(8)
	await _until(func(): return not _win.run.busy(), 600000)
	await _settle(10)
	return Time.get_ticks_msec() - t


func _card(kind: String) -> Dictionary:
	for i in range(_win.cards.size() - 1, -1, -1):
		if String(_win.cards[i].get("kind", "params")) == kind:
			return _win.cards[i]
	return {}


func _usage() -> Dictionary:
	return {"usage": _win.run.last_usage, "first_token_ms": _win.run.first_token_ms}


func _pick_place() -> Dictionary:
	var rows: Array = _bridge.world_gen.get_settlements()
	var best: Dictionary = {}
	for r in rows:
		if best.is_empty() or int(r["population"]) > int(best["population"]):
			best = r
	return best


func _paint_ridge() -> String:
	REGION_SCRIPT.clear(_app)
	REGION_SCRIPT.install(_app)
	REGION_SCRIPT.painting = true
	REGION_SCRIPT.brush = 16.0
	var drag: Callable = _app._drag_handlers[REGION_SCRIPT.TOOL_ID]
	var rel: Callable = _app._release_handlers[REGION_SCRIPT.TOOL_ID]
	var g: Vector2i = _bridge.grid_size()
	var start := Vector2i(-1, -1)
	for y in range(24, g.y - 50, 4):
		for x in range(24, g.x - 80, 4):
			var ok := true
			for k in 5:
				var s: Dictionary = _bridge.sample_cell(x + k * 14, y + k * 7)
				var e := float(s.get("elevation_m", -1.0))
				if e < 10.0 or e > 3000.0:
					ok = false
					break
			if ok:
				start = Vector2i(x, y)
				break
		if start.x >= 0:
			break
	if start.x < 0:
		return "no site"
	for k in 12:
		drag.call(float(start.x + k * 6), float(start.y + k * 3))
	rel.call(float(start.x + 66), float(start.y + 33), true)
	REGION_SCRIPT.painting = false
	return "diagonal stroke from (%d, %d)" % [start.x, start.y]


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a.begins_with("--") and _o.has(a.substr(2)) and i + 1 < args.size():
			i += 1
			_o[a.substr(2)] = args[i]
		elif a == "--force-touch" or a.begins_with("--density"):
			pass
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		i += 1
	if DisplayServer.get_name() == "headless" or _o["out"] == "" or _o["vault"] == "":
		print("ABORT: run windowed with --out and --vault")
		get_tree().quit(2)
		return
	get_tree().create_timer(2400.0).timeout.connect(func() -> void:
		print("WATCHDOG")
		get_tree().quit(2))
	DirAccess.make_dir_recursive_absolute(_o["out"])
	if _o["shots"] != "":
		DirAccess.make_dir_recursive_absolute(_o["shots"])
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_bridge = _app.bridge
	_win = _app.assistant_window
	_t0 = Time.get_ticks_msec()
	_rec["scenario"] = _o["scenario"]
	_rec["model"] = _o["model"]
	_rec["guard_window_tokens"] = int(_o["ctx"])
	await _generate(SEED, _o["scenario"] == "qa")
	var place := _pick_place()
	_rec["world"] = {"seed": SEED, "grid": str(_bridge.grid_size())}
	match String(_o["scenario"]):
		"qa":
			await _scenario_qa(place)
		"params":
			await _scenario_params()
		"edit_ridge":
			await _scenario_edit("Raise the area I painted into a ridge about 300 m high.")
		"edit_mesa":
			await _scenario_edit("Turn the area I painted into a mesa, a flat-topped plateau with steep sides, about 250 m high.")
		"edit_paint":
			await _scenario_edit("Paint the area I painted red.")
		"edit_raise":
			await _scenario_edit("Raise the area I painted by 150 m.")
		"region_tick":
			await _scenario_region_tick()
		_:
			print("ABORT: unknown scenario")
			get_tree().quit(2)
			return
	_rec["panel_final"] = _transcript()
	var f := FileAccess.open(String(_o["out"]).path_join("result_%s%s.json" % [_o["scenario"], _o["tag"]]), FileAccess.WRITE)
	f.store_string(JSON.stringify(_rec, "  "))
	f.close()
	LlmWire.shutdown_all()
	print("RESULT: DONE scenario=%s" % _o["scenario"])
	get_tree().quit(0)


func _scenario_qa(place: Dictionary) -> void:
	var nm := String(place["name"])
	_rec["place"] = {"name": nm, "population": int(place["population"])}
	DirAccess.make_dir_recursive_absolute(_o["vault"])
	_put("Places/%s.md" % nm, "# %s\nThe harbour of %s was walled in 1204 by the Order of Dawn.\nA chronicler counted %d people.\n" % [nm, nm, int(place["population"]) + 2222])
	_put("Places/Hidden.md", "---\nprivate: true\n---\n# Hidden\n%s a secret about %s: the true heir lives in the cellar.\n" % [CANARY, nm])
	var vc: Dictionary = _bridge.vault_connect(String(_o["vault"]), "live-run scratch vault")
	_log("vault_connect", {"ok": bool(vc.get("ok", false))})
	_configure({"commands": true, "ctx_rag_world": true, "ctx_rag_vault": true, "ctx_params": false})
	_win.open()
	await _settle(10)
	var qs := [
		"Which settlements trade with %s?" % nm,
		"What does my Vault say about the harbour of %s?" % nm,
		## No canary in the question itself: a typed word is the user's own text and would make the body check a false positive.
		"Is there a secret about %s in my notes, such as a hidden heir? Quote it if you find it." % nm]
	for q in qs:
		var ms := await _send(q, "commands")
		var tr := _transcript()
		_log("question", {"q": q, "ms": ms, "usage": _usage(), "transcript_tail": tr.right(1800)})
		await _shot("q%d" % (qs.find(q) + 1))
	_rec["answers_have_canary"] = _transcript().contains(CANARY)


func _scenario_params() -> void:
	_configure({"commands": true, "ctx_params": true})
	_win.open()
	await _settle(10)
	var peak0 := float(_bridge.param_get("peak_m"))
	var dig0: Dictionary = _bridge.edit_state_digest()
	var ms := await _send("Make the climate drier and the mountains lower.", "commands")
	_log("proposal", {"ms": ms, "usage": _usage(), "transcript_tail": _transcript().right(2400)})
	var card := _card("params")
	_rec["card_found"] = not card.is_empty()
	await _shot("card")
	if card.is_empty() or card.get("apply") == null or (card["apply"] as Button).disabled:
		_log("no_apply", {"card": not card.is_empty()})
		return
	_rec["card_summary"] = String(card["info"].get("summary", ""))
	_rec["card_valid"] = (card["info"].get("valid", []) as Array).size()
	_rec["apply_label"] = (card["apply"] as Button).text
	(card["apply"] as Button).pressed.emit()
	await _settle(10)
	await _until(func(): return not _bridge.generating, 600000)
	await _settle(20)
	var peak1 := float(_bridge.param_get("peak_m"))
	_log("applied", {"state": String(card["state"]), "peak_before": peak0, "peak_after": peak1})
	await _shot("applied")
	var ur = card.get("undo_regen")
	_rec["undo_regen_offered"] = ur != null and is_instance_valid(ur) and not (ur as Button).disabled
	if bool(_rec["undo_regen_offered"]):
		(ur as Button).pressed.emit()
		await _settle(10)
		await _until(func(): return not _bridge.generating, 600000)
		await _settle(20)
		var peak2 := float(_bridge.param_get("peak_m"))
		var dig2: Dictionary = _bridge.edit_state_digest()
		_log("undo_regenerated", {"peak_back": peak2, "peak_original": peak0, "field_digest_equal": dig2.get("field") == dig0.get("field")})
		await _shot("undone")


func _scenario_edit(prompt: String) -> void:
	_configure({"edit": true, "ctx_region": true})
	_win.open()
	await _settle(10)
	_rec["region"] = _paint_ridge()
	_win._edit_key = ""
	_win._refresh()
	var dig0: Dictionary = _bridge.edit_state_digest()
	var ms := await _send(prompt, "edit")
	_log("proposal", {"prompt": prompt, "ms": ms, "usage": _usage(), "transcript_tail": _transcript().right(2400)})
	var card := _card("edit")
	_rec["card_found"] = not card.is_empty()
	await _shot("card")
	if card.is_empty():
		return
	## F12 fix 8 (`--revise 1`): press the card's own "Ask the model to revise" button once and take the new card.
	if String(_o["revise"]) == "1" and card.get("revise") != null and not (card["revise"] as Button).disabled:
		_rec["first_card_ops"] = (card["info"].get("valid", []) as Array).map(func(v): return String(v["operation"]))
		var t := Time.get_ticks_msec()
		(card["revise"] as Button).pressed.emit()
		await _settle(8)
		await _until(func(): return not _win.run.busy(), 600000)
		await _settle(10)
		_log("revised", {"ms": Time.get_ticks_msec() - t, "old_card_state": String(card["state"]), "usage": _usage(), "transcript_tail": _transcript().right(1800)})
		card = _card("edit")
		await _shot("card_revised")
	var info: Dictionary = card["info"]
	var valid: Array = []
	for v in info.get("valid", []):
		valid.append({"text": String(v["text"]), "operation": v["operation"]})
	_rec["valid"] = valid
	_rec["rejected"] = info.get("rejected", [])
	_rec["structural"] = info.get("structural", "")
	_rec["preview"] = {"ok": (card.get("preview", {}) as Dictionary).get("ok", false), "changed": (card.get("preview", {}) as Dictionary).get("changed", 0)}
	if card.get("apply") == null or (card["apply"] as Button).disabled:
		_log("no_apply", {})
		return
	var g0: int = _app.undo_groups.cursor
	(card["apply"] as Button).pressed.emit()
	await _settle(14)
	await _until(func(): return not _bridge.generating, 600000)
	var dig1: Dictionary = _bridge.edit_state_digest()
	_log("applied", {"state": String(card["state"]), "field_changed": dig1.get("field") != dig0.get("field"), "undo_groups_added": _app.undo_groups.cursor - g0})
	await _shot("applied")
	_app.undo_last()
	await _settle(14)
	_log("undone", {"field_restored": _bridge.edit_state_digest().get("field") == dig0.get("field")})


## The stored active profile's `live.ctx_region` (what a restart or a switch back would restore), or "unknown".
func _profile_region_tick() -> String:
	var set: Dictionary = LlmProfiles.load_set()
	for pr in set.get("profiles", []):
		if String(pr["id"]) == LlmProfiles.active_id():
			return str(bool((pr.get("live", {}) as Dictionary).get("ctx_region", false)))
	return "unknown"


func _scenario_region_tick() -> void:
	## Profiles are in play: the built-in Local profile carries the region tick (Ruling FW item 3). A fresh install has no
	## active profile, so switch to Local first, point it at the recording proxy (the edits write through to Local), tick the
	## region, ask; then a copy of Local with the tick OFF is asked (Send must be refused, no request at all); then Local again.
	_configure({"edit": true})
	var sw: Dictionary = LlmProfiles.switch_to("local")
	_log("switch_local", {"ok": bool(sw.get("ok", false)), "active": LlmProfiles.active_id()})
	var origin := "http://127.0.0.1:%s" % _o["port"]
	for k in {"base_url": origin + "/v1", "model": _o["model"], "thinking": "none", "context_tokens": int(_o["ctx"]), "ctx_region": true}:
		DccSettings.set_llm_value(k, {"base_url": origin + "/v1", "model": _o["model"], "thinking": "none", "context_tokens": int(_o["ctx"]), "ctx_region": true}[k])
	DccSettings.llm_add_consent(origin)
	_win.open()
	await _settle(10)
	_rec["region"] = _paint_ridge()
	_win._edit_key = ""
	_win._refresh()
	var prompt := "Raise the area I painted into a ridge about 200 m high."
	var ms := await _send(prompt, "edit")
	_log("local_tick_on", {"profile": LlmProfiles.active_id(), "profile_stores_region_tick": _profile_region_tick(), "ms": ms, "usage": _usage(), "card": not _card("edit").is_empty()})
	var cp: Dictionary = LlmProfiles.copy_profile("local")
	LlmProfiles.switch_to(String(cp.get("id", "")))
	DccSettings.set_llm_value("ctx_region", false)
	await _settle(6)
	_win._edit_key = ""
	_win._refresh()
	var reason := String((_win.controls["reason"] as Label).text)
	var ms2 := await _send(prompt, "edit")
	_log("copy_tick_off", {"profile": LlmProfiles.active_id(), "profile_stores_region_tick": _profile_region_tick(), "setting": bool(DccSettings.llm_value("ctx_region")), "ms": ms2, "send_refused_with": reason, "request_sent": ms2 >= 0})
	LlmProfiles.switch_to("local")
	await _settle(6)
	_win._edit_key = ""
	_win._refresh()
	var ms3 := await _send(prompt, "edit")
	_log("back_on_local", {"profile": LlmProfiles.active_id(), "setting": bool(DccSettings.llm_value("ctx_region")), "ms": ms3, "usage": _usage()})
