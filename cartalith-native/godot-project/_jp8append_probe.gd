extends Node
## Verification harness for the owner's 2026-09-04 extension of the right-dock
## ruling (`LARGE_ITEM_RULINGS.md`): *"`rdMode4()` (rule 8) is the last built
## context that replaces the selection; it becomes an appended section like
## every other."*
##
## Written to the hazard the ruling itself carries across: rule 1's conversion
## guarded the DISARM path and missed the ARM-ANOTHER-TOOL path. So this
## enumerates every transition INTO the converted state -- over a settlement,
## over nothing, over a live sculpt draft, over a live way draft, across a
## domain switch, across a regenerate -- and reads the DOCK BACK each time
## rather than reasoning from `_tool_section()`.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _jp8append_probe.tscn

var app: Node
var _fail := 0
var _sel: Dictionary = {}
var _sel_name := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("JP8 %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
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

## Position of the first section whose title starts with `prefix`, or -1.
## A position, not a bool, because "appends" is a claim about ORDER.
func _hdr(prefix: String) -> int:
	var hs := _headers()
	var want := prefix.to_upper()
	for i in hs.size():
		if String(hs[i]).begins_with(want):
			return i
	return -1

## `SUPPLY REACH` is `build_results()`'s own, drawn by nothing else in the dock.
func _jp() -> int:
	return _hdr("SUPPLY REACH")

func _name_shown() -> bool:
	return _texts().has(_sel_name)

func _title() -> String:
	return String(app.right_dock_ctrl._current_title())

func _readout() -> String:
	return String(app.right_dock_ctrl._dock_readout_text())

func _arm_journey() -> void:
	app.select_domain("civilization")
	await _frames(3)
	app.arm_tool("journey")
	await _frames(10)

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 1800:
		await get_tree().process_frame
		waited += 1
	print("JP8 world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("JP8  !! generate failed -- nothing else here can run")
		get_tree().quit(1)
		return

	var rd = app.right_dock_ctrl
	var bridge = app.bridge

	# -- A committed route, so `build_results()` draws its real sections --------
	var gs: Vector2 = bridge.grid_size()
	bridge.route_begin("mixed")
	bridge.route_append_stop(gs.x * 0.30, gs.y * 0.45)
	bridge.route_append_stop(gs.x * 0.62, gs.y * 0.55)
	var ridx: int = bridge.route_commit()
	_check("a committed route exists for the planner to plan over", ridx >= 0,
		"route_commit=%d route_count=%d" % [ridx, bridge.route_count()])

	var settlements: Array = bridge.settlements()
	if settlements.is_empty():
		print("JP8  !! no settlement to select -- the whole probe is about one")
		get_tree().quit(1)
		return
	_sel = settlements[0]
	_sel_name = String(_sel.get("name", ""))

	# == T1 -- arm Journey OVER A SELECTION (the headline) =====================
	app.select_domain("civilization")
	await _frames(3)
	rd.on_settlement_selected(_sel, 0)
	await _frames(4)
	_check("T1 before: the dock shows the settlement",
		_title() == "Settlement" and _name_shown(),
		"title=%s name=%s headers=%s" % [_title(), _name_shown(), _headers()])
	var settle_before := _hdr("SETTLEMENT")

	await _arm_journey()
	print("JP8 [T1] headers after arming Journey: %s" % [_headers()])
	_check("T1: the selected settlement's name SURVIVES arming Journey",
		_name_shown(), "looking for '%s'; texts=%d" % [_sel_name, _texts().size()])
	_check("T1: the dock title still names the selection, not the tool",
		_title() == "Settlement", "title=%s" % _title())
	_check("T1: the Journey section is on screen", _jp() >= 0,
		"headers=%s" % [_headers()])
	_check("T1: it is APPENDED -- below the selection, not instead of it",
		_hdr("SETTLEMENT") >= 0 and _jp() > _hdr("SETTLEMENT"),
		"settlement@%d journey@%d" % [_hdr("SETTLEMENT"), _jp()])
	_check("T1: `_context` is untouched by the arm",
		String(rd._context) == "settlement", "_context=%s" % rd._context)
	_check("T1: the collapsed readout still reports the SELECTION",
		_readout() == _sel_name, "readout=%s" % _readout())
	_check("T1: `_tool_section()` answers TOOL_JOURNEY",
		String(rd._tool_section()) == "journey", "section=%s" % rd._tool_section())

	# == T2 -- the disarm path (guarded before; re-checked here) ================
	app.arm_tool("inspect")
	await _frames(6)
	_check("T2: disarming Journey drops its section", _jp() < 0,
		"headers=%s" % [_headers()])
	_check("T2: disarming Journey LEAVES THE SELECTION (no reset to Sample)",
		_name_shown() and _title() == "Settlement" and _hdr("SETTLEMENT") == settle_before,
		"title=%s name=%s headers=%s" % [_title(), _name_shown(), _headers()])

	# == T3 -- armed with NOTHING selected ====================================
	rd.on_settlement_selected(null, -1)
	await _frames(4)
	await _arm_journey()
	_check("T3: with nothing selected the Journey section still appends",
		_jp() >= 0, "headers=%s" % [_headers()])
	_check("T3: the title falls back to Sample, not to Journey",
		_title() == "Sample", "title=%s" % _title())
	var jv = rd._journey_view
	_check("T3: the collapsed readout reports the plan when nothing is selected",
		jv != null and _readout() == String(jv.readout_text()),
		"readout=%s  view=%s" % [_readout(), String(jv.readout_text()) if jv != null else "<null>"])

	# == T4 -- arm Journey OVER A LIVE SCULPT DRAFT ===========================
	app.arm_tool("inspect")
	app.select_domain("world")
	await _frames(4)
	var made := false
	if bridge.sculpt_begin_stroke():
		for i in 6:
			bridge.sculpt_add_point(40.0 + i * 4.0, 30.0 + i * 3.0)
		made = bridge.sculpt_end_stroke() >= 0
	var n_draft: int = bridge.sculpt_stamp_count()
	_check("T4 setup: an uncommitted sculpt draft exists", made and n_draft > 0,
		"stamps=%d" % n_draft)
	rd._rebuild()
	await _frames(3)
	_check("T4 control: the draft's Stamp stack draws in WORLD (its own domain)",
		_hdr("STAMP") >= 0, "headers=%s" % [_headers()])
	app.select_domain("civilization")
	await _frames(4)
	var stack_in_civ_before := _hdr("STAMP")
	await _arm_journey()
	_check("T4: arming Journey does not destroy the draft",
		bridge.sculpt_stamp_count() == n_draft,
		"before=%d after=%d" % [n_draft, bridge.sculpt_stamp_count()])
	_check("T4: arming Journey changes nothing about the stack's visibility in CIVIL",
		_hdr("STAMP") == stack_in_civ_before,
		"civ before arm=%d after arm=%d (world-gated either way)" % [stack_in_civ_before, _hdr("STAMP")])
	app.arm_tool("inspect")
	app.select_domain("world")
	await _frames(5)
	_check("T4: back in WORLD the draft and its Stamp stack are both intact",
		_hdr("STAMP") >= 0 and bridge.sculpt_stamp_count() == n_draft,
		"stamps=%d headers=%s" % [bridge.sculpt_stamp_count(), _headers()])
	bridge.sculpt_discard()
	await _frames(3)

	# == T5 -- arm Journey OVER A LIVE WAY DRAFT ==============================
	app.select_domain("civilization")
	await _frames(4)
	app.arm_tool("way")
	await _frames(4)
	## `InfrastructureWorkspace` is NOT in `app._workspaces` -- since the
	## 2026-08-20 domain merge it is a nested instance owned by
	## `CivilizationWorkspace._infra`, which is the registered one. Reaching for
	## the wrong owner made this whole case silently no-op on its first run
	## (headers=["SAMPLE"], roads 46 -> 46, no draft ever created).
	var infra = null
	for w in app._workspaces:
		if w.get_script() != null and String(w.get_script().resource_path).ends_with("civilization_workspace.gd"):
			infra = w._infra
	_check("T5 setup: the nested InfrastructureWorkspace was reachable", infra != null)
	if infra != null:
		infra._way_click(gs.x * 0.20, gs.y * 0.30)
		infra._way_click(gs.x * 0.35, gs.y * 0.40)
	await _frames(4)
	var ways_before: int = bridge.roads().size()
	_check("T5 setup: a live way draft draws its own section",
		_hdr("WAY") >= 0, "headers=%s" % [_headers()])
	await _arm_journey()
	var ways_after: int = bridge.roads().size()
	_check("T5: the way draft is COMMITTED by the arm, not silently dropped",
		ways_after > ways_before, "roads %d -> %d" % [ways_before, ways_after])
	_check("T5: the Journey section is what draws in the tool slot now",
		_jp() >= 0 and _hdr("WAY") < 0, "headers=%s" % [_headers()])

	# == T6 -- across a DOMAIN SWITCH =========================================
	rd.on_settlement_selected(_sel, 0)
	await _frames(3)
	await _arm_journey()
	_check("T6 before: Journey appended under the selection", _jp() > _hdr("SETTLEMENT"),
		"headers=%s" % [_headers()])
	app.select_domain("world")
	await _frames(6)
	_check("T6: leaving CIVIL drops the Journey section", _jp() < 0,
		"armed=%s headers=%s" % [app.armed_tool, _headers()])
	_check("T6: leaving CIVIL leaves the selection alone",
		_name_shown() and _title() == "Settlement",
		"title=%s headers=%s" % [_title(), _headers()])
	app.select_domain("civilization")
	await _frames(8)
	_check("T6: returning to CIVIL brings the Journey section back appended",
		_jp() >= 0 and _jp() > _hdr("SETTLEMENT"),
		"armed=%s headers=%s" % [app.armed_tool, _headers()])

	# == T7 -- arm-another-tool while Journey is armed (the exit) =============
	app.arm_tool("label")
	await _frames(6)
	_check("T7: arming Label displaces Journey in the ONE tool slot",
		_hdr("ANNOTATION") >= 0 and _jp() < 0, "headers=%s" % [_headers()])
	_check("T7: and still leaves the selection alone",
		_name_shown() and _title() == "Settlement", "title=%s" % _title())

	# == T8 -- after a REGENERATE =============================================
	await _arm_journey()
	_check("T8 before: Journey appended", _jp() >= 0, "headers=%s" % [_headers()])
	var rc_before: int = bridge.route_count()
	app._run_pipeline()
	waited = 0
	while bridge.generating and waited < 1800:
		await get_tree().process_frame
		waited += 1
	await _frames(10)
	print("JP8 [T8] after regenerate: armed=%s domain=%s route_count %d -> %d headers=%s"
		% [app.armed_tool, app.active_domain(), rc_before, bridge.route_count(), _headers()])
	_check("T8: the dock does not crash or strand a stale plan section",
		_jp() < 0 or bridge.route_count() > 0,
		"route_count=%d journey@%d" % [bridge.route_count(), _jp()])

	# == Width -- does appending widen the dock? ==============================
	var body: Control = app.right_dock_body
	rd.on_settlement_selected(_sel, 0)
	app.arm_tool("inspect")
	app.select_domain("civilization")
	await _frames(6)
	var w_sel := body.get_combined_minimum_size().x
	rd.on_settlement_selected(null, -1)
	await _arm_journey()
	var w_jp := body.get_combined_minimum_size().x
	rd.on_settlement_selected(_sel, 0)
	await _frames(6)
	var w_both := body.get_combined_minimum_size().x
	var dock_w := float(DccTheme.role_px("w_right_dock"))
	print("JP8 [W] right_dock_body min.x -- selection only=%.1f  journey only=%.1f  both=%.1f  |  w_right_dock=%.1f"
		% [w_sel, w_jp, w_both, dock_w])
	print("JP8 [W] ancestors: scroll=%.1f  right_dock=%.1f  window=%s"
		% [app._right_dock_scroll.get_combined_minimum_size().x if app._right_dock_scroll != null else -1.0,
		app.right_dock.get_combined_minimum_size().x if app.right_dock != null else -1.0,
		DisplayServer.window_get_size()])
	_check("W: appending does not widen the dock past the wider of its two parts",
		w_both <= maxf(w_sel, w_jp) + 0.5,
		"both=%.1f max(sel,jp)=%.1f" % [w_both, maxf(w_sel, w_jp)])

	print("JP8 DONE  fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
