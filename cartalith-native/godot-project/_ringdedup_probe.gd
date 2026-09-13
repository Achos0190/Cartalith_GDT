extends Node
## The ring/glyph de-dup key, asserted at the coordinate where it used to be
## able to disagree with itself.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _ringdedup_probe.tscn
##
## ## The defect this exists for, and why nothing else could see it
##
## `map_overlay.gd::_draw_annotation_marks` suppresses a generated POI glyph
## whose cell already carries a landmark ring (owner ruling 14's "one layer").
## Until 2026-09-06 it built that key **two different ways**: the ring side
## wrote `Vector2i(int(lm.x), int(lm.y))` -- truncation -- and the icon side
## looked it up as `Vector2i(roundi(float(ic.x)), roundi(float(ic.y)))` --
## rounding. Both are exact on an integer, and every coordinate reaching them
## is one today (`icon_bridge/generate.rs::icon_candidates` writes `l.x as f64`
## off a `usize`), so the pair was correct by coincidence of its inputs rather
## than by construction.
##
## The failure mode is silent in the worst way. If either side ever carried a
## sub-cell coordinate the two would part company at every `.5`, the lookup
## would miss, and the landmark would go back to drawing twice -- a ring and a
## POI diamond half a cell apart. That is the defect ruling 14 closed, and
## `_iconmerge_probe.gd` recorded it at **139 ring pixels beside 27 POI-glyph
## pixels** in one 45x45 px box on the way. No test in this tree fails when a
## mark is drawn twice: both marks are real draw calls on real data.
##
## So this probe's centre case is a fixture the shipping engine does not yet
## produce -- a generated POI icon at its landmark's **cell centre**,
## `l.x as f64 + 0.5`. That is the sub-cell convention anyone adding one is
## most likely to reach for, and it is the single value that separates `floori`
## from `roundi`: `floori(24.5)` = 24 (the landmark's own cell), `roundi(24.5)`
## = 25 (one cell down-right of it, where no landmark is).
##
## ## Two sections, because a right predicate nobody calls is still a bug
##
## **Section 1** calls `_ringed_cells()` and `_icon_shadowed_by_ring()`
## directly. It is exact, needs no rasteriser, and it is the assertion that
## would have gone red the day the key was written two ways.
##
## **Section 2** is the framebuffer, and it is here because section 1 cannot
## see the draw pass dropping the call. Same three landmark cells, counted as
## pixels: one ring and no diamond at each.
##
## Pixels mean this runs **windowed**. Under `--headless` Godot loads the dummy
## display driver, `RenderingServer.frame_post_draw` never fires and the first
## capture blocks forever -- the run is stopped, not slow (`MISTAKES.md`,
## reproduced twice). The guard below refuses to start rather than hang.
##
## ## This key is only pinned in ONE direction, and that is not this probe's
## fault
##
## `_ringed_cells()` and `_icon_shadowed_by_ring()` both now call the one
## shared `_mark_cell()` (`map_overlay.gd`), but only the ICON side's own
## fixture (`_icons()`, below) ever carries a sub-cell value -- every
## `_landmarks()` row is an integer `Vector2i`. Re-verified 2026-09-13: reverting
## `_ringed_cells()`'s call back to its pre-fix inline
## `Vector2i(int(lm.x), int(lm.y))` -- the exact mirror of the ICON-side
## mutation this probe DOES catch -- leaves every check here green (13/13
## PASS, byte-identical output). That is not a hole in this file: `Landmark::x`/
## `y` are `usize` (`cartalith-civ/src/landmark.rs`), so a real landmark
## dictionary can never carry the sub-cell coordinate this probe would need
## to tell `floori` apart from a truncating cast, and `roundi` too, since all
## three agree on any exact integer. Adding a fractional row to `_landmarks()`
## would fake a state the type forbids, not test one that exists. **Close this
## when a landmark can hold a sub-cell coordinate** (a `cartalith-civ` type
## change), not by inventing the fixture here.
##
## ## Why the fixture is synthetic, and why that is faithful
##
## `map_overlay.gd` holds no `EngineBridge`: `ViewportHost.refresh_annotations()`
## pushes `_bridge.icon_list()` into `set_manual_icons()` and `_bridge.landmarks()`
## into `set_landmarks()`. Two Arrays of Dictionaries are the overlay's whole
## input, so handing them over directly exercises the real path with no `.dll`,
## no world generation and no seed in the evidence. Row shapes are read off
## `lib.rs::icon_dict` (`{x,y,family,slot,set,scale,origin}`) and
## `lib.rs::landmark_dict` (`{id,kind,class,x,y,...}`), not invented.

const W := 900
const H := 600
const GW := 64
const GH := 40
const BG := Color(0.03, 0.03, 0.04)

## The four cells the pixel census looks at. Ten grid cells apart at the
## closest, which is ~141 px here against a HALF of 22, so no box can see a
## neighbour's mark.
const CELL_INT := Vector2i(10, 10)      ## landmark + generated POI at an exact integer
const CELL_HALF := Vector2i(24, 12)     ## landmark + generated POI at the cell CENTRE
const CELL_EDGE := Vector2i(40, 28)     ## landmark + generated POI just inside the cell
const CELL_ORPHAN := Vector2i(52, 8)    ## generated POI with NO landmark -- must draw

## Cells used only by section 1, where boxes cannot overlap because nothing is
## rasterised. `CELL_NEIGH` is one cell down-right of `CELL_HALF`'s landmark:
## at 14 px that is inside the pixel census's own box, so it is asserted on the
## predicate and left out of the framebuffer pass.
const CELL_NEIGH := Vector2i(25, 13)
const CELL_MANUAL := Vector2i(10, 10)   ## a HAND-PLACED POI on a ring cell

## Half-width of the census box in framebuffer pixels. A continental ring is
## 9 px (`LANDMARK_CLASS_RADIUS`) and a POI diamond 5.5 px
## (`ICON_BASE_RADIUS`) at scale 1; the cell-centre offset this probe
## exists for is half a cell, 7.03 px here. All well inside.
const HALF := 22

var _fails := 0
var _checks := 0


func _p(s: String) -> void:
	print("RINGDEDUP  %s" % s)


func _ok(name: String, got: bool, want: bool) -> void:
	_checks += 1
	if got == want:
		_p("ok    %s   (shadowed=%s)" % [name, got])
	else:
		_fails += 1
		_p("FAIL  %s   got shadowed=%s want %s" % [name, got, want])


func _bad(s: String) -> void:
	_fails += 1
	_p("FAIL  %s" % s)


## Three landmarks, at the three cells whose glyph must be suppressed. Classes
## differ so a mis-keyed ring cannot be mistaken for a right one by size alone;
## none is `cultural`, whose accent orange would have to be told from POI
## yellow by the green channel alone (`_iconmerge_probe.gd`'s own reasoning).
static func _landmarks() -> Array:
	return [
		{"id": 1, "kind": "peak", "class": "continental",
			"x": CELL_INT.x, "y": CELL_INT.y, "importance": 0.9},
		{"id": 2, "kind": "cave", "class": "regional",
			"x": CELL_HALF.x, "y": CELL_HALF.y, "importance": 0.7},
		{"id": 3, "kind": "gorge", "class": "regional",
			"x": CELL_EDGE.x, "y": CELL_EDGE.y, "importance": 0.6},
	]


## `origin` is `IconOrigin::key()`'s own string, written on every row including
## the manual one -- an engine predating the key omits it entirely, which is
## why the overlay tests `has("origin")` rather than defaulting.
##
## Row 2 is the fixture this probe is for: the landmark's cell **centre**.
## Row 3 sits at `+0.999`, the far end of the same cell, so a key that floored
## correctly at `.5` but not near the boundary is still caught.
static func _icons() -> Array:
	return [
		{"x": float(CELL_INT.x), "y": float(CELL_INT.y), "family": "poi",
			"slot": "mountain_peak", "set": "", "scale": 1.0, "origin": "generated"},
		{"x": float(CELL_HALF.x) + 0.5, "y": float(CELL_HALF.y) + 0.5, "family": "poi",
			"slot": "cave", "set": "", "scale": 1.0, "origin": "generated"},
		{"x": float(CELL_EDGE.x) + 0.999, "y": float(CELL_EDGE.y) + 0.999, "family": "poi",
			"slot": "gorge", "set": "", "scale": 1.0, "origin": "generated"},
		{"x": float(CELL_ORPHAN.x), "y": float(CELL_ORPHAN.y), "family": "poi",
			"slot": "cave", "set": "", "scale": 1.0, "origin": "generated"},
	]


var _vp: SubViewport
var _ov: Control
var _blank_vp: SubViewport
var _blank: PackedByteArray = PackedByteArray()


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 180.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		_p("WATCHDOG -- probe did not finish")
		get_tree().quit(2))
	wd.start()

	if DisplayServer.get_name() == "headless":
		_p("ABORT: section 2 measures pixels and cannot run headless -- "
			+ "RenderingServer.frame_post_draw never fires with the dummy driver.")
		_p("  Godot_v4.7.1-stable_win64_console.exe --path . _ringdedup_probe.tscn")
		get_tree().quit(2)
		return

	_vp = _make_vp()
	_ov = Control.new()
	_ov.set_script(load("res://map_overlay.gd"))
	_ov.size = Vector2(W, H)
	_vp.add_child(_ov)
	_ov.set_civ_data([], [], [], GW, GH, 0.0)
	_ov.set_manual_icons(_icons())
	_ov.set_landmarks(_landmarks())

	_section_predicate()

	## The negative control. Same viewport, same background, no overlay at all:
	## the exact framebuffer "nothing was drawn" produces. Without it a census
	## of zeroes reads as "the duplicate is gone" when it really means the
	## fixture never reached the control.
	_blank_vp = _make_vp()

	for f in 3:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	_blank = _blank_vp.get_texture().get_image().get_data()
	if _blank.is_empty():
		_bad("the blank control captured nothing -- every census below would be vacuous")

	await _section_pixels()

	_p("---- %d checks, %s ----" % [_checks,
		"PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


## Section 1: the key itself. `_ringed_cells()` and `_icon_shadowed_by_ring()`
## are the two halves `_draw_annotation_marks` calls, and calling them here is
## the only way to state the invariant as an equality rather than as a pixel
## count.
func _section_predicate() -> void:
	_p("== section 1: the de-dup key ==")
	var ringed: Dictionary = _ov._ringed_cells()

	## First, and before anything is concluded from a `false`: the ring set is
	## the size the fixture asked for. An empty `ringed` makes every
	## "not shadowed" below true for the wrong reason.
	_checks += 1
	if ringed.size() != 3:
		_fails += 1
		_p("FAIL  _ringed_cells() holds %d cells, want 3 -- every case below is vacuous"
			% ringed.size())
	else:
		_p("ok    _ringed_cells() holds the fixture's 3 landmark cells: %s" % [ringed.keys()])

	var ic: Array = _icons()

	## The shipping shape: an exact integer, which both `floori` and `roundi`
	## resolve identically. This is what says the change moved no real world.
	_ok("integer coordinate (%d.0) still de-dups" % CELL_INT.x,
		_ov._icon_shadowed_by_ring(ic[0], ringed), true)

	## **The row.** Cell centre. `floori(24.5)` = 24 = the landmark's cell;
	## `roundi(24.5)` = 25, one cell away, and the glyph draws beside its ring.
	_ok("SUB-CELL: cell centre (%d.5) de-dups" % CELL_HALF.x,
		_ov._icon_shadowed_by_ring(ic[1], ringed), true)

	## The far end of the same cell -- `floori(40.999)` = 40. A key that
	## rounded would send this one cell down-right too.
	_ok("SUB-CELL: cell far edge (%d.999) de-dups" % CELL_EDGE.x,
		_ov._icon_shadowed_by_ring(ic[2], ringed), true)

	## The positive control for every `true` above: a generated POI icon whose
	## cell has no landmark must NOT be suppressed. Without this, a predicate
	## hardwired to `true` would pass the three cases above.
	_ok("no landmark at this cell -- the glyph survives",
		_ov._icon_shadowed_by_ring(ic[3], ringed), false)

	## And the other direction: one whole cell down-right of a landmark is a
	## different cell. `floori` must not swallow a neighbour to make the
	## sub-cell case pass.
	var neigh := {"x": float(CELL_NEIGH.x), "y": float(CELL_NEIGH.y), "family": "poi",
		"slot": "cave", "set": "", "scale": 1.0, "origin": "generated"}
	_ok("the NEXT cell (%d.0, %d.0) is not shadowed" % [CELL_NEIGH.x, CELL_NEIGH.y],
		_ov._icon_shadowed_by_ring(neigh, ringed), false)

	## Authored content never answers to a generated-content rule, sub-cell or
	## not. Same cell as landmark 1, same half-cell offset as the row's own
	## case: if the key ever started ignoring `origin` this goes red.
	var manual := {"x": float(CELL_MANUAL.x) + 0.5, "y": float(CELL_MANUAL.y) + 0.5,
		"family": "poi", "slot": "cave", "set": "", "scale": 1.0, "origin": "manual"}
	_ok("a HAND-PLACED POI on a ring cell is never shadowed",
		_ov._icon_shadowed_by_ring(manual, ringed), false)

	## A generated icon of another family has no landmark source, so the rule
	## does not reach it however exactly its cell matches.
	var feat := {"x": float(CELL_HALF.x) + 0.5, "y": float(CELL_HALF.y) + 0.5,
		"family": "feature", "slot": "tree_conifer", "set": "", "scale": 1.0,
		"origin": "generated"}
	_ok("a generated FEATURE icon on a ring cell is never shadowed",
		_ov._icon_shadowed_by_ring(feat, ringed), false)

	## Landmarks off: no ring draws, so nothing can be shadowed by one. The
	## glyphs at those cells are hidden by `_icon_layer_hidden()` instead, which
	## `_iconmerge_probe.gd` already measures in pixels.
	_ov.set_landmarks_visible(false)
	var off: Dictionary = _ov._ringed_cells()
	_checks += 1
	if off.is_empty():
		_p("ok    Landmarks off -- _ringed_cells() is empty, nothing can be shadowed")
	else:
		_fails += 1
		_p("FAIL  Landmarks off but _ringed_cells() holds %d cells" % off.size())
	_ov.set_landmarks_visible(true)


## Section 2: the same three cells, counted in the framebuffer. Section 1 can
## be entirely green while `_draw_annotation_marks` stops calling the predicate
## -- this is what says the suppression reaches the screen.
func _section_pixels() -> void:
	_p("== section 2: the framebuffer ==")
	_ov.queue_redraw()
	for f in 3:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var img: Image = _vp.get_texture().get_image()
	if img.get_data() == _blank:
		_bad("the frame is byte-identical to the blank control -- nothing drew")
		return

	var seen := {}
	for cell in [CELL_INT, CELL_HALF, CELL_EDGE, CELL_ORPHAN]:
		var c := _census(img, cell)
		seen[cell] = c
		_p("  cell %-8s ring=%-5d poi=%-5d" % [
			"(%d,%d)" % [cell.x, cell.y], c["ring"], c["poi"]])

	## Runs FIRST and unconditionally: it is what makes every `poi == 0` below
	## mean "suppressed" rather than "the census cannot see a POI diamond".
	_checks += 1
	if int(seen[CELL_ORPHAN]["poi"]) <= 0:
		_fails += 1
		_p("FAIL  cell (%d,%d) drew no POI glyph -- every 'poi=0' below is vacuous"
			% [CELL_ORPHAN.x, CELL_ORPHAN.y])
	else:
		_p("ok    the un-landmarked generated POI still draws its own glyph (poi=%d)"
			% int(seen[CELL_ORPHAN]["poi"]))

	for cell in [CELL_INT, CELL_HALF, CELL_EDGE]:
		var c: Dictionary = seen[cell]
		_checks += 1
		if int(c["ring"]) <= 0:
			_fails += 1
			_p("FAIL  cell (%d,%d) has no landmark ring -- the merge deleted the "
				% [cell.x, cell.y] + "landmark layer instead of de-duplicating it")
		elif int(c["poi"]) != 0:
			_fails += 1
			_p("FAIL  cell (%d,%d) draws a POI glyph BESIDE its ring (poi=%d) -- "
				% [cell.x, cell.y, int(c["poi"])] + "the landmark is drawn twice")
		else:
			_p("ok    cell (%d,%d) carries one mark and it is the ring (ring=%d poi=0)"
				% [cell.x, cell.y, int(c["ring"])])


func _make_vp() -> SubViewport:
	var vp := SubViewport.new()
	vp.size = Vector2i(W, H)
	vp.transparent_bg = false
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var bg := ColorRect.new()
	bg.size = Vector2(W, H)
	bg.color = BG
	vp.add_child(bg)
	return vp


## Pixels in a box around `cell`'s own screen position, classified by dominant
## hue -- `_iconmerge_probe.gd`'s classifier, and deliberately the same one:
## background is near-black, POI yellow is `(0.941, 0.894, 0.259)` and a
## physical landmark ring is the cool blue `(0.612, 0.769, 0.816)`.
## `ICON_OUTLINE` and `LANDMARK_OUTLINE` are both near-black and fall under the
## brightness floor, so an outline never scores as a mark.
##
## The box is centred on `_cell_to_screen`'s +0.5 centring because that is
## where the ring lands (`+0.5`). A glyph drawn through `_point_to_screen`
## sits up to half a cell from that centre -- 7.03 px at this grid's 14.06 px
## cell -- and every fixture coordinate here is within one cell of it, so no
## glyph can fall outside HALF.
func _census(img: Image, cell: Vector2i) -> Dictionary:
	var rect: Rect2 = _ov.displayed_rect()
	## The overlay's own transform, called rather than re-derived: a second
	## copy of the letterbox maths here could drift from the one under test and
	## quietly measure the wrong box.
	var c: Vector2 = _ov._cell_to_screen(Vector2(cell.x, cell.y), rect)
	var out := {"ring": 0, "poi": 0}
	var x0 := maxi(0, int(c.x) - HALF)
	var x1 := mini(W - 1, int(c.x) + HALF)
	var y0 := maxi(0, int(c.y) - HALF)
	var y1 := mini(H - 1, int(c.y) + HALF)
	for y in range(y0, y1 + 1):
		for x in range(x0, x1 + 1):
			var p := img.get_pixel(x, y)
			## Brightness floor: an antialiased edge fading into the background
			## must not score as the mark it belongs to.
			if maxf(p.r, maxf(p.g, p.b)) < 0.30:
				continue
			if p.b > p.r and p.b > p.g:
				out["ring"] += 1
			elif p.r > 0.45 and p.g > 0.45 and p.b < 0.40:
				out["poi"] += 1
	return out
