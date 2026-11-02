## DRAFT, never loaded by Godot (it sits outside `godot-project/`). SH-9 of
## `STYLUS_HAPTICS_RESEARCH.md` §8.1: a self-contained stylus tester and
## calibration screen, in the spirit of Krita's tablet tester.
##
## Owner, 2026-11-02: the haptics feedback is parked; this is the stylus **input**
## work, which sits within standard permissions. This screen reads only ordinary
## pointer input — pressure, tilt, hover, the eraser flag, buttons, the report
## rate and the device id — exactly what Godot already delivers for a pen. It asks
## for no permission, touches no engine code, and writes nothing.
##
## WHY IT EXISTS. Two uses:
##   1. The owner opens it on the OnePlus Pad 3 with the Stylo 2 and SEES what the
##      pen actually reports: the pressure range, whether tilt is ever non-zero,
##      whether hover (motion with no contact) arrives, whether the eraser end is
##      flagged, the report rate, and the device id. That is SH-0's "what does the
##      pen report" measurement, got without adb (`STYLUS_HAPTICS_RESEARCH.md`
##      §8.2). The research warns, from the Godot 4.7.2 source, that the eraser
##      flag, the pen's buttons and the hover distance are DROPPED on Android;
##      this screen is how we confirm that on the real device rather than assume.
##   2. It is the calibration surface SH-4's pressure curve (soft/linear/firm) and
##      SH-5's tilt option will preview against, once those land.
##
## WHAT IT MUST NEVER DO:
##   - talk to the engine, the map or any world state (it is a diagnostic);
##   - change any `DccSettings` value (SH-4 adds the curve control; this only reads
##     it, when it exists, to preview — not in this draft);
##   - assume a property exists on an event class it does not: every read is
##     guarded, because `InputEventScreenTouch` carries no `pressure`/`tilt` and a
##     mouse carries no `index`.
##
## HOW A PEN REACHES HERE (read at the symbols, `map_overlay.gd::_emit_stroke_sample`,
## 2026-11-02). On this project an Android pen arrives as an EMULATED MOUSE:
## `InputEventMouseMotion` with `.pressure` (0..1), `.tilt` (Vector2, each axis the
## sine of the tilt angle, -1..1), `.pen_inverted` (the eraser end), and
## `.button_mask`. It can also arrive as `InputEventScreenDrag` with the same pen
## fields (Godot 4.3+), and a bare touch as `InputEventScreenTouch`. This screen
## listens for all three so it shows the truth whatever path the device uses.
##
## HOW TO WIRE IT (for the shipped tree, not in this draft):
##   - add a row **Help ▸ Stylus tester…** (or Preferences ▸ Touch & gestures), in
##     `menus.gd`, that instances this and shows it in a `Window`/`AcceptDialog`,
##     the same way other tool windows are shown;
##   - give it a `command_index.gd` EXTRAS alias "Stylus tester" so Find reaches it.
##   It needs nothing else: it is a leaf `Control`.
##
## VERIFY (on the device, since no pen exists on the desktop host): open it on the
## Pad 3, write and hover with the Stylo 2, and read the live values. On the
## desktop a mouse shows pressure 1.0, tilt (0,0), no hover distinction — which is
## itself the control that proves the readout works.
extends Control

## The readout labels, built in `_ready()`. Keyed by field name so `_set` can
## write one without re-finding it. Never holds a reference to anything outside
## this node.
var _rows: Dictionary = {}

## Rolling report-rate state: event timestamps (ms) within the last second. The
## rate is the count. Cheap, and it needs no fixed buffer.
var _event_times: Array[int] = []

## Observed extremes over the session, so a quick press-and-release shows the
## pen's real pressure span rather than one instant. Reset by the Reset button.
var _p_min := 2.0
var _p_max := -1.0
var _rate_max := 0
var _tilt_seen := false
var _hover_seen := false
var _eraser_seen := false
var _last_class := "—"
var _last_device := -1

## The live sample, for `_draw()`: position (local), pressure (or -1 for none),
## tilt vector, whether a button is down. Omitted fields stay at these neutrals;
## "no value" is never drawn as a plausible value (a bare 0 pressure).
var _pos := Vector2.ZERO
var _pressure := -1.0
var _tilt := Vector2.ZERO
var _down := false

## The colours. Taken from `DccTheme` where it is available, so the screen matches
## the shell; plain fallbacks otherwise, because a tester must render even if it is
## opened standalone. `DccTheme` is the shell's theme singleton (`dcc_theme.gd`).
var _ink := Color(0.91, 0.92, 0.93)
var _dim := Color(0.55, 0.57, 0.59)
var _accent := Color(0.88, 0.64, 0.29)
var _panel := Color(0.07, 0.08, 0.09)

func _ready() -> void:
	name = "StylusTester"
	mouse_filter = Control.MOUSE_FILTER_STOP
	set_anchors_preset(Control.PRESET_FULL_RECT)
	_pull_theme()
	_build_ui()
	set_process(true)

## Reads the shell's theme tokens so the screen matches the shell. `DccTheme` is a
## `class_name` with STATIC accessors (`c(token)`), not an autoload node, so it is
## called directly as every shipped file does (`DccTheme.c("accent")`). The token
## strings are the real palette keys in `dcc_theme.gd`'s dark/light dictionaries
## (`text`, `text_secondary`, `accent`, `panel`), confirmed 2026-11-02; `c()`
## returns magenta for an unknown token, which reads as "a token was renamed, fix
## me" rather than crashing. The literals above are the standalone fallback only
## if this draft is ever pasted into a project without `DccTheme`.
func _pull_theme() -> void:
	_ink = DccTheme.c("text")
	_dim = DccTheme.c("text_secondary")
	_accent = DccTheme.c("accent")
	_panel = DccTheme.c("panel")

## Builds a column of labelled readouts plus a Reset button. Pure Godot Controls,
## no project widgets, so the draft pastes and runs with nothing else present.
func _build_ui() -> void:
	var root := VBoxContainer.new()
	root.set_anchors_preset(Control.PRESET_TOP_LEFT)
	root.position = Vector2(16, 16)
	root.add_theme_constant_override("separation", 4)
	add_child(root)

	var title := Label.new()
	title.text = "STYLUS TESTER"
	title.add_theme_color_override("font_color", _accent)
	root.add_child(title)

	var hint := Label.new()
	hint.text = "Write and hover with the pen. A mouse reads pressure 1.0, no tilt, no hover."
	hint.add_theme_color_override("font_color", _dim)
	root.add_child(hint)

	for field in ["event", "device", "pressure", "pressure span", "tilt",
			"tilt angle", "eraser", "hover", "buttons", "rate Hz", "rate max"]:
		var line := Label.new()
		line.add_theme_color_override("font_color", _ink)
		_rows[field] = line
		root.add_child(line)
		_set(field, "—")

	var reset := Button.new()
	reset.text = "Reset observed span"
	reset.pressed.connect(_reset_observed)
	root.add_child(reset)

## Writes one readout row. Named so a caller never pokes `_rows` directly.
func _set(field: String, value: String) -> void:
	if _rows.has(field):
		(_rows[field] as Label).text = "%s: %s" % [field, value]

## Clears the observed extremes. The live readouts keep updating.
func _reset_observed() -> void:
	_p_min = 2.0
	_p_max = -1.0
	_rate_max = 0
	_tilt_seen = false
	_hover_seen = false
	_eraser_seen = false

## Every input event this control sees. It reads the three event classes a pen can
## arrive as and never assumes a field exists on the wrong one. It consumes
## nothing it does not understand, so a tester embedded in a window still lets the
## window work; but it is normally shown full-screen.
func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var mm := event as InputEventMouseMotion
		var down := (mm.button_mask & MOUSE_BUTTON_MASK_LEFT) != 0
		_sample("MouseMotion", mm.device, mm.position, mm.pressure, mm.tilt,
			mm.pen_inverted, down, true)
	elif event is InputEventScreenDrag:
		var sd := event as InputEventScreenDrag
		## ScreenDrag carries the pen fields on Godot 4.3+; a drag is always "down".
		_sample("ScreenDrag#%d" % sd.index, sd.device, sd.position, sd.pressure,
			sd.tilt, sd.pen_inverted, true, true)
	elif event is InputEventScreenTouch:
		var st := event as InputEventScreenTouch
		## ScreenTouch has NO pressure or tilt: pass the no-value sentinels, and let
		## `_sample` render them as "—" rather than a fake 0.
		_sample("ScreenTouch#%d" % st.index, st.device, st.position, -1.0,
			Vector2.ZERO, false, st.pressed, false)

## Folds one sample into the live state and the observed extremes, updates the
## readouts, and redraws the visualisation. `has_pen_fields` is false for a bare
## touch, which has no pressure or tilt to believe.
func _sample(klass: String, device: int, pos: Vector2, pressure: float,
		tilt: Vector2, eraser: bool, down: bool, has_pen_fields: bool) -> void:
	_last_class = klass
	_last_device = device
	_pos = pos
	_down = down

	# -- report rate: timestamps within the last second --
	var now := Time.get_ticks_msec()
	_event_times.append(now)
	while _event_times.size() > 0 and now - _event_times[0] > 1000:
		_event_times.remove_at(0)
	var rate := _event_times.size()
	_rate_max = maxi(_rate_max, rate)

	# -- pressure: a pen reports 0<p<1; a mouse a flat 1.0; a bare touch none --
	if has_pen_fields and pressure >= 0.0:
		_pressure = pressure
		if down:
			_p_min = minf(_p_min, pressure)
			_p_max = maxf(_p_max, pressure)
		_set("pressure", "%.4f" % pressure)
	else:
		_pressure = -1.0
		_set("pressure", "—")

	# -- tilt: each axis is sin(tilt_angle); the angle from vertical is below --
	if has_pen_fields and tilt != Vector2.ZERO:
		_tilt = tilt
		_tilt_seen = true
		var sine := clampf(tilt.length(), 0.0, 1.0)
		_set("tilt", "(%.3f, %.3f)" % [tilt.x, tilt.y])
		_set("tilt angle", "%.1f deg from vertical" % rad_to_deg(asin(sine)))
	else:
		_tilt = Vector2.ZERO
		_set("tilt", "(0, 0)" if has_pen_fields else "—")
		_set("tilt angle", "—")

	# -- hover: motion with no contact. The research expects Android to deliver it
	#    as plain motion with no pressure; this confirms it on the device. --
	var hovering := has_pen_fields and not down
	if hovering:
		_hover_seen = true
	if eraser:
		_eraser_seen = true

	_set("event", klass)
	_set("device", str(device))
	_set("pressure span", ("%.4f .. %.4f" % [_p_min, _p_max]) if _p_max >= 0.0 else "—")
	_set("eraser", ("yes" if eraser else "no") + ("  (seen this session)" if _eraser_seen else ""))
	_set("hover", ("hovering" if hovering else ("contact" if down else "—"))
		+ ("  (hover seen)" if _hover_seen else ""))
	_set("buttons", "down" if down else "up")
	_set("rate Hz", str(rate))
	_set("rate max", str(_rate_max))
	queue_redraw()

## Draws the live visualisation on the right: a pressure bar, a tilt needle, and a
## ring at the pointer that fills with pressure. Pure drawing; reads only the live
## state above.
func _draw() -> void:
	var r := size
	if r.x <= 0.0 or r.y <= 0.0:
		return
	# pressure bar, bottom-right
	var bar := Rect2(r.x - 56.0, 80.0, 24.0, r.y - 160.0)
	draw_rect(bar, _panel)
	draw_rect(bar, _dim, false, 1.0)
	if _pressure >= 0.0:
		var h := bar.size.y * clampf(_pressure, 0.0, 1.0)
		draw_rect(Rect2(bar.position.x, bar.end.y - h, bar.size.x, h), _accent)
	# tilt needle, from a fixed origin
	var origin := Vector2(r.x - 120.0, r.y - 120.0)
	draw_circle(origin, 48.0, _panel)
	draw_arc(origin, 48.0, 0.0, TAU, 48, _dim, 1.0)
	if _tilt != Vector2.ZERO:
		draw_line(origin, origin + _tilt * 48.0, _accent, 2.0)
	# pointer ring that fills with pressure, where the pen is
	if _pos != Vector2.ZERO:
		draw_arc(_pos, 18.0, 0.0, TAU, 32, _ink, 1.5)
		if _pressure > 0.0:
			draw_arc(_pos, 18.0, -PI * 0.5, -PI * 0.5 + TAU * clampf(_pressure, 0.0, 1.0),
				24, _accent, 3.0)
