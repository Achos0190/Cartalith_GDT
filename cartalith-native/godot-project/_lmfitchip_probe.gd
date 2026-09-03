extends Node
## Lane A evidence probe: §5's two funnel chips, and the crowding direction the
## panel had backwards.
##
## Every assertion here fails if the fix is reverted. Four questions:
##
##   A. Which way does Crowding go? The panel said "lower Crowding" in three
##      places and the engine divides by it. Asserted against a real pass, not
##      against a comment.
##   B. Does `_lm_radius_in_force()` agree with `LandmarkSettings::radius_km`?
##      They are two copies of one formula and the panel quotes a km figure off
##      its own.
##   C. Does `_lm_fit_plan()` reach every one of its five states, and does it
##      treat an ABSENT `needs_crowding` as absent? `get(..., 0.0)` there would
##      produce a pressable chip offering × 0.05.
##   D. On a real world: do the reject rows behave the way the two chips assume
##      -- no `0.0` sentinel, some rows with the key genuinely missing, and a
##      derivable state per kind?
##
##   Godot_v4.7.1 --headless --path . _lmfitchip_probe.tscn

const CW = preload("res://shell/workspaces/civilization_workspace.gd")

## `DEFAULT_CLASS_RADIUS_KM[1]`, the Regional figure the Crowding readout
## quotes and the one every design artboard prints ("keeps 34 km clear").
const REGIONAL_KM := 34.0

var _fails := 0

func _check(ok: bool, what: String) -> void:
	if ok:
		print("  ok   ", what)
	else:
		_fails += 1
		print("  FAIL ", what)

## Enough of `EngineBridge` for §5's popover: the reject list the two chips
## read, and a record of the one write the first chip makes. Everything else is
## inherited, which is what keeps this a test of the panel and not of a fixture.
class StubBridge extends EngineBridge:
	var rows: Array = []
	var writes: Array = []

	func landmark_rejects() -> Array:
		return rows

	func landmark_set_crowding(v: float) -> bool:
		writes.append(v)
		return true

	func landmark_headroom() -> Dictionary:
		return {}

func _ready() -> void:
	_pure()
	_live()
	await _popover()
	print("")
	if _fails == 0:
		print("_lmfitchip_probe: ALL PASS")
	else:
		print("_lmfitchip_probe: ", _fails, " FAILED")
	get_tree().quit(1 if _fails > 0 else 0)

# -- B and C: the pure halves ------------------------------------------------

func _pure() -> void:
	print("B. _lm_radius_in_force -- the engine divides")
	## `radius_km` is `base / crowding_in_force()`. If this line is ever
	## multiplied back, × 2.00 reads 68 km here and the assertion below fails.
	_check(is_equal_approx(CW._lm_radius_in_force(REGIONAL_KM, 1.0), 34.0),
		"× 1.00 keeps 34 km (the figure every artboard prints)")
	_check(is_equal_approx(CW._lm_radius_in_force(REGIONAL_KM, 2.0), 17.0),
		"× 2.00 keeps 17 km, NOT 68 -- denser, not sparser")
	_check(CW._lm_radius_in_force(REGIONAL_KM, 2.0) < CW._lm_radius_in_force(REGIONAL_KM, 0.5),
		"raising Crowding shrinks the ring")
	## `crowding_in_force`'s own two guards, mirrored: a literal zero would send
	## the radius to infinity, and NaN would collapse the bucket grid.
	_check(is_finite(CW._lm_radius_in_force(REGIONAL_KM, 0.0)),
		"a zero Crowding stays finite (clamped at 0.05, as the engine clamps)")
	_check(is_equal_approx(CW._lm_radius_in_force(REGIONAL_KM, NAN), 34.0),
		"a NaN Crowding falls back to × 1.00, as radius_km does")

	print("C. _lm_fit_plan -- five states, and `has()` not `get(.., 0.0)`")
	var spacing := func(k: String, needs) -> Dictionary:
		var d := {"kind": k, "x": 1, "y": 2, "score": 0.5, "reason": "spacing"}
		if needs != null:
			d["needs_crowding"] = needs
		return d

	_check(String(CW._lm_fit_plan([], "peak", 0).get("state")) == "capped",
		"room 0 -> capped, before the list is even consulted")
	_check(String(CW._lm_fit_plan([], "peak", 5).get("state")) == "none",
		"no rows at all -> none")
	_check(String(CW._lm_fit_plan([spacing.call("gorge", 1.5)], "peak", 5).get("state")) == "none",
		"another kind's rows are not this kind's")
	_check(String(CW._lm_fit_plan([{"kind": "peak", "reason": "cap"}], "peak", 5)
			.get("state")) == "none",
		"a cap reject is not a spacing reject")

	## **The defect guard.** A spacing row whose blocker sat on its own cell
	## carries no `needs_crowding` key at all. Read with `get(.., 0.0)` it
	## becomes a figure of 0.0, snaps up to × 0.05, and the chip goes green
	## offering a setting that places nothing.
	var blocked := CW._lm_fit_plan([spacing.call("peak", null)], "peak", 5)
	_check(String(blocked.get("state")) == "no_figure",
		"an ABSENT needs_crowding is absent, not × 0.00")
	_check(int(blocked.get("listed", 0)) == 1, "  ...and the row is still counted as listed")

	## ...and the same row as an **older cdylib** sends it: the key present,
	## carrying `unwrap_or(0.0)`. `crowding_in_force` floors at 0.05, so no real
	## figure can be at or under it; without that floor this lights a pressable
	## "Raise crowding to × 0.05" that places nothing.
	_check(String(CW._lm_fit_plan([spacing.call("peak", 0.0)], "peak", 5)
			.get("state")) == "no_figure",
		"a 0.0 from an older cdylib is not a figure either")
	_check(String(CW._lm_fit_plan([spacing.call("peak", 0.05)], "peak", 5)
			.get("state")) == "no_figure",
		"nor is 0.05 -- a real answer is strictly above the crowding floor")
	_check(String(CW._lm_fit_plan([spacing.call("peak", INF)], "peak", 5)
			.get("state")) == "no_figure",
		"nor an infinity")

	var over := CW._lm_fit_plan([spacing.call("peak", 7.4)], "peak", 5)
	_check(String(over.get("state")) == "over",
		"× 7.40 is above the dial's ceiling -> over, not clamped to × 2.00")
	_check(is_equal_approx(float(over.get("target", 0.0)), 7.4),
		"  ...and the honest figure survives into the tooltip")

	var ok := CW._lm_fit_plan([spacing.call("peak", 1.32)], "peak", 5)
	_check(String(ok.get("state")) == "ok", "a reachable figure -> ok")
	_check(is_equal_approx(float(ok.get("target", 0.0)), 1.35),
		"snapped UP to the 0.05 step (1.32 -> 1.35); 1.30 would still reject it")

	var exact := CW._lm_fit_plan([spacing.call("peak", 1.35)], "peak", 5)
	_check(is_equal_approx(float(exact.get("target", 0.0)), 1.35),
		"a figure already on a step is not pushed to the next one")

	## Three candidates, room for two: the target is the second-smallest, and
	## the gain is what that target actually admits, not the room.
	var many := CW._lm_fit_plan([spacing.call("peak", 1.10), spacing.call("peak", 1.60),
		spacing.call("peak", 1.15)], "peak", 2)
	_check(is_equal_approx(float(many.get("target", 0.0)), 1.15),
		"room 2 of 3 -> the 2nd-smallest figure, snapped")
	_check(int(many.get("gain", 0)) == 2, "  ...admitting 2, and never more than the room")
	_check(int(CW._lm_fit_plan([spacing.call("peak", 1.10), spacing.call("peak", 1.60)],
		"peak", 9).get("gain", 0)) == 2, "room to spare -> every figure admitted")

	print("   _lm_reject_count")
	var mixed := [spacing.call("peak", 1.1), spacing.call("gorge", 1.1),
		{"kind": "peak", "reason": "cap"}]
	_check(CW._lm_reject_count(mixed, "peak") == 2, "counts every reason, one kind")
	_check(CW._lm_reject_count(mixed, "cliff") == 0, "and none for a kind with no rows")

# -- A and D: against a real world -------------------------------------------

func _live() -> void:
	print("A/D. a real pass")
	var g := WorldGen.new()
	if not g.has_method("landmark_rejects"):
		_fails += 1
		print("  FAIL this build's WorldGen has no landmark_rejects -- probe cannot run")
		return
	g.set_sea_level(0.45)
	g.set_villages_enabled(true)
	g.generate_sized(483920, 2400.0, 512, 384)

	## A. The direction, measured. `crowding_higher_packs_tighter` asserts this
	## in Rust; this asserts the same thing through the bindings the panel
	## actually calls, so a shell that writes the dial backwards fails here.
	g.landmark_set_crowding(0.5)
	g.landmark_run()
	var sparse := int(g.landmark_last_run().get("placed", 0))
	g.landmark_set_crowding(2.5)
	g.landmark_run()
	var dense := int(g.landmark_last_run().get("placed", 0))
	print("   placed at × 0.50: ", sparse, "   at × 2.50: ", dense)
	_check(dense > sparse,
		"RAISING Crowding places more -- the chip says Raise, not Lower")

	## D. The reject rows the two chips read.
	g.landmark_set_crowding(1.0)
	g.landmark_run()
	var rejects: Array = g.landmark_rejects()
	var spacing_rows := 0
	var with_figure := 0
	var absent := 0
	var zero_sentinel := 0
	for e in rejects:
		var d: Dictionary = e
		if String(d.get("reason", "")) != "spacing":
			continue
		spacing_rows += 1
		if d.has("needs_crowding"):
			with_figure += 1
			if float(d["needs_crowding"]) == 0.0:
				zero_sentinel += 1
		else:
			absent += 1
	print("   rejects=", rejects.size(), "  spacing=", spacing_rows,
		"  with figure=", with_figure, "  key absent=", absent)
	_check(rejects.size() > 0, "the pass produced reject rows to draw")
	_check(zero_sentinel == 0,
		"no spacing row carries needs_crowding == 0.0 (the sentinel is gone)")
	_check(spacing_rows == with_figure + absent,
		"every spacing row either carries a figure or omits the key")

	## Every kind the funnel reports must yield a state the chip can draw --
	## `_lm_fit_plan` is total by construction and this is the assertion that
	## keeps it so against real data.
	var states := {}
	for f in g.landmark_funnels():
		var d: Dictionary = f
		var kind := String(d.get("kind", ""))
		var room: int = maxi(int(d.get("cap", 0)) - int(d.get("placed", 0)), 0)
		var st := String(CW._lm_fit_plan(rejects, kind, room).get("state", ""))
		states[st] = int(states.get(st, 0)) + 1
		if st == "":
			_fails += 1
			print("  FAIL no state for kind '", kind, "'")
	print("   states across every kind: ", states)
	_check(not states.has(""), "every kind resolves to a named state")

# -- E: the popover itself ----------------------------------------------------
#
# `--check-only` cannot see inside `_lm_fit_chip` / `_lm_show_chip`: a wrong
# method name or a bad `match` arm in there parses clean and dies on the click.
# This builds the real popover body and presses the real chip.
#
# Built by calling `_lm_funnel_body()` directly rather than by standing the
# whole shell up. `_landmark_probe.gd` does the latter and cannot run today --
# another lane's untracked `shell/diagnostic_report.gd` declares `class_name
# DiagnosticReport`, which is not in `.godot/global_script_class_cache.cfg`, so
# `menus.gd` fails to compile and the boot dies before any workspace exists.

func _buttons(n: Node, out: Array) -> void:
	if n is Button:
		out.append(n)
	for c in n.get_children():
		_buttons(c, out)

func _body(ws: CivilizationWorkspace, kind: String, cap: int, placed: int) -> Array:
	var pad: Control = ws.call("_lm_funnel_body", kind,
		{"label": "Waterfall", "cap": cap},
		{"kind": kind, "candidates": 1284, "rejected_constraint": 1149,
			"rejected_score": 0, "rejected_spacing": 124, "rejected_cap": 0,
			"cap": cap, "placed": placed, "limit": "spacing"})
	var out: Array = []
	_buttons(pad, out)
	return out

func _popover() -> void:
	print("E. the real popover body, and a real press")
	var stub := StubBridge.new()
	add_child(stub)
	var ws := CivilizationWorkspace.new()
	ws.bridge = stub
	add_child(ws)
	await get_tree().process_frame

	## Room for 29 more, and 124 spacing rejects whose figures start at 1.32 --
	## `WhyFewer.dc.html`'s own numbers, with the multiplier the engine actually
	## produces rather than the canvas's inverted × 0.70.
	stub.rows = []
	for i in 124:
		stub.rows.append({"kind": "waterfall", "x": i, "y": 1, "score": 0.4,
			"reason": "spacing", "needs_crowding": 1.32 + 0.01 * i})
	var chips := _body(ws, "waterfall", 40, 11)
	print("   chips built: ", chips.size(), "  -> ",
		[] if chips.is_empty() else [chips[0].text, chips[1].text])
	_check(chips.size() == 2, "the popover draws exactly two action chips")
	if chips.size() != 2:
		return
	_check(not (chips[0] as Button).disabled, "the crowding chip is ENABLED, not stranded")
	_check((chips[0] as Button).text.begins_with("Raise crowding to × "),
		"...and says what it will do, with the number in the label")
	_check((chips[0] as Button).text == "Raise crowding to × 1.60",
		"...the 29th-smallest figure (1.59), snapped up to the dial's step")
	_check(not (chips[1] as Button).disabled and (chips[1] as Button).text == "Show 124 rejected",
		"the show chip is enabled and carries the count the map will draw")
	_check((chips[0] as Button).tooltip_text.find("does NOT re-generate") >= 0,
		"the tooltip says what the press will NOT do")

	## The press. `_lm_crowd_slider` is null here, which is the guarded path, so
	## this asserts the write reaches the engine through it.
	ws.call("_lm_apply_crowding", 1.60)
	_check(is_equal_approx(float(ws.get("_lm_crowding")), 1.60),
		"pressing it moves the panel's own Crowding")
	_check(stub.writes.size() == 1 and is_equal_approx(float(stub.writes[0]), 1.60),
		"...and writes it to the engine once (drag_ended never fires for us)")
	## `_lm_show_rejects` with a null `app` -- the guard that keeps a chip press
	## from taking the dock down on a workspace built outside the shell.
	ws.call("_lm_show_rejects")
	_check(true, "the show chip's handler survives a null app")

	print("   ...and the states that must NOT be pressable")
	stub.rows = []
	for i in 124:
		stub.rows.append({"kind": "waterfall", "x": i, "y": 1, "score": 0.4,
			"reason": "spacing", "needs_crowding": 1.32 + 0.01 * i})
	var full := _body(ws, "waterfall", 40, 40)
	_check(full.size() == 2 and (full[0] as Button).disabled
		and (full[0] as Button).text == "Crowding cannot place more",
		"a full cap disables the chip on a fact about the world")
	_check(full.size() == 2 and (full[0] as Button).tooltip_text.find("Raise the cap") >= 0,
		"...and names the control that WOULD help")

	stub.rows = [{"kind": "waterfall", "x": 1, "y": 1, "score": 0.4,
		"reason": "spacing", "needs_crowding": 7.4}]
	var over := _body(ws, "waterfall", 40, 11)
	_check(over.size() == 2 and (over[0] as Button).disabled
		and (over[0] as Button).text == "Needs crowding × 7.40 — off this dial",
		"an unreachable figure is shown, not clamped to × 2.00")

	stub.rows = []
	var bare := _body(ws, "waterfall", 40, 11)
	_check(bare.size() == 2 and (bare[1] as Button).disabled
		and (bare[1] as Button).text == "No rejects to show",
		"nothing to draw -> the show chip says so instead of being a no-op")
