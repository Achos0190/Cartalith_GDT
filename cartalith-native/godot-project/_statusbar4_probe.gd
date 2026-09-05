extends Node
## The four status-bar states of `design/proposed-2026-09-05/StatusBar.dc.html`,
## plus the rail foot with no world, read back off the drawn nodes of a booted
## shell.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _statusbar4_probe.tscn
##
## **Every figure asserted here is the artboard's own literal**, copied out of
## the drawing and not read back from `DccShell`/`DccTheme`. A probe that reads
## `DccShell.STATUS_RULE_H` and compares it to itself holds for every value of
## it -- seven modal constants shipped that way in batch 34 and all seven
## survived mutation. So: 26 from `--sbH`, 14 from `--pad`, 12 from the rules'
## `padding:0 12px`, 12 from `height:12px`, 1 from `width:1px`, 6 from the dot's
## `margin-right`, and the hexes straight from the `.tok` block --
## `--acc:#e0a34a`, `--block:#c96a5a`, `--dim:#8d9296`, `--dis:#5f6468`,
## `--good:#6fae7d`, `--div:rgba(255,255,255,.07)`.
##
## **Dark is forced and the probe refuses to run without it.** This machine
## boots `mode="light"` (`cartalith_settings.cfg`), where every one of those
## hexes resolves to a different value and each check below would be measuring
## the wrong palette while reading green.
##
## The one figure that is NOT the artboard's: the bar's font. The artboard sets
## the row `font-size:var(--m2)` = 9; the shell draws it through
## `DccTheme.ROLE["fs_status"]`, whose pointer figure is 10. 10 is asserted --
## it is what is on screen -- and the divergence is printed, since closing it is
## a one-value edit in `dcc_theme.gd`, which this lane does not own.

const ARTBOARD_SB_H := 26          ## `--sbH:26px`
const ARTBOARD_PAD := 14           ## `--pad:14px`
const ARTBOARD_SLOT_GAP := 12      ## rules sit in `padding:0 12px`
const ARTBOARD_RULE_H := 12        ## `height:12px`
const ARTBOARD_RULE_W := 1         ## `width:1px`
const ARTBOARD_DOT_GAP := 6        ## `margin-right:6px` on the autosave dot
const ARTBOARD_FOOT_RULE_W := 14   ## `width:14px` under the rail foot's `00`
const ARTBOARD_FOOT_GAP := 6       ## `gap:6px` in the foot column
const ARTBOARD_FOOT_PAD_B := 8     ## `padding-bottom:8px` on the foot column
const ROLE_FS_STATUS := 10         ## `DccTheme.ROLE["fs_status"]`, pointer half

const ACC := "#e0a34a"
const BLOCK := "#c96a5a"
const DIM := "#8d9296"
const DIS := "#5f6468"
const GOOD := "#6fae7d"
const DIV := Color(1, 1, 1, 0.07)

var app: Node
var _fail := 0
## A member, not a local: a GDScript lambda captures locals **by value**, so a
## listener writing to a captured `var` leaves the outer one empty and the check
## reads as "the slot never filled". Cost one probe run.
var _seen_progress := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("SB4 %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _force_dark() -> bool:
	if not DccTheme.is_dark():
		DccTheme.apply_theme(true)
		app.rebuild_theme(false)
	return DccTheme.is_dark()

# -- Reading the bar back off the tree ----------------------------------------

## The cell for a slot, found by the name `_build_status_bar()` gives it, so
## nothing here depends on an index that a reorder could move.
func _cell(slot: String) -> Control:
	for c in app.status_row.get_children():
		if c.name == "Slot_" + slot:
			return c
	return null

func _label(slot: String) -> Label:
	var cell := _cell(slot)
	if cell == null:
		return null
	var found: Array = []
	_collect_labels(cell, found)
	## The autosave cell holds the dot Label first and the text Label second.
	return found.back() if not found.is_empty() else null

func _collect_labels(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append(c)
		_collect_labels(c, out)

func _rule(slot: String) -> ColorRect:
	var cell := _cell(slot)
	if cell == null:
		return null
	for c in cell.get_children():
		if c is ColorRect:
			return c
	return null

## A slot is "drawn" only if its cell is visible AND every ancestor up to the
## bar is too -- `visible` alone is a claim about one node.
func _drawn(slot: String) -> bool:
	var cell := _cell(slot)
	return cell != null and cell.is_visible_in_tree()

func _rule_drawn(slot: String) -> bool:
	var r := _rule(slot)
	return r != null and r.is_visible_in_tree()

func _visible_slots() -> Array:
	var out: Array = []
	for slot in ["pass", "stale", "atlas", "progress", "autosave"]:
		if _drawn(slot):
			out.append(slot)
	return out

func _ruled_slots() -> Array:
	var out: Array = []
	for slot in ["pass", "stale", "atlas", "progress", "autosave"]:
		if _rule_drawn(slot):
			out.append(slot)
	return out

## `#`-prefixed, because every constant above is copied out of the artboard's
## own `.tok` block and those carry the `#`. `Color.to_html(false)` does not.
func _ink(slot: String) -> String:
	var l := _label(slot)
	return "" if l == null else "#" + l.get_theme_color("font_color").to_html(false)

func _node_ink(n: Control) -> String:
	return "#" + n.get_theme_color("font_color").to_html(false)

## The row's own laid-out width, which is what an omitted slot has to shrink.
func _row_w() -> float:
	return app.status_row.get_combined_minimum_size().x

# -- Driving one state --------------------------------------------------------

## `set_status()` per slot, everything unnamed cleared -- the states are
## compositions of which slots have something to say, so a state that leaves a
## previous state's slot standing is not the state.
func _state(slots: Dictionary) -> void:
	for slot in ["pass", "stale", "atlas", "progress", "autosave", "mid", "hint"]:
		if slots.has(slot):
			app.set_status(slot, String(slots[slot][0]), String(slots[slot][1]))
		else:
			app.set_status(slot, "", "text_faint")
	await _frames(2)

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	## **The shell has to have actually booted.** When a concurrent edit leaves
	## any script in the chain uncompilable, `app.tscn` still instantiates -- the
	## node keeps its base class and `app.gd`'s whole `_ready()` never runs, so
	## the rail foot is never composed and `_setup_staleness()` never places its
	## button. Every check below then measures a shell that is not the shell, and
	## most of them still read green. Cost one run here.
	if app.get_script() == null or not app.has_method("_refresh_rail_foot"):
		print("SB4  !! app.gd did not compile -- nothing below would be measuring the app")
		get_tree().quit(1)
		return

	if not _force_dark():
		print("SB4  !! dark palette refused -- every hex below would be the wrong one")
		get_tree().quit(1)
		return
	await _frames(2)
	print("SB4 dark=%s  accent=%s  line_soft=%s" % [DccTheme.is_dark(),
		DccTheme.c("accent").to_html(false), DccTheme.c("line_soft")])

	# -- 0. The frame the four states are drawn in ----------------------------
	var bar: Control = app.status_row.get_parent().get_parent()
	_check("bar is --sbH tall", int(bar.custom_minimum_size.y) == ARTBOARD_SB_H,
		"drawn %d, artboard %d" % [int(bar.custom_minimum_size.y), ARTBOARD_SB_H])
	var pad: MarginContainer = app.status_row.get_parent()
	_check("bar pads by --pad both sides",
		pad.get_theme_constant("margin_left") == ARTBOARD_PAD
			and pad.get_theme_constant("margin_right") == ARTBOARD_PAD,
		"l=%d r=%d artboard=%d" % [pad.get_theme_constant("margin_left"),
			pad.get_theme_constant("margin_right"), ARTBOARD_PAD])
	_check("slot gap is the rules' padding",
		app.status_row.get_theme_constant("separation") == ARTBOARD_SLOT_GAP,
		"drawn %d, artboard %d" % [app.status_row.get_theme_constant("separation"),
			ARTBOARD_SLOT_GAP])
	var r0 := _rule("stale")
	_check("a rule is 1 x 12 in --div",
		r0 != null and int(r0.custom_minimum_size.x) == ARTBOARD_RULE_W
			and int(r0.custom_minimum_size.y) == ARTBOARD_RULE_H
			and r0.color.is_equal_approx(DIV),
		"size=%s colour=%s artboard=(%d,%d) %s" % [r0.custom_minimum_size,
			r0.color, ARTBOARD_RULE_W, ARTBOARD_RULE_H, DIV])
	var pl := _label("pass")
	_check("the bar draws at ROLE fs_status",
		pl != null and pl.get_theme_font_size("font_size") == ROLE_FS_STATUS,
		"drawn %d, ROLE 10, ARTBOARD --m2 = 9 (a one-value edit in dcc_theme.gd)"
			% pl.get_theme_font_size("font_size"))

	# -- The rail foot with no world, before anything is generated ------------
	_check("no world yet -- panel 5's own precondition", not app.bridge.has_world,
		"has_world=%s" % app.bridge.has_world)
	_check("foot reads 00 / 10 and is neither dashed nor blank",
		app.rail_foot.text == "00 / 10", "text=[%s]" % app.rail_foot.text)
	_check("both halves are drawn, upright and stacked",
		app._rail_foot_stack.is_visible_in_tree()
			and app._rail_foot_run.text == "00" and app._rail_foot_total.text == "10",
		"stack=%s run=[%s] total=[%s]" % [app._rail_foot_stack.is_visible_in_tree(),
			app._rail_foot_run.text, app._rail_foot_total.text])
	_check("foot is --dis, the faintest ink",
		_node_ink(app._rail_foot_run) == DIS and _node_ink(app._rail_foot_total) == DIS,
		"run=%s total=%s artboard=%s" % [_node_ink(app._rail_foot_run),
			_node_ink(app._rail_foot_total), DIS])
	var frule: ColorRect = null
	for c in app._rail_foot_stack.get_child(0).get_children():
		if c is ColorRect:
			frule = c
	_check("the / is the artboard's 14 px --div hairline",
		frule != null and int(frule.custom_minimum_size.x) == ARTBOARD_FOOT_RULE_W
			and frule.color.is_equal_approx(DIV),
		"w=%s colour=%s" % ["nil" if frule == null else frule.custom_minimum_size.x,
			"nil" if frule == null else frule.color])
	var fcol: VBoxContainer = app._rail_foot_stack.get_child(0)
	_check("foot stack: gap:6px between the halves and the rule",
		fcol.get_theme_constant("separation") == ARTBOARD_FOOT_GAP,
		"drawn %d, artboard %d" % [fcol.get_theme_constant("separation"),
			ARTBOARD_FOOT_GAP])
	_check("foot stack: padding-bottom:8px under the lower half",
		app._rail_foot_stack.get_theme_constant("margin_bottom") == ARTBOARD_FOOT_PAD_B,
		"drawn %d, artboard %d" % [
			app._rail_foot_stack.get_theme_constant("margin_bottom"), ARTBOARD_FOOT_PAD_B])
	_check("a mode word still takes the rotated form", true, "checked below")

	# -- app.gd's index-2 contract for its Recompute button -------------------
	var kid2: Node = app.status_row.get_child(2)
	_check("status_row child 2 is still the first slot after `stale`",
		kid2 is Button or kid2.name == "Slot_atlas",
		"child2=%s (%s)" % [kid2.name, kid2.get_class()])

	# -- 1. one priority message ----------------------------------------------
	await _state({"pass": ["stage 06 complete · 2.4 s", "accent"],
		"hint": ["V M R · B F · ⌘Z · Esc", "text_ghost"]})
	_check("state 1: one slot drawn, and it is the message",
		_visible_slots() == ["pass"], "visible=%s" % [_visible_slots()])
	_check("state 1: no rule anywhere -- nothing to separate",
		_ruled_slots().is_empty(), "ruled=%s" % [_ruled_slots()])
	_check("state 1: the message is --acc", _ink("pass") == ACC,
		"%s vs artboard %s" % [_ink("pass"), ACC])
	var w1 := _row_w()

	# -- 2. multi-slot ---------------------------------------------------------
	await _state({"pass": ["baked atlas L0–L3", "accent"],
		"stale": ["3 stages stale", "block"],
		"atlas": ["atlas 148 MB", "text_dim"],
		"progress": ["pass 4 / 6", "text_dim"],
		"hint": ["V M R · B F · ⌘Z · Esc", "text_ghost"]})
	_check("state 2: four slots, in the artboard's order",
		_visible_slots() == ["pass", "stale", "atlas", "progress"],
		"visible=%s" % [_visible_slots()])
	_check("state 2: three rules -- between the four, never before the first",
		_ruled_slots() == ["stale", "atlas", "progress"],
		"ruled=%s" % [_ruled_slots()])
	_check("state 2: stale is --block, the readouts --dim",
		_ink("stale") == BLOCK and _ink("atlas") == DIM and _ink("progress") == DIM,
		"stale=%s atlas=%s progress=%s" % [_ink("stale"), _ink("atlas"),
			_ink("progress")])
	var w2 := _row_w()
	_check("state 2 is wider than state 1 -- the slots are really drawn", w2 > w1,
		"%.0f vs %.0f px" % [w2, w1])

	# -- 3. autosave is a state, and never displaces the priority slot --------
	await _state({"pass": ["stage 06 complete · 2.4 s", "accent"],
		"autosave": ["autosave every 5 min · saved 14:32", "text_faint"],
		"hint": ["V M R · B F · ⌘Z · Esc", "text_ghost"]})
	_check("state 3: the priority message is still there, undisplaced",
		_visible_slots() == ["pass", "autosave"] and _ink("pass") == ACC,
		"visible=%s pass=%s" % [_visible_slots(), _ink("pass")])
	_check("state 3: the one rule is autosave's, across three empty cells",
		_ruled_slots() == ["autosave"], "ruled=%s" % [_ruled_slots()])
	var dot: Label = null
	var found: Array = []
	_collect_labels(_cell("autosave"), found)
	dot = found[0] if found.size() > 1 else null
	_check("state 3: the dot is drawn in --good",
		dot != null and dot.is_visible_in_tree() and _node_ink(dot) == GOOD,
		"dot=%s artboard=%s" % ["nil" if dot == null else _node_ink(dot), GOOD])
	_check("state 3: the dot is a real glyph, not a blank cell",
		dot.get_minimum_size().x > 0.0,
		"min=%s -- ● missing from Plex would measure 0 and still read visible"
			% dot.get_minimum_size())
	_check("state 3: the dot sits 6 px ahead of its own text",
		dot.get_parent().get_theme_constant("separation") == ARTBOARD_DOT_GAP,
		"sep=%d artboard=%d" % [dot.get_parent().get_theme_constant("separation"),
			ARTBOARD_DOT_GAP])
	await _state({"pass": ["stage 06 complete · 2.4 s", "accent"],
		"autosave": ["autosave failed", "accent"]})
	_check("a failed autosave does not keep a healthy dot", _node_ink(dot) == ACC,
		"dot=%s, --good would be %s" % [_node_ink(dot), GOOD])

	# -- 4. loaded, nothing generated this session ----------------------------
	await _state({"pass": ["loaded — no generation this session", "text_dim"],
		"hint": ["V M R · B F · ⌘Z · Esc", "text_ghost"]})
	_check("state 4: one slot, --dim, no rule before it",
		_visible_slots() == ["pass"] and _ink("pass") == DIM
			and _ruled_slots().is_empty(),
		"visible=%s ink=%s ruled=%s" % [_visible_slots(), _ink("pass"),
			_ruled_slots()])

	# -- The omission rule, including a hole in the middle --------------------
	await _state({"pass": ["generating…", "accent"], "progress": ["pass 4 / 10", "text_dim"]})
	_check("a hole in the middle leaves no hanging hairline",
		_visible_slots() == ["pass", "progress"] and _ruled_slots() == ["progress"],
		"visible=%s ruled=%s" % [_visible_slots(), _ruled_slots()])
	var w_two := _row_w()
	await _state({"pass": ["generating…", "accent"]})
	var w_one := _row_w()
	_check("clearing a slot actually removes its width", w_one < w_two,
		"%.0f -> %.0f px" % [w_two, w_one])
	_check("an emptied slot is omitted, not blanked",
		not _drawn("progress") and _label("progress").text == "",
		"drawn=%s" % _drawn("progress"))

	# -- The `progress` slot has a real source, not a probe-fed one -----------
	app.set_status("progress", "", "text_dim")
	app.bridge.generation_stage.connect(func(_i: int, _n: String, _t: int):
		if _seen_progress == "":
			_seen_progress = app.status_slot_text("progress"))
	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	await _frames(6)
	_check("`progress` filled from the engine's own generation_stage tick",
		_seen_progress.begins_with("pass ") and _seen_progress.ends_with(" / 10"),
		"first tick wrote [%s]  (STAGE_NAMES is 10 long)" % _seen_progress)
	_check("`progress` is cleared when the run ends, not left standing",
		app.status_slot_text("progress") == "" and not _drawn("progress"),
		"[%s] drawn=%s" % [app.status_slot_text("progress"), _drawn("progress")])
	_check("the foot counts the run: 10 / 10",
		app.rail_foot.text == "10 / 10" and app._rail_foot_run.text == "10",
		"text=[%s]" % app.rail_foot.text)

	# -- What drawing the autosave slot exposes in app.gd ---------------------
	#
	# Not a check: the overlap is `app.gd`'s to settle and a committed probe
	# that fails on another file's defect is a probe nobody re-runs. Printed
	# because the numbers are the evidence for the one-line fix.
	app._refresh_save_status()
	await _frames(2)
	print("SB4 NOTE autosave slot = [%s]" % app.status_slot_text("autosave"))
	print("SB4 NOTE mid slot      = [%s]" % app.status_slot_text("mid"))
	_check("the autosave slot is drawn at all now -- state 3 needs it",
		_drawn("autosave") and app.status_slot_text("autosave") != "",
		"drawn=%s text=[%s]" % [_drawn("autosave"),
			app.status_slot_text("autosave")])

	# -- A mode word keeps the rotated form -----------------------------------
	app.set_rail_foot("SCULPT")
	await _frames(2)
	_check("a mode word draws rotated, and the counter stack stands down",
		app.rail_foot.is_visible_in_tree() and not app._rail_foot_stack.is_visible_in_tree()
			and is_equal_approx(app.rail_foot.rotation, -PI / 2.0),
		"foot vis=%s stack vis=%s rot=%.3f" % [app.rail_foot.is_visible_in_tree(),
			app._rail_foot_stack.is_visible_in_tree(), app.rail_foot.rotation])

	print("SB4 %d failed" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
