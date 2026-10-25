extends Node
## **LLM-6c stage-2 driver: one setting request against one provider, headless, as JSON.**
## (`LLM_ASSISTANT_SCOPE.md` s.5b.7; milestones LLM-6a/6b.) Probe-only: nothing in `shell/`
## references this file.
##
## WHY it exists: stage 2 runs the same request on the same seed across several models and
## needs the APP's own path, not the Python prototype's: the real catalogue (`param_catalogue.rs`
## over the registry), the real two-step conversation and validator (`cartalith-llm`), the real
## gate (`LlmStream.prepare`) and the real wire (`LlmRun` + `LlmWire`). It prints and writes the
## proposal JSON and the validator's verdicts. It **never applies** anything: there is no
## setter call in this file, and the world it builds is thrown away on exit.
##
##   godot --headless --path . res://_llmparams_cli.tscn -- --preset ollama --model qwen3:14b \
##       --request "drier climate, lower mountains" [--base-url URL] [--seed N] [--grid W] \
##       [--key-env VAR] [--max-tokens N] [--out FILE]
##
## * `--preset` is a `WorldGen`/`LlmBridge` preset id (`ollama`, `lm_studio`, `anthropic`,
##   `custom_openai`, ...); `--base-url` defaults to the preset's. `--model` is free text.
## * `--seed N` generates a world first (grid `--grid`, default 256 wide, the reference aspect),
##   so `current` is that world's settings; without it the shipped defaults are used.
## * Key: `--key-env VAR` reads it from that environment variable; otherwise the OS credential
##   store entry for the preset (where Settings saves it). The key is never printed or written.
## * **Consent is taken from the command line itself**: running this tool against an endpoint is
##   the consent (it is a developer tool, and the panel's consent card is untouched). The
##   settings are passed to the gate directly and never written to `cartalith_settings.cfg`.
## * Exit 0 when a request completed (any outcome, including a refused proposal: refusing is
##   data here), 1 when it could not run (gate refusal, transport failure), 2 for a bad argument.
## Output: one line `PARAMS_RESULT <json>` and, with `--out`, the same JSON in that file.

var _opts := {"preset": "", "model": "", "request": "", "base-url": "", "seed": "", "grid": "256",
	"key-env": "", "max-tokens": "1024", "out": ""}
var _run: Node
var _result := {}
var _t0 := 0


func _fail(code: int, why: String) -> void:
	_result["error"] = why
	_emit()
	get_tree().quit(code)


func _emit() -> void:
	var text := JSON.stringify(_result)
	print("PARAMS_RESULT " + text)
	if String(_opts["out"]) != "":
		var f := FileAccess.open(String(_opts["out"]), FileAccess.WRITE)
		if f != null:
			f.store_string(text + "\n")
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
	for need in ["preset", "model", "request"]:
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
	var key_env := String(_opts["key-env"])
	var settings := {"enabled": true, "commands": true, "ctx_params": true,
		"local_allowed": local, "remote_allowed": not local, "preset_id": String(_opts["preset"]),
		"base_url": base, "model": String(_opts["model"]),
		"key_source": "env" if key_env != "" else "store", "env_var": key_env if key_env != "" else "CART_LLM_KEY"}
	# The world whose settings the model is shown.
	var wg: Object = ClassDB.instantiate("WorldGen")
	if String(_opts["seed"]) != "":
		var gw := int(_opts["grid"])
		wg.generate_sized(int(_opts["seed"]), 1200.0, gw, int(wg.reference_grid_height(gw, false)))
	var current: Dictionary = wg.get_params()
	_result = {"preset": String(_opts["preset"]), "model": String(_opts["model"]), "origin": String(ep["origin"]),
		"request": String(_opts["request"]), "seed": String(_opts["seed"]), "steps": [], "tool_calls": []}
	_run = load("res://shell/llm_run.gd").new()
	add_child(_run)
	_run.params_source = func() -> Dictionary: return wg.get_params()
	_run.tool_called.connect(func(info: Dictionary): (_result["tool_calls"] as Array).append(info))
	_run.params_step.connect(func(info: Dictionary): (_result["steps"] as Array).append(info))
	_run.proposal.connect(func(info: Dictionary): _result["outcome"] = "proposal"; _result["proposal"] = info)
	_run.answered.connect(func(t: String): _result["outcome"] = "answer"; _result["text"] = t)
	_run.failed.connect(func(m: String): _result["outcome"] = "failed"; _result["reason"] = m)
	_run.usage.connect(func(u: Dictionary): _result["usage_last"] = u)
	_run.idle.connect(_on_idle)
	_t0 = Time.get_ticks_msec()
	var r: Dictionary = _run.begin(settings, "commands", PackedStringArray([String(ep["origin"])]),
		{"request": String(_opts["request"]), "current": current, "max_tokens": int(_opts["max-tokens"]), "include_usage": true})
	if not bool(r.get("ok", false)):
		_fail(1, "refused before sending: %s" % String(r.get("reason", "")))
		return
	get_tree().create_timer(900.0).timeout.connect(func(): _fail(1, "watchdog: no answer in 900 s"))


func _on_idle() -> void:
	_result["total_ms"] = Time.get_ticks_msec() - _t0
	if not _result.has("outcome"):
		_result["outcome"] = "cancelled"
	_emit()
	LlmWire.shutdown_all()
	get_tree().quit(0 if String(_result["outcome"]) != "failed" else 1)
