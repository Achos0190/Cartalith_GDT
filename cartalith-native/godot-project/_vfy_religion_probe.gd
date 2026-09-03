extends Node
## VERIFIER probe (independent of `_religionpanel_probe.gd`): does the Religion
## panel ever show a settlement a faith it does not hold?
##
## Compares the RENDERED text of every settlement row against the engine's own
## report for that settlement, key by key -- the union of `adherents`' live
## head-counts and the `religion` plurality, because those are two facts and a
## population-0 settlement has only the second (see the widening comment at the
## `want` array below). Counts:
##   A) rows whose printed faith set != the engine's faith set
##   B) rows with >=2 faiths rendered as a single faith
##   C) rows with no `religion` key that do NOT carry a dashed reason
##   D) hover-card lines printing a bare `0%` or a self-contradicting `100%`
## Then edits a culture and re-reads the panel.

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
	print("settlements: ", bridge.settlements().size())

	bridge.civ_set_faction_field(1, "religion", "sun_cult")
	bridge.civ_set_faction_field(3, "religion", "old_gods")
	ws._religion_years = 50
	ws._religion_run()

	var places: Array = bridge.settlements()
	print("after run: ", places.size(), " settlements; status=", ws._religion_status)

	# ---- engine-side truth -------------------------------------------------
	var mixed := 0
	var no_key := 0
	var faith_names := {}
	for p in places:
		var d: Dictionary = p
		if not d.has("religion"):
			no_key += 1
			continue
		var ad: Dictionary = d.get("adherents", {})
		var live := 0
		for k in ad.keys():
			if int(ad[k]) > 0:
				live += 1
				faith_names[String(k)] = ws._religion_label(String(k))
		## Same widening as below: a key that only ever appears as a population-0
		## settlement's plurality still has to be in the vocabulary, or the EXTRA
		## test cannot see it being printed on the wrong row.
		faith_names[String(d["religion"])] = ws._religion_label(String(d["religion"]))
		if live >= 2:
			mixed += 1
	print("engine: mixed-adherence settlements = ", mixed, " ; settlements missing the key = ", no_key)
	_chk(mixed > 0, "the fixture actually produces mixed settlements to get wrong (%d)" % mixed)

	# ---- rendered text -----------------------------------------------------
	var lines := _render(ws)
	# Row anchors are "<name> · <n> people"; the faith line is the next line
	# that contains "◆" or "◇" or begins with "—".
	var bad_set := 0
	var single_for_mixed := 0
	var undashed := 0
	var checked := 0
	var examples: Array = []
	for p in places:
		var d: Dictionary = p
		var nm := String(d.get("name", ""))
		if nm.is_empty():
			continue
		var anchor := -1
		for i in lines.size():
			var t := String(lines[i])
			if t.begins_with(nm + " · "):
				anchor = i
				break
		if anchor < 0:
			continue
		var faith_line := ""
		for j in range(anchor + 1, mini(anchor + 4, lines.size())):
			var t := String(lines[j])
			if t.contains("◆") or t.contains("◇") or t.begins_with("—"):
				faith_line = t
				break
		checked += 1
		if not d.has("religion"):
			if not faith_line.begins_with("— no adherence"):
				undashed += 1
				examples.append("UNDASHED: " + nm + " -> " + faith_line)
			continue
		## `adherents` is not the whole of what a settlement holds, and treating
		## it as the whole was this oracle's own gap. `get_settlements()` emits
		## TWO religion facts: `religion`, the plurality of a real share vector,
		## and `adherents`, head-counts that `lib.rs` omits when they are zero.
		## A settlement created at population 0 has a real plurality and an
		## empty `adherents` -- 158 of 173 on this fixture -- so an oracle built
		## from `adherents` alone says such a settlement holds NOTHING, and
		## every faith the panel names for it reads as fabricated.
		##
		## Widened 2026-09-03 to the union of the two, which is what the engine
		## actually reports. It is not a relaxation: a faith in neither is still
		## an EXTRA, and the plurality is checked against this settlement's own
		## key, never against the world's vocabulary.
		var ad: Dictionary = d.get("adherents", {})
		var want: Array = []
		for k in ad.keys():
			if int(ad[k]) > 0:
				want.append(ws._religion_label(String(k)))
		var plur_lbl: String = ws._religion_label(String(d["religion"]))
		if not want.has(plur_lbl):
			want.append(plur_lbl)
		var printed := 0
		for lbl in want:
			if faith_line.contains(String(lbl)):
				printed += 1
		if printed != want.size():
			bad_set += 1
			if examples.size() < 6:
				examples.append("SETMISS: " + nm + " want=" + str(want) + " got=" + faith_line)
		if want.size() >= 2 and faith_line.count("◆") + faith_line.count("◇") < 2:
			single_for_mixed += 1
			if examples.size() < 6:
				examples.append("SINGLE: " + nm + " want=" + str(want) + " got=" + faith_line)
		# a faith the settlement does NOT hold must not appear
		for key in faith_names.keys():
			var lbl2 := String(faith_names[key])
			if faith_line.contains(lbl2) and not want.has(lbl2):
				bad_set += 1
				if examples.size() < 6:
					examples.append("EXTRA: " + nm + " showed " + lbl2 + " -> " + faith_line)
	print("rows matched to rendered text: ", checked, " of ", places.size())
	for e in examples:
		print("   ", e)
	_chk(checked >= places.size() - 2, "essentially every settlement was located in the render")
	_chk(bad_set == 0, "no row prints a faith set that differs from the engine's (%d bad)" % bad_set)
	_chk(single_for_mixed == 0,
		"no mixed settlement is rendered as a single faith (%d)" % single_for_mixed)
	_chk(undashed == 0, "every keyless settlement is dashed with its reason (%d undashed)" % undashed)

	# bare 0% / contradictory 100%
	var zero_pct := 0
	var hundred_bad := 0
	for t in lines:
		var s := String(t)
		if s.contains("◆") or s.contains("◇"):
			if s.contains(" 0.0%"):
				zero_pct += 1
			if s.contains("100.0%") and (s.count("◆") + s.count("◇")) > 1:
				hundred_bad += 1
	_chk(zero_pct == 0, "no panel row prints a bare 0.0%% for a real congregation (%d)" % zero_pct)
	_chk(hundred_bad == 0, "no row reads 100.0%% while listing another faith (%d)" % hundred_bad)

	# hover card
	var ov = _app.viewport.overlay
	var card_zero := 0
	var card_hundred := 0
	var card_rows := 0
	for p in places:
		var fl: Array = ov._faith_lines(p)
		if fl.is_empty():
			continue
		card_rows += 1
		var joined := " / ".join(PackedStringArray(fl))
		if joined.contains(" 0%"):
			card_zero += 1
		if joined.contains(" 100%") and joined.contains("also"):
			card_hundred += 1
	print("hover cards with faith lines: ", card_rows)
	_chk(card_zero == 0, "no hover card prints ` 0%%` (%d)" % card_zero)
	_chk(card_hundred == 0, "no hover card reads 100%% then lists another faith (%d)" % card_hundred)

	# divergence count vs the panel's sentence
	var faiths: PackedStringArray = ws._religion_faction_column("religion")
	var div := 0
	for p in places:
		var d: Dictionary = p
		if not d.has("religion"):
			continue
		var f := int(d.get("faction", 0))
		if f <= 0 or f > faiths.size() or faiths[f - 1].is_empty():
			continue
		if String(d["religion"]) != faiths[f - 1]:
			div += 1
	var joined_all := " | ".join(PackedStringArray(lines))
	print("divergent settlements (independently counted): ", div)
	# Wording re-cut 2026-09-03: the sentence now carries a denominator and reads
	# "N of M settlements no longer share their ruler's state religion." The old
	# "N settlements no longer follow" is gone. Match the count, not the phrasing
	# around it, so a future re-word does not fail a check about arithmetic.
	_chk(joined_all.contains("%d of " % div) and joined_all.contains(
		"no longer share their ruler's state"),
		"the panel's divergence sentence matches an independent count (%d)" % div)

	# ---- the culture gap ---------------------------------------------------
	_chk(not joined_all.contains("STALE"), "no staleness claim right after a run")
	var before: PackedStringArray = ws._religion_culture_key.duplicate()
	var cults: PackedStringArray = bridge.civ_culture_vocabulary()
	var target := ""
	for c in cults:
		if String(c) != before[0]:
			target = String(c)
			break
	var ok: bool = bridge.civ_set_faction_field(1, "culture", target)
	var after_lines := _render(ws)
	var ja := " | ".join(PackedStringArray(after_lines))
	var eng: Dictionary = bridge.civ_belief_run(0)
	print("culture ", before[0], " -> ", target, " ok=", ok, " ; engine seeded=", eng.get("seeded"))
	_chk(ok, "the culture edit was accepted")
	_chk(bool(eng.get("seeded", true)) == false, "the ENGINE still reports the layer current")
	_chk(ja.contains("STALE"), "the panel does NOT claim currency it does not have")
	_chk(ja.contains("culture has changed"), "and names culture as the reason")

	print("")
	print("VFY RESULT: ", "ALL PASS" if _fails == 0 else "%d FAILURES" % _fails)
	get_tree().quit(0 if _fails == 0 else 1)
