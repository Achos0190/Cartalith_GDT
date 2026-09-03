extends Node
## INDEPENDENT verifier probe for the 2026-09-03 right-dock ruling.
## Deliberately does NOT call `_current_title()` or `_dock_readout_text()` --
## it reads the shell chrome that is actually on screen:
##   * `app.right_dock_title.text` (the Label `set_right_dock_title()` writes)
##   * `app._dock_readouts["right"].text` (the Label `set_dock_readout()` writes)
##   * every Label in `app.right_dock_body`
## so a bug in either helper cannot be hidden by asking the helper.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _vfy_rdappend_probe.tscn

var app: Node
var _fail := 0
var _sel_name := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ck(name: String, cond: bool, detail: String = "") -> void:
	print("VFY %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _collect(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append((c as Label).text)
		_collect(c, out)

func _texts() -> Array:
	var out: Array = []
	_collect(app.right_dock_body, out)
	return out

func _headers() -> Array:
	var out: Array = []
	for t in _texts():
		var s := String(t)
		if s.begins_with("§ "):
			out.append(s.substr(2))
	return out

func _hdr(prefix: String) -> int:
	var hs := _headers()
	var want := prefix.to_upper()
	for i in hs.size():
		if String(hs[i]).begins_with(want):
			return i
	return -1

## The chrome Label, not the function that computes it.
func _title_on_screen() -> String:
	return String(app.right_dock_title.text)

func _readout_on_screen() -> String:
	return String((app._dock_readouts["right"] as Label).text)

func _name_shown() -> bool:
	return _texts().has(_sel_name)

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 2400:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	if not app.bridge.has_world:
		print("VFY !! no world")
		get_tree().quit(1)
		return
	var rd = app.right_dock_ctrl
	var st: Array = app.bridge.settlements()
	if st.is_empty():
		print("VFY !! no settlement")
		get_tree().quit(1)
		return
	_sel_name = String((st[0] as Dictionary).get("name", ""))
	_ck("settlement has a name", _sel_name != "", "name='%s'" % _sel_name)

	# ---- 1. select ----------------------------------------------------------
	app.select_domain("world")
	app.arm_tool("inspect")
	await _frames(3)
	rd.on_settlement_selected(st[0], 0)
	await _frames(4)
	var t0 := _title_on_screen()
	var r0 := _readout_on_screen()
	_ck("[select] chrome title says SETTLEMENT", t0 == "SETTLEMENT", "title='%s'" % t0)
	_ck("[select] the name label is in the body", _name_shown(), "headers=%s" % [_headers()])
	_ck("[select] chrome readout is the name", r0 == _sel_name, "readout='%s'" % r0)

	# ---- 2. arm each tool over the live selection ---------------------------
	for pair in [["paint", "PAINT", "world"], ["territory", "TERRITORY", "world"],
			["label", "ANNOTATION", "cartography"]]:
		var tool_id := String(pair[0])
		var hdr := String(pair[1])
		app.select_domain(String(pair[2]))
		await _frames(3)
		rd.on_settlement_selected(st[0], 0)
		await _frames(3)
		app.arm_tool(tool_id)
		await _frames(5)
		var ti := _title_on_screen()
		var ri := _readout_on_screen()
		var s_i := _hdr("SETTLEMENT")
		var x_i := _hdr(hdr)
		_ck("[%s] title UNCHANGED on screen" % tool_id, ti == "SETTLEMENT", "title='%s'" % ti)
		_ck("[%s] settlement name still in body" % tool_id, _name_shown(), "headers=%s" % [_headers()])
		_ck("[%s] tool section present" % tool_id, x_i >= 0, "headers=%s" % [_headers()])
		_ck("[%s] tool section is BELOW the selection" % tool_id,
			s_i >= 0 and x_i > s_i, "settlement@%d %s@%d" % [s_i, hdr, x_i])
		_ck("[%s] chrome readout still the selection" % tool_id, ri == _sel_name, "readout='%s'" % ri)
		# ---- 3. disarm -------------------------------------------------------
		app.arm_tool("inspect")
		await _frames(5)
		_ck("[%s] disarm: selection intact" % tool_id,
			_title_on_screen() == "SETTLEMENT" and _name_shown(),
			"title='%s' name=%s" % [_title_on_screen(), _name_shown()])
		_ck("[%s] disarm: tool section gone" % tool_id, _hdr(hdr) < 0,
			"headers=%s" % [_headers()])

	# stops: armed by the CARTO domain itself with inspect
	app.select_domain("world")
	await _frames(2)
	rd.on_settlement_selected(st[0], 0)
	await _frames(3)
	app.select_domain("cartography")
	await _frames(5)
	_ck("[stops] title UNCHANGED on screen", _title_on_screen() == "SETTLEMENT",
		"title='%s'" % _title_on_screen())
	_ck("[stops] section below the selection",
		_hdr("SETTLEMENT") >= 0 and _hdr("RAMP") > _hdr("SETTLEMENT"),
		"headers=%s" % [_headers()])

	# ---- 4. a tool armed with NOTHING selected ------------------------------
	rd.on_settlement_selected(null, -1)
	await _frames(5)
	_ck("[none] settlement section gone", _hdr("SETTLEMENT") < 0, "headers=%s" % [_headers()])
	_ck("[none+stops] tool section still drawn", _hdr("RAMP") >= 0, "headers=%s" % [_headers()])
	_ck("[none] title falls back to SAMPLE", _title_on_screen() == "SAMPLE",
		"title='%s'" % _title_on_screen())
	_ck("[none+stops] readout reports the tool", _readout_on_screen().find("stop") >= 0,
		"readout='%s'" % _readout_on_screen())

	app.select_domain("world")
	app.arm_tool("paint")
	await _frames(5)
	_ck("[none+paint] Paint drawn with no selection", _hdr("PAINT") >= 0,
		"headers=%s" % [_headers()])
	_ck("[none+paint] Sample drawn ABOVE it", _hdr("SAMPLE") >= 0 and _hdr("PAINT") > _hdr("SAMPLE"),
		"sample@%d paint@%d" % [_hdr("SAMPLE"), _hdr("PAINT")])
	_ck("[none+paint] title SAMPLE", _title_on_screen() == "SAMPLE", "title='%s'" % _title_on_screen())

	app.arm_tool("territory")
	await _frames(5)
	_ck("[none+territory] Territory drawn with no selection", _hdr("TERRITORY") >= 0,
		"headers=%s" % [_headers()])
	_ck("[none+territory] Paint section gone (one tool slot)", _hdr("PAINT") < 0,
		"headers=%s" % [_headers()])

	# ---- 5. the rejected shapes, asserted absent ----------------------------
	for id in ["paint", "stops", "anno", "territory"]:
		_ck("no CTX_TITLES row for '%s'" % id, not rd.CTX_TITLES.has(id),
			"keys=%s" % [rd.CTX_TITLES.keys()])
	_ck("_context is not a tool id",
		not (rd._context in ["paint", "stops", "anno", "territory"]),
		"_context=%s" % rd._context)

	print("VFY fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
