extends Node
## Verification harness for `rdMode4()` **rule 7** -- the Way/Route draft
## section (`05-right-dock-and-bars.md` §1.14, `right_dock.gd`'s `TOOL_WAY`),
## the last rung of the right-dock ladder that had no surface in this shell.
##
## Written against two hazards this exact work has already produced once each,
## both found by verifiers rather than by the code's own author:
##
## 1. **Arming a different tool must not take a live draft's controls away.**
##    `_tool_section()` answers with ONE id, so a new arm can displace a draft
##    that is still uncommitted. Every transition INTO `TOOL_WAY` is exercised
##    here, not only the disarm out of it -- including arming Way while a
##    sculpt draft is live, which must leave the Stamp stack's Commit/Discard
##    exactly where they were.
## 2. **A probe here was once green while pinning the rejected design.** So
##    nothing below is an unconditional `_check(..., true, ...)`: every claim
##    reads the dock body back, and the one numeric claim (`Grade · max`) is
##    compared against a figure this probe computes for itself out of
##    `sample_cell()` -- if that row were the reference's hardcoded `4.2%`
##    literal, or any other constant, that comparison fails.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _rdway_probe.tscn

var app: Node
var _fail := 0
var _sel_name := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	var tag := "ok  " if cond else "FAIL"
	print("RDW %s  %s%s" % [tag, name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

# -- Reading the dock back, never the state that fed it -----------------------

func _collect(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append((c as Label).text)
		_collect(c, out)

func _texts() -> Array:
	var out: Array = []
	_collect(app.right_dock_body, out)
	return out

## Section headers in draw order -- `DccWidgets.section()` renders
## `"§ " + title.to_upper()`, so this strips the sigil back off.
func _headers() -> Array:
	var out: Array = []
	for t in _texts():
		var s := String(t)
		if s.begins_with("§ "):
			out.append(s.substr(2))
	return out

## Position of the first section titled exactly `title`, or -1. A position and
## not a bool, because "appends" is a claim about ORDER.
func _hdr(title: String) -> int:
	var hs := _headers()
	for i in hs.size():
		if String(hs[i]) == title.to_upper():
			return i
	return -1

## The reading drawn beside `key`, or "" when that row is not on screen.
##
## By **adjacency in draw order**, not by container type, because this dock
## draws its label/value pairs two different ways -- `_field()` puts them in an
## `HBoxContainer`, `_accent_readout()` in a `VBoxContainer` -- and a reader that
## knew only one of them reported an absent row for a row that was right there.
## (It did: the first run of this probe called `Waypoints` missing while the
## dump in the same line showed `"Waypoints", "2"`.)
func _value(key: String) -> String:
	var t := _texts()
	for i in t.size():
		if String(t[i]) == key and i + 1 < t.size():
			return String(t[i + 1])
	return ""

## Every `BaseButton` caption in the dock body. `Label`s alone are not the dock:
## `DccWidgets.action()` builds a `Button`, so the Stamp stack's Commit and
## Discard -- the controls hazard 1 is about -- are invisible to `_texts()`.
func _buttons(n: Node = null, out: Array = []) -> Array:
	var root: Node = app.right_dock_body if n == null else n
	for c in root.get_children():
		if c is BaseButton and "text" in c:
			out.append(String(c.text))
		_buttons(c, out)
	return out

func _has_button(fragment: String) -> bool:
	for b in _buttons():
		if String(b).find(fragment) >= 0:
			return true
	return false

func _title() -> String:
	return String(app.right_dock_ctrl._current_title())

func _readout() -> String:
	return String(app.right_dock_ctrl._dock_readout_text())

func _name_shown() -> bool:
	return _texts().has(_sel_name)

## The steepest straight-line segment of `pts`, computed here from the same
## public bridge calls `right_dock.gd` uses -- independently, so the dock's own
## row has something real to be wrong against.
func _expected_grade(pts: PackedVector2Array) -> float:
	var gw: int = app.bridge.grid_size().x
	var m_per_cell: float = app.bridge.last_width_km * 1000.0 / float(gw)
	var worst := -1.0
	for i in range(1, pts.size()):
		var a: Dictionary = app.bridge.sample_cell(roundi(pts[i - 1].x), roundi(pts[i - 1].y))
		var b: Dictionary = app.bridge.sample_cell(roundi(pts[i].x), roundi(pts[i].y))
		if not a.has("elevation_m") or not b.has("elevation_m"):
			return -1.0
		var run: float = pts[i - 1].distance_to(pts[i]) * m_per_cell
		if run <= 0.0:
			continue
		worst = maxf(worst, absf(float(b["elevation_m"]) - float(a["elevation_m"])) / run * 100.0)
	return worst

func _click(gx: float, gy: float) -> void:
	app._on_map_clicked(gx, gy)
	await _frames(3)

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
	print("RDW world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("RDW  !! generate failed -- nothing else here can run")
		get_tree().quit(1)
		return

	var rd = app.right_dock_ctrl
	var g: Vector2i = app.bridge.grid_size()
	var settlements: Array = app.bridge.settlements()
	if settlements.is_empty():
		print("RDW  !! no settlement to select -- 'selection wins' needs one")
		get_tree().quit(1)
		return
	_sel_name = String((settlements[0] as Dictionary).get("name", ""))
	_check("the settlement has a name to look for", _sel_name != "", "name='%s'" % _sel_name)

	app.select_domain("civilization")
	await _frames(4)
	rd.on_settlement_selected(settlements[0], 0)
	await _frames(4)
	_check("after select: the dock shows the settlement",
		_title() == "Settlement" and _name_shown(),
		"title=%s name=%s" % [_title(), _name_shown()])

	# -- Rule 7 is gated on the DRAFT, not merely on the tool -------------------
	app.arm_tool("way")
	await _frames(4)
	_check("arming Way with an empty draft appends nothing (rule 7's own draft.length > 0)",
		_hdr("WAY DRAFT") < 0, "headers=%s" % [_headers()])
	_check("...and the selection is untouched by that arm",
		_title() == "Settlement" and _name_shown(),
		"title=%s headers=%s" % [_title(), _headers()])

	# -- Two waypoints, through the real click path ----------------------------
	var p0 := Vector2(g.x * 0.30, g.y * 0.45)
	var p1 := Vector2(g.x * 0.55, g.y * 0.60)
	await _click(p0.x, p0.y)
	await _click(p1.x, p1.y)
	var way_i := _hdr("WAY DRAFT")
	var settle_i := _hdr("SETTLEMENT")
	_check("a non-empty draft appends the Way section", way_i >= 0,
		"headers=%s" % [_headers()])
	_check("it is APPENDED -- below the selection, not instead of it",
		settle_i >= 0 and way_i > settle_i,
		"settlement@%d way@%d" % [settle_i, way_i])
	_check("the dock title still names the selection, not the tool",
		_title() == "Settlement", "title=%s" % _title())
	_check("_context is untouched by the draft", rd._context == "settlement",
		"_context=%s" % rd._context)
	_check("the collapsed readout still reports the selection",
		_readout() == _sel_name, "readout=%s" % _readout())

	# -- §1.14's four rows, read off the dock ----------------------------------
	var draft: PackedVector2Array = rd._way_draft
	_check("Waypoints reports the count the engine actually took",
		_value("Waypoints") == str(draft.size()) and draft.size() >= 1,
		"row='%s' draft=%d" % [_value("Waypoints"), draft.size()])
	_check("Stops is NOT the label used for a way",
		_value("Stops") == "", "row='%s'" % _value("Stops"))
	_check("Length is a real reading, not a dash",
		_value("Length") != "" and _value("Length") != "—",
		"row='%s'" % _value("Length"))
	_check("Surface names the armed way type",
		_value("Surface") == "Road", "row='%s'" % _value("Surface"))

	## The one numeric claim, against an independently computed figure. A
	## hardcoded row -- the reference's own `4.2%` included -- fails this.
	var want := _expected_grade(draft)
	var got_s := _value("Grade · max")
	var got := 0.0
	if want < 0.0:
		_check("Grade · max dashes when no elevation is readable", got_s == "—",
			"row='%s'" % got_s)
	else:
		got = float(got_s.trim_suffix("%"))
		_check("Grade · max equals the grade computed independently here",
			got_s.ends_with("%") and absf(got - want) < 0.05,
			"row='%s' independent=%.4f%%" % [got_s, want])
		## A second check lived here and was REMOVED, not rewritten, after a
		## verifier refuted it: `absf(want - 4.2) > 0.001 or got_s != "4.2%"`.
		## Its first disjunct is unconditionally true for any fixture whose real
		## grade is not ~4.2% (this one measures ~0.19%), so it could never fail.
		##
		## Nothing replaces it, deliberately. The claim it was reaching for --
		## that the row is computed rather than a constant -- is already carried
		## by the check immediately above, which compares the dock against an
		## independently computed figure, and that check is falsifiable: mutating
		## `_way_max_grade()`'s km->m constant turns it red. A runtime assertion
		## cannot add to that, and a second green check that pins nothing is
		## worse than no check, because it reads as coverage.
	## Renamed after a verifier refuted the name: this asserts the readout shows
	## the SELECTION, which is the opposite of what the old name claimed, and it
	## duplicated the check six lines above. The draft-fallback claim is tested
	## for real in block (f), with no selection, where it reads "1 waypoint".
	_check("the readout shows the selection while one is selected",
		_readout() == _sel_name, "readout=%s" % _readout())

	## The **two printed rows against each other**, which is a different claim
	## from the one above: the `Grade · max` on screen has to be the rise over
	## the run the `Length` on screen reports. Length reaches the dock through
	## `_route_length_text()` and the grade through `_way_max_grade()` -- two
	## code paths over one geometry, so a defect in either shows up here.
	## Absolute tolerance, because the dock prints the grade to 0.1% and the
	## length to the whole kilometre.
	if draft.size() == 2 and want >= 0.0:
		var km := float(String(_value("Length")).trim_suffix(" km"))
		var a0: Dictionary = app.bridge.sample_cell(roundi(draft[0].x), roundi(draft[0].y))
		var b0: Dictionary = app.bridge.sample_cell(roundi(draft[1].x), roundi(draft[1].y))
		var via_len := absf(float(b0["elevation_m"]) - float(a0["elevation_m"])) / (km * 1000.0) * 100.0
		_check("the printed Grade · max is the rise over the printed Length's own run",
			km > 0.0 and absf(via_len - got) < 0.08,
			"from-Length=%.4f%% printed-grade=%s (length=%.0f km)" % [via_len, got_s, km])

	## `MISTAKES.md`, "read a layout that overflows the screen": this dock's
	## `ScrollContainer` has horizontal scrolling disabled, so any row wider than
	## the dock raises the dock's own minimum width and takes those pixels off
	## the viewport -- silently, with no scrollbar to reveal it. Measured with
	## the new section actually on screen, not reasoned about.
	var minw: float = app.right_dock_body.get_combined_minimum_size().x
	_check("the Way section does not widen the dock (disabled scroll axis folds child width upward)",
		minw <= app.right_dock_body.size.x + 1.0,
		"body min=%.0f actual=%.0f headers=%s" % [minw, app.right_dock_body.size.x, _headers()])

	# -- Hazard 1: every transition INTO the new state -------------------------
	#
	# (a) Way armed over a LIVE SCULPT DRAFT. `_append_tool()` draws one section
	# per `_tool_section()` answer, so the draft clause has to survive an arm
	# that beats it -- the exact defect rule 1's own conversion shipped.
	app.select_domain("world")
	await _frames(4)
	app.arm_tool("sculpt")
	await _frames(3)
	app.bridge.sculpt_begin_stroke()
	app.bridge.sculpt_add_point(g.x * 0.2, g.y * 0.2)
	app.bridge.sculpt_add_point(g.x * 0.25, g.y * 0.25)
	app.bridge.sculpt_end_stroke()
	rd.show_sculpt_stack()
	await _frames(4)
	_check("a sculpt draft is live before the Way arm",
		app.bridge.sculpt_stamp_count() > 0 and _hdr("STAMP STACK") >= 0,
		"stamps=%d headers=%s" % [app.bridge.sculpt_stamp_count(), _headers()])
	app.arm_tool("way")
	await _frames(3)
	await _click(g.x * 0.62, g.y * 0.30)
	await _click(g.x * 0.70, g.y * 0.38)
	_check("arming Way over an uncommitted sculpt draft KEEPS the Stamp stack",
		_hdr("STAMP STACK") >= 0, "headers=%s" % [_headers()])
	_check("...and the Way section is there too, both at once",
		_hdr("WAY DRAFT") >= 0, "headers=%s" % [_headers()])
	_check("the sculpt draft's own Commit/Discard are still on screen under Way",
		_has_button("Commit to map") and _has_button("Discard draft"),
		"buttons=%s" % [_buttons()])
	app.bridge.sculpt_discard()
	rd.show_sculpt_stack()
	await _frames(3)

	# (b) Way -> Route. The way auto-commits (`_on_infra_tool_armed`), so the
	# WAY section must go with it and must NOT be redrawn under the Route tool
	# from the points it still remembers.
	app.select_domain("civilization")
	await _frames(4)
	_check("the way draft survived the domain round-trip", _hdr("WAY DRAFT") >= 0,
		"headers=%s" % [_headers()])
	app.arm_tool("route")
	await _frames(4)
	_check("arming Route commits the way and drops its section",
		_hdr("WAY DRAFT") < 0, "armed=%s headers=%s" % [app.armed_tool, _headers()])
	_check("Route with no stops yet appends nothing either", _hdr("ROUTE DRAFT") < 0,
		"headers=%s" % [_headers()])
	await _click(g.x * 0.35, g.y * 0.35)
	await _click(g.x * 0.45, g.y * 0.55)
	_check("two stops append the Route section, under its own title",
		_hdr("ROUTE DRAFT") >= 0, "headers=%s" % [_headers()])
	_check("a route draft counts Stops, not Waypoints",
		_value("Stops") != "" and _value("Waypoints") == "",
		"stops='%s' waypoints='%s'" % [_value("Stops"), _value("Waypoints")])
	_check("a route draft omits Surface -- it has no type to report",
		_value("Surface") == "", "row='%s'" % _value("Surface"))
	_check("a route draft does not take the collapsed readout off the selection",
		_readout() == _sel_name, "readout=%s" % _readout())

	# (c) The stale-owner guard, directly. A remembered draft whose tool is not
	# the armed one must SUPPRESS the section, never draw one.
	var keep_owner: String = rd._way_owner
	var keep_draft: PackedVector2Array = rd._way_draft
	rd._way_owner = "way"
	rd._way_draft = PackedVector2Array([Vector2(1, 1), Vector2(9, 9)])
	rd._rebuild()
	await _frames(2)
	_check("a Way-owned draft draws nothing while Route is the armed tool",
		app.armed_tool == "route" and _hdr("WAY DRAFT") < 0 and _hdr("ROUTE DRAFT") < 0,
		"armed=%s headers=%s" % [app.armed_tool, _headers()])
	rd._way_owner = keep_owner
	rd._way_draft = keep_draft
	rd._rebuild()
	await _frames(2)
	_check("restoring the real draft brings the Route section back",
		_hdr("ROUTE DRAFT") >= 0, "headers=%s" % [_headers()])

	# (c2) The title collision the reference cannot have: a committed route
	# SELECTED while the Route tool draws a draft. Both sections are on screen
	# at once under the ruling, so their headers have to differ.
	var roads: Array = app.bridge.roads()
	if roads.is_empty():
		_check("a committed way exists to select for the title-collision case", false,
			"roads=0 -- the earlier commit produced nothing to select")
	else:
		rd.show_route(roads[0], "road")
		await _frames(4)
		_check("the selected Route and the Route draft do not share a header",
			_hdr("ROUTE") >= 0 and _hdr("ROUTE DRAFT") >= 0 and _hdr("ROUTE") != _hdr("ROUTE DRAFT"),
			"headers=%s" % [_headers()])
		_check("...and the draft is the appended one, below the selection",
			_hdr("ROUTE DRAFT") > _hdr("ROUTE"),
			"route@%d draft@%d" % [_hdr("ROUTE"), _hdr("ROUTE DRAFT")])
		rd.on_settlement_selected(app.bridge.settlements()[0], 0)
		await _frames(4)

	_check("the new section id got no CTX_TITLES row (the warning the TOOL_* block carries)",
		not rd.CTX_TITLES.has(rd.TOOL_WAY) and rd._context != rd.TOOL_WAY,
		"keys=%s _context=%s" % [rd.CTX_TITLES.keys(), rd._context])

	# (d) Route -> Territory, a tool from another file entirely.
	app.arm_tool("territory")
	await _frames(4)
	_check("arming Territory commits the route and drops its section",
		_hdr("ROUTE DRAFT") < 0, "headers=%s" % [_headers()])
	_check("...and Territory's own section is what appended instead",
		_hdr("TERRITORY") >= 0, "headers=%s" % [_headers()])
	_check("...with the selection still intact through all of it",
		_title() == "Settlement" and _name_shown(),
		"title=%s name=%s" % [_title(), _name_shown()])

	# (e) Disarm to Inspect.
	app.arm_tool("way")
	await _frames(3)
	await _click(g.x * 0.5, g.y * 0.5)
	_check("Way appends again after everything above", _hdr("WAY DRAFT") >= 0,
		"headers=%s" % [_headers()])
	app.arm_tool("inspect")
	await _frames(4)
	_check("disarming to Inspect drops the Way section", _hdr("WAY DRAFT") < 0,
		"headers=%s" % [_headers()])
	_check("...and leaves the selection exactly as it was",
		_title() == "Settlement" and _name_shown(),
		"title=%s name=%s" % [_title(), _name_shown()])

	# (f) With nothing selected, the section still appends and owns the readout.
	rd.on_settlement_selected(null, -1)
	await _frames(3)
	app.arm_tool("way")
	await _frames(3)
	await _click(g.x * 0.28, g.y * 0.62)
	_check("Way appends with no selection to append to",
		_hdr("WAY DRAFT") >= 0 and _title() == "Sample",
		"title=%s headers=%s" % [_title(), _headers()])
	_check("and only then does the collapsed readout report the draft",
		_readout().find("waypoint") >= 0, "readout=%s" % _readout())
	app.arm_tool("inspect")
	await _frames(3)

	# (g) A regenerate must not leave a draft measured over ground it never saw.
	app.arm_tool("way")
	await _frames(3)
	await _click(g.x * 0.4, g.y * 0.4)
	await _click(g.x * 0.6, g.y * 0.6)
	_check("a draft is live before the regenerate", _hdr("WAY DRAFT") >= 0,
		"headers=%s" % [_headers()])
	app._run_pipeline()
	waited = 0
	while app.bridge.generating and waited < 1800:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	_check("a regenerate forgets the draft rather than re-measuring it",
		rd._way_draft.is_empty() and _hdr("WAY DRAFT") < 0,
		"remembered=%d headers=%s" % [rd._way_draft.size(), _headers()])

	print("RDW ---- %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
