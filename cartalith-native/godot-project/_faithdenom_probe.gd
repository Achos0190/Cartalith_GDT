extends Node
## VERIFY: does any religion surface still present a settlement count as
## though it were a head-count, and does every settlement now show the faith
## the roll-up counts it for?
##
## Every expected number below is counted independently from
## `get_settlements()` in this file, then required to appear in the RENDERED
## text. None of the checks restates a format string: change the panel's
## arithmetic and the number stops matching; change only its wording and the
## check still passes, which is the intent.
##
##   Godot_v4.7.1-stable_win64.exe --headless --path . _faithdenom_probe.tscn

var _app: Node
var _fails := 0

func _chk(ok: bool, what: String) -> void:
	print(("  PASS  " if ok else "  FAIL  ") + what)
	if not ok:
		_fails += 1

func _harvest(node: Node, out: Array) -> void:
	for c in node.get_children():
		if c is Label or c is Button or c is CheckBox:
			var t := String(c.text)
			if not t.is_empty():
				out.append(t)
		_harvest(c, out)

func _render(ws: Control) -> Array:
	var host := VBoxContainer.new()
	add_child(host)
	ws._fill_religion(host)
	var lines: Array = []
	_harvest(host, lines)
	remove_child(host)
	host.queue_free()
	return lines

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 420.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge
	var ws: Control = null
	for w in _app._workspaces:
		if w.name == "CivilizationWorkspace":
			ws = w
	if ws == null:
		print("PROBE FAIL: no CivilizationWorkspace")
		get_tree().quit(2)
		return

	bridge.generate({
		"seed": 77021, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout

	bridge.civ_set_faction_field(1, "religion", "sun_cult")
	bridge.civ_set_faction_field(3, "religion", "old_gods")
	ws._religion_years = 50
	ws._religion_run()

	var places: Array = bridge.settlements()
	var n := places.size()

	# ---- independent truth -------------------------------------------------
	var populated := 0
	var leads := {}
	var leads_pop := {}
	var pop0_faith: Array = []
	var held := {}
	var counted := {}
	for p in places:
		var d: Dictionary = p
		var pop := int(d.get("population", 0))
		var f := int(d.get("faction", 0))
		held[f] = int(held.get(f, 0)) + 1
		if pop > 0:
			populated += 1
			counted[f] = int(counted.get(f, 0)) + 1
		if not d.has("religion"):
			continue
		var r := String(d["religion"])
		leads[r] = int(leads.get(r, 0)) + 1
		if pop > 0:
			leads_pop[r] = int(leads_pop.get(r, 0)) + 1
		elif r != "none":
			pop0_faith.append([String(d.get("name", "")), r])
	print("independent: %d settlements, %d with a population" % [n, populated])
	print("independent: leads=%s leads_with_people=%s" % [leads, leads_pop])
	print("independent: pop-0 settlements led by a faith = %d" % pop0_faith.size())

	## Printed, not asserted, and it is the evidence for what this pass did NOT
	## build. A per-faith map focus ("mark every settlement Sun Cult leads") was
	## considered and dropped: on this world every faith-led settlement lies
	## inside the single faction whose ruler set that faith, so the focus layer
	## would be a recolour of the faction wash `territory_view` already draws --
	## which is exactly the argument `civilization_workspace.gd`'s own "Why the
	## map half is a mark, not a wash" block makes, now measured rather than
	## reasoned. A world where a faith crosses a border would show it here.
	var xt := {}
	for p in places:
		var d: Dictionary = p
		if d.has("religion"):
			var k2 := "%s@f%d" % [String(d["religion"]), int(d.get("faction", 0))]
			xt[k2] = int(xt.get(k2, 0)) + 1
	var xk: Array = xt.keys()
	xk.sort()
	for k in xk:
		print("   plurality %-22s %d settlements" % [k, int(xt[k])])

	# Premises. Each check below is unsatisfiable if its premise is false, so
	# say so here rather than let a vacuous PASS stand in for a measurement.
	_chk(populated > 0 and populated < n,
		"the fixture has BOTH populated and population-0 settlements (%d / %d)" % [populated, n])
	var disjoint := 0
	for k in leads.keys():
		if int(leads[k]) > 0 and int(leads_pop.get(k, 0)) == 0:
			disjoint += 1
	_chk(disjoint > 0,
		"at least one faith leads only settlements with nobody in them (%d faiths)" % disjoint)
	_chk(pop0_faith.size() > 0,
		"the diffusion left faiths leading population-0 settlements (%d)" % pop0_faith.size())

	# ---- rendered panel ----------------------------------------------------
	var lines := _render(ws)
	var joined := " | ".join(PackedStringArray(lines))

	_chk(joined.contains("%d of %d settlements" % [populated, n]),
		"the headline denominator is the settlements the head-counts come from (%d of %d)"
		% [populated, n])
	_chk(not joined.contains("people across %d settlements" % n),
		"and the old world-sized denominator is gone")

	var missing_tail := 0
	var wrong_lead := 0
	for k in leads.keys():
		var lbl: String = ws._religion_label(String(k))
		var line := ""
		for t in lines:
			var s := String(t)
			if s.contains(lbl + " ") and s.contains("leads ") and s.contains("people ("):
				line = s
				break
		if line == "":
			continue
		if not line.contains("leads %d settlement" % int(leads[k])):
			wrong_lead += 1
			print("   LEADCOUNT: ", line)
		if int(leads_pop.get(k, 0)) == 0 and int(leads[k]) > 0:
			if not line.contains("none of them with a population"):
				missing_tail += 1
				print("   NOTAIL: ", line)
	_chk(wrong_lead == 0, "every faith settlement count matches an independent count (%d off)"
		% wrong_lead)
	_chk(missing_tail == 0,
		"a faith whose settlements are all empty says so on the same line (%d silent)"
		% missing_tail)

	## The divergence sentence counts a different set from the roll-up, and the
	## panel now says so. Required only when the two CAN differ.
	if populated < n:
		_chk(joined.contains("over religion keys, not over people"),
			"the divergence count says its denominator is not the head-count one")
	_chk(joined.to_upper().contains("%d WITH A POPULATION" % populated),
		"the settlement list is split and the first group carries its own count")
	_chk(joined.to_upper().contains("%d WITH NO POPULATION" % (n - populated)),
		"and the second group carries the remainder (%d)" % (n - populated))

	# every population-0 settlement led by a faith now names it
	var unnamed := 0
	var mislabelled := 0
	## Counted on the SETTLEMENT rows only. The same sentence is a legitimate
	## `_religion_faction_row` output for a faction whose settlements all have
	## population 0, so searching the whole render for it would make this check
	## fail for a reason that has nothing to do with what it tests.
	var old_dash := 0
	var all_labels: Array = []
	for k in ["none", "sun_cult", "earth_mother", "sea_lords", "sky_pantheon",
			"ancestor_rites", "flame_creed", "old_gods"]:
		all_labels.append(ws._religion_label(k))
	for e in pop0_faith:
		var nm := String(e[0])
		var want: String = ws._religion_label(String(e[1]))
		var anchor := -1
		for i in lines.size():
			if String(lines[i]).begins_with(nm + " · "):
				anchor = i
				break
		if anchor < 0 or anchor + 1 >= lines.size():
			continue
		var row := String(lines[anchor + 1])
		if row.begins_with("— no adherents"):
			old_dash += 1
		if not row.contains(want):
			unnamed += 1
			if unnamed <= 3:
				print("   UNNAMED: ", nm, " wants ", want, " got: ", row)
		for other in all_labels:
			if String(other) != want and row.contains(String(other)):
				mislabelled += 1
				if mislabelled <= 3:
					print("   EXTRA: ", nm, " showed ", other, " -> ", row)
	_chk(unnamed == 0,
		"every population-0 settlement the roll-up counts names its faith (%d silent)" % unnamed)
	_chk(mislabelled == 0,
		"and none of them shows a faith it does not hold (%d)" % mislabelled)
	_chk(old_dash == 0,
		"no settlement row is the old dash that dropped the faith entirely (%d)" % old_dash)

	# faction composition denominators
	var want_denom := 0
	var got_denom := 0
	for f in held.keys():
		var c := int(counted.get(f, 0))
		var h := int(held[f])
		if c > 0 and c < h:
			want_denom += 1
			if joined.contains("from %d of its %d settlements" % [c, h]):
				got_denom += 1
			else:
				print("   NODENOM: faction %d counted=%d held=%d" % [f, c, h])
	_chk(want_denom > 0, "at least one faction composition is summed over a subset (%d)"
		% want_denom)
	_chk(got_denom == want_denom,
		"and every one of them prints that subset as its denominator (%d/%d)"
		% [got_denom, want_denom])

	# ---- hover card --------------------------------------------------------
	var ov = _app.viewport.overlay
	var card_bad := 0
	var card_ok := 0
	var card_false := 0
	for p in places:
		var d: Dictionary = p
		var fl: Array = ov._faith_lines(d)
		if fl.is_empty():
			continue
		var head := String(fl[0])
		if int(d.get("population", 0)) <= 0:
			if head.contains("no population, so no share"):
				card_ok += 1
			else:
				card_bad += 1
		elif head.contains("no population"):
			card_false += 1
	print("hover cards: %d population-0 cards say so, %d do not" % [card_ok, card_bad])
	_chk(card_ok > 0, "the fixture produces population-0 hover cards at all (%d)" % card_ok)
	_chk(card_bad == 0, "no population-0 card states a faith as if it were measured over people")
	_chk(card_false == 0, "and no populated card claims it has no population (%d)" % card_false)

	print("")
	print("FAITHDENOM RESULT: ", "ALL PASS" if _fails == 0 else "%d FAILURES" % _fails)
	get_tree().quit(0 if _fails == 0 else 1)
