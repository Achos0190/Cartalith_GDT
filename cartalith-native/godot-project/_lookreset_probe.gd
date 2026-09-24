extends SceneTree

## `OUTSTANDING_WORK.md` §2.11, "Opening a flat or legacy archive keeps the
## previous session's look edits": `WorldGen::load_save` now resets the
## project-owned half of the look -- the scalar overrides, the ramp, the NPR
## block and CA-19's biome colour table -- and `project_open`'s restore from
## `appearance.json` still wins over that reset.
##
##   Godot_v4.7.1-stable_win64_console --headless --path . \
##       --script res://_lookreset_probe.gd -- --head <HEAD-saved project.zip>
##
## `--head <zip>` is required: a project written by the writer before
## `appearance.json` had a `biome_cols` member (the archive
## `_biomesave_probe.gd --capture` wrote). Any other argument fails loudly.
## The flat archive is the repository's real HTML-app export,
## `crates/cartalith-io/tests/fixtures/real_export_seed24601.zip`.
##
## Headless and `extends SceneTree`: nothing rasterises. Every "default" below
## is read from a fresh `WorldGen` that ran the same `generate_sized`, never
## typed in, so the comparison is session-default against session-default.
## The edit values are literals and are checked to differ from the defaults
## first, so a pass cannot be vacuous.

const EDIT_INDEX := 3
const EDIT_RGB := [12, 200, 77]
const EDIT_SUN := 300.0

var _checks := 0
var _fails := 0

func _check(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("LOOKRESET %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _args() -> Dictionary:
	var a := OS.get_cmdline_user_args()
	var out := {}
	var i := 0
	while i < a.size():
		var k: String = a[i]
		if k == "--head" and i + 1 < a.size():
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

func _table(wg: WorldGen) -> Array:
	var t := []
	for k in range(1, 16):
		t.append(_rgb(wg.get_biome_color(k)))
	return t

## Everything the reset covers, as one comparable value.
func _look(wg: WorldGen) -> Dictionary:
	return {
		"appearance": var_to_str(wg.get_appearance()),
		"ramp": var_to_str(wg.get_color_ramp()),
		"npr": var_to_str(wg.get_npr()),
		"biome": var_to_str(_table(wg)),
	}

func _edit(wg: WorldGen) -> void:
	wg.set_appearance({"sun_az_deg": EDIT_SUN})
	wg.set_color_ramp([[0.0, Color(1, 0, 0)], [1.0, Color(0, 0, 1)]])
	wg.set_npr({"sepia": 0.5, "multi_sun": false})
	wg.set_biome_color(EDIT_INDEX, EDIT_RGB[0], EDIT_RGB[1], EDIT_RGB[2])

func _same_as(label: String, got: Dictionary, want: Dictionary) -> void:
	for k in ["appearance", "ramp", "npr", "biome"]:
		_check("%s: %s is the session default" % [label, k], got[k] == want[k],
			"" if got[k] == want[k] else "got %s" % String(got[k]).left(160))

func _init() -> void:
	var args := _args()
	if args.has("bad"):
		quit(2)
		return
	if not args.has("--head"):
		print("FAIL: --head <zip> is required (see the header)")
		quit(2)
		return
	var flat := ProjectSettings.globalize_path("res://").path_join("../crates/cartalith-io/tests/fixtures/real_export_seed24601.zip").simplify_path()
	if not FileAccess.file_exists(flat):
		print("FAIL: flat fixture missing at %s" % flat)
		quit(2)
		return

	var defaults := _look(_world())
	## Positive control: every edit moves its member, so "equals the default"
	## below cannot pass by the edit having done nothing.
	var edited_wg := _world()
	_edit(edited_wg)
	var edited := _look(edited_wg)
	for k in ["appearance", "ramp", "npr", "biome"]:
		_check("control: the edit moves %s" % k, edited[k] != defaults[k])

	## 1. A flat HTML-app archive, through `project_open` (the shell's path).
	var wg1 := _world()
	_edit(wg1)
	var o1: Dictionary = wg1.project_open(flat)
	_check("flat opens via project_open", bool(o1.get("ok", false)), String(o1.get("error", "")))
	_check("it is read as flat", String(o1.get("layout", "")) == "flat", String(o1.get("layout", "")))
	_check("no appearance restored from it", not ("appearance" in (o1.get("restored", []) as Array)), str(o1.get("restored")))
	_same_as("flat via project_open", _look(wg1), defaults)
	_check("flat via project_open: class %d is the reference colour" % EDIT_INDEX,
		_rgb(wg1.get_biome_color(EDIT_INDEX)) != EDIT_RGB, str(_rgb(wg1.get_biome_color(EDIT_INDEX))))

	## 2. The same archive through the flat reader directly (the fallback).
	var wg2 := _world()
	_edit(wg2)
	_check("flat opens via load_save", wg2.load_save(flat))
	_same_as("flat via load_save", _look(wg2), defaults)

	## 3. A project written by this build with an edited look: the restore must
	## win over `load_save`'s reset, in a session carrying different edits.
	var p3 := OS.get_user_data_dir().path_join("_lookreset_edit.zip")
	var r3: Dictionary = edited_wg.project_save(p3)
	_check("edited project saves", bool(r3.get("ok", false)), String(r3.get("error", "")))
	var wg3 := _world()
	wg3.set_biome_color(EDIT_INDEX, 1, 2, 3)
	wg3.set_appearance({"sun_az_deg": 45.0})
	var o3: Dictionary = wg3.project_open(p3)
	_check("edited project opens", bool(o3.get("ok", false)), String(o3.get("error", "")))
	_check("appearance restored", "appearance" in (o3.get("restored", []) as Array), str(o3.get("restored")))
	var got3 := _look(wg3)
	for k in ["appearance", "ramp", "npr", "biome"]:
		_check("edited project: %s is the archive's" % k, got3[k] == edited[k],
			"" if got3[k] == edited[k] else "got %s" % String(got3[k]).left(160))
	_check("edited project: class %d = %s" % [EDIT_INDEX, EDIT_RGB], _rgb(wg3.get_biome_color(EDIT_INDEX)) == EDIT_RGB,
		str(_rgb(wg3.get_biome_color(EDIT_INDEX))))
	_check("edited project: sun_az_deg = %s" % EDIT_SUN, float(wg3.get_appearance().get("sun_az_deg", -1.0)) == EDIT_SUN,
		str(wg3.get_appearance().get("sun_az_deg")))

	## 4. A HEAD-written project (appearance.json with no `biome_cols`), opened
	## in an edited session: its own overrides come back, the table is the
	## reference one.
	var head := String(args["--head"])
	var wg4 := _world()
	_edit(wg4)
	var o4: Dictionary = wg4.project_open(head)
	_check("HEAD project opens", bool(o4.get("ok", false)), String(o4.get("error", "")))
	_check("HEAD project restores appearance", "appearance" in (o4.get("restored", []) as Array), str(o4.get("restored")))
	var got4 := _look(wg4)
	_check("HEAD project: biome table is the reference one", got4["biome"] == defaults["biome"],
		"" if got4["biome"] == defaults["biome"] else String(got4["biome"]).left(160))
	_check("HEAD project: its own sun_az_deg %s" % EDIT_SUN, float(wg4.get_appearance().get("sun_az_deg", -1.0)) == EDIT_SUN,
		str(wg4.get_appearance().get("sun_az_deg")))

	print("RESULT %s  checks=%d fails=%d" % ["PASS" if _fails == 0 else "FAIL", _checks, _fails])
	quit(1 if _fails > 0 else 0)
