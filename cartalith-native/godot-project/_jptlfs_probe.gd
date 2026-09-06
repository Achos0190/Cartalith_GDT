extends Node
## **The journey planner's timeline-strip type, at four densities.**
##
## `_phonesweep_probe.gd` already walks 23 screens and reports every label under
## 11 px, and it is what measured this defect -- but it only ever sees the
## strip's **empty** branch (`_route_index < 0`), because no sweep screen commits
## a route. The committed branch draws four more labels into the same row
## (`day 1`, `day N`, and the four legend captions) plus two `custom_minimum_size`
## marks, and none of them had ever been read back at touch density. This probe
## is that read.
##
## Run (one density per process: `DccTheme._touch` / `_phone_mode` are latched
## for the life of the process, so the legs cannot share a run):
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _jptlfs_probe.tscn -- --pointer
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _jptlfs_probe.tscn -- --tablet --force-touch
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _jptlfs_probe.tscn -- --phone --force-touch
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _jptlfs_probe.tscn -- --phone-1440 --force-touch
##
## Every flag is read by `_ready()` below and an unrecognised one is a hard
## refusal, not a default -- `MISTAKES.md`'s "a probe's usage header is a claim
## about the probe's own code" row. `--resolution` is **not** read: the
## composition is chosen by the `SubViewport`'s size, exactly as
## `_tlfloor_probe.gd` and `_tabletparity_probe.gd` do it.
##
## **`--headless` is correct here.** Nothing rasterises and nothing is timed:
## every number is a `get_theme_font_size()` or a `custom_minimum_size`, both
## plain `Control` state that the dummy driver reports truthfully. The pixel
## rule (`MISTAKES.md`: `ImageTexture.update()` is a no-op headless) is about
## framebuffer reads, and this probe makes none. The windowed evidence for the
## same change is `_phonesweep_probe.tscn`, which is where the screen tallies
## come from.
##
## ## The floor each leg is judged against, and why they differ
##
## | leg | SubViewport | `is_*()` | assertion |
## |---|---|---|---|
## | `--pointer` | 1920 x 1080 | neither | **exactly the role's own pointer value** -- the control state |
## | `--tablet` | 2560 x 1600 | `is_tablet()` | **>= 11 px** |
## | `--phone` | 1080 x 2340 | `is_phone()` | **>= 11 px** |
## | `--phone-1440` | 1440 x 3168 | `is_phone()` | **>= 11 px** |
##
## **The pointer leg asserts an equality, not the floor, and that is deliberate.**
## 11 px is a *touch* legibility floor; `ROLE["fs_timeline"]` is `[10, 13]` and
## `ROLE["fs_dock_header"]` is `[9, 11]`, so 10 and 9 are the design's own desktop
## figures for this strip and judging either against a touch bar would be
## `MISTAKES.md`'s "a figure without a density invites being compared to the wrong
## bar". What the desktop leg is for is the opposite question -- that this pass
## changed **nothing** there -- so it pins each label to its own role's pointer
## value, which is byte-identical to the `FS_TINY` (10) and `FS_MICRO` (9) those
## call sites drew before. A floor could not fail on a regression from 10 to 13;
## an equality can. See `_pointer_expect()` for how the two are told apart.
##
## **A blanket `== 10` was tried first and was wrong**, and the record is worth
## keeping: it reported the four legend captions as failures on the desktop leg
## for drawing the 9 px they have always drawn and should. The same run reported
## them at 9 px on TABLET, where 9 is genuinely under the floor -- one leg's
## false positive and one leg's real defect wearing the same output line.
##
## ## What is synthetic here, stated rather than buried
##
## The empty branch is driven the real way: boot the shell, arm the planner,
## read the row. The **committed** branch needs a generated world, two placed
## GIS points and a completed solve, so it is reached by handing the shipped
## `_rebuild_timeline_band()` a synthetic `plan` dictionary. The *function* and
## the *nodes* are the real ones and every measured number comes off the live
## tree; only the input is a fixture. It is shaped to reach the code rather than
## to look plausible -- two stages of different `cat` so both segment tokens are
## exercised, and a non-zero `rest_days` so the trailing block exists, which is
## what makes the legend draw all four captions.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them.

const FLOOR := 11.0            ## `_phonesweep_probe.gd`'s own FONT_FLOOR, so the two agree.

var _fail := 0
var _app: Node
var _leg := ""
var _dens := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## The density this process actually booted into, read back off `DccTheme`
## rather than off the flag that asked for it -- a leg that silently fell into
## the wrong composition would otherwise report the wrong bar's verdict.
func _density() -> String:
	if DccTheme.is_phone():
		return "PHONE"
	if DccTheme.is_tablet():
		return "TABLET"
	if DccTheme.is_laptop():
		return "LAPTOP"
	return "DESKTOP"

func _collect_labels(n: Node, out: Array, depth: int) -> void:
	for c in n.get_children():
		if c is Label:
			out.append({"l": c, "depth": depth})
		_collect_labels(c, out, depth + 1)

## **The pointer control value for one label, resolved structurally.**
##
## The strip's four `fs_timeline` labels are direct children of `timeline_row`;
## the four legend captions hang two levels down inside the legend's own
## `HBoxContainer` beside their swatches. Keyed on that nesting rather than on
## the caption text, which would make the assertion a copy of the strings under
## test and would go quietly vacuous the day one of them is reworded.
##
## Both figures are the pointer half of the role the call site names, so this
## is the "assert the independent thing the value must equal" form and not
## `assert(x == THE_CONSTANT)`: a re-base of either pair moves the code and this
## check together, and a *wrong role* at the call site still fails here.
func _pointer_expect(depth: int) -> int:
	return DccTheme.role_px("fs_timeline") if depth == 0 \
		else DccTheme.role_px("fs_dock_header")

## Walk `timeline_row` and judge every `Label` in it.
##
## **A count of zero is never a pass**: an empty walk is the failure mode this
## whole probe exists to avoid, since a filter that matches nothing reports the
## same clean line as a row that is genuinely correct.
func _walk(branch: String) -> void:
	var row: Control = _app.timeline_row
	print("\n--- %s branch, %s density ---" % [branch, _dens])
	print("  timeline_bar.visible=%s  row.children=%d  phone_scale=%.3f"
		% [str(_app.timeline_bar.visible), row.get_child_count(), DccTheme.phone_scale()])
	var labels: Array = []
	_collect_labels(row, labels, 0)
	if labels.is_empty():
		_fail += 1
		print("  FAIL no Label found in timeline_row -- the walk is broken, not the row.")
		return
	var under := 0
	for e in labels:
		var lab: Label = (e as Dictionary)["l"]
		var depth: int = (e as Dictionary)["depth"]
		var fs: int = lab.get_theme_font_size("font_size")
		var want := _pointer_expect(depth)
		var bad := false
		if _leg == "pointer":
			bad = fs != want
		else:
			bad = float(fs) < FLOOR - 0.001
		if bad:
			under += 1
			_fail += 1
		## The expected column is printed on the pointer leg **only**. On a touch
		## leg `_pointer_expect()` returns `role_px()` as this process resolves
		## it -- the tablet half at tablet, the pointer half on a phone -- so a
		## column headed "pointer" there would be labelling two different things
		## with one name, and on the phone leg it would be a number the
		## assertion does not use at all. `MISTAKES.md`: a wrong reason reads as
		## freshly checked.
		if _leg == "pointer":
			print("    %-6s fs=%-4d (role's pointer value %d, depth %d)  '%s'"
				% ["BAD" if bad else "ok", fs, want, depth, lab.text.substr(0, 40)])
		else:
			print("    %-6s fs=%-4d (floor %.0f, depth %d)  '%s'"
				% ["BAD" if bad else "ok", fs, FLOOR, depth, lab.text.substr(0, 40)])
	if _leg == "pointer":
		print("  %d label(s): %d not equal to their role's own pointer value at %s"
			% [labels.size(), under, _dens])
	else:
		print("  %d label(s): %d under the %.0f px floor at %s"
			% [labels.size(), under, FLOOR, _dens])
	## The marks laid beside that type. Not asserted -- there is no published
	## floor for a legend swatch -- but printed, because a 7 px square next to
	## 35 px of glyph is the mismatch this pass exists to avoid and a reader
	## should not have to take it on trust.
	for c in row.get_children():
		if c is Control and (c as Control).custom_minimum_size != Vector2.ZERO:
			print("    mark   min=%s  %s" % [str((c as Control).custom_minimum_size), c.get_class()])

## The fixture, and the branch it reaches. See the header for what is synthetic.
func _committed_plan() -> Dictionary:
	return {
		"total_days": 12.0,
		"stages": [{"cat": "land"}, {"cat": "water"}],
		"results": [{"days": 7.0, "km": 210.0}, {"days": 3.0, "km": 90.0}],
		"rest_days": 2,
		"layover_days": 0,
	}

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var args := OS.get_cmdline_user_args()
	var legs := 0
	var vp_size := Vector2i(1920, 1080)
	for a in args:
		match a:
			"--pointer":
				_leg = "pointer"; vp_size = Vector2i(1920, 1080); legs += 1
			"--tablet":
				_leg = "tablet"; vp_size = Vector2i(2560, 1600); legs += 1
			"--phone":
				_leg = "phone"; vp_size = Vector2i(1080, 2340); legs += 1
			"--phone-1440":
				## The second phone sample. One size is one sample, and the
				## number under test is `round(px * phone_scale)` -- a factor
				## that differs between the two handsets `_phonesweep_probe.gd`
				## sweeps (2.621 and 3.495), so one of them cannot show that the
				## expression scales at all.
				_leg = "phone"; vp_size = Vector2i(1440, 3168); legs += 1
			"--force-touch":
				pass
			_:
				print("[FATAL] unrecognised argument '%s' -- this probe reads only" % a,
					" --pointer / --tablet / --phone / --phone-1440 / --force-touch")
				get_tree().quit(1); return
	if legs != 1:
		print("[FATAL] pass exactly one leg flag (got ", legs, ")")
		get_tree().quit(1); return
	if _leg != "pointer" and not ("--force-touch" in args):
		print("[FATAL] --%s needs --force-touch: _touch can never be true in this" % _leg,
			" dev environment without it (dcc_shell.gd::_ready)")
		get_tree().quit(1); return

	var vp := SubViewport.new()
	vp.size = vp_size
	vp.gui_embed_subwindows = true
	add_child(vp)
	_app = load("res://shell/app.tscn").instantiate()
	vp.add_child(_app)
	await _frames(60)
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	await _frames(4)

	_dens = _density()
	var want: String = {"tablet": "TABLET", "phone": "PHONE", "pointer": "DESKTOP"}[_leg]
	if _dens != want:
		print("[FATAL] --%s booted into %s, not %s -- the leg would report the" % [_leg, _dens, want],
			" wrong bar's verdict")
		get_tree().quit(1); return
	print("[BOOT] leg=%s  viewport=%dx%d  density=%s  phone_scale=%.3f"
		% [_leg, vp_size.x, vp_size.y, _dens, DccTheme.phone_scale()])

	_app.open_journey_planner()
	await _frames(30)
	var jpv: Node = _app.journey_planner_view
	if jpv == null:
		print("[FATAL] app.journey_planner_view is null"); get_tree().quit(1); return

	_walk("empty (no committed route)")

	## The committed branch. `_route_index` has to be non-negative or
	## `_rebuild_timeline_band()` takes the empty arm the walk above already
	## covered -- so this is set first and checked afterwards by the label count,
	## which rises from 1 to 7 when the arm actually changed.
	jpv.set("_route_index", 0)
	jpv.call("_rebuild_timeline_band", _committed_plan())
	await _frames(12)
	_walk("committed route (synthetic plan -- see header)")

	print("\n_jptlfs_probe (%s): %s" % [_dens, "clean" if _fail == 0 else "%d FAILURE(S)" % _fail])
	get_tree().quit(1 if _fail > 0 else 0)
