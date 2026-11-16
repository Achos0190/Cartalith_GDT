extends "res://_pcl_base.gd"
## Ruling FO item 1: a settlement's founding year (`civ.founding_year`, `PlaceExtras::founded`, saved as `extras.founded`).
## Protects the engine's stamp, the Inspect page's "Founded" row (a year link on desktop and tablet, a plain row on the
## phone), Unknown as a dash and as an ABSENT key, year 0 as a real year, Inspect's edit mode and the hand drop's year.
##   A. Every settlement of the generated world carries `founded`, equal to minus the reference's age worked out here
##      independently from the population (60 + 240 log10(max(1, max(pop, 20) / 100)), rounded, clamped 30..1000),
##      strictly before year 0; `civ_settlement_details` agrees.
##   B. Inspect shows the year: desktop and tablet a link reading "<n> BC" whose real press moves the timeline to
##      exactly that year; the phone a plain "Founded" row with the same text.
##   C. Unknown (`founded_clear`): the key is ABSENT from `get_settlements` and `civ_settlement_details`, the row is a
##      dash, there is no link.
##   D. Year 0 is a year: `founded` 0 reads back as 0 (key present), and the row reads "0 AD", not a dash.
##   E. Edit mode (desktop and tablet): the Unknown/Set choice and the Year field exist; Set writes the timeline's
##      year; typing a year writes it; Unknown clears it.
##   F. Refusals: a year past +-1 000 000, and `founded` with `founded_clear`, return false and change nothing.
##   G. A hand-dropped settlement is founded in the year the timeline shows.
## Mutants (see `STATUS.md`): FOUNDED_SIGN (the Rust stamp's minus), FOUNDED_DASH_LINK (the unknown branch draws a link),
## FOUNDED_LINK_YEAR (the link moves to 0), FOUNDED_EDIT_SET (Set writes 0), FOUNDED_DROP_YEAR (the drop stamps 0).

func _age(pop: int) -> int:
	return clampi(int(roundf(60.0 + 240.0 * log(maxf(1.0, float(maxi(pop, 20)) / 100.0)) / log(10.0))), 30, 1000)


## The text of the "Founded" row's value on the page: the link's text when there is one, else the value label of a
## `Founded` field row. "" when the page has no such row.
func _founded_text(body: Node) -> String:
	var link := body.find_child("InspectFoundedLink", true, false) as Button
	if link != null:
		return link.text
	for l in body.find_children("*", "Label", true, false):
		if (l as Label).text == "Founded" and l.get_parent() is HBoxContainer:
			var kids := l.get_parent().get_children()
			var at: int = kids.find(l)
			if at >= 0 and at + 1 < kids.size() and kids[at + 1] is Label:
				return (kids[at + 1] as Label).text
	return ""


func _run() -> void:
	_prefix = "fo"
	var chrome := _mode != "phone"
	print("== A. the stamp ==")
	var all: Array = _bridge.settlements()
	var missing := 0
	var wrong := 0
	var non_neg := 0
	for i in all.size():
		var d: Dictionary = all[i]
		if not d.has("founded"):
			missing += 1
			continue
		var y := int(d["founded"])
		if y != -_age(int(d.get("population", 0))):
			wrong += 1
		if y >= 0:
			non_neg += 1
		var det: Dictionary = _bridge.civ_settlement_details(i)
		if int(det.get("founded", 999999)) != y:
			wrong += 1
	_ok(all.size() >= 3 and missing == 0, "every settlement has a founding year (%d of %d lack one)" % [missing, all.size()])
	_ok(wrong == 0, "each year is minus the age worked out from its population, and details agree (%d wrong)" % wrong)
	_ok(non_neg == 0, "every generated year is before year 0")
	var idx := _anchor
	var y0 := int((all[idx] as Dictionary)["founded"])
	_ok(y0 < 0 and y0 >= -1000, "the anchor's year %d is within -1000..-1" % y0)

	print("== B. Inspect shows it ==")
	_app.select("settlement", idx)
	await _settle(10)
	var body: Control = _app.right_dock_body
	var want := "%d BC" % -y0
	_ok(_founded_text(body) == want, "the Founded row reads %s (got '%s')" % [want, _founded_text(body)])
	if chrome:
		var link := body.find_child("InspectFoundedLink", true, false) as Button
		_ok(link != null, "desktop and tablet draw the year as a link")
		await _shot("inspect_known")
		if link != null:
			_ok(await _press(link, "the Founded link"), "the link was pressed")
			await _settle(8)
			_ok(_year() == y0, "its press moved the timeline to exactly year %d (now %d)" % [y0, _year()])
		_app.tl_set_year(0, true)
		await _settle(6)
	else:
		_ok(body.find_child("InspectFoundedLink", true, false) == null, "the phone's sheet keeps a plain row, no link")
		await _shot("inspect_known")

	print("== C. unknown is absent and a dash ==")
	_ok(_bridge.civ_edit_settlement(idx, {"founded_clear": true}), "founded_clear is accepted")
	_app.select("settlement", idx)
	await _settle(10)
	body = _app.right_dock_body
	_ok(not (_bridge.settlements()[idx] as Dictionary).has("founded"), "get_settlements omits the key when unknown")
	_ok(not _bridge.civ_settlement_details(idx).has("founded"), "civ_settlement_details omits it too")
	_ok(_founded_text(body) == "—", "the row is a dash (got '%s')" % _founded_text(body))
	_ok(body.find_child("InspectFoundedLink", true, false) == null, "and has no link")
	await _shot("inspect_unknown")
	## A founding-year edit feeds no road, territory or trade, so it must NOT mark the civ layer stale (no Recompute
	## offered); the control proves the check can fail: a history edit does.
	var stale_civ := func() -> bool: return (_bridge.world_gen.stale_stages() as Dictionary).has("civ")
	_ok(not stale_civ.call(), "a founding-year edit leaves the civ layer current (no Recompute offered)")
	_ok(_bridge.civ_edit_settlement(idx, {"history": "probe"}), "control: a history edit is accepted")
	_ok(stale_civ.call(), "control: and it does mark the civ layer stale")

	print("== D. year 0 is a year ==")
	_ok(_bridge.civ_edit_settlement(idx, {"founded": 0}), "founded 0 is accepted")
	_app.select("settlement", idx)
	await _settle(10)
	body = _app.right_dock_body
	var s0: Dictionary = _bridge.settlements()[idx]
	_ok(s0.has("founded") and int(s0["founded"]) == 0, "year 0 is present and 0, not absent")
	_ok(_founded_text(body) == "0 AD", "the row reads 0 AD (got '%s')" % _founded_text(body))

	if chrome:
		print("== E. edit mode ==")
		var tg := body.find_child("InspectEditToggle", true, false) as Button
		if not await _press(tg, "the Edit toggle"):
			return
		await _settle(8)
		body = _app.right_dock_body
		var mode := body.find_child("InspectEditFoundedMode", true, false) as OptionButton
		_ok(mode != null and mode.selected == 1, "edit mode draws the Founded choice on Set (a year is known)")
		var yr := body.find_child("InspectEditFoundedYear", true, false) as SpinBox
		_ok(yr != null and int(yr.value) == 0, "and the Year field reads 0")
		await _shot("edit_set")
		if yr != null:
			yr.value = -77
			await _settle(8)
			_ok(int(_bridge.civ_settlement_details(idx).get("founded", 999)) == -77, "typing -77 writes -77 to the engine")
		body = _app.right_dock_body
		mode = body.find_child("InspectEditFoundedMode", true, false) as OptionButton
		mode.item_selected.emit(0)
		await _settle(8)
		_ok(not _bridge.civ_settlement_details(idx).has("founded"), "Unknown clears it (key absent)")
		body = _app.right_dock_body
		_ok(body.find_child("InspectEditFoundedYear", true, false) == null, "and the Year field goes away")
		await _shot("edit_unknown")
		_app.tl_set_year(123, true)
		await _settle(6)
		body = _app.right_dock_body
		mode = body.find_child("InspectEditFoundedMode", true, false) as OptionButton
		mode.item_selected.emit(1)
		await _settle(8)
		_ok(int(_bridge.civ_settlement_details(idx).get("founded", 999)) == 123, "Set starts at the timeline's year (123)")
		_app.tl_set_year(0, true)
		await _settle(4)
		body = _app.right_dock_body
		var done := body.find_child("InspectEditToggle", true, false) as Button
		await _press(done, "Done editing")
		await _settle(6)

	print("== F. refusals ==")
	_ok(_bridge.civ_edit_settlement(idx, {"founded": -50}), "a set to -50 first")
	_ok(not _bridge.civ_edit_settlement(idx, {"founded": 2000000}), "a year past +1 000 000 is refused")
	_ok(not _bridge.civ_edit_settlement(idx, {"founded": -2000000}), "and one past -1 000 000")
	_ok(_bridge.civ_edit_settlement(idx, {"founded": 1000000}), "the limit itself is accepted")
	_ok(not _bridge.civ_edit_settlement(idx, {"founded": 5, "founded_clear": true}), "founded with founded_clear is refused")
	_ok(int(_bridge.civ_settlement_details(idx).get("founded", 999)) == 1000000, "and a refusal changed nothing")

	print("== G. a hand drop is founded in the shown year ==")
	_app.tl_set_year(123, true)
	await _settle(6)
	var before: int = _bridge.settlements().size()
	var a: Dictionary = _bridge.settlements()[idx]
	var made := -1
	for off in [Vector2(9, 0), Vector2(-9, 0), Vector2(0, 9), Vector2(0, -9), Vector2(14, 7), Vector2(-14, -7), Vector2(20, 0)]:
		var r := int(_bridge.civ_drop_settlement(float(a["x"]) + off.x, float(a["y"]) + off.y, "hamlet", _f1, "Dropton", false))
		if r >= 0 and _bridge.settlements().size() > before:
			made = r
			break
	_ok(made >= 0, "a hamlet was dropped (index %d)" % made)
	if made >= 0:
		_ok(int(_bridge.civ_settlement_details(made).get("founded", 999999)) == 123, "it was founded in year 123 (got %s)" % str(_bridge.civ_settlement_details(made).get("founded", "none")))
	_app.tl_set_year(0, true)
	await _settle(4)
