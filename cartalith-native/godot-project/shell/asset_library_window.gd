extends AcceptDialog
class_name AssetLibraryWindow

## §8's Asset library window (`DCC_SHELL_SPEC.md` §2.3/§8) -- Assets ▸
## ⧉ Asset library / ▦ Sprite sheet slicer's actual destination. Replaces
## v2.10's `#assetLibrary`.
##
## ## The one finding this file is built around: eight families, not 24
##
## §8 says "24 families... Settlements, Terrain, Cartography, plus
## Collections." That count does not match the shipped engine.
## `cartalith-assets/src/slots.rs` + `library.rs` (read directly this pass,
## not assumed from the spec) define **eight** families -- `textures`,
## `biomes`, `terrains`, `icons`, `settlement`, `trait`, `poi`, `custom` --
## and `ASSET_LIBRARY_SCOPE.md` §1 already recorded exactly this ("eight
## families, seven of them closed vocabularies") when Phase 4's engine side
## was built. §8's 24-family, four-group rail is the mockup's own richer
## subdivision (splitting e.g. "Feature icons" into "Trees & cover" /
## "Rock & scree") and was never ported to `cartalith-assets` -- there is no
## Rust type that draws that finer line. `FAMILIES` below is the real eight,
## with the real per-family slot count (`slots.size()`), grouped the way the
## crate itself groups them (`Family::is_texture()`, the `structures.*`
## trio), not the spec's fictional four.
##
## ## What is real vs. disclosed gap, control by control
##
## `GUI_GAP_REGISTER.md` rows AS-01..AS-08/AS-13/DM-05 (verified 2026-08-20):
## `cartalith-godot` now carries a real, live Asset Library authoring session
## (`asset_bridge::AssetLibrarySession`, a `WorldGen` field that survives a
## re-generate like `travel_library` does) behind an `as_*` `#[func]` surface
## -- import, per-slot fill state + real thumbnails, the inspector's file/
## scale/tags/pack-metadata queries, batch tag/collect/rename/duplicate/
## delete, Validate, Clear library, Export pack .zip, and Apply to map are
## all real engine calls now, not disclosed gaps. Concretely:
##
## - **Real**: everything the previous pass already had (family list, slot
##   ids/titles, anchor/bake-size/variant metadata, search/sort, the zoom
##   control, preview-background swatches, Import asset pack .zip…, the
##   sprite-sheet slicer's image load/grid-overlay arithmetic) **plus**:
##   Import image… into the focused slot (`as_import_item`/`as_add_custom_slot`);
##   per-slot fill state and a real baked thumbnail for every filled slot
##   (`as_family_slots`/`as_thumbnail_png`); the inspector's file/scale/tags
##   readout (`as_slot_summary`/`as_item_summary`) and pack metadata fields
##   (`as_pack_info`/`as_set_pack_info`); the four batch operations
##   (`as_batch_tag`/`as_batch_collect`/`as_batch_rename`/`as_batch_duplicate`/
##   `as_batch_delete`) -- batch Rename is honestly split: a custom slot is
##   renamed for real, a frozen slot instead renames its *item variants*
##   (frozen slot names are engine constants, `slot_title`, not editable --
##   a real spec/engine disagreement `GUI_GAP_REGISTER.md` AS-06 already
##   named); Validate (`as_validate`); Clear library… (`as_clear_library`);
##   Export pack .zip… (`as_export_pack_bytes`, bytes written to disk here
##   via `FileAccess`); Apply to map (`as_apply_to_map`, the reference's own
##   `applyToMap()` -- compiles the session and loads it straight into the
##   renderer, no round trip through a file).
## - **Disclosed gap, still honest**: the sprite-sheet slicer's actual slice
##   operation, trim/skip toggles, and assign-to-family/fill-from
##   (`cartalith-assets::raster` decodes/encodes whole images only -- checked
##   `raster.rs`/`manifest.rs`/`archive.rs` directly, no sheet-splitting
##   function exists anywhere in the crate; AS-09/AS-10/AS-11, a real engine
##   gap, not a binding gap); per-item scale/pan editing (the inspector shows
##   the real transform now but does not yet let it be dragged/typed -- no
##   `as_set_item_transform` exists; a smaller follow-on than this dispatch's
##   scope, left disabled with that reason).
##
## Every disabled control below carries its reason as a tooltip, the same
## `_todo()`-with-tooltip convention `menus.gd` uses at the menu level.

# ---------------------------------------------------------------------------
# The real family list (`cartalith-assets::slots`/`library`, verbatim ids)
# ---------------------------------------------------------------------------

const FAMILIES: Array[Dictionary] = [
	{"key": "textures", "code": "TX", "title": "Splat channels", "group": "Ground textures",
		"anchor": "none", "texture": true, "size": 512,
		"slots": ["grass", "rock", "sand", "snow", "wetland", "canopy", "parchment"]},
	{"key": "biomes", "code": "BI", "title": "Biome textures", "group": "Ground textures",
		"anchor": "none", "texture": true, "size": 512,
		"slots": ["coastal", "temperate_forest", "mediterranean", "wetlands", "steppe",
			"jungle", "boreal", "mountain", "cold_desert", "hot_desert", "tundra",
			"ruined", "hills", "lake_river", "ocean"]},
	{"key": "terrains", "code": "TR", "title": "Terrain textures", "group": "Ground textures",
		"anchor": "none", "texture": true, "size": 512,
		"slots": ["paved", "dirt", "hardpack", "plains", "forest_path", "hills", "rocky",
			"mtn_pass", "mtn_trail", "swamp", "deep_sand", "snow", "ruins"]},
	{"key": "icons", "code": "IC", "title": "Feature icons", "group": "Feature icons",
		"anchor": "bottom", "texture": false, "size": 256,
		"slots": ["mountain", "hill", "tree_conifer", "tree_broadleaf", "tree_rainforest",
			"tree_savanna", "tree_wetland", "shrub", "cactus", "boulder"]},
	{"key": "settlement", "code": "SC", "title": "Settlement pins", "group": "Structures",
		"anchor": "center", "texture": false, "size": 256,
		"slots": ["hamlet", "village", "town", "city", "capital", "monastery",
			"fortress", "university", "industrial"]},
	{"key": "trait", "code": "ST", "title": "Settlement traits", "group": "Structures",
		"anchor": "center", "texture": false, "size": 256,
		"slots": ["fortified", "mining", "port", "administrative", "trade_hub",
			"military", "religious"]},
	{"key": "poi", "code": "PI", "title": "Points of interest", "group": "Structures",
		## The Library's own 10-slot `poi` vocabulary (`lake`/`bridge`
		## included) -- not the narrower 8-slot `PACK_POI_SLOTS` a pack
		## *imports* against. `library.rs`'s own doc comment: those two "can
		## be authored... but never load" (no engine POI kind to attach to).
		"anchor": "center", "texture": false, "size": 256,
		"slots": ["ruin", "landmark", "mountain_peak", "lake", "named_forest",
			"battlefield", "shrine", "cave", "bridge", "other"]},
	{"key": "custom", "code": "CU", "title": "Custom icons", "group": "Custom",
		"anchor": "center", "texture": false, "size": 256, "custom": true, "slots": []},
]

const GROUP_ORDER: Array[String] = ["Ground textures", "Feature icons", "Structures", "Custom"]

const PREVIEW_SWATCHES: Array[Dictionary] = [
	{"label": "white", "mode": "color", "color": Color(1, 1, 1)},
	{"label": "checker", "mode": "checker", "color": Color(1, 1, 1)},
	{"label": "dark", "mode": "color", "color": Color("#101218")},
	{"label": "green", "mode": "color", "color": Color("#3bbf5a")},
	{"label": "blue", "mode": "color", "color": Color("#3b6fe2")},
]

# ---------------------------------------------------------------------------
# Small drawn controls -- checkerboard-empty slot cell, sheet-slicer preview
# ---------------------------------------------------------------------------

## A slot cell / the inspector's preview: checkerboard by default (honest
## "no art data" rather than a guessed empty/filled state), or a flat swatch
## when the inspector's preview-background picker is used. `_draw()` custom
## canvas, matching how `tool_overlay.gd`/`map_overlay.gd` already draw here.
class SlotCell extends Control:
	var uid := ""
	var selected := false
	var bg_mode := "checker"   ## "checker" | "color"
	var bg_color := Color(1, 1, 1)
	## A real baked thumbnail (`as_thumbnail_png`) once one has loaded --
	## `null` means "no art data queried/found yet", still the checkerboard.
	var thumb: ImageTexture

	func _ready() -> void:
		mouse_filter = Control.MOUSE_FILTER_STOP

	func _draw() -> void:
		var r := Rect2(Vector2.ZERO, size)
		if bg_mode == "checker":
			var a := DccTheme.c("sunken")
			var b := DccTheme.c("panel_alt")
			var cell := 8.0
			var yy := 0.0
			while yy < r.size.y:
				var xx := 0.0
				while xx < r.size.x:
					var idx := int(xx / cell) + int(yy / cell)
					draw_rect(Rect2(xx, yy, minf(cell, r.size.x - xx), minf(cell, r.size.y - yy)),
						a if idx % 2 == 0 else b, true)
					xx += cell
				yy += cell
		else:
			draw_rect(r, bg_color, true)
		if thumb != null:
			draw_texture_rect(thumb, r, false)
		draw_rect(r, DccTheme.c("line"), false, 1.0)
		if selected:
			draw_rect(r, DccTheme.c("accent"), false, 2.0)

## The slicer modal's sheet preview -- a real loaded `Image` plus a real
## columns/rows/margin/spacing grid overlay (arithmetic only; no engine call).
class SheetPreview extends Control:
	var img_tex: ImageTexture
	var cols := 6
	var rows := 4
	var margin := 0.0
	var spacing := 0.0

	func _draw() -> void:
		var r := Rect2(Vector2.ZERO, size)
		draw_rect(r, DccTheme.c("sunken"), true)
		if img_tex == null:
			return
		var tex_size := img_tex.get_size()
		if tex_size.x <= 0 or tex_size.y <= 0:
			return
		var scale: float = minf(r.size.x / tex_size.x, r.size.y / tex_size.y)
		var draw_size := tex_size * scale
		var origin := r.position + (r.size - draw_size) * 0.5
		draw_texture_rect(img_tex, Rect2(origin, draw_size), false)
		if cols <= 0 or rows <= 0:
			return
		var cw := (tex_size.x - margin * 2 - spacing * (cols - 1)) / float(cols)
		var ch := (tex_size.y - margin * 2 - spacing * (rows - 1)) / float(rows)
		if cw <= 0 or ch <= 0:
			return
		var line_color := DccTheme.c("accent")
		var xs: Array = []
		for i in cols:
			xs.append(margin + i * (cw + spacing))
		xs.append(margin + (cols - 1) * (cw + spacing) + cw)
		for x in xs:
			var sx: float = origin.x + float(x) * scale
			draw_line(Vector2(sx, origin.y), Vector2(sx, origin.y + draw_size.y), line_color, 1.0)
		var ys: Array = []
		for i in rows:
			ys.append(margin + i * (ch + spacing))
		ys.append(margin + (rows - 1) * (ch + spacing) + ch)
		for y in ys:
			var sy: float = origin.y + float(y) * scale
			draw_line(Vector2(origin.x, sy), Vector2(origin.x + draw_size.x, sy), line_color, 1.0)

# ---------------------------------------------------------------------------
# State
# ---------------------------------------------------------------------------

var _host: DccApp
var _bridge: EngineBridge

var _current_family := ""
var _search_text := ""
var _sort_mode := 0          ## 0 = slot order, 1 = name
var _cell_px := 84.0
var _select_mode := false
var _selected: Dictionary = {}     ## uid -> true
var _last_index := -1
var _focused_uid := ""
var _slot_order: Array = []        ## current family's filtered/sorted entries
var _cells: Dictionary = {}        ## uid -> SlotCell
var _preview_bg := "checker"
var _preview_color := Color(1, 1, 1)

var _status_label: Label
var _sort_button: OptionButton
var _select_mode_btn: Button
var _rail_buttons: Dictionary = {}
var _grid: GridContainer
var _grid_header: Label
var _select_count_label: Label
var _inspector_body: VBoxContainer

## Real per-slot state from the last `as_family_slots(family_key)` call --
## uid -> {"item_count","filled","has_dupe"} -- rebuilt on every `_refresh_grid()`.
var _slot_state: Dictionary = {}
var _apply_btn: Button
var _export_btn: Button
var _import_btn: Button
var _validate_btn: Button
var _clear_btn: Button
var _batch_buttons: Dictionary = {}   ## "tag"/"collect"/"rename"/"duplicate"/"delete" -> Button
var _pack_name_field: LineEdit
var _pack_author_field: LineEdit
var _pack_license_field: LineEdit

var _slicer: AcceptDialog
var _sheet_image: Image
var _sheet_preview: SheetPreview
var _sheet_readout: Label
var _slicer_cols: SpinBox
var _slicer_rows: SpinBox
var _slicer_margin: SpinBox
var _slicer_spacing: SpinBox
var _slicer_summary: Label
var _slice_btn: Button

# ---------------------------------------------------------------------------
# Setup
# ---------------------------------------------------------------------------

func setup(host: DccApp, bridge: EngineBridge) -> void:
	_host = host
	_bridge = bridge
	title = "⧉ ASSET LIBRARY"
	get_ok_button().hide()   ## the window bar's own Close button replaces it.
	size = Vector2i(1180, 760)
	min_size = Vector2i(960, 620)
	_bridge.world_loaded.connect(func(): _refresh_pack_status())
	_build()
	_build_slicer_modal()

## AS-13's `Assets ▸ Asset pack ▸` submenu (`menus.gd`'s `_assets()`) drives
## these four global, no-slot-context actions through the window's own real
## handlers rather than duplicating the dialog logic at the menu level.
func validate_now() -> void:
	_on_validate()

func apply_to_map_now() -> void:
	_on_apply_to_map()
	_refresh_pack_status()

func export_pack_now() -> void:
	_on_export_pack()

func clear_library_now() -> void:
	_on_clear_library()

func _family_by_key(key: String) -> Dictionary:
	for f in FAMILIES:
		if String(f["key"]) == key:
			return f
	return {}

## `family_key` scopes the family rail's selection (Assets ▸ Icon families ▸
## / Texture sets ▸ open the window this way); `open_slicer` opens the slicer
## modal on top, per §2.3's "opens the library window with the slicer modal
## already open."
func open(family_key: String = "", open_slicer: bool = false) -> void:
	popup_centered()
	_refresh_pack_status()
	if family_key != "" and not _family_by_key(family_key).is_empty():
		_select_family(family_key)
	elif _current_family == "":
		_select_family(String(FAMILIES[0]["key"]))
	if open_slicer:
		_open_slicer()

# ---------------------------------------------------------------------------
# Layout
# ---------------------------------------------------------------------------

func _build() -> void:
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 6)
	add_child(outer)

	outer.add_child(_build_window_bar())
	outer.add_child(DccTheme.rule())

	_status_label = DccTheme.label("", "text_ghost", DccTheme.FS_MICRO)
	outer.add_child(_status_label)
	outer.add_child(_build_pack_info_row())

	var main := HBoxContainer.new()
	main.add_theme_constant_override("separation", 0)
	main.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(main)

	main.add_child(_build_family_rail())
	main.add_child(DccTheme.rule(true))
	main.add_child(_build_slot_grid())
	main.add_child(DccTheme.rule(true))
	main.add_child(_build_inspector())

func _build_window_bar() -> Control:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)

	var search := LineEdit.new()
	search.placeholder_text = "search name · type · category · tag · file"
	search.custom_minimum_size.x = 260
	search.text_changed.connect(func(t: String): _search_text = t; _refresh_grid())
	row.add_child(search)

	_sort_button = OptionButton.new()
	_sort_button.add_item("Slot order")
	_sort_button.add_item("Name")
	_sort_button.item_selected.connect(func(i: int): _sort_mode = i; _refresh_grid())
	row.add_child(_sort_button)

	var slicer_btn := Button.new()
	slicer_btn.text = "▦ Sprite sheet…"
	slicer_btn.focus_mode = Control.FOCUS_NONE
	slicer_btn.pressed.connect(func(): _open_slicer())
	row.add_child(slicer_btn)

	_select_mode_btn = Button.new()
	_select_mode_btn.toggle_mode = true
	_select_mode_btn.focus_mode = Control.FOCUS_NONE
	_select_mode_btn.tooltip_text = "Batch selection driving Tag/Collect/Rename/Duplicate/Delete below."
	_select_mode_btn.toggled.connect(func(on: bool): _select_mode = on; _update_select_count())
	row.add_child(_select_mode_btn)

	row.add_child(DccTheme.spacer())

	_apply_btn = DccWidgets.action(row, "Apply to map", func(): _on_apply_to_map())
	_export_btn = DccWidgets.action(row, "Export pack .zip", func(): _on_export_pack())

	var close_btn := Button.new()
	close_btn.text = "Close"
	close_btn.focus_mode = Control.FOCUS_NONE
	close_btn.pressed.connect(func(): hide())
	row.add_child(close_btn)

	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 4)
	pad.add_theme_constant_override("margin_top", 4)
	pad.add_theme_constant_override("margin_right", 4)
	pad.add_child(row)
	return pad

func _build_family_rail() -> Control:
	var wrap := PanelContainer.new()
	wrap.custom_minimum_size.x = 260
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("panel_alt", {"right": 1}))
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	wrap.add_child(scroll)

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(col)

	var note_pad := MarginContainer.new()
	note_pad.add_theme_constant_override("margin_left", 14)
	note_pad.add_theme_constant_override("margin_top", 8)
	note_pad.add_theme_constant_override("margin_right", 10)
	var note := DccWidgets.note(note_pad,
		"8 families, frozen against the reference engine (cartalith-assets::slots/library) -- not this spec's own 24; see ASSET_LIBRARY_SCOPE.md §1. Capacity and fill counts below are both real (AssetDB::slots_in_family/filled state).")
	note.custom_minimum_size.x = 220
	col.add_child(note_pad)

	var by_group: Dictionary = {}
	for fam in FAMILIES:
		var g := String(fam["group"])
		if not by_group.has(g):
			by_group[g] = []
		(by_group[g] as Array).append(fam)

	for g in GROUP_ORDER:
		var body := DccWidgets.section(col, g)
		for f in by_group.get(g, []):
			var fam: Dictionary = f
			var btn := Button.new()
			btn.flat = true
			btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
			btn.focus_mode = Control.FOCUS_NONE
			btn.custom_minimum_size.y = 26
			btn.add_theme_font_override("font", DccTheme.mono(0))
			btn.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
			btn.add_theme_color_override("font_color", DccTheme.c("text"))
			btn.pressed.connect(_select_family.bind(String(fam["key"])))
			_rail_buttons[String(fam["key"])] = btn
			body.add_child(btn)
	_refresh_rail_counts()

	col.add_child(DccTheme.rule())
	var foot_pad := MarginContainer.new()
	foot_pad.add_theme_constant_override("margin_left", 14)
	foot_pad.add_theme_constant_override("margin_top", 8)
	foot_pad.add_theme_constant_override("margin_bottom", 10)
	foot_pad.add_theme_constant_override("margin_right", 10)
	var foot := VBoxContainer.new()
	foot.add_theme_constant_override("separation", 4)
	foot_pad.add_child(foot)
	col.add_child(foot_pad)

	_import_btn = DccWidgets.action(foot, "Import image…", func(): _on_import_image())
	_refresh_import_button()
	var import_pack_btn := Button.new()
	import_pack_btn.text = "Import pack…"
	import_pack_btn.focus_mode = Control.FOCUS_NONE
	import_pack_btn.pressed.connect(func(): _host.open_asset_pack_picker())
	foot.add_child(import_pack_btn)

	return wrap

## Real fill counts (AS-08) on the rail itself, not just the grid --
## `as_family_slots` once per family, cheap enough to run on every
## `_refresh_pack_status()` (window open, and after `world_loaded`).
func _refresh_rail_counts() -> void:
	for fam in FAMILIES:
		var key := String(fam["key"])
		var btn: Button = _rail_buttons.get(key)
		if btn == null:
			continue
		var slots: Array = _bridge.as_family_slots(key)
		var filled := 0
		for s in slots:
			if bool(s.get("filled", false)):
				filled += 1
		var cap_text: String
		if bool(fam.get("custom", false)):
			cap_text = "%d items" % slots.size() if not slots.is_empty() else "open vocabulary"
		else:
			cap_text = "%d / %d" % [filled, (fam["slots"] as Array).size()]
		btn.text = "%s   %s   %s" % [String(fam["code"]), String(fam["title"]), cap_text]

## "Import image…" targets whichever slot is focused in the grid -- real once
## a slot is selected, honestly disabled ("select a slot first") otherwise.
func _refresh_import_button() -> void:
	if _import_btn == null:
		return
	if _focused_uid == "":
		_import_btn.disabled = true
		_import_btn.tooltip_text = "Select a slot in the grid first -- Import image… lands the file on the focused slot."
	else:
		_import_btn.disabled = false
		_import_btn.tooltip_text = "Import a PNG into %s." % _focused_uid

func _on_import_image() -> void:
	if _focused_uid == "":
		return
	var target_uid := _focused_uid
	var d := FileDialog.new()
	d.title = "Import image"
	d.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	d.access = FileDialog.ACCESS_FILESYSTEM
	d.add_filter("*.png ; PNG image")
	d.file_selected.connect(func(path: String):
		var bytes := FileAccess.get_file_as_bytes(path)
		var result: Dictionary = _bridge.as_import_item(target_uid, path.get_file(), bytes)
		if bool(result.get("ok", false)):
			_host.set_status("hint", "imported %s" % path.get_file(), "accent")
			_refresh_grid()
			_refresh_inspector()
			_refresh_rail_counts()
		else:
			_host.set_status("hint", "import failed — %s" % String(result.get("error", "unknown error")), "warn")
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered_ratio(0.6)

func _build_slot_grid() -> Control:
	var wrap := VBoxContainer.new()
	wrap.add_theme_constant_override("separation", 6)
	wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 12)
	head_pad.add_theme_constant_override("margin_top", 10)
	head_pad.add_theme_constant_override("margin_right", 12)
	var head_row := HBoxContainer.new()
	_grid_header = DccTheme.mono_label("", "accent", DccTheme.FS_HEADER, 2, true)
	head_row.add_child(_grid_header)
	head_row.add_child(DccTheme.spacer())
	_select_count_label = DccTheme.mono_label("0 selected", "text_dim", DccTheme.FS_SMALL)
	head_row.add_child(_select_count_label)
	head_pad.add_child(head_row)
	wrap.add_child(head_pad)

	var batch_pad := MarginContainer.new()
	batch_pad.add_theme_constant_override("margin_left", 12)
	batch_pad.add_theme_constant_override("margin_right", 12)
	var batch_row := HBoxContainer.new()
	batch_row.add_theme_constant_override("separation", 4)
	batch_pad.add_child(batch_row)
	_batch_buttons["tag"] = DccWidgets.action(batch_row, "Tag…", func(): _on_batch_tag())
	_batch_buttons["collect"] = DccWidgets.action(batch_row, "Collect…", func(): _on_batch_collect())
	_batch_buttons["rename"] = DccWidgets.action(batch_row, "Rename…", func(): _on_batch_rename())
	_batch_buttons["duplicate"] = DccWidgets.action(batch_row, "Duplicate", func(): _on_batch_duplicate())
	_batch_buttons["delete"] = DccWidgets.action(batch_row, "Delete", func(): _on_batch_delete())
	_refresh_batch_buttons()
	wrap.add_child(batch_pad)
	wrap.add_child(DccTheme.rule())

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	wrap.add_child(scroll)
	var grid_pad := MarginContainer.new()
	grid_pad.add_theme_constant_override("margin_left", 12)
	grid_pad.add_theme_constant_override("margin_top", 8)
	grid_pad.add_theme_constant_override("margin_right", 12)
	grid_pad.add_theme_constant_override("margin_bottom", 8)
	scroll.add_child(grid_pad)
	_grid = GridContainer.new()
	_grid.columns = 6
	_grid.add_theme_constant_override("h_separation", 10)
	_grid.add_theme_constant_override("v_separation", 10)
	grid_pad.add_child(_grid)

	wrap.add_child(DccTheme.rule())
	var foot_pad := MarginContainer.new()
	foot_pad.add_theme_constant_override("margin_left", 12)
	foot_pad.add_theme_constant_override("margin_top", 4)
	foot_pad.add_theme_constant_override("margin_right", 12)
	foot_pad.add_theme_constant_override("margin_bottom", 8)
	var foot_row := HBoxContainer.new()
	foot_row.add_theme_constant_override("separation", 6)
	foot_pad.add_child(foot_row)
	var hint := DccTheme.label("⇧-click ranges · ⌘/Ctrl-click adds · drag-to-fill has no engine call", "text_ghost", DccTheme.FS_MICRO)
	foot_row.add_child(hint)
	foot_row.add_child(DccTheme.spacer())
	foot_row.add_child(DccTheme.mono_label("ZOOM", "text_faint", DccTheme.FS_MICRO, 1))
	var zoom := HSlider.new()
	zoom.min_value = 56
	zoom.max_value = 132
	zoom.step = 4
	zoom.value = _cell_px
	zoom.custom_minimum_size.x = 100
	zoom.focus_mode = Control.FOCUS_NONE
	zoom.value_changed.connect(func(v: float): _cell_px = v; _refresh_grid())
	foot_row.add_child(zoom)
	wrap.add_child(foot_pad)

	return wrap

func _build_inspector() -> Control:
	var wrap := VBoxContainer.new()
	wrap.add_theme_constant_override("separation", 0)
	wrap.size_flags_vertical = Control.SIZE_EXPAND_FILL

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 12)
	pad.add_theme_constant_override("margin_top", 10)
	pad.add_theme_constant_override("margin_right", 12)
	pad.add_theme_constant_override("margin_bottom", 10)
	scroll.add_child(pad)
	_inspector_body = VBoxContainer.new()
	_inspector_body.add_theme_constant_override("separation", 6)
	pad.add_child(_inspector_body)
	wrap.add_child(scroll)

	wrap.add_child(DccTheme.rule())
	var foot_pad := MarginContainer.new()
	foot_pad.add_theme_constant_override("margin_left", 12)
	foot_pad.add_theme_constant_override("margin_top", 8)
	foot_pad.add_theme_constant_override("margin_right", 12)
	foot_pad.add_theme_constant_override("margin_bottom", 10)
	var foot := HBoxContainer.new()
	foot.add_theme_constant_override("separation", 6)
	foot_pad.add_child(foot)
	_validate_btn = DccWidgets.action(foot, "Validate", func(): _on_validate())
	_clear_btn = DccWidgets.action(foot, "Clear library…", func(): _on_clear_library())
	wrap.add_child(foot_pad)

	var outer := PanelContainer.new()
	outer.custom_minimum_size.x = 300
	outer.add_theme_stylebox_override("panel", DccTheme.panel("panel_alt", {"left": 1}))
	outer.add_child(wrap)
	_refresh_inspector()
	return outer

# ---------------------------------------------------------------------------
# Family / grid / selection
# ---------------------------------------------------------------------------

func _select_family(key: String) -> void:
	_current_family = key
	for k in _rail_buttons:
		var b: Button = _rail_buttons[k]
		b.add_theme_color_override("font_color", DccTheme.c("accent") if k == key else DccTheme.c("text"))
	_selected.clear()
	_last_index = -1
	_focused_uid = ""
	_refresh_grid()
	_refresh_inspector()
	_refresh_import_button()

func _humanize(id: String) -> String:
	var parts := id.split("_")
	var out: Array[String] = []
	for p in parts:
		if p == "":
			continue
		out.append(p.substr(0, 1).to_upper() + p.substr(1))
	return " ".join(out)

func _refresh_grid() -> void:
	for c in _grid.get_children():
		_grid.remove_child(c)
		c.queue_free()
	_cells.clear()

	var fam := _family_by_key(_current_family)
	if fam.is_empty():
		_grid_header.text = ""
		return

	# AS-08: real per-slot fill state from the live session, keyed by uid.
	_slot_state.clear()
	var server_slots: Array = _bridge.as_family_slots(_current_family)
	for s in server_slots:
		_slot_state[String(s["uid"])] = s

	var q := _search_text.to_lower()
	var entries: Array = []
	var is_custom := bool(fam.get("custom", false))
	if is_custom:
		# The custom family has no frozen id list (`fam["slots"]` is empty by
		# design -- see this file's own header note) -- its entries are
		# whatever custom slots the live session actually has.
		for i in server_slots.size():
			var s: Dictionary = server_slots[i]
			var name := String(s["name"])
			var code := "%s-%02d" % [String(fam["code"]), i + 1]
			var uid := String(s["uid"])
			if q != "" and name.to_lower().find(q) < 0 and code.to_lower().find(q) < 0:
				continue
			entries.append({"uid": uid, "id": String(s["id"]), "name": name, "code": code})
	else:
		var ids: Array = fam["slots"]
		for i in ids.size():
			var id := String(ids[i])
			var name := _humanize(id)
			var code := "%s-%02d" % [String(fam["code"]), i + 1]
			var uid := "%s:%s" % [String(fam["key"]), id]
			if q != "" and id.to_lower().find(q) < 0 and name.to_lower().find(q) < 0 and code.to_lower().find(q) < 0:
				continue
			entries.append({"uid": uid, "id": id, "name": name, "code": code})
	if _sort_mode == 1:
		entries.sort_custom(func(a, b): return String(a["name"]) < String(b["name"]))
	_slot_order = entries

	for entry in entries:
		_grid.add_child(_build_cell(entry))

	var shown := entries.size()
	var total: int = server_slots.size() if is_custom else (fam["slots"] as Array).size()
	var filled := 0
	for s in server_slots:
		if bool(s.get("filled", false)):
			filled += 1
	_grid_header.text = "%s · %s · %d OF %d SHOWN · %d FILLED" % [
		String(fam["code"]), String(fam["title"]).to_upper(), shown, total, filled]
	_refresh_selection_visuals()

func _build_cell(entry: Dictionary) -> Control:
	var wrap := VBoxContainer.new()
	wrap.add_theme_constant_override("separation", 2)
	var cell := SlotCell.new()
	cell.custom_minimum_size = Vector2(_cell_px, _cell_px)
	var uid := String(entry["uid"])
	cell.uid = uid
	var state: Dictionary = _slot_state.get(uid, {})
	if bool(state.get("filled", false)):
		var png: PackedByteArray = _bridge.as_thumbnail_png(uid, 0, 128)
		if png.size() > 0:
			var img := Image.new()
			if img.load_png_from_buffer(png) == OK:
				cell.thumb = ImageTexture.create_from_image(img)
	cell.gui_input.connect(_on_cell_input.bind(uid))
	wrap.add_child(cell)
	var count := int(state.get("item_count", 0))
	var suffix := " ×%d" % count if count > 1 else ""
	var dupe_mark := " ⚠" if bool(state.get("has_dupe", false)) else ""
	var lbl := DccTheme.mono_label("%s %s%s%s" % [String(entry["code"]), String(entry["name"]), suffix, dupe_mark], "text_dim", DccTheme.FS_TINY)
	lbl.clip_text = true
	lbl.custom_minimum_size.x = _cell_px
	wrap.add_child(lbl)
	_cells[uid] = cell
	return wrap

func _on_cell_input(ev: InputEvent, uid: String) -> void:
	if not (ev is InputEventMouseButton and ev.pressed and ev.button_index == MOUSE_BUTTON_LEFT):
		return
	var index := -1
	for i in _slot_order.size():
		if String(_slot_order[i]["uid"]) == uid:
			index = i
			break
	if ev.shift_pressed and _last_index >= 0 and index >= 0:
		var lo: int = mini(_last_index, index)
		var hi: int = maxi(_last_index, index)
		_selected.clear()
		for i in range(lo, hi + 1):
			_selected[String(_slot_order[i]["uid"])] = true
	elif ev.ctrl_pressed or ev.meta_pressed:
		if _selected.has(uid):
			_selected.erase(uid)
		else:
			_selected[uid] = true
		_last_index = index
	else:
		_selected = {uid: true}
		_last_index = index
	_focused_uid = uid
	_refresh_selection_visuals()
	_refresh_inspector()
	_refresh_import_button()

func _refresh_selection_visuals() -> void:
	for uid in _cells:
		var cell: SlotCell = _cells[uid]
		cell.selected = _selected.has(uid)
		cell.queue_redraw()
	_update_select_count()

func _update_select_count() -> void:
	_select_count_label.text = "%d selected" % _selected.size()
	if _select_mode_btn:
		_select_mode_btn.text = "%s Select (%d)" % [
			DccIcons.SYMBOLS["checked"] if _select_mode else DccIcons.SYMBOLS["unchecked"], _selected.size()]
	_refresh_batch_buttons()

## The five batch actions all need at least one selected uid; disabled
## (with a real reason) otherwise rather than silently doing nothing.
func _refresh_batch_buttons() -> void:
	var has_sel := not _selected.is_empty()
	for key in _batch_buttons:
		var b: Button = _batch_buttons[key]
		b.disabled = not has_sel
		b.tooltip_text = "" if has_sel else "Select at least one slot in the grid first."

func _selected_uids() -> PackedStringArray:
	var out := PackedStringArray()
	for uid in _selected.keys():
		out.append(String(uid))
	return out

## A minimal single-line text-input modal -- the reference's own `prompt()`
## has no Godot equivalent, so this is the reusable stand-in every batch
## handler below shares.
func _prompt_text(title: String, label_text: String, default_text: String, on_confirm: Callable) -> void:
	var d := ConfirmationDialog.new()
	d.title = title
	d.min_size = Vector2i(360, 0)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 6)
	body.add_child(DccTheme.label(label_text, "text_dim", DccTheme.FS_SMALL))
	var le := LineEdit.new()
	le.text = default_text
	le.select_all_on_focus = true
	body.add_child(le)
	d.add_child(body)
	d.confirmed.connect(func():
		var t := le.text.strip_edges()
		if t != "":
			on_confirm.call(t)
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered()
	le.grab_focus.call_deferred()

func _on_batch_tag() -> void:
	var uids := _selected_uids()
	if uids.is_empty():
		return
	_prompt_text("Tag %d asset(s)" % uids.size(), "Add tag(s) -- comma-separated:", "", func(t: String):
		var result: Dictionary = _bridge.as_batch_tag(uids, t)
		_host.set_status("hint", "tagged %d asset(s)" % int(result.get("tagged", 0)), "accent")
		_refresh_inspector())

func _on_batch_collect() -> void:
	var uids := _selected_uids()
	if uids.is_empty():
		return
	_prompt_text("Collect %d asset(s)" % uids.size(), "Add to collection:", "Fantasy Pack", func(t: String):
		_bridge.as_batch_collect(uids, t)
		_host.set_status("hint", "added %d asset(s) to \"%s\"" % [uids.size(), t], "accent")
		_refresh_inspector())

func _on_batch_rename() -> void:
	var uids := _selected_uids()
	if uids.is_empty():
		return
	_prompt_text("Rename %d asset(s)" % uids.size(),
			"Rename pattern -- selected assets become \"Base_01\", \"Base_02\", …\n(custom slots are renamed; frozen slots rename their variants)", "Village",
			func(t: String):
		var result: Dictionary = _bridge.as_batch_rename(uids, t)
		_host.set_status("hint", "renamed %d asset(s)" % int(result.get("renamed", 0)), "accent")
		var remap: Dictionary = result.get("remap", {})
		if remap.has(_focused_uid):
			_focused_uid = String(remap[_focused_uid])
		_selected.clear()
		for old_uid in uids:
			var s := String(old_uid)
			_selected[String(remap.get(s, s))] = true
		_refresh_grid()
		_refresh_inspector())

func _on_batch_duplicate() -> void:
	var uids := _selected_uids()
	if uids.is_empty():
		return
	var result: Dictionary = _bridge.as_batch_duplicate(uids)
	var made := int(result.get("made", 0))
	_host.set_status("hint",
		("duplicated %d asset(s) → Custom/Duplicates" % made) if made > 0 else "nothing to duplicate", "accent")
	_refresh_grid()
	_refresh_rail_counts()

func _on_batch_delete() -> void:
	var uids := _selected_uids()
	if uids.is_empty():
		return
	var d := ConfirmationDialog.new()
	d.title = "Delete %d asset(s)?" % uids.size()
	d.dialog_text = "Delete images of %d selected asset(s)? (custom slots are removed entirely; frozen slots are emptied, not removed.)" % uids.size()
	d.confirmed.connect(func():
		var result: Dictionary = _bridge.as_batch_delete(uids)
		_host.set_status("hint", "deleted %d asset(s)" % int(result.get("deleted", 0)), "accent")
		_selected.clear()
		_refresh_grid()
		_refresh_inspector()
		_refresh_rail_counts()
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered()

# ---------------------------------------------------------------------------
# Slot inspector
# ---------------------------------------------------------------------------

func _refresh_inspector() -> void:
	for c in _inspector_body.get_children():
		_inspector_body.remove_child(c)
		c.queue_free()

	if _focused_uid == "":
		DccWidgets.note(_inspector_body, "Select a slot to inspect it.")
		return

	# AS-07: real slot/item/pack queries -- `as_slot_summary` is the source
	# of truth (it also confirms the slot still exists, e.g. after a batch
	# delete/rename elsewhere).
	var summary: Dictionary = _bridge.as_slot_summary(_focused_uid)
	if not bool(summary.get("ok", false)):
		DccWidgets.note(_inspector_body, "This slot no longer exists in the live session (removed by a batch edit).")
		return

	var fam_key := String(summary.get("family", ""))
	var fam := _family_by_key(fam_key)
	if fam.is_empty():
		return

	var entry: Dictionary = {}
	for e in _slot_order:
		if String(e["uid"]) == _focused_uid:
			entry = e
			break
	var code := String(entry.get("code", "—"))
	var name := String(summary.get("name", ""))
	var item_count := int(summary.get("item_count", 0))

	_inspector_body.add_child(DccTheme.mono_label("%s  %s" % [code, name], "accent", DccTheme.FS_HEADER, 2, true))
	_inspector_body.add_child(DccTheme.label(String(fam["title"]), "text_dim", DccTheme.FS_SMALL))
	_inspector_body.add_child(DccTheme.rule())

	var preview := SlotCell.new()
	preview.custom_minimum_size = Vector2(0, 160)
	preview.bg_mode = _preview_bg
	preview.bg_color = _preview_color
	if item_count > 0:
		var preview_png: PackedByteArray = _bridge.as_thumbnail_png(_focused_uid, 0, 256)
		if preview_png.size() > 0:
			var pimg := Image.new()
			if pimg.load_png_from_buffer(preview_png) == OK:
				preview.thumb = ImageTexture.create_from_image(pimg)
	_inspector_body.add_child(preview)

	var sw_row := HBoxContainer.new()
	sw_row.add_theme_constant_override("separation", 4)
	_inspector_body.add_child(sw_row)
	for sw in PREVIEW_SWATCHES:
		var swatch: Dictionary = sw
		var b := ColorRect.new()
		b.color = swatch["color"] if swatch["mode"] == "color" else DccTheme.c("sunken")
		b.custom_minimum_size = Vector2(18, 18)
		b.tooltip_text = String(swatch["label"])
		b.mouse_filter = Control.MOUSE_FILTER_STOP
		b.gui_input.connect(func(ev: InputEvent):
			if ev is InputEventMouseButton and ev.pressed and ev.button_index == MOUSE_BUTTON_LEFT:
				_preview_bg = String(swatch["mode"])
				_preview_color = swatch["color"]
				preview.bg_mode = _preview_bg
				preview.bg_color = _preview_color
				preview.queue_redraw())
		sw_row.add_child(b)

	if item_count == 0:
		DccWidgets.note(_inspector_body, "No art stored in this slot yet -- focus it and use Import image… (family rail foot) to add one.")
	elif item_count > 1:
		DccWidgets.note(_inspector_body, "%d variants stored; the preview and File/Scale rows below show variant 1." % item_count)

	_inspector_body.add_child(DccTheme.rule())
	var anchor_labels := {"none": "tiled, not anchored", "bottom": "bottom-anchored (base on the cell)", "center": "centre-anchored"}
	var anchor_label: String = anchor_labels.get(String(fam["anchor"]), "?")
	_kv_row(_inspector_body, "Anchor", "%s (fixed by family, not a per-slot setting)" % anchor_label)
	_kv_row(_inspector_body, "Bake size", "%dpx %s" % [int(fam["size"]), "opaque, seamless tile" if bool(fam["texture"]) else "RGBA, straight alpha"])
	_kv_row(_inspector_body, "Variants", "%d stored" % item_count)

	_inspector_body.add_child(DccTheme.rule())
	if item_count > 0:
		var item: Dictionary = _bridge.as_item_summary(_focused_uid, 0)
		if bool(item.get("ok", false)):
			_kv_row(_inspector_body, "File", "%s · %d×%d px" % [String(item.get("name", "")), int(item.get("w", 0)), int(item.get("h", 0))])
			_kv_row(_inspector_body, "Scale",
				"×%.2f · pan (%.0f, %.0f)" % [float(item.get("scale", 1.0)), float(item.get("pan_x", 0.0)), float(item.get("pan_y", 0.0))])
		else:
			_kv_row(_inspector_body, "File", "—")
			_kv_row(_inspector_body, "Scale", "—")
	else:
		_gap_kv_row(_inspector_body, "File", "This slot has no items yet.")
		_gap_kv_row(_inspector_body, "Scale", "This slot has no items yet.")
	_gap_kv_row(_inspector_body, "Edit scale/pan", "as_set_item_transform not yet exposed -- reading the transform is real, dragging/typing a new one is a smaller follow-on.")

	var tags: PackedStringArray = summary.get("tags", PackedStringArray())
	_kv_row(_inspector_body, "Tags", ", ".join(tags) if tags.size() > 0 else "none -- Batch ▸ Tag… adds one")

	var pack_info: Dictionary = _bridge.as_pack_info()
	var pn := String(pack_info.get("name", ""))
	var pa := String(pack_info.get("author", ""))
	var pl := String(pack_info.get("license", ""))
	_kv_row(_inspector_body, "Pack metadata", "%s · %s · %s" % [
		pn if pn != "" else "(unnamed)", pa if pa != "" else "(no author)", pl if pl != "" else "(no license)"])

func _kv_row(parent: Control, label_text: String, value: String) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	var l := DccTheme.mono_label(label_text, "text_dim", DccTheme.FS_TINY)
	l.custom_minimum_size.x = 90
	row.add_child(l)
	var v := DccTheme.label(value, "text", DccTheme.FS_SMALL)
	v.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(v)
	parent.add_child(row)

func _gap_kv_row(parent: Control, label_text: String, reason: String) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.tooltip_text = reason
	var l := DccTheme.mono_label(label_text, "text_ghost", DccTheme.FS_TINY)
	l.custom_minimum_size.x = 90
	row.add_child(l)
	row.add_child(DccTheme.label("—", "text_ghost", DccTheme.FS_SMALL))
	parent.add_child(row)

## `AssetValidator.run()` -- the real, ordered warning list, shown in a
## simple modal (the reference's own `alert`-style summary).
func _on_validate() -> void:
	var warnings: PackedStringArray = _bridge.as_validate()
	var d := AcceptDialog.new()
	d.title = "Validation"
	d.min_size = Vector2i(420, 0)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 4)
	if warnings.is_empty():
		body.add_child(DccTheme.label("✓ No issues found.", "accent", DccTheme.FS_SMALL))
	else:
		for w in warnings:
			var l := DccTheme.label("⚠ %s" % String(w), "warn", DccTheme.FS_SMALL)
			l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
			body.add_child(l)
	d.add_child(body)
	d.confirmed.connect(func(): d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered()

func _on_clear_library() -> void:
	var d := ConfirmationDialog.new()
	d.title = "Clear the asset library?"
	d.dialog_text = "Clear the entire asset library? This removes every imported item and custom slot."
	d.confirmed.connect(func():
		_bridge.as_clear_library()
		_selected.clear()
		_focused_uid = ""
		_refresh_grid()
		_refresh_inspector()
		_refresh_import_button()
		_refresh_pack_status()
		_host.set_status("hint", "asset library cleared", "accent")
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered()

# ---------------------------------------------------------------------------
# Pack status / pack metadata (AS-13's "Active pack" header)
# ---------------------------------------------------------------------------

func _refresh_pack_status() -> void:
	if _status_label == null:
		return
	var info: Dictionary = _bridge.as_pack_info()
	var total := int(info.get("total_items", 0))
	_status_label.text = "%s in the editing session · %s for rendering" % [
		("%d item%s" % [total, "" if total == 1 else "s"]) if total > 0 else "empty library",
		"a pack is loaded" if _bridge.has_asset_pack() else "no pack loaded (Import pack… loads one)"]
	_refresh_pack_info_fields(info)
	_refresh_rail_counts()

func _build_pack_info_row() -> Control:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 4)
	pad.add_theme_constant_override("margin_bottom", 4)
	pad.add_child(row)

	var mk := func(label_text: String, w: float) -> LineEdit:
		var l := DccTheme.mono_label(label_text, "text_faint", DccTheme.FS_MICRO, 1)
		row.add_child(l)
		var le := LineEdit.new()
		le.custom_minimum_size.x = w
		row.add_child(le)
		return le
	_pack_name_field = mk.call("NAME", 160)
	_pack_author_field = mk.call("AUTHOR", 120)
	_pack_license_field = mk.call("LICENSE", 80)
	for le in [_pack_name_field, _pack_author_field, _pack_license_field]:
		var field: LineEdit = le
		field.text_submitted.connect(func(_t: String): _commit_pack_info())
		field.focus_exited.connect(func(): _commit_pack_info())
	return pad

func _refresh_pack_info_fields(info: Dictionary) -> void:
	if _pack_name_field == null:
		return
	# Don't clobber text the user is actively editing.
	if not _pack_name_field.has_focus():
		_pack_name_field.text = String(info.get("name", ""))
	if not _pack_author_field.has_focus():
		_pack_author_field.text = String(info.get("author", ""))
	if not _pack_license_field.has_focus():
		_pack_license_field.text = String(info.get("license", ""))

func _commit_pack_info() -> void:
	_bridge.as_set_pack_info(_pack_name_field.text, _pack_author_field.text, _pack_license_field.text)

func _on_apply_to_map() -> void:
	var result: Dictionary = _bridge.as_apply_to_map()
	if bool(result.get("ok", false)):
		_host.set_status("hint", "asset pack applied to the map", "accent")
	else:
		_host.set_status("hint", "apply failed — %s" % String(result.get("error", "unknown error")), "warn")
	_refresh_pack_status()

func _on_export_pack() -> void:
	var result: Dictionary = _bridge.as_export_pack_bytes()
	if not bool(result.get("ok", false)):
		_host.set_status("hint", "export failed — %s" % String(result.get("error", "unknown error")), "warn")
		return
	var bytes: PackedByteArray = result.get("bytes", PackedByteArray())
	var suggested := "%s.zip" % _slug_name(String(result.get("name", "asset_pack")))
	var d := FileDialog.new()
	d.title = "Export pack .zip"
	d.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	d.access = FileDialog.ACCESS_FILESYSTEM
	d.add_filter("*.zip ; Asset pack")
	d.current_file = suggested
	d.file_selected.connect(func(path: String):
		var f := FileAccess.open(path, FileAccess.WRITE)
		if f == null:
			_host.set_status("hint", "export failed — could not open %s for writing" % path.get_file(), "warn")
		else:
			f.store_buffer(bytes)
			f.close()
			_host.set_status("hint", "exported %s (%d bytes)" % [path.get_file(), bytes.size()], "accent")
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered_ratio(0.6)

## The reference's own `slugName` (`Cartalith Gen1 v2.10.html` line ~27011):
## lowercase, collapse every non-alphanumeric run to `_`, trim the ends, fall
## back to `"asset_pack"`.
func _slug_name(s: String) -> String:
	var out := s.to_lower()
	var re := RegEx.new()
	re.compile("[^a-z0-9]+")
	out = re.sub(out, "_", true)
	out = out.strip_edges().lstrip("_").rstrip("_")
	return out if out != "" else "asset_pack"

# ---------------------------------------------------------------------------
# Sprite-sheet slicer modal
# ---------------------------------------------------------------------------

func _build_slicer_modal() -> void:
	_slicer = AcceptDialog.new()
	_slicer.title = "▦ SPRITE SHEET SLICER"
	_slicer.get_ok_button().hide()
	_slicer.size = Vector2i(780, 660)
	add_child(_slicer)

	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 8)
	var pad := MarginContainer.new()
	for m in ["margin_left", "margin_top", "margin_right", "margin_bottom"]:
		pad.add_theme_constant_override(m, 12)
	pad.add_child(body)
	_slicer.add_child(pad)

	var note := DccTheme.label(
		"Preview and the grid overlay below are real -- Godot loads the image and computes the grid math. The slice operation itself is unbacked: cartalith-assets decodes/encodes whole PNGs (raster.rs) with no sheet-splitting function, so nothing below can be applied to a slot.",
		"text_ghost", DccTheme.FS_MICRO)
	note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	body.add_child(note)

	var choose_btn := Button.new()
	choose_btn.text = "Choose image…"
	choose_btn.focus_mode = Control.FOCUS_NONE
	choose_btn.pressed.connect(_pick_sheet_image)
	body.add_child(choose_btn)

	_sheet_readout = DccTheme.mono_label("no sheet chosen", "text_dim", DccTheme.FS_SMALL)
	body.add_child(_sheet_readout)

	_sheet_preview = SheetPreview.new()
	_sheet_preview.custom_minimum_size = Vector2(0, 320)
	_sheet_preview.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(_sheet_preview)

	var params := HBoxContainer.new()
	params.add_theme_constant_override("separation", 10)
	body.add_child(params)
	_slicer_cols = DccWidgets.number(params, "Columns", 1, 64, 1, 6,
		func(v: float): _sheet_preview.cols = int(v); _sheet_preview.queue_redraw(); _refresh_slicer_summary())
	_slicer_rows = DccWidgets.number(params, "Rows", 1, 64, 1, 4,
		func(v: float): _sheet_preview.rows = int(v); _sheet_preview.queue_redraw(); _refresh_slicer_summary())
	_slicer_margin = DccWidgets.number(params, "Margin px", 0, 128, 1, 0,
		func(v: float): _sheet_preview.margin = v; _sheet_preview.queue_redraw(); _refresh_slicer_summary())
	_slicer_spacing = DccWidgets.number(params, "Spacing px", 0, 64, 1, 0,
		func(v: float): _sheet_preview.spacing = v; _sheet_preview.queue_redraw(); _refresh_slicer_summary())

	var toggles := HBoxContainer.new()
	toggles.add_theme_constant_override("separation", 12)
	body.add_child(toggles)
	var trim := DccWidgets.toggle(toggles, "Trim transparent edges", false, func(_v): pass,
		"No slice operation exists to apply this to.")
	trim.disabled = true
	var skip := DccWidgets.toggle(toggles, "Skip empty cells", false, func(_v): pass,
		"No slice operation exists to apply this to.")
	skip.disabled = true

	var assign_row := HBoxContainer.new()
	assign_row.add_theme_constant_override("separation", 12)
	body.add_child(assign_row)
	var fam_names: Array = []
	for f in FAMILIES:
		fam_names.append(String(f["title"]))
	var assign_ob := DccWidgets.choice(assign_row, "Assign to family", fam_names, 0, func(_i): pass,
		"No slot target: there is no in-memory library session for a slice result to land in.")
	assign_ob.disabled = true
	var fill_ob := DccWidgets.choice(assign_row, "Fill from", ["first empty", "overwrite"], 0, func(_i): pass,
		"Same -- unreachable without a slice operation.")
	fill_ob.disabled = true

	_slicer_summary = DccTheme.mono_label("", "text_dim", DccTheme.FS_SMALL)
	body.add_child(_slicer_summary)

	body.add_child(DccTheme.rule())
	var foot := HBoxContainer.new()
	foot.add_theme_constant_override("separation", 8)
	body.add_child(foot)
	foot.add_child(DccTheme.spacer())
	var cancel_btn := Button.new()
	cancel_btn.text = "Cancel"
	cancel_btn.focus_mode = Control.FOCUS_NONE
	cancel_btn.pressed.connect(func(): _slicer.hide())
	foot.add_child(cancel_btn)
	_slice_btn = _gap_button(foot, "Slice",
		"cartalith-assets has no sheet-slicing function -- checked raster.rs, manifest.rs and archive.rs directly; only whole-image decode/encode exist (decode_png/encode_png/render_item).")

func _open_slicer() -> void:
	if not visible:
		popup_centered()
	_slicer.popup_centered()

func _pick_sheet_image() -> void:
	var d := FileDialog.new()
	d.title = "Choose sprite sheet"
	d.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	d.access = FileDialog.ACCESS_FILESYSTEM
	d.add_filter("*.png ; PNG image")
	d.file_selected.connect(func(path: String):
		_load_sheet_image(path)
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	_slicer.add_child(d)
	d.popup_centered_ratio(0.6)

func _load_sheet_image(path: String) -> void:
	var img := Image.new()
	var err := img.load(path)
	if err != OK:
		_sheet_readout.text = "failed to load %s (error %d)" % [path.get_file(), err]
		return
	_sheet_image = img
	_sheet_preview.img_tex = ImageTexture.create_from_image(img)
	_sheet_preview.queue_redraw()
	_sheet_readout.text = "%s · %d × %d · %s" % [
		path.get_file(), img.get_width(), img.get_height(), path.get_extension().to_upper()]
	_refresh_slicer_summary()

func _refresh_slicer_summary() -> void:
	if _slicer_summary == null:
		return
	if _sheet_image == null:
		_slicer_summary.text = ""
		if _slice_btn:
			_slice_btn.text = "Slice"
		return
	var cols := int(_slicer_cols.value)
	var rows := int(_slicer_rows.value)
	var total := cols * rows
	var non_empty := _sample_non_empty(cols, rows, _slicer_margin.value, _slicer_spacing.value)
	_slicer_summary.text = "%d cells detected · ~%d non-empty (8×8-sampled, not an exact pixel scan)" % [total, non_empty]
	if _slice_btn:
		_slice_btn.text = "Slice %d cells" % total

## A coarse 8x8-sample-per-cell alpha probe -- real (reads the loaded
## `Image`'s own pixels), but approximate by design rather than a full pixel
## scan, since nothing downstream consumes the result (the slice op itself
## is unbacked). Labelled "sampled" in the summary so it never reads as exact.
func _sample_non_empty(cols: int, rows: int, margin: float, spacing: float) -> int:
	if _sheet_image == null or cols <= 0 or rows <= 0:
		return 0
	var w := _sheet_image.get_width()
	var h := _sheet_image.get_height()
	var cw := (w - margin * 2 - spacing * (cols - 1)) / float(cols)
	var ch := (h - margin * 2 - spacing * (rows - 1)) / float(rows)
	if cw <= 0 or ch <= 0:
		return 0
	var count := 0
	for ry in rows:
		for cx in cols:
			var x0 := margin + cx * (cw + spacing)
			var y0 := margin + ry * (ch + spacing)
			var found := false
			for sy in 8:
				for sx in 8:
					var px := int(x0 + (sx + 0.5) / 8.0 * cw)
					var py := int(y0 + (sy + 0.5) / 8.0 * ch)
					if px < 0 or py < 0 or px >= w or py >= h:
						continue
					if _sheet_image.get_pixel(px, py).a > 0.0:
						found = true
						break
				if found:
					break
			if found:
				count += 1
	return count

# ---------------------------------------------------------------------------
# Small helpers
# ---------------------------------------------------------------------------

## A disabled button carrying its reason as a tooltip -- `menus.gd`'s
## `_todo()` convention, for a plain `Control` window rather than a
## `PopupMenu`.
func _gap_button(parent: Control, text: String, reason: String) -> Button:
	var b := DccWidgets.action(parent, text, func(): pass)
	b.disabled = true
	b.tooltip_text = reason
	return b
