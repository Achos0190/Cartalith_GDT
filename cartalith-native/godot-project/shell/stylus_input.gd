class_name StylusInput
extends RefCounted
## Standard stylus input helpers (Ruling EE: pressure, tilt, hover and touch as Godot 4.7.2 delivers them; no
## plugin, no pen SDK, no haptics). Pure statics plus one small piece of process state, so every rule here is
## testable without a pen and without a window.
##
## WHAT GODOT DELIVERS, read at the source (`STYLUS_HAPTICS_RESEARCH.md` section 4.2, `STATUS.md`'s MS-5 entry):
##   - On Android a pen (TOOL_TYPE_STYLUS or ERASER) and a real mouse are both `InputEventMouse*` events whose
##     `device` is left at the default 0. A finger is `InputEventScreenTouch/Drag` plus Godot's own touch-to-mouse
##     emulation, whose `device` is `DEVICE_ID_EMULATION` (-1). So `device >= 0` means "a pen or a mouse", and
##     `device < 0` means "a finger". That is the only finger-versus-pen signal standard input has; certain
##     detection needs TOOL_TYPE, which the engine does not pass on (and the Kotlin plugin that would is off,
##     Ruling EE). A mouse on an Android device therefore reads as a pen: a labelled limit, not a defect.
##   - `pen_inverted` is never set by the Android port, and hover arrives as a motion event with a position and no
##     pressure or tilt. So "pen near the glass" cannot be read from a field; it is read from RECENT EVENTS.
##
## MUST NEVER: encode "no pressure" as a plausible number (an absent value is `null` here and at every caller), talk
## to the engine, or read a world. Settings live in `DccSettings`; this file holds none.

## The palm rule's window, ms: a finger that lands this soon after the last pen event (contact OR hover) is a
## palm and is ignored. SH-3's own figure ("ignores touches during pen contact and for 300 ms after, and while the
## pen hovers", `STYLUS_HAPTICS_RESEARCH.md` section 8.1): a labelled judgement, since hover exit is not
## distinguished on Android, so "still hovering" has to mean "an event within this window". Hover reports
## continuously while the pen is in range, so a pen that has gone leaves a gap longer than this.
const PALM_MS := 300

## Test seam only: probes cannot make `OS.has_feature("mobile")` true on a desktop, so they set this to exercise
## the Android branches. Never set in shipped code.
static var assume_mobile := false

## `Input.use_accumulated_input` as it was before a pen contact switched it off, or -1 when no contact owns it.
static var _accum_before := -1

## Whether this build is a touch device whose pointer events follow Android's device convention.
static func is_mobile() -> bool:
	return assume_mobile or OS.has_feature("mobile")

## True for a finger: Godot's touch-to-mouse emulation stamps `DEVICE_ID_EMULATION` (-1). The one test every
## finger-versus-pen decision uses, so no caller restates the convention.
static func is_finger(device: int) -> bool:
	return device < 0

## ET-D2's pen signature on a motion sample: a pressure strictly between 0 and 1, a non-zero tilt, or the eraser
## flag. A real mouse reports exactly 1.0 held and no tilt (measured in Godot 4.7.1, `map_overlay.gd`'s
## `_stroke_pen_seen`), so it never satisfies this.
static func has_pen_signature(pressure: float, tilt: Vector2, inverted: bool) -> bool:
	return (pressure > 0.0 and pressure < 1.0) or tilt != Vector2.ZERO or inverted

## Whether a mouse-class event is a pen (or a pen-or-mouse the build cannot tell apart) rather than a finger.
## A finger never is one (SH-2's fix: Android passes the finger's own `getPressure`, so a light finger drag
## strictly between 0 and 1 used to latch as a pen). On a touch build any non-finger pointer counts, because the
## Android port gives hover no fields to test; elsewhere only a motion that carries the signature does, so a
## desktop mouse never counts. A button event has no pen fields, so off a touch build it never counts.
static func is_pen_event(ev: InputEventMouse) -> bool:
	if is_finger(ev.device):
		return false
	if is_mobile():
		return true
	if ev is InputEventMouseMotion:
		var mm := ev as InputEventMouseMotion
		return has_pen_signature(mm.pressure, mm.tilt, mm.pen_inverted)
	return false

## The pressure a motion sample carries, or `null` when it carries none a person made: a finger (its pressure is
## the finger's, not a pen's), or a sample with no pen signature (a mouse's flat 1.0, a hover's absent value).
## `null` is "no value"; it is never 1.0 and never 0.0.
static func pen_pressure(mm: InputEventMouseMotion) -> Variant:
	if is_finger(mm.device) or not has_pen_signature(mm.pressure, mm.tilt, mm.pen_inverted):
		return null
	return mm.pressure

## SH-3's palm test: whether a finger landing at `now_ms` falls inside the palm window of the last pen event at
## `last_pen_ms`. `last_pen_ms < 0` means no pen event yet (never a window), so a device that has not yet seen a
## pen is never "palm-blocked". Boundary: exactly `PALM_MS` after is still inside; one ms later is outside.
static func in_palm_window(now_ms: int, last_pen_ms: int) -> bool:
	return last_pen_ms >= 0 and now_ms - last_pen_ms <= PALM_MS

## SH-2: a pen contact began on a touch build, so deliver every sample, not one per frame. Godot's default merges
## the frame's motion events and keeps the OLDER event's pressure and tilt (`InputEventMouseMotion::accumulate`,
## `STYLUS_HAPTICS_RESEARCH.md` section 4.2), which flattens a fast stroke. Only for a non-finger press on a
## touch build, so a desktop mouse and a finger are exactly as before, and only while the contact lasts:
## `end_pen_contact()` restores what was there. Idempotent while a contact is open.
static func begin_pen_contact(device: int) -> void:
	if is_finger(device) or not is_mobile() or _accum_before != -1:
		return
	_accum_before = 1 if Input.use_accumulated_input else 0
	Input.use_accumulated_input = false

## Restores `Input.use_accumulated_input` after `begin_pen_contact`. A no-op when no contact owns it.
static func end_pen_contact() -> void:
	if _accum_before == -1:
		return
	Input.use_accumulated_input = _accum_before == 1
	_accum_before = -1

## Whether a pen contact currently owns the accumulation switch (what a probe reads).
static func pen_contact_open() -> bool:
	return _accum_before != -1
