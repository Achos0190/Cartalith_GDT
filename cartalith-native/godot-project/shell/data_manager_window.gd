extends AcceptDialog
class_name DataManagerWindow

## §9's Data manager window -- Data ▸ Import/Export/Sources/
## Validation's actual destination. `world_data_window.gd`'s own doc comment
## draws the line this file is the other side of: that window is the
## settlement/province/economy table browser (`Data ▸ World data tables…`,
## §9's own related-but-distinct sibling), unrelated to and untouched by this
## file.
##
## Titled `⧉ DATA MANAGER`, subtitle was "import · export · sources ·
## conversion · validation" (§9, verbatim) until the Conversion group itself
## was deleted (this file's own `GROUP_ORDER` doc comment below) -- the
## subtitle line missed that pass (caught by the 2026-08-20 visual sweep,
## `GUI_GAP_REGISTER.md`) and still advertised a fifth area the routes rail
## no longer has. Now four, matching `GROUP_ORDER`. Structure per §9: a
## routes rail (`GROUP_ORDER`'s groups) and a route pane showing the
## selected route's real controls.
##
## **What is real vs. disclosed gap, route by route** -- most of §9 has no
## engine behind it, and this file says so per-route rather than building
## chrome that implies a capability that doesn't exist:
##
## - **Import ▸ World Data (.zip · fields)** is real: it routes to the exact
##   same `bridge.load_save(path)` / `DccApp.open_project_picker()` path File
##   ▸ Open project… already uses, not a second implementation.
## - **Import ▸ Assets** is real as a routing shortcut: it calls
##   `DccApp.open_asset_pack_picker()` directly, per §2.4's own table
##   ("Assets (routes to the Assets menu)").
## - **Export ▸ World Data** stays a disclosed gap: `cartalith-io` reads
##   `.zip` saves (`load_save`) but the only `zip::ZipWriter` in the crate is
##   inside its own `#[cfg(test)]` fixture builder
##   (`cartalith-io/src/lib.rs::tests::build_test_zip`) -- there is no
##   production save writer. Confirmed by reading the crate directly this
##   pass, not assumed from an old comment.
## - **Export ▸ Assets** is real as a routing shortcut too (DM-05, since
##   2026-08-20): it calls the Asset library window's own real
##   `export_pack_now()` (AS-04, `as_export_pack_bytes` ->
##   `archive::write_pack`), the same "routes, doesn't reimplement" shape
##   `import_assets` above already has.
## - **Import ▸ Heightmaps (PNG)** is real (DM-01, since 2026-08-20): it
##   routes to `DccApp.open_heightmap_import()` ->
##   `EngineBridge.import_heightmap` -> `WorldGen::import_heightmap`, which
##   decodes the PNG, takes it as the elevation field and runs
##   `cartalith_engine::import::infer_tectonics` under it -- the reference's
##   own `#loadBtn` + `#inferTectBtn` pair, ported and golden-tested
##   (`cartalith-terrain/tests/golden_parity_infer.rs`). TIFF is absent and
##   that is parity, not a shortfall: the reference's file input is
##   `accept="image/*"` decoded by the browser, which does not read TIFF
##   either.
## - **Import ▸ Maps/GIS**, **Export ▸ Maps/GIS**, **Sources** and
##   **Validation** are all disclosed gaps: no tile-map or GeoJSON *import*,
##   no tile/GIS export, no source registry and no validation pass exist
##   anywhere in the workspace (`load_save` returns a plain bool, nothing a
##   warning count could be read from).
## - **Conversion is gone, not disclosed.** See `GROUP_ORDER` below.

var _host: DccApp
var _bridge: EngineBridge

## `[{group, id, label, kind, reason}]`. `kind` is "live" (real control),
## "route" (a real shortcut into another menu) or "gap" (disclosed, no
## engine support -- `reason` is shown verbatim, mirroring `menus.gd`'s own
## `_todo()` tooltip convention for a window rather than a popup item).
const ROUTES: Array[Dictionary] = [
	{"group": "Import", "id": "import_heightmap", "label": "Heightmaps (PNG)", "kind": "live"},
	{"group": "Import", "id": "import_maps", "label": "Maps (tiles) · GIS / GeoJSON", "kind": "gap",
		"reason": "No tile-map or GeoJSON *import* path exists (cartalith-engine::geojson only writes region GeoJSON, and nothing reads one back). TIFF is also absent, and deliberately: the reference's own file input is accept=\"image/*\" and decodes through the browser, which does not decode TIFF either -- so PNG is parity, not a shortfall. Heightmap import itself is live now; see the row above."},
	{"group": "Import", "id": "import_world", "label": "World Data (.zip · fields)", "kind": "live"},
	{"group": "Import", "id": "import_assets", "label": "Assets (routes to the Assets menu)", "kind": "route"},
	{"group": "Export", "id": "export_maps", "label": "Maps (image · tiles)", "kind": "gap",
		"reason": "No tile-pyramid or image export exists. cartalith-terrain::tile_render draws per-tile PNGs for Region select/export (unified tool plan milestone E2), but nothing assembles a Leaflet-style pyramid from it."},
	{"group": "Export", "id": "export_gis", "label": "GIS / GeoJSON", "kind": "gap",
		"reason": "cartalith-engine::geojson exports region GeoJSON for the Region-select tool only, with no route into this window and no CRS/world-file support."},
	{"group": "Export", "id": "export_world", "label": "World Data", "kind": "gap",
		"reason": "cartalith-io reads .zip saves but does not write them -- the only zip::ZipWriter in the crate lives in its own #[cfg(test)] fixture builder, not production code. A save writer is a separate, larger piece of work, out of scope here."},
	{"group": "Export", "id": "export_assets", "label": "Assets (pack .zip)", "kind": "route"},
	{"group": "Sources", "id": "sources_external", "label": "External Sources", "kind": "gap", "reason": "No source registry exists."},
	{"group": "Sources", "id": "sources_connected", "label": "Connected Sources", "kind": "gap", "reason": "Same -- no source registry exists."},
	{"group": "Sources", "id": "sources_registry", "label": "Source Registry", "kind": "gap", "reason": "Same -- no source registry exists."},
	{"group": "Validation", "id": "val_check", "label": "Check Data", "kind": "gap",
		"reason": "load_save() returns pass/fail only (cartalith-godot's load_save binding) -- no warning collection exists anywhere to surface a count from."},
	{"group": "Validation", "id": "val_repair", "label": "Repair / Normalize", "kind": "gap", "reason": "No validation pass exists to repair against."},
]

## **Four groups, not five.** Conversion (Coordinate Systems (EPSG) / Format
## Conversion / Data Transformation) was deleted 2026-08-20 on the owner's
## decision: `GUI_GAP_REGISTER.md` §7.4's research found no serious GIS or
## mapping application carries a top-level Conversion route, because
## reprojection and format handling belong to the import/export step that is
## actually reading or writing the file. The three rows were disclosed gaps
## with no engine work behind them and none planned, so they were removed
## rather than kept as a standing promise. `DCC_SHELL_SPEC.md` §2.4 carries
## the correction note; `GUI_GAP_REGISTER.md` DM-07/08/09 are resolved by
## deletion.
const GROUP_ORDER: Array[String] = ["Import", "Export", "Sources", "Validation"]

var _pane_body: VBoxContainer
var _breadcrumb: Label
var _rail_buttons: Dictionary = {}   ## route id -> Button
var _selected_id := ""

func setup(host: DccApp, bridge: EngineBridge) -> void:
	_host = host
	_bridge = bridge
	title = "⧉ DATA MANAGER"
	size = Vector2i(920, 600)
	min_size = Vector2i(760, 480)
	_build()

func _build() -> void:
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 6)
	add_child(outer)

	outer.add_child(DccTheme.mono_label("import · export · sources · validation",
		"text_faint", DccTheme.FS_MICRO, 2))
	outer.add_child(DccTheme.rule())

	var main := HBoxContainer.new()
	main.add_theme_constant_override("separation", 0)
	main.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(main)

	main.add_child(_build_rail())
	main.add_child(DccTheme.rule(true))
	main.add_child(_build_pane())

func _build_rail() -> Control:
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

	var by_group: Dictionary = {}
	for r in ROUTES:
		var g := String(r["group"])
		if not by_group.has(g):
			by_group[g] = []
		(by_group[g] as Array).append(r)

	for g in GROUP_ORDER:
		var body := DccWidgets.section(col, g)
		for r in by_group.get(g, []):
			var route: Dictionary = r
			var btn := Button.new()
			btn.text = String(route["label"])
			btn.flat = true
			btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
			btn.focus_mode = Control.FOCUS_NONE
			btn.custom_minimum_size.y = 26
			btn.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
			btn.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
			btn.add_theme_color_override("font_color",
				DccTheme.c("text_dim") if String(route["kind"]) == "gap" else DccTheme.c("text"))
			btn.pressed.connect(_select_route.bind(route["id"]))
			_rail_buttons[route["id"]] = btn
			body.add_child(btn)

	var foot := VBoxContainer.new()
	foot.add_theme_constant_override("separation", 2)
	var foot_pad := MarginContainer.new()
	foot_pad.add_theme_constant_override("margin_left", 14)
	foot_pad.add_theme_constant_override("margin_top", 10)
	foot_pad.add_theme_constant_override("margin_bottom", 10)
	foot_pad.add_theme_constant_override("margin_right", 10)
	foot_pad.add_child(foot)
	col.add_child(DccTheme.rule())
	col.add_child(foot_pad)
	foot.add_child(DccTheme.mono_label("EXPORTS ROOT", "text_faint", DccTheme.FS_HEADER, 2, true))
	var exports_label := DccTheme.label(DccSettings.storage_root("exports"), "text_dim", DccTheme.FS_TINY)
	exports_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	foot.add_child(exports_label)
	## §9: "Foot: exports root and last run (`14:02 · 62 MB`)." No export
	## capability exists yet (every Export route above is a disclosed gap),
	## so there is genuinely no run to report -- said plainly rather than
	## inventing a placeholder timestamp.
	foot.add_child(DccTheme.label("no export has run yet", "text_ghost", DccTheme.FS_TINY))

	return wrap

func _build_pane() -> Control:
	var wrap := VBoxContainer.new()
	wrap.add_theme_constant_override("separation", 8)
	wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 16)
	head_pad.add_theme_constant_override("margin_top", 12)
	head_pad.add_theme_constant_override("margin_right", 16)
	_breadcrumb = DccTheme.mono_label("", "accent", DccTheme.FS_HEADER, 2, true)
	head_pad.add_child(_breadcrumb)
	wrap.add_child(head_pad)
	wrap.add_child(DccTheme.rule())

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	wrap.add_child(scroll)

	var body_pad := MarginContainer.new()
	body_pad.add_theme_constant_override("margin_left", 16)
	body_pad.add_theme_constant_override("margin_top", 10)
	body_pad.add_theme_constant_override("margin_right", 16)
	body_pad.add_theme_constant_override("margin_bottom", 10)
	scroll.add_child(body_pad)

	_pane_body = VBoxContainer.new()
	_pane_body.add_theme_constant_override("separation", 8)
	_pane_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body_pad.add_child(_pane_body)

	return wrap

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
	for rid in _rail_buttons:
		var b: Button = _rail_buttons[rid]
		b.add_theme_color_override("font_color",
			DccTheme.c("accent") if rid == id else
			(DccTheme.c("text_dim") if String(_route_by_id(rid).get("kind", "gap")) == "gap" else DccTheme.c("text")))

	_breadcrumb.text = "%s ▸ %s" % [String(route["group"]).to_upper(), String(route["label"]).to_upper()]

	for c in _pane_body.get_children():
		_pane_body.remove_child(c)
		c.queue_free()

	match String(route.get("kind", "gap")):
		"live":
			match id:
				"import_heightmap":
					if _bridge != null and _bridge.import_api:
						DccWidgets.note(_pane_body,
							"Reads a PNG heightmap (white = high), resamples it to the working grid at the image's own aspect ratio, and infers a tectonic substrate from its morphology so lithology, resources and settlement have something to read -- the reference's Import ▸ Load heightmap… followed by Infer tectonics from heightmap. Scale (width, peak) comes from New world…, exactly as the reference's own calibrate step reuses its generate form.")
						DccWidgets.action(_pane_body, "Import heightmap…", func():
							hide()
							_host.open_heightmap_import())
					else:
						DccWidgets.note(_pane_body,
							"This build's GDExtension predates the heightmap-import binding (WorldGen::import_heightmap). Rebuild cartalith-godot to enable it.")
				"import_world":
					DccWidgets.note(_pane_body,
						"Opens the same .zip project picker as File ▸ Open project… -- routed here per §9, not reimplemented.")
					DccWidgets.action(_pane_body, "Open project…", func():
						hide()
						_host.open_project_picker())
		"route":
			match id:
				"import_assets":
					DccWidgets.note(_pane_body,
						"Routes to Assets ▸ Import asset pack .zip… -- §2.4's own table calls this item a shortcut, not a second implementation.")
					DccWidgets.action(_pane_body, "Import asset pack .zip…", func():
						hide()
						_host.open_asset_pack_picker())
				"export_assets":
					## DM-05: routes to the Asset library window's own real
					## Export pack .zip… (AS-04, `as_export_pack_bytes` ->
					## `archive::write_pack`) -- §2.4's own table calls this a
					## shortcut, not a second implementation, same as
					## `import_assets` above.
					DccWidgets.note(_pane_body,
						"Routes to the Asset library window's own Export pack .zip… (Assets ▸ ⧉ Asset library, §8's window bar) -- real now (as_export_pack_bytes -> archive::write_pack).")
					DccWidgets.action(_pane_body, "Export pack .zip…", func():
						hide()
						_host.open_asset_library()
						_host.asset_library_window.export_pack_now())
		_:
			var reason := String(route.get("reason", "Not implemented."))
			DccWidgets.note(_pane_body, reason)

## `group`, if given, selects that group's first route; empty selects the
## very first route overall. Both `menus.gd`'s five Data-menu group items and
## a bare "open the window" caller go through this one entry point.
func open(group: String = "") -> void:
	popup_centered()
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
