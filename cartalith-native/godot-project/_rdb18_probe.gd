extends Node
## Right dock, batch 18: the Stamp stack stops replacing the selection, and the
## Settlement context grows a faith row.
##
## Two claims, both measured on screen rather than reasoned from state:
##
## **A. `rdMode4()` rule 1 obeys the owner's 2026-09-03 ruling** ("selection
## wins, the tool appends a section"). The Stamp stack shipped as `CTX_SCULPT`
## -- a context constant with a `CTX_TITLES` row and a `_dispatch()` arm, which
## is the same triple that was measured taking the dock away from a selected
## settlement when Territory armed. It is now `TOOL_STAMPS`, appended. So the
## assertions below are the two halves of that measured failure line -- the
## dock **title** and the selected settlement's own **name label** in the body
## -- read back after arming Sculpt, and again after every other way the
## section can arrive or leave.
##
## **B. The Settlement context's faith row is honest.** `get_settlements()`
## emits `religion` and `adherents` only once a diffusion has run. The checks
## here are deliberately not restatements of the row's own format string: the
## printed plurality is re-derived from `adherents` independently of the
## `religion` key, and the head-counts are summed against `population`, which
## is the engine's own contract (`lib.rs`: *"head-counts summing to exactly
## `population`, by largest remainder"*).
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _rdb18_probe.tscn

var app: Node
var _fail := 0
var _sel: Dictionary = {}
var _sel_name := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("RDB %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

# -- Reading the dock back ----------------------------------------------------

func _collect(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append((c as Label).text)
		elif c is Button:
			out.append((c as Button).text)
		_collect(c, out)

func _texts() -> Array:
	var out: Array = []
	_collect(app.right_dock_body, out)
	return out

## Section headers in draw order. `DccWidgets.section()` renders the title
## upper-cased behind a sigil, so this is the sigil stripped back off.
func _headers() -> Array:
	var out: Array = []
	for t in _texts():
		var s := String(t)
		if s.begins_with("§ "):
			out.append(s.substr(2))
	return out

## Position of the first section whose title starts with `prefix`, or -1.
## A position and not a bool, because "appends" is a claim about ORDER.
func _hdr(prefix: String) -> int:
	var hs := _headers()
	var want := prefix.to_upper()
	for i in hs.size():
		if String(hs[i]).begins_with(want):
			return i
	return -1

func _name_shown() -> bool:
	return _texts().has(_sel_name)

## The chrome Label the shell actually writes, not the function behind it.
## `DccShell.set_right_dock_title()` upper-cases what it is given
## (`05-right-dock-and-bars.md` §1.1's `letter-spacing:.2em` mono caption), so
## the comparison is on the upper-cased form and the expected strings below say
## so -- reading the Label and then asserting the un-styled string would fail
## for a reason that has nothing to do with which context is showing.
func _title() -> String:
	return String(app.right_dock_title.text).to_upper()

func _readout() -> String:
	return String((app._dock_readouts["right"] as Label).text)

func _has_text(frag: String) -> bool:
	for t in _texts():
		if String(t).find(frag) >= 0:
			return true
	return false

func _text_with(frag: String) -> String:
	for t in _texts():
		if String(t).find(frag) >= 0:
			return String(t)
	return ""

# -- A: the Stamp stack, over a live selection --------------------------------

func _stamps_over_selection() -> void:
	var rd = app.right_dock_ctrl
	var bridge = app.bridge
	app.select_domain("world")
	await _frames(3)
	rd.on_settlement_selected(_sel, 0)
	await _frames(4)
	_check("A0 the dock is showing the settlement before Sculpt arms",
		_title() == "SETTLEMENT" and _name_shown(),
		"title=%s name=%s" % [_title(), _name_shown()])

	app.arm_tool("sculpt")
	await _frames(4)
	var stack := _hdr("STAMP STACK")
	var settle := _hdr("SETTLEMENT")
	_check("A1 arming Sculpt keeps the selected settlement's name on screen",
		_name_shown(), "looking for '%s'; headers=%s" % [_sel_name, _headers()])
	_check("A2 the dock title still names the selection, not the tool",
		_title() == "SETTLEMENT", "title=%s" % _title())
	_check("A3 the Stamp stack is on screen", stack >= 0, "headers=%s" % [_headers()])
	_check("A4 it is APPENDED -- below the selection, not above it",
		settle >= 0 and stack > settle,
		"settlement@%d stack@%d headers=%s" % [settle, stack, _headers()])
	_check("A5 _context is untouched by the arm", rd._context == "settlement",
		"_context=%s" % rd._context)
	_check("A6 the collapsed readout still reports the selection",
		_readout() == _sel_name, "readout=%s" % _readout())

	## The three shapes the ruling rejects, asserted absent for this section too.
	_check("A7 no CTX_TITLES row for the stack", not rd.CTX_TITLES.has("sculpt")
		and not rd.CTX_TITLES.values().has("Stamp stack"),
		"CTX_TITLES=%s" % [rd.CTX_TITLES])
	_check("A8 _context is never the sculpt id", rd._context != "sculpt",
		"_context=%s" % rd._context)

	## Disarm with an EMPTY draft: the tool clause is the only thing holding
	## the section up, so it goes and the selection stays.
	_check("A9 premise: the draft is empty for the disarm check",
		bridge.sculpt_stamp_count() == 0, "stamps=%d" % bridge.sculpt_stamp_count())
	app.arm_tool("inspect")
	await _frames(4)
	_check("A10 disarming drops the stack and keeps the selection",
		_hdr("STAMP STACK") < 0 and _name_shown() and _title() == "SETTLEMENT",
		"headers=%s title=%s" % [_headers(), _title()])

	## The draft clause. A real stroke through the same bridge calls the map
	## uses, then the section must come back with Inspect still armed -- this
	## is the behaviour `_sculpt_escape()` and Edit > Select All depended on
	## when the context flag survived a disarm.
	var made := false
	if bridge.sculpt_begin_stroke():
		for i in 6:
			bridge.sculpt_add_point(140.0 + i * 5.0, 100.0 + i * 3.0)
		made = bridge.sculpt_end_stroke() >= 0
	if not made or bridge.sculpt_stamp_count() == 0:
		print("RDB  note: no stamp could be drawn (sculpt_get_globals empty=%s) -- "
			% bridge.sculpt_get_globals().is_empty()
			+ "the draft clause stays unproven")
		return
	rd.show_sculpt_stack()
	await _frames(4)
	_check("A11 a non-empty draft brings the stack back with Inspect armed",
		_hdr("STAMP STACK") >= 0,
		"armed=%s stamps=%d headers=%s" % [app.armed_tool,
			bridge.sculpt_stamp_count(), _headers()])
	_check("A12 and it is still below the selection, not instead of it",
		_hdr("SETTLEMENT") >= 0 and _hdr("STAMP STACK") > _hdr("SETTLEMENT")
		and _name_shown() and _title() == "SETTLEMENT",
		"headers=%s title=%s" % [_headers(), _title()])

	## The WORLD gate, which is `leave_sculpt_context()`'s own condition.
	app.select_domain("civilization")
	await _frames(4)
	_check("A13 the stack does not follow a live draft out of WORLD",
		_hdr("STAMP STACK") < 0, "headers=%s" % [_headers()])
	_check("A14 and the domain switch did not take the selection either",
		_name_shown() and _title() == "SETTLEMENT",
		"title=%s name=%s" % [_title(), _name_shown()])
	app.select_domain("world")
	await _frames(4)
	_check("A15 coming back to WORLD restores the stack under the selection",
		_hdr("STAMP STACK") > _hdr("SETTLEMENT") and _name_shown(),
		"headers=%s" % [_headers()])

	## §6's "stamp count for the stack" -- the readout only when there is
	## no selection to report instead.
	rd.on_settlement_selected(null, -1)
	await _frames(4)
	var n: int = bridge.sculpt_stamp_count()
	_check("A16 with nothing selected the readout reports the stamp count",
		_readout() == "%d stamp%s" % [n, "" if n == 1 else "s"],
		"readout=%s stamps=%d" % [_readout(), n])
	_check("A17 and the title falls back to Sample, never to Stamp stack",
		_title() == "SAMPLE", "title=%s" % _title())
	bridge.sculpt_discard()
	app.arm_tool("inspect")
	await _frames(3)

# -- B: the faith row ---------------------------------------------------------

## Every religion key the engine will admit, with its own label.
func _vocab() -> Dictionary:
	var out := {}
	for e in app.bridge.civ_religion_vocabulary():
		var d: Dictionary = e
		out[String(d.get("key", ""))] = String(d.get("label", ""))
	return out

func _faith_value() -> String:
	var t := _texts()
	var i := t.find("Faith")
	return "" if i < 0 or i + 1 >= t.size() else String(t[i + 1])

func _faith_row() -> void:
	var rd = app.right_dock_ctrl
	var bridge = app.bridge
	app.select_domain("world")
	app.arm_tool("inspect")
	await _frames(3)
	rd.on_settlement_selected(_sel, 0)
	await _frames(4)

	_check("B0 premise: this build has the belief binding at all",
		bridge.has_belief_api(), "has_belief_api=%s" % bridge.has_belief_api())
	_check("B1 the Settlement context carries a Faith row", _has_text("Faith"),
		"texts=%s" % [_texts().slice(0, 14)])
	## Before any diffusion the two keys are absent, and absence must render as
	## a dash rather than as `none` -- which is a real and different answer.
	var pre: Dictionary = bridge.settlements()[0]
	if not pre.has("religion"):
		_check("B2 with no diffusion run the faith is dashed, not defaulted",
			_faith_value() == "—", "value='%s'" % _faith_value())
		_check("B3 and nothing on screen claims a faith yet",
			not _has_text("Adherence") and not _has_text("No religion"),
			"texts=%s" % [_texts().slice(0, 20)])
	else:
		print("RDB  note: settlements already carry `religion` before this probe ran -- "
			+ "B2/B3 (the absent case) cannot be staged")

	## Give a faction a religion, then run the model. Without this every
	## faction is `none` and the layer is legitimately secular.
	## **This settlement's own faction, not the first one in the roster.**
	## Seeding a different faction leaves this town wholly `none`, which is a
	## real answer and exercises none of the share formatting -- measured on the
	## first run of this probe: `adherents={"none": 19755}`.
	var fid := int(_sel.get("faction", -1))
	if fid < 0:
		print("RDB  !! no faction to give a religion -- the live case cannot be staged")
		return
	var setter_ok: bool = bridge.civ_set_faction_field(fid, "religion", "sun_cult")
	var status: Dictionary = bridge.civ_belief_run(50)
	print("RDB belief run: set_field=%s status=%s" % [setter_ok, status])
	rd.on_settlement_selected(bridge.settlements()[0], 0)
	await _frames(4)

	var live: Dictionary = bridge.settlements()[0]
	if not live.has("religion"):
		_check("B4 a diffusion run puts a religion key on the settlement", false,
			"status=%s" % status)
		return
	var pop := int(live.get("population", 0))
	var adherents: Dictionary = live.get("adherents", {})
	var vocab := _vocab()

	## The engine's own contract, checked rather than restated: head-counts sum
	## to exactly `population`, and a zero count is omitted rather than written.
	var summed := 0
	var zero_rows := 0
	for k in adherents.keys():
		summed += int(adherents[k])
		if int(adherents[k]) <= 0:
			zero_rows += 1
	_check("B4 the head-counts sum to exactly this settlement's population",
		summed == pop, "sum=%d population=%d adherents=%s" % [summed, pop, adherents])
	_check("B5 no zero-adherent faith is present to be shown",
		zero_rows == 0, "adherents=%s" % adherents)

	## The plurality, re-derived from `adherents` alone -- the label on screen
	## has to be the label of the key with the most adherents, computed here
	## without reading the `religion` key the row prints.
	var top_key := ""
	var top_n := -1
	for k in adherents.keys():
		var n := int(adherents[k])
		if n > top_n or (n == top_n and String(k) < top_key):
			top_n = n
			top_key = String(k)
	var expect: String = "No religion" if top_key == "none" else top_key.capitalize()
	var shown := _faith_value()
	_check("B6 the Faith row prints the label of the largest adherent count",
		shown == expect,
		"shown='%s' expected='%s' (top=%s n=%d of %d)"
		% [shown, expect, top_key, top_n, pop])
	_check("B7 it is not a dash -- an answer exists and is printed as one",
		shown != "—" and shown != "", "shown='%s'" % shown)

	## Never show a faith this settlement does not hold: no vocabulary label
	## may appear as an adherence row unless its key is in THIS settlement's
	## own `adherents`.
	_check("B8 the Adherence group is drawn", _has_text("ADHERENCE"),
		"texts=%s" % [_texts().slice(0, 24)])
	var strays := PackedStringArray()
	for key in vocab.keys():
		var k := String(key)
		if adherents.has(k):
			continue
		var lbl: String = "No religion" if k == "none" else k.capitalize()
		if _has_text("◆ " + lbl) or _has_text("◇ " + lbl):
			strays.append(k)
	_check("B9 no faith is listed that this settlement does not hold",
		strays.is_empty(), "strays=%s adherents=%s" % [strays, adherents.keys()])

	## The denominator is stated, and it is this town.
	_check("B10 the share note names its denominator",
		_has_text("Shares of this settlement's own population"),
		"note='%s'" % _text_with("Shares of"))
	var row_line := _text_with(" people (")
	_check("B11 each adherence row prints a count and a share",
		row_line.find("%") >= 0 or row_line.find("(—)") >= 0, "row='%s'" % row_line)

	## `_live_settlement`'s stale-index guard: a snapshot whose `tid` is not the
	## engine's must not resolve to whatever sits at that index.
	var bogus := {"tid": -424242, "name": "no such settlement"}
	var got: Dictionary = rd._live_settlement(bogus)
	_check("B12 a snapshot with a foreign tid resolves to nothing, not to index N",
		got.is_empty(), "got=%s" % [got])
	_check("B13 the real snapshot still resolves",
		not (rd._live_settlement(live) as Dictionary).is_empty())

# -- C: the appended sections still fit the narrowest dock ---------------------
#
# `MISTAKES.md`: *"a `ScrollContainer` with an axis DISABLED folds its child's
# minimum size into its own on that axis, so the overflow propagates to every
# ancestor with no scrollbar to reveal it."* `DccShell._scroll()` disables the
# HORIZONTAL axis, so an appended section that is too wide has no scrollbar to
# hide behind and pushes the whole dock past its own minimum. The vertical axis
# is left enabled, which is what keeps the selection at the top of a long body
# rather than off the bottom of it.

func _fits() -> void:
	var rd = app.right_dock_ctrl
	var bridge = app.bridge
	app.select_domain("world")
	await _frames(3)
	## The worst case on purpose: a selected settlement with a live Adherence
	## group AND a stamp stack that has a selected stamp in it, which is the
	## widest thing this dock can be asked to draw at once.
	if bridge.sculpt_begin_stroke():
		for i in 6:
			bridge.sculpt_add_point(150.0 + i * 4.0, 110.0 + i * 3.0)
		bridge.sculpt_end_stroke()
	rd.on_settlement_selected(bridge.settlements()[0], 0)
	app.arm_tool("sculpt")
	await _frames(6)
	var body: Control = app.right_dock_body
	var scroll: ScrollContainer = body.get_parent() as ScrollContainer
	_check("C0 premise: everything that can append is on screen at once",
		_hdr("SETTLEMENT") == 0 and _hdr("STAMP STACK") > 0 and _has_text("Faith"),
		"headers=%s" % [_headers()])
	_check("C1 the horizontal axis is the disabled one, so width cannot scroll away",
		scroll != null and scroll.horizontal_scroll_mode == ScrollContainer.SCROLL_MODE_DISABLED,
		"mode=%s" % ("<no scroll>" if scroll == null else str(scroll.horizontal_scroll_mode)))
	var min_x: float = body.get_combined_minimum_size().x
	_check("C2 the body still fits the narrowest the dock can be dragged (%d px)"
		% DccTheme.W_RIGHT_DOCK_MIN, min_x <= float(DccTheme.W_RIGHT_DOCK_MIN),
		"combined_minimum_size.x=%.1f" % min_x)
	_check("C3 the vertical axis scrolls, so a long append cannot push the selection off",
		scroll != null and scroll.vertical_scroll_mode != ScrollContainer.SCROLL_MODE_DISABLED,
		"mode=%s" % ("<no scroll>" if scroll == null else str(scroll.vertical_scroll_mode)))
	_check("C4 the selection is still the first thing drawn",
		_headers().size() > 0 and String(_headers()[0]) == "SETTLEMENT",
		"headers=%s" % [_headers()])
	print("RDB  measured: body min width %.1f px, %d sections, %d text nodes"
		% [min_x, _headers().size(), _texts().size()])
	bridge.sculpt_discard()
	app.arm_tool("inspect")
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
	while app.bridge.generating and waited < 2400:
		await get_tree().process_frame
		waited += 1
	print("RDB world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("RDB  !! generate failed -- nothing else here can run")
		get_tree().quit(1)
		return

	var settlements: Array = app.bridge.settlements()
	if settlements.is_empty():
		print("RDB  !! no settlement to select -- every check here is about one")
		get_tree().quit(1)
		return
	_sel = settlements[0]
	_sel_name = String(_sel.get("name", ""))
	_check("the settlement has a name to look for", _sel_name != "", "name='%s'" % _sel_name)

	await _stamps_over_selection()
	await _faith_row()
	await _fits()

	print("RDB fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
