extends Node
## Owner ruling 14's spacing half, end to end rather than at the seam:
## **does a hand-placed icon move a landmark, and does deleting the icon put
## the landmark back?**
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _lmicon_probe.tscn
##
## Logic only, no pixels and no frame timing, so `--headless` is right here
## (MISTAKES.md: headless for logic and layout, windowed for anything that
## rasterises).
##
## ## Why a probe and not a unit test
##
## Each half of this already has Rust coverage and neither half is what can
## break. `cartalith_civ::landmark::generate` honours `manual_icons` (its own
## `an_icon_ring_is_weaker_than_a_class_ring`, `an_empty_icon_slice_changes_
## nothing`); `landmark_bridge::icon_to_mark` filters `IconOrigin::Generated`
## out (its own tests). What had no coverage until this file is the **wiring
## between them** -- `WorldGen::landmark_run_inner` reading `self.icons` into
## `LandmarkInputs::manual_icons` -- and `WorldGen` is a cdylib `GodotClass`
## that cannot be constructed in a Rust unit test.
##
## ## The negative is the point
##
## A run that merely shows a landmark moving cannot tell *"the icon moved it"*
## from *"the pass is not reproducible"*. So there are three runs: A with no
## icons, B with one icon on A's first landmark, C after deleting that icon.
## B must lose A's key; **C must equal A exactly**, which is what makes B's
## difference attributable to the icon rather than to noise.
##
## ## The scale is chosen, not inherited
##
## `MANUAL_ICON_EXCLUSION_KM` is 3 km and the ring is `3 / crowding / cell_km`
## cells, so at the sibling probes' 2400 km / 256 cells (9.375 km per cell) the
## disc is 0.32 cells and excludes the icon's own cell and nothing else. That
## is still a real test -- the icon is placed on the landmark's exact cell --
## and the print below states the measured radius so a reader is not left
## guessing how wide the obstacle was.

const PACK := "res://../crates/cartalith-assets/tests/fixtures/reference_pack.zip"
const SEED := 483920
const WIDTH_KM := 2400.0
const GW := 256
const GH := 160

var _fails := 0


func _p(s: String) -> void:
	print("LMICON  %s" % s)


func _bad(s: String) -> void:
	_fails += 1
	_p("FAIL  %s" % s)


## `"<kind>@<x>,<y>"` -- `cartalith_civ::landmark::Landmark::key()`'s own
## shape, and the one `LandmarksDoc` addresses a landmark by. Not `id`, which
## is a position in the result vector and moves whenever a cap does.
func _keys(rows: Array) -> Array:
	var out: Array = []
	for r: Dictionary in rows:
		out.append("%s@%d,%d" % [String(r.get("kind", "?")), int(r.get("x", -1)), int(r.get("y", -1))])
	return out


func _done() -> void:
	_p("---- %s ----" % ("PASS" if _fails == 0 else "%d FAILED" % _fails))
	get_tree().quit(1 if _fails > 0 else 0)


func _ready() -> void:
	var g := WorldGen.new()
	g.set_sea_level(0.45)
	g.set_villages_enabled(true)
	g.generate_sized(SEED, WIDTH_KM, GW, GH)
	var cell_km := WIDTH_KM / float(GW)
	_p("grid %d x %d, %.4f km/cell, icon ring %.3f cells (3 km / crowding 1.0)"
		% [g.get_width(), g.get_height(), cell_km, 3.0 / cell_km])

	## ---- Run A: no icons -------------------------------------------------
	if not g.landmark_run():
		_bad("run A refused: %s" % g.landmark_last_run().get("error", "<no error>"))
		_done()
		return
	var a: Array = g.landmarks()
	var a_keys := _keys(a)
	_p("run A: %d landmarks" % a.size())
	if a.is_empty():
		_bad("run A placed nothing -- every assertion below would be vacuous")
		_done()
		return
	if not g.icon_list().is_empty():
		_bad("run A must start from an empty icon collection; icon_list() has %d rows"
			% g.icon_list().size())

	## ---- Place one icon on run A's first landmark -------------------------
	## `icon_arm` refuses without a loaded pack (`has_asset_pack()`), so a
	## failed load is reported by name rather than silently passing over an
	## icon that was never placed.
	if not g.load_asset_pack(ProjectSettings.globalize_path(PACK)):
		_bad("%s did not load, so `icon_arm` is gated shut and nothing can be placed" % PACK)
		_done()
		return
	if not g.icon_arm("feature", 0, 1.0, 0.0, 0.0):
		_bad("icon_arm refused with a pack loaded")
		_done()
		return

	var target: Dictionary = a[0]
	var tx := int(target.get("x", -1))
	var ty := int(target.get("y", -1))
	var t_key: String = a_keys[0]
	var idx: int = g.icon_place(float(tx), float(ty))
	if idx < 0:
		_bad("icon_place refused at the landmark's own cell (%d, %d)" % [tx, ty])
		_done()
		return
	var placed_row: Dictionary = g.icon_get(idx)
	_p("icon %d placed on %s -> origin=%s at (%s, %s)"
		% [idx, t_key, placed_row.get("origin", "<absent>"),
			placed_row.get("x", "?"), placed_row.get("y", "?")])
	if String(placed_row.get("origin", "")) != "manual":
		_bad("the icon this test rests on is not `manual`; icon_to_mark would drop it")

	## ---- Run B: the icon in force ----------------------------------------
	if not g.landmark_run():
		_bad("run B refused: %s" % g.landmark_last_run().get("error", "<no error>"))
		_done()
		return
	var b: Array = g.landmarks()
	var b_keys := _keys(b)
	_p("run B: %d landmarks (A had %d)" % [b.size(), a.size()])

	if b_keys.has(t_key):
		_bad("THE WIRING IS DEAD: %s survived an icon on its own cell. "
			% t_key
			+ "`landmark_run_inner` is not assembling `LandmarkInputs::manual_icons`, "
			+ "or `icon_to_mark` dropped the row.")
	else:
		## Say which of the two things happened rather than asserting one:
		## the kind may have an alternative candidate nearby, or it may not.
		var same_kind_moved := ""
		for k: String in b_keys:
			if k.begins_with(String(target.get("kind", "?")) + "@") and not a_keys.has(k):
				same_kind_moved = k
				break
		if same_kind_moved != "":
			_p("POSITIVE  %s is gone; the same kind now sits at %s" % [t_key, same_kind_moved])
		else:
			_p("POSITIVE  %s is gone; that kind found no alternative and placed one fewer"
				% t_key)

	if a_keys == b_keys:
		_bad("run B is identical to run A -- the icon changed nothing at all")

	## ---- Run C: the icon deleted -----------------------------------------
	if not g.icon_delete(idx):
		_bad("icon_delete(%d) refused" % idx)
		_done()
		return
	var manual_left := 0
	for r: Dictionary in g.icon_list():
		if String(r.get("origin", "")) == "manual":
			manual_left += 1
	if manual_left != 0:
		_bad("%d hand-placed icons survive the delete; run C is not a clean control" % manual_left)
	if not g.landmark_run():
		_bad("run C refused: %s" % g.landmark_last_run().get("error", "<no error>"))
		_done()
		return
	var c_keys := _keys(g.landmarks())
	_p("run C: %d landmarks" % c_keys.size())
	if c_keys != a_keys:
		var first := -1
		for i in range(maxi(a_keys.size(), c_keys.size())):
			var av: String = a_keys[i] if i < a_keys.size() else "<end>"
			var cv: String = c_keys[i] if i < c_keys.size() else "<end>"
			if av != cv:
				first = i
				_bad("NEGATIVE FAILED  run C differs from run A at %d: A=%s C=%s" % [i, av, cv])
				break
		if first < 0:
			_bad("NEGATIVE FAILED  run C differs from run A in length only: %d vs %d"
				% [a_keys.size(), c_keys.size()])
	else:
		_p("NEGATIVE  run C == run A, all %d keys in order -- so run B's difference is "
			% a_keys.size() + "the icon's and not the pass's")

	_done()
