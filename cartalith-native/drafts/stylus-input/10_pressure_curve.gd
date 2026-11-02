## DRAFT, never loaded by Godot (it sits outside `godot-project/`). SH-4: the pen
## **pressure curve** (soft / linear / firm) and the comfortable maximum, applied
## to a live pen's pressure before it reaches the engine. GDScript only; the
## engine's pressure law is untouched, so no golden can move.
##
## Depends on `01_dcc_settings_stylus.gd` (the `pen_pressure_curve` /
## `pen_pressure_max` accessors). Order: after 01.
##
## WHY IT IS SAFE FOR GOLDENS. The curve reshapes only the pressure value a LIVE
## pen produces — which is user input, never part of deterministic generation. The
## default is the identity (curve "linear", max 1.0), short-circuited below so the
## returned value is BITWISE the shipped value until the user picks a curve.
## `hash_gen1`-style parity does not enter: this is shell input, not the engine.
##
## WHERE IT APPLIES. There are exactly two places `world_workspace.gd` hands a
## pen's pressure to the engine, both reading a float 0..1 or leaving it `null`:
##   - sculpt: `_sculpt_take_pressure` returns `ps["pressure"]`;
##   - paint:  `_paint_drag` sets `pressure = ps["pressure"]`.
## One helper remaps both. `null` (no pen, or the Pressure toggle off) stays `null`
## — "no value is not a plausible value", the rule both sites already keep.
##
## HOW TO APPLY (three edits in `shell/workspaces/world_workspace.gd`):
##
##   ADD the helper below, next to `_sculpt_take_pressure`.
##
##   EDIT S1 — in `_sculpt_take_pressure`, the success return:
##       -   return ps["pressure"]
##       +   return _apply_pen_pressure(ps["pressure"])
##     (the `if not _sculpt_pressure_on: return null` above is unchanged: a
##      pressure-less stroke stays pressure-less, the curve never invents one.)
##
##   EDIT P1 — in `_paint_drag`, the matched-sample branch:
##       -       pressure = ps["pressure"]
##       +       pressure = _apply_pen_pressure(ps["pressure"])
##     (leave `tilt = ps["tilt"]` as is — tilt is SH-5, not this draft.)
##
## MUST NEVER: change a `null` to a number, or alter the value at the default
## settings (the short-circuit guarantees it), or touch the engine's own pressure
## handling (`WorldGen::sculpt_add_point_p` / `paint_stroke_sample` are unchanged).
##
## VERIFY: `godot --headless --check-only` on the file. Then a windowed probe:
##   - with the default settings, feed a known pen pressure and assert the value
##     reaching the bridge is BITWISE equal to today's (the identity short-circuit);
##   - set curve "firm", feed 0.5, assert the engine receives pow(0.5, 1.6) ~ 0.33;
##   - set `pressure_max` 0.7, feed 0.7, assert it receives 1.0.
##   On the desktop a mouse returns `null` here, so the curve is a no-op — exercise
##   it on the device, or by injecting a float through the probe.

## --- paste into shell/workspaces/world_workspace.gd, by _sculpt_take_pressure ---

## SH-4. Remaps a live pen's pressure through the user's curve and comfortable
## maximum (`DccSettings`), for both the sculpt and paint pressure paths. Takes and
## returns a `Variant`: a float 0..1 is remapped, and `null` (no pen, or the
## Pressure toggle off) passes straight through. At the default settings (curve
## "linear", max 1.0) it returns the value UNCHANGED, bitwise — the short-circuit
## below — so turning the feature on changes nothing until a curve is chosen.
## Must never: invent a pressure where there was none, or touch the engine's law.
func _apply_pen_pressure(p: Variant) -> Variant:
	if not (p is float):
		return p
	var curve := DccSettings.pen_pressure_curve()
	var pmax := DccSettings.pen_pressure_max()
	## Identity at the defaults: return the exact float, no arithmetic, no fuzz.
	if curve == "linear" and is_equal_approx(pmax, 1.0):
		return p
	var raw := clampf(float(p), 0.0, 1.0)
	## Comfortable maximum: the raw pressure that should read as full strength.
	var scaled := clampf(raw / pmax, 0.0, 1.0) if pmax > 0.0 else raw
	var gamma: float = DccSettings.PRESSURE_CURVES.get(curve, 1.0)
	return pow(scaled, gamma)
