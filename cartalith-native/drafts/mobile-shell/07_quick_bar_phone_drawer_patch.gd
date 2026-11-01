## DRAFT, not loaded by Godot. MS-11 wiring: the quick bar's phone form moves from CQ-6's chip row in the tool
## sheet to the side drawer (draft 06). That is Q10's default; the drawer is mounted only if the owner says yes.
## Two targets, marked below: `shell/quick_bar_phone.gd` and `shell/dcc_shell.gd`.
##
## What stays: MENU's Quick section (`PhoneMenu._fill_quick()`), the `All…` list sheet (`QuickBarPhoneList`) and
## every desktop and tablet form (CQ-2's cluster, CQ-4's ring dock). Only the row in the sheet is retired.
## `QuickBarPhoneRow` and `set_phone_quick_row()` are removed once nothing calls them; grep both, and the probes,
## before deleting (`MISTAKES.md`, "Call a constant dead").
##
## NOT parse-checked (no Godot binary in the session that wrote it).

# ===================== shell/quick_bar_phone.gd =====================

## Draft 06, loaded by path (no `class_name`).
const _QuickDrawer := preload("res://shell/quick_drawer.gd")
## The side drawer, or null while there is nothing to show. That null is what keeps an empty bar drawing
## nothing on the phone (CQ-6's pixel-identity rule, `_quickbar_phoneid_probe.gd`).
var _drawer: Control

## Replaces `refresh()`. It has the same callers and the same early exits; it mounts and feeds the drawer instead
## of the row. An empty list removes the drawer, so no grip is drawn.
func refresh() -> void:
	if _app == null or not _app.is_phone():
		return
	var es := entries(_app)
	if es.is_empty():
		if _drawer != null:
			_app.set_phone_quick_drawer(null)
			_drawer.queue_free()
			_drawer = null
		return
	if _drawer == null:
		_drawer = _QuickDrawer.new()
		_drawer.call("setup", _app)
		_app.set_phone_quick_drawer(_drawer)
	_drawer.call("set_entries", es)
	_drawer.call("relayout")
	if _list != null and _list.visible:
		_list.show_entries(es)
	_app.refresh_phone_quick()

## For probes: the mounted drawer, or null.
func drawer() -> Control:
	return _drawer

# ===================== shell/dcc_shell.gd =====================

## Mounts the phone's quick drawer over the map: above the map and its floating controls, below the tool sheet
## and the bottom bar. A null argument does nothing; the caller frees the drawer itself.
## The z-order is the one thing to check on glass: the drawer must cover the map's floating controls, and must
## not cover the sheet's grab handle or the bar (MS-11's bar).
func set_phone_quick_drawer(d: Control) -> void:
	if d == null or _phone_root == null:
		return
	if d.get_parent() == null:
		_phone_root.add_child(d)
		## Directly above the map's container: everything added after it (the sheet, the bars) stays on top.
		## `viewport_content` is the map surface the phone root holds; place the drawer right after its
		## top-level child under `_phone_root`.
		var map_child: Node = viewport_content
		while map_child != null and map_child.get_parent() != _phone_root:
			map_child = map_child.get_parent()
		if map_child != null:
			_phone_root.move_child(d, map_child.get_index() + 1)

## Add to the phone's system-back chain, right after the domain fan (draft 03) and before the sheet levels:
## an open quick drawer closes first (proposal §4's order).
func _close_quick_drawer_for_back() -> bool:
	var qb = get("phone_quick")   ## DccApp's QuickBarPhone (`app.gd`: `phone_quick = QuickBarPhone.new()`)
	if qb == null:
		return false
	var d: Control = qb.call("drawer")
	return d != null and bool(d.call("close_if_open"))
