extends Node
## Lane Units, verification pass: does every distance and rate the Journey
## planner draws follow `Preferences ▸ Units`, and does it round-trip?
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _jpunits_probe.tscn
##
## **Windowed, deliberately -- there is no `--headless` in that line.** Nothing
## here reads a pixel, so `MISTAKES.md`'s "headless for logic and layout" would
## have permitted the dummy driver; windowed is the stricter environment (a real
## display server, real font metrics, a real `MenuBar`) and it is what this
## lane's brief asked the round-trip to be proved in. The density it actually
## ran at is printed in the `UNITS ===` banner below rather than assumed, and
## no resolution flag is passed -- this probe reads no argument, so its header
## claims none (`MISTAKES.md`: "Write a probe's usage header").
##
## Three seeds, because what a panel prints is content-dependent and one world
## is one sample.
##
## **The conversion factors below are literals, not a re-read of
## `DccUnits.KM_PER_MI`.** A check that quotes the constant it is checking
## passes for every value of that constant; these two are definitional (1 mi =
## 1 609.344 m, 1 NM = 1 852 m exactly), so mutating the shell's constant makes
## this probe go red.
##
## **It restores `DccSettings.units_mode()` to whatever it found.** The setting
## is persisted to `cartalith_settings.cfg` on every write, so a probe that
## exits in miles leaves the app in miles.

const SEEDS := [483920, 77021, 4242]
const MODES := ["km", "mi", "nmi"]

const KM_PER := {"km": 1.0, "mi": 1.609344, "nmi": 1.852}
const EXPECT_SUFFIX := {"km": "km", "mi": "mi", "nmi": "nm"}

## Stragglers this pass found, could not fix from `journey_planner_view.gd`, and
## reported instead. The sweep below asserts the straggler set is EXACTLY this
## -- so a new unconverted site in the view fails, and neither of these two is
## quietly forgotten either.
##
## Each entry is `[substring, where it lives, why it is not this view's]`.
const KNOWN_STRAGGLERS := [
	["Cursor position in km",
		"right_dock.gd:1836 (Sample ▸ Position tooltip)",
		"a foreign file. Its readout at right_dock.gd:6273 is raw km too, so the tooltip is not merely stale prose -- the whole Sample block is unrouted."],
	["of column",
		"cartalith-civ lib.rs:11168, term(\"column\", format!(\"{} km of column\", ...))",
		"the engine hands the trace a PRE-FORMATTED detail string with the unit baked in. The shell has no number to convert -- `col_km` reaches no other readout -- so converting it here would mean parsing engine prose."],
]

var app: Node
var checks := 0
var fails := 0
var entry_mode := "km"

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(ok: bool, what: String) -> void:
	checks += 1
	if ok:
		print("UNITS   ok    ", what)
	else:
		fails += 1
		print("UNITS   FAIL  ", what)

# -- capture -------------------------------------------------------------------

## Every string the Journey UI is currently showing, in tree order: label text,
## button text (an `OptionButton`'s `.text` is its selected row, which is how
## the committed-route line is reached), line-edit text, and every non-empty
## tooltip. Tooltips are in because a tooltip is a readout too -- the reach
## bar's per-leg distance has no other surface.
func _walk(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	elif n is LineEdit:
		out.append((n as LineEdit).text)
	if n is Control and (n as Control).tooltip_text != "":
		out.append("[tip] " + (n as Control).tooltip_text)
	for c in n.get_children():
		_walk(c, out)

func _capture(jpv) -> Array:
	var out: Array = []
	for root in [jpv._left_panel, jpv._center_panel, app.right_dock_body]:
		if root != null:
			_walk(root, out)
	out.append("[readout] " + jpv.readout_text())
	return out

## The element immediately after an exact match -- the shape every `_kv_row`,
## `_totals_row` and `_footer_stat` in this view builds (label Label, then value
## Label, in one HBox).
func _value_after(lines: Array, key: String) -> String:
	for i in lines.size():
		if String(lines[i]) == key:
			return String(lines[i + 1]) if i + 1 < lines.size() else "<end>"
	return "<missing:%s>" % key

## The leading number of a formatted readout: "1 234 km" -> 1234.0,
## "31.2 mi/d" -> 31.2. Space-grouping is removed first because that is this
## view's own thousands separator.
func _lead_num(s: String) -> float:
	var t := s.replace(" ", "")
	var out := ""
	for i in t.length():
		var c := t[i]
		if (c >= "0" and c <= "9") or c == "." or (out == "" and c == "-"):
			out += c
		elif out != "":
			break
	return float(out) if out != "" else -1.0

## Any captured string that still prints a kilometre VALUE while a non-km mode
## is selected: a digit immediately before " km", or the literal "km/d" anywhere
## (which is what an unconverted column header, legend or stat label looks
## like). Prose that merely names the engine's own unit -- "per tonne-km",
## "km-weighted", "points/km/mode" -- is not matched, and that is the line this
## pass drew deliberately.
func _km_stragglers(lines: Array) -> Array:
	var bad: Array = []
	for s in lines:
		var t := String(s)
		if t.contains("km/d"):
			bad.append(t)
			continue
		var i := t.find(" km")
		while i >= 0:
			if i > 0 and t[i - 1] >= "0" and t[i - 1] <= "9":
				bad.append(t)
				break
			i = t.find(" km", i + 1)
	return bad

# -- run -----------------------------------------------------------------------

func _ready() -> void:
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)
	entry_mode = DccSettings.units_mode()
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(3)
	print("UNITS === windowed  phone=", app.is_phone(), " phone_scale=", app.phone_scale(),
		" viewport=", app.get_viewport_rect().size, " entry_units=", entry_mode, " ===")

	var bridge = app.bridge
	for seed_v in SEEDS:
		await _one_seed(bridge, seed_v)

	DccSettings.set_units_mode(entry_mode)
	print("UNITS === restored units_mode=", DccSettings.units_mode(),
		"  checks=", checks, " fails=", fails, " ===")
	get_tree().quit()

func _one_seed(bridge, seed_v: int) -> void:
	bridge.generate({
		"seed": seed_v, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	var waited := 0
	while bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	await _frames(10)
	if not bridge.has_world:
		print("UNITS seed %d: generate FAILED" % seed_v)
		return
	var gs: Vector2i = bridge.grid_size()
	bridge.route_begin("mixed")
	bridge.route_append_stop(gs.x * 0.20, gs.y * 0.30)
	bridge.route_append_stop(gs.x * 0.55, gs.y * 0.50)
	bridge.route_append_stop(gs.x * 0.82, gs.y * 0.72)
	var ridx: int = bridge.route_commit()

	app.open_journey_planner()
	await get_tree().create_timer(0.8).timeout
	var jpv = app.journey_planner_view
	jpv._refresh_route_choice()
	## Same load reduction `_jptrace_probe.gd` uses, and for the same reason: it
	## does NOT unblock the journey, it buys the unblocked legs whose rate cells
	## and trace rows this probe reads.
	jpv._plan_values["cargo_kg"] = 40.0
	jpv._compute()
	await _frames(12)

	if not bool(jpv._last_result.get("ok", false)):
		print("UNITS seed %d: compute not ok: %s" % [seed_v, String(jpv._last_result.get("error", "?"))])
		return
	var plan: Dictionary = jpv._last_result.get("plan", {})
	print("UNITS ##### seed %d route=%d stages=%d plan.km=%.4f avg_km_day=%.4f #####"
		% [seed_v, ridx, (plan.get("stages", []) as Array).size(),
			float(plan.get("km", 0.0)), float(plan.get("avg_km_day", 0.0))])

	var caps: Dictionary = {}
	for mode in MODES:
		DccSettings.set_units_mode(mode)
		jpv.refresh_units()
		await _frames(8)
		caps[mode] = _capture(jpv)
		_report_mode(jpv, plan, mode, caps[mode], seed_v)

	## THE ROUND TRIP. Back to km after two other modes, and every string the
	## planner shows must be byte-identical to the km capture taken before
	## either of them -- a conversion that does not come home is a new defect,
	## not a half-fixed one.
	DccSettings.set_units_mode("km")
	jpv.refresh_units()
	await _frames(8)
	var back: Array = _capture(jpv)
	var a := "\n".join(PackedStringArray(caps["km"]))
	var b := "\n".join(PackedStringArray(back))
	_check(a == b, "seed %d round-trip km -> mi -> nmi -> km is byte-identical (%d strings)" % [seed_v, back.size()])
	if a != b:
		for i in mini(caps["km"].size(), back.size()):
			if String(caps["km"][i]) != String(back[i]):
				print("UNITS      first divergence [%d]: %s   !=   %s" % [i, caps["km"][i], back[i]])
				break

	## The status-bar write, which has no re-render of its own to ride on and so
	## cannot be read out of a capture -- it has to be provoked.
	DccSettings.set_units_mode("mi")
	jpv.refresh_units()
	await _frames(4)
	jpv._reroute_journey()
	await _frames(10)
	var hint := String((app._status_labels["hint"] as Label).text)
	print("UNITS      status hint = %s" % hint)
	if hint.begins_with("Re-routed for"):
		_check(hint.contains(" mi") and not hint.contains(" km"),
			"seed %d status-bar re-route line follows the preference" % seed_v)
	else:
		print("UNITS      (jp_reroute unavailable on this build -- status line not exercised)")
	DccSettings.set_units_mode("km")
	jpv.refresh_units()
	await _frames(4)

func _report_mode(jpv, plan: Dictionary, mode: String, cap: Array, seed_v: int) -> void:
	var suf: String = EXPECT_SUFFIX[mode]
	var dist := _value_after(cap, "distance")
	var speed := _value_after(cap, "mean speed")
	var desert := _value_after(cap, "desert %s" % suf)
	print("UNITS   [%s] distance=%s  mean speed=%s  desert=%s  readout=%s"
		% [mode, dist, speed, desert, String(cap[cap.size() - 1])])

	## The headline and the rate both carry the mode's own word.
	_check(dist.ends_with(" " + suf), "seed %d [%s] headline distance ends in '%s' (%s)" % [seed_v, mode, suf, dist])
	_check(speed.ends_with(" %s/d" % suf), "seed %d [%s] mean speed reads '%s/d' (%s)" % [seed_v, mode, suf, speed])

	## THE COLUMN HEADER. `km/d` names the unit for the whole rate column, so it
	## has to move with the cells; the negative half is what makes this
	## discriminating -- in mi it must ALSO no longer say km/d.
	var header_hit := false
	for s in cap:
		if String(s) == "%s/d" % suf:
			header_hit = true
			break
	_check(header_hit, "seed %d [%s] stage-matrix rate column header is '%s/d'" % [seed_v, mode, suf])
	if mode != "km":
		var stale := 0
		for s in cap:
			if String(s) == "km/d" or String(s) == "km/day":
				stale += 1
		_check(stale == 0, "seed %d [%s] no header or stat label left reading km/d or km/day (found %d)" % [seed_v, mode, stale])

	## The whole-surface sweep: any digit still followed by " km", or any
	## "km/d" anywhere, while a non-km mode is selected.
	## **Positive control for the sweep itself.** In km mode the detector must
	## fire on plenty -- the headline, the rate column, the trace -- because
	## those ARE kilometre values there. A sweep that returns nothing in km mode
	## is broken, and would then return nothing in mi mode too and report a
	## green it did not earn.
	if mode == "km":
		var seen := _km_stragglers(cap)
		_check(seen.size() >= 5,
			"seed %d [km] the km-value detector fires on the unconverted baseline (%d hits) -- it can fail" % [seed_v, seen.size()])

	if mode != "km":
		var bad := _km_stragglers(cap)
		var novel: Array = []
		var known_hit: Array = []
		for s in bad:
			var matched := ""
			for k in KNOWN_STRAGGLERS:
				if String(s).contains(String(k[0])):
					matched = String(k[1])
					break
			if matched == "":
				novel.append(s)
				print("UNITS      NEW straggler: %s" % s)
			elif not known_hit.has(matched):
				known_hit.append(matched)
		_check(novel.is_empty(),
			"seed %d [%s] no unconverted km VALUE this view owns (%d new, %d known-and-reported: %s)"
				% [seed_v, mode, novel.size(), known_hit.size(), str(known_hit)])

	## The ratio, against the definitional factor written as a literal above.
	if mode != "km":
		var v := _lead_num(dist)
		var km_v := float(plan.get("km", 0.0))
		var ratio := (km_v / v) if v > 0.0 else -1.0
		_check(absf(ratio - float(KM_PER[mode])) < 0.005 * float(KM_PER[mode]),
			"seed %d [%s] headline is plan.km / %.6f (measured %.6f)" % [seed_v, mode, float(KM_PER[mode]), ratio])
		var sv := _lead_num(speed)
		var akd := float(plan.get("avg_km_day", 0.0))
		var sratio := (akd / sv) if sv > 0.0 else -1.0
		_check(absf(sratio - float(KM_PER[mode])) < 0.02 * float(KM_PER[mode]),
			"seed %d [%s] mean speed is avg_km_day / %.6f (measured %.6f)" % [seed_v, mode, float(KM_PER[mode]), sratio])

	## The calculation trace: its locator now names a heading that moves, and
	## its total row moves with it. Both halves are asserted, plus the premise
	## the conversion rests on -- that exactly ONE term in the chain carries a
	## length, so scaling it and the total by the same factor leaves the chain
	## closing.
	var trace_total := false
	var trace_locator := false
	for s in cap:
		if String(s) == "= %s/day" % suf:
			trace_total = true
		if String(s) == "Stage matrix · %s/d" % suf:
			trace_locator = true
	_check(trace_total, "seed %d [%s] trace total row reads '= %s/day'" % [seed_v, mode, suf])
	_check(trace_locator, "seed %d [%s] trace locator points at 'Stage matrix · %s/d'" % [seed_v, mode, suf])

	if mode == "km":
		var results: Array = plan.get("results", [])
		var walked := 0
		var one_base := 0
		var first_base := 0
		for r in results:
			var rd: Dictionary = r
			if bool(rd.get("blocked", false)):
				continue
			var calc: Dictionary = rd.get("land", rd.get("water", {}))
			var trace: Array = calc.get("trace", [])
			if trace.is_empty():
				continue
			walked += 1
			var bases := 0
			for t in trace:
				if String((t as Dictionary).get("key", "")) == "base":
					bases += 1
			if bases == 1:
				one_base += 1
			if String((trace[0] as Dictionary).get("key", "")) == "base":
				first_base += 1
		_check(walked > 0 and one_base == walked and first_base == walked,
			"seed %d trace premise: every one of %d unblocked legs has exactly one 'base' term and it is first (one=%d first=%d)"
				% [seed_v, walked, one_base, first_base])
