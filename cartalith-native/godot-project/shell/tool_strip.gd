extends PanelContainer
## MS-5, second step (`MOBILE_SHELL_PROPOSAL.md` section 2.1 "The armed tool", section 5's "`tool_fan.gd` pill plus
## the per-tool phone bars -> one tool strip", section 6's MS-5 bar "every tool armed, used and disarmed from the
## strip"): the phone's one home for the ARMED tool, a strip above the sheet that shows
##   row 1   the tool chip (its name; a TAP raises or lowers the sheet to the tool's full options, a HOLD or a slide
##           opens the tool fan), a one-line hint, and one 44 dp button that disarms (X) or, from Inspect, re-arms
##           the tool that Inspect replaced;
##   row 2   the tool's main options as chips, then Commit and Discard where the tool drafts. Row 2 is absent while
##           it has nothing to say (Inspect, and every tool with neither an option nor a draft).
##
## ## It replaces the pill, and it does not copy anything
##
## This file owns NO tool logic. Every control calls a function that already existed:
##   - the chip IS `tool_fan.gd`'s `_Pill` (reparented here by `ToolFan.mount()`), so the press, hold, slide and
##     lift grammar and the fan it opens are the fan's own, unchanged;
##   - the option chips are the `"tool"`-section rows the context card already collects
##     (`ContextBroker.collect()`; Settlement's class, Sculpt's brush size, Paint's radius and Erase), each row's
##     own `param.value` / `param.step` callables, so the strip, the card and the dock write one state;
##   - Commit and Discard are `draft_verbs(tool_id)`, which `world_workspace.gd` and `civilization_workspace.gd`
##     answer with the SAME callables their own Commit / Discard buttons and the card's Draft rows run;
##   - disarm and re-arm are `ToolFan.toggle_inspect()`, the function the pill's tap used to call, through
##     `app.arm_tool` like every other route.
## The per-tool option bars (`tool_bar.gd`, the workspaces' `_tool_options_*` builders) are NOT replaced by this
## step: they are "the full options" the chip opens (section 2.1), still drawn in the sheet.
##
## ## Where it sits
##
## Bottom-anchored in `DccShell._phone_content_gap`, directly above the tool sheet and clear of the bottom nav, as
## the pill was. `ViewportHost` lifts the map's control row by `reserve()` (read through
## `DccShell.phone_tool_pill_reserve()`), so the row sits above the strip and the strip never covers a control.
## It is hidden, like the pill, while the MORE menu or the card's sheet covers the screen (`ToolFan._process`).
##
## ## What must never change
##
## The strip must never arm a tool by any route but `app.arm_tool`, never call an engine `#[func]` while
## `bridge.generating` (every read below is guarded; a worker owns the engine and `bind()` panics), and never
## show a Commit as live when the draft is known to be empty (`can`).

const ToolFan := preload("res://shell/tool_fan.gd")

## Gap between the strip's bottom edge and the bottom of its container, in dp. The pill used 10 dp above the sheet
## (a labelled judgement carried over so the map's control row clears it by the same margin as before).
const GAP_DP := 10.0
## Side margin of the strip in dp: the map row's own `NAVPAD_EDGE` (12 dp, `viewport_host.gd`), so the strip and
## the row share one edge and nothing floats wider than the row below it.
const EDGE_DP := 12.0
## Poll period for the strip's live state (draft counts, option values) in seconds. 0.25 s: a stroke that adds a
## stamp turns Commit live within a quarter of a second, and four reads a second of two cheap getters is below
## `DccShell._process`'s own per-frame keyboard-height poll in cost. A labelled judgement, not a measurement.
const POLL_S := 0.25
## The strip's widest extent in dp. A phone in portrait is at most about 440 dp wide, so this changes nothing there;
## in landscape (2 340 px / 2.8 is 830 dp) a full-width strip hid a third of the map (a three-row strip measured
## 940 x 260 px at 2340 x 1080), so it is centred at this width instead. A labelled judgement: wide enough for a
## chip, a hint, three option chips and the two verbs on one or two rows.
const MAX_W_DP := 560.0

## What each tool's strip says, one short line. The words are this file's own (a labelled judgement, the way a
## status line is worded); the strings only describe what the tool's existing handlers do. An id this table does
## not name shows NO hint (an absent string, never a placeholder one). Inspect's is Ruling DZ's own sentence for
## the no-tool state, shortened for the phone (it has no cartwheel).
const HINTS := {
	"inspect": "Tap selects · hold opens the card",
	"measure": "Tap points on the map to measure",
	"sculpt": "Drag to sculpt · Commit bakes the draft",
	"paint": "Drag to paint · Commit writes the dabs",
	"colour_paint": "Drag to paint colour onto the map",
	"settlement": "Tap to place a settlement",
	"label": "Tap to place a label",
	"icon": "Tap to stamp an icon",
	"conflict": "Tap points · Commit saves the conflict",
	"route": "Tap waypoints · Commit solves the route",
	"way": "Tap points · Commit lays the way",
	"region": "Drag to outline a region",
	"territory": "Drag to paint territory · Commit",
	"territory_lasso": "Tap vertices · Commit closes the ring",
	"journey": "Plan the journey in the sheet",
}

var app  ## DccApp (== DccShell)
var fan  ## ToolFan; owns the chip and the gesture grammar
var _hint: Label
var _disarm: Button
var _row1: HBoxContainer
var _flow: HFlowContainer
## The `"tool"` rows from the last `refresh()` (each `param` carries live `value` and `step` callables).
var _rows: Array = []
## A string that changes whenever anything the strip draws changes; `_process` compares it, `refresh()` rebuilds on
## a difference. Built from the armed tool, the draft counts and every option's current value.
var _sig := ""
var _acc := 0.0
## Whether the current draw offered Commit / Discard (a probe seam, `debug_state()`).
var _has_verbs := false
var _commit: Button
var _discard: Button


## Wires the strip to the shell and to `fan` (already `setup()`), builds its two rows and mounts the fan's chip into
## row 1. `host` is `_phone_content_gap`. Call once, after `fan.setup(app)`. Never twice: it adds children.
func setup(app_ref, fan_ref, host: Control) -> void:
	app = app_ref
	fan = fan_ref
	name = "ToolStrip"
	mouse_filter = Control.MOUSE_FILTER_STOP  ## the strip's own padding must not let a tap fall through to the map
	var r: int = app.phone_px(14)
	add_theme_stylebox_override("panel", DccTheme.pill(false, r, app.phone_px(6), app.phone_px(4)))
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", app.phone_px(4))
	add_child(col)
	_row1 = HBoxContainer.new()
	_row1.add_theme_constant_override("separation", app.phone_px(8))
	col.add_child(_row1)
	fan.mount(_row1)  ## the chip, first in row 1
	_hint = Label.new()
	_hint.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_hint.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	## One line, trimmed with an ellipsis: an autowrapping label reports its height at its MINIMUM width, which made
	## the whole row 317 px tall at 1440 x 3120 (found by `_ms5b_probe`). The label is the row's expanding cell, so
	## the ellipsis cannot collapse it (MISTAKES.md, "Add clip_text / ellipsis").
	_hint.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	_hint.custom_minimum_size.x = app.phone_px(40)
	_hint.add_theme_font_override("font", DccTheme.mono())
	_hint.add_theme_font_size_override("font_size", app.phone_font(DccTheme.FS_MICRO))
	_hint.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	_hint.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_row1.add_child(_hint)
	_disarm = _make_button("")
	_disarm.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	_disarm.pressed.connect(_on_disarm_pressed)
	_row1.add_child(_disarm)
	_flow = HFlowContainer.new()
	_flow.add_theme_constant_override("h_separation", app.phone_px(6))
	_flow.add_theme_constant_override("v_separation", app.phone_px(4))
	col.add_child(_flow)
	## Bottom-anchored and growing upward: the strip's height is its content's, row 2 included.
	set_anchors_preset(Control.PRESET_BOTTOM_WIDE)
	grow_vertical = Control.GROW_DIRECTION_BEGIN
	var h: float = float(app.phone_tap(DccTheme.PHONE_TAP_MIN))
	offset_left = float(app.phone_px(EDGE_DP))
	offset_right = -float(app.phone_px(EDGE_DP))
	offset_bottom = -float(app.phone_px(GAP_DP))
	offset_top = offset_bottom - h
	host.add_child(self)
	host.resized.connect(_fit_width)
	_fit_width()
	## The strip grows and shrinks with its second row; the map's control row sits above it by `reserve()` and
	## re-reads that only on `phone_insets_changed`, so a height change must say so (found by `_ms5b_probe`: the
	## row overlapped a two-row strip by 14 px).
	resized.connect(func() -> void: app.phone_insets_changed.emit())
	if app.has_signal("tool_armed"):
		app.tool_armed.connect(func(_id: String) -> void: refresh())
	## MS-1 S4.3: wired one idle frame later. On a cold boot the shell builds this strip from `DccShell._ready()`,
	## BEFORE `DccApp._ready()` creates the bridge, so `app.bridge` is null here and the two connections were never
	## made (measured: 0 receivers after a cold phone boot, 1 after a remount, where the bridge exists), leaving a
	## strip that did not refresh after a generate or a load. A remount and a boot now both end with exactly one.
	_wire_bridge.call_deferred()
	refresh()


## True once the bridge connections exist, so a deferred call that finds them made does nothing.
var _bridge_wired := false

## Connects the strip to the bridge's `world_loaded` and `generation_finished`, once. Does nothing while the bridge
## does not exist yet or after the strip has left the tree (a swap freed it before this deferred call ran).
func _wire_bridge() -> void:
	var b = app.bridge if app != null and is_instance_valid(app) else null
	if _bridge_wired or b == null or not is_inside_tree():
		return
	_bridge_wired = true
	b.world_loaded.connect(refresh)
	b.generation_finished.connect(func(_ok: bool) -> void: refresh())


## Anchors the strip full width (less the side margins), or, when the container is wider than `MAX_W_DP`, centred at
## that width. Runs on every resize of the container (a rotation or a window change).
func _fit_width() -> void:
	var edge: float = float(app.phone_px(EDGE_DP))
	var max_w: float = float(app.phone_px(MAX_W_DP))
	var host_w: float = (get_parent() as Control).size.x
	if host_w - 2.0 * edge > max_w:
		anchor_left = 0.5
		anchor_right = 0.5
		offset_left = -max_w * 0.5
		offset_right = max_w * 0.5
	else:
		anchor_left = 0.0
		anchor_right = 1.0
		offset_left = edge
		offset_right = -edge


## Height in device px of the band the strip takes above the bottom of its container (its own height plus the gap
## under it); 0 while it is hidden. `ViewportHost` lifts the map's control row by it.
func reserve() -> float:
	if not visible:
		return 0.0
	return maxf(size.y, get_combined_minimum_size().y) - offset_bottom


## A 44 dp-tall pill button in the strip's idiom. Its width is its text's plus padding, floored at 44 dp.
func _make_button(text: String, primary: bool = false) -> Button:
	var b := Button.new()
	b.text = text
	b.focus_mode = Control.FOCUS_NONE
	var d: int = app.phone_tap(DccTheme.PHONE_TAP_MIN)
	b.custom_minimum_size = Vector2(d, d)
	var rest := DccTheme.pill(primary, d / 2, app.phone_px(12), app.phone_px(4))
	var lit := DccTheme.pill(primary, d / 2, app.phone_px(12), app.phone_px(4))
	lit.bg_color = DccTheme.c("accent_hover") if primary else DccTheme.c("accent_wash_2")
	var off := DccTheme.pill(false, d / 2, app.phone_px(12), app.phone_px(4))
	b.add_theme_stylebox_override("normal", rest)
	b.add_theme_stylebox_override("hover", lit)
	b.add_theme_stylebox_override("pressed", lit)
	b.add_theme_stylebox_override("disabled", off)
	b.add_theme_stylebox_override("focus", DccTheme.empty())
	b.add_theme_font_override("font", DccTheme.mono(0, true))
	b.add_theme_font_size_override("font_size", app.phone_font(DccTheme.FS_SMALL))
	var ink := DccTheme.c("accent_ink") if primary else DccTheme.c("text_bright")
	b.add_theme_color_override("font_color", ink)
	b.add_theme_color_override("font_hover_color", ink)
	b.add_theme_color_override("font_pressed_color", ink)
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	return b


## The tool-section rows for the armed tool, asked of the context broker with NO map point (the strip is not a
## gesture on the map), so a provider that reads `gx` / `gy` sees 0, which its `"tool"` rows ignore. A provider
## answers only while its own domain is the active one (the card's rule: a settlement right-clicked in CARTO gets
## CIVIL's verbs through one jump row), and the strip must show the ARMED tool's options whichever tab is lit, so
## the request is made once per domain and the rows are de-duplicated by `id` (a `"tool"` row has one provider).
## Empty while the engine is busy or the shell has no broker (a bare shell in a probe).
func _tool_rows() -> Array:
	if app.bridge != null and app.bridge.generating:
		return []
	if app.context_broker == null:
		return []
	var out: Array = []
	var seen := {}
	for dom in ["world", "civilization", "cartography"]:
		## `gx` / `gy` are present (0.0) because several providers read `req["gx"]` unconditionally; their
		## `"tool"` rows never use the point.
		var req: Dictionary = app.context_broker.build_request({"gx": 0.0, "gy": 0.0})
		req["domain"] = dom
		for row in app.context_broker.collect(req):
			var d := row as Dictionary
			if String(d.get("section", "")) == "tool" and d.has("param") and not seen.has(d.get("id", "")):
				seen[d.get("id", "")] = true
				out.append(d)
	return out


## The armed tool's draft verbs from whichever workspace drafts with it, `{}` if none does (Inspect, Measure ...).
func _verbs() -> Dictionary:
	var id := String(app.armed_tool)
	if id == "inspect":
		return {}
	for ws in app._workspaces:
		if ws.has_method("draft_verbs"):
			var v: Dictionary = ws.draft_verbs(id)
			if not v.is_empty():
				return v
	return {}


## The change signature `_process` polls: armed tool, the verbs' `can` and `count`, and each option's value text.
func _signature() -> String:
	var parts: Array = [String(app.armed_tool)]
	var v := _verbs()
	parts.append("%s:%d" % [str(v.get("can", "-")), int(v.get("count", -2))])
	for row in _rows:
		parts.append(String(((row as Dictionary)["param"] as Dictionary)["value"].call()))
	return "|".join(PackedStringArray(parts))


## Rebuilds both rows from the live state. Safe at any time; draws only what the state supports.
func refresh() -> void:
	if app == null:
		return
	var id := String(app.armed_tool)
	_hint.text = String(HINTS.get(id, ""))
	_hint.visible = not _hint.text.is_empty()
	## The disarm / re-arm button. Armed: X, disarms (arm Inspect, remembering the tool). Inspect with a remembered
	## tool: the return arrow, which re-arms it. Inspect with nothing remembered: absent, not a dead button.
	if id != "inspect":
		_disarm.text = "✕"
		_disarm.visible = true
		_disarm.accessibility_name = "Disarm %s" % id.capitalize()
		_disarm.tooltip_text = "Disarm %s (back to Inspect)" % id.capitalize()
	elif fan.can_rearm():
		_disarm.text = "↩"
		_disarm.visible = true
		_disarm.accessibility_name = "Re-arm %s" % fan.prev_tool().capitalize()
		_disarm.tooltip_text = "Re-arm %s" % fan.prev_tool().capitalize()
	else:
		_disarm.visible = false
	for c in _flow.get_children():
		_flow.remove_child(c)
		c.queue_free()
	_commit = null
	_discard = null
	_rows = _tool_rows() if id != "inspect" else []
	for row in _rows:
		_add_param_chips(row as Dictionary)
	var v := _verbs()
	_has_verbs = not v.is_empty()
	if _has_verbs:
		var can := bool(v.get("can", true))
		_commit = _make_button("Commit", true)
		_commit.disabled = not can
		_commit.accessibility_name = "Commit"
		_commit.pressed.connect(_run_verb.bind(v["commit"]))
		_flow.add_child(_commit)
		_discard = _make_button("Discard")
		_discard.disabled = not can
		_discard.accessibility_name = "Discard"
		_discard.pressed.connect(_run_verb.bind(v["discard"]))
		_flow.add_child(_discard)
	_flow.visible = _flow.get_child_count() > 0
	## No `reset_size()`: it sets the size to the MINIMUM and drops the anchors' width (the strip came out 567 px wide
	## and 510 tall at 1080 x 2340). The offsets own the size; the container grows it upward when a row appears.
	_sig = _signature()


## One option row as chips: a toggle (its value is "on" / "off") is one chip; a stepped value is a minus, the
## value, a plus. Each press calls the row's own `param.step(dir)` and then refreshes, so the strip never shows a
## value the provider did not just write.
func _add_param_chips(row: Dictionary) -> void:
	var p: Dictionary = row["param"]
	var label := String(row.get("label", ""))
	var val := String((p["value"] as Callable).call())
	var step: Callable = p["step"]
	if val == "on" or val == "off":
		var t := _make_button("%s %s" % [label, val], val == "on")
		t.accessibility_name = "%s %s" % [label, val]
		t.pressed.connect(func() -> void: _step(step, 1))
		_flow.add_child(t)
		return
	var minus := _make_button("−")
	minus.accessibility_name = "%s less" % label
	minus.pressed.connect(func() -> void: _step(step, -1))
	_flow.add_child(minus)
	var shown := _make_button("%s %s" % [label, val])
	shown.disabled = true  ## the value reads like a chip but is not a control: the signs step it
	_flow.add_child(shown)
	var plus := _make_button("+")
	plus.accessibility_name = "%s more" % label
	plus.pressed.connect(func() -> void: _step(step, 1))
	_flow.add_child(plus)


func _step(step: Callable, dir: int) -> void:
	if app.bridge != null and app.bridge.generating:
		return
	step.call(dir)
	refresh()


## Runs a draft verb the workspace handed over, then redraws (a Commit empties the draft, so Commit goes dim).
func _run_verb(cb: Callable) -> void:
	if cb.is_valid():
		cb.call()
	refresh()


func _on_disarm_pressed() -> void:
	fan.toggle_inspect()


func _process(dt: float) -> void:
	_acc += dt
	if _acc < POLL_S or not is_visible_in_tree():
		return
	_acc = 0.0
	if app.bridge != null and app.bridge.generating:
		return
	if _signature() != _sig:
		refresh()


## Probe seam: what the strip shows now. Read-only; never feed it back into logic.
func debug_state() -> Dictionary:
	return {
		"hint": _hint.text, "disarm_text": _disarm.text if _disarm.visible else "",
		"options": _rows.size(), "verbs": _has_verbs,
		"commit_disabled": _commit != null and _commit.disabled,
		"flow_visible": _flow.visible,
	}
