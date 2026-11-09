extends VBoxContainer
## The landmark filter's controls (owner, 2026-10-19: *"In the layers I'd also
## like an option to turn landmarks off globally or filter per type. The
## landmarks themselves should also scale according to zoom level."*).
##
## One body, built in two places: the map's Layers popover (`layers_popover.gd`,
## the desktop popover and the phone sheet) and CARTO ▸ Layers on the rail
## (`cartography_workspace.gd`). Both instances write through `ViewportHost` and
## re-read from it on `landmark_filter_changed`, so the two can never disagree --
## the same one-state-two-mirrors rule the Visible layers rows already follow.
## The master Landmarks switch is NOT here: it is the existing Landmarks row in
## each surface's Visible layers list, which now persists through
## `ViewportHost.set_landmarks_shown_by_user()`.
##
## What it draws, top to bottom:
##   - Show all / Hide all, over the rows listed below;
##   - one toggle per type, labelled with its mark count on this world;
##   - "Scale with zoom", the zoom law switch (`map_overlay.gd::_landmark_zoom_k`);
##   - one note saying what the rows do and what they leave alone.
##
## **Which types get a row** -- derived from the engine's own registry
## (`bridge.landmark_kinds()`, `cartalith_civ::landmark::kinds()`), not restated
## here: every kind the generator can place (`buildable`), plus any kind this
## world actually carries that the registry does not mark buildable (a project
## written by another build), plus the hand-placed POI icons row
## (`map_overlay.gd::LANDMARK_MANUAL_POI`). A declared-but-unbuilt kind with no marks
## is left out: it can never appear, and a switch for it would be a control
## that does nothing.
##
## **Touch.** Rows are rebuilt on every refresh, so they escape the one-shot
## `phone_fit()`/`tablet_fit()` passes (`MISTAKES.md`, "floor a control that a
## rebuild creates"); every button here floors itself on BOTH axes to
## `TOUCH_FLOOR` when `DccTheme.is_touch()` is true, at the call site.
##
## Must never hold a copy of the filter state: every value it shows is read
## from `host` at refresh time.

## The touch floor for every tappable control here, in canvas units (the phone
## sheet applies its own content scale on top, as every phone surface does).
## 44, the project's touch target (`role_px("btn_min_h")`'s tablet value and
## `phone_fit()`'s floor).
const TOUCH_FLOOR := 44.0
## `map_overlay.gd`, for its `LANDMARK_MANUAL_POI` key. A preload, not a class
## name: that script declares none (`viewport_host.gd` reaches it the same way).
const OVERLAY := preload("res://map_overlay.gd")

var host: ViewportHost
var bridge: EngineBridge
## The row keys the current build listed, in order -- what Show all / Hide all
## act on.
var _keys: Array = []
## A refresh is already queued for the end of this frame (`_queue_refresh`).
var _pending := false

## Wires the panel to the shell and builds it once. Call after it is parented.
func setup(h: ViewportHost, b: EngineBridge) -> void:
	host = h
	bridge = b
	add_theme_constant_override("separation", 2)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	host.landmark_filter_changed.connect(_queue_refresh)
	host.layer_visibility_changed.connect(func(layer: String, _on: bool):
		if layer == "landmarks": _queue_refresh())
	## Counts change when a pass lands or a world arrives. Deferred so the
	## viewport's own refresh (which hands the overlay the new list) runs first.
	bridge.generation_finished.connect(func(_ok: bool): _queue_refresh())
	bridge.world_loaded.connect(_queue_refresh)
	bridge.landmark_finished.connect(func(_r: Dictionary): _queue_refresh())
	## A hand-placed POI count can change with no signal here (the Icon tool);
	## re-reading when the panel is shown again keeps what the user SEES current.
	visibility_changed.connect(func(): if is_visible_in_tree(): _queue_refresh())
	refresh()

## Coalesces several triggers in one frame into one rebuild.
func _queue_refresh() -> void:
	if _pending:
		return
	_pending = true
	(func():
		_pending = false
		if is_instance_valid(self) and is_inside_tree():
			refresh()).call_deferred()

## `{key, label}` rows, in the engine registry's order (its families are
## already grouped there), then unknown kinds this world carries, then the
## hand-placed row. `counts` is `ViewportHost.landmark_kind_counts()`.
func _rows(counts: Dictionary) -> Array:
	var out: Array = []
	var seen := {}
	for k in bridge.landmark_kinds():
		var spec: Dictionary = k
		var key := String(spec.get("key", ""))
		if key == "" or seen.has(key):
			continue
		if bool(spec.get("buildable", false)) or int(counts.get(key, 0)) > 0:
			out.append({"key": key, "label": String(spec.get("label", key.capitalize()))})
			seen[key] = true
	var extra: Array = []
	for key in counts:
		if String(key) != OVERLAY.LANDMARK_MANUAL_POI and not seen.has(String(key)):
			extra.append(String(key))
	extra.sort()
	for key in extra:
		out.append({"key": key, "label": String(key).capitalize()})
	out.append({"key": OVERLAY.LANDMARK_MANUAL_POI, "label": "Hand-placed POI icons"})
	return out

## Floors one control on both axes when the session is touch (see the header).
static func _floor(c: Control) -> void:
	if DccTheme.is_touch():
		c.custom_minimum_size = Vector2(maxf(c.custom_minimum_size.x, TOUCH_FLOOR),
			maxf(c.custom_minimum_size.y, TOUCH_FLOOR))

## Empties and rebuilds every row from `host`. Safe to call at any time.
func refresh() -> void:
	if host == null or bridge == null:
		return
	for c in get_children():
		remove_child(c)
		c.queue_free()
	var counts: Dictionary = host.landmark_kind_counts()
	var rows := _rows(counts)
	_keys = []
	for r in rows:
		_keys.append(String(r["key"]))

	var acts := HBoxContainer.new()
	acts.add_theme_constant_override("separation", 6)
	add_child(acts)
	var show_all := DccWidgets.chip(acts, "Show all", func():
		host.set_landmark_kinds_visible(_keys, true))
	show_all.name = "LmShowAll"
	show_all.tooltip_text = "Turn every landmark type below back on. The Landmarks switch above still gates them all."
	var hide_all := DccWidgets.chip(acts, "Hide all", func():
		host.set_landmark_kinds_visible(_keys, false))
	hide_all.name = "LmHideAll"
	hide_all.tooltip_text = "Turn every landmark type below off, including hand-placed POI icons."
	for b in [show_all, hide_all]:
		_floor(b)

	var master_on: bool = host.layer_visible("landmarks")
	for r in rows:
		var key := String(r["key"])
		var n := int(counts.get(key, 0))
		var tip := ("Hand-placed POI-family icons (the Icon tool). Authored, so the Landmarks switch leaves them; this row hides them."
			if key == OVERLAY.LANDMARK_MANUAL_POI else
			"Shows or hides every %s landmark on the map, its name and its click target. %d on this world." % [String(r["label"]).to_lower(), n])
		var cb := DccWidgets.toggle(self, "%s  %d" % [String(r["label"]), n],
			host.landmark_kind_visible(key),
			func(on: bool): host.set_landmark_kind_visible(key, on), tip)
		cb.name = "LmKind_" + key
		cb.set_meta("landmark_kind", key)
		## Generated rows read as gated when the master is off; the hand-placed
		## row does not answer to the master, so it never dims.
		if not master_on and key != OVERLAY.LANDMARK_MANUAL_POI:
			cb.modulate.a = 0.55
		_floor(cb)

	var law := DccWidgets.toggle(self, "Scale with zoom", host.landmark_zoom_law(),
		func(on: bool): host.set_landmark_zoom_law(on),
		"On: landmark marks grow gently as you zoom in past the opening view (to at most 2.5x) and never shrink below their fit size. Off: the old behaviour, where a mark grew in step with the map without limit.")
	law.name = "LmZoomLaw"
	_floor(law)

	DccWidgets.note(self,
		"Counts are marks on this world. A hidden type is not drawn, named, "
		+ "hovered or clicked, and an export follows the screen. Choices are kept "
		+ "on this device across worlds and regenerates.")
