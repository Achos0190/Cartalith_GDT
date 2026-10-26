extends Node
## **LLM-11 live: grounded Vault prose through the REAL assistant panel, one model arm.**
## (`LLM_ASSISTANT_SCOPE.md` "Grounded place prose"; Rulings DD, DF, DI.) Probe-only, NOT part of any
## suite: it needs a model server that is already running (LM Studio) and never starts, downloads or
## stops anything itself.
##
##   godot --path . --resolution 1600x1000 res://_llmplace_live_probe.tscn -- \
##       --arm A --preset lm_studio --base-url http://127.0.0.1:1234/v1 --model qwen3-14b \
##       --seeds 71077345,24601 --reps 2 --shots <dir> --out <file.jsonl> --vault <dir>
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped user
## dir (`MISTAKES.md`). Reads exactly the flags below; anything else exits 2.
##
## WHAT IT DOES, per arm (the same seeds, places, prompt and order for every arm):
##   1. Configures the assistant through the app's OWN settings code (master switch, Describe place on,
##      the preset, base URL and model; "named Vault notes" OFF for the first condition), runs
##      `LlmGpuRule.sync` exactly as the settings window does, and records the plan.
##   2. For each seed: generates at the New World dialog's default size through the user's path
##      (`generate_checked`), runs the belief model (what opening the Religion layer does), and chooses
##      FOUR places from the engine's own numbers, identically for every arm: the largest settlement; the
##      smallest village or hamlet that has at least two neighbours within 25 km; a coastal town or city
##      (a port) that is neither; and the highest-elevation remaining settlement with at least two
##      neighbours (a remote or mountain place near others).
##   3. NO NOTES condition: for each place, types the owner's request, presses the real Send button, waits,
##      and records timing, tokens, the sent line, the facts size, the grounding check, heuristics for
##      contradiction and for use of neighbours, trade, religion and relations, and the FULL reply text.
##      `--reps` repetitions per place. Screenshots of the card.
##   4. WITH NOTES condition: connects a scratch Vault (`--vault`) holding synthetic notes for ONE place per
##      seed (its own note, a person, its faction, a neighbour, an unrelated note, a private note, and ONE
##      deliberate contradiction with the data), ticks "Send: named Vault notes", and repeats the request
##      for that place `--reps` times.
##
## WHAT IT MUST NEVER DO: read, print or write a key; write anything but its JSON lines, images and the
## scratch Vault; click Save (the Vault is never written here); send a map image to any model (Ruling DI P5).
##
## Output: one `LIVE <json>` line per record and the same JSON in `--out`;
## `RESULT: DONE arm=<a> records=<n>`. Exit 0 finished, 1 could not run, 2 bad argument or watchdog.

const REQUEST := "Write the entry for this settlement for the world Vault: history, daily life, trade, faith, neighbours."
## A hang guard, ms, not an expectation.
const ASK_TIMEOUT_MS := 600000

var _o := {"arm": "", "preset": "lm_studio", "base-url": "http://127.0.0.1:1234/v1", "model": "", "key-source": "env",
	"seeds": "71077345,24601", "reps": "2", "shots": "", "out": "", "vault": "", "watchdog": "5400", "only-notes": "0"}
var _app: Node
var _bridge
var _win
var _records := 0
var _gen_started := 0
var _ev: Array = []
var _t_send := 0
var _vault_connected := false


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


func _until(cond: Callable, ms: int) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


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


func _jpg(name: String) -> String:
	if String(_o["shots"]) == "":
		return ""
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	img.convert(Image.FORMAT_RGB8)
	var path := String(_o["shots"]).path_join("%s_%s.jpg" % [String(_o["arm"]), name])
	img.save_jpg(path, 0.85)
	return path


func _set_seed(s: int) -> void:
	var nwd = _app.new_world_dialog
	nwd.seed_input.set_value_no_signal(s)
	nwd._seed_dirty = false


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
	return {"generate_ms": ms, "backend": str(_bridge.gpu_last_backend())}


func _configure() -> Dictionary:
	var preset := String(_o["preset"])
	var base := String(_o["base-url"])
	var ep: Dictionary = _bridge.llm_endpoint_info(preset, base)
	var local := bool(ep.get("is_local", false))
	var s := {"enabled": true, "commands": false, "prose": true, "edit": false,
		"local_allowed": local, "remote_allowed": not local, "preset_id": preset, "base_url": base,
		"model": String(_o["model"]), "key_source": String(_o["key-source"]), "ctx_notes": false, "ctx_entity": false,
		"gpu_override": "automatic"}
	for k in s:
		if not DccSettings.set_llm_value(k, s[k]):
			return {"error": "set_llm_value refused %s" % k}
	DccSettings.llm_add_consent(String(ep.get("origin", "")))
	var plan: Dictionary = LlmGpuRule.sync(_bridge)
	return {"endpoint": ep, "gpu_plan": plan, "gpu_rule_applied": LlmGpuRule.is_applied(), "selected_after": _bridge.gpu_selected_devices()}


func _hook_run() -> void:
	var run: LlmRun = _win.run
	run.token.connect(func(_t: String): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": "token"}))
	run.usage.connect(func(i: Dictionary): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": "usage", "u": i}))
	run.failed.connect(func(m: String): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": "failed", "message": m}))
	run.retrying.connect(func(m: String): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": "retrying", "message": m}))
	run.finished.connect(func(i: Dictionary): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": "finished", "info": i}))
	run.idle.connect(func(): _ev.append({"t": Time.get_ticks_msec() - _t_send, "e": "idle"}))


# ------------------------------------------------------------------ place choice

## `{index, why}` x4, chosen from the engine's own numbers; identical for every arm on the same world.
func _choose_places() -> Array:
	var ss: Array = _bridge.settlements()
	var ck: float = float(_bridge.world_gen.get_map_width_km()) / float(_bridge.world_gen.get_width())
	var near := {}
	for i in ss.size():
		var n := 0
		for j in ss.size():
			if j == i:
				continue
			var dx := float(ss[j].x) - float(ss[i].x)
			var dy := float(ss[j].y) - float(ss[i].y)
			if sqrt(dx * dx + dy * dy) * ck <= 25.0:
				n += 1
		near[i] = n
	var used := {}
	var out := []
	# (a) the largest
	var a := 0
	for i in ss.size():
		if int(ss[i].population) > int(ss[a].population) or (int(ss[i].population) == int(ss[a].population) and int(ss[i].tid) < int(ss[a].tid)):
			a = i
	used[a] = true
	out.append({"index": a, "why": "capital or largest town"})
	# (b) the smallest village or hamlet; prefer two neighbours, then one, then any (a sparse world has few)
	var b := -1
	for need in [2, 1, 0]:
		for i in ss.size():
			## A hamlet-tier add-on has population 0 in the reference's own shape; a "small village" with people
			## is the interesting case, so those are tried first.
			if used.has(i) or not (String(ss[i].kind) in ["village", "hamlet"]) or int(near[i]) < need or int(ss[i].population) <= 0:
				continue
			if b < 0 or int(ss[i].population) < int(ss[b].population) or (int(ss[i].population) == int(ss[b].population) and int(ss[i].tid) < int(ss[b].tid)):
				b = i
		if b >= 0:
			break
	if b < 0:
		for i in ss.size():
			if not used.has(i) and int(ss[i].population) > 0 and (b < 0 or int(ss[i].population) < int(ss[b].population)):
				b = i
	if b < 0:
		for i in ss.size():
			if not used.has(i):
				b = i
				break
	used[b] = true
	out.append({"index": b, "why": "small village with neighbours"})
	# (c) a port: the largest coastal town or city; else any coastal; else the largest unused
	var c := -1
	for i in ss.size():
		if used.has(i) or not bool(ss[i].coastal) or not (String(ss[i].kind) in ["town", "city", "metropolis"]):
			continue
		if c < 0 or int(ss[i].population) > int(ss[c].population) or (int(ss[i].population) == int(ss[c].population) and int(ss[i].tid) < int(ss[c].tid)):
			c = i
	if c < 0:
		for i in ss.size():
			if not used.has(i) and bool(ss[i].coastal) and (c < 0 or int(ss[i].population) > int(ss[c].population)):
				c = i
	if c < 0:
		for i in ss.size():
			if not used.has(i) and (c < 0 or int(ss[i].population) > int(ss[c].population)):
				c = i
	used[c] = true
	out.append({"index": c, "why": "port (coastal town)"})
	# (d) the highest remaining settlement; prefer two neighbours, then one, then any
	var d := -1
	var best := -1.0e9
	for need in [2, 1, 0]:
		for i in ss.size():
			if used.has(i) or int(near[i]) < need:
				continue
			var cell: Dictionary = _bridge.sample_cell(int(ss[i].x), int(ss[i].y))
			var e := float(cell.get("elevation_m", -1.0e9))
			if e > best:
				best = e
				d = i
		if d >= 0:
			break
	used[d] = true
	out.append({"index": d, "why": "highest-elevation place with neighbours (remote or mountain)", "elevation_m": best})
	for k in out.size():
		out[k]["neighbours_within_25km"] = int(near[int(out[k]["index"])])
	return out


# ------------------------------------------------------------------ analysis helpers

static func _lower(s: String) -> String:
	return s.to_lower()


## Whether `reply` mentions any of `words` (case-insensitive substring).
static func _any(reply: String, words: Array) -> Array:
	var hit := []
	var l := reply.to_lower()
	for w in words:
		var ws := String(w).to_lower()
		if ws != "" and l.contains(ws):
			hit.append(String(w))
	return hit


## The structured facts the model was shown, for the "did it use X" and contradiction heuristics.
func _use_report(reply: String, facts: Dictionary, place: Dictionary) -> Dictionary:
	var wd: Dictionary = facts.get("world_data", {})
	var out := {}
	var neigh := []
	for n in wd.get("neighbours", {}).get("listed", []):
		neigh.append(String(n["name"]))
	out["neighbours_named"] = _any(reply, neigh)
	var goods := []
	var eco: Dictionary = wd.get("economy", {})
	for k in ["exports", "imports", "faction_exports", "faction_imports", "strategic"]:
		for g in eco.get(k, []):
			goods.append(String(g))
	for f in eco.get("flows", []):
		goods.append(String(f.get("good", "")))
	out["trade_goods_named"] = _any(reply, goods)
	var rel: Dictionary = wd.get("religion", {})
	var faiths := []
	if rel.has("plurality"):
		faiths.append(String(rel["plurality"]))
	for s in rel.get("shares", []):
		faiths.append(String(s.get("faith", "")))
	# labels, as the facts text prints them
	out["faith_named"] = _any(reply, _faith_labels(faiths, facts))
	var rels := []
	for p in wd.get("relations", {}).get("pairs", []):
		rels.append(String(p.get("other", "")))
	out["relation_factions_named"] = _any(reply, rels)
	out["stance_words"] = _any(reply, ["allied", "friendly", "neutral", "wary", "hostile"])
	out["faction_named"] = _any(reply, [String(wd.get("faction", {}).get("name", ""))])
	out["uses_neighbours"] = not (out["neighbours_named"] as Array).is_empty()
	out["uses_trade"] = not (out["trade_goods_named"] as Array).is_empty() or reply.to_lower().contains("trade")
	out["uses_religion"] = not (out["faith_named"] as Array).is_empty()
	out["uses_relations"] = not (out["relation_factions_named"] as Array).is_empty() or not (out["stance_words"] as Array).is_empty()
	# contradiction heuristics (candidates only; the report reads the text)
	var l := reply.to_lower()
	var flags := []
	var coastal := bool(place.get("coastal", false))
	if not coastal and (l.contains("harbour") or l.contains("harbor") or l.contains("seaport") or l.contains("quay") or l.contains("on the coast") or l.contains("coastal")):
		flags.append("coast words for an inland place")
	if coastal and (l.contains("landlocked") or l.contains("far from the sea")):
		flags.append("inland words for a coastal place")
	var cap := bool(place.get("capital", false))
	if cap and (l.contains("not the capital") or l.contains("no capital")):
		flags.append("denies the capital")
	if not cap and (l.contains("the capital of") or l.contains("capital city")):
		flags.append("calls it a capital though it is not")
	out["contradiction_candidates"] = flags
	return out


func _collect_text(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	for ch in n.get_children():
		_collect_text(ch, out)


func _faith_labels(keys: Array, facts: Dictionary) -> Array:
	var labels := []
	var vocab: Dictionary = {}
	for r in _bridge.civ_religion_vocabulary():
		vocab[String(r.key)] = String(r.label)
	for k in keys:
		labels.append(String(vocab.get(String(k), String(k))))
	return labels


# ------------------------------------------------------------------ the asking

func _ask(condition: String, seed_: int, pick: Dictionary, rep: int, tag: String) -> void:
	var ss: Array = _bridge.settlements()
	var idx: int = int(pick["index"])
	var place: Dictionary = ss[idx]
	_app.right_dock_ctrl.on_settlement_selected(place, idx)
	await _frames(4)
	_win.open()
	await _frames(10)
	_win._pick_task("prose")
	if _vault_connected and bool(DccSettings.llm_value("ctx_notes")):
		# Pick the place's own note as the save target so it is the "existing note".
		var picked := _c("note") as OptionButton
		for k in picked.item_count:
			if picked.get_item_text(k) == "Places/%s.md" % String(place.name):
				picked.select(k)
				picked.item_selected.emit(k)
	var inp := _c("input") as TextEdit
	inp.text = REQUEST
	inp.text_changed.emit()
	await _frames(4)
	_win._place = _win._current_place()
	_win._ensure_facts(true)
	_win._refresh()
	await _frames(3)
	var send := _c("send") as Button
	var rec := {"kind": "prose", "condition": condition, "seed": seed_, "rep": rep, "place": String(place.name), "tid": int(place.tid),
		"place_kind": String(place.kind), "population": int(place.population), "coastal": bool(place.coastal), "capital": bool(place.capital),
		"why_chosen": String(pick["why"]), "send_disabled": send.disabled, "send_reason": (_c("reason") as Label).text if _c("reason") != null else "",
		"sent_line": (_c("sent") as Label).text, "facts_status": (_c("facts_status") as Label).text}
	var fx: Dictionary = _win._facts
	if not fx.is_empty() and bool(fx.get("bundle", {}).get("ok", false)):
		var b: Dictionary = fx["bundle"]
		rec["facts_chars"] = int(b.get("chars", 0))
		rec["facts_gather_ms"] = int(fx.get("gather_ms", 0))
		rec["facts_sections"] = b.get("sections", [])
		rec["facts_text_sha"] = String(b.get("text", "")).sha256_text().left(16)
		rec["vault_titles"] = b.get("vault_titles", [])
		rec["vault_withheld"] = int(b.get("vault_withheld", 0))
		rec["neighbours_listed"] = int(b.get("neighbours_listed", 0))
		rec["neighbours_within"] = int(b.get("neighbours_within", 0))
		rec["truncated"] = b.get("truncated", [])
		rec["not_available"] = b.get("not_available", [])
	_ev = []
	_t_send = Time.get_ticks_msec()
	await _click(send)
	var started := await _until(func(): return _win.run.busy(), 5000)
	rec["clicks"] = 1
	if not started:
		## Measured in the first full run: about a third of the injected clicks on Send did nothing, with the
		## button enabled and no message from the panel (the same click works the next time). The cause is
		## not established. The request is therefore retried by a second real click, and then by the
		## button's own `pressed` signal, and the record says which one started it.
		await _click(send)
		rec["clicks"] = 2
		started = await _until(func(): return _win.run.busy(), 5000)
		if not started and not send.disabled:
			send.pressed.emit()
			rec["clicks"] = 3
			started = await _until(func(): return _win.run.busy(), 5000)
	rec["started"] = started
	if not started:
		## A Send that never started a reply: say what the panel said (a refusal is a message card).
		var tail := []
		for k in range(maxi(0, _win._msgs.get_child_count() - 3), _win._msgs.get_child_count()):
			var parts: Array = []
			_collect_text(_win._msgs.get_child(k), parts)
			tail.append(" | ".join(parts))
		rec["msgs_tail"] = tail
		rec["send_button_disabled_after"] = send.disabled
		rec["send_text"] = send.text
		rec["send_rect"] = str(send.get_global_rect())
		rec["win_rect"] = str(Rect2(Vector2(_win.position), Vector2(_win.size)))
	var finished := await _until(func(): return not _win.run.busy(), ASK_TIMEOUT_MS)
	await _frames(8)
	rec["finished"] = finished
	var total := Time.get_ticks_msec() - _t_send
	rec["total_ms"] = total
	rec["first_token_ms"] = int(_win.run.first_token_ms)
	var u: Dictionary = _win.run.last_usage if started else {}
	rec["usage"] = u
	rec["reply"] = _win._reply
	rec["reply_chars"] = String(_win._reply).length()
	rec["reply_words"] = String(_win._reply).split(" ", false).size()
	var fin := {}
	for e in _ev:
		if e["e"] == "finished":
			fin = e.get("info", {})
		if e["e"] == "failed":
			rec["failed"] = String(e["message"])
	rec["finish_reason"] = String(fin.get("finish_reason", ""))
	## The Rust usage event names its counts `prompt`, `completion` and `total`.
	var ctoks := int(u.get("completion", 0))
	rec["prompt_tokens"] = int(u.get("prompt", 0)) if u.has("prompt") else null
	rec["completion_tokens"] = ctoks if u.has("completion") else null
	if ctoks > 1 and rec["first_token_ms"] >= 0 and total > int(rec["first_token_ms"]):
		rec["tok_per_s"] = float(ctoks - 1) / (float(total - int(rec["first_token_ms"])) / 1000.0)
	rec["grounding"] = _win.last_grounding.duplicate(true) if _win._grounded_reply == _win._reply and _win._reply != "" else {}
	if fx.has("bundle") and bool(fx["bundle"].get("ok", false)):
		var facts_json: Dictionary = JSON.parse_string(String(fx["bundle"]["facts_json"]))
		rec["use"] = _use_report(String(_win._reply), facts_json, place)
	# The model's reply as the panel's own card text for the grounding card
	var g := _c("grounding")
	rec["grounding_card"] = (g as Label).text if g != null else ""
	rec["shot"] = await _jpg("%s_%s" % [tag, condition])
	_log(rec)


func _run_seed(seed_: int, reps: int) -> void:
	## The user's size (2048 x 1311 cells, 800 km) with the village add-on ON: without it the default world holds
	## about forty settlements and almost none has a neighbour within 25 km (measured in the smoke run), which
	## would leave the neighbour half of the facts untested. The engine path is the same `generate`.
	var t_gen := Time.get_ticks_msec()
	_bridge.generate({"seed": seed_, "width_km": 800.0, "grid_w": 2048, "grid_h": 1311, "archetype": "", "villages": true})
	await _frames(4)
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	var gen := {"generate_ms": Time.get_ticks_msec() - t_gen, "backend": str(_bridge.gpu_last_backend()), "villages": true}
	_bridge.civ_belief_run(60)
	await _frames(4)
	var ss: Array = _bridge.settlements()
	_log({"kind": "world", "seed": seed_, "generate": gen, "settlements": ss.size(), "grid": [_bridge.world_gen.get_width(), _bridge.world_gen.get_height()],
		"map_width_km": _bridge.world_gen.get_map_width_km(), "has_religion_keys": ss.size() > 0 and (ss[0] as Dictionary).has("religion")})
	var picks := _choose_places()
	_log({"kind": "places", "seed": seed_, "picks": picks.map(func(p): return {"index": p["index"], "name": String((ss[int(p["index"])] as Dictionary).name), "why": p["why"]})})
	if String(_o["only-notes"]) != "1":
		DccSettings.set_llm_value("ctx_notes", false)
		for k in picks.size():
			for rep in range(1, reps + 1):
				await _ask("no_notes", seed_, picks[k], rep, "s%d_p%d_r%d" % [seed_, k + 1, rep])
	# ---- with notes: a scratch Vault with synthetic notes for places 1 and 3 of this seed
	var vdir := String(_o["vault"])
	if vdir != "":
		for k in [0, 2]:
			var pk: Dictionary = picks[k]
			var place: Dictionary = ss[int(pk["index"])]
			var made := _make_notes(vdir, place, seed_)
			if not _vault_connected:
				var vi: Dictionary = _bridge.vault_connect(vdir, "LLM-11 live scratch vault")
				_vault_connected = bool(vi.get("ok", false))
			var att: Dictionary = _bridge.vault_attach("settlement", int(place.tid), String(place.name), "Places/%s.md" % String(place.name), "")
			_bridge.vault_refresh_backlinks()
			_log({"kind": "vault_setup", "seed": seed_, "place": String(place.name), "attached": bool(att.get("ok", false)), "notes": made["files"], "contradiction": made["contradiction"]})
			DccSettings.set_llm_value("ctx_notes", true)
			for rep in range(1, reps + 1):
				await _ask("with_notes", seed_, pk, rep, "s%d_p%d_r%d" % [seed_, k + 1, rep])
			DccSettings.set_llm_value("ctx_notes", false)


func _write(root: String, rel: String, text: String) -> void:
	var path := root.path_join(rel)
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


## Synthetic notes for `place`: its own note with canon and ONE deliberate contradiction with the data, a
## person, its faction, its largest neighbour, an unrelated note and a private-marked note.
func _make_notes(root: String, place: Dictionary, seed_: int) -> Dictionary:
	var name := String(place.name)
	var fac := ""
	for fr in _bridge.get_factions():
		if int(fr.id) == int(place.faction):
			fac = String(fr.name)
	var wall := ""
	for r in _bridge.civ_military_summary().settlements:
		if String((r as Dictionary).get("name", "")) == name:
			wall = String(r.get("wall_spec", ""))
	var claims_stone := wall != "stone"
	var contradiction := "the note says the walls are great stone walls; the data says walls: %s" % wall if claims_stone else "the note says the walls are a timber palisade; the data says walls: stone"
	var wall_text := "Its walls are great stone walls, raised after the Ash Winter." if claims_stone else "Its walls are only a timber palisade, never rebuilt since the Ash Winter."
	var ss: Array = _bridge.settlements()
	var ck: float = float(_bridge.world_gen.get_map_width_km()) / float(_bridge.world_gen.get_width())
	var nb := ""
	var nbpop := -1
	for s in ss:
		var d := sqrt(pow(float(s.x) - float(place.x), 2) + pow(float(s.y) - float(place.y), 2)) * ck
		if d <= 25.0 and String(s.name) != name and int(s.population) > nbpop:
			nb = String(s.name)
			nbpop = int(s.population)
	var files := []
	var fname := "Places/%s.md" % name
	var body := "# %s\nThe founders were refugees from the drowned coast; they called the first street Lantern Row. The harbourmaster is [[Mara Venn]], who answers to [[%s]]. %s The old shrine keeps a bronze bell that is rung at dusk.\n#harbour\n" % [name, fac, wall_text]
	_write(root, fname, body)
	files.append(fname)
	_write(root, "People/Mara Venn.md", "# Mara Venn\nHarbourmaster of %s. She keeps the tide tables and distrusts the Lantern Row guild.\n" % name)
	files.append("People/Mara Venn.md")
	if fac != "":
		_write(root, "Factions/%s.md" % fac, "# %s\nA stubborn realm that prizes ferrymen above knights.\n" % fac)
		files.append("Factions/%s.md" % fac)
	if nb != "":
		_write(root, "Places/%s.md" % nb, "# %s\nA mill town whose grain feeds %s in lean years.\n" % [nb, name])
		files.append("Places/%s.md" % nb)
	_write(root, "Random/Grocery list.md", "# Grocery list\neggs, flour, UNRELATEDWORD\n")
	_write(root, "Places/Secret plot.md", "---\nprivate: true\n---\n# Secret plot\nThe harbourmaster is a spy for %s. SECRETPLOTWORD.\n" % nb)
	return {"files": files, "contradiction": contradiction}


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a.begins_with("--") and _o.has(a.substr(2)) and i + 1 < args.size():
			_o[a.substr(2)] = args[i + 1]
			i += 2
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
	if String(_o["arm"]) == "" or String(_o["model"]) == "":
		print("ABORT: --arm and --model are required")
		get_tree().quit(2)
		return
	get_tree().create_timer(float(_o["watchdog"])).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if String(_o["shots"]) != "":
		DirAccess.make_dir_recursive_absolute(String(_o["shots"]))
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.5).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(8)
	_bridge = _app.bridge
	_win = _app.assistant_window
	_bridge.generation_started.connect(func(): _gen_started += 1)
	if not _bridge.llm_api or _win == null:
		print("RESULT: CANNOT RUN: no LlmBridge or no assistant panel")
		get_tree().quit(1)
		return
	_win.open()
	await _frames(6)
	_hook_run()
	var cfg := _configure()
	_log({"kind": "config", "arm": String(_o["arm"]), "preset": String(_o["preset"]), "model": String(_o["model"]), "cfg": cfg,
		"request": REQUEST, "build": ProjectSettings.get_setting("application/config/name")})
	for sd in String(_o["seeds"]).split(","):
		await _run_seed(int(sd), int(_o["reps"]))
	print("RESULT: DONE arm=%s records=%d" % [String(_o["arm"]), _records])
	get_tree().quit(0)
