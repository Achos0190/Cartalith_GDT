extends Node
## **RK-8 live-evaluation driver: the RK-0 question set against one model, as JSON lines.**
## (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.8 RK-8; Rulings DM, DN.) Probe-only: nothing in `shell/` references
## this file. The `_llmknow_cli.gd` / `_llmparams_cli.gd` pattern, scaled from one question to the set.
##
## WHAT it does: for each world seed it generates the world the app generates, asks the engine to build the
## RK-0 synthetic Vault and question set FROM THAT WORLD (`knowledge_eval_prepare`), connects the Vault, and
## then for each repeat x mode x question runs the app's own path (the real gate, conversation, index and
## wire: `llm_run.gd` + `LlmKnowledge`) and appends one `RunRecord` JSON line (`live_eval.rs`) to
## `<out-dir>/<label>__<seed>.jsonl`. It scores nothing: the report is
##
##   cargo run -p cartalith-knowledge --example rk8_report -- <out-dir> "<title>"
##
## so a record file is the whole evidence and the report reproduces from it alone.
##
## THE COMMAND (one per model; the owner swaps the model between commands, this driver never loads,
## unloads or switches one):
##
##   godot --headless --path . res://_llmknow_eval_cli.tscn -- --preset lm_studio --model qwen3-14b \
##       --out-dir <dir> [--label NAME] [--seeds 9137,24601,71077345] [--modes tools,pre] [--repeats 3] \
##       [--grid 256] [--per-category N] [--categories a,b] [--max-tokens 1024] [--base-url URL] [--key-env VAR]
##
## For a hosted model add `--key-env <VAR>` naming an environment variable that holds the key (the key is
## read by the app's own path and is never written to a record). `--headless` is correct here: nothing
## rasterises. Consent is the command line itself (a developer tool); the settings go to the gate
## directly and are never written to `cartalith_settings.cfg`.
##
## OFFLINE VALIDATION (no model, no LM Studio): `--fake oracle|blank|invent` starts the in-test loopback
## server (`_llm_mock_server.gd`) and scripts it, so the SAME code path runs with a scripted "model":
##   oracle - calls `search_knowledge` with the question (tools/both) and answers with the expected literals
##            and gold citations; the positive control (what a perfect reader of the retrieval would say).
##   blank  - always says it has no information; the refusal control (scores 0 on every answerable question).
##   invent - answers with an invented figure and an unretrieved citation; the groundedness control.
## A fake run proves the pipeline and the scorer; it says NOTHING about any real model.
##
## `--per-category N` runs only the first N questions of each category (a cheap smoke run); the default 0
## runs the whole set. Unknown arguments exit 2.

const KEY_VAR := "CART_LLM_EVAL_KEY"
const FAKE_KEY := "sk-EVAL-fake-0123456789"

var _opts := {"preset": "", "model": "", "label": "", "out-dir": "", "seeds": "9137,24601,71077345", "modes": "tools,pre", "repeats": "3",
	"grid": "256", "per-category": "0", "categories": "", "max-tokens": "1024", "base-url": "", "key-env": "", "fake": "", "timeout": "900"}
var _server: LlmMockServer
var _vault_dir := ""
var _runs := 0


## The two members `LlmKnowledge` reads off the engine bridge.
class Shim extends RefCounted:
	var generating := false
	var world_gen: Object


func _fail(code: int, why: String) -> void:
	print("EVAL_ERROR " + why)
	_cleanup()
	get_tree().quit(code)


func _cleanup() -> void:
	if _server != null:
		_server.stop()
	if _vault_dir != "":
		_rm(_vault_dir)
	LlmWire.shutdown_all()


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
	for need in ["model", "out-dir"]:
		if String(_opts[need]) == "":
			_fail(2, "--%s is required" % need)
			return
	var fake := String(_opts["fake"])
	if fake != "" and not fake in ["oracle", "blank", "invent"]:
		_fail(2, "--fake is oracle, blank or invent")
		return
	if fake == "" and String(_opts["preset"]) == "":
		_fail(2, "--preset is required unless --fake is given")
		return
	if not ClassDB.class_exists("LlmBridge") or not ClassDB.class_exists("WorldGen"):
		_fail(1, "this build has no LlmBridge/WorldGen")
		return
	var modes: PackedStringArray = String(_opts["modes"]).split(",", false)
	for m in modes:
		if not m in ["tools", "pre", "both"]:
			_fail(2, "--modes takes tools, pre and both")
			return
	DirAccess.make_dir_recursive_absolute(String(_opts["out-dir"]))
	_main.call_deferred(modes)


## The settings dictionary the gate reads, for one mode.
func _settings(mode: String, preset: String, base: String, origin: String, local: bool, model: String, key_var: String) -> Dictionary:
	return {"enabled": true, "commands": true, "ctx_params": false, "ctx_rag_world": true, "ctx_rag_vault": true,
		"rag_pre": "on" if mode in ["pre", "both"] else "off", "rag_tools": mode in ["tools", "both"],
		"local_allowed": local, "remote_allowed": not local, "preset_id": preset, "base_url": base, "model": model,
		"key_source": "env", "env_var": key_var, "rag_hosted_origins": PackedStringArray([origin])}


func _main(modes: PackedStringArray) -> void:
	var fake := String(_opts["fake"])
	var llm: Object = ClassDB.instantiate("LlmBridge")
	var preset_id := String(_opts["preset"])
	var base := String(_opts["base-url"])
	if fake != "":
		_server = LlmMockServer.new()
		add_child(_server)
		if not _server.start():
			_fail(1, "the loopback server could not start")
			return
		preset_id = "ollama"
		base = "http://127.0.0.1:%d/v1" % _server.port
		OS.set_environment(KEY_VAR, FAKE_KEY)
	else:
		var preset := {}
		for p in llm.llm_presets():
			if String(p["id"]) == preset_id:
				preset = p
		if preset.is_empty():
			_fail(2, "unknown preset %s" % preset_id)
			return
		if base == "":
			base = String(preset.get("base_url", ""))
	var ep: Dictionary = llm.llm_endpoint_info(preset_id, base)
	if not bool(ep.get("ok", false)):
		_fail(2, "base URL refused: %s" % String(ep.get("reason", "")))
		return
	var local := String(ep.get("class", "")) in ["same_machine", "local_network"]
	var key_var := KEY_VAR if fake != "" else (String(_opts["key-env"]) if String(_opts["key-env"]) != "" else "CART_LLM_KEY")
	var label := String(_opts["label"]) if String(_opts["label"]) != "" else (("fake-" + fake) if fake != "" else String(_opts["model"]))
	var consented := PackedStringArray([String(ep["origin"])])
	var cats: PackedStringArray = String(_opts["categories"]).split(",", false)
	var per_cat := int(_opts["per-category"])
	var repeats := int(_opts["repeats"])
	var timeout := float(_opts["timeout"]) if fake == "" else 60.0
	_vault_dir = ProjectSettings.globalize_path("user://rk8_eval_vault")
	var started := Time.get_ticks_msec()

	for seed_text in String(_opts["seeds"]).split(",", false):
		var seed := int(seed_text)
		_rm(_vault_dir)
		var wg: Object = ClassDB.instantiate("WorldGen")
		var gw := int(_opts["grid"])
		wg.generate_sized(seed, 1200.0, gw, int(wg.reference_grid_height(gw, false)))
		var prep: Dictionary = wg.knowledge_eval_prepare(_vault_dir)
		if not bool(prep.get("ok", false)):
			_fail(1, "world %d: %s" % [seed, String(prep.get("reason", "prepare failed"))])
			return
		var vc: Dictionary = wg.vault_connect(_vault_dir, "RK-8 evaluation vault")
		if not bool(vc.get("ok", false)):
			_fail(1, "world %d: the evaluation vault could not be connected" % seed)
			return
		var all_q: Array = JSON.parse_string(String(prep["questions_json"]))
		var questions: Array = []
		var taken := {}
		for q in all_q:
			var c := String(q["category"])
			if not cats.is_empty() and not c in cats:
				continue
			taken[c] = int(taken.get(c, 0)) + 1
			if per_cat > 0 and int(taken[c]) > per_cat:
				continue
			questions.append(q)
		var shim := Shim.new()
		shim.world_gen = wg
		var path := String(_opts["out-dir"]).path_join("%s__%d.jsonl" % [label, seed])
		var file := FileAccess.open(path, FileAccess.WRITE)
		if file == null:
			_fail(1, "cannot write %s" % path)
			return
		print("EVAL_WORLD seed=%d questions=%d notes=%d vault_hash=%s -> %s" % [seed, questions.size(), int(prep["notes"]), String(prep["vault_hash"]), path])
		for r in repeats:
			for mode in modes:
				var settings := _settings(mode, preset_id, base, String(ep["origin"]), local, String(_opts["model"]), key_var)
				for q in questions:
					var rec: Dictionary = await _one(wg, shim, settings, consented, q, mode, fake, timeout)
					rec["model"] = label
					rec["mode"] = mode
					rec["seed"] = seed
					rec["repeat"] = r
					rec["question"] = q
					file.store_line(JSON.stringify(rec))
					file.flush()
					_runs += 1
		file.close()
		wg.knowledge_clear()
	print("EVAL_DONE runs=%d seconds=%.1f label=%s" % [_runs, (Time.get_ticks_msec() - started) / 1000.0, label])
	_cleanup()
	get_tree().quit(0)


## The mock server's script for one question (offline controls only).
func _script(fake: String, mode: String, q: Dictionary) -> Array:
	match fake:
		"blank":
			return [{"text": "I do not have information about that in the sources."}]
		"invent":
			return [{"text": "It holds 98765 people, founded in 1987. [W:settlement:99999]"}]
	var gold: Array = q.get("gold", [])
	var text := ""
	if String(q["category"]) in ["unanswerable", "private"]:
		text = "I could not find that in the sources."
	else:
		var parts: Array = []
		for e in q.get("expect", []):
			parts.append(String(e))
		text = "From the sources: " + ", ".join(parts) + "."
		for g in gold:
			text += " [%s]" % String(g)
	var steps: Array = []
	if mode != "pre":
		steps.append({"name": "search_knowledge", "args": {"query": String(q["text"]), "scope": "both", "limit": 5}})
	steps.append({"text": text})
	return steps


## One question, one mode, one repeat: the `RunRecord` fields the run produces (the caller adds model,
## mode, seed, repeat and the question).
func _one(wg: Object, shim: Object, settings: Dictionary, consented: PackedStringArray, q: Dictionary, mode: String, fake: String, timeout: float) -> Dictionary:
	var rec := {"outcome": "", "answer": "", "ledger": [], "citations": [], "context": [], "tool_calls": [],
		"latency_ms": null, "prompt_tokens": null, "completion_tokens": null}
	var run: Node = load("res://shell/llm_run.gd").new()
	add_child(run)
	var state := {"done": false, "prompt": 0, "completion": 0, "usage": false, "text": ""}
	run.knowledge_source = func(needs: Dictionary) -> Dictionary:
		var plan: Dictionary = run.stream.knowledge_plan(settings, "commands", consented)
		var out: Dictionary = LlmKnowledge.run(shim, plan, needs)
		for k in out:
			(rec["context"] as Array).append(String(out[k]))
		return out
	run.params_source = func() -> Dictionary: return wg.get_params()
	run.tool_called.connect(func(info: Dictionary): (rec["tool_calls"] as Array).append(String(info.get("name", ""))))
	run.token.connect(func(t: String): state["text"] = String(state["text"]) + t)
	run.sources_shown.connect(func(info: Dictionary):
		var ids: Array = []
		for s in info.get("sources", []):
			ids.append(String((s as Dictionary).get("id", "")))
		rec["ledger"] = ids
		var cits: Array = []
		for c in info.get("citations", []):
			cits.append({"id": String((c as Dictionary).get("id", "")), "verified": bool((c as Dictionary).get("verified", false))})
		rec["citations"] = cits)
	run.answered.connect(func(t: String): rec["outcome"] = "answer"; rec["answer"] = t)
	run.failed.connect(func(m: String): rec["outcome"] = "failed"; rec["reason"] = m)
	run.usage.connect(func(u: Dictionary):
		state["usage"] = true
		state["prompt"] = int(state["prompt"]) + int(u.get("prompt", 0))
		state["completion"] = int(state["completion"]) + int(u.get("completion", 0)))
	run.idle.connect(func(): state["done"] = true)
	if fake != "":
		_server.scenario = "tools"
		_server.tool_script = _script(fake, mode, q)
		_server.tool_served = 0
	var plan0: Dictionary = run.stream.knowledge_plan(settings, "commands", consented)
	var a := {"request": String(q["text"]), "current": wg.get_params(), "max_tokens": int(_opts["max-tokens"]), "include_usage": true}
	var shim_facts := Shim.new()
	shim_facts.world_gen = wg
	var pack := LlmKnowledge.search_pack(shim_facts, plan0, String(q["text"]))
	if pack != "":
		a["knowledge_pack"] = pack
		(rec["context"] as Array).append(pack)
	var t0 := Time.get_ticks_msec()
	var r: Dictionary = run.begin(settings, "commands", consented, a)
	if not bool(r.get("ok", false)):
		rec["outcome"] = "failed"
		rec["reason"] = "refused before sending: %s" % String(r.get("reason", ""))
	else:
		var waited := 0.0
		while not bool(state["done"]) and waited < timeout:
			await get_tree().process_frame
			waited += get_process_delta_time()
		if not bool(state["done"]):
			run.cancel()
			rec["outcome"] = "cancelled"
			rec["reason"] = "no end within %d s" % int(timeout)
		else:
			# A latency is recorded only for a run that reached its end; a refused or timed-out run has none.
			rec["latency_ms"] = Time.get_ticks_msec() - t0
	if String(rec["outcome"]) == "":
		rec["outcome"] = "cancelled"
	if bool(state["usage"]):
		rec["prompt_tokens"] = int(state["prompt"])
		rec["completion_tokens"] = int(state["completion"])
	run.queue_free()
	return rec
