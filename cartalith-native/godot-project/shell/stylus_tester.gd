class_name StylusTester
extends Control
## SH-9 (`STYLUS_HAPTICS_RESEARCH.md` section 8.1), in the spirit of Krita's tablet tester: a diagnostic that shows what
## the pointer actually reports -- pressure, tilt, tilt angle, the eraser flag, hover, buttons, report rate, the device
## id, the pressure span seen, and the last key and pan/magnify gesture. It is the owner's way to SEE what the Stylo 2
## sends on the Pad 3 (SH-0's "what does the pen report", without adb). Transplanted from
## `drafts/stylus-input/00_stylus_tester.gd` (Ruling EE: standard pointer input only).
##
## WHAT IT MUST NEVER DO: talk to the engine, the map or any world; write a `DccSettings` value; or show "no value" as
## a plausible number. A field the event class does not carry, or that Android's port never fills, reads "—"; a
## mouse's flat 1.0 is labelled as a mouse-like reading, never as a pen's pressure.
##
## HOW A POINTER REACHES HERE (`STYLUS_HAPTICS_RESEARCH.md` section 4.2, `StylusInput`): a pen and a mouse arrive as
## `InputEventMouseMotion` / `Button` with device >= 0; a finger as touch emulation (device -1) and as
## `InputEventScreenTouch/Drag`. Pressure, tilt (each axis the sine of the tilt angle) and `pen_inverted` live on the
## motion events; Android's port never sets `pen_inverted` and gives hover no fields, so those rows staying "no" and
## "—" on the Pad 3 is itself the measurement.

## Readout rows in display order. Each is a Label in `_rows`, keyed by name so a caller never pokes the tree.
const FIELDS := ["event", "source", "device", "pressure", "pressure span", "tilt", "tilt angle", "eraser flag",
	"hover", "contact", "rate Hz", "rate max", "last key", "gestures", "pen link", "pen battery", "pen firmware",
	"pen last", "sculpt maps", "paint maps", "colour maps"]

var _rows: Dictionary = {}
var _values: Dictionary = {}          ## field -> its current text; what `snapshot()` returns
var _event_times: Array[int] = []     ## ms timestamps within the last second: the rate is the count
var _p_min := 2.0                     ## pressure extremes seen while in contact; 2 / -1 are "none yet", never shown
var _p_max := -1.0
var _rate_max := 0
var _pen_link: PenLink = null         ## Ruling EK: the facade whose state the "pen ..." rows show; null when not bound
var _hover_seen := false
var _eraser_seen := false
var _pan_n := 0
var _mag_n := 0
## The live sample for `_draw`: position, pressure (-1 = none, never drawn as a bar), tilt, contact.
var _pos := Vector2.ZERO
var _has_pos := false
var _pressure := -1.0
var _tilt := Vector2.ZERO
var _down := false
var _ink := Color.WHITE
var _dim := Color.GRAY
var _accent := Color.ORANGE
var _panel := Color.BLACK

func _ready() -> void:
	name = "StylusTester"
	mouse_filter = Control.MOUSE_FILTER_STOP
	focus_mode = Control.FOCUS_ALL
	set_anchors_preset(Control.PRESET_FULL_RECT)
	_ink = DccTheme.c("text")
	_dim = DccTheme.c("text_secondary")
	_accent = DccTheme.c("accent")
	_panel = DccTheme.c("panel")
	_build_ui()

## A column of labelled readouts and a Reset button; plain Controls, so the tester has no dependency on any panel.
func _build_ui() -> void:
	var root := VBoxContainer.new()
	root.position = Vector2(16, 16)
	root.add_theme_constant_override("separation", 4)
	add_child(root)
	var title := Label.new()
	title.text = "STYLUS TESTER"
	title.add_theme_color_override("font_color", _accent)
	root.add_child(title)
	var hint := Label.new()
	hint.text = "Write and hover with the pen. A mouse reads a flat 1.0 held and no tilt; a finger is device -1."
	hint.add_theme_color_override("font_color", _dim)
	hint.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	hint.custom_minimum_size = Vector2(420, 0)
	root.add_child(hint)
	for f in FIELDS:
		var line := Label.new()
		line.add_theme_color_override("font_color", _ink)
		_rows[f] = line
		root.add_child(line)
		_put(f, "—")
	for k in ["sculpt", "paint", "colour"]:
		_put(k + " maps", DccSettings.pressure_effect_text(k, -1.0))
	var reset := Button.new()
	reset.text = "Reset observed span"
	reset.pressed.connect(reset_observed)
	root.add_child(reset)

## Ruling EK: shows the Stylo 2 Pen link in the four "pen ..." rows. A value the link does not have reads "—" followed
## by the reason (never `0 %`). Reads the facade only; it never starts the link or changes a setting.
func bind_pen_link(link: PenLink) -> void:
	_pen_link = link
	if link == null:
		for f in ["pen link", "pen battery", "pen firmware"]:
			_put(f, "— (no Pen link in this window)")
		return
	link.state_changed.connect(func(_s): _show_pen_link())
	link.info_changed.connect(_show_pen_link)
	link.pen_event.connect(func(_k, _d): _put("pen last", link.last_action() if link.last_action() != "" else _d))
	_show_pen_link()

func _show_pen_link() -> void:
	var link := _pen_link
	if link == null:
		return
	var why := link.reason()
	_put("pen link", link.state() if why == "" else "%s (%s)" % [link.state(), why])
	_put("pen battery", "%d %%" % link.battery() if link.battery() >= 0 else "— (%s)" % (why if why != "" else "not reported"))
	_put("pen firmware", link.firmware() if link.firmware() != "" else "— (%s)" % (why if why != "" else "not reported"))

## Writes one readout row and records it for `snapshot()`.
func _put(field: String, value: String) -> void:
	_values[field] = value
	if _rows.has(field):
		(_rows[field] as Label).text = "%s: %s" % [field, value]

## The readouts as text, by field; what a probe (or a bug report) reads instead of the Labels.
func snapshot() -> Dictionary:
	return _values.duplicate()

## Clears the observed extremes; the live readouts keep updating.
func reset_observed() -> void:
	_p_min = 2.0
	_p_max = -1.0
	_rate_max = 0
	_hover_seen = false
	_eraser_seen = false
	_pan_n = 0
	_mag_n = 0
	_put("pressure span", "—")
	_put("rate max", "—")
	_put("gestures", "—")

## Every input event this control sees. It reads the pointer classes a pen can arrive as and the gestures and keys
## that matter to pen work, never assuming a field exists on the wrong class.
func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var mm := event as InputEventMouseMotion
		feed("MouseMotion", mm.device, mm.position, mm.pressure, mm.tilt, mm.pen_inverted,
			(mm.button_mask & MOUSE_BUTTON_MASK_LEFT) != 0, true)
	elif event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		## A button event carries no pressure or tilt: a press is contact, a release is not.
		feed("MouseButton", mb.device, mb.position, 0.0, Vector2.ZERO, false, mb.pressed, false)
	elif event is InputEventScreenDrag:
		var sd := event as InputEventScreenDrag
		feed("ScreenDrag#%d" % sd.index, sd.device, sd.position, sd.pressure, sd.tilt, sd.pen_inverted, true, true)
	elif event is InputEventScreenTouch:
		var st := event as InputEventScreenTouch
		feed("ScreenTouch#%d" % st.index, st.device, st.position, 0.0, Vector2.ZERO, false, st.pressed, false)
	elif event is InputEventPanGesture:
		_pan_n += 1
		_put("gestures", "pan %d, magnify %d" % [_pan_n, _mag_n])
	elif event is InputEventMagnifyGesture:
		_mag_n += 1
		_put("gestures", "pan %d, magnify %d" % [_pan_n, _mag_n])
	elif event is InputEventKey and (event as InputEventKey).pressed:
		var k := event as InputEventKey
		_put("last key", "keycode %d, physical %d%s" % [k.keycode, k.physical_keycode,
			"  (KEY_UNKNOWN: a key Godot's table has no name for)" if k.keycode == KEY_UNKNOWN else ""])

## Folds one pointer sample into the readouts. `has_fields` is false for an event class that carries no pressure or
## tilt (a button, a touch), whose pressure and tilt are then "—" -- not 0 and not 1. Public so a probe can feed it.
func feed(klass: String, device: int, pos: Vector2, pressure: float, tilt: Vector2, inverted: bool, down: bool,
		has_fields: bool) -> void:
	_pos = pos
	_has_pos = true
	_down = down
	var now := Time.get_ticks_msec()
	_event_times.append(now)
	while _event_times.size() > 0 and now - _event_times[0] > 1000:
		_event_times.remove_at(0)
	var rate := _event_times.size()
	_rate_max = maxi(_rate_max, rate)

	var finger := StylusInput.is_finger(device)
	var signature := has_fields and StylusInput.has_pen_signature(pressure, tilt, inverted)
	var source := "—"
	if finger:
		source = "finger (device -1, touch emulation)"
	elif signature:
		source = "pen (pressure strictly between 0 and 1, tilt or eraser flag)"
	elif has_fields and down:
		source = "mouse-like (a flat 1.0 held, no tilt) -- or a pen pressed to exactly 1.0 with no tilt"
	elif has_fields:
		source = "pointer hovering (Android gives hover no fields)"
	else:
		source = "button or touch event (carries no pen fields)"

	# pressure: only where the event class carries one, and labelled for what it is
	_pressure = -1.0
	if not has_fields:
		_put("pressure", "—")
	elif finger:
		_pressure = pressure
		_put("pressure", "%.4f  (the finger's own, not a pen's)" % pressure)
	elif signature:
		_pressure = pressure
		_put("pressure", "%.4f" % pressure)
		if down:
			_p_min = minf(_p_min, pressure)
			_p_max = maxf(_p_max, pressure)
	elif down:
		_put("pressure", "%.4f  (mouse-like, not a pen reading)" % pressure)
	else:
		_put("pressure", "—  (hover: none reported)")

	# Ruling EM item 3: what this reading does on each tool under the stored Pressure controls choice. Only a pen
	# reading counts as pressure; everything else is "no pen pressure", so the tester never invents a multiplier.
	var pen_p := pressure if (signature and not finger) else -1.0
	_put("sculpt maps", DccSettings.pressure_effect_text("sculpt", pen_p))
	_put("paint maps", DccSettings.pressure_effect_text("paint", pen_p))
	_put("colour maps", DccSettings.pressure_effect_text("colour", pen_p))

	# tilt: each axis is sin(angle from vertical)
	_tilt = Vector2.ZERO
	if has_fields and not finger and tilt != Vector2.ZERO:
		_tilt = tilt
		_put("tilt", "(%.3f, %.3f)" % [tilt.x, tilt.y])
		_put("tilt angle", "%.1f deg from vertical" % rad_to_deg(asin(clampf(tilt.length(), 0.0, 1.0))))
	elif has_fields and not finger and signature:
		_put("tilt", "(0, 0)  (none reported)")
		_put("tilt angle", "—")
	else:
		_put("tilt", "—")
		_put("tilt angle", "—")

	var hovering := has_fields and not down and not finger
	if hovering:
		_hover_seen = true
	if inverted and not finger:
		_eraser_seen = true
	_put("event", klass)
	_put("source", source)
	_put("device", str(device))
	_put("pressure span", ("%.4f .. %.4f" % [_p_min, _p_max]) if _p_max >= 0.0 else "—")
	_put("eraser flag", ("yes" if inverted else "no") + ("  (seen this session)" if _eraser_seen
		else "  (Android's port never sets it)"))
	_put("hover", ("hovering" if hovering else "no") + ("  (hover seen this session)" if _hover_seen else ""))
	_put("contact", "down" if down else "up")
	_put("rate Hz", str(rate))
	_put("rate max", str(_rate_max))
	queue_redraw()

## A pressure bar, a tilt needle and a ring at the pointer that fills with pressure. Pure drawing from the live state;
## no bar is drawn for "no pressure".
func _draw() -> void:
	var r := size
	if r.x <= 0.0 or r.y <= 0.0:
		return
	var bar := Rect2(r.x - 56.0, 80.0, 24.0, maxf(r.y - 160.0, 10.0))
	draw_rect(bar, _panel)
	draw_rect(bar, _dim, false, 1.0)
	if _pressure >= 0.0:
		var h := bar.size.y * clampf(_pressure, 0.0, 1.0)
		draw_rect(Rect2(bar.position.x, bar.end.y - h, bar.size.x, h), _accent)
	var origin := Vector2(r.x - 120.0, r.y - 120.0)
	draw_circle(origin, 48.0, _panel)
	draw_arc(origin, 48.0, 0.0, TAU, 48, _dim, 1.0)
	if _tilt != Vector2.ZERO:
		draw_line(origin, origin + _tilt * 48.0, _accent, 2.0)
	if _has_pos:
		draw_arc(_pos, 18.0, 0.0, TAU, 32, _ink, 1.5)
		if _pressure > 0.0:
			draw_arc(_pos, 18.0, -PI * 0.5, -PI * 0.5 + TAU * clampf(_pressure, 0.0, 1.0), 24, _accent, 3.0)

## Opens the tester in its own window under `parent`. Used by `Preferences > Stylus > Stylus tester...`; the window
## frees itself on close. Returns the tester so a caller can read `snapshot()`.
static func open_in_window(parent: Node) -> StylusTester:
	var w := Window.new()
	w.title = "Stylus tester"
	w.size = Vector2i(560, 600)
	w.transient = true
	w.close_requested.connect(w.queue_free)
	var t := StylusTester.new()
	w.add_child(t)
	parent.add_child(w)
	## Bound AFTER the window is in the tree: `_ready` builds the rows and writes their dashes, which would
	## otherwise overwrite what the binding just showed.
	t.bind_pen_link(parent.get("pen_link") as PenLink)
	w.popup_centered()
	t.grab_focus()
	return t
