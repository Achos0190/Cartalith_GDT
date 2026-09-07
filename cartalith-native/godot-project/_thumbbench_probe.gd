extends SceneTree

## Lane THUMBNAILS: does `OpenProjectDialog.thumbnail()` draw the world, and
## what does it cost the gallery? Headless on purpose -- every image asserted
## here is a CPU-side `Image` the renderer never touches. The two claims this
## CANNOT make -- that a tile reaches the screen, and that the phone card draws
## one -- are `_thumbbench_shot.gd` and `_thumbbench_phone.gd`, both windowed.
##
## Rebuild the corpus first; it is 21 archives in the projects root and the
## goldens beside this file, and it is not checked in:
##
##   python _thumbbench_corpus.py && python _thumbbench_corpus_edges.py
##   Godot_v4.7.1-stable_win64_console.exe --headless --script _thumbbench_probe.gd
##
## The BEFORE half of section A needs the pre-change file, which is also not
## kept (a second copy of a 1 300-line shell file in the project directory is
## an edit-the-wrong-one hazard). Regenerate it when a timing is wanted:
##
##   git show HEAD:cartalith-native/godot-project/shell/open_project_dialog.gd ##     | sed "s/^class_name OpenProjectDialog$//" > _thumbbench_before.gd

const BEFORE := "res://_thumbbench_before.gd"
const OUT := "user://_thumbbench"
const GOLDEN := "_thumbbench_golden.zip"

var fails: Array[String] = []

func ok(cond: bool, msg: String) -> void:
	if cond:
		print("  PASS  ", msg)
	else:
		fails.append(msg)
		print("  FAIL  ", msg)

## Emptied through an open `DirAccess`, not `remove_absolute()`: a run that
## used the latter left files behind and reported 20 cached tiles where 13 were
## rendered, which would have made the cold timing a warm one.
func wipe(dir: String) -> void:
	var da := DirAccess.open(dir)
	if da == null:
		return
	for f in da.get_files():
		var err := da.remove(String(f))
		if err != OK:
			print("  !!    could not remove %s (%d)" % [f, err])
	var left := DirAccess.open(dir).get_files().size()
	if left != 0:
		print("  !!    %s still holds %d file(s) after the wipe" % [dir, left])

## Is this pixel on the SEA ramp?
##
## **`b > r` alone is wrong, and the first run of this probe proved it**: the
## LAND ramp's top stop is snow, (248,248,250), whose blue also exceeds its
## red, so a whole-tile land check called every snowcap "sea". Every SEA stop
## has `b - r` of at least 36/255 and every interpolation between two of them
## stays above it; snow's is 2/255. 0.1 sits between, with room either side.
func is_sea(c: Color) -> bool:
	return c.b - c.r > 0.1

## One gallery open: build the composition, then time only `_refresh()` --
## which is exactly what `open()` runs after presenting the window, and the
## only part of the open that this change touches.
func time_refresh(script_path: String) -> Array:
	var dlg = (load(script_path) as GDScript).new()
	root.add_child(dlg)
	dlg._scope = "all"
	dlg._build()
	var t0 := Time.get_ticks_usec()
	dlg._refresh()
	var dt := Time.get_ticks_usec() - t0
	var n: int = dlg._grid.get_child_count() - 1   ## less the dashed import tile
	return [dt, n, dlg]

func _init() -> void:
	DirAccess.make_dir_recursive_absolute(OUT)
	wipe(OUT)
	var root_dir := DccSettings.storage_root("projects")
	print("projects root: ", root_dir)
	print("thumb cache:   ", OpenProjectDialog.THUMB_CACHE_DIR)

	# -- A. before/after gallery-open timings ------------------------------
	wipe(OpenProjectDialog.THUMB_CACHE_DIR)
	OpenProjectDialog._thumb_cache = {}
	OpenProjectDialog._meta_cache = {}

	var before_tiles := -1
	if FileAccess.file_exists(BEFORE):
		var b = time_refresh(BEFORE)
		before_tiles = b[1]
		print("BEFORE (identicon)        %8.1f ms  %d tiles" % [b[0] / 1000.0, b[1]])
		b[2].queue_free()
	else:
		print("BEFORE (identicon)        skipped -- %s absent, see this file's header" % BEFORE)

	OpenProjectDialog._meta_cache = {}
	var c = time_refresh("res://shell/open_project_dialog.gd")
	print("AFTER  cold (no cache)    %8.1f ms  %d tiles" % [c[0] / 1000.0, c[1]])
	var cold_files := DirAccess.get_files_at(OpenProjectDialog.THUMB_CACHE_DIR).size()
	c[2].queue_free()

	OpenProjectDialog._thumb_cache = {}
	var d = time_refresh("res://shell/open_project_dialog.gd")
	print("AFTER  warm disk cache    %8.1f ms  %d tiles" % [d[0] / 1000.0, d[1]])
	d[2].queue_free()

	var e = time_refresh("res://shell/open_project_dialog.gd")
	print("AFTER  warm memory cache  %8.1f ms  %d tiles" % [e[0] / 1000.0, e[1]])
	e[2].queue_free()

	ok((before_tiles == -1 or before_tiles == c[1]) and c[1] == d[1],
		"the same tile count before and after (%d)" % c[1])
	ok(cold_files > 0, "cold open wrote %d cached tiles" % cold_files)
	ok(e[0] <= d[0], "memory cache is not slower than the disk cache")

	# -- B. the render is the world, checked against an outside golden ------
	var img: Image = OpenProjectDialog._render_thumbnail(root_dir.path_join(GOLDEN))
	ok(img != null, "the golden world renders")
	if img != null:
		ok(img.get_width() == 96 and img.get_height() == 72,
			"96x72, got %dx%d" % [img.get_width(), img.get_height()])
		var want := FileAccess.get_file_as_bytes("res://_thumbbench_golden.raw")
		ok(want.size() == 96 * 72 * 3, "golden fixture loaded (%d bytes)" % want.size())
		var worst := 0
		var exact := 0
		var got := img.get_data()
		for i in mini(got.size(), want.size()):
			var diff: int = absi(int(got[i]) - int(want[i]))
			worst = maxi(worst, diff)
			if diff == 0:
				exact += 1
		ok(worst == 0, "every channel matches the outside golden exactly (worst delta %d, %d/%d exact)"
			% [worst, exact, want.size()])
		img.save_png(OUT.path_join("golden_render.png"))

		## Not just "some pixels differ": the tile has to carry sea AND land,
		## or a ramp bug that painted the whole world one colour would pass.
		var seen := {}
		for y in 72:
			for x in 96:
				seen[img.get_pixel(x, y).to_rgba32()] = true
		ok(seen.size() > 40, "the tile carries %d distinct colours" % seen.size())
		var blue := 0
		for y in 72:
			for x in 96:
				if is_sea(img.get_pixel(x, y)):
					blue += 1
		var green: int = 96 * 72 - blue
		ok(blue > 200 and green > 200, "sea %d px and land %d px, both present" % [blue, green])

	# -- C. two worlds are two pictures -------------------------------------
	var a0: Image = OpenProjectDialog._render_thumbnail(root_dir.path_join("_thumbbench_big0.zip"))
	var a1: Image = OpenProjectDialog._render_thumbnail(root_dir.path_join("_thumbbench_big1.zip"))
	ok(a0 != null and a1 != null and a0.get_data() != a1.get_data(),
		"two different worlds render two different tiles")
	if a0 != null:
		a0.save_png(OUT.path_join("big0.png"))
	if a1 != null:
		a1.save_png(OUT.path_join("big1.png"))

	# -- D. the fallback, at every rung it has to cover ---------------------
	for bad in ["_thumbbench_damaged.zip", "_thumbbench_foreign.zip"]:
		ok(OpenProjectDialog._render_thumbnail(root_dir.path_join(bad)) == null,
			"%s refuses to render" % bad)
		ok(OpenProjectDialog.thumbnail(root_dir.path_join(bad)) is GradientTexture2D,
			"%s still gets an identicon tile" % bad)
	ok(OpenProjectDialog._render_thumbnail(root_dir.path_join("_nonexistent.zip")) == null,
		"a path that is not there refuses to render")
	## Section 7's own refusal table: sea_level absent is "Refuse. It is the
	## coastline. A default would silently redraw it." Both of these carry a
	## length-correct heightmap, so only the sea_level clause can reject them.
	for s_bad in ["_thumbbench_nosea.zip", "_thumbbench_badsea.zip"]:
		ok(OpenProjectDialog._render_thumbnail(root_dir.path_join(s_bad)) == null,
			"%s refuses rather than inventing a coastline" % s_bad)
		ok(OpenProjectDialog.thumbnail(root_dir.path_join(s_bad)) is GradientTexture2D,
			"%s still gets an identicon tile" % s_bad)

	# -- D2. the grid, at the edges section 7 names --------------------------
	## `grid_width` absent and `grid_width` 0 are both "Refuse" in section 7's
	## table; 1x1 is the smallest grid it allows and must therefore *render*.
	## Without the 1x1 case a `gw < 2` guard passes everything this probe has.
	for g_bad in ["_thumbbench_nogrid.zip", "_thumbbench_zerogrid.zip"]:
		ok(OpenProjectDialog._render_thumbnail(root_dir.path_join(g_bad)) == null,
			"%s refuses (no usable grid)" % g_bad)
	var one: Image = OpenProjectDialog._render_thumbnail(root_dir.path_join("_thumbbench_one.zip"))
	ok(one != null, "the smallest grid section 7 allows (1x1) still renders")

	## A 96x72 world is the one grid where the sample index actually reaches
	## `gw - 1` / `gh - 1`, which is what makes the two axis clamps testable at
	## all: for every larger grid the largest index is `(96-1)*gw/96 < gw - 1`.
	var ex: Image = OpenProjectDialog._render_thumbnail(root_dir.path_join("_thumbbench_exact.zip"))
	ok(ex != null, "a world exactly the tile's own size renders")
	if ex != null:
		ex.save_png(OUT.path_join("exact.png"))
		var wex := FileAccess.get_file_as_bytes("res://_thumbbench_exact.raw")
		ok(wex.size() == 96 * 72 * 3, "exact-size golden loaded (%d bytes)" % wex.size())
		var gex := ex.get_data()
		var bad := 0
		for i in mini(gex.size(), wex.size()):
			if gex[i] != wex[i]:
				bad += 1
		ok(bad == 0, "the 96x72 world matches its outside golden (%d bytes differ)" % bad)

	# -- E. the flat (section 15) layout ------------------------------------
	var flat: Image = OpenProjectDialog._render_thumbnail(root_dir.path_join("_thumbbench_flat.zip"))
	ok(flat != null, "a flat archive renders from params.json GW/GH/state.seaLevel")
	if flat != null:
		flat.save_png(OUT.path_join("flat.png"))

	# -- F. a real engine-written save --------------------------------------
	var real: Image = OpenProjectDialog._render_thumbnail(root_dir.path_join("_thumbbench_real.zip"))
	ok(real != null, "a real engine-written save renders")
	if real != null:
		real.save_png(OUT.path_join("real.png"))

	# -- G. the two hypso guards a conforming save can reach -----------------
	var s0: Image = OpenProjectDialog._render_thumbnail(root_dir.path_join("_thumbbench_sea0.zip"))
	var s1: Image = OpenProjectDialog._render_thumbnail(root_dir.path_join("_thumbbench_sea1.zip"))
	ok(s0 != null and s1 != null, "sea_level 0.0 and 1.0 both render without dividing by zero")
	if s0 != null and s1 != null:
		## Corrected after the first run falsified the version of this that
		## assumed both guards fire at (0,0). `sea_level` 0.0 never takes the
		## `v < sea` branch at all for a [0,1] raster, so the whole tile is the
		## LAND ramp; `sea_level` 1.0 puts everything on the depth ramp except a
		## cell of exactly 1.0, which is the one that reaches `1 - sea <= 0`.
		var sea_at_zero := 0
		for y in 72:
			for x in 96:
				if is_sea(s0.get_pixel(x, y)):
					sea_at_zero += 1
		ok(sea_at_zero == 0, "sea_level 0.0 puts the whole tile on the land ramp (%d sea px)" % sea_at_zero)
		var lowest := 0
		var sea_px := 0
		var land0 := Color(47 / 255.0, 122 / 255.0, 68 / 255.0)
		for y in 72:
			for x in 96:
				var q := s1.get_pixel(x, y)
				if q.is_equal_approx(land0):
					lowest += 1
				elif is_sea(q):
					sea_px += 1
		ok(sea_px > 6000, "sea_level 1.0 puts %d px on the depth ramp" % sea_px)
		ok(lowest >= 1, "and the `1 - sea <= 0` guard is reached %d time(s), reading LAND[0]" % lowest)

	# -- H2. the cache key separates two worlds ------------------------------
	## `_render_thumbnail` above proves the *pixels* differ; this proves the
	## **key** does. A key that dropped the path -- or hashed it to a constant --
	## would serve one world's map for the other, and nothing that compares
	## rendered images would notice.
	wipe(OpenProjectDialog.THUMB_CACHE_DIR)
	OpenProjectDialog._thumb_cache = {}
	OpenProjectDialog.thumbnail(root_dir.path_join("_thumbbench_big0.zip"))
	OpenProjectDialog.thumbnail(root_dir.path_join("_thumbbench_big1.zip"))
	var keyed := DirAccess.open(OpenProjectDialog.THUMB_CACHE_DIR).get_files().size()
	ok(keyed == 2, "two worlds cache under two keys, got %d file(s)" % keyed)

	# -- H. the cache key actually invalidates -------------------------------
	var gp := root_dir.path_join(GOLDEN)
	var t_a := OpenProjectDialog.thumbnail(gp)
	var t_b := OpenProjectDialog.thumbnail(gp)
	ok(t_a == t_b, "the same save twice is the same texture object (memory cache hit)")
	## Touch the file: same bytes, new mtime -> a new key, a new texture.
	var fa := FileAccess.open(gp, FileAccess.READ)
	var blob := fa.get_buffer(fa.get_length())
	fa.close()
	OS.delay_msec(1100)
	var fw := FileAccess.open(gp, FileAccess.WRITE)
	fw.store_buffer(blob)
	fw.close()
	var t_c := OpenProjectDialog.thumbnail(gp)
	ok(t_c != t_a, "a re-save invalidates the cache (new mtime -> new key)")
	ok(t_c is ImageTexture, "and re-renders rather than falling back")

	print("\n%d failure(s)" % fails.size())
	for f in fails:
		print("  - ", f)
	quit(1 if fails.size() > 0 else 0)
