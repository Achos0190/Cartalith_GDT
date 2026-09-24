extends SceneTree

## CA-19 persistence (`OUTSTANDING_WORK.md` §2.11, "Biome colour edits are
## not saved with the project"): the biome colour table round-trips through
## `appearance.json`'s optional `biome_cols` member.
##
##   Godot_v4.7.1-stable_win64_console --headless --path . \
##       --script res://_biomesave_probe.gd -- --legacy <path-to-HEAD-saved.zip>
##
##   Godot_v4.7.1-stable_win64_console --headless --path . \
##       --script res://_biomesave_probe.gd -- --capture <out.zip>
##
## Headless and `extends SceneTree`, `_lmpersist_probe.gd`'s shape: nothing
## rasterises, and the thing under test is `project_save` / `project_open`.
##
## `--capture <out.zip>` only writes a project and exits. It was run once
## against the pre-change DLL (`bab9edd`, `project_bridge.rs` unchanged from
## HEAD) to produce the backward-compatibility fixture: a world whose look,
## an override and **a biome colour** were all edited, saved by a writer that
## had no `biome_cols` member. `project_bridge.rs`'s
## `APPEARANCE_BEFORE_BIOME_COLS` is that archive's `appearance.json`, verbatim.
##
## `--legacy <zip>` is required for the full run: the archive `--capture`
## wrote. Any other argument fails loudly rather than defaulting (MISTAKES.md,
## "Write a probe's usage header").
##
## What is checked, each against a literal:
##  1. an edited class survives save -> open in a fresh `WorldGen`;
##  2. an untouched table writes **no** `biome_cols` member;
##  3. the legacy archive opens with the default table even when the session
##     that opens it had an edited one (absent = default, not "keep");
##  4. a `biome_cols` of the wrong length opens with the default table and a
##     warning, and the rest of the appearance still comes back.

const EDIT_INDEX := 3
const EDIT_RGB := [12, 200, 77]

func _args() -> Dictionary:
	var a := OS.get_cmdline_user_args()
	var out := {}
	var i := 0
	while i < a.size():
		var k: String = a[i]
		if k in ["--legacy", "--capture"] and i + 1 < a.size():
			out[k] = a[i + 1]
			i += 2
		else:
			print("FAIL: unknown or incomplete argument %s" % k)
			return {"bad": true}
	return out

func _world() -> WorldGen:
	var wg: WorldGen = WorldGen.new()
	wg.generate_sized(24601, 640.0, 96, 72)
	return wg

func _rgb(c: Color) -> Array:
	return [c.r8, c.g8, c.b8]

func _appearance_json(path: String) -> String:
	var zr := ZIPReader.new()
	if zr.open(path) != OK:
		return ""
	var s := ""
	if "appearance.json" in zr.get_files():
		s = zr.read_file("appearance.json").get_string_from_utf8()
	zr.close()
	return s

func _init() -> void:
	var args := _args()
	if args.has("bad"):
		quit(2)
		return
	if args.has("--capture"):
		var wg := _world()
		wg.set_look("vibrant")
		wg.set_appearance({"sun_az_deg": 300.0, "bio_blend": 0.625})
		wg.set_biome_color(EDIT_INDEX, EDIT_RGB[0], EDIT_RGB[1], EDIT_RGB[2])
		var r: Dictionary = wg.project_save(String(args["--capture"]))
		print("capture: ", r.get("ok"), " ", r.get("error", ""))
		print(_appearance_json(String(args["--capture"])))
		quit(0 if bool(r.get("ok", false)) else 1)
		return
	if not args.has("--legacy"):
		print("FAIL: --legacy <zip> is required (see the header)")
		quit(2)
		return

	var fails := 0
	var dir := OS.get_user_data_dir()
	var default3: Array = _rgb(WorldGen.new().get_biome_color(EDIT_INDEX))
	print("  default class %d = %s" % [EDIT_INDEX, default3])
	if default3 == EDIT_RGB:
		print("  FAIL: the edit colour equals the default; the round trip would prove nothing")
		quit(1)
		return

	## 1. An edited class survives save -> open.
	var wg := _world()
	wg.set_biome_color(EDIT_INDEX, EDIT_RGB[0], EDIT_RGB[1], EDIT_RGB[2])
	var p1 := dir.path_join("_biomesave_edit.zip")
	if not bool(wg.project_save(p1).get("ok", false)):
		print("  FAIL: save refused")
		quit(1)
		return
	var doc1 := _appearance_json(p1)
	print("  edited appearance.json has biome_cols: ", "\"biome_cols\"" in doc1)
	if not ("\"biome_cols\"" in doc1):
		print("  FAIL: an edited table wrote no biome_cols member")
		fails += 1
	var wg2: WorldGen = WorldGen.new()
	var o2: Dictionary = wg2.project_open(p1)
	var got: Array = _rgb(wg2.get_biome_color(EDIT_INDEX))
	print("  reopened class %d = %s (want %s)" % [EDIT_INDEX, got, EDIT_RGB])
	if got != EDIT_RGB:
		print("  FAIL: the edited colour did not survive save -> open")
		fails += 1
	var others_moved := 0
	for k in range(1, 16):
		if k != EDIT_INDEX and _rgb(wg2.get_biome_color(k)) != _rgb(wg.get_biome_color(k)):
			others_moved += 1
	if others_moved > 0:
		print("  FAIL: %d untouched classes changed across the round trip" % others_moved)
		fails += 1
	if not (o2.get("warnings", []) as Array).is_empty():
		print("  FAIL: a clean round trip raised warnings: ", o2.get("warnings"))
		fails += 1

	## 2. An untouched table writes no member.
	var wg3 := _world()
	var p3 := dir.path_join("_biomesave_default.zip")
	wg3.project_save(p3)
	var doc3 := _appearance_json(p3)
	if doc3.is_empty():
		print("  FAIL: appearance.json absent from a default save")
		fails += 1
	elif "biome_cols" in doc3:
		print("  FAIL: a default table wrote biome_cols")
		fails += 1

	## 3. The legacy archive, opened by a session that has an edited table.
	var legacy := String(args["--legacy"])
	var legacy_doc := _appearance_json(legacy)
	if legacy_doc.is_empty() or "biome_cols" in legacy_doc:
		print("  FAIL: %s is not a pre-biome_cols archive" % legacy)
		quit(1)
		return
	var wg4 := _world()
	wg4.set_biome_color(EDIT_INDEX, EDIT_RGB[0], EDIT_RGB[1], EDIT_RGB[2])
	var o4: Dictionary = wg4.project_open(legacy)
	if not bool(o4.get("ok", false)):
		print("  FAIL: the legacy archive did not open -> ", o4.get("error", "?"))
		fails += 1
	var leg: Array = _rgb(wg4.get_biome_color(EDIT_INDEX))
	print("  legacy open: class %d = %s, restored=%s, warnings=%s" % [EDIT_INDEX, leg, o4.get("restored"), o4.get("warnings")])
	if leg != default3:
		print("  FAIL: the legacy archive kept the session's edit (want the default %s)" % [default3])
		fails += 1
	if not ("appearance" in (o4.get("restored", []) as Array)):
		print("  FAIL: the legacy appearance was not restored")
		fails += 1
	## Its look came back: the capture set "vibrant", which the engine stores as "Natural Vibrant".
	if wg4.has_method("get_look") and String(wg4.get_look()).to_lower() != "natural vibrant":
		print("  FAIL: the legacy look came back as %s" % wg4.get_look())
		fails += 1

	## 4. A malformed member costs itself only.
	var bad := dir.path_join("_biomesave_bad.zip")
	var zr := ZIPReader.new()
	zr.open(p1)
	var zp := ZIPPacker.new()
	zp.open(bad)
	for f in zr.get_files():
		var bytes := zr.read_file(f)
		if f == "appearance.json":
			var d: Dictionary = JSON.parse_string(bytes.get_string_from_utf8())
			(d["biome_cols"] as Array).pop_back()
			bytes = JSON.stringify(d).to_utf8_buffer()
		zp.start_file(f)
		zp.write_file(bytes)
		zp.close_file()
	zp.close()
	zr.close()
	var wg5: WorldGen = WorldGen.new()
	var o5: Dictionary = wg5.project_open(bad)
	var w5: Array = o5.get("warnings", [])
	var b5: Array = _rgb(wg5.get_biome_color(EDIT_INDEX))
	print("  short table: ok=%s class %d = %s warnings=%s" % [o5.get("ok"), EDIT_INDEX, b5, w5])
	if not bool(o5.get("ok", false)):
		print("  FAIL: a short biome_cols failed the whole open")
		fails += 1
	if b5 != default3:
		print("  FAIL: a short table was applied (partially or wholly)")
		fails += 1
	var warned := false
	for w in w5:
		if "biome_cols" in String(w):
			warned = true
	if not warned:
		print("  FAIL: a short table was dropped without a warning")
		fails += 1
	if not ("appearance" in (o5.get("restored", []) as Array)):
		print("  FAIL: a bad biome_cols cost the rest of the appearance")
		fails += 1

	## 5. How "Reset appearance" and a saved look meet the table -- the
	## CARTO ▸ Biome colours note states these, so they are measured, not
	## reasoned. (a) Reset appearance leaves an edit alone.
	var wg6 := _world()
	wg6.set_biome_color(EDIT_INDEX, EDIT_RGB[0], EDIT_RGB[1], EDIT_RGB[2])
	wg6.reset_appearance()
	var r6: Array = _rgb(wg6.get_biome_color(EDIT_INDEX))
	print("  after Reset appearance: class %d = %s" % [EDIT_INDEX, r6])
	if r6 != EDIT_RGB:
		print("  FAIL: Reset appearance dropped a biome edit")
		fails += 1
	## (b) A saved look carries the table it was saved with; loading it makes
	## that table the base the per-class resets return to.
	var pp := ProjectSettings.globalize_path(dir.path_join("_biomesave_look.json"))
	if not wg6.save_appearance_preset(pp, "biome probe"):
		print("  FAIL: save_appearance_preset refused")
		fails += 1
	wg6.reset_biome_colors()
	var cleared: Array = _rgb(wg6.get_biome_color(EDIT_INDEX))
	wg6.load_appearance_preset(pp)
	var via_look: Array = _rgb(wg6.get_biome_color(EDIT_INDEX))
	var n_reset: int = wg6.reset_biome_colors()
	var after_reset_all: Array = _rgb(wg6.get_biome_color(EDIT_INDEX))
	wg6.reset_appearance()
	var after_reset_app: Array = _rgb(wg6.get_biome_color(EDIT_INDEX))
	print("  look: cleared=%s loaded=%s reset-all(n=%d)=%s reset-appearance=%s"
		% [cleared, via_look, n_reset, after_reset_all, after_reset_app])
	if cleared != default3:
		print("  FAIL: reset_biome_colors did not return to the default before the look loaded")
		fails += 1
	if via_look != EDIT_RGB:
		print("  FAIL: a saved look did not bring its biome table back")
		fails += 1
	if after_reset_all != EDIT_RGB:
		print("  FAIL: Reset all under a loaded look went somewhere other than the look's table")
		fails += 1
	if after_reset_app != default3:
		print("  FAIL: Reset appearance (which drops the look) did not return to the reference table")
		fails += 1
	## (c) A project saved under that look writes the reference table plus the
	## session's own edits -- the look itself is not project state.
	var wg7 := _world()
	wg7.load_appearance_preset(pp)
	var p7 := dir.path_join("_biomesave_look.zip")
	wg7.project_save(p7)
	if "biome_cols" in _appearance_json(p7):
		print("  FAIL: a loaded look's table was written into the project as if it were an edit")
		fails += 1

	if fails == 0:
		print("PASS: biome colours persist; default writes nothing; legacy and malformed open at the default table")
	else:
		print("FAIL: %d checks failed" % fails)
	quit(1 if fails > 0 else 0)
