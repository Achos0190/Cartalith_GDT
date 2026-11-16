extends "res://_pcl_base.gd"
## PC-3 leftovers (`PC_SHELL_PROPOSAL.md` 3.2): the COLLAPSED strip's year slider, and one undo step for + Record year.
##   A. Collapsed strip (the default): `TimelineCollapsedSlider` exists on the desktop; its range is `tl_range()`; its value is
##      the engine's year. A REAL click on its left end, middle and right end writes the year through the one writer: the
##      engine year equals the slider value, left end = range min, right end = range max (within one step), and the value in
##      between lies strictly between. Pressing the slider does NOT expand the strip.
##   B. Moving the year elsewhere (the engine) moves the slider (repaint, no feedback write).
##   C. Expanded strip: a REAL click on `RecordYear` at a year not yet recorded adds it to the engine's recorded years;
##      a REAL Ctrl+Z removes exactly that year and the Edit menu's Undo label named it before; a REAL Ctrl+Shift+Z records it
##      again; a second Ctrl+Z removes it; with the year then removed by hand a third Ctrl+Z says it was already gone and does
##      not touch other years. Recording an already-recorded year makes NO step.
##   D. TABLET (`--force-touch`): the collapsed strip is 45 px (44 target + 1 rule, Ruling FO item 5) and has the slider.
## Mutants: SLIDER_NO_WRITE (`tl_set_year` removed from the slider's handler), SLIDER_RANGE (max_value from min), UNDO_NOT_REMOVED
## (`civ_remove_year` -> `civ_add_year` in `year_record_history.gd::undo`), UNDO_NO_STEP (`note_recorded` call removed),
## UNDO_NEW_ONLY (the `was_new` test dropped).

func _leg_slider() -> void:
	var tablet := DccTheme.is_tablet()
	print("== A. the collapsed slider ==")
	_app._tl_expanded = false
	_app._fill_timeline_strip()
	await _settle(6)
	var sl := _app.timeline_row.find_child("TimelineCollapsedSlider", true, false) as HSlider
	_ok(sl != null, "the collapsed strip has the slider (%s)" % ("tablet, Ruling FO item 5" if tablet else "desktop"))
	if tablet and sl != null:
		var bar_h: float = _app.timeline_bar.size.y
		_ok(is_equal_approx(bar_h, 45.0), "D. the tablet's collapsed strip is 44 px of target under its 1 px rule (bar %.1f)" % bar_h)
		_ok(sl.size.y >= 44.0 and sl.size.y <= 44.5, "D. the tablet's slider is 44 px tall (%.1f)" % sl.size.y)
	if sl == null:
		return
	var rng: Dictionary = _app.tl_range()
	_ok(int(sl.min_value) == int(rng["min"]) and int(sl.max_value) == int(rng["max"]), "its range is tl_range() (%d..%d vs %d..%d)" % [sl.min_value, sl.max_value, int(rng["min"]), int(rng["max"])])
	await _shot("collapsed")
	var r := sl.get_global_rect()
	await _click_at(Vector2(r.position.x + 2.0, r.get_center().y))
	var y_left := _year()
	await _click_at(Vector2(r.position.x + r.size.x * 0.5, r.get_center().y))
	var y_mid := _year()
	await _click_at(Vector2(r.end.x - 2.0, r.get_center().y))
	var y_right := _year()
	var step_tol := int(ceil((sl.max_value - sl.min_value) / maxf(r.size.x, 1.0))) + 2
	_ok(absi(y_left - int(sl.min_value)) <= step_tol, "a click at the left end writes the range minimum (%d vs %d, tol %d)" % [y_left, int(sl.min_value), step_tol])
	_ok(absi(y_right - int(sl.max_value)) <= step_tol, "a click at the right end writes the range maximum (%d vs %d)" % [y_right, int(sl.max_value)])
	_ok(y_mid > y_left and y_mid < y_right, "the middle writes a year strictly between (%d < %d < %d)" % [y_left, y_mid, y_right])
	_ok(int(sl.value) == y_right or absi(int(sl.value) - _year()) <= 1, "the slider's value follows the year (%d vs %d)" % [int(sl.value), _year()])
	_ok(not _app._tl_expanded, "pressing the slider did not expand the strip")
	print("== B. repaint ==")
	_app.tl_set_year(int(rng["min"]) + 10, true)
	await _settle(4)
	_ok(int((_app.timeline_row.find_child("TimelineCollapsedSlider", true, false) as HSlider).value) == int(rng["min"]) + 10, "an engine-side year write moves the slider")


func _run() -> void:
	_prefix = "strip"
	await _leg_slider()
	print("== C. undo for + Record year ==")
	_app._tl_expanded = true
	_app._fill_timeline_strip()
	await _settle(6)
	var yr := 123
	_ok(not Array(_bridge.get_civ_timeline_years()).has(yr), "year %d is not recorded yet" % yr)
	_app.tl_set_year(yr, true)
	await _settle(4)
	var rec := _app.timeline_row.find_child("RecordYear", true, false) as Button
	if not await _press(rec, "+ Record year"):
		return
	await _settle(6)
	_ok(Array(_bridge.get_civ_timeline_years()).has(yr), "the click recorded %d" % yr)
	_ok(_app.undo_target_label() == "record year %d" % yr, "Edit > Undo is labelled for it (got '%s')" % _app.undo_target_label())
	await _shot("recorded")
	await _key(KEY_Z, true)
	_ok(not Array(_bridge.get_civ_timeline_years()).has(yr), "a real Ctrl+Z removed exactly that year")
	await _key(KEY_Z, true, true)
	_ok(Array(_bridge.get_civ_timeline_years()).has(yr), "a real Ctrl+Shift+Z recorded it again")
	var before := Array(_bridge.get_civ_timeline_years()).duplicate()
	var hist_n: int = _app.year_history.hist.size()
	rec = _app.timeline_row.find_child("RecordYear", true, false) as Button
	await _press(rec, "+ Record year again (already recorded)")
	await _settle(4)
	_ok(_app.year_history.hist.size() == hist_n and Array(_bridge.get_civ_timeline_years()) == before, "recording an already-recorded year makes no step (%d steps)" % _app.year_history.hist.size())
	await _key(KEY_Z, true)
	_ok(not Array(_bridge.get_civ_timeline_years()).has(yr), "Ctrl+Z again removes it")
	_bridge.civ_add_year(yr)
	_bridge.civ_remove_year(yr)
	await _key(KEY_Z, true, true)
	_bridge.civ_remove_year(yr)
	var others := Array(_bridge.get_civ_timeline_years()).duplicate()
	await _key(KEY_Z, true)
	_ok(Array(_bridge.get_civ_timeline_years()) == others, "an undo over a year already removed by hand touches no other year")
