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
## - **Disclosed gap, still honest**: per-item scale/pan *editing*; the
##   slicer's canvas interaction (pan/zoom, draggable grid lines,
##   click-to-select cells) -- the modal slices the whole uniform grid rather
##   than a hand-picked selection; drag-and-drop onto a slot.
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
const W_SLICER := 760
## The canvas's card is 760 × ~390. This port's settings column carries three
## rows the canvas does not (the reference's own chroma key + tolerance, and
## the new-name / custom-set fields its flat target dropdown needs), so the
## card is taller by exactly those rows.
const H_SLICER := 560
const W_SLICER_SIDE := 274
const H_SHEET_PREVIEW := 296
const W_INSP_LABEL := 70
const SZ_VARIANT := 56
const SZ_SWATCH := 20

## The disclosure the family rail used to spend 90 px of prose on. Same words,
## now on the FAMILIES band's tooltip so the rail can look like the canvas.
const FAMILIES_NOTE := "Eight families, frozen against the reference engine (cartalith-assets::slots / library) -- not the design canvas's own 24. The canvas subdivides more finely (splitting e.g. \"Feature icons\" into \"Trees & cover\" / \"Rock & scree\"); no Rust type draws that line, and ASSET_LIBRARY_SCOPE.md §1 recorded the real eight when Phase 4's engine side was built. Capacity and fill counts are both real (AssetDB::slots_in_family + per-slot filled state)."

const SCALE_GAP_NOTE := "The transform shown is real (as_item_summary). Editing it is not wired: no as_set_item_transform exists on the engine side yet -- a smaller follow-on than the rest of this window."

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

	func _ready() -> void:
		mouse_filter = Control.MOUSE_FILTER_STOP

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

## The slicer modal's sheet preview -- a real loaded `Image` plus the real cell
## rectangles the engine will cut.
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
class SheetPreview extends Control:
	var img_tex: ImageTexture
	var col_x0: PackedFloat64Array = PackedFloat64Array()
	var col_x1: PackedFloat64Array = PackedFloat64Array()
	var row_y0: PackedFloat64Array = PackedFloat64Array()
	var row_y1: PackedFloat64Array = PackedFloat64Array()
	## Cell indices the engine's detection pass found empty, so the overlay can
	## dim them the way §8's "19 non-empty" readout implies.
	var blank_cells: Dictionary = {}
	var usable := true

	func _draw() -> void:
		var r := Rect2(Vector2.ZERO, size)
		draw_rect(r, DccTheme.c("sunken"), true)
		draw_rect(r, DccTheme.c("line"), false, 1.0)
		if img_tex == null:
			return
		var tex_size := img_tex.get_size()
		if tex_size.x <= 0 or tex_size.y <= 0:
			return
		var scale: float = minf(r.size.x / tex_size.x, r.size.y / tex_size.y)
		var draw_size := tex_size * scale
		var origin := r.position + (r.size - draw_size) * 0.5
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
				draw_dashed_line(cell.position + Vector2(cell.size.x, 0.0),
					cell.position + cell.size, line_color, 1.0, 4.0)
				draw_dashed_line(cell.position + Vector2(0.0, cell.size.y),
					cell.position + cell.size, line_color, 1.0, 4.0)

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

var _slice_trim := false
var _slice_skip_empty := true
var _slice_chroma := false
var _slice_target_index := 0
var _slice_family_index := 0
var _slice_overwrite := false

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
	_bridge.world_loaded.connect(func(): _refresh_pack_status())
	_build()
	_build_slicer_modal()
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

	var main := HBoxContainer.new()
	main.add_theme_constant_override("separation", 0)
	main.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(main)

	main.add_child(_build_family_rail())
	main.add_child(_build_slot_grid())
	main.add_child(_build_inspector())

	outer.add_child(_build_status_line())

## `⧉ ASSET LIBRARY · map hidden while open │ search · sort · slicer · select
## … Apply to map · Export pack .zip · Close ✕` -- the canvas's own order and
## its own chip treatment, not stock buttons.
func _build_window_bar() -> Control:
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"bottom": 1}))
	wrap.custom_minimum_size.y = H_BAR
	var pad := _pad(wrap, 16, 0, 16, 0)
	var row := HBoxContainer.new()
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

	_chip(row, "%s Sprite sheet…" % DccIcons.SYMBOLS["panels"], func(): _open_slicer())

	_select_mode_btn = _chip(row, "", func(): _toggle_select_mode(), true)
	_select_mode_btn.tooltip_text = "Batch selection driving Tag/Collect/Rename/Duplicate/Delete in the grid header."

	row.add_child(DccTheme.spacer())

	_apply_btn = _chip(row, "Apply to map", func(): _on_apply_to_map(), false, 10, 4)
	_export_btn = _chip(row, "Export pack .zip", func(): _on_export_pack(), true, 10, 4)
	## Visual sweep (2026-08-20) caught the slicer modal left stranded on top of
	## the whole app -- `_slicer` is a child `Window` of this dialog, and a child
	## `Window`'s visibility is independent of its parent's, so closing the
	## library while the slicer was open used to leave it floating over every
	## surface opened afterward. Closing this window always closes the slicer.
	var close_chip := _chip(row, "Close %s" % DccIcons.SYMBOLS["cross"],
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
	if _status_pack != null:
		var info: Dictionary = _bridge.as_pack_info()
		var pack_name := String(info.get("name", ""))
		_status_pack.text = "%s · %d / %d slots · %d item%s" % [
			pack_name if pack_name != "" else "unnamed pack",
			total_filled, total_slots, int(info.get("total_items", 0)),
			"" if int(info.get("total_items", 0)) == 1 else "s"]

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

## `replace_first` empties the slot's first variant once the new image is in --
## the inspector's Replace…, built out of `as_import_item` + `as_remove_item`
## rather than a binding that does not exist. Import order matters: the new
## bytes have to land successfully *before* anything is removed.
func _on_import_image(replace_first: bool = false) -> void:
	if _focused_uid == "":
		return
	var target_uid := _focused_uid
	var d := FileDialog.new()
	d.title = "Replace image" if replace_first else "Import image"
	d.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	d.access = FileDialog.ACCESS_FILESYSTEM
	d.add_filter("*.png ; PNG image")
	d.file_selected.connect(func(path: String):
		var bytes := FileAccess.get_file_as_bytes(path)
		var result: Dictionary = _bridge.as_import_item(target_uid, path.get_file(), bytes)
		if bool(result.get("ok", false)):
			if replace_first:
				_bridge.as_remove_item(target_uid, 0)
			_dirty = true
			_host.set_status("hint", "imported %s" % path.get_file(), "accent")
			_preview_index = 0
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

# -- slot grid ----------------------------------------------------------------

func _build_slot_grid() -> Control:
	var wrap := PanelContainer.new()
	wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"right": 1}))
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	wrap.add_child(col)

	var band := _band(col, 16, 14)
	_grid_header = DccTheme.mono_label("", "text_dim", DccTheme.FS_MICRO, 2, true)
	_grid_header.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_grid_header.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	band.add_child(_grid_header)
	_select_count_label = DccTheme.mono_label("0 SELECTED", "accent", DccTheme.FS_MICRO, 1, true)
	_select_count_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	band.add_child(_select_count_label)
	## The canvas folds the batch verbs into this band as quiet text rather
	## than giving them a row of filled slabs of their own.
	var verbs := HBoxContainer.new()
	verbs.add_theme_constant_override("separation", 2)
	verbs.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	band.add_child(verbs)
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
	_grid.columns = GRID_COLS
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
	var drop_hint := DccTheme.mono_label("drop-to-fill is not wired — use Import image…",
		"text_faint", DccTheme.FS_TINY)
	drop_hint.tooltip_text = "The canvas offers drag-and-drop onto a slot; no engine call backs it (as_import_item takes a path chosen in the file dialog). Said plainly rather than drawn as if it worked."
	drop_hint.mouse_filter = Control.MOUSE_FILTER_STOP
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
	wrap.custom_minimum_size.y = H_STATUS
	var pad := _pad(wrap, 16, 0, 16, 0)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 22)
	row.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	pad.add_child(row)
	_status_state = DccTheme.mono_label("", "text_faint", DccTheme.FS_TINY)
	_status_state.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	row.add_child(_status_state)
	_status_pack = DccTheme.mono_label("", "text_faint", DccTheme.FS_TINY)
	_status_pack.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	row.add_child(_status_pack)
	row.add_child(DccTheme.spacer())
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

func _refresh_selection_visuals() -> void:
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
	d.dialog_text = "Delete images of %d selected asset(s)? (custom slots are removed entirely; frozen slots are emptied, not removed.)" % uids.size()
	d.confirmed.connect(func():
		var result: Dictionary = _bridge.as_batch_delete(uids)
		_dirty = true
		_host.set_status("hint", "deleted %d asset(s)" % int(result.get("deleted", 0)), "accent")
		_selected.clear()
		_focused_uid = ""
		_refresh_grid()
		_refresh_inspector()
		_refresh_rail_counts()
		_refresh_status_line()
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered()

# ---------------------------------------------------------------------------
# Slot inspector
#
# Built once, refreshed in place. The previous version rebuilt every child on
# every selection change, which is why the pack-metadata fields needed a
# has_focus() guard to survive being typed into.
# ---------------------------------------------------------------------------

func _build_inspector() -> Control:
	var wrap := PanelContainer.new()
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

	var scale_row := _insp_row(rows, "Scale")
	_insp_scale = HSlider.new()
	_insp_scale.min_value = 10
	_insp_scale.max_value = 400
	_insp_scale.step = 1
	_insp_scale.value = 100
	_insp_scale.editable = false
	_insp_scale.focus_mode = Control.FOCUS_NONE
	_insp_scale.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_insp_scale.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	_insp_scale.custom_minimum_size.y = 14
	_insp_scale.tooltip_text = SCALE_GAP_NOTE
	_style_slider(_insp_scale)
	scale_row.add_child(_insp_scale)
	_insp_scale_readout = DccTheme.mono_label("—", "text", DccTheme.FS_TINY)
	_insp_scale_readout.custom_minimum_size.x = 38
	_insp_scale_readout.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	_insp_scale_readout.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	scale_row.add_child(_insp_scale_readout)

	var btn_row := HBoxContainer.new()
	btn_row.add_theme_constant_override("separation", 5)
	rows.add_child(btn_row)
	for gap_label in ["Fit", "Reset"]:
		var gb := _chip(btn_row, gap_label, Callable())
		gb.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
		gb.disabled = true
		gb.tooltip_text = SCALE_GAP_NOTE
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
	if item_count > 0:
		var item: Dictionary = _bridge.as_item_summary(_focused_uid, _preview_index)
		if bool(item.get("ok", false)):
			## The canvas's line is `capital-star.png · 512 × 512 · PNG · 84 KB`.
			## The engine reports no stored byte size (`as_item_summary` carries
			## name/transform/decoded size/hash and nothing else), so the last
			## field is dropped rather than invented; the rest is real.
			_insp_file.text = "%s · %d × %d · PNG" % [
				String(item.get("name", "")), int(item.get("w", 0)), int(item.get("h", 0))]
			_insp_file.tooltip_text = "Bakes to %s · pan (%.0f, %.0f) · content hash %s" % [
				bake_note, float(item.get("pan_x", 0.0)), float(item.get("pan_y", 0.0)),
				String(item.get("hash", "—"))]
			var pct := float(item.get("scale", 1.0)) * 100.0
			_insp_scale.value = clampf(pct, _insp_scale.min_value, _insp_scale.max_value)
			_insp_scale_readout.text = "%d%%" % int(roundf(pct))
		else:
			_insp_file.text = "—"
			_insp_scale_readout.text = "—"
	else:
		_insp_file.text = "no art in this slot"
		_insp_file.tooltip_text = "Bakes to %s once an image lands here." % bake_note
		_insp_scale.value = 100
		_insp_scale_readout.text = "—"

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
	add_child(_slicer)

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

	var body := HBoxContainer.new()
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
	_sheet_preview.custom_minimum_size = Vector2(0, H_SHEET_PREVIEW)
	_sheet_preview.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_sheet_preview.size_flags_vertical = Control.SIZE_EXPAND_FILL
	left_col.add_child(_sheet_preview)
	_sheet_readout = DccTheme.mono_label("no sheet chosen", "text_faint", DccTheme.FS_TINY)
	_sheet_readout.clip_text = true
	left_col.add_child(_sheet_readout)
	var choose := _chip(left_col, "Choose image…", func(): _pick_sheet_image())
	choose.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	body.add_child(left)

	# -- right: settings ------------------------------------------------------
	var right := PanelContainer.new()
	right.custom_minimum_size.x = W_SLICER_SIDE
	right.add_theme_stylebox_override("panel", DccTheme.panel("panel"))
	var right_pad := _pad(right, 16, 16, 16, 16)
	var side := VBoxContainer.new()
	side.add_theme_constant_override("separation", 10)
	right_pad.add_child(side)
	body.add_child(right)

	## 128 is the engine's own ceiling (`clampInt(v,1,128)`, ported as
	## `slicer::clamp_grid_count`), so the spinbox cannot ask for a grid the
	## engine would silently clamp behind the user's back.
	_slicer_cols = _slicer_number(side, "Columns", 1, 128, 6)
	_slicer_rows = _slicer_number(side, "Rows", 1, 128, 4)
	_slicer_margin = _slicer_number(side, "Margin", 0, 512, 0)
	_slicer_spacing = _slicer_number(side, "Spacing", 0, 256, 0)

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
	_sheet_readout.text = "%s · %d × %d" % [
		path.get_file(), int(result.get("w", 0)), int(result.get("h", 0))]
	_refresh_slicer_summary()

func _clear_sheet_preview() -> void:
	_sheet_image = null
	_sheet_loaded = false
	_sheet_preview.img_tex = null
	_sheet_preview.queue_redraw()
	_refresh_slicer_summary()

## The slicer modal's four numbers and three toggles, in `as_slice_preview`/
## `as_slice_apply`'s own `opts` shape. One builder for both calls, so the
## preview can never describe a different grid than the slice cuts.
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
	_sheet_preview.col_x0 = p.get("col_x0", PackedFloat64Array())
	_sheet_preview.col_x1 = p.get("col_x1", PackedFloat64Array())
	_sheet_preview.row_y0 = p.get("row_y0", PackedFloat64Array())
	_sheet_preview.row_y1 = p.get("row_y1", PackedFloat64Array())
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
