extends Node
## Committed verification harness for GUI replacement stage 5's four missing
## right-dock tool sections (`05-right-dock-and-bars.md` §1.8/§1.9/§1.10/§1.12):
## PAINT, RAMP · STOPS, ANNOTATION, TERRITORY (`right_dock.gd`'s `TOOL_PAINT`/
## `TOOL_STOPS`/`TOOL_ANNO`/`TOOL_TERR`). Drives the *real shell* through
## `app.arm_tool`/`app.select_domain` and the same bridge calls the map's own
## pointer handlers make -- `_iconhandle_probe.gd`'s own pattern, reused
## rather than reinvented.
##
## **Six of these checks were rewritten on 2026-09-03** and the reason is worth
## keeping: they asserted `rd._context == "paint"` / `"territory"` / `"anno"` /
## `"stops"`, which is the shape the owner's ruling that day rejected
## (`LARGE_ITEM_RULINGS.md`: *"Selection wins; the tool appends a section."*).
## A green probe pinning the rejected design is worse than no probe. They now
## read `rd._tool_section()`, and whether the selection survives is
## `_rdappend_probe.gd`'s subject, which reads the dock body itself.
##
## Checks, per section: the wiring reaches it (arming the tool / switching
## domain actually appends it, not just that calling `show_*` directly would),
## the section is really on screen (a header scan -- `get_child_count() > 0`
## became true-by-construction the moment Sample was always drawn underneath),
## and the live numbers this stage's own header brief cared
## about (painted-cell counts, ramp stop count, label/icon counts, claimed
## territory cells) are real, not placeholders. Godot's own engine prints any
## script error (a bad method name, an argument mismatch, a null deref) to
## stderr on its own -- this probe does not try to catch those itself, and is
## graded on the absence of any such line the same way every `--check-only`
## pass in this repo already is.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _rightdock5_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence
## for the passes that wrote them, not deleted after them.

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	var tag := "ok  " if cond else "FAIL"
	print("RD5 %s  %s%s" % [tag, name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

## Every `DccWidgets.section()` title in the right dock, in draw order --
## `DccTheme.header()` renders `"§ " + title.to_upper()`.
func _collect(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			var s := (c as Label).text
			if s.begins_with("§ "):
				out.append(s.substr(2))
		_collect(c, out)

func _headers() -> Array:
	var out: Array = []
	_collect(app.right_dock_body, out)
	return out

func _has_section(prefix: String) -> bool:
	for h in _headers():
		if String(h).begins_with(prefix.to_upper()):
			return true
	return false

## `_paint_apply_dab` (`world_workspace.gd`) rebuilds the brush from its own
## `_paint_brush` dict -- `land_only: true` by default -- before every
## stroke, so a dab this probe drives through the real click path is
## genuinely gated on land the same way a player's would be, and the map's
## own centre is not reliably land (two different random seeds both put
## ocean there in earlier runs of this same probe). A placed settlement is
## guaranteed to be on land by construction (`civ_place_pick_radius`), so
## its own position is a land cell with no per-cell search needed.
func _find_land_cell() -> Vector2:
	var list: Array = app.bridge.settlements()
	if list.is_empty():
		return Vector2(-1, -1)
	var s: Dictionary = list[0]
	return Vector2(float(s.get("x", -1.0)), float(s.get("y", -1.0)))

func _ready() -> void:
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
	print("RD5 world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("RD5  !! generate failed -- nothing else here can run")
		get_tree().quit(1)
		return

	var rd = app.right_dock_ctrl

	# -- PAINT (§1.8) -----------------------------------------------------------
	app.arm_tool("paint")
	await _frames(4)
	_check("arming paint -> the Paint section is appended", rd._tool_section() == "paint",
		"got '%s' (armed=%s)" % [rd._tool_section(), app.armed_tool])
	_check("the Paint section is really on screen", _has_section("PAINT"),
		"headers=%s" % [_headers()])
	# One real dab so the legend/counts have something to report, exactly the
	# gesture `world_workspace._paint_click` makes.
	var gw: int = app.bridge.grid_size().x
	var gh: int = app.bridge.grid_size().y
	var land := _find_land_cell()
	print("RD5 info  land cell for the paint dab: %s" % land)
	if land.x >= 0.0:
		app._on_map_clicked(land.x, land.y)
		await _frames(3)
	var counts: Dictionary = app.bridge.paint_painted_counts()
	_check("a paint dab on land is reflected in paint_painted_counts()", int(counts.get("total", 0)) > 0,
		"total=%s (land cell found=%s)" % [counts.get("total", 0), land.x >= 0.0])
	app.arm_tool("inspect")
	await _frames(2)
	## **This check was an unconditional `true` recording the opposite**:
	## "leaving paint (domain unchanged) does not itself clear the context".
	## It was accurate then -- nothing called `leave_paint_context()` except a
	## domain switch -- and it could not fail, so it never said so out loud.
	## The section is now derived from `app.armed_tool` (`_tool_section()`), so
	## disarming inside WORLD drops it, and this asserts that instead.
	_check("leaving paint inside the same domain drops the Paint section",
		rd._tool_section() == "" and not _has_section("PAINT"),
		"section='%s' headers=%s" % [rd._tool_section(), _headers()])

	# -- TERRITORY (§1.12) -------------------------------------------------------
	var factions: Array = app.bridge.get_factions()
	_check("world has at least one faction to test Territory with", not factions.is_empty(),
		"count=%d" % factions.size())
	if not factions.is_empty():
		var fid := int((factions[0] as Dictionary).get("id", 1))
		app.arm_tool("territory")
		await _frames(4)
		_check("arming territory -> the Territory section is appended",
			rd._tool_section() == "territory",
			"got '%s' (armed=%s)" % [rd._tool_section(), app.armed_tool])
		# Paint and commit a real claim -- `civilization_workspace._territory_drag`
		# / `_commit_territory`'s own calls, driven directly since the drag
		# gesture itself is not this probe's subject.
		app.bridge.civ_territory_paint_at(gw * 0.5, gh * 0.5, fid, 8.0, false)
		app.bridge.civ_territory_commit()
		rd.show_territory(fid)
		await _frames(3)
		var stats: Dictionary = app.bridge.civ_faction_territory_stats(fid)
		_check("a committed claim shows up in civ_faction_territory_stats()",
			int(stats.get("claimed_cells", 0)) > 0, "claimed_cells=%s" % stats.get("claimed_cells", 0))
		_check("the Territory section is really on screen", _has_section("TERRITORY"),
			"headers=%s" % [_headers()])
		app.arm_tool("inspect")
		await _frames(2)

	# -- ANNOTATION (§1.10) ------------------------------------------------------
	app.select_domain("cartography")
	await _frames(3)
	var lidx: int = app.bridge.label_create(gw * 0.4, gh * 0.4, "Rightdock5 Region")
	_check("label_create placed a real label", lidx >= 0, "idx=%d" % lidx)
	app.bridge.label_select(lidx)
	app.arm_tool("label")
	await _frames(4)
	_check("arming label -> the Annotation section is appended", rd._tool_section() == "anno",
		"got '%s' (armed=%s)" % [rd._tool_section(), app.armed_tool])
	_check("the Annotation section is really on screen", _has_section("ANNOTATION"),
		"headers=%s" % [_headers()])
	_check("label_list() reflects the placed label", app.bridge.label_list().size() >= 1,
		"count=%d" % app.bridge.label_list().size())
	app.arm_tool("inspect")
	await _frames(2)
	_check("leaving label drops the Annotation section",
		rd._tool_section() != "anno" and not _has_section("ANNOTATION"),
		"section='%s' headers=%s" % [rd._tool_section(), _headers()])

	# -- RAMP · STOPS (§1.9) ------------------------------------------------------
	app.arm_tool("inspect")
	await _frames(4)
	_check("CARTO + inspect -> the Ramp · stops section is appended", rd._tool_section() == "stops",
		"got '%s' (ramp_api=%s)" % [rd._tool_section(), app.bridge.ramp_api])
	_check("the Ramp · stops section is really on screen", _has_section("RAMP"),
		"headers=%s" % [_headers()])
	var ramp: Array = app.bridge.color_ramp()
	_check("color_ramp() has stops to show", ramp.size() >= 2, "size=%d" % ramp.size())
	# Add a stop through this dock's own handler, exactly the "+ add" button's
	# own call, and confirm it is both selected and reflected server-side.
	var before_n := ramp.size()
	rd._on_stops_add()
	await _frames(2)
	var after: Array = app.bridge.color_ramp()
	_check("dock's own + add grew the ramp by one stop", after.size() == before_n + 1,
		"before=%d after=%d" % [before_n, after.size()])
	_check("+ add selected the new stop", rd._stops_selected >= 0 and rd._stops_selected < after.size(),
		"selected=%d" % rd._stops_selected)
	app.select_domain("world")
	await _frames(3)
	_check("leaving CARTO drops the Ramp · stops section",
		rd._tool_section() != "stops" and not _has_section("RAMP"),
		"section='%s' headers=%s" % [rd._tool_section(), _headers()])

	print("RD5 fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
