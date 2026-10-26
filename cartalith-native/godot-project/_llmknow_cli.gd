extends Node
## **RK-5/RK-6 live driver: one knowledge question against one real model, as JSON.**
## (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.5, s.8; Rulings DM and DN.) Probe-only: nothing in `shell/`
## references this file. The `_llmparams_cli.gd` pattern.
##
## WHY it exists: to see what a real model does with the knowledge tools and the pre-retrieval pack through the
## APP's own path (the real gate, the real conversation, the real index, the real wire), on a real generated
## world and a real Vault written to a scratch directory. It never applies anything: the conversation has no
## setting tool in it, and the world and the Vault it builds are thrown away on exit (the Vault directory is
## removed). Single runs; what it prints are observations, not rates.
##
##   godot --headless --path . res://_llmknow_cli.tscn -- --preset lm_studio --model qwen3-14b \
##       --question port|faith|custom --mode tools|pre|both [--text "..."] [--seed N] [--grid W] \
##       [--base-url URL] [--max-tokens N] [--reasoning none] [--out FILE]
##
## * `--question port` asks "Which settlements trade with <S>?" for a settlement S that has trade partners, with the
##   expected partners read from `civ_trade_flows` (the engine's own getter); `faith` asks "What do we know about
##   <F>?" for a faith F of the world that a planted note also mentions; `custom` asks `--text`.
## * `--mode` is how the knowledge base reaches the model: `pre` (the app searches first, the model gets the pack and
##   no tool), `tools` (the model has the three tools and no pack) or `both`.
## * Consent is the command line itself (a developer tool); the settings are passed to the gate directly and never
##   written to `cartalith_settings.cfg`. No key is read unless `--key-env` names a variable.
## Output: one line `KNOW_RESULT <json>` and, with `--out`, the same JSON in that file. Exit 0 when a request completed.

var _opts := {"preset": "", "model": "", "question": "port", "mode": "tools", "text": "", "base-url": "", "seed": "9137",
	"grid": "256", "max-tokens": "1024", "out": "", "key-env": "", "reasoning": ""}
var _run: Node
var _result := {}
var _t0 := 0
var _vault_dir := ""
var _steps_text := ""


## The two members `LlmKnowledge` reads off the engine bridge.
class Shim extends RefCounted:
	var generating := false
	var world_gen: Object


func _emit() -> void:
	var text := JSON.stringify(_result)
	print("KNOW_RESULT " + text)
	if String(_opts["out"]) != "":
		var f := FileAccess.open(String(_opts["out"]), FileAccess.WRITE)
		if f != null:
			f.store_string(text + "\n")
			f.close()


func _fail(code: int, why: String) -> void:
	_result["error"] = why
	_emit()
	_cleanup()
	get_tree().quit(code)


func _cleanup() -> void:
	if _vault_dir != "":
		_rm(_vault_dir)


func _rm(path: String) -> void:
	var d := DirAccess.open(path)
	if d == null:
		return
	for f in d.get_files():
		d.remove(f)
	for s in d.get_directories():
		_rm(path.path_join(s))
		d.remove(s)
	DirAccess.remove_absolute(path)


func _put(rel: String, text: String) -> void:
	var p := _vault_dir.path_join(rel)
	DirAccess.make_dir_recursive_absolute(p.get_base_dir())
	var f := FileAccess.open(p, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if not a.begins_with("--") or not _opts.has(a.substr(2)) or i + 1 >= args.size():
			_fail(2, "unrecognised argument %s; this driver reads --%s" % [a, ", --".join(_opts.keys())])
			return
		_opts[a.substr(2)] = String(args[i + 1])
		i += 2
	for need in ["preset", "model"]:
		if String(_opts[need]) == "":
			_fail(2, "--%s is required" % need)
			return
	if not ClassDB.class_exists("LlmBridge") or not ClassDB.class_exists("WorldGen"):
		_fail(1, "this build has no LlmBridge/WorldGen")
		return
	var llm: Object = ClassDB.instantiate("LlmBridge")
	var preset := {}
	for p in llm.llm_presets():
		if String(p["id"]) == String(_opts["preset"]):
			preset = p
	if preset.is_empty():
		_fail(2, "unknown preset %s" % _opts["preset"])
		return
	var base := String(_opts["base-url"]) if String(_opts["base-url"]) != "" else String(preset.get("base_url", ""))
	var ep: Dictionary = llm.llm_endpoint_info(String(_opts["preset"]), base)
	if not bool(ep.get("ok", false)):
		_fail(2, "base URL refused: %s" % String(ep.get("reason", "")))
		return
	var local := String(ep.get("class", "")) in ["same_machine", "local_network"]
	var mode := String(_opts["mode"])
	var key_env := String(_opts["key-env"])
	var settings := {"enabled": true, "commands": true, "ctx_params": false, "ctx_rag_world": true, "ctx_rag_vault": true,
		"rag_pre": "on" if mode in ["pre", "both"] else "off", "rag_tools": mode in ["tools", "both"],
		"local_allowed": local, "remote_allowed": not local, "preset_id": String(_opts["preset"]), "base_url": base,
		"model": String(_opts["model"]), "key_source": "env", "env_var": key_env if key_env != "" else "CART_LLM_KEY",
		"rag_hosted_origins": PackedStringArray([String(ep["origin"])])}

	# The world and the Vault.
	var wg: Object = ClassDB.instantiate("WorldGen")
	var gw := int(_opts["grid"])
	wg.generate_sized(int(_opts["seed"]), 1200.0, gw, int(wg.reference_grid_height(gw, false)))
	var rows: Array = wg.get_settlements()
	var flows: Dictionary = wg.civ_trade_flows()
	var partners := {}
	for f in flows.get("flows", []):
		var ta := int((rows[int(f["from"])] as Dictionary)["tid"])
		var tb := int((rows[int(f["to"])] as Dictionary)["tid"])
		if ta == tb:
			continue
		if not partners.has(ta):
			partners[ta] = []
		if not partners.has(tb):
			partners[tb] = []
		if not partners[ta].has(tb):
			partners[ta].append(tb)
		if not partners[tb].has(ta):
			partners[tb].append(ta)
	var name_of := {}
	for r in rows:
		name_of[int((r as Dictionary)["tid"])] = String((r as Dictionary)["name"])
	var docs: Array = (wg.knowledge_world_documents() as Dictionary).get("documents", [])
	var faith := ""
	var faith_id := ""
	for d in docs:
		if String((d as Dictionary)["kind"]) == "religion":
			faith = String(d["title"])
			faith_id = String(d["id"])
			break
	var sx := -1
	for tid in partners:
		if partners[tid].size() >= 2:
			sx = int(tid)
			break
	if sx < 0:
		for tid in partners:
			sx = int(tid)
			break
	_vault_dir = ProjectSettings.globalize_path("user://rk_cli_vault")
	_rm(_vault_dir)
	if sx >= 0:
		_put("Places/%s.md" % name_of[sx], "# %s\nThe harbour of %s was walled in 1204 by the Order of Dawn. Merchants say the market square floods in spring.\n" % [name_of[sx], name_of[sx]])
	if faith != "":
		_put("Faiths/%s.md" % faith, "# %s\nThe first %s shrine was raised in the year 1100 by hill farmers; pilgrims still leave grain at its door.\n" % [faith, faith])
	_put("DM/Plot.md", "# Plot\nDMCANARY the secret ending.\n")
	var vc: Dictionary = wg.vault_connect(_vault_dir, "CLI vault")
	var expected_names: Array = []
	var question := String(_opts["text"])
	match String(_opts["question"]):
		"port":
			if sx >= 0:
				question = "Which settlements trade with %s?" % name_of[sx]
				for t in partners[sx]:
					expected_names.append(String(name_of[t]))
		"faith":
			question = "What do we know about %s?" % faith
	if question == "":
		_fail(2, "no question: --text is required for --question custom (or the world had no trade partner / faith)")
		return
	var shim := Shim.new()
	shim.world_gen = wg
	_run = load("res://shell/llm_run.gd").new()
	add_child(_run)
	var consented := PackedStringArray([String(ep["origin"])])
	_run.knowledge_source = func(needs: Dictionary) -> Dictionary:
		var plan: Dictionary = _run.stream.knowledge_plan(settings, "commands", consented)
		return LlmKnowledge.run(shim, plan, needs)
	_run.params_source = func() -> Dictionary: return wg.get_params()
	_result = {"preset": String(_opts["preset"]), "model": String(_opts["model"]), "mode": mode, "seed": int(_opts["seed"]),
		"question": question, "vault_connected": bool(vc.get("ok", false)), "expected_names": expected_names,
		"expected_partner_tids": partners.get(sx, []), "settlement_tid": sx, "faith_id": faith_id, "faith": faith,
		"tool_calls": [], "steps": [], "text": ""}
	_run.tool_called.connect(func(info: Dictionary): (_result["tool_calls"] as Array).append(info))
	_run.params_step.connect(func(info: Dictionary): (_result["steps"] as Array).append(info))
	_run.token.connect(func(t: String): _result["text"] = String(_result["text"]) + t)
	_run.sources_shown.connect(func(info: Dictionary): _result["sources"] = info.get("sources", []); _result["citations"] = info.get("citations", []))
	_run.answered.connect(func(t: String): _result["outcome"] = "answer"; _result["answer"] = t)
	_run.failed.connect(func(m: String): _result["outcome"] = "failed"; _result["reason"] = m)
	_run.finished.connect(func(info: Dictionary): if not _result.has("outcome"): _result["outcome"] = "finished")
	_run.usage.connect(func(u: Dictionary): _result["usage_last"] = u)
	_run.idle.connect(_on_idle)
	var plan0: Dictionary = _run.stream.knowledge_plan(settings, "commands", consented)
	_result["plan"] = plan0
	var a := {"request": question, "current": wg.get_params(), "max_tokens": int(_opts["max-tokens"]), "include_usage": true}
	var shimfacts := Shim.new()
	shimfacts.world_gen = wg
	var raw := LlmKnowledge.search_pack(shimfacts, plan0, question)
	if raw != "":
		a["knowledge_pack"] = raw
		_result["pack_raw"] = raw
	_t0 = Time.get_ticks_msec()
	var r: Dictionary = _run.begin(settings, "commands", consented, a)
	_result["prepare"] = {"ok": r.get("ok", false), "tools": r.get("tools", []), "pack_hits": r.get("pack_hits", -1), "pack_tokens": r.get("pack_tokens", -1), "reason": r.get("reason", "")}
	if not bool(r.get("ok", false)):
		_fail(1, "refused before sending: %s" % String(r.get("reason", "")))
		return
	get_tree().create_timer(900.0).timeout.connect(func(): _fail(1, "watchdog: no answer in 900 s"))


func _on_idle() -> void:
	_result["total_ms"] = Time.get_ticks_msec() - _t0
	if not _result.has("outcome"):
		_result["outcome"] = "cancelled"
	var txt := String(_result.get("answer", _result.get("text", "")))
	var hit: Array = []
	for n in _result.get("expected_names", []):
		if txt.contains(String(n)):
			hit.append(String(n))
	_result["expected_names_in_answer"] = hit
	_emit()
	LlmWire.shutdown_all()
	_cleanup()
	get_tree().quit(0 if String(_result["outcome"]) != "failed" else 1)
