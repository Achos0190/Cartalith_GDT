extends AcceptDialog
class_name DataManagerWindow

## §9's Data manager window -- Data ▸ Import/Export/Sources/Validation's actual
## destination. `world_data_window.gd`'s own doc comment draws the line this
## file is the other side of: that window is the settlement/province/economy
## table browser (`Data ▸ World data tables…`, §9's own related-but-distinct
## sibling), unrelated to and untouched by this file.
##
## ## 2026-08-20: rebuilt against the design canvas, not against §9's prose
##
## Same story as `asset_library_window.gd` two commits ago, and found the same
## way. This file was written from `DCC_SHELL_SPEC.md` §9's *description* of the
## window before any of its export bindings were reachable; the 2026-08-20
## visual sweep then recorded **PASS** for it by checking that the routes worked
## and that the disclosures were honest -- never by laying the layout against
## `design/Cartalith DCC Shell.dc.html`'s `Data manager window 1920` screen. It
## did not match: a floating 920×600 `AcceptDialog` with an OS title bar and a
## stock OK button, a `§`-sigil routes rail of autowrapping flat buttons, and a
## route pane that showed one grey paragraph where the canvas designs seven
## labelled columns, an estimate block, a footer and a status line.
## `GUI_GAP_REGISTER.md` §14.2's row is corrected and §14.7 carries the delta
## list.
##
## The window is now laid out from that screen read as a literal spec: a
## full-bleed workspace window (borderless, sized under the app menu bar), a
## 34 px window bar, a 252 px routes rail with its own `ROUTES` band, a route
## pane whose header band is 28 px and whose body is the canvas's own
## `grid-template-columns:1fr 1fr; gap:0 34px`, an 11 px/18 px action footer and
## a 26 px status line. Every number below is off that canvas; every colour is a
## `DccTheme` token, so no hex appears here. The control vocabulary -- chip,
## segment, well, text button, band -- moved into `dcc_widgets.gd` in this pass,
## from where the Asset library rebuild left it (that file's own note: *"if a
## second window needs them, they move"*).
##
## ## One deliberate divergence from the canvas
##
## **The canvas still has a CONVERSION group** (Coordinate Systems / Format
## Conversion / Data Transformation) in its rail and in its subtitle. It
## predates the owner's 2026-08-20 decision to delete that group outright
## (`17ccc18`, `GUI_GAP_REGISTER.md` §7.4, DM-07/08/09 "resolved by deletion").
## It is **not** restored here. `GROUP_ORDER` is four, and the subtitle says
## four. This is the one place the shipped window intentionally does not follow
## the mockup.
##
## ## What is real vs. disclosed gap, route by route
##
## Most of §9 has no engine behind it, and this file says so per-route rather
## than building chrome that implies a capability that doesn't exist. What
## changed in this pass is *where* the disclosure lives: a route with no engine
## still shows the canvas's pane shape where one is designed, with the
## impossible controls disabled and carrying their reason as a tooltip, rather
## than a wall of prose replacing the layout.
##
## - **Import ▸ Heightmaps (PNG)** is real (DM-01): `DccApp.
##   open_heightmap_import()` → `EngineBridge.import_heightmap` →
##   `WorldGen::import_heightmap`, which decodes the PNG, takes it as the
##   elevation field and runs `cartalith_engine::import::infer_tectonics` under
##   it -- the reference's own `#loadBtn` + `#inferTectBtn` pair, golden-tested
##   (`cartalith-terrain/tests/golden_parity_infer.rs`). TIFF is absent and that
##   is parity, not a shortfall: the reference's file input is
##   `accept="image/*"`, decoded by a browser that does not read TIFF either.
## - **Import ▸ World Data (.zip · fields)** is real: it routes to the exact
##   same `bridge.load_save(path)` / `DccApp.open_project_picker()` path File ▸
##   Open project… already uses, not a second implementation.
## - **Import ▸ Assets** and **Export ▸ Assets** are real as routing shortcuts,
##   per §2.4's own table ("Assets (routes to the Assets menu)"):
##   `DccApp.open_asset_pack_picker()` and the Asset library window's own real
##   `export_pack_now()` (AS-04, `as_export_pack_bytes` → `archive::write_pack`).
## - **Export ▸ Maps** is real as of this pass (DM-02, partial -- see the
##   `EXPORT_SCHEME_NOTE` block below for exactly which half). `region_export_
##   tiles` was bound and golden-tested but had no caller; this window is now
##   that caller. It exports the **current Region-select marquee** as a zipped
##   `cols × rows` tile grid (`tiles/refined_{row}_{col}_rg16.bin`, plus
##   `tiles/refined_{row}_{col}.png` when visual tiles are on, plus
##   `tiles/index.json`). It is **not** a Leaflet XYZ pyramid, which is what the
##   canvas draws; every control the pyramid needs and this export does not have
##   is drawn and disabled with that reason.
## - **Export ▸ GIS / GeoJSON** is real as of this pass (DM-03). Same shape of
##   story as Export ▸ Maps: `cartalith_engine::geojson` was fully ported and
##   golden-verified character-for-character against the reference's own
##   document, with no `#[func]` binding and therefore no caller.
##   `geojson_bridge.rs` is the binding and this window is the caller. It
##   writes the **whole world**, not the marquee: settlements, ways, sea lanes,
##   rivers, territory and provinces, in local planar kilometres.
## - **Import ▸ Maps**, **Import ▸ GIS / GeoJSON**, **Export ▸ World Data**,
##   **Sources** and **Validation** are disclosed gaps: no tile-map or GeoJSON
##   *import*, no save writer (`cartalith-io` reads `.zip` saves; its only
##   `zip::ZipWriter` lives in its own `#[cfg(test)]` fixture builder), no
##   source registry and no validation pass exist anywhere in the workspace.
## - **Conversion is gone, not disclosed.** See above.

# ---------------------------------------------------------------------------
# Geometry, read off `Data manager window 1920`
# ---------------------------------------------------------------------------

const W_RAIL := 252         ## the canvas's `width:252px` routes rail
const H_BAR := 34           ## window bar
const H_BAND := 28          ## ROUTES band, route-pane header band
const H_STATUS := 26
const W_ROW_LABEL := 120    ## the pane's `width:120px` row label column
const PANE_PAD_X := 18      ## `padding:6px 18px 18px` on the pane body
const COL_GAP := 34         ## `gap:0 34px` between the two pane columns
const RAIL_PAD_X := 14
const RAIL_INDENT := 24     ## `padding:5px 14px 5px 24px` on a route row

# ---------------------------------------------------------------------------
# Routes
#
# `label` is the canvas's own short name; the qualifier it used to be
# concatenated with now lives in `badge`, the quiet right-hand column the canvas
# draws (`→ Assets`, `tiles`, `1`, `8`). `kind` is "live" (real control),
# "route" (a real shortcut into another menu) or "gap" (disclosed, no engine
# support -- `reason` is shown verbatim). `sub` is the header band's right-hand
# descriptor, the canvas's `web-map ready · XYZ scheme`.
#
# Import Maps and Import GIS / GeoJSON are two rows here, as on the canvas; they
# used to be one concatenated `Maps (tiles) · GIS / GeoJSON` row.
# ---------------------------------------------------------------------------

const ROUTES: Array[Dictionary] = [
	{"group": "Import", "id": "import_maps", "label": "Maps", "badge": "tiles", "kind": "gap",
		"sub": "no importer",
		"reason": "No tile-map import path exists. Nothing in the workspace reads a tile set back in. TIFF is also absent, and deliberately: the reference's own file input is accept=\"image/*\" and decodes through the browser, which does not decode TIFF either -- so PNG is parity, not a shortfall. Heightmap import itself is live; see the Heightmaps row."},
	{"group": "Import", "id": "import_heightmap", "label": "Heightmaps", "badge": "PNG", "kind": "live",
		"sub": "elevation + inferred tectonics"},
	{"group": "Import", "id": "import_gis", "label": "GIS / GeoJSON", "badge": "", "kind": "gap",
		"sub": "no importer",
		"reason": "No GeoJSON import path exists. cartalith-engine::geojson is write-only (export_geojson, golden-verified); nothing in the workspace parses a FeatureCollection back into places, ways or territory."},
	{"group": "Import", "id": "import_world", "label": "World Data", "badge": ".zip", "kind": "live",
		"sub": "same loader as File ▸ Open project…"},
	{"group": "Import", "id": "import_assets", "label": "Assets", "badge": "→ Assets", "kind": "route",
		"sub": "routes to the Assets menu"},
	{"group": "Export", "id": "export_maps", "label": "Maps", "badge": "tiles", "kind": "live",
		"sub": "region marquee · zipped tile grid"},
	{"group": "Export", "id": "export_gis", "label": "GIS / GeoJSON", "badge": ".geojson", "kind": "live",
		"sub": "whole world · planar km"},
	{"group": "Export", "id": "export_world", "label": "World Data", "badge": "map + atlas", "kind": "live",
		"sub": "whole world · 2K/4K/8K raster · channel atlas"},
	{"group": "Export", "id": "export_assets", "label": "Assets", "badge": ".zip", "kind": "route",
		"sub": "routes to the Asset library"},
	{"group": "Sources", "id": "sources_external", "label": "External Sources", "badge": "", "kind": "gap",
		"sub": "no registry", "reason": "No source registry exists anywhere in the workspace. §9 designs no pane for this route either -- GUI_GAP_REGISTER.md DM-06 is classed (C), needing a design before it can need code."},
	{"group": "Sources", "id": "sources_connected", "label": "Connected Sources", "badge": "", "kind": "gap",
		"sub": "no registry", "reason": "Same -- no source registry exists. The canvas's `1` badge on this row is mockup data, not a count this build could produce, so no badge is drawn."},
	{"group": "Sources", "id": "sources_registry", "label": "Source Registry", "badge": "", "kind": "gap",
		"sub": "no registry", "reason": "Same -- no source registry exists."},
	{"group": "Validation", "id": "val_check", "label": "Check Data", "badge": "", "kind": "gap",
		"sub": "no warning store",
		"reason": "load_save() returns pass/fail only (cartalith-godot's load_save binding) -- no warning collection exists anywhere to surface a count from. The canvas's `8` badge on this row is mockup data, so no badge is drawn. What would be validated, and against which invariant, is itself undefined (DM-10, classed (C))."},
	{"group": "Validation", "id": "val_repair", "label": "Repair / Normalize", "badge": "", "kind": "gap",
		"sub": "nothing to repair against", "reason": "No validation pass exists to repair against."},
]

## **Four groups, not five.** See the header's divergence note.
const GROUP_ORDER: Array[String] = ["Import", "Export", "Sources", "Validation"]

# ---------------------------------------------------------------------------
# Export ▸ Maps -- the disclosures the canvas's pyramid controls need
# ---------------------------------------------------------------------------

const SCHEME_NOTE := "The export writes a flat row/column tile grid plus tiles/index.json (cartalith_engine::region_export::export_region_tiles), not a slippy-map pyramid. XYZ, TMS and WMTS all address tiles by zoom/x/y over a projected CRS; none of that addressing exists in the engine, and adding it is DM-02's remaining half."

const CRS_NOTE := "The export is in the world's own cell grid. No CRS handling exists anywhere in the workspace -- reprojection was the substance of the Conversion group the owner deleted on 2026-08-20 (GUI_GAP_REGISTER.md §7.4), and no import or export path has carried a projection since."

const LAYER_NOTE := "region_export_tiles bakes elevation (RG16) and, with visual tiles on, a hillshaded colour raster. Political tint, labels/icons and rivers are drawn by render.rs into the live viewport texture and never reach the export path; compositing them into a tile is CA-04's separable-layer work, not a switch here."

const PRESET_NOTE := "No preset store exists. DccSettings persists storage roots and window state only; export presets would be a new section in it, and nothing reads one yet."

const VAULT_NOTE := "MARKDOWN_VAULT_INTEGRATION.md is owner-supplied design that is explicitly \"Not started; no code exists\", and its §33 lists two-way sync as a V1 non-goal. DM-14 is deferred by owner decision, so this block is drawn in the canvas's shape and quiet rather than accent -- there is no vault to be linked to."

const PACKAGING_NOTE := "zip_region_export always produces one stored (uncompressed) .zip. A loose folder tree and MBTiles are both new writers, and MBTiles additionally needs the XYZ addressing the scheme row above does not have."

# ---------------------------------------------------------------------------
# Export ▸ GIS / GeoJSON -- DM-03's own two disclosures
# ---------------------------------------------------------------------------

## The document carries this same sentence as its own `note` property, verbatim
## from the reference, so a consumer reading the file learns it too.
const GEOJSON_CRS_NOTE := "Coordinates are local planar kilometres (east, north) at this world's own scale, with north up -- not WGS84 longitude/latitude. RFC 7946 assumes WGS84, but a procedurally generated world has no true georeference; the reference makes the same call, and the document says so in its own note property."

const GEOJSON_CIV_NOTE := "Settlements, ways, territory and provinces come from the civilisation layer, which only exists for a freshly generated world -- a loaded .zip save carries none of the substrate that pipeline needs (SAVEFILE_COMPAT.md). Exporting a loaded save produces a valid document whose features are rivers and nothing else. This port also has no point-of-interest kind, so there is no poi layer: every place is a settlement."

## Tile-grid choices the engine accepts (`cols`/`rows`, any `n > 0`). The
## canvas's own row is a four-way `0–4 / 0–6 / 0–8 / custom` zoom segment; this
## is the same control over the dimension this export actually has.
const GRID_CHOICES: Array[int] = [2, 4, 8]
const TILE_SIZES: Array[int] = [256, 512, 1024]

# ---------------------------------------------------------------------------
# State
# ---------------------------------------------------------------------------

## Phone (§13) -- PH-07. Same shape and the same answer as
## `asset_library_window.gd`: a 252 px routes rail beside a pane does not fit
## 393 dp, so the two become panes behind a two-way switcher, and selecting a
## route moves to the route. `DccWidgets.phone_window()`'s header carries the
## general reasoning; what is specific here is that the pane body is rebuilt
## per route, so the fit has to be re-run (`_phone_refit()`).
var _phone := false
var _phone_pane := ""
var _phone_pane_buttons: Dictionary = {}
var _phone_rail: Control
var _phone_body: Control
var _phone_title: Label

var _host: DccApp
var _bridge: EngineBridge

var _rail_rows: Dictionary = {}   ## route id -> {button, label, badge, caret}
var _selected_id := ""

var _pane_title: Label
var _pane_sub: Label
var _pane_body: VBoxContainer     ## cleared and rebuilt per route
var _pane_footer: HBoxContainer
var _foot_dest: Label
var _foot_last_run: Label
var _status_left: Label
var _status_mid: Label

## Export ▸ Maps, live `region_export_tiles` opts. Defaults are the binding's
## own (`4`/`4`/`512`, gzip off, ridged off) except `visual`, which the binding
## defaults off and this window defaults **on** -- a map export whose tiles
## carry no colour is not what this route is for. That is a shell default, not
## an engine change.
var _tx_cols := 4
var _tx_rows := 4
var _tx_tile := 512
var _tx_gzip := false
var _tx_visual := true
var _tx_ridged := false
var _tx_dest := ""

## Export ▸ World Data, live `export_raster_png`/`export_channel_atlas` opts.
## `bakeRes`' own three widths with the reference's own default in the middle,
## and `bakeTiles` off -- both the reference's own initial state.
##
## The width list is asked of the binding (`export_raster_widths`) rather than
## written here, so the shell cannot offer a resolution the engine refuses.
const WD_WIDTH_FALLBACK: Array[int] = [2048, 4096, 8192]
var _wd_widths: Array[int] = WD_WIDTH_FALLBACK.duplicate()
var _wd_width := 4096
var _wd_tiled := false
## `layersPreviewChk` (reference line 555, read by `exportZip` at 12452).
## Off by default, exactly as the reference has it -- v0.92 made these
## opt-in on the grounds that nothing reads them back on load.
var _wd_layers := false

## Export ▸ World Data -- the two disclosures this route owes, and the one it
## no longer does. Until 2026-08-24 this row was a **gap** whose reason read
## "cartalith-io reads .zip saves but does not write them"; that stopped being
## true when FI-01 landed the writer, and the row outlived it.
const WD_RASTER_NOTE := "render::bake_rect runs the whole material path -- materials, hillshade, AO, the river tint, the paper ground and the plate frame -- at the fractional grid position each output pixel lands on, so an 8K export carries four times the material detail of a 2K one rather than the same picture resampled. Measured at the grid's own resolution against the live viewport: a dozen or so bytes of 8,060,928 differ, all by a single level, from the f32 prologue the reference stores in a Float32Array too."
const WD_TILES_NOTE := "Writes tile_{row}_{col}.png plus index.json (cartalith_io::build_tile_manifest) instead of one file. The raster is rendered ONCE either way and only the file layout differs, so this cannot change what the map looks like -- unlike the reference, which re-renders per tile because a browser canvas has a hard area cap no native build has."
const WD_ATLAS_NOTE := "chanAtlasChk: soil fertility, water access and carrying capacity in one RGB8 PNG; settlement suitability in another; the fifteen resource potentials three to a file; biome and lithology indices in a third -- plus atlas/index.json documenting which channel of which file holds which field. Data at grid resolution, not a picture. The Köppen channel is documented and left at zero: this port retains no Köppen raster, exactly as the reference leaves it null when state.climate.seasons never built one."
const WD_LAYERS_NOTE := "layersPreviewChk: the reference's own four human-viewable previews of the f32 data layers -- biome, hillshade, temperature, rainfall -- written into a layers/ folder beside whatever this run just wrote. Each is built from the pass the reference's own layerBytes(mode, debug) branch would have taken: bake_rect for biome, render::hillshade_raster for renderNow's mode==='shade' branch, and the temp/rain debug rasters, which are whole-image palette replacements rather than overlays because the reference's debugOpacity defaults to 1. Always at the GRID's size, not the raster width above: the .f32 blobs these preview are one value per cell, and the README line calls them reference only. Generated worlds only."
const WD_ZIP_NOTE := "This route writes loose files, not one project .zip. The save writer (cartalith_io::write_save, FI-01) and these two rasters are both real now; assembling exportZip's full archive -- params.json, the f32 layer blobs, map.png, the atlas and features.json in one file -- is the remaining third piece and is not wired here."

## Session-scoped run log -- `[{stamp, label, bytes, secs, ok}]`, newest first.
## DM-12 asks for the canvas's `last run 14:02 · 62 MB`; nothing persists a run
## history, so this is what is honestly available: the runs of *this* session.
var _runs: Array = []

# ---------------------------------------------------------------------------
# Setup
# ---------------------------------------------------------------------------

func setup(host: DccApp, bridge: EngineBridge) -> void:
	_host = host
	_bridge = bridge
	title = "⧉ DATA MANAGER"
	get_ok_button().hide()   ## the window bar's own Close chip replaces it.
	## The canvas draws this as a full-bleed workspace with its own 34 px window
	## bar, not a floating dialog with an OS title bar -- so the OS chrome comes
	## off and `_popup_full()` sizes it under the app menu bar, exactly as the
	## Asset library window does.
	borderless = true
	add_theme_stylebox_override("panel", DccTheme.panel("bg"))
	add_theme_constant_override("buttons_min_height", 0)
	add_theme_constant_override("margin", 0)
	## `AcceptDialog` turns `wrap_controls` on in its constructor, which makes
	## the window **grow** to its contents' minimum size on every
	## `child_controls_changed()` -- and only ever grow, never shrink back. A
	## window whose whole point is to be exactly the viewport minus the menu bar
	## must not do that: one oversized child min, even for a single frame,
	## permanently pushes the footer and status line past the bottom edge where
	## no scroll can reach them. Measured on this window before the fix: popped
	## correctly at 997 px, then grown to 2032 px by the rail footer's two
	## autowrap labels (which now carry a min width too -- see `_build_rail()`).
	wrap_controls = false
	size = Vector2i(1180, 760)
	min_size = Vector2i(1024, 640)
	## PH-07: rotation relay plus the "may I stack?" answer. Also re-asserts
	## `wrap_controls = false`, which this window already set for its own reason
	## above.
	_phone = DccWidgets.phone_window(self, host)
	_tx_dest = DccSettings.storage_root("exports").path_join("region-tiles.zip")
	_build()
	## `1.0`: `phone_present()` applies the scale once as `content_scale_factor`.
	if _phone:
		_host.phone_fit(self, 1.0)

## The canvas's own placement: the window occupies everything below the app menu
## bar, which is what "map hidden while open" means in a shell with no separate
## workspace stack for windows.
##
## **The size comes from the host Control's viewport rect, not from
## `get_tree().root.size`.** `Window.size` is the OS window in *physical*
## pixels; an embedded subwindow's `Rect2i` is in the parent viewport's *2D*
## coordinate space, and the two differ by the content scale on any HiDPI
## display. Measured here on a 200 %-scaled Windows desktop: `root.size.y` read
## 2066 against a 1031 px viewport, so this window was popped 2032 px tall
## inside a 1031 px space and its own footer and status line fell off the bottom
## edge -- invisible, and reachable by no scroll. `Control.get_viewport_rect()`
## is already in the right space. The Asset library window carried the identical
## bug (same code, copied) and is fixed the same way.
func _popup_full() -> void:
	## PH-07: a phone fills the whole screen -- §13 relocates the app menu bar
	## into the ⋯ overflow sheet, so there is nothing for this window to sit
	## under and the 34 px reserved for it is 125 physical px of nothing.
	if DccWidgets.phone_present(self, _host):
		return
	var vp: Vector2 = _host.get_viewport_rect().size if _host != null \
		else Vector2(get_tree().root.get_visible_rect().size)
	var top := DccTheme.H_MENU_BAR
	var w: int = maxi(int(vp.x), min_size.x)
	var h: int = maxi(int(vp.y) - top, min_size.y)
	popup(Rect2i(0, top, w, h))

## `group`, if given, selects that group's first route; empty selects the very
## first route overall. Both `menus.gd`'s four Data-menu group items and a bare
## "open the window" caller go through this one entry point.
func open(group: String = "") -> void:
	_popup_full()
	var target := ""
	if group != "":
		for r in ROUTES:
			if String(r["group"]) == group:
				target = String(r["id"])
				break
	if target == "" and not ROUTES.is_empty():
		target = String(ROUTES[0]["id"])
	if target != "":
		_select_route(target)
	_refresh_foot()
	_refresh_status()

## `right_dock.gd`'s Region select ▸ *Send to Data ▸ Export* (RD-09): open
## straight onto the tile-export route with the marquee already read.
func open_tile_export() -> void:
	_popup_full()
	_select_route("export_maps")
	_refresh_foot()
	_refresh_status()

# ---------------------------------------------------------------------------
# Layout -- window bar / rail · pane / status line
# ---------------------------------------------------------------------------

func _build() -> void:
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)

	outer.add_child(_build_window_bar())
	if _phone:
		outer.add_child(_build_phone_switcher())

	## PH-07: rail beside pane on a pointer, one at a time on a phone.
	var main: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	main.add_theme_constant_override("separation", 0)
	main.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(main)

	_phone_rail = _build_rail()
	_phone_body = _build_pane()
	main.add_child(_phone_rail)
	main.add_child(_phone_body)

	outer.add_child(_build_status_line())
	if _phone:
		_phone_title = DccWidgets.phone_head(outer, "Data manager",
			"import · export · sources · validation")
		_show_phone_pane("routes")

## PH-07, `asset_library_window.gd`'s switcher with two segments instead of
## three. See there for why this is a segmented row and not a `TabContainer`.
func _build_phone_switcher() -> Control:
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"bottom": 1}))
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 0)
	DccWidgets.pad(wrap, 8, 4, 8, 4).add_child(row)
	for spec in [["routes", "ROUTES"], ["route", "ROUTE"]]:
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

func _show_phone_pane(pane: String) -> void:
	if not _phone:
		return
	_phone_pane = pane
	_phone_rail.visible = pane == "routes"
	_phone_body.visible = pane == "route"
	for key in _phone_pane_buttons:
		var b: Button = _phone_pane_buttons[key]
		var on: bool = key == pane
		b.add_theme_stylebox_override("normal",
			DccTheme.flat(DccTheme.c("accent_wash")) if on else DccTheme.empty())
		b.add_theme_color_override("font_color",
			DccTheme.c("accent") if on else DccTheme.c("text_dim"))

## PH-07. The route pane is cleared and rebuilt on every `_select_route()`, so
## its rows have never been through `setup()`'s one-shot fit. Idempotent by
## meta-flag (`DccShell.phone_fit`), so re-walking the window only touches what
## the rebuild just made.
## Deferred, so it runs after the rebuild that triggered it has finished
## rather than in the middle of it.
func _phone_refit() -> void:
	if _phone and _host != null:
		_do_phone_refit.call_deferred()

func _do_phone_refit() -> void:
	if _phone and _host != null and is_instance_valid(self):
		_host.phone_fit(self, 1.0)

## `⧉ DATA MANAGER · import · export · sources · validation … Close ✕` -- the
## canvas's own 34 px bar. Four areas, not the canvas's five: see the header.
func _build_window_bar() -> Control:
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"bottom": 1}))
	if not _phone:
		wrap.custom_minimum_size.y = H_BAR
	var pad := DccWidgets.pad(wrap, 16, 0, 16, 0)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 14)
	row.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	pad.add_child(row)

	## PH-07: the title and its four-area subtitle are what `phone_head()` draws
	## in place of the title bar this borderless window gave up, so repeating
	## them here would be two headers -- and the subtitle's own disclosure lives
	## in a tooltip, which a phone cannot reach anyway. The bar keeps the one
	## thing that is not a caption: the way out.
	if not _phone:
		var title_label := DccTheme.mono_label("⧉ DATA MANAGER", "accent", DccTheme.FS_SMALL, 1)
		title_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		row.add_child(title_label)

		var sub := DccTheme.mono_label("import · export · sources · validation",
			"text_ghost", DccTheme.FS_SMALL)
		sub.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		sub.tooltip_text = "Four areas, not §9's five. The Conversion group (Coordinate Systems / Format Conversion / Data Transformation) was deleted on the owner's 2026-08-20 decision -- GUI_GAP_REGISTER.md §7.4 found no serious GIS application carries a top-level Conversion route, because reprojection belongs to the import or export step actually reading the file. The design canvas predates that decision and still shows it."
		sub.mouse_filter = Control.MOUSE_FILTER_STOP
		row.add_child(sub)

	row.add_child(DccTheme.spacer())

	var close_chip := DccWidgets.chip(row, "Close %s" % DccIcons.SYMBOLS["cross"],
		func(): hide(), false, 10, 4)
	close_chip.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	return wrap

# -- routes rail --------------------------------------------------------------

func _build_rail() -> Control:
	var wrap := PanelContainer.new()
	## PH-07: 252 px is 64% of a phone's 393 dp, and this is a full-width pane
	## there. The axis that has to expand changes with the axis it stacks on.
	if _phone:
		wrap.size_flags_vertical = Control.SIZE_EXPAND_FILL
	else:
		wrap.custom_minimum_size.x = W_RAIL
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"right": 1}))
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	wrap.add_child(col)

	var band := DccWidgets.band(col, RAIL_PAD_X, 9, H_BAND)
	var head := DccTheme.mono_label("ROUTES", "text_dim", DccTheme.FS_MICRO, 2, true)
	head.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	band.add_child(head)

	var scroll := _unpad_scroll(ScrollContainer.new())
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(scroll)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 0)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(body)

	var by_group: Dictionary = {}
	for r in ROUTES:
		var g := String(r["group"])
		if not by_group.has(g):
			by_group[g] = []
		(by_group[g] as Array).append(r)

	var first := true
	for g in GROUP_ORDER:
		## The canvas's `padding:9px 14px 4px` group header -- plain and tracked,
		## with none of `DccWidgets.section()`'s `§` sigil. That sigil is the
		## dock disclosure grammar's L3 marker; a window's routes rail is not a
		## dock section, and the canvas draws no sigil here.
		var gp := DccWidgets.pad(body, RAIL_PAD_X, 6 if first else 9, RAIL_PAD_X, 4)
		gp.add_child(DccTheme.mono_label(g.to_upper(), "text_ghost", DccTheme.FS_MICRO, 1))
		first = false
		for r in by_group.get(g, []):
			_rail_row(body, r)

	col.add_child(DccTheme.rule())
	var foot_pad := DccWidgets.pad(col, RAIL_PAD_X, 10, RAIL_PAD_X, 10)
	var foot := VBoxContainer.new()
	foot.add_theme_constant_override("separation", 3)
	foot_pad.add_child(foot)
	_foot_dest = DccTheme.mono_label("", "text_faint", DccTheme.FS_TINY)
	_foot_dest.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	## An autowrap Label with no minimum *width* reports a giant minimum
	## *height* -- it lays the text out at whatever width it currently has,
	## which before the first layout pass is zero. The Asset library rebuild
	## recorded this trap after a 1 700 px-tall slicer; here it grew the whole
	## window. The rail is `W_RAIL` wide with `RAIL_PAD_X` either side.
	_foot_dest.custom_minimum_size.x = _rail_text_w()
	foot.add_child(_foot_dest)
	## §9: "Foot: exports root and last run (`14:02 · 62 MB`)." Nothing persists
	## a run history (DM-12), so this reports the runs of *this session* and says
	## plainly when there are none, rather than inventing the canvas's timestamp.
	_foot_last_run = DccTheme.mono_label("", "text_ghost", DccTheme.FS_TINY)
	_foot_last_run.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_foot_last_run.custom_minimum_size.x = _rail_text_w()
	_foot_last_run.tooltip_text = "Session-scoped. No run history is persisted anywhere (DccSettings stores storage roots and window state only), so this resets when the app closes."
	foot.add_child(_foot_last_run)
	return wrap

## The width an autowrapping label in the rail foot must be given -- the rail's
## own text column. Fixed at `W_RAIL` on a pointer; on a phone the rail is the
## full 393 dp pane, and giving those two labels 224 dp there would wrap them at
## well under half the width they have.
##
## An autowrap `Label` with no minimum WIDTH reports a giant minimum HEIGHT (it
## lays the text out at whatever width it has, which before the first pass is
## zero), so the value matters in both directions -- see `_build_rail()`.
func _rail_text_w() -> int:
	if not _phone:
		return W_RAIL - RAIL_PAD_X * 2
	## `- 4`, not `- 0`: the rail's own `PanelContainer` draws a 1 px right
	## border and the window rounds its content scale, so the naive
	## `393 - 14 - 14` came out at a 394 dp minimum inside a 393 dp column --
	## one pixel, and enough to widen the window past the screen.
	return int(DccTheme.PHONE_REF_SHORT) - RAIL_PAD_X * 2 - 4

## The canvas's route row: `padding:5px 14px 5px 24px`, the short name, a quiet
## right-hand badge, and an accent `▸` on the selected row over an `accent_wash`
## ground. Three aligned parts on a `Button`, not one autowrapping label.
func _rail_row(parent: Control, route: Dictionary) -> void:
	var id := String(route["id"])
	var btn := Button.new()
	## Deliberately *not* `flat` -- a flat Button draws no stylebox at all, so
	## the canvas's `background:rgba(224,163,74,.09)` selected ground never
	## appears. `normal` is an empty box instead, which is what flat was for.
	btn.focus_mode = Control.FOCUS_NONE
	btn.custom_minimum_size.y = 22
	btn.add_theme_stylebox_override("normal", DccTheme.empty())
	btn.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	btn.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("accent_wash")))
	btn.pressed.connect(_select_route.bind(id))
	if String(route.get("kind", "gap")) == "gap":
		btn.tooltip_text = String(route.get("reason", ""))

	var row := HBoxContainer.new()
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_theme_constant_override("separation", 9)
	row.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	row.offset_left = RAIL_INDENT
	row.offset_right = -RAIL_PAD_X
	btn.add_child(row)

	var name_l := DccTheme.label(String(route["label"]), "text", DccTheme.FS_SMALL)
	name_l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	name_l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	name_l.clip_text = true
	name_l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(name_l)

	var badge := DccTheme.mono_label(String(route.get("badge", "")), "text_faint", DccTheme.FS_TINY)
	badge.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	badge.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(badge)

	var caret := DccTheme.mono_label(DccIcons.SYMBOLS["submenu"], "accent", DccTheme.FS_TINY)
	caret.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	caret.mouse_filter = Control.MOUSE_FILTER_IGNORE
	caret.visible = false
	row.add_child(caret)

	_rail_rows[id] = {"button": btn, "label": name_l, "badge": badge, "caret": caret}
	parent.add_child(btn)

# -- route pane ---------------------------------------------------------------

func _build_pane() -> Control:
	var wrap := VBoxContainer.new()
	wrap.add_theme_constant_override("separation", 0)
	wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if _phone:
		wrap.size_flags_vertical = Control.SIZE_EXPAND_FILL   ## PH-07, stacked

	var band := DccWidgets.band(wrap, PANE_PAD_X, 14, H_BAND)
	_pane_title = DccTheme.mono_label("", "text_dim", DccTheme.FS_MICRO, 2, true)
	_pane_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_pane_title.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	band.add_child(_pane_title)
	_pane_sub = DccTheme.mono_label("", "text_ghost", DccTheme.FS_MICRO)
	_pane_sub.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	band.add_child(_pane_sub)

	var scroll := _unpad_scroll(ScrollContainer.new())
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	wrap.add_child(scroll)

	var body_pad := DccWidgets.pad(scroll, PANE_PAD_X, 6, PANE_PAD_X, PANE_PAD_X)
	body_pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_pane_body = VBoxContainer.new()
	_pane_body.add_theme_constant_override("separation", 0)
	_pane_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body_pad.add_child(_pane_body)

	wrap.add_child(DccTheme.rule())
	var foot_pad := DccWidgets.pad(wrap, PANE_PAD_X, 11, PANE_PAD_X, 11)
	_pane_footer = HBoxContainer.new()
	_pane_footer.add_theme_constant_override("separation", 12)
	foot_pad.add_child(_pane_footer)

	return wrap

func _build_status_line() -> Control:
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"top": 1}))
	## PH-07: stacked and clipped on a phone, for the reason
	## `asset_library_window.gd`'s status line records -- two unclipped `Label`s
	## side by side report more minimum width than a 393 dp column has, and
	## `phone_fit()`'s ellipsis pass reaches only `Button`s. The `Esc` hint goes
	## with them: a phone has no Esc, and its way out is the Close chip above
	## plus the Android back gesture.
	wrap.custom_minimum_size.y = H_STATUS * 2 if _phone else H_STATUS
	var pad := DccWidgets.pad(wrap, 16, 0, 16, 0)
	var row: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	row.add_theme_constant_override("separation", 0 if _phone else 22)
	row.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	pad.add_child(row)
	_status_left = DccTheme.mono_label("idle · no pass running", "text_faint", DccTheme.FS_TINY)
	row.add_child(_status_left)
	_status_mid = DccTheme.mono_label("", "text_ghost", DccTheme.FS_TINY)
	row.add_child(_status_mid)
	if _phone:
		_status_left.clip_text = true
		_status_mid.clip_text = true
		return wrap
	row.add_child(DccTheme.spacer())
	row.add_child(DccTheme.mono_label("Esc close window", "text_ghost", DccTheme.FS_TINY))
	return wrap

# ---------------------------------------------------------------------------
# Pane row vocabulary -- the canvas's own `120px label · control` grammar
# ---------------------------------------------------------------------------

## `theme/dark_theme.tres` gives `ScrollContainer/styles/panel` a stylebox with
## `content_margin_left/right = 10`, `content_margin_top = 6`, a 1 px border and
## a **4 px corner radius** -- it reuses `SB_FieldDisabled`, an input-well box,
## for a container that draws no chrome on either canvas screen. Every scrolled
## region in the shell is therefore inset by 10 px against its own header band,
## which is what made this window's column headers sit 10 px right of the
## `EXPORT ▸ MAPS` band above them. Overridden per scroll region here rather
## than edited in the theme: the theme is shared with every dock, and a global
## change belongs in its own pass with its own visual check.
static func _unpad_scroll(s: ScrollContainer) -> ScrollContainer:
	s.add_theme_stylebox_override("panel", DccTheme.empty())
	return s

## The canvas's `font:9px mono; letter-spacing:.16em; padding:14px 0 6px`
## column header.
func _col_header(parent: Control, text: String, tip: String = "") -> Label:
	var p := DccWidgets.pad(parent, 0, 14, 0, 6)
	var l := DccTheme.mono_label(text, "text_ghost", DccTheme.FS_MICRO, 1)
	if tip != "":
		l.tooltip_text = tip
		l.mouse_filter = Control.MOUSE_FILTER_STOP
	p.add_child(l)
	return l

## The canvas's `display:flex;align-items:center;gap:10px;padding:4px 0` row,
## with its `width:120px` label column. Returns the row to fill.
func _row(parent: Control, label_text: String) -> HBoxContainer:
	var p := DccWidgets.pad(parent, 0, 4, 0, 4)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	p.add_child(row)
	if label_text != "":
		var l := DccTheme.label(label_text, "text_dim", DccTheme.FS_SMALL)
		l.custom_minimum_size.x = W_ROW_LABEL
		l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		row.add_child(l)
	return row

## A segment group -- `items` is `[{text, enabled, tip}]`, `on_pick` takes the
## index. The lit one is `selected`; a disabled-but-lit segment survives Godot
## resolving `disabled` ahead of `normal` (`DccWidgets.set_segment_on`).
func _segments(row: Control, items: Array, selected: int, on_pick: Callable) -> Array:
	var group := HBoxContainer.new()
	group.add_theme_constant_override("separation", 2)
	group.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(group)
	var out: Array = []
	for i in items.size():
		var item: Dictionary = items[i]
		var idx := i
		var b := DccWidgets.segment(group, String(item["text"]),
			func(): on_pick.call(idx))
		b.disabled = not bool(item.get("enabled", true))
		if String(item.get("tip", "")) != "":
			b.tooltip_text = String(item["tip"])
		DccWidgets.set_segment_on(b, i == selected)
		out.append(b)
	return out

## The canvas's read-only value well: `padding:4px 9px; border:1px solid`, mono.
func _well_label(row: Control, text: String, tip: String = "") -> Label:
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccWidgets.box("line", "", 9, 4))
	wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var l := DccTheme.mono_label(text, "text", DccTheme.FS_TINY)
	l.clip_text = true
	wrap.add_child(l)
	if tip != "":
		wrap.tooltip_text = tip
	row.add_child(wrap)
	return l

## The canvas's `☑ label … note` row. A borderless Button carries all three so
## the whole row is the hit target, matching the rail rows above.
func _check(parent: Control, text: String, value: bool, on_toggle: Callable,
		note: String = "", enabled: bool = true, tip: String = "") -> Button:
	var btn := Button.new()
	btn.focus_mode = Control.FOCUS_NONE
	btn.custom_minimum_size.y = 20
	btn.disabled = not enabled
	btn.add_theme_stylebox_override("normal", DccTheme.empty())
	btn.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	btn.add_theme_stylebox_override("pressed", DccTheme.empty())
	btn.add_theme_stylebox_override("disabled", DccTheme.empty())
	if tip != "":
		btn.tooltip_text = tip
		btn.mouse_filter = Control.MOUSE_FILTER_STOP

	var row := HBoxContainer.new()
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_theme_constant_override("separation", 10)
	row.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	btn.add_child(row)

	var glyph := DccTheme.mono_label(
		DccIcons.SYMBOLS["checked"] if value else DccIcons.SYMBOLS["unchecked"],
		"accent" if value else "text_ghost", DccTheme.FS_SMALL)
	glyph.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	glyph.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(glyph)

	var l := DccTheme.label(text, "text" if value and enabled else "text_dim", DccTheme.FS_SMALL)
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	l.clip_text = true
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(l)

	if note != "":
		var n := DccTheme.mono_label(note, "text_faint", DccTheme.FS_TINY)
		n.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		n.mouse_filter = Control.MOUSE_FILTER_IGNORE
		row.add_child(n)

	if enabled and on_toggle.is_valid():
		btn.pressed.connect(on_toggle)
	var p := DccWidgets.pad(parent, 0, 4, 0, 4)
	p.add_child(btn)
	return btn

## The canvas's bordered blocks: ESTIMATE (`1px solid rgba(255,255,255,.10)`)
## and MARKDOWN VAULT (`1px solid rgba(224,163,74,.35)` there, quiet here --
## nothing is linked). Returns the inner column.
func _block(parent: Control, border_token: String = "line") -> VBoxContainer:
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccWidgets.box(border_token, "", 14, 12))
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 5)
	wrap.add_child(col)
	parent.add_child(wrap)
	return col

## A `justify-content:space-between` line inside a block.
func _kv(parent: Control, key: String, value: String, token: String = "text") -> Label:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	var k := DccTheme.mono_label(key, "text_dim", DccTheme.FS_TINY)
	row.add_child(k)
	row.add_child(DccTheme.spacer())
	var v := DccTheme.mono_label(value, token, DccTheme.FS_TINY)
	row.add_child(v)
	parent.add_child(row)
	return v

# ---------------------------------------------------------------------------
# Route selection
# ---------------------------------------------------------------------------

func _route_by_id(id: String) -> Dictionary:
	for r in ROUTES:
		if String(r["id"]) == id:
			return r
	return {}

func _select_route(id: String) -> void:
	var route := _route_by_id(id)
	if route.is_empty():
		return
	_selected_id = id

	for rid in _rail_rows:
		var parts: Dictionary = _rail_rows[rid]
		var on: bool = rid == id
		var btn: Button = parts["button"]
		btn.add_theme_stylebox_override("normal",
			DccTheme.flat(DccTheme.c("accent_wash")) if on else DccTheme.empty())
		var lbl: Label = parts["label"]
		lbl.add_theme_color_override("font_color",
			DccTheme.c("text_bright") if on else
			(DccTheme.c("text_dim") if String(_route_by_id(rid).get("kind", "gap")) == "gap"
				else DccTheme.c("text")))
		(parts["caret"] as Label).visible = on

	_pane_title.text = "%s ▸ %s" % [String(route["group"]).to_upper(),
		String(route["label"]).to_upper()]
	_pane_sub.text = String(route.get("sub", ""))

	for c in _pane_body.get_children():
		_pane_body.remove_child(c)
		c.queue_free()
	for c in _pane_footer.get_children():
		_pane_footer.remove_child(c)
		c.queue_free()

	if id == "export_maps":
		_build_tile_export_pane()
	elif id == "export_world":
		_build_world_data_pane()
	else:
		_build_simple_pane(route)
	_refresh_status()
	## PH-07: picking a route in the ROUTES pane is a navigation whose whole
	## result is the pane next door, so the switcher follows it; and the pane it
	## just built is fresh nodes that have never been fitted.
	_phone_refit()
	_show_phone_pane("route")

## Every route §9 does not design a pane for: the canvas's own column-header
## grammar around whatever the route really is -- the live action, the routing
## shortcut, or the disclosed reason. One column, because there is one thing to
## say; the two-column grid belongs to the route the canvas actually designs.
func _build_simple_pane(route: Dictionary) -> void:
	var id := String(route["id"])
	var kind := String(route.get("kind", "gap"))
	## One column at roughly the width of the canvas's own `1fr` half, plus a
	## spacer -- prose set across the full 1 400 px pane is unreadable, and the
	## canvas never sets a line that long.
	var lane := HBoxContainer.new()
	lane.add_theme_constant_override("separation", COL_GAP)
	lane.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_pane_body.add_child(lane)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	## PH-07: 620 dp of measure inside a 393 dp column widens the window past
	## the screen. The reason for the number is "prose set across a 1 400 px
	## pane is unreadable" -- a phone's column is already narrower than any
	## measure this was protecting against, so it expands instead.
	if _phone:
		col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	else:
		col.custom_minimum_size.x = 620
	lane.add_child(col)
	if not _phone:
		lane.add_child(DccTheme.spacer())

	match id:
		"import_heightmap":
			_col_header(col, "HEIGHTMAP")
			if _bridge != null and _bridge.import_api:
				DccWidgets.note(col,
					"Reads a PNG heightmap (white = high), resamples it to the working grid at the image's own aspect ratio, and infers a tectonic substrate from its morphology so lithology, resources and settlement have something to read -- the reference's Import ▸ Load heightmap… followed by Infer tectonics from heightmap. Scale (width, peak) comes from New world…, exactly as the reference's own calibrate step reuses its generate form.")
				_footer_note("replaces the current elevation field")
				DccWidgets.chip(_pane_footer, "Import heightmap…", func():
					hide()
					_host.open_heightmap_import(), true, 16, 6)
			else:
				DccWidgets.note(col,
					"This build's GDExtension predates the heightmap-import binding (WorldGen::import_heightmap). Rebuild cartalith-godot to enable it.")
				_footer_note("binding missing in this build")
		"import_world":
			_col_header(col, "PROJECT ARCHIVE")
			DccWidgets.note(col,
				"Opens the same .zip project picker as File ▸ Open project… -- routed here per §9, not reimplemented.")
			_footer_note("replaces the whole world")
			DccWidgets.chip(_pane_footer, "Open project…", func():
				hide()
				_host.open_project_picker(), true, 16, 6)
		"import_assets":
			_col_header(col, "ASSET PACK")
			DccWidgets.note(col,
				"Routes to Assets ▸ Import asset pack .zip… -- §2.4's own table calls this item a shortcut, not a second implementation.")
			_footer_note("routes to the Assets menu")
			DccWidgets.chip(_pane_footer, "Import asset pack .zip…", func():
				hide()
				_host.open_asset_pack_picker(), true, 16, 6)
		"export_gis":
			## DM-03: `export_geojson` (geojson_bridge.rs) over
			## `cartalith_engine::geojson`, which is golden-verified
			## character-for-character against the reference's own document.
			_col_header(col, "FEATURE COLLECTION")
			DccWidgets.note(col,
				"Writes the whole world as one GeoJSON FeatureCollection: settlements, roads, sea lanes, rivers (Strahler order 2 and up), faction territory and provinces, each tagged with its own layer property. Not a region export -- the Region-select marquee is Export ▸ Maps' input, not this one's.")
			DccWidgets.note(col, GEOJSON_CRS_NOTE)
			DccWidgets.note(col, GEOJSON_CIV_NOTE)
			_footer_note("writes one .geojson file")
			DccWidgets.chip(_pane_footer, "Export .geojson…", func():
				_pick_geojson_destination(), true, 16, 6)
		"export_assets":
			## DM-05: routes to the Asset library window's own real Export pack
			## .zip… (AS-04, `as_export_pack_bytes` → `archive::write_pack`) --
			## §2.4's table calls this a shortcut, same as `import_assets`.
			_col_header(col, "ASSET PACK")
			DccWidgets.note(col,
				"Routes to the Asset library window's own Export pack .zip… (Assets ▸ ⧉ Asset library, §8's window bar) -- real (as_export_pack_bytes -> archive::write_pack).")
			_footer_note("routes to the Asset library")
			DccWidgets.chip(_pane_footer, "Export pack .zip…", func():
				hide()
				_host.open_asset_library()
				_host.asset_library_window.export_pack_now(), true, 16, 6)
		_:
			_col_header(col, "NOT BUILT")
			DccWidgets.note(col, String(route.get("reason", "Not implemented.")))
			_footer_note("nothing to run on this route")
	if kind == "gap":
		var disabled := DccWidgets.chip(_pane_footer, "Run", func(): pass, false, 16, 6)
		disabled.disabled = true
		disabled.tooltip_text = String(route.get("reason", ""))

func _footer_note(text: String) -> void:
	var l := DccTheme.mono_label(text, "text_faint", DccTheme.FS_TINY)
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	_pane_footer.add_child(l)
	_pane_footer.add_child(DccTheme.spacer())

# ---------------------------------------------------------------------------
# Export ▸ World Data -- the export raster and the channel atlas
# (`PARITY_AUDIT.md` §5 item 14, `GUI_GAP_REGISTER.md` DM-04)
#
# The reference puts these four controls in its header bar next to Export:
# `bakeRes` (2K/4K/8K), `bakeTiles`, `chanAtlasChk` and `layersPreviewChk`.
# This shell has no header-bar export strip, and §9 routes every export through
# this window -- so they live here, in the route the canvas already names for
# whole-world output, rather than in a fifth place.
#
# All four are real as of 2026-08-24. `layersPreviewChk` (human-viewable PNG
# previews of the f32 data layers) was the last one drawn disabled; it now
# writes the reference's own four PNGs into a `layers/` folder beside the
# raster export, at the grid's own size -- `WorldGen::export_layer_previews`.
# ---------------------------------------------------------------------------

func _build_world_data_pane() -> void:
	## PH-07: the canvas's two equal columns become one stacked column on a
	## phone -- `COL_GAP` apart, both `EXPAND_FILL`, they would each get half of
	## 393 dp and every `120px label · control` row inside them would overlap
	## rather than clip.
	var grid: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	grid.add_theme_constant_override("separation", COL_GAP)
	grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_pane_body.add_child(grid)

	var left := VBoxContainer.new()
	left.add_theme_constant_override("separation", 0)
	left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	grid.add_child(left)

	var right := VBoxContainer.new()
	right.add_theme_constant_override("separation", 0)
	right.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	grid.add_child(right)

	## Ask the engine which widths it offers rather than trusting the local
	## fallback -- a build whose binding predates this pane answers nothing,
	## and then the whole pane must say so instead of offering dead buttons.
	var api := _raster_api()
	if api:
		var got: PackedInt32Array = _bridge.world_gen.export_raster_widths()
		if got.size() > 0:
			_wd_widths.clear()
			for w in got:
				_wd_widths.append(int(w))
			if not _wd_widths.has(_wd_width):
				_wd_width = _wd_widths[_wd_widths.size() / 2]

	_build_wd_raster_column(left, api)
	_build_wd_atlas_column(left, api)
	_build_wd_output_column(right, api)
	_build_wd_footer(api)

## True when this build's GDExtension carries the bindings **and** there is a
## world to export. Both halves matter and they fail differently, so the two
## messages below are separate.
func _raster_api() -> bool:
	return (_bridge != null and _bridge.world_gen != null
		and _bridge.world_gen.has_method("export_raster_png")
		and _bridge.world_gen.has_method("export_channel_atlas"))

func _build_wd_raster_column(col: Control, api: bool) -> void:
	_col_header(col, "MAP RASTER", WD_RASTER_NOTE)

	if not api:
		DccWidgets.note(col,
			"This build's GDExtension predates the export-raster binding (WorldGen::export_raster_png). Rebuild cartalith-godot to enable it.")
		return

	## `bakeRes` -- 2K / 4K / 8K, labelled the way the reference labels them.
	var res_row := _row(col, "Resolution")
	var items: Array = []
	for w in _wd_widths:
		items.append({"text": "%dK" % int(round(float(w) / 1024.0)), "enabled": true})
	_segments(res_row, items, _wd_widths.find(_wd_width), func(i: int):
		_wd_width = _wd_widths[i]
		_rebuild_world_data())

	## `bakeDims` -- the real output size, read back from the engine rather
	## than recomputed here, so the shell can never disagree with what the
	## file will actually be.
	var est := _wd_estimate()
	var dim_row := _row(col, "Output size")
	if est.is_empty():
		_well_label(dim_row, "no world",
			"Generate or load a world first -- bake_dims needs the grid to keep the export at the world's own aspect ratio.")
	else:
		_well_label(dim_row, "%d × %d px" % [int(est.get("width", 0)), int(est.get("height", 0))],
			"WorldGen::export_raster_estimate -> render::bake_dims, the reference's own Math.round(W*GH/GW).")

	_check(col, "Write as %d px tiles" % int(est.get("tile_size", 1024)), _wd_tiled, func():
		_wd_tiled = not _wd_tiled
		_rebuild_world_data(),
		("%d files" % int(est.get("tiles", 0))) if _wd_tiled and not est.is_empty() else "",
		true, WD_TILES_NOTE)

	## `layersPreviewChk` -- real since 2026-08-24. Four PNGs at the *grid's*
	## own size (not the raster width above), written into a `layers/` folder
	## beside whatever the raster export just wrote: biome, hillshade,
	## temperature, rainfall -- the reference's own four, from the passes its
	## own `layerBytes(mode, debug)` branches would have taken.
	if not _bridge.world_gen.has_method("export_layer_previews"):
		DccWidgets.note(col,
			"This build's GDExtension predates the layer-preview binding (WorldGen::export_layer_previews). Rebuild cartalith-godot to enable it.")
		return
	var gen := _bridge != null and _bridge.has_world
	var layers_note := ""
	if gen and not est.is_empty():
		layers_note = "4 PNGs · %d × %d" % [int(_bridge.world_gen.get_width()), int(_bridge.world_gen.get_height())]
	var layers_row := _check(col, "Human-viewable f32 layer previews", _wd_layers, func():
		_wd_layers = not _wd_layers
		_rebuild_world_data(),
		layers_note, gen, WD_LAYERS_NOTE)
	if not gen and layers_row != null:
		layers_row.tooltip_text = ("Generate a world first.\n\n" + WD_LAYERS_NOTE)

func _build_wd_atlas_column(col: Control, api: bool) -> void:
	_col_header(col, "CHANNEL ATLAS", WD_ATLAS_NOTE)
	if not api:
		DccWidgets.note(col, "Same -- rebuild cartalith-godot for WorldGen::export_channel_atlas.")
		return
	var gen := _bridge != null and _bridge.has_world
	## "8 PNGs" is the measured count, not an estimate: habitat, settlement,
	## the fifteen resource potentials three to a file, and classes -- plus
	## atlas/index.json, which is not a PNG and is not counted here.
	_check(col, "Habitat · settlement · resources · classes", true, func(): pass,
		"8 PNGs" if gen else "", false,
		"Every group the reference's channelAtlasGroups builds, and there is no option to omit one: an atlas missing a documented channel is worse than no atlas. A group whose every channel is empty is dropped rather than written black -- channel_atlas::entries' own rule.")
	if not gen:
		DccWidgets.note(col,
			"A loaded .zip save carries none of the tectonic substrate these fields are derived from (SAVEFILE_COMPAT.md), which is the same reason its civilisation layer is absent. The atlas needs a generated world.")

func _build_wd_output_column(col: Control, api: bool) -> void:
	_col_header(col, "OUTPUT")
	var dest_row := _row(col, "Folder")
	_well_label(dest_row, DccSettings.storage_root("exports"),
		"DccSettings' own exports root -- the same folder Export ▸ Maps and Export ▸ GIS write into.")

	var est := _wd_estimate()
	if api and not est.is_empty():
		var peak_row := _row(col, "Peak memory")
		_well_label(peak_row, _fmt_bytes(int(est.get("peak_bytes", 0))),
			"3 bytes per output pixel for the raster plus 12 for the local-contrast pass' luma and its two blur buffers. Reported by the binding, not modelled here -- at 8K it is worth seeing before you press the button.")
		var px_row := _row(col, "Pixels")
		_well_label(px_row, "%.1f MP" % (float(est.get("pixels", 0)) / 1_000_000.0))

	_col_header(col, "NOT THIS ROUTE")
	DccWidgets.note(col, WD_ZIP_NOTE)

	_build_recent_runs(col)

func _build_wd_footer(api: bool) -> void:
	var gen := _bridge != null and _bridge.has_world
	if not api:
		_footer_note("binding missing in this build")
		return
	_footer_note("writes into %s" % DccSettings.storage_root("exports"))
	var atlas := DccWidgets.chip(_pane_footer, "Export channel atlas…", func():
		_pick_atlas_destination(), false, 16, 6)
	atlas.disabled = not gen
	if not gen:
		atlas.tooltip_text = "Generate a world first -- the atlas' fields are all derived from the tectonic substrate."
	var go := DccWidgets.chip(_pane_footer, "Export %dK map…" % int(round(float(_wd_width) / 1024.0)), func():
		_pick_raster_destination(), true, 16, 6)
	go.disabled = _bridge == null or not _bridge.has_world
	go.tooltip_text = ("export_raster_png -> render::bake_rect, written with std::fs. "
		+ "Synchronous: an 8K export is seconds of work and the window will not repaint while it runs.")

func _wd_estimate() -> Dictionary:
	if not _raster_api():
		return {}
	return _bridge.world_gen.export_raster_estimate(_wd_width)

func _rebuild_world_data() -> void:
	if _selected_id == "export_world":
		_select_route("export_world")

func _pick_raster_destination() -> void:
	var d := FileDialog.new()
	d.access = FileDialog.ACCESS_FILESYSTEM
	d.current_dir = DccSettings.storage_root("exports")
	if _wd_tiled:
		## Tiled mode writes a *directory* of tiles plus index.json, so the
		## picker asks for one -- the reference's own tiles/ prefix, as a real
		## folder rather than a path inside a zip.
		d.title = "Export map tiles into…"
		d.file_mode = FileDialog.FILE_MODE_OPEN_DIR
		d.dir_selected.connect(func(path: String):
			_run_raster_export(path)
			d.queue_free())
	else:
		d.title = "Export map raster"
		d.file_mode = FileDialog.FILE_MODE_SAVE_FILE
		d.add_filter("*.png ; PNG image")
		## `exportZip`'s own name for this file.
		d.current_file = "map.png"
		d.file_selected.connect(func(path: String):
			_run_raster_export(path)
			d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered_ratio(0.6)

func _pick_atlas_destination() -> void:
	var d := FileDialog.new()
	d.title = "Export channel atlas into…"
	d.file_mode = FileDialog.FILE_MODE_OPEN_DIR
	d.access = FileDialog.ACCESS_FILESYSTEM
	d.current_dir = DccSettings.storage_root("exports")
	d.dir_selected.connect(func(path: String):
		_run_atlas_export(path)
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered_ratio(0.6)

## Both exports write from Rust with `std::fs`, so the path handed across has
## to be a real OS path -- `globalize_path` is a no-op for one already, and the
## difference only shows up for a `user://` root, which `DccSettings` can
## legitimately hold.
func _run_raster_export(path: String) -> void:
	if not _raster_api():
		return
	_status_left.text = "baking %dK…" % int(round(float(_wd_width) / 1024.0))
	_status_left.add_theme_color_override("font_color", DccTheme.c("accent"))
	var r: Dictionary = _bridge.world_gen.export_raster_png(
		ProjectSettings.globalize_path(path), _wd_width, _wd_tiled)
	_record_wd_run("map %dK" % int(round(float(_wd_width) / 1024.0)), r)
	if bool(r.get("ok", false)):
		var files: PackedStringArray = r.get("files", PackedStringArray())
		_host.set_status("hint", "exported %d × %d px%s — %s in %.1f s"
			% [int(r.get("width", 0)), int(r.get("height", 0)),
				(" as %d tiles" % files.size()) if _wd_tiled else "",
				_fmt_bytes(int(r.get("bytes", 0))), float(r.get("ms", 0.0)) / 1000.0], "accent")
	else:
		_host.set_status("hint", "export failed — %s" % String(r.get("error", "see the Godot log")), "warn")
	if bool(r.get("ok", false)) and _wd_layers:
		_run_layer_previews(path)
	_rebuild_world_data()
	_refresh_foot()
	_refresh_status()

## `layersPreviewChk`'s half of `exportZip`: four grid-resolution PNGs under a
## `layers/` folder, written *beside* the raster the run above just produced.
## The tiled route already picked a directory, so that is the base; the single
## route picked a file, so its own directory is.
func _run_layer_previews(path: String) -> void:
	if _bridge == null or _bridge.world_gen == null or not _bridge.world_gen.has_method("export_layer_previews"):
		return
	var base := path if _wd_tiled else path.get_base_dir()
	_status_left.text = "writing layer previews…"
	_status_left.add_theme_color_override("font_color", DccTheme.c("accent"))
	var r: Dictionary = _bridge.world_gen.export_layer_previews(ProjectSettings.globalize_path(base))
	_record_wd_run("layers", r)
	if bool(r.get("ok", false)):
		_host.set_status("hint", "…and 4 layer previews at %d × %d (%s)"
			% [int(r.get("width", 0)), int(r.get("height", 0)), _fmt_bytes(int(r.get("bytes", 0)))], "accent")
	else:
		_host.set_status("hint", "layer previews failed — %s" % String(r.get("error", "see the Godot log")), "warn")

func _run_atlas_export(dir: String) -> void:
	if not _raster_api():
		return
	_status_left.text = "packing channel atlas…"
	_status_left.add_theme_color_override("font_color", DccTheme.c("accent"))
	var r: Dictionary = _bridge.world_gen.export_channel_atlas(ProjectSettings.globalize_path(dir))
	_record_wd_run("atlas", r)
	if bool(r.get("ok", false)):
		var files: PackedStringArray = r.get("files", PackedStringArray())
		_host.set_status("hint", "exported %d atlas files (%s) in %.1f s"
			% [files.size(), _fmt_bytes(int(r.get("bytes", 0))), float(r.get("ms", 0.0)) / 1000.0], "accent")
	else:
		_host.set_status("hint", "atlas export failed — %s" % String(r.get("error", "see the Godot log")), "warn")
	_rebuild_world_data()
	_refresh_foot()
	_refresh_status()

func _record_wd_run(label: String, r: Dictionary) -> void:
	var t := Time.get_time_dict_from_system()
	_runs.push_front({
		"stamp": "%02d:%02d" % [int(t["hour"]), int(t["minute"])],
		"label": label, "bytes": int(r.get("bytes", 0)),
		"secs": float(r.get("ms", 0.0)) / 1000.0, "ok": bool(r.get("ok", false)),
	})
	while _runs.size() > 3:
		_runs.pop_back()

# ---------------------------------------------------------------------------
# Export ▸ Maps -- §9's one fully-designed route pane (DM-13)
#
# The canvas's `grid-template-columns:1fr 1fr; gap:0 34px`: TILES / PROJECTION /
# LAYERS INCLUDED down the left column, OUTPUT / ESTIMATE / MARKDOWN VAULT /
# RECENT RUNS down the right, then a footer of `Save as preset · Dry run ·
# Export N tiles`.
#
# `region_export_tiles` is the engine behind it (bound, golden-tested by
# `cartalith-engine`'s own `region_export` tests, and until this pass callerless
# -- `right_dock.gd`'s Region select ▸ *Send to Data ▸ Export* said so). What it
# does **not** do is the pyramid the canvas draws; every control that needs the
# pyramid is drawn in its canvas position and disabled with its reason.
# ---------------------------------------------------------------------------

func _build_tile_export_pane() -> void:
	## PH-07: the canvas's two equal columns become one stacked column on a
	## phone -- `COL_GAP` apart, both `EXPAND_FILL`, they would each get half of
	## 393 dp and every `120px label · control` row inside them would overlap
	## rather than clip.
	var grid: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	grid.add_theme_constant_override("separation", COL_GAP)
	grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_pane_body.add_child(grid)

	var left := VBoxContainer.new()
	left.add_theme_constant_override("separation", 0)
	left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	grid.add_child(left)

	var right := VBoxContainer.new()
	right.add_theme_constant_override("separation", 0)
	right.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	grid.add_child(right)

	var region := _bridge.region_get() if _bridge != null else {}

	_build_tiles_column(left)
	_build_projection_column(left, region)
	_build_layers_column(left)

	_build_output_column(right)
	_build_estimate_block(right, region)
	_build_vault_block(right)
	_build_recent_runs(right)

	_build_tile_export_footer(region)

func _build_tiles_column(col: Control) -> void:
	_col_header(col, "TILES")

	var scheme_row := _row(col, "Scheme")
	_segments(scheme_row, [
		{"text": "grid + index.json", "enabled": true},
		{"text": "XYZ", "enabled": false, "tip": SCHEME_NOTE},
		{"text": "TMS", "enabled": false, "tip": SCHEME_NOTE},
		{"text": "WMTS", "enabled": false, "tip": SCHEME_NOTE},
	], 0, func(_i: int): pass)

	## The canvas's `Zoom range 0–4 / 0–6 / 0–8 / custom` row, over the dimension
	## this export actually has: `cols`/`rows`, a flat grid with no zoom ladder.
	var grid_row := _row(col, "Tile grid")
	var grid_items: Array = []
	for n in GRID_CHOICES:
		grid_items.append({"text": "%d×%d" % [n, n], "enabled": true})
	var grid_sel := GRID_CHOICES.find(_tx_cols) if _tx_cols == _tx_rows else -1
	_segments(grid_row, grid_items, grid_sel, func(i: int):
		_tx_cols = GRID_CHOICES[i]
		_tx_rows = GRID_CHOICES[i]
		_rebuild_tile_export())

	var size_row := _row(col, "Tile size")
	var size_items: Array = []
	for n in TILE_SIZES:
		size_items.append({"text": "%d px" % n, "enabled": true})
	_segments(size_row, size_items, TILE_SIZES.find(_tx_tile), func(i: int):
		_tx_tile = TILE_SIZES[i]
		_rebuild_tile_export())

	var fmt_row := _row(col, "Format")
	_well_label(fmt_row, "RG16 .bin" + (" + PNG" if _tx_visual else ""),
		"Every tile is written as a 16-bit RG height raster (tiles/refined_{row}_{col}_rg16.bin). With visual tiles on, a colour PNG is written alongside it. Both names, and tiles/index.json, come from cartalith_engine::region_export.")

	_check(col, "Visual colour + hillshade tiles", _tx_visual, func():
		_tx_visual = not _tx_visual
		_rebuild_tile_export(),
		"×2 files" if _tx_visual else "", true,
		"region_export_tiles' own `visual` option -- the shaded colour raster render.rs draws, baked per tile at the world's real sea level. Off, only the RG16 height data is written.")

	_check(col, "Gzip the height tiles", _tx_gzip, func():
		_tx_gzip = not _tx_gzip
		_rebuild_tile_export(),
		".bin.gz", true,
		"region_export_tiles' own `gzip` option. The archive itself is stored, not deflated, so this is where compression actually happens.")

	_check(col, "Ridged detail amplification", _tx_ridged, func():
		_tx_ridged = not _tx_ridged
		_rebuild_tile_export(),
		"", true,
		"cartalith_terrain::amplify's `ridged` flag, the same detail pass the deep-zoom LOD tiles use. Off is the binding's own default.")

	_check(col, "Skip all-ocean tiles", false, func(): pass, "", false,
		"No tile is skipped: export_region_tiles writes every cell of the cols × rows grid unconditionally. Detecting an all-ocean tile would need a per-tile sea test before the amplify pass, which the export path does not do.")

func _build_projection_column(col: Control, region: Dictionary) -> void:
	_col_header(col, "PROJECTION", CRS_NOTE)

	var crs_row := _row(col, "CRS")
	_segments(crs_row, [
		{"text": "world cell grid", "enabled": true},
		{"text": "EPSG:3857", "enabled": false, "tip": CRS_NOTE},
		{"text": "EPSG:4326", "enabled": false, "tip": CRS_NOTE},
	], 0, func(_i: int): pass)

	var bounds_row := _row(col, "World bounds")
	if region.is_empty():
		_well_label(bounds_row, "no region selected",
			"Arm the Region select tool (R) and drag a marquee on the map. region_export_tiles exports that marquee and nothing else -- with none set it returns an empty archive, so Export stays disabled.")
	else:
		_well_label(bounds_row, "%d %d · %d × %d cells" % [
			int(region.get("x", 0)), int(region.get("y", 0)),
			int(region.get("w", 0)), int(region.get("h", 0))],
			"The live Region-select marquee (WorldGen::region_get). §4.5.1: the marquee and this route's bounds are two views of one rect, not two states.")
		var km_row := _row(col, "Extent")
		_well_label(km_row, "%.0f × %.0f km" % [
			float(region.get("w_km", 0.0)), float(region.get("h_km", 0.0))])

	_check(col, "Write world file (.wld + .prj)", false, func(): pass, "", false, CRS_NOTE)

func _build_layers_column(col: Control) -> void:
	_col_header(col, "LAYERS INCLUDED", LAYER_NOTE)
	_check(col, "Elevation (RG16)", true, func(): pass, "always", false,
		"Every export writes the height tiles; there is no option to omit them.")
	_check(col, "Relief + hillshade", _tx_visual, func():
		_tx_visual = not _tx_visual
		_rebuild_tile_export(), "", true,
		"The same `visual` option as the TILES column above -- the canvas lists it in both places, so both are drawn and both drive the one flag.")
	_check(col, "Political tint", false, func(): pass, "", false, LAYER_NOTE)
	_check(col, "Labels & icons", false, func(): pass, "raster", false, LAYER_NOTE)
	_check(col, "Rivers & coastlines", false, func(): pass, "", false, LAYER_NOTE)

func _build_output_column(col: Control) -> void:
	_col_header(col, "OUTPUT")

	var dest_row := _row(col, "Destination")
	_well_label(dest_row, _tx_dest if _tx_dest != "" else "—", _tx_dest)
	DccWidgets.chip(dest_row, "Choose…", func(): _pick_destination(), false, 8, 3)

	var pack_row := _row(col, "Packaging")
	_segments(pack_row, [
		{"text": ".zip", "enabled": true},
		{"text": "folder", "enabled": false, "tip": PACKAGING_NOTE},
		{"text": "MBTiles", "enabled": false, "tip": PACKAGING_NOTE},
	], 0, func(_i: int): pass)

	_check(col, "Emit tiles/index.json", true, func(): pass, "always", false,
		"export_region_tiles always writes tiles/index.json -- the per-tile file names, dimensions and world metadata. It is not optional.")
	_check(col, "Emit leaflet-preview.html", false, func(): pass, "", false, SCHEME_NOTE)
	_check(col, "Emit style.json + attribution", false, func(): pass, "", false, SCHEME_NOTE)

func _build_estimate_block(col: Control, region: Dictionary) -> void:
	_col_header(col, "ESTIMATE")
	var block := _block(col)
	var tiles := _tx_cols * _tx_rows
	var files := tiles * (2 if _tx_visual else 1) + 1
	_kv(block, "tiles", "%d (%d × %d)" % [tiles, _tx_cols, _tx_rows])
	_kv(block, "files in archive", "%d" % files)
	_kv(block, "tile size", "%d × %d px" % [_tx_tile, _tx_tile])
	var last := _last_run()
	_kv(block, "size on disk",
		_fmt_bytes(int(last.get("bytes", 0))) if not last.is_empty() else "measured by Dry run",
		"text" if not last.is_empty() else "text_ghost")
	_kv(block, "render time",
		("%.1f s" % float(last.get("secs", 0.0))) if not last.is_empty() else "measured by Dry run",
		"text" if not last.is_empty() else "text_ghost")
	if region.is_empty():
		_kv(block, "source", "no region selected", "accent")
	else:
		_kv(block, "source", "marquee · %d × %d cells" % [
			int(region.get("w", 0)), int(region.get("h", 0))])
	## The canvas's `~ 214 MB` and `~ 3 min 40 s` are a size *model*; this port
	## has none, and inventing one would be exactly the kind of plausible fiction
	## the rest of this window avoids. Dry run measures both for real instead.
	var note := DccTheme.label(
		"Size and time are measured, not modelled -- Dry run performs the whole export and reports what it produced without writing it.",
		"text_ghost", DccTheme.FS_MICRO)
	note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	note.custom_minimum_size.x = 200
	block.add_child(note)

func _build_vault_block(col: Control) -> void:
	_col_header(col, "MARKDOWN VAULT · NOT LINKED", VAULT_NOTE)
	## The canvas borders this block in accent (`rgba(224,163,74,.35)`) because
	## its mockup vault *is* linked. Nothing is linked here and nothing can be,
	## so it is drawn quiet -- the shape without the claim.
	var block := _block(col)
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 9)
	head.add_child(DccTheme.mono_label(DccIcons.SYMBOLS["off"], "text_ghost", DccTheme.FS_TINY))
	var path := DccTheme.mono_label("no vault linked", "text_dim", DccTheme.FS_TINY)
	path.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(path)
	head.add_child(DccTheme.mono_label("0 notes", "text_ghost", DccTheme.FS_TINY))
	block.add_child(head)

	var prose := DccTheme.label(
		"MARKDOWN_VAULT_INTEGRATION.md is owner-supplied design with no code behind it. Settlements, factions and journeys would resolve to notes by name, and exported tiles would carry obsidian:// links -- none of that is built, and DM-14 is deferred by owner decision.",
		"text_ghost", DccTheme.FS_MICRO)
	prose.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	prose.custom_minimum_size.x = 200
	block.add_child(prose)

	_check(block, "Two-way sync (write place notes back)", false, func(): pass,
		"V1 non-goal", false, VAULT_NOTE)
	_check(block, "Link labels to notes in GeoJSON", false, func(): pass, "", false, VAULT_NOTE)
	_check(block, "Include front-matter as properties", false, func(): pass, "", false, VAULT_NOTE)

	var btns := HBoxContainer.new()
	btns.add_theme_constant_override("separation", 6)
	block.add_child(btns)
	for t in ["Re-scan vault", "Change folder…", "Unlink"]:
		var b := DccWidgets.chip(btns, t, func(): pass, false, 0, 6)
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
		b.disabled = true
		b.tooltip_text = VAULT_NOTE

func _build_recent_runs(col: Control) -> void:
	_col_header(col, "RECENT RUNS")
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 5)
	col.add_child(body)
	if _runs.is_empty():
		DccWidgets.note(body,
			"No export has run in this session. Nothing persists a run history (DM-12), so this list starts empty every launch rather than showing the canvas's invented 14:02 · 62 MB.")
		return
	for r in _runs:
		var run: Dictionary = r
		_kv(body, "%s · %s" % [String(run.get("stamp", "")), String(run.get("label", ""))],
			"%s %s" % [_fmt_bytes(int(run.get("bytes", 0))),
				DccIcons.SYMBOLS["tick"] if bool(run.get("ok", false)) else DccIcons.SYMBOLS["cross"]],
			"text_dim" if bool(run.get("ok", false)) else "accent")

func _build_tile_export_footer(region: Dictionary) -> void:
	var ready: bool = not region.is_empty()
	_footer_note("writes to %s" % (_tx_dest if _tx_dest != "" else "—"))

	var preset := DccWidgets.chip(_pane_footer, "Save as preset", func(): pass, false, 14, 6)
	preset.disabled = true
	preset.tooltip_text = PRESET_NOTE

	var dry := DccWidgets.chip(_pane_footer, "Dry run", func(): _run_export(true), false, 14, 6)
	dry.disabled = not ready
	dry.tooltip_text = ("Runs the whole export and reports the tile count and archive size without writing a file."
		if ready else "No region marquee is set. Arm the Region select tool (R) and drag one on the map.")

	var go := DccWidgets.chip(_pane_footer, "Export %d tiles" % (_tx_cols * _tx_rows),
		func(): _run_export(false), true, 16, 6)
	go.disabled = not ready
	go.tooltip_text = ("region_export_tiles -> zip_region_export, written with FileAccess."
		if ready else "No region marquee is set. Arm the Region select tool (R) and drag one on the map.")

## The pane is small enough that rebuilding it on a toggle is cheaper than
## threading a refresh through every control, and it keeps the estimate, the
## footer's tile count and the Format well in one consistent state.
func _rebuild_tile_export() -> void:
	if _selected_id == "export_maps":
		_select_route("export_maps")

# ---------------------------------------------------------------------------
# Export ▸ Maps -- the run
# ---------------------------------------------------------------------------

func _pick_destination() -> void:
	var d := FileDialog.new()
	d.title = "Export tiles .zip"
	d.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	d.access = FileDialog.ACCESS_FILESYSTEM
	d.add_filter("*.zip ; Tile archive")
	d.current_dir = _tx_dest.get_base_dir() if _tx_dest != "" else DccSettings.storage_root("exports")
	d.current_file = _tx_dest.get_file() if _tx_dest != "" else "region-tiles.zip"
	d.file_selected.connect(func(path: String):
		_tx_dest = path
		_rebuild_tile_export()
		_refresh_foot()
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered_ratio(0.6)

# ---------------------------------------------------------------------------
# Export ▸ GIS / GeoJSON -- the run (DM-03)
#
# One picker, one write. There is no options pane because the binding has no
# options: `export_geojson` describes the whole world, and every layer it can
# emit it always emits.
# ---------------------------------------------------------------------------

func _pick_geojson_destination() -> void:
	var d := FileDialog.new()
	d.title = "Export GeoJSON"
	d.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	d.access = FileDialog.ACCESS_FILESYSTEM
	d.add_filter("*.geojson ; GeoJSON FeatureCollection")
	d.current_dir = DccSettings.storage_root("exports")
	## The reference names its own download `world_{seed}.geojson`; the shell
	## has no seed of its own to interpolate, and the document carries it as a
	## property anyway.
	d.current_file = "world.geojson"
	d.file_selected.connect(func(path: String):
		_run_geojson_export(path)
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered_ratio(0.6)

func _run_geojson_export(path: String) -> void:
	if _bridge == null:
		return
	_status_left.text = "exporting…"
	_status_left.add_theme_color_override("font_color", DccTheme.c("accent"))

	var t0 := Time.get_ticks_msec()
	var text := _bridge.export_geojson()
	var secs := float(Time.get_ticks_msec() - t0) / 1000.0

	if text.is_empty():
		_record_geojson_run(0, secs, false)
		_host.set_status("hint",
			"export failed — no world is loaded, or this build's GDExtension predates export_geojson", "warn")
	else:
		var f := FileAccess.open(path, FileAccess.WRITE)
		if f == null:
			_record_geojson_run(text.length(), secs, false)
			_host.set_status("hint",
				"export failed — could not open %s for writing" % path.get_file(), "warn")
		else:
			f.store_string(text)
			f.close()
			_record_geojson_run(text.to_utf8_buffer().size(), secs, true)
			_host.set_status("hint", "exported %s (%s)"
				% [path.get_file(), _fmt_bytes(text.to_utf8_buffer().size())], "accent")
	_refresh_foot()
	_refresh_status()

func _record_geojson_run(bytes: int, secs: float, ok: bool) -> void:
	var t := Time.get_time_dict_from_system()
	_runs.push_front({
		"stamp": "%02d:%02d" % [int(t["hour"]), int(t["minute"])],
		"label": "geojson", "bytes": bytes, "secs": secs, "ok": ok,
	})
	while _runs.size() > 3:
		_runs.pop_back()

## `dry` performs the identical export and reports what it produced without
## writing it -- which is how the ESTIMATE block gets a real size and time
## instead of a modelled one.
func _run_export(dry: bool) -> void:
	if _bridge == null:
		return
	if _bridge.region_get().is_empty():
		_host.set_status("hint", "export failed — no region marquee is set", "warn")
		return
	if not dry and _tx_dest == "":
		_pick_destination()
		return

	_status_left.text = "exporting…"
	_status_left.add_theme_color_override("font_color", DccTheme.c("accent"))

	var t0 := Time.get_ticks_msec()
	var bytes: PackedByteArray = _bridge.region_export_tiles({
		"cols": _tx_cols, "rows": _tx_rows, "tile_size": _tx_tile,
		"gzip": _tx_gzip, "ridged": _tx_ridged, "visual": _tx_visual,
	})
	var secs := float(Time.get_ticks_msec() - t0) / 1000.0

	if bytes.is_empty():
		_record_run(dry, 0, secs, false)
		_host.set_status("hint",
			"export failed — region_export_tiles returned no bytes (see the Godot log)", "warn")
		_rebuild_tile_export()
		_refresh_foot()
		_refresh_status()
		return

	if dry:
		_record_run(true, bytes.size(), secs, true)
		_host.set_status("hint", "dry run — %d tiles, %s, %.1f s (nothing written)"
			% [_tx_cols * _tx_rows, _fmt_bytes(bytes.size()), secs], "accent")
	else:
		var f := FileAccess.open(_tx_dest, FileAccess.WRITE)
		if f == null:
			_record_run(false, bytes.size(), secs, false)
			_host.set_status("hint",
				"export failed — could not open %s for writing" % _tx_dest.get_file(), "warn")
		else:
			f.store_buffer(bytes)
			f.close()
			_record_run(false, bytes.size(), secs, true)
			_host.set_status("hint", "exported %s (%s)"
				% [_tx_dest.get_file(), _fmt_bytes(bytes.size())], "accent")

	_rebuild_tile_export()
	_refresh_foot()
	_refresh_status()

func _record_run(dry: bool, bytes: int, secs: float, ok: bool) -> void:
	var t := Time.get_time_dict_from_system()
	_runs.push_front({
		"stamp": "%02d:%02d" % [int(t["hour"]), int(t["minute"])],
		"label": "%s %d×%d z%d%s" % ["dry run" if dry else "tile grid",
			_tx_cols, _tx_rows, _tx_tile, "" if _tx_visual else " (height only)"],
		"bytes": bytes, "secs": secs, "ok": ok,
	})
	while _runs.size() > 3:
		_runs.pop_back()

func _last_run() -> Dictionary:
	for r in _runs:
		if bool((r as Dictionary).get("ok", false)):
			return r
	return {}

func _fmt_bytes(n: int) -> String:
	if n <= 0:
		return "—"
	if n < 1024:
		return "%d B" % n
	if n < 1048576:
		return "%.1f kB" % (n / 1024.0)
	return "%.1f MB" % (n / 1048576.0)

# ---------------------------------------------------------------------------
# Rail footer + status line
# ---------------------------------------------------------------------------

func _refresh_foot() -> void:
	if not is_instance_valid(_foot_dest):
		return
	## The canvas's foot is one line (`exports → ~/Cartalith/Exports`). A real
	## Windows `app_userdata` root is four lines wrapped into a 224 px rail, so
	## the last two segments are shown and the whole path is the tooltip.
	var root := DccSettings.storage_root("exports")
	var parts := root.replace("\\", "/").split("/", false)
	var short := root if parts.size() < 2 else \
		".../%s/%s" % [parts[parts.size() - 2], parts[parts.size() - 1]]
	_foot_dest.text = "exports → %s" % short
	_foot_dest.tooltip_text = root
	_foot_dest.mouse_filter = Control.MOUSE_FILTER_STOP
	var last := _last_run()
	_foot_last_run.text = ("no export has run yet" if last.is_empty()
		else "last run %s · %s" % [String(last.get("stamp", "")),
			_fmt_bytes(int(last.get("bytes", 0)))])

func _refresh_status() -> void:
	if not is_instance_valid(_status_left):
		return
	_status_left.text = "idle · no pass running"
	_status_left.add_theme_color_override("font_color", DccTheme.c("text_faint"))
	if _selected_id != "export_maps":
		_status_mid.text = ""
		return
	var region := _bridge.region_get() if _bridge != null else {}
	_status_mid.text = ("no region marquee — arm Region select (R) and drag one"
		if region.is_empty()
		else "marquee %d × %d cells · %d tiles queued" % [
			int(region.get("w", 0)), int(region.get("h", 0)), _tx_cols * _tx_rows])
	_status_mid.add_theme_color_override("font_color",
		DccTheme.c("accent") if region.is_empty() else DccTheme.c("text_faint"))
