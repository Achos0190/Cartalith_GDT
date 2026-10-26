extends Node
## Shared harness for PW-3 of `POPOUT_WINDOWS_SCOPE.md` (Ruling DK): the six civilisation / Vault /
## Assistant windows as native, transient, non-exclusive windows reached by the global setting AND by
## each window's own "Pop out / Dock" button, against the real shell, the real `PopoutHost`,
## `PopoutKeyForwarder` and `BusyOverlay`. **One table of window specs drives every bar** -- the PW-2
## roster probe (`_popout_roster_probe.gd`) is the model, and its bars (a)-(g) are re-run here for each
## window by the same helpers rather than copied six times (scope PW-3: "by a shared harness, not copied
## probes").
##
##   desktop: godot --path . --resolution 1400x900 _popout_windows_probe.tscn -- --form desktop [--only a,b] [--shots <dir>]
##   tablet:  godot --path . --resolution 1280x800 _popout_windows_probe.tscn -- --form tablet --force-touch
##   phone:   godot --path . --resolution 393x852  _popout_windows_probe.tscn -- --form phone  --force-touch
##
## **Windowed, never `--headless`** (it opens real native windows, reads their textures and compares
## frames). Arguments: `--form desktop|tablet|phone` (required; the probe checks the shell booted into
## that form), `--only <sections>` (comma list of: tables, stay, state, keys, popups, busy, setting,
## hashes, opener, assistant, orphan; empty = all -- the mutation runs use it), `--shots <dir>`,
## `--force-touch` (the shell's own flag) and `--child <dir>` (internal: the bar-d child process).
## Anything else aborts, exit 2. **Run from a scratch worktree with its own `config/name` and a wiped
## user dir**; section 0 fails if it was not clean. One monitor is assumed: only the single-monitor
## halves of the bars are claimed, and the result lists what needs two monitors.
##
## What it protects (every check names it):
##   tables    the literal `LIVE_KEYS`; `opener_rule()` and `centred_in()` by literal rows; the META_MOVING
##             flag is true inside a move's hide and false inside a plain hide.
##   stay      with the setting ON the six windows the scope keeps embedded (Data Manager, Asset Library,
##             Travel library, World data, the assistant's settings, the quick-bar customiser) open
##             embedded, with no button and no extra OS window.
##   state     (g) per window, setting OFF then ON: selection, tab, scroll positions and half-typed
##             text survive Pop out and Dock, by `toggle()` and by the real button; the button and the
##             setting agree; the Vault's unsaved-edit guard neither asks nor loses the edit on a move.
##   keys      (b) Ctrl+Z, Ctrl+Y, Ctrl+Shift+Z and Ctrl+S sent to each native window do to the app what
##             the main window does; a focused text field keeps its own Ctrl+Z.
##   popups    (e) a dropdown and a colour popup of a native window open embedded in it and visible.
##   busy      (a) with all seven windows native at once, a generate mutes and mirrors each one (a click
##             reaches 0 controls, the mirror changes pixels), a dock and a re-pop DURING the run stay
##             muted, and input returns afterwards.
##   setting   the real Preferences row turns the setting on and off, clears manual choices.
##   hashes    (c) a scripted edit through three windows leaves the same state hash docked and native on 3
##             seeds, and a world generated with all windows native hashes equal to a docked one.
##   opener    a window opened from a native window places itself on the opener's screen (the rule table
##             and the live one-screen case; the two-monitor half is owner-run).
##   assistant the Assistant native, against the REAL LM Studio qwen3-14b (owner 2026-10-26: no mock servers in new
##             probes): Send, streaming, a move mid-stream, Stop, Close mid-stream, the off-gate, the busy
##             mute, and a Commands card with Apply and one undo IF the model proposes one (else NOT RUN).
##   orphan    (d) a child process that opened all six natively and quit its main tree exits.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument/watchdog.

const SEEDS := [483920, 24601, 71077345]
const GRID_SMALL := Vector2i(160, 120)
const ENV_VAR := "CART_PW3_PROBE_KEYVAR"
const SECRET := "sk-PROBE-SECRET-pw3-0123456789"
const VAULT_DIR := "user://pw3_probe_vault"
## The seven v1 keys, in the scope's order, written out here so the table pins the constant against a
## literal and not against itself.
const LIVE := ["faction_roster_window", "settlement_types_window", "place_editor_window",
	"generation_rules_window", "city_viewer_window", "vault_window", "llm_assistant_window"]

var _fails := 0
var _checks := 0
var _form := ""
var _shots := ""
var _child := ""
var _only := ""
var _app: Node
var _bridge
var _specs: Array = []
var _all_native_hash := ""


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame


func _until(cond: Callable, ms: int = 10000) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


func _nwin() -> int:
	return DisplayServer.get_window_list().size()


func _finish() -> void:
	OS.unset_environment(ENV_VAR)
	LlmWire.shutdown_all()
	print("RESULT: %s (%d failed, %d checks) form=%s" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks, _form])
	get_tree().quit(0 if _fails == 0 else 1)


func _want(name: String) -> bool:
	return _only == "" or (name in _only.split(","))


func _shot(img_src: Object, tag: String) -> void:
	if _shots == "":
		return
	var img: Image = null
	if img_src is Window:
		img = (img_src as Window).get_texture().get_image()
	elif img_src is Viewport:
		img = (img_src as Viewport).get_texture().get_image()
	if img != null:
		img.save_png(_shots.path_join("pw3_%s.png" % tag))


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a in ["--form", "--shots", "--child", "--only"] and i + 1 < args.size():
			match a:
				"--form": _form = String(args[i + 1])
				"--shots": _shots = String(args[i + 1])
				"--child": _child = String(args[i + 1])
				"--only": _only = String(args[i + 1])
			i += 2
		elif a == "--force-touch":
			i += 1
		else:
			print("ABORT: unknown argument %s" % a)
			get_tree().quit(2)
			return
	if not _form in ["desktop", "tablet", "phone"]:
		print("ABORT: --form desktop|tablet|phone is required")
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed; this probe opens native windows and reads frames")
		get_tree().quit(2)
		return
	get_tree().create_timer(2400.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		LlmWire.shutdown_all()
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	OS.set_environment(ENV_VAR, SECRET)
	if _child != "":
		await _child_main()
		return
	print("== 0. clean user dir ==")
	_ok(not FileAccess.file_exists(ProjectSettings.globalize_path(DccSettings.CONFIG_PATH)), "no cartalith_settings.cfg at start (the user dir was wiped)")
	await _boot()
	if _form != "desktop":
		await _forms_other()
		_finish()
		return
	if _want("tables"):
		_tables()
	await _gen(SEEDS[0], GRID_SMALL)
	_connect_vault()
	if _want("stay"):
		await _stay_embedded()
	if _want("state"):
		await _state_all()
	if _want("keys"):
		await _keys_all()
	if _want("popups"):
		await _popups_all()
	if _want("busy"):
		await _busy()
	if _want("setting"):
		await _setting_row()
	if _want("hashes"):
		await _hashes()
	if _want("opener"):
		await _opener()
	if _want("assistant"):
		await _assistant()
	if _want("orphan"):
		await _orphan()
	_finish()


func _boot() -> void:
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	var got := "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_ok(got == _form, "the shell booted into the %s form (got %s)" % [_form, got])
	_bridge = _app.bridge
	_build_specs()


func _gen(seed: int, grid: Vector2i, poll: Callable = Callable()) -> void:
	_bridge.generate({"seed": seed, "width_km": 1200.0, "grid_w": grid.x, "grid_h": grid.y, "archetype": "", "villages": true})
	var polls := 0
	while _bridge.generating:
		await get_tree().create_timer(0.1).timeout
		polls += 1
		if poll.is_valid():
			await poll.call(polls)
	await _settle(10)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()


## A probe Vault with two notes, so the Vault window has a tree, a picked note and an editor.
func _connect_vault() -> void:
	var dir := ProjectSettings.globalize_path(VAULT_DIR)
	DirAccess.make_dir_recursive_absolute(dir)
	for e in [["Veldmark.md", "# Veldmark\nMy own notes about the port.\n"], ["Other.md", "# Other\nA second note.\n"]]:
		var f := FileAccess.open(dir.path_join(e[0]), FileAccess.WRITE)
		f.store_string(e[1])
		f.close()
	var info: Dictionary = _bridge.vault_connect(dir, "PW3 probe")
	_ok(bool(info.get("ok", false)), "a probe Vault is connected: %s" % str(info.get("error", "")))


# -- the window table ---------------------------------------------------------------------------

## One row per window the scope puts in PW-3 (the roster is PW-2's, kept in the busy and hash bars as a
## control). `open` calls the window's own opener (never `PopoutHost`), `mutate` is the window-specific
## state change a move must keep, `snap` its window-specific readout. `win` is the live node.
func _build_specs() -> void:
	_specs = [
		{"name": "settlement types", "key": "settlement_types_window", "win": _app.settlement_types_window,
			"open": func() -> void: _app.settlement_types_window.open(), "mutate": _mut_types, "snap": _snap_types, "text": true},
		{"name": "place editor", "key": "place_editor_window", "win": _app.place_editor_window,
			"open": func() -> void: _app.place_editor_window.open_for(0), "mutate": _mut_place, "snap": _snap_place, "text": true},
		{"name": "generation rules", "key": "generation_rules_window", "win": _app.generation_rules_window,
			"open": func() -> void: _app.generation_rules_window.open(), "mutate": _mut_rules, "snap": _snap_rules, "text": false},
		{"name": "city viewer", "key": "city_viewer_window", "win": _app.city_viewer_window,
			"open": func() -> void: _app.city_viewer_window.open(0), "mutate": _mut_city, "snap": _snap_city, "text": false},
		{"name": "vault", "key": "vault_window", "win": _app.vault_window,
			"open": func() -> void: _app.vault_window.open_browse(), "mutate": _mut_vault, "snap": _snap_vault, "text": true},
		{"name": "assistant", "key": "llm_assistant_window", "win": _app.assistant_window,
			"open": func() -> void: _app.assistant_window.open(), "mutate": _mut_assistant, "snap": _snap_assistant, "text": true},
	]


func _roster_open() -> void:
	_app.faction_roster_window.open(1, "identity")


func _mut_types(w: Window) -> void:
	## The library may hold fewer than two types (a fresh profile holds none): make sure there are two, then
	## select the one that is NOT selected now, so the change is real on every call.
	while SettlementTypeStore.types().size() < 2:
		SettlementTypeStore.new_type("PW3 type %d" % SettlementTypeStore.types().size())
	for t in SettlementTypeStore.types():
		if String((t as Dictionary).get("id", "")) != w._selected_id:
			w._selected_id = String((t as Dictionary).get("id", ""))
			break
	w._rebuild()
	await _settle(3)


func _snap_types(w: Window) -> Dictionary:
	return {"sel": w._selected_id}


## The place editor's tab is checked by `_place_tab()` (a tab switch rebuilds the form and drops a half-typed
## field, by design, so the two states cannot be set up together); here the half-typed name field and the
## scroll are the state, set by the generic mutation on the Overview tab.
func _mut_place(w: Window) -> void:
	w._switch_tab("overview")
	await _settle(4)


func _snap_place(w: Window) -> Dictionary:
	return {"index": w._index, "tab": w._active_tab}


func _mut_rules(w: Window) -> void:
	var k := 2 if w._active_preset == String(w._PRESETS[1].label) else 1
	w._apply_preset(w._PRESETS[k])
	await _settle(4)


func _snap_rules(w: Window) -> Dictionary:
	return {"preset": w._active_preset, "rules": JSON.stringify(_bridge.get_active_urban_rules())}


func _mut_city(w: Window) -> void:
	if w._picker.item_count > 1:
		w._picker.select(1)
		w._picker.item_selected.emit(1)
	await _settle(4)


func _snap_city(w: Window) -> Dictionary:
	return {"index": w._index, "picked": w._picker.selected}


func _mut_vault(w: Window) -> void:
	w._select_file("Veldmark.md")
	await _settle(6)


func _snap_vault(w: Window) -> Dictionary:
	return {"pick": w._pick_file, "kind": w._kind, "browse": w._browse_only, "path": w._browse_path, "text": w._browse_text, "saved": w._browse_saved}


func _mut_assistant(w: Window) -> void:
	w._pick_task("story")
	await _settle(4)


func _snap_assistant(w: Window) -> Dictionary:
	return {"task": w._task, "cards": w.cards.size(), "reply": String(w._reply_label.text) if w._reply_label != null else ""}


## Every `LineEdit` / `TextEdit` text in `w`, by path: the half-typed fields a move must keep.
func _texts(w: Window) -> Array:
	var out: Array = []
	for n in w.find_children("*", "LineEdit", true, false):
		out.append([String(w.get_path_to(n)), (n as LineEdit).text])
	for n in w.find_children("*", "TextEdit", true, false):
		out.append([String(w.get_path_to(n)), (n as TextEdit).text])
	return out


func _selected_options(w: Window) -> Array:
	var out: Array = []
	for n in w.find_children("*", "OptionButton", true, false):
		out.append([String(w.get_path_to(n)), (n as OptionButton).selected])
	return out


func _snap(spec: Dictionary) -> Dictionary:
	var w: Window = spec["win"]
	return {"scrolls": PopoutHost.snapshot_scrolls(w), "texts": _texts(w), "opts": _selected_options(w), "extra": (spec["snap"] as Callable).call(w)}


## The generic half of a state change: the first scroller that has range is scrolled, the first visible
## editable text field gets a marker. Returns what it managed so the probe can say what a window offers.
func _mutate_generic(w: Window) -> Dictionary:
	var res := {"scrolled": false, "typed": false}
	for n in w.find_children("*", "ScrollContainer", true, false):
		var sc := n as ScrollContainer
		var bar := sc.get_v_scroll_bar()
		if bar.max_value - bar.page > 20.0 and sc.is_visible_in_tree():
			sc.scroll_vertical = mini(60, int(bar.max_value - bar.page))
			res["scrolled"] = true
			break
	await _settle(2)
	for n in w.find_children("*", "LineEdit", true, false):
		var le := n as LineEdit
		if le.editable and le.is_visible_in_tree():
			le.text = le.text + "~pw3"
			res["typed"] = true
			break
	if not res["typed"]:
		for n in w.find_children("*", "TextEdit", true, false):
			var te := n as TextEdit
			if te.editable and te.is_visible_in_tree():
				te.text = te.text + "~pw3"
				res["typed"] = true
				break
	await _settle(2)
	return res


func _btn(w: Window) -> Button:
	return w.find_child(PopoutHost.BUTTON_NAME, true, false) as Button


func _hide_all() -> void:
	for s in _specs:
		(s["win"] as Window).hide()
	_app.faction_roster_window.hide()
	await _settle(4)


# -- 1 ------------------------------------------------------------------------------------------

func _tables() -> void:
	print("== 1. tables ==")
	_ok(Array(PopoutHost.LIVE_KEYS) == LIVE, "LIVE_KEYS is exactly the seven v1 windows in the scope's order (literal)")
	for s in _specs:
		_ok(PopoutHost.key_for(s["win"]) == s["key"], "%s: the window's key is '%s'" % [s["name"], s["key"]])
		_ok(PopoutHost.wants_native(true, true, s["key"], PopoutHost.LIVE_KEYS), "%s: wants_native(setting on, platform ok) is true" % s["name"])
		_ok(not PopoutHost.wants_native(false, true, s["key"], PopoutHost.LIVE_KEYS), "%s: wants_native with the setting off is false" % s["name"])
	for pair in [["Data manager", _app.data_manager_window], ["Asset library", _app.asset_library_window], ["Travel library", _app.travel_library_window],
			["World data", _app.world_data_window], ["Assistant settings", _app.assistant_settings_window], ["Quick bar", _app.quick_bar_window]]:
		_ok(not LIVE.has(PopoutHost.key_for(pair[1])) and not PopoutHost.wants_native(true, true, PopoutHost.key_for(pair[1]), PopoutHost.LIVE_KEYS), "%s ('%s') is NOT live: it stays embedded (scope Q4)" % [pair[0], PopoutHost.key_for(pair[1])])
	# opener_rule(placed, stored_screen, opener, current)
	for r in [[false, -1, -1, 0, "engine"], [true, 0, -1, 0, "stored"], [true, 1, -1, 0, "stored"],
			[false, -1, 1, 0, "opener"], [false, -1, 1, 1, "engine"], [true, 1, 1, 0, "stored"],
			[true, 0, 1, 0, "opener"], [true, 0, 1, 1, "opener"], [false, -1, 0, 0, "engine"], [true, 0, 0, 1, "stored"]]:
		var got: String = PopoutHost.opener_rule(r[0], r[1], r[2], r[3])
		_ok(got == r[4], "opener_rule(placed=%s, stored_screen=%d, opener=%d, current=%d) == '%s' (got '%s')" % [r[0], r[1], r[2], r[3], r[4], got])
	# centred_in(usable, size)
	for r in [[Rect2i(1920, 0, 1920, 1040), Vector2i(880, 620), Rect2i(2440, 226, 880, 620)],
			[Rect2i(0, 0, 800, 600), Vector2i(2000, 2000), Rect2i(0, 32, 800, 568)],
			[Rect2i(-1280, 100, 1280, 1024), Vector2i(500, 400), Rect2i(-890, 428, 500, 400)],
			[Rect2i(0, 0, 1680, 1002), Vector2i(560, 800), Rect2i(560, 32 + (1002 - 32 - 800) / 2, 560, 800)]]:
		_ok(PopoutHost.centred_in(r[0], r[1]) == r[2], "centred_in(%s, %s) == %s (got %s)" % [str(r[0]), str(r[1]), str(r[2]), str(PopoutHost.centred_in(r[0], r[1]))])
	_ok(PopoutHost.native_windows(get_tree()).is_empty(), "no native window at boot")
	_ok(not PopoutHost.is_moving(_app.assistant_window), "is_moving() is false at boot")
	# META_MOVING is true inside a move's hide and false inside a plain hide
	var seen := {"move": [], "plain": []}
	var w: Window = _app.settlement_types_window
	DccSettings.set_popout_enabled(false)
	w.open()
	await _settle(6)
	var cb := func() -> void:
		if not w.visible:
			(seen["move"] if seen.get("mode", "") == "move" else seen["plain"]).append(PopoutHost.is_moving(w))
	w.visibility_changed.connect(cb)
	seen["mode"] = "move"
	await PopoutHost.toggle(w)
	await _settle(4)
	seen["mode"] = "plain"
	w.hide()
	await _settle(3)
	w.visibility_changed.disconnect(cb)
	_ok(seen["move"] == [true], "META_MOVING is true inside the hide a move makes (saw %s)" % str(seen["move"]))
	_ok(seen["plain"] == [false], "and false inside a plain hide (saw %s)" % str(seen["plain"]))
	_ok(not PopoutHost.is_moving(w), "and erased once the move is over")
	PopoutHost.forget_choices(get_tree())


func _forms_other() -> void:
	print("== 2'. %s: no pop-out surface at all ==" % _form)
	_ok(not PopoutHost.supported(), "PopoutHost.supported() is false on the %s" % _form)
	await _gen(SEEDS[0], GRID_SMALL)
	DccSettings.set_popout_enabled(true)
	for s in _specs:
		var w: Window = s["win"]
		_ok(_btn(w) == null, "%s has no PopoutButton on the %s" % [s["name"], _form])
		_ok(PopoutHost.add_toggle_button(w) == null, "%s: add_toggle_button() adds nothing" % s["name"])
		(s["open"] as Callable).call()
		await _settle(8)
		_ok(w.visible and (w.is_embedded() or w.borderless or true) and not w.force_native and _nwin() == 1, "%s opens embedded with the setting ON (window count %d, force_native %s)" % [s["name"], _nwin(), str(w.force_native)])
		w.hide()
		await _settle(3)
	DccSettings.set_popout_enabled(false)


# -- 2 ------------------------------------------------------------------------------------------

func _stay_embedded() -> void:
	print("== 2. the windows the scope keeps embedded stay embedded with the setting ON ==")
	DccSettings.set_popout_enabled(true)
	PopoutHost.forget_choices(get_tree())
	var rows := [
		["data manager", _app.data_manager_window, func() -> void: _app.open_data_manager()],
		["asset library", _app.asset_library_window, func() -> void: _app.open_asset_library()],
		["travel library", _app.travel_library_window, func() -> void: _app.open_travel_library()],
		["world data", _app.world_data_window, func() -> void: _app.open_world_data()],
		["assistant settings", _app.assistant_settings_window, func() -> void: _app.open_assistant_settings()],
		["quick bar customise", _app.quick_bar_window, func() -> void: _app.open_quick_bar_window()],
	]
	for r in rows:
		var w: Window = r[1]
		(r[2] as Callable).call()
		await _settle(8)
		_ok(w.visible and w.is_embedded() and not w.force_native and _nwin() == 1, "%s: opens embedded, force_native false, OS window count %d" % [r[0], _nwin()])
		_ok(w.find_child(PopoutHost.BUTTON_NAME, true, false) == null, "%s: carries no Pop out button" % r[0])
		w.hide()
		await _settle(3)
	DccSettings.set_popout_enabled(false)


# -- 3 ------------------------------------------------------------------------------------------

func _state_all() -> void:
	print("== 3. state across a move (g), per window ==")
	for s in _specs:
		await _state_one(s)


func _state_one(spec: Dictionary) -> void:
	var w: Window = spec["win"]
	var nm: String = spec["name"]
	DccSettings.set_popout_enabled(false)
	PopoutHost.forget_choices(get_tree())
	await _hide_all()
	(spec["open"] as Callable).call()
	await _settle(12)
	_ok(w.visible and w.is_embedded() and not w.force_native and _nwin() == 1, "%s, setting OFF: opens embedded, count 1" % nm)
	var b := _btn(w)
	_ok(b != null and b.text == "Pop out" and b.visible, "%s: a 'Pop out' button exists beside Close" % nm)
	var bal := w.find_child(PopoutHost.BUTTON_NAME + "Balance", true, false) as Control
	if spec["key"] == "generation_rules_window":
		## This window hides the dialog's Close, so its button lives in its own header beside its own Close and
		## needs no balance control (`PopoutHost.add_toggle_button`'s `header_row` note).
		_ok(bal == null and b != null and b.get_parent() is HBoxContainer and b.get_parent() != w.get_ok_button().get_parent(), "%s: the button is in the window's own header and there is no balance control" % nm)
	else:
		_ok(bal != null and b != null and is_equal_approx(bal.custom_minimum_size.x, b.size.x), "%s: the balance is as wide as the button's laid-out width (%s vs %s)" % [nm, str(bal.custom_minimum_size.x if bal else -1), str(b.size.x if b else -1)])
	var before := await _snap_after_change(spec)
	var g: Dictionary = before["gen"]
	print("  %s: generic change %s, snapshot scrolls=%d texts=%d options=%d" % [nm, str(g), (before["snap"]["scrolls"] as Array).size(), (before["snap"]["texts"] as Array).size(), (before["snap"]["opts"] as Array).size()])
	if bool(spec["text"]):
		_ok(bool(g["typed"]), "%s: a half-typed field was set up" % nm)
	await PopoutHost.toggle(w)
	await _settle(8)
	_ok(not w.is_embedded() and w.visible and _nwin() == 2, "%s: the button popped it out: native, window count %d" % [nm, _nwin()])
	_ok(w.transient and not w.exclusive, "%s: transient and non-exclusive" % nm)
	_ok(_btn(w).text == "Dock", "%s: the button now reads 'Dock'" % nm)
	_ok(w.get_node_or_null(PopoutHost.FORWARDER_NAME) != null, "%s: native, with its key forwarder child" % nm)
	_ok(PopoutHost.native_windows(get_tree()) == [w], "%s: native_windows() lists exactly this window" % nm)
	var after: Dictionary = _snap(spec)
	_ok(after == before["snap"], "%s: selection, tab, scrolls, options and typed text survived the move%s" % [nm, "" if after == before["snap"] else " -- before %s after %s" % [str(before["snap"]), str(after)]])
	await _shot_native(w, "window_" + String(spec["key"]))
	await PopoutHost.toggle(w)
	await _settle(8)
	_ok(w.is_embedded() and w.visible and _nwin() == 1, "%s: the button docked it back: embedded, count %d" % [nm, _nwin()])
	_ok(_btn(w).text == "Pop out", "%s: the button reads 'Pop out' again" % nm)
	var back: Dictionary = _snap(spec)
	_ok(back == before["snap"], "%s: state survived the move back%s" % [nm, "" if back == before["snap"] else " -- %s" % str(back)])
	_btn(w).pressed.emit()
	await _settle(14)
	_ok(not w.is_embedded() and _nwin() == 2, "%s: pressing the real button pops it out (count %d)" % [nm, _nwin()])
	_btn(w).pressed.emit()
	await _settle(14)
	_ok(w.is_embedded() and _nwin() == 1, "%s: pressing it again docks it (count %d)" % [nm, _nwin()])
	_ok(_snap(spec) == before["snap"], "%s: state survived the real-button round trip" % nm)
	if nm == "vault":
		await _vault_dirty(w)
	if nm == "place editor":
		await _place_tab(w)
	w.hide()
	await _settle(4)

	# the setting ON: pops out at open
	DccSettings.set_popout_enabled(true)
	PopoutHost.forget_choices(get_tree())
	(spec["open"] as Callable).call()
	await _settle(12)
	_ok(w.visible and not w.is_embedded() and _nwin() == 2, "%s, setting ON: open pops it out (count %d)" % [nm, _nwin()])
	_ok(_btn(w).text == "Dock", "%s: ...and the button reads 'Dock'" % nm)
	var before_on := await _snap_after_change(spec)
	await PopoutHost.toggle(w)
	await _settle(8)
	_ok(w.is_embedded() and w.visible, "%s: Dock with the setting ON: embedded" % nm)
	_ok(_snap(spec) == before_on["snap"], "%s: state survived Dock with the setting ON" % nm)
	w.hide()
	await _settle(3)
	(spec["open"] as Callable).call()
	await _settle(10)
	_ok(w.is_embedded(), "%s: reopened: the manual Dock holds for the session with the setting ON" % nm)
	w.hide()
	await _settle(3)
	PopoutHost.forget_choices(get_tree())
	(spec["open"] as Callable).call()
	await _settle(10)
	_ok(not w.is_embedded() and _nwin() == 2, "%s: once choices are forgotten it follows the setting again (native)" % nm)
	w.hide()
	await _settle(4)
	_ok(_nwin() == 1, "%s: hiding it leaves no OS window (count %d)" % [nm, _nwin()])
	DccSettings.set_popout_enabled(false)
	PopoutHost.forget_choices(get_tree())
	(spec["open"] as Callable).call()
	await _settle(10)
	_ok(w.is_embedded() and _nwin() == 1, "%s: setting off again, no choice: embedded, count 1" % nm)
	w.hide()
	await _settle(3)


## Makes the window-specific change, then the generic one, and returns the snapshot plus what the generic
## change managed. A positive control rides along: the snapshot after the change must differ from the one before.
func _snap_after_change(spec: Dictionary) -> Dictionary:
	var w: Window = spec["win"]
	var pre: Dictionary = _snap(spec)
	await (spec["mutate"] as Callable).call(w)
	var gen: Dictionary = await _mutate_generic(w)
	var post: Dictionary = _snap(spec)
	_ok(post != pre, "%s: control: the scripted change moved the snapshot, so equal snapshots after a move compare something" % spec["name"])
	return {"snap": post, "gen": gen}


## Native window's own texture, saved only when a shot directory was asked for.
func _shot_native(w: Window, tag: String) -> void:
	if _shots != "" and not w.is_embedded():
		w.get_texture().get_image().save_png(_shots.path_join("pw3_%s.png" % tag))


## The place editor's active tab survives a move: switch to Economy, pop out, dock, still Economy.
func _place_tab(w: Window) -> void:
	w._switch_tab("economy")
	await _settle(4)
	await PopoutHost.toggle(w)
	await _settle(8)
	_ok(not w.is_embedded() and w._active_tab == "economy", "place editor: the Economy tab is still the active tab after Pop out")
	await PopoutHost.toggle(w)
	await _settle(8)
	_ok(w.is_embedded() and w._active_tab == "economy", "place editor: and after Dock")


## A move with unsaved edits in the Vault: no "Unsaved edits" confirm appears, the edit is still unsaved,
## and the same window then CLOSED (not moved) still asks. The two halves are the bar: a guard that never
## asks would also pass the first half.
func _vault_dirty(w: Window) -> void:
	w._browse_path = "Veldmark.md"
	w._pick_file = "Veldmark.md"
	w._browse_saved = "a"
	w._browse_text = "b"
	_ok(w._is_dirty(), "vault: set up an unsaved edit (dirty)")
	var asks0 := _confirms()
	await PopoutHost.toggle(w)
	await _settle(8)
	_ok(not w.is_embedded() and w.visible, "vault: popped out with an unsaved edit")
	_ok(_confirms() == asks0 and w._is_dirty() and w._browse_text == "b", "vault: the move neither asked nor lost the unsaved edit (confirm dialogs %d -> %d, text '%s')" % [asks0, _confirms(), w._browse_text])
	await PopoutHost.toggle(w)
	await _settle(8)
	_ok(w.is_embedded() and w.visible and w._is_dirty(), "vault: docked again, still dirty")
	w.hide()
	await _settle(8)
	_ok(_confirms() > asks0 and w.visible, "vault: control: a real CLOSE with the edit still asks (confirm dialogs %d, window came back: %s)" % [_confirms(), str(w.visible)])
	_close_confirms()
	w._close_editor()
	await _settle(4)


func _confirms() -> int:
	var n := 0
	for c in _app.get_children():
		if c is ConfirmationDialog and (c as ConfirmationDialog).title == "Unsaved edits":
			n += 1
	return n


func _close_confirms() -> void:
	for c in _app.get_children():
		if c is ConfirmationDialog and (c as ConfirmationDialog).title == "Unsaved edits":
			(c as ConfirmationDialog).hide()
			c.queue_free()


# -- 4 ------------------------------------------------------------------------------------------

func _key(code: int, ctrl: bool, shift: bool) -> InputEventKey:
	var e := InputEventKey.new()
	e.keycode = code
	e.physical_keycode = code
	e.ctrl_pressed = ctrl
	e.shift_pressed = shift
	e.pressed = true
	return e


func _hist() -> Array:
	return [_app.look_history.can_undo(), _app.look_history.can_redo(), _app.look_history.cursor]


func _reset_history() -> void:
	var lh = _app.look_history
	while lh.can_undo():
		_app.undo_last()
	lh.entries.clear()
	lh.cursor = 0
	for n in 3:
		lh.commit("pw3 step %d" % n, {"zz": n + 1}, _bridge, {"zz": n})


func _sequence(target: Viewport) -> Array:
	_reset_history()
	var out: Array = [_hist()]
	for k in [[KEY_Z, true, false], [KEY_Z, true, false], [KEY_Y, true, false], [KEY_Z, true, true], [KEY_Z, true, false]]:
		target.push_input(_key(k[0], k[1], k[2]), true)
		await _settle(2)
		out.append(_hist())
	return out


func _wait_save(path: String) -> void:
	for _i in 60:
		await get_tree().create_timer(0.25).timeout
		if FileAccess.file_exists(path) and not _bridge.generating and not _app.busy_overlay.is_blocking():
			break
	await _settle(4)


func _keys_all() -> void:
	print("== 4. keys typed in each native window (b) ==")
	DccSettings.set_popout_enabled(false)
	PopoutHost.forget_choices(get_tree())
	await _hide_all()
	var main_seq: Array = await _sequence(get_tree().root)
	_ok(main_seq.size() == 6 and main_seq[0] == [true, false, 3], "control: the main window starts at 3 undoable steps")
	_ok(main_seq[1] != main_seq[0] and main_seq[3] != main_seq[2], "control: the main window's sequence moves the history (undo, then redo)")
	for s in _specs:
		var w: Window = s["win"]
		(s["open"] as Callable).call()
		await _settle(10)
		await PopoutHost.toggle(w)
		await _settle(8)
		_ok(not w.is_embedded(), "%s: native" % s["name"])
		var pop_seq: Array = await _sequence(w)
		_ok(pop_seq == main_seq, "%s: the pop-out's history after each of 5 keys equals the main window's%s" % [s["name"], "" if pop_seq == main_seq else " -- %s vs %s" % [str(pop_seq), str(main_seq)]])
		# a focused text field keeps its own Ctrl+Z (the forwarder refuses while typing)
		var tf: Control = null
		for n in w.find_children("*", "LineEdit", true, false):
			if (n as LineEdit).is_visible_in_tree() and (n as LineEdit).editable:
				tf = n
				break
		if tf == null:
			for n in w.find_children("*", "TextEdit", true, false):
				if (n as TextEdit).is_visible_in_tree() and (n as TextEdit).editable:
					tf = n
					break
		if tf != null:
			_reset_history()
			var fw := w.get_node(PopoutHost.FORWARDER_NAME)
			tf.grab_focus()
			await _settle(2)
			var h1 := _hist()
			fw._unhandled_key_input(_key(KEY_Z, true, false))
			await _settle(2)
			_ok(_hist() == h1, "%s: the forwarder refuses Ctrl+Z while a text field has focus (history %s unchanged)" % [s["name"], str(h1)])
			tf.release_focus()
			await _settle(2)
			fw._unhandled_key_input(_key(KEY_Z, true, false))
			await _settle(2)
			_ok(_hist() != h1, "%s: control: the same call with nothing focused undoes a step" % s["name"])
		else:
			print("  %s: no text field to focus (the typing guard is pinned by the windows that have one)" % s["name"])
		# Ctrl+S saves exactly as in the main window
		var p2 := ProjectSettings.globalize_path("user://pw3_save_%s.zip" % String(s["key"]))
		if FileAccess.file_exists(p2):
			DirAccess.remove_absolute(p2)
		_app.current_project_path = p2
		w.push_input(_key(KEY_S, true, false), true)
		await _wait_save(p2)
		_ok(FileAccess.file_exists(p2) and FileAccess.get_file_as_bytes(p2).size() > 1000, "%s: Ctrl+S in the pop-out wrote a project (%d bytes)" % [s["name"], FileAccess.get_file_as_bytes(p2).size() if FileAccess.file_exists(p2) else -1])
		_app.current_project_path = ""
		await PopoutHost.toggle(w)
		await _settle(6)
		w.hide()
		await _settle(3)


# -- 5 ------------------------------------------------------------------------------------------

func _popups_all() -> void:
	print("== 5. popups of a native window embed in it (e) ==")
	DccSettings.set_popout_enabled(false)
	PopoutHost.forget_choices(get_tree())
	await _hide_all()
	var tested := 0
	for s in _specs:
		var w: Window = s["win"]
		(s["open"] as Callable).call()
		await _settle(10)
		await PopoutHost.toggle(w)
		await _settle(8)
		var ob: OptionButton = null
		for n in w.find_children("*", "OptionButton", true, false):
			if (n as OptionButton).is_visible_in_tree() and (n as OptionButton).item_count > 1 and not (n as OptionButton).disabled:
				ob = n
				break
		if ob != null:
			ob.show_popup()
			await _settle(6)
			var pop := ob.get_popup()
			var pr := Rect2(Vector2(pop.position), Vector2(pop.size))
			print("  %s: dropdown popup rect %s in window %s (embedded %s)" % [s["name"], str(pr), str(w.size), str(pop.is_embedded())])
			_ok(pop.visible and pop.is_embedded(), "%s: the dropdown opens visible and embedded in the native window" % s["name"])
			pop.hide()
			await _settle(3)
			tested += 1
		var cp: ColorPickerButton = null
		for n in w.find_children("*", "ColorPickerButton", true, false):
			if (n as ColorPickerButton).is_visible_in_tree():
				cp = n
				break
		if cp != null:
			cp.pressed.emit()
			await _settle(8)
			var pop2 := cp.get_popup()
			var pr2 := Rect2(Vector2(pop2.position), Vector2(pop2.size))
			var inside := Rect2(Vector2.ZERO, Vector2(w.size)).encloses(pr2)
			print("  %s: colour popup rect %s in window %s (embedded %s, fully inside: %s)" % [s["name"], str(pr2), str(w.size), str(pop2.is_embedded()), str(inside)])
			_ok(pop2.visible and pop2.is_embedded(), "%s: the colour popup opens visible and embedded in the native window" % s["name"])
			if not inside:
				print("  NOTE  %s: the colour popup is taller or wider than the pop-out and is clipped to it (the window is %s, the popup %s)" % [s["name"], str(w.size), str(pr2.size)])
			await _shot_native(w, "popup_colour_" + String(s["key"]))
			pop2.hide()
			await _settle(3)
		await PopoutHost.toggle(w)
		await _settle(6)
		w.hide()
		await _settle(3)
	_ok(tested >= 3, "at least three windows had a dropdown to test (%d)" % tested)


# -- 6 ------------------------------------------------------------------------------------------

## A button on its own canvas layer inside `w`, above everything the window draws, with a hit counter.
## Mute is `Window.gui_disable_input`, which stops ALL input into the window, so a probe control is as good
## a target as any shipped one and works the same in every window.
func _add_target(w: Window) -> Dictionary:
	var cl := CanvasLayer.new()
	cl.layer = 50
	cl.name = "Pw3Target"
	var b := Button.new()
	b.text = "target"
	b.position = Vector2(8, 8)
	b.size = Vector2(80, 30)
	var hits := [0]
	b.pressed.connect(func() -> void: hits[0] += 1)
	cl.add_child(b)
	w.add_child(cl)
	return {"layer": cl, "btn": b, "hits": hits}


func _hit(w: Window, t: Dictionary) -> int:
	var before: int = t["hits"][0]
	var c := Vector2(48, 23)
	for pressed in [true, false]:
		var e := InputEventMouseButton.new()
		e.button_index = MOUSE_BUTTON_LEFT
		e.pressed = pressed
		e.position = c
		e.global_position = c
		w.push_input(e, true)
		await _settle(1)
	return int(t["hits"][0]) - before


func _sampled_diff(a: Image, b: Image) -> int:
	if a.get_size() != b.get_size():
		return -1
	a.convert(Image.FORMAT_RGBA8)
	b.convert(Image.FORMAT_RGBA8)
	var n := 0
	for y in range(0, a.get_height(), 3):
		for x in range(0, a.get_width(), 3):
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
	return n


## All seven windows native at once (the six of PW-3 and the roster of PW-2).
func _open_all_native() -> Array:
	DccSettings.set_popout_enabled(true)
	PopoutHost.forget_choices(get_tree())
	var wins: Array = []
	_roster_open()
	wins.append(["roster", _app.faction_roster_window])
	for s in _specs:
		(s["open"] as Callable).call()
		wins.append([s["name"], s["win"]])
	await _settle(14)
	return wins


func _busy() -> void:
	print("== 6. the busy overlay and every native window at once (a) ==")
	await _hide_all()
	var wins := await _open_all_native()
	var targets := {}
	for e in wins:
		var w: Window = e[1]
		_ok(w.visible and not w.is_embedded(), "%s: native" % e[0])
		targets[e[0]] = _add_target(w)
	_ok(_nwin() == 8, "eight OS windows (main + 7): %d" % _nwin())
	for e in wins:
		_ok(await _hit(e[1], targets[e[0]]) == 1, "%s: control: with no run, a click lands in the native window (1 hit)" % e[0])
	var st := {"checked": false, "blocked": false, "mirrors": 0, "muted": {}, "hits": {}, "diff": {}}
	var poll := func(n: int) -> void:
		if st["checked"] or n < 3:
			return
		st["checked"] = true
		st["blocked"] = _app.busy_overlay.is_blocking() and _bridge.generating
		st["mirrors"] = _app.busy_overlay.mirror_count()
		for e in wins:
			var w: Window = e[1]
			st["muted"][e[0]] = w.gui_disable_input
			st["hits"][e[0]] = await _hit(w, targets[e[0]])
			var with_card := w.get_texture().get_image()
			var mirror := w.get_node_or_null("BusyOverlayMirror") as CanvasItem
			if mirror != null:
				mirror.visible = false
				await _settle(3)
				var without := w.get_texture().get_image()
				st["diff"][e[0]] = _sampled_diff(with_card, without)
				mirror.visible = true
				await _settle(2)
			else:
				st["diff"][e[0]] = -2
			if _shots != "" and (e[0] == "assistant" or e[0] == "place editor"):
				with_card.save_png(_shots.path_join("pw3_busy_%s.png" % String(e[0]).replace(" ", "_")))
	await _gen(24601, Vector2i(768, 576), poll)
	_ok(st["checked"], "the in-run checks ran (the generate lasted long enough)")
	_ok(st["blocked"], "the overlay was blocking and the engine generating at the check")
	_ok(int(st["mirrors"]) >= 7, "the overlay carries a card mirror into every window (mirror_count %d)" % int(st["mirrors"]))
	for e in wins:
		_ok(bool(st["muted"].get(e[0], false)), "%s: gui_disable_input was true during the run" % e[0])
		_ok(int(st["hits"].get(e[0], -1)) == 0, "%s: a click during the run reached 0 controls (control above: 1)" % e[0])
		_ok(int(st["diff"].get(e[0], -3)) > 200, "%s: the card mirror changes %d sampled pixels of its own texture (flip-the-flag)" % [e[0], int(st["diff"].get(e[0], -3))])
	for e in wins:
		_ok(not (e[1] as Window).gui_disable_input, "%s: after the run the window takes input again" % e[0])
		_ok(await _hit(e[1], targets[e[0]]) == 1, "%s: after the run a click lands again (1 hit)" % e[0])
	_ok(_app.busy_overlay.mirror_count() == 0, "and no mirror is left (%d)" % _app.busy_overlay.mirror_count())
	# a dock and a re-pop DURING a second run keep every window muted
	var st2 := {"checked": false, "docked": {}, "native": {}, "hits": {}}
	var poll2 := func(n: int) -> void:
		if st2["checked"] or n < 2:
			return
		st2["checked"] = true
		for e in wins:
			var w: Window = e[1]
			await PopoutHost.toggle(w)
			await _settle(3)
			st2["docked"][e[0]] = w.is_embedded() and w.gui_disable_input
			await PopoutHost.toggle(w)
			await _settle(3)
			st2["native"][e[0]] = (not w.is_embedded()) and w.gui_disable_input
			st2["hits"][e[0]] = await _hit(w, targets[e[0]]) if is_instance_valid(targets[e[0]]["btn"]) else -1
	await _gen(71077345, Vector2i(1536, 1152), poll2)
	_ok(st2["checked"], "the second run's in-run moves happened")
	for e in wins:
		_ok(bool(st2["docked"].get(e[0], false)), "%s: docked DURING the run it is still muted" % e[0])
		_ok(bool(st2["native"].get(e[0], false)), "%s: popped out again DURING the run it is still muted" % e[0])
		_ok(int(st2["hits"].get(e[0], -1)) == 0, "%s: and a click after the re-pop reached 0 controls" % e[0])
	for e in wins:
		_ok(not (e[1] as Window).gui_disable_input, "%s: after the second run input returns" % e[0])
		(targets[e[0]]["layer"] as CanvasLayer).queue_free()
	await _hide_all()
	DccSettings.set_popout_enabled(false)
	PopoutHost.forget_choices(get_tree())


# -- 7 ------------------------------------------------------------------------------------------

func _setting_row() -> void:
	print("== 7. the Preferences row and the buttons agree ==")
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	var row := {}
	for r in idx.all():
		if String(r["title"]) == "Open panels in their own window":
			row = r
	_ok(not row.is_empty(), "the Preferences row exists")
	var popup: PopupMenu = row["_popup"]
	var id := int(row["_id"])
	var at := -1
	for k in popup.item_count:
		if popup.get_item_id(k) == id:
			at = k
	_ok(at >= 0 and popup.get_item_tooltip(at).contains("Settlement types") and popup.get_item_tooltip(at).contains("Vault") and popup.get_item_tooltip(at).contains("Assistant"), "the row's tooltip names the panels that honour it")
	DccSettings.set_popout_enabled(false)
	await _hide_all()
	for s in _specs:
		var w: Window = s["win"]
		w.set_meta(PopoutHost.META_CHOICE, false)
	popup.id_pressed.emit(id)
	_ok(DccSettings.popout_enabled(), "the row turned the setting ON")
	for s in _specs:
		_ok(not (s["win"] as Window).has_meta(PopoutHost.META_CHOICE), "%s: flipping the setting cleared its manual choice" % s["name"])
		(s["open"] as Callable).call()
		await _settle(10)
		_ok(not (s["win"] as Window).is_embedded() and _nwin() == 2, "%s: after the flip it follows the setting (native)" % s["name"])
		(s["win"] as Window).hide()
		await _settle(3)
	popup.id_pressed.emit(id)
	_ok(not DccSettings.popout_enabled(), "the row turned the setting OFF")
	for s in _specs:
		(s["open"] as Callable).call()
		await _settle(8)
		_ok((s["win"] as Window).is_embedded() and _nwin() == 1, "%s: setting off, no choice: embedded again, count 1" % s["name"])
		(s["win"] as Window).hide()
		await _settle(3)


# -- 8 ------------------------------------------------------------------------------------------

func _world_hash() -> String:
	var parts: Array = []
	for gy in range(5, GRID_SMALL.y, 13):
		for gx in range(5, GRID_SMALL.x, 15):
			parts.append(_bridge.sample_cell(gx, gy))
	parts.append(_bridge.get_factions())
	var s := JSON.stringify(parts)
	return s.sha256_text().substr(0, 16) + ":" + str(s.length())


func _state_hash() -> String:
	var b := JSON.stringify([_bridge.settlements(), SettlementTypeStore.document(), _bridge.get_active_urban_rules(), _bridge.get_factions()])
	return b.sha256_text().substr(0, 16)


func _first_line_edit(w: Window) -> LineEdit:
	for n in w.find_children("*", "LineEdit", true, false):
		if (n as LineEdit).editable and (n as LineEdit).is_visible_in_tree():
			return n
	return null


## The scripted edit for one window of the (c) bar, through the window's own controls. Returns what it did.
func _edit_through(spec: Dictionary) -> void:
	var w: Window = spec["win"]
	match String(spec["key"]):
		"place_editor_window", "settlement_types_window":
			var le := _first_line_edit(w)
			if le != null:
				le.text = "PW3 edit"
				le.text_submitted.emit("PW3 edit")
				await _settle(3)
		"generation_rules_window":
			## index 3 is "Medina"; index 2 is the default rule set, which would change nothing
			w._apply_preset(w._PRESETS[3])
			await _settle(3)


func _hashes() -> void:
	print("== 8. engine state, docked versus native (c) ==")
	DccSettings.set_popout_enabled(false)
	PopoutHost.forget_choices(get_tree())
	await _hide_all()
	while SettlementTypeStore.types().is_empty():
		SettlementTypeStore.new_type("PW3 base type")
	var edit_specs: Array = _specs.filter(func(s) -> bool: return s["key"] in ["place_editor_window", "settlement_types_window", "generation_rules_window"])
	for seed in SEEDS:
		for spec in edit_specs:
			var res := {}
			var pre_doc := SettlementTypeStore.document()
			for mode in ["docked", "native"]:
				await _gen(seed, GRID_SMALL)
				SettlementTypeStore.restore_document(pre_doc)
				_bridge.reset_active_urban_rules()
				var w: Window = spec["win"]
				(spec["open"] as Callable).call()
				await _settle(8)
				if mode == "native":
					await PopoutHost.toggle(w)
					await _settle(8)
				_ok((mode == "native") == (not w.is_embedded()), "seed %d %s %s: window is %s" % [seed, spec["name"], mode, mode])
				var pre := _state_hash()
				await _edit_through(spec)
				var post := _state_hash()
				_ok(pre != post, "seed %d %s %s: control: the scripted edit changed the state (%s -> %s)" % [seed, spec["name"], mode, pre, post])
				res[mode] = [post, _world_hash()]
				if mode == "native":
					await PopoutHost.toggle(w)
					await _settle(4)
				w.hide()
				await _settle(3)
			SettlementTypeStore.restore_document(pre_doc)
			_ok(res["docked"][0] == res["native"][0], "seed %d %s: the state hash after the edit is equal docked and native (%s)" % [seed, spec["name"], res["docked"][0]])
			_ok(res["docked"][1] == res["native"][1], "seed %d %s: the world hash is equal (%s)" % [seed, spec["name"], res["docked"][1]])
	# a world GENERATED while all windows are native hashes equal to one generated docked
	var gen := {}
	for mode in ["docked", "native"]:
		if mode == "native":
			await _open_all_native()
		await _gen(SEEDS[0], GRID_SMALL)
		gen[mode] = _world_hash()
		await _hide_all()
		DccSettings.set_popout_enabled(false)
		PopoutHost.forget_choices(get_tree())
	_ok(gen["docked"] == gen["native"], "a world generated with all seven windows native equals one generated docked (%s)" % gen["docked"])
	_ok(gen["docked"].split(":")[1].to_int() > 3000, "control: the fingerprint covered real cells (%s)" % gen["docked"])


# -- 9 ------------------------------------------------------------------------------------------

func _opener() -> void:
	print("== 9. a window opened from another window ==")
	DccSettings.set_popout_enabled(true)
	PopoutHost.forget_choices(get_tree())
	await _hide_all()
	_app.faction_roster_window.open(1, "settlements")
	await _settle(12)
	var roster: Window = _app.faction_roster_window
	_ok(not roster.is_embedded(), "the roster is native")
	roster.grab_focus()
	await _settle(10)
	var place: Window = _app.place_editor_window
	var focused := roster.has_focus()
	print("  roster.has_focus() after grab_focus: %s" % str(focused))
	_app.open_place_editor(0)
	await _settle(12)
	_ok(place.visible and not place.is_embedded(), "the place editor opened from the roster is native (setting on)")
	_ok(place.current_screen == roster.current_screen, "it is on the roster's screen (%d == %d; one monitor, so this half is trivial and the two-screen half is owner-run)" % [place.current_screen, roster.current_screen])
	var usable := DisplayServer.screen_get_usable_rect(place.current_screen)
	_ok(usable.encloses(Rect2i(place.position, place.size)) or place.size.y > usable.size.y, "and inside that screen's usable area (%s in %s)" % [str(Rect2i(place.position, place.size)), str(usable)])
	if focused:
		_ok(PopoutHost.opener_screen(_app.vault_window) == roster.current_screen, "opener_screen() reports the focused roster's screen for a window about to open from it")
	else:
		print("  NOTE  the OS did not give the roster the keyboard focus, so opener_screen() could not be shown live; the rule table above pins its decision")
	_shot(place, "opener_place_editor_native")
	await _hide_all()
	DccSettings.set_popout_enabled(false)
	PopoutHost.forget_choices(get_tree())


# -- 10 -----------------------------------------------------------------------------------------

func _configure(over: Dictionary = {}) -> void:
	var s := {"enabled": true, "commands": true, "prose": true, "local_allowed": true, "preset_id": "lm_studio",
		"base_url": "http://127.0.0.1:1234/v1", "model": "qwen3-14b",
		"key_source": "env", "env_var": ENV_VAR, "ctx_entity": true, "ctx_params": true}
	s.merge(over, true)
	for k in s:
		DccSettings.set_llm_value(k, s[k])
	DccSettings.llm_add_consent("http://127.0.0.1:1234")


func _c(name: String) -> Control:
	return _app.assistant_window.controls.get(name) as Control


func _type(text: String) -> void:
	var inp := _c("input") as TextEdit
	inp.text = text
	inp.text_changed.emit()
	await _settle(3)


func _click(c: Control) -> void:
	var w := c.get_window()
	var target: Viewport = get_tree().root if w.is_embedded() else w
	var r := c.get_global_rect()
	if w.is_embedded():
		r.position += Vector2(w.position)
	var p := r.get_center()
	for pressed in [true, false]:
		var e := InputEventMouseButton.new()
		e.button_index = MOUSE_BUTTON_LEFT
		e.pressed = pressed
		e.position = p
		e.global_position = p
		target.push_input(e, true)
		await _settle(2)
	await _settle(3)


func _digest() -> Dictionary:
	return (_bridge.edit_state_digest() as Dictionary).duplicate()


func _reply() -> String:
	var a: LlmAssistantWindow = _app.assistant_window
	return String(a._reply_label.text) if a._reply_label != null else ""


func _last_card() -> Dictionary:
	var a: LlmAssistantWindow = _app.assistant_window
	return a.cards[a.cards.size() - 1] if not a.cards.is_empty() else {}


func _text_of(n: Node) -> String:
	var parts: Array = []
	_all_text(n, parts)
	return "\n".join(parts)


func _all_text(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	for ch in n.get_children(true):
		_all_text(ch, out)


## Owner instruction (2026-10-26): no mock model servers in new probes. This section talks to the REAL LM Studio
## server (`lms server start --port 1234`, 127.0.0.1 only) with the real `qwen3-14b` loaded (`lms load qwen3-14b
## --gpu max -c 8192 --parallel 1 -y`), thinking off through the preset's `reasoning_effort: none`. The probe starts
## and stops nothing: with no server it records ONE failure saying so rather than passing vacuously. A real model
## is not deterministic, so no assertion depends on WHAT it says -- only on what the window does with it (a partial
## reply is visible, Stop and Close end the stream and join the worker, a move does not, and a card, if the model
## proposes one, applies and undoes). A bar the model did not give the chance to run is printed as NOT RUN, never
## counted as a pass.
func _lms_up() -> bool:
	var out: Array = []
	OS.execute("curl", ["-s", "-m", "5", "http://127.0.0.1:1234/v1/models"], out)
	return String("".join(out)).contains("qwen3-14b")


func _assistant() -> void:
	print("== 10. the Assistant as a native window (real LM Studio, qwen3-14b) ==")
	var a: LlmAssistantWindow = _app.assistant_window
	if not _lms_up():
		_ok(false, "LM Studio is not serving qwen3-14b on 127.0.0.1:1234 (start it first; this probe never starts it)")
		return
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()
	_configure()
	await _hide_all()
	DccSettings.set_popout_enabled(false)
	PopoutHost.forget_choices(get_tree())
	var places: Array = _bridge.settlements()
	_app.right_dock_ctrl.on_settlement_selected(places[0], 0)
	await _settle(4)
	a.open()
	await _settle(12)
	a._pick_task("prose")
	await _settle(3)
	_ok(a.visible and a.is_embedded(), "the assistant opens docked (setting off)")
	await _type("In about 300 words, describe the port of this settlement and what its people trade.")
	await PopoutHost.toggle(a)
	await _settle(10)
	_ok(not a.is_embedded() and a.visible, "popped out with a half-typed prompt")
	_ok((_c("input") as TextEdit).text.begins_with("In about 300 words"), "the half-typed prompt survived the move")
	_ok(not (_c("send") as Button).disabled, "Send is enabled in the native window (gate open: '%s')" % (_c("reason") as Label).text.left(60))

	print("-- 10a. Send and stream in the native window --")
	await _click(_c("send"))
	var saw_partial := false
	var saw_stop := false
	var t0 := Time.get_ticks_msec()
	while a.run.busy() and Time.get_ticks_msec() - t0 < 120000:
		var cur := _reply()
		if cur.length() > 20 and a.run.busy():
			if not saw_partial:
				_shot(a, "assistant_native_streaming")
			saw_partial = true
		if (_c("stop") as Button).visible:
			saw_stop = true
		await get_tree().process_frame
	_ok(saw_partial, "a PARTIAL reply was visible in the native window while the model was still generating")
	_ok(saw_stop, "Stop was visible while the reply ran")
	await _settle(6)
	_ok(_reply().length() > 100, "a finished reply of %d characters arrived" % _reply().length())
	_ok(LlmWire.live_threads == 0, "the worker is joined (live threads %d)" % LlmWire.live_threads)
	_ok(_text_of(a).contains("MODEL-GENERATED"), "the reply carries the MODEL-GENERATED chip")
	_shot(a, "assistant_native_reply")

	print("-- 10b. a MOVE mid-stream does not end the reply --")
	await _type("Write a long, detailed history of this settlement, at least 500 words.")
	await _click(_c("send"))
	_ok(await _until(func() -> bool: return _reply().length() >= 30, 60000), "tokens are arriving in the native window")
	var len0 := _reply().length()
	await PopoutHost.toggle(a)
	await _settle(4)
	_ok(a.is_embedded() and a.visible and a.run.busy(), "docked mid-stream: the run is still busy (not cancelled by the move)")
	_ok(await _until(func() -> bool: return _reply().length() > len0 + 20, 60000), "and tokens keep arriving after the dock (%d -> %d chars)" % [len0, _reply().length()])
	var len1 := _reply().length()
	await PopoutHost.toggle(a)
	await _settle(4)
	_ok(not a.is_embedded() and a.run.busy(), "popped out again mid-stream: still busy")
	_ok(await _until(func() -> bool: return _reply().length() > len1 + 20, 60000), "and tokens keep arriving after the re-pop (%d -> %d chars)" % [len1, _reply().length()])

	print("-- 10c. Stop in the native window --")
	var stop := _c("stop") as Button
	_ok(a.run.busy() and LlmWire.live_threads == 1, "positive control: the reply is still running with one worker thread (%d)" % LlmWire.live_threads)
	_ok(stop.visible and not stop.disabled, "Stop is visible and enabled in the native window")
	await _click(stop)
	_ok(await _until(func() -> bool: return _text_of(a).contains("Stopped."), 5000), "a real click on Stop in the native window: the panel says 'Stopped.'")
	var frozen := _reply()
	await _settle(40)
	_ok(_reply() == frozen, "no token was added after Stop")
	_ok(not a.run.busy() and LlmWire.live_threads == 0, "the worker is joined (live threads %d)" % LlmWire.live_threads)
	_ok(not a.is_embedded(), "the window is still native")

	print("-- 10d. Close mid-stream in the native window --")
	await _type("Write another long history of this settlement, at least 500 words.")
	await _click(_c("send"))
	_ok(await _until(func() -> bool: return _reply().length() >= 30, 60000), "tokens are arriving")
	_ok(LlmWire.live_threads == 1, "positive control: a worker thread exists mid-stream")
	a.hide()
	await _settle(6)
	_ok(not a.run.busy() and LlmWire.live_threads == 0, "closing the native window mid-stream cancelled and joined the worker (live threads %d)" % LlmWire.live_threads)
	await _settle(6)

	print("-- 10e. the off-gate and a forced Send --")
	a.open()
	await _settle(10)
	if a.is_embedded():
		await PopoutHost.toggle(a)
		await _settle(8)
	_configure({"enabled": false})
	a._refresh()
	await _type("Describe the port.")
	_ok((_c("send") as Button).disabled and (_c("reason") as Label).text.begins_with("The assistant is off"), "feature off: Send is disabled with its reason in the native window ('%s')" % (_c("reason") as Label).text.left(50))
	var att0 := LlmWire.connect_attempts
	a._on_send()
	await _settle(10)
	_ok(LlmWire.connect_attempts == att0, "a forced Send in the native window opened no connection (attempts %d)" % [LlmWire.connect_attempts - att0])
	_configure()
	a._refresh()
	await _settle(3)

	print("-- 10f. a Commands card from the real model: Apply and one undo, in the native window --")
	a._pick_task("commands")
	var d_before := _digest()
	var card: Dictionary = {}
	for attempt in 3:
		var cards0 := a.cards.size()
		await _type("Set the peak height (peak_m) to 2800 metres and nothing else. Use the proposal tool.")
		await _click(_c("send"))
		await _until(func() -> bool: return not a.run.busy(), 180000)
		await _settle(6)
		if a.cards.size() > cards0 and _last_card().get("apply") != null:
			card = _last_card()
			break
		print("  attempt %d: the model gave no card with Apply (%d cards)" % [attempt + 1, a.cards.size() - cards0])
	if card.is_empty():
		print("  NOT RUN  the real model did not produce a proposal card with Apply in 3 attempts; the card/Apply/undo path in a native window is NOT demonstrated by this run (the mock-based _llmparams_probe still gates the card logic itself)")
	else:
		_ok(_text_of(card["box"]).contains("MODEL PROPOSAL"), "the card carries the MODEL PROPOSAL chip")
		_ok(_digest() == d_before, "NOTHING was written before the click (world digest unchanged)")
		_ok(not (card["apply"] as Button).disabled, "'%s' is enabled" % (card["apply"] as Button).text)
		_shot(a, "assistant_native_card")
		await _click(card["apply"])
		await _settle(6)
		_ok(card["state"] == "applied", "a real click on Apply in the native window applied the card (%s)" % (card["reason"] as Label).text)
		_ok(_digest() != d_before, "positive control: the digest moved")
		_ok(String(_app.undo_groups.undo_label()).begins_with("Assistant: "), "Edit > Undo names the group: '%s'" % _app.undo_groups.undo_label())
		(_c("input") as TextEdit).release_focus()
		await _settle(3)
		a.push_input(_key(KEY_Z, true, false), true)
		await _settle(8)
		_ok(_digest() == d_before, "ONE Ctrl+Z typed in the native assistant (the forwarder) returns the whole world digest to before the Apply")

	print("-- 10g. the busy overlay mutes the native assistant while a world generates --")
	var st := {"was_gen": false, "muted": false}
	var poll := func(n: int) -> void:
		if st["was_gen"] or n < 3:
			return
		st["was_gen"] = _bridge.generating
		st["muted"] = a.gui_disable_input
	await _gen(SEEDS[1], Vector2i(512, 384), poll)
	_ok(st["was_gen"] and st["muted"], "while generating, the native assistant window is muted by the busy overlay")
	_ok(not a.gui_disable_input, "and takes input again afterwards")
	_ok(not a.is_embedded(), "the assistant is still native after all of that")
	await PopoutHost.toggle(a)
	await _settle(6)
	a.hide()
	await _settle(3)
	_configure({"enabled": false})
	LlmWire.shutdown_all()


# -- 11 -----------------------------------------------------------------------------------------

func _orphan() -> void:
	print("== 11. closing the main window closes every pop-out (d) ==")
	var dir := ProjectSettings.globalize_path("user://pw3_child")
	DirAccess.make_dir_recursive_absolute(dir)
	var marker := dir.path_join("child.txt")
	if FileAccess.file_exists(marker):
		DirAccess.remove_absolute(marker)
	var pid := OS.create_process(OS.get_executable_path(), ["--path", ProjectSettings.globalize_path("res://"), "--resolution", "900x640", "res://_popout_windows_probe.tscn", "--", "--form", "desktop", "--child", dir])
	_ok(pid > 0, "a child process started (pid %d)" % pid)
	var t := 0.0
	while OS.is_process_running(pid) and t < 150.0:
		await get_tree().create_timer(0.5).timeout
		t += 0.5
	_ok(not OS.is_process_running(pid), "the child process has exited (%.1f s): no orphan window or process" % t)
	var txt := FileAccess.get_file_as_string(marker) if FileAccess.file_exists(marker) else ""
	_ok(txt.begins_with("native windows=7"), "control: the child had really reached 7 OS windows (main + six) before it quit ('%s')" % txt.strip_edges())


func _child_main() -> void:
	await _boot()
	await _gen(SEEDS[1], GRID_SMALL)
	DccSettings.set_popout_enabled(true)
	PopoutHost.forget_choices(get_tree())
	for s in _specs:
		(s["open"] as Callable).call()
	await _settle(14)
	var f := FileAccess.open(_child.path_join("child.txt"), FileAccess.WRITE)
	f.store_string("native windows=%d\n" % _nwin())
	f.close()
	await get_tree().create_timer(0.5).timeout
	get_tree().quit(0)
