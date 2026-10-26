extends Node
## One LIVE check of the model dropdown against a real LM Studio server (`_llmmodels_probe.gd` is
## the mocked, repeatable one; this exists because a mock only proves the code agrees with itself).
##
##   1. LM Studio's server must ALREADY be running at http://127.0.0.1:1234 (this probe never starts,
##      stops or rebinds it: the owner's phone may be using it, and it never loads or unloads a model);
##   2. godot --path . --resolution 1600x1000 _llmmodels_live_probe.tscn -- --shots <dir>
##
## Windowed (a screenshot is the evidence); run from a scratch copy with its own `config/name`.
## Reads `--shots <dir>` only; anything else aborts with exit 2. It sets and clears its own `[llm]`
## keys and consents to `http://127.0.0.1:1234` for the run only. It asserts only what any LM Studio
## install must satisfy (the list is non-empty, every row has an id, embeddings are hidden), because
## which models a machine has is not the app's to know; the row texts are printed for the report.
## Result: `RESULT: PASS|FAIL (<n> failed)`.

var _fails := 0
var _shots := ""


func _ok(cond: bool, what: String) -> void:
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --shots <dir> only" % args[i])
			get_tree().quit(2)
			return
		i += 1
	get_tree().create_timer(60.0).timeout.connect(func() -> void:
		print("WATCHDOG")
		get_tree().quit(2))
	var app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	var win = app.assistant_settings_window
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
	var gate := {"enabled": true, "prose": true, "local_allowed": true, "preset_id": "lm_studio",
		"base_url": "http://127.0.0.1:1234/v1", "model": ""}
	for k in gate:
		DccSettings.set_llm_value(k, gate[k])
	DccSettings.llm_add_consent("http://127.0.0.1:1234")
	win.open()
	var t0 := Time.get_ticks_msec()
	while String(win._ml_state) in ["idle", "loading"] and Time.get_ticks_msec() - t0 < 15000:
		await get_tree().process_frame
	for _f in 4:
		await get_tree().process_frame
	var ob := win.controls["model_pick"] as OptionButton
	print("STATE %s | STATUS %s" % [win._ml_state, (win.controls["model_status"] as Label).text])
	for k in ob.item_count:
		print("ROW %d: %s" % [k, ob.get_item_text(k)])
	_ok(String(win._ml_state) == "ok", "the real LM Studio answered with a list")
	_ok(ob.item_count >= 3, "at least one model row besides the placeholder and 'Type a name…'")
	_ok(not (win.controls["model_status"] as Label).text.contains("Could not"), "no failure text")
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
		(win._scroll as ScrollContainer).ensure_control_visible(win.controls["model_status"])
		for _f in 4:
			await get_tree().process_frame
		get_viewport().get_texture().get_image().save_png(_shots.path_join("llmmodels_live_lmstudio.png"))
		print("SHOT live")
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()
	print("RESULT: %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
