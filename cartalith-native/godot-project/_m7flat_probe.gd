extends Node
## PC-9 finding M7 probe (`PC_SHELL_PROPOSAL.md` s.3.8: "Three-level nesting is flattened"), inside the real
## shell, for the four submenus whose third level became a captioned band of the second:
##   Window > 3D capability > FORCE TIER; Preferences > Lighting rig defaults > AZIMUTH / ELEVATION / AMBIENT /
##   MULTIDIRECTIONAL; Preferences > Tile size · LOD levels > LOD LEVELS; Preferences > Atlas cache > SIZE CAP · GB.
## (The fifth path, Assets > Landmark types > family > type, was settled by Ruling EH item 2 as a window, not a
## band: `_lmtypes_probe.gd` is its probe.)
##
##   desktop: godot --path . --resolution 1600x1000 _m7flat_probe.tscn
##   touch:   godot --path . --resolution 1600x1000 _m7flat_probe.tscn -- --force-touch
##
## Windowed, from a project copy with its own `config/name` and a wiped user dir. The only argument read is
## `--force-touch`; anything else aborts with exit 2.
##
## What it protects. Every number and caption is a literal, never read back from `menus.gd`:
##   A/B. none of the four popups has a submenu row any more (the third level is gone), each holds its caption,
##        and each holds its rungs by id, once each: 802-806 (5), 170-177 + 190-196 + 210-215 + 230-235 (27),
##        140-142 + 150-158 (12), 630-636 (7).
##   C. one rung is checked per ladder after a refresh: 1 in FORCE TIER, 4 in the lighting popup, 2 in the tile
##      popup (tile size and LOD depth), 1 in the atlas popup's cap band.
##   D. pressing a rung does what its handler did: a LOD rung other than the stored depth (3 by default, so 155)
##      stores that depth and moves the check; cap 632 stores
##      1.0 GB and moves the check; lighting 174 stores 180 deg.
##   E. the Preferences row "Tile size · LOD levels" still reads its tile size, as before the flatten.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`; exit 0 pass, 1 fail, 2 bad argument or watchdog.

const ID_CLEAR_ATLAS := 149
const ID_LIGHT_RESET := 250

var _fails := 0
var _checks := 0
var _app: Node


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


## The popup that holds item `id` AND a separator captioned `caption` (ids repeat across popups: 82 is a
## Deselect row and an Atlas row, so an id alone does not name a popup).
func _popup_with(id: int, caption: String) -> PopupMenu:
	var stack: Array = [_app]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
			if c is PopupMenu and (c as PopupMenu).get_item_index(id) >= 0:
				var pm := c as PopupMenu
				for i in pm.item_count:
					if pm.is_item_separator(i) and pm.get_item_text(i) == caption:
						return pm
	return null


func _has_band(pm: PopupMenu, caption: String) -> bool:
	for i in pm.item_count:
		if pm.is_item_separator(i) and pm.get_item_text(i) == caption:
			return true
	return false


func _ids_present(pm: PopupMenu, ids: Array) -> bool:
	for id in ids:
		if pm.get_item_index(int(id)) < 0:
			return false
	return true


func _count_checked(pm: PopupMenu, ids: Array) -> int:
	var n := 0
	for id in ids:
		var i := pm.get_item_index(int(id))
		if i >= 0 and pm.is_item_checked(i):
			n += 1
	return n


func _has_submenu(pm: PopupMenu) -> bool:
	for i in pm.item_count:
		if pm.get_item_submenu(i) != "":
			return true
	return false


func _unique_ids(pm: PopupMenu) -> bool:
	var seen := {}
	for i in pm.item_count:
		if pm.is_item_separator(i):
			continue
		var id := pm.get_item_id(i)
		if seen.has(id):
			return false
		seen[id] = true
	return true


func _ranges(spec: Array) -> Array:
	var out: Array = []
	for r in spec:
		for k in range(int(r[0]), int(r[0]) + int(r[1])):
			out.append(k)
	return out


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		if a != "--force-touch":
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch only" % a)
			get_tree().quit(2)
			return
	get_tree().create_timer(300.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(8)

	var force_ids := _ranges([[802, 5]])
	var light_ids := _ranges([[170, 8], [190, 7], [210, 6], [230, 6]])
	var tile_ids := _ranges([[140, 3], [150, 9]])
	var cap_ids := _ranges([[630, 7]])

	print("== A/B. the four popups ==")
	var cap := _popup_with(802, "FORCE TIER")
	_ok(cap != null, "3D capability holds a FORCE TIER band over rung 802")
	if cap != null:
		_ok(not _has_submenu(cap), "3D capability has no submenu row")
		_ok(_ids_present(cap, force_ids) and _unique_ids(cap), "its five rungs 802-806 are present once each")
		_ok(cap.get_item_index(801) >= 0, "Re-test 3D (801) is still its row")
	var light := _popup_with(170, "AZIMUTH")
	_ok(light != null, "Lighting rig defaults holds an AZIMUTH band over rung 170")
	if light != null:
		_ok(not _has_submenu(light), "the lighting popup has no submenu row")
		_ok(_ids_present(light, light_ids) and _unique_ids(light), "its 27 rungs are present once each")
		for cap_text in ["ELEVATION", "AMBIENT", "MULTIDIRECTIONAL"]:
			_ok(_has_band(light, cap_text), "the lighting popup has a %s band" % cap_text)
		_ok(light.get_item_index(ID_LIGHT_RESET) >= 0, "Use the engine's own rig (250) is still its row")
	var tile := _popup_with(150, "LOD LEVELS")
	_ok(tile != null, "Tile size · LOD levels holds a LOD LEVELS band over rung 150")
	if tile != null:
		_ok(not _has_submenu(tile), "the tile popup has no submenu row")
		_ok(_ids_present(tile, tile_ids) and _unique_ids(tile),
			"its 3 tile sizes and 9 LOD depths are present once each")
		_ok(tile.get_item_index(140) == 0, "the tile sizes stay first (rows 0-2: the refresher indexes them)")
	var atlas := _popup_with(630, "SIZE CAP · GB")
	_ok(atlas != null, "Atlas cache holds a SIZE CAP · GB band over rung 630")
	if atlas != null:
		_ok(not _has_submenu(atlas), "the Atlas cache popup has no submenu row")
		_ok(_ids_present(atlas, cap_ids) and _unique_ids(atlas), "its seven cap rungs are present once each")
		_ok(atlas.get_item_index(ID_CLEAR_ATLAS) >= 0, "Clear atlas cache now… is still its row")

	print("== C. one rung checked per ladder ==")
	if cap != null:
		cap.about_to_popup.emit()
		_ok(_count_checked(cap, force_ids) == 1, "FORCE TIER has exactly one checked rung")
	if light != null:
		light.about_to_popup.emit()
		_ok(_count_checked(light, light_ids) == 4,
			"the lighting popup has exactly four checked rungs (one per ladder)")
	if tile != null:
		tile.about_to_popup.emit()
		_ok(_count_checked(tile, tile_ids) == 2, "the tile popup has exactly two checked rungs")
	if atlas != null:
		atlas.about_to_popup.emit()
		_ok(_count_checked(atlas, cap_ids) == 1, "the atlas cap band has exactly one checked rung")

	print("== D. pressing a rung ==")
	var bd0: int = DccSettings.bake_depth()
	var pick := 5 if bd0 != 5 else 6   ## a depth that is not the stored one: pressing the stored one proves nothing
	if tile != null:
		tile.id_pressed.emit(150 + pick)
		await _settle(3)
		_ok(DccSettings.bake_depth() == pick, "LOD rung %d stored bake depth %d" % [150 + pick, pick])
		_ok(tile.is_item_checked(tile.get_item_index(150 + pick))
				and not tile.is_item_checked(tile.get_item_index(150 + bd0)),
			"and moved the check to it")
		DccSettings.set_bake_depth(bd0)
	var cap0: float = DccSettings.atlas_cap_gb()
	if atlas != null:
		atlas.id_pressed.emit(632)
		await _settle(3)
		_ok(is_equal_approx(DccSettings.atlas_cap_gb(), 1.0), "cap rung 632 stored 1.0 GB")
		_ok(atlas.is_item_checked(atlas.get_item_index(632)), "and checked it")
		DccSettings.set_atlas_cap_gb(cap0)
	var had_light: bool = DccSettings.has_lighting_default("sun_az_deg")
	var az0: float = float(DccSettings.lighting_defaults().get("sun_az_deg", -1.0))
	if light != null:
		light.id_pressed.emit(174)
		await _settle(3)
		_ok(is_equal_approx(float(DccSettings.lighting_defaults().get("sun_az_deg", -1.0)), 180.0),
			"azimuth rung 174 stored 180 deg")
		_ok(light.is_item_checked(light.get_item_index(174)) and not light.is_item_checked(light.get_item_index(170)),
			"and the azimuth check moved to rung 174 (the refresher finds a rung by id, not by index)")
		if had_light:
			DccSettings.set_lighting_default("sun_az_deg", az0)
		else:
			DccSettings.reset_lighting_defaults()

	print("== E. the Preferences row reads its tile size ==")
	var row_text := ""
	var stack: Array = [_app]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
			if c is PopupMenu:
				var pm := c as PopupMenu
				for i in pm.item_count:
					if pm.get_item_submenu(i) == "AtlasTileSize":
						## The readout is stamped when the Preferences popup opens, so open it.
						pm.about_to_popup.emit()
						row_text = pm.get_item_text(i)
	_ok(row_text.begins_with("Tile size · LOD levels") and row_text.ends_with(" px"),
		"the row reads '... · <n> px' (got %s)" % row_text)
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
