extends SceneTree

## Lane B verification probe (not a deliverable).
##
## Exercises the code this lane wrote, over a real generated world, in the
## places no `cargo test` can reach:
##
## 1. the three-state contract `_religion_state` is written against, read off
##    the real bindings rather than off the doc comment;
## 2. `civilization_workspace.gd`'s static formatters, including the two
##    absence cases (`none` must never print blank; a zero total must dash);
## 3. `map_overlay.gd`'s `_faith_diverged` across every branch of its own
##    definition -- no key, unclaimed, unpushed roster, agreement, difference;
## 4. `map_overlay.gd`'s `_faith_lines` over real settlement dictionaries.

const CIVWS := preload("res://shell/workspaces/civilization_workspace.gd")
const OVERLAY := preload("res://map_overlay.gd")

var fails := 0

func _chk(ok: bool, what: String) -> void:
	if ok:
		print("  PASS  ", what)
	else:
		print("  FAIL  ", what)
		fails += 1

func _any_religion_key(wg: WorldGen) -> bool:
	for p in wg.get_settlements():
		if (p as Dictionary).has("religion"):
			return true
	return false

func _init() -> void:
	print("== 1. the three states, from the bindings ==")
	var wg: WorldGen = WorldGen.new()
	print("  before any world: settlements=", wg.get_settlements().size(),
		" has civ_belief_run=", wg.has_method("civ_belief_run"))
	_chk(wg.has_method("civ_belief_run"), "the binding exists (else state 'no_binding')")

	wg.generate_sized(24601, 640.0, 96, 64)
	var places: Array = wg.get_settlements()
	print("  generated: ", places.size(), " settlements")
	_chk(not places.is_empty(), "the world has settlements (else state 'no_world')")
	_chk(not _any_religion_key(wg),
		"state 'not_run': no settlement carries a religion key before a run")

	var st: Dictionary = wg.civ_belief_run(50)
	print("  run over an all-None roster -> ", st)
	_chk(not bool(st.get("any_faith", true)) and st.has("reason"),
		"state 'secular': any_faith false AND a reason string is present")
	_chk(_any_religion_key(wg),
		"the layer exists even when all-secular (keys present, any_faith false)")

	wg.civ_set_faction_field(1, "religion", "sun_cult")
	wg.civ_set_faction_field(2, "religion", "old_gods")
	var st2: Dictionary = wg.civ_belief_run(80)
	print("  run after two religions set -> ", st2)
	_chk(bool(st2.get("any_faith", false)), "state 'live': any_faith true")
	_chk(not st2.has("reason"),
		"reason is OMITTED on the normal path, not blanked (MISTAKES: absent keys)")

	places = wg.get_settlements()
	var mixed := 0
	var secular_plurality := 0
	for p in places:
		var d: Dictionary = p
		if int((d.get("adherents", {}) as Dictionary).size()) > 1:
			mixed += 1
		if String(d.get("religion", "")) == "none":
			secular_plurality += 1
	print("  ", places.size(), " settlements; ", mixed, " hold more than one faith; ",
		secular_plurality, " have a plurality of `none`")

	print("== 2. the panel's formatters ==")
	_chk(CIVWS._religion_label("none") == "No religion",
		"`none` renders as words, never blank (got '%s')" % CIVWS._religion_label("none"))
	_chk(CIVWS._religion_label("sun_cult") == "Sun Cult",
		"capitalize() reproduces the CIV_RELIGIONS label (got '%s')"
			% CIVWS._religion_label("sun_cult"))
	_chk(CIVWS._religion_pct(1, 0) == "—", "a zero total dashes rather than printing 0.0%")
	_chk(CIVWS._religion_pct(1, 4) == "25.0%", "a real share formats (got '%s')"
		% CIVWS._religion_pct(1, 4))
	_chk(CIVWS._religion_pct(1, 5000) == "<0.1%",
		"one adherent in 5000 reads as a minority, not as 0.0%% (got '%s')"
			% CIVWS._religion_pct(1, 5000))
	_chk(CIVWS._religion_pct(0, 100) == "0.0%",
		"a genuine zero still prints zero (got '%s')" % CIVWS._religion_pct(0, 100))
	var sorted: Array = CIVWS._religion_sorted({"old_gods": 3, "none": 0, "sun_cult": 3,
		"sea_lords": 9})
	print("  sorted: ", sorted)
	_chk(sorted.size() == 3, "a zero count is dropped, not listed as a present faith")
	_chk(sorted[0][1] == "sea_lords" and sorted[1][1] == "old_gods" and sorted[2][1] == "sun_cult",
		"descending by count, ties by key -- deterministic across runs")
	var c_none: Color = CIVWS._religion_color("none")
	var c_sun: Color = CIVWS._religion_color("sun_cult")
	var c_old: Color = CIVWS._religion_color("old_gods")
	print("  colours: none=", c_none, " sun_cult=", c_sun, " old_gods=", c_old)
	_chk(c_none != c_sun and c_sun != c_old, "three keys, three distinct swatches")
	_chk(CIVWS._religion_swatch_glyph("none") != CIVWS._religion_swatch_glyph("sun_cult"),
		"the unaffiliated slot is marked by shape, not by its grey alone")

	print("== 3. _faith_diverged, every branch of its own definition ==")
	var ov: Control = OVERLAY.new()
	ov.set_faith_divergence_visible(true)
	ov.set_faction_religions(PackedStringArray(["sun_cult", "old_gods", ""]))
	_chk(not ov._faith_diverged({"faction": 1}),
		"no `religion` key -> false (nothing to differ from, not 'diverged')")
	_chk(not ov._faith_diverged({"faction": 0, "religion": "sun_cult"}),
		"faction 0 (Unclaimed) -> false (no ruler)")
	_chk(not ov._faith_diverged({"faction": 9, "religion": "sun_cult"}),
		"faction past the pushed roster -> false")
	_chk(not ov._faith_diverged({"faction": 3, "religion": "sun_cult"}),
		"an empty pushed key -> false (row not read, not 'secular')")
	_chk(not ov._faith_diverged({"faction": 1, "religion": "sun_cult"}),
		"agreement -> false")
	_chk(not ov._faith_diverged({"faction": 2, "religion": "none"}) == false,
		"a secular town under an old_gods ruler HAS diverged")
	_chk(ov._faith_diverged({"faction": 1, "religion": "old_gods"}), "a real difference -> true")
	ov.set_faith_divergence_visible(false)
	_chk(not ov._faith_diverged({"faction": 1, "religion": "old_gods"}),
		"the layer off -> false, so nothing draws")
	ov.set_faith_divergence_visible(true)

	## The same test over the real world, counted the way the panel counts it.
	var faiths := PackedStringArray()
	for r in wg.get_factions():
		var d: Dictionary = r
		var id := int(d.get("id", 0))
		while faiths.size() < id:
			faiths.append("")
		if id > 0:
			faiths[id - 1] = String(d.get("religion", ""))
	print("  roster religion column: ", faiths)
	ov.set_faction_religions(faiths)
	var diverged := 0
	for p in places:
		if ov._faith_diverged(p):
			diverged += 1
	print("  diverged on the real world: ", diverged, " of ", places.size())
	_chk(diverged <= places.size(), "the divergence count is bounded by the roster")

	print("== 4. _faith_lines over real settlement dictionaries ==")
	_chk((ov._faith_lines({"name": "x", "population": 10}) as Array).is_empty(),
		"no religion key -> no lines at all (the panel owns that disclosure)")
	var shown := 0
	for p in places:
		var lines: Array = ov._faith_lines(p)
		if lines.is_empty():
			_chk(false, "a live layer produced no faith line for %s" % String(p.get("name", "?")))
			break
		if shown < 4:
			print("  ", String(p.get("name", "?")), " -> ", lines)
			shown += 1
	var secular_line: Array = ov._faith_lines({"population": 100, "religion": "none",
		"adherents": {"none": 61, "sun_cult": 39}})
	print("  majority-secular card: ", secular_line)
	_chk(String(secular_line[0]).contains("no religion"),
		"a majority-secular town says so; it is not labelled with its largest faith")
	_chk(secular_line.size() == 2 and String(secular_line[1]).contains("Sun Cult"),
		"the minority faith is still listed -- adherence is a distribution")
	var zero_pop: Array = ov._faith_lines({"population": 0, "religion": "none", "adherents": {}})
	print("  population 0 card: ", zero_pop)
	_chk(not String(zero_pop[0]).contains("%"),
		"population 0 prints no percentage rather than 0%")
	var tiny: Array = ov._faith_lines({"population": 900, "religion": "none",
		"adherents": {"none": 898, "old_gods": 2}})
	print("  a two-person congregation: ", tiny)
	_chk(String(tiny[1]).contains("<1%"),
		"a congregation too small to round is `<1%`, never `0%` (got '%s')" % String(tiny[1]))
	_chk(String(tiny[0]).contains(">99%"),
		"and the majority beside it is `>99%%`, so the card cannot say 100%% and `also ...` "
			+ "in the same breath (got '%s')" % String(tiny[0]))
	_chk(CIVWS._religion_pct(9999, 10000) == ">99.9%",
		"the panel carries the same top-end guard (got '%s')"
			% CIVWS._religion_pct(9999, 10000))
	_chk(CIVWS._religion_pct(100, 100) == "100.0%",
		"a real whole still prints 100.0%% (got '%s')" % CIVWS._religion_pct(100, 100))

	print("")
	print("RESULT: ", "ALL PASS" if fails == 0 else "%d FAILURES" % fails)
	quit(0 if fails == 0 else 1)
