extends Node
## Windowed proof for AP-1 (`ASSET_PACK_TOOLS_PROPOSAL.md`, Ruling CQ Q1):
## opening an asset pack `.zip` into the Asset library.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 \
##       _packopen_probe.tscn -- --nowelcome
##   phone composition:  ... _packopen_probe.tscn -- --nowelcome --force-touch --vp 1080x2400
##   tablet composition: ... _packopen_probe.tscn -- --nowelcome --force-touch --vp 2560x1600
## `--vp WxH` hosts the app in a SubViewport of that size (the `_compfit_probe`
## pattern), because a desktop monitor clamps a `--resolution` taller than
## itself and the shell then picks the tablet composition. Neither is the
## real device (`MISTAKES.md`: `--force-touch` is not the phone).
##
## Needs the reference pack fixture at
## `res://../crates/cartalith-assets/tests/fixtures/reference_pack.zip` (where it
## lives in the repository) and a DLL that has `as_import_pack`; it exits 2 and
## says so when either is missing, rather than reporting a pass it cannot make.
##
## **Windowed, and `--headless` is refused**: the cards it captures are drawn
## subwindows, and `MISTAKES.md`'s "run a pixel probe" row applies to the
## screenshots it saves for a human to read.
##
## What it establishes, in order:
##   1. The Assets menu carries both rows, Import asset pack .zip… and Open pack
##      in library…, each with its own tooltip, and pressing the second (by
##      emitting the popup's own `id_pressed`) opens the library window and a
##      file picker.
##   2. The picker's callback (`app._import_asset_pack(path, false)`, the one
##      function the picker calls; the picker itself is not driven) opens the
##      fixture into the library: 17 items, the result card is drawn, and the
##      MAP IS NOT TOUCHED (`has_asset_pack()` stays false).
##   3. Opening it again raises the Replace / Cancel card; Cancel changes
##      nothing; Replace leaves the same 17 items (no doubling).
##   4. A result row's Show button closes the card and focuses its slot.
##   5. Import pack… (`apply_to_map` true) loads the map AND fills the library.
##   6. Refusals: a non-zip file and a path-traversal zip are refused with a
##      reason on the status line and change neither the library nor the map;
##      mid-generation the bridge refuses with its busy text.
## Prints `PACKOPEN ok|FAIL <what>` rows and `PACKOPEN checks=<n> failures=<n>`;
## saves PNGs under `user://packopen/` (`PACKOPEN saved <path>`). Exit 0 when
## every check held, 1 when one failed, 2 when it could not run.

const FIXTURE := "res://../crates/cartalith-assets/tests/fixtures/reference_pack.zip"
const OPEN_ROW := "Open pack in library…"
const IMPORT_ROW := "Import asset pack .zip…"

var app: Node
## The viewport the app draws into: the window's, or the `--vp` SubViewport.
var _view: Viewport
var _fails := 0
var _checks := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("PACKOPEN %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _walk(n: Node, out: Array) -> void:
	for c in n.get_children(true):
		out.append(c)
		_walk(c, out)

## Every visible AcceptDialog under the app whose title starts with `prefix`.
func _cards(prefix: String) -> Array:
	var all: Array = []
	_walk(get_tree().root, all)
	var out: Array = []
	for n in all:
		if n is AcceptDialog and (n as Window).visible and String((n as Window).title).begins_with(prefix):
			out.append(n)
	return out

func _button_in(root: Node, text: String) -> Button:
	var all: Array = []
	_walk(root, all)
	for n in all:
		if n is Button and (n as Button).text == text and (n as Button).is_visible_in_tree():
			return n
	return null

func _items() -> int:
	return int(app.bridge.as_pack_info().get("total_items", -1))

func _hint() -> String:
	var labels: Dictionary = app.get("_status_labels")
	return String((labels["hint"] as Label).text) if labels.has("hint") else ""

func _snap(tag: String) -> void:
	await RenderingServer.frame_post_draw
	var img := _view.get_texture().get_image()
	DirAccess.make_dir_recursive_absolute("user://packopen")
	var path := "user://packopen/%s_%s.png" % [tag, "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")]
	img.save_png(path)
	print("PACKOPEN saved %s" % ProjectSettings.globalize_path(path))

## A STORED zip, built by hand, holding one entry named `name`. Its CRC is
## deliberately 0: the engine must refuse the NAME before it reads any data.
func _zip_with_name(name: String) -> PackedByteArray:
	var nb := name.to_utf8_buffer()
	var data := "x".to_utf8_buffer()
	var b := PackedByteArray()
	_put32(b, 0x04034b50); _put16(b, 20); _put16(b, 0); _put16(b, 0)
	_put16(b, 0); _put16(b, 0x21); _put32(b, 0); _put32(b, data.size())
	_put32(b, data.size()); _put16(b, nb.size()); _put16(b, 0)
	b.append_array(nb); b.append_array(data)
	var cd_off := b.size()
	_put32(b, 0x02014b50); _put16(b, 20); _put16(b, 20); _put16(b, 0)
	_put16(b, 0); _put16(b, 0); _put16(b, 0x21); _put32(b, 0)
	_put32(b, data.size()); _put32(b, data.size()); _put16(b, nb.size())
	_put16(b, 0); _put16(b, 0); _put16(b, 0); _put16(b, 0)
	_put32(b, 0); _put32(b, 0)
	b.append_array(nb)
	var cd_len := b.size() - cd_off
	_put32(b, 0x06054b50); _put16(b, 0); _put16(b, 0); _put16(b, 1)
	_put16(b, 1); _put32(b, cd_len); _put32(b, cd_off); _put16(b, 0)
	return b

## Appends `v` little-endian, four bytes.
func _put32(arr: PackedByteArray, v: int) -> void:
	for k in 4:
		arr.append((v >> (8 * k)) & 0xFF)

## Appends `v` little-endian, two bytes.
func _put16(arr: PackedByteArray, v: int) -> void:
	for k in 2:
		arr.append((v >> (8 * k)) & 0xFF)

func _write(path: String, bytes: PackedByteArray) -> String:
	DirAccess.make_dir_recursive_absolute("user://packopen")
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_buffer(bytes)
	f.close()
	return ProjectSettings.globalize_path(path)

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("PACKOPEN REFUSED: headless (the cards are drawn subwindows; run windowed)")
		get_tree().quit(2)
		return
	var fixture := ProjectSettings.globalize_path(FIXTURE)
	if not FileAccess.file_exists(fixture):
		print("PACKOPEN CANNOT RUN: no fixture at %s" % fixture)
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
	if "--nowelcome" in OS.get_cmdline_user_args() and app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
		await _frames(2)
	if not app.bridge.world_gen.has_method("as_import_pack"):
		print("PACKOPEN CANNOT RUN: this DLL has no as_import_pack")
		get_tree().quit(2)
		return
	print("PACKOPEN view=%s palette=%s phone=%s tablet=%s" % [_view.get_visible_rect().size,
		"dark" if DccTheme.is_dark() else "light", DccTheme.is_phone(), DccTheme.is_tablet()])
	app.bridge.as_clear_library()
	var had_pack: bool = app.bridge.has_asset_pack()
	_ok("CONTROL: the session starts with an empty library and no pack on the map",
		_items() == 0 and not had_pack, "items=%d has_asset_pack=%s" % [_items(), had_pack])

	# -- 1. the two menu rows ------------------------------------------------
	var all: Array = []
	_walk(app, all)
	var assets: PopupMenu = null
	for n in all:
		if n is PopupMenu:
			var pm := n as PopupMenu
			for i in pm.item_count:
				if pm.get_item_text(i) == OPEN_ROW:
					assets = pm
	_ok("a PopupMenu carries the Open pack in library… row", assets != null)
	if assets == null:
		print("PACKOPEN checks=%d failures=%d" % [_checks, _fails])
		get_tree().quit(1)
		return
	var open_i := -1
	var import_i := -1
	for i in assets.item_count:
		if assets.get_item_text(i) == OPEN_ROW:
			open_i = i
		if assets.get_item_text(i) == IMPORT_ROW:
			import_i = i
	_ok("the same menu carries Import asset pack .zip…, directly above it", import_i >= 0 and open_i == import_i + 1,
		"import=%d open=%d" % [import_i, open_i])
	var t_open := assets.get_item_tooltip(open_i)
	var t_import := assets.get_item_tooltip(import_i) if import_i >= 0 else ""
	_ok("each row has its own tooltip saying what it does to the map",
		t_open.contains("map is not changed") and t_import.contains("onto the map") and t_open != t_import,
		"open='%s' import='%s'" % [t_open, t_import])
	assets.id_pressed.emit(assets.get_item_id(open_i))
	await _frames(10)
	var win: Window = app.asset_library_window
	var pickers: Array = []
	var after: Array = []
	_walk(get_tree().root, after)
	for n in after:
		if n is Window and (n as Window).visible and n != win and String((n as Window).title).contains("Open pack in library"):
			pickers.append(n)
	_ok("pressing the row opens the library window and a picker titled for it",
		win.visible and not pickers.is_empty(), "window=%s pickers=%d" % [win.visible, pickers.size()])
	for p in pickers:
		(p as Window).hide()
	await _frames(4)

	# -- 2. open into the library; the map is untouched -----------------------
	app._import_asset_pack(fixture, false)
	await _frames(12)
	_ok("the fixture opens to 17 library items", _items() == 17, "items=%d" % _items())
	_ok("Open pack in library leaves the map alone", not app.bridge.has_asset_pack())
	var result := _cards("Pack opened")
	_ok("the result card is drawn", result.size() == 1, "cards=%d" % result.size())
	_ok("the status line reports the counts", _hint().contains("17 imported"), _hint())
	await _snap("01_open_result")
	for c in result:
		var done := _button_in(c, "Done")
		if done != null:
			done.pressed.emit()
	await _frames(6)

	# -- 3. conflicts: Cancel, then Replace -----------------------------------
	app._import_asset_pack(fixture, false)
	await _frames(12)
	var conflict := _cards("Replace library art")
	_ok("a second open asks before replacing", conflict.size() == 1, "cards=%d" % conflict.size())
	await _snap("02_conflict")
	if not conflict.is_empty():
		var cancel := _button_in(conflict[0], "Cancel")
		_ok("the card has Cancel and Replace", cancel != null and _button_in(conflict[0], "Replace") != null)
		if cancel != null:
			cancel.pressed.emit()
	await _frames(6)
	_ok("Cancel changes nothing", _items() == 17 and _cards("Pack opened").is_empty(),
		"items=%d hint='%s'" % [_items(), _hint()])
	app._import_asset_pack(fixture, false)
	await _frames(12)
	conflict = _cards("Replace library art")
	if not conflict.is_empty():
		var rep := _button_in(conflict[0], "Replace")
		if rep != null:
			rep.pressed.emit()
	await _frames(12)
	_ok("Replace leaves the same 17 items, not 34", _items() == 17, "items=%d" % _items())

	# -- 4. Show focuses the slot ---------------------------------------------
	result = _cards("Pack opened")
	var show: Button = _button_in(result[0], "Show") if not result.is_empty() else null
	_ok("a result row carries a Show button", show != null)
	if show != null:
		show.pressed.emit()
		await _frames(10)
		_ok("Show closes the card and focuses the first row's slot",
			_cards("Pack opened").is_empty() and String(win.get("_focused_uid")) == "textures:grass",
			"focused='%s'" % String(win.get("_focused_uid")))
		await _snap("03_focused")

	# -- 5. Import pack…: map AND library --------------------------------------
	app.bridge.as_clear_library()
	app._import_asset_pack(fixture, true)
	await _frames(12)
	_ok("Import pack… loads the map AND fills the library",
		app.bridge.has_asset_pack() and _items() == 17,
		"has_asset_pack=%s items=%d" % [app.bridge.has_asset_pack(), _items()])
	for c in _cards("Pack opened"):
		var done := _button_in(c, "Done")
		if done != null:
			done.pressed.emit()
	await _frames(6)

	# -- 6. refusals ------------------------------------------------------------
	app.bridge.as_clear_library()
	var not_zip := _write("user://packopen/not_a_zip.zip", "hello, I am not a zip".to_utf8_buffer())
	app._import_asset_pack(not_zip, false)
	await _frames(6)
	_ok("a non-zip is refused with a reason and imports nothing",
		_items() == 0 and _hint().contains("not opened") and _hint().contains("zip"), _hint())
	var evil := _write("user://packopen/traversal.zip", _zip_with_name("../evil.png"))
	var pack_before: bool = app.bridge.has_asset_pack()
	app._import_asset_pack(evil, true)
	await _frames(6)
	_ok("a path-traversal zip is refused before the map loader sees it",
		_items() == 0 and _hint().contains("../evil.png"), _hint())
	_ok("and the map's pack is whatever it was before", app.bridge.has_asset_pack() == pack_before)
	app.bridge.generating = true
	var busy: Dictionary = app.bridge.as_import_pack(PackedByteArray(), "x.zip", "dry_run")
	app.bridge.generating = false
	_ok("mid-generation the bridge refuses with its busy text",
		not bool(busy.get("ok", true)) and String(busy.get("error", "")).contains("generating"), str(busy))

	print("PACKOPEN checks=%d failures=%d" % [_checks, _fails])
	get_tree().quit(0 if _fails == 0 else 1)
