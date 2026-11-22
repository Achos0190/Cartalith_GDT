extends Node
## **F12 (`TOOLING_PLAN.md`, 2026-11-22): the Edit region's new facts and the card's revise button, in the real window.**
## Windowed, one form per launch (desktop 1600x900; tablet `--resolution 1280x800 -- --force-touch`; phone
## `--resolution 393x852 -- --force-touch`), under a scratch APPDATA, with NO model behind the endpoint (the settings point at a closed
## port): what is checked is what the app builds and shows, not what a model says.
##
##   godot --path . res://_f12_card_probe.tscn -- --shots <dir> --tag <name> [--mutate no_stroke|no_profile|no_peakfix]
##
## Checks: (1) the engine's facts for a painted stroke carry `stroke`, `axis`, an 8-slice `profile` and the world's
## `highest_m`; (2) the text the model would read (`LlmStream.edit_preview`, the same Rust that builds the request) states the
## region kind and geometry, the profile and the ceiling, and no longer says "peak altitude is"; (3) an Edit card drawn for a
## proposal has an "Ask the model to revise" button of touch size, and pressing it (a real `pressed`) reaches Rust, which
## refuses because no request is open, and the card stays open with the reason; (4) a card with nothing valid offers "Ask the
## model to fix this". `--mutate` removes one fact; the matching check must go red (exit 1). Exit 0 held, 1 a check failed,
## 2 could not run.

const REGION_SCRIPT := preload("res://shell/llm_region.gd")
const EDIT_SCRIPT := preload("res://shell/llm_edit.gd")

var _o := {"shots": "", "tag": "run", "mutate": "", "seed": "9137", "vp": "", "density": ""}
var _app: Node
var _bridge
var _win
var _fails: Array = []
var _checks := 0


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _check(name: String, ok: bool, detail: String = "") -> void:
	_checks += 1
	print("%s %s %s" % ["PASS" if ok else "FAIL", name, detail.left(300)])
	if not ok:
		_fails.append(name)


func _paint_stroke() -> void:
	REGION_SCRIPT.clear(_app)
	REGION_SCRIPT.install(_app)
	REGION_SCRIPT.painting = true
	REGION_SCRIPT.brush = 16.0
	var drag: Callable = _app._drag_handlers[REGION_SCRIPT.TOOL_ID]
	var rel: Callable = _app._release_handlers[REGION_SCRIPT.TOOL_ID]
	## A diagonal stroke across the middle of the 256 x 164 world: 12 dabs, 6 cells in x and 3 in y apart.
	for k in 12:
		drag.call(float(80 + k * 6), float(60 + k * 3))
	rel.call(float(146), float(93), true)
	REGION_SCRIPT.painting = false


func _all_buttons(n: Node, out: Array) -> void:
	if n is Button:
		out.append(n)
	for c in n.get_children(true):
		_all_buttons(c, out)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a.begins_with("--") and _o.has(a.substr(2)) and i + 1 < args.size():
			i += 1
			_o[a.substr(2)] = args[i]
		elif a == "--force-touch":
			pass
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		i += 1
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed")
		get_tree().quit(2)
		return
	get_tree().create_timer(300.0).timeout.connect(func() -> void:
		print("WATCHDOG")
		get_tree().quit(2))
	if String(_o["shots"]) != "":
		DirAccess.make_dir_recursive_absolute(_o["shots"])
	DccSettings._ensure_loaded()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_bridge = _app.bridge
	_win = _app.assistant_window
	_bridge.generate({"seed": int(_o["seed"]), "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": false})
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	var origin := "http://127.0.0.1:9"
	for k in {"enabled": true, "local_allowed": true, "preset_id": "lm_studio", "base_url": origin + "/v1", "model": "none", "thinking": "none", "edit": true, "ctx_region": true, "conv_save_asked": true}:
		DccSettings.set_llm_value(k, {"enabled": true, "local_allowed": true, "preset_id": "lm_studio", "base_url": origin + "/v1", "model": "none", "thinking": "none", "edit": true, "ctx_region": true, "conv_save_asked": true}[k])
	_paint_stroke()
	_win.open()
	await _settle(6)
	_win._pick_task("edit")
	await _settle(6)

	# (1) the engine's facts.
	var d: Dictionary = _bridge.region_describe(REGION_SCRIPT.spec())
	var facts: Dictionary = REGION_SCRIPT.facts(_bridge)
	var region: Dictionary = facts.get("region", {})
	if _o["mutate"] == "no_stroke":
		region.erase("stroke")
	if _o["mutate"] == "no_profile":
		region.erase("profile")
	if _o["mutate"] == "no_peakfix":
		facts["world"].erase("highest_m")
	var st: Dictionary = region.get("stroke", {})
	var prof: Array = region.get("profile", [])
	_check("facts.stroke_length_km", st.has("length_km") and absf(float(st["length_km"]) - float(st.get("length_cells", 0.0)) * 1200.0 / 256.0) < 0.5 and float(st["length_cells"]) > 90.0 and float(st["length_cells"]) < 110.0, str(st))
	_check("facts.stroke_width_is_brush_diameter", absf(float(st.get("width_cells", 0.0)) - 32.0) < 0.01, str(st))
	_check("facts.profile_eight_slices", prof.size() == 8 and prof.all(func(v): return v == null or typeof(v) == TYPE_FLOAT), str(prof))
	_check("facts.axis_unit_vector", region.has("axis") and absf((Vector2(float(region["axis"][0]), float(region["axis"][1]))).length() - 1.0) < 0.01, str(region.get("axis")))
	var hi := float(facts.get("world", {}).get("highest_m", -1.0e9))
	_check("facts.world_highest_is_the_real_maximum", hi >= float(d["ground"]["max_m"]) - 0.01 and hi > 0.0, "highest %s region max %s" % [hi, d["ground"]["max_m"]])
	# (2) the text the model reads.
	var ep: Dictionary = _win.run.stream.edit_preview({"request": "Raise the area I painted into a ridge about 300 m high.", "edit": JSON.stringify(facts)})
	var t := String(ep.get("region_text", ""))
	_check("text.kind_and_geometry", t.contains("Region kind: a painted stroke (a path of dabs),") and t.contains("km long, about") and t.contains(" to "), t.substr(t.find("Region kind"), 160) if t.contains("Region kind") else t.left(200))
	_check("text.profile_line", t.contains("Ground profile, mean metres in 8 equal slices running"), t.right(300))
	_check("text.ceiling_is_not_contradictory", t.contains("Raising stops at") and t.contains("The highest cell anywhere in this world is") and not t.contains("peak altitude"), "")

	# (3) the card and its revise button.
	var verdict := {"valid": [{"operation": "ridge", "amount_m": 300.0, "falloff": "smooth", "softness": 1.0, "defaulted": false, "text": "Make a ridge along the stroke, up to 300 m"}], "rejected": [], "summary": "A ridge"}
	_win._on_edit_proposal({"edit": verdict})
	await _settle(8)
	var card: Dictionary = _win.cards[_win.cards.size() - 1] if not _win.cards.is_empty() else {}
	var rv = card.get("revise")
	_check("card.drawn_with_revise_button", rv != null and is_instance_valid(rv) and (rv as Button).text == "Ask the model to revise", str(rv))
	if rv != null and is_instance_valid(rv):
		var b := rv as Button
		var touch := DccTheme.is_touch()
		_check("card.revise_button_enabled_and_sized", not b.disabled and b.is_visible_in_tree() and b.size.y >= (44.0 if touch else 20.0) - 0.5, "size %s touch %s" % [b.size, touch])
		await _shot("card")
		b.pressed.emit()
		await _settle(6)
		var rl: Label = card["reason"]
		_check("card.revise_refused_without_a_request_and_stays_open", String(card["state"]) == "new" and rl.visible and rl.text.contains("no edit request"), "state %s reason '%s'" % [card["state"], rl.text])
	# (4) a card with nothing valid.
	_win._on_edit_proposal({"edit": {"valid": [], "rejected": [{"operation": "raise", "rule": "R7", "why": "raise does not take strength"}]}})
	await _settle(8)
	var card2: Dictionary = _win.cards[_win.cards.size() - 1]
	var rv2 = card2.get("revise")
	_check("card.nothing_valid_offers_fix", rv2 != null and is_instance_valid(rv2) and (rv2 as Button).text == "Ask the model to fix this", str(rv2))
	# (4b) a revised proposal replaces the card it revised (the card stays open until then). The `revising` flag is what
	# `_revise_edit_card` sets once Rust grants the revise; no model is behind this probe, so it is set here, and the new
	# proposal arrives through the same handler the run's signal calls.
	card["revising"] = true
	_win._on_edit_proposal({"edit": verdict})
	await _settle(8)
	_check("card.revised_proposal_replaces_the_revised_card", String(card["state"]) == "revised" and String((_win.cards[_win.cards.size() - 1])["state"]) == "new", "old state %s" % card["state"])
	# (5) the preview handed back is numbers only.
	var pj: String = EDIT_SCRIPT.preview_json(card.get("preview", {}))
	_check("preview_json_is_numbers_only", pj.contains("\"changed\"") and not pj.contains("thumb") and not pj.contains("\"ok\""), pj.left(200))
	await _shot("cards")
	print("RESULT: %s checks=%d fails=%s" % ["HELD" if _fails.is_empty() else "FAILED", _checks, str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


func _shot(tag: String) -> void:
	if String(_o["shots"]) == "":
		return
	await _settle(4)
	get_viewport().get_texture().get_image().save_png(String(_o["shots"]).path_join("f12_%s_%s.png" % [_o["tag"], tag]))
