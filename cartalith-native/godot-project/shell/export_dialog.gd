extends AcceptDialog
## **The single-image export dialog** (`EXPORT_SCOPE.md` §7, milestone E5):
## `Data ▸ Export ▸ Export map image…` (`menus.gd`, `ID_EXPORT_MAP_IMAGE`),
## opened by `DccApp.open_export_dialog()`. One flat PNG of the whole map --
## the owner's *"a user generated monolithic image of the map ... just to be
## used outside of Cartalith in an image viewer"* (2026-09-06, top of the
## scope) -- at a ladder width, with the overlays Ruling AZ lists, in the
## colour space Ruling CB allows, written by `ExportOverlay.run`
## (`shell/export_overlay.gd`, E4) with progress and cancel.
##
## What it must never do:
##   * call `WorldGen` while a generation holds it -- every engine read goes
##     through `_estimate()`, which refuses while `bridge.generating`, and the
##     export itself only through `ExportOverlay.run`'s gated
##     `EngineBridge.export_session_*` wrappers;
##   * offer a size the device cannot hold without saying why: a size whose
##     largest band would not fit in reported free memory is disabled with the
##     figures (`size_refusal`), never silently clamped to a smaller one;
##   * invent a number: a measured time is shown only for the two widths it was
##     measured at, and a field the engine omits (`file_bytes`, free memory) is
##     dashed with its reason, not shown as 0;
##   * leave an export running behind a closed window: hiding the dialog
##     cancels it (`ExportOverlay.cancel`, which removes the partial file).
##
## One script with inline phone/desktop branches, the shape `DccApp.
## open_storage_locations()` and the other hand-built windows use:
## `DccWidgets.phone_window` decides, a phone gets a titled header and a
## scrolling body, and `DccWidgets.phone_present` opens it full-screen.
##
## **Not in this first slice** (E5 remainder, `OUTSTANDING_WORK.md`): JPEG
## (Ruling AZ amended ruling 26 to offer it; the streaming writer has PNG and
## BigTIFF only, `export_options.rs::FORMAT_NAMES`), a style/look override (E3
## carries `style`; no control here yet), and a file-exists confirmation.

## Emitted when an export started from this dialog ends, with
## `ExportOverlay.run`'s dictionary (`ok`, `path`, ..., or `error` and
## `cancelled`). The probes wait on it.
signal export_finished(result: Dictionary)

const EXPORT_OVERLAY := preload("res://shell/export_overlay.gd")

## The widths offered when the engine cannot list its own (`export_raster_widths`,
## `export_raster.rs::BAKE_WIDTHS`) -- the same five rungs, copied so a dialog
## over an older binary still draws; the engine's list wins when present.
const FALLBACK_WIDTHS: Array[int] = [2048, 4096, 8192, 16384, 32768]

## The width the dialog selects first when it is affordable: 16K, the size the
## owner asked for first (*"a png at 16k or even 32k"*, 2026-08-25). Labelled
## judgement -- the request names both; 16K is the cheaper of the two.
const DEFAULT_WIDTH := 16384

## What `EXPORT_SCOPE.md` §5 "E4 batches C and D" measured for a whole export
## with overlays (seed 20260906, 2048 x 1311 grid, towns, ways, labels, icons,
## borders; one run each on the dev desktop, RX 7800 XT): wall time, and the
## file it wrote (the estimated band peak, 942 MB at both, is shown from
## the live estimate instead). **One run, one machine**: shown
## as "measured once on a desktop", never as a prediction for this device.
const MEASURED := {
	16384: "Measured once on a desktop: 17.4 s, 68.6 MB file.",
	32768: "Measured once on a desktop: 112.2 s, 170.1 MB file.",
}

## `export_options.rs::SettlementTier::ALL`'s names, largest first, with the
## words the dialog shows. `"all"` adds the additive villages.
const TIERS := [["metropolis", "Metropolises"], ["capital", "Capitals and up"],
	["city", "Cities and up"], ["town", "Towns and up"], ["village", "Villages and up"],
	["hamlet", "Hamlets and up"], ["all", "Everything, with the extra villages"]]
## Ruling AZ (2026-09-28): "the user picks a minimum settlement tier (default
## towns)". Must equal `export_options.rs::SettlementTier::DEFAULT`'s name.
const DEFAULT_TIER := "town"

## The overlay switches, `[content key, label, default, tooltip]`. Ruling AZ's
## v1 overlays (settlements, labels, roads and ways, icons, borders) default on,
## since the owner listed them as the export's content; `routes` (journey
## routes, not in AZ's list) defaults off. Labelled judgement for the defaults.
const LAYERS := [
	["rivers", "Rivers", true, "The vector river line, painted with the terrain."],
	["ways", "Roads and ways", true, "The road and way network."],
	["labels", "Labels", true, "Place names."],
	["icons", "Icons", true, "Landmark and settlement icons."],
	["territory", "Borders", true, "Faction territory and province lines."],
	["routes", "Journey routes", false, "Planned journey routes. Not in Ruling AZ's v1 list; off by default."],
]

var _app: Node = null
var _bridge: Node = null
var _phone := false
var _presented_once := false

var _sizes: OptionButton
var _size_note: Label
var _widths: Array[int] = []
## `export_image_estimate` per width from the last `refresh()`; `{}` where none.
var _estimates: Dictionary = {}
var _settle: CheckBox
var _tier: OptionButton
var _layer_boxes: Dictionary = {}
var _space: OptionButton
var _path_label: Label
var _path := ""
var _go: Button
var _stop: Button
var _bar: ProgressBar
var _status: Label
var _runner: Node = null
var _running := false
## The last `bridge.generating` this dialog drew, so `_process` redraws only on a change.
var _was_generating := false

## Builds the window's contents for `app` (a `DccApp`). Call once, before
## `add_child`; `present()` opens it.
func setup(app: Node) -> void:
	_app = app
	_bridge = app.bridge
	title = "Export map image"
	size = Vector2i(620, 600)
	_phone = DccWidgets.phone_window(self, app)
	## `AcceptDialog` lays out only its FIRST child (`open_storage_locations` says why).
	var host := VBoxContainer.new()
	host.add_theme_constant_override("separation", 0)
	add_child(host)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 8)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	if _phone:
		DccWidgets.phone_head(host, "Export map image", "one flat PNG of the whole map")
	host.add_child(scroll)
	scroll.add_child(body)

	body.add_child(_heading("SIZE"))
	_sizes = OptionButton.new()
	_sizes.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_sizes.item_selected.connect(func(_i: int): _redraw_state())
	body.add_child(_sizes)
	_size_note = _note("")
	body.add_child(_size_note)

	body.add_child(_heading("CONTENT"))
	var srow := HBoxContainer.new()
	srow.add_theme_constant_override("separation", 8)
	_settle = CheckBox.new()
	_settle.text = "Settlements"
	_settle.button_pressed = true
	_settle.tooltip_text = "Settlement pins and plans, down to the tier on the right (Ruling AZ: towns by default)."
	_settle.toggled.connect(func(on: bool): _tier.disabled = not on or _running)
	srow.add_child(_settle)
	_tier = OptionButton.new()
	_tier.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	for t in TIERS:
		_tier.add_item(String(t[1]))
	_tier.select(_tier_index(DEFAULT_TIER))
	srow.add_child(_tier)
	body.add_child(srow)
	## Two columns on a pointer, one on a phone (412 dp has no room for two).
	var grid := GridContainer.new()
	grid.columns = 1 if _phone else 2
	grid.add_theme_constant_override("h_separation", 16)
	for l in LAYERS:
		var cb := CheckBox.new()
		cb.text = String(l[1])
		cb.button_pressed = bool(l[2])
		cb.tooltip_text = String(l[3])
		grid.add_child(cb)
		_layer_boxes[String(l[0])] = cb
	body.add_child(grid)

	body.add_child(_heading("COLOUR SPACE"))
	_space = OptionButton.new()
	var spaces: Array = _bridge.export_color_spaces() if _bridge.has_method("export_color_spaces") else ["sRGB"]
	for s in spaces:
		_space.add_item(String(s))
	_space.select(0)
	body.add_child(_space)
	## Ruling CB (2026-10-11) item 1: sRGB is the default and the file is tagged
	## as such; Display P3 stays an explicit choice.
	body.add_child(_note("sRGB suits any screen or viewer. Display P3 keeps wider colour for P3 displays and print workflows; a viewer that ignores the tag shows it slightly dull."))

	body.add_child(_heading("DESTINATION"))
	var prow: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	prow.add_theme_constant_override("separation", 8)
	_path_label = DccTheme.mono_label("No file chosen", "text_dim", DccTheme.FS_SMALL)
	_path_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if _phone:
		_path_label.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
	else:
		_path_label.clip_text = true
	prow.add_child(_path_label)
	DccWidgets.action(prow, "Choose…", _choose_path)
	body.add_child(prow)

	body.add_child(DccTheme.rule())
	_bar = ProgressBar.new()
	_bar.min_value = 0.0
	_bar.max_value = 1.0
	_bar.step = 0.0
	_bar.show_percentage = false
	_bar.custom_minimum_size.y = 8
	body.add_child(_bar)
	_status = _note("")
	body.add_child(_status)
	var arow := HBoxContainer.new()
	arow.add_theme_constant_override("separation", 8)
	arow.alignment = BoxContainer.ALIGNMENT_END
	_stop = DccWidgets.action(arow, "Cancel export", cancel_export)
	_go = DccWidgets.action(arow, "Export", start_export, true)
	body.add_child(arow)

	## Hiding the window while an export runs cancels it (header: never leave
	## an export running behind a closed window).
	visibility_changed.connect(func():
		if not visible and _running and _runner != null:
			_runner.cancel())

## A small tracked section heading, the shell's §-header style.
func _heading(text: String) -> Label:
	return DccTheme.mono_label(text, "text_dim", DccTheme.FS_HEADER, 1)

## A wrapping note. No hard minimum width: on a phone that would push the
## window past the screen (`MISTAKES.md`, "Read a layout that overflows").
func _note(text: String) -> Label:
	var l := DccTheme.label(text, "text_ghost", DccTheme.FS_MICRO)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	return l

func _tier_index(tier_name: String) -> int:
	for i in TIERS.size():
		if TIERS[i][0] == tier_name:
			return i
	return 0

## Re-prices every size against the world as it is now and opens the window.
func present() -> void:
	refresh()
	if _phone and not _presented_once:
		## `1.0`: `phone_present` applies the scale as `content_scale_factor`
		## (`open_storage_locations` says why not `phone_scale()` here).
		_app.phone_fit(self, 1.0)
	_presented_once = true
	if not DccWidgets.phone_present(self, _app):
		popup_centered()

## The engine's estimate for an export at `width` with the dialog's content,
## or `{}` while a generation holds the engine, before a world exists, or on a
## binary without `export_image_estimate`.
func _estimate(width: int) -> Dictionary:
	if _bridge == null or bool(_bridge.generating) or not bool(_bridge.has_world):
		return {}
	if not _bridge.world_gen.has_method("export_image_estimate"):
		return {}
	return _bridge.world_gen.export_image_estimate({"width": width})

## Why an export priced by `est` (`export_image_estimate`) cannot run on this
## device, or `""`. The session renders band by band, so the bound is the
## largest band's peak (`band_peak_bytes`: the band's pixels at
## `export_raster.rs::PEAK_BYTES_PER_PIXEL` = 23 bytes, measured 2026-09-06)
## against the free memory the platform reports (`memory_available`). When the
## platform reports none, the key is absent and nothing is refused -- the
## engine's own gate still applies at `export_session_begin`.
static func size_refusal(est: Dictionary) -> String:
	if est.is_empty():
		return "Not priced: no world, or a generation is running."
	if not bool(est.get("ok", true)):
		return String(est.get("error", "the engine refused these options"))
	if est.has("band_affordable") and not bool(est["band_affordable"]):
		return "Needs about %.2f GB of working memory for its largest band; %.2f GB is free." % [
			float(est.get("band_peak_bytes", 0)) / 1e9, float(est.get("memory_available", 0)) / 1e9]
	return ""

## One size's line in the list: dimensions and the estimated file size, or a
## dash with why when the engine gave no estimate.
static func size_text(width: int, est: Dictionary) -> String:
	if est.is_empty() or not bool(est.get("ok", true)):
		return "%s px wide  --  not priced" % _thousands(width)
	var file := "file size --  (no model for this format)"
	if est.has("file_bytes"):
		file = "about %.0f MB" % (float(est["file_bytes"]) / 1e6)
	return "%s x %s px  ·  %s" % [_thousands(int(est["width"])), _thousands(int(est["height"])), file]

static func _thousands(n: int) -> String:
	var s := str(n)
	var out := ""
	while s.length() > 3:
		out = " " + s.substr(s.length() - 3) + out
		s = s.substr(0, s.length() - 3)
	return s + out

## Re-reads the size ladder and each size's estimate, and redraws the list.
## Keeps the current selection when it is still enabled.
func refresh() -> void:
	var keep := _selected_width()
	_widths.clear()
	if _bridge != null and _bridge.world_gen != null and _bridge.world_gen.has_method("export_raster_widths"):
		for w in _bridge.world_gen.export_raster_widths():
			_widths.append(int(w))
	if _widths.is_empty():
		_widths = FALLBACK_WIDTHS.duplicate()
	_sizes.clear()
	_estimates.clear()
	for i in _widths.size():
		var w := _widths[i]
		var est := _estimate(w)
		_estimates[w] = est
		_sizes.add_item(size_text(w, est))
		var why := size_refusal(est)
		_sizes.set_item_disabled(i, why != "")
		_sizes.set_item_tooltip(i, why if why != "" else String(MEASURED.get(w, "")))
	## The kept (else default) width when enabled; else the largest enabled
	## width below it; else the smallest enabled one.
	var want := keep if keep > 0 else DEFAULT_WIDTH
	var pick := -1
	for i in _widths.size():
		if _sizes.is_item_disabled(i):
			continue
		if _widths[i] <= want and (pick < 0 or _widths[i] > _widths[pick]):
			pick = i
	if pick < 0:
		for i in _widths.size():
			if not _sizes.is_item_disabled(i):
				pick = i
				break
	if pick >= 0:
		_sizes.select(pick)
	_redraw_state()

func _selected_width() -> int:
	if _sizes == null or _sizes.selected < 0 or _sizes.selected >= _widths.size():
		return 0
	return _widths[_sizes.selected]

## The options dictionary `ExportOverlay.run` / `export_session_begin` take
## (`export_options.rs::ExportOptions`), from the dialog's controls.
func options() -> Dictionary:
	var content := {}
	for k in _layer_boxes:
		content[k] = (_layer_boxes[k] as CheckBox).button_pressed
	content["settlements"] = _settle.button_pressed
	if _settle.button_pressed:
		content["settlement_tier"] = String(TIERS[_tier.selected][0])
	return {"width": _selected_width(), "color_space": _space.get_item_text(_space.selected),
		"content": content}

## Why Export cannot be pressed now, or `""`. In order: running, generating,
## no world, no affordable size, no destination.
func export_refusal() -> String:
	if _running:
		return "An export is running."
	if _bridge == null or bool(_bridge.generating):
		return "A generation is running. The export snapshots a finished world; wait for it to finish."
	if not bool(_bridge.has_world):
		return "No world yet. Generate or open one first."
	if _selected_width() <= 0:
		return "No size fits in this device's free memory."
	var why := size_refusal(_estimates.get(_selected_width(), {}))
	if why != "":
		return why
	if _path == "":
		return "Choose where to save the image."
	return ""

## Enables and labels everything from the current state; the reason Export is
## off goes on its tooltip and on the status line.
func _redraw_state() -> void:
	if _go == null:
		return
	var why := export_refusal()
	_go.disabled = why != ""
	_go.tooltip_text = why
	_stop.disabled = not _running
	for c in [_sizes, _settle, _space]:
		c.disabled = _running
	_tier.disabled = _running or not _settle.button_pressed
	for k in _layer_boxes:
		(_layer_boxes[k] as CheckBox).disabled = _running
	var w := _selected_width()
	var note := String(MEASURED.get(w, "Not measured at this width."))
	var est: Dictionary = _estimates.get(w, {})
	if est.has("bands"):
		note = "%d band%s, largest band about %.2f GB in memory. %s" % [int(est["bands"]),
			"" if int(est["bands"]) == 1 else "s", float(est.get("band_peak_bytes", 0)) / 1e9, note]
	if not est.has("memory_available") and not est.is_empty():
		note += " This device does not report free memory, so no size is refused here."
	_size_note.text = note
	if not _running:
		_status.text = why

## Polls `bridge.generating`: a generation can start or end while the window
## is open, and the sizes are re-priced when it ends.
func _process(_d: float) -> void:
	if _bridge == null or not visible:
		return
	var g := bool(_bridge.generating)
	if g != _was_generating:
		_was_generating = g
		if not g:
			refresh()
		else:
			_redraw_state()

## Opens the shell's save picker (`DccBrowseDialog.choose_save_path`, as
## `menus.gd::_export_atlas` does) under the `exports` storage root.
func _choose_path() -> void:
	var fname := "map_%d.png" % _selected_width()
	DccBrowseDialog.choose_save_path(_app, "Export map image", "png",
		DccSettings.storage_root("exports"),
		"One PNG of the whole map. An existing file of the same name is replaced.",
		fname, func(p: String): set_destination(p))

## Sets the file the export writes (an absolute OS path ending `.png`).
func set_destination(p: String) -> void:
	_path = p
	_path_label.text = p if p != "" else "No file chosen"
	_path_label.tooltip_text = p
	_redraw_state()

## Starts the export with the dialog's options. Returns at once; the result
## arrives on `export_finished`. Does nothing while `export_refusal()` says why.
func start_export() -> void:
	if export_refusal() != "":
		return
	_running = true
	_bar.value = 0.0
	_status.text = "Starting"
	_redraw_state()
	_runner = EXPORT_OVERLAY.new()
	add_child(_runner)
	_runner.progress.connect(func(f: float, text: String):
		_bar.value = f
		_status.text = text)
	var vp = _app.viewport
	var t0 := Time.get_ticks_msec()
	var r: Dictionary = await _runner.run(_bridge, _path, options(), vp.overlay,
		[vp.territory_view, vp.province_view])
	_runner.queue_free()
	_runner = null
	_running = false
	if bool(r.get("ok", false)):
		_status.text = "Wrote %s x %s px, %.1f MB, in %.1f s: %s" % [r.get("width"), r.get("height"),
			float(r.get("bytes", 0)) / 1e6, (Time.get_ticks_msec() - t0) / 1000.0, r.get("path", _path)]
	elif bool(r.get("cancelled", false)):
		_bar.value = 0.0
		_status.text = "Cancelled. The partial file was removed."
	else:
		_bar.value = 0.0
		_status.text = "Export failed: %s" % r.get("error", "no reason given")
	var keep := _status.text
	_redraw_state()
	_status.text = keep
	export_finished.emit(r)

## Asks a running export to stop (`ExportOverlay.cancel`); it aborts at its
## next step and removes the partial file. Nothing when none is running.
func cancel_export() -> void:
	if _running and _runner != null:
		_runner.cancel()
		_status.text = "Cancelling"
