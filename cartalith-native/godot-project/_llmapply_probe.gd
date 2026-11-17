extends Node
## **The Assistant's edit APPLY PATH, tested directly** (`LLM_ASSISTANT_SCOPE.md` LLM-7, s.5h items 2, 4 and 6;
## Ruling DR). `_llmedit_probe.gd` drives the same path through the card and the dialog window, and has gone stale
## against the PC-6 dock (it reads `_win.visible`, which the docked Ask tab never sets); this probe reaches the
## apply path without the window so it can carry the apply-path mutation run (`STATUS.md`, LLM-7 entry):
##   * the shell's writer `llm_edit.gd::apply` (region-id check, group begin, abort on a refused engine call,
##     commit) and `llm_edit.gd::blocked` (generating, finalised, paint under the region);
##   * the engine's `WorldGen.region_edit_apply` (bake check, paint check, undo snapshot, redo tail, ledger row,
##     write-back) called by that writer;
##   * the two Rust gate rules of `LlmStream::prepare_edit` that guard the request (`region_context_off`,
##     `edit_local_only`), by `run.begin`, with an in-test loopback server for the positive control only.
##   desktop: godot --path . --resolution 1600x900 res://_llmapply_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1700x1000 res://_llmapply_probe.tscn -- --force-touch --density=1.0 --shots <dir>
##   phone:   godot --path . --resolution 393x852 res://_llmapply_probe.tscn -- --force-touch --shots <dir>
## **Windowed, never `--headless`**, with a wiped user dir. Arguments: `--shots <dir>`, `--force-touch`,
## `--density D` (the shell's), `--failfast` (the mutation harness); anything else exits 2.
##
## What it protects (each check names it):
##   1. HAPPY PATH on 3 seeds: ONE group is added, the field changes, the changed-cell count equals the preview's,
##      the height stack grows by exactly one, parameters, hand ways and colour paint are untouched, no generate
##      starts; ONE Edit > Undo restores the field digest, Redo re-applies, Undo again restores.
##   2. REDO TAIL: an apply after an undo empties the engine's redo stack.
##   3. REGION ID: a region painted over after the proposal is refused, with nothing opened or written.
##   4. REFUSED ENGINE CALL: an over-cap edit rolls the group back: no group, none open, field and stack as before.
##   5. ENGINE GATES: committed colour paint under the region and a finalised world each refuse with nothing
##      written and no undo step; `blocked` names both.
##   6. GENERATING: `blocked` names a running generate.
##   8. RULING FW ITEM 2 (colour): a colour is one area mark in the colour-paint document, undone in one step with
##      a raise in the same group, a refused colour (a hole) rolls the raise back, E7 is not triggered by colour marks,
##      and the card draws the pictures and its Apply works.
##   7. RUST GATES: `run.begin` refuses an edit request with the region source off (`region_context_off`) and
##      with a hosted endpoint (`edit_local_only`), each dialling nothing; the allowed case starts (control).
const ENV_VAR := "CART_LLM_PROBE_KEYVAR"
const SECRET := "sk-PROBE-SECRET-apply-0123456789"
const SEEDS := [9137, 24601, 71077345]
const _EDIT := preload("res://shell/llm_edit.gd")
const _REGION := preload("res://shell/llm_region.gd")

var _fails := 0
var _checks := 0
var _failfast := false
## `--only8`: boot, generate once and run section 8 only (a quick loop while building Ruling FW; the full run is the gate).
var _only8 := false
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _win
var _server: LlmMockServer
var _port := 0
var _gen_started := 0


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1
		if _failfast:
			_finish()


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _finish() -> void:
	OS.unset_environment(ENV_VAR)
	if _server != null:
		_server.stop()
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _digest() -> Dictionary:
	return (_bridge.edit_state_digest() as Dictionary).duplicate()


func _depth() -> int:
	return int(_bridge.undo_stats().get("depth", -1))


func _redo_depth() -> int:
	return int(_bridge.undo_stats().get("redo_depth", -1))


func _generate(seed_: int) -> void:
	_bridge.generate({"seed": seed_, "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": false})
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(45)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()


## A region far from a coast and from the highest ground, so a raise has headroom on every seed (the edit probe's).
func _paint_region() -> void:
	_REGION.clear(_app)
	_REGION.brush = 16.0
	_REGION.install(_app)
	_REGION.painting = true
	for x in range(60, 140, 8):
		_app._drag_handlers[_REGION.TOOL_ID].call(float(x), 80.0)
	_app._release_handlers[_REGION.TOOL_ID].call(140.0, 80.0, true)
	for y in range(60, 100, 8):
		_app._drag_handlers[_REGION.TOOL_ID].call(100.0, float(y))
	_app._release_handlers[_REGION.TOOL_ID].call(100.0, 100.0, true)
	_REGION.painting = false


func _rid() -> String:
	return String(_REGION.facts(_bridge).get("region", {}).get("id", ""))


## The validated rows as Rust hands them over (`edit_verdict_dict`): a raise then a smooth.
func _rows(amount := 300.0) -> Array:
	return [{"operation": "raise", "falloff": "linear", "softness": 1.0, "amount_m": amount},
		{"operation": "smooth", "falloff": "linear", "softness": 1.0, "strength": 0.5, "passes": 2}]


func _info() -> Dictionary:
	return {"region_id": _rid(), "summary": "probe edit"}


func _configure(over: Dictionary = {}) -> void:
	var s := {"enabled": true, "commands": false, "prose": false, "edit": true, "ctx_region": true, "local_allowed": true,
		"preset_id": "ollama", "base_url": "http://127.0.0.1:%d/v1" % _port, "model": "mock-model",
		"key_source": "env", "env_var": ENV_VAR, "ctx_params": false}
	s.merge(over, true)
	for k in s:
		DccSettings.set_llm_value(k, s[k])
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % _port)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--force-touch":
			pass
		elif args[i] == "--density" and i + 1 < args.size():
			i += 1
		elif args[i].begins_with("--density="):
			pass
		elif args[i] == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		elif args[i] == "--failfast":
			_failfast = true
		elif args[i] == "--only8":
			_only8 = true
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch, --density, --shots <dir>, --failfast, --only8" % args[i])
			get_tree().quit(2)
			return
		i += 1
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed; this probe saves frames and boots the real shell")
		get_tree().quit(2)
		return
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	OS.set_environment(ENV_VAR, SECRET)
	_server = LlmMockServer.new()
	add_child(_server)
	_ok(_server.start(), "the mock server listens")
	_port = _server.port
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, viewport %s ==" % [_mode, get_viewport().get_visible_rect().size])
	_bridge = _app.bridge
	_win = _app.assistant_window
	_ok(_bridge.llm_api and _win != null and _app.get("undo_groups") != null and _bridge.has_method("region_edit_apply"), "the library has LlmBridge, the grouped undo and the region functions")
	if not _bridge.llm_api or _win == null:
		_finish()
		return
	_bridge.generation_started.connect(func(): _gen_started += 1)
	var ug = _app.undo_groups
	if _only8:
		await _generate(SEEDS[0])
		_paint_region()
		await _section8(ug)
		_finish()
		return

	for si in SEEDS.size():
		var seed_: int = SEEDS[si]
		print("== 1. seed %d: one Apply, one group, one undo ==" % seed_)
		await _generate(seed_)
		_paint_region()
		var valid := _rows()
		var info := _info()
		var pv: Dictionary = _EDIT.preview(_bridge, valid)
		_ok(bool(pv.get("ok", false)) and int(pv["changed"]) > 0, "seed %d: the preview says %d cells change" % [seed_, int(pv.get("changed", 0))])
		_ok(_EDIT.blocked(_bridge, {"preview": pv}) == "", "seed %d: nothing blocks Apply" % seed_)
		var d0 := _digest()
		var n0: int = ug.groups.size()
		var depth0 := _depth()
		var gen0 := _gen_started
		var r: Dictionary = _EDIT.apply(_app, _bridge, ug, info, valid)
		_ok(bool(r.get("ok", false)), "seed %d: apply answered ok: %s" % [seed_, str(r.get("reason", ""))])
		var d1 := _digest()
		_ok(d1["field"] != d0["field"], "seed %d: the height field changed" % seed_)
		_ok(ug.groups.size() == n0 + 1 and String(ug.undo_label()).begins_with("Assistant:"), "seed %d: exactly ONE group was added: '%s'" % [seed_, ug.undo_label()])
		_ok(int(r.get("changed", -1)) == int(pv["changed"]), "seed %d: the engine changed exactly the preview's %d cells (%d)" % [seed_, int(pv["changed"]), int(r.get("changed", -1))])
		_ok(_depth() == depth0 + 1, "seed %d: the height stack grew by exactly one step (%d to %d)" % [seed_, depth0, _depth()])
		_ok(d1["params"] == d0["params"] and d1["ways_manual"] == d0["ways_manual"] and d1["colour_paint"] == d0["colour_paint"], "seed %d: parameters, hand-drawn ways and colour paint are untouched" % seed_)
		_ok(_gen_started == gen0 and not ug.is_open(), "seed %d: no generate started and no group is left open" % seed_)
		_app.undo_last()
		await _settle(8)
		_ok(_digest()["field"] == d0["field"] and _depth() == depth0, "seed %d: ONE Edit > Undo restored the field digest and the stack depth" % seed_)
		_app.redo_last()
		await _settle(8)
		_ok(_digest()["field"] == d1["field"], "seed %d: Edit > Redo re-applied it (digest equal to the applied state)" % seed_)
		_app.undo_last()
		await _settle(8)
		_ok(_digest()["field"] == d0["field"], "seed %d: undone again, back to the original" % seed_)
		if si == 0:
			print("== 2. an apply after an undo empties the redo tail ==")
			_ok(_redo_depth() >= 1, "control: the undo left a redo tail (%d)" % _redo_depth())
			var r2: Dictionary = _EDIT.apply(_app, _bridge, ug, _info(), valid)
			_ok(bool(r2.get("ok", false)) and _redo_depth() == 0, "the second apply emptied the engine's redo stack (%d)" % _redo_depth())
			_app.undo_last()
			await _settle(8)

	await _generate(SEEDS[0])
	_paint_region()
	print("== 3. the region must still be the region ==")
	var info3 := _info()
	_REGION.dabs.append(Vector3(40.0, 40.0, 16.0))
	var d3 := _digest()
	var n3: int = ug.groups.size()
	var depth3 := _depth()
	var r3: Dictionary = _EDIT.apply(_app, _bridge, ug, info3, _rows(20.0))
	_ok(not bool(r3.get("ok", true)) and String(r3.get("reason", "")).contains("region changed"), "a region painted over after the proposal is refused: '%s'" % String(r3.get("reason", "")).left(60))
	_ok(_digest() == d3 and ug.groups.size() == n3 and not ug.is_open() and _depth() == depth3, "...with nothing written, no group made and none open")
	_REGION.dabs.pop_back()

	print("== 4. an engine refusal rolls the group back ==")
	var r4: Dictionary = _EDIT.apply(_app, _bridge, ug, _info(), _rows(9999.0))
	_ok(not bool(r4.get("ok", true)), "an over-cap edit is refused: '%s'" % String(r4.get("reason", "")).left(70))
	_ok(_digest() == d3 and ug.groups.size() == n3 and not ug.is_open() and _depth() == depth3, "...no group, none open, field and stack as before")

	print("== 4b. an edit that would change no cell is refused ==")
	## No generated world has a flat patch to smooth, so the edit is a raise too small for a 32-bit height to hold
	## (a few millimetres on a 4 km peak is below one ULP): if the validator lets it through, the engine must
	## refuse it rather than push an empty undo step.
	var d4b := _digest()
	var depth4b := _depth()
	var refused4b := ""
	for tiny in [0.0001, 0.001]:
		var tiny_ops: Array = [{"operation": "raise", "falloff": "constant", "softness": 0.0, "amount_m": tiny}]
		var e4b: Dictionary = _bridge.region_edit_apply(_REGION.spec(), tiny_ops, "probe")
		print("  (a %.4f m raise: ok=%s reason='%s')" % [tiny, str(e4b.get("ok", "?")), String(e4b.get("reason", "")).left(80)])
		if not bool(e4b.get("ok", true)) and String(e4b.get("reason", "")).contains("would not change any cell"):
			refused4b = String(e4b["reason"])
			break
		if bool(e4b.get("ok", false)):
			_app.undo_last()
			await _settle(6)
	_ok(refused4b != "", "the engine refuses an edit that changes no cell: '%s'" % refused4b.left(60))
	_ok(_digest() == d4b and _depth() == depth4b, "...pushing no undo step")

	print("== 5. the engine's gates ==")
	var spec: Dictionary = _REGION.spec()
	var ops: Array = _EDIT.ops_of(_rows(100.0))
	var good: Dictionary = _bridge.region_edit_preview(spec, ops)
	_ok(bool(good.get("ok", false)) and int(good.get("paint_cells", -1)) == 0, "control: the engine previews the edit and finds no paint under the region")
	## Committed colour paint under the region.
	_bridge.paint_set_layer("biome")
	_bridge.paint_set_brush(1, 6.0, 1.0, 0.0, false, false)
	_bridge.paint_stroke_at(100.0, 80.0)
	_bridge.paint_commit()
	var d5 := _digest()
	var depth5 := _depth()
	var pv5: Dictionary = _EDIT.preview(_bridge, _rows(100.0))
	_ok(int(pv5.get("paint_cells", 0)) > 0, "the preview counts the committed paint under the region (%d cells)" % int(pv5.get("paint_cells", 0)))
	_ok(_EDIT.blocked(_bridge, {"preview": pv5}).contains("painted cells"), "`blocked` names the paint: '%s'" % _EDIT.blocked(_bridge, {"preview": pv5}).left(50))
	var e5: Dictionary = _bridge.region_edit_apply(spec, ops, "probe")
	_ok(not bool(e5.get("ok", true)) and String(e5.get("reason", "")).contains("painted cells"), "the engine call refuses over committed paint: '%s'" % String(e5.get("reason", "")).left(60))
	_ok(_digest()["field"] == d5["field"] and _depth() == depth5, "...writing nothing and pushing no undo step")
	await _generate(SEEDS[0])
	_paint_region()
	## A finalised world.
	_bridge.set_atlas_tile_size(256)
	_bridge.atlas_clear()
	var baked: Dictionary = _bridge.bake_all(1)
	var fin: bool = bool(baked.get("ok", false)) and _bridge.set_finalized(true)
	_ok(fin and _bridge.is_finalized(), "control: the world is finalised (%s)" % str(baked.get("ok", false)))
	if fin:
		var d6 := _digest()
		var depth6 := _depth()
		var pv6: Dictionary = _EDIT.preview(_bridge, _rows(100.0))
		_ok(_EDIT.blocked(_bridge, {"preview": pv6}).contains("finalised"), "`blocked` names the finalised world: '%s'" % _EDIT.blocked(_bridge, {"preview": pv6}).left(50))
		var e6: Dictionary = _bridge.region_edit_apply(_REGION.spec(), _EDIT.ops_of(_rows(100.0)), "probe")
		_ok(not bool(e6.get("ok", true)) and String(e6.get("reason", "")).contains("finalized"), "the engine call refuses on a finalised world: '%s'" % String(e6.get("reason", "")).left(60))
		_ok(_digest()["field"] == d6["field"] and _depth() == depth6, "...writing nothing and pushing no undo step")
		_bridge.set_finalized(false)

	print("== 6. a running generate blocks Apply ==")
	_app.new_world_dialog.generate_checked()
	await _settle(3)
	_ok(_bridge.generating and _EDIT.blocked(_bridge, {"preview": good}).contains("generating"), "`blocked` names the running generate: '%s'" % _EDIT.blocked(_bridge, {"preview": good}).left(50))
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(45)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	_paint_region()

	## The same world on every form (section 6 regenerated through the New World dialog, whose defaults differ per form).
	await _generate(SEEDS[0])
	_paint_region()
	await _section8(ug)
	_paint_region()

	print("== 7. the Rust gate rules ==")
	_win.open()
	await _settle(8)
	_ok(_win.run != null, "the panel's run exists")
	var facts := _REGION.facts_json(_bridge)
	var ask := {"request": "raise it", "edit": facts, "max_tokens": 256}
	var acc0: int = _server.accepted
	_configure({"ctx_region": false})
	var no_src: Dictionary = _win.run.begin(DccSettings.llm_gate_dict(), "edit", DccSettings.llm_consented_origins(), ask)
	_ok(not bool(no_src.get("ok", true)) and String(no_src.get("code", "")) == "region_context_off" and _server.accepted == acc0, "the region source off is refused in Rust (code %s), nothing dialled" % str(no_src.get("code", "")))
	_configure({"preset_id": "openai", "base_url": "https://hosted.invalid/v1", "model": "gpt-x", "remote_allowed": true, "local_allowed": false})
	DccSettings.llm_add_consent("https://hosted.invalid")
	var hosted: Dictionary = _win.run.begin(DccSettings.llm_gate_dict(), "edit", DccSettings.llm_consented_origins(), ask)
	_ok(not bool(hosted.get("ok", true)) and String(hosted.get("code", "")) == "edit_local_only" and _server.accepted == acc0, "a hosted endpoint is refused in Rust (code %s), nothing dialled" % str(hosted.get("code", "")))
	_configure({})
	var allowed: Dictionary = _win.run.begin(DccSettings.llm_gate_dict(), "edit", DccSettings.llm_consented_origins(), ask)
	_ok(bool(allowed.get("ok", false)), "control: the allowed local request starts (code %s)" % str(allowed.get("code", "")))
	_win.run.cancel()
	await _settle(30)
	_finish()


## ---------------------------------------------------------------- 8. Ruling FW: the stamps and the colour

const GRID_W := 256
const GRID_H := 164


## Every cell's raw height, in grid order. The engine's `sample_cell` is the only per-cell reader the shell has.
func _snapshot() -> PackedFloat64Array:
	var out := PackedFloat64Array()
	out.resize(GRID_W * GRID_H)
	for y in GRID_H:
		for x in GRID_W:
			out[y * GRID_W + x] = float((_bridge.sample_cell(x, y) as Dictionary).get("elevation", NAN))
	return out


## Whether cell (x, y)'s centre is inside the spec's dabs: the Rust rasteriser's own rule (`<=`, centre `+ 0.5`).
func _inside(spec: Dictionary, x: int, y: int) -> bool:
	for d in spec.get("dabs", []):
		var dx := float(x) + 0.5 - float(d[0])
		var dy := float(y) + 0.5 - float(d[1])
		if dx * dx + dy * dy <= float(d[2]) * float(d[2]):
			return true
	return false


## How many cells differ between two snapshots, split by inside and outside the spec.
func _diff(a: PackedFloat64Array, b: PackedFloat64Array, spec: Dictionary) -> Vector2i:
	var inside_n := 0
	var outside_n := 0
	for y in GRID_H:
		for x in GRID_W:
			var i := y * GRID_W + x
			if a[i] != b[i]:
				if _inside(spec, x, y):
					inside_n += 1
				else:
					outside_n += 1
	return Vector2i(inside_n, outside_n)


func _stamp_rows(kind: String, amount := 200.0) -> Array:
	## (Item 2 only: the height row that rides beside the colour is a raise, whatever `kind` says.)
	return [{"operation": "raise", "falloff": "smooth", "softness": 0.5, "amount_m": amount,
		"text": "probe raise", "defaulted": false}]


func _paint_rows(colour := "red", blend := "multiply", opacity := 0.8) -> Array:
	return [{"operation": "paint", "falloff": "smooth", "softness": 0.0, "colour": colour, "blend": blend, "opacity": opacity,
		"text": "probe paint", "defaulted": false}]


func _info_for(pv: Dictionary) -> Dictionary:
	var i := _info()
	return i


func _section8(ug) -> void:
	print("== 8. Ruling FW item 2: the colour, through the shell's writer ==")
	_ok(ug.paint_tool() != null, "the colour-paint tool exists for the group to collect")
	var spec: Dictionary = _REGION.spec()
	print("-- 8d. colour paint --")
	var rows_p := _paint_rows()
	var pv_p: Dictionary = _EDIT.preview(_bridge, rows_p)
	_ok(bool(pv_p.get("ok", false)) and pv_p.has("paint") and int(pv_p["paint"]["cells_recoloured"]) > 0, "the colour preview recolours %d land cells in %d piece(s)" % [int(pv_p.get("paint", {}).get("cells_recoloured", 0)), int(pv_p.get("paint", {}).get("pieces", 0))])
	_ok(_EDIT.blocked(_bridge, {"preview": pv_p}) == "", "nothing blocks the paint")
	var s_p0 := _snapshot()
	var d_p0 := _digest()
	var top_p0: int = ug.undo_top_id()
	var depth_p0 := _depth()
	var r_p: Dictionary = _EDIT.apply(_app, _bridge, ug, _info(), rows_p)
	_ok(bool(r_p.get("ok", false)) and bool(r_p.get("painted", false)), "paint applied: %s" % str(r_p.get("reason", "")))
	var d_p1 := _digest()
	_ok(d_p1["colour_paint"] != d_p0["colour_paint"] and d_p1["field"] == d_p0["field"], "the colour-paint document changed and the height field did not")
	_ok(_diff(s_p0, _snapshot(), spec) == Vector2i(0, 0) and _depth() == depth_p0, "no cell moved and the height stack is unchanged")
	_ok(ug.undo_top_id() != top_p0 and ug.wants_undo(), "exactly ONE group was added for the colour")
	var info_doc: Dictionary = JSON.parse_string(String(_bridge.colour_paint_document()))
	var marks_all: Array = []
	for lay in info_doc.get("layers", [{"marks": info_doc.get("marks", [])}]):
		marks_all.append_array(lay.get("marks", []))
	var last: Dictionary = marks_all[-1] if not marks_all.is_empty() else {}
	var rgb_got: Array = last.get("rgb", [])
	_ok(not last.is_empty() and String(last.get("shape", {}).get("kind", "")) == "area" and String(last.get("blend", "")) == "multiply" and rgb_got.size() == 3 and int(rgb_got[0]) == 200 and int(rgb_got[1]) == 40 and int(rgb_got[2]) == 40 and bool(last.get("land_only", false)), "the document's newest mark is an area, multiply, the named red, land-only: %s" % str(last).left(160))
	## E7 reads the world's own override layer, not Ruling CN's marks: a ridge over the new colour is not blocked.
	var pv_e7: Dictionary = _EDIT.preview(_bridge, _stamp_rows("ridge"))
	_ok(int(pv_e7.get("paint_cells", -1)) == 0 and _EDIT.blocked(_bridge, {"preview": pv_e7}) == "", "colour marks under the region do not trigger the height edit's paint refusal (E7 is the override layer)")
	_app.undo_last()
	await _settle(8)
	_ok(_digest()["colour_paint"] == d_p0["colour_paint"], "ONE Edit > Undo took the colour back (document digest equal)")
	_app.redo_last()
	await _settle(8)
	_ok(_digest()["colour_paint"] == d_p1["colour_paint"], "Edit > Redo put it back")
	_app.undo_last()
	await _settle(8)

	print("-- 8e. a ridge and a colour in one proposal are one undo step; a refused colour rolls the ridge back --")
	var rows_m := _stamp_rows("ridge") + _paint_rows("blue", "colour", 1.0)
	var pv_m: Dictionary = _EDIT.preview(_bridge, rows_m)
	var d_m0 := _digest()
	var top_m0: int = ug.undo_top_id()
	var r_m: Dictionary = _EDIT.apply(_app, _bridge, ug, _info_for(pv_m), rows_m)
	_ok(bool(r_m.get("ok", false)) and int(r_m.get("changed", 0)) > 0 and bool(r_m.get("painted", false)) and ug.undo_top_id() != top_m0 and ug.wants_undo(), "the mixed proposal applied as ONE group: %s" % str(r_m.get("reason", "")))
	_app.undo_last()
	await _settle(8)
	_ok(_digest() == d_m0, "ONE Edit > Undo took both back (field and colour digests equal)")
	## A region with a hole: the ridge applies, the colour cannot, and the whole group rolls back.
	_REGION.clear(_app)
	_REGION.brush = 8.0
	_REGION.install(_app)
	for k in 24:
		var a := TAU * float(k) / 24.0
		_REGION.dabs.append(Vector3(120.0 + 26.0 * cos(a), 80.0 + 26.0 * sin(a), 8.0))
	var spec_h: Dictionary = _REGION.spec()
	var d_h := _digest()
	var n_h: int = ug.groups.size()
	var pv_h: Dictionary = _bridge.region_edit_preview(spec_h, _EDIT.ops_of(_stamp_rows("ridge")))
	var pv_h_paint: Dictionary = _bridge.region_paint_preview(spec_h, {"colour": "red", "blend": "colour", "opacity": 0.8})
	_ok(bool(pv_h.get("ok", false)), "control: the ring-shaped region previews a ridge")
	_ok(not bool(pv_h_paint.get("ok", true)) and String(pv_h_paint.get("reason", "")).contains("hole"), "a region with a hole refuses the colour, with its reason: '%s'" % String(pv_h_paint.get("reason", "")).left(70))
	var info_h := {"region_id": _rid(), "summary": "probe"}
	var r_h: Dictionary = _EDIT.apply(_app, _bridge, ug, info_h, _stamp_rows("ridge") + _paint_rows())
	_ok(not bool(r_h.get("ok", true)) and String(r_h.get("reason", "")).contains("hole"), "the mixed apply is refused at the colour: '%s'" % String(r_h.get("reason", "")).left(60))
	_ok(_digest() == d_h and ug.groups.size() == n_h and not ug.is_open(), "...the ridge's height step was rolled back with the group (digests equal, no group, none open)")
	_paint_region()

	print("-- 8f. the card, through the window's own Apply button --")
	_win.open()
	await _settle(8)
	var verdict := {"summary": "A ridge and a red stain", "region_id": _rid(), "valid": _stamp_rows("ridge") + _paint_rows("red", "multiply", 0.8), "rejected": []}
	_win._on_edit_proposal({"edit": verdict})
	await _settle(8)
	var card: Dictionary = _win.cards[_win.cards.size() - 1]
	_ok(String(card.get("kind", "")) == "edit" and card["apply"] != null and not (card["apply"] as Button).disabled, "the card has an enabled Apply: %s" % str(_EDIT.blocked(_bridge, card)))
	for t in ["edit_thumb_before", "edit_thumb_after", "edit_thumb_paint_before", "edit_thumb_paint_after"]:
		_ok((card["box"] as Control).find_child(t, true, false) != null, "the card draws '%s'" % t)
	await _shot_card("8_card", card)
	var d_f0 := _digest()
	var top_f0: int = ug.undo_top_id()
	(card["apply"] as Button).pressed.emit()
	await _settle(10)
	_ok(String(card.get("state", "")) == "applied" and ug.undo_top_id() != top_f0 and ug.wants_undo() and _digest() != d_f0, "the Apply button applied the proposal as one group (card state %s)" % String(card.get("state", "")))
	_ok(String(card.get("applied_text", "")).contains("coloured"), "the card says the region was coloured: '%s'" % String(card.get("applied_text", "")).left(80))
	await _shot_card("8_card_applied", card)
	await _shot_map("8_map_after")
	_app.undo_last()
	await _settle(8)
	_ok(_digest() == d_f0, "Edit > Undo takes the card's whole change back")


func _shot_card(tag: String, card: Dictionary) -> void:
	if _shots == "":
		return
	await _settle(4)
	var c: Control = card["box"]
	var p := c.get_parent()
	while p != null:
		if p is ScrollContainer:
			(p as ScrollContainer).ensure_control_visible(c)
			await _settle(2)
		p = p.get_parent()
	var path := _shots.path_join("fwedit_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


## The whole window as the user sees it (the map with the applied ridge and colour in it).
func _shot_map(tag: String) -> void:
	if _shots == "":
		return
	await _settle(6)
	var path := _shots.path_join("fwedit_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)
