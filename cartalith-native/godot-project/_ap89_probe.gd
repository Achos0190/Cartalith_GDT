extends Node
## Windowed proof for AP-8 (folder import) and AP-9 (pack diff and merge,
## favourites and recent), `ASSET_PACK_TOOLS_PROPOSAL.md` §6, Ruling CQ.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x900 \
##       _ap89_probe.tscn -- --nowelcome
##   phone composition: ... _ap89_probe.tscn -- --nowelcome --force-touch --vp 1080x2400
##   after either, in a NEW process with the same user dir:
##       ... _ap89_probe.tscn -- --nowelcome --phase restart
## `--vp WxH` hosts the app in a SubViewport of that size (`_packopen_probe`'s
## pattern). Neither is the real device (`MISTAKES.md`: `--force-touch` is not
## the phone).
##
## **Run it from a scratch copy with its own `config/name`, and clear that
## copy's user dir before the first phase**: the first phase deletes
## `user://cartalith_settings.cfg` before the shell boots (it sets up the
## state it asserts on) and the restart phase reads what the first wrote.
## Needs the reference pack at
## `res://../crates/cartalith-assets/tests/fixtures/reference_pack.zip` and a
## DLL with `as_import_folder`, `as_pack_diff` and `as_pack_merge`; exits 2
## and says so when either is missing.
##
## The pickers are not driven: like `_packopen_probe`, it calls the one
## function each picker calls (`app._import_asset_folder(path)`,
## `app._compare_pack(path)`), after checking the menu rows open a picker.
## Slots are focused with `focus_slot`, the function the cards' Show buttons
## call; the star and the rail rows are pressed as buttons.
##
## What the first phase establishes, in order:
##   1. Assets carries Import folder of art… and Compare pack with library…,
##      each with its own tooltip, and pressing the first opens the library
##      window and a folder picker titled for it.
##   2. Compare: the reference pack against a library opened from it gives
##      the "No differences" card; after one variant is removed the card
##      lists the slot as different; Keep library leaves the library's
##      exported pack bytes identical; Take pack makes the diff empty again.
##   3. Folder import of three synthetic trees made here (no third-party
##      art): loose (6 items, 4 slots, licence text in each slot's meta and on
##      the card), Wonderdraft (5 items) and Dungeondraft (5 items), with the
##      expected custom set names; a second import asks before replacing and
##      Cancel changes nothing; an empty folder gives "Nothing to import".
##   4. Favourites and Recent: three focused slots are Recent, newest first;
##      the inspector's star adds the slot to Favourites and the rail count
##      follows; pressing the Favourites row shows exactly that slot; on a
##      phone the FAMILIES pane draws both rows inside the screen.
##   5. Leaves Favourites = [icons:hill, custom:loose_art/castle] and Recent
##      starting [icons:hill, poi:ruin] for the restart phase.
## The restart phase asserts those exact lists from a fresh process, that the
## rail counts only the slot this library knows (1, with the other named in
## the tooltip), and that the Favourites row shows icons:hill.
## Prints `AP89 ok|FAIL <what>` rows and `AP89 checks=<n> failures=<n>`; saves
## PNGs under `user://ap89/`. Exit 0 when every check held, 1 when one
## failed, 2 when it could not run.

const FIXTURE := "res://../crates/cartalith-assets/tests/fixtures/reference_pack.zip"
const FOLDER_ROW := "Import folder of art…"
const COMPARE_ROW := "Compare pack with library…"
## The state the first phase leaves and the restart phase reads back.
const EXPECT_FAVOURITES := ["icons:hill", "custom:loose_art/castle"]
const EXPECT_RECENT_HEAD := ["icons:hill", "poi:ruin"]

var app: Node
var _view: Viewport
var _fails := 0
var _checks := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("AP89 %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _walk(n: Node, out: Array) -> void:
	for c in n.get_children(true):
		out.append(c)
		_walk(c, out)

## Every visible AcceptDialog whose title starts with `prefix`.
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

## All label text under `root`, joined, for "the card says X" checks.
func _text_in(root: Node) -> String:
	var all: Array = []
	_walk(root, all)
	var parts: PackedStringArray = []
	for n in all:
		if n is Label:
			parts.append((n as Label).text)
	return "\n".join(parts)

## Presses a card's button by its text, then lets the card close.
func _press(card_prefix: String, text: String) -> bool:
	var cards := _cards(card_prefix)
	if cards.is_empty():
		return false
	var b := _button_in(cards[0], text)
	if b == null:
		return false
	b.pressed.emit()
	await _frames(8)
	return true

func _close_cards(prefix: String) -> void:
	for c in _cards(prefix):
		var done := _button_in(c, "Done")
		if done != null:
			done.pressed.emit()
	await _frames(6)

func _items() -> int:
	return int(app.bridge.as_pack_info().get("total_items", -1))

func _hint() -> String:
	var labels: Dictionary = app.get("_status_labels")
	return String((labels["hint"] as Label).text) if labels.has("hint") else ""

func _snap(tag: String) -> void:
	await RenderingServer.frame_post_draw
	var img := _view.get_texture().get_image()
	DirAccess.make_dir_recursive_absolute("user://ap89")
	var path := "user://ap89/%s_%s.png" % [tag, "phone" if DccTheme.is_phone() else "desktop"]
	img.save_png(path)
	print("AP89 saved %s" % ProjectSettings.globalize_path(path))

## Custom slots as "set/name×items", in add order.
func _custom() -> PackedStringArray:
	var out := PackedStringArray()
	for s in app.bridge.as_family_slots("custom"):
		out.append("%s/%s×%d" % [String(s.get("set", "")), String(s.get("name", "")), int(s.get("item_count", 0))])
	return out

## Writes a one-colour PNG at `rel` under `root` (absolute), making folders.
func _png(root: String, rel: String, w: int, h: int, c: Color) -> void:
	var path := root.path_join(rel)
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var img := Image.create_empty(w, h, false, Image.FORMAT_RGBA8)
	img.fill(c)
	img.save_png(path)

func _text(root: String, rel: String, body: String) -> void:
	var path := root.path_join(rel)
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(body)
	f.close()

## Three synthetic trees and an empty folder under `user://ap89/trees`,
## rebuilt from nothing each run. Returns their absolute paths.
func _make_trees() -> Dictionary:
	var base := ProjectSettings.globalize_path("user://ap89/trees")
	_rm(base)
	var loose := base.path_join("Loose Art")
	_png(loose, "castle.png", 32, 32, Color(0.8, 0.1, 0.1))
	_png(loose, "tower_01.png", 32, 32, Color(0.1, 0.8, 0.1))
	_png(loose, "tower_02.png", 32, 32, Color(0.1, 0.1, 0.8))
	_png(loose, "trees/oak_1.png", 32, 32, Color(0.2, 0.5, 0.2))
	_png(loose, "trees/oak_2.png", 32, 32, Color(0.3, 0.5, 0.2))
	_png(loose, "trees/pine.png", 32, 32, Color(0.1, 0.4, 0.3))
	_text(loose, "notes.txt", "not an image")
	_text(loose, "LICENSE.txt", "CC BY 4.0, probe author")
	var wd := base.path_join("WD Package")
	_png(wd, "sprites/symbols/castles/keep_01.png", 32, 32, Color(0.5, 0.5, 0.5))
	_png(wd, "sprites/symbols/castles/keep_02.png", 32, 32, Color(0.6, 0.5, 0.5))
	_text(wd, "sprites/symbols/castles/.wonderdraft_symbols", "{}")
	_png(wd, "sprites/trees/conifer/fir.png", 32, 32, Color(0.1, 0.3, 0.1))
	_png(wd, "textures/grass.png", 64, 64, Color(0.2, 0.6, 0.2))
	_png(wd, "textures/cobbles.png", 64, 64, Color(0.4, 0.4, 0.4))
	_png(wd, "brushes/soft.png", 16, 16, Color(0, 0, 0))
	var dd := base.path_join("DD Bits")
	_png(dd, "textures/objects/furniture/chair_1.png", 32, 32, Color(0.5, 0.3, 0.1))
	_png(dd, "textures/objects/furniture/chair_2.png", 32, 32, Color(0.6, 0.3, 0.1))
	_png(dd, "textures/objects/barrel.png", 32, 32, Color(0.4, 0.2, 0.1))
	_png(dd, "textures/terrain/dirt.png", 64, 64, Color(0.5, 0.4, 0.3))
	_png(dd, "textures/terrain/moss_stone.png", 64, 64, Color(0.3, 0.4, 0.3))
	_png(dd, "textures/walls/brick.png", 32, 32, Color(0.6, 0.2, 0.2))
	_text(dd, "data/default.dungeondraft_tags",
		'{"tags":{"Furniture":["textures/objects/furniture/chair_1.png"],"Outdoor":["textures/objects/barrel.png"]}}')
	var empty := base.path_join("Empty")
	_text(empty, "readme.md", "nothing here")
	return {"loose": loose, "wd": wd, "dd": dd, "empty": empty}

## Removes a folder tree under the user dir (the probe's own scratch only).
func _rm(path: String) -> void:
	if not DirAccess.dir_exists_absolute(path):
		return
	var d := DirAccess.open(path)
	for f in d.get_files():
		DirAccess.remove_absolute(path.path_join(f))
	for sub in d.get_directories():
		_rm(path.path_join(sub))
	DirAccess.remove_absolute(path)

func _boot() -> bool:
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
	for m in ["as_import_folder", "as_pack_diff", "as_pack_merge"]:
		if not app.bridge.world_gen.has_method(m):
			print("AP89 CANNOT RUN: this DLL has no %s" % m)
			return false
	print("AP89 view=%s palette=%s phone=%s" % [_view.get_visible_rect().size,
		"dark" if DccTheme.is_dark() else "light", DccTheme.is_phone()])
	return true

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("AP89 REFUSED: headless (the cards are drawn subwindows; run windowed)")
		get_tree().quit(2)
		return
	var fixture := ProjectSettings.globalize_path(FIXTURE)
	if not FileAccess.file_exists(fixture):
		print("AP89 CANNOT RUN: no fixture at %s" % fixture)
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var restart := args.has("--phase") and args.find("restart") > args.find("--phase")
	if not restart and FileAccess.file_exists(DccSettings.CONFIG_PATH):
		## Set up the state this phase asserts on: no stored favourites.
		DirAccess.remove_absolute(ProjectSettings.globalize_path(DccSettings.CONFIG_PATH))
	if not await _boot():
		get_tree().quit(2)
		return
	if restart:
		await _restart_phase()
	else:
		await _first_phase(fixture)
	print("AP89 checks=%d failures=%d" % [_checks, _fails])
	get_tree().quit(0 if _fails == 0 else 1)

func _first_phase(fixture: String) -> void:
	var win: Window = app.asset_library_window
	app.bridge.as_clear_library()
	_ok("CONTROL: the library starts empty and nothing is a favourite",
		_items() == 0 and DccSettings.asset_favourites().is_empty(),
		"items=%d fav=%s" % [_items(), DccSettings.asset_favourites()])

	# -- 1. menu rows ---------------------------------------------------------
	var all: Array = []
	_walk(app, all)
	var assets: PopupMenu = null
	for n in all:
		if n is PopupMenu:
			for i in (n as PopupMenu).item_count:
				if (n as PopupMenu).get_item_text(i) == FOLDER_ROW:
					assets = n
	_ok("a PopupMenu carries Import folder of art…", assets != null)
	if assets == null:
		return
	var fi := -1
	var ci := -1
	for i in assets.item_count:
		if assets.get_item_text(i) == FOLDER_ROW:
			fi = i
		if assets.get_item_text(i) == COMPARE_ROW:
			ci = i
	_ok("the same menu carries Compare pack with library…", ci >= 0, "folder=%d compare=%d" % [fi, ci])
	_ok("each row has its own tooltip",
		assets.get_item_tooltip(fi).contains("Wonderdraft") and assets.get_item_tooltip(ci).contains("slot by slot"))
	assets.id_pressed.emit(assets.get_item_id(fi))
	await _frames(10)
	var pickers: Array = []
	var after: Array = []
	_walk(get_tree().root, after)
	for n in after:
		if n is Window and (n as Window).visible and n != win and String((n as Window).title).contains("Import folder of art"):
			pickers.append(n)
	_ok("pressing it opens the library window and a folder picker titled for it",
		win.visible and not pickers.is_empty(), "window=%s pickers=%d" % [win.visible, pickers.size()])
	for p in pickers:
		(p as Window).hide()
	await _frames(4)

	# -- 2. compare and merge -------------------------------------------------
	app._import_asset_pack(fixture, false)
	await _frames(12)
	await _close_cards("Pack opened")
	_ok("CONTROL: the reference pack opens to 17 items", _items() == 17, "items=%d" % _items())
	app._compare_pack(fixture)
	await _frames(10)
	_ok("the pack against a library opened from it: No differences", _cards("No differences").size() == 1)
	await _snap("01_no_differences")
	await _close_cards("No differences")
	var multi := ""
	for s in app.bridge.as_family_slots("icons"):
		if int(s.get("item_count", 0)) > 1 and multi == "":
			multi = String(s["uid"])
	_ok("CONTROL: the fixture has a slot with more than one variant", multi != "", multi)
	app.bridge.as_remove_item(multi, 0)
	var before: PackedByteArray = app.bridge.as_export_pack_bytes().get("bytes", PackedByteArray())
	app._compare_pack(fixture)
	await _frames(10)
	var cmp := _cards("Compare")
	_ok("after removing a variant the compare card lists a difference", cmp.size() == 1)
	if not cmp.is_empty():
		var t := _text_in(cmp[0])
		_ok("the row names the slot as different, with counts", t.contains("different") and t.contains("library"), t.substr(0, 300))
		_ok("the card offers all three merges", _button_in(cmp[0], "Keep library") != null
			and _button_in(cmp[0], "Keep both") != null and _button_in(cmp[0], "Take pack") != null)
	await _snap("02_compare")
	await _press("Compare", "Keep library")
	var merged := _cards("Merged")
	_ok("Keep library shows what it did", merged.size() == 1)
	await _snap("03_merged_keep")
	await _close_cards("Merged")
	var after_keep: PackedByteArray = app.bridge.as_export_pack_bytes().get("bytes", PackedByteArray())
	_ok("Keep library leaves the library's exported pack byte-identical",
		before.size() > 0 and before == after_keep, "%d vs %d bytes" % [before.size(), after_keep.size()])
	app._compare_pack(fixture)
	await _frames(10)
	await _press("Compare", "Take pack")
	await _close_cards("Merged")
	app._compare_pack(fixture)
	await _frames(10)
	_ok("after Take pack the compare is empty again", _cards("No differences").size() == 1)
	await _close_cards("No differences")

	# -- 3. folder import -----------------------------------------------------
	var trees := _make_trees()
	app.bridge.as_clear_library()
	app._import_asset_folder(trees["wd"])
	await _frames(12)
	_ok("Wonderdraft: 5 items, sets by sprite folder, a slot-named texture in its slot",
		_items() == 5 and _custom() == PackedStringArray(["symbols / castles/keep×2", "trees / conifer/fir×1", "Ground candidates/cobbles×1"]),
		"items=%d custom=%s" % [_items(), _custom()])
	await _close_cards("Folder imported")
	app.bridge.as_clear_library()
	app._import_asset_folder(trees["dd"])
	await _frames(12)
	var barrel: Dictionary = app.bridge.as_slot_summary("custom:dd_bits/barrel")
	_ok("Dungeondraft: 5 items, objects as sets, terrain as ground, tags kept",
		_items() == 5 and _custom() == PackedStringArray(["DD Bits/barrel×1", "furniture/chair×2", "Ground candidates/moss_stone×1"])
		and str(barrel.get("tags", [])).contains("Outdoor"),
		"items=%d custom=%s barrel=%s" % [_items(), _custom(), barrel.get("tags", [])])
	await _close_cards("Folder imported")
	app.bridge.as_clear_library()
	app._import_asset_folder(trees["loose"])
	await _frames(12)
	_ok("loose: 6 items in 4 slots, sets by folder",
		_items() == 6 and _custom() == PackedStringArray(["Loose Art/castle×1", "Loose Art/tower×2", "trees/oak×2", "trees/pine×1"]),
		"items=%d custom=%s" % [_items(), _custom()])
	var castle: Dictionary = app.bridge.as_slot_summary("custom:loose_art/castle")
	_ok("the licence text lands in the slot's licence field",
		String(castle.get("meta_license", "")) == "CC BY 4.0, probe author", String(castle.get("meta_license", "<absent>")))
	var res := _cards("Folder imported")
	_ok("the result card names the layout and shows the licence",
		not res.is_empty() and _text_in(res[0]).contains("Read as a folder of images") and _text_in(res[0]).contains("CC BY 4.0"))
	await _snap("04_folder_result")
	await _close_cards("Folder imported")
	app._import_asset_folder(trees["loose"])
	await _frames(10)
	_ok("a second import asks before replacing", _cards("Replace library art").size() == 1)
	await _press("Replace library art", "Cancel")
	_ok("Cancel changes nothing", _items() == 6 and _cards("Folder imported").is_empty())
	app._import_asset_folder(trees["empty"])
	await _frames(10)
	_ok("a folder with no PNG gives Nothing to import", _cards("Nothing to import").size() == 1, _hint())
	await _snap("05_empty")
	await _close_cards("Nothing to import")

	# -- 4. favourites and recent ---------------------------------------------
	for uid in ["textures:grass", "poi:ruin", "icons:hill"]:
		win.focus_slot(uid)
		await _frames(3)
	var recent := DccSettings.asset_recent()
	_ok("three focused slots are Recent, newest first",
		recent.size() >= 3 and recent[0] == "icons:hill" and recent[1] == "poi:ruin" and recent[2] == "textures:grass", str(recent))
	var star: Button = win.get("_insp_star")
	_ok("the inspector shows an unlit star for the focused slot", star != null and star.is_visible_in_tree() and star.text == "☆",
		"%s" % (star.text if star != null else "null"))
	star.pressed.emit()
	await _frames(4)
	_ok("pressing it lights the star and stores the favourite",
		star.text == "★" and DccSettings.asset_favourites() == PackedStringArray(["icons:hill"]), str(DccSettings.asset_favourites()))
	var rows: Dictionary = win.get("_pinned_rows")
	var fav_count: Label = rows["favourites"]["count"]
	var rec_count: Label = rows["recent"]["count"]
	await _snap("06_star_lit")
	if DccTheme.is_phone():
		## Measured on the SLOT pane, where the star is drawn (focus moved there).
		var star_rect: Rect2 = star.get_global_rect()
		_ok("phone: the inspector star is at least 44 x 44 dp",
			star.is_visible_in_tree() and star_rect.size.x >= 43.5 and star_rect.size.y >= 43.5,
			"star %s dp" % star_rect.size)
	_ok("the rail counts follow", fav_count.text == "1" and int(rec_count.text) >= 3, "fav=%s recent=%s" % [fav_count.text, rec_count.text])
	if DccTheme.is_phone():
		var sw: Dictionary = win.get("_phone_pane_buttons")
		(sw["families"] as Button).pressed.emit()
		await _frames(6)
		var fb: Button = rows["favourites"]["button"]
		var rb: Button = rows["recent"]["button"]
		## Control rects here are in phone px (dp): the shell lays the phone
		## out on a 412 dp canvas and scales it by `DccTheme.phone_scale()` to
		## the physical 1080 px, so the screen in the same units is the
		## viewport's physical size divided by that factor.
		var screen := Rect2(Vector2.ZERO, _view.get_visible_rect().size / DccTheme.phone_scale())
		var fr := fb.get_global_rect()
		var rr := rb.get_global_rect()
		_ok("phone: the FAMILIES pane draws both rows inside the screen",
			fb.is_visible_in_tree() and rb.is_visible_in_tree() and screen.encloses(fr) and screen.encloses(rr),
			"screen=%s fav=%s recent=%s" % [screen, fr, rr])
		## The touch floor is 44 dp, in the same phone-px units as the rects.
		_ok("phone: each row is at least the 44 dp touch floor tall, and wide",
			fr.size.y >= 43.5 and rr.size.y >= 43.5 and fr.size.x >= 44.0 and rr.size.x >= 44.0,
			"fav %s recent %s dp; phone_scale=%.2f" % [fr.size, rr.size, DccTheme.phone_scale()])
		await _snap("06_phone_families")
	(rows["favourites"]["button"] as Button).pressed.emit()
	await _frames(8)
	var order: Array = win.get("_slot_order")
	_ok("pressing Favourites shows exactly the starred slot",
		order.size() == 1 and String(order[0]["uid"]) == "icons:hill", str(order))
	await _snap("07_favourites_grid")
	(rows["recent"]["button"] as Button).pressed.emit()
	await _frames(8)
	order = win.get("_slot_order")
	_ok("pressing Recent shows the recent slots newest first",
		order.size() >= 3 and String(order[0]["uid"]) == "icons:hill", str(order).substr(0, 200))

	# -- 5. the state the restart phase reads -----------------------------------
	win.focus_slot("custom:loose_art/castle")
	await _frames(3)
	star = win.get("_insp_star")
	star.pressed.emit()
	await _frames(3)
	win.focus_slot("poi:ruin")
	await _frames(3)
	win.focus_slot("icons:hill")
	await _frames(3)
	_ok("leaves the favourites and recent the restart phase expects",
		DccSettings.asset_favourites() == PackedStringArray(EXPECT_FAVOURITES)
		and DccSettings.asset_recent().slice(0, 2) == PackedStringArray(EXPECT_RECENT_HEAD),
		"fav=%s recent=%s" % [DccSettings.asset_favourites(), DccSettings.asset_recent()])

func _restart_phase() -> void:
	var win: Window = app.asset_library_window
	_ok("a fresh process reads the favourites the first phase stored",
		DccSettings.asset_favourites() == PackedStringArray(EXPECT_FAVOURITES), str(DccSettings.asset_favourites()))
	_ok("and the recent list, newest first",
		DccSettings.asset_recent().slice(0, 2) == PackedStringArray(EXPECT_RECENT_HEAD), str(DccSettings.asset_recent()))
	app.open_asset_library()
	await _frames(10)
	var rows: Dictionary = win.get("_pinned_rows")
	var fav_count: Label = rows["favourites"]["count"]
	var fb: Button = rows["favourites"]["button"]
	_ok("the rail counts only the favourite this library knows, and names the other",
		fav_count.text == "1" and fb.tooltip_text.contains("1 not in this library now"),
		"count=%s tip=%s" % [fav_count.text, fb.tooltip_text])
	fb.pressed.emit()
	await _frames(8)
	var order: Array = win.get("_slot_order")
	_ok("the Favourites row shows icons:hill after the restart",
		order.size() == 1 and String(order[0]["uid"]) == "icons:hill", str(order))
	await _snap("08_after_restart")
