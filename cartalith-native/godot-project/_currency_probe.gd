extends Node
## Proof for per-faction currencies (Ruling R, kept by AR; Ruling AU: the user
## types the rate in the faction roster and the engine only converts). On a
## real generated world, through the real controls:
##   1. match trade; a Busiest-partners row into a real faction shows its value
##      "(at par)" -- no rate set, the index value itself;
##   2. type 12.5 into the roster's Rate field and submit: the SAME row's value
##      is exactly 12.5x what it was, and no longer says "at par";
##   3. type a symbol: the row shows it;
##   4. type "abc" and "0": both refused, the rate stays 12.5 and the field
##      shows 12.5 again;
##   5. the trade match itself is untouched: a fresh civ_trade_flows() has the
##      same flows, volumes and prices as before the edit (a rate is display);
##   6. clearing the field returns the row to its at-par value exactly.
##
## Windowed (MISTAKES.md: headless textures and layout are not evidence):
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _currency_probe.tscn

var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("CURRENCY %s  %s%s" % ["ok  " if cond else "FAIL", what,
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

## Every partner row: a Button carrying the "worth" meta.
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

func _flow_sig(d: Dictionary) -> Array:
	var out: Array = []
	for f in (d.get("flows", []) as Array):
		var r: Dictionary = f
		out.append([int(r["from"]), int(r["to"]), String(r["good"]), float(r["volume"]), float(r["price"]), float(r["tariff"])])
	return out

func _ready() -> void:
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

	var before_sig := _flow_sig(bridge.civ_trade_flows())
	_check("the world trades", before_sig.size() > 0, "%d flows" % before_sig.size())

	var ws: Node = _find_script(app, "infrastructure_workspace.gd")
	_check("the Trade workspace is found", ws != null)
	if ws == null:
		_finish()
		return
	ws._match_trade_flows()
	await _frames(4)

	## The first row whose importer is a real faction (Unclaimed has no currency).
	var row_i := -1
	var rows := _worth_rows(ws._flows_body)
	for i in rows.size():
		if int((rows[i].get_meta("worth") as Dictionary)["faction"]) >= 1:
			row_i = i
			break
	_check("a partner row imports into a real faction", row_i >= 0, "%d rows" % rows.size())
	if row_i < 0:
		_finish()
		return
	var w0: Dictionary = rows[row_i].get_meta("worth")
	var fid := int(w0["faction"])
	var a0 := float(w0["amount"])
	var text0: String = rows[row_i].text
	print("CURRENCY row %d, importer faction %d: %s" % [row_i, fid, text0])
	_check("an unset currency reads at par", bool(w0["rate_default"]) and text0.contains("(at par)") and text0.contains("¤"), text0)
	_check("at par is the index value itself", a0 > 0.0 and is_equal_approx(a0,
		float((TradeStore.last()["flows"] as Array)[row_i]["value"])), "amount %f" % a0)

	## -- the roster, through its own field ------------------------------------
	var roster = app.faction_roster_window
	roster.open()
	await _frames(3)
	roster._selected = fid
	roster._rebuild_inspector()
	await _frames(3)
	var rate_le: LineEdit = _find_named(roster._inspector_body, "Currency_rate")
	var sym_le: LineEdit = _find_named(roster._inspector_body, "Currency_symbol")
	_check("the roster draws Rate and Symbol fields", rate_le != null and sym_le != null)
	if rate_le == null or sym_le == null:
		_finish()
		return
	_check("an unset rate is a placeholder, not a value", rate_le.text == "" and rate_le.placeholder_text.contains("default"),
		"text '%s', placeholder '%s'" % [rate_le.text, rate_le.placeholder_text])

	rate_le.text = "12.5"
	rate_le.text_submitted.emit(rate_le.text)
	await _frames(3)
	rows = _worth_rows(ws._flows_body)
	var w1: Dictionary = rows[row_i].get_meta("worth")
	var a1 := float(w1["amount"])
	print("CURRENCY after rate 12.5: %s" % rows[row_i].text)
	_check("the same row now reads exactly 12.5x", a1 == a0 * 12.5, "%.10f vs %.10f" % [a1, a0 * 12.5])
	_check("and no longer says at par", not bool(w1["rate_default"]) and not String(rows[row_i].text).contains("(at par)"), rows[row_i].text)
	_check("the engine holds the typed rate", float(bridge.civ_faction_currency(fid)["rate"]) == 12.5
		and not bool(bridge.civ_faction_currency(fid)["rate_default"]))

	sym_le.text = "dn"
	sym_le.text_submitted.emit(sym_le.text)
	await _frames(3)
	rows = _worth_rows(ws._flows_body)
	_check("the row shows the typed symbol", String(rows[row_i].text).contains(" dn"), rows[row_i].text)

	for bad in ["abc", "0", "-3"]:
		rate_le.text = bad
		rate_le.text_submitted.emit(rate_le.text)
		await _frames(2)
		_check("'%s' is refused and the field reverts" % bad,
			float(bridge.civ_faction_currency(fid)["rate"]) == 12.5 and rate_le.text == "12.5",
			"engine %s, field '%s'" % [str(bridge.civ_faction_currency(fid)["rate"]), rate_le.text])
	rows = _worth_rows(ws._flows_body)
	_check("after refusals the row still reads 12.5x", float((rows[row_i].get_meta("worth") as Dictionary)["amount"]) == a0 * 12.5)

	var after_sig := _flow_sig(bridge.civ_trade_flows())
	_check("the trade match is untouched by a rate", after_sig == before_sig,
		"%d flows before, %d after" % [before_sig.size(), after_sig.size()])

	rate_le.text = ""
	rate_le.text_submitted.emit(rate_le.text)
	await _frames(3)
	rows = _worth_rows(ws._flows_body)
	var w2: Dictionary = rows[row_i].get_meta("worth")
	_check("clearing the rate returns the row to par exactly", float(w2["amount"]) == a0 and bool(w2["rate_default"]),
		"%.10f vs %.10f" % [float(w2["amount"]), a0])
	_check("Unclaimed has no currency", bridge.civ_faction_currency(0).is_empty()
		and not bridge.civ_set_faction_currency(0, "rate", "2"))
	_finish()

func _finish() -> void:
	print("CURRENCY %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
