class_name PenLink
extends Node
## The GDScript facade of the OnePlus Stylo 2 Pen link (Ruling EK; STYLO2_PEN_PROPOSAL.md sections 4, 5 and 6.3).
## It owns ALL policy: whether the link is on, the permission flow, the misfire guard, the slide accumulator and
## the mapping from a double tap or a slide to an EXISTING entry point. The Kotlin plugin `CartalithPen`
## (`android/pen-plugin/`) knows bytes and nothing about tools.
##
## ABSENT, NEVER AN ERROR. Off Android, on the 6T, or in any build without the plugin, `Engine.has_singleton
## ("CartalithPen")` is false: `state()` is "absent", every method is a no-op and nothing is raised. A probe injects
## a fake backend through `attach_backend` (the real plugin has no stand-in on a PC; there is no BLE here).
##
## EVERY ACTION GOES THROUGH AN EXISTING ENTRY POINT, and none moves generated output: `DccApp.arm_tool` (last tool),
## `DccApp.undo_last`, the WORLD workspace's own brush steppers (`pen_step_size`, `pen_toggle_erase`, which wrap
## `_step_brush_size`/`_step_paint_radius`/`_on_paint_erase_changed`), `ViewportHost.zoom_step`,
## `DccShell.tl_set_year` and `DccQuickBar.run_pin_on` (the quick bar's `pin_key` path).
##
## MUST NEVER: write to the pen beyond the plugin's three queries, offer a vibration setting (P6), request a
## permission before the user switches the link on, show a missing battery as `0`, or let an action fail silently
## (an action that cannot run says why in the status line).

## The plugin's singleton name (the Kotlin `getPluginName()`).
const PLUGIN := "CartalithPen"

## The one runtime permission the link needs (Ruling EK: no scan, no location).
const PERMISSION := "android.permission.BLUETOOTH_CONNECT"

## Ignore a double tap while the tip is on the glass, and this long after it lifts, ms (proposal section 4.1; SP-4
## measures whether the figure is needed: a labelled judgement).
const TAP_GUARD_MS := 150

## A slide report this long after the previous one starts a NEW contact, so no delta crosses a finger lift, ms
## (proposal section 6.3; SP-5 measures it: a labelled judgement).
const SLIDE_GAP_MS := 250

## Slide positions per action step. Labelled judgements, tuned in SP-5: the strip has about 250 positions
## (`0x05`..`0xFF`), so 6 gives the 40-cell paint radius range in ~240 positions (one pass), the sculpt brush's
## twenty steps in 120, a zoom step per 12 and a 5-year step per 5.
const POS_PER_STEP := {"brush_size": 6, "zoom": 12, "year": 5}

## Years moved per year step (a labelled judgement, see POS_PER_STEP).
const YEAR_STEP := 5

## The low-battery chip's threshold, percent (after Samsung's S Pen alert, proposal section 4.3).
const LOW_BATTERY := 20

## Emitted when the link state changes: absent, off, unsupported, no_permission, not_paired, connecting, connected, lost.
signal state_changed(state: String)
## Emitted when battery, firmware or name changed. Read `battery()` etc.; an unknown field is absent, not a value.
signal info_changed
## Every pen event for the tester: kind is "double_tap" or "slide", detail the position or the action's outcome.
signal pen_event(kind: String, detail: String)

## The owning app (`DccApp`), set by `app.gd` when it creates this node. Null in a bare unit test.
var app = null

var _backend: Object = null
var _state := "off"
var _battery := -1
var _firmware := ""
var _name := ""
var _chip_shown := false
var _cur_tool := ""
var _prev_tool := ""
var _tip_down := false
var _tip_up_ms := -1000000
var _slide := {"last_pos": -1, "last_ms": -1000000, "carry": 0}
## Last outcome strings, for the tester: what the last tap/slide did.
var _last_action := ""

func _ready() -> void:
	if _backend == null and Engine.has_singleton(PLUGIN):
		attach_backend(Engine.get_singleton(PLUGIN))
	if app != null and app.has_signal("tool_armed"):
		_cur_tool = String(app.armed_tool)
		app.tool_armed.connect(_on_tool_armed)
	if DccSettings.pen_link_on():
		apply_setting()

## Connects a backend (the real plugin singleton, or a probe's fake) and listens to its four signals. Idempotent.
func attach_backend(b: Object) -> void:
	if _backend == b:
		return
	_backend = b
	if b == null:
		return
	for pair in [["pen_state", _on_pen_state], ["pen_double_tap", _on_double_tap], ["pen_slide", _on_slide],
			["pen_info", _on_pen_info]]:
		if b.has_signal(String(pair[0])):
			b.connect(String(pair[0]), pair[1])

## "absent" when there is no plugin; otherwise the last state the plugin reported ("off" before start).
func state() -> String:
	return "absent" if _backend == null else _state

## Whether a plugin (or a probe's fake) is attached.
func available() -> bool:
	return _backend != null

## Battery percent, or -1 when unknown. -1 is the ABSENT marker callers must test; never display it.
func battery() -> int:
	return _battery

func firmware() -> String:
	return _firmware

func device_name() -> String:
	return _name

## One sentence saying why the link is not connected, or "" when it is. Shown in settings and the tester next to a
## dash, so a missing figure always comes with its reason.
func reason() -> String:
	match state():
		"connected":
			return ""
		"absent":
			return "Needs a OnePlus Stylo 2 on Android"
		"off":
			return "Pen link is off"
		"unsupported":
			return "Needs Android 13 or newer with Bluetooth LE"
		"no_permission":
			return "Bluetooth permission denied"
		"not_paired":
			return "Pen not paired"
		"connecting":
			return "Connecting"
		"lost":
			return "Connection lost, retrying"
	return state()

## The text of the last double tap or slide outcome, for the tester.
func last_action() -> String:
	return _last_action

## Applies `DccSettings.pen_link_on()`: starts the link (asking for the permission first) or stops it. Called when
## the user flips the switch and once at startup if it was left on. Safe with no plugin.
func apply_setting() -> void:
	if _backend == null:
		return
	if not DccSettings.pen_link_on():
		_backend.call("stop")
		return
	if not bool(_backend.call("is_supported")):
		_on_pen_state("unsupported")
		return
	if bool(_backend.call("has_permission")):
		_backend.call("start")
		return
	# The Nearby-devices prompt appears HERE and nowhere earlier (Ruling EK). The result arrives asynchronously.
	if not get_tree().on_request_permissions_result.is_connected(_on_permission_result):
		get_tree().on_request_permissions_result.connect(_on_permission_result)
	OS.request_permission(PERMISSION)

func _on_permission_result(permission: String, granted: bool) -> void:
	if permission != PERMISSION:
		return
	if granted:
		if DccSettings.pen_link_on() and _backend != null:
			_backend.call("start")
	else:
		# Denied: the switch goes back to off with the reason; it must not keep claiming to be on.
		DccSettings.set_pen_link_on(false)
		_on_pen_state("no_permission")

func _on_pen_state(s: String) -> void:
	_state = s
	if s != "connected":
		_battery = -1
		_firmware = ""
		_name = ""
	state_changed.emit(state())
	info_changed.emit()

func _on_pen_info(b: int, fw: String, nm: String) -> void:
	_battery = b if b >= 0 and b <= 100 else -1
	_firmware = fw
	_name = nm
	info_changed.emit()
	_maybe_chip()

## The low-battery chip: once per session, at or below LOW_BATTERY, if enabled. Shown on the status line (the shell's
## existing message slot) rather than a new widget; dismissed by the next status message. Returns whether it fired.
func _maybe_chip() -> bool:
	if _chip_shown or _battery < 0 or _battery > LOW_BATTERY or not DccSettings.pen_low_battery_chip():
		return false
	_chip_shown = true
	_say("Pen battery %d %%. Charge the Stylo 2." % _battery, "warn")
	return true

# -- tool tracking ------------------------------------------------------------------------------------------------

func _on_tool_armed(id: String) -> void:
	if id == _cur_tool:
		return
	_prev_tool = _cur_tool
	_cur_tool = id

## The tool armed before the current one, or "" when there is none. The "Last tool" target.
func prev_tool() -> String:
	return _prev_tool

# -- the misfire guard ----------------------------------------------------------------------------------------------

func _input(event: InputEvent) -> void:
	if event is InputEventMouseButton and (event as InputEventMouseButton).button_index == MOUSE_BUTTON_LEFT \
			and (event as InputEventMouseButton).device >= 0:
		_tip_down = (event as InputEventMouseButton).pressed
		if not _tip_down:
			_tip_up_ms = Time.get_ticks_msec()

## True when a double tap at `now_ms` should be ignored: the tip is on the glass, or lifted less than TAP_GUARD_MS ago.
func tap_guarded(now_ms: int) -> bool:
	return _tip_down or now_ms - _tip_up_ms < TAP_GUARD_MS

# -- tool kind ------------------------------------------------------------------------------------------------------

## "drawing", "placing" or "none" for the armed tool: the key of the per-kind setting rows. App.tool_kind is the
## one classification (an unknown tool id is a DRAWING tool by its own rule).
func kind_name() -> String:
	var id := "inspect" if app == null else String(app.armed_tool)
	match DccApp.tool_kind(id):
		ViewportHost.TOOL_NONE:
			return "none"
		ViewportHost.TOOL_PLACING:
			return "placing"
	return "drawing"

# -- double tap --------------------------------------------------------------------------------------------------------

func _on_double_tap() -> void:
	handle_double_tap(Time.get_ticks_msec())

## Runs the mapped action for a double tap at `now_ms`. Returns "" when it ran or the sentence saying why it did
## not (also shown in the status line: never a silent no-op). `now_ms` is a parameter so a probe sets the clock.
func handle_double_tap(now_ms: int) -> String:
	if tap_guarded(now_ms):
		_last_action = "double tap ignored (pen on the glass)"
		pen_event.emit("double_tap", _last_action)
		return _last_action
	var action := DccSettings.pen_double_tap(kind_name())
	var why := run_tap_action(action)
	_last_action = "double tap: " + action + ("" if why == "" else " (" + why + ")")
	pen_event.emit("double_tap", _last_action)
	if why != "":
		_say(why, "warn")
	return why

## Runs one double-tap action. "" on success, else the reason.
func run_tap_action(action: String) -> String:
	if app == null:
		return "The pen is not attached to the app."
	match action:
		"off":
			return ""
		"last_tool":
			if _prev_tool == "":
				return "No earlier tool to go back to."
			app.arm_tool(_prev_tool)
			return ""
		"undo":
			app.undo_last()
			return ""
		"eraser":
			var ws = app.call("_world_workspace") if app.has_method("_world_workspace") else null
			if ws == null or not ws.has_method("pen_toggle_erase"):
				return "This tool has no erase mode."
			return String(ws.pen_toggle_erase())
	if action.begins_with("pin:"):
		return DccQuickBar.run_pin_on(app, action.substr(4))
	return "Unknown pen action."

# -- slide ---------------------------------------------------------------------------------------------------------------

func _on_slide(pos: int) -> void:
	handle_slide(pos, Time.get_ticks_msec())

## One slide report. Returns the signed number of steps applied (0 for a new contact or inside the carry).
func handle_slide(pos: int, now_ms: int) -> int:
	var action := DccSettings.pen_slide(kind_name())
	pen_event.emit("slide", str(pos))
	if action == "off":
		_slide["last_pos"] = pos
		_slide["last_ms"] = now_ms
		return 0
	var per: int = int(POS_PER_STEP.get(action, 6))
	var steps := slide_steps(_slide, pos, now_ms, per)
	if DccSettings.pen_slide_invert():
		steps = -steps
	if steps == 0:
		return 0
	var why := run_slide_action(action, steps)
	_last_action = "slide: %s %+d" % [action, steps] + ("" if why == "" else " (" + why + ")")
	if why != "":
		_say(why, "warn")
	return steps

## PURE slide accumulator. `st` carries last_pos, last_ms and carry across calls. Relative, not absolute (proposal
## section 4.2): a report more than SLIDE_GAP_MS after the previous starts a new contact and applies nothing; within a
## contact the deltas accumulate and a step is emitted per `per` positions, so a finger that jitters by one or two
## positions around a point sums to ~0 and never moves the value (the dead band IS `per`). Returns the signed steps.
static func slide_steps(st: Dictionary, pos: int, now_ms: int, per: int) -> int:
	var fresh: bool = int(st["last_pos"]) < 0 or now_ms - int(st["last_ms"]) > SLIDE_GAP_MS
	st["last_ms"] = now_ms
	if fresh:
		st["last_pos"] = pos
		st["carry"] = 0
		return 0
	st["carry"] = int(st["carry"]) + pos - int(st["last_pos"])
	st["last_pos"] = pos
	var steps := int(st["carry"] / per)  # GDScript integer division truncates toward zero
	st["carry"] = int(st["carry"]) - steps * per
	return steps

## Applies `steps` (signed) of a slide action through its existing entry point. "" on success, else the reason.
func run_slide_action(action: String, steps: int) -> String:
	if app == null:
		return "The pen is not attached to the app."
	var dir := 1 if steps > 0 else -1
	match action:
		"brush_size":
			var ws = app.call("_world_workspace") if app.has_method("_world_workspace") else null
			if ws == null or not ws.has_method("pen_step_size"):
				return "This tool has no brush size."
			for k in absi(steps):
				var why := String(ws.pen_step_size(dir))
				if why != "":
					return why
			return ""
		"zoom":
			if app.viewport == null:
				return "No map to zoom."
			for k in absi(steps):
				app.viewport.zoom_step(ViewportHost.ZOOM_BUTTON_STEP if dir > 0 else 1.0 / ViewportHost.ZOOM_BUTTON_STEP)
			return ""
		"year":
			if not app.tl_available():
				return DccShell.TL_UNAVAILABLE
			app.tl_set_year(app.tl_year() + steps * YEAR_STEP)
			return ""
	return "Unknown pen action."

func _say(text: String, _token: String = "") -> void:
	if app != null and app.has_method("set_status"):
		app.set_status("hint", text, "text_ghost")
