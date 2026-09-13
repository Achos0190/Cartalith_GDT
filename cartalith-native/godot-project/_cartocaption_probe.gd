extends Node
## Lane GATE part A (2026-09-13). Verifies the Cartography options-row caption
## names the ACTIVE MODE through the one shared source
## (`app.gd::_tool_options_cartography_default()`), for every tool armable in
## Cartography and after a bare re-entry, on both desktop and
## (`--force-touch`) phone.
##
## **The defect this pins.** `cartography_workspace.gd::_on_any_tool_armed()`
## used to call its own `_show_style_tool_options()`, which hardcoded
## "CARTOGRAPHY · STYLE" for every tool that is not Icon/Label (Inspect,
## Measure, Region) -- so arming Region while CARTO sat in Labels (or Icons,
## or Terrain appearance) painted STYLE over an open Labels panel. The fix
## deletes that duplicate and routes both writers (`app.gd`'s own
## `_on_workspace_changed`, and `cartography_workspace.gd`'s tool-arm handler)
## through `_tool_options_cartography_default()`.
##
## Mutation: reinstate the hardcoded "CARTOGRAPHY · STYLE" string at either
## call site (or reintroduce a second copy of it) and every non-"style" mode's
## Inspect/Measure/Region row fails below; every Icon/Label row and every
## "style"-mode row is an unchanged control.
##
## Run desktop:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _cartocaption_probe.tscn
## Run phone:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _cartocaption_probe.tscn -- --force-touch

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(cond: bool, msg: String) -> void:
	if cond:
		print("  OK    ", msg)
	else:
		_fail += 1
		print("  FAIL  ", msg)

## `tool_options_row`'s own convention (`_rearmscan_probe.gd`'s identical
## helper, and every `_tool_options_*` builder in `app.gd` /
## `cartography_workspace.gd`): the caption label is always child 0.
func _row_text(app) -> String:
	var row: HBoxContainer = app.tool_options_row
	if row == null or row.get_child_count() == 0:
		return ""
	var c = row.get_child(0)
	return c.text if c is Label else ""

const MODES := ["style", "labels", "icons", "terrain"]
const TOOLS := ["inspect", "measure", "region", "icon", "label"]

## Icon/Label draw their own fixed caption regardless of mode
## (`_build_icon_tool_options_row` / `_build_label_tool_options_row`, both
## `cartography_workspace.gd`) -- neither is this row's shared writer and
## neither is touched by this fix.
const FIXED_CAPTION := {
	"icon": "CARTO · ICON",
	"label": "CARTO · LABEL",
}

## **Measure is not part of this bug's surface at all, discovered by this
## probe's first run** (it asserted "CARTOGRAPHY · <mode>" for Measure and
## failed in all four modes) -- `tool_bar.gd::DccToolBar` connects its OWN
## `tool_armed` listener and claims `tool_options_row` outright for
## `MODES := ["sculpt", "paint", "measure"]`, unconditionally, in every domain
## (`_on_tool_armed()` -> `rebuild()` -> `app.set_tool_options(_build)`, whose
## `_build()` parents a two-row `VBoxContainer` as the row's only child). So
## Measure's child 0 is never a `Label` and `_row_text()` is `""` by
## construction, regardless of mode and regardless of this fix -- verified
## separately below rather than folded into `_expected()`, so a regression
## that made Measure fall through to the shared cartography default (which
## would be the OPPOSITE defect: Measure losing its own toolbar) still fails
## loudly via `_measure_shape_ok()`.
func _expected(tool: String, mode: String) -> String:
	if FIXED_CAPTION.has(tool):
		return String(FIXED_CAPTION[tool])
	return "CARTOGRAPHY · " + mode.to_upper()

## True when `tool_options_row` holds `DccToolBar`'s own composition: a single
## non-Label child (the `col` `VBoxContainer`, `tool_bar.gd::_build()`) with
## its own children, never an empty row and never the accent caption Label
## every `_tool_options_*` builder in `app.gd` / `cartography_workspace.gd`
## uses.
func _measure_shape_ok(app) -> bool:
	var row: HBoxContainer = app.tool_options_row
	if row == null or row.get_child_count() != 1:
		return false
	var c = row.get_child(0)
	return not (c is Label) and c is Node and c.get_child_count() > 0

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return

	var phone_requested := "--force-touch" in OS.get_cmdline_user_args()
	var vp := SubViewport.new()
	vp.size = Vector2i(1080, 2340) if phone_requested else Vector2i(1600, 900)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(50)
	var phone := bool(app.is_phone())
	if phone_requested and not phone:
		print("[FATAL] --force-touch given but did not boot into phone composition")
		get_tree().quit(1); return
	print("[BOOT] composition=%s" % ("phone" if phone else "desktop"))

	for mode in MODES:
		app.call("select_domain_mode", "cartography", mode)
		await _frames(2)
		for tool in TOOLS:
			## Force a genuine OFF->ON transition into `tool` even when the
			## previous iteration already left `armed_tool == tool` --
			## `arm_tool()`'s own same-id early return (`app.gd`) would
			## otherwise make the real arm below a no-op that tests nothing.
			var pre := "measure" if tool != "measure" else "inspect"
			app.call("arm_tool", pre)
			await _frames(2)
			app.call("arm_tool", tool)
			await _frames(2)
			if tool == "measure":
				_ok(_measure_shape_ok(app),
					"mode=%-7s tool=measure after arm:      DccToolBar composition present (not the caption path)" % mode)
			else:
				var want: String = _expected(tool, mode)
				var got: String = _row_text(app)
				_ok(got == want,
					"mode=%-7s tool=%-7s after arm:      got=\"%s\" want=\"%s\"" % [mode, tool, got, want])

			## Re-entry -- Gate A1's own shape: a bare re-selection of the SAME
			## (domain, mode) the tool is already armed under
			## (`app.gd::_on_workspace_changed`'s `re_entry`) must leave this
			## row exactly as it is, not fall back to a stale default.
			app.call("select_domain_mode", "cartography", mode)
			await _frames(2)
			if tool == "measure":
				_ok(_measure_shape_ok(app),
					"mode=%-7s tool=measure after RE-ENTRY: DccToolBar composition still present" % mode)
			else:
				var want2: String = _expected(tool, mode)
				var after: String = _row_text(app)
				_ok(after == want2,
					"mode=%-7s tool=%-7s after RE-ENTRY: got=\"%s\" want=\"%s\"" % [mode, tool, after, want2])

	app.call("arm_tool", "inspect")
	await _frames(2)
	print("\n_cartocaption_probe (%s): %d failure(s)" % [("phone" if phone else "desktop"), _fail])
	get_tree().quit(1 if _fail > 0 else 0)
