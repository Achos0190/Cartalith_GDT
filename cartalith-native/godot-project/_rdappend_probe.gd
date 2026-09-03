extends Node
## Committed verification harness for the owner's 2026-09-03 right-dock ruling
## (`LARGE_ITEM_RULINGS.md`, verbatim): *"Selection wins; the tool appends a
## section."*
##
## The defect this exists to catch was measured in a booted app on 2026-09-03,
## after the row had already been signed off as satisfied:
##
##     [A] after select: title=Settlement  has 'VFY_TOWN' label=true
##     [A] after arming Territory: title=Territory  settlement name SURVIVED=false
##
## So the assertions here are deliberately the two halves of that line -- the
## dock **title** and the presence of the selected settlement's own name label
## in the dock body -- read back after arming each of the four tool sections
## (`right_dock.gd`'s `TOOL_PAINT`/`TOOL_STOPS`/`TOOL_ANNO`/`TOOL_TERR`), and
## again after disarming. Reasoning from `_tool_section` alone would prove
## nothing: the question is what is on screen.
##
## **All four tools, not the one that was measured failing.** `_append_tool()`
## has four arms and each is reached by a different workspace's own arming
## path, so each is exercised through `app.arm_tool` / `app.select_domain`
## rather than by calling `show_*` directly -- otherwise this would test the
## setter and not the wiring.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _rdappend_probe.tscn

var app: Node
var _fail := 0
var _sel: Dictionary = {}
var _sel_name := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	var tag := "ok  " if cond else "FAIL"
	print("RDA %s  %s%s" % [tag, name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

# -- Reading the dock back, not the state that fed it -------------------------

func _collect(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append((c as Label).text)
		_collect(c, out)

func _texts() -> Array:
	var out: Array = []
	_collect(app.right_dock_body, out)
	return out

## Section headers in draw order. `DccWidgets.section()` -> `DccTheme.header()`
## renders `"§ " + title.to_upper()`, so this is the sigil stripped back off.
func _headers() -> Array:
	var out: Array = []
	for t in _texts():
		var s := String(t)
		if s.begins_with("§ "):
			out.append(s.substr(2))
	return out

## Position of the first section whose title starts with `prefix` (upper-cased),
## or -1. A position, not a bool, because "appends" is a claim about ORDER.
func _hdr(prefix: String) -> int:
	var hs := _headers()
	var want := prefix.to_upper()
	for i in hs.size():
		if String(hs[i]).begins_with(want):
			return i
	return -1

func _name_shown() -> bool:
	return _texts().has(_sel_name)

func _title() -> String:
	return String(app.right_dock_ctrl._current_title())

func _readout() -> String:
	return String(app.right_dock_ctrl._dock_readout_text())

# -- One tool, over a live selection ------------------------------------------

## Arms `tool_id`, checks the selection survived intact beside the tool's own
## appended section, disarms to `back_to`, checks it is still intact and the
## tool's section is gone.
func _over_selection(tool_id: String, hdr: String, back_to: String) -> void:
	var rd = app.right_dock_ctrl
	app.arm_tool(tool_id)
	await _frames(4)
	var sec := _hdr(hdr)
	var settle := _hdr("SETTLEMENT")
	_check("%s: the selected settlement's name survives the arm" % tool_id,
		_name_shown(), "looking for '%s'; headers=%s" % [_sel_name, _headers()])
	_check("%s: the dock title still names the selection" % tool_id,
		_title() == "Settlement", "title=%s" % _title())
	_check("%s: the tool's own section is on screen" % tool_id, sec >= 0,
		"headers=%s" % [_headers()])
	_check("%s: it is APPENDED -- below the selection, not above it" % tool_id,
		settle >= 0 and sec > settle, "settlement@%d %s@%d" % [settle, hdr, sec])
	_check("%s: _context is untouched by the arm" % tool_id,
		rd._context == "settlement", "_context=%s" % rd._context)
	_check("%s: the collapsed readout still reports the selection" % tool_id,
		_readout() == _sel_name, "readout=%s" % _readout())

	app.arm_tool(back_to)
	await _frames(4)
	_check("%s: disarm leaves the selection intact" % tool_id,
		_name_shown() and _title() == "Settlement",
		"title=%s name=%s" % [_title(), _name_shown()])
	_check("%s: disarm drops the tool's section" % tool_id, _hdr(hdr) < 0,
		"headers=%s" % [_headers()])

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
	print("RDA world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("RDA  !! generate failed -- nothing else here can run")
		get_tree().quit(1)
		return

	var rd = app.right_dock_ctrl
	var settlements: Array = app.bridge.settlements()
	if settlements.is_empty():
		print("RDA  !! no settlement to select -- the whole probe is about one")
		get_tree().quit(1)
		return
	_sel = settlements[0]
	_sel_name = String(_sel.get("name", ""))
	_check("the settlement has a name to look for", _sel_name != "", "name='%s'" % _sel_name)

	# -- The measured failure, in its own shape ---------------------------------
	rd.on_settlement_selected(_sel, 0)
	await _frames(4)
	_check("after select: the dock shows the settlement",
		_title() == "Settlement" and _name_shown(),
		"title=%s name=%s" % [_title(), _name_shown()])

	# -- Each of `_append_tool()`'s four arms, through its real arming path -----
	await _over_selection("paint", "PAINT", "inspect")          # WORLD
	await _over_selection("territory", "TERRITORY", "inspect")   # CIVIL

	## A domain switch, not a disarm. `armed_tool` survives one (nothing in the
	## shell re-arms Inspect on a domain change), so what drops a section here is
	## `_tool_section()`'s own domain condition -- and the two directions must
	## differ, because the code being replaced differed: `app.gd` cleared Paint
	## on leaving WORLD and nothing ever cleared Territory on a domain switch
	## (`rdMode4()` rule 4 is unconditional on the tool).
	app.arm_tool("paint")
	await _frames(3)
	_check("paint is appended in its own domain first", _hdr("PAINT") >= 0,
		"headers=%s" % [_headers()])
	app.select_domain("civilization")
	await _frames(4)
	_check("Paint does NOT follow its still-armed tool out of WORLD",
		_hdr("PAINT") < 0, "armed=%s headers=%s" % [app.armed_tool, _headers()])
	app.select_domain("world")
	app.arm_tool("territory")
	await _frames(4)
	app.select_domain("civilization")
	await _frames(4)
	_check("Territory DOES survive a domain switch (rule 4, and the old behaviour)",
		_hdr("TERRITORY") >= 0, "armed=%s headers=%s" % [app.armed_tool, _headers()])
	app.select_domain("world")
	app.arm_tool("inspect")
	await _frames(4)
	rd.on_settlement_selected(_sel, 0)
	await _frames(3)

	## CARTO. Stops is armed by the domain switch itself (`rdMode4()` rule 6 is
	## a domain rule, so `app._on_workspace_changed` fires it, not a tool press).
	app.select_domain("cartography")
	await _frames(4)
	_check("a domain switch does not take the selection away either",
		_title() == "Settlement" and _name_shown(),
		"title=%s name=%s headers=%s" % [_title(), _name_shown(), _headers()])
	var stops_i := _hdr("RAMP")
	var settle_i := _hdr("SETTLEMENT")
	_check("CARTO + inspect appends Ramp - stops below the selection",
		stops_i >= 0 and settle_i >= 0 and stops_i > settle_i,
		"settlement@%d ramp@%d headers=%s" % [settle_i, stops_i, _headers()])
	_check("the collapsed readout still reports the selection under Stops",
		_readout() == _sel_name, "readout=%s" % _readout())

	## Label replaces Stops in the one tool slot (§1.10 rule 3 beats rule 6) --
	## which is a tool-over-tool swap, not a tool-over-selection one.
	var lidx: int = app.bridge.label_create(
		app.bridge.grid_size().x * 0.4, app.bridge.grid_size().y * 0.4, "RDAppend Label")
	app.bridge.label_select(lidx)
	app.arm_tool("label")
	await _frames(4)
	_check("arming Label appends Annotation and keeps the selection",
		_hdr("ANNOTATION") > _hdr("SETTLEMENT") and _name_shown() and _title() == "Settlement",
		"title=%s headers=%s" % [_title(), _headers()])
	_check("Label displaces Stops (rule 3 over rule 6), not the selection",
		_hdr("RAMP") < 0, "headers=%s" % [_headers()])
	app.arm_tool("inspect")
	await _frames(4)
	_check("disarming Label restores Stops and still keeps the selection",
		_hdr("ANNOTATION") < 0 and _hdr("RAMP") >= 0 and _name_shown(),
		"headers=%s" % [_headers()])

	# -- A tool armed with NO selection must still show its own section ---------
	rd.on_settlement_selected(null, -1)
	await _frames(4)
	_check("deselecting drops the settlement section", _hdr("SETTLEMENT") < 0,
		"headers=%s" % [_headers()])
	_check("the tool's section survives the DEselect too", _hdr("RAMP") >= 0,
		"headers=%s" % [_headers()])
	_check("with nothing selected the title falls back to Sample",
		_title() == "Sample", "title=%s" % _title())
	_check("with nothing selected the readout reports the armed tool",
		_readout().find("stop") >= 0, "readout=%s" % _readout())

	app.arm_tool("label")
	await _frames(4)
	_check("Annotation appends with no selection to append to",
		_hdr("ANNOTATION") >= 0 and _title() == "Sample",
		"title=%s headers=%s" % [_title(), _headers()])
	_check("Annotation's own collapsed readout survives the ruling",
		_readout().find(" labels · ") >= 0, "readout=%s" % _readout())

	app.select_domain("world")
	app.arm_tool("paint")
	await _frames(4)
	_check("Paint appends with no selection to append to",
		_hdr("PAINT") >= 0 and _title() == "Sample",
		"title=%s headers=%s" % [_title(), _headers()])
	_check("Sample is drawn under it, not replaced by it", _hdr("SAMPLE") >= 0,
		"headers=%s" % [_headers()])
	_check("Paint's own collapsed readout survives the ruling",
		_readout().find("cells") >= 0 or _readout() == "no world", "readout=%s" % _readout())

	# -- The three shapes the ruling rejects, asserted absent -------------------
	for id in ["paint", "stops", "anno", "territory"]:
		_check("no CTX_TITLES row for '%s'" % id, not rd.CTX_TITLES.has(id),
			"CTX_TITLES keys=%s" % [rd.CTX_TITLES.keys()])
	_check("_context is never a tool id", rd._context != "paint" and rd._context != "stops"
		and rd._context != "anno" and rd._context != "territory", "_context=%s" % rd._context)

	print("RDA fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
