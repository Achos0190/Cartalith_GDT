extends Node
## **RK-9 live embedding driver: the gate's vectors from a real embeddings endpoint, as JSON.**
## (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.7.2, s.8 RK-9; owner question K10.) Probe-only: nothing in `shell/`
## references this file. The `_llmknow_eval_cli.gd` pattern, for `/v1/embeddings` and `/api/embed`.
##
## WHAT it does: for each committed evaluation world (`knowledge_eval_<seed>.json`) it asks the engine bridge
## for the exact texts the gate must embed (`knowledge_gate_inputs`: every retrieval unit and every question,
## the model's prefixes applied), posts them in batches to the endpoint, checks every reply in Rust
## (`embeddings_parse`: count, order, one dimension, finite numbers), and writes
## `<out-dir>/<label>__<seed>.json` = `{vectors: {model, dim, units, queries}, timing: {tokens, seconds, vram_mib}}`.
## It decides nothing. The gate is
##
##   cargo run -p cartalith-knowledge --example rk9_gate -- live <out-dir>
##
## THE COMMAND (the owner swaps the embedding model between commands; this driver never loads, unloads or
## switches one -- an embeddings request to LM Studio loads the named model on demand, which is why it is the
## owner's to run, not this build's):
##
##   godot --headless --path . res://_llmknow_embed_cli.tscn -- --preset lm_studio --model <embedding model id> \
##       --profile qwen3|nomic|plain --out-dir <dir> [--label NAME] [--kind openai|ollama] [--seeds 71077345,24601,9137] \
##       [--passes 3] [--batch 16] [--dimensions N] [--base-url URL] [--key-env VAR] [--vram-mib N] [--allow-remote yes]
##
## * `--profile` picks the prefixes (Qwen3-Embedding wants an instruction on queries only; nomic wants both
##   `search_` prefixes); the endpoint applies neither.
## * `--passes N` repeats the whole embedding pass N times (default 3); `timing.seconds` holds one entry per pass
##   so the report gives median and range, and the vectors kept are the last pass's. Run it alone: the
##   throughput of a pass that shares the machine with anything else is not a measurement (`MISTAKES.md`).
## * `--vram-mib` is recorded verbatim when the owner measured the spill above idle by other means; it is `null`
##   otherwise. This driver does not sample VRAM.
## * Only a same-machine or local-network endpoint is used unless `--allow-remote yes` (the app's own rule, Ruling DF).
## * `--fake` starts the in-test loopback server (`_llm_mock_server.gd`, scenario `embeddings`) in place of an
##   endpoint, so the driver and its file format are verified without any model.
## Unknown arguments exit 2.

const KEY_VAR := "CART_LLM_EMBED_KEY"

var _opts := {"preset": "", "model": "", "label": "", "profile": "plain", "kind": "openai", "out-dir": "", "seeds": "71077345,24601,9137",
	"passes": "3", "batch": "16", "dimensions": "0", "base-url": "", "key-env": "", "vram-mib": "", "allow-remote": "", "fixture-dir": "../crates/cartalith-knowledge/tests/fixtures", "fake": ""}
var _server: LlmMockServer


func _fail(code: int, why: String) -> void:
	print("EMBED_ERROR " + why)
	if _server != null:
		_server.stop()
	get_tree().quit(code)


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
	if String(_opts["fake"]) == "" and String(_opts["preset"]) == "":
		_fail(2, "--preset is required unless --fake is given")
		return
	if not String(_opts["kind"]) in ["openai", "ollama"]:
		_fail(2, "--kind is openai or ollama")
		return
	if not ClassDB.class_exists("LlmBridge") or not ClassDB.class_exists("WorldGen"):
		_fail(1, "this build has no LlmBridge/WorldGen")
		return
	_main.call_deferred()


## Posts `body` to `url`; `{ok, code, body}`. Never raises.
func _post(url: String, headers: PackedStringArray, body: String) -> Dictionary:
	var req := HTTPRequest.new()
	req.timeout = 180.0
	add_child(req)
	var err := req.request(url, headers, HTTPClient.METHOD_POST, body)
	if err != OK:
		req.queue_free()
		return {"ok": false, "code": 0, "body": "request error %d" % err}
	var res: Array = await req.request_completed
	req.queue_free()
	var text := (res[3] as PackedByteArray).get_string_from_utf8()
	return {"ok": int(res[0]) == HTTPRequest.RESULT_SUCCESS and int(res[1]) >= 200 and int(res[1]) < 300, "code": int(res[1]), "body": text}


func _main() -> void:
	var llm: Object = ClassDB.instantiate("LlmBridge")
	var wg: Object = ClassDB.instantiate("WorldGen")
	var fake := String(_opts["fake"]) != ""
	var preset := String(_opts["preset"])
	var base := String(_opts["base-url"])
	if fake:
		_server = LlmMockServer.new()
		add_child(_server)
		if not _server.start():
			_fail(1, "the loopback server could not start")
			return
		_server.scenario = "embeddings"
		_server.embed_dim = int(_opts["dimensions"]) if int(_opts["dimensions"]) > 0 else 8
		preset = "ollama"
		base = "http://127.0.0.1:%d/v1" % _server.port
	else:
		for p in llm.llm_presets():
			if String(p["id"]) == preset and base == "":
				base = String(p.get("base_url", ""))
	var ep: Dictionary = llm.llm_endpoint_info(preset, base)
	if not bool(ep.get("ok", false)):
		_fail(2, "base URL refused: %s" % String(ep.get("reason", "")))
		return
	var local := String(ep.get("class", "")) in ["same_machine", "local_network"]
	if not local and String(_opts["allow-remote"]) != "yes":
		_fail(2, "the endpoint is not on this machine or network; pass --allow-remote yes to send the corpus there")
		return
	var kind := String(_opts["kind"])
	var url := (base.trim_suffix("/") + "/embeddings") if kind == "openai" else (String(ep["origin"]).trim_suffix("/") + "/api/embed")
	var headers := PackedStringArray(["Content-Type: application/json"])
	if String(_opts["key-env"]) != "":
		var key := OS.get_environment(String(_opts["key-env"]))
		if key == "":
			_fail(2, "--key-env names %s, which is empty" % String(_opts["key-env"]))
			return
		headers.append("Authorization: Bearer " + key)
	var label := String(_opts["label"]) if String(_opts["label"]) != "" else String(_opts["model"]).replace("/", "_")
	var dims := int(_opts["dimensions"])
	var batch := maxi(1, int(_opts["batch"]))
	var passes := maxi(1, int(_opts["passes"]))
	DirAccess.make_dir_recursive_absolute(String(_opts["out-dir"]))
	for seed_text in String(_opts["seeds"]).split(",", false):
		var seed := int(seed_text)
		var fx_path := ProjectSettings.globalize_path("res://").path_join(String(_opts["fixture-dir"])).path_join("knowledge_eval_%d.json" % seed)
		var inp: Dictionary = JSON.parse_string(String(wg.knowledge_gate_inputs(fx_path, String(_opts["profile"]))))
		if inp == null or not bool(inp.get("ok", false)):
			_fail(1, "world %d: %s" % [seed, String(inp.get("reason", "inputs failed")) if inp != null else "inputs unreadable"])
			return
		var rows: Array = []
		for u in inp["units"]:
			rows.append(["u", String(u[0]), String(u[1])])
		for q in inp["queries"]:
			rows.append(["q", String(q[0]), String(q[1])])
		var seconds: Array = []
		var vectors: Array = []
		var tokens_total := 0
		var tokens_known := true
		for pass_n in passes:
			vectors = []
			tokens_total = 0
			var t0 := Time.get_ticks_msec()
			var k := 0
			while k < rows.size():
				var chunk: Array = rows.slice(k, k + batch)
				var texts := PackedStringArray()
				for r in chunk:
					texts.append(String(r[2]))
				var res: Dictionary = await _post(url, headers, String(wg.embeddings_request_body(kind, String(_opts["model"]), texts, dims)))
				if not bool(res["ok"]):
					_fail(1, "world %d: the endpoint answered %d: %s" % [seed, int(res["code"]), String(res["body"]).left(300)])
					return
				var parsed: Dictionary = JSON.parse_string(String(wg.embeddings_parse(kind, String(res["body"]), texts.size())))
				if parsed == null or not bool(parsed.get("ok", false)):
					_fail(1, "world %d: bad reply: %s" % [seed, String(parsed.get("reason", "unreadable")) if parsed != null else "unreadable"])
					return
				vectors.append_array(parsed["vectors"])
				var body_json = JSON.parse_string(String(res["body"]))
				var used := -1
				if body_json is Dictionary:
					if (body_json as Dictionary).has("usage") and ((body_json as Dictionary)["usage"] as Dictionary).has("prompt_tokens"):
						used = int(((body_json as Dictionary)["usage"] as Dictionary)["prompt_tokens"])
					elif (body_json as Dictionary).has("prompt_eval_count"):
						used = int((body_json as Dictionary)["prompt_eval_count"])
				if used < 0:
					tokens_known = false
				else:
					tokens_total += used
				k += batch
			seconds.append((Time.get_ticks_msec() - t0) / 1000.0)
			print("EMBED_PASS seed=%d pass=%d texts=%d seconds=%.2f" % [seed, pass_n + 1, rows.size(), seconds[-1]])
		var dim := (vectors[0] as Array).size()
		var units := {}
		var queries := {}
		for n in rows.size():
			var v: Array = []
			for x in (vectors[n] as Array):
				v.append(snappedf(float(x), 0.000001))
			if String(rows[n][0]) == "u":
				units[String(rows[n][1])] = v
			else:
				queries[String(rows[n][1])] = v
		var vram = null
		if String(_opts["vram-mib"]) != "":
			vram = float(_opts["vram-mib"])
		var doc := {"vectors": {"model": label, "dim": dim, "units": units, "queries": queries},
			"timing": {"tokens": tokens_total if tokens_known else null, "seconds": seconds, "vram_mib": vram}}
		var path := String(_opts["out-dir"]).path_join("%s__%d.json" % [label, seed])
		var f := FileAccess.open(path, FileAccess.WRITE)
		if f == null:
			_fail(1, "cannot write %s" % path)
			return
		f.store_string(JSON.stringify(doc))
		f.close()
		print("EMBED_WORLD seed=%d units=%d queries=%d dim=%d -> %s" % [seed, units.size(), queries.size(), dim, path])
	if _server != null:
		_server.stop()
	print("EMBED_DONE label=%s" % label)
	get_tree().quit(0)
