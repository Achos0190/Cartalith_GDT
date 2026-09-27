extends Node
## Regression guard for two undo quirks closed in the same batch
## (`OUTSTANDING_WORK.md` §2.11, "Two undo quirks outside the way work"):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _undofixes_probe.tscn -- --vp 1080x2340 --force-touch
##
## A  `WorldGen::undo_revert_to` did not mark the height stage stale after
##    reverting -- `undo_last` did (its own doc comment: "Undo was the only
##    height mutator in the crate that wrote `ws.field` and left the
##    staleness graph saying nothing had changed"), but the multi-step
##    revert-to path never got the same line. Two height edits are committed,
##    the graph is settled clean, then a revert-to the floor pops both back
##    off; `stale_stages()` must now report a downstream stage stale with
##    `reason == "undo_revert_to"`, where before the fix it reported nothing
##    at all -- the same silent-desync `undo_last`'s own doc comment names.
##
## B  The phone Undo chip (`dcc_shell.gd::_do_phone_undo()`) called
##    `bridge.undo_last()` directly, bypassing `DccApp.undo_last()`'s own
##    repaint (`viewport.map_view.texture = bridge.color_texture()`,
##    `invalidate_lod_tiles()`). A height undo therefore left the OLD texture
##    on screen. Driven here by calling the chip's own handler directly
##    (`app._do_phone_undo()`), the same way `_redodock_probe.gd` drives
##    `carve_fjords()` straight through its bridge rather than through a
##    button press, and comparing the viewport's texture bytes before and
##    after against `bridge.color_texture()`'s own bytes taken right after --
##    proof the repaint actually happened, not merely that *a* texture object
##    changed.
##
## Windowed, not `--headless`: part B needs the real phone shell tree
## (`--force-touch`) with `app.viewport` built, the same requirement
## `_ctxphone_probe.gd` states for the same reason. Nothing here reads a
## rendered frame's pixels -- both texture comparisons are `Image.get_data()`
## byte compares on freshly built `ImageTexture`s (`build_color_texture()`
## constructs one from scratch every call rather than mutating one in place),
## so this is not the `ImageTexture.update()`-under-headless trap; windowed
## anyway, because the brief that scoped this probe asks for it and a real
## phone shell is cheap to boot.

const SEED := 552017

var app: Node
var _vp: SubViewport
var _fails := 0
var _checks := 0


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("%s  %s%s" % ["PASS" if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _tex_bytes(t: Texture2D) -> PackedByteArray:
	if t == null:
		return PackedByteArray()
	var img: Image = t.get_image()
	if img == null:
		return PackedByteArray()
	return img.get_data()


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 150.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("WATCHDOG"); get_tree().quit(3))
	wd.start()

	var vp_size := Vector2i(1080, 2340)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a == "--vp" and i + 1 < args.size():
			var wh := args[i + 1].split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
			continue
		if a == "--force-touch":
			i += 1
			continue
		print("### UNDOFIX ABORT: unknown argument '%s' ###" % a)
		get_tree().quit(2)
		return

	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)

	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.0).timeout

	var bridge = app.bridge
	bridge.generate({
		"seed": SEED, "width_km": 1200.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)

	if not DccTheme.is_phone():
		print("### UNDOFIX ABORT: needs the phone form for leg B -- use --force-touch and a phone-sized --vp ###")
		get_tree().quit(2)
		return

	# ==================================================================== A
	print("=== A: undo_revert_to marks the height stage stale, like undo_last ===")

	var r1: Dictionary = bridge.carve_fjords()
	await _frames(3)
	# `r1`'s own row, captured before `r2` exists -- reverting *to* it pops
	# its own snapshot too (`HistoryLedger::steps_to_revert_to_with`'s own
	# doc comment: "the count is every live row AT OR ABOVE seq"), landing
	# back at the pre-`r1` floor state. The floor row itself is never a valid
	# target (`EntryKind::Floor` is never `live`, so `undo_revert_to` would
	# answer `0`).
	var target_seq: int = int((bridge.undo_ledger()[-1] as Dictionary).get("seq", 0))
	var r2: Dictionary = bridge.carve_fjords()
	await _frames(3)
	_ok("A setup: both carves ran", bool(r1.get("ok", false)) and bool(r2.get("ok", false)),
		"r1=%s r2=%s" % [str(r1), str(r2)])
	_ok("A setup: the target row is real", target_seq > 0, "target_seq=%d" % target_seq)

	# Settle the graph before the revert, so anything reported stale
	# afterwards is caused by the revert and not left over from the carves.
	bridge.world_gen.recompute_stale_stages()
	await _frames(2)
	var before: Dictionary = bridge.stale_stages()
	# `civ` legitimately stays stale here -- `recompute_stale_stages()`'s own
	# doc comment: "'civ' in still_stale is the normal steady state after a
	# terrain edit ... UNIFIED_TOOL_PLAN.md milestone C measured why it is
	# not cascaded per stroke". The baseline this test needs clean is
	# hydrology/climate, the two stages a settle actually recomputes.
	_ok("A setup: hydrology and climate are clean before the revert",
		not before.has("hydrology") and not before.has("climate"),
		"stale_stages()=%s" % str(before))

	var done: int = bridge.undo_revert_to(target_seq)
	_ok("A: the revert popped both carves", done == 2, "done=%d" % done)

	var after: Dictionary = bridge.stale_stages()
	print("A stale_stages() after undo_revert_to: %s" % str(after))
	_ok("A: something is stale after the revert (the bug: this used to be empty)",
		not after.is_empty(), "after=%s" % str(after))
	var hydro: Dictionary = after.get("hydrology", {})
	_ok("A: hydrology (Height's own consumer) is the stale stage",
		not hydro.is_empty(), "after=%s" % str(after))
	_ok("A: its reason names the revert, not the carves it undid",
		String(hydro.get("reason", "")) == "undo_revert_to",
		"reason=%s" % String(hydro.get("reason", "")))

	# Leaves the pipeline in a known state for leg B: settle it again so leg
	# B's own staleness is caused only by what it does.
	bridge.world_gen.recompute_stale_stages()
	await _frames(2)

	# ==================================================================== B
	print("=== B: the phone Undo chip repaints the height texture ===")

	var r3: Dictionary = bridge.carve_fjords()
	await _frames(3)
	_ok("B setup: a fresh height edit committed", bool(r3.get("ok", false)), str(r3))
	_ok("B setup: can_undo() is true", bridge.can_undo())

	var before_tex := _tex_bytes(app.viewport.map_view.texture)
	app._do_phone_undo()
	await _frames(6)
	var after_tex := _tex_bytes(app.viewport.map_view.texture)
	var oracle_tex := _tex_bytes(bridge.color_texture())

	_ok("B POSITIVE CONTROL: the drawn texture actually changed", after_tex != before_tex,
		"before %d bytes, after %d bytes" % [before_tex.size(), after_tex.size()])
	_ok("B: the drawn texture is not merely different but the CORRECT one",
		after_tex == oracle_tex and not oracle_tex.is_empty(),
		"after %d bytes, oracle %d bytes" % [after_tex.size(), oracle_tex.size()])

	print("### UNDOFIX %s  %d/%d checks passed ###" % [
		"GREEN" if _fails == 0 else "RED", _checks - _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
