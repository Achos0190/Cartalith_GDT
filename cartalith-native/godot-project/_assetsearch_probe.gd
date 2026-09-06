extends Node
## Windowed proof for the Asset library window's own search well
## (`asset_library_window.gd::_slot_matches()` / `SEARCH_PLACEHOLDER`).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 \
##       _assetsearch_probe.tscn -- --nowelcome
##
## **Windowed on purpose, and `--headless` is refused below.** This is a visible
## control and the assertions include drawn pixels; `ImageTexture.update()` is a
## no-op under `--headless`, so a headless run would pass vacuously.
## `MISTAKES.md`'s "run a pixel probe" row.
##
## What it establishes, in order, each against a measured control state:
##   1. Both wells carry `SEARCH_PLACEHOLDER` -- one string, two compositions.
##   2. A query narrows the grid, and clearing it restores the original count
##      exactly (the restore is the control for the narrowing).
##   3. **tag** matching is real: the same query scores 0 before the tag is
##      written and exactly 1 after, on a slot whose name/id/code cannot match.
##      Without the before-half, a 1 proves nothing about which field matched.
##   4. **set** matching is real, on the `custom` family -- which is also the
##      branch that until 2026-09-06 did not match `id` at all.
##   5. The pixel half: non-uniform rows inside the grid's own rect, which is
##      palette-agnostic (`MISTAKES.md`: a threshold is palette-bound, a
##      distinct-colour/uniform-row measure is not). This machine boots LIGHT
##      and the run prints which palette it got.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them.

## `biomes` is the largest family in `FAMILIES` (15 frozen slots), so it is both
## the widest narrowing available and the worst case for `_slot_matches()`'
## per-slot `as_slot_summary()` fallback.
const FAM := "biomes"
## A token no slot id, engine title or grid code in `FAMILIES` contains -- so a
## hit on it can only have come from the tag lookup. Asserted, not assumed:
## step 3's before-half scores it against every slot in the family first.
const TAG := "zzqtag"
const SET := "zzqset"

var app: Node
var win: Node
var field: LineEdit
var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _ok(what: String, cond: bool, detail: String = "") -> void:
	print("ASRCH %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _find_typed(n: Node, cls: String, out: Array) -> void:
	for c in n.get_children():
		if c.is_class(cls):
			out.append(c)
		_find_typed(c, cls, out)

## Drives the REAL widget rather than writing `_search_text` directly: setting
## the field's own text and emitting its own signal is the path a keystroke
## takes, so a well that was built but never connected would fail here.
func _type(s: String) -> void:
	field.text = s
	field.text_changed.emit(s)
	await _frames(4)

func _shown() -> int:
	return (win.get("_grid") as GridContainer).get_child_count()

## Rows inside the grid's rect that are not a single flat colour. Palette
## agnostic: it counts drawn structure, not brightness, so it means the same
## thing under the light palette this machine boots into and under the dark one.
func _busy_rows() -> int:
	var grid := win.get("_grid") as Control
	if grid == null:
		return -1
	var r := grid.get_global_rect()
	var img := get_viewport().get_texture().get_image()
	var x0 := int(max(0.0, r.position.x))
	var x1 := int(min(float(img.get_width()), r.position.x + r.size.x))
	var y0 := int(max(0.0, r.position.y))
	var y1 := int(min(float(img.get_height()), r.position.y + r.size.y))
	if x1 - x0 < 2 or y1 - y0 < 2:
		return -1
	var busy := 0
	for y in range(y0, y1):
		var first := img.get_pixel(x0, y)
		for x in range(x0 + 1, x1):
			if img.get_pixel(x, y) != first:
				busy += 1
				break
	return busy

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		push_error("_assetsearch_probe: run WINDOWED. ImageTexture.update() is a "
			+ "no-op under --headless, so every pixel assertion here passes vacuously.")
		print("ASRCH REFUSED: headless")
		get_tree().quit(2)
		return

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	if "--nowelcome" in OS.get_cmdline_user_args():
		app.open_project_dialog.hide()
		await _frames(2)

	var w := get_window()
	print("ASRCH window=%s  screen_dpi=%d  screen_scale=%.2f  palette=%s" % [
		w.size, DisplayServer.screen_get_dpi(), DisplayServer.screen_get_scale(),
		"dark" if DccTheme.is_dark() else "light"])

	app.open_asset_library(FAM)
	await _frames(20)
	win = app.asset_library_window
	_ok("the Asset library window is open", win != null and (win as Window).visible)
	if win == null:
		get_tree().quit(1)
		return

	# -- 1. the placeholder is one constant, not two hand-written strings ------
	var edits: Array = []
	_find_typed(win, "LineEdit", edits)
	var wells: Array = []
	for e in edits:
		if String((e as LineEdit).placeholder_text) == AssetLibraryWindow.SEARCH_PLACEHOLDER:
			wells.append(e)
	_ok("exactly one search well carries SEARCH_PLACEHOLDER", wells.size() == 1,
		"%d of %d LineEdits  placeholder=%s" % [wells.size(), edits.size(),
			AssetLibraryWindow.SEARCH_PLACEHOLDER])
	if wells.is_empty():
		get_tree().quit(1)
		return
	field = wells[0]

	# -- 2. narrow, then restore ----------------------------------------------
	var n0 := _shown()
	var b0 := _busy_rows()
	print("ASRCH baseline family=%s shown=%d busy_rows=%d header='%s'" %
		[FAM, n0, b0, String((win.get("_grid_header") as Label).text)])
	_ok("the family draws its full slot list before any query", n0 == 15,
		"shown=%d (FAMILIES.biomes has 15 frozen slots)" % n0)
	_snap("00_unfiltered")

	await _type("tund")
	var n1 := _shown()
	var b1 := _busy_rows()
	print("ASRCH query='tund' shown=%d busy_rows=%d header='%s'" %
		[n1, b1, String((win.get("_grid_header") as Label).text)])
	_ok("a name query narrows the grid", n1 == 1, "shown=%d (expected tundra alone)" % n1)
	_ok("and narrows it in PIXELS, not only in node count", b1 < b0,
		"busy rows %d -> %d inside %s" % [b0, b1, (win.get("_grid") as Control).get_global_rect()])
	_snap("01_query_tund")

	await _type("")
	var n2 := _shown()
	var b2 := _busy_rows()
	_ok("clearing the well restores the grid exactly", n2 == n0,
		"shown %d -> %d -> %d   busy rows %d -> %d -> %d" % [n0, n1, n2, b0, b1, b2])
	_snap("02_cleared")

	# -- 3. tag matching, with its own before-half ----------------------------
	await _type(TAG)
	var before := _shown()
	_ok("CONTROL: the tag token matches nothing before it is written",
		before == 0, "shown=%d" % before)

	await _type("")
	var uid := "%s:jungle" % FAM
	var tagged: Dictionary = app.bridge.as_batch_tag(PackedStringArray([uid]), TAG)
	_ok("as_batch_tag wrote the tag", bool(tagged.get("ok", false)), str(tagged))
	await _frames(2)

	await _type(TAG)
	var after := _shown()
	var order: Array = win.get("_slot_order")
	_ok("a tag query now matches exactly the tagged slot", after == 1,
		"shown %d -> %d" % [before, after])
	_ok("and it is the right slot", order.size() == 1 and String(order[0]["id"]) == "jungle",
		"first entry=%s" % ("none" if order.is_empty() else String(order[0]["id"])))
	_snap("03_query_tag")

	# -- 4. set matching, on the custom family --------------------------------
	await _type("")
	## **Two slots, not one, and in different sets.** A single custom slot makes
	## `1 -> 1` the answer for "the set matched", for "everything matched" and
	## for "the filter is not running", which is three states one number cannot
	## separate. With a second slot in another set the query has something to
	## exclude. `library.rs::add_custom_slot()` derives `id` as `slug_id(name)`
	## and keeps the set separately, so neither slot's id can carry the token --
	## re-asserted below off the live entry rather than off that reading.
	var made: Dictionary = app.bridge.as_add_custom_slot("Probe slot", SET)
	var other: Dictionary = app.bridge.as_add_custom_slot("Other slot", "zzqother")
	_ok("as_add_custom_slot made both slots", bool(made.get("ok", false))
		and bool(other.get("ok", false)), "%s / %s" % [str(made), str(other)])
	win.call("_select_family", "custom")
	await _frames(6)
	var c0 := _shown()
	await _type(SET)
	var c1 := _shown()
	var corder: Array = win.get("_slot_order")
	_ok("a set query narrows the custom family to the one slot in that set",
		c0 >= 2 and c1 == 1, "custom shown %d -> %d for set '%s'" % [c0, c1, SET])
	## Which field matched is the whole claim, so it is checked rather than
	## inferred: with name/id/code all excluded and no tags on this slot, `set`
	## is the only field left in `_slot_matches()` that can have produced the
	## hit.
	if corder.size() == 1:
		var e: Dictionary = corder[0]
		_ok("and it matched on `set`, not on name/id/code",
			String(e["id"]).findn(SET) < 0 and String(e["name"]).findn(SET) < 0
				and String(e["code"]).findn(SET) < 0,
			"id=%s name=%s code=%s" % [e["id"], e["name"], e["code"]])
	else:
		_ok("and it matched on `set`, not on name/id/code", false,
			"%d entries, expected 1" % corder.size())
	_snap("04_query_set")

	await _type("")
	print("ASRCH failures=%d" % _fails)
	get_tree().quit(1 if _fails > 0 else 0)

func _snap(tag: String) -> void:
	var dir := "user://assetsearch"
	DirAccess.make_dir_recursive_absolute(dir)
	var img := get_viewport().get_texture().get_image()
	var out := "%s/%s.png" % [dir, tag]
	img.save_png(out)
	print("ASRCH saved ", ProjectSettings.globalize_path(out))
