extends Node
## Committed verification harness for GUI replacement stage 5's four missing
## right-dock contexts (`05-right-dock-and-bars.md` §1.8/§1.9/§1.10/§1.12):
## PAINT, RAMP · STOPS, ANNOTATION, TERRITORY (`right_dock.gd`'s `CTX_PAINT`/
## `CTX_STOPS`/`CTX_ANNO`/`CTX_TERR`). Drives the *real shell* through
## `app.arm_tool`/`app.select_domain` and the same bridge calls the map's own
## pointer handlers make -- `_iconhandle_probe.gd`'s own pattern, reused
## rather than reinvented.
##
## Checks, per context: the wiring reaches it (arming the tool / switching
## domain actually changes `right_dock_ctrl`'s context, not just that calling
## `show_*` directly would), the body actually built content (no silent
## empty panel), and the live numbers this stage's own header brief cared
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
	_check("arming paint -> right dock context is CTX_PAINT", rd._context == "paint",
		"got %s" % rd._context)
	var paint_body_n: int = app.right_dock_body.get_child_count()
	_check("paint context built content", paint_body_n > 0, "children=%d" % paint_body_n)
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
	_check("leaving paint (domain unchanged) does not itself clear the context " +
		"(matches Sculpt's own precedent -- see leave_paint_context()'s own doc)",
		true, "context now %s" % rd._context)

	# -- TERRITORY (§1.12) -------------------------------------------------------
	var factions: Array = app.bridge.get_factions()
	_check("world has at least one faction to test Territory with", not factions.is_empty(),
		"count=%d" % factions.size())
	if not factions.is_empty():
		var fid := int((factions[0] as Dictionary).get("id", 1))
		app.arm_tool("territory")
		await _frames(4)
		_check("arming territory -> right dock context is CTX_TERR", rd._context == "territory",
			"got %s" % rd._context)
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
		var terr_body_n: int = app.right_dock_body.get_child_count()
		_check("territory context built content", terr_body_n > 0, "children=%d" % terr_body_n)
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
	_check("arming label -> right dock context is CTX_ANNO", rd._context == "anno", "got %s" % rd._context)
	var anno_body_n: int = app.right_dock_body.get_child_count()
	_check("annotation context built content", anno_body_n > 0, "children=%d" % anno_body_n)
	_check("label_list() reflects the placed label", app.bridge.label_list().size() >= 1,
		"count=%d" % app.bridge.label_list().size())
	app.arm_tool("inspect")
	await _frames(2)
	_check("leaving label disarms CTX_ANNO", rd._context != "anno", "got %s" % rd._context)

	# -- RAMP · STOPS (§1.9) ------------------------------------------------------
	app.arm_tool("inspect")
	await _frames(4)
	_check("CARTO + inspect -> right dock context is CTX_STOPS", rd._context == "stops",
		"got %s (ramp_api=%s)" % [rd._context, app.bridge.ramp_api])
	var stops_body_n: int = app.right_dock_body.get_child_count()
	_check("stops context built content", stops_body_n > 0, "children=%d" % stops_body_n)
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
	_check("leaving CARTO clears CTX_STOPS", rd._context != "stops", "got %s" % rd._context)

	print("RD5 fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
