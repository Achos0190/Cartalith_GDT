extends Node
## Lane PC-BROWSE, 2026-09-07 -- DIAGNOSIS, not a fix harness.
##
## The owner reports that Storage locations and Data management "just accept a
## path" and never open a browser. `DccBrowseDialog` exists and both surfaces
## call it, so the defect is reachability or capability, not absence. This
## probe separates the three claims the brief insists on separating:
##
##   RENDERS   -- the control has a non-degenerate drawn rect
##   OPERABLE  -- pressing it at a point ON SCREEN runs its callback
##   FINDABLE  -- that drawn rect is inside every ancestor's VISIBLE rect,
##                so a person looking at the dialog can see it
##
## `get_global_rect()` is unclipped, so FINDABLE is the claim it cannot make
## and the one this probe computes by intersecting up the parent chain.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _pcbrowse_probe.tscn
##
## Windowed on purpose: `AcceptDialog.popup_centered()` clamps against the
## real screen, and the dummy driver's screen is not this machine's.

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(label: String, cond: bool, detail: String = "") -> void:
	print("PCB %s  %s%s" % ["ok  " if cond else "FAIL", label,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

## The rect a viewer can actually see: the control's own rect intersected with
## every ancestor Control's rect **and then with the owning Window's client
## rect**. `get_global_rect()` refuses to make this measurement, and so did the
## first version of this function -- it stopped at the last Control ancestor,
## which is worthless when the ancestors overflow too. Measured 2026-09-07:
## the Data manager's Folder picker sits at x=1291 inside a 1152 px window and
## scored `shown=1.000`, because every container above it is equally oversized.
## That is the `CREATE WORLD at 7 of 46 dp` shape exactly, reproduced inside the
## check written to catch it.
func _visible_rect(c: Control) -> Rect2:
	var r := c.get_global_rect()
	var n: Node = c.get_parent()
	while n != null:
		if n is Control:
			r = r.intersection((n as Control).get_global_rect())
		elif n is Window:
			## Window-local coordinates start at the origin, so the frame is
			## the clip. This is the term that was missing.
			r = r.intersection(Rect2(Vector2.ZERO, Vector2((n as Window).size)))
			break
		if r.size.x <= 0.0 or r.size.y <= 0.0:
			return Rect2(r.position, Vector2.ZERO)
		n = n.get_parent()
	if r.size.x <= 0.0 or r.size.y <= 0.0:
		return Rect2(r.position, Vector2.ZERO)
	return r

## Fraction of the control's own drawn area that survives the clip. 1.0 is
## fully on screen; the `CREATE WORLD` failure this brief cites was 0.15.
func _shown(c: Control) -> float:
	var own := c.get_global_rect()
	if own.size.x <= 0.0 or own.size.y <= 0.0:
		return 0.0
	var v := _visible_rect(c)
	return (v.size.x * v.size.y) / (own.size.x * own.size.y)

func _describe(tag: String, c: Control) -> void:
	var own := c.get_global_rect()
	var vis := _visible_rect(c)
	print("PCB      %-22s own=[%.0f,%.0f %.0fx%.0f] vis=[%.0f,%.0f %.0fx%.0f] shown=%.2f min=%.0fx%.0f"
		% [tag, own.position.x, own.position.y, own.size.x, own.size.y,
			vis.position.x, vis.position.y, vis.size.x, vis.size.y, _shown(c),
			c.get_combined_minimum_size().x, c.get_combined_minimum_size().y])

## Every Button under `root` whose text starts with "Browse".
func _browse_buttons(root: Node, out: Array) -> Array:
	if root is Button and String((root as Button).text).begins_with("Browse"):
		out.append(root)
	for ch in root.get_children():
		_browse_buttons(ch, out)
	return out

func _first_dialog(host: Node, title: String) -> Window:
	for c in host.get_children():
		if c is Window and String((c as Window).title) == title:
			return c
	return null

func _live_browser(host: Node) -> Node:
	for c in host.get_children():
		if c is DccBrowseDialog and (c as Window).visible:
			return c
	return null

## Drive the control the way a person does: a press and a release at a point
## inside its VISIBLE rect, pushed into the Window that owns it. A synthesized
## `pressed.emit()` would bypass exactly the mouse-filter / overlap / clip
## layer this probe exists to test.
## Which delivery route reaches an embedded sub-window's GUI. Established by
## `_calibrate_tap()` below rather than assumed: the first route this probe
## tried (`Window.push_input`, local coords) reached NOTHING, and without a
## positive control that would have been filed as "the shell ignores clicks on
## Browse". Routes, in the order they are tried:
##   0  win.push_input(e, true)          -- window-local
##   1  root.push_input(e, false)        -- root-viewport coords, win.position + local
var _tap_route := -1

func _send(win: Window, at: Vector2, route: int) -> void:
	var target: Viewport = win if route == 0 else get_viewport()
	var pos: Vector2 = at if route == 0 else at + Vector2(win.position)
	var motion := InputEventMouseMotion.new()
	motion.position = pos
	motion.global_position = pos
	target.push_input(motion, route == 0)
	await _frames(1)
	for pressed in [true, false]:
		var e := InputEventMouseButton.new()
		e.button_index = MOUSE_BUTTON_LEFT
		e.pressed = pressed
		e.position = pos
		e.global_position = pos
		target.push_input(e, route == 0)
		await _frames(1)
	await _frames(2)

func _tap(win: Window, c: Control) -> void:
	var v := _visible_rect(c)
	await _send(win, v.position + v.size * 0.5, _tap_route)

## The positive control, run BEFORE anything is concluded from a tap. A press
## that reaches nothing and a harness that delivers nothing look identical from
## the outside, so this establishes a route on a throwaway dialog of the shell's
## own making whose reaction is unambiguous -- the dialog hides.
func _calibrate_tap() -> void:
	for route in [0, 1]:
		var probe := AcceptDialog.new()
		probe.title = "tap calibration"
		probe.size = Vector2i(320, 160)
		add_child(probe)
		probe.popup_centered()
		await _frames(4)
		var ok := probe.get_ok_button()
		await _send(probe, ok.get_global_rect().get_center(), route)
		var worked := not probe.visible
		print("PCB   tap route %d: OK button at %s -> dialog hidden=%s" % [
			route, ok.get_global_rect().get_center(), worked])
		probe.hide()
		probe.queue_free()
		await _frames(2)
		if worked:
			_tap_route = route
			break
	_ok("POSITIVE CONTROL: a synthetic press can operate a dialog at all",
		_tap_route >= 0, "route=%d" % _tap_route)

func _ready() -> void:
	print("PCB screen=%s window=%s" % [DisplayServer.screen_get_size(),
		DisplayServer.window_get_size()])
	## `-- --vp WxH`. The Data manager sizes itself to the viewport, so its
	## overflow is a function of screen width and a single width is a single
	## sample. This machine's screen is 1680x1050 and the project's default
	## window is 1152x648; both are run, because a control that fits one and
	## not the other is a width finding, not a layout finding.
	var args := OS.get_cmdline_user_args()
	for i in args.size():
		if String(args[i]) == "--vp" and i + 1 < args.size():
			var wh := String(args[i + 1]).split("x")
			DisplayServer.window_set_size(Vector2i(int(wh[0]), int(wh[1])))
			await _frames(6)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	print("PCB density touch=%s phone=%s tablet=%s laptop=%s viewport=%s" % [
		DccTheme.is_touch(), DccTheme.is_phone(), DccTheme.is_tablet(),
		DccTheme.is_laptop(), app.get_viewport_rect().size])

	await _calibrate_tap()
	await _storage_locations()
	await _data_manager()
	await _volume_reach()

	print("PCB DONE fails=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

# ---------------------------------------------------------------------------
# READING 1 -- Storage locations
# ---------------------------------------------------------------------------

func _storage_locations() -> void:
	print("PCB --- Storage locations ---")
	app.open_storage_locations()
	await _frames(6)
	var d := _first_dialog(app, "Storage locations")
	if d == null:
		_ok("Storage locations: the dialog exists", false)
		return
	var body := d.get_child(d.get_child_count() - 1) as Control
	print("PCB   dialog size=%s min_size=%s contents_min=%s wrap=%s" % [
		d.size, d.min_size, d.get_contents_minimum_size(), d.wrap_controls])
	_describe("body", body)

	var browses: Array = _browse_buttons(d, [])
	_ok("Storage locations: one Browse per root",
		browses.size() == DccSettings.ROOT_KEYS.size(),
		"%d buttons for %d roots" % [browses.size(), DccSettings.ROOT_KEYS.size()])

	## The whole point: is the button INSIDE the frame, or has a hard-minimum
	## note pushed the row wider than the window that draws it?
	for i in browses.size():
		var b: Button = browses[i]
		var key: String = String(DccSettings.ROOT_KEYS[i]) if i < DccSettings.ROOT_KEYS.size() else str(i)
		_describe("Browse[%s]" % key, b)
		_ok("Browse[%s] is fully drawn inside its containers" % key,
			_shown(b) > 0.999, "shown=%.3f" % _shown(b))
		var own := b.get_global_rect()
		_ok("Browse[%s] is inside the dialog frame" % key,
			own.position.x >= -0.5 and own.position.x + own.size.x <= float(d.size.x) + 0.5,
			"right edge %.0f vs window %d" % [own.position.x + own.size.x, d.size.x])

	## The two hard minimums the brief names, measured rather than read.
	var notes: Array = []
	for c in body.get_children():
		if c is Label and (c as Label).autowrap_mode != TextServer.AUTOWRAP_OFF:
			notes.append(c)
	for n in notes:
		_describe("note(min.x=%.0f)" % (n as Control).custom_minimum_size.x, n)
	var body_min := body.get_combined_minimum_size()
	print("PCB   body min=%.0fx%.0f  window=%dx%d" % [body_min.x, body_min.y, d.size.x, d.size.y])
	_ok("Storage locations: the body's minimum fits the window it was given",
		body_min.x <= float(d.size.x) + 0.5,
		"body_min.x=%.0f window.x=%d" % [body_min.x, d.size.x])
	_ok("Storage locations: the dialog fits the screen",
		d.size.x <= DisplayServer.screen_get_size().x
			and d.size.y <= DisplayServer.screen_get_size().y,
		"%s vs %s" % [d.size, DisplayServer.screen_get_size()])

	## OPERABLE, split two ways so a failure names its own layer.
	##   emit  -- the callback is wired and the browser can be built at all
	##   tap   -- a press at the button's own drawn centre REACHES it
	## They disagree exactly when something invisible is over the control.
	if browses.size() > 0:
		browses[0].pressed.emit()
		await _frames(6)
		var by_emit := _live_browser(app)
		_ok("Browse's callback builds a browser when emitted", by_emit != null)
		if by_emit != null:
			print("PCB   browser cwd=%s size=%s rows=%d" % [
				by_emit._cwd, (by_emit as Window).size, by_emit._list.get_child_count()])
			_ok("The browser lists something to navigate",
				by_emit._list.get_child_count() > 0,
				"%d rows in %s" % [by_emit._list.get_child_count(), by_emit._cwd])
			_ok("The browser starts at a real directory",
				DirAccess.dir_exists_absolute(by_emit._cwd), by_emit._cwd)
			(by_emit as Window).hide()
			await _frames(3)

		if _tap_route >= 0:
			await _tap(d, browses[0])
			var by_tap := _live_browser(app)
			_ok("Tapping Browse where it is drawn opens the browser", by_tap != null)
			if by_tap != null:
				(by_tap as Window).hide()
				await _frames(3)
	d.hide()
	d.queue_free()
	await _frames(3)

# ---------------------------------------------------------------------------
# READING 2 -- Data management
# ---------------------------------------------------------------------------

func _data_manager() -> void:
	print("PCB --- Data management ---")
	var w: Node = app.data_manager_window
	app.open_data_manager()
	await _frames(8)
	## Every route, not the two the brief named. The first pass of this probe
	## searched for a Button whose text begins with "Browse" and reported
	## `export_maps` as having none -- it has one, spelled `Choose…`. Searching
	## for a name can only find what you already guessed the name was, so the
	## inventory below asks "does this well's row carry ANY button", and the
	## route walk is driven from `ROUTES` rather than a hand-written pair.
	for r in w.ROUTES:
		var route: String = String(r["id"])
		w.open_route(route)
		await _frames(8)
		var browses: Array = _browse_buttons(w, [])
		for b in browses:
			_ok("%s: %s is fully drawn" % [route, (b as Button).text],
				_shown(b) > 0.999, "shown=%.3f" % _shown(b))
		## The wells: bordered, mono, and read-only. The owner's words were
		## "it just accepts a path", which is what a well beside no button
		## looks like.
		var wells: Array = []
		_path_wells(w, wells)
		if wells.is_empty():
			continue
		for pair in wells:
			print("PCB   route=%s well=%s  editable=%s  buttons_in_row=[%s] row_min=%.0f" % [
				route, pair[0], pair[1], pair[2], pair[3]])
		_ok("%s: every path well's row carries a picker" % route,
			wells.all(func(p): return String(p[2]) != ""),
			"%d wells, %d with no button beside them" % [wells.size(),
				wells.filter(func(p): return String(p[2]) == "").size()])
		## The two file names this pass hoisted to `TX_DEFAULT_NAME` /
		## `GIS_DEFAULT_NAME`, asserted against the literal a person reads off
		## the well -- not against the constant, which would hold for every
		## value of it and go green under any mutation.
		if route == "export_maps":
			_ok("export_maps' pre-filled destination is named region-tiles.zip",
				String(wells[0][0]).ends_with("/region-tiles.zip"), String(wells[0][0]))
		if route == "export_gis":
			_ok("export_gis' pre-filled destination is named world.geojson",
				String(wells[0][0]).ends_with("/world.geojson"), String(wells[0][0]))
		## MINIMUMS. Adding a control to a row is exactly where an overflow
		## propagates outward, and this window declares `wrap_controls = false`
		## precisely because one oversized child min pushes its footer off the
		## bottom for good. `export_maps` is the control, not a self-assertion:
		## its Destination row has carried a chip in this identical
		## `_row` + `_well_label` + `chip` shape since before this pass.
		print("PCB   route=%s contents_min=%s size=%s min_size=%s" % [
			route, w.get_contents_minimum_size(), (w as Window).size, (w as Window).min_size])
		_ok("%s: the window's contents still fit the size it opens at" % route,
			w.get_contents_minimum_size().x <= float((w as Window).size.x)
				and w.get_contents_minimum_size().y <= float((w as Window).size.y),
			"contents_min=%s size=%s" % [w.get_contents_minimum_size(), (w as Window).size])

	await _reroot(w)
	(w as Window).hide()
	await _frames(2)

## The World Data route's new `Folder` picker, and the cache-coherence rule it
## needs. Nothing here confirms a pick: `_pick_exports_root()` writes a real
## persisted setting, and a probe must not move the owner's exports root.
func _reroot(w: Node) -> void:
	w.open_route("export_world")
	await _frames(8)
	var picker: Button = null
	for b in _browse_buttons(w, []):
		picker = b
	_ok("export_world: the Folder row's picker exists and is drawn",
		picker != null and _shown(picker) > 0.999,
		"shown=%.3f" % (_shown(picker) if picker != null else 0.0))
	if picker != null and _tap_route >= 0:
		await _tap(w as Window, picker)
		print("PCB   tap diag: picker=%s win.pos=%s hovered_in_win=%s hovered_in_root=%s" % [
			picker.get_global_rect(), (w as Window).position,
			(w as Window).gui_get_hovered_control(),
			get_viewport().gui_get_hovered_control()])
		var br := _live_browser(w)
		_ok("export_world: tapping it where it is drawn opens a FOLDERS browser",
			br != null and br._mode == DccBrowseDialog.PickKind.FOLDERS,
			"browser=%s" % (br._cwd if br != null else "none"))
		if br != null:
			(br as Window).hide()   ## cancel; never confirm
			await _frames(3)

	## The two cached destinations, driven directly -- the button above would
	## persist a setting. Both directions, because the rule has two halves.
	var old_root: String = DccSettings.storage_root("exports")
	var moved := "X:/somewhere-else"
	var tx_before: String = w._tx_dest
	var gis_before: String = w._gis_dest
	w._tx_dest = old_root.path_join(w.TX_DEFAULT_NAME)
	w._gis_dest = old_root.path_join(w.GIS_DEFAULT_NAME)
	w._reroot_defaults(old_root, moved)
	_ok("a still-default tiles destination follows the root",
		w._tx_dest == moved.path_join(w.TX_DEFAULT_NAME), w._tx_dest)
	_ok("a still-default GeoJSON destination follows the root",
		w._gis_dest == moved.path_join(w.GIS_DEFAULT_NAME), w._gis_dest)
	## The other half: a destination the user picked is an answer, not a
	## default, and must survive the root moving under it.
	var hand := "Q:/picked-by-hand/mine.zip"
	w._tx_dest = hand
	w._gis_dest = hand
	w._reroot_defaults(old_root, moved)
	_ok("a hand-picked tiles destination is left alone", w._tx_dest == hand, w._tx_dest)
	_ok("a hand-picked GeoJSON destination is left alone", w._gis_dest == hand, w._gis_dest)
	w._tx_dest = tx_before
	w._gis_dest = gis_before
	_ok("the probe left the exports root where it found it",
		DccSettings.storage_root("exports") == old_root,
		DccSettings.storage_root("exports"))

## A "path well" for this reading: a mono Label whose text looks like an
## absolute path, sitting inside a PanelContainer. Reports its text, whether
## anything in the row can edit it, and whether the row carries a Browse.
func _path_wells(root: Node, out: Array) -> void:
	if root is Label and root.get_parent() is PanelContainer:
		## A well is a Label inside a PanelContainer -- the bordered box the
		## canvas draws. A footer note is a bare Label and is not one; an
		## earlier pass counted them and reported two wells per route where
		## there is one.
		var t := String((root as Label).text)
		if (t.contains(":/") or t.contains(":\\") or t.begins_with("/")) and t.length() > 6:
			var row: Node = root
			for i in 4:
				if row.get_parent() == null:
					break
				row = row.get_parent()
				if row is HBoxContainer:
					break
			## ANY button in the row, by its own text -- not a name search.
			## `export_maps` spells its picker `Choose…`, and a probe hunting
			## for "Browse" reported that route as having none.
			var picker: Array[String] = []
			for n in row.get_children():
				if n is Button and String((n as Button).text) != "":
					picker.append(String((n as Button).text))
			var editable := false
			for n in row.get_children():
				if n is LineEdit:
					editable = true
			out.append([t, editable, ", ".join(picker),
				(row as Control).get_combined_minimum_size().x])
	for ch in root.get_children():
		_path_wells(ch, out)

# ---------------------------------------------------------------------------
# READING 3 -- can the browser reach a second volume?
# ---------------------------------------------------------------------------

## The reading the first two eliminated hypotheses point at. On this machine
## every storage root defaults under `C:`, and the owner's own content -- what
## `OS.get_system_dir()` calls DOCUMENTS, DOWNLOADS, PICTURES, MOVIES, MUSIC --
## is on `D:`. So the question is not "does the browser open" (it does, measured
## above) but "starting from where it opens, can a person walk to their files
## WITHOUT typing a path". Every affordance the dialog offers is enumerated:
## the rows, the breadcrumb segments, and the Home button.
func _volume_reach() -> void:
	print("PCB --- Volume reach ---")
	var volumes: Array[String] = []
	for letter in "CDEFGH":
		var root := "%s:/" % letter
		if DirAccess.dir_exists_absolute(root):
			volumes.append(root)
	print("PCB   volumes present: %s" % ", ".join(volumes))
	var elsewhere: Array[String] = []
	for i in 8:
		var sd := OS.get_system_dir(i)
		if sd != "" and not sd.begins_with(volumes[0]) and not elsewhere.has(sd):
			elsewhere.append(sd)
	print("PCB   system dirs off the boot volume: %s" % ", ".join(elsewhere))

	## Walk from the top of the volume the roots default to. If a second
	## volume exists, THIS is where a person is stranded.
	var d = DccBrowseDialog.choose_folder(app, "reach", volumes[0],
		"reach test", Callable())
	await _frames(6)
	## EVERY offer in the whole dialog, not the list and the crumbs. The first
	## version of this check walked `_list` and `_crumb_row` only -- the two
	## places this lane had already read -- and would have reported a FAIL that
	## a concurrent lane's new places strip, a sibling node it never looked at,
	## already answers. Searching the containers you happen to know about is the
	## same error as searching for a name you happen to have guessed.
	var offers: Array[String] = []
	for n in _labels(d, []):
		offers.append(String((n as Label).text))
	for b in _all_buttons(d, []):
		offers.append(String((b as Button).text))
		offers.append(String((b as Button).tooltip_text))
	var crumbs: Array[String] = []
	for c in d._crumb_row.get_children():
		if c is Button:
			crumbs.append(String((c as Button).text))
	print("PCB   at %s: %d list rows, crumbs=[%s], home=%s, offers=%d" % [
		d._cwd, d._list.get_child_count(), ", ".join(crumbs),
		DccBrowseDialog.home_dir(), offers.size()])

	## An affordance reaches another volume only if something on the dialog
	## NAMES a path on it. `D:` never appears inside a `C:` listing, so a name
	## search over every offer is sound here in a way it would not be for a
	## same-volume destination.
	var reachable := ""
	for v in volumes:
		if v == volumes[0]:
			continue
		var stem := v.substr(0, 2)
		for o in offers + [DccBrowseDialog.home_dir()]:
			if String(o).begins_with(stem) and reachable == "":
				reachable = String(o)
	_ok("From the top of %s a second volume is reachable without typing" % volumes[0],
		reachable != "" or volumes.size() < 2,
		("reached by %s" % reachable) if reachable != "" else
			"%d volumes; nothing on the dialog names one but %s" % [volumes.size(), volumes[0]])

	## The positive control for that search: the path well DOES get there, so
	## the capability exists and it is only the pointing-and-clicking that does
	## not. This is what makes the defect "cannot navigate", not "cannot reach".
	if volumes.size() > 1:
		d._on_path_submitted(volumes[1])
		await _frames(4)
		_ok("POSITIVE CONTROL: typing the volume into the path well does reach it",
			d._cwd == volumes[1].trim_suffix("/") or d._cwd == volumes[1],
			"cwd=%s" % d._cwd)
	(d as Window).hide()
	await _frames(2)

func _labels(root: Node, out: Array) -> Array:
	if root is Label:
		out.append(root)
	for c in root.get_children():
		_labels(c, out)
	return out

func _all_buttons(root: Node, out: Array) -> Array:
	if root is Button:
		out.append(root)
	for c in root.get_children():
		_all_buttons(c, out)
	return out
