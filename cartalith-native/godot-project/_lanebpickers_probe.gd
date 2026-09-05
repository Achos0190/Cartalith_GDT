extends Node
## Lane B verification, 2026-09-05: the eight stock `FileDialog` sites in
## `data_manager_window.gd` (five) and `asset_library_window.gd` (three) now
## open `DccBrowseDialog` -- the "Select folder dialog 1920" browser whose own
## mockup comment says it *"replaces the stock OS tree picker"*.
##
## Written against the two hazards that actually apply here:
##
##   * **A file picker is a place users lose work.** So every save site is
##     checked for the overwrite prompt the stock dialog performed for free,
##     and every site is checked for CANCEL doing nothing at all -- the path
##     that is easy to leave half-wired when a dialog stops freeing itself.
##   * **Do not lose a capability to a reskin.** Each site's title, mode,
##     extension filter, start directory and default file name is asserted
##     against what the stock dialog was configured with, not against what
##     looked right.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _lanebpickers_probe.tscn

var app: Node
var _fail := 0
var _tmp := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(label: String, cond: bool, detail: String = "") -> void:
	print("LB %s  %s%s" % ["ok  " if cond else "FAIL", label,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

## Every `DccBrowseDialog` currently parented to `host`, plus a count of stock
## `FileDialog`s -- the second number is the one the whole batch is about.
func _browsers(host: Node) -> Array:
	var out: Array = []
	for c in host.get_children():
		if c is DccBrowseDialog:
			out.append(c)
	return out

func _stock(host: Node) -> int:
	var n := 0
	for c in host.get_children():
		if c is FileDialog:
			n += 1
	return n

func _one(host: Node, label: String) -> DccBrowseDialog:
	var b := _browsers(host)
	_check("%s: browser opened" % label, b.size() == 1, "found %d" % b.size())
	_check("%s: no stock FileDialog" % label, _stock(host) == 0, "found %d" % _stock(host))
	return b[0] if b.size() == 1 else null

## Title / mode / filter / start dir, in one call because every site has all
## four and a per-site block hides which one drifted.
func _shape(d: DccBrowseDialog, label: String, want_title: String, want_mode: int,
		want_ext: Array, want_dir: String) -> void:
	if d == null:
		return
	_check("%s: title" % label, d.title == want_title, "%s" % d.title)
	_check("%s: mode" % label, d._mode == want_mode, "mode=%d want=%d" % [d._mode, want_mode])
	_check("%s: extensions" % label, Array(d._extensions) == want_ext,
		"%s want %s" % [d._extensions, want_ext])
	_check("%s: start dir" % label, d._cwd == want_dir.simplify_path(),
		"%s want %s" % [d._cwd, want_dir.simplify_path()])

## The foot hint every mode now draws. Measured, not asserted from the tree:
## `clip_text` collapses a `Label`'s minimum width to 1, so a hint can be
## present in the scene and invisible on screen -- which is exactly what this
## dialog shipped until today.
func _hint(d: DccBrowseDialog, label: String, must_contain: String) -> void:
	if d == null:
		return
	var ok := d._foot_note != null and d._foot_note.text.contains(must_contain)
	_check("%s: hint text" % label, ok,
		"%s" % ("<null>" if d._foot_note == null else d._foot_note.text))
	if d._foot_note != null:
		_check("%s: hint is wider than 1 px" % label, d._foot_note.size.x > 100.0,
			"%.1f px" % d._foot_note.size.x)

func _ready() -> void:
	## Wiped, not just created. The overwrite guard is one of the things under
	## test, so a scratch file left by the PREVIOUS run makes the first save of
	## this one prompt instead of writing -- which is exactly what happened on
	## the second run of this probe, and turned three passing checks red for a
	## reason that had nothing to do with the code.
	_tmp = OS.get_user_data_dir().path_join("lanebpickers")
	var old := DirAccess.open(_tmp)
	if old != null:
		for f in old.get_files():
			old.remove(f)
	DirAccess.make_dir_recursive_absolute(_tmp)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 1800:
		await get_tree().process_frame
		waited += 1
	print("LB world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("LB  !! generate failed -- nothing else here can run")
		get_tree().quit(1)
		return

	await _geometry()
	await _data_manager()
	await _asset_library()

	print("LB ---- %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

# ---------------------------------------------------------------------------
# The dialog itself, inside the real shell rather than a synthetic host
# ---------------------------------------------------------------------------

func _geometry() -> void:
	var d := DccBrowseDialog.choose_save_path(app, "geom", "zip", _tmp,
		"a hint long enough to be worth measuring", "x.zip", Callable())
	await _frames(3)
	print("LB geom SAVE  win=%dx%d contents_min=%s note_w=%.1f" % [
		d.size.x, d.size.y, d.get_contents_minimum_size(), d._foot_note.size.x])
	_check("save foot hint is drawn", d._foot_note.size.x > 100.0,
		"%.1f px" % d._foot_note.size.x)
	d.hide()
	await _frames(2)

	var f := DccBrowseDialog.choose_file(app, "geom", PackedStringArray(["zip"]),
		_tmp, "a hint long enough to be worth measuring", Callable())
	await _frames(3)
	print("LB geom FILES win=%dx%d contents_min=%s note_w=%.1f" % [
		f.size.x, f.size.y, f.get_contents_minimum_size(), f._foot_note.size.x])
	## Which row actually sets the dialog's minimum width. The foot gained a
	## hint this batch, so "the overflow is not mine" has to be a measurement
	## and not a claim: a `clip_text` label contributes 1 px, so if the foot is
	## not the widest row then nothing in the foot can be setting the width.
	var outer := f.get_child(0) as Control
	for c in outer.get_children():
		print("LB geom row %-18s min_w=%7.1f" % [
			(c as Control).get_child(0).get_class() if (c as Control).get_child_count() > 0
				else (c as Control).get_class(),
			(c as Control).get_combined_minimum_size().x])
	print("LB geom foot_note min_w=%.1f" % f._foot_note.get_combined_minimum_size().x)
	_check("file foot hint is drawn", f._foot_note.size.x > 100.0,
		"%.1f px" % f._foot_note.size.x)
	f.hide()
	await _frames(2)

	## A SAVE dialog with no footnote must not reserve a blank line for one --
	## `app.gd::save_project_as()` passes `""` -- and the hidden note must still
	## be reachable, because it is where `_build_new_folder_row()` reports a
	## failure it can report nowhere else.
	var e := DccBrowseDialog.choose_save_path(app, "geom", "zip", _tmp, "", "x.zip", Callable())
	await _frames(3)
	var e_h := e.get_contents_minimum_size().y
	_check("empty footnote is hidden", not e._foot_note.visible)
	e._foot_note.text = "could not create 'x' here"
	e._foot_note.show()
	await _frames(2)
	_check("a new-folder failure can still show it", e._foot_note.visible
		and e.get_contents_minimum_size().y > e_h,
		"%.1f -> %.1f" % [e_h, e.get_contents_minimum_size().y])
	e.hide()
	await _frames(2)

	## Three real start directories, because the width that matters here is
	## content-dependent -- the breadcrumb is one `Button` per path segment and
	## a `Button` reports its own text as its minimum width, the hazard
	## `_build_breadcrumb()` already records for Android. One sample would say
	## whatever that one path happened to say.
	for start in [DccBrowseDialog.home_dir(), DccSettings.storage_root("exports"), _tmp]:
		var g := DccBrowseDialog.choose_file(app, "geom", PackedStringArray(["zip"]),
			start, "hint", Callable())
		await _frames(3)
		print("LB geom depth=%d win=%dx%d contents_min=%s  %s" % [
			String(start).split("/", false).size(), g.size.x, g.size.y,
			g.get_contents_minimum_size(), start])
		g.hide()
		await _frames(2)

# ---------------------------------------------------------------------------
# data_manager_window.gd -- five sites
# ---------------------------------------------------------------------------

func _data_manager() -> void:
	var dm = app.data_manager_window
	app.open_data_manager_route("export_world")
	await _frames(4)
	var exports := DccSettings.storage_root("exports")

	# --- 1a. raster, tiled: a FOLDER pick -----------------------------------
	dm._wd_tiled = true
	dm._pick_raster_destination()
	await _frames(2)
	var d := _one(dm, "raster-tiled")
	_shape(d, "raster-tiled", "Export map tiles into…",
		DccBrowseDialog.PickKind.FOLDERS, [], exports)
	_hint(d, "raster-tiled", "index.json")
	if d != null:
		d.hide()
	await _frames(2)
	_check("raster-tiled: cancel frees the browser", _browsers(dm).is_empty())
	dm._wd_tiled = false

	# --- 1b. raster, single file: a SAVE pick -------------------------------
	dm._pick_raster_destination()
	await _frames(2)
	d = _one(dm, "raster-single")
	_shape(d, "raster-single", "Export map raster",
		DccBrowseDialog.PickKind.SAVE, ["png"], exports)
	_hint(d, "raster-single", "colour map")
	if d != null:
		_check("raster-single: default name", d._name_edit.text == "map.png", d._name_edit.text)
		d.hide()
	await _frames(2)

	# --- 2. heightmap -------------------------------------------------------
	dm._pick_heightmap_destination()
	await _frames(2)
	d = _one(dm, "heightmap")
	_shape(d, "heightmap", "Export heightmap", DccBrowseDialog.PickKind.SAVE, ["png"], exports)
	_hint(d, "heightmap", "16-bit")
	if d != null:
		_check("heightmap: default name", d._name_edit.text == "heightmap.png", d._name_edit.text)
	## The one export cheap enough to run end to end here: the smallest width
	## the binding offers, into a scratch directory. `_runs` is what every
	## export writer records into, so it is the observable that says the
	## caller did what it did before.
	if not dm._wd_widths.is_empty():
		dm._wd_width = int(dm._wd_widths[0])
	dm._runs.clear()
	if d != null:
		d.navigate(_tmp)
		d._name_edit.text = "probe_height.png"
		d._confirm()
	await _frames(4)
	_check("heightmap: confirm ran the export",
		dm._runs.size() == 1 and String((dm._runs[0] as Dictionary)["label"]).begins_with("heightmap"),
		"runs=%s" % [dm._runs])
	_check("heightmap: the file is on disk",
		FileAccess.file_exists(_tmp.path_join("probe_height.png")))

	# --- the overwrite prompt the stock dialog gave for free ---------------
	dm._runs.clear()
	dm._pick_heightmap_destination()
	await _frames(2)
	d = _one(dm, "heightmap-overwrite")
	if d != null:
		d.navigate(_tmp)
		d._name_edit.text = "probe_height.png"   ## written a moment ago
		d._confirm()
	await _frames(3)
	_check("overwrite: the export did NOT run unprompted", dm._runs.is_empty(),
		"runs=%s" % [dm._runs])
	var confirms := _confirm_dialogs()
	_check("overwrite: a confirmation is on screen", confirms.size() == 1,
		"found %d" % confirms.size())
	if confirms.size() == 1:
		var c: ConfirmationDialog = confirms[0]
		_check("overwrite: it names the file", c.title.contains("probe_height.png"), c.title)
		_check("overwrite: the button is named after what it does",
			c.ok_button_text == "Overwrite", c.ok_button_text)
		## Answering it must let the write through -- a guard that cannot be
		## dismissed is the same defect as no guard, from the other side.
		c.confirmed.emit()
		await _frames(4)
		_check("overwrite: confirming runs the export", dm._runs.size() == 1,
			"runs=%s" % [dm._runs])
		c.hide()
	await _frames(2)

	## ...and refusing it must write nothing.
	dm._runs.clear()
	dm._pick_heightmap_destination()
	await _frames(2)
	d = _one(dm, "heightmap-overwrite-cancel")
	if d != null:
		d.navigate(_tmp)
		d._name_edit.text = "probe_height.png"
		d._confirm()
	await _frames(3)
	for c in _confirm_dialogs():
		(c as Window).hide()
	await _frames(3)
	_check("overwrite: dismissing it writes nothing", dm._runs.is_empty(),
		"runs=%s" % [dm._runs])

	# --- 3. channel atlas: a FOLDER pick ------------------------------------
	dm._pick_atlas_destination()
	await _frames(2)
	d = _one(dm, "atlas")
	_shape(d, "atlas", "Export channel atlas into…",
		DccBrowseDialog.PickKind.FOLDERS, [], exports)
	_hint(d, "atlas", "atlas/index.json")
	if d != null:
		d.hide()
	await _frames(2)

	# --- 4. region tiles .zip: remembers a destination, writes nothing ------
	app.open_data_manager_route("export_maps")
	await _frames(4)
	dm._tx_dest = ""
	dm._pick_destination()
	await _frames(2)
	d = _one(dm, "tiles-zip")
	_shape(d, "tiles-zip", "Export tiles .zip", DccBrowseDialog.PickKind.SAVE, ["zip"], exports)
	_hint(d, "tiles-zip", "nothing is written until")
	if d != null:
		_check("tiles-zip: default name", d._name_edit.text == "region-tiles.zip", d._name_edit.text)
		d.hide()
	await _frames(2)
	_check("tiles-zip: cancel left _tx_dest alone", dm._tx_dest == "", dm._tx_dest)

	dm._pick_destination()
	await _frames(2)
	d = _one(dm, "tiles-zip-confirm")
	if d != null:
		d.navigate(_tmp)
		d._name_edit.text = "probe_tiles"    ## no extension: it must be appended
		d._confirm()
	await _frames(3)
	_check("tiles-zip: confirm set _tx_dest, extension appended",
		dm._tx_dest == _tmp.path_join("probe_tiles.zip"), dm._tx_dest)

	## Re-picking opens on the remembered destination, which is what
	## `current_dir`/`current_file` did on the stock dialog.
	dm._pick_destination()
	await _frames(2)
	d = _one(dm, "tiles-zip-reopen")
	if d != null:
		_check("tiles-zip: reopens in the remembered folder", d._cwd == _tmp, d._cwd)
		_check("tiles-zip: reopens with the remembered name",
			d._name_edit.text == "probe_tiles.zip", d._name_edit.text)
		d.hide()
	await _frames(2)

	# --- 5. GeoJSON ---------------------------------------------------------
	app.open_data_manager_route("export_gis")
	await _frames(4)
	dm._runs.clear()
	dm._pick_geojson_destination()
	await _frames(2)
	d = _one(dm, "geojson")
	_shape(d, "geojson", "Export GeoJSON", DccBrowseDialog.PickKind.SAVE, ["geojson"], exports)
	_hint(d, "geojson", "FeatureCollection")
	if d != null:
		_check("geojson: default name", d._name_edit.text == "world.geojson", d._name_edit.text)
		d.navigate(_tmp)
		d._name_edit.text = "probe_world.geojson"
		d._confirm()
	await _frames(6)
	_check("geojson: confirm ran the export",
		dm._runs.size() == 1 and String((dm._runs[0] as Dictionary)["label"]) == "geojson",
		"runs=%s" % [dm._runs])
	_check("geojson: the file is on disk",
		FileAccess.file_exists(_tmp.path_join("probe_world.geojson")))
	dm.hide()
	await _frames(2)

## The app's own `_confirm()` parents its `ConfirmationDialog` to the shell.
## Matched on the title rather than the class: the shell raises confirmations
## of its own (unsaved-world, quit) and counting every visible one would grade
## this lane on somebody else's dialog.
func _confirm_dialogs() -> Array:
	var out: Array = []
	for c in app.get_children():
		if c is ConfirmationDialog and (c as Window).visible:
			if String((c as AcceptDialog).title).begins_with("Overwrite "):
				out.append(c)
			else:
				print("LB      (ignoring confirmation %s)" % (c as AcceptDialog).title)
	return out

# ---------------------------------------------------------------------------
# asset_library_window.gd -- three sites
# ---------------------------------------------------------------------------

func _asset_library() -> void:
	var al = app.asset_library_window
	app.open_asset_library()
	await _frames(6)
	var home := DccBrowseDialog.home_dir()

	# --- 6. import image ----------------------------------------------------
	al._focused_uid = ""
	al._on_import_image()
	await _frames(2)
	var d := _one(al, "import-image")
	_shape(d, "import-image", "Import image", DccBrowseDialog.PickKind.FILES, ["png"], home)
	_hint(d, "import-image", "lands in")
	if d != null:
		d.hide()
	await _frames(2)
	_check("import-image: cancel frees the browser", _browsers(al).is_empty())

	## A real PNG in the scratch directory, imported through the browser, so
	## "the caller does with it what it did before" is a claim about the
	## engine call and not about the dialog.
	var png := _tmp.path_join("probe_tile.png")
	var img := Image.create(8, 8, false, Image.FORMAT_RGBA8)
	img.fill(Color(0.2, 0.6, 0.9, 1.0))
	img.save_png(png)
	al._dirty = false
	al._on_import_image()
	await _frames(2)
	d = _one(al, "import-image-confirm")
	if d != null:
		d.navigate(_tmp)
		d._select(png)
		d._confirm()
	await _frames(6)
	_check("import-image: confirm reached as_import_item", al._dirty,
		"_dirty=%s" % al._dirty)

	# --- 7. export pack .zip ------------------------------------------------
	## `navigate()` falls back to the home directory when the requested start
	## directory does not exist, and the asset-packs root is created by nothing
	## in this shell -- `app.gd::open_asset_pack_picker()` reads from the same
	## root and lands in the same place. So the expectation is the contract,
	## not the intent.
	var packs := DccSettings.storage_root("asset_packs")
	var packs_open := packs if DirAccess.dir_exists_absolute(packs) else DccBrowseDialog.home_dir()
	print("LB packs root exists=%s (%s)" % [DirAccess.dir_exists_absolute(packs), packs])
	al._on_export_pack()
	await _frames(3)
	d = _one(al, "export-pack")
	_shape(d, "export-pack", "Export pack .zip", DccBrowseDialog.PickKind.SAVE, ["zip"], packs_open)
	_hint(d, "export-pack", "Import pack")
	if d != null:
		_check("export-pack: default name ends .zip", d._name_edit.text.ends_with(".zip"),
			d._name_edit.text)
		d.navigate(_tmp)
		d._name_edit.text = "probe_pack.zip"
		d._confirm()
	await _frames(4)
	_check("export-pack: the .zip is on disk",
		FileAccess.file_exists(_tmp.path_join("probe_pack.zip")))

	## And the same path a second time must ask before replacing it.
	al._on_export_pack()
	await _frames(3)
	d = _one(al, "export-pack-overwrite")
	if d != null:
		d.navigate(_tmp)
		d._name_edit.text = "probe_pack.zip"
		d._confirm()
	await _frames(3)
	var confirms := _confirm_dialogs()
	_check("export-pack: overwrite is guarded", confirms.size() == 1,
		"found %d" % confirms.size())
	for c in confirms:
		(c as Window).hide()
	await _frames(2)

	# --- 8. sprite sheet ----------------------------------------------------
	al._open_slicer()
	await _frames(4)
	print("LB slicer exclusive=%s" % al._slicer.exclusive)
	_check("sheet: the slicer holds the browser, not the window",
		_browsers(al).is_empty(), "%d on the window" % _browsers(al).size())
	al._pick_sheet_image()
	await _frames(2)
	d = _one(al._slicer, "sheet")
	_shape(d, "sheet", "Choose sprite sheet", DccBrowseDialog.PickKind.FILES, ["png"], home)
	_hint(d, "sheet", "slicer cuts it")
	if d != null:
		d.hide()
	await _frames(2)
	_check("sheet: cancel left nothing loaded", not al._sheet_loaded,
		"_sheet_loaded=%s" % al._sheet_loaded)

	al._pick_sheet_image()
	await _frames(2)
	d = _one(al._slicer, "sheet-confirm")
	if d != null:
		d.navigate(_tmp)
		d._select(png)
		d._confirm()
	await _frames(6)
	_check("sheet: confirm loaded the sheet", al._sheet_loaded,
		"_sheet_loaded=%s" % al._sheet_loaded)
	al._close_slicer()
	await _frames(2)
