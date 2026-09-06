extends Node
## Owner ruling 14's visible half: `map_overlay.gd` must draw ONE annotation
## layer over the one icon collection, not a manual-icon pass and a landmark
## pass that both draw the same generated landmark.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _iconmerge_probe.tscn
##
## ## Why this probe measures pixels and therefore runs windowed
##
## The defect is *two marks where one belongs*, which no scene-graph assertion
## can see: both passes are real code that runs, and `_manual_icons` and
## `_landmarks` are both non-empty in the world that has the bug. Only the
## framebuffer says whether one cell carries one mark or two. Under
## `--headless` Godot loads the dummy display driver and
## `RenderingServer.frame_post_draw` never fires, so the first capture blocks
## forever -- the run is stopped, not slow (`MISTAKES.md`, reproduced twice).
## The guard below refuses to run rather than hang.
##
## ## Why the fixture is synthetic, and why that is faithful rather than convenient
##
## `map_overlay.gd` holds no `EngineBridge`: `ViewportHost.refresh_annotations()`
## pushes `_bridge.icon_list()` into `set_manual_icons()` and `_bridge.landmarks()`
## into `set_landmarks()`. So the overlay's whole input is two Arrays of
## Dictionaries, and feeding them by hand exercises exactly the code path the
## app does -- with no `.dll`, no world generation and no seed in the evidence.
##
## The fixture's shape is read off the engine, not invented:
## `icon_bridge/generate.rs::icon_candidates` builds a POI candidate per
## landmark as `IconCandidate { x: l.x as f64, y: l.y as f64, slot:
## poi_slot_for_landmark(&l.kind) }` -- the landmark's own `usize` cell index,
## widened. `IconEditor::generate` stamps every row it places with
## `IconOrigin::Generated`, and `lib.rs::icon_dict` now writes that as the
## `origin` key. So a generated POI icon at the same cell as a landmark is what
## the engine really produces, and the two rows below marked `"generated"` are
## that, verbatim.
##
## ## The three colours this counts, and why they separate
##
## Background is near-black, so every mark is the dominant hue of its own
## pixels: POI yellow `(0.941, 0.894, 0.259)`, Feature green
## `(0.0, 0.620, 0.451)` and the physical landmark ring's cool blue
## `(0.612, 0.769, 0.816)`. No cultural landmark is in the fixture on purpose --
## its accent `(0.878, 0.639, 0.290)` is an orange that would have to be told
## apart from POI yellow by the green channel alone, and a classifier that
## needs a threshold between 0.639 and 0.894 is a classifier that can be wrong.
## `ICON_OUTLINE` and `LANDMARK_OUTLINE` are both near-black and fall under the
## brightness floor, so an outline never scores as a mark.

const W := 900
const H := 600
const GW := 64
const GH := 40
const BG := Color(0.03, 0.03, 0.04)

## Cells the census looks at, and what each is for.
const CELL_DUP_A := Vector2i(10, 10)   ## landmark + generated POI icon
const CELL_DUP_B := Vector2i(20, 20)   ## landmark + generated POI icon
const CELL_RING := Vector2i(30, 30)    ## landmark the pass culled -- ring only
const CELL_MANUAL := Vector2i(40, 15)  ## hand-placed Feature icon
const CELL_GENFEAT := Vector2i(50, 25) ## GENERATED Feature icon (a TREES run)

## Half-width of the census box in framebuffer pixels. A continental ring is
## 9 px and a Feature triangle 5.5 px at scale 1, both well inside this, and
## the nearest two fixture cells are 10 grid cells apart -- 140 px here -- so
## no box can see a neighbour's mark.
const HALF := 22

var _fails := 0


static func _landmarks() -> Array:
	return [
		{"id": 1, "kind": "peak", "class": "continental", "x": CELL_DUP_A.x, "y": CELL_DUP_A.y,
			"importance": 0.9},
		{"id": 2, "kind": "cave", "class": "regional", "x": CELL_DUP_B.x, "y": CELL_DUP_B.y,
			"importance": 0.6},
		{"id": 3, "kind": "gorge", "class": "regional", "x": CELL_RING.x, "y": CELL_RING.y,
			"importance": 0.6},
	]


## `origin` is `IconOrigin::key()`'s own string. A build whose engine predates
## the key omits it entirely, which is why the overlay tests `has("origin")`
## rather than defaulting -- and why this probe writes it explicitly on every
## row, including the manual ones.
static func _icons() -> Array:
	return [
		{"x": float(CELL_MANUAL.x), "y": float(CELL_MANUAL.y), "family": "feature",
			"slot": "mountain", "set": "", "scale": 1.0, "origin": "manual"},
		{"x": float(CELL_DUP_A.x), "y": float(CELL_DUP_A.y), "family": "poi",
			"slot": "mountain_peak", "set": "", "scale": 1.0, "origin": "generated"},
		{"x": float(CELL_DUP_B.x), "y": float(CELL_DUP_B.y), "family": "poi",
			"slot": "cave", "set": "", "scale": 1.0, "origin": "generated"},
		{"x": float(CELL_GENFEAT.x), "y": float(CELL_GENFEAT.y), "family": "feature",
			"slot": "tree_conifer", "set": "", "scale": 1.0, "origin": "generated"},
	]


func _p(s: String) -> void:
	print("ICONMERGE  %s" % s)


func _bad(s: String) -> void:
	_fails += 1
	_p("FAIL  %s" % s)


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
		_p("ABORT: this probe measures pixels and cannot run headless -- "
			+ "RenderingServer.frame_post_draw never fires with the dummy driver.")
		_p("  Godot_v4.7.1-stable_win64_console.exe --path . _iconmerge_probe.tscn")
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

	await _case("landmarks ON ", true)
	await _case("landmarks OFF", false)

	_p("---- %s ----" % ("PASS" if _fails == 0 else "%d FAILED" % _fails))
	get_tree().quit(1 if _fails > 0 else 0)


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


func _case(name: String, landmarks_on: bool) -> void:
	_ov.set_landmarks_visible(landmarks_on)
	_ov.queue_redraw()
	for f in 3:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var img: Image = _vp.get_texture().get_image()
	if img.get_data() == _blank:
		_bad("%s: the frame is byte-identical to the blank control -- nothing drew" % name)
		return

	var seen := {}
	for cell in [CELL_DUP_A, CELL_DUP_B, CELL_RING, CELL_MANUAL, CELL_GENFEAT]:
		var c := _census(img, cell)
		seen[cell] = c
		_p("%s  cell %-8s ring=%-5d poi=%-5d feature=%-5d" % [
			name, "(%d,%d)" % [cell.x, cell.y], c["ring"], c["poi"], c["feature"]])

	## These two run FIRST and unconditionally, because they are what makes
	## every "gone" reading below mean something. A hand-placed icon is authored
	## content and never answers to a generated-content toggle; a generated icon
	## of a non-POI family has no landmark source and answers to it either.
	## If either went to zero, every `poi == 0` under it would be the fixture
	## failing to arrive rather than the merge working.
	for cell in [CELL_MANUAL, CELL_GENFEAT]:
		if int(seen[cell]["feature"]) <= 0:
			_bad("%s: cell (%d,%d) drew no Feature glyph -- every 'gone' reading "
				% [name, cell.x, cell.y] + "in this case is vacuous")

	for cell in [CELL_DUP_A, CELL_DUP_B, CELL_RING]:
		var c: Dictionary = seen[cell]
		if landmarks_on:
			## One mark per landmark, and it is the ring. `poi > 0` here is the
			## duplicate this merge exists to remove; `ring == 0` would mean the
			## merge deleted the landmark layer instead of de-duplicating it.
			if int(c["ring"]) <= 0:
				_bad("%s: cell (%d,%d) has no landmark ring" % [name, cell.x, cell.y])
			if int(c["poi"]) != 0:
				_bad("%s: cell (%d,%d) draws a POI glyph BESIDE its ring (poi=%d) "
					% [name, cell.x, cell.y, c["poi"]] + "-- the landmark is drawn twice")
		else:
			## The whole landmark layer answers to one flag now, glyphs included.
			if int(c["ring"]) != 0 or int(c["poi"]) != 0:
				_bad("%s: cell (%d,%d) still draws with Landmarks off (ring=%d poi=%d)"
					% [name, cell.x, cell.y, c["ring"], c["poi"]])


## Pixels in a box around `cell`'s own screen position, classified by dominant
## hue. `_cell_to_screen`'s +0.5 centring is used for the box because that is
## where a landmark ring lands; a generated POI icon drawn through
## `_point_to_screen` sits half a cell up-left of it, which at this grid is
## 7.0 px and comfortably inside HALF.
func _census(img: Image, cell: Vector2i) -> Dictionary:
	var rect: Rect2 = _ov.displayed_rect()
	## The overlay's own transform, called rather than re-derived: a second
	## copy of the letterbox maths here could drift from the one under test and
	## quietly measure the wrong box.
	var c: Vector2 = _ov._cell_to_screen(Vector2(cell.x, cell.y), rect)
	var out := {"ring": 0, "poi": 0, "feature": 0}
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
			elif p.g > p.r and p.g > 0.30:
				out["feature"] += 1
	return out
