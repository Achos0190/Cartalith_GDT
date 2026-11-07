## DRAFT, never loaded by Godot (it sits outside `godot-project/`). The
## `DccSettings` additions for the stylus **input** work (Ruling EE: standard
## input only — pressure and touch). Per-install viewing/interaction preferences,
## exactly like Handedness and Theme above them: they persist across a restart and
## **never travel in a world `.zip`**.
##
## HOW TO APPLY: paste the block below into `shell/dcc_settings.gd`, after the
## `_SEC_HANDEDNESS` accessors (`dominant_hand`/`set_dominant_hand`), so the new
## section sits with the other interaction preferences. It mirrors that pair's
## shape exactly: a `_SEC_*` const, a validated getter with `_ensure_loaded()`, and
## a setter that ignores an unknown value and `_save()`s.
##
## WHAT EACH KEY IS:
##   - `pressure_curve` (SH-4): "soft" / "linear" / "firm" — the gamma the live
##     pen's pressure is remapped through BEFORE it reaches the engine. Default
##     "linear" = identity, so turning the feature on changes nothing until the
##     user picks a curve. `10_pressure_curve.gd` reads it.
##   - `pressure_max` (SH-4, "comfortable maximum"): the raw pressure that should
##     read as full, 0.5..1.0, default 1.0 (no change). A pen a user never presses
##     past ~0.7 then reaches full strength at that 0.7. `10_pressure_curve.gd`
##     reads it.
##   - `pen_draws_finger_pans` (SH-3): while true and a pen is in use, the finger
##     only navigates (Ruling EC / `MOBILE_SHELL_PROPOSAL.md` §4). Default false.
##     `20_pen_draws_finger_pans.gd` reads it.
##   - `tilt_to_size` (SH-5): whether the brush reads pen tilt. Default false,
##     because tilt reaching the paint engine is itself the SH-5 Rust change; this
##     only exposes the toggle. The GDScript side reads it; the engine field is a
##     separate parity-gated change.
##
## MUST NEVER: hold anything a world needs, or anything secret — these are plain
## interaction preferences. Never store an unknown enum value (the setter drops
## it), the "no value is not a plausible value" rule this file already follows.
##
## VERIFY: `godot --headless --check-only shell/dcc_settings.gd` after pasting;
## then that a round-trip survives a restart (set firm, reopen, read firm).

## --- paste into shell/dcc_settings.gd, by the other interaction sections ---

## The stylus input preferences (Ruling EE, 2026-11-02; `STYLUS_HAPTICS_RESEARCH.md`
## §8.1). Per install, never a world's: a `.zip` carrying "firm pressure, pen-only
## drawing" would reshape another person's input. Standard pointer input only.
const _SEC_STYLUS := "stylus"

## The pen pressure response curves and their gamma. "linear" is the identity, so
## it is the default and the control's neutral position. The 0.6 / 1.6 pair is a
## labelled judgement matching the research's SH-4 (`soft` eases light pressure in,
## `firm` holds back until a harder press), not a measurement.
const PRESSURE_CURVES := {"soft": 0.6, "linear": 1.0, "firm": 1.6}

## The pen pressure response curve: "soft" / "linear" / "firm". An unset or unknown
## value reads as "linear" (identity), never a guessed curve.
static func pen_pressure_curve() -> String:
	_ensure_loaded()
	var cv := String(_cfg.get_value(_SEC_STYLUS, "pressure_curve", "linear"))
	return cv if PRESSURE_CURVES.has(cv) else "linear"

## Stores the curve and saves; an unknown name is ignored, not stored.
static func set_pen_pressure_curve(curve: String) -> void:
	if not PRESSURE_CURVES.has(curve):
		return
	_ensure_loaded()
	_cfg.set_value(_SEC_STYLUS, "pressure_curve", curve)
	_save()

## The raw pressure that reads as full strength, clamped to 0.5..1.0. 1.0 is the
## default and the identity (the pen must reach full pressure for full strength).
## The floor 0.5 keeps the mapping from exploding light touches to full.
static func pen_pressure_max() -> float:
	_ensure_loaded()
	return clampf(float(_cfg.get_value(_SEC_STYLUS, "pressure_max", 1.0)), 0.5, 1.0)

## Stores the comfortable maximum and saves, clamped to the valid range.
static func set_pen_pressure_max(value: float) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_STYLUS, "pressure_max", clampf(value, 0.5, 1.0))
	_save()

## Whether, with a pen in use, the finger only navigates (SH-3, Ruling EC). Default
## false: the shipped behaviour (the pen and finger both draw) is unchanged until
## the user turns this on.
static func pen_draws_finger_pans() -> bool:
	_ensure_loaded()
	return bool(_cfg.get_value(_SEC_STYLUS, "pen_draws_finger_pans", false))

## Stores the pen-only-drawing preference and saves.
static func set_pen_draws_finger_pans(on: bool) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_STYLUS, "pen_draws_finger_pans", on)
	_save()

## Whether the brush reads pen tilt (SH-5). Default false: tilt reaching the engine
## is the SH-5 Rust change; this toggle only gates the GDScript side, so it stays
## off until that lands. Reading it off here never breaks a build that lacks the
## engine field.
static func pen_tilt_to_size() -> bool:
	_ensure_loaded()
	return bool(_cfg.get_value(_SEC_STYLUS, "tilt_to_size", false))

## Stores the tilt-to-size preference and saves.
static func set_pen_tilt_to_size(on: bool) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_STYLUS, "tilt_to_size", on)
	_save()
