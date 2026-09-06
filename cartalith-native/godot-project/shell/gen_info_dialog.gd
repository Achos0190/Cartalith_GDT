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

## Phone (§13) -- PH-12. One column of text, so nothing stacks; what was missing
## was the content scale (this opened as a 560x480 desktop card inside a
## 1440x3168 panel), the tap floor on Copy to clipboard, and a way out bigger
## than `AcceptDialog`'s stock 29 dp OK button.
var _phone := false

func setup(app: Node, bridge: EngineBridge) -> void:
	_app = app
	_bridge = bridge
	title = "Generation info"
	size = Vector2i(560, 480)
	ok_button_text = "Close"
	_phone = DccWidgets.phone_window(self, app)

	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 6)
	add_child(body)
	if _phone:
		DccWidgets.phone_head(body, "Generation info", "parameters, as plain text")

	## **The dump sits in a `section()` since 2026-09-05.** It was three bare
	## children of a `VBoxContainer` -- a note, a `TextEdit`, and a row holding
	## one raw `Button` -- with no `§` header and none of `section()`'s 14 px
	## gutter. `section()`'s own docstring is "a titled band of rows, always
	## expanded", and that is the one composition the DCC vocabulary has for
	## what this window is.
	## Derived from the shell's own two other multi-line `TextEdit`s rather than
	## from a canvas, because no canvas draws a multi-line text well at all:
	## `vault_window.gd::_build_reader()` is `section(_body, "Working copy")` ->
	## `TextEdit` -> `DccWidgets.action(sec, "Save local copy", ...)`, and
	## `place_editor_window.gd::_build_history()` is `section(_body, "History")`
	## -> `TextEdit` with no button under it. This file was the only one of the
	## three outside that shape.
	##
	## What is deliberately NOT changed: the `TextEdit`'s own chrome. It stays
	## stock, like both precedents. `DccWidgets.well()` is the canvas's text
	## field, and its own docstring scopes it to what the canvas draws -- "the
	## canvas's Tile size / World bounds / Destination wells and the Asset
	## library's search", every one a single-line field. Nothing in either design
	## settles what a 560x360 read-only paste-into-a-bug-report well looks like,
	## so it is left alone and named here as an open question for
	## `DESIGN_HANDOFF.md` rather than answered by guess.
	var sec := DccWidgets.section(body, "Parameters")
	## `section()` returns its body VBox inside a `MarginContainer`, and neither
	## carries a vertical size flag -- it is built for a dock column that scrolls,
	## where nothing needs to claim leftover height. In a fixed-height dialog the
	## `TextEdit` below is exactly the thing that must claim it, so both links of
	## the chain are set here rather than in the factory: changing `section()`
	## itself would move its 189 call sites across 18 shell files for one
	## dialog. (Counted 2026-09-05: `grep -rn 'DccWidgets\.section('` over
	## `shell/`, minus comment lines. This comment shipped saying **94**, which
	## was never measured and is off by about half.)
	sec.size_flags_vertical = Control.SIZE_EXPAND_FILL
	(sec.get_parent() as Control).size_flags_vertical = Control.SIZE_EXPAND_FILL
	## `note()` already sets `AUTOWRAP_WORD_SMART`; the line that used to follow
	## this one overrode it back down to `AUTOWRAP_WORD`, which is the same
	## wrapping every other note in the shell does not do. Dropped rather than
	## kept, since nothing ever said why this one note wanted the plainer rule.
	DccWidgets.note(sec, "The current generation's parameters, as plain text -- select and copy, or use the button below. Paste into a bug report for troubleshooting.")

	_text = TextEdit.new()
	_text.editable = false
	_text.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
	## PH-12: 360 authored px is 360 dp of an 864 dp phone screen and reads as a
	## sensible band there too -- but it must not be a FLOOR under a header, a
	## hint and a button, or the four together exceed the screen. `EXPAND_FILL`
	## already gives it every pixel the others do not want.
	##
	## **The floor is gone at every density, measured 2026-09-05**
	## (`_lanea_probe.gd`, three named densities, laid-out extent against the
	## declared `560x480`). It was `0 if _phone else 360`, and 360 was never a
	## size this control needed -- `EXPAND_FILL` over the expand chain now set on
	## `sec` gives it every pixel the note and the button do not want, measured
	## at **345 px** on both pointer densities and **312 px** on tablet, where
	## the note above it is set 3 px larger. What 360 did instead was overflow:
	##
	##   tablet   2560x1600 (`is_tablet()`)   524 px   44 over, before this pass
	##   desktop  1920x1080 (pointer, base)   486 px    6 over, with the section
	##   laptop   1600x900  (`is_laptop()`)   486 px    6 over, with the section
	##
	## The tablet figure is `DccWidgets.note()` resolving `role_px("fs_prose")` at
	## 14 instead of 11, so the hint above grows against a floor that cannot
	## yield; the 6 px is the `§ PARAMETERS` band this pass added over the same
	## floor. One cause, and removing the floor answers both -- all three now
	## measure 480, with the `TextEdit` taking the slack rather than setting it.
	_text.custom_minimum_size = Vector2(0, 0)
	_text.size_flags_vertical = Control.SIZE_EXPAND_FILL
	sec.add_child(_text)

	## Through `DccWidgets.action()`, not a raw `Button`. The rule is already
	## written down in this shell and was simply never applied here --
	## `world_data_window.gd::_cap_note()`: *"a raw Button draws Godot's stock
	## rounded grey pill, which is not a shape this design has anywhere. Caught
	## by screenshot 2026-08-25."* `action()` is the canvas's outline chip
	## (`padding:4px 10px; border:1px solid rgba(255,255,255,.16)`, quoted in
	## that factory's own header), and it is what both `TextEdit` precedents put
	## under their well.
	##
	## It has one phone consequence worth stating rather than discovering:
	## `DccShell.phone_fit()` swaps an `ACTION_META`-marked button for the 412
	## canvas's 48 dp pill, and a raw `Button` carries no such meta. The 44 dp
	## *tap floor* was never the gap -- that walk floors every `BaseButton` it
	## finds, so this button already had it -- but the pill shape it draws in
	## is new here, and it is the shape the phone canvas draws for every other
	## action in the app.
	##
	## **In an `HBoxContainer`, not straight into the section**, and that is not
	## cosmetic: `action()` turns on `AUTOWRAP_WORD_SMART` for any parent that is
	## not a horizontal one, which collapses a button's minimum width. Beside a
	## `SIZE_SHRINK_BEGIN` flag that asks for exactly the minimum, "Copy to
	## clipboard" would draw as one wrapped word. The two precedents cited above
	## avoid it the other way -- they leave the flag at `SIZE_FILL` and take the
	## full section width -- but this button has been compact and left-aligned on
	## the pointer densities since it shipped, and this pass changes what it is
	## made of, not where it sits.
	##
	## A phone has no visible caret-drag select-all, so the button is not a
	## convenience there, it is the only way to get this text out -- hence the
	## full-width flag on that density and not on the others.
	var row := HBoxContainer.new()
	sec.add_child(row)
	var copy_btn := DccWidgets.action(row, "Copy to clipboard", _on_copy)
	copy_btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL if _phone \
		else Control.SIZE_SHRINK_BEGIN
	if _phone:
		_app.phone_fit(self, 1.0)

func open() -> void:
	_text.text = _dump_text()
	if DccWidgets.phone_present(self, _app):
		return
	popup_centered()

func _on_copy() -> void:
	DisplayServer.clipboard_set(_text.text)
	if _app != null and _app.has_method("set_status"):
		_app.set_status("hint", "Generation info copied to clipboard.", "text_ghost")

## **`include_seed_and_params` exists for `diagnostic_report.gd`'s review
## panel, and this dialog never passes it.** The panel lets the user drop the
## seed and the parameter dump from the file it writes (a world is
## reproducible from a seed, so that is a real disclosure choice), and it can
## only offer that honestly if the two blocks are separable. `true` reproduces
## this function's output byte for byte, including the order the lines were in
## before the split -- the seed still sits between the grid and the quality
## tier, which is why the pieces below are five small statics rather than one
## "summary" call with the seed bolted on either end.
func _dump_text(include_seed_and_params: bool = true) -> String:
	if _bridge == null or not _bridge.has_world:
		return NO_WORLD
	var lines: Array[String] = ["Cartalith native port", grid_text(_bridge)]
	if include_seed_and_params:
		var s := seed_text(_bridge)
		if s != "":
			lines.append(s)
	lines.append(quality_text(_bridge))
	lines.append(format_version_text(_bridge))
	lines.append(bindings_text(_bridge))
	if include_seed_and_params:
		lines.append("")
		lines.append(PARAMS_HEADING)
		lines.append(params_text(_bridge))
	return "\n".join(lines)

## The one string both this dialog and `diagnostic_report.gd` show when there
## is nothing to dump, held once so the report's review panel can recognise it
## rather than pattern-match a sentence that might be reworded here.
const NO_WORLD := "No world generated yet."
const PARAMS_HEADING := "Full generation parameters (for reproducing this exact world):"

static func grid_text(bridge: EngineBridge) -> String:
	var g := bridge.grid_size()
	return "Grid %d x %d  ·  %.0f x %.0f km" % [g.x, g.y, bridge.last_width_km, bridge.last_height_km]

## `""` when the binding is absent -- an absent seed is omitted, never printed
## as `Seed 0`, which is a legal seed.
static func seed_text(bridge: EngineBridge) -> String:
	if bridge == null or bridge.world_gen == null or not bridge.world_gen.has_method("get_seed"):
		return ""
	return "Seed %d" % int(bridge.world_gen.get_seed())

static func quality_text(bridge: EngineBridge) -> String:
	return "Quality tier: %s\nGPU: %s" % [bridge.quality_tier(),
		"on" if bool(bridge.param_get("use_gpu")) else "off"]

## The `format_version` this build writes (`SAVEFILE_COMPAT.md` §4). It is
## the first thing asked when an old `.zip` misbehaves, and nothing else in
## the UI printed it. `project_format_version()` returns 0 when the binding
## is absent, which is not a version -- say so rather than print "0".
static func format_version_text(bridge: EngineBridge) -> String:
	var fmt := bridge.project_format_version()
	return "Project format version: %s" % (str(fmt) if fmt > 0 else "unknown (binding absent)")

## The staleness fingerprint. `EngineBridge._has()` already warns once per
## missing method and accumulates the names; printing them here means a
## report that says "feature X is greyed out" arrives with its own answer,
## instead of needing the reporter to find the warnings in a log.
static func bindings_text(bridge: EngineBridge) -> String:
	var missing := bridge.missing_bindings()
	return "Bindings missing: %s" % ("none" if missing.is_empty() else ", ".join(missing))

static func params_text(bridge: EngineBridge) -> String:
	if bridge == null or bridge.world_gen == null or not bridge.world_gen.has_method("get_params"):
		return "(get_params() unavailable -- built against an older binary.)"
	var params: Dictionary = bridge.world_gen.get_params()
	var keys := params.keys()
	keys.sort()
	var out: Array[String] = []
	for k in keys:
		out.append("%s: %s" % [k, JSON.stringify(params[k])])
	return "\n".join(out)
