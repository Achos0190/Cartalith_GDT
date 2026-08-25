extends AcceptDialog
class_name AssetLibraryWindow

## §8's Asset library window (`DCC_SHELL_SPEC.md` §2.3/§8) -- Assets ▸
## ⧉ Asset library / ▦ Sprite sheet slicer's actual destination. Replaces
## v2.10's `#assetLibrary`.
##
## ## 2026-08-20: rebuilt against the design canvas, not against §8's prose
##
## The first version of this file was written from `DCC_SHELL_SPEC.md` §8's
## *description* of the window, before the engine bindings existed; the real
## `#[func]`s were then wired into that shape. The result was functionally
## complete and visually wrong -- default Godot buttons and option buttons on a
## floating 1180×760 dialog, a pack-metadata row bolted to the top, the family
## rail opening with a 90 px prose paragraph, batch actions as a row of filled
## slabs, tile captions floating outside their tiles, and an inspector that was
## a stack of label/value pairs. The 2026-08-20 visual sweep passed it by
## checking that the controls *worked*, which is not the same test.
##
## This file is now laid out from `design/Cartalith DCC Shell.dc.html`'s own
## `Asset library window 1920` screen, read as a literal spec: a full-bleed
## workspace window (borderless, sized under the app menu bar -- the mockup's
## "map hidden while open"), a 34 px window bar of outline chips, a 266 px
## family rail, a 6-column slot grid, a 330 px slot inspector and a 26 px
## status line. Every number below -- band heights, rail/inspector widths,
## paddings, the 76 px tile art band, the 56 px variant tiles, the 20 px
## swatches, the two 274/760 slicer columns -- is off that canvas. Every colour
## is a `DccTheme` token; the canvas and the theme share one palette, so no hex
## appears here.
##
## Four engine realities the mockup does not know about, kept rather than
## re-drawn (each is a recorded, disclosed decision -- see `GUI_GAP_REGISTER.md`):
##
## - **AS-16 · eight families, not 24.** §8 says "24 families… Settlements,
##   Terrain, Cartography, plus Collections." `cartalith-assets/src/slots.rs` +
##   `library.rs` define **eight** -- `textures`, `biomes`, `terrains`, `icons`,
##   `settlement`, `trait`, `poi`, `custom` -- and `ASSET_LIBRARY_SCOPE.md` §1
##   recorded exactly this when Phase 4's engine side was built. The rail keeps
##   the mockup's *visual grammar* (group headers, `code · name · filled/
##   capacity`, accent when incomplete) and lists the real eight; the FAMILIES
##   band's own tooltip carries the disclosure the old prose paragraph carried.
## - **AS-15 · anchor is family-level.** The mockup draws a per-slot
##   top/centre/base segmented control. `Family` fixes the anchor for every slot
##   in it, so the segment is drawn, the real one is lit, and the other two are
##   disabled with that reason.
## - **AS-14 · variants are weighted at render time.** There is no "active
##   variant", so the VARIANTS strip selects which one the *preview* shows and
##   says nothing about which one the map draws.
## - **Per-item scale/pan is read-only.** `as_item_summary` reports the real
##   transform; no `as_set_item_transform` exists, so Scale/Fit/Reset are drawn
##   in the mockup's own row shape and disabled with that reason. Replace… and
##   + Variant in the same row are real (`as_import_item` / `as_remove_item`).
##
## ## What is real vs. disclosed gap, control by control
##
## `GUI_GAP_REGISTER.md` rows AS-01..AS-08/AS-13/DM-05: `cartalith-godot`
## carries a real, live Asset Library authoring session
## (`asset_bridge::AssetLibrarySession`, a `WorldGen` field that survives a
## re-generate like `travel_library` does) behind an `as_*` `#[func]` surface.
##
## - **Real**: the family list, slot ids/titles, anchor/bake-size/variant
##   metadata, search/sort, the zoom control, the preview-background swatches,
##   Import asset pack .zip…, Import image… into the focused slot
##   (`as_import_item`/`as_add_custom_slot`), per-slot fill state and a real
##   baked thumbnail for every filled slot (`as_family_slots`/
##   `as_thumbnail_png`), the inspector's file/scale/tags readout
##   (`as_slot_summary`/`as_item_summary`), pack metadata (`as_pack_info`/
##   `as_set_pack_info`), the five batch operations (`as_batch_tag`/`collect`/
##   `rename`/`duplicate`/`delete`) -- batch Rename honestly split: a custom
##   slot is renamed for real, a frozen slot instead renames its *item
##   variants* (frozen slot names are engine constants, `slot_title`, not
##   editable -- AS-06), Validate (`as_validate`), Clear library…
##   (`as_clear_library`), Export pack .zip… (`as_export_pack_bytes`, bytes
##   written here via `FileAccess`) and Apply to map (`as_apply_to_map`, the
##   reference's own `applyToMap()`).
## - **Real, since the slicer pass (AS-09/AS-10/AS-11)**: the sprite-sheet
##   slicer, end to end. `cartalith-assets::slicer` carries a golden-verified
##   port of the reference's `SpriteSheetImporter` -- `computeCells` (whose
##   spacing is a *half-gutter on interior edges*, not a pitch), `cropCell`,
##   `applyChroma` and `isBlank`'s alpha>8 threshold -- and `as_load_sheet`/
##   `as_slice_preview`/`as_slice_apply` expose it. The `N cells detected · M
##   non-empty` readout is the engine's real detection pass; the grid overlay
##   draws engine-computed spans, so it shows the exact rectangles the slice
##   cuts; slicing is non-destructive. Two honest notes: *Trim transparent
##   edges* is a **port-side addition** (§8 asks for it, the reference has
##   `background → transparent` chroma keying instead, which is also wired
##   here), and *Assign to family / Fill from* is §8's framing of what the
##   reference expresses as a flat target-slot dropdown.
## - **Real, since the Collections/drag-and-drop/slicer-interaction pass**:
##   a Collections rail row (`as_collections`, `_refresh_collections_rail`)
##   listing every real collection with its member count, selectable into a
##   collection-scoped grid view (`_refresh_grid_collection`); in-app
##   drag-and-drop of one or more selected tiles onto a Collections row to add
##   them (`SlotCell._get_drag_data` / `CollectionRow._can_drop_data`/
##   `_drop_data`, real Godot virtuals, calling the same `as_batch_collect`
##   the Collect… prompt does); and the slicer's pan (wheel-zoom centred on
##   the cursor, middle-drag to pan), click-to-select-a-cell (a real picker/
##   highlight), and a draggable handle on the grid's own Margin boundary
##   (`SheetPreview`, AS-17).
## - **Real, since the AS-07/AS-12/AS-17 closeout pass (2026-08-23)**: per-item
##   scale (`as_set_item_transform`) and pan (two SpinBoxes -- no headless-safe
##   drag-to-pan equivalent, so this port exposes the same value a different,
##   smaller way rather than not at all) with real Fit/Reset
##   (`as_reset_item_transform`); a slot-less "Unassigned imports" holding
##   bucket (`UNASSIGNED_SET`, ordinary custom slots the footer's Import
##   image… lands in when no slot is focused, per `DCC_SHELL_SPEC.md` §8);
##   `cartalith_assets::SliceGrid::with_lines`/`move_line` giving the engine's
##   grid genuine per-interior-line positions (still uniform by default, now
##   draggable off it -- `SheetPreview`'s vertical/horizontal line handles,
##   `as_slicer_move_line`); and cell-scoped slicing, `as_slice_apply`'s new
##   `only_cell` narrowing a slice to the one selected cell instead of the
##   whole sheet.
## - **Disclosed gap, still honest**: dragging a file from OUTSIDE Godot onto a
##   slot to fill it -- Godot's own drag-and-drop is two unrelated systems, and
##   OS-external file drops only ever reach `Window.files_dropped`, never a
##   Control's `_can_drop_data`/`_drop_data`, so a slot cannot structurally be
##   that kind of drop target (use Import image… instead); there is no engine
##   primitive to *move* an already-assigned item into Unassigned imports
##   (only into/out of a Collection), so that bucket is reachable from imports
##   only, not from reassigning existing art.
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

## The five places a slice can land, in `as_slice_apply`'s own `target` terms.
## The first four are the reference's own `#alSlTarget` options; `family` is
## `DCC_SHELL_SPEC.md` §8's "Assign to family" + "Fill from", which the
## reference expresses as a flat slot dropdown instead.
const SLICE_TARGETS: Array[Dictionary] = [
	{"key": "family", "label": "a family, slot by slot", "needs": ["family", "fill"]},
	{"key": "slot", "label": "the focused slot", "needs": []},
	{"key": "new_custom", "label": "one new custom icon", "needs": ["name", "set"]},
	{"key": "per_cell", "label": "separate custom icons (per cell)", "needs": ["set"]},
]

# ---------------------------------------------------------------------------
# Geometry, read off `Asset library window 1920`
# ---------------------------------------------------------------------------

const W_RAIL := 266
const W_INSPECTOR := 330
const H_BAR := 34          ## window bar
const H_BAND := 28         ## the three column header bands
const H_STATUS := 26
const H_TILE_ART := 76     ## default; the zoom slider drives it
const TILE_GAP := 12
const GRID_COLS := 6
const PHONE_GRID_COLS := 2   ## PH-07; see `_build_slot_grid()`.
const W_SLICER := 760
## The canvas's card is 760 × ~390. This port's settings column carries three
## rows the canvas does not (the reference's own chroma key + tolerance, and
## the new-name / custom-set fields its flat target dropdown needs), so the
## card is taller by exactly those rows.
const H_SLICER := 560
const W_SLICER_SIDE := 274
const H_SHEET_PREVIEW := 296
## PH-07. Dp, on the same reasoning `city_viewer_window.gd`'s `PHONE_CANVAS_H`
## records: enough to read a sprite sheet's grid at the fit scale, and under a
## third of the screen so the settings column below opens on real controls.
const PHONE_SHEET_PREVIEW := 240
const W_INSP_LABEL := 70
const SZ_VARIANT := 56
const SZ_SWATCH := 20

## The disclosure the family rail used to spend 90 px of prose on. Same words,
## now on the FAMILIES band's tooltip so the rail can look like the canvas.
const FAMILIES_NOTE := "Eight families, frozen against the reference engine (cartalith-assets::slots / library) -- not the design canvas's own 24. The canvas subdivides more finely (splitting e.g. \"Feature icons\" into \"Trees & cover\" / \"Rock & scree\"); no Rust type draws that line, and ASSET_LIBRARY_SCOPE.md §1 recorded the real eight when Phase 4's engine side was built. Capacity and fill counts are both real (AssetDB::slots_in_family + per-slot filled state)."

## AS-07 closed 2026-08-23: `as_set_item_transform`/`as_reset_item_transform`
## exist now, so Scale/Pan/Fit/Reset below write straight through them.

## AS-12's holding area (DCC_SHELL_SPEC.md §8's rail: "plus Collections (tag
## sets, Unassigned imports with count)"): the reserved custom-slot `set` a
## slot-less import lands under, since `cartalith_assets::AssetDB` has no
## uid-less item concept for a real slot-less bucket to sit on.
const UNASSIGNED_SET := "Unassigned imports"

# ---------------------------------------------------------------------------
# Small drawn controls -- slot tile art, preview swatch, sheet-slicer preview
# ---------------------------------------------------------------------------

## A slot tile's art band, the inspector's preview, and the preview-background
## swatches are all the same drawing problem: a checkerboard ground (honest "no
## art data" rather than a guessed empty state) or a flat swatch, optionally
## with a baked thumbnail over it, plus the canvas's own corner marks. `_draw()`
## custom canvas, matching how `tool_overlay.gd`/`map_overlay.gd` already draw.
class SlotCell extends Control:
	var uid := ""
	var selected := false
	var bg_mode := "checker"   ## "checker" | "color"
	var bg_color := Color(1, 1, 1)
	var checker_px := 10.0     ## canvas: 10 px in a tile, 12 px in the preview
	## A real baked thumbnail (`as_thumbnail_png`) once one has loaded --
	## `null` means "no art data queried/found yet", still the checkerboard.
	var thumb: ImageTexture
	## Canvas marks: the word `empty` on an unfilled tile, a `×N` variant badge
	## bottom-right, a `☑` top-right while selected. `false`/0 draws none.
	var show_empty := false
	var variant_count := 0
	var show_check := false
	var draw_border := true
	## The canvas draws a *tile's* art on a flat ground (`#191c1e`) and the
	## *inspector preview's* art on the checkerboard, so straight alpha is
	## visible where it matters. One flag, because they are otherwise the same
	## control.
	var checker_under_art := false
	## Drag source (AS-12/AS-17 territory note: drag-and-drop onto slots).
	## Only the grid's own tiles set both -- the inspector preview and the
	## preview-background swatches reuse this same class for their drawing
	## and must not become drag sources by accident.
	var draggable := false
	var owner_window: AssetLibraryWindow

	func _ready() -> void:
		mouse_filter = Control.MOUSE_FILTER_STOP

	## Dragging a tile carries either the whole current multi-selection (if
	## this tile is part of one) or just itself -- `owner_window` decides
	## which, since only it knows `_selected`. Empty selection (an unfilled
	## slot with nothing to move) returns null, which Godot reads as "no drag
	## started" -- the standard way to refuse one from `_get_drag_data`.
	func _get_drag_data(_at_position: Vector2) -> Variant:
		if not draggable or uid == "" or owner_window == null:
			return null
		var uids: PackedStringArray = owner_window._drag_uids_for(uid)
		if uids.is_empty():
			return null
		var preview := Label.new()
		preview.text = ("%s (%d)" % [uids[0], uids.size()]) if uids.size() > 1 else uids[0]
		preview.add_theme_color_override("font_color", DccTheme.c("text_bright"))
		preview.add_theme_stylebox_override("normal", DccTheme.flat(DccTheme.c("raised")))
		set_drag_preview(preview)
		return {"type": "asset_uids", "uids": uids}

	func _draw() -> void:
		var r := Rect2(Vector2.ZERO, size)
		if thumb != null and not (checker_under_art and bg_mode == "checker"):
			draw_rect(r, DccTheme.c("raised") if bg_mode == "checker" else bg_color, true)
		elif bg_mode == "checker":
			## `sunken`/`raised`, not `sunken`/`panel_alt`: those two tokens are
			## one level apart and the checkerboard came out invisible.
			var a := DccTheme.c("sunken")
			var b := DccTheme.c("raised")
			var yy := 0.0
			while yy < r.size.y:
				var xx := 0.0
				while xx < r.size.x:
					var idx := int(xx / checker_px) + int(yy / checker_px)
					draw_rect(Rect2(xx, yy,
						minf(checker_px, r.size.x - xx), minf(checker_px, r.size.y - yy)),
						a if idx % 2 == 0 else b, true)
					xx += checker_px
				yy += checker_px
		else:
			draw_rect(r, bg_color, true)
		if thumb != null:
			draw_texture_rect(thumb, r, false)

		var font := DccTheme.mono(0)
		if show_empty and thumb == null:
			draw_string(font, Vector2(0.0, r.size.y * 0.5 + 3.0), "empty",
				HORIZONTAL_ALIGNMENT_CENTER, r.size.x, DccTheme.FS_MICRO,
				DccTheme.c("text_ghost"))
		if variant_count > 1:
			draw_string(font, Vector2(0.0, r.size.y - 5.0), "×%d" % variant_count,
				HORIZONTAL_ALIGNMENT_RIGHT, r.size.x - 4.0, DccTheme.FS_MICRO,
				DccTheme.c("accent"))
		if show_check and selected:
			draw_string(font, Vector2(0.0, 13.0), DccIcons.SYMBOLS["checked"],
				HORIZONTAL_ALIGNMENT_RIGHT, r.size.x - 4.0, DccTheme.FS_TINY,
				DccTheme.c("accent"))
		if draw_border:
			draw_rect(r, DccTheme.c("accent") if selected else DccTheme.c("line"), false, 1.0)

## A Collections-rail row (AS-12) -- a real drop target: dropping a
## `SlotCell`'s drag payload here adds every dragged uid to this collection
## (`as_batch_collect`). A dedicated `Button` subclass, the same reason
## `SlotCell` above is its own `class` rather than a bare node -- Godot's
## `_can_drop_data`/`_drop_data` are virtuals a script has to declare, and a
## plain `Button.new()` from `_rail_row`'s shared helper cannot carry them.
class CollectionRow extends Button:
	var owner_window: AssetLibraryWindow
	var coll_name := ""

	func _can_drop_data(_at_position: Vector2, data: Variant) -> bool:
		return typeof(data) == TYPE_DICTIONARY and String(data.get("type", "")) == "asset_uids" \
			and not (data.get("uids", []) as Array).is_empty()

	func _drop_data(_at_position: Vector2, data: Variant) -> void:
		if owner_window != null:
			owner_window._on_drop_uids_on_collection(coll_name, data.get("uids", []))

## The slicer modal's sheet preview -- a real loaded `Image` plus the real cell
## rectangles the engine will cut, and (AS-17) real canvas interaction on top:
## wheel-zoom, middle-drag pan, click-to-select a cell, and a draggable handle
## on the grid's own Margin.
##
## The spans are **not** computed here. `computeCells`'s spacing is a
## half-gutter on interior edges only, so the outer cells come out wider than
## the interior ones, and the obvious equal-pitch formula this class used to
## carry drew a grid the slice did not actually follow. `as_slice_preview`
## hands back `col_x0`/`col_x1`/`row_y0`/`row_y1` in sheet pixels
## (`cartalith-assets::slicer::CellGrid::column_spans`), and this only maps
## them into view space -- the presentation half, which is all that belongs
## in GDScript. The canvas draws those spans dashed at 35% accent; that is a
## stroke change only, and does not touch the arithmetic above.
##
## AS-17's own gap-register note explains why the grid lines below are
## draggable on exactly one thing, not arbitrarily: `cartalith-assets::slicer`
## computes a uniform grid from `cols`/`rows`/`margin`/`spacing`
## (`SliceGrid`/`compute_cells`) -- there is no per-line position an engine
## call could accept, so a line that isn't Margin's own boundary has nothing
## real to drag it *to*. Margin is real (`GridRect::inset`, a uniform inset of
## the whole sheet), so that boundary -- and only that one -- gets a handle.
## Per-cell click-to-select is real as a picker/highlight; it does not narrow
## what Slice cuts, because `as_slice_apply` has no cell-selection parameter
## to narrow it with (`slice_target_from`, `lib.rs` -- always the whole grid,
## minus Skip empty cells). Disclosed here and in the grid-footer note this
## pass leaves in place, not silently implied to do more than it does.
class SheetPreview extends Control:
	var img_tex: ImageTexture
	var col_x0: PackedFloat64Array = PackedFloat64Array()
	var col_x1: PackedFloat64Array = PackedFloat64Array()
	var row_y0: PackedFloat64Array = PackedFloat64Array()
	var row_y1: PackedFloat64Array = PackedFloat64Array()
	## AS-17: the *undisplaced* division lines (`as_slice_preview`'s
	## `col_lines_px`/`row_lines_px`) -- a drag handle's hit-test and draw
	## target, distinct from `col_x0`/`col_x1`'s gutter-narrowed cell edges.
	## `cols+1`/`rows+1` entries; indices `0` and the last are the outer sheet
	## edges (undraggable -- the grid rect's own margin owns those).
	var col_lines_px: PackedFloat64Array = PackedFloat64Array()
	var row_lines_px: PackedFloat64Array = PackedFloat64Array()
	## Cell indices the engine's detection pass found empty, so the overlay can
	## dim them the way §8's "19 non-empty" readout implies.
	var blank_cells: Dictionary = {}
	var usable := true
	var owner_window: AssetLibraryWindow

	## View transform: `zoom` multiplies the auto-fit scale, `pan` is an extra
	## pixel offset on top of the auto-centred position -- so `zoom=1,
	## pan=ZERO` reproduces the old fixed-fit behaviour exactly.
	var zoom := 1.0
	var pan := Vector2.ZERO
	var selected_cell := -1   ## flat row*cols+col index, -1 = none

	const MIN_ZOOM := 0.25
	const MAX_ZOOM := 8.0
	const HANDLE_RADIUS := 6.0

	var _panning := false
	var _dragging_margin := false
	var _dragging_line_axis := ""   ## "col" / "row" / "" (not dragging a line)
	var _dragging_line_index := -1

	func _ready() -> void:
		mouse_filter = Control.MOUSE_FILTER_STOP
		focus_mode = Control.FOCUS_CLICK

	func reset_view() -> void:
		zoom = 1.0
		pan = Vector2.ZERO
		queue_redraw()

	## The same fit-then-zoom-then-pan transform `_draw()` uses, factored out
	## so input handling can convert between screen and sheet space without
	## duplicating (and risking drifting from) the drawing math.
	func _transform() -> Dictionary:
		var r := Rect2(Vector2.ZERO, size)
		if img_tex == null:
			return {"scale": 1.0, "origin": r.position}
		var tex_size := img_tex.get_size()
		if tex_size.x <= 0 or tex_size.y <= 0:
			return {"scale": 1.0, "origin": r.position}
		var fit_scale: float = minf(r.size.x / tex_size.x, r.size.y / tex_size.y)
		var scale: float = fit_scale * zoom
		var draw_size := tex_size * scale
		var origin := r.position + (r.size - draw_size) * 0.5 + pan
		return {"scale": scale, "origin": origin}

	func _screen_to_sheet(local_pos: Vector2) -> Vector2:
		var t := _transform()
		return (local_pos - (t["origin"] as Vector2)) / float(t["scale"])

	func _sheet_to_screen(sheet_pos: Vector2) -> Vector2:
		var t := _transform()
		return (t["origin"] as Vector2) + sheet_pos * float(t["scale"])

	## The Margin handle sits at the grid rect's own top-left corner in sheet
	## space -- `GridRect::inset`'s `(margin, margin)`. `-1` (no handle) before
	## a sheet is loaded.
	func _margin_handle_sheet_pos() -> Vector2:
		if owner_window == null:
			return Vector2(-1, -1)
		var m: float = owner_window._slicer_margin.value
		return Vector2(m, m)

	func _find_cell(sheet_pos: Vector2) -> int:
		if col_x0.is_empty() or row_y0.is_empty():
			return -1
		var col := -1
		for i in col_x0.size():
			if sheet_pos.x >= col_x0[i] and sheet_pos.x < col_x1[i]:
				col = i
				break
		var row := -1
		for j in row_y0.size():
			if sheet_pos.y >= row_y0[j] and sheet_pos.y < row_y1[j]:
				row = j
				break
		if col < 0 or row < 0:
			return -1
		return row * col_x0.size() + col

	## AS-17: which interior line (if any) `local_pos` is close enough to
	## grab -- the outer two entries of `col_lines_px`/`row_lines_px` (indices
	## `0` and the last) are the sheet edges, not draggable lines, so they are
	## excluded from the start rather than relying on `move_line`'s own
	## no-op refusal to keep the click from being swallowed for nothing.
	func _find_line(local_pos: Vector2) -> Dictionary:
		var t := _transform()
		var origin: Vector2 = t["origin"]
		var scale: float = t["scale"]
		var tex_size: Vector2 = img_tex.get_size() if img_tex != null else Vector2.ZERO
		var draw_size := tex_size * scale
		var reach := HANDLE_RADIUS + 3.0
		if col_lines_px.size() > 2 and local_pos.y >= origin.y - reach and local_pos.y <= origin.y + draw_size.y + reach:
			for i in range(1, col_lines_px.size() - 1):
				var sx := origin.x + float(col_lines_px[i]) * scale
				if absf(local_pos.x - sx) <= reach:
					return {"axis": "col", "index": i}
		if row_lines_px.size() > 2 and local_pos.x >= origin.x - reach and local_pos.x <= origin.x + draw_size.x + reach:
			for j in range(1, row_lines_px.size() - 1):
				var sy := origin.y + float(row_lines_px[j]) * scale
				if absf(local_pos.y - sy) <= reach:
					return {"axis": "row", "index": j}
		return {}

	func _gui_input(event: InputEvent) -> void:
		if img_tex == null:
			return
		if event is InputEventMouseButton:
			var mb := event as InputEventMouseButton
			if mb.button_index == MOUSE_BUTTON_WHEEL_UP and mb.pressed:
				_zoom_at(mb.position, 1.2)
				accept_event()
			elif mb.button_index == MOUSE_BUTTON_WHEEL_DOWN and mb.pressed:
				_zoom_at(mb.position, 1.0 / 1.2)
				accept_event()
			elif mb.button_index == MOUSE_BUTTON_MIDDLE:
				_panning = mb.pressed
				accept_event()
			elif mb.button_index == MOUSE_BUTTON_LEFT:
				if mb.pressed:
					var handle_screen := _sheet_to_screen(_margin_handle_sheet_pos())
					var line_hit := _find_line(mb.position)
					if handle_screen.distance_to(mb.position) <= HANDLE_RADIUS + 3.0:
						_dragging_margin = true
					elif not line_hit.is_empty():
						_dragging_line_axis = String(line_hit["axis"])
						_dragging_line_index = int(line_hit["index"])
					else:
						var idx := _find_cell(_screen_to_sheet(mb.position))
						## AS-17: clicking the already-selected cell again
						## clears it -- click-to-toggle deselect, since there
						## is no separate "clear selection" control here.
						selected_cell = -1 if idx == selected_cell and idx >= 0 else idx
						queue_redraw()
						if owner_window != null:
							owner_window._on_slicer_cell_selected(selected_cell)
				else:
					_dragging_margin = false
					_dragging_line_axis = ""
					_dragging_line_index = -1
				accept_event()
		elif event is InputEventMouseMotion:
			var mm := event as InputEventMouseMotion
			if _panning:
				pan += mm.relative
				queue_redraw()
				accept_event()
			elif _dragging_margin and owner_window != null:
				var sp := _screen_to_sheet(mm.position)
				var tex_size := img_tex.get_size()
				var cap: float = maxf(0.0, minf(tex_size.x, tex_size.y) * 0.5 - 1.0)
				var new_margin: float = clampf(minf(sp.x, sp.y), 0.0, cap)
				owner_window._on_slicer_margin_dragged(new_margin)
				accept_event()
			elif _dragging_line_axis != "" and owner_window != null:
				var sp := _screen_to_sheet(mm.position)
				var value: float = sp.x if _dragging_line_axis == "col" else sp.y
				owner_window._on_slicer_line_dragged(_dragging_line_axis, _dragging_line_index, value)
				accept_event()

	func _zoom_at(local_pos: Vector2, factor: float) -> void:
		var before := _screen_to_sheet(local_pos)
		zoom = clampf(zoom * factor, MIN_ZOOM, MAX_ZOOM)
		var t := _transform()
		var after_origin: Vector2 = local_pos - before * float(t["scale"])
		var r := Rect2(Vector2.ZERO, size)
		var tex_size: Vector2 = img_tex.get_size()
		var fit_scale: float = minf(r.size.x / tex_size.x, r.size.y / tex_size.y)
		var draw_size := tex_size * fit_scale * zoom
		pan = after_origin - r.position - (r.size - draw_size) * 0.5
		queue_redraw()

	func _draw() -> void:
		var r := Rect2(Vector2.ZERO, size)
		draw_rect(r, DccTheme.c("sunken"), true)
		draw_rect(r, DccTheme.c("line"), false, 1.0)
		if img_tex == null:
			return
		var tex_size := img_tex.get_size()
		if tex_size.x <= 0 or tex_size.y <= 0:
			return
		var t := _transform()
		var scale: float = t["scale"]
		var origin: Vector2 = t["origin"]
		var draw_size := tex_size * scale
		draw_texture_rect(img_tex, Rect2(origin, draw_size), false)
		if not usable or col_x0.is_empty() or row_y0.is_empty():
			return
		var line_color := DccTheme.c("accent")
		line_color.a = 0.35
		var cols := col_x0.size()
		for j in row_y0.size():
			for i in cols:
				var cell := Rect2(
					origin.x + float(col_x0[i]) * scale,
					origin.y + float(row_y0[j]) * scale,
					float(col_x1[i] - col_x0[i]) * scale,
					float(row_y1[j] - row_y0[j]) * scale)
				if blank_cells.has(j * cols + i):
					draw_rect(cell, Color(0, 0, 0, 0.45), true)
				if j * cols + i == selected_cell:
					draw_rect(cell, DccTheme.c("accent"), false, 2.0)
				draw_dashed_line(cell.position + Vector2(cell.size.x, 0.0),
					cell.position + cell.size, line_color, 1.0, 4.0)
				draw_dashed_line(cell.position + Vector2(0.0, cell.size.y),
					cell.position + cell.size, line_color, 1.0, 4.0)
		## The Margin handle -- a filled dot so it reads as grabbable, distinct
		## from the dashed cell lines it sits among.
		var handle_pos := _sheet_to_screen(_margin_handle_sheet_pos())
		draw_circle(handle_pos, HANDLE_RADIUS, DccTheme.c("accent") if _dragging_margin else DccTheme.c("text_bright"))
		draw_circle(handle_pos, HANDLE_RADIUS, DccTheme.c("bg"), false, 1.5)
		## AS-17's interior-line handles: a small diamond at the line's
		## midpoint (col lines) or midpoint (row lines), same grabbable-dot
		## language as the Margin handle so both read as "drag me" rather than
		## one looking real and the other looking decorative.
		if col_lines_px.size() > 2:
			var mid_y := origin.y + draw_size.y * 0.5
			for i in range(1, col_lines_px.size() - 1):
				var hp := Vector2(origin.x + float(col_lines_px[i]) * scale, mid_y)
				var on: bool = _dragging_line_axis == "col" and _dragging_line_index == i
				draw_circle(hp, HANDLE_RADIUS * 0.7, DccTheme.c("accent") if on else DccTheme.c("text_faint"))
				draw_circle(hp, HANDLE_RADIUS * 0.7, DccTheme.c("bg"), false, 1.0)
		if row_lines_px.size() > 2:
			var mid_x := origin.x + draw_size.x * 0.5
			for j in range(1, row_lines_px.size() - 1):
				var hp2 := Vector2(mid_x, origin.y + float(row_lines_px[j]) * scale)
				var on2: bool = _dragging_line_axis == "row" and _dragging_line_index == j
				draw_circle(hp2, HANDLE_RADIUS * 0.7, DccTheme.c("accent") if on2 else DccTheme.c("text_faint"))
				draw_circle(hp2, HANDLE_RADIUS * 0.7, DccTheme.c("bg"), false, 1.0)

# ---------------------------------------------------------------------------
# State
# ---------------------------------------------------------------------------

var _host: DccApp
var _bridge: EngineBridge

var _current_family := ""
var _search_text := ""
var _sort_mode := 0          ## 0 = slot order, 1 = name
var _cell_px := float(H_TILE_ART)
var _select_mode := false
var _selected: Dictionary = {}     ## uid -> true
var _last_index := -1
var _focused_uid := ""
var _preview_index := 0            ## which variant the inspector preview shows (AS-14)
var _slot_order: Array = []        ## current family's filtered/sorted entries
var _cells: Dictionary = {}        ## uid -> {"tile": PanelContainer, "cell": SlotCell}
var _preview_bg := "checker"
var _preview_color := Color(1, 1, 1)
## Set by every mutating call, cleared by Apply to map -- the status line's own
## `● library edited — apply to map to use it`.
var _dirty := false

var _sort_button: OptionButton
var _select_mode_btn: Button
var _rail_buttons: Dictionary = {}   ## family key -> {button, code, name, count}
var _rail_count_label: Label
## AS-12's Collections rail: non-empty means the grid is showing a
## collection's members instead of a family's slots (`_refresh_grid_collection`).
var _current_collection := ""
var _collection_buttons: Dictionary = {}   ## collection name -> {button, name, count}
var _collections_rail_body: VBoxContainer
## AS-12's "Unassigned imports" holding bucket: a pinned row above the
## dynamic collections list (`_build_unassigned_row`), backed by ordinary
## custom slots under the reserved `UNASSIGNED_SET` name -- see
## `as_add_custom_slot`'s own doc comment, which names this bucket as the
## real engine call it would sit on top of.
var _current_unassigned := false
var _unassigned_row: Dictionary = {}   ## {button, name, count}
var _grid: GridContainer
var _grid_header: Label
var _select_count_label: Label

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

## The inspector is built once and refreshed in place -- rebuilding it on every
## selection is what used to clobber the pack-metadata fields mid-typing.
var _insp_head: Label
var _insp_empty: Label
var _insp_detail: VBoxContainer
var _insp_preview: SlotCell
var _insp_file: Label
var _insp_scale: HSlider
var _insp_scale_readout: Label
var _insp_pan_x: SpinBox
var _insp_pan_y: SpinBox
var _insp_fit_btn: Button
var _insp_reset_btn: Button
var _insp_syncing := false   ## true while _refresh_inspector writes the controls above
var _insp_replace_btn: Button
var _insp_variant_btn: Button
var _insp_swatches: Array = []        ## SlotCell, parallel to PREVIEW_SWATCHES
var _insp_anchor_chips: Dictionary = {}
var _insp_tags: HFlowContainer
var _insp_variants_head: Label
var _insp_variants: HBoxContainer
var _insp_note: Label

var _status_state: Label
var _status_pack: Label

var _slicer: AcceptDialog
var _sheet_image: Image
var _sheet_loaded := false          ## the engine holds a decoded sheet (`as_load_sheet`)
var _sheet_preview: SheetPreview
var _sheet_readout: Label
var _slicer_reset_view_btn: Button
var _slicer_cols: SpinBox
var _slicer_rows: SpinBox
var _slicer_margin: SpinBox
var _slicer_spacing: SpinBox
var _slicer_summary: Label
var _slice_btn: Button
var _slicer_trim: Button
var _slicer_skip: Button
var _slicer_chroma: Button
var _slicer_chroma_color: ColorPickerButton
var _slicer_chroma_tol: SpinBox
var _slicer_target: OptionButton
var _slicer_family: OptionButton
var _slicer_fill_chips: Dictionary = {}
var _slicer_name: LineEdit
var _slicer_set: LineEdit
## AS-17: `cartalith_assets::SliceGrid::col_lines`/`row_lines` overrides, in
## the engine's own fraction units -- empty means "uniform, no override" (the
## reference's own always-uniform grid), matching `as_slicer_move_line`'s and
## `_slice_opts()`'s own empty-means-None convention.
var _slicer_col_lines: PackedFloat64Array = PackedFloat64Array()
var _slicer_row_lines: PackedFloat64Array = PackedFloat64Array()

var _slice_trim := false
var _slice_skip_empty := true
var _slice_chroma := false
var _slice_target_index := 0
var _slice_family_index := 0
var _slice_overwrite := false

# ---------------------------------------------------------------------------
# Phone (§13) -- PH-07
# ---------------------------------------------------------------------------
#
# This window is the one the owner named on the OnePlus 12 device pass
# (2026-08-25): *"not all screens are optimised for a mobile phone, among
# others the asset manager screen."* It had none of the shell's three-call
# phone treatment (`DccWidgets.phone_window` / `phone_present` /
# `DccShell.phone_fit`), so `_popup_full()` filled a 1440x3168 panel with a
# composition authored at 1180x760 and drew every one of it at **native device
# resolution**. Measured before the fix, at 1440x3168 (`_ph9_probe.gd`):
# 59 tappable controls, all 59 under §13's floor (44 dp = 161 physical px at
# this density), the smallest **13 px** -- about 0.65 mm on a 510 ppi panel.
#
# The content scale fixes the density. What it cannot fix is the composition:
# the canvas's three columns are a 266 px rail, a flexible grid and a 330 px
# inspector, and 266 + 330 = 596 of the 393 dp a phone has before the grid gets
# a single pixel. `phone_window()` returns `is_phone` precisely so a caller can
# answer that, and this window answers it the way §13 answers it for the docks
# -- *"docks become full-height sheets, one at a time"*. The three columns
# become three full-width panes behind a segmented switcher, and the switcher
# follows the work rather than waiting to be pressed: picking a family moves to
# SLOTS, focusing a slot moves to SLOT. A slot-less import that lands in
# Unassigned imports does both, in that order, which is what it does on the
# desktop composition too -- there all three panes are simply visible at once.
#
# The window bar is the other thing that does not survive 393 dp: a 340 px
# search well plus six chips is ~880 px of minimum width, and a `BoxContainer`
# handed more minimum than it has does not clip, it **overlaps**
# (`open_project_dialog.gd`'s own finding). Phone splits it into a full-width
# search row over an `HFlowContainer` of chips, which wraps instead.
var _phone := false
var _phone_pane := ""                  ## "families" | "slots" | "slot"
var _phone_pane_buttons: Dictionary = {}
var _phone_rail: Control
var _phone_grid: Control
var _phone_inspector: Control
var _phone_title: Label
var _slicer_phone := false

# ---------------------------------------------------------------------------
# The canvas's control vocabulary
#
# The `Asset library window 1920` screen draws exactly four kinds of control,
# and none of them is a stock Godot widget: an outline **chip** (`padding:4px
# 9px; border:1px solid`), a smaller outline **segment** (`3px 8px`, used for
# anchor/fill-from), an outline **well** (a bordered text field), and a plain
# **text button** (the grid header's batch verbs, which carry no border at all).
#
# They were written here, with the note *"if a second window needs them, they
# move."* The 2026-08-20 Data manager rebuild is that second window, so the
# bodies now live in `dcc_widgets.gd` alongside the dock vocabulary. These
# eight stay as one-line delegators so none of this file's 74 call sites moved
# with them -- a rename across a 2 300-line file is churn that would have
# obscured the diff of a rebuild happening in the very next commit.
# ---------------------------------------------------------------------------

static func _box(border_token: String, bg_token: String, px: int, py: int) -> StyleBoxFlat:
	return DccWidgets.box(border_token, bg_token, px, py)

static func _chip(parent: Control, text: String, on_press: Callable,
		accent: bool = false, px: int = 9, py: int = 4) -> Button:
	return DccWidgets.chip(parent, text, on_press, accent, px, py)

static func _segment(parent: Control, text: String, on_press: Callable) -> Button:
	return DccWidgets.segment(parent, text, on_press)

static func _set_segment_on(b: Button, on: bool) -> void:
	DccWidgets.set_segment_on(b, on)

static func _well(le: Control, px: int = 9, py: int = 4, accent: bool = false) -> void:
	DccWidgets.well(le, px, py, accent)

static func _text_button(parent: Control, text: String, on_press: Callable) -> Button:
	return DccWidgets.text_button(parent, text, on_press)

static func _band(parent: Control, pad_x: int, gap: int = 14) -> HBoxContainer:
	return DccWidgets.band(parent, pad_x, gap, H_BAND)

static func _rule_soft() -> Control:
	var r := ColorRect.new()
	r.color = DccTheme.c("line_soft")
	r.custom_minimum_size = Vector2(0, 1)
	r.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	return r

## §11: no fills, radius 0, a 2 px rule with the travelled part in accent.
## Same treatment `DccWidgets` gives its dock sliders; repeated rather than
## reached into, because that one is private to its own row builder.
static func _style_slider(s: HSlider) -> void:
	var track := StyleBoxFlat.new()
	track.bg_color = DccTheme.c("line")
	track.content_margin_top = 1
	track.content_margin_bottom = 1
	s.add_theme_stylebox_override("slider", track)
	var filled := StyleBoxFlat.new()
	filled.bg_color = DccTheme.c("accent")
	s.add_theme_stylebox_override("grabber_area", filled)
	s.add_theme_stylebox_override("grabber_area_highlight", filled)
	s.add_theme_icon_override("grabber", ImageTexture.new())
	s.add_theme_icon_override("grabber_highlight", ImageTexture.new())
	s.add_theme_icon_override("grabber_disabled", ImageTexture.new())
	s.add_theme_constant_override("center_grabber", 1)

## A `70px label · control` inspector row (`W_INSP_LABEL`, canvas value).
static func _insp_row(parent: Control, label_text: String) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	var l := DccTheme.label(label_text, "text_dim", DccTheme.FS_SMALL)
	l.custom_minimum_size.x = W_INSP_LABEL
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	row.add_child(l)
	parent.add_child(row)
	return row

static func _pad(parent: Control, l: int, t: int, r: int, b: int) -> MarginContainer:
	return DccWidgets.pad(parent, l, t, r, b)

# ---------------------------------------------------------------------------
# Setup
# ---------------------------------------------------------------------------

func setup(host: DccApp, bridge: EngineBridge) -> void:
	_host = host
	_bridge = bridge
	title = "⧉ ASSET LIBRARY"
	get_ok_button().hide()   ## the window bar's own Close chip replaces it.
	## The canvas draws this as a full-bleed workspace with its own 34 px window
	## bar, not as a floating dialog with an OS title bar -- so the OS chrome
	## comes off and `_popup_full()` below sizes it under the app menu bar.
	borderless = true
	add_theme_stylebox_override("panel", DccTheme.panel("bg"))
	add_theme_constant_override("buttons_min_height", 0)
	add_theme_constant_override("margin", 0)
	## `AcceptDialog` enables `wrap_controls` in its constructor: the window
	## grows to its contents' minimum size on every `child_controls_changed()`,
	## and only ever grows. A full-bleed window sized by `_popup_full()` must
	## not, or one oversized child min -- for one frame -- pushes its own status
	## line past the bottom edge for good. Found on the Data manager rebuild
	## (2026-08-20); see that file's `setup()` for the measurement.
	wrap_controls = false
	size = Vector2i(1180, 760)
	min_size = Vector2i(1024, 640)
	## PH-07. Installs the rotation relay and reports whether the three columns
	## have to stack; also re-asserts `wrap_controls = false`, which this file
	## already set for its own reasons two lines up.
	_phone = DccWidgets.phone_window(self, host)
	## PH-07: two columns of ~168 dp each, so the canvas's 76 dp art band would
	## letterbox every tile. 120 keeps it near square and is inside the zoom
	## slider's own 56..132 range, so it is a starting point rather than a
	## second set of bounds.
	if _phone:
		_cell_px = 120.0
	_bridge.world_loaded.connect(func(): _refresh_pack_status())
	_build()
	_build_slicer_modal()
	## `1.0`, not `phone_scale()`: `phone_present()` applies the scale once as
	## the window's `content_scale_factor`, and applying it again here would
	## square it. Idempotent by meta-flag, so `_refresh_grid()`'s own re-fit
	## below only ever touches the cells it just made.
	if _phone:
		_host.phone_fit(self, 1.0)
		_host.phone_fit(_slicer, 1.0)
	## Escape / the titlebar X path -- the explicit Close chip handles the click
	## path, this handles the other two ways this dialog closes.
	close_requested.connect(_close_slicer)
	canceled.connect(_close_slicer)

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

## The canvas's own placement: the window occupies everything below the app
## menu bar, which is what "map hidden while open" means in a shell that has no
## separate workspace stack for windows.
##
## Sized from the host Control's viewport rect, **not** `get_tree().root.size`
## -- see `data_manager_window.gd::_popup_full()` for the full finding. Short
## version: `Window.size` is physical pixels, an embedded subwindow's `Rect2i`
## is the parent viewport's 2D space, and on a HiDPI display the two differ by
## the content scale, which pops the window at twice the height it should be and
## drops its own status line off the bottom edge with no scroll that can reach
## it. Found during the Data manager rebuild (2026-08-20); this file had the
## same line and the same bug.
func _popup_full() -> void:
	## PH-07: a phone fills the screen *including* the app menu bar's band --
	## there is no desktop menu bar there to sit under (§13 relocates it into
	## the ⋯ overflow sheet), and the 34 px reserved for one is 125 physical px
	## of nothing at this density.
	if DccWidgets.phone_present(self, _host):
		return
	var vp: Vector2 = _host.get_viewport_rect().size if _host != null \
		else Vector2(get_tree().root.get_visible_rect().size)
	var top := DccTheme.H_MENU_BAR
	var w: int = maxi(int(vp.x), min_size.x)
	var h: int = maxi(int(vp.y) - top, min_size.y)
	popup(Rect2i(0, top, w, h))

## `family_key` scopes the family rail's selection (Assets ▸ Icon families ▸
## / Texture sets ▸ open the window this way); `open_slicer` opens the slicer
## modal on top, per §2.3's "opens the library window with the slicer modal
## already open."
func open(family_key: String = "", open_slicer: bool = false) -> void:
	_popup_full()
	_refresh_pack_status()
	if family_key != "" and not _family_by_key(family_key).is_empty():
		_select_family(family_key)
	elif _current_family == "":
		_select_family(String(FAMILIES[0]["key"]))
	if open_slicer:
		_open_slicer()

# ---------------------------------------------------------------------------
# Layout -- window bar / rail · grid · inspector / status line
# ---------------------------------------------------------------------------

func _build() -> void:
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)

	outer.add_child(_build_window_bar())
	if _phone:
		outer.add_child(_build_phone_switcher())

	## PH-07: three columns on a pointer, three panes one at a time on a phone.
	## See the phone section at the top of this file for the arithmetic that
	## leaves no third option.
	var main: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	main.add_theme_constant_override("separation", 0)
	main.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(main)

	_phone_rail = _build_family_rail()
	_phone_grid = _build_slot_grid()
	_phone_inspector = _build_inspector()
	main.add_child(_phone_rail)
	main.add_child(_phone_grid)
	main.add_child(_phone_inspector)

	outer.add_child(_build_status_line())
	if _phone:
		## The header a borderless window draws in place of the title bar it
		## gave up. Its subtitle tracks the pane, so `_show_phone_pane()` owns
		## the text from here on.
		_phone_title = DccWidgets.phone_head(outer, "Asset library", "families")
		_show_phone_pane("families")

## §13's *"docks become full-height sheets, one at a time"*, applied to a
## window whose three columns are exactly that shape. A segmented row rather
## than a `TabContainer`: the panes are already built and already own their own
## headers/bands, and a `TabContainer` would reparent them and draw a second
## one.
func _build_phone_switcher() -> Control:
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"bottom": 1}))
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 0)
	_pad(wrap, 8, 4, 8, 4).add_child(row)
	for spec in [["families", "FAMILIES"], ["slots", "SLOTS"], ["slot", "SLOT"]]:
		var key := String(spec[0])
		var b := Button.new()
		b.text = String(spec[1])
		b.focus_mode = Control.FOCUS_NONE
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.add_theme_font_override("font", DccTheme.mono(0))
		b.add_theme_font_size_override("font_size", DccTheme.FS_MICRO)
		b.pressed.connect(func(): _show_phone_pane(key))
		row.add_child(b)
		_phone_pane_buttons[key] = b
	return wrap

## The switcher follows the work: `_select_family()`, `_select_collection()`
## and `_select_unassigned()` all move to SLOTS, and focusing a slot moves to
## SLOT. Calling it with the pane already showing is a no-op except for the
## chip states, which is what makes it safe to call from every one of those.
func _show_phone_pane(pane: String) -> void:
	if not _phone:
		return
	_phone_pane = pane
	_phone_rail.visible = pane == "families"
	_phone_grid.visible = pane == "slots"
	_phone_inspector.visible = pane == "slot"
	for key in _phone_pane_buttons:
		var b: Button = _phone_pane_buttons[key]
		var on: bool = key == pane
		b.add_theme_stylebox_override("normal",
			DccTheme.flat(DccTheme.c("accent_wash")) if on else DccTheme.empty())
		b.add_theme_color_override("font_color",
			DccTheme.c("accent") if on else DccTheme.c("text_dim"))
	if _phone_title != null:
		var trail: Label = _phone_title.get_parent().get_child(1) as Label
		if trail != null:
			var fam := _family_by_key(_current_family)
			trail.text = {
				"families": "%d families · collections" % FAMILIES.size(),
				"slots": String(fam.get("title", "slots")) if not fam.is_empty() else "slots",
				"slot": _focused_uid if _focused_uid != "" else "no slot selected",
			}.get(pane, "")

## `⧉ ASSET LIBRARY · map hidden while open │ search · sort · slicer · select
## … Apply to map · Export pack .zip · Close ✕` -- the canvas's own order and
## its own chip treatment, not stock buttons.
func _build_window_bar() -> Control:
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"bottom": 1}))
	if not _phone:
		wrap.custom_minimum_size.y = H_BAR
	var pad := _pad(wrap, 16, 0, 16, 0)
	## PH-07. One row of a 340 px search well plus six chips asks for ~880 px of
	## minimum width; a phone has 393 dp, and a `BoxContainer` handed more
	## minimum than it has **overlaps** rather than clipping. Two rows, the
	## second an `HFlowContainer` so the chips wrap onto a third by themselves
	## rather than by a count hard-coded here.
	## `Container`, not `BoxContainer`: an `HFlowContainer` is a `Container`
	## directly, not a box, so the two branches below have no closer common
	## ancestor than this.
	var row: Container
	if _phone:
		pad.add_theme_constant_override("margin_top", 6)
		pad.add_theme_constant_override("margin_bottom", 6)
		var col := VBoxContainer.new()
		col.add_theme_constant_override("separation", 6)
		pad.add_child(col)
		row = HFlowContainer.new()
		row.add_theme_constant_override("h_separation", 6)
		row.add_theme_constant_override("v_separation", 6)
		var search_phone := LineEdit.new()
		search_phone.placeholder_text = "Search name · type · tag · file…"
		search_phone.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		_well(search_phone)
		search_phone.text_changed.connect(func(t: String): _search_text = t; _refresh_grid())
		col.add_child(search_phone)
		col.add_child(row)
	else:
		row = HBoxContainer.new()
		row.add_theme_constant_override("separation", 14)
		row.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		pad.add_child(row)

		var title_label := DccTheme.mono_label("⧉ ASSET LIBRARY", "accent", DccTheme.FS_SMALL, 1)
		title_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		row.add_child(title_label)
		var sub := DccTheme.label("map hidden while open", "text_ghost", DccTheme.FS_SMALL)
		sub.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		row.add_child(sub)

		var divider := ColorRect.new()
		divider.color = DccTheme.c("line")
		divider.custom_minimum_size = Vector2(1, 16)
		divider.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		row.add_child(divider)

		var search := LineEdit.new()
		search.placeholder_text = "Search name · type · category · tag · file…"
		search.custom_minimum_size.x = 340   ## canvas: `flex:1;max-width:340px`
		_well(search)
		search.text_changed.connect(func(t: String): _search_text = t; _refresh_grid())
		row.add_child(search)

	## The canvas draws this as `Sort: slot order ⌄` -- a chip with a caret.
	## An OptionButton *is* that once its stock slab is replaced, and it keeps
	## the live `_sort_mode` binding rather than reimplementing a popup.
	_sort_button = OptionButton.new()
	_sort_button.add_item("Sort: slot order")
	_sort_button.add_item("Sort: name")
	_sort_button.focus_mode = Control.FOCUS_NONE
	_sort_button.add_theme_font_override("font", DccTheme.mono(0))
	_sort_button.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	_sort_button.add_theme_color_override("font_color", DccTheme.c("text"))
	_sort_button.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	for sb_name in ["normal", "pressed", "focus"]:
		_sort_button.add_theme_stylebox_override(sb_name, _box("line", "", 9, 4))
	_sort_button.add_theme_stylebox_override("hover", _box("line", "line_soft", 9, 4))
	_sort_button.item_selected.connect(func(i: int): _sort_mode = i; _refresh_grid())
	row.add_child(_sort_button)

	## PH-07: the same four chips, named shorter. Each label is its own minimum
	## width, and at 393 dp four full-length ones wrap onto three rows -- a
	## quarter of the screen spent on a toolbar. The short forms are the design
	## canvas's own vocabulary elsewhere (`Slice`, `Apply`, `.zip`), so this
	## abbreviates rather than renaming anything.
	_chip(row, ("%s Slicer…" if _phone else "%s Sprite sheet…") % DccIcons.SYMBOLS["panels"],
		func(): _open_slicer())

	_select_mode_btn = _chip(row, "", func(): _toggle_select_mode(), true)
	_select_mode_btn.tooltip_text = "Batch selection driving Tag/Collect/Rename/Duplicate/Delete in the grid header."

	## An expanding spacer is what pushes the three right-hand chips to the far
	## end of a row. An `HFlowContainer` has no far end -- it wraps -- so on a
	## phone the spacer would simply be one more item taking a whole line.
	if not _phone:
		row.add_child(DccTheme.spacer())

	_apply_btn = _chip(row, "Apply" if _phone else "Apply to map",
		func(): _on_apply_to_map(), false, 10, 4)
	_export_btn = _chip(row, "Export .zip" if _phone else "Export pack .zip",
		func(): _on_export_pack(), true, 10, 4)
	## Visual sweep (2026-08-20) caught the slicer modal left stranded on top of
	## the whole app -- `_slicer` is a child `Window` of this dialog, and a child
	## `Window`'s visibility is independent of its parent's, so closing the
	## library while the slicer was open used to leave it floating over every
	## surface opened afterward. Closing this window always closes the slicer.
	var close_chip := _chip(row, DccIcons.SYMBOLS["cross"] if _phone \
			else "Close %s" % DccIcons.SYMBOLS["cross"],
		func(): _close_slicer(); hide(), false, 10, 4)
	close_chip.add_theme_color_override("font_color", DccTheme.c("text_dim"))

	_update_select_count()
	return wrap

func _toggle_select_mode() -> void:
	_select_mode = not _select_mode
	_update_select_count()

# -- family rail --------------------------------------------------------------

func _build_family_rail() -> Control:
	var wrap := PanelContainer.new()
	## PH-07: a fixed 266 px column is 68% of a phone's 393 dp, and the pane it
	## is one of three of is full width there anyway.
	if _phone:
		wrap.size_flags_vertical = Control.SIZE_EXPAND_FILL
	else:
		wrap.custom_minimum_size.x = W_RAIL
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"right": 1}))
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	wrap.add_child(col)

	var band := _band(col, 14, 9)
	var head := DccTheme.mono_label("FAMILIES", "text_dim", DccTheme.FS_MICRO, 2, true)
	head.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	head.tooltip_text = FAMILIES_NOTE
	head.mouse_filter = Control.MOUSE_FILTER_STOP
	band.add_child(head)
	_rail_count_label = DccTheme.mono_label("%d" % FAMILIES.size(), "text_ghost", DccTheme.FS_MICRO)
	_rail_count_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	_rail_count_label.tooltip_text = FAMILIES_NOTE
	_rail_count_label.mouse_filter = Control.MOUSE_FILTER_STOP
	band.add_child(_rail_count_label)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(scroll)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 0)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(body)

	var by_group: Dictionary = {}
	for fam in FAMILIES:
		var g := String(fam["group"])
		if not by_group.has(g):
			by_group[g] = []
		(by_group[g] as Array).append(fam)

	var first := true
	for g in GROUP_ORDER:
		var gp := _pad(body, 14, 7 if first else 10, 14, 4)
		gp.add_child(DccTheme.mono_label(String(g).to_upper(), "text_ghost", DccTheme.FS_MICRO, 1))
		first = false
		for f in by_group.get(g, []):
			_rail_row(body, f)

	## AS-12's Collections rail: `Family` above is the mockup's own fixed
	## eight; collections are a live, unbounded, user-created set
	## (`as_batch_collect`/`as_collections`), so this section is rebuilt
	## in place (`_refresh_collections_rail`) rather than built once here.
	body.add_child(DccTheme.rule())
	var cgp := _pad(body, 14, 10, 14, 4)
	cgp.add_child(DccTheme.mono_label("COLLECTIONS", "text_ghost", DccTheme.FS_MICRO, 1))
	_build_unassigned_row(body)
	_collections_rail_body = VBoxContainer.new()
	_collections_rail_body.add_theme_constant_override("separation", 0)
	body.add_child(_collections_rail_body)
	_refresh_collections_rail()
	## After `_unassigned_row` exists, so its count populates on first build too.
	_refresh_rail_counts()

	col.add_child(DccTheme.rule())
	var foot_pad := _pad(col, 14, 9, 14, 9)
	var foot := HBoxContainer.new()
	foot.add_theme_constant_override("separation", 6)
	foot_pad.add_child(foot)
	_import_btn = _chip(foot, "Import image…", func(): _on_import_image(), false, 0, 5)
	_import_btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_import_btn.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	_refresh_import_button()
	var import_pack := _chip(foot, "Import pack…", func(): _host.open_asset_pack_picker(),
		false, 0, 5)
	import_pack.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	import_pack.add_theme_font_size_override("font_size", DccTheme.FS_TINY)

	return wrap

## The canvas's rail row: a 26 px code column, the family name, and the
## fill/capacity count -- three aligned columns, not one concatenated string.
## A `Button` hosts them so hover/press/keyboard behaviour is Godot's rather
## than hand-rolled on a panel.
func _rail_row(parent: Control, fam: Dictionary) -> void:
	var key := String(fam["key"])
	var btn := Button.new()
	## Deliberately *not* `flat` -- a flat Button draws no stylebox at all, so
	## the canvas's `background:rgba(224,163,74,.09)` on the selected row (and
	## the hover wash) simply never appeared. `normal` is an empty box instead,
	## which is what "flat" was reaching for.
	btn.focus_mode = Control.FOCUS_NONE
	btn.custom_minimum_size.y = 24
	btn.add_theme_stylebox_override("normal", DccTheme.empty())
	btn.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	btn.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("accent_wash")))
	btn.pressed.connect(_select_family.bind(key))

	var row := HBoxContainer.new()
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_theme_constant_override("separation", 9)
	row.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	row.offset_left = 14
	row.offset_right = -14
	btn.add_child(row)

	var code := DccTheme.mono_label(String(fam["code"]), "text_ghost", DccTheme.FS_TINY)
	code.custom_minimum_size.x = 26
	code.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	code.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(code)
	var name_l := DccTheme.label(String(fam["title"]), "text", DccTheme.FS_SMALL)
	name_l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	name_l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	name_l.clip_text = true
	name_l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(name_l)
	var count := DccTheme.mono_label("", "text_faint", DccTheme.FS_TINY)
	count.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	count.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(count)

	_rail_buttons[key] = {"button": btn, "code": code, "name": name_l, "count": count}
	parent.add_child(btn)

## Real fill counts (AS-08) on the rail itself, not just the grid --
## `as_family_slots` once per family, cheap enough to run on every
## `_refresh_pack_status()` (window open, and after `world_loaded`). The canvas
## paints an incomplete family's count in accent and a complete one's quiet.
func _refresh_rail_counts() -> void:
	var total_filled := 0
	var total_slots := 0
	for fam in FAMILIES:
		var key := String(fam["key"])
		var parts: Dictionary = _rail_buttons.get(key, {})
		if parts.is_empty():
			continue
		var slots: Array = _bridge.as_family_slots(key)
		var filled := 0
		for s in slots:
			if bool(s.get("filled", false)):
				filled += 1
		var capacity: int = slots.size() if bool(fam.get("custom", false)) \
			else (fam["slots"] as Array).size()
		total_filled += filled
		total_slots += capacity
		var count_label: Label = parts["count"]
		if bool(fam.get("custom", false)) and capacity == 0:
			count_label.text = "open"
			count_label.add_theme_color_override("font_color", DccTheme.c("text_ghost"))
		else:
			count_label.text = "%d/%d" % [filled, capacity]
			count_label.add_theme_color_override("font_color",
				DccTheme.c("text_faint") if filled >= capacity else DccTheme.c("accent"))
	## AS-12: the same `custom` pull above already has every unassigned slot in
	## it once `key == "custom"` runs, but `_refresh_unassigned_count()` is
	## also called on its own from `_select_family`/`_select_collection`-free
	## paths -- one extra `as_family_slots("custom")` per refresh is cheap
	## next to a PNG import or a batch op, so it isn't threaded through the
	## loop above just to save it.
	var unassigned := _refresh_unassigned_count()
	if _status_pack != null:
		var info: Dictionary = _bridge.as_pack_info()
		var pack_name := String(info.get("name", ""))
		_status_pack.text = "%s · %d / %d slots · %d item%s%s" % [
			pack_name if pack_name != "" else "unnamed pack",
			total_filled, total_slots, int(info.get("total_items", 0)),
			"" if int(info.get("total_items", 0)) == 1 else "s",
			" · %d unassigned" % unassigned if unassigned > 0 else ""]

## Rebuilds the Collections rail from `as_collections()` -- unlike `FAMILIES`
## (a fixed compile-time list), collections are created/emptied at runtime by
## `as_batch_collect`/drag-and-drop, so this section is torn down and rebuilt
## rather than refreshed in place. Called on window open/`world_loaded`
## (`_refresh_pack_status`) and after anything that can change membership.
##
## Calls `_bridge.world_gen.as_collections()` directly rather than through a
## new `EngineBridge` wrapper -- every other `as_*` call in this file goes
## through one (`as_family_slots`/`as_slot_summary`/etc.), but `engine_bridge.gd`
## is a concurrently-edited file this pass, and `bridge.world_gen.<method>()`
## is an already-established escape hatch elsewhere (`app.gd`, `journey_
## planner_view.gd`, `new_world_dialog.gd` all do the same). `has_method`
## guards it the same defensive way `new_world_dialog.gd` does, so a binary
## built before this pass's `as_collections` addition degrades to "no
## collections yet" instead of a script error.
func _refresh_collections_rail() -> void:
	if _collections_rail_body == null:
		return
	for c in _collections_rail_body.get_children():
		_collections_rail_body.remove_child(c)
		c.queue_free()
	_collection_buttons.clear()
	var colls: Array = _bridge.world_gen.as_collections() \
		if _bridge.world_gen != null and _bridge.world_gen.has_method("as_collections") else []
	if colls.is_empty():
		var note := DccTheme.mono_label("none yet -- Collect… or drag tiles here",
			"text_faint", DccTheme.FS_TINY)
		note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		var np := _pad(_collections_rail_body, 14, 4, 14, 6)
		np.add_child(note)
		return
	for c in colls:
		_collection_row(_collections_rail_body, String(c["name"]), (c["uids"] as PackedStringArray).size())
	_phone_refit()   ## PH-07 -- see `_phone_refit()`.

## Same three-column grammar `_rail_row` draws for a family (code · name ·
## count), minus the code column -- collections have no engine-assigned code,
## unlike a family's `TX`/`BI`/etc. `CollectionRow`, not `Button.new()`,
## because this row is also a real drop target (its own class comment).
func _collection_row(parent: Control, coll_name: String, count: int) -> void:
	var btn := CollectionRow.new()
	btn.owner_window = self
	btn.coll_name = coll_name
	btn.focus_mode = Control.FOCUS_NONE
	btn.custom_minimum_size.y = 24
	btn.add_theme_stylebox_override("normal", DccTheme.empty())
	btn.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	btn.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("accent_wash")))
	btn.tooltip_text = "Drag asset tiles here to add them to \"%s\"." % coll_name
	btn.pressed.connect(_select_collection.bind(coll_name))

	var row := HBoxContainer.new()
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_theme_constant_override("separation", 9)
	row.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	row.offset_left = 14
	row.offset_right = -14
	btn.add_child(row)

	var name_l := DccTheme.label(coll_name, "text", DccTheme.FS_SMALL)
	name_l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	name_l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	name_l.clip_text = true
	name_l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(name_l)
	var count_l := DccTheme.mono_label("%d" % count, "text_faint", DccTheme.FS_TINY)
	count_l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	count_l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(count_l)

	_collection_buttons[coll_name] = {"button": btn, "name": name_l, "count": count_l}
	parent.add_child(btn)

## AS-12's pinned "Unassigned imports" row -- same three-column grammar as
## `_collection_row`, minus the drag-drop target: there is no engine primitive
## to *move* an item from a real slot into this bucket (only into/out of a
## collection, `as_batch_collect`), so this row is browse-only, matching the
## register's own honest scope ("a holding area", not a reassignment tool).
func _build_unassigned_row(parent: Control) -> void:
	var btn := Button.new()
	btn.focus_mode = Control.FOCUS_NONE
	btn.custom_minimum_size.y = 24
	btn.add_theme_stylebox_override("normal", DccTheme.empty())
	btn.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	btn.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("accent_wash")))
	btn.tooltip_text = "Imports made with no slot focused land here (as_add_custom_slot, set \"%s\"). Drag a tile onto a Collections row, or Rename… it, to organise it from here." % UNASSIGNED_SET
	btn.pressed.connect(_select_unassigned)

	var row := HBoxContainer.new()
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_theme_constant_override("separation", 9)
	row.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	row.offset_left = 14
	row.offset_right = -14
	btn.add_child(row)

	var name_l := DccTheme.label("Unassigned imports", "text", DccTheme.FS_SMALL)
	name_l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	name_l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	name_l.clip_text = true
	name_l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(name_l)
	var count_l := DccTheme.mono_label("0", "text_faint", DccTheme.FS_TINY)
	count_l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	count_l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(count_l)

	_unassigned_row = {"button": btn, "name": name_l, "count": count_l}
	parent.add_child(btn)

## Live count of custom slots sitting in `UNASSIGNED_SET` -- folded into
## `_refresh_rail_counts()` (called after every import/batch op already)
## rather than given its own call site.
func _refresh_unassigned_count() -> int:
	var n := 0
	for s in _bridge.as_family_slots("custom"):
		if String(s.get("set", "")) == UNASSIGNED_SET:
			n += 1
	if not _unassigned_row.is_empty():
		var count_l: Label = _unassigned_row["count"]
		count_l.text = "%d" % n
		count_l.add_theme_color_override("font_color",
			DccTheme.c("accent") if n > 0 else DccTheme.c("text_faint"))
	return n

func _highlight_unassigned_row(on: bool) -> void:
	if _unassigned_row.is_empty():
		return
	var btn: Button = _unassigned_row["button"]
	var name_l: Label = _unassigned_row["name"]
	btn.add_theme_stylebox_override("normal",
		DccTheme.flat(DccTheme.c("accent_wash")) if on else DccTheme.empty())
	name_l.add_theme_color_override("font_color",
		DccTheme.c("text_bright") if on else DccTheme.c("text"))

## Selects the Unassigned imports bucket the same way `_select_family`/
## `_select_collection` select their own rail entries.
func _select_unassigned() -> void:
	_current_unassigned = true
	_current_collection = ""
	_current_family = ""
	_highlight_collection_row("")
	_highlight_unassigned_row(true)
	for k in _rail_buttons:
		var parts: Dictionary = _rail_buttons[k]
		(parts["button"] as Button).add_theme_stylebox_override("normal", DccTheme.empty())
		(parts["code"] as Label).add_theme_color_override("font_color", DccTheme.c("text_ghost"))
		(parts["name"] as Label).add_theme_color_override("font_color", DccTheme.c("text"))
	_selected.clear()
	_last_index = -1
	_focused_uid = ""
	_preview_index = 0
	_refresh_grid()
	_refresh_inspector()
	_refresh_import_button()
	_show_phone_pane("slots")   ## PH-07, as `_select_family()`.

## AS-12's unassigned-mode grid: entries are exactly the custom slots
## `_refresh_unassigned_count` counts, so the two can never disagree.
func _refresh_grid_unassigned() -> void:
	_slot_state.clear()
	var entries: Array = []
	for s in _bridge.as_family_slots("custom"):
		if String(s.get("set", "")) != UNASSIGNED_SET:
			continue
		var uid := String(s["uid"])
		_slot_state[uid] = s
		entries.append({
			"uid": uid,
			"id": String(s["id"]),
			"name": String(s["name"]),
			"code": "UN-%02d" % [entries.size() + 1],
		})
	_slot_order = entries
	for entry in entries:
		_grid.add_child(_build_cell(entry))
	_grid_header.text = "UNASSIGNED IMPORTS · %d ITEM%s" % [entries.size(), "" if entries.size() == 1 else "S"]
	_refresh_selection_visuals()

## Selects a collection the same way `_select_family` selects a family --
## clears the other rail's highlight, resets the grid selection, and switches
## `_refresh_grid` into collection mode.
func _select_collection(coll_name: String) -> void:
	_current_collection = coll_name
	_current_family = ""
	_current_unassigned = false
	_highlight_unassigned_row(false)
	for k in _rail_buttons:
		var parts: Dictionary = _rail_buttons[k]
		(parts["button"] as Button).add_theme_stylebox_override("normal", DccTheme.empty())
		(parts["code"] as Label).add_theme_color_override("font_color", DccTheme.c("text_ghost"))
		(parts["name"] as Label).add_theme_color_override("font_color", DccTheme.c("text"))
	_highlight_collection_row(coll_name)
	_selected.clear()
	_last_index = -1
	_focused_uid = ""
	_preview_index = 0
	_refresh_grid()
	_refresh_inspector()
	_refresh_import_button()
	_show_phone_pane("slots")   ## PH-07, as `_select_family()`.

func _highlight_collection_row(coll_name: String) -> void:
	for k in _collection_buttons:
		var parts: Dictionary = _collection_buttons[k]
		var on: bool = k == coll_name
		(parts["button"] as Button).add_theme_stylebox_override("normal",
			DccTheme.flat(DccTheme.c("accent_wash")) if on else DccTheme.empty())
		(parts["name"] as Label).add_theme_color_override("font_color",
			DccTheme.c("text_bright") if on else DccTheme.c("text"))

## `SlotCell._get_drag_data`'s own query: drag the whole current selection if
## this tile is part of one and it's a real multi-selection, otherwise just
## this one tile -- matches how the batch buttons already read "the current
## selection, or nothing" (`_selected_uids`).
func _drag_uids_for(uid: String) -> PackedStringArray:
	if _selected.size() > 1 and _selected.has(uid):
		var out := PackedStringArray()
		for u in _selected:
			out.append(String(u))
		return out
	return PackedStringArray([uid])

## `CollectionRow._drop_data`'s callback: real engine call, same one
## `_on_batch_collect`'s prompt uses, just skipping the prompt because the
## target collection is exactly the row the drag landed on.
func _on_drop_uids_on_collection(coll_name: String, uids: Array) -> void:
	if uids.is_empty():
		return
	var uid_arr := PackedStringArray()
	for u in uids:
		uid_arr.append(String(u))
	_bridge.as_batch_collect(uid_arr, coll_name)
	_dirty = true
	_host.set_status("hint",
		"added %d asset(s) to \"%s\" (drag-and-drop)" % [uid_arr.size(), coll_name], "accent")
	_refresh_collections_rail()
	_refresh_inspector()
	_refresh_status_line()

## "Import image…" targets whichever slot is focused in the grid; with none
## focused it still works (AS-12), landing the file in a fresh custom slot
## under `UNASSIGNED_SET` instead of doing nothing.
func _refresh_import_button() -> void:
	if _import_btn == null:
		return
	_import_btn.disabled = false
	if _focused_uid == "":
		_import_btn.tooltip_text = "No slot focused -- lands in Unassigned imports instead."
	else:
		_import_btn.tooltip_text = "Import a PNG into %s." % _focused_uid

## `replace_first` empties the slot's first variant once the new image is in --
## the inspector's Replace…, built out of `as_import_item` + `as_remove_item`
## rather than a binding that does not exist. Import order matters: the new
## bytes have to land successfully *before* anything is removed. Only ever
## called with a real `_focused_uid` (the inspector's Replace… chip is itself
## disabled while `item_count == 0`), so the AS-12 unassigned branch below is
## the plain "Import image…" path (`replace_first == false`) alone.
func _on_import_image(replace_first: bool = false) -> void:
	if replace_first and _focused_uid == "":
		return
	var target_uid := _focused_uid
	var into_unassigned := target_uid == ""
	var d := FileDialog.new()
	d.title = "Replace image" if replace_first else "Import image"
	d.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	d.access = FileDialog.ACCESS_FILESYSTEM
	d.add_filter("*.png ; PNG image")
	d.file_selected.connect(func(path: String):
		var bytes := FileAccess.get_file_as_bytes(path)
		var uid := target_uid
		if into_unassigned:
			## AS-12: no slot was focused, so this file gets a fresh custom
			## slot of its own under the reserved "Unassigned imports" set --
			## `as_add_custom_slot` is the real engine call that bucket sits
			## on, same as every other custom slot.
			var made: Dictionary = _bridge.as_add_custom_slot(path.get_file().get_basename(), UNASSIGNED_SET)
			uid = String(made.get("uid", ""))
		var result: Dictionary = _bridge.as_import_item(uid, path.get_file(), bytes)
		if bool(result.get("ok", false)):
			if replace_first:
				_bridge.as_remove_item(uid, 0)
			_dirty = true
			_host.set_status("hint", "imported %s%s" % [
				path.get_file(), " → Unassigned imports" if into_unassigned else ""], "accent")
			_preview_index = 0
			if into_unassigned:
				_current_unassigned = true
				_current_collection = ""
				_current_family = ""
				_highlight_collection_row("")
				_highlight_unassigned_row(true)
				for k in _rail_buttons:
					var parts: Dictionary = _rail_buttons[k]
					(parts["button"] as Button).add_theme_stylebox_override("normal", DccTheme.empty())
					(parts["code"] as Label).add_theme_color_override("font_color", DccTheme.c("text_ghost"))
					(parts["name"] as Label).add_theme_color_override("font_color", DccTheme.c("text"))
				_focused_uid = uid
			_refresh_grid()
			_refresh_inspector()
			_refresh_rail_counts()
			_refresh_status_line()
		else:
			_host.set_status("hint", "import failed — %s" % String(result.get("error", "unknown error")), "warn")
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered_ratio(0.6)

## AS-07: the Scale slider / Pan X / Pan Y spinboxes writing live, straight
## through `as_set_item_transform`. Only the preview repaints here -- the
## reference's own `alScale.oninput` repaints its canvas (`this.paint()`) and
## leaves the grid card stale until the next full render, so this matches
## rather than rebuilding `_refresh_grid()` on every drag tick.
func _on_insp_transform_changed() -> void:
	if _insp_syncing or _focused_uid == "":
		return
	var ok: bool = _bridge.as_set_item_transform(_focused_uid, _preview_index,
		_insp_scale.value / 100.0, _insp_pan_x.value, _insp_pan_y.value)
	if not ok:
		return
	_dirty = true
	_insp_scale_readout.text = "%d%%" % int(roundf(_insp_scale.value))
	var preview_png: PackedByteArray = _bridge.as_thumbnail_png(_focused_uid, _preview_index, 256)
	if preview_png.size() > 0:
		var pimg := Image.new()
		if pimg.load_png_from_buffer(preview_png) == OK:
			_insp_preview.thumb = ImageTexture.create_from_image(pimg)
			_insp_preview.queue_redraw()

## AS-07's Fit/Reset buttons -- `as_reset_item_transform` does the actual
## `defaultTransform()`/`fitToBottom` arithmetic (reference `alFit`/`alReset`,
## line 27347-27348) so this stays "no numbers in GDScript"; both are
## discrete clicks, so (unlike the live slider) a full re-render is cheap and
## matches the reference's own `this.render(); AssetBrowserUI.buildGrid();`.
func _on_insp_fit_or_reset(fit: bool) -> void:
	if _focused_uid == "":
		return
	var result: Dictionary = _bridge.as_reset_item_transform(_focused_uid, _preview_index, fit)
	if not bool(result.get("ok", false)):
		return
	_dirty = true
	_refresh_grid()
	_refresh_inspector()

# -- slot grid ----------------------------------------------------------------

func _build_slot_grid() -> Control:
	var wrap := PanelContainer.new()
	wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## PH-07: in the phone `VBoxContainer` the three panes stack, so the axis
	## that has to expand is the vertical one -- without this the pane collapses
	## to its own minimum (measured: a 12 px scroll viewport) and the grid is a
	## scrollbar with nothing under it.
	if _phone:
		wrap.size_flags_vertical = Control.SIZE_EXPAND_FILL
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"right": 1}))
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	wrap.add_child(col)

	var band := _band(col, 16, 14)
	_grid_header = DccTheme.mono_label("", "text_dim", DccTheme.FS_MICRO, 2, true)
	_grid_header.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_grid_header.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	band.add_child(_grid_header)
	## PH-07: `TX · SPLAT CHANNELS · 0 OF 7 FILLED` is a `Label`, and
	## `phone_fit()`'s ellipsis pass only reaches `Button`s -- so on a phone this
	## one heading contributed its full natural width to a 393 dp column.
	_grid_header.clip_text = _phone
	_select_count_label = DccTheme.mono_label("0 SELECTED", "accent", DccTheme.FS_MICRO, 1, true)
	_select_count_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	band.add_child(_select_count_label)
	## The canvas folds the batch verbs into this band as quiet text rather
	## than giving them a row of filled slabs of their own.
	##
	## PH-07: five verbs plus the heading plus the count is a 544 dp row, and a
	## `Window` cannot be narrower than its content's minimum -- measured, this
	## one row alone was what pushed the whole window 151 dp wider than the
	## screen and took `Apply to map` and the SLOT tab off the right edge with
	## it. On a phone the verbs get their own wrapping row under the band; they
	## are still the same five buttons, at the same 44 dp the tap floor gives
	## them, and `_refresh_batch_buttons()` still greys them with no selection.
	var verbs: Container
	if _phone:
		var verbs_flow := HFlowContainer.new()
		verbs_flow.add_theme_constant_override("h_separation", 4)
		verbs_flow.add_theme_constant_override("v_separation", 2)
		_pad(col, 12, 0, 12, 4).add_child(verbs_flow)
		verbs = verbs_flow
	else:
		var verbs_row := HBoxContainer.new()
		verbs_row.add_theme_constant_override("separation", 2)
		verbs_row.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		band.add_child(verbs_row)
		verbs = verbs_row
	_batch_buttons["tag"] = _text_button(verbs, "Tag…", func(): _on_batch_tag())
	_batch_buttons["collect"] = _text_button(verbs, "Collect…", func(): _on_batch_collect())
	_batch_buttons["rename"] = _text_button(verbs, "Rename…", func(): _on_batch_rename())
	_batch_buttons["duplicate"] = _text_button(verbs, "Duplicate", func(): _on_batch_duplicate())
	_batch_buttons["delete"] = _text_button(verbs, "Delete", func(): _on_batch_delete())
	_refresh_batch_buttons()

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(scroll)
	var grid_pad := _pad(scroll, 16, 16, 16, 16)
	grid_pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_grid = GridContainer.new()
	## PH-07: six columns across 393 dp is a 55 dp tile, smaller than the tap
	## floor the tile itself has to clear -- and the caption strip under each
	## one carries a two-letter code plus a slot name. Two columns keeps the
	## default 76 dp art band square-ish and the caption readable.
	_grid.columns = PHONE_GRID_COLS if _phone else GRID_COLS
	_grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_grid.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
	_grid.add_theme_constant_override("h_separation", TILE_GAP)
	_grid.add_theme_constant_override("v_separation", TILE_GAP)
	grid_pad.add_child(_grid)

	col.add_child(DccTheme.rule())
	var foot_pad := _pad(col, 16, 8, 16, 8)
	var foot := HBoxContainer.new()
	foot.add_theme_constant_override("separation", 22)
	foot_pad.add_child(foot)
	## Drag a tile onto a Collections-rail row to add it (real,
	## `CollectionRow`/`SlotCell._get_drag_data`, AS-12). Drag-onto-a-SLOT
	## specifically -- i.e. dropping a file from outside Godot to fill it --
	## stays unwired: Godot's own drag-and-drop is two unrelated systems, and
	## OS-external file drops only ever reach `Window.files_dropped`, never a
	## Control's `_can_drop_data`/`_drop_data` (those two are in-app-drag-only,
	## which is exactly what tile-onto-collection uses). Said plainly rather
	## than drawn as if a slot accepted a file drop it structurally cannot.
	var drop_hint := DccTheme.mono_label("drag a tile onto a Collection to add it",
		"text_faint", DccTheme.FS_TINY)
	drop_hint.tooltip_text = "Real: drag one or more selected tiles onto a Collections-rail row (as_batch_collect). Dropping a file from outside Godot to fill a slot stays unwired -- OS file drops reach Window.files_dropped, not a Control's _can_drop_data/_drop_data, so a slot cannot be that kind of drop target; use Import image… for that."
	drop_hint.mouse_filter = Control.MOUSE_FILTER_STOP
	## PH-07: both foot hints describe pointer modifiers. `⇧-click ranges ·
	## Ctrl-click adds` has no touch equivalent at all, and the drop hint's
	## tooltip -- which is where its real disclosure lives -- is unreachable
	## without hover. Dropped on a phone rather than kept as two lines of prose
	## about gestures the device cannot make; the zoom slider beside them is
	## the only thing in this row a finger can use.
	if not _phone:
		foot.add_child(drop_hint)
		foot.add_child(DccTheme.mono_label("⇧-click ranges · Ctrl-click adds",
			"text_faint", DccTheme.FS_TINY))
	foot.add_child(DccTheme.spacer())
	var zoom_label := DccTheme.mono_label("zoom", "text_faint", DccTheme.FS_TINY)
	zoom_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	foot.add_child(zoom_label)
	var zoom := HSlider.new()
	zoom.min_value = 56
	zoom.max_value = 132
	zoom.step = 4
	zoom.value = _cell_px
	zoom.custom_minimum_size = Vector2(96, 14)
	zoom.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	zoom.focus_mode = Control.FOCUS_NONE
	_style_slider(zoom)
	zoom.value_changed.connect(func(v: float): _cell_px = v; _refresh_grid())
	foot.add_child(zoom)

	return wrap

func _build_status_line() -> Control:
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"top": 1}))
	## PH-07: two lines rather than one clipped one. Side by side the pair asks
	## for 401 dp of a 393 dp column, and clipping them both left the state half
	## reading `(` -- the width went to whichever expanded, and neither of them
	## is optional: one says whether the library is in sync, the other what pack
	## it is and how full.
	wrap.custom_minimum_size.y = H_STATUS * 2 if _phone else H_STATUS
	var pad := _pad(wrap, 16, 0, 16, 0)
	var row: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	row.add_theme_constant_override("separation", 22 if not _phone else 0)
	row.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	pad.add_child(row)
	_status_state = DccTheme.mono_label("", "text_faint", DccTheme.FS_TINY)
	_status_state.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	row.add_child(_status_state)
	_status_pack = DccTheme.mono_label("", "text_faint", DccTheme.FS_TINY)
	_status_pack.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	row.add_child(_status_pack)
	## Clipped as well as stacked: `phone_fit()`'s ellipsis pass reaches only
	## `Button`s, so a `Label` still reports its full natural width and a
	## `Window` cannot be narrower than its content's minimum.
	if _phone:
		_status_state.clip_text = true
		_status_pack.clip_text = true
	else:
		row.add_child(DccTheme.spacer())
	## PH-07: a phone has no Esc. Its way out is the Close chip in the window
	## bar and the Android back gesture (`DccShell::_notification`'s chain),
	## neither of which this hint names.
	if not _phone:
		var keys := DccTheme.mono_label("Esc close window", "text_faint", DccTheme.FS_TINY)
		keys.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		row.add_child(keys)
	_refresh_status_line()
	return wrap

func _refresh_status_line() -> void:
	if _status_state == null:
		return
	if _dirty:
		_status_state.text = "%s library edited — apply to map to use it" % DccIcons.SYMBOLS["on"]
		_status_state.add_theme_color_override("font_color", DccTheme.c("accent"))
	else:
		_status_state.text = "%s in sync with the map" % DccIcons.SYMBOLS["off"]
		_status_state.add_theme_color_override("font_color", DccTheme.c("text_faint"))

# ---------------------------------------------------------------------------
# Family / grid / selection
# ---------------------------------------------------------------------------

func _select_family(key: String) -> void:
	_current_family = key
	_current_collection = ""
	_current_unassigned = false
	_highlight_collection_row("")
	_highlight_unassigned_row(false)
	for k in _rail_buttons:
		var parts: Dictionary = _rail_buttons[k]
		var on: bool = k == key
		var b: Button = parts["button"]
		b.add_theme_stylebox_override("normal",
			DccTheme.flat(DccTheme.c("accent_wash")) if on else DccTheme.empty())
		(parts["code"] as Label).add_theme_color_override("font_color",
			DccTheme.c("accent") if on else DccTheme.c("text_ghost"))
		(parts["name"] as Label).add_theme_color_override("font_color",
			DccTheme.c("text_bright") if on else DccTheme.c("text"))
	_selected.clear()
	_last_index = -1
	_focused_uid = ""
	_preview_index = 0
	_refresh_grid()
	_refresh_inspector()
	_refresh_import_button()
	## PH-07: picking a family in the FAMILIES pane is a navigation, not a
	## setting -- its whole result is the grid, which on a phone is the next
	## pane over. Also covers `open(family_key)`, so opening the window from
	## `Assets ▸ Icon families ▸` lands on the slots that entry names rather
	## than on the rail it was picked from.
	_show_phone_pane("slots")

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

	if _current_unassigned:
		_refresh_grid_unassigned()
		return

	if _current_collection != "":
		_refresh_grid_collection()
		return

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
			var slot_name := String(s["name"])
			var code := "%s-%02d" % [String(fam["code"]), i + 1]
			var uid := String(s["uid"])
			if q != "" and slot_name.to_lower().find(q) < 0 and code.to_lower().find(q) < 0:
				continue
			entries.append({"uid": uid, "id": String(s["id"]), "name": slot_name, "code": code})
	else:
		var ids: Array = fam["slots"]
		for i in ids.size():
			var id := String(ids[i])
			var slot_name := _humanize(id)
			var code := "%s-%02d" % [String(fam["code"]), i + 1]
			var uid := "%s:%s" % [String(fam["key"]), id]
			if q != "" and id.to_lower().find(q) < 0 and slot_name.to_lower().find(q) < 0 \
					and code.to_lower().find(q) < 0:
				continue
			entries.append({"uid": uid, "id": id, "name": slot_name, "code": code})
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
	## The canvas's own header line: `P · PLACES · 10 OF 12 FILLED`. A search
	## that narrows the grid appends what it narrowed to, rather than changing
	## the line's shape.
	_grid_header.text = "%s · %s · %d OF %d FILLED" % [
		String(fam["code"]), String(fam["title"]).to_upper(), filled, total]
	if shown != total:
		_grid_header.text += " · %d SHOWN" % shown
	_refresh_selection_visuals()

## AS-12's collection-mode grid: entries come straight from `as_collections()`'s
## member uid list rather than a family's frozen slot ids. Which family a uid
## belongs to is read back per-uid off `as_slot_summary` (a collection can mix
## uids from several families, unlike the family view), and reuses `_build_cell`
## unchanged -- a collection-mode entry has the same `{uid,id,name,code}` shape
## a family-mode one does. No search/sort/family-scoping here: a collection is
## already a hand-picked set, matching AS-12's own scope (a browse row, not a
## second filtering layer on top of one).
func _refresh_grid_collection() -> void:
	var members := PackedStringArray()
	var colls: Array = _bridge.world_gen.as_collections() \
		if _bridge.world_gen != null and _bridge.world_gen.has_method("as_collections") else []
	for c in colls:
		if String(c["name"]) == _current_collection:
			members = c["uids"]
			break

	_slot_state.clear()
	var entries: Array = []
	for uid in members:
		var summary: Dictionary = _bridge.as_slot_summary(uid)
		## A stale membership referencing a since-removed custom slot --
		## `AssetCollections`'s own doc comment (library.rs) names exactly
		## this case as the one real way membership can outlive its slot.
		if not bool(summary.get("ok", false)):
			continue
		var fam := _family_by_key(String(summary.get("family", "")))
		var item_count := int(summary.get("item_count", 0))
		_slot_state[String(uid)] = {
			"filled": item_count > 0,
			"item_count": item_count,
			"has_dupe": summary.get("has_dupe", false),
		}
		entries.append({
			"uid": String(uid),
			"id": String(summary.get("id", "")),
			"name": String(summary.get("name", "")),
			"code": "%s-%02d" % [String(fam.get("code", "?")), entries.size() + 1],
		})
	_slot_order = entries

	for entry in entries:
		_grid.add_child(_build_cell(entry))

	_grid_header.text = "%s · %d ITEM%s" % [
		_current_collection.to_upper(), entries.size(), "" if entries.size() == 1 else "S"]
	_refresh_selection_visuals()

## The canvas's tile: one bordered box holding a 76 px art band and, inside the
## same border under a hairline, a `code · name` caption. The caption used to
## float outside the tile, which is why the grid read as a scatter of squares
## with text under them rather than as a contact sheet.
func _build_cell(entry: Dictionary) -> Control:
	var uid := String(entry["uid"])
	var state: Dictionary = _slot_state.get(uid, {})
	var filled := bool(state.get("filled", false))
	var count := int(state.get("item_count", 0))

	var tile := PanelContainer.new()
	tile.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	tile.add_theme_stylebox_override("panel", _tile_box(false))
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	tile.add_child(col)

	var cell := SlotCell.new()
	cell.custom_minimum_size.y = _cell_px
	cell.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cell.uid = uid
	cell.show_empty = not filled
	cell.variant_count = count
	cell.show_check = true
	cell.draw_border = false
	cell.draggable = true
	cell.owner_window = self
	if filled:
		var png: PackedByteArray = _bridge.as_thumbnail_png(uid, 0, 128)
		if png.size() > 0:
			var img := Image.new()
			if img.load_png_from_buffer(png) == OK:
				cell.thumb = ImageTexture.create_from_image(img)
	cell.gui_input.connect(_on_cell_input.bind(uid))
	col.add_child(cell)
	col.add_child(_rule_soft())

	var cap_pad := _pad(col, 6, 4, 6, 4)
	var cap := HBoxContainer.new()
	cap.add_theme_constant_override("separation", 6)
	cap_pad.add_child(cap)
	cap.add_child(DccTheme.mono_label(String(entry["code"]), "text_ghost", DccTheme.FS_MICRO))
	var dupe_mark := " %s" % DccIcons.SYMBOLS["warn_tri"] if bool(state.get("has_dupe", false)) else ""
	var name_l := DccTheme.label("%s%s" % [String(entry["name"]), dupe_mark],
		"text" if filled else "text_ghost", DccTheme.FS_TINY)
	name_l.clip_text = true
	name_l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cap.add_child(name_l)

	_cells[uid] = {"tile": tile, "cell": cell}
	return tile

## The canvas's selected tile is `border:1px solid accent` plus a 35%-accent
## outline one pixel out. A StyleBoxFlat shadow is the closest thing Godot has
## to `outline-offset`, and reads the same at this size -- but only over an
## opaque fill; on a transparent box the shadow shows *through* the tile and
## washes the caption strip accent.
static func _tile_box(selected: bool) -> StyleBoxFlat:
	var sb := DccTheme.outline("accent" if selected else "line", "bg")
	if selected:
		var halo := DccTheme.c("accent")
		halo.a = 0.35
		sb.shadow_color = halo
		sb.shadow_size = 2
	return sb

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
	_preview_index = 0
	_refresh_selection_visuals()
	_refresh_inspector()
	_refresh_import_button()
	## PH-07: on the desktop composition the inspector is the column beside the
	## grid and a tap on a tile fills it in place. On a phone it is the pane
	## *behind* the grid, so the same tap has to move there or the tap appears
	## to do nothing at all. Not while batch-selecting: there the tap means
	## "add to the selection", and the verbs that act on it live in the grid's
	## own header band.
	if _phone and not _select_mode:
		_show_phone_pane("slot")

## PH-07. The grid, the collections rail and the inspector are all rebuilt from
## fresh nodes on every refresh, and a node built after `setup()`'s one-shot
## pass has never been through it. `DccShell.phone_fit()` is idempotent by
## meta-flag, so re-walking the whole window costs one visit per already-sized
## control and touches only what is new -- which is why this can be a blunt
## "fit everything again" rather than three careful subtree calls that would
## each have to know what their caller just replaced.
## **Deferred**: the callers are rebuild functions with early returns in the
## middle of them, so a direct call at the top would fit nodes about to be
## freed and one at the bottom would be skipped on exactly the paths that
## return early. One deferred pass runs after the rebuild, however it ended.
func _phone_refit() -> void:
	if _phone and _host != null:
		_do_phone_refit.call_deferred()

func _do_phone_refit() -> void:
	if _phone and _host != null and is_instance_valid(self):
		_host.phone_fit(self, 1.0)

func _refresh_selection_visuals() -> void:
	_phone_refit()
	for uid in _cells:
		var parts: Dictionary = _cells[uid]
		var sel: bool = _selected.has(uid)
		var cell: SlotCell = parts["cell"]
		cell.selected = sel
		cell.queue_redraw()
		(parts["tile"] as PanelContainer).add_theme_stylebox_override("panel", _tile_box(sel))
	_update_select_count()

func _update_select_count() -> void:
	if _select_count_label != null:
		_select_count_label.text = "%d SELECTED" % _selected.size()
		_select_count_label.add_theme_color_override("font_color",
			DccTheme.c("accent") if not _selected.is_empty() else DccTheme.c("text_ghost"))
	if _select_mode_btn != null:
		_select_mode_btn.text = "%s Select · %d" % [
			DccIcons.SYMBOLS["checked"] if _select_mode else DccIcons.SYMBOLS["unchecked"],
			_selected.size()]
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
func _prompt_text(prompt_title: String, label_text: String, default_text: String,
		on_confirm: Callable) -> void:
	var d := ConfirmationDialog.new()
	d.title = prompt_title
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
		_dirty = true
		_host.set_status("hint", "tagged %d asset(s)" % int(result.get("tagged", 0)), "accent")
		_refresh_inspector()
		_refresh_status_line())

func _on_batch_collect() -> void:
	var uids := _selected_uids()
	if uids.is_empty():
		return
	_prompt_text("Collect %d asset(s)" % uids.size(), "Add to collection:", "Fantasy Pack", func(t: String):
		_bridge.as_batch_collect(uids, t)
		_dirty = true
		_host.set_status("hint", "added %d asset(s) to \"%s\"" % [uids.size(), t], "accent")
		_refresh_collections_rail()
		_refresh_inspector()
		_refresh_status_line())

func _on_batch_rename() -> void:
	var uids := _selected_uids()
	if uids.is_empty():
		return
	_prompt_text("Rename %d asset(s)" % uids.size(),
			"Rename pattern -- selected assets become \"Base_01\", \"Base_02\", …\n(custom slots are renamed; frozen slots rename their variants)", "Village",
			func(t: String):
		var result: Dictionary = _bridge.as_batch_rename(uids, t)
		_dirty = true
		_host.set_status("hint", "renamed %d asset(s)" % int(result.get("renamed", 0)), "accent")
		var remap: Dictionary = result.get("remap", {})
		if remap.has(_focused_uid):
			_focused_uid = String(remap[_focused_uid])
		_selected.clear()
		for old_uid in uids:
			var s := String(old_uid)
			_selected[String(remap.get(s, s))] = true
		_refresh_grid()
		_refresh_inspector()
		_refresh_status_line())

func _on_batch_duplicate() -> void:
	var uids := _selected_uids()
	if uids.is_empty():
		return
	var result: Dictionary = _bridge.as_batch_duplicate(uids)
	var made := int(result.get("made", 0))
	_dirty = _dirty or made > 0
	_host.set_status("hint",
		("duplicated %d asset(s) → Custom/Duplicates" % made) if made > 0 else "nothing to duplicate", "accent")
	_refresh_grid()
	_refresh_rail_counts()
	_refresh_status_line()

func _on_batch_delete() -> void:
	var uids := _selected_uids()
	if uids.is_empty():
		return
	var d := ConfirmationDialog.new()
	d.title = "Delete %d asset(s)?" % uids.size()
	d.dialog_text = "Delete images of %d selected asset(s)? This cannot be undone. (Custom slots are removed entirely; frozen slots are emptied, not removed.)" % uids.size()
	d.confirmed.connect(func():
		var result: Dictionary = _bridge.as_batch_delete(uids)
		_dirty = true
		_host.set_status("hint", "deleted %d asset(s)" % int(result.get("deleted", 0)), "accent")
		_selected.clear()
		_focused_uid = ""
		_refresh_grid()
		_refresh_inspector()
		_refresh_rail_counts()
		_refresh_collections_rail()
		_refresh_status_line()
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered()

## Delete / Backspace on the grid selection (`GUI_GAP_REGISTER.md` §31's last
## open item -- MN-09 recorded that "the Asset Library window has no key
## handling at all", which is why the Assets ▸ Batch ▸ Delete row lost the
## accelerator glyph it used to print).
##
## **Routed through `_on_batch_delete` rather than deleting.** This is a
## destructive batch operation with no undo, so the key does exactly what the
## button does -- it raises the same confirmation, with the same count and the
## same "custom slots are removed entirely, frozen slots are emptied" wording.
## A second, key-only prompt would be a second place for that wording to drift.
##
## Two guards, both of them the ones `app.gd`'s own `_unhandled_key_input`
## found it needed:
##
##   - **A focused text field wins.** Backspace inside a `LineEdit` means
##     "delete a character", never "delete eleven assets", and
##     `_unhandled_key_input` still fires for a focused text field on some
##     platforms. That covers the rename prompt, the tag field and the
##     pack-metadata fields, none of which is ever a delete.
##   - **An empty selection says so.** Returning silently would make the key
##     look dead on exactly the press that teaches a user it exists.
##
## `_unhandled_key_input` on this node rather than on `DccApp`: this is a
## `Window`, so it is its own `Viewport`, and the key arrives here only while
## the library has focus -- which is also why no "is the library open?" check
## is needed, and why the slicer modal (its own `Window`) does not steal it.
func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey and event.pressed):
		return
	if event.keycode != KEY_DELETE and event.keycode != KEY_BACKSPACE:
		return
	var typing := get_viewport().gui_get_focus_owner()
	if typing is LineEdit or typing is TextEdit or typing is SpinBox:
		return
	get_viewport().set_input_as_handled()
	if _selected.is_empty():
		_host.set_status("hint", "select at least one slot to delete", "text_ghost")
		return
	_on_batch_delete()

# ---------------------------------------------------------------------------
# Slot inspector
#
# Built once, refreshed in place. The previous version rebuilt every child on
# every selection change, which is why the pack-metadata fields needed a
# has_focus() guard to survive being typed into.
# ---------------------------------------------------------------------------

func _build_inspector() -> Control:
	var wrap := PanelContainer.new()
	if _phone:
		wrap.size_flags_vertical = Control.SIZE_EXPAND_FILL
	else:
		wrap.custom_minimum_size.x = W_INSPECTOR
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg"))
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	wrap.add_child(col)

	var band := _band(col, 14)
	_insp_head = DccTheme.mono_label("", "text_dim", DccTheme.FS_MICRO, 2, true)
	_insp_head.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	band.add_child(_insp_head)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(scroll)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 0)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(body)

	var empty_pad := _pad(body, 14, 14, 14, 14)
	_insp_empty = DccTheme.label("Select a slot to inspect it.", "text_ghost", DccTheme.FS_SMALL)
	_insp_empty.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	## An autowrapping Label with no minimum width reports a minimum *height*
	## computed at one character per line, which is how the slicer modal ended
	## up 1700 px tall on its first run. Every autowrap label in this file
	## carries an explicit width for that reason -- 52 rather than 28 off the
	## column, because a `ScrollContainer` folds its own scrollbar width into
	## the minimum it hands upwards, and 28 pushed the whole inspector 24 px
	## past the canvas's 330.
	_insp_empty.custom_minimum_size.x = W_INSPECTOR - 52
	empty_pad.add_child(_insp_empty)

	_insp_detail = VBoxContainer.new()
	_insp_detail.add_theme_constant_override("separation", 0)
	_insp_detail.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(_insp_detail)

	# -- preview + file readout (canvas: 150 px on a 12 px checkerboard) -------
	var prev_pad := _pad(_insp_detail, 14, 14, 14, 10)
	var prev_col := VBoxContainer.new()
	prev_col.add_theme_constant_override("separation", 7)
	prev_pad.add_child(prev_col)
	_insp_preview = SlotCell.new()
	_insp_preview.custom_minimum_size.y = 150
	_insp_preview.checker_px = 12.0
	_insp_preview.checker_under_art = true
	_insp_preview.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	prev_col.add_child(_insp_preview)
	_insp_file = DccTheme.mono_label("", "text_faint", DccTheme.FS_TINY)
	_insp_file.clip_text = true
	prev_col.add_child(_insp_file)

	# -- Scale / Fit / Reset / Replace / +Variant / bg / anchor / tags ---------
	var rows_pad := _pad(_insp_detail, 14, 0, 14, 12)
	var rows := VBoxContainer.new()
	rows.add_theme_constant_override("separation", 9)
	rows_pad.add_child(rows)

	## Reference bounds (`#alScale`, line 27277): 5..600 -> 0.05x..6.00x.
	var scale_row := _insp_row(rows, "Scale")
	_insp_scale = HSlider.new()
	_insp_scale.min_value = 5
	_insp_scale.max_value = 600
	_insp_scale.step = 1
	_insp_scale.value = 100
	_insp_scale.focus_mode = Control.FOCUS_NONE
	_insp_scale.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_insp_scale.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	_insp_scale.custom_minimum_size.y = 14
	_style_slider(_insp_scale)
	_insp_scale.value_changed.connect(func(_v): _on_insp_transform_changed())
	scale_row.add_child(_insp_scale)
	_insp_scale_readout = DccTheme.mono_label("—", "text", DccTheme.FS_TINY)
	_insp_scale_readout.custom_minimum_size.x = 38
	_insp_scale_readout.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	_insp_scale_readout.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	scale_row.add_child(_insp_scale_readout)

	## `ImageEditor`'s drag-to-pan (line 27231-27237) has no headless-friendly
	## equivalent in this shell's control set, so pan is exposed as two direct
	## SpinBoxes instead -- same value (`item.t.panX`/`panY`, output-px units),
	## a smaller control than a drag surface and no screen-space conversion to
	## get subtly wrong.
	var pan_row := _insp_row(rows, "Pan")
	var pan_box := HBoxContainer.new()
	pan_box.add_theme_constant_override("separation", 6)
	pan_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pan_row.add_child(pan_box)
	_insp_pan_x = SpinBox.new()
	_insp_pan_y = SpinBox.new()
	for sb in [_insp_pan_x, _insp_pan_y]:
		sb.min_value = -2048
		sb.max_value = 2048
		sb.step = 1
		sb.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		_well(sb.get_line_edit(), 8, 3)
		sb.value_changed.connect(func(_v): _on_insp_transform_changed())
		pan_box.add_child(sb)

	var btn_row := HBoxContainer.new()
	btn_row.add_theme_constant_override("separation", 5)
	rows.add_child(btn_row)
	_insp_fit_btn = _chip(btn_row, "Fit", func(): _on_insp_fit_or_reset(true))
	_insp_fit_btn.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	_insp_fit_btn.tooltip_text = "Reset the transform, then re-fit to the slot's anchor (fitToBottom for base-anchored families)."
	_insp_reset_btn = _chip(btn_row, "Reset", func(): _on_insp_fit_or_reset(false))
	_insp_reset_btn.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	_insp_reset_btn.tooltip_text = "Reset scale to 1.00x and pan to (0, 0)."
	_insp_replace_btn = _chip(btn_row, "Replace…", func(): _on_import_image(true))
	_insp_replace_btn.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	_insp_variant_btn = _chip(btn_row, "%s Variant" % DccIcons.SYMBOLS["add"],
		func(): _on_import_image(false), true)
	_insp_variant_btn.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	_insp_variant_btn.tooltip_text = "Import another image into this slot. Variants are chosen by weight at render time (AS-14) -- there is no \"active\" one."

	var bg_row := _insp_row(rows, "Preview bg")
	var sw_box := HBoxContainer.new()
	sw_box.add_theme_constant_override("separation", 5)
	bg_row.add_child(sw_box)
	for i in PREVIEW_SWATCHES.size():
		var swatch: Dictionary = PREVIEW_SWATCHES[i]
		var sw := SlotCell.new()
		sw.custom_minimum_size = Vector2(SZ_SWATCH, SZ_SWATCH)
		sw.checker_px = 8.0
		sw.bg_mode = String(swatch["mode"])
		sw.bg_color = swatch["color"]
		sw.tooltip_text = String(swatch["label"])
		sw.selected = String(swatch["mode"]) == _preview_bg
		var idx := i
		sw.gui_input.connect(func(ev: InputEvent):
			if ev is InputEventMouseButton and ev.pressed and ev.button_index == MOUSE_BUTTON_LEFT:
				_pick_preview_bg(idx))
		sw_box.add_child(sw)
		_insp_swatches.append(sw)

	var anchor_row := _insp_row(rows, "Anchor")
	var anchor_box := HBoxContainer.new()
	anchor_box.add_theme_constant_override("separation", 2)
	anchor_row.add_child(anchor_box)
	## AS-15: the canvas draws this as an editable three-way (top/centre/base).
	## `Family` fixes the anchor for every slot in it, so the real one is lit and
	## the other two are disabled with that reason rather than being drawn as
	## choices. The labels are the engine's own three values, not the canvas's --
	## `Family::anchor` is `none` (a tiled texture, anchored nowhere), `center`
	## or `bottom`, and there is no "top" for a label to be honest about.
	for entry in [["tiled", "none"], ["centre", "center"], ["base", "bottom"]]:
		var chip := _segment(anchor_box, String(entry[0]), Callable())
		chip.disabled = true
		## The per-family reason is written in `_refresh_inspector` below; this
		## is the one it carries before anything is selected, so the row never
		## renders as three greyed chips with nothing to say (2026-08-25 sweep).
		chip.tooltip_text = "Anchor is fixed by the family (cartalith-assets::Family), not a per-slot setting. Select a slot to see which of the three its family uses."
		_insp_anchor_chips[String(entry[1])] = chip

	var tag_row := _insp_row(rows, "Tags")
	_insp_tags = HFlowContainer.new()
	_insp_tags.add_theme_constant_override("h_separation", 4)
	_insp_tags.add_theme_constant_override("v_separation", 4)
	_insp_tags.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	tag_row.add_child(_insp_tags)

	_insp_note = DccTheme.label("", "text_ghost", DccTheme.FS_MICRO)
	_insp_note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_insp_note.custom_minimum_size.x = W_INSPECTOR - 52
	rows.add_child(_insp_note)

	# -- VARIANTS -------------------------------------------------------------
	_insp_detail.add_child(_rule_soft())
	var var_pad := _pad(_insp_detail, 14, 11, 14, 11)
	var var_col := VBoxContainer.new()
	var_col.add_theme_constant_override("separation", 8)
	var_pad.add_child(var_col)
	_insp_variants_head = DccTheme.mono_label("VARIANTS", "text_ghost", DccTheme.FS_MICRO, 1)
	_insp_variants_head.tooltip_text = "AS-14: the renderer picks a variant by weight, so this strip selects which one the preview above shows -- not which one the map draws."
	_insp_variants_head.mouse_filter = Control.MOUSE_FILTER_STOP
	var_col.add_child(_insp_variants_head)
	_insp_variants = HBoxContainer.new()
	_insp_variants.add_theme_constant_override("separation", 8)
	var_col.add_child(_insp_variants)

	# -- PACK METADATA (built once; never rebuilt under the caret) ------------
	body.add_child(_rule_soft())
	var pack_pad := _pad(body, 14, 11, 14, 11)
	var pack_col := VBoxContainer.new()
	pack_col.add_theme_constant_override("separation", 7)
	pack_pad.add_child(pack_col)
	pack_col.add_child(DccTheme.mono_label("PACK METADATA", "text_ghost", DccTheme.FS_MICRO, 1))
	_pack_name_field = _pack_field(pack_col, "name")
	_pack_author_field = _pack_field(pack_col, "author")
	_pack_license_field = _pack_field(pack_col, "license")

	# -- foot -----------------------------------------------------------------
	col.add_child(DccTheme.rule())
	var foot_pad := _pad(col, 14, 9, 14, 9)
	var foot := HBoxContainer.new()
	foot.add_theme_constant_override("separation", 6)
	foot_pad.add_child(foot)
	_validate_btn = _chip(foot, "Validate", func(): _on_validate(), false, 0, 6)
	_validate_btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_validate_btn.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	_clear_btn = _chip(foot, "Clear library…", func(): _on_clear_library(), false, 0, 6)
	_clear_btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_clear_btn.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	_clear_btn.add_theme_color_override("font_color", DccTheme.c("text_dim"))

	_refresh_inspector()
	return wrap

func _pack_field(parent: Control, label_text: String) -> LineEdit:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	var l := DccTheme.mono_label(label_text, "text_faint", DccTheme.FS_TINY)
	l.custom_minimum_size.x = 52
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	row.add_child(l)
	var le := LineEdit.new()
	le.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_well(le, 8, 3)
	le.text_submitted.connect(func(_t: String): _commit_pack_info())
	le.focus_exited.connect(func(): _commit_pack_info())
	row.add_child(le)
	parent.add_child(row)
	return le

func _pick_preview_bg(index: int) -> void:
	var swatch: Dictionary = PREVIEW_SWATCHES[index]
	_preview_bg = String(swatch["mode"])
	_preview_color = swatch["color"]
	for i in _insp_swatches.size():
		var sw: SlotCell = _insp_swatches[i]
		sw.selected = i == index
		sw.queue_redraw()
	_insp_preview.bg_mode = _preview_bg
	_insp_preview.bg_color = _preview_color
	_insp_preview.queue_redraw()

func _refresh_inspector() -> void:
	if _insp_detail == null:
		return
	_phone_refit()   ## PH-07 -- see `_phone_refit()`.
	if _focused_uid == "":
		_insp_head.text = "NO SLOT SELECTED"
		_insp_empty.visible = true
		_insp_detail.visible = false
		return

	# AS-07: real slot/item/pack queries -- `as_slot_summary` is the source of
	# truth (it also confirms the slot still exists, e.g. after a batch
	# delete/rename elsewhere).
	var summary: Dictionary = _bridge.as_slot_summary(_focused_uid)
	if not bool(summary.get("ok", false)):
		_insp_head.text = "SLOT GONE"
		_insp_empty.text = "This slot no longer exists in the live session (removed by a batch edit)."
		_insp_empty.visible = true
		_insp_detail.visible = false
		return
	_insp_empty.visible = false
	_insp_detail.visible = true

	var fam_key := String(summary.get("family", ""))
	var fam := _family_by_key(fam_key)
	var entry: Dictionary = {}
	for e in _slot_order:
		if String(e["uid"]) == _focused_uid:
			entry = e
			break
	var code := String(entry.get("code", "—"))
	var slot_name := String(summary.get("name", ""))
	var item_count := int(summary.get("item_count", 0))
	_insp_head.text = "%s · %s" % [code, slot_name.to_upper()]
	_preview_index = clampi(_preview_index, 0, maxi(item_count - 1, 0))

	# -- preview + file readout ----------------------------------------------
	_insp_preview.bg_mode = _preview_bg
	_insp_preview.bg_color = _preview_color
	_insp_preview.thumb = null
	if item_count > 0:
		var preview_png: PackedByteArray = _bridge.as_thumbnail_png(_focused_uid, _preview_index, 256)
		if preview_png.size() > 0:
			var pimg := Image.new()
			if pimg.load_png_from_buffer(preview_png) == OK:
				_insp_preview.thumb = ImageTexture.create_from_image(pimg)
	_insp_preview.show_empty = item_count == 0
	_insp_preview.queue_redraw()

	var bake_note := "%d px %s" % [int(fam.get("size", 0)),
		"opaque, seamless tile" if bool(fam.get("texture", false)) else "RGBA, straight alpha"]
	## Setting .value below fires value_changed on every one of these controls;
	## _on_insp_transform_changed reads _insp_syncing and no-ops while it's up,
	## so a refresh never turns into a spurious as_set_item_transform write.
	_insp_syncing = true
	if item_count > 0:
		var item: Dictionary = _bridge.as_item_summary(_focused_uid, _preview_index)
		if bool(item.get("ok", false)):
			## The canvas's line is `capital-star.png · 512 × 512 · PNG · 84 KB`.
			## The engine reports no stored byte size (`as_item_summary` carries
			## name/transform/decoded size/hash and nothing else), so the last
			## field is dropped rather than invented; the rest is real.
			_insp_file.text = "%s · %d × %d · PNG" % [
				String(item.get("name", "")), int(item.get("w", 0)), int(item.get("h", 0))]
			var pan_x := float(item.get("pan_x", 0.0))
			var pan_y := float(item.get("pan_y", 0.0))
			_insp_file.tooltip_text = "Bakes to %s · pan (%.0f, %.0f) · content hash %s" % [
				bake_note, pan_x, pan_y, String(item.get("hash", "—"))]
			var pct := float(item.get("scale", 1.0)) * 100.0
			_insp_scale.value = clampf(pct, _insp_scale.min_value, _insp_scale.max_value)
			_insp_scale_readout.text = "%d%%" % int(roundf(pct))
			_insp_pan_x.value = clampf(pan_x, _insp_pan_x.min_value, _insp_pan_x.max_value)
			_insp_pan_y.value = clampf(pan_y, _insp_pan_y.min_value, _insp_pan_y.max_value)
		else:
			_insp_file.text = "—"
			_insp_scale_readout.text = "—"
	else:
		_insp_file.text = "no art in this slot"
		_insp_file.tooltip_text = "Bakes to %s once an image lands here." % bake_note
		_insp_scale.value = 100
		_insp_scale_readout.text = "—"
		_insp_pan_x.value = 0
		_insp_pan_y.value = 0
	_insp_syncing = false

	_insp_scale.editable = item_count > 0
	_insp_pan_x.editable = item_count > 0
	_insp_pan_y.editable = item_count > 0
	_insp_fit_btn.disabled = item_count == 0
	_insp_reset_btn.disabled = item_count == 0

	_insp_replace_btn.disabled = item_count == 0
	_insp_replace_btn.tooltip_text = "Nothing to replace yet -- use ＋ Variant to add the first image." \
		if item_count == 0 else "Import a PNG and drop the slot's current first variant."

	# -- anchor (AS-15) -------------------------------------------------------
	var real_anchor := String(fam.get("anchor", "center"))
	for key in _insp_anchor_chips:
		var chip: Button = _insp_anchor_chips[key]
		var on: bool = key == real_anchor
		_set_segment_on(chip, on)
		chip.tooltip_text = "Anchor is fixed by the family (cartalith-assets::Family), not a per-slot setting -- %s is %s." % [
			String(fam.get("title", fam_key)), real_anchor]

	# -- tags -----------------------------------------------------------------
	for c in _insp_tags.get_children():
		_insp_tags.remove_child(c)
		c.queue_free()
	var tags: PackedStringArray = summary.get("tags", PackedStringArray())
	for t in tags:
		var tc := _segment(_insp_tags, String(t), Callable())
		tc.disabled = true
		tc.tooltip_text = "Tags are added in batch (as_batch_tag); removing one has no binding yet."
	var add_tag := _segment(_insp_tags, DccIcons.SYMBOLS["add"], func(): _on_tag_focused())
	add_tag.tooltip_text = "Add tag(s) to this slot (as_batch_tag on the focused slot)."

	# -- disclosure note ------------------------------------------------------
	if item_count == 0:
		_insp_note.text = "No art stored yet — ＋ Variant, or Import image… in the rail foot, lands one here."
	elif item_count > 1:
		_insp_note.text = "%d variants stored; the renderer picks one by weight (AS-14). The preview shows variant %d." % [
			item_count, _preview_index + 1]
	else:
		_insp_note.text = ""
	_insp_note.visible = _insp_note.text != ""

	# -- variants strip -------------------------------------------------------
	_insp_variants_head.text = "VARIANTS · %d" % item_count
	for c in _insp_variants.get_children():
		_insp_variants.remove_child(c)
		c.queue_free()
	for i in item_count:
		var vc := SlotCell.new()
		vc.custom_minimum_size = Vector2(SZ_VARIANT, SZ_VARIANT)
		vc.checker_px = 8.0
		vc.selected = i == _preview_index
		var vpng: PackedByteArray = _bridge.as_thumbnail_png(_focused_uid, i, 96)
		if vpng.size() > 0:
			var vimg := Image.new()
			if vimg.load_png_from_buffer(vpng) == OK:
				vc.thumb = ImageTexture.create_from_image(vimg)
		var idx := i
		vc.tooltip_text = "Show variant %d in the preview above." % (i + 1)
		vc.gui_input.connect(func(ev: InputEvent):
			if ev is InputEventMouseButton and ev.pressed and ev.button_index == MOUSE_BUTTON_LEFT:
				_preview_index = idx
				_refresh_inspector())
		_insp_variants.add_child(vc)

	_refresh_pack_info_fields(_bridge.as_pack_info())

func _on_tag_focused() -> void:
	if _focused_uid == "":
		return
	var uids := PackedStringArray([_focused_uid])
	_prompt_text("Tag %s" % _focused_uid, "Add tag(s) -- comma-separated:", "", func(t: String):
		_bridge.as_batch_tag(uids, t)
		_dirty = true
		_refresh_inspector()
		_refresh_status_line())

## `AssetValidator.run()` -- the real, ordered warning list, shown in a
## simple modal (the reference's own `alert`-style summary).
func _on_validate() -> void:
	var warnings: PackedStringArray = _bridge.as_validate()
	if _validate_btn != null:
		_validate_btn.text = "Validate" if warnings.is_empty() \
			else "Validate · %d warning%s" % [warnings.size(), "" if warnings.size() == 1 else "s"]
	var d := AcceptDialog.new()
	d.title = "Validation"
	d.min_size = Vector2i(420, 0)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 4)
	if warnings.is_empty():
		body.add_child(DccTheme.label("%s No issues found." % DccIcons.SYMBOLS["tick"],
			"accent", DccTheme.FS_SMALL))
	else:
		for w in warnings:
			var l := DccTheme.label("%s %s" % [DccIcons.SYMBOLS["warn_tri"], String(w)],
				"warn", DccTheme.FS_SMALL)
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
		_dirty = true
		_selected.clear()
		_focused_uid = ""
		_preview_index = 0
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
	_refresh_rail_counts()
	_refresh_collections_rail()
	_refresh_pack_info_fields(_bridge.as_pack_info())
	_refresh_status_line()

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
	_refresh_rail_counts()

func _on_apply_to_map() -> void:
	var result: Dictionary = _bridge.as_apply_to_map()
	if bool(result.get("ok", false)):
		_dirty = false
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
#
# The canvas draws this as a 760 px card: a title bar with its own ✕, a sheet
# preview column, and a 274 px settings column that ends in a summary line and
# a Cancel / Slice pair. It used to be a single vertical stack of stock
# widgets, wide enough to clip its own labels.
# ---------------------------------------------------------------------------

func _build_slicer_modal() -> void:
	_slicer = AcceptDialog.new()
	_slicer.title = "▦ SPRITE SHEET SLICER"
	_slicer.get_ok_button().hide()
	_slicer.borderless = true
	## The canvas floats this card on `box-shadow:0 30px 90px rgba(0,0,0,.7)` --
	## the one place in the whole shell where anything is raised off the ground.
	var card := DccTheme.outline("line", "panel")
	card.shadow_color = Color(0, 0, 0, 0.6)
	card.shadow_size = 18
	card.shadow_offset = Vector2(0, 10)
	_slicer.add_theme_stylebox_override("panel", card)
	_slicer.add_theme_constant_override("buttons_min_height", 0)
	_slicer.add_theme_constant_override("margin", 0)
	_slicer.size = Vector2i(W_SLICER, H_SLICER)
	## Closing by ESC must drop the engine-side sheet just like Cancel does, or
	## a ~24MB decoded raster outlives the modal that owns it.
	_slicer.close_requested.connect(_close_slicer)
	_slicer.canceled.connect(_close_slicer)
	## PH-07: parented to the SHELL, not to this dialog. An embedded subwindow
	## is laid out in its parent viewport's own 2D space, and on a phone this
	## dialog's viewport is content-scaled by `phone_scale()` -- so a slicer
	## sized to fill "the screen" from inside it would be sized in units 3.66x
	## larger than the screen's. That is the same physical-pixels-vs-parent-
	## space confusion `_popup_full()` records above, one level further in.
	## Parenting it to the shell puts it in the same unscaled space every other
	## phone window measures itself against, and costs nothing on the desktop:
	## the two `_close_slicer` connections above plus the Close chip's own call
	## are what keep it from outliving this window, not the parent link (see
	## the 2026-08-20 stranded-modal note in `_build_window_bar()`).
	(_host if _host != null else self).add_child(_slicer)
	_slicer_phone = DccWidgets.phone_window(_slicer, _host)

	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	_slicer.add_child(outer)

	var title_wrap := PanelContainer.new()
	title_wrap.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"bottom": 1}))
	var title_pad := _pad(title_wrap, 16, 11, 16, 11)
	var title_row := HBoxContainer.new()
	title_pad.add_child(title_row)
	var t := DccTheme.mono_label("%s SPRITE SHEET SLICER" % DccIcons.SYMBOLS["panels"],
		"text_bright", DccTheme.FS_MICRO, 2, true)
	t.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	title_row.add_child(t)
	_text_button(title_row, DccIcons.SYMBOLS["cross"], func(): _close_slicer())
	outer.add_child(title_wrap)

	## PH-07: the canvas's two columns are a flexible preview beside a fixed
	## 274 px settings stack -- 70% of a phone's 393 dp before the preview gets
	## anything. They stack, preview over settings, exactly as
	## `city_viewer_window.gd` stacks its canvas over its info column.
	var body: BoxContainer = VBoxContainer.new() if _slicer_phone else HBoxContainer.new()
	body.add_theme_constant_override("separation", 0)
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(body)

	# -- left: sheet preview --------------------------------------------------
	var left := PanelContainer.new()
	left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"right": 1}))
	var left_pad := _pad(left, 16, 16, 16, 16)
	var left_col := VBoxContainer.new()
	left_col.add_theme_constant_override("separation", 9)
	left_pad.add_child(left_col)
	_sheet_preview = SheetPreview.new()
	## A fixed band under a scrolling settings column, for the same reason
	## `city_viewer_window.gd` gives: "the leftover height" is not a well-defined
	## quantity once the column below scrolls.
	_sheet_preview.custom_minimum_size = Vector2(0,
		PHONE_SHEET_PREVIEW if _slicer_phone else H_SHEET_PREVIEW)
	_sheet_preview.owner_window = self
	_sheet_preview.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_sheet_preview.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_sheet_preview.tooltip_text = "Wheel to zoom · middle-drag to pan · click a cell to pick it (view only -- Slice still cuts the whole grid) · drag the dot to set Margin."
	left_col.add_child(_sheet_preview)
	var preview_foot := HBoxContainer.new()
	preview_foot.add_theme_constant_override("separation", 10)
	_sheet_readout = DccTheme.mono_label("no sheet chosen", "text_faint", DccTheme.FS_TINY)
	_sheet_readout.clip_text = true
	_sheet_readout.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	preview_foot.add_child(_sheet_readout)
	_slicer_reset_view_btn = _text_button(preview_foot, "Reset view", func(): _sheet_preview.reset_view())
	left_col.add_child(preview_foot)
	var choose := _chip(left_col, "Choose image…", func(): _pick_sheet_image())
	choose.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	body.add_child(left)

	# -- right: settings ------------------------------------------------------
	var right := PanelContainer.new()
	right.add_theme_stylebox_override("panel", DccTheme.panel("panel"))
	var side := VBoxContainer.new()
	side.add_theme_constant_override("separation", 10)
	if _slicer_phone:
		## PH-07. Fourteen rows of `66px label · control` at §13's 44 dp floor is
		## ~700 dp of column in a screen that has ~610 of them under the preview
		## band. A desktop column that merely fits is a phone column that has to
		## scroll; without this the Cancel/Slice foot -- the only way to run the
		## thing -- is below the bottom edge with nothing to reach it.
		right.size_flags_vertical = Control.SIZE_EXPAND_FILL
		var side_scroll := ScrollContainer.new()
		side_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
		side_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
		right.add_child(side_scroll)
		var side_pad := _pad(side_scroll, 16, 16, 16, 16)
		side_pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		side.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		side_pad.add_child(side)
	else:
		right.custom_minimum_size.x = W_SLICER_SIDE
		_pad(right, 16, 16, 16, 16).add_child(side)
	body.add_child(right)

	## 128 is the engine's own ceiling (`clampInt(v,1,128)`, ported as
	## `slicer::clamp_grid_count`), so the spinbox cannot ask for a grid the
	## engine would silently clamp behind the user's back.
	_slicer_cols = _slicer_number(side, "Columns", 1, 128, 6)
	_slicer_rows = _slicer_number(side, "Rows", 1, 128, 4)
	_slicer_margin = _slicer_number(side, "Margin", 0, 512, 0)
	_slicer_spacing = _slicer_number(side, "Spacing", 0, 256, 0)
	## AS-17: a cols/rows edit reshapes the grid a dragged line array was
	## built for, so it goes stale under whatever fingers it -- reset to
	## uniform (`compute_cells`'s own fallback) rather than carried over onto
	## a grid it no longer describes. `_slicer_number`'s own `value_changed`
	## connection above already calls `_refresh_slicer_summary()`; this is a
	## second, independent connection, not a replacement for it.
	_slicer_cols.value_changed.connect(func(_v): _slicer_col_lines = PackedFloat64Array())
	_slicer_rows.value_changed.connect(func(_v): _slicer_row_lines = PackedFloat64Array())

	_slicer_trim = _slicer_check(side, "Trim transparent edges", false,
		func(v: bool): _slice_trim = v,
		"Crops each cell to its content. Note: the reference slicer has no trim -- this is a port-side addition (DCC_SHELL_SPEC.md §8), using the reference's own alpha>8 threshold so it agrees with Skip empty cells about what content is.")
	_slicer_skip = _slicer_check(side, "Skip empty cells", true,
		func(v: bool): _slice_skip_empty = v; _refresh_slicer_summary(),
		"isBlank: a cell with no pixel over alpha 8 is dropped rather than added. On by default, as in the reference.")
	## `background → transparent` -- the reference's *own* second pixel toggle
	## (`#alChEnable`/`#alChTol`), which the canvas omits. Real, so it is here.
	_slicer_chroma = _slicer_check(side, "Background → transparent", false,
		func(v: bool): _slice_chroma = v; _refresh_slicer_summary(),
		"applyChroma: pixels within Tolerance of the keyed colour have their alpha zeroed, per cell.")
	var chroma_row := _slicer_row(side, "Key · tol")
	_slicer_chroma_color = ColorPickerButton.new()
	_slicer_chroma_color.color = Color(1, 1, 1)
	_slicer_chroma_color.edit_alpha = false
	_slicer_chroma_color.custom_minimum_size = Vector2(38, 22)
	_slicer_chroma_color.focus_mode = Control.FOCUS_NONE
	_slicer_chroma_color.color_changed.connect(func(_c: Color): _refresh_slicer_summary())
	chroma_row.add_child(_slicer_chroma_color)
	_slicer_chroma_tol = SpinBox.new()
	_slicer_chroma_tol.min_value = 0
	_slicer_chroma_tol.max_value = 150
	_slicer_chroma_tol.value = 40
	_slicer_chroma_tol.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_well(_slicer_chroma_tol.get_line_edit(), 8, 3)
	_slicer_chroma_tol.value_changed.connect(func(_v): _refresh_slicer_summary())
	chroma_row.add_child(_slicer_chroma_tol)

	var target_row := _slicer_row(side, "Assign to")
	_slicer_target = OptionButton.new()
	_slicer_target.focus_mode = Control.FOCUS_NONE
	_slicer_target.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_slicer_target.tooltip_text = "Where the cells land. The first four are the reference's own targets; \"a family, slot by slot\" is DCC_SHELL_SPEC.md §8's instead."
	for t2 in SLICE_TARGETS:
		_slicer_target.add_item(String(t2["label"]))
	_style_option(_slicer_target, true)
	_slicer_target.item_selected.connect(func(i: int):
		_slice_target_index = i
		_refresh_slicer_target_controls())
	target_row.add_child(_slicer_target)

	var fam_row := _slicer_row(side, "Family")
	_slicer_family = OptionButton.new()
	_slicer_family.focus_mode = Control.FOCUS_NONE
	_slicer_family.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	for f in FAMILIES:
		_slicer_family.add_item("%s · %s" % [String(f["code"]), String(f["title"])])
	_style_option(_slicer_family, false)
	_slicer_family.item_selected.connect(func(i: int): _slice_family_index = i)
	fam_row.add_child(_slicer_family)

	var fill_row := _slicer_row(side, "Fill from")
	var fill_box := HBoxContainer.new()
	fill_box.add_theme_constant_override("separation", 2)
	fill_row.add_child(fill_box)
	_slicer_fill_chips["first"] = _segment(fill_box, "first empty", func(): _set_fill(false))
	_slicer_fill_chips["over"] = _segment(fill_box, "overwrite", func(): _set_fill(true))
	_set_fill(false)

	var name_row := _slicer_row(side, "New name")
	_slicer_name = LineEdit.new()
	_slicer_name.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_well(_slicer_name, 8, 3)
	name_row.add_child(_slicer_name)
	var set_row := _slicer_row(side, "Custom set")
	_slicer_set = LineEdit.new()
	_slicer_set.text = "Default"
	_slicer_set.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_well(_slicer_set, 8, 3)
	set_row.add_child(_slicer_set)

	side.add_child(DccTheme.spacer())
	_slicer_summary = DccTheme.mono_label("", "text_ghost", DccTheme.FS_TINY)
	_slicer_summary.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_slicer_summary.custom_minimum_size.x = W_SLICER_SIDE - 32
	_slicer_summary.tooltip_text = "Every control here is live. The grid, the cell detection and the slice itself all run in the engine (cartalith-assets::slicer, a port of the reference's SpriteSheetImporter); the overlay draws the exact rectangles the slice will cut. Slicing is non-destructive -- the sheet stays loaded, so you can re-slice it with different settings."
	side.add_child(_slicer_summary)

	var foot := HBoxContainer.new()
	foot.add_theme_constant_override("separation", 6)
	side.add_child(foot)
	var cancel := _chip(foot, "Cancel", func(): _close_slicer(), false, 0, 7)
	cancel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cancel.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	_slice_btn = _chip(foot, "Slice", func(): _on_slice(), true, 0, 7)
	_slice_btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_slice_btn.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	_slice_btn.disabled = true
	_slice_btn.tooltip_text = "Choose a sprite sheet first."
	_refresh_slicer_target_controls()
	_refresh_slicer_summary()

## The slicer column's own `66px label · control` row (canvas value).
static func _slicer_row(parent: Control, label_text: String) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	var l := DccTheme.label(label_text, "text_dim", DccTheme.FS_SMALL)
	l.custom_minimum_size.x = 66
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	row.add_child(l)
	parent.add_child(row)
	return row

func _slicer_number(parent: Control, label_text: String, lo: int, hi: int, value: int) -> SpinBox:
	var row := _slicer_row(parent, label_text)
	var sb := SpinBox.new()
	sb.min_value = lo
	sb.max_value = hi
	sb.step = 1
	sb.value = value
	sb.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_well(sb.get_line_edit(), 8, 3)
	sb.value_changed.connect(func(_v): _refresh_slicer_summary())
	row.add_child(sb)
	return sb

## The canvas writes a slicer toggle as an accent `☑` followed by a body-text
## label -- a typographic mark, not a boxed control, and two colours in one
## row. Godot's `CheckBox` brings the stock theme's filled slab *and* a system
## fallback that draws U+2611 as a colour emoji, so this is a borderless toggle
## `Button` hosting two Plex labels instead (the same trick the family rail
## uses to get aligned columns inside a button).
static func _slicer_check(parent: Control, label_text: String, value: bool,
		on_change: Callable, tip: String) -> Button:
	var b := Button.new()
	b.toggle_mode = true
	b.button_pressed = value
	b.focus_mode = Control.FOCUS_NONE
	b.custom_minimum_size.y = 22
	b.tooltip_text = tip
	for sb_name in ["normal", "pressed", "hover", "hover_pressed", "focus"]:
		b.add_theme_stylebox_override(sb_name, DccTheme.empty())

	var row := HBoxContainer.new()
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_theme_constant_override("separation", 10)
	row.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	b.add_child(row)
	var mark := DccTheme.mono_label(
		DccIcons.SYMBOLS["checked"] if value else DccIcons.SYMBOLS["unchecked"],
		"accent" if value else "text_ghost", DccTheme.FS_SMALL)
	mark.custom_minimum_size.x = 12
	mark.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	mark.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(mark)
	var l := DccTheme.label(label_text, "text", DccTheme.FS_SMALL)
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(l)

	b.toggled.connect(func(v: bool):
		mark.text = DccIcons.SYMBOLS["checked"] if v else DccIcons.SYMBOLS["unchecked"]
		mark.add_theme_color_override("font_color",
			DccTheme.c("accent") if v else DccTheme.c("text_ghost"))
		on_change.call(v))
	parent.add_child(b)
	return b

static func _style_option(ob: OptionButton, accent: bool) -> void:
	var token := "accent" if accent else "line"
	ob.add_theme_font_override("font", DccTheme.mono(0))
	ob.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	ob.add_theme_color_override("font_color",
		DccTheme.c("text_bright") if accent else DccTheme.c("text"))
	ob.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	for sb_name in ["normal", "pressed", "focus"]:
		ob.add_theme_stylebox_override(sb_name, _box(token, "", 9, 4))
	ob.add_theme_stylebox_override("hover", _box(token, "line_soft", 9, 4))
	ob.add_theme_stylebox_override("disabled", _box("line_soft", "", 9, 4))

func _set_fill(overwrite: bool) -> void:
	_slice_overwrite = overwrite
	_set_segment_on(_slicer_fill_chips["first"], not overwrite)
	_set_segment_on(_slicer_fill_chips["over"], overwrite)

func _open_slicer() -> void:
	if not visible:
		_popup_full()
	_refresh_slicer_target_controls()
	if DccWidgets.phone_present(_slicer, _host):
		_host.phone_fit(_slicer, 1.0)   ## PH-07; idempotent, so only new rows pay.
		return
	_slicer.popup_centered(Vector2i(W_SLICER, H_SLICER))

## Closing drops the engine-side sheet too -- a decoded 3072×2048 sheet is
## ~24MB of RGBA the session has no reason to hold once the modal is gone.
func _close_slicer() -> void:
	if _sheet_loaded:
		_bridge.as_clear_sheet()
		_sheet_loaded = false
	_slicer.hide()

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

## The image goes to the engine (which owns the slice) *and* to a local
## `ImageTexture` (which is only ever drawn). Godot's own decoder is not asked
## for anything the engine then trusts -- if `as_load_sheet` refuses the bytes,
## the preview is cleared too, so the modal can never show a sheet the engine
## does not hold.
func _load_sheet_image(path: String) -> void:
	var bytes := FileAccess.get_file_as_bytes(path)
	if bytes.is_empty():
		_sheet_readout.text = "could not read %s" % path.get_file()
		return
	var result: Dictionary = _bridge.as_load_sheet(path.get_file(), bytes)
	if not bool(result.get("ok", false)):
		_clear_sheet_preview()
		_sheet_readout.text = "%s: %s" % [path.get_file(), String(result.get("error", "load failed"))]
		return
	var img := Image.new()
	if img.load(path) != OK:
		_clear_sheet_preview()
		_sheet_readout.text = "%s decoded in the engine but not for preview" % path.get_file()
		return
	_sheet_image = img
	_sheet_loaded = true
	_sheet_preview.img_tex = ImageTexture.create_from_image(img)
	_sheet_preview.reset_view()   ## a new sheet starts fit-to-view, not wherever the last one was panned/zoomed to
	_sheet_preview.selected_cell = -1
	## `loadSheet`'s own `this.resetLines()` (reference line 27837): a new
	## sheet starts with uniform lines, not whatever the previous one's got
	## dragged to.
	_slicer_col_lines = PackedFloat64Array()
	_slicer_row_lines = PackedFloat64Array()
	_sheet_readout.text = "%s · %d × %d" % [
		path.get_file(), int(result.get("w", 0)), int(result.get("h", 0))]
	_refresh_slicer_summary()

func _clear_sheet_preview() -> void:
	_sheet_image = null
	_sheet_loaded = false
	_sheet_preview.img_tex = null
	_sheet_preview.reset_view()
	_sheet_preview.selected_cell = -1
	_slicer_col_lines = PackedFloat64Array()
	_slicer_row_lines = PackedFloat64Array()
	_sheet_preview.queue_redraw()
	_refresh_slicer_summary()

## The slicer modal's four numbers and three toggles, in `as_slice_preview`/
## `as_slice_apply`'s own `opts` shape. One builder for both calls, so the
## preview can never describe a different grid than the slice cuts. AS-17
## adds two more, both optional: `col_lines`/`row_lines` (a dragged interior
## line's fractions -- omitted, not sent empty, so the engine's own uniform
## default takes over) and `only_cell` (the selected cell's flat index, so
## `as_slice_apply` narrows to it instead of the whole grid).
func _slice_opts() -> Dictionary:
	var opts := {
		"cols": int(_slicer_cols.value),
		"rows": int(_slicer_rows.value),
		"margin": _slicer_margin.value,
		"spacing": _slicer_spacing.value,
		"trim": _slice_trim,
		"skip_empty": _slice_skip_empty,
		"chroma": _slice_chroma,
	}
	if _slice_chroma:
		var c := _slicer_chroma_color.color
		opts["chroma_r"] = int(roundf(c.r * 255.0))
		opts["chroma_g"] = int(roundf(c.g * 255.0))
		opts["chroma_b"] = int(roundf(c.b * 255.0))
		opts["chroma_tol"] = _slicer_chroma_tol.value
	if not _slicer_col_lines.is_empty():
		opts["col_lines"] = _slicer_col_lines
	if not _slicer_row_lines.is_empty():
		opts["row_lines"] = _slicer_row_lines
	if _sheet_preview != null and _sheet_preview.selected_cell >= 0:
		opts["only_cell"] = _sheet_preview.selected_cell
	return opts

## §8's `N cells detected · M non-empty` readout, and the overlay behind it --
## both from `as_slice_preview`, the engine's real detection pass (the same
## crop, chroma key and alpha>8 `isBlank` the slice itself runs).
func _refresh_slicer_summary() -> void:
	if _slicer_summary == null or _slice_btn == null:
		return
	if not _sheet_loaded:
		_slicer_summary.text = "Choose a sheet, then set the grid. Slicing is non-destructive; the sheet stays loaded for a re-slice."
		_slice_btn.text = "Slice"
		_slice_btn.disabled = true
		_slice_btn.tooltip_text = "Choose a sprite sheet first."
		return
	var p: Dictionary = _bridge.as_slice_preview(_slice_opts())
	if not bool(p.get("ok", false)):
		_slicer_summary.text = String(p.get("error", "preview failed"))
		_sheet_preview.usable = false
		_sheet_preview.queue_redraw()
		_slice_btn.text = "Slice"
		_slice_btn.disabled = true
		_slice_btn.tooltip_text = String(p.get("error", ""))
		return
	var total := int(p.get("total", 0))
	var non_empty := int(p.get("non_empty", 0))
	var usable := bool(p.get("usable", false))
	_sheet_preview.usable = usable
	## A cols/rows/margin/spacing edit reshapes the grid under whatever index
	## was picked -- stale rather than wrong-but-plausible, so it's cleared
	## rather than carried over onto a cell it may no longer point at.
	_sheet_preview.selected_cell = -1
	_sheet_preview.col_x0 = p.get("col_x0", PackedFloat64Array())
	_sheet_preview.col_x1 = p.get("col_x1", PackedFloat64Array())
	_sheet_preview.row_y0 = p.get("row_y0", PackedFloat64Array())
	_sheet_preview.row_y1 = p.get("row_y1", PackedFloat64Array())
	## AS-17: the undisplaced division lines -- a dragged line's handle draws
	## and hit-tests against these, not the gutter-narrowed cell edges above.
	_sheet_preview.col_lines_px = p.get("col_lines_px", PackedFloat64Array())
	_sheet_preview.row_lines_px = p.get("row_lines_px", PackedFloat64Array())
	var blanks: Dictionary = {}
	for i in p.get("blank", PackedInt32Array()):
		blanks[int(i)] = true
	_sheet_preview.blank_cells = blanks
	_sheet_preview.queue_redraw()
	if not usable:
		_slicer_summary.text = "Grid too dense — reduce columns/rows or spacing."
		_slice_btn.text = "Slice"
		_slice_btn.disabled = true
		_slice_btn.tooltip_text = "The cells would have zero or negative size."
		return
	var will_add := non_empty if _slice_skip_empty else total
	_slicer_summary.text = "%d cells detected · %d non-empty%s. Slicing is non-destructive; the sheet stays loaded." % [
		total, non_empty,
		("  ·  %d skipped" % (total - non_empty)) if _slice_skip_empty and total > non_empty else ""]
	_slice_btn.text = "Slice %d cells" % will_add
	_slice_btn.disabled = will_add <= 0
	_slice_btn.tooltip_text = "Every cell is empty." if will_add <= 0 else ""

## `SheetPreview`'s own margin-handle drag callback (AS-17). Writing straight
## to the SpinBox, rather than to `_slice_opts()`'s state directly, is
## deliberate: `_slicer_number`'s `value_changed` connection already calls
## `_refresh_slicer_summary()` on any edit, spinbox or drag alike, so the
## readout/overlay/Slice-button-count all stay the single source of truth
## this file already had -- dragging the handle is just a second way to move
## the same number the spinbox moves.
func _on_slicer_margin_dragged(new_margin: float) -> void:
	if _slicer_margin != null:
		_slicer_margin.value = new_margin

## `SheetPreview`'s own interior-line-handle drag callback (AS-17). `axis` is
## `"col"`/`"row"`, `index` the line within `_slicer_col_lines`/`row_lines`,
## `sheet_value` the drag's raw sheet-space x/y. Converting that to a fraction
## of the grid rect's own span (`GridRect::inset`'s own `(margin, dim-2*margin)`
## terms -- the same relationship `SheetPreview._margin_handle_sheet_pos()`
## already reads off this window) is the one piece of geometry done here; the
## actual clamp-so-lines-never-cross rule is `move_line`, real engine logic,
## fetched via `as_slicer_move_line` rather than reimplemented.
func _on_slicer_line_dragged(axis: String, index: int, sheet_value: float) -> void:
	if _sheet_image == null:
		return
	var margin: float = _slicer_margin.value
	var dim: float = float(_sheet_image.get_width() if axis == "col" else _sheet_image.get_height())
	var span: float = maxf(1.0, dim - margin * 2.0)
	var frac: float = clampf((sheet_value - margin) / span, 0.0, 1.0)
	var n: int = int(_slicer_cols.value) if axis == "col" else int(_slicer_rows.value)
	var lines: PackedFloat64Array = _slicer_col_lines if axis == "col" else _slicer_row_lines
	if lines.size() != n + 1:
		lines = _bridge.as_uniform_lines(n)
	lines = _bridge.as_slicer_move_line(lines, index, frac)
	if axis == "col":
		_slicer_col_lines = lines
	else:
		_slicer_row_lines = lines
	_refresh_slicer_summary()

## `SheetPreview`'s own click-to-select callback (AS-17). Real now, not
## view-only: `_slice_opts()` reads `_sheet_preview.selected_cell` into
## `only_cell`, so `as_slice_apply` (`slice_target_from`, `lib.rs`) narrows
## the cut to exactly this cell. `index < 0` is a deselect (clicking the same
## cell again) -- the Slice button reverts to the whole-grid count
## `_refresh_slicer_summary()` last computed.
func _on_slicer_cell_selected(index: int) -> void:
	if _slicer_summary == null or _slice_btn == null:
		return
	if index < 0:
		_host.set_status("hint", "selection cleared -- Slice cuts the whole grid again.", "text_faint")
		_refresh_slicer_summary()
		return
	var cols: int = _sheet_preview.col_x0.size()
	if cols <= 0:
		return
	var col := index % cols
	var row := index / cols
	var blank: bool = _sheet_preview.blank_cells.has(index)
	var can_add: bool = not (blank and _slice_skip_empty)
	_host.set_status("hint",
		"cell col %d, row %d %s selected -- Slice cuts only this cell now. Click it again for the whole grid." %
			[col + 1, row + 1, "(empty)" if blank else "(non-empty)"], "text_faint")
	_slice_btn.text = "Slice this cell" if can_add else "Slice"
	_slice_btn.disabled = not can_add
	_slice_btn.tooltip_text = "" if can_add else "The selected cell is empty."

## Only the fields the chosen target actually uses stay enabled -- the
## reference greys nothing, but its own three targets read different inputs
## and leaving all of them live invites filling one the engine will ignore.
func _refresh_slicer_target_controls() -> void:
	if _slicer_target == null:
		return
	var needs: Array = SLICE_TARGETS[_slice_target_index]["needs"]
	_slicer_family.disabled = not needs.has("family")
	for key in _slicer_fill_chips:
		(_slicer_fill_chips[key] as Button).disabled = not needs.has("fill")
	_slicer_name.editable = needs.has("name")
	_slicer_set.editable = needs.has("set")
	if String(SLICE_TARGETS[_slice_target_index]["key"]) == "slot":
		_slicer_target.tooltip_text = ("Cells go to %s." % _focused_uid) if _focused_uid != "" \
			else "No slot is focused — pick one in the grid behind this modal first."

## `addSlices()`: the real slice. Non-destructive, so the modal stays open with
## the sheet still loaded and the user can slice it again differently.
func _on_slice() -> void:
	if not _sheet_loaded:
		return
	var opts := _slice_opts()
	var target: Dictionary = SLICE_TARGETS[_slice_target_index]
	var key := String(target["key"])
	opts["target"] = key
	match key:
		"family":
			opts["family"] = String(FAMILIES[_slice_family_index]["key"])
			opts["overwrite"] = _slice_overwrite
		"slot":
			if _focused_uid == "":
				_host.set_status("hint", "focus a slot in the grid first", "warn")
				return
			opts["uid"] = _focused_uid
		"new_custom":
			opts["name"] = _slicer_name.text
			opts["set"] = _slicer_set.text
		"per_cell":
			opts["set"] = _slicer_set.text
	var result: Dictionary = _bridge.as_slice_apply(opts)
	if not bool(result.get("ok", false)):
		_host.set_status("hint", String(result.get("error", "slice failed")), "warn")
		return
	var added := int(result.get("added", 0))
	var skipped := int(result.get("skipped_blank", 0))
	var unplaced := int(result.get("unplaced", 0))
	var msg := "sliced %d cell%s" % [added, "" if added == 1 else "s"]
	if skipped > 0:
		msg += " (%d blank skipped)" % skipped
	if unplaced > 0:
		msg += " — %d had nowhere to go" % unplaced
	_dirty = _dirty or added > 0
	_host.set_status("hint", msg, "accent")
	_refresh_grid()
	_refresh_inspector()
	_refresh_rail_counts()
	_refresh_pack_status()
	_refresh_slicer_summary()
