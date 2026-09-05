extends Node
## Committed verification harness for the Data manager's route-pane **pattern**
## (`design/proposed-2026-09-05/DataPane.dc.html`, owner-approved 2026-09-05).
##
## Five things are asserted, and none of them is readable from the source:
##
##   1. **The token and metric mapping**, pinned against the ARTBOARD's own
##      literals -- a different file from `dcc_theme.gd`, so mutating a palette
##      entry or one of the pane's new constants turns this red. Nothing here
##      compares a constant to itself.
##   2. **The anatomy**, read off the drawn nodes: title, purpose, two
##      `--div` rules, the FORMAT track, the INCLUDE chips, EXTENT, the TO row
##      with its `Browse…`, the receipt block, the action row.
##   3. **Which routes render through the pattern and which are bespoke** --
##      measured by selecting all fourteen and looking for the pattern's own
##      title label on each, not by reading the dispatch `match`.
##   4. **No invented data.** Every lit INCLUDE chip's number is cross-checked
##      against the document a real export produced -- two independent sources,
##      the shell's getters and the written file. Every dashed chip must carry
##      a non-empty reason.
##   5. **The receipt states what was NOT included**, and its absent state is a
##      dash with a reason rather than a success line full of zeroes.
##
## Forced dark, and refuses to run otherwise: the artboard is the dark canvas
## and this machine's `cartalith_settings.cfg` boots `mode="light"`.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _datapane_probe.tscn

const OUT_PATH := "user://_datapane_probe_out.geojson"

var app: Node
var dm: DataManagerWindow
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("DP %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _eq(name: String, got, want) -> void:
	_check(name, got == want, "got=%s want=%s" % [got, want])

# -- Reading the drawn tree, never the variables that built it ----------------

func _walk(n: Node, out: Array, want: String) -> void:
	if (want == "Label" and n is Label) or (want == "Button" and n is Button) \
			or (want == "PanelContainer" and n is PanelContainer) \
			or (want == "ColorRect" and n is ColorRect):
		out.append(n)
	for c in n.get_children(true):
		_walk(c, out, want)

func _labels(root: Node) -> Array:
	var out: Array = []
	_walk(root, out, "Label")
	return out

func _texts(root: Node) -> PackedStringArray:
	var out := PackedStringArray()
	for l in _labels(root):
		out.append((l as Label).text)
	return out

func _buttons(root: Node) -> Array:
	var out: Array = []
	_walk(root, out, "Button")
	var vis: Array = []
	for b in out:
		if (b as Button).visible:
			vis.append(b)
	return vis

func _button_named(root: Node, text: String) -> Button:
	for b in _buttons(root):
		if (b as Button).text == text:
			return b
	return null

func _label_starting(root: Node, prefix: String) -> Label:
	for l in _labels(root):
		if (l as Label).text.begins_with(prefix):
			return l
	return null

func _panels(root: Node) -> Array:
	var out: Array = []
	_walk(root, out, "PanelContainer")
	return out

func _sb(p: PanelContainer) -> StyleBoxFlat:
	var s := p.get_theme_stylebox("panel")
	return s as StyleBoxFlat

# ---------------------------------------------------------------------------

func _ready() -> void:
	DccTheme.apply_theme(true)
	if not DccTheme.is_dark():
		print("DP FAIL  could not force the dark palette -- refusing to run")
		get_tree().quit(1)
		return

	# --- 0. The token mapping, pinned against `DataPane.dc.html`'s `.tok` ----
	## Every literal below is copied from the artboard, which is a different
	## file from the one being asserted.
	_eq("--pan   -> panel", DccTheme.c("panel"), Color("#121314"))
	_eq("--ins   -> sunken", DccTheme.c("sunken"), Color("#191c1e"))
	_eq("--ink   -> text_bright", DccTheme.c("text_bright"), Color("#e8ebec"))
	_eq("--body  -> text", DccTheme.c("text"), Color("#c8cbcd"))
	_eq("--sec   -> text_secondary", DccTheme.c("text_secondary"), Color("#a9adb0"))
	_eq("--dim   -> text_dim", DccTheme.c("text_dim"), Color("#8d9296"))
	_eq("--faint -> text_faint", DccTheme.c("text_faint"), Color("#6f7478"))
	_eq("--dis   -> text_ghost", DccTheme.c("text_ghost"), Color("#5f6468"))
	_eq("--acc   -> accent", DccTheme.c("accent"), Color("#e0a34a"))
	_eq("--accInk-> accent_ink", DccTheme.c("accent_ink"), Color("#141005"))
	_eq("--good  -> good", DccTheme.c("good"), Color("#6fae7d"))
	_eq("--div   -> line_soft", DccTheme.c("line_soft"), Color(1, 1, 1, 0.07))
	_check("--wash2 alpha is .16",
		abs(DccTheme.c("accent_wash_2").a - 0.16) < 0.005,
		str(DccTheme.c("accent_wash_2").a))

	# --- 0b. The metrics, likewise off the artboard's own numbers -----------
	_eq("--m2 9px  -> FS_MICRO", DccTheme.FS_MICRO, 9)
	_eq("--m1 10px -> FS_TINY", DccTheme.FS_TINY, 10)
	_eq("--ctl 24px  -> DccWidgets.MODAL_CTL", DccWidgets.MODAL_CTL, 24)
	_eq("--btnH 28px -> DccWidgets.MODAL_BTN_H", DccWidgets.MODAL_BTN_H, 28)
	_eq("radius 8 -> DccWidgets.MODAL_INSET_RADIUS",
		DccWidgets.MODAL_INSET_RADIUS, 8)
	_eq("pane title font-size:14px", DataManagerWindow.PATTERN_TITLE_FS, 14)
	_eq("form label width:74px", DataManagerWindow.PATTERN_LABEL_W, 74)
	_eq("chip border-radius:999px", DataManagerWindow.PATTERN_CHIP_R, 999)
	_eq("chip padding-x 11px", DataManagerWindow.PATTERN_CHIP_PAD_X, 11)
	_eq("chip padding-y 3px", DataManagerWindow.PATTERN_CHIP_PAD_Y, 3)
	_eq("track border-radius:14px", DataManagerWindow.PATTERN_TRACK_R, 14)
	_eq("track padding:2px", DataManagerWindow.PATTERN_TRACK_PAD, 2)
	_eq("segment border-radius:12px", DataManagerWindow.PATTERN_SEG_R, 12)
	_eq("segment padding 3px 12px",
		[DataManagerWindow.PATTERN_SEG_PAD_Y, DataManagerWindow.PATTERN_SEG_PAD_X],
		[3, 12])
	_eq("receipt padding 9px 11px",
		[DataManagerWindow.PATTERN_RECEIPT_PAD_Y, DataManagerWindow.PATTERN_RECEIPT_PAD_X],
		[9, 11])
	_eq("rule margins 12/13 + 12",
		[DataManagerWindow.PATTERN_RULE_TOP, DataManagerWindow.PATTERN_RULE_TOP_2,
			DataManagerWindow.PATTERN_RULE_BOTTOM], [12, 13, 12])

	# --- 0c. Every route has a purpose line ---------------------------------
	var missing := PackedStringArray()
	for r in DataManagerWindow.ROUTES:
		if not DataManagerWindow.PANE_PURPOSE.has(String(r["id"])):
			missing.append(String(r["id"]))
	_check("every ROUTES id has a PANE_PURPOSE", missing.size() == 0,
		", ".join(missing))
	_eq("PANE_PURPOSE has no orphan rows",
		DataManagerWindow.PANE_PURPOSE.size(), DataManagerWindow.ROUTES.size())

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	DccTheme.apply_theme(true)

	await _generate()
	dm = app.data_manager_window
	dm.open_route("export_gis")
	await _frames(4)

	await _coverage()
	await _anatomy()
	await _browse_is_the_shell_browser()
	await _no_invented_counts()
	await _gap_receipt()

	print("=== DATA PANE ", "OK" if _fail == 0 else "FAILED (%d)" % _fail, " ===")
	get_tree().quit(0 if _fail == 0 else 1)

func _generate() -> void:
	app.bridge.generate({"seed": 40417, "width_km": 2000.0, "grid_w": 256,
		"grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while app.bridge.generating and waited < 6000:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	_check("world generated", app.bridge.has_world,
		"%d settlements" % app.bridge.settlements().size())

# ---------------------------------------------------------------------------
# 3. Which routes render through the pattern
# ---------------------------------------------------------------------------

## The pattern's own tell, read off the drawing: a `Label` whose text is the
## route's `label` and whose font size override is `PATTERN_TITLE_FS`. The two
## bespoke panes draw no such label -- their first body node is a `_col_header`
## in mono at `FS_MICRO`.
func _has_pattern_title(route: Dictionary) -> bool:
	for l in _labels(dm._pane_body):
		var lb := l as Label
		if lb.text == String(route["label"]) \
				and lb.has_theme_font_size_override("font_size") \
				and lb.get_theme_font_size("font_size") == DataManagerWindow.PATTERN_TITLE_FS:
			return true
	return false

func _coverage() -> void:
	print("[3] pattern coverage")
	var pattern := PackedStringArray()
	var bespoke := PackedStringArray()
	for r in DataManagerWindow.ROUTES:
		dm.open_route(String(r["id"]))
		await _frames(2)
		if _has_pattern_title(r):
			pattern.append(String(r["id"]))
		else:
			bespoke.append(String(r["id"]))
	print("DP      pattern (%d): %s" % [pattern.size(), ", ".join(pattern)])
	print("DP      bespoke (%d): %s" % [bespoke.size(), ", ".join(bespoke)])
	_eq("12 routes render through the pattern", pattern.size(), 12)
	_eq("2 routes keep a bespoke body", Array(bespoke),
		["export_maps", "export_world"])

# ---------------------------------------------------------------------------
# 2. The anatomy, on the one route that has every part
# ---------------------------------------------------------------------------

func _anatomy() -> void:
	print("[2] anatomy -- Export > GIS / GeoJSON")
	dm.open_route("export_gis")
	await _frames(3)
	var body: Control = dm._pane_body
	var texts := _texts(body)

	var title := _label_starting(body, "GIS / GeoJSON")
	_check("title drawn at 14 px in --ink", title != null
		and title.get_theme_font_size("font_size") == DataManagerWindow.PATTERN_TITLE_FS
		and title.get_theme_color("font_color") == DccTheme.c("text_bright"),
		"" if title == null else "%d px %s" % [title.get_theme_font_size("font_size"),
			title.get_theme_color("font_color")])

	var purpose := _label_starting(body, "every generated entity")
	_check("one-line purpose under it, --m2 --faint", purpose != null
		and purpose.get_theme_font_size("font_size") == DccTheme.FS_MICRO
		and purpose.get_theme_color("font_color") == DccTheme.c("text_faint"))

	## Two `.rule`s at `--div`, and they must not be `DccTheme.rule()`'s `line`.
	var rects: Array = []
	_walk(body, rects, "ColorRect")
	var soft := 0
	for r in rects:
		if (r as ColorRect).color == DccTheme.c("line_soft"):
			soft += 1
	_eq("two --div rules in the pane body", soft, 2)

	_check("FORMAT / INCLUDE / EXTENT / TO labels present",
		texts.has("FORMAT") and texts.has("INCLUDE") and texts.has("EXTENT")
			and texts.has("TO"), ", ".join(texts.slice(0, 8)))

	## The FORMAT track: an `--ins` PanelContainer at radius 14 holding three
	## segment buttons, exactly one of them lit at `--wash2`.
	var track: PanelContainer = null
	for p in _panels(body):
		var sb := _sb(p)
		if sb != null and sb.corner_radius_top_left == DataManagerWindow.PATTERN_TRACK_R \
				and sb.bg_color == DccTheme.c("sunken"):
			track = p
			break
	_check("FORMAT is a track, --ins at radius 14", track != null)
	if track != null:
		var segs := _buttons(track)
		_eq("three format segments", segs.size(), 3)
		var lit := 0
		for b in segs:
			var sb: StyleBoxFlat = (b as Button).get_theme_stylebox("normal") as StyleBoxFlat
			if sb != null and sb.bg_color == DccTheme.c("accent_wash_2"):
				lit += 1
				_eq("the lit segment is GeoJSON", (b as Button).text, "GeoJSON")
				_eq("lit segment ink is --acc",
					(b as Button).get_theme_color("font_color"), DccTheme.c("accent"))
			else:
				_check("dead segment %s carries its reason" % (b as Button).text,
					(b as Button).tooltip_text != "")
		_eq("exactly one segment lit", lit, 1)

	## The receipt block and the TO well are both `--ins` at radius 8.
	var insets := 0
	for p in _panels(body):
		var sb := _sb(p)
		if sb != null and sb.bg_color == DccTheme.c("sunken") \
				and sb.corner_radius_top_left == DccWidgets.MODAL_INSET_RADIUS:
			insets += 1
	_check("TO well + receipt block drawn as --ins radius 8", insets >= 2,
		"found %d" % insets)

	var browse := _button_named(body, "Browse…")
	_check("Browse… sits in the destination row", browse != null)
	if browse != null:
		_eq("Browse… is shrunk to --ctl", int(browse.custom_minimum_size.y),
			DccWidgets.MODAL_CTL)

	## The action row is the pinned footer: quiet action first, primary last.
	var acts := _buttons(dm._pane_footer)
	_eq("action row is Reveal file then Export",
		[(acts[0] as Button).text if acts.size() > 0 else "",
			(acts[1] as Button).text if acts.size() > 1 else ""],
		["Reveal file", "Export"])
	if acts.size() > 1:
		var primary: StyleBoxFlat = (acts[1] as Button).get_theme_stylebox("normal") as StyleBoxFlat
		_eq(".btn is the accent fill", primary.bg_color, DccTheme.c("accent"))
		_eq(".btn ink is --accInk",
			(acts[1] as Button).get_theme_color("font_color"), DccTheme.c("accent_ink"))
		var quiet: StyleBoxFlat = (acts[0] as Button).get_theme_stylebox("normal") as StyleBoxFlat
		_eq(".btn2 is the --ins fill", quiet.bg_color, DccTheme.c("sunken"))
		_check("Reveal file is disabled before anything is written",
			(acts[0] as Button).disabled and (acts[0] as Button).tooltip_text != "")

# ---------------------------------------------------------------------------
# The destination row goes through the shell's own browser
# ---------------------------------------------------------------------------

func _browse_is_the_shell_browser() -> void:
	print("[2b] Browse… raises DccBrowseDialog, not a stock FileDialog")
	dm.open_route("export_gis")
	await _frames(3)
	var before_stock := _count_of(dm, "FileDialog")
	var browse := _button_named(dm._pane_body, "Browse…")
	if browse == null:
		_check("Browse… present", false)
		return
	browse.emit_signal("pressed")
	await _frames(4)
	var browsers := _count_of(dm, "DccBrowseDialog")
	_check("a DccBrowseDialog was spawned", browsers >= 1, "%d" % browsers)
	_eq("no stock FileDialog appeared", _count_of(dm, "FileDialog"), before_stock)
	for c in dm.get_children():
		if c is DccBrowseDialog:
			c.queue_free()
	await _frames(3)

func _count_of(root: Node, cls: String) -> int:
	var n := 0
	if root.is_class(cls) or (root.get_script() != null and cls == "DccBrowseDialog"
			and root is DccBrowseDialog):
		n += 1
	for c in root.get_children(true):
		n += _count_of(c, cls)
	return n

# ---------------------------------------------------------------------------
# 4 + 5. No invented data, and the receipt says what was left out
# ---------------------------------------------------------------------------

## Every INCLUDE chip, as `{group: text}` read off the drawn labels inside the
## pill PanelContainers.
func _chip_texts() -> Dictionary:
	var out := {}
	for p in _panels(dm._pane_body):
		var sb := _sb(p)
		if sb == null or sb.corner_radius_top_left < 90:
			continue
		var ls := _labels(p)
		if ls.is_empty():
			continue
		var t: String = (ls[0] as Label).text
		out[t.split(" ")[0]] = {"text": t, "tip": p.tooltip_text,
			"ink": (ls[0] as Label).get_theme_color("font_color"),
			"bg": sb.bg_color}
	return out

func _no_invented_counts() -> void:
	print("[4] no invented data")
	dm.open_route("export_gis")
	await _frames(3)
	var chips := _chip_texts()
	_eq("one chip per GIS_GROUPS row", chips.size(),
		DataManagerWindow.GIS_GROUPS.size())

	var dashed := 0
	for k in chips:
		var c: Dictionary = chips[k]
		if String(c["text"]).ends_with("—"):
			dashed += 1
			_check("dashed chip '%s' carries its reason" % k, String(c["tip"]) != "")
			_eq("dashed chip '%s' is dimmed, not hidden" % k, c["ink"],
				DccTheme.c("text_ghost"))
	print("DP      %d dashed chip(s) of %d" % [dashed, chips.size()])
	_check("rivers is dashed before a run -- no binding counts them",
		chips.has("rivers") and String(chips["rivers"]["text"]).ends_with("—"))
	_check("landmarks is drawn dim, not lit -- no GeoJSON layer carries it",
		chips.has("landmarks")
			and chips["landmarks"]["bg"] == DccTheme.c("sunken"))

	# -- run a real export and cross-check the chips against the document ----
	dm._gis_dest = ProjectSettings.globalize_path(OUT_PATH)
	var go := _button_named(dm._pane_footer, "Export")
	_check("Export is enabled with a world loaded", go != null and not go.disabled)
	if go == null or go.disabled:
		return
	go.emit_signal("pressed")
	await _frames(6)

	var f := FileAccess.open(OUT_PATH, FileAccess.READ)
	_check("a document was written", f != null)
	if f == null:
		return
	var doc = JSON.parse_string(f.get_as_text())
	f.close()
	_check("the document parses as a FeatureCollection",
		typeof(doc) == TYPE_DICTIONARY and (doc as Dictionary).has("features"))
	if typeof(doc) != TYPE_DICTIONARY:
		return
	var layers := {}
	for fe in (doc as Dictionary)["features"]:
		var props: Dictionary = (fe as Dictionary).get("properties", {})
		if props.has("layer"):
			var key := String(props["layer"])
			layers[key] = int(layers.get(key, 0)) + 1
	print("DP      document layers: %s" % str(layers))

	## The independent oracle: the chip numbers came from the shell's getters,
	## these came from the file the engine wrote. A fabricated chip disagrees.
	var pairs := {"settlements": "settlement", "ways": "way",
		"provinces": "province"}
	for group in pairs:
		if not chips.has(group):
			continue
		var shown := String(chips[group]["text"])
		if shown.ends_with("—"):
			continue
		var n := int(shown.split(" ")[1])
		_eq("chip '%s' matches the written document" % group, n,
			int(layers.get(String(pairs[group]), -1)))

	# -- the receipt -------------------------------------------------------
	await _frames(3)
	var recv := _label_starting(dm._pane_body, "wrote ")
	_check("receipt reports the measured feature count", recv != null,
		"" if recv == null else recv.text)
	if recv != null:
		_eq("receipt's feature count is the document's", recv.text,
			"wrote %d features · %s" % [int((doc as Dictionary)["features"].size()),
				dm._fmt_bytes(int(FileAccess.get_file_as_bytes(OUT_PATH).size()))])
	var omitted := _label_starting(dm._pane_body, "landmarks and religions")
	_check("the receipt states what was NOT included", omitted != null,
		"" if omitted == null else omitted.text)
	var ago := _label_starting(dm._pane_body, "0.")
	_check("the receipt carries an age, measured not stamped",
		ago != null and ago.text.ends_with("s ago"),
		"" if ago == null else ago.text)
	## Rivers were dashed before the run and are counted in the document.
	_check("rivers were written even though the chip could not count them",
		layers.has("river"), str(layers.get("river", 0)))

func _gap_receipt() -> void:
	print("[5] the absent state on a gap route")
	dm.open_route("val_check")
	await _frames(3)
	var body: Control = dm._pane_body
	var texts := _texts(body)
	_check("no form region on a gap route",
		not texts.has("FORMAT") and not texts.has("INCLUDE")
			and not texts.has("TO"))
	var dash := _label_starting(body, "— nothing has run here")
	_check("receipt is a dash carrying its reason", dash != null,
		", ".join(texts.slice(maxi(0, texts.size() - 3))))
	var run := _button_named(dm._pane_footer, "Run")
	_check("the action row's Run is disabled and says why",
		run != null and run.disabled and run.tooltip_text != "")
