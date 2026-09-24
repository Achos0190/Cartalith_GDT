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
## ## 2026-09-05: the twelve undrawn routes get a pattern
##
## That canvas designs **one** route pane -- Export ▸ Maps -- and says nothing
## about the other thirteen, which is why every one of them was a column header
## over a paragraph of prose. `design/proposed-2026-09-05/DataPane.dc.html`
## (owner-approved: *"I like the layouts as proposed, implement those"*) draws a
## second pane, and its value is that it is a **pattern** rather than a
## fourteenth bespoke design: title, one-line purpose, a form region, a
## destination row, a receipt, an action row. `_build_pattern_pane()` and its
## parts are that pattern, and every route except the two with fully-designed
## bodies now renders through it -- see that function's own header for what the
## pattern keeps of this window's frame and what it does not.
##
## ## 2026-09-06: Validation ▸ Definitions (DM-10) -- the finding is the screen
##
## **Cartalith cannot ship QGIS's geometry validator, and the reason is not
## that nobody has written one.** Measured at the symbols this pass:
##
## - `poly_self_intersects` (`cartalith-urban/src/geom.rs`) is `pub` inside its
##   own crate and has exactly two callers -- `blocks.rs:407` and
##   `geom.rs::inset_poly` -- both of which are *city blocks*, not territory,
##   coastline or province rings. Nothing crosses the gdext boundary.
## - In `cartalith-spatial/src/geo.rs` **an unclosed ring is normal output**,
##   not a defect: the module doc's own item 2 says "a traced ring is not
##   necessarily closed", `ring_area` iterates `i < len - 1` and deliberately
##   omits the closing segment, and `tests/golden_parity_geo.rs` asserts the
##   unclosed shape twice ("still six points, still unclosed"). Reporting it as
##   an error would be **wrong**, not merely absent.
## - Ring orientation is checked nowhere. `ensure_ccw` *imposes* a winding; no
##   function anywhere reports one as invalid.
##
## So the Map geometry group is drawn **dashed, carrying that reason**, and the
## route validates what the engine actually validates: **definitions**. Five
## validators, and every one of them was already `#[func]`-bound and already
## forwarded through `EngineBridge` before this pass -- `tl_list(kind)` carries
## `validation_state`/`validation_missing`/`validation_conflicts` per row, and
## `as_validate()` returns the asset library's ordered warning strings. **This
## route needed no new boundary and none was added**; what was missing was a
## caller.
##
## Three shapes the table holds because the engine holds them, not because a
## design preferred them:
##
## - **`FIELDS` is empty for asset rows.** `AssetLibrarySession::validate()`
##   returns `Vec<String>`; the subject is inside the sentence, not a payload.
##   Dashed with that reason rather than filled with a guess.
## - **`LOCATE` is present and disabled on every row.** No row here is a placed
##   thing -- a definition has no coordinates -- and
##   `viewport_host.gd::move_view_to()` takes grid cells.
##   The column stays so the table already has the right shape the day a
##   positional validator exists. Not omitted, and not wired.
## - **There is no FIX SELECTED.** The engine has validators only; nothing here
##   changes state. `RESOLUTION` therefore names the window that edits the
##   definition, which is a navigation, not a repair.
##
## `RUN ALL VALIDATORS` simply rebuilds the pane: all five are pure functions
## over state already in memory, so re-running them costs nothing and cannot
## fail differently.
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
##   `tiles/index.json`). **Since 2026-09-23 the scheme row's XYZ / TMS / WMTS
##   segments are live too**: they export the whole world's LOD pyramid through
##   `slippy_export_tiles` (`cartalith_engine::slippy_export`), with the
##   canvas's zoom range and retina toggle, and (2026-09-23) a
##   `leaflet-preview.html` viewer page inside the same archive. What the
##   canvas draws and neither export has -- a CRS, MBTiles/folder packaging,
##   style.json, ocean skipping, overlay layers -- is still drawn disabled with
##   its reason (`SCHEME_NOTE` and the notes beside it).
## - **Export ▸ GIS / GeoJSON** is real as of this pass (DM-03). Same shape of
##   story as Export ▸ Maps: `cartalith_engine::geojson` was fully ported and
##   golden-verified character-for-character against the reference's own
##   document, with no `#[func]` binding and therefore no caller.
##   `geojson_bridge.rs` is the binding and this window is the caller. It
##   writes the **whole world**, not the marquee: settlements, ways, sea lanes,
##   rivers, territory and provinces, in local planar kilometres.
## - **Export ▸ World Data** is real, and this bullet used to say it was not.
##   *"No save writer -- `cartalith-io` reads `.zip` saves; its only
##   `zip::ZipWriter` lives in its own `#[cfg(test)]` fixture builder"* was
##   already false when `WD_ZIP_NOTE` below was written: `cartalith_io::write_
##   save` (`save.rs`) and `project.rs`'s own `ZipWriter` are both shipping
##   code, and this route's pane has drawn a live raster/heightmap/atlas export
##   since 2026-08-24. Corrected 2026-09-05, having survived two passes that
##   edited the constants three hundred lines below it.
## - **Validation ▸ Definitions** is real as of 2026-09-06 (DM-10) -- see the
##   next header section. **Validation ▸ Check Data** and **Repair / Normalize**
##   stay disclosed gaps beside it, and the distinction is the whole point:
##   what exists validates *definitions*, not world state.
## - **Import ▸ Maps**, **Import ▸ GIS / GeoJSON** and **Sources** are disclosed
##   gaps: no tile-map import and no source registry exist anywhere in the
##   workspace. *This bullet used to say the same of the whole Validation group,
##   on the ground that "no validation pass exists anywhere in the workspace".
##   Measured 2026-09-06, that is false: `validate_animal`, `validate_vehicle`,
##   `validate_vessel`, `validate_party_preset` and
##   `AssetLibrarySession::validate()` all ship, are all `#[func]`-bound
##   (`tl_list`, `as_validate`) and are all forwarded through `EngineBridge`.
##   What was missing was a caller, not an engine. When the clause stopped being
##   true was not established and is not asserted here.*
##   GeoJSON is the
##   one that landed later: since `d79d776` its row imports for real
##   (`_run_geojson_import` -> `apply_geojson_document`, over
##   `cartalith_io::parse_geojson`). The bound `WorldGen::geojson_inspect`
##   still has no GDScript caller -- the import applies with no preview step.
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

## The width the pane footer's note keeps for itself when the action chips have
## taken everything else. **This is a floor on a control that is otherwise
## `SIZE_EXPAND_FILL` beside chips that are not**, which is the shape
## `MISTAKES.md` records as making a trimmed `Label` vanish outright: an
## ellipsised `Label` reports a minimum width of 1, and 1 px is what it then
## gets. See `_footer_note()` for the measurement that made this row a
## constraint rather than a nicety.
##
## The ceiling on it is arithmetic, not taste, and it is why 160 rather than
## 250: at the window's own declared `min_size.x` of 1024 the footer row has
## `1024 - W_RAIL - PANE_PAD_X * 2` = 736 px, and the widest chip set in the
## window (`export_world`'s three, 441 px plus 48 px of separation = 489)
## leaves 247. Anything above that puts the footer back in front of the pane
## body as the window's binding minimum, which is the defect this constant was
## added to end. 160 also stays under the widest *body* minimum measured across
## all fifteen routes (698 px, `export_maps`), so the body stays the constraint
## and a future route may add a control without the footer silently deciding
## the window's width.
const FOOT_NOTE_MIN_W := 160

# ---------------------------------------------------------------------------
# The route-pane PATTERN, read off `design/proposed-2026-09-05/DataPane.dc.html`
# (owner-approved 2026-09-05, *"I like the layouts as proposed, implement
# those"*).
#
# `Data manager window 1920` designed exactly one route pane -- Export ▸ Maps --
# and the other thirteen were never drawn. `DataPane.dc.html` draws a **second**
# one (its exemplar is an Export ▸ World data that writes entities), and its
# value is that it is a *pattern*: title, a one-line purpose, a form region
# (FORMAT · INCLUDE · EXTENT), a destination row with `Browse…`, a receipt, an
# action row. Every route that was a wall of prose now renders through it, so
# the eleven undrawn routes inherit a design instead of each needing one.
#
# Two of its details are rules rather than decoration, and both are load-bearing
# here:
#
#   * **The receipt states what was NOT included.** The artboard's reads
#     `landmarks and religions were not included — 2 groups off`. A receipt that
#     only reports success hides the thing the user needs to notice --
#     `_pattern_receipt()` computes that line from the groups this format does
#     not carry, plus the carried groups the written document turned out to hold
#     none of.
#   * **A chip with no data is dashed and dimmed, not hidden** (`religions —`
#     beside live counts). `_include_chips()` has three states for that reason,
#     not two.
#
# **The artboard's contents are illustrative and its numbers are not this
# engine's.** Every count below is asked of a real source or drawn as a dash
# carrying its reason; see `GIS_GROUPS` and `_gis_count()`.
#
# ## Where this pane and the window frame disagree, and why the frame wins
#
# The artboard draws no header band -- its pane opens straight onto the title.
# This window has one (`H_BAND`, `_pane_title`/`_pane_sub`), it belongs to the
# *window* rather than to any pane, and Export ▸ Maps and Export ▸ World data
# are laid out under it. Removing it is a window-level change this artboard does
# not scope and would leave those two panes headerless, so the band stays and
# the pattern draws the artboard's title inside the body. The two carry
# different strings: the band is the breadcrumb (`EXPORT ▸ GIS / GEOJSON`) plus
# `ROUTES.sub`, the body title is the route's own name plus its purpose line.
# ---------------------------------------------------------------------------

## `width:74px` on the artboard's FORMAT / INCLUDE / EXTENT / TO label column --
## a different, narrower column from the Export ▸ Maps pane's `W_ROW_LABEL`.
const PATTERN_LABEL_W := 74
## `.chip` -- `padding:3px 11px;border-radius:999px`. Byte-identical in
## `Modal.dc.html`, which is why the two artboards' shared `_shared.css`
## vocabulary is treated as one system here.
const PATTERN_CHIP_R := 999
const PATTERN_CHIP_PAD_X := 11
const PATTERN_CHIP_PAD_Y := 3
## The FORMAT control is a **track**: `background:var(--ins);border-radius:14px;
## padding:2px`, with the lit segment `border-radius:12px;padding:3px 12px;
## background:var(--wash2);color:var(--acc)` inside it. That is a different
## shape from the outline segments `_segments()` draws for Export ▸ Maps'
## Scheme / CRS / Packaging rows, and the difference is the artboard's, not a
## reinterpretation -- see `_pattern_segments()`.
const PATTERN_TRACK_R := 14
const PATTERN_TRACK_PAD := 2
const PATTERN_SEG_R := 12
const PATTERN_SEG_PAD_X := 12
const PATTERN_SEG_PAD_Y := 3
## The receipt block: `background:var(--ins);border-radius:8px;padding:9px 11px`.
## The radius is `DccWidgets.MODAL_INSET_RADIUS`, whose own comment names the
## same `border-radius:8px` from the same `.tok` block; only the paddings are
## local.
const PATTERN_RECEIPT_PAD_X := 11
const PATTERN_RECEIPT_PAD_Y := 9
## `color:var(--ink);font-weight:500;font-size:14px` on the pane title. **No
## `DccTheme` role carries 14 px as a font size** -- `role_px("fs_prose")`
## reaches 14 only as its *tablet* rung, which is a density answer and not this
## one -- so it is a local constant read off the artboard, exactly as `W_RAIL`
## and `H_BAND` above are. The 500 weight is not reachable: the sans face this
## shell loads has one weight, and only `DccTheme.mono()` has a medium cut.
const PATTERN_TITLE_FS := 14
## `margin:12px 0 12px` on the first `.rule`, `margin:13px 0 12px` on the
## second.
const PATTERN_RULE_TOP := 12
const PATTERN_RULE_TOP_2 := 13
const PATTERN_RULE_BOTTOM := 12

## The one-line purpose under each route's title -- the artboard's
## `every generated entity as one file — settlements, factions, ways,
## landmarks`, written per route.
##
## Keyed by `ROUTES.id`, and kept beside `ROUTES` rather than inside it because
## the route list itself is settled (`GROUP_ORDER` is four; see the header's
## divergence note) and this pass draws the pane rather than re-deciding the
## routes. `_datapane_probe.gd` asserts every `ROUTES` id has an entry, so a
## route added later cannot ship without one.
##
## Prose, not data: each line describes what the route actually does today, and
## every one was checked against the code it names in this pass.
const PANE_PURPOSE := {
	"import_maps": "read a tile set back into the world — no importer exists",
	"import_heightmap": "a PNG becomes the elevation field, with a tectonic substrate inferred under it",
	"import_gis": "read a FeatureCollection into the world — settlements and faction territory are placed; ways, rivers, POIs and provinces are counted but not yet placed",
	"import_world": "a .ctl project archive replaces the whole world — the same loader as File ▸ Open project…",
	"import_assets": "routes to Assets ▸ Import asset pack .zip…",
	"export_maps": "the Region-select marquee as a zipped tile grid, or the whole world as an XYZ / TMS / WMTS tile pyramid",
	"export_gis": "every generated entity as one document — settlements, ways, rivers, territory, provinces",
	"export_world": "the whole world as a colour raster, a heightmap and a channel atlas",
	"export_assets": "routes to the Asset library's own Export pack .zip…",
	"sources_external": "point the project at data held outside it — no source registry exists",
	"sources_connected": "what this project is currently reading from — no source registry exists",
	"sources_registry": "the list of every known source — no source registry exists",
	"val_defs": "run every validator the engine has over the definitions it has — nothing is written back",
	"val_check": "look for contradictions in the world's own data — nothing collects warnings",
	"val_repair": "fix what a check found — no world check exists yet to repair against",
}

## Export ▸ GIS / GeoJSON's INCLUDE row: the entity groups this window can name,
## and whether `export_geojson` actually carries each.
##
## `carried` is read off `geojson_bridge.rs::export_geojson` and
## `cartalith_engine::geojson::export_geojson`, which between them emit exactly
## **five** `properties.layer` values: `settlement`, `way`, `river`,
## `territory`, `province`. (This said *six*, counting `poi`, until a verifier
## re-parsed a written document on 2026-09-05 and measured five. `poi` is a
## layer the exporter *can* express and this port never emits -- see the note
## below -- so it is not one of the values a document actually carries.)
## There is no landmark layer and no religion layer, so
## those two are drawn dim -- present in the world, absent from the file. That
## is the artboard's own `landmarks 214` state, and it is true here rather than
## illustrative.
##
## `poi` is emitted by the engine and never produced by this port (`is_poi:
## false` for every place, `GEOJSON_CIV_NOTE`), so it is not a group a user
## could include or exclude and is not listed.
const GIS_GROUPS: Array[Dictionary] = [
	{"key": "settlements", "carried": true},
	{"key": "factions", "carried": true},
	{"key": "ways", "carried": true},
	{"key": "rivers", "carried": true},
	{"key": "provinces", "carried": true},
	{"key": "landmarks", "carried": false},
	{"key": "religions", "carried": false},
]

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
	{"group": "Import", "id": "import_gis", "label": "GIS / GeoJSON", "badge": "", "kind": "live",
		"sub": "settlements + territory · unknown factions created"},
	{"group": "Import", "id": "import_world", "label": "World Data", "badge": ".ctl", "kind": "live",
		"sub": "same loader as File ▸ Open project…"},
	{"group": "Import", "id": "import_assets", "label": "Assets", "badge": "→ Assets", "kind": "route",
		"sub": "routes to the Assets menu"},
	{"group": "Export", "id": "export_maps", "label": "Maps", "badge": "tiles", "kind": "live",
		"sub": "marquee grid · or world pyramid, XYZ/TMS/WMTS"},
	{"group": "Export", "id": "export_gis", "label": "GIS / GeoJSON", "badge": ".geojson", "kind": "live",
		"sub": "whole world · planar km"},
	{"group": "Export", "id": "export_world", "label": "World Data", "badge": "map + atlas", "kind": "live",
		"sub": "whole world · 2K-32K raster · channel atlas"},
	{"group": "Export", "id": "export_assets", "label": "Assets", "badge": ".zip", "kind": "route",
		"sub": "routes to the Asset library"},
	{"group": "Sources", "id": "sources_external", "label": "External Sources", "badge": "", "kind": "gap",
		"sub": "no registry", "reason": "No source registry exists anywhere in the workspace. §9 designs no pane for this route either -- GUI_GAP_REGISTER.md DM-06 is classed (C), needing a design before it can need code."},
	{"group": "Sources", "id": "sources_connected", "label": "Connected Sources", "badge": "", "kind": "gap",
		"sub": "no registry", "reason": "Same -- no source registry exists. The canvas's `1` badge on this row is mockup data, not a count this build could produce, so no badge is drawn."},
	{"group": "Sources", "id": "sources_registry", "label": "Source Registry", "badge": "", "kind": "gap",
		"sub": "no registry", "reason": "Same -- no source registry exists."},
	{"group": "Validation", "id": "val_defs", "label": "Definitions", "badge": "", "kind": "live",
		"sub": "five validators · definitions, not geometry"},
	{"group": "Validation", "id": "val_check", "label": "Check Data", "badge": "", "kind": "gap",
		"sub": "no warning store",
		"reason": "There is a warning collection and it is about opening a FILE, not about checking a world: project_open() returns a warnings array, EngineBridge.last_open_warnings holds it, and app.gd surfaces its first entry after an open. Nothing anywhere walks a loaded WORLD looking for contradictions in its own data, which is what this route means -- and what would be checked, against which invariant, is itself undefined. The five validators that do exist check DEFINITIONS rather than world state and have their own route, Validation > Definitions, next door; this row is what is still missing after it. The canvas's `8` badge on this row is mockup data, so no badge is drawn. (Re-checked 2026-09-05; this row previously said no warning collection existed anywhere, which project_open's own return had already falsified. Re-checked 2026-09-06 when Definitions landed: the DM-10 register row moved there, so it no longer reads (C) here.)"},
	{"group": "Validation", "id": "val_repair", "label": "Repair / Normalize", "badge": "", "kind": "gap",
		## Corrected 2026-09-24 (ALIGNMENT_AUDIT Part 2 B12): "No validation
		## pass exists" contradicted `val_defs` two rows up, which is live.
		"sub": "nothing to repair against", "reason": "Validation > Definitions does check definitions, but it only reports -- nothing is written back -- and there is no world check (Check Data, above) whose findings a repair could act on."},
]

## **Four groups, not five.** See the header's divergence note.
const GROUP_ORDER: Array[String] = ["Import", "Export", "Sources", "Validation"]

# ---------------------------------------------------------------------------
# Export ▸ Maps -- the disclosures the canvas's pyramid controls need
# ---------------------------------------------------------------------------

const SCHEME_NOTE := "grid + index.json exports the Region-select marquee as a flat row/column grid of height and colour tiles plus tiles/index.json (region_export_tiles). XYZ, TMS and WMTS export the WHOLE world's LOD pyramid, levels 0..N, as PNG tiles plus a tiles.json manifest (slippy_export_tiles): XYZ is z/x/y with row 0 at the top, TMS flips y, WMTS is TileMatrixSet/TileMatrix/TileRow/TileCol. There is no CRS -- tiles sit on the world's own planar cell grid, so a web client needs a flat CRS such as Leaflet's L.CRS.Simple."

## The OUTPUT checkboxes around the viewer page. The pyramid export writes
## `leaflet-preview.html` beside tiles.json (`cartalith_io::slippy::
## leaflet_preview_html`, 2026-09-23); the marquee grid does not, and nothing
## writes a style.json.
const PREVIEW_NOTE := "The pyramid export (XYZ / TMS / WMTS) always writes leaflet-preview.html beside tiles.json: a Leaflet page on L.CRS.Simple that addresses the archive's own tiles -- unzip and open it in a browser. The grid + index.json scheme is not a slippy pyramid, so it has no viewer page."
const STYLE_NOTE := "Not built. No style model exists to write a style.json from; the viewer page carries its own attribution line."

## Why `0–8` is disabled: `slippy_export_tiles` clamps to the bake's own
## ceiling, `bake_bridge::MAX_BAKE_DEPTH` (6).
const ZOOM_NOTE := "The pyramid export shares the atlas bake's depth ceiling (MAX_BAKE_DEPTH = 6, 5 461 tiles per pixel density) and builds the whole archive in memory, so levels past 6 are refused rather than attempted."

const CRS_NOTE := "The export is in the world's own cell grid. No CRS handling exists anywhere in the workspace -- reprojection was the substance of the Conversion group the owner deleted on 2026-08-20 (GUI_GAP_REGISTER.md §7.4), and no import or export path has carried a projection since."

## **Corrected 2026-09-01.** This used to say the political tint, labels/icons
## and rivers "are drawn by render.rs into the live viewport texture". None of
## those three is a `render.rs` stage, and `render.rs` says so itself: its own
## grading-stage header states it runs "before the Godot overlays draw rivers,
## labels, settlement icons, territory and the scale bar (`map_overlay.gd` and
## its siblings, which composite over the `ImageTexture` this raster
## becomes)". Traced this pass:
##
## - labels, settlement icons and manual icons -> `map_overlay.gd`'s own
##   `_draw_labels` / `_draw_manual_icons`, Control-level draw calls
## - political tint -> `lib.rs::territory_texture()`, its own TextureRect
##   layer under `viewport_host.gd`
## - rivers -> `map_overlay.gd::_draw_rivers` vector strokes since 2026-09-22
##   (a loaded save still gets a channel-mask tint inside
##   `lib.rs::build_color_texture()`)
##
## The *tile* export is a third thing again: `region_export::tile_png_bytes`
## calls `render_height_tile_rgba(tile, ...)`, a hillshade of the tile's own
## heights, so it carries none of the four. (The whole-world raster export is
## the one place a river tint does survive -- `render.rs::channel_tint`
## transcribes it -- which is why the sentence has to name the path, not just
## the feature.)
const LAYER_NOTE := "region_export_tiles bakes elevation (RG16) and, with visual tiles on, a hillshade computed from the tile's own heights (render_height_tile_rgba) -- nothing else. The political tint is lib.rs's territory_texture() on its own layer, labels, icons and rivers are map_overlay.gd Control draws; none of them is a raster stage the tile writer could switch on. Compositing them into a tile means rasterising overlay geometry, which is CA-04's separable-layer work and not a switch here."

## The enumeration this used to carry -- "DccSettings persists storage roots
## and window state only" -- was two of the ten sections `dcc_settings.gd`
## now declares (roots, recents, GPU, autosave, LOD, lighting, theme,
## graphics, layout, atlas). The claim that matters is unchanged and is the
## only thing left here: no *export-preset* section exists and nothing reads
## one. `dcc_settings.gd`'s `§2.6 Window > Save layout as...` is the nearest
## shape such a list would take.
const PRESET_NOTE := "No export-preset store exists: DccSettings has no preset section and nothing reads one. Saving one would mean a new section there plus a reader in this pane, which is why the chip is drawn dead rather than writing a preset nothing would load."

## **Rewritten 2026-09-01.** This constant used to read "MARKDOWN_VAULT_
## INTEGRATION.md is owner-supplied design that is explicitly 'Not started; no
## code exists' [...] there is no vault to be linked to", and the block below
## hard-coded `NOT LINKED` / `0 notes` to match. That stopped being true on
## 2026-08-24: `cartalith-vault` is a real crate, `vault_bridge.rs` binds it,
## and `Data ▸ Markdown vault` opens a panel that connects a folder and links
## settlements, provinces and continents to notes in it. The block now reads
## `EngineBridge.vault_info()` for its status instead of asserting one.
##
## What is genuinely still DM-14 is the *export* half, and only that half:
## `obsidian://` links in exported tiles and note links in exported GeoJSON
## (`MARKDOWN_VAULT_SCOPE.md`'s own divergence table), plus two-way sync,
## which `MARKDOWN_VAULT_INTEGRATION.md` §33 keeps an explicit V1 non-goal.
## Those three are exactly the three checkboxes below, and they stay disabled.
## Corrected 2026-09-24 (audit B17): it named three linkable kinds; the
## vault's `EntityKind` has six (settlement, province, continent, faction,
## culture, landmark). Still open is DM-14's export half; two-way sync is
## MARKDOWN_VAULT_INTEGRATION.md §33's V1 non-goal.
const VAULT_NOTE := "The vault connection itself is live (Data ▸ Markdown vault): it indexes a folder of .md files and links settlements, provinces, continents, factions, cultures and landmarks to notes in it. Not built yet is the export half -- obsidian:// links in exported tiles and note links in exported GeoJSON -- plus two-way sync, which is deliberately left out of this first version. The three checkboxes below are those three items."

## The disabled reason for the three checkboxes specifically, which is a
## narrower statement than `VAULT_NOTE`: these three are unbuilt, not the
## connection above them.
const VAULT_EXPORT_NOTE := "Not built. cartalith-vault deliberately generates no obsidian:// URLs (its own module doc: \"no obsidian:// scheme here, no wikilink generation\"), cartalith-engine::geojson writes no note property, and two-way sync is MARKDOWN_VAULT_INTEGRATION.md §33's explicit V1 non-goal. Linking an entity to a note is live and lives in Data ▸ Markdown vault."

const PACKAGING_NOTE := "Both exports produce one stored (uncompressed) .zip. A loose folder tree and MBTiles are both new writers; MBTiles would take the XYZ/TMS pyramid the scheme row above now writes as its input."

# ---------------------------------------------------------------------------
# Export ▸ GIS / GeoJSON -- DM-03's own two disclosures
# ---------------------------------------------------------------------------

## The document carries this same sentence as its own `note` property, verbatim
## from the reference, so a consumer reading the file learns it too.
const GEOJSON_CRS_NOTE := "Coordinates are local planar kilometres (east, north) at this world's own scale, with north up -- not WGS84 longitude/latitude. RFC 7946 assumes WGS84, but a procedurally generated world has no true georeference; the reference makes the same call, and the document says so in its own note property."

const GEOJSON_CIV_NOTE := "Settlements, ways, territory and provinces come from the civilisation layer. A generated world and a reopened project both carry it; a legacy .zip save does not. Rivers are traced from the drainage network, which only a world generated in this session holds, so a reopened project exports its settlements, roads and borders without rivers, and a .zip save exports a valid document with no features. Landmarks (the map's points of interest) are not exported yet, so there is no poi layer: every exported place is a settlement."
## Civ/river sentences corrected 2026-09-24 (ALIGNMENT_AUDIT Part 1 A1):
## `geojson_bridge.rs::export_geojson` reads `self.civ` whatever the source
## (a reopened .ctl project restores it) and traces rivers only for
## `WorldSource::Generated`; the old text had both backwards for a project.
## POI sentence corrected 2026-09-24 (ALIGNMENT_AUDIT Part 2 B9): landmarks
## and the `poi` icon family exist; what is missing is only the GeoJSON side --
## `geojson_bridge.rs` builds every place `is_poi: false` and reads no landmark.

## The pattern pane's FORMAT row on this route, and the reason its other two
## segments are dead.
##
## **Measured this pass**, not assumed: `grep -rni csv --include=*.rs crates/`
## outside `cartalith-assets` and `cartalith-godot/src/lib.rs` returns **0**;
## inside them it is `parse_pack_csv`/`MANIFEST_CSV` (an asset-pack manifest
## *reader*) and `as_batch_tag`'s comma-separated tag argument. Neither is an
## entity writer.
const GIS_FORMAT_NOTE := "GeoJSON is the only entity document this engine writes: cartalith_engine::geojson is the whole of it, and it emits a FeatureCollection or nothing. Nothing in the workspace writes settlements, ways or factions as a plain JSON document or as CSV -- the only CSV anywhere is cartalith-assets' pack.csv manifest reader (parse_pack_csv), which is an asset-pack input rather than an export. Either format would be a new writer, not a switch on this row."

## Import ▸ Heightmaps' FORMAT row. The reasoning is `import_maps`' own, which
## has carried it since this window was rebuilt: parity, not a shortfall.
const HEIGHTMAP_FORMAT_NOTE := "TIFF is absent, and deliberately: the reference's own file input is accept=\"image/*\" and decodes through the browser, which does not decode TIFF either -- so PNG is parity, not a shortfall."

## Tile-grid choices the engine accepts (`cols`/`rows`, any `n > 0`). The
## canvas's own row is a four-way `0–4 / 0–6 / 0–8 / custom` zoom segment; this
## is the same control over the dimension this export actually has.
const GRID_CHOICES: Array[int] = [2, 4, 8]

## `slippy_export_tiles`' `scheme` values, by scheme-segment index; index 0 is
## the marquee grid, which is not a slippy scheme.
const TX_SCHEMES: Array[String] = ["", "xyz", "tms", "wmts"]
## The canvas's own `0–4 / 0–6 / 0–8` zoom segment, as each range's top level.
const ZOOM_CHOICES: Array[int] = [4, 6, 8]

## The two file names `setup()` derives from the exports root, named once
## because four sites use each of them: `setup()`'s pre-fill, the picker's
## default name, and -- since 2026-09-07 -- `_reroot_defaults()`, which has to
## recognise a pre-fill it did not write in order to leave a hand-picked
## destination alone. Two spellings of "region-tiles.zip" that drift apart make
## a moved exports root silently strand `_tx_dest` in the old folder.
const TX_DEFAULT_NAME := "region-tiles.zip"
const GIS_DEFAULT_NAME := "world.geojson"
const TILE_SIZES: Array[int] = [256, 512, 1024]

# ---------------------------------------------------------------------------
# State
# ---------------------------------------------------------------------------

## Phone (§13) -- PH-12. Same shape and the same answer as
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

## The last `_checks_run()` on Validation ▸ Definitions, so the status line can
## report the numbers the pane actually drew rather than scan again. Empty
## until that route has been built once -- and the status line tests for that
## rather than printing a zero, which would be a real count on a healthy world.
var _checks_last: Dictionary = {}

var _pane_title: Label
var _pane_sub: Label
var _pane_body: VBoxContainer     ## cleared and rebuilt per route
## **`Container`, because it is not the same class at every density.** An
## `HBoxContainer` on a pointer or tablet, an `HFlowContainer` on a handset --
## see `_build_pane()` for the measurement. Every one of the eleven call sites
## that fills it only ever calls `add_child`, `get_children` and `remove_child`,
## so none of them notices; the type here is the only place the difference has
## to be admitted. Declaring it `HBoxContainer` and assigning a flow container
## to it is a hard runtime error in GDScript, not a warning.
var _pane_footer: Container
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
## Scheme segment: 0 is the marquee grid, 1-3 the whole-world pyramid under
## `TX_SCHEMES[_tx_scheme]` addressing. `_tx_zmax`/`_tx_retina` are read only
## by the pyramid.
var _tx_scheme := 0
var _tx_zmax := 4
var _tx_retina := false
var _tx_tile := 512
var _tx_gzip := false
var _tx_visual := true
var _tx_ridged := false
var _tx_dest := ""

## Export ▸ World Data, live `export_raster_png`/`export_channel_atlas` opts.
## `bakeRes`' widths with the reference's own default (4096) among them, and
## `bakeTiles` off -- both the reference's own initial state.
##
## **The fallback is FIVE since owner ruling 15 (2026-09-06)**, not the three
## the reference shipped. It is only reached if `export_raster_widths` cannot be
## asked, and a fallback that disagrees with the engine would silently offer a
## ladder the binding refuses -- which is the failure this constant exists to
## avoid, so it has to move whenever `BAKE_WIDTHS` does.
##
## The width list is asked of the binding (`export_raster_widths`) rather than
## written here, so the shell cannot offer a resolution the engine refuses.
const WD_WIDTH_FALLBACK: Array[int] = [2048, 4096, 8192, 16384, 32768]
var _wd_widths: Array[int] = WD_WIDTH_FALLBACK.duplicate()
var _wd_width := 4096
var _wd_tiled := false
## `layersPreviewChk` (reference line 555, read by `exportZip` at 12452).
## Off by default, exactly as the reference has it -- v0.92 made these
## opt-in on the grounds that nothing reads them back on load.
var _wd_layers := false

## Export ▸ GIS / GeoJSON's destination, held the way `_tx_dest` is held for
## Export ▸ Maps.
##
## **New with the pattern pane.** Until 2026-09-05 this route had no persistent
## destination at all: its footer chip raised a save picker and wrote inside the
## callback, so there was nothing to draw in a TO row and nothing for a second
## export to reuse. `DataPane.dc.html` draws that row, so the destination is now
## chosen first (`Browse…`) and written second (`Export`), which is the shape
## Export ▸ Maps already had.
var _gis_dest := ""

## The last GeoJSON document's own per-layer feature counts, measured by parsing
## the text this window just wrote -- `{layer: count}` plus `"features"`.
##
## Measured, not modelled, and that is the point: these counts come off the
## **written file**, so the receipt reports what the export actually holds
## rather than repeating what the INCLUDE chips predicted it would.
##
## Until 2026-09-05 this note also said rivers were the one group the shell
## could not count before a run, "which no binding exposes". `WorldGen::
## get_rivers(min_order)` always exposed it; what was missing was the GDScript
## forwarder, and `EngineBridge.rivers(2)` is it -- so every carried group now
## has a pre-run number and this dictionary's job is narrower and sharper:
## it is the independent check on all five, not the only source for one.
## Empty until an export has run in this session; `has()`, never a zero.
var _gis_doc_layers: Dictionary = {}

## Export ▸ World Data -- the two disclosures this route owes, and the one it
## no longer does. Until 2026-08-24 this row was a **gap** whose reason read
## "cartalith-io reads .zip saves but does not write them"; that stopped being
## true when FI-01 landed the writer, and the row outlived it.
## Rewritten 2026-09-24 (ALIGNMENT_AUDIT Part 2 B10). It quoted "a dozen or
## so bytes of 8,060,928 differ" against the live viewport -- stale since
## `d657091`: `export_raster.rs::screen_river_ink` returns `None` for a
## generated world (the screen draws `map_overlay.gd::_draw_rivers`' vector
## strokes) while exports still stamp `river_ink()` into the raster. The
## re-render claim is `render::bake_rect`'s.
const WD_RASTER_NOTE := "The export re-renders the map at each output pixel's own position -- materials, hillshade, shading, the paper ground and the plate frame -- so an 8K export carries four times the material detail of a 2K one rather than the same picture resampled. One visible difference from the screen: on a generated world the screen draws rivers as smooth lines over the map, while an export paints them into the image from the map grid instead, so the two do not match exactly."
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
	## PH-12: rotation relay plus the "may I stack?" answer. Also re-asserts
	## `wrap_controls = false`, which this window already set for its own reason
	## above.
	_phone = DccWidgets.phone_window(self, host)
	_tx_dest = DccSettings.storage_root("exports").path_join(TX_DEFAULT_NAME)
	## Pre-filled exactly as `_tx_dest` is, so the pattern pane's TO row has a
	## real path to draw rather than a dash on first open. Neither default is
	## overwrite-guarded on the *write* -- both guard at pick time -- which is
	## unchanged by this pass and stated here rather than left to be discovered.
	_gis_dest = DccSettings.storage_root("exports").path_join(GIS_DEFAULT_NAME)
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
	## PH-12: a phone fills the whole screen -- §13 relocates the app menu bar
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
	## `ROUTES` is a 15-entry `const`, so the `is_empty()` test could never be
	## false and the `target != ""` test after it could never fail either -- the
	## line above assigns one. Both are gone; the fallback is unconditional.
	if target == "":
		target = String(ROUTES[0]["id"])
	_select_route(target)
	_refresh_foot()
	_refresh_status()

## One exact route, by id. The Data dropdown generates a row per `ROUTES` entry
## (`menus.gd::_data()`) -- the canvas's own fourteen, plus `val_defs`, which
## the canvas predates -- and each is its own destination. `open()`
## above takes a *group* and lands on that group's first route, which is what
## the four collapsed group rows used to do.
func open_route(route_id: String) -> void:
	_popup_full()
	_select_route(route_id)
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

	## PH-12: rail beside pane on a pointer, one at a time on a phone.
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

## PH-12, `asset_library_window.gd`'s switcher with two segments instead of
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
		## `GUI_GAP_REGISTER.md` §50: only "normal" was overridden, so a tap
		## left this `Button` in Godot's stock "hover" state -- there is no
		## hover to release on a touchscreen; Android leaves the emulated
		## pointer where the finger last was (the same mechanism the navpad's
		## first pill was found stuck in) -- and the theme's own raised grey
		## pill drew instead of either chip state. Every style a plain Button
		## can land in has to say the same thing "normal" does.
		var fill: StyleBox = DccTheme.flat(DccTheme.c("accent_wash")) if on else DccTheme.empty()
		b.add_theme_stylebox_override("normal", fill)
		b.add_theme_stylebox_override("hover", fill)
		b.add_theme_stylebox_override("pressed", fill)
		b.add_theme_stylebox_override("focus", DccTheme.empty())
		b.add_theme_color_override("font_color",
			DccTheme.c("accent") if on else DccTheme.c("text_dim"))

## PH-12. The route pane is cleared and rebuilt on every `_select_route()`, so
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

	## PH-12: the title and its four-area subtitle are what `phone_head()` draws
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
	## PH-12: 252 px is 64% of a phone's 393 dp, and this is a full-width pane
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
		wrap.size_flags_vertical = Control.SIZE_EXPAND_FILL   ## PH-12, stacked

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
	## **The handset footer WRAPS; the pointer and tablet one does not.**
	##
	## An `HBoxContainer` demands the sum of its children as a hard minimum, and
	## that is what the pane footer was at every density. Measured with
	## `_panemin_probe.tscn -- --force-touch --vp 500x1080`, `export_world`'s
	## three chips are 171 + 125 + 145 = 441 px plus 36 px of separation, plus
	## the note's own 1 px = **478**, against the **376 px** this pane has:
	## the window is laid out in its own content-scaled space, **412 px** wide
	## at the narrowest handset this shell claims, less `PANE_PAD_X` twice.
	## A `Control` cannot be laid below its combined minimum, so the row grew
	## 102 px past the pane and took the window's whole contents minimum with
	## it. Only `export_world` exceeded the room; the next widest footer was
	## `export_maps` at 364, 12 px inside it.
	##
	## **376, not 464.** An earlier reading of this compared the row's demand,
	## which is in the window's 412 px content space, against the OS window's
	## 500 physical px, and made the shortfall look like 14 px. See
	## `_panemin_probe.gd`'s `_client_rect()` for the correction and for how the
	## conflation announced itself: the pane measured the same 393 px wide at
	## `--vp 500x1080` and at `--vp 1080x2340`, being one dp layout at two
	## scales, while the checks compared it against 500 and against 1080.
	##
	## `FOOT_NOTE_MIN_W` cannot reach this and deliberately does not try: see
	## `_footer_note()`, where applying the 160 px floor on a handset was
	## measured putting the route's own picker at `shown=0.000`. **The floor
	## that fixes the desktop breaks the phone**, and the chips are 477 px with
	## or without a note.
	##
	## So the row is allowed a second line instead. `HFlowContainer`'s minimum
	## width is its **widest single child**, not the sum -- 171 px here -- because
	## one-per-line is a layout it can produce; measured on an isolated container
	## before it was relied on, and `_panemin_probe`'s `footer terms` line now
	## prints the row sum and the container's own demand side by side so the
	## difference is visible per route rather than argued. Nothing is truncated,
	## nothing moves off screen, and the note keeps its `SIZE_EXPAND_FILL`: a
	## flow container honours expand within a line, so the note still absorbs
	## line 1's slack and the chips still finish flush on the row's right edge.
	##
	## **What it costs, stated rather than left to be found.** A flow container's
	## minimum HEIGHT is its worst case -- every child stacked -- so the handset
	## footer's vertical minimum goes from one row to as many rows as there are
	## chips (measured 44 -> 167 px at `export_world`). That is affordable only
	## because there is room: this window's whole contents minimum on a handset
	## is 339 px tall against a 1056 px window at 500x1080, the narrowest case.
	## It is **not** affordable at pointer density, where the window promises to
	## work at 640 px tall and the footer fits on one line anyway -- which is why
	## this is a handset branch and not a replacement.
	##
	## `separation` is a `BoxContainer` constant and means nothing to a flow
	## container; the two it reads are `h_separation` and `v_separation`. Setting
	## the wrong one is silent.
	if _phone:
		var flow := HFlowContainer.new()
		flow.add_theme_constant_override("h_separation", 12)
		flow.add_theme_constant_override("v_separation", 8)
		_pane_footer = flow
	else:
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 12)
		_pane_footer = row
	foot_pad.add_child(_pane_footer)

	return wrap

func _build_status_line() -> Control:
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"top": 1}))
	## PH-12: stacked and clipped on a phone, for the reason
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
	elif id == "val_defs":
		## DM-10. A third bespoke body, and for the reason the other two are:
		## the pattern's one 620 px prose column cannot hold a seven-column
		## table. Its footer action row is this window's shared `_pane_footer`,
		## exactly as `_build_pattern_pane` uses it.
		_build_checks_pane()
		_checks_actions()
	else:
		_build_pattern_pane(route)
	_refresh_status()
	## PH-12: picking a route in the ROUTES pane is a navigation whose whole
	## result is the pane next door, so the switcher follows it; and the pane it
	## just built is fresh nodes that have never been fitted.
	_phone_refit()
	_show_phone_pane("route")

## The twelve routes `Data manager window 1920` never drew a pane for, rendered
## through `DataPane.dc.html`'s pattern.
##
## The anatomy is the artboard's, in its order: title, a one-line purpose, a
## rule, the form region (FORMAT · INCLUDE · EXTENT), the route's own prose, a
## rule, the destination row, the receipt, and the action row. Every part is
## **omitted** where the route has no real source for it rather than drawn with
## a stand-in, which is why `_pattern_form()` and `_pattern_destination()` both
## start with a `match` that falls through to nothing for most routes.
##
## **The action row is the pinned `_pane_footer`, not an in-body row.** The
## artboard draws it inside the pane because its pane is the whole window; this
## window already has a footer band above its status line, and every route's
## actions have lived there since the 2026-08-20 rebuild. Moving them into the
## scrolled body would make them scroll away.
func _build_pattern_pane(route: Dictionary) -> void:
	var id := String(route["id"])
	## One column at roughly the width of the canvas's own `1fr` half, plus a
	## spacer -- prose set across the full 1 400 px pane is unreadable, and
	## neither canvas sets a line that long (`DataPane.dc.html`'s own pane is
	## 760 - 196 = 564 px, the same order).
	var lane := HBoxContainer.new()
	lane.add_theme_constant_override("separation", COL_GAP)
	lane.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_pane_body.add_child(lane)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	## PH-12: 620 dp of measure inside a 393 dp column widens the window past
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

	_pattern_heading(col, String(route["label"]), String(PANE_PURPOSE.get(id, "")))
	_pattern_rule(col, PATTERN_RULE_TOP, PATTERN_RULE_BOTTOM)
	_pattern_form(col, id)
	_pattern_prose(col, route)
	_pattern_rule(col, PATTERN_RULE_TOP_2, PATTERN_RULE_BOTTOM)
	_pattern_destination(col, id)
	## The artboard's `flex:1;min-height:12px` between the destination row and
	## the receipt. Fixed rather than expanding: this body scrolls, so "push the
	## receipt to the bottom" has no bottom to push to, and giving the column an
	## expand flag would change how the shared `_pane_body` sizes for the two
	## bespoke panes as well.
	var gap := Control.new()
	gap.custom_minimum_size.y = 12
	col.add_child(gap)
	_pattern_receipt(col, route)
	_pattern_actions(route)

# -- The pattern's parts ------------------------------------------------------

## `color:var(--ink);font-weight:500;font-size:14px` over
## `class="mono";padding-top:3px;font-size:var(--m2);color:var(--faint)`.
func _pattern_heading(parent: Control, title: String, purpose: String) -> void:
	parent.add_child(DccTheme.label(title, "text_bright", PATTERN_TITLE_FS))
	if purpose == "":
		return
	var p := DccWidgets.pad(parent, 0, 3, 0, 0)
	var l := DccTheme.mono_label(purpose, "text_faint", DccTheme.FS_MICRO)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	p.add_child(l)

## `.rule{height:1px;background:var(--div)}`.
##
## **Not `DccTheme.rule()`**, which paints `line` (`--hair`). The artboard's
## `.rule` is `--div`, and `line_soft`'s own token comment draws the same
## distinction from the other side: it is *"the lighter rule, used inside a
## surface rather than between two"*. These two sit inside the pane.
func _pattern_rule(parent: Control, top: int, bottom: int) -> void:
	var p := DccWidgets.pad(parent, 0, top, 0, bottom)
	var r := ColorRect.new()
	r.color = DccTheme.c("line_soft")
	r.custom_minimum_size = Vector2(0, 1)
	r.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	p.add_child(r)

## The artboard's form row: a `width:74px` mono label at `--m2` / `.1em` /
## `--faint`, then the control. A *different, narrower* label column from the
## Export ▸ Maps pane's `W_ROW_LABEL` 120 -- both are drawn, from two artboards.
func _pattern_row(parent: Control, label_text: String, top: int = 0,
		top_align: bool = false) -> HBoxContainer:
	var p := DccWidgets.pad(parent, 0, top, 0, 0)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	p.add_child(row)
	var l := DccTheme.mono_label(label_text, "text_faint", DccTheme.FS_MICRO, 1)
	l.custom_minimum_size.x = PATTERN_LABEL_W
	l.vertical_alignment = VERTICAL_ALIGNMENT_TOP if top_align else VERTICAL_ALIGNMENT_CENTER
	if top_align:
		l.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
	row.add_child(l)
	return row

## One segment of the FORMAT track: `border-radius:12px;padding:3px 12px`, lit
## at `background:var(--wash2);color:var(--acc)` and quiet at `color:var(--dim)`
## with no fill.
func _seg_box(fill: Color) -> StyleBoxFlat:
	var sb := DccTheme.flat(fill, PATTERN_SEG_R)
	sb.content_margin_left = PATTERN_SEG_PAD_X
	sb.content_margin_right = PATTERN_SEG_PAD_X
	sb.content_margin_top = PATTERN_SEG_PAD_Y
	sb.content_margin_bottom = PATTERN_SEG_PAD_Y
	return sb

func _style_pattern_segment(b: Button, on: bool) -> void:
	var rest := _seg_box(DccTheme.c("accent_wash_2") if on else Color(0, 0, 0, 0))
	for n in ["normal", "pressed", "disabled"]:
		b.add_theme_stylebox_override(n, rest)
	b.add_theme_stylebox_override("hover",
		rest if on else _seg_box(DccTheme.c("line_soft")))
	var ink := DccTheme.c("accent") if on else DccTheme.c("text_dim")
	b.add_theme_color_override("font_color", ink)
	b.add_theme_color_override("font_hover_color",
		ink if on else DccTheme.c("text_bright"))
	## A lit-but-DISABLED segment has to survive Godot resolving `disabled`
	## ahead of `normal` -- the trap `DccWidgets.set_segment_on()` documents,
	## and this pane has the same shape: one real format beside impossible ones.
	b.add_theme_color_override("font_disabled_color",
		ink if on else DccTheme.c("text_ghost"))

## The artboard's FORMAT control, which is a **track**: `background:var(--ins);
## border-radius:14px;padding:2px` holding the segments.
##
## That is a different shape from `_segments()` above, which draws the outline
## chips `Data manager window 1920` specifies for the Export ▸ Maps pane's
## Scheme / CRS / Packaging rows. The difference is the two artboards', not a
## reinterpretation of either: this window now draws both, and reconciling them
## is a decision about the older pane rather than about this one.
func _pattern_segments(row: Control, items: Array, selected: int,
		on_pick: Callable) -> Array:
	var track := PanelContainer.new()
	var tb := DccTheme.flat(DccTheme.c("sunken"), PATTERN_TRACK_R)
	tb.content_margin_left = PATTERN_TRACK_PAD
	tb.content_margin_right = PATTERN_TRACK_PAD
	tb.content_margin_top = PATTERN_TRACK_PAD
	tb.content_margin_bottom = PATTERN_TRACK_PAD
	track.add_theme_stylebox_override("panel", tb)
	track.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	var group := HBoxContainer.new()
	group.add_theme_constant_override("separation", 4)
	track.add_child(group)
	row.add_child(track)
	var out: Array = []
	for i in items.size():
		var item: Dictionary = items[i]
		var idx := i
		var b := Button.new()
		b.text = String(item["text"])
		b.focus_mode = Control.FOCUS_NONE
		b.add_theme_font_override("font", DccTheme.mono(0))
		b.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
		b.disabled = not bool(item.get("enabled", true))
		if String(item.get("tip", "")) != "":
			b.tooltip_text = String(item["tip"])
		_style_pattern_segment(b, i == selected)
		if on_pick.is_valid():
			b.pressed.connect(func(): on_pick.call(idx))
		group.add_child(b)
		out.append(b)
	return out

## `.chip` -- `padding:3px 11px;border-radius:999px`, a **read-only** status
## pill and not a button. `DccWidgets.chip()` is the canvas's outline *action*
## chip and would read as pressable here.
func _pill(parent: Control, text: String, bg_token: String, ink_token: String,
		tip: String) -> Label:
	var wrap := PanelContainer.new()
	var sb := DccTheme.flat(DccTheme.c(bg_token), PATTERN_CHIP_R)
	sb.content_margin_left = PATTERN_CHIP_PAD_X
	sb.content_margin_right = PATTERN_CHIP_PAD_X
	sb.content_margin_top = PATTERN_CHIP_PAD_Y
	sb.content_margin_bottom = PATTERN_CHIP_PAD_Y
	wrap.add_theme_stylebox_override("panel", sb)
	wrap.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	if tip != "":
		wrap.tooltip_text = tip
		wrap.mouse_filter = Control.MOUSE_FILTER_STOP
	var l := DccTheme.mono_label(text, ink_token, DccTheme.FS_TINY)
	wrap.add_child(l)
	parent.add_child(wrap)
	return l

## The INCLUDE row's chips, in the artboard's three states and no fourth:
##
##   * **lit** (`--wash2` / `--acc`) -- this format carries the group, and a
##     real source answered how many there are;
##   * **dim** (`--ins` / `--dim`) -- the group exists in this world and the
##     format does not carry it. The artboard's `landmarks 214`, and true here:
##     `export_geojson` emits no landmark layer;
##   * **dashed** (`--ins` / `--dis`) -- no count could be asked for. The
##     artboard's `religions —`, and the artboard's own rule that such a chip is
##     *dashed and dimmed, not hidden*.
##
## Every count comes from `_gis_count()`, which omits the key rather than
## returning a zero. The tooltip always carries the reason, in both directions:
## a dashed chip says why there is no number, a dim chip says why the number is
## not going into the file.
func _include_chips(row: Control) -> void:
	var flow := HFlowContainer.new()
	flow.add_theme_constant_override("h_separation", 4)
	flow.add_theme_constant_override("v_separation", 4)
	flow.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(flow)
	for g in GIS_GROUPS:
		var key := String(g["key"])
		var carried := bool(g["carried"])
		var got := _gis_count(key)
		var has_count: bool = got.has("count")
		var text := ("%s %d" % [key, int(got["count"])]) if has_count \
			else ("%s —" % key)
		var ink := "text_ghost"
		if has_count:
			ink = "accent" if carried else "text_dim"
		var bg := "accent_wash_2" if (has_count and carried) else "sunken"
		var tip := ""
		if not has_count:
			tip = String(got.get("why", ""))
		if not carried:
			tip = ("%s\n\n" % tip if tip != "" else "") \
				+ "Not written by this format: export_geojson emits settlement, poi, way, river, territory and province layers, and there is no %s layer among them." % key
		elif has_count:
			tip = String(got.get("how", ""))
		_pill(flow, text, bg, ink, tip)

## The count for one INCLUDE group, or the reason there is none.
##
## Returns `{"count": int, "how": String}` when a real source answers and
## `{"why": String}` when none can -- **never a zero standing in for an
## absence**, which is why every caller tests `has("count")`. A world whose
## civilisation layer is missing entirely (a legacy `.zip` save:
## `SAVEFILE_COMPAT.md`; a reopened project keeps its layer) reports the
## reason, not `0`.
func _gis_count(key: String) -> Dictionary:
	if _bridge == null or not _bridge.has_world:
		return {"why": "No world is loaded."}
	## A reopened .ctl project restores its civilisation layer; only a legacy
	## .zip save lacks one (`SAVEFILE_COMPAT.md`). Corrected 2026-09-24.
	var civ_absent := "None. Either this world has none (they were cleared, or never placed), or it was opened from a legacy .zip save, which carries no civilisation layer -- this window cannot tell those apart. Either way there is nothing of this group to write."
	match key:
		"settlements":
			var n: int = _bridge.settlements().size()
			return {"count": n, "how": "EngineBridge.settlements() -> get_settlements(), the same list export_geojson turns into settlement features."} if n > 0 else {"why": civ_absent}
		"factions":
			var n: int = _bridge.civ_faction_count()
			return {"count": n, "how": "EngineBridge.civ_faction_count(), the roster excluding Unclaimed. The document carries one territory polygon per faction that actually holds cells, so a faction with no claimed ground contributes a property and no feature."} if n > 0 else {"why": civ_absent}
		"ways":
			## Generated ways and sea lanes only. `get_roads()`/`get_sea_routes()`
			## also return hand-drawn `infra.ways` (flagged `manual`), and
			## `export_geojson` reads `civ.ways` and `civ.sea_routes` and never
			## `infra.ways` -- so counting the getters whole would report a
			## number the file will not contain.
			var n := 0
			for w in _bridge.roads():
				if not bool((w as Dictionary).get("manual", false)):
					n += 1
			for w in _bridge.sea_routes():
				if not bool((w as Dictionary).get("manual", false)):
					n += 1
			return {"count": n, "how": "Generated roads plus sea lanes (civ.ways where not hidden, plus civ.sea_routes). Hand-drawn ways are excluded because export_geojson does not read infra.ways -- if any exist, the receipt says how many were left out."} if n > 0 else {"why": civ_absent}
		"rivers":
			## `2` is not a choice here. `geojson_bridge.rs`'s
			## `EXPORT_MIN_RIVER_ORDER` is `2`, so asking for any other order
			## would put this chip and the post-run receipt at different
			## numbers -- a chip that disagrees with the file is worse than the
			## dash this branch was until 2026-09-05.
			var n: int = _bridge.rivers(2).size()
			return {"count": n, "how": "EngineBridge.rivers(2) -> WorldGen::get_rivers(2). Both this and the exporter's river block build their set from the same split_river_polylines(trace_river_polylines(order, recv, w, h, 2), w, None) pair -- river_entities() calls it on one side, export_geojson() on the other -- so the chip and the written document count the same runs by construction, not by coincidence. The receipt still measures the file itself."} if n > 0 else {"why": "get_rivers(2) answered empty, and this window cannot tell which absence that is: a loaded .zip save retains no channel topology at all (SAVEFILE_COMPAT.md -- stream_order and channels are None, so nothing can be traced), the forwarder also answers empty while a generation is in flight, and a generated world can simply have no run reaching Strahler order 2. export_geojson's river block tests the same condition, so the document carries no river feature either way."}
		"provinces":
			var n: int = _bridge.provinces().size()
			return {"count": n, "how": "EngineBridge.provinces() -> get_provinces(). A province with no cells in the province raster contributes no feature."} if n > 0 else {"why": civ_absent}
		"landmarks":
			var n: int = _bridge.landmarks().size()
			return {"count": n, "how": "EngineBridge.landmarks(), the last landmark pass's placements."} if n > 0 else {"why": "No landmark pass has run in this world, or one ran and placed nothing -- landmarks() reports the last run's placements and answers an empty list to both. Either way there is nothing of this group in the world."}
		"religions":
			## The idiom is `civilization_workspace.gd::_religion_head_refresh()`:
			## distinct `adherents` keys, `none` excluded, count above zero. Its
			## own comment is why this dashes instead of printing `0`: *"no run
			## and no faiths are different answers"*.
			var places: Array = _bridge.settlements()
			var seen := {}
			var carried_any := false
			for p in places:
				var ad: Dictionary = (p as Dictionary).get("adherents", {})
				if not ad.is_empty():
					carried_any = true
				for k in ad.keys():
					if String(k) != "none" and int(ad[k]) > 0:
						seen[String(k)] = true
			if not carried_any:
				return {"why": "The belief layer has not run. settlements() carries no adherents dictionary until civ_belief_run does (Civilisation ▸ Religion), and 0 faiths is a different answer from 'not run' -- the same distinction civilization_workspace.gd's own faith count makes."}
			return {"count": seen.size(), "how": "Distinct adherents keys with a non-zero count, the unaffiliated slot excluded -- civilization_workspace.gd::_religion_head_refresh()'s own measure."}
	return {"why": "No source in this shell reports this group."}

## The form region. A `match` that falls through to nothing for every route
## whose form would have to be invented -- which is most of them, and is the
## honest half of inheriting a pattern.
func _pattern_form(col: Control, id: String) -> void:
	match id:
		"import_heightmap":
			var fmt := _pattern_row(col, "FORMAT")
			_pattern_segments(fmt, [
				{"text": "PNG", "enabled": true},
				{"text": "TIFF", "enabled": false, "tip": HEIGHTMAP_FORMAT_NOTE},
			], 0, func(_i: int): pass)
		"export_gis":
			var fmt := _pattern_row(col, "FORMAT")
			_pattern_segments(fmt, [
				{"text": "GeoJSON", "enabled": true},
				{"text": "JSON", "enabled": false, "tip": GIS_FORMAT_NOTE},
				{"text": "CSV", "enabled": false, "tip": GIS_FORMAT_NOTE},
			], 0, func(_i: int): pass)
			var inc := _pattern_row(col, "INCLUDE", 9, true)
			_include_chips(inc)
			var ext := _pattern_row(col, "EXTENT", 9)
			var v := DccTheme.mono_label("whole world", "text_secondary", DccTheme.FS_TINY)
			v.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
			ext.add_child(v)
			## The artboard's own hint here reads `· a Region marquee narrows
			## this`. **It does not, on this route**, and the artboard is
			## illustrative: `export_geojson` describes the whole world and the
			## marquee is Export ▸ Maps' input. The row keeps its shape and says
			## the true thing.
			var hint := DccTheme.mono_label(
				"· the Region marquee narrows Export ▸ Maps, not this route",
				"text_ghost", DccTheme.FS_MICRO)
			hint.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
			ext.add_child(hint)

## The route's own paragraph(s): what it really is -- the live description, the
## routing shortcut, or the disclosed reason, which for a gap route is the whole
## content of the pane.
func _pattern_prose(col: Control, route: Dictionary) -> void:
	match String(route["id"]):
		"import_heightmap":
			if _bridge != null and _bridge.import_api:
				DccWidgets.note(col,
					"Reads a PNG heightmap (white = high), resamples it to the working grid at the image's own aspect ratio, and infers a tectonic substrate from its morphology so lithology, resources and settlement have something to read -- the reference's Import ▸ Load heightmap… followed by Infer tectonics from heightmap. Scale (width, peak) comes from New world…, exactly as the reference's own calibrate step reuses its generate form.")
			else:
				DccWidgets.note(col,
					"This build's GDExtension predates the heightmap-import binding (WorldGen::import_heightmap). Rebuild cartalith-godot to enable it.")
		"import_gis":
			## DM-03's other half, Ruling V (LARGE_ITEM_RULINGS.md, 2026-09-21):
			## `geojson_apply.rs` over `cartalith_io::parse_geojson`. The poi
			## clause below said "this port has no POI concept" until
			## 2026-09-24 (ALIGNMENT_AUDIT Part 2 B9) -- landmarks exist; the
			## import simply places no landmark from a poi feature.
			DccWidgets.note(col,
				"Reads a FeatureCollection and places what it can: a settlement feature (bounds/occupied/water gates, same as the Settlement tool) and a territory polygon (rasterised cell by cell). A feature naming a faction this world doesn't have creates it -- by its imported name, never a fuzzy remap, never silently dropped to unclaimed.")
			DccWidgets.note(col,
				"Imported territory is laid down as Territory paint: it survives Recompute civilisation, a later Territory stroke paints over it, and a subtract stroke over it shows the computed owner again. A territory polygon naming no faction is skipped, because paint cannot force a cell unclaimed.")
			DccWidgets.note(col,
				"poi, way, river and province features are read and counted but not placed: the import does not turn a point of interest into a landmark yet, a way needs real settlement endpoints an import doesn't carry, a river is generated hydrology rather than user data, and a province must stay inside its own faction's territory in a way an arbitrary polygon isn't checked against. The result after importing names each one.")
			DccWidgets.note(col,
				"Coordinates are read as this world's own planar kilometres regardless of what the document's own CRS property claims -- the only coordinate system a generated world has.")
		"import_world":
			DccWidgets.note(col,
				"Opens the same .ctl project picker as File ▸ Open project… -- routed here per §9, not reimplemented.")
		"import_assets":
			DccWidgets.note(col,
				"Routes to Assets ▸ Import asset pack .zip… -- §2.4's own table calls this item a shortcut, not a second implementation.")
		"export_gis":
			## DM-03: `export_geojson` (geojson_bridge.rs) over
			## `cartalith_engine::geojson`, which is golden-verified
			## character-for-character against the reference's own document.
			DccWidgets.note(col,
				"Writes the whole world as one GeoJSON FeatureCollection: settlements, roads, sea lanes, rivers (Strahler order 2 and up), faction territory and provinces, each tagged with its own layer property.")
			DccWidgets.note(col, GEOJSON_CRS_NOTE)
			DccWidgets.note(col, GEOJSON_CIV_NOTE)
		"export_assets":
			## DM-05: routes to the Asset library window's own real Export pack
			## .zip… (AS-04, `as_export_pack_bytes` → `archive::write_pack`) --
			## §2.4's table calls this a shortcut, same as `import_assets`.
			DccWidgets.note(col,
				"Routes to the Asset library window's own Export pack .zip… (Assets ▸ ⧉ Asset library, §8's window bar) -- real (as_export_pack_bytes -> archive::write_pack).")
		_:
			DccWidgets.note(col, String(route.get("reason", "Not implemented.")))

## The artboard's `TO` row: a filled `--ins` well at `border-radius:8px;
## min-height:var(--ctl);padding:4px 11px` beside a `.btn2` `Browse…` shrunk to
## `--ctl` by the artboard's own inline override.
##
## Drawn for the one pattern route that keeps a destination. The import routes
## and the two `→` shortcuts hand the whole file dialog to another window, so
## there is no path for this row to hold and it is omitted rather than drawn
## with a placeholder.
func _pattern_destination(col: Control, id: String) -> void:
	if id != "export_gis":
		return
	var row := _pattern_row(col, "TO")
	var well := PanelContainer.new()
	var sb := DccTheme.flat(DccTheme.c("sunken"), DccWidgets.MODAL_INSET_RADIUS)
	sb.content_margin_left = PATTERN_CHIP_PAD_X
	sb.content_margin_right = PATTERN_CHIP_PAD_X
	sb.content_margin_top = 4
	sb.content_margin_bottom = 4
	well.add_theme_stylebox_override("panel", sb)
	well.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	well.custom_minimum_size.y = DccWidgets.MODAL_CTL
	var l := DccTheme.mono_label(_gis_dest if _gis_dest != "" else "—",
		"text_secondary" if _gis_dest != "" else "text_ghost", DccTheme.FS_TINY)
	l.clip_text = true
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	well.add_child(l)
	if _gis_dest != "":
		well.tooltip_text = _gis_dest
		well.mouse_filter = Control.MOUSE_FILTER_STOP
	row.add_child(well)
	## `.btn2` -- `DccWidgets.modal_quiet()`. `Modal.dc.html` and
	## `DataPane.dc.html` declare `.btn` and `.btn2` byte-identically (checked
	## with `diff` this pass), so the factory pair that landed for the modal
	## artboard is the design system's, not the modal's; only its name records
	## where it first shipped.
	var browse := DccWidgets.modal_quiet(row, "Browse…", func():
		_pick_geojson_destination())
	## The artboard overrides `.btn2`'s own `--btnH` down to `--ctl` in this row.
	## Not on a touch density, where the factory's floor is a tap target rather
	## than a drawing decision.
	if not (DccTheme.is_tablet() or DccTheme.is_phone()):
		browse.custom_minimum_size.y = DccWidgets.MODAL_CTL

## The result block: `background:var(--ins);border-radius:8px;padding:9px 11px`,
## a tick, what was written, how long ago -- and then the line that is the point
## of the whole block, **what was not included**.
##
## Its absent state is a dash carrying its reason, per route, rather than a
## success line with zeroes in it.
func _pattern_receipt(col: Control, route: Dictionary) -> void:
	var wrap := PanelContainer.new()
	var sb := DccTheme.flat(DccTheme.c("sunken"), DccWidgets.MODAL_INSET_RADIUS)
	sb.content_margin_left = PATTERN_RECEIPT_PAD_X
	sb.content_margin_right = PATTERN_RECEIPT_PAD_X
	sb.content_margin_top = PATTERN_RECEIPT_PAD_Y
	sb.content_margin_bottom = PATTERN_RECEIPT_PAD_Y
	wrap.add_theme_stylebox_override("panel", sb)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 4)
	wrap.add_child(body)
	col.add_child(wrap)

	var run := _last_route_run(String(route["id"]))
	if run.is_empty():
		var l := DccTheme.mono_label(_receipt_absent_reason(route),
			"text_ghost", DccTheme.FS_MICRO)
		l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		body.add_child(l)
		return

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 10)
	body.add_child(head)
	var ok := bool(run.get("ok", false))
	head.add_child(DccTheme.mono_label(
		DccIcons.SYMBOLS["tick"] if ok else DccIcons.SYMBOLS["cross"],
		"good" if ok else "block", DccTheme.FS_TINY))
	var line := DccTheme.mono_label(String(run.get("receipt", "")),
		"text_secondary", DccTheme.FS_TINY)
	line.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(line)
	## A snapshot taken when the pane was built, not a ticking clock -- which is
	## what the artboard draws too. Every action in this pane rebuilds the pane,
	## so it is re-read on each one.
	var ago := _ago(int(run.get("msec", 0)))
	if ago != "":
		head.add_child(DccTheme.mono_label(ago, "text_ghost", DccTheme.FS_MICRO))
	var omitted := String(run.get("omitted", ""))
	if omitted != "":
		var n := DccTheme.mono_label(omitted, "text_ghost", DccTheme.FS_MICRO)
		n.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		n.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		body.add_child(n)

func _receipt_absent_reason(route: Dictionary) -> String:
	var kind := String(route.get("kind", "gap"))
	if kind == "gap":
		return "— nothing has run here, and nothing can: this route is disclosed above as having no engine behind it."
	if kind == "route":
		return "— this route hands off to another window, which reports its own result on the app status line. Nothing runs here to receipt."
	if String(route["id"]).begins_with("import_"):
		return "— this route hands the file to the shell's own importer; the result lands on the app status line and in the world itself, not in a receipt here."
	return "— no export has run on this route in this session. Nothing persists a run history (DM-12), so this starts empty at every launch rather than showing a remembered figure."

## The newest run recorded for one route, or `{}`. `_runs` is session-scoped and
## shared across routes, so entries carry the route they belong to.
func _last_route_run(id: String) -> Dictionary:
	for r in _runs:
		if String((r as Dictionary).get("route", "")) == id:
			return r
	return {}

## The artboard's `1.1 s ago`. Empty string when the record carries no monotonic
## stamp at all -- an omitted key, so no caller can read a zero as "just now".
func _ago(msec: int) -> String:
	if msec <= 0:
		return ""
	var s := float(Time.get_ticks_msec() - msec) / 1000.0
	if s < 90.0:
		return "%.1f s ago" % s
	if s < 5400.0:
		return "%d min ago" % int(round(s / 60.0))
	return "%d h ago" % int(round(s / 3600.0))

## `a, b and c`, for the receipt's not-included line.
func _and_list(items: PackedStringArray) -> String:
	var n := items.size()
	if n == 0:
		return ""
	if n == 1:
		return items[0]
	var head := PackedStringArray()
	for i in n - 1:
		head.append(items[i])
	return "%s and %s" % [", ".join(head), items[n - 1]]

## The artboard's `Reveal file` + `Export` action row, in `_pane_footer`.
##
## `.btn2` then `.btn` -- the quiet action first, the primary last, which is the
## order both approved artboards draw and the order `DccWidgets.modal_choices()`
## enforces for the modal one.
func _pattern_actions(route: Dictionary) -> void:
	var id := String(route["id"])
	var kind := String(route.get("kind", "gap"))
	match id:
		"import_heightmap":
			if _bridge != null and _bridge.import_api:
				_footer_note("replaces the current elevation field")
				DccWidgets.chip(_pane_footer, "Import heightmap…", func():
					hide()
					_host.open_heightmap_import(), true, 16, 6)
			else:
				_footer_note("binding missing in this build")
		"import_gis":
			_footer_note("places settlements + territory into the current world")
			var go_import := DccWidgets.chip(_pane_footer, "Import GeoJSON…", func():
				_pick_geojson_import(), true, 16, 6)
			go_import.disabled = _bridge == null or not _bridge.has_world
			go_import.tooltip_text = ("parse_geojson -> geojson_apply -> apply_geojson_document. An unknown faction is created; see the notes above for what isn't placed yet."
				if not go_import.disabled else "No world is loaded -- generate or open one first.")
		"import_world":
			_footer_note("replaces the whole world")
			DccWidgets.chip(_pane_footer, "Open project…", func():
				hide()
				_host.open_project_picker(), true, 16, 6)
		"import_assets":
			_footer_note("routes to the Assets menu")
			DccWidgets.chip(_pane_footer, "Import asset pack .zip…", func():
				hide()
				_host.open_asset_pack_picker(), true, 16, 6)
		"export_gis":
			_footer_note("writes to %s" % (_gis_dest if _gis_dest != "" else "—"))
			## Real only once a run has actually written a file: `reveal_on_disk`
			## opens a folder, and offering it before there is anything in it is
			## the same fiction as a zero standing in for an absence.
			var last := _last_route_run("export_gis")
			var wrote: String = String(last.get("path", "")) if bool(last.get("ok", false)) else ""
			var reveal := DccWidgets.modal_quiet(_pane_footer, "Reveal file", func():
				_host.reveal_on_disk(wrote))
			reveal.disabled = wrote == "" or DccTheme.is_touch()
			reveal.tooltip_text = ("Opens %s in the file manager." % wrote.get_file()
				if wrote != "" else
				("A touch build has no desktop file manager to open." if DccTheme.is_touch()
					else "Nothing has been written on this route yet."))
			var go := DccWidgets.modal_safe(_pane_footer, "Export", func():
				_run_geojson_export_here())
			go.disabled = _bridge == null or not _bridge.has_world
			go.tooltip_text = ("export_geojson -> cartalith_engine::geojson, written with FileAccess. Choose the destination with Browse… if none is set."
				if not go.disabled else "No world is loaded, so there are no entities to describe.")
		"export_assets":
			_footer_note("routes to the Asset library")
			DccWidgets.chip(_pane_footer, "Export pack .zip…", func():
				hide()
				_host.open_asset_library()
				_host.asset_library_window.export_pack_now(), true, 16, 6)
		_:
			_footer_note("nothing to run on this route")
	if kind == "gap":
		var disabled := DccWidgets.chip(_pane_footer, "Run", func(): pass, false, 16, 6)
		disabled.disabled = true
		disabled.tooltip_text = String(route.get("reason", ""))

## **The footer note is the widest thing in this window, and it decided the
## window's width until 2026-09-07.**
##
## Measured before this change, `_panemin_probe.tscn -- --vp 1152x648`, all
## fifteen routes: the pane *body* asks for at most 698 px (`export_maps`), but
## three of this helper's eleven call sites interpolate an absolute filesystem
## path -- `export_world` "writes into <exports root>" rendered **594 px**,
## `export_maps` **684**, `export_gis` **666** -- and a plain `Label` reports
## every one of those pixels as a hard minimum. `_pane_footer` is an
## `HBoxContainer`, so that minimum adds to the chips beside it (`export_world`:
## 594 + 171 + 125 + 145 + 48 of separation = **1083**) and travels straight up
## the pane's `VBoxContainer` to the window, whose contents minimum came out at
## **1372 px** against a declared `min_size.x` of **1024**.
##
## A `Control` cannot be laid out below its combined minimum, so the window's
## root container came out wider than the frame drawing it and everything past
## 1152 px was simply off the window. Measured, not inferred: `export_world`'s
## `Browse…` drew at x 1291..1354 inside a 1152 px window and scored
## `shown=0.000` against its containers' visible rects, `export_maps`' `Choose…`
## at 1259..1322, the same. **The button was never the defect** -- the pane was
## wider than any window it is allowed to open in, and both pickers happened to
## sit in the part that fell off the edge.
##
## **Raising `min_size` instead would not have fixed it, and this was measured
## rather than assumed** -- see `_popup_full()`, which pops at
## `maxi(viewport.x, min_size.x)`. A 1372 px declared minimum pops a 1372 px
## sub-window inside a 1152 px viewport, and the picker moves from *clipped by
## the window* to *outside the application*. `shown` stays 0.000 either way.
##
## So the note gives up its claim on the width instead. It is the one thing in
## the row that is prose rather than an action: it expands into whatever the
## chips leave, trims with an ellipsis when that is not enough, and keeps the
## untrimmed text on its own tooltip so nothing is actually lost. The
## `FOOT_NOTE_MIN_W` floor is what stops the trim from going all the way to the
## 1 px a trimmed `Label` reports as its minimum.
##
## The `spacer()` that used to follow is gone with it: a left-aligned Label that
## fills the row *is* the spacer, and two `SIZE_EXPAND_FILL` siblings would have
## split the slack and given the note half of what it can use.
func _footer_note(text: String) -> void:
	var l := DccTheme.mono_label(text, "text_faint", DccTheme.FS_TINY)
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	## **Pointer and tablet only.** `FOOT_NOTE_MIN_W`'s ceiling is derived from
	## the 736 px this row has at `min_size.x`; a handset's route pane is
	## 376 px wide once `PANE_PAD_X` is paid twice, and the same 160 px there is
	## 43 % of the row. The reason it stands down was that an `HBoxContainer`
	## **adds** the floor to the chips beside it, so a floored note pushed the
	## route's own picker off the screen.
	##
	## **That reason expired on 2026-09-07, in the same batch, and the carve-out
	## is left standing anyway rather than changed on a lane's own authority.**
	## The handset footer is now an `HFlowContainer` (see `_build_pane()`), whose
	## minimum is its **widest child** and not the sum -- so a 160 px note is
	## absorbed instead of added. Measured, floor forced on, `--force-touch
	## --vp 500x1080`, one route per run so the pane sits at its true width:
	##
	##   export_world   note min 1 -> 160, DRAWN 7 px -> 192 px, footer demand
	##                  171 either way, 2 wrapped lines, `fail=0` either way
	##   export_gis     note min 1 -> 160, drawn 302 px both ways, demand
	##                  94 -> 160, still inside the 376 px room
	##   full sweep     `fail=5` with the floor and `fail=5` without -- the same
	##                  five, every one of them a pane-BODY overflow
	##
	## So the floor now costs nothing on a handset and buys back a note that is
	## otherwise **7 px of ellipsis** on the route whose whole job is to say
	## where the file goes. Turning it on is a user-visible behaviour change on
	## a surface this lane was told not to touch, so it is recorded here as a
	## measurement and left for whoever owns that call. Do not repeat the old
	## justification: it was true of an `HBoxContainer` footer and is not true
	## of this one.
	##
	## §13's own answer for the note is unchanged either way --
	## `DccShell.phone_fit()` sets `clip_text` on any `Label` carrying
	## `SIZE_EXPAND`, which this one does -- so the trim still happens on a
	## handset whatever the floor does.
	if not _phone:
		l.custom_minimum_size.x = FOOT_NOTE_MIN_W
	## A trimmed tail is only acceptable because the whole string is still
	## reachable. `Label`'s default `mouse_filter` is `IGNORE`, so the tooltip
	## needs `STOP` to ever appear -- the same pairing `_col_header()` makes.
	l.tooltip_text = text
	l.mouse_filter = Control.MOUSE_FILTER_STOP
	_pane_footer.add_child(l)

# ---------------------------------------------------------------------------
# Validation ▸ Definitions -- DM-10
#
# The header section "2026-09-06: Validation ▸ Definitions (DM-10)" carries the
# finding this pane exists to record and the three shapes the table takes from
# the engine rather than from a preference. Everything below is the drawing.
# ---------------------------------------------------------------------------

## The table's seven columns, in order.
##
## `world_data_window.gd::_cells()` is the shape being followed rather than
## re-invented: a mono/`FS_MICRO`/tracked header band over mono rows, the
## identity column brightest and every other cell at `text`. The one departure
## is `LOCATE`, which is a `Button` and not a `Label`, so this file lays its own
## row out instead of reaching into that window's private helper.
const CHECKS_COLS: Array[String] = ["STATE", "KIND", "NAME", "FIELDS",
	"MESSAGE", "RESOLUTION", "LOCATE"]

## Stretch for the first six columns; `LOCATE` is `CHECKS_LOCATE_W` and fixed.
##
## MESSAGE is the widest because it is the only cell whose text the *engine*
## writes -- a conflict sentence out of `validate_animal` is a whole clause
## ("...but Marsh still carries a 0.60x multiplier -- block Marsh or loosen the
## grazing tolerance.", one of `TL_TERRAIN_KEYS`' ten rows). RESOLUTION is next
## because its longest real value is "Data ▸ Travel library ⇧L ▸ Animals & mounts".
##
## Balanced against the **1 152 px window `_dm10_probe.gd` opens**, which is
## the narrowest this pane was looked at rather than the narrowest it can be;
## the wider the screen, the more of each cell survives. Every cell carries its
## own full text as a tooltip, so clipping loses nothing.
const CHECKS_STRETCH: Array[float] = [0.8, 0.9, 1.3, 1.8, 3.9, 2.3]

const CHECKS_LOCATE_W := 74

## Why `LOCATE` is drawn and disabled on every row, shown as its tooltip.
##
## Not "not implemented": the column is disabled because **nothing in this
## table is a placed thing**. A definition has no coordinates, and the
## `viewport_host.gd::move_view_to(gx, gy)` takes grid cells -- it divides by
## `grid_size()`, checked this pass. Keeping the column means
## the table has the right shape the day a positional validator exists.
const CHECKS_LOCATE_NOTE := "Nothing in this table is a placed thing: these are definitions, and a definition has no coordinates. viewport_host.gd::move_view_to(gx, gy) takes grid cells, so there is nothing to hand it. The column is kept, disabled, so the table already has the right shape the day a positional validator exists."

## `FIELDS`' dash reason on an asset row.
const CHECKS_ASSET_FIELDS_NOTE := "AssetLibrarySession::validate() returns Vec<String> -- one ordered sentence per warning, with the subject inside the sentence. There is no field list to put here, so this is a dash rather than a guess."

## `FIELDS`' dash reason on a conflicting row.
const CHECKS_CONFLICT_FIELDS_NOTE := "ValidationState::Conflicting carries sentences, not field names -- \"every detected conflict, as a human-readable sentence\" (travel_library.rs). The fields it is between are named inside the message."

## `NAME`'s dash reason on an asset row whose pack has no name.
const CHECKS_ASSET_NAME_NOTE := "as_pack_info().name is empty -- the pack has not been named. That absence is itself one of the warnings in this table."

## The Map geometry row's reason -- the finding, in the UI, and worded to
## survive being read out of context. Every clause was measured at the symbol
## on 2026-09-06; the header section says where.
const CHECKS_GEOMETRY_NOTE := "No geometry-validity entry point exists over map layers, and that is a finding rather than a to-do. poly_self_intersects lives in cartalith-urban and has two callers, blocks.rs and inset_poly -- its subject is city blocks, not territory, coastline or province rings, and it does not cross the gdext boundary. In cartalith-spatial/src/geo.rs an unclosed ring is NORMAL output: the module doc says a traced ring is not necessarily closed, ring_area deliberately omits the closing segment, and golden_parity_geo.rs asserts the unclosed shape. Reporting it as an error would be wrong, not merely absent. Ring orientation is checked nowhere -- ensure_ccw imposes a winding; nothing reports one as invalid."

## The four travel-definition validators, as this pane needs them. The asset
## one is a single session rather than a list and is drawn separately.
##
## `kind` is `tl_list`'s own argument. The RESOLUTION column's tab name is
## resolved from `TravelLibraryWindow.KINDS` at build time rather than retyped
## here, so it cannot name a tab that moved.
##
## `states` is what each validator can actually return. **Party set-ups list
## two, not three**, and that is `validate_party_preset`'s own doc: *"party
## set-ups carry no terrain/grazing constraint fields to conflict-check against
## each other -- only completeness is meaningful, so
## ValidationState::Conflicting never appears here."*
const CHECKS_SOURCES: Array[Dictionary] = [
	{"kind": "animal", "name": "Animal", "fn": "validate_animal",
		"states": "ok · incomplete · conflicting"},
	{"kind": "vehicle", "name": "Vehicle", "fn": "validate_vehicle",
		"states": "ok · incomplete · conflicting"},
	{"kind": "vessel", "name": "Vessel", "fn": "validate_vessel",
		"states": "ok · incomplete · conflicting"},
	{"kind": "preset", "name": "Party preset", "fn": "validate_party_preset",
		"states": "ok · incomplete"},
]

## Both bindings, checked on the `WorldGen` object itself rather than through
## `EngineBridge`'s forwarders -- those answer `[]` for a build without them,
## which is exactly the value a healthy, empty result has. Distinguishing the
## two is the whole reason this exists: an all-clear that actually means "this
## build cannot ask" is the worst row this table could draw.
func _checks_api() -> bool:
	return (_bridge != null and _bridge.world_gen != null
		and _bridge.world_gen.has_method("tl_list")
		and _bridge.world_gen.has_method("as_validate"))

## Every validator, run. Returns `{"rows": Array, "defs": int}` -- `defs` is how
## many travel definitions were examined, so the summary can say what was
## looked at as well as what came back.
##
## Pure: five functions over state already in memory. Nothing here writes.
func _checks_run() -> Dictionary:
	var rows: Array = []
	var defs := 0
	for src in CHECKS_SOURCES:
		var kind := String(src["kind"])
		var listed: Array = _bridge.tl_list(kind)
		defs += listed.size()
		for r in listed:
			var d: Dictionary = r
			var state := String(d.get("validation_state", "ok"))
			if state == "ok":
				continue
			var missing: PackedStringArray = d.get("validation_missing", PackedStringArray())
			var conflicts: PackedStringArray = d.get("validation_conflicts", PackedStringArray())
			rows.append({
				"state": state,
				"kind": String(src["name"]),
				"name": String(d.get("name", "")),
				"name_reason": "",
				"fields": ", ".join(missing),
				"fields_reason": "" if state == "incomplete" else CHECKS_CONFLICT_FIELDS_NOTE,
				"message": ("%d constraint field%s unset — every one is named in FIELDS."
						% [missing.size(), "" if missing.size() == 1 else "s"]
					if state == "incomplete" else " ".join(conflicts)),
				"resolution": "Data ▸ Travel library ⇧L ▸ %s" % _checks_tab_label(kind),
			})
	var pack: Dictionary = _bridge.as_pack_info()
	var pack_name := String(pack.get("name", ""))
	for w in _bridge.as_validate():
		rows.append({
			"state": "warning",
			"kind": "Asset library",
			"name": pack_name,
			"name_reason": "" if pack_name != "" else CHECKS_ASSET_NAME_NOTE,
			"fields": "",
			"fields_reason": CHECKS_ASSET_FIELDS_NOTE,
			"message": String(w),
			"resolution": "Assets ▸ ⧉ Asset library",
		})
	return {"rows": rows, "defs": defs}

## The Travel library window's own tab name for a `tl_list` kind, read from
## `TravelLibraryWindow.KINDS` so RESOLUTION cannot name a tab that moved.
func _checks_tab_label(kind: String) -> String:
	for k in TravelLibraryWindow.KINDS:
		if String((k as Dictionary)["key"]) == kind:
			return String((k as Dictionary)["label"])
	return kind

func _build_checks_pane() -> void:
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_pane_body.add_child(col)

	if not _checks_api():
		## Cleared, not left: a stale count from a previous build's scan would
		## reach the status line under a pane that just said it cannot ask.
		_checks_last = {}
		_col_header(col, "FINDINGS")
		DccWidgets.note(col,
			"This build's GDExtension predates the validator bindings (WorldGen::tl_list and WorldGen::as_validate). Nothing is reported below -- deliberately, rather than an all-clear, which a build that cannot ask would be unable to tell apart from a healthy library.")
		_col_header(col, "COVERAGE")
		_checks_geometry_group(col)
		return

	var run := _checks_run()
	## Kept so `_refresh_status()` reports the same numbers the pane drew
	## rather than scanning a third time and risking a different answer.
	_checks_last = run
	_checks_summary(col, run)
	_col_header(col, "FINDINGS")
	var rows: Array = run["rows"]
	if rows.is_empty():
		_checks_all_clear(col, run)
	else:
		_checks_table(col, rows)
	_col_header(col, "COVERAGE")
	_checks_coverage(col)
	_checks_geometry_group(col)

## The one-line count above the table. Says what was **examined** as well as
## what came back, so a short table cannot be misread as a short scan.
func _checks_summary(col: Control, run: Dictionary) -> void:
	var n: int = (run["rows"] as Array).size()
	var l := DccTheme.mono_label(
		"%d finding%s · %d definitions and 1 asset library examined"
			% [n, "" if n == 1 else "s", int(run["defs"])],
		"text_bright" if n > 0 else "good", DccTheme.FS_TINY)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col.add_child(l)
	DccWidgets.note(col,
		"Nothing on this route needs a generated world: the travel library is stock-seeded from init() and the asset library is a session. All five validators are pure -- they read, and nothing here writes.")

## The all-clear, with the counts. **Not an empty pane**: a blank body and a
## build that cannot ask look identical, and the difference between "nothing
## found" and "nothing looked" is this table's whole subject.
func _checks_all_clear(col: Control, run: Dictionary) -> void:
	var wrap := PanelContainer.new()
	var sb := DccTheme.flat(DccTheme.c("sunken"), DccWidgets.MODAL_INSET_RADIUS)
	sb.content_margin_left = PATTERN_RECEIPT_PAD_X
	sb.content_margin_right = PATTERN_RECEIPT_PAD_X
	sb.content_margin_top = PATTERN_RECEIPT_PAD_Y
	sb.content_margin_bottom = PATTERN_RECEIPT_PAD_Y
	wrap.add_theme_stylebox_override("panel", sb)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 4)
	wrap.add_child(body)
	col.add_child(wrap)
	body.add_child(DccTheme.mono_label("✓  every validator returned ok",
		"good", DccTheme.FS_SMALL))
	var detail := DccTheme.mono_label(
		"%d travel definitions across four types, and the asset library's own AssetValidator.run() — 0 warnings."
			% int(run["defs"]), "text_faint", DccTheme.FS_MICRO)
	detail.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	detail.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(detail)

## The header band, then one row per finding.
func _checks_table(col: Control, rows: Array) -> void:
	if not _phone:
		col.add_child(_checks_header_row())
		col.add_child(DccTheme.rule())
	for r in rows:
		_checks_row(col, r as Dictionary)

func _checks_header_row() -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	for i in CHECKS_COLS.size():
		var l := DccTheme.mono_label(CHECKS_COLS[i], "text_faint",
			DccTheme.FS_MICRO, 1, true)
		if i == CHECKS_COLS.size() - 1:
			l.custom_minimum_size.x = CHECKS_LOCATE_W
		else:
			l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			l.size_flags_stretch_ratio = CHECKS_STRETCH[i]
		row.add_child(l)
	return row

## One cell. An empty `text` means the value is genuinely absent, and the cell
## is then a dash carrying `reason` as its tooltip -- never a plausible-looking
## stand-in. Every dash this table can produce has a reason and the caller
## passes it; a dash with an empty tooltip is a bug, and the probe asserts so.
func _checks_cell(row: Control, text: String, i: int, token: String,
		reason: String = "") -> Label:
	var l: Label
	if text == "":
		l = DccTheme.mono_label("—", "text_ghost", DccTheme.FS_SMALL)
		l.tooltip_text = reason
	else:
		l = DccTheme.mono_label(text, token, DccTheme.FS_SMALL)
		l.tooltip_text = text
	l.mouse_filter = Control.MOUSE_FILTER_STOP
	l.clip_text = true
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.size_flags_stretch_ratio = CHECKS_STRETCH[i]
	row.add_child(l)
	return l

## `warn` for an incomplete definition and for an asset warning, `block` for a
## conflict -- the theme's own verdict vocabulary, whose positive half `good`
## the all-clear uses. All three are defined for both palettes.
func _checks_state_token(state: String) -> String:
	return "block" if state == "conflicting" else "warn"

func _checks_row(col: Control, r: Dictionary) -> void:
	if _phone:
		_checks_phone_row(col, r)
		return
	var state := String(r["state"])
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	row.custom_minimum_size.y = 20
	_checks_cell(row, state, 0, _checks_state_token(state))
	_checks_cell(row, String(r["kind"]), 1, "text")
	_checks_cell(row, String(r["name"]), 2, "text_bright",
		String(r.get("name_reason", "")))
	_checks_cell(row, String(r["fields"]), 3, "text",
		String(r.get("fields_reason", "")))
	_checks_cell(row, String(r["message"]), 4, "text")
	_checks_cell(row, String(r["resolution"]), 5, "text_dim")
	_checks_locate(row)
	col.add_child(row)

## PH-12: seven `clip_text` columns across 393 dp is ~50 dp each, which is not a
## table. Same answer `world_data_window.gd` reached for its six -- the row
## becomes a stack, identity over the rest as prose -- and the header band is
## dropped with the columns rather than left labelling a table that is not on
## screen. `LOCATE` stays: "present and disabled on every row" is a statement
## about rows, not about a density.
func _checks_phone_row(col: Control, r: Dictionary) -> void:
	var rule := ColorRect.new()
	rule.color = DccTheme.c("line_soft")
	rule.custom_minimum_size.y = 1
	col.add_child(rule)
	var line := HBoxContainer.new()
	line.add_theme_constant_override("separation", 10)
	line.custom_minimum_size.y = DccTheme.PHONE_TAP_MIN + 8
	col.add_child(line)
	var stack := VBoxContainer.new()
	stack.add_theme_constant_override("separation", 2)
	stack.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	line.add_child(stack)
	var head := DccTheme.mono_label("%s · %s · %s"
		% [String(r["state"]), String(r["kind"]),
			String(r["name"]) if String(r["name"]) != "" else "—"],
		_checks_state_token(String(r["state"])), DccTheme.FS_SMALL)
	head.clip_text = true
	head.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	stack.add_child(head)
	var sub := DccTheme.mono_label(String(r["message"]), "text_ghost",
		DccTheme.FS_MICRO)
	sub.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	sub.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	stack.add_child(sub)
	var res := DccTheme.mono_label(String(r["resolution"]), "text_faint",
		DccTheme.FS_MICRO)
	res.clip_text = true
	res.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	stack.add_child(res)
	_checks_locate(line)

## Present and disabled, on every row and at every density. `CHECKS_LOCATE_NOTE`
## is why it is drawn at all, and it is the button's tooltip so a reader gets
## the reason rather than an inert control with nothing to say.
func _checks_locate(row: Control) -> Button:
	var b := DccWidgets.chip(row, "Locate", func(): pass, false, 8, 2)
	b.disabled = true
	b.tooltip_text = CHECKS_LOCATE_NOTE
	b.custom_minimum_size.x = CHECKS_LOCATE_W
	return b

## What ran, by validator, with what it can return and how much it saw. This is
## the block that makes a short -- or empty -- FINDINGS table readable: five
## named validators with real counts beside them, rather than a silence.
func _checks_coverage(col: Control) -> void:
	for src in CHECKS_SOURCES:
		var n: int = (_bridge.tl_list(String(src["kind"])) as Array).size()
		_checks_coverage_row(col, String(src["name"]), String(src["fn"]),
			String(src["states"]), "%d checked" % n, "")
	_checks_coverage_row(col, "Asset library",
		"AssetLibrarySession::validate()", "ordered warning strings",
		"1 session", "")

## The dashed group. Present, named, and carrying the code's own reason -- not
## omitted, because an absent group reads as an oversight and this one is a
## measured finding.
func _checks_geometry_group(col: Control) -> void:
	_checks_coverage_row(col, "Map geometry", "—", "—", "—",
		CHECKS_GEOMETRY_NOTE)

## One coverage row. A non-empty `reason` makes it the dashed kind: the row
## carries the reason as a tooltip on every cell and as a wrapped line beneath
## it, so it reads as a stated absence rather than as a row that failed to load.
func _checks_coverage_row(col: Control, name: String, fn: String,
		states: String, count: String, reason: String) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	row.custom_minimum_size.y = 18
	var dashed := reason != ""
	var parts: Array[String] = [name, fn, states, count]
	var ratios: Array[float] = [1.2, 2.0, 1.8, 1.0]
	for i in parts.size():
		var token := "text_ghost"
		if not dashed:
			token = "text_bright" if i == 0 else "text"
		elif i == 0:
			token = "text_dim"
		var l := DccTheme.mono_label(parts[i], token, DccTheme.FS_SMALL)
		l.clip_text = true
		l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		l.size_flags_stretch_ratio = ratios[i]
		if dashed:
			l.tooltip_text = reason
			l.mouse_filter = Control.MOUSE_FILTER_STOP
		row.add_child(l)
	col.add_child(row)
	if not dashed:
		return
	var why := DccTheme.mono_label(reason, "text_ghost", DccTheme.FS_MICRO)
	why.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	why.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col.add_child(why)

## The action row for this route -- **one button, and it is not a repair.**
##
## `RUN ALL VALIDATORS` re-selects the route, which rebuilds the pane and so
## re-calls all five. There is deliberately no `FIX SELECTED`: the engine has
## validators only, and nothing in `cartalith-civ::travel_library` or
## `AssetLibrarySession` changes state in response to one. Building a repair
## affordance over nothing is the failure this route was written to avoid.
func _checks_actions() -> void:
	_footer_note("validators are pure — nothing here writes")
	var run := DccWidgets.chip(_pane_footer, "Run all validators", func():
		_select_route("val_defs"), true, 16, 6)
	run.disabled = not _checks_api()
	run.tooltip_text = ("Re-calls validate_animal, validate_vehicle, validate_vessel, validate_party_preset and AssetLibrarySession::validate(). All five are pure functions over state already in memory, so re-running them costs nothing and cannot fail differently."
		if not run.disabled
		else "This build's GDExtension carries neither WorldGen::tl_list nor WorldGen::as_validate.")

# ---------------------------------------------------------------------------
# Export ▸ World Data -- the export raster and the channel atlas
# (`PARITY_AUDIT.md` §5 item 14, `GUI_GAP_REGISTER.md` DM-04)
#
# The reference puts these four controls in its header bar next to Export:
# `bakeRes` (2K/4K/8K/16K/32K since ruling 15), `bakeTiles`, `chanAtlasChk`
# and `layersPreviewChk`.
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
	## PH-12: the canvas's two equal columns become one stacked column on a
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
			"A world opened from a file -- a .zip save or a reopened project -- carries none of the tectonic fields these are derived from, so the atlas needs a world generated in this session.")

func _build_wd_output_column(col: Control, api: bool) -> void:
	_col_header(col, "OUTPUT")
	var dest_row := _row(col, "Folder")
	_well_label(dest_row, DccSettings.storage_root("exports"),
		"DccSettings' own exports root -- the same folder Export ▸ Maps and Export ▸ GIS write into.")
	## The owner, 2026-09-07, of this window and of Storage locations: *"it just
	## accepts a path, it doesn't open a file explorer or browser to manually
	## navigate and appoint."* Measured on this route the same day: it was the
	## one destination well of the three in this window with **no button beside
	## it** (`export_maps` has `Choose…`, `export_gis` has `Browse…`), so the
	## only way to move where World Data writes was File ▸ Storage locations, a
	## different surface. The well already said it *is* the exports root; this
	## makes the row that says so able to change it.
	DccWidgets.chip(dest_row, "Browse…", func(): _pick_exports_root(), false, 8, 3)

	var est := _wd_estimate()
	if api and not est.is_empty():
		var peak_row := _row(col, "Peak memory")
		_well_label(peak_row, _fmt_bytes(int(est.get("peak_bytes", 0))),
			"3 bytes per output pixel for the raster, 4 for the local-contrast pass' luma and 16 for its blur buffers -- 23 B/px, measured at 21.7. Reported by the binding, not modelled here -- at 16K and 32K the export is refused outright if the device cannot hold it.")
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
	## The height field as a 16-bit grayscale PNG. Sits beside the colour
	## raster because it is the same bake at the same width -- and exists at
	## all because `import_heightmap` has been able to READ this format since
	## Phase 1 while nothing could write it.
	var hm := DccWidgets.chip(_pane_footer, "Export heightmap…", func():
		_pick_heightmap_destination(), false, 16, 6)
	hm.disabled = _bridge == null or not _bridge.has_world
	hm.tooltip_text = ("The committed height field as a 16-bit grayscale PNG, at the width "
		+ "selected above -- for game engines and terrain editors, and readable back by "
		+ "File ▸ New world ▸ Import a heightmap.

"
		+ "Does NOT include an open Sculpt draft: that is uncommitted state, so commit it first "
		+ "if you want it in the export.")

## The World Data route's `Folder` row picker. Unlike the other two
## destinations in this window it does not hold a path of its own: the row
## draws `DccSettings.storage_root("exports")`, so choosing here writes the
## setting -- the same one `app.gd::_browse_root()` writes from File ▸ Storage
## locations, and persisted the same way, immediately and with no confirm step.
##
## Writing a *shared* setting from a route pane is what makes
## `_reroot_defaults()` necessary; see its own comment.
func _pick_exports_root() -> void:
	var was := DccSettings.storage_root("exports")
	DccBrowseDialog.choose_folder(self, "Choose the exports folder", was,
		"every route in this window that writes without asking writes here",
		func(path: String):
			if path == was:
				return
			DccSettings.set_storage_root("exports", path)
			_reroot_defaults(was, path)
			_rebuild_world_data())

## `setup()` pre-fills `_tx_dest` and `_gis_dest` from the exports root and
## then caches them for the window's whole life, so moving the root leaves both
## pointing into the folder it used to be -- two views of one setting that
## disagree, which is the failure this project keeps re-finding.
##
## Each is re-derived here **only while it still holds exactly what `setup()`
## put there**. A destination the user picked with its own `Choose…` / `Browse…`
## is an answer, not a default, and moving the root must not silently overwrite
## it. The pair below is the whole list: `setup()` derives two fields from that
## root and no other cache reads it.
func _reroot_defaults(old_root: String, new_root: String) -> void:
	if _tx_dest == old_root.path_join(TX_DEFAULT_NAME):
		_tx_dest = new_root.path_join(TX_DEFAULT_NAME)
	if _gis_dest == old_root.path_join(GIS_DEFAULT_NAME):
		_gis_dest = new_root.path_join(GIS_DEFAULT_NAME)

func _wd_estimate() -> Dictionary:
	if not _raster_api():
		return {}
	return _bridge.world_gen.export_raster_estimate(_wd_width)

func _rebuild_world_data() -> void:
	if _selected_id == "export_world":
		_select_route("export_world")

## Every picker in this window is `DccBrowseDialog` -- the "Select folder
## dialog 1920" browser from `design/Cartalith DCC Shell.dc.html`, whose own
## comment says it *"replaces the stock OS tree picker"*. Five stock
## `FileDialog`s lived here until 2026-09-05, each raising Godot's own generic
## file browser (`use_native_dialog` is set nowhere in this project, so these
## were never the Windows shell dialog either) in the middle of a shell that
## draws every other pixel itself.
##
## The one capability the swap does not inherit is the overwrite prompt, so it
## is rebuilt here -- see `_overwrite_guard()`.
##
## The rest mapped across without loss: `add_filter` becomes the browser's
## extension list (non-matching files stay visible but dimmed rather than
## being hidden, which is the design's own choice), `current_dir` the start
## directory, `current_file` the save name, and `FILE_MODE_OPEN_DIR` /
## `FILE_MODE_SAVE_FILE` the `choose_folder` / `choose_save_path` entry
## points. The stock dialog also appended the filter's extension to a name
## that lacked one -- measured at 4.7.1, `wrong.txt` came back as
## `wrong.txt.png` -- and `_save_path()` does the same thing.
func _pick_raster_destination() -> void:
	var start := DccSettings.storage_root("exports")
	if _wd_tiled:
		## Tiled mode writes a *directory* of tiles plus index.json, so the
		## picker asks for one -- the reference's own tiles/ prefix, as a real
		## folder rather than a path inside a zip.
		DccBrowseDialog.choose_folder(self, "Export map tiles into…", start,
			"one .png per tile plus index.json, written into the folder you choose",
			func(path: String): _run_raster_export(path))
	else:
		## `exportZip`'s own name for this file.
		DccBrowseDialog.choose_save_path(self, "Export map raster", "png", start,
			"the colour map as one .png, %d px wide" % _wd_width, "map.png",
			func(path: String): _overwrite_guard(path, func(): _run_raster_export(path)))

## The heightmap's own picker. A single file, always -- unlike the colour
## raster it has no tiled mode, because the formats that read a heightmap want
## one image.
func _pick_heightmap_destination() -> void:
	DccBrowseDialog.choose_save_path(self, "Export heightmap", "png",
		DccSettings.storage_root("exports"),
		"16-bit grayscale .png, %d px wide" % _wd_width, "heightmap.png",
		func(path: String): _overwrite_guard(path, func(): _run_heightmap_export(path)))

## Godot's stock `FileDialog` would not hand a save path back to its caller
## while a file of that name already existed: it interposed *File "…" already
## exists. Do you want to overwrite it?* and emitted `file_selected` only once
## that was answered. Measured against 4.7.1 on 2026-09-05, by driving a
## SAVE-mode dialog at an existing file and watching `file_selected` not fire.
##
## `DccBrowseDialog` deliberately does not do this -- `choose_save_path`'s own
## doc gives the reason: the caller "is the only side that knows what is about
## to be overwritten". So every save picker in this window comes through here,
## which is the shape `app.gd::save_project_as()` already uses for File ▸ Save
## as…. Without it the eight-picker swap would have quietly removed the only
## thing standing between a mistyped name and a destroyed export.
func _overwrite_guard(path: String, then: Callable) -> void:
	if not FileAccess.file_exists(path):
		then.call()
		return
	_host._confirm("Overwrite %s?" % path.get_file(),
		"That file already exists. Exporting replaces it.", "Overwrite", then)

func _run_heightmap_export(path: String) -> void:
	if _bridge == null or not _bridge.has_world:
		return
	_status_left.text = "writing heightmap…"
	_status_left.add_theme_color_override("font_color", DccTheme.c("accent"))
	var r: Dictionary = _bridge.export_heightmap_png(
		ProjectSettings.globalize_path(path), _wd_width)
	_record_wd_run("heightmap %dK" % int(round(float(_wd_width) / 1024.0)), r)
	if bool(r.get("ok", false)):
		var shown: bool = _host.reveal_on_disk(path)
		_host.set_status("hint", "heightmap %d × %d px — %s in %.1f s → %s"
			% [int(r.get("width", 0)), int(r.get("height", 0)),
				_fmt_bytes(int(r.get("bytes", 0))), float(r.get("ms", 0.0)) / 1000.0,
				(path.get_file() if shown else path)], "accent")
	else:
		_host.set_status("hint", "heightmap export failed — %s"
			% String(r.get("error", "see the Godot log")), "warn")
	_rebuild_world_data()

## A folder, not a file: the atlas is several PNGs plus its own index, and
## `WD_ATLAS_NOTE` above names them. No `_overwrite_guard()` -- the pick
## returns a directory and the names written inside it are the engine's, not
## the user's, so there is no name here for a typo to destroy. `_run_atlas_
## export()` will still replace a file of the same name it wrote last time,
## which is what it did before this swap too.
func _pick_atlas_destination() -> void:
	DccBrowseDialog.choose_folder(self, "Export channel atlas into…",
		DccSettings.storage_root("exports"),
		"one .png per channel group plus atlas/index.json, written into the folder you choose",
		func(path: String): _run_atlas_export(path))

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
		## The size and the duration were already here; WHERE it went was not,
		## and `r.files` -- the list of what was actually written -- was read
		## only for its `.size()`. An export that finishes into silence leaves
		## the user to go looking for their own file.
		var shown: bool = _host.reveal_on_disk(path)
		_host.set_status("hint", "exported %d × %d px%s — %s in %.1f s → %s"
			% [int(r.get("width", 0)), int(r.get("height", 0)),
				(" as %d tiles" % files.size()) if _wd_tiled else "",
				_fmt_bytes(int(r.get("bytes", 0))), float(r.get("ms", 0.0)) / 1000.0,
				(path.get_file() if shown else path)], "accent")
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

## `route` and `msec` are carried on every run record so `_last_route_run()` and
## `_ago()` can answer for any route, not only the one the pattern pane happens
## to draw today. `msec` is monotonic (`Time.get_ticks_msec()`); `stamp` is the
## wall clock the RECENT RUNS block prints, and the two are not interchangeable.
func _record_wd_run(label: String, r: Dictionary) -> void:
	var t := Time.get_time_dict_from_system()
	_runs.push_front({
		"stamp": "%02d:%02d" % [int(t["hour"]), int(t["minute"])],
		"msec": Time.get_ticks_msec(), "route": "export_world",
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
# does **not** do is the pyramid the canvas draws -- that is `slippy_export_tiles`,
# behind the scheme row's XYZ / TMS / WMTS segments. Every canvas control that
# neither export backs is drawn in its canvas position and disabled with its
# reason.
# ---------------------------------------------------------------------------

func _build_tile_export_pane() -> void:
	## PH-12: the canvas's two equal columns become one stacked column on a
	## phone -- `COL_GAP` apart, both `EXPAND_FILL`, they would each get half of
	## 393 dp and every `120px label · control` row inside them would overlap
	## rather than clip.
	##
	## **Extended to tablet touch density, 2026-09-13** (`OUTSTANDING_WORK.md`
	## / `TABLET_UI_SPEC.md`, re-opened after the PANE-MIN lane filed it
	## 2026-09-07 as pre-existing and out of that lane's footer-only scope).
	## PH-12's own reasoning is about CONTROL size, not about being a phone:
	## side by side, `left`'s PROJECTION row alone (the CRS three-segment
	## control) demands 359 px of touch-sized buttons, and the two columns
	## together drove this pane's body to **887 px against 698 at pointer
	## density** -- `_panemin_probe.tscn --route export_maps --verbose
	## --force-touch --vp 1600x1000`, contents_min 1176 against a declared
	## min_size.x of 1024, the single failure in that probe's tablet run.
	## `_phone` alone missed it because a tablet is touch without being a
	## phone (`DccTheme.is_tablet()`'s own header: "a phone is `is_touch()`
	## too"; the same fact in reverse -- `is_touch()` is not `is_phone()`
	## either). Stacking trades the unscrollable WIDTH failure for ordinary
	## extra height, which `_build_pane()`'s own `ScrollContainer` already
	## exists to absorb (vertical scroll is enabled; only the horizontal axis
	## is `SCROLL_MODE_DISABLED`, DS-03's "keep everything, reflow only" doing
	## the rest). Pointer and phone are unchanged BY CONSTRUCTION, not just by
	## measurement: pointer has `_phone=false` and `is_touch()=false`, so the
	## `or` is false either way (HBox, as before); phone already had
	## `_phone=true`, so the `or` was already true (VBox, as before). Only
	## tablet (`_phone=false`, `is_touch()=true`) changes outcome -- the one
	## density this condition had no term for.
	var grid: BoxContainer = VBoxContainer.new() if (_phone or DccTheme.is_touch()) \
		else HBoxContainer.new()
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
		{"text": "grid + index.json", "enabled": true, "tip": SCHEME_NOTE},
		{"text": "XYZ", "enabled": true, "tip": SCHEME_NOTE},
		{"text": "TMS", "enabled": true, "tip": SCHEME_NOTE},
		{"text": "WMTS", "enabled": true, "tip": SCHEME_NOTE},
	], _tx_scheme, func(i: int):
		_tx_scheme = i
		_rebuild_tile_export())

	if _tx_pyramid():
		_build_pyramid_rows(col)
		return

	## The marquee grid has no zoom ladder: its dimension is `cols`/`rows`.
	var grid_row := _row(col, "Tile grid")
	var grid_items: Array = []
	for n in GRID_CHOICES:
		grid_items.append({"text": "%d×%d" % [n, n], "enabled": true})
	var grid_sel := GRID_CHOICES.find(_tx_cols) if _tx_cols == _tx_rows else -1
	_segments(grid_row, grid_items, grid_sel, func(i: int):
		_tx_cols = GRID_CHOICES[i]
		_tx_rows = GRID_CHOICES[i]
		_push_tile_grid_preview()
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

func _tx_pyramid() -> bool:
	return _tx_scheme > 0

## Tiles the current settings write: `cols × rows` for the grid, and
## `(4^(N+1) − 1) / 3` per pixel density for the pyramid -- the same count
## `cartalith_spatial::pyramid_tile_count` gives the bake.
func _tx_tile_count() -> int:
	if not _tx_pyramid():
		return _tx_cols * _tx_rows
	var n := 0
	for z in _tx_zmax + 1:
		n += 1 << (2 * z)
	return n * (2 if _tx_retina else 1)

## The TILES column in pyramid mode: the canvas's zoom range, tile size, the
## fixed PNG format, and the retina toggle. No gzip / visual / ridged / ocean
## rows -- the pyramid writes colour PNGs only, and says so in Format.
func _build_pyramid_rows(col: Control) -> void:
	var zoom_row := _row(col, "Zoom range")
	var zoom_items: Array = []
	for n in ZOOM_CHOICES:
		zoom_items.append({"text": "0–%d" % n, "enabled": n <= 6, "tip": "" if n <= 6 else ZOOM_NOTE})
	_segments(zoom_row, zoom_items, ZOOM_CHOICES.find(_tx_zmax), func(i: int):
		_tx_zmax = ZOOM_CHOICES[i]
		_rebuild_tile_export())

	var size_row := _row(col, "Tile size")
	var size_items: Array = []
	for n in TILE_SIZES:
		size_items.append({"text": "%d px" % n, "enabled": true})
	_segments(size_row, size_items, TILE_SIZES.find(_tx_tile), func(i: int):
		_tx_tile = TILE_SIZES[i]
		_rebuild_tile_export())

	var fmt_row := _row(col, "Format")
	_well_label(fmt_row, "PNG · tiles.json",
		"Every tile is a colour + hillshade PNG of the bake's own pyramid tile at that address (cartalith_engine::slippy_export). tiles.json is a TileJSON 3.0.0 manifest for XYZ/TMS, with the zoom ladder under a cartalith key; WMTS gets the same manifest without the TileJSON claim, since TileJSON has no WMTS scheme.")

	_check(col, "Retina @2x variants", _tx_retina, func():
		_tx_retina = not _tx_retina
		_rebuild_tile_export(),
		"×2 files" if _tx_retina else "", true,
		"Writes each tile again at twice the pixels over the same ground: z/x/y@2x.png for XYZ/TMS (Leaflet's {r}), a cartalith@2x TileMatrixSet for WMTS.")

	_check(col, "Skip all-ocean tiles", false, func(): pass, "", false,
		"No tile is skipped: slippy_export_tiles writes every tile of every level. A client asking for a missing ocean tile would get a 404 rather than sea, so skipping also needs a fallback the manifest does not describe yet.")

func _build_projection_column(col: Control, region: Dictionary) -> void:
	_col_header(col, "PROJECTION", CRS_NOTE)

	var crs_row := _row(col, "CRS")
	_segments(crs_row, [
		{"text": "world cell grid", "enabled": true},
		{"text": "EPSG:3857", "enabled": false, "tip": CRS_NOTE},
		{"text": "EPSG:4326", "enabled": false, "tip": CRS_NOTE},
	], 0, func(_i: int): pass)

	var bounds_row := _row(col, "World bounds")
	if _tx_pyramid():
		_well_label(bounds_row, "whole world",
			"The pyramid covers the whole world at every level; the Region-select marquee is not used.")
	elif region.is_empty():
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
	if _tx_pyramid():
		_check(col, "Elevation (RG16)", false, func(): pass, "", false,
			"The pyramid export writes colour PNG tiles only. Height data is the grid + index.json scheme's, or the atlas's own World/ archive.")
		_check(col, "Relief + hillshade", true, func(): pass, "always", false,
			"Every pyramid tile is the colour + hillshade raster; a slippy map of blank tiles is not an export.")
	else:
		_build_grid_layer_rows(col)
	_check(col, "Political tint", false, func(): pass, "", false, LAYER_NOTE)
	_check(col, "Labels & icons", false, func(): pass, "raster", false, LAYER_NOTE)
	_check(col, "Rivers & coastlines", false, func(): pass, "", false, LAYER_NOTE)

func _build_grid_layer_rows(col: Control) -> void:
	_check(col, "Elevation (RG16)", true, func(): pass, "always", false,
		"Every export writes the height tiles; there is no option to omit them.")
	_check(col, "Relief + hillshade", _tx_visual, func():
		_tx_visual = not _tx_visual
		_rebuild_tile_export(), "", true,
		"The same `visual` option as the TILES column above -- the canvas lists it in both places, so both are drawn and both drive the one flag.")

func _build_output_column(col: Control) -> void:
	_col_header(col, "OUTPUT")

	var dest_row := _row(col, "Destination")
	_well_label(dest_row, _tx_dest if _tx_dest != "" else "—", _tx_dest)
	DccWidgets.chip(dest_row, "Choose…", func(): _pick_destination(), false, 8, 3)

	var pack_row := _row(col, "Packaging")
	_segments(pack_row, [
		{"text": ".ctl", "enabled": true},
		{"text": "folder", "enabled": false, "tip": PACKAGING_NOTE},
		{"text": "MBTiles", "enabled": false, "tip": PACKAGING_NOTE},
	], 0, func(_i: int): pass)

	if _tx_pyramid():
		_check(col, "Emit tiles.json", true, func(): pass, "always", false,
			"slippy_export_tiles always writes tiles.json -- the address template, zoom range, tile size and per-level ground resolution. It is not optional.")
	else:
		_check(col, "Emit tiles/index.json", true, func(): pass, "always", false,
			"export_region_tiles always writes tiles/index.json -- the per-tile file names, dimensions and world metadata. It is not optional.")
	_check(col, "Emit leaflet-preview.html", _tx_pyramid(), func(): pass,
		"always" if _tx_pyramid() else "", false, PREVIEW_NOTE)
	_check(col, "Emit style.json + attribution", false, func(): pass, "", false, STYLE_NOTE)

func _build_estimate_block(col: Control, region: Dictionary) -> void:
	_col_header(col, "ESTIMATE")
	var block := _block(col)
	var tiles := _tx_tile_count()
	if _tx_pyramid():
		## One PNG per tile plus leaflet-preview.html and tiles.json. The long
		## edge is `_tx_tile`; the short edge follows the world's aspect
		## (`tile_dims`), so only the long edge is stated.
		_kv(block, "tiles", "%d (z0–%d%s)" % [tiles, _tx_zmax, " · 1x + 2x" if _tx_retina else ""])
		_kv(block, "files in archive", "%d" % (tiles + 2))
		_kv(block, "tile size", "%d px long edge" % _tx_tile)
	else:
		_kv(block, "tiles", "%d (%d × %d)" % [tiles, _tx_cols, _tx_rows])
		_kv(block, "files in archive", "%d" % (tiles * (2 if _tx_visual else 1) + 1))
		_kv(block, "tile size", "%d × %d px" % [_tx_tile, _tx_tile])
	var last := _last_run()
	_kv(block, "size on disk",
		_fmt_bytes(int(last.get("bytes", 0))) if not last.is_empty() else "measured by Dry run",
		"text" if not last.is_empty() else "text_ghost")
	_kv(block, "render time",
		("%.1f s" % float(last.get("secs", 0.0))) if not last.is_empty() else "measured by Dry run",
		"text" if not last.is_empty() else "text_ghost")
	if _tx_pyramid():
		_kv(block, "source", "whole world · synthesised, not read from the atlas")
	elif region.is_empty():
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

## §9's MARKDOWN VAULT block, reading the live vault rather than asserting it
## away. See `VAULT_NOTE` for what changed and what of DM-14 genuinely remains.
##
## Everything on screen here traces to one `vault_info()` call plus, when the
## backlink index is already built, `vault_backlink_stats()`. Neither walks the
## folder, so this is safe from a pane rebuild -- the note count is reported
## only when the index already holds one, and says so plainly when it does not,
## rather than forcing a filesystem scan to fill a caption.
func _build_vault_block(col: Control) -> void:
	var info: Dictionary = _bridge.vault_info() if _bridge != null else {}
	var bound := bool(info.get("bound", false))
	var root := String(info.get("root", ""))
	var display := String(info.get("display_name", ""))
	var link_count := int(info.get("link_count", 0))
	_col_header(col, "MARKDOWN VAULT · %s" % ("LINKED" if bound else "NOT LINKED"), VAULT_NOTE)
	## The canvas borders this block in accent (`rgba(224,163,74,.35)`) because
	## its mockup vault *is* linked. The border now reads the same source the
	## header does, so the shape can never claim a link the engine has not got.
	var block := _block(col, "accent_wash_2" if bound else "line")
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 9)
	head.add_child(DccTheme.mono_label(
		DccIcons.SYMBOLS["on"] if bound else DccIcons.SYMBOLS["off"],
		"accent" if bound else "text_ghost", DccTheme.FS_TINY))
	## Three states, not two. `vault_info()` distinguishes "connected here" from
	## "this project knows a vault this device has not got" (a non-empty
	## `display_name` with `bound == false`) -- §27's Unbound case, and the one a
	## user opening someone else's project actually lands in.
	var path_text := "no vault linked"
	if bound:
		path_text = root if root != "" else display
	elif display != "":
		path_text = "%s — known to this project, not connected on this device" % display
	var path := DccTheme.mono_label(path_text, "text" if bound else "text_dim", DccTheme.FS_TINY)
	path.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## Clipped, so a long root cannot push the count off the block -- with the
	## full path in the tooltip, and `MOUSE_FILTER_STOP` so that tooltip can
	## actually be reached (a Label ignores the mouse by default, which is how a
	## tooltip ends up set and unreadable). Only when there IS a path to show.
	path.clip_text = true
	if root != "":
		path.tooltip_text = root
		path.mouse_filter = Control.MOUSE_FILTER_STOP
	head.add_child(path)
	var count_text := "0 notes"
	if bound:
		var stats: Dictionary = _bridge.vault_backlink_stats()
		count_text = ("%d note(s) · %d link(s)" % [int(stats.get("notes", 0)), link_count]) \
			if bool(stats.get("built", false)) else ("%d link(s) · index not built" % link_count)
	head.add_child(DccTheme.mono_label(count_text, "text_dim" if bound else "text_ghost", DccTheme.FS_TINY))
	block.add_child(head)

	var prose := DccTheme.label(
		("Settlements, provinces and continents link to notes in this folder; the panel that owns those links is Data ▸ Markdown vault. What is missing HERE is the export half -- exported tiles carry no obsidian:// link and exported GeoJSON carries no note property (DM-14)."
			if bound else
			"Connect any folder of .md files -- Obsidian is one such folder, and nothing requires it -- and settlements, provinces and continents can be linked to notes in it. Cartalith reads only the folder you choose, and writes only where you tell it to."),
		"text_ghost", DccTheme.FS_MICRO)
	prose.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	prose.custom_minimum_size.x = 200
	block.add_child(prose)

	## Still the three unbuilt items, and only those three. Verified this pass:
	## `cartalith-vault` generates no `obsidian://` URL by design (its own module
	## doc), `cartalith-engine::geojson` writes no note property, and §33 makes
	## two-way sync a V1 non-goal. Drawn disabled with THAT reason rather than the
	## old "there is no vault" one, which the block above now visibly contradicts.
	_check(block, "Two-way sync (write place notes back)", false, func(): pass,
		"V1 non-goal", false, VAULT_EXPORT_NOTE)
	_check(block, "Link labels to notes in GeoJSON", false, func(): pass, "", false, VAULT_EXPORT_NOTE)
	_check(block, "Include front-matter as properties", false, func(): pass, "", false, VAULT_EXPORT_NOTE)

	var btns := HBoxContainer.new()
	btns.add_theme_constant_override("separation", 6)
	block.add_child(btns)

	## Re-scan is two calls, not one: `vault_rebuild_backlinks()` only throws the
	## index away -- `vault_refresh_backlinks()` is what re-reads, and its own
	## return says how far it got.
	##
	## The outcome goes to the app status line, not to `_status_mid`: this window
	## rebuilds its own pane afterwards to pick up the new count, and
	## `_refresh_status()` rewrites `_status_mid` with the marquee line on the way
	## through -- so anything written there would be gone before it was read.
	## `_run_export()` reports through `_host.set_status()` for the same reason.
	##
	## **Saved, since 2026-09-24.** A successful re-scan writes the index to
	## `VaultStore.INDEX_PATH`, exactly as Data ▸ Markdown vault's own Refresh
	## and Rebuild do (`vault_window.gd::_refresh_index`). Before, the rebuilt
	## index lived only in memory and the next launch restored the stale one.
	var rescan := DccWidgets.chip(btns, "Re-scan vault", func():
		_bridge.vault_rebuild_backlinks()
		var r: Dictionary = _bridge.vault_refresh_backlinks()
		if bool(r.get("ok", false)):
			VaultStore.save_index_from(_bridge)
		if _host != null:
			_host.set_status("hint",
				("vault re-scanned — %d note(s)" % int(r.get("notes", 0)))
					if bool(r.get("ok", false))
					else "vault re-scan failed — %s" % String(r.get("error", "unknown")),
				"accent" if bool(r.get("ok", false)) else "warn")
		_rebuild_tile_export(), false, 0, 6)
	rescan.disabled = not bound
	rescan.tooltip_text = ("Throws the backlink index away and reads the folder again (vault_rebuild_backlinks, then vault_refresh_backlinks)."
		if bound else "No vault is connected. Use Change folder… first.")

	## Change folder routes to the panel that owns connecting rather than opening
	## a second folder browser here: `vault_window.gd` is where a chosen path
	## meets `vault_connect()`, and two connect paths is how the two ends of a
	## binding start disagreeing.
	var change := DccWidgets.chip(btns, "Change folder…", func():
		hide()
		_host.open_vault_overview(), false, 0, 6)
	change.disabled = _host == null
	change.tooltip_text = ("Opens Data ▸ Markdown vault, which owns the folder picker and the connection itself."
		if _host != null else "This window has no shell host to open the vault panel from.")

	## **Persisted, since 2026-09-24**, through the same path Connect uses:
	## `vault_window.gd` emits `store_changed` after `vault_connect`, and
	## `app.gd`'s handler is the only writer of `VaultStore.PATH` (and marks the
	## project dirty). This chip used to call `vault_disconnect` and stop, so the
	## sidecar kept the old binding and the next launch silently re-bound it.
	## Emitted after the engine call, never before.
	var unlink := DccWidgets.chip(btns, "Unlink", func():
		_bridge.vault_disconnect()
		_vault_store_changed()
		_rebuild_tile_export(), false, 0, 6)
	unlink.disabled = not bound
	unlink.tooltip_text = ("Drops this device's binding (vault_disconnect). The links themselves survive -- that is the difference between disconnecting and detaching."
		if bound else "Nothing is connected to unlink.")

	for b in [rescan, change, unlink]:
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.add_theme_font_size_override("font_size", DccTheme.FS_TINY)

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
	## The pyramid needs no marquee. With no world at all the binding returns
	## no bytes and `_run_export` reports that as a failed run.
	var ready: bool = _tx_pyramid() or not region.is_empty()
	var how := ("slippy_export_tiles (%s) -> one stored .zip, written with FileAccess." % TX_SCHEMES[_tx_scheme].to_upper()
		if _tx_pyramid() else "region_export_tiles -> zip_region_export, written with FileAccess.")
	_footer_note("writes to %s" % (_tx_dest if _tx_dest != "" else "—"))

	var preset := DccWidgets.chip(_pane_footer, "Save as preset", func(): pass, false, 14, 6)
	preset.disabled = true
	preset.tooltip_text = PRESET_NOTE

	var dry := DccWidgets.chip(_pane_footer, "Dry run", func(): _run_export(true), false, 14, 6)
	dry.disabled = not ready
	dry.tooltip_text = ("Runs the whole export and reports the tile count and archive size without writing a file."
		if ready else "No region marquee is set. Arm the Region select tool (R) and drag one on the map.")

	var go := DccWidgets.chip(_pane_footer, "Export %d tiles" % _tx_tile_count(),
		func(): _run_export(false), true, 16, 6)
	go.disabled = not ready
	go.tooltip_text = (how
		if ready else "No region marquee is set. Arm the Region select tool (R) and drag one on the map.")

## The pane is small enough that rebuilding it on a toggle is cheaper than
## threading a refresh through every control, and it keeps the estimate, the
## footer's tile count and the Format well in one consistent state.
## Hands a vault mutation made here to the one place that persists vault
## state: `VaultWindow.store_changed`, whose handler in `app.gd` writes
## `VaultStore.PATH` and marks the project dirty. Emitting that signal rather
## than calling `VaultStore.save_from` here keeps one writer, so this window
## cannot disagree with the vault panel about when or how the sidecar is saved.
func _vault_store_changed() -> void:
	if _host != null and _host.vault_window != null:
		_host.vault_window.store_changed.emit()

func _rebuild_tile_export() -> void:
	if _selected_id == "export_maps":
		_select_route("export_maps")

# ---------------------------------------------------------------------------
# Export ▸ Maps -- the run
# ---------------------------------------------------------------------------

## One of the **two** pickers here that write nothing: it only remembers
## `_tx_dest` for the footer's Export button. (It was the only one until
## 2026-09-05, when `_pick_geojson_destination()` became the second -- the
## pattern pane draws Export ▸ GIS's destination in a TO row, so that route now
## chooses first and writes second as well.) The overwrite guard still runs at
## pick time, which is exactly where the stock dialog put it -- and
## `_run_export()` has never had one of its own, so re-exporting to a
## destination already chosen still clobbers without asking. That is unchanged
## by this swap, and it is the reason the guard cannot simply move to the write.
func _pick_destination() -> void:
	DccBrowseDialog.choose_save_path(self, "Export tiles .zip", "zip",
		_tx_dest.get_base_dir() if _tx_dest != "" else DccSettings.storage_root("exports"),
		"one .zip of %d tiles; nothing is written until you press Export" % _tx_tile_count(),
		_tx_dest.get_file() if _tx_dest != "" else TX_DEFAULT_NAME,
		func(path: String): _overwrite_guard(path, func():
			_tx_dest = path
			_rebuild_tile_export()
			_refresh_foot()))

# ---------------------------------------------------------------------------
# Export ▸ GIS / GeoJSON -- the run (DM-03)
#
# One picker, one write. There is no options pane because the binding has no
# options: `export_geojson` describes the whole world, and every layer it can
# emit it always emits.
# ---------------------------------------------------------------------------

## **Chooses, it no longer writes.** `DataPane.dc.html` puts a destination row
## with a `Browse…` button ahead of the action row, so this picker records
## `_gis_dest` and the footer's Export writes there -- the shape
## `_pick_destination()` has always had for Export ▸ Maps. The overwrite guard
## stays here, at pick time, for the same reason it does there: `_run_geojson_
## export()` is reachable a second time from the footer with the destination
## already set, and a guard on the write would prompt every time.
func _pick_geojson_destination() -> void:
	## The reference names its own download `world_{seed}.geojson`; the shell
	## has no seed of its own to interpolate, and the document carries it as a
	## property anyway.
	DccBrowseDialog.choose_save_path(self, "Export GeoJSON", "geojson",
		_gis_dest.get_base_dir() if _gis_dest != "" else DccSettings.storage_root("exports"),
		"one FeatureCollection describing the whole world; nothing is written until you press Export",
		_gis_dest.get_file() if _gis_dest != "" else GIS_DEFAULT_NAME,
		func(path: String): _overwrite_guard(path, func():
			_gis_dest = path
			_rebuild_gis()
			_refresh_foot()))

## The footer's Export. Picks a destination first if none is set, exactly as
## `_run_export(false)` does for Export ▸ Maps.
func _run_geojson_export_here() -> void:
	if _gis_dest == "":
		_pick_geojson_destination()
		return
	_run_geojson_export(_gis_dest)

func _run_geojson_export(path: String) -> void:
	if _bridge == null:
		return
	_status_left.text = "exporting…"
	_status_left.add_theme_color_override("font_color", DccTheme.c("accent"))

	var t0 := Time.get_ticks_msec()
	var text := _bridge.export_geojson()
	var secs := float(Time.get_ticks_msec() - t0) / 1000.0

	if text.is_empty():
		_gis_doc_layers = {}
		_record_geojson_run(0, secs, false, path, {})
		_host.set_status("hint",
			"export failed — no world is loaded, or this build's GDExtension predates export_geojson", "warn")
	else:
		var f := FileAccess.open(path, FileAccess.WRITE)
		if f == null:
			_gis_doc_layers = {}
			_record_geojson_run(text.length(), secs, false, path, {})
			_host.set_status("hint",
				"export failed — could not open %s for writing" % path.get_file(), "warn")
		else:
			f.store_string(text)
			f.close()
			_gis_doc_layers = _measure_geojson(text)
			_record_geojson_run(text.to_utf8_buffer().size(), secs, true, path,
				_gis_doc_layers)
			_host.set_status("hint", "exported %s (%s)"
				% [path.get_file(), _fmt_bytes(text.to_utf8_buffer().size())], "accent")
	_rebuild_gis()
	_refresh_foot()
	_refresh_status()

func _rebuild_gis() -> void:
	if _selected_id == "export_gis":
		_select_route("export_gis")

# ---------------------------------------------------------------------------
# Import ▸ GIS / GeoJSON -- the run (Ruling V, LARGE_ITEM_RULINGS.md, 2026-09-21)
#
# One picker, one read, one apply -- `apply_geojson_document`
# (`engine_bridge.gd` -> `geojson_bridge.rs` -> `geojson_apply.rs`). Unlike
# Export ▸ GIS this route keeps no receipt in the pane:
# `_receipt_absent_reason`'s own "hands the file to the shell's own importer"
# rule for every `import_` route already covers it -- the result lands on the
# app status line and in the world itself, not in a receipt block here.
# ---------------------------------------------------------------------------

func _pick_geojson_import() -> void:
	DccBrowseDialog.choose_file(self, "Import GeoJSON",
		PackedStringArray(["geojson", "json"]),
		DccSettings.storage_root("exports"),
		"reads a FeatureCollection and places what it can into the current world",
		func(path: String): _run_geojson_import(path))

func _run_geojson_import(path: String) -> void:
	if _bridge == null or not _bridge.has_world:
		return
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		_host.set_status("hint",
			"import failed — could not open %s for reading" % path.get_file(), "warn")
		return
	var text := f.get_as_text()
	f.close()

	_status_left.text = "importing…"
	_status_left.add_theme_color_override("font_color", DccTheme.c("accent"))

	var result: Dictionary = _bridge.apply_geojson_document(text)
	if not bool(result.get("ok", false)):
		_host.set_status("hint",
			"import failed — %s" % String(result.get("error", "unknown error")), "warn")
	else:
		_host.set_status("hint", _geojson_import_summary(result), "accent")
	_refresh_status()

## The status-line summary for a finished import: every count
## `apply_geojson_document` reported, in one line. Every clause is
## conditional on its own count being nonzero -- `MISTAKES.md`'s "never
## encode no value as a plausible value" applied to a status line rather
## than a data field: an import that placed nothing says so rather than
## printing "0 settlements placed".
func _geojson_import_summary(result: Dictionary) -> String:
	var parts := PackedStringArray()

	var placed := int(result.get("settlements_placed", 0))
	var skipped := int(result.get("settlements_skipped", 0))
	if placed > 0 or skipped > 0:
		var s := "%d settlement%s placed" % [placed, "" if placed == 1 else "s"]
		if skipped > 0:
			s += " (%d skipped)" % skipped
		parts.append(s)

	var terr := int(result.get("territory_features_applied", 0))
	if terr > 0:
		parts.append("%d territory polygon%s -> %d cells" % [terr, "" if terr == 1 else "s",
			int(result.get("territory_cells_painted", 0))])
	## Omitted by the engine when zero. The usual cause is a polygon naming no
	## faction: imported territory is Territory paint, which cannot force a
	## cell unclaimed (`geojson_apply.rs`'s module doc).
	var terr_skipped := int(result.get("territory_features_skipped", 0))
	if terr_skipped > 0:
		parts.append("%d territory polygon%s skipped" % [terr_skipped, "" if terr_skipped == 1 else "s"])

	var created := PackedStringArray(result.get("factions_created", PackedStringArray()))
	if not created.is_empty():
		parts.append("%d new faction%s: %s" % [created.size(), "" if created.size() == 1 else "s",
			", ".join(created)])

	var unsupported: Dictionary = result.get("unsupported_layers", {})
	if not unsupported.is_empty():
		var uparts := PackedStringArray()
		for k in unsupported:
			uparts.append("%d %s" % [int(unsupported[k]), String(k) if String(k) != "" else "unlabelled"])
		parts.append("not placed: %s" % ", ".join(uparts))

	if bool(result.get("crs_unstated", false)):
		parts.append("coordinates assumed planar km -- the document did not confirm this")

	if parts.is_empty():
		var n := int(result.get("features", 0))
		return "imported %d feature%s, nothing placed" % [n, "" if n == 1 else "s"]
	return "imported: " + " · ".join(parts)

## The written document's own feature counts, keyed by `properties.layer`, plus
## `"features"` for the whole collection.
##
## Parsed back out of the text this window just wrote, so the receipt reports
## what the file holds rather than what the chips predicted. (Until 2026-09-05
## this said the shell had no other way to know how many rivers went in.
## `EngineBridge.rivers(2)` is that other way now -- but it is a second
## opinion, not a replacement: the two agreeing is what makes printing this
## worth the pass.) A run is already synchronous seconds of
## work, so one more pass over the string it produced is not what makes this
## route slow.
##
## `{}` when the text does not parse as a `FeatureCollection`, which is an
## absence and not a document of zero features: `has()`, never a zero.
func _measure_geojson(text: String) -> Dictionary:
	var doc = JSON.parse_string(text)
	if typeof(doc) != TYPE_DICTIONARY or not (doc as Dictionary).has("features"):
		return {}
	var feats: Array = (doc as Dictionary)["features"]
	var out: Dictionary = {"features": feats.size()}
	for fe in feats:
		if typeof(fe) != TYPE_DICTIONARY:
			continue
		var props: Dictionary = (fe as Dictionary).get("properties", {})
		if not props.has("layer"):
			continue
		var k := String(props["layer"])
		out[k] = int(out.get(k, 0)) + 1
	return out

## The receipt's second line: **what did not go into the file.**
##
## Three separate reasons, and each is measured rather than modelled:
##
##   1. the groups no GeoJSON layer carries (`GIS_GROUPS`' own `carried` flag);
##   2. the carried groups the written document turned out to hold none of --
##      read off `layers`, which came from the document itself;
##   3. hand-drawn ways, which `export_geojson` does not read (it takes
##      `civ.ways` and `civ.sea_routes`, never `infra.ways`) while
##      `get_roads()`/`get_sea_routes()` do. A user who drew a road and then
##      exported would otherwise never learn it was dropped.
func _gis_omitted_line(layers: Dictionary) -> String:
	var parts := PackedStringArray()

	var not_carried := PackedStringArray()
	for g in GIS_GROUPS:
		if not bool(g["carried"]):
			not_carried.append(String(g["key"]))
	if not_carried.size() > 0:
		parts.append("%s not included — no GeoJSON layer carries them"
			% _and_list(not_carried))

	## The document's own layer names, per group. Only the carried groups have
	## one, which is what makes this list and `not_carried` disjoint.
	const GROUP_LAYER := {"settlements": "settlement", "factions": "territory",
		"ways": "way", "rivers": "river", "provinces": "province"}
	var empty := PackedStringArray()
	if not layers.is_empty():
		for key in GROUP_LAYER:
			if not layers.has(String(GROUP_LAYER[key])):
				empty.append(String(key))
	if empty.size() > 0:
		parts.append("no %s in this world" % _and_list(empty))

	var manual := _gis_manual_way_count()
	if manual > 0:
		parts.append("%d hand-drawn way%s dropped — export_geojson reads civ.ways and civ.sea_routes, not infra.ways"
			% [manual, "" if manual == 1 else "s"])

	return " · ".join(parts)

## Hand-drawn ways and sea lanes (`infra.ways`, flagged `manual` by
## `get_roads()`/`get_sea_routes()`). Zero is a real answer here -- "no way was
## hand-drawn" -- and is not standing in for an absence, so it is an `int`.
func _gis_manual_way_count() -> int:
	if _bridge == null or not _bridge.has_world:
		return 0
	var n := 0
	for w in _bridge.roads():
		if bool((w as Dictionary).get("manual", false)):
			n += 1
	for w in _bridge.sea_routes():
		if bool((w as Dictionary).get("manual", false)):
			n += 1
	return n

## `layers` is `_measure_geojson()`'s return: `{}` for a failed run, so the
## `features` count is **omitted** rather than written as 0 and the receipt
## reports the failure instead of "wrote 0 features".
func _record_geojson_run(bytes: int, secs: float, ok: bool, path: String,
		layers: Dictionary) -> void:
	var t := Time.get_time_dict_from_system()
	var run := {
		"stamp": "%02d:%02d" % [int(t["hour"]), int(t["minute"])],
		"msec": Time.get_ticks_msec(),
		"route": "export_gis", "path": path,
		"label": "geojson", "bytes": bytes, "secs": secs, "ok": ok,
	}
	if ok and layers.has("features"):
		run["receipt"] = "wrote %d features · %s" % [int(layers["features"]),
			_fmt_bytes(bytes)]
		run["omitted"] = _gis_omitted_line(layers)
	elif ok:
		## Written, but the text did not parse back -- so the file exists and
		## its feature count is unknown. Said, not guessed.
		run["receipt"] = "wrote %s · feature count unreadable" % _fmt_bytes(bytes)
	else:
		run["receipt"] = "export failed after %.1f s" % secs
	_runs.push_front(run)
	while _runs.size() > 3:
		_runs.pop_back()

## `dry` performs the identical export and reports what it produced without
## writing it -- which is how the ESTIMATE block gets a real size and time
## instead of a modelled one.
func _run_export(dry: bool) -> void:
	if _bridge == null:
		return
	if not _tx_pyramid() and _bridge.region_get().is_empty():
		_host.set_status("hint", "export failed — no region marquee is set", "warn")
		return
	if not dry and _tx_dest == "":
		_pick_destination()
		return

	_status_left.text = "exporting…"
	_status_left.add_theme_color_override("font_color", DccTheme.c("accent"))

	var t0 := Time.get_ticks_msec()
	var binding := "slippy_export_tiles" if _tx_pyramid() else "region_export_tiles"
	var bytes: PackedByteArray
	if _tx_pyramid():
		bytes = _bridge.slippy_export_tiles({
			"scheme": TX_SCHEMES[_tx_scheme], "max_z": _tx_zmax, "tile_size": _tx_tile,
			"retina": _tx_retina,
		})
	else:
		bytes = _bridge.region_export_tiles({
			"cols": _tx_cols, "rows": _tx_rows, "tile_size": _tx_tile,
			"gzip": _tx_gzip, "ridged": _tx_ridged, "visual": _tx_visual,
		})
	var secs := float(Time.get_ticks_msec() - t0) / 1000.0

	if bytes.is_empty():
		_record_run(dry, 0, secs, false)
		_host.set_status("hint",
			"export failed — %s returned no bytes (see the Godot log)" % binding, "warn")
		_rebuild_tile_export()
		_refresh_foot()
		_refresh_status()
		return

	if dry:
		_record_run(true, bytes.size(), secs, true)
		_host.set_status("hint", "dry run — %d tiles, %s, %.1f s (nothing written)"
			% [_tx_tile_count(), _fmt_bytes(bytes.size()), secs], "accent")
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
		"msec": Time.get_ticks_msec(), "route": "export_maps",
		"label": ("%s %s z0–%d %dpx%s" % ["dry run" if dry else "pyramid",
			TX_SCHEMES[_tx_scheme].to_upper(), _tx_zmax, _tx_tile, " @2x" if _tx_retina else ""]
			if _tx_pyramid() else
			"%s %d×%d z%d%s" % ["dry run" if dry else "tile grid",
			_tx_cols, _tx_rows, _tx_tile, "" if _tx_visual else " (height only)"]),
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
	if _selected_id == "val_defs":
		## DM-10. `_checks_last` is empty only before that route has ever been
		## built; a healthy scan puts a real `0` there, so the two are told
		## apart by `is_empty()` rather than by the count.
		if _checks_last.is_empty():
			_status_mid.text = ""
			return
		var found: int = (_checks_last["rows"] as Array).size()
		_status_mid.text = ("%d finding%s across %d definitions and 1 asset library"
			% [found, "" if found == 1 else "s", int(_checks_last["defs"])])
		_status_mid.add_theme_color_override("font_color",
			DccTheme.c("text_faint") if found == 0 else DccTheme.c("warn"))
		return
	if _selected_id != "export_maps":
		_status_mid.text = ""
		return
	if _tx_pyramid():
		_status_mid.text = "whole world · %s pyramid z0–%d · %d tiles queued" % [
			TX_SCHEMES[_tx_scheme].to_upper(), _tx_zmax, _tx_tile_count()]
		_status_mid.add_theme_color_override("font_color", DccTheme.c("text_faint"))
		return
	var region := _bridge.region_get() if _bridge != null else {}
	_status_mid.text = ("no region marquee — arm Region select (R) and drag one"
		if region.is_empty()
		else "marquee %d × %d cells · %d tiles queued" % [
			int(region.get("w", 0)), int(region.get("h", 0)), _tx_cols * _tx_rows])
	_status_mid.add_theme_color_override("font_color",
		DccTheme.c("accent") if region.is_empty() else DccTheme.c("text_faint"))

## The reference's `#lodShowGrid` preview ("Show tile borders on the map", line
## 1281) draws `refCols` x `refRows` over the map -- the *export* split, not the
## LOD pyramid. Those two numbers are `_tx_cols`/`_tx_rows` here, and this is
## the only place they change, so the viewport is told from here rather than
## keeping a second pair that could disagree with what Export actually writes.
##
## Called on every grid change, not only when the preview is showing: the
## overlay caches the split and redraws itself, so turning the preview on later
## finds the current numbers already there instead of the 4x4 default.
func _push_tile_grid_preview() -> void:
	if _host == null or _host.viewport == null:
		return
	if not _host.viewport.has_method("set_export_tile_grid"):
		return
	_host.viewport.set_export_tile_grid(
		_host.viewport.export_tile_grid_enabled(), _tx_cols, _tx_rows)

## No `tile_grid()` accessor here, deliberately. One existed for "the menu row
## that toggles its preview", and that row never called it: `menus.gd`'s
## `ID_LOD_TILE_BORDERS` handler calls
## `viewport.set_export_tile_grid(not viewport.export_tile_grid_enabled())`
## with `cols`/`rows` left at their `-1` default, which that setter reads as
## "keep what you were told last". What told it is `_push_tile_grid_preview()`
## above, on every grid change -- so the numbers are already there and a
## getter would only offer a second way to disagree with them.
