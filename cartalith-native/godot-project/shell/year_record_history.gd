extends RefCounted
## PC-3 leftover (`PC_SHELL_PROPOSAL.md` 3.2, "+ Record year ... one undo step"): the undo history of the timeline
## strip's **+ Record year**. The engine keeps no undo step for a recorded year (`civ_add_year` writes a snapshot
## and `civ_remove_year` deletes one, neither ledgered), so this keeps the one thing needed: the YEARS the strip
## recorded, newest last, with a cursor. Undo removes the year (`civ_remove_year`), redo records it again.
##
## It is the third small history beside the look history and the planner's stop history, and it routes the same
## way (`DccApp._undo_last_impl`): a step belongs to this history when it is applied AND either nothing was done
## in the other histories since it (their depth fingerprint equals the one stamped at record time) or nothing else
## can be undone. A recorded year the user has since removed by hand is dropped with a status line, never
## "undone" over nothing; a year already recorded makes no step (the call would be a no-op).
##
## Must never: write a year it did not record; claim an undo while the year is still in `tl_recorded_years()`
## (the check after the engine call is the truth); keep steps across a world (`reset()` on a new world).
## No `class_name` (preloaded by `app.gd`), so no `--import` pass.

## The app (`DccApp`): bridge, the other histories and the CIVIL workspace's repaint.
var app = null
## Applied steps, oldest first: `{year, sig, label}`. `cursor` of them are applied; the rest are redoable.
var hist: Array = []
var cursor := 0
## The fingerprint stamped when the newest undo happened, so redo is offered only while nothing else moved.
var _redo_sig := ""
## The most steps kept. A labelled judgement: a person records a handful of years; 50 bounds a long session.
const MAX_STEPS := 50

func can_undo() -> bool:
	return cursor > 0

func can_redo() -> bool:
	return cursor < hist.size()

## The other histories' depths, as `JourneyPlannerView._history_sig()` does: an undo elsewhere changes the redo half
## but not the depth, and the depth is what says "everything after my step has been undone".
func _sig() -> String:
	var b = app.bridge
	if b == null or b.generating:
		return "generating"
	var st: Dictionary = b.undo_stats()
	return "%s|%s|%s|%d|%d" % [str(st.get("depth", "")), str(st.get("way_depth", "")),
		str(st.get("annotation_depth", "")), int(app.look_history.cursor), int(app.undo_groups.cursor)]

func _others_can_undo() -> bool:
	return app.look_history.can_undo() or app.bridge.can_undo() or app.undo_groups.wants_undo()

## True when the next Edit ▸ Undo is this history's.
func wants_undo() -> bool:
	if not can_undo() or app.bridge.generating:
		return false
	return String((hist[cursor - 1] as Dictionary).get("sig", "")) == _sig() or not _others_can_undo()

## The Redo mirror.
func wants_redo() -> bool:
	if not can_redo() or app.bridge.generating:
		return false
	return _redo_sig == _sig() or not (app.look_history.can_redo() or app.bridge.redo_available()
		or app.undo_groups.wants_redo())

func undo_label() -> String:
	return String((hist[cursor - 1] as Dictionary).get("label", "")) if can_undo() else ""

func redo_label() -> String:
	return String((hist[cursor] as Dictionary).get("label", "")) if can_redo() else ""

## Called by the strip after `civ_add_year(year)` recorded a NEW year. Drops the redo tail (a new edit), stamps the
## fingerprint of the other histories, caps the list.
func note_recorded(year: int, label: String) -> void:
	hist.resize(cursor)
	hist.append({"year": year, "sig": _sig(), "label": label})
	if hist.size() > MAX_STEPS:
		hist.pop_front()
	cursor = hist.size()

## Undoes the newest applied step: removes its year. Returns `{ok, text}`; a year that is no longer recorded drops
## the step with a reason (nothing was undone).
func undo() -> Dictionary:
	if not can_undo():
		return {"ok": false, "text": "Nothing to undo."}
	var e: Dictionary = hist[cursor - 1]
	var y := int(e["year"])
	if not Array(app.bridge.get_civ_timeline_years()).has(y):
		hist.remove_at(cursor - 1)
		cursor -= 1
		return {"ok": false, "text": "That recorded year (%d) was already removed, so there is nothing to undo." % y}
	app.bridge.civ_remove_year(y)
	cursor -= 1
	_redo_sig = _sig()
	app.civ_year_records_changed()
	return {"ok": true, "text": String(e["label"])}

## The mirror: records the year again.
func redo() -> Dictionary:
	if not can_redo():
		return {"ok": false, "text": "Nothing to redo."}
	var e: Dictionary = hist[cursor]
	app.bridge.civ_add_year(int(e["year"]))
	cursor += 1
	app.civ_year_records_changed()
	return {"ok": true, "text": String(e["label"])}

## A new world drops the steps: a year of the old world is not a year of this one.
func reset() -> void:
	hist.clear()
	cursor = 0
	_redo_sig = ""
