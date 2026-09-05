extends Node
## Verifier for three of the 2026-09-05 menu-conformance fixes. (Unrelated to
## the committed `_menuconf_probe.gd`, which is the 2026-08-25 menu *sweep*.) Layout- and
## content-level, no pixels, so `--headless` is honest here.
##
##   godot --headless --path . _confsix_probe.tscn
##
## (`--nowelcome` is not a shell flag -- only `_shot_phone.gd` reads it. The
## welcome dialog is hidden below, in `_ready`, the way every other shell probe
## here does it.)
##
## 1 -- `right_dock.gd::_settlement_faction_row()`: `05 §1.11 rdPlace` gives
##      FACTION as *"the owning faction name"*. Read the RENDERED row back out
##      of the dock and match it against `get_factions()`' own `name` for that
##      settlement's id -- never against the function's own arithmetic. Three
##      seeds, because one world is one sample.
##
## 2 -- `tool_bar.gd::_build_paint_options()`: §2.2.6 draws `✓ Commit` **and**
##      `Discard`. Assert the chip exists, is dead with an empty draft, is live
##      with a pending one, and that pressing it actually empties the draft --
##      a button that draws and does nothing is the failure this replaces.
##
## 3 -- `Workspace.push_dock_readout()`: the collapsed left dock's line follows
##      the domain. Includes the regression that motivated the gate in
##      `WorldWorkspace`: a WORLD push arriving while the reader is in CIVIL (a
##      `generation_stage` tick) must not overwrite CIVIL's line.

const SEEDS := [4021, 77123, 918237]

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got: Variant, want: Variant) -> void:
	var pass_ := str(got) == str(want)
	print("MC %s  %-46s got=%s want=%s" % ["ok  " if pass_ else "FAIL", name, str(got), str(want)])
	if not pass_:
		_fail += 1

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("MC %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

## Every `_field()` row in the right dock, as label -> value. `_field` builds
## one `HBoxContainer` of exactly two `Label`s, so the pairing is read off the
## tree rather than off an index into a flat text list.
func _rows(n: Node, out: Dictionary) -> void:
	for c in n.get_children():
		if c is HBoxContainer:
			var labels: Array = []
			for g in c.get_children():
				if g is Label:
					labels.append(g)
			if labels.size() == 2:
				out[String((labels[0] as Label).text)] = String((labels[1] as Label).text)
		_rows(c, out)

func _dock_rows() -> Dictionary:
	var out := {}
	_rows(app.right_dock_body, out)
	return out

func _buttons(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Button:
			out.append(c)
		_buttons(c, out)

func _chip(text: String) -> Button:
	var out: Array = []
	_buttons(app.tool_options_row, out)
	for b in out:
		if String((b as Button).text) == text:
			return b
	return null

func _left_readout() -> String:
	return String((app._dock_readouts["left"] as Label).text)

func _generate(seed_v: int) -> bool:
	app.bridge.generate({"seed": seed_v, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while app.bridge.generating and waited < 4000:
		await get_tree().process_frame
		waited += 1
	await _frames(10)
	return app.bridge.has_world

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	var bridge = app.bridge
	var rd = app.right_dock_ctrl

	# -- 3: the collapsed left dock's line, before any world exists -----------
	for pair in [["world", "no world"], ["civilization", "CIVIL"], ["cartography", "CARTO"]]:
		app.select_domain(String(pair[0]))
		await _frames(6)
		_ok("left readout in %s (no world)" % pair[0], _left_readout(), String(pair[1]))

	for seed_v in SEEDS:
		if not await _generate(seed_v):
			_check("seed %d generated" % seed_v, false)
			continue
		print("MC ================= seed %d =================" % seed_v)

		# -- 1: FACTION is a name --------------------------------------------
		## `EngineBridge.settlements()`, not `get_settlements()` -- the wrapper
		## drops the `get_` the Rust `#[func]` carries. Grepped for `func` before
		## the second call; the first cost a run.
		var settlements: Array = bridge.settlements()
		var names := {}
		for f in bridge.get_factions():
			names[int((f as Dictionary).get("id", -1))] = String((f as Dictionary).get("name", ""))
		_check("seed %d has factions" % seed_v, not names.is_empty(),
			"%d roster rows, %d settlements" % [names.size(), settlements.size()])
		app.select_domain("civilization")
		await _frames(4)
		var checked := 0
		for i in mini(settlements.size(), 5):
			var s: Dictionary = settlements[i]
			rd.on_settlement_selected(s, i)
			await _frames(6)
			var rows := _dock_rows()
			var shown := String(rows.get("Faction", "<no Faction row>"))
			var fid := int(s.get("faction", 0))
			var want: String = String(names.get(fid, "—")) if fid > 0 else "—"
			_ok("seed %d place %d (faction %d)" % [seed_v, i, fid], shown, want)
			## The defect this replaces printed the id. Assert the shape as
			## well as the value: a roster whose names were themselves digits
			## would let the equality above pass over the old behaviour.
			_check("  value is not a bare integer", not shown.is_valid_int(), shown)
			checked += 1
		_check("seed %d checked at least one settlement" % seed_v, checked > 0, str(checked))

	## The two absent-value arms the three seeds above never reach: 645 real
	## settlements and not one carried faction 0. `_build_settlement` reads
	## `_settlement_data` whole, so a hand-made entry exercises the arms
	## without needing a world that happens to contain one -- and an arm that
	## is never rendered is an arm nobody has checked (MISTAKES.md, "encoding
	## no value as a plausible value").
	app.select_domain("civilization")
	await _frames(4)
	for probe_case in [[0, "unclaimed"], [99, "stale id"]]:
		rd.on_settlement_selected({"name": "Probeton", "kind": "town",
			"population": 100, "faction": int(probe_case[0]), "coastal": false,
			"capital": false, "tid": 999999}, 0)
		await _frames(6)
		_ok("synthetic faction %s -> dash" % probe_case[1],
			String(_dock_rows().get("Faction", "<no Faction row>")), "—")

	# -- 3 continued: a WORLD push must not reach a CIVIL dock ----------------
	app.select_domain("civilization")
	await _frames(4)
	_ok("left readout in CIVIL (world exists)", _left_readout(), "CIVIL")
	## What a `generation_stage` tick does while the reader is elsewhere.
	app._workspace_panels["world"].push_dock_readout()
	await _frames(2)
	_ok("WORLD push while in CIVIL leaves it alone", _left_readout(), "CIVIL")
	app.select_domain("world")
	await _frames(6)
	_ok("left readout back in WORLD", _left_readout(), "resolved")

	# -- 2: the paint options row's Discard -----------------------------------
	app.select_domain("world")
	await _frames(4)
	var layers: PackedStringArray = bridge.get_paint_layers()
	_check("paint layers exist", layers.size() > 0, str(layers))
	bridge.paint_set_layer(String(layers[0]))
	bridge.paint_set_brush(1, 6.0, 1.0, 0.0, false, true)
	app.arm_tool("paint")
	await _frames(8)
	var discard: Button = _chip("Discard")
	var commit: Button = _chip("Commit")
	_check("Commit chip on the paint options row", commit != null)
	_check("Discard chip on the paint options row", discard != null)
	if discard == null or commit == null:
		print("MC %s (%d failed)" % ["PASS" if _fail == 0 else "FAIL", _fail])
		get_tree().quit(_fail)
		return
	_ok("Discard dead with an empty draft", discard.disabled, true)
	_ok("draft empty before painting", bridge.paint_draft_count(), 0)

	var gs: Vector2i = bridge.grid_size()
	for k in 4:
		bridge.paint_stroke_at(float(gs.x) * (0.35 + 0.04 * k), float(gs.y) * 0.5)
	app.tool_bar.rebuild()
	await _frames(6)
	var pending: int = bridge.paint_draft_count()
	_check("draft is pending after painting", pending > 0, "%d cells" % pending)
	discard = _chip("Discard")
	_check("Discard still drawn with a live draft", discard != null)
	_ok("Discard live with a pending draft", discard.disabled, false)

	discard.emit_signal("pressed")
	await _frames(8)
	_ok("draft emptied by Discard", bridge.paint_draft_count(), 0)
	discard = _chip("Discard")
	_check("Discard redrawn after the press", discard != null)
	if discard != null:
		_ok("Discard dead again", discard.disabled, true)

	print("MC %s (%d failed)" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(_fail)
