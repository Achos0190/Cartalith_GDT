extends AcceptDialog
class_name GenInfoDialog

## The reference's ℹ️ `#genInfoBtn` / `#genInfoPanel` / `generationInfoText()`
## (`PARITY_AUDIT.md` §5 item 6) — a bug-report affordance: dump every
## generation parameter as plain text a user can paste into a report.
##
## `generationInfoText()` itself is two parts: a hand-picked summary (grid,
## plates, temperature range, altitude range, max grade -- values read
## straight off live JS arrays this port has no equivalent accessor for) and
## a `JSON.stringify` of the *whole* generation-affecting state, deliberately
## not hand-picked so a future slider needs no update here. Only the second
## part is buildable "almost entirely call the existing function, format it,
## show it" -- `WorldGen.get_params()` (`cartalith-godot/src/lib.rs`) is
## already exactly that: every generation parameter, current value, flat
## dotted-key dictionary, self-updating as new params are added. The
## elevation/temperature/grade summary line is real engine-side work
## (no `#[func]` anywhere returns field min/max) and out of this ticket's
## scope -- this dialog leads with what IS free (grid size, seed, extent)
## and lets `get_params()` cover the rest, same spirit as the reference's
## own "don't hand-pick, dump everything" reasoning.

var _bridge: EngineBridge
var _app: Node
var _text: TextEdit

func setup(app: Node, bridge: EngineBridge) -> void:
	_app = app
	_bridge = bridge
	title = "Generation info"
	size = Vector2i(560, 480)

	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 6)
	add_child(body)

	var hint := DccWidgets.note(body, "The current generation's parameters, as plain text -- select and copy, or use the button below. Paste into a bug report for troubleshooting.")
	hint.autowrap_mode = TextServer.AUTOWRAP_WORD

	_text = TextEdit.new()
	_text.editable = false
	_text.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
	_text.custom_minimum_size = Vector2(0, 360)
	_text.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body.add_child(_text)

	var row := HBoxContainer.new()
	body.add_child(row)
	var copy_btn := Button.new()
	copy_btn.text = "Copy to clipboard"
	copy_btn.pressed.connect(_on_copy)
	row.add_child(copy_btn)

func open() -> void:
	_text.text = _dump_text()
	popup_centered()

func _on_copy() -> void:
	DisplayServer.clipboard_set(_text.text)
	if _app != null and _app.has_method("set_status"):
		_app.set_status("hint", "Generation info copied to clipboard.", "text_ghost")

func _dump_text() -> String:
	if _bridge == null or not _bridge.has_world:
		return "No world generated yet."

	var lines: Array[String] = []
	var g := _bridge.grid_size()
	lines.append("Cartalith native port")
	lines.append("Grid %d x %d  ·  %.0f x %.0f km" % [g.x, g.y, _bridge.last_width_km, _bridge.last_height_km])
	if _bridge.world_gen.has_method("get_seed"):
		lines.append("Seed %d" % int(_bridge.world_gen.get_seed()))
	lines.append("Quality tier: %s" % _bridge.quality_tier())
	var gpu_on := bool(_bridge.param_get("use_gpu"))
	lines.append("GPU: %s" % ("on" if gpu_on else "off"))
	lines.append("")
	lines.append("Full generation parameters (for reproducing this exact world):")

	if _bridge.world_gen.has_method("get_params"):
		var params: Dictionary = _bridge.world_gen.get_params()
		var keys := params.keys()
		keys.sort()
		for k in keys:
			lines.append("%s: %s" % [k, JSON.stringify(params[k])])
	else:
		lines.append("(get_params() unavailable -- built against an older binary.)")

	return "\n".join(lines)
