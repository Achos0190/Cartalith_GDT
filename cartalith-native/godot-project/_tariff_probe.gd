extends Node
## Proof for the tariff control (`OUTSTANDING_WORK.md` "Tariffs have no
## control"; Ruling AE, `LARGE_ITEM_RULINGS.md`). The engine and its save
## round trip were already built and tested (`cartalith-civ::trade::Tariff`,
## `civ_roster_bridge.rs::FactionRoster::set_tariff`, `civ_trade_bridge.rs`'s
## `civ_set_trade_tariff`/`civ_trade_tariff`, `_in13_tariff_probe.gd`); this
## probe is for the new part -- the control itself
## (`faction_roster_window.gd::_build_tariffs`) and its wiring into the
## Busiest-partners readout (`infrastructure_workspace.gd`,
## `civilization_workspace.gd::_on_tariff_changed`).
##
## On a real generated world, through the real controls:
##   1. match trade; record the busiest real cross-faction flow's row text
##      and volume in Civilization ▸ Economy ▸ Trade flows ▸ Busiest partners;
##   2. open the Faction roster on the IMPORTING faction, find the Tariff row
##      for the EXPORTING faction -- blank, not a fake zero;
##   3. type 25 and submit: WITHOUT pressing "Match trade flows" again, the
##      same partner row's volume is exactly 0.75x and its text has changed;
##   4. the engine agrees: `civ_trade_tariff` reads back 0.25, and a fresh
##      `civ_trade_flows()` shows the taxed flow at 0.75x with every other
##      flow's volume and price untouched;
##   5. "abc" and "200" are both refused -- the field reverts and the readout
##      is unchanged;
##   6. clearing the field (blank, not "0") restores the row exactly.
##
## Windowed (MISTAKES.md: headless textures and layout are not evidence; a
## pixel probe run --headless passes vacuously with 0 pixels compared). Saves
## a framebuffer PNG as the positive control that same row asks for.
##
## Flags, after `--`:
##   --vp WxH        the real OS window size, applied before the shell boots
##                    so `dcc_shell.gd::_compute_layout_mode` sees it. Default
##                    1920x1080.
##   --force-touch   `dcc_shell.gd`'s own testing override, read there --
##                    this probe only echoes it and names the saved PNG by it.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _tariff_probe.tscn
##   Godot_v4.7.1-stable_win64_console.exe --path . _tariff_probe.tscn -- --vp 1080x2340 --force-touch

var _fails := 0
var _phone := false

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("TARIFFC %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _find_script(n: Node, file: String) -> Node:
	for c in n.get_children():
		if c.get_script() != null and String(c.get_script().resource_path).ends_with(file):
			return c
		var r := _find_script(c, file)
		if r != null:
			return r
	return null

func _find_named(n: Node, nm: String) -> Node:
	for c in n.get_children():
		if String(c.name) == nm:
			return c
		var r := _find_named(c, nm)
		if r != null:
			return r
	return null

## Every "Busiest partners" row: a Button carrying the "worth" meta, in the
## same order `_fill_flows_partners` built them -- one per entry of
## `TradeStore.last()["flows"]`, index for index (`_currency_probe.gd`'s own
## precedent for this alignment).
func _worth_rows(body: Node) -> Array:
	var out: Array = []
	var stack: Array = [body]
	while not stack.is_empty():
		var n: Node = stack.pop_front()
		for c in n.get_children():
			stack.append(c)
			if c is Button and c.has_meta("worth"):
				out.append(c)
	return out

func _key(f: Dictionary) -> String:
	return "%d>%d:%s" % [int(f.get("from", -1)), int(f.get("to", -1)), String(f.get("good", ""))]

func _arg(name: String, fallback: String) -> String:
	var a := OS.get_cmdline_user_args()
	var i := a.find(name)
	return a[i + 1] if i >= 0 and i + 1 < a.size() else fallback

func _ready() -> void:
	## `-- --vp WxH --force-touch`. `--force-touch` is read by `dcc_shell.gd`
	## itself, off `OS.get_cmdline_user_args()` -- nothing here needs to touch
	## it. `--vp` is this probe's own: `dcc_shell.gd::_compute_layout_mode`
	## decides phone-vs-tablet off `get_viewport_rect().size` AT `_ready()`, so
	## the real OS window has to already be the target size before `app.tscn`
	## is instantiated, not resized after -- `DisplayServer.window_set_size`
	## first, one frame to let it land, then load the shell.
	var args := OS.get_cmdline_user_args()
	_phone = "--force-touch" in args
	var parts: PackedStringArray = _arg("--vp", "1920x1080").split("x")
	var vp := Vector2i(1920, 1080)
	if parts.size() == 2 and parts[0].is_valid_int() and parts[1].is_valid_int():
		vp = Vector2i(int(parts[0]), int(parts[1]))
	else:
		_check("--vp parses as WxH", false, _arg("--vp", ""))
	DisplayServer.window_set_size(vp)
	await _frames(2)
	print("TARIFFC boot vp=%dx%d force-touch=%s" % [vp.x, vp.y, _phone])

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge: Node = app.bridge
	bridge.generate({"seed": 9137, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)

	var ws: Node = _find_script(app, "infrastructure_workspace.gd")
	_check("the Trade workspace is found", ws != null)
	if ws == null:
		_finish()
		return
	ws._match_trade_flows()
	await _frames(4)

	## The busiest real cross-faction pair, the same choice
	## `_in13_tariff_probe.gd::_tariff` makes and for the same reason: the
	## effect has to land on real volume, not a corner case with nothing to
	## shrink.
	var flows: Array = TradeStore.last().get("flows", [])
	_check("the world trades", flows.size() > 0, "%d flows" % flows.size())
	if flows.is_empty():
		_finish()
		return
	var best_i := -1
	var best_vol := -1.0
	for i in flows.size():
		var f: Dictionary = flows[i]
		if int(f.get("to_faction", 0)) != int(f.get("from_faction", -1)) and int(f.get("to_faction", 0)) >= 1:
			var v := float(f.get("volume", 0.0))
			if v > best_vol:
				best_vol = v
				best_i = i
	_check("a cross-faction flow exists to tax", best_i >= 0)
	if best_i < 0:
		_finish()
		return
	var f0: Dictionary = flows[best_i]
	var imp := int(f0["to_faction"])
	var exp := int(f0["from_faction"])
	var key0 := _key(f0)
	var vol0 := float(f0["volume"])
	_check("no tariff set yet", float(bridge.civ_trade_tariff(imp, exp)) == 0.0)

	var rows := _worth_rows(ws._flows_body)
	_check("the busiest pair is within the Busiest-partners list",
		best_i < rows.size() and best_i < 12, "index %d of %d shown" % [best_i, min(12, rows.size())])
	var text0 := String(rows[best_i].text) if best_i < rows.size() else ""
	print("TARIFFC pre-tariff row: %s" % text0)

	## -- the roster, through its own field ------------------------------------
	var roster = app.faction_roster_window
	## FH-0: Currency and Tariffs live on the Economy tab now, which is built
	## lazily -- open straight onto it.
	roster.open(-1, "economy")
	await _frames(3)
	roster._selected = imp
	roster._rebuild_inspector()
	await _frames(3)
	var t_le: LineEdit = _find_named(roster._inspector_body, "Tariff_%d" % exp)
	_check("the roster draws a Tariff field for this exporter", t_le != null)
	if t_le == null:
		_finish()
		return
	_check("an unset tariff is blank, not a fake zero",
		t_le.text == "" and t_le.placeholder_text.contains("none"),
		"text '%s', placeholder '%s'" % [t_le.text, t_le.placeholder_text])

	t_le.text = "25"
	t_le.text_submitted.emit("25")
	await _frames(4)

	_check("the engine reads back 0.25",
		absf(float(bridge.civ_trade_tariff(imp, exp)) - 0.25) <= 1e-9,
		str(bridge.civ_trade_tariff(imp, exp)))
	_check("the reverse pair stays untaxed (directional)",
		float(bridge.civ_trade_tariff(exp, imp)) == 0.0)

	## The readout re-matched itself (`civilization_workspace.gd::
	## _on_tariff_changed`) -- no second press of "Match trade flows".
	var flows1: Array = TradeStore.last().get("flows", [])
	var i1 := -1
	for i in flows1.size():
		if _key(flows1[i]) == key0:
			i1 = i
			break
	_check("the taxed flow still exists after the edit", i1 >= 0)
	if i1 >= 0:
		var f1: Dictionary = flows1[i1]
		var vol1 := float(f1["volume"])
		_check("its volume is exactly 0.75x", absf(vol1 - 0.75 * vol0) <= 1e-9 * vol0,
			"%f -> %f (want %f)" % [vol0, vol1, 0.75 * vol0])
		var rows1 := _worth_rows(ws._flows_body)
		_check("it is still in the visible Busiest-partners list",
			i1 < rows1.size() and i1 < 12, "index %d of %d shown" % [i1, min(12, rows1.size())])
		if i1 < rows1.size() and i1 < 12:
			var text1 := String(rows1[i1].text)
			print("TARIFFC post-tariff row: %s" % text1)
			_check("the readout's text changed", text1 != text0, text1)

	for bad in ["abc", "200"]:
		t_le.text = bad
		t_le.text_submitted.emit(bad)
		await _frames(2)
		## The reverted text is `_tariff_text()`'s own formatting
		## (`String.num(25.0, 4)` == "25.0", not the literal "25" typed
		## earlier), so this checks the numeric value, not the literal string.
		_check("'%s' is refused and the field reverts to 25%%" % bad,
			absf(float(bridge.civ_trade_tariff(imp, exp)) - 0.25) <= 1e-9
				and is_equal_approx(t_le.text.to_float(), 25.0),
			"engine %s, field '%s'" % [str(bridge.civ_trade_tariff(imp, exp)), t_le.text])

	t_le.text = ""
	t_le.text_submitted.emit("")
	await _frames(4)
	_check("clearing the field clears the tariff",
		float(bridge.civ_trade_tariff(imp, exp)) == 0.0)
	var flows2: Array = TradeStore.last().get("flows", [])
	var i2 := -1
	for i in flows2.size():
		if _key(flows2[i]) == key0:
			i2 = i
			break
	_check("the flow's volume is restored exactly",
		i2 >= 0 and absf(float(flows2[i2]["volume"]) - vol0) <= 1e-9 * vol0)

	## Positive control (`MISTAKES.md`: a pixel probe needs one, and
	## `--headless` makes `get_image()` null -- this save is the check that
	## the window is real).
	await _frames(2)
	var shot_path := "res://_tariff_probe_phone.png" if _phone else "res://_tariff_probe.png"
	var img := get_viewport().get_texture().get_image()
	_check("the framebuffer image is real (not --headless)", img != null and img.get_size().x > 0)
	if img != null:
		img.save_png(shot_path)
		print("TARIFFC saved %s (%s)" % [shot_path, img.get_size()])

	_finish()

func _finish() -> void:
	print("TARIFFC %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
