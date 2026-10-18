extends Node
## Windowed proof for AP-2 (`ASSET_PACK_TOOLS_PROPOSAL.md`, Ruling CQ): the
## pack report card and Q8's blank-licence export confirmation.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 \
##       _packreport_probe.tscn -- --nowelcome
##   phone composition: ... _packreport_probe.tscn -- --nowelcome --force-touch --vp 1080x2400
## (`--vp` hosts the app in a SubViewport, as `_packopen_probe.gd` does.)
##
## Needs the reference pack fixture at
## `res://../crates/cartalith-assets/tests/fixtures/reference_pack.zip` and a
## DLL with `as_pack_report`; exits 2 and says so when either is missing.
## `--headless` is refused: the cards are drawn subwindows it photographs.
##
## What it establishes:
##   1. Assets ▸ Pack report… exists, with a tooltip.
##   2. With the fixture opened into the library, pressing the inspector's
##      Report chip (the real button, by `pressed`) draws the card; its row
##      count equals `as_pack_report()`'s finding count (or the cap plus a
##      tail row), its first row is Validate's own first warning verbatim, and
##      the report names Sea marks as the empty family. CONTROL: with an empty
##      library the report holds only Validate's warnings.
##   3. A row's Show closes the card and focuses that row's slot.
##   4. Q8: with the licence blank, the window's Export chip raises "Export
##      without a licence"; Cancel opens no save picker; Export as CC0 does.
##      With a licence set, no card is raised. The exported bytes still say
##      "CC0" (read back with `ZIPReader`).
## Prints `PACKREPORT ok|FAIL <what>` and `PACKREPORT checks=<n> failures=<n>`;
## saves PNGs under `user://packreport/`. Exit 0 / 1 / 2 as `_packopen_probe`.

const FIXTURE := "res://../crates/cartalith-assets/tests/fixtures/reference_pack.zip"

var app: Node
var _view: Viewport
var _fails := 0
var _checks := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("PACKREPORT %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _walk(n: Node, out: Array) -> void:
	for c in n.get_children(true):
		out.append(c)
		_walk(c, out)

## Every visible Window under the root whose title starts with `prefix`.
func _windows(prefix: String) -> Array:
	var all: Array = []
	_walk(get_tree().root, all)
	var out: Array = []
	for n in all:
		if n is Window and (n as Window).visible and String((n as Window).title).begins_with(prefix):
			out.append(n)
	return out

func _button_in(root: Node, text: String) -> Button:
	var all: Array = []
	_walk(root, all)
	for n in all:
		if n is Button and (n as Button).text == text and (n as Button).is_visible_in_tree():
			return n
	return null

## The rows a report card drew: HBoxContainers directly inside the card's
## list column, which `modal_list_row` adds one per row.
func _card_rows(card: Node) -> int:
	var all: Array = []
	_walk(card, all)
	for n in all:
		if n is VBoxContainer and n.has_meta(DccWidgets.MODAL_DOTS_META):
			var c := 0
			for ch in (n as VBoxContainer).get_children():
				if ch is HBoxContainer:
					c += 1
			return c
	return -1

func _close_all(prefix: String) -> void:
	for w in _windows(prefix):
		(w as Window).hide()
		w.queue_free()
	await _frames(4)

func _snap(tag: String) -> void:
	await RenderingServer.frame_post_draw
	var img := _view.get_texture().get_image()
	DirAccess.make_dir_recursive_absolute("user://packreport")
	var path := "user://packreport/%s_%s.png" % [tag, "phone" if DccTheme.is_phone() else "desktop"]
	img.save_png(path)
	print("PACKREPORT saved %s" % ProjectSettings.globalize_path(path))

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("PACKREPORT REFUSED: headless (the cards are drawn subwindows; run windowed)")
		get_tree().quit(2)
		return
	var fixture := ProjectSettings.globalize_path(FIXTURE)
	if not FileAccess.file_exists(fixture):
		print("PACKREPORT CANNOT RUN: no fixture at %s" % fixture)
		get_tree().quit(2)
		return
	app = load("res://shell/app.tscn").instantiate()
	_view = get_viewport()
	var args := OS.get_cmdline_user_args()
	var vp_at := args.find("--vp")
	if vp_at >= 0 and vp_at + 1 < args.size():
		var parts := String(args[vp_at + 1]).split("x")
		var sv := SubViewport.new()
		sv.size = Vector2i(int(parts[0]), int(parts[1]))
		sv.gui_embed_subwindows = true
		sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		add_child(sv)
		Input.set_emulate_touch_from_mouse(true)
		sv.add_child(app)
		_view = sv
	else:
		add_child(app)
	await get_tree().create_timer(1.0).timeout
	if "--nowelcome" in args and app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
		await _frames(2)
	if not app.bridge.world_gen.has_method("as_pack_report"):
		print("PACKREPORT CANNOT RUN: this DLL has no as_pack_report")
		get_tree().quit(2)
		return
	print("PACKREPORT view=%s palette=%s phone=%s" % [_view.get_visible_rect().size,
		"dark" if DccTheme.is_dark() else "light", DccTheme.is_phone()])

	# -- 1. the menu row -----------------------------------------------------
	var all: Array = []
	_walk(app, all)
	var tip := ""
	var report_menu: PopupMenu = null
	var report_id := -1
	for n in all:
		if n is PopupMenu:
			for i in (n as PopupMenu).item_count:
				if (n as PopupMenu).get_item_text(i) == "Pack report…":
					tip = (n as PopupMenu).get_item_tooltip(i)
					report_menu = n
					report_id = (n as PopupMenu).get_item_id(i)
	_ok("Assets carries Pack report… with a tooltip", tip.contains("Read-only"), tip)

	# -- 2. the report card --------------------------------------------------
	app.bridge.as_clear_library()
	var empty: Dictionary = app.bridge.as_pack_report()
	var kinds: Array = []
	for f in empty.get("findings", []):
		kinds.append(String((f as Dictionary).get("kind", "")))
	_ok("CONTROL: an empty library reports only Validate's own warnings and the blank licence",
		not kinds.is_empty() and kinds.all(func(k): return k == "validator" or k == "licence"), str(kinds))

	## Open pack in library… opens the window before its picker; the picker
	## itself is not driven here (`_packopen_probe` covers that row).
	app.open_asset_library()
	await _frames(10)
	app._import_asset_pack(fixture, false)
	await _frames(12)
	await _close_all("Pack opened")
	var win: Window = app.asset_library_window
	_ok("the fixture is in the library", int(app.bridge.as_pack_info().get("total_items", 0)) == 17)
	var r: Dictionary = app.bridge.as_pack_report()
	var findings: Array = r.get("findings", [])
	## Desktop and tablet: the inspector foot's Report chip, pressed. On the
	## phone the inspector is a pane behind the slot grid, so the route a user
	## has from anywhere is the menu row (the phone menu walks the same
	## popup), and that row's own `id_pressed` is what is emitted.
	if DccTheme.is_phone():
		_ok("phone: the Pack report… row is there to press", report_menu != null)
		report_menu.id_pressed.emit(report_id)
	else:
		var report_btn := _button_in(win, "Report")
		_ok("the inspector foot carries a Report chip", report_btn != null)
		if report_btn == null:
			print("PACKREPORT checks=%d failures=%d" % [_checks, _fails])
			get_tree().quit(1)
			return
		report_btn.pressed.emit()
	await _frames(12)
	var cards := _windows("Pack report")
	_ok("pressing Report draws the card", cards.size() == 1, "cards=%d" % cards.size())
	if cards.is_empty():
		print("PACKREPORT checks=%d failures=%d" % [_checks, _fails])
		get_tree().quit(1)
		return
	var drawn := _card_rows(cards[0])
	var want := findings.size() if findings.size() <= AssetLibraryWindow.CARD_ROW_CAP else AssetLibraryWindow.CARD_ROW_CAP + 1
	_ok("the card draws one row per finding", drawn == want, "rows=%d findings=%d" % [drawn, findings.size()])
	var validate: PackedStringArray = app.bridge.as_validate()
	_ok("the report's first row is Validate's first warning, verbatim",
		not validate.is_empty() and String((findings[0] as Dictionary).get("text", "")) == String(validate[0]),
		"%s | %s" % [str(findings[0]), validate[0] if not validate.is_empty() else ""])
	var empty_families: Array = []
	for f in findings:
		if String((f as Dictionary).get("kind", "")) == "empty_family":
			empty_families.append(String((f as Dictionary).get("label", "")))
	_ok("Sea marks is the one empty family", empty_families == ["Sea marks"], str(empty_families))
	await _snap("01_report")

	# -- 3. Show ----------------------------------------------------------------
	var first_uid := ""
	for f in findings:
		if (f as Dictionary).has("uid"):
			first_uid = String((f as Dictionary)["uid"])
			break
	var show := _button_in(cards[0], "Show")
	_ok("a row carries Show", show != null)
	if show != null:
		show.pressed.emit()
		await _frames(10)
		_ok("Show closes the card and focuses the first slot-row's slot",
			_windows("Pack report").is_empty() and String(win.get("_focused_uid")) == first_uid,
			"focused='%s' want='%s'" % [String(win.get("_focused_uid")), first_uid])
		await _snap("02_show")

	# -- 4. Q8: export with a blank licence ---------------------------------------
	var info: Dictionary = app.bridge.as_pack_info()
	app.bridge.as_set_pack_info(String(info["name"]), String(info["author"]), "")
	win.call("_refresh_pack_status")
	var export_btn := _button_in(win, "Export pack .zip")
	if export_btn == null:
		export_btn = _button_in(win, "Export .zip")
	_ok("the window bar carries the Export chip", export_btn != null)
	export_btn.pressed.emit()
	await _frames(10)
	var confirm := _windows("Export without a licence")
	_ok("a blank licence raises the confirmation", confirm.size() == 1, "cards=%d" % confirm.size())
	await _snap("03_licence_confirm")
	if not confirm.is_empty():
		var cancel := _button_in(confirm[0], "Cancel")
		_ok("it offers Cancel and Export as CC0", cancel != null and _button_in(confirm[0], "Export as CC0") != null)
		cancel.pressed.emit()
		await _frames(8)
		_ok("Cancel opens no save picker", _windows("Export pack .zip").is_empty())
	export_btn.pressed.emit()
	await _frames(10)
	confirm = _windows("Export without a licence")
	if not confirm.is_empty():
		_button_in(confirm[0], "Export as CC0").pressed.emit()
	await _frames(10)
	_ok("Export as CC0 opens the save picker", _windows("Export pack .zip").size() == 1)
	await _close_all("Export pack .zip")
	var bytes: PackedByteArray = app.bridge.as_export_pack_bytes().get("bytes", PackedByteArray())
	DirAccess.make_dir_recursive_absolute("user://packreport")
	var f := FileAccess.open("user://packreport/out.zip", FileAccess.WRITE)
	f.store_buffer(bytes)
	f.close()
	var zr := ZIPReader.new()
	var pack_json := ""
	if zr.open("user://packreport/out.zip") == OK:
		pack_json = zr.read_file("pack.json").get_string_from_utf8()
		zr.close()
	_ok("the exported pack.json still says CC0 (bytes unchanged by Q8)",
		pack_json.contains("\"license\": \"CC0\"") or pack_json.contains("\"license\":\"CC0\""), pack_json.left(160))
	app.bridge.as_set_pack_info(String(info["name"]), String(info["author"]), "MIT")
	win.call("_refresh_pack_status")
	export_btn.pressed.emit()
	await _frames(10)
	_ok("with a licence set, export asks nothing and opens the picker",
		_windows("Export without a licence").is_empty() and _windows("Export pack .zip").size() == 1)
	await _close_all("Export pack .zip")

	print("PACKREPORT checks=%d failures=%d" % [_checks, _fails])
	get_tree().quit(0 if _fails == 0 else 1)
