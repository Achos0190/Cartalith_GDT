extends Node
## MENUS lane, Part C, 2026-09-13. Dumps `File ▸ Recent worlds`'s real leaf
## rows -- label and tooltip -- against whatever `DccSettings.recent_projects()`
## actually holds on this machine, so the report quotes real output rather than
## reasoning about the code in the abstract.
##
##   Godot_v4.7.1 --path . _recentworlds_probe.tscn
##
## Read-only: does not seed, does not clear, does not write
## `user://cartalith_settings.cfg`. Whatever is in `[recent]` today is what
## this prints.

## **OPEN lane, Part C, 2026-09-13 -- a DIFFERENT "Part C" than the header
## above.** That one (same date, an earlier dispatch) is about the world NAME
## a Recent-worlds row shows; `menus.gd:824`'s own comment carries its Gate C1.
## This one is about the SEED: `open_project_dialog.gd::project_meta()` read
## only the legacy `params.json` `state.tect.seed`, and `project_save()` --
## the function the real Save command calls (`engine_bridge.gd`) -- writes a
## params.json shaped `{"cartalith": …, "reference": …}` with no `state` key
## at all, so every native save showed "seed unread" everywhere `project_meta()`
## is read: this menu (`menus.gd:838`), the welcome tiles and the phone picker
## (`phone_project_picker.gd:304`, same static call). Fixed by reading
## `project.json`'s own `world.seed` first, falling back to the legacy key.
##
## Three fixtures, built and torn down here, that never touch
## `DccSettings`/`recent_projects()`/`user://cartalith_settings.cfg`:
## `project_meta()` is a pure `static func` over a path, so this needs none of
## `_openfailgate_probe.gd`'s settings backup/restore machinery.
##   - REAL native: a genuine `WorldGen.project_save()` archive with a known
##     seed -- the actual defect, on the actual writer.
##   - Legacy-shaped: a hand-built zip carrying only the old `params.json`
##     shape, no `project.json` at all -- Gate C1, the fallback path
##     unchanged for a save that predates `world.seed`.
##   - Mixed: BOTH files present, but `project.json`'s `format` does not say
##     `cartalith-project` -- hardens the `is_native` gate: an unrelated zip
##     that happens to carry some `project.json` must not hand back a
##     confident number, and the legacy fallback must still fire.
const PARTC_NATIVE_SEED := 55111
const PARTC_LEGACY_SEED := 66222
const PARTC_MIXED_SEED := 77333

func _write_minimal_zip(path: String, entries: Dictionary) -> void:
	var zp := ZIPPacker.new()
	zp.open(path)
	for name in entries:
		zp.start_file(String(name))
		zp.write_file(String(entries[name]).to_utf8_buffer())
		zp.close_file()
	zp.close()

func _run_seed_gate() -> void:
	print("")
	print("=== OPEN lane Part C (2026-09-13): project_meta() seed source ===")

	## Globalized: `project_save()` is a Rust `#[func]` that opens the path
	## through `std::fs`, which does not understand Godot's `user://` virtual
	## scheme -- `ZIPPacker`/`ZIPReader` below are Godot classes and accept
	## either form, but the real writer needs a real OS path.
	var native_path := ProjectSettings.globalize_path("user://_partc_native_seed.ctl")
	var wg: WorldGen = WorldGen.new()
	wg.generate_sized(PARTC_NATIVE_SEED, 640.0, 48, 32)
	var w: Dictionary = wg.project_save(native_path)
	_ok("fixture: real native save wrote ok", bool(w.get("ok", false)), str(w))
	var native_meta := OpenProjectDialog.project_meta(native_path)
	print("  native  meta.seed=", native_meta.get("seed"))
	_ok("REGRESSION: a real native save's seed is no longer \"seed unread\"",
		String(native_meta.get("seed", "")) != "seed unread")
	_ok("native save's seed reads from project.json's world.seed",
		String(native_meta.get("seed", "")) == str(PARTC_NATIVE_SEED))

	var legacy_path := "user://_partc_legacy_seed.ctl"
	_write_minimal_zip(legacy_path, {
		"params.json": JSON.stringify({"state": {"tect": {"seed": PARTC_LEGACY_SEED}}}),
	})
	var legacy_meta := OpenProjectDialog.project_meta(legacy_path)
	print("  legacy  meta.seed=", legacy_meta.get("seed"))
	_ok("Gate C1: a legacy save (no project.json at all) still reads its seed",
		String(legacy_meta.get("seed", "")) == str(PARTC_LEGACY_SEED))

	var mixed_path := "user://_partc_mixed_seed.ctl"
	_write_minimal_zip(mixed_path, {
		"project.json": JSON.stringify({"format": "not-cartalith-project", "world": {"seed": 999999}}),
		"params.json": JSON.stringify({"state": {"tect": {"seed": PARTC_MIXED_SEED}}}),
	})
	var mixed_meta := OpenProjectDialog.project_meta(mixed_path)
	print("  mixed   meta.seed=", mixed_meta.get("seed"))
	_ok("a project.json whose format does not match falls back to the legacy seed",
		String(mixed_meta.get("seed", "")) == str(PARTC_MIXED_SEED))

	for p in [native_path, legacy_path, mixed_path]:
		if FileAccess.file_exists(p):
			DirAccess.remove_absolute(p)
	print("  [cleanup] fixtures removed:",
		[native_path, legacy_path, mixed_path].map(func(p): return not FileAccess.file_exists(p)))

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _menu_button(root: Node, title: String) -> MenuButton:
	if root is MenuButton and (root as MenuButton).text == title:
		return root as MenuButton
	for c in root.get_children(true):
		var r := _menu_button(c, title)
		if r != null:
			return r
	return null

func _submenu_by_title(pm: PopupMenu, title: String) -> PopupMenu:
	for i in pm.item_count:
		if pm.is_item_separator(i):
			continue
		if pm.get_item_text(i) == title:
			var sub := pm.get_item_submenu(i)
			if sub != "":
				var node := pm.get_node_or_null(NodePath(sub))
				if node is PopupMenu:
					return node as PopupMenu
	return null

var _fail := 0

func _ok(name: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fail += 1
	print("  ", "ok  " if cond else "FAIL", " ", name, (("   " + detail) if detail != "" else ""))

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return

	## Independent of the app/menu walk below -- `project_meta()` is a pure
	## static function and needs no shell, no viewport, no boot.
	_run_seed_gate()

	var vp := SubViewport.new()
	vp.size = Vector2i(1600, 1000)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(45)

	print("=== recent_projects() on disk, in order ===")
	var recents: Array = DccSettings.recent_projects()
	for i in recents.size():
		var p := String(recents[i])
		print("  [", i, "] exists=", FileAccess.file_exists(p), "  ", p)

	var file_mb := _menu_button(app, "File")
	if file_mb == null:
		print("[FATAL] no File menu button found"); get_tree().quit(1); return
	var file_popup := file_mb.get_popup()
	file_popup.about_to_popup.emit()
	var recent_popup := _submenu_by_title(file_popup, "Recent worlds")
	if recent_popup == null:
		print("[FATAL] no Recent worlds submenu found"); get_tree().quit(1); return
	print("")
	print("=== File > Recent worlds, real rows (label | tooltip) ===")
	print("  item_count=", recent_popup.item_count)
	var any_double_space_dash := false
	var any_edited_prefix := false
	var any_double_space_dot := false
	var missing_rows_ok := true
	var any_tooltip_empty := false
	for i in recent_popup.item_count:
		if recent_popup.is_item_separator(i):
			print("  ---")
			continue
		var label := recent_popup.get_item_text(i)
		var tip := recent_popup.get_item_tooltip(i)
		print("  LABEL|", label)
		print("    tip|", tip)
		if label.contains("  —  "):
			any_double_space_dash = true
		if label.contains("  ·  "):
			any_double_space_dot = true
		if tip.contains("edited "):
			## The tooltip is always the bare path for a present file and the
			## true-reason sentence for a missing one -- neither should ever
			## carry the source function's raw "edited " prose; only the
			## LABEL ever did, and only before Part C's fix.
			pass
		if label.contains("edited "):
			any_edited_prefix = true
		if tip.strip_edges() == "":
			any_tooltip_empty = true
		if not FileAccess.file_exists(String(recents[i]) if i < recents.size() else ""):
			if not label.ends_with("— file not found"):
				missing_rows_ok = false
	print("")
	print("=== MENUS lane Part C gates ===")
	_ok("no row uses the old double-space em-dash ('  —  ')", not any_double_space_dash)
	_ok("no row uses the old double-space middot ('  ·  ')", not any_double_space_dot)
	_ok("no row carries the raw 'edited ' prefix (terse-age ran)", not any_edited_prefix)
	_ok("every missing-file row ends '— file not found'", missing_rows_ok)
	_ok("no row's tooltip is empty", not any_tooltip_empty)
	print("")
	print("_recentworlds_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	print("=== end ===")
	get_tree().quit(1 if _fail > 0 else 0)
