extends SceneTree

## 'The drop-count warning is written but wired to nothing that tests it'
## (`OUTSTANDING_WORK.md` §2.8, `SAVEFILE_COMPAT.md` §6.4a). Exercises the real
## `#[func] project_open()` call site in
## `cartalith-godot/src/project_bridge.rs` -- the drop-count subtraction for
## BOTH slots it exists for (`entities/landmarks.json` ~line 2165,
## `annotations/icons.json` ~line 2217) and the
## `.chain(restore_warnings.iter())` step that is the only path either one
## reaches the returned `warnings` array. All three are the mutants the
## row's own verifier found surviving `cargo test -p cartalith-godot --lib`
## on 2026-09-06 (`let dropped = ...` -> `0usize` in both places, and
## deleting the `.chain(...)`): `WorldGen` is a cdylib `GodotClass` and
## cannot be constructed in a `cargo test`, so the existing unit test
## re-derived the subtraction INSIDE its own body instead of exercising the
## call site, and stayed green with the warning deleted outright. This is
## the probe that call site needed.
##
##   Godot_v4.7.1-stable_win64_console --headless --path . \
##       --script res://_dropwarn_probe.gd
##
## Headless and `extends SceneTree`, `_lmpersist_probe.gd`'s own shape for the
## same reason: nothing here rasterises, and `project_save`/`project_open`
## need no renderer.
##
## ## Method
##
## Save a REAL project (a real world, a real `landmark_run()`), then splice
## one deliberately-bad row into EACH of `entities/landmarks.json`'s
## `results.landmarks` and `annotations/icons.json`'s `icons` -- an unknown
## landmark `kind` and an unresolvable icon `family`, the exact two
## `into_result`/`icon_from_dto` drop conditions the row's comments cite --
## by editing the real archive's BYTES with `ZIPReader`/`ZIPPacker` rather
## than hand-building a project from scratch, so every OTHER slot
## `project_open` reads (world seed, grid, heightmap, `format_version`, ...)
## stays exactly what the engine actually wrote. Reopen in a FRESH `WorldGen`
## and read `warnings` back off the real `#[func]` return, not off a
## re-derived count -- the exact gap the row names.

const BAD_KIND := "totally_bogus_kind_xyz_does_not_exist"
const BAD_FAMILY := "totally_bogus_family_xyz_does_not_exist"
const LM_SLOT := "entities/landmarks.json"
const ICON_SLOT := "annotations/icons.json"

var _fails := 0


func _fail(s: String) -> void:
	_fails += 1
	print("  FAIL: ", s)


## Copies every entry of `src_path` except the two slots under test into a
## new archive at `dst_path`, then writes `landmarks_json`/`icons_json` in
## their place -- so the spliced archive is byte-identical to a real save
## everywhere `project_open` is not the thing being tested.
func _splice_zip(src_path: String, dst_path: String, landmarks_json: String,
		icons_json: String) -> bool:
	var zr := ZIPReader.new()
	if zr.open(src_path) != OK:
		_fail("could not reopen the saved archive to splice it")
		return false
	var names: PackedStringArray = zr.get_files()
	var packer := ZIPPacker.new()
	if packer.open(dst_path) != OK:
		_fail("could not open the spliced archive for writing")
		zr.close()
		return false
	for name in names:
		if name == LM_SLOT or name == ICON_SLOT:
			continue
		packer.start_file(name)
		packer.write_file(zr.read_file(name))
		packer.close_file()
	packer.start_file(LM_SLOT)
	packer.write_file(landmarks_json.to_utf8_buffer())
	packer.close_file()
	packer.start_file(ICON_SLOT)
	packer.write_file(icons_json.to_utf8_buffer())
	packer.close_file()
	packer.close()
	zr.close()
	return true


func _init() -> void:
	var wg: WorldGen = WorldGen.new()
	for m in ["landmark_run", "landmarks", "project_save", "project_open"]:
		if not wg.has_method(m):
			print("FAIL: %s absent -- the probe cannot run" % m)
			quit(1)
			return

	## Same size `_lmpersist_probe.gd` already proved reliably places 20+
	## landmarks at this seed -- not re-gambled on a smaller/faster world.
	wg.generate_sized(24601, 640.0, 192, 144)
	if not wg.landmark_run():
		print("FAIL: landmark_run refused -> ", wg.landmark_last_run().get("error", "?"))
		quit(1)
		return
	var placed_before: Array = wg.landmarks()
	print("DROPWARN placed before splice: ", placed_before.size())
	if placed_before.size() < 1:
		print("FAIL: 0 landmarks placed -- too thin to prove a drop from the rest")
		quit(1)
		return

	var save_path := OS.get_user_data_dir().path_join("_dropwarn_probe.zip")
	var r: Dictionary = wg.project_save(save_path)
	if not bool(r.get("ok", false)):
		print("FAIL: save -> ", r.get("error", "?"))
		quit(1)
		return

	## Read the real `entities/landmarks.json` this save just wrote, and add
	## ONE row no `kind_spec()` will resolve -- `into_result`'s first drop
	## condition ("unknown kind").
	var zr := ZIPReader.new()
	if zr.open(save_path) != OK:
		print("FAIL: could not reopen the saved archive")
		quit(1)
		return
	var lm_text := "{}"
	if LM_SLOT in zr.get_files():
		lm_text = zr.read_file(LM_SLOT).get_string_from_utf8()
	zr.close()
	var lm_obj: Variant = JSON.parse_string(lm_text)
	if typeof(lm_obj) != TYPE_DICTIONARY:
		print("FAIL: %s did not parse as an object" % LM_SLOT)
		quit(1)
		return
	var lm_dict: Dictionary = lm_obj
	if not lm_dict.has("results") or lm_dict["results"] == null:
		_fail("%s carries no 'results' after a successful landmark_run() -- cannot splice" % LM_SLOT)
		quit(1)
		return
	var results: Dictionary = lm_dict["results"]
	var lm_list: Array = results.get("landmarks", [])
	var rows_before := lm_list.size()
	if rows_before != placed_before.size():
		_fail("document carries %d landmarks, the run placed %d -- fixture assumption is wrong"
			% [rows_before, placed_before.size()])
	lm_list.append({
		"kind": BAD_KIND, "x": 0, "y": 0,
		"elevation_m": 0.0, "score": 0.0, "importance": 0.0, "causal": [],
	})
	results["landmarks"] = lm_list
	lm_dict["results"] = results
	var lm_json := JSON.stringify(lm_dict)

	## `annotations/icons.json` may not exist yet (no manual icons placed) --
	## written fresh with exactly one row `icon_from_dto` cannot resolve
	## ("unresolvable family").
	var icons_json := JSON.stringify({"icons": [
		{"x": 5.0, "y": 5.0, "family": BAD_FAMILY, "slot": "", "scale": 1.0},
	]})

	var spliced_path := OS.get_user_data_dir().path_join("_dropwarn_probe_spliced.zip")
	if not _splice_zip(save_path, spliced_path, lm_json, icons_json):
		quit(1)
		return

	## The reopen, in a fresh engine object -- the state under test is what
	## `project_open` reports, not what this one still happens to hold.
	var wg2: WorldGen = WorldGen.new()
	var o: Dictionary = wg2.project_open(spliced_path)
	if not bool(o.get("ok", false)):
		print("FAIL: reopen of the spliced archive -> ", o.get("error", "?"))
		quit(1)
		return

	var warnings: Array = Array(o.get("warnings", []))
	print("DROPWARN warnings on reopen: ", warnings)

	## The exact strings `project_bridge.rs` formats -- SLOT constant, the
	## real row count (survivors + the one bad row), and the literal
	## suffix text, byte for byte off the source.
	var want_lm := "%s: 1 of %d landmarks skipped (unknown kind, off-grid cell, or a non-finite number)" \
		% [LM_SLOT, rows_before + 1]
	var want_icon := "%s: 1 of 1 icons skipped (unresolvable family or origin)" % ICON_SLOT

	if want_lm in warnings:
		print("  ok: landmark drop-count warning present: ", want_lm)
	else:
		_fail("landmark drop-count warning absent or wrong -- want: %s" % want_lm)

	if want_icon in warnings:
		print("  ok: icon drop-count warning present: ", want_icon)
	else:
		_fail("icon drop-count warning absent or wrong -- want: %s" % want_icon)

	## The count is reported AND the drop is real: the survivor count must be
	## exactly what went in before the splice, and the bad row itself must
	## not have snuck through as a real landmark.
	var after: Array = wg2.landmarks()
	if after.size() != rows_before:
		_fail("landmarks() returned %d after reopen, want %d (the bad row must not survive)"
			% [after.size(), rows_before])
	else:
		print("  ok: landmarks() returned %d after reopen, matching the pre-splice count" % after.size())
	for lm: Dictionary in after:
		if String(lm.get("kind", "")) == BAD_KIND:
			_fail("the unresolvable-kind landmark survived into landmarks()")

	print("DROPWARN %s (%d failed)" % ["ALL PASS" if _fails == 0 else "SOME FAILED", _fails])
	quit(1 if _fails > 0 else 0)
