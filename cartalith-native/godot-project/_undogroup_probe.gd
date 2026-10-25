extends Node
## **LLM-5: the grouped undo (`shell/undo_group.gd`), through the real app.**
##
## Protects (`LLM_ASSISTANT_SCOPE.md` LLM-5 bar, s.5b.4). Every check names what it guards:
##  A. on each of three seeds, ONE group of seven mixed members -- a height edit (Carve fjords),
##     a way (a routed road drag), an appearance edit (a biome colour, through `LookHistory`), a
##     colour-paint stroke and three parameter writes -- undoes in exactly ONE `app.undo_last()`
##     and redoes in ONE `app.redo_last()`; after the undo every part of the world digest (height
##     field, geology column, hand-drawn ways, generated-way identities, parameters, colour paint,
##     look) equals the pre-group digest, and after the redo the post-group one; the walk works
##     twice; and the Edit menu's label is the group's. Positive control: every part moved.
##  B. a failure inside a group rolls the whole group back: (1) at apply time, a clamped
##     parameter write after a carve, a paint stroke and a good write leaves the world equal to
##     before; (2) a refused step part-way through an UNDO leaves the group applied and the world
##     equal to after; (3) the same part-way through a REDO leaves it equal to before.
##  C. single-step undo is unchanged where no group is involved: a carve undoes alone, a look
##     edit undoes alone, the paint tool's own Undo still works, and every guard answers "" with
##     no group.
##  D. no member is ever taken alone: an ungrouped carve after a group is undone first and alone,
##     then the group whole; a group under a later ungrouped paint stroke refuses the plain
##     Edit > Undo that would take its carve (world unchanged, reason shown), the paint tool's
##     own Undo refuses a grouped step, and a History revert into a group is refused.
##  E. caps: `MAX_MEMBERS` (a 25-write group is refused and rolled back, a 24-write one is
##     accepted; literals); `height_room()` is the engine's own step capacity minus the group's
##     height steps.
##  F. settings only after a regenerate: a parameter-only group survives a Generate, its undo
##     restores the settings, leaves the regenerated field untouched and says "(settings only)".
##  G. no regeneration anywhere: `generation_started` fires 0 times across A-E's applies, undos
##     and redos (it fires only for the two deliberate generates of F and the seed setup).
##
## Run WINDOWED (never headless), from a project copy with its own `config/name` and a wiped
## user dir:
##   godot --path . res://_undogroup_probe.tscn -- [--vp WxH] [--force-touch] [--tag T] [--out DIR]
## `--vp` defaults to 1600x900, `--tag` to `undogroup`; `--out DIR` saves a few frames there.
## Result: an assertion. Every line is prefixed `[<tag>] `, checks print `PASS|FAIL  <what>`, the
## last line is `RESULT <tag> checks=<n> fail=<n>`. Exit 0 when no check failed, 1 otherwise,
## 2 for an unknown flag, when headless, or when a premise is missing (`ABORT ...`).

const SEEDS := [4242, 71077345, 24601]
const GRID_W := 256
const GRID_H := 192
const LookHistory := preload("res://shell/look_history.gd")
## The cap `undo_group.gd` documents (`MAX_MEMBERS`), written as a literal so a changed cap is
## noticed here rather than followed.
const CAP := 24

var app: Node
var bridge
var _vp: SubViewport
var _tag := "undogroup"
var _out := ""
var _fail := 0
var _checks := 0
var _gens := 0

func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])

func _check(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fail += 1
	_log("%s  %s" % ["PASS" if cond else "FAIL", what])

func _abort(why: String) -> void:
	_log("ABORT %s" % why)
	_log("RESULT %s checks=%d fail=%d" % [_tag, _checks, _fail + 1])
	get_tree().quit(2)

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	var img := _vp.get_texture().get_image()
	if img != null:
		img.save_png(_out.path_join("%s_%s.png" % [_tag, name]))

# -- fixtures ----------------------------------------------------------------------

func _generate(seed_: int) -> void:
	bridge.generate({"seed": seed_, "width_km": 1200.0, "grid_w": GRID_W, "grid_h": GRID_H,
		"archetype": "", "villages": true, "sea_level": 0.42})
	await bridge.generation_finished
	await _frames(10)

func _tool():
	for ws in app._workspaces:
		if ws is CartographyWorkspace:
			return ws.colour_paint
	return null

## Everything a group can change, as comparable values: the engine digest (field, column,
## ways, params, colour paint) plus the look state the LookHistory records.
func _digest() -> Dictionary:
	var d: Dictionary = bridge.edit_state_digest().duplicate()
	d["look"] = str(LookHistory.snapshot(bridge, ["ramp", "biomes", "style"], RenderWorkspace._preset_appearance_keys())).sha256_text()
	return d

## The parts of `a` and `b` that differ, as a list of keys (empty = equal).
func _diff(a: Dictionary, b: Dictionary) -> Array:
	var out: Array = []
	for k in a.keys() + b.keys():
		if not (k in out) and (not a.has(k) or not b.has(k) or a[k] != b[k]):
			out.append(k)
	return out

## Two land cells `span` apart on one row, for a road drag; Vector2i(-1, -1) pair when none.
func _land_pair(span: int) -> Array:
	for y in range(12, GRID_H - 12, 4):
		for x in range(12, GRID_W - 12 - span, 4):
			var ok := true
			for k in [0, span / 2, span]:
				if String(bridge.world_gen.sample_cell(x + k, y).get("water", "")) != "land":
					ok = false
			if ok:
				return [Vector2(x, y), Vector2(x + span, y)]
	return []

## A routed road drag between two land cells: one `WayDrag` ledger row.
func _road() -> bool:
	var pr := _land_pair(10)
	if pr.is_empty():
		return false
	if not bridge.way_stroke_begin({"way_type": "road"}):
		return false
	var a: Vector2 = pr[0]
	var b: Vector2 = pr[1]
	for i in 11:
		var p := a.lerp(b, i / 10.0)
		bridge.way_stroke_sample(p.x, p.y, i * 16)
	return bool(bridge.way_stroke_end().get("committed", false))

## One biome-colour gesture through the LookHistory, as an editor makes it.
func _look_edit(index: int, c: Color) -> bool:
	var before: Dictionary = LookHistory.snapshot(bridge, ["biomes"])
	bridge.set_biome_color(index, c)
	var after: Dictionary = LookHistory.snapshot(bridge, ["biomes"])
	return app.look_history.commit("Biome colour", after, bridge, before)

## One colour-paint stroke across the map's middle.
func _paint(t) -> bool:
	var kpc := float(bridge.colour_paint_info().get("km_per_cell", 0.0))
	if kpc <= 0.0:
		return false
	return t.commit_line([Vector2(GRID_W * 0.3, GRID_H * 0.5) * kpc, Vector2(GRID_W * 0.6, GRID_H * 0.5) * kpc])

## A float parameter and a value inside its range different from its current value.
func _float_keys(n: int) -> Array:
	var info: Dictionary = bridge.world_gen.get_param_info()
	var out: Array = []
	var keys: Array = info.keys()
	keys.sort()
	for k in keys:
		var d: Dictionary = info[k]
		if String(d["type"]) != "float" or float(d["max"]) <= float(d["min"]):
			continue
		var cur := float(bridge.param_get(k))
		var v := (float(d["min"]) * 0.25 + float(d["max"]) * 0.75)
		if is_equal_approx(v, cur):
			v = (float(d["min"]) * 0.75 + float(d["max"]) * 0.25)
		out.append([String(k), v, d])
		if out.size() == n:
			break
	return out

# -- sections ----------------------------------------------------------------------

func _section_a(seed_: int, first: bool) -> void:
	_log("=== A seed %d: one group of seven mixed members ===" % seed_)
	var t = _tool()
	var keys := _float_keys(3)
	var d0 := _digest()
	var gens0 := _gens
	_check(app.undo_groups.begin("Assistant: drier and lower") == "", "A%d begin opens a group" % seed_)
	var carve: Dictionary = bridge.carve_fjords()
	_check(bool(carve.get("ok", false)), "A%d height member: Carve fjords ran (%s)" % [seed_, str(carve.get("reason", ""))])
	_check(_road(), "A%d way member: a routed road drag committed" % seed_)
	_check(_look_edit(3, Color8(200, 40, 160)), "A%d appearance member: a biome colour gesture pushed one look entry" % seed_)
	_check(_paint(t), "A%d colour-paint member: one stroke" % seed_)
	for k in keys:
		_check(app.undo_groups.write_param(k[0], k[1]), "A%d parameter member: %s -> %s (%s)" % [seed_, k[0], str(k[1]), app.undo_groups.last_reason])
	var c: Dictionary = app.undo_groups.commit()
	_check(bool(c.get("ok", false)) and int(c.get("members", 0)) == 7, "A%d commit: ok with 7 members (%s)" % [seed_, str(c)])
	await _frames(2)
	var d1 := _digest()
	var moved := _diff(d0, d1)
	for part in ["field", "ways_manual", "params", "colour_paint", "look"]:
		_check(part in moved, "A%d positive control: the group moved '%s'" % [seed_, part])
	_check(app.undo_target_label() == "Assistant: drier and lower", "A%d Edit > Undo names the group (%s)" % [seed_, app.undo_target_label()])
	if first:
		await _shot("a_applied")
	app.undo_last()
	await _frames(2)
	var du := _digest()
	_check(_diff(d0, du).is_empty(), "A%d ONE undo restores every part to the pre-group digest (differs: %s)" % [seed_, str(_diff(d0, du))])
	_check(app.redo_target_label() == "Assistant: drier and lower", "A%d Edit > Redo names the group (%s)" % [seed_, app.redo_target_label()])
	if app._status_labels.has("pass"):
		var said: String = (app._status_labels["pass"] as Label).text
		_check(said == "undid assistant: drier and lower", "A%d the status line names the whole group (%s)" % [seed_, said])
	if first:
		await _shot("a_undone")
	app.redo_last()
	await _frames(2)
	var dr := _digest()
	_check(_diff(d1, dr).is_empty(), "A%d ONE redo restores every part to the post-group digest (differs: %s)" % [seed_, str(_diff(d1, dr))])
	app.undo_last()
	await _frames(2)
	_check(_diff(d0, _digest()).is_empty(), "A%d a second undo after the redo is again the pre-group digest" % seed_)
	app.redo_last()
	await _frames(2)
	_check(_diff(d1, _digest()).is_empty(), "A%d a second redo is again the post-group digest" % seed_)
	_check(_gens == gens0, "A%d no generate was started by apply, undo or redo (%d)" % [seed_, _gens - gens0])
	## Leave the world as it was for the next section: one undo.
	app.undo_last()
	await _frames(2)

func _section_b() -> void:
	_log("=== B: a failure inside a group rolls the whole group back ===")
	var t = _tool()
	var keys := _float_keys(2)
	var gens0 := _gens
	## B1 apply-time: a clamped write after three good members.
	var d0 := _digest()
	app.undo_groups.begin("Assistant: clamp")
	bridge.carve_fjords()
	_paint(t)
	_check(app.undo_groups.write_param(keys[0][0], keys[0][1]), "B1 premise: a good write lands")
	var info: Dictionary = keys[1][2]
	var over := float(info["max"]) + absf(float(info["max"]) - float(info["min"])) + 1.0
	_check(not app.undo_groups.write_param(keys[1][0], over), "B1 an out-of-range write (%s = %s) is a failure" % [keys[1][0], str(over)])
	_check(not app.undo_groups.is_open(), "B1 the failure closed the group")
	await _frames(2)
	_check(_diff(d0, _digest()).is_empty(), "B1 the carve, the stroke and the good write were all rolled back (differs: %s)" % str(_diff(d0, _digest())))
	_check(app.undo_groups.groups.filter(func(x): return String(x["label"]) == "Assistant: clamp").is_empty(), "B1 nothing was pushed")
	## B2 undo-time.
	app.undo_groups.begin("Assistant: undo fails")
	bridge.carve_fjords()
	_look_edit(5, Color8(10, 220, 30))
	_paint(t)
	app.undo_groups.write_param(keys[0][0], keys[0][1])
	_check(bool(app.undo_groups.commit().get("ok", false)), "B2 premise: a four-member group committed")
	var d1 := _digest()
	app.undo_groups.fail_at_step = 2
	var r: Dictionary = app.undo_groups.undo()
	_check(not bool(r.get("ok", true)) and String(r.get("reason", "")).contains("test seam"), "B2 a refused third step fails the undo (%s)" % str(r.get("reason", "")))
	await _frames(2)
	_check(_diff(d1, _digest()).is_empty(), "B2 ... and the two steps taken were put back: the world is the post-group digest (differs: %s)" % str(_diff(d1, _digest())))
	_check(app.undo_groups.wants_undo(), "B2 ... and the group still owns the next Undo")
	app.undo_last()
	await _frames(2)
	_check(_diff(d0, _digest()).is_empty(), "B2 the normal undo afterwards is the pre-group digest")
	## B3 redo-time.
	app.undo_groups.fail_at_step = 3
	r = app.undo_groups.redo()
	_check(not bool(r.get("ok", true)) and String(r.get("reason", "")).contains("test seam"), "B3 a refused fourth step fails the redo (%s)" % str(r.get("reason", "")))
	app.undo_groups.fail_at_step = -1
	await _frames(2)
	_check(_diff(d0, _digest()).is_empty(), "B3 ... and the three steps taken were undone again: the pre-group digest (differs: %s)" % str(_diff(d0, _digest())))
	_check(app.undo_groups.wants_redo(), "B3 ... and the group still owns the next Redo")
	app.redo_last()
	await _frames(2)
	_check(_diff(d1, _digest()).is_empty(), "B3 the normal redo afterwards is the post-group digest")
	app.undo_last()
	await _frames(2)
	_check(_gens == gens0, "B no generate was started (%d)" % (_gens - gens0))

func _section_c() -> void:
	_log("=== C: single-step undo where no group is involved ===")
	var g = app.undo_groups
	var t = _tool()
	var d0 := _digest()
	bridge.carve_fjords()
	_check(not g.wants_undo(), "C an ungrouped carve on top: no group owns the press")
	app.undo_last()
	await _frames(2)
	_check(String(_digest()["field"]) == String(d0["field"]), "C the carve undid alone")
	_look_edit(7, Color8(1, 2, 3))
	app.undo_last()
	await _frames(2)
	_check(String(_digest()["look"]) == String(d0["look"]), "C the look edit undid alone")
	_paint(t)
	t.undo()
	await _frames(2)
	_check(String(_digest()["colour_paint"]) == String(d0["colour_paint"]), "C the paint tool's own Undo still undoes an ungrouped stroke")
	## The empty-list guards, on a fresh object so the session's groups do not answer.
	var fresh = preload("res://shell/undo_group.gd").new()
	fresh.app = app
	_check(fresh.blocks_single_undo(true) == "" and fresh.blocks_single_undo(false) == "" and fresh.blocks_single_redo(false) == "" and fresh.blocks_revert_to(1) == "" and fresh.blocks_engine_redo(5) == "" and not fresh.wants_undo() and not fresh.wants_redo(),
		"C with no group every guard answers '' and no group wants a press")

func _section_d() -> void:
	_log("=== D: no member is taken alone ===")
	var g = app.undo_groups
	var t = _tool()
	var d0 := _digest()
	g.begin("Assistant: guarded")
	bridge.carve_fjords()
	_look_edit(4, Color8(90, 90, 250))
	_paint(t)
	_check(bool(g.commit().get("ok", false)), "D premise: a three-member group committed")
	var d1 := _digest()
	## The later ungrouped height edit is a droplet erosion, not a carve: a carve emits
	## `world_loaded`, whose handler (`_refresh_world_dependent`) clears the look history and
	## the paint stack -- an existing behaviour, and it would take the group's look and paint
	## members with it (the group is then dissolved; see `undo_group.gd::on_world_changed`).
	var er: Dictionary = bridge.world_gen.erode_op({})
	var d2 := _digest()
	_check(bool(er.get("ok", false)) and String(d2["field"]) != String(d1["field"]), "D premise: an ungrouped droplet erosion moved the field")
	_check(not g.wants_undo(), "D an ungrouped erosion on top: the group does not own the press")
	app.undo_last()
	await _frames(2)
	_check(_diff(d1, _digest()).is_empty(), "D ... the plain Undo took the ungrouped erosion alone (differs: %s)" % str(_diff(d1, _digest())))
	_check(g.wants_undo(), "D ... and the group owns the next press again")
	## A later ungrouped paint stroke: paint's top is no longer the group's, so the plain Undo
	## would route to the engine and take the group's carve alone -- refused.
	_paint(t)
	var d3 := _digest()
	_check(not g.wants_undo(), "D an ungrouped stroke on top of the paint stack: the group does not own the press")
	app.undo_last()
	await _frames(2)
	_check(_diff(d3, _digest()).is_empty(), "D the plain Undo that would take the group's carve alone changed nothing (differs: %s)" % str(_diff(d3, _digest())))
	var hint: String = (app._status_labels["hint"] as Label).text if app._status_labels.has("hint") else ""
	_check(hint.contains("part of \"Assistant: guarded\""), "D ... and says why (%s)" % hint)
	t.undo()
	await _frames(2)
	_check(g.wants_undo(), "D after the paint tool undid its own stroke, the group owns the press")
	## The paint tool's own Undo on a grouped step refuses.
	var before_doc: String = bridge.colour_paint_document()
	t.undo()
	await _frames(2)
	_check(bridge.colour_paint_document() == before_doc, "D the paint tool's own Undo refuses the group's stroke")
	## A History revert into the group.
	var seqs: Array = g.groups[g.cursor - 1]["engine"] if g.cursor > 0 else []
	_check(not seqs.is_empty() and g.blocks_revert_to(int(seqs[0])) != "", "D a History revert into the group is refused (%s)" % (g.blocks_revert_to(int(seqs[0])) if not seqs.is_empty() else "no seq"))
	app.undo_last()
	await _frames(2)
	_check(_diff(d0, _digest()).is_empty(), "D the group then undoes whole to the pre-group digest (differs: %s)" % str(_diff(d0, _digest())))
	## Redo side: an ungrouped commit after the group's undo drops its redo (linear history),
	## even once that commit is itself undone.
	_check(g.wants_redo(), "D premise: the undone group owns the next Redo")
	bridge.world_gen.erode_op({})
	app.undo_last()
	await _frames(2)
	_check(not g.wants_redo(), "D a commit (and its undo) after the group's undo drops the group's redo")

func _section_e() -> void:
	_log("=== E: caps ===")
	var g = app.undo_groups
	var d0 := _digest()
	var keys := _float_keys(CAP + 1)
	if keys.size() < CAP + 1:
		_check(false, "E premise: %d float settings to write (%d found)" % [CAP + 1, keys.size()])
		return
	g.begin("Assistant: too many")
	for k in keys:
		g.write_param(k[0], k[1])
	var c: Dictionary = g.commit()
	_check(not bool(c.get("ok", true)), "E %d members are refused (%s)" % [CAP + 1, str(c.get("reason", ""))])
	_check(_diff(d0, _digest()).is_empty(), "E ... and rolled back (differs: %s)" % str(_diff(d0, _digest())))
	g.begin("Assistant: at the cap")
	for k in keys.slice(0, CAP):
		g.write_param(k[0], k[1])
	c = g.commit()
	_check(bool(c.get("ok", false)) and int(c.get("members", 0)) == CAP, "E exactly %d members are accepted (%s)" % [CAP, str(c)])
	app.undo_last()
	await _frames(2)
	_check(_diff(d0, _digest()).is_empty(), "E ... and undo in one step")
	var st: Dictionary = bridge.undo_stats()
	var cap := maxi(1, mini(int(st["max_steps"]), int(st["budget_bytes"]) / maxi(int(st["step_bytes"]), 1)))
	g.begin("Assistant: room")
	var room0: int = g.height_room()
	bridge.carve_fjords()
	var room1: int = g.height_room()
	g.abort("probe")
	## At 256 x 192 a step is 196 608 bytes, far inside the 256 MiB default budget, so the
	## count bound (5, the reference's MAX_UNDO) binds: the literal, not the constant.
	_check(cap == 5 and room0 == 5 and room1 == 4, "E height_room is the engine's capacity minus the group's height steps (cap %d, %d -> %d)" % [cap, room0, room1])

func _section_f(seed_: int) -> void:
	_log("=== F: a parameter-only group after a regenerate ===")
	var g = app.undo_groups
	var keys := _float_keys(2)
	var old0 = bridge.param_get(keys[0][0])
	var old1 = bridge.param_get(keys[1][0])
	g.begin("Assistant: settings")
	g.write_param(keys[0][0], keys[0][1])
	g.write_param(keys[1][0], keys[1][1])
	_check(bool(g.commit().get("ok", false)), "F premise: a two-setting group committed")
	await _generate(seed_)
	var dg := _digest()
	_check(g.wants_undo(), "F the group survives the Generate and owns the press")
	_check(app.undo_target_label().ends_with("(settings only)"), "F its label says settings only (%s)" % app.undo_target_label())
	var gens0 := _gens
	app.undo_last()
	await _frames(2)
	_check(bridge.param_get(keys[0][0]) == old0 and bridge.param_get(keys[1][0]) == old1, "F undo restored both settings")
	var du := _digest()
	_check(String(du["field"]) == String(dg["field"]) and String(du["ways_manual"]) == String(dg["ways_manual"]), "F ... and left the regenerated world as it was")
	_check(_gens == gens0, "F ... and started no generate")
	await _shot("f_settings_only")

# -- main --------------------------------------------------------------------------

func _ready() -> void:
	_tag = _arg("--tag", "undogroup")
	_out = _arg("--out", "")
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--force-touch", "--vp", "--tag", "--out"]):
			_abort("unknown flag %s" % s)
			return
	if DisplayServer.get_name() == "headless":
		_abort("run WINDOWED")
		return
	if _out != "":
		DirAccess.make_dir_recursive_absolute(_out)
	get_tree().create_timer(900.0).timeout.connect(func() -> void: _abort("watchdog"))
	var parts: PackedStringArray = _arg("--vp", "1600x900").split("x")
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	bridge = app.bridge
	_log("viewport %dx%d phone=%s tablet=%s touch=%s" % [_vp.size.x, _vp.size.y, str(DccTheme.is_phone()), str(DccTheme.is_tablet()), str(DccTheme.is_touch())])
	for m in ["undo_commit_seq", "redo_offer_ids", "edit_state_digest"]:
		if not bridge.world_gen.has_method(m):
			_abort("the loaded extension has no WorldGen.%s (stale DLL?)" % m)
			return
	if not bridge.colour_paint_api or not bridge.biome_colors_api:
		_abort("the loaded extension lacks colour paint or biome colours")
		return
	bridge.generation_started.connect(func(): _gens += 1)
	for i in SEEDS.size():
		await _generate(SEEDS[i])
		if _tool() == null:
			_abort("no colour-paint tool")
			return
		await _section_a(SEEDS[i], i == 0)
		if i == 0:
			await _section_b()
			await _section_c()
			await _section_d()
			await _section_e()
			await _section_f(SEEDS[i])
	_log("RESULT %s checks=%d fail=%d" % [_tag, _checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
